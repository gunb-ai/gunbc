//! Floor discovery snapshot materialization (R0).
//!
//! ## Consumer census (coordinated scoped floor worker)
//!
//! | Consumer site | What it reads from the discovery walk | Closed projection |
//! |---|---|---|
//! | Demand-directed `discover_floor_witness_roster([], [])` (runs only when the plan schedules a discovery or scoped-witness batch, gunbc#8140) | `test fn` placement hygiene, producer roster, module-graph facts, effect-reach derivation (the orphan-helper census and the `__`-basename rule were deleted, gunbc#8155) | Full snapshot payload + installed module-graph cache |
//! | Discovery corpus with `scan_dirs=[]` + explicit entries | Skips roster walk; still calls `build_module_graph_facts_live` on selection/skip paths | Module-graph facts bytes in snapshot (cache install) |
//! | Discovery corpus with non-empty `scan_dirs` | Full roster walk for that scan shape | Not covered by pre-plan snapshot (distinct request identity) |
//!
//! Coordinator before scoped spawn: terminal receipt + request identity digest + payload digest.
//!
//! ## Module-graph field census (gunbc Cut 4)
//!
//! `ModuleGraphFactsSnapshot` transports the module-graph facts so the scoped child's
//! `build_module_graph_facts_live` call is a cache hit rather than a second whole-corpus
//! acquisition. A field earns its place in that payload only by having a live reader; the
//! census below is by exact reader, not by plausibility.
//!
//! | Field | Live production reader |
//! |---|---|
//! | `nodes` | `runtime_data_dependency_touched_via_carrier_closure`, `declared_source_refs_axis_for_entry` |
//! | `adjacency` | `import_closure_live_paths_with_facts`, `repo_paths_match_touched`, `collect_sorted_decl_lines_for_file`, `census_exclude_derive::derive_census_exclude_closure` |
//! | `selection_adjacency` | `entry_file_touched_via_import_closure` |
//! | `reference_unaccounted` | `entry_file_touched_via_import_closure` |
//! | `declared_paths` | `declared_repo_paths`, `declares_repo_path` |
//! | `path_to_module` | the discovery witness run loop (an entry with no module identity refuses rather than fabricating a `DeclarationRef`), `declared_source_refs_axis_for_entry` |
//! | `read_refusals` | `refuse_on_module_graph_read_refusals` |
//!
//! Two fields were carried with NO production reader at all and are deleted:
//!
//! - `edges` — consumed only as a local inside `build_module_graph_facts_live_uncached` (it
//!   builds `adjacency` and `selection_adjacency` and is then dead). Its one reader on the
//!   struct was `import_closure_bfs_vs_fixpoint_perf_receipt`, an `#[ignore]`d manual receipt
//!   timing a legacy fixpoint that exists only inside that test; receipt and legacy helper are
//!   deleted with the field. At the corpus's ~17.4k import lines this was the payload's largest
//!   term — one `{path, import_module, target_declared}` row per edge, serialized, digested into
//!   `module_graph_facts_digest`, written by the producer, read and reconstructed by the child.
//! - `observed_paths` — the source inventory (~3.5k paths). Its stated job, distinguishing a
//!   vanished module from an absent one, is done by `read_refusals`, which is what
//!   `refuse_on_module_graph_read_refusals` actually reads; the inventory itself is produced and
//!   asserted on at `ImportResolutionObservation`, which is unchanged. Retaining a second copy on
//!   the facts struct only widened the payload.
//!
//! DECLARED SCOPE LOSS: nothing downstream can ask the facts value for the raw edge list or the
//! inventory any more. Both remain available at the observation, one call above.
//!
//! OPEN, and deliberately not claimed here: the seven surviving fields each have a production
//! reader, which is NOT the same fact as "the coordinated scoped child reads it". That question
//! is a runtime one and is answered by measurement, not by this table.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::io::Write as IoWrite;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::std_content_hash::{
    content_hash_atom, content_hash_combine_structural, fnv1a64_structural_hex_digest,
};
use crate::v1_rt;

pub const FLOOR_ATTEMPTS_DIR: &str = "target/floor-attempts";
pub const FLOOR_DISCOVERY_SNAPSHOT_FILE: &str = "floor-discovery-snapshot.json";
pub const FLOOR_DISCOVERY_TERMINAL_FILE: &str = "floor-discovery-snapshot.terminal";
pub const FLOOR_DISCOVERY_TRACE_FILE: &str = "floor-discovery-trace.tsv";
pub const FLOOR_DISCOVERY_CONSUMER_ENV: &str = "GUNBC_FLOOR_DISCOVERY_CONSUMER";

static COORDINATED_DISCOVERY_COMPUTE_COUNT: AtomicUsize = AtomicUsize::new(0);
static COORDINATED_SNAPSHOT_INSTALLED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);
static IN_PROCESS_ROSTER_BY_REQUEST: OnceLock<Mutex<HashMap<String, Vec<super::DiscoveryRow>>>> =
    OnceLock::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FloorDiscoveryConsumerRole {
    Producer,
    CoordinatedConsumer,
    Standalone,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FloorDiscoveryRequest {
    pub tested_commit: String,
    pub tested_tree: String,
    pub source_roots: Vec<String>,
    pub scan_dirs: Vec<String>,
    pub exclude_substrings: Vec<String>,
    pub discovery_scope_dirs: Vec<String>,
    pub execution_mode: String,
    pub execution_authority_source_roots: Vec<String>,
    pub naming_authority_digest: String,
    pub tool_identity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryRowSnapshot {
    pub label: String,
    pub entry: String,
    pub function: String,
    pub reads_live_tree: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleGraphFactsSnapshot {
    pub nodes: Vec<super::ModuleDeclarationFactRaw>,
    pub adjacency: BTreeMap<String, Vec<String>>,
    pub selection_adjacency: BTreeMap<String, Vec<String>>,
    pub reference_unaccounted: Vec<String>,
    pub path_to_module: BTreeMap<String, String>,
    pub read_refusals: Vec<(String, String)>,
    pub declared_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FloorDiscoverySnapshot {
    pub request: FloorDiscoveryRequest,
    pub request_identity_digest: String,
    pub roster: Vec<DiscoveryRowSnapshot>,
    pub roster_digest: String,
    pub module_graph_facts: ModuleGraphFactsSnapshot,
    pub module_graph_facts_digest: String,
    pub payload_digest: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FloorDiscoveryTerminalReceipt {
    pub tested_commit: String,
    pub tested_tree: String,
    pub walk_attempt_id: String,
    pub request_identity_digest: String,
    pub payload_digest: String,
    pub outcome: String,
}

pub fn coordinated_discovery_compute_count() -> usize {
    COORDINATED_DISCOVERY_COMPUTE_COUNT.load(Ordering::SeqCst)
}

/// A coordinated consumer may enter witness execution only after the exact snapshot
/// has been verified and installed. This is a construction wall for the transitional
/// provider path: absence refuses rather than silently rebuilding a cold world.
pub fn coordinated_snapshot_installed() -> bool {
    COORDINATED_SNAPSHOT_INSTALLED.load(Ordering::SeqCst)
}

#[cfg(test)]
pub fn reset_floor_discovery_snapshot_for_test() {
    COORDINATED_DISCOVERY_COMPUTE_COUNT.store(0, Ordering::SeqCst);
    COORDINATED_SNAPSHOT_INSTALLED.store(false, Ordering::SeqCst);
    if let Some(lock) = IN_PROCESS_ROSTER_BY_REQUEST.get() {
        lock.lock().unwrap().clear();
    }
}

pub fn floor_discovery_consumer_role_from_env() -> FloorDiscoveryConsumerRole {
    match std::env::var(FLOOR_DISCOVERY_CONSUMER_ENV).as_deref() {
        Ok("producer") => FloorDiscoveryConsumerRole::Producer,
        Ok("coordinated_consumer") => FloorDiscoveryConsumerRole::CoordinatedConsumer,
        Ok("standalone") => FloorDiscoveryConsumerRole::Standalone,
        _ => FloorDiscoveryConsumerRole::Standalone,
    }
}

pub fn build_floor_discovery_request(
    source_roots: &[String],
    scan_dirs: &[String],
    exclude_substrings: &[String],
    discovery_scope_dirs: &[String],
    execution_mode: &str,
    execution_authority_source_roots: &[String],
) -> Result<FloorDiscoveryRequest, String> {
    let (tested_commit, tested_tree) = floor_tested_commit_and_tree()?;
    Ok(FloorDiscoveryRequest {
        tested_commit,
        tested_tree,
        source_roots: source_roots.to_vec(),
        scan_dirs: scan_dirs.to_vec(),
        exclude_substrings: exclude_substrings.to_vec(),
        discovery_scope_dirs: discovery_scope_dirs.to_vec(),
        execution_mode: execution_mode.to_string(),
        execution_authority_source_roots: execution_authority_source_roots.to_vec(),
        naming_authority_digest: naming_authority_digest_hex()?,
        tool_identity: floor_tool_identity()?,
    })
}

pub fn request_identity_digest(request: &FloorDiscoveryRequest) -> Result<String, String> {
    let seed = content_hash_atom("floor-discovery-request-identity-v2".to_string());
    let with_commit =
        content_hash_combine_structural(seed, content_hash_atom(request.tested_commit.clone()));
    let mut acc = content_hash_combine_structural(
        with_commit,
        content_hash_atom(request.tested_tree.clone()),
    );
    for root in &request.source_roots {
        acc = content_hash_combine_structural(acc, content_hash_atom(root.clone()));
    }
    for dir in &request.scan_dirs {
        acc = content_hash_combine_structural(acc, content_hash_atom(dir.clone()));
    }
    for ex in &request.exclude_substrings {
        acc = content_hash_combine_structural(acc, content_hash_atom(ex.clone()));
    }
    for dir in &request.discovery_scope_dirs {
        acc = content_hash_combine_structural(acc, content_hash_atom(dir.clone()));
    }
    acc = content_hash_combine_structural(acc, content_hash_atom(request.execution_mode.clone()));
    for root in &request.execution_authority_source_roots {
        acc = content_hash_combine_structural(acc, content_hash_atom(root.clone()));
    }
    let naming_authority = fnv1a64_structural_hex_digest(request.naming_authority_digest.clone())
        .ok_or_else(|| {
            format!(
                "floor naming authority identity `{}` is not a modeled lower-hex Fnv1a64Structural digest",
                request.naming_authority_digest
            )
        })?;
    acc = content_hash_combine_structural(acc, naming_authority);
    let tool_identity =
        fnv1a64_structural_hex_digest(request.tool_identity.clone()).ok_or_else(|| {
            format!(
                "floor tool identity `{}` is not a modeled lower-hex Fnv1a64Structural digest",
                request.tool_identity
            )
        })?;
    acc = content_hash_combine_structural(acc, tool_identity);
    Ok(acc.digest.clone())
}

fn git_rev_parse(spec: &str, coordinate: &str) -> Result<String, String> {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--verify", spec])
        .output()
        .map_err(|e| format!("floor {coordinate} git rev-parse `{spec}`: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "floor {coordinate} git rev-parse `{spec}` failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn floor_git_object_format() -> Result<String, String> {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--show-object-format=storage"])
        .output()
        .map_err(|e| format!("floor git object format observation: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "floor git object format observation failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let format = String::from_utf8_lossy(&output.stdout).trim().to_string();
    match format.as_str() {
        "sha1" | "sha256" => Ok(format),
        _ => Err(format!(
            "floor git object format `{format}` is unsupported; expected sha1 or sha256"
        )),
    }
}

fn validate_git_object_hex(value: &str, format: &str, coordinate: &str) -> Result<(), String> {
    let expected_len = match format {
        "sha1" => 40,
        "sha256" => 64,
        _ => {
            return Err(format!(
                "floor {coordinate} cannot validate unsupported git object format `{format}`"
            ));
        }
    };
    if value.len() != expected_len
        || value != value.to_ascii_lowercase()
        || !value.chars().all(|c| c.is_ascii_hexdigit())
    {
        return Err(format!(
            "floor {coordinate} `{value}` is not a canonical lowercase {format} object id"
        ));
    }
    Ok(())
}

fn floor_tested_commit_and_tree() -> Result<(String, String), String> {
    let object_format = floor_git_object_format()?;
    let selected_commit = match std::env::var("GITHUB_SHA") {
        Ok(sha) => {
            validate_git_object_hex(&sha, &object_format, "tested_commit GITHUB_SHA")?;
            sha
        }
        Err(std::env::VarError::NotPresent) => git_rev_parse("HEAD^{commit}", "tested_commit")?,
        Err(std::env::VarError::NotUnicode(_)) => {
            return Err("floor tested_commit GITHUB_SHA is not Unicode".to_string());
        }
    };
    let canonical_commit =
        git_rev_parse(&format!("{selected_commit}^{{commit}}"), "tested_commit")?;
    validate_git_object_hex(&canonical_commit, &object_format, "tested_commit")?;
    if canonical_commit != selected_commit {
        return Err(format!(
            "floor tested_commit mismatch: selected `{selected_commit}`, canonical `{canonical_commit}`"
        ));
    }
    let tree_hex = git_rev_parse(&format!("{canonical_commit}^{{tree}}"), "tested_tree")?;
    validate_git_object_hex(&tree_hex, &object_format, "tested_tree")?;
    Ok((canonical_commit, format!("{object_format}:{tree_hex}")))
}

fn verify_floor_worktree_matches_subject(request: &FloorDiscoveryRequest) -> Result<(), String> {
    let diff = std::process::Command::new("git")
        .args([
            "diff",
            "--quiet",
            "--no-ext-diff",
            &request.tested_commit,
            "--",
        ])
        .status()
        .map_err(|e| format!("floor exact-subject tracked-worktree observation: {e}"))?;
    match diff.code() {
        Some(0) => {}
        Some(1) => {
            return Err(format!(
                "floor exact-subject refusal: tracked worktree bytes differ from tested commit {}",
                request.tested_commit
            ));
        }
        code => {
            return Err(format!(
                "floor exact-subject tracked-worktree observation failed with status {code:?}"
            ));
        }
    }

    let untracked = std::process::Command::new("git")
        .args(["ls-files", "--others", "--exclude-standard"])
        .output()
        .map_err(|e| format!("floor exact-subject untracked-worktree observation: {e}"))?;
    if !untracked.status.success() {
        return Err(format!(
            "floor exact-subject untracked-worktree observation failed: {}",
            String::from_utf8_lossy(&untracked.stderr)
        ));
    }
    let untracked_paths = String::from_utf8_lossy(&untracked.stdout);
    if let Some(first) = untracked_paths.lines().next() {
        return Err(format!(
            "floor exact-subject refusal: untracked worktree path `{first}` is not named by tested tree {}",
            request.tested_tree
        ));
    }
    Ok(())
}

const NAMING_AUTHORITY_PATHS: &[&str] = &[
    "src/v2/workflow/floor_naming_hygiene.dag",
    "src/v2/workflow/floor_discovery_producer.dag",
    "src/v2/workflow/floor_discovery.dag",
    "dag/gunbc/test_module_hygiene.dag",
    "src/v1/stage0/src/cli_run/test_module_hygiene_bridge.rs",
];

fn naming_authority_digest_hex() -> Result<String, String> {
    let workspace = super::workspace_root();
    let mut digest = content_hash_atom("floor-discovery-naming-authority-v1".to_string());
    for rel in NAMING_AUTHORITY_PATHS {
        let path = workspace.join(rel);
        let bytes = fs::read(&path)
            .map_err(|e| format!("naming authority read {}: {e}", path.display()))?;
        digest = content_hash_combine_structural(
            digest,
            content_hash_atom(format!("{}:{}", rel, v1_rt::bytes_identity_hash(&bytes))),
        );
    }
    Ok(digest.digest.clone())
}

fn floor_tool_identity() -> Result<String, String> {
    let exe =
        std::env::current_exe().map_err(|e| format!("floor tool_identity current_exe: {e}"))?;
    let bytes =
        fs::read(&exe).map_err(|e| format!("floor tool_identity read {}: {e}", exe.display()))?;
    Ok(v1_rt::bytes_identity_hash(&bytes))
}

fn snapshot_dir(walk_attempt_id: &str) -> PathBuf {
    super::workspace_root()
        .join(FLOOR_ATTEMPTS_DIR)
        .join(walk_attempt_id)
}

fn snapshot_json_path(walk_attempt_id: &str) -> PathBuf {
    snapshot_dir(walk_attempt_id).join(FLOOR_DISCOVERY_SNAPSHOT_FILE)
}

fn snapshot_terminal_path(walk_attempt_id: &str) -> PathBuf {
    snapshot_dir(walk_attempt_id).join(FLOOR_DISCOVERY_TERMINAL_FILE)
}

fn trace_path(walk_attempt_id: &str) -> PathBuf {
    snapshot_dir(walk_attempt_id).join(FLOOR_DISCOVERY_TRACE_FILE)
}

/// Wall-clock origin for a trace row, in milliseconds since the Unix epoch.
///
/// `started_ms` and `completed_ms` must be comparable ACROSS rows and across the
/// producer/consumer process boundary, so they cannot be derived from a per-call
/// `Instant`: a monotonic origin is only meaningful relative to itself. The row's
/// duration is carried separately by `wall_ns`.
///
/// `0` is the unreadable-clock sentinel (system time at or before the epoch). It is
/// deliberately an implausible timestamp rather than a plausible one, and it is
/// observational telemetry only — no gate reads this value.
fn trace_epoch_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

pub fn append_discovery_trace_row(
    walk_attempt_id: &str,
    computation_id: &str,
    producer_consumer: &str,
    placement: &str,
    materialization_decision: &str,
    started_ms: u128,
    completed_ms: u128,
    payload_digest: &str,
    outcome: &str,
    wall_ns: u128,
) {
    let path = trace_path(walk_attempt_id);
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let header_needed = !path.is_file();
    let mut file = match fs::OpenOptions::new().create(true).append(true).open(&path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("floor_discovery_snapshot: trace open refused: {e}");
            return;
        }
    };
    if header_needed {
        let _ = file.write_all(
            b"computation_id\tproducer_consumer\tplacement\tmaterialization_decision\tstarted_ms\tcompleted_ms\tpayload_digest\toutcome\twall_ns\n",
        );
    }
    let row = format!(
        "{computation_id}\t{producer_consumer}\t{placement}\t{materialization_decision}\t{started_ms}\t{completed_ms}\t{payload_digest}\t{outcome}\t{wall_ns}\n"
    );
    let _ = file.write_all(row.as_bytes()).and_then(|()| file.flush());
}

fn facts_to_snapshot(facts: &super::ModuleGraphFactsLive) -> ModuleGraphFactsSnapshot {
    ModuleGraphFactsSnapshot {
        nodes: facts.nodes.clone(),
        adjacency: facts
            .adjacency
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect(),
        selection_adjacency: facts
            .selection_adjacency
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect(),
        reference_unaccounted: facts.reference_unaccounted.iter().cloned().collect(),
        path_to_module: facts
            .path_to_module
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect(),
        read_refusals: facts.read_refusals.clone(),
        declared_paths: facts.declared_paths.iter().cloned().collect(),
    }
}

fn snapshot_to_facts(snapshot: &ModuleGraphFactsSnapshot) -> super::ModuleGraphFactsLive {
    super::ModuleGraphFactsLive {
        nodes: snapshot.nodes.clone(),
        adjacency: snapshot
            .adjacency
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect(),
        selection_adjacency: snapshot
            .selection_adjacency
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect(),
        reference_unaccounted: snapshot.reference_unaccounted.iter().cloned().collect(),
        path_to_module: snapshot
            .path_to_module
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect(),
        read_refusals: snapshot.read_refusals.clone(),
        declared_paths: snapshot.declared_paths.iter().cloned().collect(),
    }
}

fn digest_rows(rows: &[DiscoveryRowSnapshot]) -> String {
    let mut acc = content_hash_atom("floor-discovery-roster-v1".to_string());
    for row in rows {
        acc = content_hash_combine_structural(acc, content_hash_atom(row.entry.clone()));
        acc = content_hash_combine_structural(acc, content_hash_atom(row.function.clone()));
        acc = content_hash_combine_structural(
            acc,
            content_hash_atom(format!("live={}", row.reads_live_tree)),
        );
    }
    acc.digest.clone()
}

fn digest_facts_snapshot(facts: &ModuleGraphFactsSnapshot) -> String {
    let json = serde_json::to_string(facts).unwrap_or_default();
    content_hash_atom(json).digest.clone()
}

#[derive(Serialize)]
struct FloorDiscoveryPayloadDigestView<'a> {
    request: &'a FloorDiscoveryRequest,
    request_identity_digest: &'a str,
    roster: &'a [DiscoveryRowSnapshot],
    roster_digest: &'a str,
    module_graph_facts: &'a ModuleGraphFactsSnapshot,
    module_graph_facts_digest: &'a str,
}

fn digest_payload(snapshot: &FloorDiscoverySnapshot) -> String {
    let payload = FloorDiscoveryPayloadDigestView {
        request: &snapshot.request,
        request_identity_digest: &snapshot.request_identity_digest,
        roster: &snapshot.roster,
        roster_digest: &snapshot.roster_digest,
        module_graph_facts: &snapshot.module_graph_facts,
        module_graph_facts_digest: &snapshot.module_graph_facts_digest,
    };
    let json = serde_json::to_string(&payload)
        .expect("floor discovery payload view contains only infallibly serializable fields");
    content_hash_atom(json).digest.clone()
}

fn verify_snapshot_payload_integrity(snapshot: &FloorDiscoverySnapshot) -> Result<(), String> {
    let observed = digest_payload(snapshot);
    if observed != snapshot.payload_digest {
        return Err(format!(
            "floor discovery snapshot payload digest mismatch: recorded {}, observed {observed}",
            snapshot.payload_digest
        ));
    }
    Ok(())
}

fn row_to_snapshot(row: &super::DiscoveryRow) -> DiscoveryRowSnapshot {
    DiscoveryRowSnapshot {
        label: row.label.clone(),
        entry: row.entry.clone(),
        function: row.function.clone(),
        reads_live_tree: row.reads_live_tree,
    }
}

fn snapshot_to_row(row: &DiscoveryRowSnapshot) -> super::DiscoveryRow {
    super::DiscoveryRow {
        label: row.label.clone(),
        entry: row.entry.clone(),
        function: row.function.clone(),
        reads_live_tree: row.reads_live_tree,
    }
}

pub fn install_module_graph_facts_cache(
    source_roots: &[String],
    facts: &super::ModuleGraphFactsLive,
) {
    super::install_module_graph_facts_cache_entry(source_roots, facts.clone());
}

fn install_roster_cache(request_digest: &str, rows: &[super::DiscoveryRow]) {
    IN_PROCESS_ROSTER_BY_REQUEST
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .unwrap()
        .insert(request_digest.to_string(), rows.to_vec());
}

pub fn produce_floor_discovery_snapshot(
    request: &FloorDiscoveryRequest,
) -> Result<FloorDiscoverySnapshot, String> {
    COORDINATED_DISCOVERY_COMPUTE_COUNT.fetch_add(1, Ordering::SeqCst);
    let mut rows = super::invoke_floor_discovery_producer(
        &request.source_roots,
        &request.scan_dirs,
        &request.exclude_substrings,
    )?;
    rows = super::apply_discovery_scope_dirs_filter(rows, &request.discovery_scope_dirs);
    let facts = super::build_module_graph_facts_live_uncached(&request.source_roots);
    super::refuse_on_module_graph_read_refusals(&facts)?;
    super::apply_effect_reach_derived_reads_live_tree(&mut rows, &facts);
    let roster: Vec<DiscoveryRowSnapshot> = rows.iter().map(row_to_snapshot).collect();
    let facts_snapshot = facts_to_snapshot(&facts);
    let request_identity_digest = request_identity_digest(request)?;
    let roster_digest = digest_rows(&roster);
    let module_graph_facts_digest = digest_facts_snapshot(&facts_snapshot);
    let snapshot = FloorDiscoverySnapshot {
        request: request.clone(),
        request_identity_digest,
        roster,
        roster_digest,
        module_graph_facts: facts_snapshot,
        module_graph_facts_digest,
        payload_digest: String::new(),
    };
    let payload_digest = digest_payload(&snapshot);
    Ok(FloorDiscoverySnapshot {
        payload_digest,
        ..snapshot
    })
}

pub fn publish_floor_discovery_snapshot(
    walk_attempt_id: &str,
    snapshot: &FloorDiscoverySnapshot,
) -> Result<(), String> {
    let dir = snapshot_dir(walk_attempt_id);
    fs::create_dir_all(&dir)
        .map_err(|e| format!("create snapshot directory {}: {e}", dir.display()))?;
    let json_path = snapshot_json_path(walk_attempt_id);
    let json = serde_json::to_string_pretty(snapshot)
        .map_err(|e| format!("serialize floor discovery snapshot: {e}"))?;
    fs::write(&json_path, json)
        .map_err(|e| format!("write snapshot {}: {e}", json_path.display()))?;
    let terminal = FloorDiscoveryTerminalReceipt {
        tested_commit: snapshot.request.tested_commit.clone(),
        tested_tree: snapshot.request.tested_tree.clone(),
        walk_attempt_id: walk_attempt_id.to_string(),
        request_identity_digest: snapshot.request_identity_digest.clone(),
        payload_digest: snapshot.payload_digest.clone(),
        outcome: "Published".to_string(),
    };
    let terminal_path = snapshot_terminal_path(walk_attempt_id);
    let terminal_json = serde_json::to_string(&terminal)
        .map_err(|e| format!("serialize snapshot terminal: {e}"))?;
    fs::write(&terminal_path, terminal_json)
        .map_err(|e| format!("write terminal {}: {e}", terminal_path.display()))?;
    Ok(())
}

pub fn verify_floor_discovery_terminal_for_coordinator(
    walk_attempt_id: &str,
    expected_request: &FloorDiscoveryRequest,
) -> Result<String, String> {
    verify_floor_worktree_matches_subject(expected_request)?;
    let expected_request_digest = request_identity_digest(expected_request)?;
    let terminal_path = snapshot_terminal_path(walk_attempt_id);
    let bytes = fs::read_to_string(&terminal_path).map_err(|e| {
        format!(
            "coordinator snapshot terminal missing {}: {e}",
            terminal_path.display()
        )
    })?;
    let terminal: FloorDiscoveryTerminalReceipt = serde_json::from_str(&bytes).map_err(|e| {
        format!(
            "coordinator snapshot terminal parse {}: {e}",
            terminal_path.display()
        )
    })?;
    if terminal.walk_attempt_id != walk_attempt_id {
        return Err(format!(
            "coordinator snapshot terminal walk_attempt_id mismatch: expected {walk_attempt_id}, got {}",
            terminal.walk_attempt_id
        ));
    }
    if terminal.tested_commit != expected_request.tested_commit {
        return Err(format!(
            "coordinator snapshot terminal tested_commit mismatch: expected {}, got {}",
            expected_request.tested_commit, terminal.tested_commit
        ));
    }
    if terminal.tested_tree != expected_request.tested_tree {
        return Err(format!(
            "coordinator snapshot terminal tested_tree mismatch: expected {}, got {}",
            expected_request.tested_tree, terminal.tested_tree
        ));
    }
    if terminal.request_identity_digest != expected_request_digest {
        return Err(format!(
            "coordinator snapshot terminal request_identity_digest mismatch: expected {expected_request_digest}, got {}",
            terminal.request_identity_digest
        ));
    }
    if terminal.outcome != "Published" {
        return Err(format!(
            "coordinator snapshot terminal outcome `{}`, expected Published",
            terminal.outcome
        ));
    }
    let json_path = snapshot_json_path(walk_attempt_id);
    let snapshot_bytes = fs::read_to_string(&json_path).map_err(|e| {
        format!(
            "coordinator snapshot payload missing {}: {e}",
            json_path.display()
        )
    })?;
    let snapshot: FloorDiscoverySnapshot = serde_json::from_str(&snapshot_bytes)
        .map_err(|e| format!("coordinator snapshot payload parse: {e}"))?;
    if snapshot.request != *expected_request {
        return Err(
            "coordinator snapshot payload request subject mismatch — recompute refused".to_string(),
        );
    }
    verify_snapshot_payload_integrity(&snapshot)?;
    if snapshot.payload_digest != terminal.payload_digest {
        return Err(
            "coordinator snapshot payload_digest does not match terminal receipt".to_string(),
        );
    }
    if snapshot.request_identity_digest != expected_request_digest {
        return Err("coordinator snapshot payload request_identity_digest mismatch".to_string());
    }
    Ok(terminal.payload_digest.clone())
}

pub fn load_floor_discovery_snapshot_coordinated(
    walk_attempt_id: &str,
    request: &FloorDiscoveryRequest,
) -> Result<FloorDiscoverySnapshot, String> {
    let expected_digest = request_identity_digest(request)?;
    let json_path = snapshot_json_path(walk_attempt_id);
    let bytes = fs::read_to_string(&json_path).map_err(|_| {
        format!(
            "coordinated floor discovery consumer: snapshot absent at {} — recompute refused",
            json_path.display()
        )
    })?;
    let snapshot: FloorDiscoverySnapshot = serde_json::from_str(&bytes)
        .map_err(|e| format!("coordinated consumer snapshot parse: {e}"))?;
    if snapshot.request != *request {
        return Err(
            "coordinated floor discovery consumer: snapshot request subject mismatch — recompute refused"
                .to_string(),
        );
    }
    if snapshot.request_identity_digest != expected_digest {
        return Err(
            "coordinated floor discovery consumer: snapshot identity digest mismatch — recompute refused"
                .to_string(),
        );
    }
    verify_snapshot_payload_integrity(&snapshot)
        .map_err(|e| format!("coordinated floor discovery consumer: {e} — recompute refused"))?;
    Ok(snapshot)
}

pub fn install_floor_discovery_snapshot(snapshot: &FloorDiscoverySnapshot) {
    let facts = snapshot_to_facts(&snapshot.module_graph_facts);
    install_module_graph_facts_cache(&snapshot.request.source_roots, &facts);
    let rows: Vec<super::DiscoveryRow> = snapshot.roster.iter().map(snapshot_to_row).collect();
    install_roster_cache(&snapshot.request_identity_digest, &rows);
    COORDINATED_SNAPSHOT_INSTALLED.store(true, Ordering::SeqCst);
}

pub fn discover_floor_witness_roster_with_snapshot(
    source_roots: &[String],
    scan_dirs: &[String],
    exclude_substrings: &[String],
    discovery_scope_dirs: &[String],
    walk_attempt_id: &str,
    consumer: FloorDiscoveryConsumerRole,
    execution_mode: &str,
    execution_authority_source_roots: &[String],
) -> Result<Vec<super::DiscoveryRow>, String> {
    let request = build_floor_discovery_request(
        source_roots,
        scan_dirs,
        exclude_substrings,
        discovery_scope_dirs,
        execution_mode,
        execution_authority_source_roots,
    )?;
    if consumer != FloorDiscoveryConsumerRole::Standalone {
        verify_floor_worktree_matches_subject(&request)?;
    }
    let request_digest = request_identity_digest(&request)?;
    let started = Instant::now();
    let started_ms = trace_epoch_millis();

    match consumer {
        FloorDiscoveryConsumerRole::CoordinatedConsumer => {
            let snapshot = load_floor_discovery_snapshot_coordinated(walk_attempt_id, &request)?;
            install_floor_discovery_snapshot(&snapshot);
            let rows: Vec<super::DiscoveryRow> =
                snapshot.roster.iter().map(snapshot_to_row).collect();
            let completed_ms = trace_epoch_millis();
            append_discovery_trace_row(
                walk_attempt_id,
                "discover_floor_witness_roster",
                "coordinated_consumer",
                "LocalFilesystem",
                "Share",
                started_ms,
                completed_ms,
                &snapshot.payload_digest,
                "VerifiedSnapshot",
                started.elapsed().as_nanos(),
            );
            return Ok(rows);
        }
        FloorDiscoveryConsumerRole::Producer | FloorDiscoveryConsumerRole::Standalone => {
            if let Some(lock) = IN_PROCESS_ROSTER_BY_REQUEST.get() {
                if let Some(rows) = lock.lock().unwrap().get(&request_digest) {
                    let completed_ms = trace_epoch_millis();
                    append_discovery_trace_row(
                        walk_attempt_id,
                        "discover_floor_witness_roster",
                        match consumer {
                            FloorDiscoveryConsumerRole::Producer => "producer",
                            FloorDiscoveryConsumerRole::Standalone => "standalone",
                            FloorDiscoveryConsumerRole::CoordinatedConsumer => {
                                "coordinated_consumer"
                            }
                        },
                        "LocalInProcess",
                        "Share",
                        started_ms,
                        completed_ms,
                        &request_digest,
                        "InProcessHit",
                        started.elapsed().as_nanos(),
                    );
                    return Ok(rows.clone());
                }
            }
        }
    }

    let snapshot = produce_floor_discovery_snapshot(&request)?;
    install_floor_discovery_snapshot(&snapshot);
    let rows: Vec<super::DiscoveryRow> = snapshot.roster.iter().map(snapshot_to_row).collect();
    if consumer == FloorDiscoveryConsumerRole::Producer {
        publish_floor_discovery_snapshot(walk_attempt_id, &snapshot)?;
    }
    let completed_ms = trace_epoch_millis();
    append_discovery_trace_row(
        walk_attempt_id,
        "discover_floor_witness_roster",
        match consumer {
            FloorDiscoveryConsumerRole::Producer => "producer",
            FloorDiscoveryConsumerRole::Standalone => "standalone",
            FloorDiscoveryConsumerRole::CoordinatedConsumer => "coordinated_consumer",
        },
        "LocalInProcess",
        "Recompute",
        started_ms,
        completed_ms,
        &snapshot.payload_digest,
        "Computed",
        started.elapsed().as_nanos(),
    );
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal_snapshot(request: FloorDiscoveryRequest) -> FloorDiscoverySnapshot {
        let facts = ModuleGraphFactsSnapshot {
            nodes: vec![],
            adjacency: BTreeMap::new(),
            selection_adjacency: BTreeMap::new(),
            reference_unaccounted: vec![],
            path_to_module: BTreeMap::new(),
            read_refusals: vec![],
            declared_paths: vec![],
        };
        let mut snapshot = FloorDiscoverySnapshot {
            request_identity_digest: request_identity_digest(&request).expect("request identity"),
            request,
            roster: vec![],
            roster_digest: digest_rows(&[]),
            module_graph_facts_digest: digest_facts_snapshot(&facts),
            module_graph_facts: facts,
            payload_digest: String::new(),
        };
        snapshot.payload_digest = digest_payload(&snapshot);
        snapshot
    }

    #[test]
    fn tested_commit_and_tree_are_distinct_exact_git_coordinates() {
        let (commit, tree) = floor_tested_commit_and_tree().expect("exact subject");
        let format = floor_git_object_format().expect("git object format");
        let expected_commit = git_rev_parse("HEAD^{commit}", "test commit").expect("HEAD commit");
        let expected_tree =
            git_rev_parse(&format!("{expected_commit}^{{tree}}"), "test tree").expect("HEAD tree");
        assert_eq!(commit, expected_commit);
        assert_eq!(tree, format!("{format}:{expected_tree}"));
        assert_ne!(
            commit, expected_tree,
            "a commit id is not its tree object id"
        );
    }

    #[test]
    fn request_identity_binds_commit_and_tree_independently() {
        let request = build_floor_discovery_request(
            &["dag".to_string()],
            &[],
            &[],
            &[],
            "Hermetic",
            &["dag".to_string()],
        )
        .expect("request");
        let canonical = request_identity_digest(&request).expect("canonical request identity");
        let changed_commit = FloorDiscoveryRequest {
            tested_commit: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
            ..request.clone()
        };
        let changed_tree = FloorDiscoveryRequest {
            tested_tree: "sha1:1111111111111111111111111111111111111111".to_string(),
            ..request
        };
        assert_ne!(
            canonical,
            request_identity_digest(&changed_commit).expect("changed commit identity")
        );
        assert_ne!(
            canonical,
            request_identity_digest(&changed_tree).expect("changed tree identity")
        );
        assert_ne!(
            request_identity_digest(&changed_commit).expect("changed commit identity"),
            request_identity_digest(&changed_tree).expect("changed tree identity")
        );
    }

    #[test]
    fn request_identity_matches_modeled_golden() {
        let request = FloorDiscoveryRequest {
            tested_commit: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
            tested_tree: "sha1:1111111111111111111111111111111111111111".to_string(),
            source_roots: vec!["dag".to_string(), "src/v1".to_string()],
            scan_dirs: vec!["dag/test/claim".to_string()],
            exclude_substrings: vec!["/long/".to_string()],
            discovery_scope_dirs: vec!["dag/test/claim".to_string()],
            execution_mode: "Hermetic".to_string(),
            execution_authority_source_roots: vec!["dag".to_string(), "src/v2".to_string()],
            naming_authority_digest: content_hash_atom(
                "floor-discovery-naming-authority-fixture".to_string(),
            )
            .digest
            .clone(),
            tool_identity: content_hash_atom("floor-discovery-tool-fixture".to_string())
                .digest
                .clone(),
        };
        assert_eq!(
            request_identity_digest(&request).expect("request identity"),
            "f8e90d00ba0ad7f8"
        );
    }

    #[test]
    fn coordinated_consumer_refuses_absent_snapshot() {
        reset_floor_discovery_snapshot_for_test();
        let request = FloorDiscoveryRequest {
            tested_commit: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
            tested_tree: "sha1:1111111111111111111111111111111111111111".to_string(),
            source_roots: vec!["dag".to_string()],
            scan_dirs: vec![],
            exclude_substrings: vec![],
            discovery_scope_dirs: vec![],
            execution_mode: "Hermetic".to_string(),
            execution_authority_source_roots: vec!["dag".to_string()],
            naming_authority_digest: "0123456789abcdef".to_string(),
            tool_identity: "fedcba9876543210".to_string(),
        };
        assert!(
            load_floor_discovery_snapshot_coordinated("attempt-a", &request).is_err(),
            "absent snapshot must refuse"
        );
    }

    #[test]
    fn coordinated_consumer_refuses_subject_mismatch() {
        reset_floor_discovery_snapshot_for_test();
        let request = build_floor_discovery_request(
            &["dag".to_string()],
            &[],
            &[],
            &[],
            "Hermetic",
            &["dag".to_string()],
        )
        .expect("request");
        let mut snapshot = FloorDiscoverySnapshot {
            request: request.clone(),
            request_identity_digest: request_identity_digest(&request).expect("request identity"),
            roster: vec![],
            roster_digest: digest_rows(&[]),
            module_graph_facts: ModuleGraphFactsSnapshot {
                nodes: vec![],
                adjacency: BTreeMap::new(),
                selection_adjacency: BTreeMap::new(),
                reference_unaccounted: vec![],
                path_to_module: BTreeMap::new(),
                read_refusals: vec![],
                declared_paths: vec![],
            },
            module_graph_facts_digest: digest_facts_snapshot(&ModuleGraphFactsSnapshot {
                nodes: vec![],
                adjacency: BTreeMap::new(),
                selection_adjacency: BTreeMap::new(),
                reference_unaccounted: vec![],
                path_to_module: BTreeMap::new(),
                read_refusals: vec![],
                declared_paths: vec![],
            }),
            payload_digest: String::new(),
        };
        snapshot.payload_digest = digest_payload(&snapshot);
        publish_floor_discovery_snapshot("attempt-b", &snapshot).expect("publish");
        let mismatched = FloorDiscoveryRequest {
            source_roots: vec!["src/v2".to_string()],
            ..request
        };
        assert!(
            load_floor_discovery_snapshot_coordinated("attempt-b", &mismatched).is_err(),
            "subject mismatch must refuse"
        );
    }

    #[test]
    fn coordinated_consumer_consumes_published_minimal_snapshot() {
        reset_floor_discovery_snapshot_for_test();
        let request = build_floor_discovery_request(
            &["dag".to_string()],
            &[],
            &[],
            &[],
            "Hermetic",
            &["dag".to_string()],
        )
        .expect("request");
        let snapshot = FloorDiscoverySnapshot {
            request: request.clone(),
            request_identity_digest: request_identity_digest(&request).expect("request identity"),
            roster: vec![],
            roster_digest: digest_rows(&[]),
            module_graph_facts: ModuleGraphFactsSnapshot {
                nodes: vec![],
                adjacency: BTreeMap::new(),
                selection_adjacency: BTreeMap::new(),
                reference_unaccounted: vec![],
                path_to_module: BTreeMap::new(),
                read_refusals: vec![],
                declared_paths: vec![],
            },
            module_graph_facts_digest: digest_facts_snapshot(&ModuleGraphFactsSnapshot {
                nodes: vec![],
                adjacency: BTreeMap::new(),
                selection_adjacency: BTreeMap::new(),
                reference_unaccounted: vec![],
                path_to_module: BTreeMap::new(),
                read_refusals: vec![],
                declared_paths: vec![],
            }),
            payload_digest: String::new(),
        };
        let mut snapshot = snapshot;
        snapshot.payload_digest = digest_payload(&snapshot);
        publish_floor_discovery_snapshot("attempt-c", &snapshot).expect("publish");
        let loaded = load_floor_discovery_snapshot_coordinated("attempt-c", &request)
            .expect("coordinated consumer must load published snapshot");
        install_floor_discovery_snapshot(&loaded);
        let rows: Vec<_> = loaded.roster.iter().map(snapshot_to_row).collect();
        assert!(rows.is_empty());
        assert_eq!(
            coordinated_discovery_compute_count(),
            0,
            "coordinated consumer must not invoke produce_floor_discovery_snapshot"
        );
    }

    #[test]
    fn coordinated_consumer_refuses_damaged_payload_without_recompute() {
        reset_floor_discovery_snapshot_for_test();
        let request = build_floor_discovery_request(
            &["dag".to_string()],
            &[],
            &[],
            &[],
            "Hermetic",
            &["dag".to_string()],
        )
        .expect("request");
        let snapshot = minimal_snapshot(request.clone());
        publish_floor_discovery_snapshot("attempt-d", &snapshot).expect("publish");

        let path = snapshot_json_path("attempt-d");
        let mut damaged: FloorDiscoverySnapshot =
            serde_json::from_str(&fs::read_to_string(&path).expect("read snapshot"))
                .expect("decode snapshot");
        damaged.roster.push(DiscoveryRowSnapshot {
            label: "damaged".to_string(),
            entry: "dag/test/claim/damaged_test.dag".to_string(),
            function: "damaged".to_string(),
            reads_live_tree: false,
        });
        fs::write(
            &path,
            serde_json::to_string_pretty(&damaged).expect("encode damaged snapshot"),
        )
        .expect("write damaged snapshot");

        let error = load_floor_discovery_snapshot_coordinated("attempt-d", &request)
            .expect_err("damaged payload must refuse");
        assert!(error.contains("payload digest mismatch"), "{error}");
        assert_eq!(
            coordinated_discovery_compute_count(),
            0,
            "damage must never activate a cold reconstruction"
        );
    }
}
