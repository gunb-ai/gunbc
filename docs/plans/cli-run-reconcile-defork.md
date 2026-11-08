# Resolve/reconcile orchestration de-fork

**Goal:** move hand-authored `cli_run.rs` orchestration onto existing `.dag` authorities. **Method:** shared closure computation, one cache interface with distinct localities, and measured batch de-duplication. **Validation:** identical admitted closure, no recursive bootstrap dependency/extra full-corpus resolve, correct cache hits/misses and no reconciliation cost regression. This is scoped design, not implementation evidence; cutover requires operator sign-off.

## Scope

Own `build_module_index`/module/import extraction; `resolve_transitively`; `resolve_entry_graph` and indexed/discovery variants; `build_multi_entry_index`/`load_sources_for_entry_with_index`; `resolve_entry_with_parse_cache`/`resolved_graph_from_sources`; `reconcile_with_typed_cache`; `whole_tree_resolved_ctx`/`discover_owned_data_decls` and `DiscoveryResolveGroup` plumbing. Exclude resolver/infer internals and generated twins (#6155 resolver-B), `run_walk` and unrelated CI memo work; coordinate ordering before touching shared surfaces.

`concept_index` DeclFact/ConceptDecl has no import edges. The relevant existing authority is `v2.lens.module_graph` over ImportResolutionFact/ModuleDeclarationFact from their live host producers. Repoint the consumer; do not invent `compile_module_graph`, another BFS or substitute `std.graph.CallGraph`. Its bounded closure-algebra overlap is a separate question. Host regex extraction is a deeper separate lever, not eliminated by a consumer repoint or blocked on #6158’s unrelated fact projection.

**Corrected 2026-09-18:** typed-module cache already uses `std.interface_summary.typed_module_key` (source content, direct-import interface hashes, compiler identity), with per-content-key guards in `shared_typecheck_store` and the interface-summary witness. Name→content re-keying is done; earlier proposed work/false-hit claims are withdrawn. `cache_interface` already has `resolved_graph` filesystem/content-addressed realization. The historical parse_table_memo in-process exemplar was deleted by operator R2 on 2026-10-08; do not restore it.

## Phases

1. Repoint closure orchestration to existing modeled closure authority. Execute real-corpus set equivalence against the old route with a divergent-closure red control; notify the #6155 measurement owner before landing. Missing module provenance must produce a typed error, never skip. Trace the call graph to prove the query does not re-enter `resolve_entry_graph`; report cost and reject a second full-corpus resolve. Current namespace/reference-closure contracts govern the subject, not stale import-only assumptions.
2. Model in-process parse/typed-cache realizations under the existing interface, preserving their difference from PerHostFilesystem storage; do not flatten them. Reconcile the disk implementation with its declared content key and atomic temp/rename publication. Existing typed-content keys remain. This critical-path cut needs **separate explicit pre-merge sign-off**, same-workload wall time no worse than baseline, and red controls for both false misses and false hits.
3. Reuse affected-set/shared-dependency grouping for discovery batching if it displaces measured cost; do not create a third scheduler. This was an advisory/latent path, so drop an unprofitable de-fork. The historical `ci_floor_plan` target has been deleted; identify its surviving consumer before implementation.

Per phase, receipt actual hand-written orchestration displacement and applicable generation/drift invariants. Historical `regen_stage0 --verify` cannot be required after its mechanism’s retirement; current cut authority owns the replacement proof. A Rust template is not substrate migration. Completion requires the orchestration itself to be generated or removed.

## Workspace-root interim receipt

#6474 repaired compile-time `CARGO_MANIFEST_DIR` assumptions across packed-build/execution hosts. #6484’s shared `workspace_root_from(start_cwd)` follows `.git` ancestors and is the production rule exercised by explicit-path tests; no parallel Cargo.toml/dag heuristic.

Authority: `cli_run_workspace_root_scaffold.cli_run_workspace_root_discovery_scaffold` and `cli_run_workspace_root_discovery_dissolve_trigger`. Controls: `workspace_root_discovery_tests::{from_subdirectory_matches_git_toplevel, refuses_path_outside_git_checkout}` and `test.claim.cli_run_workspace_root_hand_rust_witness`. It is retained debt, not census shrink; remove workspace_root_from/OnceLock when Chunk F/`5-dissolve-patches` removes hand path discovery.

Retire this document when all admitted phases land and the resolve/reconcile/discovery orchestration is `.dag`-generated rather than hand maintained.
