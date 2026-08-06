//! S2a increment C cross-worker typed-module store (cross-worker-typecheck-share-design.md §4).
//!
//! Only the **typed_module_cache** (the 52% prize path) crosses worker threads when
//! explicitly armed via `build_multi_entry_index_with_shared_caches`. Parse, normalize,
//! ownership memos, and the **intern table** stay per-index on each worker.
//!
//! Normal indexes keep typed results as per-index `Rc` maps (main memory path). This
//! module holds the interim **serde byte transport** for cross-worker share only.
//! When armed, the shared store is the sole typed-cache authority — `index_insert_typed`
//! never writes per-index `typed_module_cache` (reads decode shared bytes only; avoids
//! Rc+JSON double retention). 🟡 dissolve-on:
//! store-path `Rc`→`Arc` on `TypecheckModuleResult` / nested infer carriers (design §4.2).
//!
//! **Cross-worker serde contract:** `TypecheckModuleResult` serializes authored module/type
//! *names* and diagnostic trees — not per-worker `InternTable` indices — so worker B can decode
//! worker A's byte snapshot against its own intern table without a cross-representation straddle.

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
}

static SHARED_STORE_HIT: AtomicUsize = AtomicUsize::new(0);
static SHARED_STORE_MISS: AtomicUsize = AtomicUsize::new(0);
static SHARED_STORE_ENCODE: AtomicUsize = AtomicUsize::new(0);
static SHARED_STORE_DECODE: AtomicUsize = AtomicUsize::new(0);
static PRIVATE_STORE_FALLBACK: AtomicUsize = AtomicUsize::new(0);

pub fn shared_typecheck_store_counters_snapshot() -> SharedTypecheckStoreCounters {
    SharedTypecheckStoreCounters {
        shared_store_hit: SHARED_STORE_HIT.load(Ordering::SeqCst),
        shared_store_miss: SHARED_STORE_MISS.load(Ordering::SeqCst),
        shared_store_encode: SHARED_STORE_ENCODE.load(Ordering::SeqCst),
        shared_store_decode: SHARED_STORE_DECODE.load(Ordering::SeqCst),
        private_store_fallback: PRIVATE_STORE_FALLBACK.load(Ordering::SeqCst),
    }
}

#[doc(hidden)]
pub fn reset_shared_typecheck_store_counters_for_test() {
    SHARED_STORE_HIT.store(0, Ordering::SeqCst);
    SHARED_STORE_MISS.store(0, Ordering::SeqCst);
    SHARED_STORE_ENCODE.store(0, Ordering::SeqCst);
    SHARED_STORE_DECODE.store(0, Ordering::SeqCst);
    PRIVATE_STORE_FALLBACK.store(0, Ordering::SeqCst);
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

/// Cross-worker share shell: typed byte cache + collision registry only.
/// Construct once per explicit cross-worker run; clone the `Arc<RwLock<_>>` to every worker.
/// Keys are typed-module CONTENT keys (`std.interface_summary.typed_module_key` — source
/// hash ⊕ direct-import interface hashes ⊕ compiler identity), never authored module names;
/// `module_source_identity` stays name→file (it guards the name-keyed graph assembly).
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
        let _guard = guard_arc
            .lock()
            .map_err(|e| format!("typed-module key guard poisoned: {e}"))?;
        f()
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
        let value: TypecheckModuleResult = serde_json::from_slice(bytes)
            .map_err(|e| format!("shared typecheck store decode: {e}"))?;
        Ok(Rc::new(value))
    }

    /// Encode a typed result **without** holding the store lock.
    pub fn encode_typed_snapshot(result: &TypecheckModuleResult) -> Result<Arc<Vec<u8>>, String> {
        record_shared_store_encode();
        let bytes = serde_json::to_vec(result)
            .map_err(|e| format!("shared typecheck store encode: {e}"))?;
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
