# Git Transport API: Hermetic Git Operations

**Status**: Done
**Date**: 2026-01-31
**Completed**: 2026-01-31

## Goal

Create a proper Git transport interface — a `GitRequest` builder type and `GitOps` pure operation enum — so that all git commands in the codebase go through the standard Prepare → Execute → Parse transport chain. No node should construct raw `ShellRequest { command: "git" ... }` directly. Git operations become hermetic, deterministic, interceptable by DryRun, and testable.

## Problem

### 1. Raw shell strings are fragile

The gist tool constructs git commands inline (`gist/graph.rs:166-177`):

```rust
let request = TransportRequest::Shell(ShellRequest {
    command: "git".to_string(),
    args: vec!["ls-files".to_string(), "--cached".to_string(), ...],
    cwd: Some(repo_path.to_string()),
    env: HashMap::new(),
    stdin: None,
});
```

This is the equivalent of SQL string concatenation. Every consumer re-invents argument construction, there's no central place to enforce deterministic flags, and new git operations (like `git diff` for the diff gist feature) would repeat the same pattern.

### 2. Git output is environment-sensitive

Git respects user config (`~/.gitconfig`, env vars) that can alter output format:

| Config | Effect | Breaks |
|--------|--------|--------|
| `color.ui=always` | ANSI escape codes in output | Any stdout parser |
| `diff.external` | Custom diff driver | diff header splitting |
| `diff.noprefix` | Removes `a/` `b/` prefixes | ParseDiff file extraction |
| `core.quotepath` | Escapes non-ASCII filenames | File path matching |
| `log.date` | Alters date format | Log parsing |
| `pager.*` | Pipes through less/more | Hangs in non-interactive |

Every git command needs a baseline set of flags to produce machine-parseable output. This belongs in one place, not scattered across graph builders.

### 3. No shared interface for new operations

The diff gist feature (see `TODO/TODONE/2026-Q1/diff-gist.md`) needs `git diff`. Future features may need `git log`, `git rev-parse`, `git status`. Without a shared interface, each would re-invent shell construction and output parsing.

## Existing Patterns to Follow

The codebase already has exactly the right layering. We mirror what `gist.rs` does for the GitHub Gist API:

```
core/ir/src/transport/
├── github/           ← Platform layer (auth, API, CLI)
│   ├── api.rs
│   └── cli.rs
├── gist.rs           ← Service layer: GistRequest builder → TransportRequest
├── git.rs            ← NEW: Service layer: GitRequest builder → TransportRequest
└── mod.rs
```

| Existing | Git equivalent |
|----------|---------------|
| `GistRequest::new().file("x", content).to_shell_request()` | `GitRequest::ls_files().cwd(path).to_shell_request()` |
| `GistOps::PrepareRequest` (pure op) | `GitOps::PrepareLsFiles` (pure op) |
| `GistOps::ParseGistResponse` (pure op) | `GitOps::ParseLsFiles` (pure op) |
| `parse_gist_url_from_shell()` helper | `parse_file_list_from_shell()` helper |

## Design

### Layer 1: `GitRequest` — Service-level request builder

**Location**: `core/ir/src/transport/git.rs` (new file)

```rust
/// Git operation request.
///
/// High-level representation of a git command that converts to a
/// deterministic TransportRequest::Shell. All git commands go through
/// this builder to enforce consistent, environment-independent output.
///
/// # Example
///
/// ```
/// let req = GitRequest::ls_files()
///     .cwd("/path/to/repo")
///     .to_shell_request();
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GitRequest {
    pub subcommand: GitSubcommand,
    pub cwd: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GitSubcommand {
    /// git ls-files --cached --others --exclude-standard
    LsFiles,
    /// git diff <base_ref>...HEAD (triple-dot: changes since fork point)
    Diff { base_ref: String },
    /// git diff --name-only <base_ref>...HEAD
    DiffNameOnly { base_ref: String },
    /// git rev-parse --abbrev-ref HEAD
    CurrentBranch,
    /// git merge-base <base_ref> HEAD
    MergeBase { base_ref: String },
}
```

### The Deterministic Shell Translation

`to_shell_request()` is the single source of truth for how git commands are invoked:

```rust
impl GitRequest {
    pub fn to_shell_request(&self) -> TransportRequest {
        let mut args = Vec::new();

        // ============================================================
        // Global flags: deterministic output regardless of user config
        // ============================================================
        // -c: override config for this invocation only
        args.extend([
            "-c", "color.ui=never",         // no ANSI escapes
            "-c", "core.quotepath=false",    // don't escape unicode paths
            "-c", "log.date=iso-strict",     // deterministic dates
        ].map(String::from));

        // --no-pager: never pipe through less/more
        args.push("--no-pager".into());

        match &self.subcommand {
            GitSubcommand::LsFiles => {
                args.extend([
                    "ls-files",
                    "--cached",
                    "--others",
                    "--exclude-standard",
                ].map(String::from));
            }
            GitSubcommand::Diff { base_ref } => {
                args.extend([
                    "diff",
                    "--no-ext-diff",        // no external diff driver
                    "--no-color",           // redundant with color.ui=never, belt+suspenders
                    "--src-prefix=a/",      // enforce standard prefixes
                    "--dst-prefix=b/",      // even if diff.noprefix is set
                    "--find-renames",       // detect renames
                ].map(String::from));
                args.push(format!("{}...HEAD", base_ref));
            }
            GitSubcommand::DiffNameOnly { base_ref } => {
                args.extend([
                    "diff",
                    "--no-ext-diff",
                    "--no-color",
                    "--name-only",
                ].map(String::from));
                args.push(format!("{}...HEAD", base_ref));
            }
            GitSubcommand::CurrentBranch => {
                args.extend([
                    "rev-parse",
                    "--abbrev-ref",
                    "HEAD",
                ].map(String::from));
            }
            GitSubcommand::MergeBase { base_ref } => {
                args.extend(["merge-base"].map(String::from));
                args.push(base_ref.clone());
                args.push("HEAD".into());
            }
        }

        TransportRequest::Shell(ShellRequest {
            command: "git".to_string(),
            args,
            cwd: self.cwd.clone(),
            env: HashMap::new(),
            stdin: None,
        })
    }
}
```

### Fluent Builder API

```rust
impl GitRequest {
    /// List tracked and untracked files (respects .gitignore).
    pub fn ls_files() -> Self {
        Self { subcommand: GitSubcommand::LsFiles, cwd: None }
    }

    /// Unified diff: changes on HEAD since it diverged from base_ref.
    pub fn diff(base_ref: impl Into<String>) -> Self {
        Self { subcommand: GitSubcommand::Diff { base_ref: base_ref.into() }, cwd: None }
    }

    /// File list only: names of files changed since base_ref.
    pub fn diff_name_only(base_ref: impl Into<String>) -> Self {
        Self { subcommand: GitSubcommand::DiffNameOnly { base_ref: base_ref.into() }, cwd: None }
    }

    /// Current branch name (or "HEAD" if detached).
    pub fn current_branch() -> Self {
        Self { subcommand: GitSubcommand::CurrentBranch, cwd: None }
    }

    /// Common ancestor commit between base_ref and HEAD.
    pub fn merge_base(base_ref: impl Into<String>) -> Self {
        Self { subcommand: GitSubcommand::MergeBase { base_ref: base_ref.into() }, cwd: None }
    }

    /// Set working directory for the git command.
    pub fn cwd(mut self, path: impl Into<String>) -> Self {
        self.cwd = Some(path.into());
        self
    }
}
```

### Response Parsers (standalone helpers, like `parse_gist_url_from_shell`)

```rust
/// Parse file list from `git ls-files` output.
pub fn parse_ls_files(stdout: &str) -> Vec<String> {
    stdout.lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .map(String::from)
        .collect()
}

/// Parse unified diff into per-file chunks.
/// Returns BTreeMap<filename, diff_chunk> for deterministic ordering.
///
/// Splits on `diff --git a/... b/...` headers.
/// Key = the b/ path (post-image filename), prefix stripped.
/// Value = entire chunk including header and hunks.
pub fn parse_diff_chunks(stdout: &str) -> BTreeMap<String, String> {
    // Split on "diff --git" boundaries
    // Extract filename from "b/<path>" in the header
    // Collect into ordered map
}

/// Parse a single branch name from `git rev-parse --abbrev-ref HEAD`.
pub fn parse_current_branch(stdout: &str) -> String {
    stdout.trim().to_string()
}

/// Parse a commit hash from `git merge-base` output.
pub fn parse_merge_base(stdout: &str) -> String {
    stdout.trim().to_string()
}

/// Compute diff stats from parsed diff chunks.
/// Returns (additions, deletions, file_count).
pub fn diff_stats(chunks: &BTreeMap<String, String>) -> (usize, usize, usize) {
    // Count +/- lines across all chunks
}
```

### Layer 2: `GitOps` — Pure operation enum for DAG nodes

**Location**: `lib/git-ops/src/lib.rs` (new crate)

Follows the exact pattern of `GistOps`:

```rust
/// Git operations for use in DAG nodes.
///
/// All operations are PURE — no I/O. They build TransportRequests or
/// parse TransportResponses. Actual I/O happens at TransportOps::Execute.
#[derive(Debug, Clone)]
pub enum GitOps {
    /// Build a git ls-files request (PURE)
    PrepareLsFiles,
    /// Parse ls-files response into file list (PURE)
    ParseLsFiles,
    /// Build a git diff request (PURE)
    PrepareDiff { base_ref: String },
    /// Parse unified diff into per-file chunks (PURE)
    ParseDiff,
    /// Build a git diff --name-only request (PURE)
    PrepareDiffNameOnly { base_ref: String },
    /// Parse diff --name-only into file list (PURE)
    ParseDiffNameOnly,
    /// Build a git rev-parse --abbrev-ref HEAD request (PURE)
    PrepareCurrentBranch,
    /// Parse current branch name (PURE)
    ParseCurrentBranch,
}

impl Executable for GitOps {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            GitOps::PrepareLsFiles => {
                let repo_path = inputs.get("repo_path").and_then(|v| v.as_str()).unwrap_or(".");
                let request = GitRequest::ls_files().cwd(repo_path).to_shell_request();
                Ok(hashmap!{ "request" => Value::Request(request) })
            }
            GitOps::ParseLsFiles => {
                let response = inputs.get("response").and_then(|v| v.as_response())?;
                let files = parse_ls_files(&shell_stdout(response));
                Ok(hashmap!{ "files" => Value::StrList(files) })
            }
            GitOps::PrepareDiff { base_ref } => {
                let repo_path = inputs.get("repo_path").and_then(|v| v.as_str()).unwrap_or(".");
                // Allow runtime override of base_ref
                let effective_ref = inputs.get("base_ref")
                    .and_then(|v| v.as_str())
                    .unwrap_or(base_ref);
                let request = GitRequest::diff(effective_ref).cwd(repo_path).to_shell_request();
                Ok(hashmap!{ "request" => Value::Request(request) })
            }
            GitOps::ParseDiff => {
                let response = inputs.get("response").and_then(|v| v.as_response())?;
                let stdout = shell_stdout(response);
                let chunks = parse_diff_chunks(&stdout);
                let (adds, dels, count) = diff_stats(&chunks);
                Ok(hashmap!{
                    "diff_files" => Value::MapStrStr(chunks),
                    "stats" => Value::Str(format!("+{} -{} across {} files", adds, dels, count)),
                })
            }
            // ... other variants follow the same Prepare/Parse pattern
        }
    }
}
```

### Layer 3: Integration — Migrating the gist tool

**Current** (`gist/graph.rs`): Inline `ShellRequest` construction + inline parsing.

**After**: The gist graph op enum gains a `Git(GitOps)` variant, and the graph uses it:

```rust
// Before (inline shell request)
GistGraphOp::PrepareListFiles  // → raw ShellRequest { command: "git", args: [...] }

// After (via GitOps)
GistGraphOp::Git(GitOps::PrepareLsFiles)  // → GitRequest::ls_files().to_shell_request()
GistGraphOp::Git(GitOps::ParseLsFiles)    // → parse_ls_files(stdout)
```

The existing `execute_prepare_list_files` and `execute_parse_list_files` functions in `gist/graph.rs` are replaced by `GitOps::PrepareLsFiles` and `GitOps::ParseLsFiles`. The logic is the same, just moved to the git-ops crate where it's reusable.

### Layer 3b: Integration — Diff gist graph

The diff gist graph (from `TODO/TODONE/2026-Q1/diff-gist.md`) becomes a consumer of `GitOps`:

```
GitOps::PrepareDiff → TransportOps::Execute → GitOps::ParseDiff → Filter → Render → Gist
```

No raw shell strings. The deterministic flags are baked into `GitRequest::diff().to_shell_request()`.

## Module Layout

```
core/ir/src/transport/
├── git.rs              ← NEW: GitRequest, GitSubcommand, parsers
├── gist.rs             ← Existing: GistRequest (pattern to follow)
├── mod.rs              ← Add: pub mod git; pub use git::GitRequest;
└── ...

lib/git-ops/
├── Cargo.toml          ← NEW: crate
└── src/
    └── lib.rs          ← NEW: GitOps enum, Executable impl
```

The split mirrors the existing pattern:
- `core/ir/src/transport/gist.rs` → request builder + parsers (no DAG dependency)
- `lib/gist-ops/src/lib.rs` → DAG-aware ops enum (depends on `gunbc_exec`)

## Diagram

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          Consumer (gist graph, diff gist graph, CI, ...)    │
│                                                                             │
│    GitOps::PrepareLsFiles       GitOps::PrepareDiff { base_ref }           │
│          │                              │                                   │
│          ▼                              ▼                                   │
│    ┌───────────┐                  ┌───────────┐                             │
│    │ GitRequest │                  │ GitRequest │                            │
│    │ ::ls_files │                  │ ::diff     │                            │
│    │ .cwd(path) │                  │ .cwd(path) │                            │
│    └─────┬─────┘                  └─────┬─────┘                             │
│          │ .to_shell_request()          │ .to_shell_request()               │
│          ▼                              ▼                                   │
│   TransportRequest::Shell       TransportRequest::Shell                     │
│   ┌──────────────────────┐      ┌────────────────────────────────────────┐  │
│   │ git                  │      │ git                                    │  │
│   │  -c color.ui=never   │      │  -c color.ui=never                    │  │
│   │  -c core.quotepath=… │      │  -c core.quotepath=false              │  │
│   │  --no-pager          │      │  --no-pager                           │  │
│   │  ls-files            │      │  diff --no-ext-diff --no-color        │  │
│   │  --cached --others   │      │  --src-prefix=a/ --dst-prefix=b/      │  │
│   │  --exclude-standard  │      │  --find-renames main...HEAD           │  │
│   └──────────────────────┘      └────────────────────────────────────────┘  │
│          │                              │                                   │
│          ▼                              ▼                                   │
│   TransportOps::Execute (BOUNDARY — interceptable by DryRun)                │
│          │                              │                                   │
│          ▼                              ▼                                   │
│   TransportResponse::Shell      TransportResponse::Shell                    │
│          │                              │                                   │
│          ▼                              ▼                                   │
│    GitOps::ParseLsFiles          GitOps::ParseDiff                          │
│    → parse_ls_files(stdout)      → parse_diff_chunks(stdout)                │
│    → Vec<String>                 → BTreeMap<String, String>                 │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Migration Checklist

### Current git usages to migrate

| Location | Command | Replacement |
|----------|---------|-------------|
| `lib/tools/gist/src/graph.rs:166-177` | Raw `ShellRequest { command: "git", args: ["ls-files", ...] }` | `GitRequest::ls_files().cwd(repo_path).to_shell_request()` |
| `lib/tools/gist/src/graph.rs:197-220` | Inline `parse_list_files` function | `git::parse_ls_files()` from `core/ir/src/transport/git.rs` |
| `lib/tools/gist/src/graph_mock.rs` | Mock for `execute_list_files` | Unchanged (transport mock is format-agnostic) |

Only **one production call site** and **one parser** to migrate. All other git references are metadata (tool definitions, CI runner declarations, test assertions).

### New usages for diff gist

| Operation | GitRequest call |
|-----------|----------------|
| Get branch diff | `GitRequest::diff(&base_ref).cwd(repo_path)` |
| Get changed file list | `GitRequest::diff_name_only(&base_ref).cwd(repo_path)` |
| Detect current branch | `GitRequest::current_branch().cwd(repo_path)` |

## Tasks

### Phase 1: GitRequest builder + parsers (`core/ir`)

- [x] Create `core/ir/src/transport/git.rs`
- [x] Implement `GitRequest` struct with `GitSubcommand` enum
- [x] Implement `to_shell_request()` with deterministic global flags
- [x] Implement fluent builder methods (`ls_files()`, `diff()`, etc.)
- [x] Implement response parsers (`parse_ls_files`, `parse_diff_chunks`, etc.)
- [x] Add `pub mod git` to `core/ir/src/transport/mod.rs`
- [x] Add re-export: `pub use git::GitRequest`
- [x] Unit tests for each subcommand's shell translation
- [x] Unit tests for each parser

### Phase 2: GitOps crate (`lib/git-ops`)

- [x] Create `lib/git-ops/` crate with `Cargo.toml`
- [x] Implement `GitOps` enum with Prepare/Parse variants
- [x] Implement `Executable for GitOps`
- [x] Unit tests for each op (pure, so trivially testable)

### Phase 3: Migrate gist tool

- [x] Add `Git(GitOps)` variant to `GistGraphOp`
- [x] Replace inline `execute_prepare_list_files` with `GitOps::PrepareLsFiles`
- [x] Replace inline `execute_parse_list_files` with `GitOps::ParseLsFiles`
- [x] Verify mock specs still work (transport boundary unchanged)
- [x] Remove dead code (old inline implementations)

### Phase 4: Wire into diff gist

- [x] Use `GitOps::PrepareDiff` + `GitOps::ParseDiff` in `build_diff_gist_graph()`
- [x] (See `TODO/TODONE/2026-Q1/diff-gist.md` for the full diff gist pipeline)

## Design Decisions

### Why a `GitRequest` builder, not just `GitOps` directly?

Same reason `GistRequest` exists separately from `GistOps`. The builder lives in `core/ir` (no DAG dependency) and produces `TransportRequest` values. The ops enum lives in `lib/git-ops` (depends on `gunbc_exec`) and wraps the builder for use in DAG nodes. This keeps the IR layer clean and lets non-DAG code (tests, CLI tools) use `GitRequest` directly.

### Why enforce flags in `to_shell_request()` instead of per-caller?

Centralization. If we discover another config knob that breaks parsing (and git has hundreds), we fix it in one place. Callers get deterministic output by construction — they can't forget to add `--no-color`.

### Why `GitSubcommand` enum instead of free-form args?

Type safety. A `GitSubcommand::Diff { base_ref }` can only produce a diff command with the correct flags. Free-form args would let callers construct arbitrary git commands, defeating the purpose of the interface. If a new subcommand is needed, it's a one-variant addition to the enum — an explicit design decision, not an implicit shell string.

### What about write operations (commit, push, checkout)?

The current `GitSubcommand` variants are all read-only (`AccessMode::Read`). Write operations (`git commit`, `git push`, `git checkout`) would need `AccessMode::Exclusive` and should carry that in the type:

```rust
pub enum GitSubcommand {
    // Read operations (parallelizable)
    LsFiles,
    Diff { base_ref: String },
    // ...

    // Write operations (exclusive - future)
    // Commit { message: String },
    // Push { remote: String, branch: String },
}
```

For now, we only need read operations. Write operations can be added when there's a concrete use case, following the same pattern.

### Extensibility for future subcommands

Adding a new git operation is mechanical:

1. Add variant to `GitSubcommand` enum
2. Add builder method to `GitRequest` (e.g., `GitRequest::log(n)`)
3. Add match arm in `to_shell_request()` with appropriate flags
4. Add parser function (e.g., `parse_log_entries`)
5. Add Prepare/Parse variants to `GitOps`

No existing code changes. No new patterns to learn.

## Related

- `TODO/TODONE/2026-Q1/diff-gist.md` — Consumer: uses `GitOps::PrepareDiff` + `GitOps::ParseDiff`
- `core/ir/src/transport/gist.rs` — Pattern source: `GistRequest` builder
- `lib/gist-ops/src/lib.rs` — Pattern source: `GistOps` pure ops
- `core/ir/src/transport/cli.rs:477-484` — Existing `CliToolDef::GIT` (tool acquisition, unchanged)
- `core/ir/src/git.rs` — `GitConfig.default_branch` (source of `base_ref`)
