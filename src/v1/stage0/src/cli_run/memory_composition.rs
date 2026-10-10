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
//! Instrument: `GUNBC_MEMORY_COMPOSITION=1 GUNBC_MEMORY_COMPOSITION_RECEIPT=<file>
//! GUNBC_MEMORY_COMPOSITION_CONFIGURATION=<cell> gunbc test //gunbc/instruments:self-host`.
//! The seed itself appends the receipt that `gunbc.self_host_step_memory_demand` reads: one
//! `bucket` line per released structure and one `stage` line per in-emission resident-set
//! readback, both from the same run, each carrying the configuration cell as its last column.
//! The module reads the file it is pointed at; it never carries a figure of its own.

use std::mem::take;

fn enabled() -> bool {
    std::env::var_os("GUNBC_MEMORY_COMPOSITION").is_some()
}

fn rss_kb() -> Option<u64> {
    super::current_rss_bytes().map(|b| b / 1024)
}

/// The receipt is the measurement's only output, so a reading the writer cannot make or cannot
/// write stops the instrument run (exit 2: no observation) rather than leaving a partial file or a
/// fabricated number the reader would accept.
fn refuse(what: &str) -> ! {
    eprintln!("[memory-composition] REFUSED: {what}; the receipt is not a measurement");
    std::process::exit(2)
}

/// One TSV cell naming the instrument configuration (pool policy, scope, run). Required whenever
/// a receipt path is set: a line without it cannot be selected by the reader.
fn configuration_cell() -> String {
    match std::env::var("GUNBC_MEMORY_COMPOSITION_CONFIGURATION") {
        Ok(s) if !s.is_empty() && !s.contains('\t') && !s.contains('\n') => s,
        Ok(_) => refuse(
            "GUNBC_MEMORY_COMPOSITION_CONFIGURATION must be one non-empty TSV cell (no tab or newline)",
        ),
        Err(_) => refuse(
            "GUNBC_MEMORY_COMPOSITION_CONFIGURATION unset; every receipt line must name the configuration",
        ),
    }
}

/// One resident-set reading with no side effect, for the seams inside the emission.
pub(super) fn readback(stage: &str) {
    if !enabled() {
        return;
    }
    let (Some(rss), Some(peak)) = (rss_kb(), super::peak_rss_vhwm_bytes().map(|b| b / 1024)) else {
        refuse(&format!("resident set unreadable at stage={stage}"));
    };
    eprintln!("[memory-composition] stage={stage} rss_kb={rss} peak_kb={peak}");
    record_line(&format!("stage\t{stage}\t{}\t{}", rss * 1024, peak * 1024));
}

/// The RECEIPT: one `bucket<TAB>name<TAB>bytes<TAB>configuration` line per released structure
/// appended to the file named by `GUNBC_MEMORY_COMPOSITION_RECEIPT`, which
/// `gunbc.self_host_step_memory_demand` reads. Bytes are what `malloc_trim` returned after the
/// release, i.e. the structure's live size. The last column is the configuration cell.
fn record_line(line: &str) {
    use std::io::Write;
    let Some(path) = std::env::var_os("GUNBC_MEMORY_COMPOSITION_RECEIPT") else {
        return;
    };
    let cfg = configuration_cell();
    let mut f = match std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
    {
        Ok(f) => f,
        Err(e) => refuse(&format!("cannot open receipt {path:?}: {e}")),
    };
    if let Err(e) = writeln!(f, "{line}\t{cfg}") {
        refuse(&format!("cannot write receipt {path:?}: {e}"));
    }
}

fn record_bucket(name: &str, trimmed_kb: Option<u64>) {
    let Some(kb) = trimmed_kb else {
        refuse(&format!(
            "malloc_trim reading unavailable for bucket={name}"
        ));
    };
    record_line(&format!("bucket\t{name}\t{}", kb * 1024));
}

fn stage(name: &str, release: impl FnOnce()) {
    let before = rss_kb();
    release();
    let after_release = rss_kb();
    let trimmed = super::trim_retained_heap();
    let after_trim = rss_kb();
    record_bucket(name, trimmed);
    eprintln!(
        "[memory-composition] release={name} rss_kb_before={before:?} rss_kb_after_release={after_release:?} \
         trim_reclaimed_kb={trimmed:?} rss_kb_after_trim={after_trim:?}"
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
        super::pool_acquire::release_whole_tree_after_census,
    );
}
