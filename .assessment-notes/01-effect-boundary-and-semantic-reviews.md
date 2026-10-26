# Effect-boundary census and semantic reviews

Source baseline: `7daaddd1f0ce99734e1ddb57138313701455c758`; source audits and pricing retrieval: 2026-10-23. This is a preserved research export for the broader architecture assessment. It is not an approved migration plan, runtime verification, or permission to deploy. Package labels refer to the now-paused storage-only draft. No live record population was observed. Raw originals and exact-source data remain in `raw/`; the export manifest records their byte hashes.

The final candidate roster is [classified_inventory.json](raw/effect-boundary/classified_inventory.json): **637 candidate files, not 637 durable stores**. The primary scan found 635 files / 4,625 lexical sites; a typed-helper supplement added two unique files. The 26 resource rows, 61 mixed-local reviews, 45 compiler reviews and 84 receipt/output reviews overlap. The semantic reviews supersede preliminary classes; do not add their counts. Exact selector sets are in [selectors.json](raw/effect-boundary/selectors.json), [supplement.py](raw/effect-boundary/supplement.py), and the Rust supplemental JSON files.

The frozen 84-row receipt/output review covers the then-current 7 `migrate_evidence` + 77 `local_output_adapter` rows. It does not independently close the earlier 91-row grouping or prove repo-wide execution coverage.



## Method, selectors and limits

Original: [raw/effect-boundary/METHODOLOGY.md](raw/effect-boundary/METHODOLOGY.md).

# Pinned filesystem/shell source audit

Source: 7daaddd1f0ce99734e1ddb57138313701455c758. No repository edits, builds, new tests or runtime filesystem probes. DESIGN.md and docs/plans/fabric-storage.md informed semantic dispositions.

## Reproduction

From that checkout, run `python3 /tmp/storage_effect_audit_7daaddd1f0ce/scan.py`, then `supplement.py`, `classify.py`, `refine.py`, `rust_review.py`, and `local_compiler_review.py` in that directory. These are disposable audit tools, not a permanent census/gate or parallel model authority. The exact patterns and name sets are persisted in `selectors.json`, `supplemental_inventory.json` and the scripts. The JSON source excerpts and line locations make each row inspectable without rerunning an effect.

Primary universe: `git ls-files -z`, 10381 tracked paths, 10369 UTF-8/text-readable files and 12 nontext files. Primary language/config selector: .dag/.rs/.py/.sh/.yml/.yaml/.toml or a basename containing Dockerfile/Makefile: 9761 files. Hidden .github workflow files are included because selection is from Git, not rg's hidden-file defaults. Primary result: 635 candidate files and 4625 lexical sites. These are NOT durable domain counts. A typed-constructor/service supplement adds two otherwise absent files; final candidate roster is 637 files.

Read-only text outside that language selector is not silently equated to absence of effects: root audited .githooks/{pre-commit,pre-push,generated-artifact-merge}, tools/fabric_ci_fci1_bounded_execution_context.env, and recovered initramfs.spec.in separately. Swift/iOS persistence and fixture C/JS are joined from /tmp/ios-persistence-inventory.md under the parallel owner. Markdown, JSON/text evidence, lockfiles, source patches/recovered files, images and binaries are not executable source merely because they contain historical commands. The parent integration should retain explicit dispositions for its supplemental files.

## Selectors and interpretation

Direct DAG operations: Filesystem.{Write,WriteBytes,WriteOwnerOnly,WriteCreateNew,WriteCreateNewWithMode,LinkCreateNew,Delete}; shell.{Mkdir,Mktemp,Link,Remove,Move,Chmod,Copy} exact operation names in selectors.json. Filesystem helper assessors `filesystem_delete/create_new/link_create_new` are included as boundary anchors, but their presence alone is not an effect. GNU coreutils/mkdir/chmod/chown argv constructors and OpenBMC remote filesystem operations are explicitly enumerated by supplement.py; constructor rows are separated from realized stores.

Direct DAG/Rust matching strips comments, ordinary and raw strings and character literals while retaining newlines; escaped newline Rust strings are handled. Quoted source is separately inspected for generated Rust effects and shell program/redirection candidates. Shell candidate programs: mkdir/install/cp/mv/rm/rmdir/ln/chmod/chown/touch/truncate/tee/dd/mktemp, including absolute command paths and argv string tokens. Redirection excludes /dev/null, stderr/stdout forwarding. Actual command builders, exported shell realizations and generated workflow sources are inspected; CSS `>` and English/model/example words are explicit false positives.

Rust mutation candidates: fs/std::fs/tokio::fs write/copy/rename/remove/create/link/set_permissions; File/OpenOptions/DirBuilder constructors; write_all/set_len/set_permissions/create_new/create/append/truncate methods. These deliberately overselect: Vec/String append/truncate, formatter/socket/stdout/pipe writes and OpenOptions configuration are not file publication. rust_review.py bounds each cfg(test) item/block by its braces instead of wrongly excluding everything after the first test. This found production regeneration journal functions after an interspersed test module.

Supplemental Rust check used unqualified names `write rename create_dir create_dir_all remove_file remove_dir remove_dir_all copy symlink hard_link set_permissions` with `(?<![:.\w])(?:NAME)\s*\(` after lexical stripping, excluding fn declarations. All 14 matches resolve to already-inventoried local test helpers in cli_run.rs, emitted_closure_compile_host.rs, or interpreter_dispatch_bijection_real_roster_red.rs; no new production domain. Direct `use (std|tokio)::(fs|os::unix::fs)` imports introduced none of those functions. Additional `.write(`, `.write_fmt(` and `write!/writeln!` sites are in rust_import_write_supplement.json: real file macro writes are native_product_cache receipt append and native_lane_runner GITHUB_OUTPUT, both already present through OpenOptions; other macros format Display values or write stderr/process pipes. phase_profile.rs::emit_record writes stderr only.

Python fixture writes are included; four generated Python source `.append` matches (cadquery_realization, print_preparation_kernel, review_realization, deposited_geometry) are list construction, explicitly excluded.

## Semantic deliverables

- classified_inventory.json: every candidate file, exact sites, initial semantic class, binding/path/declaration evidence and state-standing field.
- durable_resources.json / durable_roster.md: 26 named durable resources with exact module roster, operation partition, one accountable package, entry binding, provisioning, invariants and current-state standing. Mixed-source operations are split (OCI launch record M4 vs exclusive lock M2). Other direct-CAS/hold resources join the parent inventory.
- local_effect_review.json / .md: all 61 files classified as local/mixed retained effect, source-reviewed by shared external-process/platform/materialization contract. Calls out durable exit spool, SOL marker/evidence, and local installed-tree attestations that generic 'local' would hide.
- compiler_effect_review.json / .md: all 45 compiler/seed candidate files (including native_product_cache). Explicit persistent resources: native product cache, cached native workspace and publication lock, census run history, rustfmt disk cache, regeneration rollback journal, recorded fixture store, and C2 supervised command-ID adapter. Compiler emission, kernel cgroup files, socket operations, stdio, isolated tests and collection methods have concrete exclusions.
- /tmp/evidence-effect-dispositions.md (parallel owner): supersedes generic preliminary local_output_adapter labels for the original 91 report/receipt/output modules.
- /tmp/ios-persistence-inventory.md (parallel owner): Swift client persistence and fixture C/JS.

## Limits and standing

This is a pinned source boundary inventory and staffing map, not semantic execution coverage. Arbitrarily computed argv/shell commands, higher-order carriers and path selectors can be invisible to any finite token set; transport/provider source definitions and parent effect inventories must close those paths. No static count proves a live store empty or unused. Dynamic CLI/source invocation can select exported functions even when ordinary call-reference search finds no caller. In particular deploy_transition mutation functions have no repository symbol uses outside definitions/prose at this pin: that is a selected-root gap, not proof that old files do not exist. Megarac KVM start is explicitly unavailable in current source; historical journal evidence remains meaningful.

All deployed population fields are UNOBSERVED. Source desired paths, enrollment lists, historical receipt fixtures and prose saying 'pre-provisioned' are recorded as their actual evidence kind; they are never promoted to current machine measurements. Final semantic resource/interface choices remain review judgments. Parent supplemental inventories are necessary joins, not optional examples.


## Durable resource mappings

Original: [raw/effect-boundary/durable_roster.md](raw/effect-boundary/durable_roster.md).

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


## Mixed local/platform effects

Original: [raw/effect-boundary/local_effect_review.md](raw/effect-boundary/local_effect_review.md).

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


## Compiler and bootstrap effects

Original: [raw/effect-boundary/compiler_effect_review.md](raw/effect-boundary/compiler_effect_review.md).

# Compiler/seed effect boundary

Every matching source file is listed. This separates durable compiler state from local emission/control/test effects; generated status alone is never an exclusion.

`src/v1/05_emit_rust.dag` — Compiler-authored Rust file-transport backend fragments (and stdout/list operations); retain at extdeps realization/emit boundary, not standalone durable domain store.

`src/v1/compiler_tests_rust.dag` — No production filesystem mutation established after bounded cfg(test) and source-body review: fixture construction, pure collection append/truncate, diagnostic/prose or generated expected-source strings. Retain controls/exclude false positives.

`src/v1/runtime_rust.dag` — Compiler-authored Rust file-transport backend fragments (and stdout/list operations); retain at extdeps realization/emit boundary, not standalone durable domain store.

`src/v1/stage0/src/behavioral_receipt_host.rs` — Driver workspace source writes and mirror mutation/restore for behavioral differential; retain compiler validation realization.

`src/v1/stage0/src/bin/bootstrap_witness.rs` — Bootstrap stage source/binary trees and control fixtures; CI timing append is explicit measurement output, not shared application state.

`src/v1/stage0/src/bin/carrier_realization_census.rs` — Explicit CLI census receipt path; retain verification output.

`src/v1/stage0/src/bin/claim_executor.rs` — No production filesystem mutation established after bounded cfg(test) and source-body review: fixture construction, pure collection append/truncate, diagnostic/prose or generated expected-source strings. Retain controls/exclude false positives.

`src/v1/stage0/src/bin/generic_identity_census.rs` — Explicit CLI census receipt path; retain verification output.

`src/v1/stage0/src/bin/interp_recorded_fixture_witness.rs` — Controls for fixture provider, mutate isolated fixture roots and cleanup; retain tests, distinct from recorded_fixture.rs production provider.

`src/v1/stage0/src/bin/parse_witness.rs` — No production filesystem mutation established after bounded cfg(test) and source-body review: fixture construction, pure collection append/truncate, diagnostic/prose or generated expected-source strings. Retain controls/exclude false positives.

`src/v1/stage0/src/cli_run.rs` — Local source overlay, compiler measurement/bootstrap invocation output, Unix socket unlink/chmod; stdout/socket writes and Vec append/truncate are not storage. Process workspace root/source-root/serve command provide binding.

`src/v1/stage0/src/cli_run/checker_dependency.rs` — Explicit checker input record output path, not inferred database root; retain compiler observation artifact interface.

`src/v1/stage0/src/cli_run/demand_acquire.rs` — No production filesystem mutation established after bounded cfg(test) and source-body review: fixture construction, pure collection append/truncate, diagnostic/prose or generated expected-source strings. Retain controls/exclude false positives.

`src/v1/stage0/src/cli_run/derived_row_roster.rs` — No production filesystem mutation established after bounded cfg(test) and source-body review: fixture construction, pure collection append/truncate, diagnostic/prose or generated expected-source strings. Retain controls/exclude false positives.

`src/v1/stage0/src/cli_run/emit_host.rs` — No production filesystem mutation established after bounded cfg(test) and source-body review: fixture construction, pure collection append/truncate, diagnostic/prose or generated expected-source strings. Retain controls/exclude false positives.

`src/v1/stage0/src/cli_run/emitted_crate_workspace_host.rs` — Emitted crate materialization and scoped root removal; explicit compiler artifact adapter.

`src/v1/stage0/src/cli_run/entry_resolve.rs` — No production filesystem mutation established after bounded cfg(test) and source-body review: fixture construction, pure collection append/truncate, diagnostic/prose or generated expected-source strings. Retain controls/exclude false positives.

`src/v1/stage0/src/cli_run/native_lane_runner.rs` — M4 census-scoped incomplete/completed run records at workspace/target/census-runs; all other crate/source/native control fixtures and pipe/stdout writes stay local. See durable_resources.

`src/v1/stage0/src/cli_run/native_product_cache.rs` — M3 native-product cache and append-only diagnostic receipt; see durable_resources, not excluded by Rust placement.

`src/v1/stage0/src/cli_run/provider_acquire.rs` — No production filesystem mutation established after bounded cfg(test) and source-body review: fixture construction, pure collection append/truncate, diagnostic/prose or generated expected-source strings. Retain controls/exclude false positives.

`src/v1/stage0/src/cli_run/reach_base_standings.rs` — No production filesystem mutation established after bounded cfg(test) and source-body review: fixture construction, pure collection append/truncate, diagnostic/prose or generated expected-source strings. Retain controls/exclude false positives.

`src/v1/stage0/src/cli_run/required_ci_measurement.rs` — Required CI measurement receipt output path; retain verification artifact interface.

`src/v1/stage0/src/cli_run/rostered_row_join.rs` — No production filesystem mutation established after bounded cfg(test) and source-body review: fixture construction, pure collection append/truncate, diagnostic/prose or generated expected-source strings. Retain controls/exclude false positives.

`src/v1/stage0/src/cli_run/user_work_scope.rs` — Pure String path truncation in observed_cgroup_bound_in; test-only fixture writes remain isolated controls, no production filesystem write at this hit.

`src/v1/stage0/src/codex_app_server_stdio_session.rs` — C2 durable command ID pre-submission write; direct stdio session adapter shares supervisor identity resource. Pipe handles are not filesystem persistence.

`src/v1/stage0/src/compiler_tests.rs` — No production filesystem mutation established after bounded cfg(test) and source-body review: fixture construction, pure collection append/truncate, diagnostic/prose or generated expected-source strings. Retain controls/exclude false positives.

`src/v1/stage0/src/cssl_seed_linked_closure_assembly.rs` — No production filesystem mutation established after bounded cfg(test) and source-body review: fixture construction, pure collection append/truncate, diagnostic/prose or generated expected-source strings. Retain controls/exclude false positives.

`src/v1/stage0/src/emitted_closure_compile_host.rs` — Exclusive temporary probe root, mutation/restore controls and not-selected identity receipt; retain verification/compiler evidence adapter.

`src/v1/stage0/src/evaluation_budget_consequence_falsifier_host.rs` — HTTP socket write plus controlled worktree/source perturbation/restore; retain verification effect.

`src/v1/stage0/src/gunbc_file_transport_generated.rs` — Filesystem backend projection from extdeps.filesystem.rust_realization: staged fsync + hard-link create-only publication and cleanup; retain backend, not domain adopter.

`src/v1/stage0/src/main.rs` — Compiler output file emission to explicit output_dir; stdout return channel is not storage; retain command output API.

`src/v1/stage0/src/memory_governor.rs` — Kernel cgroup interface /sys/fs/cgroup: mkdir/remove/write memory.max/cgroup.procs. Retain ephemeral process-control effect; not durable application data.

`src/v1/stage0/src/namespace_baseline.rs` — Revision source closure archive/extraction and cleanup under scoped temporary checkout roots; retain compiler input realization.

`src/v1/stage0/src/partition_crate_boundary_host.rs` — Materializes emitted crate files for boundary validation; retain compiler verification artifact.

`src/v1/stage0/src/process_group.rs` — No production filesystem mutation established after bounded cfg(test) and source-body review: fixture construction, pure collection append/truncate, diagnostic/prose or generated expected-source strings. Retain controls/exclude false positives.

`src/v1/stage0/src/recorded_fixture.rs` — M4 recorded fixtures from claim_batch --record --fixture-store path; persistent operation/input/response/freshness store. See durable_resources.

`src/v1/stage0/src/required_regen_host.rs` — M3 formatter cache + M4 convergence rollback journal, backups and manifest; emitted/normalized sources and installation remain local. See durable_resources.

`src/v1/stage0/src/target_invocation_host.rs` — Creates source fixtures for census/cost controls in selected scratch root; retain verification fixture effects.

`src/v1/stage0/src/v1_compiler_emit_rust.rs` — Compiler-authored Rust file-transport backend fragments (and stdout/list operations); retain at extdeps realization/emit boundary, not standalone durable domain store.

`src/v1/stage0/src/v1_compiler_infer.rs` — No production filesystem mutation established after bounded cfg(test) and source-body review: fixture construction, pure collection append/truncate, diagnostic/prose or generated expected-source strings. Retain controls/exclude false positives.

`src/v1/stage0/src/v1_compiler_infer_method.rs` — No production filesystem mutation established after bounded cfg(test) and source-body review: fixture construction, pure collection append/truncate, diagnostic/prose or generated expected-source strings. Retain controls/exclude false positives.

`src/v1/stage0/src/v1_compiler_runtime_rust.rs` — Compiler-authored Rust file-transport backend fragments (and stdout/list operations); retain at extdeps realization/emit boundary, not standalone durable domain store.

`src/v1/stage0/src/v1_interpreter.rs` — M3 native compile workspace cache/publication lock/ready marker; extdeps file transport backend retains Write/Delete; shell detached log files retain supplied output adapter; Vec append/truncate excluded. See durable_resources.

`src/v1/stage0/src/v1_rt.rs` — Filesystem backend projection of same extdeps authority plus pure list.append false positives; retain backend.

`src/v2/extdeps/languages/rust_fixtures.dag` — No production filesystem mutation established after bounded cfg(test) and source-body review: fixture construction, pure collection append/truncate, diagnostic/prose or generated expected-source strings. Retain controls/exclude false positives.


## Receipt/output semantic corrections

Original: [raw/semantic/evidence-effect-dispositions.md](raw/semantic/evidence-effect-dispositions.md).

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
