# T-ImpossibleBugs — unhandled diagnostic paths `(S, R2)`

> **Director ad-hoc dispatch.** R2 T-ImpossibleBugs class 2 of 3 per
> [`docs/r2-structure.md`](../r2-structure.md) §"Goal 4". Independent
> of the other two impossible-bug classes — any worker can dispatch
> in parallel. Reports to Director (`zesty-bear-812`).

## Read first

- **[`THESIS.md` §"Tier 2 — Runtime safety" + §"Enumerable impossible-bug classes" lines 348-350](../../THESIS.md)** — class definition: *"Division by zero, integer overflow, out-of-bounds, force-unwrap, partial functions — either proven safe at compile time or made total. No partial functions in the runtime."* THESIS.md:350: *"Gated on Tier 2 substrate (post-R1)."*
- **[`docs/r2-structure.md` §"Goal 4"](../r2-structure.md)** — sub-lane scoping; `[R2+]` per ROADMAP T-Demo row.
- **[`src/v3/std/verification.dag`](../../src/v3/std/verification.dag)** — current `DiagnosticKind` (tokenizer / parse / type / arity / resolve). Extend (or sibling carrier) for runtime-safety diagnostic paths.
- **[`src/v3/compiler/src/infer.rs`](../../src/v3/compiler/src/infer.rs)** — type inference site; proof-obligation generation for partial ops would attach here. File:line for the operator-resolution site already known via T-Substrate parametric-algebra brief: `:3688-3767` `resolve_operator_arrow`.
- **[`docs/thesis/what-falls-out.md`](../thesis/what-falls-out.md)** — Tier 2 model framing. May need extension into a sibling doc if this brief authors the carrier-evidence shape.
- **Existing precedent — ownership proofs.** Per ROADMAP §Tier 2 ("ownership: the compiler proves no aliased mutation"), v3 has a partial-ops-style proof carrier already. Search for `OwnershipProof` / `ownership_lens` / similar to find the precedent shape; the new diagnostic-path proof carrier follows the same pattern.
- **[`MODELING.md`](../../MODELING.md)** — M9 + closed-system framing.
- **[`INVARIANTS.md`](../../INVARIANTS.md)** — `feedback_construction_over_ratchets`: model first, violations dissolve.

## Frame

Tier 2 obligates that **every partial operation** (divide, index, force-unwrap, integer-overflow, OOB) is either (a) **proven safe** via a structural proof carried alongside the operation, or (b) **made total** via a return-type lift (`Result<T, E>` / `T | DivisionFailed` / etc.). Neither is enforced today; partial operations type-check unconditionally.

This brief introduces the **substrate to track partiality + proof-or-totality requirement** and the **diagnostic that fires when neither is provided**. The bug class becomes impossible-by-construction once the type system refuses to type-check `a / b` without either `b ≠ 0` proof or a total-Int signature.

Sub-lane scope: enough Tier 2 substrate to close at least one partial-op class end-to-end (likely `divide` as the demo). Other partial-op classes follow the same pattern; demonstrate the substrate, scope to one consumer.

## Three consumer-side requirements

1. **Partiality fact on operations.** Each partial op carries a substrate fact marking it as partial + the precondition shape it expects (e.g., `divide` → `denominator != 0` precondition; `index` → `0 <= idx < length` precondition; `force_unwrap` → `is_some` precondition). Substrate-declared, not Rust-mirrored.
2. **Proof-or-totality check at type-checking.** When a partial op is used, the type-checker requires either: (a) a structurally-attached proof-term satisfying the precondition (e.g., a `where b != 0` refinement on `b`), or (b) the call-site uses a total signature variant returning `Result<T, E>`. Otherwise emit `Diagnostic::UnhandledDiagnosticPath { op, missing_proof, fix_hints }`.
3. **End-to-end demo: divide.** Smoke + integration test: `let x = a / b` where `b: Int` (no proof) compile-errors with the new diagnostic; `let x = a / b` where `b: Int where b != 0` (proof attached) compiles; `let x = divide_safe(a, b)` returning `Result<Int, DivideByZero>` compiles. Other partial ops out of scope; demo proves the substrate.

## Slice — partiality fact + proof-or-totality check + divide demo

1. Add partiality fact to operation declarations. Likely a new field on the relevant declaration carrier (probably operator declarations in `dsl/std/algebra.dag` or per-target `primitives.dag`).
2. Extend type-checker (in `infer.rs`) to enforce proof-or-totality at use sites of partial ops. New diagnostic variant.
3. Annotate `divide` (or whichever partial op the worker picks for the demo) with the partiality fact.
4. Smoke + integration tests per req 3.
5. Doc updates — likely a new `docs/thesis/tier-2-runtime-safety-proofs.md` or extension to `docs/thesis/what-falls-out.md`.

## Acceptance

- [ ] All 3 consumer-side requirements satisfied + documented in PR body.
- [ ] Partiality fact substrate lands; round-trips through DB-8.
- [ ] Type-checker rejects un-proven partial-op uses with structured diagnostic.
- [ ] `divide` (or chosen demo) end-to-end: bare use rejected; proof-attached use accepted; total-variant use accepted.
- [ ] No regression on existing operator resolution.
- [ ] `cargo test --workspace --exclude v2-compiler-tests` / `clippy --all-targets -- -D warnings` / `fmt --all --check` clean.
- [ ] DB-8 fixed-point converges bit-identically.

## STOP-AND-ESCALATE

Surface to Director.

- **Tier 2 substrate dependency** — if proof-term carrier requires inventing fundamental new substrate (predicate-as-fact distinct from DB-11's value-refinement), STOP. May indicate the sub-lane is mis-scoped as S; could be M+.
- **Proof-attachment syntax interaction with DB-11** — if `where` clause reuse for proof-attachment conflicts with DB-11's value-refinement semantics, STOP.
- **Other partial-op classes generalize differently** — if the chosen demo (divide) reveals that index / force-unwrap / overflow need divergent substrate, STOP. Director-call on demo choice + scope.
- **Existing ownership-proof precedent doesn't generalize** — if reusing the ownership-proof shape doesn't fit, STOP.
- **DB-8 fixed-point drifts** — STOP immediately.

## Non-goals

- **Not closing all partial-op classes.** One demo-end-to-end is sufficient evidence for the substrate; bulk migration of remaining classes is post-cascade work.
- **Not building Tier 2's full proof system** — scoped to enough substrate for the demo.
- **Not implementing the other two T-ImpossibleBugs classes** — independent briefs.
- **Not lifting all integer ops to `Result<T, Overflow>`** — only the chosen demo path.

## Reporting

- Single PR. Title: `feat(v3): T-ImpossibleBugs — partiality fact + proof-or-totality check (closes unhandled-diagnostic-paths class via divide demo)`.
- PR body cites this brief + addresses the 3 reqs + documents which demo op was picked + the precedent (ownership-proof) reuse.
- On merge: signal Director; bulk migration of other partial-op classes is post-cascade work.

## Cross-manager note

- **Zero-Floor Manager**: heads-up if substrate.dag-adjacent.
- **Grounding Manager**: no current overlap.
