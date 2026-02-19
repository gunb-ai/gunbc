# Task Sheet — Dependency-Ordered, Parallelizable

**Last updated**: 2026-02-19
**Verification**: `cargo test --workspace` + `cargo clippy --all-targets -- -D warnings`
**Archive**: Completed items in `TODONE/tasks-completed.md`. Backlog in `backlog.md`.

**Sizing**: S (<1 day), M (1-3 days), L (3-5 days), XL (5+ days)

### Conventions

- **Definition of Done**: each task is done when code compiles, tests pass, and clippy is clean.
- **Code TODO/HACK comments** must reference a task ID (e.g., `TODO(P1): ...`) so orphans
  are discoverable via grep.
- **Active Docs invariant**: every path in the task sheet must exist; no doc under
  `TODO/TODONE/` may appear in active sections.

---

## Sprint 2: Review Findings + Polish

### Review Findings

Bug surfaced by automated review. This is real but latent (not causing test failures yet).

| ID | Task | Deps | Size |
|----|------|------|------|
| **R2** | **[STARTED 2026-02-19]** **Wildcard resource semantics deferred**: remove generated/injected `res:file:*` usage for now (coarsen to `res:file`), treat coarse `file` as conflicting with any specific `file:<path>` lock in admission control, and normalize wildcard IDs to coarse `file` in resource accounting. Track full glob semantics as design work in `backlog.md` before enabling pattern-aware admission control. | — | M |

### Code TODOs & DSL Compiler Polish

#### Design Decision: Backend Semantics Must Be Encoded in IR (not naming conventions)

Recent emitter bugs (Go/C redeclaration, MIPS early-return epilogue bypass, MIPS temp
clobber) show a shared issue: backend lowering currently relies on string/name conventions
in places where the IR should carry the semantic constraints.

For this track, prefer **model enrichment first**:
- Add/extend IR nodes so correctness is enforced by construction.
- Lowerers and renderers should consume explicit modeled semantics, not infer behavior
  from hardcoded names.
- Validation passes are still useful, but treated as guardrails, not the primary design.

| ID | Task | Deps | Size | Source |
|----|------|------|------|--------|
| **R3** | **Backend modeling enrichment RFC + IR schema update**: define and land minimal IR/API extensions needed for correctness-by-construction across these bugs. Scope: (a) Go declaration intent (short-declare vs assign), (b) C lexical scope block statement for temporary-lifetime isolation, (c) MIPS explicit return-to-epilogue contract (single-exit semantics), (d) fallible temp allocation API for register pressure. Include migration notes for lowerers/renderers/tests. **Acceptance**: changed IR types compile across emit crate; no backend fix requires hardcoded temp names to simulate scope; return/epilogue flow is representable without raw `jr $ra` in lowering logic. | — | M | review synthesis |
| **R4** | **Go + C lowerer migration to modeled semantics**: migrate transport-call statement lowering to the new IR contracts. Go must avoid repeated `:=` redeclare failures by construction; C must avoid same-scope `__rc` redefinition by construction (scoped block or equivalent modeled mechanism). **Acceptance**: add regressions with 2+ transport expression statements in one function; generated Go/C compile cleanly; tests assert structural IR behavior (not string fragments only). | R3 | M | review synthesis |
| **R5** | **MIPS control-flow + allocator fail-closed migration**: implement single-exit return lowering (all returns route through epilogue path), and make temp allocation fail with explicit `LowerError` on exhaustion instead of wrapping/clobbering. **Acceptance**: framed functions do not contain body-level direct `JumpReg(Register::Ra)` for lowered returns; deep-expression/register-pressure test fails closed with explicit lowering error, not silent corruption. | R3 | M | review synthesis |
| **R6** | **Holistic backend correctness harness**: add cross-backend adversarial fixtures + smoke compilation checks for generated artifacts (Go/C compile, MIPS assembly structure checks), plus invariant checks that encode the modeled contracts. **Acceptance**: old buggy patterns are caught by tests; new modeled path passes; CI command includes this harness. | R4, R5 | M | review synthesis |

#### Design Decision: DeferredCallableOp Elimination Strategy (blocks P6)

P6 replaces `DeferredCallableOp` (identity passthrough) with per-tool domain ops.
The design question is: **what concrete `Executable` impl replaces each deferred
callable, and how should dry-run mode work without a passthrough fallback?**

Current deferred callables (from `resolve.rs` + `rust_exec_runtime.rs:306`):

| Module | Callables | What they actually do |
|--------|-----------|----------------------|
| `tools.build` | `build_all` | Orchestrates `cargo build` — prepare shell request, parse result |
| `tools.docgen` | `docgen`, `render_ab_workflows_doc` | Generate markdown docs from registry data |
| `tools.testgen` | `generate_tests`, `testgen` | Generate test harnesses from DagSpecDef |
| `tools.clippy` | `clippy_lint` | Prepare `cargo clippy` invocation, parse diagnostics |
| `tools.deps` | `render_deps_toml`, `select_platform_deps`, `deps_install`, `deps_generate` | Dependency resolution and rendering |
| `pipelines.ci` | all | CI stage orchestration (entrypoint + stage dispatch) |
| `shared.dag_util` | all | DAG construction helpers (pure combinators) |
| `shared.gist_modes` | all | Gist mode selection logic |
| `std.patterns` | all | Standard patterns (content_upsert, while, etc.) |

**Approach**: implement per-module `*Op` enums following the PragmaOp pattern.
For dry-run: resolved ops should check `ExecutionMode::DryRun` internally and
short-circuit with typed empty outputs (not generic identity passthrough). The
catch-all `_ => Ok(deferred_callable(...))` on line 872 becomes
`_ => Err(unknown_callable(...))` once all modules have resolution arms.

| ID | Task | Deps | Size | Source |
|----|------|------|------|--------|
| **P6** | `DeferredCallableOp` → per-module domain ops: implement `*Op` enums for each deferred module (see table above), replace catch-all passthrough with `Err(unknown_callable(...))`. Dry-run via `ExecutionMode` check inside each op, not via identity passthrough. ~15 modules, ~25 callables total. (`resolve.rs`, `rust_exec_runtime.rs:306`) | — | L | PR review |
| **P12** | Move `resolve_infrastructure()` string-prefix matching up to lowering: lowerer should emit typed `LoweredOp` variants (e.g., `LoweredOp::Primitive(PrepareFileWrite)`) instead of encoding op identity in callable name strings. Resolver becomes exhaustive enum match, not prefix scan. 9 golden tests already cover current behavior. (`resolve.rs:758-842`, `daglang-lower`) | — | M | PR review |

---

## Sprint 3: Dev Pipeline — Real Workflow

**Goal**: Build a working AI-assisted development pipeline that runs locally:

```
Daily roadmap (GitHub issues / TODO)
    → Implementation (Cursor / Codex)
    → Review (LLM review pipeline)
    → CI check (is it passing?)
    → Submit (user approves & merges)
```

The infrastructure (DSL, executor, credentials, transport) is built. This sprint
wires it into a usable end-to-end flow.

### Design Decision: Runtime Environment

The pipeline runs **locally** as a CLI tool. Credential resolution supports both:
- **Local dev**: `OPENAI_API_KEY` / `ANTHROPIC_API_KEY` from env vars
- **CI/cloud**: GitHub Actions OIDC → GCP WIF → Secret Manager

No server or stateful service needed initially — the DAG executor already handles
orchestration. State lives in git (branches, PRs, issues).

### Phase 1: Credentials Work (prerequisite for everything)

LLM credential resolution exists (`lib/llm-ops/`, `lib/cloud-ops/credential_policy.rs`)
with `ureq` HTTP client, OpenAI + Anthropic support, and GCP Secret Manager integration.
Live integration tests exist but are env-gated. Need to verify the full chain works
end-to-end outside of test harness.

| ID | Task | Deps | Size |
|----|------|------|------|
| **W1** | **`gunbc review` CLI binary**: Add binary entry point that builds the review DAG, resolves credentials from env/policy, and executes in Real mode. Input: git diff (stdin or `--base-ref`). Output: structured findings JSON to stdout. This is the first real end-to-end execution of the full stack. | — | M |
| **W2** | **Credential smoke test**: Run `gunbc review` locally with `ANTHROPIC_API_KEY` set, feeding a small diff. Verify: credential resolves, HTTP request goes out, response parses, findings are structured. Fix any issues found. | W1 | S |
| **W3** | **Multi-provider support**: Verify `gunbc review --provider openai` and `--provider anthropic` both work. Test credential policy override via `GUNBC_CREDENTIAL_POLICY_JSON` for switching between providers without changing env vars. | W2 | S |

### Design Decision: Abstract Review Model

The review pipeline is **domain-agnostic**. Four abstract dimensions, each with
its own **criteria input port** and **depth port** (Fermi size: XS/S/M/L/XL).
Criteria are provided by the caller — omitting a dimension's criteria opts it out.

#### Dimensions and Ports

Each dimension is a SubDag node with these input ports:

| Port | Type | Required | Description |
|------|------|----------|-------------|
| `artifact` | String | yes | The thing being reviewed (diff, doc, config, etc.) |
| `criteria` | String | opt-in | Domain-specific standards/rules for this dimension. **Omitting opts out the dimension entirely.** |
| `depth` | FermiSize | yes (default: M) | Cost/quality tradeoff — how thorough the review should be. XS = quick sanity check, XL = exhaustive deep-dive. |
| `context` | String | optional | Additional context (project architecture, prior findings, etc.) |

| Dimension | What it checks | Criteria examples |
|-----------|---------------|-------------------|
| **Coherence** | Internal consistency — bugs, contradictions, state mismatches, logic errors. "Does this make sense on its own?" | *Coding*: type safety rules, invariant docs. *Design*: consistency checklist. *Config*: schema constraints. |
| **Quality** | Against injected standards. | *Coding*: clippy policy, AGENT.md, style guide. *API*: design principles. *Security*: OWASP policy. |
| **Requirements** | Does it accomplish the stated goal? Integrate into the project? Missing edge cases? | *Coding*: GitHub issue body, spec, acceptance criteria. *Design*: product requirements doc. |
| **Aspirational** | What could be better? Fix now, defer, or accept? | *Coding*: perf heuristics, refactoring patterns. *General*: "good enough" threshold. |

#### Depth as Fermi Size

Depth controls the cost/quality tradeoff per dimension:

| Depth | Behavior | Typical use |
|-------|----------|-------------|
| **XS** | Single-pass sanity check, minimal context | Quick pre-commit check |
| **S** | Focused review, key issues only | Daily dev workflow |
| **M** | Standard review, good coverage | Default for most reviews |
| **L** | Thorough review, edge cases explored | Pre-merge review |
| **XL** | Exhaustive deep-dive, multi-pass | Security audit, critical path |

Depth maps to concrete LLM parameters: prompt detail, number of passes,
context window usage, and whether follow-up questions are generated.

#### DAG Shape

```
                           ┌─→ coherence(criteria, depth) ──────┐
artifact ─────────────────┤                                      │
                           ├─→ quality(criteria, depth) ──────────┼─→ merge ─→ aspirational(criteria, depth) ─→ output
coherence_criteria ────────┘                                      │
quality_criteria ──────────────┘                                  │
requirements_criteria ─────────────→ requirements(criteria, depth)┘
project_context ───────────────────┘
```

Coherence, quality, and requirements run in **parallel** (independent LLM calls).
Aspirational runs **last** — sees all prior findings and classifies each as
must-fix / defer / accept.

Dimensions with no `criteria` provided are **skipped** (not called). This lets
callers run just coherence (XS depth) for a quick sanity check, or all four at
L depth for a thorough pre-merge review.

#### Domain Modeling (like languages/services)

Review domains are modeled as **criteria bundles** — similar to how the codebase
models language targets and service interfaces as external dependencies:

```
coding_review = ReviewProfile {
    coherence_criteria: "clippy invariants, type safety, state consistency",
    quality_criteria: load_file("AGENT.md") + load_file("clippy.toml"),
    requirements_criteria: load_github_issue(issue_number),
    aspirational_criteria: "refactoring heuristics, perf patterns",
    default_depth: M,
}
```

Future domains (design review, security audit, config review) provide different
criteria bundles but use the same 4-dimension pipeline.

#### Modeling Requirements (composable DAG types)

Review dimensions must be **first-class DAG types** — composable, typed, and
registered like all other domain modeling in the system. Ad-hoc string configs
that bypass the type system will get out of hand fast.

**Type hierarchy** (bottom-up):

1. **`Review`** (base type): the atomic review unit. Interface:
   - Input: `artifact` (content blob — `BlobMeta` / content hash, already in
     `gunbc-infra`) — **what** we are reviewing.
   - Input: `criteria` (`ReviewCriteria`) — **what we are reviewing against**.
     Not a raw string — a typed port with `PortTypeTag` so the compiler catches
     mismatches.
   - Input: `depth` (`FermiDepth`) — cost/quality tradeoff signal.
   - Output: `findings` (`ReviewFindings`) — typed structured results.

   This is the simplest callable: "review X against Y at depth Z, return
   findings." Everything else composes from this.

2. **`ReviewCriteria`** (criteria type): structured description of the standards
   a review checks against. Domain-specific criteria (clippy rules, security
   policies, design guidelines, acceptance criteria) are values of this type.
   Like how `TransportRequest` has Shell/File/Rest variants, `ReviewCriteria`
   can carry domain-specific content while remaining a single typed port.

3. **Higher-order dimension types** (`CoherenceReview`, `QualityReview`,
   `RequirementsReview`, `AspirationalReview`): specialized `Review` subtypes.
   Each refines the base interface with dimension-specific semantics — e.g.,
   `AspirationalReview` takes additional `prior_findings` input. Each is a
   SubDag with typed input/output ports, composable into larger pipelines via
   standard DAG wiring. These are more specific/intricate about their criteria
   definitions — similar to how the-gunbai and gunb.ai define specialized
   service interfaces on top of generic transport.

4. **`ReviewProfile`** (criteria bundle): maps dimensions to domain-specific
   criteria + depth. Registered via inventory like `SystemModel` behaviors.
   Profiles are the composition unit for domain specialization — a coding
   review profile, a security audit profile, a design review profile each
   provide different criteria values but wire into the same dimension SubDags.

**Patterns from existing codebase to reuse**:

| Pattern | Source | Application |
|---------|--------|-------------|
| `PortTypeTag` trait | `core/ir/src/typed_io.rs` | `ReviewCriteriaTag`, `ReviewFindingsTag`, `FermiDepthTag`, `ArtifactTag` — compile-time markers |
| `TypedInput<T>` / `TypedOutput<T>` | `core/ir/src/typed_io.rs` | Dimension SubDag ports: `TypedInput<ArtifactTag>`, `TypedInput<ReviewCriteriaTag>`, `TypedOutput<ReviewFindingsTag>` |
| `SystemModel` + `Behavior` | `core/ir/src/system_model.rs` | Register `ReviewProfile` variants (coding, security, design) via inventory |
| `FermiEstimate` | `the-gunbai: gunbai-types/estimate.rs` | Model for `FermiDepth` enum (XS/S/M/L/XL) |
| `Contract` (prereqs/provisions) | `the-gunbai: gunbai-types/contract.rs` | Aspirational declares prerequisite on coherence + quality + requirements findings |
| Content blob (`BlobMeta`) | `lib/blob` | Artifact input — the thing being reviewed |
| `TransportRequest` variants | `core/ir/src/transport/` | Model for `ReviewCriteria` — single type, multiple domain-specific payloads |
| `ObligationCategory` | `core/ir/src/obligation.rs` | New variants for review lifecycle if needed |

**Key invariants**:
- Every review dimension is a SubDag node with typed ports — composes via
  standard DAG edges, no special orchestration.
- Aspirational's dependency on the other three = normal DAG edge wiring
  (findings ports → aspirational context port), not a separate scheduler.
- Dimensions are discoverable via inventory registration (not a hardcoded list).
- The DAG *is* the orchestrator — no separate "review orchestrator" node.

### Phase 2: Review Pipeline

| ID | Task | Deps | Size |
|----|------|------|------|
| **W4** | **Abstract review DAG**: Build `dsl/tools/review.dag` with the 4-dimension model. Each dimension is a SubDag with `artifact`, `criteria`, `depth`, `context` input ports. Dimensions are opt-in: no `criteria` = skipped. `depth` defaults to M. Aspirational sees merged findings from the other three. Output: findings JSON with dimension labels and severity (must-fix / defer / accept). | W3 | M |
| **W5** | **Coding review profile**: Implement `ReviewProfile` for code — loads criteria from `AGENT.md` + `clippy.toml` (quality), GitHub issue body (requirements), refactoring heuristics (aspirational), invariant docs (coherence). `gunbc review --pr <number>` fetches diff, resolves profile, runs pipeline. | W4 | M |
| **W6** | **CI status as review context**: `gunbc review --pr <number>` queries CI via `gh run list`. If failing, inject failure context (which tests, which step) into the requirements dimension's `context` port. Also support `--depth XS|S|M|L|XL` flag to override default depth for all dimensions. | W5 | S |

### Phase 3: Orchestration

| ID | Task | Deps | Size |
|----|------|------|------|
| **W7** | **`gunbc pipeline` command**: Orchestrates the full daily flow for a branch/PR: fetch CI status → run 4-dimension review → output summary with actionable items categorized as must-fix / defer / accept. Single command before submitting. | W6 | M |
| **W8** | **GitHub issue integration**: `gunbc pipeline --issue <number>` reads issue description as the `original_intent` input to the requirements dimension. Validates that implementation matches intent. If no issue linked, requirements dimension uses PR description instead. | W7 | S |

---

## Sprint 4: DeferredCallableOp Elimination

Better informed after W1-W8 reveal which deferred callables are actually exercised.

| ID | Task | Deps | Size |
|----|------|------|------|
| **P6** | Per-module domain ops (see design decision above) | W4 | L |

---

## Parallelization Guide

```
SPRINT 1 ├─ DONE                   (2984/2984 passing, 0 failures)
         │
    ─────┤ (Sprint 2: review fixes + polish)
         │
         ├─ R2                     (review finding: wildcard resources)
         ├─ R3→(R4, R5)→R6         (modeled backend correctness hardening)
         ├─ P12                    (resolve_infrastructure typed-lowering migration)
         │
    ─────┤ (Sprint 3: dev pipeline — real workflow)
         │
         ├─ W1→W2→W3              (credentials + CLI binary + multi-provider)
         ├─ W4→W5→W6              (abstract review model → PR mode → CI context)
         ├─ W7→W8                  (orchestration + issue integration)
         │
    ─────┤ (Sprint 4: cleanup informed by real usage)
         │
         └─ P6                     (per-module domain ops, L)
```

**Sprint 2**: R2 + R3/R4/R5/R6 modeling-first backend hardening + remaining integration fixes.
**Sprint 3**: W1 (`gunbc review` CLI) is the critical path — first real end-to-end
execution. W4 (abstract review DAG with 4 dimensions) is the design centerpiece.
By end of sprint: `gunbc pipeline --pr 123` runs coherence + quality + requirements
+ aspirational review with CI context, outputs must-fix / defer / accept findings.
**Sprint 4**: P6 informed by which deferred callables real execution exercises.
**Backlog**: XL features and migration work in `backlog.md`.
