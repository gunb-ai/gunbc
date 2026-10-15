//! Process-global `OnceLock` fill.
//!
//! The floor attribution ledger that used to read fill-stack timings was reachable
//! only from `run_required_floor` and is deleted. This wrapper is just
//! `OnceLock::get_or_init`.

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

use std::sync::OnceLock;

pub(crate) fn once<T>(cell: &'static OnceLock<T>, build: impl FnOnce() -> T) -> &'static T {
    cell.get_or_init(build)
}
