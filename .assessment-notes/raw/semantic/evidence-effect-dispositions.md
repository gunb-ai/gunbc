# Evidence-effect dispositions

Source HEAD `7daaddd1f0ce99734e1ddb57138313701455c758`; read-only audit 2026-10-23. No repository changes, builds or live probes. The shared inventory had already been refined from the requested 91-module evidence group when this audit began: the current relevant union was **7 migrate_evidence + 77 local_output_adapter = 84 modules**. I froze those rows in `/tmp/evidence-effect-review-input.json`. The JSON companion provides module-grain overrides, exact effect symbols/source sites, roots, path declarations and preservation requirements. This is not a claim that the original 91 rows or the whole repository have been closed.

## Corrections requiring inventory changes

- Do not exclude the entire R2 token-mint module: mint/orphan receipts carry token IDs and custody needed for recovery. Their local uploader exports can remain adapters.
- Do not exclude OIDC JWKS as diagnostic output: one effect is a persistent, expiring cache; other effects are OpenSSL scratch inputs. Preserve origin/freshness semantics and classify them separately.
- The roadmap snapshot census rereads its previous receipt across runs. It is persistent diagnostic state, though not authority to delete snapshots; M3 owns its disposition with the Git carrier cut.
- The roadmap event carrier is a conditional Git state authority. Assign its full cut to M3; local private snapshots are its materialization layer.
- `roadmap_verify::persist_verification_receipt` has no tracked caller under exact-name search. Retire it explicitly as legacy; do not invent an active root or use it to stand for the separate current verification producers.
- Provider-wire retention is a real consumed private retention API, but the discovered outer Codex entrances are called by wet witnesses. A declared srv1 path is not proof that production dispatch uses it.

## Durable, cached and retired subjects

### `gunbc.auth.gcp_iam_bootstrap` — M4 / migrate

Create-only bootstrap start/election/closure/retirement records gate privileged retries and recovery; not diagnostics.

- Operations: `iam_authority_pin`, `iam_bootstrap_apply`, `iam_bootstrap_authority_policy_change`.
- Entrances/readers: `gunbc.auth.gcp_iam_bootstrap::iam_bootstrap_apply`; `gunbc.auth.gcp_iam_bootstrap::iam_bootstrap_authority_cleanup`; `gunbc.auth.gcp_iam_bootstrap::iam_bootstrap_probe_cleanup`.
- Binding: Bootstrap authority identity/deadline and receipt identity; iam_authority_record and iam_bootstrap_cell_authority_reconcile reread pins.
- Setup: Operator checkout target/; no parent creation in these writers. Source explicitly preserves these pre-journal authority records on operator workstation because the ceremony creates the journal bucket.
- Declared locations: `target/gcp-iam-bootstrap-<receipt>-started.txt`; `target/gcp-iam-bootstrap-<receipt>-<stage>.txt`; `target/<derived authority elected/closed/retired name>`.
- Preserve: Create-only election, exact bytes on retry, independent IAM readback, cleanup owed after unknown completion. A fabric realization must be available before journal/bootstrap prerequisites; do not make it depend on the credential/bucket it creates.
- Live population: unobserved.

### `gunbc.auth.oidc_id_token_verification` — M4 / migrate

jwks_fetch_and_cache writes a cross-request expiring JWKS cache; rs256_verify_over_key creates local OpenSSL transport inputs. Neither is a plain report.

- Operations: `jwks_fetch_and_cache`, `rs256_verify_over_key`.
- Entrances/readers: `gunbc.roadmap.roadmap_auth_routes::auth_callback_verify`; `gunbc.roadmap.roadmap_auth_routes::auth_callback_verdict`; `gunbc.auth.oidc_id_token_verification::oidc_id_token_signature_verify`.
- Binding: jwks_cache_layout_for_instance derives from jwks_cache_directory(instance); callback retains the served HostDashboardInstance.
- Setup: jwks_cache_directory is enrolled in dashboard_instance_directory_demands, consumed by both deployment routes.
- Declared locations: `<instance private auth-jwks directory>/cache.json`; `<same directory>/gunbc-jwks-<verify_id>-key.pem`; `<same directory>/gunbc-jwks-<verify_id>-message.bin`; `<same directory>/gunbc-jwks-<verify_id>-signature.b64url`; `<same directory>/gunbc-jwks-<verify_id>-signature.bin`.
- Preserve: Cache is derived data, not issuer authority: 3600-second freshness; stale/unreadable/miss fetches origin; failed fetch does not accept stale keys; failed cache write does not invalidate a successful origin read. M4 may move logical cache storage; preserve local OpenSSL materialization as transport, not retained evidence.
- Live population: unobserved.

### `gunbc.cloudflare.r2_token_mint_run` — M4 / migrate

Mint and orphan receipts retain token identity/custody needed for later reconciliation. Token-list and revoke report exports are separate.

- Operations: `ensure_r2_mint_receipt_dir`, `list_account_tokens_for_reconcile`, `mint_origin_token_and_store`, `refuse_with_orphan_receipt`, `revoke_unheld_object_read_token`.
- Entrances/readers: `gunbc.cloudflare.r2_token_mint_run::run`; `run_object_write`; `run_boot_object_read`; `run_boot_object_write`; `run_cache_object_read`; `run_cache_object_write`; `run_workspace_object_read`; `run_workspace_object_write`.
- Binding: mint_origin_token_and_store(profile,purpose); r2_mint_receipt_scope_for_this_run binds GITHUB_RUN_ID and GITHUB_RUN_ATTEMPT; unscoped operator run is explicit. run_bucket_admin is a separate bootstrap ceremony, not this mint fold.
- Setup: ensure_r2_mint_receipt_dir precedes non-idempotent mint. fleet_converge_workflow derives artifact-upload glob from r2_mint_receipt_glob_in using the same scope.
- Declared locations: `/tmp/r2-mint-receipts/<run>-<attempt>/r2-<request.name>-mint-receipt.txt`; `/tmp/r2-mint-receipts/r2-<request.name>-mint-receipt.txt (operator unscoped)`; `/tmp/r2-account-token-list.txt (report only)`; `/tmp/r2-object-read-revoke-receipt.txt (report only)`.
- Preserve: Record token ID and exact stored version; retain orphan ID/custody on failed secret storage; do not repeat unknown AccountTokens.Create. Storage bootstrap must not depend on the token being minted. Preserve scoped uploader as local export if needed.
- Live population: unobserved.

### `gunbc.fleet.fleet_converge_cli` — M4 / migrate

Published current per-host convergence receipt is read by fleet_receipt_collector, not only printed.

- Operations: `converge_cli_atomic_receipt_write`.
- Entrances/readers: `gunbc.fleet.fleet_converge_cli::converge_cli_output`; `gunbc.fleet.fleet_converge_cli::converge_cli_output_with_verify`.
- Binding: converge_cli_publish -> converge_cli_atomic_receipt_write derives path from resolved host. collector::collect_host_receipt reads fleet_converge_receipt_path over typed SSH.
- Setup: fleet_converge_receipt_dir is /opt/gunbc/receipts; live_deploy.emit belt-member install owns instance_receipts_dir. Source documents existing host-vs-instance mismatch, not a generalized setup guarantee.
- Declared locations: `/opt/gunbc/receipts/<host>-converge-receipt.txt`; `/opt/gunbc/receipts/<host>-converge-receipt.staging`.
- Preserve: Atomic visible replacement, actual host/revision, service-principal owner readback. Do not confuse host scope with the blue/green per-instance receipt namespace.
- Live population: unobserved.

### `gunbc.fleet.fleet_converge_plan_cli` — M4 / migrate

Reviewed plan, apply script and provenance sidecars are consumed by a later privileged apply, not ordinary diagnostics.

- Operations: `fleet_converge_apply_commissioning_bound_wet`, `fleet_converge_apply_ordinary_bound_wet`, `fleet_converge_operator_cli_entry`, `fleet_converge_plan_scoped_bound_wet`, `write_fleet_converge_plan_artifact_wet`.
- Entrances/readers: `gunbc.fleet.fleet_converge_plan_cli::fleet_converge_plan_wet`; `fleet_converge_launch_environment_plan_wet`; `fleet_converge_workspace_commissioning_plan_wet`; `fleet_converge_workspace_allocation_plan_wet`; `fleet_converge_apply_wet`; `fleet_converge_operator_cli_entry`.
- Binding: ActionsRunBinding plus host/scope/revision, prior/planned generation, member-set and bundle hash; apply reads exactly the reviewed artifact and receipt.
- Setup: write_fleet_converge_plan_artifact_wet / fleet_converge_plan_scoped_bound_wet mkdir plan artifact directory; apply creates apply receipt dir. Workflow uploads/downloads the declared bundle.
- Declared locations: `/tmp/fleet-converge-plan/plan.txt`; `/tmp/fleet-converge-plan/apply.sh`; `/tmp/fleet-converge-plan/<baseline,hash,host,generation,member-set,scope sidecars>`; `/tmp/fleet-converge-apply/apply_receipt.json`; `<operator receipt_path> (export only)`.
- Preserve: Bundle coherence and exact run/revision/host/subject/generation; retain deliberate local apply-script materialization. Operator receipt_path is an export and does not become another authority.
- Live population: unobserved.

### `gunbc.fleet.fleet_converge_receipt` — M4 / migrate

Typed plan/apply receipts are read by the dashboard transaction and final launch-deployment receipt join.

- Operations: `write_fleet_converge_apply_receipt`, `write_fleet_converge_plan_receipt`.
- Entrances/readers: `gunbc.fleet.fleet_converge_plan_cli::fleet_converge_plan_scoped_bound_wet`; `fleet_converge_apply_ordinary_bound_wet`; `fleet_converge_apply_commissioning_bound_wet`.
- Binding: FleetConvergePlanReceipt/FleetConvergeApplyReceipt; read_fleet_converge_plan_receipt_at and read_fleet_converge_apply_receipt_at; consumers live_deploy.apply::live_deploy_transaction_admitted_wet_with_access and roadmap_launch_deployment_cli::rlm_launch_deployment_receipt_bound.
- Setup: Plan/apply CLI creates declared artifact directories; receiving job downloads artifacts. No live population measured.
- Declared locations: `/tmp/fleet-converge-plan/plan_receipt.json`; `/tmp/fleet-converge-apply/apply_receipt.json`.
- Preserve: Exact identity of predecessor receipts and refusal on unreadable/mismatched provenance; migrate with the bundle writers/readers, preserving workflow export adapter.
- Live population: unobserved.

### `gunbc.heal_candidate` — M4 / migrate

Candidate manifest and blobs form the sealed input to a later credentialed publisher.

- Operations: `prepare_candidate_directory`, `write_candidate_blob`, `write_candidate_manifest`.
- Entrances/readers: `gunbc.heal_candidate_stage0::heal_candidate_stage0_produce_wet`; `gunbc.heal_candidate::heal_candidate_produce_over`; `gunbc.heal_candidate::heal_candidate_produce_declaring`.
- Binding: HealRepairCandidate repo/PR/run/expected-head identity; candidate_blob_file_name from proposed Git blob identity. heal_publication::heal_publication_publish_wet/publish_for reads and independently admits bundle.
- Setup: prepare_candidate_directory deletes/remakes target/heal-repair-candidate; heal_workflow::heal_candidate_upload_step exports it; publisher workflow downloads.
- Declared locations: `target/heal-repair-candidate/<manifest>`; `target/heal-repair-candidate/<derived blob name>`.
- Preserve: Exact candidate bytes/blob identities/readback and independent publisher admission. Keep artifact transport compatibility; no privileged publication from an unbound output directory.
- Live population: unobserved.

### `gunbc.live_deploy.apply` — M4 / migrate

Deployment receipt is later read in final release/launch evidence join.

- Operations: `live_deploy_write_apply_receipt`.
- Entrances/readers: `gunbc.live_deploy.apply::live_deploy_apply_prod_role_transaction_wet`; `gunbc.live_deploy.apply::live_deploy_transaction_admitted_wet_with_access`.
- Binding: Selected DesiredDeployment and ActionsRunBinding; live_deploy_write_apply_receipt(spec,receipt,exit). Reader roadmap_launch_deployment_cli::rlm_launch_deployment_receipt_bound uses selected instance.
- Setup: live_deploy_write_apply_receipt creates dir; CI uploads path derived by apply_receipt authority; later join reads downloaded artifact.
- Declared locations: `target/live-deploy-<instance_id>-receipt/live_deploy_receipt.json`.
- Preserve: Apply outcome, exact candidate/revision/provenance and independent post-apply healthz observation; preserve local artifact export rather than treating it as proof that runtime converged.
- Live population: unobserved.

### `gunbc.provider_wire_evidence` — M4 / migrate

Content-addressed private provider-wire byte retention returns PrivateEvidenceRef and digest/size, not a throwaway report.

- Operations: `ensure_private_evidence_root`, `mktemp_file_in_dir`, `publish_evidence_object_exclusive`, `retain_private_evidence_bytes_detailed`.
- Entrances/readers: `gunbc.codex_app_server_press::observe_codex_account_preflight`; `gunbc.codex_supervised_turn_run::run_codex_supervised_turn`; `test.claim.codex_app_server_press_wet_witness`; `test.claim.codex_supervised_turn_wet_witness`.
- Binding: Caller evidence_root -> retain_provider_wire_capture / retain_private_evidence_bytes_detailed; PrivateEvidenceRef includes physical root plus SHA-512 address. Actual outer callers found for those Codex entrances are wet witnesses; do not claim a production dispatch root from comments.
- Setup: ensure_private_evidence_root mkdir+chmod0700; mktemp private staging; hard-link exclusive publish; chmod0600; independent SHA-512 readback.
- Declared locations: `<caller evidence_root>/<SHA-512-derived object path>`; `/opt/gunbc/provider-runtime/evidence (declared host_layout root, live use/population unobserved)`.
- Preserve: Confidential bytes, exact cryptographic identity, exclusive publication and custody. Do not downgrade to the structural metadata reference or infer real srv1 records from the path declaration. Classify migration as consumed retention library plus its actual witness cohort, not invented live fleet use.
- Live population: unobserved.

### `gunbc.roadmap.roadmap_event_carrier` — M3 / migrate

Actual Git-backed issue/event log and per-operation idempotency receipts; not report export.

- Operations: `roadmap_event_carrier_ensure`, `roadmap_event_private_snapshot`, `roadmap_event_publish_private`.
- Entrances/readers: `gunbc.roadmap.roadmap_event_carrier::roadmap_event_append`; `roadmap_event_append_private`; `roadmap_event_publish_private`.
- Binding: roadmap_event_carrier_layout_for_instance; logical event subject/operation identity; Git remote refs/heads/roadmap-events; ordinary origin vs srv2 preview instance-local remote.
- Setup: roadmap_event_carrier_directory enrolled in dashboard_instance_directory_demands; roadmap_event_carrier_ensure bootstraps Git carrier; per-read/append private snapshots.
- Declared locations: `<instance roadmap-events worktree>/events/<node>/<event>.json`; `<private snapshot>/<operation receipt>`; `origin:refs/heads/roadmap-events`; `<srv2 preview instance_root>/ui-preview-events.git`.
- Preserve: Publish event+operation receipt together with exact remote-ref CAS; retry binds original issue/principal/payload/precondition and recovers committed operation before checking latest revision. M3 must own full reader/writer/root transition; snapshot directories remain local Git effects until carrier is retired.
- Live population: unobserved.

### `gunbc.roadmap.roadmap_event_snapshot_census` — M3 / migrate

Reads the previous saved census to report snapshots present across censuses, then replaces it; persistent diagnostic history, not a write-only report.

- Operations: `roadmap_snapshot_census_at`.
- Entrances/readers: `gunbc.roadmap.roadmap_belt_tick_cli::belt_instance_obligation_cli (SnapshotCensusObligation)`; `gunbc.roadmap.roadmap_event_snapshot_census::roadmap_snapshot_census_for_instance`.
- Binding: HostDashboardInstance -> roadmap_event_carrier_layout_for_instance plus roadmap_snapshot_census_receipt_path_for_instance. Namespace is checked when decoding prior receipt.
- Setup: Instance receipts directory is provisioned by belt deployment member; no directory creation in census writer.
- Declared locations: `<instance_root>/receipts/roadmap-event-snapshot-census.json`.
- Preserve: Per-instance scope, prior-census population, namespace mismatch/unreadable distinctions. It is diagnostic state (not deletion authority), but it persists across runs. Retire only with explicit M3 carrier/snapshot-census retirement receipt.
- Live population: unobserved.

### `gunbc.roadmap.roadmap_verify` — M4 / retire

persist_verification_receipt defines a durable attempt receipt writer, but exact-name search finds no tracked caller in dag/ or src/. Do not invent a live process entry.

- Operations: `persist_verification_receipt`.
- Entrances/readers: No active call found for the selected producer..
- Binding: Potential layout: dispatch_attempt_verification_path_for_instance(instance,node,attempt); DashboardSshProvision refuses, DashboardLocalProvision writes. Current exact-head verification producer/readers are separate census subjects.
- Setup: Would use attempt-state directory setup; no mkdir in this unconsumed writer.
- Declared locations: `<dispatch attempt-state>/<node>/<attempt>/verification.json (declared layout only)`.
- Preserve: Delete the unconsumed legacy producer through explicit retirement after caller census. Do not use this function to justify a live state count or claim current exact-head verification evidence is migrated.
- Live population: unobserved.

## Retained local boundaries (72 modules)

These rows retain only the selected local output, transport or materialization operations. They do not exempt other durable calls in the same module. The JSON contains the original exact operation sites so a mixed module can be split rather than assigned a blanket exemption.

| Module | Selected functions | Reason |
| --- | --- | --- |
| `gunbc.auth.branch_run_grant` | `mark_branch_run_grant_approved` | GitHub Actions step output (GITHUB_OUTPUT); approval authority lives in the grant result. Retain job protocol handoff. |
| `gunbc.auth.ci_app_key_rotation` | `ci_app_key_version_verify_wet` | Verification report artifact emitted after observing the app-key mint record; retain run-scoped output, not a new key/rotation state store. |
| `gunbc.auth.gcp_iam_converge_run` | `mark_approved_as` | GitHub Actions step output (GITHUB_OUTPUT), handing the admitted approval to the next step; retain protocol boundary. |
| `gunbc.cloudflare.r2_bucket_ensure` | `ensure`, `r2_subscription_dashboard_step` | Ensure-result and subscription/dashboard instruction reports. Actual allocation/provider state belongs to B2; these file writes are report exports. |
| `gunbc.cloudflare.r2_conditional_put_race_probe` | `race_refused_before_rounds`, `race_run_with_netrc`, `race_write_body` | Race-control request-body staging and report export. Provider object mutation and release remain F/B2 control obligations; local staging is not durable authority. |
| `gunbc.cloudflare.r2_permission_group_observe` | `observe_account_permission_groups_wet` | Account permission-group observation report for the operator; retain requested report export. |
| `gunbc.demo.board_artifacts` | `write_one` | Rendered demo assets at supplied paths, not a mutable business-state store. |
| `gunbc.demo.semantic_system_readout` | `render_demo` | Rendered semantic demo/report output, not a mutable record authority. |
| `gunbc.design.brand_mark_assets_cli` | `brand_assets_write` | Generated brand assets written beneath the requested output directory; retain output materialization. |
| `gunbc.fabric.fabric_writer_identity_observe` | `fabric_writer_identity_observe_wet` | Observed fabric identity report; selection/binding calls still join A/F elsewhere. This write is a retained report artifact. |
| `gunbc.fabric.workspace_source` | `workspace_archived_placeholder_digests`, `workspace_pack_archive`, `workspace_source_finish`, `workspace_source_pack_readable`, `workspace_source_publish`, `workspace_source_refuse_archive`, `workspace_stage_placeholders` | Mixed module: selected filesystem hits stage/archive/placeholders/readback and export a receipt. Retain these local materialization effects; workspace_source_publish also calls put_fabric_blob and must join M3 (not be excluded as a whole module). |
| `gunbc.fabric.workspace_transcript_rate` | `workspace_transcript_rate_finish` | Transcript-rate observation report export; no saved prior-state read in this writer. |
| `gunbc.fleet.altra_firmware_library` | `write_reference_report` | Caller-requested reference report at a supplied output path. |
| `gunbc.fleet.app_control_plane_inspection` | `app_control_plane_inspect_wet` | App registration/hook observation report; provider reads are the source, not the file as mutable authority. |
| `gunbc.fleet.fleet_converge_plan_revision` | `resolve_plan_revision` | GITHUB_OUTPUT revision selector handoff. Preserve run/revision binding; retain job protocol output. |
| `gunbc.fleet.fleet_health` | `write_receipt` | Optional operator-chosen health report via fleet_health_receipt_env; absence of the output does not create a durable health store. |
| `gunbc.fleet.fleet_multi_principal_probe` | `fleet_multi_principal_probe_wet`, `fleet_multi_principal_probe_with_attempt`, `spark_exact_grant_probe_ci_wet`, `spark_exact_grant_probe_wet`, `spark_managed_access_standing_with_verified_credential` | Read-only host/principal/grant observation reports, including exact sudo grant output. Retain report and credential-custody boundaries. |
| `gunbc.fleet.fleet_probe_identity_observe` | `fleet_probe_identity_observe_wet` | Fleet probe identity report export. |
| `gunbc.fleet.fleet_receipt_collector` | `collect_fleet_convergence_wet` | Collector report export; the separately read host receipts are the durable authority, assigned to fleet_converge_cli/M4. |
| `gunbc.fleet.fleet_wireless_link_converge` | `wireless_link_receipt_exit` | Wireless convergence result artifact; host networking remains the independent observed state. |
| `gunbc.fleet.flex_pass_run` | `pass` | Flex provider pass result export; underlying provider/cell stores must retain their own inventory rows. |
| `gunbc.fleet.hetzner_cloud_server_control` | `read` | Requested Hetzner read report; provider resource is observed independently. |
| `gunbc.fleet.org_actions_inspection` | `org_actions_inspect_with_credential` | Organization Actions settings observation artifact. |
| `gunbc.fleet.organization_runner_roster_read` | `organization_runner_roster_observe_with_probes` | Runner roster observation artifact, preserving partial/unavailable outcomes. |
| `gunbc.fleet.printer_receipt` | `write_receipt_at` | Printer-operation result receipt. Preserve the do-not-replay-on-receipt-failure behavior; this writer is not an idempotency store or a printer-job queue. |
| `gunbc.fleet.printer_report` | `report_with_credential`, `run` | Requested printer report plus CA/credential temporary cleanup; retain local export and credential custody. |
| `gunbc.guest_image_delivery` | `guest_image_stage_inputs`, `guest_image_write_digest_receipt` | Mixed pinned guest-image materialization and digest export. Retain required local boot input placement; image acquisition/content identity belongs to the existing artifact/M3 cohort. |
| `gunbc.heal_publication` | `readback_wet` | Publication readback export/comment. The forge branch/commit is the authoritative publication and is independently read; local receipt is not its mutable root. |
| `gunbc.host.host_memory_envelope_observe` | `host_memory_envelope_observe_ci_wet` | Host memory-envelope observation report. |
| `gunbc.host.host_reach_identity_probe` | `probe_fleet_reach_wet` | Host reach/identity observation report. Host-key enrollment is a separate delegated effect and requires its own census row. |
| `gunbc.host.host_reset_return_run` | `host_reset_return_exit`, `host_reset_return_preflight_exit` | Reset-return/preflight result export; the host is observed independently. Do not claim the file is an effect-replay ledger. |
| `gunbc.live_deploy.deployed_tree_report` | `deployed_tree_report_emit` | Deployed-tree observation report; actual deployed tree and release identity remain independently observed. |
| `gunbc.live_deploy.repository_convergence_placement` | `placement_finish` | Caller-supplied local result handoff to the deploy that ordered convergence, not a stored repository desired-state authority. |
| `gunbc.machine_intake.fan_observe` | `fan_write_boot_phase_receipt`, `fan_write_observation` | Fan observation/boot-phase report; the boot workflow reads this run-local outcome. Preserve that protocol without creating a persistent fan-control store. |
| `gunbc.machine_intake.megarac_kvm_observer_observe` | `kvm_probe_conclude` | KVM probe report beside its typed KvmObserverRecord. The record value drives the probe/seat decision; this export is not the observer seat store. |
| `gunbc.machine_intake.mtcollins1_boot_diagnostic_bundle` | `mtcollins1_boot_write_bundle` | Boot diagnostic bundle export; keep retained diagnostic artifact and its evidence references. |
| `gunbc.machine_intake.mtcollins1_boot_fan_gate` | `fan_gate_summary_append`, `fan_gate_write` | Fan safety-gate output and GITHUB_STEP_SUMMARY append. Preserve observation-before-stop ordering; this is a run outcome/report channel, not a stored fan policy. |
| `gunbc.machine_intake.mtcollins1_census_image_publish` | `census_srv2_regression`, `mtcollins1_census_image_publish_wet` | Census image verification scratch cleanup and publication report; boot image physical publication is a separate existing artifact cohort. |
| `gunbc.machine_intake.mtcollins1_census_member_readback` | `census_member_readback`, `mtcollins1_census_member_readback_wet` | Remote scratch extraction/removal and member-readback report; preserve actual readback and cleanup; no mutable census-history state in these writes. |
| `gunbc.machine_intake.mtcollins1_census_qemu_host_observe` | `census_qemu_host_verdict_wet` | QEMU host prerequisite observation report directory/file. |
| `gunbc.machine_intake.mtcollins1_fan_observe` | `mtcollins1_fan_observe_boot_phase_receipt` | Boot phase fan outcome report consumed by the same workflow; preserve refusal-to-stop-boot protocol. |
| `gunbc.namecheap.observe` | `namecheap_observe_at` | Namecheap GET-hosts report; write authority explicitly withheld. |
| `gunbc.oracle_oci.always_free_ensure` | `ensure` | OCI converge result report, not provider-state authority. |
| `gunbc.oracle_oci.compartment_ensure` | `ensure` | OCI compartment ensure result report, not provider-state authority. |
| `gunbc.roadmap.roadmap_belt_tick_cli` | `belt_credential_recovery_cli`, `belt_escalate_one_cli`, `belt_integrate_one_cli`, `belt_review_head_cli`, `belt_review_report_cli`, `belt_verify_one_cli` | Selected file writes are CLI result exports and credential recovery last-result JSON. The actual lifecycle recovery store belongs to C2 and must not be excluded with these reports. |
| `gunbc.runner.runner_group_restriction_ensure_run` | `write_receipt` | Runner-group ensure report; actual group mutation and authorization are separate domain effects. |
| `gunbc.runner.runner_host_medium_publish` | `mtcollins1_runner_image_publish_wet` | Runner-image publication report; physical image materialization remains its artifact authority. |
| `gunbc.runner.runner_lost_boot_sweep_run` | `write_sweep_receipt` | Lost-boot recovery sweep report; preserve protected recovery effects in their own cohort. |
| `gunbc.runner.runner_microvm_network_observe` | `initialize_local_network_receipt`, `record_converge_readback_wet` | Mixed: direct writes export the network readback; initialize_local_network_receipt also calls publish_initial_network_receipt, whose durable authority must join its own M1/M2 row. Do not classify the entire module as output-only. |
| `gunbc.runner.runner_microvm_slot_controller` | `controller_exit` | Slot-controller outcome report keyed by invocation. Actual in-flight record/hold/teardown stores are separate direct-consumer cohorts. |
| `gunbc.runner.runner_microvm_slot_reserve` | `microvm_slot_reserve_ci_wet`, `microvm_slot_reserve_main` | Broker result/unit receipt export. Actual fabric cell hold and unresolved money-account persistence are distinct domain state; preserve those inventory rows, not a fabricated receipt-backed account. |
| `gunbc.runner.runner_microvm_slot_start` | `microvm_slot_start_ci_wet` | Slot-start result report; actual unit and hold state are independently observed. |
| `gunbc.runner.runner_password_session_tool_converge` | `runner_password_session_tool_converge_ci_wet` | Host tool convergence report export. |
| `gunbc.runner.runner_revision_job_main` | `revision_job_await` | Copy of completed jail results and runtime reading to supplied result paths. Preserve create-only and uncertainty; retained execution/accounting authority belongs to the consuming result pipeline, not this local copy primitive. |
| `gunbc.scm.repository_convert` | `scm_capture_demand_cli`, `scm_convert_repository_cli` | Caller-requested repository conversion/capture-demand result JSON. Repository mutations occur in the SCM authority and need their own M3 disposition. |
| `gunbc.site.gunb_ai_css` | `hero_entries`, `media_entries`, `sheet_write` | Rendered CSS export; lexical helper/string hits are not storage effects. |
| `gunbc.site.gunb_ai_deck` | `deck_write`, `line_prefix`, `retained_1` | Rendered deck export; line_prefix and other text-construction matches are false effect hits. |
| `gunbc.site.gunb_ai_layout_review` | `write_all` | Requested HTML review pages. |
| `gunbc.site.gunb_ai_mark` | `mark_write` | Rendered brand mark export. |
| `gunbc.site.gunb_ai_page` | `class_install`, `page_write` | Rendered site page export; source declarations remain content authority. |
| `gunbc.site.gunb_ai_tokens_review` | `review_write` | Rendered tokens review HTML at caller directory. |
| `gunbc.srv1_tasks_preapply_observation` | `observe_srv1_tasks_preapply_wet` | srv1 task-limit preapply observation report. |
| `gunbc.srv3.srv3_boot_once_cd` | `srv3_boot_once_cd_resolved` | Redfish boot/reset request-body materialization for a local command, plus credential binding; retain transport staging. It is not saved boot desired-state authority. |
| `gunbc.srv3.srv3_os_install_actuate` | `srv3_bmcweb_session_login_resolved` | BMC login request-body/token transport materialization. Preserve secret custody; not ordinary durable evidence. |
| `gunbc.tools.bmc_onboard` | `bmc_rotate_credential` | BMC credential rotation request-body materialization; actual secret custody/provider state is elsewhere. Retain transport staging, never migrate a plaintext request body as public evidence. |
| `gunbc.workspace_operator_request` | `workspace_operator_receipt` | Workspace allocation result export; allocation/hold authority is elsewhere in the same application. |
| `product.fabric.flex_vast_template_control` | `flex_template_run` | Vast template ensure report after independent provider readback. |
| `product.printed_chassis.cad_check_run` | `check_printable_cad` | Generated local CAD-check script/manifest requested by output path; external CAD execution requires local inputs. |
| `product.printed_chassis.print_preparation` | `observe_process_batch`, `prepare`, `run_kernel`, `stage_profile` | Requested print-preparation output, immutable profile snapshot, generated kernel scripts and local slicer scratch. Preserve artifact identity/readback; these are deliverable materializations, not an unrequested persistent record service. |
| `product.printed_chassis.print_workflow` | `provision_and_run`, `run` | Requested prepared print-artifact bundle and create-only prepared-queues.json. Queue hash binds the execution hold, but the durable assembly inventory and hold are separate authorities; preserve those M1/M2 joins. |
| `product.printing.print_runtime` | `observe_print_runtime` | Create-only runtime provenance artifact and local Orca probe scratch; its digest is bound into preparation. Retain artifact/provenance semantics, not a mutable runtime configuration store. |
| `product.printing.runtime_converge` | `converge` | Caller-requested runtime dependency convergence report; package installation remains a host effect. |

## Method and limitations

Read the selected function bodies around every actual effect, then followed the suspected persistent subjects to readers and entry roots with `rg -n` searches by producer, path projection and receipt reader. The body excerpt set is `/tmp/evidence-effect-body-excerpts.txt`; long functions were read around the effect, supplemented by direct reads of admission/call sites. Findings distinguish direct calls, imports/comments, and declared locations. No live state population is inferred from source. Exact search for `persist_verification_receipt(` returned only its definition. Exact searches for the Codex outer retention entrances returned wet witnesses, not production dispatch callers. A parent still must join the 84-row snapshot against whatever rows changed in the shared inventory during this audit, and join mixed-module delegated operations to their M1/M2/M3/C2 rows.

For retained report adapters, this audit does not claim that reports are valueless or may be deleted. They remain execution/evidence artifacts under their existing owners; a write failure keeps its current refusal behavior. The exclusion is from treating each report export as a new mutable fabric authority. No secret-bearing transport request body is proposed as generic retained evidence.
