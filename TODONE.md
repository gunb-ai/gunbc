# TODONE

Completed items moved from TODO.md.

## Tooling

- [x] Decide on a policy for auto-install vs check-only in CI (per entrypoint). **DONE** — Default mode is now `upsert` (install if missing). Use `--mode check` to fail fast without installing.
- [x] Add a `--dry-run` or `--plan` output mode for `gunbc-deps` to show the intended upsert actions without executing. **DONE** — `--dry-run` flag added with operation swapping (`InstallCommand` vs `PreviewInstall`). Note: implementation still manually threads `dry_run` through graph builder; see audit TODO for infrastructure-level fix.

## Gitignoregen

- [x] Draft a DAG-driven `gitignoregen` tool to emit/maintain `.gitignore` from repo-specific rules. **DONE** — `gunbc-gitignoregen` crate created with DAG-driven architecture.
- [x] Define meta-rules for "what we don't want to commit" and how they map to gitignore patterns. **DONE** — Rules defined in `GITIGNORE_CONTENT` constant covering build artifacts, editor state, OS metadata, and secrets.
- [x] Decide whether gitignoregen should manage a marked block in `.gitignore` or own the full file. **DONE** — Full file ownership with hash-based idempotency.

## File Transport Shared Execution

- [x] Implement shared file execution via `gunbc-file-exec` (Executable for `FileOp` + hash utilities).
- [x] Refactor makegen/gitignoregen to compose `FileOp` instead of duplicating Check/Resolve/Write/Print logic.
- [x] Update file upsert DAG shape to guarded write + final resolve, with boundary declarations on real writes.

## Dev UX (Makegen / Gistgen)

- [x] Define minimal dev entrypoints for Makefile generation (start with `make gist`). **DONE** — Makegen now generates a minimal Makefile with just the `gist` target (`crates/gunbc-makegen/src/ops.rs:83-103`).
- [x] Represent Git as an explicit dependency boundary in the gistgen DAG. **DONE** — `BoundaryDeclaration` for `git_repo()` external type in `crates/gunbc-gistgen/src/graph.rs:175-179`.
- [x] Decide on the Git interface for gistgen: CLI vs Rust SDK. **DONE** — Uses CLI via `git ls-files -co --exclude-standard -z` in `crates/gunbc-gistgen/src/ops.rs:520-541`.
- [x] Add policy for default gist behavior (real vs dry-run) and selection spec defaults. **DONE** — `UnderstandingMode::Real` is the default (`crates/gunbc-gistgen/src/graph.rs:14-23`), glob default is `**/*` for all files.
- [x] Make gist snapshots repo-relative and honor gitignore via the Git boundary. **DONE** — Uses `--exclude-standard` flag to honor gitignore + `normalize_repo_relative()` for repo-relative paths (`crates/gunbc-gistgen/src/ops.rs:548-553`).
