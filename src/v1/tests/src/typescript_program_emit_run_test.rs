use std::rc::Rc;

use v1_compiler::v1_compiler_compile::{compile_to_resolved, SourceFile};
use v1_compiler::v1_interpreter::{self, ExecutionMode, InterpContext, Value};

use crate::helpers::{resolve_imports_transitively_with_source_roots, workspace_root};

const WITNESS_ENTRY: &str = "src/v2/compiler/manual/typescript_program_emit_run_test.dag";
const ADD_FN: &str = "ts_program_emit_add_source";
const IDENTITY_FN: &str = "ts_program_emit_identity_source";

fn v2_source_roots() -> Vec<std::path::PathBuf> {
    crate::helpers::v2_layer_roots()
}

fn witness_sources() -> Vec<Rc<SourceFile>> {
    let entry_content = std::fs::read_to_string(workspace_root().join(WITNESS_ENTRY))
        .unwrap_or_else(|e| panic!("read {WITNESS_ENTRY}: {e}"));
    resolve_imports_transitively_with_source_roots(WITNESS_ENTRY, &entry_content, &v2_source_roots())
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
                        panic!("unexpected FreeMonoid variant {}", ctx.resolve(variant_name));
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

fn emitted_source(function: &str) -> String {
    let resolved = compile_to_resolved(Rc::new(witness_sources()));
    let blocking: Vec<String> = resolved
        .diagnostics
        .iter()
        .map(|d| v1_compiler::v1_std_core::diagnostic_to_message(d.diagnostic.clone()))
        .filter(|m| !m.starts_with("complexity: "))
        .collect();
    assert!(
        blocking.is_empty() && resolved.graph.is_some(),
        "expected clean resolved graph for {WITNESS_ENTRY}, got diagnostics {blocking:?}"
    );
    let graph = resolved.graph.as_ref().expect("resolved graph");
    let ctx = InterpContext::new(graph, resolved.source_indices.clone(), ExecutionMode::Wet);
    let value = v1_interpreter::run_in_context(&ctx, function, true)
        .unwrap_or_else(|e| panic!("run {function}: {e:?}"));
    decode_freemonoid_string(&value, &ctx)
}

fn node_strip_types_available() -> bool {
    // node >= 22 ships `--experimental-strip-types`; bail (don't false-fail) if the
    // runner has no compatible `node` on PATH.
    std::process::Command::new("node")
        .arg("--version")
        .output()
        .ok()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Green-by-execution (DESIGN §5): the v2 TypeScript emitter produces two distinct
/// typed function declarations (`add`, a typed multi-param fn with a `+` body, and
/// `identity`, a typed return-only fn) from the `.dag` substrate. Assembled into one
/// module and run under `node --experimental-strip-types`, the emitted TypeScript
/// type-strips and executes, composing the two emitted functions to produce a
/// computed value. This is "beyond the add slice": the emitted output is proven by
/// real execution (not string equality) and exercises more than the single
/// effect-shim call the existing receipt test runs. Rust-shaped output (e.g.
/// `fn add(x: i32) -> i32`) fails type-stripping with a SyntaxError -> this witness
/// goes RED, so it discriminates structurally-TypeScript output from anything else.
#[test]
fn typescript_emitted_typed_fns_type_strip_and_run_under_node() {
    if !node_strip_types_available() {
        eprintln!("skipping: no `node` on PATH for the strip-types execution oracle");
        return;
    }

    let add_src = emitted_source(ADD_FN);
    let identity_src = emitted_source(IDENTITY_FN);

    // The emitter is the authority for these strings; assert their exact shape so a
    // drift in emit is caught here too, then prove they actually run.
    assert_eq!(
        add_src, "function add(x: number, y: number): number { return x + y; }",
        "emitted typed fn is not the expected TypeScript function declaration"
    );
    assert_eq!(
        identity_src, "function identity(x: number): number { return x; }",
        "emitted identity fn is not the expected TypeScript function declaration"
    );

    // Discriminating: emitted output must be structurally TypeScript (function
    // declarations with `: number` annotations). Compose BOTH emitted functions in
    // runnable positions so a malformed emit of EITHER fails type-stripping.
    let program = format!(
        "{add_src}\n\
         {identity_src}\n\
         console.log(add(identity(2), 3));\n"
    );

    let tmp_dir = std::env::temp_dir().join(format!("gunbc_ts_program_emit_{}", std::process::id()));
    std::fs::create_dir_all(&tmp_dir).expect("create temp dir");
    let program_path = tmp_dir.join("ts_program_emit.ts");
    std::fs::write(&program_path, &program).expect("write emitted ts program");

    let output = std::process::Command::new("node")
        .arg("--experimental-strip-types")
        .arg(&program_path)
        .output()
        .expect("spawn node");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "node failed to type-strip+run the emitted TypeScript program.\n--- program ---\n{program}\n--- stderr ---\n{stderr}"
    );
    assert_eq!(
        stdout.trim(),
        "5",
        "emitted TypeScript ran but produced the wrong result.\n--- program ---\n{program}\n--- stdout ---\n{stdout}\n--- stderr ---\n{stderr}"
    );
}
