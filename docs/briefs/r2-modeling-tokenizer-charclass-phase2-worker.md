# T-Modeling — Tokenizer charclass phase-2 worker brief `(M; consumer of T-Substrate ValueBody-list/sum)`

> **Worker brief.** Reports through Modeling Manager (post-R2 spin-up) /
> Director (pre-spin-up). T-Modeling Goal 2 item — completes the
> tokenizer charclass migration started in phase-1.
>
> **Gated on:** Substrate Manager readiness signal for the T-Substrate
> ValueBody-list/sum sub-lane (worker brief #790).
> **Do not dispatch until that signal posts.**

## Read first

- **[#790 worker brief](https://github.com/gunb-ai/gunbc/pull/790)** — T-Substrate ValueBody-list/sum sub-lane producer brief.
- **[`src/v3/std/tokenize.dag`](../../src/v3/std/tokenize.dag)** — tokenizer authority; phase-1 lands the structural shape, phase-2 retypes consumers to `Char` / `List<Char>` / `CharClass`.
- **[#662](https://github.com/gunb-ai/gunbc/pull/662)** — "tokenize: reframe character-level scaffold as consumption gap" (merged); confirm phase-1 baseline.
- **[`docs/thesis/the-substrate-two-coordinated-shapes.md`](../thesis/the-substrate-two-coordinated-shapes.md)** — connective vocabulary; `Cardinality` / `Disj` semantics for charclass sum-types.
- **[`dsl/std/unicode.dag`](../../dsl/std/unicode.dag)** — unicode authority; charclass dependency.

## Frame

Phase-1 (per #662 retrospective and ROADMAP entries) reframed the character-level scaffold as a consumption gap: tokenizer doesn't yet consume canonical `Char` / `List<Char>` / `CharClass` types because the substrate didn't admit `ValueBody::List` / sum-typed value bodies.

Producer (T-Substrate ValueBody-list/sum sub-lane #790) lands that substrate. This brief migrates tokenizer consumers to retype against `Char` / `List<Char>` / `CharClass`, dropping any host-string scaffolding.

After this lane closes: tokenizer charclass is structurally typed end-to-end; lens consumers (cost, complexity) walk charclass declarations as canonical `.dag` values; Engine sharpened-(b) (cross-lane consumer of the same producer) becomes dispatchable independently.

## Slice

1. **Confirm Substrate readiness signal** from #790 producer.
2. **Audit phase-1 baseline.** Confirm what phase-1 (#662 retrospective) landed — the structural shape — and what phase-2 must migrate (the consumer side).
3. **Migrate tokenizer charclass consumers** to consume `Char` / `List<Char>` / `CharClass` typed structures instead of host-string scaffolding.
4. **Drop host-string scaffolds** that phase-1 left as bridges. Surface what's removed in PR body.
5. **Regression tests:**
   - Tokenizer produces `CharClass` typed outputs for charclass tokens.
   - Lens fixtures (cost, complexity) walk `CharClass` declarations canonically.
   - Spoofing: host-string fallbacks no longer resolve charclass dispatch.
6. **DB-8 fixed-point bit-identical.**

## Acceptance

- [ ] Tokenizer charclass consumers retyped to `Char` / `List<Char>` / `CharClass`.
- [ ] Phase-1 host-string scaffolds dropped (preferred), **OR** any residual scaffold has the full tracked-bridge triple: (1) **documented** in-line at the residual site explaining what's bridged + why; (2) **bounded** — explicit ROADMAP debt row with named owner / lane; (3) **named dissolution trigger** — concrete event/PR that closes it (not "TBD" or "future cleanup"). Per `INVARIANTS.md` P5; B4.4 shape (b) is the precedent for the triple. Residuals without all three are a STOP-AND-ESCALATE.
- [ ] Regression tests pass; lens fixtures walk canonical types.
- [ ] DB-8 converges bit-identically.
- [ ] Cross-program signal: lane close → Modeling Manager → R2 Release Manager (Goal 2 charclass-phase-2 item).
- [ ] **Independent of Engine sharpened-(b)** — this brief shares producer with Grounding Manager's Engine work but does not depend on Engine landing first.
- [ ] `cargo test` / clippy / fmt clean.

## STOP-AND-ESCALATE

- **Producer brief landed without `ValueBody::List` for the shapes tokenizer needs** — surface; tokenizer consumer can't migrate without the structural support.
- **`std.unicode` dependency surfaces.** If charclass-phase-2 needs `std.unicode` declarations not yet in std, surface — that's a producer-brief expansion call.
- **Engine sharpened-(b) (Grounding Manager's consumer of same producer) lands first and surfaces a different consumer pattern** — coordinate cross-program; the substrate may need refinement.
- **DB-8 drifts** — STOP immediately.

## Non-goals

- Not authoring full Unicode tables — minimal what tokenizer needs.
- Not extending tokenizer beyond charclass — separate work.
- Not authoring the producer carrier (#790's job).
- Not engine sharpened-(b) consumer migration (Grounding Manager's territory; same producer, different consumer).

## Cross-program note

- **Producer:** Substrate Manager → ValueBody-list/sum sub-lane (#790).
- **Consumer (this brief):** Modeling Manager.
- **Sibling consumer:** Grounding Manager → Engine sharpened-(b) — independent dispatch; share producer signal but not consumer code.
- **Downstream signal:** R2 Release Manager (Goal 2 close).

## Reporting

Single PR. Title: `feat(v3): T-Modeling tokenizer charclass phase-2 — retype consumers to Char/List<Char>/CharClass per ValueBody-list/sum substrate`. Body cites this brief + #790 producer + #662 phase-1 retrospective + signal-receipt + DB-8 disposition.

On merge: signal R2 Release Manager.
