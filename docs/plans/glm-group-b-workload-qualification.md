# GLM group B: workload qualification after the variant-e capture

Status 2026-09-28. Supersedes the "per-step timing experiment" proposal. External review
2026-09-27 (serving mechanisms) supplies the corrections this plan encodes; the variant-e
capture (2026-09-27, watchdog ts=1790547523) is the evidence base.

## What the capture established — and what it did not

Established:

- Six concurrent cold 262K submissions: five completed (~5 min apart), the sixth's 1800s
  client bound expired while the engine was still making progress. Zero preemption log lines
  anywhere; the preemption/recompute-storm hypothesis is DISPROVED for this arm.
- The watchdog stacks show the engine core waiting on workers with a worker on-CPU — no
  deadlock shape.
- The budget arithmetic closes the observation: 6 × 262,144 = 1,572,864 fresh tokens at the
  observed ~874 tok/s aggregate ≈ 30 minutes of engine work; the five completions plus the
  bound expiry fit inside it.

Corrections now law (from the review):

- Six cache seats are SIX REQUEST STATES, not six compute lanes. TP4 splits every operation
  across the four ranks; "the engine serializes seats" is not a mechanism, it is one shared
  compute pool under one shared 2,048-token step budget (running-before-waiting admission in
  the pinned scheduler).
- A max_tokens=1 completion latency is a COMPLETION latency (input processing, queueing,
  prefill, sampling, delivery), not an instrumented GPU-prefill duration.
- Completion order alone cannot distinguish driver-serial from budget-starved from
  shared-throughput. Every timing claim binds to the frozen run plan / launch receipt, never
  to the live spark_active_workload_variant selector (a mid-flight flip or a second launcher
  — both happened 2026-09-27 — must not be able to relabel a run's mode). Runs interrupted by
  another controller are CONTENDED evidence and excluded from latency claims.

The remaining open question: is ~874 tok/s aggregate (≈2.3s per 2,048-token step) an
EFFICIENT execution of that work on this realization? That is an attribution question —
sparse-indexer, attention, collectives, host gaps — and it is answered by operation-level
timing on representative steps, not by more repro campaigns.

## Corrections adopted 2026-09-28 (review of the W1 reading)

1. **W1's observation is narrower than first reported.** One cold 262K completing in 323.16s
   without six-way contention establishes that the slow near-full-window request OCCURS
   without the cohort. It does not establish compute-bound execution and does not make
   ~5 minutes an immutable price. The observation stays bound to its exact
   runtime/profile/workload subject; W1b attribution remains REQUIRED.
2. **The "26k tok/s" engine log lines are reconciled and retired as a progress signal.** The
   pinned metric producer credits prompt tokens at FIRST-OUTPUT processing divided by the
   logging interval — completion accounting. Per-step progress evidence must come from
   scheduled-token/per-step observation (W1b), never from those lines.
3. **gap_mib=2 does NOT discharge any rank fragmentation obligation.** The present producer
   measures head-host maximum-process footprint growth above a post-prefill reference — not
   allocator fragmentation — and observes the head host only, not the other ranks. The
   observation is preserved under that actual meaning; the instrumentation-to-obligation join
   needs repair (an allocator-level, per-rank measurement) before any fragmentation obligation
   can close.
4. **Wet work is strictly sequential under exclusive cohort ownership.** W1b and other
   investigations are prepared independently, but no second run may replace the engine whose
   cache or timing another run is observing (the 2026-09-27 mid-flight teardown is the named
   counterexample).
5. **W2 is an explicit cache-reuse qualification, not a two-seat latency comparison.** Cold
   prime, exact replay, genuine suffix continuation, and a controlled prefix-miss leg, all
   bound to the same admitted engine/cache incarnation, recording ACTUAL cached and computed
   token counts (the response's prompt_tokens_details.cached_tokens, not inferred reuse) plus
   request-level timing. A fast replay alone does not establish free general follow-ups.

## The frozen objective and the unified qualification (2026-09-28 steering)

**Objective: restore GLM at a qualified 262,144-token x 8-seat configuration, then stop.** One
deliverable: an operational eight-seat service with its exact acceptance bundle and persistent
readback. DeepSeek reuse continues independently and never becomes a deployment prerequisite.

The closing qualification is ONE frozen eight-seat candidate and ONE exclusively owned
execution — not separate campaigns:

    freeze candidate (eight requested; executable derived; the old summary-based six-seat
      derivation retained as floor only)
    -> all-rank startup + resolved allocation evidence (the eight-seat candidate's OWN
       allocation plan and per-rank phase bounds; the six-seat run's 1,035-block inventory is
       a conditional expectation, never the verdict)
    -> W2 cache-reuse legs (cold prime / exact replay / suffix continuation / prefix miss)
       BEFORE any unrelated churn can evict the prefix under test
    -> occupancy high-water evidence: live request identities, resident context/block demands,
       concurrent decode/workspace shape, per-rank memory standing — eight submitted
       max_tokens=1 requests do NOT establish eight simultaneously resident full-window
       contexts; the run records the population it actually exercised
    -> real decode + at least one real harness protocol interaction (one-token completions do
       not establish tool-call parsing, multi-turn continuation, or sustained decode)
    -> settlement: cancellation, its observed completion, deterministic stop
    -> promotion through persistent convergence; deployed configuration, generation, and route
       verified after the existing readiness/access gates

The execution budget covers the WHOLE sequence (startup + readback + queued and executing
requests + reuse controls + settlement). A client deadline is never an engine failure, and a
client's disappearance is never proof its engine work or reservation was released —
cancellation and its observed completion ride the shared request lifecycle.

Deployment gate vs follow-on (the release distinction):

    REQUIRED before normal serving: exact runtime/checkpoint/config/rank identity; complete
    applicable memory bounds including the allocator allowance; ENFORCED context/concurrency
    limits (the first operational admission policy is conservative about NEW cold-prefill
    work, and the restriction is enforced, never aspirational); correct generation and
    required protocol behavior; exclusive ownership, settlement, restart, readback; route
    withdrawal and cancellation/recovery; any latency/reuse guarantee actually advertised.

    FOLLOW-ON unless explicitly promised: W1b performance attribution (blocks only if it
    uncovers a correctness/resource defect or an explicitly required operating contract);
    maximizing concurrent cold-prefill throughput; broad benchmark matrices; an unadvertised
    "nearly free warm continuation" objective (a cache miss is not a deployment failure when
    cold execution is safe); DeepSeek's complete deployment.

Safety-term closure order for each open term: derive a defensible bound from the mechanism;
use a justified conservative bound where exact derivation is unnecessary (a conservative bound
covers the candidate's actual allocation behavior — never an arbitrary allowance beside a
green result); measure the specifically unresolved term where needed.

## The three qualification workloads (replace the single cold stress test)

W1. ONE COLD FULL-WINDOW REQUEST — the attributable cold-prefill cost. One 262K request,
    nothing else admitted. Per-request timing: submission, engine arrival, first scheduled
    work, prefill completion / first output, request completion, scheduled tokens per step,
    plus operation-level durations and rank waits for a small number of representative
    steps. The output is the cold-prefill service time with its dominant term named
    (compute / indexer / communication / host / cache-recompute / unresolved).

W2. CACHE-REUSE QUALIFICATION, four legs on one admitted engine/cache incarnation: (1) COLD
    PRIME — a full-window body A, the cold reference; (2) EXACT REPLAY — A again, the pure
    reuse measurement; (3) SUFFIX CONTINUATION — A plus a small new suffix, the genuine
    follow-up shape; (4) PREFIX MISS — a disjoint body B, the control that must NOT reuse.
    Each leg records the response's actual cached/computed token counts
    (prompt_tokens_details.cached_tokens) and driver timing. Verdict shape: replay near-free
    AND suffix near-free AND miss cold-priced means reuse is real and general for this
    subject; a fast replay with a cold suffix means replay-only reuse; a fast MISS means the
    measurement is lying. No leg's speed alone establishes free general follow-ups.

W3. COLD PREFILL ALONGSIDE ONGOING DECODES — does admitting new work damage useful
    interactive progress? N decode sessions established and streaming; one cold 262K prefill
    admitted; the decode-latency standing before/during is the verdict's subject.

## Service-promise shape this feeds

The capacity intent stops being "262144 × N uniform seats" and becomes separate quantities:

    resident context capacity      (the CacheFitProved wall — done: 6 proved, 8 hard ceiling)
    active decode population       (W3's subject)
    new-prefill admission policy   (W1's price, W3's coexistence standing)
    per-step token budget          (2048 today; raising it is an OPTIMIZATION CANDIDATE that
                                    changes the activation/workspace subject — it goes through
                                    the memory model before any service promise reads it)
    cold-start latency objective
    decode-latency objective while prefill is active

The serving-liveness verdict (ArmServingLivenessVerdict, one open obligation) consumes W1–W3
evidence; the watchdog remains capture-only instrumentation and never mints a verdict. The
stall predicate ("no forward progress within T under admitted occupancy") is a named modeled
term over the engine's own counters before any further watchdog-armed run — see the owner
directive 2026-09-27 on watchdogs not substituting for modeling.

## Non-goals

No preemption-flag tuning, no max_num_batched_tokens change, and no service-shape restoration
decision until W1 attributes the cold cost. Group B stays down in the interim; the arm stays
Unestablished.
