---
status: draft (worker brief; queued — gates on T-LBP cost-lens substrate landing + frozen v2-oracle snapshot capture)
authority parent: R3 Substrate Manager (#1739)
ratification: T-LBP narrowed to complexity + cost lenses only per Q-Lens-Behavioral-Parity-R3-Closeability option (b) RATIFIED at gunbc#828 #issuecomment-4385329180 (zesty-bear-812, 2026-05-06)
roadmap row: §1.8 ledger row #80 (`cost_lens_behaviorally_complete`) + §1.8 row #70 (`cost_lens_demonstration`) + §1.8 row #87 (`lens_cementing_test_discipline_complete`) + §1.8 row #166 (T-LBP `lens_behavioral_parity_demonstration`)
authority docs:
  - docs/r3-program-plan.md §1.8 row #80 (cost_lens_behaviorally_complete: Dimension<SymbolicCost> wiring + SizeVar value semantics + cementing test)
  - docs/r3-program-plan.md §1.8 row #70 (cost_lens_demonstration: ≥2 algebra-instances composed + ≥1 recursive call + observable cost-bound output)
  - docs/r3-program-plan.md §1.8 row #87 (lens_cementing_test_discipline_complete: every `.dag` lens has cementing test against frozen v2-oracle)
  - docs/r3-program-plan.md §1.8 row #166 (T-LBP demonstration consumes frozen v2-oracle cementing-test snapshot, NOT live v2 oracle)
  - docs/r2-structure.md §"Lane structure" T-Lens-Behavioral-Parity row ("cementing test against v2 oracle on same source")
  - src/v3/std/lens.dag (Lens<C> Director-locked carrier shape)
  - r3-substrate-t-lbp-complexity-lens-cementing-test-worker.md (companion brief Tier-1 4/5; same discipline applied to complexity lens)
gates:
  - §1.8 row #80 (`cost_lens_behaviorally_complete`) — cementing test bullet
  - §1.8 row #70 (`cost_lens_demonstration`) — algebra-composition + recursive-call demonstration
  - §1.8 row #87 (`lens_cementing_test_discipline_complete`) — second lens entry (closes #87 with companion brief)
worker pin: TBD (queued post-T-LBP cost-lens substrate landing)
---

# R3 Substrate T-LBP — `Lens<SymbolicCost>` cementing-test + cost-lens-demonstration worker brief

## Context

Q-Lens-Behavioral-Parity-R3-Closeability option (b) RATIFIED 2026-05-06
narrowed T-LBP R3 scope to **complexity + cost lenses only**.
This brief lands the cementing-test infrastructure for the cost lens
specifically; companion brief
`r3-substrate-t-lbp-complexity-lens-cementing-test-worker.md`
covers complexity lens. Both required for §1.8 row #87 GREEN.

This brief additionally absorbs §1.8 row #70 `cost_lens_demonstration`
(≥2 algebra-instances + ≥1 recursive call + observable cost-bound
output) — the demonstration is naturally co-located with the
cementing test (same source corpus + lens run; demonstration adds
observable-output assertion on top of parity check).

## Precondition gates

Brief dispatches when **all** of the following land:

1. **`Lens<SymbolicCost>` instance landed** in `src/v3/std/` per row
   #80: requires `Dimension<SymbolicCost>` wiring + `SizeVar` value
   semantics. Per T-CostLens-Composition; this is the substrate-producer
   side, owned upstream
2. **Frozen v2-oracle snapshot captured** for the chosen representative
   `.dag` source(s) — recorded pre-v2-retirement, stable file
3. **Cost-lens-readable algebra-instances** exist in `dsl/std/` ≥2;
   per row #37 (`cost_lens_reads_target_realization`) and row #70
   (≥2 algebra-instances composed). Worker DFS-catalogs at dispatch

If any precondition is missing at dispatch, STOP and surface — this
brief is consumer-side cementing-test landing, not substrate-producer.

## Scope

### Deliverable 1 — Representative `.dag` source corpus (row #70 demo shape)

DFS-catalog `dsl/std/` + `src/v3/std/` for sources that exercise:
- **≥2 algebra-instances composed** (e.g., `Int<32>` + `Float<64>`
  per S9 Phase-1 Step 3 + Phase-2 emission entries; or `AbelianGroup`
  + `CommutativeSemiring` per the existing algebra carrier set)
- **≥1 recursive call** — function definition that calls itself,
  exercising the iterate composition op + LoopBound semantics
- **Observable cost-bound output** — lens output is comparable to
  closed-form cost expression (e.g., `O(n)` for linear-recursive
  fold over List<T>; `O(n²)` for nested loop)

Per row #70's "≥2 algebra-instances composed + ≥1 recursive call":
the demonstration source is the cementing-test source plus
demonstration-specific assertions on top of parity.

### Deliverable 2 — Frozen v2-oracle snapshot (consumer-only)

Same precondition as companion brief: snapshot is captured separately
and consumed here. STOP and surface if missing at HEAD; do NOT
capture-against-live-oracle inside this brief (violates
`v2_oracle_no_remaining_test_consumers`).

### Deliverable 3 — Cementing test + demonstration landing

Author cementing test in `tests/` (or canonical equivalent — worker
greps existing `Lens<C>` cementing-test convention; complexity-lens
companion-brief precedent if landed first):

1. Read frozen v2-oracle snapshot file(s)
2. Run `Lens<SymbolicCost>` over the same `.dag` source(s)
3. **Verify behavioral parity**: structural fold output matches
   frozen snapshot — `Dimension<SymbolicCost>` values, `SizeVar`
   bindings, per-Behavior cost witnesses, recursive-call cost
   composition
4. **Cost-lens-demonstration assertions** (row #70):
   - Output cost expression evaluates to expected closed-form bound
     for ≥1 representative source (e.g., assert `O(n)` for the
     linear-recursive fold case)
   - ≥2 algebra-instances visibly composed in the output (assert
     both algebra-instance carrier names appear in the cost
     expression's structural decomposition)
5. **Fail-closed on divergence**: any parity mismatch OR missing
   demonstration assertion is test failure, NOT warning. Per
   `feedback_fail_closed_discipline` + INVARIANTS C-8
6. **Fail-closed on missing inputs**: missing snapshot or missing
   `Lens<SymbolicCost>` instance → test errors out (not skipped)

### Deliverable 4 — §1.8 ledger receipt

Cementing test + demonstration landing advances:
- Row #80 (`cost_lens_behaviorally_complete`) — cementing test bullet
  flips DECLARED → satisfied
- Row #70 (`cost_lens_demonstration`) — demonstration assertion
  receipt; flips DECLARED → satisfied
- Row #87 (`lens_cementing_test_discipline_complete`) — second lens
  entry. Companion brief (complexity lens) closes the first; both
  required for #87 GREEN

## Slice — single PR

Phase ordering (PR-internal):
1. Verify preconditions at HEAD (Lens<SymbolicCost> + snapshot +
   ≥2 algebra-instances) — STOP if missing
2. Author/select representative `.dag` source corpus exercising
   ≥2 algebra-instances + ≥1 recursive call + observable cost bound
3. Author cementing test consuming frozen snapshot + demonstration
   assertions on top
4. Verify test green; verify fail-closed paths fire on injected
   divergence + missing demonstration assertion
5. §1.8 ledger row updates (#80 cementing-test bullet; #70
   demonstration; #87 second lens entry)

## Acceptance

- Cementing test landed in `tests/` (or canonical equivalent) consuming
  frozen v2-oracle snapshot for ≥1 representative `.dag` source
- Test verifies behavioral parity per row #80: `Dimension<SymbolicCost>`
  values, `SizeVar` semantics, per-Behavior cost witnesses, recursive
  composition
- **Cost-lens-demonstration assertions** (row #70):
  - Output evaluates to expected closed-form bound for ≥1 source
  - ≥2 algebra-instances visibly composed in cost expression
  - ≥1 recursive call exercises iterate semantics
- Mutation-test verification: injected divergence + missing
  demonstration assertion both cause test failure
- Both fail-closed paths verified: missing snapshot → error;
  missing Lens<SymbolicCost> → error
- §1.8 row #80 cementing-test bullet satisfied; row #70 demonstration
  satisfied; row #87 second lens entry recorded (closes #87 GREEN
  with companion complexity-lens brief)
- `cargo test --workspace --exclude v2-compiler-tests` green
- `cargo test -p v2-compiler-tests` green; strict-compile diagnostic
  ratchet at 0
- `cargo clippy --all-targets -- -D warnings` clean
- `cargo fmt --all --check` clean
- Citation discipline per `docs/briefs/brief-authoring-checklist.md`
- 5-question authority audit in PR body

## STOP-AND-ESCALATE

- **`Lens<SymbolicCost>` instance not landed**: STOP — substrate-producer
  upstream. Reopen on Lens<SymbolicCost> landing
- **Frozen v2-oracle snapshot not captured**: STOP — capture is
  separate brief. Do NOT capture-against-live-oracle (violates
  `v2_oracle_no_remaining_test_consumers`)
- **<2 algebra-instances available at HEAD**: STOP — row #70
  demonstration shape requires ≥2; if S9 Phase-1 Step 3 + Phase-2
  emission entries haven't landed, demonstration source corpus
  insufficient. Surface to Substrate Mgr; brief queues until
  algebra-instances catalog passes the threshold
- **Snapshot reveals lens behavior diverges**: STOP — real behavioral-
  parity divergence, surface to Substrate Mgr; may indicate
  `Lens<SymbolicCost>` instance bug
- **Demonstration source's closed-form bound is non-trivial**:
  if asserting `O(n)` for the recursive-fold case requires evaluator-
  side symbolic-simplification beyond what's structurally present,
  scope the assertion to "cost expression contains size-variable n
  with degree 1" rather than full closed-form match. Document the
  scoping in PR body
- **Bundled-scope drift**: do NOT bundle `Lens<SymbolicCost>` edits,
  v2-oracle snapshot capture, or new algebra-instance authoring.
  Parallel infrastructure DISALLOWED per Director ratification at
  gunbc#1739 #issuecomment-4392225548. Cementing-test + demonstration
  consumer only

## Authority audit receipt

1. **Substrate exists?** At brief-author time:
   - `Lens<C>` carrier landed (🟢 TERMINAL)
   - `Lens<SymbolicCost>` instance — gates on T-CostLens-Composition
     producer landing (worker re-greps at dispatch)
   - Frozen v2-oracle snapshot — gates on snapshot-capture brief
   - ≥2 algebra-instances at HEAD — gates on S9 Phase-1 Step 3 +
     Phase-2 emission entries OR existing algebra carriers (worker
     greps at dispatch)
2. **Existing brief?** No standalone cementing-test brief at HEAD.
   Companion complexity-lens brief (Tier-1 4/5) is parallel-author
3. **Design-doc match?** §1.8 rows #80 + #70 + #87 + #166 +
   r2-structure.md §"Lane structure" name the cementing-test +
   demonstration discipline. This brief is the dispatch packet for
   row #80 cementing-test bullet + row #70 demonstration
4. **Citations live?** Worker re-verifies at dispatch
5. **Carrier dissolves the bridge?** Yes — frozen v2-oracle snapshot
   dissolves the live-oracle dependency; demonstration assertions
   dissolve the "is the cost lens behaviorally faithful?" bridge by
   reading observable cost-bound output

## Provenance

Drafted 2026-05-06 per Tier-1 brief-queue commitment at gunbc#1858
(R3 Substrate Mgr standing assignment) — 5/5 in queue, completing
the Tier-1 batch. Companion brief
`r3-substrate-t-lbp-complexity-lens-cementing-test-worker.md`
(Tier-1 4/5) covers complexity lens; both required for §1.8 row #87
GREEN per option (b) ratification narrowing T-LBP to two lenses.
Brief queues post-precondition-landing; worker pin assigned at that
time.
