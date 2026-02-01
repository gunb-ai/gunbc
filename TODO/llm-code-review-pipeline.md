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

| Phase | Type | Risk | Notes |
|-------|------|------|-------|
| **ReviewPhase** | Reasoning | Low | Produces findings, not mutations |
| **ImplementationPhase** | Reasoning | Medium | Produces patch artifacts, doesn't apply them |
| **TestPhase** | Reasoning | Medium | Observes behavior |
| **ApplyPhase** | Action | High | Mutates user artifacts |
| **CommitPhase** | Action | High | Mutates repository |

**Invariant**: Reasoning phases are hermetic — they produce *artifacts* (patches, findings) but don't apply them. Action phases are explicit and separate.

## Transport Classification: Two Orthogonal Axes

**I/O type** (read/write) — for purity, testing, DryRun mocking:
- **Read**: file reads, git diff, LLM calls, HTTP GET
- **Write**: file writes, git commit, cache updates

**Risk level** (fermi-style) — for safety/interest:
- **Low**: temp cache, tool-owned state
- **Medium**: LLM calls (cost, latency)
- **High**: repo mutation, external API calls
- **Extreme**: credential access, destructive ops

These are orthogonal: a cache write is low-risk but still a write (matters for purity). A credential read is high-risk but still a read. DryRun intercepts by I/O type; policy enforcement uses risk level.

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
- [ ] ReviewPhase subdag contains no high-risk transport ops
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

*See git history for detailed type definitions and appendix with Query/Journal/Command ownership model.*
