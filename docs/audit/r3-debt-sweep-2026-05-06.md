# R3 Debt Sweep — 2026-05-06 (Phase 1 organize 2026-05-09)

**Status:** **Framework draft + initial Class A-G + new Class P populated 2026-05-09** — Phase 1 organize cycle completed by Debt-Paydown Mgr (gentle-newt-665) per PM dispatch at gunb-ai/gunbc#846 (Brian operator directive 2026-05-09 ~20:30Z; lane scope expanded to all v3 debt + active posture). Class A-G rows seeded from `docs/audit/v3-comprehensive-debt-audit-2026-05-09.md` 10-source inventory + ROADMAP `## Tracked debts` rows + §10.3 §1.8 ledger cross-references + 4 stranded bug-fix briefs (PR #2373) + 6 gpt-5-5-pro novel findings (PR #2358 §8). Class P NEW per Brian directive — past-R3 / R4+ ambition parking lot. Mgr re-organize cycle continues; row population is initial pass, not exhaustive.

**Authority:** Brian-ratified scope at gunbc#828 (Director-relayed):
1. **v2 retirement folded into R3** (was separate Pure-Bootstrap-Zero / T-PB-A + T-PB-B program)
2. **Staffing not bottleneck**; comprehensive sweep (no partial); audit covers full v3 codebase + R2/R3-era PR history + ROADMAP + recent analyses

**R3 closure criteria (Brian-ratified)**: all 5 substrate-gap classes closed + v2 fully retired + Pattern A NYI predicates executable + BridgeLedgerZero ratcheting at zero.

**Methodology:** PM-coordinated, Mgr-parallelized canvas + compilation. Per `feedback_corrections_must_grep_verify_source`: all claims grep-verified; per `feedback_section_anchors_over_line_numbers`: cross-references use section/symbol anchors; per `feedback_modeling_inversion_and_paydown_flow`: RED items get inversion-test before flagging out-of-scope.

---

## §1 Comprehensive bridge inventory

Per Director's bridge-class framework (A-G). Populated post-Mgr-canvas (Phase 2-3).

### Schema

Each bridge has one row in the inventory:

| Bridge | Class | File / location | Dissolution trigger | R3-close eligibility | Source PR | Owner Mgr |
|---|---|---|---|---|---|---|
| `<bridge name or symbol>` | A-G | `<file path or symbol anchor>` | `<named trigger that fires at R3 close>` | GREEN / YELLOW / RED | `#NNNN` | Substrate / Verification / Evaluator / PB / Debt-Paydown / Grounding |

Column rules (load-bearing):

- **Bridge**: name or canonical symbol; not a free-text description. Examples: `CardinalityPayload::new_unchecked`, `SourceSpan.file participation checks`, `EmissionDiagnostic Rust mirror`.
- **Class**: single letter A-G per Director's framework (below). No "multi-class" entries — pick the load-bearing class; cross-reference others in the trigger column if structurally coupled.
- **File / location**: section/symbol anchor preferred; line numbers permitted only when no symbol name exists at the cited point.
- **Dissolution trigger**: named structural event that ends the bridge's existence (e.g., "T-V2-Retirement G-2 deletion landing", "R3 Substrate substrate-gap-class-3 closing", "PR #XXXX merging"). Not heuristic; not "eventually."
- **R3-close eligibility**: GREEN / YELLOW / RED per §3 rubric below.
- **Source PR**: PR that introduced the bridge, if traceable; "pre-R2" if older than R2 lane structure.
- **Owner Mgr**: which R3 Mgr's lane the bridge sits in (or coordinates-to). Single owner; cross-program coupling noted in trigger column.

### Class A — Substrate-gap-blocked (~15-20 expected)

Bridges that exist because a substrate carrier or grammar surface isn't yet modeled. Wait for parser/grammar surface, function-valued data, file-ingestion, workflow/scheduling, reflection-closure.

Initial population 2026-05-09 per Brian-directive lane scope expansion (PM dispatch at gunbc#846 → gentle-newt-665 organize cycle):

| Bridge | Class | File / location | Dissolution trigger | R3-close eligibility | Source PR | Owner Mgr |
|---|---|---|---|---|---|---|
| `parse_parser_body.txt` 1350-LOC recursive-descent algorithm | A | `src/v3/compiler/parse_parser_body.txt` | SG-2b proper `parse.dag` ownership lands | YELLOW | #589 | Substrate |
| `Bool` inhabits `BooleanAlgebra<Bool>` not wired (Class 5 Gap 1) | A | `dsl/std/bool.dag` + lower.rs | 1e-2b lane Path A landing | YELLOW | pre-R3 | Substrate |
| `data` body shape boundary (Class 5 Gap 3 — ValueBody extension) | A | `lower.rs:3514-3604` + emit.rs ValueBody::Structural | ValueBody substrate extension lane | YELLOW | pre-R3 | Substrate |
| Function-valued data not first-class (Class 2 Gap) | A | `dsl/std/algebra.dag:168-181` Semiring<SymbolicCost> deferred witness | T-E-P-Producer-Broadening Phase 1 / S10 | DECLARED §1.8 #61 | pre-R3 | Substrate |
| File-ingestion substrate gap (`.dag` consumes external files w/o include_str) | A | T-Workflow-As-Data lane | §1.8 #62 substrate_gap_file_ingestion_closed | DECLARED | pre-R3 | Substrate |
| Workflow/scheduling substrate gap (CI workflow as `.dag` data) | A | T-Workflow-As-Data + T-Lens-Self-Application | §1.8 #63 substrate_gap_workflow_scheduling_closed | DECLARED | pre-R3 | Substrate |
| Reflection-closure substrate gap (`lens_apply.rs` reflection via PB-Runtime) | A | T-LensProducer-Retirement | §1.8 #64 substrate_gap_reflection_closure_closed | DECLARED | pre-R3 | PB |
| Parser/grammar substrate gap (typed numerics + algebra-machine-constraint) | A | `src/v3/spec/rust.dag` + numeric carriers | §1.8 #60 substrate_gap_parser_grammar_closed (Int<64>/Real<64>/Nat<8> evidence) | DECLARED | pre-R3 | Substrate |
| `IntLit` host-narrowed at tokenizer boundary; `i64::MIN` unrepresentable | A | `src/v3/std/tokenize.dag:30` IntLit(Int) | unbounded-magnitude IntLit at concept layer; reconciliation-narrows | YELLOW | pre-R3 | Substrate |
| Domain refinements regressed pending Q-Regex-Primitive | A | `dsl/std/types.dag` CommitSha/Sha256/Email/Url/SemVer/Timestamp/MimeType | regex/content predicate grounding | YELLOW | pre-R3 | Substrate |
| `KNOWN_PREDICATES` Rust-side semantic authority (string carrier names) | A | `lower.rs` KNOWN_PREDICATES | substrate-reflection / `.dag` predicate declarations | YELLOW | abf5f24d | Substrate |
| Constructor ambient prefix scan in lower.rs | A | `lower.rs` payload-lookup | `TransformTarget::Callable` outer-type metadata | YELLOW | pre-R3 | Substrate |
| PathCall resolution source-order fragile | A | PathCall lowering tests | order-independent ValueBody population OR explicit declaration-order semantics | YELLOW | pre-R3 | Substrate |
| `MethodContract.size_effect` substrate-shape | A | `dsl/std/method_contract.dag` | typed size-effect carrier | YELLOW | pre-R3 | Substrate |
| `MethodContract.cost_shape` substrate-shape | A | `dsl/std/method_contract.dag` | typed cost-shape carrier (post-T-CostLens) | YELLOW | pre-R3 | Substrate |
| `MethodContract.callback_element_position` substrate-shape | A | `dsl/std/method_contract.dag` | typed callback-position carrier | YELLOW | pre-R3 | Substrate |
| Character-level under-consumption in tokenize + syntax | A | tokenize.dag + syntax authorities | character-class substrate refinement | YELLOW | #693 | Substrate |
| Unit-mismatch enforcement for phantom Unit/Currency typed wrappers | A | typed value wrappers | phantom-parameter unit-discipline carrier | YELLOW | pre-R3 | Substrate |
| `Secret<T>` nominal-wrapper graduation | A | nominal-opacity carriers | Secret PR A continuation per #900 | YELLOW | #900 | Substrate |

### Class B — Pattern-A NYI predicates (~7 expected)

Test predicates declared but `NotYetImplemented`-shaped runtime: TC1, TC2, TC3, free-consequences, RustDagIsomorphism, BridgeLedgerZero, SymbolicCostExprEquals.

| Bridge | Class | File / location | Dissolution trigger | R3-close eligibility | Source PR | Owner Mgr |
|---|---|---|---|---|---|---|
| TC1 η-equivalence executable (#11) | B | `r3-program-plan.md:212` §1.8 #11 | Evaluator E3.c #1970 merge → DECLARED→PASSING | DECLARED (HOLD branch B) | #2184 | Verification |
| TC2 Church-Rosser executable (#12) | B | §1.8 #12 | second strategy/input order + strategy-keyed report | DECLARED | pre-R3 | Verification |
| TC3 Pattern-A second-mover executable (#13) | B | §1.8 #13 | Descent execution proof (E5) + eval-step producer | DECLARED | pre-R3 | Verification |
| RustDagIsomorphism executable (#14) | B | §1.8 #14 | shape-report producers | DECLARED | pre-R3 | Verification |
| BridgeLedgerZero ratchet ledger-count (#36) | B | §1.8 #36 | unified ledger reports 0 | DECLARED | pre-R3 | Debt-Paydown |
| SymbolicCostExprEquals executable (#40) | B | §1.8 #40 + test_runner.rs | cost-lens consumer per ε path | DECLARED | pre-R3 | Verification |
| Free-Consequences demonstrations (#43-#52, 10 gates) | B | §1.8 #43-#52 | bind-independence/dependence parallelism + auto-memoization demos | DECLARED | pre-R3 | Verification |
| `resolve_producer_opt` typed return (P3 fail-closed violation) | B | brief: `r3-bug-resolve-producer-opt-typed-return-worker.md` | dispatch worker; typed Result<Producer, Reason> | YELLOW | #2373 brief | Verification |

### Class C — Pattern-C typed-carrier + Rust-mirror (~6+ expected)

Rust mirrors of `.dag` typed carriers that should dissolve when v3 evaluator authoritatively executes the .dag side: EmissionDiagnostic, Value, EvalStrategy, BridgeLedgerRef, target-primitive routing.

| Bridge | Class | File / location | Dissolution trigger | R3-close eligibility | Source PR | Owner Mgr |
|---|---|---|---|---|---|---|
| u128 grounding-pilot Rust mirror drift | C | `src/v3/grounding_pilot/src/lib.rs:6-8,256-265` ↔ `dsl/extdeps/languages/rust/primitives.dag:274-277` | dispatch worker brief; ratchet test or structural consumption | YELLOW | brief #2373 | Substrate |
| FieldProject dual-authority (`field_label` ↔ `field_child` disagreement) | C | `dag.rs:1890-1892` + `infer.rs:4198-4208` | UnresolvedFieldProject/ResolvedFieldProject lifecycle split | YELLOW | brief #2373 | Substrate |
| CallGraph forward-only authority (illegal-state-representable in product) | C | brief: `r3-bug-callgraph-forward-only-authority-worker.md` | dispatch worker | YELLOW | brief #2373 | Substrate |
| `MissingEmissionPath` stringifies typed axes (connective/behavior/target as String) | C | gpt-5-5-pro Finding 1 | typed-carrier diagnostic boundary | YELLOW | PR #2358 | Substrate |
| `ComposedEffect { idempotent, breaking_operation }` illegal product | C | gpt-5-5-pro Finding 5 | sum-typed effect-composition carrier (`CompositionVerdict` in `dsl/std/effects.dag` + v2 stage0) | GREEN | PR #2491 / #2469 | Substrate |
| `derive_op_effect(method_str, path_str)` string parser at structural boundary | C | gpt-5-5-pro Finding 6 | typed dispatch over Method × Path carriers | YELLOW | PR #2358 | Substrate |
| `EmitModelVariants` manually-enumerated cache (label-string mirror) | C | `dag.rs` EmitModelVariants | generate from `emit_model.dag` OR exhaustiveness ratchet | YELLOW | pre-R3 | Substrate |
| `EmissionDiagnostic` Rust mirror | C | `src/v3/compiler/src/diagnostics.rs` | v3 evaluator authoritative on diagnostic emission | YELLOW | pre-R3 | Substrate |
| `Value` Rust mirror | C | `src/v3/compiler/src/value.rs` | v3 evaluator value-domain authority | YELLOW | pre-R3 | Substrate |
| `EvalStrategy` Rust mirror | C | `src/v3/compiler/src/eval/` | strategy-keyed `.dag` authority | YELLOW | pre-R3 | Substrate |
| Reflected witness carriers forgeable (ParamRef/TransformRef/ElementRef/BoolPortRef) | C | `src/v3/std/substrate.dag:337-374` + `effects.dag:447-530` | substrate-level proof-carrying handles or constructor parity | YELLOW | b09e0c8 | Substrate |
| `ValueBody` Rust↔.dag mirror drift (no isomorphism gate; partially CLOSED via #2288) | C | gate #96 substrate_value_body_mirror_isomorphism | further Tier-3 retirement | CONSUMER_LANDED §1.8 #96 | #2288 | Substrate |
| `FieldMap` duplicate-free invariant lost in `.dag` mirror | C | NOVEL row 2026-04-30 | duplicate-free typed carrier | YELLOW | pre-R3 | Substrate |
| Operator inference `(T,T)→T` arrow fabrication on algebra-walk failure | C | gpt-5-5-pro NOVEL 2026-04-30 | fail-closed structural diagnostic | YELLOW | pre-R3 | Substrate |
| Duplicate record-literal fields silently accepted | C | NOVEL highest-value 2026-04-30 | typed second-write fail-closed | YELLOW | pre-R3 | Substrate |

### Class D — Generated bridges with freshness gates (~3-5 expected)

Generated artifacts that would be debt without freshness ratchets but are bounded scaffolds with explicit regeneration gates: `render_repeat_string_bootstrap.dag`, `generated_method_template_projection.rs`. **Not debt by intent**; freshness gate prevents drift. May still be GREEN at R3 close if R3-close includes v2 retirement (which it now does per Brian-ratified fold-in).

| Bridge | Class | File / location | Dissolution trigger | R3-close eligibility | Source PR | Owner Mgr |
|---|---|---|---|---|---|---|
| `render_repeat_string_bootstrap.dag` generated scaffold | D | `dsl/std/render_repeat_string_bootstrap.dag` | v2 retirement closes | GREEN | pre-R3 | Substrate |
| `generated_method_template_projection.rs` | D | grounded mirror | v2 retirement closes | GREEN | pre-R3 | Grounding |
| `slow-test-exemptions.txt` 41-row meta-ratchet | D | `scripts/slow-test-exemptions.txt` | per-exempt budget Phase 2 ratchet (#2322) | YELLOW | pre-R3 | Debt-Paydown |
| Peano materialization cap `256` literal in 3 authorities | D | `dsl/std/termination.dag:238-243` + `src/v3/std/termination.dag:138-140` + `dag.rs:1022-1027` | generate Rust cap from `.dag` OR ratchet test | YELLOW | b09e0c8 | Substrate |

### Class E — v2 ↔ v3 transition bridges (~10+ expected; **now in R3 scope per fold-in**)

Bridges that exist solely because v3 isn't yet authoritative for some surface that v2 still owns: `emit_model.dag` facade, method-template legacy adapters, v2 transport infrastructure. Per Brian's R3-fold-in ratification, these are R3-close eligible (GREEN if v2 retirement closes; previously would have been "post-R3 / Pure Bootstrap").

| Bridge | Class | File / location | Dissolution trigger | R3-close eligibility | Source PR | Owner Mgr |
|---|---|---|---|---|---|---|
| `emit_model.dag` facade duplicate authority | E | `dsl/v3/std/emit_model.dag` | T-V2-Retirement | YELLOW §1.8 #41/#42 | pre-R3 | PB |
| Method-template legacy adapters | E | emit_model + lower bridge | T-V2-Retirement | YELLOW | pre-R3 | PB |
| v2 transport infrastructure | E | `src/v2/transport/` | §1.8 #42 v2_directory_deleted | DECLARED | pre-R3 | PB |
| v2 oracle test consumers | E | tests consuming `src/v2/` | §1.8 #41 v2_oracle_no_remaining_test_consumers | DECLARED | pre-R3 | PB |
| `emit/rust_target.rs` v2-shape Class E (per PR #1804 §1.2) | E | grounded carrier | post-EmissionPathProjection / Anthropic post-projection-metadata | YELLOW | #1804 | Grounding |
| `emit/python_target.rs` v2-shape Class E | E | grounded carrier | as above | YELLOW | #1804 | Grounding |
| `collection_ops_method_contract.rs` v2-shape Class E | E | grounded carrier | as above | YELLOW | #1804 | Grounding |
| Pipeline dual-authoring drift (post-#1171 source-text bridge retirement) | E | pipeline.dag + pipeline-rust mirror | full pipeline-as-data migration | YELLOW | post-#1171 | PB |
| Method-template consumer migration over row population priority | E | method_template adapters | T-Tier3-Dissolution lane | YELLOW | pre-R3 | PB |
| Tier3 termination mirror (gate #1) | E | §1.8 #1 | T-Tier3-Dissolution | DECLARED | pre-R3 | PB |
| Tier3 computation mirror (gate #2) | E | §1.8 #2 | T-Tier3-Dissolution | DECLARED | pre-R3 | PB |
| Tier3 induction mirror (gate #3) | E | §1.8 #3 | T-Tier3-Dissolution | DECLARED | pre-R3 | PB |
| Tier3 effect-carrier mirror (gate #4) | E | §1.8 #4 | T-Tier3-Dissolution | DECLARED | pre-R3 | PB |
| `lens_apply.rs` retired (gate #5) | E | §1.8 #5 | T-LensProducer-Retirement (PB-Runtime interpreter-as-data) | DECLARED | pre-R3 | PB |
| `lens_testgen.rs` retired (gate #6) | E | §1.8 #6 | T-LensProducer-Retirement | DECLARED | pre-R3 | PB |
| `regen_lens.rs` retired (gate #7) | E | §1.8 #7 | PB-1 bin-shim emit pattern | DECLARED | pre-R3 | PB |
| SG-0 non-test zero (gate #8) | E | §1.8 #8 | EXPECTED_HAND_AUTHORED_NON_TEST count = 0 | DECLARED | pre-R3 | PB |

### Class F — Operator/algebra ontology duplication (~3 surfaces expected)

Duplicate representations of operator/algebra concepts: OperatorSpec / OperatorKind / BinaryOpRow. Should consolidate to single ontology.

| Bridge | Class | File / location | Dissolution trigger | R3-close eligibility | Source PR | Owner Mgr |
|---|---|---|---|---|---|---|
| OperatorSpec / OperatorKind / BinaryOpRow triplicate | F | parse_tables.dag + lower + emitter operator surfaces | SG-2c proper `parse_tables.dag` decision; single ontology | YELLOW | pre-R3 | Substrate |
| `algebra.dag` declaration-vs-template surface asymmetry | F | `dsl/std/algebra.dag` | rescoped per Lane G #654 investigation | YELLOW | #654 | Substrate |
| `ShapeATarget = Rust \| Python \| Go` closed enum vs `LanguageSpec` data extensibility | F | gpt-5-5-pro Finding 2 | `ShapeATarget { spec: DeclarationRef }` carrier, fail-closed refined by consumers to `LanguageSpec`; cross-target consumers enumerate loaded `LanguageSpec` declarations | GREEN | issue #2466 / crisp-raven-202 | Grounding |
| `Map<String, Bool>` as set across graph/syntax/node files (`set_has` ignores stored bool) | F | gpt-5-5-pro Finding 3 | use declared `Set<A>` from std | YELLOW | PR #2358 | Substrate |
| `StructuralEffectShape.combine_effects` bespoke local lattice merge | F | `src/v3/lenses/effect_enumeration.dag:50-54,175-207` | `BoundedLattice<StructuralEffectShape>` instance | YELLOW | b09e0c8 | Substrate |
| Hand-rolled lattice instances (NEW class instance) | F | tracked at ROADMAP `:384` | typed `Lattice<T>` instances | YELLOW | pre-R3 | Substrate |
| Anthropic schema mirror multiplication risk (provider/API class) | F | Anthropic schema + OpenAI ratchets + GitHub | typed-end-to-end serde alignment §1.8 #29/#30 | DECLARED | pre-R3 | Grounding |
| `container_template_algebra_rows` string-table duplicate authority | F | `dsl/std/types.dag:133-146` + aliases `:211-213` | `.dag` alias reflection lands | YELLOW | #651 | Substrate |

### Class G — Local/small bridges (~5-10 expected)

Smaller scope bridges: F10 install_hint, ad-hoc fold sentinels, etc.

| Bridge | Class | File / location | Dissolution trigger | R3-close eligibility | Source PR | Owner Mgr |
|---|---|---|---|---|---|---|
| `cross_target_coverage.dag` doc/code drift (RETIRED) | G | `src/v3/std/cross_target_coverage.dag` + sg0_census_test.rs comments | RETIRED via PR #2333 (2026-05-09) | RETIRED | #2333 | Substrate |
| `Strict Forward Progress` subdoc ↔ heading drift | G | `docs/invariants/strict-forward-progress.md` | naming-design call + rename or split | YELLOW | pre-R3 | PM |
| `IntegrationRsScan` raw/byte-string panic-loud mode | G | `src/v3/compiler/tests/integration/common/mod.rs` | widen scan for raw/byte strings or replace with structural reader | YELLOW | #1856 | PB |
| Stale-receipt sweep in `docs/briefs/` (PARTIAL T-Receipts W1) | G | `docs/briefs/` broad sweep | post-W1-wave staleness sweep | YELLOW | T-Receipts W1 | Debt-Paydown |
| `__EMIT_BUG_{0}__` target emit-spec sentinels (rescoped) | G | `src/v2/05_emit.dag:996,1046` | refactor to Diagnostic + halt; sentinels become unreachable | YELLOW | #657 | PB |
| `extdeps/browser.dag` typed-carrier service-boundary collapse | G | `dsl/extdeps/browser.dag` | typed service-boundary carriers | YELLOW | pre-R3 | Substrate |
| Filename / sentinel bridges in `test_runner.rs` | G | `test_runner.rs` (parallel test-predicate authority) | freeze new bespoke arms; convert ratchet to count-decreasing | YELLOW | pre-R3 | Evaluator+PB |
| Lossy user-lens reflection vs full substrate (PARTIAL) | G | lens reflection paths | full-substrate reflection landing | YELLOW | pre-R3 | Substrate |
| Duplicated declaration walkers + `SubstStack` between `lower.rs`/`infer.rs` | G | `lower.rs` + `infer.rs` | shared walker abstraction or substrate-driven dispatch | YELLOW | NOVEL | Substrate |
| Closed-vocabulary consumer proof — C1 stop ratchet | G | meta-review follow-up | NOVEL row | YELLOW | pre-R3 | Substrate |
| `go_method_template_contracts` live diagnostic mismatch | G | go_method_template_contracts | diagnostic alignment | YELLOW | pre-R3 | Grounding |
| "diagnostics empty" acceptance gate missing for new bootstrap fixtures | G | bootstrap fixtures | gate addition | YELLOW | pre-R3 | Verification |
| Lens fold execution: undeclared fallback structure + file-path semantics | G | `src/v3/compiler/src/lens_apply.rs:105-148,22-39,372-383` | structural R1-certified fold shape carrier | YELLOW | NOVEL 2026-04-25 | Substrate |
| `PartitionResult` bypassed by anonymous return type (RETIRED) | G | `dsl/std/filesystem.dag:83` | RETIRED via #2468 worker: `partition_entries` now returns named `PartitionResult` | GREEN | #2468 | Substrate |
| 4 TODO/FIXME markers in `src/v3/` non-test | G | `grep -rn 'TODO\|FIXME\|XXX' src/v3/` | per-marker dissolution | YELLOW | pre-R3 | Substrate |
| SG-0 census untagged entries (no dissolution-trigger comments) | G | `sg0_census_test.rs`; live counts in `docs/audit/sg0-census-classification-2026-05-09.md` §Summary | per-entry §1.8 gate-id linkage + Class A-G ratification (script: `scripts/classify-sg0-census.py`) | YELLOW | PR #2358 §1 | Debt-Paydown |
| Hand-Rust acceptance growing faster than `.dag` TestClaim migration | G | per-test SG-0 ratchet | net-zero/net-shrink discipline; per-PR pairing | YELLOW | pre-R3 | PB+Debt-Paydown |
| Reflection completeness over-trusted (not closed mechanical theorem) | G | reflection-completeness claim | mechanical theorem or ratchet | YELLOW | pre-R3 | Substrate |

### Class P — Past-R3 debt (R4+ ambition; lane scope per Brian directive 2026-05-09)

NEW class introduced 2026-05-09 per Brian operator directive: substrate-capability gaps that don't block R3 close but do block R4+ ambition. Inventory parking lot for forward debt-track without R3-blocking framing. Eligibility column = "P" (past-R3).

| Bridge | Class | File / location | Dissolution trigger | R3-close eligibility | Source PR | Owner Mgr |
|---|---|---|---|---|---|---|
| C1 Parallelism lens (R4-carve) | P | `docs/r4-carve-out-routing.md` C1 | Stage 2e walker port substrate; R4 lane | P (R4) | r4-carve-out | Substrate |
| C2 Effect-enumeration lens (R4-carve) | P | `r4-carve-out-routing.md` C2 | 4c carrier P1 substrate-fact-introduction | P (R4) | r4-carve-out | Substrate |
| C3 Register zero-proxy/zero-stub broader (R4-carve) | P | `r4-carve-out-routing.md` C3 | post-Q-LBP narrowing; complexity+cost in R3 only | P (R4) | r4-carve-out | Verification |
| C4/C5/C6 substrate-axis defers | P | `r4-carve-out-routing.md` C4-C6 | substrate-axis lane R4 prep | P (R4) | r4-carve-out | Substrate |
| TC1 #11 canvas-deferred (post-R3 substrate) | P | TC1 generic G1.b / X1.b path B | post-R3 generic + eta relation | P (R4) | pre-R3 | Verification |
| ProgramShape richer-shape variants | P | design §8.2 | post-R3 design call | P (R4) | pre-R3 | Substrate |
| Tier 4 ("out of scope, declared") inclusion | P | `r3-program-plan.md` §10.3 Q-Tier4-Inclusion | Brian directive (HOLD pending) | P (R4) | ctrl#1608 §6d | PM |
| Q-Class-6-Substrate-Extension-Lens (3 ctrl-side proposals) | P | §10.3 Q-Class-6 | Research PM sharper framing + Director ratification | P (R4-pending) | ctrl#370/#404/#405/#407/#408 | Research PM |

---

## §2 Hidden debt audit

Items NOT currently in ROADMAP that should be tracked. Sources:
- SG-0 census per-entry justification (live count via `EXPECTED_HAND_AUTHORED_NON_TEST` + `EXPECTED_HAND_AUTHORED_TEST` + `EXPECTED_HAND_AUTHORED_FRAGMENTS` arrays in `src/v3/compiler/tests/integration/sg0_census_test.rs`; per-entry: is it bridge or pre-existing STRUCTURAL apparatus? if bridge, dissolution trigger?)
- Worker-inbox findings not yet escalated (Mgr canvas surfaces from per-Mgr inbox histories)
- Recent analyses items not yet rowed (cross-reference gpt-5-5-pro Exploratory main@41019a8 + Reflective main@2b7b21b + earlier paired analyses 2026-04-25 / 2026-04-30 / 2026-05-01)

[Populated post-Mgr-canvas + PM compile pass]

---

## §3 R3 closure-eligibility rubric

Three valid statuses per item. Each item in §1 has a defined status; the count of RED is the load-bearing signal for Director review.

### GREEN — dissolves at R3 close per [trigger]; in scope

**Criteria:**
- Item has a **named structural dissolution trigger** that fires at R3 close (e.g., specific lane landing, substrate carrier landing, PR merging)
- No upstream prerequisite that's RED
- Trigger is **structurally** verifiable, not heuristic ("when X lands" verifiable; "eventually" not)
- Source PR known + owner Mgr identified

**Example shape** (post-population): `CardinalityPayload::new_unchecked` → trigger: "elimination per Path B (idempotence-inline rename); landing at R3 Substrate Mgr dispatch under SG-0-delta discipline"

### YELLOW — dissolves at R3 close IF [structural prerequisite] lands; tracked

**Criteria:**
- Item dissolves IF a **named structural prerequisite** lands in R3
- Prerequisite is itself in R3 scope (GREEN or YELLOW with a clear chain back to GREEN)
- **YELLOW chain rule** (per Director ratification 2026-05-06 at gunbc#828 #issuecomment-4384070969): YELLOW prerequisites must trace back to GREEN within a finite chain. Two YELLOW items listing each other (or a longer cycle) are NOT valid — cycle detection mandatory before flagging YELLOW.
- Tracked with the prerequisite chain explicit; no "we'll figure it out" prerequisites
- Source PR + owner Mgr identified

**Example shape** (post-population): `EmissionDiagnostic` Rust mirror → trigger: "v3 evaluator authoritatively executes std diagnostic constructors"; prerequisite: "T-V2-Retirement G-2 deletion landing (which gates on T-FixedPoint + T-LensProducer-Retirement)"

### RED — no clear path / unknown / requires substrate-gap escalation; flagged for Director review

**Criteria** (any of):
- No clear dissolution path identified
- Requires substrate-gap escalation to a named R3 lane (per Brian directive 2026-05-02 + `docs/r3-structure.md` §"Standing program — R3 Debt-Paydown" — *"there is no post-R3 deferral path for tracked debt; if a row appears unretirable within R3 it surfaces as a substrate gap requiring a named R3 lane, not a deferral"*)
- Unknown / unclassified — flagged for Director investigation

**No "post-R3 deferral" RED option** — Brian's R3 zero-debt close criteria + the no-post-R3-deferral discipline (per `docs/r3-structure.md` §"Standing program — R3 Debt-Paydown") mean RED items cannot resolve via "explicitly out-of-R3-scope by design"; they must either surface as named R3 work, escalate as substrate-gap requiring R3 lane, or carry explicit Director allocation. Earlier framing of "explicitly post-R3 / out-of-current-scope by design" as RED criterion conflicted with the canonical no-post-R3-deferral policy and is removed (per codex BLOCKING on PR #1808 sha `424a921a`, 2026-05-06).

**RED count is the load-bearing signal.** Each RED needs Director-level investigation per Brian's framing: "are these honestly substrate-gap-requiring-R3-lane, or are they 'we don't know how to close this'?"

### Inversion-test discipline (per `feedback_modeling_inversion_and_paydown_flow`)

Before flagging an item RED, apply inversion test:
- Forward framing: "what dissolves this bridge?" → if no answer, RED candidate
- Inverse framing: "what would let this bridge persist?" → if the inverse forces a structural fix, the bridge is actually GREEN/YELLOW with a reframed dissolution shape

If inversion dissolves the case → status updates to GREEN/YELLOW. If inversion doesn't dissolve it → RED is structurally honest.

### §3.A STRUCTURAL grandfathering — pre-existing apparatus exemption (per openai-pro NON-BLOCKING 2026-05-06 at PR #1804 #issuecomment-4384427761)

A fourth status exists alongside GREEN / YELLOW / RED for **pre-existing STRUCTURAL infrastructure only** — the SG-0 measurement apparatus itself (the sg0_census_test.rs framework, its expected-arrays, and the test-runner that uses them). This apparatus is hand-authored Rust by necessity (it measures hand-Rust at the test boundary; it cannot itself be generated without dissolving the measurement primitive). It is grandfathered from the GREEN/YELLOW/RED rubric pending explicit Director ratification of "STRUCTURAL" as a permanent status.

**Bounds — what counts as STRUCTURAL grandfathered**:
- The SG-0 measurement apparatus (`src/v3/compiler/tests/integration/sg0_census_test.rs` + its expected-array authority arrays + the test framework it depends on)
- NOT generic "compiler scaffold I think is load-bearing forever" — that's an escape hatch and explicitly disallowed per §4 going-forward contract

**Going-forward (post-framework-merge) STRUCTURAL additions**: require explicit Director allocation citing the program shape (per §4 Refinement B). There is no implicit-pass; a Mgr cannot self-classify an addition as STRUCTURAL without Director ratification.

**Interaction with §3 GREEN/YELLOW/RED**: STRUCTURAL is NOT a fourth ledger-counted status (does not contribute to ledger-zero count per §1.3 inclusion rules). It's an explicit out-of-ledger category for the measurement primitive itself. R3 close-criteria treat STRUCTURAL items as out-of-scope for `bridge_retirement_ledger_zero` predicate but in-scope for the measurement framework's continued existence.

**Boundary with substrate-gap-class closures**: STRUCTURAL apparatus is NOT eligible for the 5 substrate-gap-class closures (§1.4) — it's the measurement framework that detects gaps, not a gap itself.

**Status-flow if Director rejects "STRUCTURAL" status**: items currently in §3.A revert to YELLOW with explicit prerequisite "Director ratification of STRUCTURAL status as new R3-close ledger category." If neither STRUCTURAL ratification nor dissolution path materializes by R3 close, items become RED.

---

## §4 Anticipated debt — preventive lane

Going-forward discipline encoded as PR-authoring contract. Folds in Director's prior PR-template SG-0-delta extension routing to quiet-otter (R3 Debt-Paydown, gunbc#1744 #issuecomment-4383628247).

### Application scope (Refinement A per Director ratification 2026-05-06 at gunbc#828 #issuecomment-4384070969)

Contract applies to **PRs landing AFTER this framework merges** (not retroactively). Pre-existing SG-0 census entries are grandfathered; their bridge-class + dissolution trigger surfaces in §1 inventory but does NOT retroactively require (a)-(d) compliance below. Mgr canvases must NOT flag pre-existing entries as needing immediate dissolution-trigger receipt — only flag if the entry's existence is itself a discipline violation per the bridge-class framework.

### PR-authoring contract

**Every PR adding hand-Rust must same-PR answer (in PR description or commit message):**

(a) **Bridge class** — A-G per §1 framework above. SG-0 zero-floor discipline: hand-authored v3 Rust must dissolve same-PR, cite a named structural trigger (with R3-close eligibility GREEN/YELLOW per §3), or carry explicit Director RED allocation. **Intentional-permanent hand-Rust additions** (e.g., new SG-0 STRUCTURAL infrastructure) **require explicit Director allocation citing the program shape** (Refinement B per Director ratification 2026-05-06 at gunbc#828 #issuecomment-4384070969); there is no implicit-pass — a Mgr cannot self-classify an addition as STRUCTURAL without Director ratification. Pre-existing STRUCTURAL infrastructure (the SG-0 measurement apparatus itself; per §3.A) is grandfathered pending Director ratification of the new STRUCTURAL status.

(b) **Substrate-gap-class blocking dissolution** (if any) — name the gap that prevents same-PR dissolution. If no gap, the contract's options collapse to "dissolve same-PR" or "explicit Director RED allocation"; intentional-permanent additions without Director allocation are NOT allowed.

(c) **Same-PR dissolution OR named-trigger-with-R3-close-eligibility** — if dissolution doesn't land same-PR, name the trigger + R3-close eligibility (GREEN/YELLOW/RED per §3 rubric). RED requires Director-level allocation.

(d) **Net SG-0 delta** — positive only with explicit Director allocation citing gap-closure. Per Brian-ratified discipline: SG-0 ratchet is only-down except for Director-allocated additions tied to specific gap-closure.

### CI gate

[Routed to R3 Debt-Paydown (quiet-otter-416, gunbc#1744) per Director #issuecomment-4383628247 — the lightweight CI check that PR description carries (a)-(d) for PRs adding hand-Rust]

### PR-template extension

[Specifies (a)-(d) as required-fields for PRs adding hand-Rust; quiet-otter authors the template change]

---

## §5 Sources audited

- **v3 compiler**: ~69 `.rs` files in `src/v3/compiler/src/`
- **v3 std**: ~35 `.dag` files in `src/v3/std/`
- **SG-0 census**: live entries via `EXPECTED_HAND_AUTHORED_NON_TEST` + `EXPECTED_HAND_AUTHORED_TEST` + `EXPECTED_HAND_AUTHORED_FRAGMENTS` arrays in `src/v3/compiler/tests/integration/sg0_census_test.rs` (the SG-0 partition authority; do not snapshot the count here — derive at audit time from the live arrays per `feedback_corrections_must_grep_verify_source`)
- **R2/R3-era PR history**: ~500+ PRs (R2 close at #1275 → current HEAD ~#1803). Method: per-Mgr-lane parallel canvas (each Mgr surveys merged PRs in their lane subset for debt-introduction patterns + dissolution status). Pattern-audit, not per-PR-deep-audit.
- **ROADMAP**: open tracked items + DB-1 through DB-20 + W-C1 + scheduled deletions + Tracked debts 2026-04 + 2026-05 sub-sections + Course Corrections #1-#4 (ROADMAP §"Reflective course corrections" section anchor; per `feedback_section_anchors_over_line_numbers` — do not cite line numbers since ROADMAP edits drift them)
- **Recent analyses**: gpt-5-5-pro Exploratory main@41019a8 (F1-F11) + Reflective main@2b7b21b (Risks 1-5) + earlier paired analyses 2026-04-25 / 2026-04-30 / 2026-05-01

---

## Methodology notes (per session-discipline patterns)

- All claims grep-verified before encoding (per `feedback_corrections_must_grep_verify_source`)
- Cross-references use section/symbol anchors not line numbers (per `feedback_section_anchors_over_line_numbers`)
- Mgr canvases request grep-verified sourcing per item
- RED items get inversion-test before flagging out-of-scope (per `feedback_modeling_inversion_and_paydown_flow`)
- 8 cross-relay timing instances this session — apply sha-style timestamp pointers in routing claims

---

## Phase plan

**Phase 1 (current — framework draft)**: PM authors §1 schema + §3 rubric + §4 anticipation discipline. Surface to Director for ratification.

**Phase 2 (post-ratification)**: parallel Mgr canvas dispatch to the 6 R3 Mgr inboxes that PM has direct dispatch authority for in the PM subtree:
- R3 Substrate Mgr (quick-crab-830, gunbc#1739)
- R3 Verification Mgr (cool-owl-579, gunbc#1740)
- R3 PB Mgr (neat-bear-351, gunbc#1742)
- R3 Evaluator Mgr (merry-gull-128, gunbc#1743)
- R3 Debt-Paydown Mgr (quiet-otter-416, gunbc#1744)
- R3 Grounding Mgr (bold-ferret-748, gunbc#1745)

Identical message format. Each Mgr surfaces lane debt with grep-verified sourcing.

**Remaining R3 standing Mgrs** (per `docs/r3-structure.md` §"Manager structure" canonical 9-standing-Mgr count; the 3 not in PM subtree): **R3 Release Mgr** (owns T-Omni-Shape-B + R3 closure ledger + R3 demo coordination per `docs/r3-structure.md` §"Manager structure" line 223; may be R2 Release Mgr continuation) + **2 unenumerated others** — `docs/r3-structure.md` §"Manager structure" states "9 standing R3 managers (8 + Debt-Paydown)" but does NOT enumerate per Mgr explicitly beyond the 4 named modifications (Substrate continuation / Verification / R3 Release / Debt-Paydown). The remaining 2 of the canonical 9 are **not enumerated in canonical authority at HEAD** — Director-side enumeration TBD. Per codex BLOCKING 2026-05-06 finding 2 (P1 live-state): this is left as TBD per Director rather than guessed.

These 3 non-PM-subtree Mgrs (R3 Release + 2 TBD) **coordinate via Director** (zesty-bear-812, gunbc#828) per cross-Mgr cadence; their lane debt + canvas surfaces flow through Director-relayed comments. Per codex BLOCKING on PR #1804 line 209 (2026-05-06 finding 1): this clarification closes the silent-omission risk that 6 ≠ 9 implies (INVARIANTS P2 single-authority + P1 live-state).

No Mgr's lane debt is silently omitted — PM-subtree Mgrs report directly via canvas; non-subtree Mgrs report via Director relay. Director enumeration of the 2 TBD Mgrs is a Phase 3 compile-pass surface (not blocking; per `docs/r3-program-plan.md` §10.3 implicit deferral).

**Phase 3 (post-canvas)**: PM compiles Mgr responses + adds §2 hidden-debt audit + cross-references ROADMAP review + recent analyses. Surfaces compiled sweep for Director final ratification + R3 closure-criteria explicit list.

[Phase 1 ratification gates Phase 2; Phase 2 completion gates Phase 3]
