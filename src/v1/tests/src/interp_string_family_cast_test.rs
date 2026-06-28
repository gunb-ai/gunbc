use std::rc::Rc;

use v1_compiler::v1_compiler_compile::{compile_to_resolved, ResolvedPipelineResult};
use v1_compiler::v1_interpreter::{self, Value};

use crate::helpers::resolve_imports_transitively;

fn assert_resolved_no_hard_errors(result: &ResolvedPipelineResult) {
    let msgs: Vec<String> = result
        .diagnostics
        .iter()
        .map(|d| v1_compiler::v1_std_core::diagnostic_to_message(d.diagnostic.clone()))
        .filter(|m| !m.starts_with("complexity: "))
        .collect();
    assert!(
        msgs.is_empty() && result.graph.is_some(),
        "expected resolved graph, got diagnostics {:?} (graph present: {})",
        msgs,
        result.graph.is_some()
    );
}

fn run_witness(src: &str, witness_fn: &str) -> Value {
    let sources = resolve_imports_transitively("test/string_family_cast.dag", src);
    let resolved = compile_to_resolved(Rc::new(sources));
    assert_resolved_no_hard_errors(&resolved);
    let graph = resolved
        .graph
        .as_ref()
        .expect("graph after successful resolve");
    v1_interpreter::run(graph, resolved.source_indices.clone(), witness_fn)
        .unwrap_or_else(|e| panic!("run {witness_fn}: {e:?}"))
}

#[test]
fn string_family_alias_casts_are_identity_at_runtime() {
    // Url was a hollow String alias removed by #5818 (§3); Uri is now a struct and
    // does not participate in the string-cast family. Test mirrors string_family_cast_witness_test.dag.
    let src = r#"module test.string_family_cast

import std.types { NonEmptyStr, Secret, SecretValue }

fn string_to_secret(s: String) -> Secret { s as Secret }
fn string_to_nonempty(s: String) -> NonEmptyStr { s as NonEmptyStr }
fn string_to_secret_value(s: String) -> SecretValue { s as SecretValue }

test fn string_family_cast_holds() -> Bool {
  string_to_secret("secret-token") == "secret-token"
    && string_to_nonempty("bmc-host") == "bmc-host"
    && string_to_secret_value("rotated") == "rotated"
}
"#;
    match run_witness(src, "string_family_cast_holds") {
        Value::Bool(true) => {}
        other => panic!("expected true witness, got {other:?}"),
    }
}

#[test]
fn int_to_secret_cast_stays_fail_closed() {
    let src = r#"module test.int_secret_cast

fn bad(n: Int) -> Secret {
  n as Secret
}

test fn int_secret_cast_holds() -> Bool {
  bad(42) == ""
}
"#;
    let sources = resolve_imports_transitively("test/int_secret_cast.dag", src);
    let resolved = compile_to_resolved(Rc::new(sources));
    assert_resolved_no_hard_errors(&resolved);
    let graph = resolved
        .graph
        .as_ref()
        .expect("graph after successful resolve");
    match v1_interpreter::run(
        graph,
        resolved.source_indices.clone(),
        "int_secret_cast_holds",
    ) {
        Err(v1_interpreter::InterpError::TypeError { msg }) => {
            assert!(
                msg.contains("cannot cast Int to Secret"),
                "expected Int→Secret type error, got: {msg}"
            );
        }
        Ok(other) => panic!("expected Int→Secret cast to fail closed, got {other:?}"),
        Err(other) => panic!("expected TypeError for Int→Secret, got {other:?}"),
    }
}
