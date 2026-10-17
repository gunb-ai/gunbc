// gap-instrument: EXCLUSIVE accounting for the v1 infer second generic-argument
// round. OUT-OF-TREE measurement code, applied identically as a patch to base and
// candidate inside the same remote dispatch; NEVER committed on either side.
//
// Unit of account: ONE re-inferred generic argument -- the work that doubled per
// nesting depth before the fix.
//   base      : an argument folded while the gated SECOND
//               infer_call_arguments_generic_pass is running (arms via region).
//   candidate : an argument entering the inline recheck branch
//               (argument_needs_generic_recheck true) -- the hook fires only there.
//
// top vs nest is classified by the r2 event stack: an event entered with no r2
// event active is "top"; one entered while another is active is the cascade.
// r2_*_excl_ms is EXCLUSIVE: an event's own time minus the r2 events it directly
// contains (a child's gross is folded into its parent's child accumulator on pop).
#[allow(dead_code)]
pub(crate) mod gap_instrument {
    use std::cell::{Cell, RefCell};
    use std::time::Instant;

    thread_local! {
        static PASS_DEPTH: Cell<usize> = const { Cell::new(0usize) };
        static TOP_R1: Cell<u64> = const { Cell::new(0u64) };
        static NEST_R1: Cell<u64> = const { Cell::new(0u64) };
        static TOP_R2: Cell<u64> = const { Cell::new(0u64) };
        static NEST_R2: Cell<u64> = const { Cell::new(0u64) };
        static R2_TOP_EXCL_MS: Cell<u128> = const { Cell::new(0u128) };
        static R2_NEST_EXCL_MS: Cell<u128> = const { Cell::new(0u128) };
        static R2_REGION_DEPTH: Cell<usize> = const { Cell::new(0usize) };
        // (entry instant, gross ms already absorbed from finished nested events)
        static EV_STACK: RefCell<Vec<(Instant, u128)>> = const { RefCell::new(Vec::new()) };
    }

    fn on() -> bool {
        std::env::var("GUNBC_GENERIC_ARG_PASS_ACCOUNT").is_ok()
    }

    // ---- r1: one entry into the argument fold pass -----------------------------
    pub struct PassGuard;
    impl Drop for PassGuard {
        fn drop(&mut self) {
            if !on() {
                return;
            }
            PASS_DEPTH.with(|d| {
                let n = d.get();
                if n > 0 {
                    d.set(n - 1);
                }
            });
        }
    }
    pub fn pass_guard() -> PassGuard {
        if on() {
            PASS_DEPTH.with(|d| {
                let n = d.get() + 1;
                d.set(n);
                if n == 1 {
                    TOP_R1.with(|c| c.set(c.get() + 1));
                } else {
                    NEST_R1.with(|c| c.set(c.get() + 1));
                }
            });
        }
        PassGuard
    }

    // ---- base only: the gated second round is executing ------------------------
    pub struct R2Region;
    impl Drop for R2Region {
        fn drop(&mut self) {
            if !on() {
                return;
            }
            R2_REGION_DEPTH.with(|d| {
                let n = d.get();
                if n > 0 {
                    d.set(n - 1);
                }
            });
        }
    }
    pub fn r2_region() -> R2Region {
        if on() {
            R2_REGION_DEPTH.with(|d| d.set(d.get() + 1));
        }
        R2Region
    }

    // ---- one re-inferred generic argument --------------------------------------
    pub struct EvGuard {
        armed: bool,
        top_level: bool,
    }
    impl Drop for EvGuard {
        fn drop(&mut self) {
            if !self.armed {
                return;
            }
            let top = self.top_level;
            EV_STACK.with(|s| {
                let mut v = s.borrow_mut();
                if let Some((start, child_ms)) = v.pop() {
                    let gross = start.elapsed().as_millis();
                    let excl = gross.saturating_sub(child_ms);
                    if let Some(parent) = v.last_mut() {
                        parent.1 = parent.1 + gross;
                    }
                    if top {
                        R2_TOP_EXCL_MS.with(|c| c.set(c.get() + excl));
                    } else {
                        R2_NEST_EXCL_MS.with(|c| c.set(c.get() + excl));
                    }
                }
            });
        }
    }

    fn arm() -> EvGuard {
        let top_level = {
            let stack = EV_STACK.with(|s| s.borrow().len());
            if stack == 0 {
                TOP_R2.with(|c| c.set(c.get() + 1));
                true
            } else {
                NEST_R2.with(|c| c.set(c.get() + 1));
                false
            }
        };
        EV_STACK.with(|s| s.borrow_mut().push((Instant::now(), 0u128)));
        EvGuard {
            armed: true,
            top_level,
        }
    }

    /// Candidate hook: called from inside the recheck branch, so every call is an
    /// r2 event.
    pub fn r2_event() -> EvGuard {
        if !on() {
            return EvGuard {
                armed: false,
                top_level: false,
            };
        }
        arm()
    }

    /// Base hook: called from the argument fold body, which runs in BOTH rounds;
    /// counts only while the gated second round is active.
    pub fn r2_event_if_region() -> EvGuard {
        if !on() || R2_REGION_DEPTH.with(|d| d.get()) == 0 {
            return EvGuard {
                armed: false,
                top_level: false,
            };
        }
        arm()
    }

    pub fn dump() {
        if !on() {
            return;
        }
        eprintln!(
            "[generic-arg-pass-excl] top_r1_n={} top_r2_n={} nest_r1_n={} nest_r2_n={} r2_top_excl_ms={} r2_nest_excl_ms={}",
            TOP_R1.with(|c| c.get()),
            TOP_R2.with(|c| c.get()),
            NEST_R1.with(|c| c.get()),
            NEST_R2.with(|c| c.get()),
            R2_TOP_EXCL_MS.with(|c| c.get()),
            R2_NEST_EXCL_MS.with(|c| c.get()),
        );
    }
}
