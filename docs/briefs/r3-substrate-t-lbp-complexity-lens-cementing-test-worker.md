---
status: draft (worker brief; queued — gates on T-LBP complexity-lens substrate landing + frozen v2-oracle snapshot capture)
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
worker pin: TBD (queued post-T-LBP complexity-lens substrate landing)
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

## Precondition gates

Brief dispatches when **all** of the following land:

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

## Scope

### Deliverable 1 — Representative `.dag` source corpus

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

### Deliverable 2 — Frozen v2-oracle snapshot (consumer-only)

The snapshot is **NOT authored by this brief** — it must already exist
as a captured file (precondition #2 above). This brief consumes it.

If the snapshot does not yet exist at HEAD, STOP and surface to
Substrate Mgr — capture is a separate substrate-fact step (likely
owned by a different brief that runs against the live v2 oracle
pre-retirement). Bundling capture into this brief would require the
v2 oracle to still be live at PR time; per
`v2_oracle_no_remaining_test_consumers`, that's structurally barred.

### Deliverable 3 — Cementing test landing

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

### Deliverable 4 — §1.8 ledger receipt

Cementing test landing advances:
- Row #79 (`complexity_lens_behaviorally_complete`) — cementing test
  bullet of the multi-bullet acceptance flips DECLARED → satisfied
- Row #87 (`lens_cementing_test_discipline_complete`) — first lens
  entry. Companion brief (cost lens) closes the second; both required
  for #87 GREEN per option (b) ratification narrowing T-LBP to two
  lenses

## Slice — single PR

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

## Acceptance

- Cementing test landed in `tests/` (or canonical equivalent) consuming
  frozen v2-oracle snapshot for ≥1 representative `.dag` source
- Test verifies behavioral parity across all complexity-lens output
  facets per row #79: work/span split + asymptotic class +
  per-Behavior CostExpr
- Mutation-test verification: injected divergence in snapshot causes
  test failure (fail-closed path verified)
- Both fail-closed paths verified: missing snapshot → error (not
  skip); missing Lens<Complexity> → error (not skip)
- §1.8 row #79 cementing-test bullet flips DECLARED → satisfied
- §1.8 row #87 first lens entry recorded (cost lens follows in
  companion brief; #87 GREEN gates on both)
- `cargo test --workspace --exclude v2-compiler-tests` green (3
  pre-existing v2-compiler --lib failures verified unrelated)
- `cargo test -p v2-compiler-tests` green; strict-compile diagnostic
  ratchet at 0
- `cargo clippy --all-targets -- -D warnings` clean
- `cargo fmt --all --check` clean
- Citation discipline per `docs/briefs/brief-authoring-checklist.md`
- 5-question authority audit in PR body

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
