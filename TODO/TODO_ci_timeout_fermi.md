# CI Timeout & Fermi Cost Estimation

## Status: Design / TODO

## Problem

GitHub Actions defaults to 360 minutes (6 hours) if `timeout-minutes` is not set.
Our CI was running indefinitely at `preflight: lint-upsert (missing)` because:

1. **No workflow-level timeout** — GHA happily ran for 2+ hours with no kill
2. **No per-command timeout** — `ShellRequest` had no `timeout_ms` field;
   `execute_shell` called `wait_with_output()` which blocks forever
3. **stdin inheritance** — `execute_shell` didn't set `Stdio::null()` for
   commands without explicit stdin, causing potential deadlocks in CI

## What's Already Fixed

| Fix | Location | Scope |
|-----|----------|-------|
| `timeout-minutes: 30` in generated YAML | `RenderConfig.timeout_minutes`, `WorkflowConfig.timeout_minutes` | All generated CI workflows (GitHub Actions + GitLab CI) |
| `Stdio::null()` default | `executor.rs:execute_shell` | Every shell command (central function) |
| `ShellRequest.timeout_ms` field | `core/ir/src/transport/mod.rs` | Available on all shell requests |
| `try_wait` + kill-on-expiry | `executor.rs:execute_shell` | Every shell command with `timeout_ms` set |

## What's Missing: Default Timeout Policy

The `timeout_ms` field on `ShellRequest` is opt-in (`None` = wait forever).
This means existing callers (preflight, pragma-lint, etc.) still have no timeout
unless they explicitly set one.

### Design Options

#### Option A: Global default in `execute_shell` (recommended)

Add a `DEFAULT_SHELL_TIMEOUT_MS` constant (e.g., 5 minutes = 300_000) that
applies when `timeout_ms` is `None`. Callers that truly need long-running
commands (e.g., full `cargo build --release`) can set an explicit higher value.

```text
execute_shell():
  timeout = request.timeout_ms
           .unwrap_or(DEFAULT_SHELL_TIMEOUT_MS)  // 5 min
```

**Pro**: Every caller gets a timeout for free. No code changes needed.
**Con**: Must audit callers that legitimately take > 5 min (cargo release builds).

#### Option B: Fermi-estimated timeout per operation

Each DAG operation declares a `FermiCost` (already in `core/test/src/fermi.rs`).
Map cost to timeout:

```text
FermiCost::XS → 30s
FermiCost::S  → 2 min
FermiCost::M  → 10 min
FermiCost::L  → 30 min
FermiCost::XL → 60 min
```

This extends the existing test-gating pattern to runtime execution.

**Pro**: Each operation gets a right-sized timeout. Matches the integration test pattern.
**Con**: Requires every operation to declare cost. Migration work.

#### Option C: Hybrid (recommended long-term)

1. Immediately: Apply Option A (global default) as a safety net
2. Incrementally: Have operations declare `FermiCost`, derive timeout from it
3. The global default becomes the fallback for operations that haven't declared cost yet

### Implementation Steps

1. Add `const DEFAULT_SHELL_TIMEOUT_MS: u64 = 300_000` in `executor.rs`
2. Apply as fallback in `execute_shell` when `timeout_ms` is `None`
3. Extend `FermiCost` with `fn timeout_ms(&self) -> u64` method
4. Have preflight commands use `FermiCost::M` → 10 min timeout
5. Have cargo build/test use `FermiCost::L` → 30 min timeout
6. CI job-level `timeout-minutes` becomes `max(sum_of_step_timeouts * 1.5, 30)`

## Fermi Cost for CI Job Estimation

The `timeout-minutes` on the CI job itself should also be estimated rather than
hardcoded. Following the integration test pattern:

```text
CI job timeout = Σ(step fermi timeouts) × safety_factor + overhead
```

Where:
- `safety_factor = 1.5` (account for cold caches, network variability)
- `overhead = 2 min` (checkout, env setup)
- Each step's fermi timeout comes from `FermiCost::timeout_ms()`

This gives us CI timeouts that automatically scale with the pipeline complexity
rather than being a magic number.

## Non-TTY Progress Reporting

Related problem: the progress graph works in TTY but not in CI (non-TTY).
The generic solution:

1. Detect `!atty::is(Stream::Stdout)` at the display layer
2. In non-TTY mode: emit `::group::` (GitHub) / section markers (GitLab)
   around each step, with periodic progress lines instead of cursor rewriting
3. This should be handled in the display/rendering layer, not per-callsite

The CI provider abstraction (`CiProvider::format()`) already exists for this.
The gap is that the progress graph doesn't use it in non-TTY mode.
