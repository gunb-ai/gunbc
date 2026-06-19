//! Throwaway resolve-perf bisect for N_v2 gate direct imports (#5146-class pathology).
//!
//! Run: for t in program_assembly compiler_closure_emit dag_language_model rust_target_model host_manifest_stub; do
//!        timeout 180 cargo test -p v1-compiler-tests nv2_gate_resolve_bisect_${t} -- --ignored --exact --nocapture;
//!      done

use std::time::{Duration, Instant};

use v1_compiler::cli_run::resolve_entry_graph;

use crate::helpers::workspace_root;

const BISECTS: &[(&str, &str)] = &[
    (
        "program_assembly",
        "src/v2/compiler/manual/nv2_gate_resolve_bisect_program_assembly.dag",
    ),
    (
        "compiler_closure_emit",
        "src/v2/compiler/manual/nv2_gate_resolve_bisect_compiler_closure_emit.dag",
    ),
    (
        "dag_language_model",
        "src/v2/compiler/manual/nv2_gate_resolve_bisect_dag_language_model.dag",
    ),
    (
        "rust_target_model",
        "src/v2/compiler/manual/nv2_gate_resolve_bisect_rust_target_model.dag",
    ),
    (
        "host_manifest_stub",
        "src/v2/compiler/manual/nv2_gate_resolve_bisect_host_manifest_stub.dag",
    ),
];

fn timed_resolve(label: &str, entry: &str, budget: Duration) {
    let ws = workspace_root();
    let roots = vec![ws.join("src/v2").to_string_lossy().to_string()];
    eprintln!("bisect {label}: resolve_entry_graph({entry}) starting (budget {budget:?})...");
    let start = Instant::now();
    let result = resolve_entry_graph(&roots, entry);
    let elapsed = start.elapsed();
    match &result {
        Ok(_) => eprintln!("bisect {label}: OK in {elapsed:?}"),
        Err(e) => eprintln!("bisect {label}: ERR in {elapsed:?}: {e}"),
    }
    if elapsed > budget {
        eprintln!("bisect {label}: EXCEEDED budget {budget:?} (possible #5146-class hang)");
    }
}

macro_rules! bisect_test {
    ($name:ident, $idx:expr) => {
        #[test]
        #[ignore]
        fn $name() {
            let (label, entry) = BISECTS[$idx];
            timed_resolve(label, entry, Duration::from_secs(120));
        }
    };
}

bisect_test!(nv2_gate_resolve_bisect_program_assembly, 0);
bisect_test!(nv2_gate_resolve_bisect_compiler_closure_emit, 1);
bisect_test!(nv2_gate_resolve_bisect_dag_language_model, 2);
bisect_test!(nv2_gate_resolve_bisect_rust_target_model, 3);
bisect_test!(nv2_gate_resolve_bisect_host_manifest_stub, 4);
