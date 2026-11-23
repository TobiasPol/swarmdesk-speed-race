//! The Swarmdesk speed race checker.
//!
//! A submission is one Rust source file that defines
//! `pub fn solve(input: &[u8], output: &mut [u8]) -> usize`. The checker
//! guards the file, compiles it to WebAssembly, and runs it and the current
//! record holder on the same fresh inputs. The score is the fuel wasmtime
//! counts, so it is the same on every machine.

pub mod build;
pub mod guard;
pub mod judge;
pub mod module;
pub mod rng;
pub mod run;
pub mod targets;

use std::path::PathBuf;

/// A random seed from the operating system, for a run nobody can predict.
pub fn fresh_seed() -> u64 {
    use std::io::Read;
    let mut bytes = [0u8; 8];
    let from_os = std::fs::File::open("/dev/urandom").and_then(|mut f| f.read_exact(&mut bytes));
    if from_os.is_ok() {
        return u64::from_le_bytes(bytes);
    }
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH);
    now.map(|d| d.as_nanos() as u64).unwrap_or(1)
}

/// A scratch directory that no other run shares.
pub fn work_dir() -> PathBuf {
    let name = format!("sdcheck-{}-{:x}", std::process::id(), fresh_seed());
    std::env::temp_dir().join(name)
}
