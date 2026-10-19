# Serving hardware-neutral cutover

Doctrine: DESIGN §3 replacement migrations cut over at the root; procedure at [replacement-migration-doctrine.md](replacement-migration-doctrine.md). Shared serving vs compute occupancy: [serving-capacity-home-ruling.md](serving-capacity-home-ruling.md). This document is the **census and plan**. It does not delete anything. Deletion waits on parent (bold-tern-438) and operator admission of the cut.

Operator decision already on the record: cut Spark serving modules over to a hardware-neutral `gunbc.serving.*` home as **one replacement migration**; the served model is generic (Qwen is the first row, not the shape); GPU count and model artifact are **data**.

## 0. Why this census is authored, not deletion

`gunbc.emitted_subject_build_gate` `emitted_subject_build_rows` is two labels: `//gunbc/instruments:self-host` (`v2.compiler.compile`) and `//gunbc/instruments:v2-native-cli` (`v2.cli.compile_cli`). **None of the Spark serving modules, and none of the production consumers named below, sit in those two closures.** A required-job green after deleting them would not refuse. DESIGN §3: for a root outside the gate, enumerate consumers **by name** before delete. Prose that *names* these modules (docs, rung-drops, runbooks) also cannot break loudly — grep those names in the same change that deletes them.

This is the gap-intolerant carve-out for the **live Spark rack** (GLM four-rank native serving and harness turns on Group B; the pair/Ray deployment is retired, see §1): Y in shadow, then one authority transition. The **rented Blackwell cohort** (1× and 2× NVIDIA RTX PRO 6000 from Packet.ai / Vast.ai) is an independently complete root consumer and is the first vertical Y must hold.

## 1. Authority being replaced (X)

**Old root (consumer-relative):** `exact consumer × subject × contract` is not “Spark serving files”. The production question X answers today is:

> Given an arriving **turn**, which **loaded replica collection** may serve it, how is that collection **launched and observed**, and what **GPU inventory** sized it?

The consumer that currently gets that answer from Spark-keyed group launches is `gunbc.harness.harness_seat` (`harness_bind_seat`), fed by `gunbc.spark.serving_offer`, `gunbc.spark.serving_group_launch` (match on `FabricGroupB`), `gunbc.spark.vllm_serving_launch` / `vllm_endpoint_process_launch`, `gunbc.spark.serving_incarnation_observe`, and the pair/native realizations. `gunbc.serving.serving_enrollment` / `serving_front_door` / `serving_availability` already re-home *route* facts but still **import Spark** for ports, aliases, and process-launch keys — a meaning fork in progress, not Y.

**X population** (modules whose production presence the cut eventually removes *as serving authority*, not necessarily as files on day one):

| Module | Hardware standing |
|---|---|
| `gunbc.spark.serving_deployment_selection` | **Agnostic fold, Spark-bound evidence.** `std.decision` `select_realization`; candidates and regions cite DeepSeek V4.1, GB10, `FabricGroupB`, pair free-memory. |
| `gunbc.spark.vllm_serving_launch` | **Agnostic shape (vLLM launch).** Engine args + endpoint incarnation. First Y vertical re-homes here; Spark hosts stay rows. |
| `gunbc.spark.vllm_endpoint_process_launch` | **Agnostic shape.** Process/key the existing `gunbc.serving.{engine_progress,serving_availability,serving_front_door}` already consume. |
| `gunbc.spark.vllm_observed` | **Agnostic observation** of a models route; Spark host list is data. |
| `gunbc.spark.vllm_rank_population` | Spark-rank census; **Spark residue** until a generic rank roster exists. |
| `gunbc.spark.vllm_kv_layout_observe` / `vllm_allocation_plan_observe` / `vllm_memory_receipt` | Engine-layout observations; **agnostic folds**, Spark/GLM receipts as rows. |
| `gunbc.spark.vllm_image_identity` / `vllm_runtime_image_build` | Image identity/build; Spark host pins (`glm_native_ranked_hosts`) are **Spark residue**. |
| `gunbc.spark.serving_group_launch` | **Spark-specific dispatch.** Keys launch facts by `FabricGroup`. Y replaces this with a replica collection keyed by inventory, not fabric group. |
| `gunbc.spark.serving_offer` | **Agnostic contract, Spark route arms.** `harness_seat`'s SupplierOffer. Y keeps the offer contract; Spark site vs rented region become rows, not the type. |
| `gunbc.spark.serving_incarnation_observe` | **Agnostic observe**; wired through `harness_seat`. |
| `gunbc.spark.serving_engine` | **Agnostic engine axis** (`OllamaServing` \| `VllmServing`). Re-home under `gunbc.serving.*`. Bind/listen types drop Spark from the name. |
| `gunbc.spark.serving_arm` | **Spark-specific growth surface.** `ServingTopology` = `SingleSparkTp1` \| `SparkPairTp2` \| `FourSparkTp4` \| `EightSparkTp8`; GB10 121 GiB pool; GLM/DeepSeek checkpoint and `glm_native_*` profiles. Frozen: no new topologies, no new GB10 rows. |
| `gunbc.spark.serving_arm_launch` / `serving_arm_placement` | Spark host claims (`glm_canary_host_claim`, Group B srv9–12). **Spark residue.** |
| `gunbc.spark.serving_capacity_intent` / `serving_fit_projection` / `serving_runtime_capability_probe` | Fit against Spark unified memory / GB10 capability. Residue until a device-neutral fit joins `extdeps` device facts. |
| `gunbc.spark.serving_compatibility` / `serving_promotion` | Promotion/quality receipts; fold is agnostic, GLM FP8/NVFP4 rows are Spark/GLM residue. |
| `gunbc.spark.serving_performance_subject` / `serving_load_probe` / `serving_load_runner` | **Agnostic load contract** (`ServingLoadSubject`, bench_serve). First Y validation consumes these; Spark checkpoint/topology axes stay residue. |
| `gunbc.spark.serving_subject` | Spark Group-B subject for deployment selection. Residue. |
| `gunbc.spark.serving_rank_observe` / `serving_relaunch_transaction` | Rank inspect + relaunch delta on Spark containers. Residue. |
| `gunbc.spark.serving_deployment_decision_pending` | **No production consumer, no witness.** Dangling (§3c). Retire with receipt at cut; not a Y obligation. |
| `gunbc.spark.pair_serving_*` (desired, observed, realization, apply, authority, authority_log, launch_receipt) | **Retired, not a live boundary.** The modeled pair population is empty at this head: `FabricGroup` holds only Group B (Group A decommissioned, `gunbc.spark.fabric_switch_observed`), `spark_pair_serving_authorities` is `[]`, and `serving_group_launch` dispatches native GLM. Some declarations inside these files are still consumed (e.g. `pair_serving_realization` `declared_head_unit_render_for_group`, which delegates Group B to the native GLM renderer); each surviving helper gets its own disposition (re-home beside native GLM or delete). The pair deployment itself is not preserved or generalized. |
| `gunbc.spark.native_serving_realization` / `native_serving_apply` / `native_experiment_apply` | **Spark GLM four-rank.** Residue. Live rack boundary. |
| `gunbc.spark.fabric_serving_assignment` / `group_b_serving_capacity` | Fabric-group assignment and Group B alias/port. Residue; `gunbc.serving.admitted_model` / `serving_enrollment` still cite them. |
| `gunbc.spark.first_party_serving` | Operator-owned Spark *site* allocation (not a GPU inventory). Residue. |
| `gunbc.spark.glm_group_b_launch` / `glm_group_b_relaunch_cli` / `glm_serving_load` / `glm_canary_converge` | Spark GLM actuation. Residue. |

## 2. Production consumers by name (outside X)

Import-graph census (`.dag` `import gunbc.spark.<X> { … }`). Internal X→X edges are how X is decomposed; they do not earn survival. Reliances below **are** the migration cost.

### Root serving path (must be covered by Y or explicitly retired)

| Consumer | Imports from X | Hardware |
|---|---|---|
| `gunbc.harness.harness_seat` | `first_party_serving`, `pair_serving_authority`, `pair_serving_authority_log`, `pair_serving_desired`, `pair_serving_realization`, `serving_group_launch`, `serving_incarnation_observe`, `serving_offer`, `vllm_endpoint_process_launch`, `vllm_observed`, `vllm_serving_launch` | Mix: seat algebra is **agnostic** (`product.fabric.selection`, `product.capacity.lease`); route/launch keys are **Spark**. **This is the root consumer of the cut.** |
| `gunbc.harness.harness_backend` | `vllm_endpoint_process_launch`, `vllm_observed` | Agnostic probe of a vLLM route. |
| `gunbc.harness.harness_cli` | `vllm_endpoint_process_launch` | Agnostic. |
| `gunbc.harness.harness_reasoning_wire` | `serving_arm` | Spark/GLM unit rows (`Glm53FlashOnVllm`, …). Y: generic serving-unit table; Qwen first row. |
| `gunbc.serving.serving_enrollment` | `fabric_serving_assignment`, `group_b_serving_capacity`, `pair_serving_desired`, `pair_serving_realization` | **Fork in place:** enrollment already claims to own “who we may ask” but still reads Spark pair/Group B. Y owns this file’s *contract*; Spark ports become optional enrolled rows. |
| `gunbc.serving.serving_front_door` | `vllm_endpoint_process_launch` | Agnostic. |
| `gunbc.serving.serving_availability` | `vllm_endpoint_process_launch` | Agnostic. |
| `gunbc.serving.engine_progress` | `vllm_endpoint_process_launch` | Agnostic. |
| `gunbc.serving.admitted_model` | `group_b_serving_capacity` (`group_b_restoration_served_alias`) | Spark alias. Y: admitted ids from generic model rows. |

### Adjacent production (disposition at cut; not the first vertical)

| Consumer | Imports from X | Note |
|---|---|---|
| `gunbc.ci.ci_spec` | `native_serving_apply`, `pair_serving_apply` (receipt paths) | Workflow vocabulary; v1 required lanes withdrawn. Silent residue if X files vanish. |
| `gunbc.instruments.fabric_capacity_standing` | `pair_serving_authority`, `pair_serving_authority_log`, `vllm_kv_layout_observe`, `vllm_serving_launch` | Spark capacity instrument. |
| `gunbc.frontend_eval.serving_identity` | `vllm_observed` | Eval identity. |
| `gunbc.runner.runner_fleet_ci_posture` | `first_party_serving` | Fleet posture. |
| `gunbc.workspace_demo_preparation_policy` | `first_party_serving` | Demo policy. |
| `gunbc.fleet.fleet_workflow_steps` | `vllm_runtime_image_build` | Image build step. |
| `gunbc.fabric.fabric_store_operation_admission` | `pair_serving_authority_log` | Store admission keyed on Spark authority log. |
| `product.fabric.node_qualification` | `serving_rank_observe` | Qualification vs Spark rank inspect. |
| `product.superchip_assembly_qualification` | `serving_promotion` | Superchip vs promotion receipts. |

### Spark-adjacent (not in X; keep as Spark ops)

`gunbc.spark.host_commitment`, `host_occupancy_admission`, `managed_access_bootstrap`, `model_snapshot_materialize`, `fabric_rail_apply`, `arm_memory_fit`, `arm_memory_experiment_observed`, `prefill_batch_sweep`, `tensor_parallel_comparison`, `v41_runtime_image_probe`. These exist to operate the Spark rack. They stay **Spark rows**. They are not Y.

### Evidence modules

Witnesses under `test.claim.spark.*`, `test.claim.serving.*`, `test.claim.harness_*`, plus `test.claim.workflow_dispatch_input_witness`, `test.claim.vllm_memory_receipt`, `test.claim.vllm_kv_dtype_capability_witness`, `test.claim.frontend_eval_witness`, `test.claim.fabric.{node_qualification,jurisdiction_drop_accounting}_witness`, `test.claim.product.superchip_assembly_qualification_witness`. Doctrine: re-enroll claims Y still makes; retire Spark-only claims with a receipt. Not listed at identity grain here — the executing cut writes `EvidenceDisposition` rows.

### Prose that will not refuse (manual grep at deletion)

`gunbc.design_ledgers` `expected_design_rung_drops_md`, `docs/runbooks/glm-kv-allocation-plan-image-rebuild.md`, `docs/plans/serving-capacity-home-ruling.md`, `docs/plans/shell-to-dag-residual-census-and-arc-completion.md`, plus `gunbc.rung_drop` rows that name Spark serving symbols.

### Required gate

**Inside `emitted_subject_build_rows`: none of the above.** All X and all named consumers are outside the required job.

## 3. Target Y — `gunbc.serving.*`

Reuse, do not fork: `gunbc.serving.turn_admission` `serving_group_admission`, `serving_availability`, `engine_progress`; `product.fabric.selection` `select_supply`; `product.capacity.{LeaseGrant,SeatRequest}`; `gunbc.harness.harness_seat`; `extdeps/vllm/*`; `extdeps.vllm.bench_serve`. Re-home (do not duplicate) the agnostic folds today sitting in Spark: deployment selection, vLLM launch, load runner/probe/performance subject.

**Replica collection (the Y root shape):**

- Inventory is **data**: a list of devices (count, vendor SKU). First validation inventory: 1× and 2× NVIDIA RTX PRO 6000 Blackwell (Packet.ai, Vast.ai).
- Model artifact is **data**: a generic served-model row. First row: Qwen. Adding GLM or DeepSeek later is another row, not a new type.
- Collection **size derives from GPU inventory**. When the model **fits one card**, **one replica per device** (1 GPU → 1 replica; 2 GPUs → 2 replicas). Tensor-parallel across cards is a later row, not the first vertical — Spark `FourSparkTp4` does not become Y’s default topology.
- Each replica is a vLLM realization (`extdeps.vllm` engine args) bound through existing launch/observe types, re-homed under `gunbc.serving.*`.
- Admission stays `serving_group_admission` / `harness_bind_seat`. Occupancy stays the serving subject of the capacity ruling (loaded replica, quality-conditioned), not a compute cell.

**§3c for every new declaration:** the executing consumer is `gunbc.harness.harness_seat` (bind a seat on a replica of this collection) and the load path (`serving_load_runner` re-homed) over the 1×/2× inventories. No row without that route.

**Warm-session contract (minimum Y must establish it, not only launch replicas).** `gunbc.serving.turn_admission` today has no observation of prefix residency and treats every turn as cold-risk; its warm arm lands in #13717. The first implementation must make each of these an executed obligation:

- A session keeps its owning replica across model rounds and tool gaps. This preserves the behavior `gunbc.harness.harness_cli` `harness_worker_cli` already has (one backend bound for the whole tool loop) and lifts it from one call site to the collection.
- Adding a replica does not move existing session attachments; new sessions use the new capacity.
- Warmth is bound to the execution identity (the named launch) and the continuation identity (the exact token prefix). Context loss, process replacement and required reconstruction are distinct, typed outcomes (`ServingSessionResidency` in #13717), never folded into "warm".
- Normal continuation declares its reuse expectation (new input plus a bounded boundary replay), and execution verifies it against observed prefix-cache counters.
- vLLM stays responsible for its actual cache allocation. The model states the required lifetime and behavior, binds them to the serving profile, and names any missing backend capability as a gap rather than assuming it.

**Replica identity is the capacity-accounting domain.** `gunbc.harness.harness_seat` `HarnessSeatPoolIdentity` currently carries a `FabricGroup`, and pool partitioning and acquisition derive from it. Y replaces that key with the replica identity consistently across capacity, acquisition, the front-door permit (`serving_front_door`) and release. Each loaded replica has its own capacity limit; collection capacity is derived from its members, never declared beside them.

**What Y must refuse (minimum Y):** unread gauges are not idle (`serving_co_tenancy`); a model not in the admitted set is not served (`admitted_model`); missing inventory or a model that does not fit the per-device rule refuses rather than silently packing TP.

### Acceptance: one warm-session qualification, same procedure for every inventory

The existing load path does not exercise this workload. `extdeps.vllm.bench_serve` `VllmBenchServeRequest.dataset` is `RandomDatasetSpec` (input, output and shared-prefix lengths) and always emits `--dataset-name random`; `gunbc.spark.serving_load_runner` `ServingLoadSubject` names one host, container and endpoint. A shared random prefix exercises cache reuse, not a growing conversation that carries generated output forward, pauses for tools and resumes.

So the first vertical owes a named producer of a representative warm agent-session trace that executes through the admitted serving path (`harness_bind_seat` on a replica of the collection), not around it. The same trace qualifies the 1× and 2× inventories, covering:

- repeated continuation with realistic tool gaps and context growth;
- the declared context envelope, including permitted maximum-context requests;
- admission under pressure, and explicit recovery when warm state is lost;
- adding the second replica while existing sessions continue;
- a stated service envelope (turn latency, completion, sustainable admitted load), bound to a `serving_performance_subject`.

Adding another compatible GPU reruns this procedure against the changed inventory; it does not reopen the architecture.

## 4. The atomic cut

**Atomic = authority transition, not one commit.** Vehicle: integration branch. Main never hosts dual dispatch.

**Maximal cut refused (exact refusal):** Y cannot hold the live Spark rack (GB10 unified pool, RoCE four-rank native GLM on Group B) on day one. That is **Y-incomplete** for that cohort, not a licence to swap `serving_arm` leaf-upward.

**First complete cohort (independently complete root consumer):** warm agent-session serving on **rented 1×/2× Blackwell**, Qwen, one-replica-per-fitting-device. That cohort’s production path executes only through Y. It is physically absent from Spark schedule/dispatch. No fallback to `FabricGroupB` / `glm_native_*`.

**Staged form for the Spark rack (gap-intolerant):** X frozen — no new `ServingTopology` variants, no new Spark serving growth rows. Spark residue listed in §1 stays Spark. When a later session can hold that boundary in Y, one motion switches `harness_seat`’s Spark groups onto replica collections and deletes X’s production route.

**X residual roles after the first cohort:** historical measurement source and offline differential oracle (git). Never fallback, internal resolver, or production second opinion. Rented serving does not call `serving_group_launch(FabricGroupB)`.

**Do not:** introduce an X↔Y adapter, feature flag, or “if Spark then … else replica” in `harness_seat`. Partition by **cohort** (which inventory the turn is for), not by dual understanding of one turn.

## 5. Spark-specific residue (stays as Spark rows)

Everything whose subject is the **operator Spark fleet** as a physical fabric:

- `ServingTopology` Spark-count variants and GB10 pool arithmetic
- native GLM four-rank apply/realization/relaunch (the retired pair/Ray deployment is not residue; it is deleted, with surviving helpers dispositioned per §1)
- `FabricGroup` launch dispatch, Group B alias/port, first-party Spark site allocation
- host claims on srv9–12, RoCE GID selection, staircase container names
- GLM/DeepSeek-on-Spark checkpoint and `glm_native_*` launch policies
- `arm_memory_*`, `glm_*` converge/load/canary CLIs

These remain `gunbc.spark.*`. They are not copied into `gunbc.serving.*`. A Qwen-on-Blackwell replica collection must not grow a `NineSparkTp9` or a `FabricGroupC`.

## 6. Sequence (no deletion in this PR)

1. **This document** — census + Y shape + atomic cut. Parent review. Operator merge of the plan.
2. **Shadow Y** (later session): replica-collection types under `gunbc.serving.*`; Qwen row; inventory rows for 1×/2× RTX PRO 6000; launch/observe re-homed from the agnostic vLLM modules; load runner pointed at Y. Spark X untouched and frozen.
3. **Cohort cutover:** `harness_seat` (or a new serving binder that *is* the root for the rented cohort) binds rented inventory only through Y. Spark path still serves Spark turns only because those turns are a different subject population, not because of an adapter.
4. **Escalate before any X deletion.** Parent admits the cut. Then delete X production presence for any contract Y now dominates; grep prose names; declare silent residue / rung-drop for what cannot refuse (required gate, docs).

Terminal receipt (when the full migration closes, not now):

```
new root consumers                  = rented replica serving + (later) Spark-rack-rehomed
old root consumers of migrated contracts = 0
undispositioned consumers           = 0
uncovered surviving consumers       = 0
production X<->Y adapters           = 0
production fallbacks                = 0
migration feature flags             = 0
```

## 7. This PR’s consumer

This file is consumed by the parent serving-architecture session and by the session that executes shadow Y. It is not a typecheck trophy: the next executing route is that session’s first `gunbc.serving` replica-collection declaration, which must cite this plan’s Y shape or say why it diverges (§3b).
