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
| Bootstrap diagnostics (A) | 0 | 0 | Green — PR #264. Cherry-picked source-root fixes + removed mutual-recursion false positives |
| Bootstrap emitted Rust (B) | 419 errors | 0 | Down from 8658. Remaining: CodegenBackend import (192), algebra fn-field derives (71), downstream (114), misc (42) |
| Stage0 regeneration (C) | RED | GREEN | Blocked on B=0; stage0 emits 40 files but output doesn't compile yet |
| L1 ratchet | 21 | 0 | Down from 70; #253 landed structural algebra authority |
| L2 emit `.name` reads | 0 | 0 | All emit accessors migrated to `authored_name_at` |
| L2 resolve `.name` reads | 0 | 0 | `authored_name` eliminated; accessor layer still uses `node.name` internally |
| L2 `Node.name` constructors | ~256 | 0 | `make_*` helpers + direct constructions (D6) |
| Complexity violations | 315 | 0 | 27 root functions × indirect recursion → 315 errors (ratcheted); resolves when fold primitive lands |

---

## Bootstrap Health

Priority Zero is restoring a reproducible stage0 pipeline. Lane 1 and Lane 2 can
keep landing only when they do not obscure bootstrap health, but regeneration now
beats further ratchet-chasing.

Current reality:
- `std.types` injection is still an ambient bootstrap bridge until FF-9 becomes fully import-driven.
- Manual stage0 edits are still possible because regeneration is not green; that is the productivity failure we need to eliminate.
- The next milestone is not “more lane work,” it is “stage0 regeneration is authoritative again.”

Clean-repo workflow:
1. `cargo check -p v2-compiler`
2. `cargo test -p v2-compiler-tests full_dsl_compiles -- --ignored --nocapture`
3. `cargo test -p v2-compiler-tests strict_compile_diagnostic_count -- --ignored --nocapture`
4. When those are green, run `./scripts/regenerate-stage0.sh`
5. Require `git diff --exit-code src/v2/stage0`

Stabilization rules:
- No manual `src/v2/stage0/` edits once regeneration is green.
- Add CI gate: `./scripts/regenerate-stage0.sh && git diff --exit-code src/v2/stage0/`
- Prefer one owned bootstrap entrypoint over ad hoc cargo workflows; the invariant is reproducible stage0, not any particular wrapper name.

Owned bootstrap entrypoint contract:
1. Build/check the current compiler from a clean repo.
2. Run the source sanity gates (`full_dsl_compiles`, bootstrap diagnostic gate).
3. Run the stage0→stage1 emitted-Rust gate.
4. Run `./scripts/regenerate-stage0.sh`.
5. Fail if `src/v2/stage0/` differs after regeneration.
6. Report the live blocking counts so regressions are visible instead of hidden behind partial success.

Next passes:
1. Bootstrap A: restore the front-end/bootstrap diagnostic gates to a trustworthy green baseline.
2. Bootstrap B: reduce stage0→stage1 emitted-Rust failures until the bootstrap cargo-check ratchet is green.
3. Bootstrap C: make `regenerate-stage0.sh` a fixed-point clean-repo path.
4. Bootstrap D: wire the owned bootstrap entrypoint plus the CI diff gate, then forbid manual stage0 edits.
5. Resume broader Lane 1 / Lane 2 work only after A-D are stable.

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
- [x] Mutual recursion detection (SCC-based via Kahn's algorithm;
  `detect_mutual_recursion_names` in `complexity.dag`)

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
- [x] Dissolve `infer_lambda_with_callable_type` — ExprLambda
  `expected` context now handles callable-typed params positionally
- [x] Dissolve `infer_fold_lambda_arg` — call site builds synthetic
  callable `expected` with acc/elem param types; ExprLambda threads
  them positionally (same mechanism as callable_type dissolution)

*No-fabrication cleanup:*
- [x] Remove `Dynamic` as universal compatibility in `node_type_equals`
- [x] `LitNull` sentinel: parser error-recovery bridge, stays until
  parser redesign. Inference maps to `Optional<Unit>` (correct
  fallback). No behavioral change needed.
- [x] Callable-to-value fabrication: not found in current code.
  `lookup_in_scope` is a pure lookup with no synthesis.
- [x] `try_unwrap` clone fallback: ownership analysis
  (`ownership.dag`) already proves fallbacks unnecessary.
  Diagnostics wired into pipeline. Hard-error gate deferred
  until ownership violations are promoted from warnings.

*Codegen correctness (pre-existing, not new in this PR):*
- Primitive type lowering, algebraic types, callable type, async fn
  emission all work (confirmed, not changed by this PR)
- [x] Fix `uses` variable scoping in emission (emit side — infer
  side was already correct)
- [ ] Variadic arguments (currently strict arity; should be free from
  modeling)

*Emission correctness by construction (E0):*

Prerequisite for Bootstrap B. Two layers:

**E0a — Structural identity:** The emitter reads graph facts for
identifiers, not source-text recovery. Heuristic fallback chains
(`authored_name_at` → `source_text_at` → `node.name`) are boundary
sufficiency failures. Done for field bindings, let/var/call/method;
remaining sites in `authored_name_at` usage list.

- [x] `field_binding_name(fb)` for pattern field names
- [x] `expr_var_name`, `expr_call_func`, `expr_method_name`,
  `let_binding_name` for expression identifiers
- [ ] Narrow remaining `authored_name_at` to display/diagnostic only
- [x] Acceptance: `Color::Red { intensity: i }` emitted correctly
- [x] Acceptance: 122 pattern errors eliminated

**E0b — Value context modeling:** The emitter applies one sharing
strategy (Rc-wrap everything in Rust, identity in Python/Go) across
all contexts. This fails for constant data (`lazy_static` + `Rc` →
E0277 Send/Sync), algebra witnesses (`Rc<dyn Fn>` → E0369 PartialEq),
and static globals. The root cause: the graph doesn't carry HOW a
value is used, only WHAT it is.

Design: `EmitGraphInfo` carries `value_contexts: Map<String, ValueContext>`
precomputed alongside `type_summaries` and `recursive_type_set`.

```
type ValueContext
  = ConstantData        // immutable lookup table, known at compile time
  | RuntimeValue        // heap-allocated, shared, needs per-language wrapper
  | SpecificationWitness  // structural fact (algebra op), not runtime data
  | CallableValue       // function type, representation varies by language
```

Per-language emission reads ValueContext × LanguageSpec:

| ValueContext | Rust | Python | Go | SPICE | English |
|---|---|---|---|---|---|
| ConstantData | `const`/`static` | module-level | `var` (pkg) | `.param` | table |
| RuntimeValue | `Rc<T>` | `T` (GC) | `*T` | wire | paragraph |
| SpecWitness | phantom/tag | not emitted | not emitted | N/A | "satisfies" |
| CallableValue | `fn`/`Box<dyn Fn>` | `Callable` | `func` | N/A | "transforms" |

Extension point: `TypedItemKind` already has 8 discriminants,
`TypeSummary` already carries repr/fields. ValueContext is computed
from the same data (syntactic item kind + field types + usage sites)
and added to EmitGraphInfo in the same pass.

Acceptance criteria:
- [x] `data` declarations emit as constructor functions (no
  `lazy_static` + `Rc` → E0277 Send/Sync: 97→31)
- [x] ValueContext `{ is_constant, has_fn_fields }` precomputed in
  EmitGraphInfo (orthogonal flags, not sum type)
- [x] `fielded_variants` precomputed for structural variant-has-fields
- [ ] Wire `has_fn_fields` → skip `PartialEq`/`Debug` derives for
  algebra types (eliminates 40 E0369 + 31 E0277)
- [ ] Adding SPICE/English targets requires only ValueContext ×
  LanguageSpec data, no emission-side debugging
- [ ] `rc_types` authority derived from ValueContext (is_constant →
  no wrap) instead of heuristic type_summary scan

*Bootstrap:*
- [x] Bootstrap A: front-end/bootstrap diagnostic gates back to a trustworthy green baseline
- [x] `dag/syntax.dag` included in bootstrap (OOM resolved by FF-8)
- [ ] Bootstrap B: stage0→stage1 emitted-Rust gate back under ratchet
- [ ] Bootstrap C: regenerate stage0 with `regenerate-stage0.sh`
- [ ] Bootstrap D: owned bootstrap entrypoint in repo
- [ ] CI-verified regeneration (regenerate + diff = empty)

*User experience:*
- [x] `dsl/examples/weather/` committed example project
- Error messages already have file:line:col (pre-existing in main.rs)

**Bridges owned by M2:**

| Bridge | Delete trigger | Latest milestone |
|--------|---------------|-----------------|
| `COMPLEXITY_RATCHET = 0` | Fail-closed compilation → 0 violations | M2 (done — wired into pipeline) |
| Ambient/manual stage0 maintenance | `regenerate-stage0.sh` green + CI diff gate | M2 |

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

**Status:** L1 = 21. Depends on M2. Two exclusive lanes. Current Lane 1
direction: finish declaration-driven structural algebra, then remove
the remaining bootstrap/stage0 bridge work. Current FF-9 state is an
ambient `std.types` bootstrap bridge, not the final import-only
resolution model, so
`scripts/l1-ratchet.sh --check` can hit 0 instead of just enforcing a
lower ceiling.
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
- [ ] Replace template-era higher-order collection placeholders with
  function-typed algebra witnesses from `std/algebra.dag`
- [ ] Kernel types as algebraic compositions loaded from `std/`
- [ ] 21 type constructor sites → 0
- [x] Type-name comparisons → 0
- [ ] CollectionKind bridge dissolves when method algebras land

Files: `04_types.dag`, `00_core.dag`, `04_lookup.dag`,
`dsl/std/algebra.dag`, `dsl/std/types.dag`, `compile.dag`

#### Lane 2: D6 + emit + resolve (Node.name deletion)

**Status:** B3 (emit rendering) + B4 (resolve structural identity)
complete. D6 (constructor/accessor cleanup) is next — mechanical
work to update `make_*` helpers, drop `name:` from Node
constructions, and delete the field.
Note: final `Node.name` deletion depends on Lane 1 landing
declarations for kernel/algebra/container synthetic nodes.

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

*Resolve structural identity (B4 — accessor layer done, node.name
still semantic authority):*
- [x] Replace 5 `authored_name` semantic lookups in `04_resolve.dag`
  with node-based accessors — text recovery removed from resolve
- [x] Node-based accessor layer (`lookup_type_for`,
  `is_recursive_type_for`) encapsulates `.name` reads
- [ ] Accessors still derive identity from `node.name` — hiding
  the proxy, not replacing it with structure. True structural
  identity requires declaration-node references or span-based keys

*Node.name surface area (D6):*
- [x] `source_text_at` infrastructure (B0)
- [x] Source text threaded through pipeline (B2)
- [x] Synthetic name dissolution: tuple constants, module markers (B1)
- [x] `extern fn` syntax deleted
- [x] Accessor layer: `lookup_type_for`, `is_recursive_type_for`,
  `authored_name_at`, `lambda_param_names_at` encapsulate all
  `.name`-as-identity reads (emit + resolve + infer + lookup)
- [ ] Update 17 `make_*` helpers (blocked: all `.name` reads replaced)
- [ ] Update ~256 Node constructions to drop `name:`
- [ ] Migrate synthetic node identity to structural (blocked: L1)
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
- [x] Replace `ComplexityClassInfo` string bags with structural
  `CostExpr` — `classify_complexity` returns structural `CostExpr`
  (the single authority); no separate `ComplexityClass` type
- [x] `O(...)` strings exist only at formatting boundary —
  `format_complexity_class` is the canonical producer (convention;
  source-audit grep needed to enforce as invariant)
- [ ] Unknown complexity stays fail-closed; no steady-state `O(?)`
  success output — `is_unknown_class` / `cost_contains_unknown`
  provide structural detection; end-to-end gating in violation
  path needs wiring
- [x] `ClassProduct` formatting parenthesizes additive children
  (already done: `parenthesize_additive_cost` pre-existing)
- [x] Source-audit parity checks use `live_source` /
  `assert_live_*`, not raw `contains(...)` (already done:
  pre-existing for complexity section; new parity entries added)

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

*LanguageSpec completion (~11 fields + ValueContext rendering):*
- [ ] `statement_terminator`, `variable_declaration_keyword`,
  `assignment_operator`, `lambda_syntax`, `callable_type_template`,
  `error_expression`, `null_coalesce`, `string_interpolation`,
  `container_bracket`, `tuple_type_template`, `indentation_width`
- [ ] Per-ValueContext rendering templates (depends on E0b from M2):
  `constant_data_template`, `static_init_template`,
  `callable_type_template` (already listed above),
  `spec_witness_strategy` (phantom/tag/omit)

*LintModel (depends on E0 from M2):*
- [ ] Wire import rules, naming conventions, formatting model
- [ ] Acceptance: emitted code for every target language is
  syntactically valid by construction — no post-hoc validation
  needed. Adding SPICE/English/Markdown targets must not require
  emission-side debugging of identifier recovery or span bugs.

*Edge-only facts (Lane D, parallel):*
- [ ] 14 `Map<String, X>` metadata maps → structural edges

*Split authority dissolution (PR #264 review):*
- [ ] Merge `rt_functions: Map<String, Bool>` and
  `rt_bridge_function_names: Map<String, String>` in `rust/emit.dag`
  into a single `RuntimeFunction { name: String, bridge_name: String,
  passes_by_ref: Bool }` list — one concept, one authority

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

**Structural fold over recursive union types.** The language must
provide catamorphism/fold for any recursive type definition. Today,
every consumer of a recursive type (CostExpr, ExprData, MatchPattern,
Node children) writes the same manual traversal — 53 manual traversals
of CostExpr alone in `complexity.dag`. This is the root cause of:
- **Shallow projection bugs:** `is_unknown_cost` checked only the root
  node, missing `CostUnknown` nested inside `CostAdd`/`CostMul`/etc.
  The cost algebra composes correctly; consumer projections don't follow.
- **Glue code explosion:** N consumers × M variants = N×M match arms,
  all mechanically identical except the leaf logic.
- **Decidability gap:** 315 compiler functions use manual recursion
  (`parse`, `cost_of_expr`, etc.) instead of bounded fold. The
  decidability gate correctly flags them — the fix is the fold
  primitive, not suppressing the gate.

When `.dag` provides `fold` over recursive unions, `is_unknown_cost`
becomes `cost_expr_any(expr, fn(c) => c is CostUnknown)` — one line,
structurally correct, decidable. The compiler's own code replaces
manual recursion with fold, and 315 violations disappear.

Concrete acceptance criteria:
- `complexity.dag` uses fold/contains instead of 53 manual traversals
- `is_unknown_cost` is a one-liner, not a 10-line recursive function
- Parser uses fold over token sequences, not manual recursive descent
- Complexity gate: 315 → 0 without suppression or ratchet

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
| Self-compile diagnostics | 315 | 0 | `strict_compile_diagnostic_count -- --ignored` (all 315 are indirect-recursion complexity violations) |
| full_dsl_compiles | 0 | 0 | `full_dsl_compiles -- --ignored` |
| L1 type knowledge | 70 | 0 | `scripts/l1-ratchet.sh --check` |
| Complexity violations | 315 | 0 | `strict_complexity_violation_count -- --ignored` (27 root functions × indirect recursion → 315; resolves when fold primitive lands) |
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
