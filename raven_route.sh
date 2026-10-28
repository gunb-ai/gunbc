#!/usr/bin/env bash
# raven_route.sh <tag> -- CONTROLS-ONLY route health (no mutation phase). Decides whether the
# claim route passes on this head (the std.algebra Map prepare wall: does it still bite?).
set -u
TAG="${1:-noname}"
cd "$(dirname "$0")"
MT_KB=$(awk '/MemTotal/{print $2}' /proc/meminfo)
echo "tag=$TAG MemTotal_MB=$((MT_KB/1024))"
echo "RUNNER_HEAD=$(git rev-parse HEAD)"
git status --short | head -8
test "$MT_KB" -ge 41943040 || { echo ABORT_MEMORY; exit 42; }
CGSELF=$(awk -F: '{ if ($1=="0") {print $3; exit} else if ($2=="memory") {print $3; exit} }' /proc/self/cgroup)
if grep -q '^0::' /proc/self/cgroup; then CGBASE=/sys/fs/cgroup; else CGBASE=$(awk -F: '$2=="memory"{print "/sys/fs/cgroup/memory"; exit}' /proc/self/cgroup); fi
[ -n "${CGBASE:-}" ] || CGBASE=/sys/fs/cgroup
CGPATH="$CGBASE${CGSELF}"
BOUND=$(cat "$CGPATH/memory.max" 2>/dev/null || cat "$CGPATH/memory.limit_in_bytes" 2>/dev/null || echo unreadable)
echo "CGROUP_INHERITED path=$CGPATH bound=$BOUND (accept 42..128 GiB; systemd-run MemoryMax=64G answers 68719476736)"
case "$BOUND" in ''|max|unreadable) echo "ABORT_CGROUP_UNBOUND bound=$BOUND"; exit 42 ;; esac
awk -v b="$BOUND" 'BEGIN { exit !(b+0 >= 45035996262 && b+0 <= 137438953472) }' || { echo "ABORT_CGROUP_RANGE bound=$BOUND"; exit 42; }
export RAVEN_CGROUP_PATH="$CGPATH"
export GITHUB_ACTIONS=true GITHUB_SHA=$(git rev-parse HEAD)
unset GUNBC_MEMORY_BUDGET_BYTES GUNBC_MEMORY_ENVELOPE_REQUIRED GUNBC_BIND_MEMORY_CGROUP_BYTES
export TMPDIR=/tmp
PATH=$PATH:/usr/local/cargo/bin
echo "=== build1 (committed mirrors) ==="
cargo build --release -p v1-compiler --bins > /tmp/b1.log 2>&1; echo "build1_rc=$?"
if grep -qaE "^error(\[|:)" /tmp/b1.log; then echo ABORT_BUILD1; grep -aE -A 14 "^error(\[|:)" /tmp/b1.log | head -90 | fold -w 195; exit 46; fi
echo "=== round (regen mirrors from .dag authority) ==="
rm -rf target/stage0-regen-candidate
timeout 2400 ./target/release/claim_executor --regen-round-cost --source-root dag --source-root src/v2 > /tmp/round.out 2>/tmp/round.err
echo "round_rc=$?"
tail -3 /tmp/round.out | fold -w 195; echo "--- round.err tail (10) ---"; tail -10 /tmp/round.err | fold -w 195
echo "=== build2 (from installed round) ==="
cargo build --release -p v1-compiler --bins > /tmp/b2.log 2>&1; echo "build2_rc=$?"
if grep -qaE "^error(\[|:)" /tmp/b2.log; then echo ABORT_BUILD2; grep -aE -A 14 "^error(\[|:)" /tmp/b2.log | head -90 | fold -w 195; exit 46; fi
test -x ./target/release/gunbc || { echo ABORT_NO_GUNBC; exit 46; }
echo "=== controls on the SAME route ==="
for ctl in //v2/test/claim/compiler/infer_optional_at_required_witness:all //v2/test/claim/compiler/free_monoid_structure_bound:all; do
  timeout 1200 ./target/release/gunbc test "$ctl" > /tmp/route_ctl_$(basename ${ctl%%:*}).out 2>&1
  echo "ctl_rc[$ctl]=$?"
  grep -aE "HELD|DID NOT HOLD|NOT HELD|refused at prepare" /tmp/route_ctl_$(basename ${ctl%%:*}).out | head -14 | fold -w 195
  grep -aE "resolve_reason_unbound_symbol|declarer_refused" /tmp/route_ctl_$(basename ${ctl%%:*}).out | head -4 | fold -w 195
done
echo "=== DONE tag=$TAG ==="
