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
    let now = thread_cpu_nanos();
    SLOTS.with(|s| {
        let mut s = s.borrow_mut();
        let slot = s.entry(label).or_default();
        slot.calls += 1;
        if slot.depth == 0 {
            slot.start = now;
            if TIMELINE.contains(&label) {
                timeline(label, "enter", slot.calls);
            }
        }
        slot.depth += 1;
    });
    Guard(label)
}

impl Drop for Guard {
    fn drop(&mut self) {
        let now = thread_cpu_nanos();
        let calls = SLOTS.with(|s| {
            let mut s = s.borrow_mut();
            let slot = s.entry(self.0).or_default();
            slot.depth -= 1;
            if slot.depth == 0 {
                slot.total += now.saturating_sub(slot.start);
                if TIMELINE.contains(&self.0)
                    || (self.0 == "realize_module" && slot.calls % 250 == 0)
                {
                    timeline(self.0, "exit", slot.calls);
                }
            }
            slot.calls
        });
        // Progress snapshots at equal module counts, so two arms compare at the same point even
        // when one cannot finish inside the runner's cap.
        if self.0 == "typecheck_module" && calls % 500 == 0 {
            report(&format!("progress-{calls}"));
        }
        // A timing repetition may stop at a fixed module count, so several arms fit one runner.
        if self.0 == "typecheck_module" {
            if let Some(stop) = std::env::var("GUNBC_PHASE_STOP_AT")
                .ok()
                .and_then(|v| v.parse::<u64>().ok())
            {
                if calls == stop {
                    report(&format!("stop-{calls}"));
                    eprintln!("[phase-cpu-stop] modules={calls} maxrss_kb={}", maxrss_kb());
                    std::process::exit(0);
                }
            }
        }
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

#[repr(C)]
struct Timespec {
    tv_sec: i64,
    tv_nsec: i64,
}
extern "C" {
    fn clock_gettime(clk: i32, ts: *mut Timespec) -> i32;
}
/// CLOCK_THREAD_CPUTIME_ID on linux.
pub fn thread_cpu_nanos() -> u128 {
    let mut ts = Timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    // SAFETY: ts is owned and valid; the call only writes it.
    if unsafe { clock_gettime(3, &mut ts) } == 0 {
        (ts.tv_sec as u128) * 1_000_000_000 + ts.tv_nsec as u128
    } else {
        0
    }
}

#[repr(C)]
struct Rusage {
    fields: [i64; 18],
}
extern "C" {
    fn getrusage(who: i32, usage: *mut Rusage) -> i32;
}
/// Peak resident set of this process, in KiB (ru_maxrss, linux x86_64 struct rusage).
pub fn maxrss_kb() -> i64 {
    let mut ru = Rusage { fields: [0; 18] };
    // SAFETY: ru is owned and is the 144-byte struct rusage; the call only writes it.
    if unsafe { getrusage(0, &mut ru) } == 0 {
        ru.fields[4]
    } else {
        -1
    }
}

/// The reconcile operations whose outermost entry and exit are put on the timeline, so the wall
/// and RSS regions of `compile.reconcile` get owners.
const TIMELINE: &[&str] = &[
    "reconcile_with_census_extra",
    "typecheck_with_census_extra",
    "build_symbol_index_census",
    "build_symbol_index_qualified_fill",
    "build_global_bare_variant_locals",
    "expand_transitive_services",
    "rewire_type_env_parent_links",
    "rewire_type_env_import_str_binding_identity",
    "rewire_func_env_parent_links",
    "leaf_owner_modules_from_registry",
];

fn wall_origin() -> &'static std::time::Instant {
    static ORIGIN: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
    ORIGIN.get_or_init(std::time::Instant::now)
}

/// Current resident set from /proc/self/statm (pages * 4096), in bytes.
pub fn rss_bytes() -> u64 {
    std::fs::read_to_string("/proc/self/statm")
        .ok()
        .and_then(|s| {
            s.split_whitespace()
                .nth(1)
                .and_then(|p| p.parse::<u64>().ok())
        })
        .map(|pages| pages * 4096)
        .unwrap_or(0)
}

fn timeline(label: &str, event: &str, calls: u64) {
    eprintln!(
        "[op-timeline] label={label} event={event} calls={calls} wall_s={:.1} rss_gb={:.2} cpu_s={:.1}",
        wall_origin().elapsed().as_secs_f64(),
        rss_bytes() as f64 / 1e9,
        thread_cpu_nanos() as f64 / 1e9
    );
}

/// A named point on the timeline (probe start, subject assembled, compile done).
pub fn mark(label: &str) {
    timeline(label, "mark", 0);
}

/// Adds `n` to a counted label.
pub fn add(label: &'static str, n: u64) {
    SLOTS.with(|s| s.borrow_mut().entry(label).or_default().calls += n);
}
