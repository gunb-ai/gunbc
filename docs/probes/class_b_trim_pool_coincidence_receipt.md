# Class B live specimen — `trim` pool-membership coincidence (execution receipt)

**Date:** 2026-08-08  
**Session:** lively-bat-817  
**Authority:** `fixtures/class_b_trim/specimen.dag`, `v1-compiler-tests` `class_b_trim_specimen_test`

## Verdict

`trim` is absent from the substrate free-call builtin registry (`compile-clean-forcecheck.md` §6 blast-radius census — trim resolves via the structural method path, not registry rows). A module that free-calls `trim` without listing `std.algebra` in its import block compiles only when `std.algebra` is already in the compilation pool — **pool-membership coincidence**, not `ListedImport`.

Two discriminating failure shapes:

1. **Free-call in narrow pool** — pool contains `std.types` stub without `std.algebra` authority → `function 'trim' not found in scope` (or hard import failure on the missing algebra edge).
2. **Method/free-call on `FreeMonoid<String>` receiver** — even with `std.algebra` in pool, `trim(xs)` refuses because `trim` is on `free_monoid_scalar_templates`, not the collection monoid.

## Repro (green by execution)

```bash
CTRL_BUILD_MODE=local RUSTC_WRAPPER= cargo test -p v1-compiler-tests class_b_trim_specimen -- --nocapture
```

Five tests: narrow-pool failure, declared-pool compile, direct-import absence check, perturbation overlay stability, FreeMonoid receiver refusal.

## Relation to rust_test_fixtures Class B gate

The enrolled `run_class_b_import_closure_gate` rows observe **declared symbols** in `item_registry` (`rust_selection_policy_node`). `trim` has no such registry row — it is not observable through `observe_declared_import_closure_symbol_binding`. This specimen uses compile outcome + pool membership + direct-import census instead, matching the mechanism documented in `import-strip-witness-discovery-cascade-diagnosis.md` §12.
