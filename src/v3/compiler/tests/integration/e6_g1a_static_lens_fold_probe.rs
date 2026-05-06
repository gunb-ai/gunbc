//! Temporary probe — delete after confirming compile shape.
use crate::common::cached_compile_any;

#[test]
#[ignore]
fn probe_dag_data_literal() {
    let src = r#"
import std.list { empty }
import std.substrate { Dag }

data stub_dag: Dag = {
  declarations: empty(),
  nodes: empty(),
  ports: empty(),
  clusters: empty()
}
"#;
    let dag = cached_compile_any(src, "probe_dag_data.v3");
    for (port, d) in dag.diagnostics().iter() {
        eprintln!("{port:?}: {d:?}");
    }
    assert!(
        dag.diagnostics().is_empty(),
        "expected Dag data literal to compile"
    );
}
