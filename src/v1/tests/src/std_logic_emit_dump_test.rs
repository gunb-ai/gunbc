//! One-off diagnostic: dump whole-program emit for std/logic fixture.
use std::rc::Rc;

use v1_compiler::v1_compiler_compile::compile_to_resolved;
use v1_compiler::v1_interpreter::{self, ExecutionMode, InterpContext, Value};

use crate::helpers::{resolve_imports_transitively_with_source_roots, workspace_root};

const FIXTURE: &str = "src/v2/test/fixture/std_logic_module.dag";

fn v2_source_roots() -> Vec<std::path::PathBuf> {
    crate::helpers::v2_layer_roots()
}

fn decode_freemonoid_string(val: &Value, ctx: &InterpContext) -> String {
    fn codepoint(v: &Value) -> char {
        match v {
            Value::Int(n) => char::from_u32(*n as u32)
                .unwrap_or_else(|| panic!("codepoint {n} is not a valid char")),
            other => panic!("expected Int codepoint, got {other:?}"),
        }
    }
    match val {
        Value::Str(s) => s.clone(),
        Value::List(items) => items.iter().map(codepoint).collect(),
        Value::Variant { .. } => {
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
                    other => panic!("unexpected tail {other:?}"),
                }
            }
            out
        }
        other => panic!("not a String FreeMonoid: {other:?}"),
    }
}

fn resolve_fixture() -> InterpContext {
    let entry_content = std::fs::read_to_string(workspace_root().join(FIXTURE))
        .unwrap_or_else(|e| panic!("read {FIXTURE}: {e}"));
    let sources = resolve_imports_transitively_with_source_roots(
        FIXTURE,
        &entry_content,
        &v2_source_roots(),
    );
    let resolved = compile_to_resolved(Rc::new(sources.into()));
    let blocking: Vec<String> = resolved
        .diagnostics
        .iter()
        .map(|d| v1_compiler::v1_std_core::diagnostic_to_message(d.diagnostic.clone()))
        .filter(|m| !m.starts_with("complexity: ") && !m.starts_with("unlisted import use "))
        .collect();
    assert!(
        blocking.is_empty() && resolved.graph.is_some(),
        "resolve failed: {blocking:?}"
    );
    let graph = resolved.graph.as_ref().expect("graph");
    let ctx = InterpContext::new(graph, resolved.source_indices.clone(), ExecutionMode::Wet);
    ctx
}

#[test]
fn dump_std_logic_produced_emit() {
    let ctx = resolve_fixture();
    let emit_text = {
        let value = v1_interpreter::run_in_context(&ctx, "std_logic_module_emit_text", true)
            .unwrap_or_else(|e| panic!("run emit_text: {e:?}"));
        decode_freemonoid_string(&value, &ctx)
    };
    let emit_len = {
        let value = v1_interpreter::run_in_context(&ctx, "std_logic_module_emit_len", true)
            .unwrap_or_else(|e| panic!("run emit_len: {e:?}"));
        match value {
            Value::Int(n) => n,
            other => panic!("expected Int len, got {other}"),
        }
    };
    eprintln!("=== std_logic produced emit (len={emit_len}) ===");
    eprintln!("{emit_text}");
    eprintln!("=== end ===");
}
