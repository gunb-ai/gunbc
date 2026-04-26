# T-Modeling — `Dimension<Carrier>` phantom-parameter worker brief `(M; consumer of T-Substrate parametric-algebra-for-Dimensions)`

> **Worker brief.** Reports through Modeling Manager (post-R2 spin-up) /
> Director (pre-spin-up). T-Modeling Goal 2 item.
>
> **Gated on:** Substrate Manager readiness signal for
> [`r2-substrate-parametric-algebra-for-dimensions-subset.md`](r2-substrate-parametric-algebra-for-dimensions-subset.md).
> **Do not dispatch until that signal posts.**

## Read first

- **[`docs/briefs/r2-substrate-parametric-algebra-for-dimensions-subset.md`](r2-substrate-parametric-algebra-for-dimensions-subset.md)** — producer brief; defines the phantom-parameter carrier this worker consumes.
- **[`THESIS.md`](../../THESIS.md)** §"Enumerable impossible-bug classes" — unit-mismatch impossibility as Tier 1 thesis claim.
- **[`src/v3/std/types.dag`](../../src/v3/std/types.dag)** + **[`src/v3/std/algebra.dag`](../../src/v3/std/algebra.dag)** — algebra-attachment patterns.
- **`feedback_state_space_vs_behavioral_invariants`** — unit-mismatch unrepresentable, not validated.

## Frame

Producer landed the phantom-parameter discrimination substrate. This brief authors `Dimension<Unit, Carrier>` itself — a typed wrapper such that two `Dimension<Meters, f64>` and `Dimension<Seconds, f64>` are structurally distinct types; arithmetic that would mix them is a type error at compile time.

After this lane closes, common dimension types (Meters, Seconds, Kilograms, Amperes — the SI base units, plus a small set of derived units) are declared in `src/v3/std/dimensions.dag` (or equivalent); user code mixing units produces typed diagnostics.

## Slice

1. **Confirm Substrate readiness signal.** Producer carrier in `src/v3/std/`.
2. **Author `Dimension<Unit, Carrier>` declaration** consuming the phantom-parameter carrier per producer brief's contract.
3. **Author core unit declarations** (worker picks scope; surface in PR):
   - SI base units: `Meters`, `Seconds`, `Kilograms`, `Amperes`, `Kelvin`, `Moles`, `Candela`.
   - Common derived: `Newtons` (kg·m/s²), `Joules`, `Hertz`. Possibly defer derived to a follow-up.
4. **Algebra-method dispatch** per producer brief's design choice. Add: `add`, `sub`, `mul_scalar`, `div_scalar` over same-dimension `Dimension<Unit, Carrier>`. Cross-dimension operations are typed errors.
5. **Diagnostic for unit-mismatch.** Add `meters + seconds` produces typed diagnostic. Per C-8.
6. **Regression tests:**
   - `meters + meters` produces `meters` (same dimension).
   - `meters + seconds` produces typed diagnostic at compile time.
   - `meters * scalar(2.0)` produces `meters` (scalar lifts cleanly).
   - Spoofing test: a non-Dimension wrapper with similar shape doesn't accidentally enforce unit-mismatch.
7. **DB-8 fixed-point bit-identical.**

## Acceptance

- [ ] `Dimension<Unit, Carrier>` declared in `src/v3/std/`; consumes producer's phantom-parameter carrier.
- [ ] Core SI base units declared (worker picks scope; surface).
- [ ] Algebra-method dispatch per producer brief's design.
- [ ] Unit-mismatch diagnostic per C-8.
- [ ] Regression tests cover positive + negative + scalar-lift + spoofing.
- [ ] DB-8 converges bit-identically.
- [ ] Cross-program signal: R2 Release Manager (Goal 2 Dimensions item) + Impossible-Bugs Manager (thesis claim).
- [ ] `cargo test` / clippy / fmt clean.

## STOP-AND-ESCALATE

- **Producer carrier shape doesn't admit phantom-parameter algebra-method dispatch cleanly.** Surface; producer brief may need extension.
- **Algebra-method dispatch path requires runtime dispatch.** Per producer brief's STOP, surface — discipline call about static-vs-dynamic algebra dispatch.
- **Derived units (Newtons = kg·m/s²) require type-level computation** (multiplication of phantom parameters). If the producer's substrate doesn't support this, derived units defer to a follow-up; surface.
- **DB-8 drifts** — STOP immediately.

## Non-goals

- Not authoring complete SI / Imperial unit system — minimal core + extensible.
- Not authoring conversion functions (Meters → Feet) in this brief — those are user-space follow-ups.
- Not authoring the substrate carrier (producer's job).
- Not extending `Dimension<Unit, Carrier>` to higher-kinded patterns.

## Cross-program note

- **Producer:** Substrate Manager → parametric-algebra-for-Dimensions.
- **Consumer:** this brief.
- **Downstream signals:** R2 Release Manager (Goal 2 close); Impossible-Bugs Manager (thesis claim).

## Reporting

Single PR. Title: `feat(v3): T-Modeling Dimension<Unit, Carrier> phantom-parameter — typed wrapper + core SI units consuming parametric-algebra substrate`. Body cites this brief + producer brief + signal-receipt + DB-8 disposition + scope of authored units.

On merge: signal R2 Release Manager + Impossible-Bugs Manager.
