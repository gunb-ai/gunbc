# Provider control interface audit — typed census and source grounding

Status: **design note, no code lands from this note.** It is the durable research
handback for the operator directive of 2026-08-03 ("the control interface should
be selected deliberately"). Its job is to establish, per provider control
interface, what is *observed on this host* versus what is *not observed*, with a
typed authority on every row, so that the implementation slices that follow are
grounded rather than recalled.

Every claim below labelled OBSERVED was produced by executing the named command
in this worktree on 2026-08-03. Every claim labelled NOT OBSERVED is a typed
absence with a named acquisition trigger — never an invented shape (DESIGN §3,
§5).

---

## 0. The thesis this note is grounding

The provider control interface is a **selected realization**, not a fixed fact.
CLI + tmux is one realization — the compatibility one — and the roadmap's
provider recovery system must not be built on CLI text and process state.

"More stable" is a conjunction of decidable properties, not a preference:

- versioned machine-readable schema
- explicit run / turn / session / account identities
- typed terminal and refusal outcomes
- resumable event stream
- cancellation and follow-up operations
- structured usage and rate-limit observations
- an interface intended for application automation

It does **not** mean "cloud is always better" and does not mean the SDK is
bug-free. The provider interface is itself modeled, versioned, tested, and
receipted.

## 1. Authority is typed, never flattened

```
ProviderInterfaceAuthority
  = OfficialGeneratedSchema        -- generated from the shipped binary
  | OfficialSdkType                -- shipped .d.ts / type stub
  | OfficialImplementationSource
  | OfficialDocumentation
  | ExactLiveObservation           -- a captured transcript
  | ReverseEngineeredSource        -- decompiled / reconstructed archive
  | SyntheticProbe                 -- a deliberately provoked error
```

None of these replaces the others. A generated schema proves the *shape* the
binary can emit; only an `ExactLiveObservation` proves a shape was *actually*
emitted; only a `SyntheticProbe` proves a refusal path is reachable. A
`ReverseEngineeredSource` is admissible evidence of history and never the
current protocol authority.

## 2. Census

### 2.1 Codex `app-server` — GROUNDED, OfficialGeneratedSchema

| Axis | Finding |
|---|---|
| Installed version | `codex-cli 0.146.0` (OBSERVED, `codex --version`) |
| Schema generation | `codex app-server generate-json-schema --out DIR`; also `generate-ts` (OBSERVED) |
| Schema size | 537 definitions in `codex_app_server_protocol.v2.schemas.json`; 82 in the v1 file (OBSERVED) |
| Transport | `--listen` accepts `stdio://` (default), `unix://`, `unix://PATH`, `ws://IP:PORT`, `off`; sibling subcommands `daemon`, `proxy` (OBSERVED, `--help`) |
| Session identity | `thread/start` → `ThreadStartResponse { thread, cwd, model, modelProvider, sandbox, approvalPolicy, approvalsReviewer }` |
| Turn identity | `turn/start` → `TurnStartResponse { turn }`; `Turn.id` documented as **UUIDv7** |
| Event protocol | JSON-RPC over the chosen transport: 90 `ClientRequest` methods, 70 `ServerNotification` methods, 10 `ServerRequest` methods (server→client, incl. approvals) |
| Terminal protocol | `TurnStatus = completed \| interrupted \| failed \| inProgress`; `Turn.error: TurnError?` populated only on `failed` |
| Typed error | `TurnError { message, additionalDetails?, codexErrorInfo? }` — the whole point of the directive |
| Retry signal | `ErrorNotification { threadId, turnId, error, willRetry }` — `willRetry` **required**, not optional |
| Cancellation | `turn/interrupt { threadId, turnId }` |
| Resumption | `thread/resume { threadId, ... }` — three documented modes (by threadId, by in-memory history, by rollout path) with a stated precedence and a consistency check when rejoining a *running* thread |
| Auth / account switch | `account/login/start`, `account/login/cancel`, `account/logout`, `account/read` |
| Usage / rate limits | `account/rateLimits/read`, `account/usage/read`, `account/rateLimitResetCredit/consume`; push notification `account/rateLimits/updated` |
| Workspace ownership | `ThreadStartParams.cwd` — belt-owned local worktree; no provider-managed workspace on this interface |
| Publication / artifacts | none — the interface operates on the caller's filesystem |

`CodexErrorInfo` is a real coproduct, not a string. Unit arms:
`contextWindowExceeded`, `sessionBudgetExceeded`, `usageLimitExceeded`,
`serverOverloaded`, `cyberPolicy`, `internalServerError`, `unauthorized`,
`badRequest`, `threadRollbackFailed`, `sandboxError`, `other`. Struct arms
(each carrying an optional `httpStatusCode`): `httpConnectionFailed`,
`responseStreamConnectionFailed`, `responseStreamDisconnected`,
`responseTooManyFailedAttempts`, plus `activeTurnNotSteerable { turnKind }`.

Rate limits are structurally richer than a percentage:
`GetAccountRateLimitsResponse { rateLimits, rateLimitResetCredits?,
rateLimitsByLimitId? }` — the last making the surface explicitly multi-bucket.
`RateLimitSnapshot` carries `primary`/`secondary` `RateLimitWindow
{ usedPercent, resetsAt?, windowDurationMins? }`, plus `planType`, `limitId`,
`limitName`, `individualLimit`, `credits`, `spendControlReached`, and
`rateLimitReachedType` — a five-value enum distinguishing workspace-owner from
workspace-member credit depletion from usage-limit-reached.

**Consequence, stated plainly: the Codex usage-limit prose parser is retired
work.** Every fact a parser would guess at is a typed field here.

**`Account` is the entitlement-realm evidence.** It is a coproduct
`apiKey | chatgpt { email, planType } | amazonBedrock { usesCodexManagedCredentials }`.
The same model name reached through these three arms consumes different limits
and different money. This is the upstream fact that grounds
`ProviderEntitlementRealm` — it is cited, not minted.

### 2.2 Codex `exec --json` — the fidelity-loss specimen

Retained as fallback realization, compatibility probe, and conformance
specimen. Its loss is exactly locatable in tree: `gunbc.roadmap_provider_events`
`provider_execution_state` maps the `exec` terminal to
`ProviderFailed { activity: "agent turn failed", detail: event.raw }` — the
structured category is discarded at that arm because `exec --json` never
carried one. That arm is the discriminating RED for slice 1: reconstructing a
recovery decision from `detail` alone must fail where the app-server row
succeeds.

### 2.3 Claude Agent SDK — NOT OBSERVED

`/usr/local/lib/node_modules/@anthropic-ai/` contains only `claude-code`
(2.1.220). There is no `claude-agent-sdk` package on this host, and
`claude-code` ships only `sdk-tools.d.ts`; a grep for
`RateLimitEvent|rate_limit_type|seven_day_overage|overage_status` returns
nothing.

Therefore the Agent SDK rate-limit and result shapes **cannot be cited from
this host** and must not be modeled from memory. The row lands as
authority-not-observed with acquisition trigger: *install
`@anthropic-ai/claude-agent-sdk` and regenerate this census from its shipped
types* (`OfficialSdkType`).

Standing modeling constraint from the directive, recorded here so it survives
the acquisition: the native rate-limit type is **open** —
`type ClaudeNativeRateLimitType = NonEmptyStr` with recognized projections
layered over it. Closing it as a memory-maintained enum would make an
unrecognized upstream value unrepresentable, which is the fail-open shape §5
forbids.

### 2.4 Claude subscription CLI — distinct realization, not a substitute

`claude-code` 2.1.220 is installed (OBSERVED). Its stream-JSON realization is
retained *because the entitlement realm differs*, not as a redundant path: the
subscription buckets (weekly / Fable) are not reachable through the Agent SDK's
realm. `ProviderEntitlementRealm` is what keeps these two from being treated as
interchangeable.

The circulating "Claude source-collection repository" at version 2.1.88 (March
2026) is `ReverseEngineeredSource` and is **older than the interface observed on
this host**. Its streamlined output mode intentionally drops `rate_limit_event`,
auth status, system events, and stream events, which makes it structurally
unsuitable as the evidence authority for exactly the facts this lane needs.

### 2.5 Claude Managed Agents — OfficialDocumentation

A hosted realization in a third entitlement realm: REST `/v1/agents`,
`/v1/sessions`, `/v1/environments`; beta header `managed-agents-2026-04-01`; SSE
event stream; session statuses idle/running/rescheduling/terminated;
`stop_reason.type` ∈ requires_action / retries_exhausted / end_turn; vaults,
deployments, self-hosted sandboxes. Modeled separately, later — it is
`ProviderManagedRepositoryWorkspace`-shaped, not a drop-in for local work.

### 2.6 Cursor — SDK and Cloud Agents NOT OBSERVED; CLI is a binary receipt only

`cursor-agent` exists only as a versioned tarball binary:
`/opt/cursor-agent-home/.local/share/cursor-agent/versions/2026.07.23-e383d2b/cursor-agent`
(OBSERVED). No Cursor SDK package and no type declarations exist anywhere on
this host.

This independently confirms the directive's claim: **the Cursor installer
provides no semantic contract.** It is useful for an artifact/version receipt
and nothing more. `CursorSdkLocal` and `CursorCloudAgent` land as
authority-not-observed rows with acquisition triggers.

## 3. Three findings that change the sequencing

**(a) Slice 1 is fully groundable today.** Codex app-server needs no
acquisition step; the schema is generated from the installed binary and every
field the recovery planner needs is present and typed.

**(b) Slices 2 and 3 need a provisioning step the directive does not
sequence.** Neither the Claude Agent SDK nor any Cursor SDK is installed. Those
slices either wait on an install, or land their interface facts as typed
not-yet-observed rows. Inventing the shapes is the failure mode both DESIGN §3
and §5 name.

**(c) A raw schema digest is a change detector, not a protocol identity.**
Regenerating the schema twice from the *same* binary produced two different
sha256 digests. Measured, not inferred: the two outputs are semantically equal,
the definition set is identical (537 in both), but key **order** diverges —
first divergence at index 527, `ClientInfo` versus `InitializeCapabilities`.
That is host map-iteration order leaking into the artifact, the same class as
this repository's own open determinism thread.

So `ProviderInterfaceArtifactReceipt.schema_digest` must be taken over a
**canonical key-sorted normalization** — a structural identity that changes when
the protocol changes. The raw bytes keep a *separate* digest in the private
evidence store, where variance is expected and harmless. A raw-bytes digest in
the selection receipt would fire on every regeneration while the protocol was
unchanged, which is precisely the §5 measurement-versus-measurement trap: an
oracle that is a copy of the thing it measures.

## 4. Selection is by required capability, never by one score

An interface is not ranked. A turn **declares what it needs**; an offer is
admissible only when it satisfies every required capability.

```
ProviderInterfaceCapabilities   -- typed event stream, resumption,
                                -- cancellation, structured usage,
                                -- account switching, artifact retrieval, ...
```

Ranking by a single score would let a missing hard requirement be bought back
by strength elsewhere. Requirement satisfaction is a conjunction; a missing
capability refuses.

Cloud agents are **not transparent substitutes** for local agents, so this is
not a Boolean `cloud: true`:

```
ProviderWorkspaceRealization
  = BeltOwnedLocalWorktree
  | ProviderManagedRepositoryWorkspace
  | ProviderManagedUploadedWorkspace

ProviderWorkspaceContinuation
  = ContinueInSameWorkspace
  | ContinueAfterCheckpointPush
  | ContinueAfterArtifactTransfer
  | FreshAttemptRequired
  | ContinuationUnsupported
```

Prefer a cloud agent when the work starts from an immutable remote repo/ref and
can return a branch, PR, or artifacts. Prefer local SDK when the agent must
continue in a belt-owned worktree holding unpublished changes.

## 5. Where this lands in the model

The control interface belongs in the **existing** selection receipt. There is
no second selection system. `gunbc.dispatch_selection`
`DispatchResolvedExecution` gains four fields:

- `entitlement_realm`
- `control_interface`
- `workspace_realization`
- `interface_version`

`ProviderEntitlementRealm` is load-bearing: the same model name reached through
a different realm consumes different limits and different money. Codex's own
`Account` coproduct (§2.1) is the upstream evidence that this axis is real.

Three receipt layers, kept separate because they answer different questions:

- `ProviderInterfaceArtifactReceipt` — which interface artifact, which version,
  which canonical digest
- `ProviderTerminalEvidenceReceipt` — what the provider actually said at the
  terminal
- `ProviderResponseCoverageRow` / `ProviderResponseCoverageStanding` — which
  response shapes we have observed and which remain unobserved

**The invariant across all three: Unknown must never silently become
`ProviderFailed`, `Authenticated`, `Retryable`, or `QuotaExhausted`.** An
unobserved shape is typed unobserved. This is the same rule
`extdeps.llm.cli_lifecycle` `cli_lifecycle_fact_separation_note` already states
for readiness, applied one layer up.

## 6. Evidence handling

Raw provider evidence stays in a **private evidence store**. It may contain
account email, repository content, prompts, tool output, internal paths,
provider identifiers, or secrets accidentally echoed by the provider. The public
repository receives a **sanitized fixture and its digest**, never the raw
personal transcript.

## 7. Sequence

One active PR plus at most one stacked successor. No parallel per-provider
error PRs.

1. Provider interface port + Codex app-server vertical slice.
   RED: replacing the typed `codexErrorInfo` with only the rendered message
   makes the specimen fail.
2. Claude Agent SDK realization + subscription compatibility *(gated on §3(b)
   acquisition)*.
3. Cursor SDK local + Cloud Agent realizations *(gated on §3(b) acquisition)*.
4. Provider-native terminal → business disposition → recovery planner.
5. Replace the tmux press surface.

Slice 5 is last for a reason: the tmux surface is load-bearing until the typed
interface actually carries the recovery decisions, and removing it earlier would
be a rung regression with no replacement.

---

## Open decisions for the operator

1. **Acquisition or typed absence** for slices 2 and 3 — install the Claude
   Agent SDK and a Cursor SDK on this host and widen the census first, or land
   those rows as not-yet-observed and unblock slice 1 immediately.
2. **Canonicalization form** for the schema digest — key-sorted JSON is the
   obvious normalization, but if the receipt is meant to survive a `generate-ts`
   switch it should be over the *definition set*, not the file.
