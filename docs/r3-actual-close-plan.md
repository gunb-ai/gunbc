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
- Gate #15 `l5_cross_target_consistency` = **CONSUMER_LANDED** after PR #3060, not PASSING.
- `src/v3/compiler/tests/fixtures/r3_verification_l5_corpus.dag` defines an L5 `ForAllTargets` scaffold suite with four rows: `add_then_branch_seed`, `branch_literal_true_seed`, `branch_literal_false_seed`, and `nested_branch_seed`.
- `src/v3/compiler/tests/boundary/l5_cross_target_consistency.rs` keeps the embedded `TestClaim.source` bytes equal to the authority `.v3` fixtures under `src/v3/compiler/tests/fixtures/r3_l5_corpus/`.
- The `TestRunner` L5 path consumes `ProgramOutputBind`, validates exact `TestClaim.requires` toolchain edges (`L5RustcToolchain`, `L5Python3Toolchain`, `L5GoToolchain`), emits each corpus program to Rust/Python/Go, executes each target, and fails closed unless the observed `Int` stdout values agree.

**What's missing**: per `docs/design-cross-target-equivalence.md` Corpus Policy, each valid L5 corpus row must carry the expected semantic observation or oracle authority, the effect class, the numeric policy, and a coverage reason. The #3060 rows model target requirements and output binds, but not those locked policy facts; they are scaffold evidence rather than full L5 certification rows.

**Plan to cash**: extend the L5 corpus substrate so every row records the locked design facts above, then update the runner/fixtures to consume them fail-closed before the §1.8 row flips to PASSING. PR #3060 remains the target-execution runner scaffold and can be reused once those corpus-policy facts are modeled.

**Close criterion**:
```bash
# Predicate at gate #15 close:
cargo test --release -p v3-compiler --test integration l5_
# returns: PASS with N>0 certification-corpus programs, all 3 targets agreeing on semantic observations
```
plus §1.8 row #15 status flips CONSUMER_LANDED → PASSING with corpus enumeration and per-row design-policy facts cited.

**Alternative disposition — FORECLOSED by operator §4 ratification 2026-05-13** (codex BLOCKING #11284 PR #3013 enforcement: prior framing semantically weakened the §3.1 3-Shape-A target into Rust-only-narrow, violating the no-carves authority before operator-decision substrate cashed; retained here as audit-trail of the foreclosed path): operator §4 Item 2 ratified **IN-R3 (full 3-target Python+Go)** 2026-05-13; R4-defer / Rust-only-narrow paths NOT available. Any future re-opening of this disposition requires explicit operator override of the §4 ratification at gunbc#828.

---

### Gap 3 — Self-host fixed point (gate #16)

**Promise** (r3-structure.md §"Acceptance" + §4.2 of interrogation doc): *"bit-identical stage0 + emitted artifact" — the compiler can compile itself and emit a bit-for-bit identical compiler.*

**HEAD evidence**:
- Gate #16 = **CONSUMER_LANDED (R1 horizon; R3 stronger interpretation pending)**
- 4 joint preconditions deferred per parent brief:
  - **R2-Evaluator** — **closed-with-residuals 2026-04-29 16:34Z per ROADMAP.md:512, NOT LANDED**. Per Director audit msg_82b9c4bb 2026-05-13: 5 sub-lanes carried into R3. **HEAD refresh 2026-05-14 UTC:** the R3 Evaluator Mgr lane is re-spawned; `docs/r2-closure-ledger.md` now records 3/5 green (`runtime_value_model_structural`, `body_evaluator_structural`, `cross_target_equivalence_harness_structural`) and 2/5 still in-flight (`lens_application_complete_reflection`, `witness_construction_structural`). R3-tier slice landings (sampled merged PRs 2026-04 → 2026-05-13: #1813 E6-G0d / #1857 E6-G1.a / #2152 Phase 4 / #2190 E2 Descent / #2257 TC3 D1 / #2658/#2812 gate #5 lens_apply retire / #2681 std.computation lowering / #2825/#2827/#2826/#2941 gates #57-#59) partially discharge the ledger, but real lens-over-`Dag` body authority, generic fold/runtime-callee execution, and complete witness materialization still block the Evaluator joint precondition.
  - R2-Grounding-Rust+Python landed
  - T-LP/SG-0 landed (= Gap 1 above)
  - Row-B materialization landed

**What's missing**: the 4 joint preconditions all need to land before R3-strong-form self-host can be claimed. **R2-Evaluator joint precondition itself decomposes into 5 sub-lane closures** per the Director audit; the closure-ledger needs refresh + sub-lane ratchet-to-PASSING via §1.8 gate ratifications.

**Plan to cash**:
- **Owner**: Director-tier coordination (cross-Mgr); PM tracks. **R2-Evaluator joint precondition owner resolved 2026-05-14 UTC** via §4 sub-item 5 re-spawn of the R3 Evaluator Mgr lane.
- **Sub-program**: each precondition has its own program; this gap is meta-blocked
  - R2-Evaluator: 5 sub-lanes per Director audit (runtime_value_model + body_evaluator + lens_application_complete_reflection + witness_construction + cross_target_equivalence_harness); brief surface comprehensive per Director (c) (r2-evaluator-manager.md + 4 sub-briefs + 10+ PR-A-E + R3-tier per-slice briefs). Q-EVAL ratifications landed 2026-05-06/07 (G0d-Dispatch / Descent-Termination-Contract / Lens-Fold-First-Slice). **Owner resolved 2026-05-14 UTC:** R3 Evaluator Mgr lane re-spawned; ledger refresh currently leaves lens_application + witness_construction in-flight.
  - R2-Grounding-Rust+Python: gate #18 + extdeps work
  - T-LP/SG-0: same as Gap 1
  - Row-B materialization: T-LBP work
- **Effort estimate**: 2-4 months (joint precondition; bounded by the longest of 4). R2-Evaluator residual closure remains a dominant tail, but the 2026-05-14 UTC ledger refresh narrows the Evaluator portion to 2 in-flight sub-lanes.

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

The closure-ledger was refreshed against HEAD on 2026-05-14 UTC. Close still requires the remaining two R2-Evaluator cells to turn green and stay grep-verifiable in `docs/r2-closure-ledger.md`.

**Dispatch staffing prereq** (NOT a close criterion; Director note msg_f0a54769 Note 2 enforcement; Director note msg_f0a54769 Note 3 sequencing enforcement): execution of the 5 sub-lane closures required owner identification. **Resolved 2026-05-14 UTC:** operator §4 sub-item 5 ratified re-spawn and this R3 Evaluator Mgr lane is active. Remaining work is substrate-debt closure for the two in-flight cells, not staffing.

**Alternative disposition — FORECLOSED by operator §4 ratification 2026-05-13** (codex BLOCKING #11284 PR #3013 class enforcement — same semantic-dilution pattern as Gap 2; retained as audit-trail of foreclosed path): operator §4 Item 3 ratified **IN-R3 (4-joint-precondition cascade)** 2026-05-13; R4-defer / R1-horizon-narrow paths NOT available. **R2-Evaluator-tier per-sub-lane R4-carve** also FORECLOSED — operator §4 Item 3 IN-R3 ratification cashes the joint precondition rule, including the 5 R2-Evaluator sub-lane closures (substrate-debt-shaped close criterion per Director note msg_f0a54769 Note 2). Any future re-opening requires explicit operator override of the §4 ratification at gunbc#828.

---

### Gap 4 — Lens behavioral parity (gates #79 / #81 / #82 / #83)

**Promise** (r3-program-plan.md §1.6 Cluster F + interrogation §1.1/§1.3/§1.4): *"4 in-R3 lenses (complexity + cost + parallelism + effect_enum) behaviorally complete; lens capability register has zero proxy / zero stub."*

**HEAD evidence**:
- #79 complexity = **SATISFIED-BY-CONSTRUCTION** via temporary Rust cementing receipt (`complexity_lens_behavioral_completion.rs`) "until ComplexitySummary TestClaim literals are expressible"
- #80 cost = **PASSING** ✓
- #81 parallelism = **R3-LOAD-BEARING**, F-α sub-phase pending (Stage 2e walker port from `workflow_parallelism.rs` → `.dag`)
- #82 effect_enum = **R3-LOAD-BEARING**, F-β.1 canvas + F-β.2 atomic-migration pending
- #83 `lens_capability_register_zero_proxy_zero_stub` (**§1.8 program gate / Cluster F sub-phase F-γ.2**) = **DECLARED** — canonical receipt is the **post-all-four-BEHAVIORALLY-COMPLETE** register cascade per `docs/audit/r3-cluster-f-sequencing-plan-2026-05-09.md` §1.4.2 and `docs/r3-structure.md` §Acceptance (not merely PROXY/STUB-zero while two rows stay **PARTIAL**). `parallelism.dag` and `effect_enumeration.dag` remain **PARTIAL** in `docs/v3-lens-capability-register.md` and `std.verification` `lens_capability_register_rows` until **#81** / **#82** land. **Supporting CI (necessary not sufficient):** `lens_register_correspondence_test.rs` (`every_regen_lens_entry_has_a_capability_register_row`, `r3_gate_83_lens_capability_register_scope_is_explicit`, `r3_gate_83_lens_capability_register_has_zero_proxy_zero_stub`, `lens_capability_register_rows_match_md_v2_cementing_projection`) — regen→register discipline, **zero** `BEHAVIORALLY PROXY` / **zero** `BEHAVIORALLY STUB` on the four T-LBP basenames in the markdown capability table, and Band-C v2-cementing-slice alignment vs `std.verification` `lens_capability_register_rows`; **does not** advance the §1.8 ledger Status for gate #83 (INVARIANTS P2 — one predicate; no parallel "narrow PASSING" row state).

**What's missing**:
- Complexity: `ComplexitySummary` TestClaim literals (bridge-Rust to native-.dag migration)
- Parallelism: Cluster F sub-phase F-α (Stage 2e walker port)
- Effect: Cluster F sub-phase F-β.1 (canvas) + F-β.2 (atomic-migration impl)
- Register: **PARTIAL → COMPLETE** for parallelism + effect_enum via **#81** / **#82**, then **F-γ.2** register sweep for full **#83** closure (PROXY/STUB-zero sub-check above already green)

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
- Gate #42 `v2_directory_deleted` = **CONSUMER_LANDED + PASSING** via PR #2693; `src/v2/` is absent at HEAD.
- Gate #97 `method_template_projection_emit_shim_retirement_coherence` = **CONSUMER_LANDED + PASSING** for terminal coherence.
- Per gate row verbatim: *"Terminal retirement = v2 tree absent ⇒ lib + bin table + bin source absent; coherence holds continuously."*
- The Gap-4 emit shim is absent at HEAD: no `pb_method_template_projection_dag_emit.rs`, no `emit_method_template_projection` bin source, and no `[[bin]]` manifest table.

**What's missing**: no implementation work remains for Gap 6 after PR #2693; only close-ceremony predicate receipts remain to be synchronized.

**Plan to cash**:
- **Owner**: zesty-boar-261 (R3 Debt-Paydown Mgr) — terminal-deletion sweep completed by PR #2693.
- **Sub-program receipt**:
  1. Final v2 consumer audit completed before deletion.
  2. `src/v2/` deleted by PR #2693.
  3. Gate #97 coherence verified: lib absent ✓ + bin table empty ✓ + bin source absent ✓.
- **Effort estimate**: complete; maintenance-only if the close predicate ledger drifts.
- **Dependency** (Director feedback item 5 — explicit transitive depth call-out): Gap 6 has the deepest transitive chain in the program:
  - Gap 6 → depends on Gap 1 (PB-0 retirement complete) + Gap 3 (self-host fixed point R3-strong)
  - Gap 3 → depends on Gap 1 + R2-Evaluator + R2-Grounding-Rust+Python + Row-B materialization
  - Effective chain: **Gap 6 ← Gap 3 ← {Gap 1, R2-Evaluator, R2-Grounding, Row-B}** — 5+ deep
- **Position in close ceremony**: **Gap 6 IS the close-ceremony terminal gate** (per Director recommendation msg_cd2d8d7d item 5). The §1.8 row #97 substrate-state predicate is already PASSING at HEAD after PR #2693; broader T-V2-Retirement lane closure still waits for the ratchet → carrier → walker → emission → self-host → test-port cascade to close upstream.

**Close criterion**: `ls src/v2/ 2>&1 | grep "No such file"` returns true. §1.8 row #97 terminal-PASSING, with predicate execution recorded in `docs/audit/r3-close-predicate-execution-2026-05-13.md`.

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

- **`normalize(c: SymbolicCost)` body** (`src/v3/std/algebra.dag:537-548`) — revised per codex BLOCKING on PR #3037 sha 797a6d91: PRIOR FRAMING missed that `reduce_sum` calls `drop_dominated` (Sum-side asymptotic dominance reduction is ALREADY-LANDED at normalize-tier; the gap is at the classifier-tier for post-normalize multi-term composites):
  - Sum (top-level): drop `ConstantCost(0)` (additive identity)
  - **Sum dominance reduction (already-landed)**: `reduce_sum` calls `drop_dominated` on multi-term lists — strips dominated terms by asymptotic ordering (e.g. `Sum([Linear(n), Constant(5)])` → `Linear(n)` because `Constant` is dominated by `Linear`); post-drop single-survivor unwraps via `wrap_sum` to avoid invalid singleton `SumCost`. This means many Sum expressions reduce to a terminal variant at normalize-time + classify correctly via terminal arms; the gap is for post-normalize multi-term Sums with no single dominator
  - Product: collapse to `ConstantCost(0)` on multiplicative annihilator, drop `ConstantCost(1)` (multiplicative identity)
  - Single-element Sum/Product: collapse to the element
  - Product of two identical `LinearCost`s → `PolynomialCost(var, 2)` (n*n = n²) — the only Product-side composition fold
  - **NO log-power rule** (`log(n^k) → k * log(n)`), **NO log-product rule**, **NO nested-log handling**, **NO Product-side dominance reduction** (`reduce_product` only handles single-term unwrap + LinearCost² fold), **NO Product/Sum-to-named-tier normalization** for surviving composites (`Product([Linear, Log])` does NOT fold; stays `ProductCost` post-normalize, classifies to `ClassUnknown`).

- **What the compiler actually reports for nested algorithms at HEAD** (further revised — Sum-dominance survivor cases classify correctly via terminal arms; only post-normalize multi-term composites + Product compositions fall to Unknown):
  - **`Sum([Linear(n), Constant(5)])`** = single-dominator sum → normalize drops `Constant(5)` via `drop_dominated` → `Linear(n)` → **`ClassLinear`** ✓ (Sum-dominance flow handles this correctly)
  - **`Sum([Linear(n), Linear(m)])`** = multi-var multi-term sum (no single dominator) → normalize keeps as `Sum([Linear(n), Linear(m)])` → classify → **`ClassUnknown`** ✗ (composition handling absent at classifier-tier for surviving Sums)
  - `n log n` = `ProductCost([LinearCost(n), LogCost(n)])` → no Product-side reduction applies → **`ClassUnknown`** (classifier has no `n log n` arm; `ClassLinearithmic` is unreachable).
  - `n log(n^k)` — cannot be constructed directly (type-level: `LogCost` doesn't take `PolynomialCost`). Cost-lens fold canonicalization at construction time is UNVERIFIED at HEAD (lens-body audit deferred to Gap 11 sub-program step 1). If unconstructible: lens emits `UnknownCost(<reason>)` → classifier `ClassUnknown`.
  - `log(log(n))` — same: cannot be constructed (`LogCost` doesn't take `LogCost`).
  - `n² log n` = `ProductCost([PolynomialCost(n, 2), LogCost(n)])` → **`ClassUnknown`** (composition → Unknown).
  - `n log²n` = `ProductCost([LinearCost(n), LogCost(n), LogCost(n)])` → **`ClassUnknown`** (composition → Unknown).
  - `2^n` — no `ExponentialCost` substrate variant; cannot be represented in source. Would fall to `UnknownCost(<reason>)`.

- **`STOP SIGNAL` discipline** (`src/v3/std/algebra.dag` Pattern-3 audit block): the 7 SymbolicCost variants are declared "terminal; the asymptotic surface the thesis reasons about; any new variant should carry its own load-bearing reason." This forbids an 8th variant. It does NOT address (a) making existing variants (`LogCost`, `PolynomialCost`) recursive over `SymbolicCost` to be symmetric with `Product`/`Sum`, (b) the missing `ExponentialCost` variant, or (c) the classifier's lack of composition handling.

**What's missing** (recalibrated per actual HEAD evidence above — classifier composition-handling is the root issue, not lattice tier coverage):

1. **`classify_symbolic_cost` composition handling** (revised per codex BLOCKING on PR #3037 sha 797a6d91 — distinguishing already-landed Sum-side dominance from remaining classifier-tier gap): Sum-side dominance reduction is **ALREADY-LANDED at normalize-tier** (`reduce_sum` calls `drop_dominated` — multi-term sums where one term asymptotically dominates reduce to the survivor pre-classify, which then classifies correctly via terminal arms). The remaining gap is at the **classifier-tier for post-normalize surviving composites**: (a) `Product([Linear, Log])` → currently `ClassUnknown`, should be `ClassLinearithmic`, (b) `Product([Polynomial(k), Log])` → currently `ClassUnknown`, needs tier extension or named collapse, (c) post-normalize multi-term `Sum` with no single dominator (e.g. `Sum([Linear(n), Linear(m)])`) → currently `ClassUnknown`, needs classifier-tier handling (separate from the already-landed `drop_dominated` normalization). The dominant gap is **Product-side composition arms at the classifier-tier** + multi-var Sum survivors; Sum-side single-dominator flow already correctly produces terminal classes.
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

**HEAD evidence** (operator probe 2026-05-13; revised per codex BLOCKING on PR #3037 sha 797a6d91 + briansrls BLOCKING on PR #3038 inline at `:463`: **prior framing pointed at wrong authority** — `Quantifier::ForAll` + `QuantifiedTestClaim` + `ProgramGenerator` surface ALREADY exists at HEAD as the property-based-testing authority; gap is the RUNNER, not the substrate):

- **`ProgramGenerator` substrate carrier LANDED** (gate #86; `src/v3/std/verification.dag`: `type ProgramGenerator { ... }`). Used in carrier-surface compile test at `m1_5_verification_test.rs::program_generator_authoring_surface_compiles_cleanly` + bound into the existing `QuantifiedTestClaim` (see below).
- **`Quantifier = ForAll | Exists` sum** authored at `src/v3/std/verification.dag` (gate #85 surface). This is the **claim-layer quantifier** for property-based testing (NOT `ForAllTargets` which is a separate cross-target authority per Gap 2). Per the comment block above the type: *"Closed quantifier over a generated program family. This lives at the claim layer, not inside `TestPredicate`, so predicates keep their 'property of one program' meaning."*
- **`type QuantifiedTestClaim { name, generator: ProgramGenerator, quantifier: Quantifier, predicate: TestPredicate, requires: List<ResourceReference> }` LANDED at `src/v3/std/verification.dag:542`** — the existing single-authority for `ForAll<ProgramGenerator>` property-based claims. INVARIANTS P2 single-authority is structurally complete at the substrate level.
- **Integration into Suite + TestNode surface LANDED**: `type SuiteClaim = Enumerated(TestClaim) | Quantified(QuantifiedTestClaim)` at `verification.dag:594` + `type TestNodeRef = EnumeratedTestNode(TestClaim) | QuantifiedTestNode(QuantifiedTestClaim)` at `verification.dag:574`. The quantified-claim shape is wired into suite-membership + cost-attachment + obligation-projection (`obligation_for_quantified_claim` at `verification.dag:627`). Test fixture proves authoring surface compiles (`test_runner_test.rs:1247` `data smoke_quantified_claim: QuantifiedTestClaim = { ... }`).
- **Runner is `NotYetImplemented`** at `src/v3/compiler/src/test_runner.rs:2511`:
  > `ClaimResult::NotYetImplemented("QuantifiedTestClaim runner evaluation NotYetImplemented (gate #85 substrate-only landing; quantifier evaluation deferred to Cluster M Phase 2/3 per docs/r3-structure.md §\"T-Tests-As-Data-Completeness\" — gate #87 cementing-test discipline + gate #84 bulk-port lanes; tracking row at docs/r3-program-plan.md §1.8 row #85)")`
  Per the inline citation, runner wiring is a named gate #85 dissolution trigger via Cluster M Phase 2/3 work.
- **Complexity cementing test** (`src/v3/compiler/tests/integration/cementing/complexity_lens_behavioral_completion.rs`): **2 hand-authored test cases** at HEAD (`literal_bind_cements_constant_complexity_summary` + `recursive_countdown_cements_linear_work_and_span`). Zero random compositions; zero oracle comparison; zero quantifier-coverage.
- **No `proptest` / `quickcheck` / random-program-generation tests** for the complexity lens anywhere in `src/v3/compiler/tests/`.
- **Result**: gate #79 `lens_capability_register_zero_proxy_zero_stub` lens-completion for complexity can claim "behaviorally complete" while never having validated against arbitrary nested compositions (the substrate `SymbolicCost` algebra reach is **enormous** vs the 2 cementing cases covered). Closing gate #79 honestly requires (a) wiring the existing `QuantifiedTestClaim` runner (gate #85 follow-on per Cluster M Phase 2/3), (b) authoring a complexity-generator instance + oracle, (c) authoring property-based `QuantifiedTestClaim` data declarations against the wired runner.

**What's missing** (runner wiring + complexity-specific generator + oracle + property-based fixtures — NOT extending `ForAllTargets` per prior framing):

1. **`QuantifiedTestClaim` runner wiring** (replace `NotYetImplemented` at `test_runner.rs:2511`): the named gate #85 dissolution trigger; per the inline cite, deferred to Cluster M Phase 2/3 (gate #87 cementing-test discipline + gate #84 bulk-port lanes). Single-authority is the existing `QuantifiedTestClaim` — runner consumes `generator: ProgramGenerator` + `quantifier: Quantifier` + `predicate: TestPredicate` and evaluates the predicate over N samples drawn from the generator. NOT extending `ForAllTargets` (that's the cross-target quantifier on a different axis per Gap 2 close criterion).
2. **`ProgramGenerator` complexity-instance** (`data` of type `ProgramGenerator` for complexity testing): authored as `.dag` substrate per emission discipline; produces structurally-bounded random programs combining function primitives (constant / linear-loop / log-search / poly-iteration / product-nested / sum-branched / log-of-... per Gap 11 substrate decision). Each generated program has a known-correct asymptotic class so oracle comparison is deterministic.
3. **Complexity oracle** (reference implementation): for the structurally-bounded composition class the generator produces, computes expected `ComplexitySummary` (work + span). Authored as `.dag` function (NOT bridge-Rust oracle per `feedback_no_textual_enforcement_bridges`).
4. **Property-based `QuantifiedTestClaim` instances**: data declarations of type `QuantifiedTestClaim` with `generator = <complexity-generator-instance>`, `quantifier = ForAll`, `predicate = <complexity-oracle-equivalence-predicate>`. These are direct `data` declarations against the existing substrate — no new claim-layer carrier required.
5. **Coverage discipline at runner-wiring time**: runner accepts a sample-budget parameter (N≥100 per CI run; budgeted-N per CI capacity); explicit seed-pinning via `requires: List<ResourceReference>` if needed; fail-closed on oracle/lens divergence per `ClaimResult::Fail`; ratchet that monotonically increases sample size.
6. **Cementing-cases preserved** (subset of property-based corpus): the 2 existing hand-authored cases (`literal_bind`, `recursive_countdown`) remain as low-N seeds; the property-based corpus extends rather than replaces them.

**Plan to cash** (revised to target existing authority, NOT extension of wrong one):

- **Owner**: Verification Mgr (still-moth-538) — owns runner-wiring work as part of Cluster M Phase 2/3 (per `test_runner.rs:2511` named-dissolution-trigger cite); Substrate Mgr (warm-wolf-698) — co-owns the `ProgramGenerator` complexity-instance authoring + complexity oracle authoring as `.dag` substrate; Director ratifies any substrate-shape additions to `QuantifiedTestClaim` if extension turns out needed (per `feedback_construction_over_ratchets` discipline — model first, extend only if existing shape inadequate).
- **Sub-program**:
  1. **Audit `QuantifiedTestClaim` shape** (Substrate Mgr): confirm the existing `{ name, generator, quantifier, predicate, requires }` shape is sufficient for complexity-validation use case OR identify needed substrate extension (e.g., explicit sample-budget field if `requires` doesn't carry that semantic).
  2. **Author `ProgramGenerator` complexity-instance** (`dsl/std/test_generators/complexity_generator.dag` or similar): structurally-bounded composition generator producing programs of named asymptotic class.
  3. **Author complexity oracle** (`dsl/std/test_oracles/complexity_oracle.dag`): function from generated-program → expected `ComplexitySummary`.
  4. **Author property-based `QuantifiedTestClaim` instances** as `data` declarations against the existing substrate: `data complexity_property_validation: QuantifiedTestClaim = { name: ..., generator: <complexity-generator>, quantifier: ForAll, predicate: <oracle-equivalence>, requires: ... }`.
  5. **Wire the runner** at `src/v3/compiler/src/test_runner.rs:2511` (replace `NotYetImplemented` per the named gate #85 dissolution trigger; this is Cluster M Phase 2/3 lane scope per the inline cite). Runner consumes the `QuantifiedTestClaim` substrate; draws N samples from `generator`; evaluates `predicate` over each sample; returns `ClaimResult::Pass` if all samples pass, `Fail` on first divergence with witness.
  6. **CI integration**: property-based test runs in regular CI with budgeted sample-size; fail-closed on divergence; seed-pinned for reproducibility.
- **Effort estimate**: 2-3 weeks (ProgramGenerator-instance + oracle + property-based-claim instances + runner wiring + CI integration); parallelizable with Gap 11 substrate-shape canvas authoring + Cluster M Phase 2/3 lane work (which already owns the runner-wiring dissolution trigger per `test_runner.rs:2511`).

**Close criterion** (substrate-debt-shaped; targets existing authority):
- (a) `ProgramGenerator` complexity-instance landed in `dsl/std/` (or appropriate test-generator path) with structural bound + named composition class coverage
- (b) Complexity oracle authored as `.dag` function (no bridge-Rust oracle)
- (c) Property-based `QuantifiedTestClaim` data declarations land using the **existing** substrate (no new claim-layer carrier required absent audit (1) finding extension needed)
- (d) `QuantifiedTestClaim` runner wired at `test_runner.rs:2511` (replaces `NotYetImplemented`) — runs the property-based claim with N≥100 random compositions per CI run
- (e) Zero oracle-vs-lens divergence across the random-composition corpus
- (f) CI seed-pinning + reproducibility discipline ratcheted via test infrastructure

**Connection to Gap 11**: Gap 11 (LogCost asymmetry) addresses substrate-shape CAPABILITY (what compositions can be expressed); Gap 12 addresses lens-COVERAGE VALIDATION (whether the lens correctly classifies what the substrate expresses). Both are needed for gate #79 complexity behavioral close to honestly mean "lens correctly handles arbitrary nested compositions". Gap 11 substrate canvas should land first so Gap 12 generator can produce the full composition class.

**Alternative disposition — FORECLOSED at authoring** (per `project_no_r4_carves_directive`): R4-defer of property-based validation would scope-narrow gate #79 behavioral close to "passes hand-authored cementing cases only". The operator-adversarial probe 2026-05-13 surfaced this gap; gate #85 `forall_exists_quantifier_substrate_landed` + gate #86 `program_generator_carrier_landed` are R3-load-bearing per the Tests-as-Data-Completeness lane — using them for actual property-based validation (not just substrate-compile checks) is the natural R3 close shape.

**Authority**: operator adversarial probe 2026-05-13 (this PR); gate #79 close criterion in close plan §1 Gap 4 "complexity behavioral parity"; gates #85 + #86 close criteria in close plan §1 Gap 5 "tests-as-data completeness"; substrate-shape decision routes through Director ratification (msg-routing TBD post-canvas).

---

### Gap 13 — R2-Grounding T-Ground sub-lane residuals (no-coercion-engine architectural separation; emit.rs ~3992-line hand-Rust survivor)

**Promise** (`docs/design-emission-model.md` title: *"Design — Emission Model (no separate coercion engine)"* + line 13: *"Coercion is structural projection, not a decision process. Emission reads declared substrate facts and returns either a unique target primitive or a fail-closed diagnostic. There is no parallel authority that 'picks up slack' when the substrate is incomplete; if the substrate under-determines, the substrate is incomplete and the right response is to extend the substrate, not bolt on a decider."*): emission separates into (a) **structural-projection coercion** (substrate fact lookup, fail-closed on under-determination) over (b) **DAG-modeled substrate** (`LanguageSpec` + `TypeRealization` per-target inhabitance + per-program structural intent derivation). NO hand-Rust coercion engine; emission is a `.dag`-authored fold over substrate facts.

**HEAD evidence** (operator adversarial probe 2026-05-13 follow-on; revised per Director audit msg_8ae92369 2026-05-13 — **PM originally cited 5 T-Ground sub-lanes; actual ledger count is 11** per `docs/r2-closure-ledger.md:108` + `docs/briefs/r2-grounding-manager.md:168` "now 11 lanes; engine-reframe locked 2026-04-28"; close criterion + dispatch shape recalibrated against the correct surface):

- **`docs/design-emission-model.md`** explicitly retracts the v2 coercion-engine framing and ratifies the no-engine discipline. The R2-T-Ground sub-lane structure (per `docs/r2-closure-ledger.md:108-122` ratified 2026-04-28 engine-reframe) is **11 sub-lanes**, not the 5 PM originally enumerated:

  **GREEN at HEAD (1 of 11)** per Director audit:
  - `T-Ground-Pilot` → `pilot_inhabitance_routing_stability_landed` (PR #765 merged 2026-04-25)

  **IN-FLIGHT (7 of 11)** per closure-ledger; HEAD-state likely partial-cashed via R3-tier slices analogous to R2-Evaluator `runtime_value_model_structural` pattern:
  - `T-Ground-Rust` → `rust_target_primitives_structural` (PR #1005)
  - `T-Ground-Python` → `python_target_primitives_structural` (PR #1080)
  - `T-Ground-Go` → `go_target_primitives_structural` (commit `ac765ce10` + PR #1046)
  - `T-Ground-LanguageSpec` → `language_spec_language_agnostic_structural` (PR #1168/#1174/#1195/#1196/#1210-#1213; Phase 1 registry rows landed; Phase 2 partial)
  - `T-Ground-Coercion-Fold` → `coercion_fold_structural` (PR #989, commit `c0cc8b260`, PR #1241 scaffold crate; **R3-tier slice PR #1980 Coercion-Fold ScratchIntExamples retirement merged + #2279 SelectedTargetInhabitance** — likely closer to GREEN than ledger reads)
  - `T-Ground-Lifetime-Analyzer` → `lifetime_analyzer_structural` (PR #1177/#1206/#1218/#1220; R2-scope a/b/c basic cases impl; advanced cases d/e/f folded into T-LP-Retirement lane per `design-emission-model.md` Open call 2)
  - `T-Ground-CrossTarget-Meta` → `cross_target_meta_structural` (PR #1224 brief + PR #1414 L6 coverage fold + **PR #2103 L6 EmissionPathProjection CLOSED**)

  **NOT-STARTED (3 of 11)** — brief-only at R2-close; no implementation work landed:
  - `T-Ground-Diagnostic` → `target_primitives_diagnostic_structural` (brief PR #1216 `t-ground-diagnostic.md`; Q6.5 Layer-1 CompilerDiagnosticKind authority)
  - `T-Ground-Tests` → `grounding_tests_structural` (brief PR #1223 `t-ground-tests.md`; L4 implementation gated Q4)
  - `T-Ground-Dissolve` → `grounding_dissolve_structural` (brief PR #1234 `t-ground-dissolve.md` + Mgr closure sweep PR #1240)

  Director estimation: probably 3-5 of 11 effectively GREEN at HEAD via R3-tier slice landings (Pilot + Coercion-Fold + CrossTarget-Meta L6 likely; possibly LanguageSpec); 4-6 still in-flight; 3 not-started. Full per-sub-lane HEAD audit needed for authoritative read (analogous to neat-heron-793's closure-ledger refresh for R2-Evaluator).

- **`src/v3/compiler/src/emit.rs` (3992 lines, hand-Rust)** is the legacy v2 coercion engine that the design retracts. Still active at HEAD. On the PB-0 retirement ratchet (`EXPECTED_HAND_AUTHORED_NON_TEST:279`).
- **`src/v3/std/emit_model.dag`** exists but is explicitly marked **🟡 SCAFFOLD (Coercion-Fold dissolution — Slice B rows, Slice C consumer)**. Per-target inhabitance carrier (`TypeRealization`) is partially-stubbed; Slice B (per-target integer inhabitance rows) + Slice C (consumer wiring) not yet landed.
- **`dsl/std/coercion.dag`** exists with the new coercion vocabulary (`TypeCheckpoint`, per-target inhabitance schema) BUT still names `v2/05_emit.dag` as a consumer in its header comment — transitional form; legacy coercion engine not yet retired.
- **R2-Grounding closed-with-residuals 2026-04-29** (analogous shape to R2-Evaluator per Director audit msg_82b9c4bb). 11 T-Ground sub-lanes are R2-residual work carried into R3 as r3-continuation.
- **Stale-row date pattern**: closure-ledger frozen at #1168-#1241 era for grounding (identical pattern to #1191-#1231 era for evaluator). Both ledgers frozen at R2-close 2026-04-29; neither refreshed against HEAD. **R2-Grounding-Mgr scope is the larger residual surface** (11 sub-lanes vs R2-Evaluator's 5).
- **Named owner at HEAD**: NO R3 Grounding Mgr session — `dashboard-ops graph zesty-bear-812` shows 3 R3 Mgr sessions (warm-wolf-698 / zesty-boar-261 / still-moth-538), no R3 Grounding Mgr. Same anti-pattern as merry-gull-128 absence flagged in R2-Evaluator audit. **T-Ground execution authority has partially dispersed under warm-wolf-698 (Substrate Mgr) organically** — PR #1980 Coercion-Fold ScratchIntExamples retirement, PR #2103 L6 EmissionPathProjection CLOSED, PR #2272 u128 row, PR #2279 SelectedTargetInhabitance, PR #2229 cost_target_realization registry — but no single Mgr lane owning the 11 sub-lanes at HEAD. Absorption is partial + organic, not by named program ownership.
- **Brief coverage is COMPREHENSIVE** (per Director audit (c) — even stronger case than R2-Evaluator): `docs/briefs/r2-grounding-manager.md` (Mgr program brief) + 8 dedicated T-Ground sub-lane briefs (`t-ground-cross-target-meta.md` / `t-ground-diagnostic.md` / `t-ground-dissolve.md` / `t-ground-engine-phase-1.md` / `t-ground-engine-substrate-audit.md` / `t-ground-languagespec.md` / `t-ground-lifetime-analyzer.md` / `t-ground-rust-full-implementation.md` / `t-ground-tests.md`) + 9+ R3-tier per-slice briefs + R2-PR-D cross-target equivalence briefs. Brief-readiness preconditions for re-spawn firmly met.
- **Close plan §1 at HEAD** does NOT track these residuals as an explicit Gap. Operator adversarial probe 2026-05-13 surfaces this as a missed-during-original-sweep gap analogous to the R2-Evaluator residuals that Gap 3 absorbed.
- **emit.rs retirement is structurally gated** on the 11 T-Ground sub-lanes + R2-Evaluator (which runs the `.dag`-authored emitter) + PB-0 retirement campaign. PB-0 ratchet (177 entries) tracks emit.rs entry-counting but NOT the architectural-shape verification.

**What's missing** (recalibrated per Director audit msg_8ae92369 — 11 sub-lanes total, not 5):

1. **Per-sub-lane HEAD refresh** (analogous to neat-heron-793's R2-Evaluator closure-ledger refresh): each of 11 T-Ground sub-lanes ratchet-to-PASSING in `docs/r2-closure-ledger.md` per cell-level check, refreshed from #1168-#1241 era to HEAD. Director estimation: 3-5 of 11 likely already GREEN at HEAD via R3-tier slice landings; verification pass needed for authoritative read.
2. **3 NOT-STARTED sub-lanes** require fresh dispatch — `T-Ground-Diagnostic` + `T-Ground-Tests` + `T-Ground-Dissolve` are brief-only at R2-close.
3. **7 IN-FLIGHT sub-lanes** require closure-walk + completion per their R3-tier slice landings — Rust / Python / Go / LanguageSpec / Coercion-Fold / Lifetime-Analyzer / CrossTarget-Meta.
4. **Scope-discrimination canvas** (per Director audit (d) critical caveat): before dispatching a dedicated Grounding Mgr, Mgr-tier brief authoring must discriminate (i) Grounding-owned scope (genuinely substrate-fact-introduction or grounding-discipline-specific) vs (ii) Substrate-Mgr-already-absorbed scope (warm-wolf-698's organic absorption via overlapping substrate-canvas work — Coercion-Fold retirements, emission-provenance substrate, L6 EmissionPathProjection). Without this discrimination, a new Grounding Mgr risks being a thin wrapper over Substrate Mgr's running work OR creating dual-authority conflicts on overlapping sub-lanes.
5. **`emit_model.dag` SCAFFOLD marker removed**: Coercion-Fold dissolution Slice B rows + Slice C consumer lands; carrier (`TypeRealization`) becomes structurally complete.
6. **`coercion.dag` schema dissolution complete**: `v2/05_emit.dag` consumer reference retired from header comment; new coercion vocabulary is single-authority for emission.
7. **emit.rs retirement consumer ratchet**: as `.dag`-authored emission consumers land per the 11 sub-lanes, emit.rs entry on `EXPECTED_HAND_AUTHORED_NON_TEST` retires; ratchet ratchets toward 0 (Gap 1).
8. **§1.8 ledger row** for "no-coercion-engine discipline cashed at HEAD" — currently no gate tracks the architectural-shape verification (PB-0 ratchet is mechanical entry-counting, not architectural). Either author new §1.8 row OR formally declare existing gate covers (per Director audit verdict).

**Plan to cash**:
- **Owner**: Director-tier coordination (analogous to Gap 3 cross-Mgr audit; analogous to R2-Evaluator §4 sub-item 5 framing); execution Mgr-tier per §4 sub-item 6 ratification (see §4 below — re-spawn R3 Grounding Mgr OR fold into warm-wolf-698 Substrate Mgr OR Director-direct ad-hoc).
- **Sub-program**:
  1. **Director R2-Grounding audit complete** (Director-tier deliverable; **landed 2026-05-13 per msg_8ae92369**): 4-section audit (a)-(d) per analogous shape to msg_82b9c4bb. Outputs: (a) 11 sub-lane status at HEAD; (b) NO R3 Grounding Mgr session in current subtree; authority partially dispersed under warm-wolf-698 organically; (c) brief surface comprehensive (8 T-Ground briefs + 9+ R3-tier slice briefs); (d) PM-recommend OPTION A re-spawn 5th R3 Mgr lane with scope-discrimination caveat.
  2. **Operator §4 sub-item 6 ratification**: R3 Grounding Mgr dispatch shape (re-spawn / fold / Director-direct) — see §4 below. Bundle with §4 sub-item 5 (Evaluator) per Director recommendation since both need same dashboard-tier intervention (composite-shape support per operator escalation msg_acf78d37).
  3. **Scope-discrimination canvas** (post-§4-sub-item-6 ratification): if OPTION A re-spawn, Mgr-tier brief authoring discriminates Grounding-owned scope vs Substrate-Mgr-already-absorbed scope; clean handoff coordination on overlapping sub-lanes.
  4. **Per-sub-lane dispatch** (post-discrimination canvas): owner Mgr executes the 11 sub-lane closures per their refresh-against-HEAD status; worker briefs per sub-lane.
  5. **`emit_model.dag` SCAFFOLD dissolution**: Coercion-Fold Slice B + Slice C land; carrier becomes structurally complete.
  6. **`coercion.dag` v2/05_emit.dag consumer reference retirement**: cleanup commit retiring the transitional reference once new coercion vocabulary is single-authority.
  7. **§1.8 row decision**: author "no-coercion-engine discipline cashed" row OR formally declare existing gate covers per Director ratification.
  8. **emit.rs entry retirement** (downstream of sub-lane completions): consumer ratchet to 0 per Gap 1 sub-program coordination.
- **Effort estimate**: 8-16 weeks (revised up from 6-12 per recalibration — 11 sub-lanes vs originally-claimed 5; scope-discrimination canvas added). Analogous to Gap 3 R2-Evaluator joint precondition magnitude.

**Close criterion** (substrate-debt-shaped only — per Director audit msg_8ae92369 Note 2 enforcement (carried forward from msg_f0a54769): staffing/dispatch shape is a PRECONDITION for execution, not a close criterion for the substrate-debt itself; moved to "Dispatch staffing prereq" below):

- (a) All 11 R2-T-Ground sub-lanes status=green at HEAD per cell-level check of `docs/r2-closure-ledger.md:108-122` (refreshed from #1168-#1241 era to HEAD per Note 1 α-path) —
  - `pilot_inhabitance_routing_stability_landed` = green
  - `rust_target_primitives_structural` = green
  - `python_target_primitives_structural` = green
  - `go_target_primitives_structural` = green
  - `language_spec_language_agnostic_structural` = green
  - `coercion_fold_structural` = green
  - `lifetime_analyzer_structural` = green
  - `cross_target_meta_structural` = green
  - `target_primitives_diagnostic_structural` = green
  - `grounding_tests_structural` = green
  - `grounding_dissolve_structural` = green
- (b) `src/v3/std/emit_model.dag` 🟡 SCAFFOLD marker removed (Coercion-Fold dissolution Slice B + Slice C landed)
- (c) `dsl/std/coercion.dag` v2/05_emit.dag transitional reference removed
- (d) `src/v3/compiler/src/emit.rs` entry removed from `EXPECTED_HAND_AUTHORED_NON_TEST` (substrate retirement complete; ratchet ratchets toward Gap 1 close)
- (e) §1.8 row for no-coercion-engine discipline either landed or formally declared-covered-by-existing-gate per Director ratification

The closure-ledger row currently stale @ #1168-#1241 era; close also requires the ledger be refreshed against HEAD before status-evaluation. Grep-verifiable predicate against `docs/r2-closure-ledger.md` cell content.

**Dispatch staffing prereq** (NOT a close criterion; Director audit msg_8ae92369 Notes 2 + 3 enforcement carried forward from msg_f0a54769): execution of the 11 sub-lane closures requires owner identification. **R3 Grounding Mgr lane disposition RATIFIED 2026-05-13** per §4 sub-item 6: **(α) re-spawn as 5th R3 Mgr lane** confirmed by operator briansrls via bundled-5-asks PM-routing (see §4 + §6 ratification outcomes). **Sequencing**: re-spawn occurs AFTER `--shape` flag landing per §4 Ask 1 ratification (Director dispatched `--shape` flag worker adhoc-745d73fa-6c4 per msg_7ce4dcc0 → msg_14c3ad9d). Director executes the re-spawn the moment the `--shape` flag worker lands; orientation brief pre-authored. PM-recommendation Option (α) is on-record + operator-cashed; **execution is now gated on dashboard-tier `--shape` flag availability, NOT on operator confirmation** (which is already in place).

**Connection to Gap 1 + Gap 3**: Gap 13 is the architectural-shape sibling to Gap 1 (PB-0 mechanical retirement ratchet) — Gap 1 says "list empty"; Gap 13 says "the architectural separation that justifies the list-empty outcome is structurally complete". Gap 13 is the analogous R2-residual to Gap 3 (R2-Evaluator); both surfaced as post-§4-ratification adversarial findings (R2-Evaluator via Director audit msg_82b9c4bb; R2-Grounding via operator adversarial probe 2026-05-13 on emission architecture).

**Alternative disposition — FORECLOSED at authoring** (per `project_no_r4_carves_directive`): R4-defer of architectural-separation completeness would leave emit.rs as a permanent hand-Rust dependency, contradicting both PB-0 ("0 hand-Rust") and the design-emission-model.md no-engine discipline. Operator-adversarial probe 2026-05-13 surfaced this gap; gap is now operator-recorded and must close IN-R3 absent explicit operator override of the no-carves directive with named structural-unblockable reason.

**Authority**: operator adversarial probe 2026-05-13 (this PR); `docs/design-emission-model.md` no-engine discipline ratification; r3-structure.md §"Recommended R3 lane structure update" T-Ground-* lane enumeration; analogous structural shape to Gap 3 R2-Evaluator residuals per Director audit msg_82b9c4bb.

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
- **Gap 13 (R2-Grounding T-Ground sub-lane residuals)**: Director-tier audit **complete 2026-05-13 (msg_8ae92369)** — recalibrated count: **11 R2-T-Ground sub-lanes** (NOT 5; original PM count was a design-emission-model.md "engine-reframe 5 lanes" subset miss per `feedback_full_predicate_over_categorized_grep_in_scope_statements`). Post-audit dispatch shape: **R3 Grounding Mgr (5th R3 Mgr lane, re-spawn (α) RATIFIED by operator 2026-05-13 per §4 sub-item 6)** executes the 11 sub-lane closures + scope-discrimination canvas as Mgr-tier first-deliverable (discriminate Grounding-owned scope vs Substrate-Mgr-already-absorbed scope per Director (d) audit caveat); PM authors §1.8 row per Director ratification. **Sub-promise of design-emission-model.md no-engine discipline + emit.rs retirement surfaced by operator adversarial probe 2026-05-13** post-§4-ratification. Execution gated on `--shape` flag landing per §4 sub-item 1 ratification (Ask 1 bundled-5-asks 2026-05-13).

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

**Initial ratification batch (2026-05-13, PR #3013 merge)** — operator confirmed via PM AskUserQuestion:
- **Items 1-4 (R3 scope decisions)**: ALL **IN-R3** confirmed. No R4-carves.
  - Gap 1 (PB-0): IN-R3 — full 177-entry retirement
  - Gap 2 (L5 cross-target): IN-R3 — full 3-target Python+Go
  - Gap 3 (self-host R3-strong): IN-R3 — 4-joint-precondition cascade
  - Gap 9 (show-correct-code): IN-R3 — 100% absolute (zero `DeferredCorrection` per sum-variant carrier)
- **Item 5 (subtree-shape decision)**: **(a) re-spawn R3 Evaluator Mgr as 4th R3 Mgr lane** ratified. Director (zesty-bear-812) executes per pre-authorization msg_d456b60d.
- **Phase A immediate dispatch**: authorized.

**Bundled-5-asks ratification batch (2026-05-13, PR #3038 routing)** — operator confirmed via PM AskUserQuestion (Director-recommended bundling per msg_eaaca237 + msg_922eac5b; PM-routing per msg_7ce4dcc0):
- **Item 6 (subtree-shape decision — Grounding Mgr)**: **(α) re-spawn R3 Grounding Mgr as 5th R3 Mgr lane** ratified. Scope-discrimination canvas as Mgr-tier first-deliverable per Gap 13 sub-program step 3 (discriminate Grounding-owned vs Substrate-Mgr-already-absorbed scope per Director (d) audit caveat).
- **Dashboard-tier intervention**: (b) durable `--shape` flag in `dashboard-ops work-items create` authorized (unblocks both Evaluator + Grounding Mgr re-spawn + all future Mgr-tier spawns).
- **Cursor-composer-2 parser fix**: (a) fix-dispatch authorized (Director or Mgr dispatches parser-fix worker; class-level fix unblocks PR #3014/#3025/#3036/#3037 + all future cursor-composer-2 reviews).
- **PR #3036 + PR #3025 merge-bypass directive**: Director squash-merge both authorized — operator-named directive given; precondition (2) of `feedback_operator_tier_merge_bypass_precedent` cashed.

The presentation framing below preserved as audit-trail of the operator-questions that landed the ratifications.

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

**5. R3 Evaluator Mgr dispatch (subtree-shape decision; surfaced per Director audit msg_82b9c4bb 2026-05-13; RATIFIED (a) re-spawn by operator briansrls 2026-05-13 via bundled-5-asks PM-routing — executed 2026-05-14 UTC after `dashboard-ops --shape` landed)**: structurally distinct from the 4 scope decisions above — this decision changes Director subtree shape rather than R3 surface scope. Per Director audit findings at decision time: (a) R2-Evaluator closed-with-residuals with 5 sub-lanes carried into R3; (b) R3 Evaluator Mgr merry-gull-128 (#1743) absent from the then-current subtree; (c) brief surface comprehensive; (d) Director-recommends re-spawn as 4th R3 Mgr lane. PM-recommendation **OPTION A: re-spawn evaluator Mgr** per:
   - `feedback_standing_managers_need_owned_deliverables` — 5 named sub-lanes (runtime_value_model + body_evaluator + lens_application_complete_reflection + witness_construction + cross_target_equivalence_harness) is an owned-program count that justifies a Mgr lane.
   - `feedback_pre_authored_brief_queue` — brief surface already comprehensive per Director (c); re-spawn does NOT bottleneck on Mgr-tier dispatch authoring.
   - Operator standing directive *"staffing not a concern"* (2026-05-13 PR #3013 ratification thread) frames 4th R3 Mgr lane as IN-policy.
   - Existing 3-Mgr R3 template (Substrate / Verification / Debt-Paydown) symmetry — 4th lane is structurally parallel, not a new Mgr-tier shape.

   Operator sub-decisions (resolved by 2026-05-13 ratification):
   - (a) **Re-spawn R3 Evaluator Mgr as 4th lane** (PM-recommended; Director-recommended primary option). Brief surface ready; sub-lanes named; owned-program count meets bar.
   - (b) **Fold into existing R3 Mgrs**: witness_construction + cross_target_equivalence to Verification (swift-deer-459); runtime_value_model + body_evaluator to Substrate (warm-wolf-698); lens_application_complete_reflection to whichever has lower load. Lower agent-spawn cost; risk: scope-bloat under existing R3-lane load creates dual-program lane (the same anti-pattern that produced the dispersion).
   - (c) **Director-direct ad-hoc dispatch**: structurally equivalent to retracted r2-structure.md:73 anti-pattern ("standing managers without owned deliverables degenerate into pass-through hops; concentrating brief-authoring on Director starved lanes"); PM does NOT recommend.

**6. R3 Grounding Mgr dispatch (subtree-shape decision; surfaced per Director audit msg_8ae92369 2026-05-13; RATIFIED (α) re-spawn by operator briansrls 2026-05-13 via bundled-5-asks PM-routing — see Ratification outcomes above)**: structurally distinct from the 4 scope decisions above + parallel to sub-item 5 (Evaluator Mgr) — this decision changes Director subtree shape rather than R3 surface scope. Per Director audit findings: (a) R2-Grounding closed-with-residuals 2026-04-29 with **11 T-Ground sub-lanes** carried into R3 (1 GREEN + 7 in-flight + 3 not-started; Director estimation 3-5 of 11 likely effectively GREEN at HEAD via R3-tier slice landings); (b) NO R3 Grounding Mgr session in current subtree; authority partially dispersed under warm-wolf-698 organically; (c) brief surface COMPREHENSIVE (8 dedicated T-Ground briefs + 9+ R3-tier slice briefs); (d) Director-recommends re-spawn as 5th R3 Mgr lane with scope-discrimination caveat. PM-recommendation **OPTION (α): re-spawn Grounding Mgr** per:
   - `feedback_standing_managers_need_owned_deliverables` — 11 named sub-lanes is a larger owned-program count than R2-Evaluator's 5; even stronger case for dedicated Mgr.
   - `feedback_pre_authored_brief_queue` — brief surface comprehensive per Director (c); no Mgr-tier authoring bottleneck.
   - Operator standing directive *"staffing not a concern"* (2026-05-13 PR #3013 ratification thread) frames 5th R3 Mgr lane as IN-policy.
   - Symmetry with existing R3 Mgr template (Substrate / Verification / Debt-Paydown + Evaluator re-spawn → 5th lane structurally parallel).

   **Critical caveat (Director audit (d) discrimination requirement)**: before authoring the canvas, Mgr-tier brief authoring must discriminate (i) Grounding-owned scope vs (ii) Substrate-Mgr-already-absorbed scope. warm-wolf-698 has organically absorbed parts of T-Ground sub-lane work via overlapping substrate-canvas authoring (Coercion-Fold retirements + emission-provenance substrate + L6 EmissionPathProjection). Without discrimination, a new Grounding Mgr risks being a thin wrapper OR creating dual-authority conflicts on overlapping sub-lanes. Clean handoff coordination required.

   Operator sub-decisions:
   - (α) **Re-spawn R3 Grounding Mgr as 5th lane** (PM-recommended; Director-recommended primary option, with scope-discrimination canvas as first deliverable). 11 named sub-lanes; brief surface ready; owned-program count meets bar.
   - (β) **Fold into existing warm-wolf-698 Substrate Mgr**: Substrate Mgr already partially-absorbed T-Ground sub-lanes organically; folding formalizes the absorption. Risk: substantial dual-program lane shape (Substrate substrate-shape work + Grounding execution work); scope-bloat under existing Substrate Mgr load (9-worker Phase B batch + Cluster M Phase 3 coordination + canvas authoring); same anti-pattern as the dispersion that produced the gap.
   - (γ) **Director-direct ad-hoc dispatch**: structurally equivalent to retracted r2-structure.md:73 anti-pattern; PM does NOT recommend (same rejection as sub-item 5 (c)).

   **Bundling with sub-item 5**: per Director audit msg_8ae92369, both §4 sub-item 5 (Evaluator Mgr re-spawn) and §4 sub-item 6 (Grounding Mgr re-spawn) need the same dashboard-tier intervention (composite-shape support per operator escalation msg_acf78d37 in flight). Recommend bundling both into one operator-ratification batch — the dashboard-tier intervention unblocks both lanes simultaneously.

**§5 process discipline note**: per the standing directive, asking the operator to choose IN-R3-vs-R4-carve framing for the 4 scope items implicitly invites R4-carve consideration. Re-framing per Director feedback item 1: the question is "confirm IN-R3 (default, per directive + PM-recommended)" — explicit override only if structurally unblockable. **Sub-items 5 + 6 (Mgr-dispatch) are NOT scope questions**; they ask operator to ratify subtree-shape change. **Both sub-items 5 + 6 RATIFIED 2026-05-13** per Ratification outcomes above — Gap 3 + Gap 13 close-criteria no longer meta-blocked; substrate-debt execution proceeds per the ratified Mgr-lane dispatch shape (Director executes re-spawn post `--shape` flag landing per bundled-5-asks Ask 1).

---

## §5. Process discipline (preventing future drop)

Operator framing 2026-05-13: *"planning for this stuff has been continuously dropped."*

Anti-pattern observed: closure-ceremony work is ad-hoc and gets bumped by reactive work (PR reviews, daily merges, Mgr coordination). The closure receipt (Gap 10) was the first thing to drop.

**Process discipline going forward**:
1. **This doc is the single authoritative closure plan**. Updates land via PR with operator review. No parallel planning docs.
2. **Weekly PM closure-cadence message to Director + operator**: progress vs plan, gaps closing, blockers surfaced. Cadence: every Monday.
3. **Per-gap closure PR template**: each gap-closure PR must cite the gap number from this doc + the close criterion satisfied + audit evidence path.
4. **Gap 10 (close-audit doc) is authored FIRST** — it's the receipt mechanism for everything else. Without it, "closed" is unverifiable.

### §5.1 Per-PR ratchet-direction discipline

**Authority**: operator Decision A 2026-05-13 ("target 0 is met; work orthogonal" — net-direction-down is acceptable in any single PR as long as ultimate target = `EXPECTED_HAND_AUTHORED_NON_TEST = []` + `EXPECTED_HAND_AUTHORED_TEST = []` per Gap 1 + Gap 5 close criteria); Director recommended shape per msg_48439a7f 2026-05-13 + msg_0b8f9283 PendingFact-shape discrimination; PM authoring per msg_be7a26b4 commitment.

**Drift class empirical baseline** (per `feedback_awk_range_per_const_array_not_whole_file_grep` verified counts; 10 commits since 2026-05-08 modifying `sg0_census_test.rs`):
- Combined ratchet net trajectory: **+8 entries** (+2 `EXPECTED_HAND_AUTHORED_NON_TEST` + +6 `EXPECTED_HAND_AUTHORED_TEST`)
- Single retirement commit in the window (`b8651008` T-Cluster-F-α S1 parallelism walker port; -1 NON_TEST)
- 9+ addition commits; retirement rate ~10% of additions
- Sustained growth rate ~+0.8 entries per commit
- Direction: structurally AWAY from R3 close (which requires 0/0)

This is `feedback_ratchet_only_down` drift; sustained pattern, not 5-commit anomaly. The §5.1 discipline prevents continued drift while substrate-completion lanes land their retirement-substrate enablers.

**Per-PR assertion at PR-template tier**: any PR that increases either `EXPECTED_HAND_AUTHORED_NON_TEST` OR `EXPECTED_HAND_AUTHORED_TEST` const-array length must include in PR body:

1. **Add-class classification** per the discrimination framing below — author classifies each new entry as one of:
   - **(A) PendingFact-shape addition** — substrate-prereq carrier building toward `ResolvedFact` materialization (per `GeneratedManifestEntry = PendingFact | ResolvedFact` sum-variant landing via warm-wren-479 PR #3040 or analogous substrate); cite the named substrate-prereq + named materialization path
   - **(B) Cascade-substitution net-direction-down** — worker substitutes 1 entry with multiple finer-grained entries (e.g., swift-bee-15 PR #3046's `+13/-17 = net -4`); explicit add/retire commentary in PR body listing which entries are added vs retired
   - **(C) Hand-Rust without retirement-substrate** — requires ALL of: (i) Director-tier ratification + named structural-unblockable reason (per `project_no_r4_carves_directive` analogous discipline); (ii) **INVARIANTS.md P5 Dispatch-Discipline Mechanism (b) single checkable receipt** — exactly one of: deleted file/scaffold path, SG-0 census line shrink with before/after counts in `sg0_census_test.rs`, OR explicit deferral that **NAMES a lane AND cites a concrete `ROADMAP.md` row** (path + heading/anchor or permalink — vague deferrals do NOT satisfy); (iii) **for pipeline-stage entries** (`emit.rs` / `lower.rs` / `infer.rs` / `parse.rs` and adjacent per `SELF_HOSTING.md` §2 migration order), the stage's L2.5 domain-model SET per `SELF_HOSTING.md` §2.2 (parse domain / lower domain / infer domain / emit domain enumerations — every cross-stage-boundary or lens-consumed type modeled in `std/` and `extdeps/`; walker-local state stays in stage body per `std/`-vs-implementation split) MUST be landed-and-reviewed as a **precondition** per `SELF_HOSTING.md` §2.2 Step 1 — template-relocation paper-shrink (content-identical file at different path with codegen-driver wrapper that just re-emits the same bytes) is **FORECLOSED** per April PR #729 precedent (`a0f0b7837`: *"Real retirement should happen via actual deletion, generated ownership, or another lawful dissolution path"*) extended by 2026-05-14 cycle-4 + cycle-5 PR #3048 / PR #3057 paper-shrink discovery (`tools/pb0_cycle4_emit_templates/*.rs.in` content-identical to source `.rs` minus 5-line AUTO-GENERATED header; all 4 §2.2 steps violated)

2. **Gap classification cite** per the per-entry blocker taxonomy at `docs/audit/r3-pb0-non-test-retirement-class-taxonomy-2026-05-13.md` (Track A taxonomy doc, in flight under zesty-boar-261 / PR #3045): cite which class (a / b / c) per the taxonomy each added entry belongs to + the named blocker if any (e.g., "emit.rs depends on Gap 13 R2-Grounding T-Ground sub-lanes per close plan §1").

3. **Net-direction commentary** per operator Decision A — if the PR ratchet-array length INCREASES net, name the structurally-unavoidable reason; if DECREASES net, name the retirement-class (a/b/c). Cascade-substitution PRs (class B above) name both.

**Three-tier review enforcement chain** (worker-tier → PR-template → Director-tier):

1. **Worker-tier substrate-audit (first chain)**: workers authoring substrate-shape work that would add to ratchets MUST run a 4-axis pre-commit substrate audit per `feedback_grep_carrier_semantic_before_ratification` (extended 4-axis check 2026-05-13 per Director msg_c79a9b8d):
   - **Axis 1 — name-shape**: does the carrier name follow `dsl/std/` convention + no collisions
   - **Axis 2 — semantic-carrier-grep** (per existing memory): is there an existing carrier with the same SEMANTIC role (e.g., byte-identity vs registry-key vs content-hash discrimination per PR #2746 audit class)
   - **Axis 3 — constructability under existing inhabitants**: can the carrier be constructed from existing `data` declarations + algebra inhabitances without requiring new substrate-shape work
   - **Axis 4 — invariant-conformance vs §P1/§P2/§P5**: does the carrier obey Modeling Faithfulness, Boundary Discipline / single authority, and Progress is Dissolution per INVARIANTS.md
   - Cross-link to `feedback_substrate_principle_audit` 6-question audit for broader substrate-shape decisions
   - **Substantive evidence**: PR #3040 (warm-wolf-698 substrate refinement; warm-wren-479 worker) demonstrated this discipline — the canvas-to-merge cycle caught 3 sequential Director-tier ratification axis-failures (semantic → constructability → invariant-conformance) pre-merge via worker-tier substrate-canvas authoring. The 4-axis check IS the protective work that prevents drift from landing.

2. **PR-template-grep (second chain)**: PR reviewers (codex / cursor / openai-pro / claude) check stdout-artifact for ratchet-array length delta + classification cite (A/B/C per discrimination above) + Gap classification (a/b/c per Track A taxonomy doc). Missing classification cite = REQUEST_CHANGES (substantive). Reviewer-grep is the structural enforcement at review-time.

3. **Director-tier sanction (third chain)**: policy escape hatch for genuine class-C structurally-unavoidable additions per `project_no_r4_carves_directive` analogous discipline. Director-tier ratification required for any class-C add AND must additionally include **INVARIANTS.md P5 Dispatch-Discipline Mechanism (b) single checkable receipt** (deletion path / SG-0 census line shrink with before/after counts / explicit deferral naming a lane + concrete `ROADMAP.md` row citation). Director cites the structural-unblockable reason. **For pipeline-stage entries** (`emit.rs` / `lower.rs` / `infer.rs` / `parse.rs` and adjacent), additionally requires `SELF_HOSTING.md` §2.2 Step 1 L2.5 domain-model SET for the stage (parse/lower/infer/emit domain enumerations per §2.2; every cross-stage-boundary or lens-consumed type modeled in `std/` and `extdeps/`) landed-and-reviewed as **precondition**; template-relocation paper-shrink FORECLOSED per April PR #729 precedent (`a0f0b7837`) + 2026-05-14 cycle-4 + cycle-5 paper-shrink finding. Director-tier ratification alone is **NOT** sufficient absent the P5 receipt + (for pipeline-stage entries) the L2.5 domain-model-set precondition.

**Examples** (illustrative):

- *PR #3046 (swift-bee-15 cycle-2 NON_TEST retirement)*: cascade-substitution class B; 17 retirements + 13 finer-grained `.dag` test-fixture additions = net -4; PR body must explicitly enumerate which 3 hand-Rust files are retired + which 10 substitution entries are added per the named scope.
- *PR #3040 (warm-wren-479 generator-manifest substrate)*: substrate-prereq landing itself; not a ratchet PR but the substrate that enables future class A PendingFact-shape additions to materialize toward retirement.
- *Future PR adding new gate's hand-Rust consumer test*: must be class A (PendingFact-shape building toward generated consumer) OR class C (Director-tier-ratified hand-Rust survival with named reason).

**Sequencing prereq**: this §5.1 discipline applies post-`docs/audit/r3-pb0-non-test-retirement-class-taxonomy-2026-05-13.md` landing (Track A taxonomy under PR #3045); until taxonomy lands, the discrimination is Mgr-tier coordination per PM cross-message msg_de98e128 + msg_9806a1d4 (PendingFact-shape OK / hand-Rust-without-substrate PAUSE).

**Foreclosure clause**: this discipline does NOT impose a hard zero-add-per-PR rule (per operator Decision A "target 0 is met; work orthogonal" — net-direction-down per-PR is acceptable; substrate-prereq work during pending windows is legitimate; cascade-substitution is allowed with commentary). It IS a hard "no silent hand-Rust additions without retirement-substrate path" rule preventing the empirical +8/10-commits drift class from continuing.

**Cross-Mgr coordination handoff**: until §5.1 PR-template assertion is mechanically enforced (CI-tier check, future Cluster-M-Phase-3-style ratchet-direction-test work item TBD), §5.1 is Mgr-tier discipline coordinated via:
- zesty-boar-261 R3 Debt-Paydown Mgr surfaces ratchet-additions from sibling Mgrs to PM (per msg_48439a7f Cross-Mgr coordination clause)
- PM cross-messages still-moth-538 / warm-wolf-698 with class-A/B/C discrimination per Director msg_0b8f9283 framing
- Director-tier reinforce if any Mgr-tier escalation surfaces

### §5.2 Brief-dispatch authority-gate discipline

**Authority**: Phase 2.7 of Phase 2 corrective sweep per `docs/audit/r3-comprehensive-design-brief-implementation-audit-2026-05-14.md` §3 systemic-pattern findings + operator discipline 2026-05-14 ("no worker starts working without a design ready ... every test/file should clearly map to a design section that explains how/where it's going").

**Origin**: PR #3061 audit doc §3 identified 4 sub-patterns of systemic failure that produced the 5 authority chain breaks (PB-0 framework bypass + R2-Evaluator authority dispersal + Gap 9 canvas-stall + Cluster F sequencing-stall + Cluster M class-level-only design):
1. **Briefs lack mandatory design-authority citations** — worker briefs authored as task checklists without design-doc-tier authority embedded as preconditions
2. **Mgr lane ownership not synchronized across design / brief / implementation layers**
3. **Canvas ratification not synchronously gated with brief dispatch** — pre-authored briefs allowed against unratified canvases
4. **Audit findings reactive, not pre-dispatch blocking** — audit docs catch the pattern AFTER work merges, not before dispatch

§5.2 codifies the corrective: every worker / Mgr-tier brief dispatch MUST satisfy a 3-axis authority-gate check pre-dispatch.

**§5.2.1 — Mandatory pre-dispatch citations**

Every brief (Mgr-tier OR worker-tier) dispatched per any Phase / Mgr lane MUST include in its frontmatter or §0 a 3-axis citation block:

1. **Design-doc authority cite** (NAMED design doc + section + named lane/step) — **AND per-entry citation when brief scope is enumerable** (per operator discipline 2026-05-14: *"every test/file should clearly map to a design section that explains how/where it's going"*):
   - **Single-entry briefs** (one file/test/scope): one design-authority citation suffices.
     - Example for PB-X retirement work: *"design authority: `docs/design-pure-bootstrap-zero.md` (PB-6 emit lane) + `src/v3/SELF_HOSTING.md` §2.2 Step 1 L2.5 model review prereq"*
     - Example for Gap 13 T-Ground sub-lane: *"design authority: `docs/r2-closure-ledger.md:108-122` (T-Ground sub-lane registration) + `docs/design-emission-model.md` §3 (no-engine discipline)"*
   - **Multi-entry briefs** (class-level / cycle / sweep covering >1 file/test/scope): MUST cite **per-entry design** OR a **static pre-dispatch enumeration artifact** (e.g., per-test inventory doc, per-file taxonomy doc) that maps each entry to its design section. Lane-level citation alone is insufficient when scope is enumerable; that's exactly the Cluster M class-level-only failure mode the audit doc §3 systemic-pattern finding identified (codex BLOCKING #11738 PR #3071 follow-up). Examples:
     - **Wrong** (lane-level only): *"design authority: `docs/audit/r3-cluster-m-sequencing-plan-2026-05-09.md`"* (one citation covering 20+ unrelated tests)
     - **Right** (per-entry mapping): brief includes a table mapping each test ID to its design section, OR cites a static pre-dispatch inventory doc that has the per-entry mapping (e.g., *"per-test inventory: `docs/audit/r3-cluster-m-per-test-inventory-YYYY-MM-DD.md` §X (covering tests T1-T20)"*)
     - **Right** (single-entry breakdown): if the brief covers N entries, each entry's design section is named in the brief frontmatter
   - **Vague "see design docs" or "per `docs/`" does NOT satisfy**. Cite the SPECIFIC section + named lane (single-entry) OR specific section per entry / specific inventory artifact (multi-entry).

2. **Mgr lane ownership confirmation** (NAMED Mgr session at time of dispatch):
   - Example: *"owner Mgr: warm-wolf-698 (R3 Substrate Mgr expanded scope per α-ratification 2026-05-14)"*
   - For new-Mgr-spawn cycles, the dispatch must reference the operator-ratified ownership decision (e.g., §4 Item N ratification)
   - Cross-reference at least one of: `dashboard-ops graph <parent>` live state, prior ratification message ID, or close-plan §4 ratification

3. **Canvas-ratification status** (for substrate-tier work — explicit ratification or N/A):
   - Substrate-shape worker briefs MUST cite the ratified canvas authority (e.g., *"substrate ratified per Director msg_X 2026-05-Y; canvas at `docs/design-Z.md`"*) — NOT "canvas authoring in flight" or "Mgr canvas authoring pre-dispatch"
   - Worker briefs MAY pre-author against draft canvases per `feedback_pre_authored_brief_queue` BUT MUST NOT dispatch until canvas-ratification cite is present
   - For consumer-tier work (no new substrate; consuming existing types/lenses), cite "N/A — no new substrate authoring"

**§5.2.2 — Reviewer-grep enforcement at PR-template tier**

PR reviewers (codex / cursor / openai-pro / claude — typically the 4 dashboard-tier reviewers) check PR body / brief frontmatter for the 3-axis citation block. Missing or vague citations = `REQUEST_CHANGES` (substantive). PR-template-grep is the structural enforcement at review-time, analogous to §5.1's ratchet-direction enforcement.

**§5.2.3 — Director-tier sanction for missing citation**

If a brief dispatches without the 3-axis citation block AND a reviewer flags it, Director-tier sanction = pause-dispatch + require brief amendment + re-dispatch with citations. This is NOT a build-break (briefs are coordination artifacts not code); the sanction shape is dispatch-pause + visibility-surface to operator if the pattern repeats.

**§5.2.4 — Exemplar substrate**

`docs/audit/r3-phase2-corrective-sweep-dispatch-plan-2026-05-14.md` is the exemplar artifact applying this discipline to the Phase 2 corrective sweep itself. The dispatch plan §1 task table covers the 3-axis citation block via these columns: **Design authority** column maps to axis 1 (per-task design-doc citation); **Mgr lane** column maps to axis 2 (named owner Mgr / PM-direct); **Canvas-ratification status** column maps to axis 3 (substrate-tier ratified canvas authority / N/A for consumer-tier). §8 dispatch commands per task reference the task IDs; §9 design-coverage gap audit surfaces per-entry coverage status that the brief-dispatch authority-gate would catch.

By applying the discipline to the corrective sweep itself, the dispatch plan validates the discipline shape BEFORE codifying it as a permanent §5 process discipline addition. **Validation receipt** (per codex BLOCKING #11738 PR #3071 follow-up 2026-05-14): the §1 task table's three citation columns (Design authority + Mgr lane + Canvas-ratification status) correspond to the §5.2.1 3-axis citation block; the exemplar claim is grep-verifiable against the dispatch plan §1 header at `docs/audit/r3-phase2-corrective-sweep-dispatch-plan-2026-05-14.md`.

**§5.2.1 multi-entry discipline applied to the exemplar itself** (per codex BLOCKING re-review PR #3072 2026-05-14): 7 of 8 Phase 2 rows (2.0-2.7) cover single-entry scope (one close-plan section / one canvas / one §1.8 row) — single-entry citation suffices. Phase 2.8 covers multi-entry scope (~122 TEST entries; T-γ-subset post-framework). Per §5.2.1 multi-entry requirement: per-entry mapping OR static pre-dispatch inventory artifact citation. **Phase 2.8's deliverable IS the §5.2.1-compliant inventory artifact** — dispatch cannot fire until the artifact lands. The dispatch plan's Phase 2.8 status field shows "HOLD pending operator-ratified test-deletion framework," demonstrating §5.2 enforcement firing correctly: dispatch withheld because §5.2.1 multi-entry requirement isn't yet satisfied. This is the discipline working as designed at the very boundary the rule was added to enforce, NOT a §5.2.1 dilution.

**§5.2.5 — Foreclosure clause** (analogous to §5.1)

§5.2 does NOT impose a hard "all briefs must have all citations perfect" rule. Briefs can be DRAFTED without all 3 axes (per `feedback_pre_authored_brief_queue` queue discipline); the gate fires at DISPATCH time, not authoring time. Draft briefs sitting in queue may be incomplete; the §5.2 discipline applies when the brief is dispatched as a directive.

**§5.2.6 — Cross-Mgr coordination handoff**

Same coordination shape as §5.1:
- zesty-boar-261 R3 Debt-Paydown Mgr surfaces dispatch-discipline violations from sibling Mgrs to PM
- PM cross-messages owning Mgr with the 3-axis citation correction
- Director-tier reinforce if Mgr-tier escalation surfaces

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
- [x] **Gap 13 (R2-Grounding T-Ground sub-lane residuals / no-coercion-engine architectural separation — post-§4-ratification adversarial finding)** — surfaced by operator adversarial probe 2026-05-13; default IN-R3 per `project_no_r4_carves_directive` as design-emission-model.md no-engine discipline + PB-0 architectural completion. Director-tier R2-Grounding audit complete 2026-05-13 (msg_8ae92369): 11 T-Ground sub-lanes (NOT 5 as PM originally cited) — 1 GREEN + 7 in-flight + 3 not-started; emit_model.dag SCAFFOLD dissolution + coercion.dag schema dissolution + emit.rs retirement downstream. **Operator §4 sub-item 6 RATIFIED (α) re-spawn 2026-05-13** (bundled with sub-item 5; see below).
- [x] **§4 sub-item 6 ratification** (R3 Grounding Mgr dispatch — subtree-shape decision; surfaced per Director audit msg_8ae92369 2026-05-13): **(α) re-spawn R3 Grounding Mgr as 5th R3 Mgr lane RATIFIED** by operator briansrls 2026-05-13 (bundled-5-asks PM-routing). Scope-discrimination canvas as Mgr-tier first-deliverable per Gap 13 sub-program step 3 (discriminate Grounding-owned scope vs Substrate-Mgr-already-absorbed scope per Director (d) audit caveat). Director (zesty-bear-812) executes Mgr re-spawn post `--shape` flag landing in `dashboard-ops work-items create`.
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
