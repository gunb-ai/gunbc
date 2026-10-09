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
| `test.claim.fabric.fabric_storage_file_store_wet_witness` | `a_put_or_advance_by_a_principal_the_store_areas_do_not_admit_is_refused_by_the_real_file_store` | runner executes as a non-root uid (`shell.Chmod.RecursiveReadOnly` does not bind root) | #12658: uid→outcome relation plus an all-uid refusal claim |

## Latent — fixed

| witness | claim(s) | assumed premise | disposition |
|---|---|---|---|
| `test.claim.compute.host_capacity_wet_witness` | `competing_reservations_cannot_over_reserve_and_a_release_returns_the_capacity`, `a_reused_reference_is_a_ledger_refusal_and_never_reported_as_a_full_pool`, `releasing_one_seat_twice_is_not_a_ledger_failure_and_charges_nothing` | runner has ≥2 GiB `MemAvailable` at that instant | #12652: reading supplied (`reserve_with_room`); real route kept by `the_real_reservation_route_admits_exactly_what_the_reading_backs` (relation to the reading) |
| `test.claim.fabric.fabric_storage_file_store_wet_witness` | `a_put_or_advance_by_a_principal_the_store_areas_do_not_admit_is_refused_by_the_real_file_store` | runner uid ≠ 0 | #12658: uid→outcome relation; refusal route on every runner via `a_put_or_advance_into_store_areas_that_refuse_every_principal_is_a_typed_fault_by_real_execution` |
| `test.claim.long.fleet_release_bins_key_witness` | positive/key claims reaching `fixture_env` | runner exports no `release_bins_credential_pattern` name; no ancestor of `$TMPDIR` holds `.cargo/config*` | #12703: fixture unsets those names, pins the temp root under `/tmp` |
| `test.claim.machine_intake.sol_hold_stdin_wet_witness` | `the_collector_stdin_supplies_zero_bytes_and_no_eof_when_executed`, `a_launch_that_cannot_publish_its_pid_stops_the_child_it_created` | launch + `timeout 3` probe (or the supervisor's exit record) lands inside a fixed 5 s | #12704: bounded poll for the awaited line (`read_until_contains`) |

## Latent — recorded, not fixed (operator decision via deep-ferret-305, 2026-09-29)

Each row holds on every current runner, so no row flips a verdict by assignment today. "Flips when"
names the runner condition that would make the claim red for a reason other than its subject. A row
here is picked up when that condition is observed or planned for a runner class.

| witness | claim | assumed premise | why latent | flips when |
|---|---|---|---|---|
| `test.claim.commit_writer_heal_admission_real_execution` | `clean_staged_index_admits_by_real_execution`, `unmerged_index_with_no_markers_refuses_by_real_execution`, `unclaimed_text_blob_with_markers_refuses_red_control_by_real_execution`, `space_and_tab_named_paths_admit_by_real_execution` | the runner user's global/system git config and inherited `GIT_*` env do not act on a temp repo (`seed_witness_repository` sets only `user.email`/`user.name` locally) | no runner user carries such config today | a runner user gains `commit.gpgsign=true`, a `core.hooksPath` whose hooks refuse, an `init.templateDir` with hooks, or exports `GIT_DIR`/`GIT_INDEX_FILE`/`GIT_CONFIG_*` |
| `test.claim.contract_identity.required_ci_epoch_real_execution_witness` | `a_run_of_the_contract_at_another_epoch_refuses_naming_both_epochs_by_real_execution`, `a_run_of_the_contract_at_the_required_epoch_admits_by_real_execution`, `a_commit_that_is_not_in_the_repository_is_unreadable_not_absent_by_real_execution`, `a_readable_commit_without_the_workflow_is_path_absent_not_commit_unreadable_by_real_execution` | same git-config premise | same | same |
| `test.claim.devboot_text_blob_real_execution` | `a_lease_claim_is_stored_as_exactly_its_bytes_in_the_bare_store_by_real_execution`, `a_staged_entry_is_stored_as_exactly_its_bytes_in_the_worktree_repository_by_real_execution`, `a_path_routed_hash_under_a_clobbered_path_publishes_the_empty_blob_red_control_by_real_execution` | same git-config premise (no local identity is set either, so `user.*` must resolve from global config or not be needed) | same | same, or a runner user with no resolvable git identity if a path commits |
| `test.claim.generated_artifact_merge_driver_real_execution` | `divergent_generated_artifact_merge_refuses_by_real_execution`, `divergent_generated_artifact_merge_under_true_driver_silently_drops_theirs_red_control_by_real_execution`, `one_sided_generated_artifact_change_merges_clean_by_real_execution` | same git-config premise, plus no global `merge.*` driver or attribute file overriding the locally configured driver | same | same, or a global `core.attributesFile` / `merge.<name>.driver` naming the witnessed path |
| `test.claim.effect_plan_bash_materialize_real_execution_witness` | `effect_plan_bash_fail_fast_prevents_later_operation_execution`, `effect_plan_bash_if_branch_fail_fast_stops_the_branch_and_the_plan` | `/gunbc-wave-a-definitely-absent` does not exist | nothing creates it | anything on the runner creates that root-level path |
| `test.claim.effect_plan_bash_materialize_real_execution_witness` | `effect_plan_bash_metachar_and_newline_cannot_open_a_statement` | `/tmp/absent; printf PWN` and `/tmp/absent\nid -u` do not exist in shared `/tmp` | only a deliberate write creates them | another runner instance on a shared-`/tmp` host (srv1) leaves such a path behind |
| `test.claim.fabric.fabric_storage_file_store_wet_witness` | `a_declared_entry_mode_is_the_published_mode_of_objects_and_heads` | the runner's umask leaves the undeclared mode ≠ `0444` | runners use the default umask 022/002 | a runner runs with umask `0333` or stricter |
| `test.claim.approval_device_routes_wet_witness_test` | `a_bound_realization_passes_the_read_past_the_gate` | a read of the host-global approval store root answers (present, absent or unreadable) rather than aborting | the verdict accepts every answering state | a permission error on that root aborts the route instead of producing a typed body |
| `test.claim.self_host_logic_behavioral_witness` | `self_host_logic_behavioral_receipt_holds` | cargo/rustc present, crates registry reachable or warm, memory and disk for a release build | every runner has the toolchain and a warm registry | a runner without network and a cold `~/.cargo`, or below the build's memory |
| `test.claim.self_host_logic_seed_unavailable_witness` | `self_host_logic_seed_unavailable_receipt_holds` | same, plus `git` | same | same |
| `test.claim.runner.runner_browser_toolchain_real_execution` | `the_library_readback_runs_ldd_and_reads_a_resolved_binary_by_real_execution` | `ldd` present and `/bin/sh` dynamically linked | glibc runners | a musl/static-shell runner, or one without `ldd` |
| `test.claim.runner.runner_browser_toolchain_real_execution` | `a_verified_archive_installs_and_its_retained_bytes_read_back_verified_by_real_execution`, `a_sha256_mismatch_refuses_and_runs_no_install_by_real_execution`, `an_npm_integrity_mismatch_refuses_and_runs_no_install_by_real_execution`, `a_retained_archive_altered_after_install_reads_back_mismatched_by_real_execution` | `curl` on PATH | present everywhere | a runner without `curl` |
| `test.claim.spark.pair_serving_authority_log_real_execution` | `a_release_needs_the_host_observed_quiet_by_real_execution` | `python3` and `pgrep` on PATH | present everywhere | a runner without either |
| `test.claim.spark.spark_pair_head_unit_digest_witness`, `test.claim.spark.spark_serving_offer_route_witness_test` (the claims reaching `spark_pair_declared_head_digest`), `test.claim.spark.v41_row_store_encode_witness` `the_receipt_was_produced_by_the_rendered_program_the_mode_ships`, `test.claim.srv3_install_media_fetch_real_execution`, `test.claim.srv3_seeded_install_media_real_execution` `install_media_remaster_ensure_grub_cmdline_inserts_by_real_execution` | digest claims | `sha256sum` on PATH | coreutils everywhere | a runner without coreutils `sha256sum` |
| `test.claim.spark.v41_engram_runtime_witness` (`the_live_storage_backed_patches_are_digestfile_of_committed_bytes`, `overlay_magic_cites_the_row_store_authority`), `test.claim.spark.v41_source_patch_converge_witness` `digestfile_inhabits_the_committed_patch_population` | committed-patch reads | cwd is the repo root | the wet executor runs from the checkout root | the executor's cwd changes |

| `test.claim.machine_intake.sol_hold_stdin_wet_witness` | the watch, establishment, notice, release and publication claims #12434 added (`an_end_written_before_exit_is_judged_on_the_final_bytes`, `a_collector_gone_with_the_envelope_open_is_channel_lost_and_frozen`, `an_authentic_client_exit_is_classified_by_its_own_words`, `the_watcher_publishes_a_collector_exit_without_the_boot_watch`, `an_adopted_collector_needs_a_receipt_naming_its_instance`, `the_capture_watch_stops_when_its_observer_dies_and_waits_while_it_lives`, `the_release_signals_only_the_recorded_instance`, `a_publication_failure_stops_even_a_term_ignoring_child`, and the claims reaching `observer_alive` / `owned_release`) | a launched probe publishes its pid or reaches its state within a fixed 1–7 s sleep | runners start a `sh` probe in well under a second | a runner loaded enough that spawn + publication exceeds the fixed sleep; remedy is the bounded `read_until_contains` poll #12704 added for the two stdin claims |
The spark/v41 rows are keyed to their subject (the committed tree) rather than to the runner; they
are listed for completeness.

## Clean

`approval_assertion_counter_wet_witness_test`, `approval_device_enrolment_code_wet_witness_test`,
`approval_ntfy_access_readback_wet_witness_test` (decode claim), `compute.attempt_lifecycle_wet_witness`,
`durable_cas_fabric_storage_real_execution`, `durable_cas_file_store_wet_witness`,
`durable_exclusive_hold_file_store_wet_witness`, `eval_model_probe`,
`fabric.fabric_control_plane_wet_witness`, `fabric.fabric_event_log_wet_witness`,
`fabric.fabric_partition_read_wet_witness` (these two leak their temp roots — hygiene, not verdict),
`host_cli_dependency_wet_witness_test` (both arms accepted — vacuous, not keyed),
`live_deploy.approval_broker_helper_readback_real_execution`, `materialization_store_local_wet_witness`,
`memory_capture_real_execution_witness`,
`pre_os_capture_replay_witness`, `provenance_calibration_report_real_execution`,
`review_sheet_legacy_declaration_wet_witness`, `self_host_logic_seed_unavailable_check_fixture`,
`serving.serving_availability_bind_wet_witness`, `shell_spawn_refused_real_execution_witness`,
`spark.pair_incumbent_identity_wet_witness`, `spark.pair_serving_d0_authorization_witness`,
`spark.pair_serving_d0_real_execution`, `spark_snapshot_verify_wrapper_local_wet`,
`spark.v41_runtime_candidate_witness`.

## Absorbing-fallback finding (not a host premise)

`test.claim.fabric.fabric_storage_file_store_wet_witness` `fresh_store` drops a
`FabricStorageRootRefused` and returns the root anyway. §5 names this an absorbing fallback; queued
with the recorded rows; not scheduled.
