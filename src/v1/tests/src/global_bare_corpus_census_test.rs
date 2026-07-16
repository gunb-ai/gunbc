//! Interrogate the REAL corpus census. The residue is neither coverage nor cascade
//! (both refuted by probe), so ask the census what it actually thinks of the names
//! that red: are they ABSENT (never censused) or AMBIGUOUS (homonym → correct refusal)?
//! Those have opposite remedies — widen the census vs qualify the reference.
//!
//! Diagnostic instrument only — these tests print histograms/rosters via eprintln/println
//! and do not assert census shape beyond `sources.is_empty()`. They are not enrolled as
//! regression gates; a census logic regression would pass silently until a human reads output.

use std::rc::Rc;

use v1_compiler::v1_compiler_compile::{front_end_sources, SourceFile};
use v1_compiler::v1_compiler_infer::build_global_bare_census;
use v1_compiler::v1_compiler_infer_env::GlobalBareLookupState;

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

    // The exact names still reding on #6640 batch-1 (run 29450667128).
    for name in [
        "PullRequest",
        "ExitSuccess",
        "gunbhub_hostile_page",
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
