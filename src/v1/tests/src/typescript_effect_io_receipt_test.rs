use std::rc::Rc;

use v1_compiler::v1_compiler_compile::{compile_to_resolved, SourceFile};
use v1_compiler::v1_interpreter::{self, ExecutionMode, InterpContext, Value};

use crate::helpers::{resolve_imports_transitively_with_source_roots, workspace_root};

const WITNESS_ENTRY: &str = "src/v2/test/claim/manual/typescript_effect_io_emit_test.dag";
const WRITE_FN: &str = "ts_effect_write_call_source";
const READ_FN: &str = "ts_effect_read_call_source";

const IO_MARKER: &str = "gunbc-effect-io-roundtrip-9f1c2";

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

fn emitted_source(function: &str) -> String {
    let resolved = compile_to_resolved(Rc::new(witness_sources().into()));
    let blocking: Vec<String> = resolved
        .diagnostics
        .iter()
        .map(|d| v1_compiler::v1_std_core::diagnostic_to_message(d.diagnostic.clone()))
        .filter(|m| !m.starts_with("complexity: ") && !m.starts_with("unlisted import use "))
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

    assert_eq!(
        write_call, "__gunbc_effect_write(ioPath, ioContent)",
        "emitted WriteResource application is not the expected host-shim call"
    );
    assert_eq!(
        read_call, "__gunbc_effect_read(ioPath)",
        "emitted ReadResource application is not the expected host-shim call"
    );

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

    let on_disk = std::fs::read_to_string(&data_path)
        .unwrap_or_else(|e| panic!("emitted write did not create {data_path:?}: {e}"));
    assert_eq!(
        on_disk, IO_MARKER,
        "emitted WriteResource call did not write the marker to disk"
    );

    assert_eq!(
        stdout, IO_MARKER,
        "emitted ReadResource call did not read the marker back (stdout round-trip)"
    );

    let _ = std::fs::remove_dir_all(&tmp_dir);
}
