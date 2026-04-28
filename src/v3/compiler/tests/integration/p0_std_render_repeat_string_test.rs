//! **Layer:** integration
//!
//! P0-A: `dsl/std/render.dag` `repeat_string` / `indent_text` semantics via the
//! v2 resolved graph + interpreter (behavior oracle, not emitted-source grep).

use std::path::{Path, PathBuf};
use std::rc::Rc;

use v2_compiler::v2_compiler_compile::{compile_to_resolved, ResolvedPipelineResult};
use v2_compiler::v2_interpreter::{self, Value};
use v2_compiler_tests::helpers::resolve_imports_transitively;
use v3_compiler::test_runner::{ClaimResult, TestRunner};
use v3_compiler::{compile_to_dag, CompileError};

fn repo_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .expect("workspace root (dsl/ should exist)")
        .to_path_buf()
}

fn assert_resolved_no_hard_errors(result: &ResolvedPipelineResult) {
    let msgs: Vec<String> = result
        .diagnostics
        .iter()
        .map(|d| v2_compiler::v2_std_core::diagnostic_to_message(d.diagnostic.clone()))
        .filter(|m| !m.starts_with("complexity: "))
        .collect();
    assert!(
        msgs.is_empty() && result.graph.is_some(),
        "expected resolved graph, got diagnostics {:?} (graph present: {})",
        msgs,
        result.graph.is_some()
    );
}

#[test]
fn std_render_repeat_string_and_indent_text_match_interpreter() {
    let root = repo_root();
    assert!(
        root.join("dsl/std/render.dag").is_file(),
        "expected dsl/std/render.dag at {}",
        root.display()
    );

    let src = r#"module test.repeat_string_regression
import std.render { repeat_string }
fn repeat_string_returns_n_copies() -> String { repeat_string(s: "x", n: 3) }
fn pads_then_text_like_indent() -> String {
  let pad = repeat_string(s: " ", n: 2)
  concat(pad, "a")
}
"#;
    let sources = resolve_imports_transitively("test.dag", src);
    let resolved = compile_to_resolved(Rc::new(sources));
    assert_resolved_no_hard_errors(&resolved);
    let graph = resolved
        .graph
        .as_ref()
        .expect("graph after successful resolve");

    match v2_interpreter::run(
        graph,
        resolved.source_indices.clone(),
        "repeat_string_returns_n_copies",
    ) {
        Ok(Value::Str(s)) => assert_eq!(s, "xxx"),
        other => panic!("expected Str(\"xxx\"), got {other:?}"),
    }

    match v2_interpreter::run(
        graph,
        resolved.source_indices.clone(),
        "pads_then_text_like_indent",
    ) {
        Ok(Value::Str(s)) => assert_eq!(s, "  a"),
        other => panic!("expected Str(\"  a\"), got {other:?}"),
    }
}

/// R1C-B / T-P0 — `p0_repeat_string_correct` must pass through the v3 `TestRunner` on the same
/// oracle string asserted above for `repeat_string(s: "x", n: 3)` (ROADMAP T-P0 Day-1 gate).
#[test]
fn p0_repeat_string_correct_gate_matches_v2_render_oracle() {
    let root = repo_root();
    assert!(
        root.join("dsl/std/render.dag").is_file(),
        "expected dsl/std/render.dag at {}",
        root.display()
    );

    let src = r#"module test.repeat_string_regression
import std.render { repeat_string }
fn repeat_string_returns_n_copies() -> String { repeat_string(s: "x", n: 3) }
"#;
    let sources = resolve_imports_transitively("test.dag", src);
    let resolved = compile_to_resolved(Rc::new(sources));
    assert_resolved_no_hard_errors(&resolved);
    let graph = resolved
        .graph
        .as_ref()
        .expect("graph after successful resolve");
    match v2_interpreter::run(
        graph,
        resolved.source_indices.clone(),
        "repeat_string_returns_n_copies",
    ) {
        Ok(Value::Str(s)) => assert_eq!(s, "xxx"),
        other => panic!("expected Str(\"xxx\"), got {other:?}"),
    }

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let gate_path = manifest_dir.join("tests/fixtures/r1_gates.dag");
    let gate_source = std::fs::read_to_string(&gate_path)
        .unwrap_or_else(|e| panic!("read {}: {e}", gate_path.display()));
    let dag = match compile_to_dag(&gate_source, "src/v3/compiler/tests/fixtures/r1_gates.dag") {
        Ok(d) => d,
        Err(CompileError::Semantic(err_dag)) => panic!(
            "r1_gates.dag semantic errors: {:?}",
            err_dag.diagnostics().iter().collect::<Vec<_>>()
        ),
        Err(e) => panic!("r1_gates.dag: {e:?}"),
    };
    let results = TestRunner::new(&dag).run_suite("p0_repeat_string_correct_gate");
    assert_eq!(results.len(), 1, "{results:?}");
    assert_eq!(
        results[0].result,
        ClaimResult::Pass,
        "expected Pass on p0_repeat_string_correct, got {:?}",
        results[0]
    );
}
