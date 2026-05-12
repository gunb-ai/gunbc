# R3 Wave-1 S6 — Slice 7 affected-set implementation (#103)

**Owner**: Wave-1 Substrate worker
**Authored by**: warm-wolf-698 (R3 Substrate Mgr)
**Authoring date**: 2026-05-12

---

## §0. Status — DISPATCH GATES ON cool-crab-565 PR #2766 canvas-status

**Do not start authoring until prerequisite clears**:

1. **PR #2766 (cool-crab-565 / Slice 7 affected-set canvas) Director-ratified** — the canvas establishes the affected-set selection contract (input: `CIWorkflowDag` + diff; output: subset of gate-nodes to execute). Worker reads ratified canvas as substrate authority before authoring implementation.

If PR #2766 hasn't ratified at spawn time, hold; ping warm-wolf-698 for status. Brief may need revision per ratified contract details.

## §1. Scope

Implement the affected-set selection logic referenced by Slice 7 canvas (PR #2766). Closes gate #103 `ci_uses_affected_set_selection`.

### Phase A — Read the ratified canvas

`docs/design-ci-workflow-substrate-shape-2026-05-12.md` (Slice 7 canvas, post-#2766 merge) is the substrate authority. Inventory:
- Input domain: `CIWorkflowDag` + diff-against-base shape
- Output: gate-id subset / matrix specification
- Verifier ratchet contract (per PM dispatch msg_a63f5e7e: "Brief should reference upstream gate-state for verifier ratchet")

### Phase B — Implement the selection function

Author the selection logic in the appropriate gunbc/ci namespace module. The function shape is likely:
```
func select_affected_gates(workflow_dag: CIWorkflowDag, diff: <diff-shape>) -> List<CIGateId>
```
(exact signature per canvas ratification).

Implementation MUST:
- Honor single-authority derivation: every selection decision derives from `CIWorkflowDag` + diff input
- Be testable structurally (per TESTING.md band-A)
- Handle edge cases per canvas (empty diff, diff-touches-everything, isolated-gate)

### Phase C — Verifier ratchet integration

Wire the affected-set output into the verifier ratchet per upstream gate-state contract. The ratchet shape is whatever PR #2766 canvas ratifies; brief cannot pre-author this without that ratification.

## §2. STOP conditions

1. **PR #2766 canvas not ratified** — already noted in §0. Hold.
2. **Single-authority violation in selection logic** — if any selection rule requires reading state outside `CIWorkflowDag` + diff (e.g., environment vars, external service calls, hand-written config), **STOP** — P2/P3 violation by inventing inputs.
3. **Verifier ratchet contract ambiguity** — if the ratchet contract isn't unambiguously specified in the ratified canvas, **STOP** and surface — don't infer.

## §3. Verification

- `cargo test --workspace`
- Hermetic test cases (per TESTING.md): N input `CIWorkflowDag`s × M diff shapes; assert selected gate-id sets match canvas-specified expectations
- Verifier ratchet shows monotone reduction (or holds at zero) on expected scenarios

## §4. PR body framing

- Cite gate #103 closure
- Cite PR #2766 ratified canvas as substrate authority
- Cite verifier ratchet contract (specific paragraph in canvas)
- Inline 2-3 worked examples of selection behavior

## §5. Out of scope

- Compiled-binary main() integration (separate scope; S7 BinaryShim brief covers the projection-arm)
- ci.yml hand-authority dissolution (#98 / Slice 5)
- Affected-set heuristics not specified by canvas (no extensions per `feedback_assertion_vector_does_not_change_scope`)

## §6. Reference

- PR #2766 (cool-crab-565 Slice 7 canvas) — substrate authority
- `docs/r3-remaining-work-dependency-graph.md:125` — gate-row metadata
- TESTING.md band-A — structural test discipline
