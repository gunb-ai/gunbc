# Tasks

**Verification**: `cargo test --workspace` + `cargo clippy --all-targets -- -D warnings`
**Sizing**: S (<1 day), M (1-3 days), L (3-5 days), XL (5+ days)

---

## Roadmap: SDLC in Pure DSL (~10 weeks)

Three parallel tracks: foundation cleanup (delete Rust, add compiler features),
SDLC activation (pipeline runs e2e), and black-box testing (cross-workflow mock
corpus + transport fidelity). Model first, delete along the way.

```
              Foundation                           SDLC
        ┌─────────────────┐               ┌──────────────────┐
  DONE  │ FC-CL ✓  FC-NF7✓│               │ SDLC-1 (catalog) │
        ├─────────────────┤───────────────▶│ SDLC-5 (signal)  │
  NOW   │ FC-EG (gates)   │               │ SDLC-6 (artifact)│
        │ FC-WM (minimal) │               │        │         │
        │ FC-P6 (policy)  │               │ SDLC-2 (dispatch)│
        │ FC-P7 (registry)│               │ SDLC-3 (validate)│
        │        │        │               │ SDLC-4 (testing) │
        │ FC-CF (compiler)│               │        │         │
        │        │        │               │ SDLC-7 (verify)  │
  LAST  │ FC-P8 (anemic)  │               │ SDLC-8 (local e2e)│
        └─────────────────┘               │        │         │
                                          │ SDLC-CD (cloud)  │
                                          └──────────────────┘

         Testing (independent)
        ┌──────────────────┐
        │ BB-0 (modeling)  │
        │ BB-1 (corpus)    │
        │ BB-2 (per-node)  │
        │ BB-3 (adjacent)  │
        │ BB-4 (types)     │
        │ BB-5 (cross-wf)  │
        │ BB-6 (fidelity)  │
        └──────────────────┘
```

SDLC-1:6 can start immediately (no foundation dependency).
FC-EG (enforcement gates) can start immediately — no deps, prevents regression.
FC-NF7 DONE — fn-level evaluation landed, render.rs deleted (~1200 lines).
FC-CL DONE — dead code cleanup (tool-registry crates, orphaned spec builders, stale rules).
FC-P6 and FC-P7 are UNBLOCKED — fn eval works, can convert remaining extern bridges.
FC-WM (workflow minimality) can start immediately — no foundation dependency.
FC-CF runs in parallel with P6/P7.
FC-P8 requires FC-P6 + FC-P7 + FC-CF (split, zip, recursion at minimum).
BB-0:6 (black-box testing) runs independently — no Foundation or SDLC dependency.

---

## Foundation: Enforcement Gates (FC-EG)

Automated ratchets that prevent modeling regression. Cheap to add, high leverage.

| ID | Task | Size | Status | Deps |
|----|------|------|--------|------|
| FC-EG1 | Import-direction lint: compiler check that `tools→config→extdeps→std` import direction is never violated. Error on backward imports. | M | **Done** | — |
| FC-EG2 | Extern func count gate: CI test that counts `extern func` declarations in `.dag` files. Assert count ≤ current (ratchet — count only goes down). | S | **Done** | — |
| FC-EG3 | `format!/push_str` boundary gate: grep + allowlist test. No new `format!()` or `push_str()` in non-boundary Rust code (allowlist for transport, codegen, existing scaffolding). | S | **Done** | — |

---

## Foundation: Policy Migration (FC-P6)

Move policy data from Rust const arrays to DSL. Eliminate 3 extern bridges
(render_clippy_toml, render_disallowed_methods_allowlist, render_pragma_lint_policy).
No new compiler features needed. Detail: `docs/design/v4/extern-bridge-gap-analysis.md` § Phase 6.

**Scaffolding exists**: `dsl/config/clippy_disallowed.dag` (38 disallowed methods in 4
groups + 8 disallowed types), `dsl/config/clippy_policy.dag` (rendering helpers +
derive_clippy_toml/derive_disallowed_methods_allowlist/derive_pragma_lint_policy fns),
`dsl/config/arch_rules.dag` (AllowlistPattern type + 19 allowlist_patterns entries).
FC-NF7 is done — these fns can now execute at runtime via FnBodyDelegate.

| ID | Task | Size | Status | Deps |
|----|------|------|--------|------|
| FC-P6-0 | Validate flat_map: DSL test exercising `CollectionOpKind::FlatMap` e2e. | S | **Done** | — |
| FC-P6-a | `dsl/config/workspace.dag`: CrateSpec type + workspace_crates data + CI drift test. | M | **Done** | — |
| FC-P6-b | `dsl/config/pragma_policy.dag`: AllowlistRule, DeadCodeRule types + data from pragma.rs. Partial: clippy_disallowed.dag + arch_rules.dag AllowlistPattern data already exist. | M | **Done** | FC-P6-a |
| FC-P6-c | DSL policy rendering fns using Document types. Parity tests: allowlist + lint_policy match Rust output byte-for-byte. clippy_toml blocked on sum type variant tags (FC-CF5). | M | **Done** | FC-P6-a, FC-P6-b, FC-P6-0 |
| FC-P6-d | Delete 2 of 3 pragma extern impls (allowlist, lint_policy → DSL eval). clippy_toml remains (blocked on FC-CF5). `all_extern_symbols()`: 8→6. Golden parity tests gate transition. | S | **Done** | FC-P6-c |

---

## Foundation: Registry Migration (FC-P7)

Move workflow/target constants and tool discovery to DSL. Eliminate 3 extern
bridges (render_bootstrap_makefile, render_bootstrap_gitignore, discover_tools).
Detail: `docs/design/v4/extern-bridge-gap-analysis.md` § Phase 7.

| ID | Task | Size | Status | Deps |
|----|------|------|--------|------|
| FC-P7-a | `dsl/config/build_workflows.dag`: WorkflowSpec + MetaTarget types + data. | M | **Done** | — |
| FC-P7-b | Compiler artifact emitter: `dag_emit.rs` emits data-only `.dag` from CompileOutput. `emit_artifact_dag()` produces EntrypointInfo type + entrypoints + output_paths data. Round-trip tests verify re-parse. | L | **Done** | — |
| FC-P7-c1 | DSL Makefile types: already exist in `extdeps/make.dag` (MakeTarget, ToolTarget, Makefile, etc.). No new work needed. | M | **Done** | FC-P7-a |
| FC-P7-c2 | DSL Makefile assembly: import data, produce targets, wire to makegen output. | M | Pending | FC-P7-a, FC-P7-b, FC-P7-c1 |
| FC-P7-d | Delete 2 bootstrap extern impls (render_bootstrap_makefile, render_bootstrap_gitignore). Makefile: delegate to makegen DSL rendering. Gitignore: DSL categories + tool output data. Add parity golden tests. | M | Pending | FC-P7-c2 |

---

## Foundation: Compiler Features (FC-CF)

Language features needed for extern bridge elimination. Evaluated against concrete
business cases — features expressible via existing `fold` are deprioritized.

| ID | Feature | Size | Status | Deps | Unblocks | Notes |
|----|---------|------|--------|------|----------|-------|
| FC-CF1 | `split(delim)`: String → List\<String\>. 4 compiler stages + 5 e2e tests. | M | **Done** | — | FC-P8-a | Irreducible. Path parsing for tree rendering. |
| FC-CF7 | `zip()`: List\<A\> × List\<B\> → List\<Map{first, second}\>. 4 compiler stages + 4 e2e tests. | M | **Done** | — | FC-P8-b | Irreducible. Parallel list assembly in snapshot. |
| FC-CF5 | Recursive types (self-referential type defs) | L | Pending | — | FC-CF6 | DirEntry { children: List\<DirEntry\> }. |
| FC-CF6 | Recursive functions (self-calls in fn bodies) | L | Pending | FC-CF5 | FC-P8-a | Tree traversal (flatten, render). |
| FC-CF2 | `skip(n)`: List\<T\> → List\<T\> | S | Pending | — | FC-P8-a | Low priority — expressible via fold+index. |
| FC-CF3 | `enumerate()`: List\<T\> → List\<(Int, T)\> | M | Pending | — | FC-P8-a | Low priority — expressible via fold+counter. |

**Dropped**: FC-CF4 (`group_by`) — no current extern bridge needs it. `render_tree`
uses BTreeMap trie insertion (split + recursive insert), not group_by. Re-evaluate if
a concrete business case emerges.

---

## Foundation: Anemic Elimination (FC-P8)

Last 2 extern bridges → pure DSL. Then delete extern_impls.rs entirely.
Detail: `docs/design/v4/extern-bridge-gap-analysis.md` § Phase 5.

| ID | Task | Size | Status | Deps |
|----|------|------|--------|------|
| FC-P8-a | Tree rendering in pure DSL (DirEntry recursive type, build_dir_entries, flatten_entries, render_tree). Delete RenderTreeOp. | L | Pending | FC-CF1, FC-CF5, FC-CF6 |
| FC-P8-b | Snapshot content as MarkdownDoc. Delete BuildSnapshotContentOp. | M | Pending | FC-CF7, FC-P8-a |
| FC-P8-c | Delete extern_impls.rs, resolve_extern_call(), all_extern_symbols(), lookup_extern_impl(). Zero `extern func` in any .dag file. | S | Pending | FC-P8-a, FC-P8-b |

**Foundation endstate**: Zero extern bridges. `extern_impls.rs` (610 lines) +
policy const arrays in `pragma.rs` (~300 lines) deleted. All domain logic in DSL.
(~1,350 lines already deleted by FC-NF7 + FC-CL.)

---

## Testing: Black-Box Node Testing (BB)

Every node is a black box: feed accumulated mocks from all workflows, assert
output contracts. Type DAG provides free mock pairs via set algebra and
cardinality. Transport fidelity ladders generate tiered test variants.
Design: `docs/design/black-box-node-testing.md`.

| ID | Task | Size | Status | Deps |
|----|------|------|--------|------|
| BB-0 | **Compositional type modeling.** Define core types and prove composition algebra before any test generation. Types: `NodeIdentity` (module + callable, stable across workflows), `CorpusExample` (inputs + Expectation + Provenance), `Expectation` enum (ExactOutputs/OutputMatchers/TypeContractOnly/ExpectValidationError), `Provenance` (workflow, profile, node_instance, subdag_path, seed_kind), `MockCorpus` (flat `Vec<CorpusExample>` with union/dedup algebra), `EdgeExample` (for Level 2 window tests), `FidelityLadder` (per-TransportKind tier definitions), `FidelityLevel` (PureMock/VirtualIo/Sandboxed/RealLocal/RealRemote). Composition contracts: (1) Expectation assignment per SeedKind (WorkflowObserved+pure→ExactOutputs, TypeDerived→TypeContractOnly, etc.), (2) anchored mutation default for multi-port nodes (vary one port, hold others at base), (3) FidelityLadder × TransportKind (one canonical ladder per kind), (4) node max fidelity = transitive meet of transport dep fidelities, (5) normalization/redaction policy (canonical maps, path substitution, secret redaction, 64KB cap). Integration tests proving the algebra. | M | **Done** | — |
| BB-1 | **Mock corpus builder.** Accumulate `ObservedCase` across all DSL workflow baseline DryRuns. Piggyback on existing testgen baseline pass. Operate on `lower(&dag).dag` for SubDag visibility. Group by `NodeIdentity`, dedup by `(workflow, hash(inputs))`. Output: `HashMap<NodeIdentity, MockCorpus>`. | M | **Done** | BB-0 |
| BB-2 | **Per-node test generation (Level 1a/1b).** For each corpus case, execute node and assert output shape (1a) or exact match via OutputMatcher (1b). Start with pure nodes only (`ExecutionMode::Real`, no transport mocking). Effectful nodes use DryRun + shape-only assertions. | M | Pending | BB-1 |
| BB-3 | **Adjacent pair test generation (Level 2).** Capture `EdgeExample` per workflow edge during DryRun. For each pure→pure edge, generate 2-node window test via `Window::from_nodes` — execute through real executor wiring (not manual port-map feeding). This tests param→port translation, the exact plumbing where wiring bugs live. Extend to mixed edges after pure-only proven. | M | Pending | BB-2 |
| BB-4 | **Type-derived boundary values.** Wire `contract::witnesses()`, `cross_product_witnesses()`, `variant_witnesses()` into corpus builder. Default: anchored mutation (vary one port at a time from observed base cases). Pairwise cross-product opt-in per node. Merge/dedup with workflow-observed values. `max_test_cases_per_node = 50`. Most infrastructure already exists. | S | **Done** (enrichment logic implemented; pipeline integration TBD) | BB-1 |
| BB-5 | **Cross-workflow consistency tests (Level 4).** For nodes in 2+ workflows, assert structurally compatible outputs across all workflow-specific inputs. Gate on `is_pure && no_resource_deps && no_env_reads`. | S | Pending | BB-2 |
| BB-6 | **Transport fidelity ladders.** `FidelityLadder` type + canonical definitions per `TransportKind` (File: PureMock→VirtualFs→SandboxedFs→RealFs→RemoteFs; Shell: similar; Rest/Http/Tcp: similar). Node-level max fidelity inference (transitive meet of transport deps). Tiered test variant generation (same corpus inputs, different transport resolution per tier). DSL `fidelity { }` block syntax in resource definitions. Gate by existing `GUNBC_TEST_MAX_COST`. | L | **Done** (types + ladders + composition algebra; tiered variant gen in BB-2) | BB-0 |

BB-0 implementation: All types in `core/test/src/corpus.rs` and `core/test/src/fidelity.rs`. 23 integration tests proving dedup, merge, normalization, redaction, fidelity composition.
BB-1 implementation: `build_corpus()` in `core/codegen/src/testgen/mock_corpus.rs`. DryRun extraction + cross-workflow accumulation + DagAnalysis structural facts. E2e tests in `gunbc-dag/tests/corpus_builder.rs`.
BB-4 implementation: `enrich_corpus_with_type_witnesses()` with anchored mutation (vary one port, hold others at base). MAX_EXAMPLES_PER_NODE=50.
BB-6 implementation: Canonical ladders for all 6 TransportKind variants. `node_max_fidelity()` transitive meet inference. FidelityRung cost model.

**BB endstate**: Every node tested against every input context from every workflow.
Type-derived boundary values cover cardinality + refinement + coproduct variants.
Transport fidelity ladders generate hermetic (≤S) through live (XL) test tiers.
~4400 hermetic tests auto-generated, <13s total.

---

## SDLC: Pipeline Activation (SDLC-1:8)

Bring the pipeline from "compiles" to "runs e2e on local profile."
3,616 lines of SDLC DSL already exist (interfaces, providers, stages, worker).
Design: `docs/design/sdlc/mega-modeling-design.md`.

| ID | Task | Size | Status | Deps |
|----|------|------|--------|------|
| SDLC-1 | Register SDLC in workflow catalog + WorkspaceBinary dispatch. | M | Pending | — |
| SDLC-2 | Fill dispatch runtime: real stage transition logic via state machine. | M | Pending | SDLC-1 |
| SDLC-3 | Fill validation runtime: review_gate, ci_gate with real logic. | M | Pending | SDLC-2 |
| SDLC-4 | Complete testing→done handler (cargo test + clippy + conditional merge). | M | Pending | SDLC-1 |
| SDLC-5 | Local SignalStore provider (file-based, satisfies signal_store.dag contracts). | M | Pending | — |
| SDLC-6 | Local ArtifactStore provider (file-based, content-hash keyed, two-phase commit). | M | Pending | — |
| SDLC-7 | Profile binding verification: compile all 3 profiles, hermetic e2e on unit_test. | M | Pending | SDLC-1:6 |
| SDLC-8 | Local profile e2e: real GitHub repo, idea → design → review flow. | L | Pending | SDLC-7 |

**SDLC activation deliverable**: `gunbc sdlc --profile local --repo owner/name`

---

## SDLC: Cloud Deployment (SDLC-CD)

After local e2e works. Design: `docs/design/sdlc/mega-modeling-design.md` §2.1.

| ID | Task | Size | Status | Deps |
|----|------|------|--------|------|
| SDLC-CD1 | GCS SignalStore (PubSub-backed, at-least-once). | M | Pending | SDLC-8 |
| SDLC-CD2 | GCS ArtifactStore (GCS-backed, generation CAS). | M | Pending | SDLC-8 |
| SDLC-CD3 | GCP credential chaining (WIF OIDC exchange). | L | Pending | SDLC-8 |
| SDLC-CD4 | Cloud Run deployment DAG. | L | Pending | SDLC-CD1:3 |
| SDLC-CD5 | Multi-worker CAS stress test (3 workers, exactly-once). | M | Pending | SDLC-CD4 |
| SDLC-CD6 | CI integration (hermetic + cloud smoke). | M | Pending | SDLC-CD5 |

---

## Backlog

Not scheduled. Promote to active sections when capacity opens.

| ID | Item | Size | Priority | Notes |
|----|------|------|----------|-------|
| DG1 | Daggen: re-enable `needs_daggen()` for dynamic DAG generation from git diffs. | L | P1 | Feeds SDLC pipeline scaling |
| H10 | Compute stack orchestration: Cloud Run/GCS/LB lifecycle DAG builder. | L | P2 | `docs/design/horizon/h10-compute-stack-services.md` |
| S12-E | Multi-worker CAS: GcsClaimStore with generation-based CAS. DSL exists. | M | P2 | Deferred until cloud_run profile needed |
| H1 | Display reactive DSL: channel-driven event loop. | XL | P3 | No current use case. Review 2026-Q3, delete if not promoted. |

---

## Refactor Opportunities

Accumulated patterns awaiting consolidation. Grouped by theme — look for
larger patterns across groups before scheduling individual items.

### Theme A: Typed Dispatch (strings → enums, 1:1 mapping consolidation)

Every sub-item is a variation of "strings where there should be types" or
"the same enum→value mapping repeated in multiple locations." A1 is the
highest-value single item. A7–A10 are the 1:1 mapping long tail catalogued
from a codebase-wide audit (2026-02-26).

| ID | Pattern | Scope | Key Files | Notes |
|----|---------|-------|-----------|-------|
| RF-A1 | **NodeKind on Node\<T\>**. ~~40+ `type_id.0 == "TransportRequest"` heuristics~~. NodeKind field landed, lowerer stamps it, executor/validator read it. **Remaining hack**: `validate_node_kinds_for_interception()` is a runtime pre-flight check that rejects `kind: None` nodes with effectful port patterns in DryRun/Simulate. Target state: make this structurally impossible — `Node::opaque()` should require a `NodeKind` argument (or a typed builder that enforces it), eliminating the `Option` and the runtime check. | execute.rs, node.rs, resource/mod.rs | `Node::opaque()` defaults `kind: None`; `Node::subdag()` does too. Change signatures to require `NodeKind`, remove `Option<NodeKind>`, delete `validate_node_kinds_for_interception`. |
| RF-A2 | **Port namespace typing**. 18+ scattered `starts_with("res:")` / `"tool:"` / `"__out:"` / `"__"` checks. `is_user_param_port()` reimplemented 3× with slight variations. | 18+ sites, 6 files | builder.rs, validate.rs, signature.rs, plan.rs, obligation.rs, rust_exec_runtime.rs | Add methods to `PortName` (`is_resource()`, `is_tool()`, `is_internal()`, `is_user_facing()`). |
| RF-A3 | **Module path representation**. 4 crates use `Vec<String>` while daglang-syntax has typed `ModulePath`. 5+ manual `.join(".")` calls in pipeline.rs. | 4 crates | pipeline.rs, resolve.rs, typecheck.rs, syntax/lib.rs | Unify on `ModulePath` from daglang-syntax; add `From<Vec<String>>` + `From<&str>`. |
| RF-A4 | **Stringly-typed dispatch in resolve.rs**. 10+ string prefix matches for module/callable routing (`module.starts_with("services.")`, `name.starts_with("service_transport::")`). | 10+ sites, 1 file | gunbc-dag/src/resolve.rs | Create `CallableClass` enum parsed once, dispatched everywhere. |
| RF-A5 | **Transport node classification**. String-based prepare/execute/parse detection via `name.starts_with("service_transport::execute::")` etc. | 3+ checks per node | gunbc-dag/src/resolve.rs | Define `TransportNodeKind { Prepare, Execute, Parse }` enum. |
| RF-A6 | **String constants consolidation**. `"__deps"` (45×), `"__out:"` (6×), `"res:file"` (8×), `"tool:"` (15×) scattered as literals. | 74+ sites | scattered | Extract to central consts in `core/ir/src/signature.rs` or `core/ir/src/resource/mod.rs`. |
| RF-A7 | **Transport class heuristic shadows**. `transport_depth_ordinal()`, `transport_depth_str()`, `transport_is_hermetic()` in `fidelity.rs` are Rust replicas of DSL fns that already exist in `fidelity.dag`. | 3 fns, 3 files | fidelity.rs, dsl/std/fidelity.dag, dsl/config/test_policy.dag | Not "move to impl" — **delete entirely** once DSL `classify_transports()` is callable. See RF-G1–G3 for the blocker (fold extraction). |
| RF-A8 | **`as_str`/`parse` boilerplate**. 15+ enums implement identical bidirectional string conversion: `as_str() → &str` + `parse(&str) → Option<Self>` + `Display` delegate + `FromStr` delegate. 4 methods × 15 enums = ~60 match blocks. | 15 enums, ~12 files | platform.rs (Arch, Vendor, Os, AbiEnv, ExecutionEnv), cargo.rs (Subcommand, CodegenSubcommand, TermColor), http.rs (HttpMethod), llm/chat.rs (Role, ReasoningEffort, ReasoningSummary), cloud.rs (CloudProviderKind, CloudRuntimeKind), fermi.rs (TestClass, FermiCost) | Derive macro (`#[derive(StringEnum)]`) or trait with blanket `Display`/`FromStr` impls from `as_str()`/`parse()`. |
| RF-A9 | **Emit backend type-name tables**. DSL type names mapped to target language types in 3+ match blocks per backend. `"String" \| "Path" → "string"`, `"Bool" → "bool"`, etc. — same canonical normalization repeated in `lower_to_ir.rs`, `lower_go.rs`, `lower_rust.rs`. | 3 backends, 10+ types each | daglang-emit/src/lower_to_ir.rs, lower_go.rs, lower_rust.rs | Shared `DslTypeMapping` table indexed by backend, not per-backend match arms. |
| RF-A10 | **String dispatch in DAG tooling**. 100+ match arms on string literals for workflow names, Makefile targets, unit commands, resource types. Each maps a name to a builder/command/value. | 100+ arms, 5 files | workflow/unit_commands.rs, workflow/catalog.rs, makegen/shared.rs, resolve.rs (resource auto-construction), mock_interpreter.rs | Registry pattern or DSL data declarations. Most are active migration targets (makegen already moved to DSL). |

### Theme B: Service Transport Completeness

Mechanical migration — same pattern repeated across .dag files. Adding
`transport rest/shell/local { ... }` + `config { endpoint, auth }` blocks.

| ID | Scope | Ops Missing | Status | Notes |
|----|-------|-------------|--------|-------|
| RF-B1 | **Active services**: github/issues.dag (8), github/pull_request.dag (6), llm/openai.dag (2) | 16 | Blocks SDLC + design tools | REST transport; need `config { endpoint, auth }` |
| RF-B2 | **SDLC providers**: file stores (6), GCS stores (6), github_issue_provider (7), codex_agent (4), credential providers (4) | 27 | Blocks SDLC pipeline activation | Mixed rest/shell/file/local |
| RF-B3 | **Stub providers**: stub_providers.dag (26), stub_credential_provider.dag (2) | 28 | Intentional — unit_test profile stubs | No transport needed; consider `transport stub {}` marker |
| RF-B4 | **Infrastructure stubs**: azure (43), aws (38), gcp-infra (59) | 140 | Dormant — not in active pipeline | Defer until infrastructure provisioning lane opens |

### Theme C: Code Organization

Ongoing hygiene. Not urgent but reduces cognitive load over time.

| ID | Pattern | Scope | Notes |
|----|---------|-------|-------|
| RF-C1 | **Monolithic files**. daglang-lower/lib.rs (11K lines), execute.rs (4K), resolve.rs (2K), typecheck/lib.rs (5K). | 4 files | Lower and resolve are highest ROI splits. |
| RF-C2 | **Passthrough op sprawl**. 3+ identity variants in resolve.rs (`IdentityCallableOp`, `DeclaredOutputCallableOp`, `DeclaredOutputPassthroughOp`) with 47 lines of hardcoded port alias lists. | resolve.rs | Unify to single `PassthroughOp` with data-driven alias map. |
| RF-C3 | **Error type consolidation**. 6 error types across crates — `ResolveError`, `PipelineError`, `LowerError`, `TypeError`, `BuilderError`, `ExecError`. Most are string wrappers. | 6 files | `ExecError` is well-structured; others could adopt similar layering. |
| RF-C4 | **Test helper extraction**. ~29K lines of test code with duplicated fixture creation, mock setup, assertion patterns across daglang-cli test files. | 4 files | Extract `CompileTestHelper` + `MockFactory` trait. |
| RF-C5 | **Builder inconsistency**. Mix of `DagBuilder` fluent API, pattern builder functions, and raw struct construction (especially in tests/validate.rs). | core/ir | Low priority — working but inconsistent. |

### Theme D: M22 Scaffolding

Types defined for future features but never populated. Annotation-based
extraction path is now impossible (annotations deleted). Decision needed:
delete scaffolding or update extraction to use typed DSL syntax.

| ID | What | Status | Notes |
|----|------|--------|-------|
| RF-D1 | `RetryPolicy` / `BackoffStrategy` — always `None`. | Scaffolding | Will need DSL `retry` block syntax. |
| RF-D2 | `ErrorMapping` — always `vec![]`. | Scaffolding | Will need DSL `error_map` block syntax. |
| RF-D3 | `ContractObligation` — defined, not integrated into compiler pipeline. | Scaffolding | Contracts now use `contract` declarations; wire through lowerer. |
| RF-D4 | `ResourceRequirement` — defined, not integrated. | Scaffolding | `uses` declarations cover this; may be redundant. |

### Theme E: Pre-existing Test Gaps

| ID | Test | Root Cause | Notes |
|----|------|-----------|-------|
| RF-E1 | `registry_exposes_required_claims` | Process unit "ci.build_compile" doesn't have `file:target` Write claim. | Likely needs claim registration update in process_registry. |
| RF-E2 | `makegen_exec_runtime_e2e` (ignored) | Exec-runtime emit missing `LoadRegistry` handler + PureRender fn classification. | NF-5 handler specialization work. |
| RF-E3 | `pragma_exec_runtime_e2e` (ignored) | `ContentUpsertOutputPath` nodes unclassified for exec-runtime emit. | Depends on RF-E2 handler work. |
| RF-E4 | **Fidelity classification smoke test**. `comprehensive_auto_testgen_pipeline_validation` discards `target_def` — never asserts what `TestClass`/`FermiCost` each module got. No test catches a derive regression that silently downgrades all modules to Unit/XS (tests would be skipped at runtime). | 1 test gap | dag_test_discovery.rs | Add assertions: makegen→Hermetic/S (shell), gist→Integration/L (REST), bootstrap→Hermetic/S. |

### Theme F: Underused Abstractions

| ID | What | Implementors | Notes |
|----|------|-------------|-------|
| RF-F1 | Algebra traits (`PartialOrder`, `JoinSemilattice`, `MeetSemilattice`, `Lattice`, `BoundedLattice`, `Semiring`) | Only `Cardinality` | No generic usage. Keep or delete by 2026-Q3. |
| RF-F2 | Render traits (`OutputMedium`, `TextMedium`, `GraphicsMedium`, `CodeRenderer<M>`, `MarkupRenderer<M>`, etc.) — 8 traits. | 5 implementors | Under-amortized. Consider unifying. |

### Theme H: Structural Enforcement (parse, don't validate)

Runtime validation where types could make the invalid state
unrepresentable. Ordered by severity — findings 1–3 are in the
classification pipeline we just built (fidelity consolidation).

| ID | What | Current | Structural Fix | Scope |
|----|------|---------|---------------|-------|
| RF-H1 | **Fidelity silent fallbacks**. `classify_callable()` parses DSL output strings into enums with `unwrap_or(Unit)` / `unwrap_or(XS)`. If DSL returns a typo or `None`, classification silently downgrades to Unit — tests are skipped at runtime with no error. | `TestClass::parse(str).unwrap_or(Unit)` in fidelity.rs:129 | **Eliminated by RF-G unblock**: `classify_transports()` returns typed `DerivedClassification`, no string round-trip. Short-term: change `unwrap_or` to `.expect("DSL classify_from_facts returned invalid tier")`. | Cross-cutting (tied to G) |
| RF-H2 | **TestgenTargetDef Option fields always populated**. `test_class: Option<TestClass>`, `fermi_cost: Option<FermiCost>`, `requires: Option<Vec<String>>` — every auto-testgen call site now fills `Some(...)` from fidelity. The `Option` only exists for legacy `DagSpecDef` path (which also never overrides). `generate_target_with_types()` does `unwrap_or(Unit)` on every field. | 6 Option fields in registry.rs, unwrap_or chain in testgen-registry lib.rs:170-178 | Make fields non-Option with `Default` impl. Callers construct with values; no unwrapping. `DagSpecDef.to_def()` fills from fidelity instead of leaving `None`. | Local (registry.rs + testgen-registry) |
| RF-H3 | **TestClass/FermiCost parse returns Option not Result**. `parse(&str) -> Option<Self>` loses the invalid input. Callers chain `.unwrap_or(default)` which masks the error entirely. | fermi.rs:25-31, 56-64 | `FromStr` with `Result<Self, ParseError>`. Callers use `?` to propagate. Overlaps RF-A8 (derive macro). | Local (fermi.rs) |
| RF-H4 | **ResourceKind string dispatch**. `ResourceAcquireOp { resource_kind: String }` matched at runtime. Unknown kinds fall through to `Value::Str("resource:{other}")` — wrong type, silent. | resolve.rs:365-386 | `ResourceKind` enum parsed once at resolve time. Match is exhaustive, no fallback arm. Overlaps RF-A4. | Local (resolve.rs) |
| RF-H5 | **Port namespace string conventions**. ~30 `starts_with("res:")` / `"tool:"` / `"__deps"` checks across 8+ files. If prefix changes, every site must be updated or it silently stops matching. | resolve.rs, validate.rs, signature.rs, obligation.rs, etc. | Already RF-A2 (`PortCategory` enum + methods on `PortName`). Listed here for completeness. | Cross-cutting |

**Priority**: H1 is the most dangerous (silent test skipping). H4 is
the easiest win (local to resolve.rs). H2+H3 are mechanical cleanup
that improves the pipeline we just built. H5 is RF-A2 (already tracked).

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

So `test_policy.dag::classify_from_facts()` was written as a dumbed-down
version accepting pre-aggregated scalars (`max_depth: String,
hermetic: Bool`), and Rust replicates the aggregation to produce those
scalars. **Every 1:1 mapping in fidelity.rs is a Rust shadow of a DSL
function that already exists but can't be called.**

| ID | Rust Shadow | DSL Original | Blocker |
|----|-------------|-------------|---------|
| RF-G1 | `transport_depth_ordinal()` + `transport_depth_str()` — match `ServiceTransportClass` → u8/&str | `fidelity.dag::transport_depth()` → `FermiDepth` | `fermi_max_of` uses `fold` |
| RF-G2 | `transport_is_hermetic()` — match `ServiceTransportClass` → bool | `fidelity.dag::transport_hermetic()` → Bool | `classify_transports` uses `all` (fold-based) |
| RF-G3 | `classify_callable()` pre-aggregation — `max_by_key` + `all` + string packing | `fidelity.dag::classify_transports()` — does this natively | Same |
| RF-G4 | `test_policy.dag::classify_from_facts()` — accepts pre-aggregated scalars | Would be unnecessary if `classify_transports()` were callable | Same |
| RF-G5 | `TestClass::parse()` + `FermiCost::parse()` round-trip in `classify_callable()` | `classify_transports()` returns typed `DerivedClassification` | DSL evaluator returns `Value::Str` for sum-type variants |
| RF-G6 | `test_policy.dag::transport_depth()` + `transport_hermetic()` — string-typed duplicates | `fidelity.dag` already has the typed versions | Self-contained policy file can't import + evaluate cross-module fns |

**Unblock**: Fix the lowerer `fold` extraction limitation (fn bodies
containing `fold` decompose into `Collection` nodes, preventing
`fn_body` extraction for `evaluate_fn_body()`). Once fixed:
1. `classify_transports()` becomes callable from Rust
2. `classify_from_facts()` in test_policy.dag is deleted
3. All 4 Rust shadow fns in fidelity.rs are deleted
4. `requires_from_transport_classes()` moves to DSL
5. `classify_callable()` becomes: compile fidelity.dag, call
   `classify_transports(props.transport_classes)`, done

**Alternatively**: Implement `fold` aggregation in Rust's
`evaluate_fn_body()` evaluator (it already handles other collection
ops). This avoids changing the lowerer and directly enables calling
the existing DSL fns.

---

## Archive

NF-1 through NF-6 (compile+link hardening): complete 2026-02-25. Detail: `TODO/TODONE/tasks-completed.md`.
FC-NF7 (fn-level evaluation): complete 2026-02-25. `expr.rs` IR + `eval.rs` evaluator + `FnBodyDelegate`. `makegen/render.rs` deleted (~1200 lines). Makegen rendering is pure DSL.
FC-CL (dead code cleanup): complete 2026-02-25. Deleted `core/tool-registry` + `core/tool-registry-macros`, 14 orphaned spec builder fns, stale rules/comments.
FC-P6-b/c/d (pragma policy migration): complete 2026-02-26. `dsl_render.rs` evaluates `derive_*` DSL fns via `evaluate_fn_body()`. Allowlist + lint_policy migrated; clippy_toml blocked on FC-CF5. `all_extern_symbols()` 8→6.
FC-CF1 + FC-CF7 (split + zip): complete 2026-02-26. Both pipe methods across 4 compiler stages (typecheck, lower, eval, emit). 9 e2e tests.
FC-P7-b (artifact emitter): complete 2026-02-26. `dag_emit.rs` emits valid `.dag` syntax. `CompileOutput::emit_artifact_dag()` for downstream introspection.
FC-P7-c1 (Makefile DSL types): complete 2026-02-26. Types already existed in `extdeps/make.dag`.
