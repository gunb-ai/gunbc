# Diff Gist: Branch Diff Snapshot

**Status**: Done
**Date**: 2026-01-31
**Completed**: 2026-01-31
**Depends on**: `TODONE/git-transport-api.md` (Git transport interface)

## Goal

Extend the gist tool with a **diff mode** that captures the changes between the current branch and the repo's default branch, rather than snapshotting the entire codebase. This produces a focused, reviewable gist of "what changed" — useful for code review, sharing WIP, or feeding context to LLMs.

## Problem Statement

The current gist tool runs `git ls-files` and reads every file's full content. For a quick "here's what I changed on this branch" workflow, this is the wrong granularity:

1. **Too much noise** — The full codebase buries the signal (the changes)
2. **Wrong audience** — Reviewers / LLMs want diffs, not the whole repo
3. **Large output** — Full snapshots hit gist size limits on non-trivial repos

The diff gist should produce a markdown gist containing only the unified diffs of files changed on the current branch relative to the default branch.

## Existing Infrastructure

### What we have

| Component | Location | Relevant |
|-----------|----------|----------|
| `GitConfig.default_branch` | `core/ir/src/git.rs` | Defines the default branch (defaults to `"main"`) |
| Gist graph pipeline | `lib/tools/gist/src/graph.rs` | ListFiles → Filter → ReadFiles → Render → CreateGist |
| `MarkdownOp::RenderCodeSnapshot` | `lib/markdown/src/lib.rs` | Renders `MapStrStr` as fenced code blocks |
| `GistOps::PrepareRequest` | `lib/gist-ops/src/lib.rs` | Builds `gh gist create` TransportRequest |
| `Value::MapStrStr` | `core/ir/src/value.rs` | Fits `filename → diff_chunk` naturally |
| Transport pattern | Throughout | Prepare → Execute → Parse chains |

### What's missing

| Gap | Notes |
|-----|-------|
| Repo-level config in gunbc-dag | `GitConfig` exists but isn't wired into gunbc-dag as repo config yet |
| Diff transport chain | No `git diff` operations exist anywhere |
| Diff markdown rendering | `MarkdownOp` only has `RenderCodeSnapshot`, no diff variant |
| Diff gist graph | No graph builder for the diff pipeline |

## Design

### Pipeline Overview

```
PrepareDiff → Execute → ParseDiff → FilterDiffFiles → RenderDiffSnapshot → PrepareGist → Execute → ParseGistResponse
    │             │          │              │                  │                 │            │            │
  (PURE)      (BOUNDARY)  (PURE)        (PURE)             (PURE)           (PURE)     (BOUNDARY)     (PURE)
```

**8 nodes, 7 edges, 2 transport boundaries** (vs the current snapshot pipeline's 11 nodes, 3 boundaries).

### Node Details

#### 1. PrepareDiff (PURE) — via `GitOps::PrepareDiff`

Uses the Git transport API (`TODO/TODONE/2026-Q1/git-transport-api.md`) rather than constructing raw shell strings. The `GitRequest::diff()` builder enforces deterministic output flags (`--no-ext-diff`, `--no-color`, `--src-prefix=a/`, `--dst-prefix=b/`, etc.) so parsing is stable across environments.

```rust
/// Implemented as GitOps::PrepareDiff { base_ref }
///
/// Inputs:
///   - repo_path: String (optional, defaults to ".")
///   - base_ref: String (optional, overrides build-time base_ref)
///
/// Outputs:
///   - request: TransportRequest
///
/// Internally calls:
///   GitRequest::diff(base_ref).cwd(repo_path).to_shell_request()
///
/// Which produces (deterministic, environment-independent):
///   git -c color.ui=never -c core.quotepath=false --no-pager
///       diff --no-ext-diff --no-color --src-prefix=a/ --dst-prefix=b/
///       --find-renames main...HEAD
```

**Why `...` (triple-dot)?** `git diff main...HEAD` shows changes introduced on HEAD since it diverged from main. This is exactly "what this branch changed" — unaffected by commits that landed on main after the branch point. This matches what GitHub shows in a PR diff.

#### 2. Execute (BOUNDARY)

Standard `TransportOps::Execute`. Interceptable by dry-run.

#### 3. ParseDiff (PURE) — via `GitOps::ParseDiff`

Delegates to `git::parse_diff_chunks()` from the Git transport API. The parser is safe to rely on because `GitRequest::diff().to_shell_request()` enforces `--src-prefix=a/` and `--dst-prefix=b/`, guaranteeing stable `diff --git a/... b/...` headers regardless of user config.

```rust
/// Implemented as GitOps::ParseDiff
///
/// Inputs:
///   - response: TransportResponse (unified diff output)
///
/// Outputs:
///   - diff_files: MapStrStr (filename → diff_chunk)
///   - stats: String (summary like "+42 -17 across 5 files")
///
/// Internally calls:
///   git::parse_diff_chunks(stdout)  → BTreeMap<String, String>
///   git::diff_stats(&chunks)        → (additions, deletions, file_count)
```

This naturally produces a `MapStrStr` — the same type that the current `ReadFiles` chain produces, making downstream nodes compatible.

#### 4. FilterDiffFiles (PURE)

Reuses the existing extension-filter pattern. Filters the `MapStrStr` keys by extension.

```rust
/// Same logic as FilterByExtension but operates on MapStrStr keys
/// instead of StrList.
///
/// Inputs:
///   - diff_files: MapStrStr
///
/// Outputs:
///   - diff_files: MapStrStr (filtered)
```

**Note:** Could generalize `FilterByExtension` to work on both `StrList` and `MapStrStr`, or create a `FilterMapByExtension` variant. The latter is simpler and avoids changing existing code.

#### 5. RenderDiffSnapshot (PURE — new MarkdownOp variant)

Renders per-file diffs as a markdown document.

```rust
/// New variant: MarkdownOp::RenderDiffSnapshot
///
/// Inputs:
///   - diff_files: MapStrStr (filename → diff_chunk)
///   - stats: String (optional summary)
///
/// Outputs:
///   - markdown: String
///
/// Renders:
///   # Branch Diff
///   > +42 -17 across 5 files
///
///   ## `src/graph.rs`
///   ```diff
///   @@ -10,6 +10,8 @@
///    fn existing_code() {
///   +    new_code();
///    }
///   ```
///
///   ## `src/ops.rs`
///   ```diff
///   ...
///   ```
```

All diff chunks use the `diff` language identifier for syntax highlighting. The per-file structure is important: it makes the gist scannable (table of contents in GitHub's gist viewer).

#### 6-8. PrepareGist → Execute → ParseGistResponse

**Identical to the current gist pipeline.** The `PrepareGist` node receives markdown and builds a `gh gist create` request. No changes needed — the gist creation is format-agnostic.

### Graph Builder Signature

```rust
/// Build a diff-mode gist graph.
///
/// Parameters:
///   - base_ref: The branch to diff against (e.g., "main").
///               Sourced from GitConfig.default_branch or overridden.
///   - extensions: File extensions to include (empty = all).
///   - public: Whether the gist is public.
pub fn build_diff_gist_graph(
    base_ref: &str,
    extensions: Vec<String>,
    public: bool,
) -> Result<Dag<GistGraphOp>, BuilderError>
```

### Workflow Signature

```rust
pub fn diff_gist_signature() -> WorkflowSignature {
    WorkflowSignature::new()
        .with_input("repo_path", "String", Cardinality::ZERO_OR_ONE)
        .with_input("base_ref", "String", Cardinality::ZERO_OR_ONE)
        .with_output("url", "String", Cardinality::ONE)
}
```

`base_ref` is an optional runtime input (defaults to whatever was passed at build time). This allows the caller to override — e.g., diff against a release branch instead.

### New GistGraphOp Variants

```rust
pub enum GistGraphOp {
    // ... existing variants ...

    // Git operations (via git-ops crate, see TODO/TODONE/2026-Q1/git-transport-api.md)
    /// Git operations (PURE - builds requests, parses responses)
    Git(GitOps),

    // Diff-specific
    /// Filter diff map by extension (PURE - no I/O)
    FilterDiffByExtension { extensions: Vec<String> },
}
```

`GitOps::PrepareDiff` and `GitOps::ParseDiff` come from the `git-ops` crate. The same `Git(GitOps)` variant also replaces the existing inline `PrepareListFiles` / `ParseListFiles` in the snapshot graph.

### Where `base_ref` Comes From

The `base_ref` (default branch to diff against) should resolve through this chain:

1. **Runtime input** — If provided as `base_ref` input to the workflow, use it
2. **Build-time parameter** — Passed to `build_diff_gist_graph(base_ref, ...)`
3. **Repo config** — `GitConfig.default_branch` (currently defaults to `"main"`)

For the initial implementation, the build-time parameter is sufficient. The caller (binary entrypoint) reads `GitConfig` and passes it in:

```rust
// In the binary entrypoint (gunbc-dag/src/bin/gist.rs or similar)
let git_config = GitConfig::default(); // or load from repo config
let dag = build_diff_gist_graph(
    &git_config.default_branch,
    extensions,
    public,
)?;
```

### Repo Config (Future)

The comment in `core/ir/src/git.rs` says:

> Repo-specific choices (e.g., "our default branch is main") live in `gunbc-dag` alongside other repo config.

This design doesn't block on a full `RepoConfig` struct. The binary entrypoint can hardcode `GitConfig::default()` for now and later read from a config file when `RepoConfig` is introduced. The graph itself is parameterized — it doesn't care where `base_ref` came from.

## DAG Diagram

```
                          ┌──────────────────────────────────────────────────────────────────────┐
                          │                     Diff Gist Pipeline                               │
                          └──────────────────────────────────────────────────────────────────────┘

  repo_path (opt)──┐
                   ▼
             ┌─────────────┐     ┌──────────────┐     ┌───────────┐     ┌─────────────────────┐
  base_ref──▶│ PrepareDiff │────▶│ Execute      │────▶│ ParseDiff │────▶│ FilterDiffByExt     │
             │   (PURE)    │ req │ (BOUNDARY)   │ resp│  (PURE)   │ map │      (PURE)         │
             └─────────────┘     └──────────────┘     └───────────┘     └──────────┬──────────┘
                                                                                    │ map
                                                                                    ▼
             ┌──────────────┐     ┌──────────────┐     ┌──────────────────────────────────────┐
             │ ParseGistResp│◀────│ ExecuteGist  │◀────│ PrepareGist                          │
             │   (PURE)     │ resp│ (BOUNDARY)   │ req │   (PURE)                             │
             └──────┬───────┘     └──────────────┘     └────────────────────────┬─────────────┘
                    │                                                            ▲
                    ▼                                                            │ md
                  url                                        ┌──────────────────────────────┐
                                                             │ RenderDiffSnapshot           │
                                                             │ (PURE - MarkdownOp)          │
                                                             └──────────────────────────────┘
```

Linearized:

```
PrepareDiff → Execute → ParseDiff → FilterDiffByExt → RenderDiffSnapshot → PrepareGist → ExecuteGist → ParseGistResponse
                ↑                                                                             ↑
            (boundary)                                                                    (boundary)
```

## Comparison: Snapshot vs Diff

| Aspect | Snapshot (current) | Diff (new) |
|--------|-------------------|------------|
| Git command | `git ls-files` + batch `cat` | `git diff base...HEAD` |
| Transport calls | 3 (list, read, gist) | 2 (diff, gist) |
| Data shape | `MapStrStr` (file → content) | `MapStrStr` (file → diff_chunk) |
| Rendering | `# Code Snapshot` + full code blocks | `# Branch Diff` + diff blocks |
| Filtering | Extension filter on file list | Extension filter on diff map keys |
| Output size | Entire codebase | Only changed hunks |
| Use case | Share full codebase context | Share "what changed" for review |

## Edge Cases

| Case | Behavior |
|------|----------|
| No changes (empty diff) | ParseDiff returns empty `MapStrStr` → gist contains "No changes" |
| Binary files in diff | `git diff` shows `Binary files differ` → include as-is in chunk |
| New files (no base) | `git diff` includes full file as additions (`+` lines) |
| Deleted files | Included as full removals (`-` lines) |
| Renamed files | `git diff` with rename detection shows `rename from → to` |
| Detached HEAD | `git diff base...HEAD` still works (HEAD resolves to commit) |
| `base_ref` doesn't exist | Transport returns non-zero exit → surface error |
| Non-git repo | `git diff` fails → surface error (unlike ls-files which returns empty) |

## Tasks

### Phase 1: Core Diff Pipeline

- [x] Add `PrepareDiff` op + `execute_prepare_diff` to `GistGraphOp` (via `Git(GitOps::PrepareDiff)`)
- [x] Add `ParseDiff` op + `execute_parse_diff` (via `Git(GitOps::ParseDiff)`)
- [x] Add `FilterDiffByExtension` op (filter `MapStrStr` keys by extension)
- [x] Add `MarkdownOp::RenderDiffSnapshot` to `lib/markdown/`
- [x] Add `build_diff_gist_graph()` graph builder
- [x] Add `diff_gist_signature()` workflow signature
- [x] Add unit tests for each new op (pure functions are trivially testable)

### Phase 2: Integration

- [x] Add mock specs for diff graph (`graph_mock.rs`)
- [ ] Add binary entrypoint or CLI flag to invoke diff mode (future: depends on CLI design)
- [ ] Wire `GitConfig.default_branch` as the default `base_ref` (future: needs RepoConfig)
- [x] Add integration test: build graph, validate signature, dry-run

### Phase 3: Polish

- [x] Handle empty diff gracefully (render "No changes between X and HEAD")
- [x] Add `--stat` summary line to rendered markdown (insertions/deletions)
- [ ] Consider adding `--name-only` pre-check to skip diff when no files match filter (optional optimization)
- [ ] Document in AGENT.md or README

## Design Decisions

### Why a separate graph, not a mode flag?

The diff pipeline is structurally different from the snapshot pipeline (different transport chains, different ops, different node count). A `build_diff_gist_graph()` alongside `build_gist_graph()` is cleaner than a branching graph with conditional nodes. The shared parts (gist creation, markdown rendering) are already in separate library crates and compose naturally.

### Why parse into `MapStrStr` instead of a single diff string?

1. **Filtering** — Extension filtering operates on file names; per-file chunks enable this
2. **Rendering** — Per-file sections make the gist scannable with a table of contents
3. **Compatibility** — Same type as the snapshot pipeline's file contents, so `PrepareGist` works unchanged
4. **Future** — Per-file chunks could feed into per-file LLM review, parallel processing, etc.

### Why `...` (triple-dot) instead of `..` (double-dot)?

`git diff main...HEAD` = "changes on HEAD since it forked from main". This is stable — it doesn't change when new commits land on main. It matches what you see in a GitHub PR diff. Double-dot (`main..HEAD`) shows the symmetric difference, which includes main's new commits as removals — confusing for "what did I change" workflows.

### Why not add a new `Value::Diff` variant?

The `MapStrStr` type already fits perfectly (filename → diff content). Adding a new Value variant is a cross-cutting change that touches the IR, serialization, and every tool. Not worth it for what is semantically "a map of strings."

## Related Files

- `TODO/TODONE/2026-Q1/git-transport-api.md` — **Dependency**: Git transport interface (GitRequest, GitOps)
- `core/ir/src/git.rs` — `GitConfig.default_branch` (source of `base_ref`)
- `core/ir/src/transport/git.rs` — **New**: `GitRequest` builder + parsers
- `lib/git-ops/src/lib.rs` — **New**: `GitOps` pure ops enum
- `lib/tools/gist/src/graph.rs` — Current snapshot graph (pattern to follow)
- `lib/gist-ops/src/lib.rs` — `GistOps` (reused as-is for gist creation)
- `lib/markdown/src/lib.rs` — `MarkdownOp` (needs `RenderDiffSnapshot` variant)
- `core/ir/src/value.rs` — `Value::MapStrStr` (carries per-file diff chunks)
