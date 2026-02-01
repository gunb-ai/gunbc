# LLM Code Review Pipeline

**Status**: Design
**Date**: 2026-02-01

## Goal

Build a robust LLM-powered code review pipeline that:
1. Integrates with **Codex CLI** as the primary LLM interface
2. Supports **local execution** with durable state (resume after interruption)
3. Models state as **typed DAG values** (no external state machine)
4. Composes cleanly with existing DAG patterns (Upsert, Retry, Loop)

Future extension: GitHub Actions integration for automated PR reviews.

---

## Design Principles

### 1. State-as-Values (Not Checkpoints)

Following the gunbc philosophy, durability is achieved through:
- **Idempotent operations** via Upsert pattern (check → skip if done)
- **State flowing through edges** as typed values
- **Loop-carried state** for multi-turn conversations
- **Explicit checkpoint nodes** that persist/restore state to disk

This avoids a separate checkpoint/restore mechanism — the DAG structure *is* the state.

### 2. Codex as a Transport Boundary

Codex CLI invocations are **I/O boundaries** following the three-phase pattern:
```
PrepareCodexRequest (pure) → TransportOps::Execute (I/O) → ParseCodexResponse (pure)
```

This ensures:
- Dry-run testing with mocked responses
- Clear separation of request construction from execution
- Deterministic replay of recorded sessions

### 3. Fractal Composition

The pipeline composes from smaller patterns:
```
CodeReviewPipeline
├── CodexUpsert (check installed → install → verify)
├── ContextGather (git diff, file reads)
├── ReviewLoop (multi-turn if needed)
│   ├── PrepareReviewRequest
│   ├── ExecuteCodex
│   ├── ParseResponse
│   └── CheckpointState (persist progress)
└── OutputFormat (structured findings)
```

---

## Codex CLI Integration

### CodexToolDef

```rust
/// Codex CLI tool definition
pub static CODEX: CliToolDef = CliToolDef {
    id: "codex",
    check_cmd: &["codex", "--version"],
    install_cmd: Some(&["npm", "install", "-g", "@openai/codex"]),
    run_cmd: &["codex"],
    description: "OpenAI Codex CLI for code generation and review",
    access_mode: AccessMode::Exclusive,  // One Codex session at a time
};
```

### CodexOps

```rust
/// Operations for Codex CLI integration
#[derive(Debug, Clone)]
pub enum CodexOps {
    /// Prepare a Codex CLI invocation (PURE)
    ///
    /// Inputs:
    /// - `prompt`: String — the review/generation prompt
    /// - `context`: String — code context (diff, files)
    /// - `config`: CodexConfig — model, temperature, etc.
    /// - `session_state`: Option<CodexSessionState> — for multi-turn
    ///
    /// Outputs:
    /// - `request`: ShellRequest — CLI invocation
    /// - `session_id`: String — for state tracking
    PrepareRequest,

    /// Parse Codex CLI output (PURE)
    ///
    /// Inputs:
    /// - `stdout`: String — CLI output
    /// - `stderr`: String — CLI errors
    /// - `exit_code`: Int
    /// - `session_id`: String
    ///
    /// Outputs:
    /// - `response`: CodexResponse — structured response
    /// - `session_state`: CodexSessionState — updated state
    /// - `success`: Bool
    ParseResponse,

    /// Extract structured review findings (PURE)
    ///
    /// Inputs:
    /// - `response`: CodexResponse
    ///
    /// Outputs:
    /// - `findings`: Vec<ReviewFinding>
    /// - `summary`: String
    /// - `severity`: ReviewSeverity
    ExtractFindings,
}
```

### CodexConfig & State Types

```rust
/// Configuration for Codex invocations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexConfig {
    pub model: String,           // e.g., "gpt-4o", "o3"
    pub temperature: f64,        // 0.0-1.0
    pub max_tokens: Option<u64>,
    pub system_prompt: Option<String>,
    pub output_format: OutputFormat,  // Text | Json | Markdown
}

impl Default for CodexConfig {
    fn default() -> Self {
        Self {
            model: "gpt-4o".into(),
            temperature: 0.3,  // Lower for code review
            max_tokens: Some(4096),
            system_prompt: None,
            output_format: OutputFormat::Markdown,
        }
    }
}

/// Session state for multi-turn conversations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexSessionState {
    pub session_id: String,
    pub turn_count: u32,
    pub messages: Vec<CodexMessage>,
    pub context_files: Vec<String>,
    pub partial_findings: Vec<ReviewFinding>,
}

/// A single finding from code review
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewFinding {
    pub file: String,
    pub line: Option<u32>,
    pub severity: Severity,
    pub category: Category,
    pub message: String,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Severity { Info, Warning, Error, Critical }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Category {
    Bug,
    Security,
    Performance,
    Style,
    Documentation,
    Testing,
}
```

---

## Durability Model

### Philosophy: Idempotent Resume via Upsert

Instead of checkpointing execution state, we use the **Upsert pattern** at each durable step:

```
CheckStepComplete → [Skip if done] → ExecuteStep → MarkComplete
```

The "check" reads from persistent storage; "mark complete" writes to it.

### CheckpointNode Pattern

```rust
/// Checkpoint operations for durable state
#[derive(Debug, Clone)]
pub enum CheckpointOps {
    /// Check if a checkpoint exists (PURE read)
    ///
    /// Inputs:
    /// - `checkpoint_id`: String
    /// - `storage_path`: String (local file path)
    ///
    /// Outputs:
    /// - `exists`: Bool
    /// - `state`: Option<Value> — the stored state if exists
    CheckExists,

    /// Save checkpoint to storage (I/O)
    ///
    /// Inputs:
    /// - `checkpoint_id`: String
    /// - `storage_path`: String
    /// - `state`: Value — any serializable state
    ///
    /// Outputs:
    /// - `success`: Bool
    /// - `path`: String — actual file path written
    Save,

    /// Load checkpoint from storage (I/O)
    ///
    /// Inputs:
    /// - `checkpoint_id`: String
    /// - `storage_path`: String
    ///
    /// Outputs:
    /// - `state`: Value
    /// - `timestamp`: Int — when saved
    Load,

    /// Clear checkpoint (I/O)
    ///
    /// Inputs:
    /// - `checkpoint_id`: String
    /// - `storage_path`: String
    ///
    /// Outputs:
    /// - `success`: Bool
    Clear,
}
```

### Storage Layout

```
~/.gunbc/
├── checkpoints/
│   └── code-review/
│       ├── session-{id}.json        # Session state
│       ├── findings-{id}.json       # Partial findings
│       └── context-{id}.json        # Cached context
├── cache/
│   └── codex/
│       ├── responses/               # Response cache (by hash)
│       └── context/                 # Context cache
└── config/
    └── codex.toml                   # User config
```

### Checkpoint ID Strategy

Checkpoints are keyed by **content hash** of the operation inputs:
```rust
fn checkpoint_id(operation: &str, inputs: &[Value]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(operation.as_bytes());
    for input in inputs {
        hasher.update(&serde_json::to_vec(input).unwrap());
    }
    format!("{}-{}", operation, hasher.finalize().to_hex()[..12])
}
```

This means:
- Same inputs → same checkpoint → skip re-execution
- Changed inputs → new checkpoint → fresh execution
- No manual cache invalidation needed

---

## DAG Structure

### Top-Level Pipeline

```
┌─────────────────────────────────────────────────────────────────────┐
│                     CodeReviewPipeline                              │
│                                                                     │
│  ┌──────────────┐    ┌───────────────┐    ┌────────────────────┐   │
│  │ CodexUpsert  │───▶│ GatherContext │───▶│    ReviewLoop      │   │
│  │ (install)    │    │ (git diff)    │    │ (multi-turn)       │   │
│  └──────────────┘    └───────────────┘    └─────────┬──────────┘   │
│                                                     │              │
│                                           ┌─────────▼──────────┐   │
│                                           │  FormatOutput      │   │
│                                           │  (findings)        │   │
│                                           └────────────────────┘   │
└─────────────────────────────────────────────────────────────────────┘
```

### Entrypoints (Unconnected Inputs)

| Port | Type | Description |
|------|------|-------------|
| `target` | `String` | What to review: file path, git ref, or "staged" |
| `config` | `CodexConfig` | Review configuration |
| `storage_path` | `String` | Where to persist state (~/.gunbc/checkpoints) |

### Boundaries (Unconnected Outputs)

| Port | Type | Description |
|------|------|-------------|
| `findings` | `Vec<ReviewFinding>` | Structured review results |
| `summary` | `String` | Human-readable summary |
| `session_id` | `String` | For resuming/referencing |

### SubDag: GatherContext

```
┌─────────────────────────────────────────────────────────────────┐
│                      GatherContext                              │
│                                                                 │
│  ┌─────────────┐                                               │
│  │ ParseTarget │──┬──▶ [GitDiff]  ──┐                          │
│  │ (route)     │  │                 │    ┌────────────────┐    │
│  └─────────────┘  ├──▶ [ReadFile] ──┼───▶│ MergeContext   │    │
│                   │                 │    │ (combine)      │    │
│                   └──▶ [GitStaged]──┘    └────────────────┘    │
│                                                                 │
│  Branch on target type:                                         │
│  - "staged" → git diff --cached                                │
│  - "HEAD~N" → git diff HEAD~N                                  │
│  - file path → read file directly                              │
└─────────────────────────────────────────────────────────────────┘
```

### SubDag: ReviewLoop (with Durability)

```
┌─────────────────────────────────────────────────────────────────────┐
│                         ReviewLoop                                  │
│                                                                     │
│  ┌────────────────────────────────────────────────────────────┐    │
│  │                    WhileBuilder                            │    │
│  │  condition: !response.is_complete                          │    │
│  │  max_iterations: 5                                         │    │
│  │                                                            │    │
│  │  ┌────────────────────────────────────────────────────┐   │    │
│  │  │               Loop Body (per turn)                 │   │    │
│  │  │                                                    │   │    │
│  │  │  ┌─────────────────┐                              │   │    │
│  │  │  │ CheckpointUpsert│ ◀── Check if turn done       │   │    │
│  │  │  │ (idempotent)    │                              │   │    │
│  │  │  └───────┬─────────┘                              │   │    │
│  │  │          │ [skip if checkpoint exists]            │   │    │
│  │  │          ▼                                        │   │    │
│  │  │  ┌─────────────────┐                              │   │    │
│  │  │  │ PrepareRequest  │ ◀── Build Codex CLI args     │   │    │
│  │  │  │ (pure)          │                              │   │    │
│  │  │  └───────┬─────────┘                              │   │    │
│  │  │          ▼                                        │   │    │
│  │  │  ┌─────────────────┐                              │   │    │
│  │  │  │ ExecuteCodex    │ ◀── I/O boundary             │   │    │
│  │  │  │ (transport)     │                              │   │    │
│  │  │  └───────┬─────────┘                              │   │    │
│  │  │          ▼                                        │   │    │
│  │  │  ┌─────────────────┐                              │   │    │
│  │  │  │ ParseResponse   │ ◀── Extract structured data  │   │    │
│  │  │  │ (pure)          │                              │   │    │
│  │  │  └───────┬─────────┘                              │   │    │
│  │  │          ▼                                        │   │    │
│  │  │  ┌─────────────────┐                              │   │    │
│  │  │  │ SaveCheckpoint  │ ◀── Persist for resume       │   │    │
│  │  │  │ (I/O)           │                              │   │    │
│  │  │  └─────────────────┘                              │   │    │
│  │  │                                                    │   │    │
│  │  └────────────────────────────────────────────────────┘   │    │
│  │                                                            │    │
│  │  loop_state: CodexSessionState (carried across turns)      │    │
│  └────────────────────────────────────────────────────────────┘    │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### SubDag: CheckpointUpsert

The idempotent checkpoint pattern:

```rust
/// Build an upsert that checks/restores checkpoint before execution
pub fn build_checkpoint_upsert<T>(
    name: &str,
    checkpoint_id_expr: &str,  // How to compute checkpoint ID
    body_dag: Dag<T>,          // The actual work
) -> Result<Node<T>, BuilderError>
where
    T: From<CheckpointOps> + From<TransportOps>,
{
    UpsertBuilder::new(name)
        // Check: Does checkpoint exist?
        .check_op(CheckpointOps::CheckExists)
        .check_output("exists", "Bool")
        .check_output("state", "Option<Value>")

        // Create: Execute body (only if no checkpoint)
        .create_dag(body_dag)

        // Resolve: Return state (from checkpoint or fresh execution)
        .resolve_op(|check_result, create_result| {
            if check_result.exists {
                check_result.state  // Use cached
            } else {
                create_result       // Use fresh
            }
        })
        .build()
}
```

---

## CLI Interface

### Commands

```bash
# Review staged changes
gunbc review staged

# Review specific files
gunbc review src/lib.rs src/main.rs

# Review git range
gunbc review HEAD~3..HEAD

# Resume interrupted session
gunbc review --resume session-abc123

# With custom config
gunbc review staged --model gpt-4o --temperature 0.2

# Output formats
gunbc review staged --format json > findings.json
gunbc review staged --format markdown > review.md
```

### Configuration File

```toml
# ~/.gunbc/config/codex.toml

[codex]
model = "gpt-4o"
temperature = 0.3
max_tokens = 4096

[codex.review]
system_prompt = """
You are an expert code reviewer. Focus on:
- Correctness and potential bugs
- Security vulnerabilities
- Performance issues
- Code clarity
Provide specific, actionable feedback with line numbers.
"""

[storage]
checkpoint_dir = "~/.gunbc/checkpoints"
cache_responses = true
cache_ttl_hours = 24
```

---

## Mock Specification

```rust
/// Mock spec for testing code review pipeline
pub fn code_review_pipeline_mock_spec() -> MockSpec {
    MockSpec::new("code-review-pipeline")
        // Codex CLI execution mock
        .boundary(
            "execute_codex",
            "stdout",
            Value::Str(mock_codex_review_output()),
        )
        .boundary("execute_codex", "stderr", Value::Str("".into()))
        .boundary("execute_codex", "exit_code", Value::Int(0))

        // Git diff mock
        .boundary(
            "git_diff",
            "stdout",
            Value::Str(mock_git_diff()),
        )

        // Checkpoint mocks
        .boundary("check_checkpoint", "exists", Value::Bool(false))
        .boundary("save_checkpoint", "success", Value::Bool(true))

        // Input expectations
        .expects_input("target", InputConstraint::NonEmpty)
        .expects_input("config", InputConstraint::NonEmpty)
}

fn mock_codex_review_output() -> String {
    r#"{
        "findings": [
            {
                "file": "src/lib.rs",
                "line": 42,
                "severity": "warning",
                "category": "bug",
                "message": "Potential panic: unwrap() on Result that may be Err",
                "suggestion": "Use `?` operator or handle the error explicitly"
            }
        ],
        "summary": "Found 1 issue. Overall code quality is good.",
        "complete": true
    }"#.into()
}
```

---

## Implementation Tasks

### Phase 1: Core Infrastructure

- [ ] Define `CodexOps` enum in `lib/codex/src/ops.rs`
- [ ] Define state types (`CodexConfig`, `CodexSessionState`, `ReviewFinding`)
- [ ] Implement `Executable` for `CodexOps`
- [ ] Add `CODEX` to `CliToolDef` registry
- [ ] Create `CheckpointOps` for durable state

### Phase 2: DAG Patterns

- [ ] Build `GatherContext` SubDag (git diff, file read routing)
- [ ] Build `ReviewLoop` with `WhileBuilder` and loop-carried state
- [ ] Build `CheckpointUpsert` helper for idempotent steps
- [ ] Wire up `CodeReviewPipeline` top-level DAG
- [ ] Add mock specs for all boundaries

### Phase 3: CLI Integration

- [ ] Add `review` subcommand to gunbc CLI
- [ ] Implement config file loading (`~/.gunbc/config/codex.toml`)
- [ ] Add `--resume` flag for session recovery
- [ ] Add output format options (text, json, markdown)

### Phase 4: Testing & Polish

- [ ] Generate tests via testgen for pipeline DAG
- [ ] Add integration tests with mocked Codex responses
- [ ] Add real Codex integration test (requires API key)
- [ ] Documentation and examples

### Future: GitHub Actions Integration

- [ ] `GatherContext` variant that fetches PR diff via GitHub API
- [ ] Output formatting as PR comment
- [ ] GitHub Actions workflow template
- [ ] Rate limiting and error handling for API

---

## Open Questions

1. **Multi-file chunking**: Large diffs may exceed context window. Strategy?
   - Option A: Chunk by file, review each, merge findings
   - Option B: Summarize first, then deep-dive on flagged areas
   - Option C: Use file-level priority (changed lines count)

2. **Response parsing robustness**: Codex output may vary. How strict?
   - Option A: Require JSON output format, fail on parse error
   - Option B: LLM-assisted parsing (use another call to structure output)
   - Option C: Regex/heuristic extraction with fallback

3. **Session scope**: What defines a "session" for checkpointing?
   - Option A: Per-invocation (new session each `gunbc review`)
   - Option B: Per-target (same file = same session, resume by default)
   - Option C: Explicit (`--session-id` flag)

4. **Concurrent reviews**: Multiple reviews in parallel?
   - Current: `AccessMode::Exclusive` prevents concurrent Codex
   - Future: Could allow with separate session IDs

---

## Related Patterns

- **Upsert**: `core/ir/src/patterns/upsert.rs` — idempotent create/update
- **While/Retry**: `core/ir/src/patterns/repeat.rs` — iteration with state
- **CLI Tools**: `core/ir/src/transport/cli.rs` — CliToolDef, CliToolOp
- **LLM Ops**: `lib/llm-ops/src/` — existing chat completion patterns

---

## Notes

- The durability model is intentionally **not** a full checkpoint/restore system.
  Instead, it leverages DAG structure: each step is idempotent, and resumption
  is "re-run from start, skip completed steps."

- Codex CLI is treated as a transport boundary, not a special case. This means
  all existing testing infrastructure (DryRun, mocks, testgen) works automatically.

- Loop-carried state in `WhileBuilder` is the key to multi-turn conversations.
  The `CodexSessionState` accumulates messages and findings across turns.
