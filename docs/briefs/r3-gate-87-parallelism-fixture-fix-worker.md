# R3 Gate #87 — Parallelism Fixture / Suites-Table Drift Fix

**Status**: Ready-for-dispatch (blocked on operator retirement of gate-87 decomposition respawn loop; see swift-deer-459 ↔ zesty-bear-812 thread msg_d1af6d8e / msg_871317b0)

**Scope**: ~50 LOC, single targeted PR, single worker, ~1 cycle to land. **Not** a multi-slice decomposition.

## Problem

`PR #2795` (commit `b86510086`, T-Cluster-F-α S1 walker port) landed `lens_parallelism_entry` in `src/v3/compiler/regen.dag:69` but did not co-land the corresponding cementing-test fixture or suites-table row. This creates a registry/suites drift:

- `grep -cE "^data lens_.*_entry: LensRegistryEntry" src/v3/compiler/regen.dag` → **11**
- `grep -cE "include_str.*t_r3_gate_87" src/v3/compiler/src/r3_gate_87_cementing_regen_runner_suites.rs` → **10**

The fixture-inventory assertion at `src/v3/compiler/tests/integration/r3_gate_87_lens_cementing_regen_receipts_test.rs:135` (`r3_gate_87_regen_lens_registry_names_match_fixture_inventory`) compares the registry-derived name set against the suites-derived set and currently asserts `11 != 10` on main. Gate #87 ledger row at `docs/r3-program-plan.md:315` claims CONSUMER_LANDED+PASSING but is stale post-#2795 land.

## Deliverables

### 1. Author parallelism fixture

Create `src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_parallelism.dag` mirroring the structural shape of an existing fixture (`t_r3_gate_87_cementing_regen_cost.dag` is the closest precedent — same Lane-E differential-cost pattern can be reused if a parallelism-bearing program is authored; otherwise `t_r3_gate_87_cementing_regen_provenance.dag` or another non-cost lens fixture may be a better template depending on what `lens_parallelism::analyze_parallelism` consumes/emits).

**Predicate selection**: pick the receipt shape that is *behavior-bearing* per `TESTING.md` Band-C (`LensOutputEquals` / `DifferentialEquals` / `SymbolicCostExprEquals`) over a `Compiles` placeholder. The lens body lives at `src/v3/lenses/parallelism.dag` + `src/v3/compiler/src/lens_parallelism_generated.rs` — grep for the lens's output shape to pick the right predicate.

**Claim name**: `cementing_regen_parallelism` (matches the suites-row `claim_names` field below).

### 2. Add suites-table row

Append to `R3_GATE_87_CEMENTING_REGEN_SUITES` in `src/v3/compiler/src/r3_gate_87_cementing_regen_runner_suites.rs:61` (preserve alphabetical/grouping order of existing entries):

```rust
(
    include_str!("../tests/dag/t_r3_gate_87_cementing_regen_parallelism.dag"),
    "src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_parallelism.dag",
    "r3_gate_87_cementing_regen_parallelism_suite",
    &["cementing_regen_parallelism"],
),
```

### 3. Verify the receipt suite

Run locally (via `ctrl-build`):

```
cargo test -p v3-compiler r3_gate_87
cargo test -p v3-compiler r3_gate_87_regen_lens_registry_names_match_fixture_inventory
cargo test -p v3-compiler sg0_census
```

All three must be green at HEAD before opening the PR.

### 4. Ledger touch (out of worker scope — PM-tier)

Worker does NOT edit `docs/r3-program-plan.md` row #87. After merge, PM (deep-wolf-155) refreshes the §1.8 row to note parallelism coverage. Worker brief surfaces this as a follow-up note in the PR body.

## Anti-patterns to avoid

- **Do not** re-decompose this into multi-slice work. This is a single-row table addition + one fixture file.
- **Do not** touch unrelated regen.dag entries, `cementing_dispatch.rs`, `test_runner.rs`, or T-Lens-Self-Application surfaces (gates #57/#58/#59 are an orthogonal lane).
- **Do not** open parallel PRs (`#2834/#2835/#2836/#2841/#2843/#2845/#2846/#2850` are respawn-loop artifacts pending operator triage — ignore as authority).

## Coordination

- Brief author: swift-deer-459 (R3 Verification Mgr)
- Director ratification: zesty-bear-812 msg_d1af6d8e (2026-05-13T05:39Z)
- Dispatch trigger: operator retirement of work-item `adhoc-b75b3d90-3d0` + 8 spawned successor adhocs
- On dispatch: worker reports back to swift-deer-459; squash-merge per gunbc policy once review-clean + CI-green
