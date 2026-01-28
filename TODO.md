# TODO

## Dependency DAG (gunbc-deps)
- Add macOS install commands for `buck2` (brew + release fallback) in `crates/gunbc-deps/src/graph.rs`.
- Add Windows install commands for `buck2` (choco + release fallback) in `crates/gunbc-deps/src/graph.rs`.
- Add macOS install commands for `rustup`/`rust` (currently linux-only upsert in `crates/gunbc-deps/src/graph.rs`).
- Add Windows install commands for `rustup`/`rust` (currently linux-only upsert in `crates/gunbc-deps/src/graph.rs`).
- Add platform-aware install nodes for `curl`/`zstd` using `apt`, `brew`, `choco` (currently linux-only upsert commands in `crates/gunbc-deps/src/graph.rs`).
- Add explicit nodes for package manager presence (apt, brew, choco) and use them as prerequisites where relevant.
- Add optional repo install steps (e.g., apt repo setup, brew taps, choco sources) as separate nodes to make side effects explicit.
- Model “install toolchain manager” (rustup) vs “install toolchain” (stable) separately for each OS with proper checks.
- Add `deps` entrypoints beyond `buck_*` (e.g., `gistgen`, `makegen`) with their own prerequisite subgraphs.

## Tooling
- Decide on a policy for auto-install vs check-only in CI (per entrypoint).
- Add a `--dry-run` or `--plan` output mode for `gunbc-deps` to show the intended upsert actions without executing.

## Dev UX (Makegen / Gistgen)
- [ ] Define minimal dev entrypoints for Makefile generation (start with `make gist`).
- [ ] Represent Git as an explicit dependency boundary in the gistgen DAG (inputs/outputs/ports clearly modeled).
- [ ] Decide on the Git interface for gistgen: CLI (`git ls-files`, etc.) vs Rust SDK; codify in DAG and ops.
- [ ] Add policy for default gist behavior (real vs dry-run) and selection spec defaults.
- [ ] Make gist snapshots repo-relative and honor gitignore via the Git boundary.

## Gitignoregen (future tool)
- [ ] Draft a DAG-driven `gitignoregen` tool to emit/maintain `.gitignore` from repo-specific rules.
- [ ] Define meta-rules for “what we don’t want to commit” and how they map to gitignore patterns.
- [ ] Decide whether gitignoregen should manage a marked block in `.gitignore` or own the full file.
