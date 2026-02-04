# Consolidation: Dependency Injection for System Resources

**Status**: In Progress
**Date**: 2026-02-04

System resources (filesystems, platform, environment, clock) should be
acquired at DAG boundaries and injected into business logic — not
constructed inline. This doc tracks every known violation.

The pattern: acquire a handle/capability at the boundary (EnvOp, main,
transport executor), then pass it down as a parameter or DAG input.

---

## 1. ~~FilesystemHandle — constructed inline in gist-ops~~ ✅ FIXED (Phase 1)

**Where**: `lib/gist-ops/src/lib.rs`

**What was done**: Removed the non-injectable `sanitize_branch_for_filename(branch)`
wrapper. The function now requires `&FilesystemHandle` as its first parameter:
`sanitize_branch_for_filename(&fs, branch)`. The `GistOps::PrepareRequest::execute()`
method now explicitly constructs the handle at the call site (visible DI violation
for Phase 2 to wire through DAG edges). All tests updated.

**Remaining**: Phase 2 — add `FsEnv` node to the gist DAG to acquire the handle
at the boundary instead of inline in execute().

---

## 2. ~~Installer::new() calls Platform::detect() inline~~ ✅ FIXED (Phase 1)

**Where**: `lib/tools/deps/src/installer.rs`, `lib/tools/deps/src/ops.rs`

**What was done**: Removed `Installer::new()`. All call sites now use
`Installer::for_platform(Platform::detect())` (production) or
`Installer::for_platform(Platform::Linux)` (tests). The `Platform::detect()`
call is now visible at the call site. `Default` impl updated to delegate
to `for_platform(Platform::detect())`.

**Remaining**: Phase 2 — add `PlatformEnv` node to the deps DAG to acquire
the platform at the boundary instead of inline in execute().

---

## 3. ~~std::env::var() in transport executor (auth resolution)~~ ✅ FIXED (Phase 1)

**Where**: `lib/transport/src/executor.rs`, `lib/transport/src/ops.rs`

**What was done**: Added `AuthMethod::resolve_env_vars()` that takes an
injectable lookup function and converts `EnvVar(name)` → `Bearer(value)`
and `EnvVarHeader { header, env_var }` → `ApiKey { header, key }`.
`RestRequest::resolve_auth()` wraps this for convenience.

Resolution now happens in `TransportOps::Execute` (the DAG node in `ops.rs`)
before calling into the transport executor. The executor (`executor.rs`)
no longer reads env vars — `EnvVar`/`EnvVarHeader` arms hit a `debug_assert!`
if they somehow reach the executor unresolved.

Auth resolution is fully testable: tests pass mock lookup closures instead
of setting real environment variables. 8 new tests cover all variants.

**Remaining**: Phase 2 — pass resolved auth through DAG input ports instead
of resolving inline in `TransportOps::Execute`.

---

## 4. std::env in codegen-generated CI runners

**Where**: `core/codegen/src/cli_gen.rs:724,748,767`

```rust
fn load_step_inputs_from_env(step_name: &str) -> HashMap<String, Value> {
    for (key, value) in env::vars() { ... }        // line 724
}

fn emit_step_outputs(step_name: &str, outputs: &HashMap<String, Value>) {
    if let Ok(output_file) = env::var("GITHUB_OUTPUT") { ... }  // line 748
    } else if env::var("GITLAB_CI").is_ok() { ... }             // line 767
}
```

**Problem**: Generated runner functions read env vars directly. Can't mock
CI environments for testing generated code.

**Fix**: Generated functions should accept an env dictionary parameter
(`HashMap<String, String>`) instead of calling `env::vars()` directly.
The actual `env::vars()` call happens once in the generated `main()`.

**Severity**: MEDIUM — this is generated code, not library code. But it
sets a bad pattern for anyone reading it as an example.

---

## 5. ~~SystemTime::now() in gist filename generation~~ ✅ FIXED (Phase 1)

**Where**: `lib/gist-ops/src/lib.rs`

**What was done**: Removed the non-injectable `generate_gist_filename(branch)`
wrapper. The function now requires `(&FilesystemHandle, &str, SystemTime)`:
`generate_gist_filename(&fs, branch, now)`. The `GistOps::PrepareRequest::execute()`
method now explicitly captures `SystemTime::now()` at the call site. All tests
updated to use fixed timestamps, making them fully deterministic.

**Remaining**: Phase 2 — add `ClockEnv` node to the gist DAG to acquire the
timestamp at the boundary instead of inline in execute().

---

## 6. CI provider detection reads env vars directly

**Where**: `core/ir/src/transport/ci/provider.rs:79-99`

```rust
pub fn detect_provider() -> Box<dyn CiProvider> {
    if std::env::var("GITHUB_ACTIONS").is_ok() { ... }
    if std::env::var("GITLAB_CI").is_ok() { ... }
}
```

**Problem**: Reads env vars to auto-detect CI provider. Not injectable.

**Mitigating factor**: This is called from `main()` and `CiContext::detect()`
— both are at the application boundary, which is the correct place for
detection. The result is then passed around as `CiContext`.

**Fix**: Low priority. Could accept an env dictionary for testability,
but the current boundary placement is acceptable. If we make the env-dict
pattern standard (item 4), this would naturally follow.

**Severity**: LOW — already at the correct boundary.

---

## 7. resolve_tool_path() shells out to `which`

**Where**: `core/ir/src/transport/cli.rs:260-281`

```rust
pub fn resolve_tool_path(tool: &'static CliToolDef) -> Result<PathBuf, CliToolError> {
    let output = Command::new("which")
        .arg(binary)
        .output()?;
    // ...
}
```

**Problem**: Direct system call to `which` — can't mock for testing, and
`which` isn't available on all platforms (Windows uses `where`).

**Mitigating factor**: Called from `upsert_tool()` which is used at the
EnvOp boundary (the correct I/O boundary for tool acquisition).

**Fix**: Add a trait-based tool resolver that the transport layer can mock.
Or use the existing transport pattern: build a `PrepareResolve` request
and execute through the transport layer.

**Severity**: MEDIUM — at the right boundary, but hard to test/mock.

---

## Summary

| # | Resource | Location | Severity | Fix |
|---|----------|----------|----------|-----|
| 1 | ~~FilesystemHandle~~ | ~~gist-ops/lib.rs:113~~ | ~~HIGH~~ | ✅ Accept as parameter |
| 2 | ~~Platform~~ | ~~installer.rs:39, ops.rs:211~~ | ~~HIGH~~ | ✅ DAG input port |
| 3 | ~~Env vars (auth)~~ | ~~executor.rs:81,97~~ | ~~HIGH~~ | ✅ Resolve before executor |
| 4 | Env vars (codegen) | cli_gen.rs:724,748,767 | MEDIUM | Accept env dict param |
| 5 | ~~SystemTime~~ | ~~gist-ops/lib.rs:145~~ | ~~HIGH~~ | ✅ Accept as parameter |
| 6 | Env vars (CI detect) | provider.rs:79-99 | LOW | Already at boundary |
| 7 | which command | cli.rs:260-281 | MEDIUM | Trait-based resolver |

## Tasks

- [x] Item 1: Refactor `sanitize_branch_for_filename` to accept `&FilesystemHandle`
- [x] Item 2: Add `platform` input port to GenerateScripts DAG node
- [x] Item 3: Resolve `AuthMethod::EnvVar` to `AuthMethod::Bearer` at DAG boundary
- [ ] Item 4: Generate CI runner functions that accept env dict parameter
- [x] Item 5: Add `SystemTime` parameter to `generate_gist_filename`
- [ ] Item 7: Abstract tool path resolution behind a trait

## Notes

- Item 6 is already correct — just documenting it for completeness.
- Items 1 and 5 are in the same file and can be done together.
- Items 2 and 1 follow the same pattern: the fix is "acquire at the
  boundary, pass down." Once we have a convention for how DAG nodes
  receive system resources (environment handle? context object?), all
  of these become mechanical.
- The deeper question is: where does the environment handle live? Options:
  - A dedicated `EnvOp` node per resource type (current pattern for tools)
  - A single "context" input that carries platform, filesystem, clock, env
  - Implicit thread-local context (anti-pattern — avoid)
