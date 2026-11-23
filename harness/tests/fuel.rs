//! Holds the engine to the fuel prices the race relies on. If a new wasmtime
//! version charges differently, these fail and the scoring rule must be revisited.

use sdcheck::module::vet;
use sdcheck::run::{engine, run_case, Program};

const COPY: &str = r#"(module (memory (export "memory") 1)
  (func (export "run") (param i32 i32 i32 i32) (result i32)
    (memory.copy (local.get 2) (local.get 0) (local.get 1))
    (local.get 1)))"#;

const FILL: &str = r#"(module (memory (export "memory") 1)
  (func (export "run") (param i32 i32 i32 i32) (result i32)
    (memory.fill (local.get 2) (i32.const 7) (local.get 3))
    (local.get 3)))"#;

fn fuel(wat: &str, len: usize) -> u64 {
    let engine = engine().unwrap();
    let program = Program::load(&engine, &wat::parse_str(wat).unwrap()).unwrap();
    let input = vec![1u8; len];
    run_case(&engine, &program, &input, len, 1_000_000_000)
        .unwrap()
        .fuel
}

#[test]
fn a_bulk_copy_costs_one_unit_per_byte() {
    assert_eq!(fuel(COPY, 60_000) - fuel(COPY, 1_000), 59_000);
}

#[test]
fn a_bulk_fill_costs_one_unit_per_byte() {
    assert_eq!(fuel(FILL, 60_000) - fuel(FILL, 1_000), 59_000);
}

#[test]
fn the_same_module_and_input_burn_the_same_fuel() {
    assert_eq!(fuel(COPY, 4_096), fuel(COPY, 4_096));
}

#[test]
fn instructions_that_escape_the_count_are_refused() {
    let passive = r#"(module (memory (export "memory") 1) (data $d "abc")
      (func (export "run") (param i32 i32 i32 i32) (result i32)
        (memory.init $d (i32.const 0) (i32.const 0) (i32.const 3)) (i32.const 0)))"#;
    let imports = r#"(module (import "env" "f" (func)) (memory (export "memory") 1))"#;
    let start = r#"(module (memory (export "memory") 1) (func $s) (start $s))"#;
    let threads = r#"(module (memory (export "memory") 1 1 shared))"#;
    for bad in [passive, imports, start, threads] {
        assert!(vet(&wat::parse_str(bad).unwrap()).is_err(), "{bad}");
    }
}
