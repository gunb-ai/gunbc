//! FC-P6-0: flat_map e2e validation.
//!
//! Proves that `flat_map` works end-to-end through the DSL compiler pipeline:
//! parse → typecheck → lower → evaluate.

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

use daglang_driver::{compile_from_context, DriverContext};
use daglang_lower::{CallableKind, LoweredFnBody, LoweredOp};
use gunbc_ir::node::NodeBody;
use gunbc_ir::Value;

static COUNTER: AtomicUsize = AtomicUsize::new(0);

/// Write a temp .dag file, compile it, extract fn bodies, and return them.
/// Each call uses a unique temp directory to avoid parallel test races.
#[allow(clippy::disallowed_methods)]
fn compile_dag_source(source: &str) -> HashMap<String, LoweredFnBody> {
    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!("gunbc_flat_map_e2e_{id}"));
    std::fs::create_dir_all(&dir).expect("create temp dir");

    let dag_file = dir.join("flat_map_test.dag");
    std::fs::write(&dag_file, source).expect("write dag file");

    let context = DriverContext {
        roots: vec![dir.clone()],
        target_file: Some(dag_file),
    };
    let output = compile_from_context(&context).expect("DAG should compile");

    let mut fns = HashMap::new();
    for node in &output.lowered_dag.nodes {
        if let NodeBody::Opaque(LoweredOp::Callable {
            kind: CallableKind::Fn,
            name,
            fn_body: Some(body),
            ..
        }) = &node.body
        {
            fns.insert(name.clone(), *body.clone());
        }
    }

    // Clean up temp dir.
    let _ = std::fs::remove_dir_all(&dir);

    fns
}

fn eval_fn(
    fns: &HashMap<String, LoweredFnBody>,
    name: &str,
    inputs: HashMap<String, Value>,
) -> HashMap<String, Value> {
    let body = fns.get(name).unwrap_or_else(|| {
        let available: Vec<&String> = fns.keys().collect();
        panic!("fn '{name}' not found. Available fns: {available:?}");
    });
    daglang_lower::eval::evaluate_fn_body(body, &inputs, fns)
        .unwrap_or_else(|e| panic!("fn '{name}' evaluation should succeed: {e}"))
}

#[test]
fn flat_map_identity() {
    let fns = compile_dag_source(
        r#"
module flat_map_test

fn identity(values: List<String>) -> { out: List<String> } {
  result = values |> flat_map(v => [v])
  return { out: result }
}
"#,
    );

    let mut inputs = HashMap::new();
    inputs.insert(
        "values".to_string(),
        Value::List(vec![
            Value::Str("a".into()),
            Value::Str("b".into()),
            Value::Str("c".into()),
        ]),
    );

    let result = eval_fn(&fns, "identity", inputs);
    let out = result.get("out").expect("output 'out' should exist");
    match out {
        Value::List(items) => {
            let strs: Vec<&str> = items.iter().filter_map(Value::as_str).collect();
            assert_eq!(strs, vec!["a", "b", "c"]);
        }
        other => panic!("expected List, got {other:?}"),
    }
}

#[test]
fn flat_map_expansion() {
    let fns = compile_dag_source(
        r#"
module flat_map_test

fn expand(values: List<String>) -> { out: List<String> } {
  result = values |> flat_map(v => [v, v])
  return { out: result }
}
"#,
    );

    let mut inputs = HashMap::new();
    inputs.insert(
        "values".to_string(),
        Value::List(vec![Value::Str("x".into()), Value::Str("y".into())]),
    );

    let result = eval_fn(&fns, "expand", inputs);
    let out = result.get("out").expect("output 'out' should exist");
    match out {
        Value::List(items) => {
            let strs: Vec<&str> = items.iter().filter_map(Value::as_str).collect();
            assert_eq!(strs, vec!["x", "x", "y", "y"]);
        }
        other => panic!("expected List, got {other:?}"),
    }
}

#[test]
fn flat_map_chained_with_join() {
    let fns = compile_dag_source(
        r#"
module flat_map_test

fn chained(values: List<String>) -> { out: String } {
  result = values
    |> flat_map(v => [v])
    |> join(",")
  return { out: result }
}
"#,
    );

    let mut inputs = HashMap::new();
    inputs.insert(
        "values".to_string(),
        Value::List(vec![
            Value::Str("a".into()),
            Value::Str("b".into()),
            Value::Str("c".into()),
        ]),
    );

    let result = eval_fn(&fns, "chained", inputs);
    let out = result.get("out").expect("output 'out' should exist");
    assert_eq!(out.as_str(), Some("a,b,c"));
}

#[test]
fn flat_map_empty_list() {
    let fns = compile_dag_source(
        r#"
module flat_map_test

fn empty(values: List<String>) -> { out: List<String> } {
  result = values |> flat_map(v => [v])
  return { out: result }
}
"#,
    );

    let mut inputs = HashMap::new();
    inputs.insert("values".to_string(), Value::List(vec![]));

    let result = eval_fn(&fns, "empty", inputs);
    let out = result.get("out").expect("output 'out' should exist");
    match out {
        Value::List(items) => assert!(items.is_empty()),
        other => panic!("expected empty List, got {other:?}"),
    }
}
