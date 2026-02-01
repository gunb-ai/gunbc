# LLM Code Review Pipeline

**Status**: Design
**Date**: 2026-02-01

## Goal

Build a fractal DAG-based development workflow with:
1. **ImplementationPhase** — Codex CLI session management with crash-resilient durability
2. **ReviewPhase** — General-purpose, reusable review process for any artifact type

These phases compose into larger workflows (Requirements → Design → Implement → Review → Test → Commit) but are independently useful.

---

## Design Principles: Boundary Terminology & Transport Classification

This section reconciles with AGENT.md ("Nodes are Pure, Boundaries are Structural") and SPEC.md (§2.7 "No side channels", §6 "Node Contract").

> **North Star**: gunbc workflows are pure dataflow graphs whose only interaction with the outside world occurs at explicit transport nodes. Transport operations are classified by risk level (low/medium/high/extreme) to enable static validation (e.g., "review contains no high-risk ops") and runtime interception (DryRun). Composition happens through interface boundaries (unconnected ports). Higher-level workflows are built by composing subdags, concentrating mutation inside explicit **action phases** while keeping **reasoning phases** hermetic and replayable.

### Boundary Terminology (Two Distinct Concepts)

gunbc has **two kinds of boundaries** that must not be conflated:

| Term | Definition | How Detected | Purpose |
|------|------------|--------------|---------|
| **Transport Boundary** | Node that executes `TransportOps::Execute*` | By op type | Where I/O happens; DryRun intercepts here |
| **Interface Boundary** | Unconnected output port | By graph structure | DAG composition interface |

```
┌─────────────────────────────────────────────────────────────────────┐
│                          Example DAG                                │
│                                                                     │
│   ┌─────────────┐     ┌─────────────┐     ┌─────────────┐          │
│   │ Prepare     │────▶│ Execute     │────▶│ Parse       │          │
│   │ (pure)      │     │ (TRANSPORT  │     │ (pure)      │          │
│   └─────────────┘     │  BOUNDARY)  │     └──────┬──────┘          │
│                       └─────────────┘            │                  │
│                                                  ▼                  │
│                                           ┌───────────┐            │
│                                           │ output    │ (INTERFACE │
│                                           │ (unconnected) BOUNDARY)│
│                                           └───────────┘            │
└─────────────────────────────────────────────────────────────────────┘
```

### Transport Classification

**The honest framing**: Read/write isn't the real concern. We care more about *some* reads (secrets, credentials) than *some* writes (temp caches). The actual concern is **risk/interest** — what do we need to track for testing, isolation, and safety?

**Ownership vs Security**: The appendix describes a Query/Journal/Command model. That's an **ownership classification** (who controls what's being read/written), not a security classification. A "Query" can still exfiltrate secrets; a "Journal" can still persist sensitive data; a "Query-like" CLI command can execute arbitrary code.

**For V0/V1**: Use fermi-style risk categories (low/medium/high/extreme) rather than structural read/write classification. Risk is assessed per-transport based on domain and scope, not just ownership.

**Future-proofing**: Even if not enforced in V0, transport ops should carry metadata for later policy:

```rust
/// Metadata for risk assessment (V1+)
pub struct TransportMeta {
    pub risk: RiskLevel,             // Low | Medium | High | Extreme
    pub domain: TransportDomain,     // Git | Cargo | LLM | FS | HTTP | ...
    pub scope: TransportScope,       // RepoPath(...) | ToolHome(...) | NetworkHost(...)
}

pub enum RiskLevel { Low, Medium, High, Extreme }
```

Later "risk profiles" become a policy over `(risk, domain, scope)` instead of invasive refactoring.

**See**: [Appendix: Transport Classification Details](#appendix-transport-classification-details) for the full Query/Journal/Command ownership model if needed later.

### Scope Purity: SubDag Encapsulation

A SubDag is **hermetic** — its internal nodes cannot reach outside:

```
SPEC.md §2.7: "An opaque node's observable effects on the graph are
limited to its declared output ports. Any communication between nodes
that bypasses edges is a contract violation."
```

Applied to SubDags:

1. **Inputs**: SubDag internals see only what's passed through entrypoints
2. **Outputs**: Results leave only through declared interface boundaries
3. **No leakage**: Inner nodes cannot reference sibling nodes in parent DAG
4. **Resolution order**: SubDag must fully resolve before parent sees outputs

```
┌─────────────────────── Parent DAG ───────────────────────────┐
│                                                               │
│   ┌─────────┐         ┌─────────────────────────────┐        │
│   │ NodeA   │────────▶│       SubDag (Review)       │        │
│   └─────────┘   in    │                             │   out  │
│                ──────▶│  [inner nodes isolated]     │───────▶│
│   ┌─────────┐         │                             │        │
│   │ NodeB   │    ✗    │  Cannot see NodeA or NodeB  │        │
│   └─────────┘  ──────▶│  directly, only via inputs  │        │
│                       └─────────────────────────────┘        │
└───────────────────────────────────────────────────────────────┘
```

**Implication for ReviewPhase**: The parent doesn't know ReviewPhase uses an LLM internally. It only sees: `(artifact, criteria) → findings`.

### Phase Taxonomy: Reasoning vs Action

Phases are classified by whether they **decide** or **act**:

| Phase | Type | Risk Level | Reasoning |
|-------|------|------------|-----------|
| **ReviewPhase** | Reasoning | Low | LLM calls observe code, don't mutate it. Output is data (findings), not action. |
| **ImplementationPhase** | Reasoning | Medium | Codex queries produce artifacts (patches). Durability writes are tool-owned. User artifacts are not mutated. |
| **TestPhase** | Reasoning | Medium | Running tests observes behavior. May write to tool-owned caches. |
| **ApplyPhase** | Action | High | File writes, patch application. Mutates user artifacts. |
| **CommitPhase** | Action | High | Git commit mutates the repository. |

**The invariant**: High-risk mutation (Action Phases) only occurs in explicitly designated action subdags. Reasoning phases are hermetic and replayable — they produce *artifacts* (patches, plans, findings) but don't apply them.

**Static validation**: A reasoning phase subdag can be verified to contain no high-risk transport ops. This makes "ReviewPhase is read-only" a structural property, not just a naming convention.

---

## Part 1: Fractal Workflow Architecture

### The Meta-Workflow

A complete development cycle is a DAG of phases, where **ReviewPhase** appears multiple times:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           DevWorkflow (Meta-DAG)                            │
│                                                                             │
│  ┌───────────────┐    ┌─────────────┐    ┌─────────────────────────────┐   │
│  │ Requirements  │───▶│   Design    │───▶│     ImplementationPhase     │   │
│  │ (future)      │    │ (future)    │    │     (Codex session)         │   │
│  └───────────────┘    └──────┬──────┘    └──────────────┬──────────────┘   │
│                              │                          │                   │
│                              ▼                          ▼                   │
│                       ┌─────────────┐            ┌─────────────┐           │
│                       │ ReviewPhase │            │ ReviewPhase │           │
│                       │ (design)    │            │ (code)      │           │
│                       └─────────────┘            └──────┬──────┘           │
│                                                         │                   │
│                                                         ▼                   │
│                                                  ┌─────────────┐           │
│                                                  │  TestPhase  │           │
│                                                  │  (cargo)    │           │
│                                                  └──────┬──────┘           │
│                                                         │                   │
│                                                         ▼                   │
│                                                  ┌─────────────┐           │
│                                                  │ ReviewPhase │           │
│                                                  │ (tests)     │           │
│                                                  └──────┬──────┘           │
│                                                         │                   │
│                                                         ▼                   │
│                                                  ┌─────────────┐           │
│                                                  │ CommitPhase │           │
│                                                  │ (git)       │           │
│                                                  └─────────────┘           │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Existing Primitives We Can Reuse

| Primitive | Location | Purpose |
|-----------|----------|---------|
| `GitOps::PrepareDiff` | `lib/git-ops/` | Get diff for review context |
| `GitOps::PrepareCurrentBranch` | `lib/git-ops/` | Branch context |
| `CargoOp::test` | `lib/tools/cargo/` | Run tests |
| `CargoOp::check` | `lib/tools/cargo/` | Type checking |
| `LlmOps::PrepareChatRequest` | `lib/llm-ops/` | LLM invocation |
| `CliToolOp::Run` | `core/ir/src/transport/cli.rs` | CLI execution |
| `UpsertBuilder` | `core/ir/src/patterns/` | Idempotent operations |
| `WhileBuilder` | `core/ir/src/patterns/` | Loop with state |
| `BranchBuilder` | `core/ir/src/patterns/` | Conditional paths |

### Focus: Implementation + Review

For this iteration, we're building:
1. **ImplementationPhase** — Manage a Codex CLI session to generate/modify code
2. **ReviewPhase** — General-purpose review that works on any artifact

---

## Part 2: ImplementationPhase (Codex Session)

### Design Goals

1. **Durability**: Survive crashes, resume seamlessly
2. **No implicit state**: All dependencies are explicit as either (a) DAG edges, or (b) declared transport accesses (including journal/checkpoint reads)
3. **Idempotency**: Re-running from start skips completed work
4. **Reasoning/Action separation**: Codex invocations produce artifacts (patches); applying them is a separate Action Phase

**Codex invariant**: ImplementationPhase must invoke Codex in a mode that **cannot mutate user artifacts**. Any file changes must be emitted as artifacts (patches) and applied only by ApplyPhase. (Enforce via sandbox, read-only mount, or Codex flags that output patches without applying.)

### The Challenge: Codex is Stateful

Codex CLI maintains conversation state. Our challenge is to:
- Model this state explicitly in the DAG
- Persist it durably for crash recovery
- Resume without duplicating work or losing context

### Reasoning/Action Boundary in Implementation

```
┌─────────────────────────────────────────────────────────────────────┐
│                    ImplementationPhase (Reasoning)                   │
│                                                                     │
│  ┌───────────────────────────────────────────────────────────────┐ │
│  │                 REASONING ZONE (Low/Medium Risk)              │ │
│  │                                                               │ │
│  │  GatherContext ──▶ CodexInvoke ──▶ ParseResponse ──▶ Decide  │ │
│  │  (file reads)      (LLM call)      (pure)           (pure)   │ │
│  │                                                               │ │
│  │  Output: Artifacts (proposed changes, not yet applied)        │ │
│  └───────────────────────────────────────────────────────────────┘ │
│                              │                                      │
│                              ▼                                      │
│  ┌───────────────────────────────────────────────────────────────┐ │
│  │                 ACTION ZONE (High Risk) — SEPARATE PHASE      │ │
│  │                                                               │ │
│  │  ApplyArtifacts ──▶ WriteFiles (ApplyPhase, not here)        │ │
│  │  (explicit action phase, after reasoning completes)          │ │
│  └───────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────┘
```

The key insight: **Codex invocation is Reasoning** (asking "what should I do?" → produces patch artifacts). **Applying the changes is Action** (doing it → separate phase). This separation means we can:
- Test the reasoning without mutating files
- Review proposed changes before applying
- Roll back decisions without undoing file mutations
- DryRun the entire reasoning phase with no side effects

### Session State Model

```rust
/// Complete state of a Codex implementation session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplementationSession {
    /// Unique session identity (UUID/random) — for parallel sessions of same task
    pub session_id: String,

    /// Content hash of task inputs — for dedup/resume matching
    pub task_id: String,

    /// The task being implemented
    pub task: ImplementationTask,

    /// Completed steps (for idempotent resume)
    pub completed_steps: Vec<StepRecord>,

    /// Current Codex conversation state
    pub conversation: ConversationState,

    /// Files created/modified by this session
    pub artifacts: Vec<Artifact>,

    /// Session lifecycle
    pub status: SessionStatus,
}

// session_id vs task_id:
// - session_id: random UUID (identity) — allows parallel sessions for same task
// - task_id: content hash (dedup key) — identifies "same task" for resume/lookup
// This prevents: (a) collision when two developers run same task, (b) accidental
// resume of unrelated session, (c) inability to have parallel experiments

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplementationTask {
    pub description: String,
    pub context_files: Vec<String>,      // Files to read for context
    pub target_files: Vec<String>,       // Files to create/modify
    pub constraints: Vec<String>,        // Requirements to satisfy
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepRecord {
    pub step_id: String,                 // Content hash of step inputs
    pub step_type: StepType,
    pub completed_at: u64,               // Unix timestamp
    pub outputs: HashMap<String, Value>, // Step outputs for downstream
    pub cache_policy: CachePolicy,       // When to rerun this step
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StepType {
    GatherContext,
    CodexInvocation { turn: u32 },
    ReviewInvocation,
    /// Prepare patch/plan (Query-only) — actual apply is a Command phase outside
    PrepareArtifacts,
    Checkpoint,
}

/// LLMs are nondeterministic — need explicit cache policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CachePolicy {
    /// Never rerun if step_id exists (crash recovery)
    Immutable,
    /// Always rerun, ignore cache
    Refresh,
    /// Rerun if cached result is older than duration
    RefreshIfOlderThan(u64),  // millis
    /// Rerun if model version changed
    RefreshIfModelChanged { model_fingerprint: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationState {
    /// Use Codex's native session ID — don't duplicate conversation state
    /// Resume via: `codex resume <id>` or `codex exec resume <id>`
    pub codex_session_id: Option<String>,
    pub mode: CodexMode,
    pub turn_count: u32,
    pub last_prompt_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CodexMode {
    Interactive,  // `codex` — human-in-the-loop
    Exec,         // `codex exec` — non-interactive
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionStatus {
    InProgress,
    AwaitingReview,
    Completed,
    Failed { error: String },
}
```

### Durability Strategy: Step-Level Checkpointing

Instead of checkpointing the entire session, we checkpoint **per step**:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    ImplementationPhase (Durable)                            │
│                                                                             │
│  For each step:                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                     StepUpsert Pattern                              │   │
│  │                                                                     │   │
│  │  ┌─────────────────┐     ┌─────────────────┐                       │   │
│  │  │ CheckStepDone   │────▶│   Guard         │                       │   │
│  │  │ (read storage)  │     │ (skip if done)  │                       │   │
│  │  └─────────────────┘     └────────┬────────┘                       │   │
│  │                                   │                                 │   │
│  │                         ┌─────────▼─────────┐                      │   │
│  │                         │   ExecuteStep     │                      │   │
│  │                         │   (actual work)   │                      │   │
│  │                         └─────────┬─────────┘                      │   │
│  │                                   │                                 │   │
│  │                         ┌─────────▼─────────┐                      │   │
│  │                         │   SaveStepResult  │                      │   │
│  │                         │   (write storage) │                      │   │
│  │                         └───────────────────┘                      │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  Steps execute in sequence, each wrapped in StepUpsert:                     │
│                                                                             │
│  1. GatherContext ──▶ 2. CodexInvoke ──▶ 3. ApplyChanges ──▶ 4. Repeat?   │
│                              │                                              │
│                              └─── (loop until complete or max turns)        │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Step ID: Content-Addressed

Each step has a deterministic ID based on its inputs:

```rust
fn step_id(step_type: &StepType, inputs: &HashMap<String, Value>) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(&serde_json::to_vec(step_type).unwrap());

    // Sort keys for determinism
    let mut keys: Vec<_> = inputs.keys().collect();
    keys.sort();
    for key in keys {
        hasher.update(key.as_bytes());
        hasher.update(&serde_json::to_vec(&inputs[key]).unwrap());
    }

    hasher.finalize().to_hex()[..16].to_string()
}
```

**Implications**:
- Same inputs → same step ID → skip if already done
- Changed inputs (e.g., new context) → new step ID → re-execute
- No manual invalidation needed

**World state inputs**: A step is cacheable only relative to a declared **input snapshot** of the world state it depends on. Step inputs should include:
- `GatherContext`: `git HEAD` (or diff hash) + list of files read
- `CargoOp::test`: tool version (cargo + rustc), relevant env vars, manifest path
- LLM steps: model name + fingerprint, system prompt hash, temperature/seed
Otherwise caching becomes incorrect when the repo or environment changes.

### Crash Recovery Flow

```
┌──────────────────────────────────────────────────────────────────────────┐
│                         Resume After Crash                               │
│                                                                          │
│  1. Load session manifest from ~/.gunbc/sessions/{session_id}.json       │
│                                                                          │
│  2. For each step in the workflow:                                       │
│     ┌────────────────────────────────────────────────────────────────┐  │
│     │  step_id = compute_step_id(step_type, current_inputs)          │  │
│     │                                                                │  │
│     │  if session.completed_steps.contains(step_id):                 │  │
│     │      # Step already done                                       │  │
│     │      outputs = load_step_outputs(step_id)                      │  │
│     │      feed outputs to downstream steps                          │  │
│     │  else:                                                         │  │
│     │      # Step not done (or inputs changed)                       │  │
│     │      outputs = execute_step(step_type, current_inputs)         │  │
│     │      save_step_result(step_id, outputs)                        │  │
│     │      update session.completed_steps                            │  │
│     └────────────────────────────────────────────────────────────────┘  │
│                                                                          │
│  3. Continue from where we left off — no duplicate LLM calls            │
└──────────────────────────────────────────────────────────────────────────┘
```

### Storage Layout

```
~/.gunbc/
├── sessions/
│   └── impl-{session_id}/
│       ├── manifest.json           # ImplementationSession
│       ├── steps/
│       │   ├── {step_id}.json      # StepRecord with outputs
│       │   └── ...
│       └── artifacts/
│           ├── {file_path}.patch   # Changes to apply
│           └── ...
├── cache/
│   └── codex/
│       └── context-{hash}.json     # Cached context gathering
└── config/
    └── codex.toml
```

### ImplementationOps

```rust
#[derive(Debug, Clone)]
pub enum ImplementationOps {
    /// Initialize or resume a session
    ///
    /// Inputs:
    /// - `task`: ImplementationTask
    /// - `storage_path`: String
    ///
    /// Outputs:
    /// - `session`: ImplementationSession
    /// - `is_resume`: Bool
    InitSession,

    /// Check if a step is already complete
    ///
    /// Inputs:
    /// - `session`: ImplementationSession
    /// - `step_type`: StepType
    /// - `step_inputs`: HashMap<String, Value>
    ///
    /// Outputs:
    /// - `done`: Bool
    /// - `cached_outputs`: Option<HashMap<String, Value>>
    CheckStepDone,

    /// Record step completion
    ///
    /// Inputs:
    /// - `session`: ImplementationSession
    /// - `step_id`: String
    /// - `step_type`: StepType
    /// - `outputs`: HashMap<String, Value>
    ///
    /// Outputs:
    /// - `session`: ImplementationSession (updated)
    RecordStep,

    /// Prepare Codex CLI invocation
    ///
    /// Inputs:
    /// - `conversation`: ConversationState
    /// - `prompt`: String
    /// - `config`: CodexConfig
    ///
    /// Outputs:
    /// - `request`: ShellRequest
    PrepareCodexRequest,

    /// Parse Codex CLI response
    ///
    /// Inputs:
    /// - `stdout`: String
    /// - `stderr`: String
    /// - `exit_code`: Int
    /// - `conversation`: ConversationState
    ///
    /// Outputs:
    /// - `response`: CodexResponse
    /// - `conversation`: ConversationState (updated)
    /// - `complete`: Bool
    ParseCodexResponse,

    /// Extract file changes from Codex response
    ///
    /// Inputs:
    /// - `response`: CodexResponse
    ///
    /// Outputs:
    /// - `artifacts`: Vec<Artifact>
    /// - `needs_followup`: Bool
    ExtractArtifacts,
}
```

### CodexConfig

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexConfig {
    /// Model to use (e.g., "gpt-4o", "o3")
    pub model: String,

    /// Approval mode for file changes
    pub approval_mode: ApprovalMode,

    /// Working directory for Codex
    pub working_dir: Option<String>,

    /// Additional CLI flags
    pub extra_args: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApprovalMode {
    /// Auto-approve all changes (for automation)
    AutoApprove,
    /// Require explicit approval (for interactive)
    RequireApproval,
    /// Auto-approve safe changes, prompt for risky ones
    SafeAutoApprove,
}

impl Default for CodexConfig {
    fn default() -> Self {
        Self {
            model: "gpt-4o".into(),
            approval_mode: ApprovalMode::SafeAutoApprove,
            working_dir: None,
            extra_args: vec![],
        }
    }
}
```

---

## Part 3: ReviewPhase (Pure Reconciliation)

### Design Principle: Review = Reconciliation + Candidate Repairs

**ReviewPhase is maximally simple**: given an artifact and criteria, report where the artifact diverges from the criteria, and optionally propose candidate repairs. That's it.

- **No verdict** — that's orchestration
- **No severity levels** — either it diverges or it doesn't
- **No decisions** — that's orchestration (whether to apply, iterate, block)
- **Candidate repairs, not commands** — proposals are data; orchestration chooses whether to apply

```
ReviewPhase: (artifact, criteria) → (findings, candidate_remediations)
```

The reviewer doesn't decide what happens with findings or whether repairs are applied. Different criteria produce different findings. The orchestration layer decides what to do.

### Core Types

```rust
/// What is being reviewed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Artifact {
    /// Source code (diff or full file)
    Code {
        content: String,
        file_path: Option<String>,
        language: Option<String>,
        is_diff: bool,
    },
    /// Design document
    Design {
        content: String,
        format: DocFormat,
    },
    /// Test results
    TestOutput {
        stdout: String,
        stderr: String,
        exit_code: i32,
    },
    /// Generic content
    Text {
        content: String,
        artifact_type: String,
    },
}

/// Criteria = what to check for
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Criteria {
    pub name: String,              // e.g., "coherence", "quality", "security"
    pub description: String,       // What this criteria is about
    pub checks: Vec<Check>,        // Specific things to verify
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Check {
    pub id: String,                // Unique identifier
    pub question: String,          // What to ask about the artifact
    pub examples: Vec<String>,     // Example violations (optional)
}

/// Review output = pure reconciliation + candidate repairs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewOutput {
    pub criteria_name: String,            // Which criteria was applied
    pub source: String,                   // Provenance: "llm:gpt-4o", "cargo:clippy", etc.
    pub findings: Vec<Finding>,           // Where artifact diverges from criteria
    pub candidate_remediations: Option<CandidateRemediations>, // Optional repair proposals
    pub summary: String,                  // Natural language summary
}

/// Container for merging multiple reviews (same artifact, different criteria/sources)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewBundle {
    pub artifact_id: String,              // Hash of artifact being reviewed
    pub reviews: Vec<ReviewOutput>,       // Each has criteria_name + source
}

/// Finding = divergence from criteria
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    /// Stable identity for merging/dedup (hash of canonical fields)
    pub id: String,
    pub check_id: String,                 // Which Check this relates to
    pub location: Option<Location>,
    /// Stable canonical key for cross-iteration matching (reviewer-provided)
    /// Should NOT include line numbers; should include symbol/file/invariant
    pub issue_key: String,
    pub observation: String,              // What was found (may vary in wording)
    /// Example way to satisfy criteria (candidate, not command)
    pub candidate_fix: Option<String>,
}

/// Stable finding ID = hash of (check_id, location context, issue_key)
/// Using issue_key instead of observation text for stability across LLM wording changes
fn finding_id(check_id: &str, location: &Option<Location>, issue_key: &str) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(check_id.as_bytes());
    if let Some(loc) = location {
        // For diffs: include hunk context, not line numbers
        hasher.update(&serde_json::to_vec(loc).unwrap());
    }
    hasher.update(issue_key.as_bytes());
    hasher.finalize().to_hex()[..16].to_string()
}

/// Candidate repairs (proposals, not commands)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateRemediations {
    pub goals: Vec<String>,               // Acceptance criteria
    pub tasks: Vec<CandidateTask>,        // Proposed edits
    pub constraints: Vec<String>,         // "do not change public API", etc.
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateTask {
    pub finding_id: String,               // Links to Finding.id
    pub file: Option<String>,
    pub intent: String,                   // What to change
    pub candidate_patch: Option<String>,  // Optional diff snippet (proposal)
    pub validation: Vec<String>,          // Commands or checks to pass
}

/// Location in source — handles both files and diffs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Location {
    /// Position in a file
    FileLine {
        file: String,
        line: u32,
    },
    /// Span in a file
    Span {
        file: String,
        start: u32,
        end: u32,
    },
    /// Position in a diff (need to distinguish old vs new)
    DiffLine {
        file: String,
        line: u32,
        side: DiffSide,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiffSide {
    Old,  // Line in the removed version
    New,  // Line in the added version
}
```

**Note**: No `FindingLevel`, no `Verdict`, no `NextStep`. Those belong in orchestration. `CandidateRemediations` is pure data (proposals) — orchestration decides whether to execute them.

---

## Part 4: Review Cycle (Orchestration)

### What Orchestration Does

ReviewPhase is pure reconciliation. The **Review Cycle** is orchestration that:
- Decides which criteria to run (fully configurable)
- Decides whether to iterate on findings
- Decides when to stop

No hardcoded stages or order — criteria are user-defined.

### Orchestration Types

```rust
/// Orchestration decides what to do with findings
pub struct ReviewCycleConfig {
    /// Which criteria to apply (order is configurable, not mandated)
    pub criteria: Vec<Criteria>,

    /// How to decide whether to iterate
    pub iterate_policy: IteratePolicy,

    /// Maximum iterations per criteria
    pub max_iterations: u32,
}

pub enum IteratePolicy {
    /// Iterate if any findings have candidate_fix
    OnRemediableFindings,
    /// Iterate on any findings (not just those with hints)
    OnAnyFindings,
    /// Never auto-iterate, always proceed
    AlwaysProceed,
    /// Custom logic
    Custom(fn(&[Finding]) -> bool),
}

/// What the orchestration layer produces
pub struct CycleResult {
    /// All findings from all criteria runs
    pub all_findings: Vec<(String, Vec<Finding>)>,  // (criteria_name, findings)

    /// Final decision
    pub outcome: CycleOutcome,
}

pub enum CycleOutcome {
    /// All criteria passed (no findings, or findings were fixed)
    Complete,
    /// Stopped iterating (max iterations or no more fixes)
    Stabilized { remaining_findings: usize },
    /// Needs human decision
    NeedsHuman { reason: String },
}
```

### Why This Separation Matters

1. **ReviewPhase stays pure**: `(artifact, criteria) → findings` — nothing more
2. **Criteria are user-defined**: security, style, correctness — whatever you configure
3. **Orchestration is explicit**: decisions about iterating, blocking, etc. are visible
4. **Order is flexible**: sequential usually makes sense (hard to discuss design when basic correctness fails), but not enforced

### ReviewOps

```rust
#[derive(Debug, Clone)]
pub enum ReviewOps {
    /// Build review prompt from artifact + criteria
    ///
    /// Inputs:
    /// - `artifact`: Artifact
    /// - `criteria`: Criteria
    /// - `context`: Option<String>
    ///
    /// Outputs:
    /// - `prompt`: String
    /// - `system_prompt`: String
    PrepareReviewPrompt,

    /// Parse LLM response into structured findings
    ///
    /// Inputs:
    /// - `response`: String (LLM output)
    /// - `criteria`: Criteria (for check_id mapping)
    ///
    /// Outputs:
    /// - `output`: ReviewOutput
    ParseReviewResponse,

    /// Merge findings from multiple review sources
    ///
    /// Inputs:
    /// - `outputs`: Vec<ReviewOutput>
    ///
    /// Outputs:
    /// - `merged`: ReviewOutput
    MergeOutputs,
}
```

**Note**: No `DetermineVerdict` in ReviewOps — that's orchestration (see Part 4).

### ReviewPhase DAG

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                            ReviewPhase SubDag                               │
│                                                                             │
│  NOTE: Reuses lib/llm-ops chat subdag — don't re-implement the LLM chain!  │
│                                                                             │
│  Entrypoints:                                                               │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐                       │
│  │ artifact │ │ criteria │ │ context  │ │  config  │                       │
│  └────┬─────┘ └────┬─────┘ └────┬─────┘ └────┬─────┘                       │
│       │            │            │            │                              │
│       └────────────┴────────────┴────────────┘                              │
│                           │                                                 │
│                           ▼                                                 │
│              ┌─────────────────────────┐                                   │
│              │   PrepareReviewPrompt   │ ◀── PURE: build LLM prompt        │
│              │   (ReviewOps)           │                                   │
│              └────────────┬────────────┘                                   │
│                           │                                                 │
│                           ▼                                                 │
│              ┌─────────────────────────────────────────────────┐           │
│              │     EMBED: lib/llm-ops chat_completion subdag   │           │
│              │  ┌─────────────────────────────────────────┐   │           │
│              │  │ PrepareChatRequest → ExecuteQuery →     │   │           │
│              │  │ ParseChatResponse                       │   │           │
│              │  └─────────────────────────────────────────┘   │           │
│              │  (reuse existing graph, don't redefine)        │           │
│              └────────────┬────────────────────────────────────┘           │
│                           │                                                 │
│                           ▼                                                 │
│              ┌─────────────────────────┐                                   │
│              │  ParseReviewResponse    │ ◀── PURE: structure findings      │
│              │  (ReviewOps)            │                                   │
│              └────────────┬────────────┘                                   │
│                           │                                                 │
│  Workflow Boundaries:     ▼                                                 │
│  ┌───────────────┐ ┌──────────┐                                            │
│  │   findings    │ │ summary  │                                            │
│  │ Vec<Finding>  │ │  String  │                                            │
│  └───────────────┘ └──────────┘                                            │
│                                                                             │
│  NOTE: No verdict, no next_step — that's orchestration (Part 4)            │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Multi-Source Review (Parallel)

For comprehensive review, run multiple reviewers in parallel:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                      MultiReview (Parallel + Merge)                         │
│                                                                             │
│            ┌─────────────────────────────────────────────┐                 │
│            │             artifact + criteria             │                 │
│            └─────────┬───────────────┬──────────────┬────┘                 │
│                      │               │              │                       │
│            ┌─────────▼─────┐ ┌───────▼───────┐ ┌────▼────────┐            │
│            │ LLM Review    │ │ Clippy/Lint   │ │ Type Check  │            │
│            │ (ReviewPhase) │ │ (CargoOp)     │ │ (CargoOp)   │            │
│            └───────┬───────┘ └───────┬───────┘ └──────┬──────┘            │
│                    │                 │                │                    │
│                    └─────────────────┴────────────────┘                    │
│                                      │                                     │
│                           ┌──────────▼──────────┐                          │
│                           │    MergeOutputs     │                          │
│                           │    (ReviewOps)      │                          │
│                           └──────────┬──────────┘                          │
│                                      │                                     │
│                              ┌───────▼───────┐                             │
│                              │ unified output│                             │
│                              └───────────────┘                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Part 5: Composition Example

### Implement-and-Review Workflow

Compose ImplementationPhase + Review Cycle (orchestration) into a complete workflow:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    ImplementAndReview Workflow                              │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                    ImplementationPhase                              │   │
│  │  (Codex session with durability)                                    │   │
│  └─────────────────────────────┬───────────────────────────────────────┘   │
│                                │                                            │
│                                │ artifacts: Vec<Artifact>                   │
│                                ▼                                            │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                  Review Cycle (Orchestration)                       │   │
│  │  criteria: [configured by caller]                                   │   │
│  │                                                                     │   │
│  │  For each artifact, for each configured criteria:                   │   │
│  │  ┌─────────────────────────────────────────────────────────────┐   │   │
│  │  │                    ReviewPhase                              │   │   │
│  │  │  criteria: <from config>                                    │   │   │
│  │  │  outputs: (findings, summary)                               │   │   │
│  │  └─────────────────────────────────────────────────────────────┘   │   │
│  │  If findings → iterate (apply fix, re-review) or proceed            │   │
│  └─────────────────────────────┬───────────────────────────────────────┘   │
│                                │                                            │
│                                │ cycle_result: CycleResult                  │
│                                ▼                                            │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                       BranchBuilder                                 │   │
│  │  condition: cycle_result.outcome == Complete                        │   │
│  │                                                                     │   │
│  │  ┌─────────────────┐              ┌─────────────────┐              │   │
│  │  │ True: Complete  │              │ False: Stabilized│              │   │
│  │  │ → apply changes │              │ → human decides  │              │   │
│  │  └─────────────────┘              └─────────────────┘              │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Builder API (Proposed)

```rust
/// Build a complete implement-and-review workflow
pub fn build_implement_and_review(
    task: ImplementationTask,
    cycle_config: ReviewCycleConfig,
    workflow_config: WorkflowConfig,
) -> Result<Dag<WorkflowOps>, BuilderError> {
    let mut builder = DagBuilder::new("implement-and-review");

    // Phase 1: Implementation
    let impl_phase = ImplementationPhaseBuilder::new()
        .with_task(task)
        .with_durability(workflow_config.storage_path.clone())
        .build()?;

    let impl_node = builder.add_subdag("implement", impl_phase)?;

    // Phase 2: Review cycle (orchestration handles iteration)
    let review_cycle = ReviewCycleBuilder::new()
        .with_criteria(cycle_config.criteria)
        .with_iterate_policy(cycle_config.iterate_policy)
        .with_max_iterations(cycle_config.max_iterations)
        .build()?;

    let review_node = builder.add_subdag_after("review-cycle", review_cycle, &impl_node)?;
    builder.connect(impl_node.output("artifacts"), review_node.input("artifacts"))?;

    // Phase 3: Handle cycle outcome
    let check_node = builder.add_node_after(
        "handle-outcome",
        WorkflowOps::HandleCycleOutcome,
        &review_node,
    )?;
    builder.connect(review_node.output("cycle_result"), check_node.input("result"))?;

    // Set boundaries
    builder.mark_entrypoint("task", impl_node.input("task"))?;
    builder.mark_boundary("result", check_node.output("final_result"))?;

    builder.build()
}
```

---

## Implementation Tasks

### Phase 0: Core Architecture (if generalizing)

- [ ] Consider adding risk categories to transport operations (fermi-style: low/medium/high/extreme)
- [ ] Consider updating AGENT.md with risk classification principle
- [ ] (Future) Consider Query/Command structural classification — see appendix

### V0: Minimal Useful Pipeline (First Target)

**Goal**: Review current branch diff + optional cargo checks → JSON report

**What already exists** (reuse, don't rebuild):
- ✅ `GitOps::PrepareDiff` / `ParseDiff` — get diff context
- ✅ `LlmOps::PrepareChatRequest` / `ParseChatResponse` — LLM invocation
- ✅ `CargoOp::check` / `CargoOp::test` — cargo tooling
- ✅ `TransportOps::Execute` — I/O boundary
- ✅ Pattern builders: `UpsertBuilder`, `LoopBuilder`, `BranchBuilder`

**V0 flow**:
```
1. GitOps::PrepareDiff → Execute → GitOps::ParseDiff
2. (Optional) CargoOp::check/test/clippy
3. ReviewOps::PrepareReviewPrompt(diff + tool_outputs + criteria)
4. [lib/llm-ops chat subdag]  ← reuse existing
5. ReviewOps::ParseReviewResponse
6. Output: JSON ReviewOutput with findings, summary
```

**What to build for V0**:
- [ ] `lib/review/` crate with types:
  - `Artifact`, `Criteria`, `Check`
  - `Finding` (hash-based `id`, `issue_key`, `candidate_fix`), `Location` (with `DiffSide`)
  - `ReviewOutput`, `ReviewBundle`, `CandidateRemediations`, `CandidateTask`
- [ ] `ReviewOps::PrepareReviewPrompt` — assemble prompt from artifact + criteria
- [ ] `ReviewOps::ParseReviewResponse` — parse LLM response into findings + remediation plan
- [ ] Simple CLI: `gunbc review --diff HEAD~1` → prints JSON

**Note**: V0 does NOT include orchestration (Review Cycle) — just single-criteria review. `CandidateRemediations` is output but not executed.

**V0 explicitly does NOT include**:
- Durability / checkpointing
- Codex CLI integration
- Multi-turn conversations
- Apply/commit commands

**V0 Crispness Checklist** (acceptance criteria for architecture alignment):
- [ ] **Structural**: The `ReviewPhase` subdag validates as "contains no high-risk transport ops" (recursively)
- [ ] **Behavioral**: DryRun can run the entire V0 graph without touching the repo
- [ ] **Output contract**: Review output is machine-parseable JSON with schema version
- [ ] **Provenance**: Each finding indicates `(check_id, location?, observation)` and optional `candidate_fix`
- [ ] **Stable IDs**: Findings have stable-ish IDs via `issue_key`; merging is possible

---

### V1: Review Cycle + Codex Integration

After V0 works:
- [ ] Add Review Cycle orchestration (configurable criteria)
- [ ] For findings with `candidate_fix`, feed back into Codex
- [ ] ImplementationPhase runs Codex CLI (Query), outputs patch artifact
- [ ] ReviewPhase reviews the patch with same criteria
- [ ] Iterate until no findings or max iterations
- [ ] **Keep "apply patch" as explicit Command** (separate from reasoning loop)

---

### Full Implementation Phases

#### Phase 1: Core Types (V0 blocker)

- [ ] Create `lib/review/` crate
- [ ] Define types:
  - `Artifact`, `Criteria`, `Check`
  - `Finding` (hash-based id, `source` provenance, `remediation_hint`), `Location` (with `DiffSide`)
  - `ReviewOutput`, `ReviewBundle`, `CandidateRemediations`, `CandidateTask`
- [ ] Define orchestration types: `ReviewCycleConfig`, `IteratePolicy`, `CycleResult`, `CycleOutcome`
- [ ] Implement `ReviewOps` with `Executable` trait
- [ ] Wire up `ReviewPhaseBuilder` using existing patterns

#### Phase 2: Risk Classification (future)

- [ ] Define fermi-style risk categories (low/medium/high/extreme)
- [ ] Add risk annotation to transport operations
- [ ] (Future) Consider Query/Journal/Command structural classification if needed — see appendix

#### Phase 3: ImplementationPhase (V1)

- [ ] Create `lib/codex/` crate
- [ ] Define types: `ImplementationSession`, `StepRecord`, `ConversationState`, `CodexMode`
- [ ] Add `CachePolicy` for nondeterministic LLM steps
- [ ] Implement `ImplementationOps`
- [ ] Integrate Codex CLI via `codex_session_id` + native resume (`codex resume <id>`)
- [ ] Add Journal transport for session state persistence

#### Phase 4: Durability (V1+)

- [ ] Implement `ExecuteJournal` for checkpoint storage
- [ ] Implement `CheckStepDone` / `RecordStep` operations with `CachePolicy` support
- [ ] Add content-addressed step ID computation
- [ ] Test crash recovery

#### Phase 5: Composition & CLI

- [ ] Build `ImplementAndReviewBuilder` (composes V0 + V1)
- [ ] Add `gunbc review` command (V0)
- [ ] Add `gunbc implement` command (V1)
- [ ] Add `--resume` support

---

## Open Questions

1. ~~**Codex session files**~~: **Resolved** — Codex CLI has native session persistence:
   - Interactive: `codex resume [SESSION_ID]`
   - Non-interactive: `codex exec resume <SESSION_ID>`
   - State stored under `CODEX_HOME` (default `~/.codex`)
   - **Strategy**: Store `codex_session_id` as pointer, use native resume

2. **Multi-file implementation**: When Codex generates changes across many files, should we review:
   - All files together (single review)?
   - Each file separately (parallel reviews)?
   - Changed "modules" (heuristic grouping)?

3. **Feedback loop**: When review finds issues, how do we feed findings back to Codex?
   - Option A: Append findings to conversation, ask for fixes
   - Option B: New session with findings as context
   - Option C: Human-in-the-loop decides

4. **Automated vs LLM review**: When should we use Clippy/lint vs LLM?
   - Option A: Always run both in parallel, merge
   - Option B: Clippy first, LLM for things Clippy can't catch
   - Option C: Configurable per-rubric

---

## Related Patterns

- **Upsert**: `core/ir/src/patterns/upsert.rs` — idempotent operations
- **While/Retry**: `core/ir/src/patterns/repeat.rs` — iteration with state
- **Loop**: `core/ir/src/patterns/loop_pattern.rs` — per-item processing
- **Branch**: `core/ir/src/patterns/branch.rs` — conditional paths
- **CLI Tools**: `core/ir/src/transport/cli.rs` — CliToolDef, CliToolOp
- **LLM Ops**: `lib/llm-ops/src/` — existing chat completion patterns
- **Git Ops**: `lib/git-ops/src/` — diff, branch operations

---

## Appendix: Transport Classification Details

**Status**: Future consideration. This is an **ownership classification** model (who controls what's being read/written), not a security classification. Preserved here for reference when we have concrete use cases.

**Background conversation**: The repo uses read/write as the primary discrimination axis. This is convenient for testing and reasoning about purity, but it's not structurally fundamental. Read/write isn't the real concern — we care more about *some* reads (secrets, credentials) than *some* writes (temp caches). The actual concern is **risk/interest**. We'll likely need domain-specific risk profiles. Writes can structure their own risk categories (e.g., "repo mutation" vs "cache update" vs "credential storage"). The current read/write split is a placeholder for that richer model.

**When to use this model**: If you need structural enforcement ("this subdag contains no Commands") beyond risk-level annotation. The main document uses fermi-style risk levels (low/medium/high/extreme) which are sufficient for V0/V1.

### Query / Journal / Command Classification (Ownership Model)

Transport classified by typed execute ops, not flags:

| Class | Execute Op | Scope | Examples |
|-------|------------|-------|----------|
| **Query** | `ExecuteQuery` | Read external state | LLM calls, file reads, git diff, HTTP GET |
| **Journal** | `ExecuteJournal` | Write tool-owned cache/state | Checkpoints, session state, response cache |
| **Command** | `ExecuteCommand` | Mutate user artifacts / external systems | File writes, git commit, apply patch |

```rust
pub enum TransportOps {
    ExecuteQuery,
    ExecuteJournal,
    ExecuteCommand,
}
```

### Journal Scope Contract

Journal is "tool-owned state" — but without enforcement, it becomes "Command in disguise."

```rust
pub struct JournalScope {
    pub allowed_paths: Vec<PathBuf>,  // e.g., ["~/.gunbc/", "$TMPDIR/gunbc-"]
    pub resource_ids: Vec<String>,
}
```

Executor enforces: `ExecuteJournal` can only write within `allowed_paths`.

### DryRun Modes

```rust
pub enum DryRunMode {
    Strict,                        // mock all
    Safe { allow_journal: bool },  // allow Query, optionally Journal, block Command
    Off,                           // real execution
}
```

| Mode | Query | Journal | Command |
|------|-------|---------|---------|
| **Strict** | mock | mock | mock |
| **Safe(journal=true)** | allow | allow | mock |
| **Safe(journal=false)** | allow | mock | mock |
| **Off** | allow | allow | allow |

### Phase-level Validation (if using this model)

| Phase | Validation Rule |
|-------|-----------------|
| **ReviewPhase** | No `ExecuteCommand`, no `ExecuteJournal` |
| **ImplementationPhase** | No `ExecuteCommand` |
| **TestPhase** | No `ExecuteCommand` |
| **ApplyPhase** | `ExecuteCommand` allowed |
