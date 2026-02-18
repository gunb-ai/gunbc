# URGENT: Consolidate DAG-Level Logging & Error Reporting

**Status**: Active
**Date**: 2026-02-13
**Priority**: High
**DSL Alignment**: Runtime observability hardening for DSL-generated workflows
**Track**: D — Runtime/Test Hardening

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
| Non-TTY observer summaries | 2026-02-14 | Non-interactive terminals now emit concise `→/✓/✗` progress lines + final summary instead of full per-node dumps |

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

**Status**: Fixed (2026-02-13)

`print_log_entry` now delegates to `print_value`. Both CI group logging and
classic display use the same code path with truncation and secret redaction.

The deeper issue (secret redaction depending on each call site rather than
the type system) is tracked in §8 below.

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

## 8. Secret Redaction Depends on Display Functions, Not the Type System

**Where**: `core/exec/src/display.rs` (`print_value`), `core/exec/src/execute.rs` (`print_log_entry`, now unified), any code that formats `Value` for output

**What happened**: `Value::Secret` exists as a dedicated enum variant — the type
system already distinguishes secrets from plain strings. Yet `print_value` was
missing a `Secret` arm entirely and would have fallen through to the `_ => {}`
catch-all, silently dropping the value. Meanwhile `print_log_entry` had a
manual `Secret => "***"` match. The fix (2026-02-13) added `Secret => "***"` to
`print_value` too, but this is still the wrong architecture.

**Why this is a problem**: Every function that ever renders, logs, serializes,
or formats a `Value` needs to remember to handle `Secret` correctly. This is
a classic "defense by convention" pattern that fails the moment someone writes
a new display path and forgets the `Secret` arm. The type system already has
the information — it should enforce redaction, not hope each call site does.

Today there are at least these output paths for `Value`:
- `print_value` (display.rs) — now redacts, didn't before
- `truncate_for_report` / `build_report_blocks` (ci/ops.rs) — receives
  strings, not `Value`; secrets would already be stringified by parse ops
- `StructuredRenderer` — receives `StructuredBlock::Raw(String)`, opaque
- `serde_json::to_string` in mock outputs — serializes `Value` directly
- Any future display/export path

**Root cause**: There is no single "render `Value` to displayable string"
chokepoint. Instead, each consumer pattern-matches on `Value` independently.

**Suggested fix**: Add a `Value::display_redacted(&self) -> String` method
(or a `Display` impl that always redacts secrets) as the **only** sanctioned
way to produce human-visible output from a `Value`. All display/log paths
should call this method rather than matching on variants directly. The `Secret`
variant's inner value should only be extractable via an explicit
`Value::expose_secret()` method that makes the intent clear in code review.

This would turn "every call site must remember to redact" into "you can't
accidentally get the plaintext without calling a function that says `expose`
in its name."

---

## 9. Cross-Repo Target: Mirror `../gunb.ai` `terminal.progress`

The concrete target for this work is the behavior in:

- `../gunb.ai/tools/terminal/progress.go`
- `../gunb.ai/OaaS_v2/pkg/dag/progress.go`

That stack has one progress model used by both grouped and flat task flows,
with explicit APIs for task status, contention, captured output, and final
error rendering.

### Gap matrix (current `gunbc` vs target)

| Capability | `gunb.ai` today | `gunbc` today | Gap |
|------------|-----------------|---------------|-----|
| Live progress updates | Executor callbacks update display in real time (`SetTaskStatus`) | `run_with_progress` executes DAG first, then replays animation | No live view of actual runtime state |
| Grouped task progress | Native group model (`AddGroup`, counters, running labels) | DAG grid + legend only | Missing stage/task grouping UX |
| Nested detail expansion | `ExpandGroup` + auto-expand for long-running groups | None | Can't drill into noisy/slow stages on demand |
| Contention visibility | `SetTaskContention` (resource wait reasons) | None in observer/display model | No "what is blocked and why" visibility |
| Per-task captured output | `SetTaskOutput` + boxed failure output in `PrintFinal()` | Inline log printing (`print_value`) + truncation | Output is noisy and duplicated instead of failure-focused |
| Non-TTY summary mode | Deterministic status lines + final failure blocks | Classic mode prints every node output | Non-TTY logs are too verbose and hard to scan |
| Single execution/display path | `ExecuteWithProgress(opts)` drives one flow | CI and local split at binary level (`ci.rs`) | Behavior drifts across environments |

### Target contract for `gunbc`

1. One execution path with `DisplayConfig` selects surface (`TTY`, non-TTY, CI grouped) without changing execution logic.
2. Progress is live (observer-driven), not post-hoc replay.
3. Output policy is failure-first: keep per-node stdout/stderr attached to task state and render details only when failed (or explicit verbose mode).
4. Grouping is first-class (pipeline stages), with optional nested expansion for high-fanout groups.
5. CI and local terminals share the same progress model and summarization rules.
6. Verify/check modes still use the same display engine (different verbosity), not ad-hoc bypass paths.

---

## Refactoring Order (Parity-Oriented)

1. ~~**Immediate**: Unify `print_value` + `print_log_entry`~~ (done 2026-02-13)
2. **Immediate**: Replace replay-only progress with live observer-driven rendering.
3. **Immediate**: Introduce one `DisplayConfig` path used by local, CI, and verify/check modes.
4. **Immediate**: Secret redaction via `Value::display_redacted` chokepoint.
5. **Short-term**: Capture stdout + stderr for all CI stages (testgen, bootstrap, pragma, guardrail, verify).
6. **Short-term**: Add failure-first rendering (per-stage extractors + boxed failure details; avoid full-output dumps).
7. **Short-term**: Add grouped progress model for `gunbc` stages (and nested expansion hooks for noisy groups).
8. **Medium-term**: Surface resource contention/wait reasons in progress display.
9. **Medium-term**: Route preflight through the same display/grouping infrastructure.
10. **Long-term**: Standardize error field conventions across all ops.
11. **Long-term**: Keep CI/report output as summaries derived from shared task state, not separate ad-hoc formatters.

---

## Related

- `TODO_workflow_audit.md` — broader workflow consolidation plan
- `TODO_hacks.md` — other fallback/hack patterns
- PR #39 — `truncate_for_report` added as immediate mitigation

---

## Acceptance Criteria (Definition of Done)

This effort is done only when all items below are true.

### A. Unified execution/display path

- [x] Local, CI, and verify/check flows use one execution path configured by display settings (no environment-specific execution fork).
- [x] `ci.rs` no longer has separate CI/local execution logic that bypasses shared display behavior.
- [x] Verify/check modes no longer bypass display via direct `execute_with_mode_and_inputs` calls.

### B. Progress parity with `gunb.ai`

- [x] Progress is live (observer/event driven), not post-execution replay.
- [ ] Stage/task grouping exists for `gunbc` pipelines (at least CI stages).
- [x] Non-TTY mode emits concise status/progress summaries instead of full per-node output dumps.
- [ ] Long-running/noisy groups have an expansion path (or equivalent drill-down) without dumping everything by default.
- [x] Spinner behavior (TTY) is present and consistent across tools: running state, completion state, and failure state are visually distinct.

### C. Output quality and noise control

- [ ] Default output is failure-first: concise success path, detailed output only for failed stages/nodes.
- [ ] Raw stdout/stderr is not duplicated across execution logs, parse nodes, and final report.
- [ ] CI report and terminal summaries use one consistent truncation/summarization policy.
- [ ] Per-stage extractors exist for at least build, test, lint, and verify-style failures (with sensible fallback).

### D. Data capture completeness

- [ ] Parse ops for testgen/bootstrap/pragma/guardrail/verify capture both stdout and stderr.
- [ ] Report formatting can render failing stage output generically from structured stage fields.

### E. Secret safety

- [ ] Human-visible rendering of values goes through a redaction chokepoint (`Value::display_redacted` or equivalent).
- [ ] No display path can accidentally emit `Value::Secret` plaintext.
- [ ] CI masking still occurs for secret outputs.

### F. Preflight integration

- [ ] Preflight output is routed through the same display/grouping infrastructure (no standalone `println!/eprint!` progress stream).
- [ ] Preflight failures produce structured error output consistent with the rest of the pipeline.

### G. User attention and error UX parity

- [x] High-signal failures are rendered with explicit attention formatting (boxed sections / equivalent) in TTY mode.
- [x] Error detail rendering preserves a clear non-TTY fallback (plain but structured, not raw dumps).
- [ ] Attention-level messaging (error/warning/info notices) uses one shared formatting path, not per-tool ad-hoc printing.
- [x] Color semantics are consistent across status states (running, success, failure, dim/inactive) in TTY mode.

### H. End-to-end workflow adoption

- [x] All primary workflow binaries (`build`, `ci`, `codegen`, `testgen`, `makegen`, `bootstrap`, `pragma`) run through the shared progress/display path.
- [x] Generated CLI workflows also use the shared progress/display path (no generated bypass path).
- [ ] CI workflow execution and local execution both use the same underlying progress/event model, with only surface config differences.
- [ ] There are no remaining direct per-binary node-log print loops outside shared display infrastructure.

### I. Verification and regression tests

- [ ] Unit tests cover display configuration modes (TTY/non-TTY/CI) and secret redaction behavior.
- [ ] Golden/snapshot tests cover concise success output and failure detail output for terminal and CI text modes.
- [ ] A regression test reproduces the 2026-02-13 large-log failure shape and proves output stays readable (bounded + non-duplicated).
- [x] Existing tool flows (`build`, `ci`, `testgen`, `makegen`, `bootstrap`, `pragma`) pass with the new display path.
- [ ] End-to-end smoke coverage validates workflow UX parity in TTY and non-TTY modes for each primary binary.
