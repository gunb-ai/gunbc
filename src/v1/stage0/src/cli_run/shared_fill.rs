//! Fill-stack for process-global caches (`once` / `begin_fill`).
//!
//! The floor's printed attribution ledger was reachable only from `run_required_floor`
//! and is deleted. Callers still wrap cache initialization so a nested fill cannot
//! leave the stack unbalanced.

#![warn(
    clippy::assertions_on_constants,
    clippy::clone_on_copy,
    clippy::cloned_ref_to_slice_refs,
    clippy::collapsible_str_replace,
    clippy::disallowed_macros,
    clippy::doc_lazy_continuation,
    clippy::empty_line_after_doc_comments,
    clippy::enum_variant_names,
    clippy::iter_kv_map,
    clippy::manual_is_multiple_of,
    clippy::manual_strip,
    clippy::map_identity,
    clippy::missing_const_for_thread_local,
    clippy::needless_borrow,
    clippy::needless_lifetimes,
    clippy::only_used_in_recursion,
    clippy::ptr_arg,
    clippy::redundant_closure,
    clippy::single_char_add_str,
    clippy::type_complexity,
    clippy::unnecessary_to_owned,
    clippy::unneeded_struct_pattern,
    clippy::useless_vec,
    dead_code,
    unused_imports,
    unused_mut
)]

use std::cell::{Cell, RefCell};
use std::sync::OnceLock;
use std::time::Instant;

thread_local! {
    /// One accumulator per fill currently in flight, innermost last.
    static FILL_CHILD_NANOS: RefCell<Vec<u64>> = const { RefCell::new(Vec::new()) };
}

pub(crate) fn begin_fill() {
    FILL_CHILD_NANOS.with(|s| s.borrow_mut().push(0));
}

pub(crate) fn abandon_fill() {
    FILL_CHILD_NANOS.with(|s| {
        s.borrow_mut().pop();
    });
}

fn close_fill(inclusive: u64) {
    FILL_CHILD_NANOS.with(|s| {
        let mut stack = s.borrow_mut();
        let children = stack.pop().unwrap_or(0);
        if let Some(parent) = stack.last_mut() {
            *parent = parent.saturating_add(inclusive);
        }
        let _ = inclusive.saturating_sub(children);
    })
}

pub(crate) fn record_fill(_cache: &'static str, _key: &str, nanos: u64) {
    close_fill(nanos);
}

pub(crate) fn record_hit(_cache: &'static str, _key: &str) {}

pub(crate) fn once<T>(
    cell: &'static OnceLock<T>,
    cache: &'static str,
    key: &str,
    build: impl FnOnce() -> T,
) -> &'static T {
    if let Some(value) = cell.get() {
        record_hit(cache, key);
        return value;
    }
    let built = Cell::new(false);
    begin_fill();
    let start = Instant::now();
    let value = cell.get_or_init(|| {
        built.set(true);
        build()
    });
    if built.get() {
        record_fill(cache, key, start.elapsed().as_nanos() as u64);
    } else {
        abandon_fill();
        record_hit(cache, key);
    }
    value
}
