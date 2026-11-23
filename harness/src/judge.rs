//! Decides one submission: guard, build, then run it and the record holder on
//! the same fresh inputs. The verdict comes only from what this process
//! measures; nothing a submission prints or returns besides its output counts.

use crate::build;
use crate::guard;
use crate::rng::Rng;
use crate::run::{self, Program};
use crate::targets::{Case, Target};
use std::path::Path;
use wasmtime::Engine;

/// A submission must burn at least this much less fuel than the record holder.
pub const MARGIN_PERCENT: u64 = 3;
const HOLDER_FUEL_CAP: u64 = 2_000_000_000;

pub struct Report {
    pub verdict: &'static str,
    pub reason: String,
    pub target: &'static str,
    pub seed: u64,
    pub cases: usize,
    pub candidate_fuel: u64,
    pub holder_fuel: u64,
}

impl Report {
    fn new(target: &'static str, seed: u64) -> Report {
        Report {
            verdict: "FAULT",
            reason: String::new(),
            target,
            seed,
            cases: 0,
            candidate_fuel: 0,
            holder_fuel: 0,
        }
    }

    fn end(mut self, verdict: &'static str, reason: impl Into<String>) -> Report {
        self.verdict = verdict;
        self.reason = reason.into();
        self
    }

    /// One line for a person: `ACCEPTED`, `REJECTED (<reason>)` or `FAULT (<reason>)`.
    pub fn line(&self) -> String {
        if self.reason.is_empty() {
            self.verdict.to_string()
        } else {
            format!("{} ({})", self.verdict, self.reason)
        }
    }

    pub fn to_json(&self) -> String {
        let reason = self.reason.replace('\\', "\\\\").replace('"', "\\\"");
        format!(
            concat!(
                "{{\"verdict\":\"{}\",\"reason\":\"{}\",\"target\":\"{}\",\"seed\":\"{}\",",
                "\"cases\":{},\"candidate_fuel\":{},\"holder_fuel\":{},\"margin_percent\":{}}}"
            ),
            self.verdict,
            reason,
            self.target,
            self.seed,
            self.cases,
            self.candidate_fuel,
            self.holder_fuel,
            MARGIN_PERCENT
        )
    }
}

/// Judges `candidate` against `holder` for `target` with inputs from `seed`.
pub fn judge(
    target: &'static Target,
    candidate: &Path,
    holder: &Path,
    seed: u64,
    work: &Path,
) -> Report {
    let report = Report::new(target.name, seed);
    let source = match std::fs::read(candidate) {
        Ok(bytes) => bytes,
        Err(_) => return report.end("FAULT", "the submission could not be read"),
    };
    if let Err(reason) = guard::check(&source) {
        return report.end("REJECTED", reason);
    }
    let engine = match run::engine() {
        Ok(engine) => engine,
        Err(reason) => return report.end("FAULT", reason),
    };
    let holder_program = match load(&engine, holder, work, "holder") {
        Ok(program) => program,
        Err(reason) => return report.end("FAULT", format!("record holder: {reason}")),
    };
    let candidate_program = match load(&engine, candidate, work, "candidate") {
        Ok(program) => program,
        Err(reason) => return report.end("REJECTED", reason),
    };
    let cases = (target.generate)(&mut Rng::new(seed));
    race(
        report,
        target,
        &engine,
        &candidate_program,
        &holder_program,
        &cases,
    )
}

fn load(engine: &Engine, source: &Path, work: &Path, label: &str) -> Result<Program, String> {
    let wasm = build::compile(source, work, label)?;
    Program::load(engine, &wasm)
}

fn race(
    mut report: Report,
    target: &Target,
    engine: &Engine,
    candidate: &Program,
    holder: &Program,
    cases: &[Case],
) -> Report {
    report.cases = cases.len();
    for case in cases {
        let expected = (target.reference)(&case.input);
        let cap = (target.out_cap)(case.input.len());
        let held = match run::run_case(engine, holder, &case.input, cap, HOLDER_FUEL_CAP) {
            Ok(outcome) if outcome.output == expected => outcome,
            _ => return report.end("FAULT", "the record holder gave a wrong answer"),
        };
        let fuel_cap = held.fuel.saturating_mul(8).saturating_add(100_000);
        match run::run_case(engine, candidate, &case.input, cap, fuel_cap) {
            Ok(outcome) if outcome.output == expected => report.candidate_fuel += outcome.fuel,
            Ok(_) => {
                let why = format!("wrong answer on a {} input", case.class);
                return report.end("REJECTED", why);
            }
            Err(reason) => {
                let why = format!("{reason} on a {} input", case.class);
                return report.end("REJECTED", why);
            }
        }
        report.holder_fuel += held.fuel;
    }
    if wins(report.candidate_fuel, report.holder_fuel) {
        return report.end("ACCEPTED", "");
    }
    let why = format!("not at least {MARGIN_PERCENT} percent less fuel than the record holder");
    report.end("REJECTED", why)
}

/// True when the candidate burns at least `MARGIN_PERCENT` less fuel.
pub fn wins(candidate_fuel: u64, holder_fuel: u64) -> bool {
    u128::from(candidate_fuel) * 100 <= u128::from(holder_fuel) * u128::from(100 - MARGIN_PERCENT)
}

#[cfg(test)]
mod tests {
    use super::wins;

    #[test]
    fn margin_is_exact() {
        assert!(wins(97, 100));
        assert!(!wins(98, 100));
        assert!(!wins(100, 100));
        assert!(wins(0, 100));
    }
}
