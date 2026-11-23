# Swarmdesk speed race

## Overview
The problem repository for the Swarmdesk speed race: a checker that judges one Rust source
file against the current record holder by counted WebAssembly instructions, plus the record
holder for each function.

## Tech Stack
- Language: Rust (pinned in `rust-toolchain.toml`), with a little Bash and Node for CI
- Engine: wasmtime (pinned in `harness/Cargo.toml`)
- CI: GitHub Actions

## Commands
- `cargo build --release` — build the checker `sdcheck`
- `cargo test --release` — run the checker's tests
- `cargo run --release -- selftest` — check every record holder against its reference
- `cargo fmt --all --check && cargo clippy --release --all-targets -- -D warnings` — style
- `node --test .github/scripts/` — tests for the pull request guard
- `scripts/check.sh <target>` — judge a working copy against the record holder on main

Use rustup's compiler: `export PATH="$HOME/.cargo/bin:$PATH"`.

## Architecture
- `harness/src/guard.rs` — the plain-file guard (lexer rules)
- `harness/src/build.rs` — compiles a submission inside a crate root the checker owns
- `harness/src/module.rs` — vets the WebAssembly module
- `harness/src/run.rs` — runs one input and counts fuel
- `harness/src/judge.rs` — the verdict
- `harness/src/targets/` — per function: reference implementation and input generator
- `targets/<name>/holder.rs` — record holders; these are submissions, so no comments or strings
- `.github/workflows/deciding.yml`, `scripts/ci-judge.sh`, `scripts/ci-report.sh` — the check
  that counts

## Rules
- Record holder files are replaced only by the deciding workflow. Do not edit them by hand.
- A change to the guard, the engine settings or the wasmtime version changes what a score
  means. Say so in the pull request.
- No file over 300 lines, no function over 50.

## Environment Variables
None are needed to build or test. The deciding workflow reads:
- `SWARMDESK_URL` (repository variable) — the site that lists open steps and receives verdicts
- `SWARMDESK_VERDICT_SECRET` (repository secret) — signs each verdict sent to the site
