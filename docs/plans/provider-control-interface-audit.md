# Audit structured provider interfaces and lock the no-degradation migration

Status: **audit and decision receipt. No provider-interface `.dag` vocabulary
lands from this note** — the types sketched here land in the immediately
following Codex app-server PR, where the press, selection, event reader, receipt
writer, and UI consume them in the same change.

Bound in `gunbc.doc_graph_roots` as `plan/provider-control-interface-audit`.

Every claim labelled OBSERVED was produced by executing the named command on
2026-08-03. Package versions and integrity digests are recorded in §7 so the
inspected artifact is identifiable, not merely named.

---

## 1. The governing law: exact interface, or refuse

The first revision of this audit described CLI + tmux as "the compatibility
realization" and Codex `exec --json` as a "fallback realization." Both are
deleted. They were the absorbing fallback of DESIGN §5 wearing a migration
label: an execution that quietly proceeds with less evidence than the offer
promised, whose frequency of degradation is unobservable because degrading *is*
the success path.

> **A provider execution uses the exact interface admitted for that offer. If
> that interface is unavailable or lacks a required capability, dispatch
> refuses. It never silently substitutes a legacy interface.**

```dag
type ProviderInterfaceDirective
  = RequireProviderInterface { interface: ProviderControlInterface }

type ProviderInterfaceResolution
  = ProviderInterfaceResolved { binding: ProviderInterfaceBinding }
  | RequiredProviderInterfaceUnavailable {
      interface: ProviderControlInterface,
      reason: NonEmptyStr,
    }
  | RequiredProviderCapabilityMissing {
      interface: ProviderControlInterface,
      capability: ProviderInterfaceCapability,
    }
```

There is no arm named `Fallback` or `Degraded`, and no "use the old interface
when the new one is missing." Selecting a *different* provider, account, or
model is permitted only through an explicit, receipted recovery policy, and only
when the new offer still satisfies the **complete** execution contract. That is
a recovery transition, not a degradation.

**Legacy by explicit configuration, never by fallback.** An installation may
author `CodexExecJsonLegacy` as its configured interface. It then receives only
the capabilities that interface genuinely provides — work requiring structured
account limits, typed terminal causes, or resumable turns **refuses there**. A
legacy adapter is kept only when a real deployment owns it; otherwise it is
deleted at cutover and recovered from history if an actual compatibility
requirement appears.

Capability requirements are **conjunctive**. An interface is never ranked by a
score, because a score lets strength on one axis buy back a missing hard
requirement.

## 2. Source inspection is not host provisioning

The first revision treated "not installed on srv1" as "not groundable," and
proposed landing runtime typed-absence rows in place of finishing the census.
That conflated four distinct stages:

```
source inspected
→ artifact acquired and hashed
→ artifact provisioned on a runtime host
→ interface admitted by conformance
```

Research needs only the first two, in a disposable probe directory, with nothing
entering the srv1 dependency closure. Both SDKs were acquired and inspected that
way for this revision (§4, §6); the probe directory is discarded.

```dag
type ProviderInterfaceSourceReceipt
type ProviderInterfaceAcquisitionReceipt
type ProviderInterfaceProvisionReceipt
type ProviderInterfaceAdmissionReceipt
```

Only a `ProviderInterfaceAdmissionReceipt` may construct a live
`ProviderInterfaceOffer`. A research receipt may truthfully say
`ArtifactNotYetAcquired`; that absence must never enter `ProviderInventory` as
though it were an offer.

## 3. Codex app-server — OBSERVED, and the correct next interface

| Axis | Finding |
|---|---|
| Installed version | `codex-cli 0.146.0` |
| Schema generation | `codex app-server generate-json-schema --out DIR`; also `generate-ts` |
| Bundle | **37 JSON files**, not one; `codex_app_server_protocol.v2.schemas.json` holds 537 definitions, the v1 file 82 |
| Transport | `--listen` accepts `stdio://` (default), `unix://`, `unix://PATH`, `ws://IP:PORT`, `off`; sibling subcommands `daemon`, `proxy` |
| Thread identity | `thread/start` → `ThreadStartResponse { thread, cwd, model, modelProvider, sandbox, approvalPolicy, approvalsReviewer, instructionSources }` |
| Turn identity | `turn/start` → `TurnStartResponse { turn }`; `Turn.id` documented as UUIDv7 |
| Event protocol | JSON-RPC: 90 `ClientRequest` methods, 70 `ServerNotification` methods, 10 `ServerRequest` methods (server→client, incl. approvals) |
| Terminal protocol | `TurnStatus = completed \| interrupted \| failed \| inProgress`; `Turn.error: TurnError?` populated only on `failed`; `Turn` also carries `completedAt`, `durationMs` |
| Typed error | `TurnError { message, additionalDetails?, codexErrorInfo? }` |
| Retry signal | `ErrorNotification { threadId, turnId, error, willRetry }` — `willRetry` required |
| Cancellation | `turn/interrupt { threadId, turnId }` |
| Resumption | `thread/resume { threadId, ... }` — three modes (threadId, in-memory history, rollout path) with stated precedence, and a consistency check when rejoining a running thread |
| Auth / accounts | `account/login/start`, `account/login/cancel`, `account/logout`, `account/read` |
| Usage / limits | `account/rateLimits/read`, `account/usage/read`, `account/rateLimitResetCredit/consume`; push `account/rateLimits/updated` |
| Workspace | `ThreadStartParams.cwd` — belt-owned local worktree only |
| Artifacts | none; operates on the caller's filesystem |

`CodexErrorInfo` unit arms: `contextWindowExceeded`, `sessionBudgetExceeded`,
`usageLimitExceeded`, `serverOverloaded`, `cyberPolicy`, `internalServerError`,
`unauthorized`, `badRequest`, `threadRollbackFailed`, `sandboxError`, `other`.
Struct arms, each with optional `httpStatusCode`: `httpConnectionFailed`,
`responseStreamConnectionFailed`, `responseStreamDisconnected`,
`responseTooManyFailedAttempts`, plus `activeTurnNotSteerable { turnKind }`.

`GetAccountRateLimitsResponse { rateLimits, rateLimitResetCredits?,
rateLimitsByLimitId? }` is explicitly multi-bucket. `RateLimitSnapshot` carries
`primary`/`secondary` `RateLimitWindow { usedPercent, resetsAt?,
windowDurationMins? }`, plus `planType`, `limitId`, `limitName`,
`individualLimit`, `credits`, `spendControlReached`, and a five-value
`RateLimitReachedType`.

`Account = apiKey | chatgpt { email, planType } | amazonBedrock
{ usesCodexManagedCredentials }` is the **upstream evidence** that entitlement
is a real axis — cited, not minted.

### 3.1 Correction: app-server is stronger, but not lossless

The first revision claimed "every fact a prose parser would guess at is a typed
field here." That is too strong and is retracted.

Codex internally distinguishes `UsageLimitReached`, `QuotaExceeded`, and
`UsageNotIncluded`, and projects all three onto the single app-server arm
`usageLimitExceeded`. So the operation-level *business* cause is a join, not a
field read:

```
TurnError.codexErrorInfo
⋈ account/rateLimits/read
⋈ account identity and plan
⋈ native message / additionalDetails
⋈ process and control evidence
```

The prose parser retires as the **policy authority**. The raw message stays as
evidence, because it may preserve detail the coarser app-server code dropped.

`ErrorNotification.willRetry` is likewise a **provider-native observation** —
"Codex intends to retry internally" — not the belt's retry policy. Preserve it,
display it, and wait for the final turn state. Do not derive "retry the same
provider" or "do not switch provider" from that Boolean.

### 3.2 Correction: unknown cause versus terminal disposition

The first revision wrote that Unknown must never become `ProviderFailed`. That
conflated two different facts. A turn can genuinely be terminally failed while
its cause is unclassified. The corrected invariant:

> An unknown cause may project to terminal failure, but it must remain
> explicitly unclassified and **cannot authorize automatic recovery**.

```dag
ProviderTurnFailed { cause: ProviderNativeCauseUnknown { evidence: ... } }
```

What must never happen is an unknown response silently acquiring a *classified*
cause — `Authenticated`, `Retryable`, `QuotaExhausted` — because those authorize
action.

## 4. Claude — the entitlement claim is retracted, and a probe replaces it

The first revision asserted that the subscription weekly/Fable buckets are not
reachable through the Agent SDK's realm, and that direct stream-JSON must
therefore remain a distinct realization. **That was never established and is
retracted.**

Inspection of the acquired SDK (`@anthropic-ai/claude-agent-sdk` 0.3.220)
contradicts the assumption's premise. `sdk.mjs` spawns the Claude executable
with:

```
--output-format stream-json --verbose --input-format stream-json
```

inherits the process environment, and requires no separate mandatory API-key
argument. So an SDK run plausibly consumes the same local subscription login and
emits the same limit events. That is an inference from the implementation, so it
needs a live paired probe before becoming a model fact:

```
same Claude account state root · same model · same prompt
  direct claude stream-json   versus   Claude Agent SDK
```

Compare account/session identity, entitlement and rate-limit events, reset time,
Fable rejection, Opus warning, final result, and raw event preservation. If the
SDK preserves subscription behaviour, **direct CLI actuation dissolves** for that
realm. If it demonstrably cannot, the direct stream becomes an explicit
interface for that entitlement realm — not a fallback.

This is exactly why the axes must stay orthogonal. The control interface does
**not** determine the entitlement realm:

```
provider · control interface · credential binding
· entitlement subject · workspace realization · model
```

### 4.1 What the SDK does expose (OBSERVED, 0.3.220)

`SDKRateLimitEvent { type: 'rate_limit_event', rate_limit_info, uuid,
session_id }` carrying:

```
SDKRateLimitInfo {
  status: 'allowed' | 'allowed_warning' | 'rejected'
  resetsAt?, utilization?, surpassedThreshold?
  rateLimitType?: 'five_hour' | 'seven_day' | 'seven_day_opus'
                | 'seven_day_sonnet' | 'seven_day_overage_included' | 'overage'
  overageStatus?: 'allowed' | 'allowed_warning' | 'rejected'
  overageResetsAt?, overageDisabledReason?, isUsingOverage?, overageInUse?
  errorCode?: 'credits_required', canUserPurchaseCredits?, ...
}
```

Terminal facts: `SDKResultMessage = SDKResultSuccess | SDKResultError`, the
error subtype being `error_during_execution | error_max_turns |
error_max_budget_usd | error_max_structured_output_retries`. Assistant-level
errors are typed separately: `SDKAssistantMessageError =
authentication_failed | oauth_org_not_allowed | billing_error | rate_limit |
overloaded | invalid_request | model_not_found | server_error | unknown |
max_output_tokens`. `rate_limits_available` is documented false (and
`rate_limits` null) for API key, Bedrock, and Vertex — itself an entitlement
signal.

**Finding that cuts against an assumption in the review: version 0.3.220 has no
`raw` field on `SDKRateLimitInfo`.** A grep for a `raw` member across
`sdk.d.ts` returns nothing. The rate-limit surface is closed enums with no
escape hatch for an unrecognised upstream token. That *strengthens* the standing
rule rather than weakening it: gunbc must capture **beneath** the SDK parser
(raw subprocess stream), and the native limit token must be modelled open —
`ClaudeNativeRateLimitType = NonEmptyStr` with recognised projections layered
over it. The same applies to unknown top-level message types, which the SDK
parser drops.

## 5. Claude Managed Agents — a third realm, modelled later

REST `/v1/agents`, `/v1/sessions`, `/v1/environments`; beta header
`managed-agents-2026-04-01`; SSE stream; session statuses
idle/running/rescheduling/terminated; `stop_reason.type` ∈ requires_action /
retries_exhausted / end_turn; vaults, deployments, self-hosted sandboxes.
`ProviderManagedRepositoryWorkspace`-shaped; not a drop-in for local work.

## 6. Cursor — the SDK exists and was inspected

The first revision reported "no Cursor SDK anywhere," which was a statement
about srv1's disk, not about the world. The package is **`@cursor/sdk`**,
current version **1.0.26**, acquired and inspected in the probe directory.

Both modes are first-class in one API: `LocalAgentOptions` (with `cwd`,
`workspaceRef`, a local `store`, and local checkpoints) and `CloudAgentOptions`,
constructed through `createAgentPlatform` / `Agent.create`.

```
Run {
  id, requestId, agentId
  supports(op) / unsupportedReason(op)   // "stream" | "wait" | "cancel" | "conversation"
  stream(): AsyncGenerator<SDKMessage>
  wait(): Promise<RunResult>
  cancel(): Promise<void>
  status, onDidChangeStatus, usage, git, error, durationMs, createdAt
}
RunStatus       = "running" | "finished" | "error" | "cancelled"
RunResultStatus = Exclude<RunStatus, "running">
RunError        = { message, code? }
RunGitInfo      = { branches: [{ repoUrl, branch?, prUrl? }] }
```

`RunGitInfo.prUrl` is the publication/artifact axis for cloud runs, alongside
`SDKArtifact`. Errors are a typed class hierarchy —
`AuthenticationError`, `RateLimitError`, `ConfigurationError`, `AgentBusyError`,
`IntegrationNotConnectedError`, `NetworkError`, `AgentNotFoundError`,
`UnknownAgentError` — each carrying a stable backend `code`. Note that
`supports(operation)` makes capability presence a **queryable** property of the
run, which maps directly onto the conjunctive capability check of §1.

The `cursor-agent` tarball binary on srv1 (`2026.07.23-e383d2b`) remains what
the first revision said it was: an artifact/version receipt with no semantic
contract. It is a legacy interface by explicit configuration only, deleted from
the default inventory at cutover.

## 7. Evidence manifest

Codex protocol artifact, generated twice from the same binary:

```
binary                    codex-cli 0.146.0
command                   codex app-server generate-json-schema --out <DIR>
bundle                    37 .json files
raw digest A (v2 bundle)  211568ba09dae024da002c524714aa05d965e89013676f17e72450a1db5991fc
raw digest B (v2 bundle)  4d878774cdd94131971e5a18c539c58eb1a18f043e3dd31b68e08adb7cd30892
canonical bundle digest   f3319a0ec8bc64b2cb7d11ac379fee7bce81392e9373e62e3c2f02dad6b26c16   (A and B identical)
definitions               537 (v2) / 82 (v1), same set both runs
first order divergence    definition index 527: ClientInfo vs InitializeCapabilities
byte-stable files         36 of 37; only codex_app_server_protocol.v2.schemas.json varies
```

Acquired packages (npm registry, integrity as recorded in the probe lockfile):

```
@anthropic-ai/claude-agent-sdk  0.3.220
  sha512-glc7SdwPkOkLw8oxwLo9PKTdLJGqW/PIR4urWXFoRtX9YllwozsEVc5Tc1+EvLSkfrsxPJqQWqOgpjUOQXf1oA==
@anthropic-ai/sdk               0.115.0
  sha512-BJrFIVyjNuU8lfDyIJTvlRYzgQg+zEl78BxE7fq8esULsGz9IRQvGtW5spq3tydmtjQb/GFdooKGdGsetpx+lQ==
@cursor/sdk                     1.0.26
  sha512-dU3WpJwrxv8yoMjs0DxBgZr5btAJEO+NrvFutl08b6l2+jcuemfFWUlSPBQ2xZAH7zIZun+sqfHvjT5NxW8woQ==
```

Already present on srv1: `@anthropic-ai/claude-code` 2.1.220; `@openai/codex`
0.146.0; `cursor-agent` 2026.07.23-e383d2b.

Raw provider transcripts stay in the **private evidence store**: they may carry
account email, repository content, prompts, tool output, internal paths,
provider identifiers, or secrets echoed by the provider. The public repository
receives a sanitized fixture and its digest.

## 8. Protocol identity is the canonical full bundle

A definition-only digest would miss root request/notification unions, method
maps, required top-level members, the schema entrypoint, and — measured here —
**36 of the 37 files in the bundle**. The identity is:

```
all intended JSON schema files
→ parse JSON
→ refuse duplicate object keys
→ recursively sort object members
→ preserve array order
→ include normalized relative file names
→ canonical serialize
→ hash
```

```dag
type ProviderProtocolArtifactIdentity {
  canonical_bundle_digest: ContentHash
  raw_artifact_digests: List<ContentHash>
}
```

Proven by execution in §7: the canonical digest is **identical across both
generations** while the raw digest of the v2 file differs. The raw digests are
retained separately as artifact observations, where variance is expected.

This digest deliberately does **not** attempt to survive a future switch from
JSON Schema to generated TypeScript — those are different representations. If
cross-format semantic identity becomes necessary, a later
`ProviderProtocolSurfaceIdentity` derives from a typed method/event/type
relation.

Implementation note: `main` now carries a real JSON parser that retains object
members as a list and treats duplicate names as an explicit condition, and an
emitter that preserves authored member order. That is sufficient substrate to
implement this canonicalizer in `.dag` rather than shelling through `jq -S`.

## 9. Model shape: one nested binding, not four copied fields

The first revision proposed adding `entitlement_realm`, `control_interface`,
`workspace_realization`, and `interface_version` directly to
`DispatchResolvedExecution`. That would fork immediately against what `main`
already carries — `ProviderKind` fuses provider identity to the CLI interface
(`ClaudeCliProvider`, `CodexCliProvider`, `CursorCliProvider`), and
`ProviderInstance` already holds `kind`, `executable`, `observed_version`
beside `DispatchResolvedExecution.provider_version`. A receipt could then say
`kind = CodexCliProvider` while `control_interface = CodexAppServer`, and
`provider_version` while `interface_version` — four witnesses for facts that can
disagree.

Split provider identity from interface identity and bind them once:

```dag
type ProviderKind = ClaudeProvider | CodexProvider | CursorProvider

type CodexControlInterface  = CodexAppServer | CodexExecJsonLegacy
type ClaudeControlInterface = ClaudeAgentSdk | ClaudeDirectStreamJson
type CursorControlInterface = CursorSdkLocal | CursorSdkCloud
                            | CursorDirectStreamJsonLegacy

type ProviderControlInterface
  = CodexInterface  { interface: CodexControlInterface }
  | ClaudeInterface { interface: ClaudeControlInterface }
  | CursorInterface { interface: CursorControlInterface }

type ProviderInterfaceArtifactIdentity {
  interface: ProviderControlInterface
  artifact_digest: ContentHash
  artifact_version: NonEmptyStr
  protocol_identity: ProviderProtocolIdentity
}

type ProviderInterfaceBinding {
  provider: ProviderKind
  artifact: ProviderInterfaceArtifactIdentity
  credential: DispatchCredentialBindingIdentity
  entitlement_subject: ProviderEntitlementSubject
  workspace: ProviderWorkspaceBinding
}

type DispatchResolvedExecution {
  interface_binding: ProviderInterfaceBinding
  model_identity: DispatchModelIdentity
  provider_control: DispatchProviderControl
  temporal_bounds: DispatchTemporalBounds
  usage_observations: DispatchUsageObservations
  attempt_identity: DispatchAttemptIdentity
}
```

One nested binding is the structural source. Its members are never copied into
sibling fields.

Workspace stays a realization, never a Boolean `cloud: true`:

```dag
type ProviderWorkspaceRealization
  = BeltOwnedLocalWorktree
  | ProviderManagedRepositoryWorkspace
  | ProviderManagedUploadedWorkspace

type ProviderWorkspaceContinuation
  = ContinueInSameWorkspace
  | ContinueAfterCheckpointPush
  | ContinueAfterArtifactTransfer
  | FreshAttemptRequired
  | ContinuationUnsupported
```

## 10. `ProviderReadiness` must decompose before Fable is modellable

`extdeps.llm.cli_lifecycle` `ProviderReadiness` is one coproduct spanning
installation, authentication, quota, rate limit, and request refusal. It cannot
represent the observed state:

```
Claude account authenticated
weekly Opus bucket allowed-warning at 95%
Fable-specific bucket rejected
Opus remains runnable
```

One arm necessarily erases the others. The successor separates:

```dag
type ProviderInstallationStanding
type ProviderAuthenticationStanding
type ProviderEntitlementObservation
type ProviderLimitObservation
type ProviderTurnOutcome
type ProviderToolEnvironmentStanding
```

and gives limits an explicit subject:

```dag
type ProviderLimitSubject
  = ProviderAccountLimit   { credential: DispatchCredentialBindingIdentity }
  | ProviderWorkspaceLimit { workspace_account: NonEmptyStr }
  | ProviderModelLimit     { credential: DispatchCredentialBindingIdentity,
                             model: DispatchModelIdentity }
  | ProviderNativeLimitBucket { credential: DispatchCredentialBindingIdentity,
                                native_key: NonEmptyStr }
```

The native token stays open. Recognised projections may classify `seven_day`,
`five_hour`, `seven_day_opus`, or Fable-related observations, but the original
token survives — which §4.1 shows is mandatory, since the SDK's own enums are
closed and carry no `raw`.

## 11. Authority versus observation method

The first revision proposed one enum mixing `OfficialGeneratedSchema`,
`OfficialSdkType`, `OfficialImplementationSource`, `OfficialDocumentation`,
`ExactLiveObservation`, `ReverseEngineeredSource`, `SyntheticProbe`. The last
two-and-a-bit are not authorities — `ExactLiveObservation` and `SyntheticProbe`
are observation *methods*. Fusing them repeats a fact-separation problem across
five axes: source authority, observation method, evidence provenance, evidence
fidelity, claim standing.

`main` already carries the general claim/evidence machinery — `RecordedFact`,
`EvidenceProvenance`, `EvidenceFidelity`, direction, freshness, inference rules,
insufficiency — and already uses it for provider-readiness observations. Reuse
it rather than minting a provider-only evidence ontology (DESIGN §3).

```dag
type ProviderInterfaceObservationMethod
  = InterfaceSourceInspection
  | InterfaceSchemaGeneration
  | PassiveLiveCapture
  | DeliberateSyntheticProbe

type ProviderInterfaceObservation {
  interface: ProviderInterfaceArtifactIdentity
  method: ProviderInterfaceObservationMethod
  native_fact: ProviderNativeFact
}
```

The official schema or SDK source is the cited external authority; the live
capture or probe is the recorded fact and its provenance.

The circulating Claude "source-collection repository" at 2.1.88 (March 2026) is
reverse-engineered, older than the interface installed on srv1, and its
streamlined mode drops `rate_limit_event`, auth status, system events, and
stream events. It is admissible as history, never as the protocol authority.

## 12. Sequence: vertical, one provider at a time

Adapter-first sequencing would land several more pieces of infrastructure while
the product still says "dispatch accepted" and does nothing — the live incident
where a dead session container satisfies the command. Producer vocabulary
without live wiring is the recurring failure mode. So each slice is one complete
trip.

**A. This PR** — audit and decision receipt. No provider-interface vocabulary.

**B. Codex app-server press trip.** The generic model lands only insofar as
Codex consumes it.

```
UI Start → subjectful command receipt
→ exact Codex app-server / account / interface binding
→ thread/start or thread/resume → turn/start
→ streamed typed events → exact terminal receipt
→ visible success / refusal / recovery state
```

Production transport is one supported local control endpoint per admitted
account binding — a supervised Unix socket. Stdio is for conformance tests. The
experimental WebSocket transport is not used.

Acceptance: no tmux session-name authority; no `codex exec --json` substitution
when app-server is absent; exact thread and turn IDs in receipts; raw provider
evidence retained before projection; a typed usage-limit specimen joined with
rate-limit state; unknown notifications retained; click feedback names the
command and turn immediately; a dead control process produces a located
mechanism refusal; srv1's Codex `exec`/tmux offer removed at cutover. The legacy
exec stream survives only as a checked-in fidelity-loss fixture — the loss is
locatable today at `gunbc.roadmap_provider_events` `provider_execution_state`,
whose terminal arm discards the category into `detail`.

**C. Claude Agent SDK press trip.** Run the §4 paired entitlement probe first.
Then: raw subprocess capture beneath the SDK parser, official typed parsing,
unknown top-level event preservation, Fable hard limit, weekly hard limit,
weekly warning, expired authentication, resume and cancellation, and the same
press/turn receipt contract as Codex. If the SDK carries subscription semantics,
delete direct Claude CLI actuation for that realm.

**D. First cross-provider recovery.** Codex turn 1 → exact quota/capacity
receipt → an explicitly admitted equivalent Claude offer → same attempt and
local worktree → Claude turn 2 → visible recovery receipt. Not a fallback: the
second offer must prove it preserves every required capability and workspace
condition. If no equivalent offer exists, wait for reset or escalate to the
operator. Never downgrade to a weaker legacy interface.

**E. Cursor SDK local**, same belt-owned worktree contract; delete the direct
Cursor CLI path from the default inventory at cutover.

**F. Cursor cloud**, a separate workspace realization, admissible only when work
starts from an exact remote ref or a modeled checkpoint makes the local work
product transferable. It never masquerades as an ordinary provider switch from a
dirty local worktree.

---

## Locked decisions

```
No automatic legacy fallback
No runtime offer from a typed research absence
Provider identity ≠ control interface
Control interface ≠ entitlement realm
One nested interface binding, not four copied fields
Independent installation / auth / limit / turn facts
Full-bundle canonical protocol identity
One provider completed vertically before the next adapter
Legacy support only by explicit installation choice with a real owner
```

## Open, and deliberately not decided here

The paired Claude entitlement probe (§4) is a **live experiment**, not a
modelling choice. Its outcome decides whether `ClaudeDirectStreamJson` survives
as an interface or dissolves. It runs at the head of slice C, and neither answer
is assumed by this note.
