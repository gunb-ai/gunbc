//! FC-CF7: zip() e2e validation.
//!
//! Proves that `zip(other)` works end-to-end through the DSL compiler
//! pipeline: parse → typecheck → lower → evaluate.

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

use daglang_driver::{compile_from_context, DriverContext};
use daglang_lower::{CallableKind, LoweredFnBody, LoweredOp};
use gunbc_ir::node::NodeBody;
use gunbc_ir::Value;

static COUNTER: AtomicUsize = AtomicUsize::new(0);

#[allow(clippy::disallowed_methods)]
fn compile_dag_source(source: &str) -> HashMap<String, LoweredFnBody> {
    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!("gunbc_zip_e2e_{id}"));
    std::fs::create_dir_all(&dir).expect("create temp dir");

    let dag_file = dir.join("zip_test.dag");
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
fn zip_basic() {
    let fns = compile_dag_source(
        r#"
module zip_test

fn pair_up(names: List<String>, ages: List<Int>) -> { pairs: List<Map<String, String>> } {
  result = names |> zip(other: ages)
  return { pairs: result }
}
"#,
    );

    let mut inputs = HashMap::new();
    inputs.insert(
        "names".to_string(),
        Value::List(vec![Value::Str("Alice".into()), Value::Str("Bob".into())]),
    );
    inputs.insert(
        "ages".to_string(),
        Value::List(vec![Value::Int(30), Value::Int(25)]),
    );

    let result = eval_fn(&fns, "pair_up", inputs);
    let pairs = result.get("pairs").expect("output 'pairs' should exist");
    match pairs {
        Value::List(items) => {
            assert_eq!(items.len(), 2);
            // First pair
            match &items[0] {
                Value::Map(m) => {
                    assert_eq!(m.get("first"), Some(&Value::Str("Alice".into())));
                    assert_eq!(m.get("second"), Some(&Value::Int(30)));
                }
                other => panic!("expected Map, got {other:?}"),
            }
            // Second pair
            match &items[1] {
                Value::Map(m) => {
                    assert_eq!(m.get("first"), Some(&Value::Str("Bob".into())));
                    assert_eq!(m.get("second"), Some(&Value::Int(25)));
                }
                other => panic!("expected Map, got {other:?}"),
            }
        }
        other => panic!("expected List, got {other:?}"),
    }
}

#[test]
fn zip_truncates_to_shorter() {
    let fns = compile_dag_source(
        r#"
module zip_test

fn truncate(left: List<String>, right: List<Int>) -> { pairs: List<Map<String, String>> } {
  result = left |> zip(other: right)
  return { pairs: result }
}
"#,
    );

    let mut inputs = HashMap::new();
    inputs.insert(
        "left".to_string(),
        Value::List(vec![
            Value::Str("a".into()),
            Value::Str("b".into()),
            Value::Str("c".into()),
        ]),
    );
    inputs.insert(
        "right".to_string(),
        Value::List(vec![Value::Int(1), Value::Int(2)]),
    );

    let result = eval_fn(&fns, "truncate", inputs);
    let pairs = result.get("pairs").expect("output 'pairs' should exist");
    match pairs {
        Value::List(items) => {
            assert_eq!(items.len(), 2, "should truncate to shorter list");
        }
        other => panic!("expected List, got {other:?}"),
    }
}

#[test]
fn zip_empty_list() {
    let fns = compile_dag_source(
        r#"
module zip_test

fn empty(left: List<String>, right: List<Int>) -> { pairs: List<Map<String, String>> } {
  result = left |> zip(other: right)
  return { pairs: result }
}
"#,
    );

    let mut inputs = HashMap::new();
    inputs.insert("left".to_string(), Value::List(vec![]));
    inputs.insert(
        "right".to_string(),
        Value::List(vec![Value::Int(1)]),
    );

    let result = eval_fn(&fns, "empty", inputs);
    let pairs = result.get("pairs").expect("output 'pairs' should exist");
    match pairs {
        Value::List(items) => {
            assert!(items.is_empty(), "zipping empty list should produce empty list");
        }
        other => panic!("expected List, got {other:?}"),
    }
}

#[test]
fn zip_chained_with_map_and_join() {
    let fns = compile_dag_source(
        r#"
module zip_test

fn format_pairs(files: List<String>, contents: List<String>) -> { out: String } {
  result = files
    |> zip(other: contents)
    |> map(pair => pair.first + "=" + pair.second)
    |> join(",")
  return { out: result }
}
"#,
    );

    let mut inputs = HashMap::new();
    inputs.insert(
        "files".to_string(),
        Value::List(vec![Value::Str("a.rs".into()), Value::Str("b.rs".into())]),
    );
    inputs.insert(
        "contents".to_string(),
        Value::List(vec![
            Value::Str("fn a() {}".into()),
            Value::Str("fn b() {}".into()),
        ]),
    );

    let result = eval_fn(&fns, "format_pairs", inputs);
    let out = result.get("out").expect("output 'out' should exist");
    let s = out.as_str().expect("should be string");
    assert!(s.contains("a.rs=fn a() {}"), "should contain first pair: {s}");
    assert!(s.contains("b.rs=fn b() {}"), "should contain second pair: {s}");
}
