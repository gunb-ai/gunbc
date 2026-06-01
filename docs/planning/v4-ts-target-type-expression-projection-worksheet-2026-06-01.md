# v4 TS TargetTypeExpressionProjection Worksheet (per-language row)

> **Status:** **WORKSHEET APPROVED — READY-FOR-WORKER-DISPATCH** — Modeling DFS Arbiter §8 sign-off 2026-06-01 (`proud-fox-405`). **Dispatch before** TS atom-realization.  
> **Lane:** ALPHA/PREVIEW — NOT release-minimum.  
> **Shared carrier authority:** `src/v4/std/target_model.dag` — `TargetTypeExpressionProjection` (SG-2 landed; Rust row on main per #4124). **Do NOT redefine carrier.**  
> **Dispatch order:** **Before** TS TargetAtomRealization rows (`type_form` dependency).  
> **Dispatch anchor:** `docs/planning/v4-sg2-type-expression-projection-worksheet-2026-05-30.md`.

---

## Mechanical dispatch rule

> **No TS TargetTypeExpressionProjection implementation worker may land until this worksheet is Modeling DFS Arbiter–approved.**

Acceptance is falsification via **new generic carrier in std/** emitting through TS with correct arity — not E0107-style error-count reduction.

---

## §10.0-adapted worksheet

```text
SG class:               TS-TYPE-EXPRESSION-PROJECTION (per-language row on landed SG-2 carrier)
Representative failure:  Generic instantiation in emit drops type arguments for Outcome<T> /
                         Witness<C> when target is TypeScript — parallel name-keyed emitter table.
Immediate local patch:
  if (carrier_name == "Outcome") emit "Outcome<any>" in TS translate.
Why that patch is forbidden:
  Name-keyed tables duplicate std/ carrier facts; forbidden by SG-2 worksheet + INVARIANTS P3.
DFS path:
  std/ authority:
    - TargetTypeExpressionProjection, Instantiation connective, PositionalEdges
      at src/v4/std/target_model.dag + src/v4/std/node.dag
  extdeps/language authority:
    - src/v4/extdeps/languages/typescript.dag — ts_type_expression_projection() row on TargetModel
  compiler stage:
    - src/v4/compiler/06_translate.dag — type_expression_projection_from_bundle for TS target
  existing notes:
    - MVP-1 ProjectionAbsent shim until every extdeps TargetModel carries projection edge
      (🟡 sg-2-mvp1-projection-absent-shim) — TS row dissolution removes TS from shim set
Deepest unsound boundary:
  Instantiation connective has no TS syntax realization; emit invents angle-bracket forms ad hoc.
Systemic fix:
  TargetTypeExpressionProjection row on typescript.dag TargetModel bundle:
    - atom_form:           ident / keyword primitives (number, string, boolean, bigint, void)
    - conj_form:           object type { … } and/or intersection (&) per Arbiter choice
    - disj_form:           union (|) — TS primary sum-type surface
    - arrow_form:          (params) => return
    - cardinality_form:    tuple [T, U] or Arbiter-ratified stand-in for Finite/N syntax
    - instantiation_form:  generic apply Foo<Bar, Baz> (angle brackets)
  Wire bundle edge per rust.dag rust_type_expression_projection() pattern.
Non-goals:
  - Namespace / module `import type` path modeling (L1 packaging — out of L0 scope).
  - Conditional types (`T extends U ? X : Y`) — not required for L0 falsification probe.
  - Decorator / branded-type full surface.
  - Name-keyed Outcome/Witness/Refined special cases in emitter.
Falsification probe:
  Introduce NEW generic carrier FooBar<T, U> in std/; use in emit position; verify emitted TS
  contains FooBar<X, Y> with correct arity WITHOUT adding FooBar branch to 06_translate.
  Manual receipt: extend or sibling sg2 pattern —
  src/v4/test/claim/manual/sg2_type_expression_projection.dag (TS slice) OR
  typescript-specific manual claim file ratified by Arbiter.
Metric allowed only as secondary:
  tsc error count on generic-heavy snippets.
```

---

## TS connective → syntax mapping (proposed — Arbiter ratifies)

| Connective | TS surface (ECMA / Handbook) | Notes |
|------------|------------------------------|-------|
| Atom | `number`, `string`, `boolean`, `bigint`, ident | Keyword + identifier tokens from `ts_wave1_lex` |
| Conj | `{ … }` record / `A & B` intersection | Prefer record for MVP-1; intersection if emit already uses |
| Disj | `A \| B` | Primary sum-type |
| Arrow | `(a: T) => U` | Typed parameters required for tsc strict |
| Instantiation | `Foo<Bar, Baz>` | Angle brackets; no Rust `::<>` |
| Cardinality | `[T, U]` tuple or named Arbiter stand-in | TS has no Rust-style `Finite(n)` surface |

---

## Landing order (implementation — post-approval)

```text
1. Ratify TS row shape on Modeling DFS Arbiter calendar (shared-fact review — proud-fox-405).
2. Author ts_type_expression_projection() in typescript.dag; attach to ts_*_target_model bundle.
3. 06_translate consumes projection for ShapeATarget::TypeScript (or unified target dispatch).
4. Manual falsification TestClaim (TS slice of SG-2 probe).
5. Remove TS from MVP-1 ProjectionAbsent shim when row present on bundle.
```

**Lane split:** Target Realization / extdeps worker owns 1–2; Compiler Spine owns 3; Runtime/TestClaim owns 4.

---

## §8 Modeling DFS Arbiter approval checklist — CLOSED 2026-06-01

- [x] Single-authority fact: carrier in `std/target_model` only; row in `typescript.dag` only
- [x] Dispatch **before** TS atom-realization worker (operator hold enforced)
- [x] SG-2 extension only — no duplicate carrier
- [x] MVP-1 shim dissolution criterion named
- [x] **READY-FOR-WORKER-DISPATCH** (`proud-fox-405`)

## Related artifacts

- `docs/planning/v4-sg2-type-expression-projection-worksheet-2026-05-30.md`
- `src/v4/extdeps/languages/rust.dag` — `rust_type_expression_projection()`
- `src/v4/test/claim/manual/mvp1_typescript_add_translate.dag` (downstream consumer)
- `docs/planning/v4-ts-target-atom-realization-worksheet-2026-06-01.md`
