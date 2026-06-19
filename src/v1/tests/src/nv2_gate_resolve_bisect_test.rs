//! Solo resolve-perf bisect for N_v2 hang (#5146 full-interface elaboration).
//! dissolve-on: #5146-class N_v2 resolve hang root-caused — delete manual/nv2_gate_resolve_bisect_*.dag + ignored Rust bisect tests.
//!
//! Run (parent order): target_model → 06_translate → find_witness → languages
//!   for t in target_model 06_translate find_witness languages_dag languages_rust compiler_closure_emit; do
//!     timeout 600 cargo test -p v1-compiler-tests "nv2_gate_resolve_bisect_test::nv2_gate_resolve_bisect_${t}" -- --ignored --exact --nocapture;
//!   done

use std::time::Instant;

use v1_compiler::cli_run::resolve_entry_graph;

use crate::helpers::workspace_root;

const BISECTS: &[(&str, &str)] = &[
    (
        "target_model",
        "src/v2/compiler/manual/nv2_gate_resolve_bisect_target_model.dag",
    ),
    (
        "06_translate",
        "src/v2/compiler/manual/nv2_gate_resolve_bisect_06_translate.dag",
    ),
    (
        "find_witness",
        "src/v2/compiler/manual/nv2_gate_resolve_bisect_find_witness.dag",
    ),
    (
        "languages_dag",
        "src/v2/compiler/manual/nv2_gate_resolve_bisect_languages_dag.dag",
    ),
    (
        "languages_rust",
        "src/v2/compiler/manual/nv2_gate_resolve_bisect_rust_target_model.dag",
    ),
    (
        "compiler_closure_emit",
        "src/v2/compiler/manual/nv2_gate_resolve_bisect_compiler_closure_emit.dag",
    ),
    (
        "program_assembly",
        "src/v2/compiler/manual/nv2_gate_resolve_bisect_program_assembly.dag",
    ),
    (
        "host_manifest_stub",
        "src/v2/compiler/manual/nv2_gate_resolve_bisect_host_manifest_stub.dag",
    ),
];

fn timed_resolve(label: &str, entry: &str) {
    let ws = workspace_root();
    let entry_path = ws.join(entry);
    let entry_str = entry_path.to_str().expect("entry utf8");
    let roots = vec![ws.join("src/v2").to_string_lossy().to_string()];
    eprintln!("bisect {label}: resolve_entry_graph solo starting...");
    let start = Instant::now();
    let result = resolve_entry_graph(&roots, entry_str);
    let elapsed = start.elapsed();
    let secs = elapsed.as_secs_f64();
    match &result {
        Ok(_) => eprintln!("bisect {label}: OK in {elapsed:?} ({secs:.1}s)"),
        Err(e) => eprintln!("bisect {label}: ERR in {elapsed:?} ({secs:.1}s): {e}"),
    }
    if secs >= 120.0 {
        eprintln!("bisect {label}: SLOW (>=120s) — #5146-class full-interface elaboration suspect");
    }
}

macro_rules! bisect_test {
    ($name:ident, $idx:expr) => {
        #[test]
        #[ignore]
        fn $name() {
            let (label, entry) = BISECTS[$idx];
            timed_resolve(label, entry);
        }
    };
}

bisect_test!(nv2_gate_resolve_bisect_target_model, 0);
bisect_test!(nv2_gate_resolve_bisect_06_translate, 1);
bisect_test!(nv2_gate_resolve_bisect_find_witness, 2);
bisect_test!(nv2_gate_resolve_bisect_languages_dag, 3);
bisect_test!(nv2_gate_resolve_bisect_languages_rust, 4);
bisect_test!(nv2_gate_resolve_bisect_compiler_closure_emit, 5);
bisect_test!(nv2_gate_resolve_bisect_program_assembly, 6);
bisect_test!(nv2_gate_resolve_bisect_host_manifest_stub, 7);
