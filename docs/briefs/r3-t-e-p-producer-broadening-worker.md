---
status: draft (wait-window; parallel-author posture per Director #issuecomment-4377264483; non-Evaluator-gated lane carve-out per docs/r3-structure.md)
authority parent: R3 Substrate Manager (#1739)
ratification: Director ratified lane at gunbc#828 #issuecomment-4362742638 (2026-05-02); R3 critical-path locked
roadmap row: ROADMAP.md "Substrate carrier port program (scopes the substrate dissolution path half of the honesty pass)" — Lane E-P (acceptance: cost/complexity lenses consume the side table + v2-oracle-vs-v3 cementing)
authority docs:
  - docs/r3-structure.md:37 (lane definition; M-L sized; Substrate Mgr ownership; foundational)
  - docs/r3-structure.md:155-165 (table row + critical-path placement)
  - docs/r3-structure.md:163 (critical path: T-E-P-Producer-Broadening → T-Lens-Behavioral-Parity → T-Lens-Application-Surface → T-Workflow-As-Data + T-Lens-Self-Application)
  - docs/r3-structure.md:295,446 (non-Evaluator-gated; parallel-dispatchable)
  - docs/design-cost-lens-sizevar-dimension-wiring.md §391, §463, §504
  - docs/design-complexity-lens-behavioral-completeness.md §272, §282, §342, §598, §673, §684
gates:
  - e_p_per_call_descent_evidence_full_coverage
  - e_p_call_pattern_lookup_authoritative
  - e_p_sub_value_relation_per_call_landed
---

# R3 T-E-P-Producer-Broadening — full `ExprCall.descent_evidence` parity at live call sites

## Context

Lane E-P landed a first slice as
`v3_compiler::dag::per_call_descent_evidence`
(`src/v3/compiler/src/dag.rs:1384`) covering recursive self-call +
arithmetic-descent only. Unproven edges fail closed to
`SubValueRelation::SubValueUnknown`; `TransformNode` stays
unwidened. The remainder of the per-call producer surface — full
parity with v2's `ExprCall.descent_evidence`
(`src/v2/00_core.dag:199`) — has not landed.

Cost and complexity lens behavioral parity (T-Lens-Behavioral-Parity)
gates on this lane: the `v3-lens-capability-register.md` audit
classifies `cost.dag` as PROXY and `complexity.dag` as PROXY because
neither lens consumes the per-call side table at the granularity v2
produces. Ratchet receipts in `dag.rs:1187` (`CallPattern` 🟡
SCAFFOLD) and `dag.rs:1268` (`SubValueRelation` 🟡 SCAFFOLD) name
the dissolution shape: full producer coverage + lens consumption.

T-E-P-Producer-Broadening is **foundational** per `r3-structure.md:37`:
critical-path upstream of the entire R3 lens chain. Non-Evaluator-gated
(parallel-dispatchable per `:295,446`); the worker can dispatch
during R3 wait windows independent of R2-Evaluator close.

## Slice

This is M-L sized per the lane row; deliver across one PR or a
small staged set of PRs as the consumer-cementing surface dictates.
Worker should align to the three named gates structurally.

### Phase 1 — Producer surface broadening (gate `e_p_per_call_descent_evidence_full_coverage`)

1. Catalog every live call site in v3 lowered Dags where v2's
   `ExprCall.descent_evidence` populates a `SubValueRelation` other
   than `SubValueUnknown`. Use v2 oracle traces over the
   verification-corpus programs as the ground truth.
2. For each call-site class outside the current first-slice
   coverage (recursive self-call + arithmetic descent), extend
   `per_call_descent_evidence(dag: &Dag)` to populate the
   corresponding `SubValueRelation` variants. Reference variants
   live at `dag.rs:1275-1294` (full enum).
3. Where a call-site class needs a new `CallPattern` variant beyond
   today's eight (`dag.rs:1195-1225`), the addition follows
   substrate-fact-introduction procedure (`INVARIANTS.md#p1-modeling-faithfulness`
   procedure) — a P1 receipt in the PR body, not silent variant growth.
4. Worker keeps `TransformNode` unwidened. The producer is a
   side-table; widening the node to inline call-pattern facts is
   out of scope and would re-violate the audit's "off-substrate
   side-table" framing.

### Phase 2 — `CallPattern` lookup authority (gate `e_p_call_pattern_lookup_authoritative`)

1. **Authoritative lens-facing surface.** The L-7 single-authority
   substrate query for the cost + complexity lenses is
   `per_call_pattern_at(d: Dag, call_site: NodeId) -> CallPattern?`,
   exposed from `std.computation` per
   `docs/design-cost-lens-sizevar-dimension-wiring.md` §3.2 + §8.4
   and `docs/design-complexity-lens-behavioral-completeness.md`
   §278 (Director-ratified — single-authority per `INVARIANTS.md`
   P2 / L-7). The query wraps the side-table
   `v3_compiler::dag::per_call_descent_evidence` and returns
   `Option<CallPattern>` (`None` for non-recursive call sites,
   `Some(pattern)` for recursive ones). The internal lowering
   helper `lower_call_pattern` at `dag.rs:1318` routes from
   `CallPattern` to `LoweringTarget` for compiler-internal use,
   but it is **not** the lens-facing query surface — the lens
   path is `per_call_pattern_at` per the design docs.
2. **Land the typed query surface** if it doesn't already exist
   on `std.computation`. Worker greps at dispatch — if landed,
   use it; if not landed, this Phase covers landing it
   (gated on Phase 1 broadening). Co-owned with the cost-lens
   producer-consumption gate
   (`per_call_pattern_query_surface_landed` per
   `design-cost-lens-sizevar-dimension-wiring.md` §493 step 3).
3. **Confirm consumers route through `per_call_pattern_at`.**
   Cost lens (`src/v3/lenses/cost.dag`) and complexity lens
   (`src/v3/lenses/complexity.dag`) reach call-pattern facts
   through `per_call_pattern_at`, NOT by re-scanning the Dag,
   NOT through `lower_call_pattern` (which is compiler-internal).
   `induction.dag` witness construction and any other consumers
   route through the same query.
4. **Migrate any parallel paths.** If any consumer re-scans
   (string-match on operator names, arithmetic-shape detection
   in lens code, etc.) or reads `per_call_descent_evidence`
   storage directly, migrate to `per_call_pattern_at`. Per
   `feedback_parallel_representation_debt`, parallel
   call-pattern derivation paths are debt.
5. **Cementing test.** Per-call-pattern v2-oracle equivalence on
   the verification corpus programs (every `ExprCall` site
   produces the same `CallPattern` classification under v2 and
   v3 via `per_call_pattern_at`).

### Phase 3 — `SubValueRelation` consumer wiring (gate `e_p_sub_value_relation_per_call_landed`)

1. Wire `cost.dag` to consume the per-call descent-evidence side
   table for `CostBound` derivation. Design-doc spec:
   `docs/design-cost-lens-sizevar-dimension-wiring.md` §391
   (consumer wiring), §463 (lens-side projection), §504 (cementing
   shape).
2. Wire `complexity.dag` to consume the same side table for
   work/span/asymptotic-class derivation. Design-doc spec:
   `docs/design-complexity-lens-behavioral-completeness.md` §272,
   §282, §342, §598, §673, §684 (per-call-class behavioral
   completeness obligations).
3. Per-lens v2-oracle cementing test: same source program, same
   `CostBound` / asymptotic classification under v2 and v3 lens
   execution. Per-lens test discipline lives under
   T-Tests-As-Data-Completeness's `lens_cementing_test_discipline_complete`
   gate; this lane lands the cementing tests for cost + complexity
   specifically.

## Acceptance

- `e_p_per_call_descent_evidence_full_coverage`: every v2-classified
  `ExprCall.descent_evidence` populates an equivalent
  `CallDescentEvidence` row in v3; `SubValueUnknown` rate matches v2's
  unproven-edge rate exactly (no v3 regressions, no v3 fabrications).
- `e_p_call_pattern_lookup_authoritative`: `per_call_pattern_at`
  is the only route from call site to `CallPattern` for lenses
  and external consumers (the L-7 single-authority lookup gate);
  the compiler-internal helper `lower_call_pattern` (`dag.rs:1318`)
  routes call-site → `LoweringTarget` only inside lowering itself
  and is **not** a consumer-facing entry point. No parallel call-
  pattern derivation in lens or consumer code.
- `e_p_sub_value_relation_per_call_landed`: cost + complexity lens
  consume the side table; per-lens v2-oracle cementing tests green.
- `cargo test --workspace --exclude v2-compiler-tests` green.
- `cargo test -p v2-compiler-tests` green; strict-compile diagnostic
  ratchet at 0.
- `cargo clippy --all-targets -- -D warnings` clean.
- `cargo fmt --all --check` clean.
- ROADMAP "Substrate carrier port program" Lane E-P row updates from
  Partial → Retired (or to a narrower residual if any per-call class
  legitimately defers; the residual row names exactly which classes
  and why).
- `docs/v3-lens-capability-register.md` re-classifies cost +
  complexity lenses from PROXY to (at least) PARTIAL with named
  remaining work, or to BEHAVIORALLY COMPLETE if this lane's
  consumer wiring closes the register's named blockers.

## STOP-AND-ESCALATE

- **A new `CallPattern` variant requires modeling discipline review.**
  Adding a variant beyond the current eight is a substrate-fact
  introduction; STOP and follow `INVARIANTS.md` P1 procedure
  (DAG-ancestor / coproduct-vs-coordinate / primitive-vs-lens-extensible).
  The new variant ALSO inherits the existing `CallPattern` 🟡
  SCAFFOLD classification at `dag.rs:1187` per Practice 4 (Step 4),
  with the same dissolution trigger as `SizeBound`. Do not silently
  grow the enum past current count without P1 receipt AND
  Practice-4 classification confirmation in the PR body.
- **A consumer's existing call-pattern derivation cannot migrate
  cleanly to side-table lookup** (e.g., the consumer needs a
  call-pattern fact for a site the producer cannot yet classify):
  STOP — the producer side and consumer side are coupled and the
  lane's structural completeness is at issue. Surface to R3
  Substrate Manager (#1739).
- **v2-oracle and v3 producer disagree on a verification-corpus
  call site** at cementing time: the disagreement is the bug.
  Track to root cause; do not bridge with a fail-open default.
- **R2-Evaluator landing changes the consumer surface mid-lane:**
  R2-Evaluator is a parallel program (per `r3-structure.md:444-448`
  carve-out, this lane operates outside the R2-Evaluator-gated
  cluster). If R2-Evaluator's landing reshapes lens execution in a
  way that invalidates Phase 3 consumer wiring, surface to Director
  for re-sequencing rather than chasing the moving target.

## Authority audit receipt

1. **Substrate exists?** Partial. `per_call_descent_evidence` lives
   at `dag.rs:1384`; `CallPattern` (8 variants, 🟡 SCAFFOLD) at
   `:1195`; `SubValueRelation` (variants 🟡 SCAFFOLD) at `:1275`;
   `lower_call_pattern` at `:1318`. This lane broadens producer
   coverage and wires consumers; carrier types are already landed.
   No new top-level carrier introduced (variant additions follow
   P1 procedure if needed per Phase 1 step 3).
2. **Existing brief?** None for this lane specifically. The
   originally-named E-P lane sits inside the substrate carrier
   port program (ROADMAP row); T-E-P-Producer-Broadening is the
   2026-05-02 R3 expansion of that lane to full producer parity +
   consumer wiring.
3. **Design-doc match?** `docs/design-cost-lens-sizevar-dimension-wiring.md`
   §391/§463/§504 + `docs/design-complexity-lens-behavioral-completeness.md`
   §272/§282/§342/§598/§673/§684 specify the consumer wiring
   shape exactly. Worker re-reads each cited section before Phase 3
   authoring; if the design-doc recommendation has shifted post-cite
   (commit drift), re-frame Phase 3 to match.
4. **Citations live?** `dag.rs:1187, 1195, 1268, 1275, 1296, 1318,
   1384` and `00_core.dag:199` verified at HEAD via wait-window
   grep (2026-05-05). Worker re-verifies at dispatch time.
5. **Carrier dissolves the bridge?** Yes — the dissolution sentence
   in the audit is "cost/complexity lenses consume the side table,
   plus a v2-oracle-vs-v3 cementing test." Phase 1 produces the
   side table to full parity; Phase 2 ensures the lookup is the
   authoritative routing; Phase 3 wires the lens consumers. The
   PROXY classification on cost/complexity lenses lifts as the
   side-table consumption replaces today's reduced-shape derivation.

## Provenance

Drafted during R3 host-block wait window (2026-05-05) per parent
session #828 instruction (parallel-author posture for non-Evaluator-gated
lanes); replaces the earlier "Decision 2 follow-on" framing per
Director correction at #issuecomment-4377501646 (2026-05-05).
The "Decision 2" label was Director-side context-loss; lane is
**standalone foundational substrate-completion work** per
`r3-structure.md:37`. Ratification pending host restoration and
parent dispatch slot allocation.
