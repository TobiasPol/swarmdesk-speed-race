//! Checks a compiled WebAssembly module before it is ever run: which features
//! it uses, that it imports nothing, and that no instruction escapes the fuel
//! count. Fuel in wasmtime 51 charges bulk memory copies and fills per byte,
//! so those stay allowed; table and reference instructions are refused.

use wasmparser::{Operator, Parser, Payload, Validator, WasmFeatures};

pub const MAX_MODULE_BYTES: usize = 1 << 20;

fn features() -> WasmFeatures {
    WasmFeatures::MUTABLE_GLOBAL
        | WasmFeatures::SIGN_EXTENSION
        | WasmFeatures::SATURATING_FLOAT_TO_INT
        | WasmFeatures::MULTI_VALUE
        | WasmFeatures::BULK_MEMORY
        | WasmFeatures::REFERENCE_TYPES
        | WasmFeatures::SIMD
        | WasmFeatures::FLOATS
}

/// Returns `Ok(())` when the module may be run, or one line saying why not.
pub fn vet(bytes: &[u8]) -> Result<(), String> {
    if bytes.len() > MAX_MODULE_BYTES {
        return Err(format!(
            "the module is larger than {MAX_MODULE_BYTES} bytes"
        ));
    }
    Validator::new_with_features(features())
        .validate_all(bytes)
        .map_err(|_| "the module uses a WebAssembly feature that is not allowed".to_string())?;
    for payload in Parser::new(0).parse_all(bytes) {
        let payload = payload.map_err(|_| "the module cannot be parsed".to_string())?;
        section_rule(&payload)?;
    }
    Ok(())
}

fn section_rule(payload: &Payload<'_>) -> Result<(), String> {
    match payload {
        Payload::ImportSection(s) if s.count() > 0 => Err("the module imports something".into()),
        Payload::StartSection { .. } => Err("the module has a start function".into()),
        Payload::MemorySection(s) if s.count() != 1 => {
            Err("the module must have exactly one memory".into())
        }
        Payload::TableSection(s) if s.count() > 1 => {
            Err("the module has more than one table".into())
        }
        Payload::CodeSectionEntry(body) => body_rule(body),
        _ => Ok(()),
    }
}

fn body_rule(body: &wasmparser::FunctionBody<'_>) -> Result<(), String> {
    let reader = body
        .get_operators_reader()
        .map_err(|_| "the module cannot be parsed".to_string())?;
    for op in reader {
        let op = op.map_err(|_| "the module cannot be parsed".to_string())?;
        if banned(&op) {
            return Err("the module uses a table or reference instruction".into());
        }
    }
    Ok(())
}

fn banned(op: &Operator<'_>) -> bool {
    matches!(
        op,
        Operator::TableGet { .. }
            | Operator::TableSet { .. }
            | Operator::TableGrow { .. }
            | Operator::TableSize { .. }
            | Operator::TableFill { .. }
            | Operator::TableCopy { .. }
            | Operator::TableInit { .. }
            | Operator::ElemDrop { .. }
            | Operator::RefNull { .. }
            | Operator::RefIsNull
            | Operator::RefFunc { .. }
    )
}
