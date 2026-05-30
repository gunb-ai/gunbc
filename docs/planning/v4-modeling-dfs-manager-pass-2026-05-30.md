# Modeling DFS Manager pass — §11.4 item 2

**Manager session:** `cool-ibex-692` (successor to `proud-pike-680` on same role-node).
**Cites:** `docs/planning/v4-correctness-ladder-2026-05-30.md` (PR #3938) §10.0, §10.1–§10.3, §11.1, §11.4 item 2.
**Authority:** Approve §10.0 worksheets and single-authority facts before any SG-class worker touches code. No implementation in this pass.

---

## Outcome

§11.4 item 2 is **complete** for the rustc SG program (SG-1, SG-2, SG-5, SG-6). Standalone approved worksheets:

| SG class | Worksheet | Single-authority fact / gate |
| -------- | --------- | ---------------------------- |
| SG-2 | `docs/planning/v4-sg2-type-expression-projection-worksheet-2026-05-30.md` | `TargetTypeExpressionProjection` |
| SG-1 | `docs/planning/v4-sg1-target-atom-realization-worksheet-2026-05-30.md` | `TargetAtomRealization` |
| SG-5 | `docs/planning/v4-sg5-sg6-collection-bounded-lattice-worksheet-2026-05-30.md` (item a) | `TargetCollectionRealization` |
| SG-6 | same worksheet (item b) | BoundedLattice completeness gate in `04_infer` |

**Dispatch order (mechanical):**

```text
1. SG-2 worker  →  Target Realization  (substrate + rust row + 06_translate consumer)
2. SG-1 worker  →  Target Realization  (after SG-2 PR merged; type_form dependency)
3. SG-5 worker  →  Target Realization  (may start after SG-2 carrier vocabulary exists)
4. SG-6 worker  →  Compiler Spine      (infer gate; parallel with 3 when brief ready)
```

SG-CANDIDATE-1 (`symbol` / `symbols` sugar) remains **Compiler Spine** lane per §11.3 §10.5 — not part of this pass; no SG-class worker gate.

---

## Inherited from `proud-pike-680` (already approved — not re-opened)

| Artifact | Status |
| -------- | ------ |
| `docs/planning/v4-sg7-ci-offset-complexity-worksheet-2026-05-30.md` | APPROVED |
| `docs/planning/v4-upsert-t-substrate-worksheet-2026-05-30.md` | APPROVED; Phase 1.5 workers blocked on 1.4 |
| `docs/planning/v4-ci-schema-worksheet-2026-05-30.md` | APPROVED; workers blocked on 1.4 |
| `docs/design-target-realization-canonical-home.md` | Option A (`target_model.dag`); scaffold COEXIST |
| Leaf-model claim carrier shape | Spec in `v4-leaf-model-verification-2026-05-30.md` §5 — **DFS gate still open** for leaf verification workers |

---

## Spot-checks (tree at manager pass, 2026-05-30)

| Claim | Verdict |
| ----- | ------- |
| No `TargetAtomRealization` / `TargetTypeExpressionProjection` / `TargetCollectionRealization` in `src/v4/` | Confirmed |
| `Instantiation` not referenced in `src/v4/compiler/` | Confirmed |
| `loop_bound_edge` gated at `src/v4/std/node.dag:84-85` | Confirmed |
| Canonical home still `target_model.dag` only (no realization carriers yet) | Confirmed |

---

## What workers may do now

- **Target Realization Manager** may receive briefs for SG-2, then SG-1, then SG-5 — each citing the approved worksheet + `design-target-realization-canonical-home.md`.
- **Compiler Spine Manager** may receive SG-6 brief after coordination on `04_infer` touch surface.
- **Forbidden:** dispatch framed as "fix SG-1 errors" / M1 histogram chasing / name-keyed emitter tables.

---

## Unblocks downstream §11.4 items

| Item | Manager | Status after this pass |
| ---- | ------- | ------------------------ |
| 3 | Ladder/Fixture | May proceed (fixture × rung gates) |
| 5–6 | Target Realization workers | **Gate cleared** for SG-2 then SG-1 briefs |
| 7 | Ladder/Fixture worker | Still requires fixture manager pass |

---

## Related

- `docs/planning/v4-close-receipt-manager-pass-2026-05-30.md` — disposition vocabulary (item 1)
- `docs/planning/v4-done-predicate-tracker-2026-05-30.md` — predicates 2–3 cite SG worksheets
