# WS-C: Typechecker & Resolver Correctness

## Scope

Fix type resolution bugs, import provenance loss, cycle handling, and
expression-tree type holes in the resolver and typechecker.

## Files

- `src/v2/03_resolve.dag`
- `src/v2/04_typecheck.dag`

## Key invariant

> **Correctness by construction, not by validation.** If a property must hold, the
> API must make violations unrepresentable.

## Priority 1: Invariant violations (correctness)

These are documented bugs where the code does the wrong thing.

1. **`merge_envs` first-writer-wins** (`04_typecheck.dag`, fn `merge_envs` ~line 1131)
   - Documented as last-writer-wins but implements first-writer-wins
   - When merging type environments, later bindings should shadow earlier ones
   - Fix: reverse the merge order or use insert-with-overwrite semantics

2. **`build_type_env` import provenance** (`04_typecheck.dag`, fn `build_type_env` ~line 1171)
   - Ignores the target module when resolving imported names
   - Resolves every imported name against `parent_envs` without checking which
     module the name was actually imported from
   - Fix: resolve imports against the specific source module's type env

3. **`dep_order` cycle flow downstream** (`03_resolve.dag`, fn `resolve_modules`)
   - Cyclic modules get `dep_order = -1` (via `find_index_in_list` miss) and still
     flow through the rest of the pipeline
   - Lines ~90-104: `find_index_in_list` returns -1 when a module name isn't in the
     sorted topological order, which happens for cycle members
   - Fix: cyclic modules should produce a diagnostic and be excluded from downstream
     processing, or `dep_order` must be `Option<Int>`

4. **`resolve_import` partial flow** (`03_resolve.dag`, fn `resolve_import` ~line 167)
   - Returns `ResolvedImport { target_module: None }` for unresolved imports
   - This flows downstream where code assumes `target_module` is present
   - Fix: make resolution failures produce diagnostics and exclude failed imports

## Priority 2: Type resolution gaps

1. **`resolve_type_expr` cycle suppress** (`04_typecheck.dag`, fn `resolve_type_expr`)
   - Silently suppresses type resolution when a cycle is detected
   - Fix: emit diagnostic for cyclic type references

2. **`lookup_type` first-binding** (`04_typecheck.dag`, fn `lookup_type`)
   - Returns the first matching binding without checking for uniqueness
   - Multiple bindings with the same name → silently picks one
   - Fix: detect ambiguity and emit diagnostic

3. **`resolve_item_types` skips expressions** (`04_typecheck.dag`, fn `resolve_item_types`)
   - Resolves types in declarations but skips type references inside expression trees
   - Fix: walk expression trees during type resolution

4. **`validate_no_unresolved` incomplete** (`04_typecheck.dag`, fn `validate_no_unresolved`)
   - Validates top-level types but skips expression trees
   - An unresolved `Named("Foo")` inside a function body passes validation
   - Fix: recursive walk of all expression trees

## Priority 3: Provenance loss

1. **`build_type_env` dropped imports** (`04_typecheck.dag`, fn `build_type_env`)
   - Imports not found in parent envs are silently dropped
   - Fix: emit diagnostic for imports that can't be resolved

2. **`type_body_to_expr` alias name loss** (`04_typecheck.dag`, fn `type_body_to_expr`)
   - Type alias names are lost during conversion
   - `UserId = String` becomes just `String` in the type env
   - Fix: preserve alias name in the `TypeExpr`

3. **`resolve_field` from_key drop** (`04_typecheck.dag`, fn `resolve_field`)
   - `from_key` metadata on record fields is dropped during resolution
   - Fix: preserve `from_key` through resolution

4. **`collect_parent_envs` silent miss** (`04_typecheck.dag`, fn `collect_parent_envs`)
   - When a parent env is not found for a module, silently returns empty
   - Fix: emit diagnostic for missing parent environments

## Verification

```bash
# Phase 3-6 tests cover resolve + typecheck
cargo test -p v2-compiler-tests

# Confirm generated crate still compiles
cargo test -p v2-compiler-tests v2_crate_emit_to_target -- --ignored
```

## Coordination notes

- **WS-D dependency**: The emitter (WS-D) has Gap 10/12 about consuming `TypeEnv`
  properly. If you change the `TypeEnv` structure, coordinate with WS-D on the
  boundary type.
- **`TypeEnv` is the contract**: Whatever you change in `merge_envs` or
  `build_type_env`, the resulting `TypeEnv` type is consumed by `05_emit.dag`.
  Changes to the type shape should be noted in WORKBOARD.md.
