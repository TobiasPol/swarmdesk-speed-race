//! Runs a vetted module on one input and counts the fuel it burns.
//! Every case gets a fresh instance, so nothing carries over between inputs.

use crate::module;
use wasmtime::{Config, Engine, Instance, Module, Store, StoreLimits, StoreLimitsBuilder, Trap};

pub const MEMORY_LIMIT: usize = 64 << 20;
const PAGE: usize = 65_536;

pub struct Program {
    module: Module,
}

pub struct Outcome {
    pub output: Vec<u8>,
    pub fuel: u64,
}

pub fn engine() -> Result<Engine, String> {
    let mut config = Config::new();
    config.consume_fuel(true);
    config.cranelift_nan_canonicalization(true);
    config.wasm_relaxed_simd(false);
    Engine::new(&config).map_err(|e| format!("the engine did not start: {e}"))
}

impl Program {
    pub fn load(engine: &Engine, wasm: &[u8]) -> Result<Program, String> {
        module::vet(wasm)?;
        let module =
            Module::new(engine, wasm).map_err(|_| "the module cannot be compiled".to_string())?;
        Ok(Program { module })
    }
}

/// Runs `run(input, output buffer)` once and returns the output and the fuel used.
pub fn run_case(
    engine: &Engine,
    program: &Program,
    input: &[u8],
    out_cap: usize,
    fuel_cap: u64,
) -> Result<Outcome, String> {
    let limits = StoreLimitsBuilder::new()
        .memory_size(MEMORY_LIMIT)
        .instances(1)
        .build();
    let mut store: Store<StoreLimits> = Store::new(engine, limits);
    store.limiter(|limits| limits);
    store.set_fuel(fuel_cap).map_err(|e| e.to_string())?;
    let instance = Instance::new(&mut store, &program.module, &[])
        .map_err(|_| "the module cannot be instantiated".to_string())?;
    let memory = instance
        .get_memory(&mut store, "memory")
        .ok_or("the module does not export its memory")?;
    let run = instance
        .get_typed_func::<(i32, i32, i32, i32), i32>(&mut store, "run")
        .map_err(|_| "the module has no `run` with the expected type".to_string())?;

    let base = memory.data_size(&store);
    let out_at = base + input.len().div_ceil(16) * 16;
    let pages = (out_at - base + out_cap).div_ceil(PAGE) + 1;
    memory
        .grow(&mut store, pages as u64)
        .map_err(|_| "the memory limit was reached".to_string())?;
    memory
        .write(&mut store, base, input)
        .map_err(|e| e.to_string())?;

    let args = (
        base as i32,
        input.len() as i32,
        out_at as i32,
        out_cap as i32,
    );
    let before = store.get_fuel().map_err(|e| e.to_string())?;
    let returned = run.call(&mut store, args).map_err(trap_reason)?;
    let fuel = before - store.get_fuel().map_err(|e| e.to_string())?;

    let len = returned as u32 as usize;
    if len > out_cap {
        return Err("returned a length beyond the output buffer".into());
    }
    let mut output = vec![0u8; len];
    memory
        .read(&store, out_at, &mut output)
        .map_err(|e| e.to_string())?;
    Ok(Outcome { output, fuel })
}

fn trap_reason(error: wasmtime::Error) -> String {
    match error.downcast_ref::<Trap>() {
        Some(Trap::OutOfFuel) => "ran out of fuel".into(),
        _ => "trapped".into(),
    }
}
