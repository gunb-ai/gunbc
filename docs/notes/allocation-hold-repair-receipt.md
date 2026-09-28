# HOLD repair checkpoint

Review 5332745744 holds snapshot 9a17e6ed8b1c6df4d1a052a558d687f33d24c80d.
The hold remains in force: this checkpoint is not merge or deployment approval.

## Repairs

- Login sets a short-lived Secure, HttpOnly, SameSite=Lax `__Host-gunbc-login`
  cookie. Its opaque value is a domain-separated hash of the stored private PKCE
  verifier and state. The callback receives the raw request cookie and checks the
  binding before consumption or exchange. Consumption uses atomic create-new of
  a durable claim; an absent or previously claimed transaction cannot exchange.
- Event and migration-gate closure reads use the served protocol's single 65536
  bound authority. Above-cap requests still refuse; incomplete history still
  refuses. Nothing clamps a request into apparent completeness.
- The controller reconciles expiry against the durable owner ledger before
  deciding whether a committed reservation should launch or retire. Expiry uses
  the existing retirement path, including sanitation and generation-fenced release.
  Clock, state, or cleanup uncertainty retains capacity and quota.

## Executed checks

All local .dag runs used dependency snapshots copied byte-for-byte from this
worktree and bounded user scopes with zero swap. The maximum scope bound was
6 GiB. These are focused checks, not a green integrated floor.

- Browser-binding and transaction-store witnesses: 2 passed. Correct, missing,
  foreign, and duplicate cookie cases; real-file consume followed by replay.
- Full callback-handler regression: 1 passed. Missing and foreign browser cookies
  leave the stored transaction intact; the initiating cookie consumes it; a second
  already-read copy refuses before exchange. No Google code was supplied. This
  also exposed and repaired stale clock-operation and base64-carrier call sites
  in the integrated auth code.
- Actual authenticated served storage: empty history read, append, reread, and
  above-cap refusal passed. The server was stopped and restarted over the same
  temporary store; a fresh event-reader process reread the persisted event.
  The route verified a fixture HMAC key, not production credentials. The initial
  1 GiB server attempt exhausted its limit; the reduced server closure ran within
  the existing 6 GiB test bound. No production limit changed.
- Actual prelaunch controller path with expired durable reservation: 1 passed.
  It persisted retirement and retained owner quota when the hold observation was
  unavailable. This is the uncertain-cleanup negative control, not proof of
  successful host sanitation, reservation release, or subsequent reuse.

Integrated-floor work is still being validated. No actual Google login, concurrent browser acceptance, installed VM,
external SSH connection, positive teardown/reuse, or running-lease expiry is
claimed by this checkpoint.

## Integration scope

The previous floor exposed stale principal/UI witness imports, retired declaration
references, and two incomplete WorkflowAttemptEvidence constructors. These are
being aligned with the source already included in the draft. Missing capture and
integration observations remain explicitly unreadable, never fabricated success.

The first vertical still requires protected-state commissioning and its restricted
socket integration, an eligible budgeted slot, an explicit production request
consumer, and the executed login → allocation → SSH → retirement → reuse loop.
Broad task/Git migration and additional GCP consumer conversions remain separate
cutovers. The serving checkout and production fleet were not modified.

## Review commits and reproduction

- `9ee2f65fd2`: the three boundary repairs and focused regression entries.
- `7d79bfc3be`: witness/declaration alignment for the previously ported integration.

Focused entries:

```
dag/test/claim/auth/login_browser_binding_witness_test.dag
dag/test/claim/auth/login_callback_hold_witness_test.dag
dag/test/claim/workspace_prelaunch_expiry_hold_witness_test.dag
```

The served fixture is `test.manual.fabric_state_hold_server.served_storage_hold_handler`.
Bind it only to loopback. Set `GUNBC_HOLD_STORAGE_ROOT` to a fresh directory with
prefix `/tmp/gunbc-hold-storage-`; set `GUNBC_FABRIC_STATE_KEY_FILE` to a mode-0600
fixture MAC-key file for both server and clients. The default production key path
is unchanged. An invalid explicit path refuses instead of falling back.
Use `test.manual.roadmap_served_storage_hold_wet.served_storage_hold_check` with
`endpoint=http://127.0.0.1:<port>/fabric-storage/state` and sequential `step` values
`empty`, `append`, `read`, `above-cap`. Restart the server with the same root and
repeat `read`. The separate above-cap HTTP control returned 400 with
`closure bound out of range`.
