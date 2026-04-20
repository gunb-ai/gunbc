# P0-B — Unify REST_OPS authority + fix `CreateComment` drift `(M)`

## Context

Exploratory analysis found that `src/v2/tests/src/effects.rs:149-218` maintains a parallel REST operation table that duplicates facts already declared in extdep authorities (`dsl/extdeps/github/*.dag`, `dsl/extdeps/llm/*.dag`). **The duplication has already drifted into a factual error:**

- `src/v2/tests/src/effects.rs:166-170` says `CreateComment: POST /repos/{owner}/{repo}/pulls/{pull_number}/comments`
- `dsl/extdeps/github/pulls.dag:179-195` (the actual authority) says `POST /repos/{owner}/{repo}/issues/{issue_number}/comments`

The GitHub API comment-creation endpoint is on **issues**, not **pulls**. The test table is wrong. Any consumer reading it against the live GitHub API would issue a malformed request.

This is the strongest "same fact in two places" bug the analyses found — the parallel authority has already produced a live incorrectness.

## Read first

- `src/v2/tests/src/effects.rs:149-218` — the duplicated REST_OPS table
- `dsl/extdeps/github/pulls.dag:68-90, 112-126, 151-167, 179-195` — authoritative GitHub pulls operations
- `dsl/extdeps/github/gists.dag:50-69` — authoritative GitHub gists operations
- `dsl/extdeps/llm/anthropic.dag:105-125` — authoritative Anthropic operations
- `dsl/extdeps/llm/openai.dag:93-112, 130-147` — authoritative OpenAI operations
- `dsl/std/effects.dag` — how the effects system declares operations structurally
- Tests that consume `REST_OPS` — grep for usages and understand what facts they actually need

## Work

1. **Fix the CreateComment path first** (independent, small — resolves the drift before any refactor):
   - Update `src/v2/tests/src/effects.rs:166-170` to `issues/{issue_number}/comments`.
   - Commit this first. Honest bug-fix commit.
2. **Dissolve the parallel table**:
   - The extdeps already declare service operations with (name, method, path) as typed facts. The test should consume those directly, not maintain a parallel table.
   - Replace `REST_OPS` with a derivation: walk the extdep declarations and build the test table from them at test time. Or delete `REST_OPS` and have tests consume the extdep declarations directly.
   - If walking extdep declarations from Rust is awkward (because the substrate-reflection work isn't complete), propose a minimal adapter that reads the extdep `Operation` declarations and exposes them as Rust-side test fixtures — but flag this as a scaffold with a dissolution trigger (when substrate reflection lands, the adapter dissolves).
3. **Regression test**: one test that asserts `CreateComment` uses `/issues/` — locks in the fix.
4. **Per-operation check**: add a test (or extend an existing one) that walks all extdep operations and asserts no duplicate `(name, method, path)` tuple appears in `REST_OPS` with different values. This catches future drift structurally.

## Acceptance

- `CreateComment` path corrected to `/repos/{owner}/{repo}/issues/{issue_number}/comments`.
- `REST_OPS` either (a) derived from extdep declarations, or (b) deleted, with callers consuming the extdep facts directly.
- Zero duplicate declarations of the same operation across source (no fact in two places).
- Regression test proves `CreateComment` can't drift again.
- Drift-detection test (or ratchet) prevents future fact duplication.

## STOP-AND-ESCALATE

- If the Rust-side test consumption of `.dag` extdep declarations requires substrate-reflection work that isn't landed (SG-3f territory) — STOP, surface the dependency, ship (1) fix-path-only as the immediate P0 and queue full dissolution behind SG-3f.
- If `REST_OPS` has callers that depend on fields beyond (name, method, path) — enumerate them, propose whether those fields should also move into the extdep declaration or stay test-side.
- If other drift is found while reading the duplicated table (other ops besides `CreateComment`), list them separately — fix the immediate bug, flag the others for followup.

## Non-goals

- No rewrite of `effects.rs` test module beyond removing the parallel table.
- No extdep reorganization — just consumption.
- No substrate-reflection work in this PR (defer to SG-3f).

## Size: M (small fix + medium dissolution + drift detector).
