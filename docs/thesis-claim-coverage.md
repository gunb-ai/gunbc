# THESIS Claim Coverage Map — gunbc Compiler

**Status:** PROPOSAL pre-R1-close. Promotes to ACTIVE on R1 closure → R2 promotion transition per [`docs/r2-structure.md` §"Open calls" item 1](r2-structure.md). Authoritative **union receipt + disposition map** for THESIS claims; source-of-truth for each claim remains the cited gate / lane / authority doc.

**Last refresh:** 2026-04-27 against main HEAD `242c65d0`. Refresh discipline below.

## Purpose

Per `docs/r2-structure.md` Open call 1 (pre-promotion gate before ROADMAP promotion):

> *"The 'close-everything/post-R2-external-only' framing requires an explicit mapping from THESIS tiers to concrete R1/R2/post-R2 disposition, so no thesis claim is implicitly-positioned. Otherwise 'close-everything' is an assertion without audit."*

This doc is that audit. Every thesis claim in `THESIS.md §"Thesis claims — complete list"` (Tier 1 / Tier 2 / Tier 3 + categorical claims under Concept unifications, Epistemic stacking, Substrate shape, Free consequences, Omni-emission, Meta-process modeling, Self-hosting, Audience duality, Adoption model, Tests-as-data, Enumerable impossible-bug classes, Modeling discipline) is mapped to a disposition (R1-closed / R2-gated / post-R2-external) with gate name + evidence + status.

## How to use

- **R2 promotion gate:** Director reviews this doc before promoting `docs/r2-structure.md` to ROADMAP `## Release R2 Program` per Transition mechanics step 5. **Zero GAP rows** is the promotion bar; partial-status rows are notes, not blockers.
- **R1 close declaration:** R1 Closure Manager cross-references this doc when declaring all-gates-green; the 52 R1-closed claims should map 1:1 to lane gate green status.
- **R2 manager dispatch:** R2 managers reference this doc when scoping work; the 18 R2-gated claims should map to one of the 6 R2 manager programs.
- **Refresh:** at every release transition + on demand when THESIS claims author or change.

## Coverage statistics

| Disposition | Count |
|---|---|
| **R1-closed** | 55 |
| **R2-gated** | 13 |
| **post-R2-external** | 6 |
| **GAP (no disposition)** | **0** ✓ |
| **TOTAL CLAIMS** | 74 |

**Pre-promotion blocker count: 0.** Every thesis claim has a named disposition + documented evidence.

**Breakdown by Tier/Category:**

| Tier/Category | R1-closed | R2-gated | post-R2-external | Total |
|---|---|---|---|---|
| Core abstraction | 1 | 0 | 0 | 1 |
| Correctness is structural — meta-claim | 4 | 0 | 0 | 4 |
| Tier 1 (Structural correctness) | 3 | 2 | 0 | 5 |
| Tier 2 (Runtime safety) | 0 | 1 | 0 | 1 |
| Tier 3 (Verification) | 1 | 3 | 0 | 4 |
| Concept unifications | 2 | 2 | 0 | 4 |
| Epistemic stacking | 6 | 0 | 0 | 6 |
| Substrate shape | 4 | 0 | 1 | 5 |
| Free consequences | 3 | 1 | 0 | 4 |
| Omni-emission | 5 | 1 | 2 | 8 |
| Meta-process modeling | 3 | 0 | 0 | 3 |
| Self-hosting (3 facets + cost-of-change + census) | 5 | 0 | 0 | 5 |
| Audience duality | 4 | 0 | 0 | 4 |
| Adoption model | 2 | 0 | 3 | 5 |
| Tests-as-data | 4 | 0 | 0 | 4 |
| Impossible-bug classes (R1 + R2+) | 4 | 3 | 0 | 7 (incl. governance constraint) |
| Modeling discipline | 4 | 0 | 0 | 4 |

## Mapping (74 claims)

### Core abstraction

| # | Claim | Disposition | Evidence | Status |
|---|---|---|---|---|
| 1 | `.dag` is dependency modeling; parallelism default, sequential requires justification | R1-closed | T-P0 + T-Demo `fixture_integration_canonical` (parallelism demo) | green |

### Correctness is structural — meta-claim

| # | Claim | Disposition | Evidence | Status |
|---|---|---|---|---|
| 2 | Every correctness dimension is structural fact in program data model | R1-closed | T-LaneE (complexity), T-PB-A (ownership) gates | green |
| 3 | Proof/test surface structurally derived; T1/T2 compile-time, T3 generated from `TestClaim` | R1-closed | T-PB-B `pb_rust_tests_outside_residual_zero`; cascade promotion per `docs/design-pure-bootstrap-zero.md` | partial (T-TestGen runner gate-enabling) |
| 4 | Dimension system first-class user-extensible | R1-closed | T-LensAPI `user_authored_lens_compiles` + T-Demo `demo_user_authored_lens_rejects_violating_program` | green (lens compiles) / not-started (demo) |
| 5 | Structural derivation replaces testing/profiling/schema-validation cycle | R1-closed | T-TestGen runner + T-LaneE complexity lens | green |

### Tier 1 — Structural correctness

| # | Claim | Disposition | Evidence | Status |
|---|---|---|---|---|
| 6 | Type mismatches, field typos, non-exhaustive, bare containers, circular deps, stale imports, cross-target drift caught at compile | R1-closed | T-Emit `emit_omni_demo_fixtures_green` + T-Demo fixtures | green |
| 7 | CX gate: every recursive function terminates with proven bound | R1-closed | T-LaneE `complexity_merge_sort_is_nlogn`, `complexity_merge_sort_v3_matches_v2_oracle` | green |
| 8 | Coercion = emission: compiler reads target spec, translates; no separate coercion engine | R2-gated | ROADMAP §"Post-R1 Program — Grounding Completeness" T-Ground-Dissolve lane | blocked on T-Ground critical path |
| 9 | Ownership: compiler proves no aliased mutation in emitted code | R1-closed | T-LaneE E-family carrier port (ownership substrate) | partial |
| 10 | Grounding completeness: target primitives structurally modeled, inhabits-search coercion, fail-closed | R2-gated | ROADMAP §"Post-R1 Program — Grounding Completeness" T-Ground-Pilot through T-Ground-Dissolve | in-flight (pilot closed; Rust/Engine/Tests/Dissolve remain) |

### Tier 2 — Runtime safety

| # | Claim | Disposition | Evidence | Status |
|---|---|---|---|---|
| 11 | Division by zero, overflow, OOB, force-unwrap, partial functions proven safe or made total | R2-gated | `docs/r2-structure.md` Goal 4 (T-ImpossibleBugs unhandled-diagnostic-paths class); Tier 2 substrate | in-flight (Int division Result carrier PR tail active) |

### Tier 3 — Verification from structure

| # | Claim | Disposition | Evidence | Status |
|---|---|---|---|---|
| 12 | L4: emitted code executes and matches `.dag` evaluation | R2-gated | T-Ground-Tests lane | blocked on earlier T-Ground lanes |
| 13 | L5: same `.dag` produces same behavior in Rust/Python/Go | R2-gated | T-Ground-Rust / T-Ground-Python / T-Ground-Go lanes | pending/fill queue per T-Ground; Rust/Engine tails remain |
| 14 | L6: every structural form compiles to every target | R2-gated | T-Ground-Engine (inhabitance-search walker) | in-flight (ValueBody list/sum producer landed; Engine consumer active) |
| 15 | L7: operations obey declared algebraic laws | R1-closed | T-LensAPI `lens_composition_associative` | green |

### Concept unifications

| # | Claim | Disposition | Evidence | Status |
|---|---|---|---|---|
| 16 | Coercion cost = complexity | R1-closed | T-LaneE complexity lens (realization costs compose with `.dag`-level CX) | green |
| 17 | Coercion = emission | R2-gated | T-Ground-Dissolve dissolves coercion scaffolding | blocked on T-Ground critical path |
| 18 | Target language spec = transport spec = interpreter runtime | R2-gated | T-Ground lanes (Rust/Python/Go structural modeling) | in-flight |
| 19 | Idempotency + cancellation + redundancy = algebraic simplification | R1-closed | T-Demo impossible-bugs idempotency-violation demo | green |

### Epistemic stacking

| # | Claim | Disposition | Evidence | Status |
|---|---|---|---|---|
| 20 | Every concept node in ontological DAG rooted at minimal primitives; no opaque concepts | R1-closed | `dsl/std/algebra.dag` declared; substrate carries 6 connectives + 5 behaviors | green |
| 21 | Root primitives: Magma, Monoid, BooleanAlgebra, FreeMonoid<T> in `dsl/std/algebra.dag` | R1-closed | `dsl/std/algebra.dag` present; bootstrap-loaded | green |
| 22 | Concrete types attach by inhabitance; operations fall out, never declared separately | R1-closed | Type system evaluates inhabitance structurally; coercion-design.md T1 DONE | green |
| 23 | Epistemic chain IS the emission algorithm; special cases evidence ungrounded concepts upstream | R1-closed | E-family carrier port (T-LaneE) proves structurally | green |
| 24 | Math + domain primitives share one substrate; user types declared same way as `Int` | R1-closed | M0 substrate validation (3 reviewer rounds; stop signal never fired) | green |
| 25 | Substrate test: can it host `dsl/std/algebra.dag` as-is? | R1-closed | Structural constraint validates candidate Declaration shapes | green |

### Substrate shape

| # | Claim | Disposition | Evidence | Status |
|---|---|---|---|---|
| 26 | Types: 6 connectives (Atom, Conj, Disj, Arrow, Cardinality, Instantiation); composition via surface sugar | R1-closed | MODELING.md §"Composition layer"; M0 validated | green |
| 27 | Computation: 5 L1 behaviors (Value, Transform, Branch, Loop, Bind); M0 validated | R1-closed | M0 three-reviewer validation; ROADMAP M0 status complete | green |
| 28 | Composition: Transform holds FunctionRef to Arrow; body is sub-DAG (user) or realization (primitives) | R1-closed | T-Emit multi-target emission proves this | green |
| 29 | Extension stop signal (C1 class): all 4 dissolution patterns must fail before extension | R1-closed | Structural constraint; no current extension pressure | green |
| 30 | Future candidate (NOT committed): unified substrate dissolving behaviors into Node patterns | post-R2-external | Noted for future consideration; revisiting requires new failure pressure | unknown |

### Free consequences

| # | Claim | Disposition | Evidence | Status |
|---|---|---|---|---|
| 31 | Automatic parallelism from dependency graph | R1-closed | T-Demo `fixture_integration_canonical` parallelism demo | green |
| 32 | Automatic memoization from purity + cost | R1-closed | Purity structural; cost via T-LaneE complexity lens | green |
| 33 | Space bound proofs from CX | R1-closed | T-LaneE complexity dimension includes space tracking | green |
| 34 | Cross-language optimization from shared cost algebra | R2-gated | T-Ground-Engine (inhabitance-search composing cost across targets) | in-flight (Engine prerequisite landed; consumer tail active) |

### Omni-emission

| # | Claim | Disposition | Evidence | Status |
|---|---|---|---|---|
| 35 | One workflow declaration projects onto every application layer | R1-closed | T-Emit `emit_omni_demo_fixtures_green` multi-target demo | green |
| 36 | Coherence structural, not checked; drift impossible (same Node tree source) | R1-closed | T-Emit multi-target structural coherence | green |
| 37 | Shape A — language targets (Rust/Python/Go/TypeScript/Swift/HDL); cost O(1) per spec | R1-closed | T-Emit Rust/Python/Go gates; TypeScript/Swift/HDL post-R1 | partial (3 of 6+ targets) |
| 38 | Shape B — user-program artifacts (YAML/Terraform/K8s/SQL/OpenAPI) via `.dag` emitter programs | post-R2-external | ROADMAP Track 16 (user code, not compiler targets); pressure-test via `../ctrl/` | not-started |
| 39 | Target-level cost complexity composes with `.dag`-level CX statically | R2-gated | T-Ground-Engine (realization costs from language specs) | in-flight |
| 40 | Distinction Shape A vs Shape B; compiler emits languages, user code emits other artifacts | post-R2-external | ROADMAP Track 16 design decision | unknown |
| 41 | Cost scaling O(1) per target, not O(N × M); effort scales with conceptual content | R1-closed | Structural property of substrate (emit pass per target, not per layer count) | green |
| 42 | Emission independent of intent (what ≠ how) | R1-closed | Separation of program declaration from realization specs | green |

### Meta-process modeling

| # | Claim | Disposition | Evidence | Status |
|---|---|---|---|---|
| 43 | Bootstrap, CI, dev process modeled as `.dag` workflows | R1-closed | `dsl/gunbc/` compiler passes authored in `.dag` | green |
| 44 | `dag run` is primary execution path | R1-closed | PB-Runtime runner (PR #792) lands `ExecuteCommand` for `.dag` workflow execution | green |
| 45 | Adding CI gate, Node field, or target language requires editing one `.dag` file | R1-closed | Structural consequence of `.dag` single-source-of-truth design | green |

### Self-hosting — three facets + cost-of-change + census

| # | Claim | Disposition | Evidence | Status |
|---|---|---|---|---|
| 46 | (Facet 1) Compiler written in language it compiles; `.dag` authors key compiler passes | R1-closed | T-PB-A lane (`dsl/gunbc/` passes); hand-Rust scaffold dissolving | green |
| 47 | (Facet 2) Compiler self-emits (fixed-point); `.dag` graph is source of truth | R1-closed | T-PB-A `pb_self_compile_fixed_point`; Pure Bootstrap primary deliverable | partial (gate ext pending T-TestGen) |
| 48 | (Facet 3) Tests are data; `.dag` `TestClaim` declarations only (0-floor target) | R1-closed | T-PB-B `pb_rust_tests_outside_residual_zero`; cascade promotion 2026-04-25 retracts TESTING residual | partial (gate ext pending T-TestGen) |
| 49 | Cost-of-change: editing compiler concept stays one `.dag` file; stage0 Rust emitted not hand-authored | R1-closed | T-PB-A lens-producer priority slice reduces hand-Rust via lens purity by construction | green |
| 50 | Census: hand-authored files per SG-0 census shrink toward 0; generated escape hatch OK | R1-closed | T-PB-A `pb_hand_rust_at_shim_floor` + `pb_compiler_std_ratchet_zero`; SG-0 census in `sg0_census_test.rs` | green |

### Audience duality

| # | Claim | Disposition | Evidence | Status |
|---|---|---|---|---|
| 51 | Core language approachable; normal engineers write `.dag` without lenses/proofs | R1-closed | T-Demo `fixture_integration_canonical` demonstrates base-language audience | green |
| 52 | Advanced surface opt-in: lenses, tests, user-defined static reflection available | R1-closed | T-LensAPI lane (`user_authored_lens_compiles`, `lens_composition_associative`) | green |
| 53 | gunbc serves both audiences; depth opt-in, base language unchanged | R1-closed | T-Demo two fixtures (compiler-nerd + integration) exercise both audiences | green |
| 54 | T-Demo fixtures + T-LensAPI operationalize opt-in depth | R1-closed | T-LensAPI gate + T-Demo gate structure | green |

### Adoption model

| # | Claim | Disposition | Evidence | Status |
|---|---|---|---|---|
| 55 | Every program gets complexity, effects, termination, idempotency, ownership by construction | R1-closed | Closed-system thesis; lenses are folds over fixed primitives | green |
| 56 | "Leaving stack" inside language = naming patterns; compiler sees through; lenses still apply | R1-closed | Namespace/composition do not hide structural facts from lenses | green |
| 57 | "Leaving stack" outside language = different compiler; thesis does not prevent it | post-R2-external | Positive corollary per `epistemic-stacking.md`; not a release gate | unknown |
| 58 | Adoption gated by economics, not enforcement; low cost × high free value | post-R2-external | Market dynamics; R2 thesis close does not gate | unknown |
| 59 | Recruiting: guarantees unavoidable via language structure, not license check | post-R2-external | Operational consequence post-R2 | unknown |

### Tests are structural data

| # | Claim | Disposition | Evidence | Status |
|---|---|---|---|---|
| 60 | All tests are `.dag` `TestClaim` declarations (0-floor cascade promotion) | R1-closed | T-PB-B `pb_rust_tests_outside_residual_zero`; TESTING.md authority | green |
| 61 | Manual tests upstream (behavioral contracts); testgen downstream (structural coverage) | R1-closed | T-TestGen `testgen_structural_coverage` gate | green |
| 62 | Rust-authored tests are language smell; flag missing predicate/effect/mock surface | R1-closed | SG-0 census predicate; hand-Rust tests → 0 under 0-floor | green |
| 63 | Pure-function posture: effects explicit parameters, mocking by DI, no flaky tests | R1-closed | Language design constraint; structural consequence | green |

### Enumerable impossible-bug classes

| # | Claim | Disposition | Evidence | Status |
|---|---|---|---|---|
| 64 | (R1) Suboptimal-complexity contract violation: function annotated bound errors at compile | R1-closed | T-Demo `impossible_bug_class_suite_r1` includes suboptimal-complexity demo | partial (CostBounded receipt is runner-fail) |
| 65 | (R1) Idempotency-contract violation: marked `@idempotent` function structure errors | R1-closed | T-Demo impossible-bugs suite (`compose_effects` + `AppendEffect` violation) | green |
| 66 | (R1) Transport/type-drift: client/server cannot hold different types for same field | R1-closed | T-Demo impossible-bugs suite (`TypeMismatch` multi-target) | green |
| 67 | (R2+) Nested-optional flatten: `Option<Option<T>>` patterns error at compile | R2-gated | `docs/r2-structure.md` Goal 4; gated on cardinality refinement; closed by PR #890 / `6562081e` | green |
| 68 | (R2+) Unenumerated effects: operations intrinsically read/write via type signature | R2-gated | `docs/r2-structure.md` Goal 4; per `docs/briefs/t-impossiblebugs-unenumerated-effects-design.md`; substrate scoping unblocked by #893/#901 | in-flight (consumer/lens PR active) |
| 69 | (R2+) Unhandled diagnostic paths: Tier 2 runtime-safety proofs | R2-gated | `docs/r2-structure.md` Goal 4; Tier 2 substrate post-R1 | in-flight (Int division Result carrier tail active) |
| 70 | Governance: classes added require thesis commitment; removal requires named dissolution | R1-closed | THESIS authority on scheduling tags `[R1]` vs `[R2+]` | green |

### Modeling discipline

| # | Claim | Disposition | Evidence | Status |
|---|---|---|---|---|
| 71 | Every declared type has at least one structural consumer | R1-closed | Linter constraint; unused-declaration detection | green |
| 72 | Every service boundary uses typed enums, not String/Bool proxies | R1-closed | Type system constraint; coercion-design.md Tier 1 | green |
| 73 | No fabrication sentinels (`__BUG_*`, `__EMIT_BUG_*`); missing facts compile-time errors | R1-closed | T-P0 `p0_no_fabrication_sentinel` | green |
| 74 | No duplicate record shapes; one type per concept | R1-closed | Ratchet constraint (dual-representation deletion tracking) | green |

## Coverage anomalies

Six notes worth tracking (NOT pre-promotion blockers):

1. **Ownership (Tier 1, claim 9)** — marked R1-closed via E-family carrier port; full ownership-dimension infrastructure may have post-R1 tail. Within T-LaneE scope is R1-committed.
2. **Shape A omni-emission (claim 37)** — R1 demonstrates 3 of 6+ Shape A targets (Rust/Python/Go). TypeScript/Swift/HDL targets explicitly listed in THESIS but not in R1 T-Emit scope ("Rust production-grade; Python/Go demonstrably working"). Full Shape A scope defers post-R1.
3. **Self-hosting fixed-point (claim 47) + tests-as-data (claim 48)** — both `[ext]` predicates pending T-TestGen runner closure; structural commitment is R1, evaluation green-status delivery-dependent on T-TestGen.
4. **Grounding completeness (claim 10)** — Tier 1 claim, R2-gated. Intentional per ROADMAP §"Post-R1 Program — Grounding Completeness" — substantial program formerly blocked on post-R1 substrate. As of the 2026-04-27 refresh, `ValueBody` list/sum + `std.unicode` landed via #920 and tokenizer charclass phase-2 closed via #1002; T-Ground still owns the Pilot → Rust → Engine → Tests → Dissolve critical path before this claim can close.
5. **R2 impossible-bug classes (claims 67-69)** — one class is now closed: nested-optional flatten via #890 / `6562081e`. Unenumerated effects and unhandled diagnostic paths remain R2-gated and in-flight rather than not-started.
6. **Release-owned follow-through outside the 74 disposition rows** — §6a `MethodContract` pick + receipt remains closed; follow-through advanced via #990 (per-field dissolution triggers + v3 lens inventory). B-wave Release closures also advanced via #968 (B5 loop construction-closure audit), #909 (B6 file-preference checklist), and #1014 (B7 `patch_lower_helpers_generated_type_alias_refinement` retirement through PB relay).

## Refresh discipline

- **When to refresh:** at every release transition (R1→R2, R2→R3 escape-hatch if invoked); on user direction; on ROADMAP authoring of new gates that re-disposition any claim.
- **Refresh process:** sweep `THESIS.md` for new/changed claims; sweep ROADMAP / `docs/r2-structure.md` for disposition changes; rebuild table; surface any new GAP rows as pre-promotion blockers.
- **Sweep boundary:** main + open PRs that author thesis claims (rare). Source briefs remain authoritative on individual claim text; this doc owns the union view.
- **Authority of this doc:** **descriptive, not prescriptive.** THESIS is authoritative on claim text; ROADMAP / `docs/r2-structure.md` are authoritative on dispositions. This doc is the union receipt + GAP-surfacing tool.

## Cross-refs

- Parent: [`docs/r2-structure.md` §"Open calls" item 1](r2-structure.md) — R2 promotion gate.
- THESIS authority: [`THESIS.md §"Thesis claims — complete list"`](../THESIS.md).
- R1 gate authority: [`ROADMAP.md §"Lane acceptance — .dag gates"`](../ROADMAP.md).
- R2 scope authority: [`docs/r2-structure.md` §"Goals"](r2-structure.md).
- Post-R2 stance: [`docs/r2-structure.md` §"Decisions locked"](r2-structure.md) — "Post-R2 stance = STRONG."
- R1 closure: [`docs/briefs/r1-closure-manager.md`](briefs/r1-closure-manager.md) (sibling doc on this PR).
- Escalation discipline: [`docs/escalation-paths.md`](escalation-paths.md) (sibling doc on this PR).
