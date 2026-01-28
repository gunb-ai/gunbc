# TODO

## Dry-Run Infrastructure Audit

- [ ] Audit dry-run implementations across all tools (`gunbc-deps`, `gunbc-makegen`, `gunbc-gistgen`) — currently all manually thread `dry_run` through graph builders instead of having infrastructure deduce it from external boundaries.
- [ ] Write a design doc for "sentinel world-write nodes" + deepest dry-run transform (tradeoffs, open questions). See `docs/dry-run-sentinel-boundaries.md`.
- [ ] Add `writes_world() -> bool` method to operations (or use `BoundaryDeclaration` metadata).
- [ ] Add `to_preview() -> Self` method for automatic operation swapping.
- [ ] Move dry-run logic to infrastructure layer so graph builders don't need to know about it.
- [ ] Remove manual `dry_run` parameters from all `build_graph()` functions.

See `docs/postmortem-dry-run.md` for detailed analysis.

## Dependency DAG (gunbc-deps)

- Add macOS install commands for `buck2` (brew + release fallback) in `crates/gunbc-deps/src/graph.rs`.
- Add Windows install commands for `buck2` (choco + release fallback) in `crates/gunbc-deps/src/graph.rs`.
- Add macOS install commands for `rustup`/`rust` (currently linux-only upsert in `crates/gunbc-deps/src/graph.rs`).
- Add Windows install commands for `rustup`/`rust` (currently linux-only upsert in `crates/gunbc-deps/src/graph.rs`).
- Add platform-aware install nodes for `curl`/`zstd` using `apt`, `brew`, `choco` (currently linux-only upsert commands in `crates/gunbc-deps/src/graph.rs`).
- Add explicit nodes for package manager presence (apt, brew, choco) and use them as prerequisites where relevant.
- Add optional repo install steps (e.g., apt repo setup, brew taps, choco sources) as separate nodes to make side effects explicit.
- Model "install toolchain manager" (rustup) vs "install toolchain" (stable) separately for each OS with proper checks.
- Add `deps` entrypoints beyond `buck_*` (e.g., `gistgen`, `makegen`) with their own prerequisite subgraphs.
