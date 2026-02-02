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

### Resource Stack

All acquisitions share a common base, with layers adding structure:

```
┌─────────────────────────────────────────────────────────────┐
│  Layer 3: ReviewBlob                                        │
│  Structured review content (knows schema, can be assessed)  │
├─────────────────────────────────────────────────────────────┤
│  Layer 2: Blob                                              │
│  Raw data + metadata (file, git, s3, http, inline)          │
├─────────────────────────────────────────────────────────────┤
│  Layer 1: Resource                                          │
│  Base acquisition (tools, blobs, locks all use this)        │
└─────────────────────────────────────────────────────────────┘
```

Each layer builds on the one below:

| Layer | Input | Output | Purpose |
|-------|-------|--------|---------|
| **Resource** | params | handle | Generic acquisition pattern |
| **Blob** | source | data + meta | Raw content with caching info |
| **ReviewBlob** | blob + schema | structured content | Review-ready, schema-aware |

---

### Layer 1: Resource (`core/resource/`)

Base acquisition pattern shared by all resource types:

```rust
pub trait Resource {
    type Params;
    type Handle;

    fn prepare(params: &Self::Params) -> TransportRequest;
    fn parse(response: TransportResponse, params: &Self::Params) -> Self::Handle;
}
```

```
[PrepareAcquire] → [TransportOps::Execute] → [ParseAcquire]
     (params)            (I/O boundary)          (handle)
```

| Resource Type | Params | Handle | Notes |
|---------------|--------|--------|-------|
| **Tool** | name, version? | ToolHandle | Infinite, from environment |
| **Blob** | BlobSource | Blob | Read-only content |
| **Lock** | name, scope | LockHandle | Exclusive access (future) |

---

### Layer 2: Blob (`lib/blob/`)

A Resource that provides data content:

```rust
pub enum BlobOps {
    /// Build acquisition request
    PrepareAcquire,
    /// Parse response into Blob
    ParseAcquire,
    /// Get metadata only (no content fetch)
    PrepareMeta,
    ParseMeta,
}
```

#### PrepareAcquire

| Port | Direction | Type | Cardinality | Description |
|------|-----------|------|-------------|-------------|
| `source` | input | `Json` | One | BlobSource specification |
| `request` | output | `Request` | One | Transport request |

**I/O**: Pure

#### ParseAcquire

| Port | Direction | Type | Cardinality | Description |
|------|-----------|------|-------------|-------------|
| `response` | input | `Response` | One | Transport response |
| `source` | input | `Json` | One | Original source (for metadata) |
| `blob` | output | `Json` | One | ContentBlob with data + meta |

**I/O**: Pure

#### Blob Types

```rust
/// Where to get blob content from (Layer 2 params)
pub enum BlobSource {
    /// Direct inline content (no I/O needed)
    Inline {
        data: String,
        content_type: Option<String>,
    },

    /// Local filesystem
    File { path: PathBuf },

    /// Git object at ref
    GitBlob { ref_: String, path: String },

    /// S3-compatible storage
    S3 { bucket: String, key: String, region: Option<String> },

    /// HTTP GET
    Http { url: String, headers: Option<HashMap<String, String>> },
}

/// Acquired blob (Layer 2 handle)
pub struct Blob {
    pub source: BlobSource,
    pub data: String,       // Content as string (bytes later)
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

### Layer 3: ReviewBlob (`lib/review/blob.rs`)

A Blob with review-specific structure:

```rust
pub enum ReviewBlobOps {
    /// Wrap a Blob with schema information
    WrapBlob,
    /// Extract content in review-ready format
    PrepareForReview,
}
```

#### WrapBlob

| Port | Direction | Type | Cardinality | Description |
|------|-----------|------|-------------|-------------|
| `blob` | input | `Json` | One | Raw Blob from Layer 2 |
| `schema` | input | `Json` | One | ContentSchema (how to interpret) |
| `review_blob` | output | `Json` | One | ReviewBlob ready for assessment |

**I/O**: Pure

#### ReviewBlob Types

```rust
/// How to interpret blob content for review
pub enum ContentSchema {
    /// Source code with language hint
    Code { language: Option<String> },

    /// Structured diff (unified format)
    Diff { base_ref: Option<String> },

    /// Design document
    Design { format: DocFormat },

    /// Test output / logs
    TestOutput { format: OutputFormat },

    /// Freeform text
    Text,
}

pub enum DocFormat { Markdown, PlainText, Html }
pub enum OutputFormat { TestRunner, BuildLog, Structured }

/// Blob with review context (Layer 3 handle)
pub struct ReviewBlob {
    pub blob: Blob,              // Underlying data
    pub schema: ContentSchema,   // How to interpret
    pub extracted: ExtractedContent, // Pre-processed for review
}

/// Content extracted based on schema
pub enum ExtractedContent {
    Code {
        language: String,
        lines: Vec<String>,
        // Future: AST, symbols, etc.
    },
    Diff {
        hunks: Vec<DiffHunk>,
        files_changed: Vec<String>,
    },
    Design {
        sections: Vec<Section>,
    },
    Text {
        content: String,
    },
}
```

---

### Full Stack Example

```
User wants to review a git diff:

1. Resource layer:
   BlobSource::GitBlob { ref_: "HEAD~1..HEAD", path: "." }
       ↓
   [PrepareAcquire] → Request::Shell("git diff HEAD~1..HEAD")
       ↓
   [Execute] → Response::Shell { stdout: "diff --git..." }
       ↓
   [ParseAcquire] → Blob { data: "diff --git...", meta: {...} }

2. Blob layer: ✓ (output of step 1)

3. ReviewBlob layer:
   Blob + ContentSchema::Diff { base_ref: "HEAD~1" }
       ↓
   [WrapBlob] → ReviewBlob {
       blob: Blob { ... },
       schema: Diff { ... },
       extracted: ExtractedContent::Diff {
           hunks: [...],
           files_changed: ["src/main.rs", ...]
       }
   }

4. Review layer:
   ReviewBlob + Criteria
       ↓
   [PrepareReviewPrompt] → prompt with structured diff context
       ↓
   [LLM] → findings
```

---

### Updated Module Dependency Graph

```
┌───────────────────────────────────────────────────────────────┐
│                        lib/review                             │
│  (ReviewBlob, ReviewOps, Criteria, Finding)                   │
└───────────────────────────────────────────────────────────────┘
                              │
           ┌──────────────────┼──────────────────┐
           ▼                  ▼                  ▼
┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐
│    lib/blob     │  │   lib/llm-ops   │  │   lib/git-ops   │
│  (Blob, BlobOps)│  │ (PrepareChatReq)│  │  (PrepareDiff)  │
└─────────────────┘  └─────────────────┘  └─────────────────┘
           │                  │                  │
           └──────────────────┼──────────────────┘
                              ▼
                 ┌─────────────────────────┐
                 │    core/resource        │
                 │  (Resource trait)       │
                 └─────────────────────────┘
                              │
                              ▼
                 ┌─────────────────────────┐
                 │    lib/transport        │
                 │  (TransportOps::Execute)│
                 └─────────────────────────┘
```

---

### Updated ReviewPhase DAG (with stack)

```
ENTRYPOINTS:
  ├── source: Json (One) - BlobSource
  ├── schema: Json (One) - ContentSchema
  ├── criteria: Json (One) - Criteria
  └── config: Json (ZeroOrOne)

INTERNAL FLOW:
  ┌────────────────────────────────────────────────────────────────┐
  │                                                                │
  │  Layer 1-2: Blob Acquisition                                   │
  │  ┌──────────────────────────────────────────────────────────┐  │
  │  │ [PrepareAcquire] ──req──▶ [Execute] ──▶ [ParseAcquire]  │  │
  │  │                                               │          │  │
  │  │                                               ▼          │  │
  │  │                                             Blob         │  │
  │  └──────────────────────────────────────────────────────────┘  │
  │                                                  │              │
  │  Layer 3: ReviewBlob                             ▼              │
  │  ┌──────────────────────────────────────────────────────────┐  │
  │  │ [WrapBlob] ◀──── Blob + schema                          │  │
  │  │      │                                                   │  │
  │  │      ▼                                                   │  │
  │  │  ReviewBlob (with ExtractedContent)                      │  │
  │  └──────────────────────────────────────────────────────────┘  │
  │                                                  │              │
  │  Layer 4: Review                                 ▼              │
  │  ┌──────────────────────────────────────────────────────────┐  │
  │  │ [PrepareReviewPrompt] ◀──── ReviewBlob + criteria       │  │
  │  │        │                                                 │  │
  │  │        ├──prompt──▶ [LLM subdag] ──▶ [ParseResponse]    │  │
  │  │        └──system──┘                        │             │  │
  │  │                                            ▼             │  │
  │  │                                      ReviewOutput        │  │
  │  └──────────────────────────────────────────────────────────┘  │
  │                                                                │
  └────────────────────────────────────────────────────────────────┘

INTERFACE BOUNDARIES:
  ├── findings: Json (One) - ReviewOutput
  ├── blob_meta: Json (One) - BlobMeta (for caching)
  └── extracted: Json (One) - ExtractedContent (for downstream)
```

Each layer is independently testable and reusable.

---

*See git history for earlier design iterations.*
