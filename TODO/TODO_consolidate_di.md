# Consolidation: Dependency Injection for System Resources

**Status**: In Progress
**Date**: 2026-02-04
**Updated**: 2026-02-07

System resources (filesystems, platform, environment, clock) should be
acquired at DAG boundaries and injected into business logic — not
constructed inline. This doc tracks remaining violations.

The pattern: acquire a handle/capability at the boundary (EnvOp, main,
transport executor), then pass it down as a parameter or DAG input.

---

## Active violations (remaining)

### 1. CI provider detection reads env vars directly

**Where**: `core/ir/src/transport/ci/provider.rs:79-99`

```rust
pub fn detect_provider() -> Box<dyn CiProvider> {
    if std::env::var("GITHUB_ACTIONS").is_ok() { ... }
    if std::env::var("GITLAB_CI").is_ok() { ... }
}
```

**Problem**: Reads env vars to auto-detect CI provider. Not injectable.

**Mitigating factor**: This is called at the application boundary
(`main()`/`CiContext::detect()`), which is acceptable. This is low-priority
and could be aligned with the env-dict pattern above if we standardize it.

**Severity**: LOW

---

### 2. Tool path resolution defaults to `which` (Unix-only)

**Where**: `lib/transport/src/cli.rs`

**Current state**: Resolution is injectable via `ToolPathResolver` and
`resolve_tool_path_with()`, and tests use `MockResolver`. The default
`WhichResolver` still shells out to `which`.

**Problem**: Default resolver is Unix-specific; Windows should use `where`.

**Fix**: Add a Windows resolver (or a platform-agnostic resolver) and select
based on platform. Keep `WhichResolver` for Unix.

**Severity**: LOW — DI is solved, portability remains.

---

## Resolved (2026-02-06)

- **FilesystemHandle in gist-ops**: `sanitize_branch_for_filename` and
  `generate_gist_filename` now accept a `FilesystemHandle` parameter; no
  inline acquisition.
- **Platform detection in deps GenerateScripts**: DAG now passes `Platform`
  input; `Installer::for_platform(platform)` is used in ops. (`Installer::default`
  still calls `Platform::detect()` for convenience/tests.)
- **Auth env vars in transport executor**: moved to `CredentialOp` at the
  DAG boundary.
- **SystemTime in gist filename generation**: timestamp is now passed in as
  a parameter.

## Resolved (2026-02-07)

- **Env dict injection in codegen step runners**: step-mode CLI now captures
  `env::vars()` once and passes a `HashMap<String, String>` into
  `load_step_inputs_from_env()` and `emit_step_outputs()`; those functions
  no longer read env vars directly.
