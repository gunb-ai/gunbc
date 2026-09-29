# Wet-witness host-premise census (2026-09-29)

Scope: every module on the local-repo wet schedule (`v2.workflow.local_repo_wet_terminal`
`local_repo_wet_schedule`, 51 modules at this revision), classified per claim for **assumed host
state** under `gunbc.recurring_failure_mode` `wet_witness_keyed_to_the_runner_not_its_subject` and
`wet_witness_premise_read_from_the_runner_it_happens_to_land_on`. Hermetic claims reach no host
effect and are out of scope. This is a read of the sources at the revision this file landed with,
not an execution receipt; the dispositions below say which rows were executed by a fix PR.

The runner pool: CI runners are fleet hosts. srv1 hosts several runner instances sharing one `/tmp`
and one network namespace, and declares a dashboard instance; srv3/srv4 do not. User managers,
memory headroom and installed tools vary.

Severity: **RUNNER-KEYED** — the verdict flips by which runner dequeues the job. **LATENT** — the
premise holds on every current runner but is neither created nor supplied by the claim.
**CLEAN** — the premise is created by the claim (per-claim `shell.Mktemp`), supplied as data, or
both arms are asserted.

## Runner-keyed

| witness | claim(s) | assumed premise | disposition |
|---|---|---|---|
| `test.claim.spark.fabric_capacity_standing_wet_witness`, `test.claim.spark.spark_pair_serving_apply_wet_witness` | all | executor is off-fleet (no dashboard instance) | repaired: #12478 supplied the premise, #12532 removed the live-log reads |
| `test.claim.compute.manager_unaskable_wet_witness` | `a_host_without_a_user_manager_reaches_the_unavailable_arm_instead_of_aborting` | no user manager answers | repaired: #12456 asserts the route relation |
| `test.claim.approval_ntfy_access_readback_wet_witness_test` | store-root claim | approval store root absent on the runner | repaired: #12614 asserts unchanged-by-the-verb |
| `test.claim.mtcollins1_census_image_local_wet` | `a_failed_or_partial_or_disagreeing_workload_emits_no_token_by_real_execution`, `a_workload_larger_than_capacity_refuses_at_preflight_by_real_execution` | fixed `/tmp/gunbc-wl-counter`, `/tmp/gunbc-wl-dd-marker` writable by this runner user (shared `/tmp`, sticky bit) | open: #12467 (per-claim temp dir), covers both paths |
| `test.claim.spark.pair_serving_d0_front_door_real_execution` | `a_vacated_port_reads_as_no_response_by_real_execution`, `a_failed_connect_fences_under_an_active_head_unit_and_suspends_only_under_a_quiet_one_by_real_execution`, `a_listener_behind_a_failed_connect_is_read_on_the_host_and_fences_by_real_execution` | a just-vacated loopback port stays unbound; `/proc/net/tcp` shows no other runner's listener on it | **owned by crisp-lynx-364** — not touched here |
| `test.claim.fabric.fabric_storage_file_store_wet_witness` | `a_put_or_advance_by_a_principal_the_store_areas_do_not_admit_is_refused_by_the_real_file_store` | runner executes as a non-root uid (`shell.Chmod.RecursiveReadOnly` does not bind root) | open — runner-keyed only if a root-executing runner exists; remedy: assert the uid→outcome relation |

## Latent

| witness | claim(s) | assumed premise | disposition |
|---|---|---|---|
| `test.claim.compute.host_capacity_wet_witness` | `competing_reservations_…`, `a_reused_reference_…`, `releasing_one_seat_twice_…` | runner has ≥2 GiB `MemAvailable` at that instant (`compute_reserve_on` read `/proc/meminfo`) | **fixed in batch 1**: reading supplied (`reserve_with_room`); real route kept by `the_real_reservation_route_admits_exactly_what_the_reading_backs`, asserted as a relation to the reading |
| `test.claim.commit_writer_heal_admission_real_execution`, `test.claim.contract_identity.required_ci_epoch_real_execution_witness`, `test.claim.devboot_text_blob_real_execution`, `test.claim.generated_artifact_merge_driver_real_execution` (merge claims) | git-backed | runner user's global/system git config (`commit.gpgsign`, `core.hooksPath`, `init.templateDir`, `merge.*`) and inherited `GIT_*` env do not perturb a temp repo | batch 2: isolate git config per claim |
| `test.claim.long.fleet_release_bins_key_witness` | positive/key claims | runner exports no credential-shaped variable; no ancestor of `mktemp -d` holds `.cargo/config*` | batch 2: supply environment (`env -i`), pin the mktemp template |
| `test.claim.effect_plan_bash_materialize_real_execution_witness` | fail-fast and metachar claims | fixed absent paths (`/gunbc-wave-a-definitely-absent`, `/tmp/absent; printf PWN`) stay absent in shared `/tmp` | batch 3: absent paths under a per-claim temp dir |
| `test.claim.fabric.fabric_storage_file_store_wet_witness` | `a_declared_entry_mode_is_the_published_mode_of_objects_and_heads` | runner umask | batch 3 |
| `test.claim.machine_intake.sol_hold_stdin_wet_witness` | both | a fixed 5 s sleep covers a `timeout 3` probe on a loaded runner | batch 3: bounded poll |
| `test.claim.approval_device_routes_wet_witness_test` | `a_bound_realization_passes_the_read_past_the_gate` | reads host-global approval store root (verdict tolerates both states) | low; supply the root at the seam when the route takes one |
| `test.claim.self_host_logic_behavioral_witness`, `test.claim.self_host_logic_seed_unavailable_witness` | receipt claims | cargo toolchain, crates registry reachable or warm, build memory | tool/network premise; typed refusal wanted |
| `test.claim.runner.runner_browser_toolchain_real_execution` | ldd readback; archive install | `ldd` present and `/bin/sh` dynamic; `curl` present | tool premise |
| `test.claim.spark.pair_serving_authority_log_real_execution` | quiet-host release | `python3`, `pgrep` | tool premise |
| `test.claim.spark.spark_pair_head_unit_digest_witness`, `spark_serving_offer_route_witness_test` (~17 claims), `v41_row_store_encode_witness`, `srv3_install_media_fetch_real_execution`, `srv3_seeded_install_media_real_execution` | digest claims | `sha256sum` on PATH | tool premise |
| `test.claim.spark.v41_engram_runtime_witness`, `v41_source_patch_converge_witness` | committed-patch reads | cwd is the repo root | subject-keyed (reads the committed tree); low |

Tool-presence premises hold uniformly across today's runners, so they cannot flip a verdict by
assignment; they are recorded so a new runner class is judged against them, and are not scheduled
for fix PRs here.

## Clean

`approval_assertion_counter_wet_witness_test`, `approval_device_enrolment_code_wet_witness_test`,
`approval_ntfy_access_readback_wet_witness_test` (decode claim), `compute.attempt_lifecycle_wet_witness`,
`durable_cas_fabric_storage_real_execution`, `durable_cas_file_store_wet_witness`,
`durable_exclusive_hold_file_store_wet_witness`, `eval_model_probe`,
`fabric.fabric_control_plane_wet_witness`, `fabric.fabric_event_log_wet_witness`,
`fabric.fabric_partition_read_wet_witness` (these two leak their temp roots — hygiene, not verdict),
`host_cli_dependency_wet_witness_test` (both arms accepted — vacuous, not keyed),
`live_deploy.approval_broker_helper_readback_real_execution`, `materialization_store_local_wet_witness`,
`megarac_spx_ui_surface_artifact_integrity_witness`, `memory_capture_real_execution_witness`,
`pre_os_capture_replay_witness`, `provenance_calibration_report_real_execution`,
`review_sheet_legacy_declaration_wet_witness`, `self_host_logic_seed_unavailable_check_fixture`,
`serving.serving_availability_bind_wet_witness`, `shell_spawn_refused_real_execution_witness`,
`spark.pair_incumbent_identity_wet_witness`, `spark.pair_serving_d0_authorization_witness`,
`spark.pair_serving_d0_real_execution`, `spark_snapshot_verify_wrapper_local_wet`,
`spark.v41_runtime_candidate_witness`.

## Absorbing-fallback finding (not a host premise)

`test.claim.fabric.fabric_storage_file_store_wet_witness` `fresh_store` drops a
`FabricStorageRootRefused` and returns the root anyway. §5 names this an absorbing fallback; queued
with batch 3.
