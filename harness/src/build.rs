//! Compiles one submitted source file to WebAssembly. The submission is a
//! plain module; this file owns the crate root around it, so a submission
//! needs no attributes and no string literals.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// The pinned compiler. `rust-toolchain.toml` must name the same version.
pub const TOOLCHAIN: &str = "1.100.0";
const TIMEOUT: Duration = Duration::from_secs(180);

const ROOT: &str = r#"#![no_std]
#[path = "submission.rs"]
mod submission;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

#[no_mangle]
pub unsafe extern "C" fn run(inp: *const u8, len: usize, out: *mut u8, cap: usize) -> usize {
    submission::solve(
        core::slice::from_raw_parts(inp, len),
        core::slice::from_raw_parts_mut(out, cap),
    )
}
"#;

/// Builds `source` inside `work/label` and returns the module's bytes.
pub fn compile(source: &Path, work: &Path, label: &str) -> Result<Vec<u8>, String> {
    let dir = work.join(label);
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    std::fs::copy(source, dir.join("submission.rs")).map_err(|e| e.to_string())?;
    std::fs::write(dir.join("root.rs"), ROOT).map_err(|e| e.to_string())?;
    let log = std::fs::File::create(dir.join("rustc.log")).map_err(|e| e.to_string())?;
    let mut child = Command::new(rustc_path())
        .current_dir(&dir)
        .env_clear()
        .envs(passed_env())
        .env("RUSTUP_TOOLCHAIN", TOOLCHAIN)
        .args(["--edition", "2021", "--target", "wasm32-unknown-unknown"])
        .args(["--crate-type", "cdylib", "--crate-name", "submission"])
        .args([
            "-C",
            "opt-level=3",
            "-C",
            "panic=abort",
            "-C",
            "codegen-units=1",
        ])
        .args([
            "-C",
            "strip=symbols",
            "-C",
            "target-feature=+simd128",
            "-A",
            "warnings",
        ])
        .args(["-o", "out.wasm", "root.rs"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(log)
        .spawn()
        .map_err(|e| format!("the compiler did not start: {e}"))?;
    let started = Instant::now();
    loop {
        match child.try_wait().map_err(|e| e.to_string())? {
            Some(status) if status.success() => break,
            Some(_) => return Err("does not compile".into()),
            None if started.elapsed() > TIMEOUT => {
                let _ = child.kill();
                return Err("the compiler ran out of time".into());
            }
            None => std::thread::sleep(Duration::from_millis(40)),
        }
    }
    std::fs::read(dir.join("out.wasm")).map_err(|e| e.to_string())
}

/// The compiler sees only these variables, so no credential of the caller reaches it.
fn passed_env() -> Vec<(String, String)> {
    ["PATH", "HOME", "RUSTUP_HOME", "CARGO_HOME", "TMPDIR"]
        .iter()
        .filter_map(|name| {
            std::env::var(name)
                .ok()
                .map(|value| (name.to_string(), value))
        })
        .collect()
}

/// Where the compiler's messages for `label` were written.
pub fn log_path(work: &Path, label: &str) -> PathBuf {
    work.join(label).join("rustc.log")
}

/// `SD_RUSTC` wins; otherwise ask rustup for the pinned compiler; otherwise `rustc`.
fn rustc_path() -> PathBuf {
    if let Ok(path) = std::env::var("SD_RUSTC") {
        return PathBuf::from(path);
    }
    let asked = Command::new("rustup")
        .args(["which", "rustc", "--toolchain", TOOLCHAIN])
        .output();
    match asked {
        Ok(out) if out.status.success() => {
            PathBuf::from(String::from_utf8_lossy(&out.stdout).trim().to_string())
        }
        _ => PathBuf::from("rustc"),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn toolchain_file_names_the_same_version() {
        let file = include_str!("../../rust-toolchain.toml");
        assert!(file.contains(&format!("channel = \"{}\"", super::TOOLCHAIN)));
    }
}
