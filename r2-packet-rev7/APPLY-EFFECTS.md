# Host-mutating effects reachable from `srv1_lab_dashboard_apply_wet` (static trace)

**Traced at:** r2 head **26fe4b8f350a99bd57a9bc342d4625b0cb96804b**, tree **b0ee022a6bf8** (`git rev-parse HEAD HEAD^{tree}` in the trace worktree). Every module read, and both tracer runs, used this tree.

How the apply's reachable set was checked against this tree:

- The name-resolving tracer was re-run from `srv1_lab_dashboard_apply_wet` at this head. It reaches 773 functions across 102 modules, none of them in `gunbc.roadmap.ld449_lab_render_probe`.
- Every effectful callee in the tables was hand-read at this head.

No earlier commit is cited as a trace head.

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

**Principal.** Because the transport is local, every "target" exec (`dashboard_target_exec`, `dashboard_target_probe`, `dashboard_target_run`) runs through `gunbc.roadmap.roadmap_dashboard_instance_observe::dashboard_target_ran_of_local` -> `gunbc.WitnessBin.Run(workdir: "/", bin_path, args)` (the stdin leg through `shell.Exec.RunArgvStdin`) **as the uid of the process running `gunbc run`**, presumably the operator `briansrls`. No ssh hop for the lab target. **Correction (see section P):** the production reads of srv1-live (`dashboard_production_read`, before and after the plan) go over `ssh srv1 -- ...`, because srv1-live declares `actuator_host: macbook`. The only elevation anywhere in the closure is `gunbc.roadmap.roadmap_dashboard_instance_apply::dashboard_run_privileged`, which execs `executor_privileged_operation_elevated(op)` = `/usr/bin/sudo -n <argv>` (root; `gunbc.executor_privileged_operation::executor_privileged_operation_principal` returns `RunsAsRoot` for every op that can reach it; `extdeps.sudo.elevation::sudo_elevate`).

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
| 0p | production read-back, before and after the plan (added at 26fe4b8f350a; see P) | ssh + process exec (read-only) | `ssh srv1 -- /usr/bin/git -C /opt/gunbc/gunbc rev-parse HEAD`; `ssh srv1 -- /usr/bin/curl -fsS http://127.0.0.1:8080/healthz` | run user via ssh (srv1-live transport is `DashboardSshProvision{srv1}` because its `actuator_host` is `macbook`) | none (read-only) | `dashboard_production_snapshot` via `dashboard_production_read`; `typed_argv_probe_over_ssh` | preflight, then once before and once after the plan in `dashboard_instance_apply_after_preflight` |
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
- provider state root `/home/briansrls/.codex@gunbc-roadmap`, briansrls:gunbc-attempt 2770 (`dashboard_instance_provider_state_directory`)
- `<root>/bin`, briansrls:briansrls 0755 (`dashboard_instance_serve_binary_directory`)
- `<root>/source/.git/objects` 3770, and `.git/refs` and `.git/logs` 2770, all briansrls:gunbc-attempt (`dashboard_instance_worker_git_directories`)
- no inbox (convergence seed)
- plus the store directories listed in `gunbc.roadmap.dashboard_instance_directories::dashboard_instance_directory_demands`: session, login-transaction, JWKS cache, profile, issue-view, event-carrier, compute root and capacity, credential-acquisition slots, cursor attempt runtime, worktree-claim slots, attempt-identity, belt projects, dogfood-start receipts, session-attempt slots, fabric storage demands, and compute store directories. Fully expanded with literal paths in **U1 RESOLVED** below (28 rows; fabric storage contributes none).

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

### 5. UNRESOLVED / not fully enumerated (updated at 26fe4b8f350a)

U1, U2 and U4 are **RESOLVED** in the sections below. Still open:

- **U3. PATH-dependent binaries.** Several program words are bare names resolved through PATH at run time (list in U4). The source fixes no PATH (see U4), so which binary runs depends on the environment of the `gunbc run` caller.
- **U2-residual. What the external codex binary writes.** The model passes only `CODEX_HOME`. Whether `codex --version` / `codex login status` writes under CODEX_HOME (logs, sessions, token refresh into `auth.json`) is upstream behaviour this repository does not model. The `current` symlink target is resolved at run time.
- **U4-residual. Interpreter route only.** U4 traces the seed interpreter route (`src/v1/stage0/src/v1_interpreter.rs` `dispatch_shell`). If the entry ran through an emitted native binary instead, its rendered shell transport was not inspected.
- **U5. Unknown principal.** The uid running `gunbc run` is not in the source.
- **U6. Bounded reachability.** The trace is name-based and bounded; this is not a completeness proof.
- **U7. Runtime-only conditions.** Which ops fire depends on host state.

## U1 RESOLVED: every owned-directory demand, expanded (srv1_lab_dashboard_instance, @26fe4b8f350a)

**Sources.**
- The list is `gunbc.roadmap.dashboard_instance_directories::dashboard_instance_directory_demands(realization: prod_role_realization(), instance)`. The ensure leg (`dashboard_apply_owned_directories`) and the readback (`dashboard_observe_owned_directories`) use this same list; the plan ops add no other directory list.
- The ensure leg covers both the full Apply route and the bootstrap route.

**Effect per row.**
- Each row becomes `/usr/bin/install -d -m <operand> -o <user> -g <group> <path>`, run unprivileged. The operand is `"0"` + `extdeps.access.posix::file_mode_octal`, via `extdeps.tools.gnu_coreutils::coreutils_directory_mode_operand`.
- **Install only when not satisfied.** The install runs only when `gunbc.owned_directory::owned_directory_satisfied` is false: the path is absent, or the owner, group or mode differs.
- **Refusal on a foreign owner or group.** The install also needs `owned_directory_plan` to find no foreign owner. Under `ForeignOwnershipRefuses`, any owner or group mismatch refuses. Under `SameUserRegroupToDeclaredGroup`, a different user refuses, while the same user with a different group or mode is re-grouped and re-moded by the install.
- **Missing ancestors.** GNU `install -d` creates missing ancestors as the run user (umask mode); only the leaf gets `-m/-o/-g`.

**Values.**
- R = `/opt/gunbc-lab/codex-provider-feedback-v0` (`gunbc.persistent_instance_roots::materialization_store_persistent_srv1_lab_scope`).
- Owner `briansrls` (`gunbc.fleet.fleet_posix_accounts::fleet_posix_operator_user` via `gunbc.managed_directory::srv1_dashboard_service_principal`).
- Group `gunbc-attempt` (`fleet_posix_worker_shared_group_name`).
- Mode operands come from `gunbc.managed_directory::declared_directory_mode`:

| Declared mode | Operand |
|---|---|
| OwnerOnly | `00700` |
| OwnerOnlySetgid | `02700` |
| AllGroupTraverse | `00750` |
| AllEveryoneTraverse | `00755` |
| GroupWritableSetgid | `02770` |
| GroupWritableSetgidSticky | `03770` |

Rows in fold order:

| # | Absolute path | -m | owner:group | Adoption | Source symbol |
|---|---|---|---|---|---|
| 1 | `/opt/gunbc-lab` | 00755 | briansrls:briansrls | ForeignOwnershipRefuses | `roadmap_dashboard_instance::dashboard_instance_provision_parent_directory` (`srv1_dashboard_lab_provision_parent`) |
| 2 | `R` | 00750 | briansrls:gunbc-attempt | ForeignOwnershipRefuses | `dashboard_instance_root_directory` |
| 3 | `R/attempts` | 02770 | briansrls:gunbc-attempt | SameUserRegroupToDeclaredGroup | `dashboard_instance_dispatch_worktree_directory` (worktree_child "attempts"; `dashboard_instance_worker_writable_mode` -> GroupWritableSetgid) |
| 4 | `R/attempt-state` | 02770 | briansrls:gunbc-attempt | SameUserRegroup | `dashboard_instance_dispatch_attempt_state_directory` (attempt_state_child "attempt-state") |
| 5 | `/home/briansrls/.codex@gunbc-roadmap` | 02770 | briansrls:gunbc-attempt | SameUserRegroup | `dashboard_instance_provider_state_directory`; path from `gunbc.host.host_layout::srv1_roadmap_codex_home` = `gunbc.provider_account::codex_account_state_root(home: "/home/briansrls", CodexLabelledAccount{"gunbc-roadmap"})` = `/home/briansrls` + "/" + `.codex` + `@` + `gunbc-roadmap` |
| 6 | `R/bin` | 00755 | briansrls:briansrls | ForeignOwnershipRefuses | `dashboard_instance_serve_binary_directory` (parent of `R/bin/gunbc`) |
| 7 | `R/source/.git/objects` | 03770 | briansrls:gunbc-attempt | SameUserRegroup | `dashboard_instance_worker_git_directories` (`dashboard_git_worker_subtree_declared_directory_mode`: objects is sticky) |
| 8 | `R/source/.git/refs` | 02770 | briansrls:gunbc-attempt | SameUserRegroup | same |
| 9 | `R/source/.git/logs` | 02770 | briansrls:gunbc-attempt | SameUserRegroup | same |
| 10 | `R/auth-sessions` | 00700 | briansrls:briansrls | ForeignOwnershipRefuses | `gunbc.auth.session_store_root::session_store_directory` (`session_store_slots` member "auth-sessions", OwnerOnly) |
| 11 | `R/auth-login-transactions` | 00700 | briansrls:briansrls | ForeignOwnershipRefuses | `gunbc.auth.login_transaction_store_root::login_transaction_store_directory` |
| 12 | `R/auth-jwks` | 00700 | briansrls:briansrls | ForeignOwnershipRefuses | `gunbc.auth.oidc_id_token_verification::jwks_cache_directory` (`instance_jwks_cache_slots`) |
| 13 | `R/auth-profiles` | 00700 | briansrls:briansrls | ForeignOwnershipRefuses | `gunbc.auth.profile_store_root::profile_store_directory` |
| 14 | `R/issue-view-preferences` | 00700 | briansrls:briansrls | ForeignOwnershipRefuses | `gunbc.roadmap.roadmap_view_preference_store::issue_view_store_directory` -> `dashboard_instance_private_directory` |
| 15 | `R/roadmap-events` | 00755 | briansrls:briansrls | ForeignOwnershipRefuses | `gunbc.roadmap.roadmap_event_carrier_directory::roadmap_event_carrier_directory` -> `dashboard_instance_shared_directory` (`roadmap_events_worktree_child`) |
| 16 | `R/compute` | 00755 | briansrls:briansrls | ForeignOwnershipRefuses | `gunbc.compute.host_capacity::compute_root_directory` (`compute_root_member`) |
| 17 | `R/credential-acquisition-slots` | 00700 | briansrls:briansrls | ForeignOwnershipRefuses | `gunbc.dispatch_credential_acquisition::credential_acquisition_slots_directory` |
| 18 | `R/cursor-attempt-runtime` | 00700 | briansrls:briansrls | ForeignOwnershipRefuses | `gunbc.dispatch_credential_acquisition::cursor_attempt_runtime_directory` (`cursor_attempt_runtime_dirname`) |
| 19 | `R/attempt-state/worktree-claims` | 00700 | briansrls:briansrls | ForeignOwnershipRefuses | `gunbc.roadmap.roadmap_attempt_worktree_claim_root::worktree_claim_slots_directory` |
| 20 | `R/attempt-state/attempt-identities` | 00700 | briansrls:briansrls | ForeignOwnershipRefuses | `gunbc.roadmap.roadmap_attempt_identity_store_root::attempt_identity_directory` (`attempt_identity_slots` mode OwnerOnly) |
| 21 | `R/attempt-state/projects` | **02700** | briansrls:**gunbc-attempt** | ForeignOwnershipRefuses | `gunbc.roadmap.roadmap_belt_receipt_store_root::belt_projects_directory` via `gunbc.owned_directory::owned_directory_inheriting_from_parent` (the parent attempt-state is setgid, so it inherits the group and `declared_directory_mode_inheriting_setgid(OwnerOnly)`) |
| 22 | `R/attempt-state/dogfood-start-receipts` | 00700 | briansrls:briansrls | ForeignOwnershipRefuses | `gunbc.roadmap.roadmap_dogfood_start_store_root::dogfood_start_receipt_owned_directory` |
| 23 | `R/attempt-state/session-attempts` | **02700** | briansrls:**gunbc-attempt** | ForeignOwnershipRefuses | `gunbc.roadmap.roadmap_session_attempt_slots_root::session_attempt_slots_directory` (inheriting, as row 21) |
| 24 | `R/compute/capacity` | 00755 | briansrls:briansrls | ForeignOwnershipRefuses | `gunbc.compute.host_capacity::compute_capacity_directories` (`compute_capacity_slots` = ["compute","capacity"]) |
| 25 | `R/compute/capacity/objects` | 00755 | briansrls:briansrls | ForeignOwnershipRefuses | same (`compute_capacity_area`; `gunbc.fabric.fabric_storage_file_store::fabric_store_area_paths`) |
| 26 | `R/compute/capacity/heads` | 00755 | briansrls:briansrls | ForeignOwnershipRefuses | same |
| -- | (fabric storage) | -- | -- | -- | `gunbc.fabric.fabric_storage_placement::fabric_storage_directory_demands` returns **[]**: `fabric_storage_placed_on` needs the prod-role holder's deployment to equal the lab's. `gunbc.deployment_risk::prod_role_selection` = `ProdRoleHeldBy{srv1_daily_workspace_deployment}` while the lab is `srv1_lab_codex_feedback_deployment` ("srv1-lab-codex-feedback"); a refused realization also yields [] |
| 27 | `R/materialization-store` | 00700 | briansrls:briansrls | ForeignOwnershipRefuses | `gunbc.compute.compute_store_owned_directories::compute_store_owned_directories` (`PersistentInstanceWorkspace{Srv1Lab}` -> `std.materialization_store_grant::materialization_store_root_path` = scope root + "/" + `materialization_store_workspace_leaf`) |
| 28 | `R/materialization-store-staging` | 00700 | briansrls:briansrls | ForeignOwnershipRefuses | same (`materialization_store_staging_root_path` = root + "-staging") |

Notes:

- **(a) Row 5 sits outside the instance root.** It is the operator home's codex account directory. If that directory exists as `briansrls:briansrls`, or with a mode other than 2770, the apply **re-groups it to gunbc-attempt and sets 02770**: same-user adoption, run unprivileged (requires membership in gunbc-attempt).
- **(b) Rows 7-9 can create a `.git` skeleton.** If `R/source` does not exist, they create `R/source/.git/{objects,refs,logs}`, and their ancestors `R/source` and `R/source/.git` at the default mode. `DashboardResetConvergedSourceRepo` would then find `.git` present and run `git status`, which refuses on a non-repository.
- **(c) Rows 19, 20 and 22 sit under the setgid `R/attempt-state` but are declared non-inheriting.** `install -g briansrls -m 00700` explicitly sets the group to briansrls and clears setgid on them, so they do not keep the inherited gunbc-attempt group. Rows 21 and 23 are declared inheriting and get gunbc-attempt.

## U2 RESOLVED: provider executable, state root, argv

Inputs:

- **Executable.** `dashboard_instance_provider_executable` = `layout.codex_bundled_executable` = `gunbc.host.host_layout::srv1_codex_bundled_executable()` = `codex_bundled_executable_for_host("srv1")`. The host is in {srv1..srv4} -> `srv1_provider_runtime_placement().runtime_release_root` = `gunbc.codex_runtime_paths::srv1_codex_runtime_release_root` = `/opt/gunbc/provider-runtime/codex`. Then `codex_runtime_current_link_path` appends `/current` (`codex_runtime_current_link_name`), and `gunbc.package_delivery::codex_linux_arm64_native_executable_rel` appends `node_modules/@openai/codex-linux-arm64/vendor/aarch64-unknown-linux-musl/bin/codex`. Result:
  **`/opt/gunbc/provider-runtime/codex/current/node_modules/@openai/codex-linux-arm64/vendor/aarch64-unknown-linux-musl/bin/codex`**
  - The arm64 target is hard-coded for every srv host.
  - The `current` symlink is resolved by the kernel at exec time.
- **State root (CODEX_HOME).** `dashboard_instance_provider_state_root` = `layout.roadmap_codex_home` = **`/home/briansrls/.codex@gunbc-roadmap`** (derivation in U1 row 5).
- **Wrapper.** `dashboard_environment_binding_executable` = `extdeps.tools.env::env_absolute_program().invocation` = `/usr/bin/env`. The assignment comes from `extdeps.llm.cli::codex_home_env_assignment`.

Argv, both via `dashboard_target_exec` and thus realized as in U4 (`env -C / <program> <args>`):

- **verify-provider-runtime:** `/usr/bin/env CODEX_HOME=/home/briansrls/.codex@gunbc-roadmap /opt/gunbc/provider-runtime/codex/current/node_modules/@openai/codex-linux-arm64/vendor/aarch64-unknown-linux-musl/bin/codex --version` (`shape_codex_version_env_args_for_program`). It passes when the combined output contains the codex-cli identity marker (`gunbc.package_delivery::codex_runtime_version_identity_satisfied`).
- **verify-provider-auth:** `/usr/bin/env CODEX_HOME=/home/briansrls/.codex@gunbc-roadmap <same codex> login status` (`shape_codex_login_status_env_args_for_program`).
- **Observation probes.** The same two commands also run as probes in `dashboard_instance_observe`, during preflight and readback. They are not plan-gated.

What they write:

- Nothing in the model: no write op, and only `CODEX_HOME` is added to the inherited environment (`env` adds and does not clear).
- Any write by the codex binary under `/home/briansrls/.codex@gunbc-roadmap` (logs, refreshed `auth.json`) is upstream behaviour and **not modeled** (see U2-residual under item 5).
- The apply's own mutation of that directory is U1 row 5.

## U4 RESOLVED: primitive realizations (seed interpreter route)

Common realization of a shell-transport service operation: `src/v1/stage0/src/v1_interpreter.rs` `dispatch_shell`.

- It evaluates the declared `transport shell { argv: [...] }` tokens, then calls `std::process::Command::new(argv[0]).args(argv[1..])`.
- Stdin is null, or piped when the transport declares `stdin:`. Stdout and stderr are piped.
- `configure_shell_process_group_for_wall_kill` puts the child in its own process group only under a witness wall deadline.
- **No `env_clear`, no `.env()`, no `.current_dir()`**: the child inherits the gunbc process environment and cwd.
- `Command::new` with a bare name searches the inherited PATH.
- **No sudo, ssh or systemd-run** is added by the realization.

| Primitive | Ultimately execs | cwd | Env | Wraps |
|---|---|---|---|---|
| `gunbc.WitnessBin.Run(workdir, bin_path, args)` (`extdeps.gunbc`) | `env -C <workdir> <bin_path> <args...>`; transport argv `["env","-C","{workdir}","{bin_path}",args]`; `env` via PATH; `bin_path` absolute or via PATH (env's execvp) | env chdirs to `workdir` | inherited unchanged | none |
| `dashboard_local_exec(workdir, program, args)` (`roadmap_dashboard_instance_observe`) | WitnessBin.Run with the caller's workdir: `/` for the serve-binary copy (`dashboard_apply_neutral_workdir`) | `/` on this route | inherited | none |
| `dashboard_target_exec` -> `dashboard_target_run(mode: TargetExec)` | Local transport (srv1 == srv1): `dashboard_target_ran_of_local` -> WitnessBin.Run(workdir: `dashboard_instance_apply_workdir` = `/`) -> `env -C / <program> <args>`. The SSH arm (`typed_argv_exec_over_ssh`) is unreachable | `/` | inherited | none (no ssh) |
| `dashboard_target_probe` (`TargetProbe`) | same as target_exec | `/` | inherited | none |
| `dashboard_target_run(mode: TargetExecWithStdin)` (used only by `dashboard_target_write_create_new` for `dd`) | `shell.Exec.RunArgvStdin(program, arguments, stdin_payload)` (`extdeps.shell.exec`) -> transport argv `[program, arguments]` with stdin = payload -> `Command::new("dd") of=<stage> conv=fsync status=none`; `dd` via PATH | **the gunbc process cwd** (not `/`; stated divergence in source). The `of=` path is absolute | inherited | none |
| `dashboard_run_target_argvs(commands)` | fold of `dashboard_target_exec` per command, stopping at the first failure | `/` | inherited | none |
| `dashboard_copy_to_target_atomically` (local arm) | `dashboard_local_exec(workdir: "/", /usr/bin/cp, [src, dst.next])` then `dashboard_target_exec(/usr/bin/mv, [dst.next, dst])`, i.e. `env -C / /usr/bin/cp ...` and `env -C / /usr/bin/mv ...`. The `/usr/bin/scp` arm is unreachable | `/` | inherited | none |
| `dashboard_run_privileged(op)` | `dashboard_target_exec(e.bin_path="/usr/bin/sudo", e.args=["-n", <op argv>])`, i.e. `env -C / /usr/bin/sudo -n /usr/bin/systemctl ...` / `/usr/bin/mv -n -- ...` | `/` | inherited into sudo; sudo then applies its own env_reset/secure_path policy (host sudoers, not modeled) | **sudo -n** (the only wrapper) |
| `dashboard_observer_read` (owned-dir stat/ls) | local arm: WitnessBin.Run(workdir `/`) of `/usr/bin/stat ...` / `/usr/bin/ls -1A -- <p>` | `/` | inherited | none |
| `shell.Mktemp.Dir()` (`extdeps.shell`, used by `dashboard_instance_apply_after_preflight`) | `Command::new("mktemp") -d`; `mktemp` via PATH | gunbc process cwd | inherited; **the location is `$TMPDIR` if set, else `/tmp`** (coreutils mktemp default template `tmp.XXXXXXXXXX`); path read from stdout | none |
| `shell.Remove.RecursiveForce(path)` | `Command::new("rm") -rf <the mktemp path>`; `rm` via PATH | gunbc process cwd | inherited | none |

**Temp dir lifecycle.**
- Created once, on the full Apply route only, after the initial reconcile says Apply.
- Removed after the plan fold, whatever its outcome.
- The bootstrap route creates none; it passes `/nonexistent/gunbc-bootstrap-has-no-temp-dir` (`dashboard_bootstrap_unused_temp_path`) and runs no bundle op.
- For the convergence seed no bundle op runs, so `<tmp>/source.bundle` and `<tmp>/verify.git` are never created; the rm -rf removes an empty directory.

**PATH.** The source fixes **no PATH** anywhere on this route:
- WitnessBin and dispatch_shell inherit the gunbc process env.
- The only env additions are `GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_NOSYSTEM=1` (git posture writes) and `CODEX_HOME=...` (codex probes).

Bare names resolved through the inherited PATH:

| Name | Where |
|---|---|
| `env` | every WitnessBin.Run, i.e. every target and local exec. Also the inner `env` of the git posture commands, resolved by the outer env's execvp |
| `git` | posture `config --local --replace-all` x3 and readback `config --local --get` x3 (`extdeps.git::git_program` = "git") |
| `find` | migration chgrp, chmod and readback walks (`extdeps.tools.findutils::find_program`) |
| `chgrp`, `chmod` | inside `find -exec`, resolved by find's own PATH search |
| `mktemp`, `ln`, `rm` | create-new write steps (`extdeps.tools.gnu_coreutils`) |
| `dd` | stdin leg |
| `mktemp`, `rm` | `shell.Mktemp.Dir` / `shell.Remove.RecursiveForce` |

All other programs are absolute: `/usr/bin/{install,test,stat,ls,cp,mv,touch,dirname,mkdir,sha256sum,od,curl,nc,git,systemctl,sudo,env}`, `/bin/{ls,cat,rm,chmod}` and the codex path.

## A. ADMIT: `repository_converge_owner_wet` (traced at 26fe4b8f350a / tree b0ee022a6bf8)

### Call

Entry: `gunbc.live_deploy.repository_convergence_placement::repository_converge_owner_wet(candidate, object_source, receipt, instance_id)`, called by the packet with:

- `candidate = 26fe4b8f350a99bd57a9bc342d4625b0cb96804b`
- `object_source = /home/briansrls/runs/ld449-r2work/wt`
- `receipt = /opt/gunbc-lab/codex-provider-feedback-v0/receipts/repository-convergence.json`
- `instance_id = srv1-lab-codex-feedback-v0`

The tracer reaches 800 functions across 131 modules. Hand reading followed the effectful path below. **R = `/opt/gunbc-lab/codex-provider-feedback-v0`** throughout.

### Values resolved for this instance

| Fact | Value | Symbol |
|---|---|---|
| desired row | `srv1_lab_desired_deployment()` (instance `srv1_lab_dashboard_instance()`) | `gunbc.live_deploy.desired::desired_deployment_for_instance_id`, `desired_deployment_roster` |
| spec | `selected_deployment_spec(row)` = `gunbc.live_deploy.spec::deployment_spec_for(...)` | same |
| repo | `/opt/gunbc-lab/codex-provider-feedback-v0/source` (`spec.service.repo_root` = `instance.repo_root`) | `spec::deployment_service_config` |
| expected principal | `briansrls` (`spec.service.service_user` = `gunbc.live_deploy.spec::gunbc_service_user`) | `convergence_principal_admission` in `placement_converge_for_spec` |
| base_ref | **`refs/gunbc/srv1-lab-admitted`** (instance `admitted_revision.ref_name`) | `placement_convergence_base_ref`; `roadmap_dashboard_instance::srv1_lab_admitted_revision_source` |
| remote (fetch source) | `/home/briansrls/runs/ld449-r2work/wt` (= object_source, verbatim) | `convergence_operands_for` |
| candidate | `26fe4b8f350a99bd57a9bc342d4625b0cb96804b`: 40 lowercase hex, so it passes `extdeps.git.object_store::git_decode_object_id_text` (sha1); re-rendered by `git_object_id_wire_hex` | `convergence_operands_for` |
| upload-pack string | `git -c 'safe.directory=/home/briansrls/runs/ld449-r2work/wt/.git' upload-pack` | `repository_convergence::convergence_fetch_upload_pack` -> `extdeps.git::git_local_fetch_upload_pack_command(git_local_fetch_source_grant(source))`; `git_non_bare_git_dir` appends `/.git` |
| receipt path | the caller's `receipt` argument, verbatim. Production would render the same string via `convergence_receipt_path(R)` = `gunbc.host.host_layout::instance_receipts_dir(R)` + `/repository-convergence.json` | `placement_finish` |
| bootstrap branch (empty-root arm only) | `main` | `gunbc.repository::gunbc_repository.default_branch` |
| superseded files retired | `R/source/dag/gunbc/recurring_failure_mode/roster.dag` and `R/source/dag/gunbc/rung_drop/roster.dag` | `superseded_workspace_file_paths` over `gunbc.repo_workspace::repo_workspace_footprints` (the only root-anchored, file-grain `SupersededAuthority` rows; `/gunbc-planning/` and `/scripts/fixtures/` are directory patterns and are excluded) |

### Realization of every effect

- **Shell-transport operations** (`git.Core.*`, `git.Inspect.*`, `access.PosixEffectivePrincipal.Read`) are realized by the seed interpreter's `dispatch_shell` (`src/v1/stage0/src/v1_interpreter.rs`) as `Command::new(argv[0]).args(...)`.
  - argv[0] is the **bare `git` or `whoami`, resolved through the inherited PATH**.
  - **No `env -C`, no `current_dir`**: the cwd is the gunbc process's own; every git call carries `-C <path>`.
  - **No env changes**: the environment is inherited unchanged.
  - **No sudo, ssh or systemd-run.**
- **`Filesystem.*`** is the in-process file transport (`dispatch_file`, same file): `std::fs::read_dir`, `remove_file`, `read_to_string` and `write`. Nothing is spawned.
- **This function starts no systemd unit.** `convergence_unit_doc` / `convergence_unit_rendering` / `convergence_exec_start` are in the module but are not reached from this entry. If the packet runs it inside a unit, that unit's User, WorkingDirectory and EnvironmentFile are the caller's facts, not this function's.

### Effects in order (the fold stops at the first refusing step)

| # | Step | Program + argv | Writes | Fires when | Symbol |
|---|---|---|---|---|---|
| A0 | operand guards | none | none | `receipt` or `instance_id` blank -> `exit_failure`, **no receipt written**. Unknown instance or roster refusal -> `exit_failure`, no receipt | `repository_converge_owner_wet` |
| A1 | principal | `whoami` (PATH) | none | always, once the row is found | `extdeps.access.posix_effective_principal_read_op::effective_posix_principal_read` |
| A1r | refusal receipt | none | **receipt file** (see A9) with verdict `wrong-principal` or `principal-unobservable` | whoami != `briansrls`, or whoami failed | `placement_converge_for_spec` -> `placement_finish` |
| A2 | object-source serve probes (**read-only, on object_source**) | `git -C /home/briansrls/runs/ld449-r2work/wt config --local --get extensions.partialClone`; `git -C ... config --local --get-regexp 'remote\..*\.promisor'`; `git -C ... config --local --get-regexp 'remote\..*\.partialclonefilter'`; `git -C ... rev-parse --is-shallow-repository` | none | principal admitted. All four run (`map`), and the first non-Absent standing wins. Promisor/filter or unreadable -> refuse; complete or shallow -> admit | `extdeps.git.partial_clone::observe_object_source_serve`; argv in `extdeps.git.inspect` `git.Inspect.ConfigGetInRepo` / `ConfigGetRegexpInRepo` / `IsShallowRepository` |
| A2r | operands refused | none | receipt `operands-refused` | object source cannot serve, candidate not a full lowercase sha1, object_source empty or relative, or no admitted-revision row | `placement_converge_admitted` |
| A3 | pre-state observation (**on repo**) | `git -C R/source for-each-ref --sort=refname --format=%(refname)%00%(objectname)%00%(symref)`; `git -C R/source worktree list --porcelain -z`; `git -C R/source rev-parse HEAD`; `git -C R/source reflog show --format=%H %gd %gs HEAD`; `git -C R/source ls-files --others --exclude-standard -z`; `git -C R/source write-tree`; `git -C R/source show -s --format=%T <HEAD or candidate>`; `git -C R/source diff --name-only -z <HEAD or candidate> --` | `write-tree` **writes a tree object** for the current index into `R/source/.git/objects` (declared `readonly` in the model; it creates an object and changes no ref) | always after operands are admitted | `repository_convergence::observe_repository_state`, `observe_repository_coherence` |
| A3b | empty-root arm (only if the pre-state is unobservable) | `std::fs::read_dir(R/source)` | none | any A3 read refused | `observe_repository_root_emptiness` |
| B1-B4 | **bootstrap** (only if `R/source` exists, is readable and is **empty**) | `git -C R/source init -q`; then the fetch (as A4); `git -C R/source update-ref refs/gunbc/srv1-lab-admitted 26fe4b8f… ""` (empty old = must not exist); `git -C R/source checkout -q -b main 26fe4b8f…`; then A3 again for the verdict | new repository, FETCH_HEAD, ref, branch `main`, worktree | empty root only. A non-empty or unobservable root returns the pre-state fault, written to the receipt | `repository_bootstrap_wet` |
| A4 | **fetch** | `git -C /opt/gunbc-lab/codex-provider-feedback-v0/source fetch --quiet --force "--upload-pack=git -c 'safe.directory=/home/briansrls/runs/ld449-r2work/wt/.git' upload-pack" /home/briansrls/runs/ld449-r2work/wt 26fe4b8f350a99bd57a9bc342d4625b0cb96804b` | objects into `R/source/.git/objects`; **`R/source/.git/FETCH_HEAD`**. The refspec is a bare object id with no destination and the remote is a path, not a configured remote, so **no `refs/remotes/*` and no other ref is written** | pre-state observable | `git.Core.FetchForcedRefInRepoTrustingSource` (`extdeps.git`) |
| A5 | **base-ref CAS** | `git -C R/source update-ref refs/gunbc/srv1-lab-admitted 26fe4b8f350a99bd57a9bc342d4625b0cb96804b <expected_old>` | **`refs/gunbc/srv1-lab-admitted` -> 26fe4b8f…** | fetch exit 0 | `git.Core.UpdateRefCompareAndSwapInRepo`; `expected_old` = `convergence_ref_hex_at(refs_before, base_ref)` |
| A5r | CAS refusal read | `git -C R/source rev-parse refs/gunbc/srv1-lab-admitted` | none | update-ref non-zero: `ConvergenceBaseRefMoved` if the ref changed, else `BaseRefUpdateRefused` | `base_ref_update_refusal_cause` |
| A6 | **reset** | `git -C R/source reset --hard 26fe4b8f350a99bd57a9bc342d4625b0cb96804b` | **moves whatever branch HEAD is on in `R/source`** (or detached HEAD) to the candidate; rewrites index and tracked worktree files; ORIG_HEAD; HEAD and branch reflogs. Untracked files are kept | CAS succeeded | `git.Core.ResetHardInRepo` |
| A6r | partial-transition read | A3's `rev-parse HEAD` set again | none | reset non-zero. **The ref has already moved; there is no rollback** | `ConvergencePartialTransitionLeft` |
| A7 | **superseded file retirement** (cleanup of files only pre-derived-roster binaries wrote; see the A7 re-check below) | `std::fs::remove_file`, then `std::fs::read_to_string` read-back, for each of the two `roster.dag` paths above | deletes those two files if present (absent counts as retired) | reset succeeded | `retire_superseded_workspace_files` -> `retire_superseded_file` |
| A8 | post-state + verdict | A3 again (including `write-tree`) | object from `write-tree` | always after A7 | `repository_convergence_verdict` -> `repository_convergence_compare` (pure) |
| A9 | **receipt** | in-process `std::fs::write(receipt, json)` | **`/opt/gunbc-lab/codex-provider-feedback-v0/receipts/repository-convergence.json`**, schema `gunbc.repository-convergence-placement.v1` (`placement_receipt_json`), verdict one of converged / wrong-principal / principal-unobservable / operands-refused / convergence-refused | on every arm past A0 | `placement_finish` |

### A7 re-checked at 26fe4b8f350a (correction request from calm-crab-645)

**The keeper is right about the writer, and A7 still stands.**

**The writer is gone.** `src/v1/stage0/src/cli_run/derived_row_roster.rs` states it in its module doc ("An earlier realization wrote a gitignored `roster.dag` into the row directory on every walk ... A physical file at the reserved identity now REFUSES"). The code matches:
- The roster is derived in memory under the reserved name `ROSTER_BASENAME` = `"roster.dag"`.
- `acquire_dir_files` refuses a physical file at that identity and "never deletes" it ("source loading performs no cleanup").
- The only write calls in that file sit below its `#[cfg(test)]` boundary.

So no current gunbc binary creates these files, and nothing in section A claims one does.

**The deletion still exists and is reachable from `repository_converge_owner_wet`.** The call chain, all in `dag/gunbc/live_deploy/repository_convergence.dag` (my tracer reaches every symbol in it):
1. `repository_converge_owner_wet`
2. `placement_converge_for_spec`
3. `placement_converge_admitted`
4. `placement_converge_with_operands`
5. `repository_convergence::repository_converge_wet` (the `SupersededFilesAbsent` / `retire_superseded_workspace_files` arm, after `ResetHardInRepo` succeeds)
6. `retire_superseded_workspace_files`
7. `retire_superseded_file`, which runs `Filesystem.Delete`, then `superseded_file_read_back` (`Filesystem.Read`)

**Which paths it deletes.** The population is `superseded_workspace_file_paths` over `gunbc.repo_workspace::repo_workspace_footprints` -> `own_tool_footprints`, filtered by `artifact_is_superseded_file`. At this head that is exactly the two root-anchored, file-grain `SupersededAuthority` rows: `/dag/gunbc/recurring_failure_mode/roster.dag` ("Retired failure-mode roster file") and `/dag/gunbc/rung_drop/roster.dag` ("Retired rung-drop roster file"). These are joined under `repo`, giving `R/source/dag/gunbc/recurring_failure_mode/roster.dag` and `R/source/dag/gunbc/rung_drop/roster.dag`.

**What the step does.**
- **Effect.** It is cleanup of files that only *older* binaries wrote. A file that is absent counts as retired (`FilesystemDeleteTargetAbsent` -> read-back), so on a tree no old binary touched this is two `remove_file` calls returning NotFound plus two NotFound reads.
- **gitignore.** Both paths match `.gitignore` (`/dag/gunbc/recurring_failure_mode/roster.dag`, `/dag/gunbc/rung_drop/roster.dag`).

**Nothing removed.** Every symbol cited in A7 exists at this head. The A7 row and the "superseded files retired" value row are correct as written.

### Refs written in `R/source`

**`refs/gunbc/srv1-lab-admitted`** (A5) gets new value `26fe4b8f350a99bd57a9bc342d4625b0cb96804b` under compare-and-swap. The CAS behaviour, in git's update-ref semantics:

- **Old value is the hex read in A3.** The CAS refuses if the ref moved between A3 and A5.
- **Old value is `""` when the ref did not exist.** git then requires it to still not exist.
- **Reflog only if configured.** A reflog entry for `refs/gunbc/*` is written only if `core.logAllRefUpdates=always`; the default `true` logs only heads, remotes and notes. The repository's config value is not modelled.

**The branch HEAD is on (A6)** moves to the candidate. Which branch that is, is a runtime fact (`git worktree list` primary); the verdict admits that branch and the base ref moving, and refuses if any other ref, or any linked-worktree binding, differs from A3.

**`FETCH_HEAD`** (A4) is written by the fetch.

**`refs/remotes/*` is not written.** No refspec destination and no configured remote are used. The module header states this as the reason the fetch does not collide with the deployed-tree no-fetch ruling.

**Bootstrap arm only** (B1-B4): additionally `refs/heads/main` and HEAD, in a fresh repository.

### What it reads from the object source (`/home/briansrls/runs/ld449-r2work/wt`)

- **A2:** four `git config` / `rev-parse` reads.
- **A4:** `git upload-pack` run against that path by the fetch, under `safe.directory=/home/briansrls/runs/ld449-r2work/wt/.git` (one invocation, persisted nowhere). It reads:
  - its ref advertisement;
  - the object closure of `26fe4b8f…` that `R/source` lacks.
- Nothing is written to the object source.

### Principal, PATH, network

- **Principal.** The process must already be `briansrls` (A1 gate). Every git call runs as that uid with no elevation.
- **PATH lookups.** `whoami` and `git` (A2-A8). The upload-pack string's inner `git` is also PATH-resolved, by the shell the fetch uses for `--upload-pack`. The source fixes no PATH.
- **Network.** None. The remote is a local path and the transport is local upload-pack.
- **No sudo, systemctl, systemd-run or ssh** is reached. Those words appear in the closure only inside data rows (spec/host software surface, the sudoers grants roster), which this entry does not execute.

### Conditional branches (summary)

1. **Argument guards.** A blank `receipt` or `instance_id`, or a roster or instance lookup failure, exits without a receipt.
2. **Principal gate.** whoami must equal `briansrls`.
3. **Object-source serve admission.** Partial or promisor sources refuse; shallow sources are admitted.
4. **Operand checks.** Candidate decode; object_source absolute and non-empty; base ref present for the instance.
5. **Pre-state observability.** If the pre-state cannot be observed: an empty root takes the bootstrap; a non-empty or unobservable root refuses.
6. **Transition steps.** fetch, then CAS, then reset, then retirement, each stopping on failure. **A reset failure leaves the base ref already advanced.**
7. **Verdict.** It compares pre- and post-state and does not roll back on refusal.

### UNRESOLVED (section A)

- **A-U1. HEAD's branch in `R/source`.** Which branch HEAD is on there (hence which `refs/heads/*` A6 moves), and the current value of `refs/gunbc/srv1-lab-admitted`, are runtime host facts.
- **A-U2. Linked-worktree object source.** Whether `/home/briansrls/runs/ld449-r2work/wt` is a linked worktree. If it is, its `.git` is a file and the gitdir lives elsewhere, so the derived `safe.directory=.../wt/.git` may not be the path git compares. This only matters when the source is owned by another uid.
- **A-U3. Upstream git behaviour.** Not modelled:
  - fetching a bare object id over a local path (protocol v2 accepts arbitrary wants);
  - auto-gc or maintenance after the fetch (the apply's posture sets `gc.auto=0` and `maintenance.auto=false`, but only once the apply has run);
  - `git diff` index refresh.
- **A-U4. Receipt directory.** `R/receipts` is not created by this entry, and it is not in the apply's owned-directory demands (U1). `std::fs::write` does not create parents, so if `R/receipts` is absent the receipt write fails and the entry exits failure **after** the git transition already ran. File mode is 0666 & ~umask; the write is a truncate-in-place, not an atomic rename.
- **A-U5. Roster standing evaluated by hand only.** The roster (`desired_roster_standing` same-host and duplicate checks) and the spec construction are pure, but I did not evaluate them by execution. I read them as admitting the lab row.
- **A-U6. Interpreter route only.** Same as U4-residual: the seed interpreter route was traced; an emitted native binary's realization was not inspected.

## W. What writes inside a repo worktree used as cwd and source root (static, 26fe4b8f350a)

### Invocations covered

- `claim_batch [--wet] --claim-run --source-root dag --source-root src/v1 --source-root src/v2 --entry <witness> --functions ...`
- `gunbc run --source-root dag --source-root src/v2 --entry ... --function ...`

Both run with cwd = the worktree.

### W1. Writes by the Rust host itself (claim_batch, the gunbc CLI, cli_run)

**Method.** I grepped every `fs::write` / `File::create` / `create_dir_all` / `fs::rename` / `OpenOptions` site under `src/v1/stage0/src`, then:
- dropped sites inside `#[cfg(test)]` modules;
- dropped sites only reachable from other verbs or binaries (`claim_executor`, `--v2-native-route`, `gunbc build/converge`, `required_regen_host`, `--outcomes-out`, compile output).

**Result: no path under the cwd or under `dag/`, `src/v1` or `src/v2` is written by either binary on these command lines.** No `.gunbc/`, no `*-stamp`, no `roster.dag`, no `.gunbc-build-receipts/`, no `.claim-map/`.

| Path | Writer (file::symbol) | Reached by the packet's command lines? | gitignored? |
|---|---|---|---|
| `/sys/fs/cgroup/gunbc.bound/{memory.max,cgroup.procs}`, `/sys/fs/cgroup/cgroup.subtree_control` | `src/v1/stage0/src/memory_governor.rs::apply_memory_cgroup_bind` (leaf `BIND_MEMORY_CGROUP_LEAF` = `gunbc.bound`) | claim_batch only (`bin/claim_batch.rs::main`), and **only when `GUNBC_BIND_MEMORY_CGROUP_BYTES` is set**. Outside the repo | n/a (outside the repo) |
| `$TMPDIR/gunbc-emit-host-<pid>-<n>/` | `src/v1/stage0/src/v1_interpreter.rs::eval_emit_host_run_transport_builtin` | only if an executed .dag path calls the emit-host builtin (not seen in these witnesses' own bodies). Outside the repo | n/a |
| `$GUNBC_NATIVE_CACHE_ROOT`, else `$XDG_CACHE_HOME`, else `$HOME/.cache` `/gunbc_*/...` (with `.native_publish.lock`) | `v1_interpreter.rs::run_cached_process_spec` / `native_cache_rebase_workspace_dir` / `emit_host_run_transport_cached_in_workspace` | same condition as the row above. Outside the repo | n/a |
| `<outcomes-out>.partial` -> `<outcomes-out>` | `bin/claim_batch.rs::write_outcomes_ledger` | **no**: only with `--outcomes-out`, which the packet does not pass | none |
| `.gunbc-build-receipts/`, `.gunbc-converge-receipts/` (cwd-relative) | `src/v1/stage0/src/cli_run.rs::bootstrap_invocation_receipt_path` via `gunbc_cli_dispatch_generated.rs` | **no**: only the `gunbc build` / `gunbc converge` verbs; `run` goes to `run_verb` | yes: `/.gunbc-build-receipts/`, `/.gunbc-converge-receipts/` |
| `<workspace>/target/gunbc-roadmap-auth-proj-<pid>-<n>/dag/gunbc/roadmap/roadmap_authority.dag` | `cli_run.rs::project_roadmap_acceptance_event_history_from_authority_text_inner` | **no**: its only caller is in the `#[cfg(test)] mod roadmap_acceptance_history_projection_tests` (cli_run.rs, from line 1411) | yes: `/target/` |
| `roster.dag` in row directories | (removed) `cli_run/derived_row_roster.rs` writes none; the roster is in memory | n/a | yes |
| native product cache `store/`, `bin/` | `cli_run/native_product_cache.rs::event_may_publish`, `unavailable` | **no**: only from `cli_run/native_lane_runner.rs` (native route) | n/a |

### Source acquisition and workspace root are read-only

- `cli_run.rs::bind_process_workspace_root` / `workspace_root_resolve` walk ancestors for `.git` and read only.
- `derived_row_roster.rs::acquire_dir_files` reads only.
- `install_output_policy` evaluates `dag/gunbc/output_policy.dag` in memory.

### W2. Writes by the .dag code the packet executes

**These come from the executed .dag's own effects** (file transport `dispatch_file` = `std::fs::*`; shell transport = spawned `mkdir`, `mktemp`, `rm`, `ln`, ...). Paths are whatever the .dag passes.

**Hermetic vs wet.** In claim_batch hermetic mode (no `--wet`, per `cli_run::execution_mode_for`), operations that declare a `mock_response` answer from it. Whether every write in these witnesses is mocked was **not** determined (see W-U3).

| Entry | Writes it can perform (static) | Under cwd / source roots? |
|---|---|---|
| `session_placement_wet_witness` | scratch dir from `shell.Mktemp.DirWithTemplate("/tmp/gunbc_g5_...")`, then `shell.Mkdir.Parents`, `Filesystem.Write` / `WriteCreateNew*`, and the `roadmap_session_placement` writers (`session_placement_record_write`, `..._release_write`, `session_settle_note`, durable CAS slots) under that scratch instance | no (absolute `/tmp/gunbc_g5_*`) |
| `roadmap_belt_actuate_witness` | scratch `/tmp/gunbc_pub_...` via `Mktemp.DirWithTemplate` (`belt_pub_fresh_instance`, `with_publish_attempt_directory`) | no for those. Name-reachable writers such as `belt_exec`, `belt_codex_preflight_for_instance` and `compute.test_run::*` also exist (W-U1). Relative strings (`"dag/x.dag"`, `"src/v2"`, ...) appear in pure assertions |
| `roadmap_dashboard_instance_witness` | none found in the witness body (pure; `/tmp/dashboard-endpoint-witness` is a data literal) | no |
| `attempt_system_unit_launch_witness` | `/tmp/...` strings are data; name-reachable writers `attempt_system_unit_enqueue_request` / `attempt_system_unit_rename_exact` (W-U1) | not determined |
| `attempt_state_move_apply_wet_witness` | scratch `/tmp/gunbc_attempt_state_move_wet_...`; `Mkdir.Parents`, `Filesystem.Write`, `WriteCreateNewWithMode`, `Chmod.RecursiveReadOnly` / `RecursiveWritable`; and the apply's move leg (U4 primitives; `mktemp`, `dd`, `ln`, `rm` under the scratch attempt-state) | no |
| `lab_apply_operator_grant_witness` | none in the witness body | no |
| `worker_git_subtree_apply_spawn_wet_witness` | `mktemp` templates `/tmp/wre3-*-XXXXXX` (`scratch_root`); git repos, chgrp and chmod under them | no |
| `dispatch_worker_confinement_witness` | none in the body (`scratch_and_git_carve_outs` is a pure model) | no |
| `operator_endpoint_witness` | none in the body (`/tmp/hdr/authorization` is data; `dag/extdeps/http/client.dag` is a string, not a write) | no |
| `agent_door_witness`, `request_authentication_witness`, `roadmap_event_log_witness`, `dispatch_worker_traversal_witness` | none in the witness bodies | no |
| `harness_validate_worktree_witness` | none: uses `/nonexistent/...` paths; its header says no `WitnessBin.Run` is reached | no |
| `ld449_lab_render_probe` (gunbc run) | **none**: no effect primitive reachable (pure render) | no |
| `srv1_lab_dashboard_preflight_wet` (gunbc run) | **no writes**: only probes through `dashboard_target_probe` / `dashboard_local_exec` (`env -C / ...`: `test`, `git rev-parse` / `status`, `ls`, `stat`, `nc`, `curl`, codex `--version` / `login status`). The plan is computed but never dispatched | no. The external codex CLI may write under `/home/briansrls/.codex@gunbc-roadmap` (U2-residual) |

### gitignore cross-check (patterns named by the coordinator, from `.gitignore` at this head)

| Pattern | Producer | On these command lines? |
|---|---|---|
| `/.gunbc/` | owned by `v2.workflow.required_floor` (floor workflow) | not produced |
| `/dag/gunbc/recurring_failure_mode/roster.dag`, `/dag/gunbc/rung_drop/roster.dag` | no current writer | not produced |
| `.*-stamp` | `gunbc.generated_artifact` generators | not produced |
| `/.gunbc-build-receipts/` | `gunbc build` only | not produced |

**Nothing found to be written on these command lines lands in the worktree.** So under this trace, no gitignored or untracked file is created in the checkout. The negative is bounded by W-U1 to W-U4.

### UNRESOLVED (W)

- **W-U1. Name-reachable writers not proven unexecuted.** Name-based reachability from the witness modules reaches 83 functions that contain write primitives, among them:
  - `compute.work_provider_local::compute_exec` / `compute_unstage` / `compute_remove_owned_target`;
  - `compute.test_run::compile_entry_cli`;
  - `roadmap_belt_actuate::belt_exec`;
  - `harness_shell_job::harness_start_shell_job`;
  - `provider_wire_evidence::*`;
  - `roadmap_event_carrier::roadmap_event_private_snapshot`;
  - `materialization_store_local::*`.

  I did not prove which of these an executed test reaches, or that every path they receive is absolute and outside the worktree. The full list is in `tr/w/writers.txt` in the scratchpad.
- **W-U2. Relative-path effect arguments.** A relative path passed to the file transport resolves against the process cwd (`dispatch_file` applies no root). If any executed writer is handed a relative path, it writes into the worktree. None was found in the witness bodies, but the transitive callees were not checked path by path.
- **W-U3. Hermetic-mode coverage.** Which operations in these witnesses carry `mock_response` was not enumerated.
- **W-U4. Shell children.** Child processes spawned by shell effects (git, cargo, codex) may write into their own cwd or HOME. The shell transport inherits the gunbc cwd (U4), so a child invoked without `-C` and an absolute path would act in the worktree. Not enumerated per call.
- **W-U5. Interpreter route only.** Only the seed interpreter route was read; an emitted native binary was not.

## X. `roadmap_belt_actuate_witness` under `--wet` (dag/test/claim/roadmap/roadmap_belt_actuate_witness.dag, 26fe4b8f350a)

### Method

**1. Name-based closure per test.** `tr/pertest.py` and `tr/pertest2.py` compute a closure for each of the **149 `test fn`s**.
- **120 tests** reach no effect primitive at all, even under name over-approximation. They are pure.
- **29 tests** reach one or more primitives by name.

**2. Hand reading.** Each of the 29 was read at its call sites to separate:
- **executed**: the primitive runs on the path the test takes;
- **data**: an argv or command value that is built and compared, never run;
- **unreached**: the primitive sits in a match arm or branch that the test's inputs do not take.

**Two reading rules.**
- **Strict evaluation.** The seed interpreter evaluates call arguments strictly: `Value` in `src/v1/stage0/src/v1_interpreter.rs` has no thunk variant. So an argument expression runs even when the callee's taken arm ignores it.
- **The reviewer's builders are data here.** The `shell.Mktemp.Dir`, `systemd_run_command_of` and ssh argv builders the reviewer named are reachable **only by name**. None is executed by any test in this file (detail below).

### X1. Effects that are actually EXECUTED

| Test(s) | Effect | Program / argv or path | Principal / transport | Cleaned up? | Citation |
|---|---|---|---|---|---|
| `witness_exec_ok_runs_echo`, `witness_exec_refuses_missing_program`, `witness_exec_fails_on_nonzero_exit`, `witness_exec_all_runs_every_step_when_ok`, `witness_exec_all_short_circuits_after_failure`, `witness_exec_all_fails_on_later_step`, `witness_named_exec_steps_locate_the_first_failure` | process exec (twice per command) | (1) `sh -c 'command -v "$1"' sh <prog>` (`shell.PosixCommandV.Check`). (2) If found, `env -C / <prog> <args>` (`gunbc.WitnessBin.Run`). Programs: `echo belt-ok`, `echo first`, `echo second`, `echo ok`, `false`, `/usr/bin/true`, `/usr/bin/false`. `belt-nonexistent-program-xyz` gets only the `command -v` probe. `echo should-not-run` is never run (short-circuit) | local, the run user; `sh`, `env`, `echo`, `false` via PATH | nothing written | `gunbc.roadmap.roadmap_belt_actuate::belt_exec`, `belt_exec_all`, `belt_exec_steps`, `belt_program_available` |
| `a_failed_write_reports_loss_rather_than_the_outcome_it_computed` | mktemp dir (x2) | `mktemp -d /tmp/gunbc_pub_pubok.XXXXXX` and `mktemp -d /tmp/gunbc_pub_publost.XXXXXX` | local; `mktemp` via PATH | **no**: this file contains no `shell.Remove` / `rm`, so both directories persist | witness `belt_pub_fresh_instance` (`shell.Mktemp.DirWithTemplate`) |
| same | mkdir -p | `mkdir -p <pubok dir>/...` publication attempt-state directory for node `belt-pub-node`, key `k1` (`attempt_state_directory_text(..., AttemptDirectoryPublication)`) | local | no | witness `with_publish_attempt_directory` (`shell.Mkdir.Parents`) |
| same | dir list + file write | `Filesystem.List` of the attempt-state dir. Then `Filesystem.Write` (in-process `std::fs::write`) of the publication receipt JSON at the `AttemptPublicationReceipt{head_sha}` member under `/tmp/gunbc_pub_pubok.*`. The `publost` instance's write is **refused** (its directory was not made), which is the point of the test | in-process | no | `roadmap_belt_actuate::belt_publish_attempt_receipt` -> `roadmap_attempt_state_store::attempt_state_write_at` / `attempt_state_write` |
| `belt_lineage_standing_is_unknown_for_an_unreadable_ledger_and_ends_a_reaped_row` | file read (production path) | `Filesystem.Read` of the srv1-live attempt worker-events log for node `shell-dag-lineage-standing-unknown`, key `a1` (`attempt_log_member_text(srv1_live_dashboard_instance(), ..., AttemptWorkerEvents)`). Only the `AttemptsObserved { attempts: [] }` arm reads; the `AttemptsRefused` arm reads nothing | in-process, read-only | n/a | `roadmap_belt_actuate::belt_lineage_standing` -> `belt_prior_attempt_recovery` |
| `verify_without_observable_subject_defers_instead_of_rerunning` | env read, file read, clock, **network** | Executed because the provider is completed, so `belt_lookup_node` -> `gunbc.roadmap.roadmap_runtime_plan::roadmap_plan_projection` runs:<br>(a) `shell.Env.Get GUNBC_FABRIC_STATE_KEY_FILE` (`printenv`).<br>(b) `Filesystem.Read /etc/gunbc-fabric-storage/state-writer-mac-key` (or the env path) (`fabric_storage_state_auth_io::fabric_state_keyring`).<br>(c) `Clock.Now` (`clock_read::clock_now_probed_at`).<br>(d) if the key loaded: **`curl -sS --connect-timeout <s> --max-time <s> -X POST --data-binary @- -w '\n%{http_code}' https://srv1.tailecbe08.ts.net:10000/fabric-storage/state/authenticated`**, with a signed `closure` request on stdin (`fabric_storage_client::served_post_raw`, `http.Client.PostStdinWithin`). The URL comes from `fabric_storage_placement::fabric_storage_endpoint_for` (holder srv1-live, `fleet_tailnet_domain`, port `fabric_storage_https_port` 10000) + `fabric_storage_state_suffix`.<br>(e) If the roster reads, the same POST again for the acceptance history (`roadmap_authority::roadmap_acceptance_event_history_load`).<br>The operation name is `closure` (a read of the roadmap task partition). This witness performs no write call against the store | local; `curl` via PATH; **network to srv1's tailnet endpoint**; authenticated with the host's fabric-state MAC key if readable | n/a | as cited |
| `observe_only_instance_refuses_dispatch_and_stop` | same as the row above: env, key read, clock, network POST, `closure` | Both refusals (observe-only) are model-only, **but** `belt_dispatch_node_staged` passes `graph: launch_graph_of_projection(proj: roadmap_plan_projection())` as an argument, and strict evaluation runs it before the observe-only refusal | same as above | n/a | `roadmap_belt_actuate::belt_dispatch_node_staged` |

### X2. Effects reachable by name only (NOT executed by this file's tests)

| Test(s) | Why nothing runs | Citation |
|---|---|---|
| `witness_spawn_member_refuses_observed_shape`, `witness_identity_survives_actuation_failure` | `ObservedMember` -> `SpawnFailed "member-shape"`. `belt_actuate_spawn` (worktree add, tmux, systemd-run, provider start) is in the untaken `DesiredMember` arm | `belt_actuate_spawn_member` |
| `witness_teardown_member_refuses_desired_shape` | `DesiredMember` -> `TeardownFailed`. Teardown (tmux kill) is untaken | `belt_actuate_teardown_member_for_instance` |
| `witness_observe_*` (7 tests) | `belt_observe_from_result` is fed synthetic results. `dispatch_actuator_selection` -> `dispatch_harness_selection` (which by name reaches `shell.Mktemp.Dir`, operator-header `Filesystem.Write`, and harness HTTP GETs in `harness_operator_route::operator_open_header` / `harness_backend::harness_bounded_get`) sits only in the `TmuxSessionsListed` arm. None of these inputs produces that arm: no-server, no-sessions and ENOENT give `TmuxNoServer`; the others refuse | `belt_observe_from_result` |
| `agent_incomplete_attempt_defers_verification` | provider events are empty, so `belt_provider_candidate_ready` is false and `VerifyDeferred` returns. The verify worktree, retry-ledger `Filesystem.Write` and `belt_exec` are untaken. `workflow_attempt_progress` is pure (the name closure reaches no primitive) | `belt_verify_attempt_in_mode` |
| `staged_post_refuses_*` / `staged_post_transition_halt_refuses_*` (4) | a fixture graph and fixture standing go to `belt_dispatch_node_staged_over`, which refuses at `launch_admission_staged` (model-only). No projection is passed | witness `staged_post_over_canary` |
| `the_belt_addresses_the_attempt_its_spawn_commands_were_built_for` | **data only**: `dispatch_spawn_commands_for_launch` builds the worktree-add and attempt-state-prepare argv, which the test compares as strings. The one `Filesystem.Read` hit is a false positive (the text inside a string literal in `roadmap_authority::declared_roadmap_nodes`) | witness body |
| `an_empty_node_id_*`, `an_overlong_node_id_*` | `dispatch_attempt_unit_name` refuses (empty stem / over 255) before `systemctl --user show` | `roadmap_belt_actuate::attempt_process_for_node` |

### Explicit answers for --wet

| Effect | Status in this file's tests |
|---|---|
| systemd-run | **never executed**: only built (spawn-command builders); no test takes a spawn arm |
| ssh | **never executed** |
| sudo | **never executed** |
| systemctl | **never executed**: both systemctl-reaching tests refuse before the call |
| network | **executed by 2 tests**: the authenticated HTTPS POST (`closure`) to `https://srv1.tailecbe08.ts.net:10000/fabric-storage/state/authenticated`, conditional on the fabric-state key being readable. The roster and history reads may repeat it |
| tmux | never executed |
| host writes | only under `/tmp/gunbc_pub_pubok.XXXXXX` (mkdir + one receipt file) and the empty `/tmp/gunbc_pub_publost.XXXXXX`, **never cleaned up** |

### X3. /tmp templates in the 14 packet witness files (direct literals in each file)

| File | Templates | Cleanup in file |
|---|---|---|
| `roadmap/session_placement_wet_witness.dag` | `/tmp/gunbc_g5_<tag>.XXXXXX` (`shell.Mktemp.DirWithTemplate`) | none found (no Remove/rm in file) |
| `roadmap/roadmap_belt_actuate_witness.dag` | `/tmp/gunbc_pub_<tag>.XXXXXX` (tags `pubok`, `publost`) | none |
| `roadmap/attempt_state_move_apply_wet_witness.dag` | `/tmp/gunbc_attempt_state_move_wet_<tag>.XXXXXX` | none |
| `roadmap/worker_git_subtree_apply_spawn_wet_witness.dag` | `/tmp/wre3-{posture,migrate,narrow,confine,worktree,entry,commit,control,posture-absent,posture-unreadable,posture-leg,gate}-XXXXXX` (`mktemp_directory_command`) | `rm_recursive_force_command` appears 20 times. Per-test coverage not verified (X-U3) |
| `roadmap/attempt_system_unit_launch_witness.dag` | none created. `/tmp/out`, `/tmp/err`, `/tmp/wt`, `/tmp/x`, `/tmp/e`, `/tmp/o`, `/tmp/gunbc-credstore.abc123`, `/tmp/gunbc=separator` are argv/data literals; line ~1264 asserts a built argv starts `mktemp -d` (data) | n/a |
| `roadmap/roadmap_dashboard_instance_witness.dag` | none created (`/tmp/dashboard-endpoint-witness` is data) | n/a |
| `operator_endpoint_witness.dag` | none created (`/tmp/hdr/authorization` is data) | n/a |
| `runner/lab_apply_operator_grant`, `roadmap/dispatch_worker_confinement`, `live_deploy/agent_door`, `request_authentication`, `roadmap/roadmap_event_log`, `harness_validate_worktree`, `roadmap/dispatch_worker_traversal` | no `/tmp` or mktemp literal | n/a |

**Completeness.**
- The list is complete for literal templates written in these 14 files (grep for `Mktemp.`, `mktemp`, `XXXXXX` and `/tmp`, plus reading each hit).
- It is **not** complete for temp paths created by library code the tests call. These library creators appear in the name closures:
  - `harness_operator_route::operator_open_header`: `shell.Mktemp.Dir`, which goes to `$TMPDIR` or `/tmp/tmp.XXXXXXXXXX`;
  - `provider_wire_evidence::mktemp_file_in_dir` (`mktemp -p <dir>`);
  - the create-new staging in `dashboard_target_write_create_new` (`<dir>/.gunbc-create-XXXXXX`);
  - durable-store staging names.
- Whether any is executed was settled only for the belt file (X1/X2: none of these runs).

### UNRESOLVED (X)

- **X-U1.** The exact subpath strings of the attempt-state members (`attempt_state_provision*`) under `/tmp/gunbc_pub_pubok.*` and under srv1-live's attempt-state root were left as symbol templates, not expanded to literals.
- **X-U2. Untested network branch.** Whether the fabric-state key is readable on the packet host, and therefore whether the network POSTs happen, is a host fact. If the key is unreadable, no network call is made and the projection refuses.
- **X-U3. wre3 cleanup.** For the 13 other witness files, only templates and cleanup presence were checked. Per-test executed-vs-data classification was done for this belt file only, and per-test cleanup of the wre3 roots was not verified.
- **X-U4. Hermetic mode.** Hermetic-mode behaviour (without `--wet`) was not analysed.

## P. `srv1_lab_dashboard_preflight_wet` (dag/gunbc/roadmap/roadmap_dashboard_instance_apply.dag, 26fe4b8f350a)

### Transports and principals

**Entry.** `dashboard_instance_preflight_wet(srv1_lab_dashboard_instance(), ProductionPeer{srv1_live_dashboard_instance()})` -> `dashboard_instance_apply_preflight`.

**Two transports are used, and that matters for the production probes.**
- **Lab probes** (instance `srv1-lab-codex-feedback-v0`): the actuator host and instance host are both `srv1`, so `DashboardLocalProvision`. Each probe is `env -C / <program> <args>` via `gunbc.WitnessBin.Run`, as the run user (see U4).
- **Production probes** (instance `srv1-live`): `srv1_live_dashboard_instance` declares `actuator_host: "macbook"` but `host_identity: "srv1"`. `dashboard_provision_transport` therefore selects **`DashboardSshProvision { host: "srv1" }`**, so each production probe runs as **`ssh srv1 -- <program> <args>`**:
  - `gunbc.typed_argv_exec::typed_argv_probe_over_ssh`
  - -> `ssh.Session.ExecPortableWords`, transport argv `["ssh", client_args]`
  - with `client_args` from `gunbc.fleet.fleet_known_hosts_anchor::shape_ssh_shell_exec` = `[host, "--", argv...]`.

  **This is true even when the preflight runs on srv1 itself.** `ssh` is PATH-resolved and uses the run user's ssh config, keys and known_hosts. The remote principal is whatever user `srv1` resolves to in that ssh config (not in source).

**Correction to earlier sections.** The apply also reads production through `dashboard_production_read`, before and after the plan (`dashboard_instance_apply_after_preflight`), so the apply makes the same two **ssh** probes. Wherever the earlier sections say "no ssh", read it as: **the lab-target execs use no ssh; the srv1-live production probes do.**

### Every process and request, in order (no step writes)

| # | Program + argv | Target (literal) | Transport / principal | R/W | Symbol |
|---|---|---|---|---|---|
| P1 | `ssh srv1 -- /usr/bin/git -C /opt/gunbc/gunbc rev-parse HEAD` | production repo `/opt/gunbc/gunbc` (`gunbc.host.host_layout::srv1_gunbc_repo_root`) | ssh to `srv1` | read-only | `dashboard_production_snapshot` |
| P2 | `ssh srv1 -- /usr/bin/curl -fsS http://127.0.0.1:8080/healthz` | srv1-live serve on loopback port 8080 (`listen_port: 8080`) | ssh to `srv1`; HTTP GET on srv1 loopback | read-only | `dashboard_production_snapshot` |
| P3 | `env -C / /usr/bin/test -d /opt/gunbc-lab/codex-provider-feedback-v0` | lab root | local | read-only | `dashboard_instance_apply_preflight` |
| P4 | `env -C / /usr/bin/test -d <B>`, `env -C / /usr/bin/test -w <B>` | B = the lab root if present, else `/opt/gunbc-lab` | local | read-only | same |
| P5 | `env -C / /usr/bin/git -C /opt/gunbc-lab/codex-provider-feedback-v0/source rev-parse refs/gunbc/srv1-lab-admitted` | lab source repo | local | read-only | `DashboardSeededByConvergence` arm |
| P6 | `env -C / /usr/bin/test -x <p>` for each of `/usr/bin/git`, `/usr/bin/curl`, `/usr/bin/mkdir`, `/usr/bin/touch`, `/usr/bin/mv`, `/usr/bin/nc`, `/usr/bin/dirname`, `/bin/chmod`, the codex executable (U2), plus `dashboard_instance_dispatch_platform_executables` (P-U2) | local executables | local | read-only | `dashboard_survey_target_executables` |
| P7 | `env -C / /usr/bin/curl -fsS http://127.0.0.1:18081/instance.json` | lab serve health on srv1 loopback | local | read-only | `dashboard_read_instance_health` |
| P8 | `env -C / /usr/bin/env CODEX_HOME=/home/briansrls/.codex@gunbc-roadmap <codex> --version`, then `... login status` | codex runtime and auth (U2) | local | read-only in the model (codex may write under CODEX_HOME: U2-residual) | `dashboard_instance_observe` |
| P9 | `env -C / /usr/bin/git -C R/source rev-parse HEAD` | lab source | local | read-only | `dashboard_source_revision_readback` |
| P10 | `env -C / /usr/bin/test -d R`, `-f R/.gunbc-dashboard-instance-srv1-lab-codex-feedback-v0`, `-d R/origin.git`, `-d R/source` | lab paths | local | read-only | `dashboard_instance_observe` |
| P11 | for each of the 28 owned-directory demands (U1): `env -C / /usr/bin/ls -1A -- <path or ancestor>` and `env -C / /usr/bin/stat <owned_directory_stat_args> <path>` | U1 paths, including `/home/briansrls/.codex@gunbc-roadmap` | local | read-only | `dashboard_observe_owned_directories` -> `dashboard_owned_directory_observe` / `dashboard_observer_read` |
| P12 | `env -C / /usr/bin/test -d R/source/.git`; `env -C / /usr/bin/git -C R/source rev-parse HEAD`; `env -C / /usr/bin/git -C R/source rev-parse refs/gunbc/srv1-lab-admitted` | lab source | local | read-only | `dashboard_source_anchor_matches` |
| P13 | `env -C / /usr/bin/nc -z 127.0.0.1 18081` | lab listen port | local | read-only (TCP connect only) | `dashboard_apply_preflight_after_reconcile` |

(R = `/opt/gunbc-lab/codex-provider-feedback-v0`.)

**Pure steps.** `dashboard_instance_paths_are_owned`, `dashboard_peer_isolation_ok`, `dashboard_instance_reconcile` and the plan construction are pure. **No plan op is dispatched.**

### Writes

**None in the model.** There is no `Filesystem.Write*`, mkdir, install, mktemp or rm on this route. The traced closure (465 functions) reaches no write primitive other than `shell.Exec.RunArgvStdin`, inside `dashboard_target_run`'s `TargetExecWithStdin` arm, which preflight never selects; it uses only `TargetProbe` and `TargetExec`.

**Incidental writes by external programs, not modelled.**
- `git rev-parse` writes nothing in normal operation.
- codex may write under CODEX_HOME.
- ssh may append to `~/.ssh/known_hosts`, depending on the user's `StrictHostKeyChecking` setting.

### Network summary

1. SSH to `srv1`: two sessions (P1, P2).
2. HTTP GET `http://127.0.0.1:8080/healthz`, executed on srv1 through ssh.
3. HTTP GET `http://127.0.0.1:18081/instance.json`, local.
4. TCP connect to `127.0.0.1:18081`, local.
5. No other network access. codex `login status` may contact its provider; not modelled.

### UNRESOLVED (P)

- **P-U1. SSH destination.** The remote user and host that `ssh srv1` resolves to (ssh config/alias), and whether that is the same machine, are not in source. The `/opt/gunbc/gunbc` git read and the `:8080` curl run on whatever `srv1` names.
- **P-U2. Dispatch platform executables.** The P6 executable list from `dashboard_instance_dispatch_platform_executables` (dispatch capability realizations) was not expanded to literals.
- **P-U3. Shell-safe word refusal.** `typed_argv_shell_safe_refusal_argv` may refuse a probe word, which turns the probe into a typed refusal and runs nothing. The literal argv above contains no unsafe characters as read, but the predicate was not evaluated.
- **P-U4. Interpreter route only.** Same as U4-residual.

## Method

1. Read `srv1_lab_dashboard_apply_wet` down through `dashboard_instance_apply`, the preflight, `dashboard_instance_apply_after_preflight`, `dashboard_apply_directories_then_plan` and `dashboard_apply_provision_op`.
2. Read `dashboard_instance_provision_plan` and `dashboard_instance_reconcile` for the plan, and `srv1_lab_dashboard_instance` / `isolated_dashboard_instance_admitting` for the instance values.
3. Followed every arm into these modules: `roadmap_dashboard_instance_observe`, `roadmap_dashboard_target_write`, `roadmap_attempt_state_layout_move`, `executor_privileged_operation`, `extdeps.sudo.elevation`, `extdeps.tools.{findutils,chmod,chown,gnu_coreutils,env}`, `extdeps.git`, `gunbc.owned_directory`, `gunbc.live_deploy.serve_binary_build`.
4. Cross-checked with a name-resolving reachability script (scratchpad `tr/trace.py`, output `tr/out.txt` and `tr/seen.txt`) that greps reached bodies for effect primitives, absolute program literals, and the words sudo, systemctl, systemd-run, chmod and chgrp.

## Limits (stated plainly)

- **Method.** Name-based reachability plus hand reading:
  - `tr/trace.py` resolves calls by imports and same-module definitions only;
  - lambdas and method-call forms were followed by name;
  - every effectful site listed here was read by hand at 26fe4b8f350a.
- **This is not a completeness claim.**
- **Read-only.** Static analysis only; nothing was executed.
- **Host state is runtime-only.** Which steps fire depends on the reconcile result, the fence reading, the directory listings and the repository state.
- **PATH-resolved primitives** (the source fixes no PATH; they resolve in the caller's environment):
  - apply: `env`, `git`, `find`, `chgrp`, `chmod`, `mktemp`, `ln`, `rm`, `dd`;
  - convergence entry (section A): `whoami`, `git`, plus the inner `git` of the `--upload-pack` string.
- **Seed interpreter route only.** Process and file realizations were read from the seed interpreter; an emitted native binary's rendering was not inspected.
- **Upstream behaviour is not modelled.** Behaviour of external programs (git internals, the codex CLI, sudo policy, systemd dependency propagation) is outside the trace.
