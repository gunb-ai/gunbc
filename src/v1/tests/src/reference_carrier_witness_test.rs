//! Reference-carrier witnesses: O(1) intern_str lookup + import-chain scaling receipt.
//!
//! intern_str previously used `skip(id) |> first` (O(id) per lookup). The .dag model now
//! uses `get(id)` — the same reference-by-key pattern as the Rust seed's Vec::get.
//!
//! The import-chain scaling receipt mirrors `type_env_scope_chain_test` (median sampling +
//! `SUB_QUADRATIC_DOUBLING_BUDGET`). It is a discriminating proxy for §6 baseline measurement
//! on synthetic chains; the whole-corpus `dag_compile_clean_gate` wall-clock is tracked separately.

use std::rc::Rc;
use std::time::{Duration, Instant};

use v1_compiler::v1_compiler_compile::{compile_to_resolved, SourceFile};
use v1_compiler::v1_std_core::{empty_intern_table, intern, intern_str, InternTable};

/// O(M²) shows ~4× wall-clock per module-count doubling; linear stays sub-4×.
const SUB_QUADRATIC_DOUBLING_BUDGET: f64 = 4.0;

fn build_intern_table(count: usize) -> Rc<InternTable> {
    let mut table = empty_intern_table();
    for i in 0..count {
        table = intern(table, format!("sym_{i}")).table.clone();
    }
    table
}

fn assert_resolved_no_hard_errors(
    resolved: &v1_compiler::v1_compiler_compile::ResolvedPipelineResult,
) {
    let msgs: Vec<String> = resolved
        .diagnostics
        .iter()
        .map(|d| v1_compiler::v1_std_core::diagnostic_to_message(d.diagnostic.clone()))
        .filter(|m| !m.starts_with("complexity: ") && !m.starts_with("unlisted import use "))
        .collect();
    assert!(
        msgs.is_empty() && resolved.graph.is_some(),
        "expected resolved graph, got diagnostics {:?}",
        msgs
    );
}

fn synthetic_import_chain_sources(depth: usize) -> Vec<Rc<SourceFile>> {
    (0..depth)
        .map(|i| {
            let content = if i == 0 {
                format!("module test.chain_{i}\ntype T{i} = Int\n")
            } else {
                format!(
                    "module test.chain_{i}\nimport test.chain_{}\ntype T{i} = T{}\n",
                    i - 1,
                    i - 1
                )
            };
            Rc::new(SourceFile {
                path: format!("chain_{i}.dag"),
                content,
            })
        })
        .collect()
}

fn compile_modules(sources: Vec<Rc<SourceFile>>) {
    let resolved = compile_to_resolved(Rc::new(sources.into()));
    assert_resolved_no_hard_errors(&resolved);
}

fn time_import_chain_compile(depth: usize) -> Duration {
    let sources = synthetic_import_chain_sources(depth);
    compile_modules(sources.clone());
    let mut samples = Vec::new();
    for _ in 0..7 {
        let start = Instant::now();
        compile_modules(sources.clone());
        samples.push(start.elapsed());
    }
    samples.sort();
    samples[samples.len() / 2]
}

#[test]
fn intern_str_o1_lookup_returns_correct_strings() {
    let table = build_intern_table(8);
    assert_eq!(intern_str(table.clone(), 0), "");
    assert_eq!(intern_str(table.clone(), 1), "sym_0");
    assert_eq!(intern_str(table.clone(), 4), "sym_3");
    assert_eq!(intern_str(table.clone(), 8), "sym_7");
    assert_eq!(intern_str(table, 99), "");
}

#[test]
fn intern_str_lookup_stays_linear_not_quadratic() {
    const SMALL: usize = 256;
    const LARGE: usize = 1024;
    const LOOKUPS: usize = 50_000;

    let small = build_intern_table(SMALL);
    let large = build_intern_table(LARGE);

    let time_table = |table: Rc<InternTable>, lookups: usize| -> Duration {
        let len = table.strings.len().max(1);
        let mut sink = String::new();
        let start = Instant::now();
        for i in 0..lookups {
            let id = (i % len) as i64;
            sink = intern_str(table.clone(), id);
        }
        let _ = sink;
        start.elapsed()
    };

    time_table(small.clone(), LOOKUPS);
    time_table(large.clone(), LOOKUPS);

    let t_small = time_table(small, LOOKUPS);
    let t_large = time_table(large, LOOKUPS);
    let ratio = t_large.as_secs_f64() / t_small.as_secs_f64().max(1e-9);

    eprintln!(
        "intern_str scaling: small={t_small:?} large={t_large:?} ratio={ratio:.2} (4× table → budget <{SUB_QUADRATIC_DOUBLING_BUDGET}× time)"
    );

    assert!(
        ratio < SUB_QUADRATIC_DOUBLING_BUDGET,
        "intern_str must stay sub-quadratic: time(large)/time(small)={ratio:.2} (budget <{SUB_QUADRATIC_DOUBLING_BUDGET})"
    );
}

#[test]
fn floor_scaling_curve_import_chain_sub_quadratic_receipt() {
    let d50 = time_import_chain_compile(50);
    let d100 = time_import_chain_compile(100);
    let d200 = time_import_chain_compile(200);
    let d400 = time_import_chain_compile(400);

    let ratio_100_50 = d100.as_secs_f64() / d50.as_secs_f64().max(1e-9);
    let ratio_200_100 = d200.as_secs_f64() / d100.as_secs_f64().max(1e-9);
    let ratio_400_200 = d400.as_secs_f64() / d200.as_secs_f64().max(1e-9);

    eprintln!(
        "import-chain scaling receipt: d50={d50:?} d100={d100:?} d200={d200:?} d400={d400:?} \
         ratio_100/50={ratio_100_50:.2} ratio_200/100={ratio_200_100:.2} ratio_400/200={ratio_400_200:.2}"
    );

    assert!(
        ratio_100_50 < SUB_QUADRATIC_DOUBLING_BUDGET,
        "import-chain compile must stay sub-quadratic: time(100)/time(50)={ratio_100_50:.2} (budget <{SUB_QUADRATIC_DOUBLING_BUDGET})"
    );
    assert!(
        ratio_200_100 < SUB_QUADRATIC_DOUBLING_BUDGET,
        "import-chain compile must stay sub-quadratic: time(200)/time(100)={ratio_200_100:.2} (budget <{SUB_QUADRATIC_DOUBLING_BUDGET})"
    );
    assert!(
        ratio_400_200 < SUB_QUADRATIC_DOUBLING_BUDGET,
        "import-chain compile must stay sub-quadratic: time(400)/time(200)={ratio_400_200:.2} (budget <{SUB_QUADRATIC_DOUBLING_BUDGET})"
    );
}
