#!/usr/bin/env bash
# raven_ctl.sh <tag> -- CONTROLS ONLY, no rebuild. Assumes target/release/gunbc already exists
# from a prior build at this head (RUN 1 left it moved aside/deleted by the timeout kill).
# Restores the binary between controls because the claim route moves it aside for native spawns
# and a timeout kill skips the restore. Timeouts are env-tunable (RAVEN_CTL_TIMEOUT).
set -u
TAG="${1:-noname}"
cd "$(dirname "$0")"
CTL_TIMEOUT="${RAVEN_CTL_TIMEOUT:-3600}"
echo "tag=$TAG mode=controls-only ctl_timeout=$CTL_TIMEOUT"
echo "RUNNER_HEAD=$(git rev-parse HEAD)"
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
if [ ! -x ./target/release/gunbc ]; then
  echo "gunbc missing -> cargo build --release -p v1-compiler --bins (mirrors unchanged, should be incremental)"
  cargo build --release -p v1-compiler --bins > /tmp/b1.log 2>&1; echo "rebuild_rc=$?"
  grep -aE -A 14 "^error(\[|:)" /tmp/b1.log | head -60 | fold -w 195
fi
test -x ./target/release/gunbc || { echo ABORT_NO_GUNBC; exit 46; }
for ctl in //v2/test/claim/compiler/infer_optional_at_required_witness:all //v2/test/claim/compiler/free_monoid_structure_bound:all; do
  name=$(basename ${ctl%%:*})
  timeout "$CTL_TIMEOUT" ./target/release/gunbc test "$ctl" > /tmp/route_ctl_${name}.out 2>&1
  echo "ctl_rc[$ctl]=$?"
  # restore the binary: the route withdraws target/release/gunbc for native spawns
  if [ ! -x ./target/release/gunbc ]; then
    cand=$(ls -t target/release/gunbc.old* target/release/.gunbc.withdrawn* 2>/dev/null | head -1)
    if [ -n "$cand" ]; then cp "$cand" target/release/gunbc && chmod +x target/release/gunbc && echo "restored_from=$cand"; else cargo build --release -p v1-compiler --bins >/tmp/b2.log 2>&1; echo "restore_rebuild_rc=$?"; fi
  fi
  grep -aE "HELD|DID NOT HOLD|NOT HELD|refused at prepare" /tmp/route_ctl_${name}.out | head -20 | fold -w 195
  grep -aE "resolve_reason_unbound_symbol|declarer_refused" /tmp/route_ctl_${name}.out | head -4 | fold -w 195
  tail -6 /tmp/route_ctl_${name}.err 2>/dev/null | fold -w 195
done
echo "=== DONE tag=$TAG ==="
