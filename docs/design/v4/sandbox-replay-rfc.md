# RFC: Sandbox Execution + Durability/Replay for DAG Runtime

Status: Draft  
Track: Workflow/Runtime hardening (`B2.4`)  
Date: 2026-02-18

## Why

Parallel DAG execution is now resource-aware and deterministic at scheduling
boundaries, but we still need a first-class story for:

1. **Sandbox safety**: proving a workflow can run without unintended side effects.
2. **Durability/replay**: reproducing transport interactions for deterministic tests,
   retries, and incident debugging.

This RFC defines a phased architecture that keeps the existing DAG/runtime model
intact while adding explicit policy and persistence layers around transport I/O.

## Goals

- Run workflows in a **no-real-I/O sandbox mode** by default-denying boundary effects.
- Record boundary requests/responses as a replayable event log.
- Replay runs deterministically (same DAG + same replay log => same observable boundary outputs).
- Keep resource declarations (`res:*`) and execution modes (`verify` / `ensure`) central.

## Non-goals

- Full VM/container isolation in v1.
- Replacing existing boundary mock semantics.
- Persisting full in-memory node state snapshots for every node (boundary events only in v1).

## Current baseline

- Boundary effects are already centralized through transport requests/responses.
- DryRun/simulate modes support interception and deterministic mock injection.
- Runtime file guard and admission control enforce declared resource safety contracts.

Missing pieces:
- Unified policy object for deny/allow/record/replay decisions.
- Canonical replay event schema and storage.
- Retry semantics that consume replayed boundary outcomes.

---

## Part A — Sandbox execution model

### A1. Execution policy model

Introduce runtime `ExecutionPolicy` layered over `ExecutionMode`:

- `Real` (current behavior)
- `SandboxDeny` (deny all boundary effects unless explicitly allowlisted)
- `SandboxRecord` (allow + record)
- `SandboxReplay` (deny real I/O, satisfy from replay log)

`ExecutionMode` still governs whether boundaries are intercepted for test semantics;
`ExecutionPolicy` governs what boundary I/O is permitted at runtime.

### A2. Boundary admission decision

At each boundary node:

1. Derive boundary class (file/shell/http/tool/etc.).
2. Derive required resources from declared `res:*` inputs.
3. Evaluate policy:
   - deny (fail immediately with policy violation),
   - allow (execute transport),
   - replay (serve from replay log).

This keeps policy decisions explicit and colocated with existing boundary
classification logic.

### A3. Allowlist surface

Sandbox allowlists are declared by stable selectors:

- resource ID prefix (`file:`, `tool:`, `api:`),
- node ID pattern,
- transport class.

Policy config is resolved before execution starts and rendered in preflight/progress
output so CI logs show the active sandbox envelope.

### A4. Failure model

Sandbox-denied operations produce deterministic, structured errors:

- node id,
- boundary kind,
- requested resource(s),
- allowlist rule miss.

No best-effort fallback to real execution in sandbox modes.

---

## Part B — Durability / replay model

### B1. Replay log schema

Persist append-only NDJSON (one event per boundary interaction):

```json
{
  "run_id": "uuid",
  "seq": 12,
  "node_id": "verify_lint",
  "request_fingerprint": "sha256:...",
  "transport_kind": "shell",
  "request": { "...": "redacted-safe payload" },
  "response": { "...": "redacted-safe payload" },
  "started_at_ms": 1730000000000,
  "duration_ms": 412
}
```

Constraints:
- request/response payloads must pass existing secret-redaction policy before persistence,
- event order is the canonical sequence for replay matching.

### B2. Matching strategy

Replay lookup key:

`(node_id, transport_kind, request_fingerprint, occurrence_index)`

If no event matches in replay mode, fail hard (no implicit real-I/O fallback).

### B3. Durability semantics for retries

On retry:

- already-recorded successful boundary events can be replayed,
- missing events continue from the first unresolved boundary,
- deterministic test mode can assert complete replay coverage.

### B4. Storage lifecycle

- default path: `target/replay/<run-id>.ndjson`
- optional retention policy: keep N latest runs or explicit export artifact in CI.

---

## Security + privacy

- Never persist raw secrets; use redacted render path for persisted payloads.
- Replay files are local build artifacts by default and must be gitignored.
- CI upload of replay files is opt-in and restricted to failure/debug jobs.

## Rollout plan

### Phase 1 (MVP)
- Add `ExecutionPolicy` plumbing.
- Add `SandboxDeny` + `SandboxReplay` handling for boundary nodes.
- Add replay event schema + writer/reader primitives.

### Phase 2
- Integrate policy/replay config with CLI/workflow registry entry points.
- Add acceptance tests for deny/allow/replay behavior.

### Phase 3
- Add selective durability for retry orchestration and incident capture tooling.
- Evaluate optional OS-level isolation hardening (seccomp/container) for high-risk paths.

## Validation criteria

1. Sandbox deny mode blocks undeclared/unauthorized boundary I/O with deterministic errors.
2. Replay mode executes boundary-heavy DAGs with zero real transport calls.
3. Record+replay roundtrip yields stable boundary outputs across runs.
4. Secret redaction remains enforced in replay artifacts.

## Open questions

- Should replay matching be strict by sequence or allow node-local matching with stable fingerprints?
- Do we need per-resource TTL/expiry metadata in replay events?
- How should policy be expressed in DSL metadata vs CLI/runtime config?
