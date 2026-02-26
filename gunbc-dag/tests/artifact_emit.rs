//! FC-P7-b: Compiler artifact emitter round-trip tests.
//!
//! Verifies that `emit_artifact_dag()` produces valid `.dag` syntax that
//! can be re-parsed by the DAG compiler.

use std::path::PathBuf;

use daglang_driver::{compile_from_context, DriverContext};

/// Compile a real module, emit artifact `.dag`, verify it re-parses cleanly.
#[test]
fn artifact_round_trip_pragma() {
    let dsl_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../dsl");
    let context = DriverContext {
        roots: vec![dsl_root.clone()],
        target_file: Some(dsl_root.join("tools/pragma.dag")),
    };

    let output = compile_from_context(&context).expect("pragma.dag should compile");

    // Emit artifact .dag
    let artifact_dag = output.emit_artifact_dag("generated.pragma_registry");

    // Verify basic structure
    assert!(
        artifact_dag.contains("module generated.pragma_registry"),
        "should contain module declaration"
    );
    assert!(
        artifact_dag.contains("type EntrypointInfo"),
        "should contain EntrypointInfo type def"
    );
    assert!(
        artifact_dag.contains("data entrypoints: List<EntrypointInfo>"),
        "should contain entrypoints data declaration"
    );
    assert!(
        artifact_dag.contains("data output_paths: List<String>"),
        "should contain output_paths data declaration"
    );

    // Verify entrypoint data is populated
    assert!(
        artifact_dag.contains("func_name:"),
        "should contain entrypoint entries"
    );

    // Re-parse the emitted .dag to verify it's syntactically valid
    let parsed = daglang_syntax::parser::parse(&artifact_dag);
    assert!(
        parsed.is_ok(),
        "emitted artifact .dag should parse without errors: {:?}",
        parsed.err()
    );
}

/// Compile makegen, emit artifact, verify output_paths populated.
#[test]
fn artifact_round_trip_makegen() {
    let dsl_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../dsl");
    let context = DriverContext {
        roots: vec![dsl_root.clone()],
        target_file: Some(dsl_root.join("tools/makegen.dag")),
    };

    let output = compile_from_context(&context).expect("makegen.dag should compile");
    let artifact_dag = output.emit_artifact_dag("generated.makegen_registry");

    assert!(artifact_dag.contains("module generated.makegen_registry"));

    // Makegen should have output_paths (Makefile, .gitignore)
    assert!(
        artifact_dag.contains("data output_paths: List<String>"),
        "should have output_paths"
    );

    // Re-parse
    let parsed = daglang_syntax::parser::parse(&artifact_dag);
    assert!(
        parsed.is_ok(),
        "emitted artifact .dag should parse without errors: {:?}",
        parsed.err()
    );
}

/// Empty compilation produces valid (empty-list) artifact.
#[test]
fn artifact_empty_data() {
    use daglang_emit::dag_emit::{emit_data_dag, DataEntry, TypeDef};

    let types = vec![TypeDef {
        name: "Foo".to_string(),
        fields: vec![("x".to_string(), "Int".to_string())],
    }];
    let data = vec![DataEntry {
        name: "items".to_string(),
        type_expr: "List<Foo>".to_string(),
        value: serde_json::json!([]),
    }];

    let result = emit_data_dag("generated.test_empty", &types, &data);
    assert!(result.contains("data items: List<Foo> = []"));

    let parsed = daglang_syntax::parser::parse(&result);
    assert!(
        parsed.is_ok(),
        "empty artifact should parse: {:?}",
        parsed.err()
    );
}
