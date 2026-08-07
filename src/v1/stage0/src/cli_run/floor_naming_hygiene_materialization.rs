//! R0: materialize floor naming hygiene once across ordinary and scoped floor workers.
//!
//! Subject equality is proven via `NamingHygieneComputationIdentity` (tested tree,
//! source-root population digest, naming-authority digest, tool identity) — never via
//! ordering or a boolean passed from parent to child.

use std::collections::HashMap;
use std::fs;
use std::io::Write as IoWrite;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

use crate::std_content_hash::{content_hash_atom, content_hash_combine_structural};
use crate::v1_rt;

pub const FLOOR_NAMING_HYGIENE_RECEIPT_PATH: &str = "target/floor-naming-hygiene-receipt.tsv";
pub const FLOOR_EXECUTION_TRACE_PATH: &str = "target/floor-execution-trace.tsv";

static PROCESS_MONO_START: OnceLock<Instant> = OnceLock::new();
static NAMING_HYGIENE_COMPUTE_COUNT: AtomicUsize = AtomicUsize::new(0);
static IN_PROCESS_HYGIENE_OK: OnceLock<Mutex<HashMap<NamingHygieneComputationIdentity, ()>>> =
    OnceLock::new();
static IN_PROCESS_ROSTER_CACHE: OnceLock<Mutex<HashMap<RosterCacheKey, Vec<super::DiscoveryRow>>>> =
    OnceLock::new();

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NamingHygieneComputationIdentity {
    pub tested_tree: String,
    pub source_roots_digest: String,
    pub naming_authority_digest: String,
    pub tool_identity: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NamingHygieneReceiptOutcome {
    VerifiedHit,
    ConfirmedMiss,
    Refused { detail: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionWaitReason {
    DependencyBlocked,
    ResourceBlocked,
    MaterializationLookup,
    ExecutorSerialized,
    ExternalProcessWait,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NamingHygieneConsumerRole {
    /// Ordinary floor worker: may publish a cross-process receipt after computing.
    FirstConsumerPublish,
    /// Scoped floor worker: must verify receipt subject before skipping recompute.
    SecondConsumerVerify,
    /// Same-process reuse only (no cross-process publish/consume).
    InProcessOnly,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct RosterCacheKey {
    identity: NamingHygieneComputationIdentity,
    scan_dirs: Vec<String>,
    exclude_substrings: Vec<String>,
    discovery_scope_dirs: Vec<String>,
}

fn mono_start() -> Instant {
    *PROCESS_MONO_START.get_or_init(Instant::now)
}

fn mono_ns() -> u64 {
    mono_start().elapsed().as_nanos() as u64
}

pub fn naming_hygiene_compute_count() -> usize {
    NAMING_HYGIENE_COMPUTE_COUNT.load(Ordering::SeqCst)
}

#[cfg(test)]
pub fn reset_naming_hygiene_materialization_for_test() {
    NAMING_HYGIENE_COMPUTE_COUNT.store(0, Ordering::SeqCst);
    if let Some(lock) = IN_PROCESS_HYGIENE_OK.get() {
        lock.lock().unwrap().clear();
    }
    if let Some(lock) = IN_PROCESS_ROSTER_CACHE.get() {
        lock.lock().unwrap().clear();
    }
    let receipt = super::workspace_root().join(FLOOR_NAMING_HYGIENE_RECEIPT_PATH);
    let _ = fs::remove_file(receipt);
    let trace = super::workspace_root().join(FLOOR_EXECUTION_TRACE_PATH);
    let _ = fs::remove_file(trace);
}

const NAMING_AUTHORITY_PATHS: &[&str] = &[
    "src/v2/workflow/floor_naming_hygiene.dag",
    "src/v2/workflow/floor_discovery_producer.dag",
    "src/v2/workflow/floor_discovery.dag",
    "dag/gunbc/test_module_hygiene.dag",
    "src/v1/stage0/src/cli_run/test_module_hygiene_bridge.rs",
];

pub fn compute_naming_hygiene_identity(
    source_roots: &[String],
) -> Result<NamingHygieneComputationIdentity, String> {
    Ok(NamingHygieneComputationIdentity {
        tested_tree: floor_tested_tree()?,
        source_roots_digest: source_roots_digest_hex(source_roots)?,
        naming_authority_digest: naming_authority_digest_hex()?,
        tool_identity: floor_tool_identity()?,
    })
}

fn floor_tested_tree() -> Result<String, String> {
    if let Ok(sha) = std::env::var("GITHUB_SHA") {
        let lower = sha.to_ascii_lowercase();
        if lower.len() == 40 && lower.chars().all(|c| c.is_ascii_hexdigit()) {
            return Ok(lower);
        }
    }
    let output = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .map_err(|e| format!("floor tested_tree git rev-parse: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "floor tested_tree git rev-parse failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let sha = String::from_utf8_lossy(&output.stdout)
        .trim()
        .to_ascii_lowercase();
    if sha.len() != 40 || !sha.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(format!("floor tested_tree invalid git HEAD `{sha}`"));
    }
    Ok(sha)
}

fn source_roots_digest_hex(source_roots: &[String]) -> Result<String, String> {
    let digest = source_roots.iter().fold(
        content_hash_atom("floor-naming-hygiene-source-roots".to_string()),
        |acc, root| content_hash_combine_structural(acc, content_hash_atom(root.clone())),
    );
    Ok(digest.digest.clone())
}

fn naming_authority_digest_hex() -> Result<String, String> {
    let workspace = super::workspace_root();
    let mut digest = content_hash_atom("floor-naming-hygiene-authority-v1".to_string());
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

fn ensure_execution_trace_header(path: &Path) -> Result<(), String> {
    if path.is_file() {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("create execution trace directory {}: {e}", parent.display()))?;
    }
    let header = "computation_id\tdependency_ids\tplacement\tmaterialization_decision\tstarted_mono_ns\tcompleted_mono_ns\tcpu_duration_ns\twall_duration_ns\tmemory_observation\toutcome\twait_reason\n";
    fs::write(path, header)
        .map_err(|e| format!("write execution trace header {}: {e}", path.display()))?;
    Ok(())
}

pub fn append_execution_trace_row(
    computation_id: &str,
    dependency_ids: &[String],
    placement: &str,
    materialization_decision: &str,
    started_mono_ns: u64,
    completed_mono_ns: u64,
    wall_duration_ns: u64,
    outcome: &str,
    wait_reason: ExecutionWaitReason,
) {
    let path = super::workspace_root().join(FLOOR_EXECUTION_TRACE_PATH);
    if let Err(e) = ensure_execution_trace_header(&path) {
        eprintln!("floor_naming_hygiene_materialization: execution trace refused: {e}");
        return;
    }
    let deps = dependency_ids.join(",");
    let wait = match wait_reason {
        ExecutionWaitReason::DependencyBlocked => "DependencyBlocked",
        ExecutionWaitReason::ResourceBlocked => "ResourceBlocked",
        ExecutionWaitReason::MaterializationLookup => "MaterializationLookup",
        ExecutionWaitReason::ExecutorSerialized => "ExecutorSerialized",
        ExecutionWaitReason::ExternalProcessWait => "ExternalProcessWait",
    };
    let row = format!(
        "{computation_id}\t{deps}\t{placement}\t{materialization_decision}\t{started_mono_ns}\t{completed_mono_ns}\t{wall_duration_ns}\t{wall_duration_ns}\t\t{outcome}\t{wait}\n"
    );
    if let Ok(mut file) = fs::OpenOptions::new().create(true).append(true).open(&path) {
        let _ = file.write_all(row.as_bytes()).and_then(|()| file.flush());
    }
}

fn identity_wire_fields(id: &NamingHygieneComputationIdentity) -> (String, String, String, String) {
    (
        id.tested_tree.clone(),
        id.source_roots_digest.clone(),
        id.naming_authority_digest.clone(),
        id.tool_identity.clone(),
    )
}

fn publish_naming_hygiene_receipt(
    identity: &NamingHygieneComputationIdentity,
    outcome: NamingHygieneReceiptOutcome,
) -> Result<(), String> {
    let path = super::workspace_root().join(FLOOR_NAMING_HYGIENE_RECEIPT_PATH);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            format!(
                "create naming hygiene receipt directory {}: {e}",
                parent.display()
            )
        })?;
    }
    let (tested_tree, source_roots_digest, naming_authority_digest, tool_identity) =
        identity_wire_fields(identity);
    let (outcome_label, detail) = match &outcome {
        NamingHygieneReceiptOutcome::VerifiedHit => ("VerifiedHit", ""),
        NamingHygieneReceiptOutcome::ConfirmedMiss => ("ConfirmedMiss", ""),
        NamingHygieneReceiptOutcome::Refused { detail } => ("Refused", detail.as_str()),
    };
    let clean_detail = detail.replace(['\t', '\r', '\n'], " ");
    let body = format!(
        "tested_tree\tsource_roots_digest\tnaming_authority_digest\ttool_identity\toutcome\tdetail\n{tested_tree}\t{source_roots_digest}\t{naming_authority_digest}\t{tool_identity}\t{outcome_label}\t{clean_detail}\n"
    );
    fs::write(&path, body)
        .map_err(|e| format!("write naming hygiene receipt {}: {e}", path.display()))?;
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ConsumedReceipt {
    VerifiedHit,
    ConfirmedMiss,
    SubjectMismatch { detail: String },
    Refused { detail: String },
    Absent,
}

fn consume_naming_hygiene_receipt(expected: &NamingHygieneComputationIdentity) -> ConsumedReceipt {
    let path = super::workspace_root().join(FLOOR_NAMING_HYGIENE_RECEIPT_PATH);
    let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return ConsumedReceipt::Absent,
    };
    let mut lines = content.lines();
    let header = lines.next().unwrap_or_default();
    if header
        != "tested_tree\tsource_roots_digest\tnaming_authority_digest\ttool_identity\toutcome\tdetail"
    {
        return ConsumedReceipt::SubjectMismatch {
            detail: format!("naming hygiene receipt header mismatch at {}", path.display()),
        };
    }
    let row = match lines.next() {
        Some(r) if !r.is_empty() => r,
        _ => return ConsumedReceipt::Absent,
    };
    let cols: Vec<&str> = row.split('\t').collect();
    if cols.len() != 6 {
        return ConsumedReceipt::SubjectMismatch {
            detail: format!("naming hygiene receipt arity {} != 6", cols.len()),
        };
    }
    let receipt_identity = NamingHygieneComputationIdentity {
        tested_tree: cols[0].to_string(),
        source_roots_digest: cols[1].to_string(),
        naming_authority_digest: cols[2].to_string(),
        tool_identity: cols[3].to_string(),
    };
    if receipt_identity != *expected {
        return ConsumedReceipt::SubjectMismatch {
            detail: format!(
                "naming hygiene receipt subject mismatch (expected tree={} roots={} authority={} tool={}, got tree={} roots={} authority={} tool={})",
                expected.tested_tree,
                expected.source_roots_digest,
                expected.naming_authority_digest,
                expected.tool_identity,
                receipt_identity.tested_tree,
                receipt_identity.source_roots_digest,
                receipt_identity.naming_authority_digest,
                receipt_identity.tool_identity,
            ),
        };
    }
    match cols[4] {
        "VerifiedHit" => ConsumedReceipt::VerifiedHit,
        "ConfirmedMiss" => ConsumedReceipt::ConfirmedMiss,
        "Refused" => ConsumedReceipt::Refused {
            detail: cols[5].to_string(),
        },
        other => ConsumedReceipt::SubjectMismatch {
            detail: format!("naming hygiene receipt unknown outcome `{other}`"),
        },
    }
}

fn run_naming_hygiene_compute(
    source_roots: &[String],
    exclude_substrings: &[String],
    role: NamingHygieneConsumerRole,
) -> Result<(), String> {
    NAMING_HYGIENE_COMPUTE_COUNT.fetch_add(1, Ordering::SeqCst);
    let started = mono_ns();
    let wall_start = Instant::now();
    let identity = compute_naming_hygiene_identity(source_roots)?;
    let result =
        super::discover_floor_witness_roster_inner(source_roots, &[], exclude_substrings, &[]).map(
            |rows| {
                let roster_key = RosterCacheKey {
                    identity: identity.clone(),
                    scan_dirs: Vec::new(),
                    exclude_substrings: exclude_substrings.to_vec(),
                    discovery_scope_dirs: Vec::new(),
                };
                IN_PROCESS_ROSTER_CACHE
                    .get_or_init(|| Mutex::new(HashMap::new()))
                    .lock()
                    .unwrap()
                    .insert(roster_key, rows);
            },
        );
    let wall_ns = wall_start.elapsed().as_nanos() as u64;
    let completed = mono_ns();
    let outcome_label = if result.is_ok() {
        "ComputedOk"
    } else {
        "ComputedRefused"
    };
    append_execution_trace_row(
        "floor_naming_hygiene_walk",
        &[],
        "LocalInProcess",
        "Share",
        started,
        completed,
        wall_ns,
        outcome_label,
        ExecutionWaitReason::DependencyBlocked,
    );
    match &result {
        Ok(()) => {
            IN_PROCESS_HYGIENE_OK
                .get_or_init(|| Mutex::new(HashMap::new()))
                .lock()
                .unwrap()
                .insert(identity.clone(), ());
            if role == NamingHygieneConsumerRole::FirstConsumerPublish {
                publish_naming_hygiene_receipt(
                    &identity,
                    NamingHygieneReceiptOutcome::VerifiedHit,
                )?;
            }
        }
        Err(detail) => {
            if role == NamingHygieneConsumerRole::FirstConsumerPublish {
                publish_naming_hygiene_receipt(
                    &identity,
                    NamingHygieneReceiptOutcome::Refused {
                        detail: detail.clone(),
                    },
                )?;
            }
        }
    }
    result
}

/// Pre-plan naming hygiene walk with materialization across workers.
pub fn materialize_pre_plan_naming_hygiene_walk(
    source_roots: &[String],
    exclude_substrings: &[String],
    role: NamingHygieneConsumerRole,
) -> Result<(), String> {
    let lookup_started = mono_ns();
    let identity = compute_naming_hygiene_identity(source_roots)?;
    if IN_PROCESS_HYGIENE_OK
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .unwrap()
        .contains_key(&identity)
    {
        let completed = mono_ns();
        append_execution_trace_row(
            "floor_naming_hygiene_walk",
            &[],
            "LocalInProcess",
            "Share",
            lookup_started,
            completed,
            0,
            "VerifiedHit",
            ExecutionWaitReason::MaterializationLookup,
        );
        return Ok(());
    }

    if role == NamingHygieneConsumerRole::SecondConsumerVerify {
        match consume_naming_hygiene_receipt(&identity) {
            ConsumedReceipt::VerifiedHit => {
                let completed = mono_ns();
                append_execution_trace_row(
                    "floor_naming_hygiene_walk",
                    &["floor_naming_hygiene_receipt".to_string()],
                    "LocalFilesystem",
                    "Share",
                    lookup_started,
                    completed,
                    0,
                    "VerifiedHit",
                    ExecutionWaitReason::MaterializationLookup,
                );
                IN_PROCESS_HYGIENE_OK
                    .get_or_init(|| Mutex::new(HashMap::new()))
                    .lock()
                    .unwrap()
                    .insert(identity, ());
                return Ok(());
            }
            ConsumedReceipt::Refused { detail } => {
                return Err(format!(
                    "naming hygiene receipt refused both consumers: {detail}"
                ));
            }
            ConsumedReceipt::SubjectMismatch { detail } => {
                append_execution_trace_row(
                    "floor_naming_hygiene_walk",
                    &[],
                    "LocalInProcess",
                    "Recompute",
                    lookup_started,
                    mono_ns(),
                    0,
                    "SubjectMismatch",
                    ExecutionWaitReason::MaterializationLookup,
                );
                return run_naming_hygiene_compute(
                    source_roots,
                    exclude_substrings,
                    NamingHygieneConsumerRole::InProcessOnly,
                );
            }
            ConsumedReceipt::Absent | ConsumedReceipt::ConfirmedMiss => {
                return Err(
                    "naming hygiene receipt absent for scoped consumer — ordinary worker must publish first"
                        .to_string(),
                );
            }
        }
    }

    append_execution_trace_row(
        "floor_naming_hygiene_walk",
        &[],
        "LocalInProcess",
        "Recompute",
        lookup_started,
        mono_ns(),
        0,
        "ConfirmedMiss",
        ExecutionWaitReason::MaterializationLookup,
    );
    run_naming_hygiene_compute(source_roots, exclude_substrings, role)
}

/// `discover_floor_witness_roster` with in-process roster memoization keyed by full subject.
pub fn discover_floor_witness_roster_materialized(
    source_roots: &[String],
    scan_dirs: &[String],
    exclude_substrings: &[String],
    discovery_scope_dirs: &[String],
) -> Result<Vec<super::DiscoveryRow>, String> {
    let identity = compute_naming_hygiene_identity(source_roots)?;
    let key = RosterCacheKey {
        identity,
        scan_dirs: scan_dirs.to_vec(),
        exclude_substrings: exclude_substrings.to_vec(),
        discovery_scope_dirs: discovery_scope_dirs.to_vec(),
    };
    let cache = IN_PROCESS_ROSTER_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    if let Some(rows) = cache.lock().unwrap().get(&key) {
        return Ok(rows.clone());
    }
    let lookup_started = mono_ns();
    let wall_start = Instant::now();
    let rows = super::discover_floor_witness_roster_inner(
        source_roots,
        scan_dirs,
        exclude_substrings,
        discovery_scope_dirs,
    )?;
    let wall_ns = wall_start.elapsed().as_nanos() as u64;
    append_execution_trace_row(
        "discover_floor_witness_roster",
        &["floor_naming_hygiene_walk".to_string()],
        "LocalInProcess",
        if scan_dirs.is_empty() && discovery_scope_dirs.is_empty() {
            "Share"
        } else {
            "Recompute"
        },
        lookup_started,
        mono_ns(),
        wall_ns,
        "ComputedOk",
        ExecutionWaitReason::DependencyBlocked,
    );
    cache.lock().unwrap().insert(key, rows.clone());
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn naming_hygiene_receipt_subject_mismatch_refuses_reuse() {
        reset_naming_hygiene_materialization_for_test();
        let identity = NamingHygieneComputationIdentity {
            tested_tree: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
            source_roots_digest: "0123456789abcdef".to_string(),
            naming_authority_digest: "fedcba9876543210".to_string(),
            tool_identity: "tool-a".to_string(),
        };
        publish_naming_hygiene_receipt(&identity, NamingHygieneReceiptOutcome::VerifiedHit)
            .expect("publish");
        let other = NamingHygieneComputationIdentity {
            tested_tree: identity.tested_tree.clone(),
            source_roots_digest: "ffffffffffffffff".to_string(),
            naming_authority_digest: identity.naming_authority_digest.clone(),
            tool_identity: identity.tool_identity.clone(),
        };
        match consume_naming_hygiene_receipt(&other) {
            ConsumedReceipt::SubjectMismatch { .. } => {}
            other => panic!("expected SubjectMismatch, got {:?}", other),
        }
    }

    #[test]
    fn naming_hygiene_scoped_consumer_reads_verified_hit_receipt() {
        reset_naming_hygiene_materialization_for_test();
        let identity = compute_naming_hygiene_identity(&["dag".to_string(), "src/v2".to_string()])
            .expect("identity");
        publish_naming_hygiene_receipt(&identity, NamingHygieneReceiptOutcome::VerifiedHit)
            .expect("publish");
        match consume_naming_hygiene_receipt(&identity) {
            ConsumedReceipt::VerifiedHit => {}
            other => panic!("expected VerifiedHit, got {:?}", other),
        }
        let excludes = super::super::witness_exclusion_substrings();
        let before = naming_hygiene_compute_count();
        materialize_pre_plan_naming_hygiene_walk(
            &["dag".to_string(), "src/v2".to_string()],
            &excludes,
            NamingHygieneConsumerRole::SecondConsumerVerify,
        )
        .expect("scoped consume");
        assert_eq!(
            naming_hygiene_compute_count(),
            before,
            "scoped consumer must not recompute on verified receipt"
        );
        let trace_path = super::super::workspace_root().join(FLOOR_EXECUTION_TRACE_PATH);
        let trace = fs::read_to_string(trace_path).expect("trace");
        assert!(
            trace.contains("VerifiedHit"),
            "trace should record materialization hit"
        );
    }
}
