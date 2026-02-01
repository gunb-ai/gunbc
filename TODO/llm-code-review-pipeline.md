# LLM Code Review Pipeline

**Status**: Design
**Date**: 2026-02-01

## Goal

Build a fractal DAG-based development workflow with:
1. **ImplementationPhase** — Codex CLI session management with crash-resilient durability
2. **ReviewPhase** — General-purpose, reusable review process for any artifact type

These phases compose into larger workflows (Requirements → Design → Implement → Review → Test → Commit) but are independently useful.

---

## Design Principles: Transport Classification & Scope Purity

This section reconciles with AGENT.md ("Nodes are Pure, Boundaries are Structural") and SPEC.md (§2.7 "No side channels", §6 "Node Contract").

### Transport Classification: Query vs Command

Not all I/O is equal. We distinguish:

| Class | Nature | Where Allowed | Examples |
|-------|--------|---------------|----------|
| **Query** | Read-like, no world mutation | Inside DAGs | LLM calls, file reads, git diff, HTTP GET |
| **Command** | Write-like, mutates state | DAG boundaries only | File writes, git commit, apply patch, HTTP POST with side effects |

**Rationale**: A Query is "observation" — it doesn't change the world, so the DAG remains **reasoning-focused**. A Command is "action" — it mutates state and should happen only after the DAG has decided what to do.

```
┌─────────────────────────────────────────────────────────────────────┐
│                        DAG Execution Model                          │
│                                                                     │
│   ┌─────────────────────────────────────────────────────────────┐  │
│   │              Reasoning Zone (Queries OK)                    │  │
│   │                                                             │  │
│   │   Pure nodes + Query transport (LLM, reads)                │  │
│   │   Produces: Decision / Intent / Plan                        │  │
│   └─────────────────────────────────────────────────────────────┘  │
│                              │                                      │
│                              ▼                                      │
│   ┌─────────────────────────────────────────────────────────────┐  │
│   │              Action Zone (Commands)                         │  │
│   │                                                             │  │
│   │   File writes, commits, patches applied                     │  │
│   │   Executes the intent produced above                        │  │
│   └─────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────┘
```

**Alignment with existing principles**:
- AGENT.md §4: "Nodes are Pure, Boundaries are Structural" — Queries are "less impure" than Commands
- AGENT.md §7: "No Escape Hatches" — Commands can't bypass DAG boundaries
- SPEC.md §6: "Port honesty" — Commands must flow through declared output ports

### Scope Purity: SubDag Encapsulation

A SubDag is **hermetic** — its internal nodes cannot reach outside:

```
SPEC.md §2.7: "An opaque node's observable effects on the graph are
limited to its declared output ports. Any communication between nodes
that bypasses edges is a contract violation."
```

Applied to SubDags:

1. **Inputs**: SubDag internals see only what's passed through entrypoints
2. **Outputs**: Results leave only through declared boundaries
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

**Implication for ReviewPhase**: The parent doesn't know ReviewPhase uses an LLM internally. It only sees: `(artifact, rubric) → (findings, verdict)`.

### How This Applies to Our Phases

| Phase | Transport Type | Reasoning |
|-------|----------------|-----------|
| **ReviewPhase** | Query only | LLM calls observe code, don't mutate it. Output is data (findings), not action. |
| **ImplementationPhase** | Query internally, Command at boundary | Codex queries produce artifacts. Applying artifacts is a Command. |
| **CommitPhase** | Command | Git commit mutates the repository. |
| **TestPhase** | Query | Running tests observes behavior, doesn't mutate code. |

**The pattern**: Phases that "decide" use Queries. Phases that "act" use Commands. Most DAGs should be reasoning-heavy, with Commands concentrated at the edges.

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
2. **Statelessness**: All state flows through DAG edges (no hidden state)
3. **Idempotency**: Re-running from start skips completed work
4. **Query/Command separation**: Codex invocations are Queries; file mutations are Commands at the boundary

### The Challenge: Codex is Stateful

Codex CLI maintains conversation state. Our challenge is to:
- Model this state explicitly in the DAG
- Persist it durably for crash recovery
- Resume without duplicating work or losing context

### Query/Command Boundary in Implementation

```
┌─────────────────────────────────────────────────────────────────────┐
│                    ImplementationPhase                              │
│                                                                     │
│  ┌───────────────────────────────────────────────────────────────┐ │
│  │                 REASONING ZONE (Queries)                      │ │
│  │                                                               │ │
│  │  GatherContext ──▶ CodexInvoke ──▶ ParseResponse ──▶ Decide  │ │
│  │  (file reads)      (LLM query)     (pure)           (pure)   │ │
│  │                                                               │ │
│  │  Output: Artifacts (proposed changes, not yet applied)        │ │
│  └───────────────────────────────────────────────────────────────┘ │
│                              │                                      │
│                              ▼                                      │
│  ┌───────────────────────────────────────────────────────────────┐ │
│  │                 ACTION ZONE (Commands)                        │ │
│  │                                                               │ │
│  │  ApplyArtifacts ──▶ WriteFiles                                │ │
│  │  (at DAG boundary, after reasoning completes)                 │ │
│  └───────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────┘
```

The key insight: **Codex invocation is a Query** (asking "what should I do?"). **Applying the changes is a Command** (doing it). This separation means we can:
- Test the reasoning without mutating files
- Review proposed changes before applying
- Roll back decisions without undoing file mutations

### Session State Model

```rust
/// Complete state of a Codex implementation session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplementationSession {
    /// Unique session identifier (content-addressed)
    pub session_id: String,

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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StepType {
    GatherContext,
    CodexInvocation { turn: u32 },
    ApplyChanges,
    Checkpoint,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationState {
    pub messages: Vec<Message>,
    pub turn_count: u32,
    pub codex_session_file: Option<String>, // Codex's own session file if applicable
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

## Part 3: ReviewPhase (General-Purpose)

### Design Goals

1. **Reusability**: Same ReviewPhase works for code, designs, tests, docs
2. **Modularity**: ReviewPhase is a self-contained SubDag
3. **Composability**: Easily plugs into any workflow point

### Abstraction: Artifact + Rubric → Findings

The key insight is that **all reviews share the same shape**:

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        ReviewPhase (Generic)                            │
│                                                                         │
│  Inputs:                                                                │
│  ┌────────────────┐  ┌────────────────┐  ┌────────────────────────┐   │
│  │   artifact     │  │    rubric      │  │  context (optional)    │   │
│  │  (what)        │  │  (criteria)    │  │  (background info)     │   │
│  └───────┬────────┘  └───────┬────────┘  └───────────┬────────────┘   │
│          │                   │                       │                 │
│          └───────────────────┴───────────────────────┘                 │
│                              │                                         │
│                              ▼                                         │
│                    ┌─────────────────┐                                 │
│                    │  PrepareReview  │                                 │
│                    │  (build prompt) │                                 │
│                    └────────┬────────┘                                 │
│                             │                                          │
│                             ▼                                          │
│                    ┌─────────────────┐                                 │
│                    │  ExecuteReview  │  ◀── LLM / Human / Automated   │
│                    │  (I/O boundary) │                                 │
│                    └────────┬────────┘                                 │
│                             │                                          │
│                             ▼                                          │
│                    ┌─────────────────┐                                 │
│                    │  ParseFindings  │                                 │
│                    │  (extract)      │                                 │
│                    └────────┬────────┘                                 │
│                             │                                          │
│  Outputs:                   ▼                                          │
│  ┌────────────────┐  ┌────────────────┐  ┌────────────────────────┐   │
│  │   findings     │  │    verdict     │  │  suggestions           │   │
│  │  (issues)      │  │  (pass/fail)   │  │  (improvements)        │   │
│  └────────────────┘  └────────────────┘  └────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────┘
```

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
        format: DocFormat,  // Markdown, PlainText, Structured
    },
    /// Test results or test code
    TestOutput {
        stdout: String,
        stderr: String,
        exit_code: i32,
        test_count: Option<TestCounts>,
    },
    /// Generic text artifact
    Text {
        content: String,
        artifact_type: String,  // e.g., "commit_message", "pr_description"
    },
}

/// Review criteria
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rubric {
    /// What aspects to evaluate
    pub criteria: Vec<Criterion>,

    /// Severity thresholds
    pub fail_on: FailureCondition,

    /// Domain-specific instructions
    pub domain_prompt: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Criterion {
    pub name: String,           // e.g., "correctness", "security"
    pub description: String,    // What to look for
    pub weight: f32,            // Importance (0.0-1.0)
    pub examples: Vec<String>,  // Example issues
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FailureCondition {
    /// Fail if any finding of this severity or higher
    OnSeverity(Severity),
    /// Fail if score below threshold
    OnScore(f32),
    /// Never auto-fail, always advisory
    Advisory,
}

/// Review output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewResult {
    pub findings: Vec<Finding>,
    pub verdict: Verdict,
    pub summary: String,
    pub score: Option<f32>,        // 0.0-1.0 if scored
    pub reviewer: ReviewerType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub id: String,
    pub severity: Severity,
    pub category: String,          // Maps to Criterion.name
    pub location: Option<Location>,
    pub message: String,
    pub suggestion: Option<String>,
    pub confidence: f32,           // 0.0-1.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub file: Option<String>,
    pub line_start: Option<u32>,
    pub line_end: Option<u32>,
    pub column: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Verdict {
    Pass,
    Fail { blocking_findings: Vec<String> },  // Finding IDs
    NeedsWork { suggested_changes: u32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Severity {
    Info,       // Informational only
    Suggestion, // Nice to have
    Warning,    // Should address
    Error,      // Must fix
    Critical,   // Blocking
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReviewerType {
    Llm { model: String },
    Automated { tool: String },  // e.g., "clippy", "eslint"
    Human { reviewer: String },
}
```

### Predefined Rubrics

```rust
impl Rubric {
    /// Standard code review rubric
    pub fn code_review() -> Self {
        Self {
            criteria: vec![
                Criterion {
                    name: "correctness".into(),
                    description: "Logic errors, bugs, edge cases".into(),
                    weight: 1.0,
                    examples: vec![
                        "Off-by-one error".into(),
                        "Null pointer dereference".into(),
                    ],
                },
                Criterion {
                    name: "security".into(),
                    description: "Vulnerabilities, unsafe practices".into(),
                    weight: 1.0,
                    examples: vec![
                        "SQL injection".into(),
                        "Hardcoded secrets".into(),
                    ],
                },
                Criterion {
                    name: "performance".into(),
                    description: "Inefficiencies, scaling issues".into(),
                    weight: 0.7,
                    examples: vec![
                        "N+1 query".into(),
                        "Unnecessary allocation in loop".into(),
                    ],
                },
                Criterion {
                    name: "maintainability".into(),
                    description: "Readability, structure, naming".into(),
                    weight: 0.5,
                    examples: vec![
                        "Unclear variable name".into(),
                        "Function too long".into(),
                    ],
                },
            ],
            fail_on: FailureCondition::OnSeverity(Severity::Error),
            domain_prompt: None,
        }
    }

    /// Design document review rubric
    pub fn design_review() -> Self {
        Self {
            criteria: vec![
                Criterion {
                    name: "completeness".into(),
                    description: "All necessary sections present".into(),
                    weight: 1.0,
                    examples: vec![],
                },
                Criterion {
                    name: "clarity".into(),
                    description: "Clear, unambiguous language".into(),
                    weight: 0.8,
                    examples: vec![],
                },
                Criterion {
                    name: "feasibility".into(),
                    description: "Technically achievable".into(),
                    weight: 1.0,
                    examples: vec![],
                },
                Criterion {
                    name: "consistency".into(),
                    description: "No contradictions".into(),
                    weight: 0.9,
                    examples: vec![],
                },
            ],
            fail_on: FailureCondition::Advisory,
            domain_prompt: None,
        }
    }

    /// Test coverage/quality review
    pub fn test_review() -> Self {
        Self {
            criteria: vec![
                Criterion {
                    name: "coverage".into(),
                    description: "Key paths tested".into(),
                    weight: 1.0,
                    examples: vec![],
                },
                Criterion {
                    name: "edge_cases".into(),
                    description: "Boundary conditions handled".into(),
                    weight: 0.9,
                    examples: vec![],
                },
                Criterion {
                    name: "clarity".into(),
                    description: "Test intent is clear".into(),
                    weight: 0.6,
                    examples: vec![],
                },
            ],
            fail_on: FailureCondition::OnSeverity(Severity::Warning),
            domain_prompt: None,
        }
    }
}
```

### ReviewOps

```rust
#[derive(Debug, Clone)]
pub enum ReviewOps {
    /// Build review prompt from artifact + rubric
    ///
    /// Inputs:
    /// - `artifact`: Artifact
    /// - `rubric`: Rubric
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
    /// - `rubric`: Rubric (for category mapping)
    ///
    /// Outputs:
    /// - `result`: ReviewResult
    ParseReviewResponse,

    /// Determine verdict from findings
    ///
    /// Inputs:
    /// - `findings`: Vec<Finding>
    /// - `fail_condition`: FailureCondition
    ///
    /// Outputs:
    /// - `verdict`: Verdict
    /// - `blocking_count`: Int
    DetermineVerdict,

    /// Merge findings from multiple review sources
    ///
    /// Inputs:
    /// - `results`: Vec<ReviewResult>
    ///
    /// Outputs:
    /// - `merged`: ReviewResult
    MergeResults,
}
```

### ReviewPhase DAG

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                            ReviewPhase SubDag                               │
│                                                                             │
│  Entrypoints:                                                               │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐                       │
│  │ artifact │ │  rubric  │ │ context  │ │  config  │                       │
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
│              ┌─────────────────────────┐                                   │
│              │  PrepareChatRequest     │ ◀── PURE: build transport req     │
│              │  (LlmOps)               │                                   │
│              └────────────┬────────────┘                                   │
│                           │                                                 │
│                           ▼                                                 │
│              ┌─────────────────────────┐                                   │
│              │  Execute                │ ◀── I/O BOUNDARY                  │
│              │  (TransportOps)         │                                   │
│              └────────────┬────────────┘                                   │
│                           │                                                 │
│                           ▼                                                 │
│              ┌─────────────────────────┐                                   │
│              │  ParseChatResponse      │ ◀── PURE: extract content         │
│              │  (LlmOps)               │                                   │
│              └────────────┬────────────┘                                   │
│                           │                                                 │
│                           ▼                                                 │
│              ┌─────────────────────────┐                                   │
│              │  ParseReviewResponse    │ ◀── PURE: structure findings      │
│              │  (ReviewOps)            │                                   │
│              └────────────┬────────────┘                                   │
│                           │                                                 │
│                           ▼                                                 │
│              ┌─────────────────────────┐                                   │
│              │  DetermineVerdict       │ ◀── PURE: pass/fail decision      │
│              │  (ReviewOps)            │                                   │
│              └────────────┬────────────┘                                   │
│                           │                                                 │
│  Boundaries:              ▼                                                 │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐                                    │
│  │ findings │ │ verdict  │ │ summary  │                                    │
│  └──────────┘ └──────────┘ └──────────┘                                    │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Multi-Source Review (Parallel)

For comprehensive review, run multiple reviewers in parallel:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                      MultiReview (Parallel + Merge)                         │
│                                                                             │
│            ┌─────────────────────────────────────────────┐                 │
│            │              artifact + rubric              │                 │
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
│                           │    MergeResults     │                          │
│                           │    (ReviewOps)      │                          │
│                           └──────────┬──────────┘                          │
│                                      │                                     │
│                              ┌───────▼───────┐                             │
│                              │ unified result│                             │
│                              └───────────────┘                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Part 4: Composition Example

### Implement-and-Review Workflow

Compose ImplementationPhase + ReviewPhase into a complete workflow:

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
│  │                       LoopBuilder                                   │   │
│  │  (review each artifact)                                             │   │
│  │                                                                     │   │
│  │  ┌─────────────────────────────────────────────────────────────┐   │   │
│  │  │                    ReviewPhase                              │   │   │
│  │  │  rubric: Rubric::code_review()                              │   │   │
│  │  └─────────────────────────────────────────────────────────────┘   │   │
│  └─────────────────────────────┬───────────────────────────────────────┘   │
│                                │                                            │
│                                │ review_results: Vec<ReviewResult>          │
│                                ▼                                            │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                       BranchBuilder                                 │   │
│  │  condition: all_passed(review_results)                              │   │
│  │                                                                     │   │
│  │  ┌─────────────────┐              ┌─────────────────┐              │   │
│  │  │ True: Complete  │              │ False: Iterate  │              │   │
│  │  │ → output result │              │ → back to Impl  │              │   │
│  │  └─────────────────┘              └─────────────────┘              │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Builder API (Proposed)

```rust
/// Build a complete implement-and-review workflow
pub fn build_implement_and_review(
    task: ImplementationTask,
    rubric: Rubric,
    config: WorkflowConfig,
) -> Result<Dag<WorkflowOps>, BuilderError> {
    let mut builder = DagBuilder::new("implement-and-review");

    // Phase 1: Implementation
    let impl_phase = ImplementationPhaseBuilder::new()
        .with_task(task)
        .with_durability(config.storage_path.clone())
        .build()?;

    let impl_node = builder.add_subdag("implement", impl_phase)?;

    // Phase 2: Review each artifact
    let review_phase = ReviewPhaseBuilder::new()
        .with_rubric(rubric)
        .build()?;

    let review_loop = LoopBuilder::new("review-artifacts")
        .with_body(review_phase)
        .build()?;

    let review_node = builder.add_subdag_after("review", review_loop, &impl_node)?;
    builder.connect(impl_node.output("artifacts"), review_node.input("items"))?;

    // Phase 3: Check verdict and potentially loop back
    let check_node = builder.add_node_after(
        "check-verdict",
        WorkflowOps::CheckAllPassed,
        &review_node,
    )?;
    builder.connect(review_node.output("results"), check_node.input("results"))?;

    // Set boundaries
    builder.mark_entrypoint("task", impl_node.input("task"))?;
    builder.mark_boundary("result", check_node.output("final_result"))?;

    builder.build()
}
```

---

## Implementation Tasks

### Phase 0: Core Architecture (if generalizing)

- [ ] Consider adding Query/Command transport classification to `core/ir/src/transport/`
- [ ] Consider updating AGENT.md with Query/Command principle
- [ ] Consider updating SPEC.md §6 (Node Contract) with transport classification

### Phase 1: Core Types & Operations

- [ ] Define `ImplementationSession` and related types in `lib/codex/src/types.rs`
- [ ] Define `Artifact`, `Rubric`, `ReviewResult` in `lib/review/src/types.rs`
- [ ] Implement `ImplementationOps` with `Executable` trait (Query operations)
- [ ] Implement `ReviewOps` with `Executable` trait (Query operations)
- [ ] Define `ApplyOps` for Command operations (file writes, commits)

### Phase 2: Durability Infrastructure

- [ ] Implement step-level checkpoint storage
- [ ] Implement `CheckStepDone` / `RecordStep` operations
- [ ] Add content-addressed step ID computation
- [ ] Test crash recovery with simulated failures

### Phase 3: ReviewPhase SubDag

- [ ] Build `ReviewPhaseBuilder`
- [ ] Implement predefined rubrics (code, design, test)
- [ ] Add multi-source review with `MergeResults`
- [ ] Generate mock specs for testing

### Phase 4: ImplementationPhase SubDag

- [ ] Build `ImplementationPhaseBuilder`
- [ ] Integrate Codex CLI invocation
- [ ] Implement conversation state tracking
- [ ] Add artifact extraction from Codex responses

### Phase 5: Composition & CLI

- [ ] Build `ImplementAndReviewBuilder`
- [ ] Add `gunbc implement` command
- [ ] Add `gunbc review` command
- [ ] Add `--resume` support for interrupted sessions

---

## Open Questions

1. **Codex session files**: Does Codex CLI have its own session persistence? If so, how do we coordinate?

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
