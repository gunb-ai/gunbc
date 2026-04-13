> Part of: [THESIS.md](../THESIS.md) > [ROADMAP.md](../ROADMAP.md)
> See also: [dag-vocabulary-reconciliation.md](dag-vocabulary-reconciliation.md),
> [binding-model-proposal.md](binding-model-proposal.md)

# Compiler Reduction Plan

Per-stage audit of the compiler pipeline. For each stage: what's
minimal (must exist), what dissolves (exists only because facts
aren't carried on edges), and what merges (separate file with no
distinct concern).

**Total:** 38,078 lines across 32 `.dag` files.

**Thesis:** the compiler constructs proofs from SubValueRelation
on edges. Parse produces the Node tree. Resolve connects modules.
Infer attaches SVR + types to every edge. CX constructs
TerminationProof from SVR. Ownership derives from SVR + UsageEdge.
Emission reads specs + proofs, mechanical translation. Everything
else is either a fact (std/), a spec (languages/coercion), or
reconstruction of facts that should already be on the edges.

---

## The pipeline and line counts

```
Stage 1: Lexing
  01_tokenize.dag                 522

Stage 2: Parsing
  02_parse.dag                  4,827

Stage 3: Module resolution
  03_resolve.dag                  461
  03_normalize.dag                 91  ← merge candidate

Stage 4: Type inference (attach SVR + types to edges)
  04_infer.dag                  5,470  ← 2nd largest: main reduction target
  04_types.dag                    992  ← pure Node vocabulary, essential
  04_resolve.dag                  992  ← TypeEnv-driven resolver, essential
  04_env.dag                      133  ← TypeBinding + TypeEnv, essential
  04_patterns.dag                 242  ← match exhaustiveness, distinct
  04_lookup.dag                   346  ← method/scope lookup, distinct
  04_items.dag                    150  ← item classification, small
  04_access.dag                   129  ← merge candidate
  04_service.dag                  250  ← service graph, distinct
  04_sigs.dag                     262  ← func sig resolution, distinct
  04_method.dag                   113  ← merge candidate (bridge)
  04_cycle.dag                    156  ← merge candidate
  04_emit_info.dag                398  ← infer→emit boundary, distinct

Stage 5: Proof construction
  complexity.dag                5,489  ← 3rd largest: parallel classification dissolves
  ownership.dag                   635  ← string matching dissolves

Stage 6: Emission
  05_emit.dag                   3,003  ← shared emission kernel
  05_emit_rust.dag              5,894  ← largest file: ownership emission dissolves
  05_emit_go.dag                  689  ← mostly unified via LanguageSpec
  05_emit_python.dag              666  ← mostly unified via LanguageSpec

Infrastructure
  00_core.dag                   1,702  ← IR types, tables, utilities
  compile.dag                   1,065  ← pipeline orchestration
  languages.dag                 1,163  ← LanguageSpec builders (data)
  coercion.dag                    297  ← type realization (data)
  artifact.dag                    113  ← RenderTarget, ArtifactPlan
  runtime_rust.dag                279  ← Rust v2_rt shim text
  compiler_tests_rust.dag       1,260  ← test extraction
  effect_derivation.dag            66  ← bridge for stage0
  trace.dag                       223  ← runtime debug contract
```

---

## Per-stage reduction

### Stage 1: Tokenize (522 lines) — minimal

The tokenizer is small and mostly necessary. Minor opportunities:
- Two-char operator `if` chain could be a table lookup
- `should_start_interpolation` is ad-hoc (4 conditions)
- `single_punct` table is already data-driven

**Reduction: ~0 lines.** Not a priority.

### Stage 2: Parse (4,827 lines) — moderate

The parser is large because recursive descent over a full language
is inherently large. But there's mechanical duplication:

- **~54 `is_*_shape` + ~30 `tok_is_*` predicates** — these are
  one-line wrappers around `match token.shape { ShFoo => true _ => false }`.
  Could be one generic `token_is(token, shape)` function. ~80 lines.
- **`ParserResultWitness` + `ParserHelperIdentity` + `ParserCallIdentity`**
  — downstream static analysis for CX parser progress proofs, not
  parsing. Could move to `complexity.dag` or a parser-analysis module. ~100 lines.
- **Hardcoded keyword strings** in `parse_stmt`, `parse_primary`,
  etc. that overlap `SyntaxSpec` / `dag_keyword_set`. Not wrong, but
  parallel to the spec — drift risk.

**Reduction: ~180 lines** mechanical compression + ~100 lines
of witness machinery that could relocate.

### Stage 3: Resolve (552 lines) — clean

`03_resolve` (461) is essential — module graph, imports, topo sort.
`03_normalize` (91) is bare containers check — **merge into
03_resolve** (one file, one responsibility: validate module graph).

**Reduction: 1 file dissolved** (-91 lines of file overhead, 0 logic lost).

### Stage 4: Inference (9,643 lines) — PRIMARY REDUCTION TARGET

**04_infer.dag (5,470 lines)** is the heart of the compiler and the
main target. Three categories of code:

**A. Essential inference (~3,500 lines):** Type resolution, scope
management, expression typing, pattern checking, import merging,
module-level typechecking. This stays.

**B. Classification/reconstruction (~1,200 lines) — DISSOLVES:**

| Function | Lines | Why it exists | After SVR on edges |
|----------|-------|--------------|-------------------|
| `classify_argument` | ~170 | Re-derive SVR from AST | Read `binding.provenance` |
| `classify_let_value` | ~100 | Re-derive SVR for let expressions | Read `binding.provenance` |
| `classify_binding_provenance` | ~30 | Narrow bind-time classifier | Inlined into binding creation |
| `classify_body_provenance` | ~150 | Re-derive SVR for output provenance | Read from bindings in body |
| `annotate_descent` | ~260 | Walk body to build `sub_value_vars` | SVR already on bindings |
| `annotate_descent_evidence` | ~20 | Entry to above | Thin reader |
| `DescentContext` type + threading | ~100 | Parallel SVR map | Dissolved — bindings carry SVR |
| `lambda_param_provenance` on InferScope | ~30 | Side-channel for fold element provenance | Dissolved — SVR from call site |
| `build_call_evidence` | ~30 | Assemble descent evidence per call | Read SVR from arg bindings |
| Various `read_arg_provenance` fallbacks | ~100 | Fallback when binding SVR unavailable | Binding SVR always available |
| `per_field_vars` on DescentContext | ~50 | Track per-field output provenance | Read from callee sig |
| `SizeExpr` (ParamSize\|DividedSize) | ~30 | Descent size aliases | Dissolved into SVR factor |

**C. Result/Accum types (~25 types, ~300 lines) — SIMPLIFY:**

Most are `{ value, diagnostics }` pairs. A generic `WithDiag<T>`
pattern would eliminate ~20 of these. Not a code reduction (the
match sites stay) but a type-count reduction.

**Sub-file merges:**
- `04_access` (129) → into `04_resolve` (same concern: type constraint checks)
- `04_method` (113) → into `04_types` (bridge for builtins, dissolves when builtins are data)
- `04_cycle` (156) → into `04_sigs` or `04_resolve` (type cycle detection)

**Reduction: ~1,200 lines dissolved** + 3 files merged (saves ~398 lines of file overhead).

### Stage 5: Proof construction (6,124 lines) — significant

**complexity.dag (5,489 lines):**

The cost algebra, SCC analysis, parser progress model, and
termination proof construction are essential. What dissolves:

| Function | Lines | Why it exists | After SVR on edges |
|----------|-------|--------------|-------------------|
| `classify_self_call_evidence` | ~60 | Parallel classification of self-call args | Read annotated `descent_evidence` from ExprCall |
| `collect_evidence_incremental` | ~200 | Incremental let/match/if evidence | Read SVR from bindings |
| `construct_termination_proof` fallback | ~70 | When annotated evidence is weak | Annotated evidence is always strong |
| Various hardcoded accessor/contraction tables | ~50 | `function_size_effects`, `is_child_accessor_in_model` | Structural from type defs |

**Reduction: ~380 lines dissolved.**

**ownership.dag (635 lines):**

| Pattern | Lines | Why it exists | After SVR + UsageEdge |
|---------|-------|--------------|----------------------|
| `fname == "fold"` string match (ExprCall) | ~20 | Detect fold for Threaded usage | Structural from call semantics |
| `mname == "fold"` string match (ExprMethodCall) | ~25 | Same, for method calls | Structural from call semantics |
| `a.name == "init"` string match | ~10 | Find fold accumulator arg | Structural from parameter position |
| Duplicate ExprCall/ExprMethodCall fold logic | ~60 | Two copies of same pattern | One path reads call structure |
| `analyze_ownership` separate-pass overhead | ~30 | Walk after inference | Computed during inference |

**Reduction: ~145 lines dissolved.** The remaining ~490 lines
(walk_expr core, branch merges, fan-out counting, fold proofs)
are essential ownership analysis that stays.

### Stage 6: Emission (10,252 lines) — moderate

**05_emit_rust.dag (5,894 lines):** The largest file. Most of it
is Rust-specific emission (derives, Rc patterns, ownership-aware
patterns). What changes with ownership as a dimension:
- Ownership-aware match arm emission simplifies (~200 lines)
- `build_shared_types` / `build_ownership_results` reconstruction
  dissolves when ownership facts are on TypeBinding (~100 lines)
- `VarBindingKind` matching dissolves (~20 lines)

**Reduction: ~320 lines.** But the bulk stays — Rust emission is
inherently complex (derive macros, module structure, cargo config).

**05_emit.dag (3,003 lines):** The shared kernel. Already well
factored via LanguageSpec. `emit_shared_expr` dispatches 22 variants
— this is necessary. Minor opportunities from binding unification
(fewer cases in expression emission) but small.

**05_emit_go.dag (689) + 05_emit_python.dag (666):** Already
mostly unified via LanguageSpec (Phase 5). Little to reduce.

**Reduction: ~320 lines** from Rust emitter.

### Infrastructure (4,869 lines)

**00_core.dag (1,702 lines):** Key reductions from reconciliation:
- `VarBindingKind` dissolution (~10 lines of type def, ~30 lines
  of `var_binding_kind_name` and related)
- `NodeFieldRole` derived from structural edges (~15 lines)
- `expr_child_roles` / `wrapper_child_roles` eventually derived
  from typed edges (~50 lines of table)
- `function_size_effects` derived from type contracts (~10 lines)
- `Connective` dissolution (longer-term, 313 consumer sites)

**Reduction: ~100 lines** near-term, plus Connective later.

**languages.dag (1,163) + coercion.dag (297):** These are data
files — one entry per language feature per target. They should be
this size. No reduction.

**compile.dag (1,065):** Pipeline orchestration + JSON serialization.
The JSON chunk (~400 lines) could move to a serialize module but
doesn't reduce total lines.

---

## Summary

| Category | Lines | Estimated reduction | Remaining |
|----------|------:|-------------------:|----------:|
| Parse | 4,827 | ~280 | ~4,547 |
| Resolve | 552 | ~91 (merge) | ~461 |
| Inference | 9,643 | ~1,600 | ~8,043 |
| Proof construction | 6,124 | ~525 | ~5,599 |
| Emission | 10,252 | ~320 | ~9,932 |
| Infrastructure | 6,680 | ~100 | ~6,580 |
| **Total** | **38,078** | **~2,916** | **~35,162** |

**~2,900 lines dissolve** — code that exists ONLY because facts
aren't carried on edges. This is ~8% of the compiler.

**3 file merges** reduce the module count from 32 to 29.

The remaining 35,000 lines are the actual compiler: parsing a
language, resolving modules, inferring types, constructing proofs,
emitting code. That's the minimal compiler for the thesis.

---

## The validator properties (the thesis, restated)

The compiler's job, reduced to 4 proof constructions:

1. **Consistent** — type dimension: every edge has compatible
   types at source and target. Already working (TypeMismatch,
   NonExhaustiveMatch, FieldNotFound diagnostics).

2. **Minimal** — cost dimension: every function has a proven cost
   bound (TerminationProof from SVR). 420 violations → 0 is the
   CX gate. KF-2 (reject suboptimal) is the stretch goal.

3. **Safe** — effect dimension: every workflow's effect composition
   is sound (EffectShape from operations). Not yet wired.

4. **Fact-respecting** — every service boundary matches its declared
   contract (extdeps types). Partially enforced (typed transports,
   REST contracts).

These are NOT four separate passes. They are four columns in the
SVR-keyed dimension table. One mechanism, four instantiations.
The compiler constructs proofs for each dimension during inference
(attach to edges) and checks them at consumption points (gates,
diagnostics). If the proof can't be constructed, that's a
construction failure — a diagnostic — not a post-hoc validation.

---

## Execution priority

Ordered by impact (lines dissolved + thesis alignment):

1. **SVR on every binding edge** (Theme 1) — unlocks ~1,600 lines
   of inference dissolution + ~380 lines of CX dissolution.
   The single highest-leverage change.

2. **Ownership as dimension** (Theme 3) — unlocks ~145 lines of
   ownership dissolution + ~320 lines of Rust emission dissolution.

3. **File merges** (04_access, 04_method, 04_cycle, 03_normalize)
   — mechanical, can happen any time.

4. **Parse compression** (predicate dedup, witness relocation)
   — mechanical, low priority.

5. **Connective dissolution** — 313 sites, large mechanical
   refactor, tracked separately.

6. **KF-2 / effect wiring** — stretch goals that complete the
   validator story.
