# R3 Program Plan — 2026-05-06

**Status:** **PR #1808 OPEN; iterating.** Plan is the live authority for path-to-R3-close. Phase 1 (skeleton + closure criteria + lane status) authored; Phase 2 (per-Mgr escalation absorption) ongoing — 5 of 8 reviewers complete (Verification + Director + Grounding + Substrate + Debt-Paydown); 3 pending (PB / Evaluator / Research PM). Q1–Q7 ratified per Brian directive 2026-05-06; live RED escalations remain in §10.3 (most notably Q-Lens-Behavioral-Parity-R3-Closeability requiring Director scope-calibration).

**Merge gates** (distinct from PR-open status — per openai-pro 2026-05-06 finding):
- **PR open**: ✓ at sha `d1bfbbe22`
- **Plan PR merge-eligible**: open RED escalations in §10.3 acknowledged + tracked (not necessarily resolved); PR can merge with RED items in flight provided they're explicitly tracked + assigned routing
- **R3 close** (NOT plan PR merge): all 75 closure gates GREEN + `r3_debt_paydown_zero_remaining` + comprehensive sweep zero-debt + Director resolution on Q-Lens-Behavioral-Parity-R3-Closeability scope + other §10.3 RED items resolved per their owners

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
| 2 — function-valued data | `substrate_gap_function_valued_data_closed` | `Lens<C>` instance with function-typed payload executes through evaluator | T-Lens-Application-Surface + T-E-P-Producer-Broadening — **chain-break per YELLOW chain rule** (Substrate Mgr 2026-05-06): prereq T-Lens-Application-Surface ← T-Lens-Behavioral-Parity ← Q-Lens-Behavioral-Parity-R3-Closeability (RED). Class 2 inherits RED until LBP scope-calibration resolves. See Q-Class-2-Chain-Break in §10.3. |
| 3 — file-ingestion | `substrate_gap_file_ingestion_closed` | One `.dag` program ingests external file (e.g., timing observation set) without `include_str!` | T-Workflow-As-Data |
| 4 — workflow/scheduling | `substrate_gap_workflow_scheduling_closed` | CI workflow modeled as `.dag` data executes through evaluator | T-Workflow-As-Data + T-Lens-Self-Application |
| 5 — reflection-closure | `substrate_gap_reflection_closure_closed` | `lens_apply.rs` reflection work routes via PB-Runtime interpreter-as-data | T-LensProducer-Retirement |

**Q2 status (per §10.1)**: RATIFIED 2026-05-06. All gap-test selections per-class take PM recommendations per Brian directive (*"take recommendations unless controversial on all Q1-Q7"*). No outstanding ratification gap.

### §1.5 Closure-ledger total

**Total R3 closure gates** (post-Q1 + Q2 ratification): **75 gates** at this commit per `r3-structure.md` §"Acceptance" enumeration. Composition: existing R3 lane gates (`T-Tier3-Dissolution` 4, `T-LensProducer-Retirement` 4, `T-V-L4-L7-Direct` 2 + 4 NEW Pattern-A executable (TC1/TC2/TC3 + RustDagIso) = 6, `T-V-L5-Corpus` 1, `T-FixedPoint` 1, `T-Numeric-Construction` 8, `T-Omni-Shape-B` 4, `T-Anthropic-Wire` 2, `T-Bridge-Retirement` 6, `T-CostLens-Composition` 3 + 1 NEW Pattern-A executable (SymbolicCostExprEquals) = 4, `T-V2-Retirement` 2, `T-Free-Consequences-Demonstration` 10, `T-Workflow-As-Data` 4, `T-Lens-Self-Application` 3) = 59 lane gates; plus 5 substrate-gap-class gates + 10 demonstration gates + 1 PR-anticipation-discipline gate = 16 gates added 2026-05-06 in this PR. Total: 59 + 16 = **75** across 18 lanes + 1 standing program.

**Per codex BLOCKING 2026-05-06 inline at line 79**: prior 70-count omitted the 5 NEW Pattern-A executable gates (`tc1_eta_equivalence_executable`, `tc2_church_rosser_executable`, `tc3_pattern_a_second_mover_executable`, `rust_dag_isomorphism_executable`, `symbolic_cost_expr_equals_executable`) declared in §1.1 but not yet in `r3-structure.md` §"Acceptance". This commit adds them to the canonical authority and bumps total from 70 → 75; single-authority restored (INVARIANTS P2/P5).

R3 closes when ALL gates pass + zero tracked-debt rows survive (`r3_debt_paydown_zero_remaining`).

**Two distinct Pass surfaces** (per Debt-Paydown Mgr poke-hole 2026-05-06 — clarification prevents conflating predicates):
- **Lane `.dag` TestClaim gates (75 total)**: per-lane closure predicates passing via `.dag` evaluation, runtime demonstration, or CI consumer (per §1.7 status taxonomy).
- **`r3_debt_paydown_zero_remaining`**: standing-program ledger predicate — no tracked ROADMAP debt rows survive R3 close (per `r3-structure.md` §"Standing program — R3 Debt-Paydown" + §1.5 tracked-debt inclusion list).

Both must hold for R3 close. "75 gates green" alone does not satisfy zero-debt; "zero debt rows" alone does not satisfy lane closure.

**Tracked-debt inclusion list for `r3_debt_paydown_zero_remaining`** (per Director poke-hole 2026-05-06 finding 3.1; closes definition gap):

- **Counts toward zero**: ROADMAP open `- **` rows under "Post-merge debt" sections; sweep-doc §1 Class A/B/C/F/G entries (per §1.3 inclusion criteria); §10 RED-flagged escalation items.
- **Excluded by-construction**: PM-coordination items (Brian-Q queue, dispatch routing); scheduled-deletions-with-named-trigger (the trigger IS the resolution); retirement-receipts (post-resolution accounting); Class D bridges (bounded-scaffold freshness gates) and Class E bridges (v2/v3 transition — retires at v2 retirement which has its own gate).

Predicate cannot be satisfied by deletion-of-rows-without-actual-resolution; each retiring row requires named PR receipt OR closure-ledger entry citing the structural event that ends the row's existence.

### §1.7 Closure-criteria status — DECLARATIONS-ONLY staging (per openai-pro meta-review 2026-05-06)

**This plan + r3-structure.md DECLARE 75 closure gates. The declarations are NOT YET load-bearing closure predicates — no consumer infrastructure verifies them at HEAD.**

Per openai-pro meta-review on PR #1808 sha `cf249389` ([#issuecomment-4384405832](https://github.com/gunb-ai/gunbc/pull/1808#issuecomment-4384405832)) PAUSE_AND_REGROUP verdict — structurally honest framing of where this PR sits in the closure-evidence chain:

**Per-gate status taxonomy:**

- **DECLARED** — gate id + Pass condition exists in `r3-structure.md` §"Acceptance" or this plan §1. Document-level only.
- **CONSUMER_LANDED** — the consumer (CI check, `.dag` TestClaim runner executing through Evaluator, manual demonstration receipt) executes the Pass condition + can fail-closed if violated.
- **PASSING** — consumer landed AND Pass condition currently true.

**Status at HEAD (PR #1808 sha at this commit)**: ~all 75 gates are **DECLARED**; very few have CONSUMER_LANDED status (existing `pb_self_compile_fixed_point` + `tier3_*_mirror_dissolved` + similar pre-R3-plan gates have consumers; the 16 NEW gates added 2026-05-06 in this PR are DECLARED-only).

**R3 close criteria implies CONSUMER_LANDED for all 75 gates**: declarations alone don't satisfy `r3_debt_paydown_zero_remaining` or the substrate-gap-class closures or the demonstration principle. Per Brian directive `feedback_no_textual_enforcement_bridges` + Director poke-hole 2026-05-06 finding 1.1 (demo-gate minimum bar) — closure requires runtime-executable verification, not document-level claims.

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
| T-LensProducer-Retirement | retirement gates (state-check) | + `lens_producer_retirement_executable_witness` (per PB Mgr poke-hole 2026-05-06 F3 — reframed from `_demonstration`) — execution-ready witness DEFERRED to Row-4 equivalence receipt landing per `docs/design-pb-runtime-interpreter.md` §5.1 + convergence matrix; near-term demo = retirement state-check + doc receipts; "demonstration" status re-promotes when Row-4 + Item 4 receipts exist |
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

**Sequence** (from `r3-structure.md` §"Lane structure" → T-V2-Retirement row + T-FixedPoint row; PB Mgr poke-hole 2026-05-06 F1 corrected sequencing — T-FixedPoint depends on "SG-0 zero from T-LensProducer-Retirement" per r3-structure.md §"Lane structure" → T-FixedPoint row, so T-LensProducer-Retirement comes BEFORE T-FixedPoint, not after):
1. **T-LensProducer-Retirement completes** (PB Mgr) — 3 hand-Rust files retired → SG-0 zero
2. **T-FixedPoint completes** (PB Mgr) — `compiler.dag` self-compile fixed-point (gated on SG-0 zero from step 1)
3. T-Numeric-Construction `Int<N>` refinement landing (path-(a) coordination — v2 refinement-syntax blocker per `r3-structure.md` §"Lane structure" → T-Numeric-Construction row; can parallel-dispatch with steps 1+2 per non-Evaluator-gated lane status)
4. v2 directory deletion + test-consumer audit (T-V2-Retirement; gated on T-FixedPoint + T-LensProducer-Retirement per r3-structure.md §"Lane structure" → T-V2-Retirement row)

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
- **Substrate-owned (2)**: `SourceSpan.file` participation checks (hand-Rust audit sites only — `bootstrap.rs:519` doc-comment + `:137` / `:287` / `:309` hardcoded path strings; per Substrate Mgr poke-hole 2026-05-06 Y4 scope-clarification: codegen-emitted `SourceSpan::new(...)` offsets in `bootstrap_generated.rs` are a separate generated-file bridge shape, NOT counted as Substrate-owned manual hand-Rust); `mark_bootstrap_secret_nominal_opacity()`
- **PB-owned (3)**: canonical lens-name dispatch; `include_str!` side channels (e.g., `pipeline_authority.rs`); `patch_lower_helpers_*` residual

Each retires per its natural-owner program prerequisites. Verification Mgr's `bridge_retirement_ledger_zero` gate audits cross-program reporting cadence.

**Sequence**:
1. Substrate: typed-identity-surface for SourceSpan participation; Secret PR A continuation.
2. PB: canonical lens-name dispatch retires alongside T-LensProducer-Retirement; `include_str!` retires post-T-FixedPoint; `patch_lower_helpers_*` post-Tier-2.
3. Verification audits cumulative ledger; closes when count = 0.

**ETA**: 6-10 weeks (gated on T-FixedPoint + T-LensProducer).

### §2.4 5 substrate-gap classes (5 predicates)

**Class 1 (parser/grammar)**: closes with T-V2-Retirement parser path (post-T-FixedPoint + post-T-LensProducer-Retirement + post-T-Numeric-Construction `Int<N>`).
**Class 2 (function-valued data)**: closes with T-Lens-Application-Surface (post-T-Lens-Behavioral-Parity). **CHAIN-BREAK per Refinement 3 YELLOW chain rule** (per Substrate Mgr 2026-05-06 ack at gunbc#846 #issuecomment-4384370416): Class 2's prerequisite chain (T-Lens-Application-Surface ← T-Lens-Behavioral-Parity ← E3 RED with no executable close path) does NOT terminate at GREEN within finite chain. Per `docs/audit/r3-debt-sweep-2026-05-06.md` §3 YELLOW criteria: invalid YELLOW status; must be RED until prerequisite chain resolves. Class 2 inherits RED from Q-Lens-Behavioral-Parity-R3-Closeability resolution. **Two resolution paths**:
- **(a) Re-pick Class 2 gap-test** to one that traces to GREEN (e.g., function-valued data gap-test that doesn't require LBP COMPLETE — possibly a representative `Lens<C>` consuming substrate facts that already exist post-T-E-P-Producer-Broadening but pre-T-Lens-Behavioral-Parity).
- **(b) Escalate Q-Lens-Behavioral-Parity-R3-Closeability** scope-calibration decision; if LBP option (b) reframes to 1-2 lenses for R3 with rest carved to R4, Class 2 gap-test must be expressible through the in-R3-scope subset.

→ **Q-Class-2-Chain-Break** in §10.3.
**Class 3 (file-ingestion)**: closes with T-Workflow-As-Data file-ingestion grammar (post-T-Lens-Behavioral-Parity).
**Class 4 (workflow/scheduling)**: closes with T-Workflow-As-Data CI-workflow-as-dag (post-T-Lens-Behavioral-Parity).
**Class 5 (reflection-closure)**: closes with T-LensProducer-Retirement (gated on T-FixedPoint via PB cascade chain — NOT parallel-to-LBP per Director poke-hole 2026-05-06 finding 2.4 correction).

**ETA**: Classes 2/3/4 parallel-eligible post-T-Lens-Behavioral-Parity COMPLETE. Class 1 + Class 5 cascade-gated (Class 1 on T-V2-Retirement chain; Class 5 on T-LensProducer-Retirement which depends on T-FixedPoint). 4-8 weeks total for parallel-eligible classes; cascade-gated classes track longer.

---

## §3. Lane status snapshot — delta-only over `r3-structure.md` §"Lane structure"

**Authority**: lane name / Mgr / scope / R2-close-dependency live in [`docs/r3-structure.md`](r3-structure.md) §"Lane structure" canonical table (per codex BLOCKING 2026-05-06 finding 1 — single-authority discipline; plan does NOT replicate lane definitions).

**Plan authoritative columns** (delta-only — current dispatch / blocker / ETA-to-close): updated weekly by lane-owning Mgr; PM compiles for Director cadence. Lane name + Mgr columns retained as join-key only (not as second authority).

| Lane (join-key only; defs in `r3-structure.md`) | Status | Current dispatch | Blocker | ETA-to-close |
|---|---|---|---|---|
| T-Tier3-Dissolution | YELLOW | (TBD from PB canvas) | (TBD) | (TBD) |
| T-LensProducer-Retirement | YELLOW | (TBD from PB canvas) | T-FixedPoint + R2-Evaluator | (TBD) |
| T-V-L4 (emit/eval match) | YELLOW | `r3-v-l4-l7-direct-*` briefs | Pattern A first slice (Q-PAFS) + corpus build-out | (TBD) |
| T-V-L7 (algebraic-law witness coverage) | YELLOW | `r3-v-l7-algebra-coverage-matrix` briefs | per-(algebra, inhabitant, law) exhaustive coverage gap | (TBD) |
| T-V-L5-Corpus | RED | (waits on L4 corpus + Shape A grounding) | T-V-L4 corpus existing first | post-L4 |
| T-FixedPoint | YELLOW | (TBD from PB canvas) | SG-0 zero from T-LensProducer | (TBD) |
| T-Numeric-Construction | YELLOW | T-NumericConstruction-ApproximateField; #1523 landed | Float migration + Real/base-carrier convention (Grounding canvas item 2) | (TBD) |
| T-Omni-Shape-B | RED | (post-R2-Evaluator + Shape A targets) | dependencies | (TBD) |
| T-Anthropic-Wire | YELLOW | 3 coproduct worker briefs dispatched on PR #1782 wait-window + 2 OPEN closure tags at HEAD (`dsl/extdeps/llm/anthropic.dag` `:189` + `:68`) | three coproduct slices land + variant-aware projection carrier authored | post-#1782 merge + 3 follow-up paydown PRs |
| T-Bridge-Retirement | YELLOW | per-bridge dispatches | 5 sub-bridges retire | post-T-FixedPoint |
| T-CostLens-Composition | YELLOW | (TBD from Substrate canvas) | R2-Evaluator + R2-T-Substrate-Lens-Primitive | (TBD) |
| T-V2-Retirement | RED | `r3-tv2-*` briefs | T-FixedPoint + T-LensProducer | longest-path |
| T-Free-Consequences-Demonstration | YELLOW | `r3-v-free-consequences-worker` | R2-Evaluator + T-CostLens-Composition | post-deps |
| T-E-P-Producer-Broadening | YELLOW | (TBD from Substrate canvas) | (foundational; in flight) | weeks |
| T-Lens-Behavioral-Parity | RED | post-T-E-P-Producer-Broadening | T-E-P-Producer-Broadening | post-T-E-P |
| T-Tests-As-Data-Completeness | YELLOW | `r3-v-tests-as-data-*` briefs | TestClaim infra | (TBD) |
| T-Lens-Application-Surface | YELLOW | `r3-substrate-tests-as-data-carrier-slice-1-stop-ping` | T-Lens-Behavioral-Parity COMPLETE | post-LBP |
| T-Workflow-As-Data | YELLOW | `r3-v-bridge-row-*` + `design-timing-lens` | T-Lens-Behavioral-Parity COMPLETE | post-LBP |
| T-Lens-Self-Application | RED | (waits on Workflow-As-Data + LAS) | T-Workflow-As-Data + T-Lens-Application-Surface | post-WAD+LAS |
| T-Debt-Paydown (standing) | YELLOW | PR #1807 (SG-0 PR-body / diff-reconcile gate; OPEN); PR #1566 (rollup hygiene; OPEN/DRAFT per `docs/debt/r3-debt-paydown-ledger-2026-05-02.md`); Tier-1 dispatch-brief drift sweep (`docs/audit/r3-dispatch-brief-drift-sweep-2026-05-05.md`) verified | per-PR rule + drift-item reconciliation (Q7) + velocity-tripwire reporting cadence + closure-receipt cadence + SG-0 PR-window net-shrink discipline | continuous |

**Status legend**: GREEN = no open work, awaiting close audit. YELLOW = work in flight. RED = blocked on upstream.

(Mgr canvases populating; full table compiles within ~24h per canvas T+24h soft deadline. Per codex 2026-05-06 BLOCKING finding 1: lane Mgr / scope / R2-close-dependency NOT replicated here — see `r3-structure.md` §"Lane structure" for authoritative metadata. This delta-only refactor closes Q-Plan-vs-Structure-Drift-Discipline option (a) per Research PM R-3.)

---

## §4. 5 substrate-gap classes — detail

**Framing (Q2 RATIFIED 2026-05-06)**: closure criteria over existing lane work, not new separate lanes. Each class is a TestClaim that fires when **both** (per §1.4 conjunctive predicate; Director poke-hole 2026-05-06 finding 1.3 + openai-pro 2026-05-06 finding 2.2): **(a)** representative gap-test executes through v3 cleanly per Pass condition AND **(b)** systematic enumeration of class-bridges shows count = 0 (or any survivors carry explicit Director allocation per §7.2 / `docs/audit/r3-debt-sweep-2026-05-06.md` §3.A STRUCTURAL exception). Single-test-pass alone is sample-of-class, NOT closure-of-class.

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

**Substrate-prerequisite enumeration** (per Substrate Mgr poke-hole 2026-05-06 R3 — closure path requires named carriers, not "we'll figure it out later"):

CI-workflow-as-`.dag`-data requires substrate carriers that **do NOT exist in `dsl/std/` today**:
- **Trigger-event sum**: `WorkflowTrigger = Push | PullRequest | Cron<Schedule> | Manual<Inputs>`
- **Step-graph substrate**: `WorkflowStep` (run command + dependencies + outputs) + `WorkflowMatrix<Axes>` (parameter expansion)
- **Secret-binding semantics**: `WorkflowSecret<Name>` (provider-typed, opaque-at-rest, scoped-by-step)
- **Runner-resource refs**: `RunnerResource<C>` (compute class, OS, hardware)
- **Workflow-as-Dag substrate**: `Workflow<Trigger, Steps, Resources>` carrier composing the above

These carriers route to **Q-Workflow-As-Data-Carriers** (NEW 2026-05-06 from Substrate Mgr poke-hole R3) in §10.3 — Substrate Mgr authoring scope; required substrate prerequisites for Class 4 closure path.



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

R3 close = all 75 gates GREEN + r3_debt_paydown_zero_remaining + comprehensive sweep zero-debt-rows-remaining (per §1 two-Pass-surfaces clarification; both must hold)
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

Per R3 Debt-Paydown standing program (`r3-structure.md` §"Standing program — R3 Debt-Paydown" + INVARIANTS §P5). **Four owned mechanisms** mirrored from canonical authority (per Debt-Paydown Mgr poke-hole 2026-05-06):
- **(1) Per-PR discipline rule**: §7.1 (debt-receipt)
- **(2) Velocity tripwire**: §7.6 (introduction:dissolution ratio enforcement)
- **(3) Closure-receipt cadence**: §7.7 (per-tracked-debt-row receipt cadence)
- **(4) SG-0 PR-window net-shrink discipline**: §7.8 (Director-ratified merge gate)

Plus discipline-application:
- §7.2 Ratchet-only-down (SG-0 zero-floor)
- §7.3 Anticipation discipline
- §7.4 Section/symbol anchors over line numbers
- §7.5 Grep-verify against source authority

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

### §7.6 Velocity tripwire (Debt-Paydown owned mechanism #2)

Per `INVARIANTS.md §P5` Dispatch-Discipline Mechanisms (c): **introduction:dissolution PR ratio ≥3:1 in any 7-day window** triggers Director review of ad-hoc lane dispatch. R3 Debt-Paydown Mgr surfaces tripwire readings to Director on cadence (per `r3-structure.md` §"Standing program — R3 Debt-Paydown"). Tripwire fire = signal that introduction is outpacing dissolution; not an automatic block, but a Director-review-required event.

### §7.7 Closure-receipt cadence (Debt-Paydown owned mechanism #3)

Per `r3-structure.md` §"Standing program — R3 Debt-Paydown": every tracked-debt row retires with a PR receipt before R3 close. The receipt names: (a) the row being retired, (b) the structural event that retires it, (c) the PR landing the retirement. Vague deferrals rejected (per INVARIANTS §P5 Dispatch-Discipline Mechanisms (b)). Closure-ledger entry per receipt; aggregation feeds `r3_debt_paydown_zero_remaining` predicate.

### §7.8 SG-0 PR-window net-shrink discipline (Debt-Paydown owned mechanism #4)

Per Director ratification + ROADMAP authority + `.github/PULL_REQUEST_TEMPLATE.md`: every PR touching SG-0 census surfaces (`src/v3/compiler/tests/integration/sg0_census_test.rs` `EXPECTED_HAND_AUTHORED_*` arrays) MUST carry net-shrink discipline — additions paired with removals OR explicit Director allocation citing the program shape. CI enforcement via `scripts/check-pr-sg0-net-shrink-discipline.sh` (gate `pr_anticipation_discipline_ci_active` per §1 closure-gate set). This is a **merge-discipline gate**, NOT a `.dag` TestClaim — separate Pass surface from lane gates (per §1 two-Pass-surfaces clarification). Routed to R3 Debt-Paydown Mgr (#1744 #issuecomment-4383628247).

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

**Status**: live. PM compiles per-Mgr canvas escalations as they arrive; surfaces consolidated list to Brian + Director for resolution. **Plan PR (#1808) is OPEN; merge-eligibility distinct from R3-close** — PR merges with §10.3 RED items in flight provided they're explicitly tracked + assigned routing; R3 close requires all RED items resolved per their owners (per openai-pro 2026-05-06 finding — distinguish PR-open vs merge-eligible vs R3-closure-eligible).

### §10.1 PM-surfaced ratifications (Q1–Q7 from gunbc#846 reply 2026-05-06)

**Status (2026-05-06): all RATIFIED per Brian directive — *"I'll take your recommendations unless controversial on all Q1-Q7."*** Plus **demonstration principle** (every feature comes with a demonstration) ratified — see §1.6.

| # | Question | PM recommendation | Status |
|---|---|---|---|
| Q1 | Closure-criteria predicate set (§1.1–§1.4) | Predicates per §1 tables; **75 closure gates total** (59 R3 lane gates including 5 NEW Pattern-A executable gates added 2026-05-06 to canonical authority + 5 substrate-gap-class + 10 demonstration + 1 PR-anticipation-discipline; precise enumeration per §1.5) | RATIFIED |
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

#### §10.2.4 R3 Evaluator Mgr (merry-gull-128, #1743) — full canvas absorbed 2026-05-06T01:52Z

Substantive review surfaces R3 Evaluator continuation work that plan was treating as already-landed under "R2-Evaluator capacity." 6 plan findings + 3 EVAL escalations.

**Plan findings absorbed**:

- **F1 (§2.1 + §10.3 TC1-first under-scoped)**: TC1 requires lens producer/fold path + generic `fold_lens<C> -> DimensionReport<C>` + lens/value authoring + report lifting + eta relation + universal-vs-representative coverage decision per `docs/briefs/r3-v-pattern-a-coverage-rollup.md`. Q-PAFS naive "TC1 quick first slice" reading is wrong. Either split Q-PAFS into TC1-static-representative (E6-G1.a) vs TC1-generic (E6-G1.b/X1.b), OR pick RustDagIsomorphism if it has fewer evaluator deps. → **Surfaces as Q-Pattern-A-First-Slice-Subscope** in §10.3.
- **F4 (§1.1 per-predicate prereqs not specific enough)**: each Pattern-A predicate has DIFFERENT runtime prereqs. Plan §1.1 implies shared `BinaryDimensionReportEquals` envelope; reality requires per-predicate decomposition (see EVAL canvas + §10.3 below).
- **F2/F3 deferred to canonical ledger** (per openai-pro meta-review structural-consolidation): §4.2 Class 2 evaluator deps + §6 R3-Evaluator-continuation-chain fold into the §1.8 canonical closure-authority ledger when authored. Currently the per-predicate prereq table below captures the load-bearing constraints.
- **F5 (§3 "R2-Evaluator" umbrella too coarse)**: deferred to canonical ledger; row-blocker columns will name specific evaluator surfaces (constructor runtime / lens fold / Descent proof / generic dispatch).

**Per-predicate runtime prereqs** (per Evaluator Mgr F4 — load-bearing for any Pattern-A consumer infrastructure):

| Predicate | Runtime prereq |
|---|---|
| TC1 | E6-G1.a static representative OR E6-G1.b generic fold_lens<C> + eta relation |
| TC2 | second executable strategy/input order + strategy-keyed report production |
| TC3 | Descent execution proof / termination contract (E5) + evaluation-step producer |
| RustDagIsomorphism | shape-report producers |
| SymbolicCostExprEquals | cost lens / report producer (T-CostLens-Composition runtime) |
| BridgeLedgerZero | ledger retirement work (NOT report comparison; Pattern E ratchet) |
| Free-Consequences | split `BinaryDimensionReportEquals` subset from `LensOutputEquals` subset |

**EVAL escalation canvas**:

- **EVAL-1. E6-G0d constructor runtime execution open** — `eval_transform_node` rejects non-Arrow constructor `Callable` ("Callable target declaration is not an Arrow type"); G0d brief at `docs/briefs/r3-pr-e6-g0d-constructor-runtime-execution-worker.md` ready (#1784) but not landed. Authorize evaluator implementation slice using `Value::RecordValue` / `Value::VariantValue` + shared substrate declaration-walk. Required before Pop A constructor behavior + honest `Witness`/`DimensionReport` construction + static lens fold can close. → **Q-EVAL-G0d-Dispatch** in §10.3.
- **EVAL-2. E5 `LoopBound::Descent` blocked on substrate termination contract** — `eval_loop` fails closed with `LoopBoundDescentResidual`; sharp-ibex #1799 STOP/audit; quick-koi substrate impl authorized. Substrate must land `descent_execution_proof(&Dag, ClusterId, PortId) -> Result<DescentExecutionProof, DescentResidual>` with `Missing | Unknown | Incomplete | NonStrict` fail-closed residuals; evaluator consumes proof token only. Required for TC3 + any lens fold traversing Descent loops. → **Q-EVAL-Descent-Termination-Contract** in §10.3.
- **EVAL-3. E6-G1.a static lens fold is next executable lens/report slice (NOT generic fold_lens<C>)** — Decision needed: Pattern A first executable uses single static top-level `data ... : Lens<C>` representative (G1.a) OR closure requires generic dispatch (G1.b). The TC1 cascading requirement chains here — F1 surfaces this as scope-question. → **Q-EVAL-Lens-Fold-First-Slice** in §10.3 (overlaps with Q-PAFS).

#### §10.2.5 R3 PB Mgr — pending canvas response (as-of 2026-05-06T00:30Z)

#### §10.2.7 Research PM (loyal-swift-270, gunb-ai/ctrl#339) — full canvas absorbed 2026-05-06T02:00Z

Substantive cross-program review surfacing ctrl-side coupling concerns + structural taxonomy + timeline risk. 3 RED + 4 YELLOW + 4 GREEN.

**RED findings**:
- **R-1 (Class 6 missing — substrate-extension lens slice)**: 3 ctrl-side substrate proposals don't fit Classes 1-5: `SchemaPreservation<TableId>` (ctrl#370/#407 LLVM TBAA + GitHub-ORM-vs-schema), `ResourceCap<T>` (ctrl#404/#405 Cloudflare), `TotalResult<E>` (ctrl#404/#405). PR ctrl#408 §3 committed PM-side to routing to Substrate Mgr (#1130). → **Q-Class-6-Substrate-Extension-Lens** in §10.3.
- **R-2 (§6 timeline is commitment not derivation)**: 8-12 weeks doesn't account for C1-class candidate-gap budget (2-6 weeks per gap) + WEDGE-CORE-CLAIM Part A thesis-edit scope (ctrl#444) + demo-authoring buffer (~9 new gates × 1-3 weeks each). → **Q-Timeline-Risk-Alternates** in §10.3.
- **R-3 (§3 vs r3-structure.md drift risk)**: Same lane data in two authoritative docs without automated consistency. Recommend §3 → pointer-with-delta to r3-structure.md §"Lane structure" + delta-only annotations. → **Q-Plan-vs-Structure-Drift-Discipline** in §10.3.

**YELLOW findings**:
- **Y-1 (ctrl coupling absent)**: WEDGE-CORE-CLAIM Part A (ctrl#444 merged 2026-05-05) + Tier 4 (ctrl#1608 §6d) + ctrl#369 substrate proposals not surfaced in plan §10. → **Q-WEDGE-A** + **Q-Tier4-Inclusion** in §10.3.
- **Y-2 (T-Lens-Self-Application stronger demo)**: Current `lens_self_application_demonstrated` cashes "lens applies to CI workflow" but doesn't tie to emission gate. Stronger shape: CI workflow includes TestClaim → fails → gunbc emission refuses. Demonstrates 4 gates simultaneously (Self-App + Workflow-As-Data + Tests-As-Data + 5th gate `integration_testgen_demonstrated`) + WEDGE-CORE-CLAIM Part B opportunistically. → **Q-Lens-Self-Application-Stronger-Demo** in §10.3.
- **Y-3 (Class boundary overlap)**: Class 3 (file-ingestion) ∩ Class 4 (workflow/scheduling) — CI workflow gap-test likely contains file-ingestion. Class 5 ∩ T-FixedPoint distinction unclear. → defer (minor scope-clarification; absorb at canonical ledger landing).
- **Y-4 (Director/PM no redundancy)**: 30-min Director loop + weekly PM compile have no fallback authority for unavailability stretches. → **Q-Director-PM-Redundancy** in §10.3.

**GREEN affirmations**: §1.6 demonstration principle + §6 dependency DAG shape + §10 RATIFIED-by-default discipline + §7 PR-authoring contract — Research-PM-validated as load-bearing.

**Plan integration**: 6 new escalations added to §10.3 (Q-Class-6 / Q-Timeline-Risk-Alternates / Q-Plan-vs-Structure-Drift / Q-WEDGE-A / Q-Tier4-Inclusion / Q-Lens-Self-App-Stronger-Demo / Q-Director-PM-Redundancy). Y-3 deferred to canonical ledger absorption.

#### §10.2.5 R3 PB Mgr (neat-bear-351, #1742) — pending canvas response (as-of 2026-05-06T01:52Z)

#### §10.2.6 R3 Debt-Paydown Mgr (quiet-otter-416) — full canvas + plan-poke-hole absorbed at sha `e41e4fbe8` (2026-05-06)

Substrate canvas + plan-poke-hole both folded in. Key absorptions:
- **§1 Two-Pass-surfaces clarification**: lane TestClaim gates (75 total) AND `r3_debt_paydown_zero_remaining` are distinct Pass surfaces; both must hold for R3 close
- **§7 four-mechanism structure** (per `r3-structure.md` §"Standing program — R3 Debt-Paydown" canonical authority): added §7.6 Velocity tripwire + §7.7 Closure-receipt cadence + §7.8 SG-0 PR-window net-shrink discipline
- **§3 T-Debt-Paydown row** updated from "(TBD from canvas)" → YELLOW with PR #1807 + PR #1566 + Tier-1 dispatch-brief-drift-sweep anchors

(Per Director poke-hole 2026-05-06 finding 6.1: timestamps clarify point-in-time placeholders for the 2 still-pending Mgrs above. Per openai-pro 2026-05-06: PR #1808 is OPEN; merge-eligibility distinct from R3-close-eligibility per the new §10 status header above.)

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
| Q-PR-F | PR-F priority + sequencing (BoundDeclaration consumer + Rust ReferenceModel<T>) | **Bandwidth-aware dispatch** (per Substrate Mgr poke-hole 2026-05-06 Y3 — Substrate's queued workers are saturated): PR-F is 2-axis substantial work. Per Substrate Mgr canvas D: only loyal-wolf + valiant-ant fully idle; proud-lynx/smart-ram/valiant-ibex holding on #1794 cascade. Dispatch worker = loyal-wolf-828 OR valiant-ant-72 (Substrate Mgr selects); brief shape = T-E-P-Producer-Broadening adjacent (per `docs/briefs/r3-t-e-p-producer-broadening-worker.md` precedent); critical-path slot = post-#1782 wait-window close + parallel to T-E-P dispatch (B3). Substrate Mgr authors brief + dispatches. | RATIFIED-by-default-with-bandwidth-routing |
| Q-Workflow-As-Data-Carriers | Substrate carriers for CI-workflow-as-`.dag`-data Class 4 closure path: `WorkflowTrigger` sum + `WorkflowStep`/`WorkflowMatrix` step-graph + `WorkflowSecret<Name>` binding + `RunnerResource<C>` + `Workflow<Trigger, Steps, Resources>` composing carrier | Substrate Mgr authors carriers as part of T-Workflow-As-Data lane; sequenced post-T-Lens-Behavioral-Parity COMPLETE (per `r3-structure.md` §"Dependency on R2"); folds into existing T-Workflow-As-Data scope without new lane spawn. Required for Class 4 substrate-gap-class closure per §1.4. | OPEN — Substrate Mgr scoping needed (NEW 2026-05-06 from Substrate Mgr poke-hole R3) |
| Q-Pattern-A-First-Slice-Subscope | Q-PAFS sub-question: TC1 first means **static representative** (E6-G1.a) or **generic** (E6-G1.b/X1.b)? Plan defaulted to TC1 first without specifying which sub-shape. Per Evaluator Mgr F1 + EVAL-3: substantial substrate/evaluator scope difference. | Per `docs/briefs/r3-v-pattern-a-coverage-rollup.md` + Evaluator Mgr canvas: TC1-static-representative via E6-G1.a is achievable with current evaluator + lens-fold infrastructure; TC1-generic requires X1.b S1/S3 + G1.b. PM-recommended: **TC1-static-representative first** (smallest viable slice); generic deferred to subsequent ratchet. | OPEN — Director / Brian ratification needed (NEW 2026-05-06 from Evaluator Mgr F1 + EVAL-3; sub-question of Q-PAFS) |
| Q-EVAL-G0d-Dispatch | E6-G0d constructor runtime execution: authorize evaluator slice for non-Arrow constructor `Callable` using `Value::RecordValue` / `Value::VariantValue` + shared substrate declaration-walk authority | Evaluator Mgr (merry-gull-128) dispatches; Director needed only if shared helper extraction crosses ownership. Brief at `docs/briefs/r3-pr-e6-g0d-constructor-runtime-execution-worker.md` ready. | RATIFIED-by-default — Evaluator Mgr executes (NEW 2026-05-06 from Evaluator Mgr EVAL-1) |
| Q-EVAL-Descent-Termination-Contract | E5 `LoopBound::Descent` execution: substrate carrier `descent_execution_proof(&Dag, ClusterId, PortId) -> Result<DescentExecutionProof, DescentResidual>` with `Missing \| Unknown \| Incomplete \| NonStrict` fail-closed residuals | Substrate Mgr authors carrier (quick-koi-190 implementation already authorized through quick-crab); Evaluator consume-side ack; sharp-ibex consumer wiring dispatch. | RATIFIED-by-default — Substrate-side already authorized; Evaluator consume-side dependency (NEW 2026-05-06 from Evaluator Mgr EVAL-2) |
| Q-EVAL-Lens-Fold-First-Slice | Overlaps with Q-Pattern-A-First-Slice-Subscope: E6-G1.a static representative vs G1.b generic for first executable lens/report slice | Same recommendation as Q-Pattern-A-First-Slice-Subscope: **G1.a static-representative first**; G1.b generic deferred. | OPEN — same scope-question as Q-Pattern-A-First-Slice-Subscope (NEW 2026-05-06 from Evaluator Mgr EVAL-3) |
| Q-Class-6-Substrate-Extension-Lens | 3 ctrl-side substrate proposals (`SchemaPreservation<TableId>` per ctrl#370/#407 LLVM TBAA; `ResourceCap<T>` per ctrl#404/#405 Cloudflare; `TotalResult<E>` per ctrl#404/#405) don't fit Classes 1-5; PM-side routing to Substrate Mgr (#1130) committed per ctrl#408 §3 | PER Brian no-post-R3-deferral discipline: include as Class 6 substrate-gap-class with closure-criterion. Add §4.6 "Class 6 — typed-metadata-preservation / extension-lens slice" with representative gap-test per substrate. Alternative: Director ratification of explicit post-R3-ecosystem-by-design scope-cession. | OPEN — Brian / Director scope-calibration (NEW 2026-05-06 from Research PM R-1; substantial scope-determination question — sixth substrate-gap class adds ~1 lane equivalent or scope-cession) |
| Q-Timeline-Risk-Alternates | §6 8-12 week estimate is commitment, not bottoms-up derivation. Missing risk-weighted alternates for: C1-class candidate-gap budget (2-6 weeks/gap) + WEDGE-CORE-CLAIM Part A thesis-edit scope + ~9 demo-gate authoring (1-3 weeks/lane) | Per `feedback_holistic_over_patches`: §9.2 add explicit "+X weeks if {C1-gap surfaces / Tier-4 thesis-edit lands in scope / demo authoring exceeds budget}" branches. R3 close horizon: 2026-08-01 ± 2 weeks PROVISIONAL → range with explicit risk-weighted alternates. | RATIFIED-by-default — PM updates §9.2 with risk-weighted branches in subsequent commit (NEW 2026-05-06 from Research PM R-2) |
| Q-Plan-vs-Structure-Drift-Discipline | §3 lane status table holds 19 rows replicating r3-structure.md §"Lane structure" data; drift risk if PM updates one but not the other | Per Research PM R-3 recommendation (a): §3 lane status becomes pointer-with-delta to r3-structure.md §"Lane structure" + delta-only annotations (current dispatch + blocker + ETA). Lane name/Mgr/scope columns NOT restated; r3-structure.md retains architectural-decision authority. | RATIFIED-by-default — PM refactors §3 in subsequent commit OR canonical-ledger-landing absorbs (path-(a) decision pending; NEW 2026-05-06 from Research PM R-3) |
| Q-WEDGE-A | WEDGE-CORE-CLAIM Part A (ctrl#444 merged 2026-05-05): build-orchestration as free consequence — should it be in §1 closure criteria as thesis-edit gate? PM-disentangled framing routes Part A to Director for thesis-clarification edit | Per Brian no-post-R3-deferral + R3-as-consequence-layer-of-thesis: **include as thesis-edit gate** in R3 closure criteria. Scope: Director ratifies thesis-clarification edit; gate fires when r3-structure.md §"R3 closure criteria" reflects Part A formalization. | OPEN — Director ratification of thesis-edit framing (NEW 2026-05-06 from Research PM Y-1) |
| Q-Tier4-Inclusion | Tier 4 ("out of scope, declared" per ctrl#1608 §6d): convergent ctrl-side recommendation across LLVM/K8s/Discord/PyTorch viability research portfolio. PM-side own. Not currently in R3 closure criteria. | PM-recommended: include Tier 4 as **named explicit-out-of-scope declaration** in §1 closure criteria — distinct from Class 6 (which is in-scope substrate extension). Tier 4 = explicit scope-cession; Class 6 = in-scope substrate addition. Director ratification of either inclusion or out-of-scope-by-design needed. | OPEN — Director ratification (NEW 2026-05-06 from Research PM Y-1) |
| Q-Lens-Self-Application-Stronger-Demo | Current `lens_self_application_demonstrated` cashes "lens applies to CI workflow" only. Stronger: CI workflow includes TestClaim → fails → gunbc emission refuses. Demonstrates 4 gates simultaneously (Self-App + Workflow-As-Data + Tests-As-Data + integration_testgen) + WEDGE-CORE-CLAIM Part B opportunistically | Per Brian recursive-flex framing ("the compiler that compiles gunbc programs validates the workflow that produces gunbc itself"): current demo cashes first half; stronger shape cashes the full sentence by tying lens output to emission gate. Verification Mgr's audit packet (per gunbc#1276) targets four-fold demo overlap. | RATIFIED-by-default — Verification Mgr authors stronger demo shape; updates §1.6 audit row + canonical ledger when authored (NEW 2026-05-06 from Research PM Y-2) |
| Q-Director-PM-Redundancy | 30-min Director loop + weekly PM compile have no fallback authority for unavailability stretches (>2 days). Specific failure modes: Director unavailable → 30-min loop drops; PM unavailable → §3 drifts; both unavailable → §8.2 escalation paths have no fallback | PM-recommended: §8.3 add Director-deputy mechanism (Substrate Mgr + Verification Mgr cross-Mgr coverage if Director unavailable >2 days; explicit slip-acceptance otherwise); §9.1 add PM-deputy mechanism (R3 Debt-Paydown Mgr covers §3 lane-status compilation if PM unavailable). | OPEN — Director / Brian ratification of fallback authority + slip-acceptance discipline (NEW 2026-05-06 from Research PM Y-4) |
| Q-Class-2-Chain-Break | Class 2 (function-valued data) prereq chain (T-Lens-Application-Surface ← T-Lens-Behavioral-Parity ← E3 RED) does NOT terminate at GREEN within finite chain. Per Refinement 3 YELLOW chain rule, Class 2's YELLOW status is **invalid** — must be RED until prereq chain resolves. Either re-pick gap-test OR escalate LBP scope-calibration. | Two resolution paths: **(a)** re-pick Class 2 gap-test to one traceable to GREEN (representative `Lens<C>` consuming post-T-E-P-Producer-Broadening substrate without requiring LBP COMPLETE); **(b)** escalate Q-Lens-Behavioral-Parity-R3-Closeability — if LBP option (b) reframes to 1-2 lenses for R3, Class 2 gap-test must be expressible through in-R3-scope subset. PM-recommended path: (a) — re-pick gap-test if Substrate Mgr can identify a function-valued substrate test that doesn't require LBP COMPLETE; otherwise (b). | OPEN — Substrate Mgr + Director scope-calibration (NEW 2026-05-06 from Substrate Mgr R2 + Refinement 3 YELLOW chain rule cycle-detection; structural chain-break per `docs/audit/r3-debt-sweep-2026-05-06.md` §3) |
| Q-V2-Retirement-Boundary-Matrix | Q3 "PB owns all v2 retirement" too coarse — Grounding §10.2.2 items (emit shim G6 / Coercion-Fold scratch G7 / LanguageSpec consumer surfaces) are v2-shaped but not obviously PB-only. Per PB Mgr poke-hole 2026-05-06 F4: explicit split matrix (PB vs Grounding vs Debt-Paydown) needed for v2-using emit surfaces so Q3 doesn't silently absorb non-PB work. | PM-recommended split: **PB owns** v2 directory deletion + v2 test-consumer retirement + bootstrap PB-Runtime trampoline; **Grounding owns** emit-shim retirement (consumes T-V2-Retirement milestones per G6) + Coercion-Fold scratch retirement (post-LanguageSpec projection per G7); **Debt-Paydown owns** v2-related drift items (post-retirement ledger reconciliation). Matrix authored by PM as plan §11 supplement OR Q3 row narrowed in §10.1 explicitly. | OPEN — Director ratification of split matrix (NEW 2026-05-06 from PB Mgr F4) |
| Q-Bridge-Retirement-Sequencing-Authority | §2.3 says "`include_str!` retires post-T-FixedPoint" — new ordering detail vs r3-structure.md §"Lane structure" → T-Bridge-Retirement row distribution map (lists 5 bridges, not total order). Per PB Mgr F5: looks like second authority for sequencing. | Move sequencing detail to r3-structure.md as symbol-anchor addition to §"Lane structure" → T-Bridge-Retirement row (single authority for both 5-bridge distribution + retirement sequencing); §2.3 cross-links instead. PM updates in subsequent commit per PB Mgr recommendation. | RATIFIED-by-default — PM moves sequencing to r3-structure.md authoritative row in subsequent commit (NEW 2026-05-06 from PB Mgr F5) |
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
