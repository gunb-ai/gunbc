# Testgen Proof Analysis: Can Our Test Infrastructure Catch Auth Failures?

> **Date**: 2026-02-27
> **Context**: Follow-up to `TODO/gist-auth-postmortem.md`
> **Question**: If we intentionally break the gist auth flow, does testgen catch it?
> **Answer**: Partially. RT-I4 (shell exit code enforcement) catches gcloud failures immediately. But testgen's scenario tests (Bucket C) never exercise error paths because `auto_mock_spec` always produces success responses.

---

## Executive Summary

We ran three proof tests (`gunbc-dag/tests/shell_exit_enforcement_proof.rs`):

| Test | Result | What it proves |
|------|--------|----------------|
| `gcloud_exit_code_1_fails_with_error` | **PASS** | RT-I4 catches gcloud exit 1 (expired session) before the empty token reaches GitHub |
| `rest_401_response_surfaces_as_error` | **PASS** | REST 401 response produces a parse error (missing `html_url` field) |
| `auto_mock_spec_always_produces_success_responses` | **PASS** | Confirms the gap: all 7 transport mocks are success-only |

**Conclusion**: The execution engine now catches these failures (post-RT-I4). But testgen's auto-generated tests would not have caught them before the fix, because the mocks only inject success responses.

---

## Proof 1: RT-I4 Catches gcloud Exit Code 1

### Setup

A custom `TransportBackend` returns `ShellResponse::failed(1, "ERROR: ...")` for gcloud commands, simulating an expired gcloud session. All other operations (git, REST) succeed normally.

### Result

```
test gcloud_exit_code_1_fails_with_error ... ok
```

The test proves:
1. **Execution fails** with a descriptive error mentioning "exited with code 1"
2. **The REST endpoint is never called** — the failure stops propagation before an empty token can reach the GitHub API

### Before RT-I4 (the gap)

Before this fix, `GenericShellParseOp::TrimStdout` would:
1. Receive `ShellResponse { exit_code: 1, stdout: "", stderr: "ERROR: ..." }`
2. Ignore the exit code
3. Trim stdout → empty string `""`
4. Return `Value::Str("")` as the `secret_value`
5. The empty string flows to `github.Gist.Create` as `auth_token`
6. The HTTP request goes out with `Authorization: Bearer ` (empty)
7. GitHub returns 401

### After RT-I4 (the fix)

`GenericShellParseOp::TrimStdout` now:
1. Checks `shell.success()` → `false`
2. Checks if output type is optional (`T?`) → `Secret` is not optional
3. Returns `Err(shell_exit_error(...))` with "shell.GCloud.SecretManagerAccessVersion: shell command exited with code 1 (stderr: ERROR: ...)"

**Code**: `gunbc-dag/src/resolve_service.rs:545-562`

---

## Proof 2: REST 401 Surfaces as Error

### Setup

A custom `TransportBackend` returns `RestResponse::new(401, { "message": "Bad credentials" })` for the GitHub Gist API. The gcloud credential retrieval succeeds with a valid-looking token.

### Result

```
test rest_401_response_surfaces_as_error ... ok
```

The 401 response body contains `{ "message": "Bad credentials" }` which lacks the expected `html_url` field. The parse node (`GenericRestParseOp`) tries to extract `html_url` and fails.

### Caveat

This works by accident, not by design. The parse node doesn't check the HTTP status code — it just tries to extract fields from the response body. If the 401 response happened to contain an `html_url` field, the parse would "succeed" with garbage. Proper status-code checking in the REST parse layer remains a gap (see RT-I2 in `tasks.md`).

---

## Proof 3: auto_mock_spec Gap Confirmed

### Setup

Build the `gist_recent` graph and call `auto_mock_spec()` to generate the standard mock spec used by testgen.

### Result

```
=== auto_mock_spec gap analysis ===
Shell mocks (all exit 0): 6
  execute_transport_services_git_git_Core_Diff:response → exit 0
  execute_transport_services_git_git_Core_RevListBase:response → exit 0
  execute_transport_services_git_git_Core_CurrentBranch_c1:response → exit 0
  execute_transport_services_git_git_Core_CurrentBranch_c2:response → exit 0
  execute_transport_services_git_git_Core_Diff_c1:response → exit 0
  execute_transport_services_shell_shell_GCloud_SecretManagerAccessVersion_c2:response → exit 0
REST mocks (all status 200): 1
  execute_transport_services_github_gist_github_Gist_Create_c2:response → status 200
Total transport mocks: 7
Error scenario mocks: 0 (THIS IS THE GAP)
```

### What this means

- **7 transport mocks**, every one a success response
- **0 error scenario mocks** — testgen never exercises exit code 1, HTTP 401, HTTP 403, HTTP 500, or any other error condition
- The existing Bucket C `SingleTransportFailure` test only injects `Value::Str("<TRANSPORT_FAILURE>")` as a sentinel — this causes the DAG to skip the node entirely, but does not test realistic error responses

### Why this matters

The gist auth 401 bug lived in production because:
1. The happy path test (`gist_recent_regressions.rs`) used a custom backend that always returned 201
2. `auto_mock_spec` generated mocks with exit 0 and status 200
3. Testgen's Bucket C `SingleTransportFailure` injected `<TRANSPORT_FAILURE>` instead of realistic errors
4. No test ever ran the flow with a 401 or exit 1 response

**Result**: Five compounding gaps combined to produce a silent credential loss that no automated test caught.

---

## The Testgen Gap: Why Bucket C Doesn't Catch Real Failures

### Current Bucket C behavior

| Test | Mock Strategy | What it proves |
|------|--------------|----------------|
| `test_scenario_all_succeed` | All mocks → success | Happy path completes |
| `test_scenario_{node}_fails` | One mock → `<TRANSPORT_FAILURE>` | DAG doesn't crash when one transport is skipped |

### What Bucket C should also test

| Test | Mock Strategy | What it would prove |
|------|--------------|---------------------|
| `test_scenario_{node}_returns_401` | One mock → `RestResponse(401, ...)` | Auth errors propagate instead of being silently swallowed |
| `test_scenario_{node}_returns_exit_1` | One mock → `ShellResponse::failed(1, ...)` | Shell failures propagate instead of returning empty strings |
| `test_scenario_{node}_returns_403` | One mock → `RestResponse(403, ...)` | Permission errors are caught |
| `test_scenario_{node}_returns_500` | One mock → `RestResponse(500, ...)` | Server errors are caught |

### What's needed

The `@mock_response` annotation exists in the DSL AST (`MockResponseDef` in `daglang-syntax/src/lib.rs:587-591`) but is never populated:
- Parser initializes empty `Vec::new()` (no parsing logic implemented)
- Lowerer doesn't carry mock data forward
- Services have zero `@mock_response` annotations

**Infrastructure readiness**: The `error_cases()` trait method exists on `Mockable` (`core/test/src/mockable.rs:59`) but is never populated for services.

---

## Recommendations

### Immediate (no DSL changes needed)

1. **Extend `auto_mock_spec` with error variants** — For each transport mock, also generate a failure variant (exit 1 for shell, 401/500 for REST). This gives Bucket C realistic error mocks without requiring `@mock_response` annotations.

2. **Add status-code checking to REST parse** — `GenericRestParseOp` should check the HTTP status code before extracting fields. A 4xx/5xx response should be an error regardless of body content.

### Medium-term (DSL changes)

3. **Implement `@mock_response` parsing** — Wire the existing AST through the parser → lowerer → testgen pipeline. Each service operation gets annotated with success and error mock responses.

4. **Add `@mock_response` to all REST services** — All 29 REST operations in `dsl/services/` should declare their success and error response shapes.

### Long-term (systemic)

5. **Migrate gist auth to `credential_chain` pattern** — Use the existing `dsl/std/patterns.dag:credential_chain` instead of raw `shell.GCloud.SecretManagerAccessVersion`.

6. **Add application-managed auth** — Use `ensures/upserts` pattern for `gcloud auth login` so `make gist` handles expired sessions automatically.

---

## Test Output Reference

Full test file: `gunbc-dag/tests/shell_exit_enforcement_proof.rs`

```
running 3 tests
test auto_mock_spec_always_produces_success_responses ... ok
test gcloud_exit_code_1_fails_with_error ... ok
test rest_401_response_surfaces_as_error ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Files Referenced

| File | Role |
|------|------|
| `gunbc-dag/tests/shell_exit_enforcement_proof.rs` | Proof tests (new) |
| `gunbc-dag/src/resolve_service.rs:483-498` | `shell_exit_error()` helper (RT-I4) |
| `gunbc-dag/src/resolve_service.rs:523-528` | SplitLines exit code check (RT-I4) |
| `gunbc-dag/src/resolve_service.rs:545-562` | TrimStdout exit code check (RT-I4) |
| `gunbc-dag/src/mock_defaults.rs:145-147` | `default_shell_response()` — always exit 0 |
| `gunbc-dag/src/mock_defaults.rs:164-184` | `default_rest_response()` — always status 200 |
| `gunbc-dag/src/mock_defaults.rs:280-505` | `auto_mock_spec()` — iterative slot filling |
| `core/test/src/mock_spec.rs:416-423` | `TransportMock` struct |
| `core/test/src/mockable.rs:59` | `error_cases()` trait method (unused) |
| `core/codegen/src/testgen/obligation.rs:228-255` | Bucket C obligations |
| `core/daglang/daglang-syntax/src/lib.rs:587-591` | `MockResponseDef` AST (unparsed) |
| `TODO/gist-auth-postmortem.md` | Full postmortem with 5 compounding gaps |
