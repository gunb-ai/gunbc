# Consolidation: Dependency Injection for System Resources

**Status**: In Progress
**Date**: 2026-02-04

System resources (filesystems, platform, environment, clock) should be
acquired at DAG boundaries and injected into business logic — not
constructed inline. This doc tracks every known violation.

The pattern: acquire a handle/capability at the boundary (EnvOp, main,
transport executor), then pass it down as a parameter or DAG input.

---

## 1. FilesystemHandle — constructed inline in gist-ops

**Where**: `lib/gist-ops/src/lib.rs:113`

```rust
pub fn sanitize_branch_for_filename(branch: &str) -> String {
    let fs = filename::FilesystemHandle::cross_platform(filename::Scope::Write);
    // ...
}
```

**Problem**: `sanitize_branch_for_filename` acquires a `FilesystemHandle`
inside its body. The handle should be acquired by the DAG node (or
environment) and passed in. Same for `generate_gist_filename` which
calls `sanitize_branch_for_filename` at line 144.

**Fix**: Accept `&FilesystemHandle` as a parameter. The `GistOps::PrepareRequest`
node should acquire (or receive) the handle and pass it through.

**Severity**: HIGH — this is the motivating example for this audit.

---

## 2. Installer::new() calls Platform::detect() inline

**Where**: `lib/tools/deps/src/installer.rs:39-42`

```rust
pub fn new() -> Self {
    Self {
        platform: Platform::detect(),
    }
}
```

**Called from**: `lib/tools/deps/src/ops.rs:211`

```rust
fn execute_generate_scripts(...) -> Result<...> {
    let installer = Installer::new();  // Platform::detect() hidden inside
    // ...
}
```

**Problem**: `execute_generate_scripts` is an `Executable` implementation
(pure business logic). It should receive `Platform` as a DAG input, not
detect it at execution time. `Installer::for_platform()` already exists
at line 46 — just need to wire it up.

**Fix**: Add a `platform` input port to the GenerateScripts node. The DAG
should acquire platform at the environment boundary and feed it in.

**Note**: `Platform::detect()` uses compile-time `cfg!` — same pattern we
already fixed in `filename.rs`. The platform is a property of the target
environment, not the build host.

**Severity**: HIGH — same class of problem as the filesystem detection hack.

---

## 3. std::env::var() in transport executor (auth resolution)

**Where**: `lib/transport/src/executor.rs:81,97`

```rust
gunbc_ir::transport::AuthMethod::EnvVar(var) => {
    if let Ok(token) = std::env::var(var) {        // line 81
        http_req.headers.insert("Authorization".into(), format!("Bearer {}", token));
    }
}
gunbc_ir::transport::AuthMethod::EnvVarHeader { header, env_var } => {
    if let Ok(value) = std::env::var(env_var) {    // line 97
        http_req.headers.insert(header.clone(), value);
    }
}
```

**Problem**: The transport executor reads env vars inline during request
execution. This makes it impossible to test auth handling without setting
real environment variables, and couples the executor to OS state.

**Fix**: The `AuthMethod::EnvVar` variant should be resolved *before*
reaching the executor — at the DAG boundary where secrets are acquired.
By the time the executor sees a request, all tokens should be concrete
`AuthMethod::Bearer(token)` values. Alternatively, pass an env-var
resolver function/trait to the executor.

**Severity**: HIGH — security-sensitive path, hard to test.

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

## 5. SystemTime::now() in gist filename generation

**Where**: `lib/gist-ops/src/lib.rs:145`

```rust
pub fn generate_gist_filename(branch: &str) -> String {
    let sanitized = sanitize_branch_for_filename(branch);
    let timestamp = format_utc_timestamp(SystemTime::now());
    format!("{}_{}.md", sanitized, timestamp)
}
```

**Problem**: Implicit dependency on the system clock. Tests can't verify
the timestamp format without race conditions, and the function is
non-deterministic.

**Fix**: Accept `SystemTime` as a parameter (or a clock trait). The DAG
node should capture "now" at the boundary and pass it in:

```rust
pub fn generate_gist_filename(branch: &str, now: SystemTime) -> String
```

**Severity**: HIGH — non-deterministic public API.

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
| 1 | FilesystemHandle | gist-ops/lib.rs:113 | HIGH | Accept as parameter |
| 2 | Platform | installer.rs:39, ops.rs:211 | HIGH | DAG input port |
| 3 | Env vars (auth) | executor.rs:81,97 | HIGH | Resolve before executor |
| 4 | Env vars (codegen) | cli_gen.rs:724,748,767 | MEDIUM | Accept env dict param |
| 5 | SystemTime | gist-ops/lib.rs:145 | HIGH | Accept as parameter |
| 6 | Env vars (CI detect) | provider.rs:79-99 | LOW | Already at boundary |
| 7 | which command | cli.rs:260-281 | MEDIUM | Trait-based resolver |

## Tasks

- [x] Item 1: Refactor `sanitize_branch_for_filename` to accept `&FilesystemHandle`
- [x] Item 2: Add `platform` input port to GenerateScripts DAG node
- [ ] Item 3: Resolve `AuthMethod::EnvVar` to `AuthMethod::Bearer` at DAG boundary
- [x] Item 4: Generate CI runner functions that accept env dict parameter
- [x] Item 5: Add `SystemTime` parameter to `generate_gist_filename`
- [x] Item 7: Abstract tool path resolution behind a trait

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
