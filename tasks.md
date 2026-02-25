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
Wk 1-2  │ FC-CL (cleanup) │               │ SDLC-1 (catalog) │
        │ FC-NF7 (lowerer)│───────────────▶│ SDLC-5 (signal)  │
        │        │        │               │ SDLC-6 (artifact)│
Wk 2-4  │ FC-P6 (policy)  │               │        │         │
        │ FC-P7 (registry)│               │ SDLC-2 (dispatch)│
        │        │        │               │ SDLC-3 (validate)│
Wk 4-7  │ FC-CF (compiler)│               │ SDLC-4 (testing) │
        │        │        │               │        │         │
Wk 7-9  │ FC-P8 (anemic)  │               │ SDLC-7 (verify)  │
        │        │        │               │ SDLC-8 (local e2e)│
        └─────────────────┘               │        │         │
                                          │ SDLC-CD (cloud)  │
                                          └──────────────────┘
```

SDLC-1:6 can start immediately (no foundation dependency).
FC-EG (enforcement gates) can start immediately — no deps, prevents regression.
FC-NF7 is in progress (fn-level evaluation plan, Phases 0-5).
FC-P6 and FC-P7 run in parallel after FC-NF7-3 (eval logic).
FC-CF runs in parallel with P6/P7.
FC-P8 requires FC-P6 + FC-P7 + FC-CF (split, zip, recursion at minimum).

---

## Foundation: Dead Code Cleanup (FC-CL)

Delete crates and code with zero dependents. Start now.

| ID | Task | Size | Status |
|----|------|------|--------|
| FC-CL1 | Delete `core/tool-registry` + `core/tool-registry-macros`. Remove from workspace Cargo.toml. | S | Done |
| FC-CL2 | Delete orphaned SDLC Rust: `sdlc_workflow_spec()` + `build_all_workflow_spec()` + all 12 `*_with_registry()` variants from `spec_builders.rs` (zero callers). Remove re-exports from `workflow/mod.rs` and `lib.rs`. | S | Done |
| FC-CL3 | Remove stale `languages.rs` dead_code rule from `policy/pragma.rs` (file doesn't exist). | S | Done |
| FC-CL4 | Remove stale `DeferredCallableOp` references from `mock_defaults.rs` comments (lines 449, 478). DeferredCallableOp deleted in P6. | S | Done |

---

## Foundation: Enforcement Gates (FC-EG)

Automated ratchets that prevent modeling regression. Cheap to add, high leverage.

| ID | Task | Size | Status | Deps |
|----|------|------|--------|------|
| FC-EG1 | Import-direction lint: compiler check that `tools→config→extdeps→std` import direction is never violated. Error on backward imports. | M | Pending | — |
| FC-EG2 | Extern func count gate: CI test that counts `extern func` declarations in `.dag` files. Assert count ≤ current (ratchet — count only goes down). | S | Pending | — |
| FC-EG3 | `format!/push_str` boundary gate: grep + allowlist test. No new `format!()` or `push_str()` in non-boundary Rust code (allowlist for transport, codegen, existing scaffolding). | S | Pending | — |

---

## Foundation: Fn-Level Evaluation (FC-NF7)

Enable DSL fn bodies to execute at runtime. Currently fn items resolve to
PassthroughOp (identity forwarding) — string interpolation, let bindings, pipe
chains are compiled but never evaluated. Two gaps: (1) NF-7 same-module fn call
output wiring, (2) fn body computation discarded by lowerer.

**Status**: In progress. Phase 0 scaffolding done (untracked). Phases 1-5 pending.

**Implementation note**: Phase 2 (FnComputation DAG nodes) may be unnecessary —
`fn_body` is already carried on Callable nodes from Phase 0. Phase 3's
`FnBodyDelegate` evaluates the whole body as a unit, making per-statement DAG
nodes redundant. Consider skipping Phase 2 and going Phase 0 → 1 → 3 → 4 → 5.

| ID | Task | Size | Status | Deps |
|----|------|------|--------|------|
| FC-NF7-0 | Parity snapshot + fn body IR: `expr.rs` (LoweredFnBody/LoweredExpr IR + `lower_fn_body()` translation), `fn_body` field on Callable, golden `makefile_parity.rs` test. | S | Done (untracked) | — |
| FC-NF7-1 | Fix NF-7: `wire_fn_call_outputs()` — wire same-module fn call return values back to caller. | M | Pending | — |
| FC-NF7-1b | Shadow→extern quick win: convert 3 active shadow bridges (`gist::build_snapshot_content`, `bootstrap::render_bootstrap_makefile`, `bootstrap::render_bootstrap_gitignore`) from `fn` to `extern func`. Delete placeholder DSL bodies. Moves from fail-open to fail-closed resolution. No fn-body eval needed. | S | Pending | FC-NF7-1 |
| FC-NF7-2 | Lower fn body computation: `FnComputation` LoweredOp variant + new pass `add_fn_body_computation_edges()`. **May skip** — see note above. | L | Pending | FC-NF7-1 |
| FC-NF7-3 | Pure eval functions in `daglang-lower/src/eval.rs` + thin `FnBodyDelegate`/`CollectionDelegate` in resolve.rs. Expressions needed: string interp, let bindings, pipe (join/map/filter/first/append), field access, if/else, fn calls to siblings, string concat, list literals, ==/!=. | L | Pending | FC-NF7-1 |
| FC-NF7-4 | Prove makegen parity: delete `RenderMakefileContentOp` from extern_impls.rs, verify golden test. | M | Pending | FC-NF7-3 |
| FC-NF7-5 | Delete `makegen/render.rs` (1137 lines), extract justfile helpers to `makegen/shared.rs`. | M | Pending | FC-NF7-4 |

Unblocks shadow fn → pure DSL conversion in P6 and P7.

---

## Foundation: Policy Migration (FC-P6)

Move policy data from Rust const arrays to DSL. Eliminate 3 extern bridges
(render_clippy_toml, render_disallowed_methods_allowlist, render_pragma_lint_policy).
No new compiler features needed. Detail: `docs/design/v4/extern-bridge-gap-analysis.md` § Phase 6.

**Scaffolding exists**: `dsl/config/clippy_disallowed.dag` (38 disallowed methods in 4
groups + 8 disallowed types), `dsl/config/clippy_policy.dag` (rendering helpers +
derive_clippy_toml/derive_disallowed_methods_allowlist/derive_pragma_lint_policy fns),
`dsl/config/arch_rules.dag` (AllowlistPattern type + 19 allowlist_patterns entries).
These are compiled but not executed until FC-NF7-3 lands.

| ID | Task | Size | Status | Deps |
|----|------|------|--------|------|
| FC-P6-0 | Validate flat_map: DSL test exercising `CollectionOpKind::FlatMap` e2e. | S | Pending | — |
| FC-P6-a | `dsl/config/workspace.dag`: CrateSpec type + workspace_crates data + CI drift test. | M | Pending | — |
| FC-P6-b | `dsl/config/pragma_policy.dag`: AllowlistRule, DeadCodeRule types + data from pragma.rs. Partial: clippy_disallowed.dag + arch_rules.dag AllowlistPattern data already exist. | M | Pending | FC-P6-a |
| FC-P6-c | DSL policy rendering fns using Document types. Partial: clippy_policy.dag rendering helpers already exist. | M | Pending | FC-P6-a, FC-P6-b, FC-P6-0 |
| FC-P6-d | Delete 3 pragma extern impls. Wire pragma.dag to call derive_* fns from clippy_policy.dag. Add parity golden tests for clippy.toml, allowlist, lint policy. | S | Pending | FC-P6-c, FC-NF7-3 |

---

## Foundation: Registry Migration (FC-P7)

Move workflow/target constants and tool discovery to DSL. Eliminate 5 extern
bridges. Detail: `docs/design/v4/extern-bridge-gap-analysis.md` § Phase 7.

| ID | Task | Size | Status | Deps |
|----|------|------|--------|------|
| FC-P7-a | `dsl/config/build_workflows.dag`: WorkflowSpec + MetaTarget types + data. | M | Pending | — |
| FC-P7-b | Compiler artifact emitter: emit `generated/tool_registry.dag` from CompileOutput. | L | Pending | — |
| FC-P7-c1 | DSL Makefile types + rendering: MakefileTarget, GitignoreCategory, render fns. | M | Pending | FC-P7-a |
| FC-P7-c2 | DSL Makefile assembly: import data, produce targets, wire to makegen output. | M | Pending | FC-P7-a, FC-P7-b, FC-P7-c1 |
| FC-P7-d | Delete 2 bootstrap extern impls (render_bootstrap_makefile, render_bootstrap_gitignore). Makefile: delegate to makegen DSL rendering. Gitignore: DSL categories + tool output data. Add parity golden tests. Note: makegen render_makefile_content already deleted by FC-NF7-4. | M | Pending | FC-P7-c2, FC-NF7-3 |

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

**Foundation endstate**: ~1,850 lines of Rust deleted. Zero extern bridges.
All domain logic in DSL.

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
