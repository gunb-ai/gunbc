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

### Design Decision Status

| Decision | Status | Notes |
|---|---|---|
| Backend semantics encoded in IR | Resolved | Applied in `R3`-`R6` (now done). |
| External system semantics typed | Resolved | Applied in `R7`-`R12` (now done). |
| DeferredCallableOp elimination strategy | Resolved (impl pending) | Contract resolved; implementation tracked by `P6`/`P12`. |
| Runtime environment | Resolved | Local-first CLI, env creds + CI/cloud WIF path. |
| Abstract review model | Resolved | Four-dimension typed model with criteria-driven opt-in. |
| Workflow minimum unit + exclusive coordination | Resolved | Canonicalized in WF design docs (`WF1-D`..`WF4-D`). |
| Control-token model | Resolved (strict default) | Keep completion-gated control; require explicit success guards for fail-fast functional paths. |
| Cached `result` persistence | Resolved (strict/minimal default) | Persist typed summary/reference by default; optional full payload in CAS. |
| Changed-input routing authority | Resolved (strict correctness) | Optimization hint only; non-authoritative for soundness. |
| Conflict commutativity exceptions | Resolved (strict default) | No commutativity exceptions in current phase. |

### Tonight Handoff Lanes (Open Work)

Use these lanes to assign workers with minimal overlap and clear stop conditions.

| Lane | Task IDs | Preconditions | Primary Files/Areas | Done When | Verify |
|---|---|---|---|---|---|
| A: Resolver de-stringing | `P12` -> `P6` | none (`P12` first) | `resolve.rs`, `daglang-lower`, runtime resolver/dispatch | no string-prefix op resolution; no deferred passthrough fallback for migrated modules | `cargo test --workspace`, resolver golden tests |
| B: Workflow planner core | `WF1` -> `WF2` -> `WF3` -> `WF4` -> `WF5` | `WF1-D`..`WF4-D` reviewed | `gunbc-dag` workflow schema/planner/ledger/executor, workflow docs | deterministic typed plan, claim-safe admission, key/rehydration correctness, plan explainability | `cargo test --workspace`, `cargo run -p gunbc-dag --bin gunbc-workflow -- --plan ci` |
| C: Workflow cutover/perf | `WF6` -> `WF7` -> `WF8` -> `WF9` | Lane B complete | workflow entrypoints + `Makefile` wrappers + CI wiring | `make ci`/`make test-all` use planner path with SLO telemetry | `make ci`, `make test-all`, CI dry run |
| D: Modeling hardening graph/runtime | `M8` -> `M9` -> `M16` and `M10` -> `M11` | corresponding `-D` tasks approved | IR type DAG/system-model/transport + runtime resource/dry-run paths | metadata inertness, typed dependency markers, strict dry-run enforced | targeted model tests + `cargo test --workspace` |
| E: Security/install/process drift | `M7`, `M15`, `M13` -> `M14`, `M17` -> `M18` -> `M19` | corresponding `-D` tasks approved | value redaction, installer model, registry/make/CLI contracts, proof harness | no accidental secret leak path; typed PM policy; no projection drift; invariants testable | test suites for each module + planner invariant suite |
| F: Universal capabilities | `WF14-D` -> `WF14` -> `WF15-D` -> `WF15` | `WF1-D`..`WF4-D` reviewed (design deps) | binary dispatch, codegen keyed unit, planner integration | compilation + codegen capabilities keyed and shared across all workflows | `gunbc-workflow --plan gist-snapshot` shows codegen CachedHit |
| G: Gist capability stack | `WF16-D` -> `WF16` -> `WF17` -> `WF18` | Lane F complete | gist graph, gist_modes, credential chain, git state units | git state + credential + transport capabilities minimized; gist modes use planner path | `make gist` warm path, credential sharing across gist/dag-viz |
| H: Remaining capabilities | `WF19-D` -> `WF19` -> `WF20` -> `WF21` -> `WF22` | Lane F complete | bootstrap/makegen/pragma/deps/dag-viz, Makefile | FS write + generator capabilities minimized; all tools on planner path with verification | per-capability hit/miss reporting, cross-workflow sharing observable |

Handoff rules:

1. One worker owns one lane at a time.
2. Every PR title begins with primary task ID (example: `[WF3] ...`).
3. Any behavioral change must include/adjust at least one regression test.
4. If a lane hits unresolved design ambiguity, open/update the matching `*-D` task first.
5. Do not start an implementation task before its `-D` pair is reviewed.

---

## Sprint 2: Review Findings + Polish

### Review Findings

Bug surfaced by automated review. This is real but latent (not causing test failures yet).

| ID | Task | Deps | Size |
|----|------|------|------|
| **R2** | **[STARTED 2026-02-19]** **Wildcard resource semantics deferred**: remove generated/injected `res:file:*` usage for now (coarsen to `res:file`), treat coarse `file` as conflicting with any specific `file:<path>` lock in admission control, and normalize wildcard IDs to coarse `file` in resource accounting. Track full glob semantics as design work in `backlog.md` before enabling pattern-aware admission control. | — | M |

### Code TODOs & DSL Compiler Polish

#### Design Decision (Resolved 2026-02-19): Backend Semantics Must Be Encoded in IR (not naming conventions)

Recent emitter bugs (Go/C redeclaration, MIPS early-return epilogue bypass, MIPS temp
clobber) show a shared issue: backend lowering currently relies on string/name conventions
in places where the IR should carry the semantic constraints.

For this track, prefer **model enrichment first**:
- Add/extend IR nodes so correctness is enforced by construction.
- Lowerers and renderers should consume explicit modeled semantics, not infer behavior
  from hardcoded names.
- Validation passes are still useful, but treated as guardrails, not the primary design.
- Tactical wrappers are acceptable only as interim mitigations; final state must encode
  declaration/scope/return semantics explicitly in IR.

| ID | Task | Deps | Size | Source |
|----|------|------|------|--------|
| **R3** | **[DONE 2026-02-19]** **Backend modeling enrichment RFC + IR schema update**: define and land minimal IR/API extensions needed for correctness-by-construction across these bugs. Scope: (a) Go binding intent in IR (short-declare vs assign, not encoded via synthetic names like `_, err`), (b) C lexical scope block statement for temporary-lifetime isolation, (c) MIPS explicit return terminator + epilogue destination contract (single-exit semantics), (d) fallible temp allocation API for register pressure, and (e) scope-aware local tracking notes for C->MIPS block lowering. Include migration notes for lowerers/renderers/tests. **Acceptance**: changed IR types compile across emit crate; Go emit path can express declaration vs assignment without string parsing; C/MIPS lowering can represent scoped temps without hardcoded unique names; return/epilogue flow is representable without raw `jr $ra` in lowering logic. | — | M | review synthesis |
| **R4** | **[DONE 2026-02-19]** **Go + C lowerer migration to modeled semantics**: migrate transport-call statement lowering to the new IR contracts. Go must avoid repeated `:=` redeclare failures by construction; C must avoid same-scope `__rc` redefinition by construction (scoped block or equivalent modeled mechanism). **Acceptance**: add regressions with 2+ transport expression statements in one function; generated Go/C compile cleanly; tests assert structural IR behavior (not string fragments only). Interim `Expr::Block` wrapping is acceptable as a stop-gap, but completion requires bind-intent modeling from R3 to be exercised in tests. | R3 | M | review synthesis |
| **R5** | **[DONE 2026-02-19]** **MIPS control-flow + allocator fail-closed migration**: implement single-exit return lowering (all returns route through epilogue path), make temp allocation fail with explicit `LowerError` on exhaustion instead of wrapping/clobbering, and define handling for C block-scope locals during C->MIPS lowering (scope stack or equivalent non-leaking strategy). **Acceptance**: framed functions do not contain body-level direct `JumpReg(Register::Ra)` for lowered returns; deep-expression/register-pressure test fails closed with explicit lowering error, not silent corruption; scoped temps from nested blocks do not alias/leak incorrectly in generated MIPS. | R3 | M | review synthesis |
| **R6** | **[DONE 2026-02-19]** **Holistic backend correctness harness**: add cross-backend adversarial fixtures + smoke compilation checks for generated artifacts (Go/C compile, MIPS assembly structure checks), plus invariant checks that encode the modeled contracts. **Acceptance**: old buggy patterns are caught by tests; new modeled path passes; CI command includes this harness. | R4, R5 | M | review synthesis |

#### Design Decision (Resolved 2026-02-19): External System Semantics Must Not Be Stringly-Typed

The same modeling-first rule applies to infra/GCP code: endpoint contracts, IAM policy
shape, boundary input parsing, and mock seeding should be typed and validated by
construction, with string heuristics only as explicit transitional compatibility paths.

| ID | Task | Deps | Size | Source |
|----|------|------|------|--------|
| **R7** | **[DONE 2026-02-19]** **Typed IAM policy domain model**: replace ad-hoc `serde_json::Value` mutation in IAM binding ops with typed structs (e.g., `IamPolicy`, `IamBinding`) + safe mutation helpers (`ensure_member(role, member)`). Preserve and round-trip `etag`, dedupe members, and support both direct-policy and envelope-policy transport shapes via typed decode adapters. **Acceptance**: `CheckAndPrepareIamBinding` and `CheckAndPrepareSaIamBinding` no longer manually push JSON strings into `bindings`; typed tests cover existing, missing, and duplicate-member cases; `etag` retained in generated setIamPolicy request bodies. | — | M | architecture review |
| **R8** | **[DONE 2026-02-19]** **`MethodMeta` as execution source-of-truth**: add shared request-construction utilities that expand endpoint templates from `MethodMeta` + typed params/query map, and migrate service impls to use this path (not duplicated `format!` URLs). **Acceptance**: service methods stop hardcoding endpoint paths already represented by `*_META`; parity tests enforce constructed URL/method equivalence to metadata; drift between metadata and request wiring is caught by tests. | R7 | M | architecture review |
| **R9** | **[DONE 2026-02-19]** **Fail-closed CLI entrypoint input parsing**: replace fallback `Value::Str` parsing in infra CLI with type-driven parsing based on `TypeId` + `ValueBacking` / compatibility helpers. Unsupported complex carriers should error explicitly with guidance instead of silently coercing to string. **Acceptance**: `parse_input_value` errors for incompatible inputs; list/map/json/basic scalar parsing covered; no silent string fallback for non-string target types. | — | S | architecture review |
| **R10** | **[DONE 2026-02-19]** **Typed REST path-variable binding in `SystemModel`**: move `Invocation::Rest.path` from wildcard `*` style to named placeholders (`{project_id}`, etc.) and extend `validate_system_model` to verify placeholder↔`BehaviorInput` coverage (no unbound placeholders, no missing required path vars). **Acceptance**: GCP models validate with named variables; validator rejects mismatched path vars; tests cover both valid and invalid models. | R8 | M | architecture review |
| **R11** | **[DONE 2026-02-19]** **Strict platform parsing at boundaries**: introduce strict parse APIs (`try_parse`/`FromStr` with real errors) for `Arch`/`Vendor`/`Os`/`AbiEnv`/`ExecutionEnv` at user-config boundaries while keeping best-effort detection paths for host introspection. **Acceptance**: config/CLI parse points can fail closed on unknown tokens; host detect remains tolerant; tests cover strict-reject and tolerant-detect behavior split. | — | S | architecture review |
| **R12** | **[DONE 2026-02-19]** **Mock-default seeding by semantic kind, not port-name heuristics**: migrate GCP mock defaults away from raw port-name matching toward typed semantic hints (`SemanticCarrierKind` and/or refined type aliases). Keep name-based fallback only behind an explicit compatibility path. **Acceptance**: mock seeding still works when ports are renamed but type semantics are preserved; tests demonstrate semantic seeding for audience/project/service-account style inputs without relying on exact port names. | R9 | M | architecture review |

#### Design Decision (Resolved; implementation pending): DeferredCallableOp Elimination Strategy (blocks P6)

P6 replaces `DeferredCallableOp` (identity passthrough) with per-tool domain ops.
Resolution:

1. each deferred callable is replaced by a module-scoped typed `*Op` enum variant
   with an explicit `Executable` implementation,
2. dry-run behavior is implemented inside each typed op via
   `ExecutionMode::DryRun` and returns typed deterministic outputs (no identity passthrough),
3. unknown callables fail closed (`Err(unknown_callable(...))`) once module
   migration is complete, and
4. resolver behavior is exhaustive typed dispatch (`P12`), not string-prefix inference.

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

### Design Decision (Resolved 2026-02-19): Runtime Environment

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

### Design Decision (Resolved 2026-02-19): Abstract Review Model

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

Execution note:

1. `P6` is defined in Sprint 2 with full scope/acceptance; schedule execution after `W4` to prioritize exercised callables first.
2. Use `P12` as prerequisite cleanup to eliminate resolver string-prefix ambiguity before broad `P6` migration.

---

## Sprint 5: Workflow Minimal Execution Model (CI/Test-All)

**Goal**: make `make ci` and `make test-all` warm-path behavior complete in seconds by
construction, with no redundant work and no implicit fallback paths.

### Design Decision (Resolved 2026-02-19): Minimum Unit of Work + Exclusive Coordination

Workflow execution must be modeled as typed, composable **minimum work units** (not command
chains). Every unit must declare:

1. explicit typed inputs and outputs,
2. deterministic materialization key inputs,
3. exclusive resource claims (or declared shared capacity), and
4. downstream coordination contracts (what can run only after this unit commits).

This makes work naturally mutually exclusive where needed and deterministically coordinated for
downstream nodes.

Reference design doc: `docs/design/workflow-minimal-execution-model.md`.

### Design Decision (Resolved 2026-02-19): Control/Dataflow Semantics

For WF1-WF4 scope, orchestration uses **completion-gated control**:

1. control readiness gates are based on upstream `commit` (completion), not domain success,
2. domain success/failure is carried on typed `result` dataflow payloads,
3. report/aggregate behavior is fail-late by construction (consumes committed results),
4. success-gated branching must be modeled explicitly via typed guard units over `result`
   (no implicit "success-only control edge" semantics in this phase), and
5. node readiness requires both control prerequisites and required dataflow inputs to be materialized.

Strict default wiring policy:

1. functional units must be success-guarded unless explicitly exempted,
2. report/aggregation units remain commit-gated for failure completeness.

### Design-First Gate (Required)

For modeling tasks in this sprint, implementation is blocked until a concrete
design artifact is reviewed:

1. each design doc must include concrete DAG structure (`Dag<...>` node list,
   typed edges including `EdgeKind`, and resource/input/output contracts),
2. each design doc must include invalidation and admission rules (no ambient
   dependencies), and
3. task implementation IDs must depend on corresponding `-D` design IDs.

| ID | Task | Deps | Size |
|----|------|------|------|
| **WF1-D** | **Workflow schema design spec**: author concrete design for `WorkflowSpec` as `Dag<WorkflowUnit>` wrapper, typed IDs (`NodeId`, `PortName`), and op semantics without parallel graph structures. **Design doc**: `docs/design/workflow/wf1-wf4-dag-design-pack.md` (Sections 2-4). **Acceptance**: design doc includes concrete DAG skeletons for `ci` and `test-all`, typed interface contracts, explicit no-fallback guarantees, and a `ProcessUnitRef -> ProcessSpec semantic-version/digest` contract used for `op_version` derivation in keying. | — | S |
| **WF1** | **Minimum work-unit schema**: implement approved `WF1-D` design (typed units over existing DAG primitives, no untyped shell fallback nodes). **Acceptance**: no planner node can be created from an untyped shell string; each unit has stable ID and explicit IO contract; schema docs landed. | WF1-D | M |
| **WF2-D** | **Mutual-exclusion/admission design spec**: define concurrency model using typed resource identities + access modes and derive admission behavior from declared resource ports/claims. **Design doc**: `docs/design/workflow/wf1-wf4-dag-design-pack.md` (Section 5). **Acceptance**: conflict matrix, fairness/tie-break rules, and control-edge sequencing model documented with concrete DAG examples. | WF1-D | S |
| **WF2** | **Mutual-exclusion claim model**: implement approved `WF2-D` admission model so conflicting units cannot co-run. **Acceptance**: planner/executor rejects unsatisfied/conflicting claims fail-closed; conflict diagnostics include both unit IDs + claim IDs; tests cover read/read allowed and write/write denied cases. | WF1, WF2-D | M |
| **WF3-D** | **Key/ledger causality design spec**: define canonical key payload structure, typed miss-reason ADT, and ledger-state ADT with atomic persistence semantics. **Design doc**: `docs/design/workflow/wf1-wf4-dag-design-pack.md` (Section 6). **Acceptance**: no ambient env/toolchain probing in key computation; miss causes are structurally diffable from payloads; fan-in contributors for multi-producer ports are preserved deterministically in key payloads; key encoding/versioning contract is explicit (`key_format_version` + canonical serializer rules); cached-hit output rehydration contract (CAS-backed) is documented, including typed `result` payload materialization; crash-safe write pattern documented. | WF1-D | S |
| **WF3** | **Deterministic materialization keys + miss reasons**: implement approved `WF3-D` key/ledger model. **Acceptance**: same repo state yields identical keys; key drift is explained by explicit miss reason; no mtime-only freshness path in planner core; cached-hit nodes rehydrate declared outputs (including `result` when consumed) before downstream dataflow. | WF1, WF3-D | M |
| **WF4-D** | **Downstream coordination design spec**: define readiness/commit boundaries with typed graph semantics (`EdgeKind::Control` where appropriate), including failure/skip propagation rules. **Design doc**: `docs/design/workflow/wf1-wf4-dag-design-pack.md` (Section 7). **Acceptance**: concrete DAGs prove downstream nodes block on uncommitted prerequisites without implicit make-order dependence; dependency gate uses commit/output-availability semantics (not domain-success semantics); dataflow/readiness interaction is explicit (required data inputs must exist before run). | WF1-D, WF2-D | S |
| **WF4** | **Downstream coordination contract**: implement approved `WF4-D` coordination model so downstream units execute only after upstream commit/output availability. **Acceptance**: planner proves topological + contract consistency before execution; downstream units are blocked on uncommitted prerequisites with explicit reason output; domain-failure results still reach report/aggregate nodes. | WF1, WF2, WF4-D | M |
| **WF5** | **Planner dry-run + execution plan explainability**: add a planner mode that prints execute-set, cache-hit/miss set, and critical path before running. **Acceptance**: `gunbc-workflow --plan ci` and `--plan test-all` produce deterministic node lists and miss reasons; tests pin output stability for a fixed fixture repo state. | WF2, WF3, WF4 | S |
| **WF6** | **Port `ci` to workflow planner**: implement `gunbc-workflow ci` using the new unit model and planner/executor, with behavior parity to current `gunbc-ci`. **Acceptance**: CI path no longer composes redundant prerequisite chains; all `ci` steps execute via typed units with claims + keys; parity tests validate outputs and failure semantics. | WF5 | M |
| **WF7** | **Port `test-all` to workflow planner**: implement `gunbc-workflow test-all` with minimal dirty-closure execution and shared artifacts with `ci` flow. **Acceptance**: warm no-op executes zero functional units; single-input edits execute only transitive dirty closure; regression tests assert no duplicate generator/build unit execution in one run. | WF5 | M |
| **WF8** | **Makefile thinning + strict mode cutover**: convert `make ci`/`make test-all` into thin wrappers over `gunbc-workflow`; remove redundant legacy chaining for these targets; keep explicit strict-mode failures for unmapped/deprecated paths. **Acceptance**: Make targets are transport-only wrappers; removed duplicate orchestration for these commands; CI gate asserts planner path is used. | WF6, WF7 | S |
| **WF9** | **Latency SLO instrumentation + guardrails**: add run-ledger timing metrics and CI assertions for warm-path budgets (seconds-scale), plus “top slow units” reporting. **Acceptance**: logs expose total units/hits/misses/critical path; failing SLO budgets fail CI with actionable slow-unit breakdown. | WF6, WF7 | S |

### Modeling Intake

Additional semantic-erasure/modeling hardening items from 2026-02-19 feedback are
tracked in `TODO/modeling.md` and should be promoted into sprint lanes as they are
prioritized.

### Resolved Strict Defaults (Locked 2026-02-19)

| ID | Decision | Chosen Default | Deps | Size |
|---|---|---|---|---|
| **WF10-D** | Control-token model beyond fail-late default (`done`/`ok` split) | Keep completion-gated control + explicit typed success guards (no dual-token expansion in current phase). | WF4-D | S |
| **WF11-D** | Cached `result` persistence strategy (full payload vs typed summary/reference) | Typed summary/reference is mandatory baseline; full payload persistence is optional per-unit policy. | WF3-D | S |
| **WF12-D** | Changed-input routing authority boundary | Routing remains optimization hint only; never authoritative for correctness in current phase. | WF3-D | S |
| **WF13-D** | Conflict commutativity policy for resource claims | No commutativity exceptions; conflicting claims require explicit ordering. | WF2-D | S |

Additional active open items:

1. Resource wildcard pattern semantics remain explicitly deferred (`R2` + `backlog.md`).
2. Deferred-callable migration is implementation-open but design-resolved (`P6`, `P12`).

---

## Sprint 5b: Tool Workflow Minimization (Gist + All Targets)

**Goal**: extend the planner/ledger model from Sprint 5 to all tool workflows by
minimizing each **capability requirement** (credentialing, upload, filesystem, git state,
codegen, compilation, pure computation) holistically. Each workflow decomposes into these
capabilities; if each capability is minimal, the workflow is minimal.

**Design reference**: `docs/design/workflow-minimal-execution-model.md` Sections 15.1-15.8.

### Design Decision (Resolved 2026-02-19): Tool Workflows Use Same Planner

Tool workflows reuse the same `WorkflowSpec`, `MaterializationKey`, and `RunLedger`
infrastructure from Sprint 5. No separate tool-specific caching layer. The global
ledger handles cross-workflow capability sharing (e.g., codegen freshness shared
between `ci` and `gist`, credential resolution shared between `gist` and `dag-viz`).

### Design Decision (Resolved 2026-02-19): Capability-First Minimization Order

Minimization is applied per-capability (not per-workflow), ordered by leverage:

1. Codegen + Compilation (universal — every workflow uses these)
2. Credentialing (most expensive single capability — WIF/OIDC/SecretManager chain)
3. Git State (shared across gist/dag-viz/review families)
4. Filesystem (shared across generator workflows: bootstrap/makegen/pragma/testgen)
5. Pure Computation (falls out from general keying)
6. Network Transport (correctly volatile — upstream minimization is the win)

### Design-First Gate (Required)

Same gate as Sprint 5: each capability design must include keying contract,
invalidation signals, and cross-workflow sharing semantics before implementation.

### Phase T-A: Universal Capabilities (Codegen + Compilation)

| ID | Task | Deps | Size |
|----|------|------|------|
| **WF14-D** | **Compilation capability design spec**: define binary freshness as a keyed unit — key inputs are source content hashes + `cargo metadata` dependency hashes + compiler version. Specify pre-built binary dispatch model replacing `cargo run`. **Design doc**: `docs/design/workflow/tool-workflow-design-pack.md` (Section 2). **Acceptance**: design includes keying contract for binary freshness, before/after Make target shapes, and cross-workflow sharing semantics (one binary build serves all tool invocations). | WF1-D | S |
| **WF14** | **Compilation capability implementation**: implement binary freshness as a planner-managed keyed unit. Make targets dispatch to pre-built binaries, bypassing `cargo run`. **Acceptance**: `make gist` no longer invokes `cargo run`; binary staleness is detected by planner key (source hash + cargo metadata), not by Cargo's internal check; one compilation unit is shared across all tool workflows. | WF14-D, WF1 | M |
| **WF15-D** | **Codegen capability design spec**: define codegen freshness as a keyed unit — key inputs are DSL source hashes (`dsl/**/*.dag`) + codegen binary semantic version. Specify how tool workflows declare codegen as a typed input dependency (not Make prerequisite). **Design doc**: `docs/design/workflow/tool-workflow-design-pack.md` (Section 3). **Acceptance**: design proves `ensure-codegen` Make target is eliminated; codegen freshness resolved via ledger lookup (no subprocess); miss reason for stale codegen is explicit; codegen unit is shared across all workflows that depend on generated artifacts. | WF3-D, WF14-D | S |
| **WF15** | **Codegen capability implementation**: implement codegen as a planner-managed keyed unit with ledger-backed freshness. Remove `ensure-codegen` as Make prerequisite for planner-managed targets. **Acceptance**: warm-state tool invocation does not spawn codegen subprocess; codegen staleness triggers re-run with typed miss reason; global ledger shares codegen freshness across `ci`, `gist`, `bootstrap`, etc. | WF15-D, WF3 | M |

### Phase T-B: Gist Capability Stack (Credentialing + Git State + Network)

| ID | Task | Deps | Size |
|----|------|------|------|
| **WF16-D** | **Gist capability decomposition design spec**: decompose all three gist modes into capability units (git state, filesystem, credentialing, pure computation, network transport). For each unit: define key inputs, invalidation signals, volatility, and cross-workflow sharing. Special focus on credential capability: local-dev path (env var hash), cloud path (WIF sub-chain with per-step TTL keying), and replacement of `GUNBC_CLOUD_CONFIG_REQUIRED=1` with explicit `runtime_mode` input port. **Design doc**: `docs/design/workflow/tool-workflow-design-pack.md` (Section 4). **Acceptance**: each capability unit has explicit keying contract; credential chain decomposes into independently-keyed sub-units; warm-state plan for each mode shows only volatile transport in execute set; git state and credential units are marked as shared with dag-viz family. | WF1-D, WF3-D, WF15-D | M |
| **WF16** | **Gist snapshot capability port**: implement `gunbc-workflow gist-snapshot` with keyed capability units for git state (`current_branch`, `ls_files`), filesystem (`read_files`), pure (`render_snapshot`), credential (`resolve`), and volatile transport (`gist_create`). **Acceptance**: warm no-op with unchanged repo: all capability units resolve from ledger, only transport executes; parity test validates output matches current behavior. | WF16-D, WF5, WF14, WF15 | M |
| **WF17** | **Gist diff capability port**: implement `gunbc-workflow gist-diff` with keyed git diff unit. **Acceptance**: key miss on `base_ref` or `HEAD` change triggers minimal dirty closure (diff + render + transport); credential unit shared with snapshot mode via global ledger. | WF16-D, WF5, WF14, WF15 | S |
| **WF18** | **Gist recent capability port + credential sub-chain**: implement `gunbc-workflow gist-recent` with full credential sub-chain keying (WIF exchange, IAM impersonation, Secret Manager access — each independently keyed with TTL). **Acceptance**: warm no-op with no new commits + valid credential TTL: zero capability units execute except transport; `runtime_mode` is an explicit typed input, not an ambient env probe. | WF16-D, WF5, WF14, WF15 | M |

### Phase T-C: Remaining Capability Ports (FS Write + Generator Workflows)

| ID | Task | Deps | Size |
|----|------|------|------|
| **WF19-D** | **Generator + remaining tool capability design spec**: decompose bootstrap, makegen, pragma, deps, dag-viz (3 modes), dag-snapshot into capability units. Focus on the filesystem write (content-upsert) capability: keying contract where generation inputs determine skip (not post-hoc content comparison). dag-viz shares git state + credential capabilities from WF16-D. **Design doc**: `docs/design/workflow/tool-workflow-design-pack.md` (Sections 5-9). **Acceptance**: each tool has capability decomposition with keying contracts; content-upsert skip is a consequence of input keying, not a separate mechanism; shared capability units are identified and marked. | WF1-D, WF3-D, WF15-D | M |
| **WF19** | **Generator workflow capability port (bootstrap/makegen/pragma)**: implement planner path with keyed generation + filesystem-upsert capabilities. **Acceptance**: warm no-op executes zero capability units; generation step skips when generation inputs (registry data, templates, config) are unchanged; filesystem write skips as consequence. | WF19-D, WF5, WF14, WF15 | M |
| **WF20** | **Remaining tool capability port (deps/dag-viz/dag-snapshot)**: implement planner path for remaining tools. dag-viz modes reuse git state + credential capability units from gist family. **Acceptance**: warm no-op executes zero capability units; dag-viz credential resolution shared with gist via global ledger. | WF19-D, WF5, WF14, WF15 | M |

### Phase T-D: Cutover + Verification

| ID | Task | Deps | Size |
|----|------|------|------|
| **WF21** | **Makefile thinning for all tool targets**: convert all `make <tool>` targets to thin wrappers over `gunbc-workflow <tool>`; remove `ensure-codegen` as prerequisite; remove `cargo run` invocations for planner-managed tools. **Acceptance**: all tool Make targets are transport-only shims; no duplicate orchestration remains. | WF16, WF17, WF18, WF19, WF20 | S |
| **WF22** | **Capability minimization verification**: extend WF9 instrumentation to all tool targets. Planner reports per-capability hit/miss/execute status. **Acceptance**: `gunbc-workflow --plan gist-snapshot` (and all tools) emits capability-level breakdown (which capabilities hit, which missed, why); cross-workflow sharing is observable (e.g., "credential.resolve: CachedHit from gist-diff run"). | WF9, WF21 | S |

### Lane Extension (Updated from Sprint 5)

| Lane | Task IDs | Done When |
|---|---|---|
| B: Workflow planner core | (unchanged) `WF1`→`WF2`→`WF3`→`WF4`→`WF5` | planner ready for both ci/test-all and tool workflows |
| C: Workflow cutover/perf | (unchanged) `WF6`→`WF7`→`WF8`→`WF9` | ci/test-all use planner path |
| **F: Universal capabilities** | `WF14-D`→`WF14`→`WF15-D`→`WF15` | compilation + codegen capabilities keyed and shared across all workflows |
| **G: Gist capability stack** | `WF16-D`→`WF16`→`WF17`→`WF18` | git state + credential + transport capabilities minimized; gist modes use planner path |
| **H: Remaining capabilities** | `WF19-D`→`WF19`→`WF20`→`WF21`→`WF22` | FS write + generator capabilities minimized; all tools on planner path with verification |

Lanes F and G can start after WF1-D..WF4-D design review (design deps only).
Lane G depends on Lane F completion.
Lane H depends on Lane F and can parallelize with Lane G (different capability families).

---

## Sprint 6: Modeling Hardening (Design-First)

**Goal**: eliminate remaining semantic erasure across system-model metadata,
resource declarations, dry-run behavior, secret handling, installer modeling,
transport-contract surfaces, and cross-workflow non-redundancy proof gaps.

### Design-First Gate (Required)

For every task in this sprint:

1. implementation is blocked on the matching `-D` design task,
2. design must satisfy the corresponding checklist in `TODO/modeling.md`, and
3. design review must include concrete DAG/typed-contract structures where runtime
   behavior or orchestration is affected.

| ID | Task | Deps | Size |
|----|------|------|------|
| **M7-D** | **Secret redaction design spec**: define secret capability boundary model, explicit plaintext extraction API, and enforcement points (formatting + lint guardrails) per `TODO/modeling.md` M7 checklist. | — | S |
| **M7** | **Secret redaction by default**: implement approved `M7-D` model so secret-bearing values are redacted by default in all display/debug paths and plaintext extraction is transport-boundary-only. | M7-D | M |
| **M8-D** | **`TypeOp::Meta` design spec**: define inert metadata payload model, migration plan from metadata-over-`Validate(Custom(...))`, and erasure-invariance test contract per `TODO/modeling.md` M8 checklist. | — | S |
| **M8** | **Semantically inert metadata op**: implement approved `M8-D` model (`TypeOp::Meta`) and migrate system-model metadata/property encoding off validation nodes. | M8-D | M |
| **M9-D** | **Typed dependency marker design spec**: define typed dependency identity/edges and migration from string marker conventions per `TODO/modeling.md` M9 checklist. | M8-D | S |
| **M9** | **Typed dependency markers**: implement approved `M9-D` model and remove string-prefix dependency semantics from runtime/validator logic. | M8, M9-D | S |
| **M10-D** | **Resource declaration + auto-wiring design spec**: define mandatory effectful-resource declaration rule, auto-wiring policy, and admission derivation model per `TODO/modeling.md` M10 checklist. | WF2-D | M |
| **M10** | **Mandatory resource declarations + auto-wiring**: implement approved `M10-D` model and enforce fail-closed behavior for undeclared effectful I/O. | WF2, M10-D | L |
| **M11-D** | **Strict dry-run poisoning design spec**: define strict/lenient dry-run semantics, poison/unset propagation, and fail-fast trace behavior per `TODO/modeling.md` M11 checklist. | M10-D | S |
| **M11** | **Strict dry-run mode**: implement approved `M11-D` model and wire strict mode into CI/testgen/integration paths. | M10, M11-D | M |
| **M15-D** | **Typed package-manager design spec**: define strict `PackageManagerId`, explicit selection policy, and compatibility boundary per `TODO/modeling.md` M15 checklist. | — | S |
| **M15** | **Typed install planning**: implement approved `M15-D` model across installer and tool-upsert bridging with fail-closed unknown PM handling. | M15-D | M |
| **M16-D** | **SystemModel/TransportBehavior unification design spec**: define shared invocation contract and parity-test model per `TODO/modeling.md` M16 checklist. | M8-D, R8, R10 | M |
| **M16** | **SystemModel/TransportBehavior unification**: implement approved `M16-D` shared contract model and remove/guard duplicate spec surfaces. | M8, M9, M16-D | M |
| **M17-D** | **Global flattening + context-free identity design spec**: define flatten-before-execute contract, `WorkIdentity` semantics independent of workflow node names, and cross-workflow dedup behavior per `TODO/modeling.md` M17 checklist. | WF3-D, WF4-D | M |
| **M17** | **Global flattening + context-free identity**: implement approved `M17-D` model so equivalent work across `ci`/`test-all` is unified and executed once per equivalent key payload. | WF3, WF4, M17-D | L |
| **M18-D** | **Single semantic authority/projection design spec**: define canonical model + generated/validated projection boundaries (Make/CLI/report) per `TODO/modeling.md` M18 checklist. | M17-D | M |
| **M18** | **Projection-only surfaces + drift enforcement**: implement approved `M18-D` model so wrapper surfaces cannot introduce divergent dependency/effect semantics. | M17, M18-D | M |
| **M19-D** | **Formal non-redundancy proof design spec**: define invariant suite + diagnostic model for at-most-once, minimal closure, and single-writer ordering per `TODO/modeling.md` M19 checklist. | M17-D, M18-D | M |
| **M19** | **Formal non-redundancy proof harness**: implement approved `M19-D` invariants and CI gates over planner preflight + execution/ledger traces. | M17, M18, M19-D | M |

---

## Parallelization Guide

```
SPRINT 1 ├─ DONE                   (2984/2984 passing, 0 failures)
         │
    ─────┤ (Sprint 2: review fixes + polish)
         │
         ├─ R2                     (review finding: wildcard resources)
         ├─ R3→(R4, R5)→R6         (modeled backend correctness hardening)
         ├─ R7→R8→R10              (typed GCP/service-model semantics)
         ├─ R9→R12                 (typed CLI boundary + semantic mock seeding)
         ├─ R11                    (strict platform parsing boundaries)
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
         │
    ─────┤ (Sprint 5: minimal workflow execution model)
         │
         └─ WF1-D→(WF2-D,WF3-D)→WF4-D→WF1→WF2→WF3→WF4→WF5→(WF6,WF7)→WF8→WF9
         │
    ─────┤ (Sprint 5b: tool workflow minimization — gist + all targets)
         │
         ├─ Lane F: WF14-D→WF14→WF15-D→WF15
         │                                 (universal: compilation + codegen capabilities)
         ├─ Lane G: WF16-D→WF16→WF17→WF18
         │                                 (gist: credential + git state + transport)
         └─ Lane H: WF19-D→WF19→WF20→WF21→WF22
                                           (remaining: FS write + generators + cutover)
         │
    ─────┤ (Sprint 6: modeling hardening, design-first)
         │
         ├─ M7-D→M7                (secret redaction by construction)
         ├─ M8-D→M8→M9-D→M9        (metadata + dependency typing)
         ├─ M10-D→M10→M11-D→M11    (resource declarations + strict dry-run)
         ├─ M15-D→M15              (typed package-manager modeling)
         ├─ M16-D→M16              (SystemModel/TransportBehavior unification)
         └─ M17-D→M17→M18-D→M18→M19-D→M19
                                   (global flattening + anti-duplicate-modeling proofs)
```

**Sprint 2**: R2 + R3/R4/R5/R6 backend modeling hardening + R7/R8/R9/R10/R11/R12
external-system modeling hardening + remaining integration fixes.
**Sprint 3**: W1 (`gunbc review` CLI) is the critical path — first real end-to-end
execution. W4 (abstract review DAG with 4 dimensions) is the design centerpiece.
By end of sprint: `gunbc pipeline --pr 123` runs coherence + quality + requirements
+ aspirational review with CI context, outputs must-fix / defer / accept findings.
**Sprint 4**: P6 informed by which deferred callables real execution exercises.
**Sprint 5**: WF track now gates implementation behind reviewed DAG-first design
artifacts (`WF1-D`..`WF4-D`), then lands typed minimum units + exclusive claims +
downstream coordination contracts before porting `ci`/`test-all`.
**Sprint 5b**: Extends Sprint 5 planner/ledger to all tool workflows via
capability-first minimization. Lane F makes the two universal capabilities
(compilation + codegen) keyed and shared across all workflows. Lane G minimizes
the gist capability stack (credential chain with TTL sub-keying, git state,
network transport). Lane H minimizes remaining capabilities (FS write/upsert
for generator workflows) and cuts over all tool targets to planner path with
capability-level verification instrumentation.
**Sprint 6**: M track is now promoted with explicit paired design/implementation
tasks (`M7-D`..`M19-D`) and checklist-based review gates from `TODO/modeling.md`.
**Backlog**: XL features and migration work in `backlog.md`.
