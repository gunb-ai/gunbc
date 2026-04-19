# tests/boundary/

**Layer:** boundary (TESTING.md § test layers — target-language
roundtrips; <2s target).

Tests whose subject is an external toolchain (rustc, go, CPython).
Emitted source is fed to the real compiler/interpreter and the
output is asserted. These tests are necessarily slower than
integration tests (cold toolchain invocations cost seconds), but
each `#[test]` still aims for <2s via per-file `OnceLock<PathBuf>`
harness caching (see `../integration/common/mod.rs::RustcHarness`).

## Phase 0 contents

Four target-emission suites moved here from `tests/integration/`:

- `m1_3_emit_go_test.rs`    — Go roundtrip
- `m1_3_emit_rust_test.rs`  — rustc roundtrip (module + standalone)
- `m1_4_emit_python_test.rs` — CPython roundtrip
- `m2_emit_multi_field_struct_variant_test.rs` — rustc emit regression

Files are still included from the consolidated `tests/integration.rs`
binary via `#[path = "boundary/…"]` — the directory carries the
taxonomy, the binary carries the compile amortization.

## Adding a file here

Header:

```rust
//! **Layer:** boundary (TESTING.md § test layers — <toolchain> roundtrip).
```

Register in `tests/integration.rs` with
`#[path = "boundary/<name>.rs"] mod <name>;` and add the workspace-
relative path to `tests/integration/sg0_census_test.rs::EXPECTED_HAND_AUTHORED`.
