# v4 SG-2 Worksheet — Generic carrier / `TargetTypeExpressionProjection`

> **Status:** WORKSHEET APPROVED — Modeling DFS Manager §10.0 sign-off 2026-05-30 (`cool-ibex-692`; manager pass completes §11.4 item 2).
> **Date:** 2026-05-30
> **Dispatch anchor:** `docs/audit/v4-rustc-error-catalog-2026-05-29.md` SG-2 row (1219 E0107 + 747 E0282); `docs/planning/v4-correctness-ladder-2026-05-30.md` §10.2.
> **Canonical home:** `src/v4/std/target_model.dag` (`v4.std.target_model`) — ratified `docs/design-target-realization-canonical-home.md` §1 Option A.
> **Dispatch order:** **First** among SG-1/2/5-6 — SG-1 `TargetAtomRealization.type_form` MUST consume this substrate; dispatching SG-1 before SG-2 is forbidden.

---

## Mechanical dispatch rule

> **No SG-2 implementation worker may land until this worksheet is complete and Modeling DFS Manager–approved.**

Acceptance is the falsification probe below, not E0107/E0282 count reduction.

---

## §10.0-adapted worksheet

```text
SG class: SG-2
Representative emitted failure:
  pub type FileReadResult = Rc<Outcome>;
  // Should be Rc<Outcome<...>> with proper type arguments.
Immediate local patch:
  Maintain a name-keyed list in the emitter:
    if type_name in {"Outcome", "Witness", "Refined", "TestClaimRun"}:
      require one (or N) type args
Why that patch is forbidden:
  - Doesn't scale to new generic carriers (every new one requires an emitter edit).
  - Creates parallel authority: the name-keyed list duplicates facts already
    declared in std/diagnostic.dag, std/witness.dag, std/refinement.dag.
  - Calcifies the name-keyed table anti-pattern INVARIANTS P3 forbids.
DFS path:
  std/ authority:
    - Outcome<T> at src/v4/std/diagnostic.dag
    - Witness<C> at src/v4/std/witness.dag
    - Refined<B> at src/v4/std/refinement.dag
    - Instantiation connective at src/v4/std/node.dag
    - Instantiation lowering uses PositionalEdges (type arguments present as edges)
  extdeps/language authority:
    - extdeps/languages/rust.dag: no TargetTypeExpression / Instantiation projection
  compiler stage consuming it:
    - grep "Instantiation" in src/v4/compiler/ → no hits (2026-05-30 spot-check)
    - 06_translate type-expression path does not consume Instantiation today
  existing scaffold/dissolution notes:
    - none; foundational missing-substrate gap
Deepest unsound boundary:
  Instantiation is declared in std/node.dag but has no per-target realization fact.
  Emit paths derive generic-application syntax independently; Rust drops type args.
Systemic fix:
  TargetTypeExpressionProjection fact-bundle in v4.std.target_model (carrier once);
  per-language projection rows in extdeps/languages/<lang>.dag.
  Refactor 06_translate type-emission to consume per-connective forms including
  instantiation_form reading Instantiation.children (PositionalEdges).
Non-goals:
  - Name-keyed special cases for Outcome / Witness / Refined / TestClaimRun.
  - Hardcoding "emit one generic arg" for known carriers.
  - Patching individual emitted Rust files.
Falsification probe:
  Introduce a NEW generic carrier in std/ (e.g. type FooBar<T, U> { ... }).
  Use it in a position. Emit. Verify emitted Rust has Rc<FooBar<X, Y>> with
  correct arity WITHOUT adding any emitter branch for FooBar.
Metric allowed only as secondary:
  1219 E0107 + 747 E0282 are evidence; NOT acceptance.
```

---

## Tightened worker brief (dispatch downstream)

```text
Implement TargetTypeExpressionProjection for Rust.

Canonical carrier home:
  Define TargetTypeExpressionProjection and supporting target-side vocabulary
  ONCE in src/v4/std/target_model.dag (module v4.std.target_model).
  Per-language rows in extdeps/languages/rust.dag only.

Consumers:
  Refactor src/v4/compiler/06_translate.dag type-emission to consume the
  projection per connective (including Instantiation via instantiation_form).
  No name-keyed carrier lists.

Bidirectional readability (§10.6):
  instantiation_form / other connective forms must support emission AND ingestion
  from the same row.

Falsification:
  New generic carrier FooBar<T,U> in std/ emits with correct arity without
  emitter edits for FooBar.

Non-goals:
  - SG-1 atom realization (separate brief; blocked until this PR lands).
  - Error-count reduction as acceptance.
```

---

## §5 Landing order (implementation — not worksheet-only PR)

```text
1. TargetTypeExpressionProjection carrier + TargetTypeExpression wire authority in v4.std.target_model.
2. Rust projection row on rust TargetModel bundle (extdeps/languages/rust.dag).
3. 06_translate type-expression projection consumes per-connective forms (including instantiation_form).
4. Manual falsification claims: sg2_type_expression_projection.dag (Instantiation → Rc<FooBar<X,Y>>; no FooBar emitter branch).
5. MVP1 ProjectionAbsent shim remains until every extdeps TargetModel carries the projection edge (🟡 sg-2-mvp1-projection-absent-shim).
```

**Lane split:** Target Realization owns steps 1–3; Runtime/TestClaim owns step 4; extdeps rollout owns step 5 dissolution.

---

## §6 Downstream worker brief (dispatch after §8)

```text
Implement SG-2 per approved worksheet §10.0 on main.

MUST:
  - Author TargetTypeExpressionProjection in v4.std.target_model; Rust row in rust.dag;
    refactor 06_translate type-emission to consume projection per connective (including instantiation_form).
  - Land manual falsification probe: new generic carrier arity without name-keyed emitter branches
    (sg2_type_expression_projection.dag CompilesClaim + EqualsClaim receipts).
  - Bidirectional readability (§10.6): instantiation_form delimiters shared for emit + wire decode.
  - Include variant-shape histogram for new coproduct arms (v4-substrate-pr-review-gate §3).

MUST NOT:
  - Name-keyed special cases for Outcome / Witness / Refined / TestClaimRun / FooBar.
  - Hardcoding "emit one generic arg" for known carriers.
  - Use SG-2 rustc error-count reduction (E0107 / E0282) as acceptance.
  - Dissolve 🟡 sg-2-mvp1-projection-absent-shim before every extdeps TargetModel carries the row.

Escalate to Modeling DFS:
  - Second target-type-expression vocabulary (violates SG-1 / SG-2 single-authority).
  - M6 identical-payload coproduct arms on new carriers without substrate-pr-review-gate ack.
```

---

## §7 Non-goals

- SG-1 `TargetAtomRealization` (separate brief; blocked until SG-2 PR lands)
- SG-RC-LAYERING / SG-1b / emit-binary M1 probe as primary acceptance
- `ci.dag` / ci.yml / shell gate migration
- Dissolving 🟡 sg-2-mvp1-projection-absent-shim in the same PR as substrate landing

---

## §8 Manager approval checklist (`cool-ibex-692`) — CLOSED 2026-05-30

- [x] Single-authority fact: `TargetTypeExpressionProjection` in `v4.std.target_model`
- [x] Spot-fix forbidden: name-keyed Outcome/Witness/Refined tables
- [x] Falsification probe accepted
- [x] Dispatch order: SG-2 before SG-1
- [x] Worker dispatch — **authorized** (`fierce-deer-774`; substrate #3962 on `main` 2026-05-30)

## Related artifacts

- `docs/planning/v4-correctness-ladder-2026-05-30.md` §10.2
- `docs/design-target-realization-canonical-home.md`
- `docs/planning/v4-modeling-dfs-manager-pass-2026-05-30.md`
