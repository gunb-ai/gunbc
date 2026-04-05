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
Bootstrap D ┼─ Lane 2: CG (codegen correctness) ───┘       └→ M5 → M6 → M7
  COMPLETE  ├─ Lane 3: CX (164 → 0)
            └─ PERF (continuous — parallel to all lanes)

Lane 1 owns: 04_infer, 04_types, 02_parse, 04_method
Lane 2 owns: 05_emit, 05_emit_rust, 04_emit_info, ownership, languages
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
| **D** | GREEN | `regenerate-stage0.sh && git diff --exit-code` | PR #308. Freshness gate blocking in CI. `emit_node_type_rc` deleted; single authority via `build_type_rendering` + `render_type` |

**Bootstrap D** = regenerated code replaces committed stage0 with zero
manual patches, AND the regenerated binary produces identical output
when it self-compiles (fixed point convergence). **Blocks all other
lanes from editing stage0 Rust directly.**

Note: "zero manual patches" means zero patches to *generated* files.
Nine hand-maintained files are still copied during regeneration
(main.rs, v2_rt.rs, compiler_tests.rs, extdeps_languages_dag_syntax.rs,
v2_coercion.rs, v2_compiler_infer_method.rs, std_types.rs,
v2_compiler_tokenize.rs, extdeps_languages_rust_emit.rs).
The last five are thread-local cache shims for performance;
eliminating these is tracked as future work under M5-full
(language plugin extraction).

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
remaining emit workaround is either an inference-boundary bug (M2) or
a missing codegen authority (CG) — never a standalone emitter patch.

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

### Acceptance

No fabricated type args, no generic/wrong fallback return types, no
error-typed children reaching emit. Fallback count promoted to CI.
Ownership and clone correctness tracked under CG lane.

---

## CG: Codegen Correctness + Optimality (Lane 2)

**Root cause:** Codegen decisions (type rendering, sharing, ownership,
clone, expression semantics) are scattered across emitter heuristics
instead of derived from structural authorities. This produces
correct-but-suboptimal code and blocks new backends.

**Status:** TypeRendering boundary established (emit_node_type_rc deleted).
TLC-3 and TLC-4 data models landed in LanguageSpec; Rust emitter consumes
both. Go/Python still use list_index/list_slice unconditionally (TLC-3 not
fully dispatched). Annotated-let path introduced but has no consumer yet
(TLC-4 not end-to-end). TLC-1 and TLC-2 blocked on upstream pipeline work.

One root cause, two symptoms:
1. Facts that exist upstream are lost before emit — emission compensates
   with heuristics (411 `rc_types` threading sites, 28 hardcoded
   `.clone()` decisions in the Rust emitter alone)
2. Target-language facts are missing entirely — expression-level gaps
   (TLC-1..4) force per-backend special cases

### Completed (FF-1: ownership analysis)

- [x] Ownership analysis wired into Rust emitter: fan-out=1 function params move instead of clone
- [x] VarBindingKind flows through ownership analysis to emission
  - Let-binding moves: BindingUsage carries binding_kind, is_owned_local filter replaces params set
  - Match-bound variable handling: MatchBoundBinding variant excludes &T refs from movable set

### CG-1: Authority consolidation

Single source of truth for each codegen decision. Kill dual authorities
and heuristic fallbacks.

**Type rendering**

`build_type_rendering` + `render_type` is the sole type rendering
authority. `emit_node_type_rc` deleted (2757 nodes validated at 0
mismatches before removal). `emit_node_type` routes through
`build_type_rendering` + `render_type`.

- [x] Delete `emit_node_type_rc` / old type rendering path
- [ ] `emit_primitive_type` fail-closed (no pass-through on miss)

**Sharing and ownership**

- [ ] `rc_types` derived from ValueContext (`is_constant` → no wrap)
- [ ] `build_rc_types` eliminated — sharing authority in TypeRendering
- [ ] `is_constant` computation with consumer
- [ ] Clone semantics in LanguageSpec (28 hardcoded `.clone()` → data-driven)
- [ ] Explicit parent-enum ownership facts through resolve/infer/emit

**Value context**

`EmitGraphInfo` carries `value_contexts: Map<String, ValueContext>`
with four kinds: ConstantData (immutable lookup table), RuntimeValue
(heap-allocated, needs per-language wrapper), SpecificationWitness
(structural fact, not runtime data), CallableValue (function type).
Per-language emission reads ValueContext × LanguageSpec. Partially
landed (`has_fn_fields` precomputed).

- [ ] ValueContext fully consumed by all emission sites

### CG-2: Expression-level gap closure (TLC-1..4)

Each gap is a missing LanguageSpec fact. Closing them unblocks new
backends.

- [ ] **TLC-1: Call syntax / reference distinction.** Zero-arg fn calls
  must render as `name()`, not bare `name`. The callable-vs-value
  distinction must survive from resolution through emit.
  *Blocked: requires callable identity to flow through pipeline (M2 Lane 1).*
- [ ] **TLC-2: Runtime bridge signature derivation.** Runtime helper
  return types and wrapping conventions must derive from the same
  type/coercion authority as emission. `v2_rt::map_keys` returns
  `Vec<K>` but emission expects `Rc<Vec<K>>`.
  *Blocked: requires M5 coercion engine for type/wrap authority.*
- [ ] **TLC-3: Indexing / character access semantics.** `IndexingSemantics`
  in LanguageSpec — per-collection-type templates (list/map/string index/slice).
  Rust dispatches on collection type; Go/Python still route through `list_index`
  unconditionally. *Remaining: Go/Python per-collection dispatch.*
- [ ] **TLC-4: Explicit annotation requirements.** `AnnotationRequirements`
  in LanguageSpec — let binding templates (inferred/annotated), lambda param
  templates (typed/untyped). Inferred-let and lambda params consume spec;
  `emit_let_binding_annotated` introduced but no caller yet.
  *Remaining: annotated-let consumer must land in same change.*

### CG-3: Parameterization

Make emission fully data-driven. Adding a backend = adding data.

- [x] Method templates: simple methods use `apply_named_template` with per-language
  `Map<String, String>` data. Templates are pure method syntax; Rc wrapping
  composed separately from sharing authority. Covers count/join/split/first/last/
  enumerate/chars/skip/take + higher-order (map/filter/fold/sort_by/any/all/flat_map).
- [ ] Transport/config: one `.dag` authority (35+ redundant sites → 1)
- [ ] LanguageSpec completion — all target-language facts data-driven
  *(method_templates landed as `Map<String, String>?`; structured `MethodTemplate`
  type with lambda/fn_ref/simple variants is the next step)*
- [ ] TypeRendering dissolves into coercion engine
- [ ] 3 backends → 1 parameterized homomorphism (~2,500 lines eliminated)

### Coercion infrastructure (reference)

`build_type_rendering` reads coercion data from `.dag` declarations:
- `TypeCheckpoint` (primitives) from language `types.dag`
- `InhabitantDecl` (algebra containers) from language `types.dag`
- `CallableRepr` (callable syntax)

Shared schema in `std/coercion.dag`; per-language instances in
`extdeps/languages/{rust,python,go}/types.dag`. Design doc:
[docs/coercion-design.md](docs/coercion-design.md).

### Acceptance

For each target language, the compiler must prove:

1. **Correctness**: emitted code compiles and passes the full test suite
   with zero manual patches or escape hatches.
2. **Optimality**: no unnecessary clone, copy, or allocation — every
   sharing decision traces to a structural fact (TypeRendering,
   ValueContext, or LanguageSpec) and can be audited. Emitted code
   should be what a competent human would write by hand.
3. **Completeness**: every codegen decision derives from exactly one
   structural authority — no heuristic fallbacks, no dual authorities,
   no hardcoded target-language knowledge in the emitter.
4. **Backend independence**: adding a new target language requires only
   LanguageSpec + coercion data files, zero emission logic changes.

### Proof mechanism: structural coverage by construction

The .dag input language is decidable and Node-bounded, so the space
of structural forms reaching the emitter is finite. Correctness is
proved by construction over this finite algebra, not by post-hoc
validation of specific programs.

**Emission algebra.** The emitter's input space is the product of
`(NodeKind × TypeForm × Cardinality)` triples that survive type
checking. This set is enumerable from the .dag type definitions
themselves — it is the compiler's own structural vocabulary.

**Structural induction on Node depth.** Node is the only recursive
type, so the proof has two parts:
- *Base case*: every leaf form (literal, variable ref, constant) emits
  valid target code for all type forms.
- *Inductive step*: if children emit valid code, the parent node's
  assembly emits valid code. This holds when every assembly decision
  (clone, wrap, annotate) reads from a structural authority rather
  than a heuristic — which is exactly what CG-1 through CG-3 enforce.

**Exhaustive form coverage.** A test generator synthesizes one minimal
`.dag` program per element of the emission algebra and compiles it to
every target. This extends `full_dsl_compiles` from "these programs
compile" to "every structural form compiles." The generator is
derivable from the .dag type definitions, so new forms added to the
language automatically produce new test cases.

**LanguageSpec modularity.** Once every emitter decision reads from
LanguageSpec (CG-3), the proof becomes modular: prove LanguageSpec
covers every structural form → proves any backend using it is
complete. Adding a language = adding a LanguageSpec instance + proving
coverage of the same finite algebra.

No escape hatches in type rendering. No fabricated names. No post-hoc
passes to fix what construction should have prevented.

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
| P1-B | Emit parameterization (3 backends → 1 parameterized homomorphism) | ~2,500 lines eliminated | CG-3 |
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
