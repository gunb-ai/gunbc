#!/usr/bin/env bash
# BOTH sides of the E0277 A/B in ONE dispatch, because the runner's /tmp is fresh per dispatch:
# the emitted file only exists while the dispatch that produced it is alive. Runs the CI-path gate
# at the head and at main (warm target dirs in /root make this ~10 min per side), records each side's
# EMITTED_FOUND path, and diffs the two emitted std_realization_measurement.rs.
# USAGE: bash both_emit_diff_14152.sh <head_sha> <main_sha>
set -uo pipefail
H="${1:?head sha}"; M="${2:?main sha}"
find_emitted() { find /tmp -maxdepth 7 -path '*gunbc-emit-compile*' -name std_realization_measurement.rs 2>/dev/null | sort; }
pre=$(find_emitted | wc -l)
CARGO_TARGET_DIR=/root/gunbc_target_gate_head bash gate_emit_build_14152.sh "$H" bdd_head repair > /tmp/bdd_head.txt 2>&1
rc_h=$?
after_h=$(find_emitted); fa=$(comm -13 <(find_emitted | head -0) /dev/null >/dev/null 2>&1; find_emitted | head -1)
# The head ran first, so its file is the first under /tmp/gunbc-emit-compile*; record it now.
fa=$(find_emitted | head -1)
echo "HEAD_SIDE rc=$rc_h EMITTED=$fa"
grep -aE "SELF_HOST_RC=|EMITTED_BUILD_FAILED_LINES=|error\[E0277\]|GUARD" /tmp/bdd_head.txt | head -5
cp -f "$fa" /tmp/emitted_head.rs 2>/dev/null && echo "SAVED_HEAD bytes=$(wc -c < /tmp/emitted_head.rs) sha=$(sha256sum /tmp/emitted_head.rs | cut -c1-16)"
CARGO_TARGET_DIR=/root/gunbc_target_gate_main bash gate_emit_build_14152.sh "$M" bdd_main main > /tmp/bdd_main.txt 2>&1
rc_m=$?
fb=$(find_emitted | grep -v "^$fa$" | head -1)
echo "MAIN_SIDE rc=$rc_m EMITTED=$fb"
grep -aE "SELF_HOST_RC=|EMITTED_BUILD_FAILED_LINES=|error\[E0277\]|GUARD" /tmp/bdd_main.txt | head -5
cp -f "$fb" /tmp/emitted_main.rs 2>/dev/null && echo "SAVED_MAIN bytes=$(wc -c < /tmp/emitted_main.rs) sha=$(sha256sum /tmp/emitted_main.rs | cut -c1-16)"
echo "=== VERDICT ==="
if [ ! -s /tmp/emitted_head.rs ] || [ ! -s /tmp/emitted_main.rs ]; then echo "VOID missing emitted side"; exit 5; fi
if cmp -s /tmp/emitted_head.rs /tmp/emitted_main.rs; then echo "EMITTED_IDENTICAL -- my change did NOT alter this emitted file; the mint is not the cause"; else echo "EMITTED_DIFFERS"; diff /tmp/emitted_main.rs /tmp/emitted_head.rs | head -60 | cut -c1-200; fi
echo "=== blamed function, head ==="; awk '/fn witness_measured_rollup_differs_from_predicted_zero/,/^}/' /tmp/emitted_head.rs | head -16 | cut -c1-200
echo "=== blamed function, main ==="; awk '/fn witness_measured_rollup_differs_from_predicted_zero/,/^}/' /tmp/emitted_main.rs | head -16 | cut -c1-200
echo "T=$(date +%T) done"
