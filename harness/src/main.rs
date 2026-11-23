//! Command line for the checker.
//!
//!   sdcheck targets
//!   sdcheck guard <file>
//!   sdcheck judge --target <name> --candidate <file> --holder <file> [--seed <n>] [--json <file>]
//!   sdcheck selftest
//!
//! Exit codes: 0 a verdict was reached (read it), 1 FAULT, 2 bad usage.

use sdcheck::judge::{judge, Report};
use sdcheck::rng::Rng;
use sdcheck::targets::{self, Target};
use sdcheck::{build, guard, run};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        Some("targets") => list_targets(),
        Some("guard") => guard_file(&args),
        Some("judge") => judge_command(&args),
        Some("selftest") => selftest(),
        _ => Err(usage()),
    };
    match result {
        Ok(code) => code,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::from(2)
        }
    }
}

fn usage() -> String {
    "usage: sdcheck targets | guard <file> | judge --target <name> --candidate <file> \
     --holder <file> [--seed <n>] [--json <file>] | selftest"
        .to_string()
}

fn flag<'a>(args: &'a [String], name: &str) -> Option<&'a str> {
    let at = args.iter().position(|a| a == name)?;
    args.get(at + 1).map(String::as_str)
}

fn list_targets() -> Result<ExitCode, String> {
    for target in targets::all() {
        println!("{}\t{}", target.name, target.summary);
    }
    Ok(ExitCode::SUCCESS)
}

fn guard_file(args: &[String]) -> Result<ExitCode, String> {
    let path = args.get(1).ok_or_else(usage)?;
    let source = std::fs::read(path).map_err(|e| format!("{path}: {e}"))?;
    match guard::check(&source) {
        Ok(()) => println!("guard: pass"),
        Err(reason) => println!("{reason}"),
    }
    Ok(ExitCode::SUCCESS)
}

fn judge_command(args: &[String]) -> Result<ExitCode, String> {
    let name = flag(args, "--target").ok_or_else(usage)?;
    let target = targets::find(name).ok_or(format!("no such target: {name}"))?;
    let candidate = PathBuf::from(flag(args, "--candidate").ok_or_else(usage)?);
    let holder = PathBuf::from(flag(args, "--holder").ok_or_else(usage)?);
    let seed = match flag(args, "--seed") {
        Some(text) => text
            .parse::<u64>()
            .map_err(|_| "--seed must be a number".to_string())?,
        None => sdcheck::fresh_seed(),
    };
    let work = sdcheck::work_dir();
    let report = judge(target, &candidate, &holder, seed, &work);
    if std::env::var("SD_KEEP").is_err() {
        let _ = std::fs::remove_dir_all(&work);
    }
    print_report(&report);
    if let Some(path) = flag(args, "--json") {
        std::fs::write(path, report.to_json()).map_err(|e| format!("{path}: {e}"))?;
    }
    Ok(if report.verdict == "FAULT" {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    })
}

fn print_report(report: &Report) {
    println!("{}", report.line());
    if !report.detail.is_empty() {
        println!("{}", report.detail);
    }
    if report.holder_fuel > 0 {
        println!(
            "target {}  seed {}  cases {}  fuel {} vs record holder {}",
            report.target, report.seed, report.cases, report.candidate_fuel, report.holder_fuel
        );
    } else {
        println!(
            "target {}  seed {}  cases {}",
            report.target, report.seed, report.cases
        );
    }
}

/// Checks every record holder in `targets/`: it passes the guard, builds, and
/// gives the reference answer on three seeds.
fn selftest() -> Result<ExitCode, String> {
    let work = sdcheck::work_dir();
    let engine = run::engine()?;
    let mut failed = false;
    for target in targets::all() {
        let result = selftest_target(target, &engine, &work);
        match &result {
            Ok(fuel) => println!("ok    {:<16} fuel {fuel}", target.name),
            Err(reason) => println!("FAIL  {:<16} {reason}", target.name),
        }
        failed |= result.is_err();
    }
    let _ = std::fs::remove_dir_all(&work);
    Ok(if failed {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    })
}

fn selftest_target(target: &Target, engine: &wasmtime::Engine, work: &Path) -> Result<u64, String> {
    let source = PathBuf::from("targets").join(target.name).join("holder.rs");
    let bytes = std::fs::read(&source).map_err(|e| format!("{}: {e}", source.display()))?;
    guard::check(&bytes)?;
    let wasm = build::compile(&source, work, target.name)?;
    let program = run::Program::load(engine, &wasm)?;
    let mut fuel = 0;
    for seed in 1..=3 {
        for case in (target.generate)(&mut Rng::new(seed)) {
            let cap = (target.out_cap)(case.input.len());
            let outcome = run::run_case(engine, &program, &case.input, cap, 2_000_000_000)
                .map_err(|e| format!("{e} on a {} input", case.class))?;
            if outcome.output != (target.reference)(&case.input) {
                return Err(format!("wrong answer on a {} input", case.class));
            }
            fuel += outcome.fuel;
        }
    }
    Ok(fuel)
}
