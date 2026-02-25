# Foundation Cleanup Roadmap

Status: Active
Date: 2026-02-25
Feeds: [SDLC implementation-roadmap.md](../sdlc/implementation-roadmap.md)
Related: [extern-bridge-gap-analysis.md](extern-bridge-gap-analysis.md) (Phase 5-8 detail)
Related: [externcall-same-module-port-wiring.md](externcall-same-module-port-wiring.md) (NF-7 design)

## 1. Goal

Delete dead Rust code. Eliminate all 9 extern bridges. Add compiler features for
structural modeling. End state: zero extern func declarations, all domain logic
in DSL.

## 2. Dependency Graph

```
FC-CL (dead code cleanup) ──────────── no deps, can start now
         │
FC-NF7 (lowerer wiring) ────────────── no deps, can start now
         │
    ┌────┴────┐
    │         │
FC-P6       FC-P7                      ◄── can run in parallel
(policy)    (registry)
    │         │
    └────┬────┘
         │
FC-CF (compiler features) ──────────── can start parallel with P6/P7
         │
FC-P8 (anemic elimination) ─────────── requires P6 + P7 + CF
         │
    zero extern bridges
```

**Key**: FC-P6 and FC-P7 need FC-NF7 only for shadow→extern conversion.
They can start data migration work immediately; the extern deletion step
within each waits for NF7.

## 3. FC-CL: Dead Code Cleanup

Delete crates and code with zero dependents. Safe to do immediately.

| ID | Task | Size | What's deleted |
|----|------|------|----------------|
| FC-CL1 | Delete `core/tool-registry` + `core/tool-registry-macros`. Remove from workspace Cargo.toml. | S | 2 dead crates |
| FC-CL2 | Delete orphaned SDLC Rust: spec_builder sdlc fns, dangling pipeline tests in resolve.rs. | S | Orphaned stubs |
| FC-CL3 | Remove stale `languages.rs` dead_code rule from `policy/pragma.rs` (file doesn't exist). | S | Stale policy entry |

## 4. FC-NF7: Lowerer Extern Func Wiring

Fix same-module extern func call wiring in the lowerer. Design complete:
[externcall-same-module-port-wiring.md](externcall-same-module-port-wiring.md).

| ID | Task | Size | Design ref |
|----|------|------|------------|
| FC-NF7 | `TypedItemSignature::ExternFunc` variant + `lower_extern_call()` + endpoint registration in both `endpoints_by_full` and `endpoints_by_name`. | L | D1-D4 in design doc |

Unblocks: shadow fn → extern func conversion in P6 and P7.

## 5. FC-P6: Policy Migration (Extern Bridge Phase 6)

Move policy data from Rust const arrays to DSL data declarations. Eliminate
3 extern bridges (render_clippy_toml, render_disallowed_methods_allowlist,
render_pragma_lint_policy). **No new compiler features needed.**

Full detail: [extern-bridge-gap-analysis.md](extern-bridge-gap-analysis.md) § Phase 6.

| ID | Task | Size | Deps |
|----|------|------|------|
| FC-P6-0 | Validate flat_map: DSL test exercising `CollectionOpKind::FlatMap` e2e. | S | — |
| FC-P6-a | `dsl/config/workspace.dag`: CrateSpec type + workspace_crates data + CI drift test vs Cargo.toml. | M | — |
| FC-P6-b | `dsl/config/pragma_policy.dag`: AllowlistRule, DeadCodeRule, AllowLint types + data from pragma.rs const arrays. | M | FC-P6-a |
| FC-P6-c | DSL policy rendering: resolve_crate_path() + render fns using Document types. | M | FC-P6-a, FC-P6-b, FC-P6-0 |
| FC-P6-d | Delete 3 pragma extern impls + shadow fn bodies from pragma.dag. Delete pragma.rs rendering fns. | S | FC-P6-c, FC-NF7 |

## 6. FC-P7: Registry Migration (Extern Bridge Phase 7)

Move workflow/target constants and tool discovery to DSL. Eliminate 5 extern
bridges (load_registry, render_makefile, makegen, render_bootstrap_makefile,
render_bootstrap_gitignore).

Full detail: [extern-bridge-gap-analysis.md](extern-bridge-gap-analysis.md) § Phase 7.

| ID | Task | Size | Deps |
|----|------|------|------|
| FC-P7-a | `dsl/config/build_workflows.dag`: WorkflowSpec + MetaTarget types + data (20 workflows, 8 meta targets). | M | — |
| FC-P7-b | Compiler artifact emitter: emit `generated/tool_registry.dag` from CompileOutput.inferred_entrypoints. Committed as seed. | L | — |
| FC-P7-c1 | DSL Makefile types + rendering: MakefileTarget, GitignoreCategory, render fns. | M | FC-P7-a |
| FC-P7-c2 | DSL Makefile assembly: import data, produce targets, wire to makegen output. | M | FC-P7-a, FC-P7-b, FC-P7-c1 |
| FC-P7-d | Delete 5 makegen/bootstrap extern impls. Delete extern func declarations. | S | FC-P7-c2, FC-NF7 |

## 7. FC-CF: Compiler Features

Add 7 language features needed for tree/graph structural modeling. Each feature
is independently testable with a small `.dag` file.

| ID | Feature | Size | Deps | Unblocks |
|----|---------|------|------|----------|
| FC-CF1 | `split(delim)`: String → List\<String\> | M | — | FC-P8-a |
| FC-CF2 | `skip(n)`: List\<T\> → List\<T\> (drop first N) | S | — | FC-P8-a |
| FC-CF3 | `enumerate()`: List\<T\> → List\<(Int, T)\> | M | — | FC-P8-a |
| FC-CF4 | `group_by(key_fn)`: List\<T\> → Map\<K, List\<T\>\> | L | — | FC-P8-a |
| FC-CF5 | Recursive types (self-referential type defs) | L | — | FC-CF6 |
| FC-CF6 | Recursive functions (self-calls in fn bodies) | L | FC-CF5 | FC-P8-a |
| FC-CF7 | `zip()`: List\<A\> × List\<B\> → List\<(A, B)\> | M | — | FC-P8-b |

Note: conditional list assembly (if/else in list context) may be expressible
via flat_map. Validate before adding as separate feature.

## 8. FC-P8: Anemic Elimination (Extern Bridge Phase 5 + 8)

Convert last 2 extern bridges to pure DSL. Delete extern_impls.rs entirely.

Full detail: [extern-bridge-gap-analysis.md](extern-bridge-gap-analysis.md) § Phase 5.

| ID | Task | Size | Deps |
|----|------|------|------|
| FC-P8-a | Tree rendering in pure DSL: DirEntry recursive type, build_dir_entries, flatten_entries, render_tree. Delete RenderTreeOp. | L | FC-CF1:6 |
| FC-P8-b | Snapshot content as MarkdownDoc. Delete BuildSnapshotContentOp. | M | FC-CF7, FC-P8-a |
| FC-P8-c | Delete extern_impls.rs, resolve_extern_call(), all_extern_symbols(), lookup_extern_impl(). Verify: zero `extern func` in any .dag file. | S | FC-P8-a, FC-P8-b |

## 9. Deletion Summary

| Phase | Deleted | ~Lines |
|-------|---------|--------|
| FC-CL | 2 dead crates, orphaned stubs, stale rule | 200 |
| FC-P6 | 3 extern impls + pragma.rs const arrays | 400 |
| FC-P7 | 5 extern impls + registry constants | 600 |
| FC-P8 | extern_impls.rs + all extern func decls | 650 |
| **Total** | | **~1,850** |

## 10. Endstate

Zero extern bridges. `extern_impls.rs` deleted. `resolve_extern_call()` deleted.
No `extern func` declarations in any `.dag` file. All domain logic — policy rules,
workspace model, tool registry, tree rendering, snapshot assembly — lives in DSL
data declarations and pure functions.

The Rust substrate compiles and executes. It has no knowledge of markdown,
makefiles, pragma policies, tree rendering, crate tiers, or workflow dependencies.
