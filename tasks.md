# Tasks

**Verification**: `cargo test --workspace` + `cargo clippy --all-targets -- -D warnings`
**Sizing**: S (<1 day), M (1-3 days), L (3-5 days), XL (5+ days)

---

## Roadmap: SDLC in Pure DSL (~10 weeks)

Two parallel tracks: foundation cleanup (delete Rust, add compiler features) and
SDLC activation (pipeline runs e2e). Model first, delete along the way.

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
```

SDLC-1:6 can start immediately (no foundation dependency).
FC-EG (enforcement gates) can start immediately — no deps, prevents regression.
FC-NF7 DONE — fn-level evaluation landed, render.rs deleted (~1200 lines).
FC-CL DONE — dead code cleanup (tool-registry crates, orphaned spec builders, stale rules).
FC-P6 and FC-P7 are UNBLOCKED — fn eval works, can convert remaining extern bridges.
FC-WM (workflow minimality) can start immediately — no foundation dependency.
FC-CF runs in parallel with P6/P7.
FC-P8 requires FC-P6 + FC-P7 + FC-CF (split, zip, recursion at minimum).

---

## Foundation: Enforcement Gates (FC-EG)

Automated ratchets that prevent modeling regression. Cheap to add, high leverage.

| ID | Task | Size | Status | Deps |
|----|------|------|--------|------|
| FC-EG1 | Import-direction lint: compiler check that `tools→config→extdeps→std` import direction is never violated. Error on backward imports. | M | Pending | — |
| FC-EG2 | Extern func count gate: CI test that counts `extern func` declarations in `.dag` files. Assert count ≤ current (ratchet — count only goes down). | S | Pending | — |
| FC-EG3 | `format!/push_str` boundary gate: grep + allowlist test. No new `format!()` or `push_str()` in non-boundary Rust code (allowlist for transport, codegen, existing scaffolding). | S | Pending | — |

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
| FC-P6-0 | Validate flat_map: DSL test exercising `CollectionOpKind::FlatMap` e2e. | S | Pending | — |
| FC-P6-a | `dsl/config/workspace.dag`: CrateSpec type + workspace_crates data + CI drift test. | M | Pending | — |
| FC-P6-b | `dsl/config/pragma_policy.dag`: AllowlistRule, DeadCodeRule types + data from pragma.rs. Partial: clippy_disallowed.dag + arch_rules.dag AllowlistPattern data already exist. | M | Pending | FC-P6-a |
| FC-P6-c | DSL policy rendering fns using Document types. Partial: clippy_policy.dag rendering helpers already exist. | M | Pending | FC-P6-a, FC-P6-b, FC-P6-0 |
| FC-P6-d | Delete 3 pragma extern impls. Wire pragma.dag to call derive_* fns from clippy_policy.dag. Add parity golden tests for clippy.toml, allowlist, lint policy. | S | Pending | FC-P6-c |

---

## Foundation: Registry Migration (FC-P7)

Move workflow/target constants and tool discovery to DSL. Eliminate 3 extern
bridges (render_bootstrap_makefile, render_bootstrap_gitignore, discover_tools).
Detail: `docs/design/v4/extern-bridge-gap-analysis.md` § Phase 7.

| ID | Task | Size | Status | Deps |
|----|------|------|--------|------|
| FC-P7-a | `dsl/config/build_workflows.dag`: WorkflowSpec + MetaTarget types + data. | M | Pending | — |
| FC-P7-b | Compiler artifact emitter: emit `generated/tool_registry.dag` from CompileOutput. | L | Pending | — |
| FC-P7-c1 | DSL Makefile types + rendering: MakefileTarget, GitignoreCategory, render fns. | M | Pending | FC-P7-a |
| FC-P7-c2 | DSL Makefile assembly: import data, produce targets, wire to makegen output. | M | Pending | FC-P7-a, FC-P7-b, FC-P7-c1 |
| FC-P7-d | Delete 2 bootstrap extern impls (render_bootstrap_makefile, render_bootstrap_gitignore). Makefile: delegate to makegen DSL rendering. Gitignore: DSL categories + tool output data. Add parity golden tests. | M | Pending | FC-P7-c2 |

---

## Foundation: Compiler Features (FC-CF)

Language features needed for extern bridge elimination. Evaluated against concrete
business cases — features expressible via existing `fold` are deprioritized.

| ID | Feature | Size | Status | Deps | Unblocks | Notes |
|----|---------|------|--------|------|----------|-------|
| FC-CF1 | `split(delim)`: String → List\<String\> | M | Pending | — | FC-P8-a | Irreducible. Path parsing for tree rendering. |
| FC-CF7 | `zip()`: List\<A\> × List\<B\> → List\<(A, B)\> | M | Pending | — | FC-P8-b | Irreducible. Parallel list assembly in snapshot. |
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
