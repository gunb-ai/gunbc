## Run-link B — head-matched CAP-BRACKET control at `4dc4800f13f` (the node's actual control)

This is the row's control `a_cap_bracket_pass_never_masks_a_running_runner_unit` executed at the FINAL head, so the bar item "executed evidence at the FINAL head" names `4dc4800f13f` literally (the prior head pass, comment 7159816994, named 5fd6b595e35).

- HOST=srv1; BOOT=`c095c0cd` (srv1's boot, printed live by the run).
- heavy-slot: slot 7/12 acquired 2026-11-25T12:03:59Z; cgroup `heavy-slot-2880549.scope`.
- `claim_batch` built at head: `GIT_HEAD_BUILD=4dc4800f13f15944ea8a51bf9247d8422c56e37b`, tree `6da80b69`, `GIT_STATUS_PORCELAIN_AFTER=[]` (clean). sha256 (disk) `bbb67389cb37008586f356761ef94b73896a3e834d5bfe4280477d9e2184f206`.
- Invocation: `claim_batch --source-root dag --source-root src/v1 --source-root src/v2 --entry dag/test/claim/runner/runner_host_cap_bracket_witness.dag --functions a_cap_bracket_pass_never_masks_a_running_runner_unit`.
- Population non-empty: 1844 modules, 54913 resolved items in closure, 1 witness executed.
- Result: **PASS** `a_cap_bracket_pass_never_masks_a_running_runner_unit`, `eval_steps=152986 subject=6b8e696bfdca55d8`, **RUN_REAL_RC=0**.
- Note: `CB_VERSION_RC=2` is from a `--version` probe only (claim_batch has no `--version` flag); it is not the claim run. The claim's real exit is `RUN_REAL_RC=0`.
- Provenance: claim_batch has no `--version`; built-from is carried by `GIT_HEAD_BUILD` in the build checkout + disk sha256 + clean porcelain. `eval_steps=152986` is identical to the b198 head pass at 87bfedfb096 — same control semantics across heads. (Subject hash differs per interpreter build, not per bracket arm; the arm discrimination is carried by the RED/GREEN mutation pair in comment 7147212536, where reverting only `mask_slot_only_over` moved subject 309b6351→c4c4a244 and eval_steps 150164→151465.)

### Raw output (key lines + full block)

```
heavy-slot: slot 7/12 acquired by royal-swift-552 at 2026-11-25T12:03:59Z: bash /home/briansrls/scratch/royal-swift-552/run5.sh
HOST=srv1
BOOT=c095c0cd
INNER_CGROUP=0::/user.slice/user-1000.slice/user@1000.service/app.slice/heavy-slot-2880549.scope
=== build claim_batch at head (incremental on parse4 cache) ===
   Compiling v1-compiler v0.1.0 (/home/briansrls/scratch/royal-swift-552/bhead3/src/v1/stage0)
    Finished `release` profile [optimized] target(s) in 4m 40s
BUILD_RC=0
GIT_HEAD_BUILD=4dc4800f13f15944ea8a51bf9247d8422c56e37b
GIT_HEAD_TREE_BUILD=6da80b6903ac648e12ab8565fe0e6883042d760e
GIT_STATUS_PORCELAIN_AFTER=[]
BIN_PATH=/home/briansrls/scratch/royal-swift-552/bhead3/target/release/claim_batch
BIN_SHA256=bbb67389cb37008586f356761ef94b73896a3e834d5bfe4280477d9e2184f206  target/release/claim_batch
CB_VERSION_RC=2
CB_VERSION=memory-cgroup-bind: not requested
claim_batch: unknown argument: --version
[process-partition-exit] run_returned_ms=1 main_end_ms=1
===== CLAIM-BATCH cap-bracket control at head 4dc4800f13f =====
memory-cgroup-bind: not requested
[floor-phase] phase=bare-reference-edge-index-warm state=completed cpu_ms=143563 wall_ms=143630 rss_growth_bytes=4946411520 source_files=1844 bare_eligible=16 provenance=built-by-preparation scope=entry-closure-union entries=1 entries_refused=0 pool_files=10642
claim_batch: 1 entry group(s), 1 witness(es) total
[floor-phase] phase=module-index state=completed wall_ms=264 modules=8715
[floor-phase] phase=declarer-discovery state=completed wall_ms=286 declarers=18
[floor-phase] phase=whole-tree-graph-facts state=completed wall_ms=1172
[floor-phase] phase=declarer-closure state=completed wall_ms=12 sources=25
◐ started compile.frontend
✓ compile.frontend done in 291ms
✓ compile.normalize done in 24ms
✓ compile.reconcile done in 1 seconds
✓ compile.analyses done in 45ms
[floor-phase] phase=closure-strict-resolve state=completed wall_ms=1663 sources=25
◷ post-mock-precompute-rss — 8.3 GiB
◷ post-mock-precompute-children-max-rss — 8.0 GiB
claim_batch: whole-tree published mock corpus — 92 operation key(s)
claim_batch: resolving dag/test/claim/runner/runner_host_cap_bracket_witness.dag once (1 witness(es))
✓ typecheck gunbc.scm.object_store done in 3 seconds
✓ typecheck std.occurrence_binding_candidates done in 2 seconds
✓ typecheck gunbc.roadmap.roadmap_issue_sort done in 2 seconds
✓ typecheck gunbc.serving.serving_layout done in 2 seconds
✓ typecheck product.fabric.provider_admission done in 2 seconds
✓ typecheck gunbc.site.layout done in 3 seconds
✓ typecheck v2.compiler.tokenize done in 4 seconds
✓ typecheck gunbc.floor.floor_demand done in 3 seconds
✓ typecheck gunbc.site.moodboard done in 2 seconds
✓ typecheck std.decision done in 3 seconds
✓ typecheck gunbc.scm.object_table_json done in 3 seconds
✓ typecheck gunbc.serving.serving_benchmark_run_receipt done in 2 seconds
✓ typecheck gunbc.spark.dgx_procurement done in 3 seconds
✓ typecheck extdeps.git.object_store done in 2 seconds
✓ typecheck std.state_durability done in 2 seconds
✓ typecheck v2.std.compilers.target_model done in 1 minutes
✓ typecheck gunbc.scm.repository_envelope done in 16 seconds
✓ typecheck v2.std.grammar done in 3 seconds
✓ typecheck gunbc.bmc_model done in 5 seconds
✓ typecheck gunbc.scm.repository_load done in 4 seconds
✓ typecheck gunbc.site.element.build_meter done in 3 seconds
✓ typecheck v2.extdeps.languages.dag done in 4 seconds
✓ typecheck v2.extdeps.languages.typescript done in 2 seconds
✓ typecheck gunbc.machine_intake.access done in 2 seconds
✓ typecheck gunbc.machine_intake.mtcollins1_power_on_account done in 14 seconds
✓ typecheck gunbc.roadmap.roadmap_event_log done in 3 seconds
✓ typecheck gunbc.roadmap.roadmap_verification_receipt done in 2 seconds
✓ typecheck v2.compiler.parse done in 5 seconds
✓ typecheck gunbc.host.memory_configuration_evaluation done in 37 seconds
✓ typecheck gunbc.machine_intake.bmc_rotation_route done in 3 seconds
✓ typecheck gunbc.network_boot_delivery done in 3 seconds
✓ typecheck gunbc.package_delivery done in 3 seconds
✓ typecheck gunbc.serving.serving_replica_profile done in 4 seconds
✓ typecheck gunbc.boot_artifact_delivery done in 22 seconds
✓ typecheck gunbc.host.memory_change_evaluation done in 12 seconds
✓ typecheck gunbc.machine_intake.bmc_secure done in 25 seconds
✓ typecheck gunbc.serving.router_dev_policy done in 3 seconds
✓ typecheck gunbc.fleet.fleet_boot_delivery_solution done in 2 seconds
✓ typecheck gunbc.host.memory_change_report done in 5 seconds
✓ typecheck gunbc.codex_app_server_press done in 2 seconds
✓ typecheck gunbc.host.mtcollins_memory_change done in 4 seconds
✓ typecheck gunbc.os_install done in 3 seconds
✓ typecheck v2.compiler.body_lowering_fold done in 9 seconds
✓ typecheck gunbc.fleet.fleet_machine_unit_binding done in 5 seconds
✓ typecheck gunbc.roadmap.roadmap_dashboard_instance done in 4 seconds
✓ typecheck gunbc.host.physical_machine_binding done in 2 seconds
✓ typecheck v2.compiler.resolve done in 12 seconds
✓ typecheck v2.compiler.infer done in 18 seconds
✓ typecheck v2.compiler.name_resolve done in 8 seconds
✓ typecheck std.materialization_provider done in 7 seconds
✓ typecheck v2.compiler.target_serialize done in 5 seconds
✓ typecheck std.materialization_object done in 11 seconds
✓ typecheck v2.compiler.source_authority done in 6 seconds
✓ typecheck v2.compiler.translate done in 8 seconds
✓ typecheck extdeps.realization.materialization_store_local done in 18 seconds
✓ typecheck gunbc.commit_workflow done in 3 seconds
✓ typecheck v2.compiler.program_assembly done in 2 seconds
✓ typecheck gunbc.serving.srv1_serving_replica_layout done in 3 seconds
✓ typecheck gunbc.fleet.fleet_release_bins_key done in 2 seconds
✓ typecheck gunbc.dispatch_selection done in 6 seconds
✓ typecheck gunbc.live_deploy.spec done in 3 seconds
✓ typecheck gunbc.host.managed_host done in 5 seconds
✓ typecheck gunbc.live_deploy.desired done in 6 seconds
✓ typecheck gunbc.host.managed_host_unit_hold done in 6 seconds
✓ typecheck gunbc.host.host_boot_attempt_admission done in 3 seconds
✓ typecheck gunbc.live_deploy.repository_convergence_placement done in 2 seconds
✓ typecheck gunbc.runner.runner_slot_allocation done in 2 seconds
✓ typecheck gunbc.roadmap.roadmap_authority done in 6 seconds
✓ typecheck gunbc.ci.repository_ci_profile done in 2 seconds
✓ typecheck gunbc.compute.work_provider_local done in 10 seconds
✓ typecheck gunbc.runner.runner_throughput_qualification done in 2 seconds
✓ typecheck product.fabric.node_qualification done in 4 seconds
✓ typecheck product.fabric.capacity_admission done in 2 seconds
✓ typecheck gunbc.runner.runner_unit_file done in 2 seconds
✓ typecheck gunbc.witness.witness_floor_workflow done in 2 seconds
✓ typecheck product.fabric.selection done in 2 seconds
✓ typecheck gunbc.ci.ci_deploy_access done in 2 seconds
✓ typecheck gunbc.fabric.fabric_control_plane done in 2 seconds
✓ typecheck gunbc.runner.runner_host_unit_slot done in 4 seconds
✓ typecheck gunbc.runner.runner_slot_actuation done in 2 seconds
✓ typecheck gunbc.roadmap.roadmap_dispatch_actuator done in 8 seconds
✓ typecheck gunbc.runner.runner_lifecycle_realize done in 16 seconds
✓ typecheck gunbc.roadmap.roadmap_attempt_metering done in 2 seconds
✓ typecheck gunbc.roadmap.roadmap_belt done in 2 seconds
✓ typecheck gunbc.roadmap.roadmap_session_placement done in 3 seconds
✓ typecheck gunbc.roadmap.attempt_system_unit_launch done in 2 seconds
✓ typecheck gunbc.roadmap.roadmap_launch_admission done in 3 seconds
✓ typecheck gunbc.roadmap.roadmap_workflow_progress done in 2 seconds
✓ typecheck gunbc.roadmap.roadmap_presentation done in 8 seconds
✓ typecheck gunbc.github_effect_perform done in 2 seconds
✓ typecheck gunbc.runner.runner_host_grants done in 4 seconds
✓ typecheck gunbc.heal_publisher_workflow done in 2 seconds
✓ typecheck gunbc.roadmap.roadmap_belt_actuate done in 38 seconds
✓ typecheck gunbc.roadmap.roadmap_component done in 4 seconds
✓ typecheck gunbc.roadmap.roadmap_interaction done in 2 seconds
✓ typecheck gunbc.roadmap.roadmap_surface_style done in 9 seconds
✓ typecheck gunbc.roadmap.roadmap_style done in 11 seconds
✓ typecheck gunbc.roadmap.roadmap_page done in 13 seconds
✓ typecheck gunbc.live_deploy.emit done in 14 seconds
✓ typecheck gunbc.witness.compiler_gate_workflow done in 3 seconds
✓ typecheck gunbc.live_deploy.retract_legs done in 2 seconds
✓ typecheck gunbc.live_deploy.apply_legs done in 23 seconds
✓ typecheck gunbc.runner.runner_fleet_job_timeout_census done in 2 seconds
✓ typecheck gunbc.host.host_effect_realize done in 6 minutes
✓ typecheck gunbc.runner.runner_host_cap_bracket done in 2 seconds
✓ typecheck gunbc.live_deploy.readiness done in 4 seconds
✓ typecheck gunbc.live_deploy.member_observe done in 6 seconds
✓ typecheck gunbc.runner.runner_host_file_converge done in 11 seconds
✓ typecheck test.claim.runner.runner_host_cap_bracket_witness done in 4 seconds
[resolve] dag/test/claim/runner/runner_host_cap_bracket_witness.dag: 2095413ms  (1844 modules, 54913 resolved items in closure)
◷ post-first-entry-resolve-rss — 17.7 GiB
claim_batch: resolved dag/test/claim/runner/runner_host_cap_bracket_witness.dag; running 1 witness(es)
PASS a_cap_bracket_pass_never_masks_a_running_runner_unit
[witness] a_cap_bracket_pass_never_masks_a_running_runner_unit: cpu=530ms wall=530ms eval_steps=152986 subject=6b8e696bfdca55d8 eval_self=0.000ms
[resolve-summary] mode=Hermetic; 1 resolve(s) in 2095413ms wall; 1 witness(es) in 530ms wall
[expectation-frontier] 2 site(s), 4 dispatch(es) undeclared: Filesystem.Read=1 ssh.Session.ExecPortableWords=3
[resolve-split] load=18.1ms parse=32068.6ms resolve=2065.5ms normalize=4359.2ms typecheck=1982744.2ms parent_envs=32.8ms reconcile_assembly=1095.5ms ownership=0.0ms
[assembly-split] schedule=36936.0ms probe=0.3ms graph=479.9ms symbol_index=13800.3ms variant_base=834.9ms root_symbol_index=872.3ms root_variant_base=3156.8ms environment=5669.4ms diagnostics=13.9ms registry=55.7ms services=6044.9ms rewire_type_env=16350.6ms rewire_func_env=10664.8ms emit_info=0.0ms other=1095.5ms rewire_total_observation=27015.4ms
[cost-partition] {"basis":"summed_top_level_resolve_span_nanos","basis_note":"additive: each top-level resolve span's own duration, thread-sequential. Elapsed wall is NOT additive over concurrent spans and is carried under `observations`, never partitioned.","parent_span_nanos":2119053479015,"spans":2,"nested_spans":0,"exclusive":{"load":18088623,"parse":32068609344,"resolve_modules":2065504246,"normalize":4359231428,"typecheck_compute":1982744179615,"parent_envs":32755727,"assembly_schedule":36936027851,"assembly_probe":260202,"assembly_graph":479904052,"assembly_symbol_index":13800315984,"pool_parse":0,"assembly_variant_base":834942101,"assembly_root_symbol_index":872284392,"assembly_root_variant_base":3156838881,"assembly_environment":5669415596,"assembly_diagnostics":13894400,"assembly_registry":55683122,"assembly_services":6044859664,"assembly_rewire_type_env":16350571299,"assembly_rewire_func_env":10664791405,"assembly_emit_info":0,"reconcile_assembly":1095453765,"ownership":0},"out_of_span":{"pool_parse":11226382347},"sum_exclusive_nanos":2117263611697,"remainder_nanos":1789867318,"inclusive":{"load_reference_scan":{"nanos":0,"contained_in":"load"},"load_import_closure":{"nanos":37022531,"contained_in":"load"},"load_bare_reference_closure":{"nanos":128846996247,"contained_in":"load"},"load_pool_reference_closure":{"nanos":162861005,"contained_in":"load"},"load_bare_edge_index":{"nanos":117601709636,"contained_in":"load_bare_reference_closure"},"load_bare_path_lookup":{"nanos":12338910,"contained_in":"load_bare_reference_closure"},"load_bare_edge_walk":{"nanos":128834655617,"contained_in":"load_bare_reference_closure"},"edge_index_ref_half":{"nanos":92407799737,"contained_in":"load_bare_edge_index"},"edge_index_bare_half":{"nanos":287921,"contained_in":"load_bare_edge_index"},"edge_index_closure_expand":{"nanos":825734947,"contained_in":"edge_index_ref_half"},"edge_index_tree_census":{"nanos":24656710521,"contained_in":"edge_index_bare_half"},"edge_index_tree_census_miss_nanos":{"nanos":14566687584,"contained_in":"edge_index_tree_census"},"edge_index_bare_candidates":{"nanos":17000017,"contained_in":"edge_index_bare_half"},"edge_index_bare_name_universe":{"nanos":74601,"contained_in":"edge_index_bare_half"},"edge_index_bare_resolve_loop":{"nanos":501825928,"contained_in":"edge_index_bare_half"},"edge_index_publish":{"nanos":11741277,"contained_in":"load_bare_edge_index"}},"edge_index_construction":{"builds":1,"source_files":1844,"bare_eligible":16,"closure_expand_calls":15294,"tree_census_calls":32,"tree_census_misses":2},"pool_parse":{"builds":1,"modules":10642},"load_reference_scan_volume":{"bytes":0,"calls":0,"fixpoint_rounds":3},"accounting_law":"parent_span_nanos == sum_exclusive_nanos + remainder_nanos","verdict":{"state":"Reconciled","residual_nanos":0,"tolerance_nanos":0},"span_nanos_by_entry":[{"entry":"dag/test/claim/runner/runner_host_cap_bracket_witness.dag","spans":1,"nanos":2095411376587,"exclusive":{"load":161001,"parse":31401814558,"resolve_modules":2025115457,"normalize":4256889527,"typecheck_compute":1962807217459,"parent_envs":32235805,"reconcile_assembly":992058580,"ownership":0}},{"entry":"dag/gunbc/output_policy.dag","spans":1,"nanos":23642102428,"exclusive":{"load":17927622,"parse":666794786,"resolve_modules":40388789,"normalize":102341901,"typecheck_compute":19936962156,"parent_envs":519922,"reconcile_assembly":103395185,"ownership":0}}],"observations":{"witness_entry_resolve_wall_nanos":2095413000000,"witness_eval_wall_nanos":530000000},"observations_note":"Elapsed-wall figures from the surrounding receipts. They measure a DIFFERENT universe than `parent_span_nanos` (witness-entry resolves only, or process wall across concurrent workers) and are reported, never partitioned."}
◷ per-shard-peak-rss — 17.7 GiB
◷ children-max-rss — 8.0 GiB
[process-partition] total_ms=2529118 pre_run_ms=0 args_ms=493 index_build_ms=258411 edge_index_warm_ms=143652 output_policy_ms=23729 roster_ms=0 published_mock_precompute_ms=3510 entries_ms=2099304 reporting_ms=16 entries_resolve_ms=2095413 entries_witness_ms=530
[pre-entry] phase=source_root_walk_and_read ms=1102 scale=tree
[pre-entry] phase=pool_source_newline_index ms=12056 scale=tree
[pre-entry] phase=pool_source_tokenize ms=257835 scale=tree
[pre-entry] phase=pool_heads_parse ms=29756 scale=tree
[pre-entry] phase=module_path_index_build ms=12683 scale=tree
[pre-entry] phase=graph_facts_import_edges ms=1000 scale=tree
[pre-entry] phase=graph_facts_module_declarations ms=23 scale=tree
[pre-entry] phase=pool_census_parse ms=11226 scale=tree
[pre-entry] phase=closure_name_census_build ms=24650 scale=tree
[pre-entry] phase=reference_pool_names_from_census ms=499 scale=tree
[pre-entry] phase=selection_tier_closure_files ms=36762 scale=closure
[process-partition-exit] run_returned_ms=2529118 main_end_ms=2529118
RUN_REAL_RC=0
RUN5_DONE
```

— sent from royal-swift-552
