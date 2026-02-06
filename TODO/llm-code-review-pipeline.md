# LLM Code Review Pipeline

**Status**: V0 complete. Tracks 2-6 implemented. Track 1 (Resource trait) still design.
**Date**: 2026-02-01
**Updated**: 2026-02-03

## North Star

gunbc workflows are pure dataflow graphs whose only interaction with the outside world occurs at explicit **transport nodes**. Transport operations are classified by **risk level** (low/medium/high/extreme) to enable static validation and DryRun interception. Higher-level workflows concentrate mutation in explicit **action phases** while keeping **reasoning phases** hermetic and replayable.

## Two Boundary Types

| Term | Definition | Purpose |
|------|------------|---------|
| **Transport Boundary** | Node executing `TransportOps::Execute*` | Where I/O happens; DryRun intercepts here |
| **Interface Boundary** | Unconnected output port | DAG composition interface |

## Phase Taxonomy

| Phase | Type | I/O | Notes |
|-------|------|-----|-------|
| **ReviewPhase** | Reasoning | Read-only | Produces findings, not mutations |
| **ImplementationPhase** | Reasoning | Read-only | Produces patch artifacts, doesn't apply them |
| **TestPhase** | Reasoning | Read-only | Observes behavior |
| **ApplyPhase** | Action | Write | Mutates user artifacts |
| **CommitPhase** | Action | Write | Mutates repository |

**Invariant**: Reasoning phases are read-only — they produce *artifacts* (patches, findings) but don't apply them. Action phases do writes and are explicit/separate.

## Transport Classification

**I/O type** (read/write) — intrinsic, stable domain knowledge:
- **Read**: file reads, git diff, LLM calls, HTTP GET
- **Write**: file writes, git commit, cache updates

This is the core model. DryRun intercepts by I/O type. Purity reasoning uses I/O type.

**Risk modeling** — separate concern, context-dependent:
- Same write: low-risk in test, extreme-risk in prod
- Risk depends on environment, what's being accessed, policies
- Will model separately when we have concrete use cases

## ReviewPhase: Reconciliation + Candidate Repairs

```
ReviewPhase: (artifact, criteria) → (findings, candidate_remediations)
```

- **No verdict, no decisions** — orchestration decides what to do with findings
- **Candidate repairs** are proposals, not commands
- `Finding` has stable `issue_key` for cross-iteration matching (not observation text)
- `ReviewBundle` merges multiple sources (LLM + clippy + cargo check)

## ImplementationPhase: Produces Patches

**Codex invariant**: Must invoke Codex in a mode that **cannot mutate user artifacts**. Changes emitted as patch artifacts, applied only by ApplyPhase.

- `session_id` (UUID) for identity, `task_id` (content hash) for dedup
- Step caching requires **world state inputs** (git HEAD, tool versions, model fingerprint)

## V0 Scope

**Goal**: Review current branch diff → JSON report with findings

```
GitOps::PrepareDiff → Execute → ReviewOps::PreparePrompt → LLM → ParseResponse → JSON
```

**V0 Crispness Checklist**:
- [x] ReviewPhase subdag is read-only (no write ops)
- [x] DryRun can run entire graph without touching repo
- [x] Output is machine-parseable JSON with schema version
- [x] Findings have stable IDs via `issue_key`

**V0 does NOT include**: durability, Codex integration, multi-turn, apply/commit.

## V1 Scope

- Review Cycle orchestration (configurable criteria, iteration)
- ImplementationPhase with Codex CLI integration
- Feed findings with `candidate_fix` back into Codex
- Keep "apply patch" as explicit Action Phase

---

## Concurrent Design Tracks

These abstractions need to be designed together - changes in one affect the others:

### Track 1: Resource Abstraction (`core/resource/`)
- [ ] `Resource` trait: `prepare(params) → request`, `parse(response) → handle`
- [ ] Unified acquisition pattern for tools, blobs, locks
- [ ] How does this interact with existing `TransportOps`?
- [ ] Caching/memoization at resource level?

### Track 2: Blob Abstraction (`lib/blob/`) — DONE
- [x] `BlobSource` enum: Inline, File, GitBlob, S3, Http
- [x] `Blob` struct: source + data + meta
- [x] `BlobMeta`: size, hash, etag for caching
- [x] How do blobs flow through DAG edges? → Json serialization via encode/decode
- [x] Blob as Value variant vs Json serialization? → Json (BlobHandle.encode/decode)

### Track 3: LLM Query Abstraction (`lib/llm-ops/`) — DONE
- [x] `LlmQueryOps`: PrepareSimpleRequest, ParseSimpleResponse
- [x] content + question → answer
- [x] Relationship to existing `PrepareChatRequest`/`ParseChatResponse` → higher-level wrapper
- [ ] Structured output schemas (JSON mode, tool use, etc.) — future

### Track 4: Review Domain (`lib/review/`) — DONE
- [x] `ReviewOps`: PrepareReviewPrompt, ParseReviewResponse, MergeOutputs, HashFinding, FormatDiffArtifact
- [x] Criteria → question formatting
- [x] Answer → findings parsing
- [x] How findings flow back for remediation → CandidateRemediations type

### Track 5: DAG Composition — DONE
- [x] ReviewPhase, InlineReview, DiffReviewPhase, MultiSourceReviewPhase builders
- [x] Entrypoint/boundary detection with new abstractions
- [x] DryRun interception via MockSpec (graph_mock.rs)
- [ ] Parallel execution with resource access declarations — future

### Track 6: CLI Integration — DONE
- [x] `gunbc review` registered in codegen registry (--base-ref, --provider, --model, --dry-run)
- [x] Default review criteria (correctness, security, performance, clarity)
- [ ] Read vs Write classification — future (all review ops are read-only)
- [ ] Integration tests against sample diff — future

### Open Questions (Resolved)

1. **Blob as first-class Value?** → JSON serialization via `BlobHandle.encode/decode`. No `Value::Blob` variant.
2. **Resource caching** → Deferred until Track 1 (Resource Abstraction) provides a unified answer.
3. **Schema validation** → In `ParseReviewResponse` via `extract_json_from_response()` + check_id validation against criteria.
4. **Review output as blob** → Not needed for V0. Findings flow as `Value::Json`.

---

## Implementation Plan

Organized by concurrent design track. Each track can be worked on in parallel, but changes affect other tracks.

---

### Track 1: Resource Abstraction (`core/resource/`)

Unify acquisition pattern across tools, blobs, locks.

**TODO 1.1: Resource trait**
- [ ] Define `Resource` trait with `Params` and `Handle` associated types
- [ ] `prepare(params) → TransportRequest`
- [ ] `parse(response, params) → Handle`
- [ ] Align with existing `CliToolOp` pattern

**TODO 1.2: Integrate with UpsertBuilder**
- [ ] Verify `UpsertBuilder` works for Check → Acquire → Resolve pattern
- [ ] Add `ResourceUpsert` convenience that uses `UpsertBuilder`
- [ ] Test with existing `CliToolOp` as reference

**TODO 1.3: ResourceId integration**
- [ ] Ensure all resources can produce `ResourceId`
- [ ] Verify `AccessMode` conflict detection works
- [ ] Add tests for parallel resource access

---

### Track 2: Blob Abstraction (`lib/blob/`) — DONE

**TODO 2.1: BlobSource types**
- [x] `BlobSource` enum: Inline, File, GitBlob, S3, Http
- [x] `resource_id()` for conflict detection
- [x] `AccessMode::Read` for all blobs

**TODO 2.2: BlobOps**
- [x] `PrepareFetch` / `ParseFetch` (fetch from source)
- [x] Implement `Executable` trait

**TODO 2.3: BlobHandle**
- [x] Sealed handle with `PhantomData`
- [x] `data()`, `meta()`, `source()` accessors
- [x] `BlobMeta`: size, hash, content_type, etag
- [x] `encode()` / `decode()` for DAG edge transmission

**TODO 2.4: Value integration**
- [x] JSON serialization via BlobHandle.encode/decode (no `Value::Blob`)
- [x] Blob flow through DAG edges tested

---

### Track 3: LLM Query Abstraction (`lib/llm-ops/`) — DONE

**TODO 3.1: LlmQueryOps**
- [x] `PrepareSimpleRequest`: content + question → request
- [x] `ParseSimpleResponse`: response → answer string
- [x] Implement `Executable` trait
- [x] Wraps existing `PrepareChatRequest`/`ParseChatResponse`

**TODO 3.2: Structured output** — deferred (future)

---

### Track 4: Review Domain (`lib/review/`) — DONE

**TODO 4.1: Core types**
- [x] `Criteria` struct (name, description, checks)
- [x] `Check` struct (id, question, examples)
- [x] `Finding` struct (id, check_id, issue_key, location, observation, candidate_fix)
- [x] `Location` enum (FileLine, Span, DiffLine, Unlocated)
- [x] `ReviewOutput` struct (schema_version, criteria_name, source, findings, summary)
- [x] `ReviewBundle` for multi-source merging
- [x] `CandidateRemediations` and `CandidateTask`

**TODO 4.2: ReviewOps**
- [x] `PrepareReviewPrompt`: criteria → prompt + system_prompt
- [x] `ParseReviewResponse`: answer → ReviewOutput (JSON)
- [x] `MergeOutputs`: Vec<ReviewOutput> → ReviewBundle with dedup
- [x] `HashFinding`: issue_key + check_id → stable SHA256 finding_id
- [x] `FormatDiffArtifact`: diff_files → artifact string
- [x] `LoadPipelineConfig`: emit config as output ports
- [x] Implement `Executable` trait

**TODO 4.3: JSON schema**
- [x] Derive Serialize/Deserialize for all types
- [x] Schema version field (`ReviewOutput::SCHEMA_VERSION = "0.1.0"`)
- [x] Validation tests

---

### Track 5: DAG Composition (`lib/review/graph.rs`) — DONE

**TODO 5.1: ReviewPhase builder**
- [x] `build_review_phase_graph()` with 8-node DAG
- [x] Entrypoints: source, artifact, criteria, provider, model
- [x] Interface boundaries: findings, errors, meta

**TODO 5.2: DiffReviewPhase builder**
- [x] `build_diff_review_graph()` composes GitOps + ReviewPhase
- [x] Input: git ref, criteria
- [x] Output: ReviewOutput

**TODO 5.3: InlineReview builder**
- [x] `build_inline_review_graph()` — simplified, no blob fetch

**TODO 5.4: DryRun support**
- [x] MockSpec definitions for all transport boundaries
- [x] `ExecutionMode::DryRun()` intercepts I/O
- [x] No side effects in DryRun mode

---

### Track 6: CLI Integration — DONE

**TODO 6.1: CLI command**
- [x] `gunbc review` registered in codegen registry
- [x] `--base-ref`, `--provider`, `--model`, `--dry-run` flags
- [x] Output JSON to stdout

**TODO 6.2: Default criteria**
- [x] Default criteria: correctness, security, performance, clarity

---

## Appendix A: DAG Modules & Interfaces

### Module Dependency Graph

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│  lib/review │────▶│ lib/llm-ops │────▶│lib/transport│
└─────────────┘     └─────────────┘     └─────────────┘
       │                   │                   │
       │            ┌──────┴──────┐            │
       └───────────▶│ lib/git-ops │────────────┘
                    └─────────────┘
```

All modules depend on `core/ir` (DAG, Value, Port) and `core/exec` (Executable trait).

---

### ReviewOps (`lib/review/ops.rs`)

```rust
pub enum ReviewOps {
    /// Build prompt from artifact + criteria
    PrepareReviewPrompt {
        /// Prompt template with placeholders
        template: String,
    },

    /// Parse LLM response into structured findings
    ParseReviewResponse,

    /// Merge multiple ReviewOutputs into ReviewBundle
    MergeOutputs,

    /// Generate stable finding ID from issue_key
    HashFinding,
}
```

#### PrepareReviewPrompt

| Port | Direction | Type | Cardinality | Description |
|------|-----------|------|-------------|-------------|
| `artifact` | input | `Str` | One | Code/design content to review |
| `criteria` | input | `Json` | One | Criteria definition |
| `context` | input | `Str` | ZeroOrOne | Optional extra context |
| `prompt` | output | `Str` | One | Formatted prompt for LLM |
| `system` | output | `Str` | One | System prompt |

**I/O**: Pure (read-only transform)

#### ParseReviewResponse

| Port | Direction | Type | Cardinality | Description |
|------|-----------|------|-------------|-------------|
| `response` | input | `Str` | One | Raw LLM response text |
| `criteria` | input | `Json` | One | For check_id resolution |
| `output` | output | `Json` | One | `ReviewOutput` as JSON |
| `errors` | output | `StrList` | ZeroOrMore | Parse errors/warnings |

**I/O**: Pure (read-only transform)

#### MergeOutputs

| Port | Direction | Type | Cardinality | Description |
|------|-----------|------|-------------|-------------|
| `outputs` | input | `Json` | OneOrMore | List of ReviewOutput |
| `bundle` | output | `Json` | One | Merged ReviewBundle |
| `conflicts` | output | `Json` | ZeroOrMore | Finding ID conflicts |

**I/O**: Pure (read-only transform)

#### HashFinding

| Port | Direction | Type | Cardinality | Description |
|------|-----------|------|-------------|-------------|
| `issue_key` | input | `Str` | One | Reviewer-provided stable key |
| `check_id` | input | `Str` | One | Which check this finding is for |
| `finding_id` | output | `Str` | One | Stable hash ID |

**I/O**: Pure (deterministic hash)

---

### Existing Ops (Reference)

#### GitOps (`lib/git-ops/`)

| Op | Inputs | Outputs | I/O |
|----|--------|---------|-----|
| `PrepareDiff` | `ref: Str` | `request: Request` | Pure |
| `ParseDiff` | `response: Response` | `diff: Str, stats: Json` | Pure |
| `PrepareLsFiles` | `ext: Str?` | `request: Request` | Pure |
| `ParseLsFiles` | `response: Response` | `files: StrList` | Pure |

#### LlmOps (`lib/llm-ops/`)

| Op | Inputs | Outputs | I/O |
|----|--------|---------|-----|
| `PrepareChatRequest` | `messages: Json, model: Str, ...` | `request: Request` | Pure |
| `ParseChatResponse` | `response: Response` | `content: Str, usage: Json` | Pure |

#### TransportOps (`lib/transport/`)

| Op | Inputs | Outputs | I/O |
|----|--------|---------|-----|
| `Execute` | `request: Request` | `response: Response` | **BOUNDARY** |

---

### ReviewPhase DAG Structure

```
ENTRYPOINTS (unconnected inputs):
  ├── artifact: Str (One)
  ├── criteria: Json (One)
  └── config: Json (ZeroOrOne)

INTERNAL FLOW:
  ┌──────────────────────────────────────────────────────────────┐
  │                                                              │
  │  [PrepareReviewPrompt] ──prompt──▶ [PrepareChatRequest]     │
  │         │                                  │                 │
  │         └────system────────────────────────┤                 │
  │                                            ▼                 │
  │                                   [TransportOps::Execute]    │
  │                                            │                 │
  │                                            ▼                 │
  │                                   [ParseChatResponse]        │
  │                                            │                 │
  │                                            ▼                 │
  │  [ParseReviewResponse] ◀───response────────┘                │
  │         │                                                    │
  │         ├──output───▶ [findings port]                       │
  │         └──errors───▶ [errors port]                         │
  │                                                              │
  └──────────────────────────────────────────────────────────────┘

INTERFACE BOUNDARIES (unconnected outputs):
  ├── findings: Json (One) - ReviewOutput
  └── errors: StrList (ZeroOrMore) - Parse errors
```

**I/O Classification**:
- Internal: All pure except one `TransportOps::Execute` (LLM call)
- Phase overall: Read-only (LLM call is a read in our classification)

---

### DiffReviewPhase DAG Structure

Composes GitOps + ReviewPhase:

```
ENTRYPOINTS:
  ├── ref: Str (One) - e.g., "HEAD~1", "main"
  ├── criteria: Json (One)
  └── config: Json (ZeroOrOne)

INTERNAL FLOW:
  ┌──────────────────────────────────────────────────────────────┐
  │                                                              │
  │  [PrepareDiff] ──request──▶ [Execute] ──▶ [ParseDiff]       │
  │        ▲                                      │              │
  │        │                                      ▼              │
  │      ref                            diff ──▶ [ReviewPhase]   │
  │                                              (subdag)        │
  │                                                  │           │
  │                                                  ▼           │
  │                                            [findings]        │
  │                                                              │
  └──────────────────────────────────────────────────────────────┘

INTERFACE BOUNDARIES:
  ├── findings: Json (One)
  └── summary: Str (ZeroOrOne)
```

**I/O Classification**:
- Two `TransportOps::Execute` calls: git diff (read), LLM (read)
- Phase overall: Read-only

---

### MultiSourceReviewPhase DAG Structure

Merges LLM + cargo check + clippy:

```
ENTRYPOINTS:
  ├── artifact: Str (One)
  ├── criteria: Json (One)
  └── tool_config: Json (ZeroOrOne)

INTERNAL FLOW (parallel fan-out, merge):
  ┌────────────────────────────────────────────────────────────────┐
  │                                                                │
  │                    ┌──▶ [ReviewPhase] ──────┐                 │
  │                    │     (LLM review)       │                 │
  │                    │                        ▼                 │
  │  artifact ─────────┼──▶ [CargoCheck] ──▶ [MergeOutputs]      │
  │                    │     (subdag)           ▲                 │
  │                    │                        │                 │
  │                    └──▶ [Clippy] ───────────┘                 │
  │                          (subdag)                             │
  │                                                               │
  └────────────────────────────────────────────────────────────────┘

INTERFACE BOUNDARIES:
  ├── bundle: Json (One) - ReviewBundle with all findings
  └── conflicts: Json (ZeroOrMore) - ID conflicts between sources
```

**I/O Classification**:
- Multiple reads: LLM, cargo check, clippy
- Phase overall: Read-only

---

### Resource Access Declarations

For parallel execution planning:

| Operation | Resource | Access |
|-----------|----------|--------|
| `PrepareReviewPrompt` | (none) | - |
| `ParseReviewResponse` | (none) | - |
| `TransportOps::Execute(LLM)` | `llm:$provider` | Read |
| `TransportOps::Execute(Shell:git)` | `repo:$path` | Read |
| `TransportOps::Execute(Shell:cargo)` | `repo:$path`, `cargo:registry` | Read |

All ReviewPhase operations are read-only, enabling safe parallelization.

---

### Type Definitions (Value Representations)

```rust
// Carried as Value::Json in the DAG

pub struct Criteria {
    pub name: String,
    pub description: String,
    pub checks: Vec<Check>,
}

pub struct Check {
    pub id: String,
    pub question: String,
    pub examples: Vec<String>,
}

pub struct Finding {
    pub id: String,           // Stable hash from issue_key + check_id
    pub check_id: String,
    pub issue_key: String,    // Reviewer-provided, no line numbers
    pub location: Location,
    pub observation: String,
    pub candidate_fix: Option<String>,
}

pub enum Location {
    FileLine { file: String, line: u32 },
    Span { file: String, start: u32, end: u32 },
    DiffLine { side: DiffSide, line: u32 },
    Unlocated,
}

pub struct ReviewOutput {
    pub schema_version: String,
    pub criteria_name: String,
    pub source: String,       // "llm", "clippy", "cargo-check"
    pub findings: Vec<Finding>,
    pub candidate_remediations: Option<CandidateRemediations>,
    pub summary: String,
}

pub struct ReviewBundle {
    pub outputs: Vec<ReviewOutput>,
    pub merged_findings: Vec<Finding>,  // Deduped by id
}

pub struct CandidateRemediations {
    pub goals: Vec<String>,
    pub constraints: Vec<String>,
    pub tasks: Vec<CandidateTask>,
}

pub struct CandidateTask {
    pub finding_id: String,
    pub file: String,
    pub intent: String,
    pub candidate_patch: Option<String>,
    pub validation: Option<String>,
}
```

---

## Appendix B: Content Blob Abstraction

### Alignment with Existing Patterns

The blob abstraction aligns with existing resource patterns in the codebase:

| Pattern | Existing Example | Blob Equivalent |
|---------|------------------|-----------------|
| **Prepare/Parse** | `GistOps`, `GitOps`, `LlmOps` | `BlobOps::PrepareAcquire/ParseAcquire` |
| **Upsert** | `CliToolOp` (Check→Create→Resolve) | `BlobOps` (Check cache→Fetch→Return) |
| **Capability handle** | `ToolHandle` (sealed, requires acquisition) | `BlobHandle` (sealed, requires acquisition) |
| **Access modes** | `ResourceId + AccessMode` | Same - blobs are `Read` access |
| **Declarative def** | `CliToolDef` | `BlobSource` |

### Resource Acquisition (Unified)

All resources follow the same pattern (see `core/ir/src/patterns/upsert.rs`):

```
┌─────────────────────────────────────────────────────────────┐
│  Check (exists/cached?)                                     │
│       ↓ exists=false                                        │
│  Acquire (fetch/install)                                    │
│       ↓                                                     │
│  Resolve (return handle)                                    │
└─────────────────────────────────────────────────────────────┘
```

| Resource | Check | Acquire | Handle |
|----------|-------|---------|--------|
| **Tool** | `check_cmd` runs | `install_cmd` runs | `ToolHandle` |
| **Blob** | cache lookup by hash | fetch from source | `BlobHandle` |
| **Lock** | lock available? | acquire lock | `LockHandle` |

### BlobOps (aligned with CliToolOp)

```rust
pub enum BlobOps {
    /// Check if blob is cached (like CliToolOp check phase)
    PrepareCheckCached,
    ParseCheckCached,    // → exists: Bool

    /// Fetch blob from source (like CliToolOp install phase)
    PrepareFetch,
    ParseFetch,          // → BlobHandle

    /// Convenience: full upsert in one subdag (uses UpsertBuilder)
    Acquire,             // → BlobHandle
}

/// Declarative blob definition (like CliToolDef)
pub struct BlobSource {
    pub source: SourceSpec,
    pub access_mode: AccessMode,  // Always Read for blobs
    pub cache_key: Option<String>, // For cache lookup
}

pub enum SourceSpec {
    Inline { data: String },
    File { path: PathBuf },
    GitBlob { ref_: String, path: String },
    S3 { bucket: String, key: String },
    Http { url: String },
}

/// Capability-based handle (like ToolHandle)
pub struct BlobHandle {
    id: BlobId,
    _sealed: PhantomData<()>,  // Prevents construction without acquisition
}

impl BlobHandle {
    pub fn data(&self) -> &str { ... }
    pub fn meta(&self) -> &BlobMeta { ... }
}
```

### Blob as ResourceId

Blobs integrate with existing `ResourceId` for conflict detection:

```rust
// From core/ir/src/resource.rs
impl BlobSource {
    pub fn resource_id(&self) -> ResourceId {
        match &self.source {
            SourceSpec::File { path } => ResourceId::file(path),
            SourceSpec::GitBlob { ref_, path } => ResourceId::file(path), // or git-specific
            SourceSpec::S3 { bucket, key } => ResourceId::new(&format!("s3:{}/{}", bucket, key)),
            SourceSpec::Http { url } => ResourceId::new(&format!("http:{}", url)),
            SourceSpec::Inline { .. } => ResourceId::new("inline"), // No conflict possible
        }
    }
}
```

All blobs have `AccessMode::Read` - safe for parallel access.

---

### BlobMeta

```rust
pub struct BlobMeta {
    pub size: usize,
    pub hash: Option<String>,   // SHA256 for caching/dedup
    pub content_type: Option<String>,
    pub etag: Option<String>,   // For HTTP/S3 caching
}
```

---

### Generic LLM Query (`lib/llm-ops/`)

A simple primitive: content + question → answer

```rust
pub enum LlmQueryOps {
    /// Build a query from content and question
    PrepareQuery,
    /// Parse structured response
    ParseQuery,
}
```

#### PrepareQuery

| Port | Direction | Type | Cardinality | Description |
|------|-----------|------|-------------|-------------|
| `content` | input | `Str` | One | Content to query (blob.data) |
| `question` | input | `Str` | One | What to ask about the content |
| `schema` | input | `Json` | ZeroOrOne | Expected response structure |
| `request` | output | `Request` | One | LLM request |

**I/O**: Pure

#### ParseQuery

| Port | Direction | Type | Cardinality | Description |
|------|-----------|------|-------------|-------------|
| `response` | input | `Response` | One | LLM response |
| `schema` | input | `Json` | ZeroOrOne | For validation |
| `answer` | output | `Json` | One | Parsed answer |
| `raw` | output | `Str` | One | Raw response text |

**I/O**: Pure

---

### How Review Uses This

Review is just LLM query with review-specific criteria:

```
Blob.data + Criteria → LlmQuery → ReviewOutput
```

The review layer:
1. Takes a Blob (any content)
2. Builds a question from Criteria (the review-specific part)
3. Uses generic LlmQuery
4. Parses response into ReviewOutput (findings, etc.)

```
┌─────────────────────────────────────────────────────────────┐
│  Review (lib/review/)                                       │
│  Knows: criteria → question, response → findings            │
├─────────────────────────────────────────────────────────────┤
│  LlmQuery (lib/llm-ops/)                                    │
│  Generic: content + question → answer                       │
├─────────────────────────────────────────────────────────────┤
│  Blob (lib/blob/)                                           │
│  Generic: source → data + meta                              │
├─────────────────────────────────────────────────────────────┤
│  Resource (core/resource/)                                  │
│  Generic: params → handle                                   │
└─────────────────────────────────────────────────────────────┘
```

---

### Simplified ReviewPhase DAG

```
ENTRYPOINTS:
  ├── source: Json (One) - BlobSource
  ├── criteria: Json (One) - Criteria
  └── config: Json (ZeroOrOne)

INTERNAL FLOW:
  ┌────────────────────────────────────────────────────────────────┐
  │                                                                │
  │  [BlobOps::PrepareAcquire] ──▶ [Execute] ──▶ [ParseAcquire]   │
  │                                                   │            │
  │                                                   ▼            │
  │  [ReviewOps::BuildQuestion] ◀─── blob.data + criteria         │
  │         │                                                      │
  │         ▼                                                      │
  │  [LlmQueryOps::PrepareQuery] ──▶ [Execute] ──▶ [ParseQuery]   │
  │                                                   │            │
  │                                                   ▼            │
  │  [ReviewOps::ParseFindings] ◀─── answer                       │
  │         │                                                      │
  │         ▼                                                      │
  │    ReviewOutput                                                │
  │                                                                │
  └────────────────────────────────────────────────────────────────┘

INTERFACE BOUNDARIES:
  ├── findings: Json (One) - ReviewOutput
  └── blob_meta: Json (One) - BlobMeta (for caching)
```

Each layer is generic and reusable. Review just adds the domain knowledge of what questions to ask and how to interpret answers.

---

*See git history for earlier design iterations.*
