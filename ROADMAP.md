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
            CM: cross-cutting design lens (informs all lanes) → src/v2/CM.md

Lane 1 owns: 04_infer, 04_types, 02_parse, 04_method
Lane 2 owns: 05_emit, 05_emit_rust, 04_emit_info, ownership, languages
Lane 3 owns: complexity, dsl/std/
CM informs how Lanes 1-3 approach work; does not own files separately
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
Two hand-maintained files are still copied during regeneration
(main.rs, compiler_tests.rs).
Goal is zero — all stage0 content should be 100% generated.
Remaining files and what blocks elimination:

| File | Role | Blocker |
|------|------|---------|
| `main.rs` | CLI entrypoint (clap, import resolution, diagnostic rendering) | Need compiler entrypoint generation — entrypoints are deducible interfaces (like HTTP servers); content is fixed, can be templated like v2_rt.rs |
| `compiler_tests.rs` | Test harness | Need .dag test generation |

Previously eliminated:
- PR #316: v2_compiler_infer_method.rs, std_types.rs,
  extdeps_languages_rust_emit.rs, v2_compiler_tokenize.rs,
  extdeps_languages_dag_syntax.rs — via thread_local! caching in emitter
  and `chars_to_string` runtime function for O(1) tokenizer substring.
- v2_coercion.rs — replaced by `src/v2/coercion.dag`, coercion registries
  now generated from .dag type checkpoint/inhabitant data declarations.
- v2_rt.rs — runtime templates already modeled in `runtime_rust.dag`;
  `rust_runtime_source()` produces the complete file. Emitter includes it
  via `emit_v2_rt_module()`.

## CI Gates

| Gate | Command | Status |
|------|---------|--------|
| Lint | `cargo clippy --workspace -- -D warnings` | GREEN |
| Tests | `cargo test -p v2-compiler-tests` | GREEN (283 pass, 0 fail, 39 ignored) |
| Full DSL | `full_dsl_compiles -- --ignored` | GREEN (93 dsl + 29 v2) |
| Diagnostic ratchet | `strict_compile_diagnostic_count -- --ignored` | 314 (all complexity violations) |
| L1 ratchet | `scripts/l1-ratchet.sh --check` | 33 (target: 0) |
| Stage0 freshness | `scripts/check-stage0-freshness.sh` | GREEN (blocking) |

## Ratchet Counts

| Metric | Current | Target | Notes |
|--------|---------|--------|-------|
| Self-compile diagnostics | 314 | 0 | All indirect-recursion complexity violations |
| L1 type knowledge | 33 | 0 | Down from 70; name-based workarounds tracked for M4 |
| Complexity violations | 164 | 0 | Down from 315; unfinished algebraic grounding |
| Emitted Rust errors | 0 | 0 | GREEN |
| DSL complexity ratchet | 2 | 0 | stack_size + fold_stack (deferred to CX lane) |

## Bootstrap Dependency Chain

```
.dag source ──(v2-compiler)──▶ stage0 .rs ──(cargo/rustc)──▶ v2-compiler binary
     ▲                                                              │
     └──────────────────────────────────────────────────────────────┘
```

- **Source of truth:** `.dag` files. Stage0 `.rs` is a derived artifact.
- **Cycle-breaker:** Committed stage0 allows fresh clones to bootstrap.
- **Fixed-point:** `regen pass N == regen pass N+1`. Two-pass bootstrap
  required when the emitter changes its own output.
- **External dependencies:** cargo, rustc (opaque transforms, not modeled).
- **Hand-maintained files (9):** Copied back during regen, not overwritten.
  These are source, not derived. See Bootstrap Status for list.

**Merge workflow problem:** Stage0 `.rs` files are derived, but git
treats them as text and produces line-level merge conflicts. These
conflicts carry zero information — the only correct resolution is
regen. Fix: `.gitattributes` marks generated stage0 files as `-merge`
(no line-level merge), CI freshness gate ensures committed stage0
matches `.dag` source. Merge workflow becomes: resolve `.dag` conflicts
→ accept either side of stage0 → regen → commit.

This dependency chain is currently implicit (shell scripts + convention).
Modeling it as `.dag` types is tracked under BP-1 (Layer 3).

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
- Stage0 generated `.rs` files use `-merge` in `.gitattributes` (no line-level merge).
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

- [~] `child_inferred_or_empty` no longer fabricates Unit — `Untyped` returns the child's own structure (partial: reuses raw structure as type authority, not explicit error propagation)
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
- [x] Incomplete parameterized types rejected at normalization — `container_expected_arity` returns `Int?` (fail-closed: unknown names → None, no false positives on operations sharing container names)
- [ ] `bare_map_node`/`bare_list_node` eliminated or gated before emit — normalization catches authored bare containers; `empty_map()` with non-keyed expected now diagnosed; fold-init path (`None` expected) still falls back to `bare_map_node()` pending expected-threading through fold accumulators. Empty list `[]` diagnostic blocked on expected-type propagation through list element inference (160 false positives from `[]` in record literal fields like `param_types: []`)
- [x] Thread `expected` to formal params at matching positions — over-arity args no longer receive synthetic expected types; non-callable expected boundary overload remains open
- [x] Refine fold accumulators structurally via `is_fully_resolved` — recursive: checks TypeVariable on self, collection arity, and recurses into all children
- [x] `CallableOf` in `AlgebraTypeTemplate` for higher-order signatures

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
Clone semantics unified in SharingStrategy (CloneTemplates removed).
TLC-1 partial: `is_zero_arg_callable_ref` is an emitter-side guardrail
(L1 violation — `rt.name=="Callable"` check); upstream concept modeling
needed (Tier 2.6). TLC-3 Rust per-collection dispatch done; Go/Python
still route through `list_index` unconditionally. TLC-4 partial:
`emit_let_binding_annotated` infrastructure exists but disabled for
bootstrap convergence (.dag type inference produces `()` for empty
collection element types). TLC-2 blocked on M5 coercion engine.
Higher-order method templates (filter/any/all/flat_map) data-driven
via `HigherOrderMethodSpec`.

One root cause, two symptoms:
1. Facts that exist upstream are lost before emit — emission compensates
   with heuristics (411 `shared_types` threading sites, 28 hardcoded
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
- [x] `emit_primitive_type` fail-closed (no pass-through on miss)

**Sharing and ownership**

- [x] Sharing derived from `is_type_constant` + `TypeSummary` (no Rc wrap for fixed-width carriers)
- [x] `build_rc_types` renamed and reorganized as `build_shared_types` in Rust emitter
- [x] `ValueContext` deleted — `has_fn_fields` moved to `TypeSummary`, sharing
  computation lives in `EmitGraphInfo.shared_types` field (still Rust-emitter-local;
  authority has not moved upstream — deferred to coercion engine)
- [x] `is_type_constant` in 05_emit_rust.dag consulted by `build_shared_types`
- [x] Clone semantics in LanguageSpec (28 hardcoded `.clone()` → data-driven)
- [x] Explicit parent-enum ownership facts through resolve/infer/emit
- [x] Phase B cleanup: rename `rc_types` parameter → `shared_types` across ~423
  occurrences in emit pipeline (mechanical, no semantic change)

### CG-2: Expression-level gap closure (TLC-1..4)

Each gap is a missing LanguageSpec fact. Closing them unblocks new
backends.

- [~] **TLC-1: Call syntax / reference distinction.** Zero-arg fn calls
  render as `name()` via `is_zero_arg_callable_ref` (FunctionValueBinding +
  Callable node with empty params). Dispatched in Rust emit_var_ref and
  emit_typed_expr_base only. Go/Python keyword mapping (none/true/false)
  landed (PR #324); zero-arg callable detection kept Rust-scoped per
  review — widening an L1 violation across backends without upstream
  modeling violates boundary sufficiency. Upstream concept modeling
  (Tier 2.6) dissolves this. Imported zero-arg function refs untested.
- [ ] **TLC-2: Runtime bridge signature derivation.** Runtime helper
  return types and wrapping conventions must derive from the same
  type/coercion authority as emission. `v2_rt::map_keys` returns
  `Vec<K>` but emission expects `Rc<Vec<K>>`.
  *Blocked: requires M5 coercion engine for type/wrap authority.*
- [~] **TLC-3: Indexing / character access semantics.** `IndexingSemantics`
  in LanguageSpec — per-collection-type templates (list/map/string index/slice).
  Rust dispatches on collection type. Go/Python still route through
  `list_index`/`list_slice` unconditionally — per-collection dispatch pending.
- [~] **TLC-4: Explicit annotation requirements.** `AnnotationRequirements`
  in LanguageSpec — let binding templates (inferred/annotated), lambda param
  templates (typed/untyped). Infrastructure exists (`emit_let_binding_annotated`)
  but disabled at all three Rust let-binding sites for bootstrap convergence:
  .dag type inference produces `()` for empty collection element types.
  Requires M2 inference improvement before re-enabling.

### CG-3: Parameterization

Make emission fully data-driven. Adding a backend = adding data.

- [x] Method templates: simple methods use `apply_named_template` with per-language
  `Map<String, String>` data. Templates are pure method syntax; Rc wrapping
  composed separately from sharing authority. Covers count/join/split/first/last/
  enumerate/chars/skip/take + higher-order (map/filter/fold/sort_by/any/all/flat_map).
- [x] Higher-order method templates: `HigherOrderMethodSpec` type in Rust extdeps
  with `method_name`, `inline_template`, `fn_ref_template`, `wraps_in_sharing`.
  Shared `emit_rust_higher_order_method` replaces 4 hardcoded emitters
  (filter/any/all/flat_map). Data-driven dispatch via method name lookup.
- [~] Transport/config: `TransportKind` enum + `classify_transport()` centralize dispatch; `ServiceFieldSet` + `compute_service_fields()` centralize field queries. Remaining per-backend sites are inherent rendering differences.
- [ ] LanguageSpec completion — all target-language facts data-driven
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
  - [x] `CallableOf` variant for higher-order callback shapes
  - [ ] Derive T/K/V type parameter names from algebra declarations
- Tier 2.6 (functional system modeling):
  - [ ] Model function application as a concept (apply/call vs function-value-ref)
  - [ ] Inference encodes "this is a call" in the IR node, not as a type-arity heuristic
  - [ ] Dissolves `is_zero_arg_callable_ref` and `rt.name == "Callable"` L1 violation
  - Same pattern as iteration modeling (fold/descend/repeat): ad-hoc emit
    decisions are symptoms of a missing concept layer. Once the functional
    system is modeled, arity-based rendering questions disappear.
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

## CX: Complexity Analyzer (Lane 3)

**Status:** Down from 315 → 164 (main) → 76 after PR #318.
Ratchet constants not yet lowered (315/316).

**Root cause:** The analyzer maintains parallel heuristic classifiers
instead of consuming the structural facts already modeled in std/.
Violations are not analyzer bugs — they are missing facts about data
and operations. The path to 0 is modeling those facts, not extending
the analyzer with name-matching heuristics.

### Design principles

**Recursion is emergent.** Recursion is not a first-class concept to
model or analyze. It falls out of functions calling each other — just
like real programs don't "know" they're recursing. Only iteration
primitives (fold/descend/repeat) are modeled explicitly because they
are intentional developer constructs. The analyzer never tries to
"prove recursion terminates" — it computes the tightest bound it can
from structural facts about data and operations.

**All programs are bounded.** All data is ultimately quantifiable
(Bit/Word64). The analyzer reports HOW bounded, not WHETHER bounded.
If it can't derive a tight bound, the answer is Forever (2^63-1), not
a violation. Forever < infinity — it is "heat death of the universe"
scale, a concrete finite quantity. The system never deals with actual
infinity.

**No rejected patterns.** Every call pattern has a finite bound.
`self(same_arg)` → `repeat(Forever)`. The analyzer computes bounds;
it never rejects. CX violations mean "the analyzer couldn't derive a
bound from available facts" — the fix is providing the missing fact,
not adding a heuristic recognizer.

**Complexity classes are emergent.** Linear, quadratic, Forever are
themselves emergent properties of numbers and arithmetic. The
underlying number modeling is anemic for now but should improve. As
arithmetic facts become richer, complexity classes will emerge with
greater precision without analyzer changes.

**CX heuristics are CM gaps.** Every name-matching classifier in the
analyzer (Theme A items below) is a symptom of a missing concept in
std/. The fix path is the same as CM: model the fact, let the
property emerge, and the heuristic dissolves.

Computation model and migration plan:
[docs/cx-computation-model.md](docs/cx-computation-model.md)

DFA triage maps violations to four algebraic root causes:

| Root cause | Count | Fix |
|-----------|-------|-----|
| Parser SCCs | ~80 | DescentEvidence lattice (std/termination.dag) |
| Fold/catamorphism | ~40 | Descend primitive (std/iteration.dag) |
| CostExpr/SizeExpr | ~30 | Flat product-of-bounds (std/computation.dag) |
| Accessor-on-var | ~14 | Signature-driven fold (std/algebra.dag) |

### PR #318 work (CX-K through CX-R)

Reduced violation count via heuristic descent recognizers. Review
flagged 15 items as heuristics-over-structure — the count dropped but
the approach needs migration to structural authorities before the
remaining path is sound. Work accomplished:

- SCC proof constructor with TokenPosition progress dimension
- Parser edge classification (`collect_parser_edges_for_scc`)
- `is_algebra_iteration_method` reads `AlgebraMethodSemantics`
- Children iteration produces descent evidence
- Match-based descent detection (is_match_option_descent)
- Field projection descent recognition
- SCC parameter name unification in emit_pattern (CX-Q/CX-R)
- Soundness fixes: branching_only, all_safe, any-argument fabrication

### Review feedback (deferred — 15 items from PR #318)

Per Review Queue Discipline, these are recorded but not stacked:

**Theme A: Heuristic descent recognizers** (read structural facts instead)
- `complexity.dag:1962` — child-descent hardcodes `"children"` and list-methods
- `complexity.dag:2056` — `is_tree_size_preserving_wrapper` hardcodes callee name
- `complexity.dag:2088` — hardcoded `rt_type`, `param_node_type_expr`, `field_binding_pattern` as sub-value extractors
- `complexity.dag:2170` — treats any `param.field` as descent without structural witness
- `complexity.dag:2244` — `is_match_option_descent` is shape heuristic for missing Option/Result facts
- `complexity.dag:2312` — `lambda_param_names |> last` heuristic for missing method-signature facts

**Theme B: Producer patches** (fix analyzer root cause instead)
- `02_parse.dag:2759` — `node_to_name_str` split to make SCC shape friendlier
- `04_types.dag:418` — optional-cardinality split to make analyzer see Same edge

**Theme C: Fabrication / scope issues** (unsound)
- `complexity.dag:275` — `ParserResultDirectState` duplicates state-progress fact
- `complexity.dag:1382` — filtering `ProgressSame` self-edges fabricates acyclicity
- `complexity.dag:2346` — `val_inner_vars` leaks arm-local bindings into outer scope
- `complexity.dag:3061` — `branching_proof_safe` fallback fabricates LinearRecursion
- `same_progress_subgraph_has_cycle` drops self-loops for 1-node SCCs

**Theme D: Boundary / testing**
- `tests/pipeline.rs:2505` — diagnostic tests read workspace source tree (not hermetic)
- `05_emit_rust.dag:1334` — `emit_variant_pattern` returns empty string on impossible input

### Dependency chain

```
CX-D (model facts in std/)
 ├→ CX-B (wire LoweringTarget into analyzer, delete RecursionPattern)
 ├→ CX-A (consume DescentEvidence from std/termination.dag)
 ├→ CX-C (consume operation size contracts for fold evidence)
 └→ CX-E (re-enable gate once violations = 0)
      └→ PERF-3 (memory ratchet — can't validate until CX re-enabled)
```

**Dead code:** Three std/ declarations exist with no downstream consumer:
- `computation.dag: LoweringTarget` — defined, lowering table complete,
  but `complexity.dag` never reads it (CX-B not started)
- `termination.dag: is_valid_proof` — declared, returns `false`
  (CX-A not wired)
- `complexity.dag: RecursionPattern` (stage0 Rust) — should be replaced
  by `LoweringTarget` but both coexist with no bridge

### Work items

- **CX-D**: Model operation facts → heuristics dissolve. **Unblocked.**
  This is the primary remaining work. Three categories of structural
  facts are missing from the function model:
  (1) **Operation size contracts** — which operations shrink their
      input (list methods, Option unwrap) and by how much.
  (2) **Type structure facts** — field sub-value relationships
      (child access produces a smaller tree).
  (3) **Coproduct projection facts** — match arms narrow the type,
      producing strictly smaller data.
  The analyzer consumes these facts as single authority. The lowering
  table in `std/computation.dag` (CallPattern → LoweringTarget) already
  models (1) but has no downstream consumer in complexity.dag.
  Dissolves all Theme A and Theme B items. Unblocks producer-patch
  reversals (02_parse.dag, 04_types.dag).
- **CX-A**: DescentEvidence lattice unification — parser mutual recursion.
  **Blocked-by: CX-D** (needs structural evidence facts).
  Files: `complexity.dag`, `dsl/std/termination.dag`.
  Done: TokenPosition dimension, SCC proof constructor, edge classification.
  Deferred: ProgressSame self-edge filtering is heuristic (Theme C);
  ParserResultDirectState duplicates facts (Theme C).
- **CX-B**: CostExpr/SizeExpr dissolution — cost expressions become flat
  products of SizeBounds from `std/computation.dag`'s lowering table.
  **Blocked-by: CX-D** (needs operation size contracts consumed).
  Planned: RecursionPattern → LoweringTarget, UnresolvableRecursion
  deleted. See [migration phases](docs/cx-computation-model.md#migration-phases).
  Not started.
- **CX-C**: Signature-driven fold evidence — self-calls inside
  `children |> fold` callbacks get structural descent proofs.
  **Blocked-by: CX-D** (needs operation size contracts for fold).
  Done: `is_algebra_iteration_method` reads `AlgebraMethodSemantics`;
  children iteration produces descent evidence.
  Deferred: lambda element uses `last` convention (Theme A);
  child-descent hardcodes `"children"` (Theme A).
- **CX-E**: Re-enable complexity gate — remove `complexity_diags = []`,
  un-ignore 14 complexity tests (10 `complexity_*`, 3 `soundness_*`,
  1 `structural_classify_*`; all `#[ignore]` with "CX track" comment).
  **Blocked-by: CX-A, CX-B, CX-C** (0 violations required).

### Acceptance

0 violations without suppression. CX gate re-enabled. Node is the only
recursive type consumed by complexity analysis. All descent evidence
reads structural facts — no heuristic name-matching in the analyzer.

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
| Diagnostic ratchet | `strict_compile_diagnostic_count` | `#[ignore]`, ratchet 316 |

### Work items

- **PERF-1**: Un-ignore `performance_ratchet` in CI. Currently 30s
  budget with ~6.5s actual. Gate on this to catch O(n^2) regressions
  early. Requires CI runner has `cargo build --release` capacity.
- **PERF-2**: Un-ignore `bootstrap_stage0_to_stage1` in CI. This is
  the full regen + convergence test. Proves pass-1 = pass-2 on every PR.
- **PERF-3**: Track self-compile memory. The CX OOM root cause was
  repeated complexity classification on large compiles, not raw budget.
  Add a memory-usage ratchet or at minimum log peak RSS during
  `performance_ratchet`. **Blocked-by: CX-E** (complexity analysis
  currently disabled for memory; can't validate memory ratchet until
  CX is re-enabled and the classification OOM is resolved).
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

# CM: Compiler Concept Modeling (cross-cutting retrospective)

**Analysis:** [`src/v2/CM.md`](src/v2/CM.md),
[`src/v2/CM-inventory.md`](src/v2/CM-inventory.md)
(will fold into MODELING.md when stable)

**Principle:** Surface existing structural authorities to every
consumer. Never add parallel classification types. Consume existing
authorities, never duplicate. After each feature, retro for new
modeling gaps.

**What CM analysis found for each lane:**

| Lane | CM diagnosis | Highest-leverage fix |
|------|-------------|---------------------|
| M2 (Lane 1) | `ExprLet` erases expected type → bare_map_node chain → 5+ downstream fallbacks | Propagate expected through ExprLet; normalize ExprVar→ExprCall for nullary |
| CG (Lane 2) | 39 heuristic sites, all "existing authority not surfaced" | Surface connective through TypeRendering; surface AlgebraFieldTemplate to emit |
| CX (Lane 3) | 4 analyzer heuristics = 4 missing std/ facts | Model operation size contracts in std/computation.dag |
| M4 | CM provides endgame rationale: structural fields ARE identity, names are rendering sugar | Surface structural fields; delete Node.name |

**Unowned files (~42 heuristic sites in 04_resolve, 04_access, coercion, 00_core):**
No lane addresses these. Highest-value: coercion.dag container-to-algebra
table (should derive from algebra declarations), 04_resolve.dag alias
resolution (4 fail-open sites).

**Arity boundaries (cross-cutting):** Container under/over-arity,
empty services/resources, Optional collapse, Callable/lambda mismatch.
Enforce arity from algebra profile at normalization. See CM.md §Arity
boundaries.

**Relationship to other lanes:** M2, CG, and CX all generate work
items that are actually CM problems. When a PR "moves a heuristic
upstream," that's CM work wearing an M2/CG hat.

## MM-1/2/3: Detailed analysis in CM.md

Full heuristic inventory, acceptance criteria, and design constraints
are in [`src/v2/CM.md`](src/v2/CM.md). Summary:

## MM-1: Item identity

**Problem:** 4 independent classification forests, ~27 branches total.
- Raw structural interrogation in emit (`.connective ==`, `.body !=`, `.transport ==`): 55 sites
- `TypedItemUnhandled` / `""` / `false` fail-open fallbacks: 8 sites

**Work items:**
- [x] Design: determine irreducible structural facts about items
- [x] Surface existing fields (`body`, `transport`, `connective`, `params`) to
  every consumption site — no new classification type
- [~] Implement: fail-closed boundaries (no TypedItemUnhandled, no `""` fallbacks)
  `TypedItemUnhandled` variant deleted; else branches still emit error
  markers (compile_error/panic/comment) — need upstream diagnostic instead.
- [x] Delete: all `classify_*` forests, all fail-open fallbacks, all name-keyed side-tables
  PR #324: `TypedItemKind` enum + `classify_typed_item` dissolved. Shared
  boolean predicates (`is_type_def_item`, `is_function_item`, etc.) in
  05_emit.dag replace the taxonomy. Backends compose predicates directly.

**Acceptance:** Emit dispatches on existing Node structural fields
directly. Classification forests dissolve because consumers pattern-match
on the facts they need (`body != none`, `connective`) rather than on a
derived taxonomy.

## MM-2: Type structure

**Problem:** `Conj/Disj/NoConnective` is a primitive, but its
interpretation (product → struct, sum → enum, leaf → primitive) is
re-derived at every consumption site.

**Current counts (2026-04-05):**
- `.connective ==` in emit + lookup + types: 57 sites
- `TypeSummary.repr` (StructRepr/EnumRepr) exists but only used by some consumers
- Type rendering dispatch in `05_emit.dag:1095-1191`: 97 lines, 7 branches

**Work items:**
- [ ] Surface connective (the existing authority) to all consumers that
  currently re-derive it — connective and `TypeSummary.repr` are the
  authorities; no new interpretation type
- [ ] Extend `TypeSummary.repr` to cover all type rendering, not just
  type definitions
- [ ] Delete: all inline `.connective == Conj` / `Disj` re-interpretation
  in emit

**Acceptance:** Emit reads connective or `TypeSummary.repr` (existing
authorities) directly. No multi-branch re-interpretation of connective
in emit — type rendering is an exhaustive match on the existing
authority.

## MM-3: Expression semantics

**Problem:** Method identity is a string. Call semantics (nullary
invocation, method kind, built-in refinement) re-derived from name
matching at every consumption site.

**Current counts (2026-04-05):**
- `method_def.name` / `method_name ==` dispatch: 21 sites (emit_rust: 12, infer: 5, complexity: 4)
- Nullary function detection: 3 sites in emit_rust, 0 in Go/Python (bug)
- Built-in call type refinement by name: 4 sites in infer

**Work items:**
- [ ] Surface existing `method_def` Node and `AlgebraFieldTemplate`
  structural facts through the pipeline to emit — these ARE method
  identity; no new dispatch enum
- [ ] Normalize ExprVar → ExprCall during inference for nullary
  functions — the expression IR is the authority for invocation
  semantics
- [ ] Fix: Go/Python emit must handle nullary function references
  (ExprCall normalization fixes all backends at once)

**Acceptance:** Emit reads method structural facts from existing
`method_def` / `AlgebraFieldTemplate` authorities, not name strings.
Nullary invocation is ExprCall in the IR (normalized during inference),
not a type-name check at render time.

## CM acceptance (structural)

Each criterion is a claim: "this class of mistake is unrepresentable."
Consumers read existing structural authorities; the wrong question
can't be asked. See `src/v2/CM.md` for full design rationale.

**One clear end-state:** The existing structural data (Node fields,
connective, method_def, AlgebraFieldTemplate) flows through stage
boundaries intact to every consumer that needs it. No new parallel
classification types. No lossy boundaries that drop distinctions
consumers need. Reading a field IS reading the authority — that is
not re-derivation.

| Model | Claim | How to verify |
|-------|-------|---------------|
| MM-1 | Existing Node fields flow to emit intact; classification is unnecessary | No `classify_*` forests; emit reads `body`, `transport`, `connective` from the Node at the boundary |
| MM-2 | Connective / TypeSummary.repr is the authority; re-interpretation is unnecessary | Emit reads connective or repr — no inline `.connective == Conj` checks. TypeSummary.repr is only the single authority if proven non-lossy for ALL consumers, not just emit |
| MM-3 | method_def / AlgebraFieldTemplate is the authority; name dispatch is unnecessary | Emit reads structural method facts — no `method_name ==` string dispatch |

**Invariant guardrail:** Consume existing authorities, never duplicate.
Any proposed new fact layer or boundary type must demonstrate that
(a) no existing authority carries the needed fact, (b) the existing
authority cannot be surfaced to the consumer, and (c) the new layer
lands with a real downstream consumer in the same change. Speculative
metadata is rejected — the repo has already deleted prior boundary
layers that introduced unused or lossy fact tables.

---

# Layer 3: Deferred

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
| CostExpr/SizeExpr | Flat product-of-bounds via std/computation.dag | CX-B |
| TypeRendering | Coercion engine | M5 |
| MatchPattern | Node discriminant metadata | M7 |
| InferredNode | Keep wrapper, non-recursive reference | M7 |

Non-recursive authority leaks (same class of unfinished migration):

| Leak | Fix |
|------|-----|
| `Node.name` as authority (~256 sites) | M4 Lane 2 |
| Transport/config per-backend rendering | Inherent per-language differences |
| Bare parameterized types | Reject at normalization |
| ~~Missing `CallableOf`~~ | ~~M4 Tier 2.5~~ (DONE) |
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

## BP-1: Build Pipeline Model

The full compilation pipeline — `.dag` source → intermediate
representation (Rust/Python/Go) → executable artifact — is a
dependency DAG that should be modeled in `.dag`. Each stage has an
artifact type, a transform tool, and its own dependencies. This is
the same problem the compiler solves for user code: resolve a
dependency graph, detect cycles, determine evaluation order.

```
.dag source ──(v2-compiler)──▶ .rs/.py/.go ──(cargo/python/go)──▶ binary/runtime
     ▲                              │
     └──────── bootstrap cycle ─────┘  (Rust only: stage0 self-compile)
```

### Existing infrastructure

Pieces already modeled:

| Concept | Where | What it captures |
|---------|-------|-----------------|
| Intermediate targets | `src/v2/artifact.dag` `RenderTarget` | Rust, Python, Go, Dag |
| Artifact taxonomy | `src/v2/artifact.dag` `ArtifactKind` | ServiceBinary, Library, Frontend, GeneratedSupport |
| Per-target scaffold | `src/v2/languages.dag` `ProjectScaffold` | manifest file, source dir, extension |
| Cargo as tool | `dsl/extdeps/cargo.dag` | packages, targets, profiles, Build/Test/Clippy ops |
| Workspace graph | `dsl/extdeps/gunbc.dag` | `CrateRole`, `WorkspacePackage`, 13 packages |
| Build phases | `dsl/tools/build.dag` | multi-stage cargo pipeline with result aggregation |
| Codegen execution | `dsl/tools/codegen.dag` | stamp-file-gated conditional codegen |
| Output paths | `dsl/config/codegen_paths.dag` | output dirs, stamp files, path templates |
| Pipeline runs | `dsl/gunbc/workflow/types.dag` | `PipelineRun`, `PipelineArtifact`, stage outcomes |

What's missing: the **derivation chain** connecting these — which
artifact is produced from which source by which tool, the bootstrap
cycle and its fixed-point condition, and the generated-vs-source
distinction as structural data.

### Work items

- [ ] **BP-1a: Derivation model.** Types for pipeline stages, derivation
  edges (source artifact → tool → output artifact), and external tool
  dependencies. Compose existing `RenderTarget`, `ProjectScaffold`,
  cargo service, and `ArtifactKind` into an explicit derivation DAG.
  Location: `dsl/std/pipeline.dag` or extend `dsl/extdeps/gunbc.dag`.
- [ ] **BP-1b: Bootstrap cycle.** Model the self-compile cycle: stage0
  as bootstrap seed, the fixed-point condition (pass N == pass N+1),
  and the two-pass rule (required when emitter changes its own output).
  Hand-maintained files (9) enumerated as source artifacts distinct
  from generated artifacts.
- [ ] **BP-1c: `.gitattributes` generation.** Files the pipeline model
  marks as "generated" get `-merge` in `.gitattributes`. The
  `.gitattributes` file is a derived artifact from the pipeline model,
  not hand-maintained config. Analogous to how `dsl/tools/bootstrap.dag`
  generates `.gitignore` from `dsl/config/gitignore.dag`.
- [ ] **BP-1d: Merge workflow.** Document and enforce: resolve `.dag`
  conflicts → accept either side of generated stage0 → regen → commit.
  CI freshness gate (`check-stage0-freshness.sh`) is the safety net.

### Acceptance

1. The derivation chain from `.dag` source through intermediate
   representation to executable is explicit `.dag` data, not
   shell-script convention.
2. `.gitattributes` is generated from the pipeline model. Generated
   files have `-merge`; hand-maintained files do not.
3. Every file in `src/v2/stage0/src/` is classified as either
   generated or hand-maintained by the model.
4. The bootstrap cycle, fixed-point condition, and two-pass rule are
   structural facts in the model.
5. Stage0 merge conflicts no longer require manual resolution.

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
