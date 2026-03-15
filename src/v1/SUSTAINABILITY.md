# Sustainability Ledger

The governing metric for this codebase is **cost of change**: when the
language grows by one type, one expression form, or one transport, how
many files need editing? The sustainable compiler is one where that
number is 1.

This ledger tracks open violations. See `src/v1/README.md` for the invariants.

---

## Deep root: Incomplete compile-time resolution

The compiler doesn't fully resolve information at compile time. Types
are referenced by string names (`TypeId`) instead of embedded structure.
Classification uses string matching instead of structural queries.
Rust code redeclares facts the DSL already defines because it can't
read compiled DSL output.

**Terminal state:** The compiler fully resolves all references at
compile time. Ports embed type structure, not string names.
Classification derives from structure, not name patterns. Rust code
reads from DSL-compiled artifacts, not parallel declarations. Each
pipeline boundary validates that its output is fully resolved.

**v2 eliminates the deep root by design:** types are `TypeExpr` values,
not strings. No TypeRegistry. No deferred resolution.

---

## Open findings

### Branch 1: TypeId is deferred computation (eliminated by v2)

`Port.type_id` is a string that requires a `TypeRegistry` lookup to
resolve into structural information. Three representations of one fact.

**Terminal state:** Self-hosting. v2 types are structural `TypeExpr` values.

**S1:** `register_core_types()` duplicates .dag definitions. 2 places per type.
**S2:** `mock_element_expr` / `try_mock_element_value` enumerate types. ~240 lines.
**S4:** `port.cardinality` caches type info derivable from the type DAG.
**S5:** Emit pipeline uses core-only registry; DSL types invisible → string fallback.
**S13:** Semantic carrier classification by string match. 50+ match arms.

### Branch 3: Open-set enumeration by string

**S47:** Container type classification duplicated across emit functions.
Fix: `ContainerKind` enum at typecheck. (Eliminated by v2.)

### Branch 4: DSL/Rust boundary duplication

**S10:** Container types are compiler built-ins. 4+ places per new container.

### Branch 5: Permissive boundary types

**S30:** Testgen re-derives type info by parsing TypeId strings. Fix: query type DAG. (Partially done.)
**S62:** `[when]` guards on `func` body service calls not lowered into DAG IR.
Guards are silently dropped, making conditional service calls execute unconditionally.
Discovered via IAM preflight incident (2026-03-09); affected code deleted.

### Branch 7: Untyped runtime — accepted debt

The v1 evaluator works but is bounded by:
- **S52:** Parser mutual recursion not covered by TCO. Not a correctness
  issue — bounded by AST depth, tests use `with_parser_stack(16MB)`.
- **Performance:** `Env::from_inputs` clones on every non-self call (partially
  mitigated with `Rc<HashMap>` COW). Map field flattening clones every field.

**Accepted permanent splits** in eval stack machine (all documented with rationale):
- `eval_expr` handles non-sibling calls via re-entrant `evaluate_stack`
- `eval_block_s` (suspendable) / `eval_block_pure` (pure, for standalone match/lambda)
- `eval_match_s` (suspendable) / `eval_match_local` (pure, guards can't use continuations)
- `wrap_value_as_output` Map flattening (structurally necessary for v2 multi-field records)

**Terminal state:** Self-hosting eliminates the evaluator entirely.

### Branch 8: Type-unaware codegen (all die with self-hosting)

fn_codegen compiles .dag function bodies to Rust without type information.
Every decision requiring type info is heuristic.

**S81: fn_codegen emits Rust, not code_ir — CRITICAL.** ~15 Rust-specific
heuristics injected directly into IR: `clone_if_needed()`, `Box::new()`,
`Some()`/`None`, `.as_str()`, `..Default::default()`, `LazyLock`, `Deref`/`*`.

The code_ir layer exists so one compilation produces IR all backends can
render. DAG nodes are **facts** — target-agnostic assertions about computation.
**Rendering facts** (ownership, optional representation, type naming) belong
in backends, not IR. The structural test: can you swap the backend without
changing the IR?

**S76:** `clone_if_needed()` — blind ownership heuristic. ~300 unnecessary clones.
**S77:** `infer_struct_name()` — field-name matching for anonymous records. Wrong on overlapping fields.
**S78:** Materialized types in `std_types_prelude()` — hand-written because v1 can't resolve cross-module imports.
**S79:** Hardcoded cross-module imports in `module_prelude()` — should derive from `import` declarations.

### Standalone

**S8:** C backend discards Map key types. Intentional (C has no native map).
**S38/S39:** Test semantic strength — re-add collection/manifest/output-value assertions. (S38 partial.)
**S72:** `generated_types_are_not_stale` is `#[ignore]`d. Naming bug fixed; test
remains ignored pending full gen-types cleanup.
**S73:** `cargo check --workspace --all-targets` is not a stable hygiene ratchet
because `gunbc-codegen` declares bins under `target/codegen/bin/*/main.rs` (generated out-of-band).

---

## v2 impact classification

**Eliminated by v2 (no v1 fix needed):**
All of Branch 1 (S1, S2, S4, S5, S13), S47, Branch 2 parallels.

**Inherited by v2 (re-implement correctly):**
S34 (callable wiring), S38 (emitted code untested), S44 (shell output
parsing annotation), S45/S46 (provider/transport metadata stamping).

**v1 maintenance only:** S39, S48, S49, Branch 7 evaluator fixes.

---

## V2 self-hosting gap analysis (updated 2026-03-14)

**V2 source:** 7 modules, 7,311 lines, all parse with zero diagnostics.
**Test status:** 59 pass, 0 fail, 2 ignored (OOM + cargo check gate).

**What works today:**
- Module discovery, parsing, type codegen, fn codegen (records, match,
  if/else, for, lambda, string interp, intrinsics, `with()`, `concat()`)
- Recursive type detection, Rust rendering, crate assembly, runtime shims

**Current state: 115 compile errors in generated v2 Rust crate.**
(Down from 2204 initial → 829 → 115.)

| Error | Count | Root cause |
|-------|-------|------------|
| E0308 type mismatches | 59 | String/i64, anon struct names, remaining Option |
| E0382 use-after-move | 26 | Multi-use String fields in match arms (S76) |
| E0609 `.value` on `Option` | 9 | Need `.unwrap()` for `Some{value:x}` access |
| E0425 missing values | 2 | Variable scoping in generated code |
| E0282 type annotation | 1 | Inference gap |
| E0063 missing field | 1 | Struct field mismatch |

**Path forward:**
1. Fix `.value` on `Option<T>` → emit `.unwrap()` (9 errors)
2. Add `.clone()` for multi-use String match bindings (26 errors)
3. Fix anonymous struct naming (8 errors)
4. Remaining type mismatches need deeper type tracking (50 errors)

---

## Heuristic elimination roadmap

Each v1 bootstrap heuristic (S81) maps to a modeling decision that makes
it unnecessary. Ordered by dependency.

### Phase A: Type-aware emission (eliminates S76, S77, S78, S81 bulk)

| Heuristic | What v2 emitter does instead |
|-----------|------------------------------|
| `clone_if_needed` (S76) | Tracks variable liveness. Last use = move, earlier = borrow. |
| `infer_struct_name` (S77) | Typechecker resolves anonymous records to structural type. |
| `Box::new()` wrapping | Checks `TypeExpr` for recursion. Per-backend rendering. |
| `Some()`/`None` injection | `TypeExpr::Optional` in typed IR. Per-backend rendering. |
| `.as_str()` insertion | Rust emitter knows String match context. |
| `..Default::default()` | Emitter has all field types. Complete struct literals. |

### Phase B: Import resolution (eliminates S78, S79)

| Heuristic | What v2 resolver does instead |
|-----------|-------------------------------|
| `std_types_prelude()` (S78) | Types from resolved module graph. No hand-written defs. |
| `module_prelude()` (S79) | Per-module `use` derived from resolved imports. |

### Phase C–F: Further modeling

- **C: Variant disambiguation** — typechecker resolves from context (expected type).
- **D: Optionality** — `TypeExpr::Optional` already modeled. Emitter reads source/target.
- **E: Ownership** — `.dag` has value semantics. Backend decides (Rust: clone/move, C: copy/refcount, Go: GC, Verilog: wire).
- **F: Static data** — `data` defs as `ConstDef { name, type, value }`. Per-backend rendering.

### What dies with self-hosting

- `fn_codegen.rs`, `v2_crate_emit.rs`, `v2_runtime_shim.rs` — entire files
- All heuristic functions: `clone_if_needed()`, `is_option_expr()`,
  `infer_struct_name()`, `std_types_prelude()`, `module_prelude()`
- `synthesize_anonymous_structs()`, `compile_intrinsic_call()` Rust-specific handlers

---

## v1 health guards

Ratchets that freeze known debt so it can't worsen while v2 progresses.

| Guard | Symptom | What it catches |
|-------|---------|-----------------|
| `#[must_use]` on `wire_callable_return_outputs()` | S34 | Ignored `Result` at new call sites |
| `ratchet_fail_open_types` | S23/S35 | New DSL types on ports without `ValueBacking` |
| `ratchet_identity_types_in_core_registry` | pre-existing | New identity/opaque types in registry |
| fidelity classification tests | S3 | `evaluate_fn_body()` regressions |

All ratchets are one-way (lists can only shrink).

---

## Resolved summary (67 findings, 2026-03-10 through 2026-03-14)

| Theme | Findings | Resolution pattern |
|-------|----------|--------------------|
| Structural classification | S11, S12, S14, S15, S22, S44–S46, S48, S49, S67, S68–S70, S75, S80 | String match → structural dispatch, typed registries, explicit intrinsics |
| Fail-closed error handling | S3, S7, S23/S35, S24, S25, S34, S64 | `.ok()`/`let _`/`.unwrap_or(true)` → `Result` propagation, explicit `match` |
| Boundary contracts | S18–S20, S31–S33, S40, S41, S57 | Validation passes, cardinality checks, runtime type enforcement |
| Eval stack machine | S50–S56, S58–S61, S52-EVAL, Eval-8/9, S67-5/6/7 | Stack ordering, TCO, type enforcement, error/control-flow separation |
| Metadata consolidation | S16, S21, S26, S42, S43 | Single-authority registries, stamped classification |
| Code quality | S6, S9, S63, S65, S66, S69, S71, S74 | Purity fixes, crate deps, dead code removal |
| Foundation | R1–R6 | Deleted duplicates, structural walks, removed fabrication |
| Test quality | BUG-6, E3.2 | Promoted aliases, non-tautological assertions |
