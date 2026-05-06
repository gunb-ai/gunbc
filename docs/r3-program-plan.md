# R3 Program Plan — 2026-05-06

**Status:** **DRAFT — under PM authorship + Mgr-canvas absorption.** Phase 1 (skeleton + closure criteria + lane status) authored; Phase 2 (per-Mgr escalation absorption) compiles canvas responses as they arrive on inboxes #1739–#1745. Phase 3 (Director ratification) closes Q1–Q7 in §10. Plan PR opens after §10 escalations clear.

**Purpose.** Forward-looking program plan for **R3 close with zero debt**. Per user directive 2026-05-05 (gunbc#846): *"clear dependency graph from here to R3 close"* + *"surface escalations now and solve them"*.

**Authority hierarchy.**
- [`docs/r3-structure.md`](r3-structure.md) — **architectural-decision archive**. Lane definitions, manager structure, design-challenges resolution, dependency-on-R2 spec. Authoritative for *what R3 is*.
- [`docs/audit/r3-debt-sweep-2026-05-06.md`](audit/r3-debt-sweep-2026-05-06.md) — **bridge inventory snapshot** (PR #1804 lane). Authoritative for *what bridges exist now*.
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
| 1 — parser/grammar surface | `substrate_gap_parser_grammar_closed` | v3 parser handles `Int<N>` refinement syntax (`dsl/std/types.dag` Slice 2 alignment per Substrate Mgr design stance) and lowers through stage0 emission without v2-fallback; gate Pass = `cargo test -p v2-compiler-tests v2_strict_compile_diagnostic_count -- --ignored` evaluates Int<N>-bearing program → 0 diagnostics + emitted artifact compiles | T-V2-Retirement (parser path) + T-Numeric-Construction (Int<N> refinement) + Substrate continuation |
| 2 — function-valued data | `substrate_gap_function_valued_data_closed` | `Lens<C>` instance with function-typed payload executes through evaluator | T-Lens-Application-Surface + T-E-P-Producer-Broadening |
| 3 — file-ingestion | `substrate_gap_file_ingestion_closed` | One `.dag` program ingests external file (e.g., timing observation set) without `include_str!` | T-Workflow-As-Data |
| 4 — workflow/scheduling | `substrate_gap_workflow_scheduling_closed` | CI workflow modeled as `.dag` data executes through evaluator | T-Workflow-As-Data + T-Lens-Self-Application |
| 5 — reflection-closure | `substrate_gap_reflection_closure_closed` | `lens_apply.rs` reflection work routes via PB-Runtime interpreter-as-data | T-LensProducer-Retirement |

**Open** (Q2, §10): all gap-test selections per-class ratified by default per Brian directive 2026-05-06 (*"take recommendations unless controversial"*).

### §1.5 Closure-ledger total

**Total R3 closure gates** (post-Q1 + Q2 ratification): existing 60+ gates from `r3-structure.md` §"Acceptance" + 5 new substrate-gap-class gates + per-lane demonstration gates (§1.6) + 1 PR-anticipation-discipline gate (§7) = **~71 gates** across 18 lanes + 1 standing program.

R3 closes when ALL gates pass + zero tracked-debt rows survive (`r3_debt_paydown_zero_remaining`).

**Tracked-debt inclusion list for `r3_debt_paydown_zero_remaining`** (per Director poke-hole 2026-05-06 finding 3.1; closes definition gap):

- **Counts toward zero**: ROADMAP open `- **` rows under "Post-merge debt" sections; sweep-doc §1 Class A/B/C/F/G entries (per §1.3 inclusion criteria); §10 RED-flagged escalation items.
- **Excluded by-construction**: PM-coordination items (Brian-Q queue, dispatch routing); scheduled-deletions-with-named-trigger (the trigger IS the resolution); retirement-receipts (post-resolution accounting); Class D bridges (bounded-scaffold freshness gates) and Class E bridges (v2/v3 transition — retires at v2 retirement which has its own gate).

Predicate cannot be satisfied by deletion-of-rows-without-actual-resolution; each retiring row requires named PR receipt OR closure-ledger entry citing the structural event that ends the row's existence.

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

**Net additions**: ~9 new demonstration gates across substrate-heavy / state-check-only lanes. Owner Mgrs author per their lane scope.

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

**Blockers** (per Grounding Mgr canvas, bold-ferret-748, item #2): PR-F (`BoundDeclaration` consumer + Rust `ReferenceModel<T>` axes) blocks T-Ground-Rust; T-NumericConstruction-ApproximateField blocks Rust float rows.

**Sequence** (from `r3-structure.md` §"Lane structure" → T-V2-Retirement row):
1. T-FixedPoint completes (PB Mgr) — `compiler.dag` self-compile fixed-point
2. T-LensProducer-Retirement completes (PB Mgr) — 3 hand-Rust files retired
3. PR-F lands (T-Ground-Rust unblocks)
4. Float carrier migration (T-NumericConstruction-ApproximateField)
5. v2 directory deletion + test-consumer audit (T-V2-Retirement)

**ETA**: 4-8 weeks; longest-path lane in R3.

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

**Class 1 (parser/grammar)**: closes with T-V2-Retirement parser path (post-T-FixedPoint).
**Class 2 (function-valued data)**: closes with T-Lens-Application-Surface (post-T-Lens-Behavioral-Parity).
**Class 3 (file-ingestion)**: closes with T-Workflow-As-Data file-ingestion grammar.
**Class 4 (workflow/scheduling)**: closes with T-Workflow-As-Data CI-workflow-as-dag.
**Class 5 (reflection-closure)**: closes with T-LensProducer-Retirement.

**ETA**: All 5 parallel-eligible post-T-Lens-Behavioral-Parity; 4-8 weeks total.

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
| T-Anthropic-Wire | Substrate | GREEN-pending | (audit) | None | near-term |
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

**What this class names**: bridges that exist because v3 parser cannot handle certain grammar surfaces (e.g., refinement syntax `Int<N>`, `where bits <= N` constraints, function-valued return types). v2 currently catches these; v3 cannot.

**Representative gap-test**: TBD via Substrate canvas. Candidate: `Int<N>` refinement parsing + lowering through stage0 emission.

**Closes via**: T-V2-Retirement parser path (post-T-FixedPoint) + Substrate parser continuation work (cited in `r3-structure.md` §"Lane structure" → T-Numeric-Construction row, T-V2-Retirement coordination dependency).

**Owner Mgr**: PB Mgr (T-V2-Retirement scope) coordinating with Substrate Mgr (parser substrate carriers).

**Open (§10)**: Substrate canvas response selects representative gap-test.

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

**Source-of-truth**: [`docs/audit/r3-debt-sweep-2026-05-06.md`](audit/r3-debt-sweep-2026-05-06.md) (PR #1804 lane). Phase 2 sweep deliverables landed §1.1–§2.2.

**Status (2026-05-06)**: per-class counts below are PRE-Phase-3-compile expected ranges from Director's bridge-class framework. Phase 2 sweep deliverables (PR #1804 §1.1–§2.2) have landed substantive partial data; **Phase 3 compile** populates verified-at-HEAD counts replacing these expected ranges before plan PR opens (per Director poke-hole 2026-05-06 finding 6.3).

**Apples-to-apples count discipline** (per `feedback_corrections_must_grep_verify_source` + INVARIANTS P1 single-authority):

- **66 `.rs` files** in `src/v3/compiler/src/` (excluding `emit/`) — file-level scope of quiet-boar-160's per-file audit work (Class A-G assignment per audited file). Filesystem count.
- **41 SG-0 NON_TEST entries** for the same scope (`src/v3/compiler/src/` excluding `emit/`) per [`src/v3/compiler/tests/integration/sg0_census_test.rs`](../src/v3/compiler/tests/integration/sg0_census_test.rs) `EXPECTED_HAND_AUTHORED_NON_TEST` filter. SG-0 ratchet authority for this scope.
- **66 - 41 = 25 file-system files NOT in SG-0 NON_TEST**: generated, build artifacts, or otherwise outside SG-0 hand-Rust ratchet scope.
- **SG-0 census authority full scope** (broader than this scope): `EXPECTED_HAND_AUTHORED_NON_TEST` = 46 paths total / `EXPECTED_HAND_AUTHORED_TEST` = 89 paths / `EXPECTED_HAND_AUTHORED_FRAGMENTS` = 1 path; **total = 136 paths**.

The 66-file audit is per-file classification work distinct from SG-0 census authority (path inclusion). Both counts are useful for different purposes: 66 = audit scope; 41 = SG-0 ratchet for that scope; 136 = SG-0 ratchet total.

Plus 35 v3 std mirror entries + 112 R2/R3-era PRs (covered by §1.3 + §2.1/§2.2 of PR #1804 sweep deliverables).

This plan section reports cumulative bridge counts; PR #1804 holds the per-row inventory.

**Phase 2 inventory shape** (counts populate post-canvas-completion; **expected ranges** below are pre-Phase-2-completion estimates from Director's bridge-class framework, NOT compiled facts — see PR #1804 for authoritative per-row inventory as canvases land):

- **Class A (substrate-gap-blocked)**: expected range ~15-20 bridges; awaiting Mgr canvas enumeration.
- **Class B (Pattern-A NYI predicates)**: count = 7 (this count IS authoritative — predicates pre-named in framework): TC1, TC2, TC3, free-consequences, RustDagIsomorphism, BridgeLedgerZero, SymbolicCostExprEquals.
- **Class C (Pattern-C typed-carrier + Rust-mirror)**: expected range ~6+ — EmissionDiagnostic, Value, EvalStrategy, BridgeLedgerRef, target-primitive routing pre-named; others awaiting canvas.
- **Class D (generated bridges with freshness gates)**: expected range ~3-5 — `render_repeat_string_bootstrap.dag`, `generated_method_template_projection.rs` pre-named; others awaiting canvas.
- **Class E (v2/v3 transition)**: expected range covered by T-V2-Retirement scope (not enumerated per-bridge here; v2 retirement closure subsumes).
- **Class F (operator-ontology-duplication)**: expected range — awaiting canvas.
- **Class G (local-small)**: expected range — distributed across lanes; awaiting canvas.

**Authoritative counts populate via PR #1804 §1.A–§1.G as Mgr canvases land**. Net-bridge count at R3 close is 0 (per `bridge_retirement_ledger_zero` + 5 substrate-gap-class closure gates). Each bridge has a named dissolution trigger; bridges without triggers escalate to substrate gap requiring named R3 lane (per [`docs/audit/r3-debt-sweep-2026-05-06.md`](audit/r3-debt-sweep-2026-05-06.md) §4 anticipation discipline).

**Drift items from Phase 2 sweep** (3 known; surfaced for §10 reconciliation):
- `#1638` ROADMAP retired but ledger Open
- `#1499` transitional fence disposition
- `declaration_by_name` retired/Open conflict

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

R3 close = all ~71 gates GREEN + r3_debt_paydown_zero_remaining + comprehensive sweep zero-debt-rows-remaining
```

**Edges added in §6 graph per Director poke-hole 2026-05-06**:
- T-Numeric-Construction → T-V2-Retirement (parser/grammar substrate path; finding 2.1)
- T-FixedPoint → Bridge-Retirement Ledger (PB-owned bridge retirement cascades; finding 2.2)
- T-Lens-Application-Surface and T-Workflow-As-Data parallel post-LBP COMPLETE (NOT sequential; finding 2.3 — both gated by T-Lens-Behavioral-Parity, neither depends on the other)

**Two longest-dependency paths** (per `r3-structure.md` §"Dependency DAG" — Verification Mgr poke-hole 2026-05-06 surfaced this dual structure was missing from earlier framing):

**Global critical-path chain** (substrate-completion + recursive-flex commit):

1. T-E-P-Producer-Broadening (Substrate, foundational)
2. → T-Lens-Behavioral-Parity (cross-program, 4 sub-slices)
3. → T-Lens-Application-Surface (cross-program)
4. → T-Workflow-As-Data (Substrate)
5. → T-Lens-Self-Application (Verification, demonstration)

**Verification-internal critical path** (longest verification path; parallel to global chain):

1. T-Verification-L4-L7-Direct (post-R2-Evaluator + R2-T-Substrate-Lens-Primitive + Shape A grounding for L4 corpus)
2. → T-Verification-L5-Corpus (consumes L4 corpus + R2-Grounding-Rust + R2-Grounding-Python)

Verification-internal path runs in parallel with global chain post-R2-Evaluator. Plus parallel longest single-lane path: **T-V2-Retirement** depends on T-FixedPoint + T-LensProducer-Retirement.

Plus the parallel longest-path: **T-V2-Retirement** depends on T-FixedPoint + T-LensProducer-Retirement; longest single-lane path.

**Estimated R3 close**: 8-12 weeks from 2026-05-06, gated on T-V2-Retirement + T-Lens-Self-Application both completing.

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
| Q1 | Closure-criteria predicate set (§1.1–§1.4) | Predicates per §1 tables; 7 Pattern-A + 2 v2 + 1 BridgeLedgerZero + 5 substrate-gap + ~9 demonstration gates = ~70 total | RATIFIED |
| Q2 | 5 substrate-gap classes — separate lanes vs closure-criteria-over-existing-lanes | Closure-criteria framing (§1.4 + §4) | RATIFIED |
| Q3 | v2 retirement Mgr ownership | PB Mgr owns all (existing T-V2-Retirement scope) | RATIFIED |
| Q4 | Mgr structure (keep 9 vs reorganize) | Keep 9 standing Mgrs with re-anchored scope | RATIFIED |
| Q5 | Free-consequences-demonstration as closure gate | Required as gate; **generalized to demonstration principle** (§1.6) — every feature comes with a demonstration | RATIFIED + EXTENDED |
| Q6 | Plan-as-doc shape (this doc vs r3-structure.md merge) | New doc (this); cross-link both directions | RATIFIED |
| Q7 | Drift items reconciliation routing (#1638, #1499, declaration_by_name) | Single Debt-Paydown worker reconciles all three → one PR | RATIFIED |

### §10.2 Per-Mgr canvas escalations (compiling as canvases land)

#### §10.2.1 R3 Verification Mgr (cool-owl-579, #1740) — 2 escalations

**V1. Pattern A (DimensionReport-typed family) blocks executable / net bridge shrink** (canvas item 1, structural)

**Note** (Verification Mgr poke-hole 2026-05-06): V1's DimensionReport-unblock fixes TC1/TC2/TC3 cluster but does NOT automatically fix **V2** (SymbolicCostExprEquals — different predicate family, `SymbolicCost`-typed, distinct runner work) NOR **BridgeLedgerZero** (Pattern E ledger-count ratchet, separate retirement-work cascade). Each Pattern-A predicate family has its own unblock path per §1.1 + §2.1.

**V1 (continued)**:
- **Needs**: Director + Brian — which first executable slice (TC1 vs RustDagIsomorphism vs other) + lane priority.
- **R3 zero-debt impact**: Must be explicit pre-close. Cannot be "post-R3" without renaming the close predicate.
- **Plan integration**: §1.1, §2.1; ratification opens Q-PAFS in §10.3.

**V2. SymbolicCostExprEquals reads as coverage while runner returns NYI** (canvas item 2, structural)
- **Needs**: coverage/completeness accounting decision — mark NYI claims incomplete in aggregation OR schedule evaluator-backed comparison slice.
- **Plan integration**: §1.1; ratification opens Q-NYI-Accounting in §10.3.

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

#### §10.2.3 R3 Substrate Mgr — pending canvas response (as-of 2026-05-06T00:30Z)

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
