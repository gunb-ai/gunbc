# T-ImpossibleBugs — Nested-optional flatten implementation worker brief `(M; consumer of T-Substrate cardinality refinement)`

> **Worker brief.** Reports through Impossible-Bugs Manager (post-R2
> spin-up) / Director (pre-spin-up). T-ImpossibleBugs Goal 4 class 1
> of 3.
>
> **Gated on:** Substrate Manager readiness signal for cardinality
> refinement substrate (T-Substrate sub-lane work; coordinate with
> `r2-substrate-cardinality-for-int-lit-subset.md` and DB-11 alias-`where`
> closure). **Do not dispatch until the substrate gates are clear.**

## Read first

- **[`docs/briefs/t-impossiblebugs-nested-optional-flatten-design.md`](t-impossiblebugs-nested-optional-flatten-design.md)** — design/scoping brief (post-`bright-moth-390` STOP-AND-ESCALATE redirect). Read in full before authoring slice.
- **[`THESIS.md` lines 342-344](../../THESIS.md)** — class definition + cardinality-refinement-substrate gate.
- **[`dsl/std/algebra.dag:423`](../../dsl/std/algebra.dag)** — `OptionalOf { inner: AlgebraTypeTemplate }` (verify line at dispatch).
- **[`docs/db-history/db-11.md`](../db-history/db-11.md)** — alias-RHS `where`; design-side closure gate.

## Frame

`Option<Option<T>>` accessor patterns require hand-unwrapping in normal languages; the bug class is the runtime `None` propagation that should be structurally impossible. Per design brief and THESIS, the resolution is structural flatten at the substrate level — `Option<Option<T>>` is structurally identical to `Option<T>` because `OptionalOf` is idempotent under self-nesting at the algebra-template level.

**The substrate prerequisite:** cardinality refinement substrate must support the structural-rewrite (`OptionalOf<OptionalOf<T>> ≡ OptionalOf<T>`). Per design brief, this is NOT a quick implementation — it's substrate work. The design brief produced a substrate proposal + bypass-or-park decision; this brief assumes the substrate landed.

## Slice

1. **Confirm Substrate readiness** for cardinality refinement that admits structural flatten on `OptionalOf`. If signal not posted, do not dispatch.
2. **Author flatten substrate consumer** — when type-checking encounters `OptionalOf<OptionalOf<T>>`, the substrate normalizes to `OptionalOf<T>` structurally.
3. **Diagnostic for legitimate distinction.** If the user explicitly wanted to distinguish "absent" vs "present-but-None" semantics, surface that as a typed pattern (e.g., a richer sum type than nested optional) rather than allowing the structural flatten silently. Surface design choice in PR.
4. **Surface syntax (`T??`)** — if the parser accepts `T??`, what does it desugar to? Per design brief, this is a worker's call: either reject (compile error: nested optional not allowed at user surface) or normalize (`T?? = T?`). Surface choice.
5. **Regression tests:**
   - `Option<Option<T>>` accessor patterns lower to `Option<T>` accessor patterns.
   - User-code `T??` either rejected or normalized per Slice §4.
   - Existing single-Option programs stay bit-identical.
6. **DB-8 fixed-point bit-identical.**

## Acceptance

- [ ] Substrate readiness confirmed; cardinality refinement covers structural flatten.
- [ ] Type-checker normalizes `OptionalOf<OptionalOf<T>>` to `OptionalOf<T>` structurally.
- [ ] Surface-syntax `T??` decision made + documented.
- [ ] Regression tests cover normalize / spoofing / bit-identity.
- [ ] DB-8 converges bit-identically.
- [ ] Cross-program signal: lane close → Impossible-Bugs Manager → R2 Release Manager (Goal 4 nested-optional class).
- [ ] `cargo test` / clippy / fmt clean.

## STOP-AND-ESCALATE

- **Substrate readiness signal exists but the substrate does NOT cover structural flatten** — the design brief presumed; verify before slicing. If gap, escalate to Substrate Manager.
- **Cardinality refinement landed differently than design brief assumed** — re-read design brief; this implementation brief may need revision.
- **Surface-syntax `T??` interaction with existing parser** breaks more than expected — surface; that may need Surface Manager coordination.
- **DB-8 drifts** — STOP immediately.

## Non-goals

- Not extending `OptionalOf` semantics beyond flatten.
- Not authoring cardinality refinement substrate (Substrate Manager territory).
- Not addressing other T-ImpossibleBugs classes (sibling worker briefs).

## Cross-program note

- **Producer prerequisite:** Substrate Manager → cardinality refinement substrate (DB-11 alias-`where` closure adjacent).
- **Consumer:** this brief.
- **Downstream signals:** R2 Release Manager (Goal 4 close).

## Reporting

Single PR. Title: `feat(v3): T-ImpossibleBugs nested-optional flatten — structural normalize OptionalOf<OptionalOf<T>>`. Body cites this brief + design brief + substrate signal-receipt + surface-syntax decision + DB-8 disposition.

On merge: signal R2 Release Manager.
