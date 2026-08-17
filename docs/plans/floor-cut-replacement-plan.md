# Floor cut (FLOOR-Y): delete all of CI, rebuild job-by-job from agreement

Doctrine: DESIGN §3 *replacement migrations cut over at the root* (delete-first; the deletion is the census). Vehicle: integration branch `integration/floor-cut`, forked from main `64ebefa74`; standing cutover PR gunbc#8283 (draft — the one merge main receives; red mid-loop is legitimate there and nowhere else). Executing session: sleek-moth-351; doctrine/coordination: tidy-pike-117. Operator rulings (2026-08-15): delete what is certainly scheduled for death up front, before any fix-forward work; then **wipe every single thing in CI — full deletion now; jobs return one by one only as agreed.** **Step gating:** each major step closes only when the operator is satisfied with its performance — the executing session stops at the boundary and presents, never rolls into the next step on its own judgment; within a step, the fix-forward loop runs continuously. **Existing designs** covering this region (the five-minute-CI-gate program, witness-realization plan, and kin) are quarry — terminal shapes and obligations are evidence; their sequencing never defers the deletion (operator ruling, 2026-08-15).

## The cut, stated in .dag terms

The `.github/workflows/*.yml` files are projections; the cut population is the **authorities that emit them** plus the floor execution machinery.

- **Old root:** the generated CI surface and its .dag authorities — every job of `ci.yml` (`build`, `regen`, `ci` — whose core is the floor pass `claim_executor --plan-entry src/v2/workflow/ci_floor_plan.dag --plan-function gunbc_ci_floor_plan` — and `heal_generated_artifacts`), plus the falsifier cadence (`falsifier.yml`, `falsifier-alert.yml`).
- **New root:** one job calling `run_required_floor`, then whatever jobs the operator re-agrees, one at a time. The seed exists as quarry: branch `session/vivid-bear-458-floor2`, commit `b19a3e2942` — the `run_required_floor` fn in `cli_run.rs`, the `claim_executor` dispatch arm, the one-job emit in `dag/gunbc/ci_workflow.dag`.
- **X's residual roles** (closed vocabulary per the doctrine): bootstrap producer and historical measurement source, served from git history and the vivid-bear branch. Never a fallback, internal resolver, or production second opinion.
- **Boundary exclusion:** `.github/workflows/fleet-converge.yml` and `dag/gunbc/fleet_converge_workflow.dag` are fleet operations, not CI — outside this cut unless the operator rules otherwise.

## Step 0 — DONE (dated receipt, 2026-08-15): selection deletion

Eight commits on the branch (`cb9277a4fe..c741d64204`): affected-set selection deleted end to end — model, plan, receipts, workflow jobs, the seed paths in `src/v1/stage0/src/cli_run.rs` + `src/v1/stage0/src/bin/claim_executor.rs`, `src/v1/stage0/src/bin/selection_control_skip_witness.rs` — with the three workflow ymls regenerated. Net −6,397/+310 across 61 files.

## Step 1 — the wipe — LANDED and ACCEPTED (2026-08-15)

Receipt: `3a6398276f` (the wipe commit −12,367; branch cumulative −18,716 across 105 files); corpus compiles clean of wipe-induced errors; regen ran, `.gitattributes` shrank; cutover-PR CI stops by construction (no workflow file on the merge ref). Operator accepted at the boundary. One deviation from the list below, since ruled: `ci_workflow.dag`/`ci_spec.dag` initially survived as gutted GHA step vocabulary because `fleet_converge_workflow` imports ten step constructors and five timeout rows from them — ruled at the boundary: re-home those into a fleet-owned module, repoint `fleet_converge_workflow`, delete the remnants; CI machinery reduces to the single emission of step 3 item 1.

The wipe as specified:

Delete entirely:

- `.github/workflows/ci.yml`
- `.github/workflows/falsifier.yml`
- `.github/workflows/falsifier-alert.yml`

Gut or delete the emitting authorities **in the same motion**, so drift gates stop expecting the artifacts:

- `src/v2/workflow/ci_floor_plan.dag` — whole file, including `gunbc_ci_regen_floor_plan` (it returns from quarry when the regen job is re-agreed; keeping the obvious survivor is how X's structure creeps back)
- `dag/gunbc/ci_workflow.dag` and the `dag/gunbc/ci_spec.dag` job rows / emit surface for the three files; `dag/tools/gunbc_ci.dag` refuses or emits nothing until the first agreed job returns
- `dag/tools/emit_falsifier_yaml.dag` · `dag/gunbc/falsifier_workflow.dag` · `dag/gunbc/falsifier_cold_corpus_execution.dag` · `dag/gunbc/falsifier_lane.dag` · `dag/gunbc/falsifier_alert_admission.dag` · `dag/gunbc/falsifier_alert_decision.dag` · `dag/gunbc/falsifier_alert_residue.dag` · `dag/gunbc/falsifier_alert_state.dag` · `dag/gunbc/falsifier_run_control.dag` (the shared release-build step ids `falsifier_release_build_step_id` / `ci_release_bins_step_id` move or die with it)
- the `CiYamlArtifact` / `FalsifierYamlArtifact` rows in `dag/gunbc/generated_artifact.dag`

The wipe commit carries the §4b rung-drop declaration (previous rung, temporary rung, reason, bounded population, restoration trigger); **its obligation list is the re-add queue** (step 3).

Side effect: once `ci.yml` is deleted on the branch, PR #8283 stops running CI at all (the merge ref carries no workflow file) — the known-red runner burn stops.

## Step 2 — unreachable machinery falls out (the census does the rest)

Known-certain from the sweep; delete as reachability confirms:

- `src/v2/workflow/affected_set_floor_runner.dag` + `affected_set_floor_runner_test.dag` · `floor_skip_proof_plan.dag` · `floor_preparation.dag` (+ `dag/test/claim/floor_preparation_witness_test.dag`) · `affected_set_selection.dag` remnants · `probe_selector_affected_tests.dag` · `probe_selector_ci_runner_test.dag` · `probe_selector_host_health.dag` · `affected_testgen_ci_runner.dag` (+ its gate test) · `claim_witness_corpus_ci_runner.dag`
- `dag/gunbc/floor_component_receipt.dag` + `floor_component_receipt_document.dag` · `dag/gunbc/floor_materialization.dag` · `dag/gunbc/floor_resolve_realization.dag` · `dag/gunbc/floor_discovery_scaffold.dag` · `dag/gunbc/affected_set_stop_line.dag`
- `dag/tools/floor_effect_gate_witness.dag` — the wrapper dies; its **seven gate obligations** go to the re-add queue
- `dag/tools/floor_skip_discovery_witness_transport.dag` + `src/v1/stage0/src/bin/floor_skip_discovery_witness.rs` (requires the `dag/gunbc/ci_release_bins.dag` row removal)
- `src/v1/stage0/src/cli_run/floor_discovery_snapshot.rs` (767 lines; sole consumer is the claim_executor coordinator pre-plan digest)
- fixtures: `src/v2/test/fixture/floor_skip/` (all 7 files)
- witness tests: `dag/test/claim/falsifier_workflow_witness_test.dag` · `falsifier_lane_witness_test.dag` · `falsifier_run_control_witness_test.dag` · `falsifier_alert_decision_witness_test.dag` · `falsifier_alert_migration_witness_test.dag` · `floor_component_receipt_witness_test.dag` · `floor_batch_clamp_authority_witness_test.dag` · `floor_gate_failure_receipt_witness_test.dag` · `ci_floor_measurement_test.dag` · `floor_materialization_witness_test.dag` · `ci_floor_on_success_materialization_receipt_hand_rust_witness_test.dag` · `floor_skip_discovery_witness_test.dag` · the `dag_compile_clean_cli_floor_agreement` pair · `src/v2/test/claim/ci_floor_plan_witness_test.dag` · `src/v2/test/claim/falsifier_lane_plan_agreement_test.dag`

## What the census does *not* do — three bounds (measured 2026-08-15, five parallel cuts)

"The deletion is the census" holds for consumers that **survive to refuse**. Executing the doctrine on five branches at once surfaced three populations it cannot reach. Each is silent — no diagnostic, no unresolved reference, no red — so a green tree is complete over external loads and says nothing about these.

1. **A consumer inside the deletion set cannot refuse.** `regen_stage0.rs` defined both `generated_stage0_files` and `assert_registry_is_partitioned`, its only consumer. Re-homing that function's meaning while deleting the file left a real roster overlap with nothing left to object; witnesses stayed green. Ask, per deleted file: *what did this define that another deleted file consumed?*
2. **A guard that leaves with its subject takes an obligation with it.** Separate **guard** from **peer** — peers dying together as one subsystem is the cut working (16 of 42 deleted witnesses here); a guard exists precisely to outlive what it constrains. Specimens: `arm_declares_dissolution_trigger`, deleted alongside the dispatch arms it asserted about, and `SELF_COMPILE_ERROR_RATCHET`, a self-compile error bound declared and enforced in one deleted file and now absent tree-wide. A vanished guard produces **no artifact**, so an artifact-keyed landing ledger is blind to this class as well.
3. **A surviving thing can stop being reached.** The first two ask about deleted things; nothing asks whether a survivor still has a consumer. Measured: 8 orphan candidates by import graph → **3** after joining roster rows, `*_test.dag` discovery, workflow argv and Cargo `[[bin]]` — a 5-in-8 false-positive rate, and three of the five were reached by the discovery fold this cut itself delivers. **A reach measurement over one medium reads exactly like a reach measurement**, and nothing in the query announces the media it omitted; the medium join is part of the query, never a follow-up pass.

Corollary for Step 3: **a re-add is discharged by the artifact *and* its guard.** Four queued gates here — `dag_compile_clean_gate`, `generated_artifact_gate`, `regen_verify_gate`, `regen_verify_transport` — had their guards deleted by this same cut, so a re-add judged by "the gate is back" silently restores an unwitnessed gate.

## Step 3 — the re-add queue (one job at a time, operator agreement each, **recursively delete-first**)

The orphaned obligations, from the wipe's rung-drop declaration. Nothing returns unagreed — and a re-add is **not** a restore-from-quarry: each returning obligation gets the same sequence applied recursively (operator ruling, 2026-08-15: "they all have a lot of cruft — the same sequence has to be followed recursively"). Per re-add: state the obligation from first principles, design the minimal job that discharges it, mine the quarry as oracle only, and expect the old job's structure to die in the re-derivation. Two known cruft specimens illustrate why: the old build job's 19-binary roster is mostly floor/falsifier bins this cut deletes, and its `CARGO_BUILD_JOBS=1`-then-unset-`RUSTC_WRAPPER` retry arm is the absorbing fallback DESIGN's srvN build-cache thread already names as masking the sccache deficit. Neither survives a first-principles re-derivation. The queue:

1. **APPROVED (operator, 2026-08-15), spec verbatim: "a single github actions emission into our own binary - that then runs all test witnesses in the repo via a single fold."** One emitted workflow, one job — checkout · toolchain · build only the bins the fold needs · invoke the binary once; inside, `run_required_floor` as one fold over the whole tree-wide witness roster, SelectionOff — no plans, batches, coordinator, or selection. The emission authority is built fresh and minimal (never grown inside `ci_spec`/`ci_workflow`); the quarry's one-job emit (`b19a3e2942`) is the oracle; the `run_required_floor` stack (`prepare_repository_once` / `PreparedClaimScope` / `evaluation_frame` + interpreter-side `PreparedScopeIndexes` support) ports surgically — never a merge of the quarry branch. Subsumes the previously listed build and witness-corpus items. Boundary: land, execute on a real push, present wall time + witness count + the red/green census (reds are boundary data, never silently pre-fixed).
3. regen self-host fixed point — **this item's disposition is NOT carried here.** The executing lane records it on `gunbc.witness_floor_workflow` — **a carrier that exists only on `integration/floor-cut`, not on `main`, until this cut merges** — and that carrier is the authority; this plan names it and asserts nothing about its contents, because summarizing an authority is how a citation becomes a second copy of it (§3). The branch qualifier is load-bearing rather than incidental: an unqualified symbol invites a `main`-side grep that cannot resolve it, which is the stale-citation class this plan is otherwise arguing against.

   What this plan does contribute is the **cross-branch order-dependency**, which is a fact about the merge graph rather than about the queue and which no single branch can observe: `regen_stage0` is alive on `main` and deleted only by `integration/v1-cut` (gunbc#8293), whose cut removes the regeneration loop's subject — the `src/v1/**/*.dag` authority. So the item's disposition is a function of merge order, and all three arms are real: v1-cut first ⇒ the obligation retires with its mechanism; **floor-cut first ⇒ main carries a live regen authority with no required job invoking it, which is a gap someone owns rather than a settled absence**; v1-cut never lands ⇒ the item returns to this queue as an ordinary unmet obligation. Whichever cut merges second owns the reconciliation, and it must not be resolved by silently re-adding a job whose producer is gone.
4. generated-artifact drift gates
5. the seven effect gates: compile-clean · generated-artifact drift · emit-host · extdeps citation · extdeps placement · prose-row · cheap-claim pool
6. rust fmt gate
7. heal — proposed: reconsider whether the new minimal CI needs it before re-adding
8. falsifier cadence — proposed: never returns (DESIGN already records it measuring a mechanism nothing consumes)

Observed at the wipe boundary (identity-grain, 2026-08-15), beyond the predicted queue: (a) with the drift gate gone, `.gitattributes` and the surviving generated artifacts can drift **silently** until item 4 returns; (b) `tools.merge_admission_gate` reads a receipt the floor run used to stamp — merge admission has **no producer** until re-added; (c) the falsifier rows carried the only unselected whole-corpus cold control; (d) fmt exists only as the bypassable opt-in pre-push hook. The full ledger lives in the lane's boundary report and in `commit_workflow.dag`'s gate roster, retained deliberately — it IS the ledger.

## Contested / do-not-delete (sweep-verified non-floor consumers)

- **Keep whole:** `src/v1/stage0/src/bin/claim_batch.rs` (the local path) · `src/v2/workflow/floor_discovery.dag`, `floor_discovery_producer.dag`, `floor_discovery_transport.dag` (reached by the local producer path in cli_run, not by the floor) · `src/v2/workflow/floor_naming_hygiene.dag` (surviving `test fn` placement rule; non-floor importer) · `src/v2/workflow/executor.dag`, `scheduler.dag`, `batch_runner.dag` (generic substrate) · the fleet-converge pair · the regen bins
- **Trim rows only:** `dag/tools/dag_compile_clean_scope.dag` (broad non-floor consumers) · `dag/gunbc/ci_layer_roots.dag` (~40 importers) · `dag/gunbc/ci_release_bins.dag` (the owned-CI control plane formerly listed here was deleted whole with its lane; it is no longer a trim row)
- `dag/gunbc/ci_floor_measurement.dag` is a **fleet budget authority** (runner placement, oomd, host budgets import it) — survives despite the name
- **Field grain:** `node_frontier_selection` comes off the generic `Runnable` in `dag/std/realization_schedule.dag` + `src/v1/stage0/src/std_realization_schedule.rs` (in flight with step 0's area)
- `dag/gunbc/plans/ci_*.dag` planning carriers are registered quarry — deleting any requires plan-registry surgery; `dag/gunbc/plans/affected_set_self_confirmation.dag` is the only unregistered one
- **Frozen X:** `src/v1/stage0/src/cli_run.rs` and the interpreter — trims only, never file deletion, and no new builtin rows land there

## Green bar / cutover

The agreed job set green by execution on the branch; the obligations ledger empty or every remaining row retired with a receipt; the declared silent residue restored or retired. Then #8283 flips ready and the operator merges — one atomic cutover; main never hosts a dual-authority interval and never sees the red.

## Registration

`gunbc.replacement_cut` row `FLOOR-Y` — the carrier merged to main in gunbc#8276 (2026-08-15), so the row is now authorable and is an open follow-up for the executing session rather than a blocked one. (Corrected 2026-08-15: this line still read *"once gunbc#8276 merges"* while the sibling namespace plan recorded the merge, so the two cut plans asserted opposite states of one fact — §3 single authority, caught in review on gunbc#8297. `dag/gunbc/replacement_cut.dag` is present on `origin/main`.)
