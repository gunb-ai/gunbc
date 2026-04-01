# gunbc Roadmap

## Architecture (summary)

Two substrate primitives: **Node** and **Edge**. Everything else —
types, truth values, cardinality, product/coproduct — is compositional
modeling in `.dag`. Languages are coercion targets. Testing is
compilation.

Full thesis: [docs/architecture.md](docs/architecture.md)
Compiler laws and coercion model: [docs/compiler-laws.md](docs/compiler-laws.md)
Coercion design (algebra-keyed inhabitants): [docs/coercion-design.md](docs/coercion-design.md)
Testing strategy: [docs/testing-strategy.md](docs/testing-strategy.md)
Invariant enforcement: [INVARIANTS.md](INVARIANTS.md)
Modeling guidelines: [MODELING.md](MODELING.md)

---

## Dashboard

| Metric | Value | Target | Notes |
|--------|-------|--------|-------|
| .dag files | 91 | — | `dsl/` (+3 transport extdeps) |
| Self-compile time | 6.47s | <30s | Release. Tokenize 4.87s dominates |
| Self-compile diagnostics | 0 | 0 | Green |
| Files emitted | 40 | — | Rust target |
| `full_dsl_compiles` | PASSES (0 diag) | 0 | 91 dsl + 29 v2 files, M1 complete |
| Bootstrap diagnostics (A) | 0 | 0 | Green — PR #264. Cherry-picked source-root fixes + removed mutual-recursion false positives |
| Bootstrap emitted Rust (B) | 12 errors | 0 | Down from 8658→99→12. TypeRendering + always-annotate-let + inference fixes. PR #277. Two independent categories remaining (8 imports + 4 inference) |
| Stage0 regeneration (C) | RED | GREEN | Blocked on B=0; stage0 emits 40 files but output doesn't compile yet |
| L1 ratchet | 21 | 0 | Down from 70→22→21; Set/NonEmptySet profile fix + algebra fn conversion |
| L2 emit `.name` reads | 0 | 0 | All emit accessors migrated to `authored_name_at` |
| L2 resolve `.name` reads | 0 | 0 | `authored_name` eliminated; accessor layer still uses `node.name` internally |
| L2 `Node.name` constructors | ~256 | 0 | `make_*` helpers + direct constructions (D6) |
| Complexity violations | 313 | 0 | 53 root functions → 313 errors (ratcheted); direct + mutual recursion are fail-closed, next drop needs parser/block/type-normalization witness work |

---

## Active: Bootstrap B → 0 (12 errors remaining)

TypeRendering infrastructure landed. Always-annotate let bindings
enforced. 10 reviewer violations resolved. Fold inference improved.
Error count: 99 → 12 (87 fixed). PR #277.

**Remaining 12 errors (2 independent categories):**

### Category A: Cross-module imports (8 E0425) — separate branch

Functions from `dsl/std/algebra.dag` (e.g., `partial_function_templates`,
`free_monoid_collection_templates`) used in `04_types.dag` and
`complexity.dag` but not in the emitted import lists. Previously
masked by `compile_error!` sentinels; revealed when fold inference
fixes resolved the sentinels. Same pattern as TestConventions/Token
fix: add functions to `.dag` import statements + stage0 `pub use`.

Files: `src/v2/04_types.dag`, `src/v2/complexity.dag`, and their
stage0 counterparts. No overlap with Category B.

### Category B: Nested collection bidirectional inference (4 E0282) — separate branch

`Map<String, List<Unit>>` fold accumulators where the inner
`List<Unit>` comes from empty `[]` literals inside struct fields.
The single-pass inference pipeline resolves struct fields top-down
but can't propagate the actual element type from downstream fold
body back into the struct's list field type. Needs field-level
bidirectional type unification.

Files: `src/v2/stage0/src/v2_compiler_infer.rs` (fold inference
path). No overlap with Category A.

**Reviewer-flagged structural debt (tracked, not blocking merge):**

- `find_struct_by_fields`: heuristic name match → needs record literal
  inference to carry struct name from boundary (M2 boundary sufficiency)
- Fold refinement magic names (Unit/Dynamic/Error) → needs bidirectional
  fold type unification in inference (M2 boundary sufficiency)
- `with()` string-keyed branch → needs algebra operation registry
  (M4 Lane 1, algebra-driven builtin operations)
- TypeRendering mixes backend policy (shared/boxed) with diagnostics
  (is_error) → dissolves into coercion engine (M5)
- Codepoint carrier: chars() returns List\<Int> but should model
  codepoints explicitly → algebra design (M4 Tier 2.5)

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

### Compiler development vs compiler usage

Two distinct workflows. The compiler is a living system — compiler
development is continuous, not a one-time bootstrap.

**Compiler usage (stable binary):**
```
User installs gunbc binary (built once from committed stage0)
User writes .dag programs
gunbc compile project/ --target rust → working Rust code
gunbc compile project/ --target python → working Python code
```
No regeneration. No cargo. No Rust knowledge needed. The binary is
the product. It updates when the compiler team ships a new version.

**Compiler development (bootstrap loop):**
```
Developer edits src/v2/*.dag (compiler source)
  ↓
gunbc-dev build
  ├─ stage0 (committed Rust) compiles .dag → stage1 (new Rust)
  ├─ cargo check stage1 (must pass — 0 errors)
  ├─ stage1 compiles .dag → stage2 (fixed point check)
  ├─ diff stage1 stage2 (must be empty)
  └─ stage1 replaces stage0 (regeneration)
  ↓
Commit includes updated stage0 (generated, not hand-edited)
CI verifies: regenerate → diff → empty
```
The `gunbc-dev build` command owns the entire loop. It is the
single entrypoint for compiler development. Developers never run
`regenerate-stage0.sh` manually — the build tool does it.

**Why the committed binary approach works for continuous development:**
- Every commit is self-contained: clone repo, `cargo build -p v2-compiler`, you have a working compiler
- No external bootstrap compiler needed
- The bootstrap binary (stage0) is Rust — cargo builds it on any platform
- CI gate ensures stage0 is always in sync with .dag source
- Rolling forward is safe: each commit's stage0 can compile the next commit's .dag

**Transition off Rust/cargo entirely:**

The current bootstrap medium is Rust (stage0 is Rust, cargo builds
it). This is not permanent. The path off Rust:

1. **Current:** .dag → Rust → cargo → binary
2. **M5-full:** .dag → {Rust, Go, Python, ...} → language toolchain → binary
3. **Self-hosted build:** .dag compiler compiles its own build system from .dag source
4. **Post-Rust:** .dag → native code (LLVM/Cranelift) directly, no Rust intermediate

Step 3 is the key transition: the .dag build system is itself a .dag
program. It knows how to invoke the target language's toolchain
(rustc, go build, gcc) because that's modeled as a language extdep.
The build process is:
```
stage0 binary (committed, any language)
  compiles .dag build system → build binary
  compiles .dag compiler → compiler binary
  build binary orchestrates: test, package, ship
```

Step 4 is optional — Rust/Go as intermediate targets may be
sufficient indefinitely. The decision depends on whether the
intermediate compilation step becomes a bottleneck.

**Stability contract for compiler developers:**
- Editing `dsl/` (std library, extdeps, user programs): stage0 unchanged, no rebuild needed
- Editing `src/v2/*.dag` (compiler source): automatic regeneration via `gunbc-dev build`
- Editing `dsl/extdeps/languages/*/emit.dag` (emission rules): stage0 unchanged unless the compiler imports the file
- Adding a new target language: stage0 unchanged (new language = new extdep data)

---

## Critical Path

```
M1 (every .dag compiles)
 └→ M2 (working Rust codegen)
     ├→ M3 (test generation)           ← parallel with M4
     ├→ M4 (L1 = 0, zero type names)
     │   ├─ Lane 1 Tier 2.5: algebra fidelity ──┐
     │   └─ Lane 1 Tier 3: declaration-driven ──┤
     │                                           │ parallel
     └→ E0c (TypeRendering boundary) ───────────┤
         └→ M5-early: coercion reads inhabitant ─┘
              data via TypeRendering (starts when
              E0c + Lane 1 Tier 2.5 land)
                 └→ M5-full (language plugin extraction)
                     └→ M6 (parse-emit symmetry)
                         └→ M7 (dissolve structural bridges)
```

**Coercion implementation parallelism:** The coercion design is complete
([docs/coercion-design.md](docs/coercion-design.md)). Implementation
does not wait for M4 to finish — it starts as soon as E0c's
`TypeRendering` boundary lands and M4 Lane 1 Tier 2.5 corrects
algebra fidelity. The coercion engine's core loop (`build_type_rendering`
reading `InhabitantDecl` data from `.dag` files) is E0c's structural
fix with coercion data as the input. Full language plugin extraction
(M5-full) still needs M4 for identity dissolution, but the
fail-closed coercion contract and algebra-keyed type rendering can
land earlier.

---

## Design Direction: .dag Model Convergence

Post-bootstrap priority. The .dag model must converge to a minimal,
non-overlapping set of files where each concept traces to an external
authority (spec, standard, Wikipedia article).

**Current violations:**

| Concept | Duplicated in | Authority | Fix |
|---------|--------------|-----------|-----|
| BinOp / BinOpKind | `std/syntax.dag`, `00_core.dag` | Ring theory (arithmetic), total order (comparison), Boolean algebra (logic) | Unify; dissolve into `std/algebra.dag` operations |
| LiteralKind / LiteralValue | `std/syntax.dag`, `00_core.dag` | Grammar (keyword literals) vs IR (all literal forms) | Keep both — different concepts. LiteralKind = grammar subset |
| ItemForm, OperatorSpec, SyntaxSpec | `std/syntax.dag`, `languages.dag` | Language grammar (BNF) | **FIXED**: `languages.dag` imports from `std.syntax` |
| NullCoalesce | `00_core.dag` as BinOpKind | Language design choice | Stays in syntax — not algebra |

**Principle:** foundational `.dag` files (algebra, syntax, types) should
be referenceable to external authorities — specs, standards, Wikipedia.
At this level, concepts should be standard and agreed-upon. Higher up,
users have their own domain models (boutique/application-level) that
interact with the standard language infrastructure. The boundary matters:
if a concept belongs to a standard, it should trace to one. If it's
user-owned domain logic, it lives in user `.dag` files.

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
- [x] Mutual recursion detection — SCC analysis now fail-closes
  indirect recursion, accepts bounded mutual descent, and keeps
  helper-into-cycle callers out of the violation set. Remaining work:
  richer SCC witnesses for parser/block/cache roots (ratchet 313)

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
  Diagnostics wired into pipeline. Ownership violations promoted
  to errors (`OwnershipViolation`); no warning severity remains.

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
- [ ] ValueContext `{ is_constant, has_fn_fields }` precomputed in
  EmitGraphInfo (type defined in `04_emit_info.dag` but not yet a
  field on `EmitGraphInfo`; `has_fn_fields` computed locally in
  `emit_struct_from_children` instead of from the boundary)
- [x] `fielded_variants` precomputed for structural variant-has-fields
- [x] `has_fn_fields` → skip `PartialEq`/`Debug` derives for
  algebra types (working locally in `emit_struct_from_children`;
  not yet sourced from `EmitGraphInfo` precomputation)
- [ ] ValueContext on EmitGraphInfo end-to-end: add field, precompute
  in `build_emit_graph_info`, read in `emit_struct_from_children`
  (E0b invariant theme — separate branch per queue discipline)
- [ ] Adding SPICE/English targets requires only ValueContext ×
  LanguageSpec data, no emission-side debugging
- [ ] `rc_types` authority derived from ValueContext (is_constant →
  no wrap) instead of heuristic type_summary scan

*Type rendering boundary (E0c — resolution→emit type parameterization):*

The resolution→emit boundary doesn't carry type parameterization for
resolved generic types. `emit_node_type_rc` dispatches on structural
shape (connective, children count, params count) which is ambiguous —
a named Conj could be a struct definition, a resolved alias, or a
self-referential field type. Emit compensates with name-based fallbacks
that silently produce wrong output for any type not in a hardcoded list.

Evidence: `FreeMonoid<T>` field `empty: FreeMonoid<T>` emits as
`Rc<FreeMonoid>` (missing `<T>`). Resolution expands the alias to a
structural Conj, stripping type params. Container templates exist
(`"free_monoid": "Vec<{0}>"`) but dispatch never reaches them. This
class of bug would silently affect every new backend.

Six escape hatches in the type rendering pipeline:

| Escape hatch | What it fabricates | Structural fix |
|---|---|---|
| `emit_node_type_conj_rc` named catch-all | Bare type name for generic Conj (e.g. `FreeMonoid` without `<T>`) | `TypeRendering` descriptor — Conj nodes carry rendering intent |
| `emit_node_type_leaf_rc` bare name | Unrecognized type name emitted literally | Fail-closed: `compile_error!` for types without rendering annotation |
| `emit_primitive_type` pass-through | Any name not in type map emitted as-is | Exhaustive type map or fail-closed on miss |
| `rt_type` → `unit_type` on inference failure | `()` for unresolved field types | Fail-closed: emit refuses error-typed fields |
| `_` placeholders in bare containers | `Vec<_>` / `HashMap<_, _>` (invalid in struct fields) | Complete type params from resolution, not placeholders |
| No type-ref vs type-def distinction | Resolution-expanded Conj treated as type reference | `TypeRendering` or nominal references for field types |

Proposed fix: `TypeRendering` — a .dag struct with named edges and
bits, precomputed at the resolution→emit boundary. Shape is emergent
from which edges are populated (no tag enum). Named edges prevent
positional fabrication — you can't confuse key with element.

```
type TypeRendering {
  type_name: String                   // identity
  element: TypeRendering?             // List<THIS>, Set<THIS>
  key: TypeRendering?                 // Map<THIS, V>
  value: TypeRendering?               // Map<K, THIS>
  params: List<TypeRendering>         // fn(THESE, ...) -> ...
  return_type: TypeRendering?         // fn(...) -> THIS
  inner: TypeRendering?               // Optional<THIS>, Refined<THIS>
  generic_args: List<TypeRendering>   // FreeMonoid<THIS>
  shared: Bool                        // needs Rc/pointer/GC
  boxed: Bool                         // recursive indirection
  is_tuple: Bool                      // structural tuple
  is_error: Bool                      // inference failure — fail-closed
  error_label: String                 // diagnostic context
}
```

`build_type_rendering(n: Node, info: EmitGraphInfo) -> TypeRendering`
is a pure function called at each type reference site. `render_type`
matches on which edges exist and applies LanguageSpec templates:

- `params` non-empty → callable: `emit_callable_type(params, return_type, target)`
- `key` set → keyed container: `emit_map_type(key, value, target)`
- `element` set → container: `emit_container(kind, element, target)`
- `inner` set → wrapper: `emit_container("optional", inner, target)`
- else → leaf/named: `target_primitive_type(type_name, target)`
- then apply bits: `shared` → `wrap_shared_type`, `boxed` → `wrap_box`

No Node inspection after `build_type_rendering`. No name-based
dispatch in `render_type`. No `rc_types` map. The boundary is
sufficient. Adding a backend means adding LanguageSpec data, not
emission logic.

**Design direction (2026-04-01):** TypeRendering is a stepping stone.
The final architecture is coercion-based emission: each target
language declares its type algebra inhabitants in .dag extdeps (e.g.,
Rust's `Vec<T>` inhabits FreeMonoid, `HashMap<K,V>` inhabits
PartialFunction, `i64` inhabits Word64). The compiler sidecasts from
.dag structural types to target types via algebraic identity —
mechanical, no heuristics. TypeRendering's named edges map directly to
the algebraic relationships that coercion will formalize. When M5's
coercion engine lands, TypeRendering dissolves into it.

Design doc: `docs/e0c-type-rendering.md` (on `fast-hen-341` worktree).

**Coercion data as input:** `build_type_rendering` is the implementation
boundary where the coercion design lands. Instead of dispatching on
node shape heuristics, it reads:
- `TypeCheckpoint` data from language `types.dag` files (primitives)
- `InhabitantDecl` data from language `types.dag` files (algebra containers)
- `CallableRepr` data (callable syntax)
- Structural recursion for products/coproducts (no data needed)
- Cardinality annotations for optionals (binding-site `?`)

The shared schema lives in `std/coercion.dag`; per-language instances
in `extdeps/languages/{rust,python,go}/types.dag`. See
[docs/coercion-design.md](docs/coercion-design.md) for the full
algorithm (Appendix A walks every case end-to-end). The fail-closed
contract: if no checkpoint or inhabitant is declared for a type, emit
produces a diagnostic error — not a bare name or placeholder.

Acceptance:
- [ ] `TypeRendering` struct defined in `04_emit_info.dag`
- [ ] `build_type_rendering(n: Node, info: EmitGraphInfo) -> TypeRendering`
  precomputed for every field type and function return type
- [ ] `render_type(tr: TypeRendering, target: RenderTarget) -> String`
  replaces `emit_node_type_rc` — matches on edges × LanguageSpec
- [x] `TypeRendering` struct defined in `04_emit_info.dag` with named
  edges (element, key, value, params, return_type, inner, generic_args)
  and structural flags (shared, boxed, is_tuple, is_error)
- [x] `build_type_rendering` + `render_type` implemented in stage0 Rust
- [x] Type-position call sites replaced with TypeRendering path
- [x] Always-annotate let bindings (type proof at every binding site)
- [ ] Delete `emit_node_type_rc` / `emit_node_type_leaf_rc` /
  `emit_node_type_conj_rc` / `emit_node_type_disj_rc` (old path)
- [ ] `build_rc_types` eliminated — sharing authority in TypeRendering
- [ ] `emit_primitive_type` fail-closed (no pass-through on miss)
- [ ] TypeRendering dissolves into coercion engine (M5): build_type_rendering
  reads `TypeCheckpoint` / `InhabitantDecl` from `.dag` declarations
- [ ] Adding SPICE/English target requires zero changes to type rendering

*Bootstrap:*
- [x] Bootstrap A: front-end/bootstrap diagnostic gates back to a trustworthy green baseline
- [x] `dag/syntax.dag` included in bootstrap (OOM resolved by FF-8)
- [ ] Bootstrap B: stage0→stage1 emitted-Rust gate back under ratchet
- [ ] Bootstrap C: regenerate stage0 with `regenerate-stage0.sh`
- [ ] Bootstrap D: owned bootstrap entrypoint in repo
- [ ] CI-verified regeneration (regenerate + diff = empty)

*Boundary sufficiency / zero guess paths (M2 hardening):*

Gate stronger than "Bootstrap B = 0": no correctness-affecting fallback
remains on the bootstrap-critical path. The resolution→emit boundary
must carry enough structure for emit to be a pure translation — every
place emit guesses from names or shape is a place a new backend can
silently go wrong.

Three blocker classes:

1. **Fabricated parameterization** — parameterized types reaching infer
   without bound children. `algebra_child_or_placeholder` and
   `map_key_type_in_env` are fail-closed (return `error_type`/`none`)
   but should be deleted behind a normalization/resolve gate. Bare
   `Map` without `<K,V>` is the canonical case.
   - [x] Fallbacks converted to fail-closed (`error_type` not `string_type`)
   - [ ] Incomplete parameterized types rejected at normalization, not infer
   - [ ] `algebra_child_or_placeholder` error_type fallback deleted

2. **Inference propagation** — expected types not flowing far enough.
   `resolve_builtin_call_type` → `unit_type`, fold accumulators
   under-resolved, higher-order method templates collapse callable
   structure into `ReceiverSelf`.
   - [x] `expected` parameter threaded to `infer_expr` (41 sites)
   - [x] ExprLambda uses `expected` for param typing
   - [ ] Thread `expected` to all formal params, not just callable ones
   - [ ] Refine fold accumulators structurally via `is_fully_resolved`
   - [ ] Model higher-order signatures explicitly for `sort_by`/`fold`

3. **Structural ownership and identity** — variant constructors must
   use structural resolved facts, not name-based stand-ins. Variant
   suffix scanning is M2 correctness with M4 deletion trigger: fix
   now by carrying explicit owner facts, let M4 identity dissolution
   remove remaining surface area.
   - [x] Variant lookup is structural (not suffix scanning)
   - [x] `emit_field_value_with_context` Rc-wraps record fields correctly
   - [ ] Explicit parent-enum ownership facts through resolve/infer/emit

Acceptance: no fabricated type args for parameterized types, no
generic/wrong fallback return types when extraction fails, no
suffix/name scans to recover ownership, no raw-node guessing in type
rendering once E0c lands. Fallback count promoted into CI alongside
existing emitted-Rust/bootstrap fixed-point gates.

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
- [x] Move `kernel_algebra_profile` to `dsl/std/algebra.dag` data
- [x] Move `is_kernel_type` / `is_container_type` predicate lists
  to `dsl/std/types.dag` data
- [x] Move `AlgebraProfile`, `AlgebraTypeTemplate`, `AlgebraFieldTemplate`
  types and all 6 template data tables to `dsl/std/algebra.dag`
- [x] `00_core.dag` re-imports from `std.types` for backward compat
- [x] `04_types.dag` imports from `std.algebra`
- [x] Convert per-profile field builders to `.dag` functions
  (`algebra_templates_for_profile` moved to `std/algebra.dag`)

*Tier 2 — factor `enrich_kernel_type` (modest compiler change):*
- [x] `enrich_kernel_type` calls `.dag` function in `std/algebra.dag`
  (`algebra_templates_for_profile` moved to `std/algebra.dag`)
- [x] Delete `intrinsic_method_index()` /
  `runtime_bridge_method_index()` — deleted 2026-03-28; 48 string
  branches replaced by algebra registry (enrich_kernel_type) +
  Tier 0 lookup_structural_method. See `04_method.dag` tombstones.
- [x] ~60 string branches → structural algebra queries — 48 classification
  branches deleted. Remaining ~12 sites read `method_def.name` from the
  structural algebra field node (emit rendering, inference refinement,
  complexity cost shape). These are structural-authority reads, not raw
  string classification.

*Tier 2.5 — algebra bridge fidelity (no new infra, modeling only):*

Informed by the coercion design ([docs/coercion-design.md](docs/coercion-design.md)):
algebra inhabitants must be correct before `build_type_rendering` can
read them. This tier is a prerequisite for early coercion implementation.

- [x] Fix `Set`/`NonEmptySet` profile: `FreeMonoidCollectionProfile`
  → `BooleanAlgebraCollectionProfile`. Added `BooleanAlgebraCollectionProfile`
  variant, `boolean_algebra_collection_templates()`, and updated
  `KERNEL_ALGEBRA_PROFILE` for `Set`/`NonEmptySet` in both `.dag` and stage0 Rust.
- [x] Fix carrier-changing type loss in `free_monoid_collection_templates`:
  `map`/`flat_map` return_type changed from `ReceiverSelf` to
  `ReceiverCollectionOf { element: NamedTemplate { name: "MappedElement" } }`;
  `fold` param_types changed to `[NamedTemplate { name: "FoldAccumulator" }]`
  and return_type to `NamedTemplate { name: "FoldAccumulator" }`.
  Same fix applied to `boolean_algebra_collection_templates`.
- [ ] Same issue in `partial_function_templates`: parallel authority for
  `PartialFunction` operations including emitter-only alias `emit_map_has`
  that doesn't exist on the carrier algebra.
- [ ] Add `CallableOf` variant to `AlgebraTypeTemplate` so `map`/`flat_map`/
  `fold` param_types can express their callback shape (`fn(T) -> U`,
  `fn(Acc, T) -> Acc`) instead of relying on downstream `refine_collection_
  result_type`. Required for full modeling faithfulness.
- [x] Delete `is_bridge_placeholder_type_name` in `04_types.dag` — replaced
  hardcoded name checks with structural detection: `collect_named_templates`
  walks AlgebraTypeTemplate trees for NamedTemplate names,
  `bridge_placeholder_type_names` combines type parameter names (T, K, V)
  with non-concrete NamedTemplate names from all algebra profiles.
  `is_bridge_placeholder_type_name` now delegates to the structural set.
- [ ] Derive T/K/V type parameter names from algebra type declarations
  instead of hardcoding. Requires accessor on algebra profile data.

*Tier 3 — full structural algebra (requires FF-9):*
- [ ] FF-9: import-driven source resolution (compiler discovers
  modules transitively from source roots)
- [ ] Compiler reads type declarations + algebra edges from `.dag`
  at resolve time
- [ ] Replace template-era higher-order collection placeholders with
  function-typed algebra witnesses from `std/algebra.dag`
- [ ] Derive kernel/container identity from type declarations
  themselves rather than from `kernel_type_set`/`container_type_set`
  name maps — compiler reads structure, not proxy strings
- [ ] Kernel types as algebraic compositions loaded from `std/`
- [ ] 21 type constructor sites → 0
- [x] Type-name comparisons → 0
- [ ] CollectionKind bridge dissolves when method algebras land

Files: `04_types.dag`, `00_core.dag`, `04_lookup.dag`,
`dsl/std/algebra.dag`, `dsl/std/types.dag`, `compile.dag`

#### Lane 2: D6 + emit + resolve (Node.name deletion)

**Status:** B3 (emit rendering) + B4 (resolve structural identity)
complete. Lane 1 Tier 1 landed (algebra/kernel/container data moved
to `dsl/std/`). D6 (constructor/accessor cleanup) is next — mechanical
work to update `make_*` helpers, drop `name:` from Node
constructions, and delete the field.
Note: final `Node.name` deletion depends on Lane 1 Tier 2+ landing
structural identity for synthetic nodes.

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
- [x] Add `_at` variants for all expression/wrapper node name
  accessors: `expr_var_name_at`, `expr_call_func_at`,
  `expr_method_name_at`, `let_binding_name_at`,
  `field_access_field_at`, `foreach_variable_at`,
  `record_lit_type_name_at`, `field_init_node_name_at`,
  `arg_name_at`, `param_node_name_at`
- [x] Migrate 9 Rust emitter rendering sites to `_at` variants
- [ ] Migrate remaining emit sites (Python ~5, Go ~5, shared ~5)
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

*D6 `name:` usage audit (2026-03-31):*

Per-file Node construction counts and classification:

| File | Constructions | Display | Semantic | Synthetic |
|------|:---:|:---:|:---:|:---:|
| `02_parse.dag` | ~54 | ~5 | ~44 | ~5 |
| `04_infer.dag` | ~20 | 0 | ~12 | ~8 |
| `00_core.dag` | ~28 | ~5 | ~10 | ~13 |
| `04_resolve.dag` | ~13 | 0 | ~13 | 0 |
| `04_types.dag` | ~10 | 0 | ~8 | ~2 |
| Other (`04_patterns`, `04_method`, `04_service`, `05_emit_rust`) | ~5 | 0 | ~5 | 0 |
| **Total** | **~130** | **~10** | **~92** | **~28** |

Blocking semantic-identity `.name` reads (must be structural before
`name:` can drop):
- Field/variant/method lookup: `filter(c => c.name == field_name)`
  in `04_lookup.dag`, `04_types.dag`, `04_patterns.dag`
- Type equality: `left.name == right.name` in `04_types.dag`
- Resolve substitution: `map_get(slot_bindings, n.name)` in `04_resolve.dag`
- Module/import graph: `module.name`, `import.name` in `03_resolve.dag`
- Closed tags: `"Refined"`, `"Callable"`, `"Tuple"`, `"Map"` checks
- Kernel identity: `is_kernel_type(name: n.name)` (6 sites)
- Expression identity: `expr_call_func`, `expr_method_name` via `.name`

Display-only sites (~10) are safe to drop now via `authored_name_at`.
Synthetic sites (~28) need Lane 1 Tier 2+ (declaration-backed identity).
Semantic sites (~92) need structural identity infrastructure (D6 blocker).

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
- [ ] Mutual-recursion cycle errors: `complexity.dag:1579` returns
  only `violations`, omitting the cycle-error diagnostics that
  `detect_mutual_recursion_names` previously supplied. Verify that
  mutual-recursion cycles produce fail-closed diagnostics in the
  pipeline output, not silent omission.
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

**Status:** Design complete. Early implementation starts parallel with
M4 (see critical path). Full extraction depends on M2, M4.
**Design:** [docs/coercion-design.md](docs/coercion-design.md) — shared
schema in `std/coercion.dag`, per-language data in
`extdeps/languages/{rust,python,go}/types.dag`.
**Gate:** Zero `match render_target` branches. Zero language mentions in
`src/v2/*.dag`. LintModel validates every emitted file.

*M5-early: coercion via TypeRendering (parallel with M4, needs E0c):*

These items can land as soon as E0c's `TypeRendering` boundary exists
and M4 Lane 1 Tier 2.5 corrects algebra profiles. They do not require
M4 completion or full language plugin extraction.

- [ ] `build_type_rendering` reads `TypeCheckpoint` data for primitives
- [ ] `build_type_rendering` reads `InhabitantDecl` data for algebra
  containers (FreeMonoid → Vec, PartialFunction → HashMap, etc.)
- [ ] `build_type_rendering` reads `CallableRepr` for callable types
- [ ] Fail-closed contract: missing checkpoint/inhabitant → diagnostic
  error (not bare name or placeholder)
- [ ] Algebra law test generation: for each `InhabitantDecl`, compile
  law predicates from `std/algebra.dag` to target-language tests
- [ ] Coercion refusal tests: for every `.dag` type without a declared
  inhabitant in target X, verify the compiler produces a diagnostic

*M5-full: language plugin extraction (needs M4 for identity):*

- [ ] `05_emit.dag` walks typed graph, invokes language-declared
  coercion
- [ ] Delete `05_emit_rust.dag` (4,121 lines) →
  `dsl/extdeps/languages/rust/`
- [ ] Delete `05_emit_python.dag` (1,349 lines) →
  `dsl/extdeps/languages/python/`
- [ ] Delete `05_emit_go.dag` (1,387 lines) →
  `dsl/extdeps/languages/go/`
- [ ] Delete `runtime_rust.dag` → `rust/runtime.dag` extdep

*Type coercion via algebra inhabitants (dissolves E0c TypeRendering):*

Type rendering is coercion: mapping .dag structural types to target
language types via algebraic identity (sidecast). Each language
declares which of its types inhabit which algebras:

```
// dsl/extdeps/languages/rust/types.dag
data type_inhabitants: Map<String, String> = {
  "Word64": "i64",
  "CharSequence": "String",
  "FreeMonoid": "Vec<{0}>",
  "PartialFunction": "HashMap<{0}, {1}>",
  "BooleanAlgebra": "HashSet<{0}>",
  "Bit": "bool"
}
data sharing_strategy = "Rc<{0}>"  // Rust has move semantics
```

The compiler walks the .dag type composition tree, finds the first
level the language declares an inhabitant for (the "checkpoint"),
and sidecasts. Languages that don't declare an inhabitant for an
algebra get a compile error — no silent fallback.

When this lands, TypeRendering (E0c) dissolves: its named edges
(`element`, `key`, `value`, `params`, `return_type`) become the
algebraic relationships that coercion formalizes. `build_type_rendering`
becomes `coerce_type`. `render_type` becomes template application.

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
- [x] Merge `rt_functions: Map<String, Bool>` and
  `rt_bridge_function_names: Map<String, String>` in `rust/emit.dag`
  into a single `RuntimeFunction { name: String, bridge_name: String,
  passes_by_ref: Bool }` list — one concept, one authority
  (backward-compat maps preserved; helpers `is_rt_function`,
  `rt_bridge_name`, `rt_passes_by_ref` added; downstream migration
  to unified helpers is follow-up)

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

### Bounded iteration: one concept, many surfaces

Every loop in .dag — `while`, `for`, recursive functions, mutual
recursion — is surface sugar over exactly three DAG primitives.
The surfaces exist for developer UX. The primitives exist for the
compiler. Same principle as variadic: templates, generics, and
variable-argument functions all desugar to the same concept.

**The three primitives** (from `std/iteration.dag`):

| Primitive | Bound | DAG representation |
|-----------|-------|-------------------|
| `fold(collection, init, f)` | \|collection\| | Bounded traversal of a finite structure |
| `descend(tree, f)` | \|tree\| | Bottom-up catamorphism over an inductive type |
| `repeat(N, init, f)` | N (explicit) | Counted iteration, N up to 2^63 - 1 |

Every iteration in the language collapses to one of these. No
fourth primitive. No special cases. The cost algebra has one rule
per primitive and composition is closed.

**Surface sugar → primitive mapping:**

| What the developer writes | Collapses to | Bound |
|--------------------------|-------------|-------|
| `for x in items { body }` | `fold(items, init, f)` | \|items\| |
| `items \|> map(f)` | `fold(items, init: [], f: ...)` | \|items\| |
| `while cond { body }` | `repeat(max_int, init, f)` with early exit | 2^63 |
| `while true { body }` | `repeat(max_int, init, f)` | 2^63 |
| `fn walk(e: Expr) { match e { ... walk(child) ... } }` | `descend(e, f)` | \|tree\| |
| `fn parse(tokens, pos) { ... parse(tokens, pos+1) ... }` | `fold(tokens, init, f)` | \|tokens\| |
| Mutual recursion A↔B on children | `descend` over SCC | \|tree\| |
| `fn count(n) { ... count(n-1) ... }` | `repeat(n, init, f)` | n |

**The architectural rule:** the DAG never represents "a while loop"
or "a recursive function" or "mutual recursion" as distinct concepts.
It represents `fold`, `descend`, or `repeat`. The surface syntax
determines UX. The primitive determines cost. Adding a new surface
(e.g. `loop { }`, `do { } while`, generators) never adds a new
primitive — it adds a new desugaring to one of the three.

**`while(true)` is decidable.** `while(true)` desugars to
`repeat(bound: max_int, ...)`. At one iteration per nanosecond,
max_int runs for 292 years. The developer writes "loop forever."
The compiler sees "bounded iteration." The cost algebra produces
`O(max_int × per_step)` — finite. The distinction is meaningless
to the developer and meaningful to the compiler.

**Recursive syntax is sugar.** Developers write recursive functions
for readability. The compiler verifies the recursion is bounded and
lowers to a primitive:

1. **Match on a recursive union, recurse on variant fields** → `descend`.
   The compiler knows which fields are recursive from the type
   definition. Verification is mechanical: self-call argument is a
   field the type marks as recursive.

2. **Recurse with advancing position** → `fold`. The compiler
   verifies the position argument increases monotonically (or the
   collection shrinks). Bound is the collection size.

3. **Recurse with arithmetic descent** (n-1, n/2) → `repeat(n, ...)`.
   Bound is the initial value.

4. **Recurse with unchanged argument** → **compilation error**. No
   primitive accepts unchanged arguments. The function is genuinely
   unbounded and cannot be expressed in the language.

**Mutual recursion uses SCC analysis.** Functions that call each
other indirectly (A→B→A) form a strongly connected component. The
compiler verifies the SCC has a shared decreasing measure:
- Parser SCC (parse_type_expr ↔ parse_callable_type_expr): token
  position advances across the cycle → `fold` over tokens
- Emit SCC (emit_typed_expr ↔ emit_shared_expr): children are
  structurally smaller → `descend` over expression tree
- Complexity SCC (cost_of_expr ↔ get_or_compute_summary): cache
  placeholder breaks the cycle → `fold` over function entries

If no shared decreasing measure exists, the SCC is a compilation
error — same as case 4 above.

**Current state (2026-03-31):** 53 root functions → 313 complexity
violations, ratcheted. This branch landed the first real proof
infrastructure:
- direct recursion is fail-closed on the actual measured parameter
- SCC ownership is explicit, so callers into a cycle do not inherit the
  cycle's violation
- parser progress is parse-owned via typed helper identities
- recursive-field facts are structural witnesses, not concatenated keys

The remaining blockers are no longer "mutual recursion is missing";
they are specific witness gaps:
(a) parser SCC progress for the `parse_type_expr` family,
(b) block/tree-walker SCC descent for emit/infer,
(c) type-normalization self-recursion, and
(d) a few cache/dispatcher SCCs. Those are the path from 313 → 0.

#### Implementation plan

Three work items, in dependency order. Each is independently
testable — the ratchet drops after each one lands.

**Critical parallel cleanup (authority / bridge dissolution).**

Recent mainline work has a clear pattern: declaration data moved out of
the compiler, semantic lookups moved from source-text/name recovery to
structural accessors, and complexity class authority moved from strings
to `CostExpr`. The recursion/progress proof path should follow the same
rule before we harden I1/I2 further.

- [x] Replace `recursive_variant_fields: Map<String, Bool>` with a
  structural recursion witness owned by inference/env (variant-local
  witness or direct edge), and thread it through imported/unresolved/final
  envs so there is one authority
- [x] Replace `ParserWitnessCall { callee: String }` with a typed parser
  helper identity so parser progress is not name-driven
- [ ] Move structural descent proof ownership out of `complexity.dag`:
  complexity should consume a resolved descent witness, not recover it
  from encoded keys, raw names, or match-shape heuristics
- [x] Acceptance: recursion/progress proofs no longer depend on
  `split(\"::\")`, concatenated field keys, or raw parser-helper-name
  strings
- [x] Acceptance: stage0/source parity stays green after the witness
  refactor
- [ ] Acceptance: fixed-point regeneration stays green after the witness
  refactor

**Next branch after PR #270 merge (313 → 0 path).**

- [ ] Parser SCC witness completion: prove token/cursor progress across
  the `parse_type_expr` / `parse_callable_type_expr` / block-expression
  SCC instead of falling back to generic non-descending mutual recursion
- [ ] Block/tree-walker SCC witness completion: add one shared
  statement-tail / child-node descent proof and apply it to
  `emit_rust_block_stmts`, `emit_py_block_stmts`, `emit_go_block_stmts`,
  `infer_block_stmts`, `emit_*_tco_if`, and `emit_node_type_rc`
- [ ] Type-normalization recursion proof: discharge
  `normalize_access_type_node` and its downstream family
  (`node_type_shape`, `node_type_compatible`, `node_type_equals`,
  `node_type_deps`) with a fail-closed structural measure.
  **Progress:** `expr_descending_witness_source` extended to recognize
  `ExprFieldAccess` chains and single-element extraction methods (first,
  last) as structural descent. `skip`/`take` excluded — not
  unconditionally shrinking (`skip(0)`, `take(length(xs))`). Remaining: propagate descent
  witness through if/match optional extraction (let x = if ... {
  param.children |> first } else { none }; match x { Some => ... }).
- [ ] Cache/frontier SCC proof: finish `resolve_callback_cost` and any
  remaining finite-key SCCs with a real frontier witness rather than a
  placeholder cycle explanation
- [ ] Parser contract tests: pin `range(...)` integer-literal behavior
  with one negative-literal test if supported and one explicit rejection
  test for computed expressions
- [ ] Re-lock regeneration only after the root witnesses move:
  `./scripts/regenerate-stage0.sh && git diff --exit-code src/v2/stage0/`
- [ ] Boundary cleanup follow-up: replace raw `.name` equality in
  `collect_item_recursive_variant_fields` with resolved structural
  recursive-type identity
- [ ] Boundary cleanup follow-up: stop copying module recursion facts
  into every `FuncEntry`; keep one module-owned authority for complexity
- [ ] Emitter cleanup follow-up: route Python `VariantPattern` emission
  through `emit_py_variant_pattern` instead of keeping a parallel inline
  implementation

**I1: `descend` primitive for recursive unions.**

The compiler already knows which types are recursive
(`recursive_type_set` in `TypeEnv`). The missing piece is knowing
which *fields* of each variant are the recursive positions.

What exists:
- `recursive_type_set` tracks recursive type names (in `04_env.dag`)
- `classify_recursion_pattern` proves direct self-recursion is bounded
  (in `complexity.dag`, checks `is_structural_descent` and
  `has_arithmetic_descent`)
- `CostSum` in the cost algebra already represents bounded iteration
- `iteration.dag` declares `descend` conceptually but no compiler
  implementation exists

What's needed:
1. At resolve time, for each recursive union type, record which
   variant fields are recursive positions (field type == parent type).
   This is a small addition to `TypeEnv` or `TypeBinding` — a
   `recursive_fields: Map<String, List<String>>` keyed by variant name.
2. In `classify_recursion_pattern`, when a function matches on a
   recursive union and self-calls receive recursive fields: classify
   as `LinearRecursion` (same as today's `is_structural_descent`,
   but using type-definition knowledge instead of heuristic child
   analysis).
3. The cost algebra already produces `CostSum` for linear recursion.
   No cost algebra changes needed.

Blocked on: nothing. All prerequisite infrastructure exists.

Test: `strict_complexity_violation_count` drops. The 53 manual
`CostExpr` traversals in `complexity.dag` and the `ExprData` tree
walkers in emit/infer become provably bounded. Expected reduction:
tree-walker root functions (emit_typed_expr, infer_expr,
simplify_cost, format_cost_inner, etc.) resolve → ~200 violations
eliminated.

**I2: SCC analysis for mutual recursion.**

What exists:
- SCC construction and ownership filtering exist in `complexity.dag`
- Mutual recursion fail-closes when no shared decreasing measure is
  found, bounded arithmetic mutual descent is accepted, and callers into
  the SCC are excluded from the cycle's violation set
- Parser helper progress is modeled in `02_parse.dag` and consumed by
  complexity, rather than guessed from helper-name strings

What's needed:
1. For each SCC with >1 function, find the shared decreasing measure.
   Three remaining cases:
   - All functions pass children of a recursive union → `descend`
     (tree walkers / block emitters / infer walkers)
   - All functions advance a position in a collection → `fold`
     (parsers like parse_type_expr ↔ parse_callable_type_expr)
   - Functions thread through a cache with finite keys → `fold`
     over the key set (complexity SCC: cost_of_expr ↔
     get_or_compute_summary)
2. Replace the remaining placeholder-cycle `CostUnknown` with the proven
   `CostSum` for the SCC's bound.
3. Once the shared witnesses are authoritative, delete any leftover
   fallback logic that compensates for missing SCC proofs.

Blocked on: a stronger `descend` witness for recursive unions and a
shared parser-progress witness across parser SCCs.

Test: parser + block-emitter + cache SCC roots disappear; 313 approaches
0 without suppression.

**I3: `while` surface sugar.**

What's needed:
1. Tokenizer: add `while` keyword.
2. Parser: `while <expr> { <body> }` desugars to
   `ExprForEach` with a synthetic `repeat(max_int)` range, or a
   new `ExprRepeat` node that the rest of the pipeline handles
   like `ExprForEach`. Design choice: reuse existing `ExprForEach`
   (less pipeline change) or add `ExprRepeat` (cleaner separation).
3. Complexity: `repeat(N)` already has cost `CostSum { upper: N }`.
   `while(true)` gets `CostSum { upper: max_int }`.
4. Emit: each target renders its native loop. Rust: `for _ in 0..N`.
   Python: `for _ in range(N)`. Go: `for i := 0; i < N; i++`.

Blocked on: nothing. Independent of I1/I2. Could land first as a
standalone language feature.

Test: `while(true)` compiles, emits working target code, complexity
reports `O(max_int × per_step)`.

**Acceptance criteria (all three landed):**
- Recursive functions on recursive unions lower to `descend`
- Mutual recursion SCCs verified and lowered to `descend`/`fold`
- `while` keyword desugars to `repeat(max_int, ...)`
- `complexity.dag` uses fold/descend instead of 53 manual traversals
- `is_unknown_cost` is one line, not ten
- Complexity gate: 313 → 0 without suppression or ratchet
- Cost algebra: one cost rule per primitive, composition closed

### Bounded by construction — no unbounded expressions

Every expression in `.dag` has a finite cost. This is not validated —
it is structurally impossible to write an unbounded expression. The
language has no primitive for unbounded computation, and composition
of bounded primitives is closed (bounded + bounded = bounded).

**Current state:** recursive syntax is sugar that the compiler lowers
to bounded primitives (`fold`, `descend`, `repeat`). The 311
violations are analyzer gaps — the programs ARE bounded, the analyzer
can't prove it yet. Fixing the analyzer to recognize all descent
patterns resolves these. Once lowering is complete, `CostUnknown`
becomes structurally unreachable — not just validated away, but
impossible to construct.

**Cyclic relations, acyclic values, bounded traversals.** See
INVARIANTS.md §Strict Forward Progress for the full formulation.
Summary: cyclic domains are expressible via acyclic encodings
(adjacency maps). Direct cyclic values are not. Traversals over
cyclic relations must be justified by an explicit finite measure
(|V|, |E|, frontier size).

### Cost comparator — refuse to compile suboptimal code

After every function has a proven tight bound, the compiler can detect
known-suboptimal patterns and refuse to compile them. This is not a
linter — it is a provable statement: "an equivalent implementation
with strictly lower cost exists using the same primitives."

**Known suboptimal patterns (O(n²) → O(n) or O(n log n)):**

| Pattern | Cost | Optimal | Cost | Detection |
|---------|------|---------|------|-----------|
| Membership check in fold accumulator: `acc \|> any(x => x == item)` | O(n²) | `map_get(seen, item) != none` with `Map<T, Bool>` | O(n log n) | fold body scans growing accumulator |
| String concat in loop: `fold(items, init: "", f: (acc, s) => concat(acc, s))` | O(n²) | `join(items, separator)` | O(n) | fold body concats to accumulator string |
| Repeated list append: `fold(items, f: (acc, x) => concat(acc, [x]))` | O(n²) | `list_push(acc, x)` | O(n) | fold body concats single-element list |
| Sort + extract: `sort(list) \|> first` | O(n log n) | `fold(list, min)` | O(n) | sort followed by single-element access |
| Nested find: `list \|> map(x => other \|> find(...))` | O(n×m) | Build index, then lookup | O(n+m) | nested collection scan in map/fold body |
| Loop-invariant computation: `fold(items, f: (acc, x) => let k = expensive_pure(constant) ...)` | ×cost(f) | Hoist before fold | O(1) | pure call with loop-invariant args inside fold |

**Design:** The cost comparator runs after complexity analysis. For
each function with proven cost, it pattern-matches the cost structure
against the suboptimal pattern table. If a match is found AND a
strictly cheaper alternative exists:

1. The alternative must use the same primitives (no new language
   features required)
2. The alternative must produce identical output (semantic
   equivalence)
3. The cost improvement must be strict (O(n²) → O(n), not O(n) →
   O(n) with better constant)

When all three hold, compilation fails with a diagnostic that shows
the suboptimal pattern and the suggested fix. The developer can then
choose the optimal implementation.

**Space-time tradeoff awareness:** O(n²) time + O(1) space is NOT
always worse than O(n) time + O(n) space. The comparator only refuses
when the alternative is strictly better in ALL dimensions, or when
the time improvement dominates (e.g., O(n²) → O(n log n) with same
space). Ambiguous tradeoffs are allowed — the developer makes the
call.

**Example: binary tree search (O(log n))**

```
type Tree<T> = Leaf | Branch { value: T, left: Tree<T>, right: Tree<T> }

fn search(tree: Tree<Int>, target: Int) -> Bool {
  match tree {
    Leaf => false
    Branch { value: v, left: l, right: r } =>
      if target == v { true }
      else if target < v { search(tree: l, target: target) }
      else { search(tree: r, target: target) }
  }
}
// Compiler lowers to: repeat(height(tree), ...) with single-branch selection
// NOT descend(tree, f) — descend is a catamorphism that visits ALL nodes (O(n)).
// BST search follows ONE branch per level — arithmetic descent on tree height.
// Cost: O(height) — which is O(log n) for balanced trees, O(n) for degenerate.
// A refinement type (BalancedTree<T>) would express the O(log n) guarantee.
```

**Example: accumulator scan detection**

```
// REJECTED: O(n²) — fold body scans growing accumulator
fn unique(items: List<String>) -> List<String> {
  items |> fold(init: [], f: (acc, item) =>
    if acc |> any(a => a == item) { acc }
    else { list_push(acc, item) }
  )
}
// Diagnostic: "fold accumulator scanned with `any` — O(n²).
//   Use Map<String, Bool> for O(n log n) membership."

// ACCEPTED: O(n log n) — Map lookup is O(log n)
fn unique(items: List<String>) -> List<String> {
  let result = items |> fold(init: { seen: empty_map(), out: [] }, f: (acc, item) =>
    if map_get(acc.seen, item) != none { acc }
    else { { seen: map_insert(acc.seen, item, true), out: list_push(acc.out, item) } }
  )
  result.out
}
```

### Cost algebra extensions

### Mathematical foundations

The cost algebra is a **semiring** (C, ⊕, ⊗, 0, 1) where:
- C = the set of cost expressions (symbolic, not numeric)
- ⊕ = CostAdd (sequential composition: f; g costs f + g)
- ⊗ = CostMul (nested composition: loop of f costs n × f)
- 0 = CostConst(0) (free operation)
- 1 = CostConst(1) (unit-cost operation)
- CostMax = join operation (conditional: if-then-else takes the max branch)

This is a **polynomial cost semiring** over symbolic size variables,
extended with bounded summation (CostSum) and logarithmic terms
(CostLog). The semiring laws guarantee that cost composition is
associative, commutative (for ⊕), and distributes correctly.
Unlike a tropical semiring (which uses min/+), this algebra uses
+/× because we are computing total cost, not shortest paths.

**Current CostExpr variants and their formal semantics:**

| Variant | Formal definition | Function class |
|---------|------------------|----------------|
| `CostConst(c)` | f(n) = c | Θ(1) |
| `CostAdd(f, g)` | f(n) + g(n) | max class of f, g |
| `CostMul(f, g)` | f(n) · g(n) | product of classes |
| `CostMax(f, g)` | max(f(n), g(n)) | max class of f, g |
| `CostSum(i, N, f)` | Σ_{i=0}^{N} f(i) | depends on f: Σ1=N, Σi=N², Σlog=N·log |
| `CostLog(b, n)` | log_b(n) | Θ(log n) |
| `CostUnknown(r)` | ⊥ (bottom) | analysis failure — structurally eliminated |

**Expressible function classes (current):**
- Θ(1), Θ(log n), Θ(n), Θ(n log n), Θ(n²), Θ(n² log n), Θ(n³)
- Arbitrary polynomial-logarithmic: Θ(n^a · log^b n) for constant a, b
- Multi-variable: Θ(n · m), Θ(n · m · log k)

**Planned extensions — new CostExpr variants:**

| Variant | Formal definition | Function class | Use case |
|---------|------------------|----------------|----------|
| `CostPow(n, k)` | n^k for constant k ∈ ℕ | Θ(n^k) | Explicit polynomial degree (matrix multiply Θ(n³), naive sort Θ(n²)) |
| `CostSqrt(n)` | n^(1/2) | Θ(√n) | Trial division, block decomposition, Mo's algorithm O((n+q)√n) |
| `CostExp(b, n)` | b^n for constant b | Θ(b^n) | Detect and REJECT — exponential cost is a compilation error |

**Recurrence resolution:** Recursive cost follows from the bounded
iteration primitives. Each primitive declares a recurrence:

| Primitive | Recurrence | Closed form |
|-----------|-----------|-------------|
| `fold(collection, f)` | T(n) = Σ_{i=1}^{n} f(element_i) | CostSum(i, \|collection\|, cost(f)) |
| `descend(tree, f)` | T(n) = Σ_{children} T(child) + f(node) | CostSum over tree structure |
| `repeat(N, f)` | T = N · f | CostMul(N, cost(f)) |
| Arithmetic descent (n/b) | T(n) = aT(n/b) + f(n) | Master theorem: case 1: Θ(n^{log_b a}) if f ∈ O(n^{log_b a - ε}); case 2: Θ(n^{log_b a} · log n) if f ∈ Θ(n^{log_b a}); case 3: Θ(f(n)) if f ∈ Ω(n^{log_b a + ε}) |

For mutual recursion (SCCs), the shared decreasing measure determines
the recurrence. Parser SCCs (token position advances) → fold over
tokens. Tree-walker SCCs (children shrink) → descend over tree.

**Amortized analysis via potential functions:**

```
CostAmortized { 
  worst_case: CostExpr,      — single-operation worst case
  amortized: CostExpr,       — per-operation amortized cost
  potential: PotentialFn      — Φ: State → ℝ⁺ (potential function)
}

// The amortized cost satisfies:
// â_i = c_i + Φ(s_{i+1}) - Φ(s_i)
// where c_i is actual cost, Φ is potential, s_i is state after op i
// Total actual cost ≤ Σ â_i + Φ(s_0) - Φ(s_n)
```

Use cases: dynamic array append (worst O(n), amortized O(1) with
Φ = 2·size - capacity), splay tree (worst O(n), amortized O(log n)
with Φ = Σ log(subtree_size)).

**Space complexity as a peer dimension:**

The `FunctionSummary` already has `work: CostExpr` and `span: CostExpr`.
Adding `space: CostExpr` makes it a three-dimensional cost vector:

```
type CostVector {
  work: CostExpr       — total operations (time)
  span: CostExpr       — critical path length (parallel time)  
  space: CostExpr      — peak memory usage
}
```

This enables the cost comparator to reason about Pareto optimality:
a program is suboptimal only if another program dominates it in ALL
dimensions. O(n²) time + O(1) space is NOT dominated by O(n) time +
O(n) space — the developer chooses the tradeoff.

**Asymptotic notation — both O and Θ:**

The analyzer should produce Θ (tight) bounds, not just O (upper).
`Conservative` certainty means O but not Θ — valid but not tight.
`Proven` means Θ — exact asymptotic behavior. The target: every
function has a Proven Θ bound. O-only is a modeling deficit.

| Notation | Meaning | Status |
|----------|---------|--------|
| Θ(f) | Tight: grows as f | Target for all functions |
| O(f) | Upper: grows no faster than f | `Conservative` — acceptable interim |
| Ω(f) | Lower: grows no slower than f | Needed for optimality proofs |
| o(f) | Strict upper: grows strictly slower | Needed for suboptimality detection |

The cost comparator needs both O and Ω: to prove g is strictly better
than f, show f ∈ Ω(h) and g ∈ o(h) for some h. Example: f ∈ Θ(n²)
and g ∈ Θ(n log n) — since n log n ∈ o(n²), g strictly dominates f
in time.

**Concrete examples for early implementation:**

1. **Binary tree search** — single-branch arithmetic descent (NOT catamorphism):
   T(n) = T(n/2) + O(1) → Θ(log n) by Master theorem (a=1, b=2, f=O(1))
   Note: `descend(tree, f)` would visit all nodes (Θ(n)). BST search
   selects one branch — lowered to `repeat(height, ...)`, not `descend`.

2. **Merge sort** — descend with merge:
   T(n) = 2T(n/2) + O(n) → Θ(n log n) by Master theorem (a=2, b=2, f=O(n))

3. **Matrix multiply (naive)** — triple nested fold:
   T(n) = n · n · n · O(1) = Θ(n³)

4. **Hash table lookup** — CostConst(1) amortized, CostSum(n) worst:
   Amortized O(1), worst O(n). Potential Φ = load_factor × n.

5. **Accumulator scan detection** — fold with inner scan:
   T(n) = Σ_{i=1}^{n} i = n(n+1)/2 = Θ(n²)
   Optimal: T(n) = Σ_{i=1}^{n} O(log n) = Θ(n log n) with Set

**Unified Sequence (Seq\<T>).** Ordered collections share FreeMonoid
algebra; access pattern determines representation. Mixed access = type
error.

**Space complexity as peer dimension.** `space: CostExpr` peer to `work`
and `span`. Currently `output_size` is unpopulated.

**Computed data declarations.** The `.dag` `data` syntax only supports
literal initializers (maps, lists, records). Computed expressions
(`data x = list |> fold(...)`) are not supported. This prevents
deriving indexed maps from authoritative lists, requiring hand-
maintained parallel data declarations (e.g., `rt_functions` maps
alongside `rt_function_registry`). When the parser gains computed
data declarations, parallel-data violations dissolve.

**Everything is coercion.** Unifying concept: minimal complete
representation in a target domain. Applies at stage boundaries, type
compatibility, and language rendering.

Type rendering is the canonical instance: .dag's `List<Int>` and
Rust's `Vec<i64>` both inhabit FreeMonoid<Word64>. The mapping is a
sidecast — same algebra, different representation. Each language
declares its algebra inhabitants as .dag extdep data. The compiler
sidecasts mechanically. Adding a new language = declaring inhabitants
+ sharing strategy. No emission logic. Fail-loud when no inhabitant
exists: the coercion has no target, the compiler refuses.

The checkpoint model: .dag types compose from primitives (Bit →
BitWord<64> → Int → NonNegativeInt). Each language checkpoints at the
level it cares about. Rust says "I know Int → i64" and skips the
intermediate compositions. SPICE might checkpoint at BitWord<64> →
"wire[63:0]". The compiler walks the composition tree and stops at
the first level the language recognizes.

---

## Verification

| Ratchet | Current | Target | Command |
|---------|---------|--------|---------|
| Self-compile diagnostics | 310 | 0 | `strict_compile_diagnostic_count -- --ignored` (all 310 are indirect-recursion complexity violations) |
| full_dsl_compiles | 0 | 0 | `full_dsl_compiles -- --ignored` |
| L1 type knowledge | 21 | 0 | `scripts/l1-ratchet.sh --check` |
| Complexity violations | 310 | 0 | `strict_complexity_violation_count -- --ignored` (27 root functions × indirect recursion → 310; resolves when fold primitive lands) |
| Emitted Rust errors | 880 | 0 | `bootstrap_stage0_to_stage1 -- --ignored` |
| Bootstrap fixed point | PASSES | PASSES | `bootstrap_fixed_point -- --ignored` |
| Performance | <30s | <30s | `performance_ratchet -- --ignored` |

### CI Gates

| Gate | Command |
|------|---------|
| Clippy | `cargo clippy --workspace -- -D warnings` |
| V2 compiler tests | `cargo test -p v2-compiler-tests` |
| Full DSL compiles | `cargo test -p v2-compiler-tests full_dsl_compiles -- --ignored` |
| Diagnostic ratchet | `cargo test -p v2-compiler-tests strict_compile_diagnostic_count -- --ignored` |
| L1 ratchet | `scripts/l1-ratchet.sh --check` |

### Required Before Merge (Tier 3)

```
scripts/l1-ratchet.sh --check
cargo test -p v2-compiler-tests full_dsl_compiles -- --ignored
cargo test -p v2-compiler-tests strict_compile_diagnostic_count -- --ignored
cargo test -p v2-compiler-tests bootstrap_fixed_point -- --ignored
```
