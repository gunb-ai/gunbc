# T-ImpossibleBugs — Unhandled diagnostic paths implementation worker brief `(M; consumer of Tier 2 substrate)`

> **Worker brief.** Reports through Impossible-Bugs Manager (post-R2
> spin-up) / Director (pre-spin-up). T-ImpossibleBugs Goal 4 class 2
> of 3.
>
> **Gated on:** Tier 2 substrate (post-R1 deeper-substrate work; per
> THESIS.md gate). Coordinate with Substrate Manager — Tier 2 substrate
> is **not** in any of the four Wave 2 sub-lanes; this gate is
> distinct from the int-lit / Secret / Dimensions producer set.
> **Do not dispatch until Tier 2 substrate gate clears.**

## Read first

- **[`docs/briefs/t-impossiblebugs-unhandled-diagnostic-paths-design.md`](t-impossiblebugs-unhandled-diagnostic-paths-design.md)** — design/scoping brief (post-`sunny-deer-629` STOP-AND-ESCALATE redirect). Read in full.
- **[`THESIS.md` §"Tier 2 — Runtime safety" + lines 348-350](../../THESIS.md)** — class definition + Tier 2 gate.
- **[`src/v3/compiler/src/infer.rs:3688-3767`](../../src/v3/compiler/src/infer.rs)** — `resolve_operator_arrow`; **DB-11 refinement-strip at `:3693-3703`** is the design-conflict anchor. Refinements are deliberately stripped at operator dispatch.
- **[`docs/db-history/db-11.md`](../db-history/db-11.md)** + **[`docs/design-m2-feature-parity.md`](../design-m2-feature-parity.md)** — DB-11 structural identity vs predicate entailment distinction.

## Frame

Tier 2 runtime-safety proofs make division-by-zero, OOB, and force-unwrap either proven safe at compile time or made total — never partial at runtime. Per THESIS Tier 2 framing.

The design brief surfaced the substrate conflict: DB-11 deliberately strips refinements at operator dispatch (the brief's original framing of "attach `where b != 0` as proof for `a / b`" directly contradicts that design). Predicate entailment (does user's `b != 0` entail operator's `denominator != 0`?) is **not in v3 substrate today**.

Per design brief: the resolution is one of (a) build predicate-entailment substrate (Tier 2 deeper substrate work — out of scope for this brief), (b) make partial operations total via design (force-unwrap dissolved per `feedback_totality_by_omission`), or (c) park until Tier 2 substrate lands.

This implementation brief assumes (a) — Tier 2 predicate-entailment substrate has landed. If (b) or (c) is the path, this brief reframes accordingly.

## Slice (assume Tier 2 substrate landed)

1. **Confirm Tier 2 substrate readiness.** Predicate entailment infrastructure available; operator dispatch consumes it.
2. **Migrate operator dispatch consumers** — division, indexing, force-unwrap call sites — to consume predicate entailment instead of raw operand types.
3. **Diagnostic for unproven safety.** When entailment fails (user calls `a / b` without proving `b != 0`), emit typed diagnostic at compile time. Per C-8.
4. **Make legitimate partial paths explicit.** Operations that genuinely admit partial-failure receive an explicit primitive (`checked_div`, `try_index`) returning a sum type — per `feedback_totality_by_omission`.
5. **Regression tests:**
   - `a / b` with `b != 0` proven entails compiles cleanly.
   - `a / b` without proof produces typed diagnostic.
   - Existing safe code stays bit-identical.
6. **DB-8 fixed-point bit-identical.**

## Acceptance

- [ ] Tier 2 substrate readiness confirmed.
- [ ] Operator dispatch consumes predicate entailment.
- [ ] Typed diagnostic for unproven safety (C-8).
- [ ] Partial paths explicit via dedicated primitives.
- [ ] Regression tests cover proven / unproven / bit-identity.
- [ ] DB-8 converges bit-identically.
- [ ] Cross-program signal: Impossible-Bugs Manager → R2 Release Manager (Goal 4 unhandled-diagnostic class).
- [ ] `cargo test` / clippy / fmt clean.

## STOP-AND-ESCALATE

- **Tier 2 substrate readiness signal exists but doesn't cover predicate entailment** — verify before slicing. Substrate-deeper work needed.
- **Path (b) `feedback_totality_by_omission` lands instead of substrate path (a).** Reframe brief: dissolve partial primitives by removing them; the brief becomes a primitive-set-narrowing task, not a predicate-entailment-consumer task.
- **Path (c) park** — surface to Director / R2 Release Manager that this class doesn't close in R2.
- **Symmetric-operator interaction (the original DB-11 strip rationale).** If Tier 2 substrate covers asymmetric operators only, surface — symmetric `>` etc. still strip refinements.
- **DB-8 drifts** — STOP immediately.

## Non-goals

- Not authoring Tier 2 substrate itself (Substrate Manager territory).
- Not extending refinement-checking to non-operator dispatch (separate substrate work).
- Not addressing other T-ImpossibleBugs classes.

## Cross-program note

- **Producer prerequisite:** Substrate Manager → Tier 2 substrate (predicate entailment infrastructure).
- **Consumer:** this brief.
- **Downstream signal:** R2 Release Manager (Goal 4 close).
- **Adjacent path (b):** if `feedback_totality_by_omission` is the chosen route, coordinate with Modeling Manager (primitive-set narrowing may touch their territory).

## Reporting

Single PR. Title: `feat(v3): T-ImpossibleBugs unhandled diagnostic paths — operator dispatch consumes Tier 2 predicate entailment OR partial-primitive dissolution`. Body cites this brief + design brief + substrate signal-receipt + path (a/b/c) decision + DB-8 disposition.

On merge: signal R2 Release Manager.
