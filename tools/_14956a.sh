#!/usr/bin/env bash
# #14956 dispatch A: ONE head-built claim_batch (PR delta origin/main...head = 11 .dag, 0 .rs =>
# subject is DATA; one driver serves every tree), static retirement scan, tree prep:
#   H = head; P = pre-deletion base @115143a4d8a; M = head + Blocking contract (emitted_binary)
#   repointed at a truly-absent witness file (RED); V = head + AuditOnly contract (vacuity) repointed
#   at the same absent file (other-shape control).
set -u
cd "$(dirname "$0")/.." || exit 9
. tools/_14956prelude.sh
echo "=== prep $(date -u +%H:%M:%S)Z"
rm -rf $H $P $M $V; git worktree prune >/dev/null 2>&1
wt $H $HEAD_SHA || { echo WTH_FAIL; exit 9; }
wt $P $BASE_SHA || { echo WTP_FAIL; exit 9; }
wt $M $HEAD_SHA || { echo WTM_FAIL; exit 9; }
wt $V $HEAD_SHA || { echo WTV_FAIL; exit 9; }
echo "=== build head tree: $(date -u +%H:%M:%S)Z"
build_head || exit 9
# ---------- static retirement scan on the head tree
echo "=== SCHEDULE_LENS_SCAN head tree: $(date -u +%H:%M:%S)Z"
(cd $H && git grep -n -i "schedule_lens" -- . | sed "s|^$H/||") | tee /tmp/956scan.txt
echo "SCAN_TOTAL_LINES=$(wc -l < /tmp/956scan.txt)"
echo "=== deleted module + its witness absent at head?"
for f in src/v2/lens/schedule_lens.dag src/v2/test/claim/walk_plan_schedule_lens_test.dag; do
  [ -e $H/$f ] && echo "PRESENT_ANOMALY $f" || echo "ABSENT_OK $f"
done
echo "=== base tree HAS them?"
for f in src/v2/lens/schedule_lens.dag src/v2/test/claim/walk_plan_schedule_lens_test.dag; do
  [ -e $P/$f ] && echo "PRESENT_OK(base) $f" || echo "ABSENT_ANOMALY(base) $f"
done
# ---------- plant the mutations (idempotent helper)
python3 tools/_14956mut.py $M $V || { echo "MUTATION_FAIL -- STOP"; exit 9; }
ls $H/src/v2/test/claim/gone_14956_probe.dag >/dev/null 2>&1 && echo "PLANT_FILE_PRESENT_ANOMALY head" || echo "PLANT_TARGET_TRULY_ABSENT head"
echo "DISPATCH_A_DONE $(date -u +%H:%M:%S)Z"
