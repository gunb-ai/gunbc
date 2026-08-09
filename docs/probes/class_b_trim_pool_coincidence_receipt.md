# Class B live specimen — `trim` pool-independent binding (execution receipt)

**Date:** 2026-08-09  
**Session:** lively-bat-817  
**Authority:** `fixtures/class_b_trim/specimen.dag`, `v1-compiler-tests` `class_b_trim_specimen_test`, `v1.compiler.infer_env` `closure_independent_bare_free_call` registry

## Verdict

`trim` is absent from the substrate free-call builtin registry (`compile-clean-forcecheck.md` §6 blast-radius census — trim resolves via the structural method path, not registry rows). **Closure-independent binding for trim landed (#8062):** bare free-call `trim` refuses unless `trim` is in `source_visible_names` via `import std.algebra { trim }`, even when `std.algebra` is already in the compilation pool via unrelated transitive import. Pool-membership coincidence success for trim is the defect this lane closed.

Discriminating shapes:

1. **Bare trim with algebra in pool but no listed import** — refuses (`function 'trim' not found in scope`); `symbol_resolves: false` on binding observation.
2. **Explicit `import std.algebra { trim }` in narrow pool** — compiles; binding source `ListedImport`; unrelated pool perturbation changes nothing.
3. **Method/free-call on `FreeMonoid<String>` receiver** — refuses because `trim` is on `free_monoid_scalar_templates`, not the collection monoid.

## Repro (green by execution)

```bash
CTRL_BUILD_MODE=local RUSTC_WRAPPER= cargo test -p v1-compiler-tests class_b_trim_specimen -- --nocapture
```

Six tests: bare trim refuses without algebra in pool; bare trim refuses with algebra in pool but no listed import; explicit-import refuses without algebra authority; explicit-import compiles in narrow pool with ListedImport binding; perturbation overlay stability; FreeMonoid receiver refusal.

## Relation to rust_test_fixtures Class B gate

The enrolled `run_class_b_import_closure_gate` rows observe **declared symbols** in `item_registry` (`rust_selection_policy_node`). `trim` has no such registry row — it is not observable through `observe_declared_import_closure_symbol_binding`. This specimen uses compile outcome + pool membership + direct-import census instead, matching the mechanism documented in `import-strip-witness-discovery-cascade-diagnosis.md` §12.
