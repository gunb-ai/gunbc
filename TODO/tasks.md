# Task Sheet — Dependency-Ordered, Parallelizable

**Last updated**: 2026-02-20
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
| DeferredCallableOp elimination strategy | Resolved (done) | Implemented in `P6`/`P12`. |
| Runtime environment | Resolved | Local-first CLI, env creds + CI/cloud WIF path. |
| Abstract review model | Resolved | Four-dimension typed model with criteria-driven opt-in. |
| Workflow minimum unit + exclusive coordination | Resolved | Canonicalized in WF design docs (`WF1-D`..`WF4-D`). |
| Control-token model | Resolved (strict default) | Keep completion-gated control; require explicit success guards for fail-fast functional paths. |
| Cached `result` persistence | Resolved (strict/minimal default) | Persist typed summary/reference by default; optional full payload in CAS. |
| Changed-input routing authority | Resolved (strict correctness) | Optimization hint only; non-authoritative for soundness. |
| Conflict commutativity exceptions | Resolved (strict default) | No commutativity exceptions in current phase. |
| Service codegen strategy | Resolved (done) | Strategy B implemented: generic interpreters over `ServiceOperationSpec` (SC1-SC3). |
| DSL as source of truth for services | Resolved (done) | `.dag` service definitions replace hand-written IR transport types (SC4-SC7). |
| Artifact dependency direction | Resolved (codegen → compilation) | Codegen outputs are compilation inputs; planner must not model compilation before codegen. See canonical model Section 17.2. |
| Two-phase compilation | Resolved (bootstrap + tool bins) | Bootstrap-safe binaries (codegen, ci) compiled without generated sources; tool binaries depend on codegen outputs. See canonical model Section 17.3. |
| Daggen status | Deferred | `needs_daggen()` returns false. Workflow DAGs remain hand-authored in Rust. Daggen is not folded into `codegen.ensure` in current phase. See canonical model Section 17.5. |
| SDLC pipeline architecture | Resolved | Issue-centric lifecycle with provider-agnostic types. DSL stubs: `sdlc.dag`, `issues.dag`, `design.dag`, types in `std/types.dag`. |

### Tonight Handoff Lanes (Open Work)

Use these lanes to assign workers with minimal overlap and clear stop conditions.

| Lane | Task IDs | Preconditions | Primary Files/Areas | Done When | Verify | Status |
|---|---|---|---|---|---|---|
| A: Resolver de-stringing | `P12` -> `P6` | none | `resolve.rs`, `daglang-lower`, runtime resolver/dispatch | no string-prefix op resolution; no deferred passthrough fallback | `cargo test --workspace`, resolver golden tests | **DONE** |
| B: Workflow planner core | `WF1` -> `WF2` -> `WF3` -> `WF4` -> `WF5` | `WF1-D`..`WF4-D` reviewed | `gunbc-dag` workflow schema/planner/ledger/executor | deterministic typed plan, claim-safe admission, key/rehydration correctness | `cargo test --workspace` | **DONE** |
| C: Workflow cutover/perf | `WF6` -> `WF7` -> `WF8` -> `WF9` | Lane B complete ✓ | workflow entrypoints + `Makefile` wrappers + CI wiring | `make ci`/`make test-all` use planner path with SLO telemetry | `make ci`, `make test-all`, CI dry run | **OPEN (unblocked)** |
| D: Modeling hardening graph/runtime | `M8` -> `M9` -> `M16` and `M10` -> `M11` | `-D` tasks approved | IR type DAG/system-model/transport + runtime resource/dry-run | metadata inertness, typed dependency markers, strict dry-run enforced | targeted model tests + `cargo test --workspace` | **DONE** |
| E: Security/install/process drift | `M7`, `M15`, `M17` -> `M18` -> `M19` | `-D` tasks approved | value redaction, installer model, proof harness | no accidental secret leak path; typed PM policy; invariants testable | test suites for each module + planner invariant suite | **DONE** |
| F: Universal capabilities | `WF14-D` -> `WF14` -> `WF15-D` -> `WF15` | Lane B complete ✓ | binary dispatch, codegen keyed unit, planner integration | compilation + codegen capabilities keyed and shared across all workflows | `gunbc-workflow --plan gist-snapshot` shows codegen CachedHit | **OPEN (design done, impl unblocked)** |
| G: Gist capability stack | `WF16-D` -> `WF16` -> (`WF17`, `WF18`) | Lane F complete | gist graph, gist_modes, credential chain, git state units | base gist workflow built; diff + recent augment base; all modes use planner path | `make gist` warm path, credential sharing across gist/dag-viz | **OPEN (design done, blocked on F)** |
| H: Remaining capabilities | `WF19-D` -> `WF19` -> `WF20` -> `WF21` -> `WF22` | Lane F complete | bootstrap/makegen/pragma/deps/dag-viz, Makefile | FS write + generator capabilities minimized; all tools on planner path with verification | per-capability hit/miss reporting, cross-workflow sharing observable | **OPEN (design done, blocked on F)** |
| I: Service codegen | `SC1` -> `SC2` -> `SC3` -> (`SC4`, `SC5`) -> `SC6` -> `SC7` | none | daglang-lower, resolve.rs, daglang-emit/*, service .dag files | 3 protocol interfaces replace all per-service Rust; all emission targets generate service code from DSL | `make gist --dry-run` uses generic interpreter | **DONE** |
| J: SDLC pipeline | `W9` -> `W10` -> `W11` -> `W12` -> `W13` | W1-W3 (credentials) | `dsl/pipelines/sdlc.dag`, `lib/ticket-ops/`, `lib/design-ops/` | issue-centric pipeline: post issue → design → review → implement → close | `gunbc sdlc --issue 42` runs full lifecycle | **OPEN (design done, blocked on W1-W3)** |

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
| **R2** | **[DONE 2026-02-20]** **Wildcard resource semantics deferred**: remove generated/injected `res:file:*` usage for now (coarsen to `res:file`), treat coarse `file` as conflicting with any specific `file:<path>` lock in admission control, and normalize wildcard IDs to coarse `file` in resource accounting. Track full glob semantics as design work in `backlog.md` before enabling pattern-aware admission control. | — | M |

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
| **P6** | **[DONE 2026-02-20]** `DeferredCallableOp` → per-module domain ops: implement `*Op` enums for each deferred module, replace catch-all passthrough with `Err(unknown_callable(...))`. Further consolidated via `domain_passthrough_op!` macro. | — | L | PR review |
| **P12** | **[DONE 2026-02-20]** Move `resolve_infrastructure()` string-prefix matching up to lowering: lowerer emits typed `LoweredOp` variants. Resolver is exhaustive enum match, not prefix scan. | — | M | PR review |

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
| **WF1-D** | **[DONE 2026-02-20]** **Workflow schema design spec**: `docs/design/workflow/wf1-wf4-dag-design-pack.md` (Sections 2-4). | — | S |
| **WF1** | **[DONE 2026-02-20]** **Minimum work-unit schema**: typed units over existing DAG primitives, no untyped shell fallback nodes. | WF1-D | M |
| **WF2-D** | **[DONE 2026-02-20]** **Mutual-exclusion/admission design spec**: `docs/design/workflow/wf1-wf4-dag-design-pack.md` (Section 5). | WF1-D | S |
| **WF2** | **[DONE 2026-02-20]** **Mutual-exclusion claim model**: conflicting units cannot co-run; fail-closed admission. | WF1, WF2-D | M |
| **WF3-D** | **[DONE 2026-02-20]** **Key/ledger causality design spec**: `docs/design/workflow/wf1-wf4-dag-design-pack.md` (Section 6). | WF1-D | S |
| **WF3** | **[DONE 2026-02-20]** **Deterministic materialization keys + miss reasons**: same repo state yields identical keys; cached-hit rehydration; fail-closed on rehydration errors. | WF1, WF3-D | M |
| **WF4-D** | **[DONE 2026-02-20]** **Downstream coordination design spec**: `docs/design/workflow/wf1-wf4-dag-design-pack.md` (Section 7). | WF1-D, WF2-D | S |
| **WF4** | **[DONE 2026-02-20]** **Downstream coordination contract**: planner proves topological + contract consistency; domain-failure results still reach report/aggregate nodes. | WF1, WF2, WF4-D | M |
| **WF5** | **[DONE 2026-02-20]** **Planner dry-run + execution plan explainability**: `gunbc-workflow --plan ci` produces deterministic node lists and miss reasons. | WF2, WF3, WF4 | S |
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
| **WF10-D** | **[DONE]** Control-token model: completion-gated control + explicit typed success guards. | WF4-D | S |
| **WF11-D** | **[DONE]** Cached `result` persistence: typed summary/reference mandatory, full payload optional. | WF3-D | S |
| **WF12-D** | **[DONE]** Changed-input routing: optimization hint only, never authoritative for correctness. | WF3-D | S |
| **WF13-D** | **[DONE]** Conflict commutativity: no exceptions, conflicting claims require explicit ordering. | WF2-D | S |

Additional active open items:

1. Resource wildcard pattern semantics remain explicitly deferred (`R2` + `backlog.md`).
2. Deferred-callable migration is implementation-open but design-resolved (`P6`, `P12`).
3. Bootstrap invariant has no CI enforcement gate. A compile-time or CI test should
   assert that bootstrap-safe binaries (`codegen.rs`, `ci.rs`) do not depend on
   generated artifacts (no `include!("generated_")` outside `#[cfg(test)]`). Scope: `WF14`.
4. `render_makefile()` has no planner-awareness parameter — it always wires
   `ensure-codegen` as a prerequisite, even when the planner manages codegen freshness.
   Scope: `WF21` (Makefile thinning).
5. `default_registry()` couples tool discovery to `gunbc_codegen::registry::derive_tool_defs()`.
   Post-planner, tool discovery should come from workflow specs. Low priority.
6. daglang CLI: `compile.rs` and `pipeline.rs` have two partially-overlapping pipeline
   implementations. Consolidate to one frontend API with commands as projections/views.
   Scope: next daglang iteration.
7. daglang CLI: `daglang manifest` currently outputs topology stats (nodes, edges, waves),
   not a progress manifest in the roadmap sense. Decide naming: rename to `daglang topology`
   or add a separate `daglang progress` command. Scope: next daglang iteration.
8. daglang CLI: canonical IR JSON (`makegen_canonical_ir.json`) is a test snapshot, not a
   stable CLI output. Promote to `daglang compile --format canonical-json` for diffable CI
   artifacts. Scope: next daglang iteration.
9. daglang CLI: `daglang viz` defaults to Mermaid output; roadmap expects ASCII default
   with `--format mermaid` as optional. Decide and update. Scope: next daglang iteration.

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
| **WF14-D** | **[DONE 2026-02-20]** **Compilation capability design spec**: two-phase compilation, bootstrap invariant, keying contract. **Design doc**: `docs/design/workflow/tool-workflow-design-pack.md` (Section 2). | WF1-D | S |
| **WF14** | **Compilation capability implementation**: implement binary freshness as a planner-managed keyed unit with two compilation phases. Make targets dispatch to pre-built binaries, bypassing `cargo run`. **Acceptance**: `make gist` no longer invokes `cargo run`; binary staleness is detected by planner key (source hash + cargo metadata + profile + target + features + RUSTFLAGS), not by Cargo's internal check; bootstrap phase compiles without codegen dependency; tool-binary phase depends on codegen outputs; one compilation unit per phase is shared across all tool workflows. | WF14-D, WF1 | M |
| **WF15-D** | **[DONE 2026-02-20]** **Codegen capability design spec**: codegen freshness as keyed unit, daggen deferred. **Design doc**: `docs/design/workflow/tool-workflow-design-pack.md` (Section 3). | WF3-D, WF14-D | S |
| **WF15** | **Codegen capability implementation**: implement codegen as a planner-managed keyed unit with ledger-backed freshness. Remove `ensure-codegen` as Make prerequisite for planner-managed targets. **Acceptance**: warm-state tool invocation does not spawn codegen subprocess; codegen staleness triggers re-run with typed miss reason; global ledger shares codegen freshness across `ci`, `gist`, `bootstrap`, etc.; **no planner-managed Make target has `ensure-codegen` as a prerequisite**; Makefile rendering no longer wires `ensure-codegen` dependency for planner-managed flows. | WF15-D, WF3 | M |

### Phase T-B: Gist Capability Stack (Base Workflow + Mode Augmentations)

The gist family has a shared base workflow (branch context, credentialing, upload)
and three mode-specific content acquisition augmentations. The base is built first;
modes compose on top. See design doc Section 15.4.

| ID | Task | Deps | Size |
|----|------|------|------|
| **WF16-D** | **[DONE 2026-02-20]** **Gist base + mode capability design spec**: base workflow + mode-specific augmentations. **Design doc**: `docs/design/workflow/tool-workflow-design-pack.md` (Section 4). | WF1-D, WF3-D, WF15-D | M |
| **WF16** | **Base gist workflow + snapshot mode**: implement the shared base gist capability units (codegen, compilation, `git.current_branch`, `credential.resolve`, `github.gist_create`) and the snapshot augmentation (`git.ls_files`, `fs.read_files`, `render_snapshot`). `gunbc-workflow gist-snapshot` composes base + snapshot. **Acceptance**: warm no-op: base + snapshot capability units resolve from ledger, only transport executes; base units are reusable by diff/recent modes; parity test validates output matches current `gunbc-gist` behavior. | WF16-D, WF5, WF14, WF15 | M |
| **WF17** | **Gist diff mode (augments base)**: implement `gunbc-workflow gist-diff` as base gist + diff-specific content acquisition (`git.diff`, `render_diff`). **Acceptance**: base units (branch context, credential, upload) shared with snapshot via global ledger — not re-implemented; key miss on `base_ref` or `HEAD` triggers minimal dirty closure (diff + render + transport only). | WF16, WF16-D | S |
| **WF18** | **Gist recent mode (augments base + cloud credential)**: implement `gunbc-workflow gist-recent` as base gist + recent-specific content acquisition (`git.rev_list`, per-commit `git.diff`, `render_recent`) + credential cloud override (`runtime_mode: Cloud` triggers WIF sub-chain). **Acceptance**: base units shared with snapshot/diff; credential sub-chain (STS exchange, IAM impersonation, Secret Manager) independently keyed with TTL; warm no-op with valid TTL + no new commits: zero capability units except transport; `runtime_mode` is typed input, not ambient env probe. | WF16, WF16-D | M |

### Phase T-C: Remaining Capability Ports (FS Write + Generator Workflows)

| ID | Task | Deps | Size |
|----|------|------|------|
| **WF19-D** | **[DONE 2026-02-20]** **Generator + remaining tool capability design spec**: bootstrap/makegen/pragma/deps/dag-viz decomposed. **Design doc**: `docs/design/workflow/tool-workflow-design-pack.md` (Sections 5-9). | WF1-D, WF3-D, WF15-D | M |
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
| **G: Gist capability stack** | `WF16-D`→`WF16`→(`WF17`,`WF18`) | base gist workflow built; diff + recent augment base; all modes use planner path |
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
| **M7-D** | **[DONE 2026-02-20]** **Secret redaction design spec**. | — | S |
| **M7** | **[DONE 2026-02-20]** **Secret redaction by default**: transport-boundary plaintext extraction enforced. | M7-D | M |
| **M8-D** | **[DONE 2026-02-20]** **`TypeOp::Meta` design spec**. | — | S |
| **M8** | **[DONE 2026-02-20]** **Semantically inert metadata op**: `TypeOp::Meta` landed; system-model metadata migrated. | M8-D | M |
| **M9-D** | **[DONE 2026-02-20]** **Typed dependency marker design spec**. | M8-D | S |
| **M9** | **[DONE 2026-02-20]** **Typed dependency markers**: string-prefix dependency semantics removed. | M8, M9-D | S |
| **M10-D** | **[DONE 2026-02-20]** **Resource declaration + auto-wiring design spec**. | WF2-D | M |
| **M10** | **[DONE 2026-02-20]** **Mandatory resource declarations + auto-wiring**: fail-closed for undeclared effectful I/O. | WF2, M10-D | L |
| **M11-D** | **[DONE 2026-02-20]** **Strict dry-run poisoning design spec**. | M10-D | S |
| **M11** | **[DONE 2026-02-20]** **Strict dry-run mode**: strict mode wired into CI/testgen/integration paths. | M10, M11-D | M |
| **M15-D** | **[DONE 2026-02-20]** **Typed package-manager design spec**. | — | S |
| **M15** | **[DONE 2026-02-20]** **Typed install planning**: fail-closed unknown PM handling. | M15-D | M |
| **M16-D** | **[DONE 2026-02-20]** **SystemModel/TransportBehavior unification design spec**. | M8-D, R8, R10 | M |
| **M16** | **[DONE 2026-02-20]** **SystemModel/TransportBehavior unification**: shared invocation contract model. | M8, M9, M16-D | M |
| **M17-D** | **[DONE 2026-02-20]** **Global flattening + context-free identity design spec**. | WF3-D, WF4-D | M |
| **M17** | **[DONE 2026-02-20]** **Global flattening + context-free identity**: equivalent work across workflows unified. | WF3, WF4, M17-D | L |
| **M18-D** | **[DONE 2026-02-20]** **Single semantic authority/projection design spec**. | M17-D | M |
| **M18** | **[DONE 2026-02-20]** **Projection-only surfaces + drift enforcement**. | M17, M18-D | M |
| **M19-D** | **[DONE 2026-02-20]** **Formal non-redundancy proof design spec**. | M17-D, M18-D | M |
| **M19** | **[DONE 2026-02-20]** **Formal non-redundancy proof harness**: CI gates over planner preflight + execution/ledger traces. | M17, M18, M19-D | M |

---

## Sprint 7: End-to-End Service Codegen from DSL

**Goal**: model 3 protocol interfaces (REST, Shell, File) bottom-up so that all services
are defined purely in DSL with zero per-service Rust. The interfaces generate native code
for all emission targets (Rust, Go, C, MIPS).

**Design reference**: `docs/design/service-codegen.md`.

### Design Decision (Resolved 2026-02-19): Bottom-Up Interface Modeling

Services are not individually coded. There are exactly 3 protocol interfaces:
- **REST**: endpoint + method + path template + JSON body/response + auth
- **Shell**: argv template + interpolation + stdout parsing
- **File**: path + read/write (already handled by `content_upsert` infrastructure)

Each interface is implemented once per target language. A `.dag` service definition
is just data parameterizing one of these interfaces. The resolver dispatches on
transport class, not service name.

### Design Decision (Resolved 2026-02-19): Multi-Language Emission

The `ServiceOperationSpec` is IR-level data, not Rust-specific. Each emission backend
(Rust exec-runtime, standalone Rust, Go, C, MIPS) reads the spec and generates native
protocol interface code. One `.dag` definition → all target languages.

### Phase SC-A: Spec Extraction + Protocol Interfaces

| ID | Task | Deps | Size |
|----|------|------|------|
| **SC1** | **[DONE 2026-02-20]** **`ServiceOperationSpec` in the IR**: `ServiceOperationSpec` enum with Rest/Shell/File variants. `ServiceCallMetadata` carries full spec. | — | M |
| **SC2** | **[DONE 2026-02-20]** **Generic protocol interpreters (Rust exec-runtime)**: `RestPrepareOp`, `RestParseOp`, `ShellPrepareOp`, `ShellParseOp` parameterized by spec. Parity tests against all 14 former hand-written adapters. | SC1 | M |
| **SC3** | **[DONE 2026-02-20]** **Switch resolver + delete per-service Rust**: transport-class dispatch. All per-service adapter structs deleted. | SC2 | M |

### Phase SC-B: LLM Services + Multi-Language Emission

| ID | Task | Deps | Size |
|----|------|------|------|
| **SC4** | **[DONE 2026-02-20]** **LLM provider service definitions**: `openai.dag`, `anthropic.dag` as standard REST services. LLM transport boilerplate deleted. | SC3 | M |
| **SC5** | **[DONE 2026-02-20]** **Multi-language service emission (Go)**: `ServiceOperationSpec` on prepare/parse nodes generates native Go `net/http` + `exec.Command`. | SC1 | M |
| **SC6** | **[DONE 2026-02-20]** **Multi-language service emission (C + MIPS)**: C generates libcurl/posix; MIPS generates syscall sequences. | SC1, SC5 | M |

### Phase SC-C: Validation + New Service Proof

| ID | Task | Deps | Size |
|----|------|------|------|
| **SC7** | **[DONE 2026-02-20]** **New service smoke test (all languages)**: `httpbin.dag` test service works in all 4 emission targets with zero hand-written code. | SC3, SC5, SC6 | S |

### Lane Summary

Lane I runs independently of Lanes B-H. SC1-SC3 are the critical path: spec extraction,
generic interpreters, and full resolver cutover. SC4 (LLM) can overlap with SC5-SC6
(multi-language). SC7 is the integration proof.

---

## Sprint 8: Issue-Centric SDLC Pipeline

**Goal**: make the repo feel alive — post an issue with an idea, and the infra picks it up,
generates a design, reviews it, tracks implementation, and closes it when done.

**Design reference**: `dsl/pipelines/sdlc.dag`, `dsl/services/github/issues.dag`,
`dsl/tools/design.dag`, SDLC types in `dsl/std/types.dag`.

### Design Decision (Resolved 2026-02-20): Provider-Agnostic Issue Lifecycle

The pipeline works against abstract types (`TrackedIssue`, `DesignOutput`,
`IssueLifecycleStage`, etc.) defined in `std/types.dag`. The concrete issue backend
(GitHub Issues, Linear, local blob store) is a transport swap — one import line change.
This mirrors the LLM pattern: `ChatRequest`/`ChatResponse` are abstract, `openai.dag`
and `anthropic.dag` are concrete bindings.

### Design Decision (Resolved 2026-02-20): Issues as State Machine

GitHub issue labels encode lifecycle stages:
`idea → design → design-review → accepted → implementing → code-review → testing → done`

Each stage transition:
1. Reads current issue state
2. Executes stage logic (LLM call, CI run, etc.)
3. Posts artifact as issue comment
4. Updates labels to next stage

### Phase W-A: Credentials + CLI (prerequisite — from Sprint 3)

| ID | Task | Deps | Size |
|----|------|------|------|
| **W1** | **`gunbc review` CLI binary**: binary entry point, credential resolution from env/policy, Real mode execution. Input: git diff. Output: structured findings JSON. | — | M |
| **W2** | **Credential smoke test**: run `gunbc review` locally with `ANTHROPIC_API_KEY`, verify full chain. | W1 | S |
| **W3** | **Multi-provider support**: verify `--provider openai` and `--provider anthropic` both work. | W2 | S |

### Phase W-B: Review Pipeline (from Sprint 3)

| ID | Task | Deps | Size |
|----|------|------|------|
| **W4** | **Abstract review DAG**: 4-dimension model (coherence, quality, requirements, aspirational). Dimensions opt-in via criteria. Depth controls cost/quality. | W3 | M |
| **W5** | **Coding review profile**: `ReviewProfile` for code — loads criteria from `AGENT.md` + `clippy.toml` + GitHub issue body. `gunbc review --pr <number>`. | W4 | M |
| **W6** | **CI status as review context**: `gunbc review --pr <number>` queries CI via `gh run list`. Inject failure context. Support `--depth XS|S|M|L|XL`. | W5 | S |

### Phase W-C: SDLC Pipeline

| ID | Task | Deps | Size |
|----|------|------|------|
| **W9** | **GitHub Issues transport**: `core/ir/src/transport/github/issues.rs` (request/response types) + `lib/ticket-ops/` (pure prepare/parse ops following `LlmOps` pattern). Provider-agnostic `TrackedIssue` adapter converts GitHub responses to abstract types. **Acceptance**: `Issues.Create`, `Get`, `Update`, `AddComment`, `SetLabels`, `List` work via generic REST interpreter; round-trip tests against mock responses. | W1 | M |
| **W10** | **DesignOps**: `lib/design-ops/` with `PrepareDesignPrompt` / `ParseDesignResponse`. System prompt produces structured markdown. Design review reuses W4's review pipeline with design-specific `ReviewProfile`. **Acceptance**: `generate_design(idea)` produces valid `DesignOutput`; `review_design(design)` produces `DesignReviewOutput` with findings. | W3, W4 | M |
| **W11** | **SDLC pipeline resolver**: wire `pipelines.sdlc` module into `resolve.rs`. Connect `issues.dag` service ops to generic REST interpreter. **Acceptance**: `gunbc sdlc --issue 42 --dry-run` resolves all ops; full stage chain works with mocked transport. | W9, W10 | M |
| **W12** | **`gunbc sdlc` CLI**: entrypoint that runs the SDLC pipeline for a given issue number. `gunbc sdlc --issue 42` fetches issue, runs stages based on current label, posts artifacts. **Acceptance**: end-to-end test with real GitHub issue + LLM call. | W11 | M |
| **W13** | **Approval gates in workflow planner**: extend `WorkflowOp` with `AwaitApproval`; extend ledger with approval state; extend admission control. Human override via label change or comment command. **Acceptance**: pipeline pauses at approval points; manual label change resumes; ledger records approval events. | W12, WF4 | L |

### Phase W-D: Orchestration + Monitoring

| ID | Task | Deps | Size |
|----|------|------|------|
| **W7** | **`gunbc pipeline` command**: orchestrates full daily flow for a branch/PR: fetch CI status → run 4-dimension review → output summary. | W6 | M |
| **W8** | **GitHub issue integration**: `gunbc pipeline --issue <number>` reads issue description as requirements input. Validates implementation matches intent. | W7 | S |
| **W14** | **Pipeline metrics + monitoring**: record stage durations, LLM costs, approval latency. Report ops extended with time-series. **Acceptance**: `gunbc sdlc --report` shows per-stage breakdown. | W12 | M |

### Daglang CLI Hardening (from external review feedback)

| ID | Task | Deps | Size |
|----|------|------|------|
| **DL1** | **Fix `normalize_path_components` root-clamping**: `ParentDir` past root should clamp (not empty). Leading `..` on relative paths should be preserved. | — | S |
| **DL2** | **Normalize diagnostics at Parse stop**: `run_pipeline()` normalizes at Build stop but not Parse — add normalization for deterministic output. | — | S |
| **DL3** | **Remove unused pipeline DAG/toposort**: `run_pipeline()` builds a DAG and computes topo-order but ignores both. Remove or use. | — | S |
| **DL4** | **Unify `.dag` directory behavior or explicit error**: compile mode treats `.dag` dirs as files; check mode walks them. Either unify or emit clear error. | — | S |

---

## Parallelization Guide (Open Work Only)

This is a dependency-accurate execution board for unfinished work. Use it to assign
multiple workers with minimal overlap.

> Naming note: `W9` (SDLC issues transport) and `WF9` (workflow latency SLO) are different tasks.

### A) Independent tasks (start anytime, parallel-safe)

| Group | Tasks | Notes |
|---|---|---|
| Wildcard semantics hardening | `R2` | **DONE** — wildcard ports normalized at construction, coarse conflict detection enforced. |
| Daglang CLI hardening | `DL1`, `DL2`, `DL3`, `DL4` | Independent quick fixes; can run in parallel with all lanes. |

### B) Dev Pipeline + SDLC track (`W*`)

| Wave | Parallel Tasks | Depends On | Unblocks |
|---|---|---|---|
| W-0 | `W1` | — | `W2`, `W9` |
| W-1 | `W2`, `W9` | `W1` | `W3`; early SDLC transport |
| W-2 | `W3` | `W2` | `W4`, `W10` (partially) |
| W-3 | `W4` | `W3` | `W5`, `W10` |
| W-4 | `W5`, `W10` | `W4` (+ `W3` for `W10`) | `W6`, `W11` |
| W-5 | `W6`, `W11` | `W5` and (`W9` + `W10`) | `W7`, `W12` |
| W-6 | `W7`, `W12` | `W6`, `W11` | `W8`, `W13`, `W14` |
| W-7 | `W8`, `W13`, `W14` | `W7` (for `W8`), `W12` (for `W13`,`W14`) | End-to-end daily + SDLC flow |

### C) Workflow planner/tool minimization track (`WF*`)

| Wave | Parallel Tasks | Depends On | Unblocks |
|---|---|---|---|
| WF-0 | `WF6`, `WF7`, `WF14`, `WF15` | done prereqs (`WF1`-`WF5`, design tasks) | planner cutover + universal capability base |
| WF-1 | `WF8`, `WF9`, `WF16`, `WF19`, `WF20` | `WF6`+`WF7` (for `WF8`,`WF9`); `WF14`+`WF15` (for `WF16`,`WF19`,`WF20`) | gist modes + remaining tools + SLO base |
| WF-2 | `WF17`, `WF18` | `WF16` | full gist mode family |
| WF-3 | `WF21` | `WF16`,`WF17`,`WF18`,`WF19`,`WF20` | global tool target cutover |
| WF-4 | `WF22` | `WF21`,`WF9` | capability-level verification complete |

### D) Suggested lane ownership (to reduce merge conflicts)

| Lane | First Task | Then |
|---|---|---|
| Lane 1: Review/SDLC core | `W1` | Follow `W-1` to `W-7` |
| Lane 2: Workflow cutover | `WF6`, `WF7` | `WF8`, `WF9` |
| Lane 3: Universal capabilities | `WF14` | `WF15`, then feed `WF16/19/20` |
| Lane 4: Gist modes | `WF16` | `WF17`, `WF18` |
| Lane 5: Remaining tool capabilities | `WF19`, `WF20` | `WF21`, `WF22` |
| Lane 6: Daglang hardening | `DL1`-`DL4` | finish independently |
| Lane 7: Resource semantics | `R2` | **DONE** |

### Priority

1. **Critical path now**: `W1` and `WF14` (they unlock the most downstream work).
2. In parallel: `WF6`/`WF7`, `DL1`-`DL4`.
3. Keep `W*` and `WF*` in separate owner lanes to avoid cross-track churn.

**Backlog**: XL features and migration work in `backlog.md`.
