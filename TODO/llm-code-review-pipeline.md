# LLM Code Review Pipeline

**Status**: Design
**Date**: 2026-02-01

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
- [ ] ReviewPhase subdag is read-only (no write ops)
- [ ] DryRun can run entire graph without touching repo
- [ ] Output is machine-parseable JSON with schema version
- [ ] Findings have stable IDs via `issue_key`

**V0 does NOT include**: durability, Codex integration, multi-turn, apply/commit.

## V1 Scope

- Review Cycle orchestration (configurable criteria, iteration)
- ImplementationPhase with Codex CLI integration
- Feed findings with `candidate_fix` back into Codex
- Keep "apply patch" as explicit Action Phase

---

## Implementation Plan

### Section 1: Core Types (`lib/review/`)

Create `lib/review/` crate with domain types.

**TODO 1.1: Artifact types**
- [ ] `Artifact` enum (Code, Design, TestOutput, Text variants)
- [ ] `DocFormat` enum for design docs

**TODO 1.2: Criteria types**
- [ ] `Criteria` struct (name, description, checks)
- [ ] `Check` struct (id, question, examples)

**TODO 1.3: Finding types**
- [ ] `Finding` struct (id, check_id, location, issue_key, observation, candidate_fix)
- [ ] `Location` enum (FileLine, Span, DiffLine with DiffSide)
- [ ] `finding_id()` hash function using issue_key

**TODO 1.4: Output types**
- [ ] `ReviewOutput` struct (criteria_name, source, findings, candidate_remediations, summary)
- [ ] `ReviewBundle` struct for multi-source merging
- [ ] `CandidateRemediations` struct (goals, tasks, constraints)
- [ ] `CandidateTask` struct (finding_id, file, intent, candidate_patch, validation)

**TODO 1.5: JSON schema**
- [ ] Derive Serialize/Deserialize for all types
- [ ] Add schema version field to ReviewOutput
- [ ] Basic validation tests

---

### Section 2: ReviewOps (`lib/review/ops.rs`)

Implement `ReviewOps` enum with `Executable` trait.

**TODO 2.1: PrepareReviewPrompt**
- [ ] `ReviewOps::PrepareReviewPrompt` variant
- [ ] Input: artifact, criteria, optional context
- [ ] Output: prompt string, system_prompt string
- [ ] Prompt template that instructs LLM to output structured findings

**TODO 2.2: ParseReviewResponse**
- [ ] `ReviewOps::ParseReviewResponse` variant
- [ ] Input: LLM response string, criteria (for check_id mapping)
- [ ] Output: `ReviewOutput`
- [ ] Parse JSON from LLM response, generate finding IDs

**TODO 2.3: MergeOutputs**
- [ ] `ReviewOps::MergeOutputs` variant
- [ ] Input: `Vec<ReviewOutput>`
- [ ] Output: `ReviewBundle`
- [ ] Dedup findings by id

---

### Section 3: DAG Wiring (`lib/review/dag.rs`)

Wire up ReviewPhase subdag using existing primitives.

**TODO 3.1: Review existing primitives**
- [ ] Understand `GitOps::PrepareDiff` / `ParseDiff` in `lib/git-ops/`
- [ ] Understand `LlmOps::PrepareChatRequest` / `ParseChatResponse` in `lib/llm-ops/`
- [ ] Understand `TransportOps::Execute` pattern

**TODO 3.2: ReviewPhase builder**
- [ ] `ReviewPhaseBuilder` struct
- [ ] Entrypoints: artifact, criteria, config
- [ ] Internal flow: PreparePrompt → LLM subdag → ParseResponse
- [ ] Interface boundaries: findings, summary

**TODO 3.3: DiffReview convenience builder**
- [ ] Combines GitOps::PrepareDiff → ReviewPhase
- [ ] Input: git ref (e.g., "HEAD~1"), criteria
- [ ] Output: ReviewOutput as JSON

---

### Section 4: CLI Integration

Add `gunbc review` command.

**TODO 4.1: CLI command**
- [ ] `gunbc review --diff <ref>` subcommand
- [ ] Load criteria from config or use default
- [ ] Output JSON to stdout

**TODO 4.2: Default criteria**
- [ ] Ship basic "code review" criteria
- [ ] Configurable via `~/.gunbc/review.toml` or similar

**TODO 4.3: Integration test**
- [ ] Test against sample diff
- [ ] Verify JSON output schema
- [ ] DryRun mode test (no actual LLM calls)

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

### Unified Acquisition Pattern

All DAG resources follow the same acquisition pattern:

```
[PrepareAcquire] → [TransportOps::Execute] → [ParseAcquire]
     (params)            (I/O boundary)          (handle)
```

| Resource Type | Params | Handle | Notes |
|---------------|--------|--------|-------|
| **Tool** | name, version? | ToolHandle | Infinite, from environment |
| **Blob** | source | ContentBlob | Read-only content |
| **Lock** | name, scope | LockHandle | Exclusive access (future) |

The acquisition layer is uniform - only the params differ.

---

### BlobOps (`lib/blob/ops.rs`)

```rust
pub enum BlobOps {
    /// Build acquisition request for content blob
    PrepareAcquire,

    /// Parse response into ContentBlob
    ParseAcquire,

    /// Extract data as string (convenience)
    ExtractData,

    /// Get metadata without full content
    PrepareMeta,
    ParseMeta,
}
```

#### PrepareAcquire

| Port | Direction | Type | Cardinality | Description |
|------|-----------|------|-------------|-------------|
| `source` | input | `Json` | One | ContentSource specification |
| `request` | output | `Request` | One | Transport request |

**I/O**: Pure

#### ParseAcquire

| Port | Direction | Type | Cardinality | Description |
|------|-----------|------|-------------|-------------|
| `response` | input | `Response` | One | Transport response |
| `source` | input | `Json` | One | Original source (for metadata) |
| `blob` | output | `Json` | One | ContentBlob with data + meta |

**I/O**: Pure

---

### Content Types

```rust
/// Where to get content from
pub enum ContentSource {
    /// Direct inline content (no acquisition needed)
    Inline {
        data: String,
        content_type: Option<String>,
    },

    /// Local filesystem
    File {
        path: PathBuf,
    },

    /// Git object (blob at ref)
    GitBlob {
        ref_: String,      // "HEAD", "main", commit SHA
        path: String,      // path within repo
    },

    /// S3-compatible storage
    S3 {
        bucket: String,
        key: String,
        region: Option<String>,
    },

    /// HTTP GET
    Http {
        url: String,
        headers: Option<HashMap<String, String>>,
    },
}

/// Acquired content with metadata
pub struct ContentBlob {
    pub source: ContentSource,
    pub data: String,           // Content as string (bytes later)
    pub meta: BlobMeta,
}

pub struct BlobMeta {
    pub size: usize,
    pub hash: Option<String>,   // SHA256 for caching/dedup
    pub content_type: Option<String>,
    pub etag: Option<String>,   // For HTTP/S3 caching
}
```

---

### Acquisition Flow Examples

**File blob:**
```
ContentSource::File { path: "src/main.rs" }
    ↓
[PrepareAcquire] → Request::File(FileRequest::Read { path })
    ↓
[Execute] → Response::File(FileResponse { content, ... })
    ↓
[ParseAcquire] → ContentBlob { data: content, meta: { size, hash } }
```

**Git blob:**
```
ContentSource::GitBlob { ref_: "HEAD", path: "src/main.rs" }
    ↓
[PrepareAcquire] → Request::Shell("git show HEAD:src/main.rs")
    ↓
[Execute] → Response::Shell { stdout, ... }
    ↓
[ParseAcquire] → ContentBlob { data: stdout, meta: { hash: git_hash } }
```

**S3 blob:**
```
ContentSource::S3 { bucket: "my-bucket", key: "data/input.json" }
    ↓
[PrepareAcquire] → Request::Http(GET s3://...)
    ↓
[Execute] → Response::Http { body, headers, ... }
    ↓
[ParseAcquire] → ContentBlob { data: body, meta: { etag, size } }
```

**Inline (no acquisition):**
```
ContentSource::Inline { data: "..." }
    ↓
[PrepareAcquire] → (passthrough, no Execute needed)
    ↓
ContentBlob { data: "...", meta: { size, hash } }
```

---

### Updated Module Dependency Graph

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│  lib/review │────▶│ lib/llm-ops │────▶│lib/transport│
└─────────────┘     └─────────────┘     └─────────────┘
       │                   │                   ▲
       │            ┌──────┴──────┐            │
       │            │ lib/git-ops │────────────┤
       │            └─────────────┘            │
       │                                       │
       └───────────▶┌─────────────┐────────────┘
                    │  lib/blob   │
                    └─────────────┘
```

---

### Updated ReviewPhase with Blob Input

```
ENTRYPOINTS:
  ├── source: Json (One) - ContentSource
  ├── criteria: Json (One)
  └── config: Json (ZeroOrOne)

INTERNAL FLOW:
  ┌──────────────────────────────────────────────────────────────┐
  │                                                              │
  │  [PrepareAcquire] ──req──▶ [Execute] ──▶ [ParseAcquire]     │
  │                                               │              │
  │                                               ▼              │
  │  [PrepareReviewPrompt] ◀────blob.data─────────┘             │
  │         │                                                    │
  │         ├──prompt──▶ [LLM subdag] ──▶ [ParseReviewResponse] │
  │         └──system──┘                          │              │
  │                                               ▼              │
  │                                         [findings]           │
  │                                                              │
  └──────────────────────────────────────────────────────────────┘

INTERFACE BOUNDARIES:
  ├── findings: Json (One) - ReviewOutput
  └── blob_meta: Json (One) - BlobMeta (for caching)
```

Now ReviewPhase takes a `ContentSource` and handles acquisition uniformly.

---

*See git history for earlier design iterations.*
