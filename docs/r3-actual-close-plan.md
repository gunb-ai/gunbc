# R3 Actual-Close Plan — Adversarial-Gap Disposition

**Status**: DRAFT (PM-authored 2026-05-13, pending Director ratification + operator approval)

**Authority chain**:
- Operator directive 2026-05-13 (verbatim): *"can we start on the planning docs to get to ACTUAL r3 close? like all of our adversarial questions answered positively? i feel like the planning for this stuff has been continuously dropped"*
- Adversarial audit conducted 2026-05-13 by PM (deep-wolf-155) against `docs/r3-close-interrogation.md` § promise enumeration
- Counterfactual surface: 10 substantive gaps found between Director's viz-as-SoT closure claim and THESIS-promise delivery on main
- This doc replaces "viz-as-SoT closed_at + DECLARED-strings-are-drift" framing with explicit per-gap disposition: PROVEN / WEAK-EVIDENCE / GAP-with-plan / R4-DEFERRED-with-operator-accepted

**Closure target**: every adversarial counterfactual either (a) cashed by a landed PR with on-main evidence, OR (b) explicitly R4-deferred with named operator acceptance recorded here, OR (c) reframed as not-R3-promised per THESIS authority.

**Closure NOT-target**: closing R3 with `closed_at` markers alone. The Director's procedural closure is necessary but not sufficient. The substantive closure requires the receipts named below.

---

## §1. Adversarial gaps + disposition plan

Each gap below has: **Promise** (verbatim quote + citation), **HEAD evidence** (what exists today), **What's missing** (the load-bearing gap), **Plan to cash** (concrete dispatch shape + owner + effort), **Close criterion** (the predicate that would let this gap flip to PROVEN).

---

### Gap 1 — PB-0 zero hand-Rust (gate #8 + #84)

**Promise** (THESIS.md `feedback_pb_zero_is_r3_close_target` + r3-program-plan.md §1.8): *"R3 close = 0 hand-Rust + 0 TESTING residual per §1.8 gates #8 + #84."*

**HEAD evidence**:
- `src/v3/compiler/tests/integration/sg0_census_test.rs:237` `EXPECTED_HAND_AUTHORED_NON_TEST` const carries **177+ entries** (the named-retirement-schedule)
- `EXPECTED_HAND_AUTHORED_TEST` const carries additional test residual
- Gate #8 row in §1.8 = **DECLARED**

**What's missing**: the 177+ hand-Rust survivors are not migrated to `.dag` substrate or eliminated. The ratchet shape is "no NEW additions"; the close criterion is "ratchet list = empty."

**Plan to cash**:
- **Owner**: warm-wolf-698 (R3 Substrate Mgr) for substrate work + zesty-boar-261 (R3 Debt-Paydown Mgr) for retirement campaign
- **Sub-program**: enumerate each entry's retirement path — (a) migrate to `.dag` carrier + walker, (b) eliminate via Practice-4 dissolution into existing substrate, (c) explicit R4-deferral with operator-recorded acceptance
- **Effort estimate**: ~3-6 months at current velocity (per memory `feedback_pre_authored_brief_queue` cadence)
- **Discipline**: every retirement PR ratchets list size DOWN by ≥1; never add to list; PR-template asserts ratchet direction

**Close criterion**:
```rust
// in sg0_census_test.rs at HEAD when R3 actually closes:
const EXPECTED_HAND_AUTHORED_NON_TEST: &[&str] = &[];
const EXPECTED_HAND_AUTHORED_TEST: &[&str] = &[];
```
plus `cargo test --release sg0_census_test` passes with predicate = 0.

**Alternative disposition (if operator accepts deferral)**: explicit R4 acceptance recorded here for specific subsets of the 177+ entries (e.g., grounding submodules may be Tier-2 R4-deferred per design-pure-bootstrap-zero.md). PM-recommended: do NOT R4-defer this in bulk — it's the load-bearing thesis claim.

---

### Gap 2 — L5 cross-target consistency (gate #15)

**Promise** (r3-structure.md §Acceptance + §3.1 of interrogation doc): *"for every `.dag` program, emitted Rust/Python/Go produce equivalent runtime behavior on the certification corpus."*

**HEAD evidence**:
- Gate #15 `l5_cross_target_consistency` = **DECLARED**
- Python emission: only `src/v3/compiler/tests/boundary/m1_4_emit_python_test.rs` (boundary test, not lane acceptance)
- Go emission: no equivalent boundary test
- No `.dag → {Rust, Python, Go}` stdout-parity demo on main

**What's missing**: certification corpus + 3-target emission per corpus program + stdout-parity assertion.

**Plan to cash**:
- **Owner**: swift-deer-459 (R3 Verification Mgr) + warm-wolf-698 (for any missing carrier-level support)
- **Sub-program** (5 phases):
  1. **Define certification corpus**: enumerate the `.dag` programs that constitute the L5 corpus; ratify with operator (estimate 10-20 programs covering 5 dimensions × 3 algebra-classes)
  2. **Land Python emitter** as lane-acceptance code path (not boundary-test scope)
  3. **Land Go emitter** as lane-acceptance code path
  4. **Per-corpus-program L5 assertion**: each program runs through Rust + Python + Go; stdout compared; assertion fail-closed on divergence
  5. **CI integration**: corpus runs on every PR; ratchet on coverage growth
- **Effort estimate**: 4-8 weeks (Python emitter alone is significant; Go follows pattern)

**Close criterion**:
```bash
# Predicate at gate #15 close:
cargo test --release -p v3-compiler --test l5_cross_target_consistency
# returns: PASS with N>0 certification-corpus programs, all 3 targets agreeing on stdout
```
plus §1.8 row #15 status flips DECLARED → PASSING with corpus enumeration cited.

**Alternative disposition**: if operator accepts Python/Go as R4-deferred (R3 = Rust-only + Shape-B SQL/Markdown/OpenAPI), record acceptance here + amend §3.1 to claim only Rust as R3-shipped Shape-A. This re-scopes the 3-target story.

---

### Gap 3 — Self-host fixed point (gate #16)

**Promise** (r3-structure.md §"Acceptance" + §4.2 of interrogation doc): *"bit-identical stage0 + emitted artifact" — the compiler can compile itself and emit a bit-for-bit identical compiler.*

**HEAD evidence**:
- Gate #16 = **CONSUMER_LANDED (R1 horizon; R3 stronger interpretation pending)**
- 4 joint preconditions deferred per parent brief:
  - R2-Evaluator landed
  - R2-Grounding-Rust+Python landed
  - T-LP/SG-0 landed (= Gap 1 above)
  - Row-B materialization landed

**What's missing**: the 4 joint preconditions all need to land before R3-strong-form self-host can be claimed.

**Plan to cash**:
- **Owner**: Director-tier coordination (cross-Mgr); PM tracks
- **Sub-program**: each precondition has its own program; this gap is meta-blocked
  - R2-Evaluator: separate program; current status unknown to PM (need Director audit)
  - R2-Grounding-Rust+Python: gate #18 + extdeps work
  - T-LP/SG-0: same as Gap 1
  - Row-B materialization: T-LBP work
- **Effort estimate**: 2-4 months (joint precondition; bounded by the longest of 4)

**Close criterion**:
```bash
# Predicate at gate #16 R3-strong-form close:
self_host_fixed_point && diff -q <stage0_binary> <emitted_binary>
# returns: byte-identical
```
All 4 precondition gates PASSING + actual self-host invocation producing bit-identical output.

**Alternative disposition**: if operator accepts R1-horizon as R3-acceptable (compiler self-compiles but bit-identity only on R1-subset), record acceptance here + amend §4.2 promise text to scope-narrow the R3 claim.

---

### Gap 4 — Lens behavioral parity (gates #79 / #81 / #82 / #83)

**Promise** (r3-program-plan.md §1.6 Cluster F + interrogation §1.1/§1.3/§1.4): *"4 in-R3 lenses (complexity + cost + parallelism + effect_enum) behaviorally complete; lens capability register has zero proxy / zero stub."*

**HEAD evidence**:
- #79 complexity = **SATISFIED-BY-CONSTRUCTION** via temporary Rust cementing receipt (`complexity_lens_behavioral_completion.rs`) "until ComplexitySummary TestClaim literals are expressible"
- #80 cost = **PASSING** ✓
- #81 parallelism = **R3-LOAD-BEARING**, F-α sub-phase pending (Stage 2e walker port from `workflow_parallelism.rs` → `.dag`)
- #82 effect_enum = **R3-LOAD-BEARING**, F-β.1 canvas + F-β.2 atomic-migration pending
- #83 lens_capability_register_zero_proxy_zero_stub = **DECLARED** (proxies/stubs still exist for 3 of 4 lenses)

**What's missing**:
- Complexity: `ComplexitySummary` TestClaim literals (bridge-Rust to native-.dag migration)
- Parallelism: Cluster F sub-phase F-α (Stage 2e walker port)
- Effect: Cluster F sub-phase F-β.1 (canvas) + F-β.2 (atomic-migration impl)
- Register: zero-proxy-zero-stub for all 4 lenses post-sub-phase landings

**Plan to cash**:
- **Owner**: warm-wolf-698 (Substrate Mgr) for canvases + worker dispatch
- **Sub-program** (in dependency order):
  1. F-α (parallelism walker port) — worker brief pending dispatch
  2. F-β.1 (effect canvas) — Substrate Mgr canvas authoring
  3. F-β.2 (effect atomic-migration) — worker dispatch post-canvas-ratification
  4. ComplexitySummary TestClaim literals — substrate work to enable native-.dag complexity assertions
  5. F-γ.2 (post-all-4-lenses-complete register sweep) — gate #83 close
- **Effort estimate**: 4-8 weeks for all 4 sub-phases (warm-wolf-698 sequential cadence per `feedback_parallel_canvas_sequential_authoring`)

**Close criterion**: §1.8 rows #79 (PASSING with native-.dag witness, no Rust cementing receipt), #81 PASSING, #82 PASSING, #83 PASSING.

---

### Gap 5 — Tests-as-data completeness (gates #84 + #85)

**Promise** (THESIS facet 3 + r3-program-plan.md §1.8 #84): *"every Rust test ports to `.dag` or is generated."*

**HEAD evidence**:
- Gate #84 `every_rust_test_ports_to_dag_or_generated` = **DECLARED**
- Gate #85 `forall_exists_quantifier_substrate_landed` = **DECLARED** (carriers landed; generated consumer pending per §P2)
- Gate #86 `program_generator_carrier_landed` = **CONSUMER_LANDED + PASSING** ✓
- Gate #87 `lens_cementing_test_discipline_complete` = **CONSUMER_LANDED + PASSING** ✓
- Per Q-PB0-ClusterM-Cold-Risk6 verbatim: *"Severity: load-bearing-blocking — without Cluster M dispatch, gate #84 cannot close inside 8-12 week R3 window."*
- 99 hand-Rust tests in `src/v3/compiler/tests/integration/`

**What's missing**:
- Cluster M Phase 3 bulk-port (per-class workers) — 80-90 of 101 SG-0 test entries dissolve via this single bulk event
- Gate #85 generated consumer (SuiteClaim wrapper migration per design §6 line 344)

**Plan to cash**:
- **Owner**: swift-deer-459 (R3 Verification Mgr) + per-class workers
- **Sub-program** (per `docs/audit/r3-cluster-m-sequencing-plan-2026-05-09.md`):
  - Phase 1 (#86): LANDED ✓
  - Phase 1a (#85 carriers): LANDED ✓
  - Phase 2 (#87 receipts): LANDED ✓
  - **Phase 3 (#84 bulk-port)**: 99 tests × per-class dispatch — per-class worker briefs needed
- **Effort estimate**: 4-8 weeks per Cluster M plan (operator "staffing not a concern" allows parallel dispatch)

**Close criterion**:
```bash
# Predicate at gate #84 close:
find src/v3/compiler/tests/integration -name "*.rs" -not -path "*/common/*" -not -path "*/boundary/*" | wc -l
# returns: 0 (or only generated-from-.dag survivors)
```
plus #85 SuiteClaim wrapper consumer landed.

---

### Gap 6 — v2 retirement terminal (gate #97)

**Promise** (interrogation §5.2): *"v2 fully retired."*

**HEAD evidence**:
- Gate #97 `method_template_projection_emit_shim_retirement_coherence` = **PASSING** for coherence-only
- Per gate row verbatim: *"Terminal retirement = v2 tree absent ⇒ lib + bin table + bin source absent; coherence holds continuously."*
- `src/v2/` exists at HEAD (terminal not reached)

**What's missing**: actual deletion of `src/v2/` tree.

**Plan to cash**:
- **Owner**: zesty-boar-261 (R3 Debt-Paydown Mgr) — terminal-deletion sweep
- **Sub-program**:
  1. Final v2 consumer audit (any `src/v2/` consumers remaining?)
  2. Delete `src/v2/` + dependent build manifests + workspace member
  3. Verify gate #97 coherence: lib absent ✓ + bin table empty ✓ + bin source absent ✓
- **Effort estimate**: 1-2 weeks (mostly audit; deletion is mechanical)
- **Dependency**: gates #16 (self-host fixed point) AND #84 (tests-as-data) precondition — can't delete v2 until v3 can self-host AND tests don't depend on v2 oracles

**Close criterion**: `ls src/v2/ 2>&1 | grep "No such file"` returns true. §1.8 row #97 terminal-PASSING.

---

### Gap 7 — T-WAD FULL R3 (gates #98-#103)

**Promise** (r3-program-plan.md §1.8 #98-#103, ratified 2026-05-12 per Director msg_5cbdad24 + msg_f9fd669e + operator directive 2026-05-12): *"Workflow-as-data FULL R3 — ci.yml is structural emission output not hand-authored; affected-set lens used in CI selection."*

**HEAD evidence**:
- All 6 gates (#98 / #99 / #100 / #101 / #102 / #103) = **DECLARED**
- `.github/workflows/ci.yml` still **hand-edited** at HEAD
- `project_github_actions` projection function not landed
- T-WAD Slice 4 (YamlStatic emitter) in-flight at PR #2816 (open)
- Affected-set lens has design doc (`docs/design-affected-set-lens.md`); no §1.8 ratification

**What's missing**: 5-slice cascade (Slice 4 substrate → Slice 5 ci.yml swap → Slice 6 binary-shim → Slice 7 affected-set → Slice 8 substrate completion).

**Plan to cash**:
- **Owner**: zesty-boar-261 (R3 Debt-Paydown Mgr) per T-WAD lane assignment + sharp-deer-576 for Slice 6 cost-dim work
- **Sub-program** (per cascade):
  1. Slice 4: PR #2816 YamlStatic emitter — IN-FLIGHT
  2. Slice 5: ci.yml swap to emission-output
  3. Slice 6: BinaryShim runtime + slow-test-exemptions dissolution
  4. Slice 7: affected-set lens CI integration
  5. Slice 8: substrate completion (gate #99 #100 #101 ratifications)
- **Effort estimate**: 4-6 weeks (Slice 4 nearly landed; rest follows quickly)

**Close criterion**: §1.8 rows #98-#103 all PASSING. `.github/workflows/ci.yml` is byte-identical to `project_github_actions(ci_workflow_dag, YamlStatic)` output.

---

### Gap 8 — Bootstrap-seed Rust survivors

**Promise** (design-pure-bootstrap-zero.md:41 + :210): *"0-floor target; machine-emitted bootstrap-seed is the only acceptable form."*

**HEAD evidence**: bound to Gap 1 — hand-authored survivors persist on the ratchet.

**Plan to cash**: same as Gap 1 (PB-0 retirement); this gap is folded into the SG-0 census campaign.

**Close criterion**: same as Gap 1.

---

### Gap 9 — Show-the-correct-code (THESIS:103-105)

**Promise** (THESIS.md:103-105): *"Diagnostics should point to the structurally correct program, not just report that the current one is wrong."*

**HEAD evidence**:
- **No §1.8 gate** exists for this promise
- `docs/audit/` does not have a "show-correct-code coverage" tracking doc
- Closure ledger doesn't carry it

**What's missing**: §1.8 gate + per-diagnostic-class audit + close criterion.

**Plan to cash**:
- **Owner**: PM (deep-wolf-155) authors the new §1.8 row + Verification Mgr (swift-deer-459) implements
- **Sub-program**:
  1. Author new §1.8 row #106 `show_correct_code_diagnostic_coverage` with substrate-shape gate type
  2. Enumerate diagnostic classes (parse / type / lens / emit / ...)
  3. For each class, audit existing diagnostics + check whether they cite "Y would be right" or only "X is wrong"
  4. Author canvas for the substrate shape that admits Diagnostic-with-correction
  5. Worker brief dispatch to retrofit existing diagnostics
- **Effort estimate**: 4-8 weeks (depends on diagnostic class count; potentially smaller if substrate already supports it)

**Close criterion**: §1.8 row #106 PASSING; every Diagnostic in `src/v3/compiler/` has a `correction: Option<Witness>` field that is `Some(_)` for ≥80% of fired diagnostics in test corpus.

**Alternative disposition**: if operator accepts that show-correct-code is THESIS-aspirational not R3-promised, record explicit not-R3-scoped here + amend interrogation doc §6 to reflect.

---

### Gap 10 — Close-audit doc absent (interrogation §8)

**Promise** (r3-close-interrogation.md §8): *"All 104 §1.8 predicates executed at HEAD within 24h of close ceremony. Execution log preserved as audit artifact."*

**HEAD evidence**:
- No `docs/audit/r3-close-predicate-execution-2026-05-13.md` exists
- The close-ceremony self-check doc has not been authored

**What's missing**: the close-ceremony doc.

**Plan to cash**:
- **Owner**: PM (deep-wolf-155) authors the doc; Verification Mgr (swift-deer-459) executes predicates
- **Sub-program**:
  1. PM authors skeleton: 105 rows, one per §1.8 gate, with column for predicate execution status
  2. Verification Mgr runs each predicate at HEAD; records pass/fail + output snippet
  3. PM closes out the doc with verdict per row + overall R3-close verdict
- **Effort estimate**: 1-2 days (execution-only; depends on remote build availability)

**Close criterion**: `docs/audit/r3-close-predicate-execution-YYYY-MM-DD.md` exists on main with ALL 105 rows filled + overall verdict cited.

**Note**: this gap is the *receipt* for the closure ceremony. It can land at any time (not blocked on other gaps). Authoring this should be PM's immediate next deliverable.

---

## §2. Dispatch sequencing (PM-recommended)

Given the cross-gap dependencies, recommended dispatch order:

**Phase A — Immediate (PM-direct, 1-2 weeks)**:
- Gap 10: PM authors close-audit doc skeleton; dispatches predicate execution
- Gap 9: PM proposes §1.8 row #106 to Director for ratification

**Phase B — Substrate Mgr lane (warm-wolf-698, 4-8 weeks)**:
- Gap 4 (lens behavioral parity): F-α → F-β.1 → F-β.2 → ComplexitySummary → F-γ.2 register sweep
- Gap 7 (T-WAD): Slice 4 → 5 → 6 → 7 → 8 cascade (zesty-boar-261 + sharp-deer-576 lane)

**Phase C — Verification Mgr lane (swift-deer-459, 4-8 weeks)**:
- Gap 5 (Cluster M Phase 3): 99-Rust-test bulk-port dispatch
- Gap 2 (L5 cross-target): certification corpus + Python/Go emitters

**Phase D — Debt-Paydown lane (zesty-boar-261, ~3-6 months)**:
- Gap 1 (PB-0): 177-entry retirement campaign
- Gap 6 (v2 terminal): gated on Gap 1 + Gap 3

**Phase E — Director-tier coordination (zesty-bear-812, parallel)**:
- Gap 3 (self-host fixed point): 4-joint-precondition cross-Mgr audit

**Phase F — Final close ceremony (PM-direct, 1 week)**:
- Re-run close-audit doc with all gaps PROVEN or R4-DEFERRED-with-acceptance
- Operator final ratification
- Bookkeeping batch PR per Director's identified hygiene fix (105 stale §1.8 strings → sync to closed_at reality)

---

## §3. Total estimated time-to-actual-close

**Optimistic**: 8-12 weeks with parallel Mgr dispatch + operator "staffing not a concern" framing.

**Realistic**: 12-20 weeks accounting for:
- Joint preconditions on Gap 3
- Cluster M Phase 3 bulk dispatch coordination (Gap 5)
- 177-entry retirement campaign velocity (Gap 1; the longest tail)

**Pessimistic**: 6+ months if Gap 1 cannot be parallelized aggressively (PB-0 is the longest tail; everything else can run faster).

---

## §4. Operator decision points (request for ratification)

Before dispatching this plan, PM requests operator decision on:

1. **Gap 1 (PB-0)**: full 177-entry retirement IN-R3, OR partial R4-deferral with explicit list?
2. **Gap 2 (L5 cross-target)**: full 3-target Python+Go IN-R3, OR Rust-only-Shape-A with Python+Go R4-deferred?
3. **Gap 3 (self-host R3-strong)**: 4-joint-precondition cascade IN-R3, OR R1-horizon acceptable with strong-form R4-deferred?
4. **Gap 9 (show-correct-code)**: new §1.8 gate IN-R3, OR THESIS-aspirational-not-R3-promised reframe?

These 4 decisions determine the actual R3 scope. Without them, the plan above optimistically assumes all gaps cash IN-R3, which is the longer time horizon.

---

## §5. Process discipline (preventing future drop)

Operator framing 2026-05-13: *"planning for this stuff has been continuously dropped."*

Anti-pattern observed: closure-ceremony work is ad-hoc and gets bumped by reactive work (PR reviews, daily merges, Mgr coordination). The closure receipt (Gap 10) was the first thing to drop.

**Process discipline going forward**:
1. **This doc is the single authoritative closure plan**. Updates land via PR with operator review. No parallel planning docs.
2. **Weekly PM closure-cadence message to Director + operator**: progress vs plan, gaps closing, blockers surfaced. Cadence: every Monday.
3. **Per-gap closure PR template**: each gap-closure PR must cite the gap number from this doc + the close criterion satisfied + audit evidence path.
4. **Gap 10 (close-audit doc) is authored FIRST** — it's the receipt mechanism for everything else. Without it, "closed" is unverifiable.

---

## §6. Pending Director + operator decisions

- [ ] Director ratifies this plan structure (PM-recommended)
- [ ] Operator approves §4 scope decisions (4 binary IN-R3 / R4-defer choices)
- [ ] Operator authorizes Phase A immediate dispatch (close-audit doc + §1.8 row #106 author)
- [ ] PM dispatches Phase B + Phase C briefs post-ratification

---

**Authored by**: deep-wolf-155 (PM)
**Date**: 2026-05-13
**Status**: DRAFT pending Director ratification + operator scope approval
