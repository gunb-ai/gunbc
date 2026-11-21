# Handoff: runner host cap bracket on live hosts (sleek-koi-601, work item node://adhoc-c17ab64c-0d5)

Parked 2026-11-21 by operator priority (remote-build bandwidth first). Everything below was written by the
session that held the work item; run IDs are github.com/gunb-ai/gunbc Actions run numbers.

## 1. State of the work item
# Acceptance evidence — runner host cap bracket on live hosts (node adhoc-c17ab64c-0d5)

All runs: workflow `fleet-converge.yml`, repo gunb-ai/gunbc, dispatched by hand (`workflow_dispatch`).
Before #16813 the drain trio queued in `gunbc-host-mutation-<executor>` (srv1); after it they queue in
`gunbc-host-mutation-<subject>`.

## srv4 — COMPLETE (width 15, srv4-16..21 retired)

| step | run id | outcome |
|---|---|---|
| unit fragment + sudoers converge | 44227331401 | success (pre-existing work) |
| withdraw (drain + mask) | 44436062310 | success |
| runner_host_file_converge (caps) | 44445154092 | success: observed=21 effective=15 retired=6 |
| runner_host_restore | 44469444478 | success: 15 slots re-admitted |
| runner_host_file_observe (host=srv4) | 44471586807 | observed=21 effective=15, 15 slots admission=open |
| sudoers narrowed to srv4-16..21 converge | 44570819203 | success |

srv4 serving converge (bright-moth-473, lane 1) 44782848445 success at 17:40Z — srv4 group released.

## srv3 — in progress (width 13, srv3-14..21 retired)

Chain attempts (all BEFORE the settle leg / subject-domain split):

| step | run id | outcome |
|---|---|---|
| baseline observe | 44454553045 | observed=21 |
| sudoers converge | 44583952053 | sudoers written; caps WITHHELD 'surplus slots have not retired' |
| withdraw | 44595938568 | 13 kept slots masked |
| caps converge x2 | 44593179957, 44597651034 | ACTIVATION-HELD-BY-WITHDRAWAL |
| restore x2 | 44603235081 (failure), 44606128349 | 'success' |
| observe | 44609820213 | observed=21 effective=0 retired=8; srv3-06/07 REFUSED 'slot tree absent while active' |
| caps converge | 44621007740 | REFUSED: srv3-06/07 slot-tree absent while activating → template cap phase never settled → 11 kept slots ALLOCATION-NOT-EFFECTIVE (CPUQuotaPerSecUSec=5s want=9000000us), set-property shadows not removed |

Lane order for srv3 (manager ruling msg_cd77f37a): (1) serving converge bright-moth-473 — DONE 44779656976
green, END ping; (2) tailscale join silent-crab-397 — DONE 44782572818 SUCCESS (srv3 FIRST join: NEW node
identity/host key, now `srv3.tailecbe08.ts.net`), END ping; (3) caps chain sleek-koi-601 — AFTER #16796 (settle
leg, merged 7ff6979940ba) and AFTER #16813 (subject-host concurrency group).

Post-#16813 chain (to be filled in):

| step | run id | outcome |
|---|---|---|
| withdraw | | |
| runner_host_file_converge | | expect the settle leg to run for srv3-06 and srv3-07 |
| runner_host_restore | | |
| runner_host_file_observe (host=srv3) | | expect observed=21 effective=13 retired=8, 13 admission=open |

## Findings that belong in the acceptance note

1. srv3-06 and srv3-07 are masked-but-kept with no tree on disk and no hold in the model or run history:
   nothing in the model, and no run of mine, ever stopped or masked them before their trees went missing.
   They are the two slots the settle leg (PR #16796) exists for.
2. srv3's tailnet identity changed between the baseline observe (44454553045) and the post-chain observe, because
   silent-crab-397's join 44782572818 was srv3's FIRST tailnet join (new node id / host key). Any host-level
   field diff between the two observes is expected, not a regression.
3. The ordering that kept the srv3 chain safe was convention only (lane order agreed in chat); #16813 is what
   makes it structural — the three drain modes now declare `TargetHostDomain { field: subject_host }`, so a
   drain of srv3 serializes in `gunbc-host-mutation-srv3` instead of srv1's group.
4. Watcher receipts for the settle leg must name srv3-06 and srv3-07 with `step=stop` + a read-back showing
   inactive before the first-install leg (rebuild_runner_slot_tree) runs.
5. The srv3 caps refusal had one root, not four. From run 44621007740 (fleet-converge job 135907412820):
   srv3-06/07 `REFUSED step=slot-tree slot tree absent while active` -> their cap `REFUSED step=activate ...
   readback ... ActiveState=activating; MainPID=` -> "the slot memory-cap phase has not settled the template cap
   drop-ins, so removing a set-property shadow could expose unadmitted caps" (retired-dropin refusals) -> all 13
   kept slots `ALLOCATION-NOT-EFFECTIVE CPUQuotaPerSecUSec=5s want=9000000us`. The quota/shadow symptoms are
   downstream of the two unsettled actuations; #16796's settle leg is the single fix, and my post-chain observe
   must show 01..13 at 9000000us. If it does not, the settle leg did not run and I stop rather than patch.

## srv3 'chain-unbounded' during a client compile -- my answer: my bracket work does not touch it
Runs (manager-dispatched, read-only except where stated): observe 44904559666 (drift: binary 8e12d5 vs pin a18110, tree 767af9 vs pin 9688a3, sidecar absent; linger on, slice capped at the derived 25 GiB / 24 GiB / swap 0) -> converge 44906587463 (settled; RecordServeIdentity's 134-byte upload proven in the log) -> observe 44907864146 (settled, no drift, independent runner slot).
Reading: the failing line is not a refusal. `standing_from_supplied` prints the observed-bound arm's reason and then continues to UserManagerOwed, i.e. wrap into gunbc-user-work.slice (src/v1/stage0/src/cli_run/user_work_scope.rs:489-495, wrap_now :300-344; the dag mirror at compute/unit_bounds.dag:244-246 says the label exists so a lost MemoryMax reads as "chain unbounded, wrapped" or "refused", not silence). Real wrap refusals print 'error: cannot wrap into gunbc-user-work.slice (<standing>)' and exit 2; the reported exit was 1.
swift-tern-46's follow-up compile (client from their container, not a srv3 slot): srv3 and srv4 both RESOLVE (860/859 files) and refuse at 'no ledger instance is declared for host srvX' -- so srv3's serving state is genuinely current and the remaining blocker is the missing capacity-ledger row (keen's lane; adjacent to my still-open capacity-carve item), not the drain, not the cgroup arm.
My bracket writes only system-manager slot units and the runner host file; the user-work chain lives under user@1000.service. No overlap.

## srv4 re-observe, ~2 h after restore (persistence check)
run 44919144088 (mode=runner_host_file_observe host=srv4 subject_host=srv4, complete/success, 14 min, fleet-converge job 136892169571 on runner srv4-04-1795273018-4183697, 15:39Z):
'slots observed=21 effective=15 retired-as-declared=6'; 15 slots 'admission=open'; the six srv4-16..21 'retired-as-declared LoadState=masked'; caps 'observed-at-desired' for srv4-01..15; no ALLOCATION-NOT-EFFECTIVE line; 'manager-reload=OWED-NOT-ATTEMPTED observe-only' (observe writes nothing).
So srv4's modelled population, effective caps and re-admitted remaining slots are still true hours later and on a second independent read.

## 2. Working notes (chronological, includes rulings I was given and dead ends)

## 11:40Z root-cause chain of the srv3 caps refusal (run 44621007740, fleet-converge job 135907412820, log /tmp/srv3-caps-job.log)
In order, from that one run's receipt:
1. `actuation actions-runner@srv3-06.service REFUSED step=slot-tree slot tree absent while active: the tree is absent while the unit is activating`; same for srv3-07.
2. `caps ... srv3-06/07 REFUSED step=activate ... REFUSED step=readback not loaded, enabled, cleanly active with a main process after one settle: ActiveState=activating; MainPID=; main=none` -> the cap phase never settles.
3. therefore `retired-dropin path=/etc/systemd/system.control/actions-runner@srv3-13.service.d/50-Memory*.conf REFUSED not removed: the slot memory-cap phase has not settled the template cap drop-ins, so removing a set-property shadow could expose unadmitted caps`.
4. therefore all 13 kept slots read `ALLOCATION-NOT-EFFECTIVE CPUQuotaPerSecUSec=5s want=9000000us` -- the modelled quota drop-in (60-fleet-cpu.conf) was never written, so the live cgroup shows the stale 5s set-property shadow.
So items 3 and 4 are DOWNSTREAM of 1 and 2, not separate holds -- which is what clever-raven-401 ruled. The #16796 settle leg is the single fix. My post-chain observe must show 01..13 at 9000000us; if it does not, the settle leg did not run and I stop rather than patch.

## 12:30Z srv3 'user_work_scope: observed-bound arm did not admit (chain-unbounded)' -- answered to clever-raven-401 (msg_d08a3130)
Separate gap, NOT in my bracket's reach. My writes are system-manager slot units; the failing chain is the USER manager's gunbc-user-work.slice (user@1000.service/gunbc.slice/gunbc-user.slice/gunbc-user-work.slice, live_deploy/srv1_worker_slice_receipt.dag:57). Refusing fold = observed_cgroup_bound (compute/unit_bounds.dag:232-243; wire 'chain-unbounded' :246-252) needs a POSITIVE hard memory limit on the materialised chain AND swap forbidden somewhere on it.
Candidate shapes: (1) slice unit installed but cgroup not instantiated -- the model states the cgroup exists only under a LIVE serving user manager, EnsureUserManagerLinger ranked ahead (compute/remote_build_serving_converge.dag:780-788); (2) ceiling derives to zero -- user_work_slice_memory_max_for_host (compute/host_capacity.dag:483-488): carved -> host_build_pool_reservation = gunbc_runner_slot_memory_max iff serving-roster AND no own dashboard instance (runner/runner_slot_desired.dag:243-259,264-269), else user_work_slice_memory_max_of(user_work_pool_slots()); a zero carve refuses by design. This touches my still-OPEN capacity-carve item.
srv3 serving converge succeeded 44779656976 (11-20T17:04Z); NO remote_build_serving_setup run in the last 200 fleet-converge runs. My chain's receipts will be silent on the user manager. Discriminator = read-only user-manager read (serving lane, not my modes).

## 13:20Z srv3 serving lane (manager-dispatched, off my main chain) -- DONE, and a correction to my own earlier claim
Runs (host=srv3 subject_host=srv3 --ref main; the serving pair allows subject==host under may_be_executors_own):
- 44904559666 remote_build_serving_observe (12:34Z, job 136850164277 on runner srv3-04, 16 min, success) -> terminal drift: serve_binary.sha256 want a18110efa98c…, have 8e12d5529863…; checkout.revision want 9688a3c90527…, have 767af989deb2…; identity sidecar CONFIRMED ABSENT. linger + slice-cap legs produced no drift line -> reading enabled and the slice unit carries the derived ceiling (25 GiB max / 24 GiB high / swap 0).
- 44906587463 remote_build_serving_converge (12:59Z, job 136854456852, 8 min, success). Step 127 success, step 128 (artifact upload) SKIPPED -> apply receipt not downloadable, text in the job log: 'remote-build-serving-plan: settled / all items met -- host adopted unchanged (no-op)' (that wording is GoalSatisfied, remote_build_serving_setup.dag:1387). Provable write: sidecar probe 'sudo -n test -f .../gunbc.identity' exit=1 at 13:08:04, then target/gunbc-serve_identity.upload 134 bytes at 13:08:09 = RecordServeIdentity fired (first time I have seen it). Binary/checkout legs are not named (successful ssh execs are logged without their commands).
- 44907864146 second read-only observe, started 13:14Z, confirms or prints want/have again. Logs: /tmp/srv3-serving-observe.log, /tmp/srv3-converge.log, /tmp/srv3-observe2.log.
CORRECTION (told manager + swift-tern-46): 'user_work_scope: observed-bound arm did not admit (chain-unbounded)' is NOT a refusal -- it is an eprintln on the fallthrough in src/v1/stage0/src/cli_run/user_work_scope.rs:489-495; the decision continues to UserManagerOwed and wrap_now() (:300-344, systemd-run into gunbc-user-work.slice, tighter of caller envelope/slice install). Real wrap refusals print 'error: cannot wrap into gunbc-user-work.slice (<standing>)' and exit 2. The forced command (compute/remote_build_serve.dag:497-515) has no wrap; the engine wraps itself. A chain-unbounded line with exit 1 = wrapped/exempt run that refused later with an unparseable stdout answer (the client appends whole stderr, compute/remote_build.dag:783). So the serve-time /proc/self/cgroup instrument would NOT explain that symptom; it is still butch/swift-tern lane's instrument but for other reasons.
Weak-evidence rows to cite whenever srv3/srv4 serving reads green: recurring_failure_mode.capability_arrives_outside_the_converged_path (srv4 'all items met' for a slice cap+linger no model path wrote) and integration_concern.serve_binary_identity_unvalidated_at_serve_time:30 (srv3 11-20 converge called InstallServeBinary a no-op, live gate refused that evening).

## 14:20Z planning state while the budget decision is open
- main = 3caeab3d9df6 (#16832). My #16813 head d02c73513457 is behind by 4 merges; delta 451835cca2da..main = 30 files, 11 inside BOTH witness half-closures (/tmp/main-delta-5.txt) -> both halves must be re-run at the merging head, as already planned.
- main's emitted fleet-converge.yml still reads timeout-minutes: 324 and RUNNER_HOST_CAP_STEP_BUDGET_SECONDS "6255" -> the derived raise (354/8055/180) is NOT on main's artifact: main's yml is stale against main's model, so ANY regen at current main carries the raise until the census is scoped (quiet-carp-163's lane) or the heal row reverts.
- Watcher jd407acdc polls the chain guard every 2 min (refuses until main's yml carries my three clauses) for ~3 h.
- Peers clear: swift-tern-46 released srv3 ("nothing of mine is pending on srv3"); their compile's only remaining blocker is 'no ledger instance is declared for host srv3/srv4' (keen's lane, adjacent to my open capacity-carve item).

## 16:25Z verified on main (no regression)
#16796 (7ff6979940ba) IS an ancestor of origin/main. Its production code lives in dag/gunbc/runner/runner_host_file_converge.dag (not runner_slot_retirement.dag), names: loop_quiet_or_refused, runner_slot_restart_loop_standing, runner_slot_loop_settlement_is_due, settle_unprovisioned_slot_loop_over (exact spelling; my earlier note wrote settle_unprovisioned_loop_over). main's copy of that file is byte-identical in size/profile to my worktree's (5723 lines). A truncated `head -12` on a token count briefly looked like the leg had vanished -- it had not; do not re-check this by token profile, use the exact names.
Also on main now: #16834 (checkout leg Met only on HEAD==pin read-back), #16835 (serving entry asks one authority who owns the host ledger -- clears swift-tern-46's 'no ledger instance' blocker), #16836 (split witness: butch is not anchored). design_ruling.dag blob unchanged (3b8f15c380af) as of 16:25Z; main's fleet-converge.yml still 324/6255 (the derived raise is not deployed).

## 18:25Z do not read a design_ruling blob change as the budget ruling
The blob moved 3b8f15c380af -> 1cbbe11e9580 because of #16838 (fabric-storage unit back into srv1 deployment membership, 18:01Z), not a budget row. No fleet-converge budget ruling row exists on main yet (rows present are still heal_convergence_job_budget_increase_2026_11_20, ci_job_budgets_closed_to_growth, drain_wait_bound_is_derived_from_declared_job_timeouts). Watcher jb6a09cda now watches the real gates only: main's yml carrying my three clauses (guard open), or 'timeout-minutes: 354'/8055 deployed (raise approved), or #16813 leaving OPEN. #16838 touched design_ruling.dag again -> both witness halves still owe a re-run at the merging head.

## 3. srv3 chain dispatcher (refuses unless main's emitted workflow carries the drain-trio clauses)
```bash
#!/usr/bin/env bash
# srv3 caps chain dispatcher (work item node://adhoc-c17ab64c-0d5).
# ONE LEG PER INVOCATION:  LEG=runner_host_withdraw SRV3_FIRE=1 SRV3_PEER_CLEARED=1 ./srv3-chain.sh
# Refuses unless: (a) main's emitted workflow already carries my three subject_host clauses -- i.e. #16813 is
# merged and its regeneration landed, so the drain trio really queues in gunbc-host-mutation-srv3;
# (b) SRV3_FIRE=1 (explicit) and SRV3_PEER_CLEARED=1 (swift-tern-46 answered my pre-fire ping).
# Executor for the three drain legs is srv1 (manager ruling n); the OBSERVE leg runs ON the subject host because
# observe is grouped and reads on its executor (manager ruling b).
set -u
REPO=gunb-ai/gunbc
SUBJ=srv3
LEG="${LEG:-}"
YML_PATH="repos/$REPO/contents/.github/workflows/fleet-converge.yml?ref=main"

case "$LEG" in
  runner_host_withdraw|runner_host_file_converge|runner_host_restore) EXEC=srv1 ;;
  runner_host_file_observe) EXEC=srv3 ;;
  "") echo "LEG= is required (one leg per run)"; exit 2 ;;
  *) echo "unknown LEG=$LEG"; exit 2 ;;
esac

# (a) the deployed artifact must already carry the change
yml=$(gh api "$YML_PATH" --jq .content | base64 -d)
missing=""
for m in runner_host_file_converge runner_host_withdraw runner_host_restore; do
  printf '%s' "$yml" | grep -q "'$m' && format('gunbc-host-mutation-{0}', github.event.inputs.subject_host)" || missing="$missing $m"
done
if [ -n "$missing" ]; then
  echo "REFUSED: main's fleet-converge.yml has no subject_host clause for:$missing"
  echo "         #16813 (or its regeneration) has not landed; this leg would queue in gunbc-host-mutation-srv1."
  exit 3
fi

# (b) explicit fire + the serving-lane peer's answer
if [ "${SRV3_FIRE:-0}" != "1" ] || [ "${SRV3_PEER_CLEARED:-0}" != "1" ]; then
  echo "DRY (needs SRV3_FIRE=1 and SRV3_PEER_CLEARED=1): gh workflow run fleet-converge.yml --repo $REPO -f mode=$LEG -f host=$EXEC -f subject_host=$SUBJ"
  exit 0
fi

# lane check: nothing of anyone's may be queued, and nothing may hold the subject group
gh run list --repo "$REPO" --workflow fleet-converge.yml --limit 30 --json databaseId,status,createdAt,displayTitle \
  | python3 -c "
import json,sys
busy=[r for r in json.load(sys.stdin) if r['status'] in ('queued','pending','in_progress','waiting')]
for r in busy: print('  busy:', r['databaseId'], r['status'], r['createdAt'], (r['displayTitle'] or '')[:78])
print('  (no busy rows above = lane clear)')
"

gh workflow run fleet-converge.yml --repo "$REPO" -f mode="$LEG" -f host="$EXEC" -f subject_host="$SUBJ"
sleep 20
gh run list --repo "$REPO" --workflow fleet-converge.yml --limit 1 \
  --json databaseId,status,createdAt,displayTitle --jq '.[0] | "\(.databaseId) \(.status) \(.createdAt) \(.displayTitle)"'
cat <<'NOTE'
Poll this run to completion before the next leg. If it sits queued or pending behind anything, cancel it
(gh run cancel <id> --force) -- a queued run is never left behind. Receipt to read for the converge leg: the
settle steps for srv3-06/07 must name step=stop with an inactive read-back BEFORE rebuild_runner_slot_tree; if
either unit reads LoopLive at the mask/actuate step, REFUSE and report, do not hack around it.
NOTE
```

## 4. Post-approval sequence for PR #16813 (merge, engine, regen, verify, commit, two witness halves)
```bash
#!/usr/bin/env bash
# Run ONLY after the operator approves the derived fleet-converge raise, the
# design_ruling row lands on main, and clever-raven-401 says go.
# Stages are separate so each can be checked before the next.
#   ./go-chain.sh merge | regen | engine | witness-job | witness-wf | verify
set -uo pipefail
export PATH=/opt/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin
WT=/home/briansrls/.worktrees/gunbc/sleek-koi-601-domain
SEEDWT=/home/briansrls/.worktrees/gunbc/sleek-koi-601-seed-398136026b3
YML=.github/workflows/fleet-converge.yml
cd "$WT"

case "${1:-}" in
merge)
  git fetch -q origin main
  git merge --no-edit origin/main            # NEVER rebase (manager ruling g)
  git log --oneline -1
  ;;
engine)
  # build the engine at the CURRENT HEAD of this worktree, local mode (arm64)
  git rev-parse --short HEAD > /tmp/go-engine-head.txt
  cd "$SEEDWT" && git fetch -q origin
  git checkout -q "$(git -C $WT rev-parse HEAD)"
  CTRL_BUILD_MODE=local RUSTC_WRAPPER= CC=cc CARGO_TARGET_DIR=/cargo-target \
    ctrl-build -- cargo build --release -p v1-compiler --bin gunbc
  /cargo-target/release/gunbc --version
  sha256sum /cargo-target/release/gunbc
  ;;
regen)
  # ~29 min, ~8.4 GiB peak. NOTHING ELSE may run in this container at the same time.
  setsid nohup bash -c "cd $WT && /cargo-target/release/gunbc run --source-root dag --source-root src/v2 \
    --entry dag/gunbc/instruments/generated_artifact_gate.dag --function main_wet_one \
    --arg path=$YML > /tmp/go-regen.log 2>&1; echo REGEN_EXIT=\$? >> /tmp/go-regen.log" \
    >/dev/null 2>&1 </dev/null & disown
  echo "regen started; poll /tmp/go-regen.log for REGEN_EXIT"
  ;;
verify)
  # what must be true of the regenerated artifact before I commit it
  git -C "$WT" diff --stat -- "$YML"
  echo "--- my three clauses must appear ---"
  for m in runner_host_file_converge runner_host_withdraw runner_host_restore; do
    printf '%s: %s\n' "$m" "$(grep -c "'$m' && format('gunbc-host-mutation-{0}', github.event.inputs.subject_host)" "$YML")"
  done
  echo "--- clause count on the shared group line (expect 7 after butch's join fold) ---"
  awk '/concurrency:/{c=1} c&&/group: /{print NR": "$0; exit}' "$YML" | grep -o "subject_host" | wc -l
  echo "--- the raise hunks must be ONLY these budget lines (cite the design_ruling row) ---"
  git -C "$WT" diff -- "$YML" | grep -E '^[-+].*(timeout-minutes|BUDGET_SECONDS)'
  ;;
commit)
  git add "$YML"
  git commit -m "regenerate fleet-converge.yml: the drain trio queues on its subject host

The three drain modes' TargetHostDomain rows now emit their subject_host clause on
the shared host-mutation concurrency group. The job/step timeout raise carried in
the same regeneration (354 / 8055 / 180) is derived from
gunbc.design_ruling::heal_convergence_job_budget_increase_2026_11_20 and is cited
to that row, approved by the operator."
  git push -q
  git log --oneline -1
  ;;
witness-job)
  # ~35 min, peak ~18.7 GiB. ONE at a time.
  setsid nohup bash -c "cd $WT && /cargo-target/release/gunbc run --source-root dag --source-root src/v2 \
    --entry dag/test/claim/fleet/fleet_converge_hostname_converge_emitter_witness.dag --claim-run \
    > /tmp/go-w-job.log 2>&1; echo JOBW_EXIT=\$? >> /tmp/go-w-job.log" >/dev/null 2>&1 </dev/null & disown
  echo "job half started; poll /tmp/go-w-job.log"
  ;;
witness-wf)
  # ~53 min, peak ~18.7 GiB. Only after the job half is DONE.
  setsid nohup bash -c "cd $WT && /cargo-target/release/gunbc run --source-root dag --source-root src/v2 \
    --entry dag/test/claim/fleet/fleet_converge_hostname_converge_workflow_witness.dag --claim-run \
    > /tmp/go-w-wf.log 2>&1; echo WFW_EXIT=\$? >> /tmp/go-w-wf.log" >/dev/null 2>&1 </dev/null & disown
  echo "workflow half started; poll /tmp/go-w-wf.log"
  ;;
*)
  echo "usage: go-chain.sh merge|engine|regen|verify|commit|witness-job|witness-wf"
  ;;
esac
```
