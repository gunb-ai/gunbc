# gunbc Roadmap

## Architecture (summary)

Two substrate primitives: **Node** and **Edge**. Everything else —
types, truth values, cardinality, product/coproduct — is compositional
modeling in `.dag`. Languages are coercion targets. Testing is
compilation.

Full thesis: [docs/architecture.md](docs/architecture.md)
Compiler laws and coercion model: [docs/compiler-laws.md](docs/compiler-laws.md)
Testing strategy: [docs/testing-strategy.md](docs/testing-strategy.md)
Invariant enforcement: [INVARIANTS.md](INVARIANTS.md)

---

## Dashboard

| Metric | Value | Target | Notes |
|--------|-------|--------|-------|
| .dag files | 90 | — | `dsl/` (+3 transport extdeps) |
| Self-compile time | 6.47s | <30s | Release. Tokenize 4.87s dominates |
| Self-compile diagnostics | 0 | 0 | Green |
| Files emitted | 40 | — | Rust target |
| `full_dsl_compiles` | PASSES (0 diag) | 0 | 90 dsl + 29 v2 files, M1 complete |
| Bootstrap ratchet (`DIAG_RATCHET`) | 65 | 0 | OOM resolved; 65 inference false-positives remain |
| L1 ratchet | 70 | 0 | 69 type constructors + 1 comparison |
| Complexity violations | 0 | 0 | Green |

---

## Critical Path

```
M1 (every .dag compiles)
 └→ M2 (working Rust codegen)
     ├→ M3 (test generation)          ← parallel with M4
     └→ M4 (L1 = 0, zero type names)
         └→ M5 (coercion engine + language plugins)
             └→ M6 (parse-emit symmetry)
                 └→ M7 (dissolve structural bridges)
```

---

## Milestones

### M1: Every .dag File Compiles (**COMPLETE**)

**Status:** Done. 90 dsl files compiled + 29 v2 files parsed, 0
diagnostics. Generic fn syntax already supported by stage0 parser.
**Gate:** `full_dsl_compiles` scans `dsl/` (compiled) and `src/v2/`
(parse-verified) with 0 diagnostics.

- [x] Parser: `fn foo<T>(...)` generic function syntax (already in
  stage0 via `parse_optional_type_params`)
- [x] All .dag files compile/parse clean (stale merge conflict in
  `05_emit_rust.dag` was the only issue)
- [x] Source discovery unified: `full_dsl_compiles` scans both trees,
  `strict_complexity_violation_count` uses import resolution (no
  hardcoded seeds). `prepare_sources` curated list documented as M2
  bridge (FF-8 OOM constraint).
- [x] Regression tests: generic fn (2 parse + 1 strict), single-variant
  enum, `uses` binding

---

### M2: Users Can Compile .dag to Working Rust

**Status:** In progress. Decidability gate, sharing bridge-reduction, inference context done.
**Gate:** `gunbc compile dsl/examples/weather/ --target rust && cargo check`

*Fail-closed decidability:*
- [x] Reject unchanged-argument recursion (`fn spin(n: n)` → error)
- [x] Reject ascending-argument recursion (`fn spin(n: n+1)` → error)
- [x] Allow proven descent (`n-1`, `n/2`, structural catamorphism)
- [x] Wire complexity ratchet into fail-closed gate
- [ ] Mutual recursion detection (SCC-based, not yet implemented)

*Container sharing (FF-8):*
- [x] Add `SharingStrategy.wrap_template` to `LanguageSpec`
  (Rust: `Rc<{0}>`, Python/Go: identity — bridge-reduction, not
  full authority dissolution)
- [x] Shared emitter reads `wrap_shared_type()` instead of
  hardcoding `Rc<...>` (rendering moved to spec; which-types-wrap
  decision still name-based via `rc_types`)
- [ ] Dissolve `rc_types` name-based wrapping authority
- [ ] Land atomically with stage0 regeneration

*Inference context (new):*
- [x] Add `expected: Node?` parameter to `infer_expr` (41 call sites)
- [x] ExprLambda uses `expected` for param typing (replaces
  `infer_lambda_with_element_type` bypass for `infer_arg_with_element_type`)
- [ ] Dissolve remaining bypass functions (`infer_lambda_with_callable_type`,
  `infer_fold_lambda_arg`)

*No-fabrication cleanup:*
- [x] Remove `Dynamic` as universal compatibility in `node_type_equals`
- [ ] Remove `LitNull` sentinel from inference (14 sites; 23 parser
  sites are OK — error recovery)
- [ ] Remove callable-to-value fabrication in `lookup_in_scope`
- [ ] Delete `try_unwrap` clone fallback

*Codegen correctness (pre-existing, not new in this PR):*
- Primitive type lowering, algebraic types, callable type, async fn
  emission all work (confirmed, not changed by this PR)
- [x] Fix `uses` variable scoping in emission (emit side — infer
  side was already correct)
- [ ] Variadic arguments (currently strict arity; should be free from
  modeling)

*Bootstrap:*
- [x] `dag/syntax.dag` included in bootstrap (OOM resolved by FF-8)
- [ ] Regenerate stage0 with `regenerate-stage0.sh` (blocked by 65
  inference false-positives: pipe-to-bare-function element type loss)
- [ ] CI-verified regeneration (regenerate + diff = empty)

*User experience:*
- [x] `dsl/examples/weather/` committed example project
- Error messages already have file:line:col (pre-existing in main.rs)

**Bridges owned by M2:**

| Bridge | Delete trigger | Latest milestone |
|--------|---------------|-----------------|
| `COMPLEXITY_RATCHET = 0` | Fail-closed compilation → 0 violations | M2 (done — wired into pipeline) |
| `DIAG_RATCHET = 65` | OOM resolved (FF-8); 65 inference false-positives | M2 |

---

### M3: Test Generation and Guarantee Receipt

**Status:** Not started. Depends on M2. Parallel with M4.
**Gate:** Receipt emitted every compilation. Generated Rust tests compile
and pass. Test freshness in CI.

**What exists today:** 184 tests pass, 9 ignored. `DryRunMode` pipeline
works. 9 scrambled-name tests in CI. Parse/emit round-trip smoke test.

*Guarantee receipt:*
- [ ] Define receipt schema as `.dag` type
- [ ] Compiler emits receipt on every `compile_sources` call
- [ ] CI validates receipt against ratchet values

*Behavioral tests:*
- [ ] Service mock tests compile and pass
- [ ] Type roundtrip, workflow dry-run, edge-contract harnesses

*Ratchet promotion (Tier 3 → CI):*
- [ ] Complexity violations, emitted Rust errors, ownership coverage,
  bootstrap fixed-point, performance — all promoted to CI gates

*Cross-language:*
- [ ] Python `py_compile`, Go `go vet`, same taxonomy across targets

---

### M4: Compiler Knows Zero Type Names (L1 = 0)

**Status:** L1 = 70. Depends on M2. Two exclusive lanes.
**Gate:** `scripts/l1-ratchet.sh --check` reports 0. Scrambled-name
tests pass (then deleted).

**Boundary rule:** `source_text_at` answers "what text was written
here?" for rendering and diagnostics. It must not become the
compiler's general answer to "what does this node mean?" —
`authored_name` is emit/diagnostic only, not semantic authority.

#### Lane 1: L1 → 0 (type knowledge dissolution + FF-9)

Goal: compiler reads type/algebra facts from `.dag` declarations
instead of hardcoding them. Includes FF-9 as prerequisite.

*Tier 1 — data tables → `.dag` declarations (no new infra):*
- [ ] Move `kernel_algebra_profile` to `dsl/std/algebra.dag` data
- [ ] Move `is_kernel_type` / `is_container_type` predicate lists
  to `dsl/std/types.dag` data
- [ ] Convert per-profile field builders to `.dag` functions

*Tier 2 — factor `enrich_kernel_type` (modest compiler change):*
- [ ] `enrich_kernel_type` calls `.dag` function in `std/algebra.dag`
- [ ] Delete `intrinsic_method_index()` /
  `runtime_bridge_method_index()`
- [ ] ~60 string branches → structural algebra queries

*Tier 3 — full structural algebra (requires FF-9):*
- [ ] FF-9: import-driven source resolution (compiler discovers
  modules transitively from source roots)
- [ ] Compiler reads type declarations + algebra edges from `.dag`
  at resolve time
- [ ] Kernel types as algebraic compositions loaded from `std/`
- [ ] 69 type constructor sites → 0
- [ ] 1 type-name comparison → 0
- [ ] CollectionKind bridge dissolves when method algebras land

Files: `04_types.dag`, `00_core.dag`, `04_lookup.dag`,
`dsl/std/algebra.dag`, `dsl/std/types.dag`, `compile.dag`

#### Lane 2: D6 + emit + resolve (Node.name deletion)

Goal: delete `Node.name` field. Rendering uses `source_text_at`,
resolve uses structural identity.

*Emit rendering (B3 — done):*
- [x] `authored_name` replaces `.name` in all 3 emit backends
  (Rust/Python/Go item, type-def, service, resource, operation)
- [x] `find_shared_enum_fields` aligned with `authored_name`
- [x] Narrow `TypeEnv` → `source_index: NewlineIndex?` in emit
  helpers (reviewer: TypeEnv is too wide for rendering)
- [x] Migrate `param_node_name` → `authored_name_at` in emit
  (same-module render sites done; cross-module boundary sites
  `order_typed_call_args` and `fill_default_args` remain on
  `param_node_name` — caller `source_index` can't recover callee
  param names across module boundaries; needs precomputed names
  at resolve time)

*Resolve structural identity:*
- [ ] Replace 5 pre-existing `authored_name` semantic lookups in
  `04_resolve.dag` with structural identity (B4 scope — text
  recovery is not semantic authority)
- [ ] Design structural identity model for resolve phase

*Node.name surface area:*
- [x] `source_text_at` infrastructure (B0)
- [x] Source text threaded through pipeline (B2)
- [x] Synthetic name dissolution: tuple constants, module markers (B1)
- [x] `extern fn` syntax deleted
- [ ] Update 17 `make_*` helpers + 11 accessor functions
- [ ] Update ~256 Node constructions to drop `name:`
- [ ] Migrate synthetic node identity to structural
- [ ] Delete `Node.name` field + scrambled-name tests

*Synthetic node audit (D6 blocker):*

| Family | Count | Deletion point |
|--------|-------|---------------|
| Kernel type constants | 6 | `std/types.dag` declarations (Lane 1) |
| `leaf_node(name: ...)` | 68 L1 | Declaration edges (Lane 1) |
| Algebra method fields | ~50 | `std/algebra.dag` declarations (Lane 1) |
| Tuple children | 2 | `.dag` type definition |
| Optional skeleton | 3 | `.dag` type definition |
| Module/import markers | 3 | Property values (B1c done) |
| `error_type` / `none_type` | 2 | Permanent (compiler infra) |
| Container/callable/map nodes | ~15 L1 | `.dag` declarations (Lane 1) |

Files: `05_emit*.dag`, `04_resolve.dag`, `04_env.dag`,
`02_parse.dag`, `04_infer.dag`, `00_core.dag` (make_* helpers only —
kernel type defs are Lane 1)

#### Lane exclusivity

Only shared file: `00_core.dag`. Lane 1 edits kernel type
definitions/predicates. Lane 2 edits Node construction helpers.
Different functions, no conflict.

*Structural complexity facts (moved from M2 / PR #249):*
- [ ] Replace `ComplexityClassInfo` string bags with structural
  `ComplexityClass`
- [ ] `O(...)` strings exist only at formatting boundary
- [ ] Unknown complexity stays fail-closed; no steady-state `O(?)`
  success output
- [ ] `ClassProduct` formatting parenthesizes additive children
- [ ] Source-audit parity checks use `live_source` /
  `assert_live_*`, not raw `contains(...)`
- [ ] Delete `large_complexity_report_limit` / large-report elision
  bridge

**Bridges owned by M4:**

| Bridge | Delete trigger | Latest milestone |
|--------|---------------|-----------------|
| `node.name: String` | `source_text_at` + edges replace all reads | M4 (Lane 2) |
| `kernel_types: List<String>` | `List<Node>` edges to type defs | M4 |
| `container_types: List<String>` | `List<Node>` edges to type defs | M4 |
| `builtin_function_registry()` | ~260 calls → method syntax | M4 |

---

### M5: Coercion Engine + Language Plugin Extraction

**Status:** Design phase. Depends on M2, M3, M4.
**Gate:** Zero `match render_target` branches. Zero language mentions in
`src/v2/*.dag`. LintModel validates every emitted file.

*Coercion engine (Lane C):*
- [ ] `05_emit.dag` walks typed graph, invokes language-declared
  coercion
- [ ] Delete `05_emit_rust.dag` (4,121 lines) →
  `dsl/extdeps/languages/rust/`
- [ ] Delete `05_emit_python.dag` (1,349 lines) →
  `dsl/extdeps/languages/python/`
- [ ] Delete `05_emit_go.dag` (1,387 lines) →
  `dsl/extdeps/languages/go/`
- [ ] Delete `runtime_rust.dag` → `rust/runtime.dag` extdep

*LanguageSpec completion (~11 fields):*
- [ ] `statement_terminator`, `variable_declaration_keyword`,
  `assignment_operator`, `lambda_syntax`, `callable_type_template`,
  `error_expression`, `null_coalesce`, `string_interpolation`,
  `container_bracket`, `tuple_type_template`, `indentation_width`

*LintModel:*
- [ ] Wire import rules, naming conventions, formatting model

*Edge-only facts (Lane D, parallel):*
- [ ] 14 `Map<String, X>` metadata maps → structural edges

*Compiler bug fixes owned by M5:*
- [ ] Optional exhaustiveness: structural, not `Some`/`None` hardcoded
- [ ] Single-variant enum parsing
- [ ] Sharing model into LanguageSpec (Rc/pointer/reference as
  cross-language concern)
- [ ] Option rendering into LanguageSpec declaration

*Challenge targets (design validation):*
- [ ] Verilog, SPICE, English/Markdown coerce+render

**Bridges owned by M5:**

| Bridge | Delete trigger | Latest milestone |
|--------|---------------|-----------------|
| `05_emit_rust/python/go.dag` in `src/v2/` | Moved to plugin dirs | M5 |

---

### M6: Parse-Emit Symmetry

**Status:** Design only. Depends on M4, M5.
**Gate:** `parse(spec, emit(spec, graph))` produces identical graph for
all `.dag` files.

- [ ] Round-trip smoke test on `.dag` subset
- [ ] Statement dispatch spec-driven (3 keyword arms)
- [ ] Block/record disambiguation spec-driven
- [ ] Second language frontend

---

### M7: Dissolve Structural Bridges

**Status:** Design only. Depends on M6.
**Gate:** `connective` removed. `Cardinality` removed. No structural
enums — compiler reads the graph.

- [ ] `Conj/Disj` → edge connectivity model
- [ ] `Cardinality` → edge existence
- [ ] Bit-graph representation for fixed-width types
- [ ] Full structural type algebra with denotational laws

**Bridges owned by M7:**

| Bridge | Delete trigger | Latest milestone |
|--------|---------------|-----------------|
| `connective: Conj/Disj` | Edge connectivity replaces enum | M7 |
| `return_cardinality` | Edge existence replaces enum | M7 |

---

## Exploratory Directions

**Unified Sequence (Seq\<T>).** Ordered collections share FreeMonoid
algebra; access pattern determines representation. Mixed access = type
error.

**Space complexity as peer dimension.** `space: CostExpr` peer to `work`
and `span`. Currently `output_size` is unpopulated.

**Everything is coercion.** Unifying concept: minimal complete
representation in a target domain. Applies at stage boundaries, type
compatibility, and language rendering.

---

## Verification

| Ratchet | Current | Target | Command |
|---------|---------|--------|---------|
| Self-compile diagnostics | 0 | 0 | `strict_compile_diagnostic_count -- --ignored` |
| full_dsl_compiles | 0 | 0 | `full_dsl_compiles -- --ignored` |
| L1 type knowledge | 70 | 0 | `scripts/l1-ratchet.sh --check` |
| Complexity violations | 0 | 0 | `strict_complexity_violation_count -- --ignored` |
| Emitted Rust errors | 880 | 0 | `bootstrap_stage0_to_stage1 -- --ignored` |
| Bootstrap fixed point | PASSES | PASSES | `bootstrap_fixed_point -- --ignored` |
| Performance | <30s | <30s | `performance_ratchet -- --ignored` |

### CI Gates

| Gate | Command |
|------|---------|
| Unit tests | `cargo test --workspace --exclude v2-compiler-tests` |
| Clippy | `cargo clippy --all-targets -- -D warnings` |
| V2 compiler tests | `cargo test -p v2-compiler-tests` |
| Scrambled-name | `cargo test -p v2-compiler-tests scrambled_name` |

### Required Before Merge (Tier 3)

```
scripts/l1-ratchet.sh --check
cargo test -p v2-compiler-tests full_dsl_compiles -- --ignored
cargo test -p v2-compiler-tests strict_compile_diagnostic_count -- --ignored
cargo test -p v2-compiler-tests bootstrap_fixed_point -- --ignored
```
