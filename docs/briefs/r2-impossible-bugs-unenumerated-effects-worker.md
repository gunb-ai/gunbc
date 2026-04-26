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
- **Closed-system effects discipline** — 5-behavior compositional-fold table; the lens IS the same shape complexity/idempotency/termination lenses already use. The discipline is captured in the design doc (`t-impossiblebugs-unenumerated-effects-design.md`) §Q1-Q3; consult that as canonical authority.
- **[`src/v3/std/effects.dag`](../../src/v3/std/effects.dag)** — `OperationEffect` enum; live-state authority for the Q5.5 retain-vs-retire decision.
- **[`docs/briefs/t-impossiblebugs-effects-lens-implementation-audit.md`](t-impossiblebugs-effects-lens-implementation-audit.md)** (if authored per #810 SHIP_WITH_DEBT recommendation) — implementation/audit follow-up brief; if not authored, this brief subsumes its scope.

## Frame

Per design doc Q1-Q3: effects compose from the 5 behaviors (`Value | Transform | Branch | Loop | Bind`) the same way complexity, idempotency, termination do. A function is "effectful" iff its body composes operations with write-shaped type signatures (returns modified resource).

Per Q5.5: the existing `OperationEffect` taxonomy (`Read | Upsert | Create | Append | Delete`) may be retired entirely (path ii — preferred default) or retained as a normalized derived view (path i). The audit decides:
- All effectful primitives in `dsl/std/` + `dsl/extdeps/` have type signatures that structurally derive the right effect classification → **path (i)** retain as normalized view.
- At least one effectful primitive requires a hand-declared tag → **path (ii)** retire (the existence-proof shows the taxonomy is parallel-representation).

Per design doc default: retire (path ii).

## Slice (per design doc Q6 — 8 reqs)

1. **Effects lens at `src/v3/lenses/effect_enumeration.dag`** — parallel to `cost.dag`. Walks the 5-behavior structure (Value / Transform / Branch / Loop / Bind); **anchors on operation type-signature shape**, not on hand-declared OperationEffect tags; composes effect classification per the design doc Q2 table; surfaces structural-fact output (no annotation comparison, no parallel-taxonomy lookup).
2. **Audit-as-existence-check** — verify every effectful primitive in `dsl/std/` + `dsl/extdeps/` has a type signature that structurally derives the right effect classification (returned-modified-resource → write-shaped; returns-derived-value-only → read-shaped). **NOT "tag every primitive."** Per design doc Q5.5: any primitive requiring a hand-declared `OperationEffect` tag because its signature doesn't structurally reveal the effect IS the existence-proof for path (ii) — taxonomy retirement.
3. **Resource-threading discipline applied to existing primitives** — primitives that violate the discipline today (e.g., logging that returns Unit instead of `log.info(msg, log: LogFile) → LogFile'`) get reshaped per the audit's findings. Foundation step toward signature-shape coverage.
4. **Redundancy lens** — referential-transparency proof for repeated identical reads with no intervening write. Emits compile-time `RedundantReadError` (Tier 1, not Tier 3). Per design doc Director's aggressive reading.
5. **`reread(key)` primitive in std/** — for legitimate re-read cases. Structurally tagged. Single authority for re-read intent (per design doc + #808 closing comment: extend `reread` rather than adding parallel primitives if cache-invalidation / transactional-refresh ever need similar affordance).
6. **Transactional-pattern lens** — derived structural fact from Bind composition + typed transaction primitives (`Transaction → Transaction'`). The lens walks the Bind chain and recognizes the begin-modify-commit shape.
7. **Asymmetric-tightening worked example** in PR body — concrete demonstration that caller-side constraint via structural type matching rejects a callee whose body composes write-shaped operations beyond the caller's pinned set. Per design doc + claude review observation: the one place declaration-shaped surface re-enters deserves a worked example.
8. **Smoke + integration tests:**
   - Function with multiple typed-effect operations produces correct effect-set structural fact (derived from signature shape, not from tag lookup).
   - Function with redundant read of same key without intervening write → compile error.
   - Function with `reread()` on the same key → compiles cleanly (legitimate re-read structurally tagged).
   - Transactional-pattern function compiles + lens recognizes begin-modify-commit shape.
   - Caller pinning effect-set ⊆ {read-shaped} rejects callee composing write-shaped operations.

**Path (i) vs (ii) decision** — falls out of Slice §2 audit; surface receipt in PR body. **Default per design doc: path (ii) retire.** If audit produces existence-proof for path (i) (every primitive's signature derives cleanly), surface for Director re-decision per design doc Q5.5 STOP.

## Acceptance

- [ ] Effects lens at `src/v3/lenses/effect_enumeration.dag` (req 1); lens output observable via standard lens-output infrastructure.
- [ ] Audit-as-existence-check (req 2) completed; receipt recorded in PR body (which primitives audited; path (i) or (ii) verdict with structural justification).
- [ ] Resource-threading discipline (req 3) applied to existing primitives where audit found violations.
- [ ] Redundancy lens (req 4) emits compile-time `RedundantReadError` for same-key re-read without intervening write.
- [ ] `reread(key)` primitive (req 5) authored in std/.
- [ ] Transactional-pattern lens (req 6) recognizes begin-modify-commit shape via Bind composition.
- [ ] Asymmetric-tightening worked example (req 7) in PR body.
- [ ] Smoke + integration tests (req 8) cover read / write / redundancy compile-error / `reread()` / transactional / asymmetric-tightening rejection.
- [ ] Lens reports structural-coverage-gap diagnostics on Q4.5 P1 bypass surfaces (gap becomes visible, not silent).
- [ ] DB-8 fixed-point converges bit-identically.
- [ ] Cross-program signal: Impossible-Bugs Manager → R2 Release Manager (Goal 4 unenumerated-effects class).
- [ ] `cargo test` / clippy / fmt clean.

## STOP-AND-ESCALATE (per design doc Q6 STOPs)

- **OperationEffect retirement decision (path (i) vs (ii)) is NOT a STOP — the audit verdict IS this lane's deliverable.** Slice §2 + Acceptance §2 own producing the verdict (path-i existence-proof OR path-ii existence-proof) with structural justification + per-primitive receipt in PR body. The follow-up — actual `OperationEffect` enum retirement (path ii: enum + `derive_op_effect` + `idempotency.dag` re-anchor) or normalized-derived-view authoring (path i) — dispatches as a **sibling sub-lane** to Impossible-Bugs Manager based on the verdict; Director does not need to be in the loop for the verdict itself. **STOP only if** the audit reveals a primitive whose signature shape can't be made to derive cleanly **even after resource-threading migration (Slice §3)** — i.e., a substrate gap not anticipated by the design doc Q5.5 binary. That's an unscoped substrate-shape question, not a retain-vs-retire pick.
- **Redundancy proof needs a `pure: Bool` carrier** — if the lens can't distinguish pure from impure Transforms inline, STOP. May need a sibling carrier on Transform targets. (Note: the deeper closed-system framing is that "pure" should also be derivable from signature shape — pure functions don't return modified resources — so this STOP may itself dissolve under further design.)
- **Asymmetric-tightening structural gap** — if caller can't actually pin effect-set constraints structurally today (i.e., the type system doesn't yet express "I require callee body's signature-shape composition ⊆ {read-shaped}"), STOP — that's its own substrate sub-lane.
- **Q4.5 P1 (extdeps typed-primitive bypass) — surfaced via lens findings**: lens reports structural-coverage-gap on `dsl/extdeps/llm/openai.dag:92-110`, `anthropic.dag:104-124`, `github/auth.dag:13-24` (and any others the audit finds). **NOT a STOP**; this is the lens delivering its closed-system-foundation-gap-visibility value. Director routes P1 closure to a dedicated extdeps-typed-primitive-consumption lane. Surface findings in PR body.
- **`reread()` ergonomics surface real legitimate-re-read patterns it doesn't cover cleanly** — per #808 closing comment, extend the primitive (or add a structurally-tagged sibling) rather than soften the compile-error stance.
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
