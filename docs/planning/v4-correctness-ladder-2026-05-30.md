# v4 Emit Correctness: Standards, Current Gating, and the Ladder Forward

> **Status:** PLANNING DRAFT — operator sign-off requested on §6 (ladder), §7 (sequencing), §8 (decisions).
> **Date:** 2026-05-30
> **Author:** PM May 29 (session `nimble-dove-733`)
> **Trigger:** v4-close diagnosis lane converging on "0 rustc errors" as the success criterion; operator question (2026-05-30): *"i don't think the goal should just be '0 rustc errors' — how do we even know the emitted code is correct? what standards do we have?"*

---

## §1. The provocation

The diagnosis lane (sunny-cat-359 catalog, `docs/audit/v4-rustc-error-catalog-2026-05-29.md`) measured 7951 rustc errors when the modeled runner (T-38-PR1) attempts to emit and compile the full `src/v4` corpus. The convergent framing inside the close lane has been *"shrink that number to zero."*

That framing has a known failure mode: **rustc-clean is one specific correctness bar — parseable, type-checking Rust — and it is far from the only standard the project has committed to.** The operator's question reframes the conversation: closing T-15 against rustc-clean only would close v4 on the lowest of the standards the thesis already names.

The operator's specific phrasing — *"how do we even know the emitted code is correct?"* — maps directly to **THESIS.md §"What falls out" L4**: *"emitted code executes and matches .dag evaluation."* The thesis already has this question answered at the *commitment* level. What's missing is the *operationalization* — which gates fire on which PRs, in what order.

This doc:
1. Inventories the 17 correctness standards the project has already articulated (§2).
2. Audits which of them gate PRs today vs which are substrate-only or aspirational (§3).
3. Critiques "0 rustc errors" as a singular target (§4).
4. Proposes a 9-rung correctness ladder that operationalizes the 17 standards (§6).
5. Proposes a fixture-first sequencing (§7) (no timelines per TASKS.md discipline; phases sequence-not-schedule).
6. Surfaces 6 operator decisions that must be answered before §7 dispatches (§8).

This doc invents no new standards. The 17 are pre-existing in `THESIS.md`. The audit, ladder, and sequencing are the contribution.

---

## §2. The 17 correctness standards already articulated

Extracted from `THESIS.md` §"Thesis claims — complete list", §"Tier 1/2/3", §"Self-hosting — four facets", and §"Enumerable impossible-bug classes":

### Tier 1 — Structural correctness (caught at compile time)

1. **Type / field / exhaustiveness / circularity / imports / cross-target drift** caught at compile time.
2. **CX termination bound** — every recursive function has a proven complexity bound.
3. **Coercion = emission** — the compiler reads a target spec and translates; no separate coercion engine.
4. **Ownership** — emitted code provably has no aliased mutation.
5. **Grounding completeness** — target primitives modeled from the target language reference; `.dag` → target mapping is a structural algebra-homomorphism search, not a string lookup.

### Tier 2 — Runtime safety (proven safe or total)

6. **Division-by-zero, integer overflow, OOB, force-unwrap, partial functions** — either proven safe at compile time or made total. No partial functions in runtime.

### Tier 3 — Verification from structure

7. **L4** — emitted code executes and matches `.dag` evaluation.
8. **L5** — same `.dag` produces same observable behavior in Rust / Python / Go (cross-target equivalence).
9. **L6** — every structural form compiles to every target.
10. **L7** — operations obey declared algebraic laws (preserved through emit).

### Self-hosting — four facets

11. **Facet 1** — compiler written in the language it compiles.
12. **Facet 2** — compiler self-emits (fixed-point); `.dag` is source of truth, emitted Rust is one realization.
13. **Facet 3** — tests are data; `TestClaim` declarations + generated target test code; **zero** hand-authored test residual under 0-floor.
14. **Facet 4** — lens self-application to build/CI pipeline.

### Enumerable impossible-bug classes (release scope)

15. **Suboptimal-complexity contract violation** — `complexity ≤ O(n log n)` annotation enforced at compile time.
16. **Idempotency-contract violation** — `@idempotent` enforced structurally.
17. **Transport / type drift** — client and server cannot hold different types for the same field.

**This is the standard.** It is the basis for "what does correct emit mean?" Nothing in the rest of this doc redefines it.

---

## §3. What is actually gated today (honest audit)

Verification conducted 2026-05-30 against `main` HEAD `4baef9551`. For each standard above, the live gating status:

| #   | Standard                                | Substrate today?            | Gate fires on PRs today? | Evidence                                                                                                          |
| --- | --------------------------------------- | --------------------------- | ------------------------ | ----------------------------------------------------------------------------------------------------------------- |
| 1   | Tier 1 type check                       | YES (Rust emit)             | Per slice; corpus **NO** | `scripts/v4-testclaim-corpus-gate.sh`; 7951 errors per `docs/audit/v4-rustc-error-catalog-2026-05-29.md`          |
| 2   | CX termination bound                    | YES (`lens/complexity.dag`, 301L) | NO                       | activation lane in flight (sharp-ferret-729); lens not wired to PR gate                                           |
| 3   | Coercion = emission                     | YES (T-9 substrate)         | partial (per-slice only) | `src/v4/compiler/infer.dag`                                                                                       |
| 4   | Ownership proof                         | YES (`lens/ownership.dag`)  | NO                       | substrate only; no PR gate                                                                                        |
| 5   | Grounding completeness                  | YES (T-30 `lens/fact_density.dag`) | NO                | substrate only                                                                                                    |
| 6   | Tier 2 runtime safety                   | partial                     | NO                       | aspirational; substrate work post-R1                                                                              |
| 7   | **L4 emit-matches-`.dag`-eval**         | NO execution loop           | NO                       | `scripts/v4-testclaim-corpus-eval.sh` reports `execution_status=blocked_m1_subset`                                |
| 8   | **L5 cross-target equivalence**         | design doc only             | NO; **zero** tests       | `docs/design-cross-target-equivalence.md` exists; no `src/v4/test/claim/cross_target/` directory                  |
| 9   | **L6 every form compiles every target** | partial (Rust only in CI)   | NO; single-target        | only `rustc` invoked; pyright / go vet / clang / tsc / lean / swift not wired into CI                             |
| 10  | **L7 algebraic law preservation**       | model-side only             | NO post-emit             | `test/claim/algebra_laws/nat_semiring.dag` tests laws on the Node model, not on emitted Rust/Python               |
| 11  | Facet 1 (.dag authors compiler)         | partial                     | n/a (descriptive)        | v4 substrate at 0 hand-maintained `.rs` in compiler tree; stage0 Rust scaffold remains for bootstrap              |
| 12  | **Facet 2 self-emit fixpoint**          | placeholder claim           | NO                       | `test/claim/self_host/claim_t15_self_host_fixed_point.dag` uses digest stubs; "deferred to T-22 eval / host harness" |
| 13  | **Facet 3 tests as data**               | 55 `TestClaim` files        | NO (never executed)      | `src/v4/test/claim/manual/` — type-checked, no eval; corpus runner blocked at M1                                  |
| 14  | Facet 4 lens self-application           | substrate (`workflow/ci.dag`) | NO                       | no lens-on-CI gate fires                                                                                          |
| 15  | Suboptimal-complexity violation         | substrate (lens/complexity) | NO                       | release-scoped per THESIS                                                                                         |
| 16  | Idempotency violation                   | substrate (lens/idempotency) | NO                      | release-scoped per THESIS                                                                                         |
| 17  | Transport / type drift                  | partial (lens/synthesis #3768 DONE) | partial          | T-17 substrate landed; not generalized to a PR gate                                                               |

**Honest summary:**
- **Of 17 named standards, exactly one fires on PRs today** — and only per-slice (#1, Rust type check). The full-corpus gate produces 7951 errors and is therefore *not* a passing PR gate.
- **Substrate exists for 13 of the remaining 16** — but the activation step (wiring the substrate into a gate that fails a PR) is missing.
- **The pattern is "substrate-rich, activation-poor"** — independently diagnosed in `docs/audit/v4-deferral-audit-2026-05-29.md` for the deferral surface, and shown here at the correctness-gate scale.

---

### §3.1 Where "honest task completion" actually lives today

Asked during PR review: *"what doc tracks what tasks have actually been completed/closed?"* Honest answer: **no single doc does that today.** The closest is `src/v4/TASKS.md` (2310 lines, 38 T-tasks), but its status vocabulary conflates "substrate landed" with "PR-gated and activated":

| Marker in TASKS.md          | What it actually means                                |
| --------------------------- | ----------------------------------------------------- |
| `[DONE]`                    | usually substrate complete; activation often unstated |
| `[SUBSTRATE LANDED]`        | explicit: substrate only, activation pending          |
| `[ENFORCEMENT GATE LANDED]` | explicit: activation confirmed                        |
| `[CORPUS FILLED]`           | substrate exists; execution unstated                  |
| `[CLOSABLE]`                | predicate satisfied but task not yet closed           |
| `[DISSOLVED]` / `[DROPPED]` | removed from scope                                    |
| `[SCHEDULED]` / `[MODELED]` | partial                                               |

The ambiguity is real and load-bearing:
- T-17 `lens/synthesis.dag` is marked `[DONE #3768]` per `TASKS.md:990` — but the §3 audit in this doc dispositions it as *partial* (substrate landed, not generalized to a PR gate beyond what T-17 itself proves).
- T-22 `compiler/05_eval.dag` has structural-bridge CI activation (`.github/workflows/ci.yml:288`, "T-22 TestClaim corpus structural bridge") yet the same step explicitly says "TestClaim verdict execution remains a T-38 follow-up" (`.github/workflows/ci.yml:342`) — substrate landed, gate fires, but the gate does not exercise the load-bearing receipt the task promises.

(Counter-example for fairness: T-19 `lens/testgen.dag` IS gate-activated — `.github/workflows/ci.yml:145` runs `scripts/check_t19_testgen_activation.py` for testgen activation + LBE generated claims + `run_test_claim` receipts. `[DONE]` is sometimes accurate.)

**Partial reality-side coverage exists across multiple docs (no consolidation):**

- `docs/audit/v4-rustc-error-catalog-2026-05-29.md` — 7951-error catalog = ground truth on T-15 P1 trigger (b)
- `docs/audit/v4-deferral-audit-2026-05-29.md` — 75-row deferral inventory (honesty on the annotation surface)
- `docs/audit/ci-anatomy-and-redundancy-2026-05-29.md` — CI ground truth
- **§3 of this doc** — gating reality for the 17 thesis standards (does not cover all 38 T-tasks)
- `docs/audit/r3-close-interrogation-validation-2026-05-13.md` — **17 days stale** (0/152 probes answered)

**Phase 0 (dispatched 2026-05-30 to child session `silent-raven-384`, work item `adhoc-7020540d-622`) is producing the missing single-source receipt-honesty doc:**
- Path: `docs/audit/v4-close-interrogation-validation-2026-05-30.md`
- Scope: all 346 probes in `docs/v4-close-interrogation.md`
- Output: per-section disposition (PROVEN / WEAK-EVIDENCE / GAP / NOT-CHECKED) with file:line evidence
- Visibility: separate PR with incremental commits; summary comment posted to PR #3938 on completion
- Budget: 6–10 hours; partial results visible mid-execution

**For in-progress review of PR #3938, the operative reading trio:**

1. **`src/v4/TASKS.md`** — claim side. Read the status markers as labeled in the table above, not as uniform "DONE."
2. **§3 of this doc** — gating reality for the 17 thesis standards (1 of 17 fires today; 13 are substrate-without-activation).
3. **`docs/audit/v4-close-interrogation-validation-2026-05-30.md`** — receipt reality (incremental commits visible on Phase 0 PR as `silent-raven-384` works through the 346 probes).

If a fourth source matters: PR list on GitHub (`gh pr list --state merged --limit 100`) shows what *actually merged*, which is the lowest-ambiguity record of work-landed (orthogonal to whether what landed is gated).

---

## §4. Why "0 rustc errors" is not the right goal

If T-15 closes against rustc-clean alone:

- We close on the **lowest of 17 standards** (#1, per-slice version).
- We have **no evidence** that emit_rust behaves equivalently to `.dag` evaluation (#7 / L4).
- We have **no evidence** that emit_rust and emit_python produce the same observable behavior (#8 / L5).
- **Six other emit targets** (python, go, cpp, ts, lean, swift) are unexercised (#9 / L6).
- Algebraic laws declared in the model (#10 / L7) are not propagated through emit.
- The **release-scope impossible-bug classes** (#15-17) are unverified.
- **Self-hosting fixpoint** (#12) is a placeholder.

Worst-case scenario: a future regression silently breaks L4 semantic correctness, rustc stays clean, no gate notices. The "substrate-rich, activation-poor" pattern is the exact failure mode this doc and the prior deferral audit both diagnose.

"0 rustc errors" is **necessary but not sufficient**. It's rung 1 of a longer ladder.

---

## §5. Two principles for sequencing

**Principle A — gate the broadest standards first, even on tiny fixtures.**
L4 (emit-matches-eval) is foundational. If emit doesn't *run*, L5/L6/L7 are moot. A working L4 gate on one fixture is more valuable than a broken L1 gate across the whole corpus.

**Principle B — small fixture end-to-end before widening.**
The 7951-error count is large because we ran a single-rung gate against the whole corpus. The inversion: run a 9-rung gate against ONE small fixture, then widen the fixture set. After widening, the 7951 number becomes diagnostic ("these Node shapes are wrong") instead of an undifferentiated mass.

These principles are derived from `TESTING.md` Principle #5 ("Mocks over compile") and INVARIANTS.md P5 ("Progress Is Dissolution") — they're not new claims.

---

## §6. The proposed correctness ladder (operator-decision artifact)

The 17 standards collapse into a 9-rung gating ladder. Each rung is a binary: gates fire on PRs or they don't.

**Note on achievability** (clarifies a prior contradiction with §3): "achievable today" is rung-dependent. Rungs 0–2 are achievable now on a small fixture using existing substrate. **Rungs 3–4 depend on T-38-PR2 runner progress** (per §7 Phase 2). **Rung 7** depends on a self-host harness that does not yet exist as more than placeholder digests (§3 row 12). The §3 audit dispositions rungs 4–9 as NO today *because the activation work has not been done*, not because the substrate is missing — but for rungs 3, 4, and 7 the activation work itself depends on runner / harness progress not yet landed. The ladder is the *target shape*; §7 sequences which rungs become achievable in which phase.

| Rung | Property                                                | Standards covered | Today's status                                  |
| ---- | ------------------------------------------------------- | ----------------- | ----------------------------------------------- |
| 0    | Parses in target                                        | (parse side of #1) | Yes (per slice, per target rustc)               |
| 1    | Type-checks in target Rust                              | #1                | Per slice; corpus blocked at 7951 errors        |
| 2    | Compiles in every project-committed target (multi-target) | #1; **subset of #9** (see note) | NO (Rust only in CI)               |
| 3    | Round-trip preserved (parse-emit-parse ≡ up to normalization) | (implicit T-36) | Claim exists; eval deferred behind T-38         |
| 4    | Emit runs and output matches `.dag` interpreter eval    | #7 (L4)           | NO                                              |
| 5    | Cross-target equivalence on small fixture               | #8 (L5)           | NO                                              |
| 6    | Algebraic laws preserved on small fixture (post-emit)   | #10 (L7)          | NO                                              |
| 7    | Self-emit fixpoint on small fixture                     | #12 (Facet 2)     | NO (placeholder digests)                        |
| 8    | `TestClaim` corpus actually executes                    | #13 (Facet 3)     | NO (55 claims un-run)                           |
| 9    | Lenses gate PRs (complexity, ownership, idempotency, grounding, synthesis) | #2-#5, #15-17, #14 | NO (substrate-rich, activation-poor) |

**Note on rung 2 vs standard #9 (L6).** Standard #9 (THESIS L6) is *"every structural form compiles to every target"* — universal across the target set. Rung 2 as a *gate* is necessarily phased: any single PR can only verify "every project-committed target as of this PR's HEAD" (a moving subset that grows as targets land). Rung 2 closure on a subset is **smoke coverage**, not L6 closure. Full L6 closure is achieved only when the target set has stabilized AND rung 2 is green on every PR for every target. This rung does NOT substitute for full L6 — both are needed, in sequence: rung 2 smoke on a fixture → rung 2 smoke on the corpus → L6 full coverage as targets stabilize. The doc treats this dilution explicitly here so an operator sign-off on §6 does not narrow the canonical L6 standard.

Each rung addresses a specific question the operator can ask of any PR:
- *Rung 0*: did the source parse?
- *Rung 1*: did the emit type-check in Rust?
- *Rung 2*: did the emit type-check in every project-committed target (subset of L6 until target set is stable)?
- *Rung 3*: did re-parsing the emit reproduce the source?
- *Rung 4*: did the emit run and produce the answer the `.dag` interpreter produces?
- *Rung 5*: did Rust and Python emits agree on the answer?
- *Rung 6*: did the algebraic properties survive emit?
- *Rung 7*: can the compiler emit itself and reproduce its own output?
- *Rung 8*: is the test corpus actually executing, not just type-checking?
- *Rung 9*: are the property lenses gating PRs, or are they shelfware?

---

## §7. Proposed fixture-first sequencing

(No durations per TASKS.md "no timelines" discipline — phases sequence by precedence, not by calendar.)

**Strategy.** Pick ONE small fixture with high algebraic content. Drive emit through every rung that has substrate. Each ungated rung gets a `TestClaim` + activation in the same PR (the 4-tuple unit-of-work from the prior dispatch contract: claim authored + substrate fix if needed + lens activation + demonstrated error-class collapse).

**Proposed fixture:** `src/v4/test/claim/algebra_laws/nat_semiring.dag` — already in the corpus; exercises Node + Atom + Conj + Arrow + algebra inhabitance + nat-add-associativity / nat-mul-associativity. Small surface, rich properties.

### Phase 1: rungs 0–2 on the fixture

- Confirm rung 0/1 (parse + Rust type-check) pass for the fixture specifically.
- Add rungs 2 for **three** targets only: Rust + Python + Go. Not all seven.
  - If silent-boar-535's multi-target checker work (currently WIP, branch `session/silent-boar-535`) is recoverable, salvage it. Otherwise build a narrow 3-target gate scaffold.
- Output: `src/v4/test/claim/nat_semiring/rung_0_to_2_three_targets.dag` + matching CI step.

### Phase 2: rungs 3–4 on the fixture

- Rung 3: round-trip `TestClaim` against the fixture, **executed**, not deferred. Requires T-38-PR2 progress.
- Rung 4: `TestClaim` that compiles the fixture's Rust emit, runs it, and asserts the output equals the `.dag` interpreter's eval for the same fixture.
- Output: rung 3/4 claims + execution wiring; first gate of the operator's "how do we know it's correct?" question.

### Phase 3: rungs 5 + 6 on the fixture

- Rung 5: emit_rust(fixture) + emit_python(fixture) both run; outputs compared for observable equivalence. First L5 gate ever to fire on this project.
- Rung 6: `claim_nat_add_associativity` re-evaluated against emitted Rust AND emitted Python — not just the Node model. First L7-post-emit gate.
- Output: cross-target equivalence + algebraic preservation proven end-to-end on one fixture.

### Phase 4: widen, not deepen

- Add 2 more fixtures targeting Node shapes the first fixture doesn't cover (proposal: one Branch-using, one Loop-using).
- Re-run rungs 0–6 against all 3 fixtures. (Rungs 7–9 stay phase-5+.)
- Catalog Node shapes still uncovered; queue as Phase 5+ fixtures.

### What this replaces

The still-fox-289 SG-1 / SG-2 / SG-7 substrate-fix lanes as the **primary** dispatch shape. Those substrate fixes still happen — but they happen *in service of* rungs 4–6 on a fixture, not as standalone error-count reduction. **The unit of work becomes "rung-of-ladder on fixture", not "error-class collapse."**

### Why the 7951 number stops being the headline

After Phase 4, the 7951 errors number is more diagnostic: it shows which Node shapes' emit is *structurally wrong* — because **rungs 0–6** of the ladder pass on the fixture-covered shapes (Phase 4 explicitly leaves rungs 7–9 for phase 5+ — self-emit fixpoint, full TestClaim corpus execution, and lens PR gates are NOT proven after Phase 4 and must not be claimed as such). The error count becomes a queue for Phase 5+ widening, not a success metric. Rungs 7–9 remain open after Phase 4 by design.

---

## §8. Operator decisions surfaced

The following questions need answers before §7 Phase 1 dispatches. Proposed answers are PM-recommendation; operator can confirm or redirect.

### D1. Is §6 (the 9-rung ladder) the right correctness ontology?

**Proposed:** Yes, with one acknowledged gap. The 9 rungs cover **16 of 17** standards from §2. Standard **#6** (Tier 2 runtime safety — division-by-zero / overflow / OOB / force-unwrap / partial functions) is **not assigned to any rung**. Per THESIS Tier 2 framing, this work is partially substrate and partially gated as post-R1; no current rung gates it. Three options for the operator:
- **Option A**: add **rung 10** ("Tier 2 partial-function elimination") to the ladder, gated separately.
- **Option B**: extend rung 9 to include #6 once Tier 2 substrate matures.
- **Option C**: ratify the gap explicitly as "out of v4 ladder scope, separately tracked under T-25 refinement substrate."

**Also confirm** (per §9.5) that the ladder is the correct *complement* to `docs/v4-close-interrogation.md` (questionnaire stays as granular probe surface; ladder stays as gate-sequencing surface; both adopt §0 disposition vocabulary).
**Operator decides:** confirm ontology + pick a #6 option, OR name additional standards missing from the ladder.

### D2. Is "small fixture first, widen later" (§7) the right strategy vs "broad rustc-fix first, ladder second"?

**Proposed:** Small fixture first. A 9-rung gate proves the ladder shape works; widening then exposes Node-shape-specific gaps cleanly. Reverse order risks fixing 7951 errors then discovering rungs 5/6/7 have no infrastructure — and that the substrate-rich/activation-poor pattern persists.
**Operator decides:** confirm sequencing, OR redirect to a different strategy.

### D3. Is rung 5 (cross-target equivalence) a release gate or post-release?

**Proposed:** Rung 5 on a minimal fixture set (3 fixtures) is **release-gate**; widening to full corpus is post-release. Rationale: THESIS L5 is a thesis claim, not labeled release-scoped explicitly — but shipping v4 without ONE proven cross-target equivalence claim ships a thesis claim that has never been demonstrated.
**Operator decides:** release-gate on 3 fixtures, OR defer entirely to post-release.

### D4. Rung 7 (self-emit fixpoint) and the TASKS.md v4-done definition

**Authority check.** `src/v4/TASKS.md:801-815` defines v4-done as: *"v4 compiles `src/v4/compiler/*.dag` end-to-end; v4 emits Rust source that compiles to a binary; That binary, run on `src/v4/compiler/*.dag`, produces bit-identical output; TestClaim suite passes; Hand-authored Rust is not the editable authority (proven by reproduction)."* And `TASKS.md:1239`: *"The release is when v4-done. Not before, not after."*

**TASKS.md:801-815 lists six v4-done predicates**, not just rungs 7+8:

| TASKS.md predicate                                                            | Ladder mapping                                |
| ----------------------------------------------------------------------------- | --------------------------------------------- |
| 1. Every other scheduled task in this plan complete (whole plan minus T-15)   | **Broader than rungs** — covers all 38 T-tasks; not a ladder rung itself |
| 2. v4 compiles `src/v4/compiler/*.dag` end-to-end                             | Rung 0 + Rung 1 (parse + Rust typecheck) on full corpus |
| 3. v4 emits Rust source that compiles to a binary                             | Rung 1 on full corpus                         |
| 4. Binary, run on `src/v4/compiler/*.dag`, produces bit-identical output      | Rung 7 (self-emit fixpoint)                   |
| 5. TestClaim suite passes                                                     | Rung 8 (TestClaim corpus executes)            |
| 6. Hand-authored Rust is not editable authority (reproduction-proven)         | Self-hosting structural commitment (Facet 1+2; meta-gate over rungs 7-8) |

So the release gate per operational authority is **predicates 1-6 collectively, including all relevant rungs**. An earlier draft of this section proposed rung 4 as release-minimum citing THESIS framing — that contradicted TASKS.md and has been retracted. *Equally important:* a prior fix that named only "rung 7+8" was also incomplete because it omitted predicate 1 (whole plan minus T-15) which is broader than any single ladder rung.

**Operator decides:**
- **Option A**: confirm all six predicates as release gate per TASKS.md. Implication: §7 must be extended with phases 5+ that achieve rungs 7-8 + complete predicate 1 (every other scheduled task) + verify predicate 6 (reproduction). Currently §7 stops at Phase 4 (rungs 0-6).
- **Option B**: amend TASKS.md:801-815 to lower the v4-done definition. This is a substantive change to operational authority and requires its own documented decision, not just a checkbox here.

**PM-recommendation (without lobbying)**: Option A. Lowering the v4-done bar to ship sooner would close on a definition that doesn't match the thesis's strongest correctness commitment. If sequencing pressure makes rung 7 infeasible by some deadline, the right surface is a TASKS.md amendment with named rationale — not a tacit narrowing via this doc.

### D5. What is the policy on standards-with-substrate-but-no-activation?

**Proposed:** Every substrate that lands without a same-PR activation gets a tracked dissolution deadline (≤30 days from substrate landing). After the deadline, blocks PRs in the same lens family until activated. This codifies a structural fix for the substrate-rich/activation-poor pattern that both this audit and the deferral audit (`docs/audit/v4-deferral-audit-2026-05-29.md`) diagnose.
**Operator decides:** confirm policy, OR propose a different anti-shelfware mechanism, OR rule that no policy is needed.

### D6. Sequencing — accept §7 fixture-first phases?

**Proposed:** §7 as drafted. Phase 1 dispatches immediately on sign-off; Phase 2/3/4 dispatched after preceding phase closes.
**Operator decides:** accept Phase 1 dispatch, OR sequence differently.

### D7. Phase 0 — questionnaire validation dispatch (retrospective ratification)

**Authority check.** Phase 0 was dispatched 2026-05-30 01:07Z at operator request before doc sign-off — see §9.4 for execution details, worker session id (`silent-raven-384`), work item (`adhoc-7020540d-622`), and the validation PR (#3941).

**Operator decides (retrospective ratification):** ratify the Phase 0 dispatch + the redo guidance + acceptance criteria. Proposed acceptance: realistic disposition distribution (not uniform GAP); per-probe file:line evidence; section-level totals — see §3.1 and the redo brief. Phase 0 output is in flight on PR #3941; landed redo confirms substrate-rich/activation-poor at probe granularity.

---

## §9. Integration with the existing ship interrogation

The project already has an adversarial questionnaire — `docs/v4-close-interrogation.md` (1335 lines, 17 sections, **346 probes** at HEAD; was 152 probes at the 2026-05-13 validation point). It was originally authored as the R3 close interrogation, migrated to v4 framing 2026-05-15 per operator directive ("v4 = R3 + R4 in one program").

The questionnaire and this ladder are **complementary, not competing**:

- The **questionnaire** asks "for each promise, show the receipt: code, demo, falsification probe." Granular, probe-by-probe.
- The **ladder** (§6) asks "which gates fire on PRs in what order." Coarse-grained, sequencing-oriented.

The questionnaire's §0 disposition vocabulary — **PROVEN / WEAK-EVIDENCE / GAP / OPERATOR-DECISION-REQUIRED / NOT-IN-V4 / NOT-PROMISED** — should be adopted across both artifacts.

### §9.1 Where the questionnaire stands today

The 2026-05-13 validation (`docs/audit/r3-close-interrogation-validation-2026-05-13.md`) found **0/152 probes marked answered** — every probe `- [ ]` not `- [x]`. The questionnaire's §0.5 (AUTHORITATIVE, 2026-05-15) verdicts **scaffold-passing** ("every promise has an owner file + task; zero unresolved OPERATOR-DECISION-REQUIRED") but does NOT verdict **receipt-passing** — receipt validation is the work that has never been done.

The §3 audit in this doc *is* a partial questionnaire validation by-rung. Mapping is direct.

### §9.2 Questionnaire ↔ ladder cross-map

Each ladder rung activates verification for one or more questionnaire sections:

| Rung | Questionnaire sections addressed |
| ---- | -------------------------------- |
| 0    | §3.1 parse paths (omni-emission entry) |
| 1    | §3.1 Rust compile path |
| 2    | §3.1 Rust + Python + Go omni-emission; §3.5 L6 every-form-every-target |
| 3    | §3.7d round-trip (currently dispositioned NOT-PROMISED-as-separate-file; ladder reinstates as receipt gate) |
| 4    | §3.7 testgen + integration; §3.6 L7 (eval matches interpreter is the foundational L7 receipt) |
| 5    | §3.5 L6; §3.1 omni-emission semantic equivalence |
| 6    | §3.6 L7 algebraic laws preserved through emit |
| 7    | §4.2 self-host fixed point |
| 8    | §3.3 tests-as-data |
| 9    | §1.1-§1.5 dimension lenses; §4.1 lens self-application; §2.5 impossible-bugs by construction (META-PROMISE, 36 probes) |

**Coverage by §7 phases.** §7 Phases 1-4 sequence rungs 0-6 only. That covers questionnaire sections §3.1 (rungs 0-2), §3.5 / §3.6 / §3.7 (rungs 4-6) and partials of others — a substantial fraction of the emit/eval probes. Phases 5+ are NOT currently in §7; they would have to add coverage for rungs 7-9 (§4.2 self-host fixpoint ~4 probes; §3.3 tests-as-data ~4 probes; §1.1-§1.5 dimensions + §4.1 lens self-application + §2.5 impossible bugs — collectively the largest single block, ~88 probes by section counts in silent-raven-384's first-pass tally).

A prior draft claimed "~90% of probes" close via §7 — that overstated the §7 commitment, since §7 explicitly stops at rung 6 and rungs 7-9 are the largest unaddressed block. The honest read: §7 as currently sequenced closes the bulk of emit/eval probes; closing the questionnaire fully requires §7 Phase 5+ (or a separate program) and is coupled to D4.

### §9.3 Sample probe evaluation (2026-05-30 spot-checks)

To anchor the disposition vocabulary, 8 probes spot-checked against current main (`4baef9551`):

| Probe (paraphrased) | §ref | Disposition | Evidence |
| ------------------- | ---- | ----------- | -------- |
| Rust primitives have algebra-inhabitance declared | §1.6 P1 | **WEAK-EVIDENCE** | `src/v4/extdeps/languages/rust.dag` exists with algebra-relevant terms; trace per-primitive grounding not verified end-to-end |
| Algebra-homomorphism-search is structural, not name-keyed | §1.6 P2 | **NOT-CHECKED** | grep across `src/v4/` did not surface an obvious search function; needs targeted investigation |
| TestClaim corpus executes on PRs | §3.3 P5 | **GAP** | CI step explicitly says "TestClaim verdict execution remains a T-38 follow-up"; 55 manual claims un-run |
| v4 compiler emits v4 → bit-identical fixed point | §4.2 P7 | **GAP** | `claim_t15_self_host_fixed_point.dag` uses digest placeholders; real cycle deferred |
| Non-trivial .dag program compiles to Rust + Python + Go | §3.1 | **GAP** | only `rustc` invoked in CI; no Python/Go compilation gate |
| Algebraic law verified on emitted code (post-emit) | §3.6 | **GAP** | `algebra_laws/nat_semiring.dag` tests laws on model only |
| Lens applied to `workflow/ci.dag` or `workflow/bootstrap.dag` (self-application) | §4.1 | **GAP** | substrate exists; no lens-on-CI-data gate fires |
| Suboptimal-complexity contract violation rejected at compile time | §2.5 P8 | **NOT-CHECKED** | candidate test files exist (`map_id.dag`, `nested.dag`, etc.); per-test rejection-vs-pass classification not confirmed |

Pattern: probes that map to **rungs 4–9 of the ladder** disposition predominantly as **GAP**. This is the same finding as §3 (substrate-rich, activation-poor) viewed through the questionnaire's lens.

### §9.4 Phase 0 — systematic questionnaire validation (dispatched 2026-05-30 at operator request)

Phase 0 was dispatched 2026-05-30 01:07Z (work item `adhoc-7020540d-622`, worker `silent-raven-384`) at operator request, in parallel with this doc's review:

- Output target: `docs/audit/v4-close-interrogation-validation-2026-05-30.md` as separate PR (#3941 opened 2026-05-30 ~01:13Z).
- Scope: all 346 probes in `docs/v4-close-interrogation.md` against current main.
- Disposition vocabulary: §0 of the questionnaire (PROVEN / WEAK-EVIDENCE / GAP / NOT-CHECKED / OPERATOR-DECISION-REQUIRED / NOT-IN-V4 / NOT-PROMISED).
- Result feeds back into THIS doc's §3 as a more complete gating audit.
- **Redo update (2026-05-30 ~01:27Z)**: silent-raven-384 completed the per-probe redo on commit `68b9bd7ed`. Final distribution: **0 PROVEN / 267 WEAK-EVIDENCE / 42 GAP / 37 NOT-CHECKED**. This matches the predicted realistic distribution and confirms the substrate-rich/activation-poor diagnosis at probe granularity. PR #3941 is mergeable=clean per dashboard (operator merges manually per current policy). The historical "uniform-GAP first pass" is preserved here as worker-management context — the first pass produced uniform-GAP boilerplate via a single structural argument (T-38 runner blocked → 346 GAP) rather than per-probe codebase search; redo brief sent 2026-05-30 01:18Z asking for per-probe disposition with realistic distribution for substrate-rich-but-not-end-to-end-gated probes; redo landed within 10 minutes.

### §9.5 Effect on §8 operator decisions

The questionnaire integration retroactively introduces one decision (now formally placed in §8 as **D7**) and modifies one:

- **D7**: moved to §8 (see §8.D7). Phase 0 dispatched at operator request before doc sign-off; operator decision is retrospective ratification.

- **D1 (modified)**: in addition to confirming the ladder ontology + #6 standard gap acknowledgment, confirm that the ladder is the correct *complement* to the questionnaire — the questionnaire stays as the granular probe surface; the ladder stays as the gate-sequencing surface; both adopt the §0 disposition vocabulary. (Folded into §8.D1.)

---

## §10. Worked-example root-cause analyses (operator-driven, DFS discipline — not error-count chasing)

**Why this section exists.** The §7 sequencing risks reverting to error-count-driven dispatch ("fix SG-1 → fix SG-2 → ..."). Past pattern: error-count reduction has led to spot-fixes that calcified poor modeling decisions into irreversible templates. **SG classes are evidence, not dispatch units.** The dispatch unit is the deepest modeling fact that removes the class without adding a new parallel authority.

### §10.0 The mandatory DFS Root-Cause Worksheet

**Mechanical dispatch rule (operator-ratified 2026-05-30):**

> **No SG class may be assigned to a worker until its DFS worksheet identifies the single-authority fact to add or consume.**

Every SG-class lane fills out this worksheet BEFORE the work item is dispatched. The worksheet is reviewable; the dispatch brief is downstream of it.

**Worksheet template:**

```text
SG class:                                  // e.g. SG-1
Representative emitted failure:            // verbatim from catalog
Immediate local patch:                     // the tempting count-reduction fix
Why that patch is forbidden or accepted:   // forbidden reason or accepted rationale
DFS path:
  std/ authority:                          // where the relevant substrate type lives
  extdeps/language authority:              // where (if anywhere) the target realization lives
  compiler stage consuming it:             // 04_infer, 05_emit, lens/, etc.
  existing scaffold/dissolution notes:     // any gated 🟡 notes pointing at modeling debt
Deepest unsound boundary:                  // where the missing fact actually lives
Systemic fix:                              // the single-authority carrier to add or consume
Non-goals:                                 // what NOT to do (often: spot fix, name-keyed list)
Falsification probe:                       // how to verify the fix is systemic, not a patch
Metric allowed only as secondary:          // error count is evidence, not goal
```

**Two load-bearing fields.** `Immediate local patch` and `Why that patch is forbidden` force the worker to name the tempting count-reduction fix and explicitly reject it when it would create a new parallel authority.

**Two-axis disposition vocabulary (operator-ratified 2026-05-30).** The single-axis `PROVEN / WEAK-EVIDENCE / GAP / NOT-CHECKED` vocabulary collapses two distinct questions. Going forward, all close-related artifacts (PR #3941 validation included) adopt two axes:

```text
ship_disposition:
  PROVEN | GAP | NOT_IN_V4 | NOT_PROMISED | OPERATOR_DECISION_REQUIRED

engineering_state:
  SUBSTRATE_PRESENT
  SCAFFOLD_PRESENT
  PARTIAL_GATE_PRESENT
  CENSUS_NOT_RUN
  EXECUTION_NOT_WIRED
  NO_ARTIFACT_FOUND
```

A row example:
```text
ship_disposition: GAP
engineering_state: SUBSTRATE_PRESENT
blocking_receipt: T-38 executable runner + adversarial falsification transcript
owner_manager: Runtime/TestClaim + Lens Activation
```

**Closure invariant (mechanical):**

> **A probe cannot move to `PROVEN` by adding more substrate declarations. It moves to `PROVEN` only by adding an executable receipt that answers the exact probe and includes the falsification case when requested.**

This prevents `WEAK-EVIDENCE` (and engineering_state=SUBSTRATE_PRESENT) from being read as "partially close-ready." Substrate presence is planning value; close-readiness is `PROVEN` only.

**The worksheet is a formal dispatch gate, not optional reviewer prose.** No worker brief may be issued from an SG class (or any close-blocker class) until the worksheet is complete and manager-approved. The worksheet is part of the manager/worker protocol — see §11 manager lanes for which manager owns approval per class.

SG-1, SG-2, and SG-5/SG-6 are worked through below using this template. SG-CANDIDATE-1 (frontend sugar) is worked through in §10.5 as a non-error candidate that uses the same discipline.

### §10.1 SG-1 — Symbol / Atom value emission (2978 errors)

**Symptom (from `docs/audit/v4-rustc-error-catalog-2026-05-29.md`).** Emitted Rust like:
```rust
pub fn loop_bound_edge() -> String {
    Symbol("loop_bound_edge".to_string())
}
```
fails because `Symbol` is realized as a type alias (`type Symbol = String;`) elsewhere — the constructor call `Symbol(...)` is invalid on an alias. Two emission stages disagree on whether Symbol is a wrapper struct or a transparent alias. 2978 instances of this shape (E0423) across emitted files.

**Identification method.** Catalog-driven — the rustc-error class table flagged E0423 as the largest single class. The pattern in the emitted Rust was unmistakable: every `Symbol(...)` constructor call. The catalog itself is honest evidence (worth keeping); the question this section answers is *what to do with that evidence*.

**Spot fix that would calcify the modeling gap.** Patch `05_emit.dag`'s value-emit template so it omits the `Symbol(...)` wrapper for Atom values. Closes 2978 errors immediately. **DO NOT DO THIS** — it cements a template special-case that hides three layers of underlying modeling debt:

**Root cause — Layer 1 (the immediate template bug).** Type-emit chose `type Symbol = String;`; value-emit chose `Symbol("...".to_string())`. The two emission stages derive Symbol's Rust realization *independently* and disagree.

**Root cause — Layer 2 (the substrate single-authority gap).** DFS through `std/` per MODELING.md M9:
- `std/node.dag:10` declares `type Symbol` — bare, no body. Symbol is kernel-ambient per INVARIANTS.md P1 hollow-alias exception ("kernel-ambient atoms are genuinely atomic and exempt from fact-bundle modeling").
- `extdeps/languages/rust.dag` imports Symbol as a type, uses it pervasively as a field type — but contains **no declaration of Symbol's Rust target realization**. (Grep `realization|projection.*rust|symbol.*rust|atom_to_rust` returns nothing.)
- Therefore both type-emit and value-emit derive Symbol's Rust form independently. They violate INVARIANTS.md P2 ("every fact lives in exactly one authoritative place"). The disagreement is the bug; the missing single-authority fact is the modeling gap.

**Root cause — Layer 3 (a tracked modeling debt *above* the entire pattern).** `std/node.dag:84-85` carries a 🟡 gated note:
> *"feature: node-behavior-loop-bound-edge-tag — owner: T-12 LoopBound cost-lens wiring — lane: shared `Interval<D>` parent for Loop measure facts — **dissolve-on-arrival: replace Symbol-tagged Loop bound attachment with structural Loop-bound coordinate owned by the interval/bound substrate**; forbidden: new Loop-bound consumers matching raw Named(loop_bound_edge) outside std.node/std.cardinality."*

So `loop_bound_edge` itself — the catalog's representative example — is a known modeling-debt site. The Symbol-as-edge-tag pattern is gated for dissolution into structural Loop-bound coordinates. **Spot-fixing the emitter for `loop_bound_edge` would create new Symbol-tag consumers in generated Rust, directly contradicting the gated "forbidden" clause and blocking the dissolution.**

**The systemic fix — what depth does the operator want?**

| Layer | Fix | Scope | Risk if skipped |
| ----- | --- | ----- | --------------- |
| 1     | Patch value-emit template | 1 file in `05_emit.dag` | calcifies layer 2 gap; blocks layer 3 dissolution |
| 2     | Declare `TargetAtomRealization` single-authority fact in `extdeps/languages/<lang>.dag` (language-parametric, not Rust-only); both type-emit + value-emit consume it | new substrate carrier + refactor 2 emit paths | layer 3 still open, but no longer blocked |
| 3     | Dissolve Symbol-as-edge-tag entirely per gated note (replace with structural Loop-bound coordinate) | substantial substrate migration spanning std/node, std/cardinality, lens/cost, all Loop-using sites | error-count side-effect; primary intent is modeling cleanup |

**Recommendation: layer 2 first, layer 3 separately (it's already gated and owned by T-12 per `std/node.dag:84-85`).**

Layer 2 is the right depth because:
- It is *non-blocking* to layer 3 (the single-authority Atom realization is needed regardless of whether Loop bounds eventually stop using Symbol tags).
- It is implementable as one bounded substrate addition + two emit-stage consumers.
- It generalizes: the same single-authority pattern applies to every kernel-ambient atom (Bool, Char, Symbol) in every target language. **The carrier is language-parametric** (operator caution 2026-05-30: a Rust-only `RustAtomRealization` would calcify the pattern as language-specific). Sketch:
  ```dag
  type TargetAtomRealization {
    source_atom: Symbol                       // e.g. Symbol, Bool, Char
    target_model: LanguageModel               // Rust, Python, Go, ...
    type_form: TargetTypeExpression           // emitted in type position
    value_form: TargetValueExpression         // emitted in value position
    constructor_form: Optional<TargetConstructorExpression>  // None for transparent aliases
  }
  ```
  Rows for Rust + parallel rows for other targets.
- It surfaces (rather than hides) layer 3 — if an Atom doesn't fit `{ type_form, value_form, constructor_form }`, that escalates as modeling work, not a worker fix.

**Acceptance is NOT "E0423 decreased."** Acceptance is *"change the Symbol realization row, and both type-emit and value-emit change together. If only one changes, the single-authority problem remains."*

**The dispatch shape that protects against the spot-fix trap.**

- **One** work item, narrowly scoped: *"author `TargetAtomRealization` (language-parametric) with Rust rows for Symbol/Bool/Char in `extdeps/languages/rust.dag`; refactor `05_emit.dag` so both type-emit and value-emit consume the row; verify the falsification probe (change row → both stages change together)."*
- **Not** "fix SG-1." The dispatch frame is the modeling fact being added, not the error count being chased.
- **Brief must FORBID** template special-casing and must require the worker to escalate (not "fix") any Atom whose realization doesn't fit the `TargetAtomRealization` schema. If found, that's a modeling escalation, not a worker call.
- **Brief must FORBID** any new Symbol-tag consumers in generated Rust per the layer-3 gated note (`std/node.dag:84-85`). The new emit must produce structural realizations only.
- **Worker output**: the substrate row(s) + the refactored emit paths + falsification probe receipts.

**What does NOT get dispatched.**
- Worker chasing the 2978 errors directly.
- Layer-3 dissolution (separately owned by T-12 per the gated note).

### §10.1.1 SG-1 worksheet form

```text
SG class: SG-1
Representative emitted failure:
  pub fn loop_bound_edge() -> String {
      Symbol("loop_bound_edge".to_string())
  }
  // Symbol used as constructor on a type alias `type Symbol = String;`
Immediate local patch:
  Patch 05_emit value-emit template so Symbol("x") becomes "x".to_string().
Why that patch is forbidden:
  Leaves type-emit and value-emit independently deriving Symbol's Rust
  realization. The single-authority problem (INVARIANTS P2) remains. Also
  calcifies the Symbol-as-edge-tag pattern that std/node.dag:84-85 says is
  supposed to dissolve (would create new forbidden consumers).
DFS path:
  std/ authority:
    - type Symbol declared bare at std/node.dag:10 (kernel-ambient).
    - Symbol values declared via `data X: Symbol = X` pattern, pervasive.
  extdeps/language authority:
    - extdeps/languages/rust.dag imports Symbol, uses as field type.
    - NO declaration of Symbol's Rust target realization (verified via grep:
      no "realization|projection.*rust|symbol.*rust|atom_to_rust" hits).
  compiler stage consuming it:
    - 05_emit type-emit chose `type Symbol = String;`.
    - 05_emit value-emit chose `Symbol("...".to_string())`.
    - Stages derived independently; they disagree.
  existing scaffold/dissolution notes:
    - std/node.dag:84-85 — Symbol-as-edge-tag dissolution gated under T-12
      (loop_bound_edge specifically named "forbidden" for new consumers).
Deepest unsound boundary:
  Missing TargetAtomRealization fact-bundle in extdeps/languages/<lang>.dag.
  Both emit stages derive independently because no single authority exists.
Systemic fix:
  TargetAtomRealization { source_atom, target_model, type_form, value_form,
  constructor_form }. Rust rows for Symbol + Bool + Char. Both type-emit and
  value-emit consume the row.
Non-goals:
  - Patching value-emit template (layer 1 spot fix).
  - Rust-only carrier name (would calcify pattern as language-specific).
  - Dissolving Symbol-tag pattern from std/node.dag (separately owned by T-12).
Falsification probe:
  Change the Symbol realization row (e.g., flip Rust to a newtype struct).
  Verify BOTH type-emit and value-emit output change together. If only one
  changes, single-authority is violated.
  Secondary: grep for any string-keyed Symbol projection logic in 05_emit.dag
  — should be zero after refactor.
Metric allowed only as secondary:
  2978 E0423 errors are evidence of the modeling gap. They collapse as
  side-effect of the systemic fix. They are NOT the acceptance criterion.
```

### §10.2 SG-2 — Generic carrier arity (1219 + 747 errors)

**Symptom (catalog).** Emitted Rust drops generic parameters:
```rust
pub type FileReadResult = Rc<Outcome>;
```
`Outcome` is emitted as a bare name; its type arguments are dropped. 1219 E0107 + 747 E0282 — second-largest cluster.

**Operator hypothesis (ratified by DFS below):** *"The target type-expression projection is losing Instantiation / generic application structure, so emitted Rust sees raw carrier names like Outcome, Witness, or TestClaimRun without their type arguments."*

**Worksheet form:**

```text
SG class: SG-2
Representative emitted failure:
  pub type FileReadResult = Rc<Outcome>;
  // Should be Rc<Outcome<...>> with proper type arguments.
Immediate local patch:
  Maintain a name-keyed list in the emitter:
    if type_name in {"Outcome", "Witness", "Refined", "TestClaimRun"}:
      require one (or N) type args
Why that patch is forbidden:
  - Doesn't scale to new generic carriers (every new one requires an emitter edit).
  - Creates parallel authority: the name-keyed list duplicates the fact already
    declared in std/diagnostic.dag, std/witness.dag, std/refinement.dag.
  - Calcifies the very anti-pattern (name-keyed table) INVARIANTS P3 forbids.
DFS path:
  std/ authority:
    - Outcome<T> declared at std/diagnostic.dag:49
    - Witness<C> declared at std/witness.dag:13
    - Refined<B> declared at std/refinement.dag:40
    - Instantiation declared as Connective at std/node.dag:19
    - Instantiation lowering uses PositionalEdges per std/node.dag:113
      (type arguments ARE present in the substrate as edges; they're not lost).
  extdeps/language authority:
    - extdeps/languages/rust.dag: grep for TargetTypeExpression / Instantiation /
      generic projection / type_arg / type_application returns NO hits.
    - No single-authority fact declares how Instantiation projects to Rust
      generic-application syntax.
  compiler stage consuming it:
    - grep for "Instantiation" in src/v4/compiler/ returns NO hits.
    - The stage that should consume Instantiation (likely 05_emit's
      type-expression projection) does not explicitly process Instantiation today.
  existing scaffold/dissolution notes:
    - none surfaced; this is a foundational missing-substrate gap.
Deepest unsound boundary:
  The Instantiation connective is declared in std/node.dag but has NO per-target
  realization fact. Each emit path encountering Instantiation must derive the
  target-language generic-application syntax independently. Rust gets no
  generic args because no consumer reads the Instantiation edges that carry
  the type arguments.
Systemic fix:
  TargetTypeExpressionProjection fact-bundle in extdeps/languages/<lang>.dag
  declaring per-connective projection to target type-expression syntax:

    type TargetTypeExpressionProjection {
      target_model: LanguageModel
      atom_form: TargetAtomTypeShape
      conj_form: TargetRecordTypeShape
      disj_form: TargetSumTypeShape
      arrow_form: TargetFunctionTypeShape
      cardinality_form: TargetGenericApply       // List<T>, Vec<T>, ...
      instantiation_form: TargetGenericApply     // F<A,B> in Rust; F[A,B] in Python
    }

  Refactor 05_emit's type-emission path to consume this projection per-connective.
  For Instantiation: emit reads Instantiation.children edges (PositionalEdges per
  std/node.dag:113) and applies instantiation_form.
Non-goals:
  - Name-keyed special cases for Outcome / Witness / Refined / TestClaimRun.
  - Hardcoding "emit one generic arg" for known carriers.
  - Patching individual emitted Rust files.
Falsification probe:
  Introduce a NEW generic carrier in std/ (e.g. `type FooBar<T, U> { ... }`).
  Use it in a position. Emit. Verify the emitted Rust has `Rc<FooBar<X, Y>>`
  with correct arity — WITHOUT adding any emitter branch for FooBar.
Metric allowed only as secondary:
  1219 E0107 + 747 E0282 are evidence the projection is missing. They are
  NOT the acceptance criterion.
```

**Dispatch shape:** one work item — *"author `TargetTypeExpressionProjection` row for Rust in `extdeps/languages/rust.dag`; refactor `05_emit.dag` type-emission to consume it per-connective (including Instantiation); verify the falsification probe."*

### §10.3 SG-5 / SG-6 — Set / BoundedLattice target-realization constraints

**Symptom (catalog).** Rust emit produces `compile_error!` stubs:
- Set keys lacking `Ord` (SG-5)
- BoundedLattice instances lacking `meet`/`join` implementations (SG-6)

Combined because they share the modeling shape — target representation requires constraints the model never declared.

**Operator hypothesis (ratified by DFS below):** *"The model has not declared the target realization constraints required by the chosen Rust representation."*

**Worksheet form:**

```text
SG class: SG-5 + SG-6
Representative emitted failure:
  // SG-5: Rust BTreeSet<T> requires T: Ord, but model declared no such
  //       constraint on Set<T>. Emit produces compile_error! at Set sites.
  pub type LookupKeys = Rc<Set<NodeId>>;

  // SG-6: BoundedLattice instances emit `meet`/`join` as compile_error! stubs
  //       because the .dag instance doesn't populate them.
Immediate local patch:
  - SG-5: patch emit to add `T: Ord` trait bound everywhere Set<T> is used.
  - SG-6: patch emit to emit dummy meet/join function bodies.
Why that patch is forbidden:
  - Fabricates a guarantee the model never proved (INVARIANTS P3:
    "no fabricated plausible output").
  - SG-5: calcifies Set<T> as BTreeSet<T> without modeling alternatives
    (HashSet for hash-keyed, Vec for ordered sequence). Realization choice
    belongs in the model.
  - SG-6: hides incomplete BoundedLattice instances under stub bodies;
    contradicts P3 "either succeeds fully or fails with a typed diagnostic."
DFS path:
  std/ authority:
    - Set<T> = PointwisePower<T> at std/collection.dag:52
    - FiniteSet<T> at std/collection.dag:107 (separate concept)
    - Set<T> declares NO Ord-eligibility constraint on T
    - BoundedLattice<T> at std/algebra.dag:66 with meet: fn(T,T)->T + join: fn(T,T)->T
    - BoundedLattice instances may not populate these fields; no required-witness check
  extdeps/language authority:
    - extdeps/languages/rust.dag has NO Set Rust realization (verified — only
      unrelated lex_rule_set match); no representation choice declared.
    - No constraint-propagation fact for collection target realization.
  compiler stage consuming it:
    - 05_emit derives "use BTreeSet, add T: Ord" without substrate authority.
  existing scaffold/dissolution notes:
    - none directly; missing-substrate situation, not tracked debt.
Deepest unsound boundary:
  (a) Collection-target realization choice — Set<T> → ? in Rust — should be
      declared per-language with named alternatives plus per-alternative
      T-constraints. Consumer picks the representation that satisfies the
      available T-constraints OR fails closed.
  (b) BoundedLattice instance completeness — the .dag should require meet/join
      witnesses at instance declaration. The witness is part of the
      inhabitance proof.
Systemic fix:
  (a) TargetCollectionRealization fact-bundle:

      type TargetCollectionRealization {
        source_carrier: Symbol                  // Set, Map, List, ...
        target_model: LanguageModel
        primary_form: TargetRepresentation      // BTreeSet, HashSet, Vec, ...
        constraints: List<RequiredTraitWitness>   // [{trait: Ord, on: T}, ...]
        alternative_forms: List<TargetRepresentation>  // fallback when constraints fail
        fallback_diagnostic: Diagnostic         // typed failure if no representation fits
      }

  (b) BoundedLattice-instance completeness gate in 04_infer: every instance
      either populates the witness fields OR is tagged as partial-instance
      with typed diagnostic at consumer sites. No silent compile_error! stubs.
Non-goals:
  - Adding T: Ord everywhere as a Rust-side patch.
  - Picking BTreeSet as the de-facto realization without modeling.
  - Emitting dummy meet/join bodies.
Falsification probe:
  (a) Introduce a Set<NonOrdable> in test/. Verify EITHER compile-time rejection
      OR alternative-representation selection OR a typed fail-closed diagnostic.
      NOT a silently-added Ord bound.
  (b) Introduce a BoundedLattice instance with missing `meet`, attempt to use
      it, verify EITHER compile-time rejection OR a typed partial-instance
      diagnostic. NOT a stubbed meet body.
Metric allowed only as secondary:
  SG-5 + SG-6 error counts are evidence; acceptance is falsification probe behavior.
```

**Dispatch shape:** two work items (one per modeling fact):
- *"author `TargetCollectionRealization` row(s) for Rust in `extdeps/languages/rust.dag`; refactor `05_emit.dag` collection-realization to consume it; verify Set<NonOrdable> falsification probe."*
- *"add BoundedLattice-instance completeness gate in `04_infer`; partial instances surface typed diagnostics rather than emit stubs; verify missing-meet falsification probe."*

### §10.4 What this DFS discipline buys

For each of SG-1, SG-2, SG-5/SG-6: dispatch is a **modeling fact to add** (or a typed gate to author), not an error count to reduce. Error count drops as side-effect; if it doesn't, the systemic fix was incomplete and the falsification probe surfaces that explicitly.

Total new work items if all three are dispatched: **4** (SG-1: 1; SG-2: 1; SG-5/SG-6: 2). Compare to "fix all 7951 errors" framing: the dispatch is bounded, the modeling deliverables are reviewable, and the spot-fix trap is structurally prevented by the brief-required worksheets.

---

## §11. What this doc is NOT

- **Not a redefinition of correctness.** The 17 standards are pre-existing in `THESIS.md`. This doc maps them to gating reality and proposes operationalization, nothing more.
- **Not a complete pre-committed plan.** Operator sign-off on §6 ladder + §7 Phase 1 unblocks the first dispatch. Phases 2/3/4 are dispatched after their predecessor closes — sequenced, not pre-committed. No timelines per TASKS.md discipline.
- **Not a substitute for per-rung detailed briefs.** Each rung at dispatch time will likely need its own brief naming the fixture, the substrate touchpoints, the activation lane, and the success predicate.
- **Not a critique of the existing tree.** The substrate-rich state of v4 is genuine progress — without the substrate, none of the rungs would even be authorable. The critique is *only* the activation gap.

---

## §12. Related artifacts

- `docs/v4-close-interrogation.md` — the existing adversarial ship interrogation (346 probes, 17 sections); §9 of this doc integrates with it.
- `docs/audit/r3-close-interrogation-validation-2026-05-13.md` — the prior validation that found 0/152 probes answered.
- `THESIS.md` §"What falls out", §"Tier 1/2/3", §"Self-hosting — four facets", §"Enumerable impossible-bug classes" — the 17 standards.
- `INVARIANTS.md` P5 "Progress Is Dissolution" — the principle driving "activation must follow substrate."
- `TESTING.md` Principle #5 "Mocks over compile" — the discipline that enables rungs 4–7 without forcing full-pipeline.
- `docs/audit/v4-rustc-error-catalog-2026-05-29.md` (sunny-cat-359) — the 7951-error catalog this doc reframes.
- `docs/audit/v4-deferral-audit-2026-05-29.md` — the prior substrate-rich/activation-poor diagnosis.
- `docs/audit/ci-anatomy-and-redundancy-2026-05-29.md` — current CI shape vs target.
- `docs/design-cross-target-equivalence.md` — existing design substrate for rung 5.
- `docs/design-pure-bootstrap-zero.md` — 0-floor self-hosting target (rung 7).
- `src/v4/TASKS.md` T-15, T-19, T-22, T-36, T-38 — the close-related task IDs implicated.
- `src/v4/test/claim/round_trip/dag_ingest_round_trip.dag` — existing rung 3 claim (eval deferred).
- `src/v4/test/claim/self_host/claim_t15_self_host_fixed_point.dag` — existing rung 7 placeholder.
- `src/v4/test/claim/algebra_laws/nat_semiring.dag` — proposed Phase 1 fixture.
