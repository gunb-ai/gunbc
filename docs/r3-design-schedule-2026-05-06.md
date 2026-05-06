# R3 Design Schedule — 2026-05-06

**Status**: live PM-tier dispatch matrix per Brian directive 2026-05-06 (chat): *"can we schedule all the design now?"*

**Authority hierarchy**:
- [`docs/r3-structure.md`](r3-structure.md) — architectural archive (lane defs / 95-gate §"Acceptance" / Mgr structure)
- [`docs/r3-program-plan.md`](r3-program-plan.md) — forward-looking dependency graph + escalation register + §1.8 canonical 95-gate ledger
- [`docs/audit/r3-debt-sweep-2026-05-06.md`](audit/r3-debt-sweep-2026-05-06.md) — bridge-inventory framework
- **THIS DOC** — per-Mgr design queue + dispatch matrix; sequencing + worker-pinning where determinable; cross-lane coordination triggers

**Purpose**: schedule per-lane design work in parallel with Brian/Director scope-calibration decisions. Mgrs do not wait for all decisions to resolve — design dispatches in flight as escalations resolve.

**Discipline**: each design item names — owner Mgr / cross-lane prerequisite / scope / dispatch trigger / closure predicate (which §1.8 ledger gate(s) it advances). Status updates flow through §1.8 ledger Status column.

---

## §1. Substrate Mgr (quick-crab-830, gunbc#1739)

**Lane scope** (per `r3-structure.md` §"Manager structure"): T-Numeric-Construction + T-Anthropic-Wire + T-CostLens-Composition + T-E-P-Producer-Broadening + T-Lens-Behavioral-Parity (cross-program) + T-Lens-Application-Surface (cross-program) + T-Workflow-As-Data + 2 Substrate-owned bridges.

**Workers idle (per Substrate Mgr canvas D)**: loyal-wolf-828 + valiant-ant-72 fully idle; proud-lynx-311 / smart-ram-167 / valiant-ibex-312 holding on #1794-cascade.

### S1 — Q-Class-2-Chain-Break gap-test surface (ENGAGE-NOW per Director disposition)

**Scope**: surface option-(a) candidate function-valued data gap-test that traces to GREEN without requiring T-LBP COMPLETE. Per Director disposition + Refinement 3 chain-rule.

**Dispatch trigger**: now.
**Closure predicate**: §1.8 gate #61 `substrate_gap_function_valued_data_closed` — currently RED-blocked on Q-LBP-R3-Closeability; option-(a) re-pick gap-test traces to GREEN.
**Cross-lane**: if option-(a) infeasible, escalates to (b) Director scope-calibration on LBP.

### S2 — T-LBP scope-calibration canvas (ENGAGE-NOW per Director disposition)

**Scope**: Director needs Substrate canvas detail before engaging Q-Lens-Behavioral-Parity-R3-Closeability. Surface specific blockers per lens (4 lenses × 4 sub-slices each); name option (a)/(b)/(c) shape per Substrate Mgr E3 RED.

**Dispatch trigger**: now.
**Closure predicate**: unblocks Q-LBP-R3-Closeability decision; cascades to §1.8 gates #79-#83 (T-Lens-Behavioral-Parity 5 gates) + #61 (Class 2 chain-break) + #88-#95 (T-Lens-Application-Surface) + #54 (timing_lens_carrier_landed) + #57-#59 (T-Lens-Self-Application).

### S3 — `MachineConstraint<C>` carrier design (Brian directive 2026-05-06)

**Scope**: design substrate carriers for **algebra × machine-constraints interaction modeling** per Brian directive: concrete types like `i64` emerge as products of independent constraint axes (algebra `Int = AbelianGroup<Nat>` × machine `MachineWidth<64>`), NOT primary entities.

Carriers needed: `MachineConstraint<C>` + `MachineWidth<bits>` + interaction-lookup substrate. May require additional axes (`RegisterClass<R>`, `EndianMode<E>`, alignment) per use cases.

**Dispatch trigger**: now (foundational; gates Class 1 closure).
**Closure predicate**: §1.8 gate #60 `substrate_gap_parser_grammar_closed` (Class 1 5-criteria Pass: substrate carriers + parser handles generic interaction syntax + ≥3 algebra×constraint pairs emit to target primitives + target primitives NOT primary substrate entities).
**Cross-lane**: T-Numeric-Construction provides algebra side (in flight); MachineConstraint<C> is new substrate.

### S4 — Workflow* family carriers (Class 4)

**Scope**: 5 carriers for CI-workflow-as-`.dag`-data per §4.4:
- `WorkflowTrigger` (trigger-event sum: Push | PullRequest | Cron<Schedule> | Manual<Inputs>)
- `WorkflowStep` (run command + dependencies + outputs) + `WorkflowMatrix<Axes>` (parameter expansion)
- `WorkflowSecret<Name>` (provider-typed, opaque-at-rest, scoped-by-step)
- `RunnerResource<C>` (compute class, OS, hardware)
- `Workflow<Trigger, Steps, Resources>` composing carrier

**Dispatch trigger**: post-T-Lens-Behavioral-Parity COMPLETE (per `r3-structure.md` §"Dependency on R2"; lens consumption needs lenses COMPLETE).
**Closure predicate**: §1.8 gates #53 (workflow_substrate_carriers_landed), #54 (timing_lens_carrier_landed), #55 (shared_external_attachment_pattern_documented), #56 (ci_workflow_modeled_as_dag), #62 (substrate_gap_file_ingestion_closed), #63 (substrate_gap_workflow_scheduling_closed).

### S5 — Variant-aware projection metadata carrier

**Scope**: typed REST response projection carrier for coproduct response bodies. Per Substrate canvas C1 + Anthropic #1702 + Grounding G5.

**Dispatch trigger**: gates #1702 re-dispatch + 3 follow-up paydown PRs (Anthropic / OpenAI ChatCompletion / OpenAI Responses).
**Closure predicate**: §1.8 gates #29-#30 (T-Anthropic-Wire 2 gates) + Q-Anthropic-Variant-Aware closure-scope ratification (Director — carrier-only vs 3 paydowns; default = all 3 per Brian no-post-R3-deferral).

### S6 — `EmissionPathProjection` carrier (L6)

**Scope**: per `docs/briefs/r3-substrate-l6-per-row-projection-routing-decision.md` Option 2 — `EmissionPathProjection` keyed by `MethodTemplateContractKey`, `List<EmissionCell>`, target on the key, carrier empty first.

**Dispatch trigger**: now (non-Evaluator-gated; can fire parallel to T-E-P-Producer-Broadening per B4).
**Closure predicate**: gates Grounding L6 row population (post-merge); cross-lane handoff to Grounding Mgr per #1745.

### S7 — PR-F (BoundDeclaration consumer + Rust ReferenceModel<T>)

**Scope**: substrate carrier landing per Substrate canvas B3 + Grounding G1.

**Dispatch trigger**: post-#1782 merge (Substrate Mgr 2026-05-06).
**Worker pin** (per Q-PR-F bandwidth-aware routing + Substrate Mgr partition 2026-05-06): **loyal-wolf-828**; brief shape = T-E-P-Producer-Broadening adjacent precedent (Substrate Mgr authors brief; valiant-ant-72 reserved for S3 `MachineConstraint<C>` implementation post-design).
**Closure predicate**: unblocks Grounding T-Ground-Rust Phase 1 (`u128` / `isize` / `usize` / walker arms / pilot mirror).

### S8 — `ApproximateField<F>` Float migration (T-NumericConstruction)

**Scope**: Float32/Float64 migration from `Field<Word*>` to `ApproximateField<F>` per Substrate canvas B2 + Grounding G2; plus Real/base-carrier convention for `F`.

**Dispatch trigger**: **parallel with S3** (per Substrate Mgr partition response 2026-05-06 at gunbc#846 #issuecomment-4385074769 — `MachineConstraint<C>` and `ApproximateField<F>` are independent axes: machine width vs algebra approximation). Both Mgr-tier design now; cross-reference at brief-landing.
**Closure predicate**: unblocks Grounding Rust float rows + Class 1 Real<N> demonstration; advances §1.8 gates #17-#24 (T-Numeric-Construction 8 gates).

### S9 — T-Numeric-Construction Mgr-tier brief authoring (B2 ratified)

**Scope**: comprehensive T-Numeric-Construction worker brief covering 13 in-scope types (Int / UInt / Float direct + 10 Int-inherited refinements: Char / EpochMs / Duration / Milliseconds / Seconds / RetryCount / HttpStatus / Port / PositiveInt / NonNegativeInt) + Slice 2 Nat-alignment migration.

**Dispatch trigger**: now (parallel-author authorized per B2).
**Closure predicate**: §1.8 gates #17-#24.

### S10 — T-E-P-Producer-Broadening dispatch (foundational)

**Scope**: brief drafted in PR #1782 (`docs/briefs/r3-t-e-p-producer-broadening-worker.md`); broaden per-call DescentEvidence / CallPattern / SubValueRelation producer coverage.

**Dispatch trigger**: post-#1782 merge (per B3 RATIFIED).
**Worker pin** (per Substrate Mgr 2026-05-06): **quick-koi-190** (currently on #1799 termination-contract; T-E-P consumes descent-evidence, natural follow-on).
**Closure predicate**: §1.8 gates #76-#78 (3 T-E-P gates) + foundational for cascade T-LBP → T-LAS → T-WAD → T-LSA.

### S11 — Slice C of #1795 follow-up (B1 ratified)

**Scope**: Slice C residual covers ~10 files (per smart-ram-167 enumeration); paired prose+regen edits per #1795 path-(a) precedent.

**Dispatch trigger**: post-#1795 (Slice A) + #1801 (Slice B) merge per B1 RATIFIED prose+regen bundling (Substrate Mgr 2026-05-06 — cascade-clearance gated).
**Worker pin** (per Substrate Mgr 2026-05-06): **smart-ram-167** (Slice B precedent owner; pattern-familiar).
**Closure predicate**: ROADMAP :425 row PARTIAL → Retired (per Substrate canvas C3); Q-Slice-C-Retirement-Receipt resolution.

### S12 — F2 + F8 doc-sharpening PR (B6 ratified)

**Scope**: F2 (`.v3` filename-suffix grammar dispatch) + F8 (bootstrap load-order/exclusion authority); single bundled Mgr-tier doc-sharpening PR per Substrate canvas B6.

**Dispatch trigger**: post-:425 close (sequenced after S11).
**Closure predicate**: ROADMAP F2/F8 rows retire.

### Substrate demonstration gates (per §1.6 minimum bar)

Per Substrate Mgr partition response 2026-05-06: **demonstration gate scope folded into parent worker brief Acceptance bullets**, NOT separate dispatches. Each gate becomes an Acceptance bullet on the parent lane's worker brief.

| Gate (§1.8 row) | Parent brief | Demonstration scope (Acceptance bullet) |
|---|---|---|
| #67 `numeric_construction_demonstration` | T-Numeric-Construction worker brief (S9) | end-to-end `Int<32>` + `Real<64>` round-trip |
| #68 `anthropic_wire_demonstration` | T-Anthropic-Wire / variant-aware brief (S5 follow-on) | full request/response cycle vs deterministic mock |
| #70 `cost_lens_demonstration` | T-CostLens-Composition brief | ≥2 algebra-instances + ≥1 recursive call + observable cost-bound |
| #72 `e_p_producer_demonstration` | T-E-P-Producer-Broadening worker brief (S10) | call-site produces full descent evidence at runtime |
| #73 `lens_behavioral_parity_demonstration` | T-LBP scope-calibrated brief (post-S2) | each lens demonstrates + matches frozen v2-oracle cementing-test snapshot (per openai-pro F2 — frozen, NOT live v2) |

---

## §2. Verification Mgr (cool-owl-579, gunbc#1740)

**Lane scope**: T-V-L4-L7-Direct + T-V-L5-Corpus + T-Free-Consequences-Demonstration + T-Tests-As-Data-Completeness + T-Lens-Self-Application + cross-program portion of T-Lens-Behavioral-Parity + cross-program portion of T-Lens-Application-Surface + `bridge_retirement_ledger_zero` audit gate.

**Worker partition** (per Verification Mgr partition response 2026-05-06 at gunbc#846 #issuecomment-4385074816):

| Track | Worker | Items | Status |
|---|---|---|---|
| **A — executable / ledger** | bold-crane-790 (#1748) | V1 (TC1 hold pending Q-PAFS + EVAL-3) + V6 (active) | V6 ACTIVE; V1 standby |
| **B — corpus / demos / data** | cool-heron-521 (#1766) | V2 + V4 + V5 (all post-R2-Evaluator-gated) | Prep now (design + skeleton); closure waits triggers |
| **C — Mgr-reserved / cross-lane** | cool-owl-579 (this lane) | V3 (post-cascade) + V7 (hold) | V3 sequenced per §8; V7 design-first only |

### V1 — Pattern-A executable cluster (TC1 first; pending Director countersignature on Q-PAFS)

**Scope**: 5 NEW Pattern-A executable gates need consumer infrastructure:
- TC1 (`tc1_eta_equivalence_executable`): static representative via E6-G1.a (PM-default; Q-Pattern-A-First-Slice-Subscope) — runtime prereq from Evaluator EVAL-3
- TC2 (`tc2_church_rosser_executable`): second strategy/input order + strategy-keyed report
- TC3 (`tc3_pattern_a_second_mover_executable`): Descent execution proof (E5) + eval-step producer
- RustDagIsomorphism (`rust_dag_isomorphism_executable`): shape-report producers

**Dispatch trigger**: TC1 first slice — pending Director countersignature on Q-PAFS + EVAL-3 (E6-G1.a static lens fold landing).
**Closure predicate**: §1.8 gates #11-#14 (4 Pattern-A executable) + cascades to BridgeLedgerZero net-shrink (gate #36).

### V2 — L4 corpus + L7 exhaustive coverage

**Scope**: l4_emit_eval_match (gate #9) + l7_algebraic_laws_witnessed (gate #10) — currently DECLARED (skeleton/staged). Closure bar = exhaustive per-(algebra, inhabitant, law) coverage per `r3-structure.md` 2026-05-02 fold-in.

**Dispatch trigger**: post-R2-Evaluator landed + Shape A grounding for L4 corpus.
**Closure predicate**: gates #9-#10 reach CONSUMER_LANDED + PASSING.

### V3 — T-Lens-Self-Application stronger demo (Y-2 RATIFIED)

**Scope**: per Research PM Y-2 — strengthen `lens_self_application_demonstrated` to tie lens output to emission gate (CI workflow → TestClaim → fails → gunbc emission refuses). Demonstrates 4 gates simultaneously + WEDGE-CORE-CLAIM Part B opportunistically.

**Dispatch trigger**: post-T-Workflow-As-Data + T-Lens-Application-Surface (per cascade).
**Closure predicate**: §1.8 gates #57-#59 (3 T-Lens-Self-Application gates) + 4-fold demo overlap (Self-App + Workflow-As-Data + Tests-As-Data + integration_testgen).

### V4 — T-Tests-As-Data-Completeness lane work

**Scope**: every Rust test ports to `.dag` TestClaim or generated target-language test code; ForAll/Exists quantifier substrate; ProgramGenerator carrier; cementing test discipline.

**Dispatch trigger**: post-R2-Evaluator (test-execution runtime needed).
**Closure predicate**: §1.8 gates #84-#87 (4 T-Tests-As-Data gates) + #74 (tests_as_data_demonstration).

### V5 — T-Free-Consequences-Demonstration (10 gates)

**Scope**: 10 demo gates — auto-parallelism × 3 + auto-loop-parallelism × 3 + auto-memoization × 2 + cross-target-optimization × 2.

**Dispatch trigger**: post-R2-Evaluator + T-CostLens-Composition (cost-related claims).
**Closure predicate**: §1.8 gates #43-#52.

### V6 — `bridge_retirement_ledger_zero` audit gate (ACTIVE)

**Scope**: unified ledger reports 0 named identity bridges remaining. 5 sub-bridges retire (2 Substrate-owned + 3 PB-owned).

**Dispatch trigger**: ongoing (audit cadence). **Worker** (per Verification Mgr 2026-05-06): bold-crane-790 — ACTIVE.
**Closure predicate**: gate #36 reaches PASSING.

### V7 — ValueBody isomorphism gate design (Q-ValueBody-Isomorphism)

**Scope**: per Verification canvas V6 + `feedback_isomorphism_or_generation_for_mirrors` — Rust↔.dag mirror conformance gate. Verification test-only pressure; Substrate Mgr authority on mirror.

**Dispatch trigger**: pending Director scope-decision (in-R3 vs post-R3 carry).
**Closure predicate**: NEW gate `value_body_isomorphism_gate_active` if added; design first.

---

## §3. PB Mgr (neat-bear-351, gunbc#1742)

**Lane scope**: T-LensProducer-Retirement + T-FixedPoint + T-Tier3-Dissolution + T-V2-Retirement + 3 PB-owned bridges.

**Worker partition** (per PB Mgr partition response 2026-05-06 at gunbc#846 #issuecomment-4385075315):

| Schedule row | Worker / inbox | Dispatch posture |
|---|---|---|
| **P1** T-LensProducer-Retirement | sleek-eagle-514 (#1768) — PR #1805 path-1 + sub-briefs gunbc#828 (Sub1/2/3) | Next executable slice = `lens_apply` retirement design/audit receipts; not stalled on Director-thread unless §3 names HOLD |
| **P1** parallel doc spine | zesty-ram-316 (#1769) — PR #1806 regen_lens audit + Sub2/Sub3 brief threads | post-R2-Evaluator + PB-1 bin-shim pattern; parallel doc-spine on P1 |
| **P4** bridge appendix / cross-links | warm-ant-877 (#1770) — grep / ledger hygiene against `bridge_ledger.dag` / `r3_bridge_retirement_ledger_zero.dag` / `verification.dag` | Lockstep with P1/P2 sequencing; `include_str!` post-T-FixedPoint per Q-Bridge-Retirement-Sequencing-Authority |
| **P5** F2 + F8 doc-sharpening | PB Mgr coordinates consumer-side with Substrate S12 owner | No duplicate PR; PB-named co-author OR comment-only on Substrate's PR (PM ratify if explicit co-author shape needed) |
| **P2** T-FixedPoint | HOLD until P1 + SG-0 zero per F1 | No worker spawn that pretends LP is done; TC3 text stays proposal-side |
| **P3** T-V2-Retirement | HOLD on broad ~79 .rs sweep until P2 + LP + T-Numeric-Construction `Int<N>` clear | Q-V2-Retirement-Boundary-Matrix split visible (PB vs Grounding vs Debt-Paydown) |

**§2.2 sequencing authority — HARD DAG (PM ratification 2026-05-06)**: per PB Mgr ask + r3-structure.md §"Lane structure" → T-FixedPoint row "R2-close dependency: SG-0 zero from T-LensProducer-Retirement" — sequencing is **hard DAG, not staffing parallelism**. T-FixedPoint cannot complete until T-LP-Retirement completes (SG-0 zero is structural precondition; not just resource sequencing). Plan §2.2 sequence is canonical authority. Per r3-structure.md T-FixedPoint row: "SG-0 zero from T-LensProducer-Retirement" is named explicit dependency.

### P1 — T-LensProducer-Retirement (3 hand-Rust files retired)

**Scope**: lens_apply.rs + lens_testgen.rs + regen_lens.rs retired via PB-Runtime + PB-1 patterns.

**Dispatch trigger**: post-R2-Evaluator + PB-1 generated bin-shim pattern.
**Closure predicate**: §1.8 gates #5-#8 (3 retirement gates + sg0_non_test_zero) + cascade to T-FixedPoint + Class 5 (#64).

### P2 — T-FixedPoint completion

**Scope**: `compiler.dag` self-compile fixed-point; bit-identical stage0 + emitted artifacts.

**Dispatch trigger**: post-T-LensProducer-Retirement → SG-0 zero (corrected sequencing per PB Mgr F1).
**Closure predicate**: §1.8 gate #16 reaches CONSUMER_LANDED at R3 horizon (already CONSUMER_LANDED at R1).

### P3 — T-V2-Retirement (post-FP+LP cascade)

**Scope**: ~79 .rs + ~32 .dag files in src/v2/ retired; workspace member removed; bootstrap routes through PB-Runtime trampoline only.

**Dispatch trigger**: post-T-FixedPoint + T-LensProducer-Retirement + T-Numeric-Construction `Int<N>` (for parser path).
**Closure predicate**: §1.8 gates #41-#42 + #71 (v3_self_host_demonstration).
**Cross-lane**: per Q-V2-Retirement-Boundary-Matrix — PB owns v2 directory + test-consumer + PB-Runtime trampoline; Grounding owns emit-shim + Coercion-Fold; Debt-Paydown owns drift items.

### P4 — 3 PB-owned bridges

**Scope**:
- canonical lens-name dispatch retires alongside T-LensProducer-Retirement
- include_str! side channels retire post-T-FixedPoint (per Q-Bridge-Retirement-Sequencing-Authority)
- patch_lower_helpers_* residual post-Tier-2

**Dispatch trigger**: per per-bridge prerequisite chain.
**Closure predicate**: contributes to §1.8 gate #36 (bridge_retirement_ledger_zero) + #69 (bridge_retirement_demonstration).

### P5 — F2 + F8 doc-sharpening PR (per S12 + B6)

**Cross-lane**: Substrate-led; PB consumer-side coordination.

### PB demonstration gates

| Gate (§1.8 row) | Demonstration scope |
|---|---|
| #65 `tier3_dissolution_demonstration_executes` | Tier3-mirror-consumer `.dag` runs end-to-end via Evaluator |
| #66 `lens_producer_retirement_executable_witness` | DEFERRED to Row-4 receipts per PB Mgr F3 |
| #69 `bridge_retirement_demonstration` | typed-identity-surface in production code |
| #71 `v3_self_host_demonstration` | bootstrap PB-Runtime trampoline runs end-to-end |

---

## §4. Evaluator Mgr (merry-gull-128, gunbc#1743)

**Lane scope**: R3 Evaluator continuation work — runtime prereqs for Pattern A predicates + lens runtime execution + constructor/field-call execution per E6 G0a/G0b/G0c/G0d work.

### E1 — E6-G0d constructor runtime execution (RATIFIED Mgr executes)

**Scope**: authorize evaluator implementation slice in `src/v3/compiler/src/lib.rs` for non-Arrow constructor `Callable`, using existing `Value::RecordValue` / `Value::VariantValue`. Brief at `docs/briefs/r3-pr-e6-g0d-constructor-runtime-execution-worker.md` (#1784 ready).

**Dispatch trigger**: now (Evaluator Mgr executes per Q-EVAL-G0d-Dispatch RATIFIED).
**Closure predicate**: unblocks Pop A constructor behavior + honest `Witness`/`DimensionReport` construction + static lens fold.

### E2 — E5 Descent termination contract consumer (RATIFIED post-Substrate carrier)

**Scope**: `eval_loop` consumes `descent_execution_proof` token from substrate carrier authored via quick-koi-190 (already authorized through quick-crab).

**Dispatch trigger**: post-Substrate carrier landing.
**Closure predicate**: unblocks TC3 (gate #13) + any lens fold traversing Descent loops.

### E3 — E6-G1.a static lens fold (Pattern A first slice — pending Director countersignature)

**Scope**: single static top-level `data ... : Lens<C>` representative under G1.a. Per Q-Pattern-A-First-Slice-Subscope (PM-default TC1-static-rep first).

**Dispatch trigger**: pending Director countersignature on Q-PAFS / Q-EVAL-Lens-Fold-First-Slice.
**Closure predicate**: §1.8 gate #11 (tc1_eta_equivalence_executable).

### E4 — E6-G1.b generic dispatch (post-G1.a)

**Scope**: generic `fold_lens<C>` for non-static lens applications; cascades from G1.a + X1.b S1/S3.

**Dispatch trigger**: post-G1.a + post-Substrate X1.b S1/S3 carrier.
**Closure predicate**: cascades to TC1-generic + TC2 + TC3 (gates #11-#13).

### E5 — X1.b S1 TransformDispatch coordination

**Scope**: cross-lane status update on E6-G0c → unblocks Substrate X1.b dispatch.

**Dispatch trigger**: cross-lane coord with Substrate Mgr.
**Closure predicate**: Q-X1b-S1-Dispatch-Coord resolution.

---

## §5. Grounding Mgr (bold-ferret-748, gunbc#1745)

**Lane scope**: L6 CrossTarget-Meta + T-Ground-Rust + Coercion-Fold retirement + emit shim consumption + F10 cleanup + Anthropic re-dispatch.

### G1 — L6 row population (post-EmissionPathProjection)

**Scope**: per `docs/briefs/r3-l6-emission-path-projection-substrate-worker.md` consumer-side; populate L6 projection rows + convert `coverage.rs`.

**Dispatch trigger**: post-Substrate S6 (EmissionPathProjection carrier landing).
**Closure predicate**: L6 cross-product fold gate.

### G2 — T-Ground-Rust full coverage (post-PR-F + post-Float)

**Scope**: Phase 1 (`u128` / `isize` / `usize` / walker arms / pilot mirror) post-PR-F; Float rows post-ApproximateField<F>.

**Dispatch trigger**: post-Substrate S7 (PR-F) + S8 (Float).
**Closure predicate**: full Rust primitive grounding + L5 readiness.

### G3 — Coercion-Fold scratch retirement (post-LanguageSpec projection)

**Scope**: `LanguageSpecProjection::ScratchIntExamples` + `TargetInhabitance` retirement in `src/v3/grounding_coercion_fold`.

**Dispatch trigger**: post-Substrate/LanguageSpec projection executable.
**Closure predicate**: Q-Coercion-Fold-Scratch closure.

### G4 — F10 install_hint cleanup

**Scope**: `dsl/extdeps/tools.dag` `install_hint` join semantics fix (per Substrate canvas G8).

**Dispatch trigger**: now (small cleanup; bundle into next Grounding signal PR).
**Closure predicate**: M5 cleanup close.

### G5 — Anthropic #1702 re-dispatch (post-variant-aware projection)

**Scope**: re-dispatch Anthropic Messages 200 content-block work post-Substrate variant-aware projection metadata carrier.

**Dispatch trigger**: post-Substrate S5 + Q-Anthropic-Variant-Aware closure-scope ratification.
**Closure predicate**: §1.8 gates #29-#30 (T-Anthropic-Wire) + #68 demonstration.

---

## §6. Debt-Paydown Mgr (quiet-otter-416, gunbc#1744)

**Lane scope**: ROADMAP debt-row retirement + velocity-tripwire enforcement + closure-receipt cadence + per-PR discipline rule.

**Partition** (per Debt-Paydown Mgr partition response 2026-05-06 at gunbc#846 #issuecomment-4385074935):

| Item | Status | Next step |
|---|---|---|
| **DP1** Q-Drift-Reconcile single PR | DISPATCH-NOW | Single Debt-Paydown worker thread; scope = one reconciliation PR + §1.8 row retirements feeding `r3_debt_paydown_zero_remaining` |
| **DP2** SG-0 PR-window CI gate | **IN-FLIGHT at PR #1807** | scripts/check-pr-sg0-net-shrink-discipline.sh + workflow + template + ROADMAP; closure target = §1.8 gate #75 |
| **DP3** Velocity tripwire | CONTINUOUS | Standing cadence slice (P5 ratio surfacing); recurring Mgr report + Director queue pings on threshold trips; no single landed event |
| **DP4** Closure-receipt cadence | CONTINUOUS | Lane ops / receipts discipline; feeds `r3_debt_paydown_zero_remaining` Pass surface |
| **DP5** #1566 rollup hygiene | HOLD pending DRAFT close | Deferred worker; no idle burn until trigger clears |

No §6 items currently Director-blocked; §7 CP items stay out-of-lane unless they become prerequisites.

### DP1 — Q-Drift-Reconcile single PR

**Scope**: 3 drift items single-PR reconciliation:
- declaration_by_name ROADMAP↔ledger drift (ROADMAP says retired by #1638; ledger marks Open)
- #1499 transitional fence ledger-row gap
- CollectionOps / StringOps / MapOps stale ledger refresh

**Dispatch trigger**: now (RATIFIED-by-default; single Debt-Paydown worker).
**Closure predicate**: 3 ledger rows retire; advances `r3_debt_paydown_zero_remaining` predicate.

### DP2 — SG-0 PR-window net-shrink CI gate landing

**Scope**: integrate `scripts/check-pr-sg0-net-shrink-discipline.sh` into CI workflow; self-test passes.

**Dispatch trigger**: now (RATIFIED-by-default per §1 closure-gate set).
**Closure predicate**: §1.8 gate #75 (pr_anticipation_discipline_ci_active).

### DP3 — Velocity tripwire reporting cadence

**Scope**: per `INVARIANTS.md §P5` — surface introduction:dissolution PR ratio readings to Director on cadence.

**Dispatch trigger**: now (continuous reporting).
**Closure predicate**: ongoing operational; not a single closure event.

### DP4 — Closure-receipt cadence

**Scope**: per-tracked-debt-row receipt before R3 close; aggregation feeds `r3_debt_paydown_zero_remaining` predicate.

**Dispatch trigger**: continuous.
**Closure predicate**: aggregation in `r3_debt_paydown_zero_remaining` Pass surface 2.

### DP5 — #1566 rollup hygiene (DRAFT)

**Scope**: per Debt-Paydown Mgr canvas — DRAFT rollup hygiene.

**Dispatch trigger**: pending DRAFT close.
**Closure predicate**: ledger-row retirement.

---

## §7. Cross-program / Director-tier decisions

### CP1 — Q-LBP-R3-Closeability scope-calibration (Brian/Director)

**Scope**: T-Lens-Behavioral-Parity scope decision (a accept / b reframe to 1-2 lenses for R3 / c carve to R4 with substrate-gap routing).

**Dispatch trigger**: post-Substrate S2 (LBP scope-calibration canvas).
**Closure predicate**: cascades to Q-Class-2-Chain-Break + Class 2 closure path + critical-path timeline (Research PM R-2 Q-Timeline-Risk-Alternates).

### CP2 — Q-Tier4-Inclusion (Brian directive)

**Scope**: Tier 4 in-scope vs declared-out per ctrl#1608 §6d. Brian directional framing 2026-05-06: "bring more into R3 now while we're planning" (B3).

**Dispatch trigger**: PM surfaces tradeoff (timeline impact + work scope) for Brian review.
**Closure predicate**: §1 closure criteria + §1.8 ledger expansion if included.

### CP3 — Q-WEDGE-A framing + Director engagement

**Scope**: PM provides 1-2 sentence framing on WEDGE-CORE-CLAIM Part A (ctrl#444 build-orchestration as free consequence) + thesis-edit scope. Then Director ratifies.

**Dispatch trigger**: PM authors framing first; Director engages second.
**Closure predicate**: thesis-edit gate inclusion in §1 if ratified.

### CP4 — Q-Class-6-Substrate-Extension-Lens (Brian/Director scope)

**Scope**: 3 ctrl-side substrate proposals (SchemaPreservation / ResourceCap / TotalResult) — Class 6 inclusion in R3 vs declared-out-of-scope.

**Dispatch trigger**: pending Research PM sharper framing per Director HOLD disposition.
**Closure predicate**: §1 + §4.6 expansion if included; or explicit out-of-scope-by-design.

### CP5 — PR #1794 merge (closes Q-Self-Host-Ratchet-Timeout)

**Scope**: PR #1794 (quick-ferret-413) bumps `self_host_ratchet` timeout-minutes 30 → 60. Director-cleared CLOSED-by-state disposition.

**Dispatch trigger**: now (Director cadence merge).
**Closure predicate**: Q-Self-Host-Ratchet-Timeout closes.

---

## §8. Sequencing summary

**Critical path** (per `r3-program-plan.md` §6):

```
T-E-P-Producer-Broadening (S10; foundational)
   ↓
T-Lens-Behavioral-Parity (Substrate + Verification cross-program)
   ↓
[ T-Lens-Application-Surface PARALLEL T-Workflow-As-Data ]
   ↓
T-Lens-Self-Application (V3; demonstration)
```

**Parallel longest single-lane**: T-V2-Retirement (P3) — gated on T-FixedPoint (P2) + T-LensProducer-Retirement (P1) + T-Numeric-Construction `Int<N>` (S9 + S3 cascading).

**Verification-internal path**: T-V-L4-L7-Direct (V2) → T-V-L5-Corpus.

**Bottleneck escalations** (R3 close horizon):
- **Q-LBP-R3-Closeability** (CP1) — gates the LBP cascade; Substrate canvas (S2) prereq
- **Q-Class-6** (CP4) — potential 6th substrate-gap class adding ~1 lane equivalent
- **Q-Tier4-Inclusion** (CP2) — fundamental scope decision

---

## §9. Status update cadence

- **Daily**: Mgr-internal dispatch tracking; lane-owning Mgr updates §1.8 Status column as gates ratchet DECLARED → CONSUMER_LANDED → PASSING
- **Weekly Monday**: PM compiles §3 lane status from Mgr per-lane reports
- **Weekly Wednesday**: closure-gate progress check
- **Weekly Friday**: bridge-ledger receipt cadence

**Cross-Mgr coord**: blockers route to Director via standard cross-Mgr queue (per `r3-structure.md` §"Manager structure"). Cross-program ctrl-side dependencies route to Research PM (gunb-ai/ctrl#339).

---

## §10. References

- [`docs/r3-structure.md`](r3-structure.md) — architectural archive (95 closure gates; 18 lanes; 9 Mgrs)
- [`docs/r3-program-plan.md`](r3-program-plan.md) — forward-looking dependency graph + escalation register + §1.8 canonical 95-gate ledger
- [`docs/audit/r3-debt-sweep-2026-05-06.md`](audit/r3-debt-sweep-2026-05-06.md) — bridge-inventory framework
- Per-Mgr canvas threads: gunbc#1739 (Substrate) / #1740 (Verification) / #1742 (PB) / #1743 (Evaluator) / #1744 (Debt-Paydown) / #1745 (Grounding)
- Director inbox: gunbc#828
- Research PM inbox: gunb-ai/ctrl#339
