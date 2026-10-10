//! Memory composition of the seed's emission, read back by staged release.
//!
//! WHY THIS EXISTS. The self-host step's slot memory is sized from a peak nobody had decomposed:
//! the seed process holds a whole-tree acquisition pool and the closure's resolution beside the
//! emitted crate's cargo build. RSS cannot say which structure holds which bytes, and a freed
//! arena that glibc retains reads as live. So every figure here is a PAIR read in one motion:
//! the resident set, then `malloc_trim`, then the resident set again -- what trim returns was
//! freed-but-retained, what it does not is live.
//!
//! WHAT IT IS NOT. Nothing is released on a production path. `release_stages` drops structures
//! only when `GUNBC_MEMORY_COMPOSITION` is set, and only AFTER the emission has produced the
//! files the caller needs, so the measured run's verdict is unchanged. Each stage's delta is
//! attributed in DROP ORDER: an `Rc` shared by two caches frees at the last holder, so a later
//! stage's figure includes anything an earlier one only released a reference to.
//!
//! Instrument: `GUNBC_MEMORY_COMPOSITION=1 gunbc test //gunbc/instruments:self-host`; the rows
//! are the `[memory-composition]` lines on stderr (parsed by `gunbc.floor_demand`'s sibling
//! `gunbc.self_host_step_memory_demand`).

use std::mem::take;

fn enabled() -> bool {
    std::env::var_os("GUNBC_MEMORY_COMPOSITION").is_some()
}

fn rss_kb() -> Option<u64> {
    super::current_rss_bytes().map(|b| b / 1024)
}

/// One resident-set reading with no side effect, for the seams inside the emission.
pub(super) fn readback(stage: &str) {
    if !enabled() {
        return;
    }
    eprintln!(
        "[memory-composition] stage={stage} rss_kb={:?} peak_kb={:?}",
        rss_kb(),
        super::peak_rss_vhwm_bytes().map(|b| b / 1024)
    );
}

fn stage(name: &str, release: impl FnOnce()) {
    let before = rss_kb();
    release();
    let after_release = rss_kb();
    let trimmed = super::trim_retained_heap();
    let after_trim = rss_kb();
    eprintln!(
        "[memory-composition] release={name} rss_kb_before={before:?} rss_kb_after_release={after_release:?} \
         trim_reclaimed_kb={trimmed:?} rss_kb_after_trim={after_trim:?}"
    );
}

/// COUNTERFACTUAL, behind its own variable: drop the whole-tree pool BEFORE the closure's
/// resolution, so the run's peak says whether the pool is live at the seed's high-water mark or
/// only resident beside it. A structure the resolution still demands is re-acquired through the
/// pool's own miss path (correct, slower), so the verdict of the measured run is unchanged and the
/// re-acquisition shows as the pool's own `[pre-entry]` rows.
pub(super) fn drop_pool_before_resolve() {
    if std::env::var_os("GUNBC_MEMORY_COMPOSITION_DROP_POOL_BEFORE_RESOLVE").is_none() {
        return;
    }
    stage(
        "pool_before_resolve",
        super::pool_acquire::drop_pool_for_measurement,
    );
}

/// Drop the seed's retained structures one at a time, reading the resident set back after each.
pub(super) fn release_stages() {
    if !enabled() {
        return;
    }
    let (entries, content_bytes, heads) = super::pool_acquire::pool_shape();
    eprintln!(
        "[memory-composition] pool_entries={entries} pool_content_bytes={content_bytes} pool_heads_parsed={heads}"
    );
    stage("resolve_store", || {
        super::entry_resolve::PROCESS_RESOLVE_STORE.with(|s| drop(take(&mut *s.borrow_mut())))
    });
    stage("resolve_index", || {
        super::entry_resolve::PROCESS_RESOLVE_INDEX.with(|s| drop(take(&mut *s.borrow_mut())))
    });
    stage("scope_and_reference_indexes", || {
        super::SCOPE_FRAGMENT_CACHES.with(|c| drop(take(&mut *c.borrow_mut())));
        super::REFERENCE_CLOSURE_INDEXES.with(|c| drop(take(&mut *c.borrow_mut())));
        super::SCOPE_ORDER_INDEXES.with(|c| drop(take(&mut *c.borrow_mut())));
        super::REFERENCE_EDGE_CACHE.with(|c| drop(take(&mut *c.borrow_mut())));
    });
    stage("module_graph_facts_and_path_index", || {
        super::MODULE_PATH_INDEX_CACHE.with(|c| drop(take(&mut *c.borrow_mut())));
        super::MODULE_GRAPH_FACTS_CACHE.with(|c| drop(take(&mut *c.borrow_mut())));
    });
    stage("emit_check_and_census_memos", || {
        super::COMPILE_DAG_RUST_EMIT_CHECK_MEMO.with(|m| drop(take(&mut *m.borrow_mut())));
        super::COMPILE_DAG_DIAGNOSTIC_CENSUS_MEMO.with(|m| drop(take(&mut *m.borrow_mut())));
    });
    stage(
        "pool_heads",
        super::pool_acquire::drop_pool_heads_for_measurement,
    );
    stage(
        "pool_content_tokens_newline",
        super::pool_acquire::drop_pool_for_measurement,
    );
}
