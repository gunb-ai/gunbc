use std::rc::Rc;

use v1_compiler::v1_compiler_compile::{compile_to_resolved, SourceFile};
use v1_compiler::v1_interpreter::{self, ExecutionMode, InterpContext, Value};

use crate::helpers::{resolve_imports_transitively_with_source_roots, workspace_root};

const WITNESS_ENTRY: &str = "src/v2/test/claim/manual/typescript_program_emit_run_test.dag";
const ADD_FN: &str = "ts_program_emit_add_source";
const CHAR_REALIZATION_FN: &str = "ts_atom_catalog_realizes_char_to_string_holds";

fn v2_source_roots() -> Vec<std::path::PathBuf> {
    crate::helpers::v2_layer_roots()
}

fn witness_sources() -> Vec<Rc<SourceFile>> {
    let entry_content = std::fs::read_to_string(workspace_root().join(WITNESS_ENTRY))
        .unwrap_or_else(|e| panic!("read {WITNESS_ENTRY}: {e}"));
    resolve_imports_transitively_with_source_roots(
        WITNESS_ENTRY,
        &entry_content,
        &v2_source_roots(),
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
                    other => panic!("unexpected tail value {other:?}"),
                }
            }
            out
        }
        other => panic!("not a String FreeMonoid: {other:?}"),
    }
}

fn resolve_witness() -> Rc<v1_compiler::v1_compiler_compile::ResolvedPipelineResult> {
    let resolved = compile_to_resolved(Rc::new(witness_sources().into()));
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
    resolved
}

fn node_strip_types_available() -> bool {
    // `--experimental-strip-types` needs node >= ~22.6. A bare `node --version` is not
    // enough (older node accepts that but rejects the flag), so probe the actual
    // capability and SKIP — rather than false-fail — where it is unavailable.
    let dir = std::env::temp_dir().join(format!("gunbc_ts_probe_{}", std::process::id()));
    if std::fs::create_dir_all(&dir).is_err() {
        return false;
    }
    let probe = dir.join("probe.ts");
    if std::fs::write(&probe, "const x: number = 1; console.log(x);\n").is_err() {
        return false;
    }
    std::process::Command::new("node")
        .arg("--experimental-strip-types")
        .arg(&probe)
        .output()
        .ok()
        .map(|o| o.status.success() && String::from_utf8_lossy(&o.stdout).trim() == "1")
        .unwrap_or(false)
}

/// Green-by-execution (DESIGN §5): the v2 TypeScript emitter produces a typed
/// multi-param function declaration (`add`, with a `+` body) from the `.dag`
/// substrate. Assembled into a module and run under `node --experimental-strip-types`,
/// the emitted TypeScript type-strips and EXECUTES, returning a value computed by the
/// emitted function. This is stronger than the existing translate tests, which only
/// string-compare the emitted declaration and only node-run the effect-shim *call*:
/// here the emitted function *declaration* itself is run. Rust-shaped output (e.g.
/// `fn add(x: i32) -> i32`) fails type-stripping with a SyntaxError -> this witness
/// goes RED, so it discriminates structurally-TypeScript output from anything else.
///
/// NOTE: richer program emit (a second typed fn, records, operators beyond `+`,
/// control flow) is gated on the Track A shared value-expr/`CanonicalOperation`
/// work; today only the `add` typed fn flat-emits to source. See PR notes.
///
/// CI-DORMANCY + DISSOLUTION TRIGGER (DESIGN §6): the execution oracle runs only
/// where node >= ~22.6 (local dev); it HONESTLY SKIPS on the current self-hosted CI
/// runner, whose node rejects `--experimental-strip-types`. CI-gated coverage for
/// this increment is the two floor `.dag` test fns (the emitted-string pin + the
/// Char realization). DISSOLVE this skip — make the oracle CI-gated — when EITHER
/// the CI runner ships node >= 22.6 OR a node-agnostic TS oracle (`tsc --noEmit`)
/// lands in `ci.yml` (the latter is the planned oracle for the value-expr-constructs
/// carrier).
#[test]
fn typescript_emitted_typed_fn_type_strips_and_runs_under_node() {
    if !node_strip_types_available() {
        eprintln!(
            "skipping node-execution oracle: `node --experimental-strip-types` unavailable \
             (needs node >= ~22.6). CI coverage is the floor .dag string/realization tests; \
             dissolve this skip when the CI runner has node >= 22.6 or a `tsc --noEmit` gate lands."
        );
        return;
    }

    let resolved = resolve_witness();
    let graph = resolved.graph.as_ref().expect("resolved graph");
    let ctx = InterpContext::new(graph, resolved.source_indices.clone(), ExecutionMode::Wet);

    // (1) Atom-realization catalog widening: the v2 TS catalog now realizes the std
    // `Char` carrier (-> TS `string`). Run the discriminating witness; it is RED if
    // the Char row is removed (lookup misses -> Rejected -> false).
    let char_value = v1_interpreter::run_in_context(&ctx, CHAR_REALIZATION_FN, true)
        .unwrap_or_else(|e| panic!("run {CHAR_REALIZATION_FN}: {e:?}"));
    assert!(
        matches!(char_value, Value::Bool(true)),
        "TS atom catalog does not realize the Char carrier to string: {char_value:?}"
    );

    // (2) The emitter is the authority for this string; assert its exact shape so a
    // drift in emit is caught here too, then prove it actually runs.
    let add_value = v1_interpreter::run_in_context(&ctx, ADD_FN, true)
        .unwrap_or_else(|e| panic!("run {ADD_FN}: {e:?}"));
    let add_src = decode_freemonoid_string(&add_value, &ctx);
    assert_eq!(
        add_src, "function add(x: number, y: number): number { return x + y; }",
        "emitted typed fn is not the expected TypeScript function declaration"
    );

    // Discriminating: emitted output must be structurally TypeScript (a function
    // declaration with `: number` annotations) and run correctly.
    let program = format!(
        "{add_src}\n\
         console.log(add(2, 3));\n"
    );

    let tmp_dir =
        std::env::temp_dir().join(format!("gunbc_ts_program_emit_{}", std::process::id()));
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
