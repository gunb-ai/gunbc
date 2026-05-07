# Canvas — Substrate T-Workflow-As-Data carriers (5-row scope-narrowing)

**Sub-issue**: gunbc#1956 (T-Workflow-As-Data CI-workflow-as-.dag-data demo, parented under #1939); umbrella scope is §10.3 row Q-Workflow-As-Data-Carriers (line 983, OPEN — Substrate Mgr scoping needed).
**Authority**: `docs/r3-program-plan.md:474-480` (5 carrier names) + `docs/r3-design-schedule-2026-05-06.md:84-88` (audit-first directive); `dsl/extdeps/github/actions.dag` (218 lines, already-substrate); audit-and-delta receipt landed 2026-05-06 via #1771 (closed #1873).
**Closure predicate**: §1.8 gates #53 (workflow_substrate_carriers_landed), #54 (timing_lens_carrier_landed), #55 (shared_external_attachment_pattern_documented), #56 (ci_workflow_modeled_as_dag), #62 (substrate_gap_file_ingestion_closed), #63 (substrate_gap_workflow_scheduling_closed).
**Status**: **canvas — Director-tier ratification needed on reuse-vs-new scope before worker brief authoring**.

## Observation: 4 of 5 named carriers ALREADY exist in `extdeps.github.actions`

Per §S4 design-schedule directive line 84 (codex BLOCKING 2026-05-06): "S4 worker brief MUST audit `extdeps.github.actions` first and either (a) extend/refine existing carriers via T-Workflow-As-Data lens-consumption-shape additions (preferred per `feedback_audit_adjacent_authority_first` + `feedback_parallel_representation_debt`), or (b) explicitly dissolve `extdeps.github.actions` with a migration path before introducing parallel carriers."

Grep-verified `dsl/extdeps/github/actions.dag` at HEAD:

| §10.3 row 983 carrier name | extdeps.github.actions HEAD | Reuse/refine | Net new substrate |
|---|---|---|---|
| `WorkflowTrigger` (Push / PullRequest / Cron<Schedule> / Manual<Inputs>) | `WorkflowTrigger` (Push / PullRequest / Schedule / WorkflowDispatch / WorkflowCall) at `:40` | Refine: `Schedule { cron: String }` → typed `Cron<Schedule>` carrier (per design-schedule:87) | minimal |
| `WorkflowStep` (run command + dependencies + outputs) | `Step` at `:103` | Reuse name `Step`; lens-consumption may add observation anchor | none if pure reuse |
| `WorkflowMatrix<Axes>` (parameter expansion) | `MatrixStrategy` at `:66+` (inside `Job`) | Reuse + possibly extract as standalone carrier for lens consumption | minimal |
| `WorkflowSecret<Name>` (provider-typed, opaque-at-rest, scoped-by-step) | NOT EXTANT in actions.dag at HEAD | **NEW substrate** | full carrier |
| `RunnerResource<C>` (compute class, OS, hardware) | `RunnerSpec` + `RunnerLabel` at `:88+` | Reuse + parameterize as `RunnerResource<C>` for lens-shape consumption | minimal |
| `Workflow<Trigger, Steps, Resources>` composing carrier | `Workflow` at `:20` (untyped composition) | Refine to parameterized form for lens generic dispatch | minimal |

**Key finding**: of the 5 named carriers in §10.3 row 983, **only `WorkflowSecret<Name>` is wholly new substrate**. The other 4 are reuse-or-refine of existing `extdeps.github.actions` types. The audit-and-delta receipt (#1771, closed via #1873) confirmed this shape; the canvas territory now is **the lens-consumption-shape question**, NOT a 5-carrier-introduction question.

## Real canvas question (post-audit)

Given the existing `extdeps.github.actions` substrate is the reuse-base, what's the **minimum additional substrate** needed for T-Workflow-As-Data closure?

### Option α — Maximalist: introduce all 5 named carriers in `dsl/std/workflow.dag` as parametric refinements

New file `dsl/std/workflow.dag` declares parameterized versions of all 5 carriers; `extdeps.github.actions` types become specialized instances via composition. New `WorkflowSecret<Name>` lands here.

**Pro**: clean substrate-internal home for T-Workflow-As-Data; lens consumption talks to `dsl/std/workflow.dag` (compiler-internal vocabulary), not `dsl/extdeps/github/` (external-tool vocabulary).
**Con**: introduces parallel-representation debt with `extdeps.github.actions` (per `feedback_parallel_representation_debt`). The audit-receipt's reuse-first directive argues against this. 5 new types when only 1 is wholly novel.

### Option β — Minimalist (audit-receipt-honoring): land only `WorkflowSecret<Name>` + `Cron<Schedule>` refinement; lens consumes existing `extdeps.github.actions` types directly

Single new file `dsl/std/workflow_secret.dag` (or fold into `extdeps.github.actions` if scope-cohering) carrying `WorkflowSecret<Name>` + the typed `Cron<Schedule>` refinement. Lens-consumption shapes (e.g., `WorkflowObservationAnchor`) land in `dsl/std/workflow.dag` separately if/when needed by the lens; T-Workflow-As-Data's CI-workflow-as-.dag-data demo (#1956) consumes the refined `extdeps.github.actions` directly.

**Pro**: minimal substrate addition; honors audit-receipt's reuse-first directive (`feedback_audit_adjacent_authority_first`); single point of new-substrate, single point of refinement. No parallel-representation debt.
**Con**: lens consumption talks to `extdeps.github.actions` (external-tool vocabulary) — may look conceptually inconsistent with other lens consumers reading `dsl/std/*` types. Mitigation: documented as deliberate audit-receipt outcome.

### Option γ — Lens-consumption-shape carrier separately + minimal new substrate

Like β but with `WorkflowObservationAnchor` (per Substrate Mgr design stance at gunbc#1130 comment-4374109666) explicitly authored alongside `WorkflowSecret<Name>`. The lens-consumption layer is its own typed carrier; doesn't conflate with `extdeps.github.actions` reuse.

**Pro**: separates "external-tool vocabulary" (extdeps.github.actions, reuse) from "lens-consumption substrate" (new `WorkflowObservationAnchor`); each layer has single concern. Lens consumers read `WorkflowObservationAnchor`, which references `extdeps.github.actions::Workflow` structurally.
**Con**: 2 new substrate carriers vs β's 1; mild scope expansion. Justifiable IF lens-consumption-shape genuinely needs typed handle distinct from `extdeps.github.actions::Workflow`.

## Mgr-tier recommendation

Provisional **β** (minimalist, audit-receipt-honoring): only `WorkflowSecret<Name>` + `Cron<Schedule>` refinement land as net-new substrate. Lens consumes `extdeps.github.actions` directly until evidence shows a typed lens-handle is needed. **γ** is the natural ratchet from β if lens-consumption-shape evidence accumulates (per `feedback_construction_over_ratchets` — model first, dissolve later if substrate evidence forces).

**α rejected** — admits parallel-representation debt against `extdeps.github.actions` audit-receipt findings.

## Director ratification ask

1. **Pick α / β / γ** (or surface fourth). Mgr recommendation: **β**.
2. Confirm `extdeps.github.actions` audit-receipt at #1771 is the binding precedent for reuse-first posture (i.e., the audit confirmed reuse is the right shape, not deprecation).
3. Confirm `WorkflowSecret<Name>` location: `dsl/std/workflow_secret.dag` (new file) vs fold into existing `extdeps.github.actions` (extension). Provisional Mgr preference: **new file** (compiler-internal substrate, distinct from external-tool vocabulary; honors layer model).
4. Confirm whether §1.8 gate #54 (`timing_lens_carrier_landed`) and gate #55 (`shared_external_attachment_pattern_documented`) are in T-Workflow-As-Data scope or fold to T-LBP / separate sub-lane.

## On ratification — worker brief scope

Will author execution brief covering:
- `WorkflowSecret<Name>` carrier (`dsl/std/workflow_secret.dag` per option β/γ)
- `Cron<Schedule>` typed refinement (location TBD per question 3)
- (γ only) `WorkflowObservationAnchor` lens-consumption-shape carrier
- Worker pin: substrate-fact-introduction precedent owners (valiant-ibex-312 / smart-ram-167)
- Acceptance: §1.8 gates #53-#56 + #62-#63 advance per closure-predicate scope
- T-Workflow-As-Data #1956 demo consumer wiring (CI-workflow-as-.dag-data) in same-slice or cross-Mgr handoff per Director ratification

## Sequencing caveat

Per §S4 design-schedule line 95: "post-T-Lens-Behavioral-Parity COMPLETE (per `r3-structure.md` §"Dependency on R2"; lens consumption needs lenses COMPLETE)". This canvas is dispatch-ready post-T-LBP COMPLETE; brief authoring can land in advance per pre-staging discipline but worker dispatch waits.

— Authored by warm-wolf-698 (Substrate Mgr) 2026-05-07 post-#2105 merge per Director endorsement of pre-staging next-up substrate canvases. Honors audit-and-delta receipt #1771 (closed #1873) reuse-first directive.
