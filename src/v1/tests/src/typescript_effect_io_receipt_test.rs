//! **Layer:** integration (executable receipt)
//!
//! **Day-goal proof (operator 2026-06-12):** emitted TypeScript that performs REAL file IO.
//!
//! This is the executable receipt for the value-expression effect-apply projection arm
//! (`target_value_expression_effect_apply` → `target_value_expression_to_concrete_tokens`'s
//! `TargetValueExprEffectApply` arm → `serialize_concrete_syntax_tokens_to_source`). It drives
//! the real v2 compile+interpret pipeline over
//! `src/v2/compiler/manual/typescript_effect_io_emit.dag` to obtain the COMPILER-EMITTED
//! TypeScript call expressions for an applied `WriteResource` and an applied `ReadResource`,
//! then assembles a node-runnable program in which the two host-shim callees
//! (`__gunbc_effect_write` / `__gunbc_effect_read`) are bound to `fs.writeFileSync` /
//! `fs.readFileSync`, runs it under `node`, and asserts a real write→read file round-trip on a
//! temp file. The emitted call expressions are used VERBATIM — only the shim bodies, operand
//! `const` bindings, and the temp path (the host workspace) are supplied by this harness, so no
//! per-effect logic is hand-listed in the compiler (anti-cement, gunbc#4623).
//!
//! Validation is BY EXECUTION (node exit 0 + on-disk file content + stdout round-trip), not a
//! structural/lens pass. The manual corpus has no CI gate; run explicitly:
//!   `cargo test -p v1-compiler-tests typescript_effect_io_receipt -- --nocapture`

use std::rc::Rc;

use v1_compiler::v1_compiler_compile::{compile_to_resolved, SourceFile};
use v1_compiler::v1_interpreter::{self, ExecutionMode, InterpContext, Value};

use crate::helpers::{resolve_imports_transitively_with_source_roots, workspace_root};

const WITNESS_ENTRY: &str = "src/v2/compiler/manual/typescript_effect_io_emit.dag";
const WRITE_FN: &str = "ts_effect_write_call_source";
const READ_FN: &str = "ts_effect_read_call_source";

/// The marker bytes written and read back through the emitted shim calls. Unique enough that a
/// stale temp file from a prior run cannot produce a false positive.
const IO_MARKER: &str = "gunbc-effect-io-roundtrip-9f1c2";

fn v2_source_roots() -> Vec<std::path::PathBuf> {
    vec![workspace_root().join("src/v2")]
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

/// Decode a `String`-typed interpreter value. In gunbc, `String = FreeMonoid<Char>`, and
/// `serialize_concrete_syntax_tokens_to_source` builds its result via `Empty`/`Cons`/`list_append`,
/// so the value arrives as a native `Str`, a `Cons`/`Empty` codepoint chain, or a `List` of
/// codepoint `Int`s. Flatten all three to a Rust `String`.
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

/// Run one no-arg witness function through the real compile+interpret pipeline and decode its
/// `String` result.
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

#[test]
fn typescript_effect_io_receipt_emitted_calls_perform_real_file_io() {
    let write_call = emitted_source(WRITE_FN);
    let read_call = emitted_source(READ_FN);

    // The emitted text must be the compiler's projection of the effect application, not a
    // degraded sentinel. (The .dag witness returns "EFFECT_EMIT_REJECTED" on a Rejected emit.)
    assert_eq!(
        write_call, "__gunbc_effect_write(ioPath, ioContent)",
        "emitted WriteResource application is not the expected host-shim call"
    );
    assert_eq!(
        read_call, "__gunbc_effect_read(ioPath)",
        "emitted ReadResource application is not the expected host-shim call"
    );

    // Host workspace: a temp data file, the two fs-backed shims, and the operand `const`
    // bindings. Everything compiler-owned (the two call expressions) is interpolated VERBATIM.
    let tmp_dir = std::env::temp_dir().join(format!("gunbc_effect_io_{}", std::process::id()));
    std::fs::create_dir_all(&tmp_dir).expect("create temp dir");
    let data_path = tmp_dir.join("effect_io_target.txt");
    let program_path = tmp_dir.join("effect_io_program.js");

    let program = format!(
        "const fs = require('fs');\n\
         const __gunbc_effect_write = (p, c) => {{ fs.writeFileSync(p, c); return c; }};\n\
         const __gunbc_effect_read = (p) => fs.readFileSync(p, 'utf8');\n\
         const ioPath = {path};\n\
         const ioContent = {content};\n\
         {write_call};\n\
         const __gunbc_roundtrip = {read_call};\n\
         process.stdout.write(__gunbc_roundtrip);\n",
        path = serde_json::to_string(&data_path.to_string_lossy().to_string()).unwrap(),
        content = serde_json::to_string(IO_MARKER).unwrap(),
        write_call = write_call,
        read_call = read_call,
    );
    std::fs::write(&program_path, &program).expect("write node program");

    let output = std::process::Command::new("node")
        .arg(&program_path)
        .output()
        .expect("spawn node (is node on PATH?)");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "emitted TypeScript/JS effect program failed under node.\nprogram:\n{program}\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );

    // Proof 1 — real WRITE happened: the emitted __gunbc_effect_write call hit the filesystem.
    let on_disk = std::fs::read_to_string(&data_path)
        .unwrap_or_else(|e| panic!("emitted write did not create {data_path:?}: {e}"));
    assert_eq!(
        on_disk, IO_MARKER,
        "emitted WriteResource call did not write the marker to disk"
    );

    // Proof 2 — real READ round-trip: the emitted __gunbc_effect_read call returned the bytes.
    assert_eq!(
        stdout, IO_MARKER,
        "emitted ReadResource call did not read the marker back (stdout round-trip)"
    );

    let _ = std::fs::remove_dir_all(&tmp_dir);
}
