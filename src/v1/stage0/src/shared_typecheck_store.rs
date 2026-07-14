//! S2a increment C cross-worker typed-module store (cross-worker-typecheck-share-design.md §4).
//!
//! Only the **typed_module_cache** (the 52% prize path) crosses worker threads when
//! explicitly armed via `build_multi_entry_index_with_shared_caches`. Parse, normalize,
//! ownership memos, and the **intern table** stay per-index on each worker.
//!
//! Normal indexes keep typed results as per-index `Rc` maps (main memory path). This
//! module holds the interim **serde byte transport** for cross-worker share only.
//! Encode/decode run **outside** the `RwLock`. 🟡 dissolve-on: store-path `Rc`→`Arc`
//! on `TypecheckModuleResult` / nested infer carriers (design §4.2).

use std::collections::HashMap as StdHashMap;
use std::rc::Rc;
use std::sync::Arc;

use crate::v1_compiler_infer::TypecheckModuleResult;

/// Cross-worker share shell: typed byte cache + collision registry only.
/// Construct once per explicit cross-worker run; clone the `Arc<RwLock<_>>` to every worker.
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

    pub fn contains_typed(&self, mod_name: &str) -> bool {
        self.typed_module_cache.contains_key(mod_name)
    }

    /// Brief read-lock helper: clone the shared byte snapshot only.
    pub fn clone_typed_bytes(&self, mod_name: &str) -> Option<Arc<Vec<u8>>> {
        self.typed_module_cache.get(mod_name).cloned()
    }

    /// Decode a typed snapshot **without** holding the store lock.
    pub fn decode_typed_snapshot(
        bytes: &[u8],
    ) -> Result<Rc<TypecheckModuleResult>, String> {
        let value: TypecheckModuleResult = serde_json::from_slice(bytes)
            .map_err(|e| format!("shared typecheck store decode: {e}"))?;
        Ok(Rc::new(value))
    }

    /// Encode a typed result **without** holding the store lock.
    pub fn encode_typed_snapshot(
        result: &TypecheckModuleResult,
    ) -> Result<Arc<Vec<u8>>, String> {
        let bytes = serde_json::to_vec(result)
            .map_err(|e| format!("shared typecheck store encode: {e}"))?;
        Ok(Arc::new(bytes))
    }

    /// Insert pre-encoded bytes under a brief write lock.
    pub fn insert_typed_preencoded(&mut self, mod_name: String, bytes: Arc<Vec<u8>>) {
        self.typed_module_cache.insert(mod_name, bytes);
    }

    pub fn get_typed(&self, mod_name: &str) -> Result<Option<Rc<TypecheckModuleResult>>, String> {
        let Some(bytes) = self.clone_typed_bytes(mod_name) else {
            return Ok(None);
        };
        Self::decode_typed_snapshot(bytes.as_slice()).map(Some)
    }

    pub fn insert_typed(
        &mut self,
        mod_name: String,
        result: Rc<TypecheckModuleResult>,
    ) -> Result<(), String> {
        let bytes = Self::encode_typed_snapshot(&result)?;
        self.insert_typed_preencoded(mod_name, bytes);
        Ok(())
    }
}

/// Allocate a **fresh** shared typed store (factory — not a process singleton).
pub fn new_shared_typecheck_caches() -> Arc<std::sync::RwLock<SharedTypecheckCaches>> {
    Arc::new(std::sync::RwLock::new(SharedTypecheckCaches::new()))
}
