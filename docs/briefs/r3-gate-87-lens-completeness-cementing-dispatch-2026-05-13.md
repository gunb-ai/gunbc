# R3 Gate 87 Lens-Completeness Cementing Dispatch — 2026-05-13

**Owner:** Verification Mgr session `valiant-otter-427`.

**Purpose:** decompose the remaining test-discipline work around the
lens-completeness invariant without reopening gate #87's already-landed
`regen.dag` corpus. The child items below are dispatch packets; each worker
must make a concrete PR or return a named blocker.

## Authority

- `docs/r3-program-plan.md` row #87 is the status authority for
  `lens_cementing_test_discipline_complete`: **CONSUMER_LANDED + PASSING** for
  the `src/v3/compiler/regen.dag` `LensRegistryEntry` corpus.
- `docs/r3-structure.md` §"Acceptance" defines that gate-#87 corpus: each
  `regen.dag` registry row has a paired `.dag` harness, PB-B-1 runner entry,
  and Rust pin receipt where `.dag` predicates remain intentionally narrower.
- `TESTING.md` §"Cementing tests (Band C — lens subsumption)" defines the
  durable discipline for any future claim that a v3 lens is behaviorally
  complete or replaces a real v2 analysis.
- `docs/v3-lens-capability-register.md` plus
  `src/v3/std/verification.dag::lens_capability_register_rows` are the
  register/projection authorities for lens-completeness claims.

## Scope Lock

This dispatch does **not** ask workers to prove gate #87 again. The current
gate-#87 pass means exhaustive coverage of the `regen.dag` enumeration surface:

- `src/v3/compiler/src/r3_gate_87_cementing_regen_runner_suites.rs`
- `src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_*.dag`
- `src/v3/compiler/tests/dag/cementing_dispatch.dag`
- `src/v3/compiler/tests/integration/r3_gate_87_lens_cementing_regen_receipts_test.rs`

The remaining work is the discipline envelope around the broader
lens-completeness invariant: when a row is promoted to COMPLETE, when a Rust
pin remains because a carrier is not authorable as `.dag` data, or when the
capability register changes, the cementing receipt must move in the same PR.

## Sub-Items To Dispatch

### G87-LC-1 — Register Completeness Drift Guard

**Goal:** make the lens-completeness invariant fail closed when the capability
register and cementing dispatch surfaces disagree.

Worker brief:
- Compare `docs/v3-lens-capability-register.md` against
  `src/v3/std/verification.dag::lens_capability_register_rows`.
- Verify every COMPLETE row with a real v2 counterpart projects into
  `cementing_dispatch.dag` and, when also present in `regen.dag`, into the
  gate-#87 runner table.
- Land a focused ratchet or tighten an existing one if any drift is possible
  without a failing test. If no gap exists, land a no-code audit receipt that
  names the exact commands and files checked.

Acceptance:
- A future COMPLETE + real-v2 row cannot merge without a Band-C receipt row or
  an explicit failing diagnostic naming the missing row.
- Any no-code outcome cites the existing failing path that already enforces
  this condition.

### G87-LC-2 — COMPLETE-Flip Same-PR Checklist

**Goal:** give lens owners a mechanical checklist for future COMPLETE
promotions so Band-C cementing cannot be deferred to a later PR.

Worker brief:
- Update the relevant live authoring guidance, preferably `TESTING.md` plus the
  closest lens-register prose, with a short same-PR checklist.
- The checklist must distinguish real-v2 `DifferentialEquals` / frozen-oracle
  receipts from v3-native `LensOutputEquals` receipts and helper-only
  `Compiles` placeholders with named dissolution triggers.
- Include the gate-#87 single-authority surfaces that must be edited together
  when the promoted lens is a `regen.dag` registry row.

Acceptance:
- A reviewer can decide from one checklist whether a COMPLETE flip is missing
  a cementing artifact.
- The checklist does not introduce a new authority or contradict row #87's
  already-passing status.

### G87-LC-3 — Rust Pin Blocker Ledger

**Goal:** convert the remaining temporary Rust cementing pins into explicit
carrier/blocker work, so the lens-completeness invariant has no invisible
exceptions.

Worker brief:
- Start from `docs/briefs/r3-cementing-discipline-pattern-2026-05-12.md` §3
  and `src/v3/compiler/tests/integration/sg0_census_test.rs`.
- For each Rust cementing residual, confirm whether it is a true Band-C lens
  cementing residual, a gate-#87 infrastructure pin, or a different lane's
  demonstration/host-wrapper receipt.
- Land doc or test-census comment updates that name the owning unblocker and
  the exact `.dag` target shape after unblock.

Acceptance:
- No Rust path under `tests/integration/cementing/` can be mistaken for an
  unowned gate-#87 exception.
- Each true Band-C residual has one named blocker and one expected replacement
  shape.

### G87-LC-4 — Next Carrier-Unblock Pilot

**Goal:** pick one blocked temporary Rust cementing receipt and either migrate
it to `.dag` or prove the smallest missing carrier work.

Worker brief:
- Choose the smallest true Band-C residual from the blocker ledger, not a
  non-cementing host-wrapper receipt.
- If the expected carrier is now authorable as `.dag` data, port the receipt:
  add or extend the `.dag` `TestClaim`, remove the Rust test/census row, and
  run the relevant gate-#87 or PB-B-1 runner slice.
- If it is still blocked, land a minimal failing/diagnostic receipt or design
  packet that names the exact carrier/predicate gap and routes it to the owning
  lane.

Acceptance:
- Either SG-0 hand-authored test count shrinks by one with equivalent Band-C
  coverage, or the blocker becomes a concrete upstream work item rather than a
  prose caveat.

### G87-LC-5 — Non-`regen.dag` Lens Coverage Sweep

**Goal:** keep the broader lens library honest without expanding gate #87's
corpus retroactively.

Worker brief:
- Enumerate lens-like `.dag` files outside the `regen.dag` registry and classify
  them as register-tracked, v3-native helper, not a shipped lens, or missing a
  register row.
- For any missing row or explicit subsumption claim, either add the register
  row with the correct behavioral status or file the needed substrate/design
  blocker.
- Do not add those files to the gate-#87 runner unless they become
  `LensRegistryEntry` rows.

Acceptance:
- The register remains the single place to check whether a lens is COMPLETE,
  PROXY, STUB, PARTIAL, or out of scope.
- Non-`regen.dag` work is routed through Band-C/register discipline, not hidden
  under the already-passing gate #87 row.

## Dispatch Notes

Workers should prefer small PRs. If a worker finds an executable gap in the
current gate-#87 corpus, that is higher priority than prose cleanup and should
be fixed directly. If a worker finds only future-discipline debt, the PR should
make that debt mechanically checkable or route it to a concrete owner.

Suggested smoke commands:

```bash
cargo test -p v3-compiler r3_gate_87
cargo test -p v3-compiler cementing_dispatch_suite_passes_through_runner
```

Docs-only dispatch or audit PRs may skip those commands, but must say why in
their final report.

