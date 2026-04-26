# T-Substrate sub-lane — Nominal-opaque-for-Secret subset `(M; substrate sub-lane scoping brief)`

> **Substrate Manager program brief.** Scopes the nominal-opaque substrate
> work needed to unblock T-Modeling `Secret<T>` (Goal 2). Per
> [`docs/r2-structure.md`](../r2-structure.md) Substrate Manager
> ownership; not full nominal-type-system substrate — narrowed to the
> `Secret<T>` consumer.
>
> **Producer:** Substrate Manager (this brief).
> **Consumer:** Modeling Manager — `r2-modeling-secret-graduation-worker.md`
> (Wave 3).

## Read first

- **[`THESIS.md`](../../THESIS.md)** §"Enumerable impossible-bug classes" — `Secret<T>` is one of the R2+ Tier 1 thesis claims; structural opacity is the impossible-bug-by-construction guarantee.
- **[`dsl/std/types.dag`](../../dsl/std/types.dag)** — current type system; how named types are namespaces (`feedback_naming_is_aliasing`); how `inhabits` edges work for algebra attachment.
- **[`src/v3/std/substrate.dag`](../../src/v3/std/substrate.dag)** — live substrate authority for `TypeConnective` + Declaration shape.
- **[`src/v3/spec/v3_l1.dag`](../../src/v3/spec/v3_l1.dag)** — sentinel meta-types; precedent for cross-cutting substrate fields like `DeclarationRef`.
- **`feedback_naming_is_aliasing`** — named types are namespaces; the compiler sees through. Nominal opacity is the deliberate exception — the brief's discipline anchor.
- **`feedback_state_space_vs_behavioral_invariants`** — type-level enforcement preferred over API-level enforcement.

## Frame

`Secret<T>` is a thesis-level commitment: a typed wrapper such that consumers cannot read the inner `T` except through specified accessors (e.g., `redact`, `compare_in_constant_time`). The bug class made impossible: accidental disclosure of secret material via naive structural access (logging, serialization, equality comparison, etc.).

**The substrate gap:** today, named types in gunbc are namespaces — the compiler sees through aliases. Nominal opacity is a **deliberate exception** to that discipline; it requires a substrate flag/carrier on declarations that says "this type's structure is hidden from generic consumers; access is gated."

**Scope is narrow:** only what T-Modeling `Secret<T>` needs. Full nominal-type-system substrate (e.g., for arbitrary nominal types with custom invariants, for nominal-typed effects, for general newtype patterns) is **out of scope** — those are separate substrate-capability lanes.

## Pre-author authority audit (mandatory)

**Before designing**, grep `src/v3/std/` + `src/v3/spec/` for:

- existing nominal-opaque flag / carrier on declarations
- existing structural-access-gated patterns (e.g., `phantom`, `opaque`, `sealed` declarations)
- `inhabits` edge usage for nominal-vs-structural distinction
- existing accessor-gating patterns in std (e.g., methods declared in a way that's the ONLY structural access path to a type's interior)

**If audit reveals existing authority sufficient, reframe as consumer migration.**

## Open design questions (worker / Substrate Manager resolves at dispatch)

1. **Carrier shape.** Possible options:
   - **Boolean flag on Declaration** (`is_nominal_opaque: Bool`) — simplest; the compiler reads it during structural-walk and refuses to descend.
   - **New TypeConnective variant** — e.g., `Opaque(T)` — explicit at type-construction. More discipline-aligned per `feedback_state_space_vs_behavioral_invariants` (illegal access is unrepresentable, not flag-checked).
   - **Sealed-accessor pattern** — declaration carries a list of permitted-accessor `DeclarationRef`s; structural walks for any other purpose fail.
   Worker picks; surface choice + reasoning in PR.

2. **Generic-walk discipline.** When a structural lens walks a Dag containing `Secret<T>`, what happens at the opacity boundary? Options:
   - Hard stop (lens emits opacity diagnostic, refuses to descend).
   - Soft stop (lens treats `Secret<T>` as opaque atom; structural walk continues but inner is invisible).
   - Per-lens declaration (each lens declares whether it respects opacity).
   Worker picks; surface in PR.

3. **Accessor gating.** How does the substrate express "only `redact` and `compare_in_constant_time` may read Secret's interior"? Options:
   - Accessor `DeclarationRef` list on the opaque type.
   - Visibility/scope marker on the inner field declarations.
   - Module-system-style export discipline.
   Worker picks; this is the highest-leverage design call.

4. **Generic parameter `T`.** Does `Secret<T>` require all `T` to be opaque, or only `T` reads through Secret accessors are gated? Per thesis discipline: `T` itself stays structural; only access *through Secret* is gated.

## Slice (worker fills at dispatch)

1. **Audit existing nominal-opaque substrate** (per audit section).
2. **Land minimal substrate carrier** in `src/v3/std/` per design choice.
3. **Lower the carrier** — declarations marked nominal-opaque get the substrate fact attached at lowering / parsing.
4. **Coproduct dissolution receipt** for any new TypeConnective variant.
5. **Document generic-walk discipline** in INVARIANTS.md or feedback memory — what does opacity mean for lens consumers?
6. **No consumer migration in this PR** — that's T-Modeling Secret<T>'s job.
7. **Cross-program signal:** on merge, signal Modeling Manager that Secret<T> is dispatchable.

## Acceptance

- [ ] Authority audit receipt recorded.
- [ ] Substrate carrier for nominal-opacity lands in `src/v3/std/` (canonical + Rust mirror).
- [ ] Coproduct dissolution receipt for any new variant.
- [ ] Generic-walk discipline documented (INVARIANTS.md or feedback memory).
- [ ] Accessor-gating mechanism documented in PR body.
- [ ] DB-8 fixed-point converges bit-identically.
- [ ] Cross-program readiness signal posted to Modeling Manager.
- [ ] `cargo test --workspace --exclude v2-compiler-tests` / `cargo clippy --all-targets -- -D warnings` / `cargo fmt --all --check` clean.

## STOP-AND-ESCALATE

- **Scope expansion:** the nominal-opaque substrate generalizes naturally to other nominal types (e.g., `Phantom<T>`, `Sealed<T>`, custom newtypes). If the design surfaces a general-purpose nominal-type-system substrate, that's a re-scope decision, not a quiet expansion.
- **`inhabits` edge interaction.** Nominal opacity may interact with how `inhabits` walks the algebra-attachment graph — surface for design call if so.
- **Generic-walk discipline conflicts with existing lens behavior.** If hard-stop opacity breaks existing lens fixtures (cost, complexity, idempotency), surface; this is a discipline call about lens contracts, not a substrate call.
- **DB-8 drifts** — STOP immediately.

## Non-goals

- **Not full nominal-type-system substrate.** General nominal types, custom newtype patterns, nominal-typed effects — separate lanes.
- Not module-system export-discipline redesign — minimal targeted carrier.
- Not extending Secret<T> semantics beyond the thesis-level commitment.
- Not authoring `Secret<T>` itself — that's T-Modeling consumer territory.

## Cross-program note

- **Producer:** Substrate Manager (this brief).
- **Consumer:** Modeling Manager → `r2-modeling-secret-graduation-worker.md` (gated on this lane's readiness signal).
- **Adjacent:** Impossible-Bugs Manager — `Secret<T>`-leakage is a thesis-level impossible-bug class; the carrier is the substrate dependency for that proof. Heads-up to Impossible-Bugs Manager at landing.

## Reporting

Single PR. Title: `feat(v3): T-Substrate nominal-opaque-for-Secret subset — minimal carrier for T-Modeling Secret<T> consumer`. Body cites this brief + audit receipt + design choice (carrier shape + walk discipline + accessor gating) + cross-program signal.

On merge: signal Modeling Manager that Secret<T> consumer is dispatchable; cc Impossible-Bugs Manager.
