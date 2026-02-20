# Next Steps Planning — Post-Integration Assessment

**Branch**: `cursor/workflow-capabilities-next-steps-b824`
**Base**: `cursor/workflow-capabilities-integration-4668`
**Date**: 2026-02-20

---

## 1. Current State Summary

### What's Built and Working

The integration branch represents a substantial body of completed work across 7 sprints:

| Area | Status | Key Artifacts |
|------|--------|---------------|
| **DSL compiler** (daglang) | Complete | Parse → typecheck → lower → emit (Rust/Go/C/MIPS) |
| **Core IR** | Complete | Typed ports, transport model, resource system, patterns |
| **Execution engine** | Complete | DryRun interception, simulation, DynOp dispatch |
| **Workflow planner** (WF1-WF5) | Complete | Schema, admission, keying, coordination, dry-run |
| **Workflow cutover** (WF6-WF9) | Complete | CI + test-all on planner path, SLO instrumentation |
| **Tool workflow specs** (WF19-WF22) | Complete | bootstrap/makegen/pragma/deps/dag-viz/dag-snapshot specs, Makefile thinning, capability verification |
| **Service codegen** (SC1-SC7) | Complete | Generic protocol interpreters, DSL-driven services, multi-language emission |
| **Modeling hardening** (M7-M19) | Complete | Secret redaction, typed deps, resource declarations, dry-run poisoning, proof harness |
| **Backend correctness** (R2-R12) | Complete | IR enrichment, Go/C/MIPS migration, adversarial harness |
| **Resolver cleanup** (P6, P12) | Complete | DeferredCallableOp eliminated, typed dispatch |
| **Review library** | Partial | `lib/review/` with ops, dimensions, profiles, graph builders; `gunbc-review` binary exists |
| **LLM ops** | Complete | OpenAI + Anthropic, credential resolution, simple/chat request patterns |
| **Cloud credential chain** | Complete | GCP WIF/OIDC, Secret Manager, policy-based binding |
| **Design docs** | Complete | Workflow design pack, tool workflow design pack, service codegen, SDLC pipeline |

### What's Open

Three major tracks remain, plus small hardening items:

| Track | Tasks | Critical Path? | Estimated Size |
|-------|-------|----------------|----------------|
| **Lane F: Universal capabilities** | WF14, WF15 | Yes — blocks Lane G | M + M |
| **Lane G: Gist capability stack** | WF16, WF17, WF18 | No — blocked on F | M + S + M |
| **Lane J: Dev pipeline + SDLC** | W1-W14 | Yes — W1 unlocks everything | ~8 M tasks |
| **Daglang hardening** | DL1-DL4 | No — independent | 4 × S |

---

## 2. Gap Analysis

### What Exists vs. What's Needed for Each Open Task

#### W1: `gunbc review` CLI binary — PARTIALLY DONE

**Exists**: `gunbc-dag/src/bin/review.rs` is a fully functional binary entry point. It:
- Builds the diff review DAG with configurable provider/model/depth
- Resolves credentials from env/policy
- Supports `--dry-run`, `--provider`, `--depth`, `--pr` flags
- Outputs structured findings JSON
- Has working dry-run mode with mock specs

**Gap**: The binary exists and appears to satisfy W1's acceptance criteria. The question
is whether it has been **tested end-to-end with real credentials** (which is W2's scope).
The code handles both OpenAI and Anthropic providers, has proper credential resolution
through the cloud chain, and structures output correctly.

**Assessment**: W1 may already be **done or nearly done**. Needs a smoke test (W2) to
confirm. The `gunbc-review` binary entry is wired in `Cargo.toml` and should compile.

#### W2: Credential smoke test — NOT STARTED

**Gap**: No evidence of a successful real-mode run. Need to verify:
1. Credential resolves from `ANTHROPIC_API_KEY` or `OPENAI_API_KEY`
2. HTTP request goes out to the LLM provider
3. Response parses correctly
4. Findings are structured JSON

**Risk**: The cloud credential chain (GCP WIF → Secret Manager) is complex. In local dev
mode, the direct API key path should work, but the fallback behavior when no credential
policy is set needs verification.

#### W3: Multi-provider support — NOT STARTED

**Gap**: The binary already accepts `--provider openai` and `--provider anthropic`. Both
code paths exist in `LlmOps` and `llm::build_chat_request`. This may be trivially
verifiable once W2 passes.

#### W4: Abstract review DAG — PARTIALLY DONE

**Exists**:
- `lib/review/src/dimension.rs`: Full 4-dimension model (coherence, quality, requirements, aspirational)
- `DimensionOps`: `PrepareDimensionPrompt`, `ParseDimensionResponse`, `MergeDimensionOutputs`, `FormatPriorFindings`
- `FermiDepth` enum with prompt instructions
- `DimensionReviewOutput` with severity classification

**Gap**: No DAG builder that composes the 4-dimension model into a single DAG. The current
`build_diff_review_graph` uses a single-pass review, not the 4-dimension pipeline. Need:
- A graph builder that wires 3 parallel dimension SubDags + aspirational as final pass
- The dimensions opt-in via criteria presence (the ops support this)
- Integration with `FermiDepth` for per-dimension depth control

**Assessment**: ~60% done. The ops and types exist; the DAG composition is missing.

#### W5: Coding review profile — PARTIALLY DONE

**Exists**:
- `lib/review/src/profile.rs`: Complete `ReviewProfile` type, `coding_review_profile()`, `coding_review_profile_with_context()`, `coding_review_profile_with_requirements()`
- `ProjectContext` struct for AGENT.md + clippy.toml injection
- Per-dimension criteria with proper checks

**Gap**: No `gunbc review --pr <number>` integration that fetches PR diff + issue body.
Needs GitHub API integration for PR metadata.

#### W6: CI status as review context — NOT STARTED

**Gap**: Needs `gh run list` integration to query CI status and inject failure context.

#### W7-W8: Orchestration — NOT STARTED

**Gap**: `gunbc pipeline` command and GitHub issue integration.

#### W9: GitHub Issues transport — NOT STARTED

**Gap**: No `lib/ticket-ops/` crate. Needs REST service definitions for GitHub Issues API
and typed `TrackedIssue` adapter.

#### W10-W14: SDLC pipeline — NOT STARTED

**Gap**: No `lib/design-ops/`, no SDLC resolver, no approval gates.

#### WF14: Compilation capability — NOT STARTED

**Exists**: Design spec in `docs/design/workflow/tool-workflow-design-pack.md` Section 2.
Workflow planner infrastructure (WF1-WF5) is complete.

**Gap**: Need to implement binary freshness as a planner-managed keyed unit. Make targets
currently invoke `cargo run`; need to dispatch to pre-built binaries.

#### WF15: Codegen capability — NOT STARTED

**Exists**: Design spec in Section 3 of tool workflow design pack.

**Gap**: Codegen as planner-managed keyed unit. `ensure-codegen` removal from Make
prerequisites for planner-managed targets.

#### WF16-WF18: Gist capability stack — NOT STARTED (blocked on WF14/WF15)

**Exists**: Design spec in Section 4. Gist tool infrastructure (`lib/tools/gist/`) exists.

**Gap**: Base gist workflow + mode-specific augmentations as planner-managed workflows.

#### DL1-DL4: Daglang CLI hardening — NOT STARTED

Small independent fixes. Low risk, low effort.

---

## 3. Critical Path Analysis

Two independent critical paths exist:

### Path A: Dev Pipeline (highest user-facing value)

```
W1 (verify) → W2 (smoke) → W3 (multi-provider) → W4 (4-dim DAG) → W5 (coding profile)
    → W6 (CI context) → W7 (pipeline cmd) → W8 (issue integration)
         ↘ W9 (issues transport)
              → W10 (design ops) → W11 (SDLC resolver) → W12 (SDLC CLI) → W13 (approval)
```

**W1 is the immediate gate**. It may already be done — needs verification.

### Path B: Workflow Minimization (infrastructure completeness)

```
WF14 (compilation) → WF15 (codegen) → WF16 (base gist) → WF17 (diff mode)
                                                         → WF18 (recent mode)
```

**WF14 is the immediate gate** for this path.

### Independent Work (no dependencies)

- DL1-DL4: Daglang CLI hardening

---

## 4. Recommended Execution Plan

### Phase 1: Verify and Ship W1-W3 (1-2 days)

**Why first**: W1 appears to be substantially done. Verifying and shipping it unlocks the
entire dev pipeline track with minimal effort. This is the highest-leverage action.

1. **Verify W1** — compile `gunbc-review`, dry-run test, confirm structured output
2. **W2** — real-mode smoke test with an API key
3. **W3** — verify both providers work (may be trivial after W2)

### Phase 2: Build 4-Dimension Review (3-5 days)

**Why**: This is the core architectural piece for the review pipeline and unblocks
the SDLC pipeline's review capability.

4. **W4** — Build the 4-dimension DAG composer. The ops exist; need the graph builder
   that wires `coherence|quality|requirements` in parallel → `FormatPriorFindings`
   → `aspirational` → `MergeDimensionOutputs`. This is a graph.rs addition in
   `lib/review/`.

5. **W5** — Wire `ReviewProfile` into the graph builder. Add `--pr` support by
   fetching PR diff via `gh pr diff` (shell transport, same as git diff pattern).

6. **W6** — Add CI status injection via `gh run list` (small extension to W5).

### Phase 3: Orchestration + GitHub Transport (5-7 days, parallelizable)

Two sub-tracks that can run in parallel:

**Track 3a: Pipeline Command**
7. **W7** — `gunbc pipeline` command
8. **W8** — Issue integration for requirements context

**Track 3b: GitHub Issues Transport + SDLC**
9. **W9** — GitHub Issues transport (REST service + typed adapter)
10. **W10** — DesignOps
11. **W11** — SDLC pipeline resolver
12. **W12** — `gunbc sdlc` CLI

### Phase 4: Workflow Minimization (parallelizable with Phase 2-3)

This can run independently of the W* track:

13. **WF14** — Compilation capability
14. **WF15** — Codegen capability
15. **WF16** → **WF17** + **WF18** — Gist capability stack

### Phase 5: Polish (independent, any time)

16. **DL1-DL4** — Daglang CLI hardening
17. **W13** — Approval gates (depends on W12)
18. **W14** — Pipeline metrics

---

## 5. Suggested Lane Assignments

| Lane | Owner Focus | Tasks | Dependencies |
|------|-------------|-------|--------------|
| **Lane 1: Review/SDLC** | W1 verification → W2 → W3 → W4 → W5 → W6 → W7 → W8 | Unlocks user-facing review pipeline | None |
| **Lane 2: GitHub transport** | W9 → W10 → W11 → W12 | Builds SDLC infrastructure | W1 (for W9), W3+W4 (for W10) |
| **Lane 3: Workflow minimization** | WF14 → WF15 → WF16 → WF17/WF18 | Completes planner coverage | None |
| **Lane 4: Daglang hardening** | DL1 → DL2 → DL3 → DL4 | Quick independent fixes | None |

Lanes 1 and 3 are the highest priority. Lane 4 can be done by anyone with spare cycles.

---

## 6. Open Design Questions

### 6a. 4-Dimension DAG Shape (W4)

The dimension ops exist, but the DAG builder needs to decide:
- How to handle the "opt-in via criteria presence" pattern in graph construction
- Whether each dimension is a separate SubDag or inline nodes
- How aspirational's dependency on merged findings is wired

**Proposed approach**: Each dimension is an inline chain (not SubDag) — the graph builder
conditionally adds dimension chains based on which criteria are present in the profile.
Aspirational chain takes `FormatPriorFindings` output as input.

### 6b. GitHub Issues Transport (W9)

The service codegen system (SC1-SC7) now supports DSL-driven REST services. Should
`github/issues.dag` be a new DSL service definition, or hand-wired like the current
review graph?

**Proposed approach**: Define `dsl/services/github/issues.dag` as a standard REST service,
following the same pattern as `dsl/services/github/gist.dag`. This validates the service
codegen system with a new domain and keeps the architecture consistent.

### 6c. WF14 Compilation Keying

The design spec describes binary freshness keyed on `(source_hash, cargo_metadata,
profile, target, features, RUSTFLAGS)`. The implementation needs to decide:
- How to compute source hash efficiently (tree hash vs file-by-file)
- Where the binary cache lives (target/release is already there; add ledger metadata)
- How to detect staleness without invoking Cargo

**Proposed approach**: Use `git rev-parse HEAD:Cargo.lock` + `git rev-parse HEAD:Cargo.toml`
as the primary freshness signal, with `cargo metadata --format-version 1` digest as
secondary. The planner checks these against the ledger; if unchanged, binary is fresh.

---

## 7. Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| W1 doesn't work end-to-end with real credentials | Medium | Blocks entire W* track | Test early; API keys are the simplest credential path |
| 4-dimension DAG (W4) is complex to wire | Low | Delays review pipeline | Ops already exist; graph wiring follows established patterns |
| GitHub Issues API has undocumented edge cases | Low | Delays SDLC pipeline | Use `gh` CLI as shell transport first; native REST later |
| WF14 binary keying is fragile | Medium | False cache hits → stale binaries | Conservative keying (include RUSTFLAGS, features); fallback to rebuild |
| Cross-track merge conflicts | Medium | Delays integration | Separate file ownership by lane; regular integration |

---

## 8. Definition of Done (per phase)

### Phase 1 (W1-W3)
- `gunbc-review -n` produces structured findings JSON (dry-run)
- `gunbc-review -r .` with `ANTHROPIC_API_KEY` or `OPENAI_API_KEY` produces real findings
- Both `--provider openai` and `--provider anthropic` work

### Phase 2 (W4-W6)
- 4-dimension DAG runs with all dimensions active (M depth)
- Single dimension (coherence only, XS depth) works as quick sanity check
- `--pr` flag enriches requirements dimension with PR context
- CI failures inject context into requirements dimension

### Phase 3 (W7-W12)
- `gunbc pipeline` produces actionable summary
- `gunbc sdlc --issue 42 --dry-run` resolves all ops
- End-to-end test with real GitHub issue + LLM

### Phase 4 (WF14-WF18)
- `make gist` no longer invokes `cargo run`
- Warm-state `make gist` resolves from ledger (seconds, not minutes)
- All 3 gist modes on planner path with shared base capabilities
