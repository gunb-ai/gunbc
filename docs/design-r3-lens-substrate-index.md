# R3 Lens Substrate — Design Index

> Part of: [`docs/r3-structure.md`](r3-structure.md), [`../INVARIANTS.md`](../INVARIANTS.md)
>
> **Purpose:** index for the 5 R3 lens-substrate design docs authored 2026-05-02 to satisfy the user directive *"all designs upfront, implementation sketches if needed, minimize escalations"*. Each doc resolves all its design questions in-doc; this index makes the 5 navigable as one substrate-design surface and documents cross-doc compatibility.

## The 5 design docs

| Doc | Lane | Foundational | Status |
|---|---|---|---|
| [design-lens-application-surface.md](design-lens-application-surface.md) | T-Lens-Application-Surface | yes — defines `apply_lens(lens, section, config)` substrate consumed by 4 worked examples | RESOLVED |
| [design-tests-as-data-completeness.md](design-tests-as-data-completeness.md) | T-Tests-As-Data-Completeness | yes — defines `QuantifiedTestClaim`/`ProgramGenerator` consumed by cementing tests in other lenses | RESOLVED |
| [design-complexity-lens-behavioral-completeness.md](design-complexity-lens-behavioral-completeness.md) | T-Lens-Behavioral-Parity slice 1 | no — refines complexity lens to BEHAVIORALLY COMPLETE | RESOLVED |
| [design-cost-lens-sizevar-dimension-wiring.md](design-cost-lens-sizevar-dimension-wiring.md) | T-Lens-Behavioral-Parity slice 2 | no — refines cost lens to BEHAVIORALLY COMPLETE | RESOLVED |
| [design-effect-enumeration-resource-threading.md](design-effect-enumeration-resource-threading.md) | T-Lens-Behavioral-Parity slice 4 | no — refines effect_enumeration to BEHAVIORALLY COMPLETE | RESOLVED |

(Slice 3, parallelism Stage 2e walk port from Rust to `.dag`, has prior design authority at [`docs/design-db20-lane2-stage2e-parallelism-lens.md`](design-db20-lane2-stage2e-parallelism-lens.md) — implementation is mechanical port per existing design.)

## Cross-doc compatibility

This index documents the cross-doc edges that the coherence audit (2026-05-02) verified.

### Substrate authority single-points (P2)

| Authority | Owning doc | Consumer docs |
|---|---|---|
| `SectionRef` / `SectionedLensApplication` / `ApplicationConfig` | lens-application-surface §2 | (used by future user-authored lens applications) |
| `QuantifiedTestClaim` / `ProgramGenerator` / `ForAll`/`Exists` quantifiers | tests-as-data §2.2 | cementing tests in cost/complexity/effect-enumeration |
| `AsymptoticClass` lattice | complexity-lens §1.4 | lens-application's `ComplexityBudget` (= `AsymptoticClass`) |
| `SymbolicCost` algebra (DB-7 carrier; semiring fix in cost-lens §4) | algebra.dag (DB-7) — not authored here | complexity-lens (consumes for symbolic-cost dimension); cost-lens (extends with semiring inhabitance + product-zero fix) |
| `SizeExpr` value-semantics carrier | cost-lens §1.2 | (replaces `SizeVariable`; cost lens internal) |
| `per_call_pattern_at(d, call_site) → CallPattern?` typed query | cost-lens §3.2 + §8.4 | complexity-lens §2 (single shared producer query — no parallel authority) |
| Resource-threaded callable signatures | effect-enumeration §2.4 | cost-lens §3.3 (consumption is signature-shape-agnostic) |
| `Operation` (replaces `OperationEffect` post-migration) | effect-enumeration §4 | DB-18 `WorkflowEffect.LinearEffect.ops` (element-type refinement) |
| `LensCapabilityRegister` (`.dag` declaration replacing markdown) | tests-as-data §8.3 | all 4 lens design docs (closure-gate "row → COMPLETE" steps depend on this migration) |

### Cementing-test format

All v2-oracle cementing tests use the same DB-15 `DifferentialEquals` predicate inside a `QuantifiedTestClaim` (per tests-as-data §2.2). Complexity-lens §4, cost-lens §5, effect-enumeration §5 all instantiate this shape. Different shapes would be parallel-test-infrastructure debt; the index confirms they are unified.

### Cross-cutting invariants (held by all 5 docs)

- **C-8 fail-closed**: every detected violation is a Diagnostic; no Warning, no Silent.
- **P5 progress is dissolution**: no bridges, named dissolution triggers for any scaffold.
- **P2 boundary discipline**: each substrate fact has one declared authority (cataloged above).
- **`feedback_no_annotations`**: every "user opts in" is a structural `.dag` declaration, not an annotation.
- **`feedback_no_metadata_markers`**: no `__is_X` strings; structural carriers throughout.
- **`feedback_lenses_not_passes`**: zero heuristics across all 5 docs.
- **`feedback_state_space_vs_behavioral_invariants`**: illegal states unrepresentable at the type level.
- **`feedback_audit_adjacent_authority_first`**: each doc cites pre-existing authority (DB-3, DB-7, DB-15, DB-18, DB-20, design-emission-model.md, lens-library-design.md) before authoring new substrate.

## Lane dispatch order

Per cascade gates in [`docs/r3-structure.md`](r3-structure.md):

```
R2-Evaluator landed (precondition for all 12 Evaluator-gated lanes)
                                ▼
                  T-E-P-Producer-Broadening
                  (foundational; broadens per_call_descent_evidence)
                                │
                                ▼
              ┌─────────────────────────────────┐
              │       T-Lens-Behavioral-Parity   │
              │  ┌────────────┐ ┌────────────┐   │
              │  │  slice 1   │ │  slice 2   │   │
              │  │ complexity │ │    cost    │   │
              │  └────────────┘ └────────────┘   │
              │  ┌────────────┐ ┌────────────┐   │
              │  │  slice 3   │ │  slice 4   │   │
              │  │parallelism │ │  effect_   │   │
              │  │   port     │ │ enumeration│   │
              │  └────────────┘ └────────────┘   │
              │  (4 slices parallel-dispatch     │
              │   post-T-E-P-Producer-Broadening)│
              └──────────────┬──────────────────┘
                             │ all 4 slices COMPLETE
                             ▼
                  T-Lens-Application-Surface
                  (cascade-gated on T-Lens-Behavioral-Parity COMPLETE)
                                │
                                ▼
              T-Tests-As-Data-Completeness
              (parallel to other lanes; step 5 register migration
               unblocks closure-gate "row → COMPLETE" steps in the
               4 lens design docs)
```

**Dispatch readiness**:
- T-E-P-Producer-Broadening: dispatchable post-R2-Evaluator
- T-Lens-Behavioral-Parity slices 1-4: dispatchable post-T-E-P-Producer-Broadening
- T-Lens-Application-Surface: dispatchable post-T-Lens-Behavioral-Parity COMPLETE (all 4 slices)
- T-Tests-As-Data-Completeness: dispatchable post-R2-Evaluator (parallel to T-E-P-Producer-Broadening); step 5 specifically must land before lens-row "→ COMPLETE" closures

## Author-time invariants

Per the user directive *"minimize escalations"*: no design questions remain open in any of the 5 docs. Every §8 (or equivalent) section in each doc is titled "Resolved design questions" rather than "Open design questions"; each question carries a stated resolution + reasoning.

If implementation surfaces a substrate question not anticipated in design, the standing protocol applies (worker STOP+PING; PM/Director adjudicates against existing design authority before resuming). This index does not pre-authorize design changes; it documents the resolved-as-of-2026-05-02 state.

---

**This document is an index, not a substrate authority.** It points at the 5 design docs and the cross-doc edges; the substrate authority lives in those docs (which in turn cite their parent authority — DB-3, DB-7, DB-15, DB-18, etc.). Adding a sixth design doc to this set updates this index; modifying any of the 5 referenced docs does not require an index update unless the cross-doc compatibility table changes.
