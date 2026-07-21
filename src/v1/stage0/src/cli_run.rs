use im::HashMap;
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock, RwLock};

use crate::coproduct_reflection::{decl_facts_corpus_walk, DeclFactRaw};
use crate::module_path_index::{parse_module_binding, ParsedModuleBinding};
use crate::shared_typecheck_store::{self, SharedTypecheckCaches};
use crate::std_node::compiler_recursive_types;
use crate::std_syntax::LiteralValue;
use crate::std_types::{kernel_type_set, SourceSpan};
use crate::v1_compiler_compile;
use crate::v1_compiler_infer;
use crate::v1_compiler_infer_env::{
    lookup_type_by_name, qualified_all_but_last, symbol_index_insert, symbol_index_lookup,
    GlobalBareLookupState, SymbolIndex,
};
use crate::v1_compiler_infer_items::{item_kind, ItemInfo, ItemKind, ResolvedGraph, TypedModule};
use crate::v1_compiler_infer_sigs::{lookup_resolved_sig, ResolvedFuncEnv, ResolvedFuncSig};
use crate::v1_compiler_normalize;
use crate::v1_compiler_parse;
use crate::v1_compiler_resolve;
use crate::v1_compiler_tokenize;
use crate::v1_interpreter;
use crate::v1_rt;
use crate::v1_std_core::{
    arg_name_at, arg_value, arm_pattern, authored_name_at, block_stmts, build_newline_index,
    byte_to_line_col, diagnostic_to_message, diagnostic_to_span, empty_intern_table,
    empty_node_list, expr_call_func_at, expr_method_name_at, expr_var_name_at, field_access_base,
    field_access_field_at, field_init_node_name_at, field_init_node_value, has_child_named,
    inferred_to_node, intern, is_discovery_corpus_advisory_typecheck_diagnostic,
    is_discovery_corpus_blocking_diagnostic, is_error_diagnostic,
    is_interpreter_blocking_diagnostic, let_binding_name_at, let_value, match_arm_nodes,
    match_scrutinee, method_arg_nodes, method_receiver, module_items, no_span, param_node_name_at,
    param_node_type_expr, Cardinality, CompilerDiagnostic, Connective, ErrorNode, ExprData,
    ExprErrorKind, InferredNode, InternTable, MatchPattern, NewlineIndex, Node,
};
use serde::Serialize;

#[path = "phase_profile.rs"]
mod phase_profile;
pub use phase_profile::{set_phase, FloorPhase, PhaseProfile};

use crate::resolved_graph_cache::{
    lookup as cross_process_lookup, resolved_graph_cache_root_from_env, subject_digest_for_closure,
    transform_content_digest, write as cross_process_write, CacheLookupResult,
};
use crate::std_interface_summary::{module_key, typed_module_key};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResolveTypecheckGate {
    Strict,
    DiscoveryCorpusAdvisory,
}

fn is_resolve_typecheck_blocking(d: Rc<CompilerDiagnostic>, gate: ResolveTypecheckGate) -> bool {
    match gate {
        ResolveTypecheckGate::Strict => is_interpreter_blocking_diagnostic(d),
        ResolveTypecheckGate::DiscoveryCorpusAdvisory => is_discovery_corpus_blocking_diagnostic(d),
    }
}

fn log_discovery_advisory_typecheck(
    d: &Rc<ErrorNode>,
    source_indices: &HashMap<String, Rc<NewlineIndex>>,
    gate: ResolveTypecheckGate,
) {
    if gate != ResolveTypecheckGate::DiscoveryCorpusAdvisory {
        return;
    }
    // Surface ALL discovery-corpus advisory diagnostics, not only those that also
    // interpreter-block. Every advisory diagnostic except UnlistedImportUse is already
    // interpreter-blocking, so this is a no-op for them; it wires UnlistedImportUse
    // (advisory + non-blocking, the diagnostic-collect signal) into the reporting path
    // instead of leaving it emitted-but-unobservable (§5 spec-without-execution).
    if is_discovery_corpus_advisory_typecheck_diagnostic(d.diagnostic.clone()) {
        let span = diagnostic_to_span(d.diagnostic.clone());
        let loc = format_error_loc(&span.file, span.start, source_indices);
        eprintln!(
            "advisory(typecheck): {}: error: {}",
            loc,
            diagnostic_to_message(d.diagnostic.clone())
        );
    }
}

pub const UNIFIED_CLAIM_VERIFICATION_MODULE: &str = "v2.std.verification";
pub const BOOL_WITNESS_CLAIM_TYPE: &str = "BoolWitnessClaim";
pub const NODE_CORPUS_TYPE: &str = "NodeCorpus";

// cargo's build-output dir (a `target` dir beside a Cargo.toml) is realization
// output, not source: a corpus copy materialized under it (e.g.
// target/func_env_semantic_baseline_corpus/dag/**) must never enter a module
// index alongside the tree it was copied from. A source root passed FROM
// inside target/ is still walked — only descent into the output dir is refused.
pub(crate) fn is_cargo_target_output_dir(
    parent: &std::path::Path,
    child: &std::path::Path,
) -> bool {
    child.file_name().and_then(|n| n.to_str()) == Some("target")
        && parent.join("Cargo.toml").is_file()
}

fn collect_dag_files(dir: &std::path::Path, files: &mut Vec<std::path::PathBuf>) {
    let mut entries: Vec<_> = std::fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("failed to read dir {:?}: {}", dir, e))
        .map(|e| e.unwrap_or_else(|e| panic!("failed to read dir entry: {}", e)))
        .collect();
    entries.sort_by_key(|e| e.file_name());
    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            if is_cargo_target_output_dir(dir, &path) {
                continue;
            }
            collect_dag_files(&path, files);
        } else if path.extension().map(|e| e == "dag").unwrap_or(false) {
            files.push(path);
        }
    }
}

pub(crate) fn extract_module_path(content: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("module ") {
            return Some(trimmed["module ".len()..].trim().to_string());
        }
        if !trimmed.is_empty() && !trimmed.starts_with("//") {
            break;
        }
    }
    None
}

/// Module-less `.dag` fragments (parse fixtures) are excluded from the compile entry
/// set. Fail-closed visibility: list every skipped path so a forgotten `module` decl
/// in real source is surfaced, not silently dropped.
pub fn report_moduleless_dag_entry_skips(skipped_paths: &[String]) {
    if skipped_paths.is_empty() {
        return;
    }
    eprintln!(
        "skipped {} module-less .dag file(s) from compile entry set (no `module` declaration):",
        skipped_paths.len()
    );
    for path in skipped_paths {
        eprintln!("  {path}");
    }
}

pub fn moduleless_dag_entry_paths(entry_files: &[(String, String)]) -> Vec<String> {
    entry_files
        .iter()
        .filter(|(_, content)| extract_module_path(content).is_none())
        .map(|(path, _)| path.clone())
        .collect()
}

pub(crate) fn extract_import_paths(content: &str) -> Vec<String> {
    let mut imports = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("import ") {
            let rest = trimmed["import ".len()..].trim();
            let module_path = rest.split('{').next().unwrap_or(rest).trim();
            if !module_path.is_empty() {
                imports.push(module_path.to_string());
            }
        }
    }
    imports
}

// SCAFFOLD (§7 seed-retained HAND-RUST — authority: gunbc.cli_run_workspace_root_scaffold;
// receipt: docs/plans/cli-run-reconcile-defork.md#interim-workspace-root-scaffold;
// witness: dag/test/claim/cli_run_workspace_root_hand_rust_witness_test.dag).
// 🟡 dissolve-on: workspace_root_from — runtime workspace-root discovery in HAND-Rust
// (landed on main as #6484 .git-ancestor walk); DISSOLVES WHEN cli_run.rs Chunk F lands
// (cli-run-reconcile-defork.md → GENERATED workflow host-effect apply) OR ROADMAP
// 5-dissolve-patches cli_run.rs shrink retires HAND path-dependent root entirely.
// Discriminating receipt: workspace_root_discovery_tests (same kernel as workspace_root()).
/// Single authority for workspace-root discovery (.git ancestor walk).
/// `workspace_root()` memoizes from the process cwd; tests pass an explicit start path.
pub(crate) fn workspace_root_from(start_cwd: &Path) -> PathBuf {
    for dir in start_cwd.ancestors() {
        if dir.join(".git").exists() {
            return dir.to_path_buf();
        }
    }
    panic!(
        "workspace_root: {} is not inside a git checkout; run from \
         the workspace (the compile-time CARGO_MANIFEST_DIR fallback was removed: \
         binaries are shared across runner workspaces and the compiling checkout's \
         path is not a runtime fact)",
        start_cwd.display()
    )
}

/// The workspace root is a property of where the process RUNS, never of where the
/// binary was COMPILED. A `CARGO_MANIFEST_DIR` bake is not a runtime fact: CI shares
/// the release binaries across jobs via artifacts, and the build job and the consuming
/// job can land on different runner instances whose checkouts live at different
/// absolute paths — the baked path then names a SIBLING runner's tree (observed
/// 2026-07-11: `build_module_path_index` refusing srv2-01 paths against a baked
/// srv2-02 root after the #6472 job split). Same class as the mixed-tree hazard
/// documented on `resolve_cli_path_arg` below, one level up.
///
/// Derivation: nearest ancestor of the process cwd that is a checkout root (`.git`
/// entry — a directory for clones, a file for worktrees). Computed once per process.
/// A cwd outside any checkout refuses loudly — no fallback to a compile-time path
/// (DESIGN §5: refuse, never widen).
// SCAFFOLD (§7 hand-Rust shrink-to-zero, dissolution named): runtime checkout-root
// derivation (#6484 / #6472 job-split). Dissolves when release bins receive checkout-root
// at spawn (env/argv) or Step 5 deletes this Rust parallel and the v2 floor workflow
// owns path resolution.
pub fn workspace_root() -> PathBuf {
    static ROOT: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();
    ROOT.get_or_init(|| {
        let cwd =
            std::env::current_dir().expect("workspace_root: process working directory unavailable");
        workspace_root_from(&cwd)
    })
    .clone()
}

// SCAFFOLD (§7 HAND-RUST — `cli_run_runtime_workspace_root_plumbing`):
// ROADMAP lane `5-dissolve-patches` (gunbc.roadmap_authority / ROADMAP.md) — `cli_run.rs`
// HAND_MAINTAINED drain (~12.1k LOC absorption point; #6046 hard-gates net-new seed logic).
// Unblock: #6106 orchestration emission → agnostic registry dispatch realizes claim-bin
// pool-root anchoring from `.dag` (same exit as bash-emit #5828 for floor shell scaffolds).
// DELETE WHEN dissolved: `process_workspace_root`, `resolve_process_workspace_root`,
// `anchor_source_root`, `repo_relative_path`, `repo_relative_path_normalized`, and call-site
// migration in `build_module_*` / `pool_roots_*` / `workspace_relative_repo_path` (~130 LOC).
// Receipt: `rg cli_run_runtime_workspace_root_plumbing src/v1/stage0/src/cli_run.rs` == 1 until
// deletion; not a compiler_frontier `.dag` row (seed-Rust, counted here not in module census).
pub(crate) const CLI_RUN_RUNTIME_WORKSPACE_ROOT_SCAFFOLD_MARKER: &str =
    "cli_run_runtime_workspace_root_plumbing";

/// Runtime workspace root for path normalization in the claim bins and module-graph pipeline.
///
/// INTERIM hand-Rust scaffold (`CLI_RUN_RUNTIME_WORKSPACE_ROOT_SCAFFOLD_MARKER` / §7): dissolves
/// under ROADMAP `5-dissolve-patches` when #6106 realizes claim-bin path resolution from `.dag`
/// and this helper family deletes (~130 LOC). Unlike [`workspace_root`] (compile-time from
/// `CARGO_MANIFEST_DIR`), this resolves against the process environment. sccache can ship a binary
/// built on one runner checkout path to another; anchoring file reads to the compile-time root
/// desyncs module-graph facts from module-content indices (DESIGN §5 — wrong answers with zero
/// diagnostic).
///
/// Resolution order: `git rev-parse --show-toplevel` when it names a Cargo.toml+dag/ tree,
/// else walk up from cwd. Fail-closed panic when neither locates the workspace — no silent
/// fallback to cwd-relative or absolute spellings as index keys.
fn process_workspace_root() -> PathBuf {
    static ROOT: OnceLock<PathBuf> = OnceLock::new();
    ROOT.get_or_init(resolve_process_workspace_root).clone()
}

fn resolve_process_workspace_root() -> PathBuf {
    if let Ok(output) = std::process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
    {
        if output.status.success() {
            let root = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !root.is_empty() {
                let candidate = PathBuf::from(&root);
                if candidate.join("Cargo.toml").is_file() && candidate.join("dag").is_dir() {
                    return candidate;
                }
            }
        }
    }
    let mut dir = std::env::current_dir().expect("process_workspace_root: cwd unavailable");
    loop {
        if dir.join("Cargo.toml").is_file() && dir.join("dag").is_dir() {
            return dir;
        }
        if !dir.pop() {
            let cwd = std::env::current_dir()
                .map(|d| d.display().to_string())
                .unwrap_or_else(|_| "<unavailable>".into());
            panic!(
                "process_workspace_root: cannot locate workspace — git rev-parse did not name \
                 a Cargo.toml+dag/ tree and no such ancestor of cwd {cwd}; compiled-in root \
                 was {}",
                workspace_root().display()
            );
        }
    }
}

/// Repo-relative path under [`process_workspace_root`]. Fail-closed: returns a typed refusal
/// when `path` is not under the runtime root — never widens to cwd-relative or absolute keys.
///
/// Authority: modeled by `gunbc.cli_run_repo_grant` (`cli_run_repo_path_admissible`);
/// witness: `dag/test/claim/cli_run_repo_grant_witness_test.dag`.
/// 🟡 dissolve-on: HAND-RUST gate retires when cli_run.rs Chunk F lands
/// (docs/plans/cli-run-reconcile-defork.md) — refusal becomes `EffectOutsideGrant` from the
/// single grant row, not a parallel string check.
fn repo_relative_path(path: &Path) -> Result<String, String> {
    let ws = process_workspace_root();
    path.strip_prefix(&ws)
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .map_err(|_| {
            format!(
                "repo_relative_path: path {} is not under process workspace root {} \
                 (compiled-in root was {})",
                path.display(),
                ws.display(),
                workspace_root().display()
            )
        })
}

/// Canonical repo-relative path for module-graph keys and index storage.
/// Tries [`process_workspace_root`] first, then compile-time [`workspace_root`] when
/// sccache embedded the latter in absolute spellings from another runner checkout.
/// A relative spelling is accepted as already being the key ONLY when it names a real
/// file or directory under the process root (verified, not trusted): walks over
/// relative source roots (`layer_import_facts`) emit exactly these spellings, while a
/// relative path anchored anywhere else stays a refusal, never a fabricated key.
fn repo_relative_path_normalized(path: &Path) -> String {
    try_repo_relative_path_normalized(path).unwrap_or_else(|| {
        panic!(
            "repo_relative_path_normalized: path {} is not under process workspace \
             root {} or compiled-in root {}",
            path.display(),
            process_workspace_root().display(),
            workspace_root().display()
        )
    })
}

fn try_repo_relative_path_normalized(path: &Path) -> Option<String> {
    if path.is_relative() && process_workspace_root().join(path).exists() {
        return Some(path.to_string_lossy().replace('\\', "/"));
    }
    if let Ok(rel) = repo_relative_path(path) {
        return Some(rel);
    }
    let baked = workspace_root();
    path.strip_prefix(&baked)
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .ok()
}

/// Module-index key for a file under a caller-supplied source root. Files under an
/// OUT-OF-TREE absolute root (a temp fixture tree, another checkout handed to the
/// parse-only audits) have no workspace-relative spelling — their absolute path IS the
/// canonical, readable key, not a fabrication. A relative spelling anchored outside the
/// process root stays a refusal (the fabricated-key hazard the panic guards).
fn module_index_path_key(path: &Path) -> String {
    match try_repo_relative_path_normalized(path) {
        Some(rel) => rel,
        None if path.is_absolute() => path.to_string_lossy().replace('\\', "/"),
        None => repo_relative_path_normalized(path),
    }
}

// SCAFFOLD (§7 seed-retained HAND-RUST — authority: gunbc.cli_run_workspace_root_scaffold
// (cli_run_source_root_anchor_scaffold row);
// receipt: docs/plans/cli-run-reconcile-defork.md#interim-workspace-root-scaffold;
// witness: dag/test/claim/cli_run_workspace_root_hand_rust_witness_test.dag).
// 🟡 dissolve-on: try_anchor_source_root — declared layer/pool roots come from HAND-Rust
// while `dag/compiler` is modeled-before-implemented (§6 model-first), so absence is a
// legitimate state skipped LOUDLY (counted line per root); DISSOLVES WHEN cli_run.rs Chunk F
// lands (roots walk GENERATED; absence becomes a typed, located, counted diagnostic at the
// roster layer) OR ROADMAP 5-dissolve-patches retires HAND path handling.
// Discriminating receipts: try_anchor_source_root_resolves_declared_present_root /
// try_anchor_source_root_skips_declared_absent_root.
/// Resolve a source/pool-root spelling to an absolute directory under
/// [`process_workspace_root`]. Absolute paths baked from the compile-time
/// [`workspace_root`] (sccache cross-runner) are re-anchored when missing on disk.
/// Non-panicking sibling of [`anchor_source_root`] for DECLARED layer/pool roots whose absence is
/// a legitimate state (a modeled-before-implemented location, e.g. `dag/compiler` in the
/// medium-structure roster): `None` when the root does not exist under any anchoring, so the
/// caller can skip it LOUDLY (counted line) instead of panicking mid-floor. CLI-provided roots
/// keep the strict panicking contract - a typo'd argument must refuse, never skip.
fn try_anchor_source_root(root: &str) -> Option<String> {
    let p = Path::new(root);
    let ws = process_workspace_root();
    if p.is_absolute() {
        if p.is_dir() {
            return Some(root.to_string());
        }
        let baked = workspace_root();
        if let Ok(rel) = p.strip_prefix(&baked) {
            let reanchored = ws.join(rel);
            if reanchored.is_dir() {
                return Some(reanchored.to_string_lossy().into_owned());
            }
        }
        if let Ok(rel) = repo_relative_path(p) {
            let reanchored = ws.join(&rel);
            if reanchored.is_dir() {
                return Some(reanchored.to_string_lossy().into_owned());
            }
        }
        return None;
    }
    let anchored = ws.join(p);
    if anchored.is_dir() {
        Some(anchored.to_string_lossy().into_owned())
    } else {
        None
    }
}

fn anchor_source_root(root: &str) -> String {
    let p = Path::new(root);
    let ws = process_workspace_root();
    if p.is_absolute() {
        if p.is_dir() {
            return root.to_string();
        }
        let baked = workspace_root();
        if let Ok(rel) = p.strip_prefix(&baked) {
            let reanchored = ws.join(rel);
            if reanchored.is_dir() {
                return reanchored.to_string_lossy().into_owned();
            }
        }
        if let Ok(rel) = repo_relative_path(p) {
            let reanchored = ws.join(&rel);
            if reanchored.is_dir() {
                return reanchored.to_string_lossy().into_owned();
            }
        }
        panic!(
            "anchor_source_root: absolute source root {} does not exist and cannot be \
             re-anchored under process workspace {} (compiled-in root was {})",
            root,
            ws.display(),
            baked.display()
        );
    }
    let anchored = ws.join(p);
    if !anchored.is_dir() {
        panic!(
            "anchor_source_root: source root {root} resolved to {} which is not a directory \
             (process workspace {})",
            anchored.display(),
            ws.display()
        );
    }
    anchored.to_string_lossy().into_owned()
}

/// CLI-boundary path resolution for the claim bins (`claim_batch` / `claim_executor`).
///
/// `workspace_root()` above was HISTORICALLY baked from `env!("CARGO_MANIFEST_DIR")` at
/// COMPILE time (now cwd-derived at runtime, see its doc), and part of the shared
/// resolution pipeline (`pool_roots_abs`) anchors RELATIVE source roots to that path
/// while the module-content index reads them relative to the process cwd. For the claim
/// bins that meant a run from any other cwd (e.g. a git worktree) silently mixed two
/// trees — module contents from the cwd, import-graph facts from the baked root — wrong
/// answers with zero diagnostic (DESIGN §5 fail-open). The runtime derivation removes
/// the cross-tree case; this boundary keeps the in-tree case exact (a cwd BELOW the
/// checkout root still resolves CLI args against the cwd, not the root).
///
/// The bins therefore resolve their path-valued arguments HERE, at the CLI boundary,
/// with standard CLI semantics: a relative path resolves against the PROCESS CWD, and a
/// resolved path that does not exist is a refusal naming the argument, the given value,
/// and the resolution base — never a fallback to the baked root, never a partial run.
/// When the cwd IS the baked workspace root, the relative spelling and the absolutized
/// spelling denote the same file for every downstream consumer (cwd-anchored reads and
/// baked-root-anchored reads agree), so the given spelling passes through unchanged and
/// the normal case (and CI) stays byte-identical. Any other cwd absolutizes, so the
/// baked-root anchoring in the shared pipeline can never re-route the read.
pub fn resolve_cli_path_arg(bin: &str, flag: &str, given: &str) -> Result<String, String> {
    if Path::new(given).is_absolute() {
        return resolve_cli_path_arg_against(bin, flag, given, Path::new("/"));
    }
    let cwd = std::env::current_dir().map_err(|e| {
        format!(
            "{bin}: {flag} {given}: cannot resolve a relative CLI path — the process \
             working directory is unavailable: {e}"
        )
    })?;
    resolve_cli_path_arg_against(bin, flag, given, &cwd)
}

/// Core of [`resolve_cli_path_arg`] with an explicit resolution base (testable without
/// mutating the process-global cwd).
fn resolve_cli_path_arg_against(
    bin: &str,
    flag: &str,
    given: &str,
    base: &Path,
) -> Result<String, String> {
    let given_path = Path::new(given);
    if given_path.is_absolute() {
        if !given_path.exists() {
            return Err(format!(
                "{bin}: {flag} {given}: absolute path does not exist; refusing (no \
                 fallback to the compiled-in workspace root {})",
                workspace_root().display()
            ));
        }
        return Ok(given.to_string());
    }
    let resolved = base.join(given_path);
    if !resolved.exists() {
        return Err(format!(
            "{bin}: {flag} {given}: resolved against the process working directory {} \
             to {}, which does not exist; refusing — relative CLI paths resolve against \
             the process cwd, never the compiled-in workspace root {} — run from the \
             tree you meant or pass an absolute path",
            base.display(),
            resolved.display(),
            workspace_root().display()
        ));
    }
    let ws = workspace_root();
    if same_canonical_file(&base.to_string_lossy(), &ws.to_string_lossy()) {
        return Ok(given.to_string());
    }
    Ok(resolved.to_string_lossy().into_owned())
}

#[cfg(test)]
mod cli_path_arg_resolution_tests {
    use super::{resolve_cli_path_arg_against, workspace_root};

    fn fixture_base(tag: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("gunbc-cli-path-arg-{tag}-{}", std::process::id()))
    }

    /// (a) Relative arg with base == workspace root: byte-identical pass-through —
    /// the exact string the bins forwarded before this boundary existed.
    #[test]
    fn relative_arg_at_workspace_root_passes_through_unchanged() {
        let ws = workspace_root();
        let resolved = resolve_cli_path_arg_against("claim_batch", "--source-root", "dag", &ws)
            .expect("`dag` exists under the workspace root");
        assert_eq!(resolved, "dag");
    }

    /// (b) Relative arg from a different cwd resolves against THAT cwd — the result is
    /// the absolutized fixture path, never the bare relative spelling (which the shared
    /// pipeline would re-anchor to the baked workspace root) and never a baked-root path.
    #[test]
    fn relative_arg_from_other_cwd_resolves_against_that_cwd() {
        let base = fixture_base("other-cwd");
        std::fs::create_dir_all(base.join("dag")).expect("create fixture tree");
        std::fs::write(base.join("dag/mini.dag"), "module mini\n").expect("write fixture module");

        let root = resolve_cli_path_arg_against("claim_batch", "--source-root", "dag", &base)
            .expect("fixture tree exists under the fixture cwd");
        assert_eq!(root, base.join("dag").to_string_lossy());
        assert_ne!(
            root, "dag",
            "must absolutize away from the baked-root anchor"
        );

        let entry = resolve_cli_path_arg_against("claim_batch", "--entry", "dag/mini.dag", &base)
            .expect("fixture entry exists under the fixture cwd");
        assert_eq!(entry, base.join("dag/mini.dag").to_string_lossy());

        let _ = std::fs::remove_dir_all(&base);
    }

    /// (c) Nonexistent relative path: refusal names the argument, the given value, and
    /// the resolution base — and does NOT resolve against the baked workspace root even
    /// though the path exists there (the discriminating red control for the fail-open).
    #[test]
    fn nonexistent_relative_arg_refuses_naming_flag_value_and_base() {
        let base = fixture_base("refusal");
        std::fs::create_dir_all(&base).expect("create empty fixture cwd");
        // `dag` exists under the baked workspace root but NOT under `base`; a baked-root
        // fallback would return Ok here — the refusal is the proof there is none.
        let err = resolve_cli_path_arg_against("claim_batch", "--source-root", "dag", &base)
            .expect_err("missing path must refuse, never fall back to the baked root");
        assert!(err.contains("claim_batch"), "names the binary: {err}");
        assert!(err.contains("--source-root"), "names the argument: {err}");
        assert!(err.contains("dag"), "names the given value: {err}");
        assert!(
            err.contains(&base.display().to_string()),
            "names the resolution base: {err}"
        );
        let _ = std::fs::remove_dir_all(&base);
    }

    /// Nonexistent absolute path refuses too — never proceeds partially.
    #[test]
    fn nonexistent_absolute_arg_refuses() {
        let missing = fixture_base("absolute-missing").join("no/such/entry.dag");
        let missing_str = missing.to_string_lossy().into_owned();
        let err = resolve_cli_path_arg_against(
            "claim_executor",
            "--plan-entry",
            &missing_str,
            std::path::Path::new("/"),
        )
        .expect_err("missing absolute path must refuse");
        assert!(err.contains("--plan-entry"), "names the argument: {err}");
        assert!(err.contains(&missing_str), "names the given value: {err}");
    }
}

#[cfg(test)]
mod process_workspace_root_tests {
    use super::{
        anchor_source_root, process_workspace_root, repo_relative_path,
        repo_relative_path_normalized, try_anchor_source_root, workspace_relative_repo_path,
        workspace_root,
    };
    use std::path::Path;

    #[test]
    fn process_workspace_root_locates_cargo_and_dag() {
        let root = process_workspace_root();
        assert!(root.join("Cargo.toml").is_file());
        assert!(root.join("dag").is_dir());
    }

    #[test]
    fn repo_relative_path_normalizes_under_process_root() {
        let root = process_workspace_root();
        let abs = root.join("dag/gunbc/ci_layer_roots.dag");
        let rel = repo_relative_path(&abs).expect("under process root");
        assert_eq!(rel, "dag/gunbc/ci_layer_roots.dag");
        assert_eq!(
            workspace_relative_repo_path(&abs.to_string_lossy()),
            "dag/gunbc/ci_layer_roots.dag"
        );
    }

    #[test]
    fn anchor_source_root_resolves_relative_dag() {
        let anchored = anchor_source_root("dag");
        assert!(Path::new(&anchored)
            .join("gunbc/ci_layer_roots.dag")
            .is_file());
    }

    #[test]
    fn try_anchor_source_root_resolves_declared_present_root() {
        let anchored = try_anchor_source_root("dag").expect("dag exists in every checkout");
        assert!(Path::new(&anchored)
            .join("gunbc/ci_layer_roots.dag")
            .is_file());
    }

    #[test]
    fn try_anchor_source_root_skips_declared_absent_root() {
        assert_eq!(
            try_anchor_source_root("dag/declared-but-not-yet-implemented-root"),
            None
        );
    }

    #[test]
    fn runtime_workspace_root_scaffold_marker_is_declared() {
        assert_eq!(
            super::CLI_RUN_RUNTIME_WORKSPACE_ROOT_SCAFFOLD_MARKER,
            "cli_run_runtime_workspace_root_plumbing"
        );
    }

    #[test]
    fn discovery_skip_before_resolve_scaffold_marker_is_declared() {
        assert_eq!(
            super::CLI_RUN_DISCOVERY_SKIP_BEFORE_RESOLVE_SCAFFOLD_MARKER,
            "cli_run_discovery_skip_before_resolve"
        );
    }

    #[test]
    fn compile_clean_diagnostic_histogram_scaffold_marker_is_declared() {
        assert_eq!(
            super::CLI_RUN_COMPILE_CLEAN_DIAGNOSTIC_HISTOGRAM_SCAFFOLD_MARKER,
            "cli_run_compile_clean_diagnostic_histogram"
        );
    }

    #[test]
    fn effect_reach_inference_bridge_scaffold_marker_is_declared() {
        assert_eq!(
            super::CLI_RUN_EFFECT_REACH_INFERENCE_BRIDGE_SCAFFOLD_MARKER,
            "cli_run_effect_reach_inference_bridge"
        );
    }

    #[test]
    fn declared_source_ref_selection_bridge_scaffold_marker_is_declared() {
        assert_eq!(
            super::CLI_RUN_DECLARED_SOURCE_REF_SELECTION_BRIDGE_MARKER,
            "cli_run_declared_source_ref_selection_bridge"
        );
    }

    #[test]
    fn truncate_histogram_label_respects_utf8_boundaries() {
        let max = 80;
        let s = "é".repeat(50); // 2-byte chars; byte slice at 79 would straddle
        let out = super::truncate_histogram_label(&s, max);
        assert!(out.ends_with('…'));
        assert!(out.is_char_boundary(out.len()));
        assert!(out.len() <= max);
    }

    #[test]
    fn repo_relative_path_normalized_reanchors_baked_absolute_file() {
        let ws = process_workspace_root();
        let baked = workspace_root();
        let abs = baked.join("dag/gunbc/ci_layer_roots.dag");
        if !abs.is_file() {
            return;
        }
        let rel = repo_relative_path_normalized(&abs);
        assert_eq!(rel, "dag/gunbc/ci_layer_roots.dag");
        assert_eq!(workspace_relative_repo_path(&abs.to_string_lossy()), rel);
        assert!(ws.join(&rel).is_file());
    }

    /// The 2026-07-11 main-red regression (#6459 -> runs 29161502622/29161935015/29162102862):
    /// `layer_import_facts` walks relative source roots ("dag", "src/v2") and feeds the
    /// resulting relative spellings here; they ARE the repo-relative keys and must pass
    /// through once verified against the process root, never strip-prefix-panic.
    #[test]
    fn repo_relative_path_normalized_accepts_relative_spelling_under_root() {
        let rel_in = Path::new("dag/gunbc/ci_layer_roots.dag");
        if !process_workspace_root().join(rel_in).is_file() {
            return;
        }
        assert_eq!(
            repo_relative_path_normalized(rel_in),
            "dag/gunbc/ci_layer_roots.dag"
        );
    }

    /// Red control: a relative spelling that does NOT exist under the process root is
    /// unattributable input and must refuse — the relative arm verifies, it never trusts.
    #[test]
    #[should_panic(expected = "repo_relative_path_normalized")]
    fn repo_relative_path_normalized_refuses_relative_spelling_not_under_root() {
        let _ = repo_relative_path_normalized(Path::new(
            "no-such-dir/no-such-file-gunbc-red-control.dag",
        ));
    }
}

/// Pins HAND-RUST `repo_relative_path` against `gunbc.cli_run_repo_grant` on the same
/// fixture spellings. Witness: `dag/test/claim/cli_run_repo_grant_hand_rust_equivalence_witness_test.dag`.
#[cfg(test)]
mod cli_run_repo_grant_equivalence_tests {
    use super::{process_workspace_root, repo_relative_path};
    use std::path::Path;

    #[test]
    fn cli_run_repo_grant_equivalence_contained_admits() {
        let root = process_workspace_root();
        let rel = Path::new("dag/std/effect_grant.dag");
        let abs = root.join(rel);
        let got = repo_relative_path(&abs).expect("contained path under workspace");
        assert_eq!(got, "dag/std/effect_grant.dag");
    }

    #[test]
    fn cli_run_repo_grant_equivalence_absolute_outside_refuses() {
        assert!(repo_relative_path(Path::new("/etc/passwd")).is_err());
    }

    #[test]
    fn cli_run_repo_grant_equivalence_parent_segment_refuses() {
        assert!(
            repo_relative_path(Path::new("../outside.dag")).is_err(),
            "parent-segment relative spelling must refuse"
        );
    }
}

#[cfg(test)]
mod workspace_root_discovery_tests {
    use super::workspace_root_from;
    use std::path::{Path, PathBuf};
    use std::process::Command;

    fn fixture_root(tag: &str) -> PathBuf {
        std::env::temp_dir().join(format!("gunbc-ws-root-{tag}-{}", std::process::id()))
    }

    fn canonical(p: &Path) -> PathBuf {
        std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf())
    }

    /// Cross-job / worktree: claim bins run from a checkout subdirectory must resolve
    /// the same root as `git rev-parse --show-toplevel`, not the compile-time baked path.
    #[test]
    fn from_subdirectory_matches_git_toplevel() {
        let top = Command::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .output()
            .expect("git rev-parse");
        assert!(top.status.success(), "must run inside a git checkout");
        let top = PathBuf::from(String::from_utf8(top.stdout).unwrap().trim());
        let sub = top.join("dag");
        assert!(sub.is_dir(), "dag/ must exist under checkout");
        let got = workspace_root_from(&sub);
        assert_eq!(canonical(&got), canonical(&top));
    }

    /// Fail-closed: Cargo.toml+dag/ without a `.git` ancestor is not a checkout root.
    #[test]
    fn refuses_path_outside_git_checkout() {
        let root = fixture_root("no-git");
        std::fs::create_dir_all(root.join("dag")).expect("dag dir");
        std::fs::write(root.join("Cargo.toml"), "[workspace]\n").expect("Cargo.toml");
        let nested = root.join("nested/deep");
        std::fs::create_dir_all(&nested).expect("nested");
        let result = std::panic::catch_unwind(|| workspace_root_from(&nested));
        let _ = std::fs::remove_dir_all(&root);
        assert!(result.is_err(), "must panic outside a git checkout");
    }
}

#[cfg(test)]
mod regen_input_closure_tests {
    use super::{regen_input_sources, regen_path_affects_regen};
    use std::collections::HashSet;
    use std::path::{Path, PathBuf};

    fn fixture_root(tag: &str) -> PathBuf {
        std::env::temp_dir().join(format!("gunbc-regen-closure-{tag}-{}", std::process::id()))
    }

    fn write(root: &Path, rel: &str, content: &str) {
        let p = root.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).expect("mkdir fixture");
        std::fs::write(p, content).expect("write fixture");
    }

    /// The regen input closure is exactly the `src/v1` entries plus their transitive
    /// `import` closure through `[src/v1, dag]` — a `dag` module nobody imports is
    /// EXCLUDED. This is the RED control: without the exclusion the skip is vacuous
    /// (every dag change would appear in-closure and the shortcut would never fire).
    #[test]
    fn closure_is_imports_only_unimported_dag_excluded() {
        let root = fixture_root("imports-only");
        let _ = std::fs::remove_dir_all(&root);
        write(
            &root,
            "src/v1/a.dag",
            "module v1.a\nimport v1.b { T }\nimport std.x { Y }\n",
        );
        write(&root, "src/v1/b.dag", "module v1.b\n");
        write(&root, "dag/std/x.dag", "module std.x\n");
        write(&root, "dag/std/y.dag", "module std.y\n"); // unimported — excluded
        write(&root, "dag/gunbc/u.dag", "module gunbc.u\n"); // unimported — excluded

        let relpaths: HashSet<String> = regen_input_sources(&root)
            .expect("closure computes")
            .into_iter()
            .map(|(p, _)| p)
            .collect();
        let _ = std::fs::remove_dir_all(&root);

        assert!(
            relpaths.contains("src/v1/a.dag"),
            "entry seed: {relpaths:?}"
        );
        assert!(
            relpaths.contains("src/v1/b.dag"),
            "imported v1 module present"
        );
        assert!(
            relpaths.contains("dag/std/x.dag"),
            "imported dag module present"
        );
        assert!(
            !relpaths.contains("dag/std/y.dag"),
            "unimported dag module must be EXCLUDED: {relpaths:?}"
        );
        assert!(
            !relpaths.contains("dag/gunbc/u.dag"),
            "unimported dag module must be EXCLUDED: {relpaths:?}"
        );
    }

    #[test]
    fn path_predicate_covers_src_v1_manifest_and_closure_only() {
        let closure: HashSet<String> = ["dag/std/x.dag".to_string()].into_iter().collect();
        // src/v1/** = emitter source + committed stage0 outputs + dag entry seeds.
        assert!(regen_path_affects_regen(
            "src/v1/stage0/src/cli_run.rs",
            &closure
        ));
        assert!(regen_path_affects_regen("src/v1/03_resolve.dag", &closure));
        // Cargo / toolchain build config (emitter binary inputs).
        assert!(regen_path_affects_regen("Cargo.lock", &closure));
        assert!(regen_path_affects_regen(
            "src/v1/stage0/Cargo.toml",
            &closure
        ));
        assert!(regen_path_affects_regen("rust-toolchain.toml", &closure));
        assert!(regen_path_affects_regen(".cargo/config.toml", &closure));
        // dag file in v1's transitive import closure.
        assert!(regen_path_affects_regen("dag/std/x.dag", &closure));
        // NOT affecting: v2 sources, unimported dag, docs — the skip-eligible surface.
        assert!(!regen_path_affects_regen(
            "src/v2/compiler/frontier.dag",
            &closure
        ));
        assert!(!regen_path_affects_regen("dag/gunbc/ci_spec.dag", &closure));
        assert!(!regen_path_affects_regen("docs/plans/x.md", &closure));
    }
}

/// Empty ingest-manifest placeholder excluded from the module index when a later
/// source root carries the host-emitted manifest (source-root ingest / closure gates).
const SOURCE_ROOT_INGEST_MANIFEST_STUB_REL: &str =
    "src/v2/test/claim/workflow/host_source_root_ingest_manifest.dag";

/// Empty module-binding-manifest placeholder, superseded the same way (module-identity
/// supply carrier). Separate carrier from the ingest manifest: module <-> path +
/// provenance, no source text.
const MODULE_BINDING_MANIFEST_STUB_REL: &str =
    "src/v2/test/claim/workflow/host_module_binding_manifest.dag";

/// Committed manifest stubs and the generated filenames that supersede them.
///
/// One table rather than a predicate pair per manifest: the stub/overlay rule is a single
/// fact, and forking it per carrier would mean the next manifest silently misses whichever
/// half its author forgot to extend (DESIGN.md §2/§3).
const MANIFEST_STUB_OVERLAYS: &[(&str, &[&str])] = &[
    (
        SOURCE_ROOT_INGEST_MANIFEST_STUB_REL,
        &[
            "v2-source-root-ingest-manifest.dag",
            "host_source_root_ingest_manifest.dag",
        ],
    ),
    (
        MODULE_BINDING_MANIFEST_STUB_REL,
        &["host_module_binding_manifest.dag"],
    ),
];

/// True when `rel_forward` is a committed manifest stub AND a strictly later source root
/// carries its generated counterpart, so the stub must be excluded from the module index
/// and the generated file becomes the sole declarer of that module. Without this the two
/// files collide and trip `module_path_collision_panic_message`.
fn manifest_stub_superseded_by_overlay(
    rel_forward: &str,
    source_roots: &[String],
    after_root_idx: usize,
) -> bool {
    let normalized = rel_forward.replace('\\', "/");
    let Some((_, overlay_names)) = MANIFEST_STUB_OVERLAYS
        .iter()
        .find(|(stub_rel, _)| normalized == *stub_rel)
    else {
        return false;
    };
    source_roots.iter().skip(after_root_idx + 1).any(|root| {
        let root_path = Path::new(root);
        overlay_names
            .iter()
            .any(|name| root_path.join(name).is_file())
    })
}

fn module_path_collision_panic_message(
    declaring_module: &str,
    existing_path: &str,
    candidate_path: &str,
) -> String {
    format!(
        "module-path collision: module '{declaring_module}' is declared by both '{existing_path}' and '{candidate_path}' — one module, one authority (DESIGN §3); silent last-root-wins shadowing broke the floor (extdeps.shell, 2026-07-01) — de-fork or rename one side"
    )
}

pub fn build_module_path_index(source_roots: &[String]) -> HashMap<String, String> {
    let key = source_roots
        .iter()
        .map(|r| anchor_source_root(r))
        .collect::<Vec<_>>()
        .join("\u{1f}");
    MODULE_PATH_INDEX_CACHE.with(|cache| {
        if let Some(index) = cache.borrow().get(&key) {
            return index.clone();
        }
        let index = build_module_path_index_uncached(source_roots);
        cache.borrow_mut().insert(key, index.clone());
        index
    })
}

fn build_module_path_index_uncached(source_roots: &[String]) -> HashMap<String, String> {
    let mut index: HashMap<String, String> = HashMap::new();
    for_each_parsed_module_binding(source_roots, |root_idx, path, binding| {
        let rel = module_index_path_key(path);
        if manifest_stub_superseded_by_overlay(&rel, source_roots, root_idx) {
            return;
        }
        insert_module_path(&mut index, &binding.module_path, rel);
    });
    index
}

#[derive(Clone)]
struct ModuleBindingManifestRow {
    module_path: String,
    rel_path: String,
    root_variant: String,
    ident_span: Rc<SourceSpan>,
}

fn witness_layer_root_spelling(root: &str) -> String {
    let p = Path::new(root);
    if p.is_absolute() {
        repo_relative_path_normalized(p)
    } else {
        root.trim_start_matches("./")
            .trim_end_matches('/')
            .to_string()
    }
}

fn insert_module_path(index: &mut HashMap<String, String>, module_path: &str, rel: String) {
    if let Some(existing) = index.get(module_path) {
        if existing != &rel && !same_canonical_file(existing, &rel) {
            panic!(
                "{}",
                module_path_collision_panic_message(module_path, existing, &rel)
            );
        }
        return;
    }
    index.insert(module_path.to_string(), rel);
}

fn for_each_parsed_module_binding(
    source_roots: &[String],
    mut visit: impl FnMut(usize, &Path, ParsedModuleBinding),
) {
    for (root_idx, root) in source_roots.iter().enumerate() {
        let anchored_root = anchor_source_root(root);
        let root_path = Path::new(&anchored_root);
        let mut dag_files = Vec::new();
        collect_dag_files(root_path, &mut dag_files);
        for path in dag_files {
            let content = std::fs::read_to_string(&path).unwrap_or_else(|e| {
                panic!(
                    "for_each_parsed_module_binding: failed to read {:?}: {}",
                    path, e
                )
            });
            let binding = match parse_module_binding(&path, &content) {
                Ok(Some(binding)) => binding,
                Ok(None) => continue,
                Err(msg) => panic!("for_each_parsed_module_binding: {msg}"),
            };
            visit(root_idx, &path, binding);
        }
    }
}

fn collect_module_binding_manifest_rows(source_roots: &[String]) -> Vec<ModuleBindingManifestRow> {
    let root_variants: Vec<String> = source_roots
        .iter()
        .map(|root| {
            let rel_root = witness_layer_root_spelling(root);
            source_root_ref_variant_for_root(&rel_root)
                .unwrap_or_else(|e| panic!("collect_module_binding_manifest_rows: {e}"))
        })
        .collect();
    let mut rows_by_module: std::collections::HashMap<String, ModuleBindingManifestRow> =
        std::collections::HashMap::new();
    for_each_parsed_module_binding(source_roots, |root_idx, path, binding| {
        let rel = module_index_path_key(path);
        if manifest_stub_superseded_by_overlay(&rel, source_roots, root_idx) {
            return;
        }
        if let Some(existing) = rows_by_module.get(&binding.module_path) {
            if existing.rel_path != rel && !same_canonical_file(&existing.rel_path, &rel) {
                panic!(
                    "{}",
                    module_path_collision_panic_message(
                        &binding.module_path,
                        &existing.rel_path,
                        &rel
                    )
                );
            }
            return;
        }
        rows_by_module.insert(
            binding.module_path.clone(),
            ModuleBindingManifestRow {
                module_path: binding.module_path,
                rel_path: rel,
                root_variant: root_variants[root_idx].clone(),
                ident_span: binding.ident_span,
            },
        );
    });
    let mut rows: Vec<ModuleBindingManifestRow> =
        rows_by_module.into_iter().map(|(_, row)| row).collect();
    rows.sort_by(|a, b| a.module_path.cmp(&b.module_path));
    rows
}

/// Resolve `import` statements transitively for an in-memory (not-on-disk) entry source
/// against `module_index` (from `build_module_path_index`), reading each imported module's
/// real file content from the workspace. This is the production-side counterpart of the
/// v1 test-harness's `resolve_imports_transitively` — the same BFS over `extract_import_paths`,
/// grounded on the same module index the floor already uses, so a `.dag` witness can compile
/// an arbitrary in-memory program (not just files already on disk) without a second resolver.
fn resolve_virtual_source_with_imports(
    entry_path: &str,
    entry_content: &str,
    module_index: &HashMap<String, String>,
) -> Vec<Rc<v1_compiler_compile::SourceFile>> {
    let ws = process_workspace_root();
    let mut seen: HashMap<String, Rc<v1_compiler_compile::SourceFile>> = HashMap::new();
    let mut queue: Vec<String> = vec![entry_content.to_string()];
    while let Some(content) = queue.pop() {
        for module_path in extract_import_paths(&content) {
            if seen.contains_key(&module_path) {
                continue;
            }
            if let Some(rel_path) = module_index.get(&module_path) {
                let abs_path = ws.join(rel_path);
                if let Ok(file_content) = std::fs::read_to_string(&abs_path) {
                    seen.insert(
                        module_path,
                        Rc::new(v1_compiler_compile::SourceFile {
                            path: rel_path.clone(),
                            content: file_content.clone(),
                        }),
                    );
                    queue.push(file_content);
                }
            }
        }
    }
    let mut sources: Vec<Rc<v1_compiler_compile::SourceFile>> =
        seen.into_iter().map(|(_, v)| v).collect();
    sources.sort_by(|a, b| a.path.cmp(&b.path));
    sources.push(Rc::new(v1_compiler_compile::SourceFile {
        path: entry_path.to_string(),
        content: entry_content.to_string(),
    }));
    sources
}

/// Host realization backing the `compile_dag_rust_emit_check` builtin: compile an in-memory
/// `.dag` program to Rust and check that the named emitted file contains every string in
/// `includes` and none of `excludes`, with zero non-`complexity:` diagnostics. A real,
/// green-by-execution consumer of the v1 Rust emitter (DESIGN §5 spec-without-execution) —
/// not a re-derivation of the emitter's own formula, so it can go red on a real emission
/// regression.
pub fn compile_dag_rust_emit_check(
    source: &str,
    file_path: &str,
    includes: &[String],
    excludes: &[String],
) -> bool {
    let module_index = build_module_path_index_from_witness_roots();
    let sources = resolve_virtual_source_with_imports("test.dag", source, &module_index);
    let result = v1_compiler_compile::compile_sources(
        Rc::new(sources.into()),
        crate::v1_compiler_artifact::RenderTarget::Rust,
    );
    let hard_diagnostics = result
        .diagnostics
        .iter()
        .filter(|d| !diagnostic_to_message(d.diagnostic.clone()).starts_with("complexity: "))
        .count();
    if hard_diagnostics != 0 {
        return false;
    }
    match result.files.iter().find(|f| f.path == file_path) {
        Some(f) => {
            includes.iter().all(|n| f.content.contains(n.as_str()))
                && excludes.iter().all(|n| !f.content.contains(n.as_str()))
        }
        None => false,
    }
}

const CI_LAYER_ROOTS_AUTHORITY_REL: &str = "dag/gunbc/ci_layer_roots.dag";
const WITNESS_LAYER_ROOTS_DATA_NAME: &str = "witness_layer_roots";
const WITNESS_DISCOVERY_SCAN_DIRS_DATA_NAME: &str = "witness_discovery_scan_dirs";
const WITNESS_EXCLUSION_SUBSTRINGS_DATA_NAME: &str = "witness_exclusion_substrings";
const WITNESS_ADMISSION_OFFLINE_EXCLUSION_SUBSTRINGS_DATA_NAME: &str =
    "witness_admission_offline_exclusion_substrings";
const WITNESS_ADMISSION_FIXTURE_EXCLUSION_SUBSTRINGS_DATA_NAME: &str =
    "witness_admission_fixture_exclusion_substrings";
const WET_RECEIPT_ENROLLMENT_AUTHORITY_REL: &str =
    "src/v2/compiler/self_host/wet_receipt_enrollment.dag";
const WHOLE_TREE_STRICT_RESOLVE_EXCLUSION_SUBSTRINGS_DATA_NAME: &str =
    "whole_tree_strict_resolve_exclusion_substrings";

fn ci_layer_roots_authority_content() -> &'static str {
    static CONTENT: OnceLock<String> = OnceLock::new();
    CONTENT
        .get_or_init(|| {
            let path = process_workspace_root().join(CI_LAYER_ROOTS_AUTHORITY_REL);
            std::fs::read_to_string(&path).unwrap_or_else(|e| {
                panic!(
                    "ci_layer_roots authority: failed to read {}: {e}",
                    path.display()
                )
            })
        })
        .as_str()
}

/// Project a `List<String>` data literal out of a `.dag` module's SOURCE TEXT via the real front-end
/// (`tokenize` + `parse`) — no second hand-rolled scanner. Pure (text in, list out) so a synthetic
/// authority carrying non-default values can drive it: a reader that ignored its input and returned
/// a hardcoded copy fails that control — the by-construction discrimination (DESIGN §5). Fail-closed:
/// a parse error, a missing data def, a non-string-list body, or (when `allow_empty` is false) an
/// empty list is a loud panic, never a silent fallback that would re-open the drift.
pub(crate) fn string_list_data_from_module_source(
    module_rel_path: &str,
    content: &str,
    data_name: &str,
    allow_empty: bool,
) -> Vec<String> {
    use crate::v1_std_core::{ExprData, LiteralValue};

    let filename = module_rel_path.to_string();
    let tokens = crate::v1_compiler_tokenize::tokenize(content.to_string(), filename.clone());
    let source_index =
        crate::v1_std_core::build_newline_index(filename.clone(), content.to_string());
    let mut source_indices = HashMap::new();
    source_indices.insert(filename.clone(), source_index);
    let result = crate::v1_compiler_parse::parse(tokens, std::rc::Rc::new(source_indices));
    if let Some(err) = result.error.as_ref() {
        panic!(
            "lens table reader: parse error in {module_rel_path}: {}",
            crate::v1_std_core::diagnostic_to_message(err.diagnostic.clone())
        );
    }
    let module = result
        .module
        .as_ref()
        .unwrap_or_else(|| panic!("lens table reader: {module_rel_path} parsed to no module"));
    for item in module.children.iter() {
        if item.name != data_name
            || !crate::v1_compiler_emit_core_support::is_data_def_item(item.clone())
        {
            continue;
        }
        let body = item.body.as_ref().unwrap_or_else(|| {
            panic!("lens table reader: `data {data_name}` in {module_rel_path} has no value body")
        });
        if !matches!(body.expr_data.as_ref(), ExprData::ExprListLit) {
            panic!(
                "lens table reader: `data {data_name}` in {module_rel_path} is not a \
                 `List<String>` literal"
            );
        }
        let mut values = Vec::new();
        for el in body.children.iter() {
            match el.expr_data.as_ref() {
                ExprData::ExprLiteral { value } => match value.as_ref() {
                    LiteralValue::LitStr { value } => values.push(value.clone()),
                    _ => panic!(
                        "lens table reader: an element of `{data_name}` in {module_rel_path} is not \
                         a string literal"
                    ),
                },
                _ => panic!(
                    "lens table reader: an element of `{data_name}` in {module_rel_path} is not a \
                     literal"
                ),
            }
        }
        if values.is_empty() && !allow_empty {
            panic!("lens table reader: `{data_name}` in {module_rel_path} is empty (fail-closed)");
        }
        return values;
    }
    panic!("lens table reader: no `data {data_name}` def in {module_rel_path}")
}

/// Read a `List<String>` data table from a live `.dag` lens authority on disk.
pub fn lens_string_list_data(
    module_rel_path: &str,
    data_name: &str,
    allow_empty: bool,
) -> Vec<String> {
    let path = workspace_root().join(module_rel_path);
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("lens table reader: failed to read {}: {e}", path.display()));
    string_list_data_from_module_source(module_rel_path, &content, data_name, allow_empty)
}

/// Project a `List<String>` data literal out of the ci_layer_roots authority's SOURCE TEXT via the
/// real front-end (`tokenize` + `parse`) — no second hand-rolled scanner.
pub(crate) fn string_list_data_from_ci_layer_roots_source(
    content: &str,
    data_name: &str,
) -> Vec<String> {
    string_list_data_from_module_source(CI_LAYER_ROOTS_AUTHORITY_REL, content, data_name, false)
}

/// Project the `witness_layer_roots` `List<String>` literal out of the ci_layer_roots authority.
pub(crate) fn witness_layer_roots_from_source(content: &str) -> Vec<String> {
    string_list_data_from_ci_layer_roots_source(content, WITNESS_LAYER_ROOTS_DATA_NAME)
}

/// Project the `witness_discovery_scan_dirs` `List<String>` literal out of the ci_layer_roots
/// authority.
pub(crate) fn witness_discovery_scan_dirs_from_source(content: &str) -> Vec<String> {
    string_list_data_from_ci_layer_roots_source(content, WITNESS_DISCOVERY_SCAN_DIRS_DATA_NAME)
}

/// Project `witness_exclusion_substrings` out of the ci_layer_roots authority.
pub(crate) fn witness_exclusion_substrings_from_source(content: &str) -> Vec<String> {
    string_list_data_from_ci_layer_roots_source(content, WITNESS_EXCLUSION_SUBSTRINGS_DATA_NAME)
}

/// Project `whole_tree_strict_resolve_exclusion_substrings` out of the ci_layer_roots authority.
pub(crate) fn whole_tree_strict_resolve_exclusion_substrings_from_source(
    content: &str,
) -> Vec<String> {
    string_list_data_from_ci_layer_roots_source(
        content,
        WHOLE_TREE_STRICT_RESOLVE_EXCLUSION_SUBSTRINGS_DATA_NAME,
    )
}

/// The witness layer roots, read live from the single .dag authority and memoized.
pub(crate) fn witness_layer_roots() -> Vec<String> {
    static ROOTS: OnceLock<Vec<String>> = OnceLock::new();
    ROOTS
        .get_or_init(|| witness_layer_roots_from_source(ci_layer_roots_authority_content()))
        .clone()
}

/// Witness discovery scan dirs, read live from `gunbc.ci_layer_roots.witness_discovery_scan_dirs`.
pub(crate) fn witness_discovery_scan_dirs() -> Vec<String> {
    static SCAN_DIRS: OnceLock<Vec<String>> = OnceLock::new();
    SCAN_DIRS
        .get_or_init(|| witness_discovery_scan_dirs_from_source(ci_layer_roots_authority_content()))
        .clone()
}

/// Floor discovery path exclusions — single authority `gunbc.ci_layer_roots.witness_exclusion_substrings`.
pub fn witness_exclusion_substrings() -> Vec<String> {
    static EXCLUDES: OnceLock<Vec<String>> = OnceLock::new();
    EXCLUDES
        .get_or_init(
            || witness_exclusion_substrings_from_source(ci_layer_roots_authority_content()),
        )
        .clone()
}

/// Whole-tree strict-resolve probe exclusions — `gunbc.ci_layer_roots.whole_tree_strict_resolve_exclusion_substrings`.
pub fn whole_tree_strict_resolve_exclusion_substrings() -> Vec<String> {
    static EXCLUDES: OnceLock<Vec<String>> = OnceLock::new();
    EXCLUDES
        .get_or_init(|| {
            whole_tree_strict_resolve_exclusion_substrings_from_source(
                ci_layer_roots_authority_content(),
            )
        })
        .clone()
}

/// Floor discovery ∪ whole-tree probe policy — `gunbc.ci_layer_roots.whole_tree_resolve_exclusion_substrings`.
pub fn whole_tree_resolve_exclusion_substrings() -> Vec<String> {
    let mut excludes = witness_exclusion_substrings();
    excludes.extend(whole_tree_strict_resolve_exclusion_substrings());
    excludes
}

/// Host census for `fn_arrow_decl_substrate_is_whole_tree` — eligible module count vs
/// `loaded` modules in the current resolve context (same exclude set as `whole_tree_resolved_ctx`).
pub fn fn_arrow_decl_substrate_is_whole_tree_for_census(loaded: usize) -> bool {
    let roots = default_source_roots();
    let excludes = whole_tree_resolve_exclusion_substrings();
    let index = build_module_path_index(&roots);
    let expected = index
        .iter()
        .filter(|(module_path, rel_path)| {
            !excludes
                .iter()
                .any(|sub| rel_path.contains(sub) || module_path.contains(sub))
        })
        .count();
    loaded >= expected
}

pub fn census_corpus_roots_follow_layer_authority() -> bool {
    let synthetic = "module gunbc.ci_layer_roots\n\n\
         data witness_layer_roots: List<String> = [\"alpha_layer_root\", \"beta_layer_root\", \"gamma_layer_root\"]\n";
    let follows = witness_layer_roots_from_source(synthetic)
        == ["alpha_layer_root", "beta_layer_root", "gamma_layer_root"];
    let live_nonempty = !witness_layer_roots().is_empty();
    follows && live_nonempty
}

pub(crate) fn default_source_roots() -> Vec<String> {
    let ws = workspace_root();
    witness_layer_roots()
        .iter()
        .map(|r| ws.join(r).to_string_lossy().into_owned())
        .collect()
}

pub fn build_module_path_index_from_witness_roots() -> HashMap<String, String> {
    build_module_path_index(&default_source_roots())
}

pub fn source_path_for_module_path(module_path: String) -> String {
    let index = build_module_path_index_from_witness_roots();
    index
        .get(&module_path)
        .cloned()
        .unwrap_or_else(|| panic!("module_path_index: unknown module path '{module_path}'"))
}

pub fn free_monoid_symbol_value_to_dotted_string(value: &v1_interpreter::Value) -> String {
    v1_interpreter::free_monoid_symbol_value_to_dotted_string(value)
}

pub fn free_monoid_symbol_value_from_dotted_string(
    ctx: &v1_interpreter::InterpContext,
    dotted: &str,
) -> v1_interpreter::Value {
    use v1_interpreter::{sorted_fields, Value};

    let fm_variant = |variant: &str, fields: Vec<_>| Value::Variant {
        type_name: ctx.sym("FreeMonoid"),
        variant_name: ctx.sym(variant),
        fields: Rc::new(fields),
    };
    if dotted.is_empty() {
        return fm_variant("Empty", vec![]);
    }
    let mut qn = fm_variant("Empty", vec![]);
    for seg in dotted.split('.').rev() {
        qn = fm_variant(
            "Cons",
            sorted_fields(vec![
                (ctx.sym("head"), Value::Str(seg.to_string())),
                (ctx.sym("tail"), qn),
            ]),
        );
    }
    qn
}

pub(crate) fn repo_rel(path: &Path) -> String {
    // Same out-of-tree fallback as the module index: corpus walks accept
    // caller-supplied roots (temp fixture trees, other checkouts), whose files key
    // by their absolute spelling; relative-not-under-root stays a refusal.
    module_index_path_key(path)
}

pub(crate) fn is_test_dag(path: &str) -> bool {
    path.ends_with("_test.dag")
}

pub(crate) fn corpus_dag_files() -> Vec<(String, String)> {
    let mut paths = Vec::new();
    for root in witness_layer_roots() {
        collect_dag_files_tolerant(&Path::new(&anchor_source_root(&root)), &mut paths);
    }
    let mut out = Vec::new();
    for p in paths {
        let rel = repo_rel(&p);
        if let Ok(content) = std::fs::read_to_string(&p) {
            out.push((rel, content));
        }
    }
    out.sort();
    out.dedup();
    out
}

pub(crate) fn strip_line_comment(line: &str) -> String {
    let bytes = line.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut in_string = false;
    let mut escaped = false;
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if in_string {
            if escaped {
                out.push(b' ');
                escaped = false;
            } else if b == b'\\' {
                out.push(b' ');
                escaped = true;
            } else if b == b'"' {
                out.push(b'"');
                in_string = false;
            } else {
                out.push(b' ');
            }
        } else if b == b'"' {
            in_string = true;
            out.push(b'"');
        } else if b == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
            break;
        } else {
            out.push(b);
        }
        i += 1;
    }
    String::from_utf8(out).expect("strip_line_comment output is valid UTF-8")
}

pub(crate) fn brace_delta(line: &str) -> i32 {
    let c = strip_line_comment(line);
    c.matches('{').count() as i32 - c.matches('}').count() as i32
}

type ModuleSourceIndex = HashMap<String, Rc<v1_compiler_compile::SourceFile>>;

fn build_module_index(source_roots: &[String]) -> ModuleSourceIndex {
    let mut index = ModuleSourceIndex::new();
    for (root_idx, root) in source_roots.iter().enumerate() {
        let anchored_root = anchor_source_root(root);
        let root_path = std::path::Path::new(&anchored_root);
        let mut dag_files = Vec::new();
        collect_dag_files(root_path, &mut dag_files);
        for path in dag_files {
            let content = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("failed to read {:?}: {}", path, e));
            if let Some(module_path) = extract_module_path(&content) {
                let rel_path = module_index_path_key(&path);
                let rel_forward = rel_path.clone();
                if manifest_stub_superseded_by_overlay(&rel_forward, source_roots, root_idx) {
                    continue;
                }
                if let Some(existing) = index.get(&module_path) {
                    if existing.path != rel_path && !same_canonical_file(&existing.path, &rel_path)
                    {
                        panic!(
                            "{}",
                            module_path_collision_panic_message(
                                &module_path,
                                &existing.path,
                                &rel_path,
                            )
                        );
                    }
                }
                index.insert(
                    module_path,
                    Rc::new(v1_compiler_compile::SourceFile {
                        path: rel_path,
                        content,
                    }),
                );
            }
        }
    }
    index
}

/// `primary-precedence` pool indexing: the first root is authoritative; later roots
/// fill only modules not already present (matches `gunbc compile --dependency-pool-index
/// primary-precedence` in `dag_compile_clean_transport`).
fn build_module_index_primary_precedence(source_roots: &[String]) -> ModuleSourceIndex {
    let mut index = ModuleSourceIndex::new();
    if source_roots.is_empty() {
        return index;
    }
    index_source_root_into_module_index(&source_roots[0], &mut index, false);
    for root in &source_roots[1..] {
        index_source_root_into_module_index(root, &mut index, true);
    }
    index
}

fn index_source_root_into_module_index(
    root: &str,
    index: &mut ModuleSourceIndex,
    pool_fill_only: bool,
) {
    let root_path = std::path::Path::new(root);
    if !root_path.exists() {
        panic!("source root does not exist: {}", root);
    }
    let mut dag_files = Vec::new();
    collect_dag_files(root_path, &mut dag_files);
    let mut within_root: HashMap<String, String> = HashMap::new();
    for path in dag_files {
        let content = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("failed to read {:?}: {}", path, e));
        if let Some(module_path) = extract_module_path(&content) {
            let rel_path = path.to_string_lossy().to_string();
            if pool_fill_only {
                if index.contains_key(&module_path) {
                    continue;
                }
            } else if let Some(existing) = index.get(&module_path) {
                if existing.path != rel_path {
                    panic!(
                        "module-path collision: module '{}' is declared by both '{}' and '{}' — one module, one authority (DESIGN §3); silent last-root-wins shadowing broke the floor (extdeps.shell, 2026-07-01) — de-fork or rename one side",
                        module_path, existing.path, rel_path
                    );
                }
            }
            if let Some(existing_path) = within_root.get(&module_path) {
                panic!(
                    "duplicate module path '{}' within source root '{}': declared in both '{}' and '{}'",
                    module_path, root, existing_path, rel_path
                );
            }
            within_root.insert(module_path.clone(), rel_path.clone());
            index.insert(
                module_path,
                Rc::new(v1_compiler_compile::SourceFile {
                    path: rel_path,
                    content,
                }),
            );
        }
    }
}

fn load_compile_clean_entry_sources(
    source_roots: &[String],
    mei: &MultiEntryIndex,
    entry_path_filter: Option<&std::collections::HashSet<String>>,
) -> Result<Vec<Rc<v1_compiler_compile::SourceFile>>, String> {
    let index = &mei.source_files;
    let facts = &mei.module_graph_facts;
    let first_root = std::path::Path::new(&source_roots[0]);
    let mut entry_files = Vec::new();
    if first_root.is_dir() {
        let mut dag_paths = Vec::new();
        collect_dag_files(first_root, &mut dag_paths);
        for path in dag_paths {
            let rel = workspace_relative_repo_path(&path.to_string_lossy());
            if let Some(filter) = entry_path_filter {
                if !filter.contains(&rel) {
                    continue;
                }
            }
            let content = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("failed to read {:?}: {}", path, e));
            entry_files.push((path.to_string_lossy().to_string(), content));
        }
    }

    let skipped_moduleless = moduleless_dag_entry_paths(&entry_files);
    report_moduleless_dag_entry_skips(&skipped_moduleless);

    let mut entry_for_queue = Vec::new();
    for (path, content) in &entry_files {
        if extract_module_path(content).is_some() {
            entry_for_queue.push(Rc::new(v1_compiler_compile::SourceFile {
                path: path.clone(),
                content: content.clone(),
            }));
        }
    }

    let mut sources = resolve_transitively(entry_for_queue, index, facts)?;
    for (path, content) in entry_files {
        if extract_module_path(&content).is_none() {
            continue;
        }
        if !sources.iter().any(|s| s.path == path) {
            sources.push(Rc::new(v1_compiler_compile::SourceFile { path, content }));
        }
    }
    // BOTH closures to a joint fixpoint via the ONE shared authority the witness
    // loader `load_sources_for_entry_with_pool` also calls (a §3 dissolution: this
    // gate loader previously ran ONLY `extend_with_reference_closure`, so an
    // affected entry reaching a provider purely through a bare name or a service
    // call — patterns.dag → `gcp.STS.Exchange`, no import — dropped that provider,
    // since the service-name → provider edge `gcp.STS` → dag/extdeps/cloud/gcp/sts.dag
    // lives ONLY in the bare closure, and its names went unresolved. Proven: ARM1
    // ref-only = 3 unresolved-type diags on patterns.dag's closure; +bare = 0).
    extend_sources_to_both_closure_fixpoint(sources, mei)
}

/// Reference-derived dependency closure (namespace Rule-1 interim). A qualified
/// reference `container.member` is a dependency edge exactly as an `import` line
/// was: with dag/ imports stripped, the import-edge closure alone silently drops
/// every module reached only by qualified reference (the referenced modules fall
/// out of the census and their qualified names refuse corpus-wide). Projection is
/// text-level longest-prefix against the declared module-path index, iterated to
/// fixpoint; each addition pulls its own import closure. The ONE closure authority
/// for both the whole-tree compile-clean walk and the per-entry claim/witness
/// loaders (a second closure rule would be a §3 fork). Dissolves into the
/// parsed-tree reference projection when the Rule-1 terminal step (import as
/// parse error, deps derived from references) lands.
fn extend_with_reference_closure(
    mut sources: Vec<Rc<v1_compiler_compile::SourceFile>>,
    index: &ModuleSourceIndex,
    facts: &ModuleGraphFactsLive,
) -> Result<Vec<Rc<v1_compiler_compile::SourceFile>>, String> {
    let path_lookup = path_to_source_lookup(index);
    let mut known_paths: std::collections::HashSet<String> = sources
        .iter()
        .flat_map(|s| [s.path.clone(), workspace_relative_repo_path(&s.path)])
        .collect();
    let mut scan_queue: Vec<Rc<v1_compiler_compile::SourceFile>> = sources.clone();
    while let Some(sf) = scan_queue.pop() {
        for module_path in referenced_module_paths_in_text(&sf.content, index) {
            let Some(dep) = index.get(&module_path) else {
                continue;
            };
            let dep_rel = workspace_relative_repo_path(&dep.path);
            if known_paths.contains(&dep_rel) || known_paths.contains(&dep.path) {
                continue;
            }
            if !facts.declares_repo_path(&dep_rel) {
                return Err(format!(
                    "reference_closure: referenced module '{module_path}' at '{dep_rel}' \
                     has no provenance in the module-graph facts pool (fail-closed)"
                ));
            }
            for path in import_closure_live_paths_with_facts(&dep_rel, facts) {
                let rel = workspace_relative_repo_path(&path);
                if known_paths.contains(&rel) {
                    continue;
                }
                let Some(dep_sf) = path_lookup.get(&rel).cloned() else {
                    return Err(format!(
                        "reference_closure: closure path '{rel}' (via referenced module \
                         '{module_path}') has no provenance in module index (fail-closed)"
                    ));
                };
                known_paths.insert(rel);
                known_paths.insert(dep_sf.path.clone());
                sources.push(dep_sf.clone());
                scan_queue.push(dep_sf);
            }
        }
    }
    Ok(sources)
}

/// Candidate module paths referenced by dotted names in `content`: every maximal
/// `seg(.seg)+` identifier chain contributes its longest leading prefix that is a
/// declared module path in `index` (>= 2 segments — a bare single identifier is a
/// global-bare census reference, never a module projection). String literals are
/// skipped: a module path inside a string is data (a registry row, prose), not a
/// reference, and following it over-pulls modules the corpus never resolves against.
fn referenced_module_paths_in_text(content: &str, index: &ModuleSourceIndex) -> Vec<String> {
    let bytes = content.as_bytes();
    let is_ident_start = |c: u8| c.is_ascii_alphabetic() || c == b'_';
    let is_ident = |c: u8| c.is_ascii_alphanumeric() || c == b'_';
    let mut out = std::collections::BTreeSet::new();
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'"' {
            i += 1;
            while i < bytes.len() && bytes[i] != b'"' {
                if bytes[i] == b'\\' && i + 1 < bytes.len() {
                    i += 1;
                }
                i += 1;
            }
            i += 1;
            continue;
        }
        if !is_ident_start(bytes[i]) || (i > 0 && (is_ident(bytes[i - 1]) || bytes[i - 1] == b'.'))
        {
            i += 1;
            continue;
        }
        let start = i;
        let mut segment_ends: Vec<usize> = Vec::new();
        loop {
            while i < bytes.len() && is_ident(bytes[i]) {
                i += 1;
            }
            segment_ends.push(i);
            if i + 1 < bytes.len() && bytes[i] == b'.' && is_ident_start(bytes[i + 1]) {
                i += 1;
            } else {
                break;
            }
        }
        if segment_ends.len() >= 2 {
            for k in (2..=segment_ends.len()).rev() {
                let candidate = &content[start..segment_ends[k - 1]];
                if index.contains_key(candidate) {
                    out.insert(candidate.to_string());
                    break;
                }
            }
        }
    }
    out.into_iter().collect()
}

fn compile_clean_resolve_has_hard_errors(
    result: &v1_compiler_compile::ResolvedPipelineResult,
) -> bool {
    compile_clean_pipeline_has_hard_errors(result.diagnostics.as_ref())
}

// Single authority (DESIGN.md §3/§7): whether a diagnostic blocks is decided by
// `00_core.dag`, never restated here. `is_interpreter_blocking_diagnostic` is the
// {ComplexityUnknown, UnlistedImportUse} tolerance this gate has always intended;
// the prior hand-rolled `!matches!(ComplexityUnknown)` predated UnlistedImportUse's
// demotion to advisory and silently reded the namespace import strip.
fn compile_clean_diagnostic_is_hard(d: &Rc<ErrorNode>) -> bool {
    crate::v1_std_core::is_interpreter_blocking_diagnostic(d.diagnostic.clone())
}

pub fn compile_clean_pipeline_has_hard_errors(diagnostics: &im::Vector<Rc<ErrorNode>>) -> bool {
    diagnostics.iter().any(compile_clean_diagnostic_is_hard)
}

fn eprint_compile_clean_hard_diagnostics(diagnostics: &im::Vector<Rc<ErrorNode>>) {
    const SHOWN_LIMIT: usize = 20;
    let mut shown = 0usize;
    let mut total = 0usize;
    // One pass, no accumulator: the total is counted, never collected (§6).
    for d in diagnostics
        .iter()
        .filter(|d| compile_clean_diagnostic_is_hard(d))
    {
        total += 1;
        if shown < SHOWN_LIMIT {
            eprintln!(
                "compile-clean: {}",
                diagnostic_to_message(d.diagnostic.clone())
            );
            shown += 1;
        }
    }
    if total > SHOWN_LIMIT {
        // Count the residue rather than hiding it (§5): a truncated burndown that
        // never reports its size makes the deficit unprioritizable.
        eprintln!("compile-clean: (truncated hard diagnostics at {SHOWN_LIMIT}; {total} total)");
    }
}

const COMPILE_CLEAN_SCOPE_ENTRY: &str = "dag/tools/dag_compile_clean_scope.dag";

#[derive(Debug, Clone, PartialEq, Eq)]
enum CompileCleanScopePlan {
    /// Local dev only — neither `GITHUB_ACTIONS` nor `GUNBC_CI_DIFF_BASE` active.
    WholeTree,
    SkipNoAffected {
        reason: String,
    },
    Scoped {
        entry_paths: Vec<String>,
    },
    /// CI path: diff observation or scope disposition failed — job must red (no widening).
    Refused {
        reason: String,
    },
}

fn compile_clean_scope_plan_from_touched_paths(
    touched_paths: &[String],
    departed_paths: &HashSet<String>,
) -> Result<CompileCleanScopePlan, String> {
    let roots = default_source_roots();
    let (graph, indices) = resolve_entry_graph_shared(&roots, COMPILE_CLEAN_SCOPE_ENTRY)
        .map_err(|e| format!("dag_compile_clean_scope resolve: {e}"))?;
    let ctx = make_eval_context(&graph, indices, v1_interpreter::ExecutionMode::Wet);
    let paths: Vec<v1_interpreter::Value> = touched_paths
        .iter()
        .map(|s| v1_interpreter::Value::Str(s.clone()))
        .collect();
    let mut departed_sorted: Vec<&String> = departed_paths.iter().collect();
    departed_sorted.sort();
    let departed: Vec<v1_interpreter::Value> = departed_sorted
        .into_iter()
        .map(|s| v1_interpreter::Value::Str(s.clone()))
        .collect();
    let args = [
        (
            Some("touched_paths".to_string()),
            list_value_from_vec(paths),
        ),
        (
            Some("departed_paths".to_string()),
            list_value_from_vec(departed),
        ),
    ];
    let result = v1_interpreter::run_in_context_with_args(
        &ctx,
        "compile_clean_scope_disposition_from_diff",
        &args,
        false,
    )
    .map_err(|e| format!("compile_clean_scope_disposition_from_diff: {e}"))?;
    match &result {
        v1_interpreter::Value::Variant {
            variant_name,
            fields,
            ..
        } if ctx.sym_eq(*variant_name, "ScopedRun") => {
            let entry_paths = match ctx.field(fields, "entry_paths") {
                Some(v) => string_list_from_value(v, "entry_paths")?,
                None => return Err("ScopedRun missing `entry_paths`".to_string()),
            };
            Ok(CompileCleanScopePlan::Scoped { entry_paths })
        }
        v1_interpreter::Value::Variant {
            variant_name,
            fields,
            ..
        } if ctx.sym_eq(*variant_name, "SkipNoAffectedEntries") => {
            let reason = match ctx.field(fields, "reason") {
                Some(v1_interpreter::Value::Str(r)) => r.clone(),
                _ => "no compile-clean entry affected".to_string(),
            };
            Ok(CompileCleanScopePlan::SkipNoAffected { reason })
        }
        v1_interpreter::Value::Variant {
            variant_name,
            fields,
            ..
        } if ctx.sym_eq(*variant_name, "RequireWholeTree") => {
            let reason = match ctx.field(fields, "reason") {
                Some(v1_interpreter::Value::Str(r)) => r.clone(),
                _ => "whole-tree baseline required".to_string(),
            };
            eprintln!("compile-clean scope: {reason}");
            Ok(CompileCleanScopePlan::WholeTree)
        }
        other => Err(format!(
            "compile_clean_scope_disposition_from_diff returned `{}`, expected ScopedRun | SkipNoAffectedEntries | RequireWholeTree",
            ctx.format_value(other)
        )),
    }
}

/// `gunbc.ci_layer_roots.compile_clean_source_roots` — witness pool + `src/v1` for cross-tree
/// import resolution in compile-clean scope disposition (not the gate receipt pool).
fn compile_clean_source_roots() -> Vec<String> {
    let mut roots = witness_layer_roots();
    if !roots.iter().any(|r| r == "src/v1") {
        roots.push("src/v1".to_string());
    }
    roots
}

fn compile_clean_touched_path_norm(path: &str) -> &str {
    path.strip_prefix("./").unwrap_or(path)
}

fn compile_clean_all_touched_paths_docs_universe(touched_paths: &[String]) -> bool {
    !touched_paths.is_empty()
        && touched_paths
            .iter()
            .all(|p| compile_clean_touched_path_norm(p).starts_with("docs/"))
}

/// Host realization of `tools.dag_compile_clean_shard_roster.compile_clean_shard_entry_paths`
/// without resolving `dag_compile_clean_scope.dag` (the interpreter path cold-scans ~minutes).
fn compile_clean_shard_entry_paths_fast() -> Vec<String> {
    let entry_root = witness_layer_roots()
        .first()
        .cloned()
        .unwrap_or_else(|| "dag".to_string());
    let abs_entry_root = anchor_source_root(&entry_root);
    let mut paths: Vec<String> = module_declaration_facts(&[abs_entry_root])
        .into_iter()
        .map(|decl| workspace_relative_repo_path(&decl.path))
        .collect();
    paths.sort();
    paths.dedup();
    paths
}

/// Mirror of `tools.dag_compile_clean_scope.compile_clean_touched_path_selectable`:
/// import-closure selection can only answer for `.dag` sources (and the docs universe);
/// any other touched path (compiler seed `.rs`, workflow yml, manifests) can change
/// compile behavior outside the module graph, so the gate keeps its whole-tree baseline.
fn compile_clean_touched_path_selectable(path: &str) -> bool {
    let norm = path.strip_prefix("./").unwrap_or(path);
    norm.starts_with("docs/") || norm.ends_with(".dag")
}

/// Mirror of `tools.dag_compile_clean_scope.compile_clean_departed_paths_outside_docs`:
/// a departed (deleted / renamed-from) non-docs path is invisible to current-tree
/// adjacency — a dangling import drops the edge, so a broken importer would not be
/// selected — whole-tree baseline.
fn compile_clean_departed_paths_outside_docs(departed_paths: &HashSet<String>) -> bool {
    departed_paths.iter().any(|p| {
        let norm = p.strip_prefix("./").unwrap_or(p);
        !norm.starts_with("docs/")
    })
}

/// Floor CI hot path: mirrors `compile_clean_scope_disposition_from_diff`
/// (`tools.dag_compile_clean_scope`, module-graph import-closure grain — channel 2 of
/// operator fork (c) 2026-07-10) without the Wet interpreter fold over
/// `compile_clean_shard_entry_paths()`. Selection reuses the SAME certified realization
/// as the discovery-corpus channel (`entry_file_touched_via_import_closure`); every arm
/// that cannot answer falls back to the gate's whole-tree baseline, loudly.
fn compile_clean_scope_plan_from_touched_paths_floor_fast(
    touched_paths: &[String],
    departed_paths: &HashSet<String>,
) -> CompileCleanScopePlan {
    if touched_paths.is_empty() {
        return CompileCleanScopePlan::SkipNoAffected {
            reason: "no touched paths in diff observation".to_string(),
        };
    }

    if compile_clean_all_touched_paths_docs_universe(touched_paths) {
        let reason =
            "docs-only diff — no compile-clean entry selection required (Ruling 1 path grain)"
                .to_string();
        eprintln!("compile-clean scope: skipped ({reason})");
        return CompileCleanScopePlan::SkipNoAffected { reason };
    }

    if let Some(outside) = touched_paths
        .iter()
        .find(|p| !compile_clean_touched_path_selectable(p))
    {
        eprintln!(
            "compile-clean scope: touched path outside the selectable universe ({outside}) — compiler/infra change, whole-tree baseline"
        );
        return CompileCleanScopePlan::WholeTree;
    }

    if compile_clean_departed_paths_outside_docs(departed_paths) {
        eprintln!(
            "compile-clean scope: departed non-docs path in diff (deletion/rename) — whole-tree baseline"
        );
        return CompileCleanScopePlan::WholeTree;
    }

    let pool_roots = compile_clean_source_roots();
    let facts = build_module_graph_facts_live(&pool_roots);
    let declared_paths = facts.declared_repo_paths();
    let mut affected = Vec::new();
    for entry_path in compile_clean_shard_entry_paths_fast() {
        match entry_file_touched_via_import_closure(
            &entry_path,
            &facts,
            &declared_paths,
            touched_paths,
        ) {
            Ok(true) => affected.push(entry_path),
            Ok(false) => {}
            Err(msg) => {
                eprintln!("compile-clean scope: {msg} — whole-tree baseline");
                return CompileCleanScopePlan::WholeTree;
            }
        }
    }
    if !affected.is_empty() {
        eprintln!(
            "compile-clean scope: {} affected entr{} (floor fast path)",
            affected.len(),
            if affected.len() == 1 { "y" } else { "ies" }
        );
        return CompileCleanScopePlan::Scoped {
            entry_paths: affected,
        };
    }
    eprintln!(
        "compile-clean scope: non-empty diff with no shard intersection — whole-tree baseline"
    );
    CompileCleanScopePlan::WholeTree
}

fn compile_clean_scoping_active() -> bool {
    FLOOR_COMPILE_CLEAN_CI_SCOPING.load(Ordering::SeqCst)
        || std::env::var("GUNBC_CI_DIFF_BASE").is_ok()
        || std::env::var("GITHUB_ACTIONS")
            .map(|v| v == "true")
            .unwrap_or(false)
        || std::env::var("CI").map(|v| v == "true").unwrap_or(false)
}

pub const DOCUMENTATION_ONLY_FLOOR_SKIP_LABEL: &str = "documentation_only_skip";
pub const RUN_FULL_FLOOR_LABEL: &str = "run_full_floor";

/// CI floor admission label for the docs-only witness-corpus skip arm.
/// Uses `tools.dag_compile_clean_scope` at Ruling 1 path grain (host fast path).
/// Empty diff or diff-observation failure returns `run_full_floor` (fail-closed).
/// Docs-only (`docs/**` universe, aligned with doc_reachability) skips without
/// waiting on #6239 substrate — the witness runs before claim_executor warms facts.
pub fn documentation_only_floor_skip_label_for_ci() -> String {
    if !compile_clean_scoping_active() {
        return RUN_FULL_FLOOR_LABEL.to_string();
    }
    match floor_git_diff_name_status_range() {
        Err(msg) => {
            eprintln!(
                "documentation-only floor skip: diff observation failed ({msg}) — full floor"
            );
            RUN_FULL_FLOOR_LABEL.to_string()
        }
        Ok((changed_paths, _departed)) => {
            if changed_paths.is_empty() {
                eprintln!("documentation-only floor skip: empty diff — full floor");
                return RUN_FULL_FLOOR_LABEL.to_string();
            }
            if compile_clean_all_touched_paths_docs_universe(&changed_paths) {
                eprintln!(
                    "documentation-only floor skip: docs-only diff — no compile-clean entry selection required (Ruling 1 path grain)"
                );
                DOCUMENTATION_ONLY_FLOOR_SKIP_LABEL.to_string()
            } else {
                eprintln!("documentation-only floor skip: full floor (non-docs-only diff)");
                RUN_FULL_FLOOR_LABEL.to_string()
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Regen (self-host fixed-point) affected-set scoping.
//
// `regen_stage0` compiles `[src/v1, dag]` into the committed stage0 seed; both the
// RegenVerifyGate and the SelfHostStalenessGate compare that emit against the
// committed crate. The emit is a pure function of exactly one input set: every
// `src/v1/**.dag` entry plus its transitive `import` closure through `[src/v1, dag]`
// (`regen_input_sources` below — the single authority `regen_stage0` also consumes),
// the emitter binary (all `src/v1/**` Rust, covered by the path prefix), the committed
// stage0 outputs (all written under `src/v1/stage0/src`, same prefix), and the Cargo
// manifest/lockfile. A PR whose diff touches none of those provably cannot change the
// regen outcome, so the regen CI step can skip. Fail-closed: any uncertainty runs.
// Main pushes and the 4-hourly falsifier run regen unconditionally as the cold control.
// ---------------------------------------------------------------------------

pub const REGEN_NOT_AFFECTED_SKIP_LABEL: &str = "regen_not_affected_skip";
pub const RUN_REGEN_LABEL: &str = "run_regen";

/// Relativize an absolute source path against the workspace root for regen display /
/// closure identity. Single authority: consumed by `regen_stage0` and the skip witness.
pub fn regen_workspace_relpath(path: &Path, workspace: &Path) -> String {
    path.strip_prefix(workspace)
        .unwrap_or(path)
        .to_string_lossy()
        .to_string()
}

// Distinct from `collect_dag_files` above (which panics on IO error and skips cargo
// `target/` output dirs): regen needs the fail-closed Result variant and the exact
// whole-tree walk `regen_stage0` has always used, so the closure stays byte-identical
// to the committed seed (guarded by `regen_stage0 --verify`).
fn regen_collect_dag_files(dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    let mut entries: Vec<_> = std::fs::read_dir(dir)
        .map_err(|e| format!("read dir {}: {e}", dir.display()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("read dir entry in {}: {e}", dir.display()))?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            regen_collect_dag_files(&path, files)?;
        } else if path.extension().is_some_and(|ext| ext == "dag") {
            files.push(path);
        }
    }
    Ok(())
}

fn regen_build_module_index(
    roots: &[PathBuf],
) -> Result<std::collections::HashMap<String, PathBuf>, String> {
    let mut index: std::collections::HashMap<String, PathBuf> = std::collections::HashMap::new();
    for root in roots {
        if !root.exists() {
            return Err(format!("source root does not exist: {}", root.display()));
        }
        let mut dag_paths = Vec::new();
        regen_collect_dag_files(root, &mut dag_paths)?;
        for path in dag_paths {
            let content = std::fs::read_to_string(&path)
                .map_err(|e| format!("read {}: {e}", path.display()))?;
            if let Some(module_path) = extract_module_path(&content) {
                if let Some(existing) = index.get(&module_path) {
                    return Err(format!(
                        "duplicate module path `{module_path}`: {} and {}",
                        existing.display(),
                        path.display()
                    ));
                }
                index.insert(module_path, path);
            }
        }
    }
    Ok(index)
}

/// The two source roots the self-host regen compiles: `src/v1` entry seeds and `dag`
/// (the import-resolution index). Single authority for both `regen_stage0` and the
/// regen affected-set skip witness.
pub fn regen_source_roots(workspace: &Path) -> Vec<PathBuf> {
    vec![workspace.join("src/v1"), workspace.join("dag")]
}

/// Every `.dag` source `regen_stage0` reads to emit the stage0 seed: all `src/v1/**.dag`
/// entries plus their transitive `import` closure through `[src/v1, dag]`, returned as
/// sorted `(workspace-relpath, content)`. This IS the set `regen_stage0` compiles — it
/// consumes this function — so the regen compile and the skip witness share one closure
/// authority (no forked "what regen reads"). The dedup is by module path, mirroring the
/// original seed-collection semantics; `regen_stage0 --verify` is the byte-identical
/// oracle guarding that equivalence.
pub fn regen_input_sources(workspace: &Path) -> Result<Vec<(String, String)>, String> {
    let roots = regen_source_roots(workspace);
    let index = regen_build_module_index(&roots)?;
    let entry_root = roots
        .first()
        .ok_or_else(|| "regen source root list must not be empty".to_string())?;
    let mut entry_paths = Vec::new();
    regen_collect_dag_files(entry_root, &mut entry_paths)?;

    // module_path -> (relpath, content); the first occurrence of a module path wins, so
    // src/v1 entry seeds and their import closure resolve to exactly the set regen emits.
    let mut seen: std::collections::HashMap<String, (String, String)> =
        std::collections::HashMap::new();
    let mut queue: Vec<String> = Vec::new(); // module contents whose imports remain to walk
    for path in &entry_paths {
        let content =
            std::fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
        let rel = regen_workspace_relpath(path, workspace);
        if let Some(module_path) = extract_module_path(&content) {
            seen.insert(module_path, (rel, content.clone()));
        }
        queue.push(content);
    }
    while let Some(content) = queue.pop() {
        for module_path in extract_import_paths(&content) {
            if seen.contains_key(&module_path) {
                continue;
            }
            if let Some(file_path) = index.get(&module_path) {
                let file_content = std::fs::read_to_string(file_path)
                    .map_err(|e| format!("read imported module {}: {e}", file_path.display()))?;
                let rel = regen_workspace_relpath(file_path, workspace);
                seen.insert(module_path, (rel, file_content.clone()));
                queue.push(file_content);
            }
        }
    }
    let mut result: Vec<(String, String)> = seen.into_values().collect();
    result.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(result)
}

/// Does a diff-changed path belong to the regen input surface (fail-closed superset)?
fn regen_path_affects_regen(changed: &str, dag_closure: &HashSet<String>) -> bool {
    let p = normalize_repo_path(changed);
    // src/v1/** = the emitter binary source (.rs), every committed stage0 output
    // (all under src/v1/stage0/src), and the src/v1 .dag entry seeds.
    if p.starts_with("src/v1/") {
        return true;
    }
    // Cargo/toolchain build config: the emitter binary is built from these; a
    // dependency, pinned-toolchain, or cargo-config change could in principle alter
    // emitted bytes. Rare in practice; fail-closed (whole-file matches, no substring).
    if p == "Cargo.lock"
        || p == "Cargo.toml"
        || p.ends_with("/Cargo.toml")
        || p == "rust-toolchain.toml"
        || p == "rust-toolchain"
        || p == ".cargo/config.toml"
        || p == ".cargo/config"
    {
        return true;
    }
    // dag/** files in v1's transitive import closure.
    dag_closure.contains(&p)
}

/// CI label for the regen self-host fixed-point step's affected-set skip arm.
/// `regen_not_affected_skip` iff the merge-base diff touches no regen input;
/// `run_regen` on any intersection, empty diff, departed non-docs path, or
/// observation/closure failure (fail-closed). This computes the label only; the CI
/// shell (ci_spec.dag) gates the skip to pull_request events, so push-to-main runs
/// regen unconditionally as the cold control that surfaces a wrong closure on the
/// next merge.
pub fn regen_floor_skip_label_for_ci() -> String {
    let (changed_paths, departed_paths) = match floor_git_diff_name_status_range() {
        Ok(v) => v,
        Err(msg) => {
            eprintln!("regen floor skip: diff observation failed ({msg}) — run regen");
            return RUN_REGEN_LABEL.to_string();
        }
    };
    if changed_paths.is_empty() {
        eprintln!("regen floor skip: empty diff — run regen (fail-closed cold control)");
        return RUN_REGEN_LABEL.to_string();
    }
    // Departed (deleted / renamed-from) non-docs paths: the closure below is computed
    // from the CURRENT tree, so a deleted `.dag` file that WAS in the regen closure is
    // invisible to it — the intersection test would skip a diff that provably changes
    // the fresh emit (the deleted module no longer contributes; its importers now fail
    // to resolve). Same guard shape as compile-clean's departed arm: run, never skip.
    if let Some(gone) = departed_paths.iter().find(|p| {
        let n = normalize_repo_path(p);
        !n.starts_with("docs/")
    }) {
        eprintln!(
            "regen floor skip: departed non-docs path in diff ({}) — run regen (current-tree closure cannot see deletions)",
            normalize_repo_path(gone)
        );
        return RUN_REGEN_LABEL.to_string();
    }
    let workspace = workspace_root();
    let dag_closure: HashSet<String> = match regen_input_sources(&workspace) {
        Ok(sources) => sources
            .into_iter()
            .map(|(p, _)| normalize_repo_path(&p))
            .collect(),
        Err(msg) => {
            eprintln!("regen floor skip: input-closure computation failed ({msg}) — run regen");
            return RUN_REGEN_LABEL.to_string();
        }
    };
    match changed_paths
        .iter()
        .find(|p| regen_path_affects_regen(p, &dag_closure))
    {
        Some(example) => {
            eprintln!(
                "regen floor skip: diff intersects regen inputs (e.g. {}) — run regen",
                normalize_repo_path(example)
            );
            RUN_REGEN_LABEL.to_string()
        }
        None => {
            eprintln!(
                "regen floor skip: {} changed path(s), none intersect the regen input closure (src/v1/** ∪ v1 dag import-closure ∪ Cargo/toolchain config) — self-host fixed-point provably unchanged (push-to-main runs regen unconditionally as the cold control)",
                changed_paths.len()
            );
            REGEN_NOT_AFFECTED_SKIP_LABEL.to_string()
        }
    }
}

fn compile_clean_scope_plan_for_ci() -> CompileCleanScopePlan {
    // Falsifier cold-control arm: force the whole-tree compile before any diff observation.
    // Widen-to-more-checking only — this env can never skip or narrow the gate, so it is a
    // control, not an escape hatch (the deterministic whole-tree counterpart to the scoped
    // per-PR admission, on the falsifier cadence).
    if std::env::var("GUNBC_CI_COMPILE_CLEAN_COLD_CONTROL")
        .map(|v| v == "1")
        .unwrap_or(false)
    {
        eprintln!(
            "compile-clean scope: whole-tree cold control forced (GUNBC_CI_COMPILE_CLEAN_COLD_CONTROL=1)"
        );
        return CompileCleanScopePlan::WholeTree;
    }
    if !compile_clean_scoping_active() {
        eprintln!("compile-clean scope: whole-tree (ci diff scoping inactive)");
        return CompileCleanScopePlan::WholeTree;
    }
    match floor_git_diff_name_status_range() {
        Ok((changed_paths, departed_paths)) => {
            if FLOOR_COMPILE_CLEAN_CI_SCOPING.load(Ordering::SeqCst) {
                return compile_clean_scope_plan_from_touched_paths_floor_fast(
                    &changed_paths,
                    &departed_paths,
                );
            }
            match compile_clean_scope_plan_from_touched_paths(&changed_paths, &departed_paths) {
                Ok(plan) => plan,
                Err(msg) => CompileCleanScopePlan::Refused {
                    reason: format!("compile-clean scope disposition failed: {msg}"),
                },
            }
        }
        Err(msg) => CompileCleanScopePlan::Refused {
            reason: format!("diff observation failed: {msg}"),
        },
    }
}

fn witness_layer_roots_compile_clean_sources_for_plan(
    plan: &CompileCleanScopePlan,
) -> Result<Option<Vec<Rc<v1_compiler_compile::SourceFile>>>, String> {
    match plan {
        CompileCleanScopePlan::Refused { reason } => {
            eprintln!("compile-clean scope: refused ({reason})");
            Err(reason.clone())
        }
        CompileCleanScopePlan::SkipNoAffected { reason } => {
            eprintln!("compile-clean scope: skipped ({reason})");
            Ok(None)
        }
        CompileCleanScopePlan::WholeTree => {
            eprintln!("compile-clean scope: whole-tree entry closure (witness_layer_roots)");
            let roots = witness_layer_roots();
            let mei = build_multi_entry_index_primary_precedence(&roots);
            load_compile_clean_entry_sources(&roots, &mei, None).map(|mut sources| {
                append_test_floor_compile_clean_inject(&mut sources);
                Some(sources)
            })
        }
        CompileCleanScopePlan::Scoped { entry_paths } => {
            eprintln!(
                "compile-clean scope: {} affected entr{} (of whole-tree gate)",
                entry_paths.len(),
                if entry_paths.len() == 1 { "y" } else { "ies" }
            );
            let filter: std::collections::HashSet<String> = entry_paths
                .iter()
                .map(|p| workspace_relative_repo_path(p))
                .collect();
            let roots = witness_layer_roots();
            let mei = build_multi_entry_index_primary_precedence(&roots);
            load_compile_clean_entry_sources(&roots, &mei, Some(&filter)).map(Some)
        }
    }
}

/// Test-only inject: append an unresolved-import module to the compile-clean closure so
/// `install_floor_compile_clean_receipt` + `consume_floor_compile_clean_gate_verdict` can be
/// proven end-to-end (§5 discriminating RED) without mutating the workspace tree.
fn append_test_floor_compile_clean_inject(sources: &mut Vec<Rc<v1_compiler_compile::SourceFile>>) {
    if std::env::var("GUNBC_TEST_FLOOR_COMPILE_CLEAN_INJECT_UNRESOLVED")
        .map(|v| v == "1")
        .unwrap_or(false)
    {
        sources.push(Rc::new(v1_compiler_compile::SourceFile {
            path: "dag/test/fixture/lever_a_e2e_unresolved_inject.dag".to_string(),
            content: "module test.lever_a_e2e_unresolved_inject\nimport totally.nonexistent.lever_a_module { Foo }\nfn probe() -> Int { 42 }".to_string(),
        }));
    }
}

/// In-run receipt for the floor's ONE whole-tree `--target dag` compile (Lever A / §2 Share).
/// `claim_executor` installs this before batch-1; `dag_compile_clean_gate` consumes it only —
/// a second compile in the gate path is unwritable (§5).
#[derive(Debug, Clone, PartialEq, Eq)]
enum FloorCompileCleanReceipt {
    Skipped { reason: String },
    Refused { reason: String },
    Compiled { ok: bool },
}

static FLOOR_COMPILE_CLEAN_RECEIPT: Mutex<Option<FloorCompileCleanReceipt>> = Mutex::new(None);

/// When set by `claim_executor` for `gunbc_ci_floor_batches`, the first gate consume installs
/// the one whole-tree receipt (after plan resolve has warmed the module-graph facts cache).
static FLOOR_COMPILE_CLEAN_LAZY_INSTALL: AtomicBool = AtomicBool::new(false);
/// Floor CI runs through `claim_executor`; env-based scoping detection alone missed some
/// self-hosted runners (silent whole-tree source load → step timeout). Tied to lazy install.
static FLOOR_COMPILE_CLEAN_CI_SCOPING: AtomicBool = AtomicBool::new(false);
/// The executor's `--source-root` vector, stored at lazy-install arm time so the
/// receipt compile builds/reuses the SAME `process_shared_index` (same roots key) the
/// plan prelude and batch-2 witness resolves use — one typed-module universe per run
/// (the in-process double-pay kill, typecheck investigation PR #6766). `None` when the
/// gate was never armed; the receipt then REFUSES (typed reason), never falls back to
/// a second raw whole-tree compile (§5 — no silent widen).
static FLOOR_COMPILE_CLEAN_INDEX_ROOTS: Mutex<Option<Vec<String>>> = Mutex::new(None);

pub fn enable_floor_compile_clean_lazy_install(source_roots: &[String]) {
    FLOOR_COMPILE_CLEAN_LAZY_INSTALL.store(true, Ordering::SeqCst);
    FLOOR_COMPILE_CLEAN_CI_SCOPING.store(true, Ordering::SeqCst);
    *FLOOR_COMPILE_CLEAN_INDEX_ROOTS
        .lock()
        .expect("floor compile-clean index-roots lock poisoned") = Some(source_roots.to_vec());
}

#[cfg(test)]
fn disable_floor_compile_clean_lazy_install_for_test() {
    FLOOR_COMPILE_CLEAN_LAZY_INSTALL.store(false, Ordering::SeqCst);
    FLOOR_COMPILE_CLEAN_CI_SCOPING.store(false, Ordering::SeqCst);
    *FLOOR_COMPILE_CLEAN_INDEX_ROOTS
        .lock()
        .expect("floor compile-clean index-roots lock poisoned") = None;
}

/// Raw-pipeline `--target dag` compile-clean over a source set. NOT the floor's
/// receipt path (that is `floor_compile_clean_emit_ok_via_index`, which shares the
/// process's typed-module universe) — this is the index-independent oracle behind
/// `witness_layer_roots_compile_clean_emit_check` (cargo tests, enrolled witnesses),
/// and precisely BECAUSE it shares no caches with the via-index path it is the
/// standing second opinion for verdict equivalence
/// (`compile_clean_via_index_verdict_equivalence` tests).
fn floor_compile_clean_emit_ok(sources: Vec<Rc<v1_compiler_compile::SourceFile>>) -> bool {
    use crate::v1_compiler_artifact::RenderTarget;
    let result = v1_compiler_compile::compile_sources(Rc::new(sources.into()), RenderTarget::Dag);
    let has_hard_errors = compile_clean_pipeline_has_hard_errors(result.diagnostics.as_ref());
    if has_hard_errors {
        eprint_compile_clean_hard_diagnostics(result.diagnostics.as_ref());
    } else if result.files.is_empty() {
        eprintln!("floor compile-clean: refused — compile produced zero files (empty emit set)");
    }
    !has_hard_errors && !result.files.is_empty()
}

/// The floor receipt's compile: the same source closure as the raw oracle, routed
/// through the shared `MultiEntryIndex` cached path (`resolved_graph_from_sources_with_index`)
/// so every module's content-keyed typecheck is computed once per process and reused
/// by batch-2's witness resolves (PR #6766 receipts: verdict-equivalent green AND on
/// the planted `GUNBC_TEST_FLOOR_COMPILE_CLEAN_INJECT_UNRESOLVED` red; warm heavy
/// witnesses 1.0–1.4s vs 10–90s cold; red verdict at the failing stage in seconds).
/// The emit leg (`--target dag` render) runs over the already-typed graph.
fn floor_compile_clean_emit_ok_via_index(
    sources: Vec<Rc<v1_compiler_compile::SourceFile>>,
    index_roots: &[String],
) -> bool {
    use crate::v1_compiler_artifact::RenderTarget;
    use crate::v1_compiler_complexity::empty_complexity_report;
    let index = process_shared_index(index_roots);
    let (graph, si) = match resolved_graph_from_sources_with_index(
        &index,
        sources,
        ResolveTypecheckGate::Strict,
        "floor-compile-clean-gate",
    ) {
        Ok(resolved) => resolved,
        Err(msg) => {
            eprintln!("compile-clean: hard diagnostics:\n{msg}");
            return false;
        }
    };
    let newline_indices: Rc<im::Vector<Rc<NewlineIndex>>> =
        Rc::new(si.values().cloned().collect::<im::Vector<_>>());
    let resolved = Rc::new(v1_compiler_compile::ResolvedPipelineResult {
        graph: Some(graph),
        diagnostics: Rc::new(im::Vector::new()),
        source_indices: si,
        complexity: empty_complexity_report(),
        ownership: Rc::new(im::Vector::new()),
        newline_indices,
    });
    let result = v1_compiler_compile::emit_resolved_for_target(resolved, RenderTarget::Dag);
    if result.files.is_empty() {
        eprintln!("floor compile-clean: refused — compile produced zero files (empty emit set)");
        return false;
    }
    true
}

fn produce_floor_compile_clean_receipt() -> FloorCompileCleanReceipt {
    // Index roots are armed by `enable_floor_compile_clean_lazy_install`; a receipt
    // demanded without them is a wiring defect and REFUSES (typed reason) — never a
    // silent fallback to a second raw whole-tree compile (§5).
    let index_roots = match FLOOR_COMPILE_CLEAN_INDEX_ROOTS.lock() {
        Ok(guard) => match guard.as_ref() {
            Some(roots) => roots.clone(),
            None => {
                return FloorCompileCleanReceipt::Refused {
                    reason: "compile-clean receipt demanded with no index roots armed \
                             (enable_floor_compile_clean_lazy_install arms them)"
                        .to_string(),
                }
            }
        },
        Err(e) => {
            return FloorCompileCleanReceipt::Refused {
                reason: format!("floor compile-clean index-roots lock poisoned: {e}"),
            }
        }
    };
    match witness_layer_roots_compile_clean_sources_for_plan(&compile_clean_scope_plan_for_ci()) {
        Ok(None) => FloorCompileCleanReceipt::Skipped {
            reason: "no compile-clean entry affected".to_string(),
        },
        Err(msg) => FloorCompileCleanReceipt::Refused { reason: msg },
        Ok(Some(sources)) => FloorCompileCleanReceipt::Compiled {
            ok: floor_compile_clean_emit_ok_via_index(sources, &index_roots),
        },
    }
}

/// Run the floor's single whole-tree `--target dag` compile and store the receipt for gate
/// consumption. Exactly one install per `claim_executor` process; second install is an error.
pub fn install_floor_compile_clean_receipt() -> Result<(), String> {
    let mut guard = FLOOR_COMPILE_CLEAN_RECEIPT
        .lock()
        .map_err(|e| format!("floor compile-clean receipt lock poisoned: {e}"))?;
    if guard.is_some() {
        return Err(
            "floor compile-clean receipt already installed — one whole-tree compile per run"
                .to_string(),
        );
    }
    eprintln!(
        "claim_executor: floor compile-clean — one whole-tree --target dag compile via shared index (gate consumes receipt; batch-2 resolves reuse the typed store)"
    );
    let receipt = produce_floor_compile_clean_receipt();
    if let FloorCompileCleanReceipt::Compiled { ok } = &receipt {
        eprintln!("claim_executor: floor compile-clean receipt ok={ok}");
    }
    *guard = Some(receipt);
    Ok(())
}

pub fn floor_compile_clean_receipt_installed() -> bool {
    FLOOR_COMPILE_CLEAN_RECEIPT
        .lock()
        .ok()
        .map(|g| g.is_some())
        .unwrap_or(false)
}

/// Gate consumer: reads the receipt from `install_floor_compile_clean_receipt` only.
/// Refuses when no receipt exists — never runs a second compile.
pub fn consume_floor_compile_clean_gate_verdict() -> bool {
    if FLOOR_COMPILE_CLEAN_LAZY_INSTALL.load(Ordering::SeqCst)
        && !floor_compile_clean_receipt_installed()
    {
        if let Err(msg) = install_floor_compile_clean_receipt() {
            if !floor_compile_clean_receipt_installed() {
                eprintln!("compile-clean gate: refused — receipt install failed ({msg})");
                return false;
            }
            // Serial `run_walk` today; if a future scheduler fans out batch-1, a concurrent
            // lazy install may win first — consume the installed receipt, do not refuse.
        }
    }
    let guard = match FLOOR_COMPILE_CLEAN_RECEIPT.lock() {
        Ok(g) => g,
        Err(e) => {
            eprintln!("compile-clean gate: refused — receipt lock poisoned ({e})");
            return false;
        }
    };
    match guard.as_ref() {
        None => {
            eprintln!(
                "compile-clean gate: refused — no in-run compile receipt (gate must consume the executor's one whole-tree --target dag compile)"
            );
            false
        }
        Some(FloorCompileCleanReceipt::Skipped { reason }) => {
            eprintln!("compile-clean gate: skipped ({reason})");
            true
        }
        Some(FloorCompileCleanReceipt::Refused { reason }) => {
            eprintln!("compile-clean gate: refused ({reason})");
            false
        }
        Some(FloorCompileCleanReceipt::Compiled { ok }) => *ok,
    }
}

#[cfg(test)]
fn reset_floor_compile_clean_receipt_for_test() {
    let mut guard = FLOOR_COMPILE_CLEAN_RECEIPT.lock().unwrap();
    *guard = None;
}

#[cfg(test)]
fn install_floor_compile_clean_receipt_fixture(receipt: FloorCompileCleanReceipt) {
    let mut guard = FLOOR_COMPILE_CLEAN_RECEIPT.lock().unwrap();
    *guard = Some(receipt);
}

// DELETE WHEN dissolved: `compile_clean_whole_tree_hard_diagnostics`,
// `compile_clean_diagnostic_histogram_key`, `truncate_histogram_label`,
// `compile_clean_internal_error_histogram_name`, and the `compile_clean_diagnostic_histogram` bin
// (~200 LOC).
// Receipt: `rg cli_run_compile_clean_diagnostic_histogram src/v1/stage0` == 1 until deletion;
// ROADMAP §1 namespace-only lane (docs/plans/namespace-resolution-design.md).
pub(crate) const CLI_RUN_COMPILE_CLEAN_DIAGNOSTIC_HISTOGRAM_SCAFFOLD_MARKER: &str =
    "cli_run_compile_clean_diagnostic_histogram";

/// Whole-tree `--target dag` compile-clean (witness_layer_roots closure).
/// Instrument path for diagnostic histogram — not for cargo tests.
///
/// INTERIM hand-Rust scaffold (`CLI_RUN_COMPILE_CLEAN_DIAGNOSTIC_HISTOGRAM_SCAFFOLD_MARKER` / §7):
/// dissolves when ROADMAP §1 namespace-only lane closes (import strip + global_bare wiring fixed)
/// or a floor-enrolled diagnostic-histogram lens subsumes this host transport.
/// Uses the same resolve kernel as `witness_layer_roots_compile_clean_check`
/// (`compile_to_resolved` on the whole-tree source closure).
pub fn compile_clean_whole_tree_hard_diagnostics() -> Result<im::Vector<Rc<ErrorNode>>, String> {
    let plan = CompileCleanScopePlan::WholeTree;
    let sources = match witness_layer_roots_compile_clean_sources_for_plan(&plan)? {
        None => return Err("compile-clean whole-tree: no sources (unexpected skip)".to_string()),
        Some(s) => s,
    };
    let result = v1_compiler_compile::compile_to_resolved(Rc::new(sources.into()));
    Ok(result
        .diagnostics
        .iter()
        .filter(|d| compile_clean_diagnostic_is_hard(d))
        .cloned()
        .collect())
}

/// `(class, name)` key for histogram aggregation over hard diagnostics.
///
/// INTERIM hand-Rust scaffold (`CLI_RUN_COMPILE_CLEAN_DIAGNOSTIC_HISTOGRAM_SCAFFOLD_MARKER` / §7):
/// total match over `CompilerDiagnostic` variants — no silent widening.
pub fn compile_clean_diagnostic_histogram_key(d: &Rc<ErrorNode>) -> (String, String) {
    use crate::v1_std_core::CompilerDiagnostic;
    let class = match d.diagnostic.as_ref() {
        CompilerDiagnostic::UnresolvedImport { .. } => "UnresolvedImport",
        CompilerDiagnostic::MissingExport { .. } => "MissingExport",
        CompilerDiagnostic::UnresolvedType { .. } => "UnresolvedType",
        CompilerDiagnostic::TypeMismatch { .. } => "TypeMismatch",
        CompilerDiagnostic::ArityMismatch { .. } => "ArityMismatch",
        CompilerDiagnostic::VariantNotFound { .. } => "VariantNotFound",
        CompilerDiagnostic::FieldNotFound { .. } => "FieldNotFound",
        CompilerDiagnostic::MissingField { .. } => "MissingField",
        CompilerDiagnostic::NonExhaustiveMatch { .. } => "NonExhaustiveMatch",
        CompilerDiagnostic::CircularDependency { .. } => "CircularDependency",
        CompilerDiagnostic::DuplicateModule { .. } => "DuplicateModule",
        CompilerDiagnostic::MissingAnnotation { .. } => "MissingAnnotation",
        CompilerDiagnostic::ParseError { .. } => "ParseError",
        CompilerDiagnostic::InternalError { .. } => "InternalError",
        CompilerDiagnostic::ComplexityUnknown { .. } => "ComplexityUnknown",
        CompilerDiagnostic::OwnershipViolation { .. } => "OwnershipViolation",
        CompilerDiagnostic::VariantCollision { .. } => "VariantCollision",
        CompilerDiagnostic::SoleConstructorViolation { .. } => "SoleConstructorViolation",
        CompilerDiagnostic::UnlistedImportUse { .. } => "UnlistedImportUse",
    };
    let name = match d.diagnostic.as_ref() {
        CompilerDiagnostic::UnresolvedImport { module_path, .. } => module_path.clone(),
        CompilerDiagnostic::MissingExport { name, .. } => name.clone(),
        CompilerDiagnostic::UnresolvedType { name, .. } => name.clone(),
        CompilerDiagnostic::TypeMismatch { got, .. } => got.clone(),
        CompilerDiagnostic::ArityMismatch { name, .. } => name.clone(),
        CompilerDiagnostic::VariantNotFound { variant, .. } => variant.clone(),
        CompilerDiagnostic::FieldNotFound { field, .. } => field.clone(),
        CompilerDiagnostic::MissingField { field, .. } => field.clone(),
        CompilerDiagnostic::NonExhaustiveMatch { .. } => "(non-exhaustive)".to_string(),
        CompilerDiagnostic::CircularDependency { .. } => "(cycle)".to_string(),
        CompilerDiagnostic::DuplicateModule { name, .. } => name.clone(),
        CompilerDiagnostic::MissingAnnotation { fn_name, .. } => fn_name.clone(),
        CompilerDiagnostic::ParseError { message, .. } => truncate_histogram_label(message, 80),
        CompilerDiagnostic::InternalError { message, .. } => {
            compile_clean_internal_error_histogram_name(message)
        }
        CompilerDiagnostic::ComplexityUnknown { func_name, .. } => func_name.clone(),
        CompilerDiagnostic::OwnershipViolation { binding, .. } => binding.clone(),
        CompilerDiagnostic::VariantCollision { variant, .. } => variant.clone(),
        CompilerDiagnostic::SoleConstructorViolation { type_name, .. } => type_name.clone(),
        CompilerDiagnostic::UnlistedImportUse { name, .. } => name.clone(),
    };
    (class.to_string(), name)
}

fn truncate_histogram_label(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        let ellipsis = '…';
        let budget = max.saturating_sub(ellipsis.len_utf8());
        let end = s
            .char_indices()
            .map(|(i, c)| i + c.len_utf8())
            .take_while(|&end| end <= budget)
            .last()
            .unwrap_or(0);
        format!("{s_prefix}{ellipsis}", s_prefix = &s[..end])
    }
}

fn compile_clean_internal_error_histogram_name(message: &str) -> String {
    if let Some(rest) = message.strip_prefix("function '") {
        if let Some(name) = rest.split_once('\'').map(|(n, _)| n) {
            return format!("function:{name}");
        }
    }
    if let Some(rest) = message.strip_prefix("undefined variable '") {
        if let Some(name) = rest.split_once('\'').map(|(n, _)| n) {
            return format!("variable:{name}");
        }
    }
    truncate_histogram_label(message, 80)
}

/// Resolve/typecheck leg of compile-clean over `witness_layer_roots` (`dag` + `src/v2` only).
/// Uses `primary-precedence` pool indexing like shell compile, but a narrower root set than
/// `compile_clean_source_roots()` (which adds `src/v1` for cross-tree perturb receipts).
/// In CI (`GITHUB_ACTIONS=true`) or when `GUNBC_CI_DIFF_BASE` is set, scopes to affected
/// shard entries from `tools.dag_compile_clean_scope` (lever a) using `gunbc_ci_spec.diff_policy`
/// defaults via `floor_diff_observe`; diff/disposition failure refuses (never widens).
/// Skip/whole-tree/skip-vs-run authority lives in `tools.dag_compile_clean_scope` (including
/// `RequireWholeTree` for non-docs infra/Rust touches with no shard intersection).
pub fn witness_layer_roots_compile_clean_check() -> bool {
    match witness_layer_roots_compile_clean_sources_for_plan(&compile_clean_scope_plan_for_ci()) {
        Ok(None) => true,
        Ok(Some(sources)) => {
            let result = v1_compiler_compile::compile_to_resolved(Rc::new(sources.into()));
            if compile_clean_resolve_has_hard_errors(&result) {
                eprint_compile_clean_hard_diagnostics(result.diagnostics.as_ref());
                false
            } else {
                true
            }
        }
        Err(msg) => {
            eprintln!("compile-clean: source load failed ({msg})");
            false
        }
    }
}

/// Emit leg: `--target dag` compile over witness layer roots without shell or disk write.
/// Direct-run oracle for non-floor contexts (cargo tests, enrolled witnesses). The CI
/// floor gate consumes `consume_floor_compile_clean_gate_verdict` instead (Lever A).
pub fn witness_layer_roots_compile_clean_emit_check() -> bool {
    match witness_layer_roots_compile_clean_sources_for_plan(&compile_clean_scope_plan_for_ci()) {
        Ok(None) => true,
        Ok(Some(sources)) => floor_compile_clean_emit_ok(sources),
        Err(msg) => {
            eprintln!("compile-clean emit: source load failed ({msg})");
            false
        }
    }
}

/// Workspace-relative path for module-graph closure queries (`v2.lens.module_graph`).
fn workspace_relative_repo_path(path: &str) -> String {
    let norm = path.strip_prefix("./").unwrap_or(path).replace('\\', "/");
    let p = Path::new(&norm);
    if p.is_absolute() {
        repo_relative_path_normalized(p)
    } else {
        norm
    }
}

/// Entry-path variant of `workspace_relative_repo_path` that NEVER panics.
///
/// A user-supplied entry can legitimately sit outside every source root (an
/// absolute path under `/tmp`, a stray file). That is definitionally out-of-pool
/// and must reach the typed, located refusal in `resolve_transitively`, not abort
/// via `repo_relative_path_normalized`'s panic arm — that panic is the correct
/// fail-closed for a CORPUS path that should be under a root, but the wrong
/// failure MODE for an entry the caller is about to reject. When the path cannot
/// be made repo-relative, return it unchanged: `declares_repo_path` rejects it
/// and the refusal fires (DESIGN §5: refuse, never abort-in-lieu).
fn workspace_relative_entry_path(path: &str) -> String {
    let norm = path.strip_prefix("./").unwrap_or(path).replace('\\', "/");
    let p = Path::new(&norm);
    if p.is_relative() {
        return norm;
    }
    if let Ok(rel) = repo_relative_path(p) {
        return rel;
    }
    if let Ok(stripped) = p.strip_prefix(workspace_root()) {
        return stripped.to_string_lossy().replace('\\', "/");
    }
    norm
}

/// Normalize `source_roots` to the workspace-relative form `import_resolution_facts` /
/// `module_declaration_facts` expect when invoked from `.dag` (`witness_layer_roots` style).
fn pool_roots_for_module_graph_closure(source_roots: &[String]) -> Vec<String> {
    source_roots
        .iter()
        .map(|r| {
            let p = Path::new(r);
            if p.is_absolute() {
                repo_relative_path_normalized(p)
            } else {
                r.replace('\\', "/")
            }
        })
        .collect()
}

fn path_to_source_lookup(
    index: &ModuleSourceIndex,
) -> HashMap<String, Rc<v1_compiler_compile::SourceFile>> {
    let mut out = HashMap::new();
    for sf in index.values() {
        let rel = workspace_relative_repo_path(&sf.path);
        out.insert(rel, sf.clone());
        out.insert(sf.path.clone(), sf.clone());
    }
    out
}

fn build_import_adjacency(
    edges: &[ImportResolutionFactRaw],
    nodes: &[ModuleDeclarationFactRaw],
) -> HashMap<String, Vec<String>> {
    let mut module_to_path: HashMap<String, String> = HashMap::new();
    for node in nodes {
        module_to_path.insert(
            node.module.clone(),
            workspace_relative_repo_path(&node.path),
        );
    }

    let mut adjacency: HashMap<String, Vec<String>> = HashMap::new();
    for edge in edges {
        let Some(imported) = module_to_path.get(&edge.import_module) else {
            continue;
        };
        let importer = workspace_relative_repo_path(&edge.path);
        let entry = adjacency.entry(importer).or_default();
        if !entry.iter().any(|p| p == imported) {
            entry.push(imported.clone());
        }
    }
    adjacency
}

/// Worklist BFS over pre-normalized adjacency (O(V+E) per entry).
pub fn import_closure_from_adjacency(
    entry_path: &str,
    adjacency: &HashMap<String, Vec<String>>,
) -> Vec<String> {
    let entry_path = workspace_relative_repo_path(entry_path);
    let mut reached: HashSet<String> = HashSet::new();
    reached.insert(entry_path.clone());
    let mut queue: VecDeque<String> = VecDeque::from([entry_path]);

    while let Some(importer) = queue.pop_front() {
        let Some(targets) = adjacency.get(&importer) else {
            continue;
        };
        for path in targets {
            if reached.insert(path.clone()) {
                queue.push_back(path.clone());
            }
        }
    }

    let mut result: Vec<String> = reached.into_iter().collect();
    result.sort();
    result
}

/// Host realization of `v2.lens.module_graph.import_closure` over modeled fact rows.
/// Authority: `src/v2/lens/module_graph.dag` — this is the consumer repoint surface for
/// `cli_run.rs` resolve/reconcile (Phase 1 de-fork); fact extraction stays on the existing
/// `import_resolution_facts` / `module_declaration_facts` builtins.
pub fn import_closure_from_facts(
    entry_path: &str,
    edges: &[ImportResolutionFactRaw],
    nodes: &[ModuleDeclarationFactRaw],
) -> Vec<String> {
    let adjacency = build_import_adjacency(edges, nodes);
    import_closure_from_adjacency(entry_path, &adjacency)
}

/// Pre-built `import_resolution_facts` / `module_declaration_facts` rows for one pool-root
/// set. Built once per `MultiEntryIndex` / resolve pass so closure queries do not re-scan the
/// corpus on every `resolve_transitively` call (Phase 1 perf receipt, DESIGN §2).
#[derive(Clone)]
pub struct ModuleGraphFactsLive {
    edges: Vec<ImportResolutionFactRaw>,
    nodes: Vec<ModuleDeclarationFactRaw>,
    adjacency: HashMap<String, Vec<String>>,
    // Workspace-relative paths of `nodes`, precomputed once per facts build: the
    // membership question is asked per ENTRY on the resolve path (see
    // `resolve_transitively`), so deriving it per call would rebuild an O(corpus)
    // set per entry (bare-minimum-cost, DESIGN §6).
    declared_paths: HashSet<String>,
    // SELECTION-ONLY adjacency: `adjacency` above PLUS strict-tier (Qualified + UniqueBare)
    // reference-derived edges for import-less files.
    //
    // It is a second map rather than a widening of `adjacency` because the two consumers need
    // different tiers and mixing them is a live regression, not a hypothetical: `adjacency` also
    // feeds LOADER closures (`import_closure_live_paths_with_facts`, and `resolve_transitively`
    // inside `precompute_whole_tree_published_mock_keys`), which then Strict-resolve whatever they
    // reach. Unioning reference edges into that map grew the mock-corpus precompute closure until
    // it pulled `dag/` modules importing `v2.*` into a dag-only pool, where those imports cannot
    // resolve — measured, not predicted (the precompute went from 82 keys to a hard failure).
    // Selection wants maximum precision; the loader wants a safe superset over a pool it can
    // actually resolve. Same facts build, two answers, no shared tier.
    selection_adjacency: HashMap<String, Vec<String>>,
    // Import-less files the reference-edge producer could not answer for (unreadable / no module
    // line / parse failure). An entry in this set has an UNKNOWN dependency set, which is the one
    // state `entry_file_touched_via_import_closure` may refuse on. Every other edgeless entry has
    // a known-empty dependency set and a precise `{self}` closure.
    reference_unaccounted: HashSet<String>,
}

#[cfg(test)]
static MODULE_GRAPH_FACTS_BUILD_COUNT: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(0);

#[cfg(test)]
pub(crate) fn reset_module_graph_facts_build_count_for_test() {
    MODULE_GRAPH_FACTS_BUILD_COUNT.store(0, std::sync::atomic::Ordering::SeqCst);
}

#[cfg(test)]
pub(crate) fn module_graph_facts_build_count_for_test() -> usize {
    MODULE_GRAPH_FACTS_BUILD_COUNT.load(std::sync::atomic::Ordering::SeqCst)
}

#[cfg(test)]
mod shared_cache_collision_guard_tests {
    use super::check_module_source_identity_map;

    // Collision-honesty receipt (union-resolve §6.3): the shared typed-module cache's
    // source-identity guard fails LOUD when one module name resolves from two declaring files
    // in a process, and stays green when the same file is re-seen through many import paths.
    // This is the guard exercised by execution with a red control — not an inert wall.
    #[test]
    fn source_identity_flags_coresidence_collision_but_allows_reexport() {
        let mut reg = std::collections::HashMap::new();
        // First sight records the identity.
        assert!(check_module_source_identity_map(&mut reg, "std.foo", "dag/std/foo.dag").is_ok());
        // GREEN control: the SAME module reached again (a legitimate re-export / second
        // import path) is benign — one authority, many hops — so no error.
        assert!(check_module_source_identity_map(&mut reg, "std.foo", "dag/std/foo.dag").is_ok());
        // A distinct name from a distinct file is fine.
        assert!(check_module_source_identity_map(&mut reg, "std.bar", "dag/std/bar.dag").is_ok());
        // RED control: the same name from a DIFFERENT declaring file is the co-residence
        // surprise — a loud typed error, never a silently divergent resolution.
        let err = check_module_source_identity_map(&mut reg, "std.foo", "src/v2/std/foo.dag")
            .expect_err("a colliding module name from a second file must fail closed");
        assert!(
            err.contains("co-residence collision") && err.contains("std.foo"),
            "collision error must name the module and the seam: {err}"
        );
    }
}

#[cfg(test)]
mod typed_module_content_key_tests {
    //! Typed-module content-key RED controls (cross-entry-typed-module-memo-sketch.md
    //! §1/§3, operator-signed 2026-07-16; PR-α — the store re-key).
    //!
    //! The typed store keys on `std.interface_summary.typed_module_key` — module source
    //! hash ⊕ direct-import interface hashes ⊕ compiler identity — never on authored
    //! module name. Each test is a discriminating control for one live key term, proven
    //! BY EXECUTION against the store (`typecheck_compute_count` counts genuine
    //! typechecks — a stale serve shows up as a missing compute):
    //!
    //!  - **source term**: mutate a module's source (same path, same authored name)
    //!    between two indexes sharing one cross-worker store → the mutated module MUST
    //!    recompute. Under the dissolved name key this control goes RED (stale serve,
    //!    0 computes).
    //!  - **import-interface term**: change an imported module's export surface (its v0
    //!    interface hash) without touching the dependent → the dependent MUST recompute.
    //!    RED under the name key the same way.
    //!  - **interface-grain minimality** (signed decision 1): a body-only edit in the
    //!    import leaves its v0 interface hash unchanged → the dependent must HIT (only
    //!    the edited module recomputes). A conservative source-transitive key would go
    //!    RED here by over-invalidating the dependent.
    //!
    //! The compiler-identity term cannot vary within one test process; it is witnessed
    //! at the algebra level by the PR1 .dag witnesses
    //! (`src/v2/test/claim/interface_summary/typed_module_key_test.dag`). Warm==cold
    //! byte-equivalence stays owned by `resolve_typed_cache_equivalence_test`.
    //!
    //! Mutations rewrite the SAME file path so the `module_source_identity` collision
    //! wall (name→file, unchanged by the re-key) never fires.

    use super::{
        build_multi_entry_index_with_shared_caches, new_shared_typecheck_caches,
        reset_typecheck_compute_count, resolve_entry_with_index, typecheck_compute_count,
        with_typecheck_compute_count_receipt, workspace_root,
    };
    use crate::shared_typecheck_store::SharedTypecheckCaches;
    use std::fs;
    use std::sync::{Arc, RwLock};

    const IMPORT_MODULE: &str = "module k.imp\nfn base() -> Int { 10 }\n";
    const IMPORT_MODULE_BODY_EDIT: &str = "module k.imp\nfn base() -> Int { 4 + 6 }\n";
    const IMPORT_MODULE_SURFACE_EDIT: &str =
        "module k.imp\nfn base() -> Int { 10 }\nfn extra() -> Int { 1 }\n";
    const DEPENDENT_MODULE: &str =
        "module k.dep\nimport k.imp { base }\nfn wit() -> Bool { base() == 10 }\n";

    struct Fixture {
        dir: std::path::PathBuf,
        roots: Vec<String>,
        entry: String,
    }

    impl Fixture {
        fn new(tag: &str) -> Fixture {
            use std::sync::atomic::{AtomicU64, Ordering};
            static SEQ: AtomicU64 = AtomicU64::new(0);
            let seq = SEQ.fetch_add(1, Ordering::SeqCst);
            let dir = workspace_root().join("target").join(format!(
                "typed-module-content-key-{tag}-{}-{seq}",
                std::process::id()
            ));
            let _ = fs::remove_dir_all(&dir);
            fs::create_dir_all(&dir).expect("create fixture dir");
            fs::write(dir.join("imp.dag"), IMPORT_MODULE).expect("write imp.dag");
            fs::write(dir.join("dep.dag"), DEPENDENT_MODULE).expect("write dep.dag");
            let roots = vec![dir.to_string_lossy().into_owned()];
            let entry = dir.join("dep.dag").to_string_lossy().into_owned();
            Fixture { dir, roots, entry }
        }

        fn rewrite_import(&self, src: &str) {
            fs::write(self.dir.join("imp.dag"), src).expect("rewrite imp.dag");
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.dir);
        }
    }

    /// Resolve `entry` against a fresh index bound to `store`, returning how many genuine
    /// typecheck computes the resolve performed (misses; hits don't count).
    fn computes_with_store(
        store: &Arc<RwLock<SharedTypecheckCaches>>,
        roots: &[String],
        entry: &str,
    ) -> usize {
        let index = build_multi_entry_index_with_shared_caches(roots, store.clone());
        reset_typecheck_compute_count();
        resolve_entry_with_index(&index, entry).expect("resolve");
        typecheck_compute_count()
    }

    #[test]
    fn source_term_recomputes_mutated_module() {
        with_typecheck_compute_count_receipt(|| {
            let fx = Fixture::new("source-term");
            let store = new_shared_typecheck_caches();

            let cold = computes_with_store(&store, &fx.roots, &fx.entry);
            assert_eq!(cold, 2, "cold resolve computes both closure modules");

            // Unchanged snapshot, fresh index: full hit — the content key is stable.
            let warm = computes_with_store(&store, &fx.roots, &fx.entry);
            assert_eq!(warm, 0, "unchanged snapshot must be a full store hit");

            // Body-only mutation of the import: its source hash moves, so its key moves.
            fx.rewrite_import(IMPORT_MODULE_BODY_EDIT);
            let after_edit = computes_with_store(&store, &fx.roots, &fx.entry);
            assert!(
                after_edit >= 1,
                "mutated module must recompute (a stale serve is a §5 fail-open); \
                 got {after_edit} computes"
            );
        });
    }

    #[test]
    fn import_interface_term_recomputes_dependent() {
        with_typecheck_compute_count_receipt(|| {
            let fx = Fixture::new("import-term");
            let store = new_shared_typecheck_caches();

            let cold = computes_with_store(&store, &fx.roots, &fx.entry);
            assert_eq!(cold, 2, "cold resolve computes both closure modules");

            // Export-surface edit: a new exported fn changes k.imp's interface rollup.
            fx.rewrite_import(IMPORT_MODULE_SURFACE_EDIT);
            let after_edit = computes_with_store(&store, &fx.roots, &fx.entry);
            assert_eq!(
                after_edit, 2,
                "an interface change in the import must recompute the import AND its \
                 dependent (the dependent's typed result consumed that interface)"
            );
        });
    }

    #[test]
    fn body_only_edit_leaves_dependent_warm() {
        with_typecheck_compute_count_receipt(|| {
            let fx = Fixture::new("minimality");
            let store = new_shared_typecheck_caches();

            let cold = computes_with_store(&store, &fx.roots, &fx.entry);
            assert_eq!(cold, 2, "cold resolve computes both closure modules");

            fx.rewrite_import(IMPORT_MODULE_BODY_EDIT);
            let after_edit = computes_with_store(&store, &fx.roots, &fx.entry);
            assert_eq!(
                after_edit, 1,
                "body-only import edit: import recomputes, dependent stays warm \
                 (interface-grain minimality, signed decision 1)"
            );
        });
    }
}

#[cfg(test)]
mod compile_clean_via_index_verdict_equivalence {
    //! Verdict-equivalence controls for the floor receipt's via-index compile
    //! (lever 1, typecheck investigation PR #6766): the raw pipeline
    //! (`floor_compile_clean_emit_ok`, still the oracle behind
    //! `witness_layer_roots_compile_clean_emit_check`) and the shared-index receipt
    //! path (`floor_compile_clean_emit_ok_via_index`) must agree — green on a clean
    //! corpus, red on a planted unresolved import, red on a planted type mismatch.
    //! The raw path shares no caches with the via-index path, so agreement is an
    //! independent second opinion, not a self-check. Whole-tree agreement receipts
    //! (green 377.7s vs 381.4s; planted-inject red 4.3s vs 361.6s; both agree) are
    //! recorded in PR #6766.

    use super::{floor_compile_clean_emit_ok, floor_compile_clean_emit_ok_via_index};
    use crate::v1_compiler_compile::SourceFile;
    use std::fs;
    use std::rc::Rc;

    struct Corpus {
        dir: std::path::PathBuf,
        roots: Vec<String>,
        sources: Vec<Rc<SourceFile>>,
    }

    impl Corpus {
        /// Write `files` under a fresh fixture dir (the via-index path builds its
        /// `process_shared_index` from on-disk roots) and mirror them as the source
        /// set both compile paths receive.
        fn new(tag: &str, files: &[(&str, &str)]) -> Corpus {
            use std::sync::atomic::{AtomicU64, Ordering};
            static SEQ: AtomicU64 = AtomicU64::new(0);
            let seq = SEQ.fetch_add(1, Ordering::SeqCst);
            let dir = super::workspace_root().join("target").join(format!(
                "compile-clean-equiv-{tag}-{}-{seq}",
                std::process::id()
            ));
            let _ = fs::remove_dir_all(&dir);
            fs::create_dir_all(&dir).expect("create fixture dir");
            let mut sources = Vec::new();
            for (name, content) in files {
                let path = dir.join(name);
                fs::write(&path, content).expect("write fixture module");
                sources.push(Rc::new(SourceFile {
                    path: path.to_string_lossy().into_owned(),
                    content: (*content).to_string(),
                }));
            }
            let roots = vec![dir.to_string_lossy().into_owned()];
            Corpus {
                dir,
                roots,
                sources,
            }
        }

        fn verdicts(&self) -> (bool, bool) {
            let raw = floor_compile_clean_emit_ok(self.sources.clone());
            let via_index =
                floor_compile_clean_emit_ok_via_index(self.sources.clone(), &self.roots);
            (raw, via_index)
        }
    }

    impl Drop for Corpus {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.dir);
        }
    }

    #[test]
    fn green_corpus_agrees_green() {
        let corpus = Corpus::new(
            "green",
            &[
                ("imp.dag", "module eqv.imp\nfn base() -> Int { 10 }\n"),
                (
                    "dep.dag",
                    "module eqv.dep\nimport eqv.imp { base }\nfn wit() -> Bool { base() == 10 }\n",
                ),
            ],
        );
        let (raw, via_index) = corpus.verdicts();
        assert!(raw, "raw pipeline must be green on the clean corpus");
        assert!(
            via_index,
            "via-index path must be green on the clean corpus"
        );
    }

    /// §5 discriminating RED: an unresolved import must red BOTH paths.
    #[test]
    fn planted_unresolved_import_agrees_red() {
        let corpus = Corpus::new(
            "unresolved",
            &[(
                "bad.dag",
                "module eqv.unresolved\nimport totally.nonexistent.module { Foo }\nfn probe() -> Int { 42 }\n",
            )],
        );
        let (raw, via_index) = corpus.verdicts();
        assert!(!raw, "raw pipeline must red on an unresolved import");
        assert!(
            !via_index,
            "via-index path must red on an unresolved import"
        );
    }

    /// Roots-key canonicalization (review 39118): the executor's absolute CLI roots
    /// and the plan's relative `witness_layer_roots` are the SAME pool and must
    /// address ONE thread-local shared index — otherwise the compile-clean receipt
    /// warms an index batch-2 silently replaces, and the whole lever-1 sharing claim
    /// is void on the CI path. Rc pointer equality is the discriminating check: two
    /// spellings, one universe.
    #[test]
    fn shared_index_roots_key_canonicalizes_absolute_and_relative_spellings() {
        // Workspace cwd: canonical roots are repo-relative and the index build
        // resolves them against cwd — the executor always runs from the repo root;
        // cargo test does not.
        let ws = super::workspace_root();
        let prior = std::env::current_dir().ok();
        std::env::set_current_dir(&ws).expect("chdir workspace");
        let absolute = vec![
            ws.join("dag").to_string_lossy().into_owned(),
            ws.join("src/v2").to_string_lossy().into_owned(),
        ];
        let relative = vec!["dag".to_string(), "src/v2".to_string()];
        let a = super::process_shared_index(&absolute);
        let b = super::process_shared_index(&relative);
        if let Some(p) = prior {
            let _ = std::env::set_current_dir(p);
        }
        assert!(
            Rc::ptr_eq(&a, &b),
            "absolute and relative spellings of the same pool must address ONE shared \
             index — a roots-key fork gives the receipt and batch-2 two typed universes"
        );
    }

    /// §5 discriminating RED: a typecheck-stage red must red BOTH paths. The planted
    /// module is the same variant-mismatch shape as the transport's canonical
    /// `perturb_module_source` (`dag/tools/dag_compile_clean_transport.dag`) — `Some`
    /// matched against a `Present`-constructed optional — a proven raw-pipeline red.
    #[test]
    fn planted_typecheck_red_agrees_red() {
        let corpus = Corpus::new(
            "typecheck-red",
            &[(
                "bad.dag",
                "module eqv.typecheck_red\nfn probe() -> String? {\n  match Present { value: \"x\" } {\n    Some { value: s } => Some { value: s }\n    None => none\n  }\n}\n",
            )],
        );
        let (raw, via_index) = corpus.verdicts();
        assert!(
            !raw,
            "raw pipeline must red on the planted variant mismatch"
        );
        assert!(
            !via_index,
            "via-index path must red on the planted variant mismatch"
        );
    }
}

thread_local! {
    static MODULE_PATH_INDEX_CACHE: RefCell<HashMap<String, HashMap<String, String>>> =
        RefCell::new(HashMap::new());
}

#[cfg(test)]
pub(crate) fn reset_module_path_index_cache_for_test() {
    MODULE_PATH_INDEX_CACHE.with(|cache| cache.borrow_mut().clear());
}

thread_local! {
    static MODULE_GRAPH_FACTS_CACHE: RefCell<HashMap<String, ModuleGraphFactsLive>> =
        RefCell::new(HashMap::new());
}

#[cfg(test)]
pub(crate) fn reset_module_graph_facts_cache_for_test() {
    MODULE_GRAPH_FACTS_CACHE.with(|cache| cache.borrow_mut().clear());
    reset_module_path_index_cache_for_test();
}

fn build_module_graph_facts_live_uncached(pool_roots: &[String]) -> ModuleGraphFactsLive {
    #[cfg(test)]
    MODULE_GRAPH_FACTS_BUILD_COUNT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    const EXCLUDE: &[String] = &[];
    let roots = pool_roots_for_module_graph_closure(pool_roots);
    // NOTE: the module-graph LOADER closure stays import-derived for now (Blocker-1 part 1). A
    // reference-derived closure changes every witness's load set tree-wide and surfaces latent issues
    // (import-less-but-referencing std files, witnesses that need src/v1 in their pool, the
    // pre-existing fleet_converge Srv3 red, and homonyms the bright-cat lane must qualify), so the
    // loader repoint is staged as a separate part after those land. The REFERENCE producer below is
    // already live via the inert-lens reach (the strips' documented CI blocker), which is hygiene-
    // only and cannot regress a compile.
    //
    // EDGE SOURCE — the swap `module_graph.dag`'s `dependency_edge_source_migration_note` designates:
    // "when [the namespace terminal step] lands, `dependency_resolution_facts_live` swaps to the
    // reference-derived producer and nothing above it changes". Imports were deleted from most of the
    // corpus without this half landing, which left ~530 claim modules with an empty adjacency and a
    // widen arm below that answered "affected" for all of them — the absorbing fallback DESIGN §5
    // names verbatim ("can't compute the affected set → rerun the entire suite").
    //
    // Attempted 2026-07-14 and REVERTED: unioning `reference_edges_as_import_facts(..., false)`
    // ballooned a single small witness entry's load set from 27 to 424 resolved sources. That
    // measurement was correct and its conclusion ("the information is unusable here") was not — it
    // was taken at the `strict = false` tier, which keeps `AmbiguousBare` edges, so every ubiquitous
    // homonym fans its referrers out across the pool (median closure 1136 of 2240 modules).
    //
    // The tier is the fix. `strict = true` keeps Qualified + UniqueBare and drops AmbiguousBare:
    // median closure 96, p95 554 — the same order as the import-only baseline's 54/175 — and 522 of
    // the 530 edgeless claim modules gain a real edge. Measured over 14 merged diffs the selected
    // witness share goes 70.3% → 49.4% (the `false` tier goes the wrong way, to 83.4%).
    //
    // The two consumers take DIFFERENT tiers on purpose, and conflating them is what made this look
    // impossible: for the LOADER an over-connected edge is harmless (a superset just compiles extra
    // modules), while for SELECTION it is precisely the thing that destroys the answer. The loader
    // (`extend_with_bare_reference_closure`) is deliberately left alone.
    //
    // Import-bearing files emit no reference edges at all (see `reference_resolution_facts` pass 2),
    // so on an un-stripped file the union is a no-op and the graph is byte-identical to before.
    let edges = import_resolution_facts(&roots, &roots, EXCLUDE);
    let nodes = module_declaration_facts(&roots);
    // Loader tier: import edges only, unchanged. Every consumer that goes on to RESOLVE what it
    // reaches reads this one.
    let adjacency = build_import_adjacency(&edges, &nodes);
    // Selection tier: import edges + strict reference edges.
    let mut selection_edges = edges.clone();
    selection_edges.extend(reference_edges_as_import_facts(
        &reference_resolution_facts(&roots, &roots, EXCLUDE),
        /* strict */ true,
    ));
    let selection_adjacency = build_import_adjacency(&selection_edges, &nodes);
    let reference_unaccounted: HashSet<String> =
        reference_accounting_refusals(&roots, &roots, EXCLUDE)
            .into_iter()
            .map(|r| workspace_relative_repo_path(&r.path))
            .collect();
    let declared_paths = nodes
        .iter()
        .map(|n| workspace_relative_repo_path(&n.path))
        .collect();
    ModuleGraphFactsLive {
        edges,
        nodes,
        adjacency,
        selection_adjacency,
        declared_paths,
        reference_unaccounted,
    }
}

pub fn build_module_graph_facts_live(pool_roots: &[String]) -> ModuleGraphFactsLive {
    let key = pool_roots_for_module_graph_closure(pool_roots).join("\u{1f}");
    MODULE_GRAPH_FACTS_CACHE.with(|cache| {
        if let Some(facts) = cache.borrow().get(&key) {
            return facts.clone();
        }
        let facts = build_module_graph_facts_live_uncached(pool_roots);
        cache.borrow_mut().insert(key, facts.clone());
        facts
    })
}

/// Host realization of `v2.lens.module_graph.import_closure_live`.
pub fn import_closure_live_paths(
    entry_path: &str,
    pool_roots: &[String],
) -> Result<Vec<String>, String> {
    let facts = build_module_graph_facts_live(pool_roots);
    Ok(import_closure_live_paths_with_facts(entry_path, &facts))
}

pub fn import_closure_live_paths_with_facts(
    entry_path: &str,
    facts: &ModuleGraphFactsLive,
) -> Vec<String> {
    import_closure_from_adjacency(entry_path, &facts.adjacency)
}

impl ModuleGraphFactsLive {
    /// Repo-relative paths of every declared module in the facts scan — the existence
    /// set for refuse-vs-answer decisions (a module can be absent from `adjacency`
    /// legitimately by importing nothing; absence from `nodes` means the facts do not
    /// cover it and a selection question about it must refuse, never narrow).
    pub fn declared_repo_paths(&self) -> HashSet<String> {
        self.declared_paths.clone()
    }

    /// Does the facts pool declare this workspace-relative path as a module? The
    /// refuse-vs-answer membership test at entry grain — one hash lookup against
    /// the set precomputed at facts build.
    pub(crate) fn declares_repo_path(&self, rel: &str) -> bool {
        self.declared_paths.contains(rel)
    }
}

/// Mirror of `v2.lens.module_graph.path_matches_touched` (strip a leading `./`, then
/// equality or suffix-containment either way) — the two sides must agree because the
/// module-grain receipt harness proves the dag-vs-Rust file-grain DECISION equal on
/// real diffs, and a matching-rule fork would surface there as a divergence.
fn repo_paths_match_touched(closure_path: &str, touched_path: &str) -> bool {
    let file = closure_path.strip_prefix("./").unwrap_or(closure_path);
    let target = touched_path.strip_prefix("./").unwrap_or(touched_path);
    file == target || file.ends_with(target) || target.ends_with(file)
}

/// Production `entry_file_touched` decision for the discovery corpus: is any touched
/// entry file inside this entry's transitive import closure?
///
/// GRAIN (declared interim, operator fork (c) 2026-07-10): the module-graph
/// import-closure relation — the same host-realized facts
/// (`import_resolution_facts`/`module_declaration_facts` → `facts.adjacency`) that the
/// `.dag` authority `v2.lens.module_graph.entry_affected_by_touched_paths` reads through
/// its `_live` builtins. The `.dag` lens stays the modeled authority; THIS realization is
/// certified against it by execution on real merged diffs by the module-grain receipt
/// harness (`affected_decision_module_grain` section below). This consciously unwinds
/// one piece of #6335 ("reground selection on DependencyView") for this channel only:
/// the fn-arrow output-grain chain both silently widened (substrate-not-whole-tree →
/// touched=true for every row; probe receipt 2026-07-10) and conflates decl identity
/// with output type (a touched file's test fns put the shared `Bool` node in the edit
/// locus set). The fn-arrow machinery remains in tree as the decl-level candidate.
/// Dissolve-on: the namespace-only resolution terminal step replaces import edges with
/// `container.member` reference edges — the closure query above the edge source is
/// grain-stable and survives; the grain itself re-decides then (see
/// `entry_file_touched_grain_interim` in `v2.lens.affected_set.entry_selection`).
///
/// Refusal, never widen: an entry absent from the facts' declared-module set is a
/// provenance gap — the relation cannot answer for it — and returns a typed error that
/// fails the batch (the §5 fail-closed arm lives HERE, on an actual refusal; the old
/// arm that called its widen "fail-closed" is deleted).
fn entry_file_touched_via_import_closure(
    entry_path: &str,
    facts: &ModuleGraphFactsLive,
    declared_paths: &HashSet<String>,
    touched_paths: &[String],
) -> Result<bool, String> {
    if touched_paths.is_empty() {
        return Ok(false);
    }
    let entry_rel = workspace_relative_repo_path(entry_path);
    if !declared_paths.contains(&entry_rel) {
        return Err(format!(
            "AFFECTED-SET REFUSAL cause=EntryOutsideModuleGraphFacts entry={entry_rel} — \
             the module-graph facts scan does not declare this entry, so the import-closure \
             relation cannot answer entry_file_touched for it; refusing the batch rather \
             than widening to run-all or narrowing to skip"
        ));
    }
    // The edgeless case is NOT a special arm. Now that the adjacency carries reference-derived
    // edges for import-less files, "no outgoing edges" means the producer looked and found none —
    // a builtin-only witness, say — and `import_closure_from_adjacency` already answers precisely
    // for it: the closure seeds with the entry itself, so the entry is affected iff its own file
    // is touched. Falling through computes that.
    //
    // The one state that may refuse is the producer being UNABLE TO ANSWER: an import-less file it
    // could not read or parse has an unknown dependency set, which is not the same fact as an empty
    // one. Answering "affected" for it (the arm deleted here) conflated ⊤-as-ignorance with
    // ⊤-as-answer and, because the widen was silent and uncounted, drove the deficit's observed
    // frequency to zero by construction while the cost showed up as a 95-minute CI floor rather
    // than as a diagnostic (DESIGN §5).
    if facts.reference_unaccounted.contains(&entry_rel) {
        return Err(format!(
            "AFFECTED-SET REFUSAL cause=ReferenceEdgesUnaccounted entry={entry_rel} — the \
             reference-edge producer could not read or parse this import-less entry, so its \
             dependency set is UNKNOWN, not empty; refusing the batch rather than widening to \
             run-all or narrowing to skip"
        ));
    }
    let closure = import_closure_from_adjacency(entry_path, &facts.selection_adjacency);
    Ok(touched_paths.iter().any(|touched| {
        closure
            .iter()
            .any(|member| repo_paths_match_touched(member, touched))
    }))
}

/// Receipted Rust mirror of the single authority `v2.std.live_read.live_read_carrier_homes_v0`
/// (`src/v2/std/live_read.dag`) — the module names of the 8 declared live-read carrier homes.
/// Kept in lockstep with that `.dag` roster by hand; a drift here under-approximates axis (iv)
/// fail-closed-safe direction only if this list is a SUPERSET of the `.dag` roster, so any
/// addition to the `.dag` roster must be mirrored here — the drift gate below
/// (`live_read_carrier_home_modules_v0_is_superset_of_dag_authority`) evaluates the `.dag`
/// authority through a real interpreter context and fails the build the moment this const falls
/// behind, so the mismatch cannot silently pass.
/// Dissolution trigger: every caller of `runtime_data_dependency_touched_via_carrier_closure`
/// (the skip-before-resolve fast path and the precompute-count helpers below) is itself a named
/// `SCAFFOLD (§7 hand-Rust shrink-to-zero)` whose own DELETE WHEN note ties dissolution to
/// `v2.workflow.affected_set_floor_runner`'s `.dag` disposition owning the same predicate
/// end-to-end. This const has no independent dissolution path or generator lane because it has
/// no independent caller: when those scaffolds delete, this const and its drift gate delete with
/// them, not before.
const LIVE_READ_CARRIER_HOME_MODULES_V0: &[&str] = &[
    "v2.lens.enforcement.cost_coverage",
    "v2.lens.enforcement.grammar_coverage",
    "v2.lens.complexity_accumulator_copy.roster_gate",
    "v2.compiler.self_host",
    "v2.std.decl_index",
    "v2.lens.module_graph",
    "tools.dag_compile_clean_shard_roster",
    "tools.dag_compile_clean_scope",
];

/// Axis (iv) of the fourth-axis law (`docs/plans/live-read-witness-classification-design.md`
/// §7): does `entry_path`'s import closure reach a declared live-read carrier home, and is
/// any path touched at all? This is a G1-only (module-closure) mirror of the landed G2
/// call-reachability lens (`v2.lens.live_read_classification`) — G2's carrier set is always
/// a superset of G1's under the same closure (`merge_g1_and_g2_carriers`), so this coarser
/// Rust check is fail-closed-safe relative to the full `.dag` authority: it may over-report
/// (an extra witness run) but never under-report (a missed run). It does not attempt to
/// prove which touched path a reached carrier actually reads at runtime (that precision is
/// G2/G3's job) — reachability plus any touch is treated as a hit.
fn import_closure_module_reaches_carrier_home(
    closure_modules: &HashSet<String>,
    carrier_home: &str,
) -> bool {
    closure_modules.iter().any(|module| {
        module == carrier_home
            || module
                .strip_prefix(carrier_home)
                .is_some_and(|suffix| suffix.starts_with('.'))
    })
}

fn runtime_data_dependency_touched_via_carrier_closure(
    entry_path: &str,
    facts: &ModuleGraphFactsLive,
    touched_paths: &[String],
) -> bool {
    if touched_paths.is_empty() {
        return false;
    }
    let mut closure_modules: HashSet<String> = HashSet::new();
    collect_import_closure_module_names_from_facts(entry_path, facts, &mut closure_modules);
    LIVE_READ_CARRIER_HOME_MODULES_V0
        .iter()
        .any(|carrier_home| {
            import_closure_module_reaches_carrier_home(&closure_modules, carrier_home)
        })
}

#[cfg(test)]
mod live_read_carrier_home_roster_drift_gate_tests {
    use super::{
        build_multi_entry_index, make_eval_context, resolve_entry_with_index_for_discovery_corpus,
        workspace_root, LIVE_READ_CARRIER_HOME_MODULES_V0,
    };
    use crate::v1_interpreter::{self, ExecutionMode, Value};
    use std::collections::HashSet;

    const LIVE_READ_ENTRY: &str = "src/v2/std/live_read.dag";

    fn record_field<'a>(
        ctx: &v1_interpreter::InterpContext,
        fields: &'a [(v1_interpreter::Symbol, Value)],
        field_name: &str,
    ) -> Option<&'a Value> {
        fields
            .iter()
            .find(|(sym, _)| ctx.sym_eq(*sym, field_name))
            .map(|(_, v)| v)
    }

    // Reads the `.dag`-side single authority directly by evaluating
    // `live_read_carrier_homes_v0` in a real interpreter context — not a re-hand-authored Rust
    // copy of the roster — so this test's own expectation cannot drift the same way the
    // production const can.
    fn dag_carrier_home_modules() -> HashSet<String> {
        std::env::set_current_dir(workspace_root()).expect("chdir workspace");
        let index = build_multi_entry_index(&["dag".to_string(), "src/v2".to_string()]);
        let (graph, indices) =
            resolve_entry_with_index_for_discovery_corpus(&index, LIVE_READ_ENTRY)
                .unwrap_or_else(|e| panic!("resolve {LIVE_READ_ENTRY}: {e}"));
        let ctx = make_eval_context(&graph, indices, ExecutionMode::Wet);
        let val = v1_interpreter::with_active_context(&ctx, || {
            v1_interpreter::eval_data_item_value(&ctx, "live_read_carrier_homes_v0")
        })
        .unwrap_or_else(|e| panic!("eval live_read_carrier_homes_v0: {e}"))
        .unwrap_or_else(|| panic!("live_read_carrier_homes_v0 not found as a data item"));
        let Value::List(items) = val else {
            panic!("live_read_carrier_homes_v0 is not a List: {val:?}");
        };
        items
            .iter()
            .map(|item| {
                let Value::Record { fields, .. } = item else {
                    panic!("live_read_carrier_homes_v0 entry is not a Record: {item:?}");
                };
                match record_field(&ctx, fields, "module") {
                    Some(Value::Str(s)) => s.clone(),
                    other => panic!("LiveReadCarrierHome.module is not a String: {other:?}"),
                }
            })
            .collect()
    }

    // The safety-critical drift direction (axis (iv)'s fail-closed-safe requirement, see the
    // doc-comment on `LIVE_READ_CARRIER_HOME_MODULES_V0`): the Rust const must be a SUPERSET of
    // the `.dag` roster. A `.dag`-only addition this const misses makes
    // `runtime_data_dependency_touched_via_carrier_closure` return `false` for that carrier —
    // silently fail-open on the exact axis this const backs.
    #[test]
    fn live_read_carrier_home_modules_v0_is_superset_of_dag_authority() {
        let dag_modules = dag_carrier_home_modules();
        let rust_modules: HashSet<String> = LIVE_READ_CARRIER_HOME_MODULES_V0
            .iter()
            .map(|s| s.to_string())
            .collect();
        let missing: Vec<&String> = dag_modules.difference(&rust_modules).collect();
        assert!(
            missing.is_empty(),
            "`.dag` authority `live_read_carrier_homes_v0` declares carrier home module(s) \
             {missing:?} not mirrored in Rust `LIVE_READ_CARRIER_HOME_MODULES_V0` \
             (src/v1/stage0/src/cli_run.rs) — add them there or axis (iv) silently fails open \
             for that carrier"
        );
    }
}

// SCAFFOLD (§7 HAND-RUST — `cli_run_discovery_skip_before_resolve`):
// ROADMAP lane `2-provenance-ingest` (gunbc.roadmap_authority / ROADMAP.md;
// docs/plans/affected-set-precompute-pruning.md Step 4 migrate floor) — host-side
// per-entry cold-resolve elision under SelectionApplied before `floor_kernel_would_skip`.
// Unblock: modeled `floor_kernel_precompute_would_skip` / skip-before-resolve arm on
// `v2.workflow.affected_set_floor_runner` realizes the same decision in `.dag` (N→1 with
// the Rust `NodeFrontierSeeds` parallel deleted per the de-fork plan).
// DELETE WHEN dissolved: `entry_eligible_for_discovery_skip_before_resolve`,
// `collect_import_closure_module_names_from_facts`, `resolve_discovery_entry_for_corpus_row`
// lazy-resolve arm, and the `SKIP-RESOLVE` / `ctx.is_none()` loop in `run_discovery_rows`
// (~120 LOC).
// Receipt: `rg cli_run_discovery_skip_before_resolve src/v1/stage0/src/cli_run.rs` == 1 until
// deletion; not a compiler_frontier `.dag` row (seed-Rust, counted here not in module census).
pub(crate) const CLI_RUN_DISCOVERY_SKIP_BEFORE_RESOLVE_SCAFFOLD_MARKER: &str =
    "cli_run_discovery_skip_before_resolve";

/// Import-closure module names from the module-graph facts scan — the same grain as
/// `roster_import_closure_nodes_pre_resolve`, used when skip-before-resolve elides a cold
/// entry resolve so the post-resolve calibration union stays aligned with the pre-resolve walk.
///
/// INTERIM hand-Rust scaffold (`CLI_RUN_DISCOVERY_SKIP_BEFORE_RESOLVE_SCAFFOLD_MARKER` / §7):
/// dissolves under ROADMAP `2-provenance-ingest` when the floor runner `.dag` owns the decision.
fn collect_import_closure_module_names_from_facts(
    entry_path: &str,
    facts: &ModuleGraphFactsLive,
    out: &mut HashSet<String>,
) {
    let closure_paths: HashSet<String> = import_closure_live_paths_with_facts(entry_path, facts)
        .into_iter()
        .map(|p| workspace_relative_repo_path(&p))
        .collect();
    for node in &facts.nodes {
        let rel = workspace_relative_repo_path(&node.path);
        if closure_paths.contains(&rel)
            || closure_paths
                .iter()
                .any(|closure_path| repo_paths_match_touched(closure_path, &rel))
        {
            out.insert(node.module.clone());
        }
    }
}

fn entry_has_edited_test_fn_in_entry(diff_edits: &FloorDiffEdits, entry_path: &str) -> bool {
    diff_edits
        .edited_test_fns
        .iter()
        .any(|(file, _)| diff_file_matches_entry(file, entry_path))
}

/// Skip-before-resolve (discovery corpus, SelectionApplied): when the diff cannot possibly
/// affect any witness in this entry — outside import-closure, no edited test fn, not a
/// host-scaffold entry file — elide the cold entry resolve and treat every kernel witness
/// row as assumed-green.
///
/// INTERIM hand-Rust scaffold (`CLI_RUN_DISCOVERY_SKIP_BEFORE_RESOLVE_SCAFFOLD_MARKER` / §7):
/// dissolves under ROADMAP `2-provenance-ingest` when `floor_kernel_precompute_would_skip` in
/// `v2.workflow.affected_set_floor_runner` is the general per-entry authority.
fn entry_eligible_for_discovery_skip_before_resolve(
    skip_enabled: bool,
    reads_live_tree: bool,
    entry_path: &str,
    facts: &ModuleGraphFactsLive,
    declared_paths: &HashSet<String>,
    touched_paths: &[String],
    diff_edits: &FloorDiffEdits,
) -> Result<bool, String> {
    if !skip_enabled {
        return Ok(false);
    }
    // Fail-closed on the substrate-declared disposition (v2.std.live_tree): a
    // `ReadsLiveTree` entry reads state outside its resolved closure, so the diff
    // cannot bound its inputs — it never predict-skips. Replaces the deleted
    // entry-text classifier (`entry_text_indicates_live_host_scan`); the disposition
    // is entry-grain, parsed onto every DiscoveryRow of the entry.
    if reads_live_tree {
        return Ok(false);
    }
    if entry_has_edited_test_fn_in_entry(diff_edits, entry_path) {
        return Ok(false);
    }
    if entry_file_touched_via_import_closure(entry_path, facts, declared_paths, touched_paths)? {
        return Ok(false);
    }
    if runtime_data_dependency_touched_via_carrier_closure(entry_path, facts, touched_paths) {
        return Ok(false);
    }
    let declared_axis = declared_source_refs_axis_for_entry(
        entry_path,
        facts,
        &default_source_roots(),
        touched_paths,
    );
    if declared_axis != DeclaredSourceRefAxis::Absent {
        if declared_source_refs_blocks_skip(declared_axis) {
            return Ok(false);
        }
    } else if effect_reach_touched_via_path_literals(entry_path, facts, touched_paths) {
        return Ok(false);
    }
    Ok(true)
}

struct DiscoveryEntryResolve {
    ctx: v1_interpreter::InterpContext,
    closure_subject: String,
    frontier_nodes: Vec<v1_interpreter::Value>,
    touches_frontier: bool,
    entry_file_touched: bool,
    entry_runtime_dependency_touched: bool,
    resolve_nanos: u128,
    stage_nanos: ResolveStageNanos,
}

fn resolve_discovery_entry_for_corpus_row(
    index: &MultiEntryIndex,
    entry_path: &str,
    execution_mode: v1_interpreter::ExecutionMode,
    whole_tree_published_keys: Option<Rc<std::collections::HashSet<String>>>,
    skip_enabled: bool,
    diff_edits: &FloorDiffEdits,
    touched_entry_paths: &[String],
    module_graph_declared_paths: &HashSet<String>,
    closure_modules: &mut HashSet<String>,
) -> Result<DiscoveryEntryResolve, String> {
    let sources = load_sources_for_entry_with_pool(index, entry_path)
        .map_err(|msg| format!("load sources failed for {entry_path}: {msg}"))?;
    let closure_subject = subject_digest_for_closure(&sources);
    let resolve_started = std::time::Instant::now();
    set_phase(FloorPhase::Resolve, entry_path);
    let (graph, source_indices) = resolve_entry_with_index_for_discovery_corpus(index, entry_path)
        .map_err(|msg| format!("resolve failed for {entry_path}: {msg}"))?;
    let resolve_nanos = resolve_started.elapsed().as_nanos();
    // Same thread, immediately after the resolve that filled it: this entry's split.
    let stage_nanos = resolve_stage_slot_snapshot();
    collect_typed_module_names(
        graph.modules.iter().cloned(),
        &source_indices,
        closure_modules,
    );
    let entry_ctx = make_eval_context_with_runtime_options(
        &graph,
        source_indices,
        execution_mode,
        None,
        whole_tree_published_keys,
    );
    let (frontier_nodes, touches_frontier, entry_file_touched, entry_runtime_dependency_touched) =
        if skip_enabled {
            let frontier_nodes =
                rerun_frontier_nodes_for_entry(&entry_ctx, entry_path, diff_edits)?;
            let touches_frontier = if frontier_nodes.is_empty() {
                false
            } else {
                entry_touches_rerun_frontier(
                    &entry_ctx,
                    &list_value_from_vec(frontier_nodes.clone()),
                )?
            };
            let entry_file_touched = if touched_entry_paths.is_empty() {
                false
            } else {
                entry_file_touched_via_import_closure(
                    entry_path,
                    &index.module_graph_facts,
                    module_graph_declared_paths,
                    touched_entry_paths,
                )?
            };
            let declared_axis = declared_source_refs_axis_for_entry(
                entry_path,
                &index.module_graph_facts,
                &default_source_roots(),
                touched_entry_paths,
            );
            let entry_runtime_dependency_touched =
                runtime_data_dependency_touched_via_carrier_closure(
                    entry_path,
                    &index.module_graph_facts,
                    touched_entry_paths,
                ) || match declared_axis {
                    DeclaredSourceRefAxis::Absent => effect_reach_touched_via_path_literals(
                        entry_path,
                        &index.module_graph_facts,
                        touched_entry_paths,
                    ),
                    DeclaredSourceRefAxis::Touched | DeclaredSourceRefAxis::Unresolved => true,
                    DeclaredSourceRefAxis::Untouched => false,
                };
            (
                frontier_nodes,
                touches_frontier,
                entry_file_touched,
                entry_runtime_dependency_touched,
            )
        } else {
            (Vec::new(), true, true, true)
        };
    Ok(DiscoveryEntryResolve {
        ctx: entry_ctx,
        closure_subject,
        frontier_nodes,
        touches_frontier,
        entry_file_touched,
        entry_runtime_dependency_touched,
        resolve_nanos,
        stage_nanos,
    })
}

/// The closure-node definition SHARED by the falsifier/floor calibration emission and
/// the space-lens memory predictor (single authority is THIS function — the predictor
/// binds to it, never a re-derivation; predictor design is in flight on PR #6442, and
/// the landed parent-lane authorities are docs/plans/compute-envelope-model.md (fleet
/// envelope) and docs/plans/input-envelope-roadmap.md (admission)): the deduped transitive
/// import-closure of every roster row plus the given prefix-context entries, counted at
/// the module-path grain via the same both-closure loader as post-resolve
/// (`extend_sources_to_both_closure_fixpoint`; import walk + bare-reference pull for
/// import-stripped modules). On a completed width-1 run this equals the post-resolve
/// resolved-graph union (`DiscoverySummary.roster_closure_nodes`), and
/// `run_discovery_corpus_with_options` asserts that equality as the definition-drift
/// oracle (a loader fork or seeding change localizes here instead of silently skewing
/// bytes-per-node).
fn collect_both_closure_module_names_for_entry(
    index: &MultiEntryIndex,
    entry_path: &str,
    out: &mut HashSet<String>,
) -> Result<(), String> {
    let sources = load_sources_for_entry_with_pool(index, entry_path)?;
    let closure_paths: HashSet<String> = sources
        .iter()
        .map(|s| workspace_relative_repo_path(&s.path))
        .collect();
    for node in &index.module_graph_facts.nodes {
        let rel = workspace_relative_repo_path(&node.path);
        if closure_paths.contains(&rel)
            || closure_paths
                .iter()
                .any(|closure_path| repo_paths_match_touched(closure_path, &rel))
        {
            out.insert(node.module.clone());
        }
    }
    Ok(())
}

pub fn roster_import_closure_nodes_pre_resolve(
    rows: &[DiscoveryRow],
    prefix_entries: &[&str],
    index: &MultiEntryIndex,
) -> Result<usize, String> {
    let mut closure_modules: HashSet<String> = HashSet::new();
    for entry in rows
        .iter()
        .map(|r| r.entry.as_str())
        .chain(prefix_entries.iter().copied())
    {
        collect_both_closure_module_names_for_entry(index, entry, &mut closure_modules)?;
    }
    Ok(closure_modules.len())
}

#[cfg(test)]
fn resolve_transitively_bfs_legacy(
    entry_sources: Vec<Rc<v1_compiler_compile::SourceFile>>,
    index: &ModuleSourceIndex,
    mut seen: HashMap<String, Rc<v1_compiler_compile::SourceFile>>,
) -> Vec<Rc<v1_compiler_compile::SourceFile>> {
    let mut queue = entry_sources;
    while let Some(source) = queue.pop() {
        for module_path in extract_import_paths(&source.content) {
            if seen.contains_key(&module_path) {
                continue;
            }
            if let Some(imported) = index.get(&module_path) {
                seen.insert(module_path, imported.clone());
                queue.push(imported.clone());
            }
        }
    }
    let mut result: Vec<_> = seen.into_iter().map(|(_, v)| v).collect();
    result.sort_by(|a, b| a.path.cmp(&b.path));
    result
}

fn resolve_transitively(
    entry_sources: Vec<Rc<v1_compiler_compile::SourceFile>>,
    index: &ModuleSourceIndex,
    facts: &ModuleGraphFactsLive,
) -> Result<Vec<Rc<v1_compiler_compile::SourceFile>>, String> {
    let mut path_lookup = path_to_source_lookup(index);
    for entry in &entry_sources {
        let rel = workspace_relative_entry_path(&entry.path);
        path_lookup.entry(rel).or_insert_with(|| entry.clone());
        path_lookup
            .entry(entry.path.clone())
            .or_insert_with(|| entry.clone());
    }

    let mut all_paths: BTreeSet<String> = BTreeSet::new();
    for entry in &entry_sources {
        let entry_rel = workspace_relative_entry_path(&entry.path);
        // An entry the facts pool does not declare has NO import edges in the
        // adjacency — not because it imports nothing, but because the scan never
        // saw it. Answering with the entry-only closure would silently drop its
        // imports and surface downstream as `unresolved import` on modules that
        // exist (the interp_recorded fixture-witness dark red, masked since the
        // facts repoint in #6210). Refuse, never narrow (DESIGN §5).
        if !facts.declares_repo_path(&entry_rel) {
            return Err(format!(
                "import_closure_live: entry '{entry_rel}' has no provenance in the \
                 module-graph facts pool (outside every source root, or missing a \
                 module declaration), so its import closure cannot be derived \
                 (fail-closed); pass a --source-root that covers the entry — the \
                 module universe is workspace-anchored"
            ));
        }
        for path in import_closure_live_paths_with_facts(&entry_rel, facts) {
            all_paths.insert(workspace_relative_repo_path(&path));
        }
    }

    let mut result = Vec::with_capacity(all_paths.len());
    for path in all_paths {
        let sf = path_lookup.get(&path).cloned().ok_or_else(|| {
            format!(
                "import_closure_live: closure path '{path}' has no provenance in module index (fail-closed)"
            )
        })?;
        result.push(sf);
    }
    result.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(result)
}

pub fn load_sources_for_entry(
    source_roots: &[String],
    entry_path: &str,
) -> Result<Vec<Rc<v1_compiler_compile::SourceFile>>, String> {
    let index = build_multi_entry_index(source_roots);
    load_sources_for_entry_with_pool(&index, entry_path)
}

/// Builtins that REQUIRE a service registration to dispatch, paired with the
/// services-census key whose provider must therefore be in the closure. This is
/// the one dependency edge a name-derived closure cannot see: the builtin's
/// identifier is the interpreter's, not any module's, so no census lookup
/// reaches the provider — under imports the edge was a name-less `import
/// extdeps.filesystem.filesystem_io`, which the strip removed and nothing can
/// re-derive.
///
/// Rows mirror the hard `ctx.service_ops.contains_key(..)` REQUIREMENT gates in
/// `v1_interpreter` — not the optional ones (`Clock.UnixSecs`,
/// `shell.Env.Get`), which fall back to a transport when the service is absent
/// and so create no closure obligation. A missing row cannot fabricate a
/// result: the builtin's own gate still refuses, typed and located (that
/// refusal is exactly how this row was found — `interp_recorded_fixture`'s
/// nested replay, CI 29722434993).
/// Pairs are (builtin identifier, SERVICES-CENSUS key). The census key is the
/// service's authored name (`Filesystem`, or a dotted one like `cron.Tab`) —
/// NOT the interpreter's `service_ops` operation key (`Filesystem.Read`), which
/// appends the operation.
const BUILTIN_REQUIRED_SERVICE_KEYS: &[(&str, &str)] = &[("filesystem_read", "Filesystem")];

/// Bare (non-dotted) identifiers in `content`, split by whether any occurrence
/// sits in CALL POSITION (immediately followed by `(`). The split is the pull
/// discriminator: the census strips fn bodies, so a 0-arg fn and a type alias
/// share a census shape ("pending a discriminator", census note) — but a 0-arg
/// fn is referenced `name()` while a type name never is. Deliberately an
/// over-approximation on the name axis (locals and keywords are included): a
/// false candidate costs a census map miss, never a wrong closure. String
/// literals are skipped for the same reason as the dotted scan.
struct BareCandidates {
    names: BTreeSet<String>,
    call_position: BTreeSet<String>,
    /// Full dotted chains (`cron.Tab.List` in `cron.Tab.List()`): the dotted
    /// module-path scan owns chains whose prefix is a module path, but a
    /// SERVICE reference's prefix is a services-census key (`cron.Tab`,
    /// `llm.Codex` — service names are themselves dotted) with no module
    /// spelling — with its import stripped, only the services census can name
    /// its provider module. The consumer tries each dot-prefix of the chain
    /// against the services census.
    dotted_chains: BTreeSet<String>,
    /// Names seen in BINDING position (`let repo`, `data x`) or KEY position
    /// (`repo:` — field init, named arg, param decl) anywhere in the file. A
    /// dotted-chain head in this set is a local/param/field projection
    /// (`repo.operation_name`), never a cross-module data-const reference —
    /// consulted to keep dotted-head pulls from re-opening the over-pull the
    /// binder/key lexer closed for bare names.
    bound: BTreeSet<String>,
}

fn bare_identifier_candidates(content: &str) -> BareCandidates {
    let bytes = content.as_bytes();
    let is_ident_start = |c: u8| c.is_ascii_alphabetic() || c == b'_';
    let is_ident = |c: u8| c.is_ascii_alphanumeric() || c == b'_';
    let mut out = BareCandidates {
        names: BTreeSet::new(),
        call_position: BTreeSet::new(),
        dotted_chains: BTreeSet::new(),
        bound: BTreeSet::new(),
    };
    let mut i = 0usize;
    // Previous identifier token on the same run (whitespace-separated): a name
    // directly after a BINDER keyword is a binding occurrence, not a reference —
    // `let repo = ...` must not pull the module that declares a census-unique
    // `data repo` (measured: it coupled the gunbhub witness to the review-agent
    // tooling's health). Cleared by any non-ident, non-whitespace byte.
    let mut prev_token: Option<&str> = None;
    let binder_keywords = [
        "let",
        "data",
        "fn",
        "type",
        "import",
        "module",
        "service",
        "transport",
    ];
    // Depth of an open `fn(`-literal parameter list: every ident inside is a
    // BINDER (untyped lambda params — `fn(acc, edge)` — carry no `:` so the
    // key-position rule never sees them; measured: rust_test.dag's `fn(acc,
    // edge)` param leaked 'edge' into the reference set and pulled the
    // unresolvable ownership_movable test module into an unrelated entry).
    // Typed idents inside (`p: T`, and type names in `fn(A) -> B` annotations)
    // over-bind harmlessly: a suppressed pull fails LOUD at typecheck.
    let mut fn_params_depth: usize = 0;
    while i < bytes.len() {
        if bytes[i] == b'"' {
            i += 1;
            while i < bytes.len() && bytes[i] != b'"' {
                if bytes[i] == b'\\' && i + 1 < bytes.len() {
                    i += 1;
                }
                i += 1;
            }
            i += 1;
            prev_token = None;
            continue;
        }
        if !is_ident_start(bytes[i]) || (i > 0 && (is_ident(bytes[i - 1]) || bytes[i - 1] == b'.'))
        {
            if bytes[i] == b'(' {
                if prev_token == Some("fn") {
                    fn_params_depth = 1;
                } else if fn_params_depth > 0 {
                    fn_params_depth += 1;
                }
            } else if bytes[i] == b')' && fn_params_depth > 0 {
                fn_params_depth -= 1;
            }
            if !bytes[i].is_ascii_whitespace() {
                prev_token = None;
            }
            i += 1;
            continue;
        }
        let start = i;
        while i < bytes.len() && is_ident(bytes[i]) {
            i += 1;
        }
        // Part of a dotted chain → the dotted scan owns module-path chains, but
        // record the FULL chain: a service reference (`cron.Tab.List()`) is a
        // dotted chain whose prefix is a services-census key, not a module path.
        if i < bytes.len() && bytes[i] == b'.' {
            while i < bytes.len()
                && bytes[i] == b'.'
                && i + 1 < bytes.len()
                && is_ident_start(bytes[i + 1])
            {
                i += 1;
                while i < bytes.len() && is_ident(bytes[i]) {
                    i += 1;
                }
            }
            out.dotted_chains.insert(content[start..i].to_string());
            prev_token = None;
            continue;
        }
        let name = &content[start..i];
        // Binding occurrence (`let repo`, `data repo`) — a name being BOUND is
        // never a reference to another module's decl.
        if fn_params_depth > 0 {
            out.bound.insert(name.to_string());
            prev_token = Some(name);
            continue;
        }
        if prev_token.is_some_and(|t| binder_keywords.contains(&t)) {
            out.bound.insert(name.to_string());
            prev_token = Some(name);
            continue;
        }
        // Key position (`repo: value` — field init, named arg, param decl):
        // the name labels a slot; it never references a decl.
        let mut peek = i;
        while peek < bytes.len() && (bytes[peek] == b' ' || bytes[peek] == b'\t') {
            peek += 1;
        }
        if peek < bytes.len() && bytes[peek] == b':' {
            out.bound.insert(name.to_string());
            // A key is not a binder-keyword context: `type: User` must leave
            // `User` a collectable reference — carrying `type` forward as
            // prev_token made the binder-keyword rule swallow the VALUE after
            // any key that happens to spell a keyword.
            prev_token = None;
            continue;
        }
        if i < bytes.len() && bytes[i] == b'(' {
            out.call_position.insert(name.to_string());
        }
        out.names.insert(name.to_string());
        prev_token = Some(name);
    }
    out
}

/// Extend the closure with the modules the tree census resolves each source's
/// BARE references to (namespace Rule-1 direction: deps derived from names, not
/// import statements — the import-stripped corpus has no import edges to follow,
/// which surfaced as `no such function` at witness runtime: typecheck resolved a
/// name through the census while the interpreter never loaded its body). A bare
/// name resolves exactly as the typecheck lookup will: census-unique → that
/// module; ambiguous → the nearest-ancestor candidate from the referencing
/// module's containment position; still ambiguous → load nothing (the typecheck
/// refusal stays the loud authority, the loader never guesses a side).
fn extend_with_bare_reference_closure(
    mut sources: Vec<Rc<v1_compiler_compile::SourceFile>>,
    index: &MultiEntryIndex,
) -> Result<Vec<Rc<v1_compiler_compile::SourceFile>>, String> {
    use crate::v1_compiler_infer_env::GlobalBareLookupState;
    let path_lookup = path_to_source_lookup(&index.source_files);
    let mut known_paths: std::collections::HashSet<String> = sources
        .iter()
        .flat_map(|s| [s.path.clone(), workspace_relative_repo_path(&s.path)])
        .collect();
    let mut scan_queue: Vec<Rc<v1_compiler_compile::SourceFile>> = sources.clone();
    while let Some(sf) = scan_queue.pop() {
        // Name-derived pulls serve IMPORT-STRIPPED modules only. A module that
        // declares imports is main-parity: its closure derives from those edges,
        // and scanning its bare names manufactures over-pull (measured: bare
        // 'repo' in an unstripped extdeps module pulled the review-agent tooling
        // into a merge-admission run, coupling the run to modules it never
        // executes — and to their health).
        if sf
            .content
            .lines()
            .any(|l| l.trim_start().starts_with("import "))
        {
            continue;
        }
        let file_rel = workspace_relative_repo_path(&sf.path);
        let Some(root) = source_tree_root_of(&index.source_roots, &file_rel) else {
            continue;
        };
        let census = tree_bare_census_for_root(index, &root)?;
        let referencing_module = extract_module_path(&sf.content).unwrap_or_default();
        let candidates = bare_identifier_candidates(&sf.content);
        // Dotted-chain prefixes that name a SERVICE pull its provider module:
        // with the import stripped (`import extdeps.cron` gone from
        // cron_tag.dag), `cron.Tab.List()` has no module-path spelling the
        // dotted scan can follow — the services census key (`cron.Tab`; service
        // names are themselves dotted) is the only edge back to the provider.
        // A module-path chain (`v2.std....`) misses the services census: no pull.
        let mut service_prefixes: BTreeSet<String> = candidates
            .dotted_chains
            .iter()
            .flat_map(|chain| {
                let mut prefixes = Vec::new();
                let mut acc = String::new();
                for seg in chain.split('.') {
                    if !acc.is_empty() {
                        acc.push('.');
                    }
                    acc.push_str(seg);
                    prefixes.push(acc.clone());
                }
                prefixes
            })
            .collect();
        // SIDE-EFFECT-ONLY dependency, the one edge names alone cannot carry: a
        // BUILTIN that dispatches through a service (`filesystem_read` →
        // `Filesystem.Read`) needs the provider module LOADED for its service
        // registration, but contributes no name the census could resolve — the
        // builtin's own identifier belongs to the interpreter, not to any
        // module. Under imports that edge was a name-less `import
        // extdeps.filesystem.filesystem_io`; stripped, it is underivable, so
        // the requirement each builtin already ENFORCES at dispatch is declared
        // here as the pull key and resolved through the same services census a
        // dotted service head uses. Keep in lockstep with the
        // `ctx.service_ops.contains_key(..)` gates in v1_interpreter (each gate
        // is a row here); a missing row does not fabricate — the builtin's gate
        // still refuses, typed and located, exactly as it did for this row.
        for (builtin_name, service_key) in BUILTIN_REQUIRED_SERVICE_KEYS {
            if candidates.call_position.contains(*builtin_name) {
                service_prefixes.insert((*service_key).to_string());
            }
        }
        // A dotted-chain HEAD that is never body-bound in this file is a
        // cross-module data-const projection (`gunbc_ci_spec.diff_policy`) —
        // resolve it like a bare reference so the declaring module is pulled
        // (its census binding carries the data const's type annotation, the
        // same pull rule bare data refs use). Bound heads (`repo.x` on a let)
        // stay excluded — that over-pull class is closed.
        let dotted_head_refs: Vec<String> = candidates
            .dotted_chains
            .iter()
            .filter_map(|chain| chain.split('.').next())
            .filter(|h| !candidates.bound.contains(*h) && !candidates.names.contains(*h))
            .map(|h| h.to_string())
            .collect();
        // A name this file BINDS anywhere (binder keyword, key position — fn
        // params, named-arg keys, let/data/type decls) is served locally at
        // every scope the reference could sit in; pulling its census homonym
        // couples the entry to an unrelated module (measured: lens
        // unit_modeling's `edge` param pulled v2.test.manual.ownership_movable,
        // whose src/v1 imports are unresolvable in this pool — resolve died on
        // a module the entry never executes). Same rule dotted-chain heads
        // already apply; a genuinely-global reference shadowed by a same-name
        // local binder elsewhere in the file fails LOUD at typecheck/runtime
        // (no such function), never silently wrong.
        let all_names: Vec<(String, bool)> = candidates
            .names
            .iter()
            .filter(|n| !candidates.bound.contains(*n))
            .map(|n| (n.clone(), false))
            .chain(dotted_head_refs.into_iter().map(|n| (n, false)))
            .chain(service_prefixes.into_iter().map(|n| (n, true)))
            .collect();
        for (name, service_head) in all_names {
            // Only CALLABLE-shaped references pull a module: the runtime gap this
            // closure exists for is the interpreter's fn/service registry
            // (`no such function`); types and variants are census-served at
            // typecheck and carried by value tags at runtime. A reference is
            // callable-shaped when it occurs in call position (`name(` — the
            // only way a 0-arg fn is used, and a shape a type name never has) or
            // when its census binding carries value params (a named fn passed as
            // an argument). Pulling for anything else only widens the closure —
            // and a widened closure couples this witness to the health of
            // modules it never executes (measured: an over-pulled v2 test
            // module red under the entry's tree view killed an unrelated dag
            // witness).
            let in_call_position = candidates.call_position.contains(&name);
            // Pullable = fn referenced in call position, named fn passed as a
            // value (census sig carries params), or a DATA const (the census
            // stub keeps `type_annotation` — `data x: T = ...` — while type
            // decls never carry one); data consts are runtime values referenced
            // bare (`design_argument`, `srv1_nvme0`: 42 counted
            // no-such-function rows in the first full batch-2).
            let pullable = |binding: &Rc<crate::v1_compiler_infer_env::TypeBinding>| {
                in_call_position
                    || !binding.resolved.params.is_empty()
                    || binding.resolved.type_annotation.is_some()
                    // A TYPE decl (Disj/Conj) referenced bare is a real Rule-1
                    // edge: typecheck is census-served, but RUNTIME construction
                    // and variant-tag identity need the declaring module loaded
                    // (re-eval of `{ type: User }` died `undefined variable:
                    // User` with the enum's module unpulled). Binder/key-position
                    // occurrences are already excluded, so the residual
                    // over-pull is a census-unique type homonym of a plain
                    // local — bounded and rare.
                    || binding.resolved.connective != crate::v1_std_core::Connective::NoConnective
            };
            let resolve_in = |census: &Rc<SymbolIndex>| -> Option<String> {
                if service_head {
                    return v1_rt::map_get(&census.services, name.clone())
                        .map(|entry| entry.module_path.clone());
                }
                match v1_rt::map_get(&census.global_bare, name.clone()) {
                    Some(state) => match state.as_ref() {
                        GlobalBareLookupState::GlobalBareUniqueBinding {
                            module_path,
                            binding,
                        } => {
                            if pullable(binding) {
                                Some(module_path.clone())
                            } else {
                                None
                            }
                        }
                        GlobalBareLookupState::GlobalBareAmbiguousBinding { candidates } => {
                            crate::v1_compiler_infer_env::global_bare_nearest_ancestor_candidate(
                                referencing_module.clone(),
                                candidates.clone(),
                            )
                            .and_then(|c| {
                                if pullable(&c.binding) {
                                    Some(c.module_path.clone())
                                } else {
                                    None
                                }
                            })
                        }
                    },
                    None => {
                        if in_call_position {
                            v1_rt::map_get(&census.services, name.clone())
                                .map(|entry| entry.module_path.clone())
                        } else {
                            None
                        }
                    }
                }
            };
            // Own tree first (same-tree names keep priority — a cross-tree
            // homonym can never steal one); the whole-pool census only fills an
            // own-tree MISS, giving cross-tree references their provider pull
            // (a v2 module's bare `gunbc_ci_spec` → dag/gunbc/ci_spec.dag).
            let target_module = match resolve_in(&census) {
                Some(m) => Some(m),
                None => resolve_in(&pool_bare_census(index)?),
            };
            let Some(module_path) = target_module else {
                continue;
            };
            // Same class as the provenance refusal below, and it refuses for the same
            // reason: the census RESOLVED this name to a module, so the pool asserted the
            // module exists — a missing source file is the pool contradicting itself, not
            // a name that failed to resolve. Dropping it silently would widen the
            // "unresolved name" bucket with a pool defect whose frequency then reads zero
            // (DESIGN 5: a failure arm must refuse, never widen).
            let Some(dep) = index.source_files.get(&module_path) else {
                return Err(format!(
                    "bare_reference_closure: census resolved '{name}' in '{file_rel}' to \
                     module '{module_path}', but that module has no source file in the pool \
                     (fail-closed)"
                ));
            };
            // A `test fn`/`test data` ROW is an execution ROOT, never a
            // dependency: a bare homonym resolving to another module's witness
            // must not couple this entry to its run. The guard is
            // DECLARATION-grain: only when the resolved name itself is declared
            // as a test row in the provider. A PLAIN fn/data declared beside
            // test rows is an ordinary provider — the former FILE-grain skip
            // (any provider containing a `test fn` line) silently dropped
            // those, converting a resolvable cross-file reference into a
            // runtime `no such function` (infer_emit_compile_anchor.dag →
            // anchor_rust_add_emit_accepts; review finding, 2026-07-19).
            let name_is_test_row = dep.content.lines().any(|l| {
                let t = l.trim_start();
                ["test fn ", "test data "].iter().any(|prefix| {
                    t.strip_prefix(prefix).is_some_and(|rest| {
                        rest.strip_prefix(name.as_str()).is_some_and(|after| {
                            after
                                .chars()
                                .next()
                                .is_none_or(|c| !c.is_alphanumeric() && c != '_')
                        })
                    })
                })
            });
            if name_is_test_row {
                continue;
            }
            let dep_rel = workspace_relative_repo_path(&dep.path);
            if known_paths.contains(&dep_rel) || known_paths.contains(&dep.path) {
                continue;
            }
            // Diagnostic read-only trace of the pull edge (name → module), for
            // locating over-pull homonyms; never alters the closure.
            if std::env::var("GUNBC_BARE_PULL_TRACE").is_ok() {
                eprintln!(
                    "[bare-pull] {} -> '{}' -> {} ({})",
                    file_rel, name, module_path, dep_rel
                );
            }
            if !index.module_graph_facts.declares_repo_path(&dep_rel) {
                return Err(format!(
                    "bare_reference_closure: referenced module '{module_path}' at \
                     '{dep_rel}' has no provenance in the module-graph facts pool \
                     (fail-closed)"
                ));
            }
            for path in import_closure_live_paths_with_facts(&dep_rel, &index.module_graph_facts) {
                let rel = workspace_relative_repo_path(&path);
                if known_paths.contains(&rel) {
                    continue;
                }
                let Some(dep_sf) = path_lookup.get(&rel).cloned() else {
                    return Err(format!(
                        "bare_reference_closure: closure path '{rel}' (via bare name \
                         '{name}' → module '{module_path}') has no provenance in \
                         module index (fail-closed)"
                    ));
                };
                known_paths.insert(rel);
                known_paths.insert(dep_sf.path.clone());
                sources.push(dep_sf.clone());
                scan_queue.push(dep_sf);
            }
        }
    }
    Ok(sources)
}

/// The full name-derived closure for one entry: import edges + dotted-reference
/// modules + BARE-reference modules (via the tree census), iterated to a joint
/// fixpoint — a bare-pulled module can carry new dotted references and vice
/// versa. This is the loader the witness paths use; the raw-pair
/// `load_sources_for_entry_with_index` stays the dotted-only base for callers
/// without a pool index.
/// The ONE closure-extension authority: run the bare/service-name and the
/// module-path reference closures to a joint fixpoint (each newly-pulled module
/// can carry either edge kind). Both source loaders — the per-entry witness
/// loader and the affected-set compile-clean gate loader — call this, so the
/// single-authority claim in `extend_with_reference_closure`'s doc-comment is
/// true by construction rather than by two functions happening to hold identical
/// loop bodies (the §2 duplicate that dissolving the §3 fork would otherwise
/// have left behind).
fn extend_sources_to_both_closure_fixpoint(
    mut sources: Vec<Rc<v1_compiler_compile::SourceFile>>,
    mei: &MultiEntryIndex,
) -> Result<Vec<Rc<v1_compiler_compile::SourceFile>>, String> {
    loop {
        let before = sources.len();
        sources = extend_with_bare_reference_closure(sources, mei)?;
        sources =
            extend_with_reference_closure(sources, &mei.source_files, &mei.module_graph_facts)?;
        sources.sort_by(|a, b| a.path.cmp(&b.path));
        sources.dedup_by(|a, b| a.path == b.path);
        if sources.len() == before {
            break;
        }
    }
    Ok(sources)
}

fn load_sources_for_entry_with_pool(
    index: &MultiEntryIndex,
    entry_path: &str,
) -> Result<Vec<Rc<v1_compiler_compile::SourceFile>>, String> {
    let sources = load_sources_for_entry_with_index(
        &index.source_files,
        &index.module_graph_facts,
        entry_path,
    )?;
    extend_sources_to_both_closure_fixpoint(sources, index)
}

fn load_sources_for_entry_with_index(
    index: &ModuleSourceIndex,
    facts: &ModuleGraphFactsLive,
    entry_path: &str,
) -> Result<Vec<Rc<v1_compiler_compile::SourceFile>>, String> {
    let entry_source = entry_source_from_index_or_disk(index, entry_path)?;
    let rel_path = entry_source.path.clone();

    let sources = resolve_transitively(vec![entry_source.clone()], index, facts)?;
    let mut sources = sources;
    if !sources
        .iter()
        .any(|s| s.path == rel_path || same_canonical_file(&s.path, &rel_path))
    {
        sources.push(entry_source);
    }
    let mut sources = extend_with_reference_closure(sources, index, facts)?;
    sources.sort_by(|a, b| a.path.cmp(&b.path));
    sources.dedup_by(|a, b| a.path == b.path);
    Ok(sources)
}

fn same_canonical_file(a: &str, b: &str) -> bool {
    // A relative path here is WORKSPACE-relative (the module index stores paths
    // stripped of the workspace prefix by `build_module_path_index`); an entry
    // argument is typically already absolute. Resolve a relative path against
    // `workspace_root()` — its real base — NOT the process CWD: `cargo`/`nextest`
    // run test binaries with CWD = the crate dir, so canonicalizing a
    // workspace-relative path against CWD fails, the dedup silently misses, and the
    // absolutely-spelled entry is minted as a SECOND declaring file for one module
    // (the `duplicate module declaration` + self-circular collision on a temp-dir-
    // under-target fixture; blackout-masked from CI's nextest gate). Fall back to a
    // CWD-relative canonicalize so a genuinely CWD-relative spelling still matches.
    let canon = |p: &str| -> Option<std::path::PathBuf> {
        let path = Path::new(p);
        if path.is_absolute() {
            return std::fs::canonicalize(path).ok();
        }
        std::fs::canonicalize(workspace_root().join(path))
            .or_else(|_| std::fs::canonicalize(path))
            .ok()
    };
    match (canon(a), canon(b)) {
        (Some(ca), Some(cb)) => ca == cb,
        _ => false,
    }
}

fn entry_source_from_index_or_disk(
    index: &ModuleSourceIndex,
    entry_path: &str,
) -> Result<Rc<v1_compiler_compile::SourceFile>, String> {
    let path = std::path::Path::new(entry_path);
    if !path.is_file() {
        return Err(format!(
            "entry file does not exist or is not a file: {}",
            entry_path
        ));
    }
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("failed to read entry {:?}: {}", path, e))?;
    let rel_path = path.to_string_lossy().to_string();
    if let Some(mod_path) = extract_module_path(&content) {
        if let Some(cached) = index.get(&mod_path) {
            // Identity is the FILE, not the spelling: a relative entry against an
            // absolutely-rooted index (or vice versa) must unify with the indexed
            // source, not mint a second declaring file — that false fork trips the
            // module-identity collision wall as "duplicate module declaration"
            // (red on main since the S1 shared index; blackout-masked). Genuine
            // two-file duplicates still collide: different canonical paths.
            if cached.path == rel_path || same_canonical_file(&cached.path, &rel_path) {
                return Ok(cached.clone());
            }
        }
    }
    Ok(Rc::new(v1_compiler_compile::SourceFile {
        path: rel_path,
        content,
    }))
}

fn load_sources(
    source_roots: &[String],
) -> Result<Vec<Rc<v1_compiler_compile::SourceFile>>, String> {
    let mei = build_multi_entry_index(source_roots);
    load_compile_clean_entry_sources(source_roots, &mei, None)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClaimOutcome {
    Pass,
    Fail,
    NotBool { got: String },
    RuntimeError { message: String },
}

pub fn resolve_entry_graph(
    source_roots: &[String],
    entry_file: &str,
) -> Result<
    (
        Rc<v1_compiler_compile::ResolvedGraph>,
        Rc<HashMap<String, Rc<NewlineIndex>>>,
    ),
    String,
> {
    // Route through the same loader engine as `resolve_entry_graph_shared`
    // (proven behaviorally identical to the cold import-adjacency resolve by
    // resolve_typed_cache_equivalence_test): with imports stripped (namespace
    // wave 1) an entry's dependencies are name-derived, and the old
    // `load_sources_for_entry_with_index` walk only follows import edges — a
    // stripped fixed entry (e.g. the floor runner) failed to resolve at all.
    let index = process_shared_index(source_roots);
    resolve_entry_with_index(&index, entry_file)
}

// Process-level (per-thread) resolve store — the S1a increment of the resolver
// graph-major design (docs/plans/resolver-graph-major-design.md). Within one
// process the source tree is a fixed snapshot, so a resolved entry graph is a
// pure fact of (source_roots, entry) — the same purity assumption the walk memo
// (M1) and typed_module_cache already ship on. Routing every fixed-entry
// consumer (floor runner context, diff observer, output policy, group syntax,
// the executor's plan entry) through this store makes "resolve the same declared
// machinery twice in one process" unwritable on these paths, with failure
// semantics unchanged: a miss resolves exactly as before, including the typed
// error path. Thread-local by design: resolved graphs are Rc-based (not Send);
// shard threads keep their own store rather than smuggling Rc across threads.
thread_local! {
    #[allow(clippy::type_complexity)]
    static PROCESS_RESOLVE_STORE: RefCell<
        HashMap<
            (String, String),
            (
                Rc<v1_compiler_compile::ResolvedGraph>,
                Rc<HashMap<String, Rc<NewlineIndex>>>,
            ),
        >,
    > = RefCell::new(HashMap::new());

    // The thread's ONE shared resolve index (union-resolve S1,
    // docs/plans/resolver-graph-major-design.md §7). Every fixed-entry consumer routed
    // through resolve_entry_graph_shared (the executor prelude: plan entry + output
    // policy + group syntax, plus the floor runner) resolves against this single
    // MultiEntryIndex, so its parse/typed caches share the union of all those closures:
    // the shared std/spec prefix typechecks ONCE, not once per prelude entry. Keyed by
    // source_roots — a run's roots are fixed, so this is a get-or-build, rebuilt only on
    // the rare roots change. Thread-local by the same Rc-not-Send reason as the store:
    // each shard keeps its own index rather than smuggling Rc across threads.
    #[allow(clippy::type_complexity)]
    static PROCESS_RESOLVE_INDEX: RefCell<Option<(String, Rc<MultiEntryIndex>)>> =
        const { RefCell::new(None) };
}

/// Canonical spelling for the shared-index roots — both the key AND the build
/// inputs: an absolute root under the workspace normalizes to its repo-relative
/// form, so the executor's CLI `$ROOT/dag` and the plan's declared `dag`
/// (`gunbc.ci_layer_roots` witness_layer_roots) address ONE index. Without this,
/// the compile-clean receipt (armed from CLI roots) and batch-2 discovery (plan
/// roots) keyed two separate typed universes in CI and the gate's warm store was
/// silently replaced before the corpus read it (review 39118 on PR #6783). Order
/// is preserved (primary-precedence pool semantics); a root outside the workspace
/// keeps its spelling — it is genuinely a different pool.
fn canonical_shared_index_roots(source_roots: &[String]) -> Vec<String> {
    source_roots
        .iter()
        .map(|r| {
            let p = Path::new(r);
            if p.is_absolute() {
                try_repo_relative_path_normalized(p).unwrap_or_else(|| r.replace('\\', "/"))
            } else {
                r.replace('\\', "/")
            }
        })
        .collect()
}

/// The thread-local shared resolve index for `source_roots` (union-resolve S1). Built once
/// per (thread, canonical roots) and reused, so consumers that resolve distinct entries
/// against it share one typed_module_cache — the union closure typechecks once per node.
/// Roots are canonicalized (`canonical_shared_index_roots`) before both keying and
/// building, so path-spelling variants of the same pool cannot fork the universe.
fn process_shared_index(source_roots: &[String]) -> Rc<MultiEntryIndex> {
    let roots = canonical_shared_index_roots(source_roots);
    let roots_key = roots.join("\u{1f}");
    let existing = PROCESS_RESOLVE_INDEX.with(|s| {
        s.borrow().as_ref().and_then(|(k, idx)| {
            if *k == roots_key {
                Some(idx.clone())
            } else {
                None
            }
        })
    });
    if let Some(idx) = existing {
        return idx;
    }
    let idx = Rc::new(build_multi_entry_index(&roots));
    PROCESS_RESOLVE_INDEX.with(|s| {
        *s.borrow_mut() = Some((roots_key, idx.clone()));
    });
    idx
}

pub fn resolve_entry_graph_shared(
    source_roots: &[String],
    entry_file: &str,
) -> Result<
    (
        Rc<v1_compiler_compile::ResolvedGraph>,
        Rc<HashMap<String, Rc<NewlineIndex>>>,
    ),
    String,
> {
    let key = (source_roots.join("\u{1f}"), entry_file.to_string());
    let hit = PROCESS_RESOLVE_STORE.with(|s| s.borrow().get(&key).cloned());
    if let Some(found) = hit {
        return Ok(found);
    }
    // Resolve through the thread's shared index instead of a fresh per-call module index.
    // resolve_entry_with_index is proven behaviorally identical to the cold resolve_entry_graph
    // by resolve_typed_cache_equivalence_test (cached == cold across every resolve order); the
    // win is that the union of all fixed-entry closures now typechecks once per node.
    let index = process_shared_index(source_roots);
    let resolved = resolve_entry_with_index(&index, entry_file)?;
    PROCESS_RESOLVE_STORE.with(|s| {
        s.borrow_mut().insert(key, resolved.clone());
    });
    Ok(resolved)
}

pub struct MultiEntryIndex {
    source_files: ModuleSourceIndex,
    module_graph_facts: ModuleGraphFactsLive,
    /// Per-index typed results, keyed by the typed-module CONTENT key
    /// (`std.interface_summary.typed_module_key`: module source hash ⊕ direct-import
    /// interface hashes ⊕ compiler identity) — never by authored module name. The
    /// content key is the soundness license for eviction (PR-β) and the S2b-ready
    /// backend shape (cross-entry-typed-module-memo-sketch.md §1, operator-signed
    /// 2026-07-16); within one process it also makes a same-name/different-file
    /// collision structurally unable to serve the wrong typecheck (the name-keyed
    /// store relied on `module_source_identity` failing loud instead).
    typed_module_cache:
        RefCell<std::collections::HashMap<String, Rc<v1_compiler_infer::TypecheckModuleResult>>>,
    /// Source-content hashes by file path, recorded in the parse loop (where the
    /// `SourceFile.content` is in hand) — the source-hash key term for
    /// `typed_module_content_key`. A reconcile of a module whose file never passed
    /// the parse loop is a fail-closed error, never a silently keyless entry.
    source_hash_by_file: RefCell<std::collections::HashMap<String, String>>,
    /// Per-index collision registry when `cross_worker_store` is absent.
    module_source_identity: RefCell<std::collections::HashMap<String, String>>,
    /// Cross-worker serde-byte transport when increment C is explicitly armed (tests / future Arc).
    cross_worker_store: Option<Arc<RwLock<SharedTypecheckCaches>>>,
    /// Per-index intern table — paired with `parse_cache` on this worker (never shared).
    intern_table: RefCell<Rc<InternTable>>,
    parse_cache: RefCell<
        std::collections::HashMap<String, (Rc<v1_compiler_parse::ParseResult>, Rc<NewlineIndex>)>,
    >,
    normalize_diag_cache: RefCell<std::collections::HashMap<String, Rc<im::Vector<Rc<ErrorNode>>>>>,
    ownership_diag_cache: RefCell<std::collections::HashMap<String, Rc<im::Vector<Rc<ErrorNode>>>>>,
    /// The source roots this index was built from — the tree identities behind the
    /// per-tree bare census layers (a module's bare-name universe is its own tree).
    source_roots: Vec<String>,
    /// Parse-grade pool snapshot (every indexed module parsed once, with pool-wide
    /// newline indexes) — the shared input of the qualified fill and the per-tree
    /// bare layers below. Entry-independent, built once per process.
    pool_parse: RefCell<Option<Rc<PoolParse>>>,
    /// Whole-pool QUALIFIED-ONLY census layer (entries keyed by qualified name;
    /// empty global_bare/services), built once per process and underlaid beneath
    /// each entry's closure census (namespace-resolution-design.md §7.5: "fill =
    /// whole tree; policy gates lookup, never fill").
    pool_qualified_fill: RefCell<Option<Rc<SymbolIndex>>>,
    /// Per-source-root full census (bare + qualified + services) over that root's
    /// pool modules — the SAME-TREE bare layer underlaid beneath a module's closure
    /// census when it typechecks (bare = own tree; qualified = whole pool; cross-
    /// tree bare reach stays refused). Keyed by source root, built lazily.
    tree_bare_census: RefCell<std::collections::HashMap<String, Rc<SymbolIndex>>>,
    /// Whole-pool census (every pool module, both trees) — the LOADER's cross-
    /// tree fallback: a bare reference that misses the referencing file's own
    /// tree census resolves here so the provider still gets pulled into the
    /// closure (fill = whole tree). Same-tree resolution keeps priority — this
    /// is consulted only on an own-tree miss, so cross-tree homonyms cannot
    /// steal a same-tree name. Typecheck-side bare visibility is unchanged
    /// (closure census + own-tree underlay); the pulled provider becomes
    /// closure-visible, which is what serves the name at typecheck.
    pool_bare_census: RefCell<Option<Rc<SymbolIndex>>>,
    // Per-process subject-digest → resolved-graph share, the ReferenceTier in
    // front of the cross-process store (materialization-ladder tier ordering:
    // the share serves repeats, the store serves the process's FIRST touch of a
    // subject, and a store hit is INSTALLED here so every later demand takes the
    // reference). Always populated on first assembly — not gated on
    // GUNBC_RESOLVED_GRAPH_CACHE_DIR (the disk tier is opt-in separately).
    // Without the install-back, N same-subject resolves each retained an independent
    // graph — the reconcile_assembly per-entry rerun receipt (resolve-split #6535).
    resolved_graph_memo: RefCell<
        HashMap<
            String,
            (
                Rc<v1_compiler_compile::ResolvedGraph>,
                Rc<HashMap<String, Rc<NewlineIndex>>>,
            ),
        >,
    >,
}

pub fn new_shared_typecheck_caches() -> Arc<RwLock<SharedTypecheckCaches>> {
    shared_typecheck_store::new_shared_typecheck_caches()
}

pub fn build_multi_entry_index(source_roots: &[String]) -> MultiEntryIndex {
    new_multi_entry_index_shell(build_module_index(source_roots), source_roots, None)
}

/// Primary-precedence `MultiEntryIndex` — the index shape the compile-clean gate
/// uses (`--dependency-pool-index primary-precedence`: root[0] wins, later roots
/// fill only absent modules). Needed so `load_compile_clean_entry_sources` can run
/// the SAME both-closure fixpoint as the witness loader (`extend_with_bare_reference_closure`
/// requires the `MultiEntryIndex` for its per-tree bare census), dissolving the §3
/// closure-authority fork the two loaders' doc-comments each falsely claimed to be single.
fn build_multi_entry_index_primary_precedence(source_roots: &[String]) -> MultiEntryIndex {
    new_multi_entry_index_shell(
        build_module_index_primary_precedence(source_roots),
        source_roots,
        None,
    )
}

pub fn build_multi_entry_index_with_shared_caches(
    source_roots: &[String],
    cross_worker_store: Arc<RwLock<SharedTypecheckCaches>>,
) -> MultiEntryIndex {
    new_multi_entry_index_shell(
        build_module_index(source_roots),
        source_roots,
        Some(cross_worker_store),
    )
}

/// Test-only: whether `parse_cache` holds a path (pool census must not pre-fill it).
#[cfg(any(test, feature = "interp_test_witness"))]
pub fn parse_cache_contains_path_for_test(index: &MultiEntryIndex, path: &str) -> bool {
    index
        .parse_cache
        .borrow()
        .keys()
        .any(|k| k == path || same_canonical_file(k, path))
}

#[cfg(any(test, feature = "interp_test_witness"))]
pub fn parse_cache_paths_for_test(index: &MultiEntryIndex) -> Vec<String> {
    index.parse_cache.borrow().keys().cloned().collect()
}

fn new_multi_entry_index_shell(
    source_files: ModuleSourceIndex,
    source_roots: &[String],
    cross_worker_store: Option<Arc<RwLock<SharedTypecheckCaches>>>,
) -> MultiEntryIndex {
    MultiEntryIndex {
        source_files,
        module_graph_facts: build_module_graph_facts_live(source_roots),
        typed_module_cache: RefCell::new(std::collections::HashMap::new()),
        source_hash_by_file: RefCell::new(std::collections::HashMap::new()),
        module_source_identity: RefCell::new(std::collections::HashMap::new()),
        cross_worker_store,
        intern_table: RefCell::new(seed_kernel_intern_names(empty_intern_table())),
        parse_cache: RefCell::new(std::collections::HashMap::new()),
        normalize_diag_cache: RefCell::new(std::collections::HashMap::new()),
        ownership_diag_cache: RefCell::new(std::collections::HashMap::new()),
        resolved_graph_memo: RefCell::new(HashMap::new()),
        source_roots: source_roots.to_vec(),
        pool_parse: RefCell::new(None),
        pool_qualified_fill: RefCell::new(None),
        tree_bare_census: RefCell::new(std::collections::HashMap::new()),
        pool_bare_census: RefCell::new(None),
    }
}

/// Parse-grade pool snapshot: every indexed module's declaration heads plus the
/// pool-wide newline indexes, in deterministic (sorted module path) order.
/// Function bodies are stripped (shared marker only) — census consumers read
/// `module_items` / `local_binding_for_item`, never bodies.
struct PoolParse {
    /// Workspace-relative file path → census-head module node.
    nodes_by_file: Vec<(String, Rc<Node>)>,
    combined_si: Rc<HashMap<String, Rc<NewlineIndex>>>,
}

// Shared per-thread stand-in so stripped fn decls keep `body.is_some()` for
// `local_binding_for_item`'s fn discriminator. Loud-on-inference only:
// `ExprErrorKind::CensusHeadsBodyStripped` raises a hard diagnostic in `infer_expr`;
// it is NOT a complete guard against non-inference body-content reads (direct
// ExprData traversal, emit, node-count, etc.). `is_census_heads_fn_stand_in` and
// `census_heads_body_traversal_refusal` are dev-convenience query helpers, not the
// safety mechanism.
// 🟡 dissolve-on (B): `pool_nodes_by_file_consumers_must_not_descend_into_body` —
// standing test forbidding any `pool.nodes_by_file` consumer from non-inference body
// descent; lands the construction wall and retires `CensusHeadsBodyStripped` as a
// validation-only backstop.
const CENSUS_HEADS_FN_STAND_IN_NAME: &str = "^census_heads_fn_stand_in";

thread_local! {
    static STRIPPED_FN_BODY_MARKER: Rc<Node> = Rc::new(Node {
        name: CENSUS_HEADS_FN_STAND_IN_NAME.to_string(),
        span: no_span(),
        ident_span: None,
        children: empty_node_list(),
        connective: Connective::NoConnective,
        params: empty_node_list(),
        inferred: None,
        return_cardinality: Cardinality::Required,
        uses: empty_node_list(),
        body: None,
        transport: None,
        properties: empty_node_list(),
        type_annotation: None,
        is_self_recursive: false,
        has_non_tail_self_call: false,
        match_pattern: None,
        expr_data: Rc::new(ExprData::ExprError {
            kind: ExprErrorKind::CensusHeadsBodyStripped,
            message: "pool census heads-only: function body stripped — refuse to interpret"
                .to_string(),
        }),
        ident: None,
    });
}

fn stripped_fn_body_marker() -> Rc<Node> {
    STRIPPED_FN_BODY_MARKER.with(Rc::clone)
}

pub fn is_census_heads_fn_stand_in(node: &Rc<Node>) -> bool {
    node.name == CENSUS_HEADS_FN_STAND_IN_NAME
        || STRIPPED_FN_BODY_MARKER.with(|marker| Rc::ptr_eq(node, marker))
}

/// Optional query helper for non-inference traversals. Loud refusal on inference is
/// enforced by `ExprErrorKind::CensusHeadsBodyStripped` in `infer_expr`, not this API.
pub fn census_heads_body_traversal_refusal(node: &Rc<Node>) -> Option<String> {
    if is_census_heads_fn_stand_in(node) {
        Some(format!(
            "census heads-only pool parse refused: body traversal hit stand-in '{}'",
            CENSUS_HEADS_FN_STAND_IN_NAME
        ))
    } else {
        None
    }
}

#[cfg(any(test, feature = "interp_test_witness"))]
pub fn census_heads_fn_stand_in_for_test() -> Rc<Node> {
    stripped_fn_body_marker()
}

#[cfg(any(test, feature = "interp_test_witness"))]
pub fn census_heads_module_node_for_test(module: Rc<Node>) -> Rc<Node> {
    census_heads_module_node(module)
}

fn census_heads_children(children: &Rc<im::Vector<Rc<Node>>>) -> Rc<im::Vector<Rc<Node>>> {
    Rc::new(
        children
            .iter()
            .cloned()
            .map(census_heads_module_item)
            .collect(),
    )
}

/// Fn-decl discriminator for heads-only shrink — must match `local_binding_for_item`'s
/// fn arm (`04_infer.dag`: `NoConnective && body.is_some() && transport.is_none()`).
fn census_heads_item_is_fn_decl(item: &Rc<Node>) -> bool {
    item.connective == Connective::NoConnective && item.body.is_some() && item.transport.is_none()
}

fn census_heads_module_item(item: Rc<Node>) -> Rc<Node> {
    let body = if census_heads_item_is_fn_decl(&item) {
        Some(stripped_fn_body_marker())
    } else {
        None
    };
    let children = if item.children.is_empty() {
        item.children.clone()
    } else {
        census_heads_children(&item.children)
    };
    Rc::new(Node {
        name: item.name.clone(),
        span: item.span.clone(),
        ident_span: item.ident_span.clone(),
        children,
        connective: item.connective.clone(),
        params: item.params.clone(),
        inferred: item.inferred.clone(),
        return_cardinality: item.return_cardinality.clone(),
        uses: empty_node_list(),
        body,
        transport: item.transport.clone(),
        properties: item.properties.clone(),
        type_annotation: item.type_annotation.clone(),
        is_self_recursive: item.is_self_recursive,
        has_non_tail_self_call: item.has_non_tail_self_call,
        match_pattern: None,
        expr_data: Rc::new(ExprData::NoExprData),
        ident: item.ident.clone(),
    })
}

fn census_heads_module_node(module: Rc<Node>) -> Rc<Node> {
    Rc::new(Node {
        name: module.name.clone(),
        span: module.span.clone(),
        ident_span: module.ident_span.clone(),
        children: Rc::new(
            module_items(module.clone())
                .iter()
                .cloned()
                .map(census_heads_module_item)
                .collect(),
        ),
        connective: module.connective.clone(),
        params: module.params.clone(),
        inferred: module.inferred.clone(),
        return_cardinality: module.return_cardinality.clone(),
        uses: empty_node_list(),
        body: None,
        transport: module.transport.clone(),
        properties: module.properties.clone(),
        type_annotation: module.type_annotation.clone(),
        is_self_recursive: module.is_self_recursive,
        has_non_tail_self_call: module.has_non_tail_self_call,
        match_pattern: None,
        expr_data: Rc::new(ExprData::NoExprData),
        ident: module.ident.clone(),
    })
}

// Once-per-node resolve receipt (union-resolve minimum-upper-bound contract, §6.2 of
// many module typechecks were actually COMPUTED (cache misses). When every consumer of
// the process shares one typed_module_cache, this stays ≤ the distinct module count of
// the process union.
static TYPECHECK_COMPUTE_COUNT: AtomicUsize = AtomicUsize::new(0);

// Union-resolve receipt tests reset/read the process-wide counter; `cargo test` runs
// `#[test]` fns in parallel by default — serialize those oracles (not production use).
static TYPECHECK_COMPUTE_COUNT_RECEIPT_LOCK: Mutex<()> = Mutex::new(());

/// Run a counter-based receipt test with exclusive access to `TYPECHECK_COMPUTE_COUNT`.
pub fn with_typecheck_compute_count_receipt<R>(f: impl FnOnce() -> R) -> R {
    let _guard = TYPECHECK_COMPUTE_COUNT_RECEIPT_LOCK
        .lock()
        .expect("typecheck_compute_count receipt lock poisoned");
    f()
}

pub fn typecheck_compute_count() -> usize {
    TYPECHECK_COMPUTE_COUNT.load(Ordering::SeqCst)
}

pub fn reset_typecheck_compute_count() {
    TYPECHECK_COMPUTE_COUNT.store(0, Ordering::SeqCst);
}

fn bump_typecheck_compute_count() {
    TYPECHECK_COMPUTE_COUNT.fetch_add(1, Ordering::SeqCst);
}

fn shared_caches_read<'a>(
    lock: &'a Arc<RwLock<SharedTypecheckCaches>>,
) -> Result<std::sync::RwLockReadGuard<'a, SharedTypecheckCaches>, String> {
    lock.read()
        .map_err(|e| format!("shared typecheck caches lock poisoned: {e}"))
}

fn shared_caches_write<'a>(
    lock: &'a Arc<RwLock<SharedTypecheckCaches>>,
) -> Result<std::sync::RwLockWriteGuard<'a, SharedTypecheckCaches>, String> {
    lock.write()
        .map_err(|e| format!("shared typecheck caches lock poisoned: {e}"))
}

/// The typed-module content key for `resolved` — the Rust realization of
/// `std.interface_summary.typed_module_key` over the live store's inputs
/// (cross-entry-typed-module-memo-sketch.md §1, operator-signed 2026-07-16):
///
///   key = typed_module_key(module_key(source_hash, direct-import interface hashes),
///                          compiler identity)
///
/// - `source_hash` was recorded by `note_source_hash` in the parse loop; a module whose
///   file never passed that loop REFUSES (fail-closed) — the key never silently drops a
///   term.
/// - Direct-import interface hashes come from `interface_hash_by_name`, filled in
///   dependency order as import results are obtained (hit or computed); a missing import
///   hash likewise refuses — it would mean the schedule dispatched a dependent before
///   its import. The interface hash is the Inc-B `ModuleInterface.summary.interface_hash`
///   (v0 fingerprint; its declared-weak grain and upgrade trigger live at
///   `src/v1/04_infer.dag` `interface_signature_fingerprint_v0_note`).
/// - Compiler identity is `resolved_graph_cache::transform_content_digest` — the same
///   single authority the resolved-graph subject digest consumes (§3: one concept, one
///   authority; a seed rebuild invalidates both stores through one term).
fn typed_module_content_key(
    index: &MultiEntryIndex,
    resolved: &Rc<v1_compiler_resolve::ResolvedModule>,
    mod_name: &str,
    interface_hash_by_name: &std::collections::HashMap<String, String>,
) -> Result<String, String> {
    let file = &resolved.module.span.file;
    let source_hash = index
        .source_hash_by_file
        .borrow()
        .get(file)
        .cloned()
        .ok_or_else(|| {
            format!(
                "typed-module content key refused: no source hash recorded for '{file}' \
                 (module '{mod_name}') — every reconciled module must pass the parse loop \
                 in this process before its typed result is keyed"
            )
        })?;
    let mut import_hashes: im::Vector<String> = im::Vector::new();
    for import in resolved.resolved_imports.iter() {
        let hash = interface_hash_by_name
            .get(&import.module_path)
            .cloned()
            .ok_or_else(|| {
                format!(
                    "typed-module content key refused: direct import '{}' of module \
                     '{mod_name}' has no interface hash yet — imports must be typechecked \
                     (or cache-served) before their dependents are keyed",
                    import.module_path
                )
            })?;
        import_hashes.push_back(hash);
    }
    Ok(typed_module_key(
        module_key(source_hash, Rc::new(import_hashes)),
        transform_content_digest(),
    ))
}

/// Record `mod_name`'s interface hash for downstream key derivation (one entry per
/// module per reconcile; the interface is a pure projection of the typed result).
fn note_interface_hash(
    interface_hash_by_name: &mut std::collections::HashMap<String, String>,
    mod_name: &str,
    tc_result: &Rc<v1_compiler_infer::TypecheckModuleResult>,
) {
    interface_hash_by_name.insert(
        mod_name.to_string(),
        tc_result.typed.interface.summary.interface_hash.clone(),
    );
}

/// Read the typed cache: per-index `Rc` when private; shared byte snapshots only when
/// cross-worker store is armed (no local duplicate — serde transport is one authority).
/// `typed_key` is the content key from `typed_module_content_key`, never a module name.
fn index_get_typed(
    index: &MultiEntryIndex,
    typed_key: &str,
) -> Result<Option<Rc<v1_compiler_infer::TypecheckModuleResult>>, String> {
    let Some(store) = index.cross_worker_store.as_ref() else {
        return Ok(index.typed_module_cache.borrow().get(typed_key).cloned());
    };
    shared_get_typed(store, typed_key)
}

fn check_index_module_source_identity(
    index: &MultiEntryIndex,
    mod_name: &str,
    decl_file: &str,
) -> Result<(), String> {
    if let Some(store) = &index.cross_worker_store {
        let mut caches = shared_caches_write(store)?;
        check_module_source_identity_map(&mut caches.module_source_identity, mod_name, decl_file)
    } else {
        check_module_source_identity_map(
            &mut index.module_source_identity.borrow_mut(),
            mod_name,
            decl_file,
        )
    }
}

fn index_insert_typed(
    index: &MultiEntryIndex,
    typed_key: String,
    result: Rc<v1_compiler_infer::TypecheckModuleResult>,
) -> Result<Rc<v1_compiler_infer::TypecheckModuleResult>, String> {
    let Some(store) = index.cross_worker_store.as_ref() else {
        index
            .typed_module_cache
            .borrow_mut()
            .insert(typed_key, result.clone());
        return Ok(result);
    };
    if let Some(bytes) = {
        let caches = shared_caches_read(store)?;
        caches.clone_typed_bytes(&typed_key)
    } {
        return SharedTypecheckCaches::decode_typed_snapshot(bytes.as_slice());
    }
    let encoded = SharedTypecheckCaches::encode_typed_snapshot(&result)?;
    let raced_bytes = {
        let mut caches = shared_caches_write(store)?;
        if let Some(existing) = caches.clone_typed_bytes(&typed_key) {
            Some(existing)
        } else {
            caches.insert_typed_preencoded(typed_key.clone(), encoded);
            None
        }
    };
    if let Some(bytes) = raced_bytes {
        return SharedTypecheckCaches::decode_typed_snapshot(bytes.as_slice());
    }
    // Insert won the race: bytes live in the shared store only (no per-index Rc copy).
    Ok(result)
}

/// Read the shared typed cache with a brief lock hold; decode happens after the guard drops.
fn shared_get_typed(
    shared_caches: &Arc<RwLock<SharedTypecheckCaches>>,
    typed_key: &str,
) -> Result<Option<Rc<v1_compiler_infer::TypecheckModuleResult>>, String> {
    let bytes = {
        let caches = shared_caches_read(shared_caches)?;
        caches.clone_typed_bytes(typed_key)
    };
    match bytes {
        Some(snapshot) => {
            SharedTypecheckCaches::decode_typed_snapshot(snapshot.as_slice()).map(Some)
        }
        None => Ok(None),
    }
}

/// Accumulate the authored module names of a set of typed modules into `out`.
///
/// This is the closure-size primitive: `|out|` after folding every graph a shard resolved is the
/// distinct-module count of that shard's union closure. It reads the resolved graph, so it is a
/// fact about the source snapshot — unlike `typecheck_compute_count()`, which reads a cumulative
/// per-thread miss counter and therefore reports a closure size only when the thread started cold.
fn collect_typed_module_names(
    modules: impl IntoIterator<Item = Rc<TypedModule>>,
    source_indices: &Rc<HashMap<String, Rc<NewlineIndex>>>,
    out: &mut HashSet<String>,
) {
    for m in modules {
        out.insert(authored_name_at(source_indices.clone(), m.module.clone()));
    }
}

fn seed_kernel_intern_names(table: Rc<InternTable>) -> Rc<InternTable> {
    let mut t = table;
    for name in v1_rt::map_keys(&kernel_type_set()).iter().cloned() {
        t = intern(t, name).table.clone();
    }
    for name in ["Optional", "Present", "Absent", "value", "none"] {
        t = intern(t, name.to_string()).table.clone();
    }
    for name in v1_rt::map_keys(&compiler_recursive_types()).iter().cloned() {
        t = intern(t, name).table.clone();
    }
    t
}

/// Primary-precedence pool for affected-set attribution of `src/v1/*.dag` edits:
/// witness_layer_roots (dag + src/v2) cannot resolve v1.compiler.* modules alone.
fn build_v1_attribution_multi_entry_index() -> MultiEntryIndex {
    let roots = vec![
        "dag".to_string(),
        "src/v2".to_string(),
        "src/v1".to_string(),
    ];
    new_multi_entry_index_shell(build_module_index_primary_precedence(&roots), &roots, None)
}

pub fn resolve_entry_with_index(
    index: &MultiEntryIndex,
    entry_file: &str,
) -> Result<
    (
        Rc<v1_compiler_compile::ResolvedGraph>,
        Rc<HashMap<String, Rc<NewlineIndex>>>,
    ),
    String,
> {
    resolve_entry_with_parse_cache(index, entry_file, ResolveTypecheckGate::Strict)
}

pub fn resolve_entry_with_index_for_discovery_corpus(
    index: &MultiEntryIndex,
    entry_file: &str,
) -> Result<
    (
        Rc<v1_compiler_compile::ResolvedGraph>,
        Rc<HashMap<String, Rc<NewlineIndex>>>,
    ),
    String,
> {
    resolve_entry_with_parse_cache(
        index,
        entry_file,
        ResolveTypecheckGate::DiscoveryCorpusAdvisory,
    )
}

fn resolve_entry_graph_with_index(
    index: &ModuleSourceIndex,
    facts: &ModuleGraphFactsLive,
    entry_file: &str,
) -> Result<
    (
        Rc<v1_compiler_compile::ResolvedGraph>,
        Rc<HashMap<String, Rc<NewlineIndex>>>,
    ),
    String,
> {
    set_phase(FloorPhase::Resolve, entry_file);
    let sources = load_sources_for_entry_with_index(index, facts, entry_file)?;
    set_phase(FloorPhase::Typecheck, entry_file);
    resolved_graph_from_sources(sources, ResolveTypecheckGate::Strict)
}

/// Per-entry stage attribution for the one-lump `resolve_nanos` (run-stability
/// throughline, docs/plans/v1-run-stability-throughline.md): per-entry resolve cost
/// was a single undifferentiated number, so which stage to memoize at module grain
/// could not be chosen by receipt. Filled per entry by `resolve_entry_with_parse_cache`
/// (reconcile internals accumulate their own rows) through a worker-local slot — each
/// floor worker resolves its entries sequentially on its own thread, so a thread-local
/// is per-worker-correct at width > 1, unlike the process-global last-writer-wins
/// `phase_profile` sampler, which is explicitly not attribution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ResolveStageNanos {
    /// `load_sources_for_entry_with_index` inside the timed window (closure walk over the index).
    pub load: u128,
    /// Parse-cache assembly loop: tokenize+parse on miss, cache clone on hit.
    pub parse: u128,
    /// `resolve_modules` + its blocking-diagnostic scan (per-module import resolution, closure topo/dup).
    pub resolve: u128,
    /// `normalize_graph` + its blocking-diagnostic scan (pure per-module diagnostics).
    pub normalize: u128,
    /// Genuine `typecheck_module` computes inside reconcile (typed-cache misses only).
    pub typecheck_compute: u128,
    /// `collect_parent_envs` calls inside reconcile (every module, cache hit or miss).
    pub parent_envs: u128,
    /// Reconcile total minus the two rows above: variant surfaces, registry merge,
    /// transitive-service expansion, the three rewire passes, emit-graph info — the
    /// whole-closure assembly residue that reruns per entry even at 100% cache hits.
    pub reconcile_assembly: u128,
    /// `extract_ownership_proofs` + its diagnostics walk.
    pub ownership: u128,
}

impl ResolveStageNanos {
    pub fn accumulate(&mut self, other: &ResolveStageNanos) {
        self.load += other.load;
        self.parse += other.parse;
        self.resolve += other.resolve;
        self.normalize += other.normalize;
        self.typecheck_compute += other.typecheck_compute;
        self.parent_envs += other.parent_envs;
        self.reconcile_assembly += other.reconcile_assembly;
        self.ownership += other.ownership;
    }

    /// Sum of the attributed stages; the caller's lump minus this is the
    /// unattributed residue (early cache-hit returns, diagnostic formatting).
    pub fn attributed_total(&self) -> u128 {
        self.load
            + self.parse
            + self.resolve
            + self.normalize
            + self.typecheck_compute
            + self.parent_envs
            + self.reconcile_assembly
            + self.ownership
    }
}

thread_local! {
    static RESOLVE_STAGE_SLOT: std::cell::Cell<ResolveStageNanos> =
        const { std::cell::Cell::new(ResolveStageNanos {
            load: 0,
            parse: 0,
            resolve: 0,
            normalize: 0,
            typecheck_compute: 0,
            parent_envs: 0,
            reconcile_assembly: 0,
            ownership: 0,
        }) };
}

fn resolve_stage_slot_reset() {
    RESOLVE_STAGE_SLOT.with(|s| s.set(ResolveStageNanos::default()));
}

fn resolve_stage_slot_add(update: impl FnOnce(&mut ResolveStageNanos)) {
    RESOLVE_STAGE_SLOT.with(|s| {
        let mut v = s.get();
        update(&mut v);
        s.set(v);
    });
}

fn resolve_stage_slot_snapshot() -> ResolveStageNanos {
    RESOLVE_STAGE_SLOT.with(|s| s.get())
}

fn resolve_entry_with_parse_cache(
    index: &MultiEntryIndex,
    entry_file: &str,
    typecheck_gate: ResolveTypecheckGate,
) -> Result<
    (
        Rc<v1_compiler_compile::ResolvedGraph>,
        Rc<HashMap<String, Rc<NewlineIndex>>>,
    ),
    String,
> {
    resolve_stage_slot_reset();
    set_phase(FloorPhase::Resolve, entry_file);
    let load_started = std::time::Instant::now();
    let sources = load_sources_for_entry_with_pool(index, entry_file)?;
    resolve_stage_slot_add(|s| s.load += load_started.elapsed().as_nanos());
    resolved_graph_from_sources_with_index(index, sources, typecheck_gate, entry_file)
}

/// The sources-taking core of `resolve_entry_with_parse_cache`: parse → resolve →
/// normalize → `reconcile_with_typed_cache` → ownership, every stage through the
/// index's per-module memo tiers (parse/normalize/typed/ownership caches + the
/// resolved-graph subject memo). Extracted so a whole-tree SOURCE SET — the
/// compile-clean gate's closure, which has no single entry file — rides the same
/// cached path as entry-file resolves: one process, ONE typecheck universe, so the
/// floor's gate compile and batch-2's witness resolves share every module's
/// content-keyed typecheck instead of double-paying it (typecheck investigation,
/// PR #6766).
///
/// Failure semantics: collect-then-refuse per stage — a stage gathers ALL of its
/// diagnostics before refusing (parse errors across every file, resolve/normalize/
/// typecheck/ownership across every module), so a multi-error tree reports its full
/// failing-stage set in one run, never one error per run. Hardness predicates:
/// typecheck refusals use `is_resolve_typecheck_blocking(typecheck_gate)` and the
/// other stages use `is_error_diagnostic` — for the gate's `Strict` mode both reduce
/// to the `00_core.dag` interpreter-blocking authority on every class those stages
/// can produce (`ComplexityUnknown`, the sole class where the predicates differ, is
/// only produced by complexity analysis, which does not run on this path).
fn resolved_graph_from_sources_with_index(
    index: &MultiEntryIndex,
    sources: Vec<Rc<v1_compiler_compile::SourceFile>>,
    typecheck_gate: ResolveTypecheckGate,
    phase_label: &str,
) -> Result<
    (
        Rc<v1_compiler_compile::ResolvedGraph>,
        Rc<HashMap<String, Rc<NewlineIndex>>>,
    ),
    String,
> {
    let entry_file = phase_label;
    let subject = subject_digest_for_closure(&sources);
    // In-process share tier (resolved_graph_memo): always on — the ReferenceTier in
    // front of the opt-in cross-process store. A subject this process has already
    // assembled is served by reference, eliminating the per-entry reconcile assembly
    // residue on re-resolve (Track A denomination receipt, resolve-split #6535).
    if let Some((graph, si)) = index.resolved_graph_memo.borrow().get(&subject) {
        return Ok((graph.clone(), si.clone()));
    }
    // Cross-process store tier: opt-in via GUNBC_RESOLVED_GRAPH_CACHE_DIR; installs into
    // the share above on hit so later same-subject demands never re-decode.
    if let Some(cache_root) = resolved_graph_cache_root_from_env() {
        match cross_process_lookup(&cache_root, &subject) {
            CacheLookupResult::Hit(hit) => {
                eprintln!(
                    "[resolved-graph-cache] decode subject={subject} (installed into process share)"
                );
                index
                    .resolved_graph_memo
                    .borrow_mut()
                    .insert(subject, (hit.graph.clone(), hit.source_indices.clone()));
                return Ok((hit.graph, hit.source_indices));
            }
            CacheLookupResult::RejectedHit(_) | CacheLookupResult::Miss => {}
        }
    }

    let mut modules: Vec<Rc<Node>> = Vec::new();
    let mut si_map: HashMap<String, Rc<NewlineIndex>> = HashMap::new();
    let mut parse_error_msgs: Vec<String> = Vec::new();

    let parse_started = std::time::Instant::now();
    for source in &sources {
        note_source_hash(index, source);
        let cached = index.parse_cache.borrow().get(&source.path).cloned();

        let (parse_result, nl_index) = match cached {
            Some(entry) => entry,
            None => {
                let tokens =
                    v1_compiler_tokenize::tokenize(source.content.clone(), source.path.clone());
                let nl_index = build_newline_index(source.path.clone(), source.content.clone());
                let current_table = index.intern_table.borrow().clone();
                let single_si: Rc<HashMap<String, Rc<NewlineIndex>>> = Rc::new({
                    let mut m = HashMap::new();
                    m.insert(source.path.clone(), nl_index.clone());
                    m
                });
                let parsed = v1_compiler_parse::parse_with_table(tokens, single_si, current_table);
                *index.intern_table.borrow_mut() = parsed.intern_table.clone();
                let entry = (parsed.result.clone(), nl_index.clone());
                index
                    .parse_cache
                    .borrow_mut()
                    .insert(source.path.clone(), entry.clone());
                entry
            }
        };

        si_map.insert(nl_index.file.clone(), nl_index.clone());
        if let Some(err) = &parse_result.error {
            // Collect-then-refuse: gather every file's parse error before refusing,
            // so a multi-file parse red reports its full set in one run.
            let span = diagnostic_to_span(err.diagnostic.clone());
            let loc = format_error_loc(&span.file, span.start, &si_map);
            parse_error_msgs.push(format!(
                "{}: error: {}",
                loc,
                diagnostic_to_message(err.diagnostic.clone())
            ));
            continue;
        }
        if let Some(module) = &parse_result.module {
            modules.push(module.clone());
        }
    }
    if !parse_error_msgs.is_empty() {
        return Err(parse_error_msgs.join("\n"));
    }

    let source_indices = Rc::new(si_map);
    let global_table = index.intern_table.borrow().clone();
    resolve_stage_slot_add(|s| s.parse += parse_started.elapsed().as_nanos());

    let resolve_started = std::time::Instant::now();
    let graph =
        v1_compiler_resolve::resolve_modules(Rc::new(modules.into()), source_indices.clone());

    if graph
        .diagnostics
        .iter()
        .any(|d| is_error_diagnostic(d.diagnostic.clone()))
    {
        return Err(format_error_nodes(&graph.diagnostics, &source_indices));
    }
    resolve_stage_slot_add(|s| s.resolve += resolve_started.elapsed().as_nanos());

    let normalize_started = std::time::Instant::now();
    // Per-module memo (normalize_diag_cache): normalize is diagnostics-only — the
    // authority passes the graph through unchanged (v1.compiler.normalize
    // `NormalizeResult { graph: graph, .. }`) — and its per-module row
    // `normalize_module_diagnostics` is a pure function of the parsed module node,
    // so an entry pays only for modules this process has not normalized before
    // (resolve-split receipt: normalize was 8% of whole-corpus resolve, recomputed
    // per entry at zero marginal information).
    let mut norm_diag_vec: im::Vector<Rc<ErrorNode>> = im::Vector::new();
    for m in graph.modules.iter() {
        let key = m.module.span.file.clone();
        let cached = index.normalize_diag_cache.borrow().get(&key).cloned();
        let module_diags = match cached {
            Some(hit) => hit,
            None => {
                let computed = v1_compiler_normalize::normalize_module_diagnostics(
                    m.clone(),
                    source_indices.clone(),
                );
                index
                    .normalize_diag_cache
                    .borrow_mut()
                    .insert(key, computed.clone());
                computed
            }
        };
        norm_diag_vec.extend(module_diags.iter().cloned());
    }
    let norm_diags = Rc::new(norm_diag_vec);

    if norm_diags
        .iter()
        .any(|d| is_error_diagnostic(d.diagnostic.clone()))
    {
        return Err(format_error_nodes(&norm_diags, &source_indices));
    }
    resolve_stage_slot_add(|s| s.normalize += normalize_started.elapsed().as_nanos());

    set_phase(FloorPhase::Typecheck, entry_file);
    let reconcile_started = std::time::Instant::now();
    let typed =
        reconcile_with_typed_cache(graph.clone(), source_indices.clone(), global_table, index)?;
    // Assembly residue = reconcile wall minus the per-module rows its internals
    // accumulated into the slot during this call (typecheck computes + parent envs).
    let reconcile_total = reconcile_started.elapsed().as_nanos();
    resolve_stage_slot_add(|s| {
        s.reconcile_assembly += reconcile_total.saturating_sub(s.typecheck_compute + s.parent_envs);
    });

    for d in typed.diagnostics.iter() {
        log_discovery_advisory_typecheck(d, &source_indices, typecheck_gate);
    }
    let has_type_errors = typed
        .diagnostics
        .iter()
        .any(|d| is_resolve_typecheck_blocking(d.diagnostic.clone(), typecheck_gate));
    if has_type_errors {
        let msgs: Vec<String> = typed
            .diagnostics
            .iter()
            .filter(|d| is_resolve_typecheck_blocking(d.diagnostic.clone(), typecheck_gate))
            .map(|d| format_error_node(d, &source_indices))
            .collect();
        return Err(msgs.join("\n"));
    }

    let ownership_started = std::time::Instant::now();
    // Per-module memo (ownership_diag_cache): ownership proofs are a pure per-module
    // map (v1.compiler.compile `module_ownership_proofs`; the authority's graph fold
    // is exactly this row flat_mapped in module order) and `ownership_diagnostics`
    // distributes over per-module concatenation, so the diagnostic list assembled in
    // `typed.modules` order is identical to the graph-grain computation — a module
    // with no bodied items contributes the same empty row the authority's filter
    // skips. First-touch per module; the per-entry graph-grain rerun (7% of
    // whole-corpus resolve in the resolve-split receipt) collapses to cache reads.
    let mut ownership_diag_vec: im::Vector<Rc<ErrorNode>> = im::Vector::new();
    for m in typed.modules.iter() {
        let key = m.module.span.file.clone();
        let cached = index.ownership_diag_cache.borrow().get(&key).cloned();
        let module_diags = match cached {
            Some(hit) => hit,
            None => {
                let proofs = v1_compiler_compile::module_ownership_proofs(m.clone());
                let computed = v1_compiler_compile::ownership_diagnostics(proofs);
                index
                    .ownership_diag_cache
                    .borrow_mut()
                    .insert(key, computed.clone());
                computed
            }
        };
        ownership_diag_vec.extend(module_diags.iter().cloned());
    }
    let ownership_diags = Rc::new(ownership_diag_vec);
    if ownership_diags
        .iter()
        .any(|d| is_error_diagnostic(d.diagnostic.clone()))
    {
        return Err(format_error_nodes(&ownership_diags, &source_indices));
    }
    resolve_stage_slot_add(|s| s.ownership += ownership_started.elapsed().as_nanos());

    // Install into the in-process share so same-subject re-resolves skip assembly.
    index
        .resolved_graph_memo
        .borrow_mut()
        .insert(subject.clone(), (typed.clone(), source_indices.clone()));
    if let Some(cache_root) = resolved_graph_cache_root_from_env() {
        // A failed store write is a disclosed refusal, never a silent shrug —
        // the swallowed error hid that big closures never landed on disk (only
        // the prelude artifact ever existed), which mis-shaped a whole OOM
        // investigation (receipt: eager-ram-612 bisect, 2026-07-10).
        if let Err(e) = cross_process_write(&cache_root, &subject, &typed, source_indices.as_ref())
        {
            eprintln!("[resolved-graph-cache] write refused subject={subject}: {e}");
        }
    }

    Ok((typed, source_indices))
}

/// Collision-honesty check for the shared typed-module cache (union-resolve receipt §6.3,
/// docs/plans/resolver-graph-major-design.md). The typed cache is keyed by authored module
/// name and reused across every entry that co-resides in one process's shared index, so a
/// name that maps to two DIFFERENT declaring files is a co-residence surprise: serving one
/// file's typecheck for the other's would be a §5 fail-open (a divergent resolution passing
/// as plausible). This fails loud instead. Re-seeing the SAME (name, file) — one module
/// reached through many import paths — is benign and records nothing new; first sight records
/// the identity, a later mismatch is a typed error. `build_module_index` already walls
/// tree-wide module-path collisions at index build; this is the same wall at the cache seam
/// the union widens (e.g. an on-disk entry whose module path shadows an indexed module,
/// reached via `entry_source_from_index_or_disk`).
fn check_module_source_identity_map(
    registry: &mut std::collections::HashMap<String, String>,
    mod_name: &str,
    decl_file: &str,
) -> Result<(), String> {
    if let Some(prev_file) = registry.get(mod_name).cloned() {
        if prev_file != decl_file {
            return Err(format!(
                "co-residence collision: module '{mod_name}' resolved from two files \
                 ('{prev_file}' and '{decl_file}') in one process — one module, one authority \
                 (DESIGN §3). The shared resolve store fails loud rather than silently serving \
                 a divergent module (resolver-graph-major-design.md §6.3)."
            ));
        }
        return Ok(());
    }
    registry.insert(mod_name.to_string(), decl_file.to_string());
    Ok(())
}

/// Antichain batches (Kahn levels) over the closure's resolved import edges — the host
/// realization of the modeled module-node schedule (resolver-graph-major-design.md §7 S2a
/// move 2: module nodes ride the same scheduler/runner shape as the CI floor,
/// `v2.workflow.module_resolution_plan` is the model authority). Nodes are the closure's
/// modules at authored-name grain (the typed-store key); edges are `resolved_imports` rows
/// restricted to the closure (a dangling import is not a schedule edge — the missing-parent
/// diagnostic stays typecheck's own, unchanged). A cyclic residue is unschedulable by a
/// forward walk; it is appended as a final batch in the resolver's original order — never
/// silently dropped — so its missing-parent diagnostics are the same set the serial fold
/// produced (original order is a DFS postorder, so acyclic imports always precede their
/// importers; only within-cycle edges can be "missing", identically in both walks).
/// Batches are deterministic: within a level, modules keep their original relative order.
fn module_schedule_batches(
    modules: &[Rc<v1_compiler_resolve::ResolvedModule>],
    module_names: &[String],
) -> Vec<Vec<usize>> {
    let position: HashMap<&str, usize> = module_names
        .iter()
        .enumerate()
        .map(|(i, name)| (name.as_str(), i))
        .collect();
    let edges: Vec<(usize, usize)> = modules
        .iter()
        .enumerate()
        .flat_map(|(i, resolved)| {
            let position = &position;
            resolved
                .resolved_imports
                .iter()
                .filter_map(move |imp| position.get(imp.module_path.as_str()).map(|&src| (src, i)))
                .collect::<Vec<_>>()
        })
        .collect();
    schedule_batches_from_edges(modules.len(), &edges)
}

/// The pure batching core of `module_schedule_batches`: nodes are `0..n` in dependency-view
/// order, `edges` are `(source, dependent)` pairs (duplicates and self-edges tolerated —
/// deduped and skipped respectively, matching repeated `import` rows of one module).
fn schedule_batches_from_edges(n: usize, edges: &[(usize, usize)]) -> Vec<Vec<usize>> {
    let mut indegree: Vec<usize> = vec![0; n];
    let mut dependents: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut seen_edges: HashSet<(usize, usize)> = HashSet::new();
    for &(src, dependent) in edges {
        if src != dependent && seen_edges.insert((src, dependent)) {
            dependents[src].push(dependent);
            indegree[dependent] += 1;
        }
    }
    let mut batches: Vec<Vec<usize>> = Vec::new();
    let mut scheduled = vec![false; n];
    let mut frontier: Vec<usize> = (0..n).filter(|&i| indegree[i] == 0).collect();
    let mut scheduled_count = 0;
    while !frontier.is_empty() {
        for &i in &frontier {
            scheduled[i] = true;
        }
        scheduled_count += frontier.len();
        let mut next: Vec<usize> = Vec::new();
        for &i in &frontier {
            for &dep in &dependents[i] {
                indegree[dep] -= 1;
                if indegree[dep] == 0 {
                    next.push(dep);
                }
            }
        }
        next.sort_unstable();
        batches.push(std::mem::replace(&mut frontier, next));
    }
    if scheduled_count < n {
        batches.push((0..n).filter(|&i| !scheduled[i]).collect());
    }
    batches
}

#[cfg(test)]
mod module_schedule_batches_tests {
    use super::schedule_batches_from_edges;

    fn flat(batches: &[Vec<usize>]) -> Vec<usize> {
        batches.iter().flatten().copied().collect()
    }

    // Once-per-node by construction (§5): every node appears in the schedule exactly once,
    // and a dependent is never batched before its source.
    #[test]
    fn diamond_schedules_antichain_levels() {
        // 0 -> {1, 2} -> 3 (diamond): 1 and 2 are the same level (the parallel frontier).
        let batches = schedule_batches_from_edges(4, &[(0, 1), (0, 2), (1, 3), (2, 3)]);
        assert_eq!(batches, vec![vec![0], vec![1, 2], vec![3]]);
    }

    #[test]
    fn chain_is_one_node_per_batch_and_covers_every_node_once() {
        let batches = schedule_batches_from_edges(3, &[(0, 1), (1, 2)]);
        assert_eq!(batches, vec![vec![0], vec![1], vec![2]]);
        assert_eq!(flat(&batches), vec![0, 1, 2]);
    }

    #[test]
    fn duplicate_and_self_edges_do_not_deadlock() {
        // A module importing the same module twice (two import rows) and a self-import
        // must not inflate indegree — both were representable in resolved_imports.
        let batches = schedule_batches_from_edges(2, &[(0, 1), (0, 1), (1, 1)]);
        assert_eq!(batches, vec![vec![0], vec![1]]);
    }

    // RED-control shape: a cycle is unschedulable by the forward walk; the residue is
    // appended as a final batch in original order — never silently dropped (coverage
    // stays total), and never interleaved ahead of schedulable nodes.
    #[test]
    fn cycle_residue_is_final_batch_in_original_order() {
        // 0 standalone; 1 <-> 2 cycle; 3 depends on the cycle (also residue: its
        // prerequisite never completes the forward walk).
        let batches = schedule_batches_from_edges(4, &[(1, 2), (2, 1), (2, 3)]);
        assert_eq!(batches, vec![vec![0], vec![1, 2, 3]]);
        let mut all = flat(&batches);
        all.sort_unstable();
        assert_eq!(all, vec![0, 1, 2, 3]);
    }

    #[test]
    fn within_level_order_is_original_relative_order() {
        // Independent nodes keep dependency-view order inside their level — the
        // determinism the byte-identical assembled-view receipt rides on.
        let batches = schedule_batches_from_edges(3, &[]);
        assert_eq!(batches, vec![vec![0, 1, 2]]);
    }
}

fn finish_resolved_graph_assembly(
    modules: Rc<im::Vector<Rc<TypedModule>>>,
    diag_chunks: Vec<Rc<im::Vector<Rc<ErrorNode>>>>,
    binding_fork_counts: (usize, usize),
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> Result<Rc<ResolvedGraph>, String> {
    let (same_tree_fork_count, cross_tree_fork_count) = binding_fork_counts;
    let item_registry = modules.iter().fold(v1_rt::rc_empty_map(), |acc, typed| {
        v1_rt::rc_map_merge(acc, typed.item_registry.clone())
    });
    let expanded_registry =
        v1_compiler_infer::expand_transitive_services(modules.clone(), item_registry, 5);
    let diagnostics: Rc<im::Vector<Rc<ErrorNode>>> = Rc::new({
        let mut acc = im::Vector::new();
        for chunk in &diag_chunks {
            acc.extend(chunk.iter().cloned());
        }
        acc
    });
    let total_fork_count = same_tree_fork_count + cross_tree_fork_count;
    if total_fork_count > 0 && floor_verbose() {
        eprintln!(
            "[binding-fork-ledger] same_tree={same_tree_fork_count} cross_tree={cross_tree_fork_count} total={total_fork_count}"
        );
    }
    let modules =
        v1_compiler_infer::rewire_type_env_parent_links(modules.clone(), source_indices.clone());
    let modules = v1_compiler_infer::rewire_type_env_import_str_binding_identity(
        modules.clone(),
        source_indices.clone(),
    );
    let modules =
        v1_compiler_infer::rewire_func_env_parent_links(modules.clone(), source_indices.clone());
    let has_v1_seed = v1_compiler_infer::corpus_has_v1_seed_source_indices(modules.clone());
    let emit_graph_info = v1_compiler_infer::build_emit_graph_info(modules.clone(), has_v1_seed);
    Ok(Rc::new(ResolvedGraph {
        modules,
        item_registry: expanded_registry,
        diagnostics,
        emit_graph_info,
    }))
}

/// When every module in the closure is already in the typed cache, skip the
/// schedule-dispatch loop's per-module `collect_parent_envs` and
/// `build_variant_export_surface` work — both exist only to feed a cold
/// `typecheck_module` — and jump straight to closure assembly (expand, rewire,
/// emit). The assembled output matches the serial dispatch path because
/// variant surfaces are not consulted on a cache hit and parent-env diagnostics
/// are empty when every import parent is already in the store.
///
/// Under the content key this is also the all-hits PROBE: the closure is walked
/// in resolver order (imports precede importers), each module's key derived from
/// its imports' interface hashes as their cached results are read. Returns
/// `Ok(None)` on the first store miss — the caller falls through to the schedule
/// loop, which recomputes keys the same way (hits are cheap Rc clones, so the
/// repeated lookups cost nothing over the old name-keyed precheck).
fn try_reconcile_all_cache_hits(
    closure_modules: &[Rc<v1_compiler_resolve::ResolvedModule>],
    closure_names: &[String],
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
    index: &MultiEntryIndex,
) -> Result<Option<Rc<ResolvedGraph>>, String> {
    let mut modules_vec = im::Vector::new();
    let mut diag_chunks: Vec<Rc<im::Vector<Rc<ErrorNode>>>> =
        Vec::with_capacity(closure_modules.len() * 2);
    let mut same_tree_fork_count: usize = 0;
    let mut cross_tree_fork_count: usize = 0;
    let empty_parent_diags = Rc::new(im::Vector::new());
    let mut interface_hash_by_name: std::collections::HashMap<String, String> =
        std::collections::HashMap::with_capacity(closure_modules.len());

    for (resolved, mod_name) in closure_modules.iter().zip(closure_names.iter()) {
        let decl_file = workspace_relative_repo_path(&resolved.module.span.file);
        {
            check_index_module_source_identity(index, mod_name, &decl_file)?;
        }
        let typed_key =
            typed_module_content_key(index, resolved, mod_name, &interface_hash_by_name)?;
        let Some(tc_result) = index_get_typed(index, &typed_key)? else {
            return Ok(None);
        };
        note_interface_hash(&mut interface_hash_by_name, mod_name, &tc_result);
        modules_vec.push_back(tc_result.typed.clone());
        diag_chunks.push(empty_parent_diags.clone());
        diag_chunks.push(tc_result.diagnostics.clone());
        for fork in tc_result.binding_forks.iter() {
            if fork.same_tree {
                same_tree_fork_count += 1;
            } else {
                cross_tree_fork_count += 1;
            }
        }
    }

    finish_resolved_graph_assembly(
        Rc::new(modules_vec),
        diag_chunks,
        (same_tree_fork_count, cross_tree_fork_count),
        source_indices,
    )
    .map(Some)
}

fn qualified_name_module_path_prefix(name: &str) -> Option<String> {
    if !name.contains('.') {
        return None;
    }
    name.rfind('.').and_then(|pos| {
        if pos > 0 {
            Some(name[..pos].to_string())
        } else {
            None
        }
    })
}

fn collect_qualified_projection_module_paths_from_node(
    node: Rc<Node>,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
    out: &mut HashSet<String>,
) {
    let name = authored_name_at(source_indices.clone(), node.clone());
    if let Some(prefix) = qualified_name_module_path_prefix(&name) {
        out.insert(prefix);
    }
    for child in node.children.iter() {
        collect_qualified_projection_module_paths_from_node(
            child.clone(),
            source_indices.clone(),
            out,
        );
    }
    for param in node.params.iter() {
        collect_qualified_projection_module_paths_from_node(
            param.clone(),
            source_indices.clone(),
            out,
        );
    }
    if let Some(inferred) = &node.inferred {
        if let Some(inner) = inferred_to_node(inferred.clone()) {
            collect_qualified_projection_module_paths_from_node(inner, source_indices.clone(), out);
        }
    }
    if let Some(type_annotation) = &node.type_annotation {
        collect_qualified_projection_module_paths_from_node(
            type_annotation.clone(),
            source_indices.clone(),
            out,
        );
    }
    if let Some(body) = &node.body {
        collect_qualified_projection_module_paths_from_node(
            body.clone(),
            source_indices.clone(),
            out,
        );
    }
}

/// Record the source-content hash for `source` (insert-if-absent: the tree is a fixed
/// snapshot within a process, so first sight is authoritative). This is the source-hash
/// key term of `typed_module_content_key`; it is recorded exactly where the content is
/// already in hand — never re-read from disk at key-derivation time (purity: the key
/// derives from declared inputs, not a fresh WorldRead).
fn note_source_hash(index: &MultiEntryIndex, source: &Rc<v1_compiler_compile::SourceFile>) {
    let mut map = index.source_hash_by_file.borrow_mut();
    if !map.contains_key(&source.path) {
        map.insert(
            source.path.clone(),
            v1_rt::atom_identity_hash(source.content.clone()),
        );
    }
}

fn parse_module_heads_for_pool_census(
    index: &MultiEntryIndex,
    source: Rc<v1_compiler_compile::SourceFile>,
) -> Result<(Rc<Node>, Rc<NewlineIndex>), String> {
    note_source_hash(index, &source);
    let tokens = v1_compiler_tokenize::tokenize(source.content.clone(), source.path.clone());
    let nl_index = build_newline_index(source.path.clone(), source.content.clone());
    let current_table = index.intern_table.borrow().clone();
    let single_si: Rc<HashMap<String, Rc<NewlineIndex>>> = Rc::new({
        let mut m = HashMap::new();
        m.insert(source.path.clone(), nl_index.clone());
        m
    });
    let parsed = v1_compiler_parse::parse_with_table(tokens, single_si, current_table);
    *index.intern_table.borrow_mut() = parsed.intern_table.clone();
    // Pool census needs declaration heads only — do NOT install full-body ASTs into
    // `parse_cache` here. Closure resolve retains full bodies on its own cache miss.
    if let Some(err) = &parsed.result.error {
        let span = diagnostic_to_span(err.diagnostic.clone());
        let loc = format_error_loc(&span.file, span.start, &Rc::new(HashMap::new()));
        return Err(format!(
            "symbol_index qualified-projection census refused: parse failed for {}: {}",
            loc,
            diagnostic_to_message(err.diagnostic.clone())
        ));
    }
    match &parsed.result.module {
        Some(module) => Ok((census_heads_module_node(module.clone()), nl_index)),
        None => Err(format!(
            "symbol_index qualified-projection census refused: no module in {}",
            source.path
        )),
    }
}

fn parse_module_node_from_index_source(
    index: &MultiEntryIndex,
    source: Rc<v1_compiler_compile::SourceFile>,
) -> Result<(Rc<Node>, Rc<NewlineIndex>), String> {
    note_source_hash(index, &source);
    let cached = index.parse_cache.borrow().get(&source.path).cloned();
    let (parse_result, nl_index) = match cached {
        Some(entry) => entry,
        None => {
            let tokens =
                v1_compiler_tokenize::tokenize(source.content.clone(), source.path.clone());
            let nl_index = build_newline_index(source.path.clone(), source.content.clone());
            let current_table = index.intern_table.borrow().clone();
            let single_si: Rc<HashMap<String, Rc<NewlineIndex>>> = Rc::new({
                let mut m = HashMap::new();
                m.insert(source.path.clone(), nl_index.clone());
                m
            });
            let parsed = v1_compiler_parse::parse_with_table(tokens, single_si, current_table);
            *index.intern_table.borrow_mut() = parsed.intern_table.clone();
            let entry = (parsed.result.clone(), nl_index.clone());
            index
                .parse_cache
                .borrow_mut()
                .insert(source.path.clone(), entry.clone());
            entry
        }
    };
    if let Some(err) = &parse_result.error {
        let span = diagnostic_to_span(err.diagnostic.clone());
        let loc = format_error_loc(&span.file, span.start, &Rc::new(HashMap::new()));
        return Err(format!(
            "symbol_index qualified-projection census refused: parse failed for {}: {}",
            loc,
            diagnostic_to_message(err.diagnostic.clone())
        ));
    }
    match &parse_result.module {
        Some(module) => Ok((module.clone(), nl_index)),
        None => Err(format!(
            "symbol_index qualified-projection census refused: no module in {}",
            source.path
        )),
    }
}

// SCAFFOLD (§7 seed-retained HAND-RUST — namespace lane, fill side)
// LAYERED CENSUS (namespace-resolution-design.md §7.5: "fill is policy-agnostic —
// fill = whole tree; policy gates lookup, never fill"): the entry's closure census
// is built exactly as if the pool did not exist (bare-name visibility, variant-alias
// gating, and services stay closure-scoped — a pool homonym must not shift what a
// compiled module's bare names mean), and the ENTIRE indexed pool underlays it as a
// QUALIFIED-ONLY entries layer, so dotted references reach any pool module. The
// qualified fill is parse-grade (tokenize + parse, never resolve — a pool module's
// imports belong to its own tree view) and entry-independent, so it is cached once
// per process on `MultiEntryIndex`; the closure census is per-entry. The prior
// whole-pool single census let fill homonyms vanish closure bare variant aliases
// (corpus-count gating) and flip unique item bindings to ambiguous — bare names a
// compiled module resolved on main went undefined (measured on the whole-tree gate:
// bare GET/Persistent/JsonValue across 28 witness rows).
// 🟡 dissolve-on: multi-entry SymbolIndex authority modeled in .dag (census over the
// indexed pool — namespace-resolution-design.md / type-env-single-authority lane).
fn pool_parse(index: &MultiEntryIndex) -> Result<Rc<PoolParse>, String> {
    if let Some(cached) = index.pool_parse.borrow().clone() {
        return Ok(cached);
    }
    // Deterministic pool order (sorted module paths keeps every derived census
    // build reproducible — determinism gate).
    let mut pool_paths: Vec<String> = index.source_files.keys().cloned().collect();
    pool_paths.sort();
    let mut combined_si: HashMap<String, Rc<NewlineIndex>> = HashMap::new();
    let mut nodes_by_file: Vec<(String, Rc<Node>)> = Vec::with_capacity(pool_paths.len());
    for module_path in pool_paths {
        let source = index
            .source_files
            .get(&module_path)
            .cloned()
            .expect("pool path came from source_files keys");
        let (module, nl_index) = parse_module_heads_for_pool_census(index, source)?;
        let file = nl_index.file.clone();
        combined_si.insert(file.clone(), nl_index);
        nodes_by_file.push((file, module));
    }
    let parsed = Rc::new(PoolParse {
        nodes_by_file,
        combined_si: Rc::new(combined_si),
    });
    *index.pool_parse.borrow_mut() = Some(parsed.clone());
    Ok(parsed)
}

fn pool_qualified_fill(index: &MultiEntryIndex) -> Result<Rc<SymbolIndex>, String> {
    if let Some(cached) = index.pool_qualified_fill.borrow().clone() {
        return Ok(cached);
    }
    let pool = pool_parse(index)?;
    let nodes: im::Vector<Rc<Node>> = pool
        .nodes_by_file
        .iter()
        .map(|(_, node)| node.clone())
        .collect();
    let fill = v1_compiler_infer::build_symbol_index_qualified_fill(
        Rc::new(nodes),
        pool.combined_si.clone(),
    );
    *index.pool_qualified_fill.borrow_mut() = Some(fill.clone());
    Ok(fill)
}

/// The source root (longest prefix match) a workspace-relative file lives under,
/// or None for files outside every indexed root.
fn source_tree_root_of(roots: &[String], file: &str) -> Option<String> {
    let mut best: Option<String> = None;
    for root in roots {
        // Roots arrive as the caller spelled them — CI's claim_executor passes
        // ABSOLUTE `--source-root "$ROOT/dag"` while pool/decl paths here are
        // workspace-relative. Normalize before comparing: without this, every
        // lookup missed under absolute roots and the bare-reference loader and
        // the per-module census underlay silently no-oped in CI while working
        // locally (relative roots) — a CI-vs-local dual-surface split.
        let trimmed = workspace_relative_repo_path(root.trim_end_matches('/'));
        let trimmed = trimmed.trim_end_matches('/');
        if (file == trimmed || file.starts_with(&format!("{trimmed}/")))
            && best.as_deref().is_none_or(|b| trimmed.len() > b.len())
        {
            best = Some(trimmed.to_string());
        }
    }
    best
}

/// The SAME-TREE bare census for one source root: the full census (bare +
/// qualified + services) over the root's WHOLE-TREE COMPILE CLOSURE — every pool
/// module under the root plus the pool modules import-reached from them. This is
/// gate parity by construction: it is exactly the module set the root's whole-tree
/// gate compile holds in its closure census, so a bare name a module resolves
/// under the gate resolves identically here (e.g. a dag witness's bare
/// `LiveTreeDisposition`, declared only in `v2.std.live_tree` but import-reached
/// from the dag tree). Built lazily, cached per root.
fn tree_bare_census_for_root(
    index: &MultiEntryIndex,
    root: &str,
) -> Result<Rc<SymbolIndex>, String> {
    if let Some(hit) = index.tree_bare_census.borrow().get(root) {
        return Ok(hit.clone());
    }
    let pool = pool_parse(index)?;
    let trimmed = root.trim_end_matches('/');
    let prefix = format!("{trimmed}/");
    let mut reached: HashSet<String> = HashSet::new();
    let mut queue: VecDeque<String> = VecDeque::new();
    for (file, _) in pool.nodes_by_file.iter() {
        if file == trimmed || file.starts_with(&prefix) {
            reached.insert(file.clone());
            queue.push_back(file.clone());
        }
    }
    while let Some(importer) = queue.pop_front() {
        let Some(targets) = index.module_graph_facts.adjacency.get(&importer) else {
            continue;
        };
        for path in targets {
            if reached.insert(path.clone()) {
                queue.push_back(path.clone());
            }
        }
    }
    let nodes: im::Vector<Rc<Node>> = pool
        .nodes_by_file
        .iter()
        .filter(|(file, _)| reached.contains(file))
        .map(|(_, node)| node.clone())
        .collect();
    let census = v1_compiler_infer::build_symbol_index_census_nodes(
        Rc::new(nodes),
        pool.combined_si.clone(),
    );
    index
        .tree_bare_census
        .borrow_mut()
        .insert(root.to_string(), census.clone());
    Ok(census)
}

/// Whole-pool census: every pool module regardless of tree. The loader's
/// cross-tree fallback (see the `pool_bare_census` field note) — a v2 module's
/// bare `gunbc_ci_spec` (declared in dag/gunbc/ci_spec.dag) resolves here after
/// missing the v2 tree census, so the provider is pulled and becomes
/// closure-visible at typecheck.
fn pool_bare_census(index: &MultiEntryIndex) -> Result<Rc<SymbolIndex>, String> {
    if let Some(hit) = index.pool_bare_census.borrow().as_ref() {
        return Ok(hit.clone());
    }
    let pool = pool_parse(index)?;
    let nodes: im::Vector<Rc<Node>> = pool
        .nodes_by_file
        .iter()
        .map(|(_, node)| node.clone())
        .collect();
    let census = v1_compiler_infer::build_symbol_index_census_nodes(
        Rc::new(nodes),
        pool.combined_si.clone(),
    );
    *index.pool_bare_census.borrow_mut() = Some(census.clone());
    Ok(census)
}

fn build_symbol_index_for_reconcile(
    index: &MultiEntryIndex,
    graph: Rc<v1_compiler_resolve::ModuleGraph>,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> Result<Rc<SymbolIndex>, String> {
    Ok(v1_compiler_infer::symbol_index_with_qualified_fill(
        v1_compiler_infer::build_symbol_index_census(graph.modules.clone(), source_indices),
        pool_qualified_fill(index)?,
    ))
}

fn reconcile_with_typed_cache(
    graph: Rc<v1_compiler_resolve::ModuleGraph>,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
    intern_table: Rc<InternTable>,
    index: &MultiEntryIndex,
) -> Result<Rc<ResolvedGraph>, String> {
    let mut module_index: Rc<HashMap<String, Rc<TypedModule>>> = v1_rt::rc_empty_map();
    let mut diag_chunks: Vec<Rc<im::Vector<Rc<ErrorNode>>>> = Vec::new();
    let mut variant_surfaces: Rc<HashMap<String, Rc<v1_compiler_infer::VariantExportSurface>>> =
        v1_rt::rc_empty_map();
    // Corpus-wide bare-name census lives on SymbolIndex.global_bare (namespace-resolution-design.md §8 PR-4):
    // built once, order-independent, over the whole graph before any module typechecks — see
    // global_bare_fallback_invariant in v1_compiler_infer_env. Layering (§7.5 "fill =
    // whole tree; policy gates lookup, never fill"): the base below is the entry's
    // closure census plus the whole-pool QUALIFIED underlay; each module that
    // actually typechecks additionally gets its OWN tree's bare census underlaid
    // (bare = own tree, qualified = whole pool, cross-tree bare stays refused),
    // composed lazily per root in `tree_symbol_index_memo`.
    let symbol_index =
        build_symbol_index_for_reconcile(index, graph.clone(), source_indices.clone())?;
    let mut tree_symbol_index_memo: std::collections::HashMap<String, Rc<SymbolIndex>> =
        std::collections::HashMap::new();

    // S2a move 2 (resolver-graph-major-design.md §7): per-module typecheck is DISPATCHED in
    // the module-node schedule's antichain-batch order, with the typed cache as the
    // node-keyed store a dependent's handler reads its imports' results from — once-per-node
    // holds by schedule (a node appears once), not merely by cache lookup. The ResolvedGraph
    // stays an ASSEMBLED VIEW in the resolver's original module order (the loop below this
    // one), so the output is byte-identical to the legacy serial fold; module-grain purity
    // (a result is a function of the module and its import closure, not of dispatch order)
    // is the same assumption the shared typed cache already ships on, held by the
    // every-order equivalence oracles (§6.1).
    let closure_modules: Vec<Rc<v1_compiler_resolve::ResolvedModule>> =
        graph.modules.iter().cloned().collect();
    let closure_names: Vec<String> = closure_modules
        .iter()
        .map(|m| authored_name_at(source_indices.clone(), m.module.clone()))
        .collect();
    if let Some(assembled) = try_reconcile_all_cache_hits(
        &closure_modules,
        &closure_names,
        source_indices.clone(),
        index,
    )? {
        return Ok(assembled);
    }
    let schedule = module_schedule_batches(&closure_modules, &closure_names);
    // Interface hashes of processed modules, for dependents' content keys — filled in
    // batch order (a batch's imports all live in earlier batches), read by
    // `typed_module_content_key` at each module's store lookup.
    let mut interface_hash_by_name: std::collections::HashMap<String, String> =
        std::collections::HashMap::with_capacity(closure_modules.len());
    let mut dispatched: Vec<
        Option<(
            Rc<im::Vector<Rc<ErrorNode>>>,
            Rc<v1_compiler_infer::TypecheckModuleResult>,
        )>,
    > = vec![None; closure_modules.len()];

    for batch in &schedule {
        for &slot in batch {
            let resolved = closure_modules[slot].clone();
            let mod_name = closure_names[slot].clone();
            // Collision-honesty guard (union-resolve receipt §6.3): the typed cache is keyed by
            // authored name and shared across every co-resident entry, so a name that resolves
            // from two DIFFERENT declaring files in one process is a co-residence surprise — the
            // shared store must fail loud here, never silently serve one file's typecheck for the
            // other's (that would be a §5 fail-open: a divergent resolution passing as plausible).
            // build_module_index already walls tree-wide module-path collisions at index build;
            // this is the same wall at the cache seam the union widens (e.g. an on-disk entry whose
            // module path shadows an indexed module reached via entry_source_from_index_or_disk).
            // Normalize to the workspace-relative form so a module reached both index-loaded
            // (absolute path) and via the disk-entry fallback (relative path) is recognized as ONE
            // authority — the guard must fire on genuinely different files, not path representations.
            let decl_file = workspace_relative_repo_path(&resolved.module.span.file);
            {
                check_index_module_source_identity(index, &mod_name, &decl_file)?;
            }
            let typed_key =
                typed_module_content_key(index, &resolved, &mod_name, &interface_hash_by_name)?;
            let cached = index_get_typed(index, &typed_key)?;
            let was_cache_hit = cached.is_some();
            let parent_diags = if was_cache_hit {
                Rc::new(im::Vector::new())
            } else {
                let parent_envs_started = std::time::Instant::now();
                let parent_result = v1_compiler_infer::collect_parent_envs(
                    resolved.clone(),
                    module_index.clone(),
                    source_indices.clone(),
                );
                resolve_stage_slot_add(|s| {
                    s.parent_envs += parent_envs_started.elapsed().as_nanos()
                });
                parent_result.diagnostics.clone()
            };
            let tc_result = match cached {
                Some(hit) => hit,
                None => {
                    // Once-per-node receipt (§6.2): count only genuine computes (cache misses).
                    bump_typecheck_compute_count();
                    if phase_profile::phase_profile_enabled() {
                        eprintln!("[typecheck-attribution] module={mod_name} start");
                    }
                    let module_tc_started = std::time::Instant::now();
                    // Same-tree bare underlay for the module being typechecked
                    // (bare = own tree, qualified = whole pool); out-of-root
                    // modules keep the closure-only bare universe.
                    let module_symbol_index =
                        match source_tree_root_of(&index.source_roots, &decl_file) {
                            Some(root) => match tree_symbol_index_memo.get(&root) {
                                Some(hit) => hit.clone(),
                                None => {
                                    let composed = v1_compiler_infer::symbol_index_with_bare_fill(
                                        symbol_index.clone(),
                                        tree_bare_census_for_root(index, &root)?,
                                    );
                                    tree_symbol_index_memo.insert(root, composed.clone());
                                    composed
                                }
                            },
                            None => symbol_index.clone(),
                        };
                    let computed = v1_compiler_infer::typecheck_module(
                        resolved.clone(),
                        module_index.clone(),
                        variant_surfaces.clone(),
                        source_indices.clone(),
                        intern_table.clone(),
                        module_symbol_index,
                    );
                    // Per-module attribution for the typecheck-dominant resolves measured
                    // 2026-07-04 (a closure sat in typecheck for 13+ min after ~1s of
                    // parse+resolve+normalize). Threshold keeps the floor log quiet;
                    // anything over it is a pathology-lane candidate by name.
                    let module_tc_elapsed = module_tc_started.elapsed();
                    resolve_stage_slot_add(|s| s.typecheck_compute += module_tc_elapsed.as_nanos());
                    let module_tc_ms = module_tc_elapsed.as_millis();
                    if module_tc_ms >= 2_000 {
                        eprintln!("[typecheck-attribution] module={mod_name} ms={module_tc_ms}");
                    }
                    let computed = index_insert_typed(index, typed_key.clone(), computed)?;
                    computed
                }
            };
            note_interface_hash(&mut interface_hash_by_name, &mod_name, &tc_result);
            let typed = tc_result.typed.clone();
            let typed_path = authored_name_at(source_indices.clone(), typed.module.clone());
            variant_surfaces = v1_rt::rc_map_insert(
                variant_surfaces.clone(),
                typed_path.clone(),
                v1_compiler_infer::build_variant_export_surface(
                    typed.clone(),
                    variant_surfaces.clone(),
                    source_indices.clone(),
                ),
            );
            module_index = v1_rt::rc_map_insert(module_index, typed_path, typed.clone());
            dispatched[slot] = Some((parent_diags, tc_result));
        }
    }

    // Binding-fork ledger receipt (declared interim, lane ruling REVISED 2026-07-11: novelty,
    // not tree, is the refusal axis). ALL pre-existing binding forks — same-tree AND cross-tree —
    // ride the typed out-of-band channel (TypecheckModuleResult.binding_forks), never diagnostics
    // (consumers read diagnostics as compile cleanliness), keeping the pre-cut overlay-wins winner
    // (behavior-preserving: main already resolves these by overlay-wins, so refusing retroactively
    // would be a regression, not a fail-open being closed). Counted per run, PARTITIONED by tree,
    // on the receipt surface the floor prints — the std-consolidation worklist, and strictly better
    // than main's SILENT overlay-wins. TREE only labels the dissolution partition (same-tree =
    // homonym/fork within one tree; cross-tree = v1-seed-vs-v2 migration debt). The actual WALL is
    // novelty-refusal (a separate per-PR gate diffing this ledger against a drift-gated baseline;
    // follow-up work item), NOT an in-run refusal (that would double floor cost). Dissolve-on: std
    // consolidation / namespace Rule-1 terminal.
    let mut same_tree_fork_count: usize = 0;
    let mut cross_tree_fork_count: usize = 0;
    // Assembled view (original resolver order): dispatch order above is the schedule's
    // concern; the graph handed to consumers — module list, registry merge order, and
    // diagnostic order — is assembled in the exact order the serial fold produced, so the
    // result is byte-identical regardless of how the schedule batched the closure.
    let mut modules_vec = im::Vector::new();
    for (slot, entry) in dispatched.into_iter().enumerate() {
        let (parent_diags, tc_result) = entry.unwrap_or_else(|| {
            unreachable!(
                "module '{}' missing from the dispatch store: module_schedule_batches must \
                 cover every closure node exactly once",
                closure_names[slot]
            )
        });
        modules_vec.push_back(tc_result.typed.clone());
        diag_chunks.push(parent_diags);
        diag_chunks.push(tc_result.diagnostics.clone());
        for fork in tc_result.binding_forks.iter() {
            if fork.same_tree {
                same_tree_fork_count += 1;
            } else {
                cross_tree_fork_count += 1;
            }
        }
    }

    finish_resolved_graph_assembly(
        Rc::new(modules_vec),
        diag_chunks,
        (same_tree_fork_count, cross_tree_fork_count),
        source_indices,
    )
}

fn format_error_loc(file: &str, start: i64, si: &HashMap<String, Rc<NewlineIndex>>) -> String {
    match si.get(file) {
        Some(idx) => {
            let lc = byte_to_line_col(idx.clone(), start);
            format!("{}:{}:{}", file, lc.line, lc.col)
        }
        None => file.to_string(),
    }
}

fn format_error_node(
    d: &Rc<ErrorNode>,
    source_indices: &Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> String {
    let span = diagnostic_to_span(d.diagnostic.clone());
    let loc = format_error_loc(&span.file, span.start, source_indices);
    format!(
        "{}: error: {}",
        loc,
        diagnostic_to_message(d.diagnostic.clone())
    )
}

fn format_error_nodes(
    diags: &Rc<im::Vector<Rc<ErrorNode>>>,
    source_indices: &Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> String {
    diags
        .iter()
        .filter(|d| is_error_diagnostic(d.diagnostic.clone()))
        .map(|d| format_error_node(d, source_indices))
        .collect::<Vec<_>>()
        .join("\n")
}

fn resolved_graph_from_sources(
    sources: Vec<Rc<v1_compiler_compile::SourceFile>>,
    typecheck_gate: ResolveTypecheckGate,
) -> Result<
    (
        Rc<v1_compiler_compile::ResolvedGraph>,
        Rc<HashMap<String, Rc<NewlineIndex>>>,
    ),
    String,
> {
    let result = match typecheck_gate {
        ResolveTypecheckGate::Strict => {
            v1_compiler_compile::compile_to_resolved(Rc::new(sources.into()))
        }
        ResolveTypecheckGate::DiscoveryCorpusAdvisory => {
            v1_compiler_compile::compile_to_resolved_discovery_corpus_advisory(Rc::new(
                sources.into(),
            ))
        }
    };

    let has_errors = result
        .diagnostics
        .iter()
        .any(|d| is_resolve_typecheck_blocking(d.diagnostic.clone(), typecheck_gate));
    if has_errors {
        let si: HashMap<String, Rc<NewlineIndex>> = result
            .newline_indices
            .iter()
            .map(|idx| (idx.file.clone(), idx.clone()))
            .collect();
        let mut msgs = Vec::new();
        for d in result.diagnostics.iter() {
            if !is_resolve_typecheck_blocking(d.diagnostic.clone(), typecheck_gate) {
                log_discovery_advisory_typecheck(d, &si, typecheck_gate);
                continue;
            }
            let span = diagnostic_to_span(d.diagnostic.clone());
            let loc = match si.get(&span.file) {
                Some(idx) => {
                    let lc = byte_to_line_col(idx.clone(), span.start);
                    format!("{}:{}:{}", span.file, lc.line, lc.col)
                }
                None => span.file.clone(),
            };
            msgs.push(format!(
                "{}: error: {}",
                loc,
                diagnostic_to_message(d.diagnostic.clone())
            ));
        }
        return Err(msgs.join("\n"));
    }

    let graph = result
        .graph
        .clone()
        .ok_or_else(|| "compilation produced no graph".to_string())?;
    Ok((graph, result.source_indices.clone()))
}

pub fn make_eval_context(
    graph: &v1_compiler_compile::ResolvedGraph,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
    execution_mode: v1_interpreter::ExecutionMode,
) -> v1_interpreter::InterpContext {
    make_eval_context_with_fixture_store(graph, source_indices, execution_mode, None)
}

/// Evaluate `gunbc.output_policy.resolve_channel_policy` from the .dag authority at
/// the current CLI verbosity and install the per-channel decisions for the
/// interpreter's host-effect trace funnel (`v1_interpreter::output_decision`). The
/// decision logic lives entirely in .dag; this only transports the evaluated
/// verdicts across the seed↔.dag boundary. Best-effort: if the policy module can't
/// be resolved/evaluated, the funnel keeps its `Full` fallback (pre-funnel behavior).
pub fn install_output_policy(source_roots: &[String]) {
    use v1_interpreter::{OutputDecision, Value};
    let (verbose, quiet) = match v1_interpreter::cli_verbosity() {
        v1_interpreter::Verbosity::Verbose => (true, false),
        v1_interpreter::Verbosity::Quiet => (false, true),
        v1_interpreter::Verbosity::Normal => (false, false),
    };
    let entry = "dag/gunbc/output_policy.dag";
    let (graph, indices) = match resolve_entry_graph_shared(source_roots, entry) {
        Ok(g) => g,
        Err(_) => return,
    };
    let ctx = make_eval_context(&graph, indices, v1_interpreter::ExecutionMode::Wet);
    let policy = match v1_interpreter::run_in_context_with_args(
        &ctx,
        "resolve_channel_policy",
        &[
            (Some("verbose".to_string()), Value::Bool(verbose)),
            (Some("quiet".to_string()), Value::Bool(quiet)),
        ],
        false,
    ) {
        Ok(v) => v,
        Err(_) => return,
    };
    let Value::Record { fields, .. } = &policy else {
        return;
    };
    let decision = |name: &str| -> OutputDecision {
        match ctx.field(fields, name) {
            Some(Value::Variant { variant_name, .. }) => {
                if ctx.sym_eq(*variant_name, "Suppressed") {
                    OutputDecision::Suppressed
                } else if ctx.sym_eq(*variant_name, "Condensed") {
                    OutputDecision::Condensed
                } else {
                    OutputDecision::Full
                }
            }
            _ => OutputDecision::Full,
        }
    };
    v1_interpreter::set_output_policy([
        decision("diagnostic"),
        decision("claim_result"),
        decision("progress"),
        decision("shell_trace"),
        decision("instrumentation"),
    ]);
}

/// Evaluate `extdeps.render.surface.resolve_group_syntax(github_actions)` from the
/// .dag authority and install the per-target group-marker strings for the host-effect
/// trace grouping (`v1_interpreter::group_begin`/`group_end`). `github_actions` is
/// read from the environment (`GITHUB_ACTIONS=true`, the runner's own signal) — the
/// ONLY seed-side fact; which markers that target implies stays the .dag authority's.
/// Best-effort: if the module can't resolve/evaluate, grouping stays off (ungrouped,
/// pre-grouping behavior).
pub fn install_group_syntax(source_roots: &[String]) {
    use v1_interpreter::{InstalledGroupSyntax, Value};
    let github_actions = std::env::var("GITHUB_ACTIONS")
        .map(|v| v == "true")
        .unwrap_or(false);
    let entry = "dag/extdeps/render/surface.dag";
    let (graph, indices) = match resolve_entry_graph_shared(source_roots, entry) {
        Ok(g) => g,
        Err(_) => return,
    };
    let ctx = make_eval_context(&graph, indices, v1_interpreter::ExecutionMode::Wet);
    let syntax = match v1_interpreter::run_in_context_with_args(
        &ctx,
        "resolve_group_syntax",
        &[(
            Some("github_actions".to_string()),
            Value::Bool(github_actions),
        )],
        false,
    ) {
        Ok(v) => v,
        Err(_) => return,
    };
    let Value::Record { fields, .. } = &syntax else {
        return;
    };
    let str_field = |name: &str| -> Option<String> {
        match ctx.field(fields, name) {
            Some(Value::Str(s)) => Some(s.clone()),
            _ => None,
        }
    };
    let (Some(open_prefix), Some(open_suffix)) =
        (str_field("open_prefix"), str_field("open_suffix"))
    else {
        return;
    };
    // close_line is an Optional: Present { value: "::endgroup::" } | Absent (none).
    let close_line = match ctx.field(fields, "close_line") {
        Some(Value::Variant {
            variant_name,
            fields: vf,
            ..
        }) if ctx.sym_eq(*variant_name, "Present") => match ctx.field(vf, "value") {
            Some(Value::Str(s)) => Some(s.clone()),
            _ => None,
        },
        _ => None,
    };
    v1_interpreter::set_group_syntax(InstalledGroupSyntax {
        open_prefix,
        open_suffix,
        close_line,
    });
}

pub fn make_eval_context_with_fixture_store(
    graph: &v1_compiler_compile::ResolvedGraph,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
    execution_mode: v1_interpreter::ExecutionMode,
    fixture_store: Option<Rc<crate::recorded_fixture::RecordedFixtureStore>>,
) -> v1_interpreter::InterpContext {
    make_eval_context_with_runtime_options(
        graph,
        source_indices,
        execution_mode,
        fixture_store,
        None,
    )
}

pub fn make_eval_context_with_runtime_options(
    graph: &v1_compiler_compile::ResolvedGraph,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
    execution_mode: v1_interpreter::ExecutionMode,
    fixture_store: Option<Rc<crate::recorded_fixture::RecordedFixtureStore>>,
    whole_tree_published_keys: Option<Rc<std::collections::HashSet<String>>>,
) -> v1_interpreter::InterpContext {
    v1_interpreter::InterpContext::with_runtime_options(
        graph,
        source_indices,
        execution_mode,
        fixture_store,
        whole_tree_published_keys,
    )
}

fn dag_source_roots(source_roots: &[String]) -> Vec<String> {
    let mut dag: Vec<String> = source_roots
        .iter()
        .filter(|r| {
            let p = Path::new(r.as_str());
            p.ends_with("dag") || p.file_name().is_some_and(|n| n == "dag")
        })
        .cloned()
        .collect();
    for root in source_roots {
        let child = Path::new(root).join("dag");
        if child.is_dir() {
            dag.push(child.to_string_lossy().into_owned());
        }
    }
    dag.sort();
    dag.dedup();
    dag
}

pub fn precompute_whole_tree_published_mock_keys(
    source_roots: &[String],
) -> Result<std::collections::HashSet<String>, String> {
    let dag_roots = dag_source_roots(source_roots);
    if dag_roots.is_empty() {
        return Ok(std::collections::HashSet::new());
    }
    let index = build_module_index(&dag_roots);
    // Only modules that DECLARE a `PublishedMockCase` corpus can contribute keys —
    // `resolve_published_mock_keys` reads them by exact type annotation. Strict-
    // resolving the whole 600+ module tree to find the ~13 declarers is §2
    // irrelevant work, and that transient whole-tree `ResolvedGraph` is the floor's
    // dominant RSS (measured ~1.46 GiB to produce ~58 strings). Select the
    // declarers and resolve only their transitive import closures. The `.contains`
    // prefilter is a safe over-inclusive candidate set: `.dag` has no comment
    // syntax (a string match is structural), and the downstream
    // `type_annotation_names(.., "PublishedMockCase")` check is exact, so a
    // false-positive file only widens the closure slightly — it cannot fabricate a key.
    let declarers: Vec<Rc<v1_compiler_compile::SourceFile>> = index
        .values()
        .filter(|sf| sf.content.contains("PublishedMockCase"))
        .cloned()
        .collect();
    if declarers.is_empty() {
        return Ok(std::collections::HashSet::new());
    }
    let facts = build_module_graph_facts_live(&dag_roots);
    let all_sources = resolve_transitively(declarers, &index, &facts)?;
    if all_sources.is_empty() {
        return Ok(std::collections::HashSet::new());
    }
    let (graph, source_indices) =
        resolved_graph_from_sources(all_sources, ResolveTypecheckGate::Strict)?;
    let ctx = v1_interpreter::InterpContext::with_runtime_options(
        &graph,
        source_indices,
        v1_interpreter::ExecutionMode::Wet,
        None,
        None,
    );
    v1_interpreter::resolve_published_mock_keys(&ctx)
        .map_err(|e| format!("whole-tree published mock corpus precompute: {e}"))
}

/// Build an interpreter context over the WHOLE source-root corpus (every `.dag`
/// module under `source_roots`), resolved in one pass under the Strict gate — the
/// same whole-tree resolve `precompute_whole_tree_published_mock_keys` performs,
/// but retaining the context so a `.dag` reflection accessor (e.g.
/// `fn_arrow_decl_facts_live`) walks `ctx.modules == the whole tree` rather than a
/// single entry's import closure. This is the #5364 widening substrate: coverage
/// goes from per-entry resolve-closure to whole-tree-in-one-pass. The marshaling
/// runs in THIS context's interner, so reflected `Node` values are self-consistent
/// (no cross-context Symbol mismatch).
/// `exclude_substrings` drop modules whose source path contains any listed
/// substring BEFORE the resolve. This is required, not optional: the corpus
/// contains intentionally-malformed scanner fixture inputs (e.g.
/// `src/v2/test/fixture/layering_scan/**/plant.dag` declaring imports of modules
/// that do not exist) which are test DATA referenced by string path, not live
/// code — a Strict whole-tree resolve over them fails on the deliberate
/// `unresolved import`. Excluding them is a coverage decision, so the count of
/// dropped modules is returned for the caller to log (DESIGN §6 — no silent cap).
pub struct WholeTreeCtx {
    pub ctx: v1_interpreter::InterpContext,
    pub modules_resolved: usize,
    pub modules_excluded: usize,
}

pub struct WholeTreeStrictSources {
    pub sources: Vec<Rc<v1_compiler_compile::SourceFile>>,
    pub modules_resolved: usize,
    pub modules_excluded: usize,
}

pub fn whole_tree_strict_sources(
    source_roots: &[String],
    exclude_substrings: &[String],
) -> Result<WholeTreeStrictSources, String> {
    let index = build_module_index(source_roots);
    let total = index.len();
    let all_sources: Vec<Rc<v1_compiler_compile::SourceFile>> = index
        .iter()
        .filter(|(module_path, sf)| {
            let p = sf.path.replace('\\', "/");
            !exclude_substrings
                .iter()
                .any(|sub| p.contains(sub.as_str()) || module_path.contains(sub.as_str()))
        })
        .map(|(_, sf)| sf.clone())
        .collect();
    if all_sources.is_empty() {
        return Err("whole-tree corpus is empty (no .dag modules under source roots)".to_string());
    }
    let modules_excluded = total - all_sources.len();
    Ok(WholeTreeStrictSources {
        sources: all_sources,
        modules_resolved: total - modules_excluded,
        modules_excluded,
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WholeCorpusSemanticOracle {
    pub diagnostic_fingerprint: String,
    pub rust_corpus_repr: String,
    /// Canonical JSON identity hash of the full `EmitGraphInfo` (resolved emit repr).
    pub emit_graph_fingerprint: String,
    /// Aggregate per-module diagnostics + emit-repr rows + graph-level emit metadata.
    pub corpus_fingerprint: String,
    pub modules_resolved: usize,
    pub per_module_rows: usize,
}

fn sort_json_object_keys(value: serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Object(map) => {
            let mut keys: Vec<String> = map.keys().cloned().collect();
            keys.sort();
            let mut out = serde_json::Map::new();
            for key in keys {
                if let Some(child) = map.get(&key) {
                    out.insert(key, sort_json_object_keys(child.clone()));
                }
            }
            serde_json::Value::Object(out)
        }
        serde_json::Value::Array(items) => serde_json::Value::Array(
            items
                .into_iter()
                .map(sort_json_object_keys)
                .collect::<Vec<_>>(),
        ),
        other => other,
    }
}

fn canonical_json_identity_hash<T: Serialize>(value: &T) -> Result<String, String> {
    let raw = serde_json::to_value(value).map_err(|e| e.to_string())?;
    let sorted = sort_json_object_keys(raw);
    let bytes = serde_json::to_vec(&sorted).map_err(|e| e.to_string())?;
    Ok(v1_rt::bytes_identity_hash(&bytes))
}

fn module_defined_type_names(
    module: &TypedModule,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> BTreeSet<String> {
    use ItemKind::{DataItem, TypeItem};
    let mut names = BTreeSet::new();
    for item in module.items.iter() {
        if matches!(item_kind(item.clone()), TypeItem | DataItem) {
            names.insert(authored_name_at(source_indices.clone(), item.clone()));
        }
    }
    names
}

fn module_emit_repr_fingerprint(
    module: &TypedModule,
    emit_info: &crate::v1_compiler_infer_emit_info::EmitGraphInfo,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> Result<String, String> {
    use crate::v1_compiler_infer_emit_info::TypeSummary;

    let type_names = module_defined_type_names(module, source_indices);
    let mut type_summaries = BTreeMap::<String, TypeSummary>::new();
    for name in type_names {
        if let Some(summary) = emit_info.type_summaries.get(&name) {
            type_summaries.insert(name, summary.as_ref().clone());
        }
    }

    canonical_json_identity_hash(&type_summaries)
}

pub fn whole_corpus_semantic_oracle_snapshot(
    source_roots: &[String],
    exclude_substrings: &[String],
) -> Result<WholeCorpusSemanticOracle, String> {
    use crate::v1_compiler_infer_emit_info::RustCorpusRepr::{FaithfulFreeMonoid, HostNative};

    let picked = whole_tree_strict_sources(source_roots, exclude_substrings)?;
    let result = v1_compiler_compile::compile_to_resolved(Rc::new(picked.sources.into()));
    let graph = result.graph.as_ref().ok_or_else(|| {
        let si: HashMap<String, Rc<NewlineIndex>> = result
            .newline_indices
            .iter()
            .map(|idx| (idx.file.clone(), idx.clone()))
            .collect();
        format!(
            "whole-corpus strict resolve failed:\n{}",
            format_error_nodes(&result.diagnostics, &Rc::new(si))
        )
    })?;
    let source_indices = result.source_indices.clone();
    let mut diag_lines: Vec<String> = graph
        .diagnostics
        .iter()
        .map(|d| v1_compiler_compile::serialize_diagnostic(d.clone()))
        .collect();
    diag_lines.sort();
    let diagnostic_fingerprint = v1_rt::bytes_identity_hash(diag_lines.join("\n").as_bytes());
    let rust_corpus_repr = match graph.emit_graph_info.corpus_repr {
        HostNative => "HostNative".to_string(),
        FaithfulFreeMonoid => "FaithfulFreeMonoid".to_string(),
    };
    let emit_graph_fingerprint = canonical_json_identity_hash(graph.emit_graph_info.as_ref())?;

    let mut modules: Vec<Rc<TypedModule>> = graph.modules.iter().cloned().collect();
    modules.sort_by(|left, right| {
        let left_path = authored_name_at(source_indices.clone(), left.module.clone());
        let right_path = authored_name_at(source_indices.clone(), right.module.clone());
        left_path.cmp(&right_path)
    });

    let mut per_module_lines = Vec::with_capacity(modules.len());
    for module in &modules {
        let module_path = authored_name_at(source_indices.clone(), module.module.clone());
        let mut module_diag_lines: Vec<String> = graph
            .diagnostics
            .iter()
            .filter(|diag| diag.module_name.as_str() == module_path.as_str())
            .map(|diag| v1_compiler_compile::serialize_diagnostic(diag.clone()))
            .collect();
        module_diag_lines.sort();
        let module_diag_fingerprint =
            v1_rt::bytes_identity_hash(module_diag_lines.join("\n").as_bytes());
        let module_emit_fingerprint = module_emit_repr_fingerprint(
            module.as_ref(),
            graph.emit_graph_info.as_ref(),
            source_indices.clone(),
        )?;
        per_module_lines.push(format!(
            "{module_path}\t{module_diag_fingerprint}\t{module_emit_fingerprint}"
        ));
    }

    let per_module_rows = per_module_lines.len();
    let per_module_blob = per_module_lines.join("\n");
    let corpus_fingerprint = v1_rt::bytes_identity_hash(
        format!(
            "diagnostic_fingerprint={diagnostic_fingerprint}\n\
             emit_graph_fingerprint={emit_graph_fingerprint}\n\
             rust_corpus_repr={rust_corpus_repr}\n\
             per_module:\n{per_module_blob}"
        )
        .as_bytes(),
    );

    Ok(WholeCorpusSemanticOracle {
        diagnostic_fingerprint,
        rust_corpus_repr,
        emit_graph_fingerprint,
        corpus_fingerprint,
        modules_resolved: picked.modules_resolved,
        per_module_rows,
    })
}

pub fn whole_tree_resolved_ctx(
    source_roots: &[String],
    exclude_substrings: &[String],
    execution_mode: v1_interpreter::ExecutionMode,
) -> Result<WholeTreeCtx, String> {
    let picked = whole_tree_strict_sources(source_roots, exclude_substrings)?;
    let modules_resolved = picked.modules_resolved;
    let modules_excluded = picked.modules_excluded;
    let (graph, source_indices) =
        resolved_graph_from_sources(picked.sources, ResolveTypecheckGate::Strict)?;
    Ok(WholeTreeCtx {
        ctx: v1_interpreter::InterpContext::with_runtime_options(
            graph.as_ref(),
            source_indices,
            execution_mode,
            None,
            None,
        ),
        modules_resolved,
        modules_excluded,
    })
}

pub fn closure_subject_for_entry(index: &MultiEntryIndex, entry: &str) -> Result<String, String> {
    let sources = load_sources_for_entry_with_pool(index, entry)?;
    Ok(subject_digest_for_closure(&sources))
}

/// M0 ancestry-retention probe (v1-run-stability-throughline M0): per-module vs
/// distinct-spine entry counts for the typecheck-env maps — the quadratic witness the
/// deleted `cache_walk` (#5888, dissolved #5899) never measured (it counted payload-Rc
/// sharing, which is healthy; the byte carrier is the per-module materialized map SPINES).
/// Pure reader over one strict whole-tree resolve; prints `[ancestry]` lines and the peak
/// RSS; no behavior change anywhere else. `retained` sums every module's map sizes (what
/// the typed cache holds resident); `distinct` sums each unique Rc spine once (what is
/// actually allocated). `dup_factor = retained/distinct` — a factor ≫1 on the ancestry
/// maps is the located §2 duplication; flat ≈1 means spines are shared and M1 is done.
pub fn whole_tree_ancestry_retention_probe(
    source_roots: &[String],
    exclude_substrings: &[String],
) -> Result<(), String> {
    let picked = whole_tree_strict_sources(source_roots, exclude_substrings)?;
    let modules_resolved = picked.modules_resolved;
    let modules_excluded = picked.modules_excluded;
    let (graph, source_indices) =
        resolved_graph_from_sources(picked.sources, ResolveTypecheckGate::Strict)?;

    struct FieldTally {
        name: &'static str,
        retained_entries: usize,
        distinct_entries: usize,
        distinct_spines: std::collections::HashSet<usize>,
    }
    impl FieldTally {
        fn new(name: &'static str) -> Self {
            FieldTally {
                name,
                retained_entries: 0,
                distinct_entries: 0,
                distinct_spines: std::collections::HashSet::new(),
            }
        }
        fn add(&mut self, spine_ptr: usize, entries: usize) {
            self.retained_entries += entries;
            if self.distinct_spines.insert(spine_ptr) {
                self.distinct_entries += entries;
            }
        }
    }

    let mut tallies = [
        FieldTally::new("tec.str_bindings"),
        FieldTally::new("tec.deps_map"),
        FieldTally::new("tec.cycle_set_str"),
        FieldTally::new("tec.variant_locals"),
        FieldTally::new("te.str_bindings"),
        FieldTally::new("te.ancestry_str_bindings"),
        FieldTally::new("te.bindings"),
        FieldTally::new("te.source_visible_names"),
        FieldTally::new("te.inductive_fields.keys"),
        FieldTally::new("te.recursive_type_set"),
    ];
    // Inductive-field LIST mass (Σ list lengths) tracked separately from key count —
    // the concat-on-collision duplication class shows up in list length, not key count.
    let mut ind_lists_retained: usize = 0;
    let mut ind_lists_distinct: usize = 0;
    let mut ind_list_spines: std::collections::HashSet<usize> = std::collections::HashSet::new();

    let mut per_module: Vec<(String, usize, usize, usize)> = Vec::new();

    for m in graph.modules.iter() {
        let te = &m.type_env;
        let tec = &m.type_env_cache;
        tallies[0].add(
            Rc::as_ptr(&tec.str_bindings) as usize,
            tec.str_bindings.len(),
        );
        tallies[1].add(Rc::as_ptr(&tec.deps_map) as usize, tec.deps_map.len());
        tallies[2].add(
            Rc::as_ptr(&tec.cycle_set_str) as usize,
            tec.cycle_set_str.len(),
        );
        tallies[3].add(
            Rc::as_ptr(&tec.variant_locals) as usize,
            tec.variant_locals.len(),
        );
        tallies[4].add(Rc::as_ptr(&te.str_bindings) as usize, te.str_bindings.len());
        tallies[5].add(
            Rc::as_ptr(&te.ancestry_str_bindings) as usize,
            te.ancestry_str_bindings.len(),
        );
        tallies[6].add(Rc::as_ptr(&te.bindings) as usize, te.bindings.len());
        tallies[7].add(
            Rc::as_ptr(&te.source_visible_names) as usize,
            te.source_visible_names.len(),
        );
        tallies[8].add(
            Rc::as_ptr(&te.inductive_fields) as usize,
            te.inductive_fields.len(),
        );
        tallies[9].add(
            Rc::as_ptr(&te.recursive_type_set) as usize,
            te.recursive_type_set.len(),
        );

        let module_ind_mass: usize = te.inductive_fields.iter().map(|(_, v)| v.len()).sum();
        ind_lists_retained += module_ind_mass;
        if ind_list_spines.insert(Rc::as_ptr(&te.inductive_fields) as usize) {
            ind_lists_distinct += module_ind_mass;
        }

        per_module.push((
            authored_name_at(source_indices.clone(), m.module.clone()),
            tec.str_bindings.len(),
            te.ancestry_str_bindings.len(),
            module_ind_mass,
        ));
    }

    eprintln!(
        "[ancestry] modules={modules_resolved} excluded={modules_excluded} (strict whole-tree resolve)"
    );
    let mut retained_total = 0usize;
    let mut distinct_total = 0usize;
    for t in &tallies {
        let dup = if t.distinct_entries > 0 {
            t.retained_entries as f64 / t.distinct_entries as f64
        } else {
            1.0
        };
        eprintln!(
            "[ancestry] field={} retained_entries={} distinct_spines={} distinct_entries={} dup_factor={:.2}",
            t.name,
            t.retained_entries,
            t.distinct_spines.len(),
            t.distinct_entries,
            dup
        );
        retained_total += t.retained_entries;
        distinct_total += t.distinct_entries;
    }
    let ind_dup = if ind_lists_distinct > 0 {
        ind_lists_retained as f64 / ind_lists_distinct as f64
    } else {
        1.0
    };
    eprintln!(
        "[ancestry] field=te.inductive_fields.list_mass retained={ind_lists_retained} distinct={ind_lists_distinct} dup_factor={ind_dup:.2}"
    );
    eprintln!(
        "[ancestry] TOTAL retained_entries={retained_total} distinct_entries={distinct_total} dup_factor={:.2}",
        if distinct_total > 0 {
            retained_total as f64 / distinct_total as f64
        } else {
            1.0
        }
    );

    per_module.sort_by(|a, b| (b.1 + b.2).cmp(&(a.1 + a.2)));
    for (name, tec_str, anc_str, ind_mass) in per_module.iter().take(10) {
        eprintln!(
            "[ancestry] top module={name} tec.str_bindings={tec_str} te.ancestry_str_bindings={anc_str} inductive_list_mass={ind_mass}"
        );
    }

    match peak_rss_vhwm_bytes() {
        Some(bytes) => {
            eprintln!("[ancestry] peak RSS: {bytes} bytes (VmHWM) modules={modules_resolved}")
        }
        None => eprintln!("[ancestry] peak RSS: unavailable (no /proc/self/status)"),
    }
    Ok(())
}

pub fn run_claim(ctx: &v1_interpreter::InterpContext, function: &str) -> ClaimOutcome {
    // ProcessExit is the wet-gate return convention (ExitSuccess => Pass, ExitFailure => Fail).
    // NotProcessExit stays NotBool — fail-closed preserved for genuine type errors. Reuses
    // pre-existing classify_exit. Required: emitted pre-push drift --wet gate runs through
    // claim_batch -> run_claim; without this mapping ExitSuccess -> exit 1 false-blocks push
    // (receipt: claim_batch rebuilt on reverted seed reproduced the false-block).
    match v1_interpreter::run_in_context(ctx, function, false) {
        Ok(v1_interpreter::Value::Bool(true)) => ClaimOutcome::Pass,
        Ok(v1_interpreter::Value::Bool(false)) => ClaimOutcome::Fail,
        Ok(other) => match classify_exit(&other, ctx) {
            ExitClass::Success => ClaimOutcome::Pass,
            ExitClass::Failure { .. } => ClaimOutcome::Fail,
            ExitClass::NotProcessExit { type_name } => ClaimOutcome::NotBool { got: type_name },
        },
        Err(e) => ClaimOutcome::RuntimeError {
            message: format!("{}", e),
        },
    }
}

pub fn run_claim_measured(
    ctx: &v1_interpreter::InterpContext,
    closure_subject_digest: &str,
    function: &str,
) -> (ClaimOutcome, v1_interpreter::PerformanceReceipt) {
    let subject_key =
        crate::resolved_graph_cache::witness_work_subject_key(closure_subject_digest, function);
    v1_interpreter::eval_profile_reset();
    v1_interpreter::eval_subject_set(subject_key.clone());
    if let Some(budget_ms) = ctx.witness_eval_budget() {
        ctx.arm_eval_deadline(budget_ms);
    }
    let started = std::time::Instant::now();
    let cpu_started_nanos = v1_interpreter::thread_cpu_nanos();
    let outcome = run_claim(ctx, function);
    // CPU consumed by THIS (witness-eval) thread — the budget metric, so the completion-side
    // check matches the cooperative stride-poll and neither fires on cold-I/O or contention
    // wall time. wall_nanos stays the measurement/receipt basis (unchanged).
    let cpu_nanos = v1_interpreter::thread_cpu_nanos().saturating_sub(cpu_started_nanos);
    let wall_nanos = started.elapsed().as_nanos();
    ctx.clear_eval_deadline();
    v1_interpreter::eval_subject_clear();
    let outcome = budget_completion_outcome(ctx.witness_eval_budget(), outcome, cpu_nanos);
    let outcome = wall_budget_completion_outcome(ctx.witness_wall_budget(), outcome, wall_nanos);
    let receipt =
        v1_interpreter::performance_receipt_from_witness(subject_key, function, wall_nanos);
    (outcome, receipt)
}

/// Completion-side budget enforcement: the cooperative deadline polls every 4096
/// eval_expr dispatches, so a witness whose time is spent in few dispatches (native
/// builtin-heavy) can finish over budget without ever hitting a poll. A Pass that
/// exceeded the budget converts to the same typed refusal here — the witness is over
/// the fast-lane classification either way, and silent green would fail open on the
/// operator 5s rule. A Fail/RuntimeError stays itself: those are already loud, and
/// replacing a genuine finding with the budget message would discard it. `cpu_nanos` is
/// THREAD CPU time (not wall), matching the stride-poll metric — a witness whose wall time
/// was inflated by cold-I/O or governor time-slicing is not misclassified as over-budget.
fn budget_completion_outcome(
    budget: Option<u64>,
    outcome: ClaimOutcome,
    cpu_nanos: u128,
) -> ClaimOutcome {
    match (budget, outcome) {
        (Some(budget_ms), ClaimOutcome::Pass) if cpu_nanos > u128::from(budget_ms) * 1_000_000 => {
            ClaimOutcome::RuntimeError {
                message: format!(
                    "{}",
                    v1_interpreter::InterpError::EvalBudgetExceeded {
                        elapsed_ms: (cpu_nanos / 1_000_000) as u64,
                        budget_ms,
                    }
                ),
            }
        }
        (_, o) => o,
    }
}

/// Whole-receipt wall budget for Wet self-host receipts: emit+cargo subprocess I/O
/// counts against wall time, not CPU. A Pass over the wall budget converts to the same
/// typed refusal — silent green would fail open on the nightly falsifier lane budget.
fn wall_budget_completion_outcome(
    budget: Option<u64>,
    outcome: ClaimOutcome,
    wall_nanos: u128,
) -> ClaimOutcome {
    match (budget, outcome) {
        (Some(budget_ms), ClaimOutcome::Pass) if wall_nanos > u128::from(budget_ms) * 1_000_000 => {
            ClaimOutcome::RuntimeError {
                message: format!(
                    "{}",
                    v1_interpreter::InterpError::WitnessWallBudgetExceeded {
                        elapsed_ms: (wall_nanos / 1_000_000) as u64,
                        budget_ms,
                    }
                ),
            }
        }
        (_, o) => o,
    }
}

#[cfg(test)]
mod budget_completion_tests {
    use super::*;

    #[test]
    fn pass_over_budget_converts_to_typed_refusal() {
        // The stride-poll blind spot: a witness burning over-budget CPU in fewer than
        // 4096 dispatches must still refuse at completion, never green silently. The
        // third arg is CPU nanos (6ms CPU > 5ms budget), matching the stride-poll metric.
        match budget_completion_outcome(Some(5), ClaimOutcome::Pass, 6_000_000) {
            ClaimOutcome::RuntimeError { message } => {
                assert!(
                    message.contains("eval budget exceeded"),
                    "typed refusal expected; got {message}"
                );
            }
            other => panic!("expected RuntimeError, got {other:?}"),
        }
    }

    #[test]
    fn pass_under_budget_stays_pass() {
        assert!(matches!(
            budget_completion_outcome(Some(5), ClaimOutcome::Pass, 4_000_000),
            ClaimOutcome::Pass
        ));
    }

    #[test]
    fn no_budget_never_converts() {
        assert!(matches!(
            budget_completion_outcome(None, ClaimOutcome::Pass, u128::MAX),
            ClaimOutcome::Pass
        ));
    }

    #[test]
    fn over_budget_fail_keeps_its_finding() {
        assert!(matches!(
            budget_completion_outcome(Some(5), ClaimOutcome::Fail, 6_000_000),
            ClaimOutcome::Fail
        ));
    }

    #[test]
    fn pass_over_wall_budget_converts_to_typed_refusal() {
        match wall_budget_completion_outcome(Some(600), ClaimOutcome::Pass, 601_000_000_000) {
            ClaimOutcome::RuntimeError { message } => {
                assert!(
                    message.contains("wet self-host receipt wall budget exceeded"),
                    "typed refusal expected; got {message}"
                );
            }
            other => panic!("expected RuntimeError, got {other:?}"),
        }
    }
}

pub fn run_value(
    ctx: &v1_interpreter::InterpContext,
    function: &str,
) -> Result<v1_interpreter::Value, String> {
    v1_interpreter::run_in_context(ctx, function, false).map_err(|e| format!("{}", e))
}

pub fn handle_ci() {
    handle_run_with_options(
        witness_layer_roots(),
        "main".to_string(),
        Some("dag/tools/gunbc_ci.dag".to_string()),
        false,
        false,
    );
}

/// Thin CLI transport handler for `gunbc converge --host <h>`: argv parse ->
/// in-process `.dag` interpreter call -> stdout/exit-code projection, no
/// converge logic here. Disposition receipt (DESIGN §3 transport-is-a-handler,
/// §7 typed self-host frontier): `gunbc.fleet_converge_cli`'s
/// `gunbc_converge_cli_stage0_receiver_disposition`.
pub fn handle_converge(host: String) {
    let roots = witness_layer_roots();
    let (graph, indices) =
        match resolve_entry_graph_shared(&roots, "dag/gunbc/fleet_converge_cli.dag") {
            Ok(g) => g,
            Err(e) => {
                eprintln!("error: {e}");
                std::process::exit(1);
            }
        };
    let ctx = make_eval_context(&graph, indices, v1_interpreter::ExecutionMode::Wet);
    let args = [(
        Some("host".to_string()),
        v1_interpreter::Value::Str(host.clone()),
    )];
    let result =
        match v1_interpreter::run_in_context_with_args(&ctx, "converge_cli_output", &args, false) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("runtime error: {e}");
                std::process::exit(1);
            }
        };
    let v1_interpreter::Value::Record { fields, .. } = &result else {
        eprintln!(
            "error: converge_cli_output returned an unexpected shape: {:?}",
            result
        );
        std::process::exit(1);
    };
    let line = match ctx.field(fields, "line") {
        Some(v1_interpreter::Value::Str(s)) => s.clone(),
        _ => {
            eprintln!("error: converge_cli_output.line was not a String");
            std::process::exit(1);
        }
    };
    let converged = matches!(
        ctx.field(fields, "converged"),
        Some(v1_interpreter::Value::Bool(true))
    );
    let reason = match ctx.field(fields, "reason") {
        Some(v1_interpreter::Value::Variant {
            variant_name,
            fields: vf,
            ..
        }) if ctx.sym_eq(*variant_name, "Present") => match ctx.field(vf, "value") {
            Some(v1_interpreter::Value::Str(s)) => Some(s.clone()),
            _ => None,
        },
        _ => None,
    };
    println!("{line}");
    if let Some(reason) = &reason {
        eprintln!("{reason}");
    }
    if !converged {
        std::process::exit(1);
    }
}

#[path = "pre_push.rs"]
mod pre_push;

/// Thin CLI transport handler for `claim_batch --pre-push`: stdin parse and gate
/// orchestration live in `pre_push`; disposition receipt in `gunbc.githooks_pre_push_cli`.
pub fn handle_pre_push() -> std::process::ExitCode {
    pre_push::run()
}

pub fn handle_run(
    source_roots: Vec<String>,
    function: String,
    entry_file: Option<String>,
    claim_run: bool,
) {
    handle_run_with_options(source_roots, function, entry_file, false, claim_run);
}

/// adhoc-c328b166-bca residual-hunt instrumentation dump: printed periodically
/// (env `GUNBC_FLATTEN_SITE_DUMP_SECS`) and once, deterministically, right
/// after a `dag run` entry returns -- the periodic dump alone races the
/// process's natural completion and under-reports on fast runs.
fn dump_residual_hunt_instrumentation() {
    let mut sites = v1_interpreter::flatten_by_site_snapshot();
    sites.sort_by(|a, b| b.3.cmp(&a.3));
    eprintln!("--- free_monoid_to_vec by call site (top 15 by items cloned) ---");
    for (file, line, calls, total) in sites.iter().take(15) {
        eprintln!("  {}:{}  calls={}  items={}", file, line, calls, total);
    }
    let (cons_calls, cons_len_sum) = v1_interpreter::list_cons_tail_split_snapshot();
    eprintln!(
        "--- list Cons-match tail split (hypothesis B): calls={} receiver_len_sum={} ---",
        cons_calls, cons_len_sum
    );
    let mut freq = v1_interpreter::call_frequency_snapshot();
    freq.sort_by(|a, b| a.0.cmp(b.0));
    eprintln!("--- hypothesis A call frequency ---");
    for (name, calls) in freq.iter() {
        eprintln!("  {}  calls={}", name, calls);
    }
    let mut big_folds = v1_interpreter::big_fold_by_dag_site_snapshot();
    big_folds.sort_by(|a, b| b.2.cmp(&a.2));
    eprintln!("--- fold_list receivers >1000 items, by .dag closure site (top 10) ---");
    for (site, calls, total) in big_folds.iter().take(10) {
        eprintln!("  {}  calls={}  items={}", site, calls, total);
    }
    let mut times = v1_interpreter::builtin_time_snapshot();
    times.sort_by(|a, b| b.2.cmp(&a.2));
    eprintln!("--- builtin inclusive wall time (top 15 by nanos) ---");
    for (name, calls, nanos) in times.iter().take(15) {
        eprintln!("  {}  calls={}  ms={}", name, calls, nanos / 1_000_000);
    }
    let mut self_times = v1_interpreter::dag_fn_self_time_snapshot();
    self_times.sort_by(|a, b| b.2.cmp(&a.2));
    eprintln!("--- .dag fn self time (top 20 by ms) ---");
    for (name, calls, nanos) in self_times.iter().take(20) {
        eprintln!("  {}  calls={}  self_ms={}", name, calls, nanos / 1_000_000);
    }
    let (memo_lookups, memo_hits, memo_distinct) = v1_interpreter::parse_memo_global_snapshot();
    eprintln!(
        "--- parse memo effectiveness discriminator: lookups={} hits={} distinct_keys={} (lookups>>distinct & hits==0 => memo never serves a re-attempted span) ---",
        memo_lookups, memo_hits, memo_distinct
    );
    let mut callers = v1_interpreter::fold_caller_snapshot();
    callers.sort_by(|a, b| b.2.cmp(&a.2));
    eprintln!("--- LARGE fold_list callers (items>=100/call), top 15 by total items; .dag fn [left|right], elem type ---");
    for (caller, calls, total, maxlen, elem) in callers.iter().take(15) {
        eprintln!(
            "  {}  calls={}  total_items={}  max_len={}  elem={}",
            caller, calls, total, maxlen, elem
        );
    }
}

pub fn handle_run_with_options(
    source_roots: Vec<String>,
    function: String,
    entry_file: Option<String>,
    dry_run: bool,
    claim_run: bool,
) {
    if source_roots.is_empty() {
        eprintln!("error: provide at least one --source-root");
        std::process::exit(1);
    }

    if claim_run && entry_file.is_none() {
        eprintln!(
            "error: --claim-run requires --entry <file.dag> (scoped import closure; \
             loading the whole --source-root tree is too large for witness runs)"
        );
        std::process::exit(1);
    }

    if let Ok(secs) = std::env::var("GUNBC_FLATTEN_SITE_DUMP_SECS") {
        if let Ok(secs) = secs.parse::<u64>() {
            std::thread::spawn(move || loop {
                std::thread::sleep(std::time::Duration::from_secs(secs));
                dump_residual_hunt_instrumentation();
            });
        }
    }

    let sources = match entry_file.as_deref() {
        Some(path) => match load_sources_for_entry(&source_roots, path) {
            Ok(sources) => sources,
            Err(msg) => {
                eprintln!("error: {}", msg);
                std::process::exit(1);
            }
        },
        None => match load_sources(&source_roots) {
            Ok(sources) => sources,
            Err(msg) => {
                eprintln!("error: {}", msg);
                std::process::exit(1);
            }
        },
    };
    eprintln!("resolved {} sources", sources.len());

    let result = v1_compiler_compile::compile_to_resolved(Rc::new(sources.into()));

    let has_errors = result
        .diagnostics
        .iter()
        .any(|d| is_interpreter_blocking_diagnostic(d.diagnostic.clone()));
    if has_errors {
        let si: HashMap<String, Rc<NewlineIndex>> = result
            .newline_indices
            .iter()
            .map(|idx| (idx.file.clone(), idx.clone()))
            .collect();
        for d in result.diagnostics.iter() {
            if !is_interpreter_blocking_diagnostic(d.diagnostic.clone()) {
                continue;
            }
            let span = diagnostic_to_span(d.diagnostic.clone());
            let loc = match si.get(&span.file) {
                Some(idx) => {
                    let lc = byte_to_line_col(idx.clone(), span.start);
                    format!("{}:{}:{}", span.file, lc.line, lc.col)
                }
                None => span.file.clone(),
            };
            eprintln!(
                "{}: error: {}",
                loc,
                diagnostic_to_message(d.diagnostic.clone())
            );
        }
        std::process::exit(1);
    }

    let graph = match result.graph.as_ref() {
        Some(g) => g,
        None => {
            eprintln!("error: compilation produced no graph");
            std::process::exit(1);
        }
    };

    eprintln!("running {}()...", function);
    let execution_mode = if dry_run {
        v1_interpreter::ExecutionMode::Hermetic
    } else {
        v1_interpreter::ExecutionMode::Wet
    };
    let ctx =
        v1_interpreter::InterpContext::new(graph, result.source_indices.clone(), execution_mode);
    v1_interpreter::with_active_context(&ctx, || {
        let run_outcome = v1_interpreter::run_in_context(&ctx, &function, !claim_run);
        v1_interpreter::print_eval_recompute_trace(&ctx);
        match run_outcome {
            Ok(val) => {
                println!("{}", val);
                if std::env::var("GUNBC_FLATTEN_SITE_DUMP_SECS").is_ok() {
                    dump_residual_hunt_instrumentation();
                }
                if claim_run {
                    match &val {
                        v1_interpreter::Value::Bool(false) => std::process::exit(1),
                        v1_interpreter::Value::Bool(true) => return,
                        other => {
                            eprintln!(
                                "error: function `{}` returned `{}`, not `Bool`. \
                                 With --claim-run the entry must return Bool (false → exit 1).",
                                function, other
                            );
                            std::process::exit(2);
                        }
                    }
                }
                match classify_exit(&val, &ctx) {
                    ExitClass::Success => {}
                    ExitClass::Failure { code, reason } => {
                        if let Some(message) = reason {
                            eprintln!("{message}");
                        }
                        std::process::exit(code);
                    }
                    ExitClass::NotProcessExit { type_name } => {
                        eprintln!(
                            "error: function `{}` returned `{}`, not `ProcessExit`. \
                             Functions invoked via `dag run` must return std/process.dag's \
                             ProcessExit so the host can map success/failure to an exit code. \
                             Wrap your rich result type in ExitSuccess / ExitFailure, or pass \
                             --claim-run for Bool witness entry points under src/v2.",
                            function, type_name
                        );
                        std::process::exit(2);
                    }
                }
            }
            Err(e) => {
                eprintln!("runtime error: {}", e);
                std::process::exit(1);
            }
        }
    });
}

enum ExitClass {
    Success,
    Failure { code: i32, reason: Option<String> },
    NotProcessExit { type_name: String },
}

fn classify_exit(val: &v1_interpreter::Value, ctx: &v1_interpreter::InterpContext) -> ExitClass {
    match val {
        v1_interpreter::Value::Variant {
            type_name,
            variant_name,
            fields,
        } => {
            if !ctx.sym_eq(*type_name, "ProcessExit") {
                return ExitClass::NotProcessExit {
                    type_name: ctx.resolve(*type_name),
                };
            }
            if ctx.sym_eq(*variant_name, "ExitSuccess") {
                ExitClass::Success
            } else if ctx.sym_eq(*variant_name, "ExitFailure") {
                let code = match ctx.field(fields, "code") {
                    Some(v1_interpreter::Value::Int(n)) => *n as i32,
                    _ => 1,
                };
                let reason = match ctx.field(fields, "reason") {
                    Some(v1_interpreter::Value::Str(s)) if !s.is_empty() => Some(s.clone()),
                    _ => None,
                };
                ExitClass::Failure { code, reason }
            } else {
                ExitClass::NotProcessExit {
                    type_name: format!("ProcessExit::{}", ctx.resolve(*variant_name)),
                }
            }
        }
        // Non-variant returns: render the actual value (symbols resolve to their
        // interned names via the active context) instead of an opaque "<non-variant>".
        // This makes `--function`-run diagnostics — e.g. a helper returning a
        // diagnostic reason Symbol — legible instead of blind.
        other => ExitClass::NotProcessExit {
            type_name: ctx.format_value(other),
        },
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ResolvedDeclRef {
    pub module: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "arm", rename_all = "snake_case")]
pub enum OwnedDataDeclInitializer {
    BoolWitnessClaim {
        witness_entry: String,
        witness_function: String,
    },
    NodeCorpus,
    Other {
        resolved: ResolvedDeclRef,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct OwnedDataDeclRecord {
    pub entry: String,
    pub module: String,
    pub decl_name: String,
    pub initializer: OwnedDataDeclInitializer,
}

fn literal_string_from_expr(node: &Rc<Node>) -> Option<String> {
    if let ExprData::ExprLiteral { value } = &*node.expr_data {
        if let LiteralValue::LitStr { value: s } = value.as_ref() {
            return Some(s.clone());
        }
    }
    None
}

fn symbol_name_from_expr(
    node: &Rc<Node>,
    source_indices: &HashMap<String, Rc<NewlineIndex>>,
) -> Option<String> {
    binding_name_from_expr(node, source_indices)
}

fn field_init_label(
    field_init: &Rc<Node>,
    source_indices: &HashMap<String, Rc<NewlineIndex>>,
) -> String {
    let si = Rc::new(source_indices.clone());
    let authored = field_init_node_name_at(field_init.clone(), si);
    if !authored.is_empty() {
        return authored;
    }
    field_init.name.clone()
}

fn field_init_named<'a>(
    record: &'a Rc<Node>,
    field: &str,
    source_indices: &HashMap<String, Rc<NewlineIndex>>,
) -> Option<Rc<Node>> {
    for child in record.children.iter() {
        if field_init_label(child, source_indices) == field {
            return Some(field_init_node_value(child.clone()));
        }
    }
    None
}

fn binding_name_from_expr(
    node: &Rc<Node>,
    source_indices: &HashMap<String, Rc<NewlineIndex>>,
) -> Option<String> {
    if let ExprData::ExprVar { .. } = &*node.expr_data {
        let name = expr_var_name_at(node.clone(), Rc::new(source_indices.clone()));
        if !name.is_empty() {
            return Some(name);
        }
    }
    let name = authored_name_at(Rc::new(source_indices.clone()), node.clone());
    if name.is_empty() {
        None
    } else {
        Some(name)
    }
}

fn extract_bool_witness_transport(
    claim_body: &Rc<Node>,
    source_indices: &HashMap<String, Rc<NewlineIndex>>,
) -> (String, String) {
    let Some(witness_node) = field_init_named(claim_body, "witness", source_indices) else {
        return (String::new(), String::new());
    };
    let entry = field_init_named(&witness_node, "entry", source_indices)
        .and_then(|n| literal_string_from_expr(&n))
        .unwrap_or_default();
    let function = field_init_named(&witness_node, "function", source_indices)
        .and_then(|n| symbol_name_from_expr(&n, source_indices))
        .unwrap_or_default();
    (entry, function)
}

fn defining_module_for_resolved_type(
    graph: &ResolvedGraph,
    source_indices: &HashMap<String, Rc<NewlineIndex>>,
    type_name: &str,
) -> Option<String> {
    let si = Rc::new(source_indices.clone());
    for tm in graph.modules.iter() {
        let mod_name = authored_name_at(si.clone(), tm.module.clone());
        if lookup_type_by_name(tm.type_env.clone(), type_name.to_string()).is_some() {
            return Some(mod_name);
        }
    }
    let parent_enum = graph
        .emit_graph_info
        .variant_to_enum
        .get(type_name)
        .cloned()?;
    for tm in graph.modules.iter() {
        let mod_name = authored_name_at(si.clone(), tm.module.clone());
        if lookup_type_by_name(tm.type_env.clone(), parent_enum.clone()).is_some() {
            return Some(mod_name);
        }
    }
    None
}

fn lookup_resolved_type_node(graph: &ResolvedGraph, type_name: &str) -> Option<Rc<Node>> {
    for tm in graph.modules.iter() {
        if let Some(node) = lookup_type_by_name(tm.type_env.clone(), type_name.to_string()) {
            return Some(node);
        }
    }
    None
}

fn declared_type_name_from_annotation(
    source_indices: &HashMap<String, Rc<NewlineIndex>>,
    type_annotation: &Rc<Node>,
) -> Option<String> {
    let si = Rc::new(source_indices.clone());
    let name = authored_name_at(si, type_annotation.clone());
    if name.is_empty() {
        None
    } else {
        Some(name)
    }
}

fn resolved_decl_ref_from_type_name(
    graph: &ResolvedGraph,
    source_indices: &HashMap<String, Rc<NewlineIndex>>,
    name: &str,
) -> Result<ResolvedDeclRef, String> {
    let module = defining_module_for_resolved_type(graph, source_indices, name)
        .ok_or_else(|| format!("no defining module for resolved type '{}'", name))?;
    Ok(ResolvedDeclRef {
        module,
        name: name.to_string(),
    })
}

fn resolved_initializer_decl_ref(
    graph: &ResolvedGraph,
    source_indices: &HashMap<String, Rc<NewlineIndex>>,
    body: &Rc<Node>,
    type_annotation: Option<&Rc<Node>>,
) -> Result<ResolvedDeclRef, String> {
    let si = Rc::new(source_indices.clone());
    if let ExprData::ExprRecordLit { parent_enum } = &*body.expr_data {
        if let Some(parent_name) = parent_enum.as_deref() {
            // A qualified constructor spelling (v2.std.verification.BoolWitnessClaim)
            // names the same arm as its bare last segment — the module prefix already
            // resolved the parent coproduct (the #6869 payload-variant class). The arm
            // check and the stored decl name both use the bare arm name; a genuinely
            // wrong variant still refuses.
            let variant_name = crate::v1_std_core::qualified_last_segment(authored_name_at(
                si.clone(),
                body.clone(),
            ));
            if variant_name.is_empty() {
                return Err(
                    "coproduct variant initializer missing constructor identity".to_string()
                );
            }
            let parent_type = lookup_resolved_type_node(graph, parent_name).ok_or_else(|| {
                format!(
                    "resolved parent coproduct '{}' not found in typed graph",
                    parent_name
                )
            })?;
            if !has_child_named(parent_type, variant_name.clone(), si.clone()) {
                return Err(format!(
                    "'{}' is not a resolved variant arm of coproduct '{}'",
                    variant_name, parent_name
                ));
            }
            let module = defining_module_for_resolved_type(graph, source_indices, parent_name)
                .ok_or_else(|| {
                    format!(
                        "no defining module for resolved coproduct '{}'",
                        parent_name
                    )
                })?;
            return Ok(ResolvedDeclRef {
                module,
                name: variant_name,
            });
        }
    }

    let inferred_name = match body.inferred.as_deref() {
        Some(InferredNode::Resolved { node }) => {
            let resolved_name = authored_name_at(si.clone(), node.clone());
            if resolved_name.is_empty() {
                None
            } else {
                Some(resolved_name)
            }
        }
        Some(InferredNode::CompilerError { message, .. }) => {
            return Err(format!("unresolved initializer type: {}", message));
        }
        Some(InferredNode::TypeVariable { .. }) => {
            return Err("unresolved initializer type variable".to_string());
        }
        None => None,
    };
    if let Some(name) = inferred_name {
        return resolved_decl_ref_from_type_name(graph, source_indices, &name);
    }
    if let Some(ann) = type_annotation {
        if let Some(name) = declared_type_name_from_annotation(source_indices, ann) {
            return resolved_decl_ref_from_type_name(graph, source_indices, &name);
        }
    }
    Err(
        "resolved initializer has empty type identity (no inferred head or declared annotation)"
            .to_string(),
    )
}

fn is_resolved_bool_witness_claim(resolved: &ResolvedDeclRef) -> bool {
    resolved.module == UNIFIED_CLAIM_VERIFICATION_MODULE && resolved.name == BOOL_WITNESS_CLAIM_TYPE
}

fn is_resolved_node_corpus(resolved: &ResolvedDeclRef) -> bool {
    resolved.module == UNIFIED_CLAIM_VERIFICATION_MODULE && resolved.name == NODE_CORPUS_TYPE
}

fn entry_typed_module(
    graph: &ResolvedGraph,
    source_indices: &HashMap<String, Rc<NewlineIndex>>,
    entry_module: &str,
) -> Result<Rc<TypedModule>, String> {
    let si = Rc::new(source_indices.clone());
    graph
        .modules
        .iter()
        .find(|tm| authored_name_at(si.clone(), tm.module.clone()) == entry_module)
        .cloned()
        .ok_or_else(|| {
            format!(
                "entry module '{}' not found in resolved graph",
                entry_module
            )
        })
}

fn owned_data_initializer_from_body(
    graph: &ResolvedGraph,
    source_indices: &HashMap<String, Rc<NewlineIndex>>,
    entry_path: &str,
    decl_name: &str,
    body: &Rc<Node>,
    type_annotation: Option<&Rc<Node>>,
) -> Result<OwnedDataDeclInitializer, String> {
    let resolved_initializer =
        resolved_initializer_decl_ref(graph, source_indices, body, type_annotation)
            .map_err(|e| format!("{entry_path}: owned data '{decl_name}': {e}"))?;
    if is_resolved_bool_witness_claim(&resolved_initializer) {
        let (witness_entry, witness_function) =
            extract_bool_witness_transport(body, source_indices);
        if witness_entry.is_empty() || witness_function.is_empty() {
            return Err(format!(
                "{}: owned data '{}' has malformed BoolWitnessClaim witness (missing entry and/or function)",
                entry_path, decl_name
            ));
        }
        return Ok(OwnedDataDeclInitializer::BoolWitnessClaim {
            witness_entry,
            witness_function,
        });
    }
    if is_resolved_node_corpus(&resolved_initializer) {
        return Ok(OwnedDataDeclInitializer::NodeCorpus);
    }
    Ok(OwnedDataDeclInitializer::Other {
        resolved: resolved_initializer,
    })
}

pub fn owned_data_decls_for_entry(
    graph: &ResolvedGraph,
    source_indices: &HashMap<String, Rc<NewlineIndex>>,
    entry_path: &str,
    entry_module: &str,
) -> Result<Vec<OwnedDataDeclRecord>, String> {
    let si = Rc::new(source_indices.clone());
    let typed_module = entry_typed_module(graph, source_indices, entry_module)
        .map_err(|e| format!("{entry_path}: {e}"))?;

    let mut records = Vec::new();
    for item in typed_module.items.iter() {
        if item.body.is_none() || item.type_annotation.is_none() {
            continue;
        }
        let decl_name = authored_name_at(si.clone(), item.clone());
        if decl_name.is_empty() {
            return Err(format!(
                "{entry_path}: owned data item in module '{}' missing authored name",
                entry_module
            ));
        }
        let info = typed_module.item_registry.get(&decl_name).ok_or_else(|| {
            format!(
                "{entry_path}: owned data '{}' missing from item_registry",
                decl_name
            )
        })?;
        if info.kind != ItemKind::DataItem {
            continue;
        }
        if info.module_name != entry_module {
            return Err(format!(
                "{entry_path}: item_registry module mismatch for '{}' (expected {}, got {})",
                decl_name, entry_module, info.module_name
            ));
        }
        if info.name != decl_name {
            return Err(format!(
                "{entry_path}: item_registry name mismatch for '{}' (registry name '{}')",
                decl_name, info.name
            ));
        }
        if !decl_name.starts_with("unified_claim_") {
            continue;
        }
        let body = item.body.as_ref().ok_or_else(|| {
            format!(
                "{entry_path}: owned data '{}' missing initializer body",
                decl_name
            )
        })?;
        let initializer = owned_data_initializer_from_body(
            graph,
            source_indices,
            entry_path,
            &decl_name,
            body,
            item.type_annotation.as_ref(),
        )?;
        records.push(OwnedDataDeclRecord {
            entry: entry_path.to_string(),
            module: entry_module.to_string(),
            decl_name,
            initializer,
        });
    }

    let discovered: HashSet<&str> = records.iter().map(|r| r.decl_name.as_str()).collect();
    for (decl_name, info) in typed_module.item_registry.iter() {
        if info.kind == ItemKind::DataItem
            && info.module_name == entry_module
            && decl_name.starts_with("unified_claim_")
        {
            if !discovered.contains(decl_name.as_str()) {
                return Err(format!(
                    "{entry_path}: item_registry data '{}' not found in entry module items",
                    decl_name
                ));
            }
        }
    }

    records.sort_by(|a, b| {
        a.entry
            .cmp(&b.entry)
            .then_with(|| a.decl_name.cmp(&b.decl_name))
    });
    Ok(records)
}

fn path_excluded(path: &Path, exclude_subpaths: &[String]) -> bool {
    let path_str = path.to_string_lossy();
    exclude_subpaths
        .iter()
        .any(|ex| !ex.is_empty() && path_str.contains(ex))
}

fn entry_likely_has_unified_claim_owned_data(content: &str) -> bool {
    content
        .lines()
        .any(|line| line.trim_start().starts_with("data unified_claim_"))
}

fn top_level_decl_names(content: &str) -> Vec<String> {
    const ITEM_KEYWORDS: [&str; 8] = [
        "data ",
        "fn ",
        "func ",
        "type ",
        "service ",
        "const ",
        "pattern ",
        "resource ",
    ];
    let mut names = Vec::new();
    for line in content.lines() {
        let Some(rest) = ITEM_KEYWORDS.iter().find_map(|kw| line.strip_prefix(kw)) else {
            continue;
        };
        let name: String = rest
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect();
        if !name.is_empty() {
            names.push(name);
        }
    }
    names
}

struct DiscoveryResolveGroup {
    entries: Vec<(String, String, usize)>,
    sources: HashMap<String, Rc<v1_compiler_compile::SourceFile>>,
    decl_names: HashMap<String, String>,
}

fn closure_group_conflict(
    group: &DiscoveryResolveGroup,
    closure: &[Rc<v1_compiler_compile::SourceFile>],
    names_by_file: &HashMap<String, Rc<Vec<String>>>,
) -> Option<(String, String, String)> {
    for source in closure {
        if group.sources.contains_key(&source.path) {
            continue;
        }
        for name in names_by_file[&source.path].iter() {
            if let Some(existing) = group.decl_names.get(name) {
                if existing != &source.path {
                    return Some((name.clone(), existing.clone(), source.path.clone()));
                }
            }
        }
    }
    None
}

fn add_closure_to_group(
    group: &mut DiscoveryResolveGroup,
    closure: Vec<Rc<v1_compiler_compile::SourceFile>>,
    names_by_file: &HashMap<String, Rc<Vec<String>>>,
) {
    for source in closure {
        if group.sources.contains_key(&source.path) {
            continue;
        }
        for name in names_by_file[&source.path].iter() {
            group.decl_names.insert(name.clone(), source.path.clone());
        }
        group.sources.insert(source.path.clone(), source);
    }
}

pub struct OwnedDataDiscovery {
    pub records: Vec<OwnedDataDeclRecord>,
    pub entry_count: usize,
    pub graph_resolves: usize,
    pub group_split_collisions: Vec<String>,
}

pub fn discover_owned_data_decls(
    source_roots: &[String],
    scan_dir: &str,
    exclude_subpaths: &[String],
) -> Result<OwnedDataDiscovery, String> {
    let scan_path = Path::new(scan_dir);
    if !scan_path.is_dir() {
        return Err(format!("scan dir does not exist: {}", scan_dir));
    }

    let mut files = Vec::new();
    collect_dag_files(scan_path, &mut files);
    files.retain(|p| !path_excluded(p, exclude_subpaths));

    let module_index = build_module_index(source_roots);
    let module_graph_facts = build_module_graph_facts_live(source_roots);

    let mut names_by_file: HashMap<String, Rc<Vec<String>>> = HashMap::new();
    let mut groups: Vec<DiscoveryResolveGroup> = Vec::new();
    let mut group_split_collisions: Vec<String> = Vec::new();
    let mut entry_count = 0usize;
    for path in files {
        let entry = path.to_string_lossy().to_string();
        let content = std::fs::read_to_string(&path)
            .map_err(|e| format!("failed to read {:?}: {}", path, e))?;
        if !entry_likely_has_unified_claim_owned_data(&content) {
            continue;
        }
        let entry_module = extract_module_path(&content).ok_or_else(|| {
            format!(
                "missing module declaration in entry {}; cannot classify owned decls",
                entry
            )
        })?;
        let marker_count = content
            .lines()
            .filter(|line| line.starts_with("data unified_claim_"))
            .count();
        entry_count += 1;

        let closure =
            load_sources_for_entry_with_index(&module_index, &module_graph_facts, &entry)?;
        for source in &closure {
            names_by_file
                .entry(source.path.clone())
                .or_insert_with(|| Rc::new(top_level_decl_names(&source.content)));
        }

        let member = (entry, entry_module, marker_count);
        let mut first_conflict: Option<(String, String, String)> = None;
        match groups.iter_mut().find(|g| {
            match closure_group_conflict(g, &closure, &names_by_file) {
                None => true,
                Some(conflict) => {
                    first_conflict.get_or_insert(conflict);
                    false
                }
            }
        }) {
            Some(group) => {
                group.entries.push(member);
                add_closure_to_group(group, closure, &names_by_file);
            }
            None => {
                if let Some((name, existing_file, new_file)) = first_conflict {
                    group_split_collisions.push(format!(
                        "entry {} split off over decl `{}` ({} vs {})",
                        member.0, name, existing_file, new_file
                    ));
                }
                let mut group = DiscoveryResolveGroup {
                    entries: vec![member],
                    sources: HashMap::new(),
                    decl_names: HashMap::new(),
                };
                add_closure_to_group(&mut group, closure, &names_by_file);
                groups.push(group);
            }
        }
    }

    let graph_resolves = groups.len();
    let mut all_records = Vec::new();
    for group in groups {
        let mut sources: Vec<Rc<v1_compiler_compile::SourceFile>> =
            group.sources.into_iter().map(|(_, v)| v).collect();
        sources.sort_by(|a, b| a.path.cmp(&b.path));
        let (graph, source_indices) =
            resolved_graph_from_sources(sources, ResolveTypecheckGate::DiscoveryCorpusAdvisory)?;
        let si: HashMap<String, Rc<NewlineIndex>> = source_indices
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        for (entry, entry_module, marker_count) in group.entries {
            let records = owned_data_decls_for_entry(&graph, &si, &entry, &entry_module)?;
            if records.len() != marker_count {
                return Err(format!(
                    "{}: merged-resolve discovery found {} owned unified_claim record(s) but the entry declares {} top-level `data unified_claim_` marker(s)",
                    entry,
                    records.len(),
                    marker_count
                ));
            }
            all_records.extend(records);
        }
    }

    all_records.sort_by(|a, b| {
        a.entry
            .cmp(&b.entry)
            .then_with(|| a.decl_name.cmp(&b.decl_name))
    });
    verify_bool_witness_transport_projection_complete(&all_records)?;
    Ok(OwnedDataDiscovery {
        records: all_records,
        entry_count,
        graph_resolves,
        group_split_collisions,
    })
}

pub fn bool_witness_claim_arm_count(records: &[OwnedDataDeclRecord]) -> usize {
    records
        .iter()
        .filter(|r| {
            matches!(
                r.initializer,
                OwnedDataDeclInitializer::BoolWitnessClaim { .. }
            )
        })
        .count()
}

fn unified_claim_arm_count(records: &[OwnedDataDeclRecord]) -> usize {
    records
        .iter()
        .filter(|r| {
            matches!(
                r.initializer,
                OwnedDataDeclInitializer::BoolWitnessClaim { .. }
                    | OwnedDataDeclInitializer::NodeCorpus
            )
        })
        .count()
}

fn illegal_other_init_count(records: &[OwnedDataDeclRecord]) -> usize {
    records
        .iter()
        .filter(|r| {
            let OwnedDataDeclInitializer::Other { resolved } = &r.initializer else {
                return false;
            };
            is_resolved_bool_witness_claim(resolved) || is_resolved_node_corpus(resolved)
        })
        .count()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnedDataDiscoveryReceipt {
    pub unified_claim_arm_count: usize,
    pub bool_witness_claim_arm_count: usize,
    pub illegal_other_init_count: usize,
    pub bool_witness_transport_row_count: usize,
    pub transport_projection_complete: bool,
}

pub const MANIFEST_INLINE_LIST_MAX: usize = 64;

pub fn compute_owned_data_discovery_receipt(
    records: &[OwnedDataDeclRecord],
) -> Result<OwnedDataDiscoveryReceipt, String> {
    verify_bool_witness_transport_projection_complete(records)?;
    let bool_witness_transport_row_count = owned_data_bool_witness_transport_tsv(records)?
        .lines()
        .filter(|l| !l.is_empty())
        .count();
    let bool_witness_claim_arm_count = bool_witness_claim_arm_count(records);
    let illegal = illegal_other_init_count(records);
    Ok(OwnedDataDiscoveryReceipt {
        unified_claim_arm_count: unified_claim_arm_count(records),
        bool_witness_claim_arm_count,
        illegal_other_init_count: illegal,
        bool_witness_transport_row_count,
        transport_projection_complete: illegal == 0
            && bool_witness_claim_arm_count == bool_witness_transport_row_count,
    })
}

pub fn verify_bool_witness_transport_projection_complete(
    records: &[OwnedDataDeclRecord],
) -> Result<(), String> {
    let arm_count = bool_witness_claim_arm_count(records);
    let tsv = owned_data_bool_witness_transport_tsv(records)?;
    let row_count = tsv.lines().filter(|l| !l.is_empty()).count();
    if arm_count != row_count {
        return Err(format!(
            "BoolWitnessClaim arm count ({arm_count}) != transport projection row count ({row_count})"
        ));
    }
    Ok(())
}

fn dag_string_escape_core(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

fn dag_manifest_scalar_escape(s: &str) -> Result<String, String> {
    if s.contains('{') || s.contains('}') {
        return Err(format!(
            "manifest scalar field must be brace-free (got '{{' or '}}'): {s:?}"
        ));
    }
    Ok(dag_string_escape_core(s))
}

fn dag_embedded_dag_source_escape(s: &str) -> String {
    dag_string_escape_core(s)
        .replace('{', "\\{")
        .replace('}', "\\}")
}

fn manifest_symbol_for_resolved_decl(module: &str, name: &str) -> String {
    match (module, name) {
        (UNIFIED_CLAIM_VERIFICATION_MODULE, BOOL_WITNESS_CLAIM_TYPE) => {
            "unified_claim_arm_bool_witness_claim".to_string()
        }
        (UNIFIED_CLAIM_VERIFICATION_MODULE, NODE_CORPUS_TYPE) => {
            "unified_claim_arm_node_corpus".to_string()
        }
        _ => format!("^{}", name),
    }
}

fn emit_owned_data_initializer(initializer: &OwnedDataDeclInitializer) -> Result<String, String> {
    match initializer {
        OwnedDataDeclInitializer::BoolWitnessClaim {
            witness_entry,
            witness_function,
        } => Ok(format!(
            "    initializer: OwnedBoolWitnessClaimInit {{\n      witness_entry: \"{}\",\n      witness_function: \"{}\"\n    }}",
            dag_manifest_scalar_escape(witness_entry)?,
            dag_manifest_scalar_escape(witness_function)?
        )),
        OwnedDataDeclInitializer::NodeCorpus => {
            Ok("    initializer: OwnedNodeCorpusInit".to_string())
        }
        OwnedDataDeclInitializer::Other { resolved } => Ok(format!(
            "    initializer: OwnedOtherInit {{\n      resolved: ResolvedDeclRef {{\n        module: \"{}\",\n        name: {}\n      }}\n    }}",
            dag_manifest_scalar_escape(&resolved.module)?,
            manifest_symbol_for_resolved_decl(&resolved.module, &resolved.name)
        )),
    }
}

pub fn emit_owned_data_manifest(
    path: &Path,
    records: &[OwnedDataDeclRecord],
) -> Result<(), String> {
    let receipt = compute_owned_data_discovery_receipt(records)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("failed to create manifest parent {:?}: {}", parent, e))?;
    }

    let mut out = String::new();
    out.push_str(
        "// GENERATED by discover_owned_data — ephemeral host transport. DO NOT COMMIT.\n",
    );
    out.push_str("module v2.test.claim.workflow.host_discovered_owned_data_manifest\n\n\n");
    out.push_str("import v2.std.collection { List }\n");
    out.push_str("import v2.std.logic { Bool }\n");
    out.push_str(
        "import v2.compiler.discovery_enumeration {\n  OwnedBoolWitnessClaimInit,\n  OwnedDataDeclRecord,\n  OwnedDataDiscoveryReceipt,\n  OwnedNodeCorpusInit,\n  OwnedOtherInit,\n  ResolvedDeclRef,\n  unified_claim_arm_bool_witness_claim,\n  unified_claim_arm_node_corpus\n}\n\n\n",
    );
    out.push_str("data host_owned_data_discovery_receipt: OwnedDataDiscoveryReceipt = OwnedDataDiscoveryReceipt {\n");
    out.push_str(&format!(
        "  unified_claim_arm_count: {},\n",
        receipt.unified_claim_arm_count
    ));
    out.push_str(&format!(
        "  bool_witness_claim_arm_count: {},\n",
        receipt.bool_witness_claim_arm_count
    ));
    out.push_str(&format!(
        "  illegal_other_init_count: {},\n",
        receipt.illegal_other_init_count
    ));
    out.push_str(&format!(
        "  bool_witness_transport_row_count: {},\n",
        receipt.bool_witness_transport_row_count
    ));
    out.push_str(&format!(
        "  transport_projection_complete: {}\n",
        receipt.transport_projection_complete
    ));
    out.push_str("}\n\n\n");
    let inline_records = if records.len() <= MANIFEST_INLINE_LIST_MAX {
        records
    } else {
        &[]
    };
    if inline_records.is_empty() && !records.is_empty() {
        out.push_str(
            "// Large corpus: inline list omitted; standing gates use host_owned_data_discovery_receipt + transport sidecar.\n",
        );
    }
    out.push_str("data host_discovered_owned_decls: List<OwnedDataDeclRecord> = [\n");
    for (idx, rec) in inline_records.iter().enumerate() {
        if idx > 0 {
            out.push(',');
            out.push('\n');
        }
        out.push_str("  OwnedDataDeclRecord {\n");
        out.push_str(&format!(
            "    entry: \"{}\",\n",
            dag_manifest_scalar_escape(&rec.entry)?
        ));
        out.push_str(&format!(
            "    module: \"{}\",\n",
            dag_manifest_scalar_escape(&rec.module)?
        ));
        out.push_str(&format!(
            "    decl_name: \"{}\",\n",
            dag_manifest_scalar_escape(&rec.decl_name)?
        ));
        out.push_str(&format!(
            "{}\n",
            emit_owned_data_initializer(&rec.initializer)?
        ));
        out.push_str("  }");
    }
    out.push_str("\n]\n");

    std::fs::write(path, out).map_err(|e| format!("failed to write manifest {:?}: {}", path, e))
}

pub fn owned_data_bool_witness_transport_tsv(
    records: &[OwnedDataDeclRecord],
) -> Result<String, String> {
    let mut rows = Vec::new();
    for rec in records {
        let OwnedDataDeclInitializer::BoolWitnessClaim {
            witness_entry,
            witness_function,
        } = &rec.initializer
        else {
            continue;
        };
        if witness_entry.is_empty() || witness_function.is_empty() {
            return Err(format!(
                "{}: owned data '{}' has malformed BoolWitnessClaim witness transport (missing entry and/or function)",
                rec.entry, rec.decl_name
            ));
        }
        let label = rec
            .decl_name
            .strip_prefix("unified_claim_")
            .unwrap_or(rec.decl_name.as_str());
        rows.push(format!("{label}\t{witness_entry}\t{witness_function}"));
    }
    rows.sort();
    let mut out = rows.join("\n");
    if !out.is_empty() {
        out.push('\n');
    }
    Ok(out)
}

#[derive(Clone)]
pub struct DiscoveryRow {
    pub label: String,
    pub entry: String,
    pub function: String,
    /// Declared live-tree disposition of this row's ENTRY file (`v2.std.live_tree`):
    /// `true` = ReadsLiveTree (undeclared defaults here, fail-closed — never
    /// predict-skip), `false` = declared SubstrateInputsOnly (selection-eligible).
    pub reads_live_tree: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntryResolveReceipt {
    pub entry: String,
    pub closure_subject: String,
    pub resolve_nanos: u128,
    /// Stage attribution of `resolve_nanos` (load/parse/resolve/normalize/
    /// typecheck/parent-envs/assembly/ownership); the lump minus
    /// `stage_nanos.attributed_total()` is the unattributed residue.
    pub stage_nanos: ResolveStageNanos,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveryWitnessOutcome {
    pub entry: String,
    pub function: String,
    pub outcome: ClaimOutcome,
}

/// A witness row excluded from discovery enrollment (exclusion substring, long lane, …).
/// Counted and logged at roster build — never a silent skip (§5 deferred-and-detected).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeferredDiscoveryRow {
    pub entry: String,
    pub function: String,
    /// Which exclusion substring matched (`witness_exclusion_substrings` authority).
    pub exclude_reason: String,
    /// Entry-grain live-tree disposition (`v2.std.live_tree`); undeclared = true.
    pub reads_live_tree: bool,
}

/// Phase 0(b) admission invariant refusal — an excluded witness row with zero executing consumers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnexecutedDeferredWitness {
    pub entry: String,
    pub function: String,
}

#[derive(Debug)]
pub struct DiscoverySummary {
    pub total: usize,
    pub passed: usize,
    pub skipped: usize,
    /// Witness rows excluded from discovery at scan time — counted, typed, observable.
    pub deferred_rows: Vec<DeferredDiscoveryRow>,
    /// PredictOnly mode: rows the selection predicted unaffected (they still ran).
    pub predicted_unaffected: Vec<(String, String)>,
    /// PredictOnly mode: predicted-unaffected rows whose cold run was red — each line is a
    /// counted, typed attribution of a missing selection edge (never a rerun trigger).
    pub divergences: Vec<String>,
    pub failures: Vec<String>,
    pub witness_outcomes: Vec<DiscoveryWitnessOutcome>,
    pub entry_resolve_receipts: Vec<EntryResolveReceipt>,
    pub total_resolve_nanos: u128,
    /// Run-total stage attribution of `total_resolve_nanos` (per-entry rows summed).
    pub total_stage_nanos: ResolveStageNanos,
    pub performance_receipts: Vec<v1_interpreter::PerformanceReceipt>,
    pub total_measured_nanos: u128,
    /// Distinct modules in this shard's union closure — the union of authored module names across
    /// every graph the shard resolved (its prefix contexts plus each roster entry). The per-shard
    /// input-size axis that per-shard resident memory is a function of. Max-merged across shards —
    /// the heaviest shard's closure governs the peak. This is the calibration pair's missing half:
    /// per-shard RSS is already emitted, the node count was not.
    ///
    /// Derived from the graphs, NOT from `typecheck_compute_count()`. That counter counts typecheck
    /// cache MISSES on the current thread and is never reset in production, so it equals a closure
    /// size only from a cold start — a condition this measurement cannot assume. It is warm here on
    /// the `width == 1` path (the same thread already resolved the changed-file entries in
    /// `floor_diff_edits_from_line_ranges` and both prefix entries), and warm across repeat calls in
    /// `floor_skip_discovery_witness`, which runs discovery three times in one thread. The union of
    /// module names is a property of the source closure: independent of cache warmth, resolve order,
    /// and — critically for a calibration datum — of the diff under test.
    pub roster_closure_nodes: usize,
}

#[derive(Debug, Clone)]
pub struct TimingPercentiles {
    pub p50: u128,
    pub p90: u128,
    pub p95: u128,
    pub p99: u128,
    pub p100: u128,
}

pub fn compute_percentiles(mut values: Vec<u128>) -> TimingPercentiles {
    if values.is_empty() {
        return TimingPercentiles {
            p50: 0,
            p90: 0,
            p95: 0,
            p99: 0,
            p100: 0,
        };
    }
    values.sort_unstable();
    let len = values.len();
    let clamp_idx = |f: f64| {
        let idx = (len as f64 * f) as usize;
        idx.min(len - 1)
    };

    TimingPercentiles {
        p50: values[clamp_idx(0.50)],
        p90: values[clamp_idx(0.90)],
        p95: values[clamp_idx(0.95)],
        p99: values[clamp_idx(0.99)],
        p100: values[len - 1],
    }
}

// SCAFFOLD (§7 hand-Rust shrink-to-zero, dissolution named): the v1 evaluator measures its own
// per-witness resolve+eval percentiles here — seed-side justified (the evaluator cannot measure
// itself without circularity). The *rendering* of these timings now lives in `dag/gunbc/ci_render.dag`
// (boxed Frames over `std.render`, width-parameterized by the medium's `Viewport.width`); this Rust
// only produces the measured data. Full dissolution: ROADMAP lane "CI observability" emits the
// `TimingPercentiles` rows as a substrate value so a .dag witness measures + histograms natively,
// at which point this measurement struct collapses too.
pub struct HistogramData {
    pub included: usize,
    pub skipped: usize,
    pub total: TimingPercentiles,
    pub resolve: TimingPercentiles,
    pub eval: TimingPercentiles,
}

/// One witness row with per-witness eval time and its entry's amortized resolve cost.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WitnessTimingRow {
    pub entry: String,
    pub function: String,
    pub eval_nanos: u128,
    pub resolve_nanos: u128,
    pub total_nanos: u128,
}

pub const DEFAULT_SLOWEST_WITNESS_ATTRIBUTION_N: usize = 15;

pub fn compute_witness_timing_rows(
    summary: &DiscoverySummary,
) -> Result<Vec<WitnessTimingRow>, String> {
    if summary.performance_receipts.len() != summary.witness_outcomes.len() {
        return Err(format!(
            "[attribution] SKIPPED: mismatched vector lengths (performance_receipts={}, witness_outcomes={}) — timings unreliable",
            summary.performance_receipts.len(),
            summary.witness_outcomes.len()
        ));
    }

    let mut entry_resolve_map: HashMap<String, u128> = HashMap::new();
    for receipt in &summary.entry_resolve_receipts {
        entry_resolve_map.insert(receipt.entry.clone(), receipt.resolve_nanos);
    }

    let mut rows: Vec<WitnessTimingRow> = Vec::new();
    for (perf, outcome) in summary
        .performance_receipts
        .iter()
        .zip(summary.witness_outcomes.iter())
    {
        let Some(resolve_nanos) = entry_resolve_map.get(&outcome.entry).copied() else {
            continue;
        };
        let eval_nanos = perf.wall_nanos;
        rows.push(WitnessTimingRow {
            entry: outcome.entry.clone(),
            function: outcome.function.clone(),
            eval_nanos,
            resolve_nanos,
            total_nanos: resolve_nanos + eval_nanos,
        });
    }
    Ok(rows)
}

/// Return the top `n` witnesses ranked by eval time (descending), stable on function name.
pub fn top_n_slowest_witnesses(rows: &[WitnessTimingRow], n: usize) -> Vec<WitnessTimingRow> {
    let mut sorted = rows.to_vec();
    sorted.sort_by(|a, b| {
        b.eval_nanos
            .cmp(&a.eval_nanos)
            .then_with(|| a.function.cmp(&b.function))
            .then_with(|| a.entry.cmp(&b.entry))
    });
    sorted.truncate(n);
    sorted
}

pub fn compute_histogram_data(summary: &DiscoverySummary) -> Result<HistogramData, String> {
    if summary.performance_receipts.len() != summary.witness_outcomes.len() {
        return Err(format!(
            "[histogram] SKIPPED: mismatched vector lengths (performance_receipts={}, witness_outcomes={}) — timings unreliable",
            summary.performance_receipts.len(),
            summary.witness_outcomes.len()
        ));
    }

    let mut entry_resolve_map: HashMap<String, u128> = HashMap::new();
    for receipt in &summary.entry_resolve_receipts {
        entry_resolve_map.insert(receipt.entry.clone(), receipt.resolve_nanos);
    }

    let mut total_times: Vec<u128> = Vec::new();
    let mut resolve_times: Vec<u128> = Vec::new();
    let mut eval_times: Vec<u128> = Vec::new();
    let mut skipped_missing_entry_resolve = 0;

    // performance_receipts and witness_outcomes are both generated in the same discovery pass
    // with matching cardinality and order, so positional matching is stable across discovery runs.
    for (perf, outcome) in summary
        .performance_receipts
        .iter()
        .zip(summary.witness_outcomes.iter())
    {
        let resolve_nanos = match entry_resolve_map.get(&outcome.entry).copied() {
            Some(nanos) => nanos,
            None => {
                skipped_missing_entry_resolve += 1;
                continue;
            }
        };
        let eval_nanos = perf.wall_nanos;
        let total_nanos = resolve_nanos + eval_nanos;

        total_times.push(total_nanos);
        resolve_times.push(resolve_nanos);
        eval_times.push(eval_nanos);
    }

    Ok(HistogramData {
        included: total_times.len(),
        skipped: skipped_missing_entry_resolve,
        total: compute_percentiles(total_times),
        resolve: compute_percentiles(resolve_times),
        eval: compute_percentiles(eval_times),
    })
}

pub const WET_HERMETIC_EQUIVALENCE_WITNESS_ENTRY: &str =
    "dag/test/claim/wet_hermetic_equivalence_witness_test.dag";
pub const WET_HERMETIC_SCAFFOLD_ROSTER_PREFIX_DATA: &str =
    "wet_hermetic_equivalence_representative_prefix";

fn resolve_entry_file_under_roots(source_roots: &[String], entry: &str) -> Result<String, String> {
    let path = Path::new(entry);
    if path.is_file() {
        return Ok(path.to_string_lossy().into_owned());
    }
    for root in source_roots {
        let root_path = Path::new(root);
        let root_name = root_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();
        if !root_name.is_empty() {
            let prefix = format!("{root_name}/");
            if let Some(suffix) = entry.strip_prefix(&prefix) {
                let candidate = root_path.join(suffix);
                if candidate.is_file() {
                    return Ok(candidate.to_string_lossy().into_owned());
                }
            }
        }
        let candidate = root_path.join(entry);
        if candidate.is_file() {
            return Ok(candidate.to_string_lossy().into_owned());
        }
    }
    Err(format!(
        "entry file does not exist or is not a file: {}",
        entry
    ))
}

pub fn wet_hermetic_scaffold_roster_entry_prefix(
    source_roots: &[String],
) -> Result<String, String> {
    let entry =
        resolve_entry_file_under_roots(source_roots, WET_HERMETIC_EQUIVALENCE_WITNESS_ENTRY)?;
    let (graph, source_indices) = resolve_entry_graph_shared(source_roots, &entry)?;
    let sources = load_sources_for_entry(source_roots, &entry)?;
    let entry_source = sources
        .iter()
        .find(|s| s.path == entry || s.path.ends_with(WET_HERMETIC_EQUIVALENCE_WITNESS_ENTRY))
        .ok_or_else(|| format!("{entry}: missing from entry closure"))?;
    let entry_module = extract_module_path(&entry_source.content)
        .ok_or_else(|| format!("{entry}: missing module declaration"))?;
    let typed_module = entry_typed_module(&graph, &source_indices, &entry_module)?;
    let si = Rc::new((*source_indices).clone());
    for item in typed_module.items.iter() {
        if item.body.is_none() {
            continue;
        }
        let decl_name = authored_name_at(si.clone(), item.clone());
        if decl_name != WET_HERMETIC_SCAFFOLD_ROSTER_PREFIX_DATA {
            continue;
        }
        let body = item.body.as_ref().ok_or_else(|| {
            format!("{entry}: data '{WET_HERMETIC_SCAFFOLD_ROSTER_PREFIX_DATA}' missing body")
        })?;
        return literal_string_from_expr(body).ok_or_else(|| {
            format!(
                "{entry}: data '{WET_HERMETIC_SCAFFOLD_ROSTER_PREFIX_DATA}' must be a string literal"
            )
        });
    }
    Err(format!(
        "{entry}: missing data '{WET_HERMETIC_SCAFFOLD_ROSTER_PREFIX_DATA}'"
    ))
}

pub fn is_governed_service_representative_row(row: &DiscoveryRow, prefix: &str) -> bool {
    !prefix.is_empty() && row.entry.contains(prefix)
}

pub fn wet_hermetic_discovery_outcome_divergences(
    wet: &[DiscoveryWitnessOutcome],
    hermetic: &[DiscoveryWitnessOutcome],
) -> Vec<String> {
    let mut divergences = Vec::new();
    if wet.len() != hermetic.len() {
        divergences.push(format!(
            "roster size mismatch: wet={} hermetic={}",
            wet.len(),
            hermetic.len()
        ));
        return divergences;
    }
    for (w, h) in wet.iter().zip(hermetic.iter()) {
        if w.entry != h.entry || w.function != h.function {
            divergences.push(format!(
                "roster order mismatch: wet=({},{}) hermetic=({},{})",
                w.function, w.entry, h.function, h.entry
            ));
            continue;
        }
        if w.outcome != h.outcome {
            divergences.push(format!(
                "{} ({}): wet={:?} hermetic={:?}",
                w.function, w.entry, w.outcome, h.outcome
            ));
        }
    }
    divergences
}

/// Peak resident set from `/proc/self/status` VmHWM (high water mark), in bytes.
pub fn peak_rss_vhwm_bytes() -> Option<u64> {
    let status = std::fs::read_to_string("/proc/self/status").ok()?;
    let line = status.lines().find(|l| l.starts_with("VmHWM"))?;
    let kb: u64 = line.split_whitespace().nth(1)?.parse().ok()?;
    Some(kb.saturating_mul(1024))
}

pub fn floor_discovery_path_excluded(path: &str) -> bool {
    matching_discovery_exclusion_substring(path).is_some()
}

fn matching_discovery_exclusion_substring(path: &str) -> Option<String> {
    witness_exclusion_substrings()
        .iter()
        .find(|sub| path.contains(sub.as_str()))
        .cloned()
}

/// Scan `*_test.dag` witnesses excluded from discovery by `exclude_substrings` and return
/// counted, typed rows for the floor receipt (§5: deferred-and-detected, never silent).
pub fn collect_deferred_discovery_rows(
    source_roots: &[String],
    exclude_substrings: &[String],
) -> Result<Vec<DeferredDiscoveryRow>, String> {
    if exclude_substrings.is_empty() {
        return Ok(Vec::new());
    }
    let facts = build_module_graph_facts_live(source_roots);
    let mut out: Vec<DeferredDiscoveryRow> = Vec::new();
    let mut seen: std::collections::BTreeSet<(String, String)> = std::collections::BTreeSet::new();
    for root in source_roots {
        let mut dag_files: Vec<PathBuf> = Vec::new();
        collect_dag_files_tolerant(Path::new(root), &mut dag_files);
        dag_files.sort();
        for path in dag_files {
            let entry = path.to_string_lossy().into_owned();
            let rel = repo_relative_dag_path(&entry);
            if !rel.ends_with("_test.dag") {
                continue;
            }
            let Some(exclude_reason) = exclude_substrings
                .iter()
                .find(|sub| rel.contains(sub.as_str()))
                .cloned()
            else {
                continue;
            };
            let content = std::fs::read_to_string(&path)
                .map_err(|e| format!("read deferred discovery entry {rel}: {e}"))?;
            let reads_live_tree = reads_live_tree_effective(&rel, &content, &facts)?;
            for (function, _) in scan_test_decl_lines(&content) {
                if seen.insert((rel.clone(), function.clone())) {
                    out.push(DeferredDiscoveryRow {
                        entry: rel.clone(),
                        function,
                        exclude_reason: exclude_reason.clone(),
                        reads_live_tree,
                    });
                }
            }
        }
    }
    out.sort_by(|a, b| {
        a.entry
            .cmp(&b.entry)
            .then_with(|| a.function.cmp(&b.function))
    });
    Ok(out)
}

fn witness_admission_offline_exclusion_substrings() -> Vec<String> {
    static PATTERNS: OnceLock<Vec<String>> = OnceLock::new();
    PATTERNS
        .get_or_init(|| {
            string_list_data_from_ci_layer_roots_source(
                ci_layer_roots_authority_content(),
                WITNESS_ADMISSION_OFFLINE_EXCLUSION_SUBSTRINGS_DATA_NAME,
            )
        })
        .clone()
}

fn witness_admission_fixture_exclusion_substrings() -> Vec<String> {
    static PATTERNS: OnceLock<Vec<String>> = OnceLock::new();
    PATTERNS
        .get_or_init(|| {
            string_list_data_from_ci_layer_roots_source(
                ci_layer_roots_authority_content(),
                WITNESS_ADMISSION_FIXTURE_EXCLUSION_SUBSTRINGS_DATA_NAME,
            )
        })
        .clone()
}

fn path_matches_any_substring(path: &str, subs: &[String]) -> bool {
    subs.iter().any(|sub| path.contains(sub.as_str()))
}

fn witness_admission_manifest_key(entry: &str, function: &str) -> String {
    format!("{entry}::{function}")
}

// 🟡 dissolve-on: witness_admission_explicit_consumer_manifest — replace this hand-rolled
// per-form scan with the `.dag`-authoritative manifest from v2.workflow.witness_admission
// (the module-binding supply-carrier pattern: host consumes emitted manifest rows; tracked in
// the Phase 1 (b) lane). Until then the scan is fail-closed in BOTH directions (§5): every
// occurrence of a recognized row head either parses to a key, is a verified definition or
// non-literal pass-through site, or PANICS with its location — a mis-parse stops the line and
// never silently excuses an orphan; and a consumer expressed in an unrecognized form yields
// NO key, so its deferred row surfaces as a loud orphan rather than being absorbed.
fn witness_admission_entry_function_keys_from_source(
    source_label: &str,
    content: &str,
) -> Vec<String> {
    const WINDOW: usize = 400;
    let mut keys: Vec<String> = Vec::new();
    fn push_pair(keys: &mut Vec<String>, entry: &str, function: &str) {
        let key = witness_admission_manifest_key(entry, function);
        if !keys.iter().any(|k| k == &key) {
            keys.push(key);
        }
    }
    let heads: [(&str, &str); 4] = [
        ("bin_wet(", "entry: String"),
        ("probe_red(", "entry: String"),
        ("self_host_wet_entry(", "entry: String"),
        ("SelfHostWetReceiptBinding {", ""),
    ];
    for (head, def_sig) in heads {
        let mut search_from = 0;
        while let Some(rel) = content[search_from..].find(head) {
            let occ = search_from + rel;
            search_from = occ + head.len();
            let after = &content[search_from..];
            let window = &after[..after.len().min(WINDOW)];
            let trimmed = window.trim_start();
            if !def_sig.is_empty() && trimmed.starts_with(def_sig) {
                continue;
            }
            if def_sig.is_empty() && content[..occ].ends_with("type ") {
                continue;
            }
            let Some(entry_rel) = window.find("entry: \"") else {
                if window.contains("entry: ") {
                    continue;
                }
                panic!(
                    "witness admission: {source_label}: `{head}` at byte {occ} has no \
                     recognizable `entry:` argument in range — refusing; a mis-parse must \
                     stop the line, never excuse an orphan"
                );
            };
            let entry_start = entry_rel + "entry: \"".len();
            let Some((entry, after_entry)) = window[entry_start..].split_once('"') else {
                panic!(
                    "witness admission: {source_label}: unterminated entry literal after \
                     `{head}` at byte {occ} — refusing"
                );
            };
            let marker = after_entry.find("f: \"").map(|p| (p, "f: \"")).or_else(|| {
                after_entry
                    .find("function: \"")
                    .map(|p| (p, "function: \""))
            });
            let Some((fn_pos, fn_marker)) = marker else {
                panic!(
                    "witness admission: {source_label}: `{head}` row for entry {entry:?} at \
                     byte {occ} has no `f:`/`function:` literal in range — refusing"
                );
            };
            let fn_start = fn_pos + fn_marker.len();
            let Some((function, _)) = after_entry[fn_start..].split_once('"') else {
                panic!(
                    "witness admission: {source_label}: unterminated function literal after \
                     `{head}` row for entry {entry:?} at byte {occ} — refusing"
                );
            };
            push_pair(&mut keys, entry, function);
        }
    }
    keys.sort();
    keys
}

fn witness_admission_explicit_consumer_keys() -> Vec<String> {
    static KEYS: OnceLock<Vec<String>> = OnceLock::new();
    KEYS.get_or_init(|| {
        let mut keys = witness_admission_entry_function_keys_from_source(
            "dag/gunbc/ci_layer_roots.dag",
            ci_layer_roots_authority_content(),
        );
        let wet =
            std::fs::read_to_string(workspace_root().join(WET_RECEIPT_ENROLLMENT_AUTHORITY_REL))
                .unwrap_or_else(|e| {
                    panic!(
                        "witness admission: failed to read {}: {e}",
                        WET_RECEIPT_ENROLLMENT_AUTHORITY_REL
                    )
                });
        for key in witness_admission_entry_function_keys_from_source(
            WET_RECEIPT_ENROLLMENT_AUTHORITY_REL,
            &wet,
        ) {
            if !keys.iter().any(|k| k == &key) {
                keys.push(key);
            }
        }
        keys.sort();
        keys
    })
    .clone()
}

/// Phase 0(b): every deferred witness row must name an executing consumer (explicit roster,
/// offline local recipe, or fixture explicit roster). Returns orphans — enrolled, zero consumers.
pub fn collect_unexecuted_deferred_witnesses(
    deferred_rows: &[DeferredDiscoveryRow],
) -> Vec<UnexecutedDeferredWitness> {
    let explicit = witness_admission_explicit_consumer_keys();
    let offline = witness_admission_offline_exclusion_substrings();
    let fixture = witness_admission_fixture_exclusion_substrings();
    let mut orphans = Vec::new();
    for row in deferred_rows {
        let key = witness_admission_manifest_key(&row.entry, &row.function);
        if explicit.iter().any(|k| k == &key) {
            continue;
        }
        if path_matches_any_substring(&row.entry, &offline)
            || path_matches_any_substring(&row.entry, &fixture)
        {
            continue;
        }
        orphans.push(UnexecutedDeferredWitness {
            entry: row.entry.clone(),
            function: row.function.clone(),
        });
    }
    orphans
}

fn refuse_unexecuted_deferred_witnesses(
    orphans: &[UnexecutedDeferredWitness],
) -> Result<(), String> {
    if orphans.is_empty() {
        return Ok(());
    }
    let mut lines: Vec<String> = orphans
        .iter()
        .take(8)
        .map(|o| format!("{} ({})", o.function, o.entry))
        .collect();
    if orphans.len() > 8 {
        lines.push(format!("… and {} more orphan row(s)", orphans.len() - 8));
    }
    Err(format!(
        "WITNESS ADMISSION REFUSAL cause=UnexecutedDeferredWitness count={} — enrolled \
         witness row(s) excluded from discovery name zero executing consumers (Phase 0(b) \
         admission invariant); each excluded row must be on falsifier_self_host_wet, \
         bin_witness_wet, known_red_probe, offline, or fixture explicit roster: {}",
        orphans.len(),
        lines.join("; ")
    ))
}

fn eprintln_deferred_discovery_rows(rows: &[DeferredDiscoveryRow]) {
    if rows.is_empty() {
        return;
    }
    let live = rows.iter().filter(|r| r.reads_live_tree).count();
    let ts = floor_ts();
    eprintln!(
        "{ts} [deferred-discovery] {} witness row(s) excluded from per-PR discovery \
         ({} declare ReadsLiveTree) — counted, not silent; run via long-lane / local recipe",
        rows.len(),
        live
    );
    for row in rows.iter().take(8) {
        eprintln!(
            "{ts} [deferred-discovery]   {} ({}) reason={}",
            row.function, row.entry, row.exclude_reason
        );
    }
    if rows.len() > 8 {
        eprintln!(
            "{ts} [deferred-discovery]   … and {} more deferred row(s)",
            rows.len() - 8
        );
    }
}

/// §5 never-skip tooth: a `ReadsLiveTree` row must never take node-frontier selection skip.
fn refuse_reads_live_tree_selection_skip(
    row: &DiscoveryRow,
    skip_kind: &str,
) -> Result<(), String> {
    if row.reads_live_tree {
        return Err(format!(
            "NEVER-SKIP REFUSAL cause=ReadsLiveTreeSelectionSkip rows=1 — \
             `{skip_kind}` would skip {} ({}) but the entry declares (or fail-closed \
             defaults to) ReadsLiveTree; a live-tree witness must run or refuse loudly, \
             never be silently selected out (§5; masks memo-wedge class defects when skipped)",
            row.function, row.entry
        ));
    }
    Ok(())
}

pub(crate) fn collect_dag_files_tolerant(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if is_cargo_target_output_dir(dir, &path) {
                continue;
            }
            collect_dag_files_tolerant(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("dag") {
            out.push(path);
        }
    }
}

fn scan_test_decl_names(content: &str) -> Vec<String> {
    scan_test_decl_lines(content)
        .into_iter()
        .map(|(name, _line)| name)
        .collect()
}

fn scan_wire_contract_decl_names(content: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix("data ") {
            let name: String = rest
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if name.is_empty() {
                continue;
            }
            let after_name = rest.get(name.len()..).unwrap_or("").trim_start();
            if after_name.starts_with(": CoproductWireContract")
                || after_name.starts_with(": VariantEncoding")
            {
                out.push(name);
            }
        }
    }
    out
}

struct SidecarPlacementRule {
    required_suffix: &'static str,
    decl_description: &'static str,
    scan: fn(&str) -> Vec<String>,
    emit_discovery: bool,
}

const SIDECAR_PLACEMENT_RULES: &[SidecarPlacementRule] = &[
    SidecarPlacementRule {
        required_suffix: "_test.dag",
        decl_description: "`test`-marked decls",
        scan: scan_test_decl_names,
        emit_discovery: true,
    },
    SidecarPlacementRule {
        required_suffix: "_contracts.dag",
        decl_description:
            "wire-contract decls (`CoproductWireContract` and `VariantEncoding` data items)",
        scan: scan_wire_contract_decl_names,
        emit_discovery: false,
    },
];

fn scan_test_decl_lines(content: &str) -> Vec<(String, i64)> {
    let mut out = Vec::new();
    for (i, line) in content.lines().enumerate() {
        let trimmed = line.trim_start();
        let rest = trimmed
            .strip_prefix("test fn ")
            .or_else(|| trimmed.strip_prefix("test data "));
        if let Some(rest) = rest {
            let name: String = rest
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if !name.is_empty() {
                out.push((name, (i + 1) as i64));
            }
        }
    }
    out
}

pub fn check_floor_filename_hygiene(source_roots: &[String]) -> Result<(), String> {
    let mut violations: Vec<String> = Vec::new();
    for root in source_roots {
        let mut dag_files: Vec<PathBuf> = Vec::new();
        collect_dag_files_tolerant(Path::new(root), &mut dag_files);
        for path in dag_files {
            if path
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|name| name.contains("__"))
            {
                violations.push(path.to_string_lossy().into_owned());
            }
        }
    }
    if violations.is_empty() {
        return Ok(());
    }
    violations.sort();
    Err(format!(
        "filename hygiene: `.dag` basenames must not contain `__` (use subdirectories); \
         offending file(s): {}",
        violations.join(", ")
    ))
}

pub fn discover_floor_corpus_rows(
    source_roots: &[String],
    scan_dirs: &[String],
    exclude_substrings: &[String],
) -> Result<Vec<DiscoveryRow>, String> {
    discover_floor_corpus_rows_inner(source_roots, scan_dirs, exclude_substrings, &[])
}

pub fn discover_floor_corpus_rows_scoped(
    source_roots: &[String],
    scan_dirs: &[String],
    exclude_substrings: &[String],
    discovery_scope_dirs: &[String],
) -> Result<Vec<DiscoveryRow>, String> {
    discover_floor_corpus_rows_inner(
        source_roots,
        scan_dirs,
        exclude_substrings,
        discovery_scope_dirs,
    )
}

struct FloorLensHygieneGraph {
    rows: Vec<DiscoveryRow>,
    path_imports: std::collections::HashMap<String, Vec<String>>,
    module_to_path: std::collections::HashMap<String, String>,
    lens_with_justification: std::collections::BTreeSet<String>,
}

fn build_floor_lens_hygiene_graph(
    source_roots: &[String],
    scan_dirs: &[String],
    exclude_substrings: &[String],
    discovery_scope_dirs: &[String],
) -> Result<FloorLensHygieneGraph, String> {
    let excludes: Vec<String> = exclude_substrings.to_vec();
    let mut rows: Vec<DiscoveryRow> = Vec::new();
    let mut seen: std::collections::BTreeSet<(String, String)> = std::collections::BTreeSet::new();
    let mut entry_dispositions: std::collections::BTreeMap<String, bool> =
        std::collections::BTreeMap::new();
    for scan_dir in scan_dirs {
        let discovery = discover_owned_data_decls(source_roots, scan_dir, &excludes)?;
        for rec in discovery.records {
            if let OwnedDataDeclInitializer::BoolWitnessClaim {
                witness_entry,
                witness_function,
            } = rec.initializer
            {
                if witness_entry.is_empty() || witness_function.is_empty() {
                    return Err(format!(
                        "discovered decl '{}' has malformed BoolWitness transport (entry/function)",
                        rec.decl_name
                    ));
                }
                if seen.insert((witness_entry.clone(), witness_function.clone())) {
                    let label = rec
                        .decl_name
                        .strip_prefix("unified_claim_")
                        .unwrap_or(&rec.decl_name)
                        .to_string();
                    let reads_live_tree = match entry_dispositions.get(&witness_entry) {
                        Some(d) => *d,
                        None => {
                            let d = read_entry_live_tree_disposition(&witness_entry)?;
                            entry_dispositions.insert(witness_entry.clone(), d);
                            d
                        }
                    };
                    rows.push(DiscoveryRow {
                        label,
                        entry: witness_entry,
                        function: witness_function,
                        reads_live_tree,
                    });
                }
            }
        }
    }

    let mut sidecar_violations: Vec<Vec<String>> =
        SIDECAR_PLACEMENT_RULES.iter().map(|_| Vec::new()).collect();
    let mut path_imports: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();
    let mut module_to_path: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();
    let mut lens_with_justification: std::collections::BTreeSet<String> =
        std::collections::BTreeSet::new();
    for root in source_roots {
        let mut dag_files: Vec<PathBuf> = Vec::new();
        collect_dag_files_tolerant(Path::new(root), &mut dag_files);
        dag_files.sort();
        for path in dag_files {
            let entry = path.to_string_lossy().into_owned();
            let content = std::fs::read_to_string(&path)
                .map_err(|e| format!("read {}: {e}", path.display()))?;
            let rel = repo_relative_dag_path(&entry);
            if let Some(m) = extract_module_path(&content) {
                if is_top_level_lens_module(&m) && declares_construction_justification(&content) {
                    lens_with_justification.insert(m.clone());
                }
                module_to_path.insert(m, rel.clone());
            }
            path_imports.insert(rel, extract_import_paths(&content));
            if excludes.iter().any(|sub| entry.contains(sub.as_str())) {
                continue;
            }
            if !discovery_scope_dirs.is_empty()
                && !discovery_scope_dirs
                    .iter()
                    .any(|d| entry.contains(d.as_str()))
            {
                continue;
            }
            let rule_decls: Vec<Vec<String>> = SIDECAR_PLACEMENT_RULES
                .iter()
                .map(|rule| (rule.scan)(&content))
                .collect();
            for (i, (rule, names)) in SIDECAR_PLACEMENT_RULES
                .iter()
                .zip(rule_decls.iter())
                .enumerate()
            {
                if !names.is_empty() && !entry.ends_with(rule.required_suffix) {
                    sidecar_violations[i].push(entry.clone());
                }
                if rule.emit_discovery && entry.ends_with(rule.required_suffix) {
                    let reads_live_tree = match entry_dispositions.get(&entry) {
                        Some(d) => *d,
                        None => {
                            let d = parse_entry_live_tree_disposition(&entry, &content)?;
                            entry_dispositions.insert(entry.clone(), d);
                            d
                        }
                    };
                    for name in names {
                        if seen.insert((entry.clone(), name.clone())) {
                            rows.push(DiscoveryRow {
                                label: name.clone(),
                                entry: entry.clone(),
                                function: name.clone(),
                                reads_live_tree,
                            });
                        }
                    }
                }
            }
        }
    }
    for (rule, violations) in SIDECAR_PLACEMENT_RULES
        .iter()
        .zip(sidecar_violations.iter())
    {
        if !violations.is_empty() {
            let mut sorted = violations.clone();
            sorted.sort();
            return Err(format!(
                "{} must live in `*{}` files; found in: {}",
                rule.decl_description,
                rule.required_suffix,
                sorted.join(", ")
            ));
        }
    }
    // Reference-derived reach edges (namespace terminal step), unioned onto the import edges above so
    // a stripped file (no import edges) still reaches its lenses. STRICT set only (Qualified +
    // UniqueBare, dropping AmbiguousBare) so an over-connected graph cannot silently clear a truly-
    // inert lens (DESIGN §5 — no fail-open hygiene). Parsed-tree, not a substring scan, so comments/
    // strings never fabricate a reach. Dedup keeps the BFS set honest.
    // Long-lane discovery exclusions (`test/claim/long/`) must not suppress reference edges here:
    // exclusion only removes a witness from per-PR enrollment; stripped long/ witnesses still wire
    // lens reachability for this hygiene pass (the inert_lens_hygiene witness lives under long/).
    for edge in reference_edges_as_import_facts(
        &reference_resolution_facts(source_roots, source_roots, &[]),
        true,
    ) {
        let importer = repo_relative_dag_path(&edge.path);
        let entry = path_imports.entry(importer).or_default();
        if !entry.contains(&edge.import_module) {
            entry.push(edge.import_module);
        }
    }

    rows.sort_by(|a, b| {
        a.entry
            .cmp(&b.entry)
            .then_with(|| a.function.cmp(&b.function))
    });
    Ok(FloorLensHygieneGraph {
        rows,
        path_imports,
        module_to_path,
        lens_with_justification,
    })
}

fn default_floor_lens_hygiene_excludes() -> Vec<String> {
    witness_exclusion_substrings()
}

/// Floor witness builtin (#5433 sibling to `doc_graph_orphan_count`): unreached top-level
/// `v2.lens.*` module count. Returns `-1` when the corpus walk fails closed.
pub fn inert_lens_unreached_module_count() -> i64 {
    match build_floor_lens_hygiene_graph(
        &default_source_roots(),
        &witness_discovery_scan_dirs(),
        &default_floor_lens_hygiene_excludes(),
        &[],
    ) {
        Ok(graph) => {
            inert_lens_modules(&graph.rows, &graph.path_imports, &graph.module_to_path).len() as i64
        }
        Err(_) => -1,
    }
}

/// Floor witness builtin: declared top-level `v2.lens.*` module count (non-vacuity oracle).
pub fn inert_lens_top_level_module_count() -> i64 {
    match build_floor_lens_hygiene_graph(
        &default_source_roots(),
        &witness_discovery_scan_dirs(),
        &default_floor_lens_hygiene_excludes(),
        &[],
    ) {
        Ok(graph) => graph
            .module_to_path
            .keys()
            .filter(|m| is_top_level_lens_module(m))
            .count() as i64,
        Err(_) => -1,
    }
}

fn discover_floor_corpus_rows_inner(
    source_roots: &[String],
    scan_dirs: &[String],
    exclude_substrings: &[String],
    discovery_scope_dirs: &[String],
) -> Result<Vec<DiscoveryRow>, String> {
    let graph = build_floor_lens_hygiene_graph(
        source_roots,
        scan_dirs,
        exclude_substrings,
        discovery_scope_dirs,
    )?;
    let FloorLensHygieneGraph {
        mut rows,
        path_imports,
        module_to_path,
        lens_with_justification,
    } = graph;
    let facts = build_module_graph_facts_live(source_roots);
    apply_effect_reach_derived_reads_live_tree(&mut rows, &facts);
    let inert = inert_lens_modules(&rows, &path_imports, &module_to_path);
    if !inert.is_empty() {
        return Err(format!(
            "inert-lens hygiene (DESIGN.md §6): {} lens module(s) under `v2.lens.*` are authored \
             but unreached by any discovered floor witness — an inert lens is a lie. Wire each \
             with a discovered fail-closed witness (a `*_test.dag` `test fn`/`test data`, or a \
             scan-dir `unified_claim_*`) or delete it: {}",
            inert.len(),
            inert.join(", ")
        ));
    }
    let unjustified = unjustified_lens_modules(&module_to_path, &lens_with_justification);
    if !unjustified.is_empty() {
        return Err(format!(
            "construction-justification (DESIGN.md §5/§6): {} lens module(s) under `v2.lens.*` do \
             not record a `construction_justification` — before adding a lens you must justify why \
             the bad-state class cannot be made unwritable by construction. Add a `data \
             construction_justification: ConstructionJustification = …` decl (see \
             v2.lens.common.construction_justification) classifying it as WallNow / \
             WallAfterGrounding / RatchetForever: {}",
            unjustified.len(),
            unjustified.join(", ")
        ));
    }
    Ok(rows)
}

fn declares_construction_justification(content: &str) -> bool {
    content.lines().any(|line| {
        let trimmed = line.trim_start();
        trimmed.starts_with("data construction_justification")
            && trimmed.contains("ConstructionJustification")
    })
}

// ITEM 2 (reference grounding): the construction->authority graph witness.
//
// `WallNow.construction` was free-text prose; it is now
// `WallNow { mechanism: ConstructionMechanism, authority: DeclarationRef }`, so "this
// lens chains to a real construction" becomes a WALKABLE graph property: every WallNow
// authority must resolve to a real top-level decl in the corpus. The witness below proves
// that graph is TOTAL and goes RED if any binding dangles.
//
// SCAFFOLD (DESIGN §6): resolution is done HOST-SIDE here (extract authority refs from
// source + the kind-agnostic `extract_top_level_decls` over the resolved module), standing
// in for a not-yet-exposed unified .dag decl-resolution primitive. It keys on identity
// (module_path + decl_name), kind-agnostically — NOT a per-kind union (no fn-index ∪
// type-index fork). Dissolve-on: item (ii) "unified kind-agnostic decl-resolution authority
// exposed to .dag" (coordinator-tracked, resolver/spine lane) — when it lands, this witness
// re-expresses over the .dag primitive and the host-side resolution is deleted.

/// Extract every `authority: DeclarationRef { module_path: "..", decl_name: ".." }` in a
/// source file as `(module_path, decl_name)`. The field name `authority` typed
/// `DeclarationRef` is unique to `WallNow`, so this captures exactly the WallNow authorities.
pub fn wall_now_authority_refs(content: &str) -> Vec<(String, String)> {
    const NEEDLE: &str = "authority: DeclarationRef {";
    let mut out = Vec::new();
    let mut rest = content;
    while let Some(pos) = rest.find(NEEDLE) {
        let after = &rest[pos + NEEDLE.len()..];
        if let (Some(mp), Some(dn)) = (
            quoted_field_value(after, "module_path:"),
            quoted_field_value(after, "decl_name:"),
        ) {
            out.push((mp, dn));
        }
        rest = after;
    }
    out
}

/// The string literal following `<field>` (the next `"..."`), whitespace/newline tolerant.
fn quoted_field_value(s: &str, field: &str) -> Option<String> {
    let start = s.find(field)? + field.len();
    let rest = &s[start..];
    let open = rest.find('"')? + 1;
    let tail = &rest[open..];
    let close = tail.find('"')?;
    Some(tail[..close].to_string())
}

/// Resolve WallNow authorities against the kind-agnostic top-level decl table of their
/// declaring module. Returns the unresolved refs as `(declaring_file, module_path, decl_name)`;
/// empty = the construction->authority graph is TOTAL.
pub fn construction_authority_unresolved(
    module_to_content: &std::collections::HashMap<String, String>,
    authorities: &[(String, String, String)],
) -> Vec<(String, String, String)> {
    authorities
        .iter()
        .filter(|(_, module_path, decl_name)| {
            !module_to_content.get(module_path).is_some_and(|content| {
                extract_top_level_decls(content)
                    .iter()
                    .any(|(name, _)| name == decl_name)
            })
        })
        .cloned()
        .collect()
}

/// Walk the corpus, collect WallNow authorities + the module->source map, and return the
/// unresolved refs (empty = total). The live driver behind the witness test.
pub fn construction_authority_graph_unresolved(
    source_roots: &[String],
) -> Result<Vec<(String, String, String)>, String> {
    let mut module_to_content: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();
    let mut authorities: Vec<(String, String, String)> = Vec::new();
    for root in source_roots {
        let mut dag_files: Vec<PathBuf> = Vec::new();
        collect_dag_files_tolerant(Path::new(root), &mut dag_files);
        dag_files.sort();
        for path in dag_files {
            let content = std::fs::read_to_string(&path)
                .map_err(|e| format!("read {}: {e}", path.display()))?;
            let file = path.to_string_lossy().into_owned();
            for (module_path, decl_name) in wall_now_authority_refs(&content) {
                authorities.push((file.clone(), module_path, decl_name));
            }
            if let Some(m) = extract_module_path(&content) {
                module_to_content.insert(m, content);
            }
        }
    }
    Ok(construction_authority_unresolved(
        &module_to_content,
        &authorities,
    ))
}

fn unjustified_lens_modules(
    module_to_path: &std::collections::HashMap<String, String>,
    justified: &std::collections::BTreeSet<String>,
) -> Vec<String> {
    let mut missing: Vec<String> = module_to_path
        .keys()
        .filter(|m| is_top_level_lens_module(m) && !justified.contains(*m))
        .cloned()
        .collect();
    missing.sort();
    missing.dedup();
    missing
}

fn repo_relative_dag_path(path: &str) -> String {
    let normalized = path.replace('\\', "/");
    let ws = workspace_root();
    let ws_prefix = format!("{}/", ws.to_string_lossy().replace('\\', "/"));
    let stripped = normalized
        .strip_prefix(&ws_prefix)
        .map(|s| s.to_string())
        .unwrap_or(normalized);
    stripped.trim_start_matches("./").to_string()
}

fn is_top_level_lens_module(module: &str) -> bool {
    match module.strip_prefix("v2.lens.") {
        Some(rest) => !rest.is_empty() && !rest.contains('.'),
        None => false,
    }
}

fn inert_lens_modules(
    rows: &[DiscoveryRow],
    path_imports: &std::collections::HashMap<String, Vec<String>>,
    module_to_path: &std::collections::HashMap<String, String>,
) -> Vec<String> {
    let mut reached: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    let mut queue: Vec<String> = Vec::new();
    let path_to_module: std::collections::HashMap<&String, &String> =
        module_to_path.iter().map(|(m, p)| (p, m)).collect();
    // Seed reachability from ALL *_test.dag files found in the source tree (not
    // just enrolled rows), so that witnesses in the execution corpus also count
    // for lens coverage even though they are excluded from the main corpus rows.
    let entry_paths: std::collections::BTreeSet<String> = {
        let mut s: std::collections::BTreeSet<String> = rows
            .iter()
            .map(|r| repo_relative_dag_path(&r.entry))
            .collect();
        for path in path_imports.keys() {
            if path.ends_with("_test.dag") {
                s.insert(path.clone());
            }
        }
        s
    };
    for ep in &entry_paths {
        if let Some(module) = path_to_module.get(ep) {
            if reached.insert((*module).clone()) {
                queue.push((*module).clone());
            }
        }
        if let Some(imports) = path_imports.get(ep) {
            for imp in imports {
                if reached.insert(imp.clone()) {
                    queue.push(imp.clone());
                }
            }
        }
    }
    while let Some(module) = queue.pop() {
        if let Some(mpath) = module_to_path.get(&module) {
            if let Some(imports) = path_imports.get(mpath) {
                for imp in imports {
                    if reached.insert(imp.clone()) {
                        queue.push(imp.clone());
                    }
                }
            }
        }
    }
    let mut inert: Vec<String> = module_to_path
        .keys()
        .filter(|m| is_top_level_lens_module(m) && !reached.contains(*m))
        .cloned()
        .collect();
    inert.sort();
    inert.dedup();
    inert
}

/// Host realization of std.realization_schedule.NodeFrontierSelection (signed design:
/// docs/plans/affected-set-differential-falsifier.md). PredictOnly computes would-skip
/// per row, RECORDS the prediction, and runs the row anyway — the falsifier cadence
/// compares predictions against cold verdicts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeFrontierSelectionMode {
    Off,
    Applied,
    PredictOnly,
}

pub struct DiscoveryCorpusOptions {
    pub node_frontier_selection: NodeFrontierSelectionMode,
    pub explicit_roster_only: bool,
    /// Path-substring exclusion list. Non-plan callers default to
    /// `witness_exclusion_substrings()`; plan-driven paths supply this from
    /// RunnableDiscoveryBatch.exclude_substrings (the model authority).
    pub exclude_substrings: Vec<String>,
    /// When non-empty, scopes the source-root `test fn` tree walk to files under one of these
    /// directories. Import resolution still uses the full source_roots. Empty = full walk.
    pub discovery_scope_dirs: Vec<String>,
    /// Fast-lane per-witness eval budget (operator 5s rule, 2026-07-12). When set, every
    /// discovered witness eval is deadline-armed and an over-budget eval unwinds as the
    /// typed EvalBudgetExceeded runtime error (a FAIL row naming the witness). None = no
    /// bound (the long-lane / local recipe posture).
    pub fast_lane_eval_budget_ms: Option<u64>,
    /// Whole-receipt wall budget for the nightly falsifier Wet self-host lane (emit+cargo).
    pub wet_receipt_wall_budget_ms: Option<u64>,
    /// Secondary interpreter CPU budget for the falsifier Wet self-host lane.
    pub wet_receipt_interp_eval_budget_ms: Option<u64>,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct WitnessBudgetPolicy {
    pub cpu_eval_budget_ms: Option<u64>,
    pub wet_receipt_wall_budget_ms: Option<u64>,
}

impl DiscoveryCorpusOptions {
    pub fn witness_budget_policy(&self) -> WitnessBudgetPolicy {
        WitnessBudgetPolicy {
            cpu_eval_budget_ms: self
                .fast_lane_eval_budget_ms
                .or(self.wet_receipt_interp_eval_budget_ms),
            wet_receipt_wall_budget_ms: self.wet_receipt_wall_budget_ms,
        }
    }
}

impl Default for DiscoveryCorpusOptions {
    fn default() -> Self {
        Self {
            node_frontier_selection: NodeFrontierSelectionMode::Off,
            explicit_roster_only: false,
            exclude_substrings: witness_exclusion_substrings(),
            discovery_scope_dirs: vec![],
            fast_lane_eval_budget_ms: None,
            wet_receipt_wall_budget_ms: None,
            wet_receipt_interp_eval_budget_ms: None,
        }
    }
}

/// How the discovery corpus parallelizes. `Serial` runs every row on the caller's thread —
/// the calibration path that also carries the width-1 closure-drift oracle. `Adaptive`
/// drains entry-groups through a worker pool whose concurrency the memory governor admits:
/// a new worker (= one more whole-tree index resident) is the expensive act the governor
/// gates, replacing the retired plan-pinned `spawn_width` / `spawn_width_cap` constants.
pub enum DiscoveryWidthPolicy {
    Serial,
    Adaptive(std::sync::Arc<crate::memory_governor::MemoryGovernor>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FileLineRange {
    start: i64,
    end: i64,
}

fn floor_git_diff_range() -> Result<String, String> {
    use v1_interpreter::Value;
    let roots = default_source_roots();
    let entry = "src/v2/workflow/floor_diff_observe.dag";
    let (graph, indices) = resolve_entry_graph_shared(&roots, entry)
        .map_err(|e| format!("floor_diff_observe resolve: {e}"))?;
    let ctx = make_eval_context(&graph, indices, v1_interpreter::ExecutionMode::Wet);
    let result =
        v1_interpreter::run_in_context(&ctx, "floor_observe_git_diff_unified_for_ci", false)
            .map_err(|e| format!("floor_observe_git_diff_unified_for_ci: {e}"))?;
    match &result {
        Value::Variant {
            variant_name,
            fields,
            ..
        } if ctx.sym_eq(*variant_name, "UnifiedDiffOk") => match ctx.field(fields, "text") {
            Some(Value::Str(s)) => Ok(s.clone()),
            _ => Err("UnifiedDiffOk missing `text` field".to_string()),
        },
        Value::Variant {
            variant_name,
            fields,
            ..
        } if ctx.sym_eq(*variant_name, "UnifiedDiffFail") => match ctx.field(fields, "reason") {
            Some(Value::Str(r)) => Err(r.clone()),
            _ => Err("git diff observation failed (no reason)".to_string()),
        },
        other => Err(format!(
            "floor_observe_git_diff_unified_for_ci returned `{}`, expected FloorUnifiedDiffResult",
            ctx.format_value(other)
        )),
    }
}

fn string_list_from_value(val: &v1_interpreter::Value, field: &str) -> Result<Vec<String>, String> {
    use v1_interpreter::Value;
    match val {
        Value::List(items) => items
            .iter()
            .map(|v| match v {
                Value::Str(s) => Ok(s.clone()),
                other => Err(format!("{field} entry not a String: `{other:?}`")),
            })
            .collect(),
        other => Err(format!("{field} not a List: `{other:?}`")),
    }
}

fn floor_git_diff_name_status_range() -> Result<(Vec<String>, HashSet<String>), String> {
    use v1_interpreter::Value;
    let roots = default_source_roots();
    let entry = "src/v2/workflow/floor_diff_observe.dag";
    let (graph, indices) = resolve_entry_graph_shared(&roots, entry)
        .map_err(|e| format!("floor_diff_observe resolve: {e}"))?;
    let ctx = make_eval_context(&graph, indices, v1_interpreter::ExecutionMode::Wet);
    let result =
        v1_interpreter::run_in_context(&ctx, "floor_observe_git_diff_name_status_for_ci", false)
            .map_err(|e| format!("floor_observe_git_diff_name_status_for_ci: {e}"))?;
    match &result {
        Value::Variant {
            variant_name,
            fields,
            ..
        } if ctx.sym_eq(*variant_name, "NameStatusDiffOk") => {
            let changed = match ctx.field(fields, "changed_paths") {
                Some(v) => string_list_from_value(v, "changed_paths")?,
                None => return Err("NameStatusDiffOk missing `changed_paths` field".to_string()),
            };
            let departed = match ctx.field(fields, "departed_paths") {
                Some(v) => string_list_from_value(v, "departed_paths")?,
                None => return Err("NameStatusDiffOk missing `departed_paths` field".to_string()),
            };
            Ok((
                changed.iter().map(|p| normalize_repo_path(p)).collect(),
                departed.iter().map(|p| normalize_repo_path(p)).collect(),
            ))
        }
        Value::Variant {
            variant_name,
            fields,
            ..
        } if ctx.sym_eq(*variant_name, "NameStatusDiffFail") => match ctx.field(fields, "reason") {
            Some(Value::Str(r)) => Err(r.clone()),
            _ => Err("git diff --name-status observation failed (no reason)".to_string()),
        },
        other => Err(format!(
            "floor_observe_git_diff_name_status_for_ci returned `{}`, expected FloorNameStatusDiffResult",
            ctx.format_value(other)
        )),
    }
}

fn normalize_repo_path(path: &str) -> String {
    path.strip_prefix("./").unwrap_or(path).replace('\\', "/")
}

fn diff_file_matches_entry(diff_file: &str, entry_path: &str) -> bool {
    let file = normalize_repo_path(diff_file);
    let entry = normalize_repo_path(entry_path);
    file == entry || entry.ends_with(&file) || file.ends_with(&entry)
}

fn parse_unified_diff_line_ranges(diff_text: &str) -> HashMap<String, Vec<FileLineRange>> {
    let mut out: HashMap<String, Vec<FileLineRange>> = HashMap::new();
    let mut current_file: Option<String> = None;
    for line in diff_text.lines() {
        if let Some(rest) = line.strip_prefix("+++ b/") {
            current_file = Some(normalize_repo_path(rest));
        } else if line.starts_with("@@ ") {
            let Some(file) = current_file.clone() else {
                continue;
            };
            let plus = line.split_whitespace().nth(2).unwrap_or("");
            let plus = plus.trim_start_matches('+');
            let (start, count) = if let Some((s, c)) = plus.split_once(',') {
                (s.parse::<i64>().unwrap_or(1), c.parse::<i64>().unwrap_or(1))
            } else {
                (plus.parse::<i64>().unwrap_or(1), 1)
            };
            // Zero-width new range (`+L,0`): the deletion gap sits between L and
            // L+1 — attribute the single following line, mirroring
            // parse_unified_diff_changed_new_lines (anchoring at L false-fired
            // the module-line refusal for import strips under a module header).
            let (start, end) = if count <= 0 {
                (start + 1, start + 1)
            } else {
                (start, start + count - 1)
            };
            out.entry(file)
                .or_default()
                .push(FileLineRange { start, end });
        }
    }
    out
}

fn parse_unified_diff_changed_new_lines(diff_text: &str) -> HashMap<String, HashSet<i64>> {
    let mut out: HashMap<String, HashSet<i64>> = HashMap::new();
    let mut current_file: Option<String> = None;
    let mut new_line: i64 = 0;
    let mut in_hunk = false;
    for line in diff_text.lines() {
        if let Some(rest) = line.strip_prefix("+++ b/") {
            current_file = Some(normalize_repo_path(rest));
            in_hunk = false;
            continue;
        }
        if line.starts_with("@@ ") {
            let plus = line.split_whitespace().nth(2).unwrap_or("");
            let plus = plus.trim_start_matches('+');
            let (anchor, new_len) = if let Some((s, c)) = plus.split_once(',') {
                (s.parse::<i64>().unwrap_or(1), c.parse::<i64>().unwrap_or(1))
            } else {
                (plus.parse::<i64>().unwrap_or(1), 1)
            };
            // A zero-width new range (`+L,0`, deletion-only hunk) anchors at the
            // new-side line BEFORE the gap; the removed content sits between L
            // and L+1, so its new-side attribution is L+1 — the same
            // following-line semantics a with-context hunk produces naturally
            // (the cursor has advanced past the leading context when the `-`
            // rows arrive). Anchoring at L false-fired the module-line (line 1)
            // fail-closed refusal for every import-block strip directly under a
            // `module` header. `+0,0` (the module line itself deleted) still
            // attributes to line 1 and still refuses.
            new_line = if new_len == 0 { anchor + 1 } else { anchor };
            in_hunk = true;
            continue;
        }
        if !in_hunk {
            continue;
        }
        let Some(file) = current_file.clone() else {
            continue;
        };
        if let Some(_add) = line.strip_prefix('+') {
            out.entry(file.clone()).or_default().insert(new_line);
            new_line += 1;
        } else if line.starts_with('-') {
            // Pure deletions advance only the old-file cursor; attribute at the new-file
            // position where the removal occurred (same line for consecutive `-` rows).
            out.entry(file).or_default().insert(new_line);
        } else if line.starts_with(' ') {
            new_line += 1;
        }
    }
    out
}

fn changed_new_lines_for_file(
    changed_new_lines_by_file: &HashMap<String, HashSet<i64>>,
    file_path: &str,
    file_norm: &str,
) -> HashSet<i64> {
    changed_new_lines_by_file
        .get(file_norm)
        .or_else(|| changed_new_lines_by_file.get(file_path))
        .cloned()
        .unwrap_or_default()
}

fn newline_index_for_span<'a>(
    span: &SourceSpan,
    source_indices: &'a HashMap<String, Rc<NewlineIndex>>,
) -> Option<&'a Rc<NewlineIndex>> {
    let file = normalize_repo_path(&span.file);
    source_indices.get(&span.file).or_else(|| {
        source_indices.iter().find_map(|(path, idx)| {
            let norm = normalize_repo_path(path);
            if norm == file || file.ends_with(&norm) || norm.ends_with(&file) {
                Some(idx)
            } else {
                None
            }
        })
    })
}

fn span_file_matches(span_file: &str, target_norm: &str) -> bool {
    let s = normalize_repo_path(span_file);
    s == target_norm || s.ends_with(target_norm) || target_norm.ends_with(&s)
}

fn import_closure_files_from_graph(graph: &v1_compiler_compile::ResolvedGraph) -> HashSet<String> {
    let mut files = HashSet::new();
    for module in graph.modules.iter() {
        for item in module.items.iter() {
            files.insert(normalize_repo_path(&item.span.file));
        }
    }
    files
}

fn touched_file_in_import_closure(touched_file: &str, closure_files: &HashSet<String>) -> bool {
    let norm = normalize_repo_path(touched_file);
    closure_files
        .iter()
        .any(|closure_file| span_file_matches(closure_file, &norm))
}

fn value_is_test_claim(val: &v1_interpreter::Value, ctx: &v1_interpreter::InterpContext) -> bool {
    match val {
        v1_interpreter::Value::Variant { variant_name, .. } => matches!(
            ctx.resolve(*variant_name).as_str(),
            "EqualsClaim"
                | "CompilesClaim"
                | "DiagnosticClaim"
                | "StructuralEqualsClaim"
                | "RoundTripClaim"
        ),
        _ => false,
    }
}

fn variant_field<'a>(
    ctx: &v1_interpreter::InterpContext,
    fields: &'a [(v1_interpreter::Symbol, v1_interpreter::Value)],
    name: &str,
) -> Option<&'a v1_interpreter::Value> {
    fields
        .iter()
        .find(|(sym, _)| ctx.sym_eq(*sym, name))
        .map(|(_, v)| v)
}

/// Node-frontier selection applies only to node-corpus TestClaim rows whose
/// evaluation footprint is structurally Node-valued at runtime. UnifiedTestClaim
/// BoolWitnessClaim rows and CompilesClaim rows whose input/expected_value are
/// Symbol atoms (parse-bridge harness claims) are out of scope for
/// `test_claim_evaluation_touches_rerun_frontier` — skipping them is
/// fail-closed-safe (cannot under-approximate touch → may rerun, never skip).
fn test_claim_selection_has_node_corpus(
    val: &v1_interpreter::Value,
    ctx: &v1_interpreter::InterpContext,
) -> bool {
    let v1_interpreter::Value::Variant {
        variant_name,
        fields,
        ..
    } = val
    else {
        return false;
    };
    let all_nodes = |vals: &[&v1_interpreter::Value]| vals.iter().all(|v| value_is_node(v, ctx));
    match ctx.resolve(*variant_name).as_str() {
        "EqualsClaim" | "StructuralEqualsClaim" => all_nodes(&[
            variant_field(ctx, fields, "lhs").expect("lhs"),
            variant_field(ctx, fields, "rhs").expect("rhs"),
        ]),
        "CompilesClaim" => all_nodes(&[
            variant_field(ctx, fields, "input").expect("input"),
            variant_field(ctx, fields, "expected_value").expect("expected_value"),
        ]),
        "RoundTripClaim" => all_nodes(&[variant_field(ctx, fields, "input").expect("input")]),
        "DiagnosticClaim" => {
            variant_field(ctx, fields, "input").is_some_and(|v| value_is_node(v, ctx))
        }
        _ => false,
    }
}

fn value_is_node(val: &v1_interpreter::Value, ctx: &v1_interpreter::InterpContext) -> bool {
    matches!(
        val,
        v1_interpreter::Value::Record { type_name, .. } if ctx.resolve(*type_name).as_str() == "Node"
    )
}

fn collect_node_values(
    val: &v1_interpreter::Value,
    ctx: &v1_interpreter::InterpContext,
    out: &mut Vec<v1_interpreter::Value>,
) {
    if value_is_node(val, ctx) {
        out.push(val.clone());
    }
    match val {
        v1_interpreter::Value::Record { fields, .. }
        | v1_interpreter::Value::Variant { fields, .. } => {
            for (_, v) in fields.iter() {
                collect_node_values(v, ctx, out);
            }
        }
        v1_interpreter::Value::List(items) => {
            for v in items.iter() {
                collect_node_values(v, ctx, out);
            }
        }
        _ => {}
    }
}

fn call_test_claim_fn_bool(
    ctx: &v1_interpreter::InterpContext,
    fn_name: &str,
    claim: &v1_interpreter::Value,
    frontier: &v1_interpreter::Value,
    claim_param: &str,
) -> Result<Option<bool>, String> {
    if !ctx.item_registry.contains_key(fn_name) {
        return Ok(None);
    }
    let args = [
        (Some(claim_param.to_string()), claim.clone()),
        (Some("frontier".to_string()), frontier.clone()),
    ];
    match v1_interpreter::run_in_context_with_args(ctx, fn_name, &args, false) {
        Ok(v1_interpreter::Value::Bool(b)) => Ok(Some(b)),
        Ok(other) => Err(format!(
            "{} returned `{}`, expected Bool",
            fn_name,
            ctx.format_value(&other)
        )),
        Err(e) => Err(format!("{}: {}", fn_name, e)),
    }
}

fn list_value_from_vec(items: Vec<v1_interpreter::Value>) -> v1_interpreter::Value {
    v1_interpreter::list_value(items)
}

fn decl_span_end_line(sorted_decl_lines: &[i64], decl_line: i64) -> i64 {
    sorted_decl_lines
        .iter()
        .position(|&line| line == decl_line)
        .map(|idx| {
            sorted_decl_lines
                .get(idx + 1)
                .map(|&next| next - 1)
                .unwrap_or(i64::MAX)
        })
        .unwrap_or(i64::MAX)
}

fn collect_sorted_decl_lines_for_file(
    index: &MultiEntryIndex,
    file_path: &str,
) -> Result<Vec<i64>, String> {
    let file_norm = normalize_repo_path(file_path);
    let (graph, source_indices) = resolve_entry_with_index(index, file_path)?;
    let content = std::fs::read_to_string(file_path)
        .map_err(|e| format!("read {file_path} for decl span: {e}"))?;
    let mut decls: Vec<i64> = Vec::new();
    for module in graph.modules.iter() {
        for item in module.items.iter() {
            if !span_file_matches(&item.span.file, &file_norm) {
                continue;
            }
            let Some(nl) = newline_index_for_span(&item.span, &source_indices).cloned() else {
                return Err(format!(
                    "newline index missing for decl span in {file_path}"
                ));
            };
            decls.push(byte_to_line_col(nl, item.span.start).line);
        }
    }
    for (_, line) in scan_test_decl_lines(&content) {
        if !decls.contains(&line) {
            decls.push(line);
        }
    }
    decls.sort_unstable();
    Ok(decls)
}

// SCAFFOLD (DESIGN §6–§7): host-side diff→declaration attribution
// (`floor_diff_edits_from_line_ranges`) and per-entry frontier materialization
// (`rerun_frontier_nodes_for_entry`, `entry_touches_rerun_frontier`) remain host
// realization until provenance ingest lands. Skip/precompute **verdicts** read `.dag`
// via `floor_kernel_would_skip` / `floor_kernel_precompute_would_skip`; the
// `entry_file_touched` axis is decided by `entry_file_touched_via_import_closure`
// (module-graph import-closure grain over `facts.adjacency` — the `.dag` authority is
// `v2.lens.module_graph.entry_affected_by_touched_paths` and the module-grain receipt
// harness certifies the pair by execution; declared interim, dissolves at the
// namespace-only terminal step where the grain re-decides — see
// `entry_file_touched_grain_interim` in `v2.lens.affected_set.entry_selection`).
// Dissolve-on: `affected_set_reading_from_git_diff_provenance` + floor-runtime provenance ingest
// expose edit-locus → delete `floor_diff_edits_from_line_ranges`, `rerun_frontier_nodes_for_entry`,
// `entry_touches_rerun_frontier`, and the inline floor-runner `resolve_entry_with_index` (census:
// `rg 'floor_diff_edits_from_line_ranges|rerun_frontier_nodes_for_entry' src/v1/stage0/src/cli_run.rs`
// must be empty).
//
// Host-side diff→declaration attribution only (line-range I/O). Skip verdicts live in
// `v2.workflow.affected_set_floor_runner` — the executor reads `.dag`, never recomputes frontier.
#[derive(Clone, Debug, Default)]
struct FloorDiffEdits {
    overlapping_data_items: HashSet<(String, String)>,
    edited_test_fns: HashSet<(String, String)>,
    /// `.dag` files with a non-data, non-test-fn declaration touched — run that entry's roster.
    touched_entry_files: HashSet<String>,
}

const FLOOR_RUNNER_ENTRY: &str = "src/v2/workflow/affected_set_floor_runner.dag";
const MODULE_GRAPH_ENTRY: &str = "src/v2/lens/module_graph.dag";

// `entry_file_touched_via_dependency_view` (the fn-arrow DependencyView wrapper) was
// deleted here 2026-07-10 (operator fork (c)): its substrate-not-whole-tree arm returned
// `Ok(true)` — a silent widen that marked every row entry-file-touched whenever any entry
// file was touched, while its comment called that arm fail-closed. The channel's decision
// now lives in `entry_file_touched_via_import_closure` (module-graph import-closure grain,
// typed refusal on a facts gap). `call_entry_affected_by_dependency_view` was deleted
// 2026-07-16 when its last consumer (compile-clean scoping) regrounded onto the same
// import-closure realization; the fn-arrow chain stays in `.dag`
// (`v2.lens.affected_set.entry_selection`) as the decl-level candidate for when the
// namespace-only terminal step re-decides the grain.

fn call_entry_affected_by_touched_paths(
    ctx: &v1_interpreter::InterpContext,
    entry_path: &str,
    pool_roots: &[String],
    touched_paths: &[String],
) -> Result<bool, String> {
    if !ctx
        .item_registry
        .contains_key("entry_affected_by_touched_paths")
    {
        return Err(
            "entry_affected_by_touched_paths missing from module_graph context".to_string(),
        );
    }
    let roots: Vec<v1_interpreter::Value> = pool_roots
        .iter()
        .map(|s| v1_interpreter::Value::Str(s.clone()))
        .collect();
    let touched: Vec<v1_interpreter::Value> = touched_paths
        .iter()
        .map(|s| v1_interpreter::Value::Str(s.clone()))
        .collect();
    let args = [
        (
            Some("entry_path".to_string()),
            v1_interpreter::Value::Str(entry_path.to_string()),
        ),
        (Some("pool_roots".to_string()), list_value_from_vec(roots)),
        (
            Some("touched_paths".to_string()),
            list_value_from_vec(touched),
        ),
    ];
    match v1_interpreter::run_in_context_with_args(
        ctx,
        "entry_affected_by_touched_paths",
        &args,
        false,
    ) {
        Ok(v1_interpreter::Value::Bool(b)) => Ok(b),
        Ok(other) => Err(format!(
            "entry_affected_by_touched_paths returned `{}`, expected Bool",
            ctx.format_value(&other)
        )),
        Err(e) => Err(format!("entry_affected_by_touched_paths: {e}")),
    }
}

fn call_floor_kernel_would_skip(
    ctx: &v1_interpreter::InterpContext,
    changed_paths: &[String],
    frontier_nodes: &[v1_interpreter::Value],
    touches_frontier: bool,
    function_edited: bool,
    entry_file_touched: bool,
    runtime_data_dependency_touched: bool,
) -> Result<bool, String> {
    if !ctx.item_registry.contains_key("floor_kernel_would_skip") {
        return Err("floor_kernel_would_skip missing from floor runner context".to_string());
    }
    let paths: Vec<v1_interpreter::Value> = changed_paths
        .iter()
        .map(|s| v1_interpreter::Value::Str(s.clone()))
        .collect();
    let args = [
        (
            Some("changed_paths".to_string()),
            list_value_from_vec(paths),
        ),
        (
            Some("frontier_nodes".to_string()),
            list_value_from_vec(frontier_nodes.to_vec()),
        ),
        (
            Some("touches_frontier".to_string()),
            v1_interpreter::Value::Bool(touches_frontier),
        ),
        (
            Some("function_edited".to_string()),
            v1_interpreter::Value::Bool(function_edited),
        ),
        (
            Some("entry_file_touched".to_string()),
            v1_interpreter::Value::Bool(entry_file_touched),
        ),
        (
            Some("runtime_data_dependency_touched".to_string()),
            v1_interpreter::Value::Bool(runtime_data_dependency_touched),
        ),
    ];
    match v1_interpreter::run_in_context_with_args(ctx, "floor_kernel_would_skip", &args, false) {
        Ok(v1_interpreter::Value::Bool(b)) => Ok(b),
        Ok(other) => Err(format!(
            "floor_kernel_would_skip returned `{}`, expected Bool",
            ctx.format_value(&other)
        )),
        Err(e) => Err(format!("floor_kernel_would_skip: {e}")),
    }
}

fn live_tree_disposition_value(
    ctx: &v1_interpreter::InterpContext,
    reads_live_tree: bool,
) -> v1_interpreter::Value {
    v1_interpreter::Value::Variant {
        type_name: ctx.sym("LiveTreeDisposition"),
        variant_name: ctx.sym(if reads_live_tree {
            "ReadsLiveTree"
        } else {
            "SubstrateInputsOnly"
        }),
        fields: Rc::new(Vec::new()),
    }
}

fn call_floor_row_would_skip(
    ctx: &v1_interpreter::InterpContext,
    reads_live_tree: bool,
    changed_paths: &[String],
    frontier_nodes: &[v1_interpreter::Value],
    touches_frontier: bool,
    function_edited: bool,
    entry_file_touched: bool,
    runtime_data_dependency_touched: bool,
) -> Result<bool, String> {
    if !ctx.item_registry.contains_key("floor_row_would_skip") {
        return Err("floor_row_would_skip missing from floor runner context".to_string());
    }
    let paths: Vec<v1_interpreter::Value> = changed_paths
        .iter()
        .map(|s| v1_interpreter::Value::Str(s.clone()))
        .collect();
    let args = [
        (
            Some("reads_live_tree".to_string()),
            live_tree_disposition_value(ctx, reads_live_tree),
        ),
        (
            Some("changed_paths".to_string()),
            list_value_from_vec(paths),
        ),
        (
            Some("frontier_nodes".to_string()),
            list_value_from_vec(frontier_nodes.to_vec()),
        ),
        (
            Some("touches_frontier".to_string()),
            v1_interpreter::Value::Bool(touches_frontier),
        ),
        (
            Some("function_edited".to_string()),
            v1_interpreter::Value::Bool(function_edited),
        ),
        (
            Some("entry_file_touched".to_string()),
            v1_interpreter::Value::Bool(entry_file_touched),
        ),
        (
            Some("runtime_data_dependency_touched".to_string()),
            v1_interpreter::Value::Bool(runtime_data_dependency_touched),
        ),
    ];
    match v1_interpreter::run_in_context_with_args(ctx, "floor_row_would_skip", &args, false) {
        Ok(v1_interpreter::Value::Bool(b)) => Ok(b),
        Ok(other) => Err(format!(
            "floor_row_would_skip returned `{}`, expected Bool",
            ctx.format_value(&other)
        )),
        Err(e) => Err(format!("floor_row_would_skip: {e}")),
    }
}

fn call_floor_row_precompute_would_skip(
    ctx: &v1_interpreter::InterpContext,
    live_row_count: usize,
    changed_paths: &[String],
    frontier_node_count: usize,
    edited_test_fn_count: usize,
    touched_entry_file_count: usize,
    touched_runtime_dependency_entry_count: usize,
) -> Result<bool, String> {
    if !ctx
        .item_registry
        .contains_key("floor_row_precompute_would_skip")
    {
        return Err(
            "floor_row_precompute_would_skip missing from floor runner context".to_string(),
        );
    }
    let paths: Vec<v1_interpreter::Value> = changed_paths
        .iter()
        .map(|s| v1_interpreter::Value::Str(s.clone()))
        .collect();
    let args = [
        (
            Some("live_row_count".to_string()),
            v1_interpreter::Value::Int(live_row_count as i64),
        ),
        (
            Some("changed_paths".to_string()),
            list_value_from_vec(paths),
        ),
        (
            Some("frontier_node_count".to_string()),
            v1_interpreter::Value::Int(frontier_node_count as i64),
        ),
        (
            Some("edited_test_fn_count".to_string()),
            v1_interpreter::Value::Int(edited_test_fn_count as i64),
        ),
        (
            Some("touched_entry_file_count".to_string()),
            v1_interpreter::Value::Int(touched_entry_file_count as i64),
        ),
        (
            Some("touched_runtime_dependency_entry_count".to_string()),
            v1_interpreter::Value::Int(touched_runtime_dependency_entry_count as i64),
        ),
    ];
    match v1_interpreter::run_in_context_with_args(
        ctx,
        "floor_row_precompute_would_skip",
        &args,
        false,
    ) {
        Ok(v1_interpreter::Value::Bool(b)) => Ok(b),
        Ok(other) => Err(format!(
            "floor_row_precompute_would_skip returned `{}`, expected Bool",
            ctx.format_value(&other)
        )),
        Err(e) => Err(format!("floor_row_precompute_would_skip: {e}")),
    }
}

// TOMBSTONE (ROADMAP `2-host-scaffold-classifier-defork`, resolved 2026-07-11): the
// Rust entry-text classifier (`witness_test_fn_uses_live_host_scan` +
// `entry_text_indicates_live_host_scan` + the `floor:host_scaffold` marker) lived here.
// It is dissolved into the substrate-declared `reads_live_tree` disposition: each
// witness entry file declares `data live_tree_disposition: LiveTreeDisposition = ...`
// (`v2.std.live_tree`); undeclared = ReadsLiveTree = never predict-skip (fail-closed).
// Declaration grade also closes the former cross-file deficit that was documented here
// (a live read hidden behind an import was invisible to entry-text scanning): the
// entry's own row asserts the fact for the whole evaluation, wherever the read hides.
// A row that DECLARES SubstrateInputsOnly while actually reading live state is not
// re-checked by any text scan — the nightly affected-set falsifier (predict-only cold
// run) is the enforcement; a lying row surfaces as a counted divergence within one
// cadence window. Call-reachability-grade classification (fn-arrow DependencyView over
// lowered bodies) remains the later lane that re-derives these declarations.
fn parse_entry_live_tree_disposition(entry: &str, content: &str) -> Result<bool, String> {
    let mut declared: Option<bool> = None;
    for line in content.lines() {
        let trimmed = line.trim_start();
        let Some(rest) = trimmed.strip_prefix("data live_tree_disposition") else {
            continue;
        };
        // Word boundary: a sibling row like `data live_tree_disposition_note: String`
        // is a different declaration, not a malformed disposition row.
        if rest
            .chars()
            .next()
            .is_some_and(|c| c.is_alphanumeric() || c == '_')
        {
            continue;
        }
        let malformed = |detail: &str| {
            format!(
                "entry {entry} declares a malformed `live_tree_disposition` row ({detail}); \
                 expected `data live_tree_disposition: LiveTreeDisposition = ReadsLiveTree | \
                 SubstrateInputsOnly` — no silent reclassification"
            )
        };
        let rest = rest.trim_start();
        let Some(rest) = rest.strip_prefix(':') else {
            return Err(malformed("missing `:` after the declaration name"));
        };
        let rest = rest.trim_start();
        // Qualification-invariant (namespace lane): the annotation may be the bare
        // authority name or its qualified projection (v2.std.live_tree.LiveTreeDisposition);
        // compare the last dot-segment, mirroring type_name_compatible's mixed-spelling
        // rule. The typechecked roster compile remains the authority behind this text scan.
        let (annotation, rest) = match rest.find(|c: char| c.is_whitespace() || c == '=') {
            Some(i) => (&rest[..i], &rest[i..]),
            None => (rest, ""),
        };
        if annotation.rsplit('.').next() != Some("LiveTreeDisposition") {
            return Err(malformed("type annotation is not `LiveTreeDisposition`"));
        }
        let rest = rest.trim_start();
        let Some(rest) = rest.strip_prefix('=') else {
            return Err(malformed("missing `=` initializer"));
        };
        let live = match rest.trim().rsplit('.').next().unwrap_or("") {
            "ReadsLiveTree" => true,
            "SubstrateInputsOnly" => false,
            _ => {
                let other = rest.trim();
                return Err(malformed(&format!("unknown variant `{other}`")));
            }
        };
        if declared.is_some() {
            return Err(malformed("declared more than once in one entry"));
        }
        declared = Some(live);
    }
    // Undeclared = ReadsLiveTree: a row must DECLARE it does not read the live
    // tree to become selection-eligible (fail-closed).
    Ok(declared.unwrap_or(true))
}

fn read_entry_live_tree_disposition(entry: &str) -> Result<bool, String> {
    let content = std::fs::read_to_string(entry).map_err(|e| {
        format!(
            "failed to read entry {entry} for live-tree disposition: {e} — a \
             discovered roster row's file must be readable; no silent reclassification"
        )
    })?;
    parse_entry_live_tree_disposition(entry, &content)
}

// SCAFFOLD (§7 HAND-RUST — `cli_run_effect_reach_inference_bridge`):
// Lane: module-identity-storage-binding Phase 0 — host-fed derived `ReadsLiveTree` and
// path-literal touch evidence routing floor discovery admission until typed `SourceRef`
// at host boundaries makes bare-string file dependencies unwritable.
// Unblock: discovery admission consumes `v2.lens.effect_reach` classification directly
// (same dissolution as the `.dag` lens `WallAfterGrounding{ dissolves_to: SingleAuthority }`).
// DELETE WHEN dissolved: `reads_live_tree_effective`, `apply_effect_reach_derived_reads_live_tree`,
// `effect_reach_touched_via_path_literals`, `effect_reach_derived_reads_live_tree_for_entry`,
// and `EFFECT_REACH_HOST_SINK_MARKERS` (~150 LOC).
// Receipt: `rg cli_run_effect_reach_inference_bridge src/v1/stage0/src/cli_run.rs` == 1 until
// deletion; drift gate `effect_reach_host_sink_markers_v0_is_synced_with_dag_authority`.
pub(crate) const CLI_RUN_EFFECT_REACH_INFERENCE_BRIDGE_SCAFFOLD_MARKER: &str =
    "cli_run_effect_reach_inference_bridge";

/// Receipted Rust mirror of `v2.std.effect_reach` `effect_reach_host_sink_callee_symbols_v0`
/// (`src/v2/std/effect_reach.dag`) — host-sink callee symbols for the derived census.
/// Under-approximating this list lets `reads_live_tree` stay false and discovery skip when
/// the `.dag` lens would classify host-reading (§5 fail-open on the skip axis); the drift gate
/// below evaluates the `.dag` authority through a real interpreter context.
///
/// INTERIM hand-Rust scaffold (`CLI_RUN_EFFECT_REACH_INFERENCE_BRIDGE_SCAFFOLD_MARKER` / §7):
/// dissolves when typed `SourceRef` at host boundaries deletes this bridge.
const EFFECT_REACH_HOST_SINK_MARKERS: &[&str] = &[
    "Read",
    "Filesystem.Read",
    "WitnessBin.Run",
    "gunbc.WitnessBin.Run",
    "Run",
    "shell.Exec.Run",
    "Exec.Run",
];

fn source_has_path_like_string_data(content: &str) -> bool {
    content.lines().any(source_line_has_path_like_string_data)
}

fn source_line_has_path_like_string_data(line: &str) -> bool {
    let trimmed = line.trim();
    if !trimmed.starts_with("data ") {
        return false;
    }
    let value = trimmed
        .split_once("String = \"")
        .or_else(|| trimmed.split_once("String=\""))
        .and_then(|(_, rest)| rest.strip_suffix('"'));
    let Some(value) = value else {
        return false;
    };
    // Pure storage path only — prose/doc values that mention a path must not classify.
    !value.contains(' ')
        && (value.starts_with("src/") || value.starts_with("dag/"))
        && value.contains(".dag")
}

fn source_data_path_literal_touches(content: &str, touched: &str) -> bool {
    if touched.is_empty() {
        return false;
    }
    content
        .lines()
        .any(|line| source_line_has_path_like_string_data(line) && line.contains(touched))
}

fn source_has_host_effect_sink(content: &str) -> bool {
    EFFECT_REACH_HOST_SINK_MARKERS
        .iter()
        .any(|marker| content_contains_host_sink_marker(content, marker))
}

fn content_contains_host_sink_marker(content: &str, marker: &str) -> bool {
    if marker.contains('.') {
        return content.contains(marker);
    }
    // Bare callee tokens (`Read`, `Run`): match call-site shapes only — naive
    // `contains("Read")` false-positives on `ReadsLiveTree` disposition imports.
    content.lines().any(|line| {
        let line = line.trim();
        line.contains(&format!(".{marker}("))
            || line.contains(&format!(" {marker}("))
            || line.contains(&format!("({marker}("))
    })
}

fn import_closure_repo_paths_for_entry(
    entry_path: &str,
    facts: &ModuleGraphFactsLive,
) -> HashSet<String> {
    import_closure_live_paths_with_facts(entry_path, facts)
        .into_iter()
        .map(|p| workspace_relative_repo_path(&p))
        .collect()
}

fn effect_reach_derived_reads_live_tree_for_closure_paths(closure_paths: &HashSet<String>) -> bool {
    let mut has_path_data = false;
    let mut has_sink = false;
    for rel in closure_paths {
        let path = if std::path::Path::new(rel).is_absolute() {
            rel.clone()
        } else {
            workspace_root().join(rel).to_string_lossy().into_owned()
        };
        let Ok(content) = std::fs::read_to_string(&path) else {
            continue;
        };
        if source_has_path_like_string_data(&content) {
            has_path_data = true;
        }
        if source_has_host_effect_sink(&content) {
            has_sink = true;
        }
        if has_path_data && has_sink {
            return true;
        }
    }
    false
}

pub(crate) fn effect_reach_derived_reads_live_tree_for_entry(
    entry_path: &str,
    facts: &ModuleGraphFactsLive,
) -> bool {
    let closure_paths = import_closure_repo_paths_for_entry(entry_path, facts);
    effect_reach_derived_reads_live_tree_for_closure_paths(&closure_paths)
}

fn reads_live_tree_effective(
    entry_path: &str,
    content: &str,
    facts: &ModuleGraphFactsLive,
) -> Result<bool, String> {
    let declared = parse_entry_live_tree_disposition(entry_path, content)?;
    if declared {
        return Ok(true);
    }
    Ok(effect_reach_derived_reads_live_tree_for_entry(
        entry_path, facts,
    ))
}

fn effect_reach_touched_via_path_literals(
    entry_path: &str,
    facts: &ModuleGraphFactsLive,
    touched_paths: &[String],
) -> bool {
    if touched_paths.is_empty() {
        return false;
    }
    let closure_paths = import_closure_repo_paths_for_entry(entry_path, facts);
    for rel in closure_paths {
        let path = if std::path::Path::new(&rel).is_absolute() {
            rel.clone()
        } else {
            workspace_root().join(&rel).to_string_lossy().into_owned()
        };
        let Ok(content) = std::fs::read_to_string(&path) else {
            continue;
        };
        if touched_paths
            .iter()
            .any(|touched| source_data_path_literal_touches(&content, touched.as_str()))
        {
            return true;
        }
    }
    false
}

// SCAFFOLD (§7 HAND-RUST — `cli_run_declared_source_ref_selection_bridge`):
// Lane: declared-source-ref selection (docs/plans/declared-source-ref-selection-design.md
// task 5) — host-fed declared-ref touch axis for floor discovery admission until discovery
// admission consumes `v2.lens.affected_set.declared_source_ref_selection` directly (same
// dissolution posture as sibling bridges: `.dag` authority via interpreter or emitted host
// dispatch once witness-realization host-effect emission lands).
// Unblock: skip-before-resolve and resolve paths evaluate the modeled selection axis instead
// of re-parsing `data declared_source_refs` rows from import-closure text.
// DELETE WHEN dissolved: `declared_source_refs_axis_for_entry`, `declared_source_refs_axis_for_paths`,
// `declared_source_ref_paths_for_entry`, `collect_declared_source_ref_paths_for_closure`,
// `parse_declared_source_ref_paths_from_content`, `parse_named_source_ref_paths`,
// `parse_source_ref_storage_path_from_rhs`, `declared_source_ref_storage_resolves`,
// `storage_path_to_module_index`, `declared_source_refs_blocks_skip`,
// `entry_has_declared_source_refs`, `DeclaredSourceRefAxis`, and
// `CLI_RUN_DECLARED_SOURCE_REF_SELECTION_BRIDGE_MARKER` (~240 LOC).
// Receipt: `rg cli_run_declared_source_ref_selection_bridge src/v1/stage0/src/cli_run.rs` == 1 until
// deletion; drift gate `declared_source_ref_selection_bridge_scaffold_marker_is_declared`.
pub(crate) const CLI_RUN_DECLARED_SOURCE_REF_SELECTION_BRIDGE_MARKER: &str =
    "cli_run_declared_source_ref_selection_bridge";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DeclaredSourceRefAxis {
    Absent,
    Unresolved,
    Touched,
    Untouched,
}

fn parse_source_ref_storage_path_from_rhs(rhs: &str) -> Option<String> {
    if let Some(rest) = rhs.split_once("path:") {
        let rest = rest.1.trim_start();
        if let Some(rest) = rest.strip_prefix('"') {
            if let Some((path, _)) = rest.split_once('"') {
                if (path.starts_with("src/") || path.starts_with("dag/")) && !path.contains(' ') {
                    return Some(path.to_string());
                }
            }
        }
    }
    None
}

fn parse_named_source_ref_paths(content: &str) -> HashMap<String, String> {
    let mut out = HashMap::new();
    for line in content.lines() {
        let trimmed = line.trim();
        let Some(rest) = trimmed.strip_prefix("data ") else {
            continue;
        };
        let Some((name, rhs)) = rest.split_once(':') else {
            continue;
        };
        if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            continue;
        }
        let rhs = rhs.trim_start();
        if !rhs.starts_with("SourceRef") && !rhs.contains("source_ref_for_storage_path") {
            continue;
        }
        let rhs = rhs.split_once('=').map(|(_, v)| v.trim()).unwrap_or(rhs);
        if let Some(path) = parse_source_ref_storage_path_from_rhs(rhs) {
            out.insert(name.to_string(), path);
        }
    }
    out
}

fn parse_declared_source_ref_paths_from_content(
    content: &str,
    named: &HashMap<String, String>,
) -> Option<Vec<String>> {
    let mut in_list = false;
    let mut list_depth = 0i32;
    let mut list_body = String::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if !in_list {
            if trimmed.starts_with("data declared_source_refs") {
                if let Some(open) = trimmed.find('[') {
                    in_list = true;
                    list_depth = 1;
                    list_body.push_str(&trimmed[open + 1..]);
                    list_body.push('\n');
                    if trimmed.contains(']') {
                        break;
                    }
                }
            }
            continue;
        }
        for ch in trimmed.chars() {
            match ch {
                '[' => list_depth += 1,
                ']' => {
                    list_depth -= 1;
                    if list_depth == 0 {
                        break;
                    }
                }
                _ => {}
            }
        }
        if list_depth <= 0 {
            if let Some(close) = trimmed.rfind(']') {
                list_body.push_str(&trimmed[..close]);
            }
            break;
        }
        list_body.push_str(trimmed);
        list_body.push('\n');
    }
    if list_body.is_empty() {
        return None;
    }
    let mut paths = Vec::new();
    for segment in list_body.split(',') {
        let segment = segment.trim();
        if segment.is_empty() {
            continue;
        }
        if let Some(path) = parse_source_ref_storage_path_from_rhs(segment) {
            paths.push(path);
            continue;
        }
        let name = segment
            .trim_end_matches(',')
            .split_whitespace()
            .next()
            .unwrap_or("");
        if let Some(path) = named.get(name) {
            paths.push(path.clone());
        }
    }
    if paths.is_empty() {
        None
    } else {
        Some(paths)
    }
}

fn collect_declared_source_ref_paths_for_closure(closure_paths: &HashSet<String>) -> Vec<String> {
    let mut paths = Vec::new();
    for rel in closure_paths {
        let path = if std::path::Path::new(rel).is_absolute() {
            rel.clone()
        } else {
            workspace_root().join(rel).to_string_lossy().into_owned()
        };
        let Ok(content) = std::fs::read_to_string(&path) else {
            continue;
        };
        let named = parse_named_source_ref_paths(&content);
        if let Some(mut declared) = parse_declared_source_ref_paths_from_content(&content, &named) {
            paths.append(&mut declared);
        }
    }
    paths.sort();
    paths.dedup();
    paths
}

fn declared_source_ref_paths_for_entry(
    entry_path: &str,
    facts: &ModuleGraphFactsLive,
) -> Vec<String> {
    let closure_paths: HashSet<String> = import_closure_repo_paths_for_entry(entry_path, facts);
    collect_declared_source_ref_paths_for_closure(&closure_paths)
}

fn declared_source_ref_storage_resolves(
    path: &str,
    path_to_module: &HashMap<String, String>,
    source_roots: &[String],
) -> bool {
    if path_to_module.contains_key(path) {
        return true;
    }
    let ws = workspace_root();
    for root in source_roots {
        let anchored = anchor_source_root(root);
        if Path::new(&anchored).join(path).is_file() {
            return true;
        }
    }
    ws.join(path).is_file()
}

fn declared_source_refs_axis_for_paths(
    declared_paths: &[String],
    path_to_module: &HashMap<String, String>,
    source_roots: &[String],
    touched_paths: &[String],
) -> DeclaredSourceRefAxis {
    if declared_paths.is_empty() {
        return DeclaredSourceRefAxis::Absent;
    }
    for path in declared_paths {
        if !declared_source_ref_storage_resolves(path, path_to_module, source_roots) {
            return DeclaredSourceRefAxis::Unresolved;
        }
    }
    if touched_paths.is_empty() {
        return DeclaredSourceRefAxis::Untouched;
    }
    for declared in declared_paths {
        if touched_paths
            .iter()
            .any(|touched| repo_paths_match_touched(declared, touched))
        {
            return DeclaredSourceRefAxis::Touched;
        }
    }
    DeclaredSourceRefAxis::Untouched
}

fn path_to_module_from_declaration_facts(
    nodes: &[ModuleDeclarationFactRaw],
) -> HashMap<String, String> {
    nodes
        .iter()
        .map(|n| (workspace_relative_repo_path(&n.path), n.module.clone()))
        .collect()
}

pub(crate) fn declared_source_refs_axis_for_entry(
    entry_path: &str,
    facts: &ModuleGraphFactsLive,
    source_roots: &[String],
    touched_paths: &[String],
) -> DeclaredSourceRefAxis {
    let declared_paths = declared_source_ref_paths_for_entry(entry_path, facts);
    let path_to_module = path_to_module_from_declaration_facts(&facts.nodes);
    declared_source_refs_axis_for_paths(
        &declared_paths,
        &path_to_module,
        source_roots,
        touched_paths,
    )
}

fn declared_source_refs_blocks_skip(axis: DeclaredSourceRefAxis) -> bool {
    matches!(
        axis,
        DeclaredSourceRefAxis::Unresolved | DeclaredSourceRefAxis::Touched
    )
}

fn entry_has_declared_source_refs(entry_path: &str, facts: &ModuleGraphFactsLive) -> bool {
    !declared_source_ref_paths_for_entry(entry_path, facts).is_empty()
}

fn discovery_rows_live_tree_count(rows: &[DiscoveryRow]) -> usize {
    rows.iter().filter(|r| r.reads_live_tree).count()
}

fn apply_effect_reach_derived_reads_live_tree(
    rows: &mut [DiscoveryRow],
    facts: &ModuleGraphFactsLive,
) {
    for row in rows.iter_mut() {
        if entry_has_declared_source_refs(&row.entry, facts) {
            continue;
        }
        if !row.reads_live_tree && effect_reach_derived_reads_live_tree_for_entry(&row.entry, facts)
        {
            row.reads_live_tree = true;
        }
    }
}

#[cfg(test)]
mod effect_reach_host_sink_markers_drift_gate_tests {
    use super::{
        build_multi_entry_index, make_eval_context, resolve_entry_with_index_for_discovery_corpus,
        workspace_root, EFFECT_REACH_HOST_SINK_MARKERS,
    };
    use crate::v1_interpreter::{self, ExecutionMode, Value};
    use std::collections::HashSet;

    const EFFECT_REACH_STD_ENTRY: &str = "src/v2/std/effect_reach.dag";

    fn dag_host_sink_callee_symbols() -> HashSet<String> {
        std::env::set_current_dir(workspace_root()).expect("chdir workspace");
        let index = build_multi_entry_index(&["dag".to_string(), "src/v2".to_string()]);
        let (graph, indices) =
            resolve_entry_with_index_for_discovery_corpus(&index, EFFECT_REACH_STD_ENTRY)
                .unwrap_or_else(|e| panic!("resolve {EFFECT_REACH_STD_ENTRY}: {e}"));
        let ctx = make_eval_context(&graph, indices, ExecutionMode::Wet);
        let val = v1_interpreter::with_active_context(&ctx, || {
            v1_interpreter::eval_data_item_value(&ctx, "effect_reach_host_sink_callee_symbols_v0")
        })
        .unwrap_or_else(|e| panic!("eval effect_reach_host_sink_callee_symbols_v0: {e}"))
        .unwrap_or_else(|| {
            panic!("effect_reach_host_sink_callee_symbols_v0 not found as a data item")
        });
        let Value::List(items) = val else {
            panic!("effect_reach_host_sink_callee_symbols_v0 is not a List: {val:?}");
        };
        items
            .iter()
            .map(|item| match item {
                Value::Str(s) => s.clone(),
                other => panic!(
                    "effect_reach_host_sink_callee_symbols_v0 entry is not a String: {other:?}"
                ),
            })
            .collect()
    }

    #[test]
    fn effect_reach_host_sink_markers_v0_is_synced_with_dag_authority() {
        let dag_symbols = dag_host_sink_callee_symbols();
        let rust_symbols: HashSet<String> = EFFECT_REACH_HOST_SINK_MARKERS
            .iter()
            .map(|s| s.to_string())
            .collect();
        let missing_from_rust: Vec<&String> = dag_symbols.difference(&rust_symbols).collect();
        assert!(
            missing_from_rust.is_empty(),
            "`.dag` authority `effect_reach_host_sink_callee_symbols_v0` declares sink symbol(s) \
             {missing_from_rust:?} not mirrored in Rust `EFFECT_REACH_HOST_SINK_MARKERS` \
             (src/v1/stage0/src/cli_run.rs) — axis skips fail-open when the Rust bridge \
             under-approximates the authority"
        );
        let extra_in_rust: Vec<&String> = rust_symbols.difference(&dag_symbols).collect();
        assert!(
            extra_in_rust.is_empty(),
            "Rust `EFFECT_REACH_HOST_SINK_MARKERS` declares sink symbol(s) {extra_in_rust:?} \
             absent from `.dag` authority `effect_reach_host_sink_callee_symbols_v0` — keep the \
             single roster in sync (§3)"
        );
    }
}

/// Precompute-grain count for axis (iv): the number of distinct entries among `rows` whose
/// import closure reaches a declared live-read carrier home
/// (`runtime_data_dependency_touched_via_carrier_closure`), given the full raw touched-path
/// set. Feeds `floor_row_precompute_would_skip`'s `touched_runtime_dependency_entry_count` —
/// nonzero pins the whole-tree precompute exactly as the other three axes do.
fn discovery_rows_runtime_dependency_touched_count(
    rows: &[DiscoveryRow],
    facts: &ModuleGraphFactsLive,
    touched_paths: &[String],
) -> usize {
    if touched_paths.is_empty() {
        return 0;
    }
    let mut seen: HashSet<&str> = HashSet::new();
    rows.iter()
        .filter(|row| seen.insert(row.entry.as_str()))
        .filter(|row| {
            runtime_data_dependency_touched_via_carrier_closure(&row.entry, facts, touched_paths)
        })
        .count()
}

/// Skip-before-resolve fast path (affected-set precompute-pruning Step 4 consumer-2):
/// when import-closure `entry_file_touched` is false and no declaration-level edit
/// targets this entry, every kernel witness in the entry would skip — receipt:
/// `green_skip_for_file_outside_import_closure` (diff outside import closure → empty
/// frontier / no touch). Live-tree entries are excluded (`reads_live_tree`: their inputs
/// are outside the resolved closure, so the diff cannot bound them). Data-item edits land
/// in `overlapping_data_items` (not `touched_entry_files`); any edited data-item file in
/// the entry import closure must resolve so `rerun_frontier_nodes_for_entry` can
/// discriminate referenced nodes (`red_node_frontier_fires_for_referenced_data_item`).
// SCAFFOLD (§7 hand-Rust shrink-to-zero, dissolution named): pre-resolve skip for rows
// provably outside all three skip axes without loading the resolved graph. Dissolves at
// Step 5 (`docs/plans/affected-set-precompute-pruning.md`) when the Rust parallel
// (`NodeFrontierSeeds`, `run_discovery_rows` selection) is deleted and the `.dag`
// `floor_witness_run_disposition` query owns the same predicate end-to-end.
fn entry_qualifies_for_skip_without_resolve(
    entry_path: &str,
    reads_live_tree: bool,
    facts: &ModuleGraphFactsLive,
    declared_paths: &HashSet<String>,
    touched_entry_paths: &[String],
    diff_edits: &FloorDiffEdits,
) -> Result<bool, String> {
    // Fail-closed on the substrate-declared disposition (v2.std.live_tree): a
    // `ReadsLiveTree` entry never predict-skips. Replaces the deleted entry-text
    // classifier's per-function `witness_test_fn_uses_live_host_scan` scan; the
    // disposition is entry-grain, so one flag decides the whole entry.
    if reads_live_tree {
        return Ok(false);
    }
    if diff_edits
        .edited_test_fns
        .iter()
        .any(|(file, _)| diff_file_matches_entry(file, entry_path))
    {
        return Ok(false);
    }
    if diff_edits
        .touched_entry_files
        .iter()
        .any(|file| diff_file_matches_entry(file, entry_path))
    {
        return Ok(false);
    }
    if entry_file_touched_via_import_closure(
        entry_path,
        facts,
        declared_paths,
        touched_entry_paths,
    )? {
        return Ok(false);
    }
    if runtime_data_dependency_touched_via_carrier_closure(entry_path, facts, touched_entry_paths) {
        return Ok(false);
    }
    let declared_axis = declared_source_refs_axis_for_entry(
        entry_path,
        facts,
        &default_source_roots(),
        touched_entry_paths,
    );
    if declared_axis != DeclaredSourceRefAxis::Absent {
        if declared_source_refs_blocks_skip(declared_axis) {
            return Ok(false);
        }
    } else if effect_reach_touched_via_path_literals(entry_path, facts, touched_entry_paths) {
        return Ok(false);
    }
    if !diff_edits.overlapping_data_items.is_empty() {
        let data_item_files: Vec<String> = diff_edits
            .overlapping_data_items
            .iter()
            .map(|(file, _)| file.clone())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        if entry_file_touched_via_import_closure(
            entry_path,
            facts,
            declared_paths,
            &data_item_files,
        )? {
            return Ok(false);
        }
    }
    Ok(true)
}

fn discovery_entry_fast_skip_without_resolve(
    rows: &[DiscoveryRow],
    facts: &ModuleGraphFactsLive,
    declared_paths: &HashSet<String>,
    touched_entry_paths: &[String],
    diff_edits: &FloorDiffEdits,
) -> Result<HashSet<String>, String> {
    // Entry-grain disposition: OR the rows' `reads_live_tree` per entry (they agree by
    // construction — one declaration per entry file — but OR fails closed if they ever diverge).
    let mut by_entry: HashMap<String, bool> = HashMap::new();
    for row in rows {
        let live = by_entry.entry(row.entry.clone()).or_insert(false);
        *live = *live || row.reads_live_tree;
    }
    let mut fast = HashSet::new();
    for (entry, reads_live_tree) in by_entry {
        if entry_qualifies_for_skip_without_resolve(
            &entry,
            reads_live_tree,
            facts,
            declared_paths,
            touched_entry_paths,
            diff_edits,
        )? {
            fast.insert(entry);
        }
    }
    Ok(fast)
}

/// Keep the width-1 closure calibration oracle honest when resolve is skipped: count the
/// same import-closure modules the pre-resolve walk uses (`roster_import_closure_nodes_pre_resolve`).
// SCAFFOLD (§7): calibration-only companion to skip-before-resolve above; dissolves with it.
fn augment_closure_modules_from_import_facts(
    entry_path: &str,
    facts: &ModuleGraphFactsLive,
    out: &mut HashSet<String>,
) {
    let closure_paths: HashSet<String> = import_closure_live_paths_with_facts(entry_path, facts)
        .into_iter()
        .map(|p| workspace_relative_repo_path(&p))
        .collect();
    for node in &facts.nodes {
        let rel = workspace_relative_repo_path(&node.path);
        if closure_paths.contains(&rel) {
            out.insert(node.module.clone());
        }
    }
}

fn parse_unified_diff_departed_paths(diff_text: &str) -> HashSet<String> {
    // The diff itself is the single authority on departure: a path leaves the
    // tree only as a deletion (`+++ /dev/null`) or the from-side of a rename
    // (`diff --git a/old b/new`, old != new). Any other diff-named path that is
    // absent from the working tree is observation incoherence, not a departure.
    let mut departed = HashSet::new();
    let mut last_minus: Option<String> = None;
    for line in diff_text.lines() {
        if let Some(rest) = line.strip_prefix("diff --git a/") {
            if let Some((old, new)) = rest.split_once(" b/") {
                if old != new {
                    departed.insert(normalize_repo_path(old));
                }
            }
        } else if let Some(rest) = line.strip_prefix("--- a/") {
            last_minus = Some(normalize_repo_path(rest));
        } else if line.starts_with("+++ /dev/null") {
            if let Some(gone) = last_minus.take() {
                departed.insert(gone);
            }
        }
    }
    departed
}

fn parse_unified_diff_added_paths(diff_text: &str) -> HashSet<String> {
    // Wholly-added files (`--- /dev/null` → `+++ b/path`) necessarily touch line 1
    // (the module header). Attribute at declaration grain; do not conflate with
    // modify-side module renames (fail-closed below).
    //
    // A rename *destination* (`rename to NEW`, git's own rename signal) is likewise
    // new-at-path: its declaration set is established fresh at NEW, so a module-header
    // (line 1) change there is the wholly-added case, not an in-place module rename.
    // This mirrors `parse_unified_diff_departed_paths`, which already treats the rename
    // FROM-side (`old != new`) as a departure; without the symmetric TO-side a
    // rename+modify would fail-closed on its unavoidable line-1 change. An in-place
    // modify (`diff --git a/PATH b/PATH`, no `rename to`) still touches line 1 with no
    // added-side entry, so it stays fail-closed as before.
    let mut added = HashSet::new();
    let mut minus_is_null = false;
    for line in diff_text.lines() {
        if let Some(rest) = line.strip_prefix("rename to ") {
            added.insert(normalize_repo_path(rest.trim()));
        } else if let Some(rest) = line.strip_prefix("--- ") {
            minus_is_null = rest.trim() == "/dev/null";
        } else if let Some(rest) = line.strip_prefix("+++ b/") {
            if minus_is_null {
                added.insert(normalize_repo_path(rest));
            }
            minus_is_null = false;
        }
    }
    added
}

fn floor_diff_edits_from_diff_text(
    index: &MultiEntryIndex,
    diff_text: &str,
) -> Result<FloorDiffEdits, String> {
    let line_ranges = parse_unified_diff_line_ranges(diff_text);
    let changed = parse_unified_diff_changed_new_lines(diff_text);
    let departed = parse_unified_diff_departed_paths(diff_text);
    let added = parse_unified_diff_added_paths(diff_text);
    floor_diff_edits_from_line_ranges(index, &line_ranges, &changed, &departed, &added)
}

fn floor_diff_edits_from_line_ranges(
    index: &MultiEntryIndex,
    line_ranges_by_file: &HashMap<String, Vec<FileLineRange>>,
    changed_new_lines_by_file: &HashMap<String, HashSet<i64>>,
    departed_paths: &HashSet<String>,
    added_paths: &HashSet<String>,
) -> Result<FloorDiffEdits, String> {
    let mut overlapping_data_items = HashSet::new();
    let mut edited_test_fns = HashSet::new();
    let mut touched_entry_files = HashSet::new();
    // #6269 attributes src/v1/ .dag changes through a dedicated index; the structural-∅ fix
    // dropped the saw_non_dag/saw_dag refusal (a non-.dag-only diff is a nominal empty frontier,
    // handled by the `continue` arm below), so neither flag is needed here.
    let v1_attribution_index = if line_ranges_by_file
        .keys()
        .any(|p| normalize_repo_path(p).starts_with("src/v1/"))
    {
        Some(build_v1_attribution_multi_entry_index())
    } else {
        None
    };
    for (file_path, ranges) in line_ranges_by_file {
        if !file_path.ends_with(".dag") {
            // A non-.dag changed path is a structural-∅ for the .dag frontier: it declares no
            // .dag nodes, so there is nothing to attribute, and its coverage lives in the Rust
            // gates (rust_tests), not the .dag witnesses. Skipping it yields an empty .dag
            // frontier -- the SAME nominal outcome as an empty diff. This is NOT ignorance: the
            // only ignorance state is a failed git-diff observation (UnifiedDiffFail upstream,
            // floor_diff_observe.dag; operator ruling 2026-07-05). Structural-∅ and ignorance
            // are different states -- the mirror of the departed-.dag-path arm below.
            continue;
        }
        let file_norm = normalize_repo_path(file_path);
        if !std::path::Path::new(file_path).exists() {
            if departed_paths.contains(&file_norm) {
                // Departed per the diff (deletion / rename-from): its decl set
                // is empty by construction — the file has no declarations to
                // attribute. The path-grain fact stays in changed_paths;
                // dependents that imported it fail loudly at their own resolve.
                continue;
            }
            // Absent from the tree but NOT marked departed by the diff: the
            // observation is incoherent (stale tree, quoting artifact, bogus
            // path). Structural-∅ and ignorance are different states — refuse.
            return Err(format!(
                "affected-set derivation refused: diff names {file_path} with \
                 content changes but the path is absent from the working tree \
                 and the diff does not mark it departed (deletion/rename)"
            ));
        }
        let resolve_index = if file_norm.starts_with("src/v1/") {
            v1_attribution_index.as_ref().expect("v1 attribution index")
        } else {
            index
        };
        let content = match std::fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(e) => return Err(format!("read failed for {file_path}: {e}")),
        };
        // Attribution is a PARSE-grade fact: it needs each touched file's
        // declaration line map (names + spans + data/fn kind), never its typecheck.
        // The former full entry RESOLVE here made every touched file's typecheck
        // health gate the whole frontier — on a corpus-wide diff, one latent-red
        // module (a batch-2 debt row no gate compiles) dead-ended batch 2 at one
        // entry per CI cycle. Parse errors still refuse (typed, located); typecheck
        // reds surface where they belong — as that entry's own counted discovery row.
        let source = Rc::new(v1_compiler_compile::SourceFile {
            path: file_norm.clone(),
            content: content.clone(),
        });
        let (module_node, nl) = match parse_module_node_from_index_source(resolve_index, source) {
            Ok(pair) => pair,
            Err(e) => return Err(format!("parse failed for {file_path}: {e}")),
        };
        let single_si: Rc<HashMap<String, Rc<NewlineIndex>>> = Rc::new({
            let mut m = HashMap::new();
            m.insert(file_norm.clone(), nl.clone());
            m
        });
        let test_fn_names: HashSet<String> = scan_test_decl_names(&content).into_iter().collect();
        let mut decls: Vec<(i64, String, bool)> = Vec::new();
        for item in crate::v1_std_core::module_items(module_node.clone()).iter() {
            let line = byte_to_line_col(nl.clone(), item.span.start).line;
            let name = authored_name_at(single_si.clone(), item.clone());
            let is_data = item_kind(item.clone()) == ItemKind::DataItem;
            decls.push((line, name, is_data));
        }
        for (name, line) in scan_test_decl_lines(&content) {
            if !decls.iter().any(|(_, n, _)| n == &name) {
                decls.push((line, name, false));
            }
        }
        if decls.is_empty() {
            // A declaration-less module (a lone `module` line — e.g. the
            // shadow-masked fixtures) has nothing to attribute at decl grain;
            // its only edit surface IS the file, so the file-grain touched set
            // carries it (dependents rerun via the import closure). Refusing
            // here dead-ended the frontier on a fixture that is legitimately
            // empty — not an incoherent observation.
            touched_entry_files.insert(file_norm.clone());
            continue;
        }
        decls.sort_by_key(|(line, _, _)| *line);
        let first_decl_line = decls[0].0;
        let mut changed =
            changed_new_lines_for_file(changed_new_lines_by_file, file_path, &file_norm);
        // Deletion-only hunks (`-` rows, zero `+` width) still carry a new-side anchor in the
        // hunk header; fall back to parsed ranges when no `+`/`-` rows were attributed.
        if changed.is_empty() {
            for r in ranges {
                let end = if r.end < r.start { r.start } else { r.end };
                for l in r.start..=end {
                    changed.insert(l);
                }
            }
        }
        // Module-line edits (line 1) stay fail-closed for modifies — renaming can
        // change entry identity. Wholly-added files necessarily touch line 1.
        if changed.contains(&1) && !added_paths.contains(&file_norm) {
            return Err(format!("diff before first declaration in {file_path}"));
        }
        let has_pre_decl = changed.iter().any(|&l| l < first_decl_line);
        let has_post_decl = changed.iter().any(|&l| l >= first_decl_line);
        if has_pre_decl {
            touched_entry_files.insert(file_norm.clone());
            if !has_post_decl {
                continue;
            }
        }
        for i in 0..decls.len() {
            let (line, name, is_data) = &decls[i];
            let decl_end = decls.get(i + 1).map(|(l, _, _)| l - 1).unwrap_or(i64::MAX);
            if !changed.iter().any(|&l| l >= *line && l <= decl_end) {
                continue;
            }
            if test_fn_names.contains(name) {
                edited_test_fns.insert((file_norm.clone(), name.clone()));
            } else if *is_data {
                overlapping_data_items.insert((file_norm.clone(), name.clone()));
            } else {
                touched_entry_files.insert(file_norm.clone());
            }
        }
    }
    // A present diff whose changed paths are all non-.dag lands here with an empty frontier
    // (structural-∅): it flows through as every row's not-affected skip -- nominal and
    // transparent, exactly like an empty diff, never a refusal. Observation failure (the only
    // ignorance state) is refused upstream in floor_git_diff_range; a successful observation
    // with an empty .dag subset is not ignorance.
    Ok(FloorDiffEdits {
        overlapping_data_items,
        edited_test_fns,
        touched_entry_files,
    })
}

/// True when `name` is declared as a `data` item at `file_norm`, verified against the entry's
/// own resolved import closure (`ctx.modules`) rather than by bare name — `item_registry` is
/// flat-namespace-keyed (name only, no origin file), so a homonym declared in some unrelated
/// file must not be mistaken for the diff-changed declaration. Because `ctx.modules` already
/// contains only this entry's resolved import closure, a `file_norm` the entry does not import
/// yields no match here regardless of name collisions elsewhere in the corpus.
fn data_item_declared_in_file(
    ctx: &v1_interpreter::InterpContext,
    name: &str,
    file_norm: &str,
) -> bool {
    ctx.modules.iter().any(|module| {
        module.items.iter().any(|item| {
            item_kind(item.clone()) == ItemKind::DataItem
                && span_file_matches(&item.span.file, file_norm)
                && authored_name_at(ctx.source_indices.clone(), item.clone()) == name
        })
    })
}

fn rerun_frontier_nodes_for_entry(
    ctx: &v1_interpreter::InterpContext,
    entry_path: &str,
    edits: &FloorDiffEdits,
) -> Result<Vec<v1_interpreter::Value>, String> {
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    let entry_norm = repo_relative_dag_path(entry_path);
    for (file, name) in &edits.overlapping_data_items {
        let file_norm = repo_relative_dag_path(file);
        // Same-entry-file overlapping data always seeds the frontier. A cross-file data item
        // seeds it too, once verified against the entry's own resolved import closure
        // (`data_item_declared_in_file`) — matching on bare name alone would silently widen
        // SelectionApplied to any entry that happens to import a same-named data item declared
        // elsewhere (e.g. `construction_justification` on every top-level lens). Before this
        // check existed, an entry that genuinely imports the changed data decl (e.g. a witness
        // importing `generated_stage0_files` from a different file) fell through the `continue`
        // below and predict-skipped (#6543 falsifier divergence, run 29293446579).
        if file_norm != entry_norm && !data_item_declared_in_file(ctx, name, &file_norm) {
            continue;
        }
        if !ctx.item_registry.contains_key(name) {
            continue;
        }
        let Some(val) = v1_interpreter::with_active_context(ctx, || {
            v1_interpreter::eval_data_item_value(ctx, name)
        })
        .map_err(|e| format!("re-eval `{name}` in {entry_path}: {e}"))?
        else {
            continue;
        };
        let mut item_nodes = Vec::new();
        collect_node_values(&val, ctx, &mut item_nodes);
        for node in item_nodes {
            let key = ctx.format_value(&node);
            if seen.insert(key) {
                out.push(node);
            }
        }
    }
    Ok(out)
}

fn entry_touches_rerun_frontier(
    ctx: &v1_interpreter::InterpContext,
    frontier: &v1_interpreter::Value,
) -> Result<bool, String> {
    let mut saw_claim = false;
    let initializer_values = v1_interpreter::with_active_context(ctx, || {
        v1_interpreter::eval_data_initializer_values(ctx)
    })
    .map_err(|e| format!("{e}"))?;
    for val in initializer_values {
        if !value_is_test_claim(&val, ctx) {
            continue;
        }
        if !test_claim_selection_has_node_corpus(&val, ctx) {
            continue;
        }
        saw_claim = true;
        match call_test_claim_fn_bool(
            ctx,
            "test_claim_evaluation_touches_rerun_frontier",
            &val,
            frontier,
            "c",
        ) {
            Ok(Some(true)) => return Ok(true),
            Ok(Some(false)) | Ok(None) => {}
            Err(msg) => {
                return Err(format!(
                    "test_claim_evaluation_touches_rerun_frontier failed ({msg}) — declared \
                     selection machinery must evaluate; no silent run-everything fallback"
                ));
            }
        }
        match call_test_claim_fn_bool(
            ctx,
            "floor_claim_touches_rerun_frontier",
            &val,
            frontier,
            "claim",
        ) {
            Ok(Some(true)) => return Ok(true),
            Ok(Some(false)) | Ok(None) => {}
            Err(msg) => {
                return Err(format!(
                    "floor_claim_touches_rerun_frontier failed ({msg}) — declared selection \
                     machinery must evaluate; no silent run-everything fallback"
                ));
            }
        }
    }
    Ok(!saw_claim)
}

/// P4 advisory-first (witness-realization plan): marshal an `Option<i64>` into the
/// `.dag` `Int?` (Optional) `Value` the modeled `realize_advisory` expects.
fn realize_advisory_optional_int(
    ctx: &v1_interpreter::InterpContext,
    v: Option<i64>,
) -> v1_interpreter::Value {
    use std::rc::Rc;
    use v1_interpreter::Value;
    match v {
        Some(n) => Value::Variant {
            type_name: ctx.sym("Optional"),
            variant_name: ctx.sym("Present"),
            fields: Rc::new(vec![(ctx.sym("value"), Value::Int(n))]),
        },
        None => Value::Variant {
            type_name: ctx.sym("Optional"),
            variant_name: ctx.sym("Absent"),
            fields: Rc::new(vec![]),
        },
    }
}

/// P4 advisory-first: for each discovery witness, DERIVE its space bound
/// (`ComplexityReport.function_space_bytes` at an empty size-env — a closed
/// witness has no free size-vars) and log the memory-packed width `std.realize_pack`
/// would schedule, alongside the live `MemoryGovernor` admission. This changes NO
/// scheduling — it proves the derived bounds are sound on the real corpus before
/// the governor is demoted (§5). The packing LAW stays modeled: `realize_advisory`
/// is interpreted through the bridge, never reimplemented in Rust (§2). Gated by
/// `GUNBC_REALIZE_ADVISORY`.
pub fn emit_realize_advisory_for_rows(source_roots: &[String], rows: &[DiscoveryRow]) {
    use v1_interpreter::Value;
    // The witness graphs don't import std.realize_pack, so interpret the law in its
    // own ctx, built once.
    let realize_ctx = match resolve_entry_graph(source_roots, "dag/std/realize_pack.dag") {
        Ok((g, idx)) => make_eval_context(&g, idx, v1_interpreter::ExecutionMode::Hermetic),
        Err(e) => {
            eprintln!("[realize-advisory] disabled: cannot load std.realize_pack: {e}");
            return;
        }
    };
    // Host budget: the SAME single authority the MemoryGovernor schedules against
    // (env -> cgroup memory.high -> memory.max -> meminfo). Unreadable -> the modeled
    // law refuses (BudgetRefused), never a fabricated width.
    let (budget_opt, budget_source) = crate::memory_governor::read_host_budget_bytes();
    let budget_bytes: Option<i64> = budget_opt.map(|b| b as i64);
    let independence: i64 = std::thread::available_parallelism()
        .map(|n| n.get() as i64)
        .unwrap_or(1);
    // Group by entry so each entry resolves + analyzes once.
    let mut by_entry: std::collections::BTreeMap<String, Vec<String>> =
        std::collections::BTreeMap::new();
    for r in rows {
        by_entry
            .entry(r.entry.clone())
            .or_default()
            .push(r.function.clone());
    }
    let (mut derivable, mut unknown) = (0usize, 0usize);
    for (entry, functions) in &by_entry {
        let (graph, source_indices) = match resolve_entry_graph(source_roots, entry) {
            Ok(p) => p,
            Err(_) => continue,
        };
        let report =
            v1_compiler_compile::run_complexity_analysis(graph.clone(), source_indices.clone());
        for function in functions {
            let derived: Option<i64> = report.function_space_bytes.get(function).copied();
            if derived.is_some() {
                derivable += 1;
            } else {
                unknown += 1;
            }
            let args = vec![
                (
                    Some("derived_bytes".to_string()),
                    realize_advisory_optional_int(&realize_ctx, derived),
                ),
                (
                    Some("budget_bytes".to_string()),
                    realize_advisory_optional_int(&realize_ctx, budget_bytes),
                ),
                (
                    Some("independence_width".to_string()),
                    Value::Int(independence),
                ),
            ];
            match v1_interpreter::run_in_context_with_args(
                &realize_ctx,
                "realize_advisory",
                &args,
                false,
            ) {
                Ok(Value::Record { fields, .. }) => {
                    let width = match realize_ctx.field(&fields, "width") {
                        Some(Value::Int(w)) => *w,
                        _ => -1,
                    };
                    let verdict = match realize_ctx.field(&fields, "verdict") {
                        Some(Value::Str(s)) => s.clone(),
                        _ => String::new(),
                    };
                    let db = derived
                        .map(|d| d.to_string())
                        .unwrap_or_else(|| "unknown".to_string());
                    let bb = budget_bytes
                        .map(|b| b.to_string())
                        .unwrap_or_else(|| "unreadable".to_string());
                    eprintln!(
                        "[realize-advisory] entry={entry} fn={function} derived_bytes={db} \
                         budget={bb} independence={independence} predicted_width={width} verdict={verdict}"
                    );
                }
                Ok(_) => eprintln!(
                    "[realize-advisory] entry={entry} fn={function} — bridge returned non-record"
                ),
                Err(e) => {
                    eprintln!("[realize-advisory] entry={entry} fn={function} — bridge error: {e}")
                }
            }
        }
    }
    eprintln!(
        "[realize-advisory] summary: {} function(s), {} with a derived bound, \
         {} unknown (maturation reserve); budget={} source={}",
        derivable + unknown,
        derivable,
        unknown,
        budget_bytes
            .map(|b| b.to_string())
            .unwrap_or_else(|| "unreadable".to_string()),
        budget_source,
    );
}

pub fn run_discovery_corpus_with_options(
    source_roots: &[String],
    scan_dirs: &[String],
    explicit_entries: &[(String, String)],
    execution_mode: v1_interpreter::ExecutionMode,
    width_policy: DiscoveryWidthPolicy,
    options: DiscoveryCorpusOptions,
) -> Result<DiscoverySummary, String> {
    check_floor_filename_hygiene(source_roots)?;
    let mut rows =
        if options.explicit_roster_only || (scan_dirs.is_empty() && !explicit_entries.is_empty()) {
            Vec::new()
        } else {
            discover_floor_corpus_rows_scoped(
                source_roots,
                scan_dirs,
                &options.exclude_substrings,
                &options.discovery_scope_dirs,
            )?
        };
    let mut seen: std::collections::BTreeSet<(String, String)> = rows
        .iter()
        .map(|r| (r.entry.clone(), r.function.clone()))
        .collect();
    for (entry, function) in explicit_entries {
        if seen.insert((entry.clone(), function.clone())) {
            rows.push(DiscoveryRow {
                label: function.clone(),
                entry: entry.clone(),
                function: function.clone(),
                reads_live_tree: read_entry_live_tree_disposition(entry)?,
            });
        }
    }
    rows.sort_by(|a, b| {
        a.entry
            .cmp(&b.entry)
            .then_with(|| a.function.cmp(&b.function))
    });
    if rows.is_empty() {
        return Err("discovery roster produced no rows (empty corpus → fail closed)".to_string());
    }
    // P4 advisory-first: predict the memory-packed width per witness from its derived
    // space bound, logged beside the governor — no scheduling change. Gated (opt-in).
    if std::env::var("GUNBC_REALIZE_ADVISORY").is_ok() {
        emit_realize_advisory_for_rows(source_roots, &rows);
    }
    let deferred_rows = if options.explicit_roster_only || scan_dirs.is_empty() {
        Vec::new()
    } else {
        collect_deferred_discovery_rows(source_roots, &options.exclude_substrings)?
    };
    let admission_orphans = collect_unexecuted_deferred_witnesses(&deferred_rows);
    refuse_unexecuted_deferred_witnesses(&admission_orphans)?;
    eprintln_deferred_discovery_rows(&deferred_rows);
    set_phase(FloorPhase::Discovery, "discovery-roster");
    let selection_enabled = options.node_frontier_selection != NodeFrontierSelectionMode::Off;
    // No degradation arm: a non-Off node_frontier_selection is a DECLARED capability,
    // so every input it needs (the git-diff observation, the frontier attribution, the
    // affected-set runner) must be present — a failure is a loud typed error, never a
    // silent run-everything fallback. To run without selection, declare the flag false.
    let diff_text = if selection_enabled {
        floor_git_diff_range().map_err(|msg| {
            format!(
                "AFFECTED-SET REFUSAL cause=DiffObservationRefusal rows={} — git diff \
                 observation failed ({msg}); observation failure is the only ignorance \
                 state (operator ruling 2026-07-05) and refuses every enrolled row rather \
                 than widening to a full-corpus run (declare node_frontier_selection: \
                 SelectionOff to run without selection)",
                rows.len()
            )
        })?
    } else {
        String::new()
    };
    // Path grain (changed_paths, departed_paths) is observed via the typed
    // `git diff --name-status -z` interface (extdeps.git), not scraped from the
    // unified diff's `diff --git a/OLD b/NEW` header: name-status is git's own
    // machine surface for path identity/rename, so it is the single authority —
    // the unified diff below stays scoped to LINE grain (hunk ranges) only.
    let (name_status_changed_paths, name_status_departed_paths) =
        if options.node_frontier_selection != NodeFrontierSelectionMode::Off {
            floor_git_diff_name_status_range().map_err(|msg| {
                format!(
                    "AFFECTED-SET REFUSAL cause=DiffObservationRefusal rows={} — git \
                     diff --name-status observation failed ({msg}); observation failure \
                     is the only ignorance state (operator ruling 2026-07-05) and \
                     refuses every enrolled row rather than widening to a full-corpus \
                     run (declare node_frontier_selection: SelectionOff to run without \
                     selection)",
                    rows.len()
                )
            })?
        } else {
            (Vec::new(), HashSet::new())
        };
    let mut line_ranges_by_file = parse_unified_diff_line_ranges(&diff_text);
    for path in &name_status_changed_paths {
        line_ranges_by_file.entry(path.clone()).or_default();
    }
    let changed_new_lines_by_file = parse_unified_diff_changed_new_lines(&diff_text);
    let added_paths = parse_unified_diff_added_paths(&diff_text);
    let changed_paths: Vec<String> = name_status_changed_paths;
    // Union-resolve S1 (resolver-graph-major-design.md §7): ONE index for the whole
    // process step on the pump thread — prelude-warmed parse/typed caches instead of a
    // private cold build per consumer. S2a increment C (cross-worker-typecheck-share-
    // design.md §4): adaptive worker shards arm ONE process-scoped typed_module_cache
    // (serde byte transport). The pump thread keeps `process_shared_index` (private per-
    // index `Rc`) so prelude work does not duplicate into the shared store; workers alone
    // read/write the shared store as the typed-cache authority (no local Rc duplicate).
    // Store creation lives in the Adaptive match arm below — unrepresentable on Serial.
    let index = process_shared_index(source_roots);
    // Calibration receipt, emitted BEFORE the heavy resolve so it survives a host-level
    // OOM kill (censored lower-bound pairs for the space-lens memory predictor — design
    // in flight on PR #6442; consumer binds to roster_import_closure_nodes_pre_resolve):
    // the transitive import-CLOSURE size — never the roster/entry count (pairing an
    // entry count against a whole-closure peak inflates bytes-per-node by the fan-in
    // factor). Skip-before-resolve (run_discovery_rows) elides cold resolve for
    // import-closure-unaffected entries while folding their module-graph closure into
    // the post-resolve union so this pre-resolve count stays paired with calibration.
    let pre_resolve_closure_nodes = {
        let prefix_entries: &[&str] = if selection_enabled {
            &[FLOOR_RUNNER_ENTRY]
        } else {
            &[]
        };
        let n = roster_import_closure_nodes_pre_resolve(&rows, prefix_entries, &index)?;
        eprintln!(
            "[calibration] roster_import_closure_nodes={} rows={} (both-closure loader, pre-resolve; pairs with the floor cgroup memory.peak steps — on a killed run this line plus the last [gantt] rss_mib sample are the lower-bound receipt)",
            n,
            rows.len()
        );
        n
    };
    // Empty diff is not a state (operator ruling 2026-07-05): an empty touched-path
    // set flows through the general selection machinery — empty frontier, zero edited
    // fns — so every row takes the normal not-affected skip. Disabling selection here
    // was the run-everything absorbing arm.
    let (skip_enabled, diff_edits) = if selection_enabled {
        match floor_diff_edits_from_line_ranges(
            &index,
            &line_ranges_by_file,
            &changed_new_lines_by_file,
            &name_status_departed_paths,
            &added_paths,
        ) {
            Ok(edits) => (true, edits),
            Err(msg) => {
                return Err(format!(
                    "node-frontier population failed ({msg}) — the diff-to-declaration \
                     attribution is declared selection machinery; no silent full-corpus \
                     fallback"
                ));
            }
        }
    } else {
        (false, FloorDiffEdits::default())
    };
    let floor_runner_ctx = if selection_enabled {
        // Resolve the floor runner through the SAME shared index as the rows (union-resolve
        // S1) rather than a private per-call resolve — its closure shares the std/spec prefix
        // with the roster, so co-resolving here means that prefix is not typechecked twice.
        match resolve_entry_with_index(&index, FLOOR_RUNNER_ENTRY) {
            Ok((graph, source_indices)) => {
                Some(make_eval_context(&graph, source_indices, execution_mode))
            }
            Err(msg) => {
                return Err(format!(
                    "floor runner resolve failed ({msg}) — a non-Off node_frontier_selection \
                     declares the affected-set machinery ({FLOOR_RUNNER_ENTRY}) and it \
                     must resolve; no silent full-corpus fallback"
                ));
            }
        }
    } else {
        None
    };
    let skip_precompute = if skip_enabled {
        let live_row_count = discovery_rows_live_tree_count(&rows);
        match floor_runner_ctx.as_ref() {
            Some(ctx) => {
                let touched_runtime_dependency_entry_count =
                    discovery_rows_runtime_dependency_touched_count(
                        &rows,
                        &index.module_graph_facts,
                        &changed_paths,
                    );
                let precompute = call_floor_row_precompute_would_skip(
                    ctx,
                    live_row_count,
                    &changed_paths,
                    diff_edits.overlapping_data_items.len(),
                    diff_edits.edited_test_fns.len(),
                    diff_edits.touched_entry_files.len(),
                    touched_runtime_dependency_entry_count,
                );
                match precompute {
                    Ok(skip) => skip,
                    Err(msg) => {
                        return Err(format!(
                            "floor precompute_would_skip failed ({msg}) — declared selection \
                             machinery must evaluate; no silent fallback"
                        ));
                    }
                }
            }
            None => false,
        }
    } else {
        false
    };
    let whole_tree_published_keys = if skip_precompute {
        eprintln!(
            "run_discovery_corpus: skipping whole-tree published-mock precompute (scoped diff, empty node frontier, no edited test fns, no entry-file fn edits)"
        );
        None
    } else {
        match precompute_whole_tree_published_mock_keys(source_roots) {
            Ok(keys) if keys.is_empty() => None,
            Ok(keys) => Some(keys),
            Err(e) => {
                return Err(format!(
                    "whole-tree published mock corpus precompute failed: {e}"
                ));
            }
        }
    };
    eprintln_affected_set_categorization(
        options.node_frontier_selection,
        &rows,
        &index,
        &diff_edits,
    );
    let floor_color = floor_color_enabled();
    let floor_stream = floor_stream_enabled();
    return match width_policy {
        DiscoveryWidthPolicy::Serial => {
            let summary = run_discovery_rows(
                &rows,
                &index,
                execution_mode,
                options.node_frontier_selection,
                &changed_paths,
                &diff_edits,
                floor_runner_ctx.as_ref(),
                whole_tree_published_keys.clone(),
                options.witness_budget_policy(),
                ShardStyle {
                    shard_id: 0,
                    shard_count: 1,
                    color: floor_color,
                    stream: floor_stream,
                },
            )?;
            // Definition-drift oracle (single-authority reconciliation, executable): on a
            // COMPLETED serial run the pre-resolve import walk and the post-resolve
            // resolved-graph union must agree — resolve resolves exactly the transitive
            // imports. Serial only: the merged multi-worker field is max-over-workers, not
            // the process union, so the comparison is ill-posed there. A mismatch means one
            // closure definition is wrong (an implicit prelude module the walk missed, or a
            // resolve seeding change) and the space-lens calibration pair would silently
            // skew — refuse rather than emit a lying receipt.
            if summary.roster_closure_nodes != pre_resolve_closure_nodes {
                return Err(format!(
                    "[calibration] closure-definition drift: pre-resolve both-closure loader = {} nodes, \
                     post-resolve resolved union = {} — the two closure definitions diverged \
                     (loader fork or seeding change); reconcile the definitions before trusting \
                     bytes-per-node calibration (roster_import_closure_nodes_pre_resolve is the \
                     shared authority)",
                    pre_resolve_closure_nodes, summary.roster_closure_nodes
                ));
            }
            eprintln!(
                "[calibration] closure consistency: pre-resolve both-closure == post-resolve union == {} node(s)",
                pre_resolve_closure_nodes
            );
            Ok(attach_deferred_discovery_rows(summary, deferred_rows))
        }
        DiscoveryWidthPolicy::Adaptive(governor) => {
            // Adaptive pool: entry-groups drain through governor-admitted workers. Each worker
            // builds ONE whole-tree index and holds it for its lifetime, amortizing the expensive
            // resident structure across every group it pulls — admission of a worker (not of a
            // group) is therefore the memory-relevant act the governor decides.
            let groups = entry_row_groups(&rows);
            let spawn_target_width = governor.current_target_width();
            eprintln!(
                "run_discovery_corpus: adaptive pool over {} entry-group(s), {} row(s) (governor target_width={})",
                groups.len(),
                rows.len(),
                spawn_target_width,
            );
            // Width=1: drain inline on the pump thread reusing `process_shared_index` (already
            // warmed for calibration + floor runner). Spawning a worker thread duplicates the
            // whole-tree index on a second thread-local cache — ~2× retention that OOM'd CI
            // batch-2 discovery (runs 29372308568 / 29373433928). Cross-worker store arms only
            // when plural workers run (below).
            //
            // This width read is deliberately SAMPLED ONCE, and at width 1 that makes the
            // window an absorbing state for this pool: the only path that grows it (a slot
            // completion) lives past the branch below, so the governor's AIMD controller is
            // not reachable from the corpus. That is a real defect in the controller — and
            // un-latching it is nonetheless a MEASURED LOSS, so the latch stays until the
            // cost it hides is gone. Same branch, same 621 entry-groups, same .rs-forced
            // whole-tree path: serial 11.75min GREEN (CI 29707161743 — max_width_reached=1,
            // admissions=1, peak 6.97 GB) vs un-latched 47min+ without finishing (CI
            // 29714863168), vs un-latched with per-unit window growth OOM-killed at
            // 101.6 GB in 11min (CI 29710324768).
            //
            // The reason is Amdahl, not a bug: a worker's front cost is its own whole-tree
            // index build (~10.7 GB, minutes) and the entire corpus is ~12 minutes of work,
            // so every added worker costs more setup than the parallelism it buys. Width is
            // not worth reaching for while the index is per-worker; the governor's job here
            // is to be correct when it IS reachable — see `CompletionKind` in
            // `memory_governor`, where the window tracks landed worker cost and never the
            // unit-completion rate.
            // 🟡 dissolve-on: Rc→Arc retires the width gate — sharing the index removes the
            // per-worker front cost, which is the thing that makes width unprofitable. Priced
            // FIRST by the share spike (docs/plans/cross-worker-typecheck-share-design.md §9
            // open decision 2), because that design's §7 warns a shared store also INCREASES
            // co-resident retention: the win is a crossover in width, not a given.
            if spawn_target_width <= 1 {
                eprintln!(
                    "run_discovery_corpus: width=1 inline drain — reusing process_shared_index (no worker duplicate index)"
                );
                eprintln!(
                    "run_discovery_corpus: cross_worker_store withheld (governor target_width={spawn_target_width}) — per-index typed cache until width > 1"
                );
                let style = ShardStyle {
                    shard_id: 0,
                    shard_count: 1,
                    color: floor_color,
                    stream: floor_stream,
                };
                let mut summaries = Vec::new();
                for group_indices in groups {
                    let group_rows: Vec<DiscoveryRow> =
                        group_indices.iter().map(|&i| rows[i].clone()).collect();
                    summaries.push(run_discovery_rows(
                        &group_rows,
                        &index,
                        execution_mode,
                        options.node_frontier_selection,
                        &changed_paths,
                        &diff_edits,
                        floor_runner_ctx.as_ref(),
                        whole_tree_published_keys.clone(),
                        options.witness_budget_policy(),
                        style,
                    )?);
                }
                return Ok(attach_deferred_discovery_rows(
                    merge_discovery_summaries(summaries),
                    deferred_rows,
                ));
            }
            // Process-scoped typed store shell — populated only when plural workers run
            // (target_width > 1). At width=1 the serde byte store adds retention without
            // cross-worker benefit and breaks the CI memory budget (design §7; OOM
            // 29349125185 / 29371206526). 🟡 dissolve-on: Rc→Arc retires the gate.
            let cross_worker_store = new_shared_typecheck_caches();
            if floor_stream && spawn_target_width > 1 {
                eprintln!(
                    "{} [affected-set] streaming run-witnesses live across the adaptive worker pool (target width {}; ▎shard N, one color each)",
                    floor_ts(),
                    spawn_target_width,
                );
            }
            let queue: std::sync::Arc<Mutex<VecDeque<Vec<DiscoveryRow>>>> =
                std::sync::Arc::new(Mutex::new(
                    groups
                        .into_iter()
                        .map(|g| g.iter().map(|&i| rows[i].clone()).collect())
                        .collect(),
                ));
            let abort = std::sync::Arc::new(AtomicBool::new(false));
            let source_roots_owned = source_roots.to_vec();
            let selection_for_workers = options.node_frontier_selection;
            let budget_policy_for_workers = options.witness_budget_policy();
            let mut handles = Vec::new();
            let mut worker_ordinal: usize = 0;
            loop {
                if abort.load(Ordering::SeqCst) || queue.lock().unwrap().is_empty() {
                    break;
                }
                match governor.try_admit() {
                    crate::memory_governor::AdmitDecision::Admit { .. } => {}
                    crate::memory_governor::AdmitDecision::Hold(_) => {
                        std::thread::sleep(std::time::Duration::from_millis(150));
                        continue;
                    }
                }
                let queue_for_worker = queue.clone();
                let abort_for_worker = abort.clone();
                let governor_for_worker = governor.clone();
                let roots = source_roots_owned.clone();
                let seeds = diff_edits.clone();
                let paths = changed_paths.clone();
                let keys = whole_tree_published_keys.clone();
                let spawn_target_width = governor.current_target_width();
                let cross_worker_store_for_worker = if spawn_target_width > 1 {
                    Some(cross_worker_store.clone())
                } else {
                    None
                };
                if worker_ordinal == 0 && spawn_target_width <= 1 {
                    eprintln!(
                "run_discovery_corpus: cross_worker_store withheld (governor target_width={spawn_target_width}) — per-index typed cache until width > 1"
            );
                }
                // Narration style for this worker: shard_id = spawn ordinal (a stable hue in the
                // interleaved stream); spawn-time target width > 1 shows the ▎shard tag — a width-1
                // admission window has no interleaving to disambiguate.
                let style = ShardStyle {
                    shard_id: worker_ordinal,
                    shard_count: governor.current_target_width(),
                    color: floor_color,
                    stream: floor_stream,
                };
                worker_ordinal += 1;
                handles.push(std::thread::spawn(
                    move || -> Result<Vec<DiscoverySummary>, String> {
                        // The slot was granted by try_admit on the pump thread; the guard owns the
                        // matching release so a panicking worker cannot wedge admissions.
                        let mut slot = crate::memory_governor::AdmittedSlot::from_admitted(
                            governor_for_worker.clone(),
                        );
                        // Process-scoped typed_module_cache when governor width > 1; private cold
                        // index at width=1 (CI budget — cross-worker-typecheck-share-design §7).
                        let index = match cross_worker_store_for_worker {
                            Some(store) => {
                                build_multi_entry_index_with_shared_caches(&roots, store)
                            }
                            None => build_multi_entry_index(&roots),
                        };
                        let runner = if selection_for_workers != NodeFrontierSelectionMode::Off {
                            match resolve_entry_with_index(&index, FLOOR_RUNNER_ENTRY) {
                                Ok((graph, source_indices)) => {
                                    Some(make_eval_context(&graph, source_indices, execution_mode))
                                }
                                Err(msg) => {
                                    abort_for_worker.store(true, Ordering::SeqCst);
                                    return Err(format!(
                                        "floor runner resolve failed in worker ({msg}) — declared \
                                 affected-set machinery must resolve; no silent \
                                 run-everything fallback"
                                    ));
                                }
                            }
                        } else {
                            None
                        };
                        // The front-loaded admission cost (index build + runner resolve) has
                        // landed and is visible to the creep signals: unblock admission pacing.
                        slot.note_first_cost_paid();
                        let mut worker_summaries = Vec::new();
                        loop {
                            // Multiplicative decrease drains here: a worker between groups retires
                            // when concurrency sits above the (possibly just-halved) window.
                            if governor_for_worker.should_retire()
                                || abort_for_worker.load(Ordering::SeqCst)
                            {
                                break;
                            }
                            let Some(group_rows) = queue_for_worker.lock().unwrap().pop_front()
                            else {
                                break;
                            };
                            match run_discovery_rows(
                                &group_rows,
                                &index,
                                execution_mode,
                                selection_for_workers,
                                &paths,
                                &seeds,
                                runner.as_ref(),
                                keys.clone(),
                                budget_policy_for_workers,
                                style,
                            ) {
                                Ok(summary) => {
                                    worker_summaries.push(summary);
                                    slot.note_unit_complete();
                                }
                                Err(e) => {
                                    abort_for_worker.store(true, Ordering::SeqCst);
                                    return Err(e);
                                }
                            }
                        }
                        Ok(worker_summaries)
                    },
                ));
            }
            let mut summaries = Vec::new();
            let mut first_err: Option<String> = None;
            for handle in handles {
                match handle
                    .join()
                    .map_err(|_| "discovery corpus worker thread panicked".to_string())
                {
                    Ok(Ok(worker_summaries)) => summaries.extend(worker_summaries),
                    Ok(Err(e)) | Err(e) => first_err = first_err.or(Some(e)),
                }
            }
            if let Some(e) = first_err {
                return Err(e);
            }
            // The pump exits when the queue is empty OR on abort; with no error the queue must be
            // fully drained (workers only exit early on retire/abort, and the pump re-admits while
            // items remain), so an undrained queue here is a scheduler bug — refuse, never under-run.
            let leftover = queue.lock().unwrap().len();
            if leftover > 0 {
                return Err(format!(
            "adaptive discovery pool exited with {leftover} undrained entry-group(s) and no \
             worker error — scheduler invariant violated; refusing a partial corpus"
        ));
            }
            Ok(attach_deferred_discovery_rows(
                merge_discovery_summaries(summaries),
                deferred_rows,
            ))
        }
    };
}

fn attach_deferred_discovery_rows(
    mut summary: DiscoverySummary,
    deferred_rows: Vec<DeferredDiscoveryRow>,
) -> DiscoverySummary {
    summary.deferred_rows = deferred_rows;
    summary
}

/// Contiguous same-entry row groups, order-preserving: the unit a pool worker pulls (rows
/// sharing an entry resolve once against the worker's index).
fn entry_row_groups(rows: &[DiscoveryRow]) -> Vec<Vec<usize>> {
    let mut entry_groups: Vec<Vec<usize>> = Vec::new();
    let mut current: Vec<usize> = Vec::new();
    let mut current_entry: Option<&str> = None;
    for (i, row) in rows.iter().enumerate() {
        if current_entry != Some(row.entry.as_str()) {
            if !current.is_empty() {
                entry_groups.push(current);
            }
            current = vec![i];
            current_entry = Some(&row.entry);
        } else {
            current.push(i);
        }
    }
    if !current.is_empty() {
        entry_groups.push(current);
    }
    entry_groups
}

fn merge_discovery_summaries(summaries: Vec<DiscoverySummary>) -> DiscoverySummary {
    let mut merged = DiscoverySummary {
        total: 0,
        passed: 0,
        skipped: 0,
        deferred_rows: Vec::new(),
        predicted_unaffected: Vec::new(),
        divergences: Vec::new(),
        failures: Vec::new(),
        witness_outcomes: Vec::new(),
        entry_resolve_receipts: Vec::new(),
        total_resolve_nanos: 0,
        total_stage_nanos: ResolveStageNanos::default(),
        performance_receipts: Vec::new(),
        total_measured_nanos: 0,
        roster_closure_nodes: 0,
    };
    for summary in summaries {
        merged.total += summary.total;
        merged.passed += summary.passed;
        merged.skipped += summary.skipped;
        merged
            .predicted_unaffected
            .extend(summary.predicted_unaffected);
        merged.divergences.extend(summary.divergences);
        merged.failures.extend(summary.failures);
        merged.witness_outcomes.extend(summary.witness_outcomes);
        merged
            .entry_resolve_receipts
            .extend(summary.entry_resolve_receipts);
        merged.total_resolve_nanos += summary.total_resolve_nanos;
        merged
            .total_stage_nanos
            .accumulate(&summary.total_stage_nanos);
        merged
            .performance_receipts
            .extend(summary.performance_receipts);
        merged.total_measured_nanos += summary.total_measured_nanos;
        // Max, not sum: shards share the std/spec prefix, so summing would double-count it. The
        // heaviest single shard's closure is the number the per-shard memory peak is a function of.
        merged.roster_closure_nodes = merged
            .roster_closure_nodes
            .max(summary.roster_closure_nodes);
    }
    merged
}

// SCAFFOLD (§7 hand-Rust shrink-to-zero, dissolution named): the floor-observability cluster
// below — `floor_verbose` / `floor_ts` / `floor_stream_enabled` / `floor_color_enabled`,
// `ShardStyle`, and `eprintln_affected_set_categorization` — is seed-side NARRATION wrapped
// around the existing affected-set selection. It adds no selection authority: the fail-closed
// skip/run decision, its refusals, and the `DiscoverySummary` counts are unchanged (see
// `run_discovery_rows`); these helpers only choose how the already-decided run is printed. They
// live in Rust because the v1 evaluator narrates its own floor walk (the same seed-side reason as
// `phase_profile.rs` and `GUNBC_FLOOR_GANTT`). The *rendering* they emit is the same class of
// output already migrating into `dag/gunbc/ci_render.dag` (the timing histogram + slowest-witness
// rollup render there today). Full dissolution: when v2 emit-host owns floor observability — a
// `.dag` floor-event carrier a witness consumes by execution, the retirement event shared with
// `phase_profile.rs` (`docs/plans/realization-measurement-loop.md` Phase 0) and the fractal Gantt
// (`docs/plans/ci-floor-fractal-gantt.md` § dissolution) — this narration collapses into that
// carrier and is deleted. Until then it is counted seed Rust, not a new authority; do not accrete
// further floor logic here — extend the `.dag` render/observability surface instead.

/// Per-witness selection detail (the `SKIP`/`SKIP-RESOLVE`/`PREDICT` lines and the
/// per-resolve `[binding-fork-ledger]` census) is opt-in. The default floor output is the
/// upfront `[affected-set]` categorization plus the final `[measurement]` tally — a wide
/// corpus otherwise streams one skip line per unaffected witness (~1.7k lines), drowning the
/// signal. The counts survive on `DiscoverySummary`; only the per-row narration is gated.
fn floor_verbose() -> bool {
    std::env::var("GUNBC_FLOOR_VERBOSE")
        .map(|v| v == "1" || v == "true")
        .unwrap_or(false)
}

/// Wall-clock stamp `HH:MM:SS.mmm` (UTC) prefixed on the live floor lines so the stream reads as
/// a timeline and correlates with CI's wall-clock log — dependency-free (no chrono): seconds
/// since the epoch reduced to a 24h clock, plus millis.
fn floor_ts() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs();
    let millis = now.subsec_millis();
    let h = (secs / 3600) % 24;
    let m = (secs / 60) % 60;
    let s = secs % 60;
    format!("{h:02}:{m:02}:{s:02}.{millis:03}")
}

/// One upfront line categorizing the corpus before any witness runs — the operator-facing
/// "N skipped, M impacted, running X" read of the affected-set selection. The skip count here
/// is the cheap import-closure disposition (entry-grain, no resolve); the finer per-node
/// frontier decision runs the affected closure down further, so `[measurement]` at the end
/// reports the exact ran/skipped tally. Print-only: the authoritative decision (and its
/// fail-closed refusal on a provenance gap) still happens per-shard in `run_discovery_rows`.
fn eprintln_affected_set_categorization(
    selection: NodeFrontierSelectionMode,
    rows: &[DiscoveryRow],
    index: &MultiEntryIndex,
    diff_edits: &FloorDiffEdits,
) {
    let total = rows.len();
    let entries = rows
        .iter()
        .map(|r| r.entry.as_str())
        .collect::<HashSet<&str>>()
        .len();
    let ts = floor_ts();
    match selection {
        NodeFrontierSelectionMode::Off => {
            eprintln!(
                "{ts} [affected-set] selection off — running all {total} witness(es) across {entries} entr(y/ies)"
            );
        }
        NodeFrontierSelectionMode::PredictOnly => {
            eprintln!(
                "{ts} [affected-set] predict-only — running all {total} witness(es) cold across {entries} entr(y/ies); node-frontier predictions recorded, divergences counted"
            );
        }
        NodeFrontierSelectionMode::Applied => {
            let declared_paths = index.module_graph_facts.declared_repo_paths();
            let touched: Vec<String> = diff_edits.touched_entry_files.iter().cloned().collect();
            match discovery_entry_fast_skip_without_resolve(
                rows,
                &index.module_graph_facts,
                &declared_paths,
                &touched,
                diff_edits,
            ) {
                Ok(fast) => {
                    let skipped = rows.iter().filter(|r| fast.contains(&r.entry)).count();
                    let candidates = total - skipped;
                    eprintln!(
                        "{ts} [affected-set] {total} witness(es) across {entries} entr(y/ies) · {skipped} unaffected (import-closure, skipped without resolve) · {candidates} in the affected closure (resolving to decide node-frontier)"
                    );
                }
                Err(e) => {
                    eprintln!(
                        "{ts} [affected-set] {total} witness(es) across {entries} entr(y/ies) · upfront import-closure categorization unavailable ({e}); per-shard selection is authoritative"
                    );
                }
            }
        }
    }
}

/// Live realization view: stream affected witnesses to stderr as they finish, one colored
/// line per shard, so a run reads as "the affected set unrolling in real time" rather than a
/// silent wait then a summary. On by default (opt out with `GUNBC_FLOOR_QUIET=1`); color
/// auto-detected (a terminal or GitHub Actions), `NO_COLOR` honored, `GUNBC_FLOOR_COLOR=1`
/// forces it on. Only RUN witnesses reach the stream — skips are counted, not narrated.
fn floor_stream_enabled() -> bool {
    !std::env::var("GUNBC_FLOOR_QUIET")
        .map(|v| v == "1" || v == "true")
        .unwrap_or(false)
}

fn floor_color_enabled() -> bool {
    if std::env::var("NO_COLOR").is_ok() {
        return false;
    }
    if std::env::var("GUNBC_FLOOR_COLOR")
        .map(|v| v == "1" || v == "true")
        .unwrap_or(false)
    {
        return true;
    }
    use std::io::IsTerminal;
    std::io::stderr().is_terminal()
        || std::env::var("GITHUB_ACTIONS")
            .map(|v| v == "true")
            .unwrap_or(false)
}

#[derive(Clone, Copy)]
struct ShardStyle {
    shard_id: usize,
    /// Number of concurrent shards in this run. When it is 1 there is no parallelism to
    /// distinguish, so the `s{id}` shard tag is dropped (it only reads as noise — the reason the
    /// operator asked "what does s0 mean?"). The tag returns, colored, only when shards run wide.
    shard_count: usize,
    color: bool,
    stream: bool,
}

impl ShardStyle {
    /// Distinct hue per concurrent shard so the interleaved stream reads as parallelism. Green
    /// and red are reserved for the pass/fail glyph, so the label palette avoids them.
    fn shard_color_code(self) -> &'static str {
        const PALETTE: [&str; 6] = [
            "\x1b[96m", // bright cyan
            "\x1b[94m", // bright blue
            "\x1b[95m", // bright magenta
            "\x1b[93m", // bright yellow
            "\x1b[36m", // cyan
            "\x1b[35m", // magenta
        ];
        PALETTE[self.shard_id % PALETTE.len()]
    }

    /// Colored `▎shard N` tag, or empty when the run is single-shard (nothing to disambiguate).
    fn shard_tag(self) -> String {
        if self.shard_count <= 1 {
            return String::new();
        }
        if self.color {
            format!(
                "{}▎shard {}\x1b[0m ",
                self.shard_color_code(),
                self.shard_id
            )
        } else {
            format!("[shard {}] ", self.shard_id)
        }
    }

    fn stream_witness(self, function: &str, entry: &str, wall_nanos: u128, passed: bool) {
        if !self.stream {
            return;
        }
        let ms = wall_nanos as f64 / 1.0e6;
        let ts = floor_ts();
        let tag = self.shard_tag();
        if self.color {
            let glyph = if passed {
                "\x1b[32m✓\x1b[0m"
            } else {
                "\x1b[31m✗\x1b[0m"
            };
            eprintln!(
                "\x1b[2m{ts}\x1b[0m {tag}{glyph} {function} \x1b[2m({entry})\x1b[0m {ms:.1}ms"
            );
        } else {
            let glyph = if passed { "PASS" } else { "FAIL" };
            eprintln!("{ts} {tag}{glyph} {function} ({entry}) {ms:.1}ms");
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn run_discovery_rows(
    rows: &[DiscoveryRow],
    index: &MultiEntryIndex,
    execution_mode: v1_interpreter::ExecutionMode,
    selection: NodeFrontierSelectionMode,
    changed_paths: &[String],
    diff_edits: &FloorDiffEdits,
    floor_runner_ctx: Option<&v1_interpreter::InterpContext>,
    whole_tree_published_keys: Option<std::collections::HashSet<String>>,
    budgets: WitnessBudgetPolicy,
    style: ShardStyle,
) -> Result<DiscoverySummary, String> {
    let mut summary = DiscoverySummary {
        total: rows.len(),
        passed: 0,
        skipped: 0,
        deferred_rows: Vec::new(),
        predicted_unaffected: Vec::new(),
        divergences: Vec::new(),
        failures: Vec::new(),
        witness_outcomes: Vec::with_capacity(rows.len()),
        entry_resolve_receipts: Vec::new(),
        total_resolve_nanos: 0,
        total_stage_nanos: ResolveStageNanos::default(),
        performance_receipts: Vec::new(),
        total_measured_nanos: 0,
        roster_closure_nodes: 0,
    };
    let skip_enabled = selection != NodeFrontierSelectionMode::Off;
    // This shard's union closure, accumulated from the graphs it resolves. Seeded with the prefix
    // context: the floor runner closure is resolved once per shard and its modules are resident
    // for the shard's lifetime, so they are part of the memory this count is paired against
    // (the entry-selection prefix context was retired with the import-closure reground — the
    // entry_file_touched decision now reads the module-graph facts, no interpreter context).
    // Row closures fold in below as each entry resolves.
    let mut closure_modules: HashSet<String> = HashSet::new();
    for prefix_ctx in [floor_runner_ctx].into_iter().flatten() {
        collect_typed_module_names(
            prefix_ctx.modules.iter().cloned(),
            &prefix_ctx.source_indices,
            &mut closure_modules,
        );
    }
    // Existence set for the entry_file_touched refuse-vs-answer decision, built once per shard.
    let module_graph_declared_paths = index.module_graph_facts.declared_repo_paths();
    let mut current_entry: Option<String> = None;
    let mut current_closure_subject: Option<String> = None;
    let mut ctx: Option<v1_interpreter::InterpContext> = None;
    let mut current_entry_touches = true;
    let mut current_entry_frontier_nodes: Vec<v1_interpreter::Value> = Vec::new();
    let mut current_entry_file_touched = true;
    let mut current_entry_runtime_dependency_touched = true;
    let touched_entry_paths: Vec<String> = diff_edits.touched_entry_files.iter().cloned().collect();
    let pool_roots = witness_layer_roots();
    let whole_tree_published_keys = whole_tree_published_keys.map(Rc::new);
    let entry_fast_skip = if selection == NodeFrontierSelectionMode::Applied {
        discovery_entry_fast_skip_without_resolve(
            rows,
            &index.module_graph_facts,
            &module_graph_declared_paths,
            &touched_entry_paths,
            diff_edits,
        )?
    } else {
        HashSet::new()
    };
    if !entry_fast_skip.is_empty() && floor_verbose() {
        eprintln!(
            "run_discovery_corpus: skip-before-resolve fast path for {} entr(y/ies) (import-closure unaffected, no declaration edits, no data-item edits in closure, no host-scaffold)",
            entry_fast_skip.len()
        );
    }
    for row in rows {
        // Applied only: PredictOnly must resolve + run cold and record via the post-resolve
        // would_skip path (falsifier semantics — docs/plans/affected-set-differential-falsifier.md).
        if selection == NodeFrontierSelectionMode::Applied && entry_fast_skip.contains(&row.entry) {
            refuse_reads_live_tree_selection_skip(&row, "skip-before-resolve-fast-path")?;
            if current_entry.as_deref() != Some(row.entry.as_str()) {
                augment_closure_modules_from_import_facts(
                    &row.entry,
                    &index.module_graph_facts,
                    &mut closure_modules,
                );
                current_entry = Some(row.entry.clone());
                current_entry_touches = false;
                current_entry_file_touched = false;
                current_entry_runtime_dependency_touched = false;
                current_entry_frontier_nodes.clear();
                current_closure_subject = None;
                ctx = None;
            }
            summary.skipped += 1;
            if floor_verbose() {
                eprintln!(
                    "SKIP [assumed-green node-frontier] {} ({})",
                    row.function, row.entry
                );
            }
            continue;
        }
        if current_entry.as_deref() != Some(row.entry.as_str()) {
            if entry_eligible_for_discovery_skip_before_resolve(
                skip_enabled,
                row.reads_live_tree,
                &row.entry,
                &index.module_graph_facts,
                &module_graph_declared_paths,
                changed_paths,
                diff_edits,
            )? {
                if floor_verbose() {
                    eprintln!(
                        "SKIP-RESOLVE [unaffected import-closure] {} (cold entry resolve elided)",
                        row.entry
                    );
                }
                collect_import_closure_module_names_from_facts(
                    &row.entry,
                    &index.module_graph_facts,
                    &mut closure_modules,
                );
                ctx = None;
                current_closure_subject = None;
                current_entry_frontier_nodes.clear();
                current_entry_touches = false;
                current_entry_file_touched = false;
                current_entry_runtime_dependency_touched = false;
                current_entry = Some(row.entry.clone());
            } else {
                let resolved = resolve_discovery_entry_for_corpus_row(
                    index,
                    &row.entry,
                    execution_mode,
                    whole_tree_published_keys.clone(),
                    skip_enabled,
                    diff_edits,
                    &touched_entry_paths,
                    &module_graph_declared_paths,
                    &mut closure_modules,
                )?;
                summary.total_resolve_nanos += resolved.resolve_nanos;
                summary.total_stage_nanos.accumulate(&resolved.stage_nanos);
                summary.entry_resolve_receipts.push(EntryResolveReceipt {
                    entry: row.entry.clone(),
                    closure_subject: resolved.closure_subject.clone(),
                    resolve_nanos: resolved.resolve_nanos,
                    stage_nanos: resolved.stage_nanos,
                });
                current_closure_subject = Some(resolved.closure_subject);
                current_entry_frontier_nodes = resolved.frontier_nodes;
                current_entry_touches = resolved.touches_frontier;
                current_entry_file_touched = resolved.entry_file_touched;
                current_entry_runtime_dependency_touched =
                    resolved.entry_runtime_dependency_touched;
                ctx = Some(resolved.ctx);
                if let Some(c) = ctx.as_ref() {
                    c.set_witness_eval_budget(budgets.cpu_eval_budget_ms);
                    c.set_witness_wall_budget(budgets.wet_receipt_wall_budget_ms);
                }
                current_entry = Some(row.entry.clone());
            }
        }
        let function_edited = skip_enabled
            && diff_edits.edited_test_fns.iter().any(|(file, func)| {
                diff_file_matches_entry(file, &row.entry) && func == &row.function
            });
        let entry_file_touched = skip_enabled && current_entry_file_touched;
        let runtime_data_dependency_touched =
            skip_enabled && current_entry_runtime_dependency_touched;
        let would_skip = if skip_enabled {
            match floor_runner_ctx {
                Some(runner_ctx) => {
                    let skip = call_floor_row_would_skip(
                        runner_ctx,
                        row.reads_live_tree,
                        changed_paths,
                        &current_entry_frontier_nodes,
                        current_entry_touches,
                        function_edited,
                        entry_file_touched,
                        runtime_data_dependency_touched,
                    );
                    match skip {
                        Ok(skip) => skip,
                        Err(msg) => {
                            return Err(format!(
                                "floor would_skip failed for {} ({}): {msg} — declared \
                                 selection machinery must evaluate; no silent \
                                 run-everything fallback",
                                row.function, row.entry
                            ));
                        }
                    }
                }
                None => false,
            }
        } else {
            false
        };
        if would_skip {
            refuse_reads_live_tree_selection_skip(&row, "node-frontier-selection")?;
            match selection {
                NodeFrontierSelectionMode::Applied => {
                    summary.skipped += 1;
                    if floor_verbose() {
                        eprintln!(
                            "SKIP [assumed-green node-frontier] {} ({})",
                            row.function, row.entry
                        );
                    }
                    continue;
                }
                NodeFrontierSelectionMode::PredictOnly => {
                    // Falsifier semantics: record the prediction and run the row cold anyway.
                    summary
                        .predicted_unaffected
                        .push((row.entry.clone(), row.function.clone()));
                    if floor_verbose() {
                        eprintln!(
                            "PREDICT [unaffected node-frontier] {} ({})",
                            row.function, row.entry
                        );
                    }
                }
                // would_skip is only computed when selection is enabled.
                NodeFrontierSelectionMode::Off => {}
            }
        }
        if ctx.is_none() {
            let resolved = resolve_discovery_entry_for_corpus_row(
                index,
                &row.entry,
                execution_mode,
                whole_tree_published_keys.clone(),
                skip_enabled,
                diff_edits,
                &touched_entry_paths,
                &module_graph_declared_paths,
                &mut closure_modules,
            )?;
            summary.total_resolve_nanos += resolved.resolve_nanos;
            summary.total_stage_nanos.accumulate(&resolved.stage_nanos);
            summary.entry_resolve_receipts.push(EntryResolveReceipt {
                entry: row.entry.clone(),
                closure_subject: resolved.closure_subject.clone(),
                resolve_nanos: resolved.resolve_nanos,
                stage_nanos: resolved.stage_nanos,
            });
            current_closure_subject = Some(resolved.closure_subject);
            current_entry_frontier_nodes = resolved.frontier_nodes;
            current_entry_touches = resolved.touches_frontier;
            current_entry_file_touched = resolved.entry_file_touched;
            current_entry_runtime_dependency_touched = resolved.entry_runtime_dependency_touched;
            ctx = Some(resolved.ctx);
            if let Some(c) = ctx.as_ref() {
                c.set_witness_eval_budget(budgets.cpu_eval_budget_ms);
                c.set_witness_wall_budget(budgets.wet_receipt_wall_budget_ms);
            }
        }
        let ctx_ref = ctx.as_ref().expect("ctx set above");
        let closure_subject = current_closure_subject
            .as_deref()
            .expect("closure subject set above");
        set_phase(
            FloorPhase::Eval,
            &format!("{}::{}", row.entry, row.function),
        );
        let (outcome, receipt) = run_claim_measured(ctx_ref, closure_subject, &row.function);
        let wall_nanos = receipt.wall_nanos;
        summary.total_measured_nanos += wall_nanos;
        summary.performance_receipts.push(receipt);
        summary.witness_outcomes.push(DiscoveryWitnessOutcome {
            entry: row.entry.clone(),
            function: row.function.clone(),
            outcome: outcome.clone(),
        });
        style.stream_witness(
            &row.function,
            &row.entry,
            wall_nanos,
            matches!(outcome, ClaimOutcome::Pass),
        );
        if selection == NodeFrontierSelectionMode::PredictOnly
            && would_skip
            && !matches!(outcome, ClaimOutcome::Pass)
        {
            // The red itself already fails the batch through the failure channel below;
            // this line is the ATTRIBUTION receipt — a missing selection edge, counted.
            let line = format!(
                "DIVERGENCE [affected-set-falsifier] {} ({}) predicted=unaffected \
                 actual=red class=node-frontier",
                row.function, row.entry
            );
            eprintln!("{line}");
            summary.divergences.push(line);
        }
        match outcome {
            ClaimOutcome::Pass => summary.passed += 1,
            ClaimOutcome::Fail => summary.failures.push(format!(
                "{} ({}) returned Bool(false)",
                row.function, row.entry
            )),
            ClaimOutcome::NotBool { got } => summary.failures.push(format!(
                "{} ({}) returned `{}`, not Bool",
                row.function, row.entry, got
            )),
            ClaimOutcome::RuntimeError { message } => summary.failures.push(format!(
                "{} ({}) runtime error: {}",
                row.function, row.entry, message
            )),
        }
    }
    // Per-shard input-size receipt: distinct modules in THIS shard's union closure, counted from the
    // graphs resolved above rather than from the thread's typecheck-miss counter (see the field doc
    // on `DiscoverySummary::roster_closure_nodes` for why the counter is not bounded to this window).
    summary.roster_closure_nodes = closure_modules.len();
    Ok(summary)
}

#[cfg(test)]
mod floor_skip_frontier_tests {
    use super::{
        build_multi_entry_index, entry_touches_rerun_frontier, floor_diff_edits_from_diff_text,
        list_value_from_vec, parse_unified_diff_added_paths, parse_unified_diff_changed_new_lines,
        parse_unified_diff_line_ranges, rerun_frontier_nodes_for_entry, scan_test_decl_lines,
        FileLineRange,
    };
    use crate::v1_compiler_infer_items::{item_kind, ItemKind, ResolvedGraph};
    use crate::v1_interpreter::ExecutionMode;
    use crate::v1_std_core::{authored_name_at, byte_to_line_col};
    use im::HashMap;
    use std::collections::HashSet;
    use std::path::PathBuf;

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(3)
            .expect("workspace root")
            .to_path_buf()
    }

    fn fixture_path() -> String {
        "src/v2/test/fixture/floor_skip/node_precise_discriminator_test.dag".to_string()
    }

    fn data_item_line(
        fixture: &str,
        source_indices: &std::rc::Rc<
            HashMap<String, std::rc::Rc<crate::v1_std_core::NewlineIndex>>,
        >,
        graph: &std::rc::Rc<ResolvedGraph>,
        name: &str,
    ) -> i64 {
        for module in graph.modules.iter() {
            for item in module.items.iter() {
                if item_kind(item.clone()) != ItemKind::DataItem {
                    continue;
                }
                if authored_name_at(source_indices.clone(), item.clone()) != name {
                    continue;
                }
                let span = &item.span;
                let index = source_indices.get(&span.file).expect("newline index");
                return byte_to_line_col(index.clone(), span.start).line;
            }
        }
        panic!("data item `{name}` not found in {fixture}");
    }

    fn unified_diff_for_line(file: &str, line: i64) -> String {
        format!(
            "diff --git a/{file} b/{file}\n--- a/{file}\n+++ b/{file}\n@@ -{line},0 +{line},1 @@\n+// node-precise touch\n"
        )
    }

    #[test]
    fn parse_unified_diff_extracts_new_side_line_ranges() {
        let diff = "\
diff --git a/src/v2/lens/affected_set.dag b/src/v2/lens/affected_set.dag
--- a/src/v2/lens/affected_set.dag
+++ b/src/v2/lens/affected_set.dag
@@ -100,0 +101,3 @@
+line1
+line2
+line3
";
        let ranges = parse_unified_diff_line_ranges(diff);
        let file = "src/v2/lens/affected_set.dag";
        assert_eq!(
            ranges.get(file),
            Some(&vec![FileLineRange {
                start: 101,
                end: 103
            }])
        );
    }

    #[test]
    fn non_dag_only_diff_is_structural_empty_frontier_not_refusal() {
        // A present diff whose changed paths are all non-.dag (a Rust/TOML/doc edit) is a
        // structural-∅ for the .dag frontier -- no .dag nodes are declared, so nothing is
        // attributable and every .dag row takes the not-affected skip, exactly as an empty diff
        // does. It is NOT an ignorance state (the only ignorance state is a failed git-diff
        // observation, refused upstream). RED CONTROL: before the arity fix this returned
        // Err("non-.dag file changed with no .dag paths in diff") and reddened the
        // discovery-corpus batch for every pure-.rs PR.
        // NB: use a non-src/v1 path -- #6269 eagerly builds the v1-attribution index for any
        // src/v1/ path, which reads the real workspace tree and cannot run from the unit-test
        // cwd. The structural-∅ arm is path-agnostic, so this isolates exactly that behavior.
        let index = build_multi_entry_index(&[]);
        let rs_only_diff = unified_diff_for_line("crates/widget/src/lib.rs", 42);
        let edits = floor_diff_edits_from_diff_text(&index, &rs_only_diff)
            .expect("non-.dag-only diff must be a nominal empty frontier, not a refusal");
        assert!(
            edits.overlapping_data_items.is_empty()
                && edits.edited_test_fns.is_empty()
                && edits.touched_entry_files.is_empty(),
            "non-.dag-only diff must produce an empty .dag frontier, got {edits:?}"
        );
    }

    #[test]
    fn parse_unified_diff_changed_new_lines_includes_deletions() {
        let diff = "\
diff --git a/src/v2/lens/affected_set.dag b/src/v2/lens/affected_set.dag
--- a/src/v2/lens/affected_set.dag
+++ b/src/v2/lens/affected_set.dag
@@ -42,2 +42,0 @@
-removed_a
-removed_b
";
        let changed = parse_unified_diff_changed_new_lines(diff);
        let file = "src/v2/lens/affected_set.dag";
        // Deletion-only hunk `+42,0`: the gap sits between new lines 42 and 43;
        // following-line semantics attribute 43 (see the parser's `+L,0` note).
        assert_eq!(changed.get(file), Some(&HashSet::from([43])));
    }

    #[test]
    fn parse_unified_diff_added_paths_detects_new_files() {
        let diff = "\
diff --git a/dag/tools/new_transport.dag b/dag/tools/new_transport.dag
new file mode 100644
--- /dev/null
+++ b/dag/tools/new_transport.dag
@@ -0,0 +1,3 @@
+module tools.new_transport
+
+fn run() -> Bool { true }
";
        let added = parse_unified_diff_added_paths(diff);
        assert!(added.contains("dag/tools/new_transport.dag"));
    }

    #[test]
    fn scan_test_decl_lines_pairs_names_with_1_based_lines() {
        let source = "module m\n\ndata d: Int = 1\n\ntest fn witness_a() -> Bool { true }\n\ntest data witness_b: Int = 2\n";
        let pairs = scan_test_decl_lines(source);
        assert_eq!(
            pairs,
            vec![("witness_a".to_string(), 5), ("witness_b".to_string(), 7)]
        );
    }

    #[test]
    fn live_tree_disposition_declared_substrate_only_is_selection_eligible() {
        let source = "module m\n\nimport v2.std.live_tree { LiveTreeDisposition, SubstrateInputsOnly }\n\ndata live_tree_disposition: LiveTreeDisposition = SubstrateInputsOnly\n\ntest fn pure_holds() -> Bool { true }\n";
        assert!(!super::parse_entry_live_tree_disposition("m_test.dag", source).unwrap());
    }

    #[test]
    fn live_tree_disposition_declared_reads_live_tree_is_live() {
        let source = "module m\n\ndata live_tree_disposition: LiveTreeDisposition = ReadsLiveTree\n\ntest fn live_holds() -> Bool { true }\n";
        assert!(super::parse_entry_live_tree_disposition("m_test.dag", source).unwrap());
    }

    #[test]
    fn live_tree_disposition_undeclared_defaults_live_fail_closed() {
        // Undeclared = ReadsLiveTree even when the entry text LOOKS pure — a row
        // must declare SubstrateInputsOnly to become selection-eligible.
        let source = "module m\n\ntest fn pure_holds() -> Bool { true }\n";
        assert!(super::parse_entry_live_tree_disposition("m_test.dag", source).unwrap());
    }

    #[test]
    fn live_tree_disposition_sibling_named_row_is_not_the_declaration() {
        // `live_tree_disposition_note` shares the prefix but is a different decl —
        // it must neither classify the entry nor trip the malformed-row refusal.
        let source = "module m\n\ndata live_tree_disposition_note: String = \"doc row\"\n\ndata live_tree_disposition: LiveTreeDisposition = SubstrateInputsOnly\n";
        assert!(!super::parse_entry_live_tree_disposition("m_test.dag", source).unwrap());
        let note_only = "module m\n\ndata live_tree_disposition_note: String = \"doc row\"\n";
        assert!(
            super::parse_entry_live_tree_disposition("m_test.dag", note_only).unwrap(),
            "a note-only entry stays undeclared = ReadsLiveTree"
        );
    }

    #[test]
    fn live_tree_disposition_malformed_variant_refuses() {
        let source = "module m\n\ndata live_tree_disposition: LiveTreeDisposition = MaybeLive\n";
        let err = super::parse_entry_live_tree_disposition("m_test.dag", source).unwrap_err();
        assert!(err.contains("unknown variant"), "got: {err}");
    }

    #[test]
    fn live_tree_disposition_wrong_type_annotation_refuses() {
        let source = "module m\n\ndata live_tree_disposition: Bool = true\n";
        let err = super::parse_entry_live_tree_disposition("m_test.dag", source).unwrap_err();
        assert!(err.contains("type annotation"), "got: {err}");
    }

    #[test]
    fn live_tree_disposition_duplicate_declaration_refuses() {
        let source = "module m\n\ndata live_tree_disposition: LiveTreeDisposition = ReadsLiveTree\ndata live_tree_disposition: LiveTreeDisposition = SubstrateInputsOnly\n";
        let err = super::parse_entry_live_tree_disposition("m_test.dag", source).unwrap_err();
        assert!(err.contains("more than once"), "got: {err}");
    }

    #[test]
    fn live_tree_declared_row_not_skipped_on_unrelated_diff() {
        let ws = workspace_root();
        std::env::set_current_dir(&ws).expect("chdir workspace");
        let roots = vec![
            ws.join("src/v2").to_string_lossy().into_owned(),
            ws.join("dag").to_string_lossy().into_owned(),
        ];
        let (runner_graph, runner_indices) =
            super::resolve_entry_graph(&roots, super::FLOOR_RUNNER_ENTRY)
                .expect("floor runner resolves");
        let runner_ctx =
            super::make_eval_context(&runner_graph, runner_indices, ExecutionMode::Wet);
        let live_entry =
            "src/v2/test/claim/long/realization_vocabulary_containment_witness_test.dag";
        assert!(
            super::read_entry_live_tree_disposition(live_entry)
                .expect("long live-corpus witness readable"),
            "the live-scan witness entry must declare (or default to) ReadsLiveTree"
        );
        let substrate_entry =
            "src/v2/test/claim/realization_vocabulary_containment/clean_tree_test.dag";
        assert!(
            !super::read_entry_live_tree_disposition(substrate_entry)
                .expect("clean_tree fast-lane note entry readable"),
            "the fast-lane note entry must declare SubstrateInputsOnly after long-lane offloading"
        );
        let changed_paths = vec!["src/v2/lens/affected_set.dag".to_string()];
        let skip = super::call_floor_row_would_skip(
            &runner_ctx,
            true,
            &changed_paths,
            &[],
            false,
            false,
            false,
            false,
        )
        .expect("live-tree row skip");
        assert!(
            !skip,
            "a ReadsLiveTree row must not skip on unrelated node-frontier diff"
        );
        let kernel_skip = super::call_floor_row_would_skip(
            &runner_ctx,
            false,
            &changed_paths,
            &[],
            false,
            false,
            false,
            false,
        )
        .expect("substrate-only row skip");
        assert!(
            kernel_skip,
            "the same unrelated diff must skip a declared SubstrateInputsOnly row \
             (discriminating control: the disposition is what flips the decision)"
        );
    }

    #[test]
    fn import_closure_carrier_home_matches_submodules() {
        use std::collections::HashSet;

        let carrier = "v2.compiler.self_host";
        let mut exact = HashSet::from([carrier.to_string()]);
        assert!(super::import_closure_module_reaches_carrier_home(
            &exact, carrier
        ));
        let mut submodule = HashSet::from(["v2.compiler.self_host.frontier".to_string()]);
        assert!(super::import_closure_module_reaches_carrier_home(
            &submodule, carrier
        ));
        let mut homonym = HashSet::from(["v2.compiler.self_hostile".to_string()]);
        assert!(!super::import_closure_module_reaches_carrier_home(
            &homonym, carrier
        ));
    }

    #[test]
    fn lying_substrate_inputs_only_stamp_census() {
        use std::collections::HashSet;

        let ws = workspace_root();
        std::env::set_current_dir(&ws).expect("chdir workspace");
        let roots = vec![
            ws.join("src/v2").to_string_lossy().into_owned(),
            ws.join("dag").to_string_lossy().into_owned(),
        ];
        let facts = super::build_module_graph_facts_live(&roots);
        let mut lying: Vec<(String, String)> = Vec::new();
        for (rel, content) in super::corpus_dag_files() {
            if !super::is_test_dag(&rel) {
                continue;
            }
            let Ok(reads_live_tree) = super::parse_entry_live_tree_disposition(&rel, &content)
            else {
                continue;
            };
            if reads_live_tree {
                continue;
            }
            let mut closure_modules = HashSet::new();
            super::collect_import_closure_module_names_from_facts(
                &rel,
                &facts,
                &mut closure_modules,
            );
            for carrier in super::LIVE_READ_CARRIER_HOME_MODULES_V0 {
                if super::import_closure_module_reaches_carrier_home(&closure_modules, carrier) {
                    lying.push((rel.clone(), (*carrier).to_string()));
                    break;
                }
            }
        }
        lying.sort();
        eprintln!(
            "lying SubstrateInputsOnly stamps (G1 carrier closure): {}",
            lying.len()
        );
        for (entry, carrier) in &lying {
            eprintln!("  {entry}  ->  {carrier}");
        }
        assert!(
            lying.is_empty(),
            "lying SubstrateInputsOnly stamps must be re-stamped ReadsLiveTree before merge"
        );
    }

    #[test]
    fn node_precise_same_file_referenced_vs_orphan_discriminates() {
        let ws = workspace_root();
        std::env::set_current_dir(&ws).expect("chdir workspace");
        let fixture = fixture_path();
        let roots = vec![
            ws.join("src/v2").to_string_lossy().into_owned(),
            ws.join("dag").to_string_lossy().into_owned(),
        ];
        let index = build_multi_entry_index(&roots);
        let (graph, source_indices) = super::resolve_entry_with_index(&index, &fixture)
            .expect("discriminator fixture resolves");
        let referenced_line =
            data_item_line(&fixture, &source_indices, &graph, "floor_disc_node_c");
        let orphan_line =
            data_item_line(&fixture, &source_indices, &graph, "floor_disc_orphan_node");
        assert_ne!(
            referenced_line, orphan_line,
            "fixture must place the two nodes on distinct lines"
        );

        let ctx = super::make_eval_context(&graph, source_indices.clone(), ExecutionMode::Wet);

        let referenced_diff = unified_diff_for_line(&fixture, referenced_line);
        let referenced_seeds = floor_diff_edits_from_diff_text(&index, &referenced_diff)
            .expect("frontier for referenced-node diff");
        assert!(
            entry_touches_rerun_frontier(
                &ctx,
                &list_value_from_vec(
                    rerun_frontier_nodes_for_entry(&ctx, &fixture, &referenced_seeds)
                        .expect("nodes")
                )
            )
            .expect("touch check (referenced)"),
            "a diff on a node some claim references must touch the entry (runs)"
        );

        let orphan_diff = unified_diff_for_line(&fixture, orphan_line);
        let orphan_seeds = floor_diff_edits_from_diff_text(&index, &orphan_diff)
            .expect("frontier for orphan-node diff");
        let orphan_nodes =
            rerun_frontier_nodes_for_entry(&ctx, &fixture, &orphan_seeds).expect("nodes");
        assert!(
            orphan_nodes.is_empty()
                || !entry_touches_rerun_frontier(&ctx, &list_value_from_vec(orphan_nodes))
                    .expect("touch check (orphan)"),
            "a diff on an orphan node (no claim references it) must NOT touch the entry (skips)"
        );
    }

    // #6543 regression: an importing witness in a DIFFERENT file than the changed `data`
    // declaration was predict-skipped (falsifier divergence, run 29293446579) because
    // `rerun_frontier_nodes_for_entry` only seeded the frontier from same-entry-file data
    // items. `data_item_declared_in_file` verifies the (file, name) pair against the
    // witness's own resolved import closure, so a genuine cross-file import is included
    // while an unrelated same-named data item elsewhere stays excluded.
    #[test]
    fn node_precise_cross_file_data_item_referenced_by_importer() {
        let ws = workspace_root();
        std::env::set_current_dir(&ws).expect("chdir workspace");
        let roots = vec![
            ws.join("src/v2").to_string_lossy().into_owned(),
            ws.join("dag").to_string_lossy().into_owned(),
        ];
        let index = build_multi_entry_index(&roots);
        let source_file = "src/v2/test/fixture/floor_skip/floor_disc_data_source.dag";
        let witness_file = "src/v2/test/fixture/floor_skip/floor_disc_data_witness_test.dag";

        let (source_graph, source_indices) = super::resolve_entry_with_index(&index, source_file)
            .expect("data-source fixture resolves");
        let data_line = data_item_line(
            source_file,
            &source_indices,
            &source_graph,
            "floor_disc_cross_file_data",
        );

        let (witness_graph, witness_indices) =
            super::resolve_entry_with_index(&index, witness_file)
                .expect("data-witness fixture resolves");
        let witness_ctx =
            super::make_eval_context(&witness_graph, witness_indices, ExecutionMode::Wet);

        let diff = unified_diff_for_line(source_file, data_line);
        let edits = floor_diff_edits_from_diff_text(&index, &diff)
            .expect("frontier for cross-file data-source diff");
        assert!(
            edits.overlapping_data_items.contains(&(
                source_file.to_string(),
                "floor_disc_cross_file_data".to_string()
            )),
            "diff on the data decl must populate overlapping_data_items, got {edits:?}"
        );

        let nodes = rerun_frontier_nodes_for_entry(&witness_ctx, witness_file, &edits)
            .expect("cross-file rerun frontier");
        assert!(
            entry_touches_rerun_frontier(&witness_ctx, &list_value_from_vec(nodes))
                .expect("touch check (cross-file importer)"),
            "a witness importing a changed cross-file data decl must be in-frontier (#6543)"
        );
    }
}

// Step 3 witness (a) PARTIAL — impl-vs-impl PROVE gate (#5994).
// Stable floor witnesses use deterministic fixture unified diffs (same structured shape as CI
// git diff parsing) so every checkout executes the proof — not branch-only origin/main...HEAD
// asserts. Node-frontier axis vs whole-tree InferredTree remains blocked on resolve grounding
// (ROADMAP 1-affected-set-defork); receipt in docs/plans/affected-set-precompute-pruning.md
// §Step 3 partial. `NodeFrontierSeeds` deleted — production and witnesses use `FloorDiffEdits`.

#[cfg(test)]
mod floor_witness_a_prove {
    use super::{
        build_multi_entry_index, diff_file_matches_entry, entry_touches_rerun_frontier,
        floor_diff_edits_from_diff_text, list_value_from_vec, make_eval_context,
        parse_unified_diff_line_ranges, rerun_frontier_nodes_for_entry, resolve_entry_with_index,
        scan_test_decl_lines, DiscoveryRow, FileLineRange, FloorDiffEdits,
    };
    use crate::v1_interpreter::{self, ExecutionMode, Value};
    use im::HashMap;
    use std::path::PathBuf;

    const FIXTURE_REL: &str = "src/v2/test/fixture/floor_skip/node_precise_discriminator_test.dag";
    const FLOOR_RUNNER: &str = "src/v2/workflow/affected_set_floor_runner.dag";
    const WITNESS_A_PROVE: &str = "src/v2/test/claim/affected_set_witness_a_prove_test.dag";
    const AFFECTED_SET_MID_PATH: &str = "src/v2/lens/affected_set.dag";

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(3)
            .expect("workspace root")
            .to_path_buf()
    }

    fn setup_roots(ws: &PathBuf) -> Vec<String> {
        vec![
            ws.join("src/v2").to_string_lossy().into_owned(),
            ws.join("dag").to_string_lossy().into_owned(),
        ]
    }

    fn fixture_line(text: &str, needle: &str) -> i64 {
        text.lines()
            .position(|l| l.contains(needle))
            .map(|i| (i + 1) as i64)
            .unwrap_or_else(|| panic!("fixture missing line containing `{needle}`"))
    }

    fn unified_diff_for_line(rel_path: &str, line: i64) -> String {
        format!(
            "diff --git a/{rel_path} b/{rel_path}\n--- a/{rel_path}\n+++ b/{rel_path}\n@@ -{line},0 +{line},1 @@\n+// witness-a touch\n"
        )
    }

    fn discriminator_roster(fixture_abs: &str) -> Vec<DiscoveryRow> {
        vec![
            DiscoveryRow {
                label: "floor_disc_witness_a".into(),
                entry: fixture_abs.to_string(),
                function: "floor_disc_witness_a_only_holds".into(),
                reads_live_tree: false,
            },
            DiscoveryRow {
                label: "floor_disc_witness_b".into(),
                entry: fixture_abs.to_string(),
                function: "floor_disc_witness_b_only_holds".into(),
                reads_live_tree: false,
            },
            DiscoveryRow {
                label: "floor_disc_witness_transitive".into(),
                entry: fixture_abs.to_string(),
                function: "floor_disc_witness_transitive_holds".into(),
                reads_live_tree: false,
            },
        ]
    }

    fn diff_line_touches_from_ranges(
        line_ranges: &HashMap<String, Vec<FileLineRange>>,
    ) -> Vec<(String, i64, i64)> {
        let mut out = Vec::new();
        for (path, ranges) in line_ranges {
            for range in ranges {
                out.push((path.clone(), range.start, range.end));
            }
        }
        out.sort();
        out
    }

    fn int_value(n: i64) -> Value {
        Value::Int(n)
    }

    fn diff_line_touch_value(
        ctx: &v1_interpreter::InterpContext,
        path: &str,
        start: i64,
        end: i64,
    ) -> Value {
        use std::rc::Rc;
        Value::Record {
            type_name: ctx.sym("FloorDiffLineTouch"),
            fields: Rc::new(vec![
                (ctx.sym("path"), Value::Str(path.to_string())),
                (ctx.sym("start_line"), int_value(start)),
                (ctx.sym("end_line"), int_value(end)),
            ]),
        }
    }

    fn call_floor_test_fn_declaration_edited(
        ctx: &v1_interpreter::InterpContext,
        touches: &[(String, i64, i64)],
        file_path: &str,
        decl_line: i64,
        decl_end_line: i64,
    ) -> Result<bool, String> {
        let touch_values: Vec<Value> = touches
            .iter()
            .map(|(p, s, e)| diff_line_touch_value(ctx, p, *s, *e))
            .collect();
        let args = [
            (
                Some("touches".to_string()),
                v1_interpreter::list_value(touch_values),
            ),
            (
                Some("file_path".to_string()),
                Value::Str(file_path.to_string()),
            ),
            (Some("test_fn_decl_line".to_string()), int_value(decl_line)),
            (
                Some("test_fn_decl_end_line".to_string()),
                int_value(decl_end_line),
            ),
        ];
        match v1_interpreter::run_in_context_with_args(
            ctx,
            "floor_test_fn_declaration_edited",
            &args,
            true,
        ) {
            Ok(Value::Bool(b)) => Ok(b),
            Ok(other) => Err(format!(
                "floor_test_fn_declaration_edited returned `{}`",
                ctx.format_value(&other)
            )),
            Err(e) => Err(format!("floor_test_fn_declaration_edited: {e}")),
        }
    }

    fn call_floor_rust_run_implies_dag_run(
        ctx: &v1_interpreter::InterpContext,
        rust_touches: bool,
        rust_func: bool,
        dag_touches: bool,
        dag_func: bool,
    ) -> Result<bool, String> {
        let args = [
            (
                Some("rust_touches_frontier".to_string()),
                Value::Bool(rust_touches),
            ),
            (
                Some("rust_function_edited".to_string()),
                Value::Bool(rust_func),
            ),
            (
                Some("dag_touches_frontier".to_string()),
                Value::Bool(dag_touches),
            ),
            (
                Some("dag_function_edited".to_string()),
                Value::Bool(dag_func),
            ),
        ];
        match v1_interpreter::run_in_context_with_args(
            ctx,
            "floor_rust_run_implies_dag_run",
            &args,
            true,
        ) {
            Ok(Value::Bool(b)) => Ok(b),
            Ok(other) => Err(format!(
                "floor_rust_run_implies_dag_run returned `{}`",
                ctx.format_value(&other)
            )),
            Err(e) => Err(format!("floor_rust_run_implies_dag_run: {e}")),
        }
    }

    fn rust_function_edited_for_row(edits: &FloorDiffEdits, row: &DiscoveryRow) -> bool {
        edits
            .edited_test_fns
            .iter()
            .any(|(file, func)| diff_file_matches_entry(file, &row.entry) && func == &row.function)
    }

    fn rust_entry_touches_from_edits(
        entry_ctx: &v1_interpreter::InterpContext,
        entry_path: &str,
        edits: &FloorDiffEdits,
    ) -> Result<bool, String> {
        let frontier_nodes = rerun_frontier_nodes_for_entry(entry_ctx, entry_path, edits)?;
        if frontier_nodes.is_empty() {
            return Ok(false);
        }
        entry_touches_rerun_frontier(entry_ctx, &list_value_from_vec(frontier_nodes))
    }

    fn dag_function_edited_for_row(
        ctx: &v1_interpreter::InterpContext,
        index: &super::MultiEntryIndex,
        touches: &[(String, i64, i64)],
        row: &DiscoveryRow,
    ) -> Result<bool, String> {
        let file_path = touches
            .iter()
            .find(|(path, _, _)| diff_file_matches_entry(path, &row.entry))
            .map(|(path, _, _)| path.clone())
            .unwrap_or_else(|| super::normalize_repo_path(&row.entry));
        let content = std::fs::read_to_string(&row.entry)
            .map_err(|e| format!("read {} for decl scan: {e}", row.entry))?;
        let decl_line = scan_test_decl_lines(&content)
            .into_iter()
            .find(|(name, _)| name == &row.function)
            .map(|(_, line)| line)
            .ok_or_else(|| {
                format!(
                    "witness row {} ({}) has no test fn declaration in entry",
                    row.function, row.entry
                )
            })?;
        let sorted_decls = super::collect_sorted_decl_lines_for_file(index, &row.entry)?;
        let decl_end = super::decl_span_end_line(&sorted_decls, decl_line);
        call_floor_test_fn_declaration_edited(ctx, touches, &file_path, decl_line, decl_end)
    }

    fn frontier_list_len(
        prove_ctx: &v1_interpreter::InterpContext,
        frontier: &v1_interpreter::Value,
    ) -> Result<usize, String> {
        let len = v1_interpreter::with_active_context(prove_ctx, || {
            v1_interpreter::free_monoid_to_vec(frontier).map(|items| items.len())
        });
        len.ok_or_else(|| {
            format!(
                "expected list frontier from .dag affected_set_closure, got `{}`",
                prove_ctx.format_value(frontier)
            )
        })
    }

    fn dag_affected_frontier_for_changed_path(
        prove_ctx: &v1_interpreter::InterpContext,
        changed_path: &str,
    ) -> Result<v1_interpreter::Value, String> {
        let args = [(
            Some("changed".to_string()),
            Value::Str(changed_path.to_string()),
        )];
        v1_interpreter::run_in_context_with_args(
            prove_ctx,
            "witness_a_dag_affected_nodes_for_path",
            &args,
            true,
        )
        .map_err(|e| format!("witness_a_dag_affected_nodes_for_path: {e}"))
    }

    fn dag_entry_touches_frontier_independently(
        prove_ctx: &v1_interpreter::InterpContext,
        entry_ctx: &v1_interpreter::InterpContext,
        changed_path: &str,
    ) -> Result<bool, String> {
        let frontier = dag_affected_frontier_for_changed_path(prove_ctx, changed_path)?;
        super::entry_touches_rerun_frontier(entry_ctx, &frontier)
    }

    fn assert_superset_on_fixture_with_real_diff_shape(
        ws: &PathBuf,
        diff_text: &str,
        roster: &[DiscoveryRow],
    ) {
        let roots = setup_roots(ws);
        let index = build_multi_entry_index(&roots);
        let line_ranges = parse_unified_diff_line_ranges(diff_text);
        assert!(
            !line_ranges.is_empty(),
            "PROVE diff must contain at least one .dag hunk"
        );
        let edits = floor_diff_edits_from_diff_text(&index, diff_text)
            .unwrap_or_else(|e| panic!("real-diff edits failed: {e}"));
        let touches = diff_line_touches_from_ranges(&line_ranges);

        let (runner_graph, runner_indices) =
            resolve_entry_with_index(&index, FLOOR_RUNNER).expect("floor runner resolves");
        let runner_ctx = make_eval_context(&runner_graph, runner_indices, ExecutionMode::Wet);
        let (prove_graph, prove_indices) =
            resolve_entry_with_index(&index, WITNESS_A_PROVE).expect("witness a prove resolves");
        let prove_ctx = make_eval_context(&prove_graph, prove_indices, ExecutionMode::Wet);

        let fixture_abs = ws.join(FIXTURE_REL).to_string_lossy().into_owned();
        let (graph, source_indices) =
            resolve_entry_with_index(&index, &fixture_abs).expect("fixture resolves");
        let entry_ctx = make_eval_context(&graph, source_indices, ExecutionMode::Wet);
        let rust_entry_touches = rust_entry_touches_from_edits(&entry_ctx, &fixture_abs, &edits)
            .expect("rust entry touch check");

        let changed_paths: Vec<String> = line_ranges.keys().cloned().collect();
        let mid_in_diff = changed_paths
            .iter()
            .any(|p| super::normalize_repo_path(p) == AFFECTED_SET_MID_PATH);
        let dag_entry_touches = if mid_in_diff {
            dag_entry_touches_frontier_independently(&prove_ctx, &entry_ctx, AFFECTED_SET_MID_PATH)
                .unwrap_or_else(|e| panic!("independent dag node-frontier: {e}"))
        } else {
            false
        };

        let mut saw_node_frontier_run = false;
        let mut saw_function_edited_run = false;

        for row in roster {
            let rust_func = rust_function_edited_for_row(&edits, row);
            let dag_func = dag_function_edited_for_row(&runner_ctx, &index, &touches, row)
                .unwrap_or_else(|e| panic!("dag function_edited for {}: {e}", row.function));
            let rust_touches = if diff_file_matches_entry(FIXTURE_REL, &row.entry) {
                rust_entry_touches
            } else {
                false
            };
            let dag_touches = if diff_file_matches_entry(FIXTURE_REL, &row.entry) && mid_in_diff {
                dag_entry_touches
            } else {
                false
            };
            assert!(
                call_floor_rust_run_implies_dag_run(
                    &runner_ctx,
                    rust_touches,
                    rust_func,
                    dag_touches,
                    dag_func
                )
                .unwrap_or_else(|e| panic!("superset predicate: {e}")),
                "superset violated for {} ({}): rust_touches={rust_touches} rust_func={rust_func} \
                 dag_touches={dag_touches} dag_func={dag_func}",
                row.function,
                row.entry
            );
            if rust_touches || rust_func {
                assert!(
                    !(call_floor_rust_run_implies_dag_run(
                        &runner_ctx,
                        rust_touches,
                        rust_func,
                        false,
                        false
                    ))
                    .unwrap_or(false),
                    "RED control sanity: strict-subset dag must fail superset for {}",
                    row.function
                );
            }
            if rust_touches && !rust_func {
                saw_node_frontier_run = true;
            }
            if rust_func {
                saw_function_edited_run = true;
            }
        }

        assert!(
            saw_node_frontier_run || saw_function_edited_run,
            "PROVE diff must fire at least one skip axis on the roster"
        );
    }

    #[test]
    fn witness_a_function_edited_axis_fixture_impl_vs_impl() {
        let ws = workspace_root();
        std::env::set_current_dir(&ws).expect("chdir workspace");
        let text = std::fs::read_to_string(ws.join(FIXTURE_REL)).expect("fixture readable");
        let line = fixture_line(&text, "test fn floor_disc_witness_a_only_holds");
        let diff = unified_diff_for_line(FIXTURE_REL, line);
        let roots = setup_roots(&ws);
        let index = build_multi_entry_index(&roots);
        let edits = floor_diff_edits_from_diff_text(&index, &diff)
            .expect("edits from function-edited fixture diff");
        assert!(
            edits
                .edited_test_fns
                .iter()
                .any(|(_, name)| name == "floor_disc_witness_a_only_holds"),
            "function-edited fixture must populate edited_test_fns"
        );
        let line_ranges = parse_unified_diff_line_ranges(&diff);
        let touches = diff_line_touches_from_ranges(&line_ranges);
        let (runner_graph, runner_indices) =
            resolve_entry_with_index(&index, FLOOR_RUNNER).expect("floor runner resolves");
        let runner_ctx = make_eval_context(&runner_graph, runner_indices, ExecutionMode::Wet);

        for (file, func) in &edits.edited_test_fns {
            let content = std::fs::read_to_string(file)
                .unwrap_or_else(|e| panic!("read {file} for decl line: {e}"));
            let decl_line = scan_test_decl_lines(&content)
                .into_iter()
                .find(|(name, _)| name == func)
                .map(|(_, line)| line)
                .unwrap_or_else(|| panic!("edited_test_fns {file}::{func} missing decl line"));
            let sorted_decls = super::collect_sorted_decl_lines_for_file(&index, file)
                .expect("sorted decl lines for impl-vs-impl");
            let decl_end = super::decl_span_end_line(&sorted_decls, decl_line);
            let dag_edited = call_floor_test_fn_declaration_edited(
                &runner_ctx,
                &touches,
                file,
                decl_line,
                decl_end,
            )
            .expect("dag function_edited model");
            assert!(
                dag_edited,
                "function_edited axis: rust edited_test_fns ({file}, {func}) must be matched by \
                 independent .dag floor_test_fn_declaration_edited"
            );
        }
    }

    #[test]
    fn witness_a_function_edited_axis_body_touch_fixture_impl_vs_impl() {
        let ws = workspace_root();
        std::env::set_current_dir(&ws).expect("chdir workspace");
        // Body line inside floor_disc_witness_a_only_holds (rebased when floor_disc_helper_fn landed in #6061).
        let diff = unified_diff_for_line(FIXTURE_REL, 83);
        let roots = setup_roots(&ws);
        let index = build_multi_entry_index(&roots);
        let edits = floor_diff_edits_from_diff_text(&index, &diff)
            .expect("edits from body-touch fixture diff");
        assert!(
            edits
                .edited_test_fns
                .iter()
                .any(|(_, name)| name == "floor_disc_witness_a_only_holds"),
            "body-only diff touch must populate edited_test_fns via decl span (not decl line only)"
        );
        let line_ranges = parse_unified_diff_line_ranges(&diff);
        let touches = diff_line_touches_from_ranges(&line_ranges);
        let (runner_graph, runner_indices) =
            resolve_entry_with_index(&index, FLOOR_RUNNER).expect("floor runner resolves");
        let runner_ctx = make_eval_context(&runner_graph, runner_indices, ExecutionMode::Wet);
        let file = FIXTURE_REL;
        let content = std::fs::read_to_string(ws.join(FIXTURE_REL)).expect("fixture readable");
        let decl_line = scan_test_decl_lines(&content)
            .into_iter()
            .find(|(name, _)| name == "floor_disc_witness_a_only_holds")
            .map(|(_, line)| line)
            .expect("witness_a decl line");
        let sorted_decls =
            super::collect_sorted_decl_lines_for_file(&index, file).expect("sorted decl lines");
        let decl_end = super::decl_span_end_line(&sorted_decls, decl_line);
        assert!(
            call_floor_test_fn_declaration_edited(&runner_ctx, &touches, file, decl_line, decl_end)
                .expect("dag function_edited model for body touch"),
            "body-only diff must match .dag floor_test_fn_declaration_edited when decl_end spans body"
        );
    }

    #[test]
    fn witness_a_red_control_under_selection_fails_superset() {
        let ws = workspace_root();
        std::env::set_current_dir(&ws).expect("chdir workspace");
        let roots = setup_roots(&ws);
        let index = build_multi_entry_index(&roots);
        let (runner_graph, runner_indices) =
            resolve_entry_with_index(&index, FLOOR_RUNNER).expect("floor runner resolves");
        let runner_ctx = make_eval_context(&runner_graph, runner_indices, ExecutionMode::Wet);
        assert!(
            !call_floor_rust_run_implies_dag_run(&runner_ctx, true, false, false, false)
                .expect("superset must fail when dag under-selects node-frontier"),
            "mandatory RED: rust-run + dag-skip must violate superset (§5 fail-open guard)"
        );
        assert!(
            !call_floor_rust_run_implies_dag_run(&runner_ctx, false, true, false, false)
                .expect("superset must fail when dag under-selects function_edited"),
            "mandatory RED: rust function_edited run + dag skip must violate superset"
        );
    }

    #[test]
    fn witness_a_node_frontier_dag_closure_independent_on_fixture() {
        let ws = workspace_root();
        std::env::set_current_dir(&ws).expect("chdir workspace");
        let roots = setup_roots(&ws);
        let index = build_multi_entry_index(&roots);
        let (prove_graph, prove_indices) =
            resolve_entry_with_index(&index, WITNESS_A_PROVE).expect("witness a prove resolves");
        let prove_ctx = make_eval_context(&prove_graph, prove_indices, ExecutionMode::Wet);
        let affected = dag_affected_frontier_for_changed_path(&prove_ctx, AFFECTED_SET_MID_PATH)
            .expect("dag affected_set_closure frontier");
        let node_count = frontier_list_len(&prove_ctx, &affected)
            .expect("frontier must be a list (List or Cons carrier)");
        assert!(
            node_count > 0,
            ".dag affected_set_closure must produce non-empty frontier for {AFFECTED_SET_MID_PATH} \
             via provenance_producer fixture (Impl-1 not inert; whole-tree Rust equivalence deferred)"
        );
    }

    #[test]
    fn witness_a_superset_on_discriminator_function_edited_fixture() {
        let ws = workspace_root();
        std::env::set_current_dir(&ws).expect("chdir workspace");
        let text = std::fs::read_to_string(ws.join(FIXTURE_REL)).expect("fixture readable");
        let line = fixture_line(&text, "test fn floor_disc_witness_a_only_holds");
        let diff = unified_diff_for_line(FIXTURE_REL, line);
        let fixture_abs = ws.join(FIXTURE_REL).to_string_lossy().into_owned();
        assert_superset_on_fixture_with_real_diff_shape(
            &ws,
            &diff,
            &discriminator_roster(&fixture_abs),
        );
    }
}

// Step 3 module-grain PROVE receipt (docs/plans/affected-set-precompute-pruning.md,
// ROADMAP 1-affected-set-defork). Node-grain (whole-tree `InferredTree`) equivalence stays
// BLOCKED (unaffordable resolve); this receipt is re-scoped to MODULE grain, using the landed
// `import_closure_live` authority (#6210/#6231).
//
// RE-PROMOTED TO PRODUCTION CERTIFICATION (2026-07-10, operator fork (c) — the #6335
// partial unwind): live floor discovery's `entry_file_touched` is decided by
// `entry_file_touched_via_import_closure` (module-graph import-closure grain over
// `facts.adjacency`), so the pair this harness proves — the `.dag` authority
// `entry_affected_by_touched_paths` (MODULE_GRAPH_ENTRY) vs the independent Rust oracle
// (`touched_file_in_import_closure` over `import_closure_files_from_graph`) — is the
// LIVE decision pair again. Between #6274 and the unwind this block was honestly labeled
// an orphan scaffold ("does NOT certify production selection after slice 2"); that
// framing dissolved with the unwind receipt (`entry_file_touched_grain_interim_note`,
// v2.lens.affected_set.entry_selection). A divergence here is a production selection bug.
//
// SCAFFOLD: dissolves into a .dag execution witness when the discovery/diff seed plumbing
// migrates off the v1 host layer (same trigger as `node_frontier_plumbing_controls` below,
// §6 dissolution trigger) — the equivalence lens itself moves on-carrier at that point, this
// hand-Rust harness is no longer needed to exercise it.
//
// It proves the module-grain "affected" decision computed by the `.dag` authority
// (`v2.lens.module_graph.entry_affected_by_touched_paths`, a thin projection over
// `import_closure_live`) agrees with an independent Rust oracle (`touched_file_in_import_closure`
// over `import_closure_files_from_graph`) on real merged-commit diffs. Deliberately separate
// implementations so agreement is proved by execution, not tautology.
// Both sides are fed by the same host-realized `import_resolution_facts`/
// `module_declaration_facts`, so this is a decision-level proof (§5: execution, not a
// grep/typecheck spec), not a re-proof of closure membership (already covered by
// `import_closure_equivalence_tests` above).
//
// Touched-paths derivation (fixed post-#6274 review): the input fed to BOTH sides is NOT the raw
// `git show --name-only` file list. Live production `entry_file_touched` is decided over
// `diff_edits.touched_entry_files` — the FILTERED set `floor_diff_edits_from_line_ranges`
// produces after excluding pure data-item edits (→ `overlapping_data_items`) and test-fn edits
// (→ `edited_test_fns`); only non-data, non-test-fn declaration edits land in
// `touched_entry_files`. A raw touched-path superset can diverge from this filtered set, so
// proving equivalence against raw paths only proves a stronger/looser predicate, not the live
// decision. This receipt runs `floor_diff_edits_from_diff_text(&index, &pinned_diff_text)`
// on each commit's full unified diff (pinned in `testdata/`, not `--name-only`) and feeds
// `.touched_entry_files` to both `dag_entry_affected` and `rust_entry_affected`.
//
// `floor_diff_edits_from_line_ranges` fail-closes (`Err`) when a touched `.dag` file's diff
// includes changed line 1 (the module declaration line) — see cli_run.rs:4831-4832 — so a commit
// that wholly ADDS new files (every new file's diff touches line 1) cannot be exercised via this
// real path (`entry_file_touched` is unreachable for that commit shape upstream of this receipt).
// Both SHAs below were chosen to be all-status-`M` (modify-only) commits for this reason.
#[cfg(test)]
mod module_grain_affected_equivalence_tests {
    use super::{
        build_multi_entry_index, floor_diff_edits_from_diff_text, import_closure_files_from_graph,
        import_resolution_facts_call_count_for_test, make_eval_context,
        module_declaration_facts_call_count_for_test, module_graph_facts_build_count_for_test,
        peak_rss_vhwm_bytes, reset_import_resolution_facts_call_counts_for_test,
        reset_module_graph_facts_build_count_for_test, resolve_entry_with_index,
        resolve_entry_with_index_for_discovery_corpus, touched_file_in_import_closure,
        workspace_root, MultiEntryIndex,
    };
    use crate::v1_interpreter::{self, ExecutionMode, Value};
    use std::collections::HashSet;
    use std::path::PathBuf;
    use std::time::Instant;

    const MODULE_GRAPH_ENTRY: &str = "src/v2/lens/module_graph.dag";

    fn setup_roots(ws: &PathBuf) -> Vec<String> {
        vec![
            ws.join("dag").to_string_lossy().into_owned(),
            ws.join("src/v2").to_string_lossy().into_owned(),
        ]
    }

    // Mirrors `gunbc.ci_layer_roots.witness_layer_roots` (`data witness_layer_roots: List<String>
    // = ["dag", "src/v2"]`) — the single authority for pool roots this receipt exercises against.
    fn pool_roots_rel() -> Vec<String> {
        vec!["dag".to_string(), "src/v2".to_string()]
    }

    // `entry_source_from_index_or_disk` stats `entry_path` directly against the process cwd (it
    // does not consult `workspace_root()`), so a bare repo-relative constant only resolves when
    // cwd happens to already be the workspace root. Rather than mutate the global process cwd
    // (the project's known `set_current_dir` parallel-test race — see the sibling
    // `node_frontier_plumbing_controls` module's `abs` helper for the same fix), build an absolute
    // path up front. `pool_roots`/`import_module` facts stay repo-relative (`rel_path_for_layer_import`),
    // so only the disk-touching entry lookups need this.
    fn abs(ws: &PathBuf, rel: &str) -> String {
        ws.join(rel).to_string_lossy().into_owned()
    }

    // Pinned unified diff fixtures for the two all-`M` commits below (NOT `--name-only`) — the
    // same shape the live floor parses via `parse_unified_diff_line_ranges`/
    // `parse_unified_diff_changed_new_lines`. Checked into `testdata/` so shallow clones and
    // remote test runners (BuildBuddy depth-1 fetch) do not need the historical git objects —
    // `git show <sha>` was the latent red on origin/main outside full-history worktrees.
    fn diff_text_for_commit(sha: &str) -> String {
        let text = match sha {
            "6edafbb5e29370c0ac791038a1c64e1a4ddbd40d" => {
                include_str!("../testdata/module_grain_affected_dag_only_6edafbb.diff")
            }
            "bb6e65649c9625d021467b0d7fe33ca7dd086e4f" => {
                include_str!("../testdata/module_grain_affected_v2_only_bb6e656.diff")
            }
            other => panic!(
                "module_grain_affected_equivalence: no pinned diff fixture for commit {other} — \
                 add testdata/module_grain_affected_<label>_<shortsha>.diff and extend \
                 diff_text_for_commit"
            ),
        };
        assert!(
            !text.trim().is_empty(),
            "pinned diff fixture for commit {sha} is empty"
        );
        text.to_string()
    }

    fn str_list_value(items: &[String]) -> Value {
        v1_interpreter::list_value(
            items
                .iter()
                .map(|s| Value::Str(s.clone()))
                .collect::<Vec<_>>(),
        )
    }

    /// #6274 orphan scaffold only — superseded import-closure query (`module_graph.dag`).
    /// The `.dag` authority side of the live decision pair (production floor realizes it
    /// via `entry_file_touched_via_import_closure`; this harness certifies the pair).
    fn dag_entry_affected(
        ctx: &v1_interpreter::InterpContext,
        entry_rel: &str,
        roots: &[String],
        touched: &[String],
    ) -> bool {
        let args = [
            (
                Some("entry_path".to_string()),
                Value::Str(entry_rel.to_string()),
            ),
            (Some("pool_roots".to_string()), str_list_value(roots)),
            (Some("touched_paths".to_string()), str_list_value(touched)),
        ];
        match v1_interpreter::run_in_context_with_args(
            ctx,
            "entry_affected_by_touched_paths",
            &args,
            true,
        ) {
            Ok(Value::Bool(b)) => b,
            Ok(other) => panic!(
                "entry_affected_by_touched_paths returned `{}`",
                ctx.format_value(&other)
            ),
            Err(e) => panic!("entry_affected_by_touched_paths: {e}"),
        }
    }

    fn rust_entry_affected(index: &MultiEntryIndex, entry_rel: &str, touched: &[String]) -> bool {
        // Shared selection rule with production (`entry_file_touched_via_import_closure`) and
        // the `.dag` authority (`entry_without_declared_edges_never_skips_note`): an entry
        // that declares no imports is never selection-skippable — its name-derived
        // dependencies are invisible to the import-edge model, so both sides answer
        // affected=true rather than risking a false skip.
        let source = std::fs::read_to_string(workspace_root().join(entry_rel))
            .unwrap_or_else(|e| panic!("read {entry_rel}: {e}"));
        if !source
            .lines()
            .any(|l| l.trim_start().starts_with("import "))
        {
            return true;
        }
        let (graph, _) = resolve_entry_with_index_for_discovery_corpus(index, entry_rel)
            .unwrap_or_else(|e| panic!("resolve {entry_rel}: {e}"));
        let closure_files: HashSet<String> = import_closure_files_from_graph(&graph);
        touched
            .iter()
            .any(|f| touched_file_in_import_closure(f, &closure_files))
    }

    struct EquivalenceReceipt {
        sha: String,
        touched: Vec<String>,
        rows: Vec<(String, bool, bool)>, // (entry, rust_decision, dag_decision)
    }

    fn run_equivalence_for_commit(sha: &str, entries: &[&str]) -> EquivalenceReceipt {
        let ws = workspace_root();
        // `entry_source_from_index_or_disk` and `floor_diff_edits_from_line_ranges` both read
        // paths off disk relative to cwd, matching the live floor's process cwd == workspace
        // root. Use the SAME relative roots/paths the live floor uses throughout (`dag`,
        // `src/v2`, `src/v2/workflow/affected_set_floor_runner.dag`-style constants) rather than
        // absolute paths — mixing the two conventions for the same physical file inside one
        // `MultiEntryIndex` re-resolves it under a second identity and trips the interpreter's
        // "duplicate module declaration" / circular-dependency guard.
        std::env::set_current_dir(&ws).expect("chdir workspace");
        let rel_roots = pool_roots_rel();

        let index = build_multi_entry_index(&rel_roots);
        let diff_text = diff_text_for_commit(sha);
        let edits = floor_diff_edits_from_diff_text(&index, &diff_text).unwrap_or_else(|e| {
            panic!(
                "floor_diff_edits_from_diff_text failed for commit {sha}: {e} — pick a \
                 different all-`M`-status dag/ or src/v2/ commit whose diffs never touch line 1"
            )
        });
        let mut touched: Vec<String> = edits.touched_entry_files.into_iter().collect();
        touched.sort();
        assert!(
            !touched.is_empty(),
            "commit {sha} produced an empty touched_entry_files set — pick a commit whose diff \
             touches at least one non-data, non-test-fn declaration"
        );
        let (mg_graph, mg_indices) =
            resolve_entry_with_index_for_discovery_corpus(&index, MODULE_GRAPH_ENTRY)
                .expect("module_graph.dag resolves as an interpreter entry");
        let dag_ctx = make_eval_context(&mg_graph, mg_indices, ExecutionMode::Wet);

        let mut rows = Vec::new();
        for entry in entries {
            let rust_decision = rust_entry_affected(&index, entry, &touched);
            let dag_decision = dag_entry_affected(&dag_ctx, entry, &rel_roots, &touched);
            rows.push((entry.to_string(), rust_decision, dag_decision));
        }
        EquivalenceReceipt {
            sha: sha.to_string(),
            touched,
            rows,
        }
    }

    fn assert_receipt_matches(receipt: &EquivalenceReceipt) {
        let mut divergences = Vec::new();
        for (entry, rust, dag) in &receipt.rows {
            if rust != dag {
                divergences.push(format!("{entry}: rust={rust} dag={dag}"));
            }
        }
        assert!(
            divergences.is_empty(),
            "module-grain decision diverged for commit {}: {}",
            receipt.sha,
            divergences.join(", ")
        );
    }

    fn write_receipt_log(name: &str, receipt: &EquivalenceReceipt) {
        let mut out = String::new();
        out.push_str(&format!("commit: {}\n", receipt.sha));
        out.push_str("touched paths:\n");
        for t in &receipt.touched {
            out.push_str(&format!("  {t}\n"));
        }
        out.push_str("entry decisions (rust_decision, dag_decision):\n");
        for (entry, rust, dag) in &receipt.rows {
            out.push_str(&format!("  {entry}: rust={rust} dag={dag}\n"));
        }
        let dir = workspace_root().join("target/module_grain_affected_receipts");
        std::fs::create_dir_all(&dir).expect("create receipt dir");
        let path = dir.join(format!("{name}.txt"));
        std::fs::write(&path, &out).unwrap_or_else(|e| panic!("write receipt {path:?}: {e}"));
        eprintln!("--- module-grain affected-set receipt: {name} ---\n{out}");
    }

    // Real diff 1: dag/-only merged commit, all-status-`M` (4 modified files: healthz grammar
    // JSON + structured shell in v1_dag_parse transport, #6166) — chosen over the original
    // all-new-file commit because `floor_diff_edits_from_line_ranges` fail-closes on any diff
    // that touches a file's line 1 (every wholly-new file does), which the real production path
    // can never reach for an all-added-files commit; a modify-only commit exercises it for real.
    const DAG_ONLY_SHA: &str = "6edafbb5e29370c0ac791038a1c64e1a4ddbd40d";
    // Real diff 2: src/v2/-only merged commit (6 modified files, bash orchestration-emit dissolve).
    const V2_ONLY_SHA: &str = "bb6e65649c9625d021467b0d7fe33ca7dd086e4f";

    // Representative sample for the dag/-only diff: the two witness files that directly declare
    // `import`s of the new ebay/tcgplayer/card_intake modules (expected affected=true on both
    // sides), plus a spread of unrelated floor witnesses drawn from both pool roots and both
    // near (same-directory) and far (cross-tree) module-graph distance from the touched files
    // (expected affected=false) — enough to show the equivalence holds both when it fires and
    // when it doesn't, without requiring the full ~514-entry roster.
    fn dag_only_entry_sample() -> Vec<&'static str> {
        vec![
            "dag/test/claim/card_intake_risk_witness_test.dag",
            "dag/test/claim/ebay_listing_witness_test.dag",
            "src/v2/test/claim/bash_program_fold_test.dag",
            "dag/test/claim/v1_dag_parse_witness_test.dag",
            "dag/tools/host_prelude.dag",
            "dag/tools/build_step.dag",
            "dag/gunbc/ci_layer_roots.dag",
            "src/v2/test/claim/bash_command_fold_test.dag",
            "src/v2/workflow/orchestration_emit_test.dag",
            "dag/test/claim/long/import_closure_live_test.dag",
            "src/v2/test/claim/affected_set_universe_test.dag",
            "src/v2/lens/module_graph.dag",
        ]
    }

    // Representative sample for the src/v2/-only diff: the two directly touched test files
    // (affected=true), a set of witnesses that import `bash.dag`/`bash_orchestration_emit.dag`
    // transitively (via the same import chain the discriminating control below exercises;
    // affected=true), and unrelated witnesses from both trees (affected=false).
    fn v2_only_entry_sample() -> Vec<&'static str> {
        vec![
            "src/v2/workflow/orchestration_bash_test.dag",
            "src/v2/workflow/orchestration_emit_test.dag",
            "src/v2/test/claim/bash_command_fold_test.dag",
            "src/v2/test/claim/bash_program_fold_test.dag",
            "src/v2/test/claim/manual/bash_emit_command_test.dag",
            "src/v2/test/claim/manual/emit_directive_bash_test.dag",
            "src/v2/test/claim/manual/emit_directive_gha_test.dag",
            "src/v2/workflow/orchestration_retry_emit_test.dag",
            "src/v2/test/claim/realization_vocabulary_containment/lens_unit/discriminators_test.dag",
            "dag/test/claim/card_intake_risk_witness_test.dag",
            "dag/tools/host_prelude.dag",
            "src/v2/test/claim/affected_set_universe_test.dag",
            "dag/test/claim/long/import_closure_live_test.dag",
        ]
    }

    #[test]
    fn module_grain_affected_equivalence_dag_only_real_diff() {
        let entries = dag_only_entry_sample();
        let receipt = run_equivalence_for_commit(DAG_ONLY_SHA, &entries);
        write_receipt_log("dag_only_real_diff", &receipt);
        // A vacuously all-false (or all-true) receipt would not be a real proof of agreement —
        // require the sample to actually exercise both outcomes on both sides.
        assert!(
            receipt.rows.iter().any(|(_, rust, dag)| *rust && *dag),
            "receipt must contain at least one true/true (affected) row to be discriminating"
        );
        assert!(
            receipt.rows.iter().any(|(_, rust, dag)| !*rust && !*dag),
            "receipt must contain at least one false/false (unaffected) row"
        );
        assert_receipt_matches(&receipt);
    }

    #[test]
    fn module_grain_affected_equivalence_v2_only_real_diff() {
        let entries = v2_only_entry_sample();

        reset_import_resolution_facts_call_counts_for_test();
        reset_module_graph_facts_build_count_for_test();
        let t0 = Instant::now();
        let receipt = run_equivalence_for_commit(V2_ONLY_SHA, &entries);
        let elapsed = t0.elapsed();
        let dag_side_import_resolution_calls = import_resolution_facts_call_count_for_test();
        let dag_side_module_decl_calls = module_declaration_facts_call_count_for_test();
        let rust_side_facts_batch_builds = module_graph_facts_build_count_for_test();
        let peak_rss = peak_rss_vhwm_bytes();

        write_receipt_log("v2_only_real_diff", &receipt);

        let mut cost_log = String::new();
        cost_log.push_str(&format!("entries_sampled: {}\n", entries.len()));
        cost_log.push_str(&format!(
            "wall_clock_ms_for_equivalence_run: {:.2}\n",
            elapsed.as_secs_f64() * 1000.0
        ));
        cost_log.push_str(&format!(
            "dag_side_import_resolution_facts_calls: {dag_side_import_resolution_calls}\n"
        ));
        cost_log.push_str(&format!(
            "dag_side_module_declaration_facts_calls: {dag_side_module_decl_calls}\n"
        ));
        cost_log.push_str(&format!(
            "rust_side_build_module_graph_facts_live_calls: {rust_side_facts_batch_builds}\n"
        ));
        cost_log.push_str(&format!("peak_rss_vhwm_bytes: {peak_rss:?}\n"));
        let dir = workspace_root().join("target/module_grain_affected_receipts");
        std::fs::create_dir_all(&dir).expect("create receipt dir");
        std::fs::write(dir.join("v2_only_real_diff_cost.txt"), &cost_log)
            .expect("write cost receipt");
        eprintln!("--- module-grain affected-set cost receipt ---\n{cost_log}");

        assert_receipt_matches(&receipt);
    }

    // Discriminating control: perturb REAL wiring by dropping an intermediate importer's
    // outgoing edges (via `import_closure_live_excluding`'s `exclude_substrings` — the same knob
    // `import_closure_live` delegates through, not a bespoke test-only mechanism) and prove the
    // module-grain decision actually flips. This shows the equivalence assertions above are a
    // real discriminator: if this control could not go RED, the checks above could pass
    // vacuously (§5 "witness re-asserting realizer is tautological").
    #[test]
    fn module_grain_affected_decision_discriminates_under_wiring_perturbation() {
        let ws = workspace_root();
        let roots = setup_roots(&ws);
        let rel_roots = pool_roots_rel();
        let index = build_multi_entry_index(&roots);
        let (mg_graph, mg_indices) =
            resolve_entry_with_index_for_discovery_corpus(&index, &abs(&ws, MODULE_GRAPH_ENTRY))
                .expect("module_graph.dag resolves as an interpreter entry");
        let dag_ctx = make_eval_context(&mg_graph, mg_indices, ExecutionMode::Wet);

        // `orchestration_bounded_poll_emit_test.dag` reaches `bash.dag` transitively along
        // two routes: via `bash_orchestration_emit.dag` and via `05_emit_orchestration.dag`
        // (both directly imported). Dropping the outgoing edges of BOTH intermediates
        // severs every route to the leaf without touching the entry's own imports — still
        // a genuine wiring perturbation (the entry stays resolvable; only reachability to
        // the leaf flips). The entry must be one that still DECLARES imports: an
        // import-less entry answers affected=true by the shared never-skip rule
        // (`entry_without_declared_edges_never_skips_note`) on both the perturbed and
        // unperturbed sides, so no wiring perturbation could flip it.
        let entry = "src/v2/workflow/orchestration_bounded_poll_emit_test.dag";
        let leaf = "src/v2/extdeps/languages/bash.dag".to_string();
        let intermediate_excludes = [
            "extdeps/languages/bash_orchestration_emit.dag".to_string(),
            "compiler/05_emit_orchestration.dag".to_string(),
        ];
        let intermediate_exclude = intermediate_excludes.join(" + ");

        let content = std::fs::read_to_string(ws.join(entry))
            .unwrap_or_else(|e| panic!("read {entry} for precondition: {e}"));
        assert!(
            !content.contains("v2.extdeps.languages.bash "),
            "precondition: {entry} must reach {leaf} only transitively (via the excluded \
             intermediates), not via a direct import — otherwise excluding the \
             intermediates would not discriminate"
        );

        let touched = vec![leaf.clone()];

        let unperturbed = dag_entry_affected(&dag_ctx, entry, &rel_roots, &touched);
        assert!(
            unperturbed,
            "precondition failed: the full (unperturbed) closure of {entry} must reach {leaf} \
             transitively via bash_orchestration_emit.dag for this control to be meaningful"
        );

        let args = [
            (
                Some("entry_path".to_string()),
                Value::Str(entry.to_string()),
            ),
            (Some("pool_roots".to_string()), str_list_value(&rel_roots)),
            (Some("touched_paths".to_string()), str_list_value(&touched)),
            (
                Some("exclude_substrings".to_string()),
                str_list_value(&intermediate_excludes),
            ),
        ];
        let perturbed = match v1_interpreter::run_in_context_with_args(
            &dag_ctx,
            "entry_affected_by_touched_paths_excluding",
            &args,
            true,
        ) {
            Ok(Value::Bool(b)) => b,
            Ok(other) => panic!(
                "entry_affected_by_touched_paths_excluding returned `{}`",
                dag_ctx.format_value(&other)
            ),
            Err(e) => panic!("entry_affected_by_touched_paths_excluding: {e}"),
        };

        assert!(
            !perturbed,
            "dropping {intermediate_exclude}'s outgoing edges must remove {leaf} from {entry}'s \
             closure (a real wiring perturbation), but the decision stayed true"
        );
        assert_ne!(
            unperturbed, perturbed,
            "discriminating control must actually flip the decision under a real wiring \
             perturbation, not merely execute the code path unchanged"
        );
    }
}

// SCAFFOLD: folds into a .dag execution witness when the discovery/diff seed plumbing
// migrates off the v1 host layer (§6 dissolution trigger)
#[cfg(test)]
mod node_frontier_plumbing_controls {
    use super::{
        build_multi_entry_index, call_floor_kernel_would_skip, entry_touches_rerun_frontier,
        floor_diff_edits_from_diff_text, list_value_from_vec, parse_unified_diff_line_ranges,
        rerun_frontier_nodes_for_entry, scan_test_decl_lines,
    };
    use crate::v1_interpreter::ExecutionMode;
    use std::path::PathBuf;

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(3)
            .expect("workspace root")
            .to_path_buf()
    }

    const FIXTURE: &str = "src/v2/test/fixture/floor_skip/node_precise_discriminator_test.dag";
    // File outside FIXTURE's import closure — precondition asserted at runtime in green control.
    // If a future import edge adds this file to FIXTURE's closure, the precondition assertion
    // fails loudly rather than letting the control silently degrade (§3 anti-drift).
    const OUTSIDE_FILE: &str = "src/v2/lens/affected_set.dag";
    // A known data-declaration line in OUTSIDE_FILE.
    // If this line shifts the test may fail seed collection — a loud failure, not a silent pass.
    const OUTSIDE_DATA_LINE: i64 = 1295;

    fn abs(ws: &PathBuf, rel: &str) -> String {
        ws.join(rel).to_string_lossy().into_owned()
    }

    // parse_unified_diff_line_ranges strips "+++ b/" prefix; "b//abs/path" yields "/abs/path".
    fn diff_at(file: &str, line: i64) -> String {
        format!(
            "diff --git a/{file} b/{file}\n--- a/{file}\n+++ b/{file}\n\
             @@ -{line},0 +{line},1 @@\n+// synthetic touch\n"
        )
    }

    fn deletion_diff_at(file: &str, line: i64) -> String {
        format!(
            "diff --git a/{file} b/{file}\n--- a/{file}\n+++ b/{file}\n\
             @@ -{line},1 +{line},0 @@\n-// synthetic deletion\n"
        )
    }

    fn setup_roots(ws: &PathBuf) -> Vec<String> {
        vec![
            ws.join("src/v2").to_string_lossy().into_owned(),
            ws.join("dag").to_string_lossy().into_owned(),
        ]
    }

    // Control 1 (GREEN/skip): diff on file outside FIXTURE's import closure → skip fires.
    // Q1 precondition asserted at runtime: if a future import edge adds OUTSIDE_FILE to
    // FIXTURE's closure, this assertion fires before the skip assertion can silently degrade.
    #[test]
    fn green_skip_for_file_outside_import_closure() {
        let ws = workspace_root();
        let roots = setup_roots(&ws);
        let index = build_multi_entry_index(&roots);

        // Q1 precondition: assert OUTSIDE_FILE is not in FIXTURE's transitive import closure.
        let (graph, source_indices) =
            super::resolve_entry_with_index(&index, &abs(&ws, FIXTURE)).expect("fixture resolves");
        let outside = OUTSIDE_FILE.replace('\\', "/");
        let in_closure = graph.modules.iter().any(|m| {
            m.items
                .iter()
                .any(|item| item.span.file.replace('\\', "/").contains(&outside))
        });
        assert!(
            !in_closure,
            "precondition: {OUTSIDE_FILE} must not be in {FIXTURE}'s import closure; \
             if it now is, update OUTSIDE_FILE to a different out-of-closure file"
        );

        // Build diff touching a data declaration in OUTSIDE_FILE (absolute path so parse_unified_diff
        // resolves it without process-global cwd — "b//abs" strips to "/abs" after the b/ prefix).
        let diff = diff_at(&abs(&ws, OUTSIDE_FILE), OUTSIDE_DATA_LINE);
        let seeds =
            floor_diff_edits_from_diff_text(&index, &diff).expect("seeds from outside-file diff");
        let ctx = super::make_eval_context(&graph, source_indices, ExecutionMode::Wet);
        let nodes =
            rerun_frontier_nodes_for_entry(&ctx, &abs(&ws, FIXTURE), &seeds).expect("nodes");
        assert!(
            nodes.is_empty()
                || !entry_touches_rerun_frontier(&ctx, &list_value_from_vec(nodes))
                    .expect("touch check"),
            "entry must NOT touch frontier when diff is on a file outside its import closure"
        );
    }

    #[test]
    fn skip_without_resolve_fast_path_eligible_outside_import_closure() {
        let ws = workspace_root();
        std::env::set_current_dir(&ws).expect("chdir workspace");
        let roots = setup_roots(&ws);
        let index = build_multi_entry_index(&roots);
        let fixture_abs = abs(&ws, FIXTURE);
        let diff = diff_at(&abs(&ws, OUTSIDE_FILE), OUTSIDE_DATA_LINE);
        let diff_edits =
            floor_diff_edits_from_diff_text(&index, &diff).expect("seeds from outside-file diff");
        let declared = index.module_graph_facts.declared_repo_paths();
        let touched_paths: Vec<String> = diff_edits.touched_entry_files.iter().cloned().collect();
        // Substrate-only fixture (reads_live_tree=false) → eligible when unaffected.
        assert!(
            super::entry_qualifies_for_skip_without_resolve(
                &fixture_abs,
                false,
                &index.module_graph_facts,
                &declared,
                &touched_paths,
                &diff_edits,
            )
            .expect("qualify"),
            "unaffected entry must qualify for skip-before-resolve when diff is outside import closure"
        );
    }

    // Discriminating RED control (§5 never-skip tooth): a `ReadsLiveTree` entry must NEVER
    // qualify for skip-before-resolve, even in the exact unaffected-diff case that WOULD skip
    // a substrate-only entry. If the `reads_live_tree` guard in
    // `entry_qualifies_for_skip_without_resolve` is removed/bypassed, this goes red — the
    // fail-open (a live-tree witness predicted-skipped → never runs → false green) is caught.
    #[test]
    fn live_tree_entry_never_qualifies_for_skip_without_resolve() {
        let ws = workspace_root();
        std::env::set_current_dir(&ws).expect("chdir workspace");
        let roots = setup_roots(&ws);
        let index = build_multi_entry_index(&roots);
        let fixture_abs = abs(&ws, FIXTURE);
        // Same unaffected diff as the eligible test above (outside the import closure).
        let diff = diff_at(&abs(&ws, OUTSIDE_FILE), OUTSIDE_DATA_LINE);
        let diff_edits =
            floor_diff_edits_from_diff_text(&index, &diff).expect("seeds from outside-file diff");
        let declared = index.module_graph_facts.declared_repo_paths();
        let touched_paths: Vec<String> = diff_edits.touched_entry_files.iter().cloned().collect();
        assert!(
            !super::entry_qualifies_for_skip_without_resolve(
                &fixture_abs,
                true,
                &index.module_graph_facts,
                &declared,
                &touched_paths,
                &diff_edits,
            )
            .expect("qualify"),
            "a ReadsLiveTree entry must NOT qualify for skip-before-resolve even when the diff is outside its import closure (never predict-skip)"
        );
    }

    // §5 prove-the-refusal-fires: layer-3 backstop is by-design hard to reach in integration
    // (the .dag model + entry_qualifies_for_skip_without_resolve gate first) — direct RED.
    #[test]
    fn refuse_reads_live_tree_selection_skip_fires_red_on_live_tree_row() {
        let live = super::DiscoveryRow {
            label: "live".to_string(),
            entry: "e.dag".to_string(),
            function: "live_holds".to_string(),
            reads_live_tree: true,
        };
        let err = super::refuse_reads_live_tree_selection_skip(&live, "test")
            .expect_err("ReadsLiveTree row must refuse selection skip");
        assert!(err.contains("ReadsLiveTreeSelectionSkip"));
        let substrate = super::DiscoveryRow {
            reads_live_tree: false,
            ..live
        };
        super::refuse_reads_live_tree_selection_skip(&substrate, "test")
            .expect("SubstrateInputsOnly row may proceed to skip evaluation");
    }

    // §5 deferred-discovery receipt: long-lane witnesses (s1_closure class) are excluded
    // from per-PR discovery but must be COUNTED in the floor log — never a silent skip.
    #[test]
    fn deferred_discovery_counts_long_lane_s1_closure_reads_live_tree() {
        let ws = workspace_root();
        std::env::set_current_dir(&ws).expect("chdir workspace");
        let roots = setup_roots(&ws);
        let excludes = super::witness_exclusion_substrings();
        let deferred =
            super::collect_deferred_discovery_rows(&roots, &excludes).expect("deferred scan");
        let s1 = deferred
            .iter()
            .find(|r| r.function == "s1_closure_parses_holds")
            .expect("s1_closure_parses_holds must appear in deferred-discovery receipt");
        assert!(
            s1.reads_live_tree,
            "s1_closure declares ReadsLiveTree — deferred row must carry the disposition"
        );
        assert!(
            s1.entry.contains("test/claim/long/"),
            "s1_closure lives in the long lane: got {}",
            s1.entry
        );
        assert!(
            s1.exclude_reason.contains("test/claim/long/"),
            "exclude reason must name the long-lane substring: got {}",
            s1.exclude_reason
        );
    }

    // Phase 0(b) admission invariant: every deferred witness row names an executing consumer.
    #[test]
    fn witness_admission_deferred_rows_have_consumers() {
        let ws = workspace_root();
        std::env::set_current_dir(&ws).expect("chdir workspace");
        let roots = setup_roots(&ws);
        let excludes = super::witness_exclusion_substrings();
        let deferred =
            super::collect_deferred_discovery_rows(&roots, &excludes).expect("deferred scan");
        let orphans = super::collect_unexecuted_deferred_witnesses(&deferred);
        super::refuse_unexecuted_deferred_witnesses(&orphans)
            .unwrap_or_else(|e| panic!("live deferred corpus must admit every row: {e}"));
        let normalize = deferred
            .iter()
            .find(|r| r.function == "self_host_03_normalize_behavioral_receipt_holds")
            .expect("03_normalize behavioral receipt must be deferred from discovery");
        assert!(
            normalize
                .entry
                .contains("self_host_03_normalize_behavioral_witness_test"),
            "03_normalize receipt entry: got {}",
            normalize.entry
        );
    }

    #[test]
    fn witness_admission_orphan_synthetic_row_refuses() {
        let orphan = super::DeferredDiscoveryRow {
            entry: "dag/test/claim/synthetic_orphan_admission_witness_test.dag".to_string(),
            function: "synthetic_orphan_no_consumer_holds".to_string(),
            exclude_reason: "synthetic_orphan_admission_witness_test.dag".to_string(),
            reads_live_tree: false,
        };
        let orphans = super::collect_unexecuted_deferred_witnesses(&[orphan]);
        assert_eq!(orphans.len(), 1);
        let err = super::refuse_unexecuted_deferred_witnesses(&orphans).expect_err("orphan");
        assert!(err.contains("UnexecutedDeferredWitness"));
    }

    // Task 5 (declared-source-ref selection): the 03_normalize flagship opts into
    // declared_source_refs on its transport — effect_reach must NOT upgrade it to
    // ReadsLiveTree; selection uses the declared-ref axis instead.
    #[test]
    fn declared_source_refs_suppress_effect_reach_upgrade_for_03_normalize_witness() {
        let ws = workspace_root();
        std::env::set_current_dir(&ws).expect("chdir workspace");
        let roots = setup_roots(&ws);
        let facts = super::build_module_graph_facts_live(&roots);
        let entry = "dag/test/claim/self_host_03_normalize_behavioral_witness_test.dag";
        assert!(
            super::entry_has_declared_source_refs(entry, &facts),
            "03_normalize flagship must declare source refs on its transport"
        );
        let mut rows = vec![super::DiscoveryRow {
            label: "normalize".to_string(),
            entry: entry.to_string(),
            function: "self_host_03_normalize_behavioral_receipt_holds".to_string(),
            reads_live_tree: false,
        }];
        super::apply_effect_reach_derived_reads_live_tree(&mut rows, &facts);
        assert!(
            !rows[0].reads_live_tree,
            "declared_source_refs must prevent effect_reach from upgrading the flagship row"
        );
    }

    #[test]
    fn declared_source_refs_selection_both_directions_for_03_normalize_witness() {
        let ws = workspace_root();
        std::env::set_current_dir(&ws).expect("chdir workspace");
        let roots = setup_roots(&ws);
        let facts = super::build_module_graph_facts_live(&roots);
        let entry = "dag/test/claim/self_host_03_normalize_behavioral_witness_test.dag";
        let touched = vec!["src/v2/compiler/03_normalize.dag".to_string()];
        assert_eq!(
            super::declared_source_refs_axis_for_entry(entry, &facts, &roots, &touched),
            super::DeclaredSourceRefAxis::Touched,
            "emitter touch must select the 03_normalize behavioral witness"
        );
        let unrelated = vec!["src/v2/std/logic.dag".to_string()];
        assert_eq!(
            super::declared_source_refs_axis_for_entry(entry, &facts, &roots, &unrelated),
            super::DeclaredSourceRefAxis::Untouched,
            "unrelated path must not select the 03_normalize behavioral witness"
        );
    }

    // Phase 0 monotone-toward-RUN (bridge a): derived census may only UPGRADE toward
    // ReadsLiveTree — a declared/disposition row must never downgrade because the census
    // returns empty for its closure.
    #[test]
    fn effect_reach_derived_reads_live_tree_never_downgrades_declared_row() {
        let ws = workspace_root();
        std::env::set_current_dir(&ws).expect("chdir workspace");
        let roots = setup_roots(&ws);
        let facts = super::build_module_graph_facts_live(&roots);
        let entry = "src/v2/test/claim/long/s1_closure_receipt_test.dag";
        let content = std::fs::read_to_string(ws.join(entry)).expect("s1_closure readable");
        assert!(
            super::parse_entry_live_tree_disposition(entry, &content).expect("parse disposition"),
            "precondition: s1_closure declares ReadsLiveTree"
        );
        assert!(
            !super::effect_reach_derived_reads_live_tree_for_entry(entry, &facts),
            "precondition: empty census for s1_closure closure (no path-literal→sink flows)"
        );
        let mut rows = vec![super::DiscoveryRow {
            label: "live".to_string(),
            entry: entry.to_string(),
            function: "s1_closure_parses_holds".to_string(),
            reads_live_tree: true,
        }];
        super::apply_effect_reach_derived_reads_live_tree(&mut rows, &facts);
        assert!(
            rows[0].reads_live_tree,
            "declared ReadsLiveTree must survive apply_effect_reach even when census is empty"
        );
    }

    // Phase 0 monotone-toward-RUN (bridge b): effect_reach_touched is additive touch
    // evidence — it may block skip on literal match but absence must not enable skip
    // beyond today's rules for a hermetic entry.
    #[test]
    fn effect_reach_touched_additive_only_hermetic_baseline_unchanged() {
        let ws = workspace_root();
        std::env::set_current_dir(&ws).expect("chdir workspace");
        let roots = setup_roots(&ws);
        let index = build_multi_entry_index(&roots);
        let fixture_abs = abs(&ws, FIXTURE);
        let diff = diff_at(&abs(&ws, OUTSIDE_FILE), OUTSIDE_DATA_LINE);
        let diff_edits =
            floor_diff_edits_from_diff_text(&index, &diff).expect("seeds from outside-file diff");
        let declared = index.module_graph_facts.declared_repo_paths();
        let touched_paths: Vec<String> = diff_edits.touched_entry_files.iter().cloned().collect();
        assert!(
            !super::effect_reach_touched_via_path_literals(
                &fixture_abs,
                &index.module_graph_facts,
                &touched_paths,
            ),
            "hermetic fixture must not match outside-diff path literals"
        );
        assert!(
            super::entry_qualifies_for_skip_without_resolve(
                &fixture_abs,
                false,
                &index.module_graph_facts,
                &declared,
                &touched_paths,
                &diff_edits,
            )
            .expect("qualify"),
            "absence of literal match must not change skip eligibility for hermetic entry"
        );
    }

    #[test]
    fn declared_source_refs_touch_blocks_skip_for_03_normalize_witness() {
        let ws = workspace_root();
        std::env::set_current_dir(&ws).expect("chdir workspace");
        let roots = setup_roots(&ws);
        let index = build_multi_entry_index(&roots);
        let entry = "dag/test/claim/self_host_03_normalize_behavioral_witness_test.dag";
        let entry_abs = abs(&ws, entry);
        let diff = diff_at(&abs(&ws, OUTSIDE_FILE), OUTSIDE_DATA_LINE);
        let diff_edits =
            floor_diff_edits_from_diff_text(&index, &diff).expect("seeds from outside-file diff");
        let declared = index.module_graph_facts.declared_repo_paths();
        let touched = vec!["src/v2/compiler/03_normalize.dag".to_string()];
        assert!(
            !super::entry_qualifies_for_skip_without_resolve(
                &entry_abs,
                false,
                &index.module_graph_facts,
                &declared,
                &touched,
                &diff_edits,
            )
            .expect("qualify"),
            "declared-source-ref touch must convert would-skip into run"
        );
    }

    #[test]
    fn declared_source_refs_unrelated_diff_skips_03_normalize_witness() {
        let ws = workspace_root();
        std::env::set_current_dir(&ws).expect("chdir workspace");
        let roots = setup_roots(&ws);
        let index = build_multi_entry_index(&roots);
        let entry = "dag/test/claim/self_host_03_normalize_behavioral_witness_test.dag";
        let entry_abs = abs(&ws, entry);
        let diff = diff_at(&abs(&ws, OUTSIDE_FILE), OUTSIDE_DATA_LINE);
        let diff_edits =
            floor_diff_edits_from_diff_text(&index, &diff).expect("seeds from outside-file diff");
        let declared = index.module_graph_facts.declared_repo_paths();
        let touched: Vec<String> = diff_edits.touched_entry_files.iter().cloned().collect();
        assert!(
            super::entry_qualifies_for_skip_without_resolve(
                &entry_abs,
                false,
                &index.module_graph_facts,
                &declared,
                &touched,
                &diff_edits,
            )
            .expect("qualify"),
            "unrelated diff must skip the 03_normalize behavioral witness via declared refs"
        );
    }

    // Touch bridge must match data-init path literals only — struct/path fields in
    // unrelated entries must not widen selection (floor_skip_discovery_witness receipt).
    #[test]
    fn effect_reach_touched_ignores_non_data_path_mentions() {
        let ws = workspace_root();
        std::env::set_current_dir(&ws).expect("chdir workspace");
        let roots = setup_roots(&ws);
        let facts = super::build_module_graph_facts_live(&roots);
        let runner = "src/v2/workflow/affected_set_floor_runner_test.dag";
        let fixture_path = "src/v2/test/fixture/floor_skip/node_precise_discriminator_test.dag";
        assert!(
            !super::effect_reach_touched_via_path_literals(
                runner,
                &facts,
                &[fixture_path.to_string()],
            ),
            "struct/path fixture mentions must not count as effect-reach touch evidence"
        );
    }

    // RED guard: data-item edits in the entry import closure must not fast-skip — the
    // node-frontier machinery needs resolve to discriminate referenced nodes.
    #[test]
    fn skip_without_resolve_fast_path_ineligible_for_referenced_data_item() {
        let ws = workspace_root();
        std::env::set_current_dir(&ws).expect("chdir workspace");
        let roots = setup_roots(&ws);
        let index = build_multi_entry_index(&roots);
        let fixture_abs = abs(&ws, FIXTURE);
        let text = std::fs::read_to_string(&fixture_abs).expect("fixture readable");
        let data_line = text
            .lines()
            .position(|l| l.contains("data floor_disc_node_a"))
            .map(|i| (i + 1) as i64)
            .expect("floor_disc_node_a line");
        let diff = diff_at(&fixture_abs, data_line);
        let diff_edits = floor_diff_edits_from_diff_text(&index, &diff)
            .expect("seeds from referenced-node diff");
        assert!(
            !diff_edits.overlapping_data_items.is_empty(),
            "data-item diff must populate overlapping_data_items"
        );
        let declared = index.module_graph_facts.declared_repo_paths();
        let touched_paths: Vec<String> = diff_edits.touched_entry_files.iter().cloned().collect();
        assert!(
            !super::entry_qualifies_for_skip_without_resolve(
                &fixture_abs,
                false,
                &index.module_graph_facts,
                &declared,
                &touched_paths,
                &diff_edits,
            )
            .expect("qualify"),
            "entry must NOT qualify for skip-before-resolve when diff edits a data item in its import closure"
        );
    }

    // Control 2 (RED/function_edited): diff edits a test fn declaration →
    // edited_test_fns populated → function_edited=true forces run for that row.
    #[test]
    fn red_function_edited_populates_edited_test_fns() {
        let ws = workspace_root();
        std::env::set_current_dir(&ws).expect("chdir workspace");
        let roots = setup_roots(&ws);
        let index = build_multi_entry_index(&roots);
        let text = std::fs::read_to_string(FIXTURE).expect("fixture readable");
        let test_fn_line = text
            .lines()
            .position(|l| l.contains("test fn floor_disc_witness_a_only_holds"))
            .map(|i| (i + 1) as i64)
            .expect("witness A test fn line");
        let diff = diff_at(FIXTURE, test_fn_line);
        let seeds =
            floor_diff_edits_from_diff_text(&index, &diff).expect("seeds from test-fn-line diff");
        assert!(
            seeds
                .edited_test_fns
                .iter()
                .any(|(_, name)| name == "floor_disc_witness_a_only_holds"),
            "diff at test fn declaration line must populate edited_test_fns with the function name"
        );
    }

    #[test]
    fn deletion_only_hunk_populates_edited_test_fns() {
        let ws = workspace_root();
        std::env::set_current_dir(&ws).expect("chdir workspace");
        let roots = setup_roots(&ws);
        let index = build_multi_entry_index(&roots);
        let text = std::fs::read_to_string(FIXTURE).expect("fixture readable");
        let test_fn_line = text
            .lines()
            .position(|l| l.contains("test fn floor_disc_witness_a_only_holds"))
            .map(|i| (i + 1) as i64)
            .expect("witness A test fn line");
        let diff = deletion_diff_at(FIXTURE, test_fn_line);
        let seeds =
            floor_diff_edits_from_diff_text(&index, &diff).expect("seeds from deletion-only diff");
        assert!(
            seeds
                .edited_test_fns
                .iter()
                .any(|(_, name)| name == "floor_disc_witness_a_only_holds"),
            "deletion-only diff at test fn line must populate edited_test_fns"
        );
    }

    // Control 3 (RED/node_frontier): diff on a data item referenced by a claim →
    // entry_touches_rerun_frontier returns true → runs.
    #[test]
    fn red_node_frontier_fires_for_referenced_data_item() {
        let ws = workspace_root();
        let roots = setup_roots(&ws);
        let index = build_multi_entry_index(&roots);
        let fixture_abs = abs(&ws, FIXTURE);
        let text = std::fs::read_to_string(&fixture_abs).expect("fixture readable");
        let data_line = text
            .lines()
            .position(|l| l.contains("data floor_disc_node_a"))
            .map(|i| (i + 1) as i64)
            .expect("floor_disc_node_a line");
        let diff = diff_at(&fixture_abs, data_line);
        let seeds = floor_diff_edits_from_diff_text(&index, &diff)
            .expect("seeds from referenced-node diff");
        let (graph, source_indices) =
            super::resolve_entry_with_index(&index, &fixture_abs).expect("fixture resolves");
        let ctx = super::make_eval_context(&graph, source_indices, ExecutionMode::Wet);
        let nodes = rerun_frontier_nodes_for_entry(&ctx, &fixture_abs, &seeds).expect("nodes");
        assert!(
            entry_touches_rerun_frontier(&ctx, &list_value_from_vec(nodes)).expect("touch check"),
            "entry must touch frontier when diff is on a data item referenced by a claim"
        );
    }

    // Control 4 (entry_file_touched / ROADMAP 1-affected-set-defork acceptance (a)):
    // non-data, non-test-fn declaration edit scopes runs to that entry only — the touched
    // entry's roster runs via `entry_file_touched`; unrelated entries skip when frontier empty.
    #[test]
    fn green_entry_file_helper_fn_edit_scopes_to_same_entry_only() {
        let ws = workspace_root();
        std::env::set_current_dir(&ws).expect("chdir workspace");
        let roots = setup_roots(&ws);
        let index = build_multi_entry_index(&roots);
        let fixture_abs = abs(&ws, FIXTURE);
        let text = std::fs::read_to_string(&fixture_abs).expect("fixture readable");
        let helper_line = text
            .lines()
            .position(|l| l.contains("fn floor_disc_helper_fn"))
            .map(|i| (i + 1) as i64)
            .expect("helper fn line");
        let diff = diff_at(&fixture_abs, helper_line);
        let seeds =
            floor_diff_edits_from_diff_text(&index, &diff).expect("seeds from helper-fn-line diff");
        assert!(
            seeds
                .touched_entry_files
                .iter()
                .any(|f| f.contains("node_precise_discriminator")),
            "helper fn edit must populate touched_entry_files"
        );
        assert!(
            seeds.overlapping_data_items.is_empty() && seeds.edited_test_fns.is_empty(),
            "helper fn edit must not populate data-item frontier or edited_test_fns"
        );

        let (graph, source_indices) =
            super::resolve_entry_with_index(&index, &fixture_abs).expect("fixture resolves");
        let entry_ctx = super::make_eval_context(&graph, source_indices, ExecutionMode::Wet);
        let nodes =
            rerun_frontier_nodes_for_entry(&entry_ctx, &fixture_abs, &seeds).expect("nodes");
        assert!(
            nodes.is_empty(),
            "helper fn edit must not materialize data-item frontier nodes"
        );

        let (runner_graph, runner_indices) = super::resolve_entry_with_index(
            &index,
            &abs(&ws, "src/v2/workflow/affected_set_floor_runner.dag"),
        )
        .expect("floor runner resolves");
        let runner_ctx =
            super::make_eval_context(&runner_graph, runner_indices, ExecutionMode::Wet);
        let changed_paths = vec![fixture_abs.clone()];
        assert!(
            !call_floor_kernel_would_skip(
                &runner_ctx,
                &changed_paths,
                &nodes,
                false,
                false,
                true,
                false
            )
            .expect("skip verdict for touched entry"),
            "helper-fn edit must RUN witnesses in the touched entry (entry_file_touched)"
        );
        assert!(
            call_floor_kernel_would_skip(
                &runner_ctx,
                &changed_paths,
                &nodes,
                false,
                false,
                false,
                false
            )
            .expect("skip verdict for unrelated entry"),
            "helper-fn edit must SKIP witnesses in an unrelated entry when frontier is empty"
        );
    }

    // Control 4b (entry_file_touched / import-closure): non-data fn edit in an imported
    // module runs witnesses in the importing entry, not only when the entry file itself changed.
    #[test]
    fn green_import_closure_helper_fn_edit_runs_importer_entry() {
        let ws = workspace_root();
        std::env::set_current_dir(&ws).expect("chdir workspace");
        let roots = setup_roots(&ws);
        let index = build_multi_entry_index(&roots);
        let helper_rel = "src/v2/test/fixture/floor_skip/floor_disc_shared_helper.dag";
        let fixture_abs = abs(&ws, FIXTURE);
        let helper_abs = abs(&ws, helper_rel);
        let text = std::fs::read_to_string(&helper_abs).expect("shared helper readable");
        let helper_line = text
            .lines()
            .position(|l| l.contains("fn floor_disc_shared_helper"))
            .map(|i| (i + 1) as i64)
            .expect("shared helper fn line");
        let diff = diff_at(&helper_abs, helper_line);
        let seeds = floor_diff_edits_from_diff_text(&index, &diff)
            .expect("seeds from cross-file helper-fn diff");
        assert!(
            seeds
                .touched_entry_files
                .iter()
                .any(|f| f.contains("floor_disc_shared_helper")),
            "cross-file helper fn edit must populate touched_entry_files"
        );

        let (graph, source_indices) =
            super::resolve_entry_with_index(&index, &fixture_abs).expect("fixture resolves");
        let closure_files = super::import_closure_files_from_graph(&graph);
        assert!(
            super::touched_file_in_import_closure(&helper_abs, &closure_files),
            "shared helper module must be in fixture import closure"
        );
        let entry_ctx = super::make_eval_context(&graph, source_indices, ExecutionMode::Wet);
        let nodes =
            rerun_frontier_nodes_for_entry(&entry_ctx, &fixture_abs, &seeds).expect("nodes");
        assert!(
            nodes.is_empty(),
            "cross-file helper fn edit must not materialize data-item frontier nodes"
        );

        let (runner_graph, runner_indices) = super::resolve_entry_with_index(
            &index,
            &abs(&ws, "src/v2/workflow/affected_set_floor_runner.dag"),
        )
        .expect("floor runner resolves");
        let runner_ctx =
            super::make_eval_context(&runner_graph, runner_indices, ExecutionMode::Wet);
        let changed_paths = vec![helper_abs.clone()];
        assert!(
            !call_floor_kernel_would_skip(
                &runner_ctx,
                &changed_paths,
                &nodes,
                false,
                false,
                true,
                false
            )
            .expect("skip verdict for importing entry"),
            "cross-file helper-fn edit must RUN witnesses in importing entry (import-closure entry_file_touched)"
        );
        assert!(
            call_floor_kernel_would_skip(&runner_ctx, &changed_paths, &nodes, false, false, false, false)
                .expect("skip verdict for unrelated entry"),
            "cross-file helper-fn edit must SKIP witnesses in an unrelated entry when frontier is empty"
        );
    }

    // Control 5 (structural-∅, not fail-closed): an exclusively non-.dag diff is a nominal
    // empty .dag frontier, NOT a refusal (#6269 dropped the saw_non_dag arm). This behavior is
    // asserted, CWD-safely, by `non_dag_only_diff_is_structural_empty_frontier_not_refusal`
    // above; the former `fail_closed_non_dag_file_forces_run_all` here tested the pre-#6269
    // fail-closed semantics and was deleted as a stale, contradictory twin (2026-07-07).

    // Control 5 (fail-closed): diff before first declaration in a .dag file → fail-closed.
    // The module header (line 1) precedes the first data/fn declaration.
    #[test]
    fn fail_closed_edit_before_first_decl_forces_run_all() {
        let ws = workspace_root();
        let roots = setup_roots(&ws);
        let index = build_multi_entry_index(&roots);
        let diff = diff_at(&abs(&ws, FIXTURE), 1);
        let err = floor_diff_edits_from_diff_text(&index, &diff)
            .expect_err("diff before first declaration must fail-closed");
        assert!(
            err.contains("before first declaration"),
            "expected pre-decl fail-closed, got: {err}"
        );
    }

    #[test]
    fn wholly_new_dag_file_does_not_fail_closed_on_module_line() {
        let ws = workspace_root();
        std::env::set_current_dir(&ws).expect("chdir workspace");
        let roots = setup_roots(&ws);
        let index = build_multi_entry_index(&roots);
        let rel = "src/v2/test/claim/manual/integer_census_stage_receipt.dag";
        let content = std::fs::read_to_string(rel).expect("read receipt");
        let line_count = content.lines().count();
        let mut diff = format!(
            "diff --git a/{rel} b/{rel}\nnew file mode 100644\n--- /dev/null\n+++ b/{rel}\n"
        );
        diff.push_str(&format!("@@ -0,0 +1,{line_count} @@\n"));
        for line in content.lines() {
            diff.push('+');
            diff.push_str(line);
            diff.push('\n');
        }
        floor_diff_edits_from_diff_text(&index, &diff)
            .unwrap_or_else(|e| panic!("wholly new receipt file must not fail-closed: {e}"));
    }

    // Rename destination (git `rename to NEW`) is new-at-path: a module-header (line 1)
    // change on the TO-side is the wholly-added case, not an in-place module rename, so it
    // must NOT fail-closed. Mirrors `wholly_new_dag_file_does_not_fail_closed_on_module_line`
    // for the rename+modify shape that a file rename produces.
    #[test]
    fn rename_destination_module_line_change_does_not_fail_closed() {
        let ws = workspace_root();
        std::env::set_current_dir(&ws).expect("chdir workspace");
        let roots = setup_roots(&ws);
        let index = build_multi_entry_index(&roots);
        let new_rel = "src/v2/test/claim/manual/integer_census_stage_receipt.dag";
        let old_rel = "src/v2/test/claim/manual/integer_census_stage_receipt_old_name.dag";
        // rename+modify whose only hunk edits line 1 (the module header) — the shape a
        // module rename produces. Without the `rename to` added-side signal this fails-closed.
        let diff = format!(
            "diff --git a/{old_rel} b/{new_rel}\n\
             similarity index 99%\n\
             rename from {old_rel}\n\
             rename to {new_rel}\n\
             --- a/{old_rel}\n\
             +++ b/{new_rel}\n\
             @@ -1,1 +1,1 @@\n\
             -module v2.test.manual.integer_census_stage_receipt_old_name\n\
             +module v2.test.manual.integer_census_stage_receipt\n"
        );
        floor_diff_edits_from_diff_text(&index, &diff).unwrap_or_else(|e| {
            panic!("rename destination line-1 change must not fail-closed: {e}")
        });
        // Control: the SAME line-1 change as an in-place modify (no `rename to`) stays fail-closed.
        let in_place = format!(
            "diff --git a/{new_rel} b/{new_rel}\n\
             --- a/{new_rel}\n\
             +++ b/{new_rel}\n\
             @@ -1,1 +1,1 @@\n\
             -module v2.test.manual.integer_census_stage_receipt\n\
             +module v2.test.manual.integer_census_stage_receipt_renamed\n"
        );
        let err = floor_diff_edits_from_diff_text(&index, &in_place)
            .expect_err("in-place module-line modify must stay fail-closed");
        assert!(
            err.contains("before first declaration"),
            "expected pre-decl fail-closed for in-place modify, got: {err}"
        );
    }

    #[test]
    fn import_preamble_plus_fn_body_populates_touched_entry_not_fail_closed() {
        let ws = workspace_root();
        std::env::set_current_dir(&ws).expect("chdir workspace");
        let roots = setup_roots(&ws);
        let index = build_multi_entry_index(&roots);
        let emit_rel = "dag/extdeps/languages/json/emit.dag";
        let diff = include_str!("../testdata/emit_import_preamble_fn_body.diff");
        let edits = floor_diff_edits_from_diff_text(&index, &diff)
            .expect("import+fn diff must not fail-closed to full corpus");
        assert!(
            edits.touched_entry_files.iter().any(|f| f == emit_rel),
            "import preamble + fn body must touch the entry file"
        );
    }

    #[test]
    fn mixed_dag_and_non_dag_diff_scopes_from_dag_only() {
        let ws = workspace_root();
        std::env::set_current_dir(&ws).expect("chdir workspace");
        let roots = setup_roots(&ws);
        let index = build_multi_entry_index(&roots);
        let emit_rel = "dag/extdeps/languages/json/emit.dag";
        let dag_diff = include_str!("../testdata/emit_import_preamble_fn_body.diff");
        let host_diff =
            "diff --git a/src/v1/stage0/src/cli_run.rs b/src/v1/stage0/src/cli_run.rs\n\
                          --- a/src/v1/stage0/src/cli_run.rs\n\
                          +++ b/src/v1/stage0/src/cli_run.rs\n\
                          @@ -1,0 +2,1 @@\n+// synthetic\n";
        let diff = format!("{dag_diff}\n{host_diff}");
        let edits = floor_diff_edits_from_diff_text(&index, &diff)
            .expect("mixed dag+host diff must scope from .dag paths only");
        assert!(
            edits.touched_entry_files.iter().any(|f| f == emit_rel),
            "mixed diff must still attribute .dag frontier seeds"
        );
    }

    // Control 6 (fail-closed / Q2): diff names a .dag path with content changes that is
    // absent from the tree and NOT marked departed by the diff → typed refusal
    // (observation incoherence), never silently absorbed as a departure.
    #[test]
    fn fail_closed_nonexistent_dag_path_refuses() {
        let ws = workspace_root();
        let roots = setup_roots(&ws);
        let index = build_multi_entry_index(&roots);
        let diff = diff_at(&abs(&ws, "src/v2/lens/does_not_exist_sentinel.dag"), 10);
        let err = floor_diff_edits_from_diff_text(&index, &diff)
            .expect_err("diff naming a non-existent, non-departed .dag path must refuse");
        assert!(
            err.contains("absent from the working tree"),
            "expected observation-incoherence refusal, got: {err}"
        );
    }

    // Control 6b (departure is the diff's fact): a deletion-shaped diff for the same
    // absent path attributes at path grain with an empty decl set — Ok, no refusal.
    #[test]
    fn deletion_shaped_diff_for_absent_path_attributes_path_grain() {
        let ws = workspace_root();
        let roots = setup_roots(&ws);
        let index = build_multi_entry_index(&roots);
        let gone = abs(&ws, "src/v2/lens/does_not_exist_sentinel.dag");
        let diff = format!(
            "diff --git a/{gone} b/{gone}\n--- a/{gone}\n+++ /dev/null\n@@ -1,3 +0,0 @@\n-a\n-b\n-c\n"
        );
        let edits = floor_diff_edits_from_diff_text(&index, &diff)
            .expect("deletion-shaped diff for an absent path must not refuse");
        assert!(
            edits.edited_test_fns.is_empty() && edits.overlapping_data_items.is_empty(),
            "departed path has a structurally-empty decl set"
        );
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceRootReadRecord {
    pub file_path: String,
    pub module_path: String,
    pub source: String,
    pub source_root: String,
}

fn source_root_ref_variant_for_root(root: &str) -> Result<String, String> {
    match root.trim_end_matches('/') {
        "src/v2" => Ok("V2Tree".to_string()),
        "dag" => Ok("DagTree".to_string()),
        other => Err(format!(
            "source_root tagging: unknown --source-root '{other}' \
             (authority gunbc.ci_layer_roots.witness_layer_roots = [src/v2, dag] -> \
             SourceRootRef {{V2Tree, DagTree}})"
        )),
    }
}

fn source_root_ref_token_for_path(
    file_path: &str,
    source_roots: &[String],
) -> Result<String, String> {
    let rel_path = repo_relative_dag_path(file_path);
    let matched: Vec<String> = source_roots
        .iter()
        .map(|r| repo_relative_dag_path(r))
        .filter(|r| {
            let r = r.trim_end_matches('/');
            rel_path == r || rel_path.starts_with(&format!("{r}/"))
        })
        .collect();
    match matched.as_slice() {
        [] => Err(format!(
            "source_root tagging: file '{file_path}' (repo-relative '{rel_path}') matches no \
             --source-root {source_roots:?}"
        )),
        [one] => source_root_ref_variant_for_root(one),
        _ => Err(format!(
            "source_root tagging: file '{file_path}' matches multiple --source-root {matched:?}"
        )),
    }
}

fn source_root_ingest_symbol_from_stem(stem: &str) -> String {
    let mut body = String::new();
    for ch in stem.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            body.push(ch);
        } else {
            body.push('_');
        }
    }
    if body.is_empty() {
        body.push_str("host_sr_empty");
    } else if body.as_bytes()[0].is_ascii_digit() {
        body = format!("sr_{body}");
    }
    format!("^{body}")
}

pub fn source_root_ingest_artifact_id_for_path(path: &str) -> String {
    let stem = Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("host_sr");
    source_root_ingest_symbol_from_stem(stem)
}

fn source_root_ingest_compilation_unit_for_path(path: &str) -> String {
    let stem = Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("host_sr");
    source_root_ingest_symbol_from_stem(stem)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceRootEntryAdmission {
    pub subject: Vec<String>,
    pub imports: Vec<Vec<String>>,
}

fn parse_dotted_module_path(path: &str) -> Option<Vec<String>> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return None;
    }
    let segments: Vec<String> = trimmed
        .split('.')
        .filter(|seg| !seg.is_empty())
        .map(str::to_string)
        .collect();
    if segments.is_empty() {
        None
    } else {
        Some(segments)
    }
}

pub fn parse_source_root_entry_admission(source: &str) -> Result<SourceRootEntryAdmission, String> {
    let mut subject: Option<Vec<String>> = None;
    let mut imports: Vec<Vec<String>> = Vec::new();
    let mut seen = std::collections::BTreeSet::new();

    for line in source.lines() {
        let line = line.split("//").next().unwrap_or("").trim();
        if let Some(rest) = line.strip_prefix("module ") {
            subject = parse_dotted_module_path(rest);
        } else if let Some(rest) = line.strip_prefix("import ") {
            let module_path = rest.split_whitespace().next().unwrap_or("");
            if let Some(segments) = parse_dotted_module_path(module_path) {
                if seen.insert(segments.clone()) {
                    imports.push(segments);
                }
            }
        }
    }

    subject
        .map(|subject| SourceRootEntryAdmission { subject, imports })
        .ok_or_else(|| "entry source missing `module` declaration".to_string())
}

fn free_monoid_symbol_emit_dag(segments: &[String]) -> String {
    if segments.is_empty() {
        return "Empty".to_string();
    }
    let mut out = String::from("Empty");
    for seg in segments.iter().rev() {
        out = format!("Cons {{ head: ^{seg}, tail: {out} }}");
    }
    out
}

#[cfg(test)]
mod manifest_emit_tests {
    use super::{
        dag_embedded_dag_source_escape, dag_manifest_scalar_escape, free_monoid_symbol_emit_dag,
    };

    #[test]
    fn free_monoid_symbol_emit_dag_three_segment_path() {
        assert_eq!(
            free_monoid_symbol_emit_dag(&["v2".into(), "compiler".into(), "compile".into()]),
            "Cons { head: ^v2, tail: Cons { head: ^compiler, tail: Cons { head: ^compile, tail: Empty } } }"
        );
    }

    #[test]
    fn free_monoid_symbol_emit_dag_empty_is_empty_variant() {
        assert_eq!(free_monoid_symbol_emit_dag(&[]), "Empty");
    }

    #[test]
    fn manifest_scalar_escape_rejects_braces() {
        assert!(dag_manifest_scalar_escape("src/v2/foo.dag").is_ok());
        assert!(dag_manifest_scalar_escape("fnv1a64:abc").is_ok());
        assert!(dag_manifest_scalar_escape("has{brace").is_err());
        assert!(dag_manifest_scalar_escape("has}brace").is_err());
    }

    #[test]
    fn embedded_dag_source_escape_preserves_braces_as_escapes() {
        assert_eq!(
            dag_embedded_dag_source_escape("match x { A => 1 }"),
            "match x \\{ A => 1 \\}"
        );
    }

    use super::source_root_ref_token_for_path;

    #[test]
    fn source_root_token_grounds_in_filesystem_location() {
        let roots = vec!["src/v2".to_string(), "dag".to_string()];
        assert_eq!(
            source_root_ref_token_for_path("src/v2/std/algebra.dag", &roots).unwrap(),
            "V2Tree"
        );
        assert_eq!(
            source_root_ref_token_for_path("dag/std/algebra.dag", &roots).unwrap(),
            "DagTree"
        );
        assert_eq!(
            source_root_ref_token_for_path("src/v2/extdeps/shell.dag", &roots).unwrap(),
            "V2Tree"
        );
        assert!(source_root_ref_token_for_path("src/v1/stage0/x.dag", &roots).is_err());
        assert!(source_root_ref_token_for_path("src/v20/x.dag", &roots).is_err());
    }

    #[test]
    fn source_root_token_admits_absolute_roots() {
        let ws = super::workspace_root();
        let abs_roots = vec![
            ws.join("src/v2").to_string_lossy().into_owned(),
            ws.join("dag").to_string_lossy().into_owned(),
        ];
        assert_eq!(
            source_root_ref_token_for_path(
                ws.join("src/v2/std/algebra.dag").to_str().unwrap(),
                &abs_roots
            )
            .unwrap(),
            "V2Tree"
        );
        assert_eq!(
            source_root_ref_token_for_path(
                ws.join("dag/std/algebra.dag").to_str().unwrap(),
                &abs_roots
            )
            .unwrap(),
            "DagTree"
        );
        assert_eq!(
            source_root_ref_token_for_path("dag/std/algebra.dag", &abs_roots).unwrap(),
            "DagTree"
        );
        assert!(source_root_ref_token_for_path(
            ws.join("src/v1/stage0/x.dag").to_str().unwrap(),
            &abs_roots
        )
        .is_err());
    }
}

fn emit_import_admission_list(imports: &[Vec<String>]) -> String {
    let mut out = String::from("Empty");
    for import in imports.iter().rev() {
        out = format!(
            "Cons {{\n  head: Import {{\n    target: {},\n    visibility: ImportVisible\n  }},\n  tail: {out}\n}}",
            free_monoid_symbol_emit_dag(import)
        );
    }
    out
}

fn emit_source_root_entry_admission_data(admission: &SourceRootEntryAdmission) -> String {
    format!(
        "data host_compiler_closure_admission: Admission = Admission {{\n  subject: ResolutionSubject {{\n    name: {}\n  }},\n  imports: {}\n}}\n\n\n",
        free_monoid_symbol_emit_dag(&admission.subject),
        emit_import_admission_list(&admission.imports)
    )
}

pub fn source_root_ingest_content_hash_fnv1a64(records: &[SourceRootReadRecord]) -> String {
    let mut material = String::new();
    for rec in records {
        material.push_str(&rec.file_path);
        material.push('\0');
        material.push_str(&rec.source);
        material.push('\0');
    }
    let mut hash = 0xcbf29ce484222325u64;
    for byte in material.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("fnv1a64:{hash:016x}")
}

fn path_matches_any_subpath(path: &str, subpaths: &[String]) -> bool {
    subpaths
        .iter()
        .any(|sub| path.contains(sub) || path.ends_with(sub))
}

pub fn discover_source_root_reads(
    source_roots: &[String],
    scan_dir: &str,
    exclude_subpaths: &[String],
) -> Result<Vec<SourceRootReadRecord>, String> {
    for root in source_roots {
        let root_path = Path::new(root);
        if !root_path.exists() {
            return Err(format!(
                "discover_source_root_ingest: source root does not exist: {}",
                root
            ));
        }
    }

    let scan_path = Path::new(scan_dir);
    if !scan_path.is_dir() {
        return Err(format!(
            "discover_source_root_ingest: scan dir does not exist: {}",
            scan_dir
        ));
    }

    let mut records: Vec<SourceRootReadRecord> = Vec::new();
    let mut seen_modules: HashMap<String, String> = HashMap::new();
    let mut dag_files = Vec::new();
    collect_dag_files(scan_path, &mut dag_files);

    for path in dag_files {
        let rel_forward = path.to_string_lossy().replace('\\', "/");
        if path_matches_any_subpath(&rel_forward, exclude_subpaths) {
            continue;
        }
        let content = std::fs::read_to_string(&path)
            .map_err(|e| format!("failed to read {:?}: {}", path, e))?;
        let module_path = extract_module_path(&content).ok_or_else(|| {
            format!(
                "discover_source_root_ingest: no module declaration in {}",
                rel_forward
            )
        })?;
        if let Some(prior) = seen_modules.insert(module_path.clone(), rel_forward.clone()) {
            return Err(format!(
                "discover_source_root_ingest: duplicate module path '{}' in {} and {}",
                module_path, prior, rel_forward
            ));
        }
        let source_root = source_root_ref_token_for_path(&rel_forward, source_roots)?;
        records.push(SourceRootReadRecord {
            file_path: rel_forward,
            module_path,
            source: content,
            source_root,
        });
    }

    records.sort_by(|a, b| a.file_path.cmp(&b.file_path));
    Ok(records)
}

pub fn discover_source_root_reads_for_entry(
    source_roots: &[String],
    entry_path: &str,
    exclude_subpaths: &[String],
) -> Result<Vec<SourceRootReadRecord>, String> {
    for root in source_roots {
        let root_path = Path::new(root);
        if !root_path.exists() {
            return Err(format!(
                "discover_source_root_ingest: source root does not exist: {}",
                root
            ));
        }
    }

    let closure = load_sources_for_entry(source_roots, entry_path)
        .map_err(|msg| format!("discover_source_root_ingest: entry closure load failed: {msg}"))?;

    let mut records: Vec<SourceRootReadRecord> = Vec::new();
    for source in closure {
        let rel_forward = source.path.replace('\\', "/");
        if path_matches_any_subpath(&rel_forward, exclude_subpaths) {
            continue;
        }
        let module_path = extract_module_path(&source.content).ok_or_else(|| {
            format!(
                "discover_source_root_ingest: no module declaration in {}",
                rel_forward
            )
        })?;
        let source_root = source_root_ref_token_for_path(&rel_forward, source_roots)?;
        records.push(SourceRootReadRecord {
            file_path: rel_forward,
            module_path,
            source: source.content.clone(),
            source_root,
        });
    }

    records.sort_by(|a, b| a.file_path.cmp(&b.file_path));
    Ok(records)
}

fn emit_source_root_read_witness(rec: &SourceRootReadRecord) -> Result<String, String> {
    let artifact_id = source_root_ingest_artifact_id_for_path(&rec.file_path);
    let compilation_unit = source_root_ingest_compilation_unit_for_path(&rec.file_path);
    Ok(format!(
        "DagSourceReadWitness {{\n  source: Medium {{ carried: \"{}\", fidelity: Lossless }},\n  artifact: Artifact {{\n    kind: SourceFile,\n    id: {artifact_id},\n    file_path: \"{}\"\n  }},\n  compilation_unit: {compilation_unit},\n  source_root: {}\n}}",
        dag_embedded_dag_source_escape(&rec.source),
        dag_manifest_scalar_escape(&rec.file_path)?,
        rec.source_root,
    ))
}

fn emit_source_root_ingest_monoid(records: &[SourceRootReadRecord]) -> Result<String, String> {
    let mut witness_nodes: Vec<String> = records
        .iter()
        .map(emit_source_root_read_witness)
        .collect::<Result<_, _>>()?;
    let mut out = String::from("Empty");
    while let Some(head) = witness_nodes.pop() {
        out = format!("Cons {{\n  head: {head},\n  tail: {out}\n}}");
    }
    Ok(out)
}

fn emit_source_root_ref_import(records: &[SourceRootReadRecord]) -> String {
    let mut variants: Vec<&str> = records.iter().map(|r| r.source_root.as_str()).collect();
    variants.sort_unstable();
    variants.dedup();
    if variants.is_empty() {
        return String::new();
    }
    format!(
        "import v2.std.cross_tree.import_model {{ {} }}\n",
        variants.join(", ")
    )
}

/// Emit the module-binding manifest: the host handler for the `.dag`-modeled op
/// `v2.compiler.source_authority.module_storage_bindings_for_source_roots`.
///
/// This is a TRANSPORT of that modeled op, not a rival authority. It carries zero
/// independent policy: it serializes the same parse-derived rows as `build_module_path_index`
/// via `collect_module_binding_manifest_rows` (shared `for_each_parsed_module_binding` walk),
/// which is the one host producer the module-identity design says must be repointed —
/// so supplying the rows and repointing the producer are the same motion.
///
/// Rows are `ParsedFromSource`: `build_module_path_index` routes through
/// `v1_compiler_parse::parse` (src/v1/stage0/src/module_path_index), the bootstrap
/// parse path — not `extract_module_path` substring scan (task 4 repoint).
///
/// Unlike the source-root ingest manifest this carries NO source text — the binding needs
/// module <-> path only. That is what lets it scale past `MANIFEST_INLINE_LIST_MAX`, which
/// exists to stop the ingest manifest from inlining the corpus.
///
/// Dissolve-on: host-effect emission (witness-realization lane), at which point this
/// handler is emitted from the `.dag` model instead of hand-written here.
pub fn emit_module_storage_binding_manifest(
    path: &Path,
    source_roots: &[String],
) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("failed to create manifest parent {:?}: {}", parent, e))?;
    }

    let mut rows = collect_module_binding_manifest_rows(source_roots);
    rows.sort_by(|a, b| a.module_path.cmp(&b.module_path));

    let mut out = String::new();
    out.push_str("module v2.test.workflow.host_module_binding_manifest\n\n\n");
    out.push_str("import v2.compiler.source_authority {\n");
    out.push_str("  ModuleStorageIndex,\n");
    out.push_str("  module_storage_parsed_binding\n");
    out.push_str("}\n");
    out.push_str("import v2.std.artifact { Artifact, SourceFile }\n");
    out.push_str("import std.algebra { Cons, Empty }\n");
    out.push_str("import v2.std.diagnostic { ByteRange, Textual }\n");
    out.push_str("import v2.std.integer { Int }\n");
    out.push_str("import v2.std.node { MintedOccurrence, OccurrenceId }\n");
    out.push_str("import v2.std.provenance { FromSource, span_index_empty, span_index_record }\n");
    out.push_str("import v2.std.qualified_name { qualified_name_from_string_segments }\n");
    out.push_str(&emit_module_binding_source_root_import(&rows));
    out.push('\n');
    out.push_str(&format!(
        "data host_module_binding_count: Int = {}\n\n\n",
        rows.len()
    ));
    out.push_str("data host_module_bindings: ModuleStorageIndex = ");
    out.push_str(&emit_module_binding_monoid(&rows)?);
    out.push('\n');

    std::fs::write(path, out).map_err(|e| format!("failed to write manifest {:?}: {}", path, e))
}

/// Import exactly the `SourceRootRef` constructors the rows reference (mirrors
/// `emit_source_root_ref_import`; an unreferenced constructor import is an unlisted-import
/// error, and a referenced-but-unimported one fails to resolve).
fn emit_module_binding_source_root_import(rows: &[ModuleBindingManifestRow]) -> String {
    let mut names: Vec<&str> = rows.iter().map(|r| r.root_variant.as_str()).collect();
    names.sort_unstable();
    names.dedup();
    if names.is_empty() {
        return String::new();
    }
    format!(
        "import v2.std.cross_tree.import_model {{ {} }}\n",
        names.join(", ")
    )
}

/// Render a dotted module path as a `QualifiedName`, via the std construction authority
/// `qualified_name_from_string_segments`.
///
/// Deliberately NOT `^segment` symbol literals: module segments may collide with `.dag`
/// keywords (`v2.test.claim.compiler.pipeline.corpus` emits `^pipeline`, which is a parse
/// error), and the `^(...)` form is discriminant sugar with different semantics, not an
/// escape hatch. Going through the std helper takes segments as STRINGS, so keywords are
/// inert, and it reuses the one construction authority instead of hand-rolling a second
/// spelling of the same value (DESIGN.md §3).
fn emit_module_binding_qualified_name(module_path: &str) -> Result<String, String> {
    let segments: Vec<&str> = module_path.split('.').filter(|s| !s.is_empty()).collect();
    if segments.is_empty() {
        return Err(
            "module-binding manifest: empty module path (cannot render QualifiedName)".to_string(),
        );
    }
    let rendered: Vec<String> = segments
        .iter()
        .map(|s| dag_manifest_scalar_escape(s).map(|e| format!("\"{e}\"")))
        .collect::<Result<_, _>>()?;
    Ok(format!(
        "qualified_name_from_string_segments(segments: [{}])",
        rendered.join(", ")
    ))
}

fn emit_module_binding_span_index(span: &SourceSpan, file_symbol: &str) -> String {
    let start = span.start.max(0);
    let end = span.end.max(start);
    let occurrence_id = start.max(1);
    format!(
        "span_index_record(\n  index: span_index_empty(),\n  id: MintedOccurrence {{ id: OccurrenceId {{ value: {occurrence_id} }} }},\n  event: FromSource {{ locus: Textual {{ file: {file_symbol}, extent: ByteRange {{ start: {start}, end: {end} }} }} }}\n)"
    )
}

fn emit_module_binding_row(row: &ModuleBindingManifestRow) -> Result<String, String> {
    let qn = emit_module_binding_qualified_name(&row.module_path)?;
    let artifact_id = source_root_ingest_artifact_id_for_path(&row.rel_path);
    let span_index = emit_module_binding_span_index(&row.ident_span, &artifact_id);
    Ok(format!(
        "module_storage_parsed_binding(\n  module: {qn},\n  artifact: Artifact {{\n    kind: SourceFile,\n    id: {artifact_id},\n    file_path: \"{}\"\n  }},\n  span_index: {span_index},\n  source_root: {}\n)",
        dag_manifest_scalar_escape(&row.rel_path)?,
        row.root_variant
    ))
}

fn emit_module_binding_monoid(rows: &[ModuleBindingManifestRow]) -> Result<String, String> {
    let mut nodes: Vec<String> = rows
        .iter()
        .map(emit_module_binding_row)
        .collect::<Result<_, _>>()?;
    let mut out = String::from("Empty");
    while let Some(head) = nodes.pop() {
        out = format!("Cons {{\n  head: {head},\n  tail: {out}\n}}");
    }
    Ok(out)
}

pub fn emit_source_root_ingest_manifest(
    path: &Path,
    records: &[SourceRootReadRecord],
    entry_admission: Option<&SourceRootEntryAdmission>,
) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("failed to create manifest parent {:?}: {}", parent, e))?;
    }

    let content_hash = source_root_ingest_content_hash_fnv1a64(records);
    let read_count = records.len();
    let inline_records = if read_count <= MANIFEST_INLINE_LIST_MAX {
        records
    } else {
        &[]
    };

    let mut out = String::new();
    out.push_str("module v2.test.workflow.host_source_root_ingest_manifest\n\n\n");
    out.push_str("import v2.compiler.source_authority {\n");
    out.push_str("  DagSourceReadWitness,\n");
    out.push_str("  SourceRootIngest,\n");
    out.push_str("  SourceRootCoverageComplete,\n");
    out.push_str("  SourceRootManifestElided,\n");
    out.push_str("  SourceRootProvenanceCoverageReceipt\n");
    out.push_str("}\n");
    out.push_str("import extdeps.communication.medium { Lossless, Medium }\n");
    out.push_str("import v2.std.algebra { Cons, Empty }\n");
    out.push_str("import v2.std.artifact { Artifact, SourceFile }\n");
    out.push_str("import v2.std.text { String }\n");
    // Each DagSourceReadWitness carries a grounded `source_root: SourceRootRef` (V2Tree/DagTree,
    // #5473/#5486), so the manifest must import the constructors it references or every witness
    // fails with `undefined variable 'V2Tree'` (the source_root ingest gate's persistent RED).
    // #6269's emit_source_root_ref_import derives exactly the referenced constructors from the
    // records (supersedes the earlier hardcoded-both-constructors form).
    if !inline_records.is_empty() {
        out.push_str(&emit_source_root_ref_import(inline_records));
    }
    if entry_admission.is_some() {
        out.push_str("import v2.compiler.name_resolve {\n");
        out.push_str("  Admission,\n");
        out.push_str("  Import,\n");
        out.push_str("  ImportVisible,\n");
        out.push_str("  ResolutionSubject\n");
        out.push_str("}\n");
        out.push_str("import v2.std.algebra { Cons, Empty }\n");
    }
    out.push('\n');
    out.push_str(&format!(
        "data host_source_root_ingest_content_hash: String = \"{}\"\n\n\n",
        dag_manifest_scalar_escape(&content_hash)?
    ));
    out.push_str("data host_source_root_ingest_coverage_receipt: SourceRootProvenanceCoverageReceipt = SourceRootProvenanceCoverageReceipt {\n");
    // The receipt must describe the carrier that actually landed, not the discovery that
    // preceded it. Past MANIFEST_INLINE_LIST_MAX the row list is elided to `Empty`, so
    // hardcoding `coverage_complete: true` with the full read_count asserted complete
    // coverage over an EMPTY carrier — and made it unfalsifiable by construction
    // (DESIGN.md §5: fabricated plausible output; a receipt that can never be false
    // reports nothing).
    //
    // The elision is now a TYPED, COUNTED refusal rather than a bool: `SourceRootManifestElided`
    // names the read count AND the cap that rejected it, so a consumer sees the size of the
    // deficit ("91 reads met a cap of 64") instead of an undifferentiated `false`. A silent
    // `Empty` carrier under a `true` receipt was an absorbing fallback — ⊤-as-ignorance
    // presented as ⊤-as-answer.
    let produced_row_count = inline_records.len();
    out.push_str(&format!("  ingest_read_count: {read_count},\n"));
    out.push_str(&format!("  produced_row_count: {produced_row_count},\n"));
    if produced_row_count == read_count {
        out.push_str("  coverage: SourceRootCoverageComplete\n");
    } else {
        out.push_str(&format!(
            "  coverage: SourceRootManifestElided {{ read_count: {read_count}, cap: {MANIFEST_INLINE_LIST_MAX} }}\n"
        ));
    }
    out.push_str("}\n\n\n");
    out.push_str("data host_source_root_ingest: SourceRootIngest = ");
    if inline_records.is_empty() {
        out.push_str("Empty\n");
    } else {
        out.push_str(&emit_source_root_ingest_monoid(inline_records)?);
        out.push('\n');
    }
    if let Some(admission) = entry_admission {
        out.push('\n');
        out.push_str(&emit_source_root_entry_admission_data(admission));
    }

    std::fs::write(path, out).map_err(|e| format!("failed to write manifest {:?}: {}", path, e))
}

#[cfg(test)]
mod source_root_ingest_manifest_tests {
    use super::{emit_source_root_ingest_manifest, SourceRootReadRecord, MANIFEST_INLINE_LIST_MAX};

    fn sr_record(i: usize) -> SourceRootReadRecord {
        SourceRootReadRecord {
            file_path: format!("src/v2/std/cov_fixture_{i}.dag"),
            module_path: format!("v2.std.cov_fixture_{i}"),
            source: format!("module v2.std.cov_fixture_{i}"),
            source_root: "src/v2".to_string(),
        }
    }

    /// Past the inline cap the row list is elided, so the receipt must say so.
    /// Before this fix `coverage_complete: true` was hardcoded and the full read_count
    /// emitted as produced_row_count, asserting complete coverage over an empty carrier.
    #[test]
    fn receipt_reports_incomplete_coverage_when_rows_are_elided() {
        let dir = std::env::temp_dir().join(format!(
            "gunbc_cov_receipt_{}_{}",
            std::process::id(),
            line!()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("host_source_root_ingest_manifest.dag");

        let over: Vec<SourceRootReadRecord> =
            (0..MANIFEST_INLINE_LIST_MAX + 1).map(sr_record).collect();
        emit_source_root_ingest_manifest(&path, &over, None).unwrap();
        let emitted = std::fs::read_to_string(&path).unwrap();
        assert!(
            emitted.contains("coverage: SourceRootManifestElided"),
            "elided manifest must report incomplete coverage, got:\n{emitted}"
        );
        assert!(
            emitted.contains("produced_row_count: 0"),
            "elided manifest must report the rows it actually carries (0), got:\n{emitted}"
        );
        assert!(
            emitted.contains(&format!(
                "ingest_read_count: {}",
                MANIFEST_INLINE_LIST_MAX + 1
            )),
            "discovered read count must still be reported, got:\n{emitted}"
        );

        // Control: within the cap, coverage really is complete.
        let under: Vec<SourceRootReadRecord> = (0..3).map(sr_record).collect();
        emit_source_root_ingest_manifest(&path, &under, None).unwrap();
        let emitted = std::fs::read_to_string(&path).unwrap();
        assert!(
            emitted.contains("coverage: SourceRootCoverageComplete")
                && emitted.contains("produced_row_count: 3"),
            "inline manifest must report complete coverage over 3 rows, got:\n{emitted}"
        );

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn manifest_imports_grounded_source_root_constructors() {
        // Each DagSourceReadWitness carries a grounded `source_root: SourceRootRef`
        // (V2Tree/DagTree, #5473/#5486). The manifest references those constructors, so it
        // MUST import them from v2.std.cross_tree.import_model -- otherwise every witness that
        // imports the manifest fails to resolve with `undefined variable 'V2Tree'`, which was
        // the source_root_ingest gate's persistent main-RED. RED CONTROL: delete the
        // cross_tree.import_model import line in the emitter and this test fails.
        let tmp = std::env::temp_dir().join(format!(
            "sri_manifest_import_test_{}.dag",
            std::process::id()
        ));
        let records = vec![SourceRootReadRecord {
            file_path: "src/v2/x.dag".to_string(),
            module_path: "x".to_string(),
            source: "module x\n".to_string(),
            source_root: "V2Tree".to_string(),
        }];
        emit_source_root_ingest_manifest(&tmp, &records, None).expect("emit manifest");
        let out = std::fs::read_to_string(&tmp).expect("read manifest");
        let _ = std::fs::remove_file(&tmp);
        assert!(
            out.contains("source_root: V2Tree"),
            "manifest must emit the grounded source_root value"
        );
        assert!(
            out.contains("import v2.std.cross_tree.import_model"),
            "manifest referencing V2Tree/DagTree must import them or witnesses hit \
             `undefined variable`; got:\n{out}"
        );
    }
}

#[cfg(test)]
mod inert_lens_hygiene_tests {
    use super::{
        default_source_roots, discover_floor_corpus_rows, inert_lens_modules,
        is_top_level_lens_module, witness_discovery_scan_dirs, witness_exclusion_substrings,
        DiscoveryRow,
    };
    use std::collections::HashMap;
    use std::path::PathBuf;

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(3)
            .expect("workspace root")
            .to_path_buf()
    }

    fn row(entry: &str, function: &str) -> DiscoveryRow {
        DiscoveryRow {
            label: function.to_string(),
            entry: entry.to_string(),
            function: function.to_string(),
            reads_live_tree: false,
        }
    }

    #[test]
    fn top_level_lens_module_predicate() {
        assert!(is_top_level_lens_module("v2.lens.effect"));
        assert!(is_top_level_lens_module(
            "v2.lens.extdeps_shape_transport_policy"
        ));
        assert!(!is_top_level_lens_module(
            "v2.lens.extdeps_shape_transport_policy.module_refs"
        ));
        assert!(!is_top_level_lens_module(
            "v2.test.lens_effect.effect_depends_on"
        ));
        assert!(!is_top_level_lens_module("v2.std.algebra"));
        assert!(!is_top_level_lens_module("v2.lens."));
    }

    #[test]
    fn detector_red_on_unreached_green_on_wired() {
        let mut module_to_path: HashMap<String, String> = HashMap::new();
        let mut path_imports: HashMap<String, Vec<String>> = HashMap::new();
        module_to_path.insert(
            "v2.lens.demo".to_string(),
            "src/v2/lens/demo.dag".to_string(),
        );
        path_imports.insert("src/v2/lens/demo.dag".to_string(), vec![]);

        let inert = inert_lens_modules(&[], &path_imports, &module_to_path);
        assert_eq!(inert, vec!["v2.lens.demo".to_string()]);

        module_to_path.insert(
            "v2.test.lens_demo.w".to_string(),
            "src/v2/workflow/lens_demo_family_eval_test.dag".to_string(),
        );
        path_imports.insert(
            "src/v2/workflow/lens_demo_family_eval_test.dag".to_string(),
            vec!["v2.lens.demo".to_string()],
        );
        let rows = vec![row("src/v2/workflow/lens_demo_family_eval_test.dag", "w")];
        assert!(
            inert_lens_modules(&rows, &path_imports, &module_to_path).is_empty(),
            "wiring a discovered witness must clear the inert flag"
        );

        module_to_path.insert("v2.lens.sib".to_string(), "src/v2/lens/sib.dag".to_string());
        path_imports.insert("src/v2/lens/sib.dag".to_string(), vec![]);
        path_imports.insert(
            "src/v2/lens/demo.dag".to_string(),
            vec!["v2.lens.sib".to_string()],
        );
        assert!(
            inert_lens_modules(&rows, &path_imports, &module_to_path).is_empty(),
            "a transitively-reached sibling lens must count as wired"
        );
    }

    #[test]
    fn builtin_inert_lens_counts_are_green_on_live_corpus() {
        let ws = workspace_root();
        std::env::set_current_dir(&ws).expect("chdir to workspace root");
        assert_eq!(
            super::inert_lens_unreached_module_count(),
            0,
            "every v2.lens.* must be reached by a floor witness"
        );
        assert!(
            super::inert_lens_top_level_module_count() > 0,
            "lens universe must be non-empty (non-vacuity oracle)"
        );
    }

    #[test]
    fn floor_corpus_has_no_inert_lenses() {
        let ws = workspace_root();
        std::env::set_current_dir(&ws).expect("chdir to workspace root");
        let roots = default_source_roots();
        let scan_dirs = witness_discovery_scan_dirs();
        let excludes = witness_exclusion_substrings();
        let result = discover_floor_corpus_rows(&roots, &scan_dirs, &excludes);
        assert!(
            result.is_ok(),
            "floor discovery must succeed — every v2.lens.* is wired or deleted: {}",
            result.err().unwrap_or_default()
        );
    }
}

#[cfg(test)]
mod construction_justification_hygiene_tests {
    use super::{
        construction_authority_graph_unresolved, construction_authority_unresolved,
        declares_construction_justification, discover_floor_corpus_rows, unjustified_lens_modules,
        wall_now_authority_refs, witness_exclusion_substrings,
    };
    use std::collections::BTreeSet;
    use std::collections::HashMap;
    use std::path::PathBuf;

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(3)
            .expect("workspace root")
            .to_path_buf()
    }

    #[test]
    fn justification_scan_predicate() {
        let with = "module v2.lens.demo\n\
            import v2.lens.common.construction_justification { ConstructionJustification, RatchetForever }\n\
            data construction_justification: ConstructionJustification = ConstructionJustification {\n\
              class: RatchetForever\n\
            }\n";
        assert!(declares_construction_justification(with));

        assert!(!declares_construction_justification(
            "data construction_justification_note: String = \"todo\"\n"
        ));
        assert!(!declares_construction_justification(
            "module v2.lens.demo\ndata other: String = \"z\"\n"
        ));
    }

    #[test]
    fn detector_red_on_missing_green_on_recorded() {
        let mut module_to_path: HashMap<String, String> = HashMap::new();
        module_to_path.insert(
            "v2.lens.demo".to_string(),
            "src/v2/lens/demo.dag".to_string(),
        );
        module_to_path.insert(
            "v2.lens.common.construction_justification".to_string(),
            "src/v2/lens/common/construction_justification.dag".to_string(),
        );
        module_to_path.insert("v2.std.text".to_string(), "src/v2/std/text.dag".to_string());

        let none: BTreeSet<String> = BTreeSet::new();
        assert_eq!(
            unjustified_lens_modules(&module_to_path, &none),
            vec!["v2.lens.demo".to_string()],
            "an unjustified top-level lens must go RED"
        );

        let mut justified: BTreeSet<String> = BTreeSet::new();
        justified.insert("v2.lens.demo".to_string());
        assert!(
            unjustified_lens_modules(&module_to_path, &justified).is_empty(),
            "recording a justification must clear the violation"
        );
    }

    #[test]
    fn floor_corpus_every_lens_is_justified() {
        let ws = workspace_root();
        std::env::set_current_dir(&ws).expect("chdir to workspace root");
        let roots = vec![
            ws.join("dag").to_string_lossy().into_owned(),
            ws.join("src/v2").to_string_lossy().into_owned(),
        ];
        let scan_dirs = vec![
            "dag/test/claim".to_string(),
            "src/v2/test/claim/manual".to_string(),
        ];
        let excludes = witness_exclusion_substrings();
        let result = discover_floor_corpus_rows(&roots, &scan_dirs, &excludes);
        assert!(
            result.is_ok(),
            "floor discovery must succeed — every v2.lens.* records a construction-justification: {}",
            result.err().unwrap_or_default()
        );
    }

    // ITEM 2 graph-property witness: the construction->authority graph is TOTAL over the
    // live corpus (every WallNow authority DeclarationRef resolves to a real top-level decl).
    // Perturb-to-RED: plant a dangling decl_name in any WallNow site -> this flips to a
    // non-empty unresolved list and the test fails. (SCAFFOLD, dissolves on item (ii) —
    // unified kind-agnostic decl-resolution exposed to .dag; see construction_authority_* docs.)
    #[test]
    fn wall_now_authority_graph_is_total() {
        let ws = workspace_root();
        std::env::set_current_dir(&ws).expect("chdir to workspace root");
        let roots = vec![
            ws.join("dag").to_string_lossy().into_owned(),
            ws.join("src/v2").to_string_lossy().into_owned(),
        ];
        let unresolved =
            construction_authority_graph_unresolved(&roots).expect("corpus walk must succeed");
        assert!(
            unresolved.is_empty(),
            "every WallNow construction-authority must resolve to a real decl; dangling: {unresolved:?}"
        );
    }

    // Discriminating control: the resolver detects a dangling authority and clears on a real one.
    #[test]
    fn dangling_authority_is_detected() {
        let mut module_to_content: HashMap<String, String> = HashMap::new();
        module_to_content.insert(
            "v2.std.node".to_string(),
            "module v2.std.node\ntype NodeKind\n  = TypeNode { connective: Connective }\n"
                .to_string(),
        );

        let real = vec![(
            "src/v2/lens/cost.dag".to_string(),
            "v2.std.node".to_string(),
            "NodeKind".to_string(),
        )];
        assert!(
            construction_authority_unresolved(&module_to_content, &real).is_empty(),
            "a real authority (v2.std.node.NodeKind) must resolve"
        );

        let dangling = vec![(
            "src/v2/lens/cost.dag".to_string(),
            "v2.std.node".to_string(),
            "NoSuchDecl".to_string(),
        )];
        assert_eq!(
            construction_authority_unresolved(&module_to_content, &dangling).len(),
            1,
            "a dangling decl_name must be flagged unresolved"
        );

        let missing_module = vec![(
            "src/v2/lens/cost.dag".to_string(),
            "v2.absent.module".to_string(),
            "NodeKind".to_string(),
        )];
        assert_eq!(
            construction_authority_unresolved(&module_to_content, &missing_module).len(),
            1,
            "an authority whose module is absent must be flagged unresolved"
        );
    }

    // Parse unit: extraction pulls (module_path, decl_name) from a WallNow authority,
    // whitespace/newline tolerant, and ignores non-WallNow DeclarationRef binds.
    #[test]
    fn wall_now_authority_refs_extraction() {
        let src = "  class: WallNow {\n    mechanism: SubstrateMandatoryTag,\n    authority: DeclarationRef { module_path: \"v2.std.node\", decl_name: \"NodeKind\", field: WholeDeclaration }\n  }\n";
        assert_eq!(
            wall_now_authority_refs(src),
            vec![("v2.std.node".to_string(), "NodeKind".to_string())]
        );
        // a Scaffold `bind: DeclarationRef { .. }` has no `authority:` field -> not captured.
        let other = "  bind: DeclarationRef { module_path: \"x.y\", decl_name: \"Z\", field: WholeDeclaration }\n";
        assert!(wall_now_authority_refs(other).is_empty());
    }
}

#[cfg(test)]
mod sidecar_placement_hygiene_tests {
    use super::{discover_floor_corpus_rows, scan_wire_contract_decl_names};
    use std::sync::atomic::{AtomicU64, Ordering};

    static SEQ: AtomicU64 = AtomicU64::new(0);
    fn tmp_dir() -> std::path::PathBuf {
        let id = SEQ.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "sidecar_placement_test_{}_{}",
            std::process::id(),
            id
        ))
    }

    #[test]
    fn scan_detects_coproduct_wire_contract_data() {
        let content =
            "data foo: CoproductWireContract = { coproduct: \"X\", encoding: UntaggedVariant }";
        assert_eq!(
            scan_wire_contract_decl_names(content),
            vec!["foo".to_string()]
        );
    }

    #[test]
    fn scan_detects_variant_encoding_data() {
        let content = "data bar: VariantEncoding = llm_snake_wire_contract";
        assert_eq!(
            scan_wire_contract_decl_names(content),
            vec!["bar".to_string()]
        );
    }

    #[test]
    fn scan_ignores_non_wire_contract_data() {
        let content = "data baz: Int = 42\ndata qux: String = \"hello\"\ndata flag: Bool = true";
        assert!(
            scan_wire_contract_decl_names(content).is_empty(),
            "should not fire on non-wire-contract data decls"
        );
    }

    #[test]
    fn misplaced_wire_contract_decl_drives_discover_to_err() {
        let dir = tmp_dir();
        std::fs::create_dir_all(&dir).expect("create temp dir");
        let file = dir.join("anthropic.dag");
        std::fs::write(
            &file,
            "data anthropic_chat_message_wire_contract: CoproductWireContract = { \
             coproduct: \"AnthropicChatMessage\", encoding: UntaggedVariant }\n",
        )
        .expect("write temp file");
        let root = dir.to_string_lossy().into_owned();
        let result = discover_floor_corpus_rows(&[root], &[], &[]);
        let _ = std::fs::remove_dir_all(&dir);
        let msg = result
            .err()
            .expect("misplaced wire-contract decl must drive discover_floor_corpus_rows to Err");
        assert!(
            msg.contains("wire-contract decls") && msg.contains("_contracts.dag"),
            "error must name the decl type and required suffix: {msg}"
        );
    }
}

#[cfg(test)]
mod moduleless_entry_skip_tests {
    use super::{extract_module_path, moduleless_dag_entry_paths};

    #[test]
    fn moduleless_dag_entry_paths_collects_fixture_like_fragments() {
        let entries = vec![
            (
                "/repo/src/v1/stage0/tests/fixtures/split.dag".to_string(),
                "data x: Int = 0\n".to_string(),
            ),
            (
                "/repo/src/v1/compile.dag".to_string(),
                "module v1.compile\n".to_string(),
            ),
        ];
        assert_eq!(
            moduleless_dag_entry_paths(&entries),
            vec!["/repo/src/v1/stage0/tests/fixtures/split.dag".to_string()]
        );
    }

    #[test]
    fn moduleless_dag_entry_paths_surfaces_real_source_without_module() {
        let entries = vec![(
            "/repo/src/v1/forgot_module.dag".to_string(),
            "data oops: Int = 0\n".to_string(),
        )];
        assert_eq!(
            moduleless_dag_entry_paths(&entries),
            vec!["/repo/src/v1/forgot_module.dag".to_string()]
        );
        assert!(extract_module_path(&entries[0].1).is_none());
    }
}

#[cfg(test)]
mod witness_timing_attribution_tests {
    use super::{
        compute_witness_timing_rows, merge_discovery_summaries, top_n_slowest_witnesses,
        ClaimOutcome, DiscoverySummary, DiscoveryWitnessOutcome, EntryResolveReceipt,
        ResolveStageNanos,
    };
    use crate::v1_interpreter::PerformanceReceipt;

    fn sample_summary() -> DiscoverySummary {
        DiscoverySummary {
            total: 3,
            passed: 3,
            skipped: 0,
            deferred_rows: Vec::new(),
            predicted_unaffected: Vec::new(),
            divergences: Vec::new(),
            failures: Vec::new(),
            witness_outcomes: vec![
                DiscoveryWitnessOutcome {
                    entry: "a.dag".to_string(),
                    function: "fast".to_string(),
                    outcome: ClaimOutcome::Pass,
                },
                DiscoveryWitnessOutcome {
                    entry: "b.dag".to_string(),
                    function: "slow".to_string(),
                    outcome: ClaimOutcome::Pass,
                },
                DiscoveryWitnessOutcome {
                    entry: "a.dag".to_string(),
                    function: "medium".to_string(),
                    outcome: ClaimOutcome::Pass,
                },
            ],
            entry_resolve_receipts: vec![
                EntryResolveReceipt {
                    entry: "a.dag".to_string(),
                    closure_subject: "subj-a".to_string(),
                    resolve_nanos: 100,
                    stage_nanos: ResolveStageNanos::default(),
                },
                EntryResolveReceipt {
                    entry: "b.dag".to_string(),
                    closure_subject: "subj-b".to_string(),
                    resolve_nanos: 200,
                    stage_nanos: ResolveStageNanos::default(),
                },
            ],
            total_resolve_nanos: 300,
            total_stage_nanos: ResolveStageNanos::default(),
            performance_receipts: vec![
                PerformanceReceipt {
                    subject_key: "subj-a".to_string(),
                    work_shape: "claim".to_string(),
                    wall_nanos: 1_000,
                    eval_self_nanos: 1_000,
                    sample_count: 1,
                },
                PerformanceReceipt {
                    subject_key: "subj-b".to_string(),
                    work_shape: "claim".to_string(),
                    wall_nanos: 50_000,
                    eval_self_nanos: 50_000,
                    sample_count: 1,
                },
                PerformanceReceipt {
                    subject_key: "subj-a".to_string(),
                    work_shape: "claim".to_string(),
                    wall_nanos: 5_000,
                    eval_self_nanos: 5_000,
                    sample_count: 1,
                },
            ],
            total_measured_nanos: 56_000,
            roster_closure_nodes: 42,
        }
    }

    #[test]
    fn merge_discovery_summaries_takes_max_roster_closure() {
        // Per-shard closure is MAX-merged, not summed: parallel shards share the std/spec prefix, so
        // summing would double-count it; the heaviest single shard's closure is what the per-shard
        // memory peak is a function of. RED if a future edit sums the field (would be 101), drops the
        // merge line (would stay 0), or reverts the carrier.
        let mut a = sample_summary();
        a.roster_closure_nodes = 30;
        let mut b = sample_summary();
        b.roster_closure_nodes = 71;
        let merged = merge_discovery_summaries(vec![a, b]);
        assert_eq!(merged.roster_closure_nodes, 71);
    }

    #[test]
    fn witness_timing_rows_pair_perf_with_outcomes() {
        let rows = compute_witness_timing_rows(&sample_summary()).expect("rows");
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].function, "fast");
        assert_eq!(rows[0].eval_nanos, 1_000);
        assert_eq!(rows[0].resolve_nanos, 100);
        assert_eq!(rows[0].total_nanos, 1_100);
    }

    #[test]
    fn top_n_slowest_ranks_by_eval_descending() {
        let rows = compute_witness_timing_rows(&sample_summary()).expect("rows");
        let top = top_n_slowest_witnesses(&rows, 2);
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].function, "slow");
        assert_eq!(top[1].function, "medium");
    }
}

pub struct LayerImportFactRaw {
    pub layer: &'static str,
    pub path: String,
    pub import_module: String,
}

const LAYER_STD: &str = "LayerPrefixStd";
const LAYER_EXTDEPS: &str = "LayerPrefixExtdeps";

fn rel_path_for_layer_import(path: &Path) -> String {
    if path.is_relative() {
        return path.to_string_lossy().replace('\\', "/");
    }
    repo_relative_path_normalized(path)
}

fn pool_roots_abs(pool_roots: &[String]) -> Vec<String> {
    pool_roots.iter().map(|r| anchor_source_root(r)).collect()
}

fn project_layer_import_root(root: &str, layer: &'static str, out: &mut Vec<LayerImportFactRaw>) {
    let Some(abs_root) = try_anchor_source_root(root) else {
        eprintln!(
            "[layer-import] declared root {root} absent on disk (layer={layer}) — skipped, no facts projected"
        );
        return;
    };
    let root_path = Path::new(&abs_root);
    if !root_path.is_dir() {
        return;
    }
    let mut dag_files: Vec<PathBuf> = Vec::new();
    collect_dag_files_tolerant(root_path, &mut dag_files);
    dag_files.sort();
    for file in dag_files {
        let content = match std::fs::read_to_string(&file) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let rel = rel_path_for_layer_import(&file);
        for import_module in extract_import_paths(&content) {
            out.push(LayerImportFactRaw {
                layer,
                path: rel.clone(),
                import_module,
            });
        }
    }
}

pub fn layer_import_facts(
    std_roots: &[String],
    extdeps_roots: &[String],
) -> Vec<LayerImportFactRaw> {
    let mut out = Vec::new();
    for root in std_roots {
        project_layer_import_root(root, LAYER_STD, &mut out);
    }
    for root in extdeps_roots {
        project_layer_import_root(root, LAYER_EXTDEPS, &mut out);
    }
    out
}

// Host-fed fact extraction for `v2.lens.fact_cardinality` — the lens `.dag` table owns
// verdict logic; this bridge only projects top-level decl keys + content hashes from the
// witness-layer trees. DISSOLUTION: node-tree reader at gunbc#5364; until then one shared
// host seam (Chunk D).
const FACT_CARDINALITY_ITEM_KEYWORDS: [&str; 8] = [
    "data ",
    "fn ",
    "func ",
    "type ",
    "service ",
    "const ",
    "pattern ",
    "resource ",
];

pub struct FactCardinalityDeclFactRaw {
    pub rel_path_decl_key: String,
    pub tree: String,
    pub content_hash: String,
}

fn normalize_decl_body(source: &str) -> String {
    source
        .lines()
        .map(|line| line.split("//").next().unwrap_or(line).trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

fn decl_body_hash(body: &str) -> String {
    crate::v1_rt::atom_identity_hash(normalize_decl_body(body))
}

/// Kind-agnostic top-level decl extraction (name, content-hash) for cross-tree cardinality.
pub fn extract_top_level_decls(content: &str) -> Vec<(String, String)> {
    let lines: Vec<&str> = content.lines().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        if line.starts_with("test ") {
            i += 1;
            continue;
        }
        let Some(kw) = FACT_CARDINALITY_ITEM_KEYWORDS
            .iter()
            .find(|kw| line.starts_with(*kw))
        else {
            i += 1;
            continue;
        };
        let rest = &line[kw.len()..];
        let name: String = rest
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect();
        if name.is_empty() {
            i += 1;
            continue;
        }
        let mut body = String::new();
        body.push_str(line);
        body.push('\n');
        i += 1;
        let mut depth = brace_delta(line);
        while i < lines.len() {
            let next = lines[i];
            if depth <= 0
                && FACT_CARDINALITY_ITEM_KEYWORDS
                    .iter()
                    .any(|kw| next.starts_with(kw))
                && !next.starts_with("test ")
            {
                break;
            }
            body.push_str(next);
            body.push('\n');
            depth += brace_delta(next);
            i += 1;
        }
        out.push((name, decl_body_hash(&body)));
    }
    out
}

fn rel_path_within_tree(top_root: &Path, path: &Path) -> String {
    path.strip_prefix(top_root)
        .unwrap_or_else(|_| {
            panic!(
                "fact_cardinality_decl_facts: path {} is not under tree root {}",
                path.display(),
                top_root.display()
            )
        })
        .to_string_lossy()
        .replace('\\', "/")
}

fn walk_fact_cardinality_tree_dir(
    top_root: &Path,
    dir: &Path,
    tree: &str,
    records: &mut Vec<FactCardinalityDeclFactRaw>,
) {
    let entries = std::fs::read_dir(dir).unwrap_or_else(|e| {
        panic!(
            "fact_cardinality_decl_facts: failed to read dir {}: {e}",
            dir.display()
        )
    });
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk_fact_cardinality_tree_dir(top_root, &path, tree, records);
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("dag") {
            continue;
        }
        let rel = rel_path_within_tree(top_root, &path);
        let content = std::fs::read_to_string(&path).unwrap_or_else(|e| {
            panic!(
                "fact_cardinality_decl_facts: failed to read {}: {e}",
                path.display()
            )
        });
        for (name, hash) in extract_top_level_decls(&content) {
            records.push(FactCardinalityDeclFactRaw {
                rel_path_decl_key: format!("{rel}:{name}"),
                tree: tree.to_string(),
                content_hash: hash,
            });
        }
    }
}

fn walk_fact_cardinality_tree(
    top_root: &Path,
    tree: &str,
    records: &mut Vec<FactCardinalityDeclFactRaw>,
) {
    if !top_root.is_dir() {
        panic!(
            "fact_cardinality_decl_facts: tree root {} does not exist",
            top_root.display()
        );
    }
    walk_fact_cardinality_tree_dir(top_root, top_root, tree, records);
}

pub fn fact_cardinality_decl_facts() -> Vec<FactCardinalityDeclFactRaw> {
    let ws = workspace_root();
    let mut records = Vec::new();
    for root in witness_layer_roots() {
        let tree = Path::new(&root)
            .file_name()
            .expect("ci_layer_roots: each root must have a file_name component")
            .to_string_lossy()
            .into_owned();
        walk_fact_cardinality_tree(&ws.join(&root), &tree, &mut records);
    }
    records
}

#[derive(Clone)]
pub struct ImportResolutionFactRaw {
    pub path: String,
    pub import_module: String,
    pub target_declared: bool,
}

#[derive(Clone)]
pub struct ModuleDeclarationFactRaw {
    pub module: String,
    pub path: String,
}

fn is_excluded_import_path(rel: &str, exclude_substrings: &[String]) -> bool {
    exclude_substrings.iter().any(|s| rel.contains(s.as_str()))
}

// Per-call counters for the two host builtins the `.dag` interpreter actually invokes when a
// `.dag` fold reads `import_resolution_facts_live`/`module_declaration_facts_live` (e.g.
// `v2.lens.module_graph.import_closure_live`). Distinct from `MODULE_GRAPH_FACTS_BUILD_COUNT`
// above, which counts the separate Rust-side `build_module_graph_facts_live` batching path used
// by `current_entry_closure_files` — the two paths are not the same call site, so a cost receipt
// comparing them needs its own counter (module-grain affected-set equivalence receipt).
#[cfg(test)]
static IMPORT_RESOLUTION_FACTS_CALL_COUNT: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(0);
#[cfg(test)]
static MODULE_DECLARATION_FACTS_CALL_COUNT: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(0);

#[cfg(test)]
pub(crate) fn reset_import_resolution_facts_call_counts_for_test() {
    IMPORT_RESOLUTION_FACTS_CALL_COUNT.store(0, std::sync::atomic::Ordering::SeqCst);
    MODULE_DECLARATION_FACTS_CALL_COUNT.store(0, std::sync::atomic::Ordering::SeqCst);
}

#[cfg(test)]
pub(crate) fn import_resolution_facts_call_count_for_test() -> usize {
    IMPORT_RESOLUTION_FACTS_CALL_COUNT.load(std::sync::atomic::Ordering::SeqCst)
}

#[cfg(test)]
pub(crate) fn module_declaration_facts_call_count_for_test() -> usize {
    MODULE_DECLARATION_FACTS_CALL_COUNT.load(std::sync::atomic::Ordering::SeqCst)
}

pub fn import_resolution_facts(
    pool_roots: &[String],
    importer_roots: &[String],
    exclude_substrings: &[String],
) -> Vec<ImportResolutionFactRaw> {
    #[cfg(test)]
    IMPORT_RESOLUTION_FACTS_CALL_COUNT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let abs_pool_roots = pool_roots_abs(pool_roots);
    let abs_importer_roots = pool_roots_abs(importer_roots);
    let declared: HashSet<String> = build_module_path_index(&abs_pool_roots)
        .into_iter()
        .map(|(k, _)| k)
        .collect();
    let mut out = Vec::new();
    for root in &abs_importer_roots {
        let root_path = Path::new(root);
        if !root_path.is_dir() {
            continue;
        }
        let mut dag_files: Vec<PathBuf> = Vec::new();
        collect_dag_files_tolerant(root_path, &mut dag_files);
        dag_files.sort();
        for file in dag_files {
            let rel = rel_path_for_layer_import(&file);
            if is_excluded_import_path(&rel, exclude_substrings) {
                continue;
            }
            let content = match std::fs::read_to_string(&file) {
                Ok(c) => c,
                Err(_) => continue,
            };
            for import_module in extract_import_paths(&content) {
                let target_declared = declared.contains(&import_module);
                out.push(ImportResolutionFactRaw {
                    path: rel.clone(),
                    import_module,
                    target_declared,
                });
            }
        }
    }
    out
}

pub fn module_declaration_facts(pool_roots: &[String]) -> Vec<ModuleDeclarationFactRaw> {
    #[cfg(test)]
    MODULE_DECLARATION_FACTS_CALL_COUNT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let abs_pool_roots = pool_roots_abs(pool_roots);
    let mut out: Vec<ModuleDeclarationFactRaw> = build_module_path_index(&abs_pool_roots)
        .into_iter()
        .map(|(module, path)| ModuleDeclarationFactRaw { module, path })
        .collect();
    out.sort_by(|a, b| a.module.cmp(&b.module));
    out
}

// ── Reference-derived module edges (namespace terminal step, DESIGN "deps via container.member") ──
//
// SCAFFOLD (DESIGN §7). Replaces the import-parse module graph (`extract_import_paths` /
// `import_resolution_facts`) with edges derived from where a module's body/type REFERENCES resolve,
// so the module graph survives corpus-wide `import` deletion (namespace-only resolution, operator-
// signed 2026-07-06; the reference is the sole representation of usage, Rule 1). One O(corpus) parse
// pass over the pool (reusing the real front-end `tokenize` + `parse` — no substring scan), cached.
// Consumers (`build_module_graph_facts_live`, the inert-lens reach) union these edges onto the import
// edges during the transition; when the import grammar is deleted (parent's terminal step) the import
// term is empty and the module graph is reference-only, the `src/v2/lens/module_graph.dag`
// single-swap-point end state. Every closure/loader/lens consumer is edge-source-agnostic (reads edge
// rows as data, never import syntax).
//
// Confidence tag (parent ruling, 2026-07-14; DESIGN §5): the LOADER closure and affected-set read
// ALL edges (over-load is safe — a superset only compiles extra modules). The inert-lens reach reads
// Qualified + UniqueBare only (dropping AmbiguousBare), so an over-connected graph can never silently
// clear a truly-inert lens (no fail-open hygiene). AmbiguousBare is a bare identifier declared in >1
// module; under namespace-only that is a Rule-2 ambiguity the source should qualify.
//
// Dissolve-on: `symbol_index_fill` (SymbolIndex lane) projects exact, scope-aware reference edges from
// the filled containment tree; when that lands, this parse-and-index approximation (which is liberal
// on bare-name reference collection — a local binder that shadows a globally-unique declared name
// yields a spurious UniqueBare edge, safe for the loader, tolerated by the inert-lens grain) deletes.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RefEdgeResolution {
    Qualified,
    UniqueBare,
    AmbiguousBare,
}

impl RefEdgeResolution {
    fn rank(self) -> u8 {
        match self {
            RefEdgeResolution::Qualified => 2,
            RefEdgeResolution::UniqueBare => 1,
            RefEdgeResolution::AmbiguousBare => 0,
        }
    }
}

#[derive(Clone)]
pub struct ReferenceEdgeRaw {
    pub path: String,
    pub target_module: String,
    pub resolution: RefEdgeResolution,
}

/// Project reference edges into the `ImportResolutionFactRaw` channel the module-graph adjacency and
/// closure consumers already read (the `module_graph.dag` single-swap-point contract — downstream is
/// edge-source-agnostic). `strict` drops `AmbiguousBare` edges.
///
/// The tier is per-CONSUMER and the two are not interchangeable:
///   - `false` (keep AmbiguousBare) for the LOADER — over-connection is harmless there, since a
///     superset only compiles extra modules.
///   - `true` for SELECTION and the inert-lens reach — over-connection is not a safety problem
///     here, it is what destroys the answer. Measured: at `false` an entry's median closure is
///     1136 of 2240 modules (homonyms fan every referrer across the pool); at `true` it is 96,
///     the same order as the import-only baseline's 54.
/// Grouping these two under one tier is what made the 2026-07-14 selection repoint look
/// impossible — see `build_module_graph_facts_live_uncached`.
pub fn reference_edges_as_import_facts(
    edges: &[ReferenceEdgeRaw],
    strict: bool,
) -> Vec<ImportResolutionFactRaw> {
    edges
        .iter()
        .filter(|e| !strict || e.resolution != RefEdgeResolution::AmbiguousBare)
        .map(|e| ImportResolutionFactRaw {
            path: e.path.clone(),
            import_module: e.target_module.clone(),
            target_declared: true,
        })
        .collect()
}

/// Parse a `.dag` module's source text through the real front-end. Returns the module node, or
/// `None` on a parse error (the whole-tree compile reports such errors loudly; the module graph
/// simply omits its edges, and the corpus stays green because a syntax-broken file never resolves).
fn parse_module_node_tolerant(rel: &str, content: &str) -> Option<Rc<crate::v1_std_core::Node>> {
    let filename = rel.to_string();
    let tokens = crate::v1_compiler_tokenize::tokenize(content.to_string(), filename.clone());
    let source_index =
        crate::v1_std_core::build_newline_index(filename.clone(), content.to_string());
    let mut source_indices = HashMap::new();
    source_indices.insert(filename.clone(), source_index);
    let result = crate::v1_compiler_parse::parse(tokens, std::rc::Rc::new(source_indices));
    if result.error.is_some() {
        return None;
    }
    result.module.clone()
}

/// The names a module EXPORTS (what an `import M { X }` could once have listed): every top-level
/// item name, plus the direct child names of type declarations (variant constructors / record
/// fields). Precise by construction — fn-body locals and params are never descended into — so the
/// name→module index is not poisoned by incidental identifiers.
fn collect_module_decl_names(module: &Rc<crate::v1_std_core::Node>) -> Vec<String> {
    use crate::v1_compiler_emit_core_support::is_type_def_item;
    let mut names = Vec::new();
    for item in module.children.iter() {
        if !item.name.is_empty() {
            names.push(item.name.clone());
        }
        if is_type_def_item(item.clone()) {
            for variant in item.children.iter() {
                if !variant.name.is_empty() {
                    names.push(variant.name.clone());
                }
            }
        }
    }
    names
}

/// Reconstruct a qualified-name segment list from a `FieldAccess` chain (`A.B.c` → `[A, B, c]`).
/// `None` when the base is not a plain identifier (e.g. a call result `f(x).field` — that is a
/// value field access, not a module-qualified name).
fn ref_field_chain(node: &Rc<crate::v1_std_core::Node>) -> Option<Vec<String>> {
    use crate::v1_std_core::ExprData;
    let mut segs: Vec<String> = vec![node.name.clone()];
    let mut cur = node.children.get(0).cloned()?;
    loop {
        match &*cur.expr_data {
            ExprData::ExprFieldAccess { .. } => {
                segs.push(cur.name.clone());
                cur = cur.children.get(0).cloned()?;
            }
            ExprData::ExprVar { .. } => {
                segs.push(cur.name.clone());
                break;
            }
            _ => return None,
        }
    }
    segs.reverse();
    if segs.iter().any(|s| s.is_empty()) {
        return None;
    }
    Some(segs)
}

/// Walk a declaration subtree collecting every reference use site: bare identifiers (`node.name` on
/// any node — over-collection is a safe superset for the loader) and qualified-name chains (for
/// module-prefix matching). Recurses through every child-bearing field of `Node`.
fn collect_node_refs(
    node: &Rc<crate::v1_std_core::Node>,
    bare: &mut std::collections::HashSet<String>,
    chains: &mut Vec<Vec<String>>,
) {
    use crate::v1_std_core::{ExprData, MatchPattern};
    if let ExprData::ExprFieldAccess { .. } = &*node.expr_data {
        if let Some(chain) = ref_field_chain(node) {
            chains.push(chain);
        }
    }
    if !node.name.is_empty() {
        bare.insert(node.name.clone());
    }
    if let Some(mp) = &node.match_pattern {
        if let MatchPattern::VariantPattern {
            name,
            field_bindings,
            ..
        } = &**mp
        {
            if !name.is_empty() {
                bare.insert(name.clone());
            }
            for fb in field_bindings.iter() {
                collect_node_refs(fb, bare, chains);
            }
        }
    }
    for c in node.children.iter() {
        collect_node_refs(c, bare, chains);
    }
    for p in node.params.iter() {
        collect_node_refs(p, bare, chains);
    }
    if let Some(b) = &node.body {
        collect_node_refs(b, bare, chains);
    }
    if let Some(t) = &node.type_annotation {
        collect_node_refs(t, bare, chains);
    }
    for u in node.uses.iter() {
        collect_node_refs(u, bare, chains);
    }
    for pr in node.properties.iter() {
        collect_node_refs(pr, bare, chains);
    }
    if let Some(tr) = &node.transport {
        collect_node_refs(tr, bare, chains);
    }
}

/// Count of shared leading dot-separated segments between two module paths (containment proximity).
fn module_prefix_shared_len(a: &str, b: &str) -> usize {
    a.split('.')
        .zip(b.split('.'))
        .take_while(|(x, y)| x == y)
        .count()
}

/// Longest module-path prefix of a qualified chain that names a declared module.
fn longest_declared_module_prefix(
    chain: &[String],
    module_names: &std::collections::HashSet<String>,
) -> Option<String> {
    let mut k = chain.len();
    while k >= 1 {
        let candidate = chain[..k].join(".");
        if module_names.contains(&candidate) {
            return Some(candidate);
        }
        k -= 1;
    }
    None
}

thread_local! {
    static REFERENCE_EDGE_CACHE: RefCell<HashMap<String, Vec<ReferenceEdgeRaw>>> =
        RefCell::new(HashMap::new());
    /// Import-less files the reference producer could NOT account for. Keyed identically to
    /// `REFERENCE_EDGE_CACHE` and populated in the same pass.
    ///
    /// This set is what separates the two states an empty adjacency used to conflate
    /// (DESIGN §5, ⊤-as-answer vs ⊤-as-ignorance). An import-less file the producer PARSED and
    /// found no outgoing references in genuinely has no dependencies — its closure is `{self}`
    /// and "affected iff my own file is touched" is a precise answer, not a gap. An import-less
    /// file that failed to read/parse is a gap: the producer never got to ask. Only the second
    /// may refuse, and because it is a list rather than a boolean it is typed, located, and
    /// countable.
    static REFERENCE_UNACCOUNTED_CACHE: RefCell<HashMap<String, Vec<ReferenceAccountingRefusal>>> =
        RefCell::new(HashMap::new());
}

/// An import-less file the reference-edge producer could not answer for, with the located cause.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReferenceAccountingRefusal {
    pub path: String,
    pub cause: &'static str,
}

/// Reference-derived analogue of `import_resolution_facts`: emit one edge per (file, referenced
/// module). Same row shape channel as import facts, plus a `resolution` confidence tag. Cached by
/// (pool_roots, importer_roots, excludes).
pub fn reference_resolution_facts(
    pool_roots: &[String],
    importer_roots: &[String],
    exclude_substrings: &[String],
) -> Vec<ReferenceEdgeRaw> {
    let abs_pool_roots = pool_roots_abs(pool_roots);
    let abs_importer_roots = pool_roots_abs(importer_roots);
    let cache_key = format!(
        "{}\u{1f}{}\u{1f}{}",
        abs_pool_roots.join("\u{1e}"),
        abs_importer_roots.join("\u{1e}"),
        exclude_substrings.join("\u{1e}")
    );
    if let Some(cached) = REFERENCE_EDGE_CACHE.with(|c| c.borrow().get(&cache_key).cloned()) {
        return cached;
    }
    let mut unaccounted: Vec<ReferenceAccountingRefusal> = Vec::new();

    // ── Pass 1: parse the pool once. Build the exported-name→module index (precedence: first root
    // wins, mirroring `build_module_path_index`) and the declared-module-name set. Keep each file's
    // parsed tree so edge emission does not re-parse.
    let mut decl_index: HashMap<String, std::collections::BTreeSet<String>> = HashMap::new();
    let mut module_names: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut seen_modules: std::collections::HashSet<String> = std::collections::HashSet::new();
    // `has_imports` decides per-file whether reference edges are emitted at all: a file that still
    // carries `import` lines is covered EXACTLY by `import_resolution_facts` (no regression, no
    // over-connection). Only an import-less (stripped) file falls back to reference edges. So on the
    // un-stripped tree this producer emits nothing and the module graph is byte-identical to before.
    let mut pool_trees: HashMap<String, (String, Rc<crate::v1_std_core::Node>, bool)> =
        HashMap::new();
    for root in &abs_pool_roots {
        let root_path = Path::new(root);
        if !root_path.is_dir() {
            continue;
        }
        let mut files: Vec<PathBuf> = Vec::new();
        collect_dag_files_tolerant(root_path, &mut files);
        files.sort();
        for file in files {
            let rel = rel_path_for_layer_import(&file);
            let content = match std::fs::read_to_string(&file) {
                Ok(c) => c,
                Err(_) => continue,
            };
            let module_name = match extract_module_path(&content) {
                Some(m) => m,
                None => continue,
            };
            let tree = match parse_module_node_tolerant(&rel, &content) {
                Some(t) => t,
                None => continue,
            };
            let has_imports = !extract_import_paths(&content).is_empty();
            // Precedence: a module name already claimed by an earlier root does not re-contribute
            // exported names (first-root-wins, as `build_module_path_index`).
            if seen_modules.insert(module_name.clone()) {
                module_names.insert(module_name.clone());
                for name in collect_module_decl_names(&tree) {
                    decl_index
                        .entry(name)
                        .or_default()
                        .insert(module_name.clone());
                }
            }
            pool_trees
                .entry(rel)
                .or_insert((module_name, tree, has_imports));
        }
    }

    // ── Pass 2: for each importer file, collect its reference use sites and resolve them to modules.
    let mut edges: Vec<ReferenceEdgeRaw> = Vec::new();
    for root in &abs_importer_roots {
        let root_path = Path::new(root);
        if !root_path.is_dir() {
            continue;
        }
        let mut files: Vec<PathBuf> = Vec::new();
        collect_dag_files_tolerant(root_path, &mut files);
        files.sort();
        for file in files {
            let rel = rel_path_for_layer_import(&file);
            if is_excluded_import_path(&rel, exclude_substrings) {
                continue;
            }
            let (self_module, tree) = match pool_trees.get(&rel) {
                // A file that still carries imports is covered exactly by `import_resolution_facts`;
                // emitting reference edges for it would only over-connect. Skip — reference edges are
                // for import-less (stripped) files.
                Some((_, _, true)) => continue,
                Some((m, t, false)) => (m.clone(), t.clone()),
                // Absent from pass 1 means pass 1 skipped it: unreadable, no module line, or a
                // parse failure. Each is the producer being UNABLE TO ASK what this file depends
                // on — ignorance, not an answer — so each is recorded as a located refusal rather
                // than silently yielding an edgeless file that downstream reads as "no
                // dependencies" (DESIGN §5: a failure arm must refuse, never widen).
                None => {
                    let content = match std::fs::read_to_string(&file) {
                        Ok(c) => c,
                        Err(_) => {
                            unaccounted.push(ReferenceAccountingRefusal {
                                path: rel.clone(),
                                cause: "unreadable",
                            });
                            continue;
                        }
                    };
                    // Import-bearing: accounted EXACTLY by `import_resolution_facts`, so this is
                    // not a refusal — the other producer owns this file's edges.
                    if !extract_import_paths(&content).is_empty() {
                        continue;
                    }
                    let module_name = match extract_module_path(&content) {
                        Some(m) => m,
                        None => {
                            unaccounted.push(ReferenceAccountingRefusal {
                                path: rel.clone(),
                                cause: "no-module-line",
                            });
                            continue;
                        }
                    };
                    match parse_module_node_tolerant(&rel, &content) {
                        Some(t) => (module_name, t),
                        None => {
                            unaccounted.push(ReferenceAccountingRefusal {
                                path: rel.clone(),
                                cause: "parse-failed",
                            });
                            continue;
                        }
                    }
                }
            };
            let mut bare: std::collections::HashSet<String> = std::collections::HashSet::new();
            let mut chains: Vec<Vec<String>> = Vec::new();
            for item in tree.children.iter() {
                collect_node_refs(item, &mut bare, &mut chains);
            }
            // Resolve to per-file (target_module → strongest confidence).
            let mut file_edges: std::collections::BTreeMap<String, RefEdgeResolution> =
                std::collections::BTreeMap::new();
            let mut upgrade = |m: String, res: RefEdgeResolution| {
                let entry = file_edges.entry(m).or_insert(res);
                if res.rank() > entry.rank() {
                    *entry = res;
                }
            };
            for chain in &chains {
                if let Some(m) = longest_declared_module_prefix(chain, &module_names) {
                    if m != self_module {
                        upgrade(m, RefEdgeResolution::Qualified);
                    }
                }
            }
            for name in &bare {
                if let Some(mods) = decl_index.get(name) {
                    // Same-module declaration wins by lexical scope (namespace-only): a bare name the
                    // referencing file itself declares resolves LOCALLY — no cross-module edge. This
                    // is what keeps a ubiquitous fixture `data` (e.g. `live_tree_disposition`,
                    // declared top-level in ~670 test files) from fanning every referrer out to every
                    // declarer.
                    if mods.contains(&self_module) {
                        continue;
                    }
                    // Proximity disambiguation (namespace-only "nearest in the containment tree"):
                    // among declarers, prefer the one sharing the longest module-path prefix with the
                    // referencing module. A single nearest → UniqueBare; a tie at the nearest depth →
                    // AmbiguousBare (a genuine homonym the source must qualify — the bright-cat lane).
                    let mut best_len = 0usize;
                    let mut winners: Vec<&String> = Vec::new();
                    for m in mods.iter() {
                        let shared = module_prefix_shared_len(&self_module, m);
                        if winners.is_empty() || shared > best_len {
                            best_len = shared;
                            winners.clear();
                            winners.push(m);
                        } else if shared == best_len {
                            winners.push(m);
                        }
                    }
                    match winners.len() {
                        0 => {}
                        1 => upgrade(winners[0].clone(), RefEdgeResolution::UniqueBare),
                        _ => {
                            // Homonym-qualification worklist dump (bright-cat lane (c) seed): each
                            // AmbiguousBare is a bare ref, in a file that does not declare it, whose
                            // nearest declarers tie — the definitive "needs qualification" site.
                            if std::env::var("REFAMBIG_DUMP").is_ok() {
                                let is_witness =
                                    rel.contains("/test/") || rel.ends_with("_test.dag");
                                let cands: Vec<String> =
                                    winners.iter().map(|s| (*s).clone()).collect();
                                eprintln!(
                                    "REFAMBIG\t{}\t{}\t{}\t{}",
                                    if is_witness { "witness" } else { "compile" },
                                    rel,
                                    name,
                                    cands.join(",")
                                );
                            }
                            for t in winners {
                                upgrade(t.clone(), RefEdgeResolution::AmbiguousBare);
                            }
                        }
                    }
                }
            }
            for (m, res) in file_edges {
                edges.push(ReferenceEdgeRaw {
                    path: rel.clone(),
                    target_module: m,
                    resolution: res,
                });
            }
        }
    }

    unaccounted.sort_by(|a, b| a.path.cmp(&b.path));
    REFERENCE_UNACCOUNTED_CACHE.with(|c| c.borrow_mut().insert(cache_key.clone(), unaccounted));
    REFERENCE_EDGE_CACHE.with(|c| c.borrow_mut().insert(cache_key, edges.clone()));
    edges
}

/// Import-less files `reference_resolution_facts` could not account for, with located causes.
/// Shares the producer's cache key, so calling this after the producer is free.
pub fn reference_accounting_refusals(
    pool_roots: &[String],
    importer_roots: &[String],
    exclude_substrings: &[String],
) -> Vec<ReferenceAccountingRefusal> {
    let cache_key = format!(
        "{}\u{1f}{}\u{1f}{}",
        pool_roots_abs(pool_roots).join("\u{1e}"),
        pool_roots_abs(importer_roots).join("\u{1e}"),
        exclude_substrings.join("\u{1e}")
    );
    if let Some(cached) = REFERENCE_UNACCOUNTED_CACHE.with(|c| c.borrow().get(&cache_key).cloned())
    {
        return cached;
    }
    // Cold: run the producer, which populates both caches in one pass.
    let _ = reference_resolution_facts(pool_roots, importer_roots, exclude_substrings);
    REFERENCE_UNACCOUNTED_CACHE.with(|c| c.borrow().get(&cache_key).cloned().unwrap_or_default())
}

/// Reachability stats for bare references to one declared export name (namespace homonym triage).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BareRefReachability {
    /// Nearest-wins ties among declarers (AmbiguousBare — needs qualification).
    pub ambiguous_sites: usize,
    /// Unique nearest declarer shares zero module-path prefix with the referrer (disjoint subtree).
    pub cross_subtree_unique_sites: usize,
}

/// Count bare-reference reachability for `name` using the same nearest-wins producer as
/// `reference_resolution_facts` (import-less files only).
pub fn bare_ref_reachability_for_name(
    pool_roots: &[String],
    importer_roots: &[String],
    exclude_substrings: &[String],
    name: &str,
) -> BareRefReachability {
    let abs_pool_roots = pool_roots_abs(pool_roots);
    let abs_importer_roots = pool_roots_abs(importer_roots);
    let mut decl_index: HashMap<String, std::collections::BTreeSet<String>> = HashMap::new();
    let mut module_names: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut seen_modules: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut pool_trees: HashMap<String, (String, Rc<crate::v1_std_core::Node>, bool)> =
        HashMap::new();
    for root in &abs_pool_roots {
        let root_path = Path::new(root);
        if !root_path.is_dir() {
            continue;
        }
        let mut files: Vec<PathBuf> = Vec::new();
        collect_dag_files_tolerant(root_path, &mut files);
        files.sort();
        for file in files {
            let rel = rel_path_for_layer_import(&file);
            let content = match std::fs::read_to_string(&file) {
                Ok(c) => c,
                Err(_) => continue,
            };
            let module_name = match extract_module_path(&content) {
                Some(m) => m,
                None => continue,
            };
            let tree = match parse_module_node_tolerant(&rel, &content) {
                Some(t) => t,
                None => continue,
            };
            let has_imports = !extract_import_paths(&content).is_empty();
            if seen_modules.insert(module_name.clone()) {
                module_names.insert(module_name.clone());
                for decl_name in collect_module_decl_names(&tree) {
                    decl_index
                        .entry(decl_name)
                        .or_default()
                        .insert(module_name.clone());
                }
            }
            pool_trees
                .entry(rel)
                .or_insert((module_name, tree, has_imports));
        }
    }

    let mut stats = BareRefReachability::default();
    for root in &abs_importer_roots {
        let root_path = Path::new(root);
        if !root_path.is_dir() {
            continue;
        }
        let mut files: Vec<PathBuf> = Vec::new();
        collect_dag_files_tolerant(root_path, &mut files);
        files.sort();
        for file in files {
            let rel = rel_path_for_layer_import(&file);
            if is_excluded_import_path(&rel, exclude_substrings) {
                continue;
            }
            let (self_module, tree) = match pool_trees.get(&rel) {
                Some((_, _, true)) => continue,
                Some((m, t, false)) => (m.clone(), t.clone()),
                None => {
                    let content = match std::fs::read_to_string(&file) {
                        Ok(c) => c,
                        Err(_) => continue,
                    };
                    if !extract_import_paths(&content).is_empty() {
                        continue;
                    }
                    let module_name = match extract_module_path(&content) {
                        Some(m) => m,
                        None => continue,
                    };
                    match parse_module_node_tolerant(&rel, &content) {
                        Some(t) => (module_name, t),
                        None => continue,
                    }
                }
            };
            let mut bare: std::collections::HashSet<String> = std::collections::HashSet::new();
            let mut chains: Vec<Vec<String>> = Vec::new();
            for item in tree.children.iter() {
                collect_node_refs(item, &mut bare, &mut chains);
            }
            if !bare.contains(name) {
                continue;
            }
            let Some(mods) = decl_index.get(name) else {
                continue;
            };
            if mods.contains(&self_module) {
                continue;
            }
            let mut best_len = 0usize;
            let mut winners: Vec<&String> = Vec::new();
            for m in mods.iter() {
                let shared = module_prefix_shared_len(&self_module, m);
                if winners.is_empty() || shared > best_len {
                    best_len = shared;
                    winners.clear();
                    winners.push(m);
                } else if shared == best_len {
                    winners.push(m);
                }
            }
            match winners.len() {
                0 => {}
                1 => {
                    if module_prefix_shared_len(&self_module, winners[0]) == 0 {
                        stats.cross_subtree_unique_sites += 1;
                    }
                }
                _ => stats.ambiguous_sites += 1,
            }
        }
    }
    stats
}

// --- Resolution divergence census (namespace-resolution-design.md §12.4) ---
// Read-only inventory: compare `lookup_resolved_sig` (first-hit over func_env.parents)
// against the landed SymbolIndex containment walk (lexical + global-unique only).
// Method: direct observation of each mechanism's return value — NOT diagnostics.

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ContainmentResolveVia {
    Lexical,
    GlobalUnique,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ContainmentResolve {
    Hit {
        owner_module: String,
        qualified_path: String,
        node_ptr: usize,
        via: ContainmentResolveVia,
        lexical_steps: usize,
    },
    Ambiguous,
    Unresolved,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FnBindingRef {
    pub owner_module: String,
    pub qualified_path: String,
    pub node_ptr: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResolutionDivergenceBucket {
    Agree,
    Diverge {
        import_binding: FnBindingRef,
        containment_binding: FnBindingRef,
    },
    ContainmentAmbiguous {
        import_binding: FnBindingRef,
    },
    ContainmentUnresolved {
        import_binding: FnBindingRef,
    },
    ImportUnresolved {
        containment_binding: FnBindingRef,
    },
    NeitherBound,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolutionDivergenceSite {
    pub calling_module: String,
    pub caller_fn: String,
    pub callee: String,
    pub call_file: String,
    pub call_span_start: i64,
    pub bucket: ResolutionDivergenceBucket,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ResolutionDivergenceCostShape {
    pub containment_hits: usize,
    pub lexical_steps_histogram: BTreeMap<usize, usize>,
    pub global_unique_hits: usize,
    pub lexical_only_hits: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ResolutionDivergenceCensus {
    pub modules_resolved: usize,
    pub modules_excluded: usize,
    pub sites_checked: usize,
    pub agree: usize,
    pub diverge: usize,
    pub containment_ambiguous: usize,
    pub containment_unresolved: usize,
    pub import_unresolved: usize,
    pub neither_bound: usize,
    pub diverge_rows: Vec<ResolutionDivergenceSite>,
    pub containment_ambiguous_rows: Vec<ResolutionDivergenceSite>,
    pub containment_unresolved_rows: Vec<ResolutionDivergenceSite>,
    pub cost_shape: ResolutionDivergenceCostShape,
}

fn module_path_to_qualified_path(module_path: &str, name: &str) -> String {
    if module_path.is_empty() {
        name.to_string()
    } else {
        format!("{module_path}.{name}")
    }
}

type ModuleItemIndex = HashMap<(String, String), Rc<Node>>;

fn build_module_item_index(ctx: &v1_interpreter::InterpContext) -> ModuleItemIndex {
    let source_indices = ctx.source_indices.clone();
    let mut index = HashMap::new();
    for tm in ctx.modules.iter() {
        let module_path = tm.type_env.module_path.clone();
        for item in tm.items.iter() {
            let name = authored_name_at(source_indices.clone(), item.clone());
            index.insert((module_path.clone(), name), item.clone());
        }
    }
    index
}

/// True when `owner_module.name` is a fn/func decl — routes through `item_kind` (same
/// classifier as `local_binding_for_item` / resolver item census) when the module item is
/// available; otherwise falls back to the SymbolIndex stub shape from `local_binding_for_item`.
fn is_fn_like_binding(
    node: &Node,
    owner_module: &str,
    name: &str,
    item_index: Option<&ModuleItemIndex>,
) -> bool {
    if let Some(index) = item_index {
        if let Some(item) = index.get(&(owner_module.to_string(), name.to_string())) {
            return matches!(
                item_kind(item.clone()),
                ItemKind::FnItem | ItemKind::FuncItem
            );
        }
    }
    is_fn_decl_symbol_index_stub(node)
}

fn is_fn_decl_symbol_index_stub(node: &Node) -> bool {
    use crate::v1_std_core::Connective;
    node.connective == Connective::NoConnective
        && node.transport.is_none()
        && node.body.is_none()
        && !(node.inferred.is_some() && node.params.is_empty() && node.type_annotation.is_none())
}

fn fn_binding_from_sig(owner_module: &str, name: &str, sig: &ResolvedFuncSig) -> FnBindingRef {
    let anchor = sig
        .params
        .iter()
        .next()
        .cloned()
        .unwrap_or_else(|| sig.inferred.clone());
    FnBindingRef {
        owner_module: owner_module.to_string(),
        qualified_path: module_path_to_qualified_path(owner_module, name),
        node_ptr: Rc::as_ptr(&anchor) as usize,
    }
}

fn fn_binding_from_node(owner_module: &str, name: &str, node: &Rc<Node>) -> FnBindingRef {
    FnBindingRef {
        owner_module: owner_module.to_string(),
        qualified_path: module_path_to_qualified_path(owner_module, name),
        node_ptr: Rc::as_ptr(node) as usize,
    }
}

fn import_chain_owner(func_env: &ResolvedFuncEnv, name: &str) -> Option<String> {
    if func_env.local.contains_key(name) {
        return Some(func_env.name.clone());
    }
    for p in func_env.parents.iter() {
        if p.local.contains_key(name) {
            return Some(p.name.clone());
        }
    }
    None
}

/// `symbol_index_lexical_lookup` from `dag/std/symbol_index.dag`, on v1 string QNs.
pub fn symbol_index_lexical_lookup_v1(
    index: &SymbolIndex,
    position: &str,
    name: &str,
) -> Option<(String, Rc<Node>, usize)> {
    let mut pos = position.to_string();
    let mut steps = 0usize;
    loop {
        steps += 1;
        let qn = module_path_to_qualified_path(&pos, name);
        if let Some(node) = symbol_index_lookup(Rc::new(index.clone()), qn) {
            return Some((pos, node, steps));
        }
        if pos.is_empty() {
            return None;
        }
        pos = qualified_all_but_last(pos);
    }
}

/// Containment walk: lexical lookup, then global-bare unique (§12.4 / `03_resolve.dag`).
pub fn containment_resolve_fn_v1(
    index: &SymbolIndex,
    module_path: &str,
    name: &str,
) -> ContainmentResolve {
    containment_resolve_fn_v1_for_module(index, module_path, name, None)
}

/// Containment walk with optional module-item index for `item_kind` classification.
pub fn containment_resolve_fn_v1_for_module(
    index: &SymbolIndex,
    module_path: &str,
    name: &str,
    item_index: Option<&ModuleItemIndex>,
) -> ContainmentResolve {
    if let Some((owner, node, steps)) = symbol_index_lexical_lookup_v1(index, module_path, name) {
        if is_fn_like_binding(&node, &owner, name, item_index) {
            return ContainmentResolve::Hit {
                owner_module: owner.clone(),
                qualified_path: module_path_to_qualified_path(&owner, name),
                node_ptr: Rc::as_ptr(&node) as usize,
                via: ContainmentResolveVia::Lexical,
                lexical_steps: steps,
            };
        }
    }
    match index.global_bare.get(name).map(|s| &**s) {
        Some(GlobalBareLookupState::GlobalBareUniqueBinding {
            module_path: owner,
            binding,
        }) => {
            if is_fn_like_binding(&binding.resolved, owner, name, item_index) {
                return ContainmentResolve::Hit {
                    owner_module: owner.clone(),
                    qualified_path: module_path_to_qualified_path(&owner, name),
                    node_ptr: Rc::as_ptr(&binding.resolved) as usize,
                    via: ContainmentResolveVia::GlobalUnique,
                    lexical_steps: 0,
                };
            }
            ContainmentResolve::Unresolved
        }
        Some(GlobalBareLookupState::GlobalBareAmbiguousBinding { .. }) => {
            ContainmentResolve::Ambiguous
        }
        None => ContainmentResolve::Unresolved,
    }
}

fn collect_bare_call_sites(
    node: &Rc<Node>,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
    out: &mut Vec<(String, Rc<Node>)>,
) {
    match &*node.expr_data {
        ExprData::ExprCall { .. } => {
            let callee = expr_call_func_at(node.clone(), source_indices.clone());
            if !callee.contains('.') {
                out.push((callee, node.clone()));
            }
        }
        _ => {}
    }
    for child in node.children.iter() {
        collect_bare_call_sites(child, source_indices.clone(), out);
    }
    if let Some(body) = &node.body {
        collect_bare_call_sites(body, source_indices, out);
    }
}

fn bindings_agree(import: &FnBindingRef, containment: &FnBindingRef) -> bool {
    import.node_ptr == containment.node_ptr || import.qualified_path == containment.qualified_path
}

fn bucket_site(
    import_binding: Option<FnBindingRef>,
    containment: ContainmentResolve,
) -> ResolutionDivergenceBucket {
    match (import_binding, containment) {
        (
            Some(import),
            ContainmentResolve::Hit {
                owner_module,
                qualified_path,
                node_ptr,
                ..
            },
        ) => {
            let containment_binding = FnBindingRef {
                owner_module,
                qualified_path,
                node_ptr,
            };
            if bindings_agree(&import, &containment_binding) {
                ResolutionDivergenceBucket::Agree
            } else {
                ResolutionDivergenceBucket::Diverge {
                    import_binding: import,
                    containment_binding,
                }
            }
        }
        (Some(import), ContainmentResolve::Ambiguous) => {
            ResolutionDivergenceBucket::ContainmentAmbiguous {
                import_binding: import,
            }
        }
        (Some(import), ContainmentResolve::Unresolved) => {
            ResolutionDivergenceBucket::ContainmentUnresolved {
                import_binding: import,
            }
        }
        (
            None,
            ContainmentResolve::Hit {
                owner_module,
                qualified_path,
                node_ptr,
                ..
            },
        ) => ResolutionDivergenceBucket::ImportUnresolved {
            containment_binding: FnBindingRef {
                owner_module,
                qualified_path,
                node_ptr,
            },
        },
        (None, ContainmentResolve::Ambiguous | ContainmentResolve::Unresolved) => {
            ResolutionDivergenceBucket::NeitherBound
        }
    }
}

/// Whole-corpus resolution divergence census over a resolved `InterpContext`.
pub fn resolution_divergence_census_from_ctx(
    ctx: &v1_interpreter::InterpContext,
) -> ResolutionDivergenceCensus {
    let source_indices = ctx.source_indices.clone();
    let item_index = build_module_item_index(ctx);

    let mut out = ResolutionDivergenceCensus::default();

    for tm in ctx.modules.iter() {
        let module_path = tm.type_env.module_path.clone();
        let func_env = tm.func_env.clone();
        let module_index = (*tm.type_env.symbol_index).clone();

        for item in tm.items.iter() {
            let caller_fn = authored_name_at(source_indices.clone(), item.clone());
            let mut calls = Vec::new();
            if let Some(body) = &item.body {
                collect_bare_call_sites(body, source_indices.clone(), &mut calls);
            }
            for (callee, call_node) in calls {
                out.sites_checked += 1;
                let import_sig = lookup_resolved_sig(func_env.clone(), callee.clone());
                let import_binding = import_sig.as_ref().and_then(|sig| {
                    import_chain_owner(&func_env, &callee)
                        .map(|owner| fn_binding_from_sig(&owner, &callee, sig))
                });
                let containment = containment_resolve_fn_v1_for_module(
                    &module_index,
                    &module_path,
                    &callee,
                    Some(&item_index),
                );

                if let ContainmentResolve::Hit {
                    via, lexical_steps, ..
                } = &containment
                {
                    out.cost_shape.containment_hits += 1;
                    match via {
                        ContainmentResolveVia::Lexical => {
                            out.cost_shape.lexical_only_hits += 1;
                            *out.cost_shape
                                .lexical_steps_histogram
                                .entry(*lexical_steps)
                                .or_insert(0) += 1;
                        }
                        ContainmentResolveVia::GlobalUnique => {
                            out.cost_shape.global_unique_hits += 1;
                        }
                    }
                }

                let bucket = bucket_site(import_binding, containment);
                let site = ResolutionDivergenceSite {
                    calling_module: module_path.clone(),
                    caller_fn: caller_fn.clone(),
                    callee: callee.clone(),
                    call_file: call_node.span.file.clone(),
                    call_span_start: call_node.span.start,
                    bucket: bucket.clone(),
                };
                match bucket {
                    ResolutionDivergenceBucket::Agree => out.agree += 1,
                    ResolutionDivergenceBucket::Diverge { .. } => {
                        out.diverge += 1;
                        out.diverge_rows.push(site);
                    }
                    ResolutionDivergenceBucket::ContainmentAmbiguous { .. } => {
                        out.containment_ambiguous += 1;
                        out.containment_ambiguous_rows.push(site);
                    }
                    ResolutionDivergenceBucket::ContainmentUnresolved { .. } => {
                        out.containment_unresolved += 1;
                        out.containment_unresolved_rows.push(site);
                    }
                    ResolutionDivergenceBucket::ImportUnresolved { .. } => {
                        out.import_unresolved += 1
                    }
                    ResolutionDivergenceBucket::NeitherBound => out.neither_bound += 1,
                }
            }
        }
    }
    out
}

/// Floor corpus source roots for the §12.4 census (`gunbc.ci_layer_roots.witness_layer_roots`).
pub fn resolution_divergence_census_source_roots(ws: &Path) -> Vec<String> {
    vec![
        ws.join("dag").to_string_lossy().into_owned(),
        ws.join("src/v2").to_string_lossy().into_owned(),
    ]
}

/// Run the whole-tree resolution divergence census (read-only).
pub fn resolution_divergence_census_live(
    source_roots: &[String],
    exclude_substrings: &[String],
) -> Result<ResolutionDivergenceCensus, String> {
    let WholeTreeCtx {
        ctx,
        modules_resolved,
        modules_excluded,
        ..
    } = whole_tree_resolved_ctx(
        source_roots,
        exclude_substrings,
        v1_interpreter::ExecutionMode::Wet,
    )?;
    let mut census = resolution_divergence_census_from_ctx(&ctx);
    census.modules_resolved = modules_resolved;
    census.modules_excluded = modules_excluded;
    Ok(census)
}

pub fn format_resolution_divergence_census(census: &ResolutionDivergenceCensus) -> String {
    let mut lines = Vec::new();
    lines.push(format!(
        "[resolution-divergence-census] scope=dag+src/v2 modules_resolved={} modules_excluded={}",
        census.modules_resolved, census.modules_excluded
    ));
    lines.push(format!(
        "[resolution-divergence-census] sites_checked={}",
        census.sites_checked
    ));
    lines.push(format!(
        "[resolution-divergence-census] agree={}",
        census.agree
    ));
    lines.push(format!(
        "[resolution-divergence-census] diverge={}",
        census.diverge
    ));
    lines.push(format!(
        "[resolution-divergence-census] containment_ambiguous={}",
        census.containment_ambiguous
    ));
    lines.push(format!(
        "[resolution-divergence-census] containment_unresolved={}",
        census.containment_unresolved
    ));
    lines.push(format!(
        "[resolution-divergence-census] import_unresolved={}",
        census.import_unresolved
    ));
    lines.push(format!(
        "[resolution-divergence-census] neither_bound={}",
        census.neither_bound
    ));
    lines.push(format!(
        "[resolution-divergence-census] cost_shape hits={} lexical_only={} global_unique={} lexical_steps_histogram={:?}",
        census.cost_shape.containment_hits,
        census.cost_shape.lexical_only_hits,
        census.cost_shape.global_unique_hits,
        census.cost_shape.lexical_steps_histogram,
    ));
    for site in &census.diverge_rows {
        if let ResolutionDivergenceBucket::Diverge {
            import_binding,
            containment_binding,
        } = &site.bucket
        {
            lines.push(format!(
                "DIVERGE\tmodule={}\tcaller={}\tcallee={}\tat={}@{}\t\
                 import_chain={} ({} ptr={})\tcontainment={} ({} ptr={})",
                site.calling_module,
                site.caller_fn,
                site.callee,
                site.call_file,
                site.call_span_start,
                import_binding.owner_module,
                import_binding.qualified_path,
                import_binding.node_ptr,
                containment_binding.owner_module,
                containment_binding.qualified_path,
                containment_binding.node_ptr,
            ));
        }
    }
    for site in &census.containment_ambiguous_rows {
        if let ResolutionDivergenceBucket::ContainmentAmbiguous { import_binding } = &site.bucket {
            lines.push(format!(
                "CONTAINMENT_AMBIGUOUS\tmodule={}\tcaller={}\tcallee={}\tat={}@{}\t\
                 import_chain={} ({})",
                site.calling_module,
                site.caller_fn,
                site.callee,
                site.call_file,
                site.call_span_start,
                import_binding.owner_module,
                import_binding.qualified_path,
            ));
        }
    }
    for site in &census.containment_unresolved_rows {
        if let ResolutionDivergenceBucket::ContainmentUnresolved { import_binding } = &site.bucket {
            lines.push(format!(
                "CONTAINMENT_UNRESOLVED\tmodule={}\tcaller={}\tcallee={}\tat={}@{}\t\
                 import_chain={} ({})",
                site.calling_module,
                site.caller_fn,
                site.callee,
                site.call_file,
                site.call_span_start,
                import_binding.owner_module,
                import_binding.qualified_path,
            ));
        }
    }
    lines.join("\n")
}

#[cfg(test)]
mod resolution_divergence_census_tests {
    use super::{
        build_module_item_index, containment_resolve_fn_v1, containment_resolve_fn_v1_for_module,
        import_chain_owner, lookup_resolved_sig, resolution_divergence_census_live,
        whole_tree_resolved_ctx, ContainmentResolve, ResolutionDivergenceBucket, WholeTreeCtx,
    };
    use crate::v1_interpreter::ExecutionMode::Wet;

    fn write_fixture(root: &std::path::Path, rel: &str, content: &str) {
        let path = root.join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).expect("mkdir");
        std::fs::write(&path, content).expect("write dag");
    }

    fn positive_control_fixture_root() -> std::path::PathBuf {
        super::process_workspace_root()
            .join("target")
            .join(format!("gunbc-resdiv-posctl-{}", std::process::id()))
    }

    #[test]
    fn resolution_divergence_positive_control_planted_site() {
        let ws = super::process_workspace_root();
        let fixture = positive_control_fixture_root();
        let _ = std::fs::remove_dir_all(&fixture);
        write_fixture(
            &fixture,
            "middle.dag",
            r#"module test.posctl.middle

import std.types { Bool }

fn lex_target(a: Bool) -> Bool {
  return a
}
"#,
        );
        write_fixture(
            &fixture,
            "other.dag",
            r#"module test.posctl.other

import std.types { Bool }

fn lex_target(a: Bool, b: Bool) -> Bool {
  return a
}
"#,
        );
        write_fixture(
            &fixture,
            "leaf.dag",
            r#"module test.posctl.middle.leaf

import std.types { Bool }
import test.posctl.other { lex_target }

fn caller() -> Bool {
  return lex_target(False)
}
"#,
        );
        let roots = vec![
            fixture.to_string_lossy().into_owned(),
            ws.join("dag/std").to_string_lossy().into_owned(),
        ];
        let WholeTreeCtx { ctx, .. } =
            whole_tree_resolved_ctx(&roots, &[], Wet).expect("resolve ctx");
        let leaf = ctx
            .modules
            .iter()
            .find(|m| m.type_env.module_path == "test.posctl.middle.leaf")
            .expect("leaf module must resolve");
        let import_sig =
            lookup_resolved_sig(leaf.func_env.clone(), "lex_target".to_string()).expect("import");
        let import_owner = import_chain_owner(&leaf.func_env, "lex_target").expect("import owner");
        let import_arity = import_sig.params.len();
        let item_index = build_module_item_index(&ctx);
        let containment = containment_resolve_fn_v1_for_module(
            &leaf.type_env.symbol_index,
            "test.posctl.middle.leaf",
            "lex_target",
            Some(&item_index),
        );
        assert_eq!(
            import_arity, 2,
            "lookup_resolved_sig must bind other.lex_target (2 params), owner={import_owner}"
        );
        assert!(
            matches!(
                &containment,
                ContainmentResolve::Hit {
                    owner_module,
                    ..
                } if owner_module.contains("middle") && !owner_module.contains("leaf")
            ),
            "§12.4 lexical ancestor must bind middle.lex_target, got {containment:?}"
        );
        let census = resolution_divergence_census_live(&roots, &[]).expect("resolve");
        assert_eq!(
            census.diverge, 1,
            "positive control: expected Diverge=1, got diverge={} agree={} ambiguous={} import_unresolved={} neither_bound={} sites={} import_owner={import_owner} import_arity={import_arity} containment={containment:?}",
            census.diverge,
            census.agree,
            census.containment_ambiguous,
            census.import_unresolved,
            census.neither_bound,
            census.sites_checked,
        );
        let diverge_rows: Vec<_> = census
            .diverge_rows
            .iter()
            .filter(|s| s.callee == "lex_target" && s.calling_module.contains("middle.leaf"))
            .collect();
        assert_eq!(diverge_rows.len(), 1, "expected one leaf lex_target site");
        assert!(
            matches!(
                diverge_rows[0].bucket,
                ResolutionDivergenceBucket::Diverge { .. }
            ),
            "positive control must classify leaf lex_target as Diverge, got {:?}",
            diverge_rows[0].bucket
        );
        if let ResolutionDivergenceBucket::Diverge {
            import_binding,
            containment_binding,
        } = &diverge_rows[0].bucket
        {
            assert!(
                import_binding.owner_module.contains("other"),
                "import chain must bind other, got {}",
                import_binding.owner_module
            );
            assert!(
                containment_binding.owner_module.contains("middle")
                    && !containment_binding.owner_module.contains("leaf"),
                "containment lexical ancestor must bind middle, got {}",
                containment_binding.owner_module
            );
        }
        let _ = std::fs::remove_dir_all(&fixture);
    }
}

#[cfg(test)]
mod reference_edge_producer_tests {
    use super::reference_resolution_facts;

    fn fixture_root(tag: &str) -> std::path::PathBuf {
        // Under the workspace `target/` (gitignored): `rel_path_for_layer_import` fail-closes on
        // paths outside the workspace root, and `target/` keeps the fixture out of version control.
        super::process_workspace_root()
            .join("target")
            .join(format!("gunbc-refedge-{tag}-{}", std::process::id()))
    }

    fn write(root: &std::path::Path, rel: &str, content: &str) {
        let p = root.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).expect("mkdir");
        std::fs::write(&p, content).expect("write dag");
    }

    // Green-by-execution + discriminating RED: the producer must derive a cross-module edge from a
    // bare reference in an import-LESS file (the stripped case), resolve a same-module name LOCALLY
    // (no edge — the live_tree_disposition fan-out guard), and emit NOTHING for an import-bearing
    // file (import-covered). A reader that ignored the parsed references would fail the first assert.
    #[test]
    fn reference_edges_derived_from_bare_refs_local_and_import_aware() {
        let root = fixture_root("core");
        let _ = std::fs::remove_dir_all(&root);
        // Declares the shared name.
        write(
            &root,
            "decl.dag",
            "module test.decl\n\nfn shared_fn() -> Bool {\n  true\n}\n",
        );
        // Import-LESS file that references it → edge to test.decl.
        write(
            &root,
            "refless.dag",
            "module test.refless\n\nfn use_it() -> Bool {\n  shared_fn()\n}\n",
        );
        // Declares its OWN shared_fn and references it → resolves locally, NO edge.
        write(
            &root,
            "reflocal.dag",
            "module test.reflocal\n\nfn shared_fn() -> Bool {\n  true\n}\n\nfn use_it() -> Bool {\n  shared_fn()\n}\n",
        );
        // Import-bearing file → covered by import facts, producer emits nothing for it.
        write(
            &root,
            "imported.dag",
            "module test.imported\n\nimport test.decl { shared_fn }\n\nfn use_it() -> Bool {\n  shared_fn()\n}\n",
        );

        let roots = vec![root.to_string_lossy().into_owned()];
        let edges = reference_resolution_facts(&roots, &roots, &[]);
        let has_edge = |from_sub: &str, to_mod: &str| {
            edges
                .iter()
                .any(|e| e.path.contains(from_sub) && e.target_module == to_mod)
        };
        let emits_any = |from_sub: &str| edges.iter().any(|e| e.path.contains(from_sub));

        assert!(
            has_edge("refless.dag", "test.decl"),
            "import-less file referencing shared_fn must yield an edge to its declaring module"
        );
        assert!(
            !emits_any("reflocal.dag"),
            "a same-module declaration resolves locally — no cross-module edge"
        );
        assert!(
            !emits_any("imported.dag"),
            "an import-bearing file is import-covered — the reference producer skips it"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    fn str_list_value(items: &[String]) -> crate::v1_interpreter::Value {
        super::list_value_from_vec(
            items
                .iter()
                .map(|s| crate::v1_interpreter::Value::Str(s.clone()))
                .collect(),
        )
    }

    fn edge_from_record(
        ctx: &crate::v1_interpreter::InterpContext,
        value: &crate::v1_interpreter::Value,
    ) -> (String, String) {
        let crate::v1_interpreter::Value::Record { fields, .. } = value else {
            panic!("expected ModuleDependencyEdge record, got {value}");
        };
        let path = match ctx.field(fields, "path") {
            Some(crate::v1_interpreter::Value::Str(s)) => s.clone(),
            other => panic!("path field: {other:?}"),
        };
        let target = match ctx.field(fields, "target_module") {
            Some(crate::v1_interpreter::Value::Str(s)) => s.clone(),
            other => panic!("target_module field: {other:?}"),
        };
        (path, target)
    }

    fn dependency_edges_from_free_monoid(
        ctx: &crate::v1_interpreter::InterpContext,
        value: &crate::v1_interpreter::Value,
    ) -> Vec<(String, String)> {
        match value {
            crate::v1_interpreter::Value::Variant {
                variant_name,
                fields,
                ..
            } if ctx.sym_eq(*variant_name, "Empty") => Vec::new(),
            crate::v1_interpreter::Value::Variant {
                variant_name,
                fields,
                ..
            } if ctx.sym_eq(*variant_name, "Cons") => {
                let head = ctx
                    .field(fields, "head")
                    .expect("Cons.head must be present");
                let tail = ctx
                    .field(fields, "tail")
                    .expect("Cons.tail must be present");
                let mut edges = vec![edge_from_record(ctx, head)];
                edges.extend(dependency_edges_from_free_monoid(ctx, tail));
                edges
            }
            other => panic!("expected FreeMonoid Cons/Empty, got {other}"),
        }
    }

    // Divergence control for the §3 producer fork dissolved in #6935: an import-less file that
    // references another module by bare name is invisible to import_resolution_facts but visible to
    // reference_resolution_facts (strict tier). Before the builtin registration the .dag lens
    // under-selected; after, dependency_resolution_facts_live and the host agree.
    #[test]
    fn reference_edge_dag_host_producer_divergence_control() {
        use super::{
            build_multi_entry_index, import_resolution_facts, make_eval_context,
            reference_edges_as_import_facts, reference_resolution_facts,
            resolve_entry_with_index_for_discovery_corpus, workspace_root,
        };
        use crate::v1_interpreter::{self, ExecutionMode};

        let root = fixture_root("divergence");
        let _ = std::fs::remove_dir_all(&root);
        write(
            &root,
            "decl.dag",
            "module test.decl\n\nfn shared_fn() -> Bool {\n  true\n}\n",
        );
        write(
            &root,
            "refless.dag",
            "module test.refless\n\nfn use_it() -> Bool {\n  shared_fn()\n}\n",
        );

        let pool = vec![root.to_string_lossy().into_owned()];
        let has_import_edge = |from_sub: &str, to_mod: &str| {
            import_resolution_facts(&pool, &pool, &[])
                .iter()
                .any(|e| e.path.contains(from_sub) && e.import_module == to_mod)
        };
        let has_host_ref_edge = |from_sub: &str, to_mod: &str| {
            reference_edges_as_import_facts(&reference_resolution_facts(&pool, &pool, &[]), true)
                .iter()
                .any(|e| e.path.contains(from_sub) && e.import_module == to_mod)
        };

        // RED control: import-only producer cannot see a reference-only dependency.
        assert!(
            !has_import_edge("refless.dag", "test.decl"),
            "import_resolution_facts must miss a reference-only edge — otherwise this test is not discriminating"
        );
        // Host selection-tier producer finds it (the fork surface we are dissolving).
        assert!(
            has_host_ref_edge("refless.dag", "test.decl"),
            "host reference_resolution_facts (strict) must find the reference-only edge"
        );

        let ws = workspace_root();
        std::env::set_current_dir(&ws).expect("chdir workspace");
        let module_graph_entry = ws
            .join("src/v2/lens/module_graph.dag")
            .to_string_lossy()
            .into_owned();
        let index_roots = vec![
            ws.join("dag").to_string_lossy().into_owned(),
            ws.join("src/v2").to_string_lossy().into_owned(),
            pool[0].clone(),
        ];
        let index = build_multi_entry_index(&index_roots);
        let (graph, indices) =
            resolve_entry_with_index_for_discovery_corpus(&index, &module_graph_entry)
                .expect("module_graph.dag resolves");
        let ctx = make_eval_context(&graph, indices, ExecutionMode::Wet);
        let args = [
            (Some("pool_roots".to_string()), str_list_value(&pool)),
            (Some("importer_roots".to_string()), str_list_value(&pool)),
            (
                Some("exclude_substrings".to_string()),
                str_list_value(&[] as &[String]),
            ),
        ];
        let dag_edges = dependency_edges_from_free_monoid(
            &ctx,
            &v1_interpreter::run_in_context_with_args(
                &ctx,
                "dependency_resolution_facts_live",
                &args,
                false,
            )
            .expect("dependency_resolution_facts_live eval"),
        );
        assert!(
            dag_edges
                .iter()
                .any(|(path, target)| path.contains("refless.dag") && target == "test.decl"),
            ".dag dependency_resolution_facts_live must find the reference-only edge after builtin registration"
        );

        let _ = std::fs::remove_dir_all(&root);
    }
}

#[cfg(test)]
mod pool_heads_oracle_tests {
    use super::*;
    use std::collections::BTreeSet;

    /// Local oracle for #6956: dump reference_resolution_facts + pool qualified-fill
    /// SymbolIndex digests (run with `--nocapture`, compare branch vs main).
    #[test]
    fn pool_heads_materialization_oracle_dump() {
        let roots = vec![
            workspace_root()
                .join("src/v2")
                .to_string_lossy()
                .into_owned(),
            workspace_root().join("dag").to_string_lossy().into_owned(),
        ];
        let ref_edges = reference_resolution_facts(&roots, &roots, &[]);
        let mut ref_rows: Vec<String> = ref_edges
            .iter()
            .map(|e| format!("{}|{}|{:?}", e.path, e.target_module, e.resolution))
            .collect();
        ref_rows.sort();
        let ref_digest = v1_rt::bytes_identity_hash(ref_rows.join("\n").as_bytes());

        let index = build_multi_entry_index(&roots);
        let fill = pool_qualified_fill(&index).expect("qualified fill");
        let qkeys: BTreeSet<String> = fill.entries.keys().cloned().collect();
        let bare_keys: BTreeSet<String> = fill.global_bare.keys().cloned().collect();
        let svc_keys: BTreeSet<String> = fill.services.keys().cloned().collect();
        let sym_digest = v1_rt::bytes_identity_hash(
            format!(
                "entries={}\n{}\nbare={}\n{}\nservices={}\n{}",
                qkeys.len(),
                qkeys.into_iter().collect::<Vec<_>>().join("\n"),
                bare_keys.len(),
                bare_keys.into_iter().collect::<Vec<_>>().join("\n"),
                svc_keys.len(),
                svc_keys.into_iter().collect::<Vec<_>>().join("\n"),
            )
            .as_bytes(),
        );

        println!(
            "POOL_HEADS_ORACLE reference_edge_count={} reference_digest={} symbol_index_digest={} qualified_entries={} global_bare={} services={}",
            ref_edges.len(),
            ref_digest,
            sym_digest,
            fill.entries.len(),
            fill.global_bare.len(),
            fill.services.len(),
        );
    }
}

// ── Non-fold-residue census (DESIGN §6) ──────────────────────────────────────────────────────────
//
// Audits the corpus for `match` expressions whose scrutinee is a function parameter with a declared
// closed-coproduct type AND whose body has a top-level `_ =>` wildcard arm.
//
// Host-fed; DISSOLUTION: folds into a pure `.dag` Node-tree reader (match nodes + scrutinee type)
// when exhaustiveness-by-default / compile-graph access lands (gunbc#5364).

const NON_FOLD_RESIDUE_ROSTER: &[&str] = &[
    // 2026-07-18 backfill: four sites that landed unrostered on main while the affected-set
    // selection predict-skipped the corpus-read nfr witness for their landing diffs (same
    // masking class as the dated blocks below). Their files predate this PR and this PR does
    // not touch them; they surfaced here only because this PR edits the roster (a .rs change),
    // which re-runs the whole-corpus nfr_roster_receipt under the nextest gate — the same role
    // the nightly cold sweep plays. Declared so the ratchet re-arms. local_tidy landed with the
    // shell->dag bash de-fork (#6751), a one-special-variant dispatch (LocalTidyCargoFmtCheck
    // special, else the glob fallback) — burns down with local_tidy_spec's fold migration. The
    // three *_eq rows are structural equality (param scrutinee, off-variant `_ => false`),
    // siblings to the std eq rows below; dissolve with derived equality from inhabitance
    // (dag/std/algebra, DESIGN §3/§4).
    "dag/gunbc/local_tidy_spec.dag::local_tidy_path_matches_trigger",
    "src/v2/lens/enforcement/lens_module_gate.dag::lens_module_gate_remedy_eq",
    "src/v2/lens/enforcement/lens_module_gate.dag::lens_module_gate_verdict_authority_eq",
    "src/v2/lens/vacuity.dag::vacuity_evidence_eq",
    // 2026-07-19 backfill: #6857 (effect-reach census lens) landed this site unrostered on
    // main — same masking class as the 2026-07-18 block above (affected-set selection
    // predict-skipped the corpus-read nfr witness for its landing diff; surfaced here when
    // the namespace branch's .rs edits re-ran the whole-corpus receipt). Structural equality
    // (param scrutinee, off-variant `_ => false`), sibling to the *_eq rows above; dissolves
    // with derived equality from inhabitance (dag/std/algebra, DESIGN §3/§4).
    "src/v2/lens/effect_reach.dag::sink_kind_eq",
    // 2026-07-13 backfill: #6533 (Wave 2 frontier probe) landed this site unrostered — the nfr
    // witnesses are corpus-read host-fed rows the affected-set selection did not run for that
    // diff, so the red surfaced on the next whole-corpus cold sweep, not on the landing PR
    // (third instance of the masking class receipted on #6530). Declared here so the ratchet
    // re-arms; burns down with the frontier probe's fold migration. (#6533's other wildcard
    // fn, compiler_frontier_probe_entry_test.dag, is outside the scan universe — no row.)
    "src/v2/compiler/self_host/frontier_probe_types.dag::frontier_blocker_class_matches",
    // 2026-07-14 backfill: the shell->dag P2b/P4 slices landed orch_emit_let_step with a
    // one-special-variant dispatch (ExprCmdSubst -> cmdsubst_assign; every other Expr through
    // the uniform spelling path) — enumerating all Expr variants would clone the general arm.
    // The nfr witness is a corpus-read host-fed row, so the landing PR predict-skipped it and
    // the red surfaced on the 2026-07-14 nightly cold sweep (masking receipt #9; live-read
    // classification P1/P2, in flight, is the structural fix). Burns down with the
    // orchestration-emit fold migration.
    "src/v2/compiler/05_emit_orchestration.dag::orch_emit_let_step",
    // 2026-07-14 (no row): fleet_converge_cli.dag::converge_cli_applied_knob_count went
    // wildcard the same day (#6598 enumerated HostEffect's then-3 variants; #6586 grew it to
    // 15 on an independent base — green alone, red together; main went compile-red). The fn
    // is one-special-variant dispatch (ConvergePlan knob count; else fallback). The nfr lens
    // scans param-scrutinee matches only, so a field-scrutinee (`intent.effect`) site is
    // lens-invisible and a row here would be STALE — recorded as a lens-precision note, not
    // a roster entry.
    // 2026-07-14: #6582 (live-read P1) landed two nested structural-equality fns (param
    // scrutinee, off-variant arm returns false) — same green-alone/red-together class as the
    // rest of this batch (nfr is corpus-read; the landing PR predict-skipped it). Siblings to
    // the std eq rows; dissolve with derived equality from inhabitance (dag/std/algebra).
    "src/v2/lens/live_read_classification.dag::live_read_carrier_eq",
    "src/v2/lens/live_read_classification.dag::path_pattern_eq",
    // 2026-07-12 backfill: sites that landed unrostered while the gate was red during the
    // land-red-with-local-proof era (revoked 2026-07-12). Declared here so the ratchet
    // re-arms; each burns down with its owning file's fold migration.
    "dag/extdeps/git/git.dag::git_diff_name_status_pending_after_status",
    "dag/gunbc/srv3_os_install_reconcile.dag::kvm_screen_from_diagnostic",
    "dag/gunbc/srv3_os_install_reconcile.dag::optional_kvm_attestation_from_observation",
    "dag/gunbc/srv3_os_install_reconcile.dag::reconcile_pending_step_id",
    "dag/gunbc/srv3_os_install_reconcile.dag::workflow_approval_from_durable_grant",
    "dag/gunbc/srv3_os_install_reconcile_apply.dag::process_exit_succeeded",
    "dag/gunbc/srv3_os_install_reconcile_receipt.dag::durable_grant_is_active",
    "dag/gunbc/srv3_os_install_reconcile_receipt.dag::reconcile_refusal_reason_wire",
    "src/v2/lens/enforcement/cost_coverage.dag::cost_coverage_fn_verdict_is_body_not_located",
    "src/v2/lens/enforcement/cost_coverage.dag::cost_coverage_fn_verdict_is_known",
    "src/v2/lens/enforcement/cost_coverage.dag::cost_coverage_fn_verdict_is_parse_tree_opaque",
    "src/v2/lens/enforcement/cost_coverage.dag::cost_coverage_fn_verdict_is_unknown",
    "src/v2/lens/enforcement/receipts.dag::consumer_receipt_ref_for",
    "dag/extdeps/languages/markdown.dag::md_nested",
    "dag/gunbc/generated_artifact.dag::artifact_eq",
    "dag/gunbc/commit_workflow.dag::commit_workflow_surface_eq",
    "dag/gunbc/commit_workflow.dag::gate_eq",
    "dag/gunbc/commit_workflow.dag::local_tidy_check_eq",
    "dag/gunbc/os_install_deduction.dag::runtime_verdict_from_kvm_attestation",
    "dag/gunbc/runner_unit_live_read.dag::converge_target_live_verdict",
    "dag/gunbc/srv3_bmc_credential_resolve.dag::bmc_credential_resolution_uses_factory",
    "dag/gunbc/srv3_bmc_credential_resolve.dag::bmc_credential_resolution_uses_secret_ref",
    "dag/gunbc/srv3_os_install_diagnostic.dag::diagnose_boot_override_consumed_or_weak",
    "dag/gunbc/srv3_os_install_diagnostic.dag::diagnose_srv3_install_post_boot",
    "dag/gunbc/srv3_os_install_diagnostic.dag::diagnose_srv3_install_when_serve_observed",
    "dag/gunbc/srv3_os_install_diagnostic.dag::diagnose_srv3_install_when_serve_ready",
    "dag/gunbc/srv3_os_install_diagnostic.dag::diagnose_weak_kvm_or_inconclusive",
    "dag/gunbc/srv3_os_install_diagnostic.dag::diagnose_when_router_not_installed",
    "dag/gunbc/srv3_os_install_diagnostic.dag::diagnose_when_serve_ready",
    "dag/gunbc/srv3_os_install_diagnostic.dag::install_diagnostic_verdict_is_boot_override_consumed_failure",
    "dag/gunbc/srv3_os_install_diagnostic.dag::install_diagnostic_verdict_is_os_installed",
    "dag/gunbc/srv3_os_install_diagnostic.dag::install_diagnostic_verdict_is_ready_to_boot",
    "dag/gunbc/srv3_os_install_diagnostic.dag::install_has_progress_evidence",
    "dag/gunbc/srv3_os_install_diagnostic.dag::parse_virtual_media_session_observation",
    "dag/gunbc/srv3_os_install_diagnostic.dag::router_lacks_os_installed_lease",
    "dag/gunbc/srv3_os_install_diagnostic.dag::sol_has_autoinstall_evidence",
    "dag/gunbc/srv3_os_install_diagnostic.dag::srv3_install_diagnostic_is_boot_override_consumed",
    "dag/gunbc/srv3_os_install_diagnostic.dag::srv3_install_diagnostic_is_os_installed",
    "dag/gunbc/srv3_os_install_diagnostic.dag::srv3_install_diagnostic_is_ready_to_boot",
    "dag/std/change.dag::keyed_diff_hunks_equal",
    "dag/std/computation.dag::constant_bound_value",
    "dag/std/computation.dag::is_constant_bound",
    // NamespaceTree structural `==` residue (nested `_ => false` off-variant arm),
    // same class as the std `*_eq` rows below; landed unrostered with effect_grant.dag
    // in #6817 (P-A), surfaced when this PR brought effect_grant.dag into the affected
    // set. Dissolves with derived equality from inhabitance (DESIGN §3/§4), with its siblings.
    "dag/std/effect_grant.dag::namespace_tree_eq",
    // 2026-07-20 STALE-ROW REMOVAL: `create_double_init_collapsible` and
    // `create_effect_is_dedupable` were rostered here AND declared kernel-permanent in
    // `cla_is_kernel_permanent_fn`, so the analyzer never reports them as live sites and the
    // two rows could only ever count as stale. DESIGN §6 makes them mutually exclusive — a
    // non-fold residue is "either a named irreducible kernel or un-migrated modeling, there is
    // no third" — so a declared kernel must not also carry a burn-down row. The kernel
    // declaration is the authority; these rows were the duplicate.
    //
    // Attribution (checked, not assumed): main was already red on
    // `non_fold_residue_clean_holds` / `non_fold_residue_no_unrostered_or_stale` before #6848
    // merged — the scheduled cold falsifier failed on exactly these two witnesses at
    // 2026-07-20T14:12Z on main at 73eea76dd7 (#6914). Main's per-PR `ci` stayed green because
    // affected-set selection did not run the corpus-read nfr witnesses for those diffs; #6848
    // surfaced it by running them, which is the masking class the dated blocks above describe.
    "dag/std/effects.dag::key_source_eq",
    "dag/std/encoding.dag::encoding_lattice_join",
    "dag/std/encoding.dag::encoding_lattice_meet",
    "dag/std/filesystem.dag::is_text_encoding",
    "dag/std/induction.dag::compose_sub_value",
    "dag/std/induction.dag::compose_sub_value_relations",
    "dag/std/induction.dag::is_strict_style_structural",
    "dag/std/induction.dag::recursion_shape_eq",
    "dag/std/induction.dag::shrink_factor_eq",
    "dag/std/induction.dag::sub_value_structural_eq",
    "dag/std/reducible.dag::reduce_verdict_combine",
    "dag/std/termination.dag::descent_evidence_lattice_join",
    "dag/std/termination.dag::descent_evidence_lattice_meet",
    "dag/std/termination.dag::promote_to_strict",
    "dag/tools/ci_gates.dag::exit_ok",
    "dag/tools/generated_artifact_gate.dag::exit_ok",
    "src/v2/compiler/01_tokenize.dag::lex_try_rules_prefer_longer",
    "src/v2/compiler/05_eval.dag::eval_branch_node_eval",
    "src/v2/compiler/05_eval.dag::eval_loop_node",
    "src/v2/compiler/05_eval.dag::eval_match_node_eval",
    "src/v2/compiler/05_eval.dag::eval_transform_node",
    "src/v2/compiler/05_eval.dag::eval_value_node",
    "src/v2/compiler/05_eval.dag::run_test_claim_assert_decided",
    "src/v2/compiler/05_eval.dag::run_test_claim_runtime_assert",
    "src/v2/compiler/06_translate.dag::translate_algebra_finalize",
    "src/v2/compiler/emit_host.dag::runtime_value_signed_i32_le_as_int",
    "src/v2/compiler/self_host/frontier_probe_types.dag::frontier_blocker_class_matches",
    "src/v2/test/claim/manual/eval_runtime.dag::eval_arg_is_two_literal",
    "src/v2/extdeps/formats/spice_passive_projection.dag::passive_spec_from_component",
    "src/v2/extdeps/formats/spice_passive_projection.dag::passive_topology_from_component",
    "src/v2/extdeps/runtimes/v2_evaluator.dag::v2_eval_runtime_bool_is_false",
    "src/v2/extdeps/runtimes/v2_evaluator.dag::v2_eval_runtime_bool_is_true",
    "src/v2/extdeps/runtimes/v2_evaluator.dag::v2_eval_runtime_value_as_int",
    "src/v2/extdeps/runtimes/v2_effect_io_pure.dag::effect_io_pure_backends_match",
    "src/v2/lens/testgen.dag::algebra_law_subject_for_manual_anchor",
    "src/v2/lens/testgen.dag::nat_manual_anchor_key_eq",
    "src/v2/lens/testgen.dag::testgen_emit_language_behavior_equivalence_claim",
    "src/v2/lens/testgen.dag::testgen_emit_refinement_preservation_claim",
    "src/v2/std/node.dag::connective_edge_discipline_for_children",
    "src/v2/test/claim/generated/coproduct_exhaustiveness.dag::anchor_is",
    "src/v2/test/claim/generated/cross_representation_equality.dag::anchor_is_straddle",
    "src/v2/lens/complexity.dag::complexity_bound_dominates",
    "src/v2/lens/complexity.dag::complexity_bound_from_class",
    "src/v2/lens/cost.dag::asymptotic_class_dominates",
    "src/v2/lens/cost.dag::multiply_classes",
    "src/v2/lens/cost.dag::symbolic_cost_dominates",
    "src/v2/lens/cost.dag::symbolic_cost_witness",
    "src/v2/lens/cost.dag::symbolic_max",
    "src/v2/lens/cost.dag::symbolic_product",
    "src/v2/lens/cost.dag::symbolic_sequential",
    "src/v2/lens/fact_density.dag::connective_is_kernel_ambient_atom",
    "src/v2/lens/idempotency.dag::idempotency_verdict_eq",
    "src/v2/lens/ownership.dag::ownership_mode_eq",
    "src/v2/lens/parallelism.dag::parallelism_relation_eq",
    "src/v2/lens/registry.dag::lens_id_v0_eq",
    "src/v2/lens/unused_parameters.dag::use_relation_eq",
    "src/v2/program.dag::program_runtime_bool_false",
    "src/v2/program.dag::program_runtime_bool_true",
    "src/v2/std/compilers/target_model.dag::source_atom_value_as_bool",
    "src/v2/std/compilers/target_model.dag::source_atom_value_as_char",
    "src/v2/std/compilers/target_model.dag::source_atom_value_as_string",
    "src/v2/std/compilers/target_model.dag::source_atom_value_as_symbol",
    "src/v2/std/compilers/target_model.dag::target_type_expr_emitted_validate_wire_shape",
    "src/v2/std/compilers/target_model.dag::target_use_site_ownership_catalog_lookup_step",
    "src/v2/std/decl_index.dag::decl_facts_is_fn_like",
    "src/v2/std/float.dag::float_body_is_nan",
    "src/v2/std/node_minimal.dag::node_superset_field_eq",
    "src/v2/std/probe_selector.dag::diagnostic_interface_kind_eq",
    "src/v2/std/qualified_name.dag::qn_fold_step",
    // 2026-07-14 backfill (fourth instance of the masking class, siblings to the #6533/#6530
    // receipts above): the nightly affected-set falsifier's whole-corpus cold sweep surfaced 3
    // unrostered sites the per-PR affected-set selection did not run the nfr witness for at
    // landing time. `orch_emit_let_step` landed with #6573 (Shell→dag P2b), the two live-read
    // eq fns with #6582 (live-read classification P1) — the same PR that landed the orphan doc
    // this sweep also caught. Declared here so the ratchet re-arms; each burns down with its
    // owning file's fold migration.
    //   - orch_emit_let_step: special-case `ExprCmdSubst` + general `Expr` dispatch via
    //     orch_emit_expr_spelling; dissolves when emit is the backward grammar-row fold (§4).
    //   - live_read_carrier_eq / path_pattern_eq: nested structural `==` (`_ => false` on the
    //     off-variant arm), the same shape as the std `*_eq` rows above; dissolves with derived
    //     equality from inhabitance (the cross-representation `==` grounding, DESIGN §3/§4).
    "src/v2/compiler/05_emit_orchestration.dag::orch_emit_let_step",
    "src/v2/lens/live_read_classification.dag::live_read_carrier_eq",
    "src/v2/lens/live_read_classification.dag::path_pattern_eq",
];

fn nfr_strip_comments(content: &str) -> String {
    content
        .lines()
        .map(strip_line_comment)
        .collect::<Vec<_>>()
        .join("\n")
}

fn nfr_closed_coproduct_names(files: &[(String, String)]) -> std::collections::BTreeSet<String> {
    let mut out = std::collections::BTreeSet::new();
    for (rel, content) in files {
        if is_test_dag(rel) {
            continue;
        }
        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;
        while i < lines.len() {
            let trimmed = lines[i].trim_start();
            let Some(rest) = trimmed.strip_prefix("type ") else {
                i += 1;
                continue;
            };
            let name: String = rest
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                .collect();
            if name.is_empty() {
                i += 1;
                continue;
            }
            let mut block = String::new();
            block.push_str(&strip_line_comment(lines[i]));
            let mut depth = brace_delta(lines[i]);
            i += 1;
            while i < lines.len() {
                let nt = lines[i].trim_start();
                if depth <= 0 {
                    if nt.is_empty() {
                        i += 1;
                        continue;
                    }
                    if !(nt.starts_with('|') || nt.starts_with('=')) {
                        break;
                    }
                }
                block.push('\n');
                block.push_str(&strip_line_comment(lines[i]));
                depth += brace_delta(lines[i]);
                i += 1;
            }
            if block.contains('|') {
                out.insert(name);
            }
        }
    }
    out
}

fn nfr_is_ident(s: &str) -> bool {
    !s.is_empty()
        && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
        && !s.chars().next().unwrap().is_ascii_digit()
}

fn nfr_matching_brace(bytes: &[u8], open: usize) -> Option<usize> {
    let mut depth = 0i32;
    let mut j = open;
    while j < bytes.len() {
        match bytes[j] {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(j);
                }
            }
            _ => {}
        }
        j += 1;
    }
    None
}

fn nfr_has_top_level_wildcard_arm(body: &str) -> bool {
    let bytes = body.as_bytes();
    let mut depth = 0i32;
    let mut k = 0;
    while k < bytes.len() {
        match bytes[k] {
            b'{' => depth += 1,
            b'}' => depth -= 1,
            b'_' => {
                let prev_ok = k == 0 || !nfr_is_ident_byte(bytes[k - 1]);
                let next_is_ident = k + 1 < bytes.len() && nfr_is_ident_byte(bytes[k + 1]);
                if depth == 0 && prev_ok && !next_is_ident {
                    let mut m = k + 1;
                    while m < bytes.len()
                        && (bytes[m] == b' ' || bytes[m] == b'\n' || bytes[m] == b'\t')
                    {
                        m += 1;
                    }
                    if m + 1 < bytes.len() && bytes[m] == b'=' && bytes[m + 1] == b'>' {
                        return true;
                    }
                }
            }
            _ => {}
        }
        k += 1;
    }
    false
}

fn nfr_is_ident_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

struct NfrFnSig {
    name: String,
    params: std::collections::BTreeMap<String, String>,
    body: String,
}

fn nfr_parse_fns(src: &str) -> Vec<NfrFnSig> {
    let bytes = src.as_bytes();
    let mut out = Vec::new();
    for (start, _) in src.match_indices("fn ") {
        if start > 0 && nfr_is_ident_byte(bytes[start - 1]) {
            continue;
        }
        let after = start + 3;
        let name: String = src[after..]
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect();
        if name.is_empty() {
            continue;
        }
        let paren_open = match src[after..].find('(') {
            Some(p) => after + p,
            None => continue,
        };
        let paren_close = match nfr_matching_paren(bytes, paren_open) {
            Some(p) => p,
            None => continue,
        };
        let params = nfr_parse_params(&src[paren_open + 1..paren_close]);
        let brace_open = match src[paren_close..].find('{') {
            Some(b) => paren_close + b,
            None => continue,
        };
        let brace_close = match nfr_matching_brace(bytes, brace_open) {
            Some(b) => b,
            None => continue,
        };
        out.push(NfrFnSig {
            name,
            params,
            body: src[brace_open + 1..brace_close].to_string(),
        });
    }
    out
}

fn nfr_matching_paren(bytes: &[u8], open: usize) -> Option<usize> {
    let mut depth = 0i32;
    let mut j = open;
    while j < bytes.len() {
        match bytes[j] {
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(j);
                }
            }
            _ => {}
        }
        j += 1;
    }
    None
}

fn nfr_parse_params(s: &str) -> std::collections::BTreeMap<String, String> {
    let mut out = std::collections::BTreeMap::new();
    let mut depth = 0i32;
    let mut cur = String::new();
    let mut parts: Vec<String> = Vec::new();
    for ch in s.chars() {
        match ch {
            '<' | '(' | '[' => {
                depth += 1;
                cur.push(ch);
            }
            '>' | ')' | ']' => {
                depth -= 1;
                cur.push(ch);
            }
            ',' if depth == 0 => parts.push(std::mem::take(&mut cur)),
            _ => cur.push(ch),
        }
    }
    if !cur.trim().is_empty() {
        parts.push(cur);
    }
    for part in parts {
        let Some((name, ty)) = part.split_once(':') else {
            continue;
        };
        let name = name.trim();
        let ty_head: String = ty
            .trim()
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect();
        if nfr_is_ident(name) && !ty_head.is_empty() {
            out.insert(name.to_string(), ty_head);
        }
    }
    out
}

fn nfr_residue_sites(files: &[(String, String)]) -> Vec<String> {
    let coproducts = nfr_closed_coproduct_names(files);
    let mut out: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for (rel, content) in files {
        if is_test_dag(rel) {
            continue;
        }
        let src = nfr_strip_comments(content);
        for sig in nfr_parse_fns(&src) {
            for (mi, _) in sig.body.match_indices("match ") {
                if mi > 0 && nfr_is_ident_byte(sig.body.as_bytes()[mi - 1]) {
                    continue;
                }
                let after = mi + "match ".len();
                let Some(brace_rel) = sig.body[after..].find('{') else {
                    continue;
                };
                let scrut = sig.body[after..after + brace_rel].trim();
                if !nfr_is_ident(scrut) {
                    continue;
                }
                let Some(ty) = sig.params.get(scrut) else {
                    continue;
                };
                if !coproducts.contains(ty) {
                    continue;
                }
                let body_bytes = sig.body.as_bytes();
                let brace_abs = after + brace_rel;
                let Some(close) = nfr_matching_brace(body_bytes, brace_abs) else {
                    continue;
                };
                let body = &sig.body[brace_abs + 1..close];
                if nfr_has_top_level_wildcard_arm(body) {
                    out.insert(format!("{}::{}", rel, sig.name));
                }
            }
        }
    }
    out.into_iter().collect()
}

struct NonFoldReport {
    sites: Vec<String>,
    coproduct_universe: usize,
    closed_coproduct_names: std::collections::BTreeSet<String>,
}

fn nfr_build_report() -> &'static NonFoldReport {
    static REPORT: std::sync::OnceLock<NonFoldReport> = std::sync::OnceLock::new();
    REPORT.get_or_init(|| {
        let files = corpus_dag_files();
        let closed_coproduct_names = nfr_closed_coproduct_names(&files);
        NonFoldReport {
            sites: nfr_residue_sites(&files),
            coproduct_universe: closed_coproduct_names.len(),
            closed_coproduct_names,
        }
    })
}

pub fn non_fold_residue_closed_coproduct_type_names() -> &'static std::collections::BTreeSet<String>
{
    &nfr_build_report().closed_coproduct_names
}

pub fn non_fold_residue_count() -> i64 {
    nfr_build_report().sites.len() as i64
}

pub fn non_fold_residue_unrostered_count() -> i64 {
    let roster: std::collections::BTreeSet<&str> =
        NON_FOLD_RESIDUE_ROSTER.iter().copied().collect();
    nfr_build_report()
        .sites
        .iter()
        .filter(|s| !roster.contains(s.as_str()))
        .count() as i64
}

pub fn non_fold_residue_site_is_rostered(site: &str) -> bool {
    NON_FOLD_RESIDUE_ROSTER.contains(&site)
}

pub fn non_fold_residue_stale_roster_count() -> i64 {
    let live: std::collections::BTreeSet<&str> = nfr_build_report()
        .sites
        .iter()
        .map(|s| s.as_str())
        .collect();
    NON_FOLD_RESIDUE_ROSTER
        .iter()
        .filter(|s| !live.contains(*s))
        .count() as i64
}

pub fn non_fold_residue_coproduct_universe_count() -> i64 {
    nfr_build_report().coproduct_universe as i64
}

pub fn non_fold_residue_live_sites() -> &'static [String] {
    &nfr_build_report().sites
}

pub fn non_fold_residue_roster_size() -> i64 {
    NON_FOLD_RESIDUE_ROSTER.len() as i64
}

#[cfg(test)]
mod nfr_tests {
    use super::*;

    fn files(pairs: &[(&str, &str)]) -> Vec<(String, String)> {
        pairs
            .iter()
            .map(|(p, c)| (p.to_string(), c.to_string()))
            .collect()
    }

    #[test]
    fn coproduct_index_finds_sums_not_records() {
        let f = files(&[(
            "t.dag",
            "module t\ntype Mode = A | B | C\ntype Rec { x: Int }\ntype Alias = Witness<Int>\n",
        )]);
        let cps = nfr_closed_coproduct_names(&f);
        assert!(cps.contains("Mode"));
        assert!(!cps.contains("Rec"));
        assert!(!cps.contains("Alias"));
    }

    #[test]
    fn red_control_wildcard_over_closed_coproduct_is_residue() {
        let f = files(&[(
            "m.dag",
            "module m\ntype Mode = A | B | C\nfn f(x: Mode) -> Bool {\n  match x {\n    A => true\n    _ => false\n  }\n}\n",
        )]);
        let sites = nfr_residue_sites(&f);
        assert!(
            sites.contains(&"m.dag::f".to_string()),
            "a wildcard over a closed-coproduct param must be flagged; got {sites:?}"
        );
    }

    #[test]
    fn green_control_total_fold_is_not_residue() {
        let f = files(&[(
            "m.dag",
            "module m\ntype Mode = A | B | C\nfn f(x: Mode) -> Bool {\n  match x {\n    A => true\n    B => false\n    C => false\n  }\n}\n",
        )]);
        let sites = nfr_residue_sites(&f);
        assert!(
            !sites.contains(&"m.dag::f".to_string()),
            "an exhaustive match (no wildcard) must NOT be flagged; got {sites:?}"
        );
    }

    #[test]
    fn nfr_roster_receipt() {
        let live: std::collections::BTreeSet<&str> = nfr_build_report()
            .sites
            .iter()
            .map(|s| s.as_str())
            .collect();
        eprintln!(
            "nfr_roster_receipt: unrostered={} stale={} live={}",
            non_fold_residue_unrostered_count(),
            non_fold_residue_stale_roster_count(),
            live.len()
        );
        for site in nfr_build_report().sites.iter() {
            if !non_fold_residue_site_is_rostered(site) {
                eprintln!("unrostered live site: {site}");
            }
            assert!(
                non_fold_residue_site_is_rostered(site),
                "unrostered: {site}"
            );
        }
        for entry in NON_FOLD_RESIDUE_ROSTER {
            if !live.contains(entry) {
                eprintln!("stale roster entry: {entry}");
            }
            assert!(live.contains(entry), "stale roster: {entry}");
        }
    }

    #[test]
    fn green_control_wildcard_over_open_domain_is_not_residue() {
        let f = files(&[(
            "m.dag",
            "module m\ntype Mode = A | B\nfn g(s: String) -> Bool {\n  match s {\n    \"y\" => true\n    _ => false\n  }\n}\n",
        )]);
        let sites = nfr_residue_sites(&f);
        assert!(
            !sites.contains(&"m.dag::g".to_string()),
            "a wildcard over an open/primitive domain must NOT be flagged; got {sites:?}"
        );
    }

    #[test]
    fn green_control_field_placeholder_underscore_is_not_a_wildcard_arm() {
        let f = files(&[(
            "m.dag",
            "module m\ntype Mode = A { v: Int } | B { v: Int }\nfn f(x: Mode) -> Int {\n  match x {\n    A { v: _ } => 1\n    B { v: _ } => 2\n  }\n}\n",
        )]);
        let sites = nfr_residue_sites(&f);
        assert!(
            !sites.contains(&"m.dag::f".to_string()),
            "field-placeholder `_` is not a wildcard arm; got {sites:?}"
        );
    }

    #[test]
    fn nested_match_wildcard_is_attributed_to_its_own_match() {
        let f = files(&[(
            "m.dag",
            "module m\ntype Mode = A | B\nfn eq(a: Mode, b: Mode) -> Bool {\n  match a {\n    A => match b { A => true _ => false }\n    B => match b { B => true _ => false }\n  }\n}\n",
        )]);
        let sites = nfr_residue_sites(&f);
        assert!(sites.contains(&"m.dag::eq".to_string()));
    }

    #[test]
    fn green_control_wildcard_and_slashes_inside_string_literal_are_ignored() {
        let f = files(&[(
            "m.dag",
            "module m\ntype Mode = A | B\nfn f(x: Mode) -> String {\n  match x {\n    A => \"see https://x/y and _ => z\"\n    B => \"b\"\n  }\n}\n",
        )]);
        let sites = nfr_residue_sites(&f);
        assert!(
            !sites.contains(&"m.dag::f".to_string()),
            "`_ =>`/`//` inside a string literal must not be read as code; got {sites:?}"
        );
    }

    #[test]
    fn red_control_real_wildcard_survives_an_in_string_decoy() {
        let f = files(&[(
            "m.dag",
            "module m\ntype Mode = A | B | C\nfn f(x: Mode) -> String {\n  match x {\n    A => \"see https://x/y and _ => z\"\n    _ => \"rest\"\n  }\n}\n",
        )]);
        let sites = nfr_residue_sites(&f);
        assert!(
            sites.contains(&"m.dag::f".to_string()),
            "a real wildcard arm must still be flagged despite an in-string decoy; got {sites:?}"
        );
    }
}

const LANGUAGES_AUTHORITY_REL: &str = "dag/std/languages.dag";

fn languages_census_collect_source_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            languages_census_collect_source_files(&path, out);
        } else {
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if ext == "dag" || ext == "rs" {
                out.push(path);
            }
        }
    }
}

fn languages_census_strip_content(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    let mut chars = source.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '/' && chars.peek() == Some(&'/') {
            while chars.next().is_some_and(|ch| ch != '\n') {}
            out.push('\n');
            continue;
        }
        if c == '"' {
            while let Some(ch) = chars.next() {
                if ch == '\\' {
                    chars.next();
                    continue;
                }
                if ch == '"' {
                    break;
                }
            }
            out.push(' ');
            continue;
        }
        if c == '`' {
            while chars.next().is_some_and(|ch| ch != '`') {}
            out.push(' ');
            continue;
        }
        out.push(c);
    }
    out
}

fn languages_census_extract_data_decl_names(content: &str) -> Vec<String> {
    content
        .lines()
        .filter_map(|line| {
            let rest = line.strip_prefix("data ")?;
            let name: String = rest
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                .collect();
            if name.is_empty() {
                None
            } else {
                Some(name)
            }
        })
        .collect()
}

fn languages_census_is_infrastructure_path(rel: &str) -> bool {
    rel.starts_with("src/v2/test/claim/languages_consumer_census/")
        || rel == "src/v2/lens/languages_consumer_census.dag"
}

fn languages_census_tokenize(content: &str) -> HashSet<String> {
    let stripped = languages_census_strip_content(content);
    stripped
        .split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
        .filter(|token| !token.is_empty())
        .map(str::to_string)
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LanguagesDeclConsumerRecord {
    pub decl_name: String,
    pub external_consumer_paths: Vec<String>,
}

fn languages_decl_records_inner() -> Vec<LanguagesDeclConsumerRecord> {
    let ws = workspace_root();
    let authority = ws.join(LANGUAGES_AUTHORITY_REL);
    let authority_content = std::fs::read_to_string(&authority).unwrap_or_else(|e| {
        panic!(
            "languages_consumer_census: failed to read {}: {e}",
            authority.display()
        )
    });
    let decl_names = languages_census_extract_data_decl_names(&authority_content);
    let decl_name_set: HashSet<String> = decl_names.iter().cloned().collect();

    let mut files = Vec::new();
    for tree in &["dag", "src"] {
        let root = ws.join(tree);
        if root.is_dir() {
            languages_census_collect_source_files(&root, &mut files);
        }
    }

    let mut by_decl: HashMap<String, HashSet<String>> = decl_names
        .iter()
        .map(|name| (name.clone(), HashSet::new()))
        .collect();

    for path in files {
        let rel = path
            .strip_prefix(&ws)
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .unwrap_or_default();
        if rel == LANGUAGES_AUTHORITY_REL || languages_census_is_infrastructure_path(&rel) {
            continue;
        }
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let tokens = languages_census_tokenize(&content);
        for decl_name in tokens.intersection(&decl_name_set) {
            by_decl
                .get_mut(decl_name)
                .expect("decl map key")
                .insert(rel.clone());
        }
    }

    let mut records = Vec::new();
    for decl_name in decl_names {
        let mut paths: Vec<String> = by_decl
            .remove(&decl_name)
            .expect("decl map key")
            .into_iter()
            .collect();
        paths.sort();
        records.push(LanguagesDeclConsumerRecord {
            decl_name,
            external_consumer_paths: paths,
        });
    }
    records
}

fn languages_decl_records_cached() -> &'static [LanguagesDeclConsumerRecord] {
    static RECORDS: OnceLock<Vec<LanguagesDeclConsumerRecord>> = OnceLock::new();
    RECORDS.get_or_init(languages_decl_records_inner)
}

fn languages_decl_record_for(decl_name: &str) -> Option<&'static LanguagesDeclConsumerRecord> {
    languages_decl_records_cached()
        .iter()
        .find(|r| r.decl_name == decl_name)
}

pub fn languages_consumer_census_data_decl_count() -> i64 {
    languages_decl_records_cached().len() as i64
}

pub fn languages_consumer_census_per_language_row_count() -> i64 {
    languages_decl_records_cached()
        .iter()
        .filter(|r| !r.decl_name.ends_with("_format"))
        .count() as i64
}

pub fn languages_consumer_census_format_row_count() -> i64 {
    languages_decl_records_cached()
        .iter()
        .filter(|r| r.decl_name.ends_with("_format"))
        .count() as i64
}

pub fn languages_consumer_census_external_consumer_count(decl_name: String) -> i64 {
    languages_decl_record_for(&decl_name)
        .map(|r| r.external_consumer_paths.len() as i64)
        .unwrap_or(-1)
}

pub fn languages_consumer_census_is_composition_only(decl_name: String) -> bool {
    languages_decl_record_for(&decl_name)
        .map(|r| r.external_consumer_paths.is_empty())
        .unwrap_or(false)
}

pub fn languages_consumer_census_has_external_consumer(decl_name: String) -> bool {
    languages_decl_record_for(&decl_name)
        .map(|r| !r.external_consumer_paths.is_empty())
        .unwrap_or(false)
}

// --- Inert carrier census (folded from inert_carrier_project.rs) ---
//
// A type carrier is "inert" iff (a) declared in a non-test file, (b) its name appears in at least
// one *_test.dag file (self-tested), and (c) its name appears in NO non-test .dag file outside its
// own declaration block (zero real consumer). This is DESIGN §5 coverage-by-illusion.
// DISSOLUTION TRIGGER: when .dag gains compile-graph / reference-edge access (gunbc#5364), the
// token scan folds into a pure .dag reader over BindsTo edges and this Rust census deletes.

fn inert_carrier_identifier_tokens(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    for ch in line.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            cur.push(ch);
        } else if !cur.is_empty() {
            out.push(std::mem::take(&mut cur));
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

fn inert_carrier_count_token(text: &str, name: &str) -> i64 {
    let mut n = 0i64;
    for raw in text.lines() {
        for tok in inert_carrier_identifier_tokens(&strip_line_comment(raw)) {
            if tok == name {
                n += 1;
            }
        }
    }
    n
}

fn inert_carrier_type_carrier_blocks(content: &str) -> Vec<(String, String)> {
    let lines: Vec<&str> = content.lines().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let trimmed = lines[i].trim_start();
        let Some(rest) = trimmed.strip_prefix("type ") else {
            i += 1;
            continue;
        };
        let name: String = rest
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect();
        if name.is_empty() {
            i += 1;
            continue;
        }
        let mut block = String::new();
        block.push_str(lines[i]);
        block.push('\n');
        let mut depth = brace_delta(lines[i]);
        i += 1;
        while i < lines.len() {
            let nt = lines[i].trim_start();
            if depth <= 0 {
                if !(nt.starts_with('|') || nt.starts_with('=')) {
                    break;
                }
            }
            block.push_str(lines[i]);
            block.push('\n');
            depth += brace_delta(lines[i]);
            i += 1;
        }
        out.push((name, block));
    }
    out
}

// A coproduct is consumed through its variant names (constructors, match arms) at least as
// often as through the type name itself, which may appear only at the declaration. Credit
// variant occurrences to the parent type or a live state machine reads as inert. Variant
// names shared across coproducts merge their tallies — an approximation that errs toward
// not flagging; the roster stays the per-name override.
fn inert_carrier_variant_names(block: &str) -> Vec<String> {
    let mut out = Vec::new();
    for (idx, raw) in block.lines().enumerate() {
        let t = raw.trim_start();
        let payload = if idx == 0 {
            match t.find('=') {
                Some(p) => &t[p + 1..],
                None => continue,
            }
        } else if let Some(rest) = t.strip_prefix('=') {
            rest
        } else if let Some(rest) = t.strip_prefix('|') {
            rest
        } else {
            continue;
        };
        for seg in payload.split('|') {
            let name: String = seg
                .trim_start()
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                .collect();
            if !name.is_empty() && name.chars().next().is_some_and(|c| c.is_ascii_uppercase()) {
                out.push(name);
            }
        }
    }
    out.sort();
    out.dedup();
    out
}

const DOC_PLAN_ROOTS: &[&str] = &["ROADMAP.md", "DESIGN.md"];
const DOC_RUNBOOK_ROOT: &str = "docs/runbooks/README.md";

fn doc_repo_rel(path: &Path) -> String {
    let ws = workspace_root();
    let s = path.to_string_lossy().replace('\\', "/");
    let prefix = format!("{}/", ws.to_string_lossy().replace('\\', "/"));
    s.strip_prefix(&prefix)
        .map(|p| p.to_string())
        .unwrap_or(s)
        .trim_start_matches("./")
        .to_string()
}

fn doc_universe() -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let docs_dir = workspace_root().join("docs");
    collect_md_files(&docs_dir, &mut out);
    out
}

fn collect_md_files(dir: &Path, out: &mut BTreeSet<String>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_md_files(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            out.insert(doc_repo_rel(&path));
        }
    }
}

fn markdown_link_targets(content: &str) -> Vec<String> {
    let mut out = Vec::new();
    let bytes = content.as_bytes();
    let mut i = 0;
    while i + 1 < bytes.len() {
        if bytes[i] == b']' && bytes[i + 1] == b'(' {
            if let Some(end) = content[i + 2..].find(')') {
                let raw = &content[i + 2..i + 2 + end];
                let target = raw.split('#').next().unwrap_or("").trim();
                if !target.is_empty()
                    && !target.starts_with("http://")
                    && !target.starts_with("https://")
                    && !target.starts_with("mailto:")
                {
                    out.push(target.to_string());
                }
                i = i + 2 + end + 1;
                continue;
            }
        }
        i += 1;
    }
    out
}

struct InertCarrierData {
    declared_count: usize,
    inert_names: Vec<String>,
}

fn compute_inert_carrier_data(files: &[(String, String)]) -> InertCarrierData {
    let mut declared: BTreeMap<String, String> = BTreeMap::new();
    let mut decl_count: BTreeMap<String, usize> = BTreeMap::new();
    let mut self_block_refs: BTreeMap<String, i64> = BTreeMap::new();
    let mut type_variants: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (rel, content) in files {
        if is_test_dag(rel) {
            continue;
        }
        for (name, block) in inert_carrier_type_carrier_blocks(content) {
            declared.entry(name.clone()).or_insert_with(|| rel.clone());
            *decl_count.entry(name.clone()).or_insert(0) += 1;
            *self_block_refs.entry(name.clone()).or_insert(0) +=
                inert_carrier_count_token(&block, &name);
            for v in inert_carrier_variant_names(&block) {
                *self_block_refs.entry(v.clone()).or_insert(0) +=
                    inert_carrier_count_token(&block, &v);
                type_variants.entry(name.clone()).or_default().push(v);
            }
        }
    }
    let mut names: BTreeSet<String> = declared.keys().cloned().collect();
    for vs in type_variants.values() {
        names.extend(vs.iter().cloned());
    }
    let mut nontest_occ: BTreeMap<String, i64> = BTreeMap::new();
    let mut self_tested: BTreeSet<String> = BTreeSet::new();
    for (rel, content) in files {
        let mut local: BTreeMap<String, i64> = BTreeMap::new();
        for raw in content.lines() {
            for tok in inert_carrier_identifier_tokens(&strip_line_comment(raw)) {
                if names.contains(&tok) {
                    *local.entry(tok).or_insert(0) += 1;
                }
            }
        }
        if is_test_dag(rel) {
            for (k, _) in local {
                self_tested.insert(k);
            }
        } else {
            for (k, v) in local {
                *nontest_occ.entry(k).or_insert(0) += v;
            }
        }
    }
    let mut inert_names: Vec<String> = Vec::new();
    for name in declared.keys() {
        if decl_count.get(name).copied().unwrap_or(0) != 1 {
            continue;
        }
        if !self_tested.contains(name) {
            continue;
        }
        let total = nontest_occ.get(name).copied().unwrap_or(0);
        let own = self_block_refs.get(name).copied().unwrap_or(0);
        let mut consumption = total - own;
        for v in type_variants.get(name).map(|v| v.as_slice()).unwrap_or(&[]) {
            let vtotal = nontest_occ.get(v).copied().unwrap_or(0);
            let vown = self_block_refs.get(v).copied().unwrap_or(0);
            consumption += vtotal - vown;
        }
        if consumption <= 0 {
            inert_names.push(name.clone());
        }
    }
    inert_names.sort();
    inert_names.dedup();
    InertCarrierData {
        declared_count: declared.len(),
        inert_names,
    }
}

fn build_inert_carrier_data() -> &'static InertCarrierData {
    static CACHE: OnceLock<InertCarrierData> = OnceLock::new();
    CACHE.get_or_init(|| compute_inert_carrier_data(&corpus_dag_files()))
}

pub fn inert_carrier_names_live() -> Vec<String> {
    build_inert_carrier_data().inert_names.clone()
}

pub fn inert_carrier_declared_count_live() -> i64 {
    build_inert_carrier_data().declared_count as i64
}

#[cfg(test)]
mod inert_carrier_tests {
    use super::*;

    fn inert_names_of(files: &[(&str, &str)]) -> Vec<String> {
        let owned: Vec<(String, String)> = files
            .iter()
            .map(|(p, c)| (p.to_string(), c.to_string()))
            .collect();
        compute_inert_carrier_data(&owned).inert_names
    }

    #[test]
    fn type_carrier_blocks_extracts_names_and_bodies() {
        let c = "module m\ntype Connective = Atom | Conj\ntype WorkDemand {\n  field: Int\n}\nfn f() -> Int { 1 }\n";
        let blocks = inert_carrier_type_carrier_blocks(c);
        let names: Vec<&String> = blocks.iter().map(|(n, _)| n).collect();
        assert_eq!(names, vec!["Connective", "WorkDemand"]);
        let wd = &blocks.iter().find(|(n, _)| n == "WorkDemand").unwrap().1;
        assert!(wd.contains("field: Int") && wd.contains('}'));
        assert!(!wd.contains("fn f"));
    }

    #[test]
    fn identifier_tokens_are_whole_words() {
        let toks = inert_carrier_identifier_tokens("  field: PlacementSupply = foo(Placement)");
        assert!(toks.contains(&"PlacementSupply".to_string()));
        assert!(toks.contains(&"Placement".to_string()));
        assert!(toks.contains(&"field".to_string()));
    }

    #[test]
    fn variant_consumed_coproduct_is_not_inert() {
        // The RustGramBuild1State shape: type name appears only at its declaration;
        // consumption is entirely via variant constructors/match arms in a nontest fn.
        let inert = inert_names_of(&[
            (
                "a.dag",
                "module a\ntype BuildState\n  = BuildInit\n  | BuildReady { expr: Int }\nfn go(x: Int) -> Int {\n  match BuildInit {\n    BuildInit => 0\n    BuildReady { expr: e } => e\n  }\n}\n",
            ),
            (
                "a_test.dag",
                "module t\nfn t() -> Bool { go(x: 1) == 0 }\nfn probe(s: BuildState) -> Bool { true }\n",
            ),
        ]);
        assert!(
            !inert.contains(&"BuildState".to_string()),
            "variant-mediated consumption must credit the parent type; got {inert:?}"
        );
    }

    #[test]
    fn red_control_variant_dead_coproduct_stays_inert() {
        let inert = inert_names_of(&[
            (
                "a.dag",
                "module a\ntype LonelyState\n  = LonelyInit\n  | LonelyReady { expr: Int }\n",
            ),
            (
                "a_test.dag",
                "module t\nfn t() -> Bool { match LonelyInit { LonelyInit => true _ => false } }\nfn probe(s: LonelyState) -> Bool { true }\n",
            ),
        ]);
        assert!(
            inert.contains(&"LonelyState".to_string()),
            "a coproduct whose variants are used nowhere outside its block must stay flagged; got {inert:?}"
        );
    }

    #[test]
    fn red_control_self_tested_zero_consumer_carrier_is_inert() {
        let inert = inert_names_of(&[
            ("a.dag", "module a\ntype Lonely { x: Int }\n"),
            (
                "a_test.dag",
                "module t\nfn t() -> Bool { Lonely { x: 1 } == Lonely { x: 1 } }\n",
            ),
        ]);
        assert!(
            inert.contains(&"Lonely".to_string()),
            "a self-tested carrier with no real consumer must be flagged inert; got {inert:?}"
        );
    }

    #[test]
    fn green_control_carrier_with_real_consumer_is_not_inert() {
        let inert = inert_names_of(&[
            ("a.dag", "module a\ntype Used { x: Int }\n"),
            (
                "b.dag",
                "module b\nimport a { Used }\nfn f(u: Used) -> Int { u.x }\n",
            ),
            (
                "a_test.dag",
                "module t\nfn t() -> Bool { Used { x: 1 } == Used { x: 1 } }\n",
            ),
        ]);
        assert!(
            !inert.contains(&"Used".to_string()),
            "a carrier with a real (non-test, cross-file) consumer must NOT be flagged; got {inert:?}"
        );
    }

    #[test]
    fn green_control_same_file_consumer_is_not_inert() {
        let inert = inert_names_of(&[
            (
                "lens.dag",
                "module lens\ntype LocalFact { x: Int }\nfn clean(fs: LocalFact) -> Bool { fs.x == 0 }\n",
            ),
            ("lens_test.dag", "module t\nfn t() -> Bool { clean(fs: LocalFact { x: 0 }) }\n"),
        ]);
        assert!(
            !inert.contains(&"LocalFact".to_string()),
            "a carrier consumed by a fn in its own file is NOT inert; got {inert:?}"
        );
    }

    #[test]
    fn green_control_untested_unused_carrier_is_not_flagged() {
        let inert = inert_names_of(&[("a.dag", "module a\ntype Staged { x: Int }\n")]);
        assert!(
            !inert.contains(&"Staged".to_string()),
            "an untested unused carrier must NOT be flagged (it is model-first, not illusion); got {inert:?}"
        );
    }

    #[test]
    fn comment_reference_is_not_a_real_consumer() {
        let inert = inert_names_of(&[
            ("a.dag", "module a\ntype Noted { x: Int }\n"),
            (
                "b.dag",
                "module b\n// Noted is described here\nfn f() -> Int { 1 }\n",
            ),
            (
                "a_test.dag",
                "module t\nfn t() -> Bool { Noted { x: 1 } == Noted { x: 1 } }\n",
            ),
        ]);
        assert!(inert.contains(&"Noted".to_string()));
    }

    #[test]
    fn doubly_declared_name_is_not_flagged() {
        let inert = inert_names_of(&[
            ("a.dag", "module a\ntype Dup { x: Int }\n"),
            ("b.dag", "module b\ntype Dup { y: Int }\n"),
        ]);
        assert!(!inert.contains(&"Dup".to_string()));
    }
}

// --- Complexity/linearity syntactic audit (folded from complexity_linearity_audit_project.rs) ---
//
// Thin host builtins over `decl_facts` + fn-body AST walk. Triage/bucket classification and the
// migration-debt roster live in `v2.lens.complexity_linearity_audit` (.dag).
// REMAINING GATE (#5364 partial): `decl_facts` exposes corpus `Node`s but v2 `.dag` has no
// `expr_data` / `MatchPattern` introspection — the wildcard-arm walk stays in this host seam
// until a `.dag`-accessible match-body reader lands (same residue class as inert_carrier_*).

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ComplexityLinearityAuditFinding {
    pub site: String,
    pub lens: &'static str,
    pub rule: &'static str,
    pub triage: &'static str,
}

#[derive(Debug, Clone, Default)]
pub struct ComplexityLinearityAuditSummary {
    pub files_scanned: usize,
    pub files_parsed: usize,
    pub fns_scanned: usize,
    pub findings: Vec<ComplexityLinearityAuditFinding>,
}

fn cla_is_wildcard_arm(arm: &Rc<Node>) -> bool {
    matches!(arm_pattern(arm.clone()).as_ref(), MatchPattern::Wildcard)
}

fn cla_type_expr_head(ty: Rc<Node>, si: &Rc<HashMap<String, Rc<NewlineIndex>>>) -> String {
    let name = authored_name_at(si.clone(), ty);
    name.chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
        .collect()
}

fn cla_fn_param_type_heads(
    item: &Rc<Node>,
    si: &Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for param in item.params.iter() {
        let pname = param_node_name_at(param.clone(), si.clone());
        if pname.is_empty() {
            continue;
        }
        let head = cla_type_expr_head(param_node_type_expr(param.clone()), si);
        if !head.is_empty() {
            out.insert(pname, head);
        }
    }
    out
}

fn cla_is_closed_coproduct_param_scrutinee(
    scrutinee_name: &str,
    param_types: &BTreeMap<String, String>,
    closed: &BTreeSet<String>,
) -> bool {
    param_types
        .get(scrutinee_name)
        .is_some_and(|ty| closed.contains(ty))
}

#[derive(Default)]
struct ClaFnBodyStats {
    node_count: usize,
    match_count: usize,
    wildcard_matches: usize,
    closed_coproduct_wildcard_matches: usize,
}

fn cla_walk_expr(
    node: &Rc<Node>,
    si: &Rc<HashMap<String, Rc<NewlineIndex>>>,
    param_types: &BTreeMap<String, String>,
    closed_coproducts: &BTreeSet<String>,
    stats: &mut ClaFnBodyStats,
) {
    stats.node_count += 1;
    if let ExprData::ExprMatch = node.expr_data.as_ref() {
        stats.match_count += 1;
        let scrutinee = match_scrutinee(node.clone());
        let scrutinee_name = expr_var_name_at(scrutinee, si.clone());
        let has_wildcard = match_arm_nodes(node.clone())
            .iter()
            .any(|arm| cla_is_wildcard_arm(arm));
        if has_wildcard {
            stats.wildcard_matches += 1;
            if !scrutinee_name.is_empty()
                && cla_is_closed_coproduct_param_scrutinee(
                    &scrutinee_name,
                    param_types,
                    closed_coproducts,
                )
            {
                stats.closed_coproduct_wildcard_matches += 1;
            }
        }
    }
    for child in node.children.iter() {
        cla_walk_expr(child, si, param_types, closed_coproducts, stats);
    }
}

fn cla_is_kernel_permanent_fn(fn_name: &str) -> bool {
    fn_name.ends_with("_eq")
        || fn_name.contains("dominates")
        || fn_name.contains("lattice_join")
        || fn_name.contains("lattice_meet")
        || fn_name == "exit_ok"
        || fn_name.contains("_relation_eq")
        || fn_name.contains("_mode_eq")
        || fn_name.ends_with("_combine")
        || fn_name == "constant_bound_value"
        || fn_name == "is_constant_bound"
        || fn_name == "create_double_init_collapsible"
        || fn_name == "create_effect_is_dedupable"
        || fn_name.starts_with("compose_sub_value")
        || fn_name == "promote_to_strict"
        || fn_name.starts_with("program_runtime_bool")
        || fn_name == "is_text_encoding"
        || fn_name == "is_strict_style_structural"
}

fn cla_triage_complexity(site: &str) -> &'static str {
    let fn_name = site.rsplit("::").next().unwrap_or("");
    if cla_is_kernel_permanent_fn(fn_name) {
        return "kernel-permanent";
    }
    if site.starts_with("dag/extdeps/")
        || site.starts_with("dag/gunbc/plans/")
        || site.starts_with("dag/test/")
    {
        "open-domain"
    } else if site.starts_with("dag/std/") || site.starts_with("dag/gunbc/") {
        "kernel-permanent"
    } else {
        "open-domain"
    }
}

fn cla_audit_function_body(
    rel: &str,
    fn_name: &str,
    body: &Rc<Node>,
    si: &Rc<HashMap<String, Rc<NewlineIndex>>>,
    param_types: &BTreeMap<String, String>,
) -> Vec<ComplexityLinearityAuditFinding> {
    let closed = non_fold_residue_closed_coproduct_type_names();
    let mut stats = ClaFnBodyStats::default();
    cla_walk_expr(body, si, param_types, closed, &mut stats);
    let site = format!("{rel}::{fn_name}");
    let mut out = Vec::new();
    if stats.wildcard_matches > 0 {
        out.push(ComplexityLinearityAuditFinding {
            site: site.clone(),
            lens: "non_fold_residue",
            rule: "syntactic_match_wildcard_arm",
            triage: "wildcard-arm",
        });
    }
    if stats.match_count >= 8 || (stats.node_count >= 200 && stats.match_count >= 4) {
        out.push(ComplexityLinearityAuditFinding {
            site,
            lens: "cost",
            rule: "syntactic_high_match_fanout",
            triage: cla_triage_complexity(&format!("{rel}::{fn_name}")),
        });
    }
    out
}

fn cla_audit_decl_fact(
    fact: &DeclFactRaw,
    si: &Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> Vec<ComplexityLinearityAuditFinding> {
    let Some(body) = fact.node.body.as_ref() else {
        return Vec::new();
    };
    let param_types = cla_fn_param_type_heads(&fact.node, si);
    cla_audit_function_body(&fact.rel_path, &fact.name, body, si, &param_types)
}

pub fn complexity_linearity_audit_corpus_over_decl_facts(
    roots: &[String],
) -> ComplexityLinearityAuditSummary {
    let walk = decl_facts_corpus_walk(roots);
    let mut summary = ComplexityLinearityAuditSummary::default();
    summary.files_scanned = walk.files_scanned;
    summary.files_parsed = walk.files_parsed;

    for fact in &walk.facts {
        if !matches!(fact.kind, ItemKind::FnItem | ItemKind::FuncItem) {
            continue;
        }
        summary.fns_scanned += 1;
        summary
            .findings
            .extend(cla_audit_decl_fact(fact, &fact.source_indices));
    }
    summary.findings.sort();
    summary
}

pub fn complexity_linearity_audit_corpus_parse_only(
    roots: &[String],
) -> ComplexityLinearityAuditSummary {
    complexity_linearity_audit_corpus_over_decl_facts(roots)
}

pub fn complexity_linearity_audit_corpus_default_roots() -> ComplexityLinearityAuditSummary {
    complexity_linearity_audit_corpus_parse_only(&witness_layer_roots())
}

struct ClaAuditBuiltinCache {
    finding_count: i64,
    sites: BTreeSet<String>,
}

fn cla_cached_builtin_cache() -> &'static ClaAuditBuiltinCache {
    static CACHE: OnceLock<ClaAuditBuiltinCache> = OnceLock::new();
    CACHE.get_or_init(|| {
        let summary = complexity_linearity_audit_corpus_default_roots();
        ClaAuditBuiltinCache {
            finding_count: summary.findings.len() as i64,
            sites: summary.findings.iter().map(|f| f.site.clone()).collect(),
        }
    })
}

pub fn complexity_linearity_syntactic_finding_count() -> i64 {
    cla_cached_builtin_cache().finding_count
}

pub fn complexity_linearity_syntactic_site_fired(site: &str) -> bool {
    cla_cached_builtin_cache().sites.contains(site)
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ComplexityLinearityWildcardFactRaw {
    pub site: String,
    pub fn_name: String,
    pub closed_coproduct_wildcard: bool,
    pub rostered: bool,
}

struct ClaWildcardFactsCache {
    facts: Vec<ComplexityLinearityWildcardFactRaw>,
}

fn cla_compute_wildcard_facts(roots: &[String]) -> Vec<ComplexityLinearityWildcardFactRaw> {
    let walk = decl_facts_corpus_walk(roots);
    let closed = non_fold_residue_closed_coproduct_type_names();
    let mut out = Vec::new();
    for fact in &walk.facts {
        if !matches!(fact.kind, ItemKind::FnItem | ItemKind::FuncItem) {
            continue;
        }
        let Some(body) = fact.node.body.as_ref() else {
            continue;
        };
        let param_types = cla_fn_param_type_heads(&fact.node, &fact.source_indices);
        let mut stats = ClaFnBodyStats::default();
        cla_walk_expr(body, &fact.source_indices, &param_types, closed, &mut stats);
        if stats.wildcard_matches > 0 {
            let site = format!("{}::{}", fact.rel_path, fact.name);
            out.push(ComplexityLinearityWildcardFactRaw {
                fn_name: fact.name.clone(),
                closed_coproduct_wildcard: stats.closed_coproduct_wildcard_matches > 0,
                rostered: non_fold_residue_site_is_rostered(&site),
                site,
            });
        }
    }
    out.sort();
    out.dedup();
    out
}

fn cla_cached_wildcard_facts() -> &'static ClaWildcardFactsCache {
    static CACHE: OnceLock<ClaWildcardFactsCache> = OnceLock::new();
    CACHE.get_or_init(|| ClaWildcardFactsCache {
        facts: cla_compute_wildcard_facts(&witness_layer_roots()),
    })
}

pub fn complexity_linearity_wildcard_facts() -> &'static [ComplexityLinearityWildcardFactRaw] {
    &cla_cached_wildcard_facts().facts
}

#[cfg(test)]
mod complexity_linearity_audit_tests {
    use super::*;
    use std::fs;
    use std::path::Path;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn write_temp_module(content: &str) -> String {
        let dir = std::env::temp_dir().join(format!(
            "complexity-linearity-audit-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("time")
                .as_nanos()
        ));
        fs::create_dir_all(&dir).expect("tempdir");
        let path = dir.join("audit_wildcard.dag");
        fs::write(&path, content).expect("write");
        path.to_string_lossy().to_string()
    }

    #[test]
    fn syntactic_wildcard_finding_on_closed_coproduct_match() {
        let path = write_temp_module(
            "module audit_wildcard\n\
             type Mode = A | B | C\n\
             fn f(x: Mode) -> Bool {\n\
               match x {\n\
                 A => true\n\
                 _ => false\n\
               }\n\
             }\n",
        );
        let root = Path::new(&path)
            .parent()
            .unwrap()
            .to_string_lossy()
            .to_string();
        let summary = complexity_linearity_audit_corpus_parse_only(&[root]);
        assert!(
            summary
                .findings
                .iter()
                .any(|f| { f.rule == "syntactic_match_wildcard_arm" && f.site.contains("::f") }),
            "expected wildcard finding; got {:?}",
            summary.findings
        );
    }

    #[test]
    fn eval_interpreter_handler_is_migration_debt_raw_fact() {
        let facts = complexity_linearity_wildcard_facts();
        let eval_bind_site = "src/v2/compiler/05_eval.dag::eval_bind_node_eval";
        assert!(
            !facts.iter().any(|f| f.site == eval_bind_site),
            "eval_bind_node_eval wildcard dissolved; should not appear in wildcard facts"
        );
        let site = "src/v2/compiler/05_eval.dag::eval_match_node_eval";
        let fact = facts.iter().find(|f| f.site == site);
        assert!(fact.is_some(), "expected wildcard fact for {site}");
        assert!(
            fact.unwrap().rostered,
            "{site} must be rostered (drives migration-debt/kernel-permanent triage in .dag)"
        );
    }

    #[test]
    fn testgen_anchor_match_is_migration_debt_raw_fact() {
        let site = "src/v2/lens/testgen.dag::testgen_emit_language_behavior_equivalence_claim";
        let facts = complexity_linearity_wildcard_facts();
        assert!(
            facts.iter().any(|f| f.site == site),
            "expected wildcard fact for testgen anchor match"
        );
    }

    #[test]
    fn live_tree_parse_audit_runs_over_witness_roots() {
        let summary = complexity_linearity_audit_corpus_default_roots();
        assert!(summary.files_scanned > 100, "corpus walk fail-opened");
        assert!(summary.files_parsed > 50, "parse fail-opened");
        assert!(summary.fns_scanned > 100, "fn scan fail-opened");
        assert!(
            !summary.findings.is_empty(),
            "expected syntactic findings on the live corpus"
        );
    }
}

fn resolve_doc_link(from: &str, target: &str) -> Vec<String> {
    let mut candidates = Vec::new();
    let from_dir = Path::new(from).parent().unwrap_or_else(|| Path::new(""));
    candidates.push(normalize_doc_path(&from_dir.join(target)));
    candidates.push(normalize_doc_path(Path::new(target)));
    candidates.dedup();
    candidates
}

fn normalize_doc_path(path: &Path) -> String {
    let mut parts: Vec<String> = Vec::new();
    for comp in path.to_string_lossy().replace('\\', "/").split('/') {
        match comp {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            other => parts.push(other.to_string()),
        }
    }
    parts.join("/")
}

fn dag_comment_bind_doc_refs() -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for root in witness_layer_roots() {
        let mut dag_files = Vec::new();
        collect_dag_files_tolerant(&workspace_root().join(&root), &mut dag_files);
        for path in dag_files {
            let content = match std::fs::read_to_string(&path) {
                Ok(c) => c,
                Err(_) => continue,
            };
            for target in bind_md_refs(&content) {
                out.insert(target);
            }
        }
    }
    out
}

fn bind_md_refs(content: &str) -> Vec<String> {
    let mut out = Vec::new();
    for (idx, _) in content.match_indices("bind:") {
        let rest = content[idx + "bind:".len()..].trim_start();
        let token: String = rest
            .chars()
            .take_while(|c| !c.is_whitespace() && *c != ')' && *c != '"' && *c != '`')
            .collect();
        if token.ends_with(".md") {
            out.push(normalize_doc_path(Path::new(&token)));
        }
    }
    out
}

fn doc_reachable_set(
    roots: &BTreeSet<String>,
    edges: &HashMap<String, Vec<String>>,
) -> BTreeSet<String> {
    let mut reached: BTreeSet<String> = BTreeSet::new();
    let mut queue: VecDeque<String> = VecDeque::new();
    for r in roots {
        if reached.insert(r.clone()) {
            queue.push_back(r.clone());
        }
    }
    while let Some(node) = queue.pop_front() {
        if let Some(neighbors) = edges.get(&node) {
            for n in neighbors {
                if reached.insert(n.clone()) {
                    queue.push_back(n.clone());
                }
            }
        }
    }
    reached
}

struct DocGraphReport {
    doc_count: usize,
    orphans: Vec<String>,
    dangling: Vec<(String, String)>,
}

fn build_doc_graph_report() -> DocGraphReport {
    let universe = doc_universe();
    let bind_refs = dag_comment_bind_doc_refs();

    let mut roots: BTreeSet<String> = BTreeSet::new();
    for r in DOC_PLAN_ROOTS {
        roots.insert((*r).to_string());
    }
    if workspace_root().join(DOC_RUNBOOK_ROOT).is_file() {
        roots.insert(DOC_RUNBOOK_ROOT.to_string());
    }
    for b in &bind_refs {
        if universe.contains(b) {
            roots.insert(b.clone());
        }
    }

    let mut edges: HashMap<String, Vec<String>> = HashMap::new();
    let mut dangling: Vec<(String, String)> = Vec::new();
    let mut sources: Vec<String> = universe.iter().cloned().collect();
    for r in DOC_PLAN_ROOTS {
        sources.push((*r).to_string());
    }
    if roots.contains(DOC_RUNBOOK_ROOT) {
        sources.push(DOC_RUNBOOK_ROOT.to_string());
    }
    sources.sort();
    sources.dedup();
    for src in &sources {
        let content = match std::fs::read_to_string(workspace_root().join(src)) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let mut out_edges: Vec<String> = Vec::new();
        for target in markdown_link_targets(&content) {
            let candidates = resolve_doc_link(src, &target);
            let existing = candidates
                .iter()
                .find(|c| workspace_root().join(c).is_file())
                .cloned();
            match existing {
                Some(path) => out_edges.push(path),
                None => {
                    if target.ends_with(".md") {
                        dangling.push((src.clone(), target.clone()));
                    }
                }
            }
        }
        edges.insert(src.clone(), out_edges);
    }

    let reached = doc_reachable_set(&roots, &edges);
    let orphans: Vec<String> = universe
        .iter()
        .filter(|d| !reached.contains(*d))
        .cloned()
        .collect();
    dangling.sort();
    dangling.dedup();
    DocGraphReport {
        doc_count: universe.len(),
        orphans,
        dangling,
    }
}

fn doc_graph_report() -> &'static DocGraphReport {
    static REPORT: OnceLock<DocGraphReport> = OnceLock::new();
    REPORT.get_or_init(build_doc_graph_report)
}

pub fn doc_graph_orphan_count() -> i64 {
    doc_graph_report().orphans.len() as i64
}

pub fn doc_graph_dangling_link_count() -> i64 {
    doc_graph_report().dangling.len() as i64
}

pub fn doc_graph_doc_count() -> i64 {
    doc_graph_report().doc_count as i64
}

// Live derivation of docs/plans/seed-shrink-census.md §5B ("T2 coverage debt"): that table was a
// hand-maintained snapshot of v1 test modules with no floor `*_test.dag` equivalent. This walks
// `src/v1/tests/src/*.rs` (modules containing `#[test]`) and `corpus_dag_files()` (the same
// witness-layer-roots roster the floor uses) and diffs them by stem, so the debt roster tracks
// the live tree instead of drifting the moment either side changes.
struct TestMigrationDebtEntry {
    module: String,
    loc: i64,
    test_fn_count: i64,
}

struct TestMigrationDebtReport {
    entries: Vec<TestMigrationDebtEntry>,
}

fn test_migration_debt_v1_test_dir() -> PathBuf {
    workspace_root().join("src/v1/tests/src")
}

fn test_migration_debt_stem(name: &str) -> String {
    let stem = name
        .strip_suffix(".rs")
        .or_else(|| name.strip_suffix(".dag"))
        .unwrap_or(name);
    stem.strip_suffix("_test").unwrap_or(stem).to_string()
}

fn test_migration_debt_floor_stems() -> Vec<String> {
    let mut stems: Vec<String> = corpus_dag_files()
        .into_iter()
        .map(|(path, _)| path)
        .filter(|p| is_test_dag(p))
        .map(|p| {
            let file_name = std::path::Path::new(&p)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(&p)
                .to_string();
            test_migration_debt_stem(&file_name)
        })
        .collect();
    stems.sort();
    stems.dedup();
    stems
}

// Exact-stem equality only. A substring match (either direction) was tried and reviewed
// unsound: e.g. v1 stem "pipeline" (the single largest debt module, 418 `#[test]` fns) is a
// substring of the floor stem "typescript_import_pipeline", so a fuzzy match falsely marked
// the whole module covered — hiding debt rather than counting it. Exact equality is decidable
// and cannot understate debt; it may list a module the operator judges topically covered by a
// differently-named floor witness, which is a correct false-debt (never a false-coverage) bias.
fn test_migration_debt_stem_covered(v1_stem: &str, floor_stems: &[String]) -> bool {
    floor_stems.iter().any(|floor_stem| floor_stem == v1_stem)
}

// Second stem source (typed retirement path): a `<stem>_retired.dag` declaration under the
// corpus records a reviewed, typed retirement (delete-redundant / delete-low-value) for a v1
// test module whose behavior does NOT migrate to an exact-stem floor `*_test.dag` witness. A
// retired stem covers the module identically to a floor-witness stem — it excludes the module
// from the debt roster and authorizes its delete through the delete-guard. The typed disposition
// and its justification live in the `.dag` decl (`test.retirement.model`, single authority,
// type-checked by the compile-clean gate); this guard reads only the filename stem, exactly as
// it reads floor witnesses. A file counts only if it actually *constructs* a `TestModuleRetirement`
// (the `TestModuleRetirement {` constructor form) — an empty stub, or one that merely imports the
// type without declaring a retirement (`{ TestModuleRetirement }`), cannot silence the guard. The
// compile-clean gate independently type-checks the constructed value against `test.retirement.model`.
fn test_migration_retired_stems() -> Vec<String> {
    let mut stems: Vec<String> = corpus_dag_files()
        .into_iter()
        .filter(|(_, content)| content.contains("TestModuleRetirement {"))
        .filter_map(|(path, _)| {
            let file_name = std::path::Path::new(&path)
                .file_name()
                .and_then(|n| n.to_str())?;
            file_name
                .strip_suffix("_retired.dag")
                .map(|s| s.to_string())
        })
        .collect();
    stems.sort();
    stems.dedup();
    stems
}

// The covered set consumed by both the debt roster and the delete-guard: floor-witness stems
// (migrate path) unioned with retired stems (delete path). One union, two consumers.
fn test_migration_covered_stems() -> Vec<String> {
    let mut stems = test_migration_debt_floor_stems();
    stems.extend(test_migration_retired_stems());
    stems.sort();
    stems.dedup();
    stems
}

fn build_test_migration_debt_report() -> TestMigrationDebtReport {
    let dir = test_migration_debt_v1_test_dir();
    let floor_stems = test_migration_covered_stems();
    let mut entries = Vec::new();
    let read_dir = match std::fs::read_dir(&dir) {
        Ok(rd) => rd,
        Err(_) => return TestMigrationDebtReport { entries },
    };
    let mut paths: Vec<std::path::PathBuf> = read_dir
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map(|e| e == "rs").unwrap_or(false))
        .collect();
    paths.sort();
    for path in paths {
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        // Line-anchored so `#[test]` mentioned in a comment/string/doc example doesn't inflate
        // the count (a `content.matches` substring scan would).
        let test_fn_count = content
            .lines()
            .filter(|line| line.trim() == "#[test]")
            .count() as i64;
        if test_fn_count == 0 {
            continue;
        }
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default()
            .to_string();
        let stem = test_migration_debt_stem(&file_name);
        if test_migration_debt_stem_covered(&stem, &floor_stems) {
            continue;
        }
        entries.push(TestMigrationDebtEntry {
            module: file_name,
            loc: content.lines().count() as i64,
            test_fn_count,
        });
    }
    TestMigrationDebtReport { entries }
}

fn test_migration_debt_report() -> &'static TestMigrationDebtReport {
    static REPORT: OnceLock<TestMigrationDebtReport> = OnceLock::new();
    REPORT.get_or_init(build_test_migration_debt_report)
}

pub fn test_migration_debt_module_count() -> i64 {
    test_migration_debt_report().entries.len() as i64
}

pub fn test_migration_debt_total_loc() -> i64 {
    test_migration_debt_report()
        .entries
        .iter()
        .map(|e| e.loc)
        .sum()
}

pub fn test_migration_debt_total_test_fns() -> i64 {
    test_migration_debt_report()
        .entries
        .iter()
        .map(|e| e.test_fn_count)
        .sum()
}

pub fn test_migration_debt_module_names() -> Vec<String> {
    test_migration_debt_report()
        .entries
        .iter()
        .map(|e| e.module.clone())
        .collect()
}

// Discriminating red witness for the stem matcher: `witness_option_bridge_test.rs` has a live
// floor counterpart (`witness_option_bridge_test.dag`) and must NOT appear in the debt roster.
// This goes red if the matcher regresses to comparing an un-stripped `.dag` suffix against a
// stripped `.rs` stem (as it did before this function existed), since every module would then
// spuriously report as debt.
pub fn test_migration_debt_known_covered_module_is_not_debt() -> bool {
    !test_migration_debt_module_names()
        .iter()
        .any(|m| m == "witness_option_bridge_test.rs")
}

// §5 hard gate per module at delete time: any `#[test]`-bearing v1 module deleted in the CI
// diff must already have an exact-stem floor `*_test.dag` witness on HEAD (same stem rule as the
// live debt roster). Uses the same `GUNBC_CI_DIFF_*` endpoints as `floor_diff_observe`.
fn test_migration_delete_guard_diff_endpoints() -> (String, String) {
    let base = std::env::var("GUNBC_CI_DIFF_BASE").unwrap_or_else(|_| "origin/main".to_string());
    let head = std::env::var("GUNBC_CI_DIFF_HEAD").unwrap_or_else(|_| "HEAD".to_string());
    (base, head)
}

fn test_migration_delete_guard_merge_base_mode() -> bool {
    match std::env::var("GUNBC_CI_DIFF_MERGE_BASE") {
        Ok(v) => v != "0" && v != "false",
        Err(_) => true,
    }
}

fn test_migration_delete_guard_run_git(args: &[&str]) -> Result<String, String> {
    let output = std::process::Command::new("git")
        .args(args)
        .current_dir(workspace_root())
        .output()
        .map_err(|e| format!("git {args:?}: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "git {args:?} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn test_migration_v1_test_module_had_line_anchored_tests(content: &str) -> bool {
    content.lines().any(|line| line.trim() == "#[test]")
}

fn test_migration_delete_guard_deleted_v1_test_paths(
    base: &str,
    head: &str,
) -> Result<Vec<String>, String> {
    let out = if test_migration_delete_guard_merge_base_mode() {
        let range = format!("{base}...{head}");
        test_migration_delete_guard_run_git(&["diff", "--name-only", "--diff-filter=D", &range])?
    } else {
        test_migration_delete_guard_run_git(&[
            "diff",
            "--name-only",
            "--diff-filter=D",
            base,
            head,
        ])?
    };
    Ok(out
        .lines()
        .map(normalize_repo_path)
        .filter(|p| {
            p.starts_with("src/v1/tests/src/") && p.ends_with(".rs") && !p.ends_with("/lib.rs")
        })
        .collect())
}

fn test_migration_delete_guard_resolve_rev(r#ref: &str) -> Result<String, String> {
    match test_migration_delete_guard_run_git(&["rev-parse", r#ref]) {
        Ok(v) => Ok(v),
        Err(e) => {
            if r#ref == "origin/main" {
                test_migration_delete_guard_run_git(&["rev-parse", "main"]).or(Err(e))
            } else {
                Err(e)
            }
        }
    }
}

fn test_migration_delete_guard_uncovered_deletes_inner() -> Result<Vec<String>, String> {
    let (base, head) = test_migration_delete_guard_diff_endpoints();
    let ci_diff_configured = std::env::var("GUNBC_CI_DIFF_BASE").is_ok();
    let base_rev = match test_migration_delete_guard_resolve_rev(&base) {
        Ok(v) => v,
        Err(_) if !ci_diff_configured => return Ok(Vec::new()),
        Err(e) => return Err(e),
    };
    let head_rev = match test_migration_delete_guard_resolve_rev(&head) {
        Ok(v) => v,
        Err(_) if !ci_diff_configured => return Ok(Vec::new()),
        Err(e) => return Err(e),
    };
    if base_rev == head_rev {
        return Ok(Vec::new());
    }
    let floor_stems = test_migration_covered_stems();
    let deleted = test_migration_delete_guard_deleted_v1_test_paths(&base, &head)?;
    let mut violations = Vec::new();
    for path in deleted {
        let content = test_migration_delete_guard_run_git(&["show", &format!("{base}:{path}")])?;
        if !test_migration_v1_test_module_had_line_anchored_tests(&content) {
            continue;
        }
        let file_name = std::path::Path::new(&path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();
        let stem = test_migration_debt_stem(file_name);
        if !test_migration_debt_stem_covered(&stem, &floor_stems) {
            violations.push(path);
        }
    }
    violations.sort();
    violations.dedup();
    Ok(violations)
}

pub fn test_migration_delete_guard_uncovered_deletes() -> Vec<String> {
    test_migration_delete_guard_uncovered_deletes_inner().unwrap_or_default()
}

pub fn test_migration_delete_guard_holds() -> bool {
    match test_migration_delete_guard_uncovered_deletes_inner() {
        Ok(violations) => violations.is_empty(),
        Err(_) => false,
    }
}

#[cfg(test)]
mod witness_layer_roots_compile_clean_tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_TEST_LOCK: Mutex<()> = Mutex::new(());

    fn with_env_test_lock<F: FnOnce()>(f: F) {
        let _guard = ENV_TEST_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        disable_floor_compile_clean_lazy_install_for_test();
        f();
    }

    struct EnvGuard {
        key: &'static str,
        prior: Option<String>,
    }
    impl EnvGuard {
        fn set(key: &'static str, value: &str) -> Self {
            let prior = std::env::var(key).ok();
            // SAFETY: `with_env_test_lock` serializes env mutation across parallel tests.
            unsafe { std::env::set_var(key, value) };
            Self { key, prior }
        }
        fn remove(key: &'static str) -> Self {
            let prior = std::env::var(key).ok();
            // SAFETY: `with_env_test_lock` serializes env mutation across parallel tests.
            unsafe { std::env::remove_var(key) };
            Self { key, prior }
        }
    }
    impl Drop for EnvGuard {
        fn drop(&mut self) {
            unsafe {
                match &self.prior {
                    Some(v) => std::env::set_var(self.key, v),
                    None => std::env::remove_var(self.key),
                }
            }
        }
    }

    fn with_workspace_cwd<F: FnOnce()>(f: F) {
        let ws = workspace_root();
        let prior = std::env::current_dir().ok();
        std::env::set_current_dir(&ws).expect("chdir workspace");
        f();
        if let Some(p) = prior {
            let _ = std::env::set_current_dir(p);
        }
    }

    /// Hand-Rust receipt: the emit leg is a strict superset of resolve for the same sources.
    #[test]
    fn emit_success_implies_resolve_success_on_live_witness_roots() {
        with_env_test_lock(|| {
            // Whole-tree path only: this receipt is about emit⊇resolve, not lever-a scoping.
            // In CI `GITHUB_ACTIONS=true` would route through the live shard-roster disposition.
            let _ga = EnvGuard::remove("GITHUB_ACTIONS");
            let _base = EnvGuard::remove("GUNBC_CI_DIFF_BASE");
            if witness_layer_roots_compile_clean_emit_check() {
                assert!(witness_layer_roots_compile_clean_check());
            }
        });
    }

    /// §5 discriminating RED: gate without an installed receipt must refuse (never run a second compile).
    #[test]
    fn floor_compile_clean_gate_refuses_without_receipt() {
        with_env_test_lock(|| {
            reset_floor_compile_clean_receipt_for_test();
            assert!(
                !consume_floor_compile_clean_gate_verdict(),
                "gate must refuse when no in-run compile receipt exists"
            );
            assert!(!floor_compile_clean_receipt_installed());
        });
    }

    /// §5 discriminating RED: gate must refuse when the one compile produced hard errors.
    #[test]
    fn floor_compile_clean_gate_refuses_on_failed_compile_receipt() {
        with_env_test_lock(|| {
            reset_floor_compile_clean_receipt_for_test();
            install_floor_compile_clean_receipt_fixture(FloorCompileCleanReceipt::Compiled {
                ok: false,
            });
            assert!(
                !consume_floor_compile_clean_gate_verdict(),
                "gate must refuse when the installed receipt records compile failure"
            );
        });
    }

    /// §5 discriminating RED (end-to-end): real whole-tree compile with an injected broken module
    /// must refuse through install_floor_compile_clean_receipt → consume_floor_compile_clean_gate_verdict.
    /// Ignored in CI: ~minutes cold whole-tree compile; recorded execution receipt in PR #6361 body.
    #[test]
    #[ignore = "manual ~minutes whole-tree compile; recorded execution receipt in PR #6361 body (clever-koi demand 1)"]
    fn floor_compile_clean_gate_e2e_refuses_on_broken_tree() {
        with_env_test_lock(|| {
            with_workspace_cwd(|| {
                let _ga = EnvGuard::remove("GITHUB_ACTIONS");
                let _base = EnvGuard::remove("GUNBC_CI_DIFF_BASE");
                let _inject =
                    EnvGuard::set("GUNBC_TEST_FLOOR_COMPILE_CLEAN_INJECT_UNRESOLVED", "1");
                reset_floor_compile_clean_receipt_for_test();
                // The receipt compile requires armed index roots (it rides the shared
                // process index); without them install records a typed Refused receipt
                // and this test would pass WITHOUT compiling — a masked tooth.
                enable_floor_compile_clean_lazy_install(&["dag".to_string(), "src/v2".to_string()]);
                install_floor_compile_clean_receipt()
                    .expect("real whole-tree compile with injected unresolved import");
                assert!(
                    !consume_floor_compile_clean_gate_verdict(),
                    "gate must refuse when the one real compile hits hard errors"
                );
            });
        });
    }

    /// widen-never-narrow box (lively-raven-355): the dag/ entry closure has zero live `import v1.*`
    /// lines — src/v1 is a shell-perturb resolution root only, not gate entry scope.
    #[test]
    fn compile_clean_dag_entry_tree_has_no_v1_module_imports() {
        let dag_root = workspace_root().join("dag");
        let mut offenders = Vec::new();
        let mut stack = vec![dag_root];
        while let Some(dir) = stack.pop() {
            let entries =
                std::fs::read_dir(&dir).unwrap_or_else(|e| panic!("read_dir {:?}: {e}", dir));
            for entry in entries {
                let entry = entry.unwrap();
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                } else if path.extension().and_then(|s| s.to_str()) == Some("dag") {
                    let content = std::fs::read_to_string(&path)
                        .unwrap_or_else(|e| panic!("read {:?}: {e}", path));
                    for (i, line) in content.lines().enumerate() {
                        let trimmed = line.trim_start();
                        if trimmed.starts_with("import v1.") {
                            offenders.push(format!("{}:{}: {}", path.display(), i + 1, trimmed));
                        }
                    }
                }
            }
        }
        assert!(
            offenders.is_empty(),
            "dag compile-clean entry tree must not import v1.* modules: {offenders:?}"
        );
    }

    /// Lever-a receipt: docs-only compile-clean scope skips at path grain.
    #[test]
    fn floor_fast_scoped_plan_skips_docs_only_touch() {
        with_workspace_cwd(|| {
            let plan = compile_clean_scope_plan_from_touched_paths_floor_fast(
                &["docs/plans/example.md".to_string()],
                &HashSet::new(),
            );
            assert_eq!(
                plan,
                CompileCleanScopePlan::SkipNoAffected {
                    reason: "docs-only diff — no compile-clean entry selection required (Ruling 1 path grain)".to_string(),
                }
            );
        });
    }

    /// Floor admission: docs-only skips before substrate warm (witness runs pre-executor).
    #[test]
    fn documentation_only_floor_skip_label_skips_docs_only_touch() {
        with_workspace_cwd(|| {
            let _gh = EnvGuard::set("GITHUB_ACTIONS", "true");
            let _ns = EnvGuard::set(
                "GUNBC_CI_DIFF_NAME_STATUS",
                "M\\000docs/plans/example.md\\000",
            );
            let label = documentation_only_floor_skip_label_for_ci();
            assert_eq!(label, DOCUMENTATION_ONLY_FLOOR_SKIP_LABEL);
        });
    }

    /// Selectable-universe guard: a mixed diff carrying a non-.dag non-docs path (compiler
    /// seed `.rs`) keeps the whole-tree baseline even though a `.dag` path also intersects.
    #[test]
    fn floor_fast_plan_whole_tree_on_mixed_rs_and_dag_touch() {
        with_workspace_cwd(|| {
            let plan = compile_clean_scope_plan_from_touched_paths_floor_fast(
                &[
                    "dag/tools/dag_compile_clean_transport.dag".to_string(),
                    "src/v1/stage0/src/cli_run.rs".to_string(),
                ],
                &HashSet::new(),
            );
            assert_eq!(plan, CompileCleanScopePlan::WholeTree);
        });
    }

    /// Departed-path guard: a deleted/renamed-from non-docs path forces the whole-tree
    /// baseline (current-tree adjacency cannot see the broken importers of a deleted module).
    #[test]
    fn floor_fast_plan_whole_tree_on_departed_dag_path() {
        with_workspace_cwd(|| {
            let departed: HashSet<String> = ["dag/std/logic.dag".to_string()].into_iter().collect();
            let plan = compile_clean_scope_plan_from_touched_paths_floor_fast(
                &["dag/std/logic.dag".to_string()],
                &departed,
            );
            assert_eq!(plan, CompileCleanScopePlan::WholeTree);
        });
    }

    /// Docs-only departure stays a skip — the departed guard fires only outside docs/**.
    #[test]
    fn floor_fast_plan_docs_only_departure_still_skips() {
        with_workspace_cwd(|| {
            let departed: HashSet<String> =
                ["docs/plans/example.md".to_string()].into_iter().collect();
            let plan = compile_clean_scope_plan_from_touched_paths_floor_fast(
                &["docs/plans/example.md".to_string()],
                &departed,
            );
            assert!(
                matches!(plan, CompileCleanScopePlan::SkipNoAffected { .. }),
                "expected SkipNoAffected, got {plan:?}"
            );
        });
    }

    /// Regen departed-path guard: a deleted `.dag` path must run regen even when the
    /// current-tree closure no longer contains it (the closure is computed from the
    /// current tree, so deletions are invisible to the intersection test). The control
    /// below proves the same path as a modification outside the closure still skips.
    #[test]
    fn regen_floor_skip_runs_on_departed_dag_path() {
        with_env_test_lock(|| {
            with_workspace_cwd(|| {
                let _ns = EnvGuard::set(
                    "GUNBC_CI_DIFF_NAME_STATUS",
                    "D\\000src/v2/lens/machine_shape.dag\\000",
                );
                assert_eq!(regen_floor_skip_label_for_ci(), RUN_REGEN_LABEL);
            });
        });
    }

    /// Control for the departed guard: the same non-closure path as a plain
    /// modification keeps the skip arm (proves the guard discriminates on D, not path).
    #[test]
    fn regen_floor_skip_skips_on_modified_non_closure_path() {
        with_env_test_lock(|| {
            with_workspace_cwd(|| {
                let _ns = EnvGuard::set(
                    "GUNBC_CI_DIFF_NAME_STATUS",
                    "M\\000src/v2/lens/machine_shape.dag\\000",
                );
                assert_eq!(
                    regen_floor_skip_label_for_ci(),
                    REGEN_NOT_AFFECTED_SKIP_LABEL
                );
            });
        });
    }

    /// The unblocked scoped arm, by execution: a single touched dag entry selects at least
    /// itself through the import-closure grain (the discriminating RED for this arm is
    /// `floor_fast_plan_whole_tree_on_mixed_rs_and_dag_touch` — same touch set plus an `.rs`
    /// path flips the disposition to whole-tree).
    #[test]
    fn floor_fast_plan_scopes_touched_dag_entry_via_import_closure() {
        with_workspace_cwd(|| {
            let plan = compile_clean_scope_plan_from_touched_paths_floor_fast(
                &["dag/std/logic.dag".to_string()],
                &HashSet::new(),
            );
            match plan {
                CompileCleanScopePlan::Scoped { entry_paths } => {
                    assert!(
                        entry_paths.iter().any(|p| p == "dag/std/logic.dag"),
                        "expected dag/std/logic.dag in {entry_paths:?}"
                    );
                    let roster_len = compile_clean_shard_entry_paths_fast().len();
                    assert!(
                        entry_paths.len() < roster_len,
                        "scoped selection must be a strict subset of the roster ({} vs {roster_len})",
                        entry_paths.len()
                    );
                }
                other => panic!("expected ScopedRun, got {other:?}"),
            }
        });
    }

    /// Discriminating receipt for the strict-tier reference-edge wiring
    /// (`build_module_graph_facts_live_uncached`), and the RED control for the deleted widen arm.
    ///
    /// `base64_rfc4648_witness_test.dag` is import-LESS: before the wiring its adjacency was empty,
    /// so the old arm answered "affected" for every touch in the repo and BOTH arms below were
    /// `true` — the assertion that matters is the second one, which can only pass once the entry
    /// has real derived edges AND the widen is gone. It fails in three distinguishable ways: no
    /// edges (wiring absent), always-true (widen still present), always-false (edges wrong).
    #[test]
    fn edgeless_entry_selects_on_its_dependency_and_skips_unrelated_touch() {
        with_workspace_cwd(|| {
            let entry = "dag/test/claim/base64_rfc4648_witness_test.dag";
            let roots = default_source_roots();
            let facts = build_module_graph_facts_live(&roots);
            let declared: HashSet<String> = facts.declared_paths.clone();

            // Wiring receipt: the entry declares no imports, so a non-empty adjacency here is
            // reference-derived by construction.
            let targets = facts
                .selection_adjacency
                .get(entry)
                .cloned()
                .unwrap_or_default();
            assert!(
                !targets.is_empty(),
                "expected reference-derived edges for import-less {entry}; empty adjacency means \
                 the strict-tier union did not land"
            );

            // MUST SELECT: `std.encoding` is a real dependency (the witness calls base64_encode).
            let on_dep = entry_file_touched_via_import_closure(
                entry,
                &facts,
                &declared,
                &["dag/std/encoding.dag".to_string()],
            )
            .expect("selection must not refuse for a parseable entry");
            assert!(
                on_dep,
                "{entry} must be selected when dag/std/encoding.dag is touched"
            );

            // MUST SKIP: a tool module that is not in this witness's closure. Under the deleted
            // widen arm this returned true, which is exactly the imprecision that made every PR
            // run the whole corpus.
            let on_unrelated = entry_file_touched_via_import_closure(
                entry,
                &facts,
                &declared,
                &["dag/gunbc/tools/review_codex.dag".to_string()],
            )
            .expect("selection must not refuse for a parseable entry");
            assert!(
                !on_unrelated,
                "{entry} must NOT be selected by an unrelated touch — this is the widen arm's RED"
            );
        });
    }

    /// Population receipt for the same wiring, and the control that IS discriminating for the
    /// deleted widen arm.
    ///
    /// The single-entry test above is NOT: once the union lands, an entry that gained edges never
    /// reaches the edgeless arm, so restoring the widen leaves it green (verified — it passed with
    /// the arm restored). The arm only ever fired for entries that stay edgeless, so the RED has to
    /// be taken there. This asserts on that residual directly, and asserts the residual is small,
    /// which is the receipt that the union actually covered the corpus rather than a lucky entry.
    #[test]
    fn edgeless_residual_is_small_and_selects_precisely() {
        with_workspace_cwd(|| {
            let roots = default_source_roots();
            let facts = build_module_graph_facts_live(&roots);
            let declared: HashSet<String> = facts.declared_paths.clone();

            let claim_entries: Vec<String> = facts
                .nodes
                .iter()
                .map(|n| workspace_relative_repo_path(&n.path))
                .filter(|p| p.ends_with("_test.dag") || p.contains("/test/claim/"))
                .collect();
            let edgeless: Vec<String> = claim_entries
                .iter()
                .filter(|p| {
                    facts
                        .selection_adjacency
                        .get(*p)
                        .is_none_or(|targets| targets.is_empty())
                })
                .cloned()
                .collect();

            eprintln!(
                "[edge-wiring receipt] claim entries={} edgeless-after-union={}",
                claim_entries.len(),
                edgeless.len()
            );
            assert!(
                edgeless.len() * 4 < claim_entries.len(),
                "edgeless residual {} of {} claim entries is too large — before the strict-tier \
                 union this was the majority, and a regression here silently restores whole-corpus \
                 selection: {:?}",
                edgeless.len(),
                claim_entries.len(),
                edgeless.iter().take(10).collect::<Vec<_>>()
            );

            // THE discriminating arm. An edgeless entry has closure {self}, so an unrelated touch
            // must not select it. The deleted widen returned `true` here unconditionally.
            if let Some(entry) = edgeless.first() {
                let on_unrelated = entry_file_touched_via_import_closure(
                    entry,
                    &facts,
                    &declared,
                    &["dag/gunbc/tools/review_codex.dag".to_string()],
                )
                .expect("an accounted edgeless entry must not refuse");
                assert!(
                    !on_unrelated,
                    "edgeless entry {entry} must NOT be selected by an unrelated touch — this \
                     assertion is what the widen arm made unconditionally true"
                );
                let on_self = entry_file_touched_via_import_closure(
                    entry,
                    &facts,
                    &declared,
                    &[entry.clone()],
                )
                .expect("an accounted edgeless entry must not refuse");
                assert!(
                    on_self,
                    "edgeless entry {entry} must still be selected when its OWN file is touched — \
                     the closure seeds with the entry, so this is the precise answer the widen \
                     was standing in for"
                );
            }
        });
    }

    /// Lever-a receipt: docs-only touched paths skip compile-clean (no affected dag entries).
    #[test]
    #[ignore = "manual: compile_clean_shard_entry_paths live scan ~minutes cold; witness in dag/test/claim/dag_compile_clean_scope_witness_test.dag"]
    fn scoped_plan_skips_docs_only_touch() {
        with_workspace_cwd(|| {
            let plan = compile_clean_scope_plan_from_touched_paths(
                &["docs/plans/example.md".to_string()],
                &HashSet::new(),
            )
            .expect("scope disposition");
            assert_eq!(
                plan,
                CompileCleanScopePlan::SkipNoAffected {
                    reason: "no compile-clean entry import-closure intersects touched paths"
                        .to_string()
                }
            );
        });
    }

    /// Lever-a PR touch receipt: dag transport edits scope (not silent whole-tree).
    #[test]
    #[ignore = "manual: compile_clean_shard_entry_paths live scan ~minutes cold"]
    fn scoped_plan_includes_lever_a_dag_transport_touch() {
        with_workspace_cwd(|| {
            let plan = compile_clean_scope_plan_from_touched_paths(
                &[
                    "dag/tools/dag_compile_clean_transport.dag".to_string(),
                    "src/v1/stage0/src/cli_run.rs".to_string(),
                ],
                &HashSet::new(),
            )
            .expect("scope disposition");
            match plan {
                CompileCleanScopePlan::Scoped { entry_paths } => {
                    assert!(
                        entry_paths
                            .iter()
                            .any(|p| p.contains("dag_compile_clean_transport")),
                        "expected transport entry in {entry_paths:?}"
                    );
                }
                other => panic!("expected ScopedRun for lever-A PR touch, got {other:?}"),
            }
        });
    }

    /// Lever-a receipt: a direct dag entry touch scopes to at least that entry.
    #[test]
    #[ignore = "manual: compile_clean_shard_entry_paths live scan ~minutes cold; witness in dag/test/claim/dag_compile_clean_scope_witness_test.dag"]
    fn scoped_plan_includes_touched_dag_entry() {
        with_workspace_cwd(|| {
            let plan = compile_clean_scope_plan_from_touched_paths(
                &["dag/std/logic.dag".to_string()],
                &HashSet::new(),
            )
            .expect("scope disposition");
            match plan {
                CompileCleanScopePlan::Scoped { entry_paths } => {
                    assert!(
                        entry_paths.iter().any(|p| p == "dag/std/logic.dag"),
                        "expected dag/std/logic.dag in {entry_paths:?}"
                    );
                }
                other => panic!("expected ScopedRun, got {other:?}"),
            }
        });
    }

    /// Lever-a receipt: diff observation failure refuses — never widens to whole-tree.
    #[test]
    fn scoped_plan_refuses_on_invalid_diff_base() {
        with_env_test_lock(|| {
            let _base = EnvGuard::set("GUNBC_CI_DIFF_BASE", "__gunbc_invalid_diff_base__");
            let _head = EnvGuard::set("GUNBC_CI_DIFF_HEAD", "HEAD");
            let plan = compile_clean_scope_plan_for_ci();
            assert!(
                matches!(plan, CompileCleanScopePlan::Refused { .. }),
                "expected Refused on diff failure, got {plan:?}"
            );
        });
    }

    /// Lever-a receipt: `GITHUB_ACTIONS=true` activates scoping (same signal as
    /// `floor_diff_observe` / `install_group_syntax`) without requiring `GUNBC_CI_DIFF_BASE`.
    /// Disposition soundness (not `WholeTree`) is covered by `.dag` witnesses — calling
    /// `compile_clean_scope_plan_for_ci` here would run the live shard-roster scan.
    #[test]
    fn github_actions_activates_compile_clean_scoping() {
        with_env_test_lock(|| {
            let _ga = EnvGuard::set("GITHUB_ACTIONS", "true");
            let _base = EnvGuard::remove("GUNBC_CI_DIFF_BASE");
            assert!(compile_clean_scoping_active());
        });
    }

    /// Falsifier cold-control arm: the env forces WholeTree before any diff observation
    /// (widen-to-more-checking only — it can never skip or narrow the gate).
    #[test]
    fn cold_control_env_forces_whole_tree_scope_plan() {
        with_env_test_lock(|| {
            let _cc = EnvGuard::set("GUNBC_CI_COMPILE_CLEAN_COLD_CONTROL", "1");
            let _base = EnvGuard::set("GUNBC_CI_DIFF_BASE", "__gunbc_invalid_diff_base__");
            let plan = compile_clean_scope_plan_for_ci();
            assert_eq!(plan, CompileCleanScopePlan::WholeTree);
        });
    }

    /// Hand-Rust receipt: primary-precedence pool defers to the first witness root.
    #[test]
    fn primary_precedence_pool_fills_only_absent_modules() {
        let roots = witness_layer_roots();
        if roots.len() < 2 {
            return;
        }
        let strict = build_module_index(&roots);
        let pooled = build_module_index_primary_precedence(&roots);
        assert!(pooled.len() >= strict.len());
        for (module_path, source) in &strict {
            let pooled_source = pooled
                .get(module_path)
                .unwrap_or_else(|| panic!("missing pooled entry for {module_path}"));
            assert_eq!(pooled_source.path, source.path);
        }
    }
}

#[cfg(test)]
mod test_migration_debt_tests {
    use super::*;

    #[test]
    fn stem_strips_rs_and_dag_suffixes_before_test_suffix() {
        assert_eq!(
            test_migration_debt_stem("witness_option_bridge_test.rs"),
            "witness_option_bridge"
        );
        assert_eq!(
            test_migration_debt_stem("witness_option_bridge_test.dag"),
            "witness_option_bridge"
        );
        assert_ne!(
            test_migration_debt_stem("typescript_import_pipeline_test.dag"),
            "pipeline"
        );
    }

    #[test]
    fn known_covered_module_is_not_debt() {
        assert!(test_migration_debt_known_covered_module_is_not_debt());
    }

    #[test]
    fn delete_guard_holds_with_no_v1_test_deletions_in_diff() {
        assert!(test_migration_delete_guard_holds());
    }

    #[test]
    fn delete_guard_rejects_uncovered_v1_test_delete() {
        let floor_stems = test_migration_debt_floor_stems();
        let stem = test_migration_debt_stem("cron_tag_test.rs");
        assert!(!test_migration_debt_stem_covered(&stem, &floor_stems));
    }

    // Green-by-execution for the typed retirement path: the demonstrator
    // `dag/test/retirement/map_lookup_dual_dispatch_retired.dag` declares a `TestModuleRetirement`
    // whose stem is `map_lookup_dual_dispatch`. That stem is NOT a floor-witness stem (its covering
    // witness is `map_lookup_dual_dispatch_witness_test.dag`, stem `map_lookup_dual_dispatch_witness`),
    // so the retirement is the *only* thing that covers it — the union must pick it up.
    #[test]
    fn retired_stem_is_covered_but_not_a_floor_stem() {
        let stem = "map_lookup_dual_dispatch";
        assert!(
            test_migration_retired_stems().iter().any(|s| s == stem),
            "retirement declaration must contribute its stem"
        );
        // Discriminating control: the same stem is NOT a floor-witness stem — so the coverage
        // comes strictly from the retirement path, not an accidental floor match.
        assert!(
            !test_migration_debt_floor_stems().iter().any(|s| s == stem),
            "stem must be covered only via retirement, not a floor witness"
        );
        assert!(test_migration_covered_stems().iter().any(|s| s == stem));
    }

    // The retired module no longer appears in the debt roster (the retirement excluded it).
    #[test]
    fn retired_module_is_not_debt() {
        assert!(
            !test_migration_debt_module_names()
                .iter()
                .any(|m| m == "map_lookup_dual_dispatch_test.rs"),
            "a retired module must drop out of the debt roster"
        );
    }
}

// Host-fed fact extraction for `v2.lens.host_language_transport_script` — the lens `.dag` table
// owns verdict logic; this bridge only projects `shell.Exec.Run` script-arg shapes from parsed
// modules. DISSOLUTION: node-tree reader at gunbc#5364; until then one shared host seam (Chunk D).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TransportScriptArgShape {
    ComputedApplication = 0,
    BareStringLiteral = 1,
    LetBoundStringLiteral = 2,
    StringInterpLiteralsOnly = 3,
}

impl TransportScriptArgShape {
    fn as_symbol(self) -> &'static str {
        match self {
            Self::ComputedApplication => "ComputedApplication",
            Self::BareStringLiteral => "BareStringLiteral",
            Self::LetBoundStringLiteral => "LetBoundStringLiteral",
            Self::StringInterpLiteralsOnly => "StringInterpLiteralsOnly",
        }
    }
}

pub struct TransportScriptPositionFactRaw {
    pub path: String,
    pub function: String,
    pub shape: &'static str,
}

fn resolve_dag_path_for_transport_script(path: &str) -> PathBuf {
    let candidate = Path::new(path);
    if candidate.is_file() {
        return candidate.to_path_buf();
    }
    let rooted = workspace_root().join(path);
    if rooted.is_file() {
        return rooted;
    }
    panic!("transport_script_position_facts: file not found: {path}");
}

fn parse_module_items_for_transport_script(
    path: &str,
) -> (
    Rc<im::Vector<Rc<Node>>>,
    Rc<HashMap<String, Rc<NewlineIndex>>>,
) {
    let resolved = resolve_dag_path_for_transport_script(path);
    let path_str = resolved.to_string_lossy();
    let content = std::fs::read_to_string(&resolved).unwrap_or_else(|e| {
        panic!("transport_script_position_facts: failed to read {path_str}: {e}")
    });
    let filename = resolved
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(path);
    let tokens = v1_compiler_tokenize::tokenize(content.clone(), filename.to_string());
    let source_index = build_newline_index(filename.to_string(), content);
    let mut source_indices = HashMap::new();
    source_indices.insert(filename.to_string(), source_index);
    let source_indices = Rc::new(source_indices);
    let result = v1_compiler_parse::parse(tokens, source_indices.clone());
    if let Some(err) = result.error.as_ref() {
        panic!(
            "transport_script_position_facts: parse error in {path}: {}",
            diagnostic_to_message(err.diagnostic.clone())
        );
    }
    let module = result
        .module
        .as_ref()
        .expect("transport_script_position_facts: missing module");
    (module.children.clone(), source_indices)
}

fn literal_string_value_transport_script(node: &Rc<Node>) -> bool {
    matches!(
        node.expr_data.as_ref(),
        ExprData::ExprLiteral {
            value: lit,
            ..
        } if matches!(lit.as_ref(), LiteralValue::LitStr { .. })
    )
}

fn classify_transport_script_arg(
    node: &Rc<Node>,
    let_literal_bindings: &HashMap<String, bool>,
    source_indices: &Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> TransportScriptArgShape {
    if literal_string_value_transport_script(node) {
        return TransportScriptArgShape::BareStringLiteral;
    }
    match node.expr_data.as_ref() {
        ExprData::ExprStringInterp => {
            for child in node.children.iter() {
                match child.expr_data.as_ref() {
                    ExprData::ExprLiteral { value, .. } => {
                        if !matches!(value.as_ref(), LiteralValue::LitStr { .. }) {
                            return TransportScriptArgShape::ComputedApplication;
                        }
                    }
                    ExprData::ExprVar { .. } => {
                        let name = expr_var_name_at(child.clone(), source_indices.clone());
                        if !let_literal_bindings.get(&name).copied().unwrap_or(false) {
                            return TransportScriptArgShape::ComputedApplication;
                        }
                    }
                    _ => return TransportScriptArgShape::ComputedApplication,
                }
            }
            TransportScriptArgShape::StringInterpLiteralsOnly
        }
        ExprData::ExprVar { .. } => {
            let name = expr_var_name_at(node.clone(), source_indices.clone());
            if let_literal_bindings.get(&name).copied().unwrap_or(false) {
                TransportScriptArgShape::LetBoundStringLiteral
            } else {
                TransportScriptArgShape::ComputedApplication
            }
        }
        _ => TransportScriptArgShape::ComputedApplication,
    }
}

fn is_shell_exec_run_transport_script(
    node: &Rc<Node>,
    source_indices: &Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> bool {
    match node.expr_data.as_ref() {
        ExprData::ExprMethodCall { .. } => {
            if expr_method_name_at(node.clone(), source_indices.clone()) != "Run" {
                return false;
            }
            let recv = method_receiver(node.clone());
            match recv.expr_data.as_ref() {
                ExprData::ExprFieldAccess { .. } => {
                    if field_access_field_at(recv.clone(), source_indices.clone()) != "Exec" {
                        return false;
                    }
                    let base = field_access_base(recv.clone());
                    match base.expr_data.as_ref() {
                        ExprData::ExprVar { .. } => {
                            expr_var_name_at(base.clone(), source_indices.clone()) == "shell"
                        }
                        _ => false,
                    }
                }
                _ => false,
            }
        }
        _ => false,
    }
}

fn is_transport_script_from_body_call(
    node: &Rc<Node>,
    source_indices: &Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> bool {
    matches!(node.expr_data.as_ref(), ExprData::ExprCall { .. })
        && expr_call_func_at(node.clone(), source_indices.clone()) == "transport_script_from_body"
}

fn transport_script_body_arg_node(
    node: &Rc<Node>,
    source_indices: &Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> Option<Rc<Node>> {
    let args = crate::v1_compiler_infer::call_args_by_name(node.clone(), source_indices.clone());
    v1_rt::map_get(&args, "body".to_string())
}

fn effective_transport_script_source(
    script_node: &Rc<Node>,
    source_indices: &Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> Rc<Node> {
    if is_transport_script_from_body_call(script_node, source_indices) {
        transport_script_body_arg_node(script_node, source_indices)
            .unwrap_or_else(|| script_node.clone())
    } else {
        script_node.clone()
    }
}

fn transport_script_arg_node(
    node: &Rc<Node>,
    source_indices: &Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> Option<Rc<Node>> {
    for arg in method_arg_nodes(node.clone()).iter() {
        if arg_name_at(arg.clone(), source_indices.clone()).as_deref() == Some("script") {
            return Some(arg_value(arg.clone()));
        }
    }
    None
}

fn binding_is_literal_shaped_transport_script(
    node: &Rc<Node>,
    bindings: &HashMap<String, bool>,
    source_indices: &Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> bool {
    matches!(
        classify_transport_script_arg(node, bindings, source_indices),
        TransportScriptArgShape::BareStringLiteral
            | TransportScriptArgShape::LetBoundStringLiteral
            | TransportScriptArgShape::StringInterpLiteralsOnly
    )
}

fn collect_let_bindings_in_block_transport_script(
    block: &Rc<Node>,
    bindings: &mut HashMap<String, bool>,
    source_indices: &Rc<HashMap<String, Rc<NewlineIndex>>>,
) {
    for stmt in block_stmts(block.clone()).iter() {
        match stmt.expr_data.as_ref() {
            ExprData::ExprLet { .. } => {
                let name = let_binding_name_at(stmt.clone(), source_indices.clone());
                let val = let_value(stmt.clone());
                let literal_shaped =
                    binding_is_literal_shaped_transport_script(&val, bindings, source_indices);
                bindings.insert(name, literal_shaped);
            }
            _ => walk_transport_script_expr(stmt, bindings, source_indices, &mut |_| {}),
        }
    }
}

fn walk_transport_script_expr(
    node: &Rc<Node>,
    let_bindings: &HashMap<String, bool>,
    source_indices: &Rc<HashMap<String, Rc<NewlineIndex>>>,
    on_run: &mut dyn FnMut(TransportScriptArgShape),
) {
    if is_shell_exec_run_transport_script(node, source_indices) {
        if let Some(script_node) = transport_script_arg_node(node, source_indices) {
            let source = effective_transport_script_source(&script_node, source_indices);
            on_run(classify_transport_script_arg(
                &source,
                let_bindings,
                source_indices,
            ));
        }
    }
    for child in node.children.iter() {
        walk_transport_script_expr(child, let_bindings, source_indices, on_run);
    }
}

fn transport_script_facts_for_function_body(
    rel_path: &str,
    function: &str,
    body: &Rc<Node>,
    source_indices: &Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> Vec<TransportScriptPositionFactRaw> {
    let mut bindings = HashMap::new();
    if let ExprData::ExprBlock { .. } = body.expr_data.as_ref() {
        collect_let_bindings_in_block_transport_script(body, &mut bindings, source_indices);
    }
    let mut facts = Vec::new();
    walk_transport_script_expr(body, &bindings, source_indices, &mut |shape| {
        facts.push(TransportScriptPositionFactRaw {
            path: rel_path.to_string(),
            function: function.to_string(),
            shape: shape.as_symbol(),
        });
    });
    facts
}

pub fn transport_script_position_facts_for_path(
    path: String,
) -> Vec<TransportScriptPositionFactRaw> {
    let (items, source_indices) = parse_module_items_for_transport_script(&path);
    let mut facts = Vec::new();
    for item in items.iter() {
        let kind = item_kind(item.clone());
        if !matches!(kind, ItemKind::FuncItem | ItemKind::FnItem) {
            continue;
        }
        let Some(body) = item.body.as_ref() else {
            continue;
        };
        facts.extend(transport_script_facts_for_function_body(
            &path,
            &item.name,
            body,
            &source_indices,
        ));
    }
    facts
}

#[cfg(test)]
mod transport_script_peel_tests {
    use super::*;

    #[test]
    fn bare_literal_plant_detects_one_violation_through_transport_script_from_body() {
        let facts = transport_script_position_facts_for_path(
            "src/v2/test/fixture/transport_script_scan/bare_string_literal/plant.dag".to_string(),
        );
        assert_eq!(facts.len(), 1);
        assert_eq!(facts[0].shape, "BareStringLiteral");
    }

    #[test]
    fn let_bound_literal_plant_detects_one_violation_through_transport_script_from_body() {
        let facts = transport_script_position_facts_for_path(
            "src/v2/test/fixture/transport_script_scan/let_bound_literal/plant.dag".to_string(),
        );
        assert_eq!(facts.len(), 1);
        assert_eq!(facts[0].shape, "LetBoundStringLiteral");
    }
}

#[cfg(test)]
mod module_path_index_tests {
    use super::*;

    #[test]
    fn cargo_build_resolves_by_module_path_not_directory_nickname() {
        let path = source_path_for_module_path("extdeps.cargo_build".to_string());
        assert_eq!(path, "dag/extdeps/rust/cargo_build.dag");
    }

    #[test]
    fn git_module_resolves() {
        let path = source_path_for_module_path("extdeps.git".to_string());
        assert_eq!(path, "dag/extdeps/git/git.dag");
    }

    #[test]
    fn extdeps_shell_resolves_to_the_dag_authority() {
        let path = source_path_for_module_path("extdeps.shell".to_string());
        assert_eq!(path, "dag/extdeps/shell/shell.dag");
    }

    #[test]
    fn duplicate_module_path_across_roots_refuses_loudly() {
        let dir = std::env::temp_dir().join(format!(
            "gunbc-module-collision-wall-{}",
            std::process::id()
        ));
        let root_a = dir.join("root_a");
        let root_b = dir.join("root_b");
        std::fs::create_dir_all(&root_a).unwrap();
        std::fs::create_dir_all(&root_b).unwrap();
        std::fs::write(root_a.join("m.dag"), "module collision.example\n").unwrap();
        std::fs::write(root_b.join("m.dag"), "module collision.example\n").unwrap();
        let roots = vec![
            root_a.to_string_lossy().into_owned(),
            root_b.to_string_lossy().into_owned(),
        ];
        // RED control: same module declared in two files refuses loudly.
        let refused = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            super::build_module_index(&roots)
        }))
        .is_err();
        // GREEN control: distinct modules build fine.
        std::fs::write(root_b.join("m.dag"), "module collision.other\n").unwrap();
        let built = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            super::build_module_index(&roots)
        }))
        .is_ok();
        std::fs::remove_dir_all(&dir).ok();
        assert!(refused, "collision must refuse loudly, not shadow silently");
        assert!(built, "distinct modules must still index");
    }

    #[test]
    fn cargo_target_dir_output_never_enters_the_module_index() {
        let dir =
            std::env::temp_dir().join(format!("gunbc-target-dir-exclusion-{}", std::process::id()));
        let root = dir.join("root");
        let baseline = root.join("target").join("baseline_corpus");
        std::fs::create_dir_all(&baseline).unwrap();
        std::fs::write(root.join("m.dag"), "module corpus.example\n").unwrap();
        std::fs::write(baseline.join("m.dag"), "module corpus.example\n").unwrap();
        let roots = vec![root.to_string_lossy().into_owned()];
        // With a Cargo.toml beside it, target/ is build output: the corpus
        // copy is skipped and the source file indexes alone (the CI regression:
        // target/func_env_semantic_baseline_corpus tripped the collision wall).
        std::fs::write(root.join("Cargo.toml"), "[package]\n").unwrap();
        let indexed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            super::build_module_index(&roots)
        }));
        // RED control: without Cargo.toml the same layout is two source files
        // declaring one module — the wall must still refuse.
        std::fs::remove_file(root.join("Cargo.toml")).unwrap();
        let refused = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            super::build_module_index(&roots)
        }))
        .is_err();
        std::fs::remove_dir_all(&dir).ok();
        let index = indexed.expect("cargo target output must be excluded, not collide");
        assert!(
            index.contains_key("corpus.example"),
            "the source-tree declaration must still index"
        );
        assert!(
            refused,
            "a plain (non-cargo) target dir is source like any other — collision must refuse"
        );
    }

    #[test]
    fn reader_follows_synthetic_authority_with_nondefault_roots() {
        let synthetic = "module gunbc.ci_layer_roots\n\n\
             data witness_layer_roots: List<String> = [\"r_one\", \"r_two\", \"r_three\"]\n";
        assert_eq!(
            witness_layer_roots_from_source(synthetic),
            vec![
                "r_one".to_string(),
                "r_two".to_string(),
                "r_three".to_string()
            ],
            "the layer-roots reader must FOLLOW the authority, not a hardcoded copy"
        );
    }

    #[test]
    fn reader_projects_live_authority_value() {
        assert_eq!(
            witness_layer_roots(),
            vec!["dag".to_string(), "src/v2".to_string()],
            "live authority value drifted from the expected [dag, src/v2]"
        );
        assert!(
            census_corpus_roots_follow_layer_authority(),
            "census corpus roots must derive from the layer-roots authority"
        );
    }

    #[test]
    fn workspace_root_prefers_process_cwd_anchor() {
        let ws = workspace_root();
        assert!(ws.join("Cargo.toml").is_file());
        assert!(ws.join("dag").is_dir());
        assert_eq!(
            default_source_roots(),
            vec![
                ws.join("dag").to_string_lossy().into_owned(),
                ws.join("src/v2").to_string_lossy().into_owned(),
            ]
        );
    }

    #[test]
    fn build_module_path_index_accepts_relative_roots_at_process_cwd() {
        let ws = workspace_root();
        let rel_roots = vec!["dag".to_string(), "src/v2".to_string()];
        let index = build_module_path_index(&rel_roots);
        let sample = index
            .get("gunbc.ci_layer_roots")
            .expect("gunbc.ci_layer_roots must be indexed from relative roots");
        assert_eq!(sample, "dag/gunbc/ci_layer_roots.dag");
        assert!(
            ws.join(sample).is_file(),
            "indexed rel path must resolve under workspace_root()"
        );
    }

    #[test]
    fn reader_follows_synthetic_authority_scan_dirs() {
        let synthetic = "module gunbc.ci_layer_roots\n\n\
             data witness_discovery_scan_dirs: List<String> = [\"scan/a\", \"scan/b\"]\n";
        assert_eq!(
            witness_discovery_scan_dirs_from_source(synthetic),
            vec!["scan/a".to_string(), "scan/b".to_string()],
            "the scan-dir reader must FOLLOW the authority, not a hardcoded copy"
        );
    }

    #[test]
    fn witness_discovery_scan_dirs_projects_live_authority_value() {
        assert_eq!(
            witness_discovery_scan_dirs(),
            vec![
                "dag/test/claim".to_string(),
                "src/v2/test/claim/manual".to_string(),
                "src/v2/test/claim/emit".to_string(),
            ],
            "live authority scan-dir value drifted"
        );
    }

    #[test]
    fn strip_blanks_string_interior_and_drops_comment() {
        let got = strip_line_comment("data u = \"https://x // y\" // real comment");
        assert!(got.starts_with("data u = \""));
        assert!(
            !got.contains("real comment"),
            "trailing // comment dropped: {got:?}"
        );
        assert!(!got.contains("https"), "string interior blanked: {got:?}");
        assert!(got.len() <= "data u = \"https://x // y\" // real comment".len());
    }

    #[test]
    fn brace_delta_ignores_braces_in_strings() {
        assert_eq!(brace_delta("fn f() {"), 1);
        assert_eq!(brace_delta("let s = \"{ { {\""), 0);
        assert_eq!(brace_delta("} // }"), -1);
    }

    #[test]
    fn is_test_dag_matches_suffix() {
        assert!(is_test_dag("src/v2/lens/x_test.dag"));
        assert!(!is_test_dag("src/v2/lens/x.dag"));
    }

    #[test]
    fn extract_top_level_decls_captures_split_brace_body() {
        let source = include_str!("../tests/fixtures/fact_cardinality_split_brace.dag");
        let decls = extract_top_level_decls(source);
        let sample = decls
            .iter()
            .find(|(name, _)| name == "split_brace_sample")
            .expect("split-brace decl must be captured");
        let expected = decl_body_hash(
            "data split_brace_sample: SplitBraceSample =\nSplitBraceSample {\n  field: \"x\"\n}\n",
        );
        assert_eq!(
            sample.1, expected,
            "split-brace body hash must include lines after the opener"
        );
    }
}

// SCAFFOLD — host-fed fact extraction for v2.lens.extdeps_shape_transport_policy (Concern A).
// Dissolution: when the Node-tree argv projection supersedes text scan (dissolve-on marker in
// extdeps_shape_transport_policy.dag construction_justification), replace this block with a
// Node-tree builtin and delete these structs. gunbc#5364 successor, Concern A lane.

pub struct ExtdepsArgvFactRaw {
    pub module_path: String,
    pub service: String,
    pub operation: String,
    pub transport_kind: &'static str,
    pub argv_index: i64,
    pub argv_token: String,
}

pub struct ExtdepsFusionFactRaw {
    pub module_path: String,
    pub endpoint_key: String,
    pub service_a: String,
    pub service_b: String,
}

pub struct ExtdepsInputFactRaw {
    pub module_path: String,
    pub service: String,
    pub operation: String,
    pub param_name: String,
}

pub struct ExtdepsEmbeddedFactRaw {
    pub module_path: String,
    pub data_name: String,
    pub field_name: String,
    pub literal_value: String,
}

pub struct ExtdepsShapeTransportPolicyModuleFacts {
    pub argv_facts: Vec<ExtdepsArgvFactRaw>,
    pub fusion_facts: Vec<ExtdepsFusionFactRaw>,
    pub input_facts: Vec<ExtdepsInputFactRaw>,
    pub embedded_facts: Vec<ExtdepsEmbeddedFactRaw>,
    pub source_nickname_literal_count: i64,
    pub gist_create_declares_filename_input: bool,
    pub gist_create_files_keyed_by_filename: bool,
}

pub fn parse_extdeps_module_items(
    path: &str,
) -> (
    Rc<im::Vector<Rc<crate::v1_std_core::Node>>>,
    Rc<HashMap<String, Rc<crate::v1_std_core::NewlineIndex>>>,
) {
    use crate::v1_compiler_parse::parse;
    use crate::v1_compiler_tokenize::tokenize;
    use crate::v1_std_core::build_newline_index;
    let candidate = std::path::Path::new(path);
    let resolved = if candidate.is_file() {
        candidate.to_path_buf()
    } else {
        let rooted = workspace_root().join(path);
        if rooted.is_file() {
            rooted
        } else {
            panic!("parse_extdeps_module_items: file not found: {path}");
        }
    };
    let path_str = resolved.to_string_lossy();
    let content = std::fs::read_to_string(&resolved)
        .unwrap_or_else(|e| panic!("parse_extdeps_module_items: failed to read {path_str}: {e}"));
    let filename = resolved
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(path);
    let tokens = tokenize(content.clone(), filename.to_string());
    let source_index = build_newline_index(filename.to_string(), content);
    let mut source_indices_map = HashMap::new();
    source_indices_map.insert(filename.to_string(), source_index);
    let source_indices = Rc::new(source_indices_map);
    let result = parse(tokens, source_indices.clone());
    if let Some(err) = result.error.as_ref() {
        panic!(
            "parse_extdeps_module_items: parse error in {path}: {}",
            crate::v1_std_core::diagnostic_to_message(err.diagnostic.clone())
        );
    }
    let module = result
        .module
        .as_ref()
        .expect("parse_extdeps_module_items: missing module");
    (module.children.clone(), source_indices)
}

pub fn shell_argv_nodes_for_operation(
    path: String,
    service: String,
    operation: String,
) -> (
    Rc<im::Vector<Rc<crate::v1_std_core::Node>>>,
    Rc<HashMap<String, Rc<crate::v1_std_core::NewlineIndex>>>,
) {
    let (items, source_indices) = parse_extdeps_module_items(&path);
    for item in items.iter() {
        if item.name != service {
            continue;
        }
        let fallback_transport = if let Some(t) = item.transport.as_ref() {
            t.clone()
        } else {
            crate::v1_std_core::local_transport_node(item.span.clone())
        };
        for op in item.children.iter() {
            if op.name != operation {
                continue;
            }
            let eff = crate::v1_compiler_emit::effective_operation_transport(
                op.clone(),
                fallback_transport.clone(),
            );
            return (eff.children.clone(), source_indices);
        }
    }
    panic!("shell_argv_nodes_for_operation: operation {service}.{operation} not found in {path}");
}

pub fn qualified_name_resolves_in_derived_module_set(qn: &crate::v1_interpreter::Value) -> bool {
    let module_path = free_monoid_symbol_value_to_dotted_string(qn);
    !module_path.is_empty()
        && build_module_path_index_from_witness_roots().contains_key(&module_path)
}

fn extdeps_argv_expr_token(
    node: &Rc<crate::v1_std_core::Node>,
    source_indices: &Rc<HashMap<String, Rc<crate::v1_std_core::NewlineIndex>>>,
) -> String {
    use crate::v1_std_core::{expr_var_name_at, ExprData, LiteralValue};
    match node.expr_data.as_ref() {
        ExprData::ExprLiteral { value } => match value.as_ref() {
            LiteralValue::LitStr { value } => value.clone(),
            other => format!("{other:?}"),
        },
        ExprData::ExprVar { .. } => {
            let name = expr_var_name_at(node.clone(), source_indices.clone());
            if name.is_empty() {
                node.name.clone()
            } else {
                format!("{{{name}}}")
            }
        }
        ExprData::ExprStringInterp => node
            .children
            .iter()
            .map(|child| match child.expr_data.as_ref() {
                ExprData::ExprLiteral { value } => match value.as_ref() {
                    LiteralValue::LitStr { value } => value.clone(),
                    _ => String::new(),
                },
                ExprData::ExprVar { .. } => {
                    let name = expr_var_name_at(child.clone(), source_indices.clone());
                    if name.is_empty() {
                        child.name.clone()
                    } else {
                        format!("{{{name}}}")
                    }
                }
                _ => String::new(),
            })
            .collect(),
        _ => String::new(),
    }
}

fn extdeps_literal_string_value(node: &Rc<crate::v1_std_core::Node>) -> Option<String> {
    use crate::v1_std_core::{ExprData, LiteralValue};
    match node.expr_data.as_ref() {
        ExprData::ExprLiteral { value } => match value.as_ref() {
            LiteralValue::LitStr { value } => Some(value.clone()),
            _ => None,
        },
        _ => None,
    }
}

fn extdeps_record_field_value(
    record: &Rc<crate::v1_std_core::Node>,
    field_name: &str,
    source_indices: &Rc<HashMap<String, Rc<crate::v1_std_core::NewlineIndex>>>,
) -> Option<Rc<crate::v1_std_core::Node>> {
    use crate::v1_std_core::{field_init_node_name_at, field_init_node_value, ExprData};
    if !matches!(record.expr_data.as_ref(), ExprData::ExprRecordLit { .. }) {
        return None;
    }
    for field_init in record.children.iter() {
        let name = field_init_node_name_at(field_init.clone(), source_indices.clone());
        if name == field_name {
            return Some(field_init_node_value(field_init.clone()));
        }
    }
    None
}

fn extdeps_module_source_nickname_count_in_node(
    node: &Rc<crate::v1_std_core::Node>,
    real_paths: &std::collections::HashSet<String>,
) -> i64 {
    let mut count = 0i64;
    if let Some(lit) = extdeps_literal_string_value(node) {
        if real_paths.contains(&lit) {
            count += 1;
        }
    }
    if let Some(body) = node.body.as_ref() {
        count += extdeps_module_source_nickname_count_in_node(body, real_paths);
    }
    for child in node.children.iter() {
        count += extdeps_module_source_nickname_count_in_node(child, real_paths);
    }
    for param in node.params.iter() {
        count += extdeps_module_source_nickname_count_in_node(param, real_paths);
    }
    if let Some(type_annotation) = node.type_annotation.as_ref() {
        count += extdeps_module_source_nickname_count_in_node(type_annotation, real_paths);
    }
    count
}

fn extdeps_gist_create_declares_filename_for_items(
    items: &Rc<im::Vector<Rc<crate::v1_std_core::Node>>>,
    source_indices: &Rc<HashMap<String, Rc<crate::v1_std_core::NewlineIndex>>>,
) -> bool {
    use crate::v1_std_core::param_node_name_at;
    for item in items.iter() {
        if item.name != "github.Gist" {
            continue;
        }
        for op in item.children.iter() {
            if op.name != "Create" {
                continue;
            }
            for param in op.params.iter() {
                let name = param_node_name_at(param.clone(), source_indices.clone());
                if name == "filename" {
                    return true;
                }
            }
        }
    }
    false
}

fn extdeps_gist_map_keys_use_filename(
    map_node: &Rc<crate::v1_std_core::Node>,
    source_indices: &Rc<HashMap<String, Rc<crate::v1_std_core::NewlineIndex>>>,
) -> bool {
    use crate::v1_std_core::{field_init_node_name_at, ExprData};
    if !matches!(map_node.expr_data.as_ref(), ExprData::ExprRecordLit { .. }) {
        return false;
    }
    if map_node.children.is_empty() {
        return false;
    }
    for entry in map_node.children.iter() {
        let key = field_init_node_name_at(entry.clone(), source_indices.clone());
        if !(key == "filename" || key.contains("{filename}")) {
            return false;
        }
    }
    true
}

fn extdeps_gist_create_files_keyed_by_filename_for_items(
    items: &Rc<im::Vector<Rc<crate::v1_std_core::Node>>>,
    source_indices: &Rc<HashMap<String, Rc<crate::v1_std_core::NewlineIndex>>>,
) -> bool {
    use crate::v1_std_core::{is_rest_transport, transport_request_body};
    for item in items.iter() {
        if item.name != "github.Gist" {
            continue;
        }
        for op in item.children.iter() {
            if op.name != "Create" {
                continue;
            }
            let Some(transport) = op.transport.as_ref() else {
                return false;
            };
            if !is_rest_transport(transport.clone(), source_indices.clone()) {
                return false;
            }
            let Some(body) = transport_request_body(transport.clone(), source_indices.clone())
            else {
                return false;
            };
            let Some(files) = extdeps_record_field_value(&body, "files", source_indices) else {
                return false;
            };
            return extdeps_gist_map_keys_use_filename(&files, source_indices);
        }
    }
    false
}

pub fn extdeps_shape_transport_policy_module_facts(
    module_path: &str,
) -> ExtdepsShapeTransportPolicyModuleFacts {
    use crate::v1_compiler_emit::effective_operation_transport;
    use crate::v1_compiler_emit_core_support::is_data_def_item;
    use crate::v1_std_core::{
        field_init_node_name_at, field_init_node_value, param_node_name_at, ExprData,
    };

    let path = source_path_for_module_path(module_path.to_string());
    let (items, source_indices) = parse_extdeps_module_items(&path);

    let mut argv_facts: Vec<ExtdepsArgvFactRaw> = Vec::new();
    let mut input_facts: Vec<ExtdepsInputFactRaw> = Vec::new();

    for item in items.iter() {
        if item.name.is_empty() || item.children.is_empty() {
            continue;
        }
        let fallback_transport = if let Some(t) = item.transport.as_ref() {
            t.clone()
        } else {
            crate::v1_std_core::local_transport_node(item.span.clone())
        };
        for op in item.children.iter() {
            if op.name.is_empty() {
                continue;
            }
            let eff = effective_operation_transport(op.clone(), fallback_transport.clone());
            let transport_kind =
                if crate::v1_std_core::is_rest_transport(eff.clone(), source_indices.clone()) {
                    "Rest"
                } else {
                    "Shell"
                };
            for (idx, arg) in eff.children.iter().enumerate() {
                let token = extdeps_argv_expr_token(arg, &source_indices);
                argv_facts.push(ExtdepsArgvFactRaw {
                    module_path: module_path.to_string(),
                    service: item.name.clone(),
                    operation: op.name.clone(),
                    transport_kind,
                    argv_index: idx as i64,
                    argv_token: token,
                });
            }
            for param in op.params.iter() {
                let name = param_node_name_at(param.clone(), source_indices.clone());
                if !name.is_empty() {
                    input_facts.push(ExtdepsInputFactRaw {
                        module_path: module_path.to_string(),
                        service: item.name.clone(),
                        operation: op.name.clone(),
                        param_name: name,
                    });
                }
            }
        }
    }

    let service_names: Vec<String> = items
        .iter()
        .filter(|item| !item.name.is_empty() && !item.children.is_empty())
        .map(|item| item.name.clone())
        .collect();
    let has_oauth_google = service_names.iter().any(|s| s == "oauth2.Google");
    let has_shell_oauth = service_names.iter().any(|s| s == "shell.OAuth2");
    let mut fusion_facts: Vec<ExtdepsFusionFactRaw> = Vec::new();
    if has_oauth_google && has_shell_oauth {
        fusion_facts.push(ExtdepsFusionFactRaw {
            module_path: module_path.to_string(),
            endpoint_key: "OAuth2.refresh".to_string(),
            service_a: "oauth2.Google".to_string(),
            service_b: "shell.OAuth2".to_string(),
        });
    }

    let mut embedded_facts: Vec<ExtdepsEmbeddedFactRaw> = Vec::new();
    for item in items.iter() {
        if !is_data_def_item(item.clone()) || item.name.is_empty() {
            continue;
        }
        let Some(body) = item.body.as_ref() else {
            continue;
        };
        if !matches!(body.expr_data.as_ref(), ExprData::ExprRecordLit { .. }) {
            continue;
        }
        for field_init in body.children.iter() {
            let field_name = field_init_node_name_at(field_init.clone(), source_indices.clone());
            let value_node = field_init_node_value(field_init.clone());
            if let Some(literal) = extdeps_literal_string_value(&value_node) {
                embedded_facts.push(ExtdepsEmbeddedFactRaw {
                    module_path: module_path.to_string(),
                    data_name: item.name.clone(),
                    field_name,
                    literal_value: literal,
                });
            }
        }
    }

    let index = build_module_path_index_from_witness_roots();
    let real_paths: std::collections::HashSet<String> = index.into_iter().map(|(_, v)| v).collect();
    let mut source_nickname_literal_count = 0i64;
    for item in items.iter() {
        source_nickname_literal_count +=
            extdeps_module_source_nickname_count_in_node(item, &real_paths);
    }

    let gist_create_declares_filename_input =
        extdeps_gist_create_declares_filename_for_items(&items, &source_indices);
    let gist_create_files_keyed_by_filename =
        extdeps_gist_create_files_keyed_by_filename_for_items(&items, &source_indices);

    ExtdepsShapeTransportPolicyModuleFacts {
        argv_facts,
        fusion_facts,
        input_facts,
        embedded_facts,
        source_nickname_literal_count,
        gist_create_declares_filename_input,
        gist_create_files_keyed_by_filename,
    }
}

// SCAFFOLD — host-fed fact extraction for v2.lens.extdeps_external_authority (Concern B).
// Dissolution: when Node-tree anchor projection supersedes module parse (dissolve-on marker in
// extdeps_external_authority.dag construction_justification), replace this block with a
// Node-tree builtin and delete these structs. gunbc#5364 successor, Concern B lane.

pub struct ExtdepsExternalAuthorityModuleFacts {
    pub anchor_kind: String,
    pub scheme_identity: String,
    pub locator: String,
    pub is_backfill_pending: bool,
    pub is_machinery_exempt: bool,
    pub is_clean_tree_roster_excluded: bool,
    pub anchor_shadow_masked: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ExternalAuthorityAnchorProjection {
    Absent,
    Present {
        scheme_identity: String,
        locator: String,
    },
}

fn external_authority_uri_record_from_anchor_body(
    body: &Rc<crate::v1_std_core::Node>,
    variant: &str,
    source_indices: &Rc<HashMap<String, Rc<crate::v1_std_core::NewlineIndex>>>,
) -> Option<Rc<crate::v1_std_core::Node>> {
    match variant {
        "ExternalAuthority" | "StableAuthority" | "ExternalUri" => {
            extdeps_record_field_value(body, "uri", source_indices)
        }
        _ => None,
    }
}

fn external_authority_scheme_identity_from_value_node(
    node: &Rc<crate::v1_std_core::Node>,
    source_indices: &Rc<HashMap<String, Rc<crate::v1_std_core::NewlineIndex>>>,
) -> String {
    use crate::v1_std_core::authored_name_at;
    authored_name_at(source_indices.clone(), node.clone())
}

fn read_external_authority_anchor_from_items(
    items: &Rc<im::Vector<Rc<crate::v1_std_core::Node>>>,
    source_indices: &Rc<HashMap<String, Rc<crate::v1_std_core::NewlineIndex>>>,
) -> ExternalAuthorityAnchorProjection {
    use crate::v1_compiler_emit_core_support::is_data_def_item;
    use crate::v1_std_core::authored_name_at;
    for item in items.iter() {
        if !is_data_def_item(item.clone()) || item.name != "extdeps_external_authority_anchor" {
            continue;
        }
        let Some(body) = item.body.as_ref() else {
            return ExternalAuthorityAnchorProjection::Absent;
        };
        let variant = authored_name_at(source_indices.clone(), body.clone());
        let Some(uri_node) =
            external_authority_uri_record_from_anchor_body(body, variant.as_str(), source_indices)
        else {
            return ExternalAuthorityAnchorProjection::Absent;
        };
        let scheme = extdeps_record_field_value(&uri_node, "scheme", source_indices)
            .map(|n| external_authority_scheme_identity_from_value_node(&n, source_indices))
            .unwrap_or_default();
        let locator = extdeps_record_field_value(&uri_node, "locator", source_indices)
            .and_then(|n| extdeps_literal_string_value(&n))
            .unwrap_or_default();
        if scheme.is_empty() {
            return ExternalAuthorityAnchorProjection::Absent;
        }
        return ExternalAuthorityAnchorProjection::Present {
            scheme_identity: scheme,
            locator,
        };
    }
    ExternalAuthorityAnchorProjection::Absent
}

fn project_external_authority_anchor(module_path: &str) -> ExternalAuthorityAnchorProjection {
    let path = source_path_for_module_path(module_path.to_string());
    let (items, source_indices) = parse_extdeps_module_items(&path);
    read_external_authority_anchor_from_items(&items, &source_indices)
}

fn external_authority_backfill_pending_module_paths() -> &'static std::collections::HashSet<String>
{
    use std::collections::HashSet;
    use std::sync::OnceLock;
    static PATHS: OnceLock<HashSet<String>> = OnceLock::new();
    PATHS.get_or_init(|| {
        let path = workspace_root().join("dag/extdeps/external_authority_backfill_pending.txt");
        let content = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("read backfill_pending snapshot {:?}: {e}", path));
        content
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .map(str::to_string)
            .collect()
    })
}

fn external_authority_machinery_exempt_module_paths() -> &'static [&'static str] {
    &["extdeps.uri", "extdeps.external_authority"]
}

fn external_authority_clean_tree_roster_exclusion_paths() -> &'static [&'static str] {
    &[
        "extdeps.fixture.external_authority_bogus_scheme",
        "extdeps.fixture.external_authority_missing",
        "extdeps.fixture.external_authority_clean_https_no_anchor",
        "extdeps.fixture.external_authority_file_anchor",
    ]
}

pub fn extdeps_derived_extdeps_module_paths() -> Vec<String> {
    let index = build_module_path_index_from_witness_roots();
    let mut paths: Vec<String> = index
        .keys()
        .filter(|k| k.starts_with("extdeps."))
        .cloned()
        .collect();
    paths.sort();
    paths
}

pub fn extdeps_derived_extdeps_modules_value(
    ctx: &crate::v1_interpreter::InterpContext,
) -> crate::v1_interpreter::Value {
    use crate::v1_interpreter::list_value;
    let items: Vec<_> = extdeps_derived_extdeps_module_paths()
        .iter()
        .map(|p| free_monoid_symbol_value_from_dotted_string(ctx, p))
        .collect();
    list_value(items)
}

pub fn extdeps_external_authority_backfill_pending_entries_value(
    ctx: &crate::v1_interpreter::InterpContext,
) -> crate::v1_interpreter::Value {
    use crate::v1_interpreter::list_value;
    let mut paths: Vec<String> = external_authority_backfill_pending_module_paths()
        .iter()
        .cloned()
        .collect();
    paths.sort();
    let items: Vec<_> = paths
        .iter()
        .map(|p| free_monoid_symbol_value_from_dotted_string(ctx, p))
        .collect();
    list_value(items)
}

fn external_authority_is_backfill_pending_for_module_path(module_path: &str) -> bool {
    external_authority_backfill_pending_module_paths().contains(module_path)
}

fn external_authority_is_machinery_exempt_for_module_path(module_path: &str) -> bool {
    external_authority_machinery_exempt_module_paths().contains(&module_path)
}

fn external_authority_is_clean_tree_roster_excluded_for_module_path(module_path: &str) -> bool {
    if module_path.starts_with("extdeps.fixture.") {
        return true;
    }
    if module_path.ends_with(".mock_corpus") {
        return true;
    }
    external_authority_clean_tree_roster_exclusion_paths().contains(&module_path)
}

fn external_authority_anchor_present_in_any_source_root(module_path: &str) -> bool {
    let ws = workspace_root();
    for root in default_source_roots() {
        let root_path = std::path::PathBuf::from(&root);
        if !root_path.is_dir() {
            continue;
        }
        let mut files = Vec::new();
        collect_dag_files_tolerant(&root_path, &mut files);
        for file in files {
            let Ok(content) = std::fs::read_to_string(&file) else {
                continue;
            };
            let declares = content.lines().find_map(|l| {
                l.trim()
                    .strip_prefix("module ")
                    .map(|m| m.trim().to_string())
            });
            if declares.as_deref() != Some(module_path) {
                continue;
            }
            let rel = file
                .strip_prefix(&ws)
                .map(|p| p.to_string_lossy().replace('\\', "/"))
                .unwrap_or_else(|_| file.to_string_lossy().into_owned());
            let (items, source_indices) = parse_extdeps_module_items(&rel);
            if matches!(
                read_external_authority_anchor_from_items(&items, &source_indices),
                ExternalAuthorityAnchorProjection::Present { .. }
            ) {
                return true;
            }
        }
    }
    false
}

fn external_authority_shadow_plant_paired_extdeps_module_path(module_path: &str) -> Option<String> {
    module_path
        .strip_prefix("test.fixture.")
        .map(|leaf| format!("extdeps.fixture.{leaf}"))
}

fn external_authority_anchor_shadow_masked_for_module_path(module_path: &str) -> bool {
    match project_external_authority_anchor(module_path) {
        ExternalAuthorityAnchorProjection::Present { .. } => false,
        ExternalAuthorityAnchorProjection::Absent => {
            if external_authority_anchor_present_in_any_source_root(module_path) {
                return true;
            }
            if let Some(extdeps_path) =
                external_authority_shadow_plant_paired_extdeps_module_path(module_path)
            {
                return external_authority_anchor_present_in_any_source_root(&extdeps_path);
            }
            false
        }
    }
}

pub fn extdeps_external_authority_module_facts(
    module_path: &str,
) -> ExtdepsExternalAuthorityModuleFacts {
    let (anchor_kind, scheme_identity, locator) =
        match project_external_authority_anchor(module_path) {
            ExternalAuthorityAnchorProjection::Absent => {
                ("absent".to_string(), String::new(), String::new())
            }
            ExternalAuthorityAnchorProjection::Present {
                scheme_identity,
                locator,
            } => ("present".to_string(), scheme_identity, locator),
        };
    ExtdepsExternalAuthorityModuleFacts {
        anchor_kind,
        scheme_identity,
        locator,
        is_backfill_pending: external_authority_is_backfill_pending_for_module_path(module_path),
        is_machinery_exempt: external_authority_is_machinery_exempt_for_module_path(module_path),
        is_clean_tree_roster_excluded:
            external_authority_is_clean_tree_roster_excluded_for_module_path(module_path),
        anchor_shadow_masked: external_authority_anchor_shadow_masked_for_module_path(module_path),
    }
}

fn external_authority_live_violation_module_paths() -> Vec<String> {
    let backfill = external_authority_backfill_pending_module_paths();
    let mut violations = Vec::new();
    for path in extdeps_derived_extdeps_module_paths() {
        if external_authority_is_clean_tree_roster_excluded_for_module_path(&path) {
            continue;
        }
        if external_authority_is_machinery_exempt_for_module_path(&path) || backfill.contains(&path)
        {
            continue;
        }
        match project_external_authority_anchor(&path) {
            ExternalAuthorityAnchorProjection::Absent => violations.push(format!("missing:{path}")),
            ExternalAuthorityAnchorProjection::Present {
                scheme_identity, ..
            } if scheme_identity != "Http" && scheme_identity != "Https" => {
                violations.push(format!("non_external:{path}:{scheme_identity}"))
            }
            _ => {}
        }
    }
    violations
}

pub fn extdeps_external_authority_live_clean_tree_holds() -> bool {
    external_authority_live_violation_module_paths().is_empty()
}

pub fn extdeps_external_authority_live_roster_module_count() -> i64 {
    extdeps_derived_extdeps_module_paths()
        .into_iter()
        .filter(|path| !external_authority_is_clean_tree_roster_excluded_for_module_path(path))
        .count() as i64
}

pub fn extdeps_external_authority_live_shadow_mask_holds() -> bool {
    for path in extdeps_derived_extdeps_module_paths() {
        if external_authority_is_clean_tree_roster_excluded_for_module_path(&path)
            || external_authority_is_machinery_exempt_for_module_path(&path)
        {
            continue;
        }
        if external_authority_anchor_shadow_masked_for_module_path(&path) {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod doc_reachability_tests {
    use super::*;
    use im::HashMap;

    fn edges_of(pairs: &[(&str, &[&str])]) -> HashMap<String, Vec<String>> {
        pairs
            .iter()
            .map(|(k, vs)| {
                (
                    (*k).to_string(),
                    vs.iter().map(|s| (*s).to_string()).collect(),
                )
            })
            .collect()
    }

    #[test]
    fn reachable_set_flags_orphan_node() {
        let roots: BTreeSet<String> = ["root.md".to_string()].into_iter().collect();
        let edges = edges_of(&[("root.md", &["linked.md"]), ("orphan.md", &[])]);
        let reached = doc_reachable_set(&roots, &edges);
        assert!(reached.contains("root.md"));
        assert!(reached.contains("linked.md"));
        assert!(
            !reached.contains("orphan.md"),
            "an unlinked node must be unreachable (the orphan witness)"
        );
    }

    #[test]
    fn reachable_set_inert_cluster_stays_unreached() {
        let roots: BTreeSet<String> = ["root.md".to_string()].into_iter().collect();
        let edges = edges_of(&[
            ("root.md", &["a.md"]),
            ("a.md", &[]),
            ("dead1.md", &["dead2.md"]),
            ("dead2.md", &["dead1.md"]),
        ]);
        let reached = doc_reachable_set(&roots, &edges);
        assert!(reached.contains("a.md"));
        assert!(!reached.contains("dead1.md") && !reached.contains("dead2.md"));
    }

    #[test]
    fn reachable_set_transitive_chain() {
        let roots: BTreeSet<String> = ["r.md".to_string()].into_iter().collect();
        let edges = edges_of(&[
            ("r.md", &["a.md"]),
            ("a.md", &["b.md"]),
            ("b.md", &["c.md"]),
        ]);
        let reached = doc_reachable_set(&roots, &edges);
        for n in ["r.md", "a.md", "b.md", "c.md"] {
            assert!(reached.contains(n), "{n} should be reached");
        }
    }

    #[test]
    fn markdown_link_targets_basic() {
        let c = "see [x](docs/plans/x.md) and [y](y.md#anchor) and [ext](https://e.com) and [z](./z.md)";
        let t = markdown_link_targets(c);
        assert_eq!(t, vec!["docs/plans/x.md", "y.md", "./z.md"]);
    }

    #[test]
    fn dangling_detection_flags_missing_md_only() {
        let doc = "[ok](https://x) [broken](docs/plans/does-not-exist-xyz.md) [code](src/lib.rs)";
        let targets = markdown_link_targets(doc);
        let dangling: Vec<&String> = targets
            .iter()
            .filter(|t| {
                t.ends_with(".md")
                    && !workspace_root()
                        .join(normalize_doc_path(Path::new(t)))
                        .is_file()
            })
            .collect();
        assert_eq!(
            dangling.len(),
            1,
            "exactly the missing .md link is dangling (not the http or the existing code link): {dangling:?}"
        );
    }

    #[test]
    fn bind_md_refs_basic() {
        let c = "// bind: docs/planning/foo.md (provenance)\n// no bind here\n// bind: bar.md";
        let t = bind_md_refs(c);
        assert_eq!(t, vec!["docs/planning/foo.md", "bar.md"]);
    }
}

// --- REST transport fact projection (folded from rest_transport_facts.rs) ---
// Pure Node-tree reader over transport annotations — zero host I/O.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeclaredRestTransportOp {
    pub service: String,
    pub name: String,
    pub method: String,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RestTransportFactError {
    MissingServiceScope { operation: String },
    MissingMethodProperty { service: String, operation: String },
    MissingPathProperty { service: String, operation: String },
}

impl std::fmt::Display for RestTransportFactError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RestTransportFactError::MissingServiceScope { operation } => {
                write!(
                    f,
                    "REST transport without enclosing service scope (operation={operation})"
                )
            }
            RestTransportFactError::MissingMethodProperty { service, operation } => {
                write!(
                    f,
                    "missing method on rest transport for {service}::{operation}"
                )
            }
            RestTransportFactError::MissingPathProperty { service, operation } => {
                write!(
                    f,
                    "missing path on rest transport for {service}::{operation}"
                )
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestTransportCollectResult {
    pub ops: Vec<DeclaredRestTransportOp>,
    pub errors: Vec<RestTransportFactError>,
}

fn rest_transport_field_string(
    props: Rc<im::Vector<Rc<Node>>>,
    prop_name: String,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> Option<String> {
    use crate::v1_std_core::{find_property, find_property_string, ExprData};
    find_property_string(props.clone(), prop_name.clone(), source_indices.clone()).or_else(|| {
        let n = find_property(props, prop_name, source_indices.clone())?;
        match (*n.expr_data).clone() {
            ExprData::ExprVar { .. } => {
                let s = authored_name_at(source_indices, n);
                if s.is_empty() {
                    None
                } else {
                    Some(s)
                }
            }
            _ => None,
        }
    })
}

pub fn collect_rest_transport_operations(
    module: &Rc<Node>,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> RestTransportCollectResult {
    use crate::v1_std_core::{
        is_rest_transport, transport_method_key, transport_path_template_key,
    };
    let mut out = Vec::new();
    let mut errors = Vec::new();
    fn walk(
        n: &Rc<Node>,
        source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
        service_ctx: Option<String>,
        out: &mut Vec<DeclaredRestTransportOp>,
        errors: &mut Vec<RestTransportFactError>,
    ) {
        let ctx_for_children = match &n.transport {
            Some(t)
                if !is_rest_transport(t.clone(), source_indices.clone()) && !n.name.is_empty() =>
            {
                Some(n.name.clone())
            }
            _ => service_ctx.clone(),
        };

        if let Some(t) = &n.transport {
            if is_rest_transport(t.clone(), source_indices.clone()) {
                let Some(svc) = service_ctx.clone() else {
                    errors.push(RestTransportFactError::MissingServiceScope {
                        operation: n.name.clone(),
                    });
                    for c in n.children.iter() {
                        walk(
                            c,
                            source_indices.clone(),
                            ctx_for_children.clone(),
                            out,
                            errors,
                        );
                    }
                    return;
                };
                let method = rest_transport_field_string(
                    t.properties.clone(),
                    transport_method_key(),
                    source_indices.clone(),
                );
                let Some(method) = method else {
                    errors.push(RestTransportFactError::MissingMethodProperty {
                        service: svc.clone(),
                        operation: n.name.clone(),
                    });
                    for c in n.children.iter() {
                        walk(
                            c,
                            source_indices.clone(),
                            ctx_for_children.clone(),
                            out,
                            errors,
                        );
                    }
                    return;
                };
                let path = rest_transport_field_string(
                    t.properties.clone(),
                    transport_path_template_key(),
                    source_indices.clone(),
                );
                let Some(path) = path else {
                    errors.push(RestTransportFactError::MissingPathProperty {
                        service: svc.clone(),
                        operation: n.name.clone(),
                    });
                    for c in n.children.iter() {
                        walk(
                            c,
                            source_indices.clone(),
                            ctx_for_children.clone(),
                            out,
                            errors,
                        );
                    }
                    return;
                };
                out.push(DeclaredRestTransportOp {
                    service: svc,
                    name: n.name.clone(),
                    method,
                    path,
                });
            }
        }

        for c in n.children.iter() {
            walk(
                c,
                source_indices.clone(),
                ctx_for_children.clone(),
                out,
                errors,
            );
        }
    }
    walk(module, source_indices, None, &mut out, &mut errors);
    RestTransportCollectResult { ops: out, errors }
}

// --- Wire value serialization (folded from wire_value_serialize.rs) ---
// Pure coproduct wire-policy projection for interpreter REST bodies — zero host I/O.

type WireSerializeResult<T> = Result<T, String>;

pub fn resolve_coproduct_wire_policy(
    coproduct_name: &str,
    modules: &[Rc<TypedModule>],
    source_indices: &HashMap<String, Rc<NewlineIndex>>,
) -> Option<Rc<crate::v1_compiler_emit_rust::RustEnumWireSerde>> {
    use crate::v1_compiler_emit_rust::resolve_local_coproduct_wire_policy;
    use crate::v1_std_core::module_imports;
    let si = Rc::new(source_indices.clone());
    let mut matches: Vec<Rc<crate::v1_compiler_emit_rust::RustEnumWireSerde>> = Vec::new();
    for tm in modules {
        let imports = module_imports(tm.module.clone());
        if let Some(local) = resolve_local_coproduct_wire_policy(
            coproduct_name.to_string(),
            false,
            tm.items.clone(),
            imports,
            si.clone(),
        ) {
            if local.error_message.is_none() {
                matches.push(local);
            }
        }
    }
    if matches.is_empty() {
        None
    } else if matches.len() == 1 {
        Some(matches[0].clone())
    } else {
        let first = &matches[0];
        if matches.iter().all(|m| m == first) {
            Some(first.clone())
        } else {
            None
        }
    }
}

fn wire_resolve_sym(ctx: &v1_interpreter::InterpContext, sym: v1_interpreter::Symbol) -> String {
    ctx.resolve(sym)
}

pub fn value_to_wire_json(
    val: &v1_interpreter::Value,
    ctx: &v1_interpreter::InterpContext,
) -> WireSerializeResult<serde_json::Value> {
    match val {
        v1_interpreter::Value::Variant {
            type_name,
            variant_name,
            fields,
        } => serialize_variant_to_wire_json(
            &wire_resolve_sym(ctx, *type_name),
            &wire_resolve_sym(ctx, *variant_name),
            fields,
            ctx,
        ),
        v1_interpreter::Value::Null => Ok(serde_json::Value::Null),
        v1_interpreter::Value::Bool(b) => Ok(serde_json::Value::Bool(*b)),
        v1_interpreter::Value::Int(n) => Ok(serde_json::json!(*n)),
        v1_interpreter::Value::Float(f) => Ok(serde_json::json!(*f)),
        v1_interpreter::Value::Str(s) => {
            if s.starts_with('[') || s.starts_with('{') {
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(s) {
                    return Ok(parsed);
                }
            }
            Ok(serde_json::Value::String(s.clone()))
        }
        v1_interpreter::Value::List(items) => {
            let mut arr = Vec::with_capacity(items.len());
            for item in items.iter() {
                arr.push(value_to_wire_json(item, ctx)?);
            }
            Ok(serde_json::Value::Array(arr))
        }
        v1_interpreter::Value::Set(members) => Ok(serde_json::Value::Array(
            members
                .iter()
                .map(|s| serde_json::Value::String(s.clone()))
                .collect(),
        )),
        v1_interpreter::Value::Map(m) => {
            let mut obj = serde_json::Map::new();
            for (k, v) in m.iter() {
                let key = match k.value_ref() {
                    v1_interpreter::Value::Str(s) => s.clone(),
                    other => {
                        return Err(format!(
                            "cannot serialize map with non-string key to JSON (got {other:?} key)"
                        ))
                    }
                };
                obj.insert(key, value_to_wire_json(v, ctx)?);
            }
            Ok(serde_json::Value::Object(obj))
        }
        v1_interpreter::Value::Record { fields, .. } => {
            let mut obj = serde_json::Map::new();
            for (k, v) in fields.iter() {
                if matches!(v, v1_interpreter::Value::Null) {
                    continue;
                }
                obj.insert(wire_resolve_sym(ctx, *k), value_to_wire_json(v, ctx)?);
            }
            Ok(serde_json::Value::Object(obj))
        }
        v1_interpreter::Value::Unit => Ok(serde_json::Value::Null),
        v1_interpreter::Value::Closure { .. } => {
            Ok(serde_json::Value::String("<closure>".to_string()))
        }
        v1_interpreter::Value::Fn { node } => {
            Ok(serde_json::Value::String(format!("<fn {}>", node.name)))
        }
    }
}

fn serialize_variant_to_wire_json(
    type_name: &str,
    variant_name: &str,
    fields: &[(v1_interpreter::Symbol, v1_interpreter::Value)],
    ctx: &v1_interpreter::InterpContext,
) -> WireSerializeResult<serde_json::Value> {
    use crate::v1_compiler_emit_rust::{
        policy_is_string_variant, policy_is_untagged, policy_serde_tag_field, rust_serde_tag_attr,
        rust_tagged_object_policy, wire_variant_tag_for_policy,
    };
    let policy = resolve_coproduct_wire_policy(
        type_name,
        &ctx.modules.iter().cloned().collect::<std::vec::Vec<_>>(),
        ctx.source_indices.as_ref(),
    )
    .unwrap_or_else(|| rust_tagged_object_policy());

    if policy.error_message.is_some() {
        return Err(policy
            .error_message
            .clone()
            .unwrap_or_else(|| format!("wire policy error for coproduct {type_name}")));
    }

    if policy_is_untagged(policy.clone()) {
        return serialize_untagged_variant(fields, ctx);
    }

    if policy_is_string_variant(policy.clone()) {
        let tag = wire_variant_tag_for_policy(variant_name.to_string(), policy.clone())
            .ok_or_else(|| format!("no wire tag for string variant {type_name}::{variant_name}"))?;
        return Ok(serde_json::Value::String(tag));
    }

    if let Some(tag_field) = policy_serde_tag_field(policy.clone()) {
        let wire_tag = wire_variant_tag_for_policy(variant_name.to_string(), policy.clone())
            .ok_or_else(|| {
                format!("no wire tag for internally-tagged variant {type_name}::{variant_name}")
            })?;
        let mut obj = serde_json::Map::new();
        obj.insert(tag_field, serde_json::Value::String(wire_tag));
        for (k, v) in fields.iter() {
            if matches!(v, v1_interpreter::Value::Null) {
                continue;
            }
            obj.insert(wire_resolve_sym(ctx, *k), value_to_wire_json(v, ctx)?);
        }
        return Ok(serde_json::Value::Object(obj));
    }

    let tag_key = policy_serde_tag_field(policy.clone()).unwrap_or_else(|| "_variant".to_string());
    let default_tag = if policy.enum_attr == rust_serde_tag_attr() {
        variant_name.to_string()
    } else {
        wire_variant_tag_for_policy(variant_name.to_string(), policy.clone())
            .unwrap_or_else(|| variant_name.to_string())
    };
    let mut obj = serde_json::Map::new();
    obj.insert(tag_key, serde_json::Value::String(default_tag));
    for (k, v) in fields.iter() {
        if matches!(v, v1_interpreter::Value::Null) {
            continue;
        }
        obj.insert(wire_resolve_sym(ctx, *k), value_to_wire_json(v, ctx)?);
    }
    Ok(serde_json::Value::Object(obj))
}

fn serialize_untagged_variant(
    fields: &[(v1_interpreter::Symbol, v1_interpreter::Value)],
    ctx: &v1_interpreter::InterpContext,
) -> WireSerializeResult<serde_json::Value> {
    let mut values: Vec<serde_json::Value> = fields
        .iter()
        .map(|(_, v)| v)
        .filter(|v| !matches!(v, v1_interpreter::Value::Null))
        .map(|v| value_to_wire_json(v, ctx))
        .collect::<Result<Vec<_>, _>>()?;
    match values.len() {
        0 => Ok(serde_json::Value::Null),
        1 => Ok(values.remove(0)),
        _ => {
            let mut obj = serde_json::Map::new();
            for (k, v) in fields.iter() {
                if matches!(v, v1_interpreter::Value::Null) {
                    continue;
                }
                obj.insert(wire_resolve_sym(ctx, *k), value_to_wire_json(v, ctx)?);
            }
            Ok(serde_json::Value::Object(obj))
        }
    }
}

#[cfg(test)]
mod import_closure_equivalence_tests {
    use super::{
        build_module_graph_facts_live, build_module_index, build_multi_entry_index,
        closure_subject_for_entry, default_source_roots, floor_discovery_path_excluded,
        import_closure_live_paths, load_sources_for_entry_with_index,
        module_graph_facts_build_count_for_test, reset_module_graph_facts_build_count_for_test,
        resolve_entry_with_index, resolve_transitively, resolve_transitively_bfs_legacy,
        witness_layer_roots, workspace_relative_repo_path,
    };
    use im::HashMap;
    use std::collections::BTreeSet;
    use std::path::{Path, PathBuf};
    use std::rc::Rc;

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(3)
            .expect("workspace root")
            .to_path_buf()
    }

    fn closure_paths(
        sources: &[Rc<crate::v1_compiler_compile::SourceFile>],
    ) -> std::collections::BTreeSet<String> {
        sources
            .iter()
            .map(|s| workspace_relative_repo_path(&s.path))
            .collect()
    }

    fn assert_bfs_matches_import_closure_live_with_facts(
        entry_rel: &str,
        index: &super::ModuleSourceIndex,
        facts: &super::ModuleGraphFactsLive,
    ) {
        let ws = workspace_root();
        let entry_abs = ws.join(entry_rel);
        let content =
            std::fs::read_to_string(&entry_abs).unwrap_or_else(|e| panic!("read {entry_rel}: {e}"));
        let entry_source = Rc::new(crate::v1_compiler_compile::SourceFile {
            path: entry_abs.to_string_lossy().into_owned(),
            content,
        });
        let mut seen: HashMap<String, Rc<crate::v1_compiler_compile::SourceFile>> = HashMap::new();
        if let Some(mod_path) = super::extract_module_path(&entry_source.content) {
            seen.insert(mod_path, entry_source.clone());
        }
        let bfs = resolve_transitively_bfs_legacy(vec![entry_source.clone()], index, seen);
        let repointed = resolve_transitively(vec![entry_source], index, facts)
            .unwrap_or_else(|e| panic!("resolve_transitively {entry_rel}: {e}"));
        let live = super::import_closure_live_paths_with_facts(entry_rel, facts);
        let bfs_paths = closure_paths(&bfs);
        let repointed_paths = closure_paths(&repointed);
        let live_paths: BTreeSet<String> = live
            .iter()
            .map(|p| workspace_relative_repo_path(p))
            .collect();
        assert_eq!(
            repointed_paths, bfs_paths,
            "repointed closure diverged from legacy BFS for {entry_rel}"
        );
        assert_eq!(
            live_paths, bfs_paths,
            "import_closure_live diverged from legacy BFS for {entry_rel}"
        );
    }

    fn assert_bfs_matches_import_closure_live(entry_rel: &str, pool_roots: &[String]) {
        let index = build_module_index(pool_roots);
        let facts = super::build_module_graph_facts_live(pool_roots);
        assert_bfs_matches_import_closure_live_with_facts(entry_rel, &index, &facts);
    }

    /// Floor witness entry paths enrolled by the source-root `*_test.dag` pass
    /// (`gunbc.ci_layer_roots.witness_layer_roots`), minus the model exclusion list.
    /// Avoids `discover_floor_corpus_rows` lens-hygiene work — closure set-identity
    /// only needs the witness entry roster, not inert-lens classification.
    fn floor_witness_entry_paths_for_oracle() -> BTreeSet<String> {
        let mut entries = BTreeSet::new();
        for root in default_source_roots() {
            let mut dag_files = Vec::new();
            super::collect_dag_files_tolerant(Path::new(&root), &mut dag_files);
            for path in dag_files {
                let rel = workspace_relative_repo_path(&path.to_string_lossy());
                if !rel.ends_with("_test.dag") || floor_discovery_path_excluded(&rel) {
                    continue;
                }
                entries.insert(rel);
            }
        }
        entries
    }

    /// Pre-BFS fixpoint from `origin/main` — retained for perf receipt only.
    fn import_closure_from_facts_fixpoint_legacy(
        entry_path: &str,
        edges: &[super::ImportResolutionFactRaw],
        nodes: &[super::ModuleDeclarationFactRaw],
    ) -> Vec<String> {
        let entry_path = workspace_relative_repo_path(entry_path);
        let mut reached: Vec<String> = vec![entry_path];
        let fuel = nodes.len();
        for _ in 0..fuel {
            let before = reached.len();
            let mut next = reached.clone();
            for importer in &reached {
                let importer_norm = workspace_relative_repo_path(importer);
                for edge in edges {
                    if workspace_relative_repo_path(&edge.path) != importer_norm {
                        continue;
                    }
                    for node in nodes {
                        if node.module == edge.import_module {
                            let path = workspace_relative_repo_path(&node.path);
                            if !next.iter().any(|p| p == &path) {
                                next.push(path);
                            }
                        }
                    }
                }
            }
            if next.len() == before {
                break;
            }
            reached = next;
        }
        reached
    }

    /// Manual perf receipt (P4): `cargo test -p v1-compiler --lib import_closure_bfs_vs_fixpoint_perf_receipt -- --ignored --nocapture`
    #[test]
    #[ignore = "manual perf receipt: import_closure BFS vs fixpoint on floor witness roster"]
    fn import_closure_bfs_vs_fixpoint_perf_receipt() {
        use std::time::Instant;

        const CLOSURE_CALLS_PER_ENTRY: usize = 2;

        let roots = default_source_roots();
        let entries: Vec<String> = floor_witness_entry_paths_for_oracle().into_iter().collect();
        let facts = super::build_module_graph_facts_live(&roots);
        let call_count = entries.len() * CLOSURE_CALLS_PER_ENTRY;

        let t0 = Instant::now();
        for entry_rel in &entries {
            for _ in 0..CLOSURE_CALLS_PER_ENTRY {
                let _ = import_closure_from_facts_fixpoint_legacy(
                    entry_rel,
                    &facts.edges,
                    &facts.nodes,
                );
            }
        }
        let fixpoint_ms = t0.elapsed().as_secs_f64() * 1000.0;

        let t1 = Instant::now();
        for entry_rel in &entries {
            for _ in 0..CLOSURE_CALLS_PER_ENTRY {
                let _ = super::import_closure_live_paths_with_facts(entry_rel, &facts);
            }
        }
        let bfs_ms = t1.elapsed().as_secs_f64() * 1000.0;

        let speedup = fixpoint_ms / bfs_ms.max(0.001);
        eprintln!(
            "import_closure perf receipt: entries={} calls={} fixpoint_ms={:.1} bfs_ms={:.1} speedup={:.1}x",
            entries.len(),
            call_count,
            fixpoint_ms,
            bfs_ms,
            speedup
        );
        assert!(
            bfs_ms < fixpoint_ms,
            "BFS should beat fixpoint on floor roster (fixpoint={fixpoint_ms:.1}ms bfs={bfs_ms:.1}ms)"
        );
    }

    #[test]
    fn import_closure_live_matches_legacy_bfs_on_whole_floor_corpus() {
        let roots = default_source_roots();
        let entries = floor_witness_entry_paths_for_oracle();
        assert!(
            entries.len() >= 4,
            "import-closure semantic oracle expects the full floor roster (got {})",
            entries.len()
        );
        let index = build_module_index(&roots);
        let facts = super::build_module_graph_facts_live(&roots);
        for entry_rel in entries {
            assert_bfs_matches_import_closure_live_with_facts(&entry_rel, &index, &facts);
        }
    }

    #[test]
    fn import_closure_live_matches_legacy_bfs_on_conformance_entry() {
        let roots = default_source_roots();
        assert_bfs_matches_import_closure_live(
            "src/v2/test/claim/manual/coproduct_reflection_conformance_test.dag",
            &roots,
        );
    }

    #[test]
    fn import_closure_live_matches_legacy_bfs_on_floor_gate_entry() {
        let roots = default_source_roots();
        assert_bfs_matches_import_closure_live("dag/tools/floor_effect_gate_witness.dag", &roots);
    }

    #[test]
    fn import_closure_live_matches_legacy_bfs_on_budget_roster_completeness() {
        let roots = default_source_roots();
        assert_bfs_matches_import_closure_live(
            "src/v2/test/claim/complexity_gate/budget_roster_completeness_test.dag",
            &roots,
        );
    }

    #[test]
    fn import_closure_live_matches_legacy_bfs_on_fold_list_generic_instantiation() {
        let roots = default_source_roots();
        assert_bfs_matches_import_closure_live(
            "src/v2/test/claim/fold_list_generic_instantiation.dag",
            &roots,
        );
    }

    fn module_paths_for_sources(
        sources: &[Rc<crate::v1_compiler_compile::SourceFile>],
    ) -> Vec<String> {
        let mut out: Vec<String> = sources
            .iter()
            .filter_map(|s| super::extract_module_path(&s.content))
            .collect();
        out.sort();
        out.dedup();
        out
    }

    #[test]
    fn import_closure_module_path_set_identity_matches_legacy_bfs_on_witness_roots() {
        let roots = default_source_roots();
        let entries = [
            "src/v2/test/claim/manual/coproduct_reflection_conformance_test.dag",
            "dag/tools/floor_effect_gate_witness.dag",
            "src/v2/test/claim/complexity_gate/budget_roster_completeness_test.dag",
            "src/v2/test/claim/fold_list_generic_instantiation.dag",
        ];
        for entry_rel in entries {
            let ws = workspace_root();
            let index = build_module_index(&roots);
            let content = std::fs::read_to_string(ws.join(entry_rel))
                .unwrap_or_else(|e| panic!("read {entry_rel}: {e}"));
            let entry_source = Rc::new(crate::v1_compiler_compile::SourceFile {
                path: ws.join(entry_rel).to_string_lossy().into_owned(),
                content,
            });
            let mut seen: HashMap<String, Rc<crate::v1_compiler_compile::SourceFile>> =
                HashMap::new();
            if let Some(mod_path) = super::extract_module_path(&entry_source.content) {
                seen.insert(mod_path, entry_source.clone());
            }
            let bfs = resolve_transitively_bfs_legacy(vec![entry_source.clone()], &index, seen);
            let facts = super::build_module_graph_facts_live(&roots);
            let repointed =
                resolve_transitively(vec![entry_source], &index, &facts).expect("repointed");
            let bfs_modules = module_paths_for_sources(&bfs);
            let repointed_modules = module_paths_for_sources(&repointed);
            assert_eq!(
                repointed_modules, bfs_modules,
                "module-path set identity diverged for {entry_rel}"
            );
            let live = super::import_closure_live_paths_with_facts(entry_rel, &facts);
            let live_modules: Vec<String> = live
                .iter()
                .filter_map(|p| {
                    let path = ws.join(super::workspace_relative_repo_path(p));
                    std::fs::read_to_string(&path)
                        .ok()
                        .and_then(|c| super::extract_module_path(&c))
                })
                .collect::<std::collections::BTreeSet<_>>()
                .into_iter()
                .collect();
            assert_eq!(
                live_modules, bfs_modules,
                "import_closure_live module-path set diverged for {entry_rel}"
            );
        }
    }

    #[test]
    fn module_graph_facts_scanned_once_per_multi_entry_index_hot_path() {
        reset_module_graph_facts_build_count_for_test();
        let ws = workspace_root();
        let roots = default_source_roots();
        let index = build_multi_entry_index(&roots);
        assert_eq!(
            module_graph_facts_build_count_for_test(),
            1,
            "module graph facts must be built once with MultiEntryIndex"
        );
        let budget = ws
            .join("src/v2/test/claim/complexity_gate/budget_roster_completeness_test.dag")
            .to_string_lossy()
            .into_owned();
        let fold = ws
            .join("src/v2/test/claim/fold_list_generic_instantiation.dag")
            .to_string_lossy()
            .into_owned();
        closure_subject_for_entry(&index, &budget).expect("budget_roster closure");
        assert_eq!(
            module_graph_facts_build_count_for_test(),
            1,
            "budget_roster closure must not re-scan corpus for facts"
        );
        closure_subject_for_entry(&index, &fold).expect("fold_list closure");
        assert_eq!(
            module_graph_facts_build_count_for_test(),
            1,
            "second entry closure must not re-scan corpus for facts"
        );
    }

    #[test]
    fn resolve_transitively_threads_prebuilt_facts_without_rescan() {
        reset_module_graph_facts_build_count_for_test();
        let roots = default_source_roots();
        let index = build_module_index(&roots);
        let facts = build_module_graph_facts_live(&roots);
        assert_eq!(module_graph_facts_build_count_for_test(), 1);
        let entries = [
            "src/v2/test/claim/complexity_gate/budget_roster_completeness_test.dag",
            "src/v2/test/claim/fold_list_generic_instantiation.dag",
        ];
        let mut entry_sources = Vec::new();
        for entry_rel in entries {
            let content = std::fs::read_to_string(workspace_root().join(entry_rel))
                .unwrap_or_else(|e| panic!("read {entry_rel}: {e}"));
            entry_sources.push(Rc::new(crate::v1_compiler_compile::SourceFile {
                path: workspace_root()
                    .join(entry_rel)
                    .to_string_lossy()
                    .into_owned(),
                content,
            }));
        }
        resolve_transitively(entry_sources, &index, &facts).expect("union closure");
        assert_eq!(
            module_graph_facts_build_count_for_test(),
            1,
            "multi-entry resolve_transitively must not re-scan when facts are threaded"
        );
    }

    // RED control for the out-of-pool entry refusal (the green control is
    // `resolve_transitively_threads_prebuilt_facts_without_rescan` above: in-pool
    // entries resolve). An entry outside every source root has no adjacency row,
    // so pre-refusal the closure silently truncated to the entry alone and its
    // imports surfaced downstream as `unresolved import` on modules that exist —
    // the interp_recorded fixture-witness dark red (6 checks, masked since #6210).
    #[test]
    fn resolve_transitively_refuses_entry_outside_facts_pool() {
        let roots = default_source_roots();
        let index = build_module_index(&roots);
        let facts = build_module_graph_facts_live(&roots);
        let scratch =
            std::env::temp_dir().join(format!("gunbc-out-of-pool-entry-{}", std::process::id()));
        std::fs::create_dir_all(&scratch).expect("scratch dir");
        let entry_path = scratch.join("out_of_pool_entry.dag");
        let content = "module test.claim.out_of_pool_entry\n\nimport extdeps.filesystem.filesystem_io\n\nfunc out_of_pool_probe() -> Bool {\n  true\n}\n";
        std::fs::write(&entry_path, content).expect("write entry");
        let entry = Rc::new(crate::v1_compiler_compile::SourceFile {
            path: entry_path.to_string_lossy().into_owned(),
            content: content.to_string(),
        });
        let err = resolve_transitively(vec![entry], &index, &facts)
            .expect_err("an entry outside every source root must refuse, not truncate");
        std::fs::remove_dir_all(&scratch).ok();
        assert!(
            err.contains("no provenance in the module-graph facts pool")
                && err.contains("out_of_pool_entry.dag"),
            "refusal must name the cause and the entry: {err}"
        );
    }

    #[test]
    fn load_sources_for_entry_does_not_duplicate_entry_under_path_alias() {
        let ws = workspace_root();
        std::env::set_current_dir(&ws).expect("chdir workspace root");
        let roots = default_source_roots();
        let index = build_module_index(&roots);
        let facts = build_module_graph_facts_live(&roots);
        let entry_rel = "src/v2/workflow/floor_diff_observe.dag";
        let sources =
            load_sources_for_entry_with_index(&index, &facts, entry_rel).expect("load closure");
        let entry_norm = workspace_relative_repo_path(entry_rel);
        let entry_count = sources
            .iter()
            .filter(|s| workspace_relative_repo_path(&s.path) == entry_norm)
            .count();
        assert_eq!(
            entry_count, 1,
            "relative entry path must not duplicate an absolute-indexed closure member"
        );
        resolve_entry_with_index(&build_multi_entry_index(&roots), entry_rel)
            .expect("floor_diff_observe must resolve without duplicate-module error");
    }

    #[test]
    fn import_closure_live_drift_discriminates_under_declaration() {
        let roots = default_source_roots();
        let live = import_closure_live_paths(
            "src/v2/test/claim/manual/coproduct_reflection_conformance_test.dag",
            &roots,
        )
        .expect("live closure");
        let mut without_entry: std::collections::BTreeSet<String> = live
            .iter()
            .map(|p| workspace_relative_repo_path(p))
            .filter(|p| p != "src/v2/test/claim/manual/coproduct_reflection_conformance_test.dag")
            .collect();
        let repointed = import_closure_live_paths(
            "src/v2/test/claim/manual/coproduct_reflection_conformance_test.dag",
            &roots,
        )
        .expect("live closure again");
        let full: std::collections::BTreeSet<String> = repointed
            .iter()
            .map(|p| workspace_relative_repo_path(p))
            .collect();
        assert_ne!(
            without_entry, full,
            "RED control: dropped entry must diverge"
        );
        without_entry.insert("src/v2/std/__bogus_never_imported__.dag".to_string());
        assert_ne!(
            without_entry, full,
            "RED control: bogus path must diverge from live closure"
        );
    }

    #[test]
    fn import_closure_live_uses_witness_layer_roots_without_extra_resolve() {
        let ws = workspace_root();
        let rel_roots: Vec<String> = witness_layer_roots();
        let abs_roots: Vec<String> = rel_roots
            .iter()
            .map(|r| ws.join(r).to_string_lossy().into_owned())
            .collect();
        let from_rel = import_closure_live_paths(
            "src/v2/test/claim/manual/coproduct_reflection_conformance_test.dag",
            &rel_roots,
        )
        .expect("relative roots");
        let from_abs = import_closure_live_paths(
            "src/v2/test/claim/manual/coproduct_reflection_conformance_test.dag",
            &abs_roots,
        )
        .expect("absolute roots");
        let norm = |paths: Vec<String>| {
            paths
                .into_iter()
                .map(|p| workspace_relative_repo_path(&p))
                .collect::<std::collections::BTreeSet<_>>()
        };
        assert_eq!(norm(from_rel), norm(from_abs));
    }
}

#[cfg(test)]
mod process_resolve_store_tests {
    use super::*;

    // S1a purity + dedup receipt: the second resolve of the same (roots, entry)
    // must be served from the process store — Rc identity proves zero recompute.
    // A tiny self-contained fixture tree keeps this test milliseconds-cold.
    #[test]
    fn process_resolve_store_dedupes_repeat_resolve() {
        // Fixture must live under the workspace (build_module_path_index requires it);
        // target/ is workspace-local and git-ignored.
        let dir = workspace_root()
            .join("target")
            .join(format!("gunbc-resolve-store-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("create fixture dir");
        std::fs::write(
            dir.join("store_probe.dag"),
            "module store_probe\n\nfn probe() -> Bool {\n  true\n}\n",
        )
        .expect("write fixture module");
        let roots = vec![dir.to_string_lossy().into_owned()];
        let entry = dir.join("store_probe.dag").to_string_lossy().into_owned();

        let (g1, i1) = resolve_entry_graph_shared(&roots, &entry).expect("first resolve");
        let (g2, i2) = resolve_entry_graph_shared(&roots, &entry).expect("second resolve");
        assert!(
            Rc::ptr_eq(&g1, &g2),
            "second resolve must be the stored graph (Rc identity), not a recompute"
        );
        assert!(Rc::ptr_eq(&i1, &i2), "source indices must be stored too");

        let _ = std::fs::remove_dir_all(&dir);
    }
}

#[cfg(test)]
mod peel_alias_fixpoint_termination {
    // §4 boundedness witness for the peel_alias_once_for_field_access fixpoint
    // guard (04_infer.dag peel_alias_fixpoint_guard_note). Discriminating RED:
    // pre-guard, resolve returning the input node itself re-enters the recurse
    // arm forever (measured 3M+ iterations / 396M resolve calls on the #6640
    // total-census witness); post-guard, once == n breaks at the first repeat.
    // The fixture binds a NoConnective/1-child/inferred=None name to its OWN
    // node — resolve then yields the identical node, the exact self-resolving
    // shape (e.g. `List`) from the strip-tree measurement. Rc types are !Send,
    // so the worker thread builds everything itself and reports a projection;
    // the timeout converts a regression from a suite hang into a located red.
    #[test]
    fn peel_terminates_at_resolve_fixpoint() {
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let span = crate::v1_std_core::kernel_span("PeelFixpointProbe".to_string());
            let elem = crate::v1_std_core::leaf_node_with_span(
                "PeelFixpointElem".to_string(),
                crate::v1_std_core::kernel_span("PeelFixpointElem".to_string()),
            );
            let base =
                crate::v1_std_core::leaf_node_with_span("PeelFixpointProbe".to_string(), span);
            let n = std::rc::Rc::new(crate::v1_std_core::Node {
                children: std::rc::Rc::new(im::vector![elem]),
                ..(*base).clone()
            });
            // The strip-tree mechanism: the name resolves via SymbolIndex.global_bare
            // to an unresolved stub that IS the same parameterized shape
            // (build_symbol_index_census stores unresolved stubs), so
            // resolve_node(n) == n — the resolve fixed point the recurse arm
            // loops on (measured: 3M+ iterations of one peel call pre-guard).
            let census_binding = std::rc::Rc::new(crate::v1_compiler_infer_env::TypeBinding {
                name: "PeelFixpointProbe".to_string(),
                resolved: n.clone(),
                provenance: std::rc::Rc::new(
                    crate::std_induction::SubValueRelation::SubValueUnknown,
                ),
            });
            let global_bare = crate::v1_rt::rc_map_insert(
                crate::v1_rt::rc_empty_map(),
                "PeelFixpointProbe".to_string(),
                std::rc::Rc::new(
                    crate::v1_compiler_infer_env::GlobalBareLookupState::GlobalBareUniqueBinding {
                        module_path: "".to_string(),
                        binding: census_binding,
                    },
                ),
            );
            let symbol_index = std::rc::Rc::new(crate::v1_compiler_infer_env::SymbolIndex {
                entries: crate::v1_rt::rc_empty_map(),
                global_bare,
                services: crate::v1_rt::rc_empty_map(),
            });
            let env = std::rc::Rc::new(crate::v1_compiler_infer_env::TypeEnv {
                module_path: "".to_string(),
                bindings: crate::v1_rt::rc_empty_map(),
                str_bindings: crate::v1_rt::rc_empty_map(),
                ancestry_str_bindings: crate::v1_rt::rc_empty_map(),
                parents: std::rc::Rc::new(im::vector![]),
                recursive_types: std::rc::Rc::new(im::vector![]),
                recursive_type_set: crate::v1_rt::rc_empty_map(),
                inductive_fields: crate::v1_rt::rc_empty_map(),
                source_indices: crate::v1_rt::rc_empty_map(),
                intern_table: crate::v1_std_core::empty_intern_table(),
                source_visible_names: crate::v1_rt::rc_empty_map(),
                symbol_index,
            });
            let out = crate::v1_compiler_infer::peel_alias_once_for_field_access(
                n.clone(),
                env,
                "peel_fixpoint_probe".to_string(),
            );
            let _ = tx.send((out.name.clone(), out == n));
        });
        let (name, is_fixpoint) = rx.recv_timeout(std::time::Duration::from_secs(30)).expect(
            "peel_alias_once_for_field_access did not terminate within 30s — the \
                 fixpoint guard regressed (pre-guard this fixture spins forever)",
        );
        assert_eq!(name, "PeelFixpointProbe");
        // Termination IS the property under test (the pre-guard control run for
        // this fixture is recorded in the PR body; the strip-tree integration
        // RED — whole-tree completion + bounded peel iterations on the #6640
        // witness — is owned by the namespace-migration lane). Strict `out == n`
        // identity is deliberately NOT asserted: resolve re-stamps node
        // decorations on return, so identity is env-shape-dependent while
        // termination + name preservation are not.
        let _ = is_fixpoint;
    }
}

#[cfg(test)]
mod sigs_env_flat_parents {
    // §6 cost-shape witnesses for the flat sigs-env closure
    // (04_sigs.dag sigs_env_flat_parents_note). The prior shape nested each
    // parent env recursively and lookup walked the shared import DAG as a
    // TREE with no visited state — probes multiplied per PATH (measured:
    // identical 541 signature requests cost 53.3M env probes, 902.8M after
    // #6750 widened one closure by 54 modules), with a quadratic
    // parents-prefix copy per step. These witnesses pin the two properties
    // the flat invariant rests on: (1) the flat list is bounded by DISTINCT
    // closure modules, never paths (the name dedup is load-bearing — without
    // it construction itself re-becomes path-counted and the watchdog fires);
    // (2) linearization preserves the old walk's shadowing order exactly
    // (own local, then closure-of-last-import before earlier imports).
    use std::rc::Rc;

    fn w2_sig(fn_name: &str, marker: &str) -> Rc<crate::v1_compiler_infer_sigs::ResolvedFuncSig> {
        Rc::new(crate::v1_compiler_infer_sigs::ResolvedFuncSig {
            name: fn_name.to_string(),
            params: Rc::new(im::vector![]),
            inferred: crate::v1_std_core::leaf_node_with_span(
                marker.to_string(),
                crate::v1_std_core::kernel_span(marker.to_string()),
            ),
            is_async: false,
            output_provenance: Rc::new(im::vector![]),
            variant_provenance: crate::v1_rt::rc_empty_map(),
        })
    }

    fn w2_env(
        name: &str,
        sigs: &[(&str, &str)],
        direct: Vec<Rc<crate::v1_compiler_infer_sigs::ResolvedFuncEnv>>,
    ) -> Rc<crate::v1_compiler_infer_sigs::ResolvedFuncEnv> {
        let mut local = crate::v1_rt::rc_empty_map();
        for (f, m) in sigs {
            local = crate::v1_rt::rc_map_insert(local, f.to_string(), w2_sig(f, m));
        }
        Rc::new(crate::v1_compiler_infer_sigs::ResolvedFuncEnv {
            name: name.to_string(),
            local,
            parents: crate::v1_compiler_infer_sigs::flatten_parent_envs(Rc::new(
                direct.into_iter().collect(),
            )),
        })
    }

    // Diamond tower, depth 64: at each level two mids (a_i, b_i) import the
    // previous join; the next join imports both. Path count to the base is
    // 2^64; distinct ancestors of the top join are exactly 3*K (a_i + b_i +
    // lower joins + base). Pre-fix, a single MISS lookup walked per-path and
    // never returned; without the name dedup, construction itself doubles
    // per level. Either regression trips the 30s watchdog; the length assert
    // is the structural tooth.
    #[test]
    fn flat_parents_diamond_dedups_to_distinct_modules_and_lookup_terminates() {
        const K: usize = 64;
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let mut prev = w2_env("base", &[("bottom_fn", "FromBase")], vec![]);
            for i in 1..=K {
                let a = w2_env(&format!("a{i}"), &[], vec![prev.clone()]);
                let b = w2_env(&format!("b{i}"), &[], vec![prev.clone()]);
                prev = w2_env(&format!("j{i}"), &[], vec![a, b]);
            }
            let deep_hit = crate::v1_compiler_infer_sigs::lookup_resolved_sig(
                prev.clone(),
                "bottom_fn".to_string(),
            );
            let miss = crate::v1_compiler_infer_sigs::lookup_resolved_sig(
                prev.clone(),
                "absent_fn".to_string(),
            );
            let _ = tx.send((
                prev.parents.len(),
                deep_hit.map(|s| s.inferred.name.clone()),
                miss.is_none(),
            ));
        });
        let (flat_len, deep_hit, miss_is_none) =
            rx.recv_timeout(std::time::Duration::from_secs(30)).expect(
                "flat sigs-env construction/lookup did not terminate within 30s — \
                 the closure is being walked or built per-path again (dedup or \
                 flat-scan regressed; pre-fix this fixture is 2^64 paths)",
            );
        assert_eq!(
            flat_len,
            3 * K,
            "flat parents must hold DISTINCT closure modules (3K for the diamond tower), never path counts"
        );
        assert_eq!(deep_hit.as_deref(), Some("FromBase"));
        assert!(
            miss_is_none,
            "absent name must resolve to None, not a diagnostic or a hang"
        );
    }

    // Shadowing linearization: the old walk was deep-first, LAST import
    // first. Flat order must reproduce all three consequences: (a) among
    // direct imports both defining f, the last import wins; (b) a name in
    // the LAST import's transitive closure beats the same name directly in
    // an EARLIER import; (c) own local beats every parent.
    #[test]
    fn flat_parents_preserve_deep_first_last_import_first_shadowing() {
        let read = |env: &Rc<crate::v1_compiler_infer_sigs::ResolvedFuncEnv>, f: &str| {
            crate::v1_compiler_infer_sigs::lookup_resolved_sig(env.clone(), f.to_string())
                .map(|s| s.inferred.name.clone())
        };

        let b = w2_env("b", &[("f", "FromB"), ("g", "FromBg")], vec![]);
        let c = w2_env("c", &[("f", "FromC")], vec![]);
        let m = w2_env("m", &[], vec![b.clone(), c.clone()]);
        assert_eq!(
            read(&m, "f").as_deref(),
            Some("FromC"),
            "last direct import wins"
        );
        let flat_names: Vec<String> = m.parents.iter().map(|p| p.name.clone()).collect();
        assert_eq!(flat_names, vec!["c".to_string(), "b".to_string()]);

        let d = w2_env("d", &[("g", "FromD")], vec![]);
        let c2 = w2_env("c2", &[], vec![d]);
        let m2 = w2_env("m2", &[], vec![b.clone(), c2]);
        assert_eq!(
            read(&m2, "g").as_deref(),
            Some("FromD"),
            "closure of the last import must beat the same name directly in an earlier import (deep-first order)"
        );

        let own = w2_env("own", &[("f", "FromSelf")], vec![b, c]);
        assert_eq!(
            read(&own, "f").as_deref(),
            Some("FromSelf"),
            "own local shadows all parents"
        );
    }

    // The .dag-level constructor threads the invariant: resolve_func_sigs
    // flattens its direct parent envs before the topo loop stores them.
    #[test]
    fn resolve_func_sigs_stores_flat_named_parents() {
        let b = w2_env("dep.b", &[("f", "FromB")], vec![]);
        let c = w2_env("dep.c", &[], vec![b.clone()]);
        let result = crate::v1_compiler_infer_sigs::resolve_func_sigs(
            crate::v1_rt::rc_empty_map(),
            Rc::new([b, c].into_iter().collect()),
            Rc::new(im::vector![]),
            "top.module".to_string(),
            crate::v1_rt::rc_empty_map(),
        );
        assert_eq!(result.func_env.name, "top.module");
        let flat_names: Vec<String> = result
            .func_env
            .parents
            .iter()
            .map(|p| p.name.clone())
            .collect();
        assert_eq!(
            flat_names,
            vec!["dep.c".to_string(), "dep.b".to_string()],
            "resolve_func_sigs must store the flattened, deduped closure (c's closure contains b; dedup keeps the first occurrence)"
        );
        assert!(
            result.func_env.parents.iter().all(|p| !p.name.is_empty()),
            "every closure member carries its module name (the dedup key)"
        );
    }
}

#[cfg(test)]
mod compile_clean_loader_closure_fork_regression {
    // Regression for the §3 closure-authority fork dissolved 2026-07-20: the
    // compile-clean gate loader `load_compile_clean_entry_sources` ran ONLY
    // `extend_with_reference_closure` (module-path refs), while the witness loader
    // `load_sources_for_entry_with_pool` ran BOTH that and
    // `extend_with_bare_reference_closure`. The service-name → provider edge
    // (`gcp.STS` → dag/extdeps/cloud/gcp/sts.dag) and bare-name provider pulls
    // live ONLY in the bare closure, so an affected entry reaching a provider
    // purely through a service call or bare name (dag/gunbc/auth/patterns.dag →
    // `gcp.STS.Exchange`, zero imports) dropped that provider from the scoped
    // compile set and its names went unresolved. This surfaced non-locally when
    // #6937's import strip made patterns.dag affected. Fix = the gate loader runs
    // the same both-closure fixpoint as the witness loader.
    //
    // Heavyweight (builds the whole-tree index; ~20s) and chdir-global, so it is
    // #[ignore]d like `witness_layer_roots_compile_clean_check` — run explicitly:
    //   cargo test -p v1-compiler --lib compile_clean_loader_closure_fork \
    //     -- --ignored --nocapture --test-threads=1
    use super::*;

    fn hard_diags(sources: &[Rc<v1_compiler_compile::SourceFile>]) -> Vec<String> {
        v1_compiler_compile::compile_sources(
            Rc::new(sources.to_vec().into()),
            crate::v1_compiler_artifact::RenderTarget::Dag,
        )
        .diagnostics
        .iter()
        .filter(|d| compile_clean_diagnostic_is_hard(d))
        .map(|d| crate::v1_std_core::diagnostic_to_message(d.diagnostic.clone()))
        .collect()
    }

    #[test]
    #[ignore = "heavyweight (whole-tree index) + chdir-global; run explicitly"]
    fn scoped_gate_loader_pulls_bare_referenced_providers() {
        std::env::set_current_dir(workspace_root()).expect("chdir workspace root");
        let roots = witness_layer_roots();
        let mei = build_multi_entry_index_primary_precedence(&roots);

        let patterns_rel = "dag/gunbc/auth/patterns.dag".to_string();
        let filter: std::collections::HashSet<String> = [patterns_rel].into_iter().collect();

        // RED control: the OLD ref-only behavior, replicated inline. Resolve the
        // scoped entry + ONLY the module-path reference closure — no bare closure.
        // The service-only provider must be ABSENT and the closure must red.
        let entry_source =
            entry_source_from_index_or_disk(&mei.source_files, "dag/gunbc/auth/patterns.dag")
                .expect("entry source");
        let mut ref_only = resolve_transitively(
            vec![entry_source.clone()],
            &mei.source_files,
            &mei.module_graph_facts,
        )
        .expect("resolve");
        if !ref_only.iter().any(|s| s.path.contains("patterns.dag")) {
            ref_only.push(entry_source);
        }
        let ref_only =
            extend_with_reference_closure(ref_only, &mei.source_files, &mei.module_graph_facts)
                .expect("ref closure");
        let sts_ref_only = ref_only
            .iter()
            .any(|s| s.path.contains("cloud/gcp/sts.dag"));
        let diags_ref_only = hard_diags(&ref_only);
        assert!(
            !sts_ref_only,
            "RED control broken: ref-only closure unexpectedly already contains sts.dag"
        );
        assert!(
            !diags_ref_only.is_empty(),
            "RED control broken: ref-only scoped closure of patterns.dag must produce unresolved-name \
             hard diagnostics (the fork this test guards). Got zero — the discriminating red is gone."
        );

        // The FIX: the real gate loader, now running BOTH closures. The bare
        // closure must pull the service/bare-referenced providers and the scoped
        // compile must be clean.
        let fixed = load_compile_clean_entry_sources(&roots, &mei, Some(&filter))
            .expect("fixed scoped load");
        let sts_fixed = fixed.iter().any(|s| s.path.contains("cloud/gcp/sts.dag"));
        let diags_fixed = hard_diags(&fixed);
        assert!(
            sts_fixed,
            "fix regressed: the both-closure gate loader must pull the service provider sts.dag \
             into patterns.dag's scoped closure"
        );
        assert!(
            diags_fixed.is_empty(),
            "fix regressed: patterns.dag scoped compile must be clean under the both-closure loader, got: {diags_fixed:?}"
        );
    }
}
