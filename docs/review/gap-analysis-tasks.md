# Gap Analysis: Remaining Work After Lane Merge

> Generated: 2026-02-28
> Branch: `cursor/lane-executions-review-87fe`
> State: 3 branches merged, compile errors fixed, 3 test failures + 1 clippy error remain

---

## P0: Bugs — Must fix before any other work

These are broken right now on the merged branch.

| # | Bug | Root Cause | Fix | Size |
|---|-----|-----------|-----|------|
| P0-1 | **Test: `compile_resolve_execute_end_to_end_function_body_expressions`** fails. C15 passthrough enforcement returns `ExecError` for missing `__out:report` input instead of `Value::Skipped`. | C15 (passthrough fail-closed) landed but C10 (ReturnExprCompute desugaring) did not. Enforcement is on but wiring is still broken. | Either: (a) fix the lowerer to wire `return { report: report }` properly (C10 fix), or (b) update test to expect the error until C10 lands. | S |
| P0-2 | **Test: `resolve_lowered_dag_defers_pipeline_nodes`** fails. `PipelineDispatchOp.execute()` returns `ExecError` for missing `__out:out` input. | Same cause as P0-1 — C15 fail-closed enforcement catches pipeline passthrough without matching input edge. | Update test fixture to provide `__out:out` input, or relax enforcement for pipeline dispatch ops. | S |
| P0-3 | **Test: `push_str_boundary_ratchet`** fails. Baseline is 77 but merged code has 93 `push_str` calls in non-boundary code. | One or more branches added `push_str` calls outside allowed directories. | Audit the 16 new push_str locations. If they're in legitimate boundary code, add the directory to `ALLOWED_DIRS`. Otherwise update the baseline to 93. | S |
| P0-4 | **Clippy: `PipeMethod::from_str` should implement `FromStr` trait.** `clippy::should_implement_trait` fires on inherent `from_str()` method. | C2 (PipeMethod enum) added `from_str()` as an inherent method instead of implementing `std::str::FromStr`. | Implement `FromStr` for `PipeMethod` instead of inherent method. | S |
| P0-5 | **Pre-existing (main): `make install` fails.** `return_expr_compute` node for `codegen.dag` can't resolve variable `check`. `dsl/tools/codegen.dag:27`: `success: check.needed \|\| run.success` — this `BinOp` expression creates a compute node that can't resolve local variable references. | C10 (ReturnExprCompute desugaring) is not implemented. Complex return expressions (`BinOp`, `If`, `Match`, `Pipe`) in the lowerer silently drop or create unresolvable compute nodes. | Implement C10: desugar complex return expressions into explicit compute DAG nodes with proper variable scoping. This is the same root cause as P0-1/P0-2. | L |

**Dependency**: P0-5 (C10) is the root cause of P0-1 and P0-2. Fixing C10 properly resolves all three. P0-3 and P0-4 are independent.

---

## P1: Critical Path — Unblocks the most downstream work

| # | Task | From | What | Blocked By | Unblocks | Size |
|---|------|------|------|------------|----------|------|
| P1-1 | **C10: ReturnExprCompute desugaring** | Worker C | Desugar `BinOp`, `UnaryOp`, `If`, `Match`, `Pipe` return expressions into explicit compute nodes. Delete `ReturnExprComputeOp`. | — | P0-1, P0-2, P0-5, `make install`, `gunbc-ci` overall_success, C19 | L |
| P1-2 | **C20: CLI generator profile/mode/subcommand** | Worker C | Generated CLIs accept `--profile` enum flag, `--mode ensure\|verify`, subcommand dispatch for multi-func modules. `KEY=VALUE` arg parsing for infra-style tools. | — | A1, A2, A3, A4, A5 (all binary eliminations) | L |
| P1-3 | **C19: Restore passthrough enforcement** | Worker C | After C10 wires return expressions, restore `ExecError` for required outputs with no input. Code ref: `resolve.rs:99` TODO. | P1-1 (C10) | CI correctness, runtime soundness | S |

---

## P2: Remaining Worker C tasks (Compiler Pipeline)

Not on critical path but part of the stated plan.

| # | Task | Status | What | Size |
|---|------|--------|------|------|
| P2-1 | **C1: Stdlib host + caching** | Not started | `OnceLock` cache for compiled fn bodies. `include_str!` for stdlib sources. Delete per-module compile wrappers. | M |
| P2-2 | **C4: LoweringContext + dead code** | Not started | Context struct grouping 8-11 params. Delete 18 `#[allow(clippy::too_many_arguments)]`. | L |
| P2-3 | **C5: Integrate scope.rs** | Partial — `scope.rs` exists (615 lines) but `detect_*_branches_in_stmts`, `IfBranchSite`, `MatchBranchSite` still in lib.rs | Wire scope.rs callers. Delete `IfBranchSite`. | M |
| P2-4 | **C6: Extract transport derivation** | Not started | `transport.rs` module returning `TransportManifest`. Invariant: every service call site → exactly one triplet. | M |
| P2-5 | **C8: Delete dead AST scaffolding** | Partial — some cleanup done | Delete `MockResponseDef`, `@retry` rejection by parser, `hermetic` warning. | S |
| P2-6 | **C9: No panics, no silent parse** | Not started | `LowerError::InvalidTransportSpec` replaces `panic!`. Parser test for bad `auth_input`. | S |
| P2-7 | **C13: Split mock_defaults** | Not started | Generic probing (~350 lines) → `core/test/`. Delete GCP blob (~230 lines). | S |
| P2-8 | **C14: REST status-code checking** | Not started | `GenericRestParseOp` checks status before field extraction. Non-2xx → error. | M |
| P2-9 | **C16: Transport class metadata** | Partial — enum exists but registry gen may still use substrings | Verify `from_node_context` reads metadata, not substrings. | S |
| P2-10 | **C18: Executor dead code** | Not started | Delete `looks_effectful_without_kind()`. Delete unwired credential expiry plumbing. | S |

**Suggested sequence**: P2-2 (C4) → P2-3 (C5) → P2-4 (C6) — this is the lowerer restructuring chain. P2-1 (C1), P2-5:10 are independent.

---

## P3: Remaining Worker A tasks (Binary Elimination)

All blocked on P1-2 (C20).

| # | Task | What | Size |
|---|------|------|------|
| P3-1 | **A1: Eliminate sdlc.rs** | Move param_source propagation to `detect_entrypoints`. Delete 263-line binary. | S |
| P3-2 | **A2: Eliminate deps_config.rs** | Delete 238-line binary. Requires C20 mode flag support. | S |
| P3-3 | **A3: Eliminate pipeline.rs** | Move `query_ci_status()` to DSL func nodes. Delete 384-line binary. | M |
| P3-4 | **A4: Eliminate workflow.rs** | Move plan rendering to DSL. Delete 716-line binary. Requires C20 subcommand dispatch. | L |
| P3-5 | **A5: Eliminate infra.rs** | 8 subcommands → DSL. Delete 1,056-line binary. Requires C20 `KEY=VALUE` parsing + multi-value flags. | L |
| P3-6 | **A10: Delete BinaryArgs** | `BinaryArgs` still exists in `core/cli/src/binary_args.rs`. New `parse()` API coexists — remove old API. | S |

**Unlock sequence**: P1-2 (C20) → P3-1 (A1) → P3-2 (A2) → P3-3 (A3); P3-4 (A4) → P3-5 (A5) → P3-6 (A10)

---

## P4: Remaining Worker B tasks (Registry Deletion)

| # | Task | Status | What | Size |
|---|------|--------|------|------|
| P4-1 | **B2: Makegen registry → DSL (remaining)** | Partial — MetaTarget done, BuildConfig/ToolInfo still in Rust | Migrate `BuildConfig` struct and `ToolInfo` manual entries from `registry.rs` to DSL data. Target: `registry.rs` from current size to ~400 lines. | M |
| P4-2 | **B10: Clean shared.rs + justfile.rs** | Partial — DSL data loading works but Rust registry type refs remain | Remove `ToolInfo`, `BuildConfig` dependencies from `shared.rs` and `justfile.rs`. | S |

**Sequence**: P4-1 → P4-2

---

## P5: Blue Lane 2 — External Dependency Modeling (21 tasks, 0% done)

This is the entirely unexecuted lane. Pure DSL authoring — no Rust changes.
Each file follows the established `extdeps/` pattern (types + data, zero functions).

**Priority order** (what SDLC scenario needs first):

| # | Task | What | Size | Phase 2 Deps |
|---|------|------|------|--------------|
| P5-1 | **ED-1: `extdeps/cloud/core.dag`** | Universal cloud concepts: `Region`, `AuthScheme`, `ServiceEndpoint`, `RateLimit`, `Credential`, `IdempotencyToken`. | S | ED-9, ED-18, ED-20 |
| P5-2 | **ED-2: `extdeps/github/core.dag`** | "What is GitHub?" `Repository`, `User`, `RateLimit`, `AuthToken`, `ApiVersion`, `Pagination`. | S | ED-3, ED-4, ED-5, SL-1 |
| P5-3 | **ED-6: `extdeps/llm/core.dag`** | "What is an LLM API?" `Message`, `Role`, `TokenUsage`, `StopReason`, `Temperature`, `MaxTokens`. | S | ED-7, ED-8, SL-2 |
| P5-4 | **ED-3: `extdeps/github/issues.dag`** | "What is a GitHub Issue?" `Issue`, `IssueState`, `Label`, `IssueEvent`, `IssueComment`. | M | SL-1 |
| P5-5 | **ED-4: `extdeps/github/pull_requests.dag`** | "What is a PR?" `PullRequest`, `ReviewState`, `CheckStatus`, `MergeStrategy`. | M | SL-1 |
| P5-6 | **ED-5: `extdeps/github/gists.dag`** | "What is a Gist?" `Gist`, `GistFile`, `GistVisibility`. | S | SL-1 |
| P5-7 | **ED-7: `extdeps/llm/anthropic.dag`** | "What is the Anthropic API?" `Model`, `ContentBlock`, `SystemPrompt`, `ThinkingConfig`. | S | SL-2 |
| P5-8 | **ED-8: `extdeps/llm/openai.dag`** | "What is the OpenAI API?" `Model`, `ResponseFormat`, `ToolChoice`. | S | SL-2 |
| P5-9 | **ED-9: `extdeps/cloud/gcp/core.dag`** | "What is GCP?" `Project`, `ServiceAccount`, `OAuth2Scope`, `WifPool`. | M | ED-10:15, SL-3 |
| P5-10 | **ED-10: `extdeps/cloud/gcp/storage.dag`** | "What is GCS?" `Bucket`, `Object`, `CasPrecondition`. | M | SL-3 |
| P5-11 | **ED-11: `extdeps/cloud/gcp/pubsub.dag`** | "What is Pub/Sub?" `Topic`, `Subscription`, `AckDeadline`. | M | SL-3 |
| P5-12 | **ED-12: `extdeps/cloud/gcp/iam.dag`** | "What is GCP IAM?" `Role`, `Binding`, `Policy`. | S | SL-3 |
| P5-13 | **ED-13: `extdeps/cloud/gcp/secret_manager.dag`** | "What is Secret Manager?" `Secret`, `SecretVersion`, `RotationSchedule`. | S | SL-3 |
| P5-14 | **ED-14: `extdeps/cloud/gcp/cloud_run.dag`** | "What is Cloud Run?" `Service`, `Revision`, `TrafficSplit`. | M | SL-3 |
| P5-15 | **ED-15: `extdeps/cloud/gcp/sts.dag`** | "What is STS?" `TokenExchange`, `SubjectTokenType`, `GrantType`. | S | SL-3 |
| P5-16 | **ED-16: `extdeps/git.dag`** | "What is Git?" `Commit`, `Branch`, `Remote`, `Ref`, `MergeStrategy`. | M | SL-4 |
| P5-17 | **ED-17: `extdeps/cargo.dag`** | "What is Cargo?" `Package`, `Target`, `Profile`, `Feature`. | S | SL-4 |
| P5-18 | **ED-18: `extdeps/cloud/aws/core.dag`** | "What is AWS?" `Arn`, `Region`, `SigV4`, `AssumeRole`. | M | ED-19 |
| P5-19 | **ED-19: `extdeps/cloud/aws/*.dag`** (5 files) | AWS service models (s3, iam, lambda, secrets_manager, sqs). | L | — |
| P5-20 | **ED-20: `extdeps/cloud/azure/core.dag`** | "What is Azure?" `Subscription`, `Tenant`, `ManagedIdentity`. | M | ED-21 |
| P5-21 | **ED-21: `extdeps/cloud/azure/*.dag`** (5 files) | Azure service models. | L | — |

**Suggested batching**: P5-1:3 (3 core files, parallel) → P5-4:8 (github + llm, parallel) → P5-9:15 (GCP, sequential) → P5-16:17 (git + cargo, parallel) → P5-18:21 (AWS + Azure, low priority)

---

## P6: BT-E1 — SDLC Transport Deduplication

| # | Task | What | Size |
|---|------|------|------|
| P6-1 | **BT-E1: Fix endpoint_use_count scope** | `daglang-lower/src/lib.rs:~6019`: `endpoint_use_count` resets per module. Callables in different modules both wire literal sources to the same shared transport prepare node's scalar port. Fix: make `endpoint_use_count` global across all modules in the compiled graph. | M |

**Unblocks**: `gunbc-sdlc --dry-run` full execution (currently fails at 408/494 nodes).

---

## Execution Plan

```
                     Parallel Track A                Parallel Track B
                     ───────────────                 ───────────────
 NOW    P0-3 (ratchet baseline)  ─────┐
        P0-4 (clippy FromStr)    ─────┤
        P5-1:3 (ED core files)  ──────┤  P4-1 (B2 registry)  ──┐
                                      │  P4-2 (B10 cleanup)  ──┤
                                      │                        │
 NEXT   P1-1 (C10 ReturnExpr)   ─────┤  P5-4:8 (ED github+llm)│
         │                            │                        │
         ├→ P1-3 (C19 enforce) ──────┤                        │
         ├→ P0-5 resolved (make install works)                │
         │                            │  P5-9:15 (ED GCP)     │
         │                            │                        │
 THEN   P1-2 (C20 CLI gen)     ─────┤  P5-16:17 (ED git+cargo)
         │                            │                        │
         ├→ P3-1:3 (A1-A3)    ──────┤  P2-* (Worker C rest)  │
         ├→ P3-4:5 (A4-A5)    ──────┤                        │
         └→ P3-6 (A10)        ──────┤                        │
                                      │                        │
 LATER  P6-1 (BT-E1 dedup)    ──────┤  P5-18:21 (ED AWS+Azure)
        P2-2:4 (C4→C5→C6)     ──────┘
```

**Quick wins** (all S, can do in parallel right now):
- P0-3: Update push_str ratchet baseline
- P0-4: Implement `FromStr` for `PipeMethod`
- P5-1, P5-2, P5-3: Three ED core files (pure DSL authoring)

**Highest leverage** (unblocks the most):
- P1-1 (C10): Fixes `make install`, 2 test failures, and unblocks C19
- P1-2 (C20): Unblocks all 5 binary eliminations (A1-A5)

**Largest remaining scope**:
- P5 (ED lane): 21 files of pure DSL authoring, ~2,000-3,000 lines
- P3 (binary elimination): ~2,657 lines of deletion once C20 lands
- P2 (compiler internals): ~10 tasks of varying size

---

## Total Remaining Work Summary

| Category | Tasks | Estimated LOC Impact |
|----------|-------|---------------------|
| P0 Bugs | 5 (2 independent, 3 share root cause) | ~50 lines fix |
| P1 Critical Path | 3 | ~500-800 lines new/changed |
| P2 Worker C remaining | 10 | ~1,000 lines changed, ~500 deleted |
| P3 Worker A remaining | 6 | ~2,657 lines deleted |
| P4 Worker B remaining | 2 | ~200 lines changed, ~800 deleted |
| P5 ED Lane | 21 | ~2,500 lines new DSL |
| P6 BT-E1 | 1 | ~20 lines fix |
| **Total** | **48** | **net ~-1,500 to -2,000 LOC** |
