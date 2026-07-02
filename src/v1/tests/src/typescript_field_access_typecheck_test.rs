//! TypeScript emit type-oracle: the v2-emitted source must pass `tsc --noEmit`.
//!
//! This is the node-AGNOSTIC TypeScript oracle for the value-expr-constructs
//! carrier (roadmap 5-ts-first-class, increment 1). Unlike a `node
//! --experimental-strip-types` run (which only strips types and needs node
//! 22.6 or newer), `tsc --noEmit` actually TYPE-CHECKS the emitted source, so
//! it is a stronger oracle and runs on the CI runner that already provisions
//! `npx`/`node` for the emit-host smoke gate (`dag/tools/emit_host_gate.dag`,
//! which runs `npx -y -p typescript@5.9.2 tsc ...` in the CI floor job).
//!
//! It covers BOTH the typed `add` fn (this dissolves #5695's `node
//! --experimental-strip-types` skip — the same emitted declaration is now
//! type-checked unconditionally on the CI runner via tsc) and the newly-wired
//! FieldAccess construct (`o.x`).
//!
//! Discriminating control (`tsc_noemit_rejects_rust_shaped_member_access`):
//! a Rust/C++-shaped `o->x` is fed through the same tsc invocation and MUST be
//! rejected — proving tsc is genuinely checking, not rubber-stamping.
//!
//! NON-DORMANCY: this test prints `[tsc-oracle] running` when it executes and
//! `[tsc-oracle] SKIP` (with a named trigger) only when `npx` is genuinely
//! absent. The CI floor proves `npx` is present on the self-hosted runner, so
//! this is expected to RUN (not skip) in CI; the markers let the CI log
//! confirm that. If a runner ever lacks `npx`, the skip is honest and names
//! its dissolution trigger rather than silently passing.

use std::rc::Rc;

use v1_compiler::v1_compiler_compile::{compile_to_resolved, SourceFile};
use v1_compiler::v1_interpreter::{self, ExecutionMode, InterpContext, Value};

use crate::helpers::{resolve_imports_transitively_with_source_roots, workspace_root};

const WITNESS_ENTRY: &str = "src/v2/test/claim/manual/typescript_field_access_emit_test.dag";
const ADD_FN: &str = "ts_add_emit_source";
const FIELD_ACCESS_FN: &str = "ts_field_access_emit_source";
const RECORD_WITNESS_ENTRY: &str =
    "src/v2/test/claim/manual/typescript_record_construct_emit_test.dag";
const RECORD_CONSTRUCT_FN: &str = "ts_record_construct_emit_source";
const RECORD_CONSTRUCT_PREPEND_FN: &str = "ts_record_construct_emit_source_prepend";

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

/// Resolve the witness once and emit both source fragments we need, so the
/// (expensive) resolve happens a single time per test.
fn emit_add_and_field_access() -> (String, String) {
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
    let emit = |function: &str| -> String {
        let value = v1_interpreter::run_in_context(&ctx, function, true)
            .unwrap_or_else(|e| panic!("run {function}: {e:?}"));
        decode_freemonoid_string(&value, &ctx)
    };
    (emit(ADD_FN), emit(FIELD_ACCESS_FN))
}

/// Resolve a witness `entry` once and emit the FreeMonoid string produced by
/// `function`. Used for the RecordConstruct witness, which lives in its own
/// `_test.dag` file.
fn emit_one(entry: &str, function: &str) -> String {
    let entry_content = std::fs::read_to_string(workspace_root().join(entry))
        .unwrap_or_else(|e| panic!("read {entry}: {e}"));
    let sources =
        resolve_imports_transitively_with_source_roots(entry, &entry_content, &v2_source_roots());
    let resolved = compile_to_resolved(Rc::new(sources));
    let blocking: Vec<String> = resolved
        .diagnostics
        .iter()
        .map(|d| v1_compiler::v1_std_core::diagnostic_to_message(d.diagnostic.clone()))
        .filter(|m| !m.starts_with("complexity: "))
        .collect();
    assert!(
        blocking.is_empty() && resolved.graph.is_some(),
        "expected clean resolved graph for {entry}, got diagnostics {blocking:?}"
    );
    let graph = resolved.graph.as_ref().expect("resolved graph");
    let ctx = InterpContext::new(graph, resolved.source_indices.clone(), ExecutionMode::Wet);
    let value = v1_interpreter::run_in_context(&ctx, function, true)
        .unwrap_or_else(|e| panic!("run {function}: {e:?}"));
    decode_freemonoid_string(&value, &ctx)
}

/// True iff `npx` is on PATH. The CI floor's emit-host gate proves `npx` is
/// present on the self-hosted runner, so in CI this returns true and the
/// oracle RUNS (it does not skip).
fn npx_available() -> bool {
    std::process::Command::new("npx")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Run `tsc --noEmit` over `source` written as `module.ts` in a fresh temp dir.
/// Returns Ok(()) on a clean type-check, Err(diagnostics) otherwise.
fn tsc_noemit(label: &str, source: &str) -> Result<(), String> {
    let dir =
        std::env::temp_dir().join(format!("gunbc_tsc_oracle_{}_{}", label, std::process::id()));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    let file = dir.join("module.ts");
    std::fs::write(&file, source).expect("write module.ts");
    let output = std::process::Command::new("npx")
        .args([
            "-y",
            "-p",
            "typescript@5.9.2",
            "tsc",
            "--noEmit",
            "--strict",
            "--target",
            "ES2022",
        ])
        .arg(&file)
        .output()
        .expect("spawn npx tsc");
    let _ = std::fs::remove_dir_all(&dir);
    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "stdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

#[test]
fn emitted_typescript_typechecks_under_tsc_noemit() {
    if !npx_available() {
        // Honest skip — names its dissolution trigger. Expected to RUN in CI
        // (the emit-host gate proves npx on the self-hosted runner).
        eprintln!(
            "[tsc-oracle] SKIP: npx not on PATH. DISSOLVE this skip when the runner provisions npx \
             (it already does for dag/tools/emit_host_gate.dag)."
        );
        return;
    }
    eprintln!("[tsc-oracle] running emitted_typescript_typechecks_under_tsc_noemit");

    let (add_src, field_access) = emit_add_and_field_access();
    assert_eq!(
        add_src, "function add(x: number, y: number): number { return x + y; }",
        "emitted add fn declaration changed"
    );
    assert_eq!(field_access, "o.x", "emitted field access changed");

    // A type-checkable module (NO @ts-nocheck) that exercises both the emitted
    // typed fn and the emitted field access. `o.x : number` flows into `add`.
    let module = format!(
        "{add_src}\n\
         const o: {{ x: number }} = {{ x: 41 }};\n\
         const __fa: number = {field_access};\n\
         const __r: number = add(__fa, 1);\n\
         export {{ __r }};\n"
    );
    tsc_noemit("ok", &module)
        .unwrap_or_else(|d| panic!("emitted TypeScript failed tsc --noEmit:\n{module}\n{d}"));
}

#[test]
fn emitted_typescript_record_construct_typechecks_under_tsc_noemit() {
    if !npx_available() {
        eprintln!(
            "[tsc-oracle] SKIP: npx not on PATH. DISSOLVE this skip when the runner provisions npx \
             (it already does for dag/tools/emit_host_gate.dag)."
        );
        return;
    }
    eprintln!(
        "[tsc-oracle] running emitted_typescript_record_construct_typechecks_under_tsc_noemit"
    );

    // AXIS 1 row-value perturbation, derived from the SAME fold: the TS target's
    // `record_literal_names_type: Omit` row makes RecordConstruct emit an ANONYMOUS
    // object literal (valid TS); swapping ONLY the row value to `Prepend` (correct
    // for Rust/DAG) prepends the nominal type name. The emitted text must FLIP — if
    // it did not, the parameterization would be cosmetic.
    let record = emit_one(RECORD_WITNESS_ENTRY, RECORD_CONSTRUCT_FN);
    let record_prepend = emit_one(RECORD_WITNESS_ENTRY, RECORD_CONSTRUCT_PREPEND_FN);
    assert_eq!(
        record.trim(),
        "{ x: a, y: b }",
        "emitted Omit record changed"
    );
    assert_eq!(
        record_prepend.trim(),
        "Point { x: a, y: b }",
        "emitted Prepend record changed"
    );
    assert_ne!(
        record.trim(),
        record_prepend.trim(),
        "Omit/Prepend row perturbation did not change the emitted text — parameterization is cosmetic"
    );

    let module = format!(
        "const a: number = 1;\n\
         const b: number = 2;\n\
         const o ={record};\n\
         const __x: number = o.x;\n\
         export {{ __x }};\n"
    );
    tsc_noemit("record_ok", &module).unwrap_or_else(|d| {
        panic!("emitted TypeScript record construct failed tsc --noEmit:\n{module}\n{d}")
    });
}

#[test]
fn tsc_noemit_rejects_fold_emitted_prepend_record() {
    if !npx_available() {
        eprintln!(
            "[tsc-oracle] SKIP: npx not on PATH. DISSOLVE this skip when the runner provisions npx \
             (it already does for dag/tools/emit_host_gate.dag)."
        );
        return;
    }
    eprintln!("[tsc-oracle] running tsc_noemit_rejects_fold_emitted_prepend_record");

    // Discriminating control for AXIS 1, FOLD-DERIVED (not hand-typed): feed the
    // SAME fold's `Prepend`-row output (`Point { x: a, y: b }`) through tsc. It is
    // NOT a valid TS object literal and MUST be rejected — proving the `Omit` row is
    // load-bearing and the accept above is genuine type-checking, not rubber-stamping.
    let record_prepend = emit_one(RECORD_WITNESS_ENTRY, RECORD_CONSTRUCT_PREPEND_FN);
    let bad_module = format!(
        "const a: number = 1;\n\
         const b: number = 2;\n\
         const o ={record_prepend};\n\
         export {{ o }};\n"
    );
    assert!(
        tsc_noemit("record_perturb", &bad_module).is_err(),
        "tsc --noEmit accepted the fold-emitted Prepend record ({record_prepend}); \
         the Omit policy / oracle is not discriminating"
    );
}

#[test]
fn tsc_noemit_rejects_rust_shaped_member_access() {
    if !npx_available() {
        eprintln!(
            "[tsc-oracle] SKIP: npx not on PATH. DISSOLVE this skip when the runner provisions npx \
             (it already does for dag/tools/emit_host_gate.dag)."
        );
        return;
    }
    eprintln!("[tsc-oracle] running tsc_noemit_rejects_rust_shaped_member_access");

    // Discriminating control: a Rust/C++-shaped member access (`o->x`) is NOT
    // valid TypeScript. If tsc accepted this, the oracle above would be
    // rubber-stamping rather than type-checking.
    let bad_module = "const o: { x: number } = { x: 41 };\n\
         const __r: number = o->x;\n\
         export { __r };\n";
    assert!(
        tsc_noemit("perturb", bad_module).is_err(),
        "tsc --noEmit accepted a Rust-shaped member access (o->x); the oracle is not discriminating"
    );
}
