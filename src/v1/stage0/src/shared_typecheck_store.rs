//! S2a increment C cross-worker typed-module store (cross-worker-typecheck-share-design.md §4).
//!
//! Only the **typed_module_cache** (the 52% prize path) crosses worker threads when
//! explicitly armed via `build_multi_entry_index_with_shared_caches`. Parse, normalize,
//! ownership memos, and the **intern table** stay per-index on each worker.
//!
//! Normal indexes keep typed results as per-index `Arc` maps (main memory path). This
//! module holds the interim **serde byte transport** for cross-worker share only.
//! When armed, the shared store is the sole typed-cache authority — `index_insert_typed`
//! never writes per-index `typed_module_cache` (reads decode shared bytes only; avoids
//! Arc+JSON double retention).
//!
//! **Cross-worker serde contract:** `TypecheckModuleResult` serializes authored module/type
//! *names* and diagnostic trees — not per-worker `InternTable` indices — so worker B can decode
//! worker A's byte snapshot against its own intern table without a cross-representation straddle.

use std::collections::HashMap as StdHashMap;
use std::sync::Arc;

use crate::v1_compiler_infer::TypecheckModuleResult;

/// Cross-worker share shell: typed byte cache + collision registry only.
/// Construct once per explicit cross-worker run; clone the `Arc<RwLock<_>>` to every worker.
/// Keys are typed-module CONTENT keys (`std.interface_summary.typed_module_key` — source
/// hash ⊕ direct-import interface hashes ⊕ compiler identity), never authored module names;
/// `module_source_identity` stays name→file (it guards the name-keyed graph assembly).
pub struct SharedTypecheckCaches {
    typed_module_cache: StdHashMap<String, Arc<Vec<u8>>>,
    pub module_source_identity: StdHashMap<String, String>,
}

impl SharedTypecheckCaches {
    pub fn new() -> Self {
        Self {
            typed_module_cache: StdHashMap::new(),
            module_source_identity: StdHashMap::new(),
        }
    }

    /// Brief read-lock helper: clone the shared byte snapshot only.
    pub fn clone_typed_bytes(&self, typed_key: &str) -> Option<Arc<Vec<u8>>> {
        self.typed_module_cache.get(typed_key).cloned()
    }

    /// Decode a typed snapshot **without** holding the store lock.
    /// Payload is name-keyed (no intern-table indices) — safe to materialize on any worker index.
    pub fn decode_typed_snapshot(bytes: &[u8]) -> Result<Arc<TypecheckModuleResult>, String> {
        let value: TypecheckModuleResult = serde_json::from_slice(bytes)
            .map_err(|e| format!("shared typecheck store decode: {e}"))?;
        Ok(Arc::new(value))
    }

    /// Encode a typed result **without** holding the store lock.
    pub fn encode_typed_snapshot(result: &TypecheckModuleResult) -> Result<Arc<Vec<u8>>, String> {
        let bytes = serde_json::to_vec(result)
            .map_err(|e| format!("shared typecheck store encode: {e}"))?;
        Ok(Arc::new(bytes))
    }

    /// Insert pre-encoded bytes under a brief write lock.
    pub fn insert_typed_preencoded(&mut self, typed_key: String, bytes: Arc<Vec<u8>>) {
        self.typed_module_cache.insert(typed_key, bytes);
    }

    pub fn get_typed(&self, typed_key: &str) -> Result<Option<Arc<TypecheckModuleResult>>, String> {
        let Some(bytes) = self.clone_typed_bytes(typed_key) else {
            return Ok(None);
        };
        Self::decode_typed_snapshot(bytes.as_slice()).map(Some)
    }

    pub fn insert_typed(
        &mut self,
        typed_key: String,
        result: Arc<TypecheckModuleResult>,
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
