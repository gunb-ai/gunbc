# Mixed local filesystem effects — exact module roster

Source pin 7daaddd1f0ce99734e1ddb57138313701455c758. Host-local files are retained only for the named external/process/materialization contract. Durable records hidden beside them are called out explicitly. No deployed population was observed.

## filesystem/platform command realizations (platform-backend)

Modules: `gunbc.executor_privileged_operation`, `gunbc.owned_directory`, `gunbc.host_command_binding`, `gunbc.typed_remote_file_write`, `gunbc.live_deploy.operations`
Root: executor_privileged_operation_argv / owned_directory_install_argv / host_write_file / typed_remote_file_* consume typed operation/target/transport path arguments, not an ambient application database root
Setup: Owning host effect or DeploymentSpec provides install owner/group/mode, desired path and transport; operations builders emit the same target
Disposition: Retain write/install/stage/rename/delete at selected host because local external consumers require files. These are provider realization boundaries; each caller-owned durable record is accounted in its domain resource, not a new DB here.

## credential custody and local activation (security-platform)

Modules: `gunbc.auth.approval_keyring_converge`, `gunbc.dispatch_credential_acquisition`, `gunbc.dispatch_credential_snapshot`, `gunbc.fleet.host_credential_custody_converge`, `gunbc.workspace_commissioning_apply_readback`, `gunbc.workspace_commissioning_executor_submission`
Root: keyring converge owns /etc/gunbc-roadmap and approval store directories; credential acquisition takes HostDashboardInstance/attempt root; workspace executor publishes under /etc/credstore after root ownership reading
Setup: keyring_ensure_dirs; credential_acquisition_slots_directory/cursor_attempt_runtime_directory in dashboard_instance_directory_demands; custody_dir_ensure_argv; executor credential root ancestry readback
Disposition: Retain actual secret/keyring/credential files and create-only cursor ownership markers on consuming host. M1/C2 own nonsecret acquisition/approval metadata; do not upload raw credentials to generic fabric metadata. Preserve snapshot tombstone stage+atomic rename and provider-specific credential consumption.

## SSH configuration and per-attempt materialization (security-platform)

Modules: `gunbc.fleet.fleet_host_key_enrollment`, `gunbc.fleet.fleet_known_hosts_anchor`, `gunbc.fleet.fleet_ssh_credential_verify`
Root: host enrollment run supplies admitted principal/SSH directory; known-host anchor and public selector derive target/fleet-ssh-attempt-*; multiplex directory is RUNNER_TEMP-bound
Setup: host-key enrollment makes .ssh and create-new config/authorized_keys then appends; known-host anchor/multiplex create-only directory; public selector exact existing-content check
Disposition: authorized_keys/known_hosts/config are durable external SSH configuration, retained as platform effect; same source declaration/probe remains authoritative. Attempt key selector/multiplex sockets are local transport resources. Enrollment receipt is explicit output adapter.

## boot media publication/local realization (M3)

Modules: `gunbc.bmc.bmc_netboot_serve`, `gunbc.machine_intake.boot_image_fetch`, `gunbc.revision_job_image_delivery`
Root: bmc_netboot_apply_prepared consumes prepared layout incl /var/lib/gunbc/artifacts/netboot, OpenBMC /tmp/netboot and per-attempt scratch; boot_image_fetch supplied host/staging/verified target; revision_job_image_build_wet root
Setup: prepared host command plan ensures path before transfer; verified boot image hard-link/no-replace and owned staging cleanup; revision-image build root/store/index dirs
Disposition: Artifact bytes + identities belong fabric blob/materialization adoption; the BMC HTTP/tftp/image consumer still requires an installed local file. Preserve provenance verification and local placement readback; fixed report files are exports.

## private materialized source workspaces (M3)

Modules: `gunbc.fabric.fabric_source_snapshot`
Root: materialize_source_snapshot/allocate_under_admitted_root receives admitted provider layout; snapshot_write_file uses typed snapshot member under private workspace
Setup: establish_provider_base create-only 0700 owner root, allocate mktemp under admitted root, per-member parent ensure, release_provider_workspace
Disposition: Retain provider workspace realization and cleanup; immutable source closure belongs metadata/blob plane. Local files are process inputs, not an independently authored source-of-truth store.

## native build roots and outputs (M3)

Modules: `gunbc.devboot.transport`, `gunbc.devboot.build`, `gunbc.devboot.produce`
Root: devboot build/serve/materialize/produce entry operands bind request, checkout, artifact/materialization roots; devboot_ensure_directory is transport leaf; GUNBC_MATERIALIZED_TREE_IDENTITY binds identity
Setup: build publish_request/materialize_ref_tree/publish_produced_tree; produce snapshot/materialize_source set directories via shared transport
Disposition: Do not replace source tree/output directory with remote string path. Share immutable products and typed records via fabric/provider while preserving required local compiler/build materialization and CLI receipt export.

## local process effect and reaping (platform-backend)

Modules: `gunbc.build_cache.build_cache_endpoint_path`, `gunbc.host.host_hygiene_reaper_remediate`, `gunbc.host.host_effect_codex_supervised_turn`
Root: stale_socket_unlink_command receives admitted endpoint path; apply_residual_reap_effect receives discovered owned residual dir; supervised turn apply receives private preamble/evidence root
Setup: Owning endpoint/host-hygiene/attempt context; no standalone storage namespace
Disposition: Unix socket unlink and proved orphan cleanup are live process/filesystem effects. Supervised preamble is local process input; generation/client command IDs separately C2 durable resource.

## installed host release and ownership attestations (platform-backend)

Modules: `gunbc.host.host_effect_realize`, `gunbc.live_deploy.emit`, `gunbc.live_deploy.repository_convergence`, `gunbc.fleet.site_pxe_edge_converge`, `gunbc.runner.runner_host_file_converge`
Root: host_effect_apply selected typed host effect + HostEffectTransport; live_deploy_apply_prod_role_transaction_wet -> DeploymentSpec release locus/owner/instance root; site PXE config /etc/gunbc-site-pxe-edge.dnsmasq.conf
Setup: live_deploy typed install/owned directories/readback; host_effect_realize installs root helper snapshots, systemd/sudoers/controller files; retire_superseded_file under observed deployed source
Disposition: Retain actual host configs, executables, ownership markers, .gunbc-tree-receipt and VMM/launcher readback receipts as materialization-provider attestations on that same host. Archive history can be fabric metadata, but remote receipt cannot establish local installed bytes. BMC token scratch is credential custody, not archival state.

## local package/toolchain release realization (M3)

Modules: `gunbc.host.verified_archive_install`, `gunbc.package_delivery`, `gunbc.runner.runner_browser_toolchain`, `gunbc.runner.runner_qemu_toolchain`, `product.printing.cad_runtime_converge`
Root: Verified archive plan supplies archives/staging/install target; Codex bundle typed release/build/publishing roots; browser/QEMU location.base; CAD environment spec derives prefix/marker/lock
Setup: digest-verified fetch; mktemp staging then no-replace publish; package current symlink; browser readback/grant drop-in; CAD create-only directory lock
Disposition: Immutable package/toolchain artifacts should use shared fabric blob/materialization provider. Preserve local installed binaries/current links/runtime marker. CAD directory lock is a local build exclusion primitive, separate M2 if generalized, not a metadata overwrite. Runtime/spec receipts remain identity/readback evidence.

## guest images and attempt devices (platform-backend)

Modules: `gunbc.runner.runner_attempt_launch`, `gunbc.runner.runner_claim_set_guest`, `gunbc.runner.runner_guest_image`, `gunbc.runner.runner_host_medium`, `gunbc.runner.runner_microvm_boot_probe`, `gunbc.runner.runner_microvm_host_ready`, `gunbc.runner.runner_microvm_lifecycle_realize`, `gunbc.runner.runner_revision_job_guest`, `gunbc.runner.runner_revision_job_image`, `gunbc.runner.runner_revision_job_launch`, `gunbc.runner.runner_revision_job_stage`, `gunbc.runner.runner_revision_job_toolchain`, `gunbc.runner.workspace_guest_image`, `gunbc.runner.workspace_microvm`
Root: Selected runner host/slot/attempt binds jail_root, image root, staging/work/store directories and bootstrap/results device. claim guest reads GUNBC_CLAIM_SET_RESULTS_DEVICE/RESULTS_BYTES. Revision job entry variables supply dirs/spec/image digest
Setup: runner host/image/controller desired plans stage kernel/rootfs/vendor/toolchain/config/unit and credentials with explicit mode/uid/gid; attempt cleanup removes exact owned paths
Disposition: Retain guest device I/O, /etc files inside image, jail rootfs copy and cgroup.kill. Images/toolchain bundles use M3 immutable artifact provider; revision receipt/index and cell/network readiness are separate M4 rows. Do not erase consumed result bytes until durable receipt publication succeeded.

## microVM host network config realization (platform-backend)

Modules: `gunbc.runner.runner_microvm_network_apply`
Root: local wet/CI entry builds selected host/scope network plan; write_local_staged_file binds scope.root_path and file.path
Setup: stage private file then mv exact target; owned/privileged network operations and cleanup
Disposition: Host daemon/sysctl/network config remains local platform state. Separate runner_microvm_network_receipt_store is M4 authority; target plan/apply report remains output projection.

## serving/model installation and access config (M3)

Modules: `gunbc.serving.rented_release`, `gunbc.serving.serving_artifact_placement`, `gunbc.spark.fabric_rail_apply`, `gunbc.spark.managed_access_apply`, `gunbc.spark.managed_grant_install`
Root: ServingArtifactRoots/model_link and selected serving target; rented release owner/paths; spark selected host/grant/key/rail facts
Setup: ensure partial dir, verified move to digest path, symbolic next-link+rename; explicit root install/grant retirement
Disposition: Retain exact installed artifact/model symlink and SSH/systemd access config. Immutable model/engine bytes should come through blob provider; authorization scope cannot dissolve into generic storage grants. Output receipts separate.

## acquired URL file installation (M3)

Modules: `gunbc.url_file_acquisition`
Root: subject.parent_directory/install_path, part_path and admitted URL/file digest supplied to url_file_publish
Setup: ensure_url_file_parent; move verified part to destination; cleanup mismatch/infra refusal
Disposition: Retain local placement consumed by external tool; artifact acquisition should use existing origin/blob provenance and local materialization provider, never path-only successful publish.

## OpenBMC fan config actuation (platform-backend)

Modules: `extdeps.bmc.openbmc_fan_control`
Root: openbmc_execute receives modeled OpenBmc operation and password SSH host/user/known_hosts binding
Setup: CopyFilePreservingMetadata/MakeDirectory*/RenameReplace/Remove*/SynchronizeFilesystems on target BMC through closed PasswordSshTransport
Disposition: External BMC configuration backup/stage/publish/rollback is actual local platform state. Preserve same-filesystem atomic rename and sync semantics; use fabric only for immutable backup/archive evidence, not as substitute for active config.

## attempt exit durable spool (C2)

Modules: `gunbc.durable_handoff_host`, `gunbc.roadmap.roadmap_attempt_exit_spool`
Root: attempt ExecStopPost emitted by attempt_exit_stop_post_command -> create-only encoded directory in instance inbox; belt vendor-pane death -> attempt_exit_record_put_mkdir_name; belt_exit_entry_move_at transitions exact stage directories
Setup: attempt_exit_spool_prepare and instance own directory demands; inbox/queued/running/done/refused paths; all producers/readers and systemd emit route must cut together
Disposition: This IS durable recovery state, not mere mkdir setup. Replace create-only name encoding and atomic stage moves with metadata CAS transition/index only after failure-safe host exit capture contract exists. Retain local exit handoff adapter if host cannot reach fabric at process death.

## SOL boot capture, observation-loss and delivery records (M4)

Modules: `gunbc.machine_intake.mtcollins1_boot_run`, `gunbc.machine_intake.mtcollins1_sol_notice`
Root: mtcollins1_boot_run entry and mtcollins1_sol_notice_wet read mtcollins1_boot_sol_capture_path_env/pid_path_env; SolNoticePaths derived siblings and GITHUB_STEP_SUMMARY
Setup: CI spec boot prelude/collector/watch setup binds one attempt capture and pid path; sol_hold supervisor creates/processes local control and capture files
Disposition: Split persistent boot/observation-loss/pre-teardown evidence and notice delivery identities into immutable fabric evidence+metadata if retained beyond job. Keep collector PID/activation/stop/ready signals and live capture local; step summary/log writes are runner protocols. Ledger is best-effort, delivery marker affects duplicate behavior and is not disposable diagnostic noise.

## KVM observer journal and controls (M4)

Modules: `gunbc.machine_intake.megarac_kvm_still`
Root: megarac_kvm_observer_start(host,dir,...); current production start always returns KvmObserverNotStarted because no public client; historical journal readers remain
Setup: Existing journal root supplied by boot attempt; trigger/stop/workflow files only under admitted KvmObserverStarted current observation
Disposition: No current reachable start writes should be advertised as active deployed store. Preserve historical journal evidence via artifact storage and local IPC controls if client returns; record explicit source rung drop, not a successful KVM capability.
