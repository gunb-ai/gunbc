//! §7 seed-retained HAND-RUST — global_bare cost-shape receipt instrumentation (sleek-wolf-190).
//! 🟡 dissolve-on: cost-shape receipt landed + namespace cost fix de-forks defork-preserve-quiet-gull-833.

use std::cell::Cell;
use std::rc::Rc;

use im_rc::HashMap;

use crate::v1_compiler_infer_env::GlobalBareLookupState;
use crate::v1_std_core::{has_child_named, Connective, NewlineIndex};

pub(crate) const MERGE_GLOBAL_BARE_VARIANT_KEY_SCAN_SCAFFOLD_MARKER: &str =
    "merge_global_bare_variant_key_scan_counter";

thread_local! {
    static MERGE_GLOBAL_BARE_VARIANT_KEY_SCANS: Cell<usize> = const { Cell::new(0) };
    static MERGE_GLOBAL_BARE_PER_MODULE_SCANS: std::cell::RefCell<Vec<(String, usize, usize)>> =
        std::cell::RefCell::new(Vec::new());
}

pub fn global_bare_receipt_uses_baseline_merge() -> bool {
    std::env::var("GUNBC_GLOBAL_BARE_RECEIPT_BASELINE_MERGE").is_ok()
}

fn global_bare_receipt_instrumentation_active() -> bool {
    std::env::var("GUNBC_GLOBAL_BARE_Q2_BISECT").is_ok()
        || std::env::var("GUNBC_GLOBAL_BARE_RECEIPT_BASELINE_MERGE").is_ok()
}

pub fn global_bare_diagnostic_reconcile_active() -> bool {
    global_bare_receipt_instrumentation_active()
}

fn record_merge_global_bare_variant_key_scans(key_count: usize) {
    if !global_bare_receipt_instrumentation_active() {
        return;
    }
    MERGE_GLOBAL_BARE_VARIANT_KEY_SCANS.with(|c| {
        c.set(c.get() + key_count);
    });
}

pub fn record_merge_global_bare_per_module_scan(
    module_name: String,
    keys_visited: usize,
    has_child_named_calls: usize,
) {
    if !global_bare_receipt_instrumentation_active() {
        return;
    }
    MERGE_GLOBAL_BARE_PER_MODULE_SCANS.with(|rows| {
        rows.borrow_mut()
            .push((module_name, keys_visited, has_child_named_calls));
    });
}

pub fn record_precomputed_merge_module_scan(
    module_name: String,
    precomputed_key_count: usize,
) {
    record_merge_global_bare_variant_key_scans(precomputed_key_count);
    record_merge_global_bare_per_module_scan(module_name, precomputed_key_count, 0);
}

pub fn record_baseline_module_scan(
    module_name: String,
    global_bare: Rc<HashMap<String, Rc<GlobalBareLookupState>>>,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
) {
    if !global_bare_receipt_instrumentation_active() {
        return;
    }
    let key_count = global_bare.len();
    let mut has_child_named_calls = 0usize;
    for (name, lookup) in global_bare.iter() {
        if let GlobalBareLookupState::GlobalBareUniqueBinding { binding } = &**lookup {
            if binding.resolved.connective == Connective::Disj {
                has_child_named_calls += 1;
                let _ = has_child_named(
                    binding.resolved.clone(),
                    name.clone(),
                    source_indices.clone(),
                );
            }
        }
    }
    record_merge_global_bare_variant_key_scans(key_count);
    record_merge_global_bare_per_module_scan(module_name, key_count, has_child_named_calls);
}

pub fn take_merge_global_bare_per_module_scans() -> Vec<(String, usize, usize)> {
    MERGE_GLOBAL_BARE_PER_MODULE_SCANS.with(|rows| {
        let out = rows.borrow().clone();
        rows.borrow_mut().clear();
        out
    })
}

pub fn take_merge_global_bare_variant_key_scans() -> usize {
    MERGE_GLOBAL_BARE_VARIANT_KEY_SCANS.with(|c| {
        let n = c.get();
        c.set(0);
        n
    })
}

fn eprint_merge_global_bare_per_module_receipt(module_count: usize) {
    if !global_bare_receipt_instrumentation_active() {
        return;
    }
    let rows = take_merge_global_bare_per_module_scans();
    if rows.is_empty() {
        return;
    }
    let mut keys_per_module: Vec<usize> = rows.iter().map(|(_, k, _)| *k).collect();
    keys_per_module.sort_unstable();
    let min_k = keys_per_module.first().copied().unwrap_or(0);
    let max_k = keys_per_module.last().copied().unwrap_or(0);
    let total_has_child: usize = rows.iter().map(|(_, _, h)| h).sum();
    let mode = if std::env::var("GUNBC_GLOBAL_BARE_RECEIPT_BASELINE_MERGE").is_ok() {
        "baseline_legacy"
    } else {
        "precomputed_merge"
    };
    eprintln!(
        "[global-bare-receipt] mode={mode} M={module_count} modules_scanned={} \
         keys_per_module min={min_k} max={max_k} constant_k={} total_has_child_named={total_has_child}",
        rows.len(),
        min_k == max_k,
    );
    for (name, keys, has_child) in rows.iter().take(20) {
        eprintln!(
            "[global-bare-receipt] module={name} keys_visited={keys} has_child_named_calls={has_child}"
        );
    }
    if rows.len() > 20 {
        eprintln!(
            "[global-bare-receipt] ... {} more module row(s) omitted",
            rows.len() - 20
        );
    }
}

pub fn finish_global_bare_diagnostic_reconcile_refusal(
    module_count: usize,
    precomputed_variant_locals_key_count: Option<usize>,
) -> Result<(), String> {
    if !global_bare_diagnostic_reconcile_active() {
        return Ok(());
    }
    eprint_merge_global_bare_per_module_receipt(module_count);
    if std::env::var("GUNBC_GLOBAL_BARE_Q2_BISECT").is_ok() {
        let scans = take_merge_global_bare_variant_key_scans();
        eprintln!(
            "[global-bare-q2-bisect] mode={} merge_global_bare_variant_key_scans={scans} \
             variant_locals_keys={}",
            std::env::var("GUNBC_GLOBAL_BARE_Q2_BISECT").unwrap_or_else(|_| "all".to_string()),
            precomputed_variant_locals_key_count.unwrap_or(0),
        );
        return Err(
            "GUNBC_GLOBAL_BARE_Q2_BISECT: diagnostic bisect subset-filtered global_bare per module; \
             refusing green resolve after receipt (DESIGN §5 — diagnostic modes report, never green starved inputs)"
                .to_string(),
        );
    }
    if std::env::var("GUNBC_GLOBAL_BARE_RECEIPT_BASELINE_MERGE").is_ok() {
        let scans = take_merge_global_bare_variant_key_scans();
        eprintln!(
            "[global-bare-receipt] baseline_legacy merge_global_bare_variant_key_scans={scans}"
        );
        return Err(
            "GUNBC_GLOBAL_BARE_RECEIPT_BASELINE_MERGE: replayed legacy per-module global_bare fold \
             for cost-shape receipt; refusing green resolve after receipt (DESIGN §5 — diagnostic \
             modes report, never green starved inputs)"
                .to_string(),
        );
    }
    Ok(())
}
