//! Census exclude-closure derivation (fierce-heron-512 / bright-heron-200).
//!
//! Pattern rows (`whole_tree_resolve_exclusion_substrings`) plus fixed-point
//! transitive-importer closure of strict-resolve failures form the probe authority.
//! Historical pin `docs/probes/census_extra_excludes.txt` is a drift witness only.
//!
//! Dissolve-on: strict whole-tree walk greens without host fixed-point closure
//! (namespace terminal + FilePath grounding); derived probe exclusion moves to a
//! `.dag` authority row with this module as thin projection only — deletes with the
//! historical 83-row drift pin and `witness_exclusion_single_authority_reconciliation_note`
//! parallel-list scaffold in `ci_layer_roots.dag`.
//!
//! Cascade semantics (sunny-wolf-225 resolution):
//! - (i) Silent live-importer loss → typed refusal.
//! - (ii) Derived closure exclusion of live importers → legitimate, reported in receipt.

use std::collections::{BTreeSet, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use im::HashMap;

use crate::v1_interpreter::ExecutionMode;

pub const PINNED_ORACLE_EXCLUDES_REL: &str = "docs/probes/census_extra_excludes.txt";
pub const PINNED_COORDINATION_REL: &str = "docs/probes/still-hawk-row-coordination.txt";
pub const PINNED_ORACLE_SEEDS_REL: &str = "docs/probes/census_extra_excludes_seeds.txt";
pub const DERIVATION_ALGO_VERSION: &str = "census_exclude_derive_v1";

const MAX_CONVERGENCE_ROUNDS: u32 = 120;

/// Live pipeline module excluded via derived closure (reported, not refused).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct LiveImporterExclusion {
    pub module_path: String,
    pub seed_chain: String,
    pub round: u32,
}

/// Derived exclude closure + provenance receipt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DerivedExcludeClosure {
    pub module_paths: BTreeSet<String>,
    pub live_importers_excluded: Vec<LiveImporterExclusion>,
    pub convergence_rounds: u32,
    pub memo_content_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetSymmetryDiff {
    pub only_left: Vec<String>,
    pub only_right: Vec<String>,
}

impl SetSymmetryDiff {
    pub fn is_empty(&self) -> bool {
        self.only_left.is_empty() && self.only_right.is_empty()
    }
}

static DERIVED_CLOSURE_MEMO: OnceLock<Result<DerivedExcludeClosure, String>> = OnceLock::new();

/// Load a git-readable module-path list (one path per line, `#` comments skipped).
pub fn load_module_path_list(rel: &str, workspace_root: &Path) -> Result<BTreeSet<String>, String> {
    let path = workspace_root.join(rel);
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("module path list: failed to read {}: {e}", path.display()))?;
    let mut paths = BTreeSet::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        paths.insert(line.to_string());
    }
    Ok(paths)
}

/// Historical 83-row pin (stern-newt @ eaf13cd3c0) — drift witness, not authority.
pub fn load_pinned_oracle_module_paths(workspace_root: &Path) -> Result<BTreeSet<String>, String> {
    let paths = load_module_path_list(PINNED_ORACLE_EXCLUDES_REL, workspace_root)?;
    if paths.is_empty() {
        return Err(format!(
            "pinned oracle: {PINNED_ORACLE_EXCLUDES_REL} contains no module paths (fail-closed)"
        ));
    }
    Ok(paths)
}

/// Set equality witness helper for historical drift pin.
pub fn symmetric_module_path_diff(
    left: &BTreeSet<String>,
    right: &BTreeSet<String>,
) -> SetSymmetryDiff {
    let only_left: Vec<String> = left.difference(right).cloned().collect();
    let only_right: Vec<String> = right.difference(left).cloned().collect();
    SetSymmetryDiff {
        only_left,
        only_right,
    }
}

fn normalize_repo_path(path: &str) -> String {
    path.replace('\\', "/")
}

/// Interim host scaffold: extract failing module paths from strict-resolve's
/// human-readable `Err(String)` surface. Stuck-round refusal is fail-closed if
/// formatting drifts. Dissolves with the module header (structured failure facts).
fn parse_strict_resolve_failure_paths(err: &str) -> BTreeSet<String> {
    let mut paths = BTreeSet::new();
    for line in err.lines() {
        if let Some((path, _)) = line.split_once(':') {
            if path.ends_with(".dag") {
                paths.insert(normalize_repo_path(path));
            }
        }
    }
    paths
}

fn build_reverse_importer_adjacency(
    forward: &HashMap<String, Vec<String>>,
) -> HashMap<String, Vec<String>> {
    let mut reverse: HashMap<String, Vec<String>> = HashMap::new();
    for (importer, imports) in forward {
        for imported in imports {
            let entry = reverse.entry(imported.clone()).or_default();
            if !entry.iter().any(|p| p == importer) {
                entry.push(importer.clone());
            }
        }
    }
    reverse
}

fn transitive_importers_of(
    module_path: &str,
    reverse: &HashMap<String, Vec<String>>,
) -> BTreeSet<String> {
    let mut result = BTreeSet::new();
    let mut queue = VecDeque::from([module_path.to_string()]);
    while let Some(current) = queue.pop_front() {
        let Some(importers) = reverse.get(&current) else {
            continue;
        };
        for importer in importers {
            if result.insert(importer.clone()) {
                queue.push_back(importer.clone());
            }
        }
    }
    result
}

fn derivation_cache_key(workspace_root: &Path, source_roots: &[String]) -> String {
    let pattern = super::whole_tree_resolve_exclusion_substrings();
    let pattern_hash = crate::v1_rt::bytes_identity_hash(pattern.join("\n").as_bytes());
    let index = super::build_module_path_index(source_roots);
    let mut module_paths: Vec<String> = index.keys().cloned().collect();
    module_paths.sort();
    let corpus_hash = crate::v1_rt::bytes_identity_hash(module_paths.join("\n").as_bytes());
    let roots_hash = crate::v1_rt::bytes_identity_hash(
        format!(
            "{DERIVATION_ALGO_VERSION}\n{workspace_root:?}\n{}",
            source_roots.join("\n")
        )
        .as_bytes(),
    );
    crate::v1_rt::hash_combine(
        crate::v1_rt::hash_combine(pattern_hash, corpus_hash),
        roots_hash,
    )
}

fn memo_hash_for_closure(closure: &DerivedExcludeClosure, cache_key: &str) -> String {
    let joined = closure
        .module_paths
        .iter()
        .cloned()
        .collect::<Vec<_>>()
        .join("\n");
    crate::v1_rt::hash_combine(
        cache_key.to_string(),
        crate::v1_rt::bytes_identity_hash(joined.as_bytes()),
    )
}

fn exclusion_substrings_with_derived(derived_module_paths: &BTreeSet<String>) -> Vec<String> {
    let mut exclude = super::whole_tree_resolve_exclusion_substrings();
    exclude.extend(derived_module_paths.iter().cloned());
    exclude
}

fn module_path_excluded_by_substrings(module_path: &str, exclude_substrings: &[String]) -> bool {
    let norm = normalize_repo_path(module_path);
    exclude_substrings
        .iter()
        .any(|sub| norm.contains(sub.as_str()))
}

/// Fixed-point derivation: pattern authority ∪ module-path closure of strict-resolve
/// failures plus transitive-importer closure each round.
pub fn derive_census_exclude_closure(
    workspace_root: &Path,
    source_roots: &[String],
) -> Result<DerivedExcludeClosure, String> {
    let pool_roots: Vec<String> = source_roots
        .iter()
        .map(|r| {
            let p = Path::new(r);
            p.strip_prefix(workspace_root)
                .map(|rel| rel.to_string_lossy().replace('\\', "/"))
                .unwrap_or_else(|_| normalize_repo_path(r))
        })
        .collect();
    let facts = super::build_module_graph_facts_live(&pool_roots);
    let reverse_adjacency = build_reverse_importer_adjacency(&facts.adjacency);
    let cache_key = derivation_cache_key(workspace_root, source_roots);

    let mut derived_module_paths = BTreeSet::new();
    let mut live_importers_excluded = Vec::new();
    let mut round = 0u32;

    loop {
        round += 1;
        if round > MAX_CONVERGENCE_ROUNDS {
            return Err(format!(
                "derive_census_exclude_closure: exceeded {MAX_CONVERGENCE_ROUNDS} convergence rounds (fail-closed)"
            ));
        }

        let exclude = exclusion_substrings_with_derived(&derived_module_paths);
        match super::whole_tree_resolved_ctx(source_roots, &exclude, ExecutionMode::Wet) {
            Ok(_) => {
                let mut closure = DerivedExcludeClosure {
                    memo_content_hash: String::new(),
                    module_paths: derived_module_paths,
                    live_importers_excluded,
                    convergence_rounds: round,
                };
                closure.memo_content_hash = memo_hash_for_closure(&closure, &cache_key);
                verify_live_pipeline_exclusions(&closure)?;
                return Ok(closure);
            }
            Err(err) => {
                let failures = parse_strict_resolve_failure_paths(&err);
                if failures.is_empty() {
                    return Err(format!(
                        "derive_census_exclude_closure: strict resolve failed without locatable .dag paths:\n{err}"
                    ));
                }

                let before_len = derived_module_paths.len();
                for failure in &failures {
                    derived_module_paths.insert(failure.clone());
                    for importer in transitive_importers_of(failure, &reverse_adjacency) {
                        if derived_module_paths.insert(importer.clone()) {
                            live_importers_excluded.push(LiveImporterExclusion {
                                module_path: importer,
                                seed_chain: failure.clone(),
                                round,
                            });
                        }
                    }
                }

                if derived_module_paths.len() == before_len {
                    return Err(format!(
                        "derive_census_exclude_closure: stuck — no new excludes at round {round}:\n{err}"
                    ));
                }
            }
        }
    }
}

/// Memoized derived closure for the default witness-layer source roots.
pub fn derived_exclude_closure_memoized() -> Result<DerivedExcludeClosure, String> {
    DERIVED_CLOSURE_MEMO
        .get_or_init(|| {
            let ws = super::workspace_root();
            let roots = super::default_source_roots();
            derive_census_exclude_closure(&ws, &roots)
        })
        .clone()
}

/// Whole-tree probe exclusion authority: pattern rows ∪ derived module-path closure.
pub fn whole_tree_probe_exclusion_substrings() -> Vec<String> {
    let closure = derived_exclude_closure_memoized()
        .expect("whole_tree_probe_exclusion_substrings: derived closure must converge");
    exclusion_substrings_with_derived(&closure.module_paths)
}

/// Live compile-clean pipeline module paths for silent-loss checks.
pub fn live_pipeline_module_paths() -> Vec<String> {
    super::compile_clean_live_pipeline_module_paths()
}

/// (i) Silent-loss detector: live pipeline module excluded without a receipt row.
pub fn refuse_silent_live_importer_loss(
    before: &BTreeSet<String>,
    after: &BTreeSet<String>,
    receipt: &DerivedExcludeClosure,
    live_pipeline_modules: &[String],
) -> Result<(), String> {
    let receipt_paths: BTreeSet<String> = receipt
        .live_importers_excluded
        .iter()
        .map(|row| row.module_path.clone())
        .chain(receipt.module_paths.iter().cloned())
        .collect();

    for module in live_pipeline_modules {
        let norm = normalize_repo_path(module);
        if before.contains(&norm) && !after.contains(&norm) && !receipt_paths.contains(&norm) {
            return Err(format!(
                "refuse_silent_live_importer_loss: live pipeline module '{norm}' \
                 dropped without a matching receipt row (fail-closed)"
            ));
        }
    }
    Ok(())
}

fn verify_live_pipeline_exclusions(receipt: &DerivedExcludeClosure) -> Result<(), String> {
    let live = live_pipeline_module_paths();
    let pattern_only = super::whole_tree_resolve_exclusion_substrings();
    let full_exclude = exclusion_substrings_with_derived(&receipt.module_paths);

    let before: BTreeSet<String> = live
        .iter()
        .map(|p| normalize_repo_path(p))
        .filter(|p| !module_path_excluded_by_substrings(p, &pattern_only))
        .collect();

    let after: BTreeSet<String> = live
        .iter()
        .map(|p| normalize_repo_path(p))
        .filter(|p| !module_path_excluded_by_substrings(p, &full_exclude))
        .collect();

    refuse_silent_live_importer_loss(&before, &after, receipt, &live)
}

pub fn workspace_root_from_manifest_dir(manifest_dir: &Path) -> PathBuf {
    manifest_dir
        .join("../../..")
        .canonicalize()
        .unwrap_or_else(|_| manifest_dir.join("../../.."))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pinned_oracle_loads_eighty_three_paths() {
        let ws = workspace_root_from_manifest_dir(Path::new(env!("CARGO_MANIFEST_DIR")));
        let paths = load_pinned_oracle_module_paths(&ws).expect("pinned oracle paths");
        assert_eq!(
            paths.len(),
            83,
            "docs/probes/census_extra_excludes.txt must enumerate 83 module paths"
        );
        assert!(paths.contains("src/v2/compiler/00_compile.dag"));
        assert!(paths.contains("src/v2/compiler/03_ingest.dag"));
    }

    #[test]
    fn pinned_seeds_are_subset_of_oracle() {
        let ws = workspace_root_from_manifest_dir(Path::new(env!("CARGO_MANIFEST_DIR")));
        let oracle = load_pinned_oracle_module_paths(&ws).expect("oracle");
        let seeds = load_module_path_list(PINNED_ORACLE_SEEDS_REL, &ws).expect("seeds");
        assert_eq!(seeds.len(), 27, "seeds file must enumerate 27 module paths");
        let diff = symmetric_module_path_diff(&seeds, &oracle);
        assert!(
            diff.only_left.is_empty(),
            "seeds must be subset of oracle; only in seeds: {:?}",
            diff.only_left
        );
    }

    #[test]
    fn refuse_silent_live_importer_loss_refuses_unreceipted_drop() {
        let mut before = BTreeSet::new();
        before.insert("src/v2/compiler/04_infer.dag".to_string());
        let after = BTreeSet::new();
        let receipt = DerivedExcludeClosure {
            memo_content_hash: String::new(),
            module_paths: BTreeSet::new(),
            live_importers_excluded: vec![],
            convergence_rounds: 1,
        };
        let live = vec!["src/v2/compiler/04_infer.dag".to_string()];
        assert!(
            refuse_silent_live_importer_loss(&before, &after, &receipt, &live).is_err(),
            "live pipeline module lost without receipt must refuse"
        );
    }

    #[test]
    fn refuse_silent_live_importer_loss_holds_when_receipted() {
        let mut before = BTreeSet::new();
        before.insert("src/v2/compiler/04_infer.dag".to_string());
        let after = BTreeSet::new();
        let mut module_paths = BTreeSet::new();
        module_paths.insert("src/v2/compiler/04_infer.dag".to_string());
        let receipt = DerivedExcludeClosure {
            memo_content_hash: String::new(),
            module_paths,
            live_importers_excluded: vec![],
            convergence_rounds: 1,
        };
        let live = vec!["src/v2/compiler/04_infer.dag".to_string()];
        assert!(
            refuse_silent_live_importer_loss(&before, &after, &receipt, &live).is_ok(),
            "receipted exclusion must not trip silent-loss guard"
        );
    }

    #[test]
    #[ignore = "manual: triggers whole-tree derivation (~minutes)"]
    fn probe_exclusion_extends_pattern_authority() {
        let pattern = super::super::whole_tree_resolve_exclusion_substrings();
        let probe = whole_tree_probe_exclusion_substrings();
        assert!(
            probe.len() >= pattern.len(),
            "probe authority must be a superset of pattern rows"
        );
        for row in pattern {
            assert!(
                probe.iter().any(|p| p == &row),
                "pattern row {row:?} missing from probe authority"
            );
        }
    }

    #[test]
    #[ignore = "manual: whole-tree strict-resolve fixed-point (~minutes); classifies drift vs historical 83-pin"]
    fn derived_closure_drift_report_vs_historical_pin() {
        let ws = workspace_root_from_manifest_dir(Path::new(env!("CARGO_MANIFEST_DIR")));
        let roots = super::super::default_source_roots();
        let derived = derive_census_exclude_closure(&ws, &roots).expect("derive");
        let historical = load_pinned_oracle_module_paths(&ws).expect("historical pin");
        let diff = symmetric_module_path_diff(&derived.module_paths, &historical);
        eprintln!(
            "derived rounds={} module_paths={} live_importer_rows={}",
            derived.convergence_rounds,
            derived.module_paths.len(),
            derived.live_importers_excluded.len()
        );
        if !diff.only_left.is_empty() {
            eprintln!(
                "only in derived ({}): {:?}",
                diff.only_left.len(),
                diff.only_left
            );
        }
        if !diff.only_right.is_empty() {
            eprintln!(
                "only in historical pin ({}): {:?}",
                diff.only_right.len(),
                diff.only_right
            );
        }
        super::super::whole_tree_resolved_ctx(
            &roots,
            &exclusion_substrings_with_derived(&derived.module_paths),
            ExecutionMode::Wet,
        )
        .expect("derived authority must strict-resolve green");
    }
}
