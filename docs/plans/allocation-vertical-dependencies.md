# First operational workspace allocation

Baseline: allocation PR #12465, `ada7daefc28696ce4ba242437cd26daf61bd0bab`.
This is a dependency and acceptance map, not a deployment receipt. Existing source
and focused tests do not establish any of the live milestones below.

## Commissioning dependencies

```mermaid
flowchart TD
  Q[Reviewed source and qualified integration checks]
  P[Authorized commissioning plan]
  S[Protected state and socket CAS replay / fencing contract]
  H[One exclusive budgeted slot and enrolled use policy]
  I[Pinned image and controller installed through convergence]
  G[Operator-context gateway producer with bounded refresh]
  C[Recoverable owner-ledger consumer and durable outcomes]
  A[Actual login and ownership authorization]
  W[Bounded acceptance campaign]
  E[Operational exposure]
  Q --> W
  P --> S
  P --> H
  H --> I
  H --> G
  S --> C
  S --> G
  I --> W
  G --> W
  C --> W
  A --> W
  W --> E
```

Reviewed, qualified code plus an authorized commissioning plan permits the bounded
acceptance run. Successful acceptance permits operational exposure. This does not
require a successful VM boot before installing the code needed for acceptance.

## Execution, retirement and reuse

```mermaid
flowchart TD
  D[Durable authenticated request]
  C[Restart-safe owner ledger rescan]
  L{Retired or expired?}
  U[Unplanned retirement: prove no acquisition obligation and settle]
  P[Current attributable offers and policy prepare exact plan]
  R[Route existing convergence to selected host]
  A[Acquire sized reservation]
  X{Expired before boot?}
  B[Boot exact image and incarnation]
  O[Maintained gateway and runtime readiness observations]
  SSH[Operator SSH verifies identity and resources]
  T[Explicit release or running expiry]
  REC[Recovery after boot failure or controller loss]
  S[Exact-incarnation sanitation]
  RI[Durable release intent]
  HR[Generation-fenced hold release]
  RG[Readiness generation advancement]
  OS[Owner and account settlement]
  N[Larger new allocation on the same physical slot]
  D --> C --> L
  L -->|yes, no plan| U
  L -->|yes, existing plan or acquisition| REC
  L -->|live| P --> R --> A --> X
  X -->|expired: zero launches| REC
  X -->|live| B --> O --> SSH --> T --> S
  B -->|failure or controller loss| REC
  REC -->|acquired hold| S
  REC -->|no hold| F[Require cancellation proof or queued-operation fence / drainage]
  F --> OS
  S --> RI --> HR --> RG --> OS --> N
  HR -->|crash before publication| REC
  O -->|producer stops or evidence expires| ST[Withdraw SSH advertising; retain reservation]
  ST --> O
```

The consumer reconciles retirement before requesting offers, including when no
eligible capacity exists. Every refusal, waiting state and recovery obligation
must remain owner-visible. A free hold read is not proof that an elected actor's
queued operation cannot later commit. Recovery of an existing release must not
release a later occupant or duplicate the account transition.

Gateway evidence currently expires after 60 seconds; page readiness after 30.
Wire `workspace_refresh_gateway_wet` in the actual operator access context and
maintain observations across boot and runtime. Never copy private keys into the
controller or turn a generic timeout into proof of forwarding permission.

Implementation can proceed before live prerequisites. No host is hardcoded in a
request; one commissioned candidate suffices for this operator-only first cut.

## Source audit: first missing join

`workspace_route_create` records intent. The existing
`observe_workspace_allocation_plan_request_wet` reads a request selected by
`GUNBC_WORKSPACE_REQUEST_ID` through `workspace_convergence_observe`: that requires
an already prepared allocation. It does not discover or prepare recorded requests.

`workspace_prepare_convergence_from_observations` can produce that prepared basis,
but its caller must provide `WorkspacePreparationEvidence` and
`WorkspacePreparationPolicy`. The consumer must join durable intent to these
authoritative observations, then enter the existing fleet plan/apply path. Adding
another HTTP effect or a second controller would not close this boundary correctly.

For the initial single authorized owner, discovery can read that owner's durable
ledger; it need not introduce a global owner index. Unreadable state must refuse,
not look empty. Preparation retries must reuse the recorded plan; retirement and
expiry must be reconciled before new preparation. An in-flight acquisition with
uncertain socket completion must retain its obligation.

## Evidence required at each join

| Join | Required evidence |
| --- | --- |
| Protected state | Authenticated write/read across process restart; unauthorized access refused; restricted socket integration; queued-operation fencing/drainage and replay evidence |
| Policy and slot | Actual approved provenance, account/use policy, exclusive full envelope, withdrawn competing purpose, initial sanitation; no floor fixtures |
| Cold gateway | Host access and forwarding eligibility without requiring a guest listener before boot |
| Durable request consumer | Recorded request discovered after restart; unreadable state refuses; retired/expired intent never launches; repeated convergence does not duplicate acquisition |
| Plan and apply | Owner revision, request, host, slot, generation, grant and artifact remain bound; stale plan refuses |
| Ready | Actual SSH to expected incarnation, guest identity and resource readback; stale observation withdraws readiness |
| Termination | Explicit release, prelaunch expiry and running expiry; observed cleanup before fenced release; uncertain cleanup holds quota |
| Reuse | Smaller allocation released, larger allocation boots on the same physical slot with changed grant and new incarnation; prior disposable contents absent; stale predecessor identity refused; POST retry returns original allocation |

The first live receipt must include guest RAM, attempt/controller limits and peaks,
swap policy, request-to-ready time and release-to-clean time. Keep the existing
memory ceilings. Diagnose the integrated validation memory failure separately;
it is not evidence for raising a production envelope.

Broad task-history migration, additional secret-consumer conversions, and GitHub
runner completion are separate cutovers, not prerequisites for this first VM.

## Review 5333734576 gates

Source gates remain separate from live acceptance: restore the principal-mint
negative specimen and require a successful file read; test competing transaction
consumers; require the exact served above-cap refusal; obtain integrated checks.
The initial source repair adds the missing principal specimen and makes a missing
file fail the witness rather than enter the compiler census as empty content.

Acceptance must separately exercise lost POST response and consumer restart,
maintained readiness followed by producer loss, all three expiry windows, and
crashes around acquisition commit and release publication. No manual preparation
helper substitutes for the website-to-consumer route.

## Initial implementation checkpoint

`workspace_request_consumer` now rereads the owner's protected ledger and selects
idle, refusal, retirement, preparation, or routing. It refuses multiple outstanding
rows and unreadable storage; retired/expired requests do not request offers. This
is only discovery/dispatch, not the complete consumer: effectful settlement,
observed supply/policy, selected-host transport and durable outcome publication
remain unconnected. Nothing is installed or deployed.

Validation on this baseline with the existing serving binary and 6 GiB/no swap:
seven principal witnesses passed, then the restored negative specimen's compiler
census stopped at a module-index parse refusal in `dag/std/machine_constraints.dag`.
The consumer closure similarly refused its byte-identical copy of that module
before witness execution. Neither suite is claimed green. Four consumer controls
are authored for unreadable discovery, expired/unplanned retirement, explicit
retirement without supply/clock, and preserving the live request's lease.
