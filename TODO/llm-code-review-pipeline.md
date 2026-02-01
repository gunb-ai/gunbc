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

*See git history for detailed type definitions from earlier design iterations.*
