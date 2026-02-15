# Causal DAG Language: Design Document

**Status**: Working Draft — February 2026
**Repo**: New (harvesting from gunbc, the-gunbai, gunb.ai)

---

## Table of Contents

1. [Problem Statement](#1-problem-statement)
2. [Three Generations of Evidence](#2-three-generations-of-evidence)
3. [Design Principles](#3-design-principles)
4. [Language Constructs](#4-language-constructs)
5. [Module System and Discovery](#5-module-system-and-discovery)
6. [Terminal Progress Model](#6-terminal-progress-model)
7. [Resource Model](#7-resource-model)
8. [Compiler Pipeline](#8-compiler-pipeline)
9. [Multi-Target Emission](#9-multi-target-emission)
10. [What to Harvest](#10-what-to-harvest)
11. [Phasing](#11-phasing)

**Appendices**:

- [A. Worked Example: Content Upsert (Makegen)](#appendix-a-content-upsert-makegen)
- [B. Worked Example: Cloud Credential Acquisition (GCP)](#appendix-b-cloud-credential-acquisition-gcp)
- [C. Worked Example: Service Composition (Gist Snapshot)](#appendix-c-service-composition-gist-snapshot)
- [D. Worked Example: CI Pipeline](#appendix-d-ci-pipeline)
- [E. Worked Example: Tool Installation (Upsert)](#appendix-e-tool-installation-upsert)
- [F. Worked Example: LLM Review Workflow](#appendix-f-llm-review-workflow)
- [G. Worked Example: Rendering / Emission](#appendix-g-rendering--emission)
- [H. Pattern Catalog](#appendix-h-pattern-catalog)
- [I. Inspiration Targets](#appendix-i-inspiration-targets)
- [J. Cross-Repository Capability Matrix](#appendix-j-cross-repository-capability-matrix)

---

## 1. Problem Statement

We need a language for authoring causal DAGs that:

1. **Compresses graph authoring** from thousands of lines of host-language builder code to tens of lines of declarations, while preserving the structural guarantees (acyclicity, type safety, port saturation) that the IR provides.

2. **Makes terminal progress structural** — the shape of progress (waves, groups, expand points) is known at compile time, not derived at runtime. This enables static visualization (before execution), live progress, and post-execution replay from the same manifest.

3. **Solves discovery by construction** — every `.dag` file is auto-discovered via the filesystem. No registration macros, no hardcoded lists, no islands. dag-viz can see itself.

4. **Models resources with lifecycle** — acquire, use, release. Resources are declared (`uses fs: Filesystem`), not manually wired through environment nodes and `res:` port conventions.

5. **Is language-agnostic** — `.dag` files compile to a target-independent IR. Codegen backends emit Rust, Go, Python, TypeScript, or any language. The semantics are simple enough that a register machine (MIPS, WASM) is a valid target.

6. **Generates 95% of the code** — the compiler emits types, transport wiring, test harnesses, CLI entrypoints, progress renderers, and Makefile/CI YAML. The developer writes pure transformation logic (the 5%).

---

## 2. Three Generations of Evidence

### 2.1 gunb.ai (Go, v1)

**What it was**: DAG-based LLM orchestration. Go + protobuf. Ticket/lease execution.

**What it proved**:
- DAGs work for complex workflow orchestration
- Terminal output capture (`CaptureWriter`) is essential for understanding subprocess behavior
- Lease-based resource coordination works for parallel execution

**What failed**:
- All tests handwritten — no structural derivation
- Adding a tool: ~500 lines across 5+ files, all manually coordinated
- Terminal output was captured but not modeled — couldn't see DAG shape during execution

### 2.2 the-gunbai (Rust, v2)

**What it was**: Understanding-driven codegen. Structured documents about external systems that generate integration code. 40+ understandings, 195+ behaviors.

**What it proved**:
- Codegen from structured knowledge scales
- Contract tests CAN be generated from behavior patterns
- The TUI progress system was the standout UX achievement:
  - Four rendering modes: Plain, Inline, TUI (full-screen animated DAG), JSONL
  - Edge pulse animations showing "energy flow" through the graph
  - Wave-based layout (nodes grouped by topological depth)
  - Scatter group progress for parallel tasks (`[2/5]`)

**What the TUI looked like**:
```
gist ─ 3/6 ━━━━━━━━━━━░░░░░░ 50%
  [✓ branch] [✓ files] [◐ render] [○ upload] [○ parse] [○ done]

Expanded:
  Wave 0           Wave 1          Wave 2          Wave 3
╭────────────╮  ╭───────────╮  ╭──────────╮  ╭──────────╮
│ ✓ branch   │──│ ◐ render  │──│ ○ upload │──│ ○ parse  │
│   4ms      │  │   48ms    │  │          │  │          │
╰────────────╯  ╰───────────╯  ╰──────────╯  ╰──────────╯
```

**What failed**:
- Behavior/testing still largely handwritten despite codegen
- No IR — the graph was implicit in runtime orchestration
- The TUI was runtime-only — couldn't visualize a graph before execution

### 2.3 gunbc (Rust, v3 — current)

**What it was**: Full IR with typed DAGs, structural invariants, transport boundaries, proof-obligation testgen.

**What it proved**:
- IR model enables structural guarantees (acyclicity, type safety, cardinality)
- Proof-obligation testgen works (2,334 generated tests, 885 handwritten — 73% generated)
- Transport boundary pattern (prepare/execute/parse) cleanly isolates I/O
- DryRun interception at transport boundaries enables zero-I/O testing
- Content upsert, credential chain, and transport triplet are universal patterns
- Frame-based progress display with pure `build_frame()` function

**What failed**:
- No front-end language — 7,000+ lines of hand-wired graph builders
- Transport types colonized the IR (17 transport modules inside `core/ir/`)
- Discovery was segregated (6 registration islands)
- dag-viz couldn't visualize itself (not in hardcoded workspace DAG list)
- Progress rendering was rebuilt from scratch (lost the-gunbai's TUI quality)
- Resources worked but lifecycle was implicit
- `Value`/`ValueExpr` parallel hierarchies
- Endless refactoring: design docs for fixing the design grew faster than fixes shipped

**The discovery problem**:
```rust
// gunbc-dag/src/workspace/subdags/mod.rs — HARDCODED
pub fn build_workspace_dag() -> Result<Dag<WorkspaceOp>, BuilderError> {
    dag.add_node(makegen::build_makegen_subdag());
    dag.add_node(clippy::build_clippy_lint_all_subdag());
    dag.add_node(deps::build_deps_install_subdag()?);
    // ... manually listed
    // dag-viz is NOT here — dag-viz can't see itself
}
```

### 2.4 Summary Table

| Concern | gunb.ai | the-gunbai | gunbc | DSL (target) |
|---------|---------|------------|-------|---------------|
| Graph authoring | Handwritten Go | Handwritten Rust | Handwritten Rust builders | `.dag` files |
| Tests | All handwritten | Mostly handwritten | 73% generated (testgen) | 95%+ generated |
| Terminal progress | Captured stdout | Full TUI with DAG viz | Frame-based, no TUI | Structural: in the IR |
| Discovery | Manual | Manual | 6 registration islands | Module system |
| Resources | Lease/heartbeat | Implicit | Typed, no lifecycle | First-class lifecycle |
| Target language | Go only | Rust only | Rust only | Language-agnostic |
| Adding a tool | ~500 lines, 5 files | ~300 lines, 3 files | ~200 lines, 2 files | ~20 lines, 1 file |

---

## 3. Design Principles

**P1: Causality is a DAG.** Every workflow is a directed acyclic graph of typed, pure nodes connected by typed edges, with I/O isolated to transport boundaries.

**P2: One type, every level.** A node is either opaque or contains a sub-DAG. Same structure from shell commands up to multi-service pipelines. (From V3 minimal spec.)

**P3: No freeform strings for semantics.** Types are enums. Identifiers are validated newtypes. Extension lanes are declared, not freeform. (From V2 P2.)

**P4: If it validates, wiring is correct.** The compiler proves structural correctness once. Developers test business logic. (From gunbc SPEC.md.)

**P5: Transport is late-bound.** The IR has a generic "external call" concept. Concrete transport (REST, Shell, File) is determined by service annotations and codegen backend. (From gunbc design commitment #7.)

**P6: Progress is a view, never a constraint.** The progress display observes the DAG and infers sections (from SubDag boundaries), groups (from parallel siblings), and waves (from topological depth). It never imposes structure on the DAG or requires authors to declare display metadata. Subprocess output is captured per-node and shown only on failure. Interactive commands declare `@interactive` for passthrough. (Synthesized from gunb.ai's CaptureWriter + the-gunbai's TUI + gunbc's FrameRenderer.)

**P7: Discovery is the filesystem.** Every `.dag` file in the project is auto-discovered. The module graph IS the workspace DAG. No registration macros, no hardcoded lists. (New — fixing gunbc's 6 registration islands.)

**P8: Resources have lifecycle.** Acquire, use, release. The compiler inserts lifecycle nodes, detects conflicts, and generates mock specs. (From V2 P6, extending gunbc's `res:` model.)

**P9: The language is total.** No side effects. No turing-completeness. Every `.dag` file describes structure, never executes it. Compilation always terminates. (From Dhall inspiration.)

**P10: Language-agnostic.** `.dag` files are like `.proto` files. The IR is the contract. Codegen backends are plugins. The semantics (node = pure function, transport = syscall, edge = data flow) map to any execution model.

---

## 4. Language Constructs

Seven constructs:

```
type        — data shapes
resource    — acquirable capabilities with lifecycle
service     — operations with typed I/O and transport annotations
pattern     — reusable DAG shapes with typed slots
journey     — composed flows (main authoring surface)
pipeline    — staged multi-journey workflows
module      — namespace, visibility, discovery metadata
```

### 4.1 Types

```
// Primitives (built-in):
// Unit, Bool, String, Int, Float, Bytes, Json, Secret

// Records
type Credential {
  token: Secret
  scheme: AuthScheme
  expires_at: String?         // ? = optional (zero-or-one)
}

// Sum types (tagged unions)
type AuthScheme
  = Bearer
  | Header { name: String }
  | Basic { username: String }

// Enums
type CloudRuntime = GitHubActions | Metadata | LocalDev
type Platform = Linux | MacOS | Windows

// Collections
// List<T>    — zero or more
// Set<T>     — zero or more, unique
// Map<K, V>  — key-value pairs
```

Design choice: no cardinality algebra. `T` is required-one, `T?` is optional, `List<T>` is zero-or-more. This is sufficient — gunbc's interval math (`Cardinality { min, max }`) was over-engineered for actual usage.

### 4.2 Resources

```
resource Filesystem {
  kind: Capability          // vs Observation
  mode: ReadWrite           // Read | Write | ReadWrite | Exclusive
  acquire {}                // acquisition logic (may be no-op)
  release {}                // release logic (may be no-op)

  capability read {
    input { path: String }
    output { content: String }
    @file(READ, "{path}")
  }

  capability write {
    input { path: String, content: String }
    output { written: Bool }
    @file(WRITE, "{path}")
  }
}

resource Network {
  kind: Capability
  mode: Read
  acquire {}
  release {}
}

resource Clock {
  kind: Observation         // snapshot, no side effect
  mode: Read
  acquire { @pure }
  release {}

  capability now {
    input {}
    output { timestamp: String }
    @pure
  }
}

resource Credential {
  kind: Capability
  mode: Read
  expires: true             // runtime tracks expiry
  acquire {
    // acquisition is itself a journey (see credential_chain pattern)
  }
  release {}
}
```

Lifecycle kinds (from V2 P6):
- `Ephemeral` — created and destroyed within journey scope
- `Persistent` — survives across invocations
- `Borrowed` — referenced but not owned

### 4.3 Services

Declares operations and their transport binding. Inspired by Smithy. Replaces Rust service traits + `MethodMeta` + ops match arms.

```
service gcp.SecretManager {
  operation AccessVersion {
    input {
      project: String
      secret: String
      version: String = "latest"
    }
    output {
      payload: Bytes
      name: String
    }
    @rest(GET, "/v1/projects/{project}/secrets/{secret}/versions/{version}:access")
    @idempotent
    @readonly
    @permissions(["secretmanager.versions.access"])
  }

  operation CreateSecret {
    input { project: String, secret_id: String }
    output { name: String }
    @rest(POST, "/v1/projects/{project}/secrets")
    @permissions(["secretmanager.secrets.create"])
  }
}

service git.Core {
  operation CurrentBranch {
    input {}
    output { branch: String }
    @shell("git rev-parse --abbrev-ref HEAD")
  }

  operation Diff {
    input { base: String, head: String = "HEAD" }
    output { diff: String }
    @shell("git diff {base}...{head}")
  }

  operation LsFiles {
    input {}
    output { files: List<String> }
    @shell("git ls-files")
  }
}

service github.Gist {
  operation Create {
    input {
      description: String
      files: Map<String, String>
      public: Bool = false
    }
    output { url: String, id: String }
    @rest(POST, "https://api.github.com/gists")
    @permissions(["gist"])
  }
}
```

Key: services are pure declarations. Every service call in a journey compiles to a transport triplet (prepare/execute/parse). The author never sees the triplet — the compiler emits it.

### 4.4 Concepts and Providers

Concepts are cross-cutting interfaces. Providers are concrete platforms.

```
concept SecretStore {
  operation Get {
    input { name: String }
    output { value: Secret }
  }
  operation Put {
    input { name: String, value: Secret }
    output { version: String }
  }
}

concept VersionControl {
  operation CurrentRef {
    input {}
    output { ref: String }
  }
  operation Diff {
    input { base: String, head: String }
    output { content: String }
  }
}

provider gcp {
  config {
    project: String
    region: String = "us-central1"
  }
  // services live under provider namespace
  service SecretManager implements SecretStore { ... }
  service IAM { ... }
  service STS { ... }
}

provider aws {
  config {
    account_id: String
    region: String = "us-east-1"
  }
  service SecretsManager implements SecretStore { ... }
}
```

### 4.5 Patterns

Reusable DAG shapes with typed slots. Replaces gunbc's `UpsertBuilder`, `ContentUpsertChain`, `BranchBuilder`, etc.

```
pattern upsert<Check, Create, Resolve> {
  node check: Check -> { exists: Bool }
  node create [when !check.exists]: Create -> { ref: String }
  node resolve: Resolve -> { handle: String }
}

pattern content_upsert {
  input { content: String, path: String }
  uses fs: Filesystem(mode: ReadWrite)

  node read: fs.read(path: path)
  node compare: eq(a: content, b: read.content) -> { changed: Bool }
  node write [when compare.changed]: fs.write(path: path, content: content)

  output { written: Bool = compare.changed }
}

pattern credential_chain {
  input {
    runtime: CloudRuntime
    audience: String
    service_account: String?
    secret_name: String
    project: String
  }
  uses net: Network

  node token = match runtime {
    GitHubActions => github_oidc(audience: audience)
    Metadata     => metadata_oidc(audience: audience)
    LocalDev     => local_auth()
  }

  node access = gcp.STS.Exchange(
    subject_token: token.token,
    audience: audience
  )

  node impersonated = when service_account {
    gcp.IAM.GenerateAccessToken(
      access_token: access.token,
      target_sa: service_account
    )
  } else {
    access
  }

  node secret = gcp.SecretManager.AccessVersion(
    project: project,
    secret: secret_name,
    credential: impersonated
  )

  output { credential: Credential = build_credential(secret.payload) }
}
```

### 4.6 Journeys

The main authoring surface. Composes services, patterns, and other journeys.

```
journey makegen {
  input { registry: ToolRegistry }
  output { written: Bool }

  content = render_makefile(registry: registry)
  result = content_upsert(content: content, path: "Makefile")

  return { written: result.written }
}

journey gist_snapshot {
  input { base_ref: String? }
  output { url: String }

  branch = git.Core.CurrentBranch()
  files = git.Core.LsFiles()

  contents = for file in files.files {
    fs.read(path: file)
  }

  markdown = render_snapshot(files: contents)
  result = gist_upload(markdown: markdown, branch: branch.branch, base_ref: base_ref)

  return { url: result.url }
}
```

Edges are implicit — references create dependencies. The compiler resolves `branch.branch` to an edge from `git.Core.CurrentBranch`'s `branch` output port.

### 4.7 Pipelines

Stages with ordering constraints, parallel groups, and aggregation.

```
pipeline ci {
  stage codegen {
    codegen_check()
  }

  stage generate [after codegen] {
    parallel {
      bootstrap()
      pragma()
      testgen()
    }
  }

  stage build [after generate] {
    cargo_build()
  }

  stage verify [after build] {
    parallel {
      cargo_test()
      clippy()
    }
  }

  stage report [after verify] {
    aggregate(results: [verify.*])
  }
}
```

---

## 5. Module System and Discovery

### 5.1 Filesystem IS the Registry

```
project/
  dag.toml                    # project manifest
  std/                        # standard library (built-in types, resources, patterns)
    types.dag
    resources.dag
    patterns.dag
  cloud/
    concepts.dag              # SecretStore, VersionControl, etc.
    gcp/
      secret_manager.dag
      iam.dag
      sts.dag
    aws/
      secrets_manager.dag
  services/
    git.dag
    github/
      gist.dag
  tools/
    clippy.dag
    gist.dag
    deps.dag
  pipelines/
    ci.dag
    build.dag
  meta/                       # self-referential tooling
    dag_viz.dag
    makegen.dag
    testgen.dag
```

### 5.2 Project Manifest

```toml
[project]
name = "gunbc"
version = "0.1.0"

[sources]
paths = ["std/", "cloud/", "services/", "tools/", "pipelines/", "meta/"]

[codegen]
backends = ["rust"]
output = "target/generated/"

[progress]
default_mode = "inline"   # plain | inline | tui | jsonl
```

### 5.3 Discovery Rules

1. Every `.dag` file under `paths` is parsed and added to the module graph.
2. Module path = filesystem path: `cloud/gcp/secret_manager.dag` → `cloud.gcp.secret_manager`.
3. Imports are resolved against the module graph.
4. The module graph IS the workspace DAG. No separate `build_workspace_dag()`.
5. `meta/dag_viz.dag` can reference itself because it's in the same module graph.

### 5.4 What This Replaces

| gunbc system | Replaced by |
|---|---|
| `#[tool_target]` proc macro | filesystem discovery |
| `#[testgen_target]` proc macro | every journey has test obligations |
| `build_workspace_dag()` hardcoded list | module graph |
| `ToolRegistry::default_registry()` | project manifest |
| `inventory` crate | eliminated |
| `derive_tool_defs()` | eliminated |

---

## 6. Terminal Progress Model

### 6.1 Core Invariant: Progress is a View, Not a Constraint

**The progress display must never impose structure on the DAG.** Groups, waves, and sections are rendering decisions derived from DAG topology — they are not metadata that DAG authors must provide or that constrains how implementations are structured.

In gunb.ai, groups were manually specified (`ProgressOptions.Groups`). That worked, but it meant every DAG had to think about display. In the DSL, the progress renderer observes the DAG and infers everything:

- **SubDag boundaries → section headers** (e.g., `› Authentication`, `› Fetching Secrets`)
- **Parallel siblings → grouped counters** (e.g., `[2/5]`)
- **Topological depth → wave columns** (for TUI layout)
- **Loop expansions → scatter groups** (e.g., `read files [8/8]`)

The renderer CAN create arbitrary groupings for visualization (collapsing parallel nodes, grouping by SubDag parent), but it MUST NOT require the DAG to declare them.

### 6.2 Subprocess Output Capture

This is a first-class concern, not an afterthought. Learned from gunb.ai's `CaptureWriter`.

**The problem**: When a transport node executes a shell command, its stdout/stderr must not leak into the progress display. Without capture, output from concurrent nodes interleaves with spinner animations, producing garbage.

**The solution**: Every transport execute node gets a per-node output buffer (like gunb.ai's `CaptureWriter`):

```
Per-node execution:
  1. Allocate CaptureBuffer for this node
  2. Redirect subprocess stdout/stderr → CaptureBuffer
  3. Execute the command
  4. On success: buffer is discarded (progress shows ✓)
  5. On failure: buffer contents shown in error box
  6. On passthrough: buffer is bypassed (see 6.3)
```

This is modeled in the IR. Transport nodes have an output capture mode:

```
type CaptureMode
  = Captured           // default: stdout/stderr → buffer, shown only on error
  | Passthrough        // interactive: stdout/stderr → terminal directly
  | Streamed           // long-running: stdout/stderr → shown live, line by line
```

### 6.3 Passthrough Mode (Interactive Commands)

Some commands need direct terminal access — OAuth flows, `gcloud auth login`, password prompts. The DAG must declare this.

```
// In a service declaration:
service gcloud.Auth {
  operation Login {
    input { update_adc: Bool = true }
    output { ok: Bool }
    @shell("gcloud auth login --update-adc")
    @interactive                         // ← marks as passthrough
  }
}

// In a journey:
journey authenticate {
  // ...
  node login [when needs_reauth]: gcloud.Auth.Login()
  // During execution:
  //   1. Progress display pauses (clears spinner line)
  //   2. gcloud auth login runs with stdin/stdout/stderr inherited
  //   3. User sees the OAuth URL, pastes code, etc.
  //   4. Command completes
  //   5. Progress display resumes
}
```

The `@interactive` annotation compiles to `CaptureMode::Passthrough` on the transport execute node. The progress renderer:
1. Clears the current progress line
2. Lets the subprocess own the terminal
3. Resumes progress display when the node completes

This is exactly what gunb.ai did for OAuth — `cmd.Stdout = os.Stdout` instead of `cmd.Stdout = captureWriter`.

### 6.4 What gunb.ai's `make login` Looks Like in the DSL

The terminal output the user showed:

```
› Authentication
   ✓ clear-cache
   ✓ detect-env
   ✓ check-account
   ✓ check-adc
   ✓ check-tokens
   ✓ configure-gcloud

   Sign in with your @gunb.ai account
   Go to the following link in your browser...
   (interactive OAuth flow — subprocess owns terminal)

› Fetching Secrets
   ✓ fetch-secrets
   ⠧ sync-remote-home
   ✓ write-bazelrc
   ✓ clear-prompt-cache
   ○ export-shell-env
   ○ Login complete (as briansrls@gunb.ai)
```

This emerges from a journey like:

```
journey login {
  output { ok: Bool }

  // These nodes form a sequential chain → they render as a flat list
  // The journey name "login" doesn't appear; the SubDag names do.

  auth = authenticate()     // SubDag → becomes "› Authentication" section
  secrets = fetch_secrets(  // SubDag → becomes "› Fetching Secrets" section
    credential: auth.credential
  )

  return { ok: secrets.ok }
}

journey authenticate {
  output { credential: Credential }

  clear_cache = cache.Clear()
  env = detect_environment()
  account = check_account()
  adc = check_adc()
  tokens = check_tokens(adc: adc)

  // Interactive step — @interactive on the service operation
  node login [when tokens.needs_reauth]: gcloud.Auth.Login()

  configure = configure_gcloud(tokens: tokens)
  return { credential: configure.credential }
}
```

**How the sections emerge**:
1. `login` journey calls `authenticate()` and `fetch_secrets()` — both are journey calls that expand to SubDags.
2. The progress renderer sees two SubDag nodes at the top level.
3. SubDag boundaries become `›` section headers.
4. Nodes inside each SubDag become the indented status lines.
5. The `@interactive` `gcloud.Auth.Login()` triggers passthrough — its output appears between the progress lines.

No manual `ProgressGroup` declarations. No `Groups: []dag.ProgressGroup{...}`. The structure IS the DAG.

### 6.5 ProgressManifest (Compiler Output)

The compiler derives a manifest from the DAG topology. The manifest is a description of what EXISTS, not a prescription of how to display:

```
type ProgressManifest {
  // Topology (what exists)
  total_nodes: Int
  topology: List<TopologyNode>        // every node with its depth and parent

  // Labels (human-readable, from DSL identifiers)
  labels: Map<NodeId, String>

  // Structural features (for renderers to use as they see fit)
  subdag_boundaries: List<SubDagBoundary>  // journey/pattern calls
  parallel_groups: List<ParallelGroup>     // siblings at same depth
  scatter_points: List<NodeId>             // loop expansion points
  interactive_nodes: List<NodeId>          // @interactive transport nodes
  capture_modes: Map<NodeId, CaptureMode>  // per-node output handling

  // Resource context
  resources: Map<NodeId, List<ResourceUsage>>
}

type TopologyNode {
  id: NodeId
  depth: Int                              // topological depth (wave)
  parent: NodeId?                         // SubDag parent, if any
}

type SubDagBoundary {
  node_id: NodeId                         // the SubDag node in the parent
  label: String                           // journey/pattern name
  inner_nodes: List<NodeId>               // nodes inside the SubDag
}

type ParallelGroup {
  nodes: List<NodeId>                     // siblings with same dependencies
  depth: Int
}
```

**Key difference from gunb.ai**: The manifest describes topology. Renderers decide how to present it:

- The `inline` renderer might collapse SubDags into single chips: `[✓ auth] [◐ secrets]`
- The `plain` renderer might expand SubDags into sections: `› Authentication\n   ✓ clear-cache`
- The `tui` renderer might show SubDags as expandable boxes
- All three read the SAME manifest — they just make different rendering choices

### 6.6 Rendering Modes

| Mode | Description | When |
|------|------------|------|
| `plain` | Sections + status lines (gunb.ai style) | CI, non-TTY |
| `inline` | Compact bar + chips (the-gunbai style) | Default TTY |
| `tui` | Full DAG with edge pulses (the-gunbai style) | Explicit opt-in |
| `jsonl` | Structured event stream | Machine consumption |

**Plain** — gunb.ai style with sections:
```
› Authentication
   ✓ clear-cache (1ms)
   ✓ detect-env (2ms)
   ✓ check-account (50ms)
   ✓ check-adc (3ms)
   ✓ check-tokens (5ms)
   ✓ configure-gcloud (10ms)

› Fetching Secrets
   ✓ fetch-secrets (1.2s)
   ⠧ sync-remote-home
   ○ write-bazelrc
   ○ clear-prompt-cache
   ○ export-shell-env
```

**Inline** — the-gunbai compact style:
```
login ─ 8/12 ━━━━━━━━━━━━░░░░ 67% [✓ auth 6/6] [◐ secrets 2/6]
```

**TUI** — full DAG visualization:
```
╭─ auth ─────────────────────╮  ╭─ secrets ─────────────╮
│ ✓ clear-cache     ✓ detect │──│ ✓ fetch     ⠧ sync   │
│ ✓ check-account   ✓ adc   │  │ ○ bazelrc   ○ cache  │
│ ✓ tokens   ✓ configure    │  │ ○ export              │
╰────────────────────────────╯  ╰───────────────────────╯
```

### 6.7 Failure Output

When a node fails, the CaptureBuffer contents are shown in an error box (gunb.ai pattern):

```
› Fetching Secrets
   ✓ fetch-secrets (1.2s)
   ✖ sync-remote-home (3.4s)

   ┌─ Error: sync-remote-home ────────────────────────┐
   │ gsutil rsync returned exit code 1                 │
   │                                                   │
   │ stderr:                                           │
   │   CommandException: No URLs matched: gs://...     │
   │   CommandException: 1 file/object could not be    │
   │   transferred.                                    │
   └───────────────────────────────────────────────────┘

   ○ write-bazelrc (skipped: dependency failed)
   ○ clear-prompt-cache (skipped)
   ○ export-shell-env (skipped)
```

The captured stderr appears ONLY on failure. On success, it's silently discarded. This prevents the double-printing problem where subprocess output would interleave with progress indicators.

### 6.8 Visual Design Specification (from gunb.ai)

The visual design is lifted directly from gunb.ai. These are exact values.

#### Color Palette (256-color ANSI)

```
SemanticColor   ANSI Code           Usage
─────────────────────────────────────────────────
Success         \033[38;5;34m       ✓ completed nodes, success messages
Active          \033[38;5;208m      ⠧ spinner, running indicators
Error           \033[38;5;196m      ✖ failed nodes, error boxes
Info            \033[38;5;39m       ℹ info boxes, URLs
Dim             \033[2m             ○ pending/skipped, captured output
Calm            \033[38;5;75m       box borders (preamble, info)
Reset           \033[0m             reset all styling
```

#### Spinner (Braille, 10 frames, 80ms tick)

```
Frame:    ⠋  ⠙  ⠹  ⠸  ⠼  ⠴  ⠦  ⠧  ⠇  ⠏
Index:    0   1   2   3   4   5   6   7   8   9
Color:    Active (orange)
Interval: 80ms (configurable via env)
```

ASCII fallback: `| / - \ | / - \ | /`

#### Status Icons

```
State        Emoji    Unicode   ASCII    Color
──────────────────────────────────────────────────
Success      ✅        ✓         OK       Success (green)
Running      🔄        ◐         [~]      Active (orange)
Pending      ⏳        ○         [ ]      Dim
Failed       ❌        ✖         FAIL     Error (red)
Skipped      ⏭️        ◌         [-]      Dim
```

Terminal rendering defaults to Unicode tier. Emoji tier is opt-in.

#### Section Format

```
{spinner} › {SectionName} [{completed}/{total}] ({running_task})

Example:
⠋ › Authentication [4/6] (check-tokens)
```

- Section marker: `›` (U+203A, "single right-pointing angle quotation mark")
- Counts: `[completed/total]` only shown for incomplete groups
- Running task: in parentheses, dimmed
- Multiple running: comma-separated `(task1, task2)`

#### Node Lines (inside sections)

```
   {icon} {node_name} ({duration})

Examples:
   ✓ clear-cache (1ms)
   ✓ detect-env (2ms)
   ⠧ check-tokens
   ○ configure-gcloud
   ✖ sync-remote-home (3.4s)
```

- Indentation: 3 spaces
- Duration: shown only after completion
- Duration format: `1ms`, `50ms`, `0.5s`, `3.4s`, `1m30s`

#### Box Drawing

```
Error box (open-right):          Preamble box (closed):
╭─ Error: node-name ──────      ╭─ gist ──────────────────╮
│ gsutil rsync returned 1       │ Create GitHub Gist       │
│                               │   repo_path: .           │
│ stderr:                       │   mode: snapshot         │
│   CommandException: ...       ╰──────────────────────────╯
╰──────────────────────────

Characters: ╭ ╮ ╰ ╯ │ ─
Default width: 60, min: 40
Error border: Error (red)
Preamble border: Calm (soft blue)
Info border: Info (cyan)
Content: Dim
```

#### Completion Animals (random success emoji)

```go
["🐶", "🐱", "🐭", "🐹", "🐰", "🦊", "🐻", "🐼",
 "🐨", "🐯", "🦁", "🐮", "🐷", "🐸", "🐵", "🦉"]
```

Displayed on DAG completion: `🐶 › Heal complete (3.4s)`

#### Prompt Status Icons (shell integration)

```
Icon     Meaning              Active    Inactive
────────────────────────────────────────────────
Auth     Authenticated?       🔐        🔓
Build    Remote build?        ⚡        🏠
AI       AI keys available?   🤖        ⏳
```

Format: `{auth} {build} {ai}  {user}@{host}:{path} ({branch})$`
Example: `🔐 ⚡ 🤖  vscode ~ (main)$`

### 6.9 Browser Launching / Platform Modeling

Interactive commands may need to open a browser (OAuth flows, HTML previews). This is modeled as a platform-aware capability.

```
// Platform detection (compile-time + runtime)
type Platform = Linux | MacOS | Windows | WSL

// Browser opening (platform-specific)
resource Browser {
  kind: Capability
  mode: Read
  acquire {
    detect_platform() -> match {
      MacOS   => @shell("open")
      Windows => @shell("cmd /c start")
      WSL     => @shell("wslview")        // opens in Windows host browser
      Linux   => @shell("xdg-open")       // requires DISPLAY
    }
  }

  capability open_url {
    input { url: String }
    output { ok: Bool }
    @interactive                           // passthrough mode
  }
}
```

Platform detection hierarchy:
1. Check `SSH_TTY` env → if set, no browser available (remote session)
2. Check `BROWSER` env → if set, use it (VS Code/Cursor devcontainer support)
3. macOS → `open`
4. Windows → `cmd /c start`
5. Linux + `WSL_DISTRO_NAME` env → `wslview` (WSL, no DISPLAY needed)
6. Linux + `DISPLAY` env → `xdg-open`
7. Linux headless → error: "No browser available"

WSL special handling: convert relative paths to absolute before calling `wslview` (Windows host needs absolute paths from WSL filesystem).

### 6.10 Harvestability Assessment

**Terminal code from gunbc: harvest.** The symbol system, box drawing, frame writing, and terminal detection total ~2,271 lines and are 95% standalone (no DAG dependencies). Only the SubDag builders (~150 lines) are tangled with IR types. Strategy:

1. Copy `symbols.rs` core (~750 lines) — remove SubDag builders
2. Copy `render_ir.rs` core (~580 lines) — remove IR trait stubs
3. Copy `terminal.rs` (~200 lines) — replace `Viewport`/`Tier` with local copies
4. Copy `box_draw.rs` (~427 lines) — standalone
5. Copy `frame_write.rs` (~314 lines) — standalone

These become a single `terminal` crate in the new repo. The gunb.ai color palette and icon vocabulary are already matched in gunbc's symbol system (same ANSI codes, same spinner frames, same braille characters). The gunbc code IS the gunb.ai design, ported to Rust.

**TUI code from the-gunbai: harvest separately.** The ratatui-based TUI (edge pulses, DAG layout, widgets) is independent of the inline/plain rendering. Harvest it as an optional feature behind a `tui` cargo feature flag.

**Frame building from gunbc: harvest.** The pure `build_frame()` function (~300 lines) computes frames from progress state. It depends on the manifest (which we'll derive from the compiler) but the frame-building logic itself is reusable.

### 6.11 How Progress Compiles

The progress model touches three compiler phases:

1. **Lower**: Transport nodes get `CaptureMode` based on `@interactive` annotations.
2. **Derive**: `ProgressManifest` computed from lowered DAG topology (depths, SubDag boundaries, parallel groups, scatter points).
3. **Emit**: Codegen backend emits:
   - `CaptureBuffer` allocation per transport node
   - Progress observer trait implementation
   - Frame builder that reads the manifest
   - Renderer selection (plain/inline/tui/jsonl)

The runtime needs only:
- The manifest (static, from compiler)
- Node state transitions (Pending → Running → Succeeded/Failed/Skipped)
- CaptureBuffer contents (for error display)
- Interactive node detection (for passthrough pause/resume)

---

## 7. Resource Model

### 7.1 Declaration

```
journey write_config {
  uses fs: Filesystem(mode: Write)
  uses clock: Clock

  timestamp = clock.now()
  content = render_config(timestamp: timestamp)
  fs.write(path: "config.toml", content: content)
}
```

### 7.2 What the Compiler Does

1. **Inserts acquisition nodes** at DAG boundaries (like gunbc's `FsEnv`, `ClockEnv`).
2. **Threads resources** through edges to consuming nodes (like gunbc's `res:*` ports).
3. **Detects conflicts** — parallel Write+Write on same resource = compile error.
4. **Generates mock specs** — DryRun substitutes resources with mocks.
5. **Derives test obligations** — Bucket D (Resource Hygiene) from testgen.
6. **Tracks lifecycle** — acquire before first use, release after last use.

### 7.3 What This Replaces

```rust
// gunbc today: manual environment nodes + resource wiring
let fs_env = builder.add_root_node(Node::opaque(
    "fs_env", vec![], vec![port("FilesystemHandle", "FilesystemHandle")],
    FsEnv::new(Scope::Write),
));
// ... later, for EVERY node that needs filesystem:
builder.add_edge(fs_env.out("FilesystemHandle"), node.in_port("res:file:Makefile"));
```

```
// DSL: declared once, compiler handles the rest
journey makegen {
  uses fs: Filesystem(mode: Write)
  // fs.write(...) automatically threads the resource
}
```

---

## 8. Compiler Pipeline

```
.dag files (filesystem)
   │
   ▼
[Discover] ──→ Module graph (all .dag files in project)
   │
   ▼
[Parse] ─────→ AST per file (concrete syntax tree)
   │
   ▼
[Resolve] ───→ Resolved AST (imports linked, names resolved against module graph)
   │
   ▼
[TypeCheck] ──→ Typed AST (expressions typed, resource requirements validated)
   │
   ▼
[PatternExpand] → PatternIR (patterns → sub-DAG templates,
   │                          resources → acquire/release nodes)
   ▼
[Lower] ─────→ GraphIR (Node / Dag / Port / Edge)
   │             - service calls → transport triplets
   │             - implicit edges → explicit Edge values
   │             - resource uses → acquire/release lifecycle nodes
   │             - for → LoopBuilder nodes
   │             - match → BranchBuilder nodes
   │             - when → guarded ports
   ▼
[Validate] ──→ Validated GraphIR (SPEC.md invariants + resource conflicts)
   │
   ▼
[Derive] ────→ ProgressManifest + TestObligations + ToolMetadata
   │
   ▼
[Emit] ──────→ Per-backend codegen
                 - Type definitions
                 - Node stubs (pure function signatures for developer to fill)
                 - Transport wiring (HTTP client, shell exec, file I/O)
                 - Test harness (4-bucket testgen)
                 - CLI entrypoint (args from DAG entrypoints)
                 - Progress renderer (manifest-driven)
                 - Makefile / CI YAML
```

---

## 9. Multi-Target Emission

### 9.1 IR Semantics Are Minimal

| IR Concept | Rust | Go | Python | MIPS |
|---|---|---|---|---|
| Node | `fn` | `func` | `def` | `jal label` |
| Edge | variable | variable | variable | register |
| Transport | `reqwest`/`Command` | `net.http`/`exec` | `requests`/`subprocess` | `syscall` |
| Guard | `if` | `if` | `if` | `beq` |
| SubDag | `fn` (inlined) | `func` (inlined) | `def` (inlined) | `jal` (nested) |
| Loop | `for .. in` | `for .. range` | `for .. in` | loop/`beq` |
| Topo schedule | sequential | goroutine pool | `asyncio.gather` | instruction order |

### 9.2 Backend Interface

Each codegen backend implements:

```
trait CodegenBackend {
  fn emit_type(ty: &TypeDef) -> String
  fn emit_node_stub(node: &Node) -> String      // pure function signature
  fn emit_transport(spec: &TransportSpec) -> String
  fn emit_test(obligation: &TestObligation) -> String
  fn emit_cli(entrypoints: &[Port]) -> String
  fn emit_progress(manifest: &ProgressManifest) -> String
}
```

### 9.3 The 95% / 5% Split

The compiler generates:
- Type definitions (structs, enums)
- Transport wiring (HTTP client setup, shell exec, file I/O)
- Test harnesses (DryRun completion, transport interception, scenario coverage, resource hygiene)
- CLI entrypoints (argument parsing from DAG entrypoint ports)
- Progress renderers (manifest-driven frame building)
- Makefile / CI YAML (from module graph)

The developer writes:
- Pure transformation logic inside node bodies (the actual business logic)
- Custom parsers for service responses (when `@rest` / `@shell` aren't sufficient)

---

## 10. What to Harvest

### From gunb.ai
- **CaptureWriter** pattern: per-node output buffer, subprocess stdout/stderr captured not printed, shown only on failure in error boxes. Thread-safe (`sync.Mutex` + `bytes.Buffer`). This is THE solution to double-printing.
- **Passthrough mode**: Interactive commands (`gcloud auth login`, OAuth) bypass capture, inherit terminal stdin/stdout/stderr directly. Progress display pauses during passthrough.
- **Section rendering**: `›` section headers from SubDag boundaries (in gunb.ai these were manually grouped via `ProgressOptions.Groups` — we make them emergent from DAG structure)
- **Error boxes**: Failed node output displayed in bordered box with captured stderr. Successful nodes silently discard captured output.
- **Preamble box**: Tool header with name, description, args displayed before execution
- **Emoji prompt icons**: Status indicators exported as shell variables (`GUNB_AUTH_ICON`, etc.) — shows system state at a glance
- Lease/heartbeat execution model (concept for distributed execution)

### From the-gunbai
- TUI progress system (ratatui + crossterm): edge pulses, wave layout, scatter groups
- Progress state machine (`ProgressState`, `NodeProgressState`, `ProgressCounts`)
- Spinner system (deterministic tick-driven, braille frames)
- JSONL event streaming (schema: `gunbai.progress.v1`)
- Inline renderer (compact progress bar + box-drawing DAG)

### From gunbc
- `Node<T>`, `Dag<T>`, `Port`, `Edge` core types (proven correct)
- Pattern builders: `UpsertBuilder`, `BranchBuilder`, `LoopBuilder`, `ContentUpsertChain`
- Execution engine: lowering, topo sort, DryRun
- Testgen obligation model (4 buckets, anti-tautology rule, `ProofObligation`, `DischargeStatus`)
- Transport executor (REST, Shell, File, TCP)
- Resource conflict detection algorithm
- Frame-based display (pure `build_frame()`, `FrameRenderer<M>` trait)
- `OutputMedium` trait hierarchy (AnsiText, PlainText, HtmlText)
- `SemanticColor` / `SymbolId` tier-based symbol resolution

### Redesign
- Decouple `TransportRequest`/`TransportResponse` from `Value` enum
- Eliminate `ValueExpr` (codegen works from IR + types)
- Move transport out of `core/ir/src/transport/` into transport crate
- Replace all registration macros with filesystem discovery
- Unify the-gunbai's TUI + gunbc's FrameRenderer into manifest-driven system

---

## 11. Phasing

### Phase 1: Language Core + Module Discovery + Progress Manifest

**Target**: Express `makegen` end-to-end.

```
tools/makegen.dag → discover → parse → typecheck → lower → validate
  → derive ProgressManifest → emit Rust → execute with inline progress
```

Proves: parser, types, patterns (content_upsert), discovery, progress manifest, Rust backend.

### Phase 2: Services + Resources + Cloud Modeling

**Target**: Express `acquire_gcp_secret`.

- Provider-qualified service calls (`gcp.SecretManager.AccessVersion`)
- `@rest` → transport generation
- `resource Credential` with lifecycle
- `match` for runtime branching, `when` for guards

### Phase 3: Composition + TUI Progress

**Target**: Express `gist_snapshot`.

- `for` loops, journey composition, SubDag expansion
- TUI progress renderer driven by ProgressManifest
- Static DAG visualization (before execution)

### Phase 4: Pipelines + Second Codegen Backend

**Target**: Express CI pipeline. Add Go or Python backend.

- Pipeline stages, parallel execution, aggregation
- Same `.dag` files → different language output

---

# Appendix A: Content Upsert (Makegen)

The simplest complete graph in gunbc. The canonical "hello world" for the DSL.

## A.1 Today: Rust (gunbc)

### Graph builder (`gunbc-dag/src/makegen/graph.rs` — 137 lines)

```rust
pub fn build_makegen_graph() -> Dag<MakegenGraphOp> {
    let mut builder: DagBuilder<MakegenGraphOp> = DagBuilder::new();

    // Root: filesystem handle
    let fs_env = builder
        .add_root_node(Node::opaque(
            "fs_env", vec![],
            vec![port("FilesystemHandle", "FilesystemHandle")],
            MakegenGraphOp::FsEnv(FsEnv::new(Scope::Write)),
        )).expect("fs_env");

    // Root: load tool registry
    let load_registry = builder
        .add_root_node(Node::opaque(
            "load_registry", vec![],
            vec![
                port("tool_count", "Int"),
                port("tool_names", "NonEmptyStringList"),
                port("registry", "Json"),
            ],
            MakegenGraphOp::Domain(MakegenOp::LoadRegistry),
        )).expect("load_registry");

    // Pure: render Makefile content
    let render_makefile = builder
        .add_node_after(Node::opaque(
            "render_makefile",
            vec![port("registry", "Json")],
            vec![port("makefile_content", "String")],
            MakegenGraphOp::Domain(MakegenOp::RenderMakefile),
        ), &load_registry).expect("render_makefile");

    builder.add_edge(
        load_registry.out("registry"),
        render_makefile.in_port("registry"),
    ).expect("load_registry.registry -> render_makefile.registry");

    // Content upsert chain (5 nodes, 8 edges — added by helper)
    add_content_upsert_chain(
        &mut builder,
        "makegen",
        &render_makefile,
        "makefile_content",
        &fs_env,
        "Makefile",
    );

    builder.build().expect("makegen_graph")
}
```

Plus: operation enum (25 lines), `Executable` impl (40 lines), operation implementations (80+ lines), testgen registration (15 lines), tool registration (15 lines).

**Total: ~200+ lines across 3 files.**

### Resulting IR (8 nodes, 10 edges)

```
Dag {
  nodes: [
    Node { id: "fs_env",                    body: Opaque(FsEnv) }
    Node { id: "load_registry",             body: Opaque(LoadRegistry) }
    Node { id: "render_makefile",           body: Opaque(RenderMakefile) }
    Node { id: "prepare_read_makegen",      body: Opaque(PrepareFileRead) }
    Node { id: "execute_read_makegen",      body: Opaque(Transport::Execute) }
    Node { id: "compare_makegen_content",   body: Opaque(Blob::CompareContent) }
    Node { id: "prepare_write_makegen",     body: Opaque(PrepareFileWrite) }
    Node { id: "execute_makegen_transport", body: Opaque(Transport::Execute) }
  ]
  edges: [
    load_registry.registry         → render_makefile.registry
    render_makefile.makefile_content → compare_makegen_content.expected_content
    render_makefile.makefile_content → prepare_write_makegen.content
    prepare_read_makegen.request    → execute_read_makegen.request
    prepare_read_makegen.skip       → execute_read_makegen.skip
    execute_read_makegen.response   → compare_makegen_content.response
    compare_makegen_content.skip    → execute_makegen_transport.skip
    compare_makegen_content.skip_reason → execute_makegen_transport.skip_reason
    prepare_write_makegen.request   → execute_makegen_transport.request
    fs_env.FilesystemHandle         → execute_read_makegen.res:file:Makefile
    fs_env.FilesystemHandle         → execute_makegen_transport.res:file:Makefile
  ]
}
```

### Generated tests (from testgen — excerpt)

```rust
#[test]
fn test_dryrun_completion() {
    let dag = build_makegen_graph();
    let log = execute_with_mode(&dag, ExecutionMode::DryRun(mock_spec().to_boundary_mocks()))
        .expect("DryRun should complete");
    assert!(!log.entries.is_empty());
}

#[test]
fn test_transport_interception() {
    let dag = build_makegen_graph();
    let result = assert_boundary_mockable(&dag, mock_spec().to_boundary_mocks());
    assert!(result.is_ok());
    assert!(result.boundary_nodes.iter().any(|n| n == "execute_read_makegen"));
    assert!(result.boundary_nodes.iter().any(|n| n == "execute_makegen_transport"));
}
```

## A.2 DSL

### `tools/makegen.dag` (5 lines of authoring)

```
module tools.makegen

import std.patterns { content_upsert }

journey makegen {
  input { registry: ToolRegistry }
  output { written: Bool }

  content = render_makefile(registry: registry)
  result = content_upsert(content: content, path: "Makefile")

  return { written: result.written }
}
```

## A.3 Compiler Output

### Resulting IR (identical structure — 8 nodes, 10 edges)

The compiler produces the same IR as the hand-wired version. The key differences:

1. `content_upsert` pattern expansion creates the 5-node chain automatically.
2. `render_makefile(registry: registry)` becomes a service call (pure, no transport).
3. `uses fs: Filesystem` is inferred from `content_upsert`'s declaration — the `fs_env` node is inserted automatically.
4. Edges are derived from references: `content_upsert(content: content, ...)` creates the edge from `render_makefile.makefile_content` to the upsert chain.

### Generated test obligations

```
Bucket A (Execution Semantics):
  - DryRunCompletion: full workflow
  - TransportInterceptable: execute_read_makegen
  - TransportInterceptable: execute_makegen_transport

Bucket B (Contract Obligations):
  - NodeContractCompliance: render_makefile

Bucket C (Scenario Coverage):
  - AllTransportsSucceed
  - SingleTransportFailure: execute_read_makegen
  - SingleTransportFailure: execute_makegen_transport
  - GuardBranchCoverage: execute_makegen_transport (skip guard)

Bucket D (Resource Hygiene):
  - TransportResourceDeclared: execute_read_makegen
  - TransportResourceDeclared: execute_makegen_transport
  - ResourceInputConnected: execute_read_makegen.res:file:Makefile
  - ResourceInputConnected: execute_makegen_transport.res:file:Makefile
```

### ProgressManifest

```
ProgressManifest {
  total_nodes: 8
  waves: [
    Wave { depth: 0, nodes: ["fs_env", "load_registry"] }
    Wave { depth: 1, nodes: ["render_makefile", "prepare_read_makegen"] }
    Wave { depth: 2, nodes: ["execute_read_makegen"] }
    Wave { depth: 3, nodes: ["compare_makegen_content", "prepare_write_makegen"] }
    Wave { depth: 4, nodes: ["execute_makegen_transport"] }
  ]
  labels: {
    "fs_env": "fs",
    "load_registry": "load",
    "render_makefile": "render",
    "prepare_read_makegen": "read (prepare)",
    "execute_read_makegen": "read",
    "compare_makegen_content": "compare",
    "prepare_write_makegen": "write (prepare)",
    "execute_makegen_transport": "write"
  }
  expand_points: []
  groups: []
}
```

### Terminal output (inline mode)

```
makegen ─ 4/4 ━━━━━━━━━━━━━━━━ 100% [✓ load] [✓ render] [✓ compare] [⊘ write]
```

(write skipped because content unchanged — the `[when compare.changed]` guard fired)

---

# Appendix B: Cloud Credential Acquisition (GCP)

The most complex graph in gunbc. The canonical stress test for the DSL.

## B.1 Today: Rust (gunbc)

### `lib/gcp-ops/src/graph.rs` — 1,688 lines (excerpt: first transport triplet)

```rust
// GitHub OIDC: prepare → execute → parse (one of ~8 such triplets)
let prepare = builder
    .add_root_node(Node::opaque(
        "prepare_github_oidc",
        vec![
            port("audience", "String"),
            optional("request_url", "OptionalString"),
            optional("request_token", "OptionalString"),
        ],
        vec![port("request", "TransportRequest"), port("skip", "Bool")],
        GcpSecretManagerGraphOp::Gcp(GcpOps::PrepareGitHubOidcRequest),
    )).expect("prepare_github_oidc");

let execute = builder
    .add_node_after(Node::opaque(
        "execute_github_oidc",
        vec![
            port("request", "TransportRequest"),
            port("skip", "Bool"),
            resource("api:network", "NetworkHandle", AccessMode::Read),
        ],
        vec![port("response", "TransportResponse")],
        GcpSecretManagerGraphOp::Transport(TransportOps::Execute),
    ), &prepare).expect("execute_github_oidc");

let parse = builder
    .add_node_after(Node::opaque(
        "parse_github_oidc",
        vec![port("response", "TransportResponse")],
        vec![port("subject_token", "String")],
        GcpSecretManagerGraphOp::Gcp(GcpOps::ParseGitHubOidcResponse),
    ), &execute).expect("parse_github_oidc");

builder.add_edge(prepare.out("request"), execute.in_port("request"));
builder.add_edge(prepare.out("skip"), execute.in_port("skip"));
builder.add_edge(net_env.out(NetEnv::PORT), execute.in_port(RESOURCE_API_NETWORK));
builder.add_edge(execute.out("response"), parse.in_port("response"));
// ... repeat for STS exchange, impersonation, secret access (~30 lines each)
```

**Plus**: `ops.rs` (2,077 lines), service traits (180 lines each), generated tests (157K chars).

**Total: ~4,000+ lines across 6+ files.**

### Service trait (`lib/gcp-ops/src/services/secret_manager.rs` — excerpt)

```rust
pub trait SecretManagerService {
    fn access_secret_version(&self, project: &str, secret: &str, version: &str) -> RestRequest;
    fn get_secret(&self, project: &str, secret: &str) -> RestRequest;
    fn create_secret(&self, project: &str, secret_id: &str) -> RestRequest;
    fn add_secret_version(&self, project: &str, secret: &str, payload_base64: &str) -> RestRequest;
}

pub const ACCESS_SECRET_VERSION_META: MethodMeta = MethodMeta {
    endpoint: "/v1/projects/{project}/secrets/{secret}/versions/{version}:access",
    http_method: HttpMethod::Get,
    idempotent: true,
    read_only: true,
    permissions: &["secretmanager.versions.access"],
    service: "secretmanager",
};
```

## B.2 DSL

### `cloud/gcp/secret_manager.dag`

```
module cloud.gcp.secret_manager

service gcp.SecretManager {
  operation AccessVersion {
    input { project: String, secret: String, version: String = "latest" }
    output { payload: Bytes, name: String }
    @rest(GET, "/v1/projects/{project}/secrets/{secret}/versions/{version}:access")
    @idempotent @readonly
    @permissions(["secretmanager.versions.access"])
  }

  operation CreateSecret {
    input { project: String, secret_id: String }
    output { name: String }
    @rest(POST, "/v1/projects/{project}/secrets")
    @permissions(["secretmanager.secrets.create"])
  }

  operation AddVersion {
    input { secret_name: String, payload: Bytes }
    output { name: String }
    @rest(POST, "/v1/{secret_name}:addVersion")
    @permissions(["secretmanager.versions.add"])
  }
}
```

### `cloud/gcp/credential.dag`

```
module cloud.gcp.credential

import cloud.gcp.secret_manager
import cloud.gcp.iam
import cloud.gcp.sts
import std.patterns { credential_chain }

journey acquire_gcp_secret {
  input {
    runtime: CloudRuntime
    project: String
    secret_name: String
    audience: String = "sigstore"
    service_account: String?
  }
  output { credential: Credential }

  cred = credential_chain(
    runtime: runtime,
    audience: audience,
    service_account: service_account,
    secret_name: secret_name,
    project: project
  )

  return { credential: cred.credential }
}
```

**Total: ~50 lines across 2 files** (vs. 4,000+ lines across 6+ files).

## B.3 What the Compiler Does

1. `gcp.SecretManager.AccessVersion(...)` expands to a transport triplet:
   - `prepare_access_version` (builds `RestRequest` from `@rest` annotation)
   - `execute_access_version` (transport boundary)
   - `parse_access_version` (extracts `payload`, `name` from response)

2. `credential_chain` pattern expands to:
   - `match runtime` → BranchBuilder with 3 arms
   - `gcp.STS.Exchange(...)` → transport triplet
   - `when service_account` → guarded node (impersonation optional)
   - `gcp.SecretManager.AccessVersion(...)` → transport triplet
   - `build_credential(...)` → pure node

3. Resource `Network` is inferred from service calls with `@rest` — the compiler inserts `net_env` and threads it to all transport execute nodes.

4. Test obligations derived automatically (100+ tests from the graph structure).

## B.4 Generated Tests (from compiler)

Same 4-bucket structure as gunbc testgen, but derived from the DSL rather than hand-wired:

```
Bucket A: DryRunCompletion, TransportInterceptable × 4
Bucket B: EdgePredicateEntailment × 2, NodeContractCompliance × 14, OptionalInputHandling × 8
Bucket C: AllTransportsSucceed, SingleTransportFailure × 4, GuardBranchCoverage × 2
Bucket D: TransportResourceDeclared × 4, ResourceInputConnected × 4
```

---

# Appendix C: Service Composition (Gist Snapshot)

Shows journey composition, loops, and multi-service orchestration.

## C.1 Today: Rust (gunbc)

`lib/tools/gist/src/graph.rs` — 1,449 lines covering 3 modes (snapshot, diff, recent).

The snapshot mode alone involves:
- Git operations (branch resolution SubDag, ls-files)
- Loop over files (LoopBuilder for per-file reads)
- Markdown rendering
- Cloud credential chain (SubDag)
- Gist API call (transport triplet)

## C.2 DSL

### `services/git.dag`

```
module services.git

service git.Core {
  operation CurrentBranch {
    input {}
    output { branch: String }
    @shell("git rev-parse --abbrev-ref HEAD")
  }

  operation LsFiles {
    input {}
    output { files: List<String> }
    @shell("git ls-files")
  }

  operation Diff {
    input { base: String, head: String = "HEAD" }
    output { diff: String }
    @shell("git diff {base}...{head}")
  }

  operation RevList {
    input { since: String }
    output { commits: List<String> }
    @shell("git rev-list --since={since} HEAD")
  }
}
```

### `services/github/gist.dag`

```
module services.github.gist

service github.Gist {
  operation Create {
    input {
      description: String
      files: Map<String, String>
      public: Bool = false
    }
    output { url: String, id: String }
    @rest(POST, "https://api.github.com/gists")
    @permissions(["gist"])
  }
}
```

### `tools/gist.dag`

```
module tools.gist

import services.git
import services.github.gist
import std.patterns { credential_chain }

journey gist_upload {
  input {
    markdown: String
    branch: String
    base_ref: String?
  }
  output { url: String }
  uses net: Network

  filename = gist_filename(branch: branch, base_ref: base_ref)
  cred = credential_chain(runtime: detect_runtime(), ...)

  result = github.Gist.Create(
    description: "Snapshot from {branch}",
    files: { filename: markdown },
    credential: cred.credential
  )

  return { url: result.url }
}

journey gist_snapshot {
  input { base_ref: String? }
  output { url: String }
  uses fs: Filesystem(mode: Read)

  branch = git.Core.CurrentBranch()
  files = git.Core.LsFiles()

  contents = for file in files.files {
    fs.read(path: file)
  }

  markdown = render_snapshot(files: contents)
  result = gist_upload(
    markdown: markdown,
    branch: branch.branch,
    base_ref: base_ref
  )

  return { url: result.url }
}

journey gist_diff {
  input { base_ref: String }
  output { url: String }

  branch = git.Core.CurrentBranch()
  diff = git.Core.Diff(base: base_ref)
  markdown = render_diff(diff: diff.diff)
  result = gist_upload(
    markdown: markdown,
    branch: branch.branch,
    base_ref: base_ref
  )

  return { url: result.url }
}

journey gist_recent {
  input { since: String = "3.days.ago" }
  output { url: String }

  branch = git.Core.CurrentBranch()
  commits = git.Core.RevList(since: since)
  diffs = for commit in commits.commits {
    git.Core.Diff(base: "{commit}~1", head: commit)
  }
  markdown = render_recent(diffs: diffs)
  result = gist_upload(
    markdown: markdown,
    branch: branch.branch
  )

  return { url: result.url }
}
```

**Total: ~80 lines** (vs. 1,449 lines for the Rust graph builder).

## C.3 ProgressManifest for `gist_snapshot`

```
ProgressManifest {
  total_nodes: 12  // includes expanded credential_chain SubDag
  waves: [
    Wave { depth: 0, nodes: ["branch", "files"] }           // parallel
    Wave { depth: 1, nodes: ["loop:contents"] }              // scatter group
    Wave { depth: 2, nodes: ["render"] }
    Wave { depth: 3, nodes: ["cred_chain"] }                 // expandable SubDag
    Wave { depth: 4, nodes: ["gist_create"] }
  ]
  labels: {
    "branch": "branch",
    "files": "ls-files",
    "loop:contents": "read files",
    "render": "render",
    "cred_chain": "credential",
    "gist_create": "upload"
  }
  expand_points: ["cred_chain"]  // SubDag that can be expanded to see inner nodes
  groups: []
}
```

**Terminal output (inline)**:
```
gist ─ 4/6 ━━━━━━━━━━━━░░░░ 67% [✓ branch] [✓ ls-files] [✓ read 8/8] [✓ render] [◐ credential] [○ upload]
```

The `read 8/8` is a scatter group — the LoopBuilder expanded to 8 parallel file reads.

---

# Appendix D: CI Pipeline

Shows pipeline construct with stages, parallel groups, and aggregation.

## D.1 Today: Rust (gunbc)

`gunbc-dag/src/ci/graph.rs` — 920 lines.

## D.2 DSL

### `pipelines/ci.dag`

```
module pipelines.ci

import tools.makegen
import tools.bootstrap
import meta.testgen
import meta.codegen

pipeline ci {
  stage codegen {
    codegen.check()
  }

  stage generate [after codegen] {
    parallel {
      bootstrap()
      pragma()
      testgen()
    }
  }

  stage build [after generate] {
    cargo_build()
  }

  stage verify [after build] {
    parallel {
      cargo_test()
      clippy()
    }
  }

  stage report [after verify] {
    aggregate(results: [verify.*])
  }
}
```

## D.3 ProgressManifest

```
ProgressManifest {
  total_nodes: 8
  waves: [
    Wave { depth: 0, nodes: ["codegen.check"] }
    Wave { depth: 1, nodes: ["bootstrap", "pragma", "testgen"] }
    Wave { depth: 2, nodes: ["cargo_build"] }
    Wave { depth: 3, nodes: ["cargo_test", "clippy"] }
    Wave { depth: 4, nodes: ["aggregate"] }
  ]
  groups: [
    StageGroup { name: "codegen",  nodes: ["codegen.check"], parallel: false }
    StageGroup { name: "generate", nodes: ["bootstrap", "pragma", "testgen"], parallel: true }
    StageGroup { name: "build",    nodes: ["cargo_build"], parallel: false }
    StageGroup { name: "verify",   nodes: ["cargo_test", "clippy"], parallel: true }
    StageGroup { name: "report",   nodes: ["aggregate"], parallel: false }
  ]
}
```

**Terminal output (inline)**:
```
ci ─ stage: verify 6/8 ━━━━━━━━━━━━░░░░ 75%
  [✓ codegen] [✓ bootstrap ✓ pragma ✓ testgen] [✓ build] [◐ test ◐ clippy] [○ report]
```

---

# Appendix E: Tool Installation (Upsert)

Shows the upsert pattern for tool installation.

## E.1 Today: Rust (gunbc)

`lib/tools/clippy/src/graph.rs` — 186 lines using `UpsertBuilder`.

```rust
let node = UpsertBuilder::new("install_clippy")
    .with_check(ClippyOp::Check)        // which clippy-driver
    .with_create(ClippyOp::Install)      // rustup component add clippy
    .with_resolve(ClippyOp::Resolve)     // clippy-driver --version
    .build();
```

## E.2 DSL

### `tools/clippy.dag`

```
module tools.clippy

import std.patterns { upsert }

journey install_clippy {
  output { handle: String }

  result = upsert {
    check:   shell("which clippy-driver") -> { exists: Bool }
    create:  shell("rustup component add clippy")
    resolve: shell("clippy-driver --version") -> { handle: String }
  }

  return { handle: result.handle }
}

journey clippy_lint {
  input { paths: List<String>? }
  output { clean: Bool, findings: String }
  uses tool: install_clippy

  result = shell("cargo clippy -- -D warnings")
  return { clean: result.exit_code == 0, findings: result.stdout }
}
```

---

# Appendix F: LLM Review Workflow

Shows cloud credential + LLM service composition.

## F.1 Today: Rust (gunbc)

`lib/review/src/graph.rs` — 1,376 lines with blob acquisition, credential chain, LLM request.

## F.2 DSL

### `tools/review.dag`

```
module tools.review

import services.git
import cloud.gcp.credential
import std.patterns { credential_chain }

service llm.OpenAI {
  operation ChatCompletion {
    input {
      model: String = "gpt-4"
      messages: List<Message>
      temperature: Float = 0.3
    }
    output { content: String, usage: TokenUsage }
    @rest(POST, "https://api.openai.com/v1/chat/completions")
  }
}

type Message {
  role: String        // "system" | "user" | "assistant"
  content: String
}

type TokenUsage {
  prompt_tokens: Int
  completion_tokens: Int
}

journey review_diff {
  input {
    base_ref: String
    system_prompt: String?
  }
  output { review: String }
  uses net: Network

  diff = git.Core.Diff(base: base_ref)
  cred = credential_chain(runtime: detect_runtime(), ...)

  prompt = build_review_prompt(
    diff: diff.diff,
    system: system_prompt
  )

  result = llm.OpenAI.ChatCompletion(
    messages: prompt,
    credential: cred.credential
  )

  return { review: result.content }
}
```

---

# Appendix G: Rendering / Emission

Shows how the DSL models rendering as a concept, not a special system.

## G.1 The Problem in gunbc

13 rendering systems, 5 different traits, 8 with no trait. The unified-emission design doc proposed `OutputMedium` trait hierarchy + 5 migration phases.

## G.2 DSL Approach

Rendering is a pure node. No special system needed.

```
// Rendering is just a pure function node — no concept/trait needed
// unless you want polymorphism across renderers

journey render_makefile {
  input { registry: ToolRegistry }
  output { content: String }

  // Pure transformation — no I/O, no resources
  content = makefile_render(registry: registry, format: Makefile)
  return { content: content }
}

journey render_ci_yaml {
  input { pipeline: PipelineManifest }
  output { content: String }

  content = ci_yaml_render(pipeline: pipeline, provider: GitHubActions)
  return { content: content }
}
```

If polymorphism is needed (e.g., render to Ansi vs Plain vs HTML):

```
concept Renderable {
  operation Render {
    input { content: Any, format: RenderFormat }
    output { rendered: String }
  }
}

type RenderFormat = PlainText | Ansi | Html | Markdown
```

The key insight: rendering doesn't need 13 systems or 5 traits. It needs typed pure nodes whose output flows through the DAG like anything else. The `content_upsert` pattern handles writing the rendered output to a file.

---

# Appendix H: Pattern Catalog

All patterns from gunbc, expressed in DSL syntax.

### Upsert (check → create → resolve)
```
pattern upsert<Check, Create, Resolve> {
  node check: Check -> { exists: Bool }
  node create [when !check.exists]: Create -> { ref: String }
  node resolve: Resolve -> { handle: String }
}
```

### Content Upsert (generate → read → compare → skip-if-unchanged write)
```
pattern content_upsert {
  input { content: String, path: String }
  uses fs: Filesystem(mode: ReadWrite)

  node read: fs.read(path: path)
  node compare: eq(a: content, b: read.content) -> { changed: Bool }
  node write [when compare.changed]: fs.write(path: path, content: content)

  output { written: Bool = compare.changed }
}
```

### Credential Chain (OIDC → STS → optional impersonation → secret access)
```
pattern credential_chain {
  input { runtime: CloudRuntime, audience: String, service_account: String?, ... }
  uses net: Network

  node token = match runtime { ... }
  node access = gcp.STS.Exchange(subject_token: token.token)
  node impersonated = when service_account { ... } else { access }
  node secret = gcp.SecretManager.AccessVersion(...)

  output { credential: Credential }
}
```

### Transaction (begin → body → commit/rollback)
```
pattern transaction<Begin, Body, Commit, Rollback> {
  node begin: Begin -> { tx_id: String }
  node body: Body
  node commit [when body.success]: Commit
  node rollback [when !body.success]: Rollback
}
```

### Retry (execute → check → re-execute on failure)
```
pattern retry<Op> {
  input { max_attempts: Int = 3, backoff_ms: Int = 1000 }
  node op: Op
  @retry(max: max_attempts, backoff: exponential(backoff_ms))
}
```

### Loop (iterate over collection)
```
// Expressed inline in journeys:
contents = for file in files.files {
  fs.read(path: file)
}
```

### Branch (conditional routing)
```
// Expressed inline:
node result = match condition {
  A => journey_a(...)
  B => journey_b(...)
}

// Or with when:
node optional_step [when flag] { ... }
```

### Emit (prepare → format → hash → compare → write → record)
```
pattern emit {
  input { content: String, path: String }
  uses fs: Filesystem(mode: ReadWrite)

  node hash: content_hash(content: content)
  node read_existing: fs.read(path: "{path}.hash")
  node compare: eq(a: hash.hash, b: read_existing.content) -> { changed: Bool }
  node write_content [when compare.changed]: fs.write(path: path, content: content)
  node write_hash [when compare.changed]: fs.write(path: "{path}.hash", content: hash.hash)

  output { written: Bool = compare.changed }
}
```

---

# Appendix I: Inspiration Targets

| Source | What to take | What to avoid |
|--------|-------------|---------------|
| **Smithy (AWS)** | `service` + `operation`, `@trait` annotations, resource lifecycle | XML heritage, complex trait algebra |
| **Terraform HCL** | Provider-qualified names, implicit deps from references | Mutable state, plan/apply split |
| **CUE** | Constraints inline, defaults, unification for inheritance | Value lattice complexity |
| **dbt** | `ref()` implicit DAG, model auto-discovery from filesystem | SQL-only, no type system |
| **Concourse CI** | Resource types, pipeline-as-DAG, `passed` constraints | YAML, no composition |
| **Dhall** | Totality, no side effects, imports with integrity checks | Haskell syntax barrier |
| **Protobuf** | Language-agnostic IDL, codegen plugins, evolution rules | No computation, no DAG |
| **the-gunbai TUI** | Inline progress, TUI DAG viz, edge pulses, wave layout | Runtime-only layout |
| **Nix** | Reproducible, declarative, lazy | Complexity, learning curve |

**Anti-inspirations**:
- **Airflow**: Imperative DAG construction (exactly what we're replacing)
- **YAML pipelines**: No type system, stringly-typed (what V2 rejected)
- **Terraform state**: Mutable state management (our model is stateless)
- **Pulumi**: Host-language coupling (defeats language-agnosticism)
- **Helm**: Template-of-a-template layering (complexity without guarantees)

---

# Appendix J: Cross-Repository Capability Matrix

This appendix documents what each generation of the platform (gunb.ai, the-gunbai, gunbc) provides "for free" — meaning what the framework gives you without manual effort — and how capabilities transfer across the lineage as scenarios evolve.

## J.1 The Lineage: What Each Generation Proved

```
gunb.ai (v1, Go)          the-gunbai (v2, Rust)       gunbc (v3, Rust)           DSL (v4, target)
──────────────────         ──────────────────────      ──────────────────         ──────────────────
DAGs work                  Codegen from knowledge      Full IR + proofs           Language-level DAGs
CaptureWriter              TUI progress                Testgen (73% gen)          95% generated code
Lease execution            40+ understandings          Transport boundaries       Filesystem discovery
                           195+ behaviors              DryRun interception        Multi-target emission
```

### What "for free" means at each generation

| Generation | "For free" = | You still write manually |
|---|---|---|
| **gunb.ai** | Parallel execution, output capture, progress sections | Everything: graph wiring, tests, types, discovery, progress groups |
| **the-gunbai** | Integration code from understandings, TUI progress, some contract tests | Graph wiring, most tests, IR is implicit, no structural guarantees |
| **gunbc** | Structural soundness, 73% of tests, DryRun, transport interception, pattern reuse | Graph builders (7,000+ lines), discovery (6 islands), progress rendering |
| **DSL** | Graph authoring (10-100x compressed), discovery, progress manifest, multi-language codegen | Pure transformation logic (~5% of total code) |

## J.2 Scenario Inventory

Seven canonical scenarios span the three repos. Each exercises different framework capabilities.

| # | Scenario | Pattern | Key Concern | Repos |
|---|---|---|---|---|
| S1 | Content upsert (makegen) | `content_upsert` | File generation, skip-if-unchanged | gunbc, DSL |
| S2 | Cloud credential acquisition | `credential_chain` | Multi-transport, branching, guards | gunb.ai, the-gunbai, gunbc, DSL |
| S3 | Tool installation | `upsert` | Check → install → resolve | gunbc, DSL |
| S4 | Service composition (gist) | Journey composition + loop | Multi-service, loops, SubDag | gunb.ai, the-gunbai, gunbc, DSL |
| S5 | CI pipeline | `pipeline` | Stages, parallel groups, aggregation | gunbc, DSL |
| S6 | LLM review | Credential + service call | Cloud + external API composition | gunbc, DSL |
| S7 | Authentication flow | Interactive + credential | Passthrough, TUI sections, OAuth | gunb.ai, the-gunbai |

## J.3 Per-Scenario Cross-Comparison

### S1: Content Upsert (Makegen)

The simplest complete graph. Generates a file, reads the existing version, compares, writes only if changed.

**What each repo gets for free:**

| Capability | gunb.ai | the-gunbai | gunbc | DSL |
|---|---|---|---|---|
| Graph wiring | Manual (~500 lines) | Manual (~300 lines) | Builder + helper (~200 lines) | 5 lines of `.dag` |
| Skip-if-unchanged logic | Handwritten | Handwritten | `ContentUpsertChain` helper | `content_upsert` pattern (compiler expands) |
| Tests | All handwritten | Mostly handwritten | 73% generated (testgen derives obligations from graph structure) | 95%+ generated |
| DryRun | N/A | N/A | Free (transport boundary interception) | Free |
| Progress display | Manual groups | Runtime TUI | Frame-based (manual wiring) | ProgressManifest (compiler-derived) |
| File resource tracking | Implicit | Implicit | Typed `res:file:*` ports | `uses fs: Filesystem(mode: Write)` — compiler inserts lifecycle |

**The transition in concrete terms:**

```
gunb.ai:     500 lines Go + 100 lines test = 600 total, 0% generated
the-gunbai:  300 lines Rust + 80 lines test = 380 total, 0% generated
gunbc:       200 lines Rust + 50 lines test = 250 total, ~60% generated (testgen)
DSL:         5 lines .dag + 20 lines pure logic = 25 total, ~95% generated
```

gunbc's contribution to the DSL: `ContentUpsertChain` proved that the 5-node pattern (read → compare → conditional write) is universal. Every content-generation tool in gunbc uses it. The DSL makes it a first-class `pattern` that the compiler expands, eliminating the helper entirely.

### S2: Cloud Credential Acquisition (GCP)

The most complex graph. OIDC token → STS exchange → optional impersonation → secret access. The canonical stress test.

**What each repo gets for free:**

| Capability | gunb.ai | the-gunbai | gunbc | DSL |
|---|---|---|---|---|
| Graph wiring | ~800 lines Go | Understanding-generated (~400 lines) | ~1,688 lines Rust builders | ~50 lines across 2 `.dag` files |
| Transport triplets | Handwritten per API call | Generated from understandings | Handwritten (prepare/execute/parse × 8) | Compiler expands from `@rest` annotations |
| Runtime branching (GitHub Actions vs Metadata vs Local) | `switch` statement | Behavior routing | `BranchBuilder` (manual arm wiring) | `match runtime { ... }` — compiler creates BranchBuilder |
| Optional impersonation | `if` block | Conditional behavior | Guarded port (manual) | `when service_account { ... }` |
| Service declarations | Inline code | Understandings (structured docs) | Rust trait + `MethodMeta` (180 lines per service) | `service gcp.SecretManager { ... }` (20 lines) |
| Mock boundary for DryRun | N/A | N/A | Free (all 8 transport nodes intercepted) | Free |
| Test scenarios | All handwritten | Some generated from behavior patterns | 100+ generated (4-bucket testgen) | 100+ generated (same model, from DSL structure) |
| Resource threading | Manual env passing | Implicit | Manual `net_env` → `res:api:network` edges | `uses net: Network` — compiler threads automatically |

**The understanding → service transition:**

the-gunbai's key innovation was *understandings* — structured documents describing external systems:

```
the-gunbai understanding (conceptual):
  system: GCP Secret Manager
  operations:
    - name: AccessSecretVersion
      http: GET /v1/projects/{project}/secrets/{secret}/versions/{version}:access
      permissions: [secretmanager.versions.access]
      idempotent: true
```

This generated integration code, but the understanding was a separate document format with no type system and no IR. gunbc replaced understandings with Rust traits (`SecretManagerService`) and `MethodMeta` structs — typed, but verbose (180 lines per service).

The DSL synthesizes both: structured declarations with a type system:

```
service gcp.SecretManager {
  operation AccessVersion {
    input { project: String, secret: String, version: String = "latest" }
    output { payload: Bytes, name: String }
    @rest(GET, "/v1/projects/{project}/secrets/{secret}/versions/{version}:access")
    @idempotent @readonly
    @permissions(["secretmanager.versions.access"])
  }
}
```

This is the understanding made first-class. The `@rest` annotation IS the understanding — but now it lives inside a typed language with a compiler that expands it to transport triplets, generates mocks, and derives test obligations.

**What gunbc gives the DSL for free:** The transport triplet pattern (prepare/execute/parse), DryRun interception model, 4-bucket testgen obligation structure, and resource conflict detection algorithm. All of these transfer directly — the DSL compiler produces the same IR that gunbc's hand-wired builders produce.

### S3: Tool Installation (Upsert)

Check if a tool exists → install if missing → resolve the installed handle.

**What each repo gets for free:**

| Capability | gunb.ai | the-gunbai | gunbc | DSL |
|---|---|---|---|---|
| Upsert pattern | Handwritten check/install/verify | Handwritten | `UpsertBuilder` (5 lines of builder code) | `upsert { check: ..., create: ..., resolve: ... }` |
| Skip-if-exists | Manual `if` | Manual | Guard on create node (structural) | `[when !check.exists]` — compiler emits guard |
| Tool registration | Hardcoded list | Hardcoded list | `#[tool_target]` proc macro → inventory | Filesystem discovery (tool is a `.dag` file) |
| Tests | All handwritten | All handwritten | Generated: DryRun, guard branch coverage, skip-path | Generated: same obligations, from `.dag` structure |

**Pattern evolution:**

```
gunb.ai:      if !which("clippy") { rustup_add("clippy") }; version = clippy_version()
the-gunbai:   behavior CheckClippy → behavior InstallClippy → behavior ResolveClippy
gunbc:        UpsertBuilder::new("clippy").with_check(...).with_create(...).with_resolve(...)
DSL:          upsert { check: shell("which clippy-driver"), create: shell("rustup ..."), resolve: shell("clippy-driver --version") }
```

Each generation compressed the pattern. gunbc's `UpsertBuilder` proved the 3-node shape is universal — every tool installation follows it. The DSL makes `upsert` a built-in pattern keyword.

### S4: Service Composition (Gist Snapshot)

Multi-service orchestration: git operations → file reads (loop) → rendering → credential chain (SubDag) → API call.

**What each repo gets for free:**

| Capability | gunb.ai | the-gunbai | gunbc | DSL |
|---|---|---|---|---|
| Graph wiring | ~600 lines Go | ~400 lines Rust | 1,449 lines Rust (3 modes) | ~80 lines `.dag` (3 modes) |
| Loop over files | Manual goroutine pool | Manual iterator | `LoopBuilder` (manual body wiring) | `for file in files.files { fs.read(path: file) }` |
| SubDag composition | N/A (flat graph) | N/A | Manual SubDag node construction | `cred = credential_chain(...)` — journey call = SubDag |
| Progress for parallel file reads | Manual counter | TUI scatter group | Not displayed (no scatter support) | Scatter group: `read files [8/8]` (from ProgressManifest) |
| Journey composition | N/A | N/A | Manual SubDag wiring | `result = gist_upload(...)` — implicit composition |
| Multi-mode support (snapshot/diff/recent) | 3 separate Go programs | 3 separate commands | 3 graph builders in one file | 3 journeys in one `.dag` file |

**What the-gunbai contributed that gunbc lost:**

the-gunbai's TUI had scatter group progress for parallel tasks: `[2/5]` showing how many items in a loop had completed. gunbc's frame-based renderer doesn't support this — it was rebuilt from scratch without inheriting the-gunbai's progress model.

The DSL fixes this by deriving `ProgressManifest` at compile time, which includes `scatter_points` — nodes that expand to parallel instances at runtime. The renderer gets scatter groups for free because the compiler tells it where loops are.

### S5: CI Pipeline

Staged workflow: codegen → generate (parallel: bootstrap, pragma, testgen) → build → verify (parallel: test, clippy) → report.

**What each repo gets for free:**

| Capability | gunb.ai | the-gunbai | gunbc | DSL |
|---|---|---|---|---|
| Stage ordering | Make targets with deps | Make targets | 920 lines of graph builder | `stage build [after generate]` |
| Parallel execution within stage | Manual goroutines | Manual | SubDag with parallel roots | `parallel { bootstrap(); pragma(); testgen() }` |
| Stage groups in progress | Manual | TUI groups | Frame sections (manual) | ProgressManifest `StageGroup` (compiler-derived) |
| Aggregation | Manual result collection | Manual | Manual aggregate node | `aggregate(results: [verify.*])` |
| Makefile/CI YAML generation | Handwritten | Handwritten | `makegen` tool (Content upsert) | Compiler emits from pipeline definition |

### S6: LLM Review

Combines credential acquisition with an external LLM API call. Shows how cloud infrastructure composes with arbitrary services.

**What each repo gets for free:**

| Capability | gunb.ai | the-gunbai | gunbc | DSL |
|---|---|---|---|---|
| Graph wiring | N/A (didn't exist) | N/A | 1,376 lines Rust | ~30 lines `.dag` |
| Credential reuse | N/A | N/A | SubDag reference to credential chain | `cred = credential_chain(...)` |
| New service declaration | N/A | New understanding document | New Rust trait (180 lines) + ops match arm | `service llm.OpenAI { operation ChatCompletion { ... } }` |
| Test generation | N/A | N/A | Full testgen (DryRun, scenarios, resource hygiene) | Same, from `.dag` structure |
| Adding this workflow | N/A | ~500 lines + understanding | ~1,500 lines across 6 files | ~50 lines across 2 files |

**Key insight:** In gunbc, adding a new service (like an LLM provider) requires ~180 lines of Rust trait + `MethodMeta`, plus ~40 lines of ops enum wiring, plus manual graph construction. In the DSL, it's a `service` declaration with `@rest` annotations — the compiler generates the trait, the meta, and the transport triplets.

### S7: Authentication Flow (Interactive)

OAuth login with interactive terminal passthrough. The user's browser opens, they authenticate, the CLI resumes.

**What each repo gets for free:**

| Capability | gunb.ai | the-gunbai | gunbc | DSL |
|---|---|---|---|---|
| Subprocess output capture | `CaptureWriter` (manual) | Integrated in TUI | Frame-based (rebuilt) | `CaptureMode::Captured` (compiler default) |
| Interactive passthrough | `cmd.Stdout = os.Stdout` (manual) | Passthrough mode in TUI | Not fully implemented | `@interactive` annotation → `CaptureMode::Passthrough` |
| Progress pause/resume during OAuth | Manual clear/resume | TUI handles it | Not implemented | Compiler + runtime handle from manifest |
| Section headers (› Authentication) | Manual `ProgressOptions.Groups` | Inferred from DAG in TUI | Manual section wiring | Inferred from SubDag boundaries (structural) |

**gunb.ai → DSL transition for progress:**

gunb.ai required manual progress group declarations:
```go
ProgressOptions{
    Groups: []dag.ProgressGroup{
        {Name: "Authentication", Nodes: [...]},
        {Name: "Fetching Secrets", Nodes: [...]},
    },
}
```

the-gunbai improved this — the TUI inferred groups from runtime DAG traversal, but only at runtime (couldn't preview before execution).

gunbc rebuilt progress from scratch, losing the-gunbai's TUI quality but gaining `build_frame()` as a pure function.

The DSL combines all three: sections emerge from SubDag boundaries (like the-gunbai), the manifest is computed at compile time (improving on runtime inference), and the visual design matches gunb.ai's proven color palette and icon vocabulary (already ported to gunbc's symbol system). Progress becomes a view that can be rendered before, during, and after execution from the same manifest.

## J.4 Capability Transfer Matrix

What each repo contributes to the DSL, organized by concern.

### DAG Modeling

| What transfers | From | To DSL as |
|---|---|---|
| `Node<T>`, `Dag<T>`, `Port`, `Edge` types | gunbc | Core IR target (identical structure after lowering) |
| `DagBuilder` with generations | gunbc | Compiler's `Lower` phase produces same builder output |
| `Cardinality` (One, ZeroOrOne, ZeroOrMore, OneOrMore) | gunbc | Simplified to `T`, `T?`, `List<T>` in surface syntax |
| Acyclicity by construction | gunbc | Guaranteed by language (no cycles expressible in `.dag`) |
| Boundary/entrypoint detection | gunbc | Compiler's `Validate` phase (same algorithm) |

### Testing

| What transfers | From | To DSL as |
|---|---|---|
| 4-bucket obligation model | gunbc testgen | Compiler's `Derive` phase produces `TestObligations` |
| Anti-tautology rule | gunbc testgen | Same: only generate tests for Unknown/RuntimeOnly obligations |
| DryRun completion test | gunbc | Generated for every journey |
| Transport interception test | gunbc | Generated for every service call |
| N+1 scenario coverage | gunbc | Generated: all-succeed + per-transport failure |
| Guard branch coverage | gunbc | Generated from `when` / `match` constructs |
| Resource hygiene | gunbc | Generated from `uses` declarations |
| MockSpec infrastructure | gunbc | Compiler generates MockSpec from service declarations |
| `Simulator` / `IoContract` | gunbc | Generated from typed ports |

### Progress & Terminal

| What transfers | From | To DSL as |
|---|---|---|
| `CaptureWriter` pattern | gunb.ai | Per-node `CaptureBuffer` (default for all transport nodes) |
| Passthrough mode | gunb.ai | `@interactive` annotation → `CaptureMode::Passthrough` |
| Section rendering (`›`) | gunb.ai | Inferred from SubDag boundaries in ProgressManifest |
| Error boxes (bordered, captured stderr) | gunb.ai | Same visual design, driven by CaptureBuffer on failure |
| Color palette (ANSI 256) | gunb.ai → gunbc | Identical: `SemanticColor` enum, same codes |
| Spinner (braille, 80ms) | gunb.ai → gunbc | Identical: same frames, same timing |
| TUI with edge pulses | the-gunbai | Optional `tui` renderer reading ProgressManifest |
| Wave-based layout | the-gunbai | `TopologyNode.depth` in ProgressManifest |
| Scatter groups (`[2/5]`) | the-gunbai | `scatter_points` in ProgressManifest |
| Inline progress bar | the-gunbai | `inline` renderer reading ProgressManifest |
| JSONL event streaming | the-gunbai | `jsonl` renderer reading ProgressManifest |
| Frame-based display (`build_frame()`) | gunbc | Manifest-driven frame builder (same pure function concept) |
| `OutputMedium` / `SemanticColor` / `SymbolId` | gunbc | Terminal crate (harvested directly, ~2,271 lines) |

### Services & Resources

| What transfers | From | To DSL as |
|---|---|---|
| Understanding concept (structured external system docs) | the-gunbai | `service` declarations with `@rest`, `@shell` annotations |
| Behavior generation from understandings | the-gunbai | Compiler expands service operations to transport triplets |
| `SecretManagerService` trait + `MethodMeta` | gunbc | `service gcp.SecretManager { operation ... }` |
| Transport triplet (prepare/execute/parse) | gunbc | Compiler generates from service call + `@rest`/`@shell` |
| DryRun interception at transport boundary | gunbc | Same: mock transport executor swapped at execute node |
| `ResourceAccess` / `detect_conflicts()` | gunbc | Compiler's resource conflict check in `Validate` phase |
| Typed resource ports (`res:*`) | gunbc | `uses fs: Filesystem(mode: Write)` — compiler threads edges |
| Lease/heartbeat model | gunb.ai | `resource` with lifecycle (acquire/use/release) |

### Discovery

| What transfers | From | To DSL as |
|---|---|---|
| Manual hardcoded lists | gunb.ai, the-gunbai | Eliminated |
| `#[tool_target]` proc macro | gunbc | Eliminated (filesystem discovery) |
| `#[testgen_target]` proc macro | gunbc | Eliminated (every journey has test obligations) |
| `build_workspace_dag()` | gunbc | Eliminated (module graph IS workspace DAG) |
| `inventory` crate | gunbc | Eliminated |

## J.5 The "For Free" Progression: Worked Example

To make the transition concrete, here is the complete lifecycle of adding a new tool — from gunb.ai through the DSL — showing what you write vs what the framework provides.

### Adding a "format" tool that runs `cargo fmt`

**gunb.ai (v1) — you write everything:**

```go
// 1. Tool struct (30 lines)
type FormatTool struct { ... }

// 2. DAG node (50 lines)
func (t *FormatTool) Execute(ctx context.Context, inputs dag.Inputs) (dag.Outputs, error) {
    cmd := exec.CommandContext(ctx, "cargo", "fmt", "--check")
    cmd.Stdout = t.captureWriter  // CaptureWriter is free
    err := cmd.Run()
    // ... parse exit code, build outputs
}

// 3. Registration in workspace DAG (20 lines)
func buildWorkspaceDag() *dag.Dag {
    // ... hardcoded list ...
    dag.AddNode("format", &FormatTool{})
}

// 4. Progress group (10 lines)
ProgressOptions{Groups: []dag.ProgressGroup{{Name: "Format", Nodes: ["format"]}}}

// 5. Tests (100 lines) — ALL handwritten
func TestFormatSuccess(t *testing.T) { ... }
func TestFormatFailure(t *testing.T) { ... }

// 6. CLI entrypoint (40 lines)
func main() { ... }

// 7. Makefile target (5 lines)
// Total: ~255 lines, 0% generated
// Free: parallel execution, output capture
```

**the-gunbai (v2) — understandings help:**

```rust
// 1. Understanding document (30 lines)
// system: cargo-fmt
// command: cargo fmt --check
// behaviors: [check_formatting, format_code]

// 2. Generated behavior code (~80 lines generated, ~40 manual)
// 3. TUI integration (free from framework)
// 4. Registration (manual, 10 lines)
// 5. Tests (~60 lines, some generated from behavior patterns)
// Total: ~150 lines manual + ~80 generated = ~230 total
// Free: TUI progress, some integration code, some tests
```

**gunbc (v3) — IR + testgen help:**

```rust
// 1. Graph builder (60 lines)
pub fn build_format_graph() -> Dag<FormatOp> {
    let mut builder = DagBuilder::new();
    let upsert = UpsertBuilder::new("cargo_fmt")
        .with_check(FormatOp::Check)       // which cargo-fmt
        .with_create(FormatOp::Install)     // rustup component add rustfmt
        .with_resolve(FormatOp::Resolve);   // cargo fmt --version
    // ... transport nodes for the actual fmt --check
    builder.build()
}

// 2. Op enum + implementations (80 lines)
// 3. MockSpec (20 lines)
// 4. Registration: #[tool_target] + #[testgen_target] (10 lines)
// 5. Tests: ~15 lines handwritten, ~40 lines generated by testgen
// 6. CLI entrypoint (generated by codegen, 0 manual)
// Total: ~170 lines manual + ~60 generated = ~230 total
// Free: structural soundness, 73% of tests, DryRun, CLI generation,
//       UpsertBuilder pattern, transport interception
```

**DSL (v4) — almost everything generated:**

```
// 1. tools/format.dag (entire tool definition)
module tools.format

import std.patterns { upsert }

journey install_fmt {
  output { handle: String }
  result = upsert {
    check:   shell("which cargo-fmt") -> { exists: Bool }
    create:  shell("rustup component add rustfmt")
    resolve: shell("cargo-fmt --version") -> { handle: String }
  }
  return { handle: result.handle }
}

journey format_check {
  input { paths: List<String>? }
  output { clean: Bool, diff: String }
  uses tool: install_fmt

  result = shell("cargo fmt --check")
  return { clean: result.exit_code == 0, diff: result.stdout }
}

// 2. Pure logic: parse_format_result (10 lines Rust/Go/Python)
// That's it. Everything else is generated.
// Total: ~25 lines manual, ~200+ lines generated
// Free: graph wiring, transport triplets, all tests, DryRun,
//       progress manifest, CLI, discovery, resource lifecycle,
//       Makefile target, CI integration
```

### Compression ratio by generation

| Generation | Manual Lines | Generated Lines | Total | Manual % |
|---|---|---|---|---|
| gunb.ai | 255 | 0 | 255 | 100% |
| the-gunbai | 150 | 80 | 230 | 65% |
| gunbc | 170 | 60 | 230 | 74% |
| DSL | 25 | 200+ | 225+ | ~11% |

Note: gunbc's manual percentage is *higher* than the-gunbai's for graph wiring because gunbc's IR is more explicit (typed ports, cardinality, explicit edges). The testgen compensates by generating more tests, but the authoring cost increased. This is precisely the problem the DSL solves — the IR's explicitness is a feature (enables testgen, DryRun, structural proofs), but the authoring surface must be compressed.

## J.6 What Doesn't Transfer (Lessons from Each Failure)

Each generation also proved what *not* to do. These negative lessons are as important as the positive transfers.

| Lesson | Learned from | Applied in |
|---|---|---|
| Manual progress groups don't scale | gunb.ai's `ProgressOptions.Groups` | DSL: progress is a view, inferred from DAG structure |
| Understanding format without type system leads to drift | the-gunbai's 40+ understanding documents | DSL: `service` declarations are typed and compiled |
| Runtime-only TUI can't preview before execution | the-gunbai's ratatui TUI | DSL: ProgressManifest computed at compile time |
| Hand-wired graph builders don't scale past ~5 tools | gunbc's 7,000+ lines of builders | DSL: `.dag` files, 10-100x compression |
| Registration islands fragment discovery | gunbc's 6 registration systems | DSL: filesystem IS the registry |
| Transport types in core IR create coupling | gunbc's 17 transport modules in `core/ir/` | DSL: transport is late-bound (annotation → codegen backend) |
| Rebuilding progress from scratch loses quality | gunbc lost the-gunbai's TUI | DSL: harvest terminal code from gunbc + TUI from the-gunbai |
| `Value`/`ValueExpr` parallel hierarchies are technical debt | gunbc | DSL: codegen works from IR + types, no runtime value expressions |

## J.7 Scenario Coverage by Testgen Bucket

For each scenario, what test obligations are derivable from graph structure.

| Scenario | Bucket A (Execution) | Bucket B (Contract) | Bucket C (Scenarios) | Bucket D (Resources) |
|---|---|---|---|---|
| **S1: Content upsert** | DryRun, 2× transport intercept | 1× node compliance | All-succeed, 2× failure, 1× guard branch | 2× resource connected, conflict absence |
| **S2: Credential chain** | DryRun, 4× transport intercept | 2× entailment, 14× compliance | All-succeed, 4× failure, 2× guard branch | 4× resource connected, conflict absence |
| **S3: Tool install** | DryRun, 3× transport intercept | 1× compliance | All-succeed, 3× failure, 1× guard branch | 3× resource connected |
| **S4: Gist snapshot** | DryRun, 5× transport intercept | 3× compliance | All-succeed, 5× failure, 1× loop expansion | 3× resource connected, conflict absence |
| **S5: CI pipeline** | DryRun, 8× transport intercept | 5× compliance | All-succeed, 8× failure, 2× stage ordering | Stage resource isolation |
| **S6: LLM review** | DryRun, 5× transport intercept | 2× compliance | All-succeed, 5× failure, 1× guard branch | 3× resource connected |
| **S7: Auth flow** | DryRun, 6× transport intercept | 2× compliance | All-succeed, 6× failure, 2× guard branch, 1× interactive passthrough | 2× resource connected |

In gunb.ai and the-gunbai, **all** of these tests would be handwritten. In gunbc, Buckets A, C, and D are generated; Bucket B is partially generated. In the DSL, all four buckets are generated from the `.dag` file structure — the same obligation model, but derived from a 10-100x smaller source.

## J.8 Summary: The DAG Modeling → Understandings → gunbc Pipeline

The three repos represent a pipeline of increasing formalization:

```
gunb.ai                    the-gunbai                  gunbc                       DSL
─────────                  ──────────                  ─────                       ───
"DAGs work"                "Knowledge scales"          "Proofs work"               "Language compresses"

Proved that causal         Proved that structured       Proved that typed IR +      Combines all three:
DAGs are the right         knowledge about external     structural invariants +     typed DAGs authored in
abstraction for            systems (understandings)     proof obligations =         a language that compresses
workflow orchestration.    can generate integration     73% generated tests,        7000 lines to 50, while
                          code at scale.               DryRun, and structural      preserving every guarantee.
                                                       soundness guarantees.
                          ┌─────────────────────┐
                          │  What transfers:     │
                          │  Understanding       │───→  service + @rest/shell
                          │  concept             │      annotations (typed)
                          └─────────────────────┘
┌─────────────────────┐
│  What transfers:     │
│  CaptureWriter,      │───────────────────────────→  CaptureMode in IR
│  progress sections,  │                              Section inference
│  passthrough mode    │                              @interactive
└─────────────────────┘
                                                  ┌─────────────────────┐
                                                  │  What transfers:     │
                                                  │  IR types, patterns, │──→  Compiler target IR
                                                  │  testgen, transport, │     (identical structure)
                                                  │  DryRun, resources   │
                                                  └─────────────────────┘
```

Each generation's "free" capabilities compound: the DSL gets parallel execution from gunb.ai + codegen from the-gunbai + structural proofs from gunbc + language-level compression from the new compiler. Nothing is thrown away — everything is harvested, formalized, and made available through the `.dag` surface syntax.
