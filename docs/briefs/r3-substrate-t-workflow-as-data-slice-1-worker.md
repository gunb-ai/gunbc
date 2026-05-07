# Worker brief — Substrate T-Workflow-As-Data Slice 1 (β ratified)

**Sub-issue**: parent #1956 (T-WAD CI-workflow-as-.dag-data demo) eventually consumes; PM authors a Slice-1-specific work-item under #1939 post-this-brief landing.
**Authority**: Director ratification of **option β** at gunbc#828 #issuecomment-4395945465 (2026-05-07); 4 asks confirmed (β / #1771 audit-receipt binding / WorkflowSecret<Name> folds into extdeps / slice 2-3 separate canvases).
**Closure predicate**: §1.8 gate #53 `workflow_substrate_carriers_landed` (this slice) + #62 `substrate_gap_file_ingestion_closed` + #63 `substrate_gap_workflow_scheduling_closed` (per T-Workflow-As-Data lane row scope).

## Slice scope (binding per Director)

T-Workflow-As-Data is split into 3 sub-slices per Director ratification ask #4:
- **Slice 1 (this brief)** — workflow substrate carriers (β minimalist)
- **Slice 2** — timing-and-pattern (TimingMeasurement + TimingObservationSet + WorkflowObservationAnchor + TimingBudget; gates #54 #55) — SEPARATE canvas, gated on T-LBP COMPLETE per §S4 design-schedule:95
- **Slice 3** — `ci_workflow_modeled_as_dag` demonstration (gate #56) — SEPARATE canvas, consumes slices 1+2

**Slice 1 net-new substrate** (only what was identified as wholly novel in the canvas):
1. `WorkflowSecret<Name>` carrier — provider-typed, opaque-at-rest, scoped-by-step
2. `Cron<Schedule>` typed refinement of existing `WorkflowTrigger::Schedule { cron: String }` (currently String; refine to typed cron expression)

Out-of-slice for the 4 reuse-named carriers (per audit #1771):
- `WorkflowTrigger` already at `dsl/extdeps/github/actions.dag:40` (only the inner `Schedule { cron: String }` refines to `Cron<Schedule>` shape per #2 above; outer `WorkflowTrigger` enum unchanged)
- `Step` (=`WorkflowStep`) at `:103`, `MatrixStrategy` (=`WorkflowMatrix`) inside `Job` at `:66+`, `RunnerSpec`+`RunnerLabel` (=`RunnerResource<C>`) at `:88+`, `Workflow` at `:20` — all reused as-is; lens consumes existing types directly per `feedback_audit_adjacent_authority_first`

## Carrier shape (binding per Director ask #3)

**Location**: `dsl/extdeps/github/actions.dag` — fold-into-extdeps, NOT new `dsl/std/` file. Per Director rationale: all sibling carriers already live there + secret management IS provider-specific (GitHub Secrets, GitLab Variables, AWS Secrets Manager, etc.) — putting `WorkflowSecret<Name>` in `dsl/std/` would imply cross-provider universality that doesn't exist at HEAD.

**`WorkflowSecret<Name>`**:

```dag
// Opaque-at-rest secret reference scoped by step. Name parameter carries the
// provider-side secret identifier (e.g., "GITHUB_TOKEN", "ANTHROPIC_API_KEY")
// without exposing the secret value at substrate level. Resolution happens at
// workflow-execution time via the provider's secret store.
type WorkflowSecret<Name> {
  name:       Name              // typed identifier (provider-scoped)
  scope:      SecretScope        // step-level vs job-level vs workflow-level
}

type SecretScope = StepScope | JobScope | WorkflowScope
```

**`Cron<Schedule>`** typed refinement at `dsl/extdeps/github/actions.dag:43`:

```dag
// Refines WorkflowTrigger::Schedule { cron: String } to typed cron carrier.
// Cron expression is structured (minute / hour / day-of-month / month / day-of-week)
// rather than opaque-string; fail-closed on parse errors at fixture load.
type CronExpression {
  minute:       CronField
  hour:         CronField
  day_of_month: CronField
  month:        CronField
  day_of_week:  CronField
}

type CronField = Wildcard | Exact(Int) | List(List<Int>) | Range(Int, Int) | Step(Int, Int)

// WorkflowTrigger update: replace inner Schedule { cron: String } with typed Cron<CronExpression>.
type WorkflowTrigger
  = ...                                    // existing variants unchanged
  | Schedule { cron: CronExpression }      // typed (was: String)
  | ...
```

**STOP-and-PING the Mgr** if `CronExpression` decomposes into more than 5 fields (e.g., year support or seconds support emerges as needed) — that's substrate-shape expansion warranting Director ratification.

### Cross-provider scope question (Director ratification ask #3 caveat)

Per Director: "if your lane visibility shows evidence the carrier is intended cross-provider (e.g., Anthropic provider work uses same shape), surface and we can elevate to `dsl/std/`. Default is fold-into-extdeps; promote-to-std only when cross-provider evidence accumulates."

Worker greps the codebase + adjacent provider work (Anthropic, OpenAI per `dsl/extdeps/llm/`) for `WorkflowSecret`-shaped patterns BEFORE folding. If cross-provider evidence emerges, **STOP-and-PING the Mgr**; otherwise proceed with fold-into-extdeps.

## Acceptance gates (same-slice, all must pass)

1. `WorkflowSecret<Name>` + `SecretScope` carriers landed in `dsl/extdeps/github/actions.dag`.
2. `CronExpression` + `CronField` carriers landed in `dsl/extdeps/github/actions.dag`; existing `WorkflowTrigger::Schedule { cron: String }` migrated to `Schedule { cron: CronExpression }` with fixture-load fail-closed parse semantics.
3. **No parallel-representation**: verify via grep that `dsl/std/` does NOT contain `WorkflowSecret`, `CronExpression`, or sibling shapes (would indicate accidental general-substrate creation against Director ratification).
4. §1.8 gates advance: #53 `workflow_substrate_carriers_landed` → CONSUMER_LANDED; #62 `substrate_gap_file_ingestion_closed` + #63 `substrate_gap_workflow_scheduling_closed` advance per closure predicate.
5. Bootstrap regen: `cargo test -p v3-compiler bootstrap_regen_fresh -- --ignored` clean.
6. Full suite: `cargo test --workspace --exclude v2-compiler-tests` green; `cargo clippy --all-targets -- -D warnings` clean.

## STOP / PING criteria

- **STOP** if cross-provider evidence emerges for `WorkflowSecret` shape (per Director ask #3 caveat) — surface to Mgr; promote-to-`dsl/std/` requires Director re-ratification.
- **STOP** if `CronExpression` field-count expands beyond 5 (e.g., year / seconds / nanoseconds) — substrate-shape expansion warrants Director ratification.
- **STOP** if migration of existing `WorkflowTrigger::Schedule { cron: String }` cascades into emit/typecheck surfaces beyond actions.dag — surface scope-creep.
- **PING** Verification Mgr (#2075) at PR-open time so they can advance §1.8 gates #53/#62/#63 ratchet authoring per standing concern.

## Sequencing

Per §S4 design-schedule:95: Slice 1 dispatch-ready post-T-LBP COMPLETE (lens consumption needs lenses COMPLETE). Brief authoring lands in advance per pre-staging discipline. Slice 2 + Slice 3 are SEPARATE canvases authored when their preconditions clear (slice 2 gates on T-LBP COMPLETE; slice 3 consumes 1+2).

## Worker pin (Mgr disposition)

valiant-ibex-312 OR smart-ram-167 (substrate-fact-introduction precedent owners). Final pin at dispatch.

## Auto-spawn caveat

Per Director's standing note + cache-staleness cluster ctrl#217: HOLD dispatch on this brief until auto-spawn fix lands per L-sized substrate-fact-introduction threshold.

— Authored by warm-wolf-698 (Substrate Mgr) 2026-05-07 per Director β-ratification at gunbc#828 #issuecomment-4395945465.
