# How gunbc Works: A Worked Example

A single example traced through every layer of the system — from domain modeling
to generated binary. All code shown here is real, pulled from the repo.

---

## The 30-Second Version

gunbc is a **workflow compiler**. You write `.dag` files that declare what
external systems are, what you need from them, and how to orchestrate them.
The compiler validates the declarations, catches wiring errors at compile time,
and generates executable CLIs, test harnesses, and build targets — all from
a few dozen lines of declarative code.

The thesis: **move contradiction discovery from runtime to static analysis**.
If a `.dag` file compiles, its types are sound, its wiring is correct, and
its execution intent is unambiguous.

---

## The Example: `make gist`

One command that snapshots your repo's files and uploads them as a GitHub Gist.
It touches five external systems (git, filesystem, GCP secrets, GitHub API, shell),
requires authentication, and has three operating modes. In gunbc, the entire
workflow is 97 lines of `.dag` code. The compiler generates the CLI, the Makefile
target, and the test obligations.

Here's how each layer works, bottom to top.

---

## Layer 1: Domain Modeling — "What is this thing?"

Before writing any workflow logic, we declare **what external systems are** as
pure data. No functions, no I/O — just types and facts.

### What is Git? (`dsl/extdeps/git.dag`)

```
module extdeps.git

type ObjectType = BlobObj | TreeObj | CommitObj | TagObj

type GitCommit {
  sha: CommitSha
  message: String
  author: GitAuthor
  committer: GitAuthor
  parent_shas: List<CommitSha>
}

type GitBranch {
  name: String
  upstream: String?
  is_head: Bool
}

type DiffHunk {
  file_path: String
  old_start: Int
  new_start: Int
  lines: List<DiffLine>
}
```

### What is a GitHub Gist? (`dsl/extdeps/github/gists.dag`)

```
module extdeps.github.gists

type GistVisibility = Public | GistSecret

type GistFile {
  filename: String
  content: String
  language: String?
  size: Int
}

type Gist {
  id: String
  description: String?
  public: Bool
  files: List<GistFile>
  owner: String
  created_at: Timestamp
  updated_at: Timestamp
  url: String
}
```

These files cost nothing to create and nothing to delete. They're the
**domain vocabulary** — shared by every workflow that touches git or gists.

---

## Layer 2: Behavioral Modeling — "How does it behave?"

Each operation on an external system has declared behaviors: side effects,
idempotency, failure modes, prerequisites. This is structured data, not
comments.

```
// Still in extdeps/github/gists.dag

data operation_behaviors: List<GistOperationBehavior> = [
  {
    operation: "create",
    behavior: {
      side_effects: WritesState,
      idempotent: false,
      failure_modes: [
        { name: "ValidationError", condition: "empty_files",
          http_status: 422, recoverable: false, retry_safe: false }
      ],
      confidence: Documented,
      prerequisites: [{ description: "gist scope", kind: Capability }]
    }
  },
  {
    operation: "get",
    behavior: {
      side_effects: ReadOnly,
      idempotent: true,
      failure_modes: [
        { name: "NotFound", condition: "gist_absent",
          http_status: 404, recoverable: false, retry_safe: false }
      ],
      assumptions: ["Public gists are readable without authentication"]
    }
  }
]
```

The compiler uses these to generate appropriate test scenarios — a `WritesState`
operation gets different mock strategies than a `ReadOnly` one.

---

## Layer 3: Service Transport — "How do we call it?"

Now we declare the concrete transport wiring: endpoint, auth, REST method,
response codes. Structural blocks, not annotations.

```
// Still in extdeps/github/gists.dag

service github.Gist {
  config {
    endpoint: "https://api.github.com"
    auth: BearerToken
    auth_input: auth_token
    rate_limit: { requests: 5000, per: hour, scope: core }
    retry: { max_attempts: 3, backoff: exponential,
             retry_on: [429, 500, 502, 503, 504] }
  }

  operation Create {
    input {
      description: String
      content: String
      public: Bool = false
      auth_token: Secret
    }
    output {
      id: GistId
      html_url: Url
    }
    transport rest { method: POST, path: "/gists" }
    response {
      201 => Gist
      401 => GitHubErrorShape
      403 => GitHubErrorShape
      422 => GitHubErrorShape
      5xx => GitHubErrorShape
    }
  }
}
```

Each block composes additively. `config` sets the provider. `transport rest`
sets the HTTP method and path. `response` declares the contract. The compiler
fuses all of these into a transport triplet: `prepare → execute → parse`.

---

## Layer 4: Auth Credential Chain — "How do we authenticate?"

Authentication is itself a workflow — fetching a GitHub token from GCP Secret
Manager. Declared as a reusable function:

```
// extdeps/github/auth.dag

module extdeps.github.auth

import std.patterns { credential_chain }
import std.resources { Filesystem, Network }
import std.types { CloudRuntime, NonEmptyStr }

func github_token(
  runtime: CloudRuntime = LocalDev,
  project_id: NonEmptyStr = "gunbai-secrets",
  secret_name: NonEmptyStr = "github-token"
) -> { token: Secret }
  uses fs: Filesystem(mode: Read)
  uses net: Network
{
  cred = credential_chain(
    runtime: runtime,
    audience: "sigstore",
    service_account: None,
    secret_name: secret_name,
    project_id: project_id,
    source_id: "github-token",
    required_scopes: ["repo", "gist"]
  )
  return { token: cred.token.token }
}
```

The workflow that needs a token just calls `github_token()`. The compiler
wires the full credential chain into the graph — local auth, STS exchange,
and Secret Manager access all flow as typed DAG edges, never as globals.

---

## Layer 5: The Workflow — "What do we actually do?"

This is the user-facing tool. It imports domain types, service definitions,
and patterns, then composes them:

```
// tools/gist.dag — 97 lines total, 3 entrypoints

module tools.gist

import extdeps.git
import extdeps.github.auth { github_token }
import extdeps.github.gists
import std.resources { Filesystem, Network }
import std.patterns { read_text_files }
import std.types { CommitSha, Url }

fn render_diff_markdown(diff: String, branch: String, base_ref: CommitSha) -> String {
  "# Diff: {branch} vs {base_ref}\n\n```diff\n{diff}\n```\n"
}

func gist(public: Bool = false) -> { url: Url }
  uses fs: Filesystem(mode: Read)
  uses net: Network
{
  branch_info = git.Core.CurrentBranch()
  listing = git.Core.LsFiles()
  result = read_text_files(paths: listing.files)
  content = build_snapshot_content(
    branch: branch_info.branch,
    files: result.files |> map(f => f.path),
    file_contents: result.files |> map(f => f.content),
    skipped: result.skipped
  )
  token = github_token()
  gist_result = github.Gist.Create(
    description: branch_info.branch,
    content: content,
    public: public,
    auth_token: token.token
  )
  return { url: gist_result.html_url as Url }
}

func gist_diff(base_ref: CommitSha = "HEAD~1", public: Bool = false) -> { url: Url }
  uses fs: Filesystem(mode: Read)
  uses net: Network
{
  current = git.Core.CurrentBranch()
  changes = git.Core.Diff(base: base_ref)
  markdown = render_diff_markdown(
    diff: changes.diff, branch: current.branch, base_ref: base_ref)
  token = github_token()
  result = github.Gist.Create(
    description: current.branch, content: markdown,
    public: public, auth_token: token.token
  )
  return { url: result.html_url as Url }
}
```

Key things to notice:

- **`fn` vs `func`**: `fn` is pure (no I/O). `func` is effectful (calls services).
  The compiler enforces this — a `fn` cannot call a service.
- **`uses` declarations**: explicit resource requirements. `Filesystem(mode: Read)`
  means this workflow reads files. The compiler validates no write operations appear.
- **Service calls look like method calls**: `git.Core.CurrentBranch()` compiles to
  a shell transport triplet. `github.Gist.Create(...)` compiles to a REST transport
  triplet. The workflow author doesn't write HTTP code.
- **Data flows through typed ports**: `token.token` is a `Secret` type. The compiler
  knows it's a credential and handles redaction in dry-run output.

---

## Layer 6: Composition — The CI Pipeline

Tools compose into pipelines. The CI tool runs build, test, and clippy as
serialized stages:

```
// tools/ci.dag

func ci() -> { success: Bool, report: String } {
  build = cargo.Build.Build()
  test = cargo.Build.Test() [after build, when build.success]
  clippy = cargo.Build.Clippy() [after test]

  stages = [
    stage_from_output(name: "build",  success: build.success,  ...),
    stage_from_output(name: "test",   success: test.success,   ...),
    stage_from_output(name: "clippy", success: clippy.success, ...)
  ]
  summary = aggregate_results(stages: stages)
  report = format_report(summary: summary, stages: stages)

  return { success: build.success && test.success && clippy.success, report: report }
}
```

`[after build, when build.success]` is a **node guard** — test only runs after
build succeeds. This is structural, not control flow. The compiler sees the
dependency graph and can visualize it, schedule it, and prove it's acyclic
before any code runs.

---

## Layer 7: Build Targets — Policy as Data

Build targets are also declared in `.dag` files, not hardcoded in a Makefile:

```
// config/build_targets.dag

data core_workflows: List<CoreWorkflow> = [
  { name: "ensure-codegen",
    description: "Ensure CLI entrypoints exist",
    deps: [],
    body: ["@cargo run -p gunbc-app --bin gunbc-codegen -- codegen"] },
  { name: "build",
    description: "Build all workspace targets",
    deps: ["ensure-codegen"],
    body: ["@cargo build --workspace --all-targets"] },
  ...
]
```

The Makefile is **generated** by evaluating pure DSL functions against this data.
35 DSL functions in `tools/makegen.dag` render targets, help text, and dependency
chains — no Makefile is hand-written.

---

## What the Compiler Produces

From the `.dag` files above, the compiler generates:

### 1. CLI Binaries

For each `func` entrypoint, a complete CLI with argument parsing, dry-run mode,
and progress display:

```
$ make gist                    # snapshot mode
$ make gist-diff BASE=HEAD~3   # diff mode
$ make gist-recent SINCE=1w    # recent changes mode
```

The generated `main.rs` (~150 lines) handles:
- Argument parsing from the DSL-declared input ports
- Building the typed DAG at startup
- `--dry-run` mode that intercepts all transport boundaries (zero real I/O)
- Animated progress display

### 2. Makefile Targets

Every discovered tool automatically gets a Make target with help text:

```makefile
# gist entrypoints: public (Bool)
gist:
	@./target/release/gist $(if $(PUBLIC),--public $(PUBLIC))
```

### 3. Test Obligations

The compiler derives test obligations from the graph structure:
- **Execution**: dry-run completes without crash
- **Transport interception**: every I/O boundary is mockable
- **Skip propagation**: upstream failures propagate correctly
- **Resource lifecycle**: credentials acquire and release

### 4. Current Scale

```
17 generated CLI binaries (target/codegen/bin/)
19 tool workflows in dsl/tools/
35 DSL rendering functions (makegen alone)
Zero hand-written Makefile, CLI, or registration code
```

---

## The Compilation Pipeline

```
.dag source files
    │
    ▼
  Parse ──── syntax tree (AST)
    │
    ▼
  Resolve ── module graph (imports resolved)
    │
    ▼
  Typecheck ─ typed signatures, validated references
    │
    ▼
  Lower ──── DAG<LoweredOp> (typed graph IR)
    │         service calls → prepare/execute/parse triplets
    │         content_upsert → 5-node read/compare/write chains
    │         fn bodies → pure evaluation nodes
    │
    ▼
  Verify ─── VerifiedDag<LoweredOp> (acyclic, types match, ports saturated)
    │
    ▼
  Derive ─── callable properties, output paths, entrypoint inference
    │
    ▼
  Resolve ── DAG<DynOp> (extern functions bound to Rust implementations)
    │
    ▼
  Execute ── real mode (actual I/O) or dry-run (transport interception)
```

Every stage is a pure function with typed inputs and outputs. If any stage
fails, it produces a diagnostic with source location, error code, and
fix suggestion — never a silent drop or default substitution.

---

## The Key Insight

Traditional workflow code embeds structure in control flow:

```python
# The workflow is implicit in the code's execution order
branch = get_current_branch()
files = list_files()
content = render_markdown(files)
token = get_github_token()  # Where does this come from?
url = create_gist(token, content)  # What if this fails?
```

gunbc makes the workflow **the artifact**:

```
func gist() -> { url: Url }
  uses fs: Filesystem(mode: Read)      ← declared resource requirement
  uses net: Network                     ← declared capability
{
  branch_info = git.Core.CurrentBranch()    ← shell transport
  listing = git.Core.LsFiles()             ← shell transport
  content = build_snapshot_content(...)     ← pure computation
  token = github_token()                    ← credential chain (GCP → secret)
  gist_result = github.Gist.Create(...)    ← REST transport
  return { url: gist_result.html_url }
}
```

The compiler can see the entire graph before execution. It knows:
- What resources are needed (filesystem read, network)
- What credentials are required (GitHub token via GCP)
- What can fail (401, 422, 5xx — declared in the service definition)
- What's pure and what's I/O (fn vs func, transport boundaries)
- How to test it (intercept transports, inject mocks, verify propagation)

This is what "everything is a DAG" means in practice. The workflow,
the types, the tests, the build targets, and the CI pipeline are all
expressed in the same language, validated by the same compiler, and
generated from the same source of truth.

---

## Numbers

| Metric | Value |
|--------|-------|
| DSL compiler | ~55,000 lines of Rust across 8 crates |
| Core IR | ~51,000 lines |
| `.dag` source files | ~25,600 lines across 181 files |
|   — stdlib + behavioral vocab | ~3,600 lines |
|   — external system models | ~9,600 lines (github, git, cargo, gcp, llm, ...) |
|   — tool workflows | ~1,700 lines (19 tools) |
|   — config + policy data | ~1,600 lines |
|   — infrastructure providers | ~2,500 lines (gcp, aws, azure) |
| Generated CLI binaries | 17 |
| Extern bridge functions (Rust fallbacks) | 13 (shrinking — DSL replaces them) |
| Lines to add a new tool | ~20-50 (one `.dag` file) |
| Lines to add a tool traditionally | ~200-500 (Rust, 2-5 files) |
