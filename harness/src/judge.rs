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
    /// How many sets of inputs were run. The deciding check runs several, each from a new seed.
    pub runs: usize,
    pub candidate_fuel: u64,
    pub holder_fuel: u64,
    /// The record holder gave a wrong answer on at least one input.
    pub holder_faulty: bool,
    /// Help for the author on their own machine: which input failed and how. Printed by
    /// the command line, never sent anywhere.
    pub detail: String,
}

impl Report {
    fn new(target: &'static str, seed: u64) -> Report {
        Report {
            verdict: "FAULT",
            reason: String::new(),
            target,
            seed,
            cases: 0,
            runs: 0,
            candidate_fuel: 0,
            holder_fuel: 0,
            holder_faulty: false,
            detail: String::new(),
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
                "\"cases\":{},\"runs\":{},\"candidate_fuel\":{},\"holder_fuel\":{},",
                "\"holder_faulty\":{},\"margin_percent\":{}}}"
            ),
            self.verdict,
            reason,
            self.target,
            self.seed,
            self.cases,
            self.runs,
            self.candidate_fuel,
            self.holder_fuel,
            self.holder_faulty,
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
    judge_runs(target, candidate, holder, &[seed], work)
}

/// Judges on one set of inputs per seed. The candidate must answer every input of every set
/// correctly, and the margin is taken over all sets together, so one lucky set decides nothing.
pub fn judge_runs(
    target: &'static Target,
    candidate: &Path,
    holder: &Path,
    seeds: &[u64],
    work: &Path,
) -> Report {
    let mut report = Report::new(target.name, seeds.first().copied().unwrap_or(0));
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
    for &seed in seeds {
        let cases = (target.generate)(&mut Rng::new(seed));
        let programs = (&candidate_program, &holder_program);
        if let Err(reason) = race(&mut report, target, &engine, programs, &cases, seed) {
            // A run that stops early has no meaningful totals.
            (report.candidate_fuel, report.holder_fuel) = (0, 0);
            return report.end("REJECTED", reason);
        }
        report.runs += 1;
    }
    decide(report)
}

/// The verdict for a candidate that answered every input correctly.
fn decide(report: Report) -> Report {
    // A record holder that answers wrongly is no record. A correct file replaces it whatever
    // its fuel, so that a faulty record holder can never block its function.
    if report.holder_faulty {
        let why = "replaces a record holder that gave a wrong answer";
        return report.end("ACCEPTED", why);
    }
    if wins(report.candidate_fuel, report.holder_fuel) {
        return report.end("ACCEPTED", "");
    }
    let why = format!("not at least {MARGIN_PERCENT} percent less fuel than the record holder");
    report.end("REJECTED", why)
}

fn load(engine: &Engine, source: &Path, work: &Path, label: &str) -> Result<Program, String> {
    let wasm = build::compile(source, work, label)?;
    Program::load(engine, &wasm)
}

/// Runs one set of inputs. `Err` holds the reason the candidate is rejected.
fn race(
    report: &mut Report,
    target: &Target,
    engine: &Engine,
    (candidate, holder): (&Program, &Program),
    cases: &[Case],
    seed: u64,
) -> Result<(), String> {
    report.cases += cases.len();
    for (index, case) in cases.iter().enumerate() {
        let expected = (target.reference)(&case.input);
        let cap = (target.out_cap)(case.input.len());
        let held = match run::run_case(engine, holder, &case.input, cap, HOLDER_FUEL_CAP) {
            Ok(outcome) if outcome.output == expected => Some(outcome.fuel),
            _ => None,
        };
        let fuel_cap = match held {
            Some(fuel) => fuel.saturating_mul(8).saturating_add(100_000),
            None => HOLDER_FUEL_CAP,
        };
        let failure = match run::run_case(engine, candidate, &case.input, cap, fuel_cap) {
            Ok(outcome) if outcome.output == expected => {
                // Fuel is compared only on inputs both files answered correctly.
                match held {
                    Some(fuel) => {
                        report.candidate_fuel += outcome.fuel;
                        report.holder_fuel += fuel;
                    }
                    None => report.holder_faulty = true,
                }
                continue;
            }
            Ok(outcome) => (
                "wrong answer".to_string(),
                mismatch(&expected, &outcome.output),
            ),
            Err(reason) => (reason, String::new()),
        };
        report.detail = format!(
            "stopped at input {} of {} in run {} with seed {} ({}, {} bytes). {}",
            index + 1,
            cases.len(),
            report.runs + 1,
            seed,
            case.class,
            case.input.len(),
            failure.1
        );
        return Err(format!("{}, input class: {}", failure.0, case.class));
    }
    Ok(())
}

/// Says where two answers part ways, for an author debugging on their own machine.
fn mismatch(expected: &[u8], got: &[u8]) -> String {
    let shared = expected.len().min(got.len());
    let lengths = format!("expected {} bytes, got {}", expected.len(), got.len());
    match (0..shared).find(|&i| expected[i] != got[i]) {
        Some(at) => format!(
            "{lengths}; first difference at byte {at}: expected 0x{:02x}, got 0x{:02x}",
            expected[at], got[at]
        ),
        None => lengths,
    }
}

/// True when the candidate burns at least `MARGIN_PERCENT` less fuel.
pub fn wins(candidate_fuel: u64, holder_fuel: u64) -> bool {
    u128::from(candidate_fuel) * 100 <= u128::from(holder_fuel) * u128::from(100 - MARGIN_PERCENT)
}

#[cfg(test)]
mod tests {
    use super::{mismatch, wins};

    #[test]
    fn a_mismatch_names_the_first_differing_byte() {
        assert_eq!(
            mismatch(&[1, 2, 3, 4], &[1, 2, 9, 4]),
            "expected 4 bytes, got 4; first difference at byte 2: expected 0x03, got 0x09"
        );
        assert_eq!(mismatch(&[1, 2, 3], &[1, 2]), "expected 3 bytes, got 2");
    }

    #[test]
    fn margin_is_exact() {
        assert!(wins(97, 100));
        assert!(!wins(98, 100));
        assert!(!wins(100, 100));
        assert!(wins(0, 100));
    }
}
