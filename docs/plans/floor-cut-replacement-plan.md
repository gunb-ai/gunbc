# Floor cut (FLOOR-Y): delete CI, rebuild from agreement

DESIGN §3 and [replacement-migration doctrine](replacement-migration-doctrine.md) govern this cut. The 2026-08-15 operator ruling requires deletion first, then one agreed job at a time. Each major step stops for operator acceptance; fix-forward runs continuously inside the step. Earlier CI designs supply obligations and terminal shapes, never a reason to postpone deletion.

The recorded vehicle is `integration/floor-cut`, based on `64ebefa74`, with cutover PR gunbc#8283. These are historical cut coordinates, not a current CI inventory; DESIGN and the live workflow authorities own today's gate. [Original analysis and receipts](https://github.com/gunb-ai/gunbc/blob/06d3c1c3149a7420c4a548b9aba9b15c721c635d/docs/plans/floor-cut-replacement-plan.md) retains the exact deletion roster and boundary evidence.

## The cut, stated in .dag terms

Cut the emitting authorities and floor execution machinery together with their YAML projections. The old root comprised every `ci.yml` job plus falsifier/alert cadence. The agreed new root was one `run_required_floor` job, followed only by separately agreed jobs. Quarry: `session/vivid-bear-458-floor2` at `b19a3e2942` (`cli_run.rs`, `claim_executor`, one-job emission). Old machinery may serve only as bootstrap producer or historical measurement source, never fallback, internal resolver or production second opinion.

Fleet operations are excluded unless the operator rules otherwise: `.github/workflows/fleet-converge.yml` and `gunbc.fleet.fleet_converge_workflow`.

## Accepted deletion receipts

Selection deletion: `cb9277a4fe..c741d64204` (2026-08-15), end-to-end removal across model, plan, receipts, workflow jobs and seed paths, with projections regenerated. Wipe: `3a6398276f`, accepted at the operator boundary; no wipe-induced compile errors, regeneration completed and CI stopped because the merge ref had no workflow.

The wipe also removes artifact expectation rows, carries the §4b rung drop (previous/temporary rung, reason, bounded population, restoration trigger), and makes its orphaned obligations the re-add queue. Reachability determines the remaining dead floor models, binaries, fixtures and witnesses. Remove release-bin registrations with their producers. `ci_workflow`/`ci_spec` vocabulary needed by fleet convergence moves to a fleet-owned home before those remnants die; retaining gutted CI authorities is not the endpoint.

## What deletion cannot census

A green tree proves only the consumers that survived to object. Three silent populations require explicit review:

1. Deleted files can consume one another. Identify meanings that must survive even when both producer and sole checker disappear; `generated_stage0_files`/`assert_registry_is_partitioned` exposed an unguarded roster overlap.
2. A guard can die with its subject. Distinguish subsystem peers from obligations that must outlive a mechanism (`arm_declares_dissolution_trigger`, `SELF_COMPILE_ERROR_RATCHET`). Artifact-only ledgers miss vanished guards.
3. Survivors can become unreachable. Join imports, roster rows, test discovery, workflow argv and Cargo bins in the reach query. The measured import-only eight candidates reduced to three after that join; a single-medium census is insufficient.

A re-add therefore restores both artifact and guard. This cut removed guards for `dag_compile_clean_gate`, `generated_artifact_gate`, `regen_verify_gate` and `regen_verify_transport`; existence alone cannot discharge their restoration.

## Re-add queue: recursively delete-first

Nothing returns without agreement. For each obligation: derive the minimal job from first principles, use quarry only as oracle, and discard inherited structure. In particular, the old 19-binary roster and retry with `CARGO_BUILD_JOBS=1` followed by clearing `RUSTC_WRAPPER` do not survive automatically; the latter masks the sccache deficit.

- **Approved 2026-08-15:** one emitted Actions workflow/job calls one binary once, folding the entire witness roster with `run_required_floor`, SelectionOff. Checkout, toolchain and build include only needed binaries; no plans, batches, coordinator or selection. Build the emitter fresh; surgically port `prepare_repository_once`, `PreparedClaimScope`, `evaluation_frame` and `PreparedScopeIndexes`, never merge the quarry branch. Execute on a real push and present wall time, witness count and red/green census; reds are boundary evidence, not silently pre-fixed.
- **Regeneration fixed point:** its disposition belongs to `gunbc.witness.witness_floor_workflow` on the historical integration branch, not a copied verdict here. Merge order matters: v1-cut (#8293) first retires the `src/v1/**/*.dag` subject and obligation; floor-cut first leaves a live `regen_stage0` authority without a required invoking job; v1-cut never landing returns the unmet obligation to this queue. The second cut owns reconciliation and must not restore a job whose producer is gone.
- Generated-artifact drift gates; the seven effect gates (compile-clean, artifact drift, emit-host, extdeps citation, extdeps placement, prose-row, cheap-claim pool); Rust formatting.
- Heal requires reconsideration before any return. The proposed falsifier disposition was non-return because its measured mechanism had no consumer; a proposal is not an accepted retirement receipt.

The wipe also exposed silent `.gitattributes`/artifact drift, `tools.merge_admission_gate` without its receipt producer, loss of the sole unselected whole-corpus cold control, and formatting reduced to an opt-in bypassable hook. The boundary report and `commit_workflow.dag` gate roster retain that obligations ledger.

## Consumer exclusions and cutover

The recorded sweep protected the local `claim_batch` path; floor-discovery producer/transport (also the required-floor per-file roster authority from 2026-08-29); generic executor/scheduler/batch runner; fleet convergence; regeneration binaries; and `ci_floor_measurement`'s fleet-budget consumers. Broadly consumed compile-scope/layer-root/release-bin modules receive row-level cuts, not automatic file deletion. `node_frontier_selection` is a field-grain cut from generic Runnable and its seed realization. Registered `gunbc.plans.ci_*` quarry needs registry surgery before deletion. `cli_run.rs` and the interpreter are frozen-X trims only, with no new builtin rows.

Cut over atomically only when the agreed job set is green by execution, every obligation is discharged or receipted retired, and silent residue is restored or retired. Then present #8283 for the operator to merge; main never hosts the dual-authority interval or the integration branch's red states.

Registration is `gunbc.replacement_cut` row `FLOOR-Y`; #8276 had already merged on 2026-08-15. Registration was authorable follow-up, not blocked on that merge.
