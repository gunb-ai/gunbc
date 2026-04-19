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
//! gets a clone of the cached outcome. Unrelated keys compile in parallel —
//! [`compile_cell_for_key`] only holds the map mutex, so the compile itself runs
//! outside the critical section inside [`OnceLock::get_or_init`].
//!
//! **Outcome-aware cache (not bare Dag).** `compile_to_dag` produces two facts:
//! the Dag and the outcome kind (`Ok` vs `Err(Semantic)`). The cache stores
//! [`CachedCompileOutcome`] rather than a bare `Dag` so the outcome kind survives
//! the cache boundary. Without this the distinction would collapse — a later
//! caller inspecting `diagnostics().is_empty()` is reading a proxy, not the
//! actual fact. See `feedback_substrate_principle_audit` Q3 (facts flow
//! forward) and the codex/ChatGPT review on PR #546.
//!
//! **Scope.** Per-test-binary (Rust integration tests are separate processes).
//! Eliminates within-binary redundancy; cross-binary sharing would need
//! serialized bootstrap Dag state on disk (separate project).

use std::collections::HashMap;
use std::sync::{Arc, LazyLock, Mutex, OnceLock};

use v3_compiler::compile_to_dag;
use v3_compiler::dag::Dag;
use v3_compiler::CompileError;

/// Cached compile result. Preserves the outcome kind so callers that care
/// about "did it compile cleanly" read the distinction structurally, not via
/// a `diagnostics().is_empty()` proxy.
#[derive(Debug, Clone)]
pub enum CachedCompileOutcome {
    /// `compile_to_dag` returned `Ok(dag)`.
    Clean(Dag),
    /// `compile_to_dag` returned `Err(CompileError::Semantic(dag))` — the
    /// Dag is partial and carries diagnostics.
    Semantic(Dag),
}

impl CachedCompileOutcome {
    /// Returns a reference to the inner Dag regardless of outcome kind.
    pub fn dag(&self) -> &Dag {
        match self {
            Self::Clean(dag) | Self::Semantic(dag) => dag,
        }
    }

    /// True iff the compile was clean (`Ok`).
    pub fn is_clean(&self) -> bool {
        matches!(self, Self::Clean(_))
    }
}

type CompileCell = Arc<OnceLock<CachedCompileOutcome>>;
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

fn populate(source: &str, file: &str) -> CachedCompileOutcome {
    match compile_to_dag(source, file) {
        Ok(dag) => CachedCompileOutcome::Clean(dag),
        Err(CompileError::Semantic(dag)) => CachedCompileOutcome::Semantic(dag),
        Err(other) => panic!("unexpected structural error: {other:?}"),
    }
}

/// Returns the full cached outcome for `(source, file)`. Use this when you
/// need to distinguish clean vs semantic-error outcomes structurally (e.g.
/// test predicates that check "did this compile?").
pub fn cached_compile_outcome(source: &str, file: &str) -> CachedCompileOutcome {
    let cell = compile_cell_for_key((source.to_string(), file.to_string()));
    cell.get_or_init(|| populate(source, file)).clone()
}

/// Returns the Dag from a cached compile regardless of outcome kind —
/// use when you specifically want to inspect diagnostics on a malformed
/// fixture. Structural errors still panic.
pub fn cached_compile_any(source: &str, file: &str) -> Dag {
    match cached_compile_outcome(source, file) {
        CachedCompileOutcome::Clean(dag) | CachedCompileOutcome::Semantic(dag) => dag,
    }
}

/// Returns the Dag from a cached clean compile. Panics structurally if the
/// cached outcome is `Semantic` — the enum variant is the authority, not a
/// `diagnostics().is_empty()` proxy. The shared cache is safe: whichever
/// helper populates a key first, `cached_compile_to_dag` enforces the
/// clean-compile contract from the cached outcome on read.
pub fn cached_compile_to_dag(source: &str, file: &str) -> Dag {
    match cached_compile_outcome(source, file) {
        CachedCompileOutcome::Clean(dag) => dag,
        CachedCompileOutcome::Semantic(dag) => panic!(
            "fixture expected to compile cleanly but produced semantic errors \
             (source={source:?}, file={file:?}) — diagnostics: {:?}",
            dag.diagnostics().iter().collect::<Vec<_>>()
        ),
    }
}
