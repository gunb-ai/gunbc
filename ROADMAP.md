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
  - P1 — cargo cache broken + double build (49%) — **needs an owner**
  - P2 — `dsl_compile_clean` sound memo (23%) — declare `content_hash(tree)` input, then RecordedFixture
  - P3 — resolver resolve blowup (15%) — self-host blocker; merry-crab pulling fix forward
    - comprep closure ~250s cold, content-specific super-linear (machine-independent)
    - root-cause: H1 missing resolve memo (maybe bypassed `resolved_graph_cache`) vs H2 generic-instantiation
- Lens universalization + host-language ban — in flight (quiet-swift-814)
  - host-language ban: enforcement via `shell.Exec.Run.script` String→ShellProgram typecheck (#5147 landed; type-flip capstone warm-badger); #5132 substring lens retired TERMINAL
  - retire 2 `.sh` transports: blocked on parameterized-argv modeling (source_root_ingest, layering_imports_scan)
  - tier 0: `v2.std.lens_verdict` (`Holds | Violation | NotApplicable | Unrealized` fail-closed) — landing
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
