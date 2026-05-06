# R3 Program Plan — 2026-05-06

**Status:** **DRAFT — under PM authorship + Mgr-canvas absorption.** Phase 1 (skeleton + closure criteria + lane status) authored; Phase 2 (per-Mgr escalation absorption) compiles canvas responses as they arrive on inboxes #1739–#1745. Phase 3 (Director ratification) closes Q1–Q7 in §10. Plan PR opens after §10 escalations clear.

**Purpose.** Forward-looking program plan for **R3 close with zero debt**. Per user directive 2026-05-05 (gunbc#846): *"clear dependency graph from here to R3 close"* + *"surface escalations now and solve them"*.

**Authority hierarchy** (all three docs land in this PR — single coordinated authority drop):
- [`docs/r3-structure.md`](r3-structure.md) — **architectural-decision archive**. Lane definitions, manager structure, design-challenges resolution, dependency-on-R2 spec. Authoritative for *what R3 is*. Updated in this PR to absorb new closure gates.
- [`docs/audit/r3-debt-sweep-2026-05-06.md`](audit/r3-debt-sweep-2026-05-06.md) — **bridge inventory snapshot framework** (PM-authored Phase 1 framework; codex APPROVE on prior sha `860e175d`; Phase 2-3 inventory populates as Mgr canvases compile via PR comments). Authoritative for *what bridges exist now*. Consolidated into this PR per Grounding Mgr poke-hole 2026-05-06 finding 1 + codex BLOCKING on PR #1808 sha `0719d7a6` line 9 (eliminates cross-PR dependency on prior PR #1804 — superseded).
- **THIS DOC** — **forward-looking dependency-graph + escalation register**. Authoritative for *path from here to R3 close + open ratifications*.

Cross-link discipline: this plan never restates lane definitions from `r3-structure.md`; cites them by section/symbol per `feedback_section_anchors_over_line_numbers`.

---

## §1. R3 closure criteria — precise predicates

**Brian-ratified language (gunbc#846, 2026-05-05):** *"all 5 substrate-gap classes closed + v2 fully retired + Pattern A NYI predicates executable + BridgeLedgerZero ratcheting at zero."*

PM-recommended TestClaim predicate set (RATIFIED 2026-05-06 per Brian directive at gunbc#846; Q1 in §10.1):

### §1.1 Pattern A NYI predicates executable (7 predicates)

Each predicate has a passing TestClaim against representative input — **NOT** a `NotYetImplemented` shell. Predicate-shape varies by family (per Verification Mgr poke-hole 2026-05-06): the Brian-ratified language *"Pattern A NYI predicates executable + BridgeLedgerZero ratcheting at zero"* covers four distinct predicate-shape families, not a single DimensionReport-typed receipt.

**Predicate-shape families:**

| Predicate | Shape family | TestClaim gate (proposed; Q1 ledger-name) | Owner Lane |
|---|---|---|---|
| TC1 (η-equivalence) | `BinaryDimensionReportEquals` (DimensionReport-typed) | `tc1_eta_equivalence_executable` | T-V-L4-L7-Direct |
| TC2 (Church-Rosser) | `BinaryDimensionReportEquals` (DimensionReport-typed) | `tc2_church_rosser_executable` | T-V-L4-L7-Direct |
| TC3 (Pattern-A second-mover) | `BinaryDimensionReportEquals` (DimensionReport-typed) | `tc3_pattern_a_second_mover_executable` | T-V-L4-L7-Direct |
| RustDagIsomorphism | structural-equivalence assertion (Dag isomorphism, not DimensionReport) | `rust_dag_isomorphism_executable` | T-V-L4-L7-Direct |
| SymbolicCostExprEquals | cost-predicate (`SymbolicCost`-typed; per `src/v3/compiler/src/test_runner.rs` `SymbolicCostExprEquals: structural shape is valid` NYI receipt) | `symbolic_cost_expr_equals_executable` | T-CostLens-Composition |
| BridgeLedgerZero | ledger-row count (Pattern E ratchet — decreasing-open-count, NOT DimensionReport; per `ROADMAP.md` §"Course correction #4" + `r3-structure.md` `bridge_retirement_ledger_zero` gate) | `bridge_retirement_ledger_zero` (existing) | T-Bridge-Retirement |
| Free-Consequences-Demonstration | **composite of 10 gates** (NOT a single predicate; `r3-structure.md` §"Acceptance" → T-Free-Consequences-Demonstration gates) | (composite — see existing gates) | T-Free-Consequences-Demonstration |

**Gate names labeled as "proposed (Q1 ledger-name)"** for the 5 newly-named predicates that don't yet exist in `r3-structure.md` §"Acceptance" or `src/v3/std/verification.dag` as authored TestClaims. Existing gates (`bridge_retirement_ledger_zero`, T-Free-Consequences-Demonstration's 10 gates) keep canonical names.

Existing fixtures in `docs/briefs/r3-v-pattern-a-coverage-rollup.md` + `r3-v-pattern-a-second-mover-conformance-audit.md` series document predicate scaffolds. **Open** (Verification Mgr canvas escalation V1, §10): first executable slice — see §2.1 + Q-PAFS in §10.3.

### §1.2 v2 fully retired (2 predicates — already declared)

Existing gates (`r3-structure.md` §"Acceptance" → T-V2-Retirement gates):
- `v2_oracle_no_remaining_test_consumers`
- `v2_directory_deleted`

Owner: PB Mgr (T-V2-Retirement lane). No new predicates needed.

### §1.3 BridgeLedgerZero ratcheting at zero (1 predicate — already declared)

Existing gate (`r3-structure.md` §"Acceptance" → T-Bridge-Retirement gates):
- `bridge_retirement_ledger_zero` — unified ledger reports 0 named identity bridges remaining.

**Bridge-class inclusion** (per Director poke-hole 2026-05-06 finding 1.2; closes count-criteria gap):
- **Counts toward zero**: Class A (substrate-gap-blocked) + Class B (Pattern-A NYI predicates) + Class C (typed-carrier + Rust-mirror) + Class F (operator-ontology-duplication) + Class G (local-small).
- **Excluded by-construction**: Class D (generated bridges with freshness gates — bounded scaffolds, not debt by intent per `r3-debt-sweep-2026-05-06.md` §Class D) + Class E (v2/v3 transition — retires at v2 retirement which has its own gate `v2_directory_deleted`).

Owner: Verification Mgr (ledger gate); retirement work distributed per `r3-structure.md` §"Lane structure" → T-Bridge-Retirement row (2 Substrate-owned + 3 PB-owned).

### §1.4 5 substrate-gap classes closed (5 predicates — NEW; Brian-ratified scope)

Per `r3-debt-sweep-2026-05-06.md` §Class A (line 39): *"parser/grammar surface, function-valued data, file-ingestion, workflow/scheduling, reflection-closure."* 

**PM-recommended framing (Q2, §10): these are CLOSURE CRITERIA over existing lane work, NOT new separate lanes.** Each class closes when **both**: (a) representative gap-test executes through v3 cleanly per Pass condition; AND (b) systematic enumeration of class-bridges shows count = 0 (or any survivors carry explicit Director allocation per §7.2 STRUCTURAL exception). Per Director poke-hole 2026-05-06 finding 1.3 — single-test-pass alone is sample-of-class, not closure-of-class; conjunctive predicate prevents class reading "GREEN" while real bridges in that class persist unexercised by the chosen representative.

| Class | TestClaim gate | Representative gap-test | Lane(s) producing closure |
|---|---|---|---|
| 1 — parser/grammar surface | `substrate_gap_parser_grammar_closed` | v3 substrate models concrete types as the **interaction of independent constraint models** — abstract algebra × machine constraints — with concrete types emerging as consequences (per Brian directive 2026-05-06: *"these concepts should be the interaction of several constraint models i.e. abstract algebra × machine constraints — i64 would naturally emerge as a consequence"*). Refinement syntax `Type<N>` expresses the interaction, NOT a primary entity. Pass condition: substrate declares abstract algebra (e.g., `Int = AbelianGroup<Nat>`) and machine constraints (e.g., `MachineWidth<bits>`) as independent models; concrete types emerge via interaction (e.g., `Int<64>` = `Int × MachineWidth<64>`; target-language `i64` is derived artifact, NOT primary substrate entity). v3 parser handles generic interaction syntax; demonstrated against ≥3 algebra×constraint pairs — `Int<64>` (integer × 64-bit register), `Real<64>` (real × 64-bit float), `Nat<8>` (natural × 8-bit register) — without v2-fallback; gate Pass = all 3 interaction-derived programs produce 0 diagnostics + emitted artifact compiles to expected target primitive (`i64`/`f64`/`u8`) where the target primitive is consequence-of-interaction, not declared. The interaction-modeling requirement prevents single-axis (algebra-only or width-only) hard-coding. | T-V2-Retirement (parser path) + T-Numeric-Construction (algebra side) + Substrate (machine-constraint side, may require new substrate carrier `MachineConstraint<C>`) + Substrate continuation |
| 2 — function-valued data | `substrate_gap_function_valued_data_closed` | `Lens<C>` instance with function-typed payload executes through evaluator | T-Lens-Application-Surface + T-E-P-Producer-Broadening |
| 3 — file-ingestion | `substrate_gap_file_ingestion_closed` | One `.dag` program ingests external file (e.g., timing observation set) without `include_str!` | T-Workflow-As-Data |
| 4 — workflow/scheduling | `substrate_gap_workflow_scheduling_closed` | CI workflow modeled as `.dag` data executes through evaluator | T-Workflow-As-Data + T-Lens-Self-Application |
| 5 — reflection-closure | `substrate_gap_reflection_closure_closed` | `lens_apply.rs` reflection work routes via PB-Runtime interpreter-as-data | T-LensProducer-Retirement |

**Q2 status (per §10.1)**: RATIFIED 2026-05-06. All gap-test selections per-class take PM recommendations per Brian directive (*"take recommendations unless controversial on all Q1-Q7"*). No outstanding ratification gap.

### §1.5 Closure-ledger total

**Total R3 closure gates** (post-Q1 + Q2 ratification): **70 gates** at sha `a72293bbf` per `r3-structure.md` §"Acceptance" enumeration. Composition: existing R3 lane gates (`T-Tier3-Dissolution` 4, `T-LensProducer-Retirement` 4, `T-V-L4-L7-Direct` 2, `T-V-L5-Corpus` 1, `T-FixedPoint` 1, `T-Numeric-Construction` 8, `T-Omni-Shape-B` 4, `T-Anthropic-Wire` 2, `T-Bridge-Retirement` 6, `T-CostLens-Composition` 3, `T-V2-Retirement` 2, `T-Free-Consequences-Demonstration` 10, `T-Workflow-As-Data` 4, `T-Lens-Self-Application` 3) = 54 lane gates; plus 5 substrate-gap-class gates + 10 demonstration gates + 1 PR-anticipation-discipline gate = 16 gates added 2026-05-06 in this PR. Total: 54 + 16 = 70 across 18 lanes + 1 standing program.

R3 closes when ALL gates pass + zero tracked-debt rows survive (`r3_debt_paydown_zero_remaining`).

**Tracked-debt inclusion list for `r3_debt_paydown_zero_remaining`** (per Director poke-hole 2026-05-06 finding 3.1; closes definition gap):

- **Counts toward zero**: ROADMAP open `- **` rows under "Post-merge debt" sections; sweep-doc §1 Class A/B/C/F/G entries (per §1.3 inclusion criteria); §10 RED-flagged escalation items.
- **Excluded by-construction**: PM-coordination items (Brian-Q queue, dispatch routing); scheduled-deletions-with-named-trigger (the trigger IS the resolution); retirement-receipts (post-resolution accounting); Class D bridges (bounded-scaffold freshness gates) and Class E bridges (v2/v3 transition — retires at v2 retirement which has its own gate).

Predicate cannot be satisfied by deletion-of-rows-without-actual-resolution; each retiring row requires named PR receipt OR closure-ledger entry citing the structural event that ends the row's existence.

### §1.7 Closure-criteria status — DECLARATIONS-ONLY staging (per openai-pro meta-review 2026-05-06)

**This plan + r3-structure.md DECLARE 70 closure gates. The declarations are NOT YET load-bearing closure predicates — no consumer infrastructure verifies them at HEAD.**

Per openai-pro meta-review on PR #1808 sha `cf249389` ([#issuecomment-4384405832](https://github.com/gunb-ai/gunbc/pull/1808#issuecomment-4384405832)) PAUSE_AND_REGROUP verdict — structurally honest framing of where this PR sits in the closure-evidence chain:

**Per-gate status taxonomy:**

- **DECLARED** — gate id + Pass condition exists in `r3-structure.md` §"Acceptance" or this plan §1. Document-level only.
- **CONSUMER_LANDED** — the consumer (CI check, `.dag` TestClaim runner executing through Evaluator, manual demonstration receipt) executes the Pass condition + can fail-closed if violated.
- **PASSING** — consumer landed AND Pass condition currently true.

**Status at HEAD (PR #1808 sha at this commit)**: ~all 70 gates are **DECLARED**; very few have CONSUMER_LANDED status (existing `pb_self_compile_fixed_point` + `tier3_*_mirror_dissolved` + similar pre-R3-plan gates have consumers; the 16 NEW gates added 2026-05-06 in this PR are DECLARED-only).

**R3 close criteria implies CONSUMER_LANDED for all 70 gates**: declarations alone don't satisfy `r3_debt_paydown_zero_remaining` or the substrate-gap-class closures or the demonstration principle. Per Brian directive `feedback_no_textual_enforcement_bridges` + Director poke-hole 2026-05-06 finding 1.1 (demo-gate minimum bar) — closure requires runtime-executable verification, not document-level claims.

**Consumer infrastructure to land before R3 close**:
- **T-Tests-As-Data-Completeness lane** (existing R3 lane): every test ports to `.dag` TestClaim or generated target-language test code; predicates execute via Evaluator. Closes the predicate→consumer gap for substrate-gap-class + Pattern-A predicate gates.
- **R3 Debt-Paydown PR-anticipation gate** (added 2026-05-06): `pr_anticipation_discipline_ci_active` — CI verifiably enforces §7 PR-authoring contract via `scripts/check-pr-sg0-net-shrink-discipline.sh`. Closes the predicate→consumer gap for going-forward debt-discipline gates.
- **Demonstration-gate minimum bar consumer** (per §1.6): each demonstration gate requires runtime-executable demonstration; lane-owning Mgr authors the consumer alongside gate declaration. Closes the predicate→consumer gap for the 10 NEW demonstration gates.

**This PR's scope is intentionally declarations + plan; consumer authoring happens in subsequent PRs by lane-owning Mgrs** (per `feedback_bundle_workstreams_per_pr` — single coordinated authority drop here, distinct PRs for consumer infrastructure).

The honest framing: this PR is **planning + declaration substrate** for R3 close, NOT R3-close-evidence. Without this label, downstream readers might misinterpret declarations as load-bearing closure predicates.

### §1.6 Demonstration principle (Brian-ratified 2026-05-06)

**Every feature comes with a demonstration.** Generalizes Q5 (free-consequences demonstration) to a structural requirement: every R3 lane MUST include at least one demonstration-shaped TestClaim — a runtime-executable claim that exercises the lane's deliverable end-to-end, not just static-fact assertions about substrate shape.

**Demonstration discipline:**
- Static-fact gate (e.g., `X declared in std/`) is necessary but not sufficient.
- Demonstration gate runs the feature on a representative input and produces an observable result.
- "Look, it runs" minimal artifact per `r3-structure.md` §"R3 demo discipline + omni-emission TDD".
- Closure ledger reports demonstration-gate status separately from substrate-shape gates.

**Demonstration-gate minimum bar** (per Director poke-hole 2026-05-06 finding 1.1; prevents Mgrs declaring static-fact gates as demonstrations):

A runtime-executable demonstration gate satisfies §1.6 only if **all three**:
- **(a) End-to-end execution**: runs through v3 evaluator end-to-end (NOT a static-fact assertion about declared types or compiler internals).
- **(b) Observable output**: produces observable result — `DimensionReport<C>`, executable artifact (compiles + runs), runtime check result, or witness-construction output. The demonstration must produce something a reader can inspect.
- **(c) Non-trivial input**: representative program with at least 2 distinct algebraic constructs (e.g., for a function lens demo: a function with ≥1 branch + ≥1 bind, not a one-line `id` function). Single-line fixtures do not satisfy.

Mgrs author per-gate spec citing the (a)/(b)/(c) satisfaction; closure-ledger entries record observed output.

**Per-lane demonstration audit** (RATIFIED 2026-05-06 per Brian directive at gunbc#846; Q5 generalized to demonstration principle in §10.1):

| Lane | Existing demo gate | Needed demo gate (if missing) |
|---|---|---|
| T-Tier3-Dissolution | `tier3_*_mirror_dissolved` (state-check) | + `tier3_dissolution_demonstration_executes` — at least one Tier3-mirror-consumer .dag program runs end-to-end via Evaluator |
| T-LensProducer-Retirement | retirement gates (state-check) | + `lens_producer_retirement_demonstration` — `lens_apply` reflection routes via PB-Runtime on representative lens program |
| T-V-L4-L7-Direct | `l4_emit_eval_match`, `l7_algebraic_laws_witnessed` (skeleton/staged at HEAD; closure bar = exhaustive per-(algebra, inhabitant, law) coverage per `r3-structure.md` §"Lane structure" → T-Verification-L4-L7-Direct row + 2026-05-02 fold-in) | (existing harness — NOT yet "fully demonstrated"; over-greening risk per Verification Mgr poke-hole 2026-05-06) |
| T-V-L5-Corpus | `l5_cross_target_consistency` ✓ | (existing) |
| T-FixedPoint | `pb_self_compile_fixed_point` ✓ | (existing) |
| T-Numeric-Construction | substrate-shape gates only | + `numeric_construction_demonstration` — end-to-end program using `Int<32>` + `Real<64>` round-trip executes |
| T-Omni-Shape-B | `omni_openapi_backend_emission_demo` ✓ | (existing — multiple demos) |
| T-Anthropic-Wire | wire alignment gates | + `anthropic_wire_demonstration` — full request/response cycle executes against deterministic mock (live-API testing tracked separately as CI cadence concern, NOT closure gate per Director poke-hole 2026-05-06 finding 4.3 — avoids CI flakiness) |
| T-Bridge-Retirement | `bridge_retirement_ledger_zero` (count-check) | + `bridge_retirement_demonstration` — at least one retired-bridge replacement path executes (e.g., typed-identity-surface used in production code) |
| T-CostLens-Composition | structural-fold gates | + `cost_lens_demonstration` — cost lens reads representative target program (≥2 algebra-instances composed, ≥1 recursive call, observable cost-bound output per (c)) and composes algebra+realization cost end-to-end |
| T-V2-Retirement | deletion gates (state-check) | + `v3_self_host_demonstration` — bootstrap path through PB-Runtime trampoline executes end-to-end; v3-only self-host pipeline runs without v2 fallback (Director poke-hole 2026-05-06 finding 4.1: reframed from `v2_retirement_demonstration` "deletion's inverse" to direct positive-statement form) |
| T-Free-Consequences-Demonstration | 10 demo gates ✓ | (existing — full demo suite) |
| T-E-P-Producer-Broadening | substrate-shape gates | + `e_p_producer_demonstration` — representative call site produces full descent evidence at runtime |
| T-Lens-Behavioral-Parity | parity-complete gates | + `lens_behavioral_parity_demonstration` — each lens (complexity/cost/parallelism/effect_enumeration) demonstrates on representative input + matches v2 oracle |
| T-Tests-As-Data-Completeness | substrate-shape gates | + `tests_as_data_demonstration` — at least one Rust test ports to .dag TestClaim and executes |
| T-Lens-Application-Surface | 4 worked-example demos ✓ | (existing) |
| T-Workflow-As-Data | `ci_workflow_modeled_as_dag` ✓ | (existing) |
| T-Lens-Self-Application | `lens_self_application_demonstrated` ✓ | (existing) |

**Net additions**: 10 new demonstration gates across substrate-heavy / state-check-only lanes (per per-lane audit table above; rows with "+" prefix in "Needed demo gate" column). Owner Mgrs author per their lane scope.

This principle is NOT a separate lane; it's a per-lane gate-shape requirement applied uniformly. Closes when each lane has at least one runtime-executable demonstration gate green.

---

## §2. Per-predicate close path

For each predicate not yet GREEN, what's blocking + sequence to close.

### §2.1 Pattern A executable (4-family predicate cluster)

**Critical-path blocker (DimensionReport-typed family — TC1/TC2/TC3)**: at least one `BinaryDimensionReportEquals` path must compare real `DimensionReport<C>` outputs (not `NotYetImplemented` shells) before any of TC1/TC2/TC3 can ratchet to executable. Per Verification Mgr canvas escalation V1 (cool-owl-579).

**Other Pattern-A families** have distinct unblock paths (per Verification Mgr poke-hole 2026-05-06):
- **RustDagIsomorphism**: structural-Dag-equivalence harness; not gated on DimensionReport unblock.
- **SymbolicCostExprEquals**: cost-predicate; gated on `SymbolicCost`-side runner work (separate from DimensionReport).
- **BridgeLedgerZero**: ledger-count ratchet; gated on retirement work landing across 5 sub-bridges (see §2.3).
- **Free-Consequences**: 10 sub-gates each with their own runtime prerequisites.

**Sequence (DimensionReport-typed family)**:
1. **Director countersignature on first executable slice** (Q-PAFS, §10) — TC1 η-equivalence vs RustDagIsomorphism vs other. *Note: `r3-v-tc1-eta-equivalence-deeper-analysis.md` is `Status: PROPOSAL / research-only` and explicitly states "This analysis does not choose among those paths." TC1-first is **PM/Brian convenience default** pending Director engineering sign-off, NOT Verification-Mgr-ratified from the analysis alone.*
2. R3 Evaluator Mgr surfaces runtime prerequisites for chosen slice (`Pattern A` runtime executable).
3. Substrate Mgr authors any missing carrier (likely none if first slice is TC1).
4. Verification worker dispatches gap-test → first predicate ratchets to executable.
5. Remaining 6 predicates parallel-fire once first path proves the shape.

**ETA**: 1-2 weeks post-ratification; cascade of remaining 6 within 2-4 weeks.

### §2.2 v2 retirement (2 predicates)

**Sequence** (from `r3-structure.md` §"Lane structure" → T-V2-Retirement row; Grounding Mgr poke-hole 2026-05-06 finding 2 corrected dependency direction):
1. T-FixedPoint completes (PB Mgr) — `compiler.dag` self-compile fixed-point
2. T-LensProducer-Retirement completes (PB Mgr) — 3 hand-Rust files retired
3. T-Numeric-Construction `Int<N>` refinement landing (path-(a) coordination — v2 refinement-syntax blocker per `r3-structure.md` §"Lane structure" → T-Numeric-Construction row)
4. v2 directory deletion + test-consumer audit (T-V2-Retirement)

**ETA**: 4-8 weeks; longest-path lane in R3.

**Distinct Grounding-side dependency branch (NOT v2-retirement prerequisite — moved out per Grounding Mgr poke-hole 2026-05-06 finding 2)**:

PR-F (`BoundDeclaration` consumer + Rust `ReferenceModel<T>` axes) + Float migration (T-NumericConstruction-ApproximateField) are blockers for **full Rust primitive grounding** (T-Ground-Rust complete-coverage) + downstream **L5 cross-target consistency**, NOT for `v2_directory_deleted` gate.

Sequence (Grounding-side):
1. PR-F lands (Substrate Mgr; unblocks T-Ground-Rust Phase 1: `u128`, `isize`, `usize`, walker arms, pilot mirror)
2. Float carrier migration (T-NumericConstruction-ApproximateField → `Float32/Float64 = ApproximateField<Word*>`; unblocks Rust float rows)
3. T-Ground-Rust full-coverage (post-PR-F + post-Float)
4. T-V-L5-Corpus closes once L4 corpus + Shape A grounding (Rust + Python) both ready

### §2.3 BridgeLedgerZero (1 predicate)

**Sub-bridges** (`r3-structure.md` §"Lane structure" → T-Bridge-Retirement row distribution map):
- **Substrate-owned (2)**: `SourceSpan.file` participation checks; `mark_bootstrap_secret_nominal_opacity()`
- **PB-owned (3)**: canonical lens-name dispatch; `include_str!` side channels (e.g., `pipeline_authority.rs`); `patch_lower_helpers_*` residual

Each retires per its natural-owner program prerequisites. Verification Mgr's `bridge_retirement_ledger_zero` gate audits cross-program reporting cadence.

**Sequence**:
1. Substrate: typed-identity-surface for SourceSpan participation; Secret PR A continuation.
2. PB: canonical lens-name dispatch retires alongside T-LensProducer-Retirement; `include_str!` retires post-T-FixedPoint; `patch_lower_helpers_*` post-Tier-2.
3. Verification audits cumulative ledger; closes when count = 0.

**ETA**: 6-10 weeks (gated on T-FixedPoint + T-LensProducer).

### §2.4 5 substrate-gap classes (5 predicates)

**Class 1 (parser/grammar)**: closes with T-V2-Retirement parser path (post-T-FixedPoint + post-T-LensProducer-Retirement + post-T-Numeric-Construction `Int<N>`).
**Class 2 (function-valued data)**: closes with T-Lens-Application-Surface (post-T-Lens-Behavioral-Parity).
**Class 3 (file-ingestion)**: closes with T-Workflow-As-Data file-ingestion grammar (post-T-Lens-Behavioral-Parity).
**Class 4 (workflow/scheduling)**: closes with T-Workflow-As-Data CI-workflow-as-dag (post-T-Lens-Behavioral-Parity).
**Class 5 (reflection-closure)**: closes with T-LensProducer-Retirement (gated on T-FixedPoint via PB cascade chain — NOT parallel-to-LBP per Director poke-hole 2026-05-06 finding 2.4 correction).

**ETA**: Classes 2/3/4 parallel-eligible post-T-Lens-Behavioral-Parity COMPLETE. Class 1 + Class 5 cascade-gated (Class 1 on T-V2-Retirement chain; Class 5 on T-LensProducer-Retirement which depends on T-FixedPoint). 4-8 weeks total for parallel-eligible classes; cascade-gated classes track longer.

---

## §3. Lane status snapshot

One row per lane: current state + blocker + ETA-to-close. Updated weekly by lane-owning Mgr; PM compiles for Director cadence reporting.

| Lane | Mgr | Status | Current dispatch | Blocker | ETA-to-close |
|---|---|---|---|---|---|
| T-Tier3-Dissolution | PB | YELLOW | (TBD from PB canvas) | (TBD) | (TBD) |
| T-LensProducer-Retirement | PB | YELLOW | (TBD from PB canvas) | T-FixedPoint + R2-Evaluator | (TBD) |
| T-V-L4 (emit/eval match) | Verification | YELLOW | r3-v-l4-l7-direct-* briefs | Pattern A first slice (Q-PAFS) + corpus build-out | (TBD) |
| T-V-L7 (algebraic-law witness coverage) | Verification | YELLOW | r3-v-l7-algebra-coverage-matrix briefs | per-(algebra, inhabitant, law) exhaustive coverage gap | (TBD) |
| T-V-L5-Corpus | Verification | RED | (waits on L4 corpus + Shape A grounding) | T-V-L4 corpus existing first | post-L4 |
| T-FixedPoint | PB | YELLOW | (TBD from PB canvas) | SG-0 zero from T-LensProducer | (TBD) |
| T-Numeric-Construction | Substrate | YELLOW | T-NumericConstruction-ApproximateField; #1523 landed | Float migration + Real/base-carrier convention (Grounding canvas item 2) | (TBD) |
| T-Omni-Shape-B | Release | RED | (post-R2-Evaluator + Shape A targets) | dependencies | (TBD) |
| T-Anthropic-Wire | Substrate | RED | #1702 CLOSED + preserved on `codex/cc1-target-integer-structural-fold` at sha `51c6a4a`; held pending Substrate variant-aware projection metadata carrier | Q-Anthropic-Variant-Aware (per Grounding Mgr poke-hole 2026-05-06 finding 3 — §3 corrected from GREEN-pending to RED-blocked) | post-Substrate-projection-carrier |
| T-Bridge-Retirement | Verification (ledger) | YELLOW | per-bridge dispatches | 5 sub-bridges retire | post-T-FixedPoint |
| T-CostLens-Composition | Substrate | YELLOW | (TBD from Substrate canvas) | R2-Evaluator + R2-T-Substrate-Lens-Primitive | (TBD) |
| T-V2-Retirement | PB | RED | r3-tv2-* briefs | T-FixedPoint + T-LensProducer | longest-path |
| T-Free-Consequences-Demonstration | Verification | YELLOW | r3-v-free-consequences-worker | R2-Evaluator + T-CostLens-Composition | post-deps |
| T-E-P-Producer-Broadening | Substrate | YELLOW | (TBD from Substrate canvas) | (foundational; in flight) | weeks |
| T-Lens-Behavioral-Parity | Substrate + Verification | RED | post-T-E-P-Producer-Broadening | T-E-P-Producer-Broadening | post-T-E-P |
| T-Tests-As-Data-Completeness | Verification | YELLOW | r3-v-tests-as-data-* briefs | TestClaim infra | (TBD) |
| T-Lens-Application-Surface | Substrate + Verification | YELLOW | r3-substrate-tests-as-data-carrier-slice-1-stop-ping | T-Lens-Behavioral-Parity COMPLETE | post-LBP |
| T-Workflow-As-Data | Substrate | YELLOW | r3-v-bridge-row-* + design-timing-lens | T-Lens-Behavioral-Parity COMPLETE | post-LBP |
| T-Lens-Self-Application | Verification | RED | (waits on Workflow-As-Data + LAS) | T-Workflow-As-Data + T-Lens-Application-Surface | post-WAD+LAS |
| T-Debt-Paydown (standing) | Debt-Paydown | YELLOW | (TBD from canvas) | per-PR rule + drift-item reconciliation | continuous |

**Status legend**: GREEN = no open work, awaiting close audit. YELLOW = work in flight. RED = blocked on upstream.

(Mgr canvases populating; full table compiles within ~24h per canvas T+24h soft deadline.)

---

## §4. 5 substrate-gap classes — detail

**Framing (PM-recommended; Q2, §10)**: closure criteria over existing lane work, not new separate lanes. Each class is a TestClaim that fires when its representative gap-test executes through v3 cleanly.

### §4.1 Class 1 — parser/grammar surface

**What this class names**: bridges that exist because v3 substrate doesn't yet model the **interaction of independent constraint models** — abstract algebra × machine constraints — from which concrete types emerge. v3 parser handles algebra-side refinements somewhat (per T-Numeric-Construction in flight) but doesn't model machine-constraint as an independent axis that **interacts** with algebra to produce concrete-type instances. Concrete types like `i64` should emerge as consequences of `Int × MachineWidth<64>`, not be declared as primary entities. v2 currently catches these via various ad-hoc paths; v3 needs the substrate-level interaction model.

**Modeling discipline (Brian directives 2026-05-06 at gunbc#846)**:

> *"We are usually modeling abstract algebra or other concepts, and its likely that any machine/computer oriented concepts are a projection of those onto specific constraints (i.e. memory registers) - my point is, we have to handle this generically for 'all' types, not just 'int' - though int is a good representation and good north star for others to take note of."*

> *"At some point we probably want these concepts to be the interaction of several constraint models i.e. abstract algebra × machine constraints — i64 would naturally emerge as a consequence."*

**Modeling target**: independent constraint models (algebra + machine) with concrete types emerging from their interaction. Substrate carries `MachineConstraint<C>` as an independent axis; concrete-type-instances are products. Other parser-surface gaps (`where bits <= N`, function-valued return types) fall under this class but are subordinate to the interaction-modeling gap.

**Representative gap-test**: substrate declares abstract algebra and machine constraints as independent models; concrete types emerge via interaction. Demonstrated against ≥3 algebra×constraint pairs:

- **`Int<64>` = `Int × MachineWidth<64>`** — integer × 64-bit register interaction; emits to target `i64` as derived consequence (north-star)
- **`Real<64>` = `Real × MachineWidth<64>`** — real × 64-bit float interaction; emits to target `f64`
- **`Nat<8>` = `Nat × MachineWidth<8>`** — natural × 8-bit register interaction; emits to target `u8`

Pass condition: 
- (a) substrate declares `Int` / `Real` / `Nat` as algebra-side carriers (T-Numeric-Construction; partly landed)
- (b) substrate declares `MachineConstraint<C>` (or analogous) as independent constraint axis (NEW substrate work; tracked in the closure-via column)
- (c) parser handles generic interaction syntax; concrete-type literals like `42i64` resolve via `Int × MachineWidth<64>` lookup
- (d) all 3 interaction-derived programs produce 0 diagnostics + emitted target primitives compile to `i64`/`f64`/`u8` respectively
- (e) target primitives (`i64` / `f64` / `u8`) are NOT primary substrate entities — they're derived artifacts of the interaction lookup

**Closes via**: T-V2-Retirement parser path (post-T-FixedPoint) + T-Numeric-Construction (algebra side; in flight) + **Substrate continuation work to add `MachineConstraint<C>` independent carrier** (NEW scope per Brian directive 2026-05-06; surfaces as Q-MachineConstraint-Carrier in §10.3).

**Owner Mgr**: Substrate Mgr (machine-constraint carrier authoring + T-Numeric-Construction algebra side) coordinating with PB Mgr (T-V2-Retirement parser path).

### §4.2 Class 2 — function-valued data

**What this class names**: bridges where `.dag` substrate cannot represent function-typed values (functions as data, lenses as functions, etc.). Currently bridged via Rust escape hatches (`lens_apply.rs`, hand-Rust witness construction).

**Representative gap-test**: `Lens<C>` instance with function-typed payload executes through evaluator and produces `DimensionReport<C>` without Rust mediation.

**Closes via**: T-Lens-Application-Surface (`lens_application_carrier_landed`) + T-E-P-Producer-Broadening (per-call descent evidence carries function references).

**Owner Mgr**: Substrate Mgr (lens carrier authoring) + Verification Mgr (gap-test demonstration).

### §4.3 Class 3 — file-ingestion

**What this class names**: bridges where `.dag` programs cannot ingest external files; currently use `include_str!` macro or path-keyed Rust reads.

**Representative gap-test**: One `.dag` program ingests external file (timing observation set, JSON config, etc.) without `include_str!`; exercises the file-ingestion grammar.

**Closes via**: T-Workflow-As-Data file-ingestion substrate (`workflow_substrate_carriers_landed` extended to file-attachment).

**Owner Mgr**: Substrate Mgr (T-Workflow-As-Data lane).

### §4.4 Class 4 — workflow/scheduling

**What this class names**: bridges where workflow/scheduling logic exists in non-`.dag` form (CI YAML, build orchestration scripts, hand-Rust workflow control flow).

**Representative gap-test**: CI workflow modeled as `.dag` data executes through evaluator + produces `DimensionReport<TimingMeasurement>` (gate `ci_workflow_modeled_as_dag`).

**Closes via**: T-Workflow-As-Data + T-Lens-Self-Application (per `r3-structure.md` §"Acceptance" → T-Workflow-As-Data gates + §"Lane structure" → T-Workflow-As-Data row).

**Owner Mgr**: Substrate Mgr (substrate authoring) + Verification Mgr (demonstration).

### §4.5 Class 5 — reflection-closure

**What this class names**: bridges where the compiler reflects on its own structure via Rust escape hatches (`lens_apply.rs` reflection work, hand-authored emission scaffolding).

**Representative gap-test**: `lens_apply.rs` reflection routes via PB-Runtime interpreter-as-data; reflection-as-data substrate exercised end-to-end.

**Closes via**: T-LensProducer-Retirement (`lens_apply_dot_rs_retired`) + advanced lifetime-analyzer cases d/e/f (per `r3-structure.md` §"Lane structure" → T-LensProducer-Retirement row).

**Owner Mgr**: PB Mgr (T-LensProducer-Retirement scope).

---

## §5. Bridge inventory snapshot

**Source-of-truth**: [`docs/audit/r3-debt-sweep-2026-05-06.md`](audit/r3-debt-sweep-2026-05-06.md) (in this PR — PR #1804 superseded; sweep framework consolidated). PM-authored Phase 1 framework + §4 anticipation discipline + §3 GREEN/YELLOW/RED rubric live in the audit doc; Phase 3 compile populates per-row inventory.

**Status (2026-05-06)**: per-class counts below are PRE-Phase-3-compile expected ranges from Director's bridge-class framework. Phase 2 partial deliverables landed as PR comments on the now-superseded PR #1804 comment thread (compiler-core `.rs` audit / emit subdir audit / v3 std `.dag` mirror audit / R2-R3-era PR debt history); **Phase 3 compile** folds those historical comment-thread deliverables into the audit doc in this PR, populating verified-at-HEAD counts before plan PR opens (per Director poke-hole 2026-05-06 finding 6.3).

**Apples-to-apples count discipline** (per `feedback_corrections_must_grep_verify_source` + INVARIANTS P1 single-authority):

- **66 `.rs` files** in `src/v3/compiler/src/` (excluding `emit/`) — file-level scope of quiet-boar-160's per-file audit work (Class A-G assignment per audited file). Filesystem count.
- **41 SG-0 NON_TEST entries** for the same scope (`src/v3/compiler/src/` excluding `emit/`) per [`src/v3/compiler/tests/integration/sg0_census_test.rs`](../src/v3/compiler/tests/integration/sg0_census_test.rs) `EXPECTED_HAND_AUTHORED_NON_TEST` filter. SG-0 ratchet authority for this scope.
- **66 - 41 = 25 file-system files NOT in SG-0 NON_TEST**: generated, build artifacts, or otherwise outside SG-0 hand-Rust ratchet scope.
- **SG-0 census authority full scope** (broader than this scope): `EXPECTED_HAND_AUTHORED_NON_TEST` = 46 paths total / `EXPECTED_HAND_AUTHORED_TEST` = 89 paths / `EXPECTED_HAND_AUTHORED_FRAGMENTS` = 1 path; **total = 136 paths**.

The 66-file audit is per-file classification work distinct from SG-0 census authority (path inclusion). Both counts are useful for different purposes: 66 = audit scope; 41 = SG-0 ratchet for that scope; 136 = SG-0 ratchet total.

Plus 35 v3 std mirror entries + 112 R2/R3-era PRs (Phase 2 audit work landed as PR comments on now-superseded PR #1804 comment thread; Phase 3 compile folds into the audit doc in this PR).

This plan section reports cumulative bridge counts; **the audit doc in this PR (`docs/audit/r3-debt-sweep-2026-05-06.md`) holds the schema + framework** — Phase 3 compile populates per-row inventory.

**Phase 2 inventory shape** (counts populate post-canvas-completion; **expected ranges** below are pre-Phase-2-completion estimates from Director's bridge-class framework, NOT compiled facts — Phase 3 compile of the audit doc in this PR folds historical Phase-2 deliverables from PR #1804 comment thread into authoritative per-row inventory):

- **Class A (substrate-gap-blocked)**: expected range ~15-20 bridges; awaiting Mgr canvas enumeration.
- **Class B (Pattern-A NYI predicates)**: count = 7 (this count IS authoritative — predicates pre-named in framework): TC1, TC2, TC3, free-consequences, RustDagIsomorphism, BridgeLedgerZero, SymbolicCostExprEquals.
- **Class C (Pattern-C typed-carrier + Rust-mirror)**: expected range ~6+ — EmissionDiagnostic, Value, EvalStrategy, BridgeLedgerRef, target-primitive routing pre-named; others awaiting canvas.
- **Class D (generated bridges with freshness gates)**: expected range ~3-5 — `render_repeat_string_bootstrap.dag`, `generated_method_template_projection.rs` pre-named; others awaiting canvas.
- **Class E (v2/v3 transition)**: expected range covered by T-V2-Retirement scope (not enumerated per-bridge here; v2 retirement closure subsumes).
- **Class F (operator-ontology-duplication)**: expected range — awaiting canvas.
- **Class G (local-small)**: expected range — distributed across lanes; awaiting canvas.

**Authoritative counts populate via Phase 3 compile of [`docs/audit/r3-debt-sweep-2026-05-06.md`](audit/r3-debt-sweep-2026-05-06.md) §1.A–§1.G in this PR** (Phase 2 partial deliverables on now-superseded PR #1804 comment thread fold in during Phase 3). Net-bridge count at R3 close is 0 (per `bridge_retirement_ledger_zero` + 5 substrate-gap-class closure gates). Each bridge has a named dissolution trigger; bridges without triggers escalate to substrate gap requiring named R3 lane (per audit doc §4 anticipation discipline).

**Drift items from Phase 2 sweep** (per Grounding Mgr poke-hole 2026-05-06 finding 5 normalization — earlier list double-counted #1638/declaration_by_name as same issue + omitted CollectionOps drift):

- **`declaration_by_name` ROADMAP↔ledger drift**: ROADMAP says `declaration_by_name(...) pattern in emit` retired by PR #1638; `docs/debt/r3-debt-paydown-ledger-2026-05-02.md` still marks `declaration_by_name(...) emit pattern` Open. Single-row reconciliation needed.
- **#1499 transitional fence ledger-row gap**: ROADMAP transitional-fence retirement not reflected in ledger row.
- **CollectionOps / StringOps / MapOps stale ledger refresh**: ROADMAP records CollectionOps fold + map progress, but ledger row says `Partial (fold)` and text says `concat / length / map` remain open. Ledger refresh needed.

---

## §6. Dependency DAG — current → close

**Authority**: `r3-structure.md` §"Dependency DAG" holds canonical dependency shape.

This section adds **forward-looking critical-path pointer**: from current state (2026-05-06) to R3 close.

```
NOW (2026-05-06)
   │
   ▼
T-E-P-Producer-Broadening  ────────────┐
   │ (Substrate, in flight)             │
   ▼                                    │
T-Lens-Behavioral-Parity                │  Parallel:
   │ (Substrate + Verification)          │  - T-FixedPoint (PB) ─────────────┐
   ├──┐                                  │  - T-LensProducer-Retirement (PB) │
   │  │                                  │  - T-CostLens-Composition         │
   │  │ (parallel post-LBP COMPLETE)     │  - T-Tests-As-Data-Completeness   │
   ▼  ▼                                  │  - T-Numeric-Construction ────────┼──┐ (parser-syntax
T-Lens-Application-Surface               │  - T-Anthropic-Wire               │  │  blocker for v2)
T-Workflow-As-Data                       │                                   │  │
   │  │                                  │                                   │  │
   ▼  ▼                                  │                                   ▼  ▼
T-Lens-Self-Application                  │                              T-V2-Retirement (PB)
   │ (Verification)                      │                                   │
   ▼                                     │                                   ▼
substrate-gap-class 3 + 4 closed         │                          v2_directory_deleted gate
                                         │                                   │
                                         ▼                                   ▼
                              Class 1 (parser) closes via T-V2-Retirement parser path
                              Class 2 (function-valued) closes via T-Lens-Application-Surface
                              Class 5 (reflection) closes via T-LensProducer-Retirement

Bridge-Retirement Ledger (Verification gate):
   - 2 Substrate-owned + 3 PB-owned bridges retire as natural-owner programs close
   - T-FixedPoint completion is upstream prerequisite (PB-owned bridges retire alongside)
   - bridge_retirement_ledger_zero gate fires when count = 0

Pattern A executable (4-family cluster — see §1.1):
   - DimensionReport-typed family (TC1/TC2/TC3): first slice via Q-PAFS (Director countersignature pending)
   - Remaining families (RustDagIsomorphism / SymbolicCostExprEquals / BridgeLedgerZero / Free-Consequences)
     have separate unblock paths per §2.1

R3 close = all 70 gates GREEN + r3_debt_paydown_zero_remaining + comprehensive sweep zero-debt-rows-remaining
```

**Edges added in §6 graph per Director poke-hole 2026-05-06**:
- T-Numeric-Construction → T-V2-Retirement (parser/grammar substrate path; finding 2.1)
- T-FixedPoint → Bridge-Retirement Ledger (PB-owned bridge retirement cascades; finding 2.2)
- T-Lens-Application-Surface and T-Workflow-As-Data parallel post-LBP COMPLETE (NOT sequential; finding 2.3 — both gated by T-Lens-Behavioral-Parity, neither depends on the other)

**Two longest-dependency paths** (per `r3-structure.md` §"Dependency DAG" — Verification Mgr poke-hole 2026-05-06 surfaced this dual structure was missing from earlier framing):

**Global critical-path chain** (substrate-completion + recursive-flex commit):

1. T-E-P-Producer-Broadening (Substrate, foundational)
2. → T-Lens-Behavioral-Parity (cross-program, 4 sub-slices)
3. → **(T-Lens-Application-Surface PARALLEL T-Workflow-As-Data)** — both gated by T-Lens-Behavioral-Parity COMPLETE; neither depends on the other (per Director poke-hole 2026-05-06 finding 2.3 + openai-pro 2026-05-06 finding 2)
4. → T-Lens-Self-Application (Verification, demonstration; depends on BOTH T-Lens-Application-Surface AND T-Workflow-As-Data)

**Verification-internal critical path** (longest verification path; parallel to global chain):

1. T-Verification-L4-L7-Direct (post-R2-Evaluator + R2-T-Substrate-Lens-Primitive + Shape A grounding for L4 corpus)
2. → T-Verification-L5-Corpus (consumes L4 corpus + R2-Grounding-Rust + R2-Grounding-Python)

Verification-internal path runs in parallel with global chain post-R2-Evaluator. Plus parallel longest single-lane path: **T-V2-Retirement** depends on T-FixedPoint + T-LensProducer-Retirement.

**Grounding-side dependency branches** (per Grounding Mgr poke-hole 2026-05-06 finding 4 — closes G10 by surfacing Grounding-driven re-dispatch edges in §6):

| Upstream node | Grounding trigger | Worker-ready dispatch | Close predicate |
|---|---|---|---|
| PR-F (`BoundDeclaration` consumer + Rust `ReferenceModel<T>`) | Substrate Mgr lands carrier | T-Ground-Rust Phase 1 (`u128`, `isize`, `usize`, walker arms, pilot mirror) | full Rust primitive grounding + L5 readiness |
| `EmissionPathProjection` carrier (Substrate; per `docs/briefs/r3-substrate-l6-per-row-projection-routing-decision.md`) | Substrate Mgr lands carrier | L6 CrossTarget-Meta row population + `coverage.rs` conversion | L6 cross-product fold gate |
| Variant-aware typed REST response projection carrier (Substrate) | Substrate Mgr lands carrier | #1702 Anthropic re-dispatch | T-Anthropic-Wire close + Q-Anthropic-Variant-Aware resolve |
| Real LanguageSpec projection (Substrate / LanguageSpec) | LanguageSpec projection executable | Coercion-Fold `ScratchIntExamples` + `TargetInhabitance` retirement in `src/v3/grounding_coercion_fold` | Q-Coercion-Fold-Scratch close |
| F10 cleanup (`dsl/extdeps/tools.dag` `install_hint`) | next Grounding signal PR or explicit small cleanup PR | Grounding worker dispatches | M5 cleanup close |

These edges parallel the global + Verification critical paths; Grounding lane consumers wait on upstream Substrate carrier landing.

**Estimated R3 close**: 8-12 weeks from 2026-05-06, gated on T-V2-Retirement + T-Lens-Self-Application both completing. **PROVISIONAL** (per Director poke-hole 2026-05-06 finding 6.4).

---

## §7. PR-authoring contract

Per R3 Debt-Paydown standing program (`r3-structure.md` §"Standing program — R3 Debt-Paydown" + INVARIANTS §P5):

### §7.1 Per-PR debt-receipt

Every R3 PR includes a "debt receipt" naming:
- **Debt found**: any debt observed in this PR's surface area (named ROADMAP row, named bridge, named NYI predicate).
- **Debt paid**: any debt retired by this PR (named row + ledger receipt OR explicit "no debt paid this PR").
- **Routing**: if debt found but not paid, names paydown-lane retirement PR or dispatch issue where it routes.

**Vague deferrals rejected**: "see ROADMAP", "TBD", narrative without cited row do not satisfy the gate.

### §7.2 Ratchet-only-down (SG-0 zero-floor)

Hand-Rust additions follow `r3-debt-sweep-2026-05-06.md` §4 anticipation discipline:
- **Default**: PRs that net-add hand-Rust violate the SG-0 zero-floor.
- **STRUCTURAL exception**: requires explicit Director allocation citing the program shape; Mgr cannot self-classify as STRUCTURAL.
- **Grandfathering**: contract applies to PRs landing AFTER framework merges; pre-existing entries grandfathered per Director ratification 2026-05-06 Refinement A.

### §7.3 Anticipation discipline

Per `r3-debt-sweep-2026-05-06.md` §4:
- New bridges introduced require named dissolution trigger at PR-merge time.
- Bridges introduced without trigger → blocking.
- "Eventually" / "when X is ready" language without named program → blocking.

### §7.4 Section/symbol anchors over line numbers

Per `feedback_section_anchors_over_line_numbers` (validated PRs #1776, #1787, #1788): all cross-references prefer section/symbol anchors. Line numbers permitted only when no symbol name exists at the cited point.

### §7.5 Grep-verify against source authority

Per `feedback_corrections_must_grep_verify_source`: any correction-of-a-relay must grep-verify against source authority before relaying. Cross-references aren't verification.

---

## §8. Cross-Mgr coordination

### §8.1 Cross-Mgr queue

Standard cross-Mgr queue per `r3-structure.md` §"Manager structure": blockers route through Director when:
- Cross-program scope conflict (e.g., who owns a bridge straddling two lanes)
- Lane definition disagreement
- Closure-gate-predicate disagreement

### §8.2 Escalation paths

| Block type | Routes to | Cadence |
|---|---|---|
| Within-lane design question | Lane-owning Mgr | Same-day |
| Cross-lane scope conflict | Director | T+1 day |
| Closure-criteria refinement | Brian via Director | T+1-3 days |
| Substrate carrier shape | Substrate Mgr → Director if not local | T+1 day |
| Verification predicate shape | Verification Mgr → Director if not local | T+1 day |
| Drift item (ROADMAP↔ledger) | Debt-Paydown Mgr | T+1-3 days |

### §8.3 Director cadence

Per `feedback_director_30min_cadence`: every 30 min — merge mergeable + account workers + queue more work; don't let idle Mgrs sit. Plan reads as input; no per-canvas framing invention.

---

## §9. Cadence + milestone schedule

### §9.1 Weekly cadence

- **Monday**: lane-status table refresh; PM compiles from Mgr per-lane reports.
- **Wednesday**: closure-gate progress check; new gate-passes ratched into ledger.
- **Friday**: bridge-ledger receipt cadence; any new bridges introduced get dissolution-trigger receipts.

### §9.2 Milestones (target dates)

**Week of 2026-05-06–2026-05-13**: All Mgr canvases compile; Q1–Q7 ratifications close; plan PR opens.
**Week of 2026-05-13–2026-05-20**: T-E-P-Producer-Broadening completes; T-Lens-Behavioral-Parity 4-slice work begins.
**Week of 2026-05-27–2026-06-03**: T-FixedPoint completes (PB); T-LensProducer-Retirement begins close.
**Week of 2026-06-10–2026-06-17**: T-V2-Retirement parser path begins; substrate-gap-class 1 close target.
**Week of 2026-06-24–2026-07-01**: T-Workflow-As-Data executes; substrate-gap-classes 3+4 close.
**Week of 2026-07-08–2026-07-15**: T-Lens-Self-Application demo; recursive-flex demonstration cashes.
**Target R3 close**: 2026-08-01 ± 2 weeks. **PROVISIONAL** (per Director poke-hole 2026-05-06 finding 6.4) — estimate based on Mgr-canvas inputs landing per §3, which is incomplete pending 4 canvas responses (Substrate / Evaluator / PB / Debt-Paydown). Re-estimate after canvas compile completes; confidence interval may widen.

(Refinement after Q1–Q7 ratification + canvas compile.)

---

## §10. Open ratification questions / escalation register

**Status**: live. PM compiles per-Mgr canvas escalations as they arrive; surfaces consolidated list to Brian + Director for resolution. Plan PR opens after all blocking items resolved.

### §10.1 PM-surfaced ratifications (Q1–Q7 from gunbc#846 reply 2026-05-06)

**Status (2026-05-06): all RATIFIED per Brian directive — *"I'll take your recommendations unless controversial on all Q1-Q7."*** Plus **demonstration principle** (every feature comes with a demonstration) ratified — see §1.6.

| # | Question | PM recommendation | Status |
|---|---|---|---|
| Q1 | Closure-criteria predicate set (§1.1–§1.4) | Predicates per §1 tables; 70 closure gates total (54 existing R3 lane gates + 5 substrate-gap-class + 10 demonstration + 1 PR-anticipation-discipline; precise enumeration per §1.5) | RATIFIED |
| Q2 | 5 substrate-gap classes — separate lanes vs closure-criteria-over-existing-lanes | Closure-criteria framing (§1.4 + §4) | RATIFIED |
| Q3 | v2 retirement Mgr ownership | PB Mgr owns all (existing T-V2-Retirement scope) | RATIFIED |
| Q4 | Mgr structure (keep 9 vs reorganize) | Keep 9 standing Mgrs with re-anchored scope | RATIFIED |
| Q5 | Free-consequences-demonstration as closure gate | Required as gate; **generalized to demonstration principle** (§1.6) — every feature comes with a demonstration | RATIFIED + EXTENDED |
| Q6 | Plan-as-doc shape (this doc vs r3-structure.md merge) | New doc (this); cross-link both directions | RATIFIED |
| Q7 | Drift items reconciliation routing (#1638, #1499, declaration_by_name) | Single Debt-Paydown worker reconciles all three → one PR | RATIFIED |

### §10.2 Per-Mgr canvas escalations (compiling as canvases land)

#### §10.2.1 R3 Verification Mgr (cool-owl-579, #1740) — 6 escalations (full canvas absorption)

**V1. Pattern A (DimensionReport-typed family) blocks executable / net bridge shrink** (canvas item 1, structural)

**Note** (Verification Mgr poke-hole 2026-05-06): V1's DimensionReport-unblock fixes TC1/TC2/TC3 cluster but does NOT automatically fix **V2** (SymbolicCostExprEquals — different predicate family, `SymbolicCost`-typed, distinct runner work) NOR **BridgeLedgerZero** (Pattern E ledger-count ratchet, separate retirement-work cascade). Each Pattern-A predicate family has its own unblock path per §1.1 + §2.1.

**V1 (continued)**:
- **Needs**: Director + Brian — which first executable slice (TC1 vs RustDagIsomorphism vs other) + lane priority.
- **R3 zero-debt impact**: Must be explicit pre-close. Cannot be "post-R3" without renaming the close predicate.
- **Plan integration**: §1.1, §2.1; ratification opens Q-PAFS in §10.3.

**V2. SymbolicCostExprEquals reads as coverage while runner returns NYI** (canvas item 2, structural)
- **Needs**: coverage/completeness accounting decision — mark NYI claims incomplete in aggregation OR schedule evaluator-backed comparison slice.
- **Plan integration**: §1.1; ratification opens Q-NYI-Accounting in §10.3.

**V3. Evaluator wall vs Verification cross-program lanes — closure-criteria alignment** (canvas item 3, structural)
- **Description**: For each Verification acceptance row, decide which predicates must execute in R3 vs remain honest NYI with explicit incomplete classification until evaluator lands. Risk: harness-heavy progress (`.dag` fixtures + `run_claim` / integration tests) proceeds while runtime-complete demonstrations remain blocked where predicates require evaluator execution.
- **Needs**: Director + R3 Evaluator Mgr + PM — closure-criteria scope decision per acceptance row.
- **R3 zero-debt impact**: must NOT claim R3 Verification "complete" while cross-program rows silently wait on evaluator; either shrink scope or extend close predicate.
- **Plan integration**: §1.6 demonstration principle minimum bar (per Director finding 1.1) addresses partially; §10.3 Q-Verification-NYI-Carry-Forward names the carry-forward classification decision.

**V4. Worker idle surfaces — bandwidth, not substrate bug** (canvas item 4, bandwidth)
- **Description**: bold-crane-790 + cool-heron-521 idle with no active PR after CI/ratchet churn; execution-ready Verification slices not queued at Mgr granularity.
- **Needs**: PM §3 dispatch with PR-sized specs (files + acceptance + ROADMAP row), not open-ended "continue R3".
- **R3 zero-debt impact**: bandwidth risk if plan assumes parallel workers without packaged tasks.
- **Plan integration**: addressed by §3 lane-status snapshot + §9 weekly cadence (lane-owning Mgr packages PR-sized specs); follows from Director cadence pattern.

**V5. SB5+6 PR-history (PR #1804) — heuristic pass ≠ closed audit** (canvas item 5, reporting)
- **Description**: Phase-2 R2/R3-era PR debt table on PR #1804 is path-filtered + column heuristics; not yet per-PR body/ledger grep with row-by-row inversion-test discipline.
- **Needs**: PM defines next pass — widen/narrow path union vs require PR-body debt citations for every row.
- **R3 zero-debt impact**: reporting debt if misread as "verified"; must NOT become a hidden "we closed history" claim.
- **Plan integration**: §5 already labels Phase 2 partials as "expected ranges; Phase 3 compile populates verified counts" (per Director finding 6.3); V5 reinforces — Phase 3 compile must apply per-PR body/ledger grep discipline, not re-import heuristic table.

**V6. ValueBody Rust↔.dag mirror drift / missing isomorphism gate** (canvas item 6, RED-adjacent)
- **Description**: ROADMAP records novel internal-taxonomy drift risk: live Rust `ValueBody` vs `.dag` `ValueBody*` mirror mismatch with no CI isomorphism gate. Tests can pass while mirror diverges.
- **Needs**: Director + R3 Substrate Mgr (mirror authority) + Brian — is structural-isomorphism / conformance gate in R3 close, or explicit post-R3 carry with named owner?
- **R3 zero-debt impact**: HIGH if "zero internal fiction" is literal R3-close definition. Verification cannot unilaterally close this.
- **Plan integration**: §10.3 Q-ValueBody-Isomorphism — escalation requires Substrate Mgr authority to scope.

#### §10.2.2 R3 Grounding Mgr (bold-ferret-748, #1745) — 5+ escalations

**G1. T-Ground-Rust blocked on PR-F**
- **Needs**: R3 Substrate Mgr to execute PR-F (Q1 BoundDeclaration consumer + Q2 Rust ReferenceModel<T>).
- **Plan integration**: §3 lane status; routes to Substrate canvas response.

**G2. Rust float rows blocked on T-NumericConstruction-ApproximateField**
- **Needs**: Float migration to `ApproximateField<F>` + Real/base-carrier convention.
- **Plan integration**: §3; routes to Substrate canvas (T-Numeric-Construction lane).

**G3. Rust std-carrier conditional gates need plan placement**
- **Items**: `Option<T>` parent vs `OptionalOf` promotion; `Cardinal` ordered-domain instance; `HigherOrderMethodSpec`; allocator/hasher carrier policy.
- **Needs**: pre-Phase-1 vs later-T-Ground-Rust-phases vs separate Substrate-prerequisite-slice decision.
- **Plan integration**: §10.3 Q-Std-Carrier-Phasing.

**G4. L6 CrossTarget-Meta held on `EmissionPathProjection`**
- **Needs**: Substrate to land sibling `EmissionPathProjection` carrier in `src/v3/std/cross_target_coverage.dag`.
- **Plan integration**: §3; routes to Substrate canvas.

**G5. Anthropic #1702 parked on orphaned Substrate dependency**
- **Description**: #1702 work preserved on branch `codex/cc1-target-integer-structural-fold`; merge blocked by missing structural projection metadata / variant-aware typed REST response `from`-path resolver.
- **Needs**: Substrate carrier/resolver for variant-aware typed REST response projection against coproduct response bodies; then re-dispatch Anthropic Messages 200 content-block work.
- **R3 zero-debt impact**: R3-blocking if Anthropic wire is in zero-debt close scope.
- **Plan integration**: §10.3 Q-Anthropic-Variant-Aware.

**G6. LanguageSpec / emit shim debt depends on SG-7/PB-6/v2 retirement**
- **Description**: PR #1804 §1.2 classified `emit/rust_target.rs`, `emit/python_target.rs`, `collection_ops_method_contract.rs` as YELLOW class-E (not RED) — persistence tied to named SG-7/PB-6/v2-retirement prerequisites.
- **Needs**: R3 plan must state whether Grounding owns direct execution here or only consumes PB/Substrate/v2-retirement milestones.
- **R3 zero-debt impact**: must resolve before zero-debt close if hand-Rust target emitters unacceptable at R3 close.
- **Plan integration**: §10.3 Q-Emit-Shim-Handoff.

**G7. Coercion-Fold `ScratchIntExamples` bridge live**
- **Description**: PR #1804 §2 verified `LanguageSpecProjection::ScratchIntExamples` and `TargetInhabitance` remain live in `src/v3/grounding_coercion_fold`; trigger "real LanguageSpec projection".
- **Needs**: lane placement decision — Grounding-owned after LanguageSpec projection lands, OR Substrate/LanguageSpec owner defines projection read surface first.
- **R3 zero-debt impact**: must close before zero-debt if Coercion-Fold scaffolds count against R3 closure.
- **Plan integration**: §10.3 Q-Coercion-Fold-Scratch.

**G8. Small cleanup F10 `install_hint` queued, not dispatched**
- **Description**: `dsl/extdeps/tools.dag` `install_hint` uses `fold(init: "")` + `acc == ""` first-element sentinel instead of standard join semantics.
- **Needs**: bundling decision — bundle into next Grounding signal PR vs small cleanup PR (despite `feedback_brief_pr_cadence`).
- **R3 zero-debt impact**: small but real M5 cleanup; resolve before zero-debt close unless explicitly ruled out.
- **Plan integration**: PM-level (PM proceeds with bundle-into-next-signal); §10.3 informational.

**G9. Worker-side idle state (proud-lark, silent-badger)**
- **Description**: Both workers idle post-#1804 sweep; proud-lark owns #1783 draft feedback; silent-badger has no active PR.
- **Needs**: execution-ready Grounding tasks from R3 plan (PR-F-dependent / L6 post-EmissionPathProjection / Anthropic post-projection-metadata / F10 cleanup).
- **Plan integration**: §3 lane status + §9 cadence; resolves once upstream substrate decisions land.

**G10. Structural concern — cross-manager dependencies must be explicit in plan**
- **Description**: Several Grounding closure predicates depend on Substrate/PB/v2-retirement milestones before Grounding can execute final consumers; plan must encode cross-manager dependency edges + re-dispatch triggers, not just assign Grounding outcomes.
- **Needs**: PM + Director + Brian — final plan shape with explicit cross-Mgr dependency edges.
- **R3 zero-debt impact**: main RED risk — if upstream carrier decisions are not scheduled as prerequisite plan nodes, Grounding cannot honestly close by R3.
- **Plan integration**: this plan §6 Dependency DAG + §3 lane status (cross-Mgr dependency edges) directly addresses this; G10 closes when §6 graph is Mgr-validated.

#### §10.2.3 R3 Substrate Mgr (quick-crab-830, #1739) — full canvas absorbed 2026-05-06T01:14Z

Substrate Mgr's full forward-looking escalation canvas (per gunbc#846 #issuecomment-4384149022) — substantial: 6 design decisions held + 4 architectural questions + 3 structural concerns including 1 RED.

**Section A — Currently-blocked items**: A1 = none. Clean.

**Section B — Design decisions held (all need Director/PM ratification)**:
- **B1. Slice C scope sub-shape**: prose+regen-bundling vs separate sub-slice carve-out for `.dag` substrate files in #1795 follow-up. → Q-Slice-C-Shape
- **B2. T-Numeric-Construction parallel-author posture**: Mgr-tier brief authoring authorization vs sequence post-:425. → Q-Numeric-Construction-Trigger
- **B3. T-E-P-Producer-Broadening dispatch trigger**: brief drafted (PR #1782); dispatch trigger needed (anytime vs gate). → Q-T-E-P-Dispatch-Trigger
- **B4. L6 EmissionPathProjection dispatch trigger**: brief drafted; cross-Mgr handoff to Grounding wired; dispatch trigger needed. → Q-L6-Dispatch-Trigger
- **B5. X1.b S1 TransformDispatch dispatch trigger**: gated on E6-G0c merge from merry-gull-128 (#1743). → Q-X1b-S1-Dispatch-Coord
- **B6. F2 + F8 doc-sharpening PR scope**: bundle vs separate; Mgr vs worker dispatch. → Q-F2-F8-PR-Shape

**Section C — Open architectural questions (Director architectural calls)**:
- **C1. Variant-aware projection carrier closure scope**: carrier-only sufficient vs 3 follow-up paydowns required for "wire-typing zero-debt" — 3-4x scope difference. → Q-Variant-Aware-Closure-Scope
- **C2. T-E-P-Producer-Broadening cascade timing realism**: does R3 close gate on full cascade (T-E-P → T-LBP → T-LAS → T-WAD + T-LSA) or just T-E-P? Major R3-close-predicate calibration. → Q-T-E-P-Cascade-Scope
- **C3. Slice C → :425 retirement receipt**: who authors (Mgr-tier vs worker), single combined vs per-slice. → Q-Slice-C-Retirement-Receipt
- **C4. R3 Substrate / R3 Grounding boundary on `.dag` files**: Slice C residual `.dag` files that overlap Grounding scope. → Q-Substrate-Grounding-DAG-Routing

**Section D — Worker bandwidth**:
- 3 workers idle (loyal-wolf, valiant-ant, plus proud-lynx/smart-ram/valiant-ibex post-:425); awaiting plan-driven dispatch shapes.

**Section E — Structural concerns about R3 zero-debt close**:
- **E1**: 10 wait-window briefs in PR #1782 all R3 Substrate scope (4 :425-related + 6 independent substrate dispatches). All must dispatch before R3 close.
- **E2**: 6+ substrate carrier dispatches form serialized queue (cross-lane coord ratchet per dispatch). Cadence rate-limits R3 close.
- **E3 (RED — *"I don't know how to close this"*)**: T-Lens-Behavioral-Parity (PROXY → BEHAVIORALLY COMPLETE for 4 lenses) is L-XL sized, gates on T-E-P-Producer-Broadening (foundational), itself has 4 sub-slices per lens. **Honest assessment: this is the largest R3-Substrate-shaped substantive work and may not close cleanly in R3 timeline.** → Q-Lens-Behavioral-Parity-R3-Closeability (RED escalation)

**Plan integration**: 12 new escalations added to §10.3 covering B1-B6 + C1-C4 + E1+E2 + E3 (RED). E3 is the highest-leverage scope question — if T-Lens-Behavioral-Parity doesn't close in R3, the entire critical-path chain (T-E-P → T-LBP → T-LAS → T-WAD → T-LSA) is at risk.

#### §10.2.4 R3 Evaluator Mgr — pending canvas response (as-of 2026-05-06T00:30Z)

#### §10.2.4 R3 Evaluator Mgr — pending canvas response (as-of 2026-05-06T00:30Z)

#### §10.2.5 R3 PB Mgr — pending canvas response (as-of 2026-05-06T00:30Z)

#### §10.2.6 R3 Debt-Paydown Mgr — pending canvas response (as-of 2026-05-06T00:30Z)

(Per Director poke-hole 2026-05-06 finding 6.1: timestamp added to clarify these are point-in-time placeholders. Plan PR opens after canvas compile completes.)

### §10.3 Net escalations awaiting decision

**Status (2026-05-06): all PM-recommended defaults applied per Brian directive — *"I'll take your recommendations unless controversial."*** Marks remain in table for visibility; any item Brian or Director rules controversial flips back to OPEN.

| ID | Question | PM recommendation (applied as default) | Status |
|---|---|---|---|
| Q-PAFS | Which Pattern-A (DimensionReport-typed family) first executable slice (TC1 vs RustDagIsomorphism vs other) | **TC1 η-equivalence first as PM/Brian convenience default; needs Director countersignature** — `r3-v-tc1-eta-equivalence-deeper-analysis.md` is `Status: PROPOSAL / research-only` and explicitly does not choose among paths. PM-default ≠ Verification-Mgr engineering sign-off. | **PENDING DIRECTOR COUNTERSIGNATURE** (per Verification Mgr poke-hole 2026-05-06; PM convenience-default applies until then) |
| Q-NYI-Accounting | NYI claim coverage/completeness accounting | Mark NYI incomplete in aggregation reporting; ratchet to executable per evaluator-slice landing | RATIFIED-by-default |
| Q-Std-Carrier-Phasing | Rust std-carrier phasing (Option/Cardinal/HigherOrderMethodSpec/allocator-hasher) | Phase 1 = default-only/default-instance; later phases as separate Substrate slices post-Phase-1 | RATIFIED-by-default |
| Q-Drift-Reconcile | Drift items routing (#1638, #1499, declaration_by_name) | Single Debt-Paydown worker reconciles all three → one PR closing rows in ledger + ROADMAP | RATIFIED-by-default |
| Q-Pattern-Class-Naming | Substrate-gap-class TestClaim names | **Q-Pattern-Class-Naming ratifies §1.4 table names verbatim as TestClaim identifiers** (per Director poke-hole 2026-05-06 finding 5.1 — explicit binding). Names: `substrate_gap_parser_grammar_closed`, `substrate_gap_function_valued_data_closed`, `substrate_gap_file_ingestion_closed`, `substrate_gap_workflow_scheduling_closed`, `substrate_gap_reflection_closure_closed`. | RATIFIED-by-default |
| Q-R2-IB-Closure | #1778 R2 Impossible-Bugs queue closure (warm-dove-810; nested optional codegen bypass) | PM-coordination item tracked in Brian-Q queue at #828; remains Brian-disposition pending; not blocking R3 plan PR | TRACKED (per Director poke-hole 2026-05-06 finding 5.2) |
| Q-1807-Cleanup | #1807 SG-0 net-shrink discipline rollout — branch contains 5 of 9 files = #1797-resurrection per Director critique at #1807 #issuecomment-4383967713 | PM-coordination item; Director ratifies cleanup approach (Option A force-push vs Option B cherry-pick); affects §7.2 anticipation-discipline rollout via R3 Debt-Paydown lane | OPEN — Director decision needed (per Director poke-hole 2026-05-06 finding 5.3) |
| Q-PR-Anticipation-Gate | Add `pr_anticipation_discipline_ci_active` closure gate (CI verifiably enforcing §7 anticipation discipline) | Add to §1 closure-gate set; owner R3 Debt-Paydown Mgr (already routed at #1744 #issuecomment-4383628247); fires when `scripts/check-pr-sg0-net-shrink-discipline.sh` is in CI workflow + self-test passes | RATIFIED-by-default (per Director poke-hole 2026-05-06 finding 3.2) |
| Q-PR-F | PR-F priority + sequencing (BoundDeclaration consumer + Rust ReferenceModel<T>) | Schedule into critical path post-T-E-P-Producer-Broadening; Substrate Mgr executes PR-F as Substrate-lane work | RATIFIED-by-default |
| Q-Anthropic-Variant-Aware | Variant-aware typed REST response projection carrier shape | Defer #1702 re-dispatch until Substrate authors variant-aware projection metadata carrier; #1702 branch preserved | RATIFIED-by-default |
| Q-Emit-Shim-Handoff | Grounding direct execution vs PB/Substrate/v2-retirement consumption for `emit/rust_target.rs` family | Grounding consumes PB/v2-retirement milestones; no Grounding direct execution on these files | RATIFIED-by-default |
| Q-Coercion-Fold-Scratch | Lane placement for `ScratchIntExamples` retirement | Grounding owns close after LanguageSpec projection executable | RATIFIED-by-default |
| Q-MachineConstraint-Carrier | Substrate carrier for `MachineConstraint<C>` independent axis (per Brian directive 2026-05-06 — concrete types emerge from algebra × machine-constraint interaction; `i64` = `Int × MachineWidth<64>` consequence, NOT primary entity) | Substrate Mgr authors `MachineConstraint<C>` carrier + interaction lookup; folds into T-Numeric-Construction continuation (algebra side already in flight); surfaces as new Substrate sub-program. Required for Class 1 substrate-gap-class closure per §1.4. | OPEN — Substrate Mgr + Director scoping needed (NEW 2026-05-06 from Brian inline at PR #1808 line 69) |
| Q-Verification-NYI-Carry-Forward | Closure-criteria alignment for cross-program Verification rows: which predicates must execute in R3 vs remain honest NYI with explicit incomplete classification until evaluator lands? | Per §1.6 demonstration-gate minimum bar (Director finding 1.1): predicates without runtime evaluator execution canNOT claim "GREEN" closure; honest NYI classification with explicit carry-forward owner required. R3-Verification rows that depend on evaluator runtime that's NYI at R3 close → explicit NYI + named carry-forward Mgr (NOT silent assumption of evaluator landing). | RATIFIED-by-default per Director finding 1.1 minimum bar; closure-ledger reports separate "demonstration-executes" vs "static-fact" status per row (per §1.6). Verification Mgr ratifies per-row carry-forward classification at lane-close audit. |
| Q-ValueBody-Isomorphism | ValueBody Rust↔.dag mirror drift gate — structural-isomorphism / conformance gate in R3 close, OR explicit post-R3 carry with named owner? Mirror divergence risk + no CI isomorphism gate currently. | Per `feedback_isomorphism_or_generation_for_mirrors`: Rust↔.dag mirrors need generation-or-isomorphism-test, not hand-maintenance. R3 zero-debt-close framing implies isomorphism gate IN R3 scope — gate fires at R3 close requiring CI conformance test. Owner: R3 Substrate Mgr (mirror authority) + Verification Mgr (test-only pressure). | OPEN — Director + R3 Substrate Mgr + Brian scoping needed (NEW 2026-05-06 from Verification Mgr canvas item 6) |
| Q-Slice-C-Shape | Slice C `.dag` files in #1795 follow-up: prose+regen-bundling (path-(a) precedent) vs separate sub-slice carve-out | Path-(a) precedent from #1795 establishes regen-bundling-acceptable; default to single-PR prose+regen. | RATIFIED-by-default (Substrate Mgr B1) |
| Q-Numeric-Construction-Trigger | T-Numeric-Construction Mgr-tier brief authoring authorization vs post-:425 sequencing | Authorize Mgr-tier brief authoring now (parallel to :425 close); non-Evaluator-gated lane per r3-structure.md §"Dependency on R2". | RATIFIED-by-default (Substrate Mgr B2) |
| Q-T-E-P-Dispatch-Trigger | T-E-P-Producer-Broadening dispatch trigger (brief in PR #1782) | Dispatch anytime post-#1782 brief landing — foundational lane, no upstream prerequisite per r3-structure.md §"Dependency on R2". | RATIFIED-by-default (Substrate Mgr B3) |
| Q-L6-Dispatch-Trigger | L6 EmissionPathProjection dispatch trigger (brief in PR #1782) | Dispatch anytime post-#1782 brief landing — non-Evaluator-gated; can fire parallel to T-E-P. | RATIFIED-by-default (Substrate Mgr B4) |
| Q-X1b-S1-Dispatch-Coord | X1.b S1 TransformDispatch gate on E6-G0c merge (merry-gull-128 / #1743) | merry-gull cross-lane status update on E6-G0c required; Substrate Mgr standing by. | OPEN — Evaluator Mgr cross-lane status needed (Substrate Mgr B5) |
| Q-F2-F8-PR-Shape | F2 (`.v3` filename-suffix) + F8 (bootstrap load-order) doc-sharpening: bundle vs separate, Mgr-tier vs worker | Bundle into single Substrate Mgr-tier doc-sharpening PR (small scope; one PR per `feedback_brief_pr_cadence`). | RATIFIED-by-default (Substrate Mgr B6) |
| Q-Variant-Aware-Closure-Scope | Variant-aware projection carrier closure: carrier-only vs 3 follow-up paydowns required for "wire-typing zero-debt" | Per Brian "no post-R3 deferral" + R3 zero-debt close: ALL 3 follow-up paydowns (Anthropic + OpenAI ChatCompletion + OpenAI Responses) required for R3 close. 3-4x scope vs carrier-only. | OPEN — Director / Brian scope calibration (Substrate Mgr C1; significant scope-determination question) |
| Q-T-E-P-Cascade-Scope | T-E-P-Producer-Broadening cascade timing: R3 close gates on full chain (T-E-P → T-LBP → T-LAS → T-WAD + T-LSA) vs T-E-P alone? | Per r3-structure.md §"Lane structure" + Brian no-post-R3-deferral: R3 close requires the FULL cascade. T-Lens-Self-Application is the closure gate (recursive-flex demonstration); without T-LBP COMPLETE there's no consumer. | OPEN — Director / Brian scope calibration (Substrate Mgr C2 + Verification V3 closure-criteria alignment; reinforces Q-Verification-NYI-Carry-Forward) |
| Q-Slice-C-Retirement-Receipt | Slice C → :425 retirement receipt: Mgr-tier vs worker authoring; single combined vs per-slice | Mgr-tier authoring; single combined receipt covering Slice A+B+C (Brian-style condensation per `feedback_brief_pr_cadence`). | RATIFIED-by-default (Substrate Mgr C3) |
| Q-Substrate-Grounding-DAG-Routing | Slice C residual `.dag` files overlap Grounding scope (`docs/parallelism-design.md` etc.) — routing | Substrate Mgr owns Slice C residual entirely; cross-Mgr coord with bold-ferret-748 for Grounding-impacting `.dag` content via comment-thread review (no separate Grounding dispatch). | RATIFIED-by-default (Substrate Mgr C4) |
| Q-Substrate-Cadence-Throughput | 10 wait-window briefs in PR #1782 + 6+ substrate carrier dispatches form serialized queue; rate-limits R3 close | Authorize parallel-dispatch where lanes have no shared prerequisite (T-E-P + L6 + Numeric-Construction parallel; X1.b sequenced post-E6-G0c; coproduct paydowns sequenced post-variant-aware carrier). PM tracks cadence in §9 weekly cadence; lane-owning Mgr packages PR-sized specs. | RATIFIED-by-default (Substrate Mgr E1+E2) |
| Q-Lens-Behavioral-Parity-R3-Closeability | **RED**: T-Lens-Behavioral-Parity (PROXY → BEHAVIORALLY COMPLETE for 4 lenses) is L-XL sized + 4 sub-slices per lens + foundational-dependency on T-E-P-Producer-Broadening. Substrate Mgr honest assessment: may not close cleanly in R3 timeline. | Per Brian no-post-R3-deferral: T-LBP MUST close in R3 OR escalate as substrate-gap requiring named R3 lane. RED escalation needs Director / Brian scope calibration: (a) accept T-LBP scope as-is + extend R3 close horizon if needed; (b) reframe T-LBP scope (e.g., 1-2 lenses BEHAVIORALLY COMPLETE for R3 close, others post-R3 with substrate-gap routing); (c) escalate as substrate-gap requiring named R3 lane subdivision. | **OPEN — RED — Director / Brian scope calibration urgent** (Substrate Mgr E3; load-bearing scope-question for R3 close horizon) |

---

## §11. Plan supersession

This plan supersedes earlier ad-hoc PM coordination shapes:
- Comprehensive R3 debt sweep PR #1804 → input to this plan; doc closes when this plan PR opens.
- 6 escalation-canvas messages on Mgr inboxes #1739/#1740/#1742/#1743/#1744/#1745 → folded into §10.

This plan does NOT supersede:
- `r3-structure.md` (architectural-decision archive remains canonical for lane definitions).
- `INVARIANTS.md` / `CODING.md` / `TESTING.md` / `MODELING.md` (governing rules).
- Per-lane briefs in `docs/briefs/r3-*` (worker-level dispatch sources).

Plan updates: weekly Mgr canvas refresh (per §9.1); structural revisions require Director ratification.

---

## Cross-refs

- `r3-structure.md` — architectural-decision archive
- `audit/r3-debt-sweep-2026-05-06.md` — bridge inventory
- `audit/r3-dispatch-brief-drift-sweep-2026-05-05.md` — Tier 1 sweep audit
- `INVARIANTS.md` §P5 — Progress Is Dissolution
- `ROADMAP.md` — debt rows + course corrections
- `THESIS.md` — closed-system + free-consequences thesis
