# T-Modeling — `Secret<T>` graduation worker brief `(M; consumer of T-Substrate nominal-opaque-for-Secret)`

> **Worker brief.** Reports through Modeling Manager (post-R2 spin-up) /
> Director (pre-spin-up). T-Modeling Goal 2 item.
>
> **Gated on:** Substrate Manager readiness signal for
> [`r2-substrate-nominal-opaque-for-secret-subset.md`](r2-substrate-nominal-opaque-for-secret-subset.md).
> **Do not dispatch until that signal posts.**

## Read first

- **[`docs/briefs/r2-substrate-nominal-opaque-for-secret-subset.md`](r2-substrate-nominal-opaque-for-secret-subset.md)** — producer brief; defines the nominal-opacity carrier this worker consumes.
- **[`THESIS.md`](../../THESIS.md)** §"Enumerable impossible-bug classes" — `Secret<T>` is a Tier 1 thesis claim.
- **[`src/v3/std/types.dag`](../../src/v3/std/types.dag)** — current type-aliasing patterns; the `Secret<T>` declaration lands here.
- **`feedback_state_space_vs_behavioral_invariants`** — type-level enforcement preferred.

## Frame

Producer landed the nominal-opacity substrate carrier. This brief authors `Secret<T>` itself as a typed wrapper that consumes the carrier — declarations of `Secret<T>` are nominal-opaque, with specified accessors (`redact`, `compare_in_constant_time`) being the only structural-access path to inner `T`.

After this lane closes, naive operations (logging, equality, serialization, structural pattern-match) on `Secret<T>` produce typed diagnostics at compile time, not runtime warnings.

## Slice

1. **Confirm Substrate readiness signal.** Producer carrier in `src/v3/std/` with Rust mirror parity.
2. **Author `Secret<T>` declaration** in `src/v3/std/secret.dag` (or appropriate location). Apply the nominal-opacity carrier per producer brief's contract.
3. **Author the gated accessors:**
   - `redact: Secret<T> -> RedactedString` (or worker-equivalent; surface choice)
   - `compare_in_constant_time: Secret<T>, Secret<T> -> Bool`
   - Any others surfaced during producer-brief design.
4. **Diagnostic for non-gated access.** When a structural lens or operation tries to read inside `Secret<T>` outside the gated accessors, emit typed diagnostic. Per `feedback_fail_closed_discipline` C-8.
5. **Regression tests:**
   - `Secret<T>` declared; gated accessors work.
   - Naive `==` / structural-walk / pattern-match on `Secret<T>` produces typed diagnostic.
   - Spoofing test: a non-Secret nominal type does NOT receive Secret-style opacity by accident.
6. **DB-8 fixed-point bit-identical.**

## Acceptance

- [ ] `Secret<T>` declared in `src/v3/std/`; consumes producer's nominal-opacity carrier.
- [ ] Gated accessors authored.
- [ ] Diagnostic for non-gated access emitted per C-8.
- [ ] Regression tests cover positive + negative + spoofing.
- [ ] DB-8 converges bit-identically.
- [ ] Cross-program signal: lane close → Modeling Manager → R2 Release Manager (Goal 2 Secret<T> item) + Impossible-Bugs Manager (thesis claim covered).
- [ ] `cargo test` / clippy / fmt clean.

## STOP-AND-ESCALATE

- **Producer carrier shape doesn't admit gated accessors cleanly.** Surface; producer brief may need extension.
- **Generic-walk discipline (per producer brief)** breaks existing lens fixtures (cost, complexity). Surface; this is a discipline call about lens contracts vs opacity.
- **Accessor-gating mechanism** doesn't fit modeled access patterns (e.g., needs runtime dispatch). Surface; nominal-opacity may need substrate refinement.
- **DB-8 drifts** — STOP immediately.

## Non-goals

- Not extending `Secret<T>` to general nominal-typed effects.
- Not authoring the substrate carrier (producer brief's job).
- Not module-system export-discipline redesign.
- Not authoring full encryption / cryptographic operations on `Secret<T>` — only the structural opacity layer.

## Cross-program note

- **Producer:** Substrate Manager → nominal-opaque-for-Secret.
- **Consumer:** this brief.
- **Downstream signals:** R2 Release Manager (Goal 2 close); Impossible-Bugs Manager (thesis claim covered structurally).

## Reporting

Single PR. Title: `feat(v3): T-Modeling Secret<T> graduation — typed wrapper consuming nominal-opacity substrate; gated accessors`. Body cites this brief + producer brief + signal-receipt + DB-8 disposition.

On merge: signal R2 Release Manager + Impossible-Bugs Manager.
