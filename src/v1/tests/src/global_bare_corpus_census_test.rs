//! Interrogate the REAL corpus census. The residue is neither coverage nor cascade
//! (both refuted by probe), so ask the census what it actually thinks of the names
//! that red: are they ABSENT (never censused) or AMBIGUOUS (homonym → correct refusal)?
//! Those have opposite remedies — widen the census vs qualify the reference.

use std::rc::Rc;

use v1_compiler::v1_compiler_compile::{compile_sources, front_end_sources, SourceFile};
use v1_compiler::v1_compiler_infer::build_global_bare_census;
use v1_compiler::v1_compiler_infer_env::GlobalBareLookupState;
use v1_compiler::v1_std_core::is_interpreter_blocking_diagnostic;

fn collect(root: &std::path::Path, dir: &std::path::Path, out: &mut Vec<Rc<SourceFile>>) {
    let mut entries: Vec<_> = match std::fs::read_dir(dir) {
        Ok(e) => e.filter_map(|x| x.ok()).collect(),
        Err(_) => return,
    };
    entries.sort_by_key(|e| e.file_name());
    for e in entries {
        let p = e.path();
        if p.is_dir() {
            collect(root, &p, out);
        } else if p.extension().map(|x| x == "dag").unwrap_or(false) {
            if let Ok(c) = std::fs::read_to_string(&p) {
                let rel = p.strip_prefix(root).unwrap_or(&p).to_string_lossy().to_string();
                out.push(Rc::new(SourceFile { path: rel, content: c }));
            }
        }
    }
}

#[test]
fn corpus_census_state_of_failing_names() {
    let ws = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("workspace root");
    let mut sources: Vec<Rc<SourceFile>> = Vec::new();
    collect(&ws, &ws.join("dag"), &mut sources);
    eprintln!("[corpus] {} .dag sources from {}", sources.len(), ws.display());
    assert!(!sources.is_empty(), "no dag sources found");

    let frontend = front_end_sources(Rc::new(sources.into_iter().collect::<im_rc::Vector<_>>()));
    let graph = frontend.graph.as_ref().expect("graph");
    let source_indices = frontend
        .newline_indices
        .iter()
        .cloned()
        .fold(im_rc::HashMap::new(), |acc, si| acc.update(si.file.clone(), si));
    let census = build_global_bare_census(graph.modules.clone(), Rc::new(source_indices));

    eprintln!("[corpus] census keys = {}", census.len());
    eprintln!("[corpus] modules     = {}", graph.modules.len());

    // Names under investigation on the #6640 namespace strip (proud-gull-205).
    for name in [
        "fold",
        "int_max",
        "bandwidth_count",
        "integer_exact_contract",
        "gunbhub_hostile_page",
        "NumericalContract",
        "MarkupNode",
        "PullRequest",
        "ExitSuccess",
        "Filesystem",
        "exit_code_general_error",
        "ProbeNothing_control",
    ] {
        let state = match census.get(name).map(|s| &**s) {
            None => "ABSENT (never censused)".to_string(),
            Some(GlobalBareLookupState::GlobalBareUniqueBinding { .. }) => "UNIQUE (binds)".to_string(),
            Some(GlobalBareLookupState::GlobalBareAmbiguousBinding) => {
                "AMBIGUOUS (refuses — homonym)".to_string()
            }
        };
        eprintln!("[census] {name:30} -> {state}");
    }

    // Histogram the whole census so the shape is visible, not sampled.
    let mut unique = 0usize;
    let mut ambiguous = 0usize;
    for (_k, v) in census.iter() {
        match &**v {
            GlobalBareLookupState::GlobalBareUniqueBinding { .. } => unique += 1,
            GlobalBareLookupState::GlobalBareAmbiguousBinding => ambiguous += 1,
        }
    }
    eprintln!("[census] UNIQUE={unique} AMBIGUOUS={ambiguous} total={}", unique + ambiguous);
}

/// Emit the AMBIGUOUS roster — the forced-qualification worklist (§8 residue).
/// These are names the census REFUSES because they are declared more than once;
/// no census widening can fix them (widening would make them guess, violating §5).
/// The only remedy is qualifying the reference. This is Wave-0 (c), sized.
#[test]
fn corpus_ambiguous_roster() {
    let ws = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("workspace root");
    let mut sources: Vec<Rc<SourceFile>> = Vec::new();
    collect(&ws, &ws.join("dag"), &mut sources);
    collect(&ws, &ws.join("src/v2"), &mut sources);
    eprintln!("[roster] {} sources (dag/ + src/v2 — CI floor source-roots)", sources.len());

    let frontend = front_end_sources(Rc::new(sources.into_iter().collect::<im_rc::Vector<_>>()));
    let graph = frontend.graph.as_ref().expect("graph");
    let source_indices = frontend
        .newline_indices
        .iter()
        .cloned()
        .fold(im_rc::HashMap::new(), |acc, si| acc.update(si.file.clone(), si));
    let census = build_global_bare_census(graph.modules.clone(), Rc::new(source_indices));

    let mut ambiguous: Vec<String> = census
        .iter()
        .filter(|(_k, v)| matches!(&***v, GlobalBareLookupState::GlobalBareAmbiguousBinding))
        .map(|(k, _v)| k.to_string())
        .collect();
    ambiguous.sort();
    eprintln!("[roster] AMBIGUOUS count = {}", ambiguous.len());
    for n in &ambiguous {
        println!("AMBIGUOUS\t{n}");
    }
    let unique = census.len() - ambiguous.len();
    eprintln!(
        "[roster] UNIQUE={} AMBIGUOUS={} total={} ({:.2}% unique)",
        unique,
        ambiguous.len(),
        census.len(),
        (unique as f64 / census.len() as f64) * 100.0
    );
}

// --- Compile probes for the two #6640 residues (proud-gull-205) ---

fn probe_src(path: &str, content: &str) -> Rc<SourceFile> {
    Rc::new(SourceFile {
        path: path.to_string(),
        content: content.to_string(),
    })
}

fn hard_diags(sources: Vec<Rc<SourceFile>>) -> Vec<String> {
    let result = compile_sources(
        Rc::new(sources.into()),
        v1_compiler::v1_compiler_artifact::RenderTarget::Rust,
    );
    result
        .diagnostics
        .iter()
        .filter(|d| is_interpreter_blocking_diagnostic(d.diagnostic.clone()))
        .map(|d| v1_compiler::v1_std_core::diagnostic_to_message(d.diagnostic.clone()))
        .collect()
}

/// fold is ABSENT from global_bare census; bare `fold(...)` must resolve via the
/// builtin/method-bridge path, never scope lookup.
#[test]
fn probe_bare_fold_call_in_single_module() {
    let src = r#"module probe.fold

fn uses_fold(xs: List<Int>) -> Int {
  fold(xs, init: 0, f: fn(acc, _) { acc })
}
"#;
    let d = hard_diags(vec![probe_src("dag/probe_fold.dag", src)]);
    eprintln!("[fold-probe] hard diags = {d:?}");
    assert!(
        !d.iter().any(|m| m.contains("function 'fold' not found in scope")),
        "bare fold must not fail scope lookup: {d:?}"
    );
}

/// Same shape as dag/tools/rust_stage0_gates.dag: bare fold over a list literal.
#[test]
fn probe_bare_fold_over_list_literal() {
    let src = r#"module probe.fold_lit

fn any_suffix(paths: List<String>) -> Bool {
  fold([".rs"], false, fn(acc, suffix) { acc })
}
"#;
    let d = hard_diags(vec![probe_src("dag/probe_fold_lit.dag", src)]);
    eprintln!("[fold-lit-probe] hard diags = {d:?}");
    assert!(
        !d.iter().any(|m| m.contains("function 'fold' not found in scope")),
        "list-literal fold must not fail scope lookup: {d:?}"
    );
}

/// Corpus shape: nullary fn returning a record type, bare-called cross-module (no import).
#[test]
fn probe_bare_nullary_fn_returning_record_resolves() {
    let definer = r#"module std.numerical_contract

type NumericalContract
  = IntegerExact { precision: Int }

fn integer_exact_contract() -> NumericalContract {
  IntegerExact { precision: 32 }
}
"#;
    let user = r#"module test.use

fn witness() -> Bool {
  match integer_exact_contract() {
    IntegerExact { precision: p } => p == 32
  }
}
"#;
    let d = hard_diags(vec![
        probe_src("dag/std/numerical_contract.dag", definer),
        probe_src("dag/test/use.dag", user),
    ]);
    eprintln!("[record-probe] hard diags = {d:?}");
    assert!(
        d.is_empty(),
        "nullary record-return fn bare-call should resolve: {d:?}"
    );
}

/// Corpus shape: nullary fn returning a named record type, bare-called cross-module.
#[test]
fn probe_bare_nullary_markup_fn_resolves() {
    let definer = r#"module gunbc.gunbhub_serve

type HostilePage = ElementNode { tag: String, text: String }

fn gunbhub_hostile_page() -> HostilePage {
  ElementNode { tag: "a", text: "x" }
}
"#;
    let user = r#"module test.react

fn witness() -> HostilePage {
  gunbhub_hostile_page()
}
"#;
    let d = hard_diags(vec![
        probe_src("dag/gunbc/gunbhub_serve.dag", definer),
        probe_src("dag/test/react.dag", user),
    ]);
    eprintln!("[markup-probe] hard diags = {d:?}");
    assert!(
        !d.iter().any(|m| m.contains("type mismatch")),
        "record-return nullary fn bare-call should not type-mismatch: {d:?}"
    );
}

/// CI floor path: resolve real corpus entries with cli_run census overlay (not
/// `compile_sources` alone). Receipt for whether fold / type-mismatch reproduce.
#[test]
fn cli_run_resolve_residue_entry_receipt() {
    use v1_compiler::cli_run::{build_multi_entry_index, resolve_entry_with_index};
    use v1_compiler::v1_std_core::diagnostic_to_message;

    let ws = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("workspace root");
    std::env::set_current_dir(&ws).expect("chdir workspace");
    let roots = vec![
        ws.join("dag").to_string_lossy().into_owned(),
        ws.join("src/v2").to_string_lossy().into_owned(),
    ];
    let index = build_multi_entry_index(&roots);

    let mut fold_not_found = 0usize;
    let mut type_mismatch_residue = 0usize;
    let mut total_hard = 0usize;

    for entry in [
        "dag/tools/rust_stage0_gates.dag",
        "dag/test/claim/react_jsx_emit_test.dag",
        "dag/test/claim/accelerator_demo_model_witness_test.dag",
    ] {
        let hard: Vec<String> = match resolve_entry_with_index(&index, entry) {
            Ok((graph, _)) => graph
                .diagnostics
                .iter()
                .filter(|d| is_interpreter_blocking_diagnostic(d.diagnostic.clone()))
                .map(|d| diagnostic_to_message(d.diagnostic.clone()))
                .collect(),
            Err(msg) => vec![msg],
        };
        total_hard += hard.len();
        let fold_hits: Vec<_> = hard
            .iter()
            .filter(|m| m.contains("function 'fold' not found in scope"))
            .cloned()
            .collect();
        let tm_hits: Vec<_> = hard
            .iter()
            .filter(|m| {
                (m.contains("integer_exact_contract") || m.contains("gunbhub_hostile_page"))
                    && m.contains("type mismatch")
            })
            .cloned()
            .collect();
        fold_not_found += fold_hits.len();
        type_mismatch_residue += tm_hits.len();
        eprintln!(
            "[cli-resolve] {entry}: hard={} fold_not_found={} type_mismatch_residue={}",
            hard.len(),
            fold_hits.len(),
            tm_hits.len()
        );
        for m in &hard {
            eprintln!("  {m}");
        }
    }

    eprintln!(
        "[cli-resolve] TOTAL hard={total_hard} fold_not_found={fold_not_found} type_mismatch_residue={type_mismatch_residue}"
    );
    assert_eq!(
        fold_not_found, 0,
        "real entries must not hit fold scope-lookup on cli_run path"
    );
    assert_eq!(
        type_mismatch_residue, 0,
        "real entries must not hit integer_exact_contract/gunbhub_hostile_page type-mismatch on cli_run path"
    );
}
