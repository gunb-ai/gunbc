# USES-0 census: authored `uses` joined to body-derived demand

Authority: operator D13 (2026-09-18), recorded in `gunbc.plans.demand_engine_program`. This analysis is the identity-grain join. It does not delete or add any `uses` row.

## Method

Two independently produced populations:

1. **Authored `uses` rows** — every parsed function/pattern header clause `uses alias: Type` in committed `.dag` source (not AST field `uses:`, not GHA `uses:`, not comments). Identity is `(module file, declaration, alias, type)`.
2. **Body-derived demand (session approximation of DependencyDemand)** — capitalized dotted operation references in the visible body that inhabit the authored resource, plus `file_compare_and_set` (Filesystem) and `clock_now_probed_at` (Clock); plus calls to other declarations that themselves author a `uses` of the same normalized resource (`Network` = `std.resources.Network`).

A dotted call is **not** Network demand merely because it is a host operation. `Filesystem.*` is Filesystem. `shell.Mktemp`, `shell.Remove`, `shell.Move`, `shell.Env`, `os.Hostname`, and `git.Inspect` / `git.Core` are local and do not establish Network. Classifying `read_docker_hub_image_config_at_digest` as DirectRestatement from `shell.Mktemp` was that error (review 67467): the Network work is `docker_hub_bearer_token` → `http.Client.GetBounded` with no `uses` on those helpers, which is Unresolved.

This is **not** the compiler `DependencyDemand` carrier. Whole-corpus resolve that would produce that carrier is CI-sized (the seed cannot hold the v2 closure under a session cap). Rows the source join cannot decide are `Unresolved` — that class is the CI-side remainder, not a count to close by hand. No cardinality is an oracle; membership is the table. A helper call that is not itself a `uses` declaration, and is not a known pure combinator, is insufficient call relation: class Unresolved, even if the body also shows a *different* resource.

## Class dispositions (D13)

| Class | When | Disposition |
| --- | --- | --- |
| DirectRestatement | body directly references a resolved operation of this resource | delete/derive |
| TransitiveRestatement | body only calls another declaration that requires it | delete/derive over call relations |
| NamedDependencyBinder | a distinct logical provider identity actually used in the body | move to dependency-subject or capability param |
| OpaqueContract | no body; requirement not derivable | keep as contract |
| PolicyEnvelope | constrains what may be used, not actual use | move to policy |
| UnusedRequirement | no body path consumes this authored identity | delete/refuse |
| Unresolved | identity/call relations insufficient (typically a helper without `uses` that still performs host work) | typed refusal until resolution names the relation |

## Empty classes in the live parsed population

- **OpaqueContract** — every live `uses` clause is followed by a visible body. The interesting inverse is `gunbc.clock_read` `clock_now_probed_at`, which calls `Clock.Now()` with **no** authored `uses`.
- **PolicyEnvelope** — no header constrains a derived set (no may-not-use / upper-bound form exists in source).
- **NamedDependencyBinder** — every live alias is the generic interface (`net: Network`, `fs: Filesystem`, `clock: Clock`). No row distinguishes `source_tree` from `output_tree`. Fixture strings that bind `net: EaNet` / `GewNet` / `ZpeNet` and read `net.id` are the binder-shaped specimens, listed separately.

Likely outcome per the ruling: the function-level `uses` clause disappears for transparent bodies; what would survive is opaque/subject/policy, of which the live corpus currently has none.

## DirectRestatement

| File | Declaration | Alias | Authored type | Evidence |
| --- | --- | --- | --- | --- |
| `dag/gunbc/assimilate/bmc_bootstrap_provision.dag` | `bmc_bootstrap_provision_srv3` | `net` | `std.resources.Network` | ops shell.GCloud.AuthPrintAccessToken, gcp.ServiceUsage.BatchEnableServices, gcp.IamAdmin.CreateServiceAccount, gcp.SecretManager.GetSecretIamPolicy, gcp.SecretManager.SetSecretIamPolicy |
| `dag/gunbc/auth/access_token_source.dag` | `ensure_access_token` | `net` | `Network` | ops shell.GCloud.AuthPrintAccessToken; local-ops shell.Env.Get |
| `dag/gunbc/auth/credentials.dag` | `gcp_secret_credential` | `net` | `std.resources.Network` | ops shell.GCloud.AuthPrintAccessToken, gcp.SecretManager.AccessVersion |
| `dag/gunbc/auth/credentials.dag` | `gcp_adc_token_from_credentials` | `net` | `std.resources.Network` | ops oauth2.Google.Refresh |
| `dag/gunbc/auth/gcp_secret_access.dag` | `secret_access_ensure_for` | `net` | `Network` | ops gcp.SecretManager.GetSecretIamPolicy, gcp.SecretManager.SetSecretIamPolicy; callees ensure_access_token |
| `dag/gunbc/auth/patterns.dag` | `credential_chain` | `net` | `std.resources.Network` | ops gcp.STS.Exchange, gcp.SecretManager.AccessVersion |
| `dag/gunbc/auth/patterns.dag` | `github_oidc` | `net` | `std.resources.Network` | ops github.OIDC.GetToken; local-ops shell.Env.Get, shell.Env.Get |
| `dag/gunbc/auth/patterns.dag` | `metadata_oidc` | `net` | `std.resources.Network` | ops gcp.Metadata.GetIdentityToken |
| `dag/gunbc/auth/secret_ref_credential.dag` | `fetch_secret_ref_credential_with_token` | `net` | `Network` | ops gcp.SecretManager.AccessVersion |
| `dag/gunbc/command_runner.dag` | `run_shell_command` | `net` | `Network` | ops shell.Exec.RunArgv |
| `dag/gunbc/command_runner.dag` | `run_shell_command_observe` | `net` | `Network` | ops shell.Exec.RunArgv |
| `dag/gunbc/host/host_reset_return_run.dag` | `host_is_reachable` | `net` | `std.resources.Network` | ops ssh.Session.Reachability |
| `dag/gunbc/machine_intake/megarac_media_attach.dag` | `megarac_attach_remote_image` | `net` | `std.resources.Network` | ops megarac.Media.OpenSession; callees megarac_attach_then_release, megarac_release_unreadable_session |
| `dag/gunbc/machine_intake/megarac_media_attach.dag` | `megarac_release_unreadable_session` | `net` | `std.resources.Network` | ops megarac.Media.CloseSession |
| `dag/gunbc/machine_intake/megarac_media_attach.dag` | `megarac_attach_then_release` | `net` | `std.resources.Network` | ops megarac.Media.CloseSession; callees megarac_attach_with_token |
| `dag/gunbc/machine_intake/megarac_media_attach.dag` | `megarac_attach_with_token` | `net` | `std.resources.Network` | ops megarac.Media.GetRemoteConfigurations, megarac.Media.GetRemoteImages; callees megarac_start_and_confirm |
| `dag/gunbc/machine_intake/megarac_media_attach.dag` | `megarac_start_and_confirm` | `net` | `std.resources.Network` | ops megarac.Media.StartMedia, megarac.Media.GetRemoteConfigurations |
| `dag/gunbc/machine_intake/oob_boot_handoff.dag` | `oob_power_cycle_after_confirmed_selection` | `net` | `std.resources.Network` | ops diagnostic.ipmi.Tool.ChassisPowerControl |
| `dag/gunbc/machine_intake/oob_boot_handoff.dag` | `oob_boot_handoff` | `net` | `std.resources.Network` | ops diagnostic.ipmi.Tool.ChassisBootDev, diagnostic.ipmi.Tool.ChassisBootParamGet; callees oob_power_cycle_after_confirmed_selection |
| `dag/gunbc/roadmap/roadmap_launch_deployment_cli.dag` | `rlm_predecessor_run` | `net` | `Network` | ops github.WorkflowRuns.GetRun |
| `dag/gunbc/srv3/srv3_boot_once_cd.dag` | `srv3_boot_once_cd_gated` | `clock` | `std.resources.Clock` | ops clock_now_probed_at |
| `dag/gunbc/srv3/srv3_boot_once_cd.dag` | `srv3_claim_boot_once_cd_authorization` | `fs` | `std.resources.Filesystem` | ops file_compare_and_set |
| `dag/gunbc/srv3/srv3_boot_once_cd.dag` | `srv3_settle_boot_once_cd_claim` | `fs` | `std.resources.Filesystem` | ops file_compare_and_set |
| `dag/gunbc/srv3/srv3_boot_once_cd.dag` | `srv3_boot_once_cd_resolved` | `net` | `std.resources.Network` | ops Filesystem.Write, redfish.Http.SetBootSourceOverride, Filesystem.Write, redfish.Http.ResetSystem; callees materialize_bmc_login_password; other-demand Filesystem |
| `dag/gunbc/tools/bmc_first_contact.dag` | `bmc_first_contact` | `net` | `std.resources.Network` | ops redfish.Http.GetSystem; local-ops shell.Env.Get |
| `dag/gunbc/tools/bmc_onboard.dag` | `bmc_rotate_credential` | `net` | `Network` | ops Filesystem.Write, redfish.Http.SetAccountPassword, redfish.Http.GetSystem; other-demand Filesystem |
| `dag/gunbc/tools/bmc_onboard.dag` | `bmc_store_and_rotate` | `net` | `Network` | ops gcp.SecretManager.AddVersion, gcp.SecretManager.AccessVersion; callees ensure_access_token, bmc_rotate_credential |
| `dag/gunbc/tools/bmc_onboard.dag` | `bmc_assimilate_credential` | `net` | `Network` | ops redfish.Http.GetServiceRoot, redfish.Http.GetSystem; callees bmc_store_and_rotate |
| `dag/gunbc/tools/bmc_onboard.dag` | `bmc_probe_credential_phase` | `net` | `Network` | ops redfish.Http.GetSystem, gcp.SecretManager.AccessVersion, redfish.Http.GetSystem; callees ensure_access_token |
| `dag/gunbc/tools/bmc_onboard.dag` | `bmc_converge_credential` | `net` | `Network` | ops redfish.Http.GetServiceRoot; callees bmc_store_and_rotate, bmc_probe_credential_phase |
| `dag/gunbc/tools/bmc_read_telemetry.dag` | `bmc_read_telemetry` | `net` | `std.resources.Network` | ops redfish.Http.GetSystem, redfish.Http.GetChassisSensors; local-ops shell.Env.Get, shell.Env.Get |
| `dag/test/ctl/pos_emit.dag` | `emits_a_service_call` | `net` | `std.resources.Network` | ops diagnostic.ipmi.Tool.SensorList |
| `dag/test/ctl/pos_emit2.dag` | `emits_a_two_segment_call` | `net` | `std.resources.Network` | ops cargo.Build.Run |
| `dag/test/ctl/pos_service.dag` | `call_three_segment` | `net` | `std.resources.Network` | ops diagnostic.ipmi.Tool.SensorList |

## TransitiveRestatement

| File | Declaration | Alias | Authored type | Evidence |
| --- | --- | --- | --- | --- |
| `dag/gunbc/auth/credentials.dag` | `gcp_oauth_access_token_via_adc_refresh` | `net` | `std.resources.Network` | ops Filesystem.Read; callees gcp_adc_token_from_credentials; other-demand Filesystem |
| `dag/gunbc/auth/credentials.dag` | `gcp_oauth_access_token_adc_for_path` | `net` | `std.resources.Network` | callees gcp_oauth_access_token_via_adc_refresh |
| `dag/gunbc/auth/secret_access_admission.dag` | `secret_access_ensure_admitted` | `net` | `Network` | callees secret_access_ensure_for |
| `dag/gunbc/auth/secret_ref_credential.dag` | `fetch_secret_ref_credential` | `net` | `Network` | callees fetch_secret_ref_credential_with_token, ensure_access_token |
| `dag/gunbc/bmc/bmc_fan_converge.dag` | `bmc_fan_run` | `net` | `Network` | callees fetch_secret_ref_credential |
| `dag/gunbc/bmc/bmc_fan_converge.dag` | `srv3_bmc_fan_observe` | `net` | `Network` | callees bmc_fan_run |
| `dag/gunbc/bmc/bmc_fan_converge.dag` | `srv3_bmc_fan_converge` | `net` | `Network` | callees bmc_fan_run |
| `dag/gunbc/bmc/bmc_fan_converge.dag` | `srv4_bmc_fan_observe` | `net` | `Network` | callees bmc_fan_run |
| `dag/gunbc/bmc/bmc_fan_converge.dag` | `srv4_bmc_fan_converge` | `net` | `Network` | callees bmc_fan_run |
| `dag/gunbc/bmc/bmc_fan_converge.dag` | `srv4_bmc_fan_rollback_previous` | `net` | `Network` | callees bmc_fan_run |
| `dag/gunbc/bmc/bmc_netboot_serve.dag` | `bmc_netboot_run_bmc` | `net` | `Network` | callees run_shell_command |
| `dag/gunbc/bmc/bmc_netboot_serve.dag` | `bmc_netboot_put_path` | `net` | `Network` | callees run_shell_command |
| `dag/gunbc/bmc/bmc_netboot_serve.dag` | `bmc_netboot_stage_file` | `net` | `Network` | callees bmc_netboot_stage_inline, bmc_netboot_put_path |
| `dag/gunbc/bmc/bmc_netboot_serve.dag` | `bmc_netboot_stage_inline` | `net` | `Network` | ops Filesystem.Write; callees bmc_netboot_put_path; other-demand Filesystem |
| `dag/gunbc/bmc/bmc_netboot_serve.dag` | `bmc_netboot_provision` | `net` | `Network` | callees bmc_netboot_run_bmc, bmc_netboot_stage_file |
| `dag/gunbc/bmc/bmc_netboot_serve.dag` | `srv4_bmc_netboot_provision` | `net` | `Network` | callees bmc_netboot_provision |
| `dag/gunbc/busybox_bmc_build.dag` | `busybox_bmc_build_run` | `net` | `Network` | callees run_shell_commands |
| `dag/gunbc/command_runner.dag` | `run_shell_command_capture` | `net` | `Network` | callees run_shell_command_observe |
| `dag/gunbc/command_runner.dag` | `run_shell_commands` | `net` | `Network` | callees run_shell_command |
| `dag/gunbc/fleet/fleet_converge_plan_cli.dag` | `observe_effective_user_wet` | `net` | `Network` | callees run_shell_command_capture |
| `dag/gunbc/fleet/fleet_converge_plan_cli.dag` | `observe_host_short_wet` | `net` | `Network` | callees run_shell_command_capture |
| `dag/gunbc/fleet/fleet_converge_plan_cli.dag` | `observe_timer_members_wet` | `net` | `Network` | callees run_shell_command_capture |
| `dag/gunbc/fleet/fleet_converge_plan_cli.dag` | `observe_fleet_converge_request_wet` | `net` | `Network` | callees observe_runner_unit_standings_wet, observe_slot_retirement_evidence_wet, observe_runner_slot_members_with_provenance_wet, observe_timer_members_wet, observe_cap_members_wet |
| `dag/gunbc/fleet/fleet_converge_plan_cli.dag` | `observe_slot_retirement_evidence_wet` | `net` | `Network` | callees observe_transition_retirement_evidence |
| `dag/gunbc/fleet/fleet_converge_plan_cli.dag` | `observe_runner_unit_standings_wet` | `net` | `Network` | callees observe_runner_unit_standing_wet |
| `dag/gunbc/fleet/fleet_converge_plan_cli.dag` | `write_fleet_converge_plan_artifact_wet` | `net` | `Network` | ops Filesystem.Write, Filesystem.Write, Filesystem.Write, Filesystem.Write, Filesystem.Write; callees run_shell_commands; other-demand Filesystem |
| `dag/gunbc/fleet/fleet_converge_plan_cli.dag` | `fleet_converge_allocation_store_plan_wet` | `net` | `Network` | callees write_fleet_converge_plan_artifact_wet, observe_generation_store_wet, observe_host_short_wet |
| `dag/gunbc/fleet/fleet_converge_plan_cli.dag` | `fleet_converge_plan_wet` | `net` | `Network` | callees fleet_converge_scoped_plan_wet |
| `dag/gunbc/fleet/fleet_converge_plan_cli.dag` | `fleet_converge_launch_environment_plan_wet` | `net` | `Network` | callees fleet_converge_scoped_plan_wet |
| `dag/gunbc/fleet/fleet_converge_plan_cli.dag` | `fleet_converge_scoped_plan_wet` | `net` | `Network` | callees fleet_converge_plan_scoped_bound_wet |
| `dag/gunbc/fleet/fleet_converge_plan_cli.dag` | `fleet_converge_plan_scoped_bound_wet` | `net` | `Network` | ops Filesystem.Write, Filesystem.Write, Filesystem.Write, Filesystem.Write, Filesystem.Write; callees run_shell_commands, observe_generation_store_wet, observe_host_short_wet, observe_fleet_converge_request_wet; other-demand Filesystem |
| `dag/gunbc/fleet/fleet_converge_plan_cli.dag` | `fleet_converge_apply_wet` | `net` | `Network` | callees fleet_converge_apply_bound_wet |
| `dag/gunbc/fleet/fleet_converge_plan_cli.dag` | `fleet_converge_apply_bound_wet` | `net` | `Network` | ops Filesystem.Read, Filesystem.Read, Filesystem.Read, Filesystem.Read, Filesystem.Read; callees run_shell_commands, observe_effective_user_wet, observe_host_short_wet, observe_fleet_converge_request_wet; other-demand Filesystem |
| `dag/gunbc/fleet/fleet_converge_plan_cli.dag` | `fleet_converge_local_plan_wet` | `net` | `Network` | callees observe_host_short_wet, observe_operator_local_run_binding, fleet_converge_plan_scoped_bound_wet |
| `dag/gunbc/fleet/fleet_converge_plan_cli.dag` | `fleet_converge_operator_cli_entry` | `net` | `Network` | ops Filesystem.Read, Filesystem.Write; callees fleet_converge_operator_cli_dispatch, observe_operator_local_run_binding; other-demand Filesystem |
| `dag/gunbc/fleet/fleet_converge_plan_cli.dag` | `fleet_converge_operator_cli_dispatch` | `net` | `Network` | local-ops git.Core.CurrentBranch; callees fleet_converge_local_apply_wet, fleet_converge_local_plan_wet, observe_host_short_wet |
| `dag/gunbc/fleet/fleet_converge_plan_cli.dag` | `fleet_converge_local_apply_wet` | `net` | `Network` | ops Filesystem.Read; callees observe_operator_local_run_binding, fleet_converge_apply_bound_wet; other-demand Filesystem |
| `dag/gunbc/fleet/fleet_converge_plan_cli.dag` | `observe_runner_slot_provenance_wet` | `net` | `Network` | callees run_shell_command_capture |
| `dag/gunbc/fleet/fleet_converge_plan_cli.dag` | `observe_runner_slot_members_with_provenance_wet` | `net` | `Network` | callees observe_runner_slot_provenance_wet, observe_runner_slot_members_wet |
| `dag/gunbc/fleet/fleet_host_key_enrollment.dag` | `verify_enrollment_outcome` | `net` | `Network` | callees run_shell_command_capture |
| `dag/gunbc/fleet/fleet_host_key_enrollment.dag` | `fleet_host_key_enroll_apply` | `net` | `Network` | ops Filesystem.Write; callees run_shell_commands, run_shell_command_capture; other-demand Filesystem |
| `dag/gunbc/fleet/fleet_host_key_enrollment.dag` | `fleet_host_key_enroll_wet` | `net` | `Network` | ops Filesystem.Write; callees fleet_host_key_enroll_apply; other-demand Filesystem |
| `dag/gunbc/fleet/fleet_host_key_enrollment.dag` | `fleet_authorized_key_self_enroll_wet` | `net` | `Network` | ops Filesystem.Write, Filesystem.Write; callees verify_enrollment_outcome, run_shell_command_capture; other-demand Filesystem |
| `dag/gunbc/fleet/fleet_multi_principal_probe.dag` | `probe_agent_credential_verification` | `net` | `Network` | callees verify_fleet_ssh_agent_credential, materialize_agent_public_selector |
| `dag/gunbc/fleet/fleet_multi_principal_probe.dag` | `fleet_multi_principal_probe_with_attempt` | `net` | `Network` | ops Filesystem.Write, clock_now_probed_at; callees materialize_fleet_known_hosts_anchor, run_with_materialized_fleet_ssh_binding; other-demand Clock,Filesystem |
| `dag/gunbc/fleet/fleet_multi_principal_probe.dag` | `spark_exact_grant_probe_wet` | `net` | `Network` | ops Filesystem.Write; callees materialize_fleet_known_hosts_anchor, run_with_materialized_fleet_ssh_binding; other-demand Filesystem |
| `dag/gunbc/fleet/fleet_multi_principal_probe.dag` | `spark_managed_access_standing_for_attempt` | `net` | `Network` | callees spark_managed_access_standing_with_verified_credential, probe_agent_credential_verification |
| `dag/gunbc/fleet/fleet_probe_identity_observe.dag` | `fleet_probe_identity_observe_wet` | `net` | `Network` | ops Filesystem.Write; callees run_shell_command_capture; other-demand Filesystem |
| `dag/gunbc/fleet/fleet_ssh_credential_verify.dag` | `derive_identity_file_public_key_line` | `net` | `Network` | callees run_shell_command_capture |
| `dag/gunbc/fleet/fleet_ssh_credential_verify.dag` | `verify_fleet_ssh_credential` | `net` | `Network` | callees derive_identity_file_public_key_line |
| `dag/gunbc/fleet/fleet_ssh_credential_verify.dag` | `list_agent_identities` | `net` | `Network` | callees run_shell_command_capture |
| `dag/gunbc/fleet/fleet_ssh_credential_verify.dag` | `verify_fleet_ssh_agent_credential` | `net` | `Network` | callees list_agent_identities |
| `dag/gunbc/fleet/fleet_ssh_locus.dag` | `prepare_fleet_ssh_agent_context` | `net` | `Network` | callees fleet_ssh_context_from_verified_binding, verify_fleet_ssh_agent_credential, materialize_agent_public_selector |
| `dag/gunbc/fleet/fleet_ssh_locus.dag` | `fleet_ssh_context_from_verified_binding` | `net` | `Network` | callees materialize_fleet_known_hosts_anchor |
| `dag/gunbc/fleet/org_actions_converge.dag` | `org_admin_app_key_access_converge_with_supplied_token` | `net` | `Network` | callees secret_access_ensure_for |
| `dag/gunbc/fleet/printer_job_start.dag` | `acquire_pinned_ca_bundle` | `net` | `Network` | local-ops shell.Mktemp.Dir; callees run_shell_command_capture |
| `dag/gunbc/fleet/printer_job_start.dag` | `start_print_on_printer` | `net` | `Network` | callees publish_start_with_credential, fetch_secret_ref_credential |
| `dag/gunbc/fleet/printer_job_start.dag` | `publish_start_with_credential` | `net` | `Network` | local-ops shell.Remove.RecursiveForce, shell.Remove.RecursiveForce, shell.Remove.RecursiveForce; callees acquire_pinned_ca_bundle, run_shell_command_capture |
| `dag/gunbc/fleet/printer_project_delivery.dag` | `deliver_project_to_printer` | `net` | `Network` | local-ops shell.Remove.FileForce, shell.Remove.FileForce; callees fetch_secret_ref_credential, run_shell_command_capture |
| `dag/gunbc/host/host_build_cache_provision.dag` | `provision_build_cache` | `net` | `Network` | callees host_toolchain_ensure |
| `dag/gunbc/host/host_codex_runtime_provision.dag` | `provision_codex_runtime` | `net` | `Network` | callees host_toolchain_ensure |
| `dag/gunbc/host/host_compile_pool_provision.dag` | `provision_compile_pool` | `net` | `Network` | callees host_toolchain_ensure |
| `dag/gunbc/host/host_reset_return_run.dag` | `watch_step` | `net` | `std.resources.Network` | callees host_is_reachable |
| `dag/gunbc/host/host_reset_return_run.dag` | `watch_for` | `net` | `std.resources.Network` | callees watch_step |
| `dag/gunbc/host/host_reset_return_run.dag` | `observe_reset_return` | `net` | `std.resources.Network` | callees watch_for |
| `dag/gunbc/host/host_reset_return_run.dag` | `subject_baseline` | `net` | `std.resources.Network` | callees host_is_reachable |
| `dag/gunbc/host/host_reset_return_run.dag` | `host_reset_return_attempt` | `net` | `std.resources.Network` | callees host_reset_return_issue_and_watch, observer_self_corroboration, subject_baseline |
| `dag/gunbc/host/host_reset_return_run.dag` | `host_reset_return_issue_and_watch` | `net` | `std.resources.Network` | callees oob_boot_handoff, observe_reset_return |
| `dag/gunbc/host/host_reset_return_run.dag` | `host_reset_return_wet` | `net` | `std.resources.Network` | local-ops shell.Env.Get, shell.Env.Get; callees host_reset_return_exit, host_reset_return_attempt, host_reset_return_preflight_exit, host_reset_return_converge_route_read |
| `dag/gunbc/instruments/pair_incumbent_identity_standing.dag` | `group_identity_standing` | `net` | `Network` | callees prepare_fleet_ssh_agent_context |
| `dag/gunbc/instruments/pair_incumbent_identity_standing.dag` | `pair_incumbent_identity_standing` | `net` | `Network` | ops Filesystem.Write, clock_now_probed_at; callees group_identity_standing; other-demand Clock,Filesystem |
| `dag/gunbc/live_deploy/deployed_tree_report.dag` | `deployed_tree_report_ci_wet` | `net` | `Network` | callees deployed_tree_report_for_context, prepare_fleet_ssh_agent_context |
| `dag/gunbc/machine_intake/boot_image_fetch.dag` | `fetch_verified_boot_image` | `net` | `std.resources.Network` | callees run_shell_command, verify_and_publish_boot_image |
| `dag/gunbc/machine_intake/boot_image_fetch.dag` | `verify_and_publish_boot_image` | `net` | `std.resources.Network` | callees admit_then_publish_boot_image |
| `dag/gunbc/machine_intake/boot_image_fetch.dag` | `admit_then_publish_boot_image` | `net` | `std.resources.Network` | callees publish_verified_boot_image |
| `dag/gunbc/machine_intake/boot_image_fetch.dag` | `publish_verified_boot_image` | `net` | `std.resources.Network` | local-ops shell.Move.File; callees confirm_published_boot_image |
| `dag/gunbc/machine_intake/mtcollins1_actuate.dag` | `mtcollins1_drive_handoff` | `net` | `std.resources.Network` | callees oob_boot_handoff |
| `dag/gunbc/machine_intake/mtcollins1_actuate.dag` | `mtcollins1_netboot_handoff` | `net` | `std.resources.Network` | callees mtcollins1_drive_handoff |
| `dag/gunbc/machine_intake/mtcollins1_actuate.dag` | `mtcollins1_cdrom_handoff` | `net` | `std.resources.Network` | callees mtcollins1_drive_handoff |
| `dag/gunbc/machine_intake/mtcollins1_actuate.dag` | `mtcollins1_denied_admission_control` | `net` | `std.resources.Network` | callees mtcollins1_drive_handoff |
| `dag/gunbc/machine_intake/mtcollins1_boot_image_fetch.dag` | `mtcollins1_fetch_boot_image` | `net` | `std.resources.Network` | callees fetch_verified_boot_image |
| `dag/gunbc/machine_intake/mtcollins1_boot_image_fetch.dag` | `mtcollins1_fetch_boot_image_wrong_pin_control` | `net` | `std.resources.Network` | callees fetch_verified_boot_image |
| `dag/gunbc/machine_intake/mtcollins1_boot_image_fetch.dag` | `mtcollins1_fetch_boot_image_unowned_export_control` | `net` | `std.resources.Network` | callees fetch_verified_boot_image |
| `dag/gunbc/machine_intake/mtcollins1_media_attach.dag` | `mtcollins1_attach_absent_image_control` | `net` | `std.resources.Network` | callees megarac_attach_remote_image |
| `dag/gunbc/machine_intake/mtcollins1_media_attach.dag` | `mtcollins1_attach_diskless_image` | `net` | `std.resources.Network` | callees megarac_attach_remote_image |
| `dag/gunbc/product/printed_chassis/slicing_toolchain.dag` | `attempt_orca_execution` | `net` | `Network` | callees run_shell_command_capture |
| `dag/gunbc/roadmap/roadmap_launch_deployment_cli.dag` | `persist_receipt` | `net` | `Network` | ops Filesystem.Write, Filesystem.Write, Filesystem.WriteOwnerOnly; callees run_shell_commands; other-demand Filesystem |
| `dag/gunbc/roadmap/roadmap_launch_deployment_cli.dag` | `rlm_launch_deployment_receipt_wet` | `net` | `Network` | callees rlm_launch_deployment_receipt_bound, prepare_fleet_ssh_agent_context |
| `dag/gunbc/roadmap/roadmap_launch_deployment_cli.dag` | `rlm_run_provenance_fact` | `net` | `Network` | callees rlm_predecessor_run |
| `dag/gunbc/roadmap/roadmap_launch_deployment_cli.dag` | `rlm_launch_deployment_receipt_bound` | `net` | `Network` | ops Filesystem.Write, clock_now_probed_at_or_unknown; callees window_snapshot, observe_host_short_wet, rlm_run_provenance_fact, persist_receipt, unit_standing_fact; other-demand Clock,Filesystem |
| `dag/gunbc/runner/runner_guest_image.dag` | `runner_guest_image_observe_wet` | `net` | `Network` | callees run_shell_commands |
| `dag/gunbc/runner/runner_guest_image.dag` | `runner_guest_image_converge_wet` | `net` | `Network` | callees run_shell_commands |
| `dag/gunbc/runner/runner_host_file_converge.dag` | `observe_runner_host_file` | `net` | `Network` | callees observe_runner_host_file_content, observe_runner_host_paths |
| `dag/gunbc/runner/runner_host_file_converge.dag` | `apply_runner_host_file` | `net` | `Network` | callees observe_runner_host_paths |
| `dag/gunbc/runner/runner_host_file_converge.dag` | `plan_runner_host_file` | `net` | `Network` | callees observe_runner_host_file |
| `dag/gunbc/runner/runner_host_file_converge.dag` | `runner_host_manager_reload_step` | `net` | `Network` | callees reload_runner_host_manager |
| `dag/gunbc/runner/runner_host_file_converge.dag` | `run_runner_host_files` | `net` | `Network` | callees apply_runner_host_file, plan_runner_host_file, observe_runner_slot_population, runner_host_manager_reload_step, observe_runner_host_administrator_privilege |
| `dag/gunbc/runner/runner_host_file_converge.dag` | `runner_host_file_ci_wet` | `net` | `Network` | ops Filesystem.Write, clock_now_probed_at; callees prepare_fleet_ssh_agent_context, run_runner_host_files; other-demand Clock,Filesystem |
| `dag/gunbc/runner/runner_host_file_converge.dag` | `runner_host_file_observe_ci_wet` | `net` | `Network` | callees runner_host_file_ci_wet |
| `dag/gunbc/runner/runner_host_file_converge.dag` | `runner_host_file_converge_ci_wet` | `net` | `Network` | callees runner_host_file_ci_wet |
| `dag/gunbc/runner/runner_host_kernel_config.dag` | `observe_built_kernel_config` | `net` | `Network` | callees run_shell_command_observe |
| `dag/gunbc/runner/runner_host_kernel_config.dag` | `verify_built_kernel_config` | `net` | `Network` | callees observe_built_kernel_config |
| `dag/gunbc/runner/runner_host_kernel_config.dag` | `verify_runner_host_kernel_config` | `net` | `Network` | callees verify_built_kernel_config |
| `dag/gunbc/runner/runner_microvm_boot_probe.dag` | `runner_microvm_boot_probe_wet` | `net` | `Network` | ops Filesystem.Write; callees run_shell_commands, run_shell_command_observe, observe_firecracker_host_standing_wet; other-demand Filesystem |
| `dag/gunbc/runner/runner_microvm_host_ready.dag` | `observe_firecracker_host_standing_wet` | `net` | `Network` | callees run_shell_command_capture |
| `dag/gunbc/runner/runner_microvm_host_ready.dag` | `runner_microvm_host_observe_wet` | `net` | `Network` | ops Filesystem.Write; callees run_shell_commands, observe_firecracker_host_standing_wet; other-demand Filesystem |
| `dag/gunbc/runner/runner_microvm_host_ready.dag` | `runner_microvm_host_converge_wet` | `net` | `Network` | ops Filesystem.Write; callees run_shell_commands, observe_firecracker_host_standing_wet; other-demand Filesystem |
| `dag/gunbc/runner/runner_observed_version_check.dag` | `observe_captured_console` | `net` | `Network` | callees run_shell_command_observe |
| `dag/gunbc/runner/runner_observed_version_check.dag` | `check_captured_console` | `net` | `Network` | callees observe_captured_console |
| `dag/gunbc/runner/runner_observed_version_check.dag` | `verify_captured_console` | `net` | `Network` | callees check_captured_console |
| `dag/gunbc/runner/runner_observed_version_check.dag` | `verify_bound_attempt_runner_version` | `net` | `Network` | callees verify_captured_console |
| `dag/gunbc/runner/runner_observed_version_check.dag` | `verify_two_attempt_runner_version` | `net` | `Network` | callees verify_captured_console |
| `dag/gunbc/runner/runner_observed_version_check.dag` | `verify_jit_absent_capture_lacks_runner_version` | `net` | `Network` | callees verify_captured_console |
| `dag/gunbc/runner/runner_password_session_tool_converge.dag` | `runner_password_session_tool_converge_ci_wet` | `net` | `Network` | ops Filesystem.Write, clock_now_probed_at; callees prepare_fleet_ssh_agent_context; other-demand Clock,Filesystem |
| `dag/gunbc/runner/runner_slot_provision.dag` | `observe_runner_slot_members_wet` | `net` | `Network` | callees run_shell_command_capture |
| `dag/gunbc/spark/credential_workflow.dag` | `observe_spark_credential_consumer_evidence` | `net` | `Network` | callees fetch_secret_ref_credential |
| `dag/gunbc/spark/credential_workflow.dag` | `run_with_materialized_fleet_ssh_binding` | `net` | `Network` | callees fetch_secret_ref_credential, verify_fleet_ssh_credential |
| `dag/gunbc/spark/credential_workflow.dag` | `resolve_spark_administrator_credential` | `net` | `Network` | callees fetch_secret_ref_credential |
| `dag/gunbc/spark/fabric_rail_apply.dag` | `fabric_rail_apply_wet` | `net` | `Network` | ops Filesystem.Write, clock_now_probed_at; callees materialize_fleet_known_hosts_anchor; other-demand Clock,Filesystem |
| `dag/gunbc/spark/fabric_reach.dag` | `observe_executor_reach` | `net` | `Network` | callees run_shell_command_capture |
| `dag/gunbc/spark/glm_canary_converge.dag` | `glm_canary_cutover_wet` | `net` | `Network` | ops Filesystem.Write, clock_now_probed_at; callees materialize_fleet_known_hosts_anchor, observe_executor_reach; other-demand Clock,Filesystem |
| `dag/gunbc/spark/glm_canary_converge.dag` | `glm_canary_converge_wet` | `net` | `Network` | ops Filesystem.Write, clock_now_probed_at; callees materialize_fleet_known_hosts_anchor, observe_executor_reach; other-demand Clock,Filesystem |
| `dag/gunbc/spark/glm_group_b_relaunch_cli.dag` | `rank_startup_facet` | `net` | `Network` | callees observe_rank_vllm_revision |
| `dag/gunbc/spark/glm_group_b_relaunch_cli.dag` | `observe_rank_report` | `net` | `Network` | callees observe_rank_container, rank_startup_facet, capture_rank_log |
| `dag/gunbc/spark/glm_group_b_relaunch_cli.dag` | `glm_group_b_observe_wet` | `net` | `Network` | ops Filesystem.Write, clock_now_probed_at; callees with_group_b_access, observe_rank_report; other-demand Clock,Filesystem |
| `dag/gunbc/spark/glm_group_b_relaunch_cli.dag` | `glm_group_b_relaunch_wet` | `net` | `Network` | ops Filesystem.Write, clock_now_probed_at; callees relaunch_arm, with_group_b_access; other-demand Clock,Filesystem |
| `dag/gunbc/spark/glm_group_b_relaunch_cli.dag` | `with_group_b_access` | `net` | `Network` | callees materialize_fleet_known_hosts_anchor, observe_executor_reach |
| `dag/gunbc/spark/managed_access_apply.dag` | `spark_managed_access_apply_wet` | `net` | `Network` | ops Filesystem.Write, clock_now_probed_at; callees resolve_spark_administrator_credential, run_with_materialized_fleet_ssh_binding; other-demand Clock,Filesystem |
| `dag/gunbc/spark/managed_access_apply.dag` | `spark_managed_access_bootstrap_ci_wet` | `net` | `Network` | ops Filesystem.Write, clock_now_probed_at; callees prepare_fleet_ssh_agent_context; other-demand Clock,Filesystem |
| `dag/gunbc/spark/managed_grant_install.dag` | `spark_grant_install_execute` | `net` | `Network` | callees run_remote_step_sequence, spark_grant_read_back, run_privileged_step |
| `dag/gunbc/spark/managed_grant_install.dag` | `spark_grant_read_back` | `net` | `Network` | callees spark_grant_probe_outcome |
| `dag/gunbc/spark/managed_grant_install.dag` | `spark_grant_install_host` | `net` | `Network` | callees spark_grant_probe_outcome, spark_grant_install_execute |
| `dag/gunbc/spark/managed_grant_install.dag` | `spark_grant_install_ci_wet` | `net` | `Network` | ops Filesystem.Write, clock_now_probed_at; callees prepare_fleet_ssh_agent_context, spark_grant_install_host; other-demand Clock,Filesystem |
| `dag/gunbc/spark/pair_serving_apply.dag` | `spark_pair_serving_apply_wet` | `net` | `Network` | ops Filesystem.Write, clock_now_probed_at; callees observe_executor_reach, materialize_fleet_known_hosts_anchor; other-demand Clock,Filesystem |
| `dag/gunbc/spark/pinned_base_env_classification.dag` | `sparkrun_pinned_base_env_read` | `net` | `Network` | callees read_docker_hub_image_config_at_digest |
| `dag/gunbc/spark/pinned_base_env_classification.dag` | `sparkrun_pinned_base_env_classification_wet` | `net` | `Network` | callees sparkrun_pinned_base_env_read |
| `dag/gunbc/spark/remote_step_sequence.dag` | `remote_step_sequence_step` | `net` | `Network` | callees run_remote_operation |
| `dag/gunbc/spark/remote_step_sequence.dag` | `run_remote_step_sequence` | `net` | `Network` | callees remote_step_sequence_step |
| `dag/gunbc/spark/secret_access_ensure.dag` | `spark_secret_access_ensure` | `net` | `Network` | callees secret_access_ensure_for |
| `dag/gunbc/spark/secret_access_ensure.dag` | `spark_secret_access_converge` | `net` | `Network` | callees spark_secret_access_ensure |
| `dag/gunbc/spark/secret_access_ensure.dag` | `spark_secret_access_converge_with_supplied_token` | `net` | `Network` | callees spark_secret_access_ensure |
| `dag/gunbc/spark/serving_rank_observe.dag` | `observe_container_presence` | `net` | `Network` | callees rank_leg |
| `dag/gunbc/spark/serving_rank_observe.dag` | `observe_rank_container` | `net` | `Network` | callees observe_container_presence |
| `dag/gunbc/spark/serving_rank_observe.dag` | `capture_rank_log` | `net` | `Network` | callees rank_leg |
| `dag/gunbc/spark/serving_rank_observe.dag` | `observe_rank_transport` | `net` | `Network` | callees capture_rank_log |
| `dag/gunbc/spark/serving_rank_observe.dag` | `observe_rank_vllm_revision` | `net` | `Network` | callees rank_leg |
| `dag/gunbc/spark/serving_relaunch_transaction.dag` | `rollback_restart_incumbent` | `net` | `Network` | callees relaunch_leg |
| `dag/gunbc/spark/serving_relaunch_transaction.dag` | `capture_rank` | `net` | `Network` | callees observe_rank_container, preserved_name_is_free, observe_rank_transport |
| `dag/gunbc/spark/serving_relaunch_transaction.dag` | `preserved_name_is_free` | `net` | `Network` | callees observe_container_presence |
| `dag/gunbc/spark/serving_relaunch_transaction.dag` | `capture_arm` | `net` | `Network` | callees capture_rank |
| `dag/gunbc/spark/serving_relaunch_transaction.dag` | `advance_rank` | `net` | `Network` | callees relaunch_leg |
| `dag/gunbc/spark/serving_relaunch_transaction.dag` | `stop_and_preserve_arm` | `net` | `Network` | callees advance_rank |
| `dag/gunbc/spark/serving_relaunch_transaction.dag` | `create_and_start_arm` | `net` | `Network` | callees advance_rank |
| `dag/gunbc/spark/serving_relaunch_transaction.dag` | `rollback_restore_names` | `net` | `Network` | callees rollback_restore_one |
| `dag/gunbc/spark/serving_relaunch_transaction.dag` | `rollback_restore_one` | `net` | `Network` | callees rollback_rename_back, rollback_after_observing |
| `dag/gunbc/spark/serving_relaunch_transaction.dag` | `rollback_after_observing` | `net` | `Network` | callees rollback_rename_back, observe_container_presence |
| `dag/gunbc/spark/serving_relaunch_transaction.dag` | `rollback_rename_back` | `net` | `Network` | callees relaunch_leg |
| `dag/gunbc/spark/serving_relaunch_transaction.dag` | `rollback_start_restored` | `net` | `Network` | callees rollback_start_one |
| `dag/gunbc/spark/serving_relaunch_transaction.dag` | `rollback_start_one` | `net` | `Network` | callees rollback_restart_incumbent |
| `dag/gunbc/spark/serving_relaunch_transaction.dag` | `rollback_arm` | `net` | `Network` | callees rollback_start_restored, rollback_restore_names |
| `dag/gunbc/spark/serving_relaunch_transaction.dag` | `await_rank_announcement` | `net` | `Network` | callees observe_rank_container, observe_rank_transport, await_after_interval |
| `dag/gunbc/spark/serving_relaunch_transaction.dag` | `await_after_interval` | `net` | `Network` | callees await_rank_announcement |
| `dag/gunbc/spark/serving_relaunch_transaction.dag` | `verify_arm` | `net` | `Network` | callees await_rank_announcement |
| `dag/gunbc/spark/serving_relaunch_transaction.dag` | `relaunch_arm` | `net` | `Network` | callees capture_arm, rollback_arm, stop_and_preserve_arm, create_and_start_arm, verify_arm |
| `dag/gunbc/spark/v41_runtime_image_probe.dag` | `v41_published_image_probe_ci_wet` | `net` | `Network` | ops Filesystem.Write, clock_now_probed_at; callees materialize_fleet_known_hosts_anchor, observe_executor_reach, v41_probe_published_image; other-demand Clock,Filesystem |
| `dag/gunbc/spark/v41_source_patch_converge.dag` | `v41_observe_member` | `net` | `Network` | callees v41_check_applicable, v41_place_declared_patch |
| `dag/gunbc/spark/v41_source_patch_converge.dag` | `v41_observe_members` | `net` | `Network` | callees v41_observe_member |
| `dag/gunbc/spark/v41_source_patch_converge.dag` | `v41_observe_tree` | `net` | `Network` | callees v41_observe_members |
| `dag/gunbc/spark/v41_source_patch_converge.dag` | `converge_vllm_source_with_patches` | `net` | `Network` | callees v41_observe_tree, converge_vllm_source_to_revision, v41_apply_plan |
| `dag/gunbc/spark/v41_source_patch_converge.dag` | `converge_vllm_runtime_image_with_patches` | `net` | `Network` | callees converge_vllm_image_keyed, converge_vllm_source_with_patches |
| `dag/gunbc/spark/v41_source_patch_converge_wet.dag` | `v41_patched_source_acquire_wet` | `net` | `Network` | ops Filesystem.Write, clock_now_probed_at; callees converge_vllm_source_with_patches, probe_agent_credential_verification, materialize_fleet_known_hosts_anchor; other-demand Clock,Filesystem |
| `dag/gunbc/spark/vllm_runtime_image_build.dag` | `vllm_source_after_revision` | `net` | `Network` | callees vllm_source_tree_clean |
| `dag/gunbc/spark/vllm_runtime_image_build.dag` | `converge_vllm_source_to_revision` | `net` | `Network` | callees vllm_source_after_revision |
| `dag/gunbc/spark/vllm_runtime_image_build.dag` | `converge_vllm_source` | `net` | `Network` | callees converge_vllm_source_to_revision |
| `dag/gunbc/spark/vllm_runtime_image_build.dag` | `converge_vllm_image` | `net` | `Network` | callees converge_vllm_image_keyed |
| `dag/gunbc/spark/vllm_runtime_image_build.dag` | `converge_vllm_image_keyed` | `net` | `Network` | callees vllm_docker_leg |
| `dag/gunbc/spark/vllm_runtime_image_build.dag` | `converge_vllm_runtime_image` | `net` | `Network` | callees converge_vllm_image, converge_vllm_source |
| `dag/gunbc/spark/vllm_runtime_image_build.dag` | `vllm_runtime_image_build_ci_wet` | `net` | `Network` | ops Filesystem.Write, clock_now_probed_at; callees observe_executor_reach, materialize_fleet_known_hosts_anchor, converge_vllm_runtime_image, probe_agent_credential_verification; other-demand Clock,Filesystem |
| `dag/gunbc/spark/vllm_runtime_image_build.dag` | `vllm_source_acquire_wet` | `net` | `Network` | ops Filesystem.Write, clock_now_probed_at; callees converge_vllm_source, probe_agent_credential_verification, materialize_fleet_known_hosts_anchor; other-demand Clock,Filesystem |
| `dag/gunbc/spark/vllm_runtime_image_build.dag` | `spark_build_host_reachability_wet` | `net` | `Network` | ops Filesystem.Write, clock_now_probed_at; callees materialize_fleet_known_hosts_anchor, probe_spark_host_reachability, probe_agent_credential_verification; other-demand Clock,Filesystem |
| `dag/gunbc/srv3/srv3_bmc_credential_resolve.dag` | `materialize_bmc_login_password` | `net` | `Network` | callees fetch_secret_ref_credential |
| `dag/gunbc/srv3/srv3_bmc_credential_resolve.dag` | `observe_and_resolve_srv3_bmc_credential` | `net` | `Network` | callees srv3_probe_credential_phase |
| `dag/gunbc/srv3/srv3_boot_once_cd.dag` | `srv3_boot_once_cd` | `clock` | `std.resources.Clock` | callees srv3_boot_once_cd_gated |
| `dag/gunbc/srv3/srv3_boot_once_cd.dag` | `srv3_boot_once_cd` | `net` | `std.resources.Network` | callees srv3_boot_once_cd_gated |
| `dag/gunbc/srv3/srv3_boot_once_cd.dag` | `srv3_boot_once_cd_operator_approved` | `clock` | `std.resources.Clock` | callees srv3_boot_once_cd_gated |
| `dag/gunbc/srv3/srv3_boot_once_cd.dag` | `srv3_boot_once_cd_operator_approved` | `net` | `std.resources.Network` | callees srv3_boot_once_cd_gated |
| `dag/gunbc/srv3/srv3_boot_once_cd.dag` | `srv3_boot_once_cd_gated` | `net` | `std.resources.Network` | ops clock_now_probed_at; callees srv3_boot_once_cd_resolved, observe_and_resolve_srv3_bmc_credential; other-demand Clock |
| `dag/gunbc/srv3/srv3_os_install_actuate.dag` | `srv3_bmcweb_session_login` | `net` | `std.resources.Network` | callees srv3_bmcweb_session_login_resolved, observe_and_resolve_srv3_bmc_credential |
| `dag/gunbc/srv3/srv3_os_install_actuate.dag` | `srv3_bmcweb_session_login_resolved` | `net` | `std.resources.Network` | ops Filesystem.Write; callees materialize_bmc_login_password; other-demand Filesystem |
| `dag/gunbc/srv3/srv3_os_install_actuator_toolchain_ensure.dag` | `srv3_os_install_actuator_toolchain_ensure` | `net` | `Network` | callees host_toolchain_ensure |
| `dag/gunbc/tailscale_acl_phase2_credential.dag` | `tailscale_acl_operator_ephemeral_credential` | `net` | `std.resources.Network` | callees tailscale_acl_credential_with_operator_token |
| `dag/gunbc/tailscale_acl_phase2_credential.dag` | `tailscale_acl_credential_with_operator_token` | `net` | `std.resources.Network` | callees gcp_secret_credential |
| `dag/gunbc/tools/bmc_onboard.dag` | `srv3_probe_credential_phase` | `net` | `Network` | callees bmc_probe_credential_phase |
| `dag/gunbc/tools/bmc_onboard.dag` | `srv3_converge_credential` | `net` | `Network` | callees bmc_converge_credential |
| `dag/gunbc/tools/bmc_onboard.dag` | `srv4_converge_credential` | `net` | `Network` | callees bmc_converge_credential |
| `dag/test/ctl/pos_emit.dag` | `forwards_to_a_service_call` | `net` | `std.resources.Network` | callees emits_a_service_call |

## UnusedRequirement

(empty in the live parsed population)

## Unresolved

| File | Declaration | Alias | Authored type | Evidence |
| --- | --- | --- | --- | --- |
| `dag/gunbc/container/registry_image_config.dag` | `read_docker_hub_image_config_at_digest` | `net` | `Network` | ops Filesystem.WriteOwnerOnly; local-ops shell.Mktemp.Dir, shell.Remove.RecursiveForce, shell.Remove.RecursiveForce, shell.Remove.RecursiveForce; other-demand Filesystem; calls check_sha256_hex, container_image_config_env_from_json, container_image_config_from_env, digest_hex_from_sha256_wire, docker_hub_bearer_token, docker_hub_blob_url, docker_hub_manifest_url, manifest_config_digest_wire |
| `dag/gunbc/fleet/fleet_converge_plan_cli.dag` | `observe_generation_store_wet` | `net` | `Network` | ops Filesystem.Read; other-demand Filesystem; calls fleet_converge_generation_admission, fleet_converge_generation_observation, fleet_converge_generation_store_path |
| `dag/gunbc/fleet/fleet_converge_plan_cli.dag` | `observe_cap_members_wet` | `net` | `Network` | ops Filesystem.List, Filesystem.Read; other-demand Filesystem; calls FilesystemFileAbsent, cap_members_observation, filesystem_file_observation, filesystem_listing_observation, filesystem_read_outcome |
| `dag/gunbc/fleet/fleet_converge_plan_cli.dag` | `observe_runner_unit_standing_wet` | `net` | `Network` | calls read_unit_property |
| `dag/gunbc/fleet/fleet_converge_plan_cli.dag` | `observe_operator_local_run_binding` | `net` | `Network` | ops clock_now_probed_at; local-ops git.Inspect.HeadCommit; other-demand Clock; calls clock_now_probed_at, release_revision_text_valid |
| `dag/gunbc/fleet/fleet_host_key_enrollment.dag` | `fleet_host_key_scan_wet` | `net` | `Network` | ops Filesystem.Write; other-demand Filesystem; calls scan_host_key_lines |
| `dag/gunbc/fleet/fleet_known_hosts_anchor.dag` | `materialize_fleet_known_hosts_anchor` | `net` | `Network` | ops Filesystem.Read, Filesystem.Write, Filesystem.Read; other-demand Filesystem; calls compare_content_hash, content_hash_of_value, fleet_known_hosts_anchor_path, fleet_known_hosts_fragment, fleet_known_hosts_registry_identity, fleet_known_hosts_rows, host_key_provenance_label, known_hosts_anchor_seal |
| `dag/gunbc/fleet/fleet_multi_principal_probe.dag` | `spark_managed_access_standing_wet` | `net` | `Network` | calls fleet_ssh_attempt_raw_from_env, spark_managed_access_standing_with_attempt |
| `dag/gunbc/fleet/fleet_multi_principal_probe.dag` | `spark_managed_access_standing_with_verified_credential` | `net` | `Network` | ops Filesystem.Write, clock_now_probed_at; other-demand Clock,Filesystem; calls agent_backed_principal_observation, clock_now_probed_at, derive_spark_managed_access_standing, probed_at_word, serialize_content_hash, spark_authorization_lines, spark_standing_line, spark_standing_unobserved_host |
| `dag/gunbc/fleet/fleet_ssh_credential_verify.dag` | `materialize_agent_public_selector` | `net` | `Network` | ops Filesystem.Read, Filesystem.Write, Filesystem.Read; other-demand Filesystem; calls agent_public_selector_content, agent_public_selector_path, agent_selector_file_seal, compare_content_hash, content_hash_of_value |
| `dag/gunbc/host/host_codex_runtime_provision.dag` | `materialize_codex_runtime_on_local_host` | `net` | `Network` | calls codex_runtime_materialization_refusal_reason, committed_admitted_codex_runtime_source, materialize_codex_runtime_bundle, provider_runtime_placement_for_host |
| `dag/gunbc/host/host_reset_return_run.dag` | `observer_self_corroboration` | `net` | `std.resources.Network` | local-ops os.Hostname.ReadShort; calls canonical_hostname_for_fleet_slot, observer_self_identity |
| `dag/gunbc/host/host_reset_return_run.dag` | `host_reset_return_preflight_exit` | `net` | `std.resources.Network` | ops Filesystem.Write; other-demand Filesystem; calls reset_preflight_refusal_wire |
| `dag/gunbc/host/host_reset_return_run.dag` | `host_reset_return_dispatch_admission_wet` | `net` | `std.resources.Network` | local-ops shell.Env.Get, shell.Env.Get; calls admit_reset_dispatch |
| `dag/gunbc/host/host_reset_return_run.dag` | `host_reset_return_converge_route_read` | `net` | `std.resources.Network` | calls actions_variable_read |
| `dag/gunbc/host/host_reset_return_run.dag` | `host_reset_return_exit` | `net` | `std.resources.Network` | ops Filesystem.Write; other-demand Filesystem; calls host_reset_return_receipt_line, reset_return_refusal_cause_wire |
| `dag/gunbc/host/host_runner_memory_provision.dag` | `provision_runner_memory_caps` | `net` | `Network` | calls fleet_compute_host_for_identity, host_effect_apply_gated, host_toolchain_direct_access, runner_memory_converge_effect |
| `dag/gunbc/host/host_toolchain_ensure.dag` | `host_toolchain_ensure` | `net` | `Network` | calls host_effect_apply_gated, host_toolchain_ensure_effect, host_toolchain_ensure_reach |
| `dag/gunbc/live_deploy/deployed_tree_report.dag` | `deployed_tree_report_for_context` | `net` | `Network` | calls converge_target_revision, deployed_tree_locus, deployed_tree_report_emit, deployed_tree_standing, fleet_locus_ssh_target, observe_deployed_tree, observe_fleet_desired_revision |
| `dag/gunbc/machine_intake/boot_image_fetch.dag` | `confirm_published_boot_image` | `net` | `std.resources.Network` | calls boot_image_digest_agrees, boot_image_observed_hex, sha256sum_file_digest_via_shell |
| `dag/gunbc/roadmap/roadmap_launch_deployment_cli.dag` | `window_snapshot` | `net` | `Network` | calls active_instance_text_routed, deployed_tree_hexes, deployed_tree_locus, fleet_locus_ssh_target, observe_deployed_tree, read_desired_hex, read_remote_main_hex, remote_ref_text |
| `dag/gunbc/roadmap/roadmap_launch_deployment_cli.dag` | `unit_standing_fact` | `net` | `Network` | calls deployment_belt_service_unit_name, deployment_belt_timer_unit_name, deployment_spec_srv1, live_deploy_belt_service_unit_file, live_deploy_belt_timer_unit_file, live_deploy_serve_unit_file, observe_belt_oneshot_standing, observe_belt_timer_standing |
| `dag/gunbc/runner/runner_host_file_converge.dag` | `observe_runner_host_administrator_privilege` | `net` | `Network` | calls elevated_argv, typed_argv_exec_over_fleet_ssh |
| `dag/gunbc/runner/runner_host_file_converge.dag` | `observe_runner_host_paths` | `net` | `Network` | calls classify_root_only_paths, runner_host_root_only_find_argv, typed_argv_exec_over_fleet_ssh |
| `dag/gunbc/runner/runner_host_file_converge.dag` | `observe_runner_host_file_content` | `net` | `Network` | calls classify_host_file_presence, classify_typed_remote_file_comparison, runner_host_file_standing_of_report, runner_host_file_write_admission, typed_argv_exec_over_fleet_ssh, typed_remote_file_compare_argv, typed_remote_file_step |
| `dag/gunbc/runner/runner_host_file_converge.dag` | `observe_runner_slot_population` | `net` | `Network` | calls classify_runner_slot_population, runner_slot_population_units, runner_slot_units_from_list_units, systemctl_list_all_service_units_argv, systemctl_show_properties_argv, typed_argv_exec_over_fleet_ssh |
| `dag/gunbc/runner/runner_host_file_converge.dag` | `reload_runner_host_manager` | `net` | `Network` | calls elevated_argv, systemctl_daemon_reload_argv, typed_argv_exec_over_fleet_ssh |
| `dag/gunbc/runner/runner_slot_population_census.dag` | `observe_slot_population_over_ssh` | `net` | `Network` | calls slot_census_from_command_outcome, systemctl_list_all_service_units_argv, typed_argv_exec_over_fleet_ssh |
| `dag/gunbc/runner/runner_slot_retirement.dag` | `observe_transition_retirement_evidence` | `net` | `Network` | calls observe_retirement_standings, transition_required_retiring |
| `dag/gunbc/spark/glm_canary_converge.dag` | `glm_canary_cutover` | `net` | `Network` | calls glm_canary_replace_container, glm_canary_run_argv |
| `dag/gunbc/spark/grant_privileged_operation.dag` | `run_privileged_step` | `net` | `Network` | calls admitted_root_argv_words, exec_as_fleet_principal_with_stdin, fleet_probe_endpoint_for, portable_remote_words, read_bootstrap_credential, sudo_rejected_the_credential |
| `dag/gunbc/spark/managed_grant_install.dag` | `spark_grant_probe_outcome` | `net` | `Network` | calls run_typed_argv_transport, spark_grant_install_transport |
| `dag/gunbc/spark/remote_step_sequence.dag` | `run_remote_operation` | `net` | `Network` | calls SshShell, converge_typed_remote_file, run_typed_argv_transport |
| `dag/gunbc/spark/serving_rank_observe.dag` | `rank_leg` | `net` | `Network` | calls argv_words, rank_observation_refusal, spark_remote_run |
| `dag/gunbc/spark/serving_relaunch_transaction.dag` | `relaunch_leg` | `net` | `Network` | calls argv_words, relaunch_step_wire, spark_remote_run |
| `dag/gunbc/spark/v41_runtime_image_probe.dag` | `v41_probe_published_image` | `net` | `Network` | calls docker_image_inspect_id_from_stdout, docker_image_inspect_names_no_such_image, image, spark_remote_run, v41_capability_reading, v41_privileged_outcome, v41_privileged_stdout, v41_probe_config_id_argv |
| `dag/gunbc/spark/v41_source_patch_converge.dag` | `v41_place_declared_patch` | `net` | `Network` | ops Filesystem.Read; other-demand Filesystem; calls converge_typed_remote_file, filesystem_read_outcome, typed_remote_file_write_seal, v41_remote_declared_patch_path, vllm_build_root |
| `dag/gunbc/spark/v41_source_patch_converge.dag` | `v41_check_applicable` | `net` | `Network` | calls git_apply_check_command, git_apply_reverse_check_command, v41_remote_declared_patch_path, vllm_image_leg_unchecked, vllm_source_dir |
| `dag/gunbc/spark/v41_source_patch_converge.dag` | `v41_apply_plan` | `net` | `Network` | calls git_apply_command, v41_remote_declared_patch_path, vllm_image_leg, vllm_source_dir |
| `dag/gunbc/spark/vllm_runtime_image_build.dag` | `vllm_docker_leg` | `net` | `Network` | calls argv_words, spark_remote_run |
| `dag/gunbc/spark/vllm_runtime_image_build.dag` | `vllm_source_tree_clean` | `net` | `Network` | calls git_status_porcelain_command, vllm_image_leg, vllm_source_dir, vllm_source_is_clean |
| `dag/gunbc/spark/vllm_runtime_image_build.dag` | `probe_spark_host_reachability` | `net` | `Network` | calls classify_reported_hostname, hostname_short_read_command, spark_reach_endpoint, vllm_image_leg |
| `dag/gunbc/srv3/srv3_install_media_fetch.dag` | `srv3_install_media_fetch` | `net` | `Network` | calls srv3_apply_to_process_exit, srv3_install_media_fetch_apply |
| `dag/gunbc/srv3/srv3_os_install_actuate.dag` | `srv3_nbd_proxy_serve` | `net` | `std.resources.Network` | calls srv3_apply_to_process_exit, srv3_nbd_proxy_serve_apply |
| `dag/gunbc/srv3/srv3_seeded_install_media.dag` | `srv3_seeded_install_media_toolchain_ensure` | `net` | `std.resources.Network` | calls srv3_apply_to_process_exit, srv3_seeded_install_media_toolchain_ensure_apply |
| `dag/gunbc/srv3/srv3_seeded_install_media.dag` | `srv3_seeded_install_media_remaster` | `net` | `std.resources.Network` | calls srv3_apply_to_process_exit, srv3_seeded_install_media_remaster_apply, srv3_seeded_install_media_toolchain_ensure_apply |

## NamedDependencyBinder

(empty in the live parsed population)

## OpaqueContract

(empty in the live parsed population)

## PolicyEnvelope

(empty in the live parsed population)

## Fixture-string population (not live declarations)

Witness modules embed programs as `data *_source: String`. Those `uses` rows are specimens, not production headers.

| Host | Embedded decl data | Alias | Type | Class in the specimen |
| --- | --- | --- | --- | --- |
| `dag/test/claim/data_class_refusal_probe_witness_test.dag` | `durable_write_literal_control_source` | `net` | `Network` | `UnusedRequirement` |
| `dag/test/claim/effectful_item_kind_collapse_witness_test.dag` | `efr_control_source` | `net` | `EfrNet` | `NamedDependencyBinder` |
| `dag/test/claim/effectful_item_kind_collapse_witness_test.dag` | `efr_non_tail_source` | `net` | `EfrNet` | `NamedDependencyBinder` |
| `dag/test/claim/effectful_item_kind_collapse_witness_test.dag` | `efr_tail_source` | `net` | `EfrNet` | `NamedDependencyBinder` |
| `dag/test/claim/effectful_item_kind_collapse_witness_test.dag` | `zpe_reads_resource_source` | `net` | `ZpeNet` | `NamedDependencyBinder` |
| `dag/test/claim/entry_authority_witness_test.dag` | `ea_one_entry_source` | `net` | `EaNet` | `NamedDependencyBinder` |
| `dag/test/claim/entry_authority_witness_test.dag` | `ea_uses_library_source` | `net` | `EaNet` | `NamedDependencyBinder` |
| `dag/test/claim/generic_effectful_declaration_wall_witness_test.dag` | `gew_generic_effectful_source` | `net` | `GewNet` | `UnusedRequirement` |
| `dag/test/claim/generic_effectful_declaration_wall_witness_test.dag` | `gew_plain_effectful_source` | `net` | `GewNet2` | `NamedDependencyBinder` |

## Inverse (derived demand without an authored `uses` on that declaration)

Not a `uses` occurrence, so not a USES-0 class member. Named because D13 derives demand from operations: `gunbc.clock_read` `clock_now_probed_at` calls `Clock.Now()` with no header contract. Callers then restate `uses clock` transitively. That is the derive-once-carry gap: the Clock demand is produced at `Clock.Now` and should be carried, not re-authored up the chain.

HOLD: no `uses` row added or deleted by this census.

