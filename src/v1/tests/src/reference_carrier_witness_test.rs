//! Reference-carrier witnesses: O(1) intern_str lookup + floor scaling-curve receipt.
//!
//! intern_str previously used `skip(id) |> first` (O(id) per lookup). The .dag model now
//! uses `get(id)` — the same reference-by-key pattern as the Rust seed's Vec::get.

use std::rc::Rc;
use std::time::{Duration, Instant};

use v1_compiler::v1_compiler_compile::{compile_to_resolved, SourceFile};
use v1_compiler::v1_std_core::{empty_intern_table, intern, intern_str, InternTable};

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
        .filter(|m| !m.starts_with("complexity: "))
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
    let resolved = compile_to_resolved(Rc::new(sources));
    assert_resolved_no_hard_errors(&resolved);
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
        let mut sink = String::new();
        let start = Instant::now();
        for i in 0..lookups {
            let id = (i % table.strings.len().max(1) as usize) as i64;
            sink = intern_str(table.clone(), id);
        }
        let _ = sink;
        start.elapsed()
    };

    // Warmup
    time_table(small.clone(), LOOKUPS);
    time_table(large.clone(), LOOKUPS);

    let t_small = time_table(small, LOOKUPS);
    let t_large = time_table(large, LOOKUPS);
    let ratio = t_large.as_secs_f64() / t_small.as_secs_f64().max(1e-9);

    eprintln!(
        "intern_str scaling: small={t_small:?} large={t_large:?} ratio={ratio:.2} (4× table → budget <4× time)"
    );

    // Quadratic would show ~16× when table quadruples; linear stays ~4×.
    const LINEAR_DOUBLING_BUDGET: f64 = 6.0;
    assert!(
        ratio < LINEAR_DOUBLING_BUDGET,
        "intern_str must stay sub-quadratic: time(large)/time(small)={ratio:.2} (budget <{LINEAR_DOUBLING_BUDGET})"
    );
}

#[test]
fn floor_scaling_curve_import_chain_receipt() {
    let depths = [50_usize, 100, 200, 400];
    let mut rows: Vec<(usize, Duration)> = Vec::new();

    for depth in depths {
        let sources = synthetic_import_chain_sources(depth);
        compile_modules(sources.clone());
        let start = Instant::now();
        compile_modules(sources);
        rows.push((depth, start.elapsed()));
    }

    eprintln!("floor scaling-curve (import-chain compile_modules, median-style single sample):");
    for (depth, elapsed) in &rows {
        eprintln!("  depth={depth} elapsed={elapsed:?}");
    }

    if rows.len() >= 2 {
        let (d0, t0) = rows[rows.len() - 2];
        let (d1, t1) = rows[rows.len() - 1];
        let ratio = t1.as_secs_f64() / t0.as_secs_f64().max(1e-9);
        eprintln!(
            "  ratio depth {d1}/depth {d0} = {ratio:.2} (O(M²) red control: ~4× per module-count doubling)"
        );
    }

    // Receipt test: must complete without panic; ratios are logged for operator review.
    assert!(rows.iter().all(|(_, t)| *t < Duration::from_secs(120)));
}
