//! Local-only timing probe — #[ignore]; not a merge gate witness.
#![allow(clippy::disallowed_macros)]

use std::rc::Rc;
use std::time::Instant;

use v1_compiler::cli_run::{whole_tree_strict_sources, FLOOR_DISCOVERY_EXCLUDES};
use v1_compiler::v1_compiler_compile::{compile_to_resolved, emit_dag_artifact};
use v1_compiler::v1_compiler_dag_collect::collect_dag_nodes;

use crate::helpers::workspace_root;

#[test]
#[ignore = "local perf probe for dag emit phases"]
fn emit_dag_phase_timing_probe() {
    let root = workspace_root();
    let roots = vec![
        root.join("dag").to_string_lossy().into_owned(),
        root.join("src/v2").to_string_lossy().into_owned(),
    ];
    let excludes: Vec<String> = FLOOR_DISCOVERY_EXCLUDES
        .iter()
        .map(|s| (*s).to_string())
        .collect();
    let t0 = Instant::now();
    let picked = whole_tree_strict_sources(&roots, &excludes).expect("pick sources");
    eprintln!("pick: {} ms ({} modules)", t0.elapsed().as_millis(), picked.modules_resolved);
    let t1 = Instant::now();
    let result = compile_to_resolved(Rc::new(picked.sources));
    let graph = result.graph.clone().expect("resolved graph");
    eprintln!("resolve: {} ms", t1.elapsed().as_millis());
    let t_collect = Instant::now();
    let collected = collect_dag_nodes(graph.clone());
    eprintln!(
        "collect_dag_nodes: {} ms ({} nodes)",
        t_collect.elapsed().as_millis(),
        collected.order.len()
    );
    let t2 = Instant::now();
    let emit = emit_dag_artifact(graph);
    eprintln!(
        "emit_dag: {} ms ({} diagnostics)",
        t2.elapsed().as_millis(),
        emit.diagnostics.len()
    );
}
