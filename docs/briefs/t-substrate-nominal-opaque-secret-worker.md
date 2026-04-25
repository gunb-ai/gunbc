# T-Substrate sub-lane 2 — nominal-opaque substrate for `Secret<T>` graduation `(M, R2 substrate)`

> **Director ad-hoc dispatch.** R2 T-Substrate sub-lane 2 per
> [`docs/r2-structure.md`](../r2-structure.md) §"Goal 3" item 2.
> Reports to Director (`zesty-bear-812`).

## Read first

- **[`docs/r2-structure.md`](../r2-structure.md) §"Goal 3" item 2** — sub-lane scoping. Adjacent to DB-11 alias-RHS `where` (landed R1 via PR #703); does **not** overlap DB-18 (workflow effects, orthogonal).
- **[`docs/thesis/compositional-modeling.md` §Part 4](../thesis/compositional-modeling.md)** lines ~619-644 — target Secret<T> shape:
  ```dag
  type Secret<T> { value: T }
    where only std.secrets::acquire may construct
          no Show instance
          no String coercion
          no Debug derivation
  ```
  Acceptance is exactly: `Secret<T>` graduates from the structural alias `type Secret = String` (`dsl/std/types.dag:237`) to the nominal-opaque construction-restricted form.
- **[`dsl/std/types.dag:21, 237, 250-305`](../../dsl/std/types.dag)** — current state: 16 branded newtypes (`IntentId`, `IssueId`, etc.) using **structural refinement via Brand DAG nodes**. Nominally disjoint but constructible anywhere — *not* construction-restricted. `Secret = String` is a plain alias (line 237), no branding even.
- **[`docs/db-history/db-11.md:13-32`](../db-history/db-11.md)** + **[`docs/design-m2-feature-parity.md:72-166`](../design-m2-feature-parity.md)** — DB-11 alias-RHS `where` (PR #703). Closest **structural precedent** — `type X = Y where <constraint>` parses + lowers; predicate becomes a `Declaration.refinement` edge. **Constrains values** (`x >= 0`), not constructors.
- **[`src/v3/compiler/parse_parser_body.txt:594-620`](../../src/v3/compiler/parse_parser_body.txt)** — DB-11 parse path; precedent for parse-side handling of `where` extensions.
- **[`docs/db-history/db-18.md:1-9`](../db-history/db-18.md)** — DB-18 scope confirmation: workflow-effect carrier + Rust reflection only. **Not parametric algebra; not nominal types.** Orthogonal to this sub-lane.
- **[`ROADMAP.md:414`](../../ROADMAP.md)** — "Secret<T> nominal-wrapper graduation" debt row. Notes: *"Alias form cannot carry those restrictions; the substrate distinction between nominal-opaque and alias types is the structural delta."*
- **[`MODELING.md`](../../MODELING.md)** + **[`INVARIANTS.md`](../../INVARIANTS.md)** + **[`CODING.md`](../../CODING.md)**.

## Frame

v3 has **brand-refinement** (16 branded newtypes via Brand DAG nodes — structurally distinct, constructible anywhere) and **value-refinement** (DB-11's `Declaration.refinement` edges — constrain values, not constructors). It has **zero construction-restriction** patterns: no `private`, no module-scoped builders, no constructor authorization, no visibility model.

`Secret<T>` graduation requires **two new substrate axes**:
1. **Nominal-opaque distinction** — a type that's structurally identical to its `T` payload but type-distinct at the boundary (so `Secret<String>` is not assignable to `String` and vice versa).
2. **Constructor-restriction modeling** — a way to express *"only `std.secrets::acquire` may construct values of this type"* and reject attempts to construct elsewhere with a structured diagnostic.

Plus likely a third axis (the Secret<T> target also needs):
3. **Instance-suppression mechanism** — the ability to declare *"no Show / Debug / String-coercion instance for this type"* so accidental logging fails to compile.

Sub-lane scope is **sufficient-to-graduate-Secret<T>**, not full nominal-types substrate. Worker decides whether instance-suppression (axis 3) lands in this sub-lane or as a follow-up; surface the call in PR description.

## Five consumer-side requirements

1. **Nominal-opaque type carrier exists in v3 substrate.** New `TypeConnective` variant or `Declaration` field expressing *"this type is nominally distinct from its structural representation."* Not a Brand node (those compose with structural refinement); a separate axis. The carrier's element shape must round-trip through serializer / cementer / DB-8 fixed-point machinery. Worker's call on naming (e.g., `NominalOpaque`, `OpaqueWrapper`, etc.).
2. **Constructor-authority predicate.** A `where only X may construct` syntax that parses + lowers + carries the authorized constructor's `DeclarationId` as substrate fact. Closest precedent shape: DB-11's predicate `Declaration.refinement` edge — extend it (or sibling field) to carry constructor-authority predicates distinct from value predicates.
3. **Construction-site rejection diagnostic.** Lowering / inference rejects construction attempts at sites other than the named constructor with a structured diagnostic naming the type, the unauthorized site, and the authorized constructor. No silent allow-through.
4. **Instance-suppression (or explicit STOP if deferred).** Either: the substrate carries `no Show / no Debug / no String coercion` facts and the relevant lens / emitter consumers honor them; OR the worker explicitly defers this axis to a follow-up brief with reasoning in PR description. Director-call routing if deferred.
5. **Demo: `Secret<String>` graduates end-to-end.** The current `dsl/std/types.dag:237` `type Secret = String` alias updates to the nominal form. Tests assert: (a) `let s: Secret<String> = std.secrets::acquire(...)` works; (b) `let s: Secret<String> = "raw"` is a compile error; (c) `println("token: " + s)` is a compile error (instance-suppression-dependent — gated on req 4 disposition).

## Slice — nominal-opaque carrier + constructor-authority predicate

1. Add nominal-opaque type carrier (per req 1) to v3 substrate (likely new `TypeConnective` variant or `Declaration` field).
2. Extend parser / lowerer for `where only X may construct` predicate (per req 2). Reuse DB-11 plumbing where applicable.
3. Construction-site enforcement at lowering / inference (per req 3). New `Diagnostic::UnauthorizedConstruction` (or equivalent) variant.
4. Instance-suppression (per req 4) — implement or explicitly defer.
5. Graduate `dsl/std/types.dag:237` `Secret` to nominal form (per req 5). Wire `std.secrets::acquire` constructor.
6. Smoke + integration tests for reqs 5(a/b/c).

## Acceptance

- [ ] All 5 consumer-side requirements satisfied + documented in PR body (req 4 may be "deferred with reasoning" — Director-call).
- [ ] Nominal-opaque substrate carrier lands; round-trips through serializer / cementer / DB-8.
- [ ] `where only X may construct` predicate parses + lowers + carries constructor-authority fact.
- [ ] Unauthorized-construction diagnostic lands; smoke test exercises it.
- [ ] `Secret<String>` graduates end-to-end with tests for (a/b) and (c if req 4 implemented).
- [ ] `cargo test --workspace --exclude v2-compiler-tests` / `clippy --all-targets -- -D warnings` / `fmt --all --check` clean.
- [ ] DB-8 fixed-point converges bit-identically.
- [ ] SG-0 census deltas: any retired hand-Rust off the list.

## STOP-AND-ESCALATE

Surface to Director.

- **Nominal-opaque axis interaction with Brand nodes** — if the chosen carrier conflicts with the existing Brand-refinement substrate (16 newtypes today) or requires retroactive migration of branded types, STOP. Director-call on whether to migrate or co-exist.
- **Constructor-authority predicate needs a visibility model** — if `where only X may construct` requires inventing a module-visibility / scope-resolution layer that doesn't exist, STOP. May escalate scope beyond sub-lane M.
- **Instance-suppression (req 4) needs cross-cutting lens / emitter changes** — if implementing it touches lens authority surfaces, STOP. Coordinate with lens producer owners or defer with reasoning.
- **DB-11 plumbing doesn't generalize** — if reusing DB-11's `Declaration.refinement` for constructor-authority breaks DB-11's value-refinement semantics, STOP.
- **DB-8 fixed-point drifts** — STOP immediately.
- **Substrate.dag declaration changes** — coordinate with PB-Substrate (Zero-Floor); STOP.

## Non-goals

- **Not implementing T-Modeling Secret<T>** beyond the graduation demo (req 5); this sub-lane is substrate, not modeling.
- **Not migrating branded types** to nominal-opaque (separate axis; co-existence is the default).
- **Not building a full visibility / module-system** — sub-lane scope is constructor-authority predicate, not general visibility.
- **Not closing DB-11** — that landed R1; this extends sibling capability.

## Reporting

- Single PR. Title: `feat(v3): T-Substrate nominal-opaque-for-Secret — nominal-opaque carrier + constructor-authority predicate (graduates Secret<T>)`.
- PR body cites this brief + addresses each of the 5 reqs + documents req 4 disposition (implemented vs deferred).
- On merge: signal Director; Director dispatches T-Modeling Secret<T> worker brief authoring.

## Cross-manager note

- **Zero-Floor Manager**: heads-up. Nominal-opaque carrier + construction-authority predicate may touch substrate.dag.
- **Grounding Manager**: no current overlap.
