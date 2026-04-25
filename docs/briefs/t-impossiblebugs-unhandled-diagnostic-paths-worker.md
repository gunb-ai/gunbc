# T-ImpossibleBugs — unhandled diagnostic paths **(DESIGN/SCOPING brief, S — produces substrate proposal, NOT implementation)**

> **Director ad-hoc dispatch.** R2 T-ImpossibleBugs class 2 of 3 per
> [`docs/r2-structure.md`](../r2-structure.md) §"Goal 4". Independent
> of the other two impossible-bug classes. Reports to Director
> (`zesty-bear-812`).
>
> **🔄 REFRAMED 2026-04-25 (post-`sunny-deer-629` STOP-AND-ESCALATE).**
> Original brief framed this as an implementation lane against an
> "ownership_lens" precedent that was shape-only, not mechanism. Worker
> verified at HEAD: (a) **DB-11 deliberately strips refinements at
> operator dispatch** at `src/v3/compiler/src/infer.rs:3693-3703` as a
> designed-in fix for a prior failure mode (refinement-as-proof-obligation
> breaks symmetric operators like `>`); (b) the brief's framing of
> *"attach `where b != 0` as a proof for `a / b`"* directly contradicts
> this design choice — operator dispatch is engineered to *ignore*
> refinements on operands; (c) ownership_lens is a post-hoc
> observability lens (consumes lowered DAG and asserts a count), NOT
> a proof carrier that gates type-checking; the precedent is
> shape-only, not mechanism. Worker correctly STOP-and-escalated with
> recommendation to redirect to design/scoping. **Director picked
> redirect.** This brief now produces a substrate proposal +
> bypass-or-substrate-or-park decision, NOT implementation.

## Read first

- **[`THESIS.md` §"Tier 2 — Runtime safety" + §"Enumerable impossible-bug classes" lines 348-350](../../THESIS.md)** — class definition + the *"Gated on Tier 2 substrate (post-R1)"* gate.
- **[`docs/r2-structure.md` §"Goal 4"](../r2-structure.md)** — sub-lane scoping; tagged `[R2+]`.
- **[`src/v3/compiler/src/infer.rs:3688-3767`](../../src/v3/compiler/src/infer.rs)** — `resolve_operator_arrow`. **Critical: DB-11 refinement-strip at `:3693-3703`** is the design conflict. Read the comment block in full; it documents *why* refinements are stripped at operator dispatch (mirror-refinement failure on symmetric operators).
- **[`docs/db-history/db-11.md`](../db-history/db-11.md)** + **[`docs/design-m2-feature-parity.md`](../design-m2-feature-parity.md)** — DB-11 alias-RHS `where` (PR #703) + its discharge semantics. **DB-11 does structural identity of refined types, NOT logical entailment.** Predicate-entailment (does user's `b != 0` entail operator's `denominator != 0`?) is materially different and is **not** in v3 substrate today.
- **[`src/v3/compiler/tests/m2_feature_parity_test.rs:331-700`](../../src/v3/compiler/tests/m2_feature_parity_test.rs)** — DB-11 test_3a3_* suite locks the strip-and-discharge semantics. The user-defined-total-wrapper path (`fn divide_safe(a: Int, b: Int where b != 0) -> ...`) already works today via standard refinement-on-parameter; that's the existing surface this lane could either build on or sidestep.
- **[`MODELING.md`](../../MODELING.md)** + **[`INVARIANTS.md`](../../INVARIANTS.md)**.

## Frame — design-scoping, not implementation

Output of this lane is a **scoping document** (lands as `docs/briefs/t-impossiblebugs-unhandled-diagnostic-paths-design.md` — worker picks placement), **NOT** code change. The doc answers four questions:

1. **DB-11 interaction analysis** — exact characterization of the refinement-strip at `infer.rs:3693-3703`, with file:line and the comment-block reasoning. Worker has already done this work; the design doc consolidates it.
2. **Substrate proposal for proof-or-totality enforcement.** What new substrate is needed to make `a / b` require `b: Int where b != 0` (or equivalent) without conflicting with DB-11? Three load-bearing components surface from the worker's investigation:
   - **Per-operator partiality fact** (which operand carries the precondition + what predicate is the precondition).
   - **Predicate-entailment check** (logical entailment, not structural identity — DB-11 does the latter only).
   - **Asymmetric per-operand refinement-honoring** at dispatch (in tension with DB-11's symmetric strip rule).
3. **Bypass-vs-park decision.** Three feasible outcomes, each with a concrete shape:
   - **(a) Bypass-feasible**: there's a narrower mechanism that closes the bug class without conflicting with DB-11 (e.g., a typed sum-totality that shifts the burden to return-type lift `Result<T, DivideByZero>` rather than precondition-attachment). If the worker finds a clean bypass, name the implementation-brief shape.
   - **(b) Substrate-design upstream**: the work is genuinely Tier 2 substrate (predicate-entailment + per-operator partiality + asymmetric refinement-honoring), and that substrate doesn't exist yet. Recommend parking this lane behind a Tier 2 substrate brief that authors the substrate first.
   - **(c) Narrow-demo theatre**: brief req 3 already names the user-defined-total-wrapper path (`divide_safe`) which works today via DB-11 refinement-on-parameter. This is a valid demo but is **not** the impossible-bug class closure THESIS:350 promises — it demonstrates total-variant ergonomics, not proof-or-totality enforcement. Worker should explicitly flag this as acceptance-theatre risk.
4. **Director-actionable recommendation.** Pick one of (a/b/c) with concrete reasoning citing DB-11 evidence + substrate-shape questions. If (a), name implementation brief shape. If (b), name the substrate-design brief shape. If (c), name what the demo proves vs doesn't.

This lane is sized **S** because it's design-scoping. Output is a doc PR.

## Three consumer-side requirements

1. **DB-11 interaction analysis documented** with file:line citations from `infer.rs:3693-3703` + the `m2_feature_parity_test.rs` test suite. Worker has done this; doc consolidates.
2. **Substrate proposal OR park-decision documented.** Section walking through the three load-bearing substrate components (per-operator partiality, predicate-entailment, asymmetric refinement-honoring) with cited substrate facts. No invented vocabulary.
3. **Director-actionable recommendation** picking one of (a) bypass-feasible / (b) substrate-design-upstream / (c) narrow-demo-theatre, with concrete reasoning + named follow-on brief shape.

## Slice — design-scoping doc

1. Document the DB-11 conflict in detail (worker's `infer.rs:3693-3703` find consolidated).
2. Substrate proposal section walking the three components.
3. Bypass investigation: is there a sum-totality-only path that sidesteps DB-11?
4. Author the scoping doc (location worker's call).
5. PR description: cite this brief + the scoping-doc receipt + the recommendation.

## Acceptance

- [ ] Scoping doc landed with all 3 consumer-side requirements addressed.
- [ ] Director-actionable recommendation: bypass-feasible / substrate-design-upstream / narrow-demo-theatre — pick one, cite reasoning.
- [ ] Acceptance-theatre risk explicitly flagged if recommendation is (c) or if (a)'s bypass turns out to be sum-totality-only (which is ergonomic, not impossible-bug-class-closure).
- [ ] No code changes to v3 substrate.
- [ ] `cargo fmt --all --check` clean.

## STOP-AND-ESCALATE

Surface to Director.

- **DB-11 conflict turns out illusory** (e.g., the refinement-strip can be made asymmetric per-operand without breaking the symmetric-operators-fix DB-11 protects against) — this is the "good outcome" for bypass-feasibility. NOT a STOP, but worker should flag explicitly with reasoning.
- **Substrate proposal requires inventing fundamental new substrate vocabulary** beyond what predicate-entailment + per-operator partiality already name — STOP. May indicate scope is even larger than M+.
- **Bypass investigation reveals a clean total-variant-only path** that closes the bug class for SOME partial ops but not others (e.g., works for divide via `Result<Int, DivideByZero>` but not for force-unwrap or OOB indexing) — STOP. Director-call on whether to scope this lane to one demo op.

## Non-goals

- **Not implementing the proof-or-totality check.** This is scoping, not implementation.
- **Not modifying v3 substrate.** Doc-only output.
- **Not closing other T-ImpossibleBugs classes** — independent briefs.
- **Not re-authoring DB-11.** That's settled R1 work.

## Reporting

- Single PR. Title: `docs(briefs): T-ImpossibleBugs unhandled-diagnostic-paths — design/scoping doc (post-sunny-deer-629 redirect)`.
- PR body cites this brief + the scoping-doc receipt + the chosen recommendation.
- On merge: signal Director with the recommendation; Director either authors the bypass implementation brief, the Tier 2 substrate brief, or parks the lane.

## Cross-manager note

- **Zero-Floor Manager**: heads-up if recommendation lands on substrate-design-upstream — that's substrate-territory potentially.
- **Grounding Manager**: no current overlap.
