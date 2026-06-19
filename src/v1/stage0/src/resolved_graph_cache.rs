// resolved_graph_cache.rs — Content-addressed cross-process resolved-graph cache.
//
// Authority row: dsl/std/cache_interface.dag `resolved_graph_cache_facts`.
// Key = subject_digest over (closure module_name→file content) + resolve-logic
// version + intern-seed-set version. Widen→MISS, never narrow→stale.

use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::v1_compiler_compile::SourceFile;
use crate::v1_compiler_infer_items::ResolvedGraph;
use crate::v1_rt::{self, Hash};
use crate::v1_std_core::NewlineIndex;

/// Bump when resolve / normalize / infer / ownership semantics change.
pub const RESOLVE_LOGIC_VERSION: &str = "v2-resolve-1";

/// Bump when `seed_kernel_intern_names` changes.
pub const KERNEL_INTERN_SEED_VERSION: &str = "kernel-seed-1";

const FORMAT_VERSION: u32 = 1;
const MAGIC: &[u8; 8] = b"gunbgrpc";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CacheRejectReason {
    ContentDigestMismatch,
    BackendKeyMalformed,
}

#[derive(Debug, Clone)]
pub enum CacheLookupResult {
    Hit(CachedResolvedGraph),
    Miss,
    RejectedHit(CacheRejectReason),
}

#[derive(Debug, Clone)]
pub struct CachedResolvedGraph {
    pub graph: Rc<ResolvedGraph>,
    pub source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct CachePayload {
    graph: ResolvedGraph,
    source_indices: HashMap<String, NewlineIndex>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheWriteOutcome {
    Written,
    AlreadyExists,
}

pub fn resolved_graph_cache_root_from_env() -> Option<PathBuf> {
    std::env::var_os("GUNBC_RESOLVED_GRAPH_CACHE_DIR").map(PathBuf::from)
}

fn extract_module_path(content: &str) -> Option<String> {
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

/// Content hash over the resolved import closure: sorted module_name → file content.
pub fn closure_content_digest(sources: &[Rc<SourceFile>]) -> Hash {
    let mut pairs: Vec<(String, &str)> = sources
        .iter()
        .map(|s| {
            let module = extract_module_path(&s.content).unwrap_or_else(|| s.path.clone());
            (module, s.content.as_str())
        })
        .collect();
    pairs.sort_by(|a, b| a.0.cmp(&b.0));
    let mut acc = v1_rt::atom_identity_hash("resolved-graph-closure-v1".to_string());
    for (module, content) in pairs {
        acc = v1_rt::hash_combine(acc, v1_rt::atom_identity_hash(module));
        acc = v1_rt::hash_combine(acc, v1_rt::atom_identity_hash(content.to_string()));
    }
    acc
}

/// Subject digest = closure content + resolve-logic version + intern-seed version.
pub fn subject_digest_for_closure(sources: &[Rc<SourceFile>]) -> Hash {
    let mut digest = closure_content_digest(sources);
    digest = v1_rt::hash_combine(
        digest,
        v1_rt::atom_identity_hash(RESOLVE_LOGIC_VERSION.to_string()),
    );
    v1_rt::hash_combine(
        digest,
        v1_rt::atom_identity_hash(KERNEL_INTERN_SEED_VERSION.to_string()),
    )
}

/// Content-hash work subject for one witness: closure cache subject × function name.
/// Keys the eval-tap PerformanceReceipt and matches the cache-subject grain (Phase 0).
///
/// SCAFFOLD — dissolve-on: v2 resolved-graph subject projection subsumes this hash
/// (`docs/plans/realization-measurement-loop.md` Phase 0 table).
pub fn witness_work_subject_key(closure_subject_digest: &str, function: &str) -> Hash {
    v1_rt::hash_combine(
        v1_rt::atom_identity_hash(closure_subject_digest.to_string()),
        v1_rt::atom_identity_hash(function.to_string()),
    )
}

fn unique_temp_path(final_path: &Path) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    final_path.with_extension(format!("{}.{}.tmp", std::process::id(), nanos))
}

fn reclaim_stale_temp(final_path: &Path) {
    let legacy = final_path.with_extension("tmp");
    if legacy.exists() {
        let _ = fs::remove_file(&legacy);
    }
}

fn artifact_path(cache_root: &Path, subject_digest: &str) -> PathBuf {
    let prefix = if subject_digest.len() >= 2 {
        &subject_digest[..2]
    } else {
        subject_digest
    };
    cache_root
        .join(prefix)
        .join(format!("{subject_digest}.bin"))
}

fn payload_content_digest(payload_bytes: &[u8]) -> Hash {
    v1_rt::atom_identity_hash(String::from_utf8_lossy(payload_bytes).into_owned())
}

fn read_cached_file(path: &Path, expected_subject: &str) -> CacheLookupResult {
    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return CacheLookupResult::Miss,
    };
    let mut bytes = Vec::new();
    if file.read_to_end(&mut bytes).is_err() {
        return CacheLookupResult::Miss;
    }
    if bytes.len() < MAGIC.len() + 4 + 16 + 16 + 8 {
        return CacheLookupResult::Miss;
    }
    if &bytes[..MAGIC.len()] != MAGIC {
        return CacheLookupResult::Miss;
    }
    let version = u32::from_le_bytes(bytes[MAGIC.len()..MAGIC.len() + 4].try_into().unwrap());
    if version != FORMAT_VERSION {
        return CacheLookupResult::Miss;
    }
    let mut off = MAGIC.len() + 4;
    let subject = std::str::from_utf8(&bytes[off..off + 16])
        .unwrap_or("")
        .to_string();
    off += 16;
    if subject != expected_subject {
        return CacheLookupResult::RejectedHit(CacheRejectReason::BackendKeyMalformed);
    }
    let stored_content_digest = std::str::from_utf8(&bytes[off..off + 16])
        .unwrap_or("")
        .to_string();
    off += 16;
    let payload_len = u64::from_le_bytes(bytes[off..off + 8].try_into().unwrap()) as usize;
    off += 8;
    if off + payload_len > bytes.len() {
        return CacheLookupResult::Miss;
    }
    let payload_bytes = &bytes[off..off + payload_len];
    let computed = payload_content_digest(payload_bytes);
    if computed != stored_content_digest {
        return CacheLookupResult::RejectedHit(CacheRejectReason::ContentDigestMismatch);
    }
    let payload: CachePayload = match serde_json::from_slice(payload_bytes) {
        Ok(p) => p,
        Err(_) => return CacheLookupResult::Miss,
    };
    let source_indices = Rc::new(
        payload
            .source_indices
            .into_iter()
            .map(|(k, v)| (k, Rc::new(v)))
            .collect(),
    );
    CacheLookupResult::Hit(CachedResolvedGraph {
        graph: Rc::new(payload.graph),
        source_indices,
    })
}

pub fn lookup(cache_root: &Path, subject_digest: &str) -> CacheLookupResult {
    if !v1_rt::is_hash_digest(subject_digest) {
        return CacheLookupResult::RejectedHit(CacheRejectReason::BackendKeyMalformed);
    }
    read_cached_file(&artifact_path(cache_root, subject_digest), subject_digest)
}

pub fn write(
    cache_root: &Path,
    subject_digest: &str,
    graph: &ResolvedGraph,
    source_indices: &HashMap<String, Rc<NewlineIndex>>,
) -> Result<CacheWriteOutcome, String> {
    if !v1_rt::is_hash_digest(subject_digest) {
        return Err("subject_digest must be a 16-char hex hash".to_string());
    }
    let final_path = artifact_path(cache_root, subject_digest);
    if final_path.exists() {
        return Ok(CacheWriteOutcome::AlreadyExists);
    }
    if let Some(parent) = final_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("failed to create cache dir {:?}: {}", parent, e))?;
    }
    let si_plain: HashMap<String, NewlineIndex> = source_indices
        .iter()
        .map(|(k, v)| (k.clone(), (**v).clone()))
        .collect();
    let payload = CachePayload {
        graph: graph.clone(),
        source_indices: si_plain,
    };
    let payload_bytes =
        serde_json::to_vec(&payload).map_err(|e| format!("cache payload encode failed: {e}"))?;
    let content_digest = payload_content_digest(&payload_bytes);

    reclaim_stale_temp(&final_path);
    let temp_path = unique_temp_path(&final_path);
    {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp_path)
            .map_err(|e| format!("failed to open cache temp {:?}: {}", temp_path, e))?;
        file.write_all(MAGIC)
            .map_err(|e| format!("cache write failed: {e}"))?;
        file.write_all(&FORMAT_VERSION.to_le_bytes())
            .map_err(|e| format!("cache write failed: {e}"))?;
        file.write_all(subject_digest.as_bytes())
            .map_err(|e| format!("cache write failed: {e}"))?;
        file.write_all(content_digest.as_bytes())
            .map_err(|e| format!("cache write failed: {e}"))?;
        file.write_all(&(payload_bytes.len() as u64).to_le_bytes())
            .map_err(|e| format!("cache write failed: {e}"))?;
        file.write_all(&payload_bytes)
            .map_err(|e| format!("cache write failed: {e}"))?;
        file.sync_all()
            .map_err(|e| format!("cache fsync failed: {e}"))?;
    }
    match fs::rename(&temp_path, &final_path) {
        Ok(()) => Ok(CacheWriteOutcome::Written),
        Err(_) if final_path.exists() => {
            let _ = fs::remove_file(&temp_path);
            Ok(CacheWriteOutcome::AlreadyExists)
        }
        Err(e) => Err(format!(
            "cache rename {:?} -> {:?}: {}",
            temp_path, final_path, e
        )),
    }
}

/// Write raw bytes at the cache path for a subject digest (test hook for poisoned-hit falsifier).
pub fn write_raw_artifact_for_test(
    cache_root: &Path,
    subject_digest: &str,
    raw_bytes: &[u8],
) -> Result<(), String> {
    let final_path = artifact_path(cache_root, subject_digest);
    if let Some(parent) = final_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("failed to create cache dir {:?}: {}", parent, e))?;
    }
    fs::write(&final_path, raw_bytes).map_err(|e| format!("write raw cache artifact: {e}"))
}

pub fn build_valid_artifact_bytes(
    subject_digest: &str,
    graph: &ResolvedGraph,
    source_indices: &HashMap<String, Rc<NewlineIndex>>,
) -> Result<Vec<u8>, String> {
    let si_plain: HashMap<String, NewlineIndex> = source_indices
        .iter()
        .map(|(k, v)| (k.clone(), (**v).clone()))
        .collect();
    let payload = CachePayload {
        graph: graph.clone(),
        source_indices: si_plain,
    };
    let payload_bytes =
        serde_json::to_vec(&payload).map_err(|e| format!("cache payload encode failed: {e}"))?;
    let content_digest = payload_content_digest(&payload_bytes);
    let mut bytes = Vec::new();
    bytes.extend_from_slice(MAGIC);
    bytes.extend_from_slice(&FORMAT_VERSION.to_le_bytes());
    bytes.extend_from_slice(subject_digest.as_bytes());
    bytes.extend_from_slice(content_digest.as_bytes());
    bytes.extend_from_slice(&(payload_bytes.len() as u64).to_le_bytes());
    bytes.extend_from_slice(&payload_bytes);
    Ok(bytes)
}

/// Test-scoped fixture payload serde (reuses `CachePayload` — no parallel serializer).
/// Dissolve-on: full-P5 v2 `.dag` hermetic inter-stage fixture loader (#5094 follow-up —
/// subsumes serde + born-mark guard; delete these `_for_test` shims so the v1 seed shrinks).
pub fn serialize_fixture_payload_for_test(
    graph: &ResolvedGraph,
    source_indices: &HashMap<String, Rc<NewlineIndex>>,
) -> Result<Vec<u8>, String> {
    let si_plain: HashMap<String, NewlineIndex> = source_indices
        .iter()
        .map(|(k, v)| (k.clone(), (**v).clone()))
        .collect();
    let payload = CachePayload {
        graph: graph.clone(),
        source_indices: si_plain,
    };
    serde_json::to_vec(&payload).map_err(|e| format!("fixture payload encode: {e}"))
}

/// Test-scoped fixture loader (inverse of `serialize_fixture_payload_for_test`).
/// Dissolve-on: full-P5 v2 `.dag` hermetic inter-stage fixture loader (#5094 follow-up —
/// subsumes serde + born-mark guard; delete these `_for_test` shims so the v1 seed shrinks).
pub fn deserialize_fixture_payload_for_test(bytes: &[u8]) -> Result<CachedResolvedGraph, String> {
    let payload: CachePayload =
        serde_json::from_slice(bytes).map_err(|e| format!("fixture payload decode: {e}"))?;
    let source_indices = Rc::new(
        payload
            .source_indices
            .into_iter()
            .map(|(k, v)| (k, Rc::new(v)))
            .collect(),
    );
    Ok(CachedResolvedGraph {
        graph: Rc::new(payload.graph),
        source_indices,
    })
}

/// Guard: binding keys must resolve to their bound names in the fixture's embedded intern_table.
/// Dissolve-on: full-P5 v2 `.dag` hermetic inter-stage fixture loader (#5094 follow-up —
/// born-mark guard inlined in loader; delete this `_for_test` shim).
pub fn validate_fixture_intern_table_for_test(cached: &CachedResolvedGraph) -> Result<(), String> {
    use crate::v1_std_core::intern_str;
    for m in cached.graph.modules.iter() {
        let env = m.type_env.clone();
        for (id, binding) in env.bindings.iter() {
            let resolved = intern_str(env.intern_table.clone(), *id);
            if resolved != binding.name {
                return Err(format!(
                    "fixture intern-table born-mark mismatch: id {id} resolves to {resolved:?}, expected {:?}",
                    binding.name
                ));
            }
        }
    }
    Ok(())
}

// ── Witness verified-green baseline (Phase 1.5a affected-set floor skip) ─────

pub const WITNESS_BASELINE_VERSION: &str = "witness-baseline-v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WitnessBaselineLookup {
    VerifiedGreen,
    Miss,
    RejectedMalformed,
}

pub fn witness_baseline_cache_root_from_env() -> Option<PathBuf> {
    std::env::var_os("GUNBC_WITNESS_BASELINE_CACHE_DIR").map(PathBuf::from)
}

fn witness_subject_digest(closure_digest: &Hash, function: &str) -> Hash {
    let mut digest = closure_digest.clone();
    digest = v1_rt::hash_combine(
        digest,
        v1_rt::atom_identity_hash(WITNESS_BASELINE_VERSION.to_string()),
    );
    v1_rt::hash_combine(digest, v1_rt::atom_identity_hash(function.to_string()))
}

fn witness_baseline_artifact_path(cache_root: &Path, subject_digest: &str) -> PathBuf {
    let prefix = if subject_digest.len() >= 2 {
        &subject_digest[..2]
    } else {
        subject_digest
    };
    cache_root
        .join(prefix)
        .join(format!("{subject_digest}.wbl"))
}

pub fn witness_baseline_lookup(
    cache_root: &Path,
    closure_digest: &Hash,
    function: &str,
) -> WitnessBaselineLookup {
    let subject = witness_subject_digest(closure_digest, function);
    let subject_str = subject.to_string();
    if !v1_rt::is_hash_digest(&subject_str) {
        return WitnessBaselineLookup::RejectedMalformed;
    }
    let path = witness_baseline_artifact_path(cache_root, &subject_str);
    if !path.is_file() {
        return WitnessBaselineLookup::Miss;
    }
    let mut file = match File::open(&path) {
        Ok(f) => f,
        Err(_) => return WitnessBaselineLookup::Miss,
    };
    let mut stored = String::new();
    if file.read_to_string(&mut stored).is_err() {
        return WitnessBaselineLookup::Miss;
    }
    if stored.trim() == subject_str {
        // Tombstone integrity: path is keyed by subject_digest; body re-states it so
        // corruption, partial writes, or wrong-file-at-path fail closed (SCAFFOLD —
        // not an independent witness payload; dissolve-on with kernel consumption).
        WitnessBaselineLookup::VerifiedGreen
    } else {
        WitnessBaselineLookup::RejectedMalformed
    }
}

pub fn witness_baseline_record_verified_green(
    cache_root: &Path,
    closure_digest: &Hash,
    function: &str,
) -> Result<(), String> {
    let subject = witness_subject_digest(closure_digest, function);
    let subject_str = subject.to_string();
    if !v1_rt::is_hash_digest(&subject_str) {
        return Err("witness baseline subject digest malformed".to_string());
    }
    let final_path = witness_baseline_artifact_path(cache_root, &subject_str);
    if final_path.is_file() {
        return Ok(());
    }
    if let Some(parent) = final_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("witness baseline mkdir {:?}: {e}", parent))?;
    }
    let tmp = final_path.with_extension(format!("{}.tmp", std::process::id()));
    {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&tmp)
            .map_err(|e| format!("witness baseline temp write {:?}: {e}", tmp))?;
        file.write_all(subject_str.as_bytes())
            .map_err(|e| format!("witness baseline write: {e}"))?;
        file.sync_all()
            .map_err(|e| format!("witness baseline sync: {e}"))?;
    }
    fs::rename(&tmp, &final_path).map_err(|e| format!("witness baseline rename: {e}"))?;
    Ok(())
}
