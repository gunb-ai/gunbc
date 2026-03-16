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
- **S52:** Parser mutual recursion not covered by TCO. — MITIGATED (stacker)
  `stacker::maybe_grow` handles deep recursion automatically. Not a correctness
  issue — bounded by AST depth.
- **Performance:** `Env::from_inputs` clones on every non-self call (partially
  mitigated with `Rc<HashMap>` COW). Map field flattening clones every field.

**Accepted permanent splits** in eval stack machine (all documented with rationale):
- `eval_expr` handles non-sibling calls via re-entrant `evaluate_stack`
- `eval_block_s` (suspendable) / `eval_block_pure` (pure, for standalone match/lambda)
- `eval_match_s` (suspendable) / `eval_match_local` (pure, guards can't use continuations)
- `wrap_value_as_output` Map flattening (structurally necessary for v2 multi-field records)

**Terminal state:** Self-hosting eliminates the evaluator entirely.

### Branch 8: Type-unaware codegen — TERMINAL (dies with self-hosting)

fn_codegen compiles .dag function bodies to Rust without type information.
Every decision requiring type info is heuristic. All findings in this branch
are terminal — v2 replaces v1, eliminating the entire codegen path.

**S81: fn_codegen emits Rust, not code_ir — TERMINAL.** ~15 Rust-specific
heuristics injected directly into IR: `clone_if_needed()`, `Box::new()`,
`Some()`/`None`, `.as_str()`, `..Default::default()`, `LazyLock`, `Deref`/`*`.

The code_ir layer exists so one compilation produces IR all backends can
render. DAG nodes are **facts** — target-agnostic assertions about computation.
**Rendering facts** (ownership, optional representation, type naming) belong
in backends, not IR. The structural test: can you swap the backend without
changing the IR?

**S76:** `clone_if_needed()` — blind ownership heuristic. ~300 unnecessary clones. — TERMINAL
**S77:** `infer_struct_name()` — field-name matching for anonymous records. Wrong on overlapping fields. — TERMINAL
**S78:** Materialized types in `std_types_prelude()` — hand-written because v1 can't resolve cross-module imports. — TERMINAL
**S79:** Hardcoded cross-module imports in `module_prelude()` — should derive from `import` declarations. — TERMINAL

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

## V2 self-hosting gap analysis (updated 2026-03-15)

**V2 source:** 10 modules (~9,600 lines), all parse with zero diagnostics.
**Test status:** 81 pass, 0 fail, 4 ignored (3 slow cargo gates + 1 evaluator stack overflow).

**What works today:**
- Module discovery, parsing, type codegen, fn codegen (records, match,
  if/else, for, lambda, string interp, intrinsics, `with()`, `concat()`)
- Recursive type detection, Rust rendering, crate assembly, runtime shims
- Target-agnostic emission architecture (Rust + Python renderers)
- Honest type system (no fabrication in lookup/resolve/emit paths)
- v1 TCO pass for recursive .dag functions (tokenize_loop is iterative)

**Generated v2 crate: cargo check passes.** 3 of 4 ignored tests pass
(cargo build, cargo test, emit-to-target). The 4th (`phase6_gist_full_pipeline`)
passes typecheck but stack-overflows in the v1 evaluator (see S83 below).

**Path forward:**
1. Fix remaining type mismatches in generated code
2. v2 emitter TCO pass (S84) — required for self-hosting
3. Evaluator stack depth for full gist pipeline (S83) — bootstrapping only

### Risks identified during Track A–C integration (2026-03-15)

**S82: Flattened function namespace causes silent overwrites.**
All v2 modules' functions are merged into one `HashMap<String, LoweredFnBody>`
for the evaluator. Name collisions silently overwrite — the last module loaded
wins. `lookup_func_sig` was defined in both `04_typecheck.dag` and `05_emit.dag`
with different signatures. The emit version overwrote the typecheck version,
causing `unbound variable: scope` when the typechecker called it with the wrong
parameter names.
**Fix applied:** Renamed emit's version to `lookup_func_sig_in_scope`.
**Systemic risk:** Any future name collision will produce the same class of bug
with a misleading error message. The flattened namespace has no module isolation.
**Terminal state:** Self-hosting. The v2 compiler resolves imports structurally
and won't flatten namespaces.
**Mitigation until then:** `compile_all_modules()` should detect and reject
duplicate function names across modules.

**S83: Re-entrant evaluator stack overflow on deep call chains. — FIXED (stacker)**
`eval_non_sibling_call_raw` calls `evaluate_stack` re-entrantly for sibling
function calls inside intrinsic lambdas (map/filter/fold). Each re-entrant call
adds ~20 Rust stack frames. Processing 11 real .dag files through the v2
typechecker exceeds the default 8MB thread stack.
**Terminal state:** Self-hosting eliminates the evaluator.
**Fix:** `stacker::maybe_grow` added to re-entrant call sites, growing the
stack on demand. No manual stack size tuning needed.

**S84: v2 emitter has no TCO pass — CRITICAL for self-hosting.**
Track C added tail-call optimization to v1's `fn_codegen.rs` (Stmt::Loop +
parameter reassignment). This fixes the bootstrapping path: v1 compiles v2 .dag
files into iterative Rust. **But the v2 emitter (`05_emit_rust.dag`) does not
perform this transformation.** When v2 compiles itself, the generated Rust will
use recursive calls for functions like `tokenize_loop`, `resolve_imports`,
`collect_service_calls`, and every `fold` accumulator pattern. This will
stack-overflow at runtime, exactly as v1 did before Track C.
**Required:** Add a TCO analysis + transformation pass to the v2 emission
pipeline, analogous to what Track C added to v1. The v2 version should operate
on the typed IR (between typecheck and emit), detecting self-tail-recursive
functions and rewriting them to use a loop construct that the per-target
renderers can emit (`loop {}` for Rust, `while True:` for Python).

**v1 implementation note (2026-03-15):** Track C now uses a `TcoPlan`
intermediate in `fn_codegen.rs` rather than the earlier classify-then-rewrite
pair of passes. This is the smallest redesign that satisfies the v1 invariants:
tail position is modeled structurally instead of by threaded booleans, analysis
and rewriting share one representation, and unsupported recursive contexts fail
closed instead of partially transforming. We explicitly did **not** introduce a
full CFG / terminator IR in v1. A CFG would be cleaner long-term and would make
TCO just another edge rewrite, but the blast radius is too large for a bootstrap
compiler whose long-term future is still uncertain. If v1 becomes strategic,
promote control flow to a real block/terminator IR instead of extending
`TcoPlan`.

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

## Resolved summary (70 findings, 2026-03-10 through 2026-03-15)

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
| v2 integration | S82 (namespace collision), G8/G12–G14 (fabrication) | Renamed, honest return types, sum types for sentinels |
