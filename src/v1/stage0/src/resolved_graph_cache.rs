use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::std_induction::InductiveField;
use crate::v1_compiler_compile::SourceFile;
use crate::v1_compiler_infer_emit_info::EmitGraphInfo;
use crate::v1_compiler_infer_env::{TypeBinding, TypeEnv};
use crate::v1_compiler_infer_items::{ItemInfo, ResolvedGraph, TypedModule};
use crate::v1_compiler_infer_sigs::ResolvedFuncEnv;
use crate::v1_rt::{self, Hash};
use crate::v1_std_core::{ErrorNode, InternTable, NewlineIndex, Node};

const FORMAT_VERSION: u32 = 2;
const MAGIC: &[u8; 8] = b"gunbgrpc";

/// Single-authority mirror of the modeled `SizeBounded` cap:
/// `extdeps.realization.resolved_graph.resolved_graph_cache_cap_bytes`
/// (`dsl/extdeps/realization/resolved_graph.dag`, eviction = SizeBounded). Kept
/// in lockstep by `cap_matches_modeled_authority` in the size-bound test.
const RESOLVED_GRAPH_CACHE_CAP_BYTES: u64 = 10_737_418_240;

/// The byte ceiling the on-disk cache is held under. Defaults to the modeled
/// cap; `GUNBC_RESOLVED_GRAPH_CACHE_CAP_BYTES` overrides it (operator escape
/// hatch / test injection). A malformed or zero override falls back to the
/// modeled default rather than disabling the bound (fail-closed).
pub fn resolved_graph_cache_cap_bytes() -> u64 {
    match std::env::var("GUNBC_RESOLVED_GRAPH_CACHE_CAP_BYTES") {
        Ok(s) => match s.trim().parse::<u64>() {
            Ok(n) if n > 0 => n,
            _ => RESOLVED_GRAPH_CACHE_CAP_BYTES,
        },
        Err(_) => RESOLVED_GRAPH_CACHE_CAP_BYTES,
    }
}

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

// ---------------------------------------------------------------------------
// Interned cache payload (§2 single-authority / content-addressing).
//
// `ResolvedGraph` retains, per module, a fully-merged copy of every binding /
// source-index / inductive-field-list / function it can transitively see. In
// RAM those are `Rc`-shared, but `serde`'s `rc` feature does NOT preserve
// identity — a naive serialize flattens the sharing, so the on-disk payload and
// (worse) the deserialized in-RAM graph balloon to N copies of one fact
// (measured: bindings 8015 stored / 434 distinct, func_env copied once per
// module, source_indices up to 59x). The interned payload stores each unique
// value ONCE in a content-addressed pool and references it by index; decode
// rebuilds one `Rc` per pool entry and hands clones to every referent, so the
// reconstructed graph is value-identical to the original AND shares storage
// the way a fresh resolve does. Pure encoding change — the in-RAM types are
// untouched, so every consumer is unaffected. Round-trip identity is the
// fail-closed oracle (`cache_purity_oracle` / interned_round_trip test).
// ---------------------------------------------------------------------------

/// Content-addressed pool: each distinct value is stored once; `intern` returns
/// a stable index. Keyed by the content hash of the value's serialization, so
/// only byte-identical values collapse (the 9 names carrying two distinct
/// resolved contents across modules stay distinct — id-keying would be lossy).
struct PoolInterner<T> {
    pool: Vec<T>,
    index: HashMap<Hash, u32>,
}

impl<T: serde::Serialize + Clone> PoolInterner<T> {
    fn new() -> Self {
        Self {
            pool: Vec::new(),
            index: HashMap::new(),
        }
    }

    fn intern(&mut self, value: &T) -> u32 {
        let bytes = serde_json::to_vec(value).expect("cache interner: value must serialize");
        let h = v1_rt::bytes_identity_hash(&bytes);
        if let Some(&existing) = self.index.get(&h) {
            return existing;
        }
        let idx = self.pool.len() as u32;
        self.pool.push(value.clone());
        self.index.insert(h, idx);
        idx
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct InternedTypeEnv {
    bindings: HashMap<i64, u32>,
    recursive_types: Vec<i64>,
    recursive_type_set: HashMap<i64, bool>,
    inductive_fields: HashMap<String, u32>,
    source_indices: HashMap<String, u32>,
    intern_table: u32,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct InternedModule {
    module: Rc<Node>,
    items: Rc<Vec<Rc<Node>>>,
    type_env: InternedTypeEnv,
    func_env: u32,
    item_registry: Rc<HashMap<String, Rc<ItemInfo>>>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct CachePayload {
    binding_pool: Vec<Rc<TypeBinding>>,
    newline_pool: Vec<Rc<NewlineIndex>>,
    inductive_pool: Vec<Rc<Vec<Rc<InductiveField>>>>,
    func_env_pool: Vec<Rc<ResolvedFuncEnv>>,
    intern_pool: Vec<Rc<InternTable>>,
    modules: Vec<InternedModule>,
    graph_item_registry: Rc<HashMap<String, Rc<ItemInfo>>>,
    graph_diagnostics: Rc<Vec<Rc<ErrorNode>>>,
    emit_graph_info: Rc<EmitGraphInfo>,
    source_indices: HashMap<String, u32>,
}

fn to_interned_payload(
    graph: &ResolvedGraph,
    source_indices: &HashMap<String, Rc<NewlineIndex>>,
) -> CachePayload {
    let mut bindings = PoolInterner::new();
    let mut newlines = PoolInterner::new();
    let mut inductive = PoolInterner::new();
    let mut func_envs = PoolInterner::new();
    let mut interns = PoolInterner::new();

    let modules: Vec<InternedModule> = graph
        .modules
        .iter()
        .map(|m| {
            let te = &*m.type_env;
            let interned_te = InternedTypeEnv {
                bindings: te
                    .bindings
                    .iter()
                    .map(|(k, v)| (*k, bindings.intern(v)))
                    .collect(),
                recursive_types: (*te.recursive_types).clone(),
                recursive_type_set: (*te.recursive_type_set).clone(),
                inductive_fields: te
                    .inductive_fields
                    .iter()
                    .map(|(k, v)| (k.clone(), inductive.intern(v)))
                    .collect(),
                source_indices: te
                    .source_indices
                    .iter()
                    .map(|(k, v)| (k.clone(), newlines.intern(v)))
                    .collect(),
                intern_table: interns.intern(&te.intern_table),
            };
            InternedModule {
                module: m.module.clone(),
                items: m.items.clone(),
                type_env: interned_te,
                func_env: func_envs.intern(&m.func_env),
                item_registry: m.item_registry.clone(),
            }
        })
        .collect();

    let top_source_indices: HashMap<String, u32> = source_indices
        .iter()
        .map(|(k, v)| (k.clone(), newlines.intern(v)))
        .collect();

    CachePayload {
        binding_pool: bindings.pool,
        newline_pool: newlines.pool,
        inductive_pool: inductive.pool,
        func_env_pool: func_envs.pool,
        intern_pool: interns.pool,
        modules,
        graph_item_registry: graph.item_registry.clone(),
        graph_diagnostics: graph.diagnostics.clone(),
        emit_graph_info: graph.emit_graph_info.clone(),
        source_indices: top_source_indices,
    }
}

/// Rebuild the shared graph from the interned pools. Returns `None` if any pool
/// index is out of bounds (treated as a cache miss — fail-safe re-resolve).
fn from_interned_payload(p: CachePayload) -> Option<CachedResolvedGraph> {
    let binding_pool = p.binding_pool;
    let newline_pool = p.newline_pool;
    let inductive_pool = p.inductive_pool;
    let func_env_pool = p.func_env_pool;
    let intern_pool = p.intern_pool;

    let mut modules: Vec<Rc<TypedModule>> = Vec::with_capacity(p.modules.len());
    for im in p.modules.into_iter() {
        let te = im.type_env;
        let mut bindings: HashMap<i64, Rc<TypeBinding>> = HashMap::with_capacity(te.bindings.len());
        for (k, i) in te.bindings.into_iter() {
            bindings.insert(k, binding_pool.get(i as usize)?.clone());
        }
        let mut inductive_fields: HashMap<String, Rc<Vec<Rc<InductiveField>>>> =
            HashMap::with_capacity(te.inductive_fields.len());
        for (k, i) in te.inductive_fields.into_iter() {
            inductive_fields.insert(k, inductive_pool.get(i as usize)?.clone());
        }
        let mut si: HashMap<String, Rc<NewlineIndex>> =
            HashMap::with_capacity(te.source_indices.len());
        for (k, i) in te.source_indices.into_iter() {
            si.insert(k, newline_pool.get(i as usize)?.clone());
        }
        let type_env = Rc::new(TypeEnv {
            bindings: Rc::new(bindings),
            recursive_types: Rc::new(te.recursive_types),
            recursive_type_set: Rc::new(te.recursive_type_set),
            inductive_fields: Rc::new(inductive_fields),
            source_indices: Rc::new(si),
            intern_table: intern_pool.get(te.intern_table as usize)?.clone(),
        });
        modules.push(Rc::new(TypedModule {
            module: im.module,
            items: im.items,
            type_env,
            func_env: func_env_pool.get(im.func_env as usize)?.clone(),
            item_registry: im.item_registry,
        }));
    }

    let graph = ResolvedGraph {
        modules: Rc::new(modules),
        item_registry: p.graph_item_registry,
        diagnostics: p.graph_diagnostics,
        emit_graph_info: p.emit_graph_info,
    };

    let mut top_si: HashMap<String, Rc<NewlineIndex>> =
        HashMap::with_capacity(p.source_indices.len());
    for (k, i) in p.source_indices.into_iter() {
        top_si.insert(k, newline_pool.get(i as usize)?.clone());
    }

    Some(CachedResolvedGraph {
        graph: Rc::new(graph),
        source_indices: Rc::new(top_si),
    })
}

/// Single authority for encoding the cache payload (used by `write`, the raw
/// artifact builder, and the fixture helper) — one grammar, both directions.
fn encode_payload(
    graph: &ResolvedGraph,
    source_indices: &HashMap<String, Rc<NewlineIndex>>,
) -> Result<Vec<u8>, String> {
    serde_json::to_vec(&to_interned_payload(graph, source_indices))
        .map_err(|e| format!("cache payload encode failed: {e}"))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheWriteOutcome {
    Written,
    AlreadyExists,
}

pub fn resolved_graph_cache_root() -> PathBuf {
    std::env::var_os("GUNBC_RESOLVED_GRAPH_CACHE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            let user_tag = std::env::var("USER")
                .or_else(|_| std::env::var("USERNAME"))
                .unwrap_or_else(|_| "shared".to_string());
            std::env::temp_dir().join(format!("gunbc-rg-cache-{user_tag}"))
        })
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

fn transform_content_digest() -> Hash {
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
    match from_interned_payload(payload) {
        Some(cached) => CacheLookupResult::Hit(cached),
        None => CacheLookupResult::Miss,
    }
}

pub fn lookup(cache_root: &Path, subject_digest: &str) -> CacheLookupResult {
    if !v1_rt::is_hash_digest(subject_digest) {
        return CacheLookupResult::RejectedHit(CacheRejectReason::BackendKeyMalformed);
    }
    read_cached_file(&artifact_path(cache_root, subject_digest), subject_digest)
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
    let payload_bytes = encode_payload(graph, source_indices)?;
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
        Ok(()) => {
            enforce_size_bound(cache_root, resolved_graph_cache_cap_bytes());
            Ok(CacheWriteOutcome::Written)
        }
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
    let payload_bytes = encode_payload(graph, source_indices)?;
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

pub fn serialize_fixture_payload_for_test(
    graph: &ResolvedGraph,
    source_indices: &HashMap<String, Rc<NewlineIndex>>,
) -> Result<Vec<u8>, String> {
    encode_payload(graph, source_indices).map_err(|e| format!("fixture payload encode: {e}"))
}

pub fn deserialize_fixture_payload_for_test(bytes: &[u8]) -> Result<CachedResolvedGraph, String> {
    let payload: CachePayload =
        serde_json::from_slice(bytes).map_err(|e| format!("fixture payload decode: {e}"))?;
    from_interned_payload(payload)
        .ok_or_else(|| "fixture payload decode: pool index out of bounds".to_string())
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
