//! Shared compile-to-dag cache for integration tests.
//!
//! **The problem.** Many integration tests call `compile_to_dag(source, file)`
//! per `#[test]`. Each call pays the full bootstrap + pipeline cost (~200ms
//! local, ~1-3s on CI cold runners). With 80+ tests per binary each doing one
//! compile, the full v3 suite accumulates minutes of redundant work compiling
//! the same trivial fixtures over and over.
//!
//! **The fix.** Per-`(source, file)` `OnceLock` cache. Within a test binary, the
//! first caller for a given key pays the compile cost; every subsequent caller
//! gets a clone of the cached Dag. Unrelated keys compile in parallel —
//! [`compile_cell_for_key`] only holds the map mutex, so the compile itself runs
//! outside the critical section inside [`OnceLock::get_or_init`].
//!
//! **Scope.** This cache is per-test-binary (Rust integration tests are separate
//! processes). It eliminates the within-binary redundancy but does not share
//! across test binaries — that would need serialized bootstrap Dag state on
//! disk, which is a separate project.

use std::collections::HashMap;
use std::sync::{Arc, LazyLock, Mutex, OnceLock};

use v3_compiler::compile_to_dag;
use v3_compiler::dag::Dag;
use v3_compiler::CompileError;

type CompileCell = Arc<OnceLock<Dag>>;
type CompileCacheMap = HashMap<(String, String), CompileCell>;

static COMPILE_CACHE: LazyLock<Mutex<CompileCacheMap>> =
    LazyLock::new(|| Mutex::new(CompileCacheMap::new()));

/// Map lookup / insert only — must not call `compile_to_dag`; the guard must
/// not survive past this return, so compilation stays out of the critical
/// section.
fn compile_cell_for_key(key: (String, String)) -> CompileCell {
    let mut guard = COMPILE_CACHE.lock().expect("compile cache mutex");
    guard
        .entry(key)
        .or_insert_with(|| Arc::new(OnceLock::new()))
        .clone()
}

/// Variant of [`cached_compile_to_dag`] that accepts semantic errors — returns
/// the partial Dag (with diagnostics attached) when `compile_to_dag` returns
/// `Err(CompileError::Semantic(dag))`. Structural errors still panic.
///
/// Used by tests that specifically exercise diagnostic emission on malformed
/// sources.
pub fn cached_compile_any(source: &str, file: &str) -> Dag {
    let cell = compile_cell_for_key((source.to_string(), file.to_string()));
    cell.get_or_init(|| match compile_to_dag(source, file) {
        Ok(dag) => dag,
        Err(CompileError::Semantic(dag)) => dag,
        Err(other) => panic!("unexpected structural error: {other:?}"),
    })
    .clone()
}

/// `compile_to_dag`, but cache hits on repeated `(source, file)` keys.
///
/// Panics if the compile result has diagnostics — use this for fixtures that
/// must compile cleanly. For tests that expect semantic errors use
/// [`cached_compile_any`] directly.
///
/// The per-call diagnostic check is contract-enforcement at the **caller
/// boundary**, not at cache-insert time. `cached_compile_to_dag` and
/// `cached_compile_any` share the same `(source, file)` cache key, so if
/// `cached_compile_any` populates a key first with a diagnostic-bearing Dag,
/// a later `cached_compile_to_dag` call for that key must still panic —
/// otherwise clean-compile assertions would become order-dependent across
/// parallel tests.
pub fn cached_compile_to_dag(source: &str, file: &str) -> Dag {
    let dag = cached_compile_any(source, file);
    assert!(
        dag.diagnostics().is_empty(),
        "fixture should compile cleanly (source={source:?}, file={file:?}) — \
         diagnostics: {:?}",
        dag.diagnostics().iter().collect::<Vec<_>>()
    );
    dag
}
