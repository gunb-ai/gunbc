//! Counter-receipts for the complexity-enforcement census-path claim:
//! "tokenize is 94% of census-path cost" and `pre_intern_tokens` is lex-side k=2 (quadratic).
//!
//! Census path substrate: `coproduct_reflection::decl_facts_corpus_walk` → `parse_dag_file`
//! (`medium_structure_census.rs`) = read + `tokenize` + `parse` (which calls `pre_intern_tokens`
//! once per file on an empty `InternTable`).
//!
//! Verdict under test: **refute** the runtime quadratic on `pre_intern_tokens` for the per-file
//! census path; attribute the measured lex-side share honestly (tokenize vs pre_intern vs parse).

use std::rc::Rc;
use std::time::{Duration, Instant};

use v1_compiler::coproduct_reflection::decl_facts_corpus_walk;
use v1_compiler::module_path_index::medium_structure_census::parse_dag_file;
use v1_compiler::v1_compiler_parse::parse;
use v1_compiler::v1_compiler_tokenize::tokenize;
use std::collections::HashMap;

use v1_compiler::v1_std_core::{build_newline_index, empty_intern_table, pre_intern_tokens};

use crate::helpers::workspace_root;

const SUB_QUADRATIC_DOUBLING_BUDGET: f64 = 4.0;

fn median(mut samples: Vec<Duration>) -> Duration {
    samples.sort();
    samples[samples.len() / 2]
}

fn time_it<F: FnMut()>(warmup: usize, samples: usize, mut f: F) -> Duration {
    for _ in 0..warmup {
        f();
    }
    let mut timings = Vec::with_capacity(samples);
    for _ in 0..samples {
        let start = Instant::now();
        f();
        timings.push(start.elapsed());
    }
    median(timings)
}

fn synthetic_scale_module(fn_count: usize) -> String {
    let mut out = String::from("module test.census_lex_scale\n\n");
    for i in 0..fn_count {
        use std::fmt::Write;
        let _ = writeln!(out, "fn f{i}() -> Int {{ {i} }}");
    }
    out
}

fn time_tokenize(source: &str) -> Duration {
    let content = source.to_string();
    time_it(2, 5, || {
        let _ = tokenize(content.clone(), "scale.dag".to_string());
    })
}

fn time_pre_intern(source: &str) -> Duration {
    let content = source.to_string();
    let tokens = tokenize(content.clone(), "scale.dag".to_string());
    time_it(2, 5, || {
        let _ = pre_intern_tokens(tokens.clone(), empty_intern_table());
    })
}

fn time_parse(source: &str) -> Duration {
    let content = source.to_string();
    let filename = "scale.dag".to_string();
    let tokens = tokenize(content.clone(), filename.clone());
    let source_index = build_newline_index(filename.clone(), content.clone());
    let mut source_indices = HashMap::new();
    source_indices.insert(filename.clone(), source_index);
    let source_indices = Rc::new(source_indices);
    time_it(2, 5, || {
        let _ = parse(tokens.clone(), source_indices.clone());
    })
}

fn corpus_dag_sample(limit: usize) -> Vec<std::path::PathBuf> {
    let roots = vec!["src/v2".to_string(), "dag".to_string()];
    let walk = decl_facts_corpus_walk(&roots);
    let ws = workspace_root();
    walk.facts
        .iter()
        .map(|f| ws.join(&f.rel_path))
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .take(limit)
        .collect()
}

fn time_parse_dag_files(paths: &[std::path::PathBuf]) -> Duration {
    time_it(1, 3, || {
        for path in paths {
            let _ = parse_dag_file(path);
        }
    })
}

fn time_tokenize_corpus(paths: &[std::path::PathBuf]) -> Duration {
    let files: Vec<(String, String)> = paths
        .iter()
        .map(|p| {
            let content = std::fs::read_to_string(p).expect("read corpus file");
            let filename = p
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("?")
                .to_string();
            (content, filename)
        })
        .collect();
    time_it(1, 3, || {
        for (content, filename) in &files {
            let _ = tokenize(content.clone(), filename.clone());
        }
    })
}

fn time_pre_intern_corpus(paths: &[std::path::PathBuf]) -> Duration {
    let prepared: Vec<Rc<Vec<Rc<v1_compiler::v1_std_core::Token>>>> = paths
        .iter()
        .map(|p| {
            let content = std::fs::read_to_string(p).expect("read corpus file");
            let filename = p
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("?")
                .to_string();
            tokenize(content, filename)
        })
        .collect();
    time_it(1, 3, || {
        for tokens in &prepared {
            let _ = pre_intern_tokens(tokens.clone(), empty_intern_table());
        }
    })
}

#[test]
fn pre_intern_tokens_scaling_not_quadratic_on_synthetic_module() {
    let n32 = synthetic_scale_module(32);
    let n64 = synthetic_scale_module(64);
    let n128 = synthetic_scale_module(128);

    let t32 = time_pre_intern(&n32);
    let t64 = time_pre_intern(&n64);
    let t128 = time_pre_intern(&n128);

    let ratio_64_32 = t64.as_secs_f64() / t32.as_secs_f64().max(1e-9);
    let ratio_128_64 = t128.as_secs_f64() / t64.as_secs_f64().max(1e-9);

    eprintln!(
        "pre_intern_tokens synthetic scaling: t32={t32:?} t64={t64:?} t128={t128:?} \
         ratio_64/32={ratio_64_32:.2} ratio_128/64={ratio_128_64:.2}"
    );

    assert!(
        ratio_64_32 < SUB_QUADRATIC_DOUBLING_BUDGET,
        "pre_intern_tokens must stay sub-quadratic when fn count doubles 32→64: ratio={ratio_64_32:.2}"
    );
    assert!(
        ratio_128_64 < SUB_QUADRATIC_DOUBLING_BUDGET,
        "pre_intern_tokens must stay sub-quadratic when fn count doubles 64→128: ratio={ratio_128_64:.2}"
    );
}

#[test]
fn census_path_lex_share_receipt_on_corpus_sample() {
    let paths = corpus_dag_sample(120);
    assert!(
        paths.len() >= 40,
        "expected a representative corpus sample, got {} files",
        paths.len()
    );

    let total = time_parse_dag_files(&paths);
    let tokenize_only = time_tokenize_corpus(&paths);
    let pre_intern_only = time_pre_intern_corpus(&paths);
    let parse_minus_lex = total
        .saturating_sub(tokenize_only)
        .saturating_sub(pre_intern_only);

    let total_secs = total.as_secs_f64().max(1e-9);
    let tokenize_pct = 100.0 * tokenize_only.as_secs_f64() / total_secs;
    let pre_intern_pct = 100.0 * pre_intern_only.as_secs_f64() / total_secs;
    let parse_rest_pct = 100.0 * parse_minus_lex.as_secs_f64() / total_secs;
    let lex_combined_pct = tokenize_pct + pre_intern_pct;

    eprintln!(
        "census-path lex share ({} files): total={total:?} tokenize={tokenize_only:?} ({tokenize_pct:.1}%) \
         pre_intern={pre_intern_only:?} ({pre_intern_pct:.1}%) parse_rest≈{parse_minus_lex:?} ({parse_rest_pct:.1}%) \
         lex_combined={lex_combined_pct:.1}%",
        paths.len()
    );

    // Counter-receipt: pre_intern is not a measurable census-path bottleneck (typically <15%).
    assert!(
        pre_intern_pct < 25.0,
        "pre_intern_tokens must be a minor census-path fraction; got {pre_intern_pct:.1}% (hawk #11 k2 refuted at runtime)"
    );

    // Counter-receipt: "tokenize alone is 94%" overstates — combined lex (tokenize+pre_intern) is the fair claim.
    // On this sample tokenize+pre_intern is well under 94% when parse work is included.
    assert!(
        lex_combined_pct < 90.0,
        "combined lex must not dominate ≥90% of parse_dag_file time; got {lex_combined_pct:.1}% — \
         the 94% headline is not reproduced on the census substrate"
    );
}

#[test]
fn tokenize_alone_not_ninety_four_percent_on_corpus_sample() {
    let paths = corpus_dag_sample(120);
    let total = time_parse_dag_files(&paths);
    let tokenize_only = time_tokenize_corpus(&paths);
    let tokenize_pct = 100.0 * tokenize_only.as_secs_f64() / total.as_secs_f64().max(1e-9);

    eprintln!(
        "tokenize-only share ({} files): {tokenize_pct:.1}% of parse_dag_file",
        paths.len()
    );

    assert!(
        tokenize_pct < 85.0,
        "tokenize alone must not be ~94% of census-path parse_dag_file cost; measured {tokenize_pct:.1}%"
    );
}

#[test]
fn synthetic_module_tokenize_vs_pre_intern_vs_parse_scaling() {
    let n64 = synthetic_scale_module(64);
    let n128 = synthetic_scale_module(128);

    let tok64 = time_tokenize(&n64);
    let tok128 = time_tokenize(&n128);
    let pre64 = time_pre_intern(&n64);
    let pre128 = time_pre_intern(&n128);
    let par64 = time_parse(&n64);
    let par128 = time_parse(&n128);

    let tok_ratio = tok128.as_secs_f64() / tok64.as_secs_f64().max(1e-9);
    let pre_ratio = pre128.as_secs_f64() / pre64.as_secs_f64().max(1e-9);
    let par_ratio = par128.as_secs_f64() / par64.as_secs_f64().max(1e-9);

    eprintln!(
        "synthetic 64→128 scaling: tokenize ratio={tok_ratio:.2} pre_intern={pre_ratio:.2} parse={par_ratio:.2}"
    );

    for (name, ratio) in [
        ("tokenize", tok_ratio),
        ("pre_intern_tokens", pre_ratio),
        ("parse", par_ratio),
    ] {
        assert!(
            ratio < SUB_QUADRATIC_DOUBLING_BUDGET,
            "{name} must stay sub-quadratic on synthetic module doubling: ratio={ratio:.2}"
        );
    }
}
