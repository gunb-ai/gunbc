# gunbc Roadmap

## Architecture

Two substrate primitives: **Node** and **Edge**. Everything else —
types, truth values, cardinality, product/coproduct — is compositional
modeling in `.dag`. Languages are coercion targets. Testing is
compilation.

**Bounded kernel invariant:** Node is the only recursive semantic
authority in the compiler IR. All durable recursive structures are
Node trees — recursion lives in the data (children list), not in
type definitions. Non-Node types are flat discriminants and data
tables. This makes descent provable by construction: any function
that walks Node.children is structurally bounded.

Full thesis: [docs/architecture.md](docs/architecture.md)
Compiler laws and coercion model: [docs/compiler-laws.md](docs/compiler-laws.md)
Coercion design (algebra-keyed inhabitants): [docs/coercion-design.md](docs/coercion-design.md)
Testing strategy: [docs/testing-strategy.md](docs/testing-strategy.md)
Invariant enforcement: [INVARIANTS.md](INVARIANTS.md)
Modeling guidelines: [MODELING.md](MODELING.md)

## Critical Path

```
            ┌─ Lane 1: M2 (boundary sufficiency) ──┐
M1 COMPLETE─┤                                       ├→ M4 (structural identity)
Bootstrap D ┼─ Lane 2: E-track (emit + LanguageSpec)┘       └→ M5 → M6 → M7
  COMPLETE  ├─ Lane 3: CX (164 → 0)
            └─ PERF (continuous — parallel to all lanes)

Lane 1 owns: 04_infer, 04_types, 02_parse, 04_method
Lane 2 owns: 05_emit, 05_emit_rust, 04_emit_info, languages
Lane 3 owns: complexity, dsl/std/
PERF owns: performance ratchets, bootstrap convergence tests, timing budgets
M4 follows Lanes 1+2 (needs structural facts + clean render path)
```

---

# Layer 1: Active Gates

## Bootstrap Status

| Stage | Status | Gate | Notes |
|-------|--------|------|-------|
| **A** | GREEN | 0 diagnostics | PR #264 |
| **B** | GREEN | 0 emitted-Rust errors | PR #307. `bootstrap_stage0_to_stage1` still `#[ignore]` |
| **C** | GREEN | regen binary self-compiles | PR #308. 1 perf-only bootstrap patch: `dag_syntax_spec` cache |
| **D** | GREEN | `regenerate-stage0.sh && git diff --exit-code` | PR #308. Freshness gate blocking in CI. Root cause was .dag source using legacy `emit_node_type_rc` while stage0 used `render_rust_type` |

**Bootstrap D** = regenerated code replaces committed stage0 with zero
manual patches, AND the regenerated binary produces identical output
when it self-compiles (fixed point convergence). **Blocks all other
lanes from editing stage0 Rust directly.**

Note: "zero manual patches" means zero patches to *generated* files.
Five hand-maintained files are still copied during regeneration
(main.rs, v2_rt.rs, compiler_tests.rs, extdeps_languages_dag_syntax.rs,
v2_coercion.rs). Eliminating these is tracked as future work under
M5-full (language plugin extraction).

## CI Gates

| Gate | Command | Status |
|------|---------|--------|
| Lint | `cargo clippy --workspace -- -D warnings` | GREEN |
| Tests | `cargo test -p v2-compiler-tests` | GREEN (271 pass, 0 fail, 36 ignored) |
| Full DSL | `full_dsl_compiles -- --ignored` | GREEN (93 dsl + 29 v2) |
| Diagnostic ratchet | `strict_compile_diagnostic_count -- --ignored` | 314 (all complexity violations) |
| L1 ratchet | `scripts/l1-ratchet.sh --check` | 30 (target: 0) |
| Stage0 freshness | `scripts/check-stage0-freshness.sh` | GREEN (blocking) |

## Ratchet Counts

| Metric | Current | Target | Notes |
|--------|---------|--------|-------|
| Self-compile diagnostics | 314 | 0 | All indirect-recursion complexity violations |
| L1 type knowledge | 30 | 0 | Down from 70; name-based workarounds tracked for M4 |
| Complexity violations | 164 | 0 | Down from 315; unfinished algebraic grounding |
| Emitted Rust errors | 0 | 0 | GREEN |
| DSL complexity ratchet | 2 | 0 | stack_size + fold_stack (deferred to CX lane) |

## Bootstrap Health Rules

Clean-repo workflow:
1. `cargo check -p v2-compiler`
2. `cargo test -p v2-compiler-tests full_dsl_compiles -- --ignored --nocapture`
3. `cargo test -p v2-compiler-tests strict_compile_diagnostic_count -- --ignored --nocapture`
4. Run `./scripts/regenerate-stage0.sh`
5. Require `git diff --exit-code src/v2/stage0`

Stabilization rules:
- No manual `src/v2/stage0/` edits once regeneration is green.
- CI gate: `check-stage0-freshness.sh` (regenerate → diff → empty).
- CX gate disabled in both stage0 and `compile.dag` — emission not blocked by complexity violations. Re-enable when CX violations reach 0.

## Reviewer Root Cause Analysis (2026-04-03)

Two stacked failures, not one. First, some target-language facts are
missing (TLC-1 through TLC-4). Second, facts that exist upstream are
lost before emit, so emission compensates with heuristics. The deepest
bug is upstream information loss, not the leaf workaround. Every
remaining emit workaround should be treated as either an
inference-boundary bug or a missing LanguageSpec/coercion fact — never
a standalone emitter patch.

---

# Layer 2: Root-Cause Tracks

Four named architectural problems. Each has one root cause and one
definition of done. Lanes 1–3 run in parallel; M4 follows.

## M2: Boundary Sufficiency (Lane 1)

**Root cause:** The resolution→emit boundary does not carry enough
structure. Emit compensates with heuristics. Every remaining workaround
is an inference-boundary bug or a missing upstream fact.

### Structural error propagation

`child_inferred_or_empty` fabricates Unit on inference failure instead
of propagating error state. `node_inferred_to_outputs` builds outputs
from fabricated types. Highest-confidence correctness bug (reviewer
2026-04-02).

- [x] `child_inferred_or_empty` propagates error state structurally
- [x] `node_inferred_to_outputs` refuses error-typed children (fail-closed) — all-or-nothing gate via `rt_node` check; returns `[]` if any child is not `Typed`

### Incomplete parameterization and bidirectional inference

One root cause behind multiple symptoms: type information does not flow
bidirectionally through inference boundaries, so downstream stages see
incomplete types and fabricate.

Symptoms:
- `bare_map_node()`/`bare_list_node()` fabricate partial structure
- Fold accumulators under-resolved (magic names Unit/Dynamic/Error)
- Higher-order method templates collapse callable structure into ReceiverSelf
- `expected` parameter threaded to `infer_expr` but incomplete for formal params
- Callback shapes (`fn(Acc, T) -> Acc`) synthesized at inference time, not declared

Open items:
- [ ] Incomplete parameterized types rejected at normalization, not infer
- [ ] `bare_map_node`/`bare_list_node` eliminated or gated before emit
- [ ] Thread `expected` to all formal params, not just callable ones — currently only threaded to fold init + literal special cases, not general parameter inference; also overloads the `expected` boundary (non-callable expected can silently type lambda args)
- [x] Refine fold accumulators structurally via `is_fully_resolved` — now checks TypeVariable on self + children, plus collection arity; non-collection concrete types correctly resolve
- [ ] `CallableOf` in `AlgebraTypeTemplate` for higher-order signatures

### Explicit ownership and identity

- [ ] Explicit parent-enum ownership facts through resolve/infer/emit
- [ ] Transport/config: one `.dag` authority for transport schema (35+ redundant sites)

### Acceptance

No fabricated type args, no generic/wrong fallback return types, no
suffix/name scans to recover ownership. Fallback count promoted to CI.

---

## E-track: Emit Boundary + LanguageSpec (Lane 2)

**Root cause:** Emission rediscovers facts available upstream.
TypeRendering is the boundary fix. LanguageSpec/coercion is the
authority fix. TLC-1 through TLC-4 are the concrete expression-level
gaps.

### TypeRendering boundary (E0c)

`build_type_rendering` + `render_type` replaces scattered
`emit_node_type_rc()`. 2757 nodes validated with 0 mismatches.
Dual authority remains (`emit_node_type_rc` still live).

- [ ] Delete `emit_node_type_rc` / old type rendering path
- [ ] `build_rc_types` eliminated — sharing authority in TypeRendering
- [ ] `emit_primitive_type` fail-closed (no pass-through on miss)
- [ ] TypeRendering dissolves into coercion engine (M5)

### Expression-level emit semantics (TLC-1..4)

These are the concrete gaps blocking new backends. Each should be a
LanguageSpec authority, not emitter special cases.

- [ ] **TLC-1: Call syntax / reference distinction.** Zero-arg fn calls
  must render as `name()`, not bare `name`. The callable-vs-value
  distinction must survive from resolution through emit.
- [ ] **TLC-2: Runtime bridge signature derivation.** Runtime helper
  return types and wrapping conventions must derive from the same
  type/coercion authority as emission. `v2_rt::map_keys` returns
  `Vec<K>` but emission expects `Rc<Vec<K>>`.
- [ ] **TLC-3: Indexing / character access semantics.** List indexing
  must be declared as a target-language fact, not fabricated in the
  emitter.
- [ ] **TLC-4: Explicit annotation requirements.** Target languages
  with incomplete inference (Rust turbofish) should model annotation
  needs as LanguageSpec properties.

### M5-early: coercion via TypeRendering

`build_type_rendering` reads coercion data from `.dag` declarations:
- `TypeCheckpoint` (primitives) from language `types.dag`
- `InhabitantDecl` (algebra containers) from language `types.dag`
- `CallableRepr` (callable syntax)

Shared schema in `std/coercion.dag`; per-language instances in
`extdeps/languages/{rust,python,go}/types.dag`. Design doc:
[docs/coercion-design.md](docs/coercion-design.md).

### ValueContext (E0b)

The graph doesn't carry HOW a value is used, only WHAT it is.
`EmitGraphInfo` carries `value_contexts: Map<String, ValueContext>`
with four kinds: ConstantData (immutable lookup table), RuntimeValue
(heap-allocated, needs per-language wrapper), SpecificationWitness
(structural fact, not runtime data), CallableValue (function type).
Per-language emission reads ValueContext × LanguageSpec. Partially
landed (`has_fn_fields` precomputed); `rc_types` authority should
derive from ValueContext (`is_constant` → no wrap).

- [ ] `rc_types` authority derived from ValueContext
- [ ] `is_constant` computation with consumer

### LanguageSpec completion

LanguageSpec is designed but underutilized — backends hardcode values
the spec provides. Full parameterization of expression/statement
emission (P1-B) depends on LanguageSpec carrying all target-language
facts.

### Acceptance

Zero escape hatches in type rendering. No fabricated names. Adding a
backend requires only data, no emission logic. No partial metadata
passes — every new fact layer must have a consumer in the same change.

---

## M4: Structural Identity (L1 = 0) — follows Lanes 1+2

**Root cause:** The compiler uses `Node.name` (a string) as semantic
authority. ~256 constructions, ~30 name-based comparisons. Deletion
requires declaration-driven identity and structural algebra.
Blocked on M2 (structural facts in resolve/infer files) and E-track
(clean render path in emit files).

### Lane 1: Declaration-driven algebra

Goal: compiler reads type/algebra facts from `.dag` declarations
instead of hardcoding them.

- Tier 1 (data tables → .dag): DONE
- Tier 2 (factor enrich_kernel_type): DONE
- Tier 2.5 (algebra bridge fidelity):
  - [ ] `CallableOf` variant for higher-order callback shapes
  - [ ] Derive T/K/V type parameter names from algebra declarations
- Tier 3 (full structural algebra, requires FF-9):
  - [ ] Compiler reads type declarations + algebra edges at resolve time
  - [ ] Derive kernel/container identity from type declarations
  - [ ] CollectionKind bridge dissolves when method algebras land
  - [ ] 21 type constructor sites → 0

### Lane 2: Node.name deletion (D6)

Goal: delete `Node.name` field. Rendering uses `source_text_at`,
resolve uses structural identity.

- B3 (emit rendering): DONE
- B4 (resolve structural identity): accessor layer done, `node.name`
  still semantic authority underneath
- D6 open:
  - [ ] Migrate remaining emit sites (Python ~5, Go ~5, shared ~5)
  - [ ] Update ~256 Node constructions to drop `name:`
  - [ ] Migrate synthetic node identity to structural
  - [ ] Delete `Node.name` field + scrambled-name tests

Lanes share only `00_core.dag` (different functions, no conflict).

### Acceptance

`scripts/l1-ratchet.sh --check` reports 0. Scrambled-name tests pass
then deleted. `Node.name` field deleted.

---

## CX: Complexity Analyzer (164 → 0) (Lane 3)

**Root cause:** 164 violations are ungrounded algebraic concepts, not
analyzer bugs. The path to 0 is grounding each concept in std/, not
extending the analyzer.

DFA triage maps all 164 to four algebraic root causes:

| Root cause | Count | Fix |
|-----------|-------|-----|
| Parser SCCs | ~80 | DescentEvidence lattice (std/termination.dag) |
| Fold/catamorphism | ~40 | Descend primitive (std/iteration.dag) |
| CostExpr/SizeExpr | ~30 | Tropical semiring → Node composition |
| Accessor-on-var | ~14 | Signature-driven fold (std/algebra.dag) |

### Work items

- **CX-A**: DescentEvidence lattice unification — parser mutual recursion
  gets structural termination proofs. Files: `complexity.dag`,
  `dsl/std/termination.dag`. Expected: 164 → ~150.
- **CX-B**: CostExpr/SizeExpr dissolution — cost expressions become Node
  compositions. All 30+ match sites in `complexity.dag` rewrite to Node
  walkers. Expected: ~150 → ~120.
- **CX-C**: Signature-driven fold evidence — self-calls inside
  `children |> fold` callbacks get structural descent proofs.
  Expected: ~120 → ~80.
- **CX-D**: MatchPattern dissolution + remaining concept grounding.
  Expected: ~80 → 0.
- **CX-E**: Re-enable complexity gate — remove `complexity_diags = []`,
  un-ignore 14 complexity tests (10 `complexity_*`, 3 `soundness_*`,
  1 `structural_classify_*`; all `#[ignore]` with "CX track" comment).

### Acceptance

0 violations without suppression. CX gate re-enabled. Node is the only
recursive type consumed by complexity analysis.

---

## PERF: Compiler & Test Performance (parallel track)

**Goal:** Continuous visibility into compiler and test performance so
regressions are caught before they compound. Runs in parallel with all
lanes — any lane can introduce a regression.

### Existing infrastructure

| What | Where | Status |
|------|-------|--------|
| Self-compile time ratchet | `bootstrap::performance_ratchet` | `#[ignore]`, 30s budget (~6.5s actual) |
| Bootstrap stage0→stage1 | `bootstrap::bootstrap_stage0_to_stage1` | `#[ignore]` |
| Full DSL compile | `pipeline::full_dsl_compiles` | `#[ignore]`, GREEN |
| Stage0 freshness gate | `scripts/check-stage0-freshness.sh` | CI blocking |
| Diagnostic ratchet | `strict_compile_diagnostic_count` | `#[ignore]`, 314 |

### Work items

- **PERF-1**: Un-ignore `performance_ratchet` in CI. Currently 30s
  budget with ~6.5s actual. Gate on this to catch O(n^2) regressions
  early. Requires CI runner has `cargo build --release` capacity.
- **PERF-2**: Un-ignore `bootstrap_stage0_to_stage1` in CI. This is
  the full regen + convergence test. Proves pass-1 = pass-2 on every PR.
- **PERF-3**: Track self-compile memory. The CX OOM root cause was
  repeated complexity classification on large compiles, not raw budget.
  Add a memory-usage ratchet or at minimum log peak RSS during
  `performance_ratchet`.
- **PERF-4**: Test suite wall-clock ratchet. Current: ~270s for 271
  tests. Budget TBD. Individual tests >2s are suspect (per project
  convention). Add per-test timing visibility.
- **PERF-5**: Operation-count contracts. Test performance via structural
  operation counts (node visits, inference passes, emit calls) rather
  than wall-clock time. More deterministic, catches algorithmic
  regressions independent of machine load.

### Acceptance

`performance_ratchet` and `bootstrap_stage0_to_stage1` running in CI.
No test >2s without justification. Self-compile time tracked per-PR.

---

# Layer 3: Deferred

Theory, long-horizon, and items that land after root-cause tracks complete.

## P1: Modeling Consolidation

Programs are applied mathematics with informal names. ~90 redundant
types are standard algebraic structures reimplemented ad-hoc.

| Item | What | Impact | Blocked on |
|------|------|--------|-----------|
| P1-A | Result monad unification (58 parser/resolve types → 2 generic) | ~58 types deleted, ~200 sites simplified | Generic type support |
| P1-B | Emit parameterization (3 backends → 1 parameterized homomorphism) | ~2,500 lines eliminated | M5-early for types |
| P1-C | String→structural identity (~200 Map<String,X> → edges) | IS M4 Lane 2 | M4 |
| P1-D | Context/accumulator dedup (10 types → 4) | ~6 types eliminated | Independent |
| P1-E | Non-Node recursive type dissolution | ~45 variants, ~56 CX violations | CX lane |

Expected total: ~100 types eliminated, ~2,500 emit lines eliminated,
stage0 mirror ~17K → ~13K lines.

## Model Convergence

Post-bootstrap. Remaining recursive types dissolve into Node:

| Type | Dissolution | Milestone |
|------|-------------|-----------|
| CostExpr/SizeExpr | Node composition in std/cost.dag | CX-B |
| TypeRendering | Coercion engine | M5 |
| MatchPattern | Node discriminant metadata | M7 |
| InferredNode | Keep wrapper, non-recursive reference | M7 |

Non-recursive authority leaks (same class of unfinished migration):

| Leak | Fix |
|------|-----|
| `Node.name` as authority (~256 sites) | M4 Lane 2 |
| Transport/config duplication (35+ sites) | One .dag authority |
| Bare parameterized types | Reject at normalization |
| Missing `CallableOf` | M4 Tier 2.5 |
| Semantic strings (parent_enum, service_name) | Structural Node references |

## Pipeline Algebraic Grounding

Every compiler type is a mathematical concept in disguise. Grounding
in std/ eliminates ad-hoc implementations and makes new capabilities
fall out of existing infrastructure.

Key redundancies:

| What | Count | Actually is | Collapses to |
|------|-------|-------------|-------------|
| Parse result types | 68 | State monad `ParseM<T>` | 1 generic type |
| ProgressKind ≅ DescentEvidence | 2 | Same BoundedLattice | 1 from std/termination.dag |
| CostExpr + SizeExpr | 12 variants | Tropical semiring | Node trees |
| AlgebraTypeTemplate | 9 variants | Type constructor free algebra | Node trees |
| TypeRendering | 7 recursive fields | Coercion checkpoint | std/coercion.dag |

Missing std/ concepts: `std/discrimination.dag` (pattern matching),
`std/graph.dag` (directed graph algebra), Signature + Term in
`std/algebra.dag`, Cardinality lattice in `std/types.dag`.

## Deferred Milestones

- **M1**: COMPLETE (90 dsl + 29 v2 files, 0 diagnostics)
- **M3**: Test generation and guarantee receipt. Receipt schema as .dag
  type, CI validates receipt against ratchets, cross-language test
  taxonomy. Depends on M2.
- **M5-full**: Language plugin extraction. Delete 3 language-specific
  emit files (~6,857 lines). Adding a language = adding data.
  Depends on M4 for identity dissolution.
- **M6**: Parse-emit symmetry. Parse and emit are structural inverses.
  The parser reads SyntaxSpec; the emitter should read the same spec
  in reverse.
- **M7**: Dissolve structural bridges. `connective`, `return_cardinality`,
  `MatchPattern`, `InferredNode` wrapper — all become graph structure
  or metadata on Node.

## Exploratory Directions

- **Bounded iteration**: three primitives — fold (structural descent
  over children), descend (recursive with proof witness), repeat
  (bounded by fuel). All loops are sugar over these. `.dag` has no
  unbounded `while`.
- **Cost comparator**: refuse to compile suboptimal code when a
  cheaper equivalent exists. Requires cost algebra completion.
- **Cost algebra extensions**: Pow, Sqrt, Exp; amortized analysis via
  potential functions; space as peer dimension. See
  `docs/cost-algebra.md`.
- **Post-Rust path**: .dag → native code (LLVM/Cranelift) directly,
  no Rust intermediate. Optional — Rust/Go as intermediates may
  suffice indefinitely.
