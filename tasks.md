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

## Archive

NF-1 through NF-6 (compile+link hardening): complete 2026-02-25. Detail: `TODO/TODONE/tasks-completed.md`.
FC-NF7 (fn-level evaluation): complete 2026-02-25. `expr.rs` IR + `eval.rs` evaluator + `FnBodyDelegate`. `makegen/render.rs` deleted (~1200 lines). Makegen rendering is pure DSL.
FC-CL (dead code cleanup): complete 2026-02-25. Deleted `core/tool-registry` + `core/tool-registry-macros`, 14 orphaned spec builder fns, stale rules/comments.
FC-P6-b/c/d (pragma policy migration): complete 2026-02-26. `dsl_render.rs` evaluates `derive_*` DSL fns via `evaluate_fn_body()`. Allowlist + lint_policy migrated; clippy_toml blocked on FC-CF5. `all_extern_symbols()` 8→6.
FC-CF1 + FC-CF7 (split + zip): complete 2026-02-26. Both pipe methods across 4 compiler stages (typecheck, lower, eval, emit). 9 e2e tests.
FC-P7-b (artifact emitter): complete 2026-02-26. `dag_emit.rs` emits valid `.dag` syntax. `CompileOutput::emit_artifact_dag()` for downstream introspection.
FC-P7-c1 (Makefile DSL types): complete 2026-02-26. Types already existed in `extdeps/make.dag`.
