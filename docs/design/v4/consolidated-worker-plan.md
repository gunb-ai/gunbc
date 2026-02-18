# Consolidated Worker Plan

**Status**: Working Draft — February 2026
**Companion**: [`dsl-roadmap.md`](./dsl-roadmap.md), [`dsl-codegen-roadmap.md`](./dsl-codegen-roadmap.md), [`TODO/README.md`](../../../TODO/README.md)
**Track A (DSL Core)**: DONE — compiler produces real binaries across 4 targets (see dsl-codegen-roadmap.md)

This document unifies all active TODO files into a single dependency-ordered execution plan.
Original TODO files retain detailed rationale; this doc provides sequencing, dependencies, and
task assignments.

**Guiding principle**: All new modeling and infrastructure work must be properly typed from the
ground up — types are DAGs, coercion is DAG comparison/transform, and external systems are
modeled as versioned understandings (following the-gunbai patterns). GCP and domain work is
gated on having these foundations in place.

---

## Current State

| Track | Status | Summary |
|-------|--------|---------|
| **A — DSL Core** | DONE | Compiler pipeline complete: parse → resolve → typecheck → lower → derive → emit. 4 targets (Rust/Go/C/MIPS), exec-runtime fast path, cross-language parity tests. |
| **B — Migration** | ~30% | Workflow audit Phases A-C done (~85%); Phase D pending. DSL migration backlog created but 0% implemented. |
| **C — Modeling** | ~10% | Type DAG infrastructure exists (`Dag<TypeOp>`, contract tower L1-L3, `can_safely_coerce_to`). Coercion Phases 1-2 done, Phases 3-4 deferred. No understanding layer. No workspace model (45+ hardcoded path assumptions). Platform/browser/anemic/transport at 0%. |
| **D — Runtime/Test** | ~40% | Logging consolidation ~44% (basics done, quality/safety/tests remain). Testgen seed policy ~50% (core fix done). Codegen quality ongoing. |
| **E — Domain Parity** | ~5% | Credential lifecycle has some wiring. GCP infra at 0%. LLM review V0 complete. **Gated on C5+C6 foundations.** |
| **F — Debt Ledger** | ~55% | Hacks: 20/35 resolved. Consolidation: ongoing. |

---

## Dependency DAG

```
WAVE 0 (type foundations)   WAVE 1 (model + migrate)   WAVE 2 (build-on)          WAVE 3+4 (complete)
─────────────────────────   ────────────────────────   ──────────────────         ─────────────────
C5.1 coerce via DAG walk ─▶ C5.3 stress tests ─────────────────────────────────▶ (validates all)
C5.2 eliminate dual enc  ─▶ C5.3
C6.1 understanding types ─▶ C6.2 GCP understandings ─▶ C6.3 contract tests ────▶ C6.4 multi-cloud
                     ├────▶ E2.1* SA/IAM (via C6)  ──▶ E2.2 WIF ──────────────▶ E2.3-E2.8
                     └────▶ C6.2b transport specs

B1.1 pragma ──────────────────────────────────────────┐
B1.2 transport triplets ──────────────────────────────┤
B1.3 codegen graph ───────────────────────────────────┼─▶ B2.1 workflow reg ───▶ B2.2-B2.4
B1.4 skip semantics ──────────────────────────────────┘

C1.1 platform types ──────▶ C1.2 toolchain migrate ──▶ C1.3 DSL alignment
                     ├────▶ C2.1 browser utility
                     └────▶ (C3.2 partial)
C3.1 delegate macro ──────▶ C3.2 run_tool() ──────────▶ C3.3 list elimination
C4.1 transport tests ─────▶ C4.2 typed ports ─────────▶ C4.3 behavioral specs

C7.1 WorkspaceLayout ────▶ C7.2 fix gen Cargo.toml
                     ├───▶ C7.3 replace glob consts ──▶ C7.5 pragma policy paths
                     └───▶ C7.4 replace parent() chains ▶ C7.5

D1.1 DisplayConfig ───────▶ D1.4 failure-first ───────▶ D1.6-D1.9
D1.2 secret redaction      D1.5 grouped progress
D1.3 stderr capture ──────▶ D1.4
D3.1 unify triplets ─────▶ D3.2 obligation-based kind ─▶ D3.3 idiom audit

E1.0 cred diagnostics ────▶ E1.1 context/profile ─────▶ E1.2 policy binding ───▶ E1.3-E1.5

* E2.1+ now depends on C6.1 (GCP services modeled as understandings, not ad-hoc)
```

---

## Cross-Track Dependencies

| From | To | Relationship |
|------|----|-------------|
| **C5.1** | **C5.3, C6.3** | DAG-based coercion validates understanding type flows |
| **C5.2** | **C5.1** | Dual encoding elimination simplifies coercion |
| **C6.1** | **C6.2, E2.1+** | Understanding types must exist before GCP services are modeled |
| **C6.2** | **C6.3** | Understandings enable contract test generation |
| C1.1 | C2.1 | Platform/Env types needed for browser utility |
| C1.1 | C1.2 | Foundation types enable toolchain migration |
| C3.1 | C3.2 | Delegation macro enables binary ceremony reduction |
| C4.1 | C4.2 | Tests-first before typed decomposition |
| D1.1 + D1.3 | D1.4 | DisplayConfig + stderr capture enable failure-first rendering |
| E1.0 | E1.1 | Diagnostics baseline enables context/profile work |
| E2.1 | E2.2 | SA/IAM lifecycle enables WIF bootstrap |
| E2.2 | E1.2+ | GCP WIF needed for credential policy binding |
| B1.1-B1.4 | B2.1 | Migration experience informs workflow registry design |
| **D3.1** | **D3.2** | Unified triplet data in derive enables obligation-based canonical kind |
| **C5.1** | **D3.2** | DAG-based coercion matures obligation metadata used for canonical kind |
| **C7.1** | **C7.2, C7.3, C7.4** | WorkspaceLayout enables all downstream path fixes |
| **C7.4** | **C7.5** | parent() chain elimination enables pragma policy derivation |
| DSL core (ext) | B1.5-B1.8 | Reactive/metaprog primitives not yet in DSL |

---

## Wave Summary

| Wave | Tasks | Parallel Swimlanes | Theme |
|------|-------|--------------------|-------|
| **0** | ~10 | 3 | **Type + workspace foundations**: DAG-based coercion, understanding layer, workspace model. Validates modeling approach before building on it. |
| **1** | ~28 | 6 | Model + migrate: GCP understandings, DSL migrations, platform types, runtime basics, unify triplet analysis, replace glob constants + parent() chains |
| **2** | ~20 | 4 | Build-on: contract tests, platform migrations, logging quality, credential context, obligation-based canonical kind, pragma policy derivation |
| **3** | ~15 | 4 | Completion: multi-cloud stress tests, logging finish, auth policy |
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

- [x] B2.2a — Implement conflict detection from `ResourceAccess` declarations
- [x] B2.2b — Add admission gating in executor before node dispatch
- [x] B2.2c — Tests: conflicting Write/Write blocked, Read/Read allowed

##### B2.3 — Fast-path freshness [M]
**Deps**: B2.1

Git HEAD + dirty state as fast-path freshness signal before full content hashing.

- [x] B2.3a — Design freshness signal (HEAD SHA + dirty files)
- [ ] B2.3b — Integrate into workflow registry execution path *(preflight path now uses signal cache; full workflow-registry integration pending B2.1)*

#### Wave 4

##### B2.4 — Sandbox + durability/replay RFC [L]
**Deps**: B2.2

Design sandbox mode (no real I/O) and replay/durability for DAG execution.

- [x] B2.4a — Draft RFC for sandbox execution model
- [x] B2.4b — Draft RFC for durability/replay

---

## Track C — Modeling Foundation

### C5 — Type DAG Coercion (Wave 0)
**Source**: `TODO/TODONE/design-type-coercion.md` Phases 3-4 (deferred), `core/ir/src/type_op.rs`,
`core/ir/src/contract.rs`, `core/ir/src/coerce.rs`

Types are already DAGs (`Dag<TypeOp>`) with a contract tower (L1 cardinality, L2 base type,
L3 predicates). Phases 1-2 done: contract extraction + `can_safely_coerce_to()`. But coercion
currently uses hardcoded rules, not DAG comparison. This must work before we build on it.

**Existing infrastructure**:
- `TypeOp` enum: `Identity`, `Validate(Predicate)`, `Transform(Coercion)`, `Wrap(WrapperKind)`, `Unwrap`
- `TypeRegistry`: maps `TypeId` → `Dag<TypeOp>`
- `TypeContract`: extracted cardinality + base_type + predicates
- `base_type_upcasts_to()`: hardcoded lattice (everything → Json, Url → String)
- Type library: `type_lib::url()`, `type_lib::string()`, etc. — constructors for common type DAGs

#### Wave 0

##### C5.1 — Coercion as DAG walk [M]
**Deps**: C5.2

Given source type DAG and target type DAG, find a valid transform path by walking both
DAGs, not by checking hardcoded rules.

- [x] C5.1a — Replace `base_type_upcasts_to()` with registry-driven DAG ancestry check
- [x] C5.1b — Coercion discovery: given `Dag<TypeOp>` for source and target, find the
      transform chain (e.g., Url→String = unwrap NonEmpty + unwrap Matches)
- [x] C5.1c — `TypeOp::Transform(Coercion)` used as explicit coercion edges in registry
- [x] C5.1d — Tests: Url→String, Int→Json, String→Json coercion found via DAG walk
- [x] C5.1e — Tests: String→Url coercion correctly rejected (narrowing = unsafe)

##### C5.2 — Eliminate cardinality dual encoding [M]
**Deps**: None

Port cardinality and type DAG `Wrap` nodes encode the same information independently.
Derive port cardinality from the type DAG so there's one source of truth.

- [x] C5.2a — `infer_cardinality()` from type DAG `Wrap`/`Unwrap` nodes
- [ ] C5.2b — Audit ports that set cardinality independently of type DAG
- [ ] C5.2c — Migrate to single source: type DAG drives cardinality, port just references type
- [x] C5.2d — Tests: `Optional<String>` type DAG → port cardinality [0,1] automatically

##### C5.3 — Type system stress tests [M]
**Deps**: C5.1, C5.2

Validate that the DAG-based coercion handles real-world type relationships.

- [ ] C5.3a — Multi-step coercion: `NonEmptyUrl` → `Url` → `String` → `Json`
- [ ] C5.3b — Container coercion: `List<Url>` → `List<String>` (covariant)
- [ ] C5.3c — Optional unwrap: `Optional<String>` → `String` (requires value present)
- [ ] C5.3d — Map coercion: `Map<String, Url>` → `Map<String, String>`
- [ ] C5.3e — Cross-provider type alignment: `GcpSecretPayload` refines `String`,
      `AwsSecretValue` refines `String` — both coerce to `String` but not to each other
- [ ] C5.3f — Credential coercion: `GcpAccessToken` and `AwsSessionToken` both refine
      `Credential` but are not interchangeable

---

### C6 — Understanding / Modeling Layer (Wave 0-1)
**Source**: the-gunbai `understanding/` pattern

External systems must be modeled as typed, versioned understandings — not ad-hoc code.
This is the foundation for all GCP infra, credential, and transport work.

#### Wave 0

##### C6.1 — Understanding type definitions [L]
**Deps**: None

Port the core understanding types from the-gunbai. Create `core/understanding` crate
(or module in `core/ir`).

- [ ] C6.1a — Define `Understanding` struct: id, name, kind, version, docs, behaviors,
      constraints, assumptions, unknowns, depends_on
- [ ] C6.1b — Define `SystemKind` enum: Cli, RestApi, LlmApi, Sdk, SecretProvider,
      Convention, Queue, Scheduler, Runner
- [ ] C6.1c — Define `Behavior` struct: id, description, invocation, inputs, outputs,
      observed_properties, requires, upsert_phase
- [ ] C6.1d — Define `Invocation` enum: Cli (with docs), Rest (with docs), Sdk, Protocol
- [ ] C6.1e — Define `Property` enum: ReadOnly, WritesWorld, Deterministic, Idempotent,
      IdempotentWithKey, FailsWhen, EdgeCase, etc.
- [ ] C6.1f — Define `InputType`/`OutputType` enums with mapping to `TypeId`/`Dag<TypeOp>`
- [ ] C6.1g — `inventory`-based auto-registration: `submit_understanding!` macro
- [ ] C6.1h — Tests: define a minimal understanding, verify registration + retrieval

#### Wave 1

##### C6.2 — First understandings: GCP + transport [L]
**Deps**: C6.1

Model the first real external systems as understandings. These validate the framework
and provide the typed foundation for E2 (GCP infra) and C4 (transport).

- [ ] C6.2a — GCP Secret Manager understanding (access_secret_version, list_secrets,
      create_secret, destroy_secret_version)
- [ ] C6.2b — GCP IAM understanding (SA CRUD, IAM bindings, WIF pool/provider)
- [ ] C6.2c — GCS understanding (get, put, list, delete + versioned CAS)
- [ ] C6.2d — File transport understanding (read, write, exists, delete)
- [ ] C6.2e — Shell transport understanding (exec with args, env, cwd, timeout)
- [ ] C6.2f — HTTP/REST transport understanding (GET, POST, PUT, DELETE with status semantics)
- [ ] C6.2g — Dependency graph: GCP Secret Manager depends_on secret:GOOGLE_APPLICATION_CREDENTIALS
- [ ] C6.2h — Tests: all understandings parseable, dependency graph acyclic

#### Wave 2

##### C6.3 — Contract test generation [L]
**Deps**: C6.2, C5.1

Auto-generate behavioral contract tests from understanding specs.

- [ ] C6.3a — Define `ContractTestSpec` from Behavior + observed_properties
- [ ] C6.3b — Upsert phase enforcement: Check=ReadOnly+Deterministic,
      Create=IdempotentWithKey, Resolve=ReadOnly+FailsWhen
- [ ] C6.3c — Generate type-safe test harnesses from InputType/OutputType
- [ ] C6.3d — Wire into testgen for understanding-driven test generation

#### Wave 3

##### C6.4 — Multi-cloud stress test understandings [L]
**Deps**: C6.2, C5.3

Model a second cloud provider to validate that the understanding + type system
handles cross-provider concerns correctly.

- [ ] C6.4a — AWS Secrets Manager understanding (get_secret_value, create_secret,
      put_secret_value, describe_secret)
- [ ] C6.4b — AWS IAM understanding (role CRUD, policy attachment, assume-role)
- [ ] C6.4c — S3 understanding (get_object, put_object, list_objects + versioned CAS)
- [ ] C6.4d — Type alignment test: GCP SecretPayload and AWS SecretValue both refine
      String but are not mutually coercible
- [ ] C6.4e — Cross-provider credential test: `GcpAccessToken` vs `AwsSessionToken` —
      both satisfy `Credential` interface but different provider strategies
- [ ] C6.4f — Storage abstraction test: `Store` trait behaviors map to both GCS and S3
      understandings with correct property preservation (CAS atomicity, TTL semantics)

---

### C1 — Platform/Toolchain Modeling
**Source**: `TODO/TODO_URGENT_platform_toolchain_modeling.md`

5+ fragmented platform models → single canonical model.

#### Wave 1

##### C1.1 — Platform/target/env foundation types [M]
**Deps**: None

Add canonical types in `core/ir` as single source of truth.

- [x] C1.1a — Add `Arch`, `Vendor`, `Os`, `AbiEnv` enums
- [x] C1.1b — Add `TargetTriple { arch, vendor, os, env }` struct
- [x] C1.1c — Add `ExecutionEnv` enum (Native, WSL, Container, CI, Emulator)
- [x] C1.1d — Add `RuntimePlatform { host: TargetTriple, env: ExecutionEnv }`
- [x] C1.1e — Add parsing/formatting helpers for target triple strings
- [x] C1.1f — Add compatibility adapters from `deps::Platform`, DSL `Platform`, etc.

#### Wave 2

##### C1.2 — Highest-ROI migrations [L]
**Deps**: C1.1

Replace the worst fragmentation points with canonical types.

- [x] C1.2a — Replace hardcoded MIPS assembler/linker/qemu strings with modeled toolchain resources
- [x] C1.2b — Replace inline browser-open platform branching with env-aware resolver
- [x] C1.2c — Switch deps install and GH install platform keys to typed platform IDs
- [x] C1.2d — Replace `PlatformDef` / `PlatformRegistry` stringly-typed keys

#### Wave 3

##### C1.3 — DSL + testgen alignment [M]
**Deps**: C1.1, C1.2

- [x] C1.3a — Align DSL `Platform`/`CodegenTarget` vocabulary with canonical types
- [x] C1.3b — Remove linux-hardcoded mock defaults in testgen
- [x] C1.3c — Add conformance tests for linux-gnu vs other env/ABI variants

---

### C2 — Browser Modeling
**Source**: `TODO/TODO_URGENT_browser_modeling.md`

#### Wave 2

##### C2.1 — Shared cross-platform browser utility [M]
**Deps**: C1.1 (Platform/Env types)

Extract inline `execute_open_browser` from `dag_viz/graph.rs` into shared utility.

- [x] C2.1a — Create browser-open utility in `lib/primitives` using `RuntimePlatform`
- [x] C2.1b — Resolution table: (Platform, Env) → command (`wslview`, `xdg-open`, `open`, etc.)
- [x] C2.1c — Migrate `dag_viz/graph.rs:451` to use shared utility
- [x] C2.1d — Handle no-browser environments (Docker, headless CI) gracefully

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

- [x] C4.1a — TCP tests: connect success/refused, read timeout, write/roundtrip
- [x] C4.1b — Shell tests: nonexistent command, exit code, env vars, cwd, stdin
- [x] C4.1c — File tests: read/write/exists for edge cases

#### Wave 2

##### C4.2 — Typed port decomposition [L]
**Deps**: C4.1

Make field routing explicit with transport-specific Prepare/Parse ops.

- [x] C4.2a — Define `PrepareTcp`, `ParseTcpResponse`, etc. ops
- [x] C4.2b — Rename `TcpRequest.connect_timeout_ms` → `write_timeout_ms`
- [x] C4.2c — Update triplet helpers to use typed ports

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

### C7 — Workspace Model (Wave 0-1)
**Source**: Code audit (2026-02-17) — 30+ sites hardcode repo-structure assumptions

No workspace abstraction exists today. Every site that needs a crate location, a source glob,
or a relative path between two points independently hardcodes string constants and
`parent().parent()` chains. This makes the repo structure a hidden, implicit dependency —
exactly the kind of stringly-typed modeling we're eliminating everywhere else.

**Current debt** (full inventory):

| Category | Sites | Example |
|----------|-------|---------|
| Generated Cargo.toml path deps | 3 | `../../core/ir` in `emit_cargo_toml()` |
| Glob constants with baked-in paths | 6 | `CODEGEN_INPUT_GLOBS`, `REPO_SOURCE_INPUT_GLOBS`, `TESTGEN_INPUT_GLOBS` |
| `CARGO_MANIFEST_DIR` + hardcoded `.join()` | 24 | `join("../../../dsl")`, `join("dsl/tools")` |
| `parent().parent()` repo root discovery | 3 | `lib/transport/src/pragma_lint.rs` |
| Pragma allowlist path prefixes | 5 | `"core/daglang/"`, `"core/exec/src/freshness.rs"` |
| Hardcoded output dir constants | 4 | `CODEGEN_OUT_DIR = "target/codegen"` |
| Hardcoded source root lists | 2 | `["core", "gunbc-dag", "lib"]` |

**Already clean** (for reference): `path_utils.rs` (pure, takes `cwd`), `main.rs` (boundary
calls `current_dir()` once), `resolve_workspace_packages()` (uses `cargo metadata`).

#### Wave 0

##### C7.1 — WorkspaceLayout type [M]
**Deps**: None

Define a `WorkspaceLayout` struct that knows where things are, derived from `cargo metadata`
or a workspace manifest — not hardcoded strings.

- [x] C7.1a — Define `WorkspaceLayout` type: workspace root, crate locations (name → path),
  source roots, output directories
- [x] C7.1b — Constructor from `cargo metadata` (runtime) and from `env!("CARGO_MANIFEST_DIR")`
  (compile-time, with depth parameter)
- [x] C7.1c — `relative_path(&self, from: &Path, to: &Path) -> PathBuf` — compute relative
  path between any two workspace locations
- [x] C7.1d — `source_globs(&self, crates: &[&str]) -> Vec<String>` — derive glob patterns
  from crate locations instead of hardcoding them

##### C7.2 — Fix generated Cargo.toml path deps [S]
**Deps**: C7.1

The immediate bug: `emit_cargo_toml()` in `rust_exec_runtime.rs` hardcodes `../../core/ir`.

- [x] C7.2a — `emit_cargo_toml()` takes output directory + workspace layout, computes
  relative deps from actual locations
- [x] C7.2b — Test: generate into arbitrary depth directory, `cargo check` succeeds
- [x] C7.2c — Remove the depth-2 assumption documented in `cli_commands.rs` e2e test

#### Wave 1

##### C7.3 — Replace glob constants with derived patterns [M]
**Deps**: C7.1

Eliminate `CODEGEN_INPUT_GLOBS`, `REPO_SOURCE_INPUT_GLOBS`, `TESTGEN_INPUT_GLOBS` etc.
Derive them from `WorkspaceLayout` crate locations.

- [x] C7.3a — Replace `CODEGEN_INPUT_GLOBS` / `CODEGEN_INPUT_FILES` in `resource/defs.rs`
- [x] C7.3b — Replace `REPO_SOURCE_INPUT_GLOBS` / `REPO_CONFIG_INPUT_FILES` in `resources.rs`
- [x] C7.3c — Replace `TESTGEN_INPUT_GLOBS` / `TESTGEN_EXTRA_FILES` in `resources.rs`
- [x] C7.3d — Verify freshness hashing unchanged (same files discovered, different derivation)

##### C7.4 — Replace parent() chains and hardcoded joins [M]
**Deps**: C7.1

Eliminate `CARGO_MANIFEST_DIR` + `join("../../..")` patterns and `parent().parent()` chains.

- [x] C7.4a — Replace `dsl_tools_root()` / `dsl_pipelines_root()` in `subdags/mod.rs`
- [x] C7.4b — Replace `workspace_root()` helpers in daglang-cli tests (7+ sites)
- [x] C7.4c — Replace `repo_root()` in `lib/transport/src/pragma_lint.rs`
- [x] C7.4d — Replace hardcoded `CODEGEN_OUT_DIR` / `CODEGEN_BIN_DIR` constants

#### Wave 2

##### C7.5 — Replace pragma policy path prefixes [S]
**Deps**: C7.4

Pragma allowlist paths (`"core/daglang/"`, `"core/exec/src/freshness.rs"`) should derive from
crate names, not path strings.

- [x] C7.5a — Allowlist entries keyed by crate name, resolved to paths via `WorkspaceLayout`
- [x] C7.5b — `PRAGMA_LINT_POLICY.allow_dead_code` paths derived from crate locations
- [x] C7.5c — If a crate moves, policy updates automatically (no manual path editing)

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
- [x] D1.1c — Formalize `DisplayConfig` struct with mode/verbosity settings
- [x] D1.1d — All execution paths use `DisplayConfig`

##### D1.2 — Secret redaction chokepoint [S]
**Deps**: None

- [x] D1.2a — Add `Secret` arm to `print_value` *(DONE)*
- [x] D1.2b — Add `Value::display_redacted(&self) -> String` method
- [x] D1.2c — Route all human-visible rendering through redaction chokepoint

##### D1.3 — Capture stdout+stderr all CI stages [S]
**Deps**: None

All CI parse stages now capture both stdout and stderr, including Verify sub-checks.

- [x] D1.3a — Audit parse ops for missing stdout capture
- [x] D1.3b — Add stdout capture to Testgen, Bootstrap, Pragma, Guardrail, Verify stages

#### Wave 2

##### D1.4 — Failure-first rendering + per-stage extractors [M]
**Deps**: D1.1, D1.3

Report node currently gets raw unstructured text. Need per-stage error extractors.

- [x] D1.4a — Implement `extract_build_errors` extractor
- [x] D1.4b — Implement `extract_test_failures` extractor
- [x] D1.4c — Implement `extract_lint_warnings` extractor
- [x] D1.4d — Default rendering shows failures first, detail on expand

##### D1.5 — Grouped progress model [M]
**Deps**: D1.1

Stage/task grouping for pipeline progress (CI stages, tool phases).

- [x] D1.5a — Design grouped progress model (stage → tasks → nodes)
- [x] D1.5b — Implement stage grouping in observer
- [x] D1.5c — Long-running/noisy groups have expansion path

#### Wave 3

##### D1.6 — Preflight into display infrastructure [M]
**Deps**: D1.4, D1.5

Preflight currently uses raw `println!/eprint!`, bypassing CI groups and progress.

- [x] D1.6a — Route preflight output through display/grouping infrastructure
- [x] D1.6b — Preflight failures produce structured error output

##### D1.7 — Unified error field conventions [M]
**Deps**: D1.4

Different ops use `"report"`, `"message"`, `"stderr"`, `"error"`, `"success"`, etc.

- [x] D1.7a — Define convention: `success: bool`, `error_summary: String`, `detail: String`
- [x] D1.7b — Migrate existing ops to convention (incremental)

##### D1.8 — Attention-level messaging shared format [S]
**Deps**: D1.4

- [x] D1.8a — Shared formatting path for attention-level messaging
- [x] D1.8b — Consistent color semantics across all tools

#### Wave 4

##### D1.9 — Verification + regression tests [L]
**Deps**: D1.6, D1.7

- [x] D1.9a — Unit tests for DisplayConfig modes + secret redaction
- [x] D1.9b — Golden/snapshot tests for TTY/non-TTY/CI text modes
- [x] D1.9c — Regression test for 2026-02-13 large-log failure
- [x] D1.9d — End-to-end smoke coverage for workflow UX parity

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

##### D3.1 — Unify triplet analysis in derive [S] (Wave 1)
**Deps**: None

Transport triplet detection currently lives in `daglang-cli` (`compile/triplets.rs`), duplicating
analysis that should be part of `DerivedArtifacts` in `daglang-derive`. CLI should be a pure renderer.

- [ ] D3.1a — Add `transport_triplets: Vec<TransportTriplet>` to `DerivedArtifacts`
- [ ] D3.1b — Move `collect_transport_triplets` logic into `daglang-derive`
- [ ] D3.1c — Update `daglang-cli` `show-triplets` to render from derived data

##### D3.2 — Obligation-based canonical kind classification [M] (Wave 2)
**Deps**: C5.1 (coercion via DAG walk), D3.1

`canonical_kind_from_shape` in `daglang-lower` uses node-ID prefix heuristics (`prepare_*`,
`execute_transport_*`, etc.) to classify canonical kinds. As `ObligationCategory` coverage grows,
canonical kind should derive from structural obligation metadata instead.

- [ ] D3.2a — Map `ObligationCategory` variants to canonical kind strings
- [ ] D3.2b — Replace prefix-heuristic branches in `canonical_kind_from_shape` with obligation lookups
- [ ] D3.2c — Verify parity snapshots unchanged (same classification, different derivation)

##### D3.3 — Cross-language idiom audit [M] (Wave 3+)
**Deps**: Active DSL backends

- [ ] D3.3a — Audit generated Rust for remaining clippy issues
- [ ] D3.3b — Audit generated Go for golint/govet issues
- [ ] D3.3c — Document IR modeling gaps discovered and fix

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

8 phases to build out GCP infrastructure management. **All GCP work builds on the
understanding layer (C6)** — services are modeled as typed understandings first,
then implemented against those specs.

#### Wave 1

##### E2.1 — SA/IAM lifecycle [L]
**Deps**: C6.1 (understanding types), C6.2b (GCP IAM understanding)

Implementation of SA/IAM operations against the typed understanding spec from C6.2b.

- [ ] E2.1a — Service Account CRUD (create, update, delete) — impl against IAM understanding
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
- [x] F1.21 — Transport executor test coverage (= C4.1) [S]

#### Code Quality (Wave 1-2)

- [x] F1.18 — Report node structured output: stage-specific extractors (= D1.4) [M]
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

### Wave 0 — 3 independent swimlanes (START HERE)

| Swimlane | Tasks | Focus |
|----------|-------|-------|
| **Type System** | C5.1, C5.2 | DAG-based coercion + eliminate dual encoding |
| **Understanding** | C6.1 | Understanding type definitions + registration |
| **Workspace** | C7.1, C7.2 | WorkspaceLayout type + fix generated Cargo.toml path deps |

These three swimlanes are independent and can run concurrently. **Wave 0 must complete
before Wave 1 domain work (E2) begins**, because GCP services must be modeled as
understandings. C7.1+C7.2 are unblocked now and fix a concrete bug (generated binaries
only work at depth-2).

### Wave 1 — 6 independent swimlanes

| Swimlane | Tasks | Focus |
|----------|-------|-------|
| **A** | B1.1, B1.2, B1.3, B1.4 | DSL migration (Ready Now) |
| **B** | C1.1, C3.1, C3.1b, C4.1 | Modeling foundations |
| **C** | D1.1, D1.2, D1.3, D2.1, D3.1 | Runtime hardening + unify triplet analysis |
| **D** | C5.3, C6.2 | Type stress tests + first understandings (GCP, transport) |
| **E** | E1.0, E2.1, F1.x | Domain baselines (E2.1 requires C6.1+C6.2b), debt cleanup |
| **F** | C7.3, C7.4 | Replace glob constants + parent() chains (requires C7.1) |

### Highest-ROI Starting Points

1. **C5.1+C5.2** (Type DAG coercion) — validates the core modeling approach before everything else
2. **C6.1** (Understanding types) — creates the foundation for all external system modeling
3. **C7.1+C7.2** (Workspace model) — fixes concrete bug, eliminates 45+ hardcoded path assumptions
4. **C6.2** (GCP + transport understandings) — first real systems modeled properly, unblocks E2
5. **C5.3** (Stress tests) — multi-step coercion, cross-provider types, container covariance
6. **B1.1** (Pragma DSL migration) — highest-ROI migration, proves DSL production readiness

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
