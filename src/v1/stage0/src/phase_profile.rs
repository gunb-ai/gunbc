//! Phase-local heartbeat for long `claim_executor` floor walks — **not** a per-thread trace.
//!
//! Answers "what is this stuck/long-running executor doing right now?" via a **single-slot,
//! process-global, last-writer-wins sampler** (`GLOBAL_INNER` behind one `Mutex`). Under
//! `spawn_width>1`, concurrent batch threads all update the same slot; each emitted tick
//! reports the **most-recently-entered** phase/context across threads — exactly right for
//! heartbeat use ("is it stuck in resolve or eval?") and explicitly **not** per-thread or
//! distributed attribution.
//!
//! **Transport only** (DESIGN §4): stderr `[phase-profile]` k=v lines are the Lossless wire
//! projection crossing the Rust→log boundary — same pattern as `[gantt]`, `[measurement]`,
//! and `[calibration]` in `claim_executor`. The authority is not this file.
//!
//! Zero cost when `GUNBC_FLOOR_PHASE_PROFILE` is unset.
//!
//! **Dissolution trigger (DESIGN §6):** delete `src/v1/stage0/src/phase_profile.rs`, remove the
//! `set_phase` hooks in `cli_run.rs` / `claim_executor.rs`, and drop `GUNBC_FLOOR_PHASE_PROFILE`
//! when realization_measurement_loop **Phase 0** (`docs/plans/realization-measurement-loop.md`) lands
//! a `.dag` `PerformanceReceipt` phase-local tick carrier in `dsl/product/compute_fabric.dag` that a
//! floor witness consumes by execution (the same retirement event that supersedes `GUNBC_FLOOR_GANTT`
//! per `docs/plans/ci-floor-fractal-gantt.md` § dissolution). Receipt = that witness green with this
//! module deleted and zero `[phase-profile]` stderr when profiling is enabled on the model path.

use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FloorPhase {
    Discovery,
    Resolve,
    Typecheck,
    Eval,
    HostEffect,
    Gate,
}

impl FloorPhase {
    pub fn tag(self) -> &'static str {
        match self {
            FloorPhase::Discovery => "discovery",
            FloorPhase::Resolve => "resolve",
            FloorPhase::Typecheck => "typecheck",
            FloorPhase::Eval => "eval",
            FloorPhase::HostEffect => "host-effect",
            FloorPhase::Gate => "gate",
        }
    }
}

struct PhaseProfileInner {
    run_started: Instant,
    phase_started: Instant,
    phase: FloorPhase,
    context: String,
    tick: AtomicU64,
    shutdown: AtomicBool,
}

impl PhaseProfileInner {
    fn emit_record(&self, reason: &str, signal: Option<&str>) {
        let tick = self.tick.fetch_add(1, Ordering::Relaxed) + 1;
        let elapsed_ms = self.run_started.elapsed().as_millis();
        let phase_elapsed_ms = self.phase_started.elapsed().as_millis();
        let signal_suffix = match signal {
            Some(s) => format!(" signal={s} flushed=1"),
            None => String::new(),
        };
        // Live type-env amplification counters (v1_compiler_infer_env atomics): during a
        // long typecheck the heartbeat shows WHICH counter grows without bound — the
        // 2026-07-04 use-site inference pathology instrument. Cheap relaxed loads.
        let flatten = crate::v1_compiler_infer_env::flatten_visible_parent_recurses();
        let builds = crate::v1_compiler_infer_env::build_type_env_calls();
        let merges = crate::v1_compiler_infer_env::merge_type_env_cache_calls();
        let rewires = crate::v1_compiler_infer_env::rewire_type_env_parent_links_calls();
        let _ = writeln!(
            io::stderr(),
            "[phase-profile] tick={tick} phase={} elapsed_ms={elapsed_ms} phase_elapsed_ms={phase_elapsed_ms} context={} reason={reason} env_flatten={flatten} env_builds={builds} env_merges={merges} env_rewires={rewires}{signal_suffix}",
            self.phase.tag(),
            self.context,
        );
        let _ = io::stderr().flush();
    }

    fn set_phase(&mut self, phase: FloorPhase, context: &str) {
        self.phase = phase;
        self.context = context.to_string();
        self.phase_started = Instant::now();
        self.emit_record("phase-enter", None);
    }
}

pub struct PhaseProfile {
    inner: Arc<Mutex<PhaseProfileInner>>,
    heartbeat: Option<JoinHandle<()>>,
}

static GLOBAL_INNER: OnceLock<Arc<Mutex<PhaseProfileInner>>> = OnceLock::new();

static SIGTERM_HOOKED: AtomicBool = AtomicBool::new(false);

/// Async-signal-safe: the handler only sets this flag; a worker thread flushes then exits.
static SIGTERM_RECEIVED: AtomicBool = AtomicBool::new(false);

/// Ensures exactly one flush+exit path runs after SIGTERM.
static SIGTERM_EXITING: AtomicBool = AtomicBool::new(false);

/// Conventional exit code for SIGTERM (128 + 15).
const SIGTERM_EXIT_CODE: i32 = 143;

fn flush_sigterm_and_exit(inner: &Arc<Mutex<PhaseProfileInner>>) -> ! {
    if SIGTERM_EXITING.swap(true, Ordering::SeqCst) {
        std::process::exit(SIGTERM_EXIT_CODE);
    }
    if let Ok(guard) = inner.lock() {
        guard.emit_record("sigterm", Some("SIGTERM"));
        guard.shutdown.store(true, Ordering::Relaxed);
    }
    std::process::exit(SIGTERM_EXIT_CODE);
}

#[cfg(unix)]
mod unix_sig {
    use std::os::raw::c_int;

    pub const SIGTERM: c_int = 15;
    type SighandlerT = Option<extern "C" fn(c_int)>;

    extern "C" {
        fn signal(signum: c_int, handler: SighandlerT) -> SighandlerT;
    }

    pub fn install(handler: extern "C" fn(c_int)) {
        unsafe {
            signal(SIGTERM, Some(handler));
        }
    }
}

#[cfg(unix)]
extern "C" fn sigterm_handler(_signum: std::os::raw::c_int) {
    SIGTERM_RECEIVED.store(true, Ordering::SeqCst);
}

pub fn phase_profile_enabled() -> bool {
    std::env::var("GUNBC_FLOOR_PHASE_PROFILE")
        .map(|v| v == "1")
        .unwrap_or(false)
}

fn heartbeat_interval() -> Duration {
    let secs = std::env::var("GUNBC_FLOOR_PHASE_PROFILE_INTERVAL_SECS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(30)
        .max(1);
    Duration::from_secs(secs)
}

fn install_sigterm_hook() {
    if SIGTERM_HOOKED.swap(true, Ordering::SeqCst) {
        return;
    }
    #[cfg(unix)]
    unix_sig::install(sigterm_handler);
}

impl PhaseProfile {
    pub fn install_from_env() -> Option<Self> {
        if !phase_profile_enabled() {
            return None;
        }
        Some(Self::new())
    }

    fn new() -> Self {
        Self::new_inner(true)
    }

    fn new_inner(register_global: bool) -> Self {
        // SIGTERM hook only on the production path (`install_from_env` → `new`); unit tests
        // use `new_inner(false)` and must not rewire the process-wide handler in `cargo test`.
        if register_global {
            install_sigterm_hook();
        }
        let now = Instant::now();
        let inner = Arc::new(Mutex::new(PhaseProfileInner {
            run_started: now,
            phase_started: now,
            phase: FloorPhase::Gate,
            context: "startup".to_string(),
            tick: AtomicU64::new(0),
            shutdown: AtomicBool::new(false),
        }));
        if register_global {
            let _ = GLOBAL_INNER.set(Arc::clone(&inner));
        }
        let shared = Arc::clone(&inner);
        let interval = heartbeat_interval();
        let heartbeat = thread::spawn(move || {
            let mut elapsed = Duration::ZERO;
            loop {
                thread::sleep(Duration::from_secs(1));
                if SIGTERM_RECEIVED.load(Ordering::SeqCst) {
                    flush_sigterm_and_exit(&shared);
                }
                let mut guard = shared.lock().expect("phase_profile lock");
                if guard.shutdown.load(Ordering::Relaxed) {
                    break;
                }
                elapsed += Duration::from_secs(1);
                if elapsed >= interval {
                    elapsed = Duration::ZERO;
                    guard.emit_record("heartbeat", None);
                }
            }
        });
        Self {
            inner,
            heartbeat: Some(heartbeat),
        }
    }

    pub fn set_phase(&self, phase: FloorPhase, context: &str) {
        if SIGTERM_RECEIVED.load(Ordering::SeqCst) {
            flush_sigterm_and_exit(&self.inner);
        }
        let mut guard = self.inner.lock().expect("phase_profile lock");
        guard.set_phase(phase, context);
    }

    pub fn tick_count(&self) -> u64 {
        self.inner
            .lock()
            .expect("phase_profile lock")
            .tick
            .load(Ordering::Relaxed)
    }

    pub fn flush_sigterm_for_test(&self) {
        let mut guard = self.inner.lock().expect("phase_profile lock");
        guard.emit_record("sigterm", Some("SIGTERM"));
        guard.shutdown.store(true, Ordering::Relaxed);
    }
}

impl Drop for PhaseProfile {
    fn drop(&mut self) {
        if self.heartbeat.is_some() {
            if SIGTERM_RECEIVED.load(Ordering::SeqCst) {
                flush_sigterm_and_exit(&self.inner);
            }
            {
                let mut guard = self.inner.lock().expect("phase_profile lock");
                guard.shutdown.store(true, Ordering::Relaxed);
                guard.emit_record("shutdown", None);
            }
            if let Some(handle) = self.heartbeat.take() {
                let _ = handle.join();
            }
        }
    }
}

pub fn set_phase(phase: FloorPhase, context: &str) {
    if SIGTERM_RECEIVED.load(Ordering::SeqCst) {
        if let Some(inner) = GLOBAL_INNER.get() {
            flush_sigterm_and_exit(inner);
        }
        std::process::exit(SIGTERM_EXIT_CODE);
    }
    if let Some(inner) = GLOBAL_INNER.get() {
        let mut guard = inner.lock().expect("phase_profile lock");
        guard.set_phase(phase, context);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn floor_phase_tags_are_stable() {
        assert_eq!(FloorPhase::Discovery.tag(), "discovery");
        assert_eq!(FloorPhase::Resolve.tag(), "resolve");
        assert_eq!(FloorPhase::Typecheck.tag(), "typecheck");
        assert_eq!(FloorPhase::Eval.tag(), "eval");
        assert_eq!(FloorPhase::HostEffect.tag(), "host-effect");
        assert_eq!(FloorPhase::Gate.tag(), "gate");
    }

    #[test]
    fn emits_multiple_ticks_and_sigterm_flush() {
        std::env::set_var("GUNBC_FLOOR_PHASE_PROFILE_INTERVAL_SECS", "1");
        let profile = PhaseProfile::new_inner(false);
        profile.set_phase(FloorPhase::Resolve, "test-entry.dag");
        profile.set_phase(FloorPhase::Typecheck, "test-entry.dag");
        profile.set_phase(FloorPhase::Eval, "witness_fn");
        thread::sleep(Duration::from_millis(1100));
        profile.flush_sigterm_for_test();
        assert!(
            profile.tick_count() >= 2,
            "expected >=2 phase-profile ticks on a held fixture, got {}",
            profile.tick_count()
        );
        std::env::remove_var("GUNBC_FLOOR_PHASE_PROFILE_INTERVAL_SECS");
    }
}
