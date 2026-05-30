# v4 CI Overhaul: Minimal-for-Highest-Confidence per PR via Affected-Set Lens

> **Status:** SCOPING DRAFT — operator sign-off requested on §8 before dispatch.
> **Date:** 2026-05-30
> **Author:** PM May 29 (session `nimble-dove-733`)
> **Trigger:** Operator 2026-05-30 (post-PR #3938 merge): *"CI is taking 30 minutes + right now — it's effectively broken — I basically need to rip it apart and reverse to figure out what I actually need/want from it — on every CI run, I want the absolutely minimal set of functionality that gives us the highest confidence that these specific changes are safe — I envisioned the affected-set lens for this, but we aren't able to actually get there for some reason."*

This doc is a focused scoping artifact for the CI overhaul. Builds on the already-ratified T-24 phase plan; does not redesign it. Identifies one addition (Phase 1.5), diagnoses the blockers, maps work to the §11 manager-lane architecture from PR #3938, and surfaces 5 operator decisions.

---

## §1. Provocation

CI takes 30+ minutes per PR. 91 step entries in 605 lines of hand-authored YAML. The substrate exists (ci.dag 1308 lines; affected_set.dag 1251 lines) but the activation hasn't delivered minimal CI. Operator wants: per-PR minimal step set that maximizes confidence for the specific changes — affected-set-lens-driven.

The operator's specific phrasing — *"we aren't able to actually get there for some reason"* — has a concrete answer (§4). The framework is already ratified; this doc is about unblocking progression.

---

## §2. Current state audit

| Artifact | State | Source |
|----------|-------|--------|
| `.github/workflows/ci.yml` | 605 lines, 91 step entries, hand-authored, runs every PR | `wc -l` + `grep -c "^    - name:"` |
| `src/v4/workflow/ci.dag` | 1308 lines substrate (CiPipeline, CiJob, CiGate, CiCommand, CiComponentAffected, SelfHostedRunnerPool, LensCiLiveWorkflowSignal, M1CiLiveWorkflowSignal) | grep + read |
| `src/v4/lens/affected_set.dag` | 1251 lines substrate (T-21) | grep + read |
| `dsl/gunbc/ci_github_actions_workflow.dag` | generated-from-ci.yml (`@generated` per INVARIANTS) | INVARIANTS P2 |
| `tools/gen_gunbc_ci_workflow_dag` | the generator | INVARIANTS P2 task-scope-drift |

**ci.dag's own header (verbatim, src/v4/workflow/ci.dag:4):**
> *"T-24 CI/YAML authority bridge (emitter wired, hand-authored ci.yml deleted) remains open per src/v4/TASKS.md §T-24 — interim hand ci.yml edits are transport wiring only, not YAML-authority dissolution."*

Diagnosis-ready: substrate-rich, activation-poor, with the bridge condition explicitly named.

---

## §3. The design is already ratified

Per `TASKS.md §T-24` ("CI overhaul close predicates (operator-ratified 2026-05-29, `docs/design-ci-dag-overhaul.md` PR #3886)"):

**Phase 1a (Tier-0 integrity)**: ci.dag is sole *policy* authority for integrity-class CI (I0–I8 in bankruptcy doc); GHA invokes T-22 interpreter on `ci_pipeline` (S2′); coarse bucket `if:` scheduling and monolithic policy jobs (`v3`/`v4`/`self_host_ratchet` as schedule drivers) are dissolved. Atoms A0–A2 (+ integrity arms). **T-24 remains open** after Phase 1a.

**Phase 1b (lane completion)**: atoms A3–A14 promoted opt-in (one PR each); A6–A8 delete `scripts/check-*` in the same PR as `DisciplinePolicyCommand` / `TestClaim` ports.

**Phase 2 (A15)**: Shape-B checked `.github/workflows/ci.yml` emitted from `CiPipeline`; **all** hand-authored workflow YAML deleted (C4 / `design-pure-bootstrap-zero.md`). **T-24 [DONE]** only after Phase 2.

**Forbidden** (per TASKS.md §T-24): "treating a hand-maintained harness `ci.yml` as steady-state authority; silently narrowing this bullet to 'interpreter-only' without a TASKS amendment."

So the framework exists. The scoping question is: **what's blocking progression through Phase 1a → 1b → 2, and what extension does affected-set-driven minimal-CI need beyond the ratified plan?**

---

## §4. Why "minimal-for-highest-confidence" hasn't been delivered

Three blockers compound:

**B1. Hand-authored ci.yml is the de-facto authority.** Every CI edit goes to ci.yml directly, not through ci.dag. So ci.dag is documentation-of-intent, not the running source. Even though the emitter exists (per the ci.dag header), no PR uses it as the authority. The "interim hand ci.yml edits are transport wiring only" admission means *every CI change is transport-wiring* — no edit is structural.

**B2. Affected-set lens substrate exists but doesn't gate step selection.** `lens/affected_set.dag` (1251 lines) projects affected components, but ci.yml's `if:` conditions are bucket-coarse (per `docs/design-ci-bankruptcy-rebuild.md`) — they don't read the lens's per-step output. So "minimal" is impossible by construction: every step runs unless its coarse bucket is excluded.

**B3. No per-step dependency-set declaration.** `ci.dag` has `CiJob` and `CiCommand` types but each *step* doesn't declare *"these are my input file globs / substrate node sets"* — so even if affected-set lens output were consumed, there's no granular per-step gate to apply it to. The lens produces per-component output; per-step is a finer granularity.

**Compound effect:** substrate-rich (ci.dag + affected_set.dag), activation-poor (hand-authored ci.yml + coarse `if:` + no per-step dependency declaration). All three must close before minimal-CI delivers.

---

## §5. Target architecture (three layers, Upsert<T>-shaped)

**The primitive is Upsert<T>**, not generic step gating. Per the operator directive 2026-05-29 (canonical pattern in `dsl/std/patterns.dag` UPSERT<T> section + `docs/audit/upsert-pattern-compiler-stray-2026-05-29.md`): **"do this" = "upsert this"** — never blind create, never blind overwrite. The 4 phases are fractal at every scale: **verify-first → satisfy-dependencies-recursively → create-if-missing → cache-outcome**.

Upsert<T> is what unifies ergonomics + mechanical efficiency in one shape: the developer writes one `upsert<Check, Create, Resolve>`; the framework handles verify, dep-resolution, cache, and action. The verify-first phase is exactly where affected-set fires.

**Layer A — Authority.** `workflow/ci.dag` is the sole authority. `.github/workflows/ci.yml` is the generated emit (Shape-B per THESIS). Hand-edits to `ci.yml` are FORBIDDEN per T-24 Phase 2.

**Layer B — Each CI step is an Upsert<T> Node.** `CiCommand` becomes an Upsert<T> specialization (per the `ensure<Check, Action>` / `upsert<Check, Create, Resolve>` / `content_upsert` specializations in the patterns canon). The step declares:
```dag
type CiUpsertStep<T> = Upsert<T> {
  inputs: List<UpsertInputRef>          // what facts the verify-first phase reads
  verify: VerifyCheck                    // is desired state already satisfied?
  create: CreateAction                   // action to take if verify says action needed
  resolve: ResolveExpr                   // stable handle / value to return
  cache_key: ContentHashKey              // structural cache key derived from inputs
}

type UpsertInputRef
  = FileGlob { glob: Symbol }            // "src/v4/std/**/*.dag"
  | SubstrateNodeSet { selector: NodeQuery }
  | LensOutputRef { lens: LensId, ports: List<Port> }
  | TestClaimRef { claim_id: Symbol }
  | UpstreamUpsert { step_id: Symbol }   // dependency-resolution: upstream upsert must succeed first
  | Always                                // unconditional (integrity-class)
```
CI generation MUST reject any step that isn't an Upsert<T> Node (fail-closed per INVARIANTS P3). This eliminates the "blind run every step" failure mode by construction.

**Layer C — Affected-set drives the verify-first phase.** `lens/affected_set.dag` projects the per-PR change set into the canonical `AffectedSet` type from `std/change.dag`. The verify-first phase of each Upsert<T> step is:
```
verify(step, PR) =
  if intersect(step.inputs, affected_set(PR)) = ∅
    then satisfied   // short-circuit: cached outcome stable, no action
    else needs_action  // verify says action required → create phase runs
```

**Minimal CI per PR** falls out of Upsert<T> semantics. By construction:
- `Always` (integrity-class) upserts run (their inputs are never satisfied-by-affected-set).
- All other upserts short-circuit to cached when affected_set doesn't intersect — no action, no compute, no time.
- Dependency resolution is recursive per UPSERT canon: if step X depends on step Y, X's verify-first triggers Y's verify-first.

The 30-minute CI dissolves: every step that doesn't need to run, doesn't run, and the framework knows because every step is an Upsert<T> with a verify-first phase that reads affected_set.

---

## §6. Sequencing

Per the ratified T-24 phase plan, with ONE addition (Phase 1.5):

| Phase | Scope | T-24 status after |
|-------|-------|-------------------|
| **1a** (already in scope) | ci.dag sole policy authority for I0–I8 integrity; T-22 interpreter on ci_pipeline; coarse bucket `if:` dissolved | OPEN |
| **1.5** (**NEW** — needed for affected-set-driven minimal-CI; **Upsert<T>-shaped per operator directive 2026-05-29**) | Every CI step becomes an Upsert<T> Node (`CiUpsertStep<T>`) with `inputs`/`verify`/`create`/`resolve`/`cache_key` fields; CI generation rejects any step not Upsert<T>-shaped. Existing-shell retirement under clever-cat-115 per `project_no_new_shell` directive. | OPEN |
| **1b** | Atoms A3–A14 promoted opt-in; A6–A8 delete `scripts/check-*` | OPEN |
| **2** (A15) | Shape-B `ci.yml` emitted from CiPipeline; all hand-authored YAML deleted (C4) | **[DONE]** |
| **2.5** (NEW — minimal-CI activation) | Each step's gate consumes Layer C's intersection predicate; minimal CI fires per PR | **[DONE+]** |

**Rationale for Phase 1.5 BEFORE Phase 1b (Upsert<T> shape):** without per-step Upsert<T> declaration, A3–A14 atoms can't be ported individually without losing their CI gate semantics. Declaring each step as an Upsert<T> Node is a prerequisite for the atom-by-atom migration AND for affected-set-driven minimal CI. Putting Phase 1.5 between 1a and 1b is the cheapest sequencing.

**Why Upsert<T> over generic dependency_set:**
- *Ergonomics*: developer writes one `upsert<Check, Create, Resolve>` per step; framework handles verify, cache, dep-resolution.
- *Mechanical efficiency*: verify-first phase reads affected_set; short-circuits to cached when inputs not touched.
- *Single authority*: the Upsert<T> canon is in `dsl/std/patterns.dag` UPSERT<T> section — no parallel "CI dependency declaration" carrier (would be P2 violation).
- *Fractal*: the same pattern that scaffolds the compiler internals scaffolds CI steps (per the operator's "do this = upsert this" framing).
- *Coordination with clever-cat-115's existing-shell retirement*: ports each shell script to a `content_upsert` or `ensure<Check, Action>` row instead of inventing new types.

**Rationale for Phase 2.5:** Phase 2's T-24 [DONE] gate only requires ci.yml-emitted-from-ci.dag — it does NOT require affected-set-driven minimal-CI. Phase 2.5 is the operator's "minimal-for-highest-confidence" ask. Should it be IN T-24 or POST?

---

## §7. Manager ownership

| Concern | Primary owner | Secondary |
|---------|--------------|-----------|
| Phase 1a (T-22 interpreter on ci_pipeline; integrity-class) | **Compiler Spine** (smart-stag-871) | Close/Receipt (verdicts) |
| Phase 1.5 (per-CiCommand dependency_set declaration) | **Modeling DFS** (proud-pike-680) — substrate decision needs DFS worksheet | Compiler Spine (consumer) |
| Phase 1b (atom-by-atom migration) | **Compiler Spine** | Close/Receipt (atom dispositions) |
| Phase 2 (Shape-B YAML emission) | **Compiler Spine** | Self-host/Release (T-24 [DONE] is a v4-done predicate) |
| Phase 2.5 (affected-set intersection gate) | **Compiler Spine** + **Ladder/Fixture** | — |

Does NOT need a new manager lane — fits within the existing §11 architecture from PR #3938.

**Critical DFS gate (Modeling DFS Manager):** Phase 1.5's `DependencySource` type is substrate work. A DFS worksheet is REQUIRED before workers touch it. Spot-fix risk: workers could add `dependency_set: List<Symbol>` (string-keyed file globs) and miss the structural authority. The worksheet must establish that `DependencySource` is a *typed* carrier consuming `NodeQuery` / `LensOutputRef` etc., not a stringly-typed list.

---

## §8. Open questions for operator

**D-CI-1.** Accept the 3-layer target (§5) as the right shape, OR redirect to a different architecture?

*Proposed: accept.* Aligns with THESIS §"Two shapes of omni-emission" (ci.yml is Shape-B emit), INVARIANTS P2 single-authority, T-24 ratified phases.

**D-CI-2.** Accept the **Phase 1.5 addition** (**every CI step becomes an Upsert<T> Node** per operator directive 2026-05-29 + dsl/std/patterns.dag UPSERT<T> canon) to the ratified T-24 plan?

*Proposed: accept.* Without per-step Upsert<T>, affected-set lens can't project minimal CI. The ratified plan implicitly assumes some step-shape but doesn't name Upsert<T> as the unit. Adopting Upsert<T> aligns with the existing canon (no new vocabulary) and coordinates with clever-cat-115's existing-shell retirement (each shell script ports to a `content_upsert` or `ensure<Check, Action>` row).

**D-CI-3.** Confirm **Compiler Spine + Modeling DFS** as the primary owner pair for this overhaul?

*Proposed: yes.* CI is workflow-stage work (Compiler Spine); per-step dependency declaration is substrate work (Modeling DFS). No new manager lane needed.

**D-CI-4.** Dispatch priority — start with **Phase 1a** (T-22 interpreter on ci_pipeline) OR **Phase 1.5** (dependency declaration) first?

*Proposed: Phase 1a first* (already in scope per T-24). Phase 1.5 dispatch can start in parallel once the DFS worksheet is approved by Modeling DFS Manager, since the substrate work is independent of Phase 1a's interpreter wiring.

**D-CI-5.** Scope of "minimal for highest confidence":
- (a) Strictly affected-set-intersect (every step runs iff its deps are touched; integrity-class is `Always`)
- (b) Affected-set-intersect PLUS confidence-boost extras (e.g., always-run smoke gates regardless of touched files)
- (c) Affected-set-intersect with operator-tunable confidence-floor (some lanes can override "minimal" to always-run during sensitive periods)

*Proposed: (a) by default, with explicit `Always` variant in `DependencySource` for integrity-class steps.* (c) can be layered later by adding a `RunPolicy` field if needed; doesn't change the core architecture.

**D-CI-6.** Should Phase 2.5 (affected-set intersection gate firing) be **part of T-24 [DONE]** or **post-T-24** (separate close gate)?

*Proposed: part of T-24.* The operator's ask is explicit that minimal-CI is the goal; T-24 closing without minimal-CI active leaves the operator's stated requirement unmet. Recommend amending T-24 close-predicate to include Phase 2.5.

---

## §9. Sub-questions worth surfacing (not blocking sign-off)

- **CI today runs `scripts/check-*`** that aren't yet typed `DisciplinePolicyCommand`. The bankruptcy doc plans A6–A8 to delete these in Phase 1b. Are there checks the operator wants preserved at integrity-class (`Always`) even after their script form is deleted?
- **Self-hosted runners** (srv1/srv2 per `SelfHostedRunnerPool`): should runner allocation also be affected-set-driven (e.g., smaller affected-set → cheaper runner)? Out of scope for first pass; flag for later.
- **CI determinism** (per memory `project_determinism_as_effect`): determinism gates are orthogonal — they apply to each step regardless of affected-set. Phase 1.5's `DependencySource = Always` handles this.
- **CI per-step timeouts**: per memory item from forwarded info, T-22 TestClaim corpus timeout (exit 143 SIGTERM at 240s) is a pre-existing infra blocker. Phase 1.5's modeling could include per-step `timeout: Duration` declaration to make this declarative, not script-bound.

---

## §10. What this doc is NOT

- **Not a redesign of the T-24 phase plan.** That's ratified (2026-05-29). This doc identifies Phase 1.5 + 2.5 additions and surfaces blockers.
- **Not a complete implementation plan.** Operator sign-off on §8 unblocks the first manager dispatch (Compiler Spine for Phase 1a; Modeling DFS for Phase 1.5).
- **Not a substitute for `docs/design-ci-dag-overhaul.md` PR #3886.** That's the design canvas. This doc is the operationalization scoping.
- **Not a critique of the current CI work.** The substrate-rich state of `ci.dag` and `affected_set.dag` is genuine progress — without it, none of the §5 architecture would be authorable.

---

## §11. Related artifacts

- `src/v4/TASKS.md §T-24` — the ratified phase plan (Phase 1a / 1b / 2)
- **`dsl/std/patterns.dag` UPSERT<T> section** — the canonical Upsert<T> vocabulary (operator-ratified 2026-05-29): verify-first / satisfy-dependencies-recursively / create-if-missing / cache-outcome
- **`docs/audit/upsert-pattern-compiler-stray-2026-05-29.md`** — the canon doc + stray-audit
- **`docs/audit/v4-upsert-stray-scan-receipt-2026-05-30.md`** — the v4 stray scan receipt
- **clever-cat-115** — owns existing-shell-in-CI retirement per `project_no_new_shell` directive; coordinates with Phase 1.5
- `docs/design-ci-dag-overhaul.md` (#3886) — the design canvas
- `docs/design-ci-bankruptcy-rebuild.md` — the Tier-0 rebuild + bucket dissolution
- `docs/audit/ci-anatomy-and-redundancy-2026-05-29.md` — current CI shape vs target
- `docs/audit/ci-warm-cache-wall-measurement-2026-05-29.md` — empirical wall-time (12m31s v3 job warm-cache)
- `docs/planning/v4-correctness-ladder-2026-05-30.md` §11 — manager-lane architecture this doc maps onto
- `src/v4/workflow/ci.dag` — current substrate (1308 lines)
- `src/v4/lens/affected_set.dag` — current substrate (1251 lines, T-21)
- `.github/workflows/ci.yml` — current hand-authored YAML (605 lines, 91 steps; the thing to dissolve)
- `dsl/gunbc/ci_github_actions_workflow.dag` — @generated artifact; tools/gen_gunbc_ci_workflow_dag is the generator
