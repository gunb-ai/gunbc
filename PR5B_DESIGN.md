# PR 5(b) — remote_build_serving_converge apply mode: FINAL spec (manager-confirmed 2026-11-16)

Branch `session/loyal-bear-900-pr5b` (stacked on 5(a) #16567 -> PR4 #16562 -> main).
SCOPE: converge/apply MODE ONLY. Roster UNCHANGED (remote_build_serving_hosts stays srv1).
No host mutation until manager GO (after 5(a) merges + main observe receipts land).
5(c) (separate, later, small): roster grow + srv2 /home->/opt switch + srv3/4 runner_host_file_converge.

## Manager decisions (msg_a5265d1d) — BINDING
(1) CLONE URL = extdeps.github.github `github_https_clone_url(repo: gunbc.repository gunbc_repository)`
    (same source as remote_build_serve remote_build_fetch_argv + PR3 client push).
    Clone as gunbc-build, then `git -C repo_root checkout --detach <github.sha>`.
    NEVER read a URL off another checkout.
(2) DO NOT build the seed on the host. The fleet-converge job already builds gunbc at github.sha
    on the executor: $ROOT/target/release/gunbc (built by the same workflow on the host input).
    Install THAT binary: scp via gunbc.fleet.fleet_known_hosts_anchor `fleet_scp_upload_command`,
    move into <instance_root>/bin/gunbc, read back its sha256 vs the local file.
    GUARD: refuse unless the executor's `uname -m` EQUALS the target's `uname -m`.
    Freshness rule = serve binary == the dispatched main revision (github.sha).
    Checkout detaches at that SAME github.sha so binary + dag agree.
    Rustup pinned channel (extdeps.rust.rustup rustup_toolchain_file_channel -> 1.93.0) is STILL
    installed on the host as gunbc-build (the provider builds the subject's gunbc), in its OWN leg
    with a GENEROUS timeout, and STATE that timeout in the workflow step. NO release build over SSH.
(3) LEDGER = key off host_capacity_home_directories OwnedDirectoryTarget rows (identity = path);
    compare owner AND mode per row. NEVER index by position.

## Model changes (in 5(b), these ENRICH the 5(a) observe so it matches what apply enforces)
### RemoteBuildServingDesired (setup.dag) — additions:
- ledger dirs become the OwnedDirectoryTarget rows from host_capacity_home_directories (path, owner_user, owner_group, mode). Identity = path.
- checkout_revision: String  (= the dispatched github.sha the checkout detaches at + the binary was built at).
- toolchain_channel: String  (= rustup_toolchain_file_channel over rust-toolchain.toml; 1.93.0).
- executor_arch: String      (the fleet-converge executor `uname -m`; must equal target arch or refuse).
- serve_binary_source: FilePath = "$ROOT/target/release/gunbc" (executor-built; the scp source).
### ServingObserved (setup.dag) — additions (richer reads so observe feeds apply):
- ledger rows owner+mode per path (stat -c '%U:%G %a' per dir) instead of a single mode string.
- checkout_revision readback: `git -C repo_root rev-parse HEAD` (or rev-parse --short) vs desired.
- serve_binary_sha256 readback: `sha256sum <instance_root>/bin/gunbc` vs local $ROOT/target/release/gunbc sha256.
- target_arch readback: `uname -m` on target vs executor_arch.
- toolchain: rustup pinned channel present under rustup_home (test -d toolchains/<channel>...).
- each new read is an ObservationAttempt -> unreadable => GoalIndeterminate => refuse (never Missing).
### converge_decision(d, a) pure — ConvergeAction coproduct (params from d):
- EnsureAccount {name,uid,gid,group,home,supplementary}
- EnsureGroupMembership {group_name, group_gid, member}   (one per member of d.group_members)
- EnsureDirectory {path, owner_user, owner_group, mode}    (store.root/staging 0755 gunbc-build:gunbc-build)
- EnsureLedgerDirectory {path, owner_user, owner_group, mode}  (from host_capacity rows)
- EnsureAuthorizedKeys {path, owner_user, content}         (0600; content = d.forced_command_line, Outertamped in receipt)
- EnsureCheckout {repo_root, clone_url=github_https_clone_url(gunbc_repository), revision=d.checkout_revision}
- InstallServeBinary {local=$ROOT/target/release/gunbc, remote=d.serve_binary, expected_sha, expected_arch=d.executor_arch}
- EnsureToolchain {rustup_home, cargo_home, channel=d.toolchain_channel, account=gunbc-build}
- converge_decision emits actions ONLY for GoalDiverged deviations (matched by item, params from d).
  GoalIndeterminate or GoalAssessmentRefused => ConvergeDecision {writes: empty, refused: true, reason} (FAIL-CLOSED, no writes).
  GoalSatisfied => {writes: empty, refused: false}.
### Wet apply entry remote_build_serving_converge_ci_wet:
- prepare_fleet_ssh_agent_context; read FLEET_CONVERGE_EXPECTED_HOST.
- observe (remote_build_serving_observe_host_reads) -> assess -> converge_decision.
- if refused OR not runnable -> write receipt (plan) + exit_failure. (This is also the pre-GO plan receipt producer.)
- else EXECUTE actions via sudo_elevate_argv over typed_argv_exec_over_fleet_ssh (the impure exec list, mirroring runner_host_file_ci_converge_wet):
  groupadd/useradd/usermod/loginctl enable-linger (account/group, idempotent via `getent` guards);
  install -d -o -g -m (store/ledger); write authorized_keys 0600; git clone+checkout as gunbc-build;
  scp serve binary via fleet_scp_upload_command to a tmp, sha256 check, install into bin/ as gunbc-build;
  rustup toolchain install channel as gunbc-build (own leg, generous timeout).
- then RE-OBSERVE + require settled; residual deviation/refusal => exit_failure with terminal in receipt.
### Workflow (fleet_converge_workflow.dag) — add RemoteBuildServingConverge arm:
- facts row (scope:none, ssh_key_demand:FleetSshKeyConsumed, release_product:SixteenBinaryPack, mutation_domain:ExecutorDomain, shared job id, is_spark_target:false) + the release-gate handling matching RunnerHostFileConverge if needed.
- step reusing fleet_converge_runner_host_file_step; gate `if: mode == 'remote_build_serving_converge'`; FLEET_CONVERGE_EXPECTED_HOST=host; timeout generous; STATE the timeout.
- REGEN .github/workflows/fleet-converge.yml (say so in body + how: seed main_wet_one path=.github/workflows/fleet-converge.yml). Note: the pre-existing managed_host_network_install drift will also publish (already disclosed; owned by PXE-2c lane, cool-owl-838 archived, manager escalating to operator).
### Witness (pure, claim-run via seed binary):
- converge_decision on a diverged assessment -> the matching write actions present (e.g. group members missing -> EnsureGroupMembership rows).
- RED CONTROL: an assessment that is GoalIndeterminate (an unread leg) -> converge_decision writes NOTHING + refused=true (apply never writes an unobserved item).
- GoalSatisfied -> empty writes, not refused.

## Reuse / sources
- clone url: extdeps.github.github github_https_clone_url, product? gunbc.repository gunbc_repository.
- scp: gunbc.fleet.fleet_known_hosts_anchor fleet_scp_upload_command.
- ledger rows: gunbc.compute.host_capacity host_capacity_home_directories (OwnedDirectoryTarget rows).
- toolchain channel: extdeps.rust.rustup rustup_toolchain_file_channel (reads rust-toolchain.toml).
- ssh legs + sudo_elevate_argv: gunbc.coding_agent_ci_support_admin_ssh; typed_argv_exec_over_fleet_ssh: gunbc.typed_argv_exec.
- sibling wet executor model: gunbc.runner.runner_host_file_converge runner_host_file_ci_converge_wet.

## DONE checklist for 5(b)
[ ] setup.dag: extend desired + observed + comparators (ledger owner+mode, checkout rev, binary sha, arch, toolchain channel).
[ ] setup witness updated (new divergences + their red controls).
[ ] converge.dag: ConvergeAction + ConvergeDecision + pure converge_decision; converge witness (diverges->writes, indeterminate->no writes refused).
[ ] converge.dag wet: remote_build_serving_converge_ci_wet.
[ ] fleet_converge_workflow.dag: RemoteBuildServingConverge arm + facts row + step + registration.
[ ] runner_ci_step.dag: converge invoke constants.
[ ] REGEN fleet-converge.yml (seed main_wet_one); say regen in body.
[ ] witnesses green (seed binary --claim-run).
[ ] open 5(b) PR (base = 5(a) branch), CI dispatch, body with plan/receipts/GO gating.
[ ] NO apply dispatch until manager GO (after 5a merges + main observe receipts).
