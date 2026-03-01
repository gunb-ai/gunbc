# Tasks

**Verification**: `cargo test --workspace` + `cargo clippy --all-targets -- -D warnings`
**Sizing**: S (<1 day), M (1-3 days), L (3-5 days), XL (5+ days)

---

## Operating Model: Blue Team / Red Team

Two blue lanes (independent), three red team workers (parallel), never blocking each other.

```
  BLUE TEAM — Advance                     RED TEAM — Harden (3 workers)
  ────────────────────────────            ──────────────────────────────
  Lane 1: SDLC Activation                 Worker A: Binary & Workflow Elimination
    BT1:16 → BT13:19 → ...cloud             A1:11 (delete bins, workflow → DSL)

  Lane 2: External Dependency Modeling     Worker B: Registry & Extern Deletion
    ED1:6 → ED7:13 → ED14:18 → ED19:21      B1:10 (registries → DSL data)

                                           Worker C: Compiler Pipeline Refactor
                                             C1:19 (lowerer, resolver, types)
```

### Protocols

**Independence**: Blue and Red touch different files. No merge conflicts.
Blue: `dsl/`, `gunbc-dag/src/workflow/`, `gunbc-dag/tests/sdlc_*`.
Red: `core/`, `gunbc-dag/src/resolve.rs`, `gunbc-dag/src/fidelity.rs`.

**Scouting**: Every PR includes a `Scouted:` line listing
opportunities for the other team discovered during implementation.
Add raw observations to the other team's **Unqueued** section — never
directly into their lane queues. The owning team triages and promotes.

**Refill**: When a lane has <3 pending items, the worker proposes
new items from codebase observation or horizon scanning. Anemic
queues are a bug — the point is to never run out of work.

**Non-blocking**: Red never blocks blue. If red team cleanup is
needed for blue team progress, blue does the minimum fix inline
and red cleans up later.

---

# BLUE TEAM — Advance

## SDLC Activation (Single Lane)

### The Scenario

A GitHub issue goes through the full lifecycle:

1. Someone creates issue with label `sdlc:idea`
2. Worker discovers issue → acquires claim → dispatches `handle_idea_to_design`
3. LLM generates design → posted as comment → labels transition to `sdlc:design`
4. Worker dispatches `handle_design_to_review` → LLM reviews → `sdlc:design-review`
5. `handle_review_to_accepted` → approves → `sdlc:accepted`
6. `handle_accepted_to_implementing` → spawns Codex agent on dedicated branch
7. Agent completes → `handle_implementing_to_code_review` → creates PR, LLM reviews diff
8. `handle_code_review_to_testing` → review approved → `sdlc:testing`
9. `handle_testing_to_done` → cargo test + clippy → merges PR if green
10. `handle_done` → closes issue

### Testing Levels

| Level | What | Profile | Transport | Proves |
|-------|------|---------|-----------|--------|
| L0 | Pipeline compiles | — | — | DSL modules resolve, type-check, lower |
| L1 | Hermetic scenario | unit_test | DryRun/stubs | Full idea→done with stubs; stage transitions, claim lifecycle, outcome recording |
| L2 | Per-stage handlers | unit_test | DryRun/stubs | Each of 8 handlers individually correct |
| L3 | Worker loop | unit_test | DryRun/stubs | Discover→claim→dispatch→record→release; replay-skip, retry, claim conflict |
| L4 | Local integration | local | Real GitHub + file | Single stage transition (idea→design) against real API |
| L5 | Full local scenario | local | Real GitHub + file | Complete idea→done on test repo, multiple worker invocations |
| L6 | Testgen | — | — | Auto-generated per-node and per-pair coverage for SDLC DAGs |
| L7 | CLI entrypoint | local | Real | `gunbc sdlc --profile local --repo owner/name` |
| L8 | Cloud deployment | cloud_run | GCS + PubSub | Multi-worker CAS, GCS stores, Cloud Run |

### Current State

**DSL functions are real** (~3,600 lines across 20 .dag files):
- Stage handlers: 8/8 with real service calls and logic (`funcs/sdlc_stages.dag`, 739 lines)
- Worker dispatch: full discover→claim→dispatch→record→release with replay-skip and retry (`funcs/sdlc_worker.dag`, 381 lines)
- Validation policy: 3 real gate fns with conditional logic (`funcs/sdlc_validation_runtime.dag`, 59 lines)
- Interfaces: 7 with 24 capabilities, Providers: 9, Profiles: 3
- DSL-level tests: 10+ test blocks defined in .dag files

**DSL gaps** (need authoring, not just Rust wiring):
- **Pipeline wiring**: `workflows/sdlc.dag` wired — `intake` (param gate), `worker` (calls `dispatch_sdlc()`), `report` (aggregates results). `sdlc_dispatch_runtime.dag` kept as compiled dispatch policy.
- **Dispatch runtime stubs**: `sdlc_dispatch_runtime.dag` has 6 fns that return hardcoded literals — zero conditional logic, zero service calls. Meanwhile `execute_stage()` in `sdlc_stages.dag` already routes correctly. Decision needed: delete dispatch_runtime (dead code?) or fill with real pre-check policy.
- **Transport declarations**: All 26 ops now have DSL transport blocks (github 14 REST, llm 2 REST, file stores 6 file, codex agent 4 shell). Rust `@file` backend still missing (RT3).

**Rust infrastructure** — better than previously assessed:
- Profile-aware compilation **works** — `build_dsl_graph_with_profile()` exists, generated tests pass with all 3 profiles for other modules.
- SubDag/Pipeline execution **works** — `SubDagDispatchOp`, `PipelineDispatchOp` are real implementations.
- Interface stub resolution **works** — unbound interfaces get stub transport, bound interfaces wire to providers.
- resolve.rs may need minimal wiring for SDLC module paths (Red backlog RF-RG2 generalizes this).
- No file transport resolver exists in Rust (`@file` annotation has no backend).
- No CLI entrypoint (catalog is manual — Red backlog RF-RG1).

**Key dependency**: L0–L3 use unit_test profile (all stubs, no transport needed).
L4+ needs transport on all services the local profile touches — BT6 handles this.

### Queue

| # | ID | Task | Level | Size | Status | Deps |
|---|-----|------|-------|------|--------|------|
| 1 | BT1 | **Compile SDLC pipeline.** `build_dsl_graph_with_profile("pipelines/sdlc.dag", "unit_test")` succeeds. Fix any resolve.rs gaps inline. | L0 | S | Done | — |
| 2 | BT2 | **Pipeline wiring.** Fill 3 empty stages in `workflows/sdlc.dag`: wire `intake`, `worker`, `report`. Decision: keep `sdlc_dispatch_runtime.dag` as compiled dispatch policy. | L0 | M | Done | BT1 |
| 3 | BT3 | **Hermetic scenario test.** unit_test profile, DryRun execution succeeds. Pipeline has substantial node count. | L1 | M | Done | BT2 |
| 4 | BT4 | **Per-stage handler tests.** Compilation + structural checks: execute_stage router, 8 handlers, interface stubs. DryRun deferred (scalar fan-in in standalone compilation). | L2 | M | Done | BT2 |
| 5 | BT5 | **Worker dispatch loop test.** Compilation + structural checks: dispatch_sdlc, claim lifecycle, discover, outcome ledger. DryRun deferred (scalar fan-in in standalone compilation). | L3 | S | Done | BT4 |
| 6 | BT6 | **Transport declarations for local profile.** 26 ops: github (14 REST), llm (2 REST), file stores (6 file), codex agent (4 shell). DSL transport blocks complete. Rust `@file` backend deferred to RT3. | — | L | Done | — |
| 7 | BT7 | **Local integration: single stage.** `#[ignore]` tests: local profile compilation + DryRun. Real API gated on `GITHUB_TOKEN`. | L4 | M | Done | BT5, BT6 |
| 8 | BT8 | **Full local scenario.** `#[ignore]` test: full lifecycle DryRun with local profile. Real execution needs RT3 (@file backend). | L5 | L | Done | BT7 |
| 9 | BT9 | **Testgen integration.** Auto-discovery verified: 5 SDLC modules discovered, 1400+ test fns generated. Verification test in `sdlc_testgen.rs`. | L6 | M | Done | BT2 |
| 10 | BT10 | **CLI entrypoint.** `gunbc-sdlc --profile --repo --issue --dry-run`. Binary registered in Cargo.toml, help + DryRun working. | L7 | S | Done | BT8 |
| 11 | BT-R1 | **SDLC review: testgen discovery fix.** `callable_count` replaces `func_count` — discovery now mirrors `module_has_callable_items()` canonical predicate. 59 modules discovered (up from ~29), 9,710 test fns generated. Deleted 493 lines of hand-written compensating tests (sdlc_handlers, sdlc_worker, sdlc_integration, sdlc_scenario). | — | M | Done | BT9 |
| 12 | BT11 | **SignalStore providers.** `pubsub_signal_store.dag` (cloud_run) + `file_signal_store.dag` (local). Transport blocks, test blocks, profile bindings. | L8 | M | Done | BT10 |
| 13 | BT12 | **ArtifactStore providers.** `gcs_artifact_store.dag` (cloud_run) + `inline_artifact_store.dag` (local). Transport blocks, test blocks, profile bindings. | L8 | M | Done | BT10 |
| 14 | BT-R2 | **SDLC review: provider completion.** Transport blocks on gcs_claim_store + gcs_outcome_ledger. Inline definitions extracted to provider files. Dead code deleted from pipelines/sdlc.dag. deploy.dag variable naming fixed. Profile bindings updated in sdlc.dag. | — | M | Done | BT11, BT12 |
| 15 | BT-R3 | **SDLC review: fix 3 execution gaps.** LLM mock responses (enriched `default_rest_response` with LLM-shaped fields), `navigate_json_path` `/` separator + array index support, auth credential embedding in `GenericRestPrepareOp`, `CallParamSourceOp` replaces `IdentityCallableOp` for param_source nodes, param_source propagation in sdlc.rs CLI. Design: `docs/design/mock-response-pipeline.md`. | — | M | Done | BT10 |
| 16 | BT-E1 | **Transport node deduplication.** `gunbc-sdlc --dry-run` fails at 408/494 nodes: `scalar input 'prepare_transport_...Anthropic_Messages.max_tokens' has multiple upstream edges`. Root cause: lowerer creates ONE shared transport triplet per service operation, but `endpoint_use_count` resets per module — callables in different modules both wire literal sources to the same prepare node's scalar port. Fix: moved `endpoint_use_count` to global scope across all modules. Cross-module regression test added. | L1 | M | Done | BT-R3 |

### Postmortem: Testgen Discovery Bug (BT-R1)

**Root cause**: `discover_compilable_modules()` in `dag_test_discovery.rs` used `func_count == 0` as a pre-filter, counting only `Item::FuncDef`. This silently dropped all modules that produce graphs via `pipeline`, `pattern`, or `fn` items.

**Impact**: 12+ modules invisible to testgen (all `dsl/workflows/*.dag`, all `dsl/pipelines/*.dag`). This forced 493 lines of compensating hand-written Rust tests across 4 files. The tests were structurally correct but redundant — they tested exactly what testgen would have auto-generated.

**Fix**: Renamed `func_count` to `callable_count`, broadened filter to match all 4 callable item types (`FnDef | FuncDef | PatternDef | PipelineDef`), mirroring the canonical `module_has_callable_items()` in `daglang-driver/src/lib.rs`. Cross-reference comment added to both locations.

**Prevention**: Comment in discovery code references `module_has_callable_items` as canonical source of truth. If a new callable item type is added to the AST, both must be updated. This postmortem documents the pattern for future audits.

**Audit result**: All other `Item::FuncDef`-specific filters in the codebase (20+ locations) were verified correct for their specific use cases (CLI param extraction, `declared_outputs` field access, etc.).

### Horizon (after BT10)

| ID | Task | Level | Size | Deps |
|----|------|-------|------|------|
| BT13 | GCP credential chaining (WIF OIDC exchange) | L8 | L | BT10 |
| BT14 | Cloud Run deployment DAG | L8 | L | BT11:13 |
| BT15 | Multi-worker CAS stress test (3 workers, exactly-once) | L8 | M | BT14 |
| BT16 | CI integration (hermetic + cloud smoke) | L8 | M | BT15 |
| BT17 | Agent provider: wire codex_agent.dag to real LLM | L5 | M | BT8 |
| BT18 | Credential provider: local keychain for tokens | L5 | M | BT8 |
| BT19 | Webhook-driven stage transitions | L8 | L | BT17 |
| CT-1 | **Contract IR.** Parse `@contract` annotations into `ContractObligation` structs in lowerer. Sequence/idempotency/destructive obligation types. Store in type registry alongside interface capabilities. Design: `docs/design/contract-testing.md` §Phase 1. | — | L | BT-R1 |
| CT-2 | **Contract test generation.** For each interface with `@contract`, testgen emits parameterized test suite. Suite takes `ServiceBinding` as input. Each obligation becomes a test case: setup → execute sequence → assert postcondition. | — | L | CT-1 |
| CT-3 | **Provider compliance wiring.** For each (profile, interface, provider) triple, instantiate CT-2 suite. Stub providers: fast/hermetic/always-run. Real providers: env-gated/integration profiles. Wire into existing PT-* infrastructure. | — | M | CT-2 |
| CT-4 | **Annotation cleanup (Category 3).** Delete metadata noise annotations (`@network`, `@credential`, `@external`, `@derived_from`, `@ledger`, ~30 uses) per `docs/design/modeling/annotation-to-dag-modeling.md` Category 3. | — | S | — |

**Deliverable**: `gunbc sdlc --profile local --repo owner/name` runs full lifecycle.
**Endstate**: SDLC on Cloud Run with GCS stores, PubSub signals, multi-worker CAS.

### Design References

| Document | What |
|----------|------|
| `docs/design/sdlc/mega-modeling-design.md` | Canonical architecture: 9 high-level boxes, core abstractions, canonical contracts, conformance model |
| `docs/design/sdlc/domain-modeling-comprehensive.md` | All domain objects, state machines, invariants |
| `docs/design/sdlc/e2e-gap-analysis.md` | Gap tracking (A–J). Header says "all resolved" but means DSL files exist — Rust can't execute them yet. Gaps C, F genuinely done in Rust. Gaps A, B, D, E, H, I, J resolved at DSL level only. Gap G (worker invokes compiled DAG) not done at all. |
| `docs/design/sdlc/implementation-roadmap.md` | Task breakdown and dependency graph |
| `docs/design/provider-contracts.md` | Provider response contracts: `@response` annotation, mandatory error modeling, testgen obligations, interface inheritance. Supersedes RT-I1/RT-I2. |

---

## External Dependency Modeling (Lane 2)

### The Principle

Every external system the SDLC scenario touches should be modeled as a
**tautological `extdeps/` declaration** — facts about what the system *is*,
not how we use it. These compose upward:

```
Layer 0: std/             — universal primitives (render, types, languages)
Layer 1: extdeps/         — "What is GCS?" "What is GitHub Issues?"
Layer 2: config/          — our repo's choices (which bucket, which branch)
Layer 3: services/        — transport-level wiring (REST endpoints, shell commands)
Layer 4: tools/           — executable workflows
```

**Why this matters**: the SDLC scenario touches 30+ external APIs across 3 cloud
providers, GitHub, 2 LLM providers, and CLI tools. Today `services/` defines
*how to talk to them* (transport blocks), but there is no *what are they* layer
underneath. Adding tautological extdeps models enables:

1. **Compiler-derived invariants** — `readonly`, `idempotent`, CAS preconditions
   flow from the extdeps model, not from per-operation annotations.
2. **Cross-provider abstraction grounding** — `infra/core.dag` defines
   `ObjectStorage` abstractly; extdeps models say *what specific behaviors*
   GCS/S3/Azure Blob exhibit for that interface.
3. **Auth model composition** — GCP OAuth2+WIF, AWS SigV4+AssumeRole,
   Azure Bearer+FederatedCredential are separate tautologies that the
   credential chain pattern composes.
4. **Test obligation derivation** — knowing an operation is `readonly` means
   testgen can skip write-side mocking; knowing it has CAS means testgen
   must generate conflict scenarios.

### Pattern to Follow

Each extdeps file answers "What is X?" with types + data, zero opinions:

```dag
module extdeps.cloud.gcp.storage

// "What is a GCS object?"
type Object {
  bucket: String
  key: String
  generation: Int          // monotonic version — the CAS primitive
  metageneration: Int
  content_type: String?
  size: Int
}

// "What is optimistic concurrency on GCS?"
type CasPrecondition {
  if_generation_match: Int?
  if_metageneration_match: Int?
}

// Behavioral facts
data get_is_readonly: Bool = true
data get_is_idempotent: Bool = true
data insert_is_idempotent: Bool = false  // unless if-generation-match: 0
```

This follows the established patterns:
- `extdeps/clippy.dag` — "What is Clippy?" (categories, config surface)
- `extdeps/make.dag` — "What is Make?" (targets, variables, sections)
- `extdeps/github_actions.dag` — "What is GitHub Actions?" (workflows, jobs, steps)
- `extdeps/yaml.dag` — "What is YAML?" (indent unit, kv separator, list prefix)

### File Layout

```
extdeps/
├── cloud/
│   ├── core.dag                — universal cloud concepts: Region, AuthScheme,
│   │                             ServiceEndpoint, RateLimit, IdempotencyToken
│   ├── gcp/
│   │   ├── core.dag            — "What is GCP?" project, SA, WIF, scopes
│   │   ├── iam.dag             — roles, bindings, impersonation
│   │   ├── secret_manager.dag  — secrets, versions, rotation
│   │   ├── storage.dag         — buckets, objects, generation-based CAS
│   │   ├── pubsub.dag          — topics, subscriptions, ack deadlines
│   │   ├── cloud_run.dag       — services, revisions, traffic, scaling
│   │   └── sts.dag             — token exchange, OIDC, subject types
│   ├── aws/
│   │   ├── core.dag            — "What is AWS?" ARNs, regions, SigV4
│   │   ├── iam.dag             — roles, policies, trust
│   │   ├── s3.dag              — buckets, objects, versioning
│   │   ├── lambda.dag          — functions, runtimes
│   │   ├── secrets_manager.dag — secrets, rotation
│   │   └── sqs.dag             — queues, messages, visibility timeout
│   └── azure/
│       ├── core.dag            — "What is Azure?" subscriptions, tenants
│       ├── identity.dag        — managed identities, RBAC
│       ├── blob_storage.dag    — containers, blobs, ETags
│       ├── container_apps.dag  — apps, revisions
│       ├── key_vault.dag       — secrets, certificates
│       └── service_bus.dag     — queues, topics, messages
├── github/
│   ├── core.dag                — "What is GitHub?" repos, auth, rate limits
│   ├── issues.dag              — states, labels, events, timeline
│   ├── pull_requests.dag       — reviews, checks, merge strategies
│   └── gists.dag               — files, versions
├── llm/
│   ├── core.dag                — "What is an LLM API?" messages, tokens, roles
│   ├── anthropic.dag           — models, message format, tool use
│   └── openai.dag              — models, chat completions, response format
├── git.dag                     — commits, branches, refs, merge strategies
└── cargo.dag                   — packages, targets, features, profiles
```

### Queue

Priority: what the SDLC scenario needs first.

| # | ID | Task | Size | Status | Deps |
|---|-----|------|------|--------|------|
| 1 | ED-1 | **`extdeps/cloud/core.dag`** — universal cloud concepts. `Region`, `AuthScheme` (Bearer/SigV4/ApiKey/OIDC), `ServiceEndpoint`, `RateLimit`, `Credential`, `IdempotencyToken`. Shared across GCP/AWS/Azure. | S | Done | — |
| 2 | ED-2 | **`extdeps/github/core.dag`** — "What is GitHub?" `Repository`, `User`, `RateLimit`, `AuthToken`, `ApiVersion`, `Pagination` (link-header cursor). Shared across Issues/PRs/Gists. | S | Done | — |
| 3 | ED-3 | **`extdeps/github/issues.dag`** — "What is a GitHub Issue?" `Issue`, `IssueState` (imported from `std.types`), `Label`, `IssueEvent`, `IssueComment`, `Timeline`. Event types as sum type. State machine: open → closed. | M | Done | ED-2 |
| 4 | ED-4 | **`extdeps/github/pull_requests.dag`** — "What is a PR?" `PullRequest`, `ReviewState`, `CheckStatus`, `MergeStrategy` (Merge/Squash/Rebase), `BranchProtection`. State machine: draft → open → review → merged/closed. | M | Done | ED-2 |
| 5 | ED-5 | **`extdeps/github/gists.dag`** — "What is a Gist?" `Gist`, `GistFile`, `GistVisibility` (Public/Secret). Minimal — gist is simpler than issues/PRs. | S | Done | ED-2 |
| 6 | ED-6 | **`extdeps/llm/core.dag`** — "What is an LLM API?" `Message`, `Role` (System/User/Assistant), `TokenUsage`, `StopReason`, `Temperature`, `MaxTokens`. Shared across Anthropic/OpenAI. | S | Done | — |
| 7 | ED-7 | **`extdeps/llm/anthropic.dag`** — "What is the Anthropic API?" `Model` (claude-4-sonnet/opus/haiku), `ContentBlock` (Text/ToolUse/ToolResult), `SystemPrompt`, `ThinkingConfig`. | S | Done | ED-6 |
| 8 | ED-8 | **`extdeps/llm/openai.dag`** — "What is the OpenAI API?" `Model` (gpt-4o/o1/o3), `ResponseFormat` (Text/JsonObject/JsonSchema), `ToolChoice`. | S | Done | ED-6 |
| 9 | ED-9 | **`extdeps/cloud/gcp/core.dag`** — "What is GCP?" `GcpProject`, `GcpServiceAccount`, `OAuth2Scope`, `WifPool`, `WifProvider`, `GcpApiEndpoint` pattern (`{service}.googleapis.com/v1`). | M | Done | ED-1 |
| 10 | ED-10 | **`extdeps/cloud/gcp/storage.dag`** — "What is GCS?" `Bucket`, `Object`, `StorageClass`, `CasPrecondition` (generation-based), `ObjectOp` behavioral properties. | M | Done | ED-9 |
| 11 | ED-11 | **`extdeps/cloud/gcp/pubsub.dag`** — "What is Pub/Sub?" `Topic`, `Subscription`, `AckDeadline`, `OrderingKey`, `DeliveryGuarantee` (AtLeastOnce). | M | Done | ED-9 |
| 12 | ED-12 | **`extdeps/cloud/gcp/iam.dag`** — "What is GCP IAM?" `Role`, `Binding`, `Policy`, `ImpersonationChain`, `TokenLifetime`. | S | Done | ED-9 |
| 13 | ED-13 | **`extdeps/cloud/gcp/secret_manager.dag`** — "What is Secret Manager?" `Secret`, `SecretVersion`, `RotationSchedule`, `AccessPolicy`. | S | Done | ED-9 |
| 14 | ED-14 | **`extdeps/cloud/gcp/cloud_run.dag`** — "What is Cloud Run?" `Service`, `Revision`, `TrafficSplit`, `ScalingConfig`, `ContainerPort`. | M | Done | ED-9 |
| 15 | ED-15 | **`extdeps/cloud/gcp/sts.dag`** — "What is STS?" `TokenExchange`, `SubjectTokenType` (JWT/AccessToken), `GrantType`. | S | Done | ED-9 |
| 16 | ED-16 | **`extdeps/git.dag`** — "What is Git?" `Commit`, `Branch`, `Remote`, `Ref`, `MergeStrategy`, `DiffStat`. | M | Done | — |
| 17 | ED-17 | **`extdeps/cargo.dag`** — "What is Cargo?" `CargoPackage`, `CargoTarget`, `CargoProfile` (dev/release), `CargoFeature`, `TestHarness`. | S | Done | — |
| 18 | ED-18 | **`extdeps/cloud/aws/core.dag`** — "What is AWS?" `Arn`, `AwsRegion`, `SigV4`, `AssumeRole`, `SessionCredentials`. | M | Done | ED-1 |
| 19 | ED-19 | **`extdeps/cloud/aws/s3.dag`** + **iam.dag** + **lambda.dag** + **secrets_manager.dag** + **sqs.dag** — AWS service models. Follow GCP patterns. | L | Done | ED-18 |
| 20 | ED-20 | **`extdeps/cloud/azure/core.dag`** — "What is Azure?" `AzureSubscription`, `AzureTenant`, `ManagedIdentity`, `FederatedCredential`. | M | Done | ED-1 |
| 21 | ED-21 | **`extdeps/cloud/azure/blob_storage.dag`** + **identity.dag** + **container_apps.dag** + **key_vault.dag** + **service_bus.dag** — Azure service models. Follow GCP patterns. | L | Done | ED-20 |

### Design Decisions

**Types vs data**: Each extdeps file has both. Types define the shape ("what is a
GCS object?"). Data declares behavioral facts ("get is readonly and idempotent").
No functions — extdeps are facts, not computation.

**Granularity**: One file per service, grouped by provider. `core.dag` at each
level provides shared concepts that service-specific files compose. This matches
the cloud provider API structure: each service has its own resource model.

**CAS modeling**: GCS uses generation numbers, S3 uses version IDs, Azure Blob
uses ETags. Each gets its own `CasPrecondition` type. The `infra/core.dag`
`ObjectStorage` interface maps these into a common `CompareAndSwap` pattern.

**Auth layering**: `cloud/core.dag` defines `AuthScheme` as a sum type.
`cloud/gcp/core.dag` fills in OAuth2+WIF+ServiceAccount. `cloud/aws/core.dag`
fills in SigV4+AssumeRole. The credential chain pattern in `std/patterns.dag`
composes whichever auth tautologies the profile selects.

**Relationship to services/**: `extdeps/` says *what the system is*.
`services/` says *how we talk to it* (REST endpoints, shell commands).
`services/gcp/secret_manager.dag` imports types from
`extdeps/cloud/gcp/secret_manager.dag` and adds transport blocks.

---

## Blue Unqueued

Raw observations from any worker. Not triaged, not sized.
Blue team promotes to backlog or lane queues during triage.

| Observation | Source | Date |
|-------------|--------|------|
| CI YAML generation (`generate_github_actions_template`, `generate_gitlab_ci_template`) is ~120 lines of hand-wired `push_str`/`write!` string concatenation in `codegen_cli.rs:503-609`. The DSL already has rendering infrastructure (`std/render.dag`, `std/markdown_render.dag`) and a proven code-generation pattern (`tools/makegen.dag`). CI YAML types (Workflow, Job, Step, Trigger, Permission, Cache) should be modeled in `.dag` with pure rendering functions, following the makegen pattern: discover via extern → render in pure DSL → `content_upsert`. Deletes both template functions + the validation functions (lines 450-500). See task breakdown below. | R1 scout | 2026-02-26 |
| `dsl/cloud/aws/credential.dag` imports `aws.STS` and `aws.SecretsManager` from abstract infra services (no transport blocks). These work via profile bindings but have no test coverage at L0 (compilation) or L1 (hermetic). Same pattern for `cloud/azure/credential.dag`. These could be BT-adjacent test targets once SDLC profiles are active. | CI fix | 2026-02-27 |
| The `credential_chain` pattern in `dsl/std/patterns.dag:236-283` is a proven 5-step OAuth2 chain (OIDC → STS → impersonation → SecretManager → AccessToken) with `local_auth()` at lines 392-411. Gist bypasses this entirely in favor of raw `shell.GCloud.SecretManagerAccessVersion`. Migrating gist to use `credential_chain` would exercise the pattern end-to-end and validate the compositional auth model. | RT-A4 analysis | 2026-02-27 |

---

## Blue Backlog

Triaged and sized. Promote to lane queues when horizon items are exhausted.

| ID | Item | Size | Priority | Notes |
|----|------|------|----------|-------|
| CG-1:5 | **DSL CI YAML generation (cigen).** Stacked tautologies: `extdeps/yaml.dag`, `extdeps/github_actions.dag`, `extdeps/gitlab_ci.dag`, `config/ci.dag`, `tools/cigen.dag`. Common render layer in `std/render.dag`. Extern bridge `discover_ci_config()`. Deleted ~700 lines Rust. | L | P1 | **Done**. Established the extdeps modeling pattern now used by Lane 2. |
| H10 | Compute stack orchestration: Cloud Run/GCS/LB lifecycle DAG builder. | L | P2 | `docs/design/horizon/h10-compute-stack-services.md` |
| S12-E | Multi-worker CAS: GcsClaimStore with generation-based CAS. DSL exists. Distinct from B-12 (which stress-tests SignalStore/ArtifactStore). | M | P2 | Deferred until cloud_run profile needed |
| H1 | Display reactive DSL: channel-driven event loop. | XL | P3 | No current use case. Review 2026-Q3, delete if not promoted. |

---

# RED TEAM — Harden

### Philosophy: Eliminate, Don't Relocate

The red team goal is **structural impossibility of defects**, not
better error messages for them. Every fix should push the problem
upstream — closer to the point of construction — so downstream code
can't encounter the bad state at all.

**Bad**: move a string match from file A to file B.
**Better**: parse the string into an enum at the boundary, match exhaustively.
**Best**: make the enum the only representation — no string ever exists.

The test: *after your fix, can a future contributor reintroduce the
same class of bug?* If yes, you relocated it. If no, you eliminated it.

### Smell Catalog (what scouts look for)

| Smell | Example | Typical Fix |
|-------|---------|-------------|
| **String dispatch** | `match kind_str { "shell" => ..., "rest" => ..., _ => ... }` | Parse once at boundary → enum. Exhaustive match, no fallback. |
| **Validation at use site** | `parse().unwrap_or(default)`, `if x.is_none() { return fallback }` | Make the constructor enforce the invariant. Fields are non-Option if always populated. |
| **Heuristic reimplementation** | Rust code that replicates logic the DSL already declares | Delete the Rust, call the DSL. If the evaluator can't handle the DSL construct yet, that's the real task (e.g., RF-G-unblock). |
| **Static mapping table** | Hand-maintained `HashMap` or match arms mapping A→B | Derive from a single source (DSL data declaration, enum with `#[derive]`, or const array). |
| **Option-that's-always-Some** | `field: Option<T>` where every construction site writes `Some(...)` | Make the field `T` with a `Default`. |
| **Stringly-typed enum** | `String` field that only holds N known values | Dedicated enum. `FromStr` at boundary, `.as_str()` only for serialization. |
| **Fallback arm** | `_ => default` or `other => ...` in a match on known variants | Exhaustive enum match. If a new variant appears, compilation forces handling it. |
| **Duplicate filter logic** | Same `starts_with("res:")` / `"tool:"` check in 5 files | Central type (`PortCategory`) with one `from()` impl. Call sites use the type. |
| **Manual registry** | Hand-maintained list mapping names → files/modules | Derive from DSL graph, Cargo.toml, or structural inference. |
| **Silent drop** | `_ => None` in wiring/lowering path, `Value::Skipped` for required output | Return typed error with source location. Required outputs must never silently skip. |
| **Dead scaffolding** | AST field/type that exists but parser never populates (e.g., `mock_response: Vec::new()`) | Either wire end-to-end with hard test, or delete. No "present-but-dead" features. |
| **Happy-path-only model** | Service operation with no error response declaration, mock always exit 0 / status 200 | Declare at least one error response. Mock spec must generate error scenarios. |
| **Accidental linkage** | `inventory::collect!` only works when crate happens to be linked into binary | Explicit force-link deps, or derive registrations from DSL annotations. |

### Remediation Ladder

When you find a smell, apply the **highest rung** that's feasible:

1. **Eliminate the representation** — the bad state can't be constructed
2. **Parse at the boundary** — raw input becomes a typed value once
3. **Derive from source of truth** — delete the hand-maintained copy
4. **Centralize** — if elimination isn't possible yet, at least one canonical impl

Rung 4 is a **waypoint**, not a destination. If you centralize, file
a follow-up to eliminate.


---

## Red Team: Three-Worker Plan

Three parallel workers with **mutually exclusive file ownership** and **zero
cross-worker dependencies**. Target: ~22k LOC net deletion. `gunbc-dag/src/`
goes from 22.7k to ~5.3k lines.

### Architectural Principles

1. **Pure functions, imperative shell.** Lowerer phases return typed data, not mutate shared state.
2. **Clear errors.** Every failure → typed error with span + stable error code. No panics on user input. No `_ => None` silent drops. No `eprintln!` diagnostics.
3. **Strong interfaces.** Pipe methods are an enum, not a string allowlist. Enums are values, not strings. Leaf refs are typed structs, not string sentinels.
4. **Stdlib is a cached registry, not compile-on-demand.** No runtime `daglang_driver` calls from `core/codegen`. Embed sources, cache once.
5. **Minimal language core.** Lambdas: no capturing, no mutation, closed combinator set. New features must delete an existing workaround.
6. **Delete from app layer; generic infra may move to core but must shrink.** Moved modules must delete app-specific branches, string dispatch, heuristics in the move. "Changed directories" without simplification is not progress. Workarounds are debt tokens paid before adding features.

### File Ownership

| Files | Worker |
|-------|--------|
| `gunbc-dag/src/bin/{sdlc,deps_config,pipeline,workflow,infra}.rs` | **A** |
| `gunbc-dag/src/workflow/` (17 files) | **A** |
| `gunbc-dag/tests/workflow_*.rs` (7), `tests/infra_cli.rs` | **A** |
| `gunbc-dag/src/makegen/`, `policy/`, `extern_impls.rs`, `resources.rs` | **B** |
| `gunbc-dag/src/{embedded_assets,docgen/,bootstrap/,build/,codegen/,infra/,gist,deps_tool}.rs` | **B** |
| `gunbc-dag/tests/{tool_registration,makefile_parity,extern_ratchet}.rs` | **B** |
| `core/daglang/` (all 5 compiler crates) | **C** |
| `core/codegen/` (cli_gen, fidelity, registry, testgen/) | **C** |
| `core/exec/` | **C** |
| `gunbc-dag/src/{resolve,resolve_service,mock_defaults}.rs`, `testgen_dag/` | **C** |
| Shared read-only: `lib.rs`, `dsl_builder.rs`, `dsl_registry.rs`, `bin/{ci,codegen_cli}.rs`, `dsl/` | all |

---

### Worker A: Binary & Workflow Elimination (-8.4k net)

Delete 5 hand-written binaries and the Rust workflow subsystem. Replace with
DSL data. After: every binary generated from DSL.

**Prerequisite**: Worker C delivers C20 (profile-aware CLI generation) first.
Worker A consumes the generated profile/mode/subcommand support; Worker A does
NOT modify `core/codegen/src/cli_gen.rs` or any compiler crate.

| # | IDs | What | Acceptance Criteria | Size |
|---|-----|------|---------------------|------|
| A1 | RT58, RT60 | **Eliminate `sdlc.rs`.** Move param_source propagation to `detect_entrypoints`. Delete handwritten binary. Requires C20 (profile CLI gen). | `sdlc.rs` deleted. Generated binary works with `--profile unit_test --dry-run`. | S |
| A2 | RT61 | **Eliminate `deps_config.rs`.** Requires C20 (mode flag support). | `deps_config.rs` deleted. `gunbc-deps-config --mode=ensure` works. | S |
| A3 | RT62 | **Eliminate `pipeline.rs`.** Move `query_ci_status()` etc. to DSL func nodes (shell transport to `gh` CLI). | `pipeline.rs` deleted. `gunbc-pipeline --depth 1` works. | M |
| A4 | RT63, RT64 | **Eliminate `workflow.rs`.** Requires C20 (subcommand dispatch). Move plan rendering to DSL. | `workflow.rs` deleted. `gunbc-workflow plan` and `run` work. | L |
| A5 | RT65 | **Eliminate `infra.rs`.** 8 subcommands → DSL. Requires C20 (`KEY=VALUE` parsing + multi-value flags). | `infra.rs` deleted. All 8 subcommands work via generated binary. | L |
| A7 | RT78 | **Workflow catalog → DSL data.** `dsl/config/workflow_catalog.dag` with `data` for `WORKFLOW_VARIANTS`. | `catalog.rs` data section deleted. Workflow count matches. | M |
| A8 | RT79 | **Unit commands → DSL data.** `dsl/config/workflow_commands.dag` with per-workflow `{ program, args }`. | `unit_commands.rs` deleted. Workflow execution uses DSL commands. | M |
| A9 | RT71 | **Extract generic workflow to `core/workflow/`.** Move planner, executor, admission, coordination, slo, projection, proof, errors, schema, key (9 modules, ~2.5k lines). | New `core/workflow/` crate. gunbc-dag imports it. All tests pass. | L |
| A10 | RT66 | **Delete binary infrastructure.** Remove `BinaryArgs` from `gunbc-cli`. Clean orphaned support. | `BinaryArgs` deleted. No `#[allow(clippy::disallowed_methods)]` in generated bins. | S |
| A11 | — | **Delete compensating tests.** 7 `workflow_*.rs` + `infra_cli.rs`. | Files deleted. `cargo test --workspace` passes. | S |

**Prerequisite**: C20 (profile/mode/subcommand CLI gen) must land before A1-A5.
**Execution order**: Start with A7 → A8 → A9 → A10 → A11 while waiting on C20.
Then run A1 → A2 → A3 and A4 → A5 once C20 is available.

---

### Worker B: Registry & Extern Deletion (-5.2k net)

Delete every manual registry, extern bridge, and thin wrapper. Replace with DSL
`data` declarations. After: adding a tool requires zero Rust changes.

| # | IDs | What | Acceptance Criteria | Size | Status |
|---|-----|------|---------------------|------|--------|
| B1 | RT75 | **Gitignore patterns → DSL data.** `dsl/config/gitignore.dag` with 14 category `data` records. | Data section in `gitignore.rs` deleted. Generated `.gitignore` identical. | S | Done |
| B2 | RT80 | **Makegen registry → DSL data.** MetaTarget/CoreWorkflow data in DSL. Dead ToolInfo factory methods deleted. `registry.rs` reduced to 559 lines (from 672). `ToolInfo::from_tool_def()` + Cargo commands remain. Generated Makefile identical. | `registry.rs` lean. Generated Makefile identical. | L | Done |
| B3 | RT74 | **Resource definitions → DSL data.** `dsl/config/resources.dag` with globs + output paths. | `resources.rs` deleted. Resource freshness works. | S | Done |
| B4 | RT76 | **Docgen targets → DSL data.** `dsl/tools/docgen.dag` data declaration. | `docgen/mod.rs` data deleted. Docgen reads from DSL. | S | Done |
| B5 | RT77 | **Delete `policy/pragma.rs`.** DSL rendering works (proven). Delete 546-line Rust mirror. | `pragma.rs` deleted. `make pragma` output identical. | S | Done |
| B6 | RT23 | **Delete `extern_impls.rs`.** Shadow bridges → DSL `extern func`. 2 recursive externs kept via inventory. | `extern_impls.rs` deleted. `lookup_extern_impl()` deleted. | M | Done |
| B7 | RT81 | **Delete tool wrappers.** 7 thin modules → generic `dsl_builder::build_dsl_graph_for_entrypoint()`. | `bootstrap/`, `build/`, `codegen/`, `infra/`, `gist.rs`, `deps_tool.rs` deleted. | S | Done |
| B8 | — | **Delete `embedded_assets.rs`.** Dead after extern deletion. | File deleted. | S | Done |
| B9 | — | **Delete compensating tests.** `tool_registration.rs`, `makefile_parity.rs`, `extern_ratchet.rs`. | 3 files deleted. `cargo test --workspace` passes. | S | Done |
| B10 | — | **Clean `makegen/shared.rs` + `justfile.rs`.** Remove deleted-registry references. | No references to deleted types. | S | Done |

**Chain**: B1 → B2 → B10; B3; B4; B5 → B6 → B7 → B8 → B9

---

### Worker C: Compiler Pipeline Refactor (-9.2k net)

Restructure compiler into Google-style layer cake. Lowerer as pure functions.
Resolver fail-closed. Stdlib cached. Types strong. Testgen/resolve extracted to core/.
Also delivers CLI generator extensions that Worker A consumes.

**Strategy**: strangler refactor. Build new lowerer phases alongside old code, test
for parity on full `.dag` corpus, switch when proven. Parity requires canonical DAG
representation (nodes sorted by ID, edges sorted by `(src, dst, ports, kind)`,
volatile metadata stripped).

Target layout for `daglang-lower/src/`:
```
lib.rs        # public API + re-exports (~2k, down from 8.7k)
context.rs    # LoweringContext struct
callable.rs   # Phase 1: lower callables → Vec<LoweredCallable>
              #   Invariant: every node has ports for every declared param/return
scope.rs      # ScopedBody (replaces ad-hoc detect_*)
transport.rs  # Phase 3: derive transports → TransportManifest
              #   Invariant: every service call site → exactly one triplet
wiring.rs     # Phases 4-6: derive edges → Vec<DerivedEdge>
              #   Invariant: every non-optional return binding has an edge or WiringGap
resource.rs   # Phases 7-8: resource lifecycle
assembly.rs   # Final: assemble_dag(parts) — sole mutation point
              #   Invariant: dedup by NodeKey, deterministic NodeId assignment
expr.rs       # LoweredExpr, LeafRef enum (not string sentinels)
eval.rs       # Pure evaluator
spec.rs       # Service operation specs
```

| # | IDs | What | Acceptance Criteria | Size |
|---|-----|------|---------------------|------|
| C1 | RT93 | **Stdlib host + caching.** `OnceLock` cache for compiled fn bodies. `include_str!` for stdlib sources. Single `StdLibHost::eval_fn()` interface. Delete per-module compile wrappers. | `classify_callable()` never calls `compile_from_context()`. No `../../dsl` paths. | M |
| C2 | RT42 | **Pipe methods first-class.** `PipeMethod` enum in syntax. Parser resolves `\|> method()` to `PipeCall(PipeMethod, ...)`. Delete `should_track_call_name()` allowlist. | Allowlist deleted. `PipeMethod` has all 20 methods. All `.dag` compile. | M |
| C3 | RT45, RT46 | **Typed enums end-to-end.** `Value::Enum { ty, variant }`. Delete `TestClass::parse()` / `FermiCost::parse()` round-trips. Replace `unwrap_or()` fallbacks with errors. | Zero `parse()` on classification. Zero `unwrap_or()` in fidelity. | M |
| C4 | RT82 | **LoweringContext + dead code (staged).** Context struct grouping 8-11 params. Delete 18 `#[allow(clippy::too_many_arguments)]`. Delete dead `_ => None` arms only after complex-return coverage is proven (C10 RT4a/c). | Zero `too_many_arguments`. `_ => None` deletion is gated by C10 parity (BinOp/If/Match/Pipe returns still wire). All `.dag` compile. | L |
| C5 | — | **Integrate scope.rs.** Replace `detect_*_branches_in_stmts`, `IfBranchSite`, `MatchBranchSite` with `ScopedBody`. Delete ad-hoc walk functions. | `IfBranchSite` deleted. `scope.rs` has non-test callers. DAG parity. | M |
| C6 | — | **Extract transport derivation.** `transport.rs` module. Returns `TransportManifest` (pure data). Invariant: every service call site maps to exactly one triplet. | `add_service_transport_triplets` returns data, not mutates builder. | M |
| C7 | — | **Expr walker totality + typed leaf refs.** Explicit arms for all `Expr` variants. `LeafRef` enum: `Param { name, field, ty }`, `Callable { endpoint, port }`, `Service { endpoint, port }`. | Zero `_ => {}` in expr walkers. `PARAM_REF_SENTINEL` deleted. | M |
| C8 | RT84 | **Delete dead AST scaffolding.** `MockResponseDef`, `error_cases()`, `@retry`, orphaned `hermetic`. | `MockResponseDef` deleted. `@retry` rejected by parser. `hermetic` warns. | S |
| C9 | RT38, RT39 | **No panics, no silent parse.** `LowerError::InvalidTransportSpec` replaces `panic!`. Parse error for bad `auth_input`. | Zero `panic!` on user DSL. Parser test for `auth_input: "token"`. | S |
| C10 | RT94, RT4a, RT4c | **Resolve ReturnExprCompute split-brain + completeness gate.** Desugar complex returns (BinOp/UnaryOp/If/Match/Pipe/...) into explicit DAG semantics so emit/interpret share behavior. Delete `MetadataOnly` for `ReturnExprCompute`. Delete `ReturnExprComputeOp`. Add fail-closed compile-time gate for non-optional unwired returns (RT4c). | Zero `ReturnExprCompute` in any compiled graph. `PrimitiveOpKind::ReturnExprCompute` deleted. No silent return-binding drops (`_ => None`) for required outputs. | L |
| C10a | RT-I5 | **`make gist` auth credential bridge fix (postmortem Option A/B/C).** Pick one option and implement in `daglang-lower` and/or `resolve_service.rs` so `auth_token: Secret` reaches REST auth reliably. This must land before C11 extraction to avoid migrating a known bug into core. | `make gist` no longer 401s due to empty bearer token path. C11 is blocked until this passes. | M |
| C11 | RT67, RT72 | **Move resolve_service.rs to core/.** 2,190 lines → `core/resolve/src/service_ops.rs`. Split `resolve.rs`: generic framework (~1.6k) → `core/resolve/`. Domain dispatch (~700) stays. Must delete app-specific string dispatch in the move. **Precondition: C10a complete. Inventory linkage gate required (RF-INV1 or RF-INV2) before/with move.** | `resolve_service.rs` deleted from gunbc-dag. New `core/resolve/` crate. Moved code is simpler than source. No dropped registrations after crate-boundary move (force-link evidence or RF-INV2 replacement). | L |
| C12 | RT73 | **Move testgen to core/.** 5 files (2,177 lines) → `core/codegen/src/testgen/`. Must delete gunbc-dag-specific assumptions in the move. **Inventory linkage gate required (RF-INV1 or RF-INV2) before/with move.** | `testgen_dag/` deleted from gunbc-dag. Testgen works from `core/codegen`. Cross-crate registrations still discoverable (or inventory removed via RF-INV2 path). | M |
| C13 | RT68 | **Split mock_defaults.** Generic probing (~350) → `core/test/`. Delete GCP blob (~230). | `mock_defaults.rs` deleted. Auto-mock works from `core/test`. | S |
| C14 | RT89 | **REST status-code checking.** `GenericRestParseOp` checks status before field extraction. Non-2xx → error. | 401 → structured error (not "field missing"). Test: mock 401 → error has status. | M |
| C15 | RT88 | **Fail-closed resolver audit.** Classify all `_ =>` fallbacks. Delete `passthrough_fallback_value()` (70 lines). | Zero undocumented fallbacks. `passthrough_fallback_value` deleted. | M |
| C16 | RT95 | **Transport class in node metadata.** `ServiceTransportClass` in lowered nodes. Registry gen reads metadata, not `node_id.contains("shell")`. | `from_node_context` reads metadata, not substrings. | S |
| C17 | RT96 | **Kill `propagate_to_param_sources`.** Fix boundary detection. Param source nodes auto-fed. | `propagate_to_param_sources` deleted. One port per input. | M |
| C18 | — | **Executor dead code.** Delete `looks_effectful_without_kind()`. Delete unwired credential expiry plumbing. | Dead code deleted. `cargo clippy` clean. | S |
| C19 | RT83, RT4b | **Restore passthrough enforcement + runtime fail-closed diagnostics.** After C4+C5+C7 wire dag_util branches, required outputs with no input must return `ExecError` (not `Skipped`) and emit clear diagnostics for missing declared passthroughs (RT4b). | `resolve.rs` returns `ExecError` for required missing outputs. Missing passthrough ports are diagnosable (no silent fallback). CI clean (no unwired branches). | S |
| ~~C20~~ | ~~RT59, RT63~~ | ~~**CLI generator: profile, mode, subcommand support.** Expose `available_profiles` in `CompileOutput`. Template generates `--profile` enum flag, `--mode ensure\|verify`, subcommand dispatch for multi-func modules. Unblocks Worker A.~~ **Done** | ~~Generated CLI for `pipelines/sdlc.dag` accepts `--profile`. Generated CLI for multi-func modules has subcommands.~~ | ~~L~~ |
| C21 | — | **CLI generator: KEY=VALUE and multi-value flag support.** For `Map<String, String>` params, generate `KEY=VALUE` parser (e.g., `--input project_id=my-project`). For `List<String>` params, generate accumulator flags (`--target A --target B`). Required for A5 (infra.rs elimination). | `gunbc-infra --input project_id=foo` parses to map. `--target A --target B` parses to list. | M |
| C22 | — | **Deductive Redundancy Elimination (DRE).** Replace naive `validate_no_operation_overlap` (removed) with idempotency fingerprinting. Phase 1: compile-time `StaticFingerprint` from `OperationKey` + `idempotency_keys` provenance. Phase 2: test-time execution ledger in `gunbc-test` that asserts `(OperationKey, Hash(values))` uniqueness per workflow run. `NonDeterministic` ops bypass checks. See `docs/design/deductive-redundancy.md`. | Static fingerprint catches duplicate reads/conflicting writes at compile time. Dynamic ledger catches runtime duplicates in hermetic tests. | L |

**Chain**: C1 → C3; C2; C10 (RT4a/c) → C4 → C5 → C6; C7; C8; C9; C10a → (RF-INV1 or RF-INV2 gate) → C11 → C14 → C15 → C19; C12; C13; C16; C17; C18; C20 (early, unblocks A); C21 (unblocks A5); C22

---

### Completed (Historical)

The following tasks are done. Kept for postmortem/audit reference.

| ID | What | Status |
|----|------|--------|
| C20 | CLI generator: profile, mode, subcommand support (RT59, RT63) | Done |
| RT1 | Credential wiring (`auth_input` → `res:credential`) | Done |
| RT2 | Execute node fail-closed when `auth_scheme` declared | Done |
| RT3 | File transport: EXISTS, CREATE_DIR, DELETE, APPEND, GLOB | Done |
| RT4 | Transport block validation in lowerer | Done |
| RT5 | `fold` extraction in evaluate_fn_body | Done |
| RT6 | `NodeKind` required (not `Option`) | Done |
| RT7 | `PortCategory` enum + `PortName` methods | Done |
| RT8 | `TransportRole` enum in resolve.rs | Done |
| RT9-RT12 | Virtual I/O DSL types + shell/HTTP/TCP registries | Done |
| RT17 | DSL Makefile assembly (4 pipeline stages) | Done |
| RT24 | Profile-aware compilation testing | Done |
| RT25-RT28 | Port constants, StringEnum derive, ModulePath, DslTypeMapping | Done |
| RT30-RT37 | Port migration, split monoliths, error consolidation, test helpers, dead scaffolding | Done |
| RT-A1:A5 | Auth postmortem analysis tasks | Done |
| RT-I3:I6 | Credential chain integrity, shell exit, gist migration, e2e verify | Done |

---

## Red Unqueued

Raw observations from any worker. Not triaged, not sized. Use the
smell catalog above to classify. Include file path + line if possible.

| Smell | Observation | File | Source | Date |
|-------|-------------|------|--------|------|
| *(credential wiring promoted to RT1)* | | | | |
| Validation at use site | `auto_mock_spec()` always produces exit 0 / status 200 for transport mocks. 7 transport mocks for gist_recent, 0 error scenarios. Testgen Bucket C `SingleTransportFailure` only injects `Value::Str("<TRANSPORT_FAILURE>")` sentinel, not realistic errors (401, exit 1). See `TODO/testgen-proof-analysis.md`. | `gunbc-dag/src/mock_defaults.rs:145-184` | RT-I4 proof | 2026-02-27 |
| Validation at use site | `GenericRestParseOp` doesn't check HTTP status code — it just tries to extract fields from the response body. A 401 with `{"message":"Bad credentials"}` only fails because `html_url` is missing. If a 401 body happened to contain expected fields, it would "succeed" with garbage. | `gunbc-dag/src/resolve_service.rs` (REST parse) | RT-I4 proof | 2026-02-27 |
| Heuristic reimplementation | `@mock_response` parser is NOT implemented. `MockResponseDef` AST struct exists in `daglang-syntax/src/lib.rs:587-591` but parser always initializes `Vec::new()`. No service in `dsl/services/` uses `@mock_response`. The `error_cases()` trait method on `Mockable` exists but is never populated. | `core/daglang/daglang-syntax/src/lib.rs`, `core/test/src/mockable.rs:59` | RT-A2 audit | 2026-02-27 |
| Validation at use site | `GNUmakefile` had `--mode=ensure` flag on bootstrap command but the binary doesn't accept that flag. Bootstrap binary only accepts `--check-mode`, `--dry-run`, `--print-inputs`. | `GNUmakefile:26` | CI fix | 2026-02-27 |
| Validation at use site | `find crates -type d` in bootstrap returns exit 1 when `crates/` directory doesn't exist. SplitLines was failing hard on this (post-RT-I4) but should return empty list for list-producing ops. | `dsl/tools/bootstrap.dag`, `gunbc-dag/src/resolve_service.rs:523-543` | CI fix | 2026-02-27 |
| ~~Static mapping table~~ | ~~Three functions deleted by RT5 — DSL `classify_transports` is now the single source.~~ | ~~`gunbc-dag/src/fidelity.rs`~~ | RF-H4 PR scout | **Done** |
| Heuristic reimplementation | `passthrough_fallback_value()` hard-codes a port alias table. | `gunbc-dag/src/resolve.rs:95-162` | RF-H4 PR scout | 2026-02-26 |
| Heuristic reimplementation | `looks_effectful_without_kind()` re-derives NodeKind from port type strings. Dead code after RT6 (NodeKind). | `core/exec/src/execute.rs:2064-2092` | RF-H4 PR scout | 2026-02-26 |
| Heuristic reimplementation | `classify_module()` inflated by transitive auth callables. | `gunbc-dag/src/fidelity.rs:184-209` | RF-E4 impl | 2026-02-26 |
| ~~Fallback arm~~ | ~~HTTP method fallback now returns `ExecError` instead of silently falling back to POST.~~ | ~~`gunbc-dag/src/resolve_service.rs:73-85`~~ | R1 scout | **Done** |
| String dispatch | `match field.type_id.as_str()` for JSON→Value appears twice. | `gunbc-dag/src/resolve_service.rs:291-335, 352-366` | R1 scout | 2026-02-26 |
| ~~Validation at use site~~ | ~~`input_as_string()` now returns `Result<String, ExecError>` — missing inputs fail-closed instead of leaking `"(unresolved)"`.~~ | ~~`gunbc-dag/src/resolve_service.rs:703-733`~~ | R1 scout | **Done** |
| ~~String dispatch~~ | ~~`FileOperationSpec.operation` now uses `FileOp` enum — parsed at boundary in `derive_file_spec()`, exhaustive match in `GenericFilePrepareOp`.~~ | ~~`gunbc-dag/src/resolve_service.rs`~~ | R1 scout | **Done** |
| String dispatch | `workflow_unit_commands()` matches workflow name strings. | `gunbc-dag/src/workflow/unit_commands.rs:300-323` | R1 scout | 2026-02-26 |
| Inventory linkage gap | `gunbc-codegen cigen` drops GCP secrets. See Theme INV below. | `gunbc-dag/src/ci/mod.rs:56-77` | lane-2 merge | 2026-02-26 |
| *(success_port workaround promoted to RT4a:c)* | | | | |
| Dead scaffolding | `@mock_response` annotation type exists in AST (`MockResponseDef`), parser never populates it (`mock_response: Vec::new()`). Wire parser → lowerer → `RestOperationSpec.mock_response` end-to-end. Design: `docs/design/mock-response-pipeline.md` Phase 1-2. | `daglang-syntax/src/parser.rs`, `daglang-lower/src/lib.rs` | BT-R3 scout | 2026-02-27 |
| Static mapping table | Kitchen sink `default_rest_response()` grows a new blob of fields for every service type. Should be derived from `@mock_response` annotations or synthesized from output field `from` paths. Design: Phase 2-3. | `gunbc-dag/src/mock_defaults.rs:200+` | BT-R3 scout | 2026-02-27 |
| Dual convention | `from` path format split: `.` separator (`head.sha`, 3 uses) vs `/` separator (`content/0/text`, 12+ uses). Normalize to `/` (JSON Pointer-like). Design: Phase 4. | `dsl/services/github/*.dag`, `dsl/services/llm/*.dag` | BT-R3 scout | 2026-02-27 |
| Heuristic reimplementation | `IdentityCallableOp` still overloaded for 2 roles (ContentUpsertOutputPath, DSL callable passthrough). `CallParamSourceOp` extracted by BT-R3. Remaining: `MetadataPassthroughOp` + `DeclaredOutputCallableOp`. Design: Phase 5. | `gunbc-dag/src/resolve.rs:260+` | BT-R3 scout | 2026-02-27 |
| Pessimistic ordering | `probe_best_response` tries `[Shell, File, REST]` — REST services are the majority but tried last, wasting 2 trial executions per REST parse node. Design: Phase 6. | `gunbc-dag/src/mock_defaults.rs` | BT-R3 scout | 2026-02-27 |

### POSTMORTEM: `make gist` 401 — Compounding Failures

**Symptom**: `make gist` returns 401 Unauthorized. Diagnostic: `BearerToken (key: "", source: static)`.

**Key evidence**: `shell.GCloud.SecretManagerAccessVersion` **succeeds** (token fetched), but the token arrives **empty** at `github.Gist.Create`. Five compounding failures:

#### Failure 1: No credential wiring from operation inputs to execute node (ROOT CAUSE)

The lowerer creates prepare→execute→parse triplets for service operations. When a service has `config { auth: BearerToken }` and an `auth_token: Secret` input field, **no code exists to wire the auth_token value from the prepare node to the execute node's `res:credential` port**.

- `daglang-lower` lines ~5731-5741: execute node gets `res:credential` input IF `has_auth` is true
- `daglang-lower` lines ~6367: profile-based auth IS explicitly wired to `res:credential`
- **Gap**: operation-level `auth_token` input field has no automatic outlet to `res:credential`

Before annotation deletion, `@headers({ "Authorization": "Bearer {auth_token}" })` explicitly wired the credential. After deletion, `config { auth: BearerToken }` was added but the corresponding wiring logic was never created.

#### Failure 2: GenericRestPrepareOp doesn't propagate auth_token

`GenericRestPrepareOp` builds the request URL, body, and headers from input fields. It does NOT:
- Detect `auth_token` as a credential input
- Set `RestRequest.auth` from the operation's auth scheme
- Expose the token as an output port for the execute node

The prepare node treats `auth_token` like any other input field — it may end up in the body but never in the auth header.

#### Failure 3: Execute node falls through silently

`TransportOps::Execute` (`lib/transport/src/ops.rs` lines 65-94) looks for `res:credential` input. If not provided, it uses whatever auth is already on the request (which is `None`). **No error** — it just sends an unauthenticated request.

#### Failure 4: Diagnostic source is misleading

`decorate_service_failure` in `error.rs` gets `credential_ref: None` because `infer_auth_from_headers` can't find an Authorization header (it was never set). This shows `source: static` — misleading because it implies a hardcoded credential when really there's NO credential.

#### Failure 5: GenericShellParseOp was emitting Value::Str for Secret outputs (FIXED)

`GenericShellParseOp.TrimStdout` always emitted `Value::Str`, ignoring `is_secret` flag. This meant the gcloud token flowed as `Value::Str` not `Value::Secret`. **Fixed in current session** — `shell_trim_value()` helper now respects `OutputFieldSpec.type_id == "Secret"`.

#### Fix options (pick one)

| Option | What | Scope | Risk |
|--------|------|-------|------|
| A | **Auto-bridge in lowerer**: when `auth_scheme.is_some()` on service config AND an input field is typed `Secret`, create edge from `prepare.{field}` → `execute.res:credential` with `Credential::new(Secret::static_value(token), AuthScheme::Bearer)` wrapping | daglang-lower + resolve_service.rs | Medium — heuristic, may mis-identify which Secret field is the credential |
| B | **Explicit auth input declaration in DSL**: add `auth_input: "auth_token"` to `config { ... }` block, lowerer uses it to wire credential | daglang-lower + daglang-syntax | Low — explicit, no heuristic |
| C | **RestPrepareOp sets auth on request**: when `auth_scheme` is in spec and `auth_token` is in inputs, set `req.auth = Some(...)` directly in prepare | resolve_service.rs only | Low — localized fix, but skips resource port pattern |

Option C is the smallest fix that unblocks `make gist`. Options A/B are more principled for the long term.

**Queue injection**: This postmortem is now explicitly assigned to Worker C as **C10a (RT-I5)** and is a hard precondition for **C11** (`resolve_service.rs` extraction).

### POSTMORTEM: `gunbc-ci` false failure — `overall_success: Skipped`

**Symptom**: `gunbc-ci` reports "A required success check returned false" even when all build/test/clippy stages succeed. Pre-existing on `main`. `make ci` (via `gunbc-workflow`) passes because it uses a different code path.

**Key evidence**: `success_port_failed()` finds `overall_success: Value::Skipped` on the `tools.build::build_all` node. The `&&` expression in the return statement was never wired.

#### Failure 1: Lowerer drops complex return expressions (ROOT CAUSE)

`resolve_return_expr_source()` (`daglang-lower/src/lib.rs:7768-7841`) handles return expression wiring for callable nodes. It matches 4 expression types:

| Expression | Match arm | Status |
|-----------|-----------|--------|
| `Expr::Ident(name)` | `return { x: my_var }` | Wired |
| `Expr::FieldAccess(base, field)` | `return { x: node.output }` | Wired |
| `Expr::Call(name, _)` | `return { x: my_fn() }` | Wired |
| `Expr::Literal` / `StringInterp` / `List` / `Map` | `return { x: "hello" }` | Wired |
| **Everything else** | `_ => None` (line 7839) | **Silent drop** |

`build.dag` line 35: `return { overall_success: build.success && test.success && clippy.success }` — this is `Expr::BinOp`, falls through to `_ => None`. No edge is created to `__out:overall_success`.

This silently drops: `BinOp` (`&&`, `||`, `+`, etc.), `UnaryOp` (`!`), `If`/`Match`, `Pipe`, `RecordUpdate`, `NullCoalesce`.

#### Failure 2: Passthrough falls back to `Value::Skipped` silently

`execute_with_declared_output_passthrough()` (`resolve.rs:71-91`) checks for `__out:overall_success` input. When the edge doesn't exist (failure 1), no input arrives. Fallback: `passthrough_fallback_value()` → `None` → `Value::Skipped`. No error, no warning.

#### Failure 3: `success_port_failed()` treats `Skipped` as failure

`success_port_failed()` (`display.rs:991-1003`) correctly treats `Value::Skipped` as failure (the success port must affirmatively be `Bool(true)`). This is correct behavior — the bug is upstream.

#### Failure 4: Testgen mocks bypass wiring

Generated tests for `build_all` (`generated_tests_tools_build.rs:658`) manually inject `__out:overall_success: Value::Bool(true)` as a mock input via `execute_single_node`. This tests the passthrough mechanism works but never validates that the lowerer creates the `__out:overall_success` edge from the `&&` return expression. The testgen model assumes all `__out:*` ports are properly wired — it tests nodes, not IR edges.

#### Failure 5: No IR-level wiring verification exists

There is no test or validation that checks: "for every return expression binding, was an edge created to the corresponding `__out:*` port?" The lowerer's `wire_callable_return_outputs()` silently `continue`s when `resolve_return_expr_source()` returns `None` (line 7890). The anti-tautology principle (testgen.md) says edge cardinality is "proven by construction" — but this is only true when the lowerer actually creates the edges.

#### Current workaround

`gunbc-dag/src/bin/ci.rs:160`: changed `success_port: Some("overall_success")` to `success_port: Some("success")`. This checks transport parse nodes' `success` output directly (which works because `success` is a simple `FieldAccess` expression, not a `BinOp`). The workaround is correct but bypasses the DAG's own aggregation logic.

#### Fix plan (three tasks, in priority order)

| # | Task | What | Scope | Eliminates |
|---|------|------|-------|-----------|
| RT4a | **Complex return expr lowering** | Extend `resolve_return_expr_source()` to handle `BinOp`, `UnaryOp`, `If`, `Match`, `Pipe` — synthesize compute nodes in the IR that evaluate the expression and wire result to `__out:*`. | `daglang-lower/src/lib.rs` | Root cause: silent drop of computed return values |
| RT4b | **Passthrough missing-input diagnostic** | When `execute_with_declared_output_passthrough()` falls back to `Value::Skipped` for a declared output port, emit a diagnostic warning (or error in strict mode). The port was declared in the type signature — `Skipped` means the wiring is broken. | `gunbc-dag/src/resolve.rs` | Silent failure: wrong output, no signal |
| RT4c | **Lowering completeness gate** | Add validation in `wire_callable_return_outputs()`: if `resolve_return_expr_source()` returns `None` for a non-optional output, emit `LowerWarning` (or error). Count unwired return expressions as a metric. | `daglang-lower/src/lib.rs` | Prevention: catch at compile time, not runtime |

RT4a is the real fix. RT4b/c are defense-in-depth so the class of bug can't recur silently. After RT4a, revert the ci.rs workaround back to `success_port: Some("overall_success")`.

**Queue injection**: RT4a/RT4c are now explicit in **C10**, RT4b is explicit in **C19**, and C10 is ordered before C4 dead-arm deletion to avoid BinOp/complex-return regressions.

---

## Red Backlog

Triaged and sized. Promote to lane queues when horizon items are exhausted.

### Theme TC: Transport Completeness (compositional modeling)

Transport declarations belong on the **service layer**, not on SDLC.
The SDLC pipeline only sees interfaces. Each service operation needs a
`transport rest { ... }` or `transport shell { ... }` block so the
compiler can generate prepare→execute→parse triplets.

Local-profile transport (26 ops) moved to Blue queue as BT6 (critical path for L4+).

| ID | Scope | Ops Missing | Notes |
|----|-------|-------------|-------|
| RF-TC3 | **Remaining providers**: GCS stores (6), github_issue_provider (7), credential providers (4) | 17 | Needed for cloud_run profile. Some overlap with BT6 (github_issue_provider delegates to github/issues.dag which BT6 covers). |
| RF-TC4 | **Stub providers**: stub_providers.dag (26), stub_credential_provider.dag (2) | 28 | Intentional — unit_test profile stubs. Consider `transport stub {}` marker. |
| RF-TC5 | **Infrastructure stubs**: azure (43), aws (38), gcp-infra (59) | 140 | Dormant — defer until infrastructure provisioning lane opens. |

### Theme E: Deleted Tests (re-add when root cause fixed)

| ID | Deleted Tests | Root Cause | Blocker |
|----|---------------|-----------|---------|
| RF-E5 | `makegen_runtime_differential_interpreter_vs_generated_rust_layer1` (codegen_parity.rs) | FnBodyDelegate gap: interpreter produces raw `{header}{body}`, fn body evaluation only works via `shared.rs` direct path. | Interpreter needs fn body evaluation support. |
| RF-E6 | `makegen_exec_runtime_e2e_structural_verification` (daglang-driver), `pragma_exec_runtime_e2e_structural_verification` (daglang-driver), `makegen_e2e_generated_binary_produces_correct_makefile` (cli_commands), `pragma_e2e_generated_binary_produces_correct_config_files` (cli_commands) | Exec-runtime emitter missing: `LoadRegistry` handler, `PureRender` fn classification, `ContentUpsertOutputPath` classification. | `daglang-emit` exec-runtime backend needs node classification for all makegen/pragma node kinds. |
| — | `clippy_toml_dsl_produces_valid_output` (pragma_parity.rs) | Sum type variant tags lost during `build_data_values()` JSON serialization. | RT19 (recursive types). |

### Theme INV: Inventory Linkage (cigen secrets gap)

`gunbc-codegen cigen` silently drops secrets from CI YAML because the
`inventory` crate's `submit!` registrations are discarded by the linker
when no symbols from the submitting crate are directly referenced.

**Root cause**: `ci_live_test_secrets()` calls `iter_dag_specs()` which
collects `DagSpecDef` entries via `inventory`. Entries with `live_required`
secrets are registered in `lib/gcp-ops`, `lib/review`, etc. The
`gunbc-codegen` binary (`codegen_cli.rs`) doesn't reference those crates'
symbols, so the linker drops them and their inventory registrations.
Meanwhile `gunbc-ci` (`ci.rs`) transitively references them through
`build_build_graph()`, so it sees the full inventory.

**Impact**: Running `gunbc-codegen cigen` produces a ci.yml missing 5
GCP secret env vars. Current workaround: ci.yml is committed with
secrets and not regenerated on every codegen pass.

**C11/C12 linkage trap**: moving large subsystems to `core/` crosses crate
boundaries and can trigger the same linker-drop behavior. Treat **RF-INV1 or
RF-INV2 as a hard gate** for C11/C12 so moved resolvers/testgen paths do not
silently lose registrations.

| ID | Fix | Size | Notes |
|----|-----|------|-------|
| RF-INV1 | **Force-link inventory crates in codegen binary.** Add explicit `use` references or `extern crate` for crates that register `DagSpecDef` with `live_required` secrets. Simplest fix, but fragile — adding a new crate with secrets requires updating codegen_cli.rs. | S | Quick fix. |
| RF-INV2 | **DSL-derive CI secrets from service annotations.** Instead of inventory, derive `live_required` from `@auth` + `@endpoint` annotations on service operations in `.dag` files. The DSL already declares auth schemes — the compiler can extract which env vars are needed. Eliminates the inventory linkage problem entirely. | M | DSL-first fix. Aligns with RT13 (derive mock registries from `@mock_response`). |

### Theme PC: Provider Response Contracts (mandatory error modeling)

> Design doc: `docs/design/provider-contracts.md`
> Aligns with: `docs/design/modeling/annotation-to-dag-modeling.md` Phase 2

Services must model the actual provider API contract — not just the happy
path. Every documented response code, error body shape, and failure mode
gets declared via structural `response { ... }` blocks on operations.
The lowerer compiles these into classify_response nodes in the transport
DAG. Testgen generates **mandatory** per-status-code test obligations.
Supersedes RT-I1 and RT-I2.

| ID | What | Size | Deps |
|----|------|------|------|
| PC-1 | **`response` block parsing.** Add to `daglang-syntax`. `Vec<ResponseEntry>` on `OperationDef` (replaces `mock_response`). | M | — |
| PC-2 | **Standard error types.** `dsl/std/errors.dag` — common HTTP, GitHub, GCP error shapes. | S | — |
| PC-3 | **`response` blocks on all REST services.** 29 operations, `doc` references to provider API docs. | L | PC-1, PC-2 |
| PC-4 | **`exit` blocks on all shell services.** Exit code → output type mapping. | M | PC-1 |
| PC-5 | **Lowerer: populate `error_mappings` + classify_response node.** Wire `response` entries to existing `ErrorMapping` on `ServiceOperationSpec`. Generate classify_response node in transport DAG. | M | PC-1 |
| PC-6 | **`GenericRestParseOp` status checking.** Route on status code before field extraction. Hard-fail on undeclared non-2xx. | M | PC-5 |
| PC-7 | **`ProviderResponseContract` obligation.** New Bucket C obligation, one per `response` entry. | M | PC-5 |
| PC-8 | **Testgen codegen for response contracts.** Per-status-code tests, mock body derived from response type. | L | PC-7 |
| PC-9 | **Interface response contract inheritance.** Implementors inherit obligations from interface `response` declarations. | M | PC-7 |
| PC-10 | **Completeness enforcement.** Compiler requires ≥1 success + ≥1 error entry in `response` block on every `transport rest {}` operation. | S | PC-1 |

### Compiler Features (low priority)

| ID | Feature | Size | Notes |
|----|---------|------|-------|
| FC-CF2 | `skip(n)`: List\<T\> → List\<T\> | S | Expressible via fold+index. |
| FC-CF3 | `enumerate()`: List\<T\> → List\<(Int, T)\> | M | Expressible via fold+counter. |

---

# PHASE 2 — Integration & Production

## Phase 2 Operating Model

Phase 1 (Blue/Red) advanced the SDLC pipeline and hardened the compiler/runtime.
Phase 2 wires those investments together and pushes SDLC to production.

Five lanes: two start immediately (independent of Phase 1 lanes), three start
as Phase 1 lanes land. Each lane is 5k+ LOC minimum.

```
Phase 1 (in progress):                    Phase 2:
──────────────────────                     ────────
Blue Lane 1: SDLC Activation        ──┐
Blue Lane 2: External Deps (ED)     ──┤── Lane 6: Service Layer Completion ── Lane 7: SDLC Production
Red Team: Workers A/B/C             ──┘                                   └── Lane 8: Contract Testing

Independent (start NOW):
  Lane 4: Domain Model Foundation  ── feeds Lane 6 (vocabulary + missing categories)
  Lane 5: Transport Layer          ── feeds Lane 7 (middleware + virtual backends)
```

### Why Phase 2 exists

Differential analysis of sibling repos (`the-gunbai`, `gunb.ai`) revealed a
recurring pattern: we build a modeling capability, forget to integrate it, then
reinvent or hack around it downstream. Phase 2 exists to prevent this for the
entire SDLC production path.

Specific gaps found:
- `the-gunbai` models **9 behavioral dimensions** per operation (side effects,
  idempotency, determinism, failure modes, edge cases, confidence, prerequisites,
  assumptions, unknowns). gunbc models **2** (`readonly`/`idempotent`).
- `gunb.ai` models **rate limits**, **capability prerequisites**, and
  **infrastructure scopes**. gunbc models **none** of these.
- **5 categories** of external systems in sibling repos are absent from gunbc:
  secret providers, coordination stores, tool lifecycle, LLM pricing/capabilities,
  API operational detail (rate limits, retry configs, versioning).
- The ED lane creates "what is X?" type files, but no service imports from them.
  Without wiring, the ED investment is orphaned.

### Phase 2 File Ownership

| Files | Lane |
|-------|------|
| `dsl/std/` (new: behavioral, coordination, rate_limit, errors, capability) | **4** |
| `dsl/extdeps/secrets/`, `coordination/`, `tools/`, `devenv/`, `api/`, `llm/pricing.dag` | **4** |
| `dsl/interfaces/` (enrichment with behavioral contracts) | **4** |
| `lib/transport/src/` (middleware, virtual backends, credential middleware) | **5** |
| `core/test/src/` (mock synthesis, failure injection) | **5** |
| `core/ir/src/transport/` (transport middleware IR types) | **5** |
| `dsl/services/` (wiring imports, response blocks) | **6** |
| `core/daglang/daglang-syntax/` (response block parsing) | **6** |
| `core/daglang/daglang-lower/` (response → classify_response nodes) | **6** |
| `lib/cloud-ops/`, `lib/gcp-ops/` (real GCP API clients) | **7** |
| `gunbc-dag/src/bin/sdlc.rs` (production CLI) | **7** |
| `dsl/cloud/`, `dsl/workflows/` (deployment DAGs) | **7** |
| `core/codegen/src/testgen/` (contract obligations, compliance) | **8** |

### Phase 2 Source Reference

All ported content from sibling repos lives in one design doc:

**`docs/design/domain-model-porting.md`** — behavioral property structure (§1),
secret providers (§2), coordination stores (§3), tool lifecycle (§4), LLM
pricing/capabilities (§5), infrastructure scope model (§6), rate limits and
retry configs (§7), Git CLI behaviors (§8), devcontainer model (§9).

Lane 4 workers reference this doc directly. No access to sibling repos needed.

---

## Lane 4: Domain Model Foundation (INDEPENDENT — start now)

### Principle

Port the modeling depth from `the-gunbai` (behavioral properties per operation)
and `gunb.ai` (rate limits, capability contracts, infrastructure scopes) into
gunbc's DSL. Create domain models for categories the ED lane doesn't cover.

**Pure DSL authoring** — no Rust changes, no compiler changes.

### Why this lane exists

The ED lane (Lane 2) creates "what is X?" type files for cloud/github/llm/git/cargo.
This lane creates the **vocabulary** those models need (behavioral properties, rate limits,
coordination primitives) and the **categories** ED doesn't cover (secrets, coordination,
tools, dev environments). Both feed into Lane 6 (Service Layer Completion).

Without this lane, the ED files are shallow type stubs — the behavioral depth that makes
the sibling repos useful (failure modes, edge cases, rate limits, prerequisites) is missing.

### File Territory

- `dsl/std/` — new vocabulary files only (no modification of existing std/ files)
- `dsl/extdeps/secrets/` — secret provider models (new subdirectory)
- `dsl/extdeps/coordination/` — coordination store models (new subdirectory)
- `dsl/extdeps/tools/` — tool lifecycle models (new subdirectory)
- `dsl/extdeps/devenv/` — dev environment models (new subdirectory)
- `dsl/extdeps/llm/pricing.dag` — complements ED-6:8 (which model message format)
- `dsl/extdeps/api/` — API operational detail (complements ED-2:5, ED-9:15 which model entities)
- `dsl/interfaces/` — enrichment of existing 7 files with behavioral contracts

**No overlap with ED lane** (which writes `extdeps/cloud/`, `extdeps/github/`,
`extdeps/llm/{core,anthropic,openai}.dag`, `extdeps/git.dag`, `extdeps/cargo.dag`).

### Queue

**Part A: Standard Vocabulary** (no deps, start first)

| # | ID | Task | Size | Status | Deps |
|---|-----|------|------|--------|------|
| 1 | DM-1 | **`std/behavioral.dag`** — behavioral property vocabulary. Types: `SideEffects` (ReadOnly \| WritesState \| WritesExternal), `Determinism` (Deterministic \| NonDeterministic \| EventuallyConsistent), `FailureMode` { name, http_status?, recoverable, retry_safe }, `EdgeCase` { description, trigger, severity }, `Confidence` (Documented \| HighConfidence \| MediumConfidence \| LowConfidence \| Assumed), `Prerequisite` { description, kind }, `OperationBehavior` { side_effects, idempotent, determinism, failure_modes, edge_cases, confidence, prerequisites, assumptions, unknowns }. Ref: `docs/design/domain-model-porting.md` §1. | M | Done | — |
| 2 | DM-2 | **`std/rate_limit.dag`** — rate limiting & retry vocabulary. Types: `RateLimit` { scope, requests_per_window, window_seconds, burst? }, `ResetStrategy` (FixedWindow \| SlidingWindow \| TokenBucket), `RetryPolicy` { max_attempts, backoff, retry_on }, `BackoffStrategy` (Exponential { base_ms, max_ms } \| Linear { step_ms } \| JitteredExponential { base_ms, max_ms }), `PropagationDelay` { typical_ms, max_ms, description }. Data: common presets. Ref: `docs/design/domain-model-porting.md` §7. | S | Done | — |
| 3 | DM-3 | **`std/coordination.dag`** — coordination primitive vocabulary. Types: `CasMechanism` (GenerationBased \| ETagBased \| VersionId \| RowVersion), `FencingToken`, `LeaseConfig` { ttl_seconds, heartbeat_interval_seconds }, `DeliveryGuarantee` (AtLeastOnce \| AtMostOnce \| ExactlyOnce), `QueueSemantics` { ordering, visibility_timeout_seconds, dead_letter? }, `DeadLetterPolicy` { max_attempts, destination }, `NotificationMechanism` (Polling \| PubSub \| Listen \| Webhook). Ref: `docs/design/domain-model-porting.md` §3. | M | Done | — |
| 4 | DM-4 | **`std/errors.dag`** — standard error type vocabulary. Types: `HttpErrorShape` { status, error_type, message, detail? }, `AuthError` (Expired \| InvalidToken \| MissingCredential \| InsufficientScope \| Forbidden), `RateLimitError` { retry_after_seconds, scope }, `ConflictError` { resource, expected_version?, actual_version? }, `ProviderError` { provider, operation, http_status?, message }. Data: canonical error shapes — GitHub `{ message, documentation_url }`, GCP `{ error: { code, message, status } }`, Anthropic `{ type, error: { type, message } }`. Ref: `docs/design/domain-model-porting.md` §2, §7. | M | Done | — |
| 5 | DM-5 | **`std/capability.dag`** — capability contract vocabulary. Types: `InfraScope` (Secret \| Identity \| Api \| Storage \| Compute \| Network \| Database \| Queue \| Federation), `AccessLevel` (Read \| Write \| Admin), `CapabilityRequirement` { scope, level, resource }, `CapabilityGrant` { provider, scope, concrete_resource }, `ServiceAccessManifest` { service_name, requirements }. Ref: `docs/design/domain-model-porting.md` §6. | M | Done | — |

**Part B: Secret Provider Models** (deps on DM-1)

| # | ID | Task | Size | Status | Deps |
|---|-----|------|------|--------|------|
| 6 | DM-6 | **`extdeps/secrets/core.dag`** — "What is a secret provider?" Abstract model. Types: `SecretLifecycle` (Active \| Disabled \| Destroyed \| PendingDeletion), `SecretVersion` { id, state, created_at? }, `RotationPolicy` { period_seconds, next_rotation? }, `SecretAccessPolicy`, `SecretEncryption` (ProviderManaged \| CustomerManaged { key_id }). Data: shared behavioral properties — all reads readonly+idempotent, creates non-idempotent, version-add non-idempotent. | M | Done | DM-1 |
| 7 | DM-7 | **`extdeps/secrets/gcp_secret_manager.dag`** — "What is GCP Secret Manager?" Types: `GcpSecret`, `GcpSecretVersion`, `GcpReplicationPolicy` (Automatic \| UserManaged). Data: 8 `OperationBehavior` records (AccessVersion, CreateSecret, AddVersion, ListSecrets, GetSecret, DestroyVersion, DisableVersion, EnableVersion), rate limits per method, propagation delays. Ref: `docs/design/domain-model-porting.md` §2.1. | M | Done | DM-6 |
| 8 | DM-8 | **`extdeps/secrets/github_secrets.dag`** — "What is GitHub Secrets?" Types: `GitHubSecretScope` (Repository \| Environment \| Organization), `GitHubPublicKey` { key_id, key }, `OidcTokenClaims`. Data: 8 `OperationBehavior` records (Get, Create, Update, Delete, ListRepo, ListEnv, GetPublicKey, GetOidcToken), encryption model (libsodium sealed box). Ref: `docs/design/domain-model-porting.md` §2.2. | M | Done | DM-6 |
| 9 | DM-9 | **`extdeps/secrets/env_file.dag`** — "What is a .env file?" Types: `EnvEntry` { key, value, comment? }, `EnvFile`, `EnvValidation` { required_keys, format_rules }. Data: 6 `OperationBehavior` records (Load, Get, Set, List, Validate, GenerateExample), parsing rules. Ref: `docs/design/domain-model-porting.md` §2.3. | S | Done | DM-6 |
| 10 | DM-10 | **`extdeps/secrets/vault.dag`** — "What is HashiCorp Vault?" Types: `VaultEngine` (Kv2 \| Transit \| Database \| Pki), `VaultAuth` (AppRole \| Oidc \| Token \| Kubernetes), `VaultLease` { id, ttl_seconds, renewable }, `VaultPolicy`, `DynamicCredential` { engine, role, ttl }. Data: 10 `OperationBehavior` records (KvGet, KvPut, KvDelete, KvList, Authenticate, RenewLease, RevokeLease, GenerateDynamicCred, RotateRoot, WrapToken), lease lifecycle. Ref: `docs/design/domain-model-porting.md` §2.4. | L | Done | DM-6 |

**Part C: Coordination Store Models** (deps on DM-3)

| # | ID | Task | Size | Status | Deps |
|---|-----|------|------|--------|------|
| 11 | DM-11 | **`extdeps/coordination/core.dag`** — "What is a coordination store?" Abstract model. Types: `CoordinationBackend` (ObjectStore \| RelationalDb \| EmbeddedDb), `CasPattern` { mechanism, conflict_resolution }, `QueuePattern` { claiming, visibility, dead_letter }, `LeasePattern` { acquisition, heartbeat, expiry }, `NotificationPattern` { mechanism, latency, ordering }. Data: pattern composition rules (CAS+lease = distributed lock, queue+CAS = exactly-once processing). | M | Done | DM-3 |
| 12 | DM-12 | **`extdeps/coordination/gcs.dag`** — "GCS as coordination store." Types: `GcsGeneration`, `GcsMetageneration`, `GcsPrecondition` (IfGenerationMatch \| IfMetagenerationMatch \| IfGenerationNotMatch). Data: 6 `OperationBehavior` records (GetObject, InsertObject, PatchMetadata, DeleteObject, ComposeObjects, WatchChanges), CAS patterns (if-generation-match:0 = create-only), PubSub notification latency. Ref: `docs/design/domain-model-porting.md` §3.1. | M | Done | DM-11 |
| 13 | DM-13 | **`extdeps/coordination/postgres.dag`** — "PostgreSQL as coordination store." Types: `PgIsolationLevel` (ReadCommitted \| Serializable), `PgSkipLocked`, `PgFencingToken` { column, monotonic }, `PgNotifyChannel`. Data: 7 `OperationBehavior` records (KvGet, KvSet, KvCas, ClaimFromQueue, Heartbeat, Release, NotifyWatch), SKIP LOCKED semantics, NOTIFY/LISTEN latency. Ref: `docs/design/domain-model-porting.md` §3.2. | M | Done | DM-11 |
| 14 | DM-14 | **`extdeps/coordination/sqlite.dag`** — "SQLite as coordination store." Types: `SqliteJournalMode` (Wal \| Delete \| Truncate), `SqliteBusyTimeout`. Data: 5 `OperationBehavior` records (KvGet, KvSet, KvCas, Heartbeat, TtlCleanup), WAL mode constraints, single-writer limitation, CAS via `UPDATE WHERE version = ?`. Ref: `docs/design/domain-model-porting.md` §3.3. | S | Done | DM-11 |

**Part D: Tool Lifecycle Models** (deps on DM-1)

| # | ID | Task | Size | Status | Deps |
|---|-----|------|------|--------|------|
| 15 | DM-15 | **`extdeps/tools/rust_toolchain.dag`** — "What is the Rust toolchain?" Types: `RustChannel` (Stable \| Beta \| Nightly { date? }), `RustupComponent` (Rustfmt \| Clippy \| RustSrc \| Miri \| LlvmTools), `CargoCommand` (Build \| Test \| Clippy \| Doc \| Run \| Install \| Bench), `RustTarget`, `RustEdition` (E2015 \| E2018 \| E2021 \| E2024). Data: 6 `OperationBehavior` records (Install, AddComponent, Build, Test, ClippyLint, PublishCrate), platform matrix. Ref: `docs/design/domain-model-porting.md` §4.1. | M | Done | DM-1 |
| 16 | DM-16 | **`extdeps/tools/gh_cli.dag`** — "What is the GitHub CLI?" Types: `GhAuthMethod` (OAuthBrowser \| Token \| GitHubApp), `GhCommand` { verb, resource, flags }, `GhOutputFormat` (Json \| Table \| Template). Data: 6 `OperationBehavior` records (Auth, ReleaseList, ReleaseDownload, IssueCreate, PrCreate, RunWatch), auth flow. Ref: `docs/design/domain-model-porting.md` §4.2. | S | Done | DM-1 |
| 17 | DM-17 | **`extdeps/tools/package_managers.dag`** — "What are package managers?" Types: `PackageManager` (Apt \| Brew \| Cargo \| Winget \| Choco \| GithubRelease), `PlatformSupport` { os, arch }, `InstallSource` (SystemRepo \| Crate \| Release { url_template }). Data: 7 `OperationBehavior` records per manager (Verify, Install, Update, Remove, Search, ListInstalled, AddSource), cross-platform matrix. Ref: `docs/design/domain-model-porting.md` §4.3. | M | Done | DM-1 |

**Part E: Complementary Models** (deps on DM-1, DM-2)

| # | ID | Task | Size | Status | Deps |
|---|-----|------|------|--------|------|
| 18 | DM-18 | **`extdeps/devenv/devcontainers.dag`** — "What is a devcontainer?" Types: `LifecycleHook` (InitializeCommand \| OnCreateCommand \| PostCreateCommand \| PostStartCommand \| PostAttachCommand), `HookExecution` { phase, runs_as, timeout? }, `DevcontainerFeature` { id, version, options }, `DevcontainerEnvVar` { name, source, available_in }. Data: lifecycle timing, common features, quirks. Ref: `docs/design/domain-model-porting.md` §9. | S | Done | DM-1 |
| 19 | DM-19 | **`extdeps/llm/pricing.dag`** — LLM pricing and capability detail. Types: `ModelPricing` { input_per_million, output_per_million, cached_input_per_million? }, `ContextWindow` { max_tokens, max_output_tokens }, `ModelCapability` (ExtendedThinking \| StructuredOutput \| Grounding \| ToolUse \| VisionInput), `CachingBehavior` { min_cache_tokens, ttl_minutes?, discount_pct }. Data: model specs for Anthropic (Claude 4 family), OpenAI (GPT-4o, o1, o3), Gemini (2.5 family). Ref: `docs/design/domain-model-porting.md` §5. Complements ED-6:8 without overlap. | M | Done | DM-1, DM-2 |
| 20 | DM-20 | **`extdeps/api/github_ops.dag`** — GitHub API operational detail. Types: `GitHubProductTier` (Free \| Pro \| Team \| EnterpriseCloud \| EnterpriseServer), `GitHubApiVersion` { date, header }, `GitHubRateLimitScope` (Core \| Search \| GraphQL \| CodeSearch), `GitHubAppAuth` { jwt_expiry_minutes, clock_skew_seconds, installation_token_ttl }, `GitHubPollingConfig` { interval_seconds, max_wait_seconds }. Data: rate limits per scope+tier, versioning policy, polling configs. Ref: `docs/design/domain-model-porting.md` §7.1, §7.3. Complements ED-2:5 without overlap. | M | Done | DM-1, DM-2, DM-5 |
| 21 | DM-21 | **`extdeps/api/gcp_ops.dag`** — GCP API operational detail. Types: `GcpApiPattern` { service, version, base_url_template }, `GcpPropagationDelay` { operation, typical_seconds, max_seconds }, `GcpOrgPolicyConstraint` { constraint_id, affects }, `GcpCommandClassification` (Allowed \| Deprecated \| Forbidden). Data: propagation delays per service (IAM ~60s, DNS ~300s), retry configs, org policy constraints, gcloud command registry. Ref: `docs/design/domain-model-porting.md` §7.2. Complements ED-9:15 without overlap. | M | Done | DM-1, DM-2, DM-5 |

**Part F: Interface Enrichment** (deps on DM-1, DM-3, DM-5)

| # | ID | Task | Size | Status | Deps |
|---|-----|------|------|--------|------|
| 22 | DM-22 | **Enrich all 7 interfaces with behavioral contracts.** Import `std.behavioral`, `std.coordination`, `std.capability`. Add `OperationBehavior` data declarations for each capability. Add `CapabilityRequirement` declarations. Add failure mode contracts. Files: `interfaces/{artifact_store, outcome_ledger, signal_store, claim_store, credential_provider, issue_provider, agent_provider}.dag`. | L | Done | DM-1, DM-3, DM-5 |

**Chain**: DM-1:5 (parallel) → DM-6:10, DM-11:14, DM-15:17, DM-18:21 (parallel within group) → DM-22

---

## Lane 5: Transport Layer (INDEPENDENT — start now)

### Principle

Two-phase approach following the Protobuf/gRPC pattern:

**Phase 1 (TL-0:10): Target SDK** — Build language-specific OS mechanisms (token
buckets, mutexes, retry loops, credential caches). This is the "runtime library"
that handles thread sleep, atomic counters, TCP sockets. Domain-agnostic.

**Phase 2 (TL-11:15): Domain Modeling** — Move domain policy (rate limit budgets,
retry rules, error shapes) from Rust code into `.dag` service definitions. The
compiler generates **configuration code** that links to the Target SDK.

The distinction:
- **Domain policy (What)** → `.dag` — "GitHub rate limit is 5000/hour"
- **OS mechanisms (How)** → Target SDK — "How do I atomically decrement a counter?"

**Design doc**: `docs/design/transport-primitives.md`

**Phase 1 work**: `lib/transport/`, `core/test/`, `core/ir/src/transport/`
**Phase 2 work**: `core/daglang/`, `dsl/services/`

### Why this lane exists

The transport layer handles basic HTTP/Shell/File I/O, but production use
requires: rate limit awareness (GitHub: 5000/hr core, 30/min search; GCP:
varies per service), automatic retry with jittered backoff, response
classification (status code → typed error before field extraction), credential
refresh/caching, and enriched virtual backends for hermetic testing.

Phase 1 builds these mechanisms as a domain-agnostic Target SDK. Phase 2
moves the domain-specific configuration (rate limits, error shapes) into
`.dag` files so the compiler can emit configuration for any target language.

### File Territory

- `lib/transport/src/` — middleware modules, virtual backend enrichment, credential middleware
- `core/test/src/` — mock response synthesis, failure injection
- `core/ir/src/transport/` — transport middleware types (RateLimitConfig, RetryConfig)

**No overlap with Red Team** (which works in `core/daglang/`, `core/codegen/`,
`core/exec/`, `gunbc-dag/src/`).

### Phase 1: Target SDK (TL-0:10)

Build the domain-agnostic OS-level middleware. This code doesn't know "GitHub"
or "GCP" — it only knows "I was configured with budget=5000, window=3600".

| # | ID | Task | Size | Status | Deps |
|---|-----|------|------|--------|------|
| 0 | TL-0 | **Transport foundation types.** `lib/transport/src/transport_types.rs`. `TransportClass` enum (Rest, Shell, File, Grpc, Stream, Pubsub, Custom). `TransportCapabilities` struct (connection_pooling, retry_safe, streaming, etc.). `EndpointBehavior` enum (RateLimited, Cacheable, Idempotent, Paginated, etc.). `OperationBehavior` struct (readonly, idempotent, hermetic flags + retry/timeout config). Exported from `lib.rs`. | M | **Done** | — |
| 1 | TL-1 | **Rate limit middleware.** `lib/transport/src/rate_limit.rs`. Token bucket + sliding window implementations. Per-endpoint rate tracking. `RateLimitConfig` from IR transport metadata. Automatic 429/retry-after handling. Shared rate state across concurrent requests. Tests: burst exhaustion, recovery, concurrent access. | L | **Done** | TL-0 |
| 2 | TL-2 | **Retry middleware with backoff.** `lib/transport/src/retry.rs`. Exponential + jittered backoff. Configurable per-operation via `RetryPolicy` from IR. Idempotency-aware: only auto-retry operations marked `idempotent` or `readonly`. Transient error classification (5xx, network timeout, rate limit). Circuit breaker for persistent failures. Tests: retry sequences, backoff timing, circuit breaker state machine. | L | **Done** | TL-1 |
| 3 | TL-3 | **Response classification.** `lib/transport/src/classify.rs`. HTTP status code → typed error mapping. Per-provider error shape parsing (GitHub `{ message, documentation_url }`, GCP `{ error: { code, message, status } }`, Anthropic `{ type, error: { type, message } }`). Classification hierarchy: auth error > rate limit > client error > server error > network error. Feeds error into retry decision (TL-2). Tests: per-provider error shapes, unknown shapes, malformed responses. | M | **Done** | — |
| 4 | TL-4 | **Credential middleware.** `lib/transport/src/credential.rs`. Token caching with TTL-aware refresh. Multi-provider credential resolution (OAuth2 bearer, GCP WIF, API key). Automatic credential injection into requests based on `AuthScheme` from service config. Proactive refresh at 80% TTL. Thread-safe credential store. Tests: token refresh, concurrent access, expired token detection. | L | **Done** | TL-3 |
| 5 | TL-5 | **Virtual HTTP stub backend.** `lib/transport/src/test_backend.rs` enrichment. Method+path+headers → status+body+headers matching. Regex path matching for parameterized endpoints (`/repos/{owner}/{repo}/issues`). Ordered response sequences (first call → X, second → Y). Unmatched request → test failure with diagnostic. Tests: exact match, regex match, sequence exhaustion, diagnostics. | M | **Done** | — |
| 6 | TL-6 | **Virtual shell cassette backend.** `lib/transport/src/test_backend.rs` enrichment. argv pattern → stdout+stderr+exit_code matching. Environment variable assertion. Working directory tracking. Stdin injection. Tests: exact match, glob match, exit code scenarios, env vars. | M | **Done** | — |
| 7 | TL-7 | **Mock response synthesis.** `core/test/src/mock_synthesis.rs`. Generate mock responses from `OperationBehavior` data (DM-1 types at runtime). Status-code-specific mock bodies derived from error shapes (DM-4). Failure injection: inject specific failure modes from behavioral property declarations. Replace `default_rest_response()` kitchen sink with behavioral-property-driven synthesis. Tests: per-provider synthesis, failure injection, round-trip. | L | Pending | TL-3, TL-5 |
| 8 | TL-8 | **Transport metrics hooks.** `lib/transport/src/metrics.rs`. Request/response timing. Retry count per request. Rate limit headroom tracking. Error classification distribution. Pluggable sink (log, structured event, /dev/null for tests). Tests: timing accuracy, concurrent metric collection. | S | **Done** | TL-1, TL-2 |
| 9 | TL-9 | **Transport middleware composition.** `lib/transport/src/pipeline.rs`. Compose middleware in order: metrics → rate_limit → retry → credential → execute → classify. `TransportPipeline` builder API. Per-operation middleware configuration from IR metadata. Override/disable individual middleware layers. Tests: full pipeline integration, middleware ordering, per-operation config. | M | **Done** | TL-1:4, TL-8 |
| 10 | TL-10 | **IR transport types.** `core/ir/src/transport/` additions. `RateLimitConfig`, `RetryConfig`, `CredentialConfig`, `ResponseClassification` as IR types. Lowerer can populate from service/operation metadata. Resolver reads to configure transport pipeline. Tests: round-trip serialization, lowerer population. | M | **Done** | — |

**Chain**: TL-1 → TL-2 → TL-9; TL-3 → TL-4, TL-7; TL-5; TL-6; TL-10 (early, no deps); TL-8 → TL-9

### Phase 2: Transport Domain Modeling (after TL-9)

**Design doc**: `docs/design/transport-primitives.md`

**Principle**: Phase 1 (TL-0:10) builds the **Target SDK** — OS-level mechanisms (token buckets, mutexes, sockets) that are language-specific. Phase 2 moves **domain data** (rate limit budgets, retry policies, error shapes) into `.dag` where it belongs.

The distinction:
- **Domain policy (What)** → `.dag` — "GitHub rate limit is 5000/hour" is an external fact about a service
- **OS mechanisms (How)** → Target SDK — "How do I atomically decrement a counter?" is Rust/Go/Python specific

This follows the Protobuf/gRPC pattern: the compiler generates **configuration code** that links to a Target SDK, not line-by-line OS implementations.

| # | ID | Task | Size | Status | Deps |
|---|-----|------|------|--------|------|
| 11 | TL-11 | **DSL syntax for transport blocks.** Add `rate_limit {}`, `retry {}`, `error_shape {}`, `credential {}` blocks to grammar. Parse budget expressions (`5000 per hour`). Typecheck scope bindings (`uses rate_limit: core`). **Implementation**: 1) Add AST types to `daglang-syntax/src/lib.rs` (add `RateLimitDef`, `RetryDef`, `ErrorShapeDef`, `CredentialDef` structs + extend `ServiceConfig` to hold `Vec<RateLimitDef>` etc). 2) Add parser rules to `parser.rs`. 3) Add typecheck in `daglang-typecheck/`. See `docs/design/transport-primitives.md` § DSL Syntax for grammar spec. | L | Pending | TL-9 |
| 12 | TL-12 | **Lower transport blocks to IR.** Lower DSL transport blocks to existing `TransportMiddlewareConfig` IR (TL-10). Rate limit budgets become `RateLimitConfig`. Retry policies become `RetryConfig`. Rust runtime still interprets. | M | Pending | TL-10, TL-11 |
| 13 | TL-13 | **Domain data migration.** Move hardcoded rate limits from Rust to `dsl/services/*.dag`. GitHub 5000/hour core, 30/min search. GCP quotas. Anthropic limits. Delete provider-specific branches from `classify.rs`. Service definitions become source of truth. | M | Pending | TL-12 |
| 14 | TL-14 | **Multi-target emit.** Emit transport configuration per target language. Rust emits code linking to existing Target SDK. Go/Python stubs for future. Generated code calls `RateLimitMiddleware::new(config)` — doesn't reimplement token bucket. | XL | Pending | TL-13 |
| 15 | TL-15 | **Substrate cleanup.** `lib/transport/` becomes pure Target SDK (no domain knowledge). Delete `GITHUB_CORE_LIMIT` constants, `host.contains("github.com")` branches. All domain facts live in `.dag`. | L | Pending | TL-14 |

**Chain (Phase 2)**: TL-11 → TL-12 → TL-13 → TL-14 → TL-15

---

## Lane 6: Service Layer Completion (after ED lane + Red Team + Lane 4)

### Principle

Wire the domain models (ED lane + Lane 4) into services: import extdeps types,
add `response {}` blocks, make the compiler enforce response contracts.

This is the **integration lane** — it connects the knowledge layer to the execution layer.
It exists specifically to prevent the "build a feature, forget to integrate it" pattern.

### Why this lane exists

The ED lane creates "what is X?" models. Lane 4 creates behavioral vocabulary. But today
`services/github/issues.dag` defines its own input/output types inline instead of importing
from `extdeps/github/issues.dag`. And no service operation declares error responses — the
system is happy-path-only. This lane closes both gaps: services consume structured domain
models, and the compiler enforces that every operation handles errors.

### File Territory

- `dsl/services/` — wiring imports, adding response blocks
- `core/daglang/daglang-syntax/` — `response {}` block parsing
- `core/daglang/daglang-lower/` — compile response entries to classify_response nodes
- `gunbc-dag/src/resolve_service.rs` — status-aware GenericRestParseOp

### Queue

| # | ID | Task | Size | Status | Deps |
|---|-----|------|------|--------|------|
| 1 | SL-1 | **Wire extdeps → github services.** `services/github/{issues,pull_request,gist}.dag` import types from `extdeps/github/{issues,pull_requests,gists}.dag`. Replace inline type references with imported types. Validate compilation succeeds with richer type information. | M | Pending | ED-2:5 |
| 2 | SL-2 | **Wire extdeps → llm services.** `services/llm/{anthropic,openai}.dag` import from `extdeps/llm/{anthropic,openai}.dag`. Model types replace string fields. | S | Pending | ED-6:8 |
| 3 | SL-3 | **Wire extdeps → gcp services.** `services/gcp/{secret_manager,sts,iam}.dag` import from `extdeps/cloud/gcp/{secret_manager,sts,iam}.dag`. | M | Pending | ED-9:13 |
| 4 | SL-4 | **Wire extdeps → shell/git/cargo services.** Import types from `extdeps/git.dag`, `extdeps/cargo.dag`, `extdeps/tools/rust_toolchain.dag`. | S | Pending | ED-16:17, DM-15 |
| 5 | SL-5 | **Enrich ED files with behavioral properties.** Add `OperationBehavior` data declarations to all ED-1:21 extdeps files using DM-1 vocabulary. Rate limits (DM-2), capability requirements (DM-5). ~21 files enriched with behavioral data. | L | Pending | ED-1:21, DM-1:5 |
| 6 | SL-6 | **`response` block parsing (PC-1).** Add `response { STATUS => TYPE }` syntax to `daglang-syntax`. `Vec<ResponseEntry>` on `OperationDef`. `ResponseEntry`: status pattern (200, 2xx, 401, 4xx, 5xx), response type, optional description. Replaces dead `MockResponseDef`. | M | Pending | Red C8 |
| 7 | SL-7 | **`response` blocks on all REST services (PC-3).** 29 operations across github (14), llm (2), gcp (3), shell-as-rest (10). Each gets success + error response entries. Error shapes imported from `std/errors.dag` (DM-4). `doc` field references provider API docs. | L | Pending | SL-6, DM-4, SL-1:3 |
| 8 | SL-8 | **`exit` blocks on all shell services (PC-4).** Exit code → output type mapping for shell operations. 0 = success, non-zero = error with stderr. | M | Pending | SL-6 |
| 9 | SL-9 | **Lowerer: response → classify_response node (PC-5).** Compile `response {}` entries into `ErrorMapping` on `ServiceOperationSpec`. Generate classify_response node in transport DAG between execute and parse. | M | Pending | SL-6 |
| 10 | SL-10 | **GenericRestParseOp status checking (PC-6).** Route on status code before field extraction. Non-2xx → classify against declared responses. Undeclared non-2xx → hard error. Supersedes Red C14. | M | Pending | SL-9 |
| 11 | SL-11 | **Completeness enforcement (PC-10).** Compiler requires ≥1 success + ≥1 error entry in `response {}` on every `transport rest {}` operation. Warning for missing, error in strict mode. | S | Pending | SL-6 |

**Chain**: SL-1:4 (wiring, parallel) → SL-5 (enrichment); SL-6 (parsing) → SL-7:8, SL-9 → SL-10 → SL-11

---

## Lane 7: SDLC Production (after SDLC lane + Lane 5 + Lane 6)

### Principle

Take the hermetic SDLC pipeline to real cloud execution. Credential chaining,
Cloud Run deployment, real LLM agent, multi-worker CAS, CI integration.

### Why this lane exists

The SDLC pipeline works hermetically (L0–L7). This lane pushes it to L8
(cloud deployment) and real-world execution. Every item here depends on
the transport middleware (Lane 5) and service contracts (Lane 6) being in place,
so when we hit real GCP APIs we have rate limiting, retry, and error classification
instead of discovering those requirements empirically.

### File Territory

- `lib/cloud-ops/`, `lib/gcp-ops/` — real GCP API clients
- `dsl/cloud/`, `dsl/workflows/` — deployment DAGs
- `gunbc-dag/src/bin/sdlc.rs` — production CLI

### Queue

| # | ID | Task | Size | Status | Deps |
|---|-----|------|------|--------|------|
| 1 | SP-1 | **GCP credential chaining (BT13).** WIF OIDC → STS token exchange → optional impersonation → scoped access token. Consumes `extdeps/cloud/gcp/sts.dag` types + `extdeps/secrets/gcp_secret_manager.dag` behavioral model. Uses transport pipeline (TL-4 credential middleware). `std/patterns.dag::credential_chain` wired to real transport. Tests: L4 integration with real GCP (env-gated). | L | Pending | ED-15, DM-7, TL-4 |
| 2 | SP-2 | **Cloud Run deployment DAG (BT14).** Service creation, revision deployment, traffic migration. Consumes `extdeps/cloud/gcp/cloud_run.dag` types. Idempotent deploy: revision exists → no-op. Traffic split: gradual rollout. Tests: deployment DAG compilation, DryRun scenario, L4 real deploy (env-gated). | L | Pending | SP-1, ED-14 |
| 3 | SP-3 | **Agent provider: real LLM (BT17).** Wire `codex_agent.dag` to Anthropic/OpenAI via transport pipeline. Uses `extdeps/llm/pricing.dag` for token budget tracking. Prompt construction from SDLC stage context. Tests: L4 integration with real LLM (env-gated). | M | Pending | SL-2, DM-19 |
| 4 | SP-4 | **Credential provider: local keychain (BT18).** Token storage for local profile. Encrypted file or OS keychain. Refresh logic. Tests: store/retrieve/refresh cycle. | M | Pending | TL-4 |
| 5 | SP-5 | **Multi-worker CAS stress test (BT15).** 3 workers, exactly-once claim processing. GCS generation-based CAS. Conflict detection + retry. Consumes `extdeps/coordination/gcs.dag` behavioral model for CAS edge cases. Tests: concurrent claim, conflict resolution, exactly-once verification. | M | Pending | SP-2, DM-12 |
| 6 | SP-6 | **CI integration (BT16).** Hermetic pipeline in CI (unit_test profile). Cloud smoke test (cloud_run profile, env-gated). Generated CI YAML includes SDLC stages. | M | Pending | SP-5 |
| 7 | SP-7 | **Webhook-driven stage transitions (BT19).** Cloud Run HTTP endpoint receives GitHub webhook events. Event → stage transition mapping. Signature verification. Tests: webhook parsing, signature validation, event→stage mapping. | L | Pending | SP-2 |

**Chain**: SP-1 → SP-2 → SP-5 → SP-6; SP-3; SP-4; SP-7 (after SP-2)

---

## Lane 8: Contract Testing & Compliance (after Red Team + Lane 6)

### Principle

Build the automated infrastructure that proves providers comply with interface
contracts. Every interface gets a generated test suite. Every provider runs it.
No happy-path-only models remain.

### Why this lane exists

Interfaces declare contracts (`contract get(id) after create(...) => { found: true }`)
but no test infrastructure validates them. Providers implement interfaces but only
the happy path is tested — error responses, conflict scenarios, and behavioral
property violations are invisible. This lane builds the compiler and testgen
support that turns those declarations into executable compliance tests.

### File Territory

- `core/codegen/src/testgen/` — contract obligation codegen
- `core/daglang/daglang-syntax/` — contract annotation refinement
- `dsl/` — test blocks, fixture enrichment

### Queue

| # | ID | Task | Size | Status | Deps |
|---|-----|------|------|--------|------|
| 1 | CT-1 | **Contract IR.** Parse `contract` declarations on interfaces into `ContractObligation` structs in lowerer. Obligation types: sequence (A after B), idempotency (A twice = same result), destructive (A then B fails). Store in type registry alongside interface capabilities. Design: `docs/design/contract-testing.md` §Phase 1. | L | Pending | Red C done |
| 2 | CT-2 | **Contract test generation.** For each interface with `contract`, testgen emits parameterized test suite. Suite takes `ServiceBinding` as input. Each obligation → test case: setup → execute sequence → assert postcondition. Parameterized over providers: stub (fast/hermetic), real (env-gated). | L | Pending | CT-1 |
| 3 | CT-3 | **Provider compliance wiring.** For each (profile, interface, provider) triple, instantiate CT-2 suite. Stub providers: always-run in CI. Real providers: env-gated. Wire into existing testgen infrastructure. | M | Pending | CT-2 |
| 4 | CT-4 | **Annotation cleanup (Category 3).** Delete metadata noise annotations (`@network`, `@credential`, `@external`, `@derived_from`, `@ledger`, ~30 uses) per `docs/design/modeling/annotation-to-dag-modeling.md` Category 3. Pure DSL cleanup. | S | Pending | — |
| 5 | CT-5 | **`ProviderResponseContract` obligation (PC-7:9).** New testgen Bucket C obligation, one per `response {}` entry. Per-status-code tests with mock body derived from response type. Interface response contract inheritance: implementors inherit obligations from interface `response` declarations. | L | Pending | SL-9, CT-1 |
| 6 | CT-6 | **Restore deleted tests (RF-E5/E6).** Root causes (ReturnExprCompute, exec-runtime emitter) fixed by Red Team C10/C19. Re-enable: `makegen_runtime_differential`, `makegen_exec_runtime_e2e`, `pragma_exec_runtime_e2e`, `clippy_toml_dsl_produces_valid_output`. | M | Pending | Red C10 |
| 7 | CT-7 | **DSL-derive CI secrets (RF-INV2).** Replace inventory linkage with derivation from `auth` + `endpoint` on service operations in `.dag` files. Compiler extracts required env vars from service configs. Eliminates the crate linkage problem entirely. | M | Pending | SL-6 |

**Chain**: CT-1 → CT-2 → CT-3 → CT-5; CT-4 (no deps, start anytime); CT-6 (after Red C10); CT-7 (after SL-6)

---

## Reference: Theme Details

Detailed descriptions for queue items. The queue has priority order,
this section has context.

**ID mapping** (old → new):
RT1:4 = new (RF-MC1→RT1, RF-MC2→RT2, RF-MC3→RT3, RF-TV1→RT4).
RF-G-unblock→RT5, RF-A1→RT6, RF-A2a→RT7, RF-A4→RT8,
VIO-1→RT9, VIO-3→RT10, VIO-4→RT11, VIO-5→RT12, VIO-2→RT13,
VIO-6→RT14, VIO-7→RT15, VIO-8→RT16, FC-P7-c2→RT17, FC-P7-d→RT18,
FC-CF5→RT19, FC-CF6→RT20, FC-P8-a→RT21, FC-P8-b→RT22, FC-P8-c→RT23.
B-0→BT1, B-PW→BT2, B-1→BT3, B-2→BT4, B-3→BT5, B-TC→BT6, B-4→BT7,
B-5→BT8, B-6→BT9, B-7→BT10.

### Theme MC: Model Correctness (compositional transport pipeline)

**Audit findings** (2026-02-26):

The compositional model (transport → service → provider → interface → profile)
is architecturally sound and proven end-to-end for 39 operations across 4
transport types (shell, rest, file, local). But three infrastructure gaps
cause **silent failures** — requests go out wrong without errors:

| Gap | Symptom | Severity |
|-----|---------|----------|
| **RT1: Credential wiring** | `config { auth: BearerToken }` + `auth_token: Secret` input → token never reaches execute node → unauthenticated request, silent | Critical — affects all authenticated REST services |
| **RT2: Execute silent fallthrough** | Missing `res:credential` → execute sends request without auth, no error | Critical — masks RT1 |
| **RT3: File transport coverage** | Only READ/READ_BYTES/WRITE implemented → EXISTS/CREATE_DIR/DELETE/APPEND/GLOB operations fail at resolve | Blocking — SDLC local profile needs EXISTS + CREATE_DIR |
| **RT4: Typecheck validation** | Typechecker ignores transport blocks entirely → invalid contents compile silently | High — errors surface late at resolve time with confusing messages |

RT1 root cause analysis: See POSTMORTEM section below.
RT4 detail: `LowerError::MissingTransport` now fires for partially-specified services
(at least one op has transport, another doesn't). Fully-abstract services (no transport
on any operation) and interface implementors are exempt — they get transport via profiles.

**Coverage snapshot** (97 total operations across 28 services):
- 39 ops (40%): transport blocks present, pipeline works end-to-end
- 24 ops (25%): intentional stubs (unit_test profile, no transport needed)
- 16 ops (16%): have `@rest` annotations but no `transport rest {}` block (github/issues, github/pull_request, llm/openai)
- 18 ops (19%): SDLC providers needing transport for local/cloud profiles

### Theme RG: Manual Registry Elimination

**Audit findings** (2026-02-26):

| Registry | Location | Entries | Derivable From | Status |
|----------|----------|---------|----------------|--------|
| `WorkspaceBinary` enum | `binaries.rs:55-70` | 13 | Cargo.toml `[[bin]]` section | ⚠️ Manual |
| Workflow variant catalog | `catalog.rs:34-127` | 10 | DSL `pipeline` declarations + annotations | ⚠️ Manual |
| Extern impls | `extern_impls.rs:26-64` | 6 | DSL `extern func` declarations (gated by ratchet) | ✓ Gated |
| Module path dispatch | `resolve.rs:683-706` | 3 + prefix | Convention-based fallthrough | ⚠️ Implicit |
| Tool definitions | `dsl_registry.rs:44-178` | dynamic | Structural inference from DSL | ✓ Auto |
| Entrypoint inference | `daglang-lower` | dynamic | Unconnected port analysis | ✓ Auto |
| Process unit registry | `process_registry.rs` | dynamic | DSL workflow DAGs | ✓ Auto |

**RF-RG1**: Eliminate `WorkspaceBinary` + workflow variant catalog.
Both are static tables that duplicate information the compiler already derives.
Tool definitions and entrypoints are already auto-derived from DSL. These two
should follow the same pattern.

**RF-RG2**: Generalize resolve.rs so new DSL modules work without adding
hardcoded paths. Current hardcoded: `std.resources` (resource lifecycle),
`tools.infra` (single custom op), prefix `services.*` / `workspace.*`
(generic transport). Convention: if a module doesn't match any special
pattern, fall through to generic callable resolution.

### Theme G: Rust Heuristics Shadowing DSL Declarations

The DSL already has the complete, correct implementation:
- `std/fidelity.dag::classify_transports(transports: List<TransportClass>)`
  takes raw transport classes, aggregates via `fermi_max_of` + `all`, returns
  typed `DerivedClassification { test_class, depth, hermetic }`.
- `std/fermi.dag::fermi_max_of(depths)` folds over magnitudes.
- `config/test_policy.dag` adds repo-specific budget policy.

But the Rust side in `fidelity.rs` **replicates this entire computation**
as hand-wired heuristics: ordinal integers for depth comparison,
boolean maps for hermetic, string round-trips for enum values. These
exist solely because `classify_transports` uses `fold` (via
`fermi_max_of`), and the lowerer can't extract fn bodies containing
`fold` for `evaluate_fn_body()`.

**RT5** (was RF-G-unblock): **Done.** Multi-param lambda parsing
(`(acc, d) => expr`), pipe method typechecker exclusion (fold, map,
filter, etc. in should_track_call_name), fidelity.rs rewrite to call
DSL `classify_transports()` directly via `evaluate_fn_body()`.
Deleted:
- ✅ `transport_depth_ordinal()`, `transport_depth_str()` (RF-G1)
- ✅ `transport_is_hermetic()` (RF-G2)
- ✅ `classify_callable()` pre-aggregation → now calls DSL (RF-G3)
- ✅ `test_policy.dag::classify_from_facts()` (RF-G4)
- ✅ `test_policy.dag` shadow fns `transport_depth`/`transport_hermetic` (RF-G6)
Remaining: TestClass::parse()/FermiCost::parse() round-trip (RF-G5)
still used — fidelity result comes back as Value::Str, needs parsing.
Silent fallbacks `unwrap_or(Unit)` / `unwrap_or(XS)` still present as
safety nets for edge cases.

### Theme H: Structural Enforcement (parse, don't validate)

| ID | What | Current | Structural Fix |
|----|------|---------|---------------|
| RF-H2 | **TestgenTargetDef Option fields always populated**. `test_class: Option<TestClass>`, `fermi_cost: Option<FermiCost>`, `requires: Option<Vec<String>>` — every auto-testgen call site now fills `Some(...)` from fidelity. The `Option` only exists for legacy `DagSpecDef` path (which also never overrides). `generate_target_with_types()` does `unwrap_or(Unit)` on every field. | 6 Option fields in registry.rs | Make fields non-Option with `Default` impl. Callers construct with values; no unwrapping. `DagSpecDef.to_def()` fills from fidelity instead of leaving `None`. |
| RF-H4 | **ResourceKind string dispatch**. `ResourceAcquireOp { resource_kind: String }` matched at runtime. Unknown kinds fall through to `Value::Str("resource:{other}")` — wrong type, silent. | resolve.rs:365-386 | `ResourceKind` enum parsed once at resolve time. Match is exhaustive, no fallback arm. |

### Theme A: Typed Dispatch (full detail)

| ID | Pattern | Key Files | Notes |
|----|---------|-----------|-------|
| RF-A1 | **NodeKind on Node\<T\>**. `validate_node_kinds_for_interception()` is a runtime check that rejects `kind: None` nodes. Target: `Node::opaque()` requires `NodeKind`, eliminating `Option` and runtime check. | node.rs, execute.rs | Remove `Option<NodeKind>`, delete validation fn. |
| RF-A2a | **Port namespace typing (definition)**. Define `PortCategory` enum + methods on `PortName` in `core/ir/`. | core/ir/ | R1 scope. |
| RF-A2b | **Port namespace typing (migration)**. Migrate 18+ `starts_with("res:")`/`"tool:"`/`"__out:"` checks in `core/daglang/`, `core/codegen/testgen/`. | 4 R2 files | R2 scope. Depends on RF-A2a. |
| RF-A3 | **Module path representation**. 4 crates use `Vec<String>` vs typed `ModulePath`. | 4 crates | Unify on `ModulePath`; add `From` impls. |
| RF-A4 | **Stringly-typed dispatch in resolve.rs**. 10+ string prefix matches for module/callable routing. | resolve.rs | `CallableClass` enum parsed once. |
| RF-A5 | **Transport node classification**. String-based prepare/execute/parse detection. | resolve.rs | `TransportNodeKind { Prepare, Execute, Parse }`. |
| RF-A6a | **String constants (definition)**. Define central consts (`__deps`, `__out:`, `res:file`, `tool:`) in `core/ir/src/signature.rs`. | core/ir/ | R1 scope. |
| RF-A6b | **String constants (migration)**. Migrate 141+ `__deps`, 7 `res:file`, 15 `tool:` references in `core/daglang/`, `core/codegen/testgen/`. | R2 files | R2 scope. Depends on RF-A6a. |
| RF-A8 | **`as_str`/`parse` boilerplate**. `#[derive(StringEnum)]` for 15 enums (~60 match blocks). Includes TestClass/FermiCost `FromStr`. | 12 files | |
| RF-A9 | **Emit backend type-name tables**. Same type mapping in 3 backends. | daglang-emit | Shared `DslTypeMapping` table. |
| RF-A10 | **String dispatch in DAG tooling**. 100+ match arms on string literals. | 5 files | Registry pattern or DSL data declarations. |

### Theme VIO: Virtual I/O Infrastructure (BB-6 S+ tiers)

BB-6 XS tier (PureMock/DryRun) works. S+ tiers are stubs awaiting virtual I/O
infrastructure. The design insight: **S-tier doesn't need real servers**. The
`TransportBackend` trait intercepts at the `TransportRequest`/`TransportResponse`
struct level. S-tier REST/HTTP/TCP is just a response registry inside
`VirtualTransportBackend` — in-process request→response matching, no sockets.

**Existing infrastructure**:
- `VirtualTransportBackend` in `lib/transport/src/test_backend.rs`: handles File
  ops (read/write/append/delete/exists/glob/metadata) and basic Shell (find,
  printenv, test -f). REST/HTTP/TCP return error.
- `TransportBackendGuard` in `lib/transport/src/backend.rs`: scoped backend
  install/restore via RAII guard. Thread-safe global swap.
- `FidelityLadder`/`FidelityLevel` in `core/test/src/fidelity.rs`: canonical
  ladders for all 6 TransportKind variants. `node_max_fidelity()` transitive meet.

**DSL-first approach**:
- RT9 defines virtual backend configuration types in DSL (`dsl/std/virtual_io.dag`)
- RT13 derives mock registries from existing `@mock_response` annotations —
  zero hand-written mock data
- RT10:12 add response registries to `VirtualTransportBackend` (shell cassettes,
  HTTP stubs, TCP loopback) — same in-process interception pattern as File
- RT14:16 wire virtual backends into testgen codegen per tier

**Key files**:
- `dsl/std/virtual_io.dag` (new, RT9)
- `lib/transport/src/test_backend.rs` (RT10, RT11, RT12)
- `core/codegen/src/testgen/mock_corpus.rs` (RT13)
- `core/codegen/src/testgen/codegen.rs` (RT14, RT15, RT16)

---

## Archive

Completed items. RF-H4, RF-H2, RF-E4, BB-2, BB-3, BB-5 completed 2026-02-26.
BT-R1 (testgen discovery fix), BT-R2 (provider completion), BT11, BT12 completed 2026-02-27.

NF-1:6 (compile+link hardening): 2026-02-25. Detail: `TODO/TODONE/tasks-completed.md`.
FC-NF7 (fn-level evaluation): 2026-02-25. `expr.rs` IR + `eval.rs` evaluator + `FnBodyDelegate`.
FC-CL (dead code cleanup): 2026-02-25. Deleted `core/tool-registry` + macros, 14 orphaned fns.
FC-EG (enforcement gates): 2026-02-25. Import-direction lint, extern count gate, format!/push_str gate.
FC-P6-a:d (policy migration): 2026-02-26. `dsl_render.rs` evaluates `derive_*` DSL fns.
FC-CF1 + FC-CF7 (split + zip): 2026-02-26. Both pipe methods, 4 compiler stages, 9 tests.
FC-P7-a (build_workflows.dag): 2026-02-26. WorkflowSpec + MetaTarget types + data.
FC-P7-b (artifact emitter): 2026-02-26. `dag_emit.rs` emits valid `.dag` syntax.
FC-P7-c1 (Makefile DSL types): 2026-02-26. Types in `extdeps/make.dag`.
BB-0 (compositional type modeling): complete. Types in `corpus.rs` + `fidelity.rs`. 23 tests.
BB-1 (mock corpus builder): complete. `build_corpus()` in `mock_corpus.rs`.
BB-4 (type-derived boundary values): complete. `enrich_corpus_with_type_witnesses()`.
BB-6 (transport fidelity ladders): complete. Canonical ladders, `node_max_fidelity()`.
RF-H4 (ResourceKind enum): 2026-02-26. String dispatch → enum in resolve.rs.
RF-H2 (TestgenTargetDef non-Option): 2026-02-26. Option fields → defaults.
RF-E4 (fidelity smoke test): 2026-02-26. makegen→Unit/XS, gist→Integration/L.
BB-2 (per-node corpus tests): 2026-02-26. `build_corpus_section()`. Pure→Real, effectful→DryRun.
BB-3 (adjacent pair tests): 2026-02-26. `build_adjacent_pair_section()`. Edge wiring verification.
BB-5 (cross-workflow consistency): 2026-02-26. `build_cross_workflow_section()`. Multi-workflow nodes.
RT-A1 (shell exit code audit): 2026-02-27. Decision matrix for all 11 shell operations. TrimStdout/SplitLines semantics defined.
RT-A2 (@mock_response gap): 2026-02-27. 29 REST ops × 0 annotations = 29 missing. Parser not implemented. See `TODO/testgen-proof-analysis.md`.
RT-I4 (shell exit code enforcement): 2026-02-27. TrimStdout fails on non-zero (unless optional), SplitLines returns empty list. Proof tests + analysis report.
Positional auth_input wiring: 2026-02-27. Fixed positional service call args dropping credentials. `operation_inputs` field on `ServiceTransportEndpoint`.
GNUmakefile bootstrap fix: 2026-02-27. Removed invalid `--mode=ensure` flag. Fixed `make install` → `make ci` pipeline.
