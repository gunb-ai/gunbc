# CI Timeout & Fermi Cost Estimation

## Status: Completed (moved to TODONE on 2026-02-14)

Core timeout enforcement and CI safety fixes are implemented. Remaining
follow-on architecture work (per-op fermi modeling and unified non-TTY
progress/reporting) is tracked in active workflow/logging docs.

## Problem

GitHub Actions defaults to 360 minutes (6 hours) if `timeout-minutes` is not set.
Our CI was running indefinitely at `preflight: lint-upsert (missing)` because:

1. **No workflow-level timeout** — GHA happily ran for 2+ hours with no kill
2. **No per-command timeout** — `ShellRequest` had no `timeout_ms` field;
   `execute_shell` called `wait_with_output()` which blocks forever
3. **stdin inheritance** — `execute_shell` didn't set `Stdio::null()` for
   commands without explicit stdin, causing potential deadlocks in CI

## What's Fixed

| Fix | Location | Scope |
|-----|----------|-------|
| `timeout-minutes: 30` in generated YAML | `RenderConfig.timeout_minutes`, `WorkflowConfig.timeout_minutes` | All generated CI workflows (GitHub Actions + GitLab CI) |
| `Stdio::null()` default | `executor.rs:execute_shell` | Every shell command (central function) |
| `ShellRequest.timeout_ms` field | `core/ir/src/transport/mod.rs` | Available on all shell requests |
| Default 5 min timeout (FermiCost::S) | `executor.rs:execute_shell` `DEFAULT_TIMEOUT_MS` | Every shell command, even without explicit timeout |
| `try_wait` + kill-on-expiry | `executor.rs:execute_shell` | Every shell command (always active) |
| `FermiCost::timeout_ms()` method | `core/test/src/fermi.rs` | Canonical timeout-per-cost-bucket mapping |

### How the Default Timeout Works

```text
execute_shell():
  timeout = request.timeout_ms           // caller-specified
           .unwrap_or(DEFAULT_TIMEOUT_MS) // 300_000 ms = FermiCost::S = 5 min
```

Every shell command now gets a 5-minute timeout by default. Callers that need
longer (e.g., `cargo build --release`) can set `ShellRequest::timeout(ms)`.

### FermiCost → Timeout Mapping

`core/test/src/fermi.rs` now has `FermiCost::timeout_ms()`:

```text
FermiCost::XS →  30 s
FermiCost::S  →   5 min  (DEFAULT_TIMEOUT_MS in executor)
FermiCost::M  →  10 min
FermiCost::L  →  30 min
FermiCost::XL →  60 min
```

## Deferred: Full Per-Operation Fermi Integration

The current implementation uses a flat default. Future work:

1. Have each DAG operation declare a `FermiCost`
2. Pass `FermiCost::timeout_ms()` into `ShellRequest::timeout()` at the call site
3. CI job-level `timeout-minutes` becomes `max(Σ(step fermi timeouts) × 1.5, 30)`
4. Move `FermiCost` to `gunbc-ir` so it's available in non-test crates without
   depending on `gunbc-test` (currently `executor.rs` uses a const mirror)

## Non-TTY Progress Reporting

Related problem: the progress graph works in TTY but not in CI (non-TTY).
The generic solution:

1. Detect `!atty::is(Stream::Stdout)` at the display layer
2. In non-TTY mode: emit `::group::` (GitHub) / section markers (GitLab)
   around each step, with periodic progress lines instead of cursor rewriting
3. This should be handled in the display/rendering layer, not per-callsite

The CI provider abstraction (`CiProvider::format()`) already exists for this.
The gap is that the progress graph doesn't use it in non-TTY mode.
