#![allow(clippy::disallowed_macros)]

//! Scratch analysis (not a CI consumer): join #6967 silent-pick sites against
//! the resolution-divergence census's agree/diverge/containment_ambiguous
//! cross-check, keyed by (module, name), to split benign (unique-on-chain)
//! from genuine (§13 fail-open) sites.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use v1_compiler::cli_run::resolution_divergence_census_live_closure_scoped;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("workspace root")
}

fn main() {
    let ws = workspace_root();
    let census = resolution_divergence_census_live_closure_scoped(&ws)
        .expect("closure-scoped census resolve");

    let mut genuine_keys: HashSet<(String, String)> = HashSet::new();
    for row in census.diverge_rows.iter() {
        genuine_keys.insert((row.calling_module.clone(), row.callee.clone()));
    }
    for row in census.containment_ambiguous_rows.iter() {
        genuine_keys.insert((row.calling_module.clone(), row.callee.clone()));
    }

    // Distinct silent-pick sites across all three classes, keyed (module, name),
    // tagged with which class(es) fired and a representative row for detail.
    let mut sites: HashMap<(String, String), (Vec<&'static str>, usize)> = HashMap::new();
    for r in census.silent_pick_global_bare_lcp_rows.iter() {
        let e = sites
            .entry((r.env_module_path.clone(), r.name.clone()))
            .or_insert((Vec::new(), 0));
        if !e.0.contains(&"bare_lcp") {
            e.0.push("bare_lcp");
        }
        e.1 += 1;
    }
    for r in census.silent_pick_global_bare_lcp_tie_rows.iter() {
        let e = sites
            .entry((r.env_module_path.clone(), r.name.clone()))
            .or_insert((Vec::new(), 0));
        if !e.0.contains(&"bare_lcp_tie") {
            e.0.push("bare_lcp_tie");
        }
        e.1 += 1;
    }
    for r in census.silent_pick_fn_parent_first_hit_rows.iter() {
        let e = sites
            .entry((r.env_module_path.clone(), r.name.clone()))
            .or_insert((Vec::new(), 0));
        if !e.0.contains(&"fn_parent_first_hit") {
            e.0.push("fn_parent_first_hit");
        }
        e.1 += 1;
    }

    let total_distinct = sites.len();
    let mut genuine: Vec<(String, String, Vec<&'static str>, usize)> = Vec::new();
    let mut benign_n = 0usize;
    for ((module, name), (classes, occ)) in sites.iter() {
        if genuine_keys.contains(&(module.clone(), name.clone())) {
            genuine.push((module.clone(), name.clone(), classes.clone(), *occ));
        } else {
            benign_n += 1;
        }
    }
    genuine.sort();

    println!("=== silent-pick x resolution-divergence cross-check ===");
    println!("distinct silent-pick sites (module,name): {total_distinct}");
    println!("benign (not in diverge/containment_ambiguous): {benign_n}");
    println!("genuine (diverge or containment_ambiguous): {}", genuine.len());
    println!();
    println!("--- genuine rows (module, name, silent-pick classes, occurrences) ---");
    for (module, name, classes, occ) in genuine.iter() {
        println!("  module={module} name={name} classes={classes:?} occurrences={occ}");
    }

    println!();
    println!("--- for reference: which arm (diverge vs containment_ambiguous) per genuine site ---");
    for (module, name, _, _) in genuine.iter() {
        let in_diverge = census
            .diverge_rows
            .iter()
            .any(|r| &r.calling_module == module && &r.callee == name);
        let in_ambig = census
            .containment_ambiguous_rows
            .iter()
            .any(|r| &r.calling_module == module && &r.callee == name);
        println!(
            "  module={module} name={name} diverge={in_diverge} containment_ambiguous={in_ambig}"
        );
    }
}
