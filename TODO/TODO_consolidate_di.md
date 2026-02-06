# Consolidation: Dependency Injection for System Resources

**Status**: In Progress
**Date**: 2026-02-04
**Updated**: 2026-02-06

System resources (filesystems, platform, environment, clock) should be
acquired at DAG boundaries and injected into business logic — not
constructed inline. This doc tracks remaining violations.

The pattern: acquire a handle/capability at the boundary (EnvOp, main,
transport executor), then pass it down as a parameter or DAG input.

---

## Active violations (remaining)

### 1. std::env in codegen-generated CI runners

**Where**: `core/codegen/src/cli_gen.rs:724,748,767`

```rust
fn load_step_inputs_from_env(step_name: &str) -> HashMap<String, Value> {
    for (key, value) in env::vars() { ... }
}

fn emit_step_outputs(step_name: &str, outputs: &HashMap<String, Value>) {
    if let Ok(output_file) = env::var("GITHUB_OUTPUT") { ... }
    } else if env::var("GITLAB_CI").is_ok() { ... }
}
```

**Problem**: Generated runner functions read env vars directly, making CI
behavior hard to test or mock.

**Fix**: Generated functions should accept an env dictionary parameter
(`HashMap<String, String>`), and `main()` should call `env::vars()` once
and pass the map down.

**Severity**: MEDIUM — generated code, but sets a bad pattern.

---

### 2. CI provider detection reads env vars directly

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

### 3. resolve_tool_path() shells out to `which`

**Where**: `core/ir/src/transport/cli.rs:260-281`

```rust
pub fn resolve_tool_path(tool: &'static CliToolDef) -> Result<PathBuf, CliToolError> {
    let output = Command::new("which")
        .arg(binary)
        .output()?;
    // ...
}
```

**Problem**: Direct system call to `which` — not mockable and not portable
(Windows uses `where`).

**Mitigating factor**: Called from `upsert_tool()` at the tool acquisition
boundary, which is the correct place for this I/O.

**Fix**: Add a trait-based resolver (mockable) or express resolution as a
transport operation.

**Severity**: MEDIUM — correct boundary, but hard to test/mock.

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
