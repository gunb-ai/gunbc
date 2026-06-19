//! Regression: v1 interpreter must treat String→String-family alias casts as identity.
//!
//! Targets whose underlying base resolves to String (nominal_opaque / where / brand)
//! share the Str runtime carrier. eval_cast walks each target's alias chain to decide
//! identity vs fail-closed, mirroring emit's shared-checkpoint elision.

use std::rc::Rc;

use v1_compiler::v1_compiler_compile::{compile_to_resolved, ResolvedPipelineResult};
use v1_compiler::v1_compiler_infer_env::lookup_type_by_name;
use v1_compiler::v1_compiler_infer_resolve::resolve_node_bounded;
use v1_compiler::v1_interpreter::{self, Value};
use v1_compiler::v1_std_core::{authored_name_at, InferredNode};

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
fn debug_nonempty_type_binding_shape() {
    let src = r#"module test.string_family_cast
import std.types { NonEmptyStr }
fn string_to_nonempty(s: String) -> NonEmptyStr { s as NonEmptyStr }
"#;
    let sources = resolve_imports_transitively("test/string_family_cast.dag", src);
    let resolved = compile_to_resolved(Rc::new(sources));
    assert_resolved_no_hard_errors(&resolved);
    let graph = resolved.graph.as_ref().unwrap();
    for module in graph.modules.iter() {
        let module_name = authored_name_at(resolved.source_indices.clone(), module.module.clone());
        if module_name != "std.types" {
            continue;
        }
        let decl = lookup_type_by_name(module.type_env.clone(), "NonEmptyStr".to_string())
            .expect("NonEmptyStr decl");
        let has_inferred = decl.inferred.is_some();
        let decl_name = authored_name_at(resolved.source_indices.clone(), decl.clone());
        let structural = if let Some(InferredNode::Resolved { node }) = decl.inferred.as_deref() {
            resolve_node_bounded(node.clone(), module.type_env.clone(), module_name.clone(), 0)
                .resolved
                .clone()
        } else {
            resolve_node_bounded(decl.clone(), module.type_env.clone(), module_name.clone(), 0)
                .resolved
                .clone()
        };
        let structural_name =
            authored_name_at(resolved.source_indices.clone(), structural.clone());
        panic!(
            "NonEmptyStr decl_name={decl_name} has_inferred={has_inferred} structural_name={structural_name}"
        );
    }
    panic!("std.types module not found");
}

#[test]
fn string_family_alias_casts_are_identity_at_runtime() {
    let src = r#"module test.string_family_cast

import std.types { NonEmptyStr, Secret, SecretValue, Url }

fn string_to_secret(s: String) -> Secret { s as Secret }
fn string_to_nonempty(s: String) -> NonEmptyStr { s as NonEmptyStr }
fn string_to_secret_value(s: String) -> SecretValue { s as SecretValue }
fn string_to_url(s: String) -> Url { s as Url }

test fn string_family_cast_holds() -> Bool {
  string_to_secret("secret-token") == "secret-token"
    && string_to_nonempty("bmc-host") == "bmc-host"
    && string_to_secret_value("rotated") == "rotated"
    && string_to_url("https://example.com") == "https://example.com"
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
