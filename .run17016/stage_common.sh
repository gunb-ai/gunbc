# Sourced preamble for every stage: HOST/BOOT/self-cgroup/recorder helpers.
export PATH="$HOME/.cargo/bin:$PATH"
echo "HOST=$(hostname)"; echo "BOOT=$(cut -c1-8 /proc/sys/kernel/random/boot_id)"
echo "SELF_CGROUP=$(tr '\n' ',' < /proc/self/cgroup) PID=$$"
cd "$HOME/scratch/bold-crane-719" || exit 9
SCRATCH="${SCRATCH:-$HOME/scratch/bold-crane-719}"; export SCRATCH
RULE_EPOCH="${RULE_EPOCH:-1795692120}"   # 2026-11-26T11:22:00Z: B voided every srv1 artifact from before this
echo "RULE_EPOCH=$RULE_EPOCH (artifacts must be newer)"

sha_line() { echo "SHA256_OF_BINARY=$(sha256sum "$1" | awk '{print $1}') bytes=$(wc -c < "$1")"; }
print_cgroup() { echo "SELF_CGROUP=$(tr '\n' ',' < /proc/self/cgroup) PID=$$"; }
assert_fresh() {
  sha_line "$1"
  print_cgroup
  MTIME=$(stat -c %Y "$1" 2>/dev/null); MTIME_EPOCH=${MTIME:-0}
  echo "BIN_MTIME_EPOCH=$MTIME_EPOCH RULE_EPOCH=$RULE_EPOCH"
  if [ "$MTIME_EPOCH" -gt "$RULE_EPOCH" ]; then echo "FRESHNESS_ASSERT_RC=0"; else echo "VOID_BUILD_NOT_FRESH mtime_epoch=$MTIME_EPOCH rule_epoch=$RULE_EPOCH"; echo "FRESHNESS_ASSERT_RC=1"; return 1; fi
}

# INSTRUMENT CHECK (corrected 12:0xZ): PR changes NO Rust; refusal arms live in the .dag corpus
# loaded from the TREE at runtime. Binary-grep VOID at 11:59:30Z was a category error -- recorded.
instr_check() {
  T="$1"
  E="$T/src/v2/compiler/effect_demand.dag"; F="$T/dag/gunbc/demand_engine_restatement_follow_up.dag"
  test -f "$E" && test -f "$F" || { echo VOID_STOP_MODEL_FILES_MISSING; return 1; }
  C1=$(grep -ac "filesystem_read_reached_outside_filesystem_handler" "$E"); R1=$?
  C2=$(grep -ac "new_demand_restatement_row_unretained" "$F"); R2=$?
  C3=$(grep -ac "NewRestatementRowRefused" "$F"); R3=$?
  C4=$(grep -ac "filesystem_effect_reaches_from_phase1_rows" "$E"); R4=$?
  echo "INSTR_TREE fs_cause=$C1(rc=$R1) restatement_cause=$C2(rc=$R2) refused_arm=$C3(rc=$R3) census_fn=$C4(rc=$R4) tree=$(git -C "$T" rev-parse HEAD)"
  [ "$C1" -ge 1 ] && [ "$C2" -ge 1 ] && [ "$C3" -ge 1 ] && [ "$C4" -ge 1 ] || { echo VOID_STOP_INSTRUMENT_MISSING; return 1; }
}

# Binary route probes: named routes and their flags, raw rcs. A route that does not exist is a
# named failure, never a silent void. claim_batch must be ABSENT (deleted ad5c67a316f / gunbc#16872).
binary_probe() {
  B="$1"
  "$B" test --help   >/dev/null 2>&1; H1=$?
  "$B" compile --help >/dev/null 2>&1; H2=$?
  "$B" run --help    >/dev/null 2>&1; H3=$?
  "$B" --help        >/dev/null 2>&1; H4=$?
  echo "BIN_PROBE test_help_rc=$H1 compile_help_rc=$H2 run_help_rc=$H3 gunbc_help_rc=$H4"
  echo "BIN_TEST_FLAG_COUNTS --source-root=$("$B" test --help 2>&1 | grep -ac -- "--source-root")"
  echo "BIN_RUN_FLAG_COUNTS --function=$("$B" run --help 2>&1 | grep -ac -- "--function") --entry=$("$B" run --help 2>&1 | grep -ac -- "--entry")"
  CF="$B"; CF="${CF%gunbc}claim_batch"
  if test -x "$CF"; then echo "VOID_STOP_CLAIM_BATCH_PRESENT"; return 1; fi
  "$B" claim_batch --help >/dev/null 2>&1; CR=$?
  echo "BIN_PROBE claim_batch_rc=$CR (deleted route; nonzero expected)"
  if [ "$H1" -ne 0 ] || [ "$H2" -ne 0 ] || [ "$H3" -ne 0 ] || [ "$H4" -ne 0 ]; then return 1; fi
  if [ "$CR" -eq 0 ]; then echo "VOID_STOP_CLAIM_BATCH_HELP_RC0"; return 1; fi
  return 0
}
boot_id() { cut -c1-8 /proc/sys/kernel/random/boot_id; }
