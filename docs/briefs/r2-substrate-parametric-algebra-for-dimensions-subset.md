# T-Substrate sub-lane — Parametric-algebra-for-Dimensions subset `(M; substrate sub-lane scoping brief)`

> **Substrate Manager program brief.** Scopes the parametric-algebra-attachment
> substrate work needed to unblock T-Modeling `Dimension<Carrier>` (Goal 2).
> Per [`docs/r2-structure.md`](../r2-structure.md) Substrate Manager
> ownership; not full parametric-algebra-substrate — narrowed to the
> `Dimension<Carrier>` consumer.
>
> **Producer:** Substrate Manager (this brief).
> **Consumer:** Modeling Manager — `r2-modeling-dimensions-phantom-worker.md`
> (Wave 3).

## Read first

- **[`THESIS.md`](../../THESIS.md)** §"Enumerable impossible-bug classes" — `Dimension<Carrier>` for unit-mismatch impossibility (e.g., adding meters to seconds is a type error). R2+ Tier 1 thesis claim.
- **[`dsl/std/types.dag:211-213`](../../dsl/std/types.dag)** — `type List<T> = FreeMonoid<T>` etc.; algebra attachment via type alias.
- **[`dsl/std/types.dag:65-74`](../../dsl/std/types.dag)** — `kernel_primitives`; algebra-attachment patterns.
- **[`docs/design-substrate-carrier-port-program.md`](../design-substrate-carrier-port-program.md)** — substrate-carrier-port program; if §6a metadata-pick lands a relevant carrier, coordinate.
- **[`src/v3/std/substrate.dag`](../../src/v3/std/substrate.dag)** — live substrate authority; `Instantiation` connective is the natural attachment point for parametric algebra.
- **[`src/v3/std/algebra.dag`](../../src/v3/std/algebra.dag)** — algebra declarations (`Monoid`, `Group`, `OrderedRing`, etc.).
- **`feedback_state_space_vs_behavioral_invariants`** — type-level enforcement preferred; unit-mismatch should be unrepresentable, not validated at runtime.
- **`feedback_naming_is_aliasing`** — phantom-parameter discipline: the dimension parameter is a namespace marker, not a structural carrier of value.

## Frame

`Dimension<Carrier>` is a thesis-level commitment: a typed wrapper such that two `Dimension<Meters, f64>` and `Dimension<Seconds, f64>` are structurally distinct types — adding them is a type error, even though both wrap `f64`. The bug class made impossible: unit-arithmetic mistakes (NASA Mars Climate Orbiter–class).

**The substrate gap:** today, `Instantiation` carries template arguments structurally — `OrderedRing<Word64>` resolves via the substitution stack. But there's no substrate fact saying "this template parameter is a *phantom dimension marker*, not a structural value carrier" — so `Dimension<Meters, f64>` and `Dimension<Seconds, f64>` would unify if substitution treats Meters/Seconds as ordinary types.

**The fix:** parametric-algebra-attachment substrate that distinguishes structural template parameters (consumed by the type's value layout) from phantom-dimension parameters (consumed only for type-discrimination at the algebra attachment point).

**Scope is narrow:** only what T-Modeling `Dimension<Carrier>` needs. Full parametric-algebra-substrate (e.g., for arbitrary phantom-typed wrappers, for general type-level computation, for higher-kinded algebra attachment) is **out of scope** — separate lanes.

## Pre-author authority audit (mandatory)

**Before designing**, grep `src/v3/std/` + `src/v3/spec/` for:

- existing phantom-parameter / phantom-type carriers
- `Instantiation` field shape — is there an existing distinction between structural and phantom arguments?
- algebra-attachment patterns: how does `Int64 = OrderedRing<Word64>` differ structurally from `Dimension<Meters, f64>`?
- existing higher-kinded patterns in std/ (Functor, Monad, etc.) and how they handle phantom parameters

**If audit reveals existing authority sufficient (e.g., the Instantiation argument shape already supports phantom-marker semantics with a flag), reframe as consumer migration.**

## Open design questions (worker / Substrate Manager resolves at dispatch)

1. **Carrier shape.** Possible options:
   - **Field on `Instantiation` argument** — each `TemplateArgument` flags `is_phantom: Bool` (or richer enum); substitution-stack walk respects the flag.
   - **Separate `PhantomParameter` declaration variant** — the template's parameter declaration itself names which parameters are phantom; instantiation respects the parameter's declared kind.
   - **Algebra-attachment carrier** — `Dimension<Meters, f64>` instantiation carries an `algebra: AlgebraRef` field that Meters provides; type-discrimination follows the algebra-ref equivalence.
   Worker picks; surface choice + reasoning in PR.

2. **Type-equivalence rule.** Two `Dimension<Meters, f64>` instantiations should unify (same dimension); `Dimension<Meters, f64>` vs `Dimension<Seconds, f64>` should NOT. The substrate must encode this. Options:
   - Phantom parameter equality is structural (Meters declaration ID == Meters declaration ID).
   - Phantom parameter equality is via algebra-attachment (Meters provides `LengthAlgebra`; equality is by AlgebraRef).
   Worker picks.

3. **Algebra-method dispatch.** When a user writes `meters_a + meters_b`, what algebra-attachment substrate routes the `+` to the correct algebra (additive group of Length)? Options:
   - Algebra resolved from the phantom parameter's declaration directly.
   - Algebra resolved from the inner carrier (`f64`'s additive group), with type-discrimination preventing cross-dimension mix.
   - Both — phantom parameter forms the cross-dimension barrier, inner carrier provides the operation.
   Worker picks; this is the highest-leverage design call.

4. **Lifting / coercion.** Does the substrate admit `f64 → Dimension<Dimensionless, f64>` lifts, or are dimensions strictly nominal at construction? Per discipline: strictly nominal preferred (no implicit lifts).

## Slice (worker fills at dispatch)

1. **Audit existing parametric-algebra substrate** (per audit section).
2. **Land minimal substrate carrier** for phantom-parameter discrimination.
3. **Document algebra-attachment dispatch** mechanism in PR body.
4. **Coproduct dissolution receipt** for any new `Instantiation` argument variant.
5. **No consumer migration in this PR** — T-Modeling Dimensions's job.
6. **Cross-program signal:** on merge, signal Modeling Manager that Dimensions is dispatchable.

## Acceptance

- [ ] Authority audit receipt recorded.
- [ ] Substrate carrier for parametric-algebra-attachment lands in `src/v3/std/` (canonical + Rust mirror).
- [ ] Coproduct dissolution receipt for any new variant.
- [ ] Type-equivalence rule documented.
- [ ] Algebra-method dispatch path documented.
- [ ] DB-8 fixed-point converges bit-identically.
- [ ] Cross-program readiness signal posted to Modeling Manager.
- [ ] `cargo test --workspace --exclude v2-compiler-tests` / `cargo clippy --all-targets -- -D warnings` / `cargo fmt --all --check` clean.

## STOP-AND-ESCALATE

- **Scope expansion:** the parametric-algebra substrate generalizes naturally to higher-kinded algebra attachment (e.g., `Functor<F>` where F is a type constructor). If the design surfaces a general-purpose higher-kinded substrate, that's a re-scope decision.
- **§6a metadata-pick interaction.** The `docs/design-substrate-carrier-port-program.md §6a` per-method-metadata pick may land a relevant carrier; coordinate with PM/R2 Release Manager — surface at audit time.
- **Algebra resolution requires runtime dispatch, not static.** If the design needs runtime algebra dispatch for `Dimension`, surface — that's a discipline call about whether algebra attachment is fully static (preferred) or admits dynamic dispatch.
- **DB-8 drifts** — STOP immediately.

## Non-goals

- **Not full higher-kinded algebra substrate.** Higher-kinded patterns, type-level computation, GADT-style discrimination — separate lanes.
- Not authoring `Dimension<Carrier>` itself — that's T-Modeling consumer.
- Not extending Instantiation beyond what phantom-parameter discrimination needs.

## Cross-program note

- **Producer:** Substrate Manager (this brief).
- **Consumer:** Modeling Manager → `r2-modeling-dimensions-phantom-worker.md` (gated on this lane's readiness signal).
- **Adjacent:** Impossible-Bugs Manager — unit-mismatch is a thesis-level impossible-bug class; the carrier is the substrate dependency for that proof. Heads-up at landing.
- **Coordination:** §6a metadata-pick (R2 Release Manager) may share substrate territory with this lane's algebra-attachment work; cross-check at audit time.

## Reporting

Single PR. Title: `feat(v3): T-Substrate parametric-algebra-for-Dimensions subset — minimal phantom-parameter carrier for T-Modeling Dimensions consumer`. Body cites this brief + audit receipt + design choice (carrier shape + type-equivalence + algebra dispatch) + cross-program signal.

On merge: signal Modeling Manager that Dimensions consumer is dispatchable; cc Impossible-Bugs Manager + R2 Release Manager (§6a coordination).
