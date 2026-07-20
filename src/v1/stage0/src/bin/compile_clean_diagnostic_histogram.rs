#![allow(clippy::disallowed_macros)]

//! SCAFFOLD (DESIGN §7 seed-retained HAND-RUST / P5) — host transport for the whole-tree
//! compile-clean hard-diagnostic histogram (namespace migration burndown).
//!
//! Runs the same resolve kernel as batch-1 `dag_compile_clean_gate` on main
//! (`witness_layer_roots` whole-tree closure + `compile_to_resolved`) but emits ALL hard
//! diagnostics aggregated by class and (class, name) — not the truncated first-20 window.
//!
//! NOT floor-enrolled — run standalone. Do NOT invoke from cargo tests (whole-tree resolve OOM
//! risk in test harness). Carrier: `CLI_RUN_COMPILE_CLEAN_DIAGNOSTIC_HISTOGRAM_SCAFFOLD_MARKER`
//! in `cli_run.rs`.
//!
//! DISSOLUTION: delete this bin and the marker-gated helpers when ROADMAP §1 namespace-only
//! lane closes (`docs/plans/namespace-resolution-design.md` — import strip + global_bare wiring
//! fixed; whole-tree compile-clean quiet on the strip tree) OR a floor-enrolled histogram lens
//! subsumes this transport. Receipt: `rg cli_run_compile_clean_diagnostic_histogram
//! src/v1/stage0` == 1 until deletion.

use std::collections::BTreeMap;
use std::process::ExitCode;
use std::rc::Rc;

use v1_compiler::cli_run::{
    compile_clean_diagnostic_histogram_key, compile_clean_whole_tree_hard_diagnostics,
    peak_rss_vhwm_bytes, workspace_root,
};
use v1_compiler::v1_std_core::{diagnostic_to_span, ErrorNode};

fn main() -> ExitCode {
    std::env::set_current_dir(workspace_root()).expect("chdir to workspace root");
    // Force whole-tree: CI scoping would shrink the closure.
    std::env::remove_var("GITHUB_ACTIONS");
    std::env::remove_var("GUNBC_CI_DIFF_BASE");

    eprintln!("compile_clean_diagnostic_histogram: starting whole-tree compile-clean…");
    let started = std::time::Instant::now();

    let diags = match compile_clean_whole_tree_hard_diagnostics() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("HISTOGRAM_STATUS error");
            eprintln!("HISTOGRAM_ERROR {e}");
            if let Some(rss) = peak_rss_vhwm_bytes() {
                eprintln!("HISTOGRAM_RSS_MIB {}", rss / (1024 * 1024));
            }
            return ExitCode::from(2);
        }
    };

    let elapsed = started.elapsed();
    let mut by_class: BTreeMap<String, usize> = BTreeMap::new();
    let mut by_class_name: BTreeMap<(String, String), usize> = BTreeMap::new();
    let mut by_file: BTreeMap<String, usize> = BTreeMap::new();
    let mut by_file_prefix: BTreeMap<String, usize> = BTreeMap::new();

    for d in diags.iter() {
        let (class, name) = compile_clean_diagnostic_histogram_key(d);
        *by_class.entry(class.clone()).or_default() += 1;
        *by_class_name.entry((class, name)).or_default() += 1;
        let file = diagnostic_decl_file(d);
        *by_file.entry(file.clone()).or_default() += 1;
        *by_file_prefix.entry(file_prefix_bucket(&file)).or_default() += 1;
    }

    let total = diags.len();
    println!("HISTOGRAM_STATUS ok");
    println!("HISTOGRAM_TOTAL_HARD {total}");
    println!("HISTOGRAM_ELAPSED_SECS {:.1}", elapsed.as_secs_f64());
    if let Some(rss) = peak_rss_vhwm_bytes() {
        println!("HISTOGRAM_RSS_MIB {}", rss / (1024 * 1024));
    }

    println!("--- CLASS ---");
    for (class, count) in &by_class {
        println!("CLASS\t{class}\t{count}");
    }

    println!("--- FILE_PREFIX ---");
    for (prefix, count) in &by_file_prefix {
        println!("FILE_PREFIX\t{prefix}\t{count}");
    }

    println!("--- FILE (top 50 by count) ---");
    let mut file_ranked: Vec<(String, usize)> = by_file.into_iter().collect();
    file_ranked.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    for (file, count) in file_ranked.iter().take(50) {
        println!("FILE\t{file}\t{count}");
    }

    println!("--- CLASS_NAME (top 50 by count) ---");
    let mut ranked: Vec<((String, String), usize)> = by_class_name.into_iter().collect();
    ranked.sort_by(|a, b| {
        b.1.cmp(&a.1)
            .then_with(|| a.0 .0.cmp(&b.0 .0))
            .then_with(|| a.0 .1.cmp(&b.0 .1))
    });
    for ((class, name), count) in ranked.iter().take(50) {
        println!("NAME\t{class}\t{name}\t{count}");
    }

    if std::env::var("COMPILE_CLEAN_HISTOGRAM_DUMP_ALL")
        .map(|v| v == "1")
        .unwrap_or(false)
    {
        println!("--- ALL CLASS_NAME ---");
        for ((class, name), count) in &ranked {
            println!("NAME\t{class}\t{name}\t{count}");
        }
    }

    if std::env::var("COMPILE_CLEAN_HISTOGRAM_DUMP_SITES")
        .map(|v| v == "1")
        .unwrap_or(false)
    {
        println!("--- SITES ---");
        for d in diags.iter() {
            let (class, name) = compile_clean_diagnostic_histogram_key(d);
            let span = diagnostic_to_span(d.diagnostic.clone());
            println!(
                "SITE\t{class}\t{name}\t{}\t{}\t{}",
                diagnostic_decl_file(d),
                span.start,
                span.end
            );
        }
    }

    if std::env::var("COMPILE_CLEAN_HISTOGRAM_DUMP_MESSAGES")
        .map(|v| !v.is_empty())
        .unwrap_or(false)
    {
        let filt = std::env::var("COMPILE_CLEAN_HISTOGRAM_DUMP_MESSAGES").unwrap_or_default();
        println!("--- MESSAGES ---");
        for d in diags.iter() {
            let file = diagnostic_decl_file(d);
            if filt == "1" || file.contains(&filt) {
                println!("MSG\t{file}\t{:?}", d.diagnostic);
            }
        }
    }

    // Residue probes: fold + type-mismatch pair (namespace migration lane).
    let fold_count = count_message_substr(&diags, "function 'fold' not found in scope");
    let iec_count = count_type_mismatch_name(&diags, "integer_exact_contract");
    let ghp_count = count_type_mismatch_name(&diags, "gunbhub_hostile_page");
    println!("--- RESIDUE_PROBES ---");
    println!("PROBE\tfold_not_found_in_scope\t{fold_count}");
    println!("PROBE\ttype_mismatch_integer_exact_contract\t{iec_count}");
    println!("PROBE\ttype_mismatch_gunbhub_hostile_page\t{ghp_count}");

    ExitCode::from(if total == 0 { 0 } else { 1 })
}

fn diagnostic_decl_file(d: &Rc<ErrorNode>) -> String {
    let raw = diagnostic_to_span(d.diagnostic.clone()).file.clone();
    normalize_repo_relative_path(&raw)
}

fn normalize_repo_relative_path(path: &str) -> String {
    let p = path.replace('\\', "/");
    if let Ok(root) = workspace_root().canonicalize() {
        if let Ok(abs) = std::path::Path::new(&p).canonicalize() {
            if let Ok(rel) = abs.strip_prefix(&root) {
                return rel.to_string_lossy().replace('\\', "/");
            }
        }
    }
    p
}

fn file_prefix_bucket(file: &str) -> String {
    let p = file.replace('\\', "/");
    if p.contains("dag/test/") {
        return "dag/test".to_string();
    }
    if p.contains("src/v2/std/") {
        return "src/v2/std".to_string();
    }
    if p.contains("dag/") {
        return "dag/other".to_string();
    }
    if p.contains("src/v2/") {
        return "src/v2/other".to_string();
    }
    "other".to_string()
}

fn count_message_substr(diags: &im::Vector<Rc<ErrorNode>>, needle: &str) -> usize {
    use v1_compiler::v1_std_core::{diagnostic_to_message, CompilerDiagnostic};
    diags
        .iter()
        .filter(|d| {
            matches!(
                d.diagnostic.as_ref(),
                CompilerDiagnostic::InternalError { .. }
            ) && diagnostic_to_message(d.diagnostic.clone()).contains(needle)
        })
        .count()
}

fn count_type_mismatch_name(diags: &im::Vector<Rc<ErrorNode>>, name: &str) -> usize {
    use v1_compiler::v1_std_core::CompilerDiagnostic;
    diags
        .iter()
        .filter(|d| match d.diagnostic.as_ref() {
            CompilerDiagnostic::TypeMismatch { expected, got, .. } => {
                expected.contains(name) || got.contains(name)
            }
            _ => false,
        })
        .count()
}
