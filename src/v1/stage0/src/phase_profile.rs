//! Periodic phase-local heartbeat records for long `claim_executor` floor walks.
//!
//! Transport only (DESIGN §4): stderr lines tagged `[phase-profile]` with
//! `phase ∈ {discovery|resolve|typecheck|eval|host-effect|gate}`. Zero cost when
//! `GUNBC_FLOOR_PHASE_PROFILE` is unset. Dissolves when the realization_measurement_loop
//! `.dag` carrier supersedes this Rust tap (same family as `GUNBC_FLOOR_GANTT`).

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
    sigterm_pending: AtomicBool,
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
        let _ = writeln!(
            io::stderr(),
            "[phase-profile] tick={tick} phase={} elapsed_ms={elapsed_ms} phase_elapsed_ms={phase_elapsed_ms} context={} reason={reason}{signal_suffix}",
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
    if let Some(inner) = GLOBAL_INNER.get() {
        inner
            .lock()
            .expect("phase_profile lock")
            .sigterm_pending
            .store(true, Ordering::SeqCst);
    }
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
        install_sigterm_hook();
        let now = Instant::now();
        let inner = Arc::new(Mutex::new(PhaseProfileInner {
            run_started: now,
            phase_started: now,
            phase: FloorPhase::Gate,
            context: "startup".to_string(),
            tick: AtomicU64::new(0),
            sigterm_pending: AtomicBool::new(false),
            shutdown: AtomicBool::new(false),
        }));
        let _ = GLOBAL_INNER.set(Arc::clone(&inner));
        let shared = Arc::clone(&inner);
        let interval = heartbeat_interval();
        let heartbeat = thread::spawn(move || {
            loop {
                thread::sleep(interval);
                let mut guard = shared.lock().expect("phase_profile lock");
                if guard.shutdown.load(Ordering::Relaxed) {
                    break;
                }
                if guard.sigterm_pending.swap(false, Ordering::SeqCst) {
                    guard.emit_record("sigterm", Some("SIGTERM"));
                    guard.shutdown.store(true, Ordering::Relaxed);
                    break;
                }
                guard.emit_record("heartbeat", None);
            }
        });
        Self {
            inner,
            heartbeat: Some(heartbeat),
        }
    }

    pub fn set_phase(&self, phase: FloorPhase, context: &str) {
        let mut guard = self.inner.lock().expect("phase_profile lock");
        if guard.sigterm_pending.swap(false, Ordering::SeqCst) {
            guard.emit_record("sigterm", Some("SIGTERM"));
            guard.shutdown.store(true, Ordering::Relaxed);
            return;
        }
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
        guard.sigterm_pending.store(true, Ordering::SeqCst);
        guard.emit_record("sigterm", Some("SIGTERM"));
        guard.shutdown.store(true, Ordering::Relaxed);
    }
}

impl Drop for PhaseProfile {
    fn drop(&mut self) {
        if self.heartbeat.is_some() {
            {
                let mut guard = self.inner.lock().expect("phase_profile lock");
                guard.shutdown.store(true, Ordering::Relaxed);
                if guard.sigterm_pending.swap(false, Ordering::SeqCst) {
                    guard.emit_record("sigterm", Some("SIGTERM"));
                } else {
                    guard.emit_record("shutdown", None);
                }
            }
            if let Some(handle) = self.heartbeat.take() {
                let _ = handle.join();
            }
        }
    }
}

pub fn set_phase(phase: FloorPhase, context: &str) {
    if let Some(inner) = GLOBAL_INNER.get() {
        let mut guard = inner.lock().expect("phase_profile lock");
        if guard.sigterm_pending.swap(false, Ordering::SeqCst) {
            guard.emit_record("sigterm", Some("SIGTERM"));
            guard.shutdown.store(true, Ordering::Relaxed);
            return;
        }
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
        std::env::set_var("GUNBC_FLOOR_PHASE_PROFILE", "1");
        std::env::set_var("GUNBC_FLOOR_PHASE_PROFILE_INTERVAL_SECS", "1");
        let profile = PhaseProfile::install_from_env().expect("profile enabled");
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
        drop(profile);
        std::env::remove_var("GUNBC_FLOOR_PHASE_PROFILE");
        std::env::remove_var("GUNBC_FLOOR_PHASE_PROFILE_INTERVAL_SECS");
    }
}
