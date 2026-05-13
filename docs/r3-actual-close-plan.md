# R3 Actual-Close Plan — Adversarial-Gap Disposition

**Status**: FULLY RATIFIED 2026-05-13 — Director structure-ratification (msg_cd2d8d7d) + operator §4 scope + Mgr-dispatch ratification (briansrls 2026-05-13). READY FOR DISPATCH post-merge.

**Authority chain**:
- Operator directive 2026-05-13 (verbatim): *"can we start on the planning docs to get to ACTUAL r3 close? like all of our adversarial questions answered positively? i feel like the planning for this stuff has been continuously dropped"*
- Adversarial audit conducted 2026-05-13 by PM (deep-wolf-155) against `docs/r3-close-interrogation.md` § promise enumeration
- Counterfactual surface: 10 substantive gaps found between Director's viz-as-SoT closure claim and THESIS-promise delivery on main
- This doc replaces "viz-as-SoT closed_at + DECLARED-strings-are-drift" framing with explicit per-gap disposition: PROVEN / WEAK-EVIDENCE / GAP-with-plan / R4-DEFERRED-with-operator-accepted

**Closure target** (codex BLOCKING #11284 PR #3013 2026-05-13 enforcement — semantic dilution retracted: prior generic R4-defer / THESIS-reframe clauses violated top-down 0-floor / no-carves authority per `docs/design-pure-bootstrap-zero.md` + `docs/r3-program-plan.md`; operator §4 ratification 2026-05-13 foreclosed all carve paths for the 4 scope items): every adversarial counterfactual cashed by a landed PR with on-main evidence. R4-defer / THESIS-reframe paths are **STRUCTURALLY FORECLOSED** for the 4 scope items (Gaps 1/2/3/9) per operator §4 IN-R3 ratification 2026-05-13 (see §4 + §6). The legacy alternative-disposition sections per gap below are preserved as **retraction citations** (audit-trail) documenting the foreclosed paths, NOT as available closure dispositions.

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

**Alternative disposition — FORECLOSED by operator §4 ratification 2026-05-13** (briansrls + codex BLOCKING PR #3013 2026-05-13 enforcement preserved as authority-chain framing — codex BLOCKING #11284 PR #3013 class enforcement adds operator-ratification foreclosure): operator §4 Item 1 ratified **IN-R3 (full 177-entry retirement)** 2026-05-13; R4-defer path NOT available. The dual-amendment authority chain below stands as **closure-rule discipline** (any future re-opening requires the dual amendment + explicit operator override of the §4 ratification at gunbc#828), NOT as a currently-available carve path:

1. **Override of `project_no_r4_carves_directive`** (operator 2026-05-08: *"we are NOT moving anything to R4 as of now"*) — naming the specific subset + structural-unblockable reason. **AND superseded by operator §4 Item 1 IN-R3 ratification 2026-05-13** — both layers stand; the §4 ratification is the operative foreclosure.
2. **Amendment to `docs/design-pure-bootstrap-zero.md` authority text itself** — adding a per-subset deferral carrier with named reason + retirement plan to the design-doc-tier authority, not just to this plan doc.

Neither override alone is sufficient. The PB-0 design doc admits no escape hatch; absent the amendment, the 0-floor stands and no carve-counter exists in the authority chain.

**Post-§4 read** (preserves PM-recommended history): the load-bearing thesis claim ("0 hand-Rust") is now operator-cashed. The IN-R3 ratification 2026-05-13 brings the design-doc 0-floor + the standing directive + the closure plan into single-authority alignment.

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

**Alternative disposition — FORECLOSED by operator §4 ratification 2026-05-13** (codex BLOCKING #11284 PR #3013 enforcement: prior framing semantically weakened the §3.1 3-Shape-A target into Rust-only-narrow, violating the no-carves authority before operator-decision substrate cashed; retained here as audit-trail of the foreclosed path): operator §4 Item 2 ratified **IN-R3 (full 3-target Python+Go)** 2026-05-13; R4-defer / Rust-only-narrow paths NOT available. Any future re-opening of this disposition requires explicit operator override of the §4 ratification at gunbc#828.

---

### Gap 3 — Self-host fixed point (gate #16)

**Promise** (r3-structure.md §"Acceptance" + §4.2 of interrogation doc): *"bit-identical stage0 + emitted artifact" — the compiler can compile itself and emit a bit-for-bit identical compiler.*

**HEAD evidence**:
- Gate #16 = **CONSUMER_LANDED (R1 horizon; R3 stronger interpretation pending)**
- 4 joint preconditions deferred per parent brief:
  - **R2-Evaluator** — **closed-with-residuals 2026-04-29 16:34Z per ROADMAP.md:512, NOT LANDED**. Per Director audit msg_82b9c4bb 2026-05-13: 5 sub-lanes at R2-close per docs/r2-closure-ledger.md:250-263 — `runtime_value_model_structural` in-flight (#1197 #1228 #1231); `body_evaluator_structural` not-started; `lens_application_complete_reflection` in-flight (#1191); `witness_construction_structural` not-started; `cross_target_equivalence_harness_structural` not-started. Sub-lanes carried into R3 as r3-continuation but closure-ledger row stale @ #1191-#1231 era (HEAD is #3013+). R3-tier slice landings (sampled merged PRs 2026-04 → 2026-05-13: #1813 E6-G0d / #1857 E6-G1.a / #2152 Phase 4 / #2190 E2 Descent / #2257 TC3 D1 / #2658/#2812 gate #5 lens_apply retire / #2681 std.computation lowering / #2825/#2827/#2826/#2941 gates #57-#59) PARTIAL coverage; do NOT discharge the 5 sub-lane ledger. **R3 Evaluator Mgr merry-gull-128 (#1743) ABSENT from current subtree at HEAD** — authority dispersed across 3 R3 Mgrs without single owner (`feedback_thesis_gate_state_drift` instance; r2-structure.md:73 anti-pattern reincarnation under R3-tier-slice procedural wrapper).
  - R2-Grounding-Rust+Python landed
  - T-LP/SG-0 landed (= Gap 1 above)
  - Row-B materialization landed

**What's missing**: the 4 joint preconditions all need to land before R3-strong-form self-host can be claimed. **R2-Evaluator joint precondition itself decomposes into 5 sub-lane closures** per the Director audit; the closure-ledger needs refresh + sub-lane ratchet-to-PASSING via §1.8 gate ratifications.

**Plan to cash**:
- **Owner**: Director-tier coordination (cross-Mgr); PM tracks. **R2-Evaluator joint precondition specifically requires a 5th R3 Mgr lane decision** (see §4 sub-item 5 below) — re-spawn evaluator Mgr / fold into existing R3 Mgrs / Director-direct ad-hoc dispatch.
- **Sub-program**: each precondition has its own program; this gap is meta-blocked
  - R2-Evaluator: 5 sub-lanes per Director audit (runtime_value_model + body_evaluator + lens_application_complete_reflection + witness_construction + cross_target_equivalence_harness); brief surface comprehensive per Director (c) (r2-evaluator-manager.md + 4 sub-briefs + 10+ PR-A-E + R3-tier per-slice briefs). Q-EVAL ratifications landed 2026-05-06/07 (G0d-Dispatch / Descent-Termination-Contract / Lens-Fold-First-Slice). **Owner pending §4 sub-item 5 operator decision**.
  - R2-Grounding-Rust+Python: gate #18 + extdeps work
  - T-LP/SG-0: same as Gap 1
  - Row-B materialization: T-LBP work
- **Effort estimate**: 2-4 months (joint precondition; bounded by the longest of 4). R2-Evaluator residual closure is the dominant tail — 2 not-started sub-lanes + 3 in-flight per audit.

**Close criterion** (substrate-debt-shaped only — Director note msg_f0a54769 2026-05-13 Note 2 enforcement: staffing/dispatch shape is a PRECONDITION for execution, not a close criterion for the substrate-debt itself; moved to "Dispatch staffing prereq" below):
```bash
# Predicate at gate #16 R3-strong-form close:
self_host_fixed_point && diff -q <stage0_binary> <emitted_binary>
# returns: byte-identical
```
All 4 precondition gates PASSING + actual self-host invocation producing bit-identical output. **R2-Evaluator joint precondition close** (Director note msg_f0a54769 Note 1 enforcement: predicate authority is the R2-closure-ledger, NOT §1.8 — sub-lane names live at `docs/r2-closure-ledger.md:250-263`, do NOT map 1:1 to §1.8 row IDs; PM-recommended path (α): use sub-lane names as predicate authority rather than introduce 5 new §1.8 rows for already-named ledger content per `feedback_parallel_representation_debt`): all 5 R2-closure-ledger Evaluator sub-lanes status=green at HEAD per cell-level check of `docs/r2-closure-ledger.md:250-263` —
- `runtime_value_model_structural` = green
- `body_evaluator_structural` = green
- `lens_application_complete_reflection` = green
- `witness_construction_structural` = green
- `cross_target_equivalence_harness_structural` = green

The closure-ledger row currently stale @ #1191-#1231 era; close also requires the ledger be refreshed against HEAD before status-evaluation. Grep-verifiable predicate against `docs/r2-closure-ledger.md` cell content.

**Dispatch staffing prereq** (NOT a close criterion; Director note msg_f0a54769 Note 2 enforcement; Director note msg_f0a54769 Note 3 sequencing enforcement): execution of the 5 sub-lane closures requires owner identification. R3 Evaluator Mgr lane disposition per §4 sub-item 5 ratification (operator decides; PM-recommends re-spawn as 4th lane). **Sequencing**: re-spawn (or fold / Director-direct per operator ratification) occurs AFTER operator §4 sub-item 5 confirmation, NOT before (don't author the Mgr until the operator-decision substrate cashes; `feedback_construction_over_ratchets` adjacent class). PM-recommendation Option A is on-record but execution waits on operator.

**Alternative disposition — FORECLOSED by operator §4 ratification 2026-05-13** (codex BLOCKING #11284 PR #3013 class enforcement — same semantic-dilution pattern as Gap 2; retained as audit-trail of foreclosed path): operator §4 Item 3 ratified **IN-R3 (4-joint-precondition cascade)** 2026-05-13; R4-defer / R1-horizon-narrow paths NOT available. **R2-Evaluator-tier per-sub-lane R4-carve** also FORECLOSED — operator §4 Item 3 IN-R3 ratification cashes the joint precondition rule, including the 5 R2-Evaluator sub-lane closures (substrate-debt-shaped close criterion per Director note msg_f0a54769 Note 2). Any future re-opening requires explicit operator override of the §4 ratification at gunbc#828.

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

**Close criterion** (briansrls + codex BLOCKING PR #3013 2026-05-13 dual enforcement: **negative-form list-emptied predicate + positive-form generator-manifest predicate, both required**; textual `// AUTO-GENERATED FROM .dag` marker RETRACTED per `feedback_no_textual_enforcement_bridges`):

**Negative authority** — hand-authored ratchet empty:
```rust
// in src/v3/compiler/tests/integration/sg0_census_test.rs at HEAD when gate #84 closes:
const EXPECTED_HAND_AUTHORED_TEST: &[&str] = &[];
```
plus `cargo test --release sg0_census_test` passes with predicate = 0.

**Positive authority** — generator-manifest covers every surviving test (codex BLOCKING strengthening: *"make the predicate consume a generator manifest or generated-output comparison, not just a header grep"*):
```
# Predicate at gate #84 close — every checked-in test file traces back to a .dag source via manifest:
for each *_test.rs in src/v3/compiler/tests:
  manifest_entry = lookup_in_test_generator_manifest(file_path)
  assert manifest_entry.exists()
  assert manifest_entry.dag_source_path.exists()
  assert generate_from(manifest_entry.dag_source_path) == file_contents
```

The generator-manifest (substrate to be authored as a Cluster M Phase 3 deliverable) maps each surviving Rust test file → its `.dag` `TestClaim` source. The predicate fails closed on any orphaned test file (no manifest entry) OR any test file whose bytes diverge from regeneration (drift). This is the structural form of "every Rust test ports to .dag or is generated."

plus #85 SuiteClaim wrapper consumer landed.

**Why dual authority is required** (briansrls + codex BLOCKING PR #3013 receipt):
- `feedback_no_textual_enforcement_bridges`: textual marker gating defeats itself; a `// AUTO-GENERATED FROM .dag` comment can be hand-authored
- **Negative authority alone** (list-emptied) is necessary but not sufficient: it proves no entry on the hand-authored ratchet, but doesn't structurally tie each surviving file to its .dag source. A generated file that loses its .dag source (orphaned) would pass the negative predicate while violating THESIS
- **Positive authority** (generator manifest) ties each surviving Rust test file structurally to its .dag source; orphan detection + regeneration-byte-equality fail-close on drift
- Together the two predicates cash the THESIS claim *"every Rust test ports to .dag or is generated"* structurally — no textual proxy
- Substrate prereq: generator-manifest substrate carrier must be designed + landed as part of Cluster M Phase 3; brief-authoring scope expands beyond the current cluster-M-sequencing-plan to include manifest substrate

**Authority for no-boundary-carve-out** (separate codex BLOCKING PR #3013 receipt; preserved):
- `TESTING.md` "🔄 RETRACTED 2026-04-25 (cascade promotion of `docs/design-pure-bootstrap-zero.md`)": *"the previous 'two residual categories' framing (compiler-internal unit tests + external-toolchain boundary tests stay Rust-authored permanently) is retracted under the 0-floor target. Both categories dissolve"*
- `TESTING.md` "0-residual is the target. Until v3's source tree reaches 0 hand-authored files..."
- `docs/design-pure-bootstrap-zero.md`:41 *"Goal: zero hand-authored files in v3's source tree."*
- `docs/design-pure-bootstrap-zero.md`:138 boundary-tests migrate to `ExecuteCommand`-based `.dag` `TestClaim` declarations per cascade-named successor pattern (PR #678 schema landed; bulk migration is Gap 5 Phase 3 scope) — boundary entries are named on `EXPECTED_HAND_AUTHORED_TEST` and dissolve through migration like any other entry, no separate boundary carve-out

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
  4. **Substrate-shape canvas authored by Substrate Mgr BEFORE worker dispatch** (per claude review exploratory observation #2 — this plan doc surfaces a sub-program step, but the actual substrate-shape commitment must be ratified via Mgr canvas, not implemented from this prose). **Canvas substrate-shape constraint (briansrls + codex BLOCKING PR #3013 2026-05-13 dual enforcement: `Option<Witness>` shape RETRACTED; absolute thesis shape and pragmatic residual policy SEPARATED into named carrier variants)**: per `feedback_state_space_vs_behavioral_invariants` ("check if the type admits illegal state combinations; type enforcement > API enforcement") + `feedback_optional_models_recovery_as_exception` ("T? where absence is the norm conceals plurality") + `feedback_practice_2_vs_4_same_variant_vs_cross_variant`, the Correction field MUST inhabit exactly one of two named carrier variants (NOT `Option<Witness>`):

     ```dag
     sum Correction {
       LiveCorrection { witness: Witness }
       | DeferredCorrection { reason: String, retirement_plan: RetirementPlan }
     }
     type Diagnostic {
       ...
       correction: Correction  // mandatory; structurally exhaustive over LiveCorrection | DeferredCorrection
     }
     ```

     **LiveCorrection** is the 100% THESIS-correct path — diagnostic points to the structurally correct program via Witness. **DeferredCorrection** models any accepted residual as an explicit named-deferral carrier with retirement plan (codex BLOCKING #3 enforcement: *"model any accepted residual as an explicit deferral carrier with named reason"*). The Option-wrapped form admits `None` = "diagnostic without correction" — exactly the state THESIS.md "show the correct code" forbids — AND collapses the absolute-thesis vs pragmatic-residual axes into a single nullable boolean. Practice-2 carrier refinement: type-level enforcement that every Diagnostic value carries a Correction; the residual path is structurally named with retirement-plan accountability rather than absorbed silently into None. **Gate #84/#106 close condition** requires every `DeferredCorrection` entry to be ratchetable to zero per its own retirement plan. Canvas authors the substrate shape that admits Diagnostic-with-correction *by construction* via the sum variant; Director ratifies; worker dispatches against ratified shape.
  5. Worker brief dispatch to retrofit existing diagnostics
- **Effort estimate**: 4-8 weeks (depends on diagnostic class count; potentially smaller if substrate already supports it). **Caveat (per claude review exploratory observation #3 — estimates unsourced)**: this estimate is PM-prior-cycle-experience-based, NOT cited against specific velocity data. Substrate Mgr canvas surfaces the actual scope + worker effort; final estimate calibrates post-canvas-ratification.

**Close criterion** (codex BLOCKING #11273 PR #3013 2026-05-13 enforcement: must match the sum-variant `Correction` carrier ratified above — `Option<Witness>` / `Some(_)` shape RETRACTED as it reintroduces the nullable form the substrate-shape canvas explicitly rejects; codex BLOCKING #11254 PR #3013 2026-05-13 prior enforcement: **absolute 100% — pragmatic-relaxation alternative RETRACTED** as it converted a load-bearing THESIS promise into a negotiable threshold):
- **THESIS-correct (100% absolute, variant-tally form)**:
  - **Structural (compiler-enforced)**: every `Diagnostic` value in `src/v3/compiler/` carries a mandatory `correction: Correction` field (sum-variant: `LiveCorrection` | `DeferredCorrection`) — type system enforces presence; no `Option`-wrapping; no `None` representable.
  - **Variant-tally (zero-DeferredCorrection)**: across the full test corpus, **every** fired `Diagnostic` value carries the `LiveCorrection` variant. Equivalently: count of fired `Diagnostic` values with `correction: DeferredCorrection { .. }` variant is exactly 0. Per the sum-variant carrier shape above, this exhausts the sum and discharges the THESIS promise. No threshold relaxation. THESIS.md "Error handling: show the correct code" — *"Diagnostics should point to the structurally correct program, not just report that the current one is wrong"* — reads as absolute promise.
  - **Substrate ratchet (deferred-correction retirement)**: per the canvas substrate-shape commitment above, every `DeferredCorrection` entry encountered during the close-walk is ratchetable to zero per its own `retirement_plan` field; close fires when the DeferredCorrection list is empty (no fired diagnostic carries that variant).

**Authority for no-relaxation** (codex BLOCKING PR #3013 receipt): the prior "Pragmatic relaxation (≥X%)" framing dilutes an absolute THESIS commitment into a negotiable threshold without THESIS-text reconciliation. Per `project_no_r4_carves_directive` (2026-05-08), R4-carve framing is NOT freely available; within-R3 relaxation paths are structurally equivalent to R4-carves under the directive. The alternative-disposition path below (R4-scope reframe with operator-recorded acceptance) is the ONLY non-100% path, and it requires explicit operator override of the no-carves directive — not a within-R3 threshold negotiation.

**Alternative disposition — FORECLOSED by operator §4 ratification 2026-05-13** (codex BLOCKING #11284 PR #3013 class enforcement — same semantic-dilution pattern as Gap 2; retained as audit-trail of foreclosed path): operator §4 Item 4 ratified **IN-R3 at 100% absolute** 2026-05-13; THESIS-aspirational-not-R3-promised reframe NOT available. The carrier shape (sum-variant `Correction { LiveCorrection | DeferredCorrection }` with zero `DeferredCorrection` in test corpus per close criterion above) is the structural form the ratification cashes. Any future re-opening requires explicit operator override of the §4 ratification at gunbc#828.

---

### Gap 10 — Close-audit doc absent (interrogation §8)

**Promise** (r3-close-interrogation.md §8, count generalized by PR #3013 close-criterion discipline): all §1.8 predicates are executed at HEAD within 24h of close ceremony, with the row count derived from the live ledger. Execution log preserved as audit artifact.

**HEAD evidence**:
- No `docs/audit/r3-close-predicate-execution-2026-05-13.md` exists
- The close-ceremony self-check doc has not been authored

**What's missing**: the close-ceremony doc.

**Plan to cash**:
- **Owner**: PM (deep-wolf-155) authors the doc; Verification Mgr (swift-deer-459) executes predicates
- **Sub-program**:
  1. PM authors skeleton: one row per live §1.8 gate, with column for predicate execution status
  2. Verification Mgr runs each predicate at HEAD; records pass/fail + output snippet
  3. PM closes out the doc with verdict per row + overall R3-close verdict
- **Effort estimate** (Director feedback item 7 — calibrated):
  - **Skeleton authoring (PM-direct)**: 1-2 days — ledger-derived table scaffold + per-row column shape + initial categorization
  - **Predicate execution (Verification Mgr serial)**: 1-2 weeks — one `cargo test` / `grep` / predicate invocation per live §1.8 row + result capture + per-row writeup
  - **Predicate execution (Verification Mgr parallel via ctrl-build)**: 3-5 days compressed if remote BuildBuddy infra healthy + parallelism dispatch
  - **Overall**: 1-2 weeks for full landing, contingent on Verification Mgr capacity + ctrl-build availability

**Close criterion** (codex non-blocking PR #3013 2026-05-13: **count-drift cleanup — derive row-count from §1.8 ledger at execution time**, NOT hard-coded; per `feedback_no_snapshot_integers_in_briefs`): `docs/audit/r3-close-predicate-execution-YYYY-MM-DD.md` exists on main with **all §1.8 ledger rows filled** (count derived at execution time via `grep -cE "^\| #[0-9]+ \|" docs/r3-program-plan.md` or equivalent §1.8 row enumeration; Gap 9 row #106 + any subsequent additions automatically included) + overall verdict cited. Avoids the snapshot-integer rot pattern.

**Note**: this gap is the *receipt* for the closure ceremony. Skeleton authoring (1-2 days) is not blocked on other gaps and is PM's immediate next deliverable; predicate execution (1-2 weeks) can run in parallel with other Phase B/C/D work.

---

### Gap 11 — Complexity composition completeness / `LogCost` asymmetry (gate #79 sub-promise)

**Promise** (THESIS.md cost lens + design-cost-lens / `src/v3/lenses/complexity.dag` header: *"behavioral complexity summary lens for the v3 DAG ... structurally terminal; behaviorally complete"*): the complexity lens behaviorally computes asymptotic class for any program structure the substrate admits, including nested algorithms (composition of poly/log/linear/constant in any permutation expressible by `SymbolicCost`).

**HEAD evidence** (operator adversarial probe 2026-05-13 — `n log(n^k)` + nested-log + polynomial-of-log probe; revised per briansrls BLOCKING comment-4445313478 PR #3037 2026-05-13: HEAD evidence verified against `src/v3/std/algebra.dag` + `src/v3/compiler/src/dag_cost_generated.rs` + `src/v3/compiler/src/complexity_lattice.rs` + `src/v3/compiler/src/enforced_lens_application.rs`):

- **`SymbolicCost` substrate** (`src/v3/std/algebra.dag` lines 191-197, 7 variant arms; `inhabits Semiring<SymbolicCost>` declaration at line 190):
  - `ProductCost(NonSingletonList<SymbolicCost>)` — recursive composition over arbitrary `SymbolicCost` ✓
  - `SumCost(NonSingletonList<SymbolicCost>)` — recursive composition over arbitrary `SymbolicCost` ✓
  - `PolynomialCost { var: SizeVariable, degree: DegreeAtLeastTwo }` — terminal: `SizeVariable` argument only; degree carrier is `DegreeAtLeastTwo` (refinement type, NOT `Nat`) — substrate-level guarantee that polynomial degree ≥ 2 (degree 1 is redundant with `LinearCost`; degree 0 with `ConstantCost`); load-bearing for the `ClassPolynomial { degree: N }` classifier arm in `dag_cost_generated.rs:297-306` and string-arm decoding in `enforced_lens_application.rs`
  - `LogCost(SizeVariable)` — terminal: `SizeVariable` argument only
  - `LinearCost(SizeVariable)` / `ConstantCost(Int)` / `UnknownCost(String)` — terminal
  - **No `ExponentialCost` variant in SymbolicCost** — exponential growth in source cost has no substrate representation.

- **`AsymptoticClass` enum** (`src/v3/compiler/src/dag_cost_generated.rs:86-95`, 8 variants): `ClassConstant`, `ClassLog`, `ClassLinear`, `ClassLinearithmic`, `ClassQuadratic`, `ClassPolynomial { degree }`, `ClassExponential`, `ClassUnknown`.

- **`classify_symbolic_cost` actual behavior** (`src/v3/compiler/src/dag_cost_generated.rs:289-312`):
  ```rust
  ConstantCost  → ClassConstant
  LinearCost    → ClassLinear
  LogCost       → ClassLog
  PolynomialCost { degree=2 } → ClassQuadratic
  PolynomialCost { degree=N>0 } → ClassPolynomial { degree=N }
  ProductCost / SumCost / UnknownCost → ClassUnknown      // <-- composition handling absent
  ```
  Any composite `SymbolicCost` (`ProductCost`, `SumCost`) → **`ClassUnknown`**. The classifier has **zero composition handling** — only terminal-variant arms.

- **`ClassLinearithmic` + `ClassExponential` are unreachable outputs from the classifier**. They are constructible only via the string-to-AsymptoticClass deserialization in `src/v3/compiler/src/enforced_lens_application.rs:960-962` (used when *users* declare an enforcement budget like `"ClassLinearithmic"`). Nothing in `classify_symbolic_cost` or `normalize` ever produces them. The lattice has 8 declared classes, but the classifier produces only 6.

- **`normalize(c: SymbolicCost)` body** (`src/v3/std/algebra.dag:537-548`) handles only:
  - Sum: drop `ConstantCost(0)` (additive identity)
  - Product: collapse to `ConstantCost(0)` on multiplicative annihilator, drop `ConstantCost(1)` (multiplicative identity)
  - Single-element Sum/Product: collapse to the element
  - Product of two identical `LinearCost`s → `PolynomialCost(var, 2)` (n*n = n²)
  - **NO log-power rule** (`log(n^k) → k * log(n)`), **NO log-product rule**, **NO nested-log handling**, **NO Product/Sum-to-named-tier normalization** (`Product([Linear, Log])` does NOT become `ClassLinearithmic` in classification — it stays `ProductCost` and classifies to `ClassUnknown`).

- **What the compiler actually reports for nested algorithms at HEAD** (revised — was overstated previously):
  - `n log n` = `ProductCost([LinearCost(n), LogCost(n)])` → **`ClassUnknown`** (classifier has no `n log n` arm; `ClassLinearithmic` is unreachable).
  - `n log(n^k)` — cannot be constructed directly (type-level: `LogCost` doesn't take `PolynomialCost`). Cost-lens fold canonicalization at construction time is UNVERIFIED at HEAD (lens-body audit deferred to Gap 11 sub-program step 1). If unconstructible: lens emits `UnknownCost(<reason>)` → classifier `ClassUnknown`.
  - `log(log(n))` — same: cannot be constructed (`LogCost` doesn't take `LogCost`).
  - `n² log n` = `ProductCost([PolynomialCost(n, 2), LogCost(n)])` → **`ClassUnknown`** (composition → Unknown; the polynomial degree and the log factor are both lost).
  - `n log²n` = `ProductCost([LinearCost(n), LogCost(n), LogCost(n)])` → **`ClassUnknown`** (composition → Unknown).
  - `2^n` — no `ExponentialCost` substrate variant; cannot be represented in source. Would fall to `UnknownCost(<reason>)`.

- **`STOP SIGNAL` discipline** (`src/v3/std/algebra.dag` Pattern-3 audit block): the 7 SymbolicCost variants are declared "terminal; the asymptotic surface the thesis reasons about; any new variant should carry its own load-bearing reason." This forbids an 8th variant. It does NOT address (a) making existing variants (`LogCost`, `PolynomialCost`) recursive over `SymbolicCost` to be symmetric with `Product`/`Sum`, (b) the missing `ExponentialCost` variant, or (c) the classifier's lack of composition handling.

**What's missing** (recalibrated per actual HEAD evidence above — classifier composition-handling is the root issue, not lattice tier coverage):

1. **`classify_symbolic_cost` composition handling**: the classifier currently maps ALL `ProductCost` and `SumCost` to `ClassUnknown`. This is the dominant gap — even `n log n` (the most common nested case) classifies to Unknown despite `ClassLinearithmic` being an enumerated tier. Composition-aware classification arms are needed: at minimum (a) `Product([Linear, Log])` → `ClassLinearithmic`, (b) `Product([Polynomial(k), Log])` → tier extension or named collapse, (c) `Sum` dominance reduction (drop dominated terms, classify the survivor).
2. **`LogCost` recursive shape (OR canonicalization rule)**: substrate cannot express `Log(complex)` directly (terminal `SizeVariable`-only argument). Either ratify `LogCost(SymbolicCost)` symmetric with Product/Sum, OR ratify a `.dag`-authored canonicalization rule that converts `Log(complex)` → `Log(SizeVariable)` of the dominant variable + multiplicative factors hoisted out (e.g. `log(n^k) → k * log(n)`) at expression-construction time.
3. **`ExponentialCost` variant absence**: no substrate way to represent exponential growth in source cost. Either add the 8th variant (against STOP SIGNAL discipline — needs ratified motivation) or formally scope `ClassExponential` as enforcement-budget-only (input-not-output) with explicit rationale.
4. **`ClassLinearithmic` / `ClassExponential` reachability gap**: 2 of 8 lattice classes are unreachable from `classify_symbolic_cost`; only constructible via string-deserialization for enforcement budgets. Either wire the classifier to produce them (per item 1) OR formally annotate them as input-only tiers with explicit reachability documentation.
5. **`normalize()` log-rule extensions**: add log-power (`log(n^k) → k * log(n)`), log-product (`log(a*b) → log(a) + log(b)`), nested-log handling — OR mark these as out-of-scope with explicit rationale.
6. **Cost-lens fold audit**: verify whether the cost-lens fold at `src/v3/lenses/complexity.dag` does construction-time canonicalization (would partially close the gap without substrate changes, but only if classifier composition handling also lands per item 1).

**Plan to cash**:
- **Owner**: Verification Mgr (still-moth-538) audits cost-lens fold + classifier behavior; Substrate Mgr (warm-wolf-698) authors the substrate-shape canvas decisions (classifier composition arms + LogCost recursive vs canonicalization-rule + ExponentialCost variant decision); Director ratifies the substrate-shape decisions.
- **Sub-program**:
  1. **Audit at HEAD** (Verification Mgr): grep cost-lens fold body for nested-construction handling; report actual behavior on `log(complex)` inputs; confirm `classify_symbolic_cost` Product/Sum → ClassUnknown behavior (per HEAD evidence above) is the dominant classification gap
  2. **Classifier composition canvas** (Substrate Mgr): add composition arms to `classify_symbolic_cost` — minimum (a) `Product([Linear, Log])` → `ClassLinearithmic` (closes the reachability gap for the most common nested case), (b) `Product([Polynomial(k), Log])` → tier-extension OR named collapse decision, (c) `Sum` dominance reduction (drop dominated terms, classify the survivor), (d) `ExponentialCost` substrate variant decision (add 8th variant OR formally annotate `ClassExponential` as enforcement-budget-only)
  3. **LogCost shape canvas** (Substrate Mgr): two options — (A) ratify `LogCost(SymbolicCost)` recursive symmetric with Product/Sum, OR (B) ratify `.dag`-authored canonicalization rule that runs before LogCost construction with named log-algebra coverage (power-rule + product-rule + nested-log policy)
  4. **`normalize()` extension** OR canonicalization-rule landing per item 3 (A)/(B), plus log-power / log-product / nested-log rule coverage
  5. **`AsymptoticClass` reachability review**: confirm classifier produces all 8 tiers or formally annotate the unreachable subset; document collapse rules with explicit rationale per `feedback_no_textual_enforcement_bridges`
  6. **Lens cementing receipt update**: extend behavioral completion test corpus to cover nested compositions (n log n, n log(n^k), n² log n, n log² n, log log n, 2^n) with the expected classification per the canvas decisions
- **Effort estimate**: 3-5 weeks (revised up from 2-4 per recalibration — classifier composition arms + ExponentialCost decision were not in the prior scope) — substrate canvas + lens-fold audit + classifier extension + normalize extension + lattice reachability review; gated on canvas ratification authority.

**Close criterion**:
- (a) `classify_symbolic_cost` produces all reachable `AsymptoticClass` tiers from `ProductCost`/`SumCost` compositions per the canvas decisions (at minimum: `n log n` classifies to `ClassLinearithmic` not `ClassUnknown`)
- (b) `LogCost` shape ratified (either recursive over `SymbolicCost` or canonicalized at construction with named rule coverage)
- (c) `ExponentialCost` substrate variant ratified-or-formally-excluded with named rationale
- (d) `normalize()` extended with log-rules OR canonicalization-rule layer landed
- (e) `AsymptoticClass` reachability review complete: every enumerated class is either produced by `classify_symbolic_cost` from valid `SymbolicCost` inputs OR formally annotated as enforcement-budget-input-only
- (f) Cementing test corpus extended to cover nested compositions per the operator probe (n log n, n log(n^k), n² log n, n log² n, log log n, 2^n) with explicit expected classification
- (g) Complexity lens behavioral-completion test passes against the extended corpus

**Alternative disposition — FORECLOSED at authoring** (per `project_no_r4_carves_directive`): R4-defer of nested-complexity completeness would scope-narrow gate #79 behavioral close to "only handles non-nested compositions". The operator-adversarial probe 2026-05-13 surfaced this gap; the gap is now operator-recorded and must close IN-R3 absent explicit operator override of the no-carves directive with named structural-unblockable reason. Substrate-shape canvas is the unblockable path.

**Authority**: operator adversarial probe 2026-05-13 (this PR); gate #79 close criterion in close plan §1 Gap 4 "complexity behavioral parity"; substrate-shape decision routes through Director ratification (msg- routing TBD post-canvas).

---

### Gap 12 — Property-based complexity-lens validation via `ProgramGenerator` (gate #79 / #85 / #86 join — coverage sub-promise)

**Promise** (Gap 11 + gate #85 `forall_exists_quantifier_substrate_landed` + gate #86 `program_generator_carrier_landed` + thesis-facet-3: *"complexity lens behaviorally complete"* extended via the operator adversarial probe 2026-05-13: *"do we have testcases representing random combinations of functions, validating that the correct complexity result is generated?"*): the complexity lens is validated against **random program compositions** generated via the `ProgramGenerator` substrate, with per-composition expected complexity verified against an oracle. Behavioral completion is not just "passes 2 hand-authored cementing cases" but "passes N>>1 random compositions per run with zero divergence from oracle".

**HEAD evidence** (operator probe 2026-05-13):

- **`ProgramGenerator` substrate carrier LANDED** (gate #86; `src/v3/std/verification.dag`: `type ProgramGenerator { ... }`). **But** only used in **one** test — `m1_5_verification_test.rs::program_generator_authoring_surface_compiles_cleanly` — which verifies the carrier's surface compiles. **NOT** used to actually generate random programs for any lens.
- **`ForAll` / `Exists` quantifiers** authored in `verification.dag` (gate #85 surface) but the `ForAll` variant is wired only for `ForAllTargets` (cross-target quantification per Gap 2 close criterion), NOT for `ForAll(random_program)` quantification per the property-based test class the operator probe targets.
- **Complexity cementing test** (`src/v3/compiler/tests/integration/cementing/complexity_lens_behavioral_completion.rs`): **2 hand-authored test cases** at HEAD (`literal_bind_cements_constant_complexity_summary` + `recursive_countdown_cements_linear_work_and_span`). Zero random compositions; zero oracle comparison; zero quantifier-coverage.
- **No `proptest` / `quickcheck` / random-program-generation tests** for the complexity lens anywhere in `src/v3/compiler/tests/`.
- **Result**: gate #79 `lens_capability_register_zero_proxy_zero_stub` lens-completion for complexity can claim "behaviorally complete" while never having validated against arbitrary nested compositions (the substrate `SymbolicCost` algebra reach is **enormous** vs the 2 cementing cases covered).

**What's missing** (substrate-driven validation pipeline + oracle + test infrastructure):

1. **`ProgramGenerator` instance for complexity-test compositions**: a `.dag`-authored generator that produces random programs by combining function primitives (constant / linear-loop / log-search / poly-iteration / product-nested / sum-branched / log-of-... per Gap 11 substrate decision). Generator must be **structurally bounded** (no infinite recursion) and **complexity-oracle-paired** (each generated program has a known-correct asymptotic class).
2. **Complexity oracle** (reference implementation): for the structurally-bounded composition class the generator produces, the oracle computes the expected `ComplexitySummary` (work + span) deterministically. Authored as a `.dag` function over the program-generator output, NOT a bridge-Rust oracle (per `feedback_no_textual_enforcement_bridges`).
3. **Property-based `TestClaim` using `ForAll`**: a TestClaim that quantifies over the ProgramGenerator output, asserting `complexity_of(generated_program) == oracle(generated_program)` for all generated samples. Uses the existing `ForAll` quantifier surface from gate #85 (extended from `ForAllTargets` to `ForAll<ProgramGenerator>`).
4. **Coverage discipline**: minimum N≥100 random compositions per CI run (or budgeted-N per CI-budget); explicit seed-pinning for reproducibility; fail-closed on any oracle/lens divergence; CI ratchet that monotonically increases sample size as confidence grows.
5. **Cementing-cases preserved** (subset of property-based corpus): the 2 existing hand-authored cases (`literal_bind`, `recursive_countdown`) remain as low-N seeds; the property-based corpus extends rather than replaces them.

**Plan to cash**:

- **Owner**: Verification Mgr (still-moth-538) — owns the property-based testing infrastructure as a Verification lane extension (Gap 5 sibling); Substrate Mgr (warm-wolf-698) — co-owns the `ProgramGenerator`-instance authoring + oracle-function authoring; Director ratifies the property-based-TestClaim shape.
- **Sub-program**:
  1. **Author `ProgramGenerator` complexity-instance** (`dsl/std/test_generators/complexity_generator.dag` or similar): structurally-bounded composition generator producing programs of named asymptotic class
  2. **Author complexity oracle** (`dsl/std/test_oracles/complexity_oracle.dag`): function from generated-program → expected `ComplexitySummary`
  3. **Extend `ForAll` quantifier surface** from `ForAllTargets` to `ForAll<ProgramGenerator>` per gate #85 sub-promise; canvas-ratify with Director (or absorb if existing surface already supports the shape)
  4. **Author property-based `TestClaim`** asserting `complexity_of(g) == oracle(g)` for ForAll g in ProgramGenerator output
  5. **CI integration**: property-based test runs in regular CI with budgeted sample-size; fail-closed on divergence; seed-pinned for reproducibility
- **Effort estimate**: 2-3 weeks (ProgramGenerator-instance authoring + oracle + test infrastructure + CI integration); parallelizable with Gap 11 substrate-shape canvas authoring.

**Close criterion**:
- (a) `ProgramGenerator` complexity-instance landed in `dsl/std/` (or appropriate test-generator path) with structural bound + named composition class coverage
- (b) Complexity oracle authored as `.dag` function (no bridge-Rust oracle)
- (c) Property-based `TestClaim` using `ForAll<ProgramGenerator>` lands + passes with N≥100 random compositions per CI run
- (d) Zero oracle-vs-lens divergence across the random-composition corpus
- (e) CI seed-pinning + reproducibility discipline ratcheted via test infrastructure

**Connection to Gap 11**: Gap 11 (LogCost asymmetry) addresses substrate-shape CAPABILITY (what compositions can be expressed); Gap 12 addresses lens-COVERAGE VALIDATION (whether the lens correctly classifies what the substrate expresses). Both are needed for gate #79 complexity behavioral close to honestly mean "lens correctly handles arbitrary nested compositions". Gap 11 substrate canvas should land first so Gap 12 generator can produce the full composition class.

**Alternative disposition — FORECLOSED at authoring** (per `project_no_r4_carves_directive`): R4-defer of property-based validation would scope-narrow gate #79 behavioral close to "passes hand-authored cementing cases only". The operator-adversarial probe 2026-05-13 surfaced this gap; gate #85 `forall_exists_quantifier_substrate_landed` + gate #86 `program_generator_carrier_landed` are R3-load-bearing per the Tests-as-Data-Completeness lane — using them for actual property-based validation (not just substrate-compile checks) is the natural R3 close shape.

**Authority**: operator adversarial probe 2026-05-13 (this PR); gate #79 close criterion in close plan §1 Gap 4 "complexity behavioral parity"; gates #85 + #86 close criteria in close plan §1 Gap 5 "tests-as-data completeness"; substrate-shape decision routes through Director ratification (msg-routing TBD post-canvas).

---

## §2. Dispatch sequencing (PM-recommended)

Given the cross-gap dependencies, recommended dispatch order:

**Phase A — Immediate (PM-direct, 1-2 weeks)**:
- Gap 10: PM authors close-audit doc skeleton; dispatches predicate execution
- Gap 9: PM proposes §1.8 row #106 to Director for ratification

**Phase B — Substrate Mgr lane (warm-wolf-698, 4-8 weeks)**:
- Gap 4 (lens behavioral parity): F-α → F-β.1 → F-β.2 → ComplexitySummary → F-γ.2 register sweep
- Gap 7 (T-WAD): Slice 4 → 5 → 6 → 7 → 8 cascade (zesty-boar-261 + sharp-deer-576 lane)
- **Gap 11 (Complexity composition completeness / LogCost asymmetry)**: substrate-shape canvas (LogCost recursive vs canonicalization-rule) + normalize() extension + lattice tier review + cementing corpus extension. **Sub-promise of gate #79 surfaced by operator adversarial probe 2026-05-13** post-§4-ratification.

**Phase C — Verification Mgr lane (still-moth-538, 4-8 weeks)**:
- Gap 5 (Cluster M Phase 3): 99-Rust-test bulk-port dispatch
- Gap 2 (L5 cross-target): certification corpus + Python/Go emitters
- **Gap 12 (Property-based complexity-lens validation via ProgramGenerator)**: ProgramGenerator complexity-instance + complexity oracle + `ForAll<ProgramGenerator>` quantifier TestClaim + CI integration. **Sub-promise of gate #79/#85/#86 surfaced by operator adversarial probe 2026-05-13** post-§4-ratification. Gated on Gap 11 substrate-shape canvas landing first so generator can produce the full composition class.

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
- Gap 10: source = direct execution count (live §1.8 predicate count × {serial 1-2wk OR parallel 3-5d via ctrl-build})

This caveat applies to the entire §3 — estimates above are PM-best-guess at draft authorship; precision improves as dispatch progresses and weekly closure-cadence messages calibrate against actual landing dates.

---

## §4. Operator decision points — RATIFIED 2026-05-13

**Ratification outcomes** (operator briansrls direct PM dispatch 2026-05-13):
- **Items 1-4 (R3 scope decisions)**: ALL **IN-R3** confirmed. No R4-carves.
  - Gap 1 (PB-0): IN-R3 — full 177-entry retirement
  - Gap 2 (L5 cross-target): IN-R3 — full 3-target Python+Go
  - Gap 3 (self-host R3-strong): IN-R3 — 4-joint-precondition cascade
  - Gap 9 (show-correct-code): IN-R3 — 100% absolute (zero `DeferredCorrection` per sum-variant carrier)
- **Item 5 (subtree-shape decision)**: **(a) re-spawn R3 Evaluator Mgr as 4th R3 Mgr lane** ratified. Director (zesty-bear-812) executes per pre-authorization msg_d456b60d.
- **Phase A immediate dispatch**: authorized.

The presentation framing below preserved as audit-trail of the operator-question that landed the ratification.

---

**Standing directive context** (per `project_no_r4_carves_directive`, Brian 2026-05-08 verbatim: *"we are NOT moving anything to R4 as of now"*): R4-carve is **NOT freely available** as a default. The 4 scope decisions below default to **IN-R3** unless operator explicitly overrides the standing directive with a structural-unblockable-reason argument per-decision.

**PM recommendation across all 4 scope decisions** (per claude review exploratory observation #1 — explicit per-gap PM view): **do not defer** any of Gaps 1/2/3/9. Each R4-carve materially dilutes a load-bearing R3 promise:
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

**5. R3 Evaluator Mgr dispatch (subtree-shape decision; surfaced per Director audit msg_82b9c4bb 2026-05-13)**: structurally distinct from the 4 scope decisions above — this decision changes Director subtree shape rather than R3 surface scope. Per Director audit findings: (a) R2-Evaluator closed-with-residuals with 5 sub-lanes carried into R3; (b) R3 Evaluator Mgr merry-gull-128 (#1743) ABSENT from current subtree at HEAD; (c) brief surface comprehensive; (d) Director-recommends re-spawn as 4th R3 Mgr lane. PM-recommendation **OPTION A: re-spawn evaluator Mgr** per:
   - `feedback_standing_managers_need_owned_deliverables` — 5 named sub-lanes (runtime_value_model + body_evaluator + lens_application_complete_reflection + witness_construction + cross_target_equivalence_harness) is an owned-program count that justifies a Mgr lane.
   - `feedback_pre_authored_brief_queue` — brief surface already comprehensive per Director (c); re-spawn does NOT bottleneck on Mgr-tier dispatch authoring.
   - Operator standing directive *"staffing not a concern"* (2026-05-13 PR #3013 ratification thread) frames 4th R3 Mgr lane as IN-policy.
   - Existing 3-Mgr R3 template (Substrate / Verification / Debt-Paydown) symmetry — 4th lane is structurally parallel, not a new Mgr-tier shape.

   Operator sub-decisions:
   - (a) **Re-spawn R3 Evaluator Mgr as 4th lane** (PM-recommended; Director-recommended primary option). Brief surface ready; sub-lanes named; owned-program count meets bar.
   - (b) **Fold into existing R3 Mgrs**: witness_construction + cross_target_equivalence to Verification (swift-deer-459); runtime_value_model + body_evaluator to Substrate (warm-wolf-698); lens_application_complete_reflection to whichever has lower load. Lower agent-spawn cost; risk: scope-bloat under existing R3-lane load creates dual-program lane (the same anti-pattern that produced the dispersion).
   - (c) **Director-direct ad-hoc dispatch**: structurally equivalent to retracted r2-structure.md:73 anti-pattern ("standing managers without owned deliverables degenerate into pass-through hops; concentrating brief-authoring on Director starved lanes"); PM does NOT recommend.

**§5 process discipline note**: per the standing directive, asking the operator to choose IN-R3-vs-R4-carve framing for the 4 scope items implicitly invites R4-carve consideration. Re-framing per Director feedback item 1: the question is "confirm IN-R3 (default, per directive + PM-recommended)" — explicit override only if structurally unblockable. **Sub-item 5 (Mgr-dispatch) is NOT a scope question**; it asks operator to ratify subtree-shape change. Default state is "no decision recorded" — Gap 3 close-criterion is meta-blocked until operator ratifies one of (a)/(b)/(c).

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
- [x] **Operator §4 confirmations** — ALL 4 IN-R3 ratified 2026-05-13 (briansrls direct PM dispatch):
  - [x] Gap 1 (PB-0): **IN-R3 confirmed** — full 177-entry retirement, no R4-carves
  - [x] Gap 2 (L5 cross-target): **IN-R3 confirmed** — full 3-target Python+Go cross-target stdout-parity
  - [x] Gap 3 (self-host R3-strong): **IN-R3 confirmed** — 4-joint-precondition cascade (no R1-horizon scope-narrow)
  - [x] Gap 9 (show-correct-code): **IN-R3 confirmed** — 100% absolute (zero DeferredCorrection in test corpus per sum-variant carrier)
- [x] **Operator §4 sub-item 5 (subtree-shape decision)** — ratified 2026-05-13 (briansrls direct PM dispatch): **(a) re-spawn R3 Evaluator Mgr as 4th lane** confirmed. Director (zesty-bear-812) executes per pre-authorization at msg_d456b60d.
- [x] **Operator authorizes Phase A immediate dispatch** — implicit in ratification 2026-05-13. Close-audit doc skeleton + §1.8 row #106 authoring proceeds PM-direct post-merge.
- [ ] **Gap 11 (Complexity composition completeness / LogCost asymmetry — post-§4-ratification adversarial finding)** — surfaced by operator adversarial probe 2026-05-13; default IN-R3 per `project_no_r4_carves_directive` as gate #79 sub-promise. Substrate-shape canvas decision required (LogCost recursive vs canonicalization-rule); routes through Substrate Mgr (warm-wolf-698) → Director ratification.
- [ ] **Gap 12 (Property-based complexity-lens validation via ProgramGenerator — post-§4-ratification adversarial finding)** — surfaced by operator adversarial probe 2026-05-13; default IN-R3 per `project_no_r4_carves_directive` as gate #79/#85/#86 join sub-promise. ProgramGenerator complexity-instance + oracle + `ForAll<ProgramGenerator>` TestClaim + CI integration; routes through Verification Mgr (still-moth-538). Gated on Gap 11 canvas landing first.
- [ ] Director-tier deliverables in-flight per msg_cd2d8d7d:
  - [x] R2-Evaluator audit — **completed 2026-05-13 (msg_82b9c4bb)**; findings absorbed into Gap 3 expansion + §4 sub-item 5 + r3-program-plan.md lines 429/435 reframe at commits 85c230b4b + 97cfb9d4c
  - [ ] Gap 3 cross-Mgr coordination tracking (ongoing, Phase E)
  - [ ] §1.8 row #106 substrate-shape canvas ratification when PM surfaces it
  - [ ] Pre-execution review of close-audit-doc rows touching Director-tier ratification gates (gate #105 / canvas-ratified-class)
- [ ] PM dispatches Phase B + Phase C briefs post-§4-ratification (ratification ✓ 2026-05-13; PM-direct close-audit-doc skeleton authoring imminent)

---

**Authored by**: deep-wolf-155 (PM)
**Date**: 2026-05-13
**Status**: Director structure-ratified 2026-05-13 (msg_cd2d8d7d); **operator fully ratified 2026-05-13** (4 scope decisions IN-R3 + §4 sub-item 5 re-spawn (a) + Phase A authorization); READY FOR DISPATCH post-merge
