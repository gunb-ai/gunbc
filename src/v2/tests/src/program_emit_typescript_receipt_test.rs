//! **Layer:** integration (executable receipt)
//!
//! M1 §4 — emitted TypeScript for the locked `program.dag` pick_if Branch (effect_io then-arm)
//! compiles under tsc, runs on Node, and performs a real write→read file round-trip.

use std::path::Path;

use emit_host_runner::{
    run_host_process, EmitHostTransportInputs, TS_HOST_TRANSPORT_PROGRAM_IDENTITY,
};
use v2_compiler::v2_compiler_compile::{compile_to_resolved, SourceFile};
use v2_compiler::v2_interpreter::{self, InterpContext, Value};

use crate::helpers::{resolve_imports_transitively_with_source_roots, workspace_root};

const WITNESS_ENTRY: &str = "src/v4/test/claim/manual/program_emit_typescript.dag";
const EMIT_FN: &str = "program_emit_typescript_emitted_source";
const IO_MARKER: &str = "gunbc-program-effect-io-roundtrip";

fn v4_source_roots() -> Vec<std::path::PathBuf> {
    vec![workspace_root().join("src/v4")]
}

fn witness_sources() -> Vec<std::rc::Rc<SourceFile>> {
    let entry_content = std::fs::read_to_string(workspace_root().join(WITNESS_ENTRY))
        .unwrap_or_else(|e| panic!("read {WITNESS_ENTRY}: {e}"));
    resolve_imports_transitively_with_source_roots(
        WITNESS_ENTRY,
        &entry_content,
        &v4_source_roots(),
    )
}

fn decode_freemonoid_string(val: &Value, ctx: &InterpContext) -> String {
    fn codepoint(v: &Value) -> char {
        match v {
            Value::Int(n) => char::from_u32(*n as u32)
                .unwrap_or_else(|| panic!("codepoint {n} is not a valid char")),
            other => panic!("expected Int codepoint in String FreeMonoid, got {other:?}"),
        }
    }
    match val {
        Value::Str(s) => s.clone(),
        Value::List(items) => items.iter().map(codepoint).collect(),
        Value::Variant {
            variant_name,
            fields,
            ..
        } => {
            let mut out = String::new();
            let mut cur = val.clone();
            loop {
                match cur {
                    Value::Variant {
                        variant_name,
                        fields,
                        ..
                    } => {
                        if ctx.sym_eq(variant_name, "Empty") {
                            break;
                        }
                        if ctx.sym_eq(variant_name, "Cons") {
                            let head = ctx.field(&fields, "head").expect("Cons.head");
                            out.push(codepoint(head));
                            cur = ctx.field(&fields, "tail").expect("Cons.tail").clone();
                            continue;
                        }
                        panic!(
                            "unexpected FreeMonoid variant {}",
                            ctx.resolve(variant_name)
                        );
                    }
                    Value::Str(s) => {
                        out.push_str(&s);
                        break;
                    }
                    other => panic!("unexpected tail value {other:?}"),
                }
            }
            out
        }
        other => panic!("not a String FreeMonoid: {other:?}"),
    }
}

fn emitted_program_source() -> String {
    let resolved = compile_to_resolved(std::rc::Rc::new(witness_sources()));
    let blocking: Vec<String> = resolved
        .diagnostics
        .iter()
        .map(|d| v2_compiler::v2_std_core::diagnostic_to_message(d.diagnostic.clone()))
        .filter(|m| !m.starts_with("complexity: "))
        .collect();
    assert!(
        blocking.is_empty() && resolved.graph.is_some(),
        "expected clean resolved graph for {WITNESS_ENTRY}, got diagnostics {blocking:?}"
    );
    let graph = resolved.graph.as_ref().expect("resolved graph");
    let ctx = InterpContext::new(graph, resolved.source_indices.clone(), false);
    let value = v2_interpreter::run_in_context(&ctx, EMIT_FN, true)
        .unwrap_or_else(|e| panic!("run {EMIT_FN}: {e:?}"));
    decode_freemonoid_string(&value, &ctx)
}

#[test]
fn program_emit_typescript_receipt_runs_on_node_with_effect_io_roundtrip() {
    let source = emitted_program_source();
    assert!(
        !source.contains("PROGRAM_EMIT_REJECTED"),
        "program emit rejected: {source}"
    );
    assert!(
        source.contains("__gunbc_effect_write"),
        "emitted source missing WriteResource shim call: {source}"
    );
    assert!(
        source.contains("__gunbc_effect_read"),
        "emitted source missing ReadResource shim call: {source}"
    );

    let work_dir = std::env::temp_dir().join(format!("gunbc_program_emit_{}", std::process::id()));
    std::fs::create_dir_all(&work_dir).expect("create temp dir");
    let data_path = work_dir.join("gunbc_program_io_target.txt");

    let receipt = run_host_process(
        TS_HOST_TRANSPORT_PROGRAM_IDENTITY,
        &source,
        &EmitHostTransportInputs {
            claim_input_root: "program_emit_typescript_claim_input".to_string(),
        },
        Path::new(&work_dir),
    )
    .unwrap_or_else(|e| panic!("run_host_process failed: {e:?}"));

    assert!(
        receipt.exit.exit_holds(),
        "program emit host run failed: exit={:?} stderr={}",
        receipt.exit,
        String::from_utf8_lossy(&receipt.stderr_bytes)
    );

    let stdout = String::from_utf8_lossy(&receipt.stdout_bytes);
    assert_eq!(
        stdout, IO_MARKER,
        "emitted program did not read the marker back (stdout round-trip)"
    );

    let on_disk = std::fs::read_to_string(&data_path)
        .unwrap_or_else(|e| panic!("emitted write did not create {data_path:?}: {e}"));
    assert_eq!(
        on_disk, IO_MARKER,
        "emitted WriteResource call did not write the marker to disk"
    );

    let _ = std::fs::remove_dir_all(&work_dir);
}
