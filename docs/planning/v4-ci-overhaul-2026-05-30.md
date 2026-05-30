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

**Layer B — Each CI step is an Upsert<T> Node.** `CiCommand` becomes an Upsert<T> specialization (per the `ensure<Check, Action>` / `upsert<Check, Create, Resolve>` / `content_upsert` specializations in the patterns canon).

**Substrate-landing note (codex review 2026-05-30):** Upsert<T> today is the operator-ratified pattern canon in `dsl/std/patterns.dag:15` (UPSERT<T> section header) with **commented-out pattern bodies** at ~lines 127–156 — blocked on pattern-declaration generics per ROADMAP desired parser features. **The Upsert<T> type is NOT yet a usable substrate primitive.** **Phase 1.4** (separate phase per §6 sequencing) lands Upsert<T> as a proper substrate type with whatever parser/substrate prerequisites it needs. **Phase 1.5** specializes Upsert<T> into `CiUpsertStep<T>` and DEPENDS on Phase 1.4 completing. Modeling DFS Manager owns BOTH Phase 1.4 (substrate-extension worksheet) and the Phase 1.5 DFS worksheet.

Once landed, the step shape:
```dag
// Phase 1.4 substrate landing (prerequisite): Upsert<T> usable as type
//   per dsl/std/patterns.dag UPSERT<T> canon + parser support
// Then CiUpsertStep<T> specializes it:
type CiUpsertStep<T> = Upsert<T> {
  inputs: List<UpsertInputRef>          // what facts the verify-first phase reads
  verify: VerifyCheck                    // is desired state already satisfied?
  create: CreateAction                   // action to take if verify says action needed
  resolve: ResolveExpr                   // stable handle / value to return
  // NOTE: no `cache_key` field. The cache key is DERIVED — `content_hash(CiUpsertStep<T>)`
  // computed by the framework (Merkle catamorphism per modeling-discipline.md
  // Practice 10), not authored on the row. Authoring `cache_key` as a payload field
  // would admit stale `cache_key != content_hash(subgraph)` states (P2 single-authority
  // violation + Practice 11 parallel-payload). Consumers project `content_hash` from
  // the complete subgraph at emission / cache-lookup time.
}

type FileSetSelector {
  root: RepositoryRoot                   // canonical root (not raw path string)
  pattern: GlobPattern                   // typed glob, not bare Symbol
}

type UpsertInputRef
  = FileSet { selector: FileSetSelector }   // typed path-set, NOT bare Symbol glob
                                              // (raw globs at GitHub ingress; immediately
                                              //  normalized into FileSetSelector inside the model)
  | SubstrateNodeSet { selector: NodeQuery }
  | LensOutputRef { lens: LensId, ports: List<Port> }
  | TestClaimRef { claim_id: Symbol }
  | UpstreamUpsert { step_id: CiStepId }     // typed step identity, NOT Symbol
```

`CiStepId` is the typed step-identity carrier (defined elsewhere in workflow/ci.dag). Step identity is never a raw atom.

**Every step is AffectedOnly by construction** — there is NO per-step `Always` policy variant. Per operator directive 2026-05-30 ("I'm really not a fan of introducing heuristics this early on — this is a closed system; if we're missing something, I'd rather add it now instead of adding these heuristics/coproducts"): no `CiRunPolicy` / `CiRunMode` enum. The closed-system reasoning (INVARIANTS P1) says heuristics are recoverable to missing structural facts.

**Carve-out list (NOT a mode/heuristic).** A small explicit data declaration enumerates steps that always run, each with a literal reason (including "superstition? not sure why exactly?" as a valid honest entry):
```dag
type CiCarveout {
  step_id: CiStepId
  reason_code: Symbol                         // stable machine reason (e.g., v2_substrate_circular_dep)
  reason_detail: String                       // operator-readable prose
                                                // (per operator review 2026-05-30: bare Symbol is fine for
                                                //  stable machine keys, NOT for prose explanations like
                                                //  "superstition? not sure why exactly")
  dissolution_target: DissolutionTarget       // explicit path to remove this entry
                                                // (what substrate fact, when modeled, removes the need for the carveout)
}

type DissolutionTarget
  = ModelMissingSubstrate { what: Symbol }    // "affected-set lens circular-dep on v2" etc.
  | UnknownYet { investigation_owner: ManagerSessionId, review_due_by: Symbol }
                                                // we genuinely don't know what would dissolve it,
                                                // BUT (per operator review 2026-05-30) UnknownYet
                                                // cannot be permanent: requires a named investigation
                                                // owner (manager) + a review cadence. Without these,
                                                // UnknownYet becomes the new "Always because vibes."

data ci_always_run_carveouts: List<CiCarveout> = [
  { step_id: ci_step_v2_compile_src_v4,
    reason_code: v2_substrate_circular_dep,
    reason_detail: "v2-compiler integrity gate; affected-set lens itself imports v2 substrate — circular dependency unmodeled",
    dissolution_target: ModelMissingSubstrate { what: "v2_substrate_dependency_modeled_in_affected_set" }
  },
  { step_id: ci_step_<integrity_X>,
    reason_code: unmodeled_dependency,
    reason_detail: "superstition — not sure why exactly",
    dissolution_target: UnknownYet { investigation_owner: <manager_session>, review_due_by: "next_quarterly_review" }
  },
  // ... small list — every entry has reason_code + reason_detail + dissolution_target
]
```

Adding a carveout requires reason_code + reason_detail + dissolution_target. `UnknownYet` is a valid honest answer for ignorance, BUT it is not free — it requires a named `investigation_owner` (a manager session-id that takes responsibility for the unknown) AND a `review_due_by` (forcing a review cadence). Each carveout entry is reviewed on its dissolution_target progress. This is the "hard dispatch rule" per operator review 2026-05-30: every UnknownYet entry has a named owner and a recurring review trigger; permanent-ignorance carveouts are not allowed.

Each carve-out entry is a **dissolution target**: as we figure out the actual missing dependency, we model it (add the substrate fact), and the entry comes off the list. The list itself is data — small, honest, reviewable. Adding an entry is a deliberate decision with a reason; removing one is a substrate-modeling deliverable.

CI generation MUST reject any step that isn't an Upsert<T> Node (fail-closed per INVARIANTS P3). Step selection rule:
```
step_runs ⟺ step ∈ ci_always_run_carveouts
           OR ∩(step.inputs, affected_set(PR)) ≠ ∅
```
This eliminates both "blind run every step" and "policy-variant heuristic enumeration" by construction.

**Cache-key boundary (T-21 alignment):** the cache key for a `CiUpsertStep<T>` is the content-hash of the COMPLETE step subgraph — same discipline as T-21's TestClaim cache-key authority (`test_claim_interpretation_cache_digest`, `inferred_tree_digest`). Hashing only inputs would lose the verify/create/resolve identity and let two structurally-different steps with the same inputs share a cache entry — a P2 violation in cache scope. The B1 content_hash of the complete subgraph is the existing pattern.

**Layer C — Affected-set drives the verify-first phase.** `lens/affected_set.dag` projects the per-PR change set into the canonical `AffectedSet` type from `std/change.dag`. The verify-first phase of each Upsert<T> step is:
```
verify(step, PR) =
  if intersect(step.inputs, affected_set(PR)) = ∅
    then satisfied   // short-circuit: cached outcome stable, no action
    else needs_action  // verify says action required → create phase runs
```

**First observable artifact — `CiSelectionReceipt`** (operator-ratified 2026-05-30 as "the actual results ASAP" surface). Each PR produces:

```dag
type CiSelectionReceipt {
  pr: ChangeSet
  affected: AffectedSet
  selected: List<CiStepSelection>
  skipped: List<CiStepSelection>
  carved_out: List<CiCarveout>             // the always-run override list, with reasons + dissolution targets
}

type CiStepSelection {
  step_id: CiStepId
  inputs_consulted: List<UpsertInputRef>
  affected_intersection: List<AffectedNode>   // what specifically intersected
  decision: SelectionDecision                  // Run | Skip | CarvedOut
  cache_digest: ContentHash                    // projected from complete step subgraph
  reason: Symbol
}

type SelectionDecision
  = Run
  | Skip                                       // dep intersection empty
  | CarvedOut { carveout_reason: Symbol }     // matched ci_always_run_carveouts
```

The receipt is the **first thing CI should produce, before active skipping is trusted**. Even in shadow mode (where the receipt is computed but the existing CI still runs everything), the receipt tells operator + managers whether the dependency machinery is producing useful output. The transition from shadow → active is: once receipts are stable and reviewable, gate the existing workflow on the receipt's `selected` list.

**Minimal CI per PR** falls out of Upsert<T> semantics. By construction:
- Steps in `ci_always_run_carveouts` always run (carve-out override).
- All other upserts short-circuit to cached when affected_set doesn't intersect — no action, no compute, no time.
- Dependency resolution is recursive per UPSERT canon: if step X depends on step Y, X's verify-first triggers Y's verify-first.

**Structural success criterion** (operator review 2026-05-30: avoid making elapsed runtime the primary acceptance metric): the receipt is correct iff every step's `decision` is justified by either (a) `inputs ∩ affected_set ≠ ∅` evidence (selected), (b) `inputs ∩ affected_set = ∅` evidence plus valid cache digest (skipped safely), or (c) explicit `ci_always_run_carveouts` match (carved out). Wall-clock reduction is a downstream consequence, not the gate. Elapsed-time wins are visible but secondary to "every decision is justified by structural facts."

---

## §6. Sequencing

Per the ratified T-24 phase plan, with THREE additions (Phase 1.4 prerequisite, Phase 1.5 main, Phase 2.5 minimal-CI activation):

| Phase | Scope | T-24 status after |
|-------|-------|-------------------|
| **1a** (already in scope) | ci.dag sole policy authority for I0–I8 integrity; T-22 interpreter on ci_pipeline; coarse bucket `if:` dissolved | OPEN |
| **1.4** (**NEW** prerequisite — substrate-extension scope) | **Land Upsert<T> as usable substrate primitive in `dsl/std/patterns.dag`** (currently header + commented stubs per upsert-pattern audit; blocked on parser-declaration generics per ROADMAP). Modeling DFS worksheet must cover the parser/substrate prerequisites. | n/a (substrate landing) |
| **1.5** (**NEW** — needed for affected-set-driven minimal-CI; **Upsert<T>-shaped per operator directive 2026-05-29**, DEPENDS on Phase 1.4) | Every CI step becomes an Upsert<T> Node (`CiUpsertStep<T>`) with `inputs` / `verify` / `create` / `resolve` fields. Cache key is **derived** as `content_hash(CiUpsertStep<T>)` per T-24 / B1 discipline (Merkle catamorphism, not a payload field). `inputs: List<UpsertInputRef>` is the typed carrier (FileSet / SubstrateNodeSet / LensOutputRef / TestClaimRef / UpstreamUpsert) — **no `Always` variant**; always-run steps are listed in `ci_always_run_carveouts` with explicit reasons. CI generation rejects any step not Upsert<T>-shaped. Existing-shell retirement under clever-cat-115 per `project_no_new_shell` directive. | OPEN |
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
| **Phase 1.4** (land Upsert<T> as usable substrate primitive in `dsl/std/patterns.dag`) | **Modeling DFS** (proud-pike-680) — substrate-extension worksheet covers parser/substrate prerequisites | (none yet — substrate landing) |
| Phase 1.5 (`CiUpsertStep<T>` + `UpsertInputRef` substrate; each CI step becomes an Upsert<T> Node; **DEPENDS on Phase 1.4**) | **Modeling DFS** (proud-pike-680) — substrate decision needs DFS worksheet | Compiler Spine (consumer) |
| Phase 1b (atom-by-atom migration) | **Compiler Spine** | Close/Receipt (atom dispositions) |
| Phase 2 (Shape-B YAML emission) | **Compiler Spine** | Self-host/Release (T-24 [DONE] is a v4-done predicate) |
| Phase 2.5 (affected-set intersection gate) | **Compiler Spine** + **Ladder/Fixture** | — |

Does NOT need a new manager lane — fits within the existing §11 architecture from PR #3938.

**Critical DFS gate (Modeling DFS Manager):** Phase 1.5's `CiUpsertStep<T>` + `UpsertInputRef` substrate is substrate work. A DFS worksheet is REQUIRED before workers touch it. Spot-fix risks: (1) workers add `inputs: List<Symbol>` (string-keyed file globs) and miss the structural authority — must be typed coproduct (FileSet / SubstrateNodeSet / LensOutputRef / TestClaimRef / UpstreamUpsert) consuming `NodeQuery` / `LensId`; (2) workers introduce a `CiRunPolicy` / `CiRunMode` enum (heuristic — forbidden per operator directive 2026-05-30) instead of the literal `ci_always_run_carveouts` list with explicit reasons. **Any worker brief still using the prior "dependency_set" / "DependencySource" terminology OR any `Always` policy variant on UpsertInputRef is wrong-spec.**

---

## §8. Open questions for operator

**D-CI-1.** Accept the 3-layer target (§5) as the right shape, OR redirect to a different architecture?

*Proposed: accept.* Aligns with THESIS §"Two shapes of omni-emission" (ci.yml is Shape-B emit), INVARIANTS P2 single-authority, T-24 ratified phases.

**D-CI-2.** Accept the **Phase 1.5 addition** (**every CI step becomes an Upsert<T> Node** per operator directive 2026-05-29 + dsl/std/patterns.dag UPSERT<T> canon) to the ratified T-24 plan? **And accept D-CI-7 (below) since Phase 1.5 depends on Phase 1.4.**

*Proposed: accept.* Without per-step Upsert<T>, affected-set lens can't project minimal CI. The ratified plan implicitly assumes some step-shape but doesn't name Upsert<T> as the unit. Adopting Upsert<T> aligns with the existing canon (no new vocabulary) and coordinates with clever-cat-115's existing-shell retirement (each shell script ports to a `content_upsert` or `ensure<Check, Action>` row).

**D-CI-3.** Confirm **Compiler Spine + Modeling DFS** as the primary owner pair for this overhaul?

*Proposed: yes.* CI is workflow-stage work (Compiler Spine); per-step dependency declaration is substrate work (Modeling DFS). No new manager lane needed.

**D-CI-4.** Dispatch priority — start with **Phase 1a** (T-22 interpreter on ci_pipeline) OR **Phase 1.5** (dependency declaration) first?

*Proposed:*
- **Phase 1a** first (already in scope per T-24).
- **Phase 1.4** (Upsert<T> substrate landing) dispatches in parallel with Phase 1a once Modeling DFS Manager's substrate-extension worksheet is approved (D-CI-7 ratifies this scope).
- **Phase 1.5** dispatch is BLOCKED on Phase 1.4 completion (per §6 sequencing); cannot start until Upsert<T> is a usable substrate primitive. Phase 1.5 DFS worksheet can be authored in parallel with Phase 1.4 implementation, but worker briefs cannot dispatch.

**D-CI-5.** Scope of "minimal for highest confidence" — per operator directive 2026-05-30 ("I would minimize it to 'run affected only' — and then we can have a separate carve-out (not a mode/heuristic), just a literal list of 'things we always run regardless (superstition? not sure why exactly?)'"):

*Proposed:* **Every step is AffectedOnly by construction.** No `CiRunPolicy` / `CiRunMode` enum (heuristic — forbidden in a closed system per INVARIANTS P1). Always-run exceptions live in an explicit `ci_always_run_carveouts: List<CiCarveout>` data declaration; each entry carries `reason_code: Symbol` (stable machine key) + `reason_detail: String` (operator-readable prose, including "superstition — not sure why exactly" as valid honest content) + `dissolution_target: DissolutionTarget`. `UnknownYet` dissolution-targets require named `investigation_owner` + `review_due_by` per operator review 2026-05-30. Each carveout entry is a dissolution target: as we figure out the actual missing dependency, we model the substrate fact and remove the entry. Adding entries is a deliberate documented decision; removing entries is a substrate-modeling deliverable.

**D-CI-7.** Accept the **Phase 1.4 prerequisite** (land Upsert<T> as a usable substrate primitive — parser/substrate prerequisites in `dsl/std/patterns.dag`)?

*Proposed: accept.* Phase 1.5 (every CI step becomes an Upsert<T> Node) is structurally impossible without Phase 1.4. Modeling DFS Manager's substrate-extension worksheet covers the parser/substrate work needed to move Upsert<T> from "operator canon + commented stubs" to "usable substrate type." Scope this as substrate-extension scope (operator-bar decision) rather than worker-level scope.

**D-CI-6.** Should Phase 2.5 (affected-set intersection gate firing) be **part of T-24 [DONE]** or **post-T-24** (separate close gate)?

*Proposed: part of T-24.* The operator's ask is explicit that minimal-CI is the goal; T-24 closing without minimal-CI active leaves the operator's stated requirement unmet. Recommend amending T-24 close-predicate to include Phase 2.5.

---

## §9. Sub-questions worth surfacing (not blocking sign-off)

- **CI today runs `scripts/check-*`** that aren't yet typed `DisciplinePolicyCommand`. The bankruptcy doc plans A6–A8 to delete these in Phase 1b. If any need to always run even after their script form is deleted, they go in `ci_always_run_carveouts` with explicit reason — not a policy variant.
- **Self-hosted runners** (srv1/srv2 per `SelfHostedRunnerPool`): should runner allocation also be affected-set-driven (e.g., smaller affected-set → cheaper runner)? Out of scope for first pass; flag for later.
- **CI determinism** (per memory `project_determinism_as_effect`): determinism gates are orthogonal — they apply to each step regardless of affected-set. Determinism-as-effect modeling means determinism is checked at the substrate level for ALL steps; the carve-out list handles the small set of gates that must run regardless of affected-set, but determinism itself doesn't need carve-out treatment.
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
