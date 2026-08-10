//! Floor discovery snapshot materialization (R0).
//!
//! ## Consumer census (coordinated scoped floor worker)
//!
//! | Consumer site | What it reads from the discovery walk | Closed projection |
//! |---|---|---|
//! | Pre-plan `discover_floor_witness_roster([], [])` | Naming hygiene, orphan/helpers, producer roster, module-graph facts, inert-lens + construction-justification gates | Full snapshot payload + installed module-graph cache |
//! | Discovery corpus with `scan_dirs=[]` + explicit entries | Skips roster walk; still calls `build_module_graph_facts_live` on selection/skip paths | Module-graph facts bytes in snapshot (cache install) |
//! | Discovery corpus with non-empty `scan_dirs` | Full roster walk for that scan shape | Not covered by pre-plan snapshot (distinct request identity) |
//!
//! Coordinator before scoped spawn: terminal receipt + request identity digest + payload digest.

use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::io::Write as IoWrite;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::std_content_hash::{content_hash_atom, content_hash_combine_structural};
use crate::v1_rt;

pub const FLOOR_ATTEMPTS_DIR: &str = "target/floor-attempts";
pub const FLOOR_DISCOVERY_SNAPSHOT_FILE: &str = "floor-discovery-snapshot.json";
pub const FLOOR_DISCOVERY_TERMINAL_FILE: &str = "floor-discovery-snapshot.terminal";
pub const FLOOR_DISCOVERY_TRACE_FILE: &str = "floor-discovery-trace.tsv";
pub const FLOOR_DISCOVERY_CONSUMER_ENV: &str = "GUNBC_FLOOR_DISCOVERY_CONSUMER";

static COORDINATED_DISCOVERY_COMPUTE_COUNT: AtomicUsize = AtomicUsize::new(0);
static IN_PROCESS_ROSTER_BY_REQUEST: OnceLock<Mutex<HashMap<String, Vec<super::DiscoveryRow>>>> =
    OnceLock::new();
static SUCCESSFUL_TOOL_IDENTITY: OnceLock<String> = OnceLock::new();

thread_local! {
    /// The identity installed by the producer/consumer boundary in this process.
    /// Prepared execution may only bind to this value; a caller cannot manufacture a
    /// request digest beside unrelated bytes and call it an observed discovery subject.
    static CURRENT_FLOOR_DISCOVERY_IDENTITY: RefCell<Option<FloorDiscoveryIdentity>> =
        const { RefCell::new(None) };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FloorDiscoveryConsumerRole {
    Producer,
    CoordinatedConsumer,
    Standalone,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FloorDiscoveryRequest {
    /// Commit whose source bytes are being tested.
    pub tested_commit: String,
    /// Git tree object for `tested_commit` (not the commit id under a misleading name).
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
    pub edges: Vec<super::ImportResolutionFactRaw>,
    pub nodes: Vec<super::ModuleDeclarationFactRaw>,
    pub adjacency: BTreeMap<String, Vec<String>>,
    pub selection_adjacency: BTreeMap<String, Vec<String>>,
    pub reference_unaccounted: Vec<String>,
    pub path_to_module: BTreeMap<String, String>,
    pub observed_paths: Vec<String>,
    pub read_refusals: Vec<(String, String)>,
    pub declared_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FloorDiscoverySnapshot {
    pub request: FloorDiscoveryRequest,
    pub request_identity_digest: String,
    pub roster: Vec<DiscoveryRowSnapshot>,
    pub roster_digest: String,
    pub naming_hygiene_refusal: Option<String>,
    pub orphan_helper_refusal: Option<String>,
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

/// Content identity handed from the discovery boundary to prepared execution.
/// Every field is observed or recomputed while installing the snapshot; this is a
/// projection of verified evidence, not a second caller-authored subject label.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FloorDiscoveryIdentity {
    pub tested_commit: String,
    pub tested_tree: String,
    pub request_identity_digest: String,
    pub roster_digest: String,
    pub module_graph_facts_digest: String,
    pub payload_digest: String,
    pub tool_identity: String,
}

pub fn coordinated_discovery_compute_count() -> usize {
    COORDINATED_DISCOVERY_COMPUTE_COUNT.load(Ordering::SeqCst)
}

#[cfg(test)]
pub fn reset_floor_discovery_snapshot_for_test() {
    COORDINATED_DISCOVERY_COMPUTE_COUNT.store(0, Ordering::SeqCst);
    if let Some(lock) = IN_PROCESS_ROSTER_BY_REQUEST.get() {
        lock.lock().unwrap().clear();
    }
    CURRENT_FLOOR_DISCOVERY_IDENTITY.with(|slot| *slot.borrow_mut() = None);
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
    let tested_commit = floor_tested_commit(None)?;
    let tested_tree = floor_tested_tree(&tested_commit)?;
    verify_source_roots_match_tested_commit(&tested_commit, source_roots, None)?;
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

pub fn request_identity_digest(request: &FloorDiscoveryRequest) -> String {
    let mut acc = content_hash_atom("floor-discovery-request-identity-v3".to_string());
    for (field, value) in [
        ("tested_commit", request.tested_commit.as_str()),
        ("tested_tree", request.tested_tree.as_str()),
        ("execution_mode", request.execution_mode.as_str()),
        (
            "naming_authority_digest",
            request.naming_authority_digest.as_str(),
        ),
        ("tool_identity", request.tool_identity.as_str()),
    ] {
        acc = content_hash_combine_structural(
            acc,
            content_hash_atom(format!("scalar-field:{field}")),
        );
        acc = content_hash_combine_structural(acc, content_hash_atom(value.to_string()));
    }
    for (field, values) in [
        ("source_roots", request.source_roots.as_slice()),
        ("scan_dirs", request.scan_dirs.as_slice()),
        ("exclude_substrings", request.exclude_substrings.as_slice()),
        (
            "discovery_scope_dirs",
            request.discovery_scope_dirs.as_slice(),
        ),
        (
            "execution_authority_source_roots",
            request.execution_authority_source_roots.as_slice(),
        ),
    ] {
        acc = content_hash_combine_structural(
            acc,
            content_hash_atom(format!("list-field:{field}:len={}", values.len())),
        );
        for (index, value) in values.iter().enumerate() {
            acc = content_hash_combine_structural(
                acc,
                content_hash_atom(format!("list-item:{field}:index={index}")),
            );
            acc = content_hash_combine_structural(acc, content_hash_atom(value.clone()));
        }
    }
    acc.digest.clone()
}

fn valid_git_object_id(value: &str) -> bool {
    matches!(value.len(), 40 | 64) && value.chars().all(|c| c.is_ascii_hexdigit())
}

fn git_rev_parse(spec: &str, label: &str) -> Result<String, String> {
    let output = std::process::Command::new("git")
        .current_dir(super::process_workspace_root())
        .args(["rev-parse", spec])
        .output()
        .map_err(|e| format!("floor {label} git rev-parse: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "floor {label} git rev-parse failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let oid = String::from_utf8_lossy(&output.stdout)
        .trim()
        .to_ascii_lowercase();
    if !valid_git_object_id(&oid) {
        return Err(format!("floor {label} invalid git object id `{oid}`"));
    }
    Ok(oid)
}

fn floor_tested_commit(candidate_override: Option<&str>) -> Result<String, String> {
    let candidate = candidate_override
        .map(str::to_string)
        .or_else(|| std::env::var("GITHUB_SHA").ok())
        .unwrap_or_else(|| "HEAD".to_string());
    git_rev_parse(&format!("{candidate}^{{commit}}"), "tested_commit")
}

fn floor_tested_tree(tested_commit: &str) -> Result<String, String> {
    git_rev_parse(&format!("{tested_commit}^{{tree}}"), "tested_tree")
}

/// Refuse ambient source bytes that are not exactly the bytes named by `tested_commit`.
/// The snapshot and prepared-subject layers read the working tree for host efficiency;
/// this check is the construction wall that makes that realization equivalent to an
/// exact Git-object read. Both tracked changes and untracked files under the observed
/// roots are disqualifying. Roots outside the repository cannot be attributed to the
/// tested Git tree and therefore refuse on this production boundary.
pub(crate) fn verify_source_roots_match_tested_commit(
    tested_commit: &str,
    source_roots: &[String],
    retained_sources: Option<&[(String, String)]>,
) -> Result<(), String> {
    if !valid_git_object_id(tested_commit) {
        return Err(format!(
            "exact source subject refused: invalid tested commit `{tested_commit}`"
        ));
    }
    // Prove the commit exists and is the object family the request names before using
    // it as a diff base. This also gives a located refusal for a stale GITHUB_SHA.
    let _ = floor_tested_tree(tested_commit)?;

    let mut pathspecs = Vec::new();
    for root in source_roots {
        let anchored = super::anchor_source_root(root);
        let rel =
            super::try_repo_relative_path_normalized(Path::new(&anchored)).ok_or_else(|| {
                format!(
                    "exact source subject refused: source root `{root}` is outside repository {}",
                    super::process_workspace_root().display()
                )
            })?;
        if !pathspecs.contains(&rel) {
            pathspecs.push(rel);
        }
    }
    if pathspecs.is_empty() {
        return Err("exact source subject refused: source-root universe is empty".to_string());
    }

    let mut diff = std::process::Command::new("git");
    diff.current_dir(super::process_workspace_root())
        .args(["diff", "--quiet", "--no-ext-diff", tested_commit, "--"])
        .args(&pathspecs);
    let status = diff
        .status()
        .map_err(|e| format!("exact source subject git diff failed: {e}"))?;
    if !status.success() {
        return Err(format!(
            "exact source subject refused: tracked bytes under [{}] differ from tested commit {tested_commit}",
            pathspecs.join(", ")
        ));
    }

    let untracked = std::process::Command::new("git")
        .current_dir(super::process_workspace_root())
        .args(["ls-files", "--others", "--exclude-standard", "--"])
        .args(&pathspecs)
        .output()
        .map_err(|e| format!("exact source subject untracked-file observation failed: {e}"))?;
    if !untracked.status.success() {
        return Err(format!(
            "exact source subject untracked-file observation failed: {}",
            String::from_utf8_lossy(&untracked.stderr)
        ));
    }
    let untracked_paths = String::from_utf8_lossy(&untracked.stdout);
    if !untracked_paths.trim().is_empty() {
        return Err(format!(
            "exact source subject refused: untracked bytes under [{}] are not present in tested commit {tested_commit}: {}",
            pathspecs.join(", "),
            untracked_paths.lines().take(8).collect::<Vec<_>>().join(", ")
        ));
    }

    // An ignored file is no more a member of the tested Git tree than an ordinary
    // untracked file. This second observation closes the hole where an explicitly
    // requested source root happened to live beneath an ignore rule.
    let ignored = std::process::Command::new("git")
        .current_dir(super::process_workspace_root())
        .args([
            "ls-files",
            "--others",
            "--ignored",
            "--exclude-standard",
            "--",
        ])
        .args(&pathspecs)
        .output()
        .map_err(|e| format!("exact source subject ignored-file observation failed: {e}"))?;
    if !ignored.status.success() {
        return Err(format!(
            "exact source subject ignored-file observation failed: {}",
            String::from_utf8_lossy(&ignored.stderr)
        ));
    }
    let ignored_paths = String::from_utf8_lossy(&ignored.stdout);
    if !ignored_paths.trim().is_empty() {
        return Err(format!(
            "exact source subject refused: ignored bytes under [{}] are not present in tested commit {tested_commit}: {}",
            pathspecs.join(", "),
            ignored_paths.lines().take(8).collect::<Vec<_>>().join(", ")
        ));
    }
    if let Some(retained_sources) = retained_sources {
        let mut sources: Vec<(String, &[u8])> = Vec::with_capacity(retained_sources.len());
        for (path, content) in retained_sources {
            let rel = super::workspace_relative_entry_path(path);
            if Path::new(&rel).is_absolute()
                || !pathspecs.iter().any(|root| {
                    rel == *root || rel.starts_with(&format!("{}/", root.trim_end_matches('/')))
                })
                || rel.contains('\n')
            {
                return Err(format!(
                    "exact source subject refused: retained source `{path}` is outside the declared root universe"
                ));
            }
            if let Some((_, existing)) = sources.iter().find(|(existing, _)| *existing == rel) {
                if *existing != content.as_bytes() {
                    return Err(format!(
                        "exact source subject refused: retained views disagree on bytes for `{rel}`"
                    ));
                }
                continue;
            }
            sources.push((rel, content.as_bytes()));
        }
        sources.sort_by(|a, b| a.0.cmp(&b.0));

        let mut child = std::process::Command::new("git")
            .current_dir(super::process_workspace_root())
            .args(["cat-file", "--batch"])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|error| format!("exact source subject git cat-file spawn failed: {error}"))?;
        {
            let input = child
                .stdin
                .as_mut()
                .ok_or_else(|| "exact source subject git cat-file stdin unavailable".to_string())?;
            for (path, _) in &sources {
                writeln!(input, "{tested_commit}:{path}").map_err(|error| {
                    format!("exact source subject git cat-file input failed: {error}")
                })?;
            }
        }
        drop(child.stdin.take());
        let output = child
            .wait_with_output()
            .map_err(|error| format!("exact source subject git cat-file failed: {error}"))?;
        if !output.status.success() {
            return Err(format!(
                "exact source subject git cat-file failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        let mut cursor = 0usize;
        for (path, retained) in sources {
            let header_end = output.stdout[cursor..]
                .iter()
                .position(|byte| *byte == b'\n')
                .map(|offset| cursor + offset)
                .ok_or_else(|| {
                    format!("exact source subject git cat-file truncated before `{path}`")
                })?;
            let header = std::str::from_utf8(&output.stdout[cursor..header_end]).map_err(|error| {
                format!("exact source subject git cat-file header for `{path}` is not UTF-8: {error}")
            })?;
            let fields: Vec<_> = header.split_whitespace().collect();
            if fields.len() != 3 || fields[1] != "blob" {
                return Err(format!(
                    "exact source subject refused: tested commit does not contain retained blob `{path}` ({header})"
                ));
            }
            let size: usize = fields[2].parse().map_err(|error| {
                format!("exact source subject git cat-file size for `{path}` is invalid: {error}")
            })?;
            cursor = header_end + 1;
            let blob_end = cursor.checked_add(size).ok_or_else(|| {
                format!("exact source subject git cat-file size overflow for `{path}`")
            })?;
            if output.stdout.get(cursor..blob_end) != Some(retained)
                || output.stdout.get(blob_end) != Some(&b'\n')
            {
                return Err(format!(
                    "exact source subject refused: retained bytes for `{path}` differ from tested commit {tested_commit}"
                ));
            }
            cursor = blob_end + 1;
        }
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
    if let Some(identity) = SUCCESSFUL_TOOL_IDENTITY.get() {
        return Ok(identity.clone());
    }
    let exe =
        std::env::current_exe().map_err(|e| format!("floor tool_identity current_exe: {e}"))?;
    let bytes =
        fs::read(&exe).map_err(|e| format!("floor tool_identity read {}: {e}", exe.display()))?;
    let identity = v1_rt::bytes_identity_hash(&bytes);
    let _ = SUCCESSFUL_TOOL_IDENTITY.set(identity.clone());
    Ok(SUCCESSFUL_TOOL_IDENTITY.get().cloned().unwrap_or(identity))
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
    let mut reference_unaccounted: Vec<String> =
        facts.reference_unaccounted.iter().cloned().collect();
    reference_unaccounted.sort();
    let mut observed_paths: Vec<String> = facts.observed_paths.iter().cloned().collect();
    observed_paths.sort();
    let mut declared_paths: Vec<String> = facts.declared_paths.iter().cloned().collect();
    declared_paths.sort();
    ModuleGraphFactsSnapshot {
        edges: facts.edges.clone(),
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
        reference_unaccounted,
        path_to_module: facts
            .path_to_module
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect(),
        observed_paths,
        read_refusals: facts.read_refusals.clone(),
        declared_paths,
    }
}

fn snapshot_to_facts(snapshot: &ModuleGraphFactsSnapshot) -> super::ModuleGraphFactsLive {
    super::ModuleGraphFactsLive {
        edges: snapshot.edges.clone(),
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
        observed_paths: snapshot.observed_paths.iter().cloned().collect(),
        read_refusals: snapshot.read_refusals.clone(),
        declared_paths: snapshot.declared_paths.iter().cloned().collect(),
    }
}

fn digest_rows(rows: &[DiscoveryRowSnapshot]) -> String {
    let mut acc = content_hash_atom("floor-discovery-roster-v2".to_string());
    for row in rows {
        acc = content_hash_combine_structural(acc, content_hash_atom(row.label.clone()));
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

fn digest_payload(snapshot: &FloorDiscoverySnapshot) -> String {
    // The digest field is not a term of its own digest. Production computes with an
    // empty slot; verification canonicalizes the populated wire value the same way.
    let mut canonical = snapshot.clone();
    canonical.payload_digest.clear();
    let json = serde_json::to_string(&canonical).unwrap_or_default();
    content_hash_atom(json).digest.clone()
}

fn verify_snapshot_integrity(snapshot: &FloorDiscoverySnapshot) -> Result<(), String> {
    let naming_authority_digest = naming_authority_digest_hex()?;
    if snapshot.request.naming_authority_digest != naming_authority_digest {
        return Err(format!(
            "floor discovery snapshot naming-authority mismatch: stored={} observed={naming_authority_digest}",
            snapshot.request.naming_authority_digest
        ));
    }
    let tool_identity = floor_tool_identity()?;
    if snapshot.request.tool_identity != tool_identity {
        return Err(format!(
            "floor discovery snapshot tool-identity mismatch: stored={} observed={tool_identity}",
            snapshot.request.tool_identity
        ));
    }
    let request_digest = request_identity_digest(&snapshot.request);
    if snapshot.request_identity_digest != request_digest {
        return Err(format!(
            "floor discovery snapshot request digest mismatch: stored={} recomputed={request_digest}",
            snapshot.request_identity_digest
        ));
    }
    let roster_digest = digest_rows(&snapshot.roster);
    if snapshot.roster_digest != roster_digest {
        return Err(format!(
            "floor discovery snapshot roster digest mismatch: stored={} recomputed={roster_digest}",
            snapshot.roster_digest
        ));
    }
    let facts_digest = digest_facts_snapshot(&snapshot.module_graph_facts);
    if snapshot.module_graph_facts_digest != facts_digest {
        return Err(format!(
            "floor discovery snapshot module-graph digest mismatch: stored={} recomputed={facts_digest}",
            snapshot.module_graph_facts_digest
        ));
    }
    let payload_digest = digest_payload(snapshot);
    if snapshot.payload_digest != payload_digest {
        return Err(format!(
            "floor discovery snapshot payload digest mismatch: stored={} recomputed={payload_digest}",
            snapshot.payload_digest
        ));
    }
    let actual_tree = floor_tested_tree(&snapshot.request.tested_commit)?;
    if snapshot.request.tested_tree != actual_tree {
        return Err(format!(
            "floor discovery snapshot tested-tree mismatch: commit {} names tree {actual_tree}, snapshot names {}",
            snapshot.request.tested_commit, snapshot.request.tested_tree
        ));
    }
    verify_source_roots_match_tested_commit(
        &snapshot.request.tested_commit,
        &snapshot.request.source_roots,
        None,
    )
}

fn discovery_identity(snapshot: &FloorDiscoverySnapshot) -> FloorDiscoveryIdentity {
    FloorDiscoveryIdentity {
        tested_commit: snapshot.request.tested_commit.clone(),
        tested_tree: snapshot.request.tested_tree.clone(),
        request_identity_digest: snapshot.request_identity_digest.clone(),
        roster_digest: snapshot.roster_digest.clone(),
        module_graph_facts_digest: snapshot.module_graph_facts_digest.clone(),
        payload_digest: snapshot.payload_digest.clone(),
        tool_identity: snapshot.request.tool_identity.clone(),
    }
}

pub fn current_floor_discovery_identity() -> Result<FloorDiscoveryIdentity, String> {
    CURRENT_FLOOR_DISCOVERY_IDENTITY.with(|slot| {
        slot.borrow().clone().ok_or_else(|| {
            "prepared floor subject refused: no integrity-verified discovery identity is installed in this process"
                .to_string()
        })
    })
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
    let naming_hygiene_refusal =
        match super::floor_filename_hygiene_refusal_via_producer(&request.source_roots) {
            Ok(()) => None,
            Err(msg) => Some(msg),
        };
    if naming_hygiene_refusal.is_some() {
        return Err(naming_hygiene_refusal.clone().unwrap());
    }
    let orphan_helper_refusal =
        match super::test_module_hygiene_bridge::check_orphan_helpers_or_err(&request.source_roots)
        {
            Ok(()) => None,
            Err(msg) => Some(msg),
        };
    if orphan_helper_refusal.is_some() {
        return Err(orphan_helper_refusal.clone().unwrap());
    }
    let mut rows = super::invoke_floor_discovery_producer(
        &request.source_roots,
        &request.scan_dirs,
        &request.exclude_substrings,
    )?;
    rows = super::apply_discovery_scope_dirs_filter(rows, &request.discovery_scope_dirs);
    let facts = super::build_module_graph_facts_live_uncached(&request.source_roots);
    super::refuse_on_module_graph_read_refusals(&facts)?;
    super::apply_effect_reach_derived_reads_live_tree(&mut rows, &facts);
    let inert = super::inert_lens_modules(&rows, &facts);
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
    let (lens_module_to_path, lens_with_justification) = super::lens_justification_census(&facts)?;
    let unjustified =
        super::unjustified_lens_modules(&lens_module_to_path, &lens_with_justification);
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
    let roster: Vec<DiscoveryRowSnapshot> = rows.iter().map(row_to_snapshot).collect();
    let facts_snapshot = facts_to_snapshot(&facts);
    let request_identity_digest = request_identity_digest(request);
    let roster_digest = digest_rows(&roster);
    let module_graph_facts_digest = digest_facts_snapshot(&facts_snapshot);
    let snapshot = FloorDiscoverySnapshot {
        request: request.clone(),
        request_identity_digest,
        roster,
        roster_digest,
        naming_hygiene_refusal,
        orphan_helper_refusal,
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
    verify_snapshot_integrity(snapshot)?;
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
    expected_request_digest: &str,
) -> Result<String, String> {
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
    verify_snapshot_integrity(&snapshot)?;
    if terminal.tested_commit != snapshot.request.tested_commit
        || terminal.tested_tree != snapshot.request.tested_tree
    {
        return Err("coordinator snapshot commit/tree does not match terminal receipt".to_string());
    }
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
    let expected_digest = request_identity_digest(request);
    let json_path = snapshot_json_path(walk_attempt_id);
    let bytes = fs::read_to_string(&json_path).map_err(|_| {
        format!(
            "coordinated floor discovery consumer: snapshot absent at {} — recompute refused",
            json_path.display()
        )
    })?;
    let snapshot: FloorDiscoverySnapshot = serde_json::from_str(&bytes)
        .map_err(|e| format!("coordinated consumer snapshot parse: {e}"))?;
    verify_snapshot_integrity(&snapshot)?;
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
    Ok(snapshot)
}

pub fn install_floor_discovery_snapshot(snapshot: &FloorDiscoverySnapshot) -> Result<(), String> {
    // Installation is the authority boundary: no cache or prepared identity can be
    // populated from bytes that have merely been deserialized or caller-authored.
    verify_snapshot_integrity(snapshot)?;
    let facts = snapshot_to_facts(&snapshot.module_graph_facts);
    install_module_graph_facts_cache(&snapshot.request.source_roots, &facts);
    let rows: Vec<super::DiscoveryRow> = snapshot.roster.iter().map(snapshot_to_row).collect();
    install_roster_cache(&snapshot.request_identity_digest, &rows);
    CURRENT_FLOOR_DISCOVERY_IDENTITY.with(|slot| {
        *slot.borrow_mut() = Some(discovery_identity(snapshot));
    });
    Ok(())
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
    let request_digest = request_identity_digest(&request);
    let started = Instant::now();
    let started_ms = trace_epoch_millis();

    match consumer {
        FloorDiscoveryConsumerRole::CoordinatedConsumer => {
            let snapshot = load_floor_discovery_snapshot_coordinated(walk_attempt_id, &request)?;
            install_floor_discovery_snapshot(&snapshot)?;
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
    install_floor_discovery_snapshot(&snapshot)?;
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

    #[test]
    fn request_identity_frames_named_lists_and_order() {
        let request = FloorDiscoveryRequest {
            tested_commit: "1111111111111111111111111111111111111111".to_string(),
            tested_tree: "2222222222222222222222222222222222222222".to_string(),
            source_roots: vec!["a".to_string()],
            scan_dirs: vec!["b".to_string()],
            exclude_substrings: vec![],
            discovery_scope_dirs: vec![],
            execution_mode: "Hermetic".to_string(),
            execution_authority_source_roots: vec!["authority".to_string()],
            naming_authority_digest: "naming".to_string(),
            tool_identity: "tool".to_string(),
        };
        let baseline = request_identity_digest(&request);
        let moved_boundary = FloorDiscoveryRequest {
            source_roots: vec!["a".to_string(), "b".to_string()],
            scan_dirs: vec![],
            ..request.clone()
        };
        let reordered = FloorDiscoveryRequest {
            source_roots: vec!["b".to_string(), "a".to_string()],
            scan_dirs: vec![],
            ..request.clone()
        };
        assert_ne!(baseline, request_identity_digest(&moved_boundary));
        assert_ne!(
            request_identity_digest(&moved_boundary),
            request_identity_digest(&reordered)
        );
    }

    #[test]
    fn snapshot_install_recomputes_every_identity_term_and_commit_tree_binding() {
        reset_floor_discovery_snapshot_for_test();
        let commit = floor_tested_commit(Some("HEAD")).expect("HEAD peels to a commit");
        let tree = floor_tested_tree(&commit).expect("HEAD commit names a tree");
        assert!(
            floor_tested_commit(Some(&tree)).is_err(),
            "a valid Git tree object id must not be admitted as tested_commit; GITHUB_SHA is peeled through ^{{commit}}"
        );
        // This tracked fixture root is intentionally outside the files changed by this
        // lane, so exact-tree verification observes committed bytes in local and remote
        // focused-test worktrees alike.
        let request = build_floor_discovery_request(
            &["fixtures/class_b_import_closure".to_string()],
            &["fixtures/class_b_import_closure".to_string()],
            &["excluded".to_string()],
            &["fixtures".to_string()],
            "Hermetic",
            &["fixtures/class_b_import_closure".to_string()],
        )
        .expect("exact request");
        let facts = ModuleGraphFactsSnapshot {
            edges: vec![],
            nodes: vec![],
            adjacency: BTreeMap::new(),
            selection_adjacency: BTreeMap::new(),
            reference_unaccounted: vec![],
            path_to_module: BTreeMap::new(),
            observed_paths: vec![],
            read_refusals: vec![],
            declared_paths: vec![],
        };
        let roster = vec![DiscoveryRowSnapshot {
            label: "exact".to_string(),
            entry: "fixtures/class_b_import_closure/exact.dag".to_string(),
            function: "holds".to_string(),
            reads_live_tree: false,
        }];
        let mut snapshot = FloorDiscoverySnapshot {
            request,
            request_identity_digest: String::new(),
            roster,
            roster_digest: String::new(),
            naming_hygiene_refusal: None,
            orphan_helper_refusal: None,
            module_graph_facts: facts,
            module_graph_facts_digest: String::new(),
            payload_digest: String::new(),
        };
        snapshot.request_identity_digest = request_identity_digest(&snapshot.request);
        snapshot.roster_digest = digest_rows(&snapshot.roster);
        snapshot.module_graph_facts_digest = digest_facts_snapshot(&snapshot.module_graph_facts);
        snapshot.payload_digest = digest_payload(&snapshot);
        verify_snapshot_integrity(&snapshot).expect("self-consistent exact snapshot");
        let retained_path = "fixtures/class_b_import_closure/perturbation_overlay/src/v2/extdeps/languages/rust_pool_perturb_ambient.dag";
        let retained_content =
            fs::read_to_string(super::super::workspace_root().join(retained_path))
                .expect("retained exact fixture source");
        verify_source_roots_match_tested_commit(
            &snapshot.request.tested_commit,
            &snapshot.request.source_roots,
            Some(&[(retained_path.to_string(), retained_content.clone())]),
        )
        .expect("retained bytes match tested Git object");
        assert!(verify_source_roots_match_tested_commit(
            &snapshot.request.tested_commit,
            &snapshot.request.source_roots,
            Some(&[(
                retained_path.to_string(),
                format!("{retained_content}\n// tampered")
            )]),
        )
        .expect_err("post-read retained-byte tamper")
        .contains("retained bytes"));

        let mut request_tamper = snapshot.clone();
        request_tamper
            .request
            .scan_dirs
            .push("moved-boundary".to_string());
        assert!(verify_snapshot_integrity(&request_tamper)
            .expect_err("request tamper")
            .contains("request digest mismatch"));

        let mut roster_tamper = snapshot.clone();
        roster_tamper.roster[0].label = "tampered".to_string();
        assert!(verify_snapshot_integrity(&roster_tamper)
            .expect_err("roster tamper")
            .contains("roster digest mismatch"));

        let mut facts_tamper = snapshot.clone();
        facts_tamper
            .module_graph_facts
            .observed_paths
            .push("fabricated.dag".to_string());
        assert!(verify_snapshot_integrity(&facts_tamper)
            .expect_err("facts tamper")
            .contains("module-graph digest mismatch"));

        let mut payload_tamper = snapshot.clone();
        payload_tamper.payload_digest = "0000000000000000".to_string();
        assert!(verify_snapshot_integrity(&payload_tamper)
            .expect_err("payload tamper")
            .contains("payload digest mismatch"));

        let mut tree_tamper = snapshot.clone();
        tree_tamper.request.tested_tree = tree_tamper.request.tested_commit.clone();
        tree_tamper.request_identity_digest = request_identity_digest(&tree_tamper.request);
        tree_tamper.payload_digest = digest_payload(&tree_tamper);
        assert!(verify_snapshot_integrity(&tree_tamper)
            .expect_err("commit/tree tamper")
            .contains("tested-tree mismatch"));

        let mut naming_tamper = snapshot.clone();
        naming_tamper.request.naming_authority_digest = "0000000000000000".to_string();
        naming_tamper.request_identity_digest = request_identity_digest(&naming_tamper.request);
        naming_tamper.payload_digest = digest_payload(&naming_tamper);
        assert!(verify_snapshot_integrity(&naming_tamper)
            .expect_err("naming identity tamper")
            .contains("naming-authority mismatch"));

        let mut tool_tamper = snapshot.clone();
        tool_tamper.request.tool_identity = "0000000000000000".to_string();
        tool_tamper.request_identity_digest = request_identity_digest(&tool_tamper.request);
        tool_tamper.payload_digest = digest_payload(&tool_tamper);
        assert!(verify_snapshot_integrity(&tool_tamper)
            .expect_err("tool identity tamper")
            .contains("tool-identity mismatch"));

        assert!(install_floor_discovery_snapshot(&payload_tamper).is_err());
        assert!(
            current_floor_discovery_identity().is_err(),
            "failed installation must not publish an identity"
        );
        install_floor_discovery_snapshot(&snapshot).expect("verified installation");
        let installed = current_floor_discovery_identity().expect("installed identity");
        assert_eq!(installed.tested_commit, snapshot.request.tested_commit);
        assert_eq!(installed.tested_tree, snapshot.request.tested_tree);
        assert_eq!(installed.payload_digest, snapshot.payload_digest);
    }

    #[test]
    fn coordinated_consumer_refuses_absent_snapshot() {
        reset_floor_discovery_snapshot_for_test();
        let request = FloorDiscoveryRequest {
            tested_commit: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
            tested_tree: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
            source_roots: vec!["dag".to_string()],
            scan_dirs: vec![],
            exclude_substrings: vec![],
            discovery_scope_dirs: vec![],
            execution_mode: "Hermetic".to_string(),
            execution_authority_source_roots: vec!["dag".to_string()],
            naming_authority_digest: "0123456789abcdef".to_string(),
            tool_identity: "tool".to_string(),
        };
        assert!(
            load_floor_discovery_snapshot_coordinated("attempt-a", &request).is_err(),
            "absent snapshot must refuse"
        );
    }

    #[test]
    fn coordinated_consumer_refuses_subject_mismatch() {
        reset_floor_discovery_snapshot_for_test();
        let clean_root = "fixtures/class_b_import_closure".to_string();
        let request = build_floor_discovery_request(
            std::slice::from_ref(&clean_root),
            &[],
            &[],
            &[],
            "Hermetic",
            std::slice::from_ref(&clean_root),
        )
        .expect("request");
        let mut snapshot = FloorDiscoverySnapshot {
            request: request.clone(),
            request_identity_digest: request_identity_digest(&request),
            roster: vec![],
            roster_digest: digest_rows(&[]),
            naming_hygiene_refusal: None,
            orphan_helper_refusal: None,
            module_graph_facts: ModuleGraphFactsSnapshot {
                edges: vec![],
                nodes: vec![],
                adjacency: BTreeMap::new(),
                selection_adjacency: BTreeMap::new(),
                reference_unaccounted: vec![],
                path_to_module: BTreeMap::new(),
                observed_paths: vec![],
                read_refusals: vec![],
                declared_paths: vec![],
            },
            module_graph_facts_digest: digest_facts_snapshot(&ModuleGraphFactsSnapshot {
                edges: vec![],
                nodes: vec![],
                adjacency: BTreeMap::new(),
                selection_adjacency: BTreeMap::new(),
                reference_unaccounted: vec![],
                path_to_module: BTreeMap::new(),
                observed_paths: vec![],
                read_refusals: vec![],
                declared_paths: vec![],
            }),
            payload_digest: String::new(),
        };
        snapshot.payload_digest = digest_payload(&snapshot);
        publish_floor_discovery_snapshot("attempt-b", &snapshot).expect("publish");
        let mismatched = FloorDiscoveryRequest {
            source_roots: vec!["fixtures/class_b_import_closure/perturbation_overlay".to_string()],
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
        let clean_root = "fixtures/class_b_import_closure".to_string();
        let request = build_floor_discovery_request(
            std::slice::from_ref(&clean_root),
            &[],
            &[],
            &[],
            "Hermetic",
            std::slice::from_ref(&clean_root),
        )
        .expect("request");
        let snapshot = FloorDiscoverySnapshot {
            request: request.clone(),
            request_identity_digest: request_identity_digest(&request),
            roster: vec![],
            roster_digest: digest_rows(&[]),
            naming_hygiene_refusal: None,
            orphan_helper_refusal: None,
            module_graph_facts: ModuleGraphFactsSnapshot {
                edges: vec![],
                nodes: vec![],
                adjacency: BTreeMap::new(),
                selection_adjacency: BTreeMap::new(),
                reference_unaccounted: vec![],
                path_to_module: BTreeMap::new(),
                observed_paths: vec![],
                read_refusals: vec![],
                declared_paths: vec![],
            },
            module_graph_facts_digest: digest_facts_snapshot(&ModuleGraphFactsSnapshot {
                edges: vec![],
                nodes: vec![],
                adjacency: BTreeMap::new(),
                selection_adjacency: BTreeMap::new(),
                reference_unaccounted: vec![],
                path_to_module: BTreeMap::new(),
                observed_paths: vec![],
                read_refusals: vec![],
                declared_paths: vec![],
            }),
            payload_digest: String::new(),
        };
        let mut snapshot = snapshot;
        snapshot.payload_digest = digest_payload(&snapshot);
        publish_floor_discovery_snapshot("attempt-c", &snapshot).expect("publish");
        let rows = discover_floor_witness_roster_with_snapshot(
            &request.source_roots,
            &request.scan_dirs,
            &request.exclude_substrings,
            &request.discovery_scope_dirs,
            "attempt-c",
            FloorDiscoveryConsumerRole::CoordinatedConsumer,
            &request.execution_mode,
            &request.execution_authority_source_roots,
        )
        .expect("coordinated consumer must consume published snapshot");
        assert!(rows.is_empty());
        assert_eq!(
            coordinated_discovery_compute_count(),
            0,
            "coordinated consumer must not invoke produce_floor_discovery_snapshot"
        );
    }
}
