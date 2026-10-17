// gap-instrument: EXCLUSIVE accounting for the v1 infer second generic-argument round.
// OUT-OF-TREE measurement code. Injected identically into every A/B variant inside the
// remote dispatch by hooks.py; NEVER committed on any variant.
//
// UNIT OF ACCOUNT (identical for control and treatment): ONE re-derived generic call
// argument.
//   control (main's algorithm): the second round is a whole second
//       infer_call_arguments_generic_pass, so EVERY argument of that call is re-derived.
//       The event fires in the fold body, armed only while the enclosing pass
//       invocation was launched from the gated second-round site.
//   treatment (sparse fold): the event fires in the recheck branch, i.e. only for an
//       argument whose `still_missing_after` name became bound.
//
// WHY A CONSUMED PENDING FLAG, NOT A DEPTH TEST. A nested generic call inside an
// argument runs its OWN first round while the outer call's second round is in flight.
// Counting by "is any second round in flight" would charge those first-round folds to
// the second round and inflate the control. So the gated call site sets PENDING_R2 once
// and the very next pass entry CONSUMES it into its own frame; pass entries opened
// inside it (nested calls) see PENDING_R2 cleared and are labelled first-round. Only the
// fold frames whose own pass invocation was launched as a second round are counted.
//
// top vs nest: an event entered with no other event active is "top"; one entered while
// an enclosing event is still open is the cascade, "nest" -- i.e. work done twice.
// *_excl_ms is EXCLUSIVE: an event's own time minus the events it directly contains (a
// child's gross is folded into its parent's child accumulator when the child pops).
//
// All counters are env-gated: with GUNBC_GENERIC_ARG_PASS_ACCOUNT unset every guard
// returns before touching a counter, so the instrumented binary is behaviourally the
// binary that ships. Counts are thread_local Cell increments and are unaffected by the
// timing calls; the ms columns include the Instant cost.
#[allow(dead_code)]
pub mod gap_instrument {
    use std::cell::{Cell, RefCell};
    use std::time::Instant;

    thread_local! {
        // pass nesting depth, and the round label of the pass frame currently open
        static PASS_DEPTH: Cell<usize> = const { Cell::new(0usize) };
        static PASS_IS_R2: Cell<bool> = const { Cell::new(false) };
        static PENDING_R2: Cell<bool> = const { Cell::new(false) };
        static PREV_PASS_IS_R2: Cell<bool> = const { Cell::new(false) };
        static TOP_R1: Cell<u64> = const { Cell::new(0u64) };
        static NEST_R1: Cell<u64> = const { Cell::new(0u64) };
        static TOP_R2: Cell<u64> = const { Cell::new(0u64) };
        static NEST_R2: Cell<u64> = const { Cell::new(0u64) };
        static R2_TOP_EXCL_MS: Cell<u128> = const { Cell::new(0u128) };
        static R2_NEST_EXCL_MS: Cell<u128> = const { Cell::new(0u128) };
        static EV_STACK: RefCell<Vec<(Instant, u128)>> = const { RefCell::new(Vec::new()) };
    }

    fn on() -> bool {
        std::env::var("GUNBC_GENERIC_ARG_PASS_ACCOUNT").is_ok()
    }

    // ---- one entry into the argument fold pass ----------------------------------
    pub struct PassGuard {
        outer_is_r2: bool,
    }
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
            PASS_IS_R2.with(|c| c.set(self.outer_is_r2));
        }
    }
    pub fn pass_guard() -> PassGuard {
        if !on() {
            return PassGuard { outer_is_r2: false };
        }
        // this frame's round label: the pending flag set by a gated second-round call
        // site, consumed here so nested calls opened inside us are first-round.
        let is_r2 = PENDING_R2.with(|c| {
            let v = c.get();
            c.set(false);
            v
        });
        let outer_is_r2 = PASS_IS_R2.with(|c| {
            let prev = c.get();
            c.set(is_r2);
            prev
        });
        PREV_PASS_IS_R2.with(|c| c.set(outer_is_r2));
        PASS_DEPTH.with(|d| {
            let n = d.get() + 1;
            d.set(n);
            if n == 1 {
                TOP_R1.with(|c| c.set(c.get() + 1));
            } else {
                NEST_R1.with(|c| c.set(c.get() + 1));
            }
        });
        PassGuard { outer_is_r2 }
    }

    // ---- the gated second-round call site (control only) -------------------------
    pub struct R2Region;
    impl Drop for R2Region {
        fn drop(&mut self) {
            if on() {
                PENDING_R2.with(|c| c.set(false));
            }
        }
    }
    pub fn r2_region() -> R2Region {
        if on() {
            PENDING_R2.with(|c| c.set(true));
        }
        R2Region
    }

    // ---- one re-derived argument --------------------------------------------------
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

    fn unarmed() -> EvGuard {
        EvGuard {
            armed: false,
            top_level: false,
        }
    }

    fn arm() -> EvGuard {
        let top_level = EV_STACK.with(|s| s.borrow().is_empty());
        if top_level {
            TOP_R2.with(|c| c.set(c.get() + 1));
        } else {
            NEST_R2.with(|c| c.set(c.get() + 1));
        }
        EV_STACK.with(|s| s.borrow_mut().push((Instant::now(), 0u128)));
        EvGuard {
            armed: true,
            top_level,
        }
    }

    /// Treatment hook: sits inside the recheck branch, so every call is a re-derivation.
    pub fn r2_event() -> EvGuard {
        if !on() {
            return unarmed();
        }
        arm()
    }

    /// Control hook: sits in the fold body, which runs in BOTH rounds. Counts only folds
    /// whose own pass invocation was launched as a second round.
    pub fn r2_event_if_region() -> EvGuard {
        if !on() || !PASS_IS_R2.with(|c| c.get()) {
            return unarmed();
        }
        arm()
    }

    pub fn dump() {
        if !on() {
            return;
        }
        eprintln!(
            "[generic-arg-pass-excl] r1_pass_n_top={} r1_pass_n_nest={} recheck_n_top={} recheck_n_nest={} recheck_top_excl_ms={} recheck_nest_excl_ms={}",
            TOP_R1.with(|c| c.get()),
            NEST_R1.with(|c| c.get()),
            TOP_R2.with(|c| c.get()),
            NEST_R2.with(|c| c.get()),
            R2_TOP_EXCL_MS.with(|c| c.get()),
            R2_NEST_EXCL_MS.with(|c| c.get()),
        );
    }
}
