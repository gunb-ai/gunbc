//! S2a increment C cross-worker typed-module store (cross-worker-typecheck-share-design (deleted) §4).
//!
//! Only the **typed_module_cache** (the 52% prize path) crosses worker threads when
//! explicitly armed via `build_multi_entry_index_with_shared_caches`. Parse, normalize,
//! ownership memos, and the **intern table** stay per-index on each worker.
//!
//! Normal indexes keep typed results as per-index `Rc` maps (main memory path); this module is
//! the interim **serde byte transport** for cross-worker share only. When armed, the shared store
//! is the sole typed-cache authority — `index_insert_typed` never writes per-index
//! `typed_module_cache` (reads decode shared bytes only; avoids Rc+JSON double retention).
//! 🟡 dissolve-on: store-path `Rc`→`Arc` on `TypecheckModuleResult` / nested infer carriers
//! (design §4.2).
//!
//! **Cross-worker serde contract:** `TypecheckModuleResult` serializes authored module/type
//! *names* and diagnostic trees — not per-worker `InternTable` indices — so worker B decodes
//! worker A's snapshot against its own intern table without a cross-representation straddle.

// CLIPPY ROSTER -- 1 finding(s) this module trips today, listed one lint per line with
// its count. Until this commit the generated crate root allowed `clippy::all` plus six
// rustc groups on behalf of every module under it, so `cargo clippy --all-targets -- -D
// warnings` decided nothing here; the root now excuses only the generated modules it
// speaks for (v1.compiler.emit_rust generated_rust_lint_relaxations), and this is what
// that leaves visible. The list is MONOTONE NON-INCREASING: a name leaves when its last
// site is repaired, and a lint not named below reds the build, which is the whole point.
#![allow(
    clippy::new_without_default,  // 1
)]

use std::collections::HashMap as StdHashMap;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use crate::v1_compiler_infer::TypecheckModuleResult;

/// Process-wide counters for the cross-worker typed-module byte store. Tests reset
/// via `reset_shared_typecheck_store_counters_for_test`; production paths increment
/// silently.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SharedTypecheckStoreCounters {
    pub shared_store_hit: usize,
    pub shared_store_miss: usize,
    pub shared_store_encode: usize,
    pub shared_store_decode: usize,
    pub private_store_fallback: usize,
    /// Bytes written by `serde_json::to_vec` on the shared-store insert path.
    pub shared_store_encode_bytes: usize,
    /// Bytes read by `serde_json::from_slice` on the shared-store decode path.
    pub shared_store_decode_bytes: usize,
    /// Nanoseconds spent waiting on per-key compute mutexes (width-2 2×2 instrumentation).
    pub shared_store_lock_wait_nanos: u64,
    /// Nanoseconds held inside per-key compute mutexes after acquisition.
    pub shared_store_compute_held_nanos: u64,
}

static SHARED_STORE_HIT: AtomicUsize = AtomicUsize::new(0);
static SHARED_STORE_MISS: AtomicUsize = AtomicUsize::new(0);
static SHARED_STORE_ENCODE: AtomicUsize = AtomicUsize::new(0);
static SHARED_STORE_DECODE: AtomicUsize = AtomicUsize::new(0);
static PRIVATE_STORE_FALLBACK: AtomicUsize = AtomicUsize::new(0);
static SHARED_STORE_ENCODE_BYTES: AtomicUsize = AtomicUsize::new(0);
static SHARED_STORE_DECODE_BYTES: AtomicUsize = AtomicUsize::new(0);
static SHARED_STORE_LOCK_WAIT_NANOS: AtomicUsize = AtomicUsize::new(0);
static SHARED_STORE_COMPUTE_HELD_NANOS: AtomicUsize = AtomicUsize::new(0);

pub fn shared_typecheck_store_counters_snapshot() -> SharedTypecheckStoreCounters {
    SharedTypecheckStoreCounters {
        shared_store_hit: SHARED_STORE_HIT.load(Ordering::SeqCst),
        shared_store_miss: SHARED_STORE_MISS.load(Ordering::SeqCst),
        shared_store_encode: SHARED_STORE_ENCODE.load(Ordering::SeqCst),
        shared_store_decode: SHARED_STORE_DECODE.load(Ordering::SeqCst),
        private_store_fallback: PRIVATE_STORE_FALLBACK.load(Ordering::SeqCst),
        shared_store_encode_bytes: SHARED_STORE_ENCODE_BYTES.load(Ordering::SeqCst),
        shared_store_decode_bytes: SHARED_STORE_DECODE_BYTES.load(Ordering::SeqCst),
        shared_store_lock_wait_nanos: SHARED_STORE_LOCK_WAIT_NANOS.load(Ordering::SeqCst) as u64,
        shared_store_compute_held_nanos: SHARED_STORE_COMPUTE_HELD_NANOS.load(Ordering::SeqCst)
            as u64,
    }
}

#[doc(hidden)]
pub fn reset_shared_typecheck_store_counters_for_test() {
    SHARED_STORE_HIT.store(0, Ordering::SeqCst);
    SHARED_STORE_MISS.store(0, Ordering::SeqCst);
    SHARED_STORE_ENCODE.store(0, Ordering::SeqCst);
    SHARED_STORE_DECODE.store(0, Ordering::SeqCst);
    PRIVATE_STORE_FALLBACK.store(0, Ordering::SeqCst);
    SHARED_STORE_ENCODE_BYTES.store(0, Ordering::SeqCst);
    SHARED_STORE_DECODE_BYTES.store(0, Ordering::SeqCst);
    SHARED_STORE_LOCK_WAIT_NANOS.store(0, Ordering::SeqCst);
    SHARED_STORE_COMPUTE_HELD_NANOS.store(0, Ordering::SeqCst);
}

pub(crate) fn record_shared_store_hit() {
    SHARED_STORE_HIT.fetch_add(1, Ordering::SeqCst);
}

pub(crate) fn record_shared_store_miss() {
    SHARED_STORE_MISS.fetch_add(1, Ordering::SeqCst);
}

pub(crate) fn record_shared_store_encode() {
    SHARED_STORE_ENCODE.fetch_add(1, Ordering::SeqCst);
}

pub(crate) fn record_shared_store_decode() {
    SHARED_STORE_DECODE.fetch_add(1, Ordering::SeqCst);
}

pub(crate) fn record_private_store_fallback() {
    PRIVATE_STORE_FALLBACK.fetch_add(1, Ordering::SeqCst);
}

/// Cross-worker share shell: typed byte cache + collision registry only. Construct once per
/// explicit cross-worker run; clone the `Arc<RwLock<_>>` to every worker. Keys are typed-module
/// CONTENT keys (`std.interface_summary.typed_module_key` — source hash ⊕ direct-import interface
/// hashes ⊕ compiler identity), never authored module names; `module_source_identity` stays
/// name→file (it guards the name-keyed graph assembly).
pub struct SharedTypecheckCaches {
    typed_module_cache: StdHashMap<String, Arc<Vec<u8>>>,
    pub module_source_identity: StdHashMap<String, String>,
    /// Per content-key mutexes: only one worker may compute+insert a typed result for a
    /// given key at a time (width-2 crossover — concurrent miss races otherwise double-pay).
    key_compute_guards: Mutex<StdHashMap<String, Arc<Mutex<()>>>>,
}

impl SharedTypecheckCaches {
    pub fn new() -> Self {
        Self {
            typed_module_cache: StdHashMap::new(),
            module_source_identity: StdHashMap::new(),
            key_compute_guards: Mutex::new(StdHashMap::new()),
        }
    }

    pub fn keyed_compute_guard(&self, typed_key: &str) -> Arc<Mutex<()>> {
        let mut guards = self
            .key_compute_guards
            .lock()
            .expect("shared typecheck key_compute_guards poisoned");
        guards
            .entry(typed_key.to_string())
            .or_insert_with(|| Arc::new(Mutex::new(())))
            .clone()
    }

    /// Serialize typed-module compute for one content key across concurrent workers.
    pub fn with_keyed_compute_lock<R>(
        store: &Arc<std::sync::RwLock<Self>>,
        typed_key: &str,
        f: impl FnOnce() -> Result<R, String>,
    ) -> Result<R, String> {
        let guard_arc = {
            let caches = store
                .read()
                .map_err(|e| format!("shared typecheck store read: {e}"))?;
            caches.keyed_compute_guard(typed_key)
        };
        let wait_started = std::time::Instant::now();
        let guard = guard_arc
            .lock()
            .map_err(|e| format!("typed-module key guard poisoned: {e}"))?;
        SHARED_STORE_LOCK_WAIT_NANOS
            .fetch_add(wait_started.elapsed().as_nanos() as usize, Ordering::SeqCst);
        let compute_started = std::time::Instant::now();
        let out = f();
        SHARED_STORE_COMPUTE_HELD_NANOS.fetch_add(
            compute_started.elapsed().as_nanos() as usize,
            Ordering::SeqCst,
        );
        drop(guard);
        out
    }

    /// Brief read-lock helper: clone the shared byte snapshot only.
    pub fn clone_typed_bytes(&self, typed_key: &str) -> Option<Arc<Vec<u8>>> {
        match self.typed_module_cache.get(typed_key) {
            Some(bytes) => {
                record_shared_store_hit();
                Some(bytes.clone())
            }
            None => {
                record_shared_store_miss();
                None
            }
        }
    }

    /// Decode a typed snapshot **without** holding the store lock.
    /// Payload is name-keyed (no intern-table indices) — safe to materialize on any worker index.
    pub fn decode_typed_snapshot(bytes: &[u8]) -> Result<Rc<TypecheckModuleResult>, String> {
        record_shared_store_decode();
        SHARED_STORE_DECODE_BYTES.fetch_add(bytes.len(), Ordering::SeqCst);
        let value: TypecheckModuleResult = serde_json::from_slice(bytes)
            .map_err(|e| format!("shared typecheck store decode: {e}"))?;
        Ok(Rc::new(value))
    }

    /// Encode a typed result **without** holding the store lock.
    pub fn encode_typed_snapshot(result: &TypecheckModuleResult) -> Result<Arc<Vec<u8>>, String> {
        record_shared_store_encode();
        let bytes = serde_json::to_vec(result)
            .map_err(|e| format!("shared typecheck store encode: {e}"))?;
        SHARED_STORE_ENCODE_BYTES.fetch_add(bytes.len(), Ordering::SeqCst);
        Ok(Arc::new(bytes))
    }

    /// Insert pre-encoded bytes under a brief write lock.
    pub fn insert_typed_preencoded(&mut self, typed_key: String, bytes: Arc<Vec<u8>>) {
        self.typed_module_cache.insert(typed_key, bytes);
    }

    pub fn get_typed(&self, typed_key: &str) -> Result<Option<Rc<TypecheckModuleResult>>, String> {
        let Some(bytes) = self.clone_typed_bytes(typed_key) else {
            return Ok(None);
        };
        Self::decode_typed_snapshot(bytes.as_slice()).map(Some)
    }

    pub fn insert_typed(
        &mut self,
        typed_key: String,
        result: Rc<TypecheckModuleResult>,
    ) -> Result<(), String> {
        let bytes = Self::encode_typed_snapshot(&result)?;
        self.insert_typed_preencoded(typed_key, bytes);
        Ok(())
    }
}

/// Allocate a **fresh** shared typed store (factory — not a process singleton).
pub fn new_shared_typecheck_caches() -> Arc<std::sync::RwLock<SharedTypecheckCaches>> {
    Arc::new(std::sync::RwLock::new(SharedTypecheckCaches::new()))
}

// ---------------------------------------------------------------------------
// Host-persisted backing for the same transport (gunbc.floor_materialization
// host_persisted_typecheck_store). DESIGN section 4.4: the transport above is
// reused as-is; only its BACKING changes from an in-process map to a host
// directory of content-addressed entries, so there is no second representation
// and no second key authority. The key is the caller's `typed_key`
// (std.interface_summary.typed_module_key -- source hash + direct-import
// interface hashes + compiler identity); this layer adds only the two terms a
// LONGER LIFETIME needs and the in-process map never did: the store FORMAT
// version and the SOURCE-ROOT set, both carried as the namespace directory
// rather than folded into the key, so a namespace mismatch cannot collide with
// a content mismatch.
//
// SCOPE OF THE RELIEF DELIVERED HERE, stated so it is not read as more:
// `typed_module_key`'s compiler-identity term is `transform_content_digest()`,
// the content hash of the RUNNING BINARY, and `build.rs` embeds
// `GUNBC_BUILD_IDENTITY` (the checkout's commit) into that binary through
// `cargo:rustc-env`. So a rebuild at ANY new commit -- including a `.dag`-only
// one -- produces different bytes and therefore a different key for every
// module. What this store delivers today is SAME-BUILD, CROSS-PROCESS reuse:
// two processes running ONE built binary over one corpus (the witness fold and
// the generated-artifact drift gate). Cross-push reuse is NOT delivered and is
// not claimed; it needs a compiler identity invariant under a corpus-only
// commit, which is a separate change to that identity's derivation.

use std::path::{Path, PathBuf};

/// Bumped when the encoded payload's meaning changes. Entries under an older
/// version live in a different namespace directory and are never decoded.
const PERSIST_FORMAT_VERSION: u32 = 1;
const PERSIST_MAGIC: &[u8; 8] = b"gunbctms";
/// Default host byte ceiling. Overridable by `GUNBC_TYPED_STORE_PERSIST_MAX_BYTES`;
/// the modeled authority is `gunbc.floor_materialization`
/// `floor_typecheck_store_persist_cap_bytes`, kept in lockstep by
/// `persist_cap_matches_modeled_authority`.
const PERSIST_DEFAULT_CAP_BYTES: u64 = 8_589_934_592;

static PERSIST_HIT: AtomicUsize = AtomicUsize::new(0);
static PERSIST_MISS: AtomicUsize = AtomicUsize::new(0);
static PERSIST_READ_BYTES: AtomicUsize = AtomicUsize::new(0);
static PERSIST_WRITE_BYTES: AtomicUsize = AtomicUsize::new(0);
static PERSIST_REJECTED: AtomicUsize = AtomicUsize::new(0);
static PERSIST_REFUSED_AT_CAPACITY: AtomicUsize = AtomicUsize::new(0);
static PERSIST_IO_ERROR: AtomicUsize = AtomicUsize::new(0);

/// The occupancy the ladder's `RetentionUnobserved` refusal demands: LIVE bytes
/// and entries resident in the store, distinct from the cumulative read/write
/// byte throughput beside them. A throughput counter cannot discharge a
/// residency obligation, so both are carried.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PersistentTypedStoreCounters {
    pub persist_hit: usize,
    pub persist_miss: usize,
    /// Payload bytes decoded from the store (cumulative throughput).
    pub persist_read_bytes: usize,
    /// Payload bytes published to the store (cumulative throughput).
    pub persist_write_bytes: usize,
    /// Entries that were present but failed verification (truncated, wrong
    /// magic/version, key mismatch) -- counted, deleted, and served as a MISS.
    pub persist_rejected: usize,
    /// Publications declined because the byte ceiling was already reached
    /// (`RefuseNewStore`).
    pub persist_refused_at_capacity: usize,
    /// Filesystem operations that failed. The run continues cold; it is never
    /// silent.
    pub persist_io_error: usize,
    /// LIVE occupancy, not throughput: bytes resident in the store.
    pub persist_occupancy_bytes: u64,
    /// LIVE occupancy, not throughput: entries resident in the store.
    pub persist_occupancy_entries: u64,
}

pub fn persistent_typed_store_counters_snapshot() -> PersistentTypedStoreCounters {
    let (bytes, entries) = match persistent_typed_store_if_open() {
        Some(store) => store.occupancy(),
        None => (0, 0),
    };
    PersistentTypedStoreCounters {
        persist_hit: PERSIST_HIT.load(Ordering::SeqCst),
        persist_miss: PERSIST_MISS.load(Ordering::SeqCst),
        persist_read_bytes: PERSIST_READ_BYTES.load(Ordering::SeqCst),
        persist_write_bytes: PERSIST_WRITE_BYTES.load(Ordering::SeqCst),
        persist_rejected: PERSIST_REJECTED.load(Ordering::SeqCst),
        persist_refused_at_capacity: PERSIST_REFUSED_AT_CAPACITY.load(Ordering::SeqCst),
        persist_io_error: PERSIST_IO_ERROR.load(Ordering::SeqCst),
        persist_occupancy_bytes: bytes,
        persist_occupancy_entries: entries,
    }
}

#[doc(hidden)]
pub fn reset_persistent_typed_store_counters_for_test() {
    PERSIST_HIT.store(0, Ordering::SeqCst);
    PERSIST_MISS.store(0, Ordering::SeqCst);
    PERSIST_READ_BYTES.store(0, Ordering::SeqCst);
    PERSIST_WRITE_BYTES.store(0, Ordering::SeqCst);
    PERSIST_REJECTED.store(0, Ordering::SeqCst);
    PERSIST_REFUSED_AT_CAPACITY.store(0, Ordering::SeqCst);
    PERSIST_IO_ERROR.store(0, Ordering::SeqCst);
}

/// The modeled byte ceiling this realization is held under. Read by the
/// lockstep test against `gunbc.floor_materialization`.
pub fn persist_default_cap_bytes() -> u64 {
    PERSIST_DEFAULT_CAP_BYTES
}

pub fn persist_format_version() -> u32 {
    PERSIST_FORMAT_VERSION
}

/// Host-local, content-addressed backing for typed-module snapshots.
///
/// Every failure arm is a COUNTED miss or a COUNTED refusal and never a
/// fabricated result (DESIGN section 5): a verification mismatch is reported as
/// `persist_rejected`, not folded into `persist_miss`, so "the store is sound"
/// stays falsifiable rather than becoming the arm that absorbs everything.
pub struct PersistentTypedStore {
    namespace_root: PathBuf,
    cap_bytes: u64,
    occupancy: Mutex<(u64, u64)>,
}

impl PersistentTypedStore {
    /// `source_roots` enters the NAMESPACE, not the key: resolution is
    /// root-relative, so entries produced under a different root set are a
    /// different question and must not be reachable at all.
    pub fn open(root: &Path, source_roots: &[String], cap_bytes: u64) -> Result<Self, String> {
        let namespace_root = root
            .join(format!("v{PERSIST_FORMAT_VERSION}"))
            .join(source_root_namespace(source_roots));
        std::fs::create_dir_all(&namespace_root).map_err(|e| {
            format!("persistent typed store: cannot create {namespace_root:?}: {e}")
        })?;
        let occupancy = scan_occupancy(&namespace_root);
        Ok(Self {
            namespace_root,
            cap_bytes,
            occupancy: Mutex::new(occupancy),
        })
    }

    pub fn occupancy(&self) -> (u64, u64) {
        *self
            .occupancy
            .lock()
            .expect("persistent typed store occupancy poisoned")
    }

    pub fn cap_bytes(&self) -> u64 {
        self.cap_bytes
    }

    pub fn namespace_root(&self) -> &Path {
        &self.namespace_root
    }

    fn entry_path(&self, typed_key: &str) -> PathBuf {
        let shard = typed_key.get(0..2).unwrap_or("00");
        self.namespace_root.join(shard).join(typed_key)
    }

    /// Verified read. Returns the payload only when magic, format version,
    /// declared payload length and the STORED KEY all agree with what was
    /// asked for; anything else deletes the entry, counts it, and misses.
    pub fn get(&self, typed_key: &str) -> Option<Vec<u8>> {
        let path = self.entry_path(typed_key);
        let raw = match std::fs::read(&path) {
            Ok(raw) => raw,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                PERSIST_MISS.fetch_add(1, Ordering::SeqCst);
                return None;
            }
            Err(_) => {
                PERSIST_IO_ERROR.fetch_add(1, Ordering::SeqCst);
                PERSIST_MISS.fetch_add(1, Ordering::SeqCst);
                return None;
            }
        };
        match decode_entry(&raw, typed_key) {
            Ok(payload) => {
                PERSIST_HIT.fetch_add(1, Ordering::SeqCst);
                PERSIST_READ_BYTES.fetch_add(payload.len(), Ordering::SeqCst);
                Some(payload)
            }
            Err(_) => {
                PERSIST_REJECTED.fetch_add(1, Ordering::SeqCst);
                PERSIST_MISS.fetch_add(1, Ordering::SeqCst);
                self.remove_entry(&path, raw.len() as u64);
                None
            }
        }
    }

    fn remove_entry(&self, path: &Path, stored_len: u64) {
        if std::fs::remove_file(path).is_ok() {
            let mut occ = self
                .occupancy
                .lock()
                .expect("persistent typed store occupancy poisoned");
            occ.0 = occ.0.saturating_sub(stored_len);
            occ.1 = occ.1.saturating_sub(1);
        }
    }

    /// Publish under the ceiling. At capacity the disposition is
    /// `RefuseNewStore` (`std.cache_interface`): the publication is declined
    /// and COUNTED -- never an unbounded write, and never a silent one.
    pub fn put(&self, typed_key: &str, payload: &[u8]) {
        let framed = encode_entry(typed_key, payload);
        let framed_len = framed.len() as u64;
        {
            let occ = self
                .occupancy
                .lock()
                .expect("persistent typed store occupancy poisoned");
            if occ.0.saturating_add(framed_len) > self.cap_bytes {
                PERSIST_REFUSED_AT_CAPACITY.fetch_add(1, Ordering::SeqCst);
                return;
            }
        }
        let path = self.entry_path(typed_key);
        if path.exists() {
            return;
        }
        let Some(parent) = path.parent() else {
            PERSIST_IO_ERROR.fetch_add(1, Ordering::SeqCst);
            return;
        };
        if std::fs::create_dir_all(parent).is_err() {
            PERSIST_IO_ERROR.fetch_add(1, Ordering::SeqCst);
            return;
        }
        // Write-then-rename: a reader never observes a partial entry, so a
        // concurrent run's truncated write cannot become a wrong answer.
        let temp = parent.join(format!(
            "{typed_key}.{}.{}.tmp",
            std::process::id(),
            PERSIST_WRITE_BYTES.load(Ordering::SeqCst)
        ));
        if std::fs::write(&temp, &framed).is_err() {
            PERSIST_IO_ERROR.fetch_add(1, Ordering::SeqCst);
            let _ = std::fs::remove_file(&temp);
            return;
        }
        if std::fs::rename(&temp, &path).is_err() {
            PERSIST_IO_ERROR.fetch_add(1, Ordering::SeqCst);
            let _ = std::fs::remove_file(&temp);
            return;
        }
        PERSIST_WRITE_BYTES.fetch_add(payload.len(), Ordering::SeqCst);
        let mut occ = self
            .occupancy
            .lock()
            .expect("persistent typed store occupancy poisoned");
        occ.0 = occ.0.saturating_add(framed_len);
        occ.1 = occ.1.saturating_add(1);
    }
}

fn source_root_namespace(source_roots: &[String]) -> String {
    let mut sorted: Vec<&str> = source_roots.iter().map(|s| s.as_str()).collect();
    sorted.sort_unstable();
    sorted.dedup();
    let mut acc: u64 = 0xcbf2_9ce4_8422_2325;
    for root in sorted {
        for byte in root.as_bytes().iter().chain(std::iter::once(&0u8)) {
            acc ^= u64::from(*byte);
            acc = acc.wrapping_mul(0x1000_0000_01b3);
        }
    }
    format!("roots-{acc:016x}")
}

fn encode_entry(typed_key: &str, payload: &[u8]) -> Vec<u8> {
    let key = typed_key.as_bytes();
    let mut out = Vec::with_capacity(PERSIST_MAGIC.len() + 4 + 4 + 8 + key.len() + payload.len());
    out.extend_from_slice(PERSIST_MAGIC);
    out.extend_from_slice(&PERSIST_FORMAT_VERSION.to_le_bytes());
    out.extend_from_slice(&(key.len() as u32).to_le_bytes());
    out.extend_from_slice(&(payload.len() as u64).to_le_bytes());
    out.extend_from_slice(key);
    out.extend_from_slice(payload);
    out
}

fn decode_entry(raw: &[u8], expect_key: &str) -> Result<Vec<u8>, String> {
    let header = PERSIST_MAGIC.len() + 4 + 4 + 8;
    if raw.len() < header {
        return Err("entry shorter than its header".to_string());
    }
    if &raw[0..PERSIST_MAGIC.len()] != PERSIST_MAGIC {
        return Err("entry magic mismatch".to_string());
    }
    let mut cursor = PERSIST_MAGIC.len();
    let version = u32::from_le_bytes(raw[cursor..cursor + 4].try_into().unwrap());
    cursor += 4;
    if version != PERSIST_FORMAT_VERSION {
        return Err(format!("entry format version {version} is not current"));
    }
    let key_len = u32::from_le_bytes(raw[cursor..cursor + 4].try_into().unwrap()) as usize;
    cursor += 4;
    let payload_len = u64::from_le_bytes(raw[cursor..cursor + 8].try_into().unwrap()) as usize;
    cursor += 8;
    if raw.len() != header + key_len + payload_len {
        return Err("entry length disagrees with its declared parts".to_string());
    }
    let stored_key = std::str::from_utf8(&raw[cursor..cursor + key_len])
        .map_err(|_| "entry key is not utf-8".to_string())?;
    if stored_key != expect_key {
        return Err("entry key does not match the key it was filed under".to_string());
    }
    cursor += key_len;
    Ok(raw[cursor..].to_vec())
}

fn scan_occupancy(namespace_root: &Path) -> (u64, u64) {
    let mut bytes = 0u64;
    let mut entries = 0u64;
    let mut stack = vec![namespace_root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(read) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in read.flatten() {
            let Ok(meta) = entry.metadata() else { continue };
            if meta.is_dir() {
                stack.push(entry.path());
            } else if entry.path().extension().and_then(|s| s.to_str()) != Some("tmp") {
                bytes = bytes.saturating_add(meta.len());
                entries += 1;
            }
        }
    }
    (bytes, entries)
}

static PERSISTENT_STORE: std::sync::OnceLock<Result<Option<PersistentTypedStore>, String>> =
    std::sync::OnceLock::new();

/// The process's persistent typed store for `source_roots`, or `None` when it
/// is not armed.
///
/// OFF BY DEFAULT and armed only by `GUNBC_TYPED_STORE_PERSIST` naming a
/// directory. The SOURCE-ROOT SET is supplied by the caller's index rather than
/// read from the environment -- resolution is root-relative, so the roots are a
/// fact about the computation and never an operator's assertion about it.
///
/// UNARMED and ARMED-BUT-UNUSABLE are different states with different arms. An
/// absent environment variable means the operator asked for no store, so the
/// run proceeds cold and that is correct: no verdict may depend on a store
/// being available. A directory that was NAMED and cannot be opened is an
/// operator asking for a store they do not have, so it REFUSES with a located
/// diagnostic rather than quietly running cold under a name that promised
/// otherwise (DESIGN section 5). A SECOND root set inside one process refuses
/// for the same reason: entries from one namespace are not answers in another.
pub fn persistent_typed_store_for(
    source_roots: &[String],
) -> Result<Option<&'static PersistentTypedStore>, String> {
    let wanted = source_root_namespace(source_roots);
    let opened = PERSISTENT_STORE.get_or_init(|| {
        let Some(root) = std::env::var_os("GUNBC_TYPED_STORE_PERSIST") else {
            return Ok(None);
        };
        let root = PathBuf::from(root);
        if root.as_os_str().is_empty() {
            return Ok(None);
        }
        let cap = std::env::var("GUNBC_TYPED_STORE_PERSIST_MAX_BYTES")
            .ok()
            .and_then(|raw| raw.trim().parse::<u64>().ok())
            .filter(|n| *n > 0)
            .unwrap_or(PERSIST_DEFAULT_CAP_BYTES);
        PersistentTypedStore::open(&root, source_roots, cap).map(Some)
    });
    let store = match opened {
        Ok(store) => store.as_ref(),
        Err(e) => {
            return Err(format!(
                "persistent typed store refused: GUNBC_TYPED_STORE_PERSIST named a store this \
                 process cannot open ({e}). Arming names a directory, so an unopenable one is a \
                 stopped line, not a quiet cold run"
            ))
        }
    };
    let Some(store) = store else {
        return Ok(None);
    };
    let opened = store
        .namespace_root()
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or_default();
    if opened != wanted {
        return Err(format!(
            "persistent typed store refused: this process opened the store for source-root \
             namespace '{opened}' and is now being asked for '{wanted}'. One process may not \
             serve two root sets from one store -- resolution is root-relative, so entries from \
             one namespace are not answers in the other"
        ));
    }
    Ok(Some(store))
}

/// The store if this process already opened one. Used by the counters and the
/// receipt line, which must observe without arming.
pub fn persistent_typed_store_if_open() -> Option<&'static PersistentTypedStore> {
    PERSISTENT_STORE
        .get()
        .and_then(|s| s.as_ref().ok())
        .and_then(|s| s.as_ref())
}

#[doc(hidden)]
pub fn persistent_typed_store_is_armed_for_test() -> bool {
    persistent_typed_store_if_open().is_some()
}

/// One line naming what the persistent store did this run, for the floor's
/// measurement receipt. Live occupancy is reported beside throughput so the
/// ladder's residency obligation is discharged by a residency number.
pub fn persistent_typed_store_receipt_line() -> Option<String> {
    let store = persistent_typed_store_if_open()?;
    let c = persistent_typed_store_counters_snapshot();
    Some(format!(
        "[typed-store-persist] root={:?} hits={} misses={} rejected={} refused_at_capacity={} \
         io_errors={} read_bytes={} write_bytes={} occupancy_bytes={} occupancy_entries={} \
         cap_bytes={}",
        store.namespace_root(),
        c.persist_hit,
        c.persist_miss,
        c.persist_rejected,
        c.persist_refused_at_capacity,
        c.persist_io_error,
        c.persist_read_bytes,
        c.persist_write_bytes,
        c.persist_occupancy_bytes,
        c.persist_occupancy_entries,
        store.cap_bytes(),
    ))
}

#[cfg(test)]
mod persistent_typed_store_tests {
    use super::*;

    /// A temp directory that removes itself. The store is a filesystem
    /// realization, so its discriminating evidence is a real directory, not a
    /// supplied byte map: the properties under test (truncation, atomicity,
    /// occupancy, the ceiling) are properties of files.
    struct TempRoot(PathBuf);

    impl TempRoot {
        fn new(tag: &str) -> Self {
            let mut path = std::env::temp_dir();
            path.push(format!(
                "gunbc-typed-store-test-{tag}-{}-{:?}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            std::fs::create_dir_all(&path).expect("temp root");
            Self(path)
        }
    }

    impl Drop for TempRoot {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn roots() -> Vec<String> {
        vec!["dag".to_string(), "src/v2".to_string()]
    }

    fn open(root: &TempRoot, cap: u64) -> PersistentTypedStore {
        PersistentTypedStore::open(&root.0, &roots(), cap).expect("store opens")
    }

    /// The positive control: a published entry is served back byte-identically
    /// to a SECOND store object over the same directory -- which is what
    /// "survives the process that computed it" means here.
    #[test]
    fn published_entry_is_served_to_a_later_store_over_the_same_directory() {
        let root = TempRoot::new("roundtrip");
        let payload = b"typed-module-snapshot-bytes".to_vec();
        {
            let writer = open(&root, 1 << 20);
            writer.put("abc123", &payload);
            assert_eq!(writer.occupancy().1, 1, "one entry resident after publish");
        }
        let reader = open(&root, 1 << 20);
        assert_eq!(reader.get("abc123"), Some(payload));
        assert_eq!(
            reader.occupancy().1,
            1,
            "a store opened over an existing directory counts what is already there"
        );
    }

    /// The RED that makes the positive control mean something: a DIFFERENT key
    /// is a miss, so a hit is evidence about the key and not about the
    /// directory being non-empty.
    #[test]
    fn a_different_key_misses() {
        let root = TempRoot::new("different-key");
        let store = open(&root, 1 << 20);
        store.put("abc123", b"payload");
        assert_eq!(store.get("abc124"), None);
    }

    /// A different SOURCE-ROOT set is a different namespace and reaches none of
    /// the first one's entries. Resolution is root-relative, so serving across
    /// that boundary would be a wrong answer, not a lucky hit.
    #[test]
    fn a_different_source_root_set_is_a_different_namespace() {
        let root = TempRoot::new("namespace");
        let a = PersistentTypedStore::open(&root.0, &roots(), 1 << 20).expect("store a");
        a.put("abc123", b"payload");
        let b =
            PersistentTypedStore::open(&root.0, &["dag".to_string()], 1 << 20).expect("store b");
        assert_eq!(b.get("abc123"), None);
        assert_ne!(a.namespace_root(), b.namespace_root());
    }

    /// Truncation is REJECTED and COUNTED, never decoded. The absorbing arm
    /// this forbids is a partial read served as a result (DESIGN section 5).
    #[test]
    fn a_truncated_entry_is_rejected_counted_and_removed() {
        let root = TempRoot::new("truncated");
        let store = open(&root, 1 << 20);
        store.put("abc123", b"a-payload-long-enough-to-truncate");
        let path = store.namespace_root().join("ab").join("abc123");
        let raw = std::fs::read(&path).expect("entry exists");
        std::fs::write(&path, &raw[..raw.len() - 5]).expect("truncate");

        reset_persistent_typed_store_counters_for_test();
        assert_eq!(store.get("abc123"), None, "a truncated entry is not served");
        let counters = persistent_typed_store_counters_snapshot();
        assert_eq!(counters.persist_rejected, 1, "the rejection is counted");
        assert_eq!(counters.persist_miss, 1, "and reported as a miss");
        assert!(
            !path.exists(),
            "the bad entry is removed, not left to be re-read"
        );
    }

    /// An entry filed under one key whose payload declares another is rejected.
    /// This is the wall against a mis-filed or colliding entry silently
    /// answering a question it is not an answer to.
    #[test]
    fn an_entry_whose_stored_key_disagrees_is_rejected() {
        let root = TempRoot::new("key-mismatch");
        let store = open(&root, 1 << 20);
        let framed = encode_entry("someone-elses-key", b"payload");
        let path = store.namespace_root().join("ab").join("abc123");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, framed).unwrap();

        reset_persistent_typed_store_counters_for_test();
        assert_eq!(store.get("abc123"), None);
        assert_eq!(
            persistent_typed_store_counters_snapshot().persist_rejected,
            1
        );
    }

    /// An entry written under an older format version is not decoded. The
    /// format version is the term that makes a codec change a miss rather than
    /// garbage.
    #[test]
    fn a_stale_format_version_is_rejected() {
        let root = TempRoot::new("format-version");
        let store = open(&root, 1 << 20);
        let mut framed = encode_entry("abc123", b"payload");
        framed[PERSIST_MAGIC.len()..PERSIST_MAGIC.len() + 4]
            .copy_from_slice(&(PERSIST_FORMAT_VERSION + 1).to_le_bytes());
        let path = store.namespace_root().join("ab").join("abc123");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, framed).unwrap();
        assert_eq!(store.get("abc123"), None);
    }

    /// At the ceiling the disposition is RefuseNewStore: the publication is
    /// declined and counted, occupancy does not grow past the ceiling, and the
    /// run is merely cold. An unbounded write here is what the ladder's
    /// RetentionUnbounded refusal exists to prevent.
    #[test]
    fn publication_at_the_ceiling_is_refused_and_counted() {
        let root = TempRoot::new("capacity");
        let store = open(&root, 64);
        reset_persistent_typed_store_counters_for_test();
        store.put("aa", &[0u8; 8]);
        let after_first = store.occupancy();
        assert_eq!(after_first.1, 1, "the first small entry fits");
        store.put("bb", &[0u8; 4096]);
        let counters = persistent_typed_store_counters_snapshot();
        assert_eq!(
            counters.persist_refused_at_capacity, 1,
            "the oversized publication is refused"
        );
        assert_eq!(store.occupancy(), after_first, "and occupancy did not grow");
        assert_eq!(store.get("bb"), None, "the refused entry was never stored");
        assert!(
            store.occupancy().0 <= store.cap_bytes(),
            "occupancy stays under the declared ceiling"
        );
    }

    /// Occupancy is LIVE RESIDENCY, not cumulative throughput. A throughput
    /// counter cannot discharge the ladder's RetentionUnobserved obligation, so
    /// this asserts the two move differently: a re-publication of an entry
    /// already present adds no residency.
    #[test]
    fn occupancy_is_residency_and_not_throughput() {
        let root = TempRoot::new("occupancy");
        let store = open(&root, 1 << 20);
        reset_persistent_typed_store_counters_for_test();
        store.put("abc123", b"payload");
        let first = store.occupancy();
        store.put("abc123", b"payload");
        assert_eq!(
            store.occupancy(),
            first,
            "re-publishing a resident entry does not grow residency"
        );
        assert_eq!(
            persistent_typed_store_counters_snapshot().persist_write_bytes,
            b"payload".len(),
            "and the throughput counter records only the one write that happened"
        );
    }

    /// A named directory that cannot be opened is an ERROR, not an empty
    /// store. This is the arm that decides whether an armed-but-unusable store
    /// runs cold or stops the line, and `persistent_typed_store_for` turns this
    /// `Err` into a located refusal: the operator named a store, so silently
    /// proceeding without one would be a promise quietly broken.
    #[test]
    fn a_root_that_cannot_be_opened_is_an_error_and_not_an_empty_store() {
        let root = TempRoot::new("unopenable");
        // A regular FILE standing where the store's directory must be created.
        let blocked = root.0.join("not-a-directory");
        std::fs::write(&blocked, b"occupied").expect("write blocker");
        let opened = PersistentTypedStore::open(&blocked, &roots(), 1 << 20);
        assert!(
            opened.is_err(),
            "an unopenable root must refuse, not report an empty store"
        );
    }

    /// The realization's default ceiling is the modeled one. A literal here
    /// disagreeing with `gunbc.floor_materialization`
    /// `floor_typecheck_store_persist_cap_bytes` is two authorities for one
    /// number (DESIGN section 3), so the test is the lockstep.
    #[test]
    fn persist_cap_matches_modeled_authority() {
        assert_eq!(
            persist_default_cap_bytes(),
            8_589_934_592,
            "floor_typecheck_store_persist_cap_bytes in dag/gunbc/floor/floor_materialization.dag"
        );
    }
}
