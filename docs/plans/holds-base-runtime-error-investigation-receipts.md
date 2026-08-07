# Holds.base runtime error — investigation receipts (2026-08-06)

Status: root cause found and fixed elsewhere — **#7916**. This document is
investigation receipts only; no code lands with it.

## What #7916 found and fixed

Under selection-applied per-entry assembly, a `Holds{value:..}` record
literal could lose its `Witness` `parent_enum` when `lookup_variant_parent_enum`
missed under a composed symbol index, producing a bare `Record{type_name:Holds}`
instead of a `Variant{type_name:Witness,variant_name:Holds}`. That nested into
`Holds{value:Holds{value:Refined}}` at the next witness-wrap site — which is
why `refined_base` received a `Holds` where a `Refined` was expected, and why
cold/unselected runs (a different composed symbol index) stayed green on the
identical tree. Fixed in `src/v1/04_infer.dag` and
`src/v1/stage0/src/v1_compiler_infer.rs` (~40 lines, 2 files), merged
2026-08-06T16:33.

## Hypotheses this session eliminated before #7916 was found

Each disconfirmed with its own control; none of them turned out to be the
cause, but each is a durable negative result worth keeping on record so a
future session does not re-walk the same dead ends.

1. **Stale/pointer-cache reuse** (`param_name_cache`, `var_sym_cache`,
   `call_func_name_cache` lacking keepalives in the v1 Rust interpreter).
   Disconfirmed via CI run
   https://github.com/gunb-ai/gunbc/actions/runs/31104786863 with the
   keepalive fix applied in the failing configuration: the error persisted
   byte-identical. The keepalive fix itself landed separately as **#7920**
   ("Interpreter cache liveness repair") because it is a real hygiene fix
   (mirrors the existing `PureCallMemo`/`EvalRecomputeTrace`/`EvalCallMemo`
   `keepalive_fns` discipline) even though it does not explain this red.
   Caveat, explicit: the *collision rate* of these three caches before the
   keepalive fix was never separately measured. #7920 makes the keys valid;
   it does not establish they were never colliding. Do not read the merge of
   #7920 as proof they were collision-free.
2. **Pool-membership/name collision** (DESIGN.md Class B, #6985).
   Disconfirmed via static analysis: no colliding declarations on the call
   path, and the resolver fails closed on ambiguity.
3. **Schedule-derived module eviction.** Disconfirmed via CI run
   https://github.com/gunb-ai/gunbc/actions/runs/31099803750 with
   `GUNBC_SCHEDULE_RETENTION_EVICT=0` (an existing measurement control): the
   error persisted byte-identical.
4. **Typecheck-cache/generic-instantiation collision at `refined_base`'s
   generic boundary.** Never tested — superseded by #7916's diagnosis before
   this hypothesis could be independently confirmed or disconfirmed.

## Deferred post-merge verification

Branch `verify-7916-holds-base` (origin) carries #7916's head plus the two
files that force the selection-narrowed CI configuration that reproduced this
red (`src/v2/lens/synthesis/synthesis_gap_polynomial.dag`,
`src/v2/test/claim/ci_floor_plan_witness_test.dag`, both comment/no-op
touches used only to perturb the affected-set selection). It is a deferred
receipt — CI floor with selection-applied on the reproducing tree, plus
`cargo test -p v1-compiler --lib` — to be run once the CI fleet is healthy,
not a merge gate for #7916 (which already merged before this receipt could
complete; the fleet was stalled fleet-wide, not merely flaky, at the time).

## Located but not fixed (out of scope here)

A clamp defect was located in `claim_executor.rs`'s `run_discovery_batch_node`
/ `batch_runtime_unit_count` path during this investigation. It is unrelated
to the Holds.base root cause and is left for a separate PR; no code changes
for it are carried in this branch.

## Board context

This work item traces to gunb-ai/gunbc PR #7907, dispatched as a child of
parent session smart-badger-549 ("CI Perf (actual benefits)"). #7869
("still-bee") separately carries unmerged process-exit-carrier work; it is
not part of this investigation and does not land here.
