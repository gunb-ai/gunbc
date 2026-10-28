# Document intentions

Review snapshot of [gunbc at d919d662ddf5](https://github.com/gunb-ai/gunbc/tree/d919d662ddf59813f9dab24ca3b25ed5b54ec236), surveyed 2026-10-28.

One inferred purpose per file, for deciding what to delete, prune/consolidate, or leave. Intent does not establish implementation status.

Scope: tracked Markdown, documentation HTML, modeled plans and substantive text notes. Supporting files are listed separately; unrelated raw logs, test fixtures and license texts are outside this review.

## Documents and designs (158)

### Repository root

| File | Intention |
| --- | --- |
| [CLAUDE.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/CLAUDE.md) | Expose the governing gunbc design document through the agent instruction entrypoint without maintaining a separate policy text. |
| [DESIGN.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/DESIGN.md) | Establish gunbc's governing design principles, semantic authorities and proof obligations so authors build consistently toward the substrate model. |
| [LICENSING.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/LICENSING.md) | Explain gunbc's permitted uses, commercial licensing threshold and release conversion terms in plain language alongside the controlling license. |
| [README.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/README.md) | Expose the governing gunbc design document through the repository's standard reader entrypoint without maintaining a separate overview. |

### apps/approve-ios

| File | Intention |
| --- | --- |
| [README.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/apps/approve-ios/README.md) | Guide building and configuring the iOS approval app while keeping its wire protocol and generated Swift tied to modeled authorities. |

### apps/approve-ios/Approve/Assets.xcassets/AppIcon.appiconset

| File | Intention |
| --- | --- |
| [README.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/apps/approve-ios/Approve/Assets.xcassets/AppIcon.appiconset/README.md) | Identify the approval app icon's geometry, colors and generation provenance so the raster remains traceable to the brand mark. |

### artifacts/bmc

| File | Intention |
| --- | --- |
| [mtcollins1-bmc-user3-route-2026-09-13.txt](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/artifacts/bmc/mtcollins1-bmc-user3-route-2026-09-13.txt) | Explain the replacement-admin credential sequence, ordering failure and channel coverage needed to distinguish restricted factory access from true credential retirement. |
| [mtcollins1-topology-control-known-good-socket0-2026-09-12.CORRECTION.txt](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/artifacts/bmc/mtcollins1-topology-control-known-good-socket0-2026-09-12.CORRECTION.txt) | Retract the topology-control success claim and explain why quiet event logs cannot establish memory training or a successful boot. |
| [mtcollins1-user2-retire-2026-09-13.txt](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/artifacts/bmc/mtcollins1-user2-retire-2026-09-13.txt) | Explain how password rotation and discriminating readbacks establish factory-credential retirement while keeping the disabled account's channel permissions unchanged. |

### artifacts/bmc/altrad8ud-1l2t-6-inband-recovery-2026-10-16

| File | Intention |
| --- | --- |
| [00_README.txt](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/artifacts/bmc/altrad8ud-1l2t-6-inband-recovery-2026-10-16/00_README.txt) | Distinguish captured evidence from unsaved session observations in ALTRAD8UD BMC recovery so later models cite each claim at its actual strength. |
| [01_summary.txt](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/artifacts/bmc/altrad8ud-1l2t-6-inband-recovery-2026-10-16/01_summary.txt) | Explain the in-band IPMI credential-recovery route, factory-reset behavior and readback needed to return an inaccessible ALTRAD8UD BMC to onboarding. |

### artifacts/bmc/mtcollins1-32dimm

| File | Intention |
| --- | --- |
| [operator-scripts.txt](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/artifacts/bmc/mtcollins1-32dimm/operator-scripts.txt) | Preserve the operator scripts used to stimulate boots and collect console, power-cycle and event-log observations during the 32-DIMM investigation. |

### artifacts/bmc/mtcollins1-virtual-media-readonly-probe-192.168.1.228-20261027T005424Z

| File | Intention |
| --- | --- |
| [ARGV.txt](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/artifacts/bmc/mtcollins1-virtual-media-readonly-probe-192.168.1.228-20261027T005424Z/ARGV.txt) | Document the anonymous virtual-media surface probe, its exact requests and authority limits so observations can be reproduced without controller mutations. |
| [META.txt](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/artifacts/bmc/mtcollins1-virtual-media-readonly-probe-192.168.1.228-20261027T005424Z/META.txt) | Bound the first virtual-media surface reading by its observer, timing, transient-network retry and concurrent-state caveats rather than implying settled media readiness. |

### artifacts/bmc/mtcollins1-virtual-media-readonly-probe-192.168.1.228-20261027T040012Z

| File | Intention |
| --- | --- |
| [ARGV.txt](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/artifacts/bmc/mtcollins1-virtual-media-readonly-probe-192.168.1.228-20261027T040012Z/ARGV.txt) | Explain the instrumented virtual-media probe's corrected requests, separate server responses and redaction procedure so its evidence remains reproducible and attributable. |
| [META.txt](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/artifacts/bmc/mtcollins1-virtual-media-readonly-probe-192.168.1.228-20261027T040012Z/META.txt) | Interpret the instrumented BMC surface reading using separate clocks and corroborated unit identity while preserving uncertainty about authenticated Redfish access. |

### artifacts/bmc/mtjade1-j3-media-attempt-192.168.1.246-20261006T185027Z

| File | Intention |
| --- | --- |
| [SEQUENCE.txt](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/artifacts/bmc/mtjade1-j3-media-attempt-192.168.1.246-20261006T185027Z/SEQUENCE.txt) | Reconstruct the Mt. Jade virtual-media incident, separating authorized actions, stop-condition violations and restoration readbacks without claiming a verified attachment. |

### artifacts/bmc/srv13-first-contact-2026-10-07

| File | Intention |
| --- | --- |
| [FINDINGS.txt](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/artifacts/bmc/srv13-first-contact-2026-10-07/FINDINGS.txt) | Explain srv13 intake findings across firmware, boot configuration and thermal failures, linking each interpretation to captures that constrain the next investigation. |

### artifacts/printing/process-observation-2026-10-05

| File | Intention |
| --- | --- |
| [README.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/artifacts/printing/process-observation-2026-10-05/README.md) | Compare failed and recovery fan-print archive commands while distinguishing process observations from geometric explanations and physical acceptance. |

### artifacts/printing/rear-posts-2026-10-05

| File | Intention |
| --- | --- |
| [INCIDENT.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/artifacts/printing/rear-posts-2026-10-05/INCIDENT.md) | Document the upright fan-print failure and its evidence so telemetry completion cannot be mistaken for a successful physical assembly. |

### artifacts/spark/glm-group-b-2026-09-16

| File | Intention |
| --- | --- |
| [README.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/artifacts/spark/glm-group-b-2026-09-16/README.md) | Identify raw GLM container and NCCL captures used to compare socket and RoCE transport configurations on Group B hosts. |

### artifacts/spark/glm-group-b-2026-09-16/narrow

| File | Intention |
| --- | --- |
| [README.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/artifacts/spark/glm-group-b-2026-09-16/narrow/README.md) | Explain narrowed GLM container observations and distinguish fresh daemon responses from fields projected out of archived RoCE captures. |

### dag/examples/blackjack

| File | Intention |
| --- | --- |
| [README.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/examples/blackjack/README.md) | Teach Dag modeling through deterministic Blackjack rounds, seeded simulation, explicit refusals and replayable test evidence. |

### dag/examples/weather

| File | Intention |
| --- | --- |
| [README.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/examples/weather/README.md) | Demonstrate Dag types, pattern matching and collection pipelines by compiling a self-contained weather model into type-checked Rust. |

### dag/gunbc/plans

| File | Intention |
| --- | --- |
| [boot_origin_bmc_media.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/boot_origin_bmc_media.dag) | Deliver content-addressed boot images from R2 through HTTPS or a segment-local NFS relay so different BMCs consume one image authority. |
| [cardinality_refinement.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/cardinality_refinement.dag) | Make cardinality constraints refuse invalid construction rather than rely on call-site convention. |
| [ci_merge_freshness.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/ci_merge_freshness.dag) | Define merge-time receipt freshness so individually green changes cannot compose against an untested target tree or gate roster. |
| [ci_process_end_to_end.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/ci_process_end_to_end.dag) | Coordinate CI cost measurement and implementation changes across the complete path from source change to accepted result. |
| [cli_run_hollowing_plan.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/cli_run_hollowing_plan.dag) | Move policy out of the Rust CLI entrypoint into substrate-owned consumers so the bootstrap remains a shrinking execution boundary. |
| [construction_justification_rule.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/construction_justification_rule.dag) | Explain why constructions need a production consumer and track the missing ingestion/enrollment enforcement. |
| [convergence_coverage_program.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/convergence_coverage_program.dag) | Provide one governing spine for covering production convergence to the execution boundary. |
| [dag_v2_defork_audit.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/dag_v2_defork_audit.dag) | Locate semantic duplicate implementations between DAG and v2 with independent negative controls. |
| [demand_engine_program.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/demand_engine_program.dag) | Define how the demand engine identifies work, shares results and selects materialization, realization and placement through one coherent model. |
| [deploy_convergence_observed_side.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/deploy_convergence_observed_side.dag) | Define what deployment observation proves and which historical paths were dead ends. |
| [durable_work_handoff.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/durable_work_handoff.dag) | Define durable work handoff across attempts and storage boundaries so replacement processes continue the same obligation without losing its identity. |
| [ensure_closure_design.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/ensure_closure_design.dag) | Require executable plans to establish their environmental prerequisites at the selected placement before compilation can admit production execution. |
| [extdeps_modeling_preflight.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/extdeps_modeling_preflight.dag) | Carry actionable external-model preflight rows rather than repeat modeling assumptions in each brief. |
| [fabric_serving_router.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/fabric_serving_router.dag) | Define how a fabric serving router selects available serving capacity and relates requests to the shared allocation model. |
| [fleet_naming_reach_access.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/fleet_naming_reach_access.dag) | Unify fleet naming, endpoint resolution and authorized access so operations reach the intended host without embedded provider or address assumptions. |
| [fleet_subsumption_manual_gaps.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/fleet_subsumption_manual_gaps.dag) | Identify manual fleet procedures missing modeled equivalents so boot and convergence programs can replace their operational responsibility. |
| [flex_supplier_generalization.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/flex_supplier_generalization.dag) | Add AWS EC2 spot and on-demand capacity as priced flex-autoscaler suppliers without duplicating the existing Vast orchestration loop. |
| [for_while_loop_sugar.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/for_while_loop_sugar.dag) | Define for and while syntax through precise desugaring so convenient loops preserve the language's control-flow and termination semantics. |
| [gpu_card_host_program.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/gpu_card_host_program.dag) | Coordinate card, host, power and serving acceptance choices with unresolved operator selections. |
| [gunb_ai_projection.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/gunb_ai_projection.dag) | Map gunb.ai website content, design tokens, layout and deployment to shared modeled authorities so the site presents one consistent source of facts. |
| [harness_session_lifecycle.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/harness_session_lifecycle.dag) | Keep one worker session across transparent supervision rounds until hand-in, decision or stop, then drive acceptance and revision through events. |
| [host_convergence_circuit_residue.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/host_convergence_circuit_residue.dag) | Record host-convergence circuit evidence and outstanding live-wiring obligations so hermetic loop tests are not mistaken for operational deployment proof. |
| [host_effect_orchestration.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/host_effect_orchestration.dag) | Replace host-side orchestration with typed apply/ensure interfaces while retaining actual effect boundaries. |
| [inert_layer_lens.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/inert_layer_lens.dag) | Detect declarations unreachable from production roots while distinguishing test-only coverage and heuristic importance from actual runtime use. |
| [input_envelope_roadmap.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/input_envelope_roadmap.dag) | Bound work inputs along declared axes so admission refuses unknown or oversized corpora and scheduling can account for their resource demands. |
| [interpreter_kernel_d.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/interpreter_kernel_d.dag) | Bound the irreducible interpreter/physics kernel and account for policy that can move to the substrate. |
| [joint_incompatibility_claim_join.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/joint_incompatibility_claim_join.dag) | Scope an advisory cross-PR import-claim join under the deferred freshness ceiling and manager/operator permissions. |
| [logical_work_realization.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/logical_work_realization.dag) | Define the consumer interface and bounded pilot from logical demand through selected realization and completion. |
| [mtcollins1_ci_service_legs.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/mtcollins1_ci_service_legs.dag) | Sequence Mt. Collins runner qualification before CI service and bind launch, observation and teardown to the authorized attempt. |
| [network_install_architecture_rows.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/network_install_architecture_rows.dag) | Generalize architecture-specific network installation while preserving existing boot behavior and trust evidence. |
| [ntfy_deployment_gate_approval.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/ntfy_deployment_gate_approval.dag) | Specify an approval-notification flow through a GitHub deployment protection app so privileged deployments require an operator's decision. |
| [p4_home_serving_power_semantics.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/p4_home_serving_power_semantics.dag) | Distinguish GPU rated power from applied limits so home-serving selection admits only power settings supported by cited evidence. |
| [physical_machine_boot_program.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/physical_machine_boot_program.dag) | Define one physical-machine boot operation reusable across installation and runtime lifecycles without conflating host and guest responsibilities. |
| [reference_grounding_migration.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/reference_grounding_migration.dag) | Replace name-like references with correctly grounded typed references without confusing role strings for names. |
| [resolver_type_name_collision_wall.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/resolver_type_name_collision_wall.dag) | Define a resolver guard against conflicting type-name authorities, with scope rules and negative controls that expose ambiguous binding. |
| [self_applying_lenses.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/self_applying_lenses.dag) | Define a shared pattern for deriving/applying lens findings and identify consumer work. |
| [serving_fleet_program.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/serving_fleet_program.dag) | Carry typed fleet deliverables and coordinate serving acceptance without leaking private procurement facts. |
| [shell_emission_model.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/shell_emission_model.dag) | Provide the surviving model for shell emission as a target and owned foreign execution boundaries. |
| [site_segment_anchor_bmc_reach.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/site_segment_anchor_bmc_reach.dag) | Use learned MAC addresses, segment observations and a fenced local holder to resolve and reach BMCs across deployment sites. |
| [solve_higher_order_design.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/solve_higher_order_design.dag) | Define how higher-order solving should express and resolve constraints through the compiler's shared constraint machinery. |
| [source_layout_authority.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/source_layout_authority.dag) | Make declared module names the sole source-path authority and coordinate corpus migration to that mapping without reverse filesystem inference. |
| [srv1_deploy_principal_split.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/srv1_deploy_principal_split.dag) | Separate srv1 deployment capabilities from operator and root privileges so routine deployment uses only its intended authority. |
| [structural_quadratic_wall_coverage_audit.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/structural_quadratic_wall_coverage_audit.dag) | Identify structurally quadratic compiler operations and determine which cost controls actually detect or prevent their repeated work. |
| [transport_argv_anemia_dissolution.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/transport_argv_anemia_dissolution.dag) | Replace duplicated transport command construction with owned operation declarations and shell realization so argument semantics have one authority. |
| [typescript_gap_census.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/typescript_gap_census.dag) | Identify missing TypeScript target capabilities and the discriminating acceptance evidence needed to close each realization gap. |
| [v2_compiler_architecture.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/v2_compiler_architecture.dag) | Define the compiler's stage decomposition, semantic boundaries and governing relationships so subordinate implementations share one architecture. |
| [v2_complexity_capability_parity.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/v2_complexity_capability_parity.dag) | Identify native complexity-analysis capabilities needed to match existing checks and specify evidence that distinguishes actual parity from declarations. |
| [v2_corpus_self_host.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/v2_corpus_self_host.dag) | Define whole-corpus native self-host acceptance and the seed obligations that must disappear before the compiler owns its complete path. |
| [v2_self_hosting.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/v2_self_hosting.dag) | Sequence the transition from weak to strong self-hosting so the emitted compiler progressively owns production generation instead of the seed. |
| [value_null_split.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/value_null_split.dag) | Distinguish language absence from runtime null-like values so lookup, optional values and evaluation refusals retain their intended semantics. |

### dag/std/runtime/bin_shims

| File | Intention |
| --- | --- |
| [README.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/std/runtime/bin_shims/README.md) | Define where binary-shim declarations live, how they are named, and how each references its modeled process entrypoint. |

### dag/test/fixture/codex_device_login

| File | Intention |
| --- | --- |
| [README.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/test/fixture/codex_device_login/README.md) | Ground headless device-login prompt parsing in captured Codex output, including ANSI wrapping, expiry text and capture provenance. |

### dag/test/fixture/codex_provider_events

| File | Intention |
| --- | --- |
| [README.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/test/fixture/codex_provider_events/README.md) | Ground Codex provider-event parsing in an actual JSON stream so message text and usage fields are modeled from emitted bytes. |

### docs/extdeps/lotes_azifa072

| File | Intention |
| --- | --- |
| [README.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/extdeps/lotes_azifa072/README.md) | Identify the LGA4926 socket body and document drawing provenance, extracted pad geometry and unresolved manufacturing assumptions. |

### docs/measurements/roster_import_sweep_native

| File | Intention |
| --- | --- |
| [README.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/measurements/roster_import_sweep_native/README.md) | Explain how to compare native roster-import sweep output and cost against fixed interpreter results, including path-refusal controls. |

### docs/measurements/value_position_block_nests

| File | Intention |
| --- | --- |
| [README.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/measurements/value_position_block_nests/README.md) | Explain fixtures and timing controls that detect repeated lowering of nested value-position blocks and its exponential cost. |

### docs/plans

| File | Intention |
| --- | --- |
| [altra-cross-channel-rank-mixing-survey.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/plans/altra-cross-channel-rank-mixing-survey.md) | Distinguish documented Altra DIMM mixing constraints from unanswered cross-channel questions so hardware selection does not treat missing guidance as prohibition. |
| [cassette-assembly-requirements.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/plans/cassette-assembly-requirements.md) | Define what makes a cassette assembly safe and acceptable and what remains to qualify. |
| [cassette-board-mechanical-references.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/plans/cassette-board-mechanical-references.md) | Collect cited board geometry, coordinate conventions and provisional dimensions needed to model cassette mounting and clearances. |
| [cassette-inventory-convergence-implementation.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/plans/cassette-inventory-convergence-implementation.md) | Define how cassette inventory, physical readback and print demands converge while keeping shadow planning separate from printer actuation. |
| [cassette-platforms.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/plans/cassette-platforms.md) | Keep multiple board platforms on one cassette generator and expose platform-specific unknowns. |
| [cassette-print-day.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/plans/cassette-print-day.md) | Coordinate the cassette print queue, replacement parts and physical reservations while preserving uncertain starts and acceptance evidence. |
| [cassette-server-psu-interface.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/plans/cassette-server-psu-interface.md) | Choose a shared server-PSU interface for cassette assemblies while making electrical, harness, clearance and thermal qualification explicit. |
| [ci-end-to-end-accounting.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/plans/ci-end-to-end-accounting.md) | Measure the full pull-request and merge-group CI cost so reduction work targets observed time and resource consumption. |
| [ci-minutes-product-design.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/plans/ci-minutes-product-design.md) | Offer GitHub-native CI jobs through selectable runner classes and spend caps while fabric allocation, isolation and billing remain behind that interface. |
| [ci-per-job-microvm-migration.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/plans/ci-per-job-microvm-migration.md) | Plan moving public and private CI jobs to disposable microVMs with evidence that launch, isolation and teardown satisfy cutover requirements. |
| [cli-run-reconcile-defork.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/plans/cli-run-reconcile-defork.md) | Unify CLI reconciliation and module resolution so execution shares one authority without repeated whole-corpus work or incomplete cache identities. |
| [compiler-guarantee-recovery-gap-analysis.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/plans/compiler-guarantee-recovery-gap-analysis.md) | Expose the difference between intended guarantees and actually enforced rungs across acceptance paths. |
| [dag-scm-design.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/plans/dag-scm-design.md) | Define typed source edits, integration and refusal semantics so source-control operations preserve authored intent rather than depend on textual merge heuristics. |
| [effect-namespace-grants.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/plans/effect-namespace-grants.md) | Define scoped read, write and execution authority over namespaces and clarify how reference and publication permissions differ. |
| [emission-admission-stage-aware-pipeline-design.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/plans/emission-admission-stage-aware-pipeline-design.md) | Define stage and generation-lineage evidence for emitted-code admission so declared compiler transitions can be distinguished from regressions before promotion or reuse. |
| [emit-type-surface-traversal-cost.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/plans/emit-type-surface-traversal-cost.md) | Document a pinned type-surface traversal experiment and its falsified explanations so performance work starts from discriminating evidence. |
| [fabric-recut-program.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/plans/fabric-recut-program.md) | Explain separation of work identity, sanction, selection and occupancy through a recut. |
| [fabric-storage.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/plans/fabric-storage.md) | Define storage contracts and migration/capability gaps used by public and private consumers. |
| [floor-cut-replacement-plan.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/plans/floor-cut-replacement-plan.md) | Coordinate replacement of the floor and account for every intentionally lost guarantee. |
| [generated-file-conflict-policy.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/plans/generated-file-conflict-policy.md) | Define deterministic handling of generated-output merge conflicts without making generated bytes authoritative. |
| [keying-relation-design.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/plans/keying-relation-design.md) | Define identity, dependency and keying relations so demand evaluation can share results without confusing equality, provenance or invalidation. |
| [layering-imports-reference-repoint-design.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/plans/layering-imports-reference-repoint-design.md) | Keep the layering gate effective after import syntax disappears by deriving its dependency edges from resolved references. |
| [microvm-launch-displacement-analysis.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/plans/microvm-launch-displacement-analysis.md) | Determine which live launcher responsibilities must be replaced, and at what cost, before a modeled executor can take over microVM actuation. |
| [milk-crate-stack-design.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/plans/milk-crate-stack-design.md) | Define a stackable cassette frame's topology, service access, airflow and mechanical interfaces for a manufacturable shared chassis. |
| [model-grounding-lens-extract.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/plans/model-grounding-lens-extract.md) | Detect modeled concepts lacking valid ownership/grounding, without mechanical retyping of arbitrary strings. |
| [namespace-cut-replacement-plan.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/plans/namespace-cut-replacement-plan.md) | Coordinate replacing namespace architecture through an ordered cutover that accounts for declaration identities, consumers and intentionally lost guarantees. |
| [namespace-resolution-design.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/plans/namespace-resolution-design.md) | Define how names bind across scopes, qualification and visibility so the resolver rejects ambiguity without relying on filesystem accidents. |
| [namespace-unique-on-chain-operational-plan.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/plans/namespace-unique-on-chain-operational-plan.md) | Plan enforcement of name uniqueness along namespace chains through explicit identity edges, migration steps and ambiguous-binding refusal controls. |
| [print-batch-orchestration.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/plans/print-batch-orchestration.md) | Turn desired part quantities into packed plates and reconciled printer schedules so retries fulfill remaining demand without duplicate prints. |
| [print-workflow-integration.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/plans/print-workflow-integration.md) | Explain how an operator drives the print workflow and where physical acceptance is required. |
| [replacement-migration-doctrine.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/plans/replacement-migration-doctrine.md) | Define how to replace an architectural authority at its root while accounting for consumers, lost guarantees and terminal evidence. |
| [roadmap-launch-mvp-plan.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/plans/roadmap-launch-mvp-plan.md) | Define the minimum usable roadmap launch experience and the acceptance evidence required for a live operational dashboard. |
| [roadmap-spawner.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/plans/roadmap-spawner.md) | Specify how roadmap tasks are sized, assigned and dispatched into workers with the context and constraints needed for execution. |
| [roadmap-workspace-remodel-plan.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/plans/roadmap-workspace-remodel-plan.md) | Restructure roadmap workspace layout and interactions so operators can inspect work, understand progress and act from one surface. |
| [roadmap-workspace-ux-plan.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/plans/roadmap-workspace-ux-plan.md) | Define the roadmap workspace's navigation, work presentation and operator interactions so task state leads to clear next actions. |
| [runner-throughput-qualification-recut.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/plans/runner-throughput-qualification-recut.md) | Measure an independently selected runner offer against its promised throughput without letting qualification mint its own sanction or supply. |
| [scaffold-admission-doctrine.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/plans/scaffold-admission-doctrine.md) | State when temporary deviations may be admitted and how their end is proved. |
| [serving-capacity-home-ruling.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/plans/serving-capacity-home-ruling.md) | Fix where compute and serving share selection but differ in occupancy authority. |
| [serving-hardware-neutral-cutover.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/plans/serving-hardware-neutral-cutover.md) | Replace hardware-specific serving behavior with shared capability and placement models, proving that real callers use the generalized path. |
| [shell-intent-emit-realization-design.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/plans/shell-intent-emit-realization-design.md) | Separate declared operation intent from shell target realization and physical executor boundaries. |
| [shell-to-dag-residual-census-and-arc-completion.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/plans/shell-to-dag-residual-census-and-arc-completion.md) | Account for every host/shell site and distinguish permanent foreign execution from dissolvable orchestration. |
| [srv1-workspace-vm-bringup.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/plans/srv1-workspace-vm-bringup.md) | Record the operator steps and kernel/network readiness required for a workspace VM. |

### docs/probes

| File | Intention |
| --- | --- |
| [leading_minus_continuation_silently_truncates_2026-08-23.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/probes/leading_minus_continuation_silently_truncates_2026-08-23.md) | Demonstrate how a leading-minus continuation silently changes expression meaning and identify the regression behavior that must be refused. |
| [mtcollins1_2drx4_group_test_prediction_2026-09-12.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/probes/mtcollins1_2drx4_group_test_prediction_2026-09-12.md) | Preregister the expected result and falsifiers for isolating four 2DRx4 spare DIMMs within a known Mt. Collins population. |
| [mtcollins1_2rx4_group_outcome_2026-09-12.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/probes/mtcollins1_2rx4_group_outcome_2026-09-12.md) | Score the Micron 2Rx4 group prediction against observed power and restart behavior and assess implications for module selection. |
| [mtcollins1_2rx4_group_test_prediction_2026-09-12.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/probes/mtcollins1_2rx4_group_test_prediction_2026-09-12.md) | Preregister a 2Rx4 DIMM substitution test that compares its behavior with the 2DRx4 quartet in the same socket arrangement. |
| [mtcollins1_2rx4_outcome_correction_2026-09-12.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/probes/mtcollins1_2rx4_outcome_correction_2026-09-12.md) | Separate observed 2Rx4 survival and console evidence from unsupported training-success claims and identify the boot-delivery defect. |
| [mtcollins1_ampere_smpro_access_paths_2026-09-12.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/probes/mtcollins1_ampere_smpro_access_paths_2026-09-12.md) | Identify documented Ampere pre-OS observation interfaces and order probes that distinguish sourced register meanings from local inference. |
| [mtcollins1_dram_training_root_cause_2026-09-12.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/probes/mtcollins1_dram_training_root_cause_2026-09-12.md) | Explain the firmware-reported DIMM package mismatch using captured console evidence and bound its implications for fleet memory qualification. |
| [mtcollins1_memory_census_and_boot_delivery_2026-09-11.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/probes/mtcollins1_memory_census_and_boot_delivery_2026-09-11.md) | Report Mt. Collins DIMM population and load-backed memory health while tracing the boot-delivery failures encountered during intake. |
| [mtcollins1_micron_module_screen_state_2026-09-12.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/probes/mtcollins1_micron_module_screen_state_2026-09-12.md) | Consolidate tested DIMM configurations, eliminated hypotheses and evidence limits so subsequent memory investigation does not repeat prior experiments. |
| [mtcollins1_sel_oem_payload_census_2026-09-12.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/probes/mtcollins1_sel_oem_payload_census_2026-09-12.md) | Count OEM payload families across the complete Mt. Collins event log to test co-emission patterns and constrain interpretations. |
| [mtcollins1_socket_swap_outcome_2026-09-12.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/probes/mtcollins1_socket_swap_outcome_2026-09-12.md) | Compare the socket-swap prediction with observed restart timing and payload changes to test whether the fault follows the DIMM quartet. |
| [mtcollins1_socket_swap_prediction_2026-09-12.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/probes/mtcollins1_socket_swap_prediction_2026-09-12.md) | Preregister whether moving the 2DRx4 quartet between sockets changes the reported fault location while preserving the failure. |
| [mtcollins1_spare_module_screen_prediction_2026-09-11.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/probes/mtcollins1_spare_module_screen_prediction_2026-09-11.md) | Preregister the all-spare 16-DIMM screening prediction, configuration and falsifiers before testing memory on Mt. Collins. |
| [mtcollins_memory_multiple_findings_2026-09-12.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/probes/mtcollins_memory_multiple_findings_2026-09-12.md) | Represent multiple firmware memory-mixture observations without generalizing specific byte mismatches into an unsupported universal compatibility rule. |
| [pre_os_refusal_consumer_sweep.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/probes/pre_os_refusal_consumer_sweep.md) | Trace pre-OS observation refusals from producers into their first consumers so assessment and replay preserve each failure distinction. |
| [prospective_memory_change_evaluation_2026-09-12.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/probes/prospective_memory_change_evaluation_2026-09-12.md) | Explain how proposed memory changes are assessed against positioned populations, firmware evidence and independent qualification obligations before purchasing or installation. |

### docs/probes/realization_census_1dd3eda5_2026-10-24

| File | Intention |
| --- | --- |
| [README.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/probes/realization_census_1dd3eda5_2026-10-24/README.md) | Explain a pinned census of where modules choose storage, caching, access, authorization and placement instead of receiving owned decisions. |
| [method.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/probes/realization_census_1dd3eda5_2026-10-24/method.md) | Describe the census sampling, classification and reconciliation rules so its counts, comparisons and uncertainty can be audited. |

### docs/probes/srv1_nvfp4_serving_2026-10-25

| File | Intention |
| --- | --- |
| [srv1_nvfp4_serving_verification_2026-10-25.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/probes/srv1_nvfp4_serving_2026-10-25/srv1_nvfp4_serving_verification_2026-10-25.md) | Verify srv1's loaded NVFP4 weights and FP8 KV configuration while measuring correctness, latency, throughput and context limits against observed engine identity. |

### docs/recovered/mtcollins-host-image-source

| File | Intention |
| --- | --- |
| [README.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/recovered/mtcollins-host-image-source/README.md) | Explain the provenance and behavior of recovered Mt. Collins host-image tooling so its modeled replacement can be checked against actual source. |

### docs/runbooks

| File | Intention |
| --- | --- |
| [README.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/runbooks/README.md) | Provide a single entrypoint for finding infrastructure procedures covering BMC access, host boot, credentials and privileged setup. |
| [bmc-assimilator-wif-setup.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/runbooks/bmc-assimilator-wif-setup.md) | Guide configuring GCP workload identity so unattended BMC assimilation obtains scoped tokens without manually pasted credentials. |
| [bmc-redfish-operator-access.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/runbooks/bmc-redfish-operator-access.md) | Guide enabling and verifying out-of-band Redfish telemetry on srv1 and srv2 through their dedicated BMC interfaces. |
| [gcp-iam-root-ceremony-apply.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/runbooks/gcp-iam-root-ceremony-apply.md) | Guide the privileged IAM bootstrap ceremony that establishes missing trust-root cells and verifies them before fleet convergence resumes. |
| [mtcollins1-ssh-host-key-mint.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/runbooks/mtcollins1-ssh-host-key-mint.md) | Guide minting, delivering and anchoring mtcollins1's stable SSH host key so transport identity remains verifiable across diskless boots. |
| [srv3-nbd-proxy-ws-upgrade-dry-run.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/runbooks/srv3-nbd-proxy-ws-upgrade-dry-run.md) | Guide an authorized websocket probe that determines whether srv3 supports NBD-proxy virtual media before choosing its installation path. |
| [srv3-os-install-actuate.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/runbooks/srv3-os-install-actuate.md) | Guide authorized srv3 installation through seeded virtual media, console observation and OS-up evidence while respecting destructive-step gates. |

### fixtures/census_scoped_acquisition

| File | Intention |
| --- | --- |
| [README.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/fixtures/census_scoped_acquisition/README.md) | Explain acquisition fixtures that distinguish missing and malformed modules, shared dependencies and requester-specific permissions without falsely completing a closure. |

### provider-runtime/codex

| File | Intention |
| --- | --- |
| [README.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/provider-runtime/codex/README.md) | Explain regenerating the pinned Codex package lock and verifying downloaded archives before provider-runtime materialization trusts their contents. |

### sites

| File | Intention |
| --- | --- |
| [README.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/sites/README.md) | Orient maintainers to the existing chassis review site and its generated meshes so browser interactions never become mechanical geometry authority. |

### src/v2

| File | Intention |
| --- | --- |
| [LAYOUT.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/src/v2/LAYOUT.md) | Specify how v2 module identities map to source directories, filenames and test locations for authors and tooling. |

### src/v2/lens/testdata

| File | Intention |
| --- | --- |
| [README.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/src/v2/lens/testdata/README.md) | Explain the anemia-lens judgment corpus, its labeled inputs and how to score decisions without confusing examples with live findings. |

### test/compiler_gate_contract_stamp/fixtures

| File | Intention |
| --- | --- |
| [README.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/test/compiler_gate_contract_stamp/fixtures/README.md) | Explain a captured silent-green workflow fixture used to detect evidence instructions and compiler subjects resolved from different revisions. |

### test/namecheap

| File | Intention |
| --- | --- |
| [README.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/test/namecheap/README.md) | Guide fixture-only checks that verify Namecheap query transport keeps credentials out of argv and emitted workflows consume the declared receipt. |

### test/native_filesystem

| File | Intention |
| --- | --- |
| [README.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/test/native_filesystem/README.md) | Explain native source-root acquisition controls for filesystem refusals, complete traversal and effectful tail-call lowering through emitted services. |

### test/primitive_runtime_body

| File | Intention |
| --- | --- |
| [README.md](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/test/primitive_runtime_body/README.md) | Explain how the primitive-runtime-body control checks runtime registry names against emitted bodies and detects missing implementations with a negative specimen. |

<details>
<summary>Supporting files (47)</summary>

### dag/gunbc/plans

| File | Intention |
| --- | --- |
| [demand_restatement_follow_up.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/demand_restatement_follow_up.dag) | Enumerate retained authored service-demand rows by exact identity and the condition that permits each redundant declaration to disappear. |
| [md_helpers.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/dag/gunbc/plans/md_helpers.dag) | Provide reusable Markdown constructors so plan modules can render headings, paragraphs, lists, quotations and tables consistently. |

### docs/extdeps/asrock_rack

| File | Intention |
| --- | --- |
| [ALTRAD8UD_standoff_grid.png](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/extdeps/asrock_rack/ALTRAD8UD_standoff_grid.png) | Show the ALTRAD8UD motherboard standoff arrangement as a mechanical reference for locating chassis supports. |

### docs/extdeps/lotes_azifa072

| File | Intention |
| --- | --- |
| [lga4926_pad_array.svg](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/extdeps/lotes_azifa072/lga4926_pad_array.svg) | Visualize the 4,926 extracted LGA4926 pad centers so a reader can inspect the array's shape and spacing. |
| [lga4926_pcb_pads.csv](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/extdeps/lotes_azifa072/lga4926_pcb_pads.csv) | Store normalized LGA4926 PCB pad-center coordinates extracted from the cited socket drawing for geometric checks and modeling. |

### docs/measurements/roster_import_sweep_native

| File | Intention |
| --- | --- |
| [manifest.txt](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/measurements/roster_import_sweep_native/manifest.txt) | Enumerate native roster-sweep test cases and source paths so the isolated child receives a fixed, reproducible subject set. |

### docs/measurements/roster_import_sweep_native/expected

| File | Intention |
| --- | --- |
| [lexical_scope_sweep.txt](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/measurements/roster_import_sweep_native/expected/lexical_scope_sweep.txt) | Specify the exact import edit expected when the roster sweep processes the lexical-scope fixture. |
| [node_dag_sweep.txt](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/measurements/roster_import_sweep_native/expected/node_dag_sweep.txt) | Specify the empty edit output expected when the roster sweep processes the node module. |
| [refuse_escaped_path.txt](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/measurements/roster_import_sweep_native/expected/refuse_escaped_path.txt) | Specify the exact refusal expected when a roster-sweep subject path escapes the repository boundary. |

### docs/measurements/value_position_block_nests/multi_d10/dag/gunbc

| File | Intention |
| --- | --- |
| [t.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/measurements/value_position_block_nests/multi_d10/dag/gunbc/t.dag) | Provide a depth-10 nested-match fixture with multi-statement value blocks to measure repeated-lowering cost in the native compiler. |

### docs/measurements/value_position_block_nests/multi_d14/dag/gunbc

| File | Intention |
| --- | --- |
| [t.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/measurements/value_position_block_nests/multi_d14/dag/gunbc/t.dag) | Provide a depth-14 nested-match fixture with multi-statement value blocks to measure repeated-lowering cost in the native compiler. |

### docs/measurements/value_position_block_nests/multi_d20/dag/gunbc

| File | Intention |
| --- | --- |
| [t.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/measurements/value_position_block_nests/multi_d20/dag/gunbc/t.dag) | Provide a depth-20 nested-match fixture with multi-statement value blocks to measure repeated-lowering cost in the native compiler. |

### docs/measurements/value_position_block_nests/single_d10/dag/gunbc

| File | Intention |
| --- | --- |
| [t.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/measurements/value_position_block_nests/single_d10/dag/gunbc/t.dag) | Provide a depth-10 nested-match fixture with single-statement value blocks to measure repeated-lowering cost in the native compiler. |

### docs/measurements/value_position_block_nests/single_d14/dag/gunbc

| File | Intention |
| --- | --- |
| [t.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/measurements/value_position_block_nests/single_d14/dag/gunbc/t.dag) | Provide a depth-14 nested-match fixture with single-statement value blocks to measure repeated-lowering cost in the native compiler. |

### docs/measurements/value_position_block_nests/single_d20/dag/gunbc

| File | Intention |
| --- | --- |
| [t.dag](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/measurements/value_position_block_nests/single_d20/dag/gunbc/t.dag) | Provide a depth-20 nested-match fixture with single-statement value blocks to measure repeated-lowering cost in the native compiler. |

### docs/plans

| File | Intention |
| --- | --- |
| [milk-crate-topology.svg](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/plans/milk-crate-topology.svg) | Illustrate the milk-crate frame, cassette arrangement and service access as a conceptual topology rather than manufacturing geometry. |

### docs/probes

| File | Intention |
| --- | --- |
| [mtcollins_bound_attempt_host_capture_2026-09-07.txt](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/probes/mtcollins_bound_attempt_host_capture_2026-09-07.txt) | Record host serial evidence whose unique runner registration binds one physical boot attempt to its GitHub job. |
| [mtcollins_checkout_and_js_action_2026-09-07.txt](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/probes/mtcollins_checkout_and_js_action_2026-09-07.txt) | Capture checkout and JavaScript-action execution inside a filtered microVM, including actual network traffic and idle-attempt metering. |
| [mtcollins_first_job_host_capture_2026-09-07.txt](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/probes/mtcollins_first_job_host_capture_2026-09-07.txt) | Provide serial boot and launch evidence for the first Mt. Collins microVM CI job and its modeled qualification receipt. |
| [mtcollins_jit_absent_launch_defect_2026-09-07.txt](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/probes/mtcollins_jit_absent_launch_defect_2026-09-07.txt) | Expose the mistaken build-label join between a successful job and a credential-less boot, including the harmful power-cycle sequence. |
| [mtcollins_signed_envelope_2026-09-07.txt](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/probes/mtcollins_signed_envelope_2026-09-07.txt) | Capture valid and impostor envelope checks on the same host path to demonstrate admission and refusal before microVM resource allocation. |
| [mtcollins_two_attempts_one_boot_2026-09-07.txt](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/probes/mtcollins_two_attempts_one_boot_2026-09-07.txt) | Record two isolated microVM jobs on one physical boot and the resource readbacks needed to judge teardown between attempts. |

### docs/probes/realization_census_1dd3eda5_2026-10-24

| File | Intention |
| --- | --- |
| [classes.tsv](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/probes/realization_census_1dd3eda5_2026-10-24/classes.tsv) | Group realization-decision findings by recurring class, linking each class to its owning authority, enforcement gap and next proof. |
| [findings.tsv](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/probes/realization_census_1dd3eda5_2026-10-24/findings.tsv) | Locate individual realization decisions by module and symbol, retaining concrete specimens and the shared authority each decision duplicates or lacks. |
| [merge_report.txt](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/probes/realization_census_1dd3eda5_2026-10-24/merge_report.txt) | Record how census slices were reconciled, including duplicate handling, relabeling and findings without supporting instance rows. |
| [modules.tsv](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/probes/realization_census_1dd3eda5_2026-10-24/modules.tsv) | Record each inspected module's realization-decision classifications across five dimensions and link its worst finding to concrete evidence. |
| [receipt.json](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/probes/realization_census_1dd3eda5_2026-10-24/receipt.json) | Bind the realization census to its source revision, scripts, populations and reviewer-agreement measurements for reproducible verification. |

### docs/probes/srv1_nvfp4_serving_2026-10-25

| File | Intention |
| --- | --- |
| [raw_arm_fp8_kv_flash_attn.json](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/probes/srv1_nvfp4_serving_2026-10-25/raw_arm_fp8_kv_flash_attn.json) | Record initial srv1 serving-task outputs and latency measurements under the FP8 KV attention test arm. |
| [raw_cold.json](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/probes/srv1_nvfp4_serving_2026-10-25/raw_cold.json) | Record cold-prefill throughput and context-limit refusal responses for large prompts sent to the srv1 NVFP4 endpoint. |
| [raw_eval3.json](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/probes/srv1_nvfp4_serving_2026-10-25/raw_eval3.json) | Record srv1 evaluation responses, answer hashes and token timings so generated answers can be distinguished from truncated initial samples. |
| [raw_eval_oracle.json](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/probes/srv1_nvfp4_serving_2026-10-25/raw_eval_oracle.json) | Preserve complete srv1 evaluation answers and independent correctness judgments for rechecking the serving test's accuracy claims. |
| [raw_eval_oracle_fix.json](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/probes/srv1_nvfp4_serving_2026-10-25/raw_eval_oracle_fix.json) | Record the palindrome-test answer judged by a form-independent predicate rather than the first oracle's label-based check. |
| [raw_longctx3.jsonl](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/probes/srv1_nvfp4_serving_2026-10-25/raw_longctx3.jsonl) | Record srv1 prefill timings and acceptance responses across increasing context lengths to measure the endpoint's practical input bounds. |

### docs/recovered/mtcollins-host-image-source

| File | Intention |
| --- | --- |
| [build.sh.bak.recovered](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/recovered/mtcollins-host-image-source/build.sh.bak.recovered) | Preserve the backup host-image build script as evidence of how the initramfs and kernel build were assembled. |
| [build.sh.recovered](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/recovered/mtcollins-host-image-source/build.sh.recovered) | Preserve the host-image build script that assembles initramfs contents and builds the kernel used for Mt. Collins runner boots. |
| [envelope.sh.recovered](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/recovered/mtcollins-host-image-source/envelope.sh.recovered) | Preserve the script packaging the runner kernel, guest artifacts and verifier into boot media readable by the BMC controller. |
| [initramfs.spec.in](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/recovered/mtcollins-host-image-source/initramfs.spec.in) | Specify initramfs files, modes and device nodes so the recovered host image starts with a usable console and runtime tree. |
| [udhcpc.script.recovered](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/recovered/mtcollins-host-image-source/udhcpc.script.recovered) | Preserve the DHCP lease hook that records and applies address, subnet and routing information to the runner host. |
| [vm.json](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/recovered/mtcollins-host-image-source/vm.json) | Preserve the Firecracker guest configuration selecting its kernel, read-only root disk, boot arguments, virtual CPU count and memory size. |

### docs/recovered/mtcollins-host-image-source/cp

| File | Intention |
| --- | --- |
| [cp-signing.pub](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/recovered/mtcollins-host-image-source/cp/cp-signing.pub) | Preserve the control-plane public key used by recovered host-image tooling to verify signed runner launch envelopes. |

### docs/recovered/mtcollins-host-image-source/guest

| File | Intention |
| --- | --- |
| [gunbc-probe.sh.recovered](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/recovered/mtcollins-host-image-source/guest/gunbc-probe.sh.recovered) | Preserve the guest PID-1 network probe used to isolate filtered-egress observations from unrelated operating-system traffic. |
| [gunbc-runner-init.sh.recovered](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/recovered/mtcollins-host-image-source/guest/gunbc-runner-init.sh.recovered) | Preserve guest PID-1 initialization that mounts filesystems, configures networking and starts the CI runner from supplied job credentials. |

### docs/recovered/mtcollins-host-image-source/initramfs

| File | Intention |
| --- | --- |
| [init.pre-supervisor.recovered](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/recovered/mtcollins-host-image-source/initramfs/init.pre-supervisor.recovered) | Preserve the host-init generation before supervision so jailer launch behavior can be compared with later attempt lifecycle handling. |
| [init.pre-verify.recovered](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/recovered/mtcollins-host-image-source/initramfs/init.pre-verify.recovered) | Preserve the host-init generation before artifact verification so the introduction of boot and guest integrity checks remains inspectable. |
| [init.prev.recovered](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/recovered/mtcollins-host-image-source/initramfs/init.prev.recovered) | Preserve the initial host-boot probe that reports image identity, network-driver binding and KVM availability through the serial console. |
| [init.recovered](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/recovered/mtcollins-host-image-source/initramfs/init.recovered) | Preserve the host-init template implementing runner-attempt preparation, authenticated admission, jailer launch, observation and teardown for modeled replacement. |

### docs/recovered/mtcollins-host-image-source/irfs

| File | Intention |
| --- | --- |
| [init.recovered](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/recovered/mtcollins-host-image-source/irfs/init.recovered) | Preserve the checkout-2 instantiation of the host-init template so deployed build-identity substitutions can be compared with their source. |

</details>
