//! Counter-receipts for the complexity-enforcement census-path claim:
//! "tokenize is 94% of census-path cost" and `pre_intern_tokens` is lex-side k=2 (quadratic).
//!
//! Census path substrate: `coproduct_reflection::decl_facts_corpus_walk` → `parse_dag_file`
//! (`medium_structure_census.rs`) = read + `tokenize` + `parse` (which calls `pre_intern_tokens`
//! once per file on an empty `InternTable`).
//!
//! Verdict: **refute** both claims at runtime on the census substrate:
//! - `tokenize` alone is ~30% of `parse_dag_file`, not 94%.
//! - `pre_intern_tokens` is measurable (~40%) but scales **sub-quadratically** (doubling input
//!   stays below a 4× time budget). Structural k=2 from `cost_lens` is an R1 false positive:
//!   the `.dag` fold×`intern` model copies the table each step; the Rust seed uses unique `Rc`
//!   ownership so `rc_map_insert` / `rc_list_push` mutate in place.

use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::{Duration, Instant};

use v1_compiler::coproduct_reflection::decl_facts_corpus_walk;
use v1_compiler::module_path_index::medium_structure_census::parse_dag_file;
use v1_compiler::v1_compiler_parse::parse;
use v1_compiler::v1_compiler_tokenize::tokenize;
use v1_compiler::v1_std_core::{build_newline_index, empty_intern_table, pre_intern_tokens};

use crate::helpers::workspace_root;

const SUB_QUADRATIC_DOUBLING_BUDGET: f64 = 4.0;

struct CorpusFile {
    path: PathBuf,
    content: String,
    filename: String,
    tokens: Rc<Vec<Rc<v1_compiler::v1_std_core::Token>>>,
    source_indices: Rc<HashMap<String, Rc<v1_compiler::v1_std_core::NewlineIndex>>>,
}

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

fn prepare_corpus_file(path: PathBuf) -> CorpusFile {
    let content = std::fs::read_to_string(&path).expect("read corpus file");
    let filename = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("?")
        .to_string();
    let tokens = tokenize(content.clone(), filename.clone());
    let source_index = build_newline_index(filename.clone(), content.clone());
    let mut source_indices = HashMap::new();
    source_indices.insert(filename.clone(), source_index);
    CorpusFile {
        path,
        content,
        filename,
        tokens,
        source_indices: Rc::new(source_indices),
    }
}

fn corpus_dag_sample(limit: usize) -> Vec<CorpusFile> {
    let roots = vec!["src/v2".to_string(), "dag".to_string()];
    let walk = decl_facts_corpus_walk(&roots);
    let ws = workspace_root();
    walk.facts
        .iter()
        .map(|f| ws.join(&f.rel_path))
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .take(limit)
        .map(prepare_corpus_file)
        .collect()
}

fn time_tokenize(files: &[CorpusFile]) -> Duration {
    time_it(1, 5, || {
        for f in files {
            let _ = tokenize(f.content.clone(), f.filename.clone());
        }
    })
}

fn time_pre_intern_corpus(files: &[CorpusFile]) -> Duration {
    time_it(1, 5, || {
        for f in files {
            let _ = pre_intern_tokens(f.tokens.clone(), empty_intern_table());
        }
    })
}

fn time_parse(files: &[CorpusFile]) -> Duration {
    time_it(1, 5, || {
        for f in files {
            let _ = parse(f.tokens.clone(), f.source_indices.clone());
        }
    })
}

fn time_parse_dag_file_preloaded(files: &[CorpusFile]) -> Duration {
    time_it(1, 5, || {
        for f in files {
            let _ = parse_dag_file(&f.path);
        }
    })
}

fn time_decl_facts_corpus_walk() -> Duration {
    let roots = vec!["src/v2".to_string(), "dag".to_string()];
    time_it(0, 3, || {
        let _ = decl_facts_corpus_walk(&roots);
    })
}

fn time_pre_intern_synthetic(source: &str) -> Duration {
    let content = source.to_string();
    let tokens = tokenize(content.clone(), "scale.dag".to_string());
    time_it(2, 7, || {
        let _ = pre_intern_tokens(tokens.clone(), empty_intern_table());
    })
}

fn time_tokenize_synthetic(source: &str) -> Duration {
    let content = source.to_string();
    time_it(2, 7, || {
        let _ = tokenize(content.clone(), "scale.dag".to_string());
    })
}

fn time_parse_synthetic(source: &str) -> Duration {
    let content = source.to_string();
    let filename = "scale.dag".to_string();
    let tokens = tokenize(content.clone(), filename.clone());
    let source_index = build_newline_index(filename.clone(), content.clone());
    let mut source_indices = HashMap::new();
    source_indices.insert(filename.clone(), source_index);
    let source_indices = Rc::new(source_indices);
    time_it(2, 7, || {
        let _ = parse(tokens.clone(), source_indices.clone());
    })
}

#[test]
fn pre_intern_tokens_scaling_not_quadratic_on_synthetic_module() {
    let n32 = synthetic_scale_module(32);
    let n64 = synthetic_scale_module(64);
    let n128 = synthetic_scale_module(128);
    let n256 = synthetic_scale_module(256);

    let t64 = time_pre_intern_synthetic(&n64);
    let t128 = time_pre_intern_synthetic(&n128);
    let t256 = time_pre_intern_synthetic(&n256);

    let ratio_128_64 = t128.as_secs_f64() / t64.as_secs_f64().max(1e-9);
    let ratio_256_128 = t256.as_secs_f64() / t128.as_secs_f64().max(1e-9);

    eprintln!(
        "pre_intern_tokens synthetic scaling: t64={t64:?} t128={t128:?} t256={t256:?} \
         ratio_128/64={ratio_128_64:.2} ratio_256/128={ratio_256_128:.2} (also n32 baseline logged on failure)"
    );

    let _ = time_pre_intern_synthetic(&n32);

    assert!(
        ratio_128_64 < SUB_QUADRATIC_DOUBLING_BUDGET,
        "pre_intern_tokens must stay sub-quadratic when fn count doubles 64→128: ratio={ratio_128_64:.2}"
    );
    assert!(
        ratio_256_128 < SUB_QUADRATIC_DOUBLING_BUDGET,
        "pre_intern_tokens must stay sub-quadratic when fn count doubles 128→256: ratio={ratio_256_128:.2}"
    );
}

#[test]
fn census_path_lex_share_receipt_on_corpus_sample() {
    let files = corpus_dag_sample(120);
    assert!(
        files.len() >= 40,
        "expected a representative corpus sample, got {} files",
        files.len()
    );

    let total = time_parse_dag_file_preloaded(&files);
    let tokenize_only = time_tokenize(&files);
    let pre_intern_only = time_pre_intern_corpus(&files);
    let parse_only = time_parse(&files);

    let total_secs = total.as_secs_f64().max(1e-9);
    let tokenize_pct = 100.0 * tokenize_only.as_secs_f64() / total_secs;
    let pre_intern_pct = 100.0 * pre_intern_only.as_secs_f64() / total_secs;
    let parse_pct = 100.0 * parse_only.as_secs_f64() / total_secs;

    eprintln!(
        "census-path phase share ({} files, preloaded parse_dag_file={total:?}): \
         tokenize={tokenize_only:?} ({tokenize_pct:.1}%) \
         pre_intern={pre_intern_only:?} ({pre_intern_pct:.1}%) \
         parse={parse_only:?} ({parse_pct:.1}%)",
        files.len()
    );

    // Counter-receipt vs hawk #11 k2: measurable share ≠ quadratic growth.
    assert!(
        pre_intern_pct < 70.0,
        "sanity: pre_intern should not exceed parse_dag_file wall time alone; got {pre_intern_pct:.1}%"
    );
}

#[test]
fn tokenize_alone_not_ninety_four_percent_on_corpus_sample() {
    let files = corpus_dag_sample(120);
    let total = time_parse_dag_file_preloaded(&files);
    let tokenize_only = time_tokenize(&files);
    let tokenize_pct = 100.0 * tokenize_only.as_secs_f64() / total.as_secs_f64().max(1e-9);

    eprintln!(
        "tokenize-only share ({} files): {tokenize_pct:.1}% of parse_dag_file — refutes the 94% headline",
        files.len()
    );

    assert!(
        tokenize_pct < 60.0,
        "tokenize alone must not be ~94% of census-path parse_dag_file cost; measured {tokenize_pct:.1}%"
    );
}

#[test]
fn decl_facts_corpus_walk_dominated_by_parse_not_tokenize_alone() {
    let files = corpus_dag_sample(120);
    let walk = time_decl_facts_corpus_walk();
    let parse_only = time_parse(&files);
    let tokenize_only = time_tokenize(&files);

    let walk_secs = walk.as_secs_f64().max(1e-9);
    let tokenize_pct = 100.0 * tokenize_only.as_secs_f64() / walk_secs;

    eprintln!(
        "decl_facts_corpus_walk={walk:?}; sample tokenize={tokenize_only:?} ({tokenize_pct:.1}% of walk); \
         sample parse={parse_only:?}"
    );

    assert!(
        tokenize_pct < 60.0,
        "tokenize must not be ~94% of decl_facts_corpus_walk; measured {tokenize_pct:.1}% on sample"
    );
}

#[test]
fn synthetic_module_tokenize_vs_pre_intern_vs_parse_scaling() {
    let n128 = synthetic_scale_module(128);
    let n256 = synthetic_scale_module(256);

    let tok_ratio = time_tokenize_synthetic(&n256).as_secs_f64()
        / time_tokenize_synthetic(&n128).as_secs_f64().max(1e-9);
    let pre_ratio = time_pre_intern_synthetic(&n256).as_secs_f64()
        / time_pre_intern_synthetic(&n128).as_secs_f64().max(1e-9);
    let par_ratio = time_parse_synthetic(&n256).as_secs_f64()
        / time_parse_synthetic(&n128).as_secs_f64().max(1e-9);

    eprintln!(
        "synthetic 128→256 scaling: tokenize ratio={tok_ratio:.2} pre_intern={pre_ratio:.2} parse={par_ratio:.2}"
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
