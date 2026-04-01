# Bootstrap B + Lane Work: Scoping & Design

This document scopes and designs the work items from the planning
discussion. Each item includes: what needs to change, which files are
touched, dependencies, risks, and a concrete design sketch where
non-trivial.

**Baseline (verified 2026-03-31, after merging bootstrap-b-remaining):**

| Gate | Status |
|------|--------|
| `cargo test --workspace --exclude v2-compiler-tests` | 12 pass, 4 ignored |
| `cargo test -p v2-compiler-tests` | 263 pass, 15 ignored |
| `cargo clippy --all-targets -- -D warnings` | Clean |
| `full_dsl_compiles -- --ignored` | 91 dsl + 29 v2, 0 diagnostics |
| `strict_compile_diagnostic_count -- --ignored` | 310 errors (ratchet 316) |
| `scripts/l1-ratchet.sh --check` | L1 = 21 (ratchet 22) |
| `EMITTED_RUST_ERROR_RATCHET` (bootstrap.rs) | 880 |
| ROADMAP dashboard Bootstrap B | **11 errors** (down from 419→11 via bootstrap-b-remaining) |

**Parallel work (bootstrap-b-remaining, now merged):**

The `bootstrap-b-remaining` branch landed 36 commits reducing Bootstrap B
from 419 → 11 errors. This work completed:
- BinOp/BinOpKind and LiteralValue/LiteralKind unification (type cascade)
- Empty map emission fixes
- Data def Rc wrapping, double-Rc removal
- CodegenBackend import attribution, ExprData variant patterns
- String/&str match with Bind patterns, char code point comparisons
- Optional comparison bin_op wiring, slice end parameter fix
- BOOTSTRAP_MODE flag for complexity gate
- Early Detection invariant added to INVARIANTS.md

**Remaining 11 errors (from ROADMAP on bootstrap-b-remaining):**
- Type cascade: 6 errors (deferred to M4 — `std.syntax` ↔ `v2.compiler.languages`)
- Inference leaks: 3 errors (Error type reaching emit — 3 coded + 7 uncoded)
- FreeMonoid/PartialFunction generics: 2 errors

**Active parallel work (still in progress on bootstrap-b-remaining):**
- Fix inference leaks: Error type reaching emit (IN PROGRESS)
- Thread generic params for FreeMonoid/PartialFunction (NOT STARTED)

**Constraint:** Items 6, 7, 8 from the original plan overlap with the
active parallel work. This branch should NOT touch the inference leak
or FreeMonoid generic threading areas until the parallel work merges.

---

## 1. L1 Tier 1 Remaining: Per-Profile Field Builders → `.dag` Functions

### What

Convert the six per-profile `data` template lists in `dsl/std/algebra.dag`
from inert data declarations that the compiler interprets in
`src/v2/04_types.dag` into `.dag` functions that produce `Node` fields
directly — so the enrichment logic lives in the DSL, not in the compiler.

### Current State

`dsl/std/algebra.dag` defines six `data` declarations:

| Data | Profile | Templates |
|------|---------|-----------|
| `ordered_ring_templates` | `OrderedRingProfile` | 6 fields (add, zero, negate, mul, one, compare) |
| `approximate_field_templates` | `ApproximateFieldProfile` | 7 fields |
| `boolean_algebra_templates` | `BooleanAlgebraProfile` | 5 fields (meet, join, complement, top, bottom) |
| `free_monoid_scalar_templates` | `FreeMonoidScalarProfile` | 17 fields (String methods) |
| `free_monoid_collection_templates` | `FreeMonoidCollectionProfile` | 20 fields (List/Set methods) |
| `partial_function_templates` | `PartialFunctionProfile` | 16 fields (Map methods) |

`src/v2/04_types.dag` has ~100 lines of interpreter code
(`instantiate_algebra_type`, `instantiate_algebra_field`,
`algebra_templates_for_profile`, `enrich_kernel_type`) that reads these
data tables and builds `Node` children.

### Design

Add functions to `dsl/std/algebra.dag` that produce the enriched field
list directly. The core idea: instead of `data` + interpreter, the field
construction functions ARE the authority.

Two options:

**Option A (incremental):** Keep `AlgebraFieldTemplate` as the type, but
add per-profile `.dag` functions that return `List<AlgebraFieldTemplate>`
instead of `data` declarations. This preserves the current compiler-side
`instantiate_algebra_field` logic but moves the template lists from data
to functions. Minimal change.

**Option B (full dissolution):** The functions in `algebra.dag` return
actual enriched `Node` children (using imported `algebra_method_field`
and `algebra_value_field` from `04_types.dag`). This requires
`algebra.dag` to import compiler node-construction helpers, which
crosses the DSL/compiler boundary and is architecturally worse.

**Recommendation: Option A.** The templates stay as the intermediate
representation; the authority moves from `data` to `fn`. The interpreter
in `04_types.dag` is preserved but reads function results instead of data.

```dag
// In algebra.dag — replaces `data ordered_ring_templates`:
fn ordered_ring_fields() -> List<AlgebraFieldTemplate> {
  [
    { name: "add", param_types: [ReceiverSelf, ReceiverSelf], return_type: ReceiverSelf },
    { name: "zero", param_types: [], return_type: ReceiverSelf },
    // ... (same content as current data declaration)
  ]
}
```

In `04_types.dag`, `algebra_templates_for_profile` calls the function:

```dag
fn algebra_templates_for_profile(profile: AlgebraProfile) -> List<AlgebraFieldTemplate> {
  match profile {
    OrderedRingProfile => ordered_ring_fields()
    // ...
  }
}
```

### Files Changed

| File | Change |
|------|--------|
| `dsl/std/algebra.dag` | Convert 6 `data` → 6 `fn` returning same `List<AlgebraFieldTemplate>` |
| `src/v2/04_types.dag` | Update imports, change `algebra_templates_for_profile` to call functions |

### Dependencies

None. Pure modeling change. No emitter or stage0 changes.

### Risk

Low. The function bodies are identical to the current data values. Tests:
`full_dsl_compiles` must stay green (0 diagnostics). Behavioral tests for
method resolution (`higher_order_method_instantiation`, keyed-collection
access tests from PR #270) must pass.

### Landing

Can land on `main` independently.

---

## 2. L1 Tier 2.5: Fix Set/NonEmptySet Algebra Profile

### What

`kernel_algebra_profile` maps `"Set"` and `"NonEmptySet"` to
`FreeMonoidCollectionProfile`, which gives them list operations (append,
sort_by, fold, etc.). The denotational story says sets inhabit
`BooleanAlgebra<A>` — they should have union/intersect/diff/member, not
append/sort_by.

### Current State

- `dsl/std/algebra.dag` line 366-369: `"Set": FreeMonoidCollectionProfile`,
  `"NonEmptySet": FreeMonoidCollectionProfile`
- Comments in `algebra.dag` lines 206-208 and `types.dag` lines 120-128
  describe Set as Boolean algebra.
- `boolean_algebra_templates` has 5 fields: meet, join, complement, top,
  bottom — suitable for `Bool`, not for `Set<A>`.

### Design

Create a new `BooleanAlgebraCollectionProfile` variant and template list:

```dag
type AlgebraProfile
  = OrderedRingProfile
  | ApproximateFieldProfile
  | BooleanAlgebraProfile
  | BooleanAlgebraCollectionProfile  // NEW
  | FreeMonoidScalarProfile
  | FreeMonoidCollectionProfile
  | PartialFunctionProfile
```

New template data (or function, depending on whether item 1 has landed):

```dag
data boolean_algebra_collection_templates: List<AlgebraFieldTemplate> = [
  // Set-theoretic operations (BooleanAlgebra<A> applied pointwise)
  { name: "union", param_types: [ReceiverSelf, ReceiverSelf], return_type: ReceiverSelf },
  { name: "intersect", param_types: [ReceiverSelf, ReceiverSelf], return_type: ReceiverSelf },
  { name: "diff", param_types: [ReceiverSelf, ReceiverSelf], return_type: ReceiverSelf },
  { name: "member", param_types: [ReceiverSelf, ReceiverElement], return_type: NamedTemplate { name: "Bool" } },
  // Shared collection operations that ARE valid for sets
  { name: "filter", param_types: [ReceiverSelf], return_type: ReceiverSelf },
  { name: "map", param_types: [ReceiverSelf], return_type: ReceiverSelf },
  { name: "fold", param_types: [ReceiverSelf], return_type: ReceiverSelf },
  { name: "any", param_types: [ReceiverSelf], return_type: NamedTemplate { name: "Bool" } },
  { name: "all", param_types: [ReceiverSelf], return_type: NamedTemplate { name: "Bool" } },
  { name: "count", param_types: [ReceiverSelf], return_type: NamedTemplate { name: "Int" } },
  { name: "contains", param_types: [ReceiverSelf, ReceiverElement], return_type: NamedTemplate { name: "Bool" } },
  // Operations NOT valid for sets (excluded):
  // - append (sets don't have positional append)
  // - sort_by (sets have no ordering)
  // - enumerate (sets have no index)
  // - reverse (sets have no order)
  // - concat (use union instead)
  // - list_push (list-specific)
]
```

Update `kernel_algebra_profile`:

```dag
data kernel_algebra_profile: Map<String, AlgebraProfile> = {
  ...
  "Set": BooleanAlgebraCollectionProfile,
  "NonEmptySet": BooleanAlgebraCollectionProfile,
  ...
}
```

### Carrier-Changing Type Loss (Parallel Issue)

The ROADMAP Tier 2.5 also notes that `map`/`flat_map`/`fold` in
`free_monoid_collection_templates` use `ReceiverSelf` for `param_types`
and `return_type`, but they should express higher-order function
parameter structure. For example, `fold` takes `fn(Acc, T) -> Acc` and
returns `Acc`, not `Self`.

This is a deeper modeling fix that affects how `instantiate_algebra_type`
builds callable signatures for these methods. It would require new
`AlgebraTypeTemplate` variants like `CallableOf { params, return_type }`
or explicit lambda parameter modeling in the templates.

**Recommendation:** Fix the Set/NonEmptySet profile assignment first
(straightforward), then address carrier-changing types as a follow-up.
The carrier-changing fix blocks the full "delete
`is_bridge_placeholder_type_name`" goal but is not required for correct
Set behavior.

### Files Changed

| File | Change |
|------|--------|
| `dsl/std/algebra.dag` | Add `BooleanAlgebraCollectionProfile` variant + template list |
| `src/v2/04_types.dag` | Add match arm in `algebra_templates_for_profile` |

### Dependencies

None. Pure `.dag` modeling work. No emitter changes.

### Risk

Low-medium. Existing code that calls `set.sort_by(...)` or
`set.append(...)` would stop resolving. Need to verify no `.dag` source
files use list-only operations on Set types.

### Landing

Can land on `main` independently.

---

## 3. Model Convergence Cleanup

### 3a. ROADMAP Convergence Table Update

The ROADMAP dashboard (line 27) reports Bootstrap B at 419 errors with
a breakdown of `CodegenBackend import (192), algebra fn-field derives (71),
downstream (114), misc (42)`. The ratchet in `bootstrap.rs` is 880.
Recent PRs (#264, #270, #272, #273, #275) have landed fixes for
BinOp/LiteralValue/OperatorSpec and algebra structural templates.

**Action:** Re-run `bootstrap_stage0_to_stage1` (if feasible in the
environment) and update both the ROADMAP dashboard number and the
`EMITTED_RUST_ERROR_RATCHET` in `bootstrap.rs` to reflect the actual
current error count. Also update `strict_compile_diagnostic_count`
ratchet if 311 < 316 (current code has `DIAG_RATCHET = 316`, observed
311).

### 3b. `rt_functions` / `rt_function_registry` Duplicate Authority

**Location:** `dsl/extdeps/languages/rust/emit.dag` lines 96-190.

**Current state:**
- `rt_function_registry: List<RuntimeFunction>` (lines 96-138) — the
  single hand-maintained authority
- `rt_functions: Map<String, Bool>` (lines 151-161) — derived index
- `rt_ref_map_functions: Map<String, Bool>` (lines 163-166) — derived
- `rt_bridge_function_names: Map<String, String>` (lines 168-175) —
  derived

All three derived maps must be kept in sync manually because `.dag`
doesn't support computed data declarations yet.

**Design:** Since computed data declarations aren't available, the
cleanest intermediate fix is to:

1. Add a comment block at each derived map explicitly listing which
   registry entries it should contain (making manual sync auditable).
2. Add a source-audit test that verifies the derived maps are consistent
   with `rt_function_registry` — parse both sections, extract the name
   lists, and assert they match.

**Longer-term:** When computed data declarations land (ROADMAP
"Exploratory Directions"), `rt_functions` et al. become
`data rt_functions = rt_function_registry |> fold(...)`.

### Files Changed

| File | Change |
|------|--------|
| `ROADMAP.md` | Update dashboard numbers |
| `src/v2/tests/src/bootstrap.rs` | Update ratchet constants |
| `dsl/extdeps/languages/rust/emit.dag` | Add sync comments |
| `src/v2/tests/src/source_audit.rs` (or equivalent) | Add consistency test |

### Dependencies

Depends on being able to run the bootstrap test (requires release build
+ cargo check on emitted code). The ROADMAP update and consistency test
can land independently.

### Risk

Low. Documentation and test changes only.

---

## 4. CI Improvements: Emitted-Rust Error Count Ratchet

### What

Add the bootstrap emitted-Rust error count as a CI ratchet test, even
before B=0 — just ratchet at the current count.

### Current CI State

`.github/workflows/ci.yml` runs only:
1. `cargo clippy --workspace -- -D warnings`
2. `cargo test -p v2-compiler-tests`

The `bootstrap_stage0_to_stage1` test is `#[ignore]` and not in CI.
The ROADMAP "Required Before Merge" section lists additional gates
(`l1-ratchet.sh`, `full_dsl_compiles`, `strict_compile_diagnostic_count`,
`bootstrap_fixed_point`) that are also not in CI.

### Design

Add a CI step that runs the bootstrap test. This requires:

1. **Release build** of `v2-compiler` (~2-5 min on CI)
2. **Stage0→stage1 compile** (~1-2 min)
3. **`cargo check`** on emitted Rust (~2-5 min)

Total: ~5-12 minutes. Within the 30-minute CI timeout.

```yaml
# In ci.yml, after the Test step:
- name: Bootstrap B Ratchet
  run: cargo test -p v2-compiler-tests bootstrap_stage0_to_stage1 -- --ignored --nocapture
  timeout-minutes: 15
```

**Alternative (lighter):** Instead of the full `cargo check`, add a
simpler ratchet that just counts emitted files or checks for specific
error patterns. But the full `cargo check` is the most honest gate.

### Incremental CI Additions

While we're editing `ci.yml`, also add the gates that ROADMAP says are
"Required Before Merge":

```yaml
- name: Full DSL Compiles
  run: cargo test -p v2-compiler-tests full_dsl_compiles -- --ignored --nocapture

- name: Diagnostic Ratchet
  run: cargo test -p v2-compiler-tests strict_compile_diagnostic_count -- --ignored --nocapture

- name: L1 Ratchet
  run: scripts/l1-ratchet.sh --check
```

The `bootstrap_stage0_to_stage1` test itself already has the ratchet
assertion (`error_count <= EMITTED_RUST_ERROR_RATCHET`). CI just needs
to run it.

### Files Changed

| File | Change |
|------|--------|
| `.github/workflows/ci.yml` | Add bootstrap + diagnostic + L1 + full_dsl steps |
| `src/v2/tests/src/bootstrap.rs` | Update `EMITTED_RUST_ERROR_RATCHET` to current count (if changed) |

### Dependencies

None. The tests already exist and pass locally. CI addition is additive.

### Risk

Medium. The bootstrap test requires building in release mode, which may
push CI time close to the 30-minute timeout on free GitHub runners. May
need to increase `timeout-minutes` or make the bootstrap step conditional
(e.g., only on PRs targeting main, not on every push).

---

## 5. Test Coverage: Higher-Order Method Instantiation & Keyed-Collection Access

### What

Add more behavioral tests for:
- Higher-order method instantiation (map/filter/fold/sort_by with lambdas)
- Keyed-collection access (Map get/insert/has/keys/values)

PR #270 added initial tests. Reviewers asked for more.

### Current Tests (from PR #270)

From the git log, PR #270 added `higher_order_method_instantiation` and
keyed-collection access tests in the pipeline test suite.

### Design

New tests should cover edge cases that the current inference engine
handles (or fails to handle). Tests that don't depend on stage0
regeneration means they go in `src/v2/tests/src/pipeline.rs` using
the existing `compile_and_check` / `compile_module` test helpers.

**Proposed test cases:**

```
// Higher-order method instantiation
1. map with identity lambda: `list |> map(x => x)` — result type = List<T>
2. map with type-changing lambda: `list |> map(x => to_string(x))` — result type = List<String>
3. filter preserves type: `list |> filter(x => x > 0)` — result type = same as input
4. fold to different type: `list |> fold(init: 0, f: (acc, x) => acc + x)` — result type = Int
5. fold to map: `list |> fold(init: empty_map(), f: (acc, x) => ...)` — result type = Map
6. sort_by preserves type: `list |> sort_by((a, b) => compare(a, b))` — result type = same
7. flat_map: `list |> flat_map(x => [x, x])` — result type = List<T>
8. chained: `list |> filter(x => ...) |> map(x => ...) |> fold(init: 0, ...)`
9. enumerate: `list |> enumerate` — result type = List<Tuple<Int, T>>

// Keyed-collection access
10. map_get: `m |> get(key)` — result type = Optional<V>
11. map_insert: `m |> map_insert(k, v)` — result type = Map<K, V>
12. map_has: `m |> has(k)` — result type = Bool
13. map_keys: `m |> keys` — result type = List<K>
14. map_values: `m |> values` — result type = List<V>
15. map_merge: `m1 |> map_merge(m2)` — result type = Map<K, V>
```

Each test is a small `.dag` snippet that compiles with 0 diagnostics,
verifying that the inference engine resolves the method + lambda params
correctly.

### Files Changed

| File | Change |
|------|--------|
| `src/v2/tests/src/pipeline.rs` | Add ~15 test functions |

### Dependencies

None. Uses existing test infrastructure.

### Risk

Low. Behavioral tests only. May uncover inference gaps that become
tracked issues.

---

## 6. Bootstrap B: Inference Propagation (the 8 errors)

### What

The highest-leverage fix for Bootstrap B. The inference engine
(`04_infer.dag`) doesn't fully propagate element types into lambda
parameters for `sort_by` and `fold` in all contexts. This causes
emitted Rust code to have type errors where rustc can't infer what
the `.dag` compiler should have told it.

### Root Cause Analysis

From `INVARIANTS.md` (IV-6, IV-7, IV-8) and the inference code:

1. **Forward-only inference:** The inference engine resolves types
   top-down from declarations. It does NOT propagate expected types
   backward from function parameter signatures to argument expressions
   in all cases.

2. **`empty_map()` as argument:** When `empty_map()` appears as an
   argument to a function expecting `Map<String, Bool>`, inference
   produces `Map<String, Unit>` instead of `Map<String, Bool>` because
   it doesn't propagate the formal parameter type to the argument.

3. **Fold accumulator type:** When fold's init is `empty_map()`, the
   accumulator type comes out as `Map<String, Unit>` instead of being
   refined from the fold body's usage.

4. **Lambda param types for sort_by:** `sort_by(fn(T, T) -> Int)` needs
   the comparator lambda to know `T` is the element type. The current
   code threads element types via `infer_arg_with_element_type` using
   `expected`, but `ExprLambda` has a special case: with 2 params and a
   non-callable `expected`, only the **last** param gets the element
   type — the first gets `type_variable_node("lambda_param")`.

### Design: Two-Phase Fix

**Phase 1: Signature-driven expected types for call arguments**

In `infer_expr` for `ExprCall`, when a function signature is known, the
current code already builds `expected` for callable-shaped formal params.
Extend this to ALL formal params, not just callable ones:

```dag
// Current (04_infer.dag, ExprCall with known sig):
if formal_param_type.params |> count > 0 {
  // Only callable formals get expected threading
  expected: Some { value: formal_param_type }
}

// Proposed:
// Thread expected type for ALL formals, not just callables
expected: Some { value: formal_param_type }
```

This fixes IV-6: `empty_map()` as an argument to `f(m: Map<String, Bool>)`
now infers as `Map<String, Bool>`.

**Phase 2: Fold accumulator refinement from body**

The fold accumulator type is currently set from the `init` expression
only (via `extract_fold_init_info`). If init is `empty_map()`, the acc
type is `Map<String, Unit>`.

Fix: after inferring the fold body lambda, extract its return type and
use it to refine the accumulator type if the init type was incomplete
(has Unit children / type variables).

```dag
// In infer_method_args_with_fold, after inferring fold body:
let body_return_type = ... // extract from inferred lambda
let refined_acc = if acc_type_has_unit_children(fold_acc_type) {
  prefer_specific_type(left: fold_acc_type, right: body_return_type)
} else {
  fold_acc_type
}
```

**Phase 3: Lambda multi-param expected types**

For `sort_by((a, b) => compare(a, b))`, the lambda has 2 params. Current
code: last param gets element type, first gets `type_variable`. Fix:
when the `expected` is a non-callable element type and the lambda has
exactly 2 params, assign element type to BOTH params (since sort_by's
comparator takes `(T, T) -> Int`).

```dag
// In ExprLambda expected handling:
} else if lam_params |> count == 2 && is_sort_by_context {
  // Both params get element type for sort_by comparators
  extend_scope(scope: scope, name: first_param, resolved: exp)
  extend_scope(scope: acc, name: second_param, resolved: exp)
} else {
  // Existing: last param gets element type
}
```

The "is_sort_by_context" detection could be structural (expected carries
some marker) or the simpler fix: when expected is not callable and there
are exactly 2 params, assume both should get the element type (since
the only 2-param non-callable lambda context in the language is
comparison).

### Files Changed

| File | Change |
|------|--------|
| `src/v2/04_infer.dag` | Extend expected threading in ExprCall, fold refinement, lambda multi-param |
| `src/v2/tests/src/pipeline.rs` | Add tests for each fix scenario |

### Dependencies

None technically, but should land after item 5 (test coverage) to have
the test infrastructure ready.

### Risk

**High.** This is inference engine work. Changes to `infer_expr` affect
every expression in every `.dag` file. Careful regression testing
required:
- `full_dsl_compiles` must stay at 0 diagnostics
- `strict_compile_diagnostic_count` must not increase
- `bootstrap_stage0_to_stage1` error count should decrease
- All 263 existing tests must pass

### Landing

Must land on the bootstrap branch, not main, because it affects emitted
code that stage0 regeneration depends on.

---

## 7. Bootstrap B: FreeMonoid Generics (the 2 errors)

### What

When the emitter renders a type that resolves to `FreeMonoid<T>` (the
algebra alias for `List<T>`), it emits `FreeMonoid<T>` as a Rust type
name instead of `Vec<T>`. Rust doesn't have a `FreeMonoid` type.

### Root Cause

Type resolution preserves the `FreeMonoid` name from `dsl/std/algebra.dag`
(`type List<element> = FreeMonoid<element>` in `types.dag`). When the
emitter encounters the resolved type, it renders the name literally.

### Design

The fix is in the Rust emitter's type rendering. When emitting a type
node whose name is `FreeMonoid`, substitute the appropriate Rust
container type:

```dag
// In emit_node_type_* or emit_inferred (05_emit_rust.dag):
let rendered_name = if name == "FreeMonoid" { "Vec" } else { name }
```

Alternatively, fix at the resolve level: type alias resolution should
normalize `FreeMonoid<T>` back to `List<T>` before it reaches emit. This
is more principled but touches the resolver.

### Files Changed

| File | Change |
|------|--------|
| `src/v2/05_emit_rust.dag` | Add FreeMonoid → Vec mapping in type rendering |
| OR `src/v2/04_resolve.dag` | Normalize FreeMonoid aliases during resolution |

### Dependencies

None. Can fix independently.

### Risk

Low. Two-line change in emitter. Must verify no other algebra type names
leak through (check `PartialFunction` → `BTreeMap` mapping exists).

---

## 8. Bootstrap B: Unit Data Item (the 1 error)

### What

A `data` item with type `Unit` (e.g., `data x: Unit = Unit`) causes an
edge case in the Rust emitter's data definition emission.

### Design

Need to identify the specific `.dag` source that triggers this. The
emitter's `emit_data_def` handles scalars, maps, and lists — but `Unit`
(which maps to `()` in Rust) likely falls through a gap in the type
rendering or the constructor function emission.

Fix: add a `Unit` special case in `emit_data_def` that either:
- Emits `fn x() -> () { () }`, or
- Skips emission entirely (Unit data is a no-op)

### Files Changed

| File | Change |
|------|--------|
| `src/v2/05_emit_rust.dag` | Handle Unit type in data definition emission |

### Risk

Low. Edge case fix.

---

## Execution Order & Dependencies (Revised Post-Merge)

After merging `bootstrap-b-remaining`, the landscape is:

- Items 6, 7, 8 (inference propagation, FreeMonoid generics, Unit data)
  **overlap with active parallel work** on `bootstrap-b-remaining`. The
  other branch is actively fixing inference leaks and will tackle
  FreeMonoid/PartialFunction generics next. **Do not duplicate this work.**

- Items 1, 2, 3b, 4, 5 are **safe to proceed** on this branch — they
  don't overlap with the parallel work.

```
                    Safe to proceed (this branch)
                    ┌──────────────────────────────┐
                    │ 1. Algebra fn builders        │
                    │ 2. Set/NonEmptySet profile    │
                    │ 3b. rt_functions consistency  │
                    │ 4. CI ratchet additions       │
                    │ 5. Test coverage              │
                    │ 3a. ROADMAP number update     │
                    └──────────────────────────────┘

                    BLOCKED — parallel work in progress
                    ┌──────────────────────────────┐
                    │ 6. Inference propagation      │ ← active on bootstrap-b-remaining
                    │ 7. FreeMonoid generics        │ ← queued on bootstrap-b-remaining
                    │ 8. Unit data item             │ ← may be covered by parallel work
                    └──────────────────────────────┘
```

**Recommended execution order for this branch:**

1. **Item 5** (Test coverage): Higher-order method + keyed-collection
   tests. Establishes safety net for everything else.

2. **Item 1** (Algebra fn builders): Pure modeling, `dsl/std/algebra.dag`
   + `src/v2/04_types.dag`. No overlap.

3. **Item 2** (Set/NonEmptySet profile): Pure modeling, same files +
   new `BooleanAlgebraCollectionProfile`. No overlap.

4. **Item 4** (CI ratchet): Add gates to ci.yml. Should update the
   ratchet constant to a tighter value after confirming current count.

5. **Item 3** (Convergence): ROADMAP numbers + rt_functions consistency
   test. Low risk.

**After parallel work merges:** Re-evaluate items 6, 7, 8. If the
parallel branch resolves inference leaks and FreeMonoid generics,
those items close. The inference propagation design (Phase 1-3) in
this document remains valid as reference if gaps remain.

### Estimated Complexity

| Item | Files | Invasiveness | Dependencies |
|------|-------|-------------|-------------|
| 1. Algebra fn builders | 2 | Low (data→fn conversion) | None |
| 2. Set/NonEmptySet | 2 | Low (new profile + templates) | None |
| 3a. ROADMAP update | 2 | Low (number changes) | Bootstrap run |
| 3b. rt_functions sync | 2 | Low (comments + test) | None |
| 4. CI ratchet | 1-2 | Low (yaml + constant) | None |
| 5. Test coverage | 1 | Low (additive tests) | None |
| 6. Inference propagation | 2 | **High** (inference engine) | Items 5, 7, 8 |
| 7. FreeMonoid generics | 1-2 | Low (name mapping) | None |
| 8. Unit data item | 1 | Low (edge case) | None |
