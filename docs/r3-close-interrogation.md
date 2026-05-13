# R3 Close Interrogation Sheet

**Authoring status**: PM-authored draft per operator directive 2026-05-13 (Director busy). Surfaces for Director ratification + cross-Mgr refinement before R3 close.

**Operational use**: this is the meta-acceptance checklist for R3 close. Each item is a question; check off when the answer is verified, not assumed. R3 is closeable when every required item is PASSING (per §13 disposition tracking).

---

## §0. Purpose + scope

**Why this sheet exists**: `docs/r3-program-plan.md` §1.8 has 104 closure-gate state-check predicates. Each predicate verifies a structural form (a count is zero, a file exists, a test passes). But structural form is not behavioral realization. A gate can pass its predicate while failing its intent — e.g., a `Lookup<C>::Miss` count of zero is satisfied if `Miss` is renamed `NotFound`, not if the deferral pattern is dissolved.

**This sheet asks the meta-questions** that §1.8 predicates alone cannot answer:
- Did each predicate execute at close-time, not at declaration-time?
- Does each predicate's match-target reflect the gate's semantic intent?
- Are cross-doc ledgers internally coherent?
- Are audit-doc ratifications all on-ledger?
- Are standing-program ledgers zero, not just predicate-trivial?

**Relationship to existing authority**:
- §1.6 (demonstration principle) governs demonstration-gate (a)/(b)/(c) minimum bar; this sheet checks that the bar was actually met per gate
- §1.7 (status taxonomy) governs DECLARED / CONSUMER_LANDED / PASSING transitions; this sheet checks that PASSING claims are evidence-backed
- §1.8 (ledger) is the per-gate state; this sheet is the meta-state over that ledger

**Out of scope**:
- Authoring new gates (that's §1.8 ledger work)
- Resolving DECLARED → PASSING transitions (that's per-gate Mgr work)
- R4 forward-looking acceptance (that's `WISHLIST.md`)

---

## §1. Per-gate predicate execution audit

- [ ] All 104 §1.8 state-check predicates have been EXECUTED at HEAD within 24h of close ceremony
- [ ] Execution log preserved as audit artifact at `docs/audit/r3-close-predicate-execution-YYYY-MM-DD.md`
- [ ] For each PASSING gate: predicate's actual output (count, command, file presence) matches the row's documented expectation
- [ ] No gate's predicate trivially passes via empty match-target (e.g., grep against non-existent path) without semantic check
- [ ] No gate marked PASSING based on predicate run from declaration-time only; close-time execution is required

---

## §2. Cross-doc ledger consistency

- [ ] §1.5 per-lane gate counts sum to §1.8 enumerated total
- [ ] §1.8 enumerated count (104 as of 2026-05-12) matches `docs/r3-structure.md` §"Acceptance" enumeration
- [ ] Q1 row in §"Q-decisions" (currently §10) sub-counts match §1.5 + §1.8 totals
- [ ] No gate ID gaps or duplicates in §1.8 (1..104 monotonically)
- [ ] All gate IDs referenced in `docs/audit/*.md` are present in §1.8
- [ ] `docs/r3-remaining-work-dependency-graph.md` gate-status snapshot consistent with §1.8 PASSING/DECLARED state

---

## §3. Semantic intent verification (gotcha audit)

For each gate, audit for "passes structurally, fails intent":

- [ ] **state-check** gates: predicate's match-target captures the semantic intent, not just textual form. Example gotcha: gate #104 Part A predicate `git grep "Lookup<"` would pass if `Lookup` is renamed to `Result`; verify the dissolution is structural (the Miss-deferral pattern is gone), not lexical.
- [ ] **demonstration** gates: §1.6 (a)/(b)/(c) bar verified per gate (end-to-end execution + observable output + ≥2 algebra-instance non-trivial input). No demonstration that runs in <50ms with single-line fixture.
- [ ] **substrate-shape** gates: each landed carrier has ≥1 consumer reading from it. No "carrier landed but unused" passes.
- [ ] **substrate-gap-class** gates: gap is structurally dissolved (existence-proof case works without bridge), not merely renamed or fenced.
- [ ] **count-check** gates: zero achieved via real elimination, not predicate narrowing. If predicate scope was narrowed during R3, audit the excluded scope for residuals.
- [ ] **substrate-gap-class** gates that involve "X works without Y" (e.g., gate #62 "no include_str!"): predicate verified against current HEAD AND against representative use-case program.

---

## §4. Cross-gate interaction

- [ ] Each "gates on X" dependency in §1.5/§1.8: X has compatible status with consumer (no PASSING gate references DECLARED-only predecessor in its authority chain)
- [ ] Lane-level demonstrations exercise ALL gates in the lane, not just convenient subsets
- [ ] Where two gates close the same class via different mechanisms (e.g., dissolution vs deletion): both mechanisms verified or one is unambiguously canonical
- [ ] Cluster sequencing (e.g., Cluster F sub-phases F-α/F-β/F-γ) all sub-gates closed before cluster-level gate marked PASSING

---

## §5. Standing-program ledger

- [ ] `r3_debt_paydown_zero_remaining` predicate executed at close: 0 results
- [ ] ROADMAP `Post-merge debt` rows: 0 entries
- [ ] §10 RED escalations: ALL CLOSED with owner sign-off recorded on-ledger
- [ ] sweep §1 Class A/B/C/F/G entries: 0 active
- [ ] No tracked-debt rows survive R3 close per §1.5 inclusion list

---

## §6. Audit-doc ratification trail

- [ ] Each `docs/audit/*.md` from R3 epoch (2026-04 onward) has one of: corresponding §1.8 gate, §10 closure entry, or explicit "R4-deferred" disposition
- [ ] No "ratified-but-not-gate-tracked" discipline lingers (per `feedback_grep_audit_docs_before_answering_close_questions`)
- [ ] All Director ratifications via dashboard messages (msg_XXXXXXXX) have on-ledger representation (either §1.8 row update or §10 closure)
- [ ] All operator directives (Brian sanctions at gunbc#846 + #828) have on-ledger representation
- [ ] No canvas-tier PR with "RATIFIED" disposition remains unreflected in §1.8

---

## §7. Behavioral demonstration coverage

- [ ] Every gate with `predicate-family = demonstration` satisfies §1.6 (a)/(b)/(c) minimum bar
- [ ] Each canonical lens (complexity + cost + parallelism + effect_enumeration) has behavioral-parity demonstration matching frozen v2-oracle snapshot (per gate #73)
- [ ] Pure Bootstrap zero achievable end-to-end (per gates #8 + #84) — verified by running PB-0 census on clean checkout
- [ ] L4 emit/eval match: full corpus exhaustion (per gate #9 graduation criterion), not just Rust/Int W1 slice
- [ ] L7 algebraic laws: exhaustive per-(algebra, inhabitant, law) coverage (per gate #10 graduation criterion), not bounded `Int` slice
- [ ] Self-host fixed point: `pb_self_compile_fixed_point` runs without v2 fallback (per gate #16)

---

## §8. Substrate fail-closed audit

- [ ] No `Lookup<C>::Miss` constructions in `src/v3/lenses/` + `src/v3/std/` (per gate #104 Part A predicate executed at close)
- [ ] No `::Miss\b` in generated lens code `src/v3/compiler/src/lens_*_generated.rs` (per gate #104 Part B)
- [ ] No `unreachable!()` or `panic!()` in lens read-channel code paths
- [ ] C-8 fail-closed discipline verified across §1.7 PASSING gates: every detectable problem produces a `Diagnostic`, no silent `None` or warning
- [ ] No `ArrowBody::Pending` residuals (per audit §3.6 sibling)

---

## §9. Hand-Rust ledger

- [ ] PB-0 census predicates report 0 hand-Rust survivors (per gates #8 + #84)
- [ ] All generated-code paths exercised by test harness (no dead generated code)
- [ ] `EXPECTED_HAND_AUTHORED_TEST` residuals match declared retirement schedule; no surprise survivors
- [ ] `lens_producer_files_remaining` per `ROADMAP.md` §"Lane acceptance — .dag gates" reaches zero
- [ ] `pb_rust_tests_outside_residual_zero` per `ROADMAP.md` T-PB-B row reaches zero

---

## §10. Documentation coherence

- [ ] `THESIS.md` cross-references to design docs all resolve (no 404 paths)
- [ ] `INVARIANTS.md` P1-P9: no §1.8 PASSING gate violates an INVARIANT
- [ ] `ROADMAP.md` milestones consistent with §1.8 closure (R3-closed milestone reflects ledger reality)
- [ ] `docs/r3-program-plan.md` §1.8 design-doc cross-references all live (no stale `:line-number` anchors)
- [ ] `docs/r3-remaining-work-dependency-graph.md` consistent with §1.8 status snapshot at close
- [ ] `WISHLIST.md` R4 items NOT promoted into R3 gates without explicit operator ratification (no silent scope leakage)
- [ ] `MODELING.md`, `CODING.md`, `TESTING.md` discipline anchors all reachable from §1.8 gate "Pass target" text

---

## §11. External-facing surfaces

- [ ] Full test suite passes on clean checkout (`cargo test --workspace`)
- [ ] `cargo fmt --all --check` clean
- [ ] `cargo clippy --all-targets -- -D warnings` clean
- [ ] No `EXPECTED_HAND_AUTHORED_TEST`-style harness-only test types beyond declared schedule
- [ ] Any breaking change in R3 has a migration note in commit message or PR body

---

## §12. Close ceremony

- [ ] Director sign-off recorded (zesty-bear-812 or successor role-node) on-ledger
- [ ] Operator sign-off recorded (Brian) on-ledger
- [ ] `ROADMAP.md` R3 → R3-closed milestone update merged
- [ ] §1.8 status sweep frozen: no further DECLARED → CONSUMER_LANDED transitions on R3 gates post-close
- [ ] R4 work-item creation greenlit by operator
- [ ] R3 close audit doc preserved: `docs/audit/r3-close-YYYY-MM-DD.md` capturing per-gate predicate execution output + this interrogation sheet's final disposition

---

## §13. Disposition tracking

Each item above resolves to one of:

| Status | Meaning |
|---|---|
| **NOT-CHECKED** | Item has not yet been audited (default state) |
| **PASSING** | Item verified at close-time HEAD; evidence preserved |
| **FAILING** | Item verified to fail; blocker for R3 close |
| **NEEDS-AUDIT** | Item attempted but evidence ambiguous; deeper investigation required |
| **N/A** | Item explicitly does not apply (with rationale recorded) |

For each item, record:
- **Status** (above)
- **Owner** — which session/role runs the check
- **Evidence** — where the check's output is preserved (audit doc anchor, log file, commit SHA)
- **Last-checked timestamp** (ISO-8601)
- **Notes** — deviations, exceptions, or context

R3 is closeable when:
- All required items in §1-§12 are PASSING or N/A
- Zero FAILING
- Zero NEEDS-AUDIT (resolve first)

---

## §14. Anti-patterns post-close (regression prevention)

To prevent R3-class debt from re-emerging in R4, post-R3 PRs should be reviewer-flagged for:

- New gate added to §1.8 without §1.7 status assignment
- `docs/audit/*.md` authored without §1.8 row OR §10 entry OR explicit R4-deferred disposition
- Demonstration gate added without §1.6 (a)/(b)/(c) bar receipt
- Predicate that trivially passes (empty match-target, narrowed scope without scope-justification)
- `Lookup<C>::Miss` or equivalent deferral-pattern re-introduction
- Hand-Rust added without PB-0 receipt or named retirement schedule
- Cross-doc count drift (§1.5 vs §1.8 vs r3-structure.md) without same-PR sync
- Audit doc ratification not reflected in §1.8 within same close window

---

## §15. Open questions for Director ratification

Items where PM-tier authoring needs Director disposition before finalizing this sheet:

- **Q1**: Should §1 (per-gate predicate execution audit) be a mechanized run (script walks §1.8 + executes each predicate, writes output to audit doc) or a manual per-gate sign-off?
- **Q2**: §6 (audit-doc ratification trail) — what's the canonical close-window? "From 2026-04 onward" or "since last R-close" or other?
- **Q3**: §12 close ceremony — does operator sign-off require formal directive at gunbc#846, or is dashboard-relayed sanction sufficient?
- **Q4**: §13 disposition — should NEEDS-AUDIT be a hard block on close, or can close proceed with named NEEDS-AUDIT items deferred to R4 with explicit migration?
- **Q5**: §14 anti-patterns — should any of these be reviewer-bot encoded (api-review providers flag automatically) or remain human-reviewer discipline?

Director / operator ratification on Q1-Q5 informs final shape of this sheet before R3-close ceremony.
