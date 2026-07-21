#![allow(clippy::disallowed_macros)]

//! Ad hoc analysis over the gate's single-authority join
//! (`resolution_divergence_silent_pick_genuine_rows`): reports the
//! benign/genuine split of #6967 silent-pick sites for human triage, on top of
//! the same closure-scoped census the gate itself consumes. Not a CI consumer —
//! `resolution_divergence_census --closure-scoped` is the gate's own binary.

use std::collections::HashMap;
use std::path::PathBuf;

use v1_compiler::cli_run::{
    resolution_divergence_census_live_closure_scoped,
    resolution_divergence_silent_pick_genuine_rows,
};

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

    // Distinct silent-pick sites across all three classes, keyed (module, name).
    let mut sites: HashMap<(String, String), usize> = HashMap::new();
    for r in census.silent_pick_global_bare_lcp_rows.iter() {
        *sites
            .entry((r.env_module_path.clone(), r.name.clone()))
            .or_insert(0) += 1;
    }
    for r in census.silent_pick_global_bare_lcp_tie_rows.iter() {
        *sites
            .entry((r.env_module_path.clone(), r.name.clone()))
            .or_insert(0) += 1;
    }
    for r in census.silent_pick_fn_parent_first_hit_rows.iter() {
        *sites
            .entry((r.env_module_path.clone(), r.name.clone()))
            .or_insert(0) += 1;
    }
    let total_distinct = sites.len();

    let genuine = resolution_divergence_silent_pick_genuine_rows(&census);
    let mut genuine_keys: std::collections::HashSet<(String, String)> =
        std::collections::HashSet::new();
    for row in &genuine {
        genuine_keys.insert((row.module.clone(), row.name.clone()));
    }
    let genuine_distinct = genuine_keys.len();
    let benign_n = total_distinct - genuine_distinct;

    println!("=== silent-pick x resolution-divergence cross-check (shared join) ===");
    println!("distinct silent-pick sites (module,name): {total_distinct}");
    println!("benign (not in diverge/containment_ambiguous): {benign_n}");
    println!("genuine (diverge or containment_ambiguous): {genuine_distinct}");
    println!();
    println!("--- genuine rows ---");
    for row in &genuine {
        println!(
            "  class={} module={} name={} {}",
            row.class, row.module, row.name, row.detail
        );
    }
}
