# Where the required floor's time goes (2026-10-02)

Status: investigation and plan. No code lands from this note; the repairs are dispatched separately.

Trigger: #12506 (N7-2) was refused by `v2.workflow.floor_enrolment_margin` (55 of 96 newly enrolled
identities over margin). The question asked was where the floor's time goes, and whether that
overage is one case of a general cause. Raising the budget is not on the table.

## Instrument and subject

The subject is main at `d8eebaea3f`, merge_group run 37070592712, `floor` job 111048851355. The
#12506 reading is run 37071586760, job 111052035724.

**No new instrument was minted.** The floor already prints every carrier this needs. They appear
in the job log:

| carrier | grain | producer |
| --- | --- | --- |
| `[floor-seam-cpu] seam=<s> thread_cpu_ms=` | cumulative thread CPU at each seam boundary | `claim_executor --required-ci` |
| `[floor-shared-fill] claim=<id> marginal_cpu_ms= fill_cpu_ms=` | per claim, split into its own work and the shared fill it paid for | `cli_run::shared_fill` |
| `[floor-shared-fill] cache=<c> key=<k> fill_ms= consumer_claims=` | per shared computation, with the count of claims that read it | `cli_run::shared_fill` |
| `[cross-claim-demand] producer=<p> claims= evals=` | per producer, recomputation across claims | `claim_executor` |
| `compile.<stage> done in` | each host compile stage | seed compile pipeline |

To re-derive, fetch the log with
`gh api --allow-escape-sequences repos/gunb-ai/gunbc/actions/jobs/<job>/logs`. Then difference
consecutive `floor-seam-cpu` values, and sum `fill_cpu_ms` and `marginal_cpu_ms` per claim and per
module.

**Instrument defect found along the way.** The two durable carriers are no longer uploaded:
`required-floor-claim-cost` and "the cross-claim demand TSV this run uploads" (that phrase is the
log's own). On both runs above, the only artifact is `required-ci-measurement-receipt`.
`tools.floor_cost_distribution_instrument` therefore has no input, and the log's sentence about the
upload is false. Restoring the upload is PR 3 below. Until then the attribution can only be read
from the log, which is a scrape.

## Structural predictions, stated before the confirming read

I wrote these down from DESIGN §2, §3 and §6b before differencing the seams:

- P1. Very little floor time is claim marginal work. Most of it is computation shared by many
  claims, paid either in preparation or by whoever touches it first.
- P2. The witness families that compile fixtures pay a compile of the fixture's whole live import
  closure once per distinct fixture, even though preparation has already compiled that closure.
- P3. The N7 overage is the same shape one layer up: a pure function of module-constant text,
  evaluated once per claim, with no provider joining the claims.
- P4. Some preparation work is demanded by the gate closure and read by no claim the run plans.

## Attribution (one reading; the carriers above are the authority)

Floor job wall time was about 47.5 minutes. Thread CPU, differenced by seam:

| slice | CPU | what it is |
| --- | --- | --- |
| build + parse + declarations | ~4m wall + ~61s | `cargo build`, parse of 7347 files |
| diff planning (base decl census, edits, nominal seeds) | ~100s | |
| **prepare-closure-resolve** | **~646s** | seed compile of the 2585-module gate closure; `compile.reconcile` alone took 8 minutes wall |
| **prepared-subject-warm** | **~377s** | 343 `cross_claim_pure_share` fills ≈ 316s, plus effect inputs |
| bare-reference edge index + discovery + site projection | ~120s | includes a second full parse of 7286 files (`reference-reading-parses full_parses=7286`) |
| **claim-evaluation-fold** | **~715s** (732s wall) | 673 claims |
| publication (wet witnesses) | ~6.5m wall, ≈0 thread CPU | `[local-repo-wet]` rows |

Inside the fold, the 197 claims with fill lines account for about 667s. Of that, about 666s is
`fill_cpu_ms` and about 1s is marginal. The fill is host compiles: 90 `compile.emit` runs (~539s)
plus the frontend and reconcile runs of 244 fixture compiles. Fourteen v1-checker witness modules
pay all of it, led by `test.claim.declared_type_inhabitance_direct_call_witness` (~210s),
`infer_function_value_argument_arrow_witness_test` (~103s) and
`declared_type_expected_type_path_witness` (~84s). Summed across all claims, the run's own
`claim_cpu_observed_total_ms` is about 10s.

Of the 343 pure-share fills paid in preparation, **298, totalling about 277s, report
`consumer_claims=0`**: no claim this run planned read them. The 45 that some claim did read total
about 39s.

P1, P2 and P4 hold on this reading. P3 holds on #12506's `[cross-claim-demand]` rows:
`cref_assemble claims=6 evals=6`, `fps_assemble claims=5 evals=5`, `mbp_assemble claims=2 evals=4`.
Each claim re-runs `assemble_program_from_ingest` on the same module-constant fixture, at roughly
0.6–0.8s per evaluation and about 250k–325k eval steps per claim. The new-witness budget is 72,300
steps.

## Causes, ranked by floor time removed

**C1. A fixture compile re-compiles the live closure the floor has already prepared (~10 min).**
The earliest unjustified boundary is `cli_run::emit_host` `compile_dag_diagnostic_census_uncached`
and its sibling `compile_dag_rust_emit_check`. For each distinct fixture they rebuild
`build_module_path_index_from_witness_roots`, resolve the fixture's imports from disk, and run
`compile_sources` over the whole closure. The contract only requires the fixture module to be new;
the closure is the prepared subject, which is already compiled under the same inventory digest
these memos key on. The memo is keyed per source, so it only absorbs repeats of the same text and
cannot reach the shared work across sources.
Repair class: **share a context at the least common ancestor.** Compile the fixture module against
the prepared closure's resolved and reconciled state, so only the new module is compiled. This is
#11401's R1 at fixture grain. Keep one claim that runs a fixture compile cold, as the inhabitance
claim for the real path.

**C2. The whole gate closure is reconciled for a small diff (~8–11 min).** Here `seeds=259`
produced `closure=2585`, and `compile.reconcile` took 8 minutes. I have not located the boundary
yet. The reconcile cost overlaps the type_env PR-2 slowdown, which calm-pike-525 owns (#13008,
#13009). Take that attribution as the input here; I am not redoing it. The open question for this
plan is narrower: does any planned claim demand a reconciled closure, or only a resolved one? If
only resolved, reconcile beyond the planned claims' reach is redundant demand.
Repair class: **delete redundant demand**, if that question confirms it.

**C3. The warm phase fills producers no planned claim reads (~4.6 min).** The roster
`v2.workflow.floor_pure_producer_share` is warmed for everything in the gate closure, but the run
plans 673 of 26,150 declared claims. The earliest unjustified boundary is the warm phase's
demand: it is keyed on the roster intersected with the closure, not on the roster intersected with
the planned claims' reach.
Repair class: **delete redundant demand.** Warm only producers that some planned claim reaches.
`body-reach-selection` already computes that reach.
Before cutting, one caveat needs checking. `unattributed_hits=399` counts hits outside the fold,
so confirm the zero-consumer rows are not being read by preparation itself.

**C4. Small items.** There is a second full parse of the pool after preparation (~1 min; share
the parse). The ~6.5 minutes of wall-bound publication and wet-witness time is not analysed here.

## Is #12506's overage a general cause? Yes: C1's shape, one layer up

The N7 witnesses intentionally execute the real front end over a fixture; their header argues
that the red is not authorable against a supplied index. A real path is legitimately demanded once
per fixture. What is not legitimate is paying it once per **claim**. The floor isolates claims, so
every claim is an isolated consumer of a pure function of module-constant text. DESIGN §2 says
such consumers create **one reuse obligation at their least common visible ancestor**. Today that
obligation is discharged only by hand. An author must make each producer nullary, roster it, and
re-verify, which is the restructure swift-wren-597 and bright-ant-369 are now doing. The roster
has 343 entries, and that number grows with every PR in the class. The next N7 PR will hit the
same wall for the same reason.

Two facts make it general.

1. **The obligation is derivable but authored.** `[cross-claim-demand]` already computes the
   signal: claims ≥ 2, keyed arguments, module-constant input. Rostering by hand is a second
   authority over a fact the demand graph carries (§3).
2. **The margin charges the shared fill to every claim.** `floor_enrolment_margin` is an honest
   policy budget under §5. It is denominated in deterministic eval steps, from a declared p90
   envelope. But it is applied to marginal work plus fill. Without a provider, every claim of the
   module re-pays the fill. So splitting a witness into more claims multiplies cost rather than
   isolating it, and the budget measures the absent provider rather than the claim.
   It is the first-toucher defect of gunbc#8455.

## Recommended first PRs

1. **C1: compile fixtures against the prepared closure** in `emit_host`. This is the largest
   removal, about 10 of 12 fold minutes. Postcondition: the 14 modules' `fill_cpu_ms` collapses
   to the fixture module's own compile, and `[floor-shared-fill]` shows one closure fill
   `paid_by=<outside-fold>`.
2. **C3: scope the pure-share warm to the planned claims' reach.** This is small and about 4.6
   minutes. Postcondition: `prepared-subject-warm` reports no `consumer_claims=0` fills.
3. **Derive the pure-share obligation from `[cross-claim-demand]` and restore the upload of the
   claim-cost and cross-claim TSV artifacts.** This also restores the named instrument this note
   had to read from the log. Postcondition: an N7-class witness module whose claims share one
   fixture is admitted with no rostering edit, and `floor_cost_distribution_instrument` runs
   against a current run.

Item 3 is the one that releases the N7 lane in general. Items 1 and 2 are the larger time
removals.
