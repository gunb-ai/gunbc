# T-Substrate sub-lane 3 — parametric algebra attachment for `Dimension<Carrier>` `(M, R2 substrate)`

> **Director ad-hoc dispatch.** R2 T-Substrate sub-lane 3 per
> [`docs/r2-structure.md`](../r2-structure.md) §"Goal 3" item 3.
> Reports to Director (`zesty-bear-812`).
>
> **Authority disclaimer:** ROADMAP `§"Post-R1 Program — Grounding
> Completeness"` tags this dependency `DB-18 parametric algebra
> attachment`, but `docs/db-history/db-18.md` scopes DB-18 to
> workflow-effect carrier + Rust reflection (Part 2 shipped) +
> Go-accessor follow-up (Part 3) — **not parametric algebra
> attachment**. R2 acceptance is defined independently of the DB-tag:
> `Dimension<Unit>` phantom-parameter arithmetic compiles with
> unit-mismatch errors. Worker should not be blocked by the
> ROADMAP↔db-history mismatch; coordinate with Director only if a
> rename or new DB number becomes load-bearing during execution.

## Read first

- **[`docs/r2-structure.md`](../r2-structure.md) §"Goal 3" item 3** — sub-lane scoping. R2 acceptance is `Dimension<Unit>` arithmetic compiles with unit-mismatch errors, *independent of the DB-tag the substrate ends up carrying*.
- **[`src/v3/std/dimensions.dag:61`](../../src/v3/std/dimensions.dag)** — current `Dimension<Carrier>`: terminal at the structural-witness scope (compose / identity / break_diagnostic). **One-parameter proof-dimension framework for behavioral analysis** — NOT a typed value wrapper. DB-3 shipped this for `SymbolicCost` analysis.
- **[`docs/db-history/db-3.md:1-9`](../db-history/db-3.md)** — DB-3 (user-declared dimensions) shipped scope: SymbolicCost coproduct + Dimension<Carrier> shape + analyze_symbolic_cost_dimension + cost lens. **Missing for `Dimension<Unit>`:** Class 5 `data` lowering (top-level `data symbolic_cost_dimension: Dimension<SymbolicCost> = {...}` lowers as `Unparsed`).
- **[`dsl/extdeps/languages/rust/primitives.dag:54-100`](../../dsl/extdeps/languages/rust/primitives.dag)** — current algebra tags (IntegerAlgebra / NonIntegerAlgebra) are **fixed enums, not parametric**. The file's own comment (flag #2, lines 54-62) acknowledges scaffolding: *"When type-expressions-as-data closure lands… these tag enums collapse into direct algebra-and-carrier references."*
- **[`src/v3/compiler/src/infer.rs:3688-3767`](../../src/v3/compiler/src/infer.rs)** — `resolve_operator_arrow`: walks declaration chain looking for algebra `Conj`, looks up operator's field by name. **Site for unit-mismatch dispatch.** Currently no phantom-parameter check.
- **[`docs/thesis/compositional-modeling.md:570-617`](../thesis/compositional-modeling.md)** — target `Money<Currency>` and `Duration<Unit>` framing as carrier + phantom parameter. Syntax (`type Money<Currency>`) parses; `Declaration.type_params` exists per `docs/design-m2-feature-parity.md:175`. **Semantics deferred** — that's this lane.
- **[`docs/db-history/db-18.md:1-9`](../db-history/db-18.md)** — DB-18 actual scope (workflow effects only); does NOT include parametric algebra attachment. Use this as evidence for the R2 acceptance disclaimer.
- **[`MODELING.md`](../../MODELING.md)** + **[`INVARIANTS.md`](../../INVARIANTS.md)** + **[`CODING.md`](../../CODING.md)**.

## Frame

`Dimension<Carrier>` exists as a one-parameter behavioral framework (DB-3, R1) for proof-dimension analysis of symbolic costs. It is **not yet a typed value wrapper**: there is no `Money<USD>` value that propagates `USD` through arithmetic and compile-errors on unit-mismatch.

To inhabit `Dimension<Unit>` as a typed wrapper:
- **Type substrate** must support phantom-parameter attachment — a type parameter that doesn't appear in the runtime value but rides through type-checking and operator dispatch.
- **Operator dispatch** at `infer.rs:3688-3767` must check phantom parameters before resolving — `Money<USD> + Money<EUR>` rejects with a structured unit-mismatch diagnostic.
- **Declaration substrate** carries phantom-parameter metadata (which params are phantom; which propagate; which compose under abelian-group laws).

R2 acceptance is the **subset that lights up `Dimension<Unit>` end-to-end**, not the full parametric-algebra-attachment capability. Other parametric algebras (e.g., user-declared monoids over phantom carriers) are out of scope unless the substrate naturally generalizes.

## Five consumer-side requirements

1. **Phantom type parameter substrate** exists in v3. Either: extend `Declaration.type_params` to carry a `phantom: Bool` (or equivalent) per param; or add a sibling `Declaration.phantom_params` field. Phantom params don't appear in runtime values; the substrate carries this fact for type-checking + operator dispatch.
2. **Abelian-group algebra attachment for phantom parameters.** A way to declare *"the `Currency` phantom parameter on `Money<C>` composes under an abelian group: `Money<USD> + Money<USD>` is valid (closure); `Money<USD> + Money<EUR>` is unit-mismatch (closure violated)."* Closest existing precedent: algebra-tag enums in `primitives.dag`, but parametric — worker decides shape.
3. **Operator dispatch checks phantom parameters.** `infer.rs:3688-3767` `resolve_operator_arrow` (or sibling site) extends to check phantom-parameter consistency before resolving. New `Diagnostic::UnitMismatch` (or equivalent) emitted on violation, naming both phantom-parameter values + the operator + the abelian-group-closure violation.
4. **`Dimension<Unit>` end-to-end demo.** Smoke test: `type Money<C> { amount: Int }` with phantom `C: Currency`; `let usd: Money<USD> = ...`; `let eur: Money<EUR> = ...`; `usd + usd` works; `usd + eur` produces structured unit-mismatch diagnostic.
5. **No regression on existing `Dimension<Carrier>` behavioral framework.** DB-3's `analyze_symbolic_cost_dimension` (`src/v3/compiler/src/dimension.rs:50`) and the `SymbolicCost` proof-dimension consumer continue to work. The behavioral-framework `Dimension<Carrier>` and the value-wrapper `Dimension<Unit>` are different consumers of the same parametric machinery; both must coexist after this PR.

## Slice — phantom parameters + abelian-group attachment + dispatch check

1. Add phantom-parameter substrate (per req 1) to v3 `Declaration` shape. Round-trip through serializer / cementer / DB-8.
2. Add abelian-group algebra attachment (per req 2). Worker picks shape (parametric algebra-tag declaration vs new carrier).
3. Extend operator dispatch (per req 3) at `infer.rs:3688-3767`. New diagnostic variant.
4. Author `Money<C>` (or `Duration<Unit>` — worker discretion) demo type per req 4. Add to `dsl/std/` or fixture location.
5. Smoke + integration tests for reqs 4 + 5. Verify DB-3 behavioral-framework still works.

## Acceptance

- [ ] All 5 consumer-side requirements satisfied + documented in PR body.
- [ ] Phantom-parameter substrate lands; round-trips through DB-8.
- [ ] Abelian-group algebra attachment shape documented; worker rationale in PR body.
- [ ] Operator dispatch checks phantom parameters; unit-mismatch diagnostic structured.
- [ ] `Dimension<Unit>` end-to-end demo (`Money<USD>` or equivalent) compiles + tests pass.
- [ ] DB-3 behavioral-framework (`analyze_symbolic_cost_dimension` etc.) regression-free.
- [ ] `cargo test --workspace --exclude v2-compiler-tests` / `clippy --all-targets -- -D warnings` / `fmt --all --check` clean.
- [ ] DB-8 fixed-point converges bit-identically.
- [ ] SG-0 census deltas as needed.

## STOP-AND-ESCALATE

Surface to Director.

- **DB-tag mismatch becomes load-bearing** — if execution surfaces that the ROADMAP↔db-history DB-18 mismatch needs to resolve before this PR can land (e.g., a rename PR), STOP. Director routes.
- **Phantom-parameter substrate requires Class 5 `data` lowering** — if the demo `data money_dimension: Dimension<Currency> = {...}` requires top-level Class 5 substrate work (per DB-3's missing capability), STOP. May overlap with `ValueBody::List`/`Map` sub-lanes (#790 / map sub-lane brief).
- **Abelian-group attachment shape diverges from DB-3's `Dimension<Carrier>`** — if the attachment shape can't unify with the existing one-parameter proof-dimension framework, STOP. May indicate two parallel `Dimension` substrates which is bad single-authority shape.
- **Operator dispatch extension breaks existing operator resolution** — if extending `resolve_operator_arrow` requires a wholesale rewrite, STOP.
- **DB-8 fixed-point drifts** — STOP immediately.
- **Substrate.dag declaration changes** — coordinate with PB-Substrate (Zero-Floor).

## Non-goals

- **Not implementing the full parametric-algebra-attachment capability.** Scoped to `Dimension<Unit>` end-to-end.
- **Not implementing T-Modeling Dimensions** beyond the demo (req 4); that's the consumer.
- **Not migrating algebra-tag enums** in `primitives.dag` from fixed-enum to parametric — separate concern (the tag-enum dissolution is named in the file's own comment; do not pull it into this PR).
- **Not resolving the DB-18 ROADMAP↔db-history mismatch in this PR** unless STOP-AND-ESCALATE forces it.

## Reporting

- Single PR. Title: `feat(v3): T-Substrate parametric-algebra-for-Dimensions — phantom params + abelian-group attachment (lights up Dimension<Unit>)`.
- PR body cites this brief + addresses each of the 5 reqs + documents the chosen abelian-group attachment shape.
- On merge: signal Director; Director dispatches T-Modeling Dimensions worker brief authoring.

## Cross-manager note

- **Zero-Floor Manager**: heads-up. Substrate.dag-adjacent.
- **Grounding Manager**: no current overlap.
