# R3 Wave-1 PB1 — Tier3 gate #2 (`tier3_computation_mirror_dissolved`) worker brief

**Status:** DISPATCH-READY (pre-authored worker brief per `docs/r3-remaining-work-dependency-graph.md` §5 Wave-1).

**Owning manager:** R3 PB Manager (nimble-crab-786 lineage).

**Lane:** T-Tier3-Dissolution — R3 §1.8 gate **#2** only (computation mirror slice).

**Authority:** [`docs/r3-structure.md`](../r3-structure.md) §"Acceptance — `.dag` gates" (`tier3_computation_mirror_dissolved`); [`docs/r3-program-plan.md`](../r3-program-plan.md) §1.8 row #2 + Status-at-HEAD paragraph; [`INVARIANTS.md`](../../INVARIANTS.md) **P1** (modeling faithfulness), **P2** (single authority), **P5** (progress is dissolution).

**Modeling discipline:** Before introducing any new type or carrier, **DFS the concept DAG** per [`MODELING.md`](../../MODELING.md) §M9 — attach to an existing `dsl/std/` concept; do not mint parallel spellings.

## Preconditions (read first)

1. **Landlord slice already on main:** `kernel_algebra_profile` substrate routing is **CONSUMER_LANDED** — integration ratchet `tier3_computation_mirror_kernel_algebra_profile_substrate_authority` in `src/v3/compiler/tests/integration/m2_substrate_inhabitance_test.rs` (see §1.8 #2 Notes in `r3-program-plan.md`). Worker **does not** re-litigate that receipt unless a regression is found.
2. **Pattern receipts:** Gate **#1** (`termination_lattice_rust_mirror_dissolved`) — `termination_lattice_rust_mirror_dissolved` test forbids reintroduced public lattice mirrors. Gate **#4** (`tier3_effect_carrier_mirror_dissolved`) — co-located projection + ratchet pattern in `dag/effects.rs` lineage. Use these as **shape references**, not copy-paste authority.

## Scope (this worker)

Advance **#2** toward **PASSING** per `r3-structure.md` prose: `std.computation` is sole authority for the named surface (`ShrinkFactor`, `IterationPrimitive`, `kernel_algebra_profile` end-state), dissolving **parallel executable Rust mirrors** as Evaluator + std-body lowering permit.

**In scope for Wave-1 worker (pick the narrowest mechanical row; do not boil the ocean):**

- Retire or further isolate **scaffold** mirrors in `src/v3/compiler/src/dag.rs` nested `mod computation` where `src/v3/std/computation.dag` is already authoritative and `ArrowBody::Unparsed` is the only honest bootstrap gap — document the gap if code cannot move yet.
- Extend **fail-closed** integration receipts in `m2_substrate_inhabitance_test.rs` (same style as gate #1 string ratchet) for any **dissolved** public surface you remove, so reintroduction fails CI.
- Keep **C1 perf-budget** artifacts honest: if you delete mirror entrypoints, update `src/v3/compiler/benches/tier3_mirror_perf.rs` + `tier3_baseline.json` / `scripts/aggregate_tier3_baseline.py` per [`docs/audit/c1-tier3-baseline-capture-procedure.md`](../audit/c1-tier3-baseline-capture-procedure.md) and [`docs/briefs/r3-pb-t-tier3-consumer-slice-worker.md`](r3-pb-t-tier3-consumer-slice-worker.md).

**Explicitly out of scope (STOP-and-escalate / PM bridge):**

- New `ValueBody` variants, new connective shapes, or broad std-body evaluation semantics — route to **Substrate Mgr (warm-wolf-698)** via **PM** per Director dispatch note on tier3 carriers.
- R1 SG-0 census chasing except **incidental** deltas from files this worker edits — report `SG-0 hand-path delta:` in PR description when `sg0_census_test.rs` moves.

## STOP — escalate to PB Mgr / Director

- Evaluator or map read-path is missing for a dissolution you thought mechanical.
- Dissolution would change **fail-closed** termination / cost / lens behavior without a frozen-oracle or existing parity test update.
- Gate #2 **Status** promotion to **PASSING** would be dishonest (slice-only); keep **CONSUMER_LANDED** + Notes until full §Acceptance is true.

## Deliverables

1. One implementation PR (or stacked PR) with: code + tests + **§1.8 row #2** + Status-at-HEAD sync in `docs/r3-program-plan.md` when status genuinely moves; `docs/r3-remaining-work-dependency-graph.md` §3/§4/§5 touch **only** if lane counts or Wave-1 table change.
2. PR description cites this brief path and names the **smallest** dissolution row executed.
3. `cargo test -p v3-compiler` (narrower if justified) + `cargo fmt` clean for edited Rust.

## Merge discipline (Mgr-self-authorized PB lane)

Per operator precedent on scoped PB merges: squash-merge message must carry **(i)** lane id + gate id, **(ii)** brief path + authority docs touched, **(iii)** SG-0 / perf-budget audit-trail or explicit "none".
