# Allocation integration checkpoint

Integration branch: `codex/allocation-demo`, isolated from the serving checkout.
The serving srv2 allocation page remains draft-only. No integration deployment,
secret creation, fleet mutation, guest boot, or production data migration has
been performed by this work.

The operator confirmed Google subject `116084671989231734979` for
`briansrls@gunb.ai`. `workspace_allocation_policy.workspace_operator_principal`
is the single enrollment authority. Access uses the existing fleet enrollment;
no browser-supplied owner or manually supplied guest private key is accepted.

## Implemented, awaiting full integration validation

- Durable owner requests, immutable reservation plans, exact 4/8 GiB guest grants,
  existing fabric broker selection, and generation-fenced hold acquisition.
- Workspace payload in the existing microVM lifecycle, separate from GitHub JIT.
- Existing fleet convergence plan/hash/lease/apply path, with a typed per-invocation
  launch directive; no independent controller or frontend state machine.
- Acquisition obligations, retirement, sanitation-gated release, and durable
  monetary settlement intent. Ambiguous cleanup retains capacity and owner quota.
- Cookie/header transport and existing Google session routes ported onto the
  integration base; session JSON adds a no-store CSRF-token response.
- Allocation JSON adapter with session/CSRF/owner admission, immutable retry lease,
  recorded-at metadata, and freshness/expiry-gated connection projection.
- Versioned task editorial events, comments/assignment history, byte-preserving
  pinned Git import, authored seed migration, and fabric CAS history.
- Existing tracker UI carried forward, retaining uniform recursive nodes and
  omitting redundant TYPE and separate task-count columns.
- Authenticated state namespace within the existing fabric storage service. Its
  objects and heads are separate from the legacy CI-writable namespace, and all
  production state clients use the served path, even on the storage host.
  Request MACs reuse the existing HMAC implementation with a distinct credential.
  Credential mint/readback and host custody reuse existing provisioning authorities.

## Verification obtained

- Existing authentication/session suite: 11 PASS, wrapper exit 0.
- Sized request witnesses: 4 PASS, copied dependency closure, 2 GiB limit.
- Browser JSON contract controls: 4 PASS, 32-module closure, 1 GiB limit.
- State MAC controls: 4 PASS, 67-module closure, 1 GiB limit; valid proof,
  expired/future proof, tampered payload, and unsigned request.
- Task editorial witnesses: 5 PASS; carrier pure witnesses: 7 PASS.
- Actual temporary-store carrier tests: 4 PASS plus migration recovery 1 PASS,
  each under 1 GiB. A six-step split-process sequence also passed. The shared
  closure traversal now stops when its pending set empties; its budget refusal
  and early-stop regression passed against actual objects.
- Private state namespace isolation: 2 PASS; exact tailnet route readback: 3 PASS.
- Rust cookie/body/header transport: 3 library tests PASS, exit 0.
- Approval-to-GCP admission utility: 3 PASS, exit 0; no live credential minted.
- Workspace readiness/recovery controls: 20 PASS. Integrated controller and
  preparation source typechecked; one of five runtime claims exposed a hash
  argument error, now corrected in source and awaiting rerun.

Dependency snapshots copy original source bytes and record their hashes. They
are focused validation, not substitutes for full landing checks. Qualified and
implicit references must be included when preparing a snapshot.

## Required before activation/completion

- Finish source integration and full type/witness checks, including socket/header
  transport, protected namespace isolation, owner-negative controls and CAS recovery.
- Build/pin/install the real workspace image and controller through administrator-
  owned convergence; test memory/CPU configuration and access bootstrap.
- Provision/read back the distinct state credential and private directories before
  migrating history or accepting requests. The desired secret pin is not evidence
  that provisioning has happened.
- Observe an eligible dedicated, fully budgeted slot; the designation roster is
  empty, deliberately. srv2 has no commissioned workspace substrate. Read-only
  srv1 inspection found slot13 failed, which is not evidence it can be reassigned.
- Check actual per-principal SSH forwarding and the complete client→host→guest
  access path. Host-originated SSH readiness alone does not establish ProxyJump.
- Wire real installed-image/SSH/tool observations into preparation, preserve the
  advanced released monetary account on reallocation, and activate HTTP routes.
- Execute real login→small request→reservation→SSH→release→larger reallocation,
  retry and short-expiry acceptance. Record guest/controller/cgroup memory,
  swap policy, request-to-ready and release-to-clean durations from actual clocks.

No end-to-end completion claim is made by the pure or temporary-store checks.

## Integration checkpoint after subject confirmation

The operator confirmed Google subject `116084671989231734979`; the explicit
workspace enrollment now uses it. Email is not an authorization identifier.

New verified checks: allocation enrollment/CSRF witness (1 pass, exit 0), exact
backend tailnet route readback (2 passes), and Rust cookie/header transport
`cargo check -p v1-compiler --bin gunbc -j 1` (exit 0). The runtime aggregate
was killed by the existing 6 GiB / zero-swap limit before verdicts; no runtime
pass is claimed. Its lanes are reducing closure size, not increasing limits.

The staged 8 GiB image was rebuilt under fakeroot after actual ext4 inspection
caught incorrect inode ownership. Corrected image root and sshd are UID/GID 0;
final measured SHA256 is in `workspace_artifact_policy` and the image build
receipt. It has not been installed or booted.

Read-only srv1 census: allocation and readiness directories absent, installed
controller revision still `cfb9ff80652fe0a74093f08cade64f59d9695266`, slot13 failed
with ReservationNotObserved. Runners01–12 active;13–50 masked. No eligible
workspace slot has been established. Initial commissioning and real network
sanitation observations remain required. State-secret provisioning now uses
the existing approval app and shared approval-to-GCP utility. Its scoped workload
credential producer remains unconnected; no operator token-file fallback is used.
The user reports broker PR #12444 installed on srv1 but still loading/typechecking
after six minutes. Its owner is measuring startup readiness; enrollment and live
approval validation wait for the broker to answer. Provisioning has not run.

Source routing now preserves issue detail/assignment/context/comment and the
existing draft allocation page. Added thin intent/list/detail/release adapters
use protected fabric storage and exact stable-principal policy. Source changes
are not deployed, route integration has not passed its combined suite, and the
page remains draft-only until convergence is commissioned.
