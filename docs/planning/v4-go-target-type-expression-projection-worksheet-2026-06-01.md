# v4 Go TargetTypeExpressionProjection Worksheet — language row extension

> **Status:** **WORKSHEET APPROVED — READY-FOR-WORKER-DISPATCH** — Modeling DFS Arbiter §8 sign-off 2026-06-01 (`proud-fox-405`) — SG-2 **language-row extension**, not new carrier.
> **Date:** 2026-06-01
> **Dispatch anchor:** SG-2 analog — `docs/planning/v4-sg2-type-expression-projection-worksheet-2026-05-30.md` (APPROVED); #4124 landed; `docs/planning/v4-predicate-dependency-graph-2026-06-01-eod.md` §11.2 TargetTypeExpression Manager lane.
> **Canonical home:** `src/v4/std/target_model.dag` (`TargetTypeExpressionProjection` — **do not redefine**).

---

## Mechanical dispatch rule

> **No Go TargetTypeExpressionProjection worker may land until:**
> 1. Modeling DFS Arbiter approves this **language-row extension brief** (not a second SG-2 worksheet), **and**
> 2. Shared SG-2 substrate + Rust rows are on main (#4124), **and**
> 3. Go `TargetAtomRealization.type_form` rows exist for atoms referenced in instantiation tests.

This is **not** per-lane reinvention of SG-2 — it names which `std/` generic carriers need Go `instantiation_form` rows and the falsification fixture for Go emit.

---

## §10.0-adapted worksheet

```text
SG class:               GO-SG-2-EXTENSION (TargetTypeExpressionProjection rows in go.dag)
Shared authority:       v4.std.target_model::TargetTypeExpressionProjection (SG-2 APPROVED)
Representative failure:  Emitted Go drops type arguments on Outcome<T>, Witness<C>, Refined<B>, etc.
                         — go build errors on generic carriers (E0107-analog at Go typecheck).
Immediate local patch:   Name-keyed generic-arity table in Go emit template.
Why forbidden:           P3 name-keyed table; parallel authority vs std/diagnostic.dag facts.
DFS path:
  std/ authority:
    - Instantiation connective src/v4/std/node.dag
    - Generic carriers: diagnostic Outcome<T>, witness Witness<C>, refinement Refined<B>, ...
  extdeps/language authority:
    - go.dag: no TargetTypeExpressionProjection rows (2026-06-01 spot-check)
    - rust.dag: rows landed #4124 — reference for instantiation_form shape only
  compiler stage:
    - 06_translate Go type-expression path (v4) + v2 05_emit_go where still active
Deepest unsound boundary:
  Instantiation declared in std/ but Go has no per-carrier instantiation_form projection rows.
Systemic fix:
  Add go_target_type_expr_projection_* rows for each SG-2 catalog member the Go emitter consumes;
  wire target_model edge; refactor Go type emit to read PositionalEdges / instantiation_form.
Non-goals:
  - New SG-2 carrier definition (escalate to Arbiter + TargetTypeExpression Manager).
  - Patching individual weather Go files without row authority.
Falsification probe (shared SG-2 contract):
  Introduce NEW std/ generic carrier FooBar<T,U> in a hermetic test module; use in Go-emit
  position; verify emitted Go shows correct arity WITHOUT adding FooBar to a name-keyed table.
Metric allowed only as secondary:
  Go generic/type errors on emitted subset — evidence only.
```

---

## §4 Go instantiation surface (sketch)

Go generic syntax constraints for `instantiation_form` rows (impl worker + Arbiter ratify):

| std carrier | Go surface (sketch) | Notes |
|---|---|---|
| `Outcome<T>` | struct type param on field or alias | May require type alias until Go 1.18+ generics in emitted slice |
| `Witness<C>` | interface or concrete wrapper per row | Match existing Go emit for witness tests |
| `Refined<B>` | embed base + constraint marker | Do not invent parallel `RefinedGo` type |

**Escalation trigger:** If Go emitter cannot express a std carrier without a 6th L1 behavior or non-`Bind` effect, **STOP** — C1 escalation per `INVARIANTS.md` / `src/v4/TASKS.md` coordination discipline.

---

## §8 Modeling DFS Arbiter approval checklist — CLOSED 2026-06-01

- [x] Classified as **extension** to SG-2 approved carrier (single authority preserved)
- [x] No duplicate `TargetTypeExpressionProjection` type definition in `go.dag`
- [x] C1 escalation path named if Go cannot express std carrier without new behavior
- [x] **READY-FOR-WORKER-DISPATCH** (`proud-fox-405`)

---

## Related artifacts

- `docs/planning/v4-sg2-type-expression-projection-worksheet-2026-05-30.md`
- PR **#4124** — SG-2 Rust implementation reference
- `docs/planning/v4-go-target-atom-realization-worksheet-2026-06-01.md` — prerequisite for atom-typed generic args
