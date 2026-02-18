# Consolidated Worker Plan

**Status**: Working Draft — February 2026
**Companion**: [`dsl-roadmap.md`](./dsl-roadmap.md), [`dsl-codegen-roadmap.md`](./dsl-codegen-roadmap.md), [`TODO/README.md`](../../../TODO/README.md)
**Track A (DSL Core)**: DONE — compiler produces real binaries across 4 targets (see dsl-codegen-roadmap.md)

This document unifies 11 active TODO files into a single dependency-ordered execution plan.
Original TODO files retain detailed rationale; this doc provides sequencing, dependencies, and
task assignments.

---

## Current State

| Track | Status | Summary |
|-------|--------|---------|
| **A — DSL Core** | DONE | Compiler pipeline complete: parse → resolve → typecheck → lower → derive → emit. 4 targets (Rust/Go/C/MIPS), exec-runtime fast path, cross-language parity tests. |
| **B — Migration** | ~30% | Workflow audit Phases A-C done (~85%); Phase D pending. DSL migration backlog created but 0% implemented. |
| **C — Modeling** | 0% | 4 URGENT items (platform, browser, anemic, transport) — all planning/spec only, no implementation. |
| **D — Runtime/Test** | ~40% | Logging consolidation ~44% (basics done, quality/safety/tests remain). Testgen seed policy ~50% (core fix done). Codegen quality ongoing. |
| **E — Domain Parity** | ~5% | Credential lifecycle has some wiring. GCP infra at 0%. LLM review V0 complete. |
| **F — Debt Ledger** | ~55% | Hacks: 20/35 resolved. Consolidation: ongoing. |

---

## Dependency DAG

```
WAVE 1 (unblocked)          WAVE 2 (after W1)         WAVE 3 (after W2)         WAVE 4 (horizon)
─────────────────           ─────────────────         ─────────────────         ────────────────
B1.1 pragma ─────────────────────────────────────────┐
B1.2 transport triplets ─────────────────────────────┤
B1.3 codegen graph ──────────────────────────────────┼─▶ B2.1 workflow reg ───▶ B2.2 admission ctrl
B1.4 skip semantics ─────────────────────────────────┘                    ├──▶ B2.3 freshness
                                                                          └──▶ B2.4 sandbox RFC
C1.1 platform types ──────▶ C1.2 toolchain migrate ──▶ C1.3 DSL alignment
                     ├────▶ C2.1 browser utility
                     └────▶ (C3.2 partial)
C3.1 delegate macro ──────▶ C3.2 run_tool() ──────────▶ C3.3 list elimination
C4.1 transport tests ─────▶ C4.2 typed ports ─────────▶ C4.3 behavioral specs ▶ C4.4 sub-DAGs?

D1.1 DisplayConfig ───────▶ D1.4 failure-first ───────▶ D1.6 preflight ────────▶ D1.9 summaries
D1.2 secret redaction      D1.5 grouped progress       D1.7 error conventions
D1.3 stderr capture ──────▶ D1.4                        D1.8 attention fmt

D2.1 seed classify ───────▶ D2.2 matrix extend
                     └────▶ D2.3 fail closed

E1.0 cred diagnostics ────▶ E1.1 context/profile ─────▶ E1.2 policy binding ───▶ E1.3-E1.5
E2.1 SA/IAM ──────────────▶ E2.2 WIF bootstrap ───────▶ E2.3-E2.4 ────────────▶ E2.5-E2.8

[B1.5-B1.8 blocked on DSL reactive/metaprogramming primitives — Wave 3+ when ready]
```

---

## Cross-Track Dependencies

| From | To | Relationship |
|------|----|-------------|
| C1.1 | C2.1 | Platform/Env types needed for browser utility |
| C1.1 | C1.2 | Foundation types enable toolchain migration |
| C3.1 | C3.2 | Delegation macro enables binary ceremony reduction |
| C4.1 | C4.2 | Tests-first before typed decomposition |
| D1.1 + D1.3 | D1.4 | DisplayConfig + stderr capture enable failure-first rendering |
| D1.4 + D1.5 | D1.6 | Rendering + progress model enable preflight integration |
| D2.1 | D2.2, D2.3 | Shared classification enables matrix extension |
| E1.0 | E1.1 | Diagnostics baseline enables context/profile work |
| E2.1 | E2.2 | SA/IAM lifecycle enables WIF bootstrap |
| E2.2 | E1.2+ | GCP WIF needed for credential policy binding |
| B1.1-B1.4 | B2.1 | Migration experience informs workflow registry design |
| B2.1 | B2.2, B2.3 | Registry enables admission control + freshness |
| DSL core (ext) | B1.5-B1.8 | Reactive/metaprog primitives not yet in DSL |

---

## Wave Summary

| Wave | Tasks | Parallel Swimlanes | Theme |
|------|-------|--------------------|-------|
| **1** | ~20 | 5 | Foundations: unblocked migrations, modeling types, runtime basics, domain baselines |
| **2** | ~15 | 4 | Build-on: platform migrations, logging quality, credential context, WIF |
| **3** | ~15 | 4 | Completion: modeling finish, logging finish, auth policy, DSL-blocked items |
| **4** | ~12 | 3 | Horizon: credential hardening, GCP compute, sandbox RFC |

---

## Sizing & Status Key

| Size | Meaning |
|------|---------|
| S | < 1 day |
| M | 1-3 days |
| L | 3-5 days |
| XL | 5+ days |

Task status: `- [ ]` = not started, `*(TAKEN)*` = claimed, `*(DONE)*` = complete.
Completed tasks move to [Appendix: Completed Work](#appendix-completed-work).

---

## Track B — Migration Targets

### B1 — DSL Migration
**Source**: `TODO/TODO_URGENT_dsl_migration.md`

These hand-rolled Rust patterns have DSL equivalents now. Migrate them.

#### Ready Now (Wave 1)

##### B1.1 — Pragma graph DSL migration [M]
**Deps**: None

`gunbc-dag/src/pragma/graph.rs` has 3 identical content upsert chains (clippy.toml,
allowlist, lint policy). Express as DSL `pattern` invocations with service calls.

- [ ] B1.1a — Write `pragma.dag` using `pattern content_upsert` for 3 chains
- [ ] B1.1b — Verify generated binary produces identical output to hand-built
- [ ] B1.1c — Wire into build system (replace hand-built pragma binary)

##### B1.2 — Transport triplet DSL migration [M]
**Deps**: None

prepare/execute/parse 3-node pattern appears in every binary. DSL already supports
via service call lowering.

- [ ] B1.2a — Audit all transport triplet instances across binaries
- [ ] B1.2b — Verify DSL `service` call lowering produces equivalent triplet structure
- [ ] B1.2c — Migrate at least one triplet to DSL and verify parity

##### B1.3 — Codegen graph DSL migration [M]
**Deps**: None

`gunbc-dag/src/codegen/graph.rs` — staged pipeline: exists check → conditional codegen
→ stamp. DSL `if` in `func` bodies.

- [ ] B1.3a — Write `codegen.dag` expressing conditional pipeline
- [ ] B1.3b — Verify generated binary matches hand-built codegen behavior

##### B1.4 — Conditional execution / skip semantics [S]
**Deps**: None

Content upsert "compare" step skips write when content matches. May need `[skip_if]`
or equivalent DSL annotation.

- [ ] B1.4a — Determine whether existing DSL constructs handle skip semantics
- [ ] B1.4b — If needed, add skip annotation to DSL syntax + lowering
- [ ] B1.4c — Verify upsert pattern with skip produces correct generated code

#### Needs DSL Work First (Wave 3+)

##### B1.5 — Display orchestration [XL]
**Deps**: DSL reactive/streaming primitives

Channel-driven event loop with timer ticks (`core/exec/src/display.rs`). Needs reactive
DSL constructs (`observe events`, `every 80ms`).

- [ ] B1.5a — Design DSL reactive/streaming primitives
- [ ] B1.5b — Implement reactive lowering
- [ ] B1.5c — Migrate display orchestration to DSL

##### B1.6 — Testgen dynamic targets [L]
**Deps**: DSL compile-time metaprogramming

N upsert chains per `DagSpecDef` discovered via inventory. Needs compile-time
metaprogramming or inventory integration in DSL.

- [ ] B1.6a — Design DSL metaprogramming / inventory integration
- [ ] B1.6b — Migrate testgen dynamic target generation

##### B1.7 — Makegen tool registry [L]
**Deps**: DSL compile-time metaprogramming

Procedural target generation from `#[tool_target]` inventory.

- [ ] B1.7a — Migrate makegen tool discovery to DSL

##### B1.8 — Loop extra inputs passthrough [M]
**Deps**: DSL `for` lowering enhancement

`for` loops where body needs non-element context (e.g., `repo_path`). DSL `for`
lowering doesn't model passthrough inputs yet.

- [ ] B1.8a — Extend DSL `for` lowering for passthrough inputs
- [ ] B1.8b — Verify loop body can access extra context

---

### B2 — Workflow Audit Phase D
**Source**: `TODO/TODO_workflow_audit.md` (Phases A-C: DONE)

#### Wave 2

##### B2.1 — Canonical workflow registry [L]
**Deps**: B1.1-B1.4 (migration context)

Consolidate Makefile + CI + CLI to single canonical workflow registry. The DSL
migration work (B1.x) informs which workflows can be registry-driven.

- [ ] B2.1a — Define `WorkflowSpec` type with entry points, deps, resources
- [ ] B2.1b — Register all existing workflows (build, test, codegen, testgen, pragma, etc.)
- [ ] B2.1c — Generate Makefile targets from registry

#### Wave 3

##### B2.2 — Resource-conflict admission control [L]
**Deps**: B2.1

Add resource-conflict admission control to the executor so parallel DAG execution
is safe.

- [ ] B2.2a — Implement conflict detection from `ResourceAccess` declarations
- [ ] B2.2b — Add admission gating in executor before node dispatch
- [ ] B2.2c — Tests: conflicting Write/Write blocked, Read/Read allowed

##### B2.3 — Fast-path freshness [M]
**Deps**: B2.1

Git HEAD + dirty state as fast-path freshness signal before full content hashing.

- [ ] B2.3a — Design freshness signal (HEAD SHA + dirty files)
- [ ] B2.3b — Integrate into workflow registry execution path

#### Wave 4

##### B2.4 — Sandbox + durability/replay RFC [L]
**Deps**: B2.2

Design sandbox mode (no real I/O) and replay/durability for DAG execution.

- [ ] B2.4a — Draft RFC for sandbox execution model
- [ ] B2.4b — Draft RFC for durability/replay

---

## Track C — Modeling Foundation

### C1 — Platform/Toolchain Modeling
**Source**: `TODO/TODO_URGENT_platform_toolchain_modeling.md`

5+ fragmented platform models → single canonical model.

#### Wave 1

##### C1.1 — Platform/target/env foundation types [M]
**Deps**: None

Add canonical types in `core/ir` as single source of truth.

- [ ] C1.1a — Add `Arch`, `Vendor`, `Os`, `AbiEnv` enums
- [ ] C1.1b — Add `TargetTriple { arch, vendor, os, env }` struct
- [ ] C1.1c — Add `ExecutionEnv` enum (Native, WSL, Container, CI, Emulator)
- [ ] C1.1d — Add `RuntimePlatform { host: TargetTriple, env: ExecutionEnv }`
- [ ] C1.1e — Add parsing/formatting helpers for target triple strings
- [ ] C1.1f — Add compatibility adapters from `deps::Platform`, DSL `Platform`, etc.

#### Wave 2

##### C1.2 — Highest-ROI migrations [L]
**Deps**: C1.1

Replace the worst fragmentation points with canonical types.

- [ ] C1.2a — Replace hardcoded MIPS assembler/linker/qemu strings with modeled toolchain resources
- [ ] C1.2b — Replace inline browser-open platform branching with env-aware resolver
- [ ] C1.2c — Switch deps install and GH install platform keys to typed platform IDs
- [ ] C1.2d — Replace `PlatformDef` / `PlatformRegistry` stringly-typed keys

#### Wave 3

##### C1.3 — DSL + testgen alignment [M]
**Deps**: C1.1, C1.2

- [ ] C1.3a — Align DSL `Platform`/`CodegenTarget` vocabulary with canonical types
- [ ] C1.3b — Remove linux-hardcoded mock defaults in testgen
- [ ] C1.3c — Add conformance tests for linux-gnu vs other env/ABI variants

---

### C2 — Browser Modeling
**Source**: `TODO/TODO_URGENT_browser_modeling.md`

#### Wave 2

##### C2.1 — Shared cross-platform browser utility [M]
**Deps**: C1.1 (Platform/Env types)

Extract inline `execute_open_browser` from `dag_viz/graph.rs` into shared utility.

- [ ] C2.1a — Create browser-open utility in `lib/primitives` using `RuntimePlatform`
- [ ] C2.1b — Resolution table: (Platform, Env) → command (`wslview`, `xdg-open`, `open`, etc.)
- [ ] C2.1c — Migrate `dag_viz/graph.rs:451` to use shared utility
- [ ] C2.1d — Handle no-browser environments (Docker, headless CI) gracefully

---

### C3 — Anemic Modeling Audit
**Source**: `TODO/TODO_URGENT_anemic_modeling_audit.md`

Reduce O(tools x concerns) boilerplate to O(concerns).

#### Wave 1

##### C3.1 — Eliminate delegation boilerplate [L]
**Deps**: None

15+ graph files × ~200 lines of Executable/Mockable delegation boilerplate.

- [ ] C3.1a — Create `#[derive(DelegateExecutable)]` proc macro
- [ ] C3.1b — Create `#[derive(DelegateMockable)]` proc macro
- [ ] C3.1c — Migrate 2-3 graph op enums to validate macro
- [ ] C3.1d — Roll out to all remaining graph op enums

##### C3.1b — FsEnv auto-wiring extraction [M]
**Deps**: None

10+ graph builders duplicate identical FsEnv root node setup with ~20 manual
edge-wiring calls each.

- [ ] C3.1b-1 — Extract FsEnv auto-wiring as post-processing DAG builder step
- [ ] C3.1b-2 — Migrate graph builders to use auto-wiring
- [ ] C3.1b-3 — Remove duplicated FsEnv setup code

#### Wave 2

##### C3.2 — Reduce per-binary ceremony [M]
**Deps**: C3.1

13+ binaries with ~20 lines identical skeleton (arg parsing, mode selection, display).

- [ ] C3.2a — Create `run_tool()` abstraction encapsulating binary entry ceremony
- [ ] C3.2b — Derive `WorkspaceBinary` from tool registry metadata
- [ ] C3.2c — Migrate binaries to use `run_tool()`

#### Wave 3

##### C3.3 — Structural derivation [M]
**Deps**: C3.1, C3.2

Replace hardcoded lists with inventory queries.

- [ ] C3.3a — Replace hardcoded tool/binary lists (5+ files) with inventory-derived registries
- [ ] C3.3b — Consider `Box<dyn Executable>` for workspace DAG dispatch
- [ ] C3.3c — Eliminate manual `From` impls for `WorkspaceOp` (currently 9 impls + ~15 match arms)

---

### C4 — Transport DAG Migration
**Source**: `TODO/TODO_transport_dag_migration.md`

Bring the transport executor (5 monolithic dispatch functions, ~180 lines, weak test
coverage) under the DAG/testing model. Recommended approach: decomposed prepare/parse +
behavioral tests.

#### Wave 1

##### C4.1 — Fill transport executor testing gap [S]
**Deps**: None

~200 lines of behavioral tests. This is the gap that allowed the TCP timeout swap bug.

- [ ] C4.1a — TCP tests: connect success/refused, read timeout, write/roundtrip
- [ ] C4.1b — Shell tests: nonexistent command, exit code, env vars, cwd, stdin
- [ ] C4.1c — File tests: read/write/exists for edge cases

#### Wave 2

##### C4.2 — Typed port decomposition [L]
**Deps**: C4.1

Make field routing explicit with transport-specific Prepare/Parse ops.

- [ ] C4.2a — Define `PrepareTcp`, `ParseTcpResponse`, etc. ops
- [ ] C4.2b — Rename `TcpRequest.connect_timeout_ms` → `write_timeout_ms`
- [ ] C4.2c — Update triplet helpers to use typed ports

#### Wave 3

##### C4.3 — Transport behavioral specs [L]
**Deps**: C4.2

Declarative specification for transport behavior.

- [ ] C4.3a — Define `TransportBehavior` spec type
- [ ] C4.3b — Write specs for TCP, HTTP, REST, File, Shell
- [ ] C4.3c — Integrate with testgen for behavioral test generation

#### Wave 4

##### C4.4 — Full sub-DAG modeling (if needed) [XL]
**Deps**: C4.3 evaluation

Only pursue if Phase 3 evaluation shows behavioral specs are insufficient.

- [ ] C4.4a — Evaluate whether C4.3 coverage is sufficient
- [ ] C4.4b — If needed, design Value model extensions for OS handles

---

## Track D — Runtime/Test Hardening

### D1 — Logging Consolidation
**Source**: `TODO/TODO_URGENT_logging_consolidation.md`

Motivated by CI log explosion on PR #39. 9 problem areas, acceptance criteria A-I.

#### Wave 1

##### D1.1 — Unified DisplayConfig execution path [M] (~44% done)
**Deps**: None

Already partially done: local/CI/verify share one execution path, `ci.rs` no longer has
separate logic. Remaining: formalize `DisplayConfig` struct, verbosity control.

- [x] D1.1a — Unify `print_value` + `print_log_entry` *(DONE)*
- [x] D1.1b — Non-TTY observer summaries *(DONE)*
- [ ] D1.1c — Formalize `DisplayConfig` struct with mode/verbosity settings
- [ ] D1.1d — All execution paths use `DisplayConfig`

##### D1.2 — Secret redaction chokepoint [S]
**Deps**: None

- [x] D1.2a — Add `Secret` arm to `print_value` *(DONE)*
- [ ] D1.2b — Add `Value::display_redacted(&self) -> String` method
- [ ] D1.2c — Route all human-visible rendering through redaction chokepoint

##### D1.3 — Capture stdout+stderr all CI stages [S]
**Deps**: None

Build/Test/Lint capture both; Testgen/Bootstrap/Pragma/Guardrail/Verify missing stdout.

- [ ] D1.3a — Audit parse ops for missing stdout capture
- [ ] D1.3b — Add stdout capture to Testgen, Bootstrap, Pragma, Guardrail, Verify stages

#### Wave 2

##### D1.4 — Failure-first rendering + per-stage extractors [M]
**Deps**: D1.1, D1.3

Report node currently gets raw unstructured text. Need per-stage error extractors.

- [ ] D1.4a — Implement `extract_build_errors` extractor
- [ ] D1.4b — Implement `extract_test_failures` extractor
- [ ] D1.4c — Implement `extract_lint_warnings` extractor
- [ ] D1.4d — Default rendering shows failures first, detail on expand

##### D1.5 — Grouped progress model [M]
**Deps**: D1.1

Stage/task grouping for pipeline progress (CI stages, tool phases).

- [ ] D1.5a — Design grouped progress model (stage → tasks → nodes)
- [ ] D1.5b — Implement stage grouping in observer
- [ ] D1.5c — Long-running/noisy groups have expansion path

#### Wave 3

##### D1.6 — Preflight into display infrastructure [M]
**Deps**: D1.4, D1.5

Preflight currently uses raw `println!/eprint!`, bypassing CI groups and progress.

- [ ] D1.6a — Route preflight output through display/grouping infrastructure
- [ ] D1.6b — Preflight failures produce structured error output

##### D1.7 — Unified error field conventions [M]
**Deps**: D1.4

Different ops use `"report"`, `"message"`, `"stderr"`, `"error"`, `"success"`, etc.

- [ ] D1.7a — Define convention: `success: bool`, `error_summary: String`, `detail: String`
- [ ] D1.7b — Migrate existing ops to convention (incremental)

##### D1.8 — Attention-level messaging shared format [S]
**Deps**: D1.4

- [ ] D1.8a — Shared formatting path for attention-level messaging
- [ ] D1.8b — Consistent color semantics across all tools

#### Wave 4

##### D1.9 — Verification + regression tests [L]
**Deps**: D1.6, D1.7

- [ ] D1.9a — Unit tests for DisplayConfig modes + secret redaction
- [ ] D1.9b — Golden/snapshot tests for TTY/non-TTY/CI text modes
- [ ] D1.9c — Regression test for 2026-02-13 large-log failure
- [ ] D1.9d — End-to-end smoke coverage for workflow UX parity

---

### D2 — Testgen Seed Policy
**Source**: `TODO/TODO_testgen_seed_policy_postmortem.md`

Core fix landed (semantic-carrier inputs seeded correctly). 4 follow-ups.

#### Wave 1

##### D2.1 — Move seed-class classification to shared IR [S]
**Deps**: None

Currently testgen-local. Move to `core/ir` so other consumers can use it.

- [ ] D2.1a — Extract `SemanticCarrierClass` enum to `core/ir`
- [ ] D2.1b — Move classification logic from testgen to shared module

#### Wave 2

##### D2.2 — Extend seed matrix enforcement [M]
**Deps**: D2.1

Beyond current "Real single-node optional-input" slice to scenario + live-flow contexts.

- [ ] D2.2a — Define matrix for scenario context
- [ ] D2.2b — Define matrix for live-flow context
- [ ] D2.2c — Add enforcement tests

##### D2.3 — Unknown semantic carriers fail closed [S]
**Deps**: D2.1

Unrecognized semantic carrier types should fail rather than fallback to structural seed.

- [ ] D2.3a — Add test asserting unknown carrier types produce error
- [ ] D2.3b — Keep parser behavior strict (no silent fallback)

---

### D3 — Codegen Quality
**Source**: `TODO/design-codegen-quality.md`

Ongoing concern: generated code must pass linters without `#[allow(...)]`. Driven by
case studies as they arise during DSL migration work.

##### D3.1 — Cross-language idiom audit [M] (Wave 3+)
**Deps**: Active DSL backends

- [ ] D3.1a — Audit generated Rust for remaining clippy issues
- [ ] D3.1b — Audit generated Go for golint/govet issues
- [ ] D3.1c — Document IR modeling gaps discovered and fix

---

## Track E — Domain Parity

### E1 — Credential Lifecycle
**Source**: `TODO/TODO_credential_lifecycle.md`

5-layer architecture: Intent → Context → Policy → Provider Strategy → Execution.

#### Wave 1

##### E1.0 — Baseline credential diagnostics [S]
**Deps**: None

Establish what works today and identify gaps.

- [ ] E1.0a — Run `make gist-recent` with diagnostic tracing
- [ ] E1.0b — Document current credential resolution path
- [ ] E1.0c — Identify where hidden defaults exist

#### Wave 2

##### E1.1 — Context/profile precedence [M]
**Deps**: E1.0

Deterministic precedence rules for credential context resolution.

- [ ] E1.1a — Define precedence: explicit > env > profile > default
- [ ] E1.1b — Implement `ResolveContext` with file-backed profile
- [ ] E1.1c — Tests: precedence correctly applied

#### Wave 3

##### E1.1.5 — pattern/authenticate contract module [L]
**Deps**: E1.1

Central authentication pattern in `core/ir` that all credentialed flows consume.

- [ ] E1.1.5a — Define `pattern/authenticate` module with canonical chain
- [ ] E1.1.5b — Migrate gist flow to use pattern
- [ ] E1.1.5c — Migrate LLM flow to use pattern

##### E1.2 — Credential policy binding [M]
**Deps**: E1.1.5, E2.2 (GCP WIF)

- [ ] E1.2a — Define credential-policy schema
- [ ] E1.2b — Implement policy loader + binding logic
- [ ] E1.2c — Tests: policy correctly selects provider strategy

#### Wave 4

##### E1.3 — Strategy execution [L]
**Deps**: E1.2

Conditional impersonation, provider selection.

- [ ] E1.3a — Wire `ShouldImpersonate` decision point
- [ ] E1.3b — Implement provider-granted scope verification

##### E1.4 — Secret lifecycle [L]
**Deps**: E1.3

Reconcile/rotate/prune loops for secrets.

- [ ] E1.4a — Implement secret rotation handlers (Manual, GitHubPat, None)
- [ ] E1.4b — Secret provisioning DAG (provision all from spec)

##### E1.5 — Credential hardening + cutover [M]
**Deps**: E1.4

- [ ] E1.5a — `make gist-recent` works without hidden hardcoded defaults
- [ ] E1.5b — Missing scope declarations fail before outbound calls

---

### E2 — GCP Infra Parity
**Source**: `TODO/TODO_gcp_infra_parity.md`

8 phases to build out GCP infrastructure management.

#### Wave 1

##### E2.1 — SA/IAM lifecycle [L]
**Deps**: None

- [ ] E2.1a — Service Account CRUD (create, update, delete)
- [ ] E2.1b — SA IAM Bindings (who can impersonate)
- [ ] E2.1c — Expand SA spec (display_name, self_roles, wif_bindings)
- [ ] E2.1d — Expand SA Catalog (from 2 to ~8 SAs)

#### Wave 2

##### E2.2 — WIF bootstrap [L]
**Deps**: E2.1

- [ ] E2.2a — WIF Pool/Provider CRUD
- [ ] E2.2b — Bootstrap DAG (idempotent setup flow)
- [ ] E2.2c — WIF Spec (OIDC issuer, attribute mapping, conditions)

#### Wave 3

##### E2.3 — Secret Manager lifecycle [M]
**Deps**: E2.2

- [ ] E2.3a — Secret rotation handlers
- [ ] E2.3b — Secret provisioning DAG
- [ ] E2.3c — Secret fetch + direnv export integration

##### E2.4 — Environment modeling [M]
**Deps**: E2.2

- [ ] E2.4a — Environment config struct (project, region, zone, domain)
- [ ] E2.4b — Additional environments (test, prod)

#### Wave 4

##### E2.5 — InfraSpec + plan/apply [L]
**Deps**: E2.3

- [ ] E2.5a — Unified InfraSpec type
- [ ] E2.5b — Plan/apply DAG builder
- [ ] E2.5c — Infrastructure graph visualization

##### E2.6 — Compute stack [XL]
**Deps**: E2.5

Compute Engine, Cloud Run, Load Balancer, GCS bucket service interfaces.

##### E2.7 — CLI + dev experience [M]
**Deps**: E2.2

- [ ] E2.7a — Unified infra CLI (bootstrap, plan, apply, spec, graph)
- [ ] E2.7b — Enhanced login flow (verify ADC, SA impersonate, direnv)
- [ ] E2.7c — Status/health check (auth, projects, SA, secrets)

##### E2.8 — Multi-project support [L]
**Deps**: E2.5

- [ ] E2.8a — Project registry (multiple ProjectSpecs)
- [ ] E2.8b — Cross-project access + WIF bindings

---

### E3 — LLM Code Review Pipeline
**Source**: `TODO/llm-code-review-pipeline.md`

V0 complete (Tracks 2-6). Track 1 (Resource abstraction trait) still in design.

##### E3.1 — Resource abstraction trait [L] (Wave 4)
**Deps**: Design decision

- [ ] E3.1a — Design Resource trait for DAG-native resource management
- [ ] E3.1b — Implement for file/credential/network resources

---

## Track F — Debt Ledger

### F1 — Hacks
**Source**: `TODO/TODO_hacks.md`

15 open items. Grouped by theme for parallel execution.

#### Type System / Modeling (Wave 1-2)

- [ ] F1.10 — DAG typing dynamic escape hatch: add `input_mocks` type validation [M]
- [ ] F1.30 — List dual-encoding cleanup: finish removing `"List"` as type_id [S] (~70%)
- [ ] F1.31 — Cardinality test-case sampling strategy (replace hardcoded cap=64) [M]
- [ ] F1.32 — Map type_id parametric specification [M]
- [ ] F1.33 — Cardinality compositional modeling [L] (Wave 4)

#### Runtime / Safety (Wave 1-2)

- [ ] F1.6 — Mtime freshness fallback: improve diagnostic beyond eprintln [S]
- [ ] F1.23 — Strict DryRun mode: fail on missing resource wiring [S]
- [ ] F1.34 — Resource capability forgery prevention (TryFrom<Value> guard) [S]

#### Testing (Wave 1-2)

- [ ] F1.14 — Fermi guard live tests: blocked on GCP WIF + codegen for secret requirements [M]
- [ ] F1.22 — Coercion coverage test assertions: design decision needed [S]
- [ ] F1.21 — Transport executor test coverage (= C4.1) [S]

#### Code Quality (Wave 1-2)

- [ ] F1.18 — Report node structured output: stage-specific extractors (= D1.4) [M]
- [ ] F1.35 — Remove legacy batch shell helpers in gist [S] (~50%)

---

### F2 — Consolidation
**Source**: `TODO/consolidation.md`

#### Wave 1

- [ ] F2.1 — Extract generic ops: `StableHashOp` to `lib/primitives`, `DeduplicateOp` [M]

#### Wave 3-4

- [ ] F2.2 — Rendering workflows as DAGs (needs second format consumer) [L]
- [ ] F2.3 — GraphOp wrapper enum unification [M] (depends on C3.3)

---

## Parallel Swimlane Guide

### Wave 1 — 5 independent swimlanes

| Swimlane | Tasks | Focus |
|----------|-------|-------|
| **A** | B1.1, B1.2, B1.3, B1.4 | DSL migration (Ready Now) |
| **B** | C1.1, C3.1, C3.1b, C4.1 | Modeling foundations |
| **C** | D1.1, D1.2, D1.3, D2.1 | Runtime hardening |
| **D** | E1.0, E2.1 | Domain baselines |
| **E** | F1.6, F1.10, F1.23, F1.30, F2.1 | Debt cleanup |

All 5 swimlanes can run concurrently with no conflicts.

### Highest-ROI Starting Points

1. **B1.1** (Pragma DSL migration) — 3 identical upsert chains → proves DSL production readiness
2. **C3.1** (DelegateExecutable macro) — eliminates ~200 lines boilerplate across 15+ files
3. **D1.1** (DisplayConfig) — already ~44% done, unblocks entire logging chain
4. **C4.1** (Transport tests) — ~200 lines fills zero-coverage gap, unblocks C4.2-C4.3
5. **C1.1** (Platform types) — single source of truth replaces 5+ fragmented models

---

## Appendix: Completed Work

*(Completed tasks from each track will be moved here with completion dates.)*

### Track A — DSL Core
All complete. See [`dsl-codegen-roadmap.md`](./dsl-codegen-roadmap.md).

### Track B — Workflow Audit Phases A-C
- [x] Phase A: Purity boundaries (clippy disallowed_methods, clean up violations) *(DONE 2026-02-14)*
- [x] Phase B: Resource declarations on all DAG builders *(DONE 2026-02-14)*
- [x] Phase C: `#[resource_test_target]` registry + test runner + CI *(DONE 2026-02-14)*

### Track D — Logging (partial)
- [x] D1.1a: Unify `print_value` + `print_log_entry` *(DONE 2026-02-13)*
- [x] D1.1b: Non-TTY observer summaries *(DONE 2026-02-14)*
- [x] D1.2a: Add `Secret` arm to `print_value` *(DONE 2026-02-13)*
- [x] Truncate-for-report (60 lines, 500 chars/line) *(DONE 2026-02-13)*
- [x] Truncate-log-value for CI (40 lines) *(DONE 2026-02-13)*

### Track D — Testgen Seed Policy (core fix)
- [x] Core fix: semantic-carrier inputs seeded correctly for Real single-node tests *(DONE 2026-02-12)*

### Track F — Hacks (20 resolved items)
See `TODO/TODO_hacks.md` items 1-5, 7-9, 11-13, 15-17, 19-20, 24-29.
