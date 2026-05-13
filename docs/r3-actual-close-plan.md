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
  2. **Gate-#87 cementing-receipt re-launch for parallelism** — content-source from **closed PR #2860** (`session/crisp-owl-183`, closed 2026-05-13T16:45:44Z under operator cleanup directive; flagged by Director msg_b3324a05 as load-bearing): `regen.dag` parallelism row needs matching gate-#87 cementing harness; `r3_gate_87_lens_cementing_regen_receipts_test::r3_gate_87_regen_lens_registry_names_match_fixture_inventory` fails closed at HEAD on inventory mismatch (`{"parallelism"}` extra in regen.dag without paired test). PR #2860 disposition: parallelism is `LensCapabilityBehavioralPartial` + `LensCapabilityV2NoneV3Native` per `src/v3/std/verification.dag:213-218` + `docs/v3-lens-capability-register.md:45`; G87-C-disciplined receipt = explicit `Compiles` placeholder + paired Rust pin + named dissolution trigger naming public `WorkflowParallelismReport` / `ParallelismUnsupportedDetail` typed-pairwise-evidence routing; mirrors infer_helpers / lower_helpers / variant_payload pattern. Concrete artifact: `src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_parallelism.dag`. Retrieve via `gh pr view 2860 --json body` or `gh pr diff 2860`.
  3. F-β.1 (effect canvas) — Substrate Mgr canvas authoring
  4. F-β.2 (effect atomic-migration) — worker dispatch post-canvas-ratification
  5. ComplexitySummary TestClaim literals — substrate work to enable native-.dag complexity assertions
  6. F-γ.2 (post-all-4-lenses-complete register sweep) — gate #83 close
- **Effort estimate (single-Mgr sequential)**: 4-8 weeks for all 5 sub-phases (warm-wolf-698 sequential cadence per `feedback_parallel_canvas_sequential_authoring`)
- **Effort estimate (parallelized — Director feedback item 3)**: F-α + F-β.1 + ComplexitySummary substrate can parallelize under a second Substrate Mgr per operator "staffing not a concern" framing. Tightens timeline to ~2-4 weeks at cost of one additional Mgr-tier dispatch. Sequential-cadence is a Mgr-bandwidth choice, not a hard constraint; surfacing as a tightening lever for operator/Director.

**Close criterion**: §1.8 rows #79 (PASSING with native-.dag witness, no Rust cementing receipt), #81 PASSING, #82 PASSING, #83 PASSING, #87 ratchet-pass at HEAD.

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

**Close criterion** (Director feedback item 4 — header-marker filter; **codex BLOCKING PR #3013 2026-05-13 enforcement: boundary-test carve-out RETRACTED** per `TESTING.md` post-PR-#678 cascade + `docs/design-pure-bootstrap-zero.md` 0-floor target — class-5 boundary tests migrate to `ExecuteCommand`-based `.dag` `TestClaim` declarations, NOT preserved as hand-authored survivors):
```bash
# Predicate at gate #84 close — ALL hand-authored *_test.rs counted; only generated-from-.dag tests survive:
find src/v3/compiler/tests -name "*.rs" -not -path "*/common/*" \
  | xargs grep -L "// AUTO-GENERATED FROM .dag" \
  | wc -l
# returns: 0 (only generated-from-.dag survivors carry the marker; hand-authored fail-closed; boundary tests ARE included, must migrate to ExecuteCommand-based .dag TestClaim per TESTING.md cascade)
```
plus #85 SuiteClaim wrapper consumer landed.

**Authority for no-carve-out** (codex BLOCKING PR #3013 receipt):
- `TESTING.md` "🔄 RETRACTED 2026-04-25 (cascade promotion of `docs/design-pure-bootstrap-zero.md`)": *"the previous 'two residual categories' framing (compiler-internal unit tests + external-toolchain boundary tests stay Rust-authored permanently) is retracted under the 0-floor target. Both categories dissolve"*
- `TESTING.md` "0-residual is the target. Until v3's source tree reaches 0 hand-authored files..."
- `docs/design-pure-bootstrap-zero.md`:41 *"Goal: zero hand-authored files in v3's source tree."*
- `docs/design-pure-bootstrap-zero.md`:138 boundary-tests migrate to `ExecuteCommand`-based `.dag` `TestClaim` declarations per cascade-named successor pattern (PR #678 schema landed; bulk migration is Gap 5 Phase 3 scope)

**Substrate prereq**: code-gen pipeline must emit `// AUTO-GENERATED FROM .dag` header line for every generated `*_test.rs` so the predicate can mechanically discriminate hand-authored vs generated. If not present at HEAD, lands in Gap 5 Phase 3 ratchet.

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
- **Dependency** (Director feedback item 5 — explicit transitive depth call-out): Gap 6 has the deepest transitive chain in the program:
  - Gap 6 → depends on Gap 1 (PB-0 retirement complete) + Gap 3 (self-host fixed point R3-strong)
  - Gap 3 → depends on Gap 1 + R2-Evaluator + R2-Grounding-Rust+Python + Row-B materialization
  - Effective chain: **Gap 6 ← Gap 3 ← {Gap 1, R2-Evaluator, R2-Grounding, Row-B}** — 5+ deep
- **Position in close ceremony**: **Gap 6 IS the close-ceremony terminal gate** (per Director recommendation msg_cd2d8d7d item 5). v2 tree deletion is the FINAL R3 dissolution; the entire ratchet → carrier → walker → emission → self-host → test-port cascade closes upstream first. Expect Gap 6 to land in the last 2 weeks of R3 close.

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
- **Owner**: PM (deep-wolf-155) authors the new §1.8 row → Substrate Mgr (warm-wolf-698) authors canvas → Verification Mgr (swift-deer-459) implements
- **Sub-program**:
  1. Author new §1.8 row #106 `show_correct_code_diagnostic_coverage` with substrate-shape gate type
  2. Enumerate diagnostic classes (parse / type / lens / emit / ...)
  3. For each class, audit existing diagnostics + check whether they cite "Y would be right" or only "X is wrong"
  4. **Substrate-shape canvas authored by Substrate Mgr BEFORE worker dispatch** (per claude review exploratory observation #2 — this plan doc surfaces the `correction: Option<Witness>` field as a sub-program step, but the actual substrate-shape commitment must be ratified via Mgr canvas, not implemented from this prose). Canvas authors the substrate shape that admits Diagnostic-with-correction; Director ratifies; worker dispatches against ratified shape.
  5. Worker brief dispatch to retrofit existing diagnostics
- **Effort estimate**: 4-8 weeks (depends on diagnostic class count; potentially smaller if substrate already supports it). **Caveat (per claude review exploratory observation #3 — estimates unsourced)**: this estimate is PM-prior-cycle-experience-based, NOT cited against specific velocity data. Substrate Mgr canvas surfaces the actual scope + worker effort; final estimate calibrates post-canvas-ratification.

**Close criterion** (codex BLOCKING PR #3013 2026-05-13 enforcement: **absolute 100% — pragmatic-relaxation alternative RETRACTED** as it converted a load-bearing THESIS promise into a negotiable threshold):
- **THESIS-correct (100% absolute)**: every Diagnostic in `src/v3/compiler/` has `correction: Witness` field present + `Some(_)` for **every** fired diagnostic in test corpus. No threshold relaxation. THESIS.md "Error handling: show the correct code" — *"Diagnostics should point to the structurally correct program, not just report that the current one is wrong"* — reads as absolute promise.

**Authority for no-relaxation** (codex BLOCKING PR #3013 receipt): the prior "Pragmatic relaxation (≥X%)" framing dilutes an absolute THESIS commitment into a negotiable threshold without THESIS-text reconciliation. Per `project_no_r4_carves_directive` (2026-05-08), R4-carve framing is NOT freely available; within-R3 relaxation paths are structurally equivalent to R4-carves under the directive. The alternative-disposition path below (R4-scope reframe with operator-recorded acceptance) is the ONLY non-100% path, and it requires explicit operator override of the no-carves directive — not a within-R3 threshold negotiation.

**Alternative disposition**: if operator accepts that show-correct-code is THESIS-aspirational not R3-promised, record explicit not-R3-scoped here + amend interrogation doc §6 to reflect. **Note per standing directive**: this alternative requires operator override of `project_no_r4_carves_directive` since "THESIS-aspirational-not-R3-promised" is structurally an R4-carve.

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
- **Effort estimate** (Director feedback item 7 — calibrated):
  - **Skeleton authoring (PM-direct)**: 1-2 days — 105-row table scaffold + per-row column shape + initial categorization
  - **Predicate execution (Verification Mgr serial)**: 1-2 weeks — 105 individual `cargo test` / `grep` invocations + result capture + per-row writeup
  - **Predicate execution (Verification Mgr parallel via ctrl-build)**: 3-5 days compressed if remote BuildBuddy infra healthy + parallelism dispatch
  - **Overall**: 1-2 weeks for full landing, contingent on Verification Mgr capacity + ctrl-build availability

**Close criterion**: `docs/audit/r3-close-predicate-execution-YYYY-MM-DD.md` exists on main with ALL 105 rows filled + overall verdict cited.

**Note**: this gap is the *receipt* for the closure ceremony. Skeleton authoring (1-2 days) is not blocked on other gaps and is PM's immediate next deliverable; predicate execution (1-2 weeks) can run in parallel with other Phase B/C/D work.

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

**Phase F — Final close ceremony (PM-direct + operator, 1-2 weeks)**:
- Re-run close-audit doc with all gaps PROVEN or R4-DEFERRED-with-acceptance
- **Operator + PM adversarial re-pass (operator directive 2026-05-13 — final closeout discipline)**: every claim in `docs/r3-close-interrogation.md` + `docs/r3-actual-close-plan.md` + every §1.8 row status is re-interrogated against HEAD evidence with operator playing adversarial-stakeholder. Symmetric to the 2026-05-13 adversarial sweep that surfaced the 10 counterfactuals — but applied at close-ceremony to confirm none survived. Anti-pattern caught: "looks closed by procedural metric" without substantive HEAD-evidence cross-check. Discipline: every PROVEN claim has a paired predicate-execution receipt + grep-at-HEAD witness; no claim survives the adversarial-re-pass on assertion alone.
- Operator final ratification
- **Bookkeeping batch PR (Director feedback item 8 — downstream of predicate-execution verdict + adversarial-re-pass verdict, NOT parallel to either)**: §1.8 manifest strings synchronize to **close-audit-doc predicate-execution outcome PLUS adversarial-re-pass survivor status** (substantive View-4 authority per `feedback_r3_close_three_views_drift`), NOT to procedural `closed_at` markers. Strings sync as: row #N status = whatever the close-audit-doc row #N predicate evaluation returned AFTER adversarial-re-pass confirmed no counterfactuals survive. Sequencing: close-audit-doc lands first (Gap 10 execution complete) → operator+PM adversarial-re-pass against doc + interrogation + §1.8 → bookkeeping PR consumes the adversarial-re-pass verdict as authority + amends §1.8 strings to match. This avoids re-introducing the procedural-closure trap by always sourcing manifest state from predicate-execution outcome PLUS operator-verified absence of counterfactuals.

---

## §3. Total estimated time-to-actual-close

**Optimistic**: 8-12 weeks with parallel Mgr dispatch + operator "staffing not a concern" framing.

**Realistic**: 12-20 weeks accounting for:
- Joint preconditions on Gap 3
- Cluster M Phase 3 bulk dispatch coordination (Gap 5)
- 177-entry retirement campaign velocity (Gap 1; the longest tail)

**Pessimistic**: 6+ months if Gap 1 cannot be parallelized aggressively (PB-0 is the longest tail; everything else can run faster).

**Velocity-citation discipline** (per claude review exploratory observation #3 — most estimates are unsourced):
- Gap 1: cited against `feedback_pre_authored_brief_queue` cadence + observed SG-0 ratchet-shrink rate over prior R3 cycles
- Gap 2/3/5/6/7/9: PM-prior-cycle-experience-based, NOT cited against specific velocity data. Final ratified version (post-operator-§4-confirmations) should:
  1. Cite per-gap velocity reference (e.g., "Cluster M Phase 1 + 1a + 2 landed in N weeks per PR #2645/#2647/#2639/#2757 cadence; Phase 3 bulk-port estimate extrapolates")
  2. Update at first weekly closure-cadence message (§5 process discipline) once first-week dispatch lands actual delta-velocity data
- Gap 10: source = direct execution count (105 predicates × {serial 1-2wk OR parallel 3-5d via ctrl-build})

This caveat applies to the entire §3 — estimates above are PM-best-guess at draft authorship; precision improves as dispatch progresses and weekly closure-cadence messages calibrate against actual landing dates.

---

## §4. Operator decision points (request for ratification)

**Standing directive context** (per `project_no_r4_carves_directive`, Brian 2026-05-08 verbatim: *"we are NOT moving anything to R4 as of now"*): R4-carve is **NOT freely available** as a default. The 4 decisions below default to **IN-R3** unless operator explicitly overrides the standing directive with a structural-unblockable-reason argument per-decision.

**PM recommendation across all 4** (per claude review exploratory observation #1 — explicit per-gap PM view): **do not defer** any of Gaps 1/2/3/9. Each R4-carve materially dilutes a load-bearing R3 promise:
- Gap 1 R4-carve = dilutes PB-0 thesis claim ("0 hand-Rust")
- Gap 2 R4-carve = scope-narrows §3.1 from 3-Shape-A targets to Rust-only (defeats omni-emission story)
- Gap 3 R4-carve = scope-narrows §4.2 self-host fixed point to R1-horizon (defeats self-host thesis claim)
- Gap 9 R4-carve = drops THESIS:103-105 absolute promise

PM requests operator confirmation (default IN-R3) or explicit override (R4-carve with stated structural-unblockable reason) on:

1. **Gap 1 (PB-0)**: IN-R3 default = full 177-entry retirement. **PM-recommended: do not defer.** R4-carve override requires operator to name specific subsets + structural reason.
2. **Gap 2 (L5 cross-target)**: IN-R3 default = full 3-target Python+Go. **PM-recommended: do not defer.** R4-carve override scope-narrows §3.1 to Rust-only Shape-A; the omni-emission falsifier promise (R4.A architectural-falsifier) loses Python/Go round-trip validation paths.
3. **Gap 3 (self-host R3-strong)**: IN-R3 default = 4-joint-precondition cascade. **PM-recommended: do not defer.** R4-carve override accepts R1-horizon as R3-final and amends §4.2 promise text.
4. **Gap 9 (show-correct-code)**: IN-R3 default = new §1.8 gate + Diagnostic-with-correction coverage at **100% absolute** (no threshold negotiation; codex BLOCKING PR #3013 2026-05-13 retracted prior `≥X%` pragmatic-relaxation framing as THESIS-promise-dilution). **PM-recommended: do not defer.** Single operator sub-decision:
   - (a) IN-R3 at 100% absolute (recommended) OR not-R3-promised reframe (= R4-carve per directive; requires explicit operator override of `project_no_r4_carves_directive`)

**§5 process discipline note**: per the standing directive, asking the operator to choose IN-R3-vs-R4-carve framing for these 4 items implicitly invites R4-carve consideration. Re-framing per Director feedback item 1: the question is "confirm IN-R3 (default, per directive + PM-recommended)" — explicit override only if structurally unblockable.

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

- [x] **Director ratifies this plan structure** — APPROVED 2026-05-13 (msg_cd2d8d7d) on structure + dispatch sequencing + §5 process discipline. 8 substantive notes integrated into this revision.
- [ ] Operator §4 confirmations (4 decisions; default IN-R3 per `project_no_r4_carves_directive`):
  - [ ] Gap 1 (PB-0): confirm IN-R3 OR override with named R4-carve subsets
  - [ ] Gap 2 (L5 cross-target): confirm IN-R3 (Python+Go) OR override with Rust-only-Shape-A scope
  - [ ] Gap 3 (self-host R3-strong): confirm IN-R3 (4-joint-precondition cascade) OR override with R1-horizon final
  - [ ] Gap 9 (show-correct-code): (a) confirm IN-R3 + ratify threshold = 100% (THESIS-correct) OR ≥X% (pragmatic, X TBD); (b) override with not-R3-promised reframe
- [ ] Operator authorizes Phase A immediate dispatch (close-audit doc skeleton + §1.8 row #106 author)
- [ ] Director-tier deliverables in-flight per msg_cd2d8d7d:
  - [ ] R2-Evaluator audit (Gap 3 precondition; this week)
  - [ ] Gap 3 cross-Mgr coordination tracking (ongoing, Phase E)
  - [ ] §1.8 row #106 substrate-shape canvas ratification when PM surfaces it
  - [ ] Pre-execution review of close-audit-doc rows touching Director-tier ratification gates (gate #105 / canvas-ratified-class)
- [ ] PM dispatches Phase B + Phase C briefs post-§4-ratification

---

**Authored by**: deep-wolf-155 (PM)
**Date**: 2026-05-13
**Status**: DRAFT pending Director ratification + operator scope approval
