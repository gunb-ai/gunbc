//! FC-CF1: split() e2e validation.
//!
//! Proves that `split(delimiter)` works end-to-end through the DSL compiler
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
    let dir = std::env::temp_dir().join(format!("gunbc_split_e2e_{id}"));
    std::fs::create_dir_all(&dir).expect("create temp dir");

    let dag_file = dir.join("split_test.dag");
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
fn split_basic_path() {
    let fns = compile_dag_source(
        r#"
module split_test

fn split_path(path: String) -> { parts: List<String> } {
  result = path |> split(delimiter: "/")
  return { parts: result }
}
"#,
    );

    let mut inputs = HashMap::new();
    inputs.insert("path".to_string(), Value::Str("a/b/c".into()));

    let result = eval_fn(&fns, "split_path", inputs);
    let parts = result.get("parts").expect("output 'parts' should exist");
    match parts {
        Value::List(items) => {
            let strs: Vec<&str> = items.iter().filter_map(Value::as_str).collect();
            assert_eq!(strs, vec!["a", "b", "c"]);
        }
        other => panic!("expected List, got {other:?}"),
    }
}

#[test]
fn split_no_delimiter_in_string() {
    let fns = compile_dag_source(
        r#"
module split_test

fn no_match(s: String) -> { parts: List<String> } {
  result = s |> split(delimiter: "x")
  return { parts: result }
}
"#,
    );

    let mut inputs = HashMap::new();
    inputs.insert("s".to_string(), Value::Str("hello".into()));

    let result = eval_fn(&fns, "no_match", inputs);
    let parts = result.get("parts").expect("output 'parts' should exist");
    match parts {
        Value::List(items) => {
            let strs: Vec<&str> = items.iter().filter_map(Value::as_str).collect();
            assert_eq!(strs, vec!["hello"]);
        }
        other => panic!("expected List, got {other:?}"),
    }
}

#[test]
fn split_empty_string() {
    let fns = compile_dag_source(
        r#"
module split_test

fn split_empty(s: String) -> { parts: List<String> } {
  result = s |> split(delimiter: "/")
  return { parts: result }
}
"#,
    );

    let mut inputs = HashMap::new();
    inputs.insert("s".to_string(), Value::Str(String::new()));

    let result = eval_fn(&fns, "split_empty", inputs);
    let parts = result.get("parts").expect("output 'parts' should exist");
    match parts {
        Value::List(items) => {
            let strs: Vec<&str> = items.iter().filter_map(Value::as_str).collect();
            assert_eq!(strs, vec![""]);
        }
        other => panic!("expected List, got {other:?}"),
    }
}

#[test]
fn split_multi_char_delimiter() {
    let fns = compile_dag_source(
        r#"
module split_test

fn split_double_colon(s: String) -> { parts: List<String> } {
  result = s |> split(delimiter: "::")
  return { parts: result }
}
"#,
    );

    let mut inputs = HashMap::new();
    inputs.insert("s".to_string(), Value::Str("a::b::c".into()));

    let result = eval_fn(&fns, "split_double_colon", inputs);
    let parts = result.get("parts").expect("output 'parts' should exist");
    match parts {
        Value::List(items) => {
            let strs: Vec<&str> = items.iter().filter_map(Value::as_str).collect();
            assert_eq!(strs, vec!["a", "b", "c"]);
        }
        other => panic!("expected List, got {other:?}"),
    }
}

#[test]
fn split_chained_with_count() {
    let fns = compile_dag_source(
        r#"
module split_test

fn depth(path: String) -> { depth: Int } {
  parts = path |> split(delimiter: "/")
  n = parts |> count()
  return { depth: n }
}
"#,
    );

    let mut inputs = HashMap::new();
    inputs.insert("path".to_string(), Value::Str("src/core/lib".into()));

    let result = eval_fn(&fns, "depth", inputs);
    let depth = result.get("depth").expect("output 'depth' should exist");
    assert_eq!(*depth, Value::Int(3));
}
