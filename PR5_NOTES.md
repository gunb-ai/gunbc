# PR5 notes (loyal-bear-900) — remote-build serving-setup converger + apply

## Status / base
- Base: `session/proud-moth-338` tip `878fd2a55cb6` (PR4 model, NOT merged; model surface declared STABLE by proud-moth-338).
- gunbc worktree reset --hard onto PR4 tip. ctrl worktree gitdir is BROKEN (main repo /home/briansrls/ctrl absent) — flag; task is gunbc-only so far.
- PR4 confirms: srv2 ledger switch is MY actuation (switch when srv2 has no builds in flight); read `host_capacity_home_directories` per host; roster = srv1,srv2,srv3,srv4 ONLY (butch/srv15/Supermicro NOT now). srv3/srv4 width 14->13,16->15 (carved slot). Shared-slice compile.reconcile is SLOW now (memory pressure) — run witnesses via existing seed binary, background, 5-10min; NO local release build.

## Roster (remote_build_hosts.dag)
remote_build_serving_hosts() = srv1,srv2,srv3,srv4 (HostIdentity). remote_build_host_is_serving(host).

## Serving instance (remote_build.dag remote_build_serving_instance(host))
- account home = fleet_posix_remote_build_home()  (=/home/gunbc-build, TBD confirm)
- instance_root = persistent_instance_scope_root(RemoteBuildServing) = <account_home>/.local/share/gunbc-remote-build
- repo_root    = <account_home>/gunbc
- toolchain    = cargo_home <account_home>/.cargo, rustup_home <account_home>/.rustup  (fleet_compute_hermetic_toolchain / IsolatedJobToolchainHomes)
- serve_binary = <instance_root>/bin/gunbc
- materialization_store_root = PersistentInstanceWorkspace{ scope: RemoteBuildServing } => root = instance_root
- instance_id = "remote-build-<host>"; deployment = deployment_id(instance_id)

## Converge architecture (how a converger plugs in) — MODEL AFTER runner_host_file_converge (5402 lines; witness 2365)
- NOT HostConverge knobs (that's slice/property policy via `gunbc converge --mode plan|apply`).
- A converger module exports wet entrypoints: `<feat>_observe_ci_wet` (plan) + `<feat>_converge_ci_wet` (apply) (+ optional `_cap_release` etc). These run `gunbc run --module <m> --function <f>` as a CI step over FleetSsh.
- Wiring:
  - runner_ci_step.dag: `data <feat>_target: GunbcRunStepTarget = gunbc_run_step_target(module, function)`
  - fleet_converge_workflow.dag: `FleetConvergeWorkflowMode` enum arm(s) (Observe + Converge/Apply) + mode-facts (wire name, release_product, ssh_key_demand: FleetSshKeyConsumed, mutation_domain: ExecutorDomain, job_id, is_spark_target) + step builders + `*_step_if` conditions.
  - grants: file_access.dag / auth/privileged_effect_registry.dag must admit the SSH commands my apply runs.
  - SSH key demand: FleetSshKeyConsumed (which key? gunbc-remote-build key for the probe; converge itself likely runs as operator/executor over fleet ssh).
- Primitives to use: gunbc.typed_argv_exec::typed_argv_exec_over_fleet_ssh (returns Converged/Refused); gunbc.fleet.fleet_known_hosts_anchor::{anchored_host_key_row, fleet_ssh_target_at_standard_port, ssh_target_wire, SshTarget...}; gunbc.fleet.fleet_ssh_locus::prepare_fleet_ssh_agent_context; extdeps.exec.command::argv_words; gunbc.command_runner::run_shell_command_observe_outcome; gunbc.typed_remote_file_write; extdeps.posix.getent::{getent_passwd_command,...}; extdeps.tools.stat::stat_default_report_command.

## Desired state per host (the converger's model) — 8 components
1. group gunbc-compute gid 1006, members: host ledger service user (briansrls on srv1; per-host) + gunbc-build.
2. user gunbc-build uid 1005 primary gid 1005, supplementary gunbc-compute, systemd linger on, no sudo, NOT in any root-equivalent group (serve entry refuses otherwise).
3. ledger /opt/gunbc/compute/capacity root + areas per host_capacity (areas <service user>:gunbc-compute 03770). PR4 host_capacity_home_directories demand is the source. srv2 switches from old /home/briansrls per-instance ledger to here — actuate only when srv2 has no builds in flight.
4. rustup toolchain under <account_home>/.rustup and .cargo (the version the seed builds with).
5. checkout <account_home>/gunbc of public repo, https clone, detached at admitted revision.
6. seed binary built from that checkout (cargo build --release -p gunbc -> target/release/gunbc) installed at <instance_root>/bin/gunbc, host arch (srv1 aarch64; CHECK others), kept at a declared revision. Freshness rule: srv1's is still from 532fbb2 and must move with it.
7. store: <instance_root>/materialization-store and materialization-store-staging, 0755 gunbc-build:gunbc-build (provider refuses if root missing).
8. <account_home>/.ssh/authorized_keys mode 0600 containing EXACTLY the rendered forced-command line for that host (render via remote_build_serve::remote_build_authorized_keys_lines / remote_build_installable_line; a host whose line is unrenderable refuses).

## srv1 ADOPT not rebuild: srv1 already has all this; plan must show no-op or only drift it finds.

## EVIDENCE OWED
- witnesses for desired-state derivation with a discriminating RED CONTROL (broken input => refuse/differ). Run via existing gunbc seed binary: `gunbc run --source-root dag --source-root src/v2 --entry <witness> --claim-run`, background, 5-10min. NO local release build in sessions slice.
- plan receipt per host (observe mode).
- after clever-raven-401 GO: applied receipts, then probe per host: `ssh -i ~/.ssh/gunbc-remote-build gunbc-build@<anchored address> -- remote-build-v1 probe` answers "remote-build-v1 ending=ready", exit 6.
- NOT authorized to merge. Tell clever-raven-401 when approved+green.

## TODO / open
- [ ] confirm fleet_posix_remote_build_home value + fleet_posix_accounts desired-state fns (uid/gid/user/group/supplementary/ssh dir).
- [ ] read remote_build_serve authorized_keys_lines + installable_line + serve-time principal check.
- [ ] find smallest complete converger as literal template (host_credential_custody_converge? gpu_power_cap? a getent/user one).
- [ ] find the SSH key + anchored addresses for srv1-4 (fleet_known_hosts_anchor) + the seed binary path to run witnesses.
- [ ] check for existing PR5 scaffolding/branch.

## Interpreter for witnesses (CONFIRMED working)
- PATH `gunbc` shim REFUSES (needs target/release/gunbc in THIS checkout; won't substitute ref).
- Stale /home/briansrls/gunbc/target-main (a0c17c7, Oct12) is TOO OLD to parse current corpus (procfs.dag:10511 FatArrow).
- USE: /home/briansrls/.worktrees/gunbc/quiet-gull-101-target-b092/release/gunbc (2026-11-15, arm64, parses my corpus). Also target-merged2 (b092f145/12:44), target-main (041051dd). 
- Run: <interp> run --source-root dag --source-root src/v2 --entry <witness.dag> --claim-run  (background, reconcile ~minutes; shared slice is slow).
- Do NOT `cargo build --release -p gunbc` locally (that's the host seed binary; heavy + remote=wrong-arch). v1-compiler interp borrowed from quiet-gull builds.

## DSL idioms learned (for writing .dag — validated by clean frontend/normalize)
- module path = file path under dag/. `import <mod> { syms }`. All top-level fn/data/type importable.
- Builtins (DO NOT import; importing from std.types FAILS): filter, map, all, any, concat, join. Positional: filter(xs,pred), map(xs,fn), all(xs,pred), any(xs,pred), concat(a,b).
- Present/Absent come from `std.optional` (NOT std.types). Optional sugar `T?` on any T (use for optional fields/returns).
- List head: `first_where(xs, _ => true)` from `std.list` (returns T?). NO `at_index` std fn. NO `.length` — use count(xs)/xs.length() (extdeps) or any(xs,_=>true) for nonempty.
- PRODUCT type: `type X { a: T ... }`; CONSTRUCT with braces + COMMAS + trailing comma: `X { a: v, b: v, }`. Bare record literal `{ a: v, ... }` ok when return type known.
- SUM type: `type S = Arm1 | Arm2 { f: T } | ...`. Cross-module MATCH: import the ARM names (like Present/Absent from std.optional), NOT the parent. Match arms separated by NEWLINES, no commas; bool match `match b { true => .. false => .. }` valid.
- Field access `x.f`. Named args required for module fns: `f(name: v)`. Coerce NonEmptyStr->String with `as String` (NOT implicit). Int literal compare `==`/`!=` ok. `&&`, `not` seen; avoid `||` (unverified).
- test claim form: `test fn name() -> Bool` (runs under --claim-run). Helper `fn` allowed. Bool false => exit 1.
- PosixUser{name:NonEmptyStr,uid:Int,gid:Int,shell:PosixLoginShell}; PosixLoginShell=LoginShellObserved{path:NonEmptyStr}|LoginShellUnobserved. PosixOwnerNames{user:NonEmptyStr,group:NonEmptyStr}.
- fleet_posix_remote_build_user: name gunbc-build uid 1005 gid 1005 shell /bin/bash. group gunbc-compute gid 1006. supplementary [gunbc-compute]. home /home/gunbc-build.
- remote_build_serving_instance(host): fields instance_root, repo_root, serve_binary, directory_owner, materialization_store_root, admitted_revision=fleet_admitted_revision_source. store dirs = instance_root+"/materialization-store" & "-staging".

## Witness status
- remote_build_serving_setup.dag (model) + remote_build_serving_setup_witness.dag (9 test fns incl red controls). FRONTEND+NORMALIZE CLEAN. Reconcile in progress (slow, ~10min under load, ~3GB).
- Witness run: <interp b092> run --source-root dag --source-root src/v2 --entry dag/test/claim/compute/remote_build_serving_setup_witness.dag --claim-run

## Manager directive (2026-11-16 ~01:27) — SCOPE + ROUTE (authoritative)
- FLEET-SIDE. Host mutation ONLY via fleet-converge workflow. Sessions do NOT hold the fleet key.
- Build as a FleetConvergeWorkflowMode arm beside RunnerHostFileObserve/Converge: observe (plan receipt, read-only, may run PRE-GO) + converge (apply).
- Split: PR5(a) = model + OBSERVE-only mode (consumer = the observe receipt). PR5(b) = converge mode + apply. Neither merges with nothing consuming it.
- srv1-4 have enrolled anchored rows (fleet_known_hosts_anchor anchored_ssh_target / anchored_host_key_row).
- Admin principal: runner_host_file_converge administrator_ssh_target + fleet_bootstrap_principal executor_bootstrap_principal; context via fleet_ssh_locus prepare_fleet_ssh_agent_context.
- I (session) run the PROBE readback after apply: `ssh -F none -i ~/.ssh/gunbc-remote-build gunbc-build@<anchored addr> -- remote-build-v1 probe` → "remote-build-v1 ending=ready" exit 6.
- If workflow can dispatch observe at MY branch ref, do pre-GO receipts; if main-only, tell manager (sequence merge-then-observe).

## Observe entrypoint recipe (PR5 a) — primitives confirmed
- host: HostIdentity comes from the CI dispatch (subject host). Read same env as sibling observe (expected host); NOT the roster.
- target: `administrator_ssh_target(host: host)` (runner_host_file_converge) -> SshTargetResolution{Resolved target | Refused}. Or build SshTarget { address: anchored_host_key_row(host).address, principal: executor_bootstrap_principal.login }.
- context: `prepare_fleet_ssh_agent_context(attempt_raw: String)` -> FleetSshContextReady{context, receipt} | FleetSshContextRefused{cause}.
- leg: `typed_argv_exec_over_fleet_ssh(target, context, argv: List<String>)` -> TypedArgvExecConverged{result} | TypedArgvExecRefused{reason}. (result has .status/.stdout — see runner_host_file_ci_wet_hosted parsing.)
- wet entry shape (mirror runner_host_file_observe_ci_wet -> ci_wet_hosted): resolve host -> context -> observe legs (getent passwd/group, stat, cat authorized_keys) -> ServingObserved -> serving_plan(desired, observed) -> write receipt file (print body) -> ExitSuccess/failure.
- Wire: gunbc_run_step_target(module=gunbc.compute.remote_build_serving_converge, function=remote_build_serving_observe_ci_wet) in runner_ci_step; FleetConvergeWorkflowMode arm RemoteBuildServingObserve + mode-facts + step builders + step_if in fleet_converge_workflow. GRANTS: read-only fleet-ssh observe effect must be admitted (file_access / privileged_effect_registry) — UNVERIFIED how much; the read-only observe may need less than apply.

## Current committed/local state
- Committed 0ddb333: model (paths/account/ledger/ak) + witness 10 claims (all red controls green once the ledger-area claim fixed).
- UNCOMMITTED (needs ONE verify run when slice clear): model += ServingObserved, ServingPlanItem, serving_plan, serving_compare_int/present, serving_membership_verdict, serving_authorized_keys_verdict, serving_plan_settled/refused. Witness += ADOPT (matching→settled) + 3 red controls (wrong uid Drifted, blank→Missing not settled, non-roster refuses). ~14 test fns.
- NEXT after green: write remote_build_serving_converge.dag observe wet entry + workflow arm + CI target + grants. Then pre-GO plan receipts (if dispatchable at branch ref) → manager → (PR5 b) apply → probe.

## CHECKPOINT (2026-11-16 ~01:45) — awaiting manager steer on Q1
- Pushed origin/session/loyal-bear-900 (db5f4c8 + 0ddb333). Tree clean.
- AWAITING clever-raven-401: Q1 observe receipt = presence+exact-ak (no parsing; ServingObserved reshaped presence-oriented for account/group) OR value-compare uid/gid/members (needs getent/id parsers; but NO std split/string->int -> heavy). My stated lean: presence+exact-ak for (a).
- AWAITING clever-raven-401: Q2 can fleet-converge dispatch RemoteBuildServingObserve at session/loyal-bear-900 ref, or main-only? (model: admit_workflow_checkout_refs / RunScopedMainRoute in branch_dispatch_federation).
- AWAITING proud-moth-338 'clear' (they run seal/account/runner witnesses on the shared slice).
- NEXT once Q1 answered: reshape ServingObserved (+witness) per answer, write gunbc.compute.remote_build_serving_converge observe wet entry (actions_variable_read expected_host -> administrator_ssh_target -> prepare_fleet_ssh_agent_context -> typed_argv_exec_over_fleet_ssh legs -> ServingObserved -> serving_plan -> print plan receipt -> ExitSuccess), verify in a coordinated window. Then FleetConvergeWorkflowMode arm + CI step target + grants. THEN pre-GO receipts (or merge-then-observe per Q2) -> manager gate -> PR5(b) apply -> probe (I run probe myself).
- No host mutation until manager GO.

## CHECKPOINT 2 (2026-11-16 ~01:59) — PR5(a) CODE-COMPLETE, pushed; blocked on slice for witness run
Commits on session/loyal-bear-900 (on PR4 tip): aece53fc5f07 (readback+parsers+witnesses),
db4ba4718a4d (observe-mode workflow wiring). Pushed: origin/session/loyal-bear-900 = db4ba4718a4d. pre-push fmt clean.
Manager (msg_4d6c8a8c) told code-complete + wiring approach + blocked-on-slice. No apply until GO (PR5b).

TO RUN when proud-moth pings 'clear' (one closure-warm window, ~9min each):
  SEED=/home/briansrls/.worktrees/gunbc/quiet-gull-101-target-b092/release/gunbc
  $SEED run --source-root dag --source-root src/v2 --entry dag/test/claim/compute/remote_build_serving_setup_witness.dag --claim-run
  $SEED run --source-root dag --source-root src/v2 --entry dag/test/claim/compute/remote_build_serving_converge_witness.dag --claim-run
Capture PASS lines + the red-control FAILs (they assert Drifted/Missing => those cells return true=PASS; the "FAIL" to record in PR body is the DISCRIMINATING red controls behaving). If parse/type error -> fix + rerun.
Then open PR5(a) (gh) with witness lines in body; base onto session/proud-moth-338 (PR4) or main once PR4 merges.

Wiring facts (all mirror RunnerHostFileObserve): RemoteBuildServingObserve arm+facts(FleetSshKeyConsumed/ExecutorDomain/shared-job)+roster+step_if+step(fleet_converge_runner_host_file_step)+runner_ci_step target/invoke. Main-only (not in branch-ref route). Derived shared_job_if/checkout_trust auto-include; key-agent gating derived from ssh_key_demand.

## REGEN of fleet-converge.yml (manager required; .yml is GENERATED, heal doesn't regen)
Header: "Generated by gunbc.fleet.fleet_converge_workflow expected_fleet_converge_yml; regen via gunbc.instruments.generated_artifact_gate main_wet".
Use main_wet_one (ONE artifact, safer than main_wet's 35 — main_wet can resurrect deleted files):
  SEED=/home/briansrls/.worktrees/gunbc/quiet-gull-101-target-b092/release/gunbc
  $SEED run --source-root dag --source-root src/v2 \
    --entry dag/gunbc/instruments/generated_artifact_gate.dag \
    --function main_wet_one --arg path=.github/workflows/fleet-converge.yml
  (NO --claim-run; it WRITES the file via Filesystem.Write. seed binary supports --arg.)
WARN: peak RSS ~8.4 GiB (resolve pays full cost regardless of roster). MUST be sole heavy run on the
shared slice or it OOMs (docs say main_wet is SIGKILLed under pressure). Run LAST, after witnesses.
After regen: `git diff .github/workflows/fleet-converge.yml` must show (a) remote_build_serving_observe
in the mode `options:` list, (b) a step remote_build_serving_observe w/ its if-gating. Then git add+commit
the .yml. PR body: state regenerated via main_wet_one(path=.github/workflows/fleet-converge.yml) from
gunbc.fleet.fleet_converge_workflow expected_fleet_converge_yml; note main_wet_one chosen over main_wet.
Run order in the clear window: setup witness, converge witness, THEN regen (heaviest last).

## 5(b) DIRECTIVE (clever-raven, msg 74393a8a): rostering srv3/srv4 fires their runner-width carve (committed runner width -1 slot, gated on rostered && no dashboard instance; PR4 carves nothing). So admitting to remote_build_serving_hosts is a RUNNER-side change => the per-host plan MUST also converge the runner population via runner_host_file_converge, else host keeps old width while ledger holds the slot. Fold runner converge into 5(b) per-host plan.

## STATUS @ ~05:50 — review 95816 ADDRESSED, awaiting re-review
- Head 587282e. Both 95816 findings fixed: (1) unread leg -> GoalIndeterminate + observe REFUSES (exit failure),
  confirmed-absent stays GoalDiverged; (2) removed ServingItemVerdict coproduct, reuse std.goal_assessment::GoalAssessment
  (ObservationEstablished/Refused + assess_by_deviations_and_unknowns, mirror machine_intake.host_resource_observation).
- serving_observed_from_reads now takes ObservationAttempts; parsers kept (called by serving_assess with Present).
- Witnesses: setup 18/18 PASS (/tmp/sw_setup2.log), converge 3/3 PASS (/tmp/sw_conv3.log) incl red controls.
- No fleet-converge.yml regen needed (observe --function sig + receipt path unchanged). tree clean.
- v2-bootstrap: run 43804460269 (prior head cba8a32) = SUCCESS; 43812646346 on 587282e = running (~40min full compile).
- Dashboard gate on 587282e: github_admissible true / CLEAN / no github decision; BUT approvals 0/1 + claude dashboard
  REQUEST_CHANGES open (reviews schedule-triggered; active=0, no re-review on new head yet). Need scheduled re-review -> APPROVE.
- Manager pinged re: whether to nudge re-review. NOT authorized to merge. 5(b) still to do (runner converge + roster grow + srv2 switch).

## STATUS @ ~05:52 — 5(a) APPROVED; 5(b) RE-SCOPED to converge MODE ONLY
- 5(a) #16567 APPROVED by claude review 95833 (no blocking findings). v2-bootstrap run 43812646346 on
  587282e still running. Hold merge until #16562 (PR4) merges (stacked). NOT authorized to merge.
- Manager sequencing fix (msg 139c2a33): roster is SOURCE -> admission NOT in an apply.
  * 5(b) = RemoteBuildServingConverge MODE ONLY (wet apply entry creates account/group/toolchain/
    checkout/seed/store/authorized_keys/ledger-areas + workflow step + regen). Roster UNCHANGED.
    Pure converge_decision witness. NO srv2 switch here.
  * per-host apply on main WITH manager GO (after 5(a) merges + observe receipts), then readback + probe.
  * 5(c) (small, later): add applied+readback-settled hosts to remote_build_serving_hosts; srv2 /home->/opt
    model switch (only when srv2 home exists + nothing in flight). Merging 5(c) FIRES srv3/4 carve ->
    immediately dispatch runner_host_file_converge for srv3/srv4 (with GO); name old-width window in body.
- Working on branch session/loyal-bear-900-pr5b (stacked on 587282e).
- 5(b) plan: add ConvergeAction/ConvergeDecision + converge_decision(d,a) to remote_build_serving_converge.dag
  (pure; writes ONLY GoalDiverged items' actions, Refuse on indeterminate/refused); witness; then wet
  entry remote_build_serving_converge_ci_wet (execute actions via sudo_elevate_argv over fleet ssh, re-observe
  -> require settled); then RemoteBuildServingConverge workflow mode arm + facts row + step + regen.

## 5(b) STATUS — paused at a design question (do not guess)
Branch session/loyal-bear-900-pr5b @ 587282e. Scope confirmed w/ manager:
- 5(b) = converge/apply MODE ONLY, roster UNCHANGED. 5(c) = roster grow + srv2 /home->/opt switch + srv3/4 runner converge (runner_host_file_converge) right after 5(c) merges.
Pure converge_decision is FULLY SOURCED for: account (uid/gid/home/group/supp), group.members, store.root/staging (0755 gunbc-build:gunbc-build), authorized_keys (0600 + rendered line). Params all from RemoteBuildServingDesired path fields.
UNRESOLVED (asked manager, msg): (1) git clone URL source for ~gunbc-build/gunbc (model has local_origin_root/source_seed/admitted_revision structs, no canonical https URL constant); (2) does apply run `cargo build --release -p gunbc` ON the host over fleet SSH (heavy) or is the seed binary shipped/pushed another way; (3) ledger areas: d.ledger_dirs is [root, objects, heads] — need stable item->path/action mapping (root normal, 2 areas service-user:gunbc-compute 03770).
PLAN once answered: add ConvergeAction coproduct + pure converge_decision(d,a) (writes only GoalDiverged items; GoalIndeterminate/Refused => refuse, no writes) to converge.dag; witness (diverged->writes, unread->NO writes red control); wet entry remote_build_serving_converge_ci_wet (execute via sudo_elevate_argv over typed_argv_exec_over_fleet_ssh, then re-observe->require settled); RemoteBuildServingConverge workflow mode + facts row + step + regen fleet-converge.yml (say regen in body); PR5(b) body. NO host mutation until GO (after 5a merges + main observe receipts).

## CHECKPOINT @ ~06:10 — 5(b) converge_plan committed; manager aligned
- PR5B_DESIGN.md is SCRATCH ONLY (untracked, per manager: plans live as .dag carriers / PR body, never tree). Read it for the full executable 5(b) spec.
- Commit 6910e9ce5716 on session/loyal-bear-900-pr5b (pushed): pure converge_plan(d, ServingAssessment)->ConvergeDecision in converge.dag ONLY. Tree clean otherwise.
- Manager (msg_a5265d1d) BINDING answers: (1) clone url = extdeps.github.github github_https_clone_url(repo: gunbc.repository gunbc_repository), clone as gunbc-build + detach at github.sha; (2) NO on-host build — install executor's $ROOT/target/release/gunbc via fleet_scp_upload_command, sha256 readback, refuse unless executor uname -m == target uname -m; rustup channel (rustup_toolchain_file_channel->1.93.0) installed on-host as gunbc-build own-leg generous timeout; (3) ledger = host_capacity_home_directories OwnedDirectoryTarget rows, identity=path, compare owner+mode per row, never positional.
- NEXT (fresh context best): (1) extend setup.dag ServingObserved+comparators: serve-binary sha256 vs executor file, checkout rev-parse HEAD==github.sha, target uname -m==executor arch, ledger owner+mode per row — each ObservationAttempt (unread=>GoalIndeterminate=>refuse). + setup-witness red controls. (2) ledger/checkout/binary/toolchain actions in converge_plan + converge witness. (3) wet entry remote_build_serving_converge_ci_wet (execute writes via sudo_elevate_argv over typed_argv_exec_over_fleet_ssh; scp+sha256+arch guard; rustup own-leg; re-observe require settled). (4) RemoteBuildServingConverge arm in fleet_converge_workflow.dag + step + REGEN .github/workflows/fleet-converge.yml (seed main_wet_one; say regen+network_install drift in body). (5) witnesses green via seed --claim-run; open 5(b) PR (base 5a branch). NO apply until manager GO (after 5a merges + main observe receipts).
- 5(a) #16567 APPROVED (claude 95833, non-blocking only); stacked on PR4 #16562; NOT authorized to merge; hold until PR4 merges. v2 poller j0b28b8ad watching run 43812646346 on head 587282e.
- Manager confirmed: re-review schedule-triggered; CI (v2-bootstrap) not the witness gate; local seed witness PASS lines are the evidence.

## (A) DECIDED (manager msg_b003e0ba) — deepen 5(a) observe; NOT yet implemented
Manager: ONE comparator, no fork. serving_assess(d, o, pin) with ServingPlanInput; the pin is the
executor's plan-time reading, taken once/run, handed to BOTH observe and apply. 5(b) adds only
converge_plan + execution. Rerun 5(a) witnesses; note the post-approval change in the PR; re-review follows.
I implemented the pure converge_plan on pr5b (6910e9ce) and STARTED the (A) observe but REVERTED the
partial setup.dag edits (kept pr5b at 6910e9ce, compiling) because the ledger compare hit an unplanned
integration:
  - ledger_dirs: List<OwnedDirectoryTarget> is a COPRODUCT: OwnedDirectoryDeclared{member, path:FilePath,
    owner: PosixOwnerNames{user:NonEmptyStr, group:NonEmptyStr}, mode: DeclaredDirectoryMode, group_adoption}
    | OwnedDirectoryManaged{directory}.
  - DeclaredDirectoryMode is an ENUM (DirectoryOwnerOnly / ...GroupTraverse / ...Setgid / EveryoneTraverse...),
    NOT an octal string. Per-row owner+mode compare vs a `stat -c '%U %G %a'` read MUST reuse the existing
    managed_directory/owned_directory mode<->octal mapping (and runner_host_file_converge's existing dir
    compare is the sibling to mirror) — do NOT hand-roll octal string compare.
Plan for (A) (do on pr5b stacked = single comparator; confirm merge landing 5a-amend vs 5b-with-5a with mgr):
  1. setup.dag: add ServingPlanInput{dispatched_revision, executor_arch, local_binary_sha256, toolchain_channel};
     ServingObserved +4 legs (serve_binary_sha256, checkout_revision, target_arch: ObservationAttempt<String>;
     ledger: ONE established-safe read -> newline "<path>|<user>|<group>|<mode>", missing dir = "|absent|absent|absent");
     ServingEvidence +3 optional; new comparators serving_expected_deviation(item,expected,leg) for sha256/revision/arch
     + serving_ledger_row_deviations(d, leg) reusing mode mapping; serving_assess(d,o,pin) folds them as unknowns+deviations;
     serving_observed_from_reads +4 args.
  2. converge.dag: serving_leg_sha256/rev_parse/uname/ledger_stat read helpers; wet observe reads the pin
     (local sha256sum $ROOT/target/release/gunbc, executor uname -m, github.sha, rustup_toolchain_file_channel)
     once + passes to serving_assess; converge_plan ARMS for checkout.revision->EnsureCheckout,
     serve_binary.sha256->InstallServeBinary (scp via fleet_scp_upload_command + sha256+arch guard),
     ledger->EnsureLedgerDirectory rows; host.arch has NO arm => unhandled => apply refuses (can't fix arch).
  3. witnesses (5a setup + 5b converge): new divergences + red controls (unread sha256/rev/arch/ledger =>
     GoalIndeterminate => converge_plan writes NOTHING; arch mismatch => unhandled => refused).
  4. workflow RemoteBuildServingConverge arm + step + REGEN fleet-converge.yml; PR body: witnesses + regen note +
     why 5(a) observe changed post-approval; network_install/#16334 disclosure; srv1-only roster.
NOTE mgr: whether (A) lands by amending 5(a) #16567 or via 5(b) (which carries setup.dag) is merge mechanics —
single comparator either way; tell me your preference before I push a 5(a) change.

## v2-bootstrap GREEN on 5(a) head 587282e (run 43812646346 = completed/success) @ ~06:40
5(a) #16567 CI witness gate passes on the GoalAssessment-refactor head. 5(a) = approved (95833) + v2 green + stacked on #16562.
pr5b @ 5fa2405 (2 additive commits: converge_plan + ServingPlanInput/serving_expected_deviation). Tree clean.
PENDING manager reply: (i) landing mechanics for (A) [amend 5(a) #16567 vs carry on 5(b)]; (ii) OK to key ledger compare off managed_directory/owned_directory DeclaredDirectoryMode<->octal mapping (mirror runner_host_file_converge dir compare). On reply -> wire serving_assess(d,o,pin) + 4 deep legs + converge arms + wet pin read + witnesses + workflow arm + regen + open 5(b) PR. NO apply until GO.

## 5(a) AMEND in progress (manager msg_0225bdda: amend #16567 with the observe change) @ ~06:55
Branch session/loyal-bear-900. Added 59e0e812e666: ServingPlanInput + serving_expected_deviation (additive, safe, compiles).
AUTHORITY located: gunbc.managed_directory declared_directory_mode_octal(DeclaredDirectoryMode)->String (= file_mode_octal over FileMode). ledger desired rows: OwnedDirectoryTarget = OwnedDirectoryDeclared{owner: PosixOwnerNames{user,group}, mode: DeclaredDirectoryMode, path}. So per-row compare: observed user==owner.user, observed group==owner.group, observed mode==declared_directory_mode_octal(mode).
GAP: runner_host_file_converge has NO dir stat/owner-mode compare (it is a FILE converge) — manager's "directory compare" pointer didn't resolve. Plan: read ledger rows via ONE established-safe leg `stat -c '%U %G %a' <paths...>` (missing dir -> per-path 'absent|absent|absent' sentinel so it stays drift not refused), compare owner.user/owner.group + declared_directory_mode_octal(mode). Asked mgr if a specific existing dir-compare helper exists; default = proceed with this.
REMAINING for 5(a) amend (big, do in a clear window, keep tree compiling each commit):
  1. ServingObserved +4 legs (serve_binary_sha256, checkout_revision, target_arch: ObservationAttempt<String>; ledger: ObservationAttempt<String>).
  2. ServingEvidence +3 optional (serve_binary_sha256, checkout_revision, target_arch).
  3. serving_ledger_row_deviations(d, leg) parse+compare via declared_directory_mode_octal; import gunbc.managed_directory { declared_directory_mode_octal }.
  4. serving_assess(d,o)->(d,o,pin): fold deep-leg unknowns + deviations (serving_expected_deviation x3 + ledger) into the existing assess_by_deviations_and_unknowns.
  5. serving_observed_from_reads +4 args.
  6. setup witness: new cells + red controls (unread sha/rev/arch/ledger -> GoalIndeterminate; wrong sha/rev/arch -> GoalDiverged). RERUN seed witness; update PASS lines in PR body + note post-approval change.
  7. PR #16567 body: why the observe changed after approval (manager msg_b003e0ba rationale: single comparator; 5a observe = GO evidence).
Then pr5b rebase onto amended 5(a): drop 5fa2405 (dup of 59e0e812); keep converge_plan (6910e9ce); ADD converge legs (sha256sum/rev-parse/uname/stat rows) + wet pin read + converge_plan arms (checkout.revision->EnsureCheckout, serve_binary.sha256->InstallServeBinary scp+guard, ledger->EnsureLedgerDirectory rows, host.arch->unhandled/refuse) + workflow RemoteBuildServingConverge arm + regen fleet-converge.yml + converge witness. NO apply until GO.
