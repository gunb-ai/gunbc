//! Read-only silent-pick telemetry for resolution divergence census slice 2.
//! Instrumentation hooks live in `global_bare_lookup` and `lookup_resolved_sig`.

use std::cell::{Cell, RefCell};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GlobalBareLcpPickSite {
    pub env_module_path: String,
    pub name: String,
    pub candidate_count: usize,
    pub chosen_module_path: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GlobalBareLcpTieSite {
    pub env_module_path: String,
    pub name: String,
    pub candidate_count: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FnParentFirstHitSite {
    pub env_module_path: String,
    pub name: String,
    pub parent_match_count: usize,
    pub chosen_parent_module: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SilentPickTelemetry {
    pub global_bare_lcp_picks: Vec<GlobalBareLcpPickSite>,
    pub global_bare_lcp_ties: Vec<GlobalBareLcpTieSite>,
    pub fn_parent_first_hits: Vec<FnParentFirstHitSite>,
}

thread_local! {
    static TELEMETRY_ENABLED: Cell<bool> = const { Cell::new(false) };
    static TELEMETRY: RefCell<SilentPickTelemetry> = RefCell::new(SilentPickTelemetry::default());
}

pub fn enable() {
    TELEMETRY.with(|t| *t.borrow_mut() = SilentPickTelemetry::default());
    TELEMETRY_ENABLED.with(|e| e.set(true));
}

pub fn disable() -> SilentPickTelemetry {
    TELEMETRY_ENABLED.with(|e| e.set(false));
    TELEMETRY.with(|t| std::mem::take(&mut *t.borrow_mut()))
}

pub fn is_enabled() -> bool {
    TELEMETRY_ENABLED.with(|e| e.get())
}

pub fn record_global_bare_lcp_pick(
    env_module_path: String,
    name: String,
    candidate_count: usize,
    chosen_module_path: String,
) {
    if !is_enabled() || candidate_count < 2 {
        return;
    }
    TELEMETRY.with(|t| {
        t.borrow_mut()
            .global_bare_lcp_picks
            .push(GlobalBareLcpPickSite {
                env_module_path,
                name,
                candidate_count,
                chosen_module_path,
            });
    });
}

pub fn record_global_bare_lcp_tie(env_module_path: String, name: String, candidate_count: usize) {
    if !is_enabled() || candidate_count < 2 {
        return;
    }
    TELEMETRY.with(|t| {
        t.borrow_mut()
            .global_bare_lcp_ties
            .push(GlobalBareLcpTieSite {
                env_module_path,
                name,
                candidate_count,
            });
    });
}

pub fn record_fn_parent_first_hit(
    env_module_path: String,
    name: String,
    parent_match_count: usize,
    chosen_parent_module: String,
) {
    if !is_enabled() || parent_match_count < 2 {
        return;
    }
    TELEMETRY.with(|t| {
        t.borrow_mut()
            .fn_parent_first_hits
            .push(FnParentFirstHitSite {
                env_module_path,
                name,
                parent_match_count,
                chosen_parent_module,
            });
    });
}
