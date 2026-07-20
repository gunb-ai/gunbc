//! Interrogate the REAL corpus census. The residue is neither coverage nor cascade
//! (both refuted by probe), so ask the census what it actually thinks of the names
//! that red: are they ABSENT (never censused) or AMBIGUOUS (homonym → correct refusal)?
//! Those have opposite remedies — widen the census vs qualify the reference.
//!
//! SEED-RETAINED namespace homonym triage probe (DESIGN §7): diagnostic histogram/roster
//! tests plus `tier2_namespace_homonym_invariants` — fail-closed gates on the tier-2
//! triage invariants (not declaration counts). Dissolve-on: namespace lane owns roster
//! sizing in a single authority and these gates move there — delete this module.

use std::rc::Rc;

use v1_compiler::cli_run::{bare_ref_reachability_for_name, BareRefReachability};
use v1_compiler::v1_compiler_compile::{front_end_sources, SourceFile};
use v1_compiler::v1_compiler_infer::build_symbol_index_census;
use v1_compiler::v1_compiler_infer_env::GlobalBareLookupState;

fn workspace_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("workspace root")
}

fn floor_source_roots(ws: &std::path::Path) -> Vec<String> {
    vec![
        ws.join("dag").to_string_lossy().into_owned(),
        ws.join("src/v2").to_string_lossy().into_owned(),
    ]
}

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
                let rel = p
                    .strip_prefix(root)
                    .unwrap_or(&p)
                    .to_string_lossy()
                    .to_string();
                out.push(Rc::new(SourceFile {
                    path: rel,
                    content: c,
                }));
            }
        }
    }
}

fn load_floor_census(
    ws: &std::path::Path,
) -> Rc<im::HashMap<String, Rc<GlobalBareLookupState>>> {
    let mut sources: Vec<Rc<SourceFile>> = Vec::new();
    collect(ws, &ws.join("dag"), &mut sources);
    collect(ws, &ws.join("src/v2"), &mut sources);
    assert!(!sources.is_empty(), "no dag sources found");
    let frontend = front_end_sources(Rc::new(sources.into_iter().collect::<im::Vector<_>>()));
    let graph = frontend.graph.as_ref().expect("graph");
    let source_indices = frontend
        .newline_indices
        .iter()
        .cloned()
        .fold(im::HashMap::new(), |acc, si| {
            acc.update(si.file.clone(), si)
        });
    build_symbol_index_census(graph.modules.clone(), Rc::new(source_indices))
        .global_bare
        .clone()
}

enum CensusInvariantDisposition {
    Unique,
    Ambiguous,
    Absent,
}

fn census_invariant_disposition(
    census: &im::HashMap<String, Rc<GlobalBareLookupState>>,
    name: &str,
) -> CensusInvariantDisposition {
    match census.get(name).map(|s| &**s) {
        None => CensusInvariantDisposition::Absent,
        Some(GlobalBareLookupState::GlobalBareAmbiguousBinding) => {
            CensusInvariantDisposition::Ambiguous
        }
        Some(GlobalBareLookupState::GlobalBareUniqueBinding { .. }) => {
            CensusInvariantDisposition::Unique
        }
    }
}

enum CensusAmbiguityLeg {
    /// Name is in census with exactly one binding shape — ambiguity leg evaluable.
    KnownUnique,
    /// Name is in census as a homonym — red.
    KnownAmbiguous,
    /// Name is not in census — ambiguity leg refused (not "non-ambiguous").
    RefusedAbsent,
}

fn census_ambiguity_leg(
    census: &im::HashMap<String, Rc<GlobalBareLookupState>>,
    name: &str,
) -> CensusAmbiguityLeg {
    match census_invariant_disposition(census, name) {
        CensusInvariantDisposition::Absent => CensusAmbiguityLeg::RefusedAbsent,
        CensusInvariantDisposition::Ambiguous => CensusAmbiguityLeg::KnownAmbiguous,
        CensusInvariantDisposition::Unique => CensusAmbiguityLeg::KnownUnique,
    }
}

fn emit_census_ambiguity_leg_refusal(
    census: &im::HashMap<String, Rc<GlobalBareLookupState>>,
    name: &str,
) {
    let key_count = census.len();
    eprintln!(
        "[tier2-census] REFUSED {name}: not in global_bare census ({key_count} keys); \
         ambiguity leg cannot be evaluated; census shape-coverage gap, precedent fold"
    );
}

/// Census ambiguity leg: UNIQUE proceeds, AMBIGUOUS reds, ABSENT refuses with a counted
/// diagnostic (reachability leg still runs separately). Returns true when refused.
fn assert_census_ambiguity_leg_or_reachability_only(
    census: &im::HashMap<String, Rc<GlobalBareLookupState>>,
    name: &str,
) -> bool {
    match census_ambiguity_leg(census, name) {
        CensusAmbiguityLeg::RefusedAbsent => {
            emit_census_ambiguity_leg_refusal(census, name);
            true
        }
        CensusAmbiguityLeg::KnownAmbiguous => panic!(
            "{name}: global bare census is AMBIGUOUS — this was construction protocol / \
             subtree-local resolution; a new homonym authority now exists; triage before consolidating"
        ),
        CensusAmbiguityLeg::KnownUnique => false,
    }
}

fn assert_zero_ambiguous_bare_sites(name: &str, stats: BareRefReachability) {
    assert_eq!(
        stats.ambiguous_sites, 0,
        "{name}: tier-2 triage established construction protocol with 0 reachable ambiguous bare \
         refs (nearest-wins ties); found {} AmbiguousBare site(s) — a cross-module bare ref now \
         needs qualification; triage before consolidating",
        stats.ambiguous_sites
    );
}

fn assert_zero_cross_subtree_bare_sites(name: &str, stats: BareRefReachability) {
    assert_eq!(
        stats.cross_subtree_unique_sites, 0,
        "{name}: tier-2 triage established 0 cross-subtree bare refs (nearest-wins resolves \
         within subtree); found {} cross-subtree site(s) — triage before consolidating or qualifying",
        stats.cross_subtree_unique_sites
    );
}

/// Census disposition snapshot — dissolve trigger: asserts tier-2 names are ABSENT from the
/// global bare census today (shape gap: entry-grain data stamps, fn bodies). When census
/// coverage widens (#6640) and a name becomes UNIQUE, this test reds to activate the leg.
#[test]
fn tier2_census_disposition_snapshot() {
    let census = load_floor_census(&workspace_root());
    let names = [
        "live_tree_disposition",
        "extdeps_external_authority_anchor",
        "emit",
    ];
    let mut refused = 0usize;
    for name in names {
        let leg = census_ambiguity_leg(&census, name);
        assert!(
            matches!(leg, CensusAmbiguityLeg::RefusedAbsent),
            "{name}: expected ABSENT from global bare census ({} keys) — census shape-coverage \
             gap; when coverage lands and name becomes UNIQUE, update this dissolve trigger",
            census.len()
        );
        emit_census_ambiguity_leg_refusal(&census, name);
        refused += 1;
    }
    eprintln!(
        "[tier2-census] census leg refused {refused}/{} names",
        names.len()
    );
}

/// Tier-2 triage proved ConstructionProtocolNoAction on the two highest-rank roster ghosts;
/// consolidating either would destroy information. These gates encode that finding durably.
#[test]
fn tier2_namespace_homonym_invariants() {
    let ws = workspace_root();
    let roots = floor_source_roots(&ws);
    let census = load_floor_census(&ws);

    let tier2_names = [
        "live_tree_disposition",
        "extdeps_external_authority_anchor",
        "emit",
    ];
    let mut census_refused = 0usize;
    for name in tier2_names {
        if assert_census_ambiguity_leg_or_reachability_only(&census, name) {
            census_refused += 1;
        }
    }
    eprintln!(
        "[tier2-census] census leg refused {census_refused}/{} names",
        tier2_names.len()
    );

    for name in ["live_tree_disposition", "extdeps_external_authority_anchor"] {
        let stats = bare_ref_reachability_for_name(&roots, &roots, &[], name);
        assert_zero_ambiguous_bare_sites(name, stats);
    }

    let emit_stats = bare_ref_reachability_for_name(&roots, &roots, &[], "emit");
    assert_zero_cross_subtree_bare_sites("emit", emit_stats);
}

#[test]
fn corpus_census_state_of_failing_names() {
    let ws = workspace_root();
    let mut sources: Vec<Rc<SourceFile>> = Vec::new();
    collect(&ws, &ws.join("dag"), &mut sources);
    eprintln!(
        "[corpus] {} .dag sources from {}",
        sources.len(),
        ws.display()
    );
    assert!(!sources.is_empty(), "no dag sources found");

    let frontend = front_end_sources(Rc::new(sources.into_iter().collect::<im::Vector<_>>()));
    let graph = frontend.graph.as_ref().expect("graph");
    let source_indices = frontend
        .newline_indices
        .iter()
        .cloned()
        .fold(im::HashMap::new(), |acc, si| {
            acc.update(si.file.clone(), si)
        });
    let census = build_symbol_index_census(graph.modules.clone(), Rc::new(source_indices))
        .global_bare
        .clone();

    eprintln!("[corpus] census keys = {}", census.len());
    eprintln!("[corpus] modules     = {}", graph.modules.len());

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
            Some(GlobalBareLookupState::GlobalBareUniqueBinding { .. }) => {
                "UNIQUE (binds)".to_string()
            }
            Some(GlobalBareLookupState::GlobalBareAmbiguousBinding) => {
                "AMBIGUOUS (refuses — homonym)".to_string()
            }
        };
        eprintln!("[census] {name:30} -> {state}");
    }

    let mut unique = 0usize;
    let mut ambiguous = 0usize;
    for (_k, v) in census.iter() {
        match &**v {
            GlobalBareLookupState::GlobalBareUniqueBinding { .. } => unique += 1,
            GlobalBareLookupState::GlobalBareAmbiguousBinding => ambiguous += 1,
        }
    }
    eprintln!(
        "[census] UNIQUE={unique} AMBIGUOUS={ambiguous} total={}",
        unique + ambiguous
    );
}

/// Emit the AMBIGUOUS roster — the forced-qualification worklist (§8 residue).
#[test]
fn corpus_ambiguous_roster() {
    let ws = workspace_root();
    let mut sources: Vec<Rc<SourceFile>> = Vec::new();
    collect(&ws, &ws.join("dag"), &mut sources);
    collect(&ws, &ws.join("src/v2"), &mut sources);
    eprintln!(
        "[roster] {} sources (dag/ + src/v2 — CI floor source-roots)",
        sources.len()
    );

    let frontend = front_end_sources(Rc::new(sources.into_iter().collect::<im::Vector<_>>()));
    let graph = frontend.graph.as_ref().expect("graph");
    let source_indices = frontend
        .newline_indices
        .iter()
        .cloned()
        .fold(im::HashMap::new(), |acc, si| {
            acc.update(si.file.clone(), si)
        });
    let census = build_symbol_index_census(graph.modules.clone(), Rc::new(source_indices))
        .global_bare
        .clone();

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
