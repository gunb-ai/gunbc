# T-ImpossibleBugs — Unenumerated effects implementation worker brief `(M; closed-system structural derivation)`

> **Worker brief.** Reports through Impossible-Bugs Manager (post-R2
> spin-up) / Director (pre-spin-up). T-ImpossibleBugs Goal 4 class 3
> of 3.
>
> **No substrate gate.** Per closed-system effects framing in
> [`docs/briefs/t-impossiblebugs-unenumerated-effects-design.md`](t-impossiblebugs-unenumerated-effects-design.md)
> (merged via #808), this is **dissolved by construction** — effects
> derive structurally from typed primitive composition; there is no
> annotation surface, no declared-vs-inferred lens, no parallel taxonomy.
> The implementation work is **audit-as-existence-check** + lens
> validation, not substrate carrier landing.

## Read first

- **[`docs/briefs/t-impossiblebugs-unenumerated-effects-design.md`](t-impossiblebugs-unenumerated-effects-design.md)** — design doc (merged #808). The 5-behavior compositional-fold mechanism + worked examples + Q5.5 OperationEffect retain-vs-retire framing. Read in full.
- **[`THESIS.md`](../../THESIS.md)** R2+ unenumerated-effects bullet — Tier 1 commitment ("dissolved by construction"); type-signature shape IS the effect.
- **[`feedback_closed_system_effects.md`](../../../.claude/projects/-Users-briansrls-gunbc/memory/feedback_closed_system_effects.md)** — closed-system effects discipline; 5-behavior fold table.
- **[`src/v3/std/effects.dag`](../../src/v3/std/effects.dag)** — `OperationEffect` enum; live-state authority for the Q5.5 retain-vs-retire decision.
- **[`docs/briefs/t-impossiblebugs-effects-lens-implementation-audit.md`](t-impossiblebugs-effects-lens-implementation-audit.md)** (if authored per #810 SHIP_WITH_DEBT recommendation) — implementation/audit follow-up brief; if not authored, this brief subsumes its scope.

## Frame

Per design doc Q1-Q3: effects compose from the 5 behaviors (`Value | Transform | Branch | Loop | Bind`) the same way complexity, idempotency, termination do. A function is "effectful" iff its body composes operations with write-shaped type signatures (returns modified resource).

Per Q5.5: the existing `OperationEffect` taxonomy (`Read | Upsert | Create | Append | Delete`) may be retired entirely (path ii — preferred default) or retained as a normalized derived view (path i). The audit decides:
- All effectful primitives in `dsl/std/` + `dsl/extdeps/` have type signatures that structurally derive the right effect classification → **path (i)** retain as normalized view.
- At least one effectful primitive requires a hand-declared tag → **path (ii)** retire (the existence-proof shows the taxonomy is parallel-representation).

Per design doc default: retire (path ii).

## Slice

1. **Audit-as-existence-check.** Walk every effectful primitive in `dsl/std/` + `dsl/extdeps/`:
   - Operations that return modified-resource → write-shaped → effect derived structurally.
   - Operations that return derived-value-only → read-shaped → effect derived structurally.
   - Operations that return Unit but mutate external state (e.g., logging that hides the file-write) — this is the existence-proof for path (ii) failure mode. **Resource-threading discipline:** `log.info(msg, log: LogFile) → LogFile'` makes the effect structural; existing primitives that violate this need migration.
2. **Decide retain vs retire** based on audit:
   - Path (i): every primitive's type signature derives effect structurally; OperationEffect stays as normalized view computed from structure.
   - Path (ii): one or more primitives need migration (e.g., logging that returns Unit needs to thread a LogFile parameter). The taxonomy is structurally derivable post-migration; retire as parallel-representation.
3. **Implement the lens.** A structural lens that walks composition and reports effect set. Per design doc §Q1-Q3: the lens IS a compositional fold over the 5 behaviors; the same shape complexity/idempotency lenses already use.
4. **Implement redundancy detection** (Director's aggressive reading per design doc §Q3): redundant operations (reads of the same key with no intervening write-effect on that resource) are structurally provable as identical via referential transparency, and **rejected at compile time**. Legitimate re-read uses an explicit `reread()` primitive.
5. **Transactional grouping.** Per design doc: derived structural fact from Bind composition + typed transaction primitives (`Transaction → Transaction'`). The lens walks the Bind chain and recognizes the begin-modify-commit shape.
6. **Lens output** integrates into existing v3 lens infrastructure; consumers can query "what effects does this function compose?".
7. **Regression tests:**
   - Function with read-only signature composes only read-shaped operations → lens reports `{Read}`.
   - Function with write-signature returning modified resource → lens reports `{Write}` or equivalent.
   - Function with redundant read of same key without intervening write → compile error.
   - Function with `reread()` on the same key → compiles cleanly (legitimate re-read tagged).
   - Transactional-pattern function compiles + lens recognizes the begin-modify-commit shape.

## Acceptance

- [ ] Audit-as-existence-check completed; receipt recorded in PR body (which primitives audited, retain or retire decision, with structural-justification).
- [ ] Path (i) or path (ii) decision landed:
  - **(i):** OperationEffect retained as normalized view; no primitive declares a tag, all derive structurally.
  - **(ii):** OperationEffect retired; consumers walk type signatures directly; primitives that needed migration (e.g., logging) migrated to thread typed resources.
- [ ] Effects lens implemented as compositional fold over 5 behaviors per design doc.
- [ ] Redundancy detection rejects same-key re-read without intervening write or `reread()` primitive.
- [ ] Transactional pattern recognized structurally.
- [ ] Regression tests cover read / write / redundancy compile-error / `reread()` / transactional patterns.
- [ ] DB-8 fixed-point converges bit-identically.
- [ ] Cross-program signal: Impossible-Bugs Manager → R2 Release Manager (Goal 4 unenumerated-effects class).
- [ ] `cargo test` / clippy / fmt clean.

## STOP-AND-ESCALATE

- **Audit reveals a primitive whose effect can't be derived from type signature even after resource-threading migration** — surface; the closed-system claim has a leak. Re-read design doc Q5.5 audit framing.
- **`reread()` primitive's ergonomics surface real legitimate-re-read patterns it doesn't cover cleanly** — per #808 closing comment, extend the primitive (or add sibling `refresh()`) rather than soften the compile-error stance.
- **Transactional pattern recognition requires substrate work beyond Bind composition + typed transaction primitives** — surface; design doc may need extension.
- **Redundancy detection produces false positives on legitimate code** — per #808 disposition, this is the load-bearing aggressive reading; surface for design-call review, don't soften unilaterally.
- **DB-8 drifts** — STOP immediately.

## Non-goals

- Not extending the 5-behavior set (substrate territory; out of scope for this lens).
- Not authoring `OperationEffect` extensions (path ii retires; path i keeps as derived view, no extensions).
- Not addressing other T-ImpossibleBugs classes.
- Not extending lens infrastructure beyond what this lens needs.

## Cross-program note

- **No substrate prerequisite** — the closed-system framing is the substrate (already landed via 5-behavior Behavior enum + typed substrate carriers).
- **Producer:** this brief produces the lens.
- **Consumer:** R2 Release Manager (Goal 4 close); future tooling that wants to query effect composition.
- **Adjacent:** Substrate Manager (B4 Identity-Carrier work) — Substrate's resource-threading discipline (e.g., `LogFile` carriers) may need landing if path (ii) audit surfaces it. Cross-program coordination at audit time.

## Reporting

Single PR. Title: `feat(v3): T-ImpossibleBugs unenumerated effects — closed-system lens + audit-as-existence-check + redundancy fail-closed`. Body cites this brief + design doc (#808) + audit receipt + path (i/ii) decision + DB-8 disposition.

On merge: signal R2 Release Manager + close THESIS R2+ unenumerated-effects bullet.
