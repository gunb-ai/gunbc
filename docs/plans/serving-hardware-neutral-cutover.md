# Serving hardware-neutral cutover

Doctrine: DESIGN §3 replacement migrations cut over at the root; procedure at [replacement-migration-doctrine.md](replacement-migration-doctrine.md). Shared serving vs compute occupancy: [serving-capacity-home-ruling.md](serving-capacity-home-ruling.md).

## 0. Standing (2026-10-12) — Spark serving retired

**Sold hardware, not a staged live-rack carve-out.** `gunbc.spark.dgx_procurement` `dgx_spark_disposed_units` records all eight procured DGX Sparks sold (Group A 2026-10-07, Group B 2026-10-10). `#13776` emptied serving enrollment. `#13765` rehomed pool/launch/front-door identity onto the rented replica (`gunbc.serving.rented_replica`, `replica_endpoint_launch`, `harness_bind_seat`).

The live-Group-B / GLM four-rank gap-intolerant carve-out in earlier revisions of this plan is **void**: there is no live Spark rack to hold. Spark serving production (launch, apply, group dispatch, native GLM apply, load probe) is **deleted**, not frozen.

Y for the rented Blackwell cohort is the production serving path: `gunbc.serving.*` + `gunbc.harness.harness_seat` + operator NInfer (`gunbc.harness.harness_operator_route`).

`gunbc.emitted_subject_build_gate` still does not close this graph. The census below is by name.

## 1. What was X (Spark serving production) — deleted

Deleted modules (actuation and Spark-keyed launch/observe):

`vllm_serving_launch`, `vllm_endpoint_process_launch`, `vllm_rank_population`, `vllm_kv_layout_observe`, `vllm_allocation_plan_observe`, `vllm_memory_receipt`, `vllm_runtime_image_build`, `serving_group_launch`, `serving_incarnation_observe`, `serving_load_probe`, `serving_relaunch_transaction`, `serving_deployment_selection`, `serving_deployment_decision_pending`, `serving_arm_launch`, `serving_fit_projection`, `serving_runtime_capability_probe`, `pair_serving_{realization,desired,observed,launch_receipt,apply}`, `native_serving_{apply,realization}`, `native_experiment_apply`, `glm_group_b_{launch,relaunch_cli}`, `glm_canary_converge`, `group_b_serving_capacity`, `fabric_serving_assignment`, `serving_engine`, `v41_runtime_image_probe`, `tensor_parallel_comparison`, `prefill_batch_sweep`, `arm_memory_experiment_observed`, `gunbc.instruments.fabric_capacity_standing`.

Witnesses under `dag/test/claim/spark/` that existed only to inhabit those modules were deleted with them.

## 2. Consumers enumerated by name (outside the required gate)

### Root serving path (Y; Spark imports removed or reduced)

| Consumer | Was importing from X | After this cut |
|---|---|---|
| `gunbc.harness.harness_seat` | `vllm_observed` (readback standing) | still reads `vllm_observed` (generic models-route observation) |
| `gunbc.harness.harness_backend` | `vllm_observed` | kept |
| `gunbc.harness.harness_reasoning_wire` | `serving_arm` projection types/rows | kept (`ReasoningResponseProjection` still lives on `serving_arm`) |
| `gunbc.harness.harness_operator_route` | none of X | kept (operator NInfer) |
| `gunbc.serving.serving_enrollment` | pair/Group B | already rehomed to rented replica; empty roster |
| `gunbc.serving.replica_endpoint_launch` | replaced `vllm_endpoint_process_launch` | Y |
| `gunbc.serving.admitted_model` | `group_b_restoration_served_alias` | alias inlined |

### Adjacent production (disposition)

| Consumer | Disposition |
|---|---|
| `gunbc.ci_spec` | receipt-path strings inlined; CI targets that named deleted apply entries are historical strings, not live modules |
| `gunbc.fleet.fleet_workflow_steps` | Spark image-build timeout dropped from the job backstop (no remaining build step; not a transcribed 180) |
| `gunbc.fabric_store_operation_admission` | still decodes **historical** pair-authority events (`pair_serving_authority_log`) — fleet store history, not serving production |
| `gunbc.instruments.fabric_capacity_standing` | **deleted** (Spark capacity instrument) |
| `product.fabric.node_qualification` | still uses `serving_rank_observe` GPUDirect receipt types |
| `product.superchip_assembly_qualification` | still uses `serving_promotion` `QualityReceipt` |
| `gunbc.workspace_demo_preparation_policy` / `gunbc.runner_fleet_ci_posture` | reuse first-party **posture types** from `first_party_serving` |
| `gunbc.frontend_eval.serving_identity` | `vllm_observed` |
| `gunbc.host_convergence_census` | Spark serving / runtime-image census rows **removed** |

## 3. Spark modules that stay (not serving production)

**Procurement / disposal / fleet history (do not delete):** `dgx_procurement`, `dgx_pxe_boot`, `fabric_switch_*`, `fabric_reach`, `fabric_rail_*` (apply now uses `host_privileged_apply`), `fabric_blob_peer`, host identity through `fleet_host_identity` sold-host **symbols** retained as fixture data (`gunbc.rung_drop.serving_fixtures_name_sold_group_b_hosts`).

**Host ops / grants / snapshots (Spark estate, not a serving path):** `host_commitment`, `host_occupancy_admission`, `host_effect_quiescence`, `managed_access_*`, `managed_grant_*`, `grant_privileged_operation`, `bootstrap_*`, `credential_workflow`, `model_snapshot_*`, `cell_role`, `training_ready`, `execution_cell`, `systemd_execution_cell`, `capability_observation`, `collective_*`, `remote_leg_began`, `remote_step_sequence`, `released_baseline`, `awaited_incarnation`, `deliberate_unit_start`, `rendezvous_participation`, `oobe_*`, `pxe_rescue_observation`, `secret_access_ensure`, `spark_file_blob`, `spark_gpu_direct_capability`.

**Empty serving-claim join for host_commitment:** `serving_arm_placement` (`spark_serving_arm_host_claims = []`), `serving_subject` (`spark_serving_reservations = []`).

**Historical names for managed-inspect grants:** `retired_serving_container_names`.

**Generic privileged SSH apply (extracted from pair apply):** `host_privileged_apply`.

**Vocabulary still consumed by GH200 qualification / harness wire / memory-fit types:** `serving_arm`, `serving_capacity_intent`, `arm_memory_fit`, `serving_compatibility`, `serving_promotion`, `serving_performance_subject`, `vllm_image_identity`, `vllm_observed`, `serving_rank_observe`, `pair_serving_authority`, `pair_serving_authority_log`, `first_party_serving` (posture; launch/incarnation types stubbed).

## 4. Target Y (unchanged shape; now the only live serving path)

Reuse: `gunbc.serving.turn_admission`, `serving_availability`, `engine_progress`, `product.fabric.selection`, `product.capacity.{LeaseGrant,SeatRequest}`, `gunbc.harness.harness_seat`, `extdeps/vllm/*`. Replica collection: inventory and model artifact are data; first vertical is rented 1×/2× RTX PRO 6000, Qwen, one replica per fitting device.

## 5. Why leftover Spark *vocabulary* files survive

These are **not** the deleted production path. Each has a live importer outside Spark serving apply/launch:

| Module | Live consumer (by name) |
|---|---|
| `serving_arm` | `gunbc.harness.harness_reasoning_wire` (`ReasoningResponseProjection`); `arm_memory_fit`; `serving_capacity_intent`; `serving_compatibility`; `serving_performance_subject`; `snapshot_rail_replica` checkpoints |
| `first_party_serving` | `gunbc.workspace_demo_preparation_policy`, `gunbc.runner.runner_fleet_ci_posture` (posture types only) |
| `serving_rank_observe` | `product.fabric.node_qualification` (GPUDirect receipts) |
| `serving_performance_subject` | `gunbc.spark.serving_promotion` |
| `arm_memory_fit` | `serving_promotion`, `kv_roster_group_vocabulary` witness |
| `model_snapshot_acquisition` / `model_snapshot_materialize` | host snapshot/grant path (`snapshot_rail_replica`, `fabric_blob_peer`, occupancy) — not serving launch |
| `host_commitment` | host-effect admission; empty serving-claim join |
| `vllm_observed` | `harness_seat`, `harness_backend`, `frontend_eval.serving_identity` |

Re-homing those types out of `gunbc.spark.*` is a later cut (sequence item 3). Deleting them in this PR would break Y.

## 6. Sequence remaining

1. **This cut** — Spark serving production deleted; this plan records sold-hardware retirement.
2. **Rented cohort** — continue on `gunbc.serving.*` / `harness_seat` (peer: bright-badger-372). Do not reintroduce `FabricGroup` as a serving key.
3. **Later** — re-home leftover Spark vocabulary (`serving_arm` projection types, `QualityReceipt`, GPUDirect rank receipts) out of `gunbc.spark.*` when those consumers are ready; not required to retire the sold path.

Terminal receipt for the Spark serving contract:

```
new root consumers                  = rented replica serving + operator NInfer
old root consumers of Spark serving apply/launch = 0
production X<->Y adapters           = 0
production fallbacks                = 0
```
