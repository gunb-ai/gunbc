# Filesystem effect audit: source-read durable resource map

Pinned to 7daaddd1f0ce99734e1ddb57138313701455c758. This closes named resources found at the scanned effect boundary; it is not a claim that lexical selectors prove every possible indirect effect. Runtime deployed populations were not observed. `classified_inventory.json` keeps every candidate, exact operation site, retained/excluded reason and local binding evidence; this map splits mixed resources by accountable package.

## dashboard authentication sessions (M4)

Modules: `gunbc.auth.session_store`
Operations: session_store_save_keyed, session_store_delete_keyed, session_store_purge_execute
Root: roadmap_serve routes -> roadmap_auth_routes receives HostDashboardInstance -> session_store_layout_for_instance -> instance.root/auth-sessions
Setup: session_store_directory -> dashboard_instance_directory_demands -> dashboard_instance_apply and live_deploy deployment directories
Disposition: fabric_storage_put/head/advance; domain revocation/tombstone + closure/index, retain HTTP cookie interface
Preserve: Private consumer identity; admitted AuthStoreKey; principal purge must enumerate whole admitted index, never infer absence from unreadable state.
Existing state: Current population UNOBSERVED; source declaration only.

## OIDC login transaction consumption (M4)

Modules: `gunbc.auth.login_transaction_store`
Operations: login_transaction_store_save_keyed/delete_keyed
Root: roadmap_auth_routes login/callback -> login_transaction_store_layout_for_instance -> instance.root/auth-login-transactions
Setup: login_transaction_store_directory in central dashboard directory demand list; both deployment routes
Disposition: metadata object + head CAS; consumption tombstone substitutes exclusive .consumed creation and unlink
Preserve: One successful consumer only, preserve return-path readmission, no replay by deleting consumed marker.
Existing state: Current population UNOBSERVED; source declaration only.

## dashboard principal profile projection (M4)

Modules: `gunbc.auth.profile_projection`
Operations: profile_store_upsert_write; profile_store_read_all
Root: roadmap_serve profile read/upsert and served observation -> profile_store_layout_for_instance -> instance.root/auth-profiles
Setup: profile_store_directory in central dashboard directory demand list; both deployment routes
Disposition: metadata object + head CAS + roster index
Preserve: Merge existing profile before publication; key is principal-label digest; listing semantics require typed logical index.
Existing state: Current population UNOBSERVED; source declaration only.

## OIDC JWKS cache (M4)

Modules: `gunbc.auth.oidc_id_token_verification`
Operations: jwks_fetch_and_cache (only)
Root: roadmap_auth_routes -> jwks_cache_layout_for_instance -> instance.root/auth-jwks/cache.json
Setup: jwks_cache_directory in central dashboard demand list
Disposition: shared metadata cache with freshness policy; retain rs256_verify_over_key temporary openssl key/message/signature files locally
Preserve: Do not silently use stale keys after fetch refusal; issuer/kid and fetched_at remain domain policy.
Existing state: Current population UNOBSERVED; source declaration only.

## approval device push registration (M4)

Modules: `gunbc.auth.approval_device_routes`
Operations: store_push_registration, push_registration_path
Root: approval broker device enrollment route -> enrollment_id -> /var/lib/gunbc/approval-device-push/<id>.json
Setup: Exact root/helper search found no provisioning registration outside this module. This is a concrete source provisioning gap, not inferred live absence.
Disposition: metadata object/head with authenticated device scope
Preserve: Stored registration currently unread: source explicitly declares APNs sender missing and app polling. Do not claim an implemented push-delivery reader.
Existing state: Current population UNOBSERVED; source declaration only.

## attempt companion records (C2)

Modules: `gunbc.compute.work_provider_local`, `gunbc.roadmap.roadmap_belt_actuate`, `gunbc.roadmap.roadmap_dispatch_actuator`, `gunbc.roadmap.roadmap_attempt_metering`, `gunbc.roadmap.roadmap_attempt_request_record`, `gunbc.roadmap.roadmap_session_placement`, `gunbc.roadmap.roadmap_served_observation`, `gunbc.roadmap.roadmap_verify`, `gunbc.roadmap.roadmap_launch_deployment_cli`
Operations: All per-attempt Write/WriteCreateNew/Delete sites in classified_inventory: request identity, admission, provider origin, oracle pin, metering roster/seal/index, capture intent/commit, verification, integration/publication, placement/release/settlement notes
Root: belt_run_once_cli/belt_instance_obligation_run and roadmap_serve -> selected HostDashboardInstance; dispatch_attempt_state_root and per-node/per-attempt path helpers; SessionPlacementStores.records_dir distinct from capacity ledger store
Setup: roadmap_dispatch_actuator::attempt_state_prepare_argv_for_instance; dashboard_instance_own_directory_demands and central consumer list; dashboard apply/live_deploy. C2 must include these companion records with all six lifecycle consumers.
Disposition: metadata objects + CAS heads/indexes; blob objects for logs/evidence; retain stdout capture, unit control, worktree and spool local realization
Preserve: No root-only replacement that leaves raw companion records active. Preserve exact attempt identity, terminal precedence, create-only identity, recovery enumeration, metering whole roster/seal and host-bound cleanup facts.
Existing state: Current population UNOBSERVED; source declaration only.

## harness turn records and seat pointers (C2)

Modules: `gunbc.harness.harness_cli`, `gunbc.harness.harness_turn`
Operations: harness_seat_pointer_persist, harness_write_events, harness_prepare_round, harness_send_round
Root: harness_worker_cli/durable_worker_cli/reviewer/auditor/supervisor receive events_path and scratch paths from attempt launcher; harness_worker_config/HarnessTurnConfig
Setup: Selected attempt setup supplies events parent; request/response scratch remains local to provider execution; no standalone host-global root
Disposition: metadata seat pointer and append/event chain via fabric; byte request/response evidence via blob; retain local transport request staging
Preserve: Seat pointer links exact grant owner; token/metering readers consume event records. Probe /tmp/gunbc-harness-probe-events.jsonl is instrument output, not production root.
Existing state: Current population UNOBSERVED; source declaration only.

## supervised Codex generation and command identity (C2)

Modules: `gunbc.codex_supervised_turn`, `src/v1/stage0/src/codex_app_server_stdio_session.rs`
Operations: persist_codex_supervised_turn_generation/client_command_id; run_codex_app_server_stdio_session writes client_command_id_path
Root: host_effect_codex_supervised_turn_apply supplies evidence_root; stdio session request carries command ID path from same attempt context
Setup: Host attempt/evidence root selected by effect application; preamble path remains a local input
Disposition: metadata object/head for generation and idempotency identity; Rust process adapter should consume canonical outcome rather than duplicate authority
Preserve: Persist identity before external session; retain uncertainty after unknown remote submission, do not allocate fresh command ID blindly.
Existing state: Current population UNOBSERVED; source declaration only.

## system-unit launch request spool (C2)

Modules: `gunbc.roadmap.attempt_system_unit_launch`, `gunbc.roadmap.roadmap_attempt_exit_spool`
Operations: attempt_system_unit_enqueue_request, attempt_system_unit_write_answer, create/rename staging directory; attempt_exit_spool_prepare
Root: attempt_system_unit_launch_cli(instance_root) -> instance_for_root; belt launch -> enqueue; attempt_unit_spool_{request,staging,claimed,answer}_dir
Setup: HostDashboardInstance own directory demands; launcher helper under /opt/gunbc/attempt-system-unit-helpers and sudoers/systemd install
Disposition: Retain privilege-boundary local spool until equivalent typed fabric request/claim/answer protocol replaces both sides; durable outcomes then metadata heads
Preserve: No requester-selected answer path; helper derives destination; atomic no-replace directory claim and user/owner boundary are not generic remote-file operations.
Existing state: Current population UNOBSERVED; source declaration only.

## publication helper request/answer spool (C2)

Modules: `gunbc.roadmap.roadmap_publication_helper`
Operations: publication_request_enqueue, publication_helper_retire_request, publication_helper_record_subject_outcome
Root: publication_helper_run_once_cli(instance_root) and belt enqueue -> publication_request_dir/answer_dir; keys from subject hash
Setup: HostDashboardInstance own directory demands and installed helper service/privilege context
Disposition: metadata request/answer object + CAS claim; retain local privilege handoff only as adapter until both reader and writer migrate
Preserve: Request contains subject, never output destination. Preserve retire/consume only after result recording and exact authority boundary.
Existing state: Current population UNOBSERVED; source declaration only.

## private provider wire evidence (M4)

Modules: `gunbc.provider_wire_evidence`
Operations: retain_private_evidence_bytes_detailed; publish_evidence_object_exclusive
Root: Caller supplies private evidence root and bytes; evidence names derived from content digest; callers are provider/attempt execution owners
Setup: ensure_private_evidence_root creates parents then narrows root; publish hardlink then owner-read-write mode
Disposition: fabric blob create-if-absent + metadata references; retain private staging files
Preserve: Do not broaden private evidence reader/writer scope; exact content identity and exclusive publication; report cleanup residue.
Existing state: Current population UNOBSERVED; source declaration only.

## printer fixed start claims and observations (M4)

Modules: `gunbc.fleet.printer_operator_approval`, `gunbc.fleet.printer_batch`, `gunbc.fleet.printer_start_reconciliation`
Operations: start_authorized; drive_tick; reconcile/monitor report writes; create-only attempt/completion claims
Root: printer_operator_approval::run -> approve_and_print/start_authorized; batch drive/drive_tick -> /var/lib/gunbc/printer-starts from printer_workflow_artifact
Setup: Source says shared pre-provisioned LAN-host state; no typed setup registration found by exact root/helper search. PrinterExecutionHold shares root and is separate M2 resource.
Disposition: metadata create-if-empty CAS claims + versioned batch head + immutable evidence objects/index
Preserve: Start authority survives output folder/digest/printer reassignment; uncertain physical start never automatically releases/replays claim; completed awaits inspection.
Existing state: Current population UNOBSERVED; source declaration only.

## OCI launch replay record (M4)

Modules: `gunbc.oracle_oci.instance_ensure`
Operations: oci_start_attempt, oci_clear_attempt
Root: oci_free_instances -> planned named instance -> /etc/gunbc/oracle-oci/launch-attempt-<name>.json
Setup: host_credential_custody_converge OracleOciApiSigningKey creates root-only directory; shares signing principal
Disposition: metadata object + CAS head/tombstone
Preserve: Keep first-send instant, exact request body and 82800-second retry window; definitive noncreation only can retire; preserve unknown-outcome reconcile.
Existing state: Current population UNOBSERVED; source declaration only.

## OCI convergence exclusive lock (M2)

Modules: `gunbc.oracle_oci.instance_ensure`
Operations: oci_free_instances WriteCreateNew/Delete of oci_instance_lock_path
Root: oci_free_instances -> /etc/gunbc/oracle-oci/instance-converge.lock
Setup: Same root-only OCI credential custody directory
Disposition: exclusive-hold policy over fabric-backed CAS; do not replace by ordinary metadata overwrite
Preserve: Holder/release/recovery semantics required before cutover; source current lock is create-only file with timestamp.
Existing state: Current population UNOBSERVED; source declaration only.

## deploy transition records: declared but mutation callers absent (M4)

Modules: `gunbc.deploy_transition`
Operations: acquire_transition_lease, set_transition_inhibition, mark_transition_partial, clear_transition_records
Root: TransitionSubject.instance_root; transition-lease.json and transition-inhibition.json. Repository exact-name references show mutation functions only at definitions; source comments mention them.
Setup: Dashboard instance root exists before transition observation; no selected writer call found. Read-side transition_actuation_standing is separately meaningful.
Disposition: Do not call active deployed migration. Retire unused mutation surface or bind through selected transition composition before designing replacement; any retained durable protocol uses CAS/hold semantics.
Preserve: Inhibition presence blocks actuation even undecodable; partial leaves it set; clear inhibition before lease; current source reachability is not evidence of no existing host files.
Existing state: Current population UNOBSERVED; source declaration only.

## runner cell readiness records (M4)

Modules: `gunbc.runner.runner_microvm_cell_readiness_store`
Operations: persist_one_record, enumerate/read records
Root: runner slot/controller observation and actuation -> cell_record_path(cell) -> /var/lib/gunbc/microvm-cell-readiness from runner_microvm_cell_readiness_location
Setup: Ownership readback at store root; exact literal/location reference search found no separate provisioner, so declared setup closure is a gap
Disposition: metadata CAS per cell + roster/index
Preserve: Sanitation, quarantine, readiness and in-flight attempt state gate launch; preserve full record validation and owner check; no absence from unreadable listing.
Existing state: Current population UNOBSERVED; source declaration only.

## runner converged network receipt (M4)

Modules: `gunbc.runner.runner_microvm_network_receipt_store`, `gunbc.runner.runner_microvm_network_observe`
Operations: publish_initial_network_receipt (durable); initialize_local_network_receipt/record_converge_readback (local export)
Root: initialize_local_network_receipt -> publish_initial_network_receipt -> runner_microvm_converged_receipt_path; host identity/read_network_receipt
Setup: network_receipt_ancestors_owned and runner network setup
Disposition: create-if-empty metadata head with typed host network receipt; retain local diagnostic output projection
Preserve: Preserve root ownership, selected host/slot population, readback admission; local receipt file and authoritative converged receipt are different resources.
Existing state: Current population UNOBSERVED; source declaration only.

## runner revision job receipt and by-revision index (M4)

Modules: `gunbc.runner.runner_revision_job_receipt`, `gunbc.runner.runner_revision_job_main`
Operations: publish_revision_job_receipt; revision_job_await local results/runtime copies
Root: runner_revision_job_main run -> GUNBC_REVISION_JOB_STORE_ROOT, work_dir receipt.json staging; published receipts + index
Setup: publish_revision_job_receipt ensures receipts/index dirs; revision job host installation/stage roots supply process inputs
Disposition: immutable structured receipt + links/index head; blob for binary output; retain local guest result drive/export
Preserve: Preserve complete staged content before hard-link visibility, unique publication and index consistency; no hidden successful publication when index failed.
Existing state: Current population UNOBSERVED; source declaration only.

## SCM repository envelope/object shards (M4)

Modules: `gunbc.scm.repository_save`
Operations: create_repository, save_repository, save_object_shards, persist_sharded_repository
Root: scm.cli::scm_init(directory,name) -> initialize_repository/create_repository_at_absence; repository_convert -> save_repository_sharded_guarded(path,...)
Setup: SCM caller-selected repository path/parent; repository_slot binding guards separate writer authority
Disposition: metadata objects + CAS repository head; object shards become immutable objects/closure; local repo file is explicit export if still consumed
Preserve: Preserve generation guard, whole shard roster and object integrity; migrate repository_slot with document writers.
Existing state: Current population UNOBSERVED; source declaration only.

## host-cap release obligation (M4)

Modules: `gunbc.runner.runner_host_cap_bracket`
Operations: write_bracket_obligation/read_bracket_obligation
Root: RUNNER_TEMP/<runner_host_cap_obligation_file_name>; identity GITHUB_RUN_ID + GITHUB_RUN_ATTEMPT + host; acquire_bracket_over
Setup: Runner job directory is GitHub protocol setup; workflow cleanup/release reads obligation
Disposition: Retain local per-job obligation adapter; if it must outlive runner loss, add metadata object/head through same bracket owner
Preserve: Do not call ordinary diagnostics: held unit obligations affect release. Empty held set writes nothing; exact run/attempt/host identity and discharged status preserved.
Existing state: Current population UNOBSERVED; source declaration only.

## native product cache (M3)

Modules: `src/v1/stage0/src/cli_run/native_product_cache.rs`, `gunbc.native_product_shared_transfer`
Operations: commit/restore_from_shared_store/record_outcome; restore/save bundle staging
Root: native_lane_runner prepare -> GUNBC_NATIVE_CACHE_ROOT/native-products; merge_group publication standing; shared transfer through named DAG entry
Setup: commit creates staging; CI selects cache root and writer credential; immutable artifact verification before install
Disposition: fabric blob bytes + metadata manifest/addressing; retain local verified executable/materialization cache
Preserve: Verify identity and executable before cache hit; refuse corrupt/partial object; publication only after red+restore completed; append receipt.jsonl is diagnostic output.
Existing state: Current population UNOBSERVED; source declaration only.

## native compiled workspace cache and lock (M3)

Modules: `src/v1/stage0/src/v1_interpreter.rs`
Operations: run_cached_process_spec, emit_host_materialize_workspace_files, emit_host_run_transport_cached_in_workspace, eval_emit_host_native_cache_evict_builtin
Root: run_native_bundle_process_cached and modeled emit_host_run_transport_cached; native_cache_rebase_workspace_dir uses GUNBC_NATIVE_CACHE_ROOT or per-user XDG_CACHE_HOME/HOME
Setup: Creates content-keyed realization root and context-keyed workspace; local .native_publish.lock spans materialize/build/ready marker
Disposition: Retain local materialization provider and physical lock; shared immutable artifacts through fabric blob/metadata when model migrated
Preserve: Cargo absolute build paths mean naive build-elsewhere rename changes behavior; .native_ready and cold_compile receipt only after complete build; local lock lifetime is process file-lock semantics.
Existing state: Current population UNOBSERVED; source declaration only.

## compiler scoped run history (M4)

Modules: `src/v1/stage0/src/cli_run/native_lane_runner.rs`
Operations: CensusRunRecordFile::open/complete; write_record_atomically
Root: native census CLI -> process_workspace_root()/target/census-runs; optional against baseline
Setup: open creates root; same execution creates incomplete record then atomic replacement
Disposition: metadata object/head/index for completed and incomplete runs; retain local CLI artifact/export
Preserve: Newest completed baseline only; include exact producer/input identities and all JSON rows; no old record mistaken for this run; concurrent runs independent.
Existing state: Current population UNOBSERVED; source declaration only.

## compiler rustfmt cache (M3)

Modules: `src/v1/stage0/src/required_regen_host.rs`
Operations: ResolvedFormatter::with_normalize_cache; normalize_cache_write
Root: regen scope -> workspace/target/stage0-regen-rustfmt-cache/<resolved rustfmt version>
Setup: with_normalize_cache queries actual formatter version and creates selected directory
Disposition: Retain derived local materialization cache; shared immutable result can use fabric put/get after source model owns key
Preserve: Raw bytes compared whole despite digest locator; formatter version changes namespace; stage+rename prevents partial read.
Existing state: Current population UNOBSERVED; source declaration only.

## compiler regeneration rollback journal (M4)

Modules: `src/v1/stage0/src/required_regen_host.rs`
Operations: journal_paths_for_subject; restore_regen_convergence_journal_for_subject; install_convergence_stage_with_backend; run_regen_round_cost
Root: regen round scoped by starting commit/subject -> workspace/target/regen-convergence-journal
Setup: journal creates backups+manifest before replacement; restore copies prepared backups then renames; completed journal removed
Disposition: Preserve local transaction journal until typed equivalent can restore same filesystem; archive durable metadata/backup blobs via fabric, keep offline local recovery contract
Preserve: Rollback before-image, starting commit, absent/present paths and atomic restore are authority. Must not exclude because target/ or Rust; pure remote dependency can make interrupted bootstrap unrecoverable.
Existing state: Current population UNOBSERVED; source declaration only.

## recorded fixture store (M4)

Modules: `src/v1/stage0/src/recorded_fixture.rs`
Operations: RecordedFixtureStore::record
Root: claim_batch --record --fixture-store <path> -> RecordedFixtureStore::open -> interpreter fixture_store
Setup: Caller supplies root; record creates operation/input-hash parents
Disposition: Metadata immutable fixture object and operation/input index; retain local export/import for hermetic execution
Preserve: 30-day freshness policy, exact inputs/response, response-drift refusal and serialization admissibility remain; fixtures survive runs and are not disposable test scratch.
Existing state: Current population UNOBSERVED; source declaration only.
