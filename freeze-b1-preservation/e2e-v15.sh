#!/bin/bash
# LOCAL structural E2E for packet-evidence-v15.sh (NOT a claim run; stub cargo + stub claim_batch + local throwaway repo).
set -u
E=/tmp/e2e; rm -rf "$E"; mkdir -p "$E/cg"
G=/tmp/packet
GIT="git -c init.defaultBranch=main -c user.name=e2e -c user.email=e2e@local -c gpg.format=none"
# --- cgroup face ---
printf '34359738368'  > "$E/cg/memory.max"
printf '30064771072'  > "$E/cg/memory.high"
printf '0'            > "$E/cg/memory.swap.max"
printf '4096'         > "$E/cg/pids.max"
printf '3200000 100000' > "$E/cg/cpu.max"
printf '999999999'    > "$E/cg/memory.peak"
printf '0'            > "$E/cg/memory.swap.current"
printf 'oom 0\noom_kill 0\noom_group_kill 0\nmax 0\n' > "$E/cg/memory.events"
# --- throwaway repo ---
mkdir -p "$E/w"; cd "$E/w"; $GIT init -q .
mkdir -p src/v1 src/v2/test/claim/namespace_xl0
echo 'pub fn v1() {}' > src/v1/lib.rs; printf '[package]\nname="v1-compiler"\n' > Cargo.toml; printf '# lock\n' > Cargo.lock
cp "$G/v15-wc-B.dag" src/v2/test/claim/namespace_xl0/wildcard_arm_resolve.dag
cp "$G/v15-rtp-B.dag" src/v2/test/claim/namespace_xl0/return_tail_position.dag
git add -A >/dev/null; $GIT commit -qm baseP; PB=$(git rev-parse HEAD)
cp "$G/v15-wc-A.dag" src/v2/test/claim/namespace_xl0/wildcard_arm_resolve.dag
cp "$G/v15-rtp-A.dag" src/v2/test/claim/namespace_xl0/return_tail_position.dag
git add -A >/dev/null; $GIT commit -qm repairP; PA=$(git rev-parse HEAD)
git checkout -q --detach "$PB"; cp "$G/v15-probe.dag" src/v2/test/claim/namespace_xl0/wc_verdict_probe.dag
git add -A >/dev/null; $GIT commit -qm seedB; SB=$(git rev-parse HEAD); TB=$(git rev-parse 'HEAD^{tree}')
git checkout -q --detach "$PA"; cp "$G/v15-probe.dag" src/v2/test/claim/namespace_xl0/wc_verdict_probe.dag
git add -A >/dev/null; $GIT commit -qm seedA; SA=$(git rev-parse HEAD); TA=$(git rev-parse 'HEAD^{tree}')
git init -q --bare "$E/repo"; git -C "$E/repo" config uploadpack.allowAnySHA1InWant true
git -C "$E/repo" config uploadpack.allowReachableSHA1InWant true
( cd "$E/w" && git push -q "$E/repo" "$SA:refs/heads/sa" "$SB:refs/heads/sb" "$PA:refs/heads/pa" "$PB:refs/heads/pb" ) || { echo "E2E PUSH FAILED"; exit 9; }
echo "E2E pins: PA=$PA PB=$PB SA=$SA TA=$TA SB=$SB TB=$TB"
# --- script copy with substituted pins + local fetch URL (E2E ONLY; frozen script keeps real pins) ---
sed -e "s|^PARENT_A=.*|PARENT_A=$PA   # repair head (PR #15999)|" \
    -e "s|^PARENT_B=.*|PARENT_B=$PB   # base|" \
    -e "s|^SEED_A=.*|SEED_A=$SA; TREE_A=$TA|" \
    -e "s|^SEED_B=.*|SEED_B=$SB; TREE_B=$TB|" \
    -e 's|https://github.com/gunb-ai/gunbc|file:///tmp/e2e/repo|g' \
    "$G/packet-evidence-v15.sh" > "$E/script.sh"
# --- stub env: cat/grep/cgroup face + cargo stub (produces target/release/claim_batch) ---
cat > "$E/run.sh" <<'RSH'
set -u
UNIT=sleek-seal-226-evidence-v15.scope
cat() { if [ "$#" -eq 0 ]; then command cat; else case "$1" in
  /proc/self/cgroup) printf '0::/%s\n' "$UNIT" ;;
  /sys/fs/cgroup/*) command cat "/tmp/e2e/cg/$(basename "${1#/sys/fs/cgroup/}")" ;;
  *) command cat "$@" ;;
esac; fi }
grep() { case "${1:-}" in
    -c) if [ "$#" -eq 3 ] && [ "$2" = "" ] && [ "$3" = "/proc/self/cgroup" ]; then echo 1; else command grep "$@"; fi ;;
    /sys/fs/cgroup/*) command grep "${@:1:$#-1}" "/tmp/e2e/cg/$(basename "${@: -1}")" ;;
    *) command grep "$@" ;;
  esac }
sed() { local last="${*: -1}"; if [ "$last" = "/proc/self/cgroup" ]; then printf '/%s\n' "$UNIT"; else command sed "$@"; fi }
cargo() { # stub build: fabricate the binary per contract; assert we were asked in release form
  local TGT=""; local a
  for a in "$@"; do case "$a" in CARGO_TARGET_DIR=*) ;; esac; done
  TGT="${CARGO_TARGET_DIR:-./target}"
  mkdir -p "$TGT/release" || return 1
  cat > "$TGT/release/claim_batch" <<'CB'
#!/bin/bash
ARGS="$*"
PFN=wc_verdict_probe_reports
CL='[process-partition] total_ms=1000 pre_run_ms=0 args_ms=0 index_build_ms=0 edge_index_warm_ms=0 output_policy_ms=0 roster_ms=0 published_mock_precompute_ms=0 entries_ms=0 reporting_ms=0 entries_resolve_ms=0 entries_witness_ms=0'
if [[ "$ARGS" == *"--functions $PFN"* ]]; then
  printf 'FAIL %s (returned ProcessExit::ExitFailure, code 3: WCVERIFY ctx=OK ctl_ok=r=0 ctl_out=r=X arm=X{r=X,a=-,s=T,al=0} field=X{r=X,a=-,s=T,al=0} binder=X{r=X,a=-,s=T,al=0} body=X{r=X,a=-,s=T,al=0})\n' "$PFN"
  printf '%s\n' "$CL"
  exit 1
fi
# claim entry: PASS every roster fn
FNS=""
while [ $# -gt 0 ]; do [ "$1" = "--functions" ] && { FNS="$2"; shift; }; shift; done
for f in $FNS; do printf 'PASS %s\n' "$f"; done
printf '%s\n' "$CL"
exit 0
CB
  chmod +x "$TGT/release/claim_batch"
  echo "STUB cargo build ok"
  return 0
}
export -f cat grep sed 2>/dev/null || true
PATH=/tmp/packet/e2e-bin:$PATH
PATH=/tmp/packet/e2e-bin:$PATH
bash "$1" "$2"
RSH
OUT="$E/out.$(date +%s%N)"
bash "$E/run.sh" "$E/script.sh" "$OUT"
RC=$?
echo "E2E rc=$RC out=$OUT"
[ -d "$OUT" ] && { echo '--- key lines ---'; grep -aE 'SELF_CONSISTENCY|ONE_BUILD_GATE|leg . seed ok|runtime-path fixtures|READER_PY|POSTCHECKS|EXPECT-|TOTAL_CLAIM_FAILS|PROBE_MISSING|artifact |NO OBSERVATION|REFUSED' "$OUT/drv.log" "$OUT"/*.log 2>/dev/null | head -40; echo '--- rows ---'; grep -aE '^[AB]\.(wc|rtp|probe)\.' "$OUT/../$(basename "$OUT")/drv.log" 2>/dev/null | head -3; ls "$OUT"; }
exit $RC
