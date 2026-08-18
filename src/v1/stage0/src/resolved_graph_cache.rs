// SCAFFOLD (§7 seed-retained HAND-RUST — authority: gunbc.resolved_graph_cache_hand_rust_scaffold;
// witness: dag/test/claim/resolved_graph_cache_hand_rust_witness_test.dag).
use im::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::v1_compiler_compile::SourceFile;
use crate::v1_compiler_infer::{rewire_func_env_parent_links, rewire_type_env_parent_links};
use crate::v1_compiler_infer_items::ResolvedGraph;
use crate::v1_rt::{self, Hash};
use crate::v1_std_core::{ErrorNode, NewlineIndex};
use im::Vector;

/// v4: persisted graphs are assembly-final for import-str identity. Decode no
/// longer replays `rewire_type_env_import_str_binding_identity` (v3 did). v1 and
/// v3 artifacts cold-rebuild. Parent-link and func-env rewire remain decode-time
/// — they repair Rc parent edges, not declaration identity.
const FORMAT_VERSION: u32 = 4;
const MAGIC: &[u8; 8] = b"gunbgrpc";
/// v3 header sentinel: union output not persisted in payload (semantically incomplete).
pub const UNION_PART_ABSENT_DIGEST: &str = "ffffffffffffffff";
const PART_COUNT: usize = 3;
const PART_DESCRIPTOR_LEN: usize = 8 + 8 + 16; // offset, length, digest
const V3_HEADER_LEN: usize =
    MAGIC.len() + 4 + 16 + 16 + 16 + 16 + 8 + PART_COUNT * PART_DESCRIPTOR_LEN;

/// Single-authority mirror of the modeled `SizeBounded` cap:
/// `extdeps.realization.resolved_graph.resolved_graph_cache_cap_bytes`
/// (`dag/extdeps/realization/resolved_graph.dag`, eviction = SizeBounded). Kept
/// in lockstep by `cap_matches_modeled_authority` in the size-bound test.
const RESOLVED_GRAPH_CACHE_CAP_BYTES: u64 = 10_737_418_240;

thread_local! {
    static RESOLVED_GRAPH_CACHE_CAP_OVERRIDE: std::cell::RefCell<Option<u64>> =
        std::cell::RefCell::new(None);
}

/// The byte ceiling the on-disk cache is held under — always the modeled
/// `SizeBounded` cap from `extdeps.realization.resolved_graph` (no production
/// env override; review 47220).
pub fn resolved_graph_cache_cap_bytes() -> u64 {
    if let Some(cap) = RESOLVED_GRAPH_CACHE_CAP_OVERRIDE.with(|c| *c.borrow()) {
        if cap > 0 {
            return cap;
        }
    }
    RESOLVED_GRAPH_CACHE_CAP_BYTES
}

/// Test-only cap injection — gated on `test_hooks` feature (enabled by v1-compiler-tests only).
#[cfg(feature = "test_hooks")]
#[doc(hidden)]
pub fn set_resolved_graph_cache_cap_bytes_for_test(cap: Option<u64>) {
    RESOLVED_GRAPH_CACHE_CAP_OVERRIDE.with(|c| *c.borrow_mut() = cap);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CacheRejectReason {
    ContentDigestMismatch,
    BackendKeyMalformed,
    /// Per-part byte digest matched but serde decode failed — codec drift or corruption.
    PartDecodeFailure,
}

/// Per-output digests and byte sizes for a FORMAT_VERSION 4 artifact — derived
/// from the bounded header without reading payload bytes on the probe path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FaithfulResolvedGraphProbeParts {
    pub graph_digest: Hash,
    pub graph_bytes: u64,
    pub indices_digest: Hash,
    pub indices_bytes: u64,
    pub union_digest: Hash,
    pub union_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheProbeHit {
    pub format_version: u32,
    pub payload_integrity_digest: Hash,
    pub payload_byte_count: u64,
    pub stored_request_key: Hash,
    pub stored_semantic_digest: Hash,
    pub parts: FaithfulResolvedGraphProbeParts,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CacheProbeResult {
    Hit(CacheProbeHit),
    /// Pre-v3 on-disk row for this subject — cold rebuild is the migration disposition.
    LegacyMigrationRequired {
        format_version: u32,
    },
    Miss,
    RejectedHit(CacheRejectReason),
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
    pub compile_clean_diags: Rc<Vector<Rc<ErrorNode>>>,
    pub content_digest: Hash,
    pub payload_byte_count: u64,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct CachePayload {
    graph: ResolvedGraph,
    source_indices: HashMap<String, NewlineIndex>,
}

fn sort_json_value(value: serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Object(map) => {
            let mut keys: Vec<String> = map.keys().cloned().collect();
            keys.sort();
            let mut out = serde_json::Map::new();
            for key in keys {
                out.insert(
                    key.clone(),
                    sort_json_value(map.get(&key).expect("key").clone()),
                );
            }
            serde_json::Value::Object(out)
        }
        serde_json::Value::Array(items) => {
            serde_json::Value::Array(items.into_iter().map(sort_json_value).collect())
        }
        other => other,
    }
}

fn encode_cache_payload(payload: &CachePayload) -> Result<Vec<u8>, String> {
    let value =
        serde_json::to_value(payload).map_err(|e| format!("cache payload value encode: {e}"))?;
    serde_json::to_vec(&sort_json_value(value))
        .map_err(|e| format!("cache payload byte encode: {e}"))
}

fn encode_graph_part(graph: &ResolvedGraph) -> Result<Vec<u8>, String> {
    let value =
        serde_json::to_value(graph).map_err(|e| format!("cache graph value encode: {e}"))?;
    serde_json::to_vec(&sort_json_value(value)).map_err(|e| format!("cache graph encode: {e}"))
}

fn encode_source_indices_part(
    source_indices: &HashMap<String, NewlineIndex>,
) -> Result<Vec<u8>, String> {
    let value = serde_json::to_value(source_indices)
        .map_err(|e| format!("cache indices value encode: {e}"))?;
    serde_json::to_vec(&sort_json_value(value)).map_err(|e| format!("cache indices encode: {e}"))
}

fn encode_compile_clean_union_part(
    compile_clean_diags: &Vector<Rc<ErrorNode>>,
) -> Result<Vec<u8>, String> {
    let plain: Vec<ErrorNode> = compile_clean_diags.iter().map(|d| (**d).clone()).collect();
    let value =
        serde_json::to_value(&plain).map_err(|e| format!("cache union value encode: {e}"))?;
    serde_json::to_vec(&sort_json_value(value)).map_err(|e| format!("cache union encode: {e}"))
}

fn decode_graph_part(bytes: &[u8]) -> Result<ResolvedGraph, String> {
    serde_json::from_slice(bytes).map_err(|e| format!("cache graph decode failed: {e}"))
}

fn decode_source_indices_part(bytes: &[u8]) -> Result<HashMap<String, NewlineIndex>, String> {
    serde_json::from_slice(bytes).map_err(|e| format!("cache indices decode failed: {e}"))
}

fn decode_compile_clean_union_part(bytes: &[u8]) -> Result<Vector<Rc<ErrorNode>>, String> {
    let plain: Vec<ErrorNode> =
        serde_json::from_slice(bytes).map_err(|e| format!("cache union decode failed: {e}"))?;
    Ok(Vector::from(
        plain.into_iter().map(Rc::new).collect::<Vec<_>>(),
    ))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheWriteOutcome {
    Written,
    AlreadyExists,
}

/// Cross-process disk cache is **opt-in**: returns `Some` only when
/// `GUNBC_RESOLVED_GRAPH_CACHE_DIR` is set. Unset env means no cross-process tier
/// (in-process `resolved_graph_memo` still serves same-process repeats).
///
/// Prior always-on `temp_dir()` default was reverted (#5789): it buffered whole
/// cache files and OOM'd concurrent floor shards on a shared host temp tree.
/// Default-on cwd cache is likewise deferred until IO realization streams rather
/// than buffers whole payloads (mechanism-inventory-red-controls; review 47005).
pub fn resolved_graph_cache_root_from_env() -> Option<PathBuf> {
    std::env::var_os("GUNBC_RESOLVED_GRAPH_CACHE_DIR").map(PathBuf::from)
}

thread_local! {
    /// Successful store decodes on this thread. With the process share installed
    /// in front of the store (the ladder's tier ordering: share serves repeats,
    /// store serves the first touch), decodes == distinct subjects touched — a
    /// second decode of one subject within a process is the inversion coming
    /// back. Disclosed so the frequency is observable, never absorbed.
    static DECODE_COUNT: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

pub fn decode_count() -> u64 {
    DECODE_COUNT.with(|c| c.get())
}

fn record_decode() {
    DECODE_COUNT.with(|c| c.set(c.get() + 1));
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

/// Content hash of the running compiler binary — the compiler-identity key term.
/// One authority for every key that must invalidate across a seed rebuild: the
/// resolved-graph subject digest (below) and the typed-module content key
/// (`std.interface_summary.typed_module_key`, cli_run's typed store) both consume
/// this digest rather than re-deriving the executable read.
pub fn transform_content_digest() -> Hash {
    static DIGEST: OnceLock<Hash> = OnceLock::new();
    DIGEST
        .get_or_init(|| {
            let exe = std::env::current_exe().unwrap_or_else(|e| {
                panic!(
                    "resolve cache: cannot locate compiler executable to content-address \
                     the transform: {e}"
                )
            });
            let bytes = fs::read(&exe).unwrap_or_else(|e| {
                panic!(
                    "resolve cache: cannot read compiler executable {:?} to content-address \
                     the transform: {}",
                    exe, e
                )
            });
            v1_rt::bytes_identity_hash(&bytes)
        })
        .clone()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyInputAxis {
    ClosureSubject,
    TransformContent,
}

impl KeyInputAxis {
    pub const ALL: &'static [KeyInputAxis] =
        &[KeyInputAxis::ClosureSubject, KeyInputAxis::TransformContent];
}

#[derive(Debug, Clone)]
pub struct KeyInputMaterials {
    closure_subject: Hash,
    transform_content: Hash,
}

impl KeyInputMaterials {
    pub fn new(closure_subject: Hash, transform_content: Hash) -> Self {
        Self {
            closure_subject,
            transform_content,
        }
    }

    fn materialize(&self, axis: KeyInputAxis) -> Hash {
        match axis {
            KeyInputAxis::ClosureSubject => self.closure_subject.clone(),
            KeyInputAxis::TransformContent => self.transform_content.clone(),
        }
    }
}

pub fn derive_subject_digest(materials: &KeyInputMaterials) -> Hash {
    KeyInputAxis::ALL.iter().fold(
        v1_rt::atom_identity_hash("resolved-graph-subject-v1".to_string()),
        |acc, axis| v1_rt::hash_combine(acc, materials.materialize(*axis)),
    )
}

pub fn subject_digest_for_closure(sources: &[Rc<SourceFile>]) -> Hash {
    let materials =
        KeyInputMaterials::new(closure_content_digest(sources), transform_content_digest());
    derive_subject_digest(&materials)
}

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
    v1_rt::bytes_identity_hash(payload_bytes)
}

fn validate_declared_artifact_len(
    payload_len: u64,
    total_len: usize,
) -> Result<(), CacheRejectReason> {
    let cap = resolved_graph_cache_cap_bytes();
    if payload_len > cap || payload_len > MAX_PART_BYTES * PART_COUNT as u64 {
        return Err(CacheRejectReason::BackendKeyMalformed);
    }
    let expected_total = (V3_HEADER_LEN as u64)
        .checked_add(payload_len)
        .filter(|n| *n <= cap)
        .ok_or(CacheRejectReason::BackendKeyMalformed)?;
    let expected_total_usize = match usize::try_from(expected_total) {
        Ok(n) => n,
        Err(_) => return Err(CacheRejectReason::BackendKeyMalformed),
    };
    if total_len != expected_total_usize {
        return Err(CacheRejectReason::BackendKeyMalformed);
    }
    Ok(())
}

const FAITHFUL_PROBE_FORMAT_VERSION: u32 = 3;
const MAX_PART_BYTES: u64 = 512 * 1024 * 1024;

#[derive(Debug, Clone)]
struct V3PartDescriptor {
    offset: u64,
    length: u64,
    digest: Hash,
}

#[derive(Debug, Clone)]
struct V3Header {
    subject: Hash,
    payload_integrity_digest: Hash,
    stored_request_key: Hash,
    stored_semantic_digest: Hash,
    payload_len: u64,
    parts: [V3PartDescriptor; PART_COUNT],
}

pub fn faithful_probe_unavailable_gap() -> String {
    "faithful resolved-graph disk probe unavailable: cache format version < 3 does not expose bounded per-part descriptors with stored request key and semantic digest"
        .to_string()
}

pub fn supports_faithful_probe() -> bool {
    FORMAT_VERSION >= FAITHFUL_PROBE_FORMAT_VERSION
}

fn parse_v3_part_descriptors(
    bytes: &[u8],
    off: &mut usize,
) -> Result<[V3PartDescriptor; PART_COUNT], CacheRejectReason> {
    let mut parts: [V3PartDescriptor; PART_COUNT] = [
        V3PartDescriptor {
            offset: 0,
            length: 0,
            digest: String::new(),
        },
        V3PartDescriptor {
            offset: 0,
            length: 0,
            digest: String::new(),
        },
        V3PartDescriptor {
            offset: 0,
            length: 0,
            digest: String::new(),
        },
    ];
    for part in parts.iter_mut() {
        if *off + PART_DESCRIPTOR_LEN > bytes.len() {
            return Err(CacheRejectReason::BackendKeyMalformed);
        }
        part.offset = u64::from_le_bytes(bytes[*off..*off + 8].try_into().unwrap());
        *off += 8;
        part.length = u64::from_le_bytes(bytes[*off..*off + 8].try_into().unwrap());
        *off += 8;
        part.digest = std::str::from_utf8(&bytes[*off..*off + 16])
            .map_err(|_| CacheRejectReason::BackendKeyMalformed)?
            .to_string();
        *off += 16;
        if !v1_rt::is_hash_digest(&part.digest) || part.length > MAX_PART_BYTES {
            return Err(CacheRejectReason::BackendKeyMalformed);
        }
    }
    Ok(parts)
}

fn parse_v3_header(
    bytes: &[u8],
    expected_subject: &str,
    total_len: usize,
) -> Result<V3Header, CacheRejectReason> {
    if bytes.len() < V3_HEADER_LEN {
        return Err(CacheRejectReason::BackendKeyMalformed);
    }
    if &bytes[..MAGIC.len()] != MAGIC {
        return Err(CacheRejectReason::BackendKeyMalformed);
    }
    let version = u32::from_le_bytes(bytes[MAGIC.len()..MAGIC.len() + 4].try_into().unwrap());
    if version != FORMAT_VERSION {
        return Err(CacheRejectReason::BackendKeyMalformed);
    }
    let mut off = MAGIC.len() + 4;
    let subject = std::str::from_utf8(&bytes[off..off + 16])
        .map_err(|_| CacheRejectReason::BackendKeyMalformed)?
        .to_string();
    off += 16;
    if subject != expected_subject {
        return Err(CacheRejectReason::BackendKeyMalformed);
    }
    let payload_integrity_digest = std::str::from_utf8(&bytes[off..off + 16])
        .map_err(|_| CacheRejectReason::BackendKeyMalformed)?
        .to_string();
    off += 16;
    let stored_request_key = std::str::from_utf8(&bytes[off..off + 16])
        .map_err(|_| CacheRejectReason::BackendKeyMalformed)?
        .to_string();
    off += 16;
    let stored_semantic_digest = std::str::from_utf8(&bytes[off..off + 16])
        .map_err(|_| CacheRejectReason::BackendKeyMalformed)?
        .to_string();
    off += 16;
    let payload_len = u64::from_le_bytes(bytes[off..off + 8].try_into().unwrap());
    off += 8;
    validate_declared_artifact_len(payload_len, total_len)?;
    let parts = parse_v3_part_descriptors(bytes, &mut off)?;
    if parts[0].offset != 0 {
        return Err(CacheRejectReason::BackendKeyMalformed);
    }
    for i in 1..PART_COUNT {
        let prev = &parts[i - 1];
        let expected_offset = prev
            .offset
            .checked_add(prev.length)
            .ok_or(CacheRejectReason::BackendKeyMalformed)?;
        if parts[i].offset != expected_offset {
            return Err(CacheRejectReason::BackendKeyMalformed);
        }
    }
    let tail = &parts[PART_COUNT - 1];
    let tail_end = tail
        .offset
        .checked_add(tail.length)
        .ok_or(CacheRejectReason::BackendKeyMalformed)?;
    if tail_end != payload_len {
        return Err(CacheRejectReason::BackendKeyMalformed);
    }
    Ok(V3Header {
        subject,
        payload_integrity_digest,
        stored_request_key,
        stored_semantic_digest,
        payload_len,
        parts,
    })
}

fn faithful_parts_from_v3_header(header: &V3Header) -> FaithfulResolvedGraphProbeParts {
    FaithfulResolvedGraphProbeParts {
        graph_digest: header.parts[0].digest.clone(),
        graph_bytes: header.parts[0].length,
        indices_digest: header.parts[1].digest.clone(),
        indices_bytes: header.parts[1].length,
        union_digest: header.parts[2].digest.clone(),
        union_bytes: header.parts[2].length,
    }
}

fn approved_probe_matches_v3_header(
    header: &V3Header,
    approved: &CacheProbeHit,
) -> Result<(), CacheRejectReason> {
    if approved.format_version != FORMAT_VERSION {
        return Err(CacheRejectReason::BackendKeyMalformed);
    }
    if header.payload_integrity_digest != approved.payload_integrity_digest {
        return Err(CacheRejectReason::ContentDigestMismatch);
    }
    if header.payload_len != approved.payload_byte_count {
        return Err(CacheRejectReason::BackendKeyMalformed);
    }
    if header.stored_request_key != approved.stored_request_key {
        return Err(CacheRejectReason::BackendKeyMalformed);
    }
    if header.stored_semantic_digest != approved.stored_semantic_digest {
        return Err(CacheRejectReason::BackendKeyMalformed);
    }
    let approved_parts = &approved.parts;
    let header_parts = faithful_parts_from_v3_header(header);
    if header_parts.graph_digest != approved_parts.graph_digest
        || header_parts.graph_bytes != approved_parts.graph_bytes
        || header_parts.indices_digest != approved_parts.indices_digest
        || header_parts.indices_bytes != approved_parts.indices_bytes
        || header_parts.union_digest != approved_parts.union_digest
        || header_parts.union_bytes != approved_parts.union_bytes
    {
        return Err(CacheRejectReason::BackendKeyMalformed);
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExistingArtifactDisposition {
    Absent,
    LegacyV1Upgrade,
    V3Present,
    Unrecognized,
}

/// Classify an on-disk artifact at the subject path before a current-format write.
/// Legacy v1 and v3 rows are migration replaceable; complete current-format rows are write-once.
fn classify_existing_artifact(
    path: &Path,
    expected_subject: &str,
) -> Result<ExistingArtifactDisposition, String> {
    if !path.exists() {
        return Ok(ExistingArtifactDisposition::Absent);
    }
    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(e) => return Err(format!("open existing cache artifact {:?}: {e}", path)),
    };
    let file_len = match file.metadata() {
        Ok(m) => m.len() as usize,
        Err(e) => return Err(format!("metadata for cache artifact {:?}: {e}", path)),
    };
    if file_len > resolved_graph_cache_cap_bytes() as usize {
        return Ok(ExistingArtifactDisposition::Unrecognized);
    }
    if file_len < MAGIC.len() + 4 {
        return Ok(ExistingArtifactDisposition::Unrecognized);
    }
    let mut prefix = [0u8; MAGIC.len() + 4];
    if file.read_exact(&mut prefix).is_err() {
        return Ok(ExistingArtifactDisposition::Unrecognized);
    }
    if &prefix[..MAGIC.len()] != MAGIC {
        return Ok(ExistingArtifactDisposition::Unrecognized);
    }
    let version = u32::from_le_bytes(prefix[MAGIC.len()..].try_into().unwrap());
    const LEGACY_HEADER_LEN: usize = MAGIC.len() + 4 + 16 + 16 + 8;
    match version {
        1 | 3 => {
            if file_len < LEGACY_HEADER_LEN {
                return Ok(ExistingArtifactDisposition::Unrecognized);
            }
            let mut legacy = vec![0u8; LEGACY_HEADER_LEN - prefix.len()];
            if file.read_exact(&mut legacy).is_err() {
                return Ok(ExistingArtifactDisposition::Unrecognized);
            }
            let subject = std::str::from_utf8(&legacy[0..16])
                .map_err(|e| format!("legacy cache subject utf8: {e}"))?;
            if subject == expected_subject {
                Ok(ExistingArtifactDisposition::LegacyV1Upgrade)
            } else {
                Ok(ExistingArtifactDisposition::Unrecognized)
            }
        }
        FORMAT_VERSION => {
            if file_len < V3_HEADER_LEN {
                return Ok(ExistingArtifactDisposition::Unrecognized);
            }
            let mut header_rest = vec![0u8; V3_HEADER_LEN - prefix.len()];
            if file.read_exact(&mut header_rest).is_err() {
                return Ok(ExistingArtifactDisposition::Unrecognized);
            }
            let header_bytes: Vec<u8> = [prefix.as_ref(), header_rest.as_ref()].concat();
            match parse_v3_header(&header_bytes, expected_subject, file_len) {
                Ok(_) => Ok(ExistingArtifactDisposition::V3Present),
                Err(_) => Ok(ExistingArtifactDisposition::Unrecognized),
            }
        }
        _ => Ok(ExistingArtifactDisposition::Unrecognized),
    }
}

fn read_cached_header(path: &Path, expected_subject: &str) -> CacheProbeResult {
    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return CacheProbeResult::Miss,
        Err(_) => return CacheProbeResult::RejectedHit(CacheRejectReason::BackendKeyMalformed),
    };
    let file_len = match file.metadata() {
        Ok(m) => m.len() as usize,
        Err(_) => return CacheProbeResult::RejectedHit(CacheRejectReason::BackendKeyMalformed),
    };
    if file_len > resolved_graph_cache_cap_bytes() as usize {
        return CacheProbeResult::RejectedHit(CacheRejectReason::BackendKeyMalformed);
    }
    let mut prefix = [0u8; MAGIC.len() + 4];
    if file.read_exact(&mut prefix).is_err() {
        return CacheProbeResult::RejectedHit(CacheRejectReason::BackendKeyMalformed);
    }
    if &prefix[..MAGIC.len()] != MAGIC {
        return CacheProbeResult::RejectedHit(CacheRejectReason::BackendKeyMalformed);
    }
    let version = u32::from_le_bytes(prefix[MAGIC.len()..].try_into().unwrap());
    const LEGACY_HEADER_LEN: usize = MAGIC.len() + 4 + 16 + 16 + 8;
    if version == 1 || version == 3 {
        if version == 1 {
            let mut legacy = vec![0u8; LEGACY_HEADER_LEN - prefix.len()];
            if file.read_exact(&mut legacy).is_err() {
                return CacheProbeResult::RejectedHit(CacheRejectReason::BackendKeyMalformed);
            }
            let subject = std::str::from_utf8(&legacy[0..16])
                .map_err(|_| CacheProbeResult::RejectedHit(CacheRejectReason::BackendKeyMalformed))
                .unwrap_or("");
            if subject != expected_subject {
                return CacheProbeResult::RejectedHit(CacheRejectReason::BackendKeyMalformed);
            }
        }
        return CacheProbeResult::LegacyMigrationRequired {
            format_version: version,
        };
    }
    if version != FORMAT_VERSION {
        return CacheProbeResult::Miss;
    }
    let mut rest = vec![0u8; V3_HEADER_LEN - prefix.len()];
    if file.read_exact(&mut rest).is_err() {
        return CacheProbeResult::RejectedHit(CacheRejectReason::BackendKeyMalformed);
    }
    let header_bytes: Vec<u8> = [prefix.as_ref(), rest.as_ref()].concat();
    match parse_v3_header(&header_bytes, expected_subject, file_len) {
        Ok(parsed) => {
            let parts = faithful_parts_from_v3_header(&parsed);
            CacheProbeResult::Hit(CacheProbeHit {
                format_version: version,
                payload_integrity_digest: parsed.payload_integrity_digest,
                payload_byte_count: parsed.payload_len,
                stored_request_key: parsed.stored_request_key,
                stored_semantic_digest: parsed.stored_semantic_digest,
                parts,
            })
        }
        Err(reason) => CacheProbeResult::RejectedHit(reason),
    }
}

fn decode_v3_payload_from_file(file: &mut File, header: V3Header) -> CacheLookupResult {
    let payload_len = match usize::try_from(header.payload_len) {
        Ok(n) => n,
        Err(_) => {
            return CacheLookupResult::RejectedHit(CacheRejectReason::BackendKeyMalformed);
        }
    };
    let mut payload_bytes = vec![0u8; payload_len];
    if file.read_exact(&mut payload_bytes).is_err() {
        return CacheLookupResult::RejectedHit(CacheRejectReason::BackendKeyMalformed);
    }
    let computed_integrity = payload_content_digest(&payload_bytes);
    if computed_integrity != header.payload_integrity_digest {
        return CacheLookupResult::RejectedHit(CacheRejectReason::ContentDigestMismatch);
    }
    let mut part_slices: [Vec<u8>; PART_COUNT] = [Vec::new(), Vec::new(), Vec::new()];
    for (i, part) in header.parts.iter().enumerate() {
        let start = part.offset as usize;
        let end = start + part.length as usize;
        if end > payload_bytes.len() {
            return CacheLookupResult::RejectedHit(CacheRejectReason::BackendKeyMalformed);
        }
        let slice = &payload_bytes[start..end];
        if v1_rt::bytes_identity_hash(slice) != part.digest {
            return CacheLookupResult::RejectedHit(CacheRejectReason::ContentDigestMismatch);
        }
        part_slices[i] = slice.to_vec();
    }
    let graph_bytes = &part_slices[0];
    let indices_bytes = &part_slices[1];
    let union_bytes = &part_slices[2];
    let decoded_graph = match decode_graph_part(graph_bytes) {
        Ok(g) => g,
        Err(_) => {
            return CacheLookupResult::RejectedHit(CacheRejectReason::PartDecodeFailure);
        }
    };
    let si_plain = match decode_source_indices_part(indices_bytes) {
        Ok(si) => si,
        Err(_) => {
            return CacheLookupResult::RejectedHit(CacheRejectReason::PartDecodeFailure);
        }
    };
    let compile_clean_diags = match decode_compile_clean_union_part(union_bytes) {
        Ok(d) => d,
        Err(_) => {
            return CacheLookupResult::RejectedHit(CacheRejectReason::PartDecodeFailure);
        }
    };
    let source_indices: Rc<HashMap<String, Rc<NewlineIndex>>> =
        Rc::new(si_plain.into_iter().map(|(k, v)| (k, Rc::new(v))).collect());
    let decoded = Rc::new(decoded_graph);
    let modules = rewire_type_env_parent_links(decoded.modules.clone(), source_indices.clone());
    let modules = rewire_func_env_parent_links(modules, source_indices.clone());
    let graph = Rc::new(ResolvedGraph {
        modules,
        item_registry: decoded.item_registry.clone(),
        diagnostics: decoded.diagnostics.clone(),
        emit_graph_info: decoded.emit_graph_info.clone(),
    });
    record_decode();
    CacheLookupResult::Hit(CachedResolvedGraph {
        graph,
        source_indices,
        compile_clean_diags: Rc::new(compile_clean_diags),
        content_digest: header.stored_semantic_digest.clone(),
        payload_byte_count: header.payload_len,
    })
}

fn read_cached_file(path: &Path, expected_subject: &str) -> CacheLookupResult {
    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return CacheLookupResult::Miss,
        Err(_) => {
            return CacheLookupResult::RejectedHit(CacheRejectReason::BackendKeyMalformed);
        }
    };
    let file_len = match file.metadata() {
        Ok(m) => m.len() as usize,
        Err(_) => return CacheLookupResult::RejectedHit(CacheRejectReason::BackendKeyMalformed),
    };
    if file_len > resolved_graph_cache_cap_bytes() as usize {
        return CacheLookupResult::RejectedHit(CacheRejectReason::BackendKeyMalformed);
    }
    if file_len < MAGIC.len() + 4 {
        return CacheLookupResult::Miss;
    }
    let mut prefix = [0u8; MAGIC.len() + 4];
    if file.read_exact(&mut prefix).is_err() {
        return CacheLookupResult::RejectedHit(CacheRejectReason::BackendKeyMalformed);
    }
    let version = u32::from_le_bytes(prefix[MAGIC.len()..].try_into().unwrap());
    if version != FORMAT_VERSION {
        return CacheLookupResult::Miss;
    }
    let mut header_rest = vec![0u8; V3_HEADER_LEN - prefix.len()];
    if file.read_exact(&mut header_rest).is_err() {
        return CacheLookupResult::RejectedHit(CacheRejectReason::BackendKeyMalformed);
    }
    let header_bytes: Vec<u8> = [prefix.as_ref(), header_rest.as_ref()].concat();
    let header = match parse_v3_header(&header_bytes, expected_subject, file_len) {
        Ok(h) => h,
        Err(reason) => return CacheLookupResult::RejectedHit(reason),
    };
    decode_v3_payload_from_file(&mut file, header)
}

fn read_cached_file_verified_probe(
    path: &Path,
    expected_subject: &str,
    approved: &CacheProbeHit,
) -> CacheLookupResult {
    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return CacheLookupResult::Miss,
        Err(_) => {
            return CacheLookupResult::RejectedHit(CacheRejectReason::BackendKeyMalformed);
        }
    };
    let file_len = match file.metadata() {
        Ok(m) => m.len() as usize,
        Err(_) => return CacheLookupResult::RejectedHit(CacheRejectReason::BackendKeyMalformed),
    };
    if file_len > resolved_graph_cache_cap_bytes() as usize {
        return CacheLookupResult::RejectedHit(CacheRejectReason::BackendKeyMalformed);
    }
    if file_len < MAGIC.len() + 4 {
        return CacheLookupResult::Miss;
    }
    let mut prefix = [0u8; MAGIC.len() + 4];
    if file.read_exact(&mut prefix).is_err() {
        return CacheLookupResult::RejectedHit(CacheRejectReason::BackendKeyMalformed);
    }
    let version = u32::from_le_bytes(prefix[MAGIC.len()..].try_into().unwrap());
    if version != FORMAT_VERSION {
        return CacheLookupResult::Miss;
    }
    let mut header_rest = vec![0u8; V3_HEADER_LEN - prefix.len()];
    if file.read_exact(&mut header_rest).is_err() {
        return CacheLookupResult::RejectedHit(CacheRejectReason::BackendKeyMalformed);
    }
    let header_bytes: Vec<u8> = [prefix.as_ref(), header_rest.as_ref()].concat();
    let header = match parse_v3_header(&header_bytes, expected_subject, file_len) {
        Ok(h) => h,
        Err(reason) => return CacheLookupResult::RejectedHit(reason),
    };
    match approved_probe_matches_v3_header(&header, approved) {
        Ok(()) => decode_v3_payload_from_file(&mut file, header),
        Err(reason) => CacheLookupResult::RejectedHit(reason),
    }
}

pub fn lookup(cache_root: &Path, subject_digest: &str) -> CacheLookupResult {
    if !v1_rt::is_hash_digest(subject_digest) {
        return CacheLookupResult::RejectedHit(CacheRejectReason::BackendKeyMalformed);
    }
    read_cached_file(&artifact_path(cache_root, subject_digest), subject_digest)
}

/// Decode a provider-approved v3 probe from disk. The reopened artifact header must
/// match the approved probe exactly before payload bytes are read — closing the TOCTOU
/// gap between logical provider verification and materialization install.
pub fn lookup_verified_probe(
    cache_root: &Path,
    subject_digest: &str,
    approved: &CacheProbeHit,
) -> CacheLookupResult {
    if !v1_rt::is_hash_digest(subject_digest) {
        return CacheLookupResult::RejectedHit(CacheRejectReason::BackendKeyMalformed);
    }
    read_cached_file_verified_probe(
        &artifact_path(cache_root, subject_digest),
        subject_digest,
        approved,
    )
}

pub fn probe(cache_root: &Path, subject_digest: &str) -> CacheProbeResult {
    if !v1_rt::is_hash_digest(subject_digest) {
        return CacheProbeResult::RejectedHit(CacheRejectReason::BackendKeyMalformed);
    }
    read_cached_header(&artifact_path(cache_root, subject_digest), subject_digest)
}

/// Enforce the modeled `SizeBounded` eviction on the on-disk cache: if the total
/// footprint of `*.bin` artifacts exceeds `cap_bytes`, evict oldest-by-mtime
/// first until back under the cap. The cache is content-addressed and write-once,
/// so every artifact is immutable and an evicted entry simply re-resolves on its
/// next miss — making mtime-LRU a safe replacement policy. Best-effort and
/// concurrency-tolerant: a file vanishing under a racing sweep is not an error.
pub fn enforce_size_bound(cache_root: &Path, cap_bytes: u64) {
    let mut artifacts: Vec<(PathBuf, u64, SystemTime)> = Vec::new();
    let mut total: u64 = 0;
    let mut stack = vec![cache_root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let Ok(meta) = entry.metadata() else {
                continue;
            };
            if meta.is_dir() {
                stack.push(path);
            } else if path.extension().and_then(|s| s.to_str()) == Some("bin") {
                let mtime = meta.modified().unwrap_or(UNIX_EPOCH);
                total += meta.len();
                artifacts.push((path, meta.len(), mtime));
            }
        }
    }
    if total <= cap_bytes {
        return;
    }
    // Oldest first; evict until under cap.
    artifacts.sort_by(|a, b| a.2.cmp(&b.2));
    for (path, len, _) in artifacts {
        if total <= cap_bytes {
            break;
        }
        if fs::remove_file(&path).is_ok() {
            total = total.saturating_sub(len);
        }
    }
}

pub struct EncodedResolvedGraphParts {
    pub graph_digest: Hash,
    pub graph_bytes: Vec<u8>,
    pub indices_digest: Hash,
    pub indices_bytes: Vec<u8>,
    pub union_digest: Hash,
    pub union_bytes: Vec<u8>,
}

pub fn encode_resolved_graph_parts(
    graph: &ResolvedGraph,
    source_indices: &HashMap<String, Rc<NewlineIndex>>,
    compile_clean_diags: &Vector<Rc<ErrorNode>>,
) -> Result<EncodedResolvedGraphParts, String> {
    let si_plain: HashMap<String, NewlineIndex> = source_indices
        .iter()
        .map(|(k, v)| (k.clone(), (**v).clone()))
        .collect();
    let graph_bytes = encode_graph_part(graph)?;
    let indices_bytes = encode_source_indices_part(&si_plain)?;
    let union_bytes = encode_compile_clean_union_part(compile_clean_diags)?;
    Ok(EncodedResolvedGraphParts {
        graph_digest: v1_rt::bytes_identity_hash(&graph_bytes),
        graph_bytes,
        indices_digest: v1_rt::bytes_identity_hash(&indices_bytes),
        indices_bytes,
        union_digest: v1_rt::bytes_identity_hash(&union_bytes),
        union_bytes,
    })
}

pub fn write(
    cache_root: &Path,
    subject_digest: &str,
    graph: &ResolvedGraph,
    source_indices: &HashMap<String, Rc<NewlineIndex>>,
    compile_clean_diags: &Vector<Rc<ErrorNode>>,
    stored_request_key: &str,
    stored_semantic_digest: &str,
) -> Result<CacheWriteOutcome, String> {
    if !v1_rt::is_hash_digest(subject_digest) {
        return Err("subject_digest must be a 16-char hex hash".to_string());
    }
    if !v1_rt::is_hash_digest(stored_request_key) || !v1_rt::is_hash_digest(stored_semantic_digest)
    {
        return Err("stored provider keys must be 16-char hex hashes".to_string());
    }
    let final_path = artifact_path(cache_root, subject_digest);
    match classify_existing_artifact(&final_path, subject_digest)? {
        ExistingArtifactDisposition::V3Present => return Ok(CacheWriteOutcome::AlreadyExists),
        ExistingArtifactDisposition::Unrecognized => {
            return Err(format!(
                "refusing v3 write: unrecognized cache artifact at {:?}",
                final_path
            ));
        }
        ExistingArtifactDisposition::Absent | ExistingArtifactDisposition::LegacyV1Upgrade => {}
    }
    if let Some(parent) = final_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("failed to create cache dir {:?}: {}", parent, e))?;
    }
    let encoded = encode_resolved_graph_parts(graph, source_indices, compile_clean_diags)?;
    let payload_bytes = [
        encoded.graph_bytes.as_slice(),
        encoded.indices_bytes.as_slice(),
        encoded.union_bytes.as_slice(),
    ]
    .concat();
    let payload_integrity_digest = payload_content_digest(&payload_bytes);
    let parts = [
        V3PartDescriptor {
            offset: 0,
            length: encoded.graph_bytes.len() as u64,
            digest: encoded.graph_digest.clone(),
        },
        V3PartDescriptor {
            offset: encoded.graph_bytes.len() as u64,
            length: encoded.indices_bytes.len() as u64,
            digest: encoded.indices_digest.clone(),
        },
        V3PartDescriptor {
            offset: (encoded.graph_bytes.len() + encoded.indices_bytes.len()) as u64,
            length: encoded.union_bytes.len() as u64,
            digest: encoded.union_digest.clone(),
        },
    ];

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
        file.write_all(payload_integrity_digest.as_bytes())
            .map_err(|e| format!("cache write failed: {e}"))?;
        file.write_all(stored_request_key.as_bytes())
            .map_err(|e| format!("cache write failed: {e}"))?;
        file.write_all(stored_semantic_digest.as_bytes())
            .map_err(|e| format!("cache write failed: {e}"))?;
        file.write_all(&(payload_bytes.len() as u64).to_le_bytes())
            .map_err(|e| format!("cache write failed: {e}"))?;
        for part in parts {
            file.write_all(&part.offset.to_le_bytes())
                .map_err(|e| format!("cache write failed: {e}"))?;
            file.write_all(&part.length.to_le_bytes())
                .map_err(|e| format!("cache write failed: {e}"))?;
            file.write_all(part.digest.as_bytes())
                .map_err(|e| format!("cache write failed: {e}"))?;
        }
        file.write_all(&payload_bytes)
            .map_err(|e| format!("cache write failed: {e}"))?;
        file.sync_all()
            .map_err(|e| format!("cache fsync failed: {e}"))?;
    }
    match commit_v3_temp_artifact(&temp_path, &final_path, subject_digest, cache_root) {
        Ok(outcome) => Ok(outcome),
        Err(e) => {
            let _ = fs::remove_file(&temp_path);
            Err(e)
        }
    }
}

fn publish_temp_without_replace(temp_path: &Path, final_path: &Path) -> std::io::Result<()> {
    match fs::hard_link(temp_path, final_path) {
        Ok(()) => {
            let _ = fs::remove_file(temp_path);
            Ok(())
        }
        Err(e) => Err(e),
    }
}

fn commit_v3_temp_artifact(
    temp_path: &Path,
    final_path: &Path,
    subject_digest: &str,
    cache_root: &Path,
) -> Result<CacheWriteOutcome, String> {
    match classify_existing_artifact(final_path, subject_digest)? {
        ExistingArtifactDisposition::V3Present => Ok(CacheWriteOutcome::AlreadyExists),
        ExistingArtifactDisposition::Unrecognized => Err(format!(
            "refusing v3 commit: unrecognized cache artifact at {:?}",
            final_path
        )),
        ExistingArtifactDisposition::LegacyV1Upgrade => {
            fs::remove_file(final_path).map_err(|e| {
                format!(
                    "refusing v3 commit: failed to remove legacy cache artifact {:?}: {e}",
                    final_path
                )
            })?;
            match publish_temp_without_replace(temp_path, final_path) {
                Ok(()) => {
                    enforce_size_bound(cache_root, resolved_graph_cache_cap_bytes());
                    Ok(CacheWriteOutcome::Written)
                }
                Err(e) => Err(format!(
                    "cache publish {:?} -> {:?}: {e}",
                    temp_path, final_path
                )),
            }
        }
        ExistingArtifactDisposition::Absent => {
            match publish_temp_without_replace(temp_path, final_path) {
                Ok(()) => {
                    enforce_size_bound(cache_root, resolved_graph_cache_cap_bytes());
                    Ok(CacheWriteOutcome::Written)
                }
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                    match classify_existing_artifact(final_path, subject_digest)? {
                        ExistingArtifactDisposition::V3Present => {
                            Ok(CacheWriteOutcome::AlreadyExists)
                        }
                        other => Err(format!(
                            "cache publish {:?} -> {:?} lost race ({other:?}): {e}",
                            temp_path, final_path
                        )),
                    }
                }
                Err(e) => Err(format!(
                    "cache publish {:?} -> {:?}: {e}",
                    temp_path, final_path
                )),
            }
        }
    }
}

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
    compile_clean_diags: &Vector<Rc<ErrorNode>>,
    stored_request_key: &str,
    stored_semantic_digest: &str,
) -> Result<Vec<u8>, String> {
    let encoded = encode_resolved_graph_parts(graph, source_indices, compile_clean_diags)?;
    let payload_bytes = [
        encoded.graph_bytes.as_slice(),
        encoded.indices_bytes.as_slice(),
        encoded.union_bytes.as_slice(),
    ]
    .concat();
    let payload_integrity_digest = payload_content_digest(&payload_bytes);
    let parts = [
        (0u64, encoded.graph_bytes.len() as u64, encoded.graph_digest),
        (
            encoded.graph_bytes.len() as u64,
            encoded.indices_bytes.len() as u64,
            encoded.indices_digest,
        ),
        (
            (encoded.graph_bytes.len() + encoded.indices_bytes.len()) as u64,
            encoded.union_bytes.len() as u64,
            encoded.union_digest,
        ),
    ];
    let mut bytes = Vec::new();
    bytes.extend_from_slice(MAGIC);
    bytes.extend_from_slice(&FORMAT_VERSION.to_le_bytes());
    bytes.extend_from_slice(subject_digest.as_bytes());
    bytes.extend_from_slice(payload_integrity_digest.as_bytes());
    bytes.extend_from_slice(stored_request_key.as_bytes());
    bytes.extend_from_slice(stored_semantic_digest.as_bytes());
    bytes.extend_from_slice(&(payload_bytes.len() as u64).to_le_bytes());
    for (offset, length, digest) in parts {
        bytes.extend_from_slice(&offset.to_le_bytes());
        bytes.extend_from_slice(&length.to_le_bytes());
        bytes.extend_from_slice(digest.as_bytes());
    }
    bytes.extend_from_slice(&payload_bytes);
    Ok(bytes)
}

pub fn is_union_part_absent(parts: &FaithfulResolvedGraphProbeParts) -> bool {
    parts.union_bytes == 0 && parts.union_digest == UNION_PART_ABSENT_DIGEST
}

pub fn build_incomplete_v3_artifact_bytes(
    subject_digest: &str,
    graph: &ResolvedGraph,
    source_indices: &HashMap<String, Rc<NewlineIndex>>,
    compile_clean_diags: &Vector<Rc<ErrorNode>>,
    stored_request_key: &str,
    stored_semantic_digest: &str,
) -> Result<Vec<u8>, String> {
    let encoded = encode_resolved_graph_parts(graph, source_indices, compile_clean_diags)?;
    let payload_bytes = [
        encoded.graph_bytes.as_slice(),
        encoded.indices_bytes.as_slice(),
    ]
    .concat();
    let payload_integrity_digest = payload_content_digest(&payload_bytes);
    let union_offset = (encoded.graph_bytes.len() + encoded.indices_bytes.len()) as u64;
    let parts = [
        (0u64, encoded.graph_bytes.len() as u64, encoded.graph_digest),
        (
            encoded.graph_bytes.len() as u64,
            encoded.indices_bytes.len() as u64,
            encoded.indices_digest,
        ),
        (union_offset, 0u64, UNION_PART_ABSENT_DIGEST.to_string()),
    ];
    let mut bytes = Vec::new();
    bytes.extend_from_slice(MAGIC);
    bytes.extend_from_slice(&FORMAT_VERSION.to_le_bytes());
    bytes.extend_from_slice(subject_digest.as_bytes());
    bytes.extend_from_slice(payload_integrity_digest.as_bytes());
    bytes.extend_from_slice(stored_request_key.as_bytes());
    bytes.extend_from_slice(stored_semantic_digest.as_bytes());
    bytes.extend_from_slice(&(payload_bytes.len() as u64).to_le_bytes());
    for (offset, length, digest) in parts {
        bytes.extend_from_slice(&offset.to_le_bytes());
        bytes.extend_from_slice(&length.to_le_bytes());
        bytes.extend_from_slice(digest.as_bytes());
    }
    bytes.extend_from_slice(&payload_bytes);
    Ok(bytes)
}

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
    encode_cache_payload(&payload).map_err(|e| format!("fixture payload encode: {e}"))
}

pub fn deserialize_fixture_payload_for_test(bytes: &[u8]) -> Result<CachedResolvedGraph, String> {
    let payload: CachePayload =
        serde_json::from_slice(bytes).map_err(|e| format!("fixture payload decode: {e}"))?;
    let source_indices: Rc<HashMap<String, Rc<NewlineIndex>>> = Rc::new(
        payload
            .source_indices
            .into_iter()
            .map(|(k, v)| (k, Rc::new(v)))
            .collect(),
    );
    let decoded = Rc::new(payload.graph);
    let modules = rewire_type_env_parent_links(decoded.modules.clone(), source_indices.clone());
    let modules = rewire_func_env_parent_links(modules, source_indices.clone());
    Ok(CachedResolvedGraph {
        graph: Rc::new(ResolvedGraph {
            modules,
            item_registry: decoded.item_registry.clone(),
            diagnostics: decoded.diagnostics.clone(),
            emit_graph_info: decoded.emit_graph_info.clone(),
        }),
        source_indices,
        compile_clean_diags: Rc::new(Vector::new()),
        content_digest: payload_content_digest(bytes),
        payload_byte_count: bytes.len() as u64,
    })
}

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

/// NodeKeyedGraphArtifact codec kernel — the interned content-keyed node table.
///
/// SEED-RETAINED (DESIGN §7): declared by the disposition row
/// `node_keyed_graph_codec_seed_disposition` + `node_keyed_graph_codec_seed_note`
/// in `v2.workflow.realization_runner` — hand-Rust because byte-level IO and Rc
/// pointer identity are realization facts the substrate cannot express today.
/// dissolve-on: §4 one-grammar-both-directions emission rows extended to a
/// binary medium (bytes carrier + row table for this format) — the kernel then
/// becomes row-derived emission dispatched from the modeled schema and retires
/// with the seed.
///
/// Modeled authority: `NodeKeyedGraphRow` / `NodeKeyedGraphArtifact` beside
/// `NodeKeyedStore` in `v2.workflow.realization_runner` (the S2b store form).
/// Codec ruling (settled 2026-07-10): a tree codec over the Rc-shared graph
/// un-shares every shared subtree (`serde_json::to_vec` measured ~3.4GiB
/// working set -> 17GiB encode at this cache's write path); serde-streaming
/// and packed-binary-tree keep the unsharing. This kernel encodes each node
/// exactly once, keyed by its content hash, children referenced by hash and
/// never inline; the hash-consed single-pass decode rebuilds the sharing and
/// refuses a forward or missing child ref, which makes an encoded cycle
/// unrepresentable. Hash + row-local size + row land in ONE bottom-up walk —
/// the transitive footprint is never a stored field (the .dag reachable-set
/// fold is the single value-size authority). Hashes are the fnv1a64 authority
/// (`bytes_identity_hash`/`hash_combine`; `std.content_hash` is the modeled
/// surface on the same authority — no third surface).
pub trait NodeKeyedGraphEncode: Sized {
    /// Canonical bytes of the node's row-local payload (child positions excluded).
    fn local_payload_bytes(&self) -> Vec<u8>;

    /// Shared children, in canonical order.
    fn graph_children(&self) -> Vec<Rc<Self>>;

    /// Rebuild from the row-local payload plus the decoded (shared) children.
    fn rebuild(local_payload: &[u8], children: Vec<Rc<Self>>) -> Result<Self, String>;
}

const GRAPH_ARTIFACT_MAGIC: &[u8; 8] = b"gunbngat";
const GRAPH_ARTIFACT_FORMAT_VERSION: u32 = 1;
const HASH_DIGEST_LEN: usize = 16;

fn graph_row_content_hash(payload: &[u8], child_refs: &[Hash]) -> Hash {
    let mut acc = v1_rt::hash_combine(
        v1_rt::atom_identity_hash("node-keyed-graph-row-v1".to_string()),
        v1_rt::bytes_identity_hash(payload),
    );
    for child in child_refs {
        acc = v1_rt::hash_combine(acc, child.clone());
    }
    acc
}

/// One decoded row's facts, row-local only (the host projection the .dag size
/// fold consumes: `encoded_length` == `payload.len()`, transitive footprint is
/// derived by the fold, never stored here).
pub struct NodeKeyedGraphRowFacts {
    pub content_hash: Hash,
    pub child_refs: Vec<Hash>,
    pub payload: Vec<u8>,
}

struct GraphEncodeState {
    rows: Vec<NodeKeyedGraphRowFacts>,
    by_ptr: std::collections::HashMap<*const (), Hash>,
    interned: std::collections::HashSet<Hash>,
    in_progress: std::collections::HashSet<*const ()>,
}

fn graph_encode_node<T: NodeKeyedGraphEncode>(
    root: &Rc<T>,
    state: &mut GraphEncodeState,
) -> Result<Hash, String> {
    let mut stack: Vec<(Rc<T>, bool)> = vec![(root.clone(), false)];
    while let Some((node, expanded)) = stack.pop() {
        let ptr = Rc::as_ptr(&node) as *const ();
        if state.by_ptr.contains_key(&ptr) {
            continue;
        }
        if expanded {
            let payload = node.local_payload_bytes();
            let child_refs: Vec<Hash> = node
                .graph_children()
                .iter()
                .map(|child| {
                    let child_ptr = Rc::as_ptr(child) as *const ();
                    state.by_ptr.get(&child_ptr).cloned().ok_or_else(|| {
                        "graph encode: child left unhashed by post-order walk".to_string()
                    })
                })
                .collect::<Result<_, String>>()?;
            let content_hash = graph_row_content_hash(&payload, &child_refs);
            if state.interned.insert(content_hash.clone()) {
                state.rows.push(NodeKeyedGraphRowFacts {
                    content_hash: content_hash.clone(),
                    child_refs,
                    payload,
                });
            }
            state.by_ptr.insert(ptr, content_hash);
        } else {
            if state.in_progress.contains(&ptr) {
                return Err(
                    "graph encode: cycle detected — the node graph is not acyclic".to_string(),
                );
            }
            state.in_progress.insert(ptr);
            stack.push((node.clone(), true));
            for child in node.graph_children() {
                let child_ptr = Rc::as_ptr(&child) as *const ();
                if !state.by_ptr.contains_key(&child_ptr) {
                    stack.push((child, false));
                }
            }
        }
    }
    state
        .by_ptr
        .get(&(Rc::as_ptr(root) as *const ()))
        .cloned()
        .ok_or_else(|| "graph encode: root left unhashed".to_string())
}

/// Encode `(store_node_hash, value_root)` entries as one interned table.
/// Rows are emitted child-before-parent in first-completion order of a
/// deterministic post-order walk, so re-encoding a decode of these bytes is
/// byte-identical.
pub fn node_keyed_graph_encode<T: NodeKeyedGraphEncode>(
    entries: &[(Hash, Rc<T>)],
) -> Result<Vec<u8>, String> {
    let mut state = GraphEncodeState {
        rows: Vec::new(),
        by_ptr: std::collections::HashMap::new(),
        interned: std::collections::HashSet::new(),
        in_progress: std::collections::HashSet::new(),
    };
    let mut entry_pairs: Vec<(Hash, Hash)> = Vec::new();
    for (store_node, value_root) in entries {
        if !v1_rt::is_hash_digest(store_node) {
            return Err(format!(
                "graph encode: store node key {store_node:?} is not a 16-char hex hash"
            ));
        }
        let root_hash = graph_encode_node(value_root, &mut state)?;
        entry_pairs.push((store_node.clone(), root_hash));
    }
    let mut bytes = Vec::new();
    bytes.extend_from_slice(GRAPH_ARTIFACT_MAGIC);
    bytes.extend_from_slice(&GRAPH_ARTIFACT_FORMAT_VERSION.to_le_bytes());
    bytes.extend_from_slice(&(state.rows.len() as u64).to_le_bytes());
    for row in &state.rows {
        bytes.extend_from_slice(row.content_hash.as_bytes());
        bytes.extend_from_slice(&(row.child_refs.len() as u64).to_le_bytes());
        for child in &row.child_refs {
            bytes.extend_from_slice(child.as_bytes());
        }
        bytes.extend_from_slice(&(row.payload.len() as u64).to_le_bytes());
        bytes.extend_from_slice(&row.payload);
    }
    bytes.extend_from_slice(&(entry_pairs.len() as u64).to_le_bytes());
    for (store_node, value_root) in &entry_pairs {
        bytes.extend_from_slice(store_node.as_bytes());
        bytes.extend_from_slice(value_root.as_bytes());
    }
    Ok(bytes)
}

struct GraphDecodeCursor<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> GraphDecodeCursor<'a> {
    fn take(&mut self, len: usize, what: &str) -> Result<&'a [u8], String> {
        let end = self.offset.checked_add(len).ok_or_else(|| {
            format!(
                "graph decode: length overflow reading {what} at offset {}",
                self.offset
            )
        })?;
        if end > self.bytes.len() {
            return Err(format!(
                "graph decode: truncated artifact reading {what} at offset {} (need {len} bytes, have {})",
                self.offset,
                self.bytes.len() - self.offset
            ));
        }
        let out = &self.bytes[self.offset..end];
        self.offset = end;
        Ok(out)
    }

    fn take_u64(&mut self, what: &str) -> Result<u64, String> {
        let raw = self.take(8, what)?;
        Ok(u64::from_le_bytes(raw.try_into().expect("8-byte slice")))
    }

    fn take_hash(&mut self, what: &str) -> Result<Hash, String> {
        let raw = self.take(HASH_DIGEST_LEN, what)?;
        let hash = std::str::from_utf8(raw)
            .map_err(|_| format!("graph decode: {what} is not utf8 at offset {}", self.offset))?
            .to_string();
        if !v1_rt::is_hash_digest(&hash) {
            return Err(format!(
                "graph decode: {what} {hash:?} is not a 16-char hex hash"
            ));
        }
        Ok(hash)
    }
}

/// Walk the artifact header + rows without rebuilding values: the row-local
/// facts projection (size/topology), shared with the .dag fold semantics.
pub fn node_keyed_graph_row_facts(bytes: &[u8]) -> Result<Vec<NodeKeyedGraphRowFacts>, String> {
    graph_row_facts_with_end(bytes).map(|(rows, _)| rows)
}

fn graph_row_facts_with_end(bytes: &[u8]) -> Result<(Vec<NodeKeyedGraphRowFacts>, usize), String> {
    let mut cursor = GraphDecodeCursor { bytes, offset: 0 };
    let magic = cursor.take(GRAPH_ARTIFACT_MAGIC.len(), "magic")?;
    if magic != GRAPH_ARTIFACT_MAGIC {
        return Err("graph decode: bad magic — not a node-keyed graph artifact".to_string());
    }
    let version_raw = cursor.take(4, "format version")?;
    let version = u32::from_le_bytes(version_raw.try_into().expect("4-byte slice"));
    if version != GRAPH_ARTIFACT_FORMAT_VERSION {
        return Err(format!(
            "graph decode: format version {version} != {GRAPH_ARTIFACT_FORMAT_VERSION}"
        ));
    }
    let row_count = cursor.take_u64("row count")?;
    let mut rows = Vec::new();
    for row_index in 0..row_count {
        let content_hash = cursor.take_hash("row content hash")?;
        let child_count = cursor.take_u64("child ref count")?;
        let mut child_refs = Vec::new();
        for _ in 0..child_count {
            child_refs.push(cursor.take_hash("child ref")?);
        }
        let payload_len = cursor.take_u64("payload length")? as usize;
        let payload = cursor.take(payload_len, "payload")?.to_vec();
        let computed = graph_row_content_hash(&payload, &child_refs);
        if computed != content_hash {
            return Err(format!(
                "graph decode: row {row_index} content hash mismatch (stored {content_hash}, computed {computed})"
            ));
        }
        rows.push(NodeKeyedGraphRowFacts {
            content_hash,
            child_refs,
            payload,
        });
    }
    Ok((rows, cursor.offset))
}

pub struct NodeKeyedGraphDecoded<T> {
    pub entries: Vec<(Hash, Rc<T>)>,
    pub row_count: usize,
}

/// Hash-consed single-pass decode: every child ref resolves to the one `Rc`
/// already decoded for that hash, so structural sharing is rebuilt by
/// construction. A forward or missing child ref refuses (child-before-parent
/// order is the format invariant; an encoded cycle is unrepresentable).
pub fn node_keyed_graph_decode<T: NodeKeyedGraphEncode>(
    bytes: &[u8],
) -> Result<NodeKeyedGraphDecoded<T>, String> {
    let (rows, table_end) = graph_row_facts_with_end(bytes)?;
    let row_count = rows.len();
    let mut by_hash: std::collections::HashMap<Hash, Rc<T>> = std::collections::HashMap::new();
    for (row_index, row) in rows.into_iter().enumerate() {
        if by_hash.contains_key(&row.content_hash) {
            return Err(format!(
                "graph decode: duplicate row for hash {} at row {row_index} — interning violated",
                row.content_hash
            ));
        }
        let children: Vec<Rc<T>> = row
            .child_refs
            .iter()
            .map(|child| {
                by_hash.get(child).cloned().ok_or_else(|| {
                    format!(
                        "graph decode: row {row_index} references child {child} with no earlier row — \
                         forward or missing child ref (child-before-parent order violated)"
                    )
                })
            })
            .collect::<Result<_, String>>()?;
        let node = T::rebuild(&row.payload, children)?;
        by_hash.insert(row.content_hash, Rc::new(node));
    }
    let mut cursor = GraphDecodeCursor {
        bytes,
        offset: table_end,
    };
    let entry_count = cursor.take_u64("entry count")?;
    let mut entries = Vec::new();
    for entry_index in 0..entry_count {
        let store_node = cursor.take_hash("entry store node")?;
        let value_root = cursor.take_hash("entry value root")?;
        let value = by_hash.get(&value_root).cloned().ok_or_else(|| {
            format!(
                "graph decode: entry {entry_index} value root {value_root} has no row in the table"
            )
        })?;
        entries.push((store_node, value));
    }
    if cursor.offset != bytes.len() {
        return Err(format!(
            "graph decode: {} trailing bytes after entries",
            bytes.len() - cursor.offset
        ));
    }
    Ok(NodeKeyedGraphDecoded { entries, row_count })
}

pub trait AuditedRealization {
    fn content_key(&self) -> Hash;

    fn realize_cold(&self) -> Vec<u8>;
}

pub struct HiddenInputProbe<'a> {
    pub axis: &'a str,
    pub perturb: Box<dyn FnMut() + 'a>,
    pub restore: Box<dyn FnMut() + 'a>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CachePurityViolation {
    pub content_key: Hash,
    pub unkeyed_axis: String,
    pub warm_digest: Hash,
    pub cold_digest: Hash,
}

impl std::fmt::Display for CachePurityViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "CACHE PURITY VIOLATION: under fixed content-key {key}, the realization output \
             diverged when hidden input axis `{axis}` changed — a WARM hit serves {warm} but a \
             fresh COLD recompute now yields {cold}. The axis `{axis}` is READ during realization \
             yet is NOT in the content-key, so a cache hit silently serves a stale result. \
             Fail-closed: either KEY on `{axis}` or stop reading it.",
            key = self.content_key,
            axis = self.unkeyed_axis,
            warm = self.warm_digest,
            cold = self.cold_digest,
        )
    }
}

impl std::error::Error for CachePurityViolation {}

pub fn audit_warm_equals_cold(
    realization: &impl AuditedRealization,
    probes: &mut [HiddenInputProbe<'_>],
) -> Result<(), CachePurityViolation> {
    let baseline_key = realization.content_key();
    let warm_digest = v1_rt::bytes_identity_hash(&realization.realize_cold());

    for probe in probes.iter_mut() {
        (probe.perturb)();
        let perturbed_key = realization.content_key();
        let perturbed_bytes = realization.realize_cold();
        (probe.restore)();

        if perturbed_key != baseline_key {
            continue;
        }

        let cold_digest = v1_rt::bytes_identity_hash(&perturbed_bytes);
        if cold_digest != warm_digest {
            return Err(CachePurityViolation {
                content_key: baseline_key,
                unkeyed_axis: probe.axis.to_string(),
                warm_digest,
                cold_digest,
            });
        }
    }
    Ok(())
}
