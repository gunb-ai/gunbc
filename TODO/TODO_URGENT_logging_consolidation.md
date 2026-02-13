# URGENT: Consolidate DAG-Level Logging & Error Reporting

**Status**: Active
**Date**: 2026-02-13
**Priority**: High

## Motivating Incident: CI Log Explosion (2026-02-13)

Two consecutive CI runs on PR #39 produced logs so massive they were
impractical to paste or read. Root-cause analysis follows.

### What happened

1. The CI runner ran out of disk space (`Free space left: 41 MB`), which
   caused the `rust-lld` linker to crash with `signal 7 [Bus error]` while
   linking `gunbc-deps`. This is an infra issue, not a code bug.

2. The build failure triggered the CI DAG's report node. The report dumped
   **the entire linker command line** (~4,000 chars of `.rlib` paths) into
   the `--- Build stderr ---` section of the report. This made the report
   itself unreadable.

3. Independently, the `execute_testgen` CI group contained the **full stdout
   of the testgen binary**. The testgen binary runs its own DAG via
   `execute_and_display` in classic mode, which prints every node's outputs.
   For 23 DAG targets with compare/execute nodes, this produced hundreds of
   lines of `[compare_*_content] skip: true / fresh: true` noise. The full
   generated test source code was also visible.

4. The `parse_build` CI group then **re-printed the entire build stderr**
   (including the linker command) via `print_log_entry`, which had no
   truncation at all. So the linker command appeared **three times**: once in
   the `execute_build` group, once in `parse_build`, and once in the report.

### Why it was so verbose

Three layers of the output system contributed, each independently too verbose:

| Layer | What it dumped | Why |
|-------|---------------|-----|
| `print_log_entry` (execute.rs) | Full stderr/stdout for every node in CI groups | No truncation, no line limit |
| `print_value` (display.rs) | Full stderr/stdout in classic mode | No truncation, no secret redaction |
| `build_report_blocks` (ci/ops.rs) | Full build stderr in final report | No truncation (fixed 2026-02-13 via `truncate_for_report`) |

### What was fixed

| Fix | Date | Scope |
|-----|------|-------|
| `truncate_for_report` in CI report | 2026-02-13 | Report sections capped at 60 lines, 500 chars/line |
| Unified `print_log_entry` → `print_value` | 2026-02-13 | Eliminated duplicate function; both CI and classic use same path |
| `truncate_log_value` for CI group output | 2026-02-13 | Multi-line values in CI groups capped at 40 lines (5 head + 35 tail) |
| Secret redaction in `print_value` | 2026-02-13 | `print_value` now redacts `Value::Secret` (was missing before) |

### What remains unfixed

- Testgen binary runs `execute_and_display` internally; its stdout is
  captured and re-printed by the CI DAG → double-printing of all node outputs
- No per-stage error extractors (like `extract_test_failures` for test output)
- Stderr not captured from all stages (testgen, bootstrap, pragma only
  capture stderr, not stdout)
- No single, configurable verbosity control across CI/local/progress modes

---

## Problem

Output logging, error reporting, and progress tracking are fragmented across
four different execution contexts with inconsistent behavior. Users see
different output depending on whether they're in CI, a TTY, a non-TTY
terminal, or verify mode. Stderr from some stages is silently discarded.

---

## 1. Four Execution Contexts, Four Output Paths

**Where**: `core/exec/src/execute.rs`, `core/exec/src/display.rs`, all `gunbc-dag/src/bin/*.rs`

**What happens**: There are four separate code paths that handle DAG output:

| Context | Entry point | What gets printed |
|---------|-------------|-------------------|
| CI with groups | `execute_with_mode_and_ci` | Every node's outputs via `print_log_entry`, inside CI groups |
| Local progress | `execute_and_display` (TTY) | Only boundary node outputs via `print_boundary_outputs` |
| Local classic | `execute_and_display` (no TTY) | Every node's outputs via `print_value` |
| Verify mode | `execute_with_mode_and_inputs` (direct) | Caller decides (varies per binary) |

**Why this is a problem**:
- CI dumps everything (too verbose), local progress shows almost nothing (too terse)
- Debugging failures locally in progress mode is hard because you can't see intermediate node outputs
- Each binary reimplements its own error handling for the verify path

**Suggested fix**: Unify into a single `ExecutionDisplay` trait with configurable verbosity.
Concrete levels: `Quiet` (exit code only), `Summary` (final report + failures),
`Verbose` (all node outputs), `CiGrouped` (verbose but collapsible). All binaries
should go through the same display path.

---

## 2. `print_value` vs `print_log_entry` — Duplicate Logic with Drift

**Where**:
- `core/exec/src/display.rs:289-332` — `print_value()`
- `core/exec/src/execute.rs:1560-1591` — `print_log_entry()`

**What happens**: Both functions format and print node outputs, but differ in:

| Aspect | `print_value` | `print_log_entry` |
|--------|---------------|-------------------|
| Long string truncation | 80 chars | 120 chars |
| Secret redaction | None | `Value::Secret(_) => "***"` |
| Used by | Classic display mode | CI group logging |

**Why this is a problem**: Security risk — `print_value` doesn't redact secrets.
Subtle formatting differences between CI and local. Any fix to one function is
easy to forget in the other.

**Suggested fix**: Extract a single `format_output_value(port, value, opts)` function
used by both. Options struct controls truncation width and redaction policy.

---

## 3. Inconsistent Stderr Capture Across CI Stages

**Where**: `gunbc-dag/src/ci/ops.rs` — parse operations for each stage

**What happens**: Some stages capture both stdout and stderr, others only stderr:

| Stage | stdout | stderr | Notes |
|-------|--------|--------|-------|
| Build | stdout + stderr | stderr | `build_stdout` exists but report ignores it |
| Test | stdout + stderr | stderr | `extract_test_failures` uses stdout |
| Lint | stdout + stderr | stderr | `lint_stdout` exists but report ignores it |
| Testgen | **missing** | stderr | stdout silently discarded |
| Bootstrap | **missing** | stderr | stdout silently discarded |
| Pragma | **missing** | stderr | stdout silently discarded |
| Guardrail | **missing** | stderr | stdout silently discarded |
| Verify | **missing** | stderr | stdout silently discarded |

**Why this is a problem**: If testgen/bootstrap/pragma/guardrail/verify write
diagnostic info to stdout (e.g., test names, progress, warnings), it's lost.
When debugging CI failures, users can't see what these stages actually printed.

**Suggested fix**: All parse operations should capture both stdout and stderr.
The report node should have a unified `format_stage_output(stage, stdout, stderr)`
that consistently presents output for any failing stage. This also makes it
possible to apply `extract_test_failures`-style summarization to any stage.

---

## 4. `ci.rs` Binary Has Completely Separate CI vs Local Paths

**Where**: `gunbc-dag/src/bin/ci.rs:160-210`

**What happens**: The CI binary detects whether it's running in CI and takes an
entirely different execution path:

```
if is_ci {
    execute_with_mode_and_ci(...)  // manual log iteration, manual exit code
} else {
    execute_and_display(...)       // uses display infrastructure
}
```

**Why this is a problem**:
- The CI path manually iterates log entries looking for `overall_success` — this
  logic is duplicated from what `execute_and_display` already does via its
  `success_port` parameter
- Changes to display/reporting only apply to one path
- The `report` node is special-cased to not get a CI group (`node_id.0 != "report"`)
  but `print_log_entry` is still called, so report output appears both as a raw
  log entry AND as the report — potential duplication

**Suggested fix**: Have a single execution path that accepts a `DisplayConfig`
struct. CI detection should only affect the display config (grouping, verbosity),
not the execution path.

---

## 5. Preflight Output Bypasses Everything

**Where**: `lib/transport/src/preflight.rs:52, 306-367`

**What happens**: Preflight runs *before* DAG execution and uses raw
`println!`/`eprint!`/`eprintln!` for progress and error reporting:

```rust
println!("preflight: lint-upsert ({})", state);
eprint!("  [{}/{}] {}...", i + 1, total, label);
eprintln!(" {:.1}s", start.elapsed().as_secs_f64());
```

**Why this is a problem**:
- Not wrapped in CI groups on GitHub Actions — preflight output appears as
  unstructured noise before the DAG log
- No progress observer integration — can't show preflight in the DAG progress view
- Errors are plain `format!()` strings with no structured fields
- If preflight fails in CI, there's no `::error::` annotation for GitHub

**Suggested fix**: Either:
- (a) Make preflight a DAG itself (preflight DAG feeds into main DAG), or
- (b) At minimum, accept a `CiContext` and wrap output in CI groups, and use
  `ci.error()` for failures

---

## 6. Report Node Gets Raw Unstructured Text

**Where**: `gunbc-dag/src/ci/ops.rs:971-1101` — `build_report_blocks`

**What happens**: The report node receives stderr as raw strings and formats them
into `StructuredBlock::Raw(...)`. The `truncate_for_report` function (added
2026-02-13) caps output at 60 lines / 500 chars per line, but the underlying
data is still unstructured text that varies wildly by stage.

**Why this is a problem**:
- Truncation is a band-aid — the real issue is that raw compiler/linker output
  shouldn't flow unfiltered into a summary report
- `extract_test_failures` exists for test output, but there's no equivalent for
  build errors (`extract_build_errors`), lint errors, etc.
- Different tooling (cargo build, cargo test, cargo clippy) has different output
  formats, but they're all treated as opaque strings

**Suggested fix**: Add per-stage error extractors:
- `extract_build_errors(stderr)` — pull `error[E...]` lines, filter linker noise
- `extract_lint_warnings(stderr)` — pull clippy warnings with file locations
- `extract_test_failures(stdout)` — already exists, good pattern to follow
- Generic fallback: last N lines of stderr if no structured extraction works

---

## 7. No Unified Error Field Convention

**Where**: Various ops across all DAGs

**What happens**: Different ops use different field names for status/error info:

| Pattern | Usage |
|---------|-------|
| `"report"` | Multi-line summary (CI report, build report) |
| `"message"` | Single-line status (CI status messages) |
| `"stderr"` / `"stdout"` | Raw command output |
| `"error"` | Rare — sometimes used for error details |
| `"skip_reason"` | Why a node was skipped |
| `"success"` / `"overall_success"` | Boolean status |

**Why this is a problem**: No convention for which field contains human-readable
error details. Tooling can't generically "find the error message" for a failed
node. The display layer can't highlight errors without knowing the field name
convention for each op.

**Suggested fix**: Adopt a convention. Every op that can fail should produce:
- `success: bool` — did it succeed?
- `error_summary: String` — one-line human-readable error (empty on success)
- `detail: String` — full output / diagnostic text (optional)

Then the display layer can generically show `error_summary` on failure without
knowing anything about the specific op.

---

## Refactoring Order

1. **Immediate**: Unify `print_value` + `print_log_entry` (fixes secret leak, removes drift)
2. **Short-term**: Add stdout capture to all CI stages (testgen, bootstrap, pragma, guardrail, verify)
3. **Short-term**: Add per-stage error extractors for the CI report
4. **Medium-term**: Unify CI vs local execution paths in `ci.rs`
5. **Medium-term**: Wrap preflight in CI groups
6. **Long-term**: Unified `ExecutionDisplay` trait with configurable verbosity
7. **Long-term**: Standardize error field conventions across all ops

---

## Related

- `TODO_workflow_audit.md` — broader workflow consolidation plan
- `TODO_hacks.md` — other fallback/hack patterns
- PR #39 — `truncate_for_report` added as immediate mitigation
