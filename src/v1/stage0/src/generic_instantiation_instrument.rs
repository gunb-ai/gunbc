// Hand-maintained instrumentation for H2a/H2b confirmation (generic instantiation counters).
// Enable with GUNBC_INSTRUMENT_GENERIC_INSTANTIATION=1; report printed by claim_batch after resolve.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::v1_std_core::Node;

type SiteKey = u64;

thread_local! {
    static SUBST_CALLS: RefCell<u64> = const { RefCell::new(0) };
    static UNIFY_CALLS: RefCell<u64> = const { RefCell::new(0) };
    static SUBST_SITE_KEYS: RefCell<HashMap<SiteKey, u64>> = RefCell::new(HashMap::new());
    static ENABLED_FLAG: RefCell<bool> = const { RefCell::new(false) };
}

static ENV_CHECKED: AtomicBool = AtomicBool::new(false);

fn hash_str(s: &str) -> SiteKey {
    let mut h: SiteKey = 0xcbf29ce484222325;
    for b in s.bytes() {
        h ^= b as SiteKey;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

fn combine_hash(a: SiteKey, b: SiteKey) -> SiteKey {
    a.wrapping_mul(0x100000001b3) ^ b
}

fn ensure_env_checked() {
    if ENV_CHECKED.swap(true, Ordering::Relaxed) {
        return;
    }
    if std::env::var("GUNBC_INSTRUMENT_GENERIC_INSTANTIATION")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
    {
        ENABLED_FLAG.with(|f| *f.borrow_mut() = true);
    }
}

pub fn enabled() -> bool {
    ensure_env_checked();
    ENABLED_FLAG.with(|f| *f.borrow())
}

pub fn reset() {
    SUBST_CALLS.with(|c| *c.borrow_mut() = 0);
    UNIFY_CALLS.with(|c| *c.borrow_mut() = 0);
    SUBST_SITE_KEYS.with(|m| m.borrow_mut().clear());
}

fn node_shape_fingerprint(n: &Rc<Node>) -> SiteKey {
    let label = if !n.name.is_empty() {
        n.name.clone()
    } else {
        format!("{:?}", n.connective)
    };
    let child_count = n.children.len() as SiteKey;
    let param_count = n.params.len() as SiteKey;
    combine_hash(hash_str(&label), combine_hash(child_count, param_count))
}

fn subst_fingerprint(subst_len: usize) -> SiteKey {
    subst_len as SiteKey
}

pub fn record_substitute_generics(formal: &Rc<Node>, subst_len: usize) {
    if !enabled() {
        return;
    }
    SUBST_CALLS.with(|c| *c.borrow_mut() += 1);
    let site_key = combine_hash(node_shape_fingerprint(formal), subst_fingerprint(subst_len));
    SUBST_SITE_KEYS.with(|m| {
        *m.borrow_mut().entry(site_key).or_insert(0) += 1;
    });
}

pub fn record_unify_generics() {
    if !enabled() {
        return;
    }
    UNIFY_CALLS.with(|c| *c.borrow_mut() += 1);
}

pub struct GenericInstantiationReport {
    pub subst_calls: u64,
    pub unify_calls: u64,
    pub distinct_subst_sites: u64,
    pub max_site_redundancy: u64,
    pub total_redundant_subst_calls: u64,
}

pub fn snapshot_report() -> GenericInstantiationReport {
    let subst_calls = SUBST_CALLS.with(|c| *c.borrow());
    let unify_calls = UNIFY_CALLS.with(|c| *c.borrow());
    let (distinct_subst_sites, max_site_redundancy, total_redundant_subst_calls) =
        SUBST_SITE_KEYS.with(|m| {
            let map = m.borrow();
            let distinct = map.len() as u64;
            let max_red = map.values().copied().max().unwrap_or(0);
            let redundant = map.values().map(|&c| c.saturating_sub(1)).sum::<u64>();
            (distinct, max_red, redundant)
        });
    GenericInstantiationReport {
        subst_calls,
        unify_calls,
        distinct_subst_sites,
        max_site_redundancy,
        total_redundant_subst_calls,
    }
}

pub fn eprint_report(context: &str) {
    if !enabled() {
        return;
    }
    let r = snapshot_report();
    eprintln!("[generic-inst] {context}");
    eprintln!(
        "  substitute_generics calls: {}  distinct site keys: {}  max repeats/site: {}  redundant calls (count-1 per site): {}",
        r.subst_calls,
        r.distinct_subst_sites,
        r.max_site_redundancy,
        r.total_redundant_subst_calls
    );
    eprintln!("  unify_generics calls: {}", r.unify_calls);
    if r.distinct_subst_sites > 0 {
        let ratio = r.subst_calls as f64 / r.distinct_subst_sites as f64;
        eprintln!(
            "  calls/distinct-site ratio: {:.1}  (H2a favored if ratio >> 4 and redundant >> distinct)",
            ratio
        );
    }
}
