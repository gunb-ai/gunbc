# Host-mutating effects reachable from `srv1_lab_dashboard_apply_wet` (static trace, wt-r2 @ 21f022640a71)

Entry: `gunbc.roadmap.roadmap_dashboard_instance_apply::srv1_lab_dashboard_apply_wet`
-> `dashboard_instance_apply_wet(instance: srv1_lab_dashboard_instance(), production: ProductionPeer{srv1_live_dashboard_instance()})`
-> `dashboard_instance_apply` -> preflight -> (`dashboard_apply_bootstrap_then_escalate` | `dashboard_instance_apply_after_preflight`).

## Instance facts that select the routes (all from `gunbc.roadmap.roadmap_dashboard_instance`)

| Fact | Value | Symbol |
|---|---|---|
| instance_id | `srv1-lab-codex-feedback-v0` | `srv1_dashboard_lab_id` |
| instance_root | `/opt/gunbc-lab/codex-provider-feedback-v0` | `srv1_dashboard_lab_root` = `gunbc.persistent_instance_roots::materialization_store_persistent_srv1_lab_scope` |
| provision parent | `/opt/gunbc-lab` | `srv1_dashboard_lab_provision_parent` |
| repo_root | `<root>/source` | `isolated_dashboard_instance_admitting` |
| serve_binary | `<root>/bin/gunbc` | same |
| attempt-state root | `<root>/attempt-state`; attempts `<root>/attempts` | `DispatchStateInstanceOwned` in `isolated_dashboard_instance_admitting` |
| source_seed | `DashboardSeededByConvergence { serve_binary_origin: DeclaredBuiltBinary { path: /home/briansrls/runs/ld449-r2work/target/release/gunbc }, apply_workdir: "/" }` | `srv1_lab_dashboard_instance`, `srv1_lab_r2_witnessed_serve_binary`, `dashboard_apply_neutral_workdir` |
| admitted / converged ref | `refs/gunbc/srv1-lab-admitted` (remote `.`) | `srv1_lab_admitted_revision_source`, `dashboard_instance_converged_ref` |
| worker | `WorkerDedicatedAccount` (worker uid `gunbc-worker`, shared group `gunbc-attempt`) | `gunbc.fleet.fleet_posix_accounts::fleet_posix_worker_user`, `fleet_posix_worker_shared_group_name` |
| directory_owner | `briansrls:briansrls` | `linux_service_directory_owner` -> `gunbc.managed_directory::srv1_dashboard_service_principal` -> `fleet_posix_operator_user` |
| actuation | `DashboardActuating` -> owned-directory policy `EnsureMayApply` | `dashboard_directory_policy` |
| transport | actuator_host `srv1` == host_identity `srv1` -> **`DashboardLocalProvision`** | `dashboard_provision_transport`; `gunbc.host.host_layout::srv1_host_layout` |
| legacy-writer fence | `LegacyWriterUnits { systemctl: /usr/bin/systemctl, units: [gunbc-srv1-lab-codex-feedback-v0.service @ /etc/systemd/system/gunbc-srv1-lab-codex-feedback-v0.service], rollback_store: /etc/systemd/system }` | `legacy_writer_fences`, `dashboard_instance_legacy_writer_fence` |
| toolchain | `srv1_linux_toolchain` (`install=/usr/bin/install`, `path_test=/usr/bin/test`, `path_stat=/usr/bin/stat`, `path_list=/usr/bin/ls`) | `srv1_linux_toolchain` |

**Principal.** Because the transport is local, every "target" exec (`dashboard_target_exec`, `dashboard_target_probe`, `dashboard_target_run`) runs through `gunbc.roadmap.roadmap_dashboard_instance_observe::dashboard_target_ran_of_local` -> `gunbc.WitnessBin.Run(workdir: "/", bin_path, args)` (the stdin leg through `shell.Exec.RunArgvStdin`) **as the uid of the process running `gunbc run`**, presumably the operator `briansrls`. No ssh hop. The only elevation anywhere in the closure is `gunbc.roadmap.roadmap_dashboard_instance_apply::dashboard_run_privileged`, which execs `executor_privileged_operation_elevated(op)` = `/usr/bin/sudo -n <argv>` (root; `gunbc.executor_privileged_operation::executor_privileged_operation_principal` returns `RunsAsRoot` for every op that can reach it; `extdeps.sudo.elevation::sudo_elevate`).

## Plan shape for this instance

`dashboard_instance_provision_plan` (seed = convergence, worker = dedicated), in order:

1. `DashboardRetireLegacyWriters`
2. `DashboardMoveAttemptStateEntries` x 2 (`attempt_state_move_rows` = `attempt_identity_slots.move` {member `attempt-identities`, prefix `attempt-identity-`, HoldPayload}, `dogfood_start_slots.move` {member `dogfood-start-receipts`, prefix `dogfood-start-receipt.`, CarryBytes})
3. (controller_source_ops = [] for the convergence seed)
4. `DashboardEnsureServeBinary` (DeclaredBuiltBinary), `DashboardEnsureOwnershipMarker`
5. `DashboardResetConvergedSourceRepo`
6. `DashboardEnsureGitSharedRepositoryPosture`, `DashboardMigrateGitWorkerSubtree { group: gunbc-attempt }`
7. `DashboardVerifyProviderRuntime`, `DashboardVerifyProviderAuth` (only if `dashboard_instance_provider_executable` is Present)
8. `DashboardAssertLegacyWriterExcluded`

Before the plan, `dashboard_apply_directories_then_plan` runs `dashboard_apply_owned_directories` over `gunbc.roadmap.dashboard_instance_directories::dashboard_instance_directory_demands`. The plan runs only when the reconcile returns `DashboardInstanceApply` (`dashboard_instance_reconcile`). If it returns Converged, nothing below runs. If preflight returns `DashboardApplyPreflightBootstrapPending` (provider not authenticated), only the directories plus `DashboardEnsureServeBinary` run (`dashboard_bootstrap_only_ops`). The fold stops at the first refusing op (`dashboard_apply_plan`).

Not in this plan for this instance (unreachable here): `DashboardHydrateControllerHistory`, `DashboardPrepareGitBundle`, `DashboardStageGitBundle`, `DashboardEnsureLocalOrigin`, `DashboardEnsureSourceRepo` (GitBundle seed only); `dashboard_build_serve_binary_in_place` (BuildInPlaceFromAdmittedRevision only).

## Effect table

Principal key: **U** = unprivileged local exec as the gunbc-run user (WitnessBin.Run, workdir `/`); **R** = `/usr/bin/sudo -n` -> root; **C** = controller-side shell primitive in the gunbc process.

| # | Op / leg | Kind | Program + argv | Principal | Destination(s) / mode / owner | Constructed at | Fires when |
|---|---|---|---|---|---|---|---|
| 0a | apply after preflight | mkdir (temp) | `shell.Mktemp.Dir()` | C | a fresh controller temp dir | `dashboard_instance_apply_after_preflight` | reconcile = Apply (full route only; not on the bootstrap route) |
| 0b | same | remove (recursive) | `shell.Remove.RecursiveForce(path: temp.path)` | C | only that mktemp dir (bundle and verify.git are never created for this seed) | same | always after the plan fold on the Apply route |
| 1 | ensure-owned-directory (each demand) | mkdir/chmod/chown/chgrp via install | `/usr/bin/install -d -m 0<octal> -o <user> -g <group> <path>` (`owned_directory_install_args` -> `extdeps.tools.gnu_coreutils::coreutils_install_directory_owned_arguments`) | U | see the directory list below | `dashboard_owned_directory_install` (passed as the `run:` lambda to `gunbc.owned_directory::owned_directory_ensure_observed`) | per demand only when `owned_directory_decide(EnsureMayApply)` says Apply (absent, or wrong mode or group on a directory the same user owns) and the act-time handover check finds no ownership change; a foreign owner or a non-directory refuses (`owned_directory_plan`) |
| 2a | retire-legacy-writers | systemctl stop | `/usr/bin/sudo -n /usr/bin/systemctl stop gunbc-srv1-lab-codex-feedback-v0.service` | R | unit | `roadmap_attempt_state_layout_move::legacy_writer_retirement_operations` -> `SystemdStopUnit`; argv in `executor_privileged_operation_argv` | only when `legacy_writer_retirement` = `LegacyWriterRetireEtcFragment`: unit read NOT excluded, LoadState=loaded, FragmentPath == the declared /etc path, ExecStart does not run the ingress binding (`http_ingress_execstart_runs_binding`) |
| 2b | same | rename (root) | `/usr/bin/sudo -n /usr/bin/mv -n -- /etc/systemd/system/gunbc-srv1-lab-codex-feedback-v0.service /etc/systemd/system/gunbc-srv1-lab-codex-feedback-v0.service.retired-by-gunbc-apply` | R | retained path = `legacy_writer_retained_path` (suffix `.retired-by-gunbc-apply`, not a unit type) | `RetireUnitFragment` | after 2a succeeds |
| 2c | same | systemctl daemon-reload | `/usr/bin/sudo -n /usr/bin/systemctl daemon-reload` | R | manager | `SystemdDaemonReload` | after 2b succeeds |
| 2d | same, rollback | rename (root) | `/usr/bin/sudo -n /usr/bin/mv -n -- <...>.service.retired-by-gunbc-apply /etc/systemd/system/gunbc-srv1-lab-codex-feedback-v0.service` | R | puts the fragment back | `legacy_writer_restore_operations` | only if the act failed or the post-act reading is not excluded (`dashboard_retire_legacy_writer_act`) |
| 2e | same, rollback | systemctl daemon-reload | `/usr/bin/sudo -n /usr/bin/systemctl daemon-reload` | R | manager | same | after 2d. **No start or restart on rollback**: a restored unit stays stopped |
| 3a | move-attempt-state-entries | create-new write: mktemp | `mktemp -- <member>/.gunbc-create-XXXXXX` (PATH-resolved `mktemp`) | U | staging file (0600 per coreutils) in `<root>/attempt-state/<member>/` | `roadmap_dashboard_target_write::dashboard_target_write_create_new` | per legacy entry, only on `MoveApply`, only while the legacy writer reads excluded, and only when the slot is not already present |
| 3b | same | write via stdin | `dd of=<stage> conv=fsync status=none` with stdin = slot content (`shell.Exec.RunArgvStdin`) | U | staging file | same (`dd_fsync_into_staging_command`) | after 3a |
| 3c | same | hard link (create-only) | `ln -T -- <stage> <member>/<slot>` | U | slot `<root>/attempt-state/attempt-identities/<cas slot name>` or `<root>/attempt-state/dogfood-start-receipts/<legacy name>`, inherits the stage's 0600 | same (`ln_hard_command`) | after 3b |
| 3d | same | remove staging | `rm -f <stage>` | U | staging file | same (`rm_force_command`) | always after 3a succeeds |
| 3e | same | remove legacy entry | `/bin/rm -- <root>/attempt-state/<legacy flat name>` | U | the flat legacy file | `dashboard_apply_provision_op` (`DashboardMoveAttemptStateEntries` arm) | after the slot reads back equal (text for HoldPayload; sha256 for CarryBytes) and the exclusion is re-observed |
| 4a | ensure-serve-binary (DeclaredBuiltBinary) | mkdir | `/usr/bin/mkdir -p <dirname of /opt/gunbc-lab/codex-provider-feedback-v0/bin/gunbc>` | U | `<root>/bin` (mode from umask; the owned-directory leg normally created it already) | `dashboard_ensure_declared_serve_binary` | after `/usr/bin/test -f /home/briansrls/runs/ld449-r2work/target/release/gunbc` succeeds |
| 4b | same | copy | `/usr/bin/cp /home/briansrls/runs/ld449-r2work/target/release/gunbc /opt/gunbc-lab/codex-provider-feedback-v0/bin/gunbc.next` | U (via `dashboard_local_exec`, workdir `/`) | `<root>/bin/gunbc.next` | `dashboard_copy_to_target_atomically`, `DashboardLocalProvision` arm | after 4a |
| 4c | same | rename | `/usr/bin/mv <root>/bin/gunbc.next <root>/bin/gunbc` | U | **replaces the live serve binary**; no `.prev` is kept and no digest or `--version` readback happens on this arm | same | after 4b |
| 4d | same | chmod | `/bin/chmod 0755 <root>/bin/gunbc` | U | mode 0755 | `dashboard_ensure_declared_serve_binary` (`extdeps.tools.chmod::chmod_binary_path`) | after 4c |
| 5 | ensure-ownership-marker | file create / mtime | `/usr/bin/touch /opt/gunbc-lab/codex-provider-feedback-v0/.gunbc-dashboard-instance-srv1-lab-codex-feedback-v0` | U | marker file (umask mode) | `dashboard_apply_provision_op` (`DashboardEnsureOwnershipMarker` arm); path from `dashboard_ownership_marker_in` | every Apply run that gets this far |
| 6 | reset-converged-source-repo | git worktree/index rewrite | `/usr/bin/git -C <root>/source reset --hard refs/gunbc/srv1-lab-admitted` | U | the working tree of `<root>/source` | `dashboard_reset_converged_source_repo` (`extdeps.git::shape_git_reset_hard_argv`) | only if `<root>/source/.git` exists, `git status --porcelain` is empty, and `rev-parse --verify --quiet` finds the ref (exit 0). No clone or fetch on this arm |
| 7 | ensure-git-shared-repository-posture | git config write x3 | `env GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_NOSYSTEM=1 git -C <root>/source config --local --replace-all core.sharedRepository group`; same with `gc.auto 0`; same with `maintenance.auto false` (PATH-resolved `env`, `git`) | U | `<root>/source/.git/config` | `roadmap_dashboard_instance::dashboard_ensure_git_shared_repository_posture` -> `extdeps.git::git_shared_repository_group_config_command` / `git_gc_auto_disabled_config_command` / `git_maintenance_auto_disabled_config_command`; argv projected by `dashboard_git_owned_step_argv` | every Apply run that gets this far; readback by `git config --local --get` |
| 8a | migrate-git-worker-subtree | chgrp (recursive via find) | `find <root>/source/<child> -maxdepth 32 [( -path <t>/pack -o -path <t>/info ) -prune -o] -not -type l -user briansrls -not -group gunbc-attempt -exec chgrp --no-dereference gunbc-attempt {} +` for child in `.git/objects` (prunes pack and info), `.git/refs`, `.git/logs` | U | group -> `gunbc-attempt` on entries owned by briansrls | `dashboard_instance_worker_git_subtree_migration` -> `dashboard_git_worker_subtree_chgrp_command` -> `extdeps.tools.findutils::find_subtree_regroup_command` | after 7 succeeds |
| 8b | same | chmod (recursive via find) | `find <...same walk...> -user briansrls` followed by: for `.git/logs` `-exec chmod g+w {} +` on everything, otherwise `( -type d -exec chmod g+w {} + )`; then `( -type d -exec chmod g+s {} + )`; for `.git/objects` also `( -type d -exec chmod a+t {} + )` | U | g+w, g+s, +t (objects only) | `dashboard_git_worker_subtree_chmod_command` -> `find_subtree_group_writable_command` | after all three 8a commands |
| 9 | verify-provider-runtime / auth | external exec (intended read-only) | `/usr/bin/env CODEX_HOME=<provider state root> <codex executable> --version`, then `... login status` | U | none intended. The codex CLI may write under CODEX_HOME; that is outside the model | `dashboard_apply_provision_op` arms; `extdeps.llm.cli::shape_codex_*_env_args_for_program` | if the provider executable is Present |
| 10 | assert-legacy-writer-excluded | read only | `/usr/bin/systemctl show --property=LoadState,ActiveState,FragmentPath,ExecStart gunbc-srv1-lab-codex-feedback-v0.service` | U | none | `dashboard_legacy_writer_reading` | always as the last op |

Unreachable for this instance, listed for completeness (`BuildInPlaceFromAdmittedRevision` arm of `dashboard_ensure_serve_binary`):

| Leg | Program + argv | Symbol |
|---|---|---|
| build | `/usr/bin/systemd-run --user --scope --quiet -pMemoryMax=17179869184 -pMemorySwapMax=0 -pRuntimeMaxSec=<n> env CARGO_TARGET_DIR=<root>/serve-build-target /home/briansrls/.cargo/bin/cargo build --release --manifest-path <root>/source/Cargo.toml -p v1-compiler --bin gunbc` | `dashboard_build_serve_binary_in_place`, `gunbc.live_deploy.serve_binary_build::serve_binary_build_argv` |
| install | `/usr/bin/mkdir -p`, `/usr/bin/cp <target>/release/gunbc <bin>.new`, `/bin/chmod 0755 <bin>.new`, `/usr/bin/ln -f <bin> <bin>.prev`, `/usr/bin/mv -f <bin>.new <bin>`, restore `/usr/bin/mv -f <bin>.prev <bin>` | `dashboard_install_built_serve_binary`, `dashboard_rename_verified_serve_binary`, `dashboard_restore_previous_after_refused_install` |

Also unreachable here (GitBundle seed only): `/usr/bin/git fetch --unshallow`, `bundle create`, `init --bare`, bare fetch, `update-ref`, `clone`, `fetch origin`, and `/usr/bin/scp` (ssh transport only).

### Owned-directory demands (row 1)

These come from `gunbc.roadmap.roadmap_dashboard_instance::dashboard_instance_own_directory_demands`. Modes come from `gunbc.managed_directory::declared_directory_mode`: OwnerOnly 0700, AllGroupTraverse 0750, AllEveryoneTraverse 0755, GroupWritableSetgid 2770, GroupWritableSetgidSticky 3770.

- `/opt/gunbc-lab`, briansrls:briansrls 0755 (`dashboard_instance_provision_parent_directory`)
- `/opt/gunbc-lab/codex-provider-feedback-v0`, briansrls:gunbc-attempt 0750 (`dashboard_instance_root_directory`)
- `<root>/attempts` and `<root>/attempt-state`, briansrls:gunbc-attempt 2770, group adoption `SameUserRegroupToDeclaredGroup`
- provider state root `/home/briansrls/<codex roadmap account dir>`, briansrls:gunbc-attempt 2770 (`dashboard_instance_provider_state_directory`)
- `<root>/bin`, briansrls:briansrls 0755 (`dashboard_instance_serve_binary_directory`)
- `<root>/source/.git/objects` 3770, and `.git/refs` and `.git/logs` 2770, all briansrls:gunbc-attempt (`dashboard_instance_worker_git_directories`)
- no inbox (convergence seed)
- plus the store directories listed in `gunbc.roadmap.dashboard_instance_directories::dashboard_instance_directory_demands`: session, login-transaction, JWKS cache, profile, issue-view, event-carrier, compute root and capacity, credential-acquisition slots, cursor attempt runtime, worktree-claim slots, attempt-identity, belt projects, dogfood-start receipts, session-attempt slots, fabric storage demands, and compute store directories. **Paths and modes not expanded; see UNRESOLVED.**

Note: `install -d` run unprivileged can create a directory or re-mode and re-group a same-owner directory (if the user is in `gunbc-attempt`). It cannot chown to another user, and the provider refuses a foreign owner before running.

## Answers

### 1. Every sudo / elevated invocation

The only site is `roadmap_dashboard_instance_apply::dashboard_run_privileged` -> `gunbc.executor_privileged_operation::executor_privileged_operation_elevated` -> `extdeps.sudo.elevation::sudo_elevate` (`/usr/bin/sudo -n`, run as root, no `-u`). It is called only from `dashboard_retire_legacy_writer_act`, which is fed only by `legacy_writer_retirement_operations` and `legacy_writer_restore_operations` (`gunbc.roadmap.roadmap_attempt_state_layout_move`). Those produce exactly `SystemdStopUnit`, `RetireUnitFragment` and `SystemdDaemonReload`. The full set of reachable elevated argv:

1. `/usr/bin/sudo -n /usr/bin/systemctl stop gunbc-srv1-lab-codex-feedback-v0.service`
2. `/usr/bin/sudo -n /usr/bin/mv -n -- /etc/systemd/system/gunbc-srv1-lab-codex-feedback-v0.service /etc/systemd/system/gunbc-srv1-lab-codex-feedback-v0.service.retired-by-gunbc-apply`
3. `/usr/bin/sudo -n /usr/bin/systemctl daemon-reload`
4. Rollback only: `/usr/bin/sudo -n /usr/bin/mv -n -- <...>.retired-by-gunbc-apply /etc/systemd/system/gunbc-srv1-lab-codex-feedback-v0.service`, then (3) again.

Binary paths come from `gunbc.fleet.fleet_posix_accounts::ci_runner_sudo_binary_path` (`fleet_systemctl_binary_path` = `/usr/bin/systemctl`, `fleet_mv_binary_path` = `/usr/bin/mv`). Units and paths come only from the `legacy_writer_fences` row. The privileged-op constructors with other verbs (`SystemdEnableUnit`, `SystemdStartUnit`, `SystemdMaskNowUnit`, `InstallFile`, `EnsureOwnedDirectory`, `RemoveDirectoryTree`, and so on) are reached only as match arms inside `executor_privileged_operation_argv`. No function in the traced closure constructs them. `legacy_writer_grant_operations` (the sudoers grant) is not in the closure. Nothing else in the closure uses sudo: `srv1_software_inhabitance_receipt` holds a sudo-list probe as data and the apply does not execute it.

### 2. systemctl verbs and units

- `show --property=LoadState,ActiveState,FragmentPath,ExecStart gunbc-srv1-lab-codex-feedback-v0.service`, unprivileged, repeated: before and after retirement, at every move entry boundary, before each rm, and as the final gate (`dashboard_legacy_writer_reading`, `extdeps.systemd.systemctl::systemctl_show_properties_command`).
- `stop gunbc-srv1-lab-codex-feedback-v0.service` (sudo), conditional as in row 2a.
- `daemon-reload` (sudo), after the rename and again after any rollback.
- No `start`, `enable`, `disable`, `mask`, `unmask`, `restart`, `reload` or `set-property` is reachable.

The only unit named anywhere in an executed argv is `gunbc-srv1-lab-codex-feedback-v0.service`. The strings `-belt-exit.path`, `-belt-discovery.path`, `-attempt-unit-drain.path`, `-publication.path` and `-belt-credential-recovery.timer` appear in no function of the traced closure; they occur only in comments. **No path in this closure enables or starts any unit.** The declared git-maintenance `.service`/`.timer` (`dashboard_git_maintenance_*`) is not installed or enabled by this apply ("THIS PR DECLARES the unit; it does not install it"), and none of those functions are reached.

Indirect effects outside the static model:
- `systemctl stop` propagates to units bound to the serve unit (Requires/BindsTo/PartOf).
- `daemon-reload` re-reads every unit file. It starts no inactive unit by itself, but it makes any already-edited unit file on disk live.

Neither starts the five named .path/.timer units, but already-active .path/.timer units keep running and may trigger their own services independently of this apply.

### 3. File writes, installs and copies

- **Owned directories** via `/usr/bin/install -d` (list above).
- **Serve binary**: `cp` to `<root>/bin/gunbc.next`, `mv` onto `<root>/bin/gunbc`, then `/bin/chmod 0755`. The owner is the run user.
- **Ownership marker**: `touch <root>/.gunbc-dashboard-instance-srv1-lab-codex-feedback-v0`.
- **Attempt-state slots**:
  - mktemp staging (0600) under `<root>/attempt-state/{attempt-identities,dogfood-start-receipts}/.gunbc-create-*`
  - `dd` write, then hard link to the slot (0600)
  - `rm -f` staging, then `/bin/rm` of each flat legacy entry in `<root>/attempt-state`
- **Git** under `<root>/source`:
  - `.git/config` (3 keys)
  - working tree and index via `reset --hard`
  - group and mode bits via find, chgrp and chmod under `.git/{objects,refs,logs}`
- **/etc/systemd/system**: the serve fragment is renamed to `.retired-by-gunbc-apply` (root). Nothing is written into it, and there is no chown anywhere in an executed argv.
- **Controller temp dir**: `shell.Mktemp.Dir` / `shell.Remove.RecursiveForce`.
- **Incidental writes** by external programs, outside the model: `git status` and `rev-parse` can refresh `.git/index`, and the `codex` CLI may write under CODEX_HOME.

### 4. systemd-run transient units

None on this instance's route. The only `systemd-run` argv in the closure is `serve_binary_build_argv` (`--user --scope`, cargo build), reached only from `dashboard_build_serve_binary_in_place`. That runs only for `BuildInPlaceFromAdmittedRevision`, and the srv1 lab now declares `DeclaredBuiltBinary`, so it is unreachable here.

### 5. UNRESOLVED / not fully enumerated

- **U1. Store directory paths and modes are not expanded.** The demands contributed by `session_store_directory`, `login_transaction_store_directory`, `jwks_cache_directory`, `profile_store_directory`, `issue_view_store_directory`, `roadmap_event_carrier_directory`, `compute_root_directory`, `compute_capacity_directories`, `credential_acquisition_slots_directory`, `cursor_attempt_runtime_directory`, `worktree_claim_slots_directory`, `attempt_identity_directory`, `belt_projects_directory`, `dogfood_start_receipt_owned_directory`, `session_attempt_slots_directory`, `fabric_storage_directory_demands(prod_role_realization())` and `compute_store_owned_directories` were not expanded. All go through the same unprivileged `install -d` effect (row 1).
- **U2. The codex executable path** comes from `gunbc.host.host_layout::codex_bundled_executable_for_host("srv1")` (placement and release-link data), and the provider state root from `codex_account_state_root(/home/briansrls, codex_roadmap_account_selection())`. Neither concrete string was expanded.
- **U3. Higher-order effects.** `owned_directory_ensure_observed` receives `observe`/`run` lambdas (resolved to `dashboard_owned_directory_observe` and `dashboard_owned_directory_install`). `dashboard_git_owned_step_argv` projects `argv_words` of built ArgvCommands, which resolved to `env`/`git`/`find` as tabulated. The program word is data, PATH-resolved (`find`, `chgrp`, `chmod`, `env`, `git`, `mktemp`, `dd`, `ln`, `rm`), so which binary runs depends on the run user's PATH.
- **U4. Effect primitives were not traced into their realizations**: `gunbc.WitnessBin.Run`, `shell.Exec.RunArgvStdin`, `shell.Mktemp.Dir`, `shell.Remove.RecursiveForce`. Their host behaviour (environment, umask, cwd) is assumed from their names.
- **U5. Unknown principal.** Which uid runs `gunbc run` on srv1 is not in the source. The "U" rows run as that uid. The sudo rows need a NOPASSWD grant for exactly those argv; `-n` makes a missing grant a refusal, not a prompt.
- **U6. Reachability is name-based and bounded.** A name-resolving tracer (773 functions reached; imports plus same-module defs) was cross-checked by reading the dispatch and every effectful callee by hand. Method-call forms (`x.filter(...)`) and lambdas were followed by name only. This is not a completeness proof.
- **U7. Runtime-only conditions.** Which ops actually fire depends on host state (reconcile verdict, fence reading, entry listings), which is not knowable statically.

## Method

1. Read `srv1_lab_dashboard_apply_wet` down through `dashboard_instance_apply`, the preflight, `dashboard_instance_apply_after_preflight`, `dashboard_apply_directories_then_plan` and `dashboard_apply_provision_op`.
2. Read `dashboard_instance_provision_plan` and `dashboard_instance_reconcile` for the plan, and `srv1_lab_dashboard_instance` / `isolated_dashboard_instance_admitting` for the instance values.
3. Followed every arm into these modules: `roadmap_dashboard_instance_observe`, `roadmap_dashboard_target_write`, `roadmap_attempt_state_layout_move`, `executor_privileged_operation`, `extdeps.sudo.elevation`, `extdeps.tools.{findutils,chmod,chown,gnu_coreutils,env}`, `extdeps.git`, `gunbc.owned_directory`, `gunbc.live_deploy.serve_binary_build`.
4. Cross-checked with a name-resolving reachability script (scratchpad `tr/trace.py`, output `tr/out.txt` and `tr/seen.txt`) that greps reached bodies for effect primitives, absolute program literals, and the words sudo, systemctl, systemd-run, chmod and chgrp.

## Limits

- Static, read-only. Nothing was executed.
- This is a bounded trace, not a completeness claim (see U1 to U7).
- Values were read at wt-r2 HEAD 21f022640a71, which includes the LAB-ONLY `DeclaredBuiltBinary` deviation.
