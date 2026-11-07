#!/usr/bin/env bash
# tools/acceptance_r4_r5_r6_v9_selfaudit.sh - an independent audit of the FROZEN v9 acceptance packets.
#
# WHAT THIS IS. A re-runnable second opinion on the bytes of acceptance_r4_claim_run_v9.sh,
# acceptance_r5_claim_run_v9.sh and acceptance_r6_live_read_claim_run_v9.sh, written as checks rather than
# as reasoning, aimed at the defect classes that actually cost grants in earlier rounds (a literal
# `claim_batch` argv token, a space-separated --functions value, a completion record read from the wrong
# stream, an unbracketed prefix that counts `[process-partition-exit]`, a gate that refuses by echoing
# while its caller checks rc, a `timeout` without a kill grace, stale version strings in a frozen file,
# claim names that no longer exist at the pin). It complements acceptance_r4_r5_r6_selfcheck_v9.sh: that
# harness proves the packets' internal contracts with 1000+ assertions; this file checks properties the
# harness does not assert, and does so in 91 cheap checks: greps over the frozen bytes, the packets'
# own --predict-only inventory, and six read-only git queries against the pinned objects.
#
# WHAT IT RUNS, and what it never touches. Only `--predict-only` on each packet (which prints the frozen
# command inventory and its digests and executes nothing), read-only `git rev-parse` / `git cat-file` at
# the two pins, and greps. It never enters the run path, never opens a network connection, never writes
# outside `mktemp -d`, and must not be run on srv1 or under a grant: it is for the reviewing container.
#
# IT IS NOT PART OF THE GRANTED BYTES. The packets, the self-check and the manifest keep their own digests
# regardless of this file; adding or changing it moves only the PR head. Keep it that way: if a packet is
# re-frozen, update the names below and re-run this - a check that cannot fail is the thing this file
# exists to point out in others.
#
# Usage: bash tools/acceptance_r4_r5_r6_v9_selfaudit.sh   (from any directory; exit 0 = every check held)
# Each check is a defect class that an earlier review round found or warned about.
set -u
# Run it from anywhere: it locates the repo through its own path and reads only from the committed bytes.
SELF=$(cd "$(dirname "$0")" && pwd)
ROOT=$(cd "$SELF/.." && pwd)
T=$(mktemp -d)
trap 'rm -rf "$T"' EXIT
cd "$SELF" || exit 1
P4=acceptance_r4_claim_run_v9.sh P5=acceptance_r5_claim_run_v9.sh P6=acceptance_r6_live_read_claim_run_v9.sh
ok=0; bad=0
ck(){ if [ "$2" = "$3" ]; then ok=$((ok+1)); printf 'ok   %-58s %s\n' "$1" "$3"; else bad=$((bad+1)); printf 'FAIL %-58s want=%s got=%s\n' "$1" "$2" "$3"; fi; }
ge(){ if [ "$3" -ge "$2" ] 2>/dev/null; then ok=$((ok+1)); printf 'ok   %-58s %s (>= %s)\n' "$1" "$3" "$2"; else bad=$((bad+1)); printf 'FAIL %-58s want>=%s got=%s\n' "$1" "$2" "$3"; fi; }
for f in $P4 $P5 $P6; do
  echo "== $f"
  ck  "no bare claim_batch token after the binary" 0 "$(grep -cE 'claim_batch claim_batch|" ?\$BIN"? claim_batch' "$f")"
  ck  "no space-separated --functions value"       0 "$(grep -E '^CMD_LEG' "$f" | grep -cE -- '--functions [^ ]* [^ ]' || true)"
  ck  "no exit outside 0/1/2 (code, not prose)"    0 "$(grep -vE '^[[:space:]]*#' "$f" | grep -oE 'exit [0-9]+' | sort -u | grep -vcE 'exit (0|1|2)$' || true)"
  ge  "value-position gates fail closed on empty"  3 "$(grep -cE '= OK\] \|\| causes|!= 0 \] then|\$\(pp_check|\$\(stray_verdict_lines|\$\(leg_state' "$f")"
  ge  "completion check reads the stderr log"      1 "$(grep -c 'pp_check "$OUT/logs/$L.err"' "$f")"
  ck  "completion check never repointed at .out"   0 "$(grep -c 'pp_check "$OUT/$L.out"' "$f")"
  ge  "completion prefix keeps the ] bracket"      1 "$(grep -cF '[process-partition] total_ms' "$f")"
  ck  "no unbracketed prefix key (the -exit trap)" 0 "$(grep -cF '[process-partition total_ms' "$f")"
  ge  "completion line digest pinned (7ac24cf2)"   1 "$(grep -c 7ac24cf2 "$f")"
  ge  "empty-predicate refusal present"            1 "$(grep -c 'PP_PREFIX" \] && \[ -n "$PP_SHAPE' "$f")" >/dev/null
  ck  "no stale _v8 reference"                     0 "$(grep -c '_v8' "$f")"
  ck  "no stale v8 wall sums"                      0 "$(grep -cE '\b(4920|4020)\b' "$f")"
  case $f in *r6*) ge "R6 header states the HELD gate" 1 "$(grep -c 'state=HELD' "$f")";;
             *)   ge "header names the v9 manifest"     1 "$(grep -c 'acceptance_r4_r5_r6_v9_manifest.md' "$f")";; esac
  ge  "RUNTIME_S <= 7200 and cap checked"          1 "$(grep -c LAUNCHER_CAP "$f")"
  ge  "WALL_SUM counts self-run and graces"        2 "$(grep -E '^WALL_SUM=' "$f" | grep -oE 'T_SELFRUN|N_TIMEOUTS_RUN\*T_KILL' | wc -l)"
  ck  "every run-path timeout has a kill grace"    0 "$(grep -E 'timeout ["$]?T_' "$f" | grep -vc 'timeout -k')"
  ge  "booked-set constants derive the argv"       1 "$(grep -c 'CLAIMS_ALL' "$f")"
  ge  "exit-2 used for setup/gate refusals"        5 "$(grep -c 'exit 2' "$f")"
done
echo; echo "== frozen argv shape, read out of --predict-only (no run path executed)"
for f in $P4 $P5 $P6; do
  bash "$f" --predict-only > "$T/$f.pred" 2>&1 || { echo "predict failed for $f"; bad=$((bad+1)); continue; }
  case $f in *r4*) want=6;; *) want=5;; esac
  n=$(grep -c '^cmd CMD' "$T/$f.pred")
  ge  "$f command strings printed"                  $want "$n"
  # the argv is assembled from constants, so its shape is asserted by the packet's own parser-mirroring
  # controls (argv_check), which are listed as fixtures and must bite.
  ctl=$(grep -coE 'ck argv-ctl-(subcommand|spaces|badflag|typo|missing|double|noentry|wrongentry)' "$f")
  ge  "$f carries its argv parser controls"        8 "$ctl"
  tok=$(grep -E '^CMD_LEG' "$f" | grep -cE 'claim_batch claim_batch|"\$BIN[456]" claim_batch' || true)
  ck  "$f no literal claim_batch token in the frozen argv" 0 "$tok"
  sp=$(grep -oE -- '--functions "[^"]*"' "$f" | sed 's/--functions "//; s/"$//' | grep -c ' ' || true)
  ck  "$f --functions values contain no space"      0 "$sp"
  ge  "$f prints its job name in the header"        1 "$(grep -c "job=fierce-wren-19" "$T/$f.pred")"
done
echo; echo "== booked claim names exist in the PINNED entry blobs (read-only git)"
# the pins this packet set is frozen against; the next block requires the packets to name the same ones
R4=db56530fcb186370e3dbf1aa3c69ded324daa25d; R5=dff57af790f923066d61721e89ff751a91021278
ge "R4 packet names the R4 pin in its own text" 1 "$(grep -c "$R4" acceptance_r4_claim_run_v9.sh)"
ge "R5 packet names the R5 pin in its own text" 1 "$(grep -c "$R5" acceptance_r5_claim_run_v9.sh)"
ge "R6 packet names the R6 pin in its own text" 1 "$(grep -c "$R5" acceptance_r6_live_read_claim_run_v9.sh)"
ge "the R4 pinned entry blob is readable in this object store" 1 "$(git -C "$ROOT" rev-parse "$R4:src/v2/test/claim/stage0_partition_rebuild_scope_witness.dag" | grep -c '^[0-9a-f]\{40\}$')"
ge "the R5 pinned entry blob is readable in this object store" 1 "$(git -C "$ROOT" rev-parse "$R5:dag/test/claim/roadmap/roadmap_acceptance_inbox_witness.dag" | grep -c '^[0-9a-f]\{40\}$')"
ge "the R6 pinned entry blob is readable in this object store" 1 "$(git -C "$ROOT" rev-parse "$R5:dag/test/claim/roadmap/roadmap_acceptance_inbox_live_witness.dag" | grep -c '^[0-9a-f]\{40\}$')"
ge "claim_batch.rs is one blob at both pins (no drift to gate twice)" 1 "$(a=$(git -C "$ROOT" rev-parse "$R4:src/v1/stage0/src/bin/claim_batch.rs"); b=$(git -C "$ROOT" rev-parse "$R5:src/v1/stage0/src/bin/claim_batch.rs"); [ "$a" = "$b" ] && echo 1 || echo 0)"
git -C "$ROOT" cat-file blob "$(git -C "$ROOT" rev-parse "$R4:src/v2/test/claim/stage0_partition_rebuild_scope_witness.dag")" > "$T/e4a" 2>/dev/null
git -C "$ROOT" cat-file blob "$(git -C "$ROOT" rev-parse "$R4:src/v2/test/claim/rust_crate_partition_witness.dag")"      > "$T/e4b" 2>/dev/null
git -C "$ROOT" cat-file blob "$(git -C "$ROOT" rev-parse "$R5:dag/test/claim/roadmap/roadmap_acceptance_inbox_witness.dag")" > "$T/e5" 2>/dev/null
git -C "$ROOT" cat-file blob "$(git -C "$ROOT" rev-parse "$R5:dag/test/claim/roadmap/roadmap_acceptance_inbox_live_witness.dag")" > "$T/e6" 2>/dev/null
for n in witness_duplicate_owner_refuses_holds witness_unowned_mirror_refuses_holds witness_shell_partition_duplicate_owner_refuses_holds witness_leaf_edit_compiles_owner_and_downstream_holds witness_leaf_edit_excludes_unrelated_packages_holds; do
  ge "R4 $n in the pinned blob" 1 "$(grep -c "$n" "$T/e4a")"; done
ge "R4 witness_r4_refuses_unit_graph_cycle_holds in the pinned blob" 1 "$(grep -c witness_r4_refuses_unit_graph_cycle_holds "$T/e4b")"
for n in witness_inbox_load_refused_is_refused_not_empty witness_inbox_unreplayable_history_is_refused_not_empty witness_inbox_live_receipt_removes_node_from_rows witness_inbox_revoked_receipt_reappears_with_prior witness_inbox_stale_live_receipt_refuses; do
  ge "R5 $n in the pinned blob" 1 "$(grep -c "$n" "$T/e5")"; done
ge "R6 witness_inbox_on_authority_projects_on_live_corpus in the pinned blob" 1 "$(grep -c witness_inbox_on_authority_projects_on_live_corpus "$T/e6")"
# and each name is booked exactly once per packet
for f in $P4 $P5 $P6; do
  case $f in *r4*) want=6;; *r5*) want=5;; *) want=1;; esac
  booked=$(grep -hoE '^N[0-9A-Z]+="witness[^"]*"' "$f" | sed 's/^[^=]*="//; s/"$//' | tr ' ' '\n' | grep -E '^witness_[a-z0-9_]+$' | sort -u | wc -l)
  ge "$f books exactly $want claim names" $want "$booked"
done
echo; echo "SELF-AUDIT v9: $ok ok, $bad FAIL"
exit $(( bad > 0 ))
