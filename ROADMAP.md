# gunbc — Roadmap

One-line task tracker for gunbc (the compiler/language). `DESIGN.md` is the authority for *why*; this is
*what's in flight*. Product/ctrl tracked elsewhere. A task's real state is its branch/PR, not this file.

## Now

- Self-host fixed point — v2 compiles its own `src/v2` to a bit-identical fixed point
  - Lane 2a match / record / coproduct end-to-end — landed (#5057)
  - Lane 2b generic instantiation — landed (#5064)
  - Lane 3a `SourceRootIngest` — in flight (#5126)
  - Lane 3a.2 resolver / `QualifiedName` materialization — in flight (merry-crab)
- Route C — module-qualify names so closures span `dsl/` + `src/v2/`, then de-fork stds — held behind 3a.2 + census (bold-bee)
  - first witness: FreeMonoid fork (RED on cross-tree bridge today, GREEN after de-fork)
- Fact-cardinality lens — one lens for the 4 problem-classes + ratchet; emits the fork census — #5124
- CI wall-time (~27 min → minutes)
  - **Plan: [Realization & Measurement loop + infra onto `.dag`](docs/plans/realization-measurement-loop.md)** — the deep reframe: profile P1–P3 are symptoms; the spine is `measure → CostAccount.Measured → Pareto width/cache, bounded by a deployment HardwareBudget`. Keystone = wire the eval tap to a content-hash key + `CostAccount` (corpus is one serial node; cost is `predicted_zero`; cache catalog has no consumer). Subsumes "Caching via Realization" + "CI floor as one `.dag` binary" below. Worker-dispatch tracker.
  - P1 — cargo cache broken + double build (49%) — **needs an owner**
  - P2 — `dsl_compile_clean` sound memo (23%) — declare `content_hash(tree)` input, then RecordedFixture
  - P3 — resolver resolve blowup (15%) — self-host blocker; merry-crab pulling fix forward
    - comprep closure ~250s cold, content-specific super-linear (machine-independent)
    - root-cause: H1 missing resolve memo (maybe bypassed `resolved_graph_cache`) vs H2 generic-instantiation
- Lens universalization + host-language ban — in flight (quiet-swift-814)
  - host-language ban: enforcement via `shell.Exec.Run.script` String→ShellProgram typecheck (#5147 landed; type-flip capstone warm-badger); #5132 substring lens retired TERMINAL
  - GO B interim guard: `v2.lens.host_language_transport_script` + hand-Rust parse bridge `src/v1/stage0/src/transport_script_position_project.rs` (receipt: same `*_count_for_path` builtin lane as `extdeps_shape_transport_policy_project` — registered in `04_method.dag`, corpus `host_language_transport_script/corpus/migrated_transports_clean_test.dag`)
  - 🟡 carve-out: `toolchain_provision_shell_exec` routes emit_host go/ts literal blobs through a computed `script:` param so the transport site reads structural — not a corpus green for those rows; dissolve-on ToolchainProvision in std
  - retire 2 `.sh` transports — DONE: `git ls-files '*.sh'` is empty. source_root_ingest lifted to `cargo.Build.Run` + gunbc claim-runs (policy literals → workflow-tier params); layering enumeration collapsed onto the single-authority projection `v2.lens.layering_imports.layer_import_facts_live` (v1 handler `layering_imports_project.rs`, reusing `extract_import_paths` + `collect_dag_files_tolerant`). Perturb oracle: 4 committed fixtures under `src/v2/test/fixture/layering_scan`.
  - tier 0: `std.lens_verdict` (`Holds | Violation | NotApplicable | Unrealized` fail-closed) — landed; ScheduleLensVerdict migrated; tier 1: `table_decision_tree` + `identical_variant_payload` bespoke verdicts migrated (#5194); interim `reason: String` dissolve-on → `v2.std.diagnostic.Diagnostic` when cross-tree import lands
  - tier 1: cost / complexity / synthesis → always-required, structural budget (enrolls resolve → catches P3 class)
  - tier 2: `InferredTree`+deps lenses → adapters, then enroll

## Next

- Caching via Realization — consolidate the forked caches onto one kernel
  - caches forked: ParseTable / RecordedFixture / `resolved_graph_cache` share no kernel; `cache_interface` dead
  - kernel-to-be: `resolved_graph_cache`; RecordedFixture + (re-keyed) ParseTable become handlers
- §3 extdeps/std de-conflation — shape / transport / policy split (verified backlog)
- CI floor as one `.dag` binary — generate `ci.yml` from a `.dag` model; glob-discovery corpus cutover
- Axioms (A1–A3) in `.dag` + a syllogism lens — DESIGN open thread

## Recently landed

- #5111 cost-model loop zero-absorption fix
- #5098 grammar-inverse emit generalized to a structural reverse-parse fold
- #5089 dsl whole-tree compile-clean CI gate
- #5090 RecordedFixture record/replay seam
- #5101 CI floor consolidated into one composed `.dag` run
