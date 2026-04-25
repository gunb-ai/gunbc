# T-PerMethodMetadata — §6a per-method-metadata carrier pick `(S, R2)`

> **Director ad-hoc dispatch.** R2 T-PerMethodMetadata per
> [`docs/r2-structure.md`](../r2-structure.md) §"Goal 5". *"Design-call
> close, not substrate-capability work"* per the lane row. Reports to
> Director (`zesty-bear-812`).
>
> **Authority is distributed:** worker decides (per design-doc
> protocol); Director reviews + signs off. Worker has scope to make
> the pick — not deferring to Director.

## Read first

- **[`docs/design-substrate-carrier-port-program.md` §6a lines 156-175](../design-substrate-carrier-port-program.md)** — the full design call. Four options (0/1/2/3) with trade-offs. Currently *"(deferred)"* — the worker's job is to pick + lock the choice.
- **[`docs/r2-structure.md` §"Goal 5" + lane row](../r2-structure.md)** — sub-lane scoping. Note the lane is "design-call close, not substrate-capability work" — small PR; pick-and-document.
- **[`dsl/std/algebra.dag` lines 447-457, 469-569](../../dsl/std/algebra.dag)** — current v2 metadata authority: `AlgebraFieldTemplate` type + 69 `*_templates()` functions (population). The metadata being scoped: `size_effect`, `cost_shape`, `callback_element_position`.
- **[`src/v3/compiler/src/bootstrap_generated.rs:56`](../../src/v3/compiler/src/bootstrap_generated.rs)** — v3 transitive bootstrap reference; v3 has no native carrier yet.
- **The four options (verbatim from §6a):**
  - **Option 0**: keep lens-local lookup tables (current v2 state). No substrate/std change.
  - **Option 1**: extend type declarations with field-level refinements. Largest substrate change (needs DB-11 annotation support).
  - **Option 2**: separate metadata carriers per algebra (`OrderedRingMetadata`, etc.). No substrate change; per-algebra carrier proliferation.
  - **Option 3**: unified `MethodContract` carrier — generic `(algebra_id, method_id)` indexed lookup. No substrate change; minimal std surface (one carrier total). Closest to `TemplateArgumentBinding` shape.
- **E-family R1 closure context:** E-T (PR #682), E-C, E-I, E-P (partial via PR #742), E-M (closed via M-b structural subsumption) all landed in R1. §6a is the **only E-family design call that didn't land in R1**; this brief closes it.
- **[`src/v3/lenses/cost.dag`](../../src/v3/lenses/cost.dag)** + **[`src/v3/lenses/complexity.dag`](../../src/v3/lenses/complexity.dag)** — the consumer lenses. Their consumption pattern is what evidence-narrows the option choice. Read to see how metadata is currently looked up + what shape the lens code naturally consumes.
- **[`MODELING.md`](../../MODELING.md)** + **[`INVARIANTS.md`](../../INVARIANTS.md)** + **[`CODING.md`](../../CODING.md)**.

## Frame

§6a was deferred at design-doc time pending E-P consumer wiring + E-I preflight evidence. Both have landed; the evidence to narrow options 0/1/2/3 is now available. This brief is the close-out PR: read the §6a options, read the consumer lenses, gather evidence on which carrier shape consumers naturally want, make the call, lock it.

The pick is **not** an extension of substrate capability — all four options are achievable with existing substrate. The call is purely **where** the metadata structurally lives + **which coupling story** is cleanest given E-family evidence.

## Three consumer-side requirements

1. **Make the pick.** Read §6a + the consumer lenses + any E-P preflight findings. Pick option 0/1/2/3. Document rationale citing concrete consumer-shape evidence (e.g., *"cost.dag and complexity.dag consume `size_effect` and `cost_shape` together at the same call sites; co-location wins → Option 3"*). Do not pick by author preference; pick by evidence.
2. **Land the chosen carrier shape.** If Option 0 (keep lens-local): document the rationale, update §6a to lock the choice, no substrate/std changes needed beyond doc updates. If Option 1/2/3: author the minimal carrier shape + minimal lens-side adoption demonstrating the chosen shape works. **Do not migrate all consumer lenses** — one demo migration is sufficient evidence for the chosen shape; bulk migration is post-pick work.
3. **Update the design doc.** `docs/design-substrate-carrier-port-program.md` §6a updates: replace *"(deferred)"* language with the picked option + rationale + evidence cited. Documentation must reflect the live state per `INVARIANTS.md`.

## Slice — pick + minimal carrier + design-doc lock

1. Read §6a + consumer lenses + E-P / E-I evidence. Make the pick.
2. If Option 0: skip carrier authoring; go to step 4.
3. If Option 1/2/3: author the minimal carrier + minimal lens-side adoption demo.
4. Update §6a in design doc to lock the choice (replace "deferred" language).
5. PR description: cite this brief + the picked option + the evidence that narrowed the choice.

## Acceptance

- [ ] All 3 consumer-side requirements satisfied + documented in PR body.
- [ ] §6a design doc updated; "(deferred)" language replaced with picked option + rationale.
- [ ] If Option 1/2/3: minimal carrier lands; one consumer lens migrated to demonstrate.
- [ ] If Option 0: rationale for keeping lens-local tables documented; no further changes.
- [ ] No regression on existing consumers (cost.dag / complexity.dag).
- [ ] `cargo test --workspace --exclude v2-compiler-tests` / `clippy --all-targets -- -D warnings` / `fmt --all --check` clean.
- [ ] DB-8 fixed-point converges bit-identically.

## STOP-AND-ESCALATE

Surface to Director (Director-review by design; STOP for substantive scope changes).

- **Evidence is ambiguous between two options** — if E-P + lens consumer evidence doesn't clearly favor one over another, STOP. Director-call on the tiebreaker.
- **Picked option requires substrate-capability work** — if Option 1 (DB-11 annotation extension) turns out to need substrate-capability beyond DB-11's current shape, STOP. The lane row says "design-call close, not substrate-capability work" — substrate work re-routes.
- **Bulk migration creep** — if implementing the demo lens migration reveals that bulk-migrating all consumers in this PR is the only honest scope, STOP. Bulk migration is post-pick work; this PR locks the shape.
- **Design-doc structural conflicts** — if updating §6a reveals adjacent §6 sections also need updating (consistency cascade), STOP.
- **DB-8 fixed-point drifts** — STOP immediately.

## Non-goals

- **Not migrating all consumer lenses** to the chosen carrier — one demo is sufficient.
- **Not extending substrate capability** beyond what the picked option naturally requires.
- **Not closing other E-family design calls** — only §6a; other E-family items closed in R1.
- **Not blocking on consumer lens refactors** that are out of scope.

## Reporting

- Single PR. Title: `feat(v3): T-PerMethodMetadata §6a — pick {Option N} for per-method metadata carrier (locks design call)`.
- PR body cites this brief + the picked option + the evidence that narrowed the choice + the §6a doc-update receipt.
- On merge: signal Director; bulk migration of remaining consumer lenses (if needed) is follow-up work.

## Cross-manager note

- **Zero-Floor Manager**: no current overlap (lens-side or design-doc work; not substrate.dag-adjacent unless Option 1 reveals DB-11 extension scope).
- **Grounding Manager**: no current overlap.
