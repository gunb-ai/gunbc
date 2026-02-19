//! Workflow run ledger + CAS-backed output materialization (WF3).
#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

// Planner-side local cache/ledger persistence currently writes under
// `.gunbc/workflow-ledger/*`. This is intentionally outside transport-boundary
// DAG execution and is treated as planner-local state.
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use gunbc_ir::{NodeId, PortName, Value};
use serde::{Deserialize, Serialize};

use super::key::{MaterializationKey, MissReason, WorkIdentity};

/// Run identifier used in ledger statuses.
pub type RunId = String;

/// Cached/executed state persisted for planner decisions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LedgerStatus {
    CachedHit { previous_run: RunId },
    Executed { reason: MissReason },
    Failed { reason: MissReason, error: String },
    Skipped { blocked_by: NodeId },
}

/// One persisted ledger record per work key.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunLedgerEntry {
    pub exec_node_id: NodeId,
    pub work_id: WorkIdentity,
    pub key: MaterializationKey,
    pub status: LedgerStatus,
    pub output_hashes: BTreeMap<PortName, String>,
    pub duration_ms: u64,
}

/// Filesystem paths for workflow ledger state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowLedgerPaths {
    pub root: PathBuf,
    pub global_ledger: PathBuf,
    pub cas_dir: PathBuf,
}

/// Workflow ledger persistence/rehydration errors.
#[derive(Debug)]
pub enum WorkflowLedgerError {
    Io(std::io::Error),
    Serde(serde_json::Error),
    MissingOutputPayload { hash: String, path: PathBuf },
}

impl std::fmt::Display for WorkflowLedgerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkflowLedgerError::Io(error) => write!(f, "workflow ledger I/O error: {error}"),
            WorkflowLedgerError::Serde(error) => {
                write!(f, "workflow ledger serialization error: {error}")
            }
            WorkflowLedgerError::MissingOutputPayload { hash, path } => write!(
                f,
                "missing output payload for hash '{}' at '{}'",
                hash,
                path.display()
            ),
        }
    }
}

impl std::error::Error for WorkflowLedgerError {}

impl From<std::io::Error> for WorkflowLedgerError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<serde_json::Error> for WorkflowLedgerError {
    fn from(error: serde_json::Error) -> Self {
        Self::Serde(error)
    }
}

/// Resolve canonical global ledger paths under `.gunbc/workflow-ledger`.
pub fn workflow_ledger_paths(workspace_root: &Path) -> WorkflowLedgerPaths {
    let root = workspace_root.join(".gunbc").join("workflow-ledger");
    WorkflowLedgerPaths {
        global_ledger: root.join("global.ndjson"),
        cas_dir: root.join("cas"),
        root,
    }
}

/// Load global run ledger entries (empty when ledger does not yet exist).
pub fn load_global_ledger(
    workspace_root: &Path,
) -> Result<Vec<RunLedgerEntry>, WorkflowLedgerError> {
    let paths = workflow_ledger_paths(workspace_root);
    if !paths.global_ledger.exists() {
        return Ok(Vec::new());
    }

    let file = File::open(&paths.global_ledger)?;
    let reader = BufReader::new(file);
    let mut entries = Vec::new();
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        entries.push(serde_json::from_str(&line)?);
    }
    Ok(entries)
}

/// Persist global run ledger entries using crash-safe temp-file replace semantics.
pub fn save_global_ledger(
    workspace_root: &Path,
    entries: &[RunLedgerEntry],
) -> Result<(), WorkflowLedgerError> {
    let paths = workflow_ledger_paths(workspace_root);
    fs::create_dir_all(&paths.root)?;

    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let tmp = paths.root.join(format!(".global.ndjson.tmp.{nonce}"));
    {
        let file = File::create(&tmp)?;
        let mut writer = BufWriter::new(file);
        for entry in entries {
            let line = serde_json::to_string(entry)?;
            writer.write_all(line.as_bytes())?;
            writer.write_all(b"\n")?;
        }
        writer.flush()?;
    }
    fs::rename(tmp, paths.global_ledger)?;
    Ok(())
}

/// Append a single run ledger entry.
pub fn append_global_ledger_entry(
    workspace_root: &Path,
    entry: RunLedgerEntry,
) -> Result<(), WorkflowLedgerError> {
    let mut entries = load_global_ledger(workspace_root)?;
    entries.push(entry);
    save_global_ledger(workspace_root, &entries)
}

fn cas_payload_path(cas_dir: &Path, hash: &str) -> PathBuf {
    cas_dir.join(format!("{hash}.json"))
}

/// Store an output payload in CAS and return its hash.
pub fn store_output_payload(
    workspace_root: &Path,
    payload: &Value,
) -> Result<String, WorkflowLedgerError> {
    let paths = workflow_ledger_paths(workspace_root);
    fs::create_dir_all(&paths.cas_dir)?;

    let bytes = serde_json::to_vec(payload)?;
    let hash = gunbc_infra::hash::ContentHash::from_bytes(&bytes)
        .as_str()
        .to_string();
    let target = cas_payload_path(&paths.cas_dir, &hash);
    if target.exists() {
        return Ok(hash);
    }

    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let tmp = paths.cas_dir.join(format!(".{hash}.tmp.{nonce}.json"));
    {
        let mut file = BufWriter::new(File::create(&tmp)?);
        file.write_all(&bytes)?;
        file.flush()?;
    }
    fs::rename(tmp, &target)?;
    Ok(hash)
}

/// Load an output payload from CAS by hash (fail-closed when missing).
pub fn load_output_payload(
    workspace_root: &Path,
    hash: &str,
) -> Result<Value, WorkflowLedgerError> {
    let paths = workflow_ledger_paths(workspace_root);
    let path = cas_payload_path(&paths.cas_dir, hash);
    if !path.exists() {
        return Err(WorkflowLedgerError::MissingOutputPayload {
            hash: hash.to_string(),
            path,
        });
    }
    let bytes = fs::read(path)?;
    Ok(serde_json::from_slice(&bytes)?)
}

/// Rehydrate all declared outputs for a cached-hit ledger entry.
pub fn rehydrate_outputs_for_entry(
    workspace_root: &Path,
    entry: &RunLedgerEntry,
) -> Result<BTreeMap<PortName, Value>, WorkflowLedgerError> {
    let mut outputs = BTreeMap::new();
    for (port, hash) in &entry.output_hashes {
        outputs.insert(port.clone(), load_output_payload(workspace_root, hash)?);
    }
    Ok(outputs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workflow::key::{CanonicalKeyPayload, MaterializationKey};
    use crate::workflow::process_registry::ProcessId;

    fn temp_root() -> PathBuf {
        std::env::temp_dir().join(format!(
            "gunbc-workflow-ledger-test-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ))
    }

    fn sample_entry() -> RunLedgerEntry {
        let key = MaterializationKey::new(
            WorkIdentity::new(ProcessId::new("ci"), NodeId::from("ci.codegen")),
            CanonicalKeyPayload {
                key_format_version: 1,
                op_version: 1,
                input_hashes: BTreeMap::new(),
                upstream_keys: BTreeMap::new(),
                policy_version: 1,
            },
        )
        .expect("materialization key should build");

        RunLedgerEntry {
            exec_node_id: NodeId::from("ci.codegen"),
            work_id: key.work_id.clone(),
            key,
            status: LedgerStatus::Executed {
                reason: MissReason::NoPriorRun,
            },
            output_hashes: BTreeMap::new(),
            duration_ms: 4,
        }
    }

    #[test]
    fn ledger_round_trip_persists_entries() {
        let root = temp_root();
        let entry = sample_entry();
        save_global_ledger(&root, std::slice::from_ref(&entry)).expect("save should succeed");
        let loaded = load_global_ledger(&root).expect("load should succeed");
        assert_eq!(loaded, vec![entry]);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn cas_rehydrates_payloads() {
        let root = temp_root();
        let payload = Value::Map(BTreeMap::from([("ok".to_string(), Value::Bool(true))]));
        let hash = store_output_payload(&root, &payload).expect("store payload");

        let mut entry = sample_entry();
        entry.output_hashes.insert(PortName::from("result"), hash);

        let rehydrated = rehydrate_outputs_for_entry(&root, &entry).expect("rehydrate outputs");
        assert_eq!(rehydrated.get(&PortName::from("result")), Some(&payload));
        let _ = fs::remove_dir_all(root);
    }
}
