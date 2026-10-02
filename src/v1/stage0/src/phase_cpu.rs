//! PROBE-ONLY: per-function thread-CPU attribution for the type_env PR-2 A/B (calm-pike-525).
//! A guard is inserted at the head of each instrumented emitted function. Time is charged to a
//! label only at its OUTERMOST activation on this thread, so recursion (ancestry_winner ->
//! surface_value -> ancestry_winner) is not double counted. Labels nest across one another
//! (build_type_env contains surface_fork_rows), so the table is INCLUSIVE per label.
//! `count` labels record calls only: they are the hot point lookups, where a clock read per call
//! would inflate their callers.
use std::cell::RefCell;
use std::collections::BTreeMap;

#[derive(Default, Clone, Copy)]
struct Slot {
    depth: u32,
    start: u128,
    total: u128,
    calls: u64,
}

thread_local! {
    static SLOTS: RefCell<BTreeMap<&'static str, Slot>> = RefCell::new(BTreeMap::new());
}

pub struct Guard(&'static str);

pub fn guard(label: &'static str) -> Guard {
    let now = crate::v1_interpreter::thread_cpu_nanos();
    SLOTS.with(|s| {
        let mut s = s.borrow_mut();
        let slot = s.entry(label).or_default();
        slot.calls += 1;
        if slot.depth == 0 {
            slot.start = now;
        }
        slot.depth += 1;
    });
    Guard(label)
}

impl Drop for Guard {
    fn drop(&mut self) {
        let now = crate::v1_interpreter::thread_cpu_nanos();
        SLOTS.with(|s| {
            let mut s = s.borrow_mut();
            let slot = s.entry(self.0).or_default();
            slot.depth -= 1;
            if slot.depth == 0 {
                slot.total += now.saturating_sub(slot.start);
            }
        });
    }
}

pub fn count(label: &'static str) {
    SLOTS.with(|s| s.borrow_mut().entry(label).or_default().calls += 1);
}

pub fn report(tag: &str) {
    SLOTS.with(|s| {
        for (label, slot) in s.borrow().iter() {
            eprintln!(
                "[phase-cpu] tag={tag} label={label} calls={} cpu_ms={}",
                slot.calls,
                slot.total / 1_000_000
            );
        }
    });
}

pub fn reset() {
    SLOTS.with(|s| s.borrow_mut().clear());
}
