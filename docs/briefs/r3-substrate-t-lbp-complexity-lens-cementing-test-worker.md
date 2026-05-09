---
status: complete for closure-receipt scope (implementation landed in #2271, focused tests verified by warm-stag-135 on 2026-05-09)
authority parent: R3 Substrate Manager (#1739)
ratification: T-LBP narrowed to complexity + cost lenses only per Q-Lens-Behavioral-Parity-R3-Closeability option (b) RATIFIED at gunbc#828 #issuecomment-4385329180 (zesty-bear-812, 2026-05-06)
roadmap row: §1.8 ledger row #79 (`complexity_lens_behaviorally_complete`) + §1.8 row #87 (`lens_cementing_test_discipline_complete`) + §1.8 row #166 (T-LBP `lens_behavioral_parity_demonstration`)
authority docs:
  - docs/r3-program-plan.md §1.8 row #79 (complexity_lens_behaviorally_complete: symbolic CostExpr + work/span split + asymptotic classification + cementing test)
  - docs/r3-program-plan.md §1.8 row #87 (lens_cementing_test_discipline_complete: every `.dag` lens has cementing test against frozen v2-oracle)
  - docs/r3-program-plan.md §1.8 row #166 (T-LBP demonstration consumes frozen v2-oracle cementing-test snapshot, NOT live v2 oracle)
  - docs/r2-structure.md §"Lane structure" T-Lens-Behavioral-Parity row ("cementing test against v2 oracle on same source")
  - src/v3/std/lens.dag (Lens<C> Director-locked carrier shape)
gates:
  - §1.8 row #79 (`complexity_lens_behaviorally_complete`) — cementing test bullet
  - §1.8 row #87 (`lens_cementing_test_discipline_complete`) — first lens entry
worker pin: warm-stag-135
---

# R3 Substrate T-LBP — `Lens<Complexity>` cementing-test worker brief

## Context

Q-Lens-Behavioral-Parity-R3-Closeability option (b) RATIFIED 2026-05-06
narrowed T-LBP R3 scope to **complexity + cost lenses only**;
parallelism + effect_enum carved to R4 per
`docs/r4-carve-out-routing.md`.

Per `docs/r2-structure.md` §"Lane structure" + §1.8 row #166: T-LBP
demonstration consumes **frozen v2-oracle snapshot** captured
pre-v2-retirement, NOT a live v2 oracle. This preserves
`v2_oracle_no_remaining_test_consumers` gate (per openai-pro
2026-05-06 finding 5).

This brief lands the cementing-test infrastructure for the complexity
lens specifically (cost lens follows in companion brief —
`r3-substrate-t-lbp-cost-lens-cementing-test-worker.md` Tier-1 5/5).

## Completion receipt

The preconditions cleared at `5a13ed800` after #2271 landed the
behaviorally complete complexity lens substrate and same-PR Band-C
cementing module:

- `src/v3/lenses/complexity.dag` is `STRUCTURALLY TERMINAL` /
  `BEHAVIORALLY COMPLETE`.
- `src/v3/compiler/src/lens_cost_generated.rs` exposes
  `ComplexitySummary` through `complexity_of`.
- `docs/v3-lens-capability-register.md` marks `complexity.dag`
  `COMPLETE` with the `src/v2/complexity.dag` counterpart and names
  `complexity_lens_behavioral_completion` as the frozen-oracle
  cementing dispatch.
- `src/v3/compiler/tests/integration/cementing/complexity_lens_behavioral_completion.rs`
  cements the published carrier on same-source fixtures for constant
  literal and recursive countdown cases: work, span, asymptotic class,
  and certainty.
- `src/v3/compiler/tests/integration/cementing/cementing_lens_registry_dispatch_test.rs`
  ratchets the register + `regen.dag` v2-complete slice so the
  cementing module and `tests/integration.rs` wiring cannot drift.

Warm-stag-135 re-verified the focused closure tests via BuildBuddy on
2026-05-09:

- `cargo test -p v3-compiler --test integration complexity_lens_behavioral_completion -- --nocapture`
  passed 2/2.
- `cargo test -p v3-compiler --test integration cementing_lens_registry_dispatch_test -- --nocapture`
  passed 14/14.

This closure receipt is intentionally narrower than the original
dispatch packet below. It records that the already-landed #2271
artifacts satisfy the complexity-lens cementing presence and focused
behavioral pins now required by `TESTING.md` Band-C. It does **not**
claim that warm-stag-135 re-ran the original full-workspace acceptance
suite, performed an external snapshot mutation test, or added new
code beyond the existing #2271 implementation.

## Historical dispatch packet

The remaining sections preserve the original 2026-05-06 dispatch packet
for provenance. They are not the live closure contract for this PR;
see **Completion receipt** above and **Acceptance accounting** below
for the evidence this receipt actually claims.

### Precondition gates

Brief dispatched after **all** of the following landed:

1. **`Lens<Complexity>` instance landed** in `src/v3/std/` (or canonical
   substrate location). Per row #79: requires symbolic CostExpr +
   work/span split + asymptotic classification.
2. **Frozen v2-oracle snapshot captured** for the chosen representative
   `.dag` source(s). Snapshot is the recorded v2-oracle output at a
   point when the v2 oracle was still authoritative; once captured,
   it's a stable file the cementing test consumes.
3. **T-CostLens-Composition shape precedent at `Lens<SymbolicCost>`**
   is referenced for cementing-test parity (worker greps the cost
   lens cementing test if landed first; mirrors the discipline).

If any precondition is missing at dispatch, STOP and surface — this
brief is consumer-side cementing-test landing, not substrate-producer.

### Scope

#### Deliverable 1 — Representative `.dag` source corpus

DFS-catalog `dsl/std/` and `src/v3/std/` to choose ≥1 representative
`.dag` source(s) that exercise:
- BindNode sequential composition (multi-step computation)
- BranchNode exclusive-choice (match arms producing different
  complexity classes)
- LoopNode bounded iteration (loop body × LoopBound)
- Mixed work/span split (parallelizable subexpressions surface
  shorter span than work)
- Asymptotic classification (constant / linear / log / quadratic /
  exponential — at least 2 classes represented for inverse-mapping
  diversity)

Per row #79's "asymptotic classification" — cementing test verifies
the lens outputs the expected asymptotic class for each
representative input.

#### Deliverable 2 — Frozen v2-oracle snapshot (consumer-only)

The snapshot is **NOT authored by this brief** — it must already exist
as a captured file (precondition #2 above). This brief consumes it.

If the snapshot does not yet exist at HEAD, STOP and surface to
Substrate Mgr — capture is a separate substrate-fact step (likely
owned by a different brief that runs against the live v2 oracle
pre-retirement). Bundling capture into this brief would require the
v2 oracle to still be live at PR time; per
`v2_oracle_no_remaining_test_consumers`, that's structurally barred.

#### Deliverable 3 — Cementing test landing

Author cementing test in `tests/` (or canonical equivalent — worker
greps existing `Lens<C>` cementing-test convention; T-CostLens
precedent if landed):

1. Read frozen v2-oracle snapshot file(s)
2. Run `Lens<Complexity>` over the same `.dag` source(s)
3. **Verify behavioral parity**: structural fold output matches
   frozen snapshot for each source — work/span values, asymptotic
   classification, per-Behavior CostExpr witnesses
4. **Fail-closed on divergence**: any mismatch (different work value,
   different class, missing per-Behavior witness) is test failure,
   NOT warning. Per `feedback_fail_closed_discipline` + INVARIANTS C-8
5. **Fail-closed on missing inputs**: if frozen snapshot is missing
   or `Lens<Complexity>` instance isn't loadable, test errors out
   (not skipped)

#### Deliverable 4 — §1.8 ledger receipt

Cementing test landing advances:
- Row #79 (`complexity_lens_behaviorally_complete`) — cementing test
  bullet of the multi-bullet acceptance flips DECLARED → satisfied
- Row #87 (`lens_cementing_test_discipline_complete`) — first lens
  entry. Companion brief (cost lens) closes the second; both required
  for #87 GREEN per option (b) ratification narrowing T-LBP to two
  lenses

### Slice — single PR

Phase ordering (PR-internal):
1. Verify preconditions at HEAD (Lens<Complexity> + snapshot) — STOP
   if missing
2. Author/select representative `.dag` source corpus
3. Author cementing test consuming frozen snapshot
4. Verify test green; verify fail-closed paths fire on injected
   divergence (mutation test: flip a work value in the snapshot,
   confirm test fails)
5. §1.8 ledger row updates (#79 cementing-test bullet; #87 first-lens
   entry)

## Acceptance accounting

The original acceptance list was authored before the #2271 landing and
before the in-tree Band-C discipline in `TESTING.md` settled on
register-driven cementing modules for v2-complete claims. Current
accounting is:

- **Satisfied by #2271:** `src/v3/compiler/tests/integration/cementing/complexity_lens_behavioral_completion.rs`
  landed and is wired through `tests/integration.rs`.
- **Satisfied by #2271:** the test pins the published
  `ComplexitySummary` carrier on same-source fixtures: work, span,
  asymptotic class, and certainty.
- **Satisfied by #2271:** `cementing_lens_registry_dispatch_test.rs`
  requires the `docs/v3-lens-capability-register.md` + `regen.dag`
  v2-complete slice to name the complexity cementing module and its
  `tests/integration.rs` path, fail-closed on missing wiring.
- **Verified by warm-stag-135 on 2026-05-09:** the focused complexity
  cementing module passed 2/2 via BuildBuddy.
- **Verified by warm-stag-135 on 2026-05-09:** the Band-C dispatch
  ratchet passed 14/14 via BuildBuddy.
- **Verified by warm-stag-135 in this PR:** `git diff --check` passed
  for this doc-only closure diff.
- **Not claimed by this closure PR:** external frozen-snapshot file
  mutation testing, full-workspace `cargo test`, `v2-compiler-tests`,
  and full clippy. Those checks exceed the scope of this receipt-only
  PR and remain CI / lane-close concerns, not evidence introduced by
  this document update.

## STOP-AND-ESCALATE

- **`Lens<Complexity>` instance not landed**: STOP — this brief is
  cementing-test consumer; substrate-producer brief is upstream.
  Reopen on Lens<Complexity> landing
- **Frozen v2-oracle snapshot not captured**: STOP — capture is
  separate brief (likely pre-v2-retirement). Reopen on snapshot
  landing. Do NOT capture-against-live-v2-oracle inside this brief
  (violates `v2_oracle_no_remaining_test_consumers`)
- **Snapshot reveals lens behavior diverges from v2 oracle on
  representative source**: STOP — this is real behavioral-parity
  divergence, not a test-authoring bug. Surface to Substrate Mgr;
  may indicate Lens<Complexity> instance bug requiring
  substrate-producer follow-on
- **Cementing test requires substrate-fact-introduction beyond
  cementing-test infrastructure** (e.g., new test-harness types):
  STOP — substrate gap; surface to Substrate Mgr
- **Bundled-scope drift**: do NOT bundle Lens<Complexity> instance
  edits or v2-oracle snapshot capture into this PR. Per Director
  bundled-scope ratification at gunbc#1739 #issuecomment-4392225548:
  parallel infrastructure DISALLOWED. This brief is cementing-test
  consumer only

## Authority audit receipt

1. **Substrate exists?** At brief-author time:
   - `Lens<C>` carrier landed (`src/v3/std/lens.dag`, 🟢 TERMINAL)
   - `Lens<Complexity>` instance — gates on T-LBP complexity-lens
     producer brief landing (worker re-greps at dispatch)
   - Frozen v2-oracle snapshot — gates on snapshot-capture brief
     landing (worker re-greps at dispatch)
2. **Existing brief?** No standalone cementing-test brief at HEAD.
   T-CostLens-Composition is shape precedent; its cementing test
   (companion brief Tier-1 5/5) is parallel-author
3. **Design-doc match?** §1.8 rows #79 + #87 + #166 + r2-structure.md
   §"Lane structure" name the cementing-test discipline against
   frozen v2-oracle snapshot. This brief is the dispatch packet for
   row #79 cementing-test bullet
4. **Citations live?** Worker re-verifies at dispatch — preconditions
   gate dispatch
5. **Carrier dissolves the bridge?** Yes — frozen v2-oracle snapshot
   is the dissolution mechanism. Pre-v2-retirement: live oracle is
   the parity reference. Post-v2-retirement: snapshot is the parity
   reference. The "bridge" is the dependency on a live v2 oracle for
   parity verification; snapshot dissolves it by capturing the oracle
   output as a stable file

## Provenance

Drafted 2026-05-06 per Tier-1 brief-queue commitment at gunbc#1858
(R3 Substrate Mgr standing assignment) — 4/5 in queue. Companion
brief `r3-substrate-t-lbp-cost-lens-cementing-test-worker.md`
(Tier-1 5/5) covers cost lens; both required for §1.8 row #87
GREEN. Brief queues post-precondition-landing; worker pin assigned
at that time.
