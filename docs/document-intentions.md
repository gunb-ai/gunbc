# Document intentions

Review snapshot of [gunbc at d919d662ddf5](https://github.com/gunb-ai/gunbc/tree/d919d662ddf59813f9dab24ca3b25ed5b54ec236), surveyed 2026-10-28.

The [operator explicitly requested committing this worksheet](https://github.com/gunb-ai/gunbc/pull/15299#issuecomment-6460721529) to review one inferred purpose per file and decide what to delete, prune/consolidate, or leave. Intent does not establish implementation status.

The review consumes this snapshot. Remove it when the operator's cleanup dispositions have been recorded at the surviving sources or in the cleanup PRs.

Scope: tracked Markdown, documentation HTML, modeled plans and substantive text notes. Supporting files are listed separately; unrelated raw logs, test fixtures and license texts are outside this review.

## Proposed cleanup buckets

- **Delete:** obsolete or redundant; repair inbound references when removing it.
- **Refactor:** reduce design prose to goal, method and validation; consolidate unique requirements at their owning sources first.
- **Leave:** keep the current authority, useful guidance or evidence for now.

These are recommendations. Refactor may end in deletion after extraction; for executable models it targets redundant narrative, not the model itself.

| Bucket | Documents and designs | Supporting files |
| --- | ---: | ---: |
| Delete | 6 | 1 |
| Refactor | 113 | 1 |
| Leave | 39 | 45 |
| Total | 158 | 47 |

## Documents and designs (158)

### Repository root

| File | Bucket | Inferred intention | Reason / next step |
| --- | --- | --- | --- |
| [CLAUDE.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/CLAUDE.md) | Leave | Expose the governing gunbc design document through the agent instruction entrypoint without maintaining a separate policy text. | Keep the symlink to DESIGN; it introduces no independent prose authority. |
| [DESIGN.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/DESIGN.md) | Refactor | Establish gunbc's governing design principles, semantic authorities and proof obligations so authors build consistently toward the substrate model. | Refactor gunbc.design_document’s presentation while preserving axioms, derivations and governing obligations; retain conformance rules at their owning carriers and regenerate this projection. |
| [LICENSING.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/LICENSING.md) | Leave | Explain gunbc's permitted uses, commercial licensing threshold and release conversion terms in plain language alongside the controlling license. | Keep concise user-facing licensing guidance alongside the controlling license. |
| [README.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/README.md) | Leave | Expose the governing gunbc design document through the repository's standard reader entrypoint without maintaining a separate overview. | Keep the symlink to DESIGN; it introduces no independent overview. |

### apps/approve-ios

| File | Bucket | Inferred intention | Reason / next step |
| --- | --- | --- | --- |
| [README.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/apps/approve-ios/README.md) | Leave | Guide building and configuring the iOS approval app while keeping its wire protocol and generated Swift tied to modeled authorities. | Keep active app build, signing and configuration instructions; these are operational guidance rather than a plan. |

### apps/approve-ios/Approve/Assets.xcassets/AppIcon.appiconset

| File | Bucket | Inferred intention | Reason / next step |
| --- | --- | --- | --- |
| [README.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/apps/approve-ios/Approve/Assets.xcassets/AppIcon.appiconset/README.md) | Leave | Identify the approval app icon's geometry, colors and generation provenance so the raster remains traceable to the brand mark. | Keep the short icon provenance and authority pointer. |

### artifacts/bmc

| File | Bucket | Inferred intention | Reason / next step |
| --- | --- | --- | --- |
| [mtcollins1-bmc-user3-route-2026-09-13.txt](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/artifacts/bmc/mtcollins1-bmc-user3-route-2026-09-13.txt) | Leave | Explain the replacement-admin credential sequence, ordering failure and channel coverage needed to distinguish restricted factory access from true credential retirement. | Keep the credential-route incident evidence consumed by machine_intake, privileged-effect and failure-mode carriers. |
| [mtcollins1-topology-control-known-good-socket0-2026-09-12.CORRECTION.txt](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/artifacts/bmc/mtcollins1-topology-control-known-good-socket0-2026-09-12.CORRECTION.txt) | Leave | Retract the topology-control success claim and explain why quiet event logs cannot establish memory training or a successful boot. | Keep the correction preventing quiet event logs from being reused as proof of training or boot. |
| [mtcollins1-user2-retire-2026-09-13.txt](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/artifacts/bmc/mtcollins1-user2-retire-2026-09-13.txt) | Leave | Explain how password rotation and discriminating readbacks establish factory-credential retirement while keeping the disabled account's channel permissions unchanged. | Keep the discriminating credential-retirement receipt consumed by security observation and privileged-effect carriers. |

### artifacts/bmc/altrad8ud-1l2t-6-inband-recovery-2026-10-16

| File | Bucket | Inferred intention | Reason / next step |
| --- | --- | --- | --- |
| [00_README.txt](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/artifacts/bmc/altrad8ud-1l2t-6-inband-recovery-2026-10-16/00_README.txt) | Leave | Distinguish captured evidence from unsaved session observations in ALTRAD8UD BMC recovery so later models cite each claim at its actual strength. | Keep provenance separating captured bytes from unsaved observations; it constrains every reading of the recovery evidence. |
| [01_summary.txt](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/artifacts/bmc/altrad8ud-1l2t-6-inband-recovery-2026-10-16/01_summary.txt) | Leave | Explain the in-band IPMI credential-recovery route, factory-reset behavior and readback needed to return an inaccessible ALTRAD8UD BMC to onboarding. | Keep the first recovery-route specimen and its explicit evidence limitations. |

### artifacts/bmc/mtcollins1-32dimm

| File | Bucket | Inferred intention | Reason / next step |
| --- | --- | --- | --- |
| [operator-scripts.txt](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/artifacts/bmc/mtcollins1-32dimm/operator-scripts.txt) | Leave | Preserve the operator scripts used to stimulate boots and collect console, power-cycle and event-log observations during the 32-DIMM investigation. | Keep exact historical collection procedures beside their captures; do not promote them into a current runbook. |

### artifacts/bmc/mtcollins1-virtual-media-readonly-probe-192.168.1.228-20261027T005424Z

| File | Bucket | Inferred intention | Reason / next step |
| --- | --- | --- | --- |
| [ARGV.txt](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/artifacts/bmc/mtcollins1-virtual-media-readonly-probe-192.168.1.228-20261027T005424Z/ARGV.txt) | Leave | Document the anonymous virtual-media surface probe, its exact requests and authority limits so observations can be reproduced without controller mutations. | Keep the superseded probe method as provenance; the corrected pass explicitly retains the earlier capture. |
| [META.txt](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/artifacts/bmc/mtcollins1-virtual-media-readonly-probe-192.168.1.228-20261027T005424Z/META.txt) | Leave | Bound the first virtual-media surface reading by its observer, timing, transient-network retry and concurrent-state caveats rather than implying settled media readiness. | Keep the earlier probe context so its timing limitations and supersession remain auditable. |

### artifacts/bmc/mtcollins1-virtual-media-readonly-probe-192.168.1.228-20261027T040012Z

| File | Bucket | Inferred intention | Reason / next step |
| --- | --- | --- | --- |
| [ARGV.txt](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/artifacts/bmc/mtcollins1-virtual-media-readonly-probe-192.168.1.228-20261027T040012Z/ARGV.txt) | Leave | Explain the instrumented virtual-media probe's corrected requests, separate server responses and redaction procedure so its evidence remains reproducible and attributable. | Keep the corrected probe procedure and redaction/provenance record attached to the measured media surface. |
| [META.txt](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/artifacts/bmc/mtcollins1-virtual-media-readonly-probe-192.168.1.228-20261027T040012Z/META.txt) | Leave | Interpret the instrumented BMC surface reading using separate clocks and corroborated unit identity while preserving uncertainty about authenticated Redfish access. | Keep observer/controller clock separation and authenticated-read limitations attached to the corrected observation. |

### artifacts/bmc/mtjade1-j3-media-attempt-192.168.1.246-20261006T185027Z

| File | Bucket | Inferred intention | Reason / next step |
| --- | --- | --- | --- |
| [SEQUENCE.txt](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/artifacts/bmc/mtjade1-j3-media-attempt-192.168.1.246-20261006T185027Z/SEQUENCE.txt) | Leave | Reconstruct the Mt. Jade virtual-media incident, separating authorized actions, stop-condition violations and restoration readbacks without claiming a verified attachment. | Keep the incident sequence and authorization overrun evidence consumed by mtjade1_access_observation. |

### artifacts/bmc/srv13-first-contact-2026-10-07

| File | Bucket | Inferred intention | Reason / next step |
| --- | --- | --- | --- |
| [FINDINGS.txt](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/artifacts/bmc/srv13-first-contact-2026-10-07/FINDINGS.txt) | Leave | Explain srv13 intake findings across firmware, boot configuration and thermal failures, linking each interpretation to captures that constrain the next investigation. | Keep the intake findings with their capture references and unresolved interpretations. |

### artifacts/printing/process-observation-2026-10-05

| File | Bucket | Inferred intention | Reason / next step |
| --- | --- | --- | --- |
| [README.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/artifacts/printing/process-observation-2026-10-05/README.md) | Refactor | Compare failed and recovery fan-print archive commands while distinguishing process observations from geometric explanations and physical acceptance. | Keep producer, input and limitation pointers; move derived findings to product.printed_chassis.print_preparation observation carriers. |

### artifacts/printing/rear-posts-2026-10-05

| File | Bucket | Inferred intention | Reason / next step |
| --- | --- | --- | --- |
| [INCIDENT.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/artifacts/printing/rear-posts-2026-10-05/INCIDENT.md) | Leave | Document the upright fan-print failure and its evidence so telemetry completion cannot be mistaken for a successful physical assembly. | Keep the short evidence index and telemetry-versus-physical distinction. |

### artifacts/spark/glm-group-b-2026-09-16

| File | Bucket | Inferred intention | Reason / next step |
| --- | --- | --- | --- |
| [README.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/artifacts/spark/glm-group-b-2026-09-16/README.md) | Leave | Identify raw GLM container and NCCL captures used to compare socket and RoCE transport configurations on Group B hosts. | Keep raw capture provenance and parsing limits. |

### artifacts/spark/glm-group-b-2026-09-16/narrow

| File | Bucket | Inferred intention | Reason / next step |
| --- | --- | --- | --- |
| [README.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/artifacts/spark/glm-group-b-2026-09-16/narrow/README.md) | Leave | Explain narrowed GLM container observations and distinguish fresh daemon responses from fields projected out of archived RoCE captures. | Keep the narrowed capture method and uncertainty attached to its evidence. |

### dag/examples/blackjack

| File | Bucket | Inferred intention | Reason / next step |
| --- | --- | --- | --- |
| [README.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/examples/blackjack/README.md) | Leave | Teach Dag modeling through deterministic Blackjack rounds, seeded simulation, explicit refusals and replayable test evidence. | Keep the working example setup and teaching entrypoint. |

### dag/examples/weather

| File | Bucket | Inferred intention | Reason / next step |
| --- | --- | --- | --- |
| [README.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/examples/weather/README.md) | Leave | Demonstrate Dag types, pattern matching and collection pipelines by compiling a self-contained weather model into type-checked Rust. | Keep concise setup and execution guidance for the working example. |

### dag/gunbc/plans

| File | Bucket | Inferred intention | Reason / next step |
| --- | --- | --- | --- |
| [boot_origin_bmc_media.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/boot_origin_bmc_media.dag) | Refactor | Deliver content-addressed boot images from R2 through HTTPS or a segment-local NFS relay so different BMCs consume one image authority. | Keep delivery goal, route selection and acceptance; cite media observations instead of restating their readings. |
| [cardinality_refinement.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/cardinality_refinement.dag) | Refactor | Make cardinality constraints refuse invalid construction rather than rely on call-site convention. | Move surviving cardinality obligations to checked-constructor authorities; remove the retired where-refinement proposal. |
| [ci_merge_freshness.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/ci_merge_freshness.dag) | Refactor | Define merge-time receipt freshness so individually green changes cannot compose against an untested target tree or gate roster. | Keep the unresolved freshness decision and proof; move incidents and measured status to merge-admission and failure-mode carriers. |
| [ci_process_end_to_end.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/ci_process_end_to_end.dag) | Refactor | Coordinate CI cost measurement and implementation changes across the complete path from source change to accepted result. | Transfer remaining CI cost obligations to current compiler-gate and demand-engine owners; retire the old pipeline charter. |
| [cli_run_hollowing_plan.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/cli_run_hollowing_plan.dag) | Refactor | Move policy out of the Rust CLI entrypoint into substrate-owned consumers so the bootstrap remains a shrinking execution boundary. | Replace repeated area narratives with the seed-retention frontier and a short self-hosting objective, migration and proof. |
| [construction_justification_rule.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/construction_justification_rule.dag) | Refactor | Explain why constructions need a production consumer and track the missing ingestion/enrollment enforcement. | Move the ingestion-enforcement gap to gunbc.guarantee_stall; retain only rationale beside the construction-justification authority. |
| [convergence_coverage_program.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/convergence_coverage_program.dag) | Refactor | Provide one governing spine for covering production convergence to the execution boundary. | Keep goal, shared method and execution proof; derive open leaves from roadmap rows and remove duplicated findings. |
| [dag_v2_defork_audit.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/dag_v2_defork_audit.dag) | Refactor | Locate semantic duplicate implementations between DAG and v2 with independent negative controls. | Replace the evolving audit narrative with identity-level fork evidence and current negative controls. |
| [demand_engine_program.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/demand_engine_program.dag) | Refactor | Define how the demand engine identifies work, shares results and selects materialization, realization and placement through one coherent model. | Keep goal, mechanism and proof; model unique decisions and milestone exits before removing repeated inventories and receipts. |
| [deploy_convergence_observed_side.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/deploy_convergence_observed_side.dag) | Refactor | Define what deployment observation proves and which historical paths were dead ends. | Move unresolved release prerequisites to live-deploy carriers; retain essential rationale and remove refuted-design history. |
| [durable_work_handoff.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/durable_work_handoff.dag) | Refactor | Define durable work handoff across attempts and storage boundaries so replacement processes continue the same obligation without losing its identity. | Model settlement, liveness and transfer obligations before reducing prose to goal, shared mechanism and execution proof. |
| [ensure_closure_design.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/ensure_closure_design.dag) | Refactor | Require executable plans to establish their environmental prerequisites at the selected placement before compilation can admit production execution. | Preserve the ruled completion/environment contracts in their owners; reduce explanatory prose to objective, specialization method and proof. |
| [extdeps_modeling_preflight.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/extdeps_modeling_preflight.dag) | Leave | Carry actionable external-model preflight rows rather than repeat modeling assumptions in each brief. | Keep the consumed typed preflight rows and their derived table. |
| [fabric_serving_router.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/fabric_serving_router.dag) | Refactor | Define how a fabric serving router selects available serving capacity and relates requests to the shared allocation model. | Keep streaming objective, selected proxy/native boundary and live proof; move slice obligations to serving and proxy owners. |
| [fleet_naming_reach_access.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/fleet_naming_reach_access.dag) | Refactor | Unify fleet naming, endpoint resolution and authorized access so operations reach the intended host without embedded provider or address assumptions. | Keep naming/access goal and cutover proof; move settled constraints to endpoint owners and replace the survey with citations. |
| [fleet_subsumption_manual_gaps.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/fleet_subsumption_manual_gaps.dag) | Refactor | Identify manual fleet procedures missing modeled equivalents so boot and convergence programs can replace their operational responsibility. | Move each surviving manual-fleet gap to boot, authentication or convergence owners; delete the old machine journal. |
| [flex_supplier_generalization.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/flex_supplier_generalization.dag) | Refactor | Add AWS EC2 spot and on-demand capacity as priced flex-autoscaler suppliers without duplicating the existing Vast orchestration loop. | Keep supplier-generalization goal and admission proof; model unique EC2 constraints before removing corpus surveys and repeated boundaries. |
| [for_while_loop_sugar.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/for_while_loop_sugar.dag) | Refactor | Define for and while syntax through precise desugaring so convenient loops preserve the language's control-flow and termination semantics. | Carry desugaring, domain and refusal contracts in language owners first; reduce the plan without implying unexecuted loops already work. |
| [gpu_card_host_program.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/gpu_card_host_program.dag) | Refactor | Coordinate card, host, power and serving acceptance choices with unresolved operator selections. | Prune narrative to objective, selection and qualification; retain typed census, dependency rows, unknowns and operator decisions. |
| [gunb_ai_projection.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/gunb_ai_projection.dag) | Refactor | Map gunb.ai website content, design tokens, layout and deployment to shared modeled authorities so the site presents one consistent source of facts. | Keep a source-to-owner migration map; move website facts to their content/design authorities and remove copied inventories. |
| [harness_session_lifecycle.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/harness_session_lifecycle.dag) | Refactor | Keep one worker session across transparent supervision rounds until hand-in, decision or stop, then drive acceptance and revision through events. | Move standing session rulings and continuation contracts to harness owners; keep only goal, method and acceptance. |
| [host_convergence_circuit_residue.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/host_convergence_circuit_residue.dag) | Refactor | Record host-convergence circuit evidence and outstanding live-wiring obligations so hermetic loop tests are not mistaken for operational deployment proof. | Move outstanding live-wiring obligations to convergence_coverage_program and host carriers; retire the acceptance narrative. |
| [host_effect_orchestration.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/host_effect_orchestration.dag) | Refactor | Replace host-side orchestration with typed apply/ensure interfaces while retaining actual effect boundaries. | Move any missing operation to host_effect_realize and std.effect_plan; reduce the old interface proposal to necessary rationale. |
| [inert_layer_lens.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/inert_layer_lens.dag) | Refactor | Detect declarations unreachable from production roots while distinguishing test-only coverage and heuristic importance from actual runtime use. | Retain unresolved production-reach obligations; remove retired backstop history and derive inventories from existing graph authorities. |
| [input_envelope_roadmap.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/input_envelope_roadmap.dag) | Refactor | Bound work inputs along declared axes so admission refuses unknown or oversized corpora and scheduling can account for their resource demands. | Move the remaining derived-ceiling and runtime-admission obligation to its envelope owner; retire the separate phase plan. |
| [interpreter_kernel_d.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/interpreter_kernel_d.dag) | Refactor | Bound the irreducible interpreter/physics kernel and account for policy that can move to the substrate. | Move remaining evaluator and host-boundary obligations to the self-host frontier; retain only the irreducible-kernel rationale. |
| [joint_incompatibility_claim_join.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/joint_incompatibility_claim_join.dag) | Refactor | Scope an advisory cross-PR import-claim join under the deferred freshness ceiling and manager/operator permissions. | Keep the permitted decision scope and evidence links; remove repeated cost calculations and incident history. |
| [logical_work_realization.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/logical_work_realization.dag) | Refactor | Define the consumer interface and bounded pilot from logical demand through selected realization and completion. | Model the logical interface and pilot obligations first; replace long source inventories with references and a short acceptance plan. |
| [mtcollins1_ci_service_legs.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/mtcollins1_ci_service_legs.dag) | Refactor | Sequence Mt. Collins runner qualification before CI service and bind launch, observation and teardown to the authorized attempt. | Keep qualification/service goal and live-run proof; cite owning contracts and retain the distinction between wired, unbound and observed. |
| [network_install_architecture_rows.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/network_install_architecture_rows.dag) | Refactor | Generalize architecture-specific network installation while preserving existing boot behavior and trust evidence. | Move architecture-specific constraints to their rows; keep the shared installation change and x86/ARM regression proof. |
| [ntfy_deployment_gate_approval.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/ntfy_deployment_gate_approval.dag) | Refactor | Specify an approval-notification flow through a GitHub deployment protection app so privileged deployments require an operator's decision. | Reconcile remaining approval obligations with srv1_root_mutation_approval_run; retire the superseded deployment-protection proposal. |
| [p4_home_serving_power_semantics.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/p4_home_serving_power_semantics.dag) | Refactor | Distinguish GPU rated power from applied limits so home-serving selection admits only power settings supported by cited evidence. | Move power distinctions to home_serving_power_policy and open decisions to gpu_card_host_program; remove repeated argument. |
| [physical_machine_boot_program.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/physical_machine_boot_program.dag) | Refactor | Define one physical-machine boot operation reusable across installation and runtime lifecycles without conflating host and guest responsibilities. | Keep the shared boot transaction and acceptance; move remaining lifecycle obligations to owners without merging physical and guest authority. |
| [reference_grounding_migration.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/reference_grounding_migration.dag) | Refactor | Replace name-like references with correctly grounded typed references without confusing role strings for names. | Keep current typed-reference obligations and controls; absorb detector residue and remove old session coordination. |
| [resolver_type_name_collision_wall.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/resolver_type_name_collision_wall.dag) | Refactor | Define a resolver guard against conflicting type-name authorities, with scope rules and negative controls that expose ambiguous binding. | Reconcile collision controls with current resolver and source-layout rules; retire the obsolete standalone flag proposal. |
| [self_applying_lenses.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/self_applying_lenses.dag) | Refactor | Define a shared pattern for deriving/applying lens findings and identify consumer work. | Keep the shared producer/applier contract and consumer gaps; derive the lens census instead of repeating it. |
| [serving_fleet_program.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/serving_fleet_program.dag) | Leave | Carry typed fleet deliverables and coordinate serving acceptance without leaking private procurement facts. | Keep typed deliverable, owner, standing and dependency rows; their prose view is already derived. |
| [shell_emission_model.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/shell_emission_model.dag) | Refactor | Provide the surviving model for shell emission as a target and owned foreign execution boundaries. | Make this the single concise shell-intent design; move contracts to effect/target owners and absorb sibling plans. |
| [site_segment_anchor_bmc_reach.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/site_segment_anchor_bmc_reach.dag) | Refactor | Use learned MAC addresses, segment observations and a fenced local holder to resolve and reach BMCs across deployment sites. | Keep site-reach goal, fenced-holder method and observation proof; move durable constraints to segment and endpoint owners. |
| [solve_higher_order_design.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/solve_higher_order_design.dag) | Refactor | Define how higher-order solving should express and resolve constraints through the compiler's shared constraint machinery. | Move enduring solve semantics to v2.std.constraints; preserve numerical capability gaps and retire the shelf narrative. |
| [source_layout_authority.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/source_layout_authority.dag) | Refactor | Make declared module names the sole source-path authority and coordinate corpus migration to that mapping without reverse filesystem inference. | Keep only remaining cutover obligations and proof; let gunbc.source_layout own mappings and remove the historical cookbook. |
| [srv1_deploy_principal_split.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/srv1_deploy_principal_split.dag) | Refactor | Separate srv1 deployment capabilities from operator and root privileges so routine deployment uses only its intended authority. | Keep privilege-separation goal and readback proof; move exact grant/caller boundaries to authentication owners before pruning the narrative. |
| [structural_quadratic_wall_coverage_audit.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/structural_quadratic_wall_coverage_audit.dag) | Refactor | Identify structurally quadratic compiler operations and determine which cost controls actually detect or prevent their repeated work. | Move independent controls and uncovered cost obligations to complexity authorities; remove the audit narrative. |
| [transport_argv_anemia_dissolution.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/transport_argv_anemia_dissolution.dag) | Refactor | Replace duplicated transport command construction with owned operation declarations and shell realization so argument semantics have one authority. | Consolidate with shell_emission_model; move remaining transport-specific obligations to their extdeps owners. |
| [typescript_gap_census.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/typescript_gap_census.dag) | Refactor | Identify missing TypeScript target capabilities and the discriminating acceptance evidence needed to close each realization gap. | Move missing capabilities and controls to TypeScript realization owners; replace the hand census with execution-derived evidence. |
| [v2_compiler_architecture.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/v2_compiler_architecture.dag) | Refactor | Define the compiler's stage decomposition, semantic boundaries and governing relationships so subordinate implementations share one architecture. | Preserve governing stage contracts in compiler owners; reduce the architecture overview and remove duplicated child-plan details. |
| [v2_complexity_capability_parity.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/v2_complexity_capability_parity.dag) | Refactor | Identify native complexity-analysis capabilities needed to match existing checks and specify evidence that distinguishes actual parity from declarations. | Keep the parity objective and proof; move capability gaps to complexity owners and derive their status. |
| [v2_corpus_self_host.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/v2_corpus_self_host.dag) | Refactor | Define whole-corpus native self-host acceptance and the seed obligations that must disappear before the compiler owns its complete path. | Keep whole-corpus self-host objective and discriminating execution proof; retain typed citations while pruning repeated mechanism explanations. |
| [v2_self_hosting.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/v2_self_hosting.dag) | Refactor | Sequence the transition from weak to strong self-hosting so the emitted compiler progressively owns production generation instead of the seed. | Move surviving gates into v2_corpus_self_host and seed-retention carriers; retire the obsolete four-wave tracker. |
| [value_null_split.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/value_null_split.dag) | Refactor | Distinguish language absence from runtime null-like values so lookup, optional values and evaluation refusals retain their intended semantics. | Move remaining runtime-null bridges into current optional/cardinality authorities; retire the old sentinel-split narrative. |

### dag/std/runtime/bin_shims

| File | Bucket | Inferred intention | Reason / next step |
| --- | --- | --- | --- |
| [README.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/std/runtime/bin_shims/README.md) | Delete | Define where binary-shim declarations live, how they are named, and how each references its modeled process entrypoint. | Delete the empty BinShim framework proposal; no carrier, instances or active consumer exist. |

### dag/test/fixture/codex_device_login

| File | Bucket | Inferred intention | Reason / next step |
| --- | --- | --- | --- |
| [README.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/test/fixture/codex_device_login/README.md) | Leave | Ground headless device-login prompt parsing in captured Codex output, including ANSI wrapping, expiry text and capture provenance. | Keep observed fixture provenance and process/version context. |

### dag/test/fixture/codex_provider_events

| File | Bucket | Inferred intention | Reason / next step |
| --- | --- | --- | --- |
| [README.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/test/fixture/codex_provider_events/README.md) | Leave | Ground Codex provider-event parsing in an actual JSON stream so message text and usage fields are modeled from emitted bytes. | Keep fixture provenance distinguishing real provider observations from synthetic expectations. |

### docs/extdeps/lotes_azifa072

| File | Bucket | Inferred intention | Reason / next step |
| --- | --- | --- | --- |
| [README.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/extdeps/lotes_azifa072/README.md) | Refactor | Identify the LGA4926 socket body and document drawing provenance, extracted pad geometry and unresolved manufacturing assumptions. | Keep source provenance and normalization limits; move duplicated attachment facts to extdeps.cpu_attachment.lotes_azifa072. |

### docs/measurements/roster_import_sweep_native

| File | Bucket | Inferred intention | Reason / next step |
| --- | --- | --- | --- |
| [README.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/measurements/roster_import_sweep_native/README.md) | Refactor | Explain how to compare native roster-import sweep output and cost against fixed interpreter results, including path-refusal controls. | Reduce to invocation, fixture schema and current frontier; retain the pinned refusal-progression evidence. |

### docs/measurements/value_position_block_nests

| File | Bucket | Inferred intention | Reason / next step |
| --- | --- | --- | --- |
| [README.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/measurements/value_position_block_nests/README.md) | Leave | Explain fixtures and timing controls that detect repeated lowering of nested value-position blocks and its exponential cost. | Keep scaling fixtures and reproduction instructions used by the native block-nesting control. |

### docs/plans

| File | Bucket | Inferred intention | Reason / next step |
| --- | --- | --- | --- |
| [altra-cross-channel-rank-mixing-survey.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/plans/altra-cross-channel-rank-mixing-survey.md) | Refactor | Distinguish documented Altra DIMM mixing constraints from unanswered cross-channel questions so hardware selection does not treat missing guidance as prohibition. | Move cited facts and unanswered questions beside the memory-population authorities; remove the research narrative from active plans. |
| [cassette-assembly-requirements.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/plans/cassette-assembly-requirements.md) | Refactor | Define what makes a cassette assembly safe and acceptable and what remains to qualify. | Keep one concise cassette goal, assembly method and qualification; carry detailed checks and open obligations in printed_chassis owners. |
| [cassette-board-mechanical-references.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/plans/cassette-board-mechanical-references.md) | Refactor | Collect cited board geometry, coordinate conventions and provisional dimensions needed to model cassette mounting and clearances. | Move source-qualified geometry to board_mechanical_reference; replace this separate note with a link from the assembly contract. |
| [cassette-inventory-convergence-implementation.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/plans/cassette-inventory-convergence-implementation.md) | Refactor | Define how cassette inventory, physical readback and print demands converge while keeping shadow planning separate from printer actuation. | Move operation syntax to one print runbook and cutover obligations to inventory owners; remove the implementation chronicle. |
| [cassette-platforms.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/plans/cassette-platforms.md) | Refactor | Keep multiple board platforms on one cassette generator and expose platform-specific unknowns. | Move platform facts to cassette_profiles and unresolved scope to the assembly contract; retire this separate plan. |
| [cassette-print-day.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/plans/cassette-print-day.md) | Refactor | Coordinate the cassette print queue, replacement parts and physical reservations while preserving uncertain starts and acceptance evidence. | Move remaining reservations and replacement demands into inventory; preserve print receipts and delete the day-plan narrative. |
| [cassette-server-psu-interface.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/plans/cassette-server-psu-interface.md) | Refactor | Choose a shared server-PSU interface for cassette assemblies while making electrical, harness, clearance and thermal qualification explicit. | Move interface and qualification obligations to power_cassette/server_power_interface; retain unresolved selection in the assembly contract. |
| [ci-end-to-end-accounting.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/plans/ci-end-to-end-accounting.md) | Refactor | Measure the full pull-request and merge-group CI cost so reduction work targets observed time and resource consumption. | Retain a compact pinned measurement receipt; move open cost-reduction obligations to current CI owners. |
| [ci-minutes-product-design.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/plans/ci-minutes-product-design.md) | Refactor | Offer GitHub-native CI jobs through selectable runner classes and spend caps while fabric allocation, isolation and billing remain behind that interface. | Keep customer goal, CI interface and validation; carry allocation, isolation and billing obligations in existing fabric/runner owners. |
| [ci-per-job-microvm-migration.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/plans/ci-per-job-microvm-migration.md) | Refactor | Plan moving public and private CI jobs to disposable microVMs with evidence that launch, isolation and teardown satisfy cutover requirements. | Move unique cutover conditions into mtcollins1_ci_service_legs and runner owners; retire this second migration plan. |
| [cli-run-reconcile-defork.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/plans/cli-run-reconcile-defork.md) | Refactor | Unify CLI reconciliation and module resolution so execution shares one authority without repeated whole-corpus work or incomplete cache identities. | Move remaining demand/reconciliation gaps to demand_engine_program and the self-host frontier; retire the historical phase plan. |
| [compiler-guarantee-recovery-gap-analysis.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/plans/compiler-guarantee-recovery-gap-analysis.md) | Refactor | Expose the difference between intended guarantees and actually enforced rungs across acceptance paths. | Move unique guarantee gaps to guarantee_measurement, guarantee_stall and rung_drop; replace the long hand ledger with a concise contract and derived evidence. |
| [dag-scm-design.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/plans/dag-scm-design.md) | Refactor | Define typed source edits, integration and refusal semantics so source-control operations preserve authored intent rather than depend on textual merge heuristics. | Model remaining source-intent integration and refusal contracts first; then keep goal, mechanism and acceptance instead of the long design narrative. |
| [effect-namespace-grants.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/plans/effect-namespace-grants.md) | Refactor | Define scoped read, write and execution authority over namespaces and clarify how reference and publication permissions differ. | Preserve unresolved grant semantics in existing authorization carriers; remove phase journals and obsolete publication experiments. |
| [emission-admission-stage-aware-pipeline-design.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/plans/emission-admission-stage-aware-pipeline-design.md) | Refactor | Define stage and generation-lineage evidence for emitted-code admission so declared compiler transitions can be distinguished from regressions before promotion or reuse. | Model generation-lineage and admission obligations first; retain pending sign-off and promotion refusals while reducing the plan to goal, method and proof. |
| [emit-type-surface-traversal-cost.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/plans/emit-type-surface-traversal-cost.md) | Leave | Document a pinned type-surface traversal experiment and its falsified explanations so performance work starts from discriminating evidence. | Keep the compact pinned experiment and falsified alternatives; no current instrument reproduces this historical evidence. |
| [fabric-recut-program.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/plans/fabric-recut-program.md) | Refactor | Explain separation of work identity, sanction, selection and occupancy through a recut. | Move remaining cut obligations to product.fabric owners; keep concise boundary rationale and remove the recut chronicle. |
| [fabric-storage.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/plans/fabric-storage.md) | Refactor | Define storage contracts and migration/capability gaps used by public and private consumers. | Keep storage goal and acceptance; move interface obligations and capability gaps to existing storage carriers and remove cutover history. |
| [floor-cut-replacement-plan.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/plans/floor-cut-replacement-plan.md) | Refactor | Coordinate replacement of the floor and account for every intentionally lost guarantee. | Retain unique census bounds in the doctrine and live losses in rung_drop; retire the branch playbook. |
| [generated-file-conflict-policy.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/plans/generated-file-conflict-policy.md) | Refactor | Define deterministic handling of generated-output merge conflicts without making generated bytes authoritative. | Keep concise integration rationale; let generated_artifact and commit_workflow own exact conflict policy and readiness. |
| [keying-relation-design.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/plans/keying-relation-design.md) | Refactor | Define identity, dependency and keying relations so demand evaluation can share results without confusing equality, provenance or invalidation. | Model the scoped keying relation and multiplicity obligations first; keep concise rationale without equating keying with existing containers. |
| [layering-imports-reference-repoint-design.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/plans/layering-imports-reference-repoint-design.md) | Delete | Keep the layering gate effective after import syntax disappears by deriving its dependency edges from resolved references. | Delete the retired directional-import proposal and repair references; DESIGN now permits all acyclic imports and current containment consumers retain their contracts. |
| [microvm-launch-displacement-analysis.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/plans/microvm-launch-displacement-analysis.md) | Refactor | Determine which live launcher responsibilities must be replaced, and at what cost, before a modeled executor can take over microVM actuation. | Move unique custody, failure and boot constraints to runner launch owners; retain evidence and remove discovery narrative. |
| [milk-crate-stack-design.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/plans/milk-crate-stack-design.md) | Refactor | Define a stackable cassette frame's topology, service access, airflow and mechanical interfaces for a manufacturable shared chassis. | Move mechanical/service obligations to stack_geometry and stack_mechanics; retain a short assembly overview and retire this separate design. |
| [model-grounding-lens-extract.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/plans/model-grounding-lens-extract.md) | Refactor | Detect modeled concepts lacking valid ownership/grounding, without mechanical retyping of arbitrary strings. | Move remaining detector gaps to grounding and its stall ledger; consolidate the migration into reference_grounding_migration. |
| [namespace-cut-replacement-plan.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/plans/namespace-cut-replacement-plan.md) | Refactor | Coordinate replacing namespace architecture through an ordered cutover that accounts for declaration identities, consumers and intentionally lost guarantees. | Retain unique census lessons and current obligations in namespace_cut carriers; remove the old branch playbook. |
| [namespace-resolution-design.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/plans/namespace-resolution-design.md) | Refactor | Define how names bind across scopes, qualification and visibility so the resolver rejects ambiguity without relying on filesystem accidents. | Model remaining binding/refusal rules in resolver owners; keep a concise contract and remove chronology, stale counts and superseded rules. |
| [namespace-unique-on-chain-operational-plan.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/plans/namespace-unique-on-chain-operational-plan.md) | Refactor | Plan enforcement of name uniqueness along namespace chains through explicit identity edges, migration steps and ambiguous-binding refusal controls. | Move identity edges and discrimination cases beside resolver authorities; retire this separate operational plan. |
| [print-batch-orchestration.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/plans/print-batch-orchestration.md) | Refactor | Turn desired part quantities into packed plates and reconciled printer schedules so retries fulfill remaining demand without duplicate prints. | Move batch release obligations to print_batch_plan and the assembly contract; remove this second plan. |
| [print-workflow-integration.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/plans/print-workflow-integration.md) | Refactor | Explain how an operator drives the print workflow and where physical acceptance is required. | Consolidate into one short printing runbook with modeled entrypoints, prerequisites and physical acceptance; remove recovery history. |
| [replacement-migration-doctrine.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/plans/replacement-migration-doctrine.md) | Refactor | Define how to replace an architectural authority at its root while accounting for consumers, lost guarantees and terminal evidence. | Retire verbatim-retention ruling only after procedure, carve-outs, consumer duties and terminal receipts have owning carriers; keep a short rationale. |
| [roadmap-launch-mvp-plan.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/plans/roadmap-launch-mvp-plan.md) | Refactor | Define the minimum usable roadmap launch experience and the acceptance evidence required for a live operational dashboard. | Keep one end-to-end acceptance scenario; move remaining frontiers to roadmap owners and remove deployment/session history. |
| [roadmap-spawner.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/plans/roadmap-spawner.md) | Refactor | Specify how roadmap tasks are sized, assigned and dispatched into workers with the context and constraints needed for execution. | Move outstanding wire obligations to spawner/dispatch carriers; retain only a concise invocation contract if needed. |
| [roadmap-workspace-remodel-plan.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/plans/roadmap-workspace-remodel-plan.md) | Refactor | Restructure roadmap workspace layout and interactions so operators can inspect work, understand progress and act from one surface. | Move standing operator rulings to design/roadmap owners; consolidate remaining UI acceptance into the launch contract. |
| [roadmap-workspace-ux-plan.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/plans/roadmap-workspace-ux-plan.md) | Refactor | Define the roadmap workspace's navigation, work presentation and operator interactions so task state leads to clear next actions. | Preserve remaining end-to-end checks in launch acceptance; remove superseded UX discussion and sequencing. |
| [runner-throughput-qualification-recut.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/plans/runner-throughput-qualification-recut.md) | Refactor | Measure an independently selected runner offer against its promised throughput without letting qualification mint its own sanction or supply. | Move remaining live-route/run proof to qualification owners and mtcollins1_ci_service_legs; remove the recut narrative. |
| [scaffold-admission-doctrine.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/plans/scaffold-admission-doctrine.md) | Refactor | State when temporary deviations may be admitted and how their end is proved. | Preserve the three admission verdicts, external authorization and actuation boundary in owning carriers; retire verbatim retention, then shorten. |
| [serving-capacity-home-ruling.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/plans/serving-capacity-home-ruling.md) | Refactor | Fix where compute and serving share selection but differ in occupancy authority. | Keep distinct selection/occupancy contracts and receipt links at product.fabric and product.capacity; replace the long census and rationale with a short ruling. |
| [serving-hardware-neutral-cutover.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/plans/serving-hardware-neutral-cutover.md) | Refactor | Replace hardware-specific serving behavior with shared capability and placement models, proving that real callers use the generalized path. | Move remaining consumer obligations to serving_fleet_program; replace the migration census with derived evidence. |
| [shell-intent-emit-realization-design.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/plans/shell-intent-emit-realization-design.md) | Refactor | Separate declared operation intent from shell target realization and physical executor boundaries. | Move unique interface/realization constraints to their owners; consolidate rationale into shell_emission_model. |
| [shell-to-dag-residual-census-and-arc-completion.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/plans/shell-to-dag-residual-census-and-arc-completion.md) | Refactor | Account for every host/shell site and distinguish permanent foreign execution from dissolvable orchestration. | Move surviving per-site obligations to existing frontier/failure carriers; derive the census and retain only a short shell program. |
| [srv1-workspace-vm-bringup.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/plans/srv1-workspace-vm-bringup.md) | Refactor | Record the operator steps and kernel/network readiness required for a workspace VM. | Reduce to a workspace operation runbook; move kernel/TUN prerequisites to commissioning readiness and keep receipt links. |

### docs/probes

| File | Bucket | Inferred intention | Reason / next step |
| --- | --- | --- | --- |
| [leading_minus_continuation_silently_truncates_2026-08-23.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/probes/leading_minus_continuation_silently_truncates_2026-08-23.md) | Leave | Demonstrate how a leading-minus continuation silently changes expression meaning and identify the regression behavior that must be refused. | Keep until an executing regression carrier preserves every reduced case and production citations move; that retirement condition remains unproved. |
| [mtcollins1_2drx4_group_test_prediction_2026-09-12.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/probes/mtcollins1_2drx4_group_test_prediction_2026-09-12.md) | Refactor | Preregister the expected result and falsifiers for isolating four 2DRx4 spare DIMMs within a known Mt. Collins population. | Consolidate with its outcome in one experiment record; preserve the prediction timestamp, configuration and uncertainty. |
| [mtcollins1_2rx4_group_outcome_2026-09-12.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/probes/mtcollins1_2rx4_group_outcome_2026-09-12.md) | Delete | Score the Micron 2Rx4 group prediction against observed power and restart behavior and assess implications for module selection. | Delete the retracted success assessment; the correction retains valid observations and pins the original history. |
| [mtcollins1_2rx4_group_test_prediction_2026-09-12.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/probes/mtcollins1_2rx4_group_test_prediction_2026-09-12.md) | Refactor | Preregister a 2Rx4 DIMM substitution test that compares its behavior with the 2DRx4 quartet in the same socket arrangement. | Consolidate prediction and corrected outcome; preserve their order and distinguish surviving observations from retracted conclusions. |
| [mtcollins1_2rx4_outcome_correction_2026-09-12.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/probes/mtcollins1_2rx4_outcome_correction_2026-09-12.md) | Refactor | Separate observed 2Rx4 survival and console evidence from unsupported training-success claims and identify the boot-delivery defect. | Carry the retraction and valid readings in one experiment record before retiring this separate narrative. |
| [mtcollins1_ampere_smpro_access_paths_2026-09-12.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/probes/mtcollins1_ampere_smpro_access_paths_2026-09-12.md) | Refactor | Identify documented Ampere pre-OS observation interfaces and order probes that distinguish sourced register meanings from local inference. | Move upstream interfaces to extdeps and access/readback requirements to machine_intake; retire the research narrative. |
| [mtcollins1_dram_training_root_cause_2026-09-12.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/probes/mtcollins1_dram_training_root_cause_2026-09-12.md) | Refactor | Explain the firmware-reported DIMM package mismatch using captured console evidence and bound its implications for fleet memory qualification. | Consolidate with memory evaluation records; preserve distinct observations and unresolved causes in their modeled owners. |
| [mtcollins1_memory_census_and_boot_delivery_2026-09-11.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/probes/mtcollins1_memory_census_and_boot_delivery_2026-09-11.md) | Refactor | Report Mt. Collins DIMM population and load-backed memory health while tracing the boot-delivery failures encountered during intake. | Retain a compact incident receipt; move remaining census and boot-delivery requirements to machine_intake owners. |
| [mtcollins1_micron_module_screen_state_2026-09-12.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/probes/mtcollins1_micron_module_screen_state_2026-09-12.md) | Refactor | Consolidate tested DIMM configurations, eliminated hypotheses and evidence limits so subsequent memory investigation does not repeat prior experiments. | Move unique configuration/outcome rows to memory evaluation; remove the resolved interim screen-state narrative. |
| [mtcollins1_sel_oem_payload_census_2026-09-12.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/probes/mtcollins1_sel_oem_payload_census_2026-09-12.md) | Refactor | Count OEM payload families across the complete Mt. Collins event log to test co-emission patterns and constrain interpretations. | Move observed facts to machine_intake receipts; keep only artifact, method and interpretation limits. |
| [mtcollins1_socket_swap_outcome_2026-09-12.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/probes/mtcollins1_socket_swap_outcome_2026-09-12.md) | Refactor | Compare the socket-swap prediction with observed restart timing and payload changes to test whether the fault follows the DIMM quartet. | Consolidate with its prediction in one typed experiment record; preserve configuration, outcome and observation limits. |
| [mtcollins1_socket_swap_prediction_2026-09-12.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/probes/mtcollins1_socket_swap_prediction_2026-09-12.md) | Refactor | Preregister whether moving the 2DRx4 quartet between sockets changes the reported fault location while preserving the failure. | Consolidate with the socket-swap outcome while preserving the prediction timestamp and original uncertainty. |
| [mtcollins1_spare_module_screen_prediction_2026-09-11.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/probes/mtcollins1_spare_module_screen_prediction_2026-09-11.md) | Leave | Preregister the all-spare 16-DIMM screening prediction, configuration and falsifiers before testing memory on Mt. Collins. | Keep the prediction artifact explicitly referenced by machine_intake; preserve its timestamp and limits. |
| [mtcollins_memory_multiple_findings_2026-09-12.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/probes/mtcollins_memory_multiple_findings_2026-09-12.md) | Refactor | Represent multiple firmware memory-mixture observations without generalizing specific byte mismatches into an unsupported universal compatibility rule. | Consolidate into one compact memory experiment receipt; keep independent findings separately typed. |
| [pre_os_refusal_consumer_sweep.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/probes/pre_os_refusal_consumer_sweep.md) | Delete | Trace pre-OS observation refusals from producers into their first consumers so assessment and replay preserve each failure distinction. | Delete the completed PR-specific review map; pre_os carriers, controls and PR11211 preserve its useful content. |
| [prospective_memory_change_evaluation_2026-09-12.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/probes/prospective_memory_change_evaluation_2026-09-12.md) | Refactor | Explain how proposed memory changes are assessed against positioned populations, firmware evidence and independent qualification obligations before purchasing or installation. | Move remaining obligations into memory_configuration_evaluation and memory_change_evaluation; remove the standalone implementation diary. |

### docs/probes/realization_census_1dd3eda5_2026-10-24

| File | Bucket | Inferred intention | Reason / next step |
| --- | --- | --- | --- |
| [README.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/probes/realization_census_1dd3eda5_2026-10-24/README.md) | Refactor | Explain a pinned census of where modules choose storage, caching, access, authorization and placement instead of receiving owned decisions. | Keep scope, source pin, schemas and caveats; remove repeated result tables already carried by the census files. |
| [method.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/probes/realization_census_1dd3eda5_2026-10-24/method.md) | Leave | Describe the census sampling, classification and reconciliation rules so its counts, comparisons and uncertainty can be audited. | Keep the census inputs, methodology and receipt used by realization_census_rows_gen and its replacement-lens oracle. |

### docs/probes/srv1_nvfp4_serving_2026-10-25

| File | Bucket | Inferred intention | Reason / next step |
| --- | --- | --- | --- |
| [srv1_nvfp4_serving_verification_2026-10-25.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/probes/srv1_nvfp4_serving_2026-10-25/srv1_nvfp4_serving_verification_2026-10-25.md) | Leave | Verify srv1's loaded NVFP4 weights and FP8 KV configuration while measuring correctness, latency, throughput and context limits against observed engine identity. | Keep the serving verification and raw observations explicitly retained by srv1_serving_startup_receipt. |

### docs/recovered/mtcollins-host-image-source

| File | Bucket | Inferred intention | Reason / next step |
| --- | --- | --- | --- |
| [README.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/recovered/mtcollins-host-image-source/README.md) | Refactor | Explain the provenance and behavior of recovered Mt. Collins host-image tooling so its modeled replacement can be checked against actual source. | Retain provenance, source inventory, non-execution constraint and exact replacement trigger; remove repeated migration narrative. |

### docs/runbooks

| File | Bucket | Inferred intention | Reason / next step |
| --- | --- | --- | --- |
| [README.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/runbooks/README.md) | Refactor | Provide a single entrypoint for finding infrastructure procedures covering BMC access, host boot, credentials and privileged setup. | Replace the stale overview with a compact index of current operational entrypoints. |
| [bmc-assimilator-wif-setup.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/runbooks/bmc-assimilator-wif-setup.md) | Refactor | Guide configuring GCP workload identity so unattended BMC assimilation obtains scoped tokens without manually pasted credentials. | Replace manual setup narrative with modeled bootstrap/delegation entrypoints and necessary operator steps. |
| [bmc-redfish-operator-access.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/runbooks/bmc-redfish-operator-access.md) | Refactor | Guide enabling and verifying out-of-band Redfish telemetry on srv1 and srv2 through their dedicated BMC interfaces. | Consolidate credential/readback operations under fleet-access owners; retain only essential operator bootstrap instructions. |
| [gcp-iam-root-ceremony-apply.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/runbooks/gcp-iam-root-ceremony-apply.md) | Leave | Guide the privileged IAM bootstrap ceremony that establishes missing trust-root cells and verifies them before fleet convergence resumes. | Keep the unique operator bootstrap ceremony used by the fleet-secret accessor path. |
| [mtcollins1-ssh-host-key-mint.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/runbooks/mtcollins1-ssh-host-key-mint.md) | Leave | Guide minting, delivering and anchoring mtcollins1's stable SSH host key so transport identity remains verifiable across diskless boots. | Keep the host-key ceremony; its command block is checked against the modeled producer. |
| [srv3-nbd-proxy-ws-upgrade-dry-run.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/runbooks/srv3-nbd-proxy-ws-upgrade-dry-run.md) | Refactor | Guide an authorized websocket probe that determines whether srv3 supports NBD-proxy virtual media before choosing its installation path. | Move the observed capability to its receipt owner, repair the hardware citation and retire the superseded dry-run procedure. |
| [srv3-os-install-actuate.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/runbooks/srv3-os-install-actuate.md) | Refactor | Guide authorized srv3 installation through seeded virtual media, console observation and OS-up evidence while respecting destructive-step gates. | Keep the current modeled entrypoint, operator prerequisites and diagnostic outcomes; remove obsolete procedural narration. |

### fixtures/census_scoped_acquisition

| File | Bucket | Inferred intention | Reason / next step |
| --- | --- | --- | --- |
| [README.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/fixtures/census_scoped_acquisition/README.md) | Refactor | Explain acquisition fixtures that distinguish missing and malformed modules, shared dependencies and requester-specific permissions without falsely completing a closure. | Replace obsolete layout claims with the actual namespace-admission controls in v2.std.cross_tree.import_model. |

### provider-runtime/codex

| File | Bucket | Inferred intention | Reason / next step |
| --- | --- | --- | --- |
| [README.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/provider-runtime/codex/README.md) | Leave | Explain regenerating the pinned Codex package lock and verifying downloaded archives before provider-runtime materialization trusts their contents. | Keep the concise pinned installation and regeneration contract. |

### sites

| File | Bucket | Inferred intention | Reason / next step |
| --- | --- | --- | --- |
| [README.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/sites/README.md) | Leave | Orient maintainers to the existing chassis review site and its generated meshes so browser interactions never become mechanical geometry authority. | Keep the short operational site/mesh entrypoint. |

### src/v2

| File | Bucket | Inferred intention | Reason / next step |
| --- | --- | --- | --- |
| [LAYOUT.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/src/v2/LAYOUT.md) | Delete | Specify how v2 module identities map to source directories, filenames and test locations for authors and tooling. | Delete obsolete CI and test-path rules; gunbc.source_layout and DESIGN own the current mapping. |

### src/v2/lens/testdata

| File | Bucket | Inferred intention | Reason / next step |
| --- | --- | --- | --- |
| [README.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/src/v2/lens/testdata/README.md) | Delete | Explain the anemia-lens judgment corpus, its labeled inputs and how to score decisions without confusing examples with live findings. | Delete the unconsumed historical evaluator guide with its unused companion corpus; preserve provenance in PR5302 and repair references. |

### test/compiler_gate_contract_stamp/fixtures

| File | Bucket | Inferred intention | Reason / next step |
| --- | --- | --- | --- |
| [README.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/test/compiler_gate_contract_stamp/fixtures/README.md) | Leave | Explain a captured silent-green workflow fixture used to detect evidence instructions and compiler subjects resolved from different revisions. | Keep negative-control fixture provenance for the silent-green regression. |

### test/namecheap

| File | Bucket | Inferred intention | Reason / next step |
| --- | --- | --- | --- |
| [README.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/test/namecheap/README.md) | Leave | Guide fixture-only checks that verify Namecheap query transport keeps credentials out of argv and emitted workflows consume the declared receipt. | Keep concise test invocation and credential/input requirements. |

### test/native_filesystem

| File | Bucket | Inferred intention | Reason / next step |
| --- | --- | --- | --- |
| [README.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/test/native_filesystem/README.md) | Leave | Explain native source-root acquisition controls for filesystem refusals, complete traversal and effectful tail-call lowering through emitted services. | Keep producer integration controls and source-root isolation instructions. |

### test/primitive_runtime_body

| File | Bucket | Inferred intention | Reason / next step |
| --- | --- | --- | --- |
| [README.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/test/primitive_runtime_body/README.md) | Refactor | Explain how the primitive-runtime-body control checks runtime registry names against emitted bodies and detects missing implementations with a negative specimen. | Remove retired required-lane claims; retain the callable checker, invocation and qualification provenance. |

<details>
<summary>Supporting files (47)</summary>

### dag/gunbc/plans

| File | Bucket | Inferred intention | Reason / next step |
| --- | --- | --- | --- |
| [demand_restatement_follow_up.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/demand_restatement_follow_up.dag) | Leave | Enumerate retained authored service-demand rows by exact identity and the condition that permits each redundant declaration to disappear. | Keep executable retained-row identities and per-row triggers; this is structured data rather than a prose plan. |
| [md_helpers.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/md_helpers.dag) | Leave | Provide reusable Markdown constructors so plan modules can render headings, paragraphs, lists, quotations and tables consistently. | Keep shared Markdown constructors; this is implementation rather than documentation prose. |

### docs/extdeps/asrock_rack

| File | Bucket | Inferred intention | Reason / next step |
| --- | --- | --- | --- |
| [ALTRAD8UD_standoff_grid.png](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/extdeps/asrock_rack/ALTRAD8UD_standoff_grid.png) | Leave | Show the ALTRAD8UD motherboard standoff arrangement as a mechanical reference for locating chassis supports. | Keep the operator-supplied source figure grounding board-hole geometry; it is not a generated duplicate. |

### docs/extdeps/lotes_azifa072

| File | Bucket | Inferred intention | Reason / next step |
| --- | --- | --- | --- |
| [lga4926_pad_array.svg](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/extdeps/lotes_azifa072/lga4926_pad_array.svg) | Refactor | Visualize the 4,926 extracted LGA4926 pad centers so a reader can inspect the array's shape and spacing. | Reconcile the SVG/CSV coordinate mismatch and preserve any independent extraction evidence in extdeps.cpu_attachment.lotes_azifa072 before retiring this rendering and its link. |
| [lga4926_pcb_pads.csv](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/extdeps/lotes_azifa072/lga4926_pcb_pads.csv) | Leave | Store normalized LGA4926 PCB pad-center coordinates extracted from the cited socket drawing for geometric checks and modeling. | Keep the full pad-coordinate source consumed by extdeps.cpu_attachment.lotes_azifa072. |

### docs/measurements/roster_import_sweep_native

| File | Bucket | Inferred intention | Reason / next step |
| --- | --- | --- | --- |
| [manifest.txt](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/measurements/roster_import_sweep_native/manifest.txt) | Leave | Enumerate native roster-sweep test cases and source paths so the isolated child receives a fixed, reproducible subject set. | Keep active instrument manifest and expected-output fixtures used by roster_import_sweep_native and its controls. |

### docs/measurements/roster_import_sweep_native/expected

| File | Bucket | Inferred intention | Reason / next step |
| --- | --- | --- | --- |
| [lexical_scope_sweep.txt](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/measurements/roster_import_sweep_native/expected/lexical_scope_sweep.txt) | Leave | Specify the exact import edit expected when the roster sweep processes the lexical-scope fixture. | Keep active instrument manifest and expected-output fixtures used by roster_import_sweep_native and its controls. |
| [node_dag_sweep.txt](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/measurements/roster_import_sweep_native/expected/node_dag_sweep.txt) | Leave | Specify the empty edit output expected when the roster sweep processes the node module. | Keep active instrument manifest and expected-output fixtures used by roster_import_sweep_native and its controls. |
| [refuse_escaped_path.txt](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/measurements/roster_import_sweep_native/expected/refuse_escaped_path.txt) | Leave | Specify the exact refusal expected when a roster-sweep subject path escapes the repository boundary. | Keep active instrument manifest and expected-output fixtures used by roster_import_sweep_native and its controls. |

### docs/measurements/value_position_block_nests/multi_d10/dag/gunbc

| File | Bucket | Inferred intention | Reason / next step |
| --- | --- | --- | --- |
| [t.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/measurements/value_position_block_nests/multi_d10/dag/gunbc/t.dag) | Leave | Provide a depth-10 nested-match fixture with multi-statement value blocks to measure repeated-lowering cost in the native compiler. | Keep scaling fixtures and reproduction instructions used by the native block-nesting control. |

### docs/measurements/value_position_block_nests/multi_d14/dag/gunbc

| File | Bucket | Inferred intention | Reason / next step |
| --- | --- | --- | --- |
| [t.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/measurements/value_position_block_nests/multi_d14/dag/gunbc/t.dag) | Leave | Provide a depth-14 nested-match fixture with multi-statement value blocks to measure repeated-lowering cost in the native compiler. | Keep scaling fixtures and reproduction instructions used by the native block-nesting control. |

### docs/measurements/value_position_block_nests/multi_d20/dag/gunbc

| File | Bucket | Inferred intention | Reason / next step |
| --- | --- | --- | --- |
| [t.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/measurements/value_position_block_nests/multi_d20/dag/gunbc/t.dag) | Leave | Provide a depth-20 nested-match fixture with multi-statement value blocks to measure repeated-lowering cost in the native compiler. | Keep scaling fixtures and reproduction instructions used by the native block-nesting control. |

### docs/measurements/value_position_block_nests/single_d10/dag/gunbc

| File | Bucket | Inferred intention | Reason / next step |
| --- | --- | --- | --- |
| [t.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/measurements/value_position_block_nests/single_d10/dag/gunbc/t.dag) | Leave | Provide a depth-10 nested-match fixture with single-statement value blocks to measure repeated-lowering cost in the native compiler. | Keep scaling fixtures and reproduction instructions used by the native block-nesting control. |

### docs/measurements/value_position_block_nests/single_d14/dag/gunbc

| File | Bucket | Inferred intention | Reason / next step |
| --- | --- | --- | --- |
| [t.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/measurements/value_position_block_nests/single_d14/dag/gunbc/t.dag) | Leave | Provide a depth-14 nested-match fixture with single-statement value blocks to measure repeated-lowering cost in the native compiler. | Keep scaling fixtures and reproduction instructions used by the native block-nesting control. |

### docs/measurements/value_position_block_nests/single_d20/dag/gunbc

| File | Bucket | Inferred intention | Reason / next step |
| --- | --- | --- | --- |
| [t.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/measurements/value_position_block_nests/single_d20/dag/gunbc/t.dag) | Leave | Provide a depth-20 nested-match fixture with single-statement value blocks to measure repeated-lowering cost in the native compiler. | Keep scaling fixtures and reproduction instructions used by the native block-nesting control. |

### docs/plans

| File | Bucket | Inferred intention | Reason / next step |
| --- | --- | --- | --- |
| [milk-crate-topology.svg](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/plans/milk-crate-topology.svg) | Delete | Illustrate the milk-crate frame, cassette arrangement and service access as a conceptual topology rather than manufacturing geometry. | Delete the unscaled conceptual duplicate and its image link; stack models and the assembly contract retain the requirements. |

### docs/probes

| File | Bucket | Inferred intention | Reason / next step |
| --- | --- | --- | --- |
| [mtcollins_bound_attempt_host_capture_2026-09-07.txt](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/probes/mtcollins_bound_attempt_host_capture_2026-09-07.txt) | Leave | Record host serial evidence whose unique runner registration binds one physical boot attempt to its GitHub job. | Keep the raw observation explicitly consumed by current runner receipt or version-check carriers. |
| [mtcollins_checkout_and_js_action_2026-09-07.txt](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/probes/mtcollins_checkout_and_js_action_2026-09-07.txt) | Leave | Capture checkout and JavaScript-action execution inside a filtered microVM, including actual network traffic and idle-attempt metering. | Keep the raw observation explicitly consumed by current runner receipt or version-check carriers. |
| [mtcollins_first_job_host_capture_2026-09-07.txt](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/probes/mtcollins_first_job_host_capture_2026-09-07.txt) | Leave | Provide serial boot and launch evidence for the first Mt. Collins microVM CI job and its modeled qualification receipt. | Keep the raw observation explicitly consumed by current runner receipt or version-check carriers. |
| [mtcollins_jit_absent_launch_defect_2026-09-07.txt](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/probes/mtcollins_jit_absent_launch_defect_2026-09-07.txt) | Leave | Expose the mistaken build-label join between a successful job and a credential-less boot, including the harmful power-cycle sequence. | Keep the raw observation explicitly consumed by current runner receipt or version-check carriers. |
| [mtcollins_signed_envelope_2026-09-07.txt](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/probes/mtcollins_signed_envelope_2026-09-07.txt) | Leave | Capture valid and impostor envelope checks on the same host path to demonstrate admission and refusal before microVM resource allocation. | Keep the raw observation explicitly consumed by current runner receipt or version-check carriers. |
| [mtcollins_two_attempts_one_boot_2026-09-07.txt](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/probes/mtcollins_two_attempts_one_boot_2026-09-07.txt) | Leave | Record two isolated microVM jobs on one physical boot and the resource readbacks needed to judge teardown between attempts. | Keep the raw observation explicitly consumed by current runner receipt or version-check carriers. |

### docs/probes/realization_census_1dd3eda5_2026-10-24

| File | Bucket | Inferred intention | Reason / next step |
| --- | --- | --- | --- |
| [classes.tsv](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/probes/realization_census_1dd3eda5_2026-10-24/classes.tsv) | Leave | Group realization-decision findings by recurring class, linking each class to its owning authority, enforcement gap and next proof. | Keep the census inputs, methodology and receipt used by realization_census_rows_gen and its replacement-lens oracle. |
| [findings.tsv](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/probes/realization_census_1dd3eda5_2026-10-24/findings.tsv) | Leave | Locate individual realization decisions by module and symbol, retaining concrete specimens and the shared authority each decision duplicates or lacks. | Keep the census inputs, methodology and receipt used by realization_census_rows_gen and its replacement-lens oracle. |
| [merge_report.txt](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/probes/realization_census_1dd3eda5_2026-10-24/merge_report.txt) | Leave | Record how census slices were reconciled, including duplicate handling, relabeling and findings without supporting instance rows. | Keep the census inputs, methodology and receipt used by realization_census_rows_gen and its replacement-lens oracle. |
| [modules.tsv](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/probes/realization_census_1dd3eda5_2026-10-24/modules.tsv) | Leave | Record each inspected module's realization-decision classifications across five dimensions and link its worst finding to concrete evidence. | Keep the census inputs, methodology and receipt used by realization_census_rows_gen and its replacement-lens oracle. |
| [receipt.json](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/probes/realization_census_1dd3eda5_2026-10-24/receipt.json) | Leave | Bind the realization census to its source revision, scripts, populations and reviewer-agreement measurements for reproducible verification. | Keep the census inputs, methodology and receipt used by realization_census_rows_gen and its replacement-lens oracle. |

### docs/probes/srv1_nvfp4_serving_2026-10-25

| File | Bucket | Inferred intention | Reason / next step |
| --- | --- | --- | --- |
| [raw_arm_fp8_kv_flash_attn.json](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/probes/srv1_nvfp4_serving_2026-10-25/raw_arm_fp8_kv_flash_attn.json) | Leave | Record initial srv1 serving-task outputs and latency measurements under the FP8 KV attention test arm. | Keep the serving verification and raw observations explicitly retained by srv1_serving_startup_receipt. |
| [raw_cold.json](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/probes/srv1_nvfp4_serving_2026-10-25/raw_cold.json) | Leave | Record cold-prefill throughput and context-limit refusal responses for large prompts sent to the srv1 NVFP4 endpoint. | Keep the serving verification and raw observations explicitly retained by srv1_serving_startup_receipt. |
| [raw_eval3.json](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/probes/srv1_nvfp4_serving_2026-10-25/raw_eval3.json) | Leave | Record srv1 evaluation responses, answer hashes and token timings so generated answers can be distinguished from truncated initial samples. | Keep the serving verification and raw observations explicitly retained by srv1_serving_startup_receipt. |
| [raw_eval_oracle.json](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/probes/srv1_nvfp4_serving_2026-10-25/raw_eval_oracle.json) | Leave | Preserve complete srv1 evaluation answers and independent correctness judgments for rechecking the serving test's accuracy claims. | Keep the serving verification and raw observations explicitly retained by srv1_serving_startup_receipt. |
| [raw_eval_oracle_fix.json](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/probes/srv1_nvfp4_serving_2026-10-25/raw_eval_oracle_fix.json) | Leave | Record the palindrome-test answer judged by a form-independent predicate rather than the first oracle's label-based check. | Keep the serving verification and raw observations explicitly retained by srv1_serving_startup_receipt. |
| [raw_longctx3.jsonl](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/probes/srv1_nvfp4_serving_2026-10-25/raw_longctx3.jsonl) | Leave | Record srv1 prefill timings and acceptance responses across increasing context lengths to measure the endpoint's practical input bounds. | Keep the serving verification and raw observations explicitly retained by srv1_serving_startup_receipt. |

### docs/recovered/mtcollins-host-image-source

| File | Bucket | Inferred intention | Reason / next step |
| --- | --- | --- | --- |
| [build.sh.bak.recovered](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/recovered/mtcollins-host-image-source/build.sh.bak.recovered) | Leave | Preserve the backup host-image build script as evidence of how the initramfs and kernel build were assembled. | Keep frozen recovered bytes until modeled operations reproduce the image without these sources; the replacement condition remains unproved. |
| [build.sh.recovered](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/recovered/mtcollins-host-image-source/build.sh.recovered) | Leave | Preserve the host-image build script that assembles initramfs contents and builds the kernel used for Mt. Collins runner boots. | Keep frozen recovered bytes until modeled operations reproduce the image without these sources; the replacement condition remains unproved. |
| [envelope.sh.recovered](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/recovered/mtcollins-host-image-source/envelope.sh.recovered) | Leave | Preserve the script packaging the runner kernel, guest artifacts and verifier into boot media readable by the BMC controller. | Keep frozen recovered bytes until modeled operations reproduce the image without these sources; the replacement condition remains unproved. |
| [initramfs.spec.in](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/recovered/mtcollins-host-image-source/initramfs.spec.in) | Leave | Specify initramfs files, modes and device nodes so the recovered host image starts with a usable console and runtime tree. | Keep frozen recovered bytes until modeled operations reproduce the image without these sources; the replacement condition remains unproved. |
| [udhcpc.script.recovered](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/recovered/mtcollins-host-image-source/udhcpc.script.recovered) | Leave | Preserve the DHCP lease hook that records and applies address, subnet and routing information to the runner host. | Keep frozen recovered bytes until modeled operations reproduce the image without these sources; the replacement condition remains unproved. |
| [vm.json](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/recovered/mtcollins-host-image-source/vm.json) | Leave | Preserve the Firecracker guest configuration selecting its kernel, read-only root disk, boot arguments, virtual CPU count and memory size. | Keep frozen recovered bytes until modeled operations reproduce the image without these sources; the replacement condition remains unproved. |

### docs/recovered/mtcollins-host-image-source/cp

| File | Bucket | Inferred intention | Reason / next step |
| --- | --- | --- | --- |
| [cp-signing.pub](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/recovered/mtcollins-host-image-source/cp/cp-signing.pub) | Leave | Preserve the control-plane public key used by recovered host-image tooling to verify signed runner launch envelopes. | Keep frozen recovered bytes until modeled operations reproduce the image without these sources; the replacement condition remains unproved. |

### docs/recovered/mtcollins-host-image-source/guest

| File | Bucket | Inferred intention | Reason / next step |
| --- | --- | --- | --- |
| [gunbc-probe.sh.recovered](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/recovered/mtcollins-host-image-source/guest/gunbc-probe.sh.recovered) | Leave | Preserve the guest PID-1 network probe used to isolate filtered-egress observations from unrelated operating-system traffic. | Keep frozen recovered bytes until modeled operations reproduce the image without these sources; the replacement condition remains unproved. |
| [gunbc-runner-init.sh.recovered](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/recovered/mtcollins-host-image-source/guest/gunbc-runner-init.sh.recovered) | Leave | Preserve guest PID-1 initialization that mounts filesystems, configures networking and starts the CI runner from supplied job credentials. | Keep frozen recovered bytes until modeled operations reproduce the image without these sources; the replacement condition remains unproved. |

### docs/recovered/mtcollins-host-image-source/initramfs

| File | Bucket | Inferred intention | Reason / next step |
| --- | --- | --- | --- |
| [init.pre-supervisor.recovered](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/recovered/mtcollins-host-image-source/initramfs/init.pre-supervisor.recovered) | Leave | Preserve the host-init generation before supervision so jailer launch behavior can be compared with later attempt lifecycle handling. | Keep frozen recovered bytes until modeled operations reproduce the image without these sources; the replacement condition remains unproved. |
| [init.pre-verify.recovered](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/recovered/mtcollins-host-image-source/initramfs/init.pre-verify.recovered) | Leave | Preserve the host-init generation before artifact verification so the introduction of boot and guest integrity checks remains inspectable. | Keep frozen recovered bytes until modeled operations reproduce the image without these sources; the replacement condition remains unproved. |
| [init.prev.recovered](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/recovered/mtcollins-host-image-source/initramfs/init.prev.recovered) | Leave | Preserve the initial host-boot probe that reports image identity, network-driver binding and KVM availability through the serial console. | Keep frozen recovered bytes until modeled operations reproduce the image without these sources; the replacement condition remains unproved. |
| [init.recovered](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/recovered/mtcollins-host-image-source/initramfs/init.recovered) | Leave | Preserve the host-init template implementing runner-attempt preparation, authenticated admission, jailer launch, observation and teardown for modeled replacement. | Keep frozen recovered bytes until modeled operations reproduce the image without these sources; the replacement condition remains unproved. |

### docs/recovered/mtcollins-host-image-source/irfs

| File | Bucket | Inferred intention | Reason / next step |
| --- | --- | --- | --- |
| [init.recovered](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/recovered/mtcollins-host-image-source/irfs/init.recovered) | Leave | Preserve the checkout-2 instantiation of the host-init template so deployed build-identity substitutions can be compared with their source. | Keep frozen recovered bytes until modeled operations reproduce the image without these sources; the replacement condition remains unproved. |

</details>
