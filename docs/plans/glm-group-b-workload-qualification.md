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

## The three qualification workloads (replace the single cold stress test)

W1. ONE COLD FULL-WINDOW REQUEST — the attributable cold-prefill cost. One 262K request,
    nothing else admitted. Per-request timing: submission, engine arrival, first scheduled
    work, prefill completion / first output, request completion, scheduled tokens per step,
    plus operation-level durations and rank waits for a small number of representative
    steps. The output is the cold-prefill service time with its dominant term named
    (compute / indexer / communication / host / cache-recompute / unresolved).

W2. WARM CONTINUATION — does the runtime ACTUALLY reuse the established prefix/state? Same
    prompt as W1 plus a small new suffix, issued after W1 completes. The witness is actual
    reuse: cached-vs-newly-computed token counts from the engine's own accounting, never
    "prefix caching enabled" as a proxy. The pinned scheduler's hybrid-state cache-boundary
    handling is named in the model so a boundary miss is a located fact, not a surprise.

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
