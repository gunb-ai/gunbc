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
- [K. Root Cause Analysis — Why gunbc Got Out of Control](#appendix-k-root-cause-analysis--why-gunbc-got-out-of-control)
- [L. A/B Workflow Comparisons and Handbook Reference](#appendix-l-ab-workflow-comparisons-and-handbook-reference)
- [M. Competitive Landscape and Alternatives Analysis](#appendix-m-competitive-landscape-and-alternatives-analysis)
- [N. Model-Based Testing and Auto-Generated Mocks](#appendix-n-model-based-testing-and-auto-generated-mocks)

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

// Refinement types (constraints on primitives — enables auto-fuzzing)
// See Appendix N for full model-based testing implications.
type CommitSha = String @pattern("^[a-f0-9]{40}$")
type RetryCount = Int @range(min: 1, max: 5)
type HttpStatus = Int @range(min: 100, max: 599)
type Email = String @pattern("^[^@]+@[^@]+\\.[^@]+$")
type Port = Int @range(min: 1, max: 65535)
type GistId = String @format(uuid)
type SecretValue = Secret @non_empty

// @pattern(regex)    — string must match regex
// @range(min, max)   — numeric bounds (inclusive)
// @format(preset)    — well-known format (uuid, uri, iso8601, semver)
// @non_empty         — string/list must have length > 0
// @one_of(values)    — value must be one of the listed literals
// @length(min, max)  — string/list length bounds
```

Design choice: no cardinality algebra. `T` is required-one, `T?` is optional, `List<T>` is zero-or-more. This is sufficient — gunbc's interval math (`Cardinality { min, max }`) was over-engineered for actual usage.

Design choice: refinement types constrain primitives with structural metadata that the compiler uses for three purposes: (1) validation at type-check time, (2) auto-generation of test inputs at derive time (see Appendix N), and (3) documentation of expected shapes for service consumers. Per Appendix K.6 guardrail G1, refinement annotations desugar to structural constraints — `@pattern` compiles to a validation predicate in the type's DAG representation, not opaque metadata.

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
    output { url: String, id: GistId }
    @rest(POST, "https://api.github.com/gists")
    @permissions(["gist"])
    @mock_response(
      status: 201,
      body: { "html_url": "https://gist.github.com/mock/{id}", "id": "{id}" }
    )
  }
}
```

Key: services are pure declarations. Every service call in a journey compiles to a transport triplet (prepare/execute/parse). The author never sees the triplet — the compiler emits it.

The `@mock_response` annotation is optional. When present, the compiler uses it to auto-generate `MockSpec` boundary values for Bucket A and Bucket C tests — eliminating hand-written mock fixtures for that operation. Output fields with refinement types (like `GistId = String @format(uuid)`) are populated by the compiler's type-aware generator. See Appendix N for the full model-based testing design.

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

---

# Appendix K: Root Cause Analysis — Why gunbc Got Out of Control

This appendix documents the precise failure modes that caused gunbc's codebase to accumulate glue, drift, and rework pressure — and traces each failure mode to the DSL construct that eliminates it.

The framing draws on internal postmortem documents: `TODO/TODONE/refactor-pressure.md` (2026-02-05), `TODO/TODONE/architecture-debt.md` (2026-02-05), `docs/design/consolidation-plan.md`, `docs/design/unified-registration.md`, and `docs/design/unified-emission.md`.

## K.1 The Precise Diagnosis

gunbc's IR is strong and ambitious. The design philosophy — "Everything is a DAG," behavior must be representable structurally, semantic meta-annotations are banned — is a very high bar for modeling. The core IR (`Node<T>`, `Dag<T>`, `Port`, `Edge`, cardinality algebra, transport boundaries, pattern library) is proven correct and heavily tested.

**The failure was not "lack of modeling." It was incomplete modeling: the IR layer was modeled aggressively, while the spec/registry/discovery/emission/progress layers often weren't, so meaning leaked into glue.**

From `architecture-debt.md` (2026-02-05 Weekly Signal):

> When a concept lacks a typed, structural home (IR/model/registry/resource), it leaks into templates, env access, string IDs, and ad-hoc rules — and then we refactor later to pull it back into structure.

And from `refactor-pressure.md`:

> We keep refactoring because the system still allows key behavior and meaning to exist outside the model (DAG/resources/types/IR), and the resulting duplicate sources of truth drift until they force a structural cleanup.

## K.2 Four Root Causes (from Internal Postmortem)

### A) Model is not closed — behavior exists outside the DAG

Code reached out to the environment implicitly: `std::env::var()`, `SystemTime::now()`, `Platform::detect()`, `FilesystemHandle::new()`. These calls happened inside opaque nodes, breaking DryRun interception, testability, and dependency reasoning.

**Leak → Fix dynamic (completed 2026-02-05):**

| Leak | Fixed with | Phase |
|---|---|---|
| Inline `SystemTime::now()` | `ClockEnv` node with explicit env port | Resource Phase 2 |
| Inline `Platform::detect()` | `PlatformEnv` node with explicit env port | Resource Phase 3 |
| Inline `FilesystemHandle::new()` | `FsEnv` node with explicit resource port | Resource Phase 1 |
| `std::env::var("ACTIONS_ID_TOKEN_REQUEST_URL")` | Explicit input ports on graph root | Resource Phase 4 |
| `GUNBC_EXEC_MODE` global | Exec mode via DAG edge | Resource Phase 5 |

**DSL fix:** `uses fs: Filesystem(mode: Write)` / `uses clock: Clock` / `uses net: Network`. Resources are declared, not accessed implicitly. The compiler inserts acquisition nodes and threads resources through edges. No code path can reach the environment without a declaration.

### B) Invariants are not enforced by construction — policy exists but allows escape hatches

The system had policies ("all I/O through transport boundaries," "no hidden env access") but allowed escape hatches. Churn happened when code violated policies and only discovered the violation during review, lint, or runtime.

**Examples:**
- `execute_transport()` was public — any opaque node could call it directly, bypassing the transport boundary model. **Fixed:** removed from public API; only `TransportOps::Execute` can perform I/O.
- `clippy.toml` banned `std::fs` and `std::process::Command`, but pragmas could disable the ban. **Fixed:** pragma audit with explicit exception list.
- Generated code could trigger lint warnings. **Policy:** if generated file triggers lint, fix the IR or clippy config — never add `#[allow]` in generated output.

**DSL fix:** The language itself prevents escape hatches. You cannot express I/O in a `.dag` file — the only way to do I/O is through a `service` operation with a `@rest` / `@shell` / `@file` annotation, which the compiler expands to a transport triplet. Escape hatches are not available in the surface syntax.

### C) Semantics are duplicated across layers — same meaning lives in two places

When the same concept exists in two places, they drift until cleanup becomes necessary.

**Documented examples:**

| Duplicate concept | Where it lived | Drift consequence |
|---|---|---|
| Boundary mocks | `registry.rs` AND `graph_mock.rs` | Mock values could disagree; tests might pass with stale mocks |
| Tool definitions | `all_tools()` vec AND `CliToolDef` constants | Forgot to add to `all_tools()` = tool exists but isn't discoverable |
| Hash logic | Per-crate implementations | Different crates computed different hashes for same content |
| Cardinality | Port cardinality AND type contract predicates | `Optional<T>` + `One` cardinality = contradictory claims |
| Graph builder identity | `GraphBuilderId` enum AND string templates | Rename builder function → string silently emits wrong name → runtime failure |

**DSL fix:** Single source of truth by construction. A `service` declaration IS the boundary mock source (the compiler generates MockSpec from `@rest` annotations). A `.dag` file IS the tool definition AND the graph builder AND the registry entry. Filesystem discovery eliminates all manual lists.

### D) Cross-cutting concerns appear before they have a home

Shared concerns (hashing, registry metadata, build artifact policy, resource dependency rules) got wedged into convenient places until third duplication forced a refactor.

**The pattern:** hashing appeared in three crates before `gunbc-infra::hash` was extracted. Freshness checking appeared in two crates before `gunbc-infra::freshness`. Rendering appeared in 13 places before the emission unification design was written.

**DSL fix:** The compiler pipeline has explicit homes for every cross-cutting concern: `Discover` phase for registry, `Derive` phase for progress manifest, `Emit` phase for rendering, `Validate` phase for resource conflicts. Concerns are modeled in the compiler, not discovered ad-hoc across crates.

## K.3 The "Generic IR Chokepoint" Problem

Beyond the four root causes, gunbc forced too much meaning through a generic IR chokepoint, and some semantics weren't preserved across that boundary.

**The canonical example: hermeticity.**

From `TODO/consolidation.md` (Integration Test Gap Analysis):

> **Design Problem: `TransportRequest` doesn't encode hermeticity.**
>
> We want test categories derived from the transport type system: **integration** (hermetic, local-only) vs **external** (non-hermetic, network/auth). But `TransportRequest` variant alone doesn't determine this.
>
> The problem is `Shell`. Higher-level domain types know whether they're hermetic, but that information is erased when they convert to `TransportRequest::Shell`:
>
> ```
> GitRequest::LsFiles.to_shell_request()    → Shell { ... }   // hermetic
> GistRequest::new().to_shell_request()     → Shell { ... }    // non-hermetic
> CargoCommand::Build.to_shell_request()    → Shell { ... }    // hermetic
> ```
>
> After conversion, these are indistinguishable at the transport layer. The executor sees `Shell(ShellRequest {...})` and has no way to know whether it hits the network.

This is the "IR-only" failure mode in a nutshell:
- The IR is too generic at the transport layer
- You lower into `TransportRequest::Shell` early
- You lose distinctions that matter for execution, policy, and test classification

**DSL fix:** Transport is late-bound (Design Principle P5). Service declarations carry semantic metadata (`@rest`, `@shell`, `@permissions`, `@idempotent`, `@readonly`) that the compiler preserves through lowering. The compiler can classify `git.Core.LsFiles()` as hermetic (local shell, no network annotation) vs `github.Gist.Create()` as non-hermetic (`@rest` with `@permissions`) — because the service declaration IS the source of truth, not a generic `Shell` variant.

## K.4 What Is NOT a Failure (Correcting Overstatements)

The internal reconciliation notes (`consolidation-plan.md`) explicitly correct some "redundancy claims" that were wrong or overstated:

| Claimed redundancy | Actual status |
|---|---|
| "Dual-source boundary mocks" | Not actually dual-sourced — MockSpec.to_boundary_mocks() is the single source |
| "CliToolDef/ToolDef duplication" | Intentional separation: platform satisfiability (ToolDef) vs runtime acquisition (CliToolDef) |
| "Type/cardinality duplication" | Already unified via TypeRegistry.infer_cardinality() |

The lesson: not all apparent duplication is harmful. Some "duplicate" structures serve distinct purposes and should remain separate.

## K.5 Root Causes → DSL Features Traceability Matrix

Each gunbc pain point mapped to the DSL construct that eliminates it and the compiler pass that enforces it.

| # | Pain Point | Root Cause | DSL Construct | Compiler Pass | Evidence |
|---|---|---|---|---|---|
| 1 | 7,000+ lines of hand-wired builders | No front-end language | `.dag` files with journey/pattern syntax | Parse → Lower | `dsl-design.md` §1 |
| 2 | 6 registration islands | No unified discovery | Filesystem as registry | Discover | `unified-registration.md` |
| 3 | `all_tools()` hardcoded vec (360 lines) | Manual bottleneck | Every `.dag` file auto-discovered | Discover | `unified-registration.md` §3 |
| 4 | `GraphBuilderId` string coupling | Meaning outside model (C) | Function pointers from module graph | Discover + Resolve | `consolidation-plan.md` §3 |
| 5 | 13 rendering systems, 5 traits | No emission model (D) | `ProgressManifest` + renderer trait | Derive + Emit | `unified-emission.md` |
| 6 | Hidden env access in opaque nodes | Model not closed (A) | `uses` declarations + compiler-inserted env nodes | Lower | `refactor-pressure.md` §A |
| 7 | `execute_transport()` escape hatch | No construction enforcement (B) | Service ops → compiler-emitted triplets | Lower | `refactor-pressure.md` §B |
| 8 | `format!()` codegen constructing source | No emission IR (D) | TestFile IR + TestRenderer trait | Emit | `architecture-debt.md` |
| 9 | Hermeticity erased at transport layer | Generic IR chokepoint | Service annotations preserved through lowering | Lower + Validate | `consolidation.md` §8 |
| 10 | Boundary mocks defined in two places | Semantic duplication (C) | MockSpec derived from service declarations | Derive | `unified-registration.md` §4 |
| 11 | Resource lifecycle implicit | Incomplete resource model | `resource` with acquire/use/release | Lower + Validate | `dsl-design.md` §7 |
| 12 | Progress rebuilt from scratch (lost TUI quality) | No progress model | `ProgressManifest` at compile time | Derive | `dsl-design.md` §6 |
| 13 | dag-viz can't see itself | Discovery doesn't include meta-tools | Module graph IS workspace DAG | Discover | `dsl-design.md` §5 |
| 14 | Manual MockSpec per tool | Test infrastructure not generated | Compiler generates MockSpec from service declarations | Derive | `dsl-design.md` §8 |
| 15 | `Value`/`ValueExpr` parallel hierarchies | Emission leaked into IR | Codegen works from IR + types; no runtime value expressions | Emit | `architecture-debt.md` |

## K.6 Guardrails for the DSL (Preventing Re-Creation of Failure Modes)

The feedback raises an important concern: how do we prevent re-creating gunbc's failure modes *inside* the DSL? Three specific guardrails:

### G1: Annotations must desugar to structure

Design Principle P9 ("The language is total") and the IR philosophy's ban on semantic meta-annotations must apply to the DSL surface syntax. `@interactive`, `@rest`, `@idempotent`, etc. must **desugar into explicit structural nodes/fields** in the lowered IR, not remain as opaque annotations that modify behavior outside the model.

**Test:** Can you delete the annotation and get a compile error or behavior change that's visible in the IR? If the annotation has no structural representation, it's a semantic meta-annotation and violates P9.

### G2: Preserve producer-level semantics through lowering

Hermeticity is the canonical example. If the DSL keeps a generic transport layer (it does — Design Principle P5), it must carry semantic properties from the service declaration through compilation and execution.

**Options from `consolidation.md`:**
- Field on `TransportRequest` (e.g., `hermetic: bool`)
- Split `Shell` variant into `LocalShell` / `NetworkShell`
- Node-level annotation that survives lowering

**Recommendation:** The DSL compiler should propagate `@rest` / `@shell` / `@idempotent` / `@readonly` / `@permissions` as metadata on the lowered transport node, so the executor and test categorizer can access them without re-deriving them from string inspection.

### G3: Kill manual bottlenecks first

The internal docs identify `all_tools()` as the #1 source of silent omission bugs. The DSL's filesystem discovery eliminates this entirely — but the Phase 1 implementation must verify this is true end-to-end: every `.dag` file in the `paths` directories must appear in the module graph, and the module graph must be the sole source for downstream automation (Makefile targets, CLI generation, testgen registration).

**Metric (from `refactor-pressure.md`):**
- Manual tool registrations → 0
- Stringly `GraphBuilderId` references → 0
- Rendering systems without IR/trait → 0
- `format!()` constructing source code → 0

---

# Appendix L: A/B Workflow Comparisons and Handbook Reference

This appendix consolidates key material from `docs/ab-writing-workflows.md` and `docs/handbook.md` to serve as a self-contained reference within this design document.

## L.1 The A/B Comparison Framework

The A/B comparison shows the same workflow written in three traditional styles (imperative, OO, functional) and then as a gunbc DAG, highlighting what each approach proves and what must be tested manually.

**Core difference:** Traditional code makes workflow structure implicit (ordering, wiring, I/O boundaries, skip paths live in control flow and conventions). gunbc makes the workflow explicit as a typed DAG (wiring, boundaries, and dataflow are first-class objects).

### What gunbc validation proves that traditional compilers cannot:

| Property | Rust/Java/Haskell compiler proves | gunbc additionally proves |
|---|---|---|
| Type safety | Types match at call sites | Types match at DAG edges (across node boundaries) |
| Acyclicity | N/A (not a concept) | DAG structure is acyclic by construction |
| Cardinality | N/A (not a concept) | `One`/`ZeroOrOne`/`ZeroOrMore` are compatible across edges |
| SubDag interfaces | N/A | Inner DAG ports match outer node usage |
| I/O isolation | N/A | All I/O goes through transport boundaries (DryRun intercepts them) |
| Resource conflicts | N/A | No unordered accesses to the same resource |

## L.2 Minimal Tool: Clippy Upsert (Four Styles)

### Imperative (Rust)

```rust
fn run_clippy(args: &[&str]) -> Result<()> {
    if !clippy_installed()? {
        install_clippy()?;
    }
    run_clippy_command(args)?;
    Ok(())
}
```

You must ensure manually: the check/install/run wiring is consistent and complete, the "already installed" fast path is correct.

### gunbc DAG

```rust
pub fn build_clippy_upsert(args: &[&str]) -> Node<CliToolOp> {
    build_cli_upsert(&cli::CLIPPY, args)
}
```

Produces a SubDag:

```
+------------------------- clippy (SubDag) --------------------------+
| [check]   (is clippy installed?)                                   |
|    | exists = false                                                |
| [create]  (rustup component add clippy)    [guard: !check.exists]  |
|    |                                                               |
| [resolve] (cargo clippy {args...})                                 |
|    ^                                                               |
|    +-- exists = true: skip create ---------------------------------+
+--------------------------------------------------------------------+
```

What gunbc proves beyond the Rust compiler:
- The upsert flow is acyclic and structurally complete
- All edges are type-compatible and cardinality-compatible
- The SubDag interface matches how the parent graph uses it

### DSL

```
journey install_clippy {
  output { handle: String }
  result = upsert {
    check:   shell("which clippy-driver") -> { exists: Bool }
    create:  shell("rustup component add clippy")
    resolve: shell("clippy-driver --version") -> { handle: String }
  }
  return { handle: result.handle }
}
```

What the DSL adds beyond gunbc: the 5-line journey replaces ~186 lines of Rust builders. The compiler expands `upsert` to the same SubDag, with the same structural guarantees, plus generates MockSpec and test obligations automatically.

## L.3 Real Workflow: Gist Snapshot (Four Styles)

### Imperative (Rust)

```rust
fn run_gist_snapshot(repo_path: &Path, public: bool) -> Result<String> {
    let files = git_ls_files(repo_path)?;
    let mut contents = BTreeMap::new();
    for file in files {
        let text = std::fs::read_to_string(repo_path.join(&file))?;
        contents.insert(file, text);
    }
    let markdown = render_code_snapshot(&contents);
    let branch = git_current_branch(repo_path)?;
    let remote_branch = git_remote_branches_at_head(repo_path)?;
    let request = prepare_gist_request(markdown, branch, remote_branch, None, public)?;
    let response = execute_transport(request)?;
    let url = parse_gist_response(response)?;
    Ok(url)
}
```

You must ensure manually: workflow is acyclic and complete, I/O boundaries are isolated/mocked/tested, optional inputs and skip paths are handled consistently.

### gunbc DAG (simplified)

```
+------------------ List + Read ------------------+
| prepare_list_files → execute_list_files → parse  |
| parse → read_files_loop (SubDag)                 |
| loop → collect → render_markdown                 |
+--------------------------------------------------+

+------------------ Branch Inputs ------------------+
| prepare_branch → execute_branch → parse_branch    |
| prepare_remote → execute_remote → parse_remote    |
+---------------------------------------------------+

[fs_env] ──fs:write──→ prepare_gist_request
[clock_env] ──clock──→ prepare_gist_request
render_markdown ──markdown──→ prepare_gist_request
prepare_gist_request → execute_gist → parse_gist_response → url
```

1,449 lines of Rust builder code. Every transport node (execute_list_files, execute_branch, execute_remote, execute_gist, execute_read_file) is interceptable by DryRun. Testgen generates ~50 tests from the graph structure.

### DSL

```
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
  result = gist_upload(markdown: markdown, branch: branch.branch, base_ref: base_ref)

  return { url: result.url }
}
```

~80 lines across 3 journeys (snapshot + diff + recent) vs 1,449 lines of Rust builders. Same IR, same tests, same DryRun behavior.

## L.4 Handbook Pattern Catalog (Reference)

The following patterns from the gunbc handbook (`docs/handbook.md`) are preserved here as reference for the DSL's pattern library.

### Structural Patterns

| Pattern | Intent | gunbc Builder | DSL Equivalent |
|---|---|---|---|
| **Fractal SubDag** | Reusable subgraphs as nodes | `Node::subdag(inner_dag)` | Journey/pattern call (implicit SubDag) |
| **Upsert** | Check → Create → Resolve | `UpsertBuilder::new(id)` | `upsert { check: ..., create: ..., resolve: ... }` |
| **Transaction** | Begin → Body → Commit/Rollback | `TransactionBuilder` | `transaction { begin: ..., body: ..., commit: ..., rollback: ... }` |
| **Atomic** | Precondition → Op → Postcondition | `AtomicBuilder` | `atomic { pre: ..., op: ..., post: ... }` |
| **Content Upsert** | Render → Read → Compare → Write | `add_content_upsert_chain()` | `content_upsert(content: ..., path: ...)` |
| **Branch** | Conditional execution with merge | `BranchBuilder` / `IfBuilder` | `match cond { A => ..., B => ... }` / `when flag { ... }` |
| **Loop** | Iterate over collections | `LoopBuilder` | `for item in collection { ... }` |
| **Retry** | Re-execute on failure | `RetryBuilder` | `@retry(max: 3, backoff: exponential(1000))` |
| **While** | Loop while condition holds | `WhileBuilder` | `while cond { ... }` |
| **Poll** | Periodic execution until success | `PollBuilder` | `poll(interval: 5s, timeout: 60s) { ... }` |

### System Patterns

| Pattern | Intent | gunbc Implementation | DSL Equivalent |
|---|---|---|---|
| **Transport Boundary** | All I/O through request/response | `TransportOps::Execute` node | Service call with `@rest`/`@shell` → compiler-emitted triplet |
| **Registration** | Auto-discovery of registrable units | `#[testgen_target]` + inventory | Filesystem discovery (`.dag` files) |
| **Emission** | IR → Renderer → Output | `TestFile` IR + `TestRenderer` trait | Compiler `Emit` phase with `CodegenBackend` trait |
| **Resource Acquisition** | Typed resources with conflict detection | `ResourceAccess` + `detect_conflicts()` | `uses` declarations + compiler-inserted lifecycle |
| **Credential Lifecycle** | Provider → acquire → Credential | `CredentialProvider` trait | `resource Credential { ... }` with lifecycle |
| **Mock Specification** | Declarative test fixtures | `MockSpec::new(name).boundary(...)` | Compiler-generated from service declarations |
| **Content Hashing** | Deterministic content-addressed hashing | `gunbc-infra::hash` | Standard library utility |
| **Freshness Check** | Mtime fast-path before full hash | `gunbc-infra::freshness` | Compiler-managed (emitted with content_upsert) |

## L.5 End-to-End Pipeline (from Handbook Appendix B)

The complete lifecycle of a tool from definition to generated tests, preserved here as reference for the DSL compiler's target output.

### Pipeline (gunbc current)

```
+-------------------------------+
| 1. Define DAG                 |   graph.rs
|    Node::opaque / DagBuilder  |   prepare → execute → parse
+-------------------------------+
              |
              v
+-------------------------------+
| 2. Write MockSpec             |   graph_mock.rs
|    extract_mock_requirements  |   type-checked against DAG structure
|    .boundary() / .transport() |
+-------------------------------+
              |
              v
+-------------------------------+
| 3. Register with testgen      |   #[testgen_target(name, output, builder)]
|    proc macro + inventory      |   auto-discovered at build time
+-------------------------------+
              |
              v
+-------------------------------+
| 4. Analyze DAG                |   analyze.rs
|    boundaries, transport,     |   structural facts + proof obligations
|    cardinalities, resources   |
+-------------------------------+
              |
              v
+-------------------------------+
| 5. Generate tests             |   codegen.rs
|    TestGenerator + buckets    |   A: execution, B: contracts,
|    (only for Unknown proofs)  |   C: scenarios, D: resources
+-------------------------------+
              |
              v
+-------------------------------+
| 6. Output: generated_tests.rs |   content-hash header
|    make testgen regenerates   |   50-150+ tests per DAG
+-------------------------------+
```

### Pipeline (DSL target)

```
+-------------------------------+
| 1. Author .dag file           |   tools/format.dag
|    journey + service + pattern |   5-50 lines
+-------------------------------+
              |
              v
+-------------------------------+
| 2. Discover                   |   Filesystem scan
|    Module graph built from    |   All .dag files in paths
|    project manifest           |
+-------------------------------+
              |
              v
+-------------------------------+
| 3. Parse → Resolve → TypeCheck|   AST → Resolved AST → Typed AST
|    Imports linked, names      |   Types validated, resources checked
|    resolved                   |
+-------------------------------+
              |
              v
+-------------------------------+
| 4. PatternExpand → Lower      |   patterns → sub-DAG templates
|    Service calls → triplets   |   resources → lifecycle nodes
|    Implicit edges → explicit  |   for → LoopBuilder, match → Branch
+-------------------------------+
              |
              v
+-------------------------------+
| 5. Validate                   |   SPEC.md invariants
|    + resource conflicts       |   + hermeticity classification
+-------------------------------+
              |
              v
+-------------------------------+
| 6. Derive                     |   ProgressManifest
|    + TestObligations          |   + MockSpec (from service decls)
|    + ToolMetadata             |
+-------------------------------+
              |
              v
+-------------------------------+
| 7. Emit (per backend)         |   Type definitions
|    Node stubs (developer fills)|   Transport wiring
|    Test harness (4-bucket)    |   CLI entrypoint
|    Progress renderer          |   Makefile / CI YAML
+-------------------------------+
```

**Key difference:** Steps 2-3 in the gunbc pipeline (MockSpec + Registration) are manual. In the DSL pipeline, they're compiler-derived (steps 6-7). The developer writes step 1 only.

## L.6 Generated Test Obligation Example (from Handbook)

For the LLM credential lifecycle graph (5 nodes), testgen produces:

**Header:**
```rust
// Generated tests for llm_credential_lifecycle_generated_tests DAG.
// Obligations: 23 obligations (9 discharged, 14 testable: A=6, B=5, C=3, D=0)
// Content-Hash: 04affa725267b9dd...
```

**Bucket A — Execution Semantics:**
```rust
#[test]
fn test_dryrun_completion() {
    let dag = build_chat_completion_graph();
    let log = execute_with_mode(&dag,
        ExecutionMode::DryRun(mock_spec().to_boundary_mocks()))
        .expect("DryRun should complete without crash");
    assert!(!log.entries.is_empty());
}
```

**Bucket C — Scenario Coverage:**
```rust
#[test]
fn test_scenario_all_succeed() {
    let dag = build_chat_completion_graph();
    let log = execute_with_mode(&dag,
        ExecutionMode::DryRun(mock_spec().to_boundary_mocks()))
        .expect("all-succeed scenario should complete");
    let entry = log.get("execute").expect("'execute' should be in log");
    assert!(entry.was_intercepted);
}

#[test]
fn test_skip_propagation_execute() {
    let dag = build_chat_completion_graph();
    let mut mocks = mock_spec().to_boundary_mocks();
    mocks.set_value("execute", "response", Value::Skipped);
    let log = execute_with_mode(&dag, ExecutionMode::DryRun(mocks))
        .expect("skip propagation should not crash");
    assert!(log.get("parse").is_some());
}
```

**Obligation stats for the CI graph (largest):**
```
Obligations: 133 obligations (58 discharged, 75 testable: A=27, B=30, C=16, D=2)
Proven by construction: acyclicity, type compatibility, cardinality satisfaction.
```

58 obligations proven statically (no test needed), 75 tests generated across 4 buckets. In the DSL, the same obligation model runs against the compiler's output — identical tests, but derived from a 10-100x smaller source.

## L.7 Consolidation Status (Living Reference)

Current status of the six work streams from `docs/design/consolidation-plan.md`, included here so the DSL design can track which gunbc issues are already fixed vs still need to be addressed by the language.

| Stream | Problem | Status | DSL Eliminates? |
|---|---|---|---|
| **1. Registration Unification** | 6 registration islands, manual `all_tools()` vec | R1-R2 complete (tool-registry crate + annotations) | Yes — filesystem discovery |
| **2. Emission Unification** | 13 rendering systems, 5 traits, 8 with no trait | Design complete, implementation not started | Yes — `Emit` phase with `CodegenBackend` |
| **3. String-Coupled Dispatch** | `GraphBuilderId::as_str()` breaks at runtime | Absorbed by Stream 1 Phase R2 | Yes — function pointers from module graph |
| **4. Documentation Consistency** | Handbook contradictions (e.g., "I/O enforcement complete" vs migration table) | Pending | Yes — this document IS the consolidated doc |
| **5. CI Verification Gaps** | `make verify` not in CI, generated files not verified | `make verify` exists, not yet in CI | Yes — compiler verifies `.dag` → generated output |
| **6. CliToolDef/ToolDef Alignment** | Two tool types with field overlap | Intentional separation, no action | N/A — DSL has `resource` + service ops |

## L.8 Reference: Key File Paths

For navigating between this design doc and the source material it consolidates:

| Document | Path | Key Content |
|---|---|---|
| DSL Design (this doc) | `docs/design/dsl-design.md` | Language spec, all appendices |
| Handbook | `docs/handbook.md` | Pattern catalog, E2E examples, repo map |
| A/B Workflows | `docs/ab-writing-workflows.md` | Imperative/OO/functional vs gunbc DAG comparisons |
| Design Overview | `docs/design/overview.md` | Philosophy, invariants, formal model |
| Testgen | `docs/design/testgen.md` | Obligation model, 4 buckets, anti-tautology rule |
| Unified Registration | `docs/design/unified-registration.md` | 6 registration islands → unified discovery |
| Unified Emission | `docs/design/unified-emission.md` | 13 rendering systems → OutputMedium hierarchy |
| Consolidation Plan | `docs/design/consolidation-plan.md` | 6 work streams, reconciliation status |
| Refactor Pressure | `TODO/TODONE/refactor-pressure.md` | Root causes A-D, decision rules, quick scans |
| Architecture Debt | `TODO/TODONE/architecture-debt.md` | Meta-root-cause, leak→fix table |
| IR Spec | `SPEC.md` | Formal IR specification |
| Agent Guide | `AGENT.md` | Onboarding, guardrails |

---

# Appendix M: Competitive Landscape and Alternatives Analysis

This appendix positions the DSL against the real alternatives people use to avoid hand-wiring graphs and integrations, identifies gaps, and documents paths-not-taken.

## M.1 What the DSL Actually Competes With

The DSL is not competing with Lombok or ORMs directly. It competes with three alternative ways people avoid hand-wiring:

| Category | Examples | What they do |
|---|---|---|
| **Host-language metaprogramming** | Java annotation processors / Lombok, Rust proc-macros, Python decorators | Generate boilerplate within one language |
| **Runtime orchestration frameworks** | LangGraph, LangChain, CrewAI, Temporal, Airflow, Dagster | Execute workflows at runtime with framework conventions |
| **IDLs that generate clients + models** | OpenAPI, Smithy, protobuf/gRPC, Thrift | Describe interfaces, generate multi-language code |

The DSL is a **hybrid of IDL + workflow compilation**: service/operation/type declarations (like Smithy/protobuf) plus workflow compilation into a typed DAG IR with transport boundaries, test derivations, and progress manifests. This hybrid is the distinguishing position — neither pure IDL nor pure orchestrator.

## M.2 What the DSL Provides That Alternatives Don't

Four capabilities that are first-class compiler output, not conventions:

| Capability | What it means | Closest alternative | Gap in alternative |
|---|---|---|---|
| **Transport triplets as structural primitive** | Service calls expand to prepare → execute → parse with skip wiring. Authors never see the triplet. | Smithy generates client stubs | No DAG structure, no skip wiring, no DryRun interception |
| **Proof-obligation test generation** | Tests derived from graph properties (4 buckets), discharged structurally or generated. Anti-tautology rule. | Dagster has `@asset` testing, but manual | No structural obligation model, no mechanical derivation |
| **Progress as derived topology view** | ProgressManifest computed at compile time from DAG structure. Renderers are pluggable views. | LangGraph has runtime streaming/tracing | Runtime-only, no compile-time manifest, no multi-renderer architecture |
| **Debuggable execution modes** | DryRun intercepts transport nodes, Simulate with timing. Step-by-step possible. | LangGraph has interrupt/resume | No compile-time boundary classification, no mock-spec derivation |

## M.3 Comparison: Host-Language Metaprogramming (Lombok / Proc-Macros)

### What Lombok provides

Compile-time code generation within Java. `@Data` generates getters/setters/toString/equals. `@Builder` generates a builder API.

```java
@Data @Builder
public class Credential {
    private final String token;
    private final String scheme;
}
```

### Overlap and divergence

| Dimension | Lombok | DSL |
|---|---|---|
| Core mechanism | Declarative metadata → generated code | Declarative metadata → generated code |
| Scope of generation | Methods on classes | Entire workflow program (graph, transports, tests, progress, CLI) |
| Language binding | Java only | Language-agnostic (Rust/Go/Python/TS/MIPS) |
| Side-effect modeling | None | Transport boundaries, resource lifecycle, DryRun interception |
| Test derivation | None | 4-bucket proof obligations from graph structure |

### The Rust proc-macro alternative (path-not-taken)

A Lombok-style approach within Rust would look like:

```rust
#[dag]
fn makegen(registry: ToolRegistry) -> Written {
    let content = render_makefile(registry);
    content_upsert(content, "Makefile")
}
```

The macro would emit the gunbc builder + IR.

**Pros:** Stays in Rust (IDE support, type checking, refactoring tools work). No new parser or type checker to build and maintain.

**Cons:** Kills the biggest goal — multi-target emission and ".dag as contract." A proc-macro is bound to the host language. You cannot emit Go or Python from a Rust proc-macro, and the "contract" is Rust source code, not a language-agnostic manifest.

**Decision:** The proc-macro approach is a valid local optimum if the commitment is "Rust forever." It is a dead-end if ".dag like .proto" is the target. Since Design Principle P10 (Language-agnostic) is a core commitment, the proc-macro path was rejected.

### What to steal from Lombok anyway

Lombok has a first-class "delombok" command that expands annotations to generated source code. The DSL should have an equivalent:

```
dag expand tools/makegen.dag        # show lowered IR
dag show-triplets tools/makegen.dag  # show transport triplet expansion
dag obligations tools/makegen.dag    # show derived test obligations
dag manifest tools/makegen.dag       # show ProgressManifest
```

This "show me what the compiler meant" tooling is essential for debugging and trust. Lomboked code that you can't inspect is frustrating; the same applies to `.dag` compilation.

## M.4 Comparison: ORMs (Hibernate/JPA)

### Overlap

Both use declarative metadata to drive generated/reflective behavior, have a notion of resources and lifecycle (transactions, sessions), and aim to eliminate stringly-typed glue by centralizing metadata.

### Divergence

| Dimension | ORM | DSL |
|---|---|---|
| What's modeled | Data persistence mapping (object graph ↔ relational schema) | Causal execution graphs (dataflow + ordering + boundaries) |
| Correctness guarantee | Runtime exceptions (lazy loading, N+1 queries, schema mismatch) | Compile-time structural proof (acyclicity, type/cardinality compatibility) |
| Code generation | Persistence behavior within one runtime | Multi-language workflow programs |

### ORM lessons that apply to the DSL

ORMs learned (painfully) that declarative metadata needs:

1. **Clear versioning/evolution rules.** `.dag` types and service operations will change over time. The compiler needs a migration/compatibility story. Protobuf's field numbering and wire compatibility rules are the model here (already listed in Appendix I as an inspiration target).

2. **Escape hatches for unusual cases.** Some APIs will not fit `@rest` or `@shell` annotations cleanly. The DSL needs a "manual transport hook" — a way to declare a service operation that the author implements directly rather than having the compiler generate a triplet. Without this, people will work around the compiler, recreating the escape-hatch problem from gunbc (Appendix K, Root Cause B).

3. **"Show me the generated SQL" equivalence.** ORM users constantly need to see what SQL the framework generates. DSL users will constantly need to see what IR the compiler produces. The `dag expand` / `dag show-triplets` commands from M.3 address this.

## M.5 Comparison: LangGraph

LangGraph is the closest real alternative. Both model workflows as graphs with explicit nodes and edges.

### LangGraph `review_diff` (concrete Python)

```python
from typing_extensions import TypedDict
from langgraph.graph import StateGraph, START, END

class State(TypedDict, total=False):
    base_ref: str
    diff: str
    prompt: list[dict]
    review: str

def get_diff(state: State) -> dict:
    return {"diff": run_git_diff(state["base_ref"])}

def build_prompt(state: State) -> dict:
    return {"prompt": [
        {"role": "system", "content": "You are a code reviewer."},
        {"role": "user", "content": state["diff"]},
    ]}

def call_llm(state: State) -> dict:
    resp = ChatOpenAI(model="gpt-4", temperature=0.3).invoke(state["prompt"])
    return {"review": resp.content}

builder = StateGraph(State)
builder.add_node("get_diff", get_diff)
builder.add_node("build_prompt", build_prompt)
builder.add_node("call_llm", call_llm)
builder.add_edge(START, "get_diff")
builder.add_edge("get_diff", "build_prompt")
builder.add_edge("build_prompt", "call_llm")
builder.add_edge("call_llm", END)

graph = builder.compile()
out = graph.invoke({"base_ref": "origin/main"})
```

### DSL `review_diff`

```
journey review_diff {
  input { base_ref: String }
  output { review: String }
  uses net: Network

  diff = git.Core.Diff(base: base_ref)
  prompt = build_review_prompt(diff: diff.diff)
  result = llm.OpenAI.ChatCompletion(messages: prompt)

  return { review: result.content }
}
```

### Where each is stronger

| Dimension | DSL | LangGraph |
|---|---|---|
| Graph typing | Explicit typed ports, cardinality, compile-time validation | Shared state dict with optional type hints (TypedDict) |
| I/O boundaries | Structural: service calls → transport triplets, DryRun intercepts | Convention: side effects live in node functions, mocking is manual |
| Test derivation | Compiler-derived: 4-bucket obligations, MockSpec from service declarations | Manual: write your own tests and mocks |
| Progress | Compile-time ProgressManifest → pluggable renderers | Runtime streaming/tracing callbacks |
| Multi-language | Core goal: Rust/Go/Python/TS backends | Python and TypeScript only |
| **Durability/HITL** | **Not core (yet)** | **Core feature: interrupt/resume, checkpointing, human approval** |
| **Dynamic fan-out** | `for` loops in IR, scatter groups | `Send` objects for map-reduce, reducer annotations |
| **Agentic patterns** | Not modeled (deterministic orchestration) | First-class: tool calling, dynamic routing, memory |

### The durability/HITL gap

LangGraph's first-class durability semantics (interrupt at any node, persist state, resume later, human-in-the-loop approval patterns) represent a genuine capability the DSL does not currently address.

**Options:**

1. **Ignore it.** The DSL targets deterministic workflow orchestration, not agentic HITL systems. Different problems.

2. **Model it structurally.** Add a `@durable` annotation on journeys that compiles to checkpointing infrastructure (state serialization at each transport boundary, resume from checkpoint). This extends the resource model — durable state becomes a resource with lifecycle.

3. **Treat LangGraph as an execution backend.** The DSL's Python codegen backend could emit LangGraph `StateGraph` code instead of raw Python. This borrows LangGraph's runtime (durability, HITL, tracing) while keeping the DSL's compile-time guarantees (typed ports, test derivation, progress manifest). The `.dag` file remains the contract; LangGraph is the execution engine.

**Recommendation:** Option 3 is the strongest near-term path. It preserves the DSL's unique value (typed, portable, test/progress derivations) while borrowing runtime capabilities where LangGraph is genuinely better. This aligns with Design Principle P10 (Language-agnostic) — execution backends are plugins.

## M.6 Comparison: LangChain (LCEL)

LangChain's core abstraction is `RunnableSequence` — output of each step feeds the next, with sync/async/batch.

### LangChain LCEL `review_diff`

```python
from langchain_core.runnables import RunnableLambda
from langchain_core.prompts import ChatPromptTemplate
from langchain_openai import ChatOpenAI

diff_fn = RunnableLambda(lambda inp: run_git_diff(inp["base_ref"]))
prompt = ChatPromptTemplate.from_messages([
    ("system", "You are a code reviewer."),
    ("user", "{diff}")
])
llm = ChatOpenAI(model="gpt-4", temperature=0.3)

chain = (
    {"diff": diff_fn, "base_ref": RunnableLambda(lambda x: x["base_ref"])}
    | prompt
    | llm
)
result = chain.invoke({"base_ref": "origin/main"})
```

### Assessment

LangChain is optimized for **mostly-linear LLM pipelines** with some branching. It provides no concept of:
- Compile-time structural proof
- Topology-derived progress
- Test obligation derivation
- Multi-language codegen

The DSL subsumes LangChain's capabilities: any linear chain is expressible as a journey, and the compiler provides strictly more guarantees. LangChain's value is in its ecosystem (hundreds of integrations, prompt templates, output parsers) — which the DSL could consume via service declarations wrapping LangChain's existing connectors.

## M.7 Comparison: CrewAI

CrewAI models workflows as **agents + tasks + processes** rather than explicit DAG edges:

```python
from crewai import Agent, Task, Crew

agent = Agent(
    role="Code Reviewer",
    goal="Review diffs for bugs and style issues",
    backstory="You are a senior engineer.",
)
task = Task(description="Review the diff in {diff}", agent=agent)
crew = Crew(agents=[agent], tasks=[task])
result = crew.kickoff(inputs={"diff": diff_text})
```

### Assessment

CrewAI is designed for **agentic, non-deterministic workflows** where an LLM decides next steps dynamically. The DSL is designed for **deterministic orchestration** where the graph structure is known at compile time.

| Dimension | DSL | CrewAI |
|---|---|---|
| Execution model | Deterministic: graph structure fixed at compile time | Non-deterministic: agent decides tool use and delegation |
| Dataflow | Typed ports and edges | Context passed between tasks (untyped) |
| Test strategy | Structural obligations + DryRun | Manual tests (framework support evolving) |
| Progress | Compile-time manifest | Verbose logs/streaming |

CrewAI is the right choice when the system is primarily **agentic** (delegation, memory, non-deterministic tool choice). The DSL is the right choice when the system is primarily **deterministic orchestration + codegen**.

These are complementary, not competing: a DSL journey could invoke a CrewAI agent as a service operation (`@crewai` annotation on a service), treating the non-deterministic agent call as a transport boundary that the DSL models structurally.

## M.8 Side-by-Side Summary

| Dimension | `.dag` DSL + compiler | LangGraph | LangChain | CrewAI |
|---|---|---|---|---|
| Authoring | Declarative `.dag` + patterns | Python/TS graph builder | Python/TS runnable composition | YAML/Python agents + tasks |
| Graph semantics | Explicit typed DAG IR, compile-time invariants | Stateful graph over shared state | Mostly linear chains | Task process model (seq/hierarchical) |
| Dynamic fan-out | `for` loops → scatter groups in IR | `Send` map-reduce patterns | Manual Python loops | `kickoff_for_each` + custom code |
| Progress model | Derived manifest + pluggable renderers | Runtime streaming/tracing | Callbacks/tracing | Verbose logs/streaming |
| Testing | Structural obligations + DryRun intercept | Manual tests + mocks | Manual tests + mocks | Manual tests |
| Multi-language | Explicit goal (Rust/Go/Python/TS/MIPS) | No (Python/TS only) | Partial (Python/JS) | No (Python only) |
| Durability/HITL | Not core (addressable via LangGraph backend) | Core feature set | Not core | Available via processes |
| Agentic patterns | Not modeled (invoke as service boundary) | First-class | First-class | Core design |

## M.9 The "Are We Going in the Right Direction?" Test

**Build the DSL if** the goal is:
- Compress authoring (stop writing 7,000+ lines of builders)
- Keep structural guarantees (acyclicity, typing, saturation)
- Keep transport boundaries mockable (DryRun / interception)
- Keep or improve progress UX (topology-derived rendering)
- Support multi-target emission
- Make the workflow description a portable contract (".dag like .proto")

**Use LangGraph/Temporal directly if** the goal is:
- Long-lived, stateful, human-in-the-loop agent platform
- LLM decides next steps dynamically
- Checkpointing/resume as core requirement
- Python/TS ecosystem integration is more important than multi-language codegen

**The reconciliation:** These are not mutually exclusive. The `.dag` file is the canonical contract. Execution backends are pluggable. A LangGraph backend for Python emission would borrow durability/HITL capabilities while preserving the DSL's compile-time guarantees. A Temporal backend would provide distributed execution. The DSL's value is in what happens *before* execution: structural proof, test derivation, progress manifests, multi-target code generation.

## M.10 Gaps and Tooling Needs Identified

From the competitive analysis, three concrete gaps and tooling needs:

### Gap 1: Durability/HITL

**Current state:** Not addressed.
**Recommendation:** Model as execution backend plugin (M.5 Option 3). LangGraph backend for Python. For Rust, consider Temporal-compatible codegen.
**Timeline:** Phase 4+ (after core language + multi-target emission).

### Gap 2: Schema evolution

**Current state:** Not addressed. ORM lesson (M.4): types and service operations will change.
**Recommendation:** Adopt protobuf-style compatibility rules. Service operations must have stable wire format. Type fields can be added (with defaults) but not removed or retyped without a version bump.
**Timeline:** Phase 2 (when services are introduced).

### Gap 3: "Show me what the compiler meant" tooling

**Current state:** Not addressed. Lombok and ORM lesson: users need to see generated output.
**Recommendation:** First-class CLI commands:

```
dag expand <file.dag>          # show lowered GraphIR (Node/Dag/Port/Edge)
dag show-triplets <file.dag>   # show transport triplet expansion for each service call
dag obligations <file.dag>     # show derived TestObligations by bucket
dag manifest <file.dag>        # show ProgressManifest (waves, groups, scatter points)
dag viz <file.dag>             # ASCII DAG visualization (pre-execution)
```

**Timeline:** Phase 1 (essential for trust and debugging from day one).

### Gap 4: Escape hatch for unusual APIs

**Current state:** Not addressed. ORM lesson: some APIs won't fit `@rest` / `@shell`.
**Recommendation:** A `@custom` transport annotation that tells the compiler "I will implement this transport myself." The compiler still emits the triplet structure (prepare/execute/parse) and generates test obligations, but the execute node delegates to a developer-provided function instead of a generated transport executor.

```
service unusual.Api {
  operation WeirdCall {
    input { payload: Bytes }
    output { result: Json }
    @custom("my_transport_impl")  // developer implements the execute step
  }
}
```

This preserves structural guarantees (the triplet exists, DryRun can intercept it, tests are generated) while allowing escape from the annotation-driven transport generation.

**Timeline:** Phase 2 (when services are introduced).

---

# Appendix N: Model-Based Testing and Auto-Generated Mocks

This appendix describes how the DSL's type system, service declarations, and testgen model combine to enable **model-based testing**: the compiler generates not just test *structure* (which tests to run) but test *data* (what mock values to use), eliminating the manual `MockSpec` fixture burden that accounts for a significant portion of gunbc's per-tool authoring cost.

## N.1 The Current State: MockSpec Is Half-Automated

In gunbc, `extract_mock_requirements()` automatically determines *what needs to be mocked* by analyzing the DAG structure — which nodes are transport boundaries, what output ports they have, what types those ports declare. This is the structural half.

But the developer still supplies the concrete *values*:

```rust
// gunbc today: structure is derived, values are manual
MockSpec::new("gist")
    .boundary("fs_env", "fs:write", mock_fs_handle())          // manual value
    .transport_response("execute_gist", "response",
        TransportResponse::Rest(mock_gist_response_json()))     // manual value
    .boundary_str("parse_gist_response", "url",
        "https://gist.github.com/mock/abc123")                  // manual value
```

For a typical tool with 5 transport nodes, this is 20-40 lines of hand-written mock fixtures. For the full repo (8+ tools), it's hundreds of lines that must be maintained in sync with service APIs.

## N.2 The Goal: Compiler-Generated Mock Values

The DSL should close the gap so the developer workflow becomes:

```
1. Write the .dag file
2. Write the pure transformation logic (5% of code)
3. Stop.
```

The compiler:
- Builds the graph (Parse → Lower)
- Proves structural soundness (Validate)
- Derives test obligations (Derive — already designed)
- **Generates mock values from type constraints and service annotations** (Derive — new)
- Generates test harnesses with those values (Emit — already designed)
- Fuzzes pure nodes with generated inputs (Emit — new)

## N.3 Three Tiers of Auto-Generation

### Tier 1: Type-Driven Generation (Refinement Types)

When a type has refinement constraints, the compiler can generate valid and invalid values mechanically.

```
type CommitSha = String @pattern("^[a-f0-9]{40}$")
type RetryCount = Int @range(min: 1, max: 5)
type GistId = String @format(uuid)
type HttpStatus = Int @range(min: 100, max: 599)
```

The compiler's type-aware generator produces:

| Type | Valid examples | Edge cases | Invalid examples |
|---|---|---|---|
| `CommitSha` | `"a" * 40`, random hex strings | Empty string, 39 chars, 41 chars | `"ZZZZ..."`, non-hex chars |
| `RetryCount` | 1, 3, 5 | 1 (min), 5 (max) | 0, 6, -1, MAX_INT |
| `GistId` | Random UUIDs | Nil UUID (`00000000-...`) | Empty string, malformed |
| `HttpStatus` | 200, 404, 500 | 100 (min), 599 (max) | 99, 600, 0, -1 |

For unconstrained primitives (`String`, `Int`, `Bool`), the compiler uses safe defaults:

| Primitive | Default valid | Default edge cases |
|---|---|---|
| `String` | `"test_value"` | `""`, very long string |
| `Int` | `42` | `0`, `MAX_INT`, `MIN_INT` |
| `Bool` | `true` | `false` |
| `Bytes` | `[0x00]` | `[]`, large buffer |
| `Json` | `{}` | `null`, deeply nested |
| `Secret` | `Secret("mock_secret")` | `Secret("")` |

Records are generated by composing field generators:

```
type Credential {
  token: Secret
  scheme: AuthScheme
  expires_at: String?
}

// Compiler generates:
// valid: Credential { token: Secret("mock"), scheme: Bearer, expires_at: None }
// valid: Credential { token: Secret("mock"), scheme: Bearer, expires_at: Some("2026-01-01") }
// edge:  Credential { token: Secret(""), scheme: Header { name: "" }, expires_at: None }
```

### Tier 2: Service-Driven Generation (`@mock_response`)

For transport boundaries, type constraints alone aren't sufficient. A randomly generated JSON string won't parse as a valid GitHub API response. The `@mock_response` annotation provides the semantic template:

```
service github.Gist {
  operation Create {
    input {
      description: String
      files: Map<String, String>
      public: Bool = false
    }
    output { url: String, id: GistId }
    @rest(POST, "https://api.github.com/gists")
    @mock_response(
      status: 201,
      body: { "html_url": "https://gist.github.com/mock/{id}", "id": "{id}" }
    )
  }
}
```

The compiler:
1. Sees `@mock_response` on the operation
2. Generates a `GistId` value using the refinement type generator (`String @format(uuid)` → `"550e8400-e29b-41d4-a716-446655440000"`)
3. Interpolates into the template: `{ "html_url": "https://gist.github.com/mock/550e8400-...", "id": "550e8400-..." }`
4. Wraps as `TransportResponse::Rest(RestResponse { status: 201, body: ... })`
5. Wires into the generated `MockSpec` for this operation's execute node

The result: **zero hand-written mock fixtures** for operations that have `@mock_response`.

For operations *without* `@mock_response`, the compiler falls back to Tier 1 type-driven generation for the output fields. This works for simple cases (`output { branch: String }` → mock value `"mock_branch"`) but may not produce semantically meaningful responses for complex APIs.

**Design choice:** `@mock_response` is optional. Operations without it still get structurally correct tests (DryRun completion, transport interception) using type-derived fallback values. Operations *with* it get semantically correct tests (the parse node receives a realistic response and can exercise the happy path).

### Tier 3: Property-Based Fuzzing (Pure Nodes)

Pure nodes (prepare, parse, render) are deterministic functions with no side effects. They are ideal targets for property-based fuzzing: generate thousands of valid inputs, verify the node never panics, and optionally verify output invariants.

The compiler can isolate every pure node and fuzz it:

```
// For a pure node:
//   input { diff: String, system: String? }
//   output { messages: List<Message> }

// Compiler generates:
#[test]
fn fuzz_build_review_prompt() {
    use proptest::prelude::*;
    proptest!(|(
        diff in ".*",
        system in proptest::option::of(".*"),
    )| {
        let inputs = inputs! { "diff" => Value::Str(diff), "system" => opt(system) };
        let result = execute_single_node("build_review_prompt", inputs);
        // Property: never panics
        prop_assert!(result.is_ok());
        // Property: output has correct shape
        let outputs = result.unwrap();
        prop_assert!(outputs.contains_key("messages"));
    });
}
```

When the input ports have refinement types, the fuzzer respects them:

```
// input { sha: CommitSha, count: RetryCount }
// CommitSha = String @pattern("^[a-f0-9]{40}$")
// RetryCount = Int @range(min: 1, max: 5)

// Compiler generates:
proptest!(|(
    sha in "[a-f0-9]{40}",          // from @pattern
    count in 1..=5i64,               // from @range
)| {
    // ...
});
```

This closes the gap for **Bucket B (Contract Obligations)**: `NodeContractCompliance` evolves from "does the node produce correct output for one example?" to "does the node produce correct output for *any* valid input?"

## N.4 The Syntactic vs Semantic Fuzzing Boundary

There is a hard line between what auto-fuzzing can and cannot do:

| Tier | What it tests | Fully automated? | Requires |
|---|---|---|---|
| **Tier 1: Type-driven** | Pure nodes don't panic on valid/invalid inputs | Yes | Refinement types on input ports |
| **Tier 2: Service-driven** | Happy-path pipeline with realistic API responses | Yes | `@mock_response` on service operations |
| **Tier 3: Property-based** | Output invariants hold for all valid inputs | Partially | Developer-written output invariants (optional) |

**What auto-fuzzing cannot do:**
- Verify business logic correctness (e.g., "this prompt template produces good reviews") — that requires human judgment
- Generate semantically meaningful API responses without `@mock_response` — a random string won't parse as GitHub JSON
- Test network-level failure modes (timeouts, partial responses, rate limiting) — these require explicit `@error_response` annotations (see N.7)

## N.5 Integration with Existing Testgen Buckets

The three tiers map to the four testgen buckets:

| Bucket | Current (gunbc) | With auto-generation (DSL) |
|---|---|---|
| **A: Execution Semantics** | DryRun + transport interception. Values from manual MockSpec. | DryRun + transport interception. Values from `@mock_response` or type-driven fallback. **No manual MockSpec needed.** |
| **B: Contract Obligations** | `NodeContractCompliance` with one example input per node. | Property-based fuzzing: thousands of inputs per pure node, crash-freedom and shape-correctness guaranteed. **Bucket B becomes exhaustive.** |
| **C: Scenario Coverage** | All-succeed, per-failure, guard branches. Values from manual MockSpec. | Same scenarios. Values from `@mock_response`. **Per-failure scenarios can also inject `@error_response` templates.** |
| **D: Resource Hygiene** | Resource connectivity, conflict absence. Values from manual MockSpec. | Same checks. Resource mock values generated from `resource` type definitions. **No manual resource MockSpec needed.** |

## N.6 Compiler Pipeline Integration

The auto-generation work happens in two compiler passes:

### Derive phase (existing, extended)

Currently derives `ProgressManifest` and `TestObligations`. Extended to also derive:

```
type MockManifest {
  boundary_mocks: Map<NodeId, Map<PortName, GeneratedValue>>
  transport_mocks: Map<NodeId, GeneratedTransportResponse>
  resource_mocks: Map<ResourceId, GeneratedResourceValue>
  fuzz_targets: List<FuzzTarget>
}

type GeneratedValue {
  source: TypeDriven | MockResponseTemplate | FallbackDefault
  value: Value
  edge_cases: List<Value>
}

type FuzzTarget {
  node_id: NodeId
  input_generators: Map<PortName, Generator>
  output_invariants: List<Invariant>    // from refinement types on output ports
}
```

### Emit phase (existing, extended)

Currently emits type definitions, transport wiring, test harnesses, CLI. Extended to emit:

- `MockSpec` construction from `MockManifest` (replaces hand-written `graph_mock.rs`)
- Property-based test functions for each `FuzzTarget`
- Edge-case test functions for each refined input port

The generated test file gains new sections:

```rust
// === Auto-generated MockSpec (from @mock_response + type generators) ===
fn mock_spec() -> MockSpec {
    MockSpec::new("gist")
        .boundary("fs_env", "fs:write",
            Value::Map(/* generated from Filesystem resource type */))
        .transport_response("execute_gist", "response",
            TransportResponse::Rest(RestResponse {
                status: 201,
                body: json!({"html_url": "https://gist.github.com/mock/550e8400-...", "id": "550e8400-..."}),
            }))
        .boundary("parse_gist_response", "url",
            Value::Str("https://gist.github.com/mock/550e8400-...".into()))
}

// === Property-based fuzz tests (Bucket B, Tier 3) ===
#[test]
fn fuzz_render_snapshot() {
    proptest!(|(
        files in prop::collection::vec((".*", ".*"), 0..20),
    )| {
        let inputs = inputs! { "files" => Value::List(/* ... */) };
        let result = execute_single_node("render_snapshot", inputs);
        prop_assert!(result.is_ok(), "render_snapshot panicked on valid input");
        let outputs = result.unwrap();
        prop_assert!(outputs.contains_key("markdown"), "missing 'markdown' output");
    });
}
```

## N.7 Error Response Templates (Failure Scenario Mocking)

For Bucket C's per-failure scenarios, the compiler needs to generate realistic error responses. A new `@error_response` annotation provides this:

```
service gcp.SecretManager {
  operation AccessVersion {
    input { project: String, secret: String, version: String = "latest" }
    output { payload: Bytes, name: String }
    @rest(GET, "/v1/projects/{project}/secrets/{secret}/versions/{version}:access")
    @mock_response(
      status: 200,
      body: { "payload": { "data": "bW9ja19zZWNyZXQ=" }, "name": "projects/p/secrets/s/versions/1" }
    )
    @error_response(
      status: 404,
      body: { "error": { "code": 404, "message": "Secret not found", "status": "NOT_FOUND" } }
    )
    @error_response(
      status: 403,
      body: { "error": { "code": 403, "message": "Permission denied", "status": "PERMISSION_DENIED" } }
    )
  }
}
```

The compiler uses `@mock_response` for the all-succeed scenario and `@error_response` for per-failure scenarios. When a transport node is the "failing" node in a Bucket C single-failure test, the compiler injects the error response instead of the success response.

Without `@error_response`, the compiler falls back to a generic transport error (connection refused, timeout) — which tests error propagation but not API-specific error handling.

## N.8 The End-State Developer Workflow

### Without auto-generation (gunbc today)

```
1. Define DAG builder               (~200 lines Rust)
2. Write op enum + implementations  (~80 lines)
3. Write MockSpec with manual values (~40 lines, must match API schemas)
4. Register with testgen             (~10 lines)
5. Write pure node logic             (~50 lines)
   Total manual: ~380 lines
   Total generated: ~60 lines (testgen)
   Manual %: ~86%
```

### With auto-generation (DSL target)

```
1. Write .dag file                   (~20 lines)
   - service declarations with @mock_response (if needed)
   - journey with pattern composition
   - refinement types on domain types (if needed)
2. Write pure transformation logic   (~20 lines Rust/Go/Python)
   Total manual: ~40 lines
   Total generated: ~350+ lines (types, transports, MockSpec, tests, CLI, progress)
   Manual %: ~10%
```

The `MockSpec` moves from a per-tool authoring burden to a compiler output.

## N.9 Relationship to gunbc's Existing Simulator Infrastructure

gunbc already has the seeds of this system in `core/test/`:

| Existing infrastructure | How it evolves in the DSL |
|---|---|
| `Simulator { generator, validator }` | Becomes the runtime representation of refinement type generators |
| `IoContract { input: Map<Simulator>, output: Map<Simulator> }` | Compiler-derived from journey port types + refinement constraints |
| `non_empty_string()`, `boolean()`, `exit_code()`, `int_range()` | Become built-in generator presets mapped from `@non_empty`, `@range`, etc. |
| `MockSpec::node_example(NodeExample { inputs, outputs })` | Compiler generates `NodeExample` from refinement types + `@mock_response` |
| `OutputMatcher::Exact`, `Contains`, `NonEmpty` | Compiler generates matchers from output port refinement types |

The DSL compiler doesn't invent new testing infrastructure — it drives the existing `Simulator` / `IoContract` / `MockSpec` / `OutputMatcher` types from declarative metadata instead of manual construction.

## N.10 Guardrail Compliance

Per Appendix K.6:

**G1 (Annotations must desugar to structure):** Refinement annotations desugar to structural predicates. `@pattern("^[a-f0-9]{40}$")` compiles to a `Predicate::Regex` node in the type's DAG representation. `@range(min: 1, max: 5)` compiles to `Predicate::IntRange { min: 1, max: 5 }`. `@mock_response` compiles to a `MockTemplate` in the `MockManifest`. None of these are opaque metadata that survives into the runtime without structural representation.

**G2 (Preserve producer-level semantics):** `@mock_response` and `@error_response` are producer-level annotations that survive through lowering. The compiler preserves them in the `MockManifest` and uses them during the `Emit` phase to generate test fixtures. They are not erased when the service call is lowered to a transport triplet.

**G3 (Kill manual bottlenecks):** The manual MockSpec is one of the three remaining manual bottlenecks (alongside graph builders and registration). Auto-generation eliminates it, reducing the per-tool manual cost from ~380 lines to ~40 lines.

## N.11 Phasing

| Phase | What's automated | Requires |
|---|---|---|
| **Phase 1** (Language Core) | Type-driven fallback values for `MockSpec` (safe defaults for primitives) | Type system (§4.1) |
| **Phase 2** (Services) | `@mock_response` → generated `MockSpec` for transport boundaries | Service declarations (§4.3) |
| **Phase 2** (Services) | `@error_response` → generated failure scenarios for Bucket C | Service declarations (§4.3) |
| **Phase 3** (Composition) | Property-based fuzzing for pure nodes (Bucket B exhaustive) | Refinement types + `execute_single_node` harness |
| **Phase 4** (Multi-target) | Fuzz tests emitted in target language (Rust proptest, Python hypothesis, Go rapid) | Codegen backend trait extended |

The progression is deliberate: Phase 1 eliminates the worst of the manual MockSpec burden (safe defaults). Phase 2 eliminates it entirely for well-annotated services. Phase 3 makes Bucket B exhaustive. Phase 4 makes it multi-language.
