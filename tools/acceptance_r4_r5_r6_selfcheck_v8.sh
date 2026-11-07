#!/usr/bin/env bash
# =====================================================================
# acceptance_r4_r5_r6_selfcheck_v8 — the fixture self-test for the three v8 packets: one job per row (R4,
# R5, and R6 held). Item 8 is the v5 stream fix (the completion record is the runner's stderr, not stdout).
# NOTHING FROM EITHER REPOSITORY IS EXECUTED and nothing is run on srv1: it feeds synthetic claim logs,
# synthetic cgroup and memory.events files, synthetic run dirs and the packets' OWN tokenized command lines
# to the packets' own functions (extracted verbatim from between the markers), runs each packet's embedded
# `--classifier-fixtures` mode, exercises the packets end-to-end up to the point where a gate refuses on THIS
# host (before any clone, fetch, build or store contact), and checks the frozen constants against local git
# objects and against the pinned claim_batch parser.
#
# Each item the acceptance checks named gets an assertion in BOTH directions, with negative controls at the
# end: (1) the launcher frame, (2) the pre-work bounds gate, (3) the strict OOM contract, (4) the three-code
# exit contract, (5) the "discriminates nothing" statement, (6) wall accounting, (7) the completion record and
# the verdict exactness rules, and (8, v5) WHICH STREAM carries the completion record: the producer emits it
# with eprintln! (stderr) at line 1460 of the pinned claim_batch.rs while verdicts go to stdout, so the gate
# must read logs/<leg>.err and the classifier <leg>.out. Getting that wrong is #15695 G1 by another route:
# every leg reports 0 records, a full-green row NO_OBSERVATIONs, and the grant is spent.
#
# Content checks are byte-identity gated, never proximate: a working-tree file stands in for the pinned bytes
# only when `git hash-object` of it equals the blob id AT the pin; otherwise the check reports SKIPPED, never
# passed. (This checkout is a blob:none partial clone: `git cat-file blob <pin>:<path>` would lazily FETCH and
# can hang on a credential prompt, so pinned bytes are reached through local tree objects plus hash-equality,
# and every git call here gets /dev/null on stdin.)
#
# usage: acceptance_r4r5_selfcheck_v3.sh [repo-with-pinned-commits]
set -uo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
# acceptance_r4_r5_r6_selfcheck_v8 -> v6: v6 SPLITS the combined R4/R5 packet into two single-row jobs,
# because the installed calmcrab-udrv v4 (855920ac) refuses any RUNTIME_S above 7200 in its own preflight
# (sleek-owl-397 via calm-eagle-670 msg_384bca0f; same shape as #15695 diag-v2 booked at 11700). So each
# row is its own job: its own clone/fetch/checkout/build, its own legs, its own launcher line, and its own
# wall_gate that refuses a booking over the cap BEFORE any fetch. Every v5 gate stays, in all three.
P4="$HERE/acceptance_r4_claim_run_v8.sh"
P5="$HERE/acceptance_r5_claim_run_v8.sh"
P6="$HERE/acceptance_r6_live_read_claim_run_v8.sh"
PKTS="$P4 $P5 $P6"
REPO="${1:-}"
for f in "$P4" "$P5" "$P6"; do [ -f "$f" ] || { echo "packet $f not found next to the self-check" >&2; exit 1; }; done
g() { git "$@" < /dev/null; }

PASS_N=0; FAIL_N=0; SKIP_N=0; FAILED=()
ok()   { PASS_N=$((PASS_N+1)); }
bad()  { FAIL_N=$((FAIL_N+1)); FAILED+=("$1"); echo "  NOT: $1"; }
skip() { SKIP_N=$((SKIP_N+1)); echo "  SKIPPED: $1"; }
check() { local name="$1" want="$2"; shift 2
  local out rc; out=$("$@" 2>&1); rc=$?
  if [ "$rc" = "$want" ]; then ok; else bad "$name (rc=$rc want=$want): $(printf '%s' "$out" | head -2 | tr '\n' '|')"; fi; }
check_out() { local name="$1" want="$2" pat="$3"; shift 3
  local out rc; out=$("$@" 2>&1); rc=$?
  if [ "$rc" = "$want" ] && printf '%s' "$out" | grep -qF -- "$pat"; then ok
  else bad "$name (rc=$rc want=$want, wanted the literal '$pat'): $(printf '%s' "$out" | head -2 | tr '\n' '|')"; fi; }
grep_in() { local name="$1" f="$2" pat="$3"; grep -qF -- "$pat" "$f" && ok || bad "$name: '$pat' not found in $(basename "$f")"; }
grep_not() { local name="$1" f="$2" pat="$3"; grep -qF -- "$pat" "$f" && bad "$name: '$pat' IS in $(basename "$f")" || ok; }

SB="$(mktemp -d "${TMPDIR:-/tmp}/r4r5r6-selfcheck-v6-XXXXXX")"
chmod 700 "$SB"
trap 'rm -rf "$SB"' EXIT
S4="$SB/fake/wt-r4/target/release/claim_batch"; S5="$SB/fake/wt-r5/target/release/claim_batch"; S6="$SB/fake/wt-r6/target/release/claim_batch"

# ---------------------------------------------------------------- read the packets' own constants and commands
# the packets' own constants, read out by NAME (an open-ended assignment grep would also pick up runtime
# lines like `O=$OUT/receipt.txt`, which is not a constant and would explode here)
csv() { local s; s=$(printf '%s,' $1); printf '%s' "${s%,}"; }
CONSTS='J|U|H|H4|H5|T|T4|T5|E4A|E4B|E5|E6|B4A|B4B|B5|B6|N4A|N4B|N5|N6|C4A|C4B|C5|C6|BOOKED|RUNTIME_S|DRIVER_S|LAUNCHER_CAP|N_LEGS|WALL_SUM|MEMMAX_G|MEMHIGH_G|TASKS_MAX|CPU_PCT|T_SETUP_CLONE|T_SETUP_FETCH|T_SETUP_CHECKOUT|T_BUILD|T_LEG'
eval "$(cat "$P4" "$P5" "$P6" | grep -E "^($CONSTS)=")"
export BOOKED   # a bash -c probe must see the booked string the packet freezes
load_cmds() { eval "$(grep -E "^CMD_[A-Z0-9_]+=" "$1")"; }
load_cmds "$P4"; load_cmds "$P5"; load_cmds "$P6"
for v in J H4 H5 T4 T5 E4A E4B E5 E6 B4A B4B B5 B6 N4A N4B N5 N6 C4A C4B C5 C6 BOOKED \
         T_SETUP_CLONE T_SETUP_FETCH T_SETUP_CHECKOUT T_BUILD T_LEG \
         CMD_CLONE_4 CMD_FETCH_4 CMD_CHECKOUT_4 CMD_BUILD_4 CMD_LEG_R4A CMD_LEG_R4B \
         CMD_CLONE_5 CMD_FETCH_5 CMD_CHECKOUT_5 CMD_BUILD_5 CMD_LEG_R5 \
         CMD_CLONE CMD_FETCH CMD_CHECKOUT CMD_BUILD CMD_LEG_R6; do
  [ -n "${!v:-}" ] && ok || bad "constant $v did not read out of the packets"
done
# the two packets must agree on the shared vocabulary, not merely resemble each other
for v in BOOKED T_SETUP_CLONE T_SETUP_FETCH T_SETUP_CHECKOUT T_BUILD T_LEG; do :; done
b4=$(grep -m1 '^BOOKED=' "$P4"); b5=$(grep -m1 '^BOOKED=' "$P5"); b6=$(grep -m1 '^BOOKED=' "$P6")
[ "$b4" = "$b5" ] && [ "$b5" = "$b6" ] && ok || bad "the three packets freeze different bounds: $b4 | $b5 | $b6"
# v6: every packet must also freeze the same launcher cap, and the per-row wall constants must be its own
for f in $PKTS; do grep -q 'LAUNCHER_CAP=7200' "$f" && ok || bad "$(basename "$f") does not freeze the installed launcher cap"; done

# ---------------------------------------------------------------- each packet's embedded fixtures must pass
for f in $PKTS; do
  out=$(bash "$f" --classifier-fixtures 2>&1); rc=$?
  np=$(printf '%s\n' "$out" | grep -c '^FIXTURE PASS'); nf=$(printf '%s\n' "$out" | grep -c '^FIXTURE FAIL')
  # v6 was 178/176/180, v7 227/225/229; v8 adds the claim-set gate and its 33 fixtures per packet, so a
  # count under 250 means fixtures were dropped, which is the same as a rule losing its proof.
  if [ "$rc" = 0 ] && [ "$nf" = 0 ] && [ "$np" -gt 250 ]; then ok; else bad "$(basename "$f") embedded fixtures: rc=$rc pass=$np fail=$nf (want rc 0, 0 failures, > 250 assertions)"; fi
  echo "  $(basename "$f"): $np embedded fixtures pass, $nf fail (rc=$rc)"
done

# ---------------------------------------------------------------- the helpers, verbatim, fed synthetic input
helpers() { sed -n '/# ---------------------------------------------------------------- gates and helpers/,/# ---------------------------------------------------------------- fixture mode/p' "$1" | sed '$d'; }
helpers "$P4" > "$SB/h4.sh"; helpers "$P5" > "$SB/h5.sh"; helpers "$P6" > "$SB/h6.sh"
# one gate, three jobs: the GATE SECTION of the three packets has to be the same code. Row-specific lines
# (the pins, entry paths, CSVs, worktree names, the runner variable) and R6's grant gate are excluded BY
# NAME; everything else -- the scope gate, the booked-bounds gate, the wall gate, the OOM readers, the
# completion-record gate, the verdict classifier, the leg runner -- must be identical, or the three jobs are
# three dialects and one of them is silently weaker than the others.
gatesection() { sed -n '/# ---------------------------------------------------------------- gates and helpers/,/# ---------------------------------------------------------------- fixture mode/p' "$1" \
  | grep -vE '^[[:space:]]*#' | grep -vE '^[[:space:]]*$' \
  | grep -vE '\b(E4A|E4B|E5|E6|N4A|N4B|N5|N6|C4A|C4B|C5|C6|CLAIMS_R4a|CLAIMS_R4b|CLAIMS_R5|CLAIMS_R6|wt-r[456]|BIN[456]|WT[456]|grant_gate|GRANT)\b' \
  | sed -e 's/LEG_R[0-9A-B]*/LEGL/g'; }
gs4=$(gatesection "$P4"); gs5=$(gatesection "$P5"); gs6=$(gatesection "$P6")
ngs=$(printf '%s\n' "$gs4" | grep -c .)
[ "$ngs" -ge 100 ] && ok || bad "the gate-section extraction found only $ngs lines: the drift check would be vacuous"
[ "$gs4" = "$gs5" ] && ok || bad "the R4 and R5 packets' gate sections drifted apart: $(diff <(printf '%s\n' "$gs4") <(printf '%s\n' "$gs5") | head -4 | tr '\n' '|')"
d6=$(diff <(printf '%s\n' "$gs4") <(printf '%s\n' "$gs6"))
[ "$(printf '%s\n' "$d6" | grep -c '^<')" = 0 ] && ok || bad "the R6 packet is missing gate code the R4 packet has: $(printf '%s\n' "$d6" | grep '^<' | head -3 | tr '\n' '|')"
printf '%s\n' "$d6" | grep '^>' | grep -qi 'execution-grant' && ok || bad "the only code R6 adds over R4 is not its grant gate"
printf '%s\n' "$d6" | grep '^>' | grep -qE 'pp_check|claim_classify|wall_gate|bounds_check|scope_gate|oom_' && bad "R6 adds a second dialect of a gate R4 already has" || ok
grep -q 'grant_gate' "$P4" && bad "the grantable R4 packet grew a grant gate of its own" || ok
grep -q 'grant_gate' "$P6" && ok || bad "the held R6 packet lost its grant gate"
for f in $PKTS; do grep_in "$(basename "$f") carries the wall gate" "$f" 'wall_gate() {'; done
for f in "$SB/h4.sh" "$SB/h5.sh" "$SB/h6.sh"; do
  for fn in scope_gate bounds_check out_absent bin_sha oom_parse oom_key oom_pair oom_verdict oom_pair_verdict oom_snap claim_classify leg_state stray_verdict_lines names_gate tm cs_check register_claims book_case_set restore_booked_set; do
    grep -q "^$fn()" "$f" && ok || bad "$(basename "$f") does not define $fn"
  done
done
H="$SB/h4.sh"

echo "== the classifier (v7): per claim, inside an OBSERVED leg; the leg's exit code checked as a set"
printf 'PASS witness_inbox_stale_live_receipt_refuses\n' > "$SB/p"
printf 'FAIL witness_inbox_stale_live_receipt_refuses\n' > "$SB/f"
{ printf 'PASS a\n'; printf 'FAIL b\n'; } > "$SB/mix"
{ printf 'PASS a\n'; printf 'PASS b\n'; } > "$SB/allpass"
N=witness_inbox_stale_live_receipt_refuses
# inside an observed leg (the second argument is the leg's exit-code consistency, computed once for the
# whole leg by leg_rc_expect): a bare PASS holds and a bare FAIL is the falsified class.
check_out "a bare PASS inside an observed leg is HELD" 0 "HELD" bash -c "source '$H'; claim_classify '$SB/p' OK $N"
check_out "a bare FAIL inside an observed leg is the falsified class" 0 "FAILED" bash -c "source '$H'; claim_classify '$SB/f' OK $N"
# the classifier refuses to classify at all when the leg was not observed. Every non-OK token is a refusal;
# these are NOT vacuous, because the same files above do classify when the token is OK.
check_out "an unobserved leg yields NO_OBSERVATION for a PASS claim" 0 "NO_OBSERVATION" bash -c "source '$H'; claim_classify '$SB/p' MISMATCH $N"
check_out "an unobserved leg yields NO_OBSERVATION for a FAIL claim" 0 "NO_OBSERVATION" bash -c "source '$H'; claim_classify '$SB/f' MISMATCH $N"
check_out "an empty leg-consistency token is a refusal" 0 "NO_OBSERVATION" bash -c "source '$H'; claim_classify '$SB/p' '' $N"
# THE LEG EXIT CODE, over every claim in the leg: 0 when all passed, 1 when at least one failed.
check_out "all-pass with rc 0 is consistent" 0 "OK" bash -c "source '$H'; leg_rc_expect '$SB/allpass' 0 'a b'"
check_out "one pass one fail with rc 1 is consistent" 0 "OK" bash -c "source '$H'; leg_rc_expect '$SB/mix' 1 'a b'"
for rc in 1 124 137 2 99 missing; do
  check_out "all-pass with rc $rc is a mismatch" 0 "MISMATCH" bash -c "source '$H'; leg_rc_expect '$SB/allpass' $rc 'a b' | grep -q MISMATCH && echo MISMATCH"
done
for rc in 0 124 137 2 99 missing; do
  check_out "a leg with a bare FAIL and rc $rc is a mismatch" 0 "MISMATCH" bash -c "source '$H'; leg_rc_expect '$SB/mix' $rc 'a b' | grep -q MISMATCH && echo MISMATCH"
done
check_out "a name the log never printed does not change the rc verdict" 0 "OK" bash -c "source '$H'; leg_rc_expect '$SB/allpass' 0 'a b $N'"
check_out "the mismatch text says what was required" 0 "0 was required" bash -c "source '$H'; leg_rc_expect '$SB/allpass' 1 'a b' | grep -o '0 was required'"
check_out "the mismatch text counts the fails it saw" 0 "1 claim" bash -c "source '$H'; leg_rc_expect '$SB/mix' 0 'a b' | grep -o '1 claim'"
# a qualified line is never an outcome, and one is enough to make the whole leg unobservable
check_out "a reasoned FAIL is not a bare FAIL: rc 0 is still consistent with it" 0 "OK" bash -c "source '$H'; printf 'PASS a\nFAIL b (boom)\n' > '$SB/vq'; leg_rc_expect '$SB/vq' 0 'a b'"
check_out "…and the same leg at rc 1 is a mismatch, because a reasoned FAIL is not a countable one" 0 "0 was required" bash -c "source '$H'; printf 'PASS a\nFAIL b (boom)\n' > '$SB/vq'; leg_rc_expect '$SB/vq' 1 'a b' | grep -o '0 was required'"
echo "== leg state: one FAIL falsifies, one NO_OBSERVATION outranks it"
mk() { : > "$SB/$1"; for x in a b c; do echo "$x HELD" >> "$SB/$1"; done; }
mk L1; check "all HELD is AS_EXPECTED" 0 bash -c "source $H; [ \"\$(leg_state '$SB/L1' 'a b c')\" = AS_EXPECTED ]"
mk L2; sed -i 's/^b HELD/b FAILED/' "$SB/L2"; check "one FAILED is FALSIFIED" 0 bash -c "source $H; [ \"\$(leg_state '$SB/L2' 'a b c')\" = FALSIFIED ]"
mk L3; sed -i 's/^b HELD/b NO_OBSERVATION/' "$SB/L3"; check "one NO_OBSERVATION is NOOBS" 0 bash -c "source $H; [ \"\$(leg_state '$SB/L3' 'a b c')\" = NOOBS ]"
mk L4; sed -i '/^c /d' "$SB/L4"; check "a name with no row is NOOBS" 0 bash -c "source $H; [ \"\$(leg_state '$SB/L4' 'a b c')\" = NOOBS ]"
mk L5; sed -i 's/^a HELD/a HELD/; s/^b HELD/b FAILED/; s/^c HELD/c NO_OBSERVATION/' "$SB/L5"; check "NO_OBSERVATION outranks FAILED" 0 bash -c "source $H; [ \"\$(leg_state '$SB/L5' 'a b c')\" = NOOBS ]"
mk L6; echo "a HELD" >> "$SB/L6"; check "a duplicated row is NOOBS" 0 bash -c "source $H; [ \"\$(leg_state '$SB/L6' 'a b c')\" = NOOBS ]"

echo "== the OOM contract: one read, exact keys, exclusive publication BEFORE the parse, rise/fall/unknown"
mkdir -p "$SB/mem"; MEMDIR="$SB/mem"
printf 'low 0\nhigh 3\nmax 1\noom 0\noom_kill 0\nfail 0\ndiscard_oom 0\n' > "$SB/ev"
check "a clean memory.events parses to 0/0" 0 bash -c "source $H; MEMDIR='$SB/mem'; [ \"\$(oom_snap '$SB/ev' t)\" = 0/0 ]"
check "the raw bytes were published by exclusive create before the parse returned" 0 bash -c "ls '$SB/mem' | grep -q -- '-t.txt$'"
check "the published file is byte-identical to the source (not a re-render)" 0 bash -c "cmp -s '$SB/mem'/*-t.txt '$SB/ev'"
check "an unreadable counter returns UNKNOWN/UNKNOWN" 0 bash -c "source $H; MEMDIR='$SB/mem'; [ \"\$(oom_snap '$SB/nope' u)\" = UNKNOWN/UNKNOWN ]"
check "a missing evidence dir returns UNKNOWN/UNKNOWN rather than writing elsewhere" 0 bash -c "source $H; MEMDIR='$SB/no-such-dir'; [ \"\$(oom_snap '$SB/ev' w)\" = UNKNOWN/UNKNOWN ]"
mkdir -p "$SB/ro"; chmod 555 "$SB/ro"
check "a read-only evidence dir returns UNKNOWN/UNKNOWN (the publish failed)" 0 bash -c "source $H; MEMDIR='$SB/ro'; [ \"\$(oom_snap '$SB/ev' w)\" = UNKNOWN/UNKNOWN ]"; chmod 755 "$SB/ro"
check "two snapshots into the same dir get distinct sequence names (no clobber)" 0 bash -c "source $H; MEMDIR='$SB/mem'; oom_snap '$SB/ev' dup >/dev/null; oom_snap '$SB/ev' dup >/dev/null; [ \$(ls '$SB/mem' | grep -c -- '-dup.txt\$') = 2 ]"
check "oom_parse takes a whole-line digit value" 0 bash -c "source $H; [ \"\$(oom_parse 'oom 0' oom)\" = 0 ]"
check "oom_parse rejects a trailing field" 0 bash -c "source $H; [ \"\$(oom_parse 'oom 0 garbage' oom)\" = UNKNOWN ]"
check "oom_parse rejects a duplicate key" 0 bash -c "source $H; [ \"\$(oom_parse \$(printf 'oom 0\noom bad') oom)\" = UNKNOWN ]"
check "oom_parse rejects two valid duplicate lines" 0 bash -c "source $H; [ \"\$(oom_parse 'oom 0
oom 0' oom)\" = UNKNOWN ]"
check "oom_parse rejects a tab-separated record" 0 bash -c "source $H; [ \"\$(oom_parse \$(printf 'oom\t0') oom)\" = UNKNOWN ]"
check "oom_parse ignores a key that is a suffix of another" 0 bash -c "source $H; [ \"\$(oom_parse 'oom_kill 3
oom 0' oom)\" = 0 ]"
check "oom_parse reports UNKNOWN for an absent key" 0 bash -c "source $H; [ \"\$(oom_parse 'low 0' oom)\" = UNKNOWN ]"
check "a rise in oom_kill is OOM_KILLED" 0 bash -c "source $H; [ \"\$(oom_verdict 0 1)\" = OOM_KILLED ]"
check "a fall is COUNTER_INCOHERENT (cannot happen inside one cgroup's life)" 0 bash -c "source $H; [ \"\$(oom_verdict 1 0)\" = COUNTER_INCOHERENT ]"
check "no change is none" 0 bash -c "source $H; [ \"\$(oom_verdict 2 2)\" = none ]"
check "a non-digit counter is UNKNOWN" 0 bash -c "source $H; [ \"\$(oom_verdict 0 x)\" = UNKNOWN ]"
check "oom_pair_verdict reports a rise in the oom key too" 0 bash -c "source $H; [ \"\$(oom_pair_verdict 0/0 0/1)\" = OOM_KILLED ]"
check "oom_pair_verdict reports a fall in either key" 0 bash -c "source $H; [ \"\$(oom_pair_verdict 1/0 0/0)\" = COUNTER_INCOHERENT ]"
check "oom_pair_verdict propagates UNKNOWN" 0 bash -c "source $H; [ \"\$(oom_pair_verdict 0/UNKNOWN 0/0)\" = UNKNOWN ]"
check "oom_pair_verdict says none for an unchanged pair" 0 bash -c "source $H; [ \"\$(oom_pair_verdict 0/0 0/0)\" = none ]"

echo "== the bounds gate: every booked value compared, before any fetch or build"
mkdir -p "$SB/cg"; printf '34359738368\n' > "$SB/cg/memory.max"; printf '30064771072\n' > "$SB/cg/memory.high"
printf '0\n' > "$SB/cg/memory.swap.max"; printf '4096\n' > "$SB/cg/pids.max"; printf '3200000 100000\n' > "$SB/cg/cpu.max"
check "bounds gate accepts the booked cgroup" 0 bash -c "source $H; bounds_check '$SB/cg' >/dev/null"
for k in memory.max memory.high memory.swap.max pids.max cpu.max; do
  printf '9\n' > "$SB/cg/$k"
  check_out "bounds gate refuses a wrong $k with the value in the message" 2 "booked bounds" bash -c "source $H; BOOKED=\"\$(grep -m1 '^BOOKED=' $P4 | cut -d= -f2-)\"; m=\$(bounds_check '$SB/cg') || { echo \"booked bounds: \$m\"; exit 2; }; exit 0"
  case "$k" in memory.max) printf '34359738368\n' > "$SB/cg/$k";; memory.high) printf '30064771072\n' > "$SB/cg/$k";; memory.swap.max) printf '0\n' > "$SB/cg/$k";; pids.max) printf '4096\n' > "$SB/cg/$k";; cpu.max) printf '3200000 100000\n' > "$SB/cg/$k";; esac
done
printf 'max\n' > "$SB/cg/memory.max"
check "an unlimited cgroup (memory.max is the literal 'max') is refused, not adapted to" 0 bash -c "source $H; ! bounds_check '$SB/cg' >/dev/null"
printf '34359738368\n' > "$SB/cg/memory.max"; mv "$SB/cg/cpu.max" "$SB/cg/cpu.gone"
check "an unreadable counter is refused" 0 bash -c "source $H; ! bounds_check '$SB/cg' >/dev/null"
mv "$SB/cg/cpu.gone" "$SB/cg/cpu.max"
check "the bounds gate is a gate, not a dump: bounds_check output decides the exit" 0 bash -c "grep -q 'BM=\$(bounds_check' '$P4' && grep -q 'booked bounds' '$P4'"
check "and it runs before the first fetch" 0 bash -c "a=\$(grep -n 'BM=\$(bounds_check' '$P4' | cut -d: -f1); b=\$(grep -n 'CMD_FETCH_4\"' '$P4' | head -1 | cut -d: -f1); [ -n \"\$a\" ] && [ -n \"\$b\" ] && [ \"\$a\" -lt \"\$b\" ]"
check "…same in the held R6 packet" 0 bash -c "a=\$(grep -n 'BM=\$(bounds_check' '$P6' | cut -d: -f1); b=\$(grep -n 'CMD_FETCH\"' '$P6' | head -1 | cut -d: -f1); [ -n \"\$a\" ] && [ -n \"\$b\" ] && [ \"\$a\" -lt \"\$b\" ]"

echo "== the scope gate: end-anchored on the booked <JOB>.scope"
check "the booked scope path is accepted" 0 bash -c "source $H; [ \"\$(scope_gate '0::/user.slice/user-1000.slice/app.slice/fierce-wren-19-r4r5-db56530f.scope' fierce-wren-19-r4r5-db56530f)\" = '/user.slice/user-1000.slice/app.slice/fierce-wren-19-r4r5-db56530f.scope' ]"
sg() { check "scope gate refuses $1" 0 bash -c "source $H; ! scope_gate \"$2\" \"$3\" >/dev/null"; }
sg root-cgroup "0::/" fierce-wren-19-r4r5-db56530f
sg foreign-scope "0::/x/other.scope" fierce-wren-19-r4r5-db56530f
sg token-only-a-suffix "0::/x/prefierce-wren-19-r4r5-db56530f.scope" fierce-wren-19-r4r5-db56530f
sg scope-with-a-child "0::/x/fierce-wren-19-r4r5-db56530f.scope/sub" fierce-wren-19-r4r5-db56530f
sg not-cgroup-v2 "1::/x/fierce-wren-19-r4r5-db56530f.scope" fierce-wren-19-r4r5-db56530f
sg two-cgroup-lines "$(printf '0::/x/y.scope\n1::/z')" y

echo "== the launcher frame (item 1): argv, ledger rows, units, and no bare systemd-run"
for f in $PKTS; do
  b=$(basename "$f")
  grep_in "$b names the calmcrab-udrv.sh launcher" "$f" "calmcrab-udrv.sh"
  grep_in "$b states the full argv tuple" "$f" "<JOB> <RUNTIME_S> <MEMMAX_G> <MEMHIGH_G> <TASKS> <CPU%> <STAGED_SCRIPT> <SCRIPT_SHA> OUT"
  grep_in "$b states the -res ledger row" "$f" "-res"
  grep_in "$b states the -drv ledger row" "$f" "-drv"
  grep_in "$b states the driver unit" "$f" "calmcrab-drv-"
  grep_in "$b states the scope unit" "$f" ".scope"
  grep_not "$b does not launch with bare systemd-run" "$f" "systemd-run --"
  grep_not "$b does not book anything itself" "$f" "systemctl "
  grep_in "$b states the row ledger row by name" "$f" "$(grep -m1 '^J=' "$f" | cut -d= -f2)       cap 0"
  grep_in "$b states the reservation ledger row" "$f" "$(grep -m1 '^J=' "$f" | cut -d= -f2)-res"
  grep_in "$b states the driver ledger row" "$f" "$(grep -m1 '^J=' "$f" | cut -d= -f2)-drv"
  grep_in "$b books the R row in GiB, not bytes" "$f" "-res   cap 32 (GiB"
  grep_not "$b does not book the R row in bytes" "$f" "-res            cap ="
# DESIGN 3 style rule, enforced on the packets: a line number in a file this size is a second naming
# scheme that any edit above it silently invalidates. Name the symbol, and let a run log hold the line.
for f in $PKTS; do grep -qE 'claim_batch\.rs:[0-9]+' "$f" && bad "$(basename "$f") cites claim_batch.rs by line number instead of by symbol" || ok; done
  grep_in "$b states the driver unit" "$f" "calmcrab-drv-$(grep -m1 "^J=" "$f" | cut -d= -f2).service"
done
check "the R4 job token names its own row job" 0 bash -c "[ \"\$(grep -m1 '^J=' '$P4' | cut -d= -f2)\" = fierce-wren-19-r4-db56530f ]"
check "the R5 job token names its own row job" 0 bash -c "[ \"\$(grep -m1 '^J=' '$P5' | cut -d= -f2)\" = fierce-wren-19-r5-dff57af7 ]"
check "the split is real: no job clones or builds the other row's pin" 0 bash -c "grep -q 'pin_clone \"\$WT4\"' '$P5' && exit 1; grep -q 'pin_clone \"\$WT5\"' '$P4' && exit 1; exit 0"
check "the R6 job token is the row name" 0 bash -c "[ \"\$(grep -m1 '^J=' '$P6' | cut -d= -f2)\" = fierce-wren-19-r6-dff57af7 ]"
check "JOB is derived from the launcher's OUT, not assumed" 0 bash -c "grep -q 'JOB=\$(basename \"\$(dirname \"\$OUT\")\"' '$P4' && grep -q 'JOB=\$(basename \"\$(dirname \"\$OUT\")\"' '$P6'"
check "and the derivation is enforced: a job dir mismatch refuses the run" 0 bash -c "source $H; ! scope_gate '0::/x/wrong-job.scope' wrong-job2 >/dev/null"

echo "== the exit contract (item 4): 0 held, 1 falsified, 2 everything else"
for f in $PKTS; do
  b=$(basename "$f")
  code=$(grep -vE '^[[:space:]]*#' "$f")
  n1=$(printf '%s\n' "$code" | grep -cE '\bexit 1\b'); [ "$n1" = 1 ] && ok || bad "$b has $n1 executable 'exit 1' (want exactly 1: the falsified branch)"
  n3=$(printf '%s\n' "$code" | grep -cE '\bexit 3\b'); [ "$n3" = 0 ] && ok || bad "$b still has an 'exit 3' ($n3)"
  n2=$(printf '%s\n' "$code" | grep -cE '\bexit 2\b'); [ "$n2" -ge 8 ] && ok || bad "$b has only $n2 exit-2 refusals"
  grep_in "$b routes every gate refusal through the nox helper" "$f" "nox() {"
  check "$b's nox helper can only exit 2" 0 bash -c "grep 'nox() {' '$f' | grep -q 'exit 2; }' || grep -A3 'nox() {' '$f' | grep -q 'exit 2'"
done
check_out "usage with no argument is NO_OBSERVATION (exit 2)" 2 "NO_OBSERVATION" bash "$P4"
check_out "usage with two arguments is NO_OBSERVATION (exit 2)" 2 "NO_OBSERVATION" bash "$P4" a b
check_out "an unknown flag is not swallowed as a path (exit 2)" 2 "NO_OBSERVATION" bash "$P4" --nope
check_out "a run dir that is not a job's out dir is refused (exit 2)" 2 "does not end in /out" bash "$P4" "$SB/notout"
check "…and nothing was created for it" 0 bash -c "[ ! -e '$SB/notout' ]"
check_out "held R6 with no argument is NO_OBSERVATION (exit 2)" 2 "NO_OBSERVATION" bash "$P6"
R6JOBOUT=$SB/$(grep -m1 "^J=" "$P6" | cut -d= -f2)/out   # the booked job name, not a made-up one
check_out "held R6 with OUT but no grant reference refuses (exit 2)" 2 "msg_3407a188" bash "$P6" "$R6JOBOUT"

check "…and the hold leaves nothing behind (no OUT, no clone)" 0 bash -c "[ ! -e '$R6JOBOUT' ]"
check_out "held R6 refuses a flag where the grant reference belongs (exit 2)" 2 "execution-grant reference" bash "$P6" "$R6JOBOUT" --yes
check "…still nothing behind it" 0 bash -c "[ ! -e '$R6JOBOUT' ]"
check_out "held R6 --predict-only states the hold and exits 0" 0 "state=HELD" bash "$P6" --predict-only
check "…and --predict-only creates nothing" 0 bash -c "[ ! -e '$SB/predict' ] && cd '$SB' && bash '$P6' --predict-only >/dev/null"
check "the grant gate refuses blank, whitespace and flag-like references" 0 bash -c "source $(helpers "$P6" > $SB/h6b.sh; echo $SB/h6b.sh); grant_gate '' >/dev/null && exit 1; grant_gate '  ' >/dev/null && exit 1; grant_gate '--x' >/dev/null && exit 1; grant_gate 'keen-otter-cleared' >/dev/null"

echo "== the frame's own claims are testable: timeouts, wall arithmetic, no mutation"
for f in $PKTS; do
  b=$(basename "$f")
  n=$(grep -cE 'timeout "\$T_(SETUP_CLONE|SETUP_FETCH|SETUP_CHECKOUT|BUILD|LEG)"' "$f")
  [ "$n" -ge 5 ] && ok || bad "$b wraps only $n steps in a frozen timeout"
  m=$(grep -n 'mutation' "$f" | grep -vE '^[0-9]+:[[:space:]]*#' | grep -vc 'mutation_legs=0\|no mutation leg')
  [ "$m" = 0 ] && ok || bad "$b mentions 'mutation' in $m executable lines (a presence row must define none)"
  for badtok in 'git push' 'git commit' 'ssh ' 'scp ' 'rsync ' 'patch -' 'git apply' 'checkout -b' '/opt/gunbc'; do
    hits=$(grep -n -- "$badtok" "$f" | grep -vE '^[0-9]+:[[:space:]]*#' | wc -l | tr -d ' ')
    [ "$hits" = 0 ] && ok || bad "$b uses '$badtok' in $hits executable lines"
  done
done
echo "== the booked bounds are the launcher's numbers, in the launcher's units (v4 blocker 1)"
for f in $PKTS; do
  b=$(basename "$f")
  eval "$(grep -E '^(RUNTIME_S|MEMMAX_G|MEMHIGH_G|TASKS_MAX|CPU_PCT|BOOKED)=' "$f")"
  want="memory.max=$((MEMMAX_G*1073741824)) memory.high=$((MEMHIGH_G*1073741824)) memory.swap.max=0 pids.max=$TASKS_MAX cpu.max=$((CPU_PCT*1000))_100000"
  [ "$BOOKED" = "$want" ] && ok || bad "$b freezes bounds ($BOOKED) that are not the GiB/pids/cpu booking it states ($want)"
  grep_in "$b passes RUNTIME in seconds" "$f" "calmcrab-udrv.sh $(grep -m1 '^J=' "$f" | cut -d= -f2) $RUNTIME_S $MEMMAX_G $MEMHIGH_G $TASKS_MAX $CPU_PCT% <STAGED_SCRIPT>"
  grep_in "$b hashes the PACKET, not a commit, in the launcher argv" "$f" "<STAGED_SCRIPT> <v7 packet sha256> OUT"
done

for f in $PKTS; do
  b=$(basename "$f")
  eval "$(grep -E '^(RUNTIME_S|DRIVER_S|LAUNCHER_CAP|N_LEGS|T_SETUP_CLONE|T_SETUP_FETCH|T_SETUP_CHECKOUT|T_BUILD|T_LEG)=' "$f")"
  sum=$(( T_SETUP_CLONE + T_SETUP_FETCH + T_SETUP_CHECKOUT + T_BUILD + N_LEGS*T_LEG ))
  pr=$(bash "$f" --predict-only)
  printf '%s\n' "$pr" | grep -qF "scope_RuntimeMaxSec=$RUNTIME_S" && ok || bad "$b's launcher RUNTIME_S ($RUNTIME_S) is not the scope wall its manifest states"
  printf '%s\n' "$pr" | grep -qF "wall_capped_sum_s=$sum " && ok || bad "$b states a capped step sum that is not the sum of its own frozen timeouts ($sum)"
  printf '%s\n' "$pr" | grep -qF "driver_RuntimeMaxSec=$((RUNTIME_S+300))" && ok || bad "$b's manifest does not size the driver unit at RUNTIME_S plus 300"
  case "$b" in
    acceptance_r4_*) exp="5400 4920 2";;
    acceptance_r5_*) exp="4500 4020 1";;
    acceptance_r6_*) exp="4800 4020 1";;
  esac
  set -- $exp
  [ "$RUNTIME_S" = "$1" ] && [ "$sum" = "$2" ] && [ "$N_LEGS" = "$3" ] && [ "$DRIVER_S" = "$(( $1 + 300 ))" ] \
    && ok || bad "$b's booking is not the v6 split's: want RUNTIME_S $1, sum $2, legs $3, driver $(( $1 + 300 )); got $RUNTIME_S, $sum, $N_LEGS, $DRIVER_S"
  [ "$RUNTIME_S" -le "$LAUNCHER_CAP" ] && ok || bad "$b books RUNTIME_S $RUNTIME_S above the installed launcher cap $LAUNCHER_CAP"
  [ "$sum" -le "$RUNTIME_S" ] && ok || bad "$b's capped step sum $sum exceeds its scope wall $RUNTIME_S"
  [ "$DRIVER_S" = "$((RUNTIME_S+300))" ] && ok || bad "$b sizes its driver unit at $DRIVER_S, not RUNTIME_S plus 300"
  grep_in "$b refuses an over-cap booking at its own pre-work gate" "$f" 'WM=$(wall_gate "$WALL_SUM" "$RUNTIME_S" "$LAUNCHER_CAP" "$DRIVER_S")'
  grep_in "$b freezes the installed launcher cap as a constant" "$f" 'LAUNCHER_CAP=7200'
done
# the three bookings, restated here independently of the packets' own arithmetic, so a packet cannot
# satisfy this file by moving a constant around inside itself
check "the v6 arithmetic, restated: R4 4920<=5400, R5 4020<=4500, R6 4020<=4800, each <= 7200, driver +300" 0 bash -c '
  for spec in "5400 4920 5700" "4500 4020 4800" "4800 4020 5100"; do
    set -- $spec
    [ "$2" -le "$1" ] || exit 1
    [ "$1" -le 7200 ] || exit 1
    [ "$3" = "$(( $1 + 300 ))" ] || exit 1
  done
  exit 0'
check "the R6 header still states the launcher's full argv and its own ledger rows" 0 bash -c "grep -q 'fierce-wren-19-r6-dff57af7-res' '$P6' && grep -q 'fierce-wren-19-r6-dff57af7-drv' '$P6'"

echo "== the OOM contract as written: exclusive publication before parse, no grep-judged counters"
for f in $PKTS; do
  b=$(basename "$f")
  grep_in "$b publishes the snapshot by exclusive create" "$f" "set -C"
  grep_not "$b never judges memory.events with a grep pattern" "$f" "grep -E '^(oom_kill|max) [1-9]"
  grep_not "$b never silently cats a judged counter" "$f" "cat /sys/fs/cgroup/memory.events"
  ns=$(grep -c 'oom_snap ' "$f"); nv=$(grep -c 'oom_pair_verdict' "$f")
  [ "$nv" -ge 2 ] && ok || bad "$b has $ns snapshots but only $nv verdicts"
done
for f in $PKTS; do nv=$(grep -c 'oom_pair_verdict' "$f"); [ "$nv" -ge 4 ] && ok || bad "$(basename "$f") compares snapshots only $nv times (gate, build, legs, final)"; done
check "memory.peak is printed, never judged" 0 bash -c "grep -q 'memory.peak (printed, never judged)' '$P4' && grep -q 'memory.peak (printed, never judged)' '$P6'"
check "the R4/R5 packet compares the FINAL snapshot against the last leg" 0 bash -c "grep -q 'oom_pair_verdict \"\$LASTOOM\" \"\$FA\"' '$P4' && grep -q 'oom_pair_verdict \"\$LASTOOM\" \"\$FA\"' '$P6'"

echo "== the classifier matches what the PINNED runner actually prints"
RBIN=src/v1/stage0/src/bin/claim_batch.rs
pinned_at() { local id; id=$(g -C "$REPO" rev-parse "$1:$2" 2>/dev/null) || true; printf '%s' "${id:-}"; }
worktree_file_equals_blob() { local h; h=$(g -C "$REPO" hash-object -- "$1" 2>/dev/null) || true; [ -n "$h" ] && [ "$h" = "$2" ]; }
if [ -n "$REPO" ] && g -C "$REPO" rev-parse --git-dir >/dev/null 2>&1; then
  RB=$(pinned_at "$H4" "$RBIN"); [ "$RB" = "$(pinned_at "$H5" "$RBIN")" ] && ok || bad "claim_batch.rs differs between the two pins; the classifier assumes one shape"
  if worktree_file_equals_blob "$RBIN" "$RB"; then
    grep_in "the pinned runner prints a bare PASS line" "$REPO/$RBIN" 'ClaimOutcome::Pass => println!("PASS {}", function)'
    grep_in "the pinned runner prints a bare FAIL line" "$REPO/$RBIN" 'println!("FAIL {}", function);'
    for lit in 'UNOBSERVED {} (' 'PANICKED {} (' 'FAIL {} (runtime error' 'FAIL {} (returned ProcessExit::ExitFailure'; do
      grep_in "the pinned runner's qualified line exists: ${lit:0:14}" "$REPO/$RBIN" "$lit"
    done
    grep_in "the pinned runner exits 1 on any failure, 2 on any unobserved" "$REPO/$RBIN" 'if any_failed || partition_failed {'
    grep_in "…and 2 for unobserved" "$REPO/$RBIN" '} else if any_unobserved || partition_unobserved {'
    for flag in --source-root --entry --function --functions --claim-run --wet --hermetic --record --eval-budget-ms --fixture-store --pre-push --print-entry-closure --outcomes-out; do
      grep_in "parser accepts $flag" "$REPO/$RBIN" "\"$flag\" =>"
    done
    grep_in "parser splits --functions on a comma" "$REPO/$RBIN" "csv.split(',')"
    grep_in "parser rejects an unknown argument with exit 2" "$REPO/$RBIN" "unknown argument"
    grep_in "the pinned runner renders the completion record with eprintln (stderr)" "$REPO/$RBIN" 'eprintln!("{}", partition.render(&timings));'
    n=$(grep -cF 'partition.render(&timings)' "$REPO/$RBIN"); [ "$n" = 1 ] && ok || bad "the pinned claim_batch.rs renders the completion record at $n sites, expected exactly 1"
    check_out "the packet's own producer_gate accepts the pinned claim_batch.rs bytes" 0 OK bash -c "source '$H'; producer_gate '$REPO/$RBIN'"
    for f in $PKTS; do grep_in "$(basename "$f") pins the producer blob id that git reports at its pin" "$f" "PRODBLOB=$RB"; done
    skip_if=0
  else
    skip "claim_batch.rs at the pin is not byte-identical to the working copy: parser checks skipped"
  fi
  for pin in "$H4" "$H5"; do
    [ "$(g -C "$REPO" cat-file -t "$pin" 2>/dev/null)" = commit ] && ok || bad "pin $pin is not a commit in the repo"
  done
  [ "$(g -C "$REPO" rev-parse "$H4^{tree}")" = "$T4" ] && ok || bad "the R4 tree gate would refuse: tree at $H4 is $(g -C "$REPO" rev-parse "$H4^{tree}"), packet freezes $T4"
  [ "$(g -C "$REPO" rev-parse "$H5^{tree}")" = "$T5" ] && ok || bad "the R5 tree gate would refuse: tree at $H5 is $(g -C "$REPO" rev-parse "$H5^{tree}"), packet freezes $T5"
  check "the R5 pin (the #15936 squash) is an ancestor of the R4 pin (main at freeze)" 0 bash -c "git -C '$REPO' merge-base --is-ancestor '$H5' '$H4'"
  OM=$(g -C "$REPO" rev-parse origin/main)
  check "the R4 pin is an ancestor of (or equal to) origin/main" 0 bash -c "git -C '$REPO' merge-base --is-ancestor '$H4' origin/main"
  echo "  (origin/main now reads $OM; the packets freeze $H4 / $H5)"
  for pair in "$E4A:$B4A" "$E4B:$B4B" "$E5:$B5" "$E6:$B6"; do
    p="${pair%%:*}"; want="${pair##*:}"
    [ "$(pinned_at "$OM" "$p")" = "$want" ] && ok || bad "origin/main blob of $p is $(pinned_at "$OM" "$p"), the packets freeze $want"
    [ "$(pinned_at "$H4" "$p")" = "$want" ] && ok || bad "$p at the R4 pin is not the frozen blob"
    [ "$(pinned_at "$H5" "$p")" = "$want" ] && ok || bad "$p at the R5/R6 pin is not the frozen blob"
  done
  check_name() { local label="$1" pin="$2" path="$3" blob="$4" name="$5"
    if ! worktree_file_equals_blob "$path" "$blob"; then skip "$label: cannot read the pinned bytes of $path offline (partial clone)"; return; fi
    grep -qE "^test fn $name\(" "$REPO/$path" && ok || bad "$label: '$name' is not a test fn in $path at $pin ($blob)"; }
  for n in $N4A; do check_name "R4/a" "$H4" "$E4A" "$B4A" "$n"; done
  for n in $N4B; do check_name "R4/b" "$H4" "$E4B" "$B4B" "$n"; done
  for n in $N5;  do check_name "R5"  "$H5" "$E5"  "$B5"  "$n"; done
  for n in $N6;  do check_name "R6"  "$H5" "$E6"  "$B6"  "$n"; done
  # the names gate uses the same spelling as the claim module
  if worktree_file_equals_blob "$E5" "$B5"; then
    n=$(grep -c "^test fn " "$REPO/$E5"); [ "$n" -ge 5 ] && ok || bad "the names gate pattern '^test fn ' matches only $n entries in $E5"
  fi
  # the R6 audit's load-bearing lines, each against the bytes AT THE PIN
  grep_at_pin() { local label="$1" pin="$2" path="$3" lit="$4" id
    id=$(pinned_at "$pin" "$path")
    if [ -z "$id" ]; then bad "$label: $path absent at $pin"; return; fi
    if ! worktree_file_equals_blob "$path" "$id"; then skip "$label: $path differs from the pin ($id); byte-identity check skipped"; return; fi
    grep -qF -- "$lit" "$REPO/$path" && ok || bad "$label: '$lit' not found in $path at $pin ($id)"; }
  grep_at_pin "R6 audit" "$H5" "dag/gunbc/fabric/fabric_storage_binding.dag" "FabricStoragePlaced { host: _, instance: _, endpoint, door_socket: _ } => FabricStorageServed {"
  grep_at_pin "R6 audit" "$H5" "dag/gunbc/deployment_risk.dag" "data prod_role_selection: ProdRoleSelection = ProdRoleHeldBy { deployment: srv1_daily_workspace_deployment }"
  grep_at_pin "R6 audit" "$H5" "dag/gunbc/roadmap/roadmap_dashboard_instance.dag" "fn srv1_live_dashboard_instance() -> HostDashboardInstance {"
  grep_at_pin "R6 audit" "$H5" "dag/gunbc/roadmap/roadmap_dashboard_instance.dag" "    deployment: srv1_daily_workspace_deployment,"
  grep_at_pin "R6 audit" "$H5" "dag/gunbc/roadmap/roadmap_operator_acceptance.dag" "FabricStorageBindingUnplaced => load_roadmap_acceptance_event_history"
  grep_at_pin "R6 audit" "$H5" "dag/gunbc/fabric/fabric_event_log.dag" "fn event_log_read_partition_with<P>(store: FabricStorageBinding, partition: PartitionId, budget: Nat, decode: fn(String) -> EventDecode<P>) -> PartitionRead<P>"
  grep_at_pin "R6 audit" "$H5" "dag/gunbc/fabric/fabric_storage_client.dag" "match served_post(endpoint: e, operation: \"closure\", request: join([name as String, \" \", to_string(bound)], \"\")) {"
  grep_at_pin "R6 audit" "$H5" "$E6" "data live_tree_disposition: LiveTreeDisposition = ReadsLiveTree"
  grep_at_pin "R6 audit (no writes)" "$H5" "dag/gunbc/fabric/fabric_event_log.dag" "fn event_log_append_with<P>("
else
  echo "  (no repo argument: the constant/object/parser/audit checks are SKIPPED, not passed)"
fi

echo "== the ARGV, tokenized exactly as bash -c tokenizes it, against the pinned parser's flag set"
tok() { # tok <CMDVAR> <binvar> -> one token per line, as `bash -c "$CMD…"` hands them to exec
  local var=$1 binvar=$2
  local bv; bv=${!var}
  printf '%s' "$bv" | grep -q . || return 0
  ( BIN4="$S4"; BIN5="$S5"; BIN6="$S6"; OUT="$SB/out"; eval "set -- $bv" 2>/dev/null
    for t in "$@"; do printf '%s\n' "$t"; done )
}
flag_value() { # flag_value <CMDVAR> <flag> -> the value that flag carries in the tokenized command
  local var=$1 flag=$2 bv; bv=${!var}
  ( BIN4=$S4; BIN5=$S5; BIN6=$S6; OUT=$SB/out; eval "set -- $bv" 2>/dev/null
    while [ $# -gt 1 ]; do if [ "$1" = "$flag" ]; then printf '%s' "$2"; return; fi; shift; done )
}
FLAGS_ONE='--source-root --entry --function --functions --eval-budget-ms --fixture-store --outcomes-out'
FLAGS_ZERO='--claim-run --wet --hermetic --record --pre-push --print-entry-closure'
argv_check() { # argv_check <label> <CMDVAR> <sentinel> <expected names…>
  local label=$1 var=$2 sentinel=$3; shift 3
  local -a T; mapfile -t T < <(tok "$var" "")
  [ "${#T[@]}" -gt 0 ] || { bad "$label: $var does not tokenize"; return; }
  [ "${T[0]}" = "$sentinel" ] || { bad "$label: argv[0] is '${T[0]}', not the built runner path"; return; }
  case "${T[0]}" in */claim_batch) ;; *) bad "$label: argv[0] does not end in /claim_batch"; return;; esac
  [ "${T[1]:-}" != claim_batch ] || { bad "$label: argv[1] is the literal token 'claim_batch'"; return; }
  local i=1 nfn=0 n bad_tok="" csvv name
  while [ $i -lt ${#T[@]} ]; do
    local t=${T[$i]}
    case " $FLAGS_ONE " in *" $t "*)
      [ $((i+1)) -lt ${#T[@]} ] || { bad "$label: $t has no value"; return; }
      if [ "$t" = --functions ]; then
        csvv=${T[$((i+1))]}
        IFS=',' read -r -a parts <<<"$csvv"
        [ "${#parts[@]}" -gt 0 ] || { bad "$label: --functions '$csvv' has no fields"; return; }
        for name in "${parts[@]}"; do
          nfn=$((nfn+1))
          [ -n "$name" ] || { bad "$label: empty field in --functions '$csvv'"; return; }
          case "$name" in *[[:space:]]*) bad "$label: --functions field '$name' contains whitespace"; return;; esac
          printf '%s' "$name" | grep -qE '^[a-z0-9_]+$' || { bad "$label: --functions field '$name' is outside ^[a-z0-9_]+$"; return; }
        done
        printf '%s' "$csvv" | grep -q ' ,\|, ' && { bad "$label: --functions '$csvv' has spaces around a comma"; return; }
      fi
      i=$((i+2)); continue;; esac
    case " $FLAGS_ZERO " in *" $t "*) i=$((i+1)); continue;; esac
    case "$t" in --*) bad_tok=$t; break;; *) bad "$label: bare non-flag argument '$t'"; return;; esac
  done
  [ -z "$bad_tok" ] || { bad "$label: token '$bad_tok' is outside the parser's accepted flag set"; return; }
  [ "$nfn" -eq $# ] || { bad "$label: $nfn function names, expected $#"; return; }
  ok
}
argv_check "R4/a argv" CMD_LEG_R4A "$S4" $N4A
argv_check "R4/b argv" CMD_LEG_R4B "$S4" $N4B
argv_check "R5 argv"   CMD_LEG_R5  "$S5" $N5
argv_check "R6 argv"   CMD_LEG_R6  "$S6" $N6
for v in CMD_LEG_R4A CMD_LEG_R4B CMD_LEG_R5 CMD_LEG_R6; do
  n=$(tok "$v" "" | grep -c '^--source-root$'); [ "$n" = 3 ] && ok || bad "$v passes $n --source-root values (want 3)"
  for want in dag src/v1 src/v2; do tok "$v" "" | grep -qxF -- "$want" && ok || bad "$v is missing the source root '$want'"; done
  ent=$(flag_value "$v" --entry)
  case "$v" in *R4A) want=$E4A;; *R4B) want=$E4B;; *R5) want=$E5;; *) want=$E6;; esac
  [ "$ent" = "$want" ] && ok || bad "$v's --entry is '$ent', not the frozen entry path"
done
for pair in "CMD_LEG_R4A:C4A" "CMD_LEG_R4B:C4B" "CMD_LEG_R5:C5" "CMD_LEG_R6:C6"; do
  v=${pair%%:*}; c=${pair##*:}
  got=$(flag_value "$v" --functions)
  [ "$got" = "${!c}" ] && ok || bad "$v's --functions is '$got', not the frozen ${c}"
done
for b in $PKTS; do
  for v in $(grep -oE '^CMD_[A-Z0-9_]+' "$b"); do
    grep -q "\"\$$v\"" "$b" && ok || bad "$(basename "$b") never uses \$$v — the manifest would print a string the run does not execute"
  done
  grep -q 'leg R4a "\$CMD_LEG_R4A"' "$b" && ok || true
done
grep -q 'leg R4a "\$CMD_LEG_R4A"' "$P4" && ok || bad "the R4/a leg does not pass the manifest string to the leg runner"
grep -q 'leg R4b "\$CMD_LEG_R4B"' "$P4" && ok || bad "the R4/b leg does not pass the manifest string to the leg runner"
grep -q 'leg R5  "\$CMD_LEG_R5"' "$P5" && ok || bad "the R5 leg does not pass the manifest string to the leg runner"
grep -q 'leg R6 "\$CMD_LEG_R6"' "$P6" && ok || bad "the R6 leg does not pass the manifest string to the leg runner"
check "the built runner's path is EXPORTED to the child shell a bash -c leg runs in" 0 bash -c "grep -q 'export OUT BIN4' '$P4' && grep -q 'export OUT BIN5' '$P5' && grep -q 'export OUT' '$P6' && grep -q 'export BIN6' '$P6'"
check "…because a plain assignment does not reach a child shell (a latent v2 defect this rewrite fixes)" 0 bash -c "[ -z \"\$(v=1; bash -c 'printf %s \"\$v\"')\" ]"

echo "== the manifest: bytes and sha256 for every command string, printed by the packet itself"
M4=$(bash "$P4" --predict-only); M5=$(bash "$P5" --predict-only); M6=$(bash "$P6" --predict-only)
n=$(printf '%s\n' "$M4" | grep -c '^cmd '); [ "$n" = 6 ] && ok || bad "the R4 manifest prints $n cmd lines (want 6: setup x4 + R4a + R4b)"
n=$(printf '%s\n' "$M5" | grep -c '^cmd '); [ "$n" = 5 ] && ok || bad "the R5 manifest prints $n cmd lines (want 5)"
n=$(printf '%s\n' "$M6" | grep -c '^cmd '); [ "$n" = 5 ] && ok || bad "the R6 manifest prints $n cmd lines (want 5)"
n=$(printf '%s\n' "$M4" | grep -c '^blob '); [ "$n" = 2 ] && ok || bad "the R4 manifest prints $n blob lines (want 2 entry blobs)"
n=$(printf '%s\n' "$M5" | grep -c '^blob '); [ "$n" = 1 ] && ok || bad "the R5 manifest prints $n blob lines (want 1)"
n=$(printf '%s\n' "$M6" | grep -c '^blob '); [ "$n" = 1 ] && ok || bad "the R6 manifest prints $n blob lines (want 1)"
printf '%s\n' "$M4" | grep -qE "^pin_r4=$H4( |$)" && ok || bad "the R4 manifest does not print the frozen R4 pin"
printf '%s\n' "$M4" | grep -qE "tree_r4=$T4" && ok || bad "the R4 manifest does not print the frozen R4 tree"
printf '%s\n' "$M5" | grep -qE "^pin_r5=$H5( |$)" && ok || bad "the R5 manifest does not print the frozen R5 pin"
printf '%s\n' "$M5" | grep -qE "tree_r5=$T5" && ok || bad "the R5 manifest does not print the frozen R5 tree"
HPIN6=$(sed -n 's/^H=\([0-9a-f]\{40\}\).*/\1/p' "$P6" | head -1)
TPIN6=$(sed -n 's/.*; T=\([0-9a-f]\{40\}\).*/\1/p' "$P6" | head -1)
[ -n "$HPIN6" ] && [ -n "$TPIN6" ] && ok || bad "the R6 packet's pin/tree constants could not be read: the two checks below would be vacuous"
printf '%s\n' "$M6" | grep -qE "^pin=$HPIN6( |$)" && ok || bad "the R6 manifest does not print its frozen pin"
printf '%s\n' "$M6" | grep -qE "tree=$TPIN6" && ok || bad "the R6 manifest does not print its frozen tree"
# v6: the per-row command inventory must be BYTE-IDENTICAL to v4/v5's, row by row. These digests are the
# v4 digests, recorded from the v4 run of this self-test; a changed command string here is a changed leg.
check "the R4 row's six command strings are byte-identical to v4/v5's" 0 env PKT="$P4" bash -c '
  want="f256a1da5244bfd6b62af2e05eb608b6b15e99fda2deda43df961af728917fe5 4a0ac69f9b81df39536466bb77e843488984137f853bbcf1b90f32dad9d78cab fdb0ba1965609f46b72f4eb4ead4fb3a870a897c6bd01bb4bbdc88b6e4331911 b9eb7b3c3870d6bcc728886a54201223fa01f449790eec91d096ce35766769a5 453568e5420cb9f95c34f54a7ddc1c8b252a7a531d29e60b6d0b358bcd741e05 98211ed70494ef8c9b85eed3b1e39e4cfd29b975d8b059cf185a11c92231d652"
  got=$(bash "$PKT" --predict-only | grep "^cmd " | cut -d= -f3 | tr "\\n" " " | sed "s/ *$//")
  [ "$got" = "$want" ] || { echo "want $want"; echo "got  $got"; exit 1; }'
check "the R5 row's five command strings are byte-identical to v4/v5's" 0 env PKT="$P5" bash -c '
  want="1c90e94474e67011795be02913ba5cb6c341346f03365a96bb884aea6e82d586 1b03a85fa7cfa0244152c9a52bae422b10db9905ed5214bf92a1f8ab7f8ade19 220bd6bb5942cb5467d4099b7d8c749ede44667d978ac32d54099a4d2809fca4 b9eb7b3c3870d6bcc728886a54201223fa01f449790eec91d096ce35766769a5 c5f7fe87f6130cc3ad2cf1f60dc99be4457f0e81a419b7025f8bb81c674ceda7"
  got=$(bash "$PKT" --predict-only | grep "^cmd " | cut -d= -f3 | tr "\\n" " " | sed "s/ *$//")
  [ "$got" = "$want" ] || { echo "want $want"; echo "got  $got"; exit 1; }'
check "the R6 row's five command strings are byte-identical to v4/v5's" 0 env PKT="$P6" bash -c '
  want="995b533e654e1c8b6aa68add106d1df71e57699aa2be4dfd6516a793cf9e9de2 4500968dca5ab55989dd67fb8468a8ebd3749074fc85674f964cdda4e68db56a 85a784694b36535a3df3822daf36611aecca3a4d12163bb96910c6228a40eff0 b9eb7b3c3870d6bcc728886a54201223fa01f449790eec91d096ce35766769a5 0053be34fb87d635cd5664cccd7880bd014a18417ca24bfcc1c9b61cd512be7a"
  got=$(bash "$PKT" --predict-only | grep "^cmd " | cut -d= -f3 | tr "\\n" " " | sed "s/ *$//")
  [ "$got" = "$want" ] || { echo "want $want"; echo "got  $got"; exit 1; }'
for m in M4 M5 M6; do
  printf '%s\n' "${!m}" | grep -qE '^cmd CMD_[A-Z0-9_]+ bytes=[0-9]* sha256=[0-9a-f]{64}$' && ok || bad "$m has a cmd line without bytes+sha256"
  printf '%s\n' "${!m}" | grep -qE '^launcher_preflight_cap=7200' && ok || bad "$m does not state the installed launcher cap"
  printf '%s\n' "${!m}" | grep -qE '^driver_service=[^ ]* RuntimeMaxSec=[0-9]+ \(RUNTIME_S plus 300 s\)' && ok || bad "$m does not size the driver unit from RUNTIME_S"
  printf '%s\n' "${!m}" | grep -qE '^bounds=memory.max=34359738368' && ok || bad "$m does not print the booked bounds"
  printf '%s\n' "${!m}" | grep -qE '^wall_capped_sum_s=[0-9]+ scope_RuntimeMaxSec=[0-9]+ driver_RuntimeMaxSec=[0-9]+ launcher_cap=7200 legs=[0-9]+$' && ok || bad "$m's wall line is not the arithmetic its own gate checks"
  printf '%s\n' "${!m}" | grep -qE -- '--source-root dag --source-root src/v1 --source-root src/v2 --entry' && ok || bad "$m lost the frozen argv shape"
  printf '%s\n' "${!m}" | grep -qE 'cmd CMD_LEG[^=]*=[^\n]*claim_batch --source-root' && bad "$m's leg command carries the subcommand token again" || ok
done
printf '%s\n' "$M6" | grep -qE '^PREDICT-ONLY row=R6 .* state=HELD gate=keen-otter-msg_3407a188' && ok || bad "the R6 manifest does not state the hold and its gate"
printf '%s\n' "$M4" | grep -qE '^PREDICT-ONLY row=R4 job=fierce-wren-19-r4-db56530f host_booked=srv1' && ok || bad "the R4 manifest does not name its own row and job"
printf '%s\n' "$M5" | grep -qE '^PREDICT-ONLY row=R5 job=fierce-wren-19-r5-dff57af7 host_booked=srv1' && ok || bad "the R5 manifest does not name its own row and job"
for m in M4 M5; do printf '%s\n' "${!m}" | grep -qE 'script_sha256_frozen=see-tools/acceptance_r4_r5_r6_v8_manifest\.md' && ok || bad "$m defers its own sha256 to the manifest without stating it"; done
for m in M4 M5 M6; do
  f=$(eval printf '%s' "\${P${m#M}}")
  d=$(diff <(printf '%s\n' "${!m}" | grep -E '^(cmd|blob|pin|tree|bounds|wall|claims|exit|launcher|driver)') <(cd "$SB" && bash "$f" --predict-only | grep -E '^(cmd|blob|pin|tree|bounds|wall|claims|exit|launcher|driver)'))
  [ -z "$d" ] && ok || bad "$m is not reproducible across cwds: $(printf '%s' "$d" | head -3)"
done
printf '%s\n' "$M4" | grep -A1 '^cmd CMD_LEG_R4A' | tail -1 | grep -qF -- "--functions \"$C4A\"" && ok || bad "the printed R4/a command lost the frozen CSV"
printf '%s\n' "$M4" | grep -A1 '^cmd CMD_LEG_R4A' | tail -1 | grep -qF 'claim_batch --source-root' && bad "the printed R4/a command carries the subcommand token again" || ok
printf '%s\n' "$M5" | grep -A1 '^cmd CMD_LEG_R5' | tail -1 | grep -qF -- "--functions \"$C5\"" && ok || bad "the printed R5 command lost the frozen CSV"
printf '%s\n' "$M6" | grep -A1 '^cmd CMD_LEG_R6' | tail -1 | grep -qF -- "--entry \"$E6\" --functions \"$C6\"" && ok || bad "the printed R6 command lost its entry and CSV"
check "predict-only writes nothing anywhere under the run root" 0 bash -c "cd '$SB' && before=\$(find '$SB' -type f | wc -l) && bash '$P4' --predict-only >/dev/null && bash '$P5' --predict-only >/dev/null && bash '$P6' --predict-only >/dev/null && after=\$(find '$SB' -type f | wc -l) && [ \"\$before\" = \"\$after\" ]"

echo "== end-to-end on THIS host: every gate must refuse before any clone, fetch or build"
run_probe() { # run_probe <packet> <extra args…> — exercises the main flow until a gate refuses
  local f=$1; shift
  local jb=$SB/probe-$RANDOM-$$.job out
  mkdir -p "$jb"
  out=$("$f" "$jb/out" "$@" 2>&1); rc=$?
  printf '%s\t%s\t%s\n' "$rc" "$out" "$jb"
}
# /usr/bin/time is absent in this container (it is booked to exist on srv1), so the earliest local refusal is
# that tooling gate; everything below it is exercised with a copy whose tooling check is relaxed.
check_out "the shipped R4 packet stops at a named missing-tool gate before creating OUT" 2 "/usr/bin/time" bash "$P4" "$SB/$(grep -m1 '^J=' "$P4" | cut -d= -f2)/out"
check_out "…the R5 packet likewise" 2 "/usr/bin/time" bash "$P5" "$SB/$(grep -m1 '^J=' "$P5" | cut -d= -f2)/out"
check_out "…and the held R6 packet stops there too, even with a grant reference in hand" 2 "/usr/bin/time" bash "$P6" "$SB/$(grep -m1 '^J=' "$P6" | cut -d= -f2)/out" keen-otter-msg_3407a188-cleared
check "…and none of the three created an evidence dir on the way to that refusal" 0 bash -c "[ ! -e '$SB/tooljob4/out' ] && [ ! -e '$SB/tooljob5/out' ] && [ ! -e '$SB/tooljob6/out' ]"
cp "$P4" "$SB/relaxed4.sh"; cp "$P5" "$SB/relaxed5.sh"; cp "$P6" "$SB/relaxed6.sh"
python3 - "$SB" <<'PY'
import sys,re
sb=sys.argv[1]
for f in (sb+"/relaxed4.sh", sb+"/relaxed5.sh", sb+"/relaxed6.sh"):
    s=open(f).read()
    s2=s.replace('[ -x /usr/bin/time ]','[ -x /bin/true ]')
    s2=s2.replace('/usr/bin/time -v bash -c','bash -c')
    assert s2!=s, f
    open(f,'w').write(s2)
PY
for pair in "$SB/relaxed4.sh:" "$SB/relaxed5.sh:" "$SB/relaxed6.sh:grantish"; do
  f=${pair%%:*}; extra=${pair##*:}
  # the packet's own argument gate refuses an OUT whose <JOB> component is not the booked job, so a probe
  # that wanted to reach the scope gate had to be booked under the real job name; the probe dir now IS it.
  jb=$SB/$(grep -m1 '^J=' "$f" | cut -d= -f2); mkdir -p "$jb"
  if [ -n "$extra" ]; then out=$(bash "$f" "$jb/out" "keen-otter-msg_3407a188-cleared" 2>&1); else out=$(bash "$f" "$jb/out" 2>&1); fi
  rc=$?
  if [ "$rc" = 2 ] && printf '%s' "$out" | grep -q "scope gate"; then ok; else bad "$f: expected a scope-gate refusal (exit 2), got rc=$rc: $(printf '%s' "$out" | head -2 | tr '\n' '|')"; fi
  if [ -d "$jb/out" ]; then
    [ -s "$jb/out/receipt.txt" ] && ok || bad "$f: the run dir exists but has no receipt"
    [ "$(find "$jb/out" -maxdepth 1 -name 'wt-*' | wc -l)" = 0 ] && ok || bad "$f: a clone appeared before the scope gate refused"
    snaps=$(find "$jb/out/memevents" -maxdepth 1 -type f 2>/dev/null | wc -l)
    [ "$snaps" = 0 ] && ok || bad "$f: the scope gate refused before any counter was read, yet $snaps snapshots were published"
    grep -q 'VERDICT=NO_OBSERVATION reason=scope' "$jb/out/receipt.txt" && ok || bad "$f: the refusal is not recorded as NO_OBSERVATION with the scope reason"
    grep -q 'script_sha256=' "$jb/out/receipt.txt" && ok || bad "$f: the staged script hash was not recorded"
    grep -q 'fixtures:' "$jb/out/receipt.txt" && ok || bad "$f: the embedded fixture result was not recorded"
    [ -s "$jb/out/logs/selfcheck.log" ] && ok || bad "$f: no selfcheck.log"
    second=$(bash "$f" "$jb/out" ${extra:+keen-otter-msg_3407a188-cleared} 2>&1); rc2=$?
    [ "$rc2" = 2 ] && printf '%s' "$second" | grep -q 'already exists' && ok || bad "$f: reusing the evidence dir was not refused with exit 2 (rc=$rc2)"
    [ "$(find "$jb/out/memevents" -maxdepth 1 -type f | wc -l)" = 0 ] && ok || bad "$f: the reused-dir refusal still wrote snapshots"
  else
    bad "$f: no run dir was created at all; the flow stopped before the evidence gate"
  fi
done

echo "== item 5: the packets say plainly what these rows do NOT show"
for f in "$P4" "$P5"; do
  grep_in "$(basename "$f") header states the row discriminates nothing" "$f" "DISCRIMINATES NOTHING"
  grep_in "$(basename "$f") header says presence observations" "$f" "PRESENCE observations"
  grep_in "$(basename "$f") header says no mutation leg is run or proposed" "$f" "NO mutation leg is run or proposed"
done
grep_in "the R6 header states the hold and the gate" "$P6" "msg_3407a188"
grep_in "the R6 header states a refusal of the subject is a finding" "$P6" "fails"
for f in $PKTS; do grep_in "$(basename "$f") records the row exit as an observation, not a tool status" "$f" "VERDICT="; done

echo "== WHICH STREAM carries the completion record (v5): the gate reads the leg stderr capture"
# the single eprintln!("{}", partition.render(&timings)) emission in claim_batch.rs's run() path writes
# the record to stderr; PASS/FAIL go to stdout. leg() sends
# stdout to <leg>.out and stderr to logs/<leg>.err. A gate on the wrong stream reads 0 records and
# NO_OBSERVATIONs a full-green row, so both the wiring and the packets' own end-to-end controls are checked.
for f in $PKTS; do
  b=$(basename "$f")
  grep_in "$b runs the completion gate on the leg stderr capture" "$f" 'pc=$(pp_check "$OUT/logs/$L.err")'
  grep_not "$b never runs the completion gate on the leg stdout" "$f" 'pp_check "$OUT/$L.out"'
  grep_in "$b keeps the verdict classifier on the leg stdout" "$f" 'claim_classify "$OUT/$L.out"'
  grep_in "$b counts stray verdict lines on the leg stdout" "$f" 'stray_verdict_lines "$OUT/$L.out"'
  grep_in "$b states the stream in its header" "$f" 'WHICH STREAM'
  grep_in "$b names the stream in --predict-only output" "$f" 'completion_record_stream=stderr(leg capture logs/<leg>.err)'
  grep_in "$b drives a REAL leg with a stub in place of the binary (fixture mode)" "$f" 'leg E2E_STDERR '
  grep_in "$b: record on stderr plus verdict on stdout must be an observation" "$f" 'ck e2e-record-on-stderr-is-an-observation AS_EXPECTED'
  grep_in "$b: that record must reach the receipt" "$f" 'ck e2e-record-on-stderr-counted-in-the-receipt 1'
  grep_in "$b: a record only on stdout must NOT be an observation" "$f" 'ck e2e-record-only-on-stdout-is-not-an-observation NOOBS'
  grep_in "$b: two records must NOT be an observation" "$f" 'ck e2e-two-records-is-not-an-observation NOOBS'
  grep_in "$b gates the pinned render site in the checkout" "$f" 'm=$(producer_gate "$W/$PROD")'
  grep_in "$b blob-gates claim_batch.rs like the parser flag set" "$f" 'blob_gate "$W" "$PROD" "$PRODBLOB"'
  want=1  # v6: one pin checkout per job, so one render-site gate and one claim_batch.rs blob gate each
  np=$(grep -c 'm=$(producer_gate "$W/$PROD")' "$f"); nb=$(grep -c 'm=$(blob_gate "$W" "$PROD" "$PRODBLOB")' "$f")
  [ "$np" = "$want" ] && ok || bad "$b gates the render site $np times, expected $want (one per pin checkout)"
  [ "$nb" = "$want" ] && ok || bad "$b blob-gates claim_batch.rs $nb times, expected $want"
  grep_in "$b refuses an empty completion predicate instead of counting every line" "$f" 'the completion predicates are empty; a vacuous pattern would count every line'
  grep_in "$b refuses an unset completion-record fixture" "$f" 'completion-real-bytes 259'
done
check_out "the shipped gate refuses an empty prefix rather than matching every line" 0 refused bash -c "source '$H'; PP_PREFIX= ; pp_check '$SB/pp_ok.txt' | grep -q vacuous && echo refused"
check_out "…and the guard is in the gate itself, not only in the fixtures" 0 refused bash -c "source '$H'; PP_SHAPE= ; pp_check '$SB/pp_ok.txt' | grep -q vacuous && echo refused"
check_out "the shipped producer_gate refuses a stdout render site" 0 refused bash -c "source '$H'; printf '    println!(\"{}\", partition.render(&timings));\n' > '$SB/pb_out.rs'; producer_gate '$SB/pb_out.rs' | grep -q 'NOT emitted to stderr' && echo refused"
check_out "…and accepts the stderr render site" 0 OK bash -c "source '$H'; printf '    eprintln!(\"{}\", partition.render(&timings));\n' > '$SB/pb_err.rs'; producer_gate '$SB/pb_err.rs'"
if [ -n "$REPO" ] && g -C "$REPO" rev-parse --git-dir >/dev/null 2>&1; then
  for pin in "$H4" "$H5"; do
    check_out "the pinned producer renders the record to stderr at ${pin:0:8}" 0 '1460' bash -c "git -C '$REPO' grep -n -e 'partition.render(&timings)' '$pin' -- '$RBIN' < /dev/null"
  done
else
  skip "no repo with the pinned commits: the pinned render-site grep was not run"
fi

# ---------------------------------------------------------------- negative controls: mutate a copy, expect failure
echo "== the completion record (v4 blocker 2) and the verdict rules (v4 blocker 3), on the shipped helpers"
PPREAL_B64='W3Byb2Nlc3MtcGFydGl0aW9uXSB0b3RhbF9tcz0zNDAxODggcHJlX3J1bl9tcz0wIGFyZ3NfbXM9MjIgaW5kZXhfYnVpbGRfbXM9MTU0MzYyIGVkZ2VfaW5kZXhfd2FybV9tcz0zNTYxNyBvdXRwdXRfcG9saWN5X21zPTEyNTA5IHJvc3Rlcl9tcz0wIHB1Ymxpc2hlZF9tb2NrX3ByZWNvbXB1dGVfbXM9MTc2MSBlbnRyaWVzX21zPTEzNTkxMSByZXBvcnRpbmdfbXM9MiBlbnRyaWVzX3Jlc29sdmVfbXM9MTEzOTU1IGVudHJpZXNfd2l0bmVzc19tcz0yMDYyMg=='
PPREAL=$(printf '%s' "$PPREAL_B64" | base64 -d)
[ "$(printf '%s' "$PPREAL" | wc -c | tr -d ' ')" = 259 ] && ok || bad "this self-test's copy of the real completion record is not 259 bytes"
[ "$(printf '%s' "$PPREAL" | sha256sum | cut -d' ' -f1)" = 7ac24cf28973511b7902ce3ba621e8c46b060ad7855ea895b1f5a157bf1835b2 ] && ok || bad "…and not the pinned digest 7ac24cf2…"
for f in $PKTS; do
  b=$(basename "$f")
  same=$(bash -c "source <(sed -n '/^  PP_REAL=/,/^  PP_REAL_SHA=/p' '$f' | sed 's/^  //'); printf '%s' \"\$PP_REAL\" | sha256sum | cut -d' ' -f1")
  [ "$same" = 7ac24cf28973511b7902ce3ba621e8c46b060ad7855ea895b1f5a157bf1835b2 ] && ok || bad "$b freezes a completion-record fixture whose bytes are not the pinned 259-byte record"
  grep_in "$b states the two-stage gate (prefix count, then shape)" "$f" "count lines with the PREFIX"
  grep_in "$b warns that whole-line anchoring spent an earlier attempt" "$f" "#15695"
  grep_in "$b excludes the [process-partition-exit] neighbour" "$f" "hence the "
done
printf '%s\n' "$PPREAL" > "$SB/pp_ok.txt"
printf '%s\n%s\n' "$PPREAL" "$PPREAL" > "$SB/pp_dup.txt"
printf '%s\n[process-partition] total_ms=zz entries_resolve_ms=1\n' "$PPREAL" > "$SB/pp_valid_bad.txt"
printf '[process-partition] total_ms=zz entries_resolve_ms=1\n' > "$SB/pp_bad.txt"
printf 'PASS w_x\n' > "$SB/pp_none.txt"
printf '%s\n[process-partition-exit] run_returned_ms=340188 main_end_ms=340188\n' "$PPREAL" > "$SB/pp_neighbour.txt"
printf '[process-partition-exit] run_returned_ms=1 main_end_ms=1\n' > "$SB/pp_neighbour_only.txt"
check_out "the shipped gate accepts the real 259-byte record" 0 "accepted" bash -c "source '$H'; pp_check '$SB/pp_ok.txt' > /dev/null && echo accepted"
check_out "the shipped gate refuses a duplicate record" 0 "refused" bash -c "source '$H'; pp_check '$SB/pp_dup.txt' | grep -q 'exactly 1 is required' && echo refused"
check_out "the shipped gate refuses one valid plus one malformed" 0 "refused" bash -c "source '$H'; pp_check '$SB/pp_valid_bad.txt' | grep -q 'exactly 1 is required' && echo refused"
check_out "the shipped gate refuses a malformed record" 0 "refused" bash -c "source '$H'; pp_check '$SB/pp_bad.txt' | grep -q malformed && echo refused"
check_out "the shipped gate refuses a missing record" 0 "refused" bash -c "source '$H'; pp_check '$SB/pp_none.txt' | grep -q '0 matched the prefix' && echo refused"
check_out "the [process-partition-exit] neighbour does not count as a record" 0 "refused" bash -c "source '$H'; pp_check '$SB/pp_neighbour_only.txt' | grep -q '0 matched the prefix' && echo refused"
check_out "the neighbour line beside a real record does not cause a false refusal" 0 "accepted" bash -c "source '$H'; pp_check '$SB/pp_neighbour.txt' > /dev/null && echo accepted"
check_out "whole-line anchoring would have rejected the real record (the #15695 mistake)" 0 "0" bash -c "printf '%s\\n' \"$PPREAL\" | grep -cE '^[[]process-partition[]] total_ms=[0-9]+\$' || true"
printf 'FAIL w_x (returned ProcessExit::ExitFailure, code 1: boom)\n' > "$SB/v_q.txt"
printf 'PASS w_x\nPASS w_x\n' > "$SB/v_two.txt"
printf 'PASS w_x\nFAIL w_x\n' > "$SB/v_both.txt"
printf 'UNOBSERVED w_x (no receipt)\n' > "$SB/v_unobs.txt"
printf 'PANICKED w_x (panic)\n' > "$SB/v_panic.txt"
printf 'no verdict here\n' > "$SB/v_none.txt"
printf 'PASS w_xy\n' > "$SB/v_prefix.txt"
check_out "a qualified FAIL is NO_OBSERVATION, never a falsification" 0 "NO_OBSERVATION" bash -c "source '$H'; claim_classify '$SB/v_q.txt' OK w_x"
check_out "two outcome lines for one name is NO_OBSERVATION" 0 "NO_OBSERVATION" bash -c "source '$H'; claim_classify '$SB/v_two.txt' OK w_x"
check_out "contradictory outcome lines are NO_OBSERVATION" 0 "NO_OBSERVATION" bash -c "source '$H'; claim_classify '$SB/v_both.txt' OK w_x"
printf 'PASS w_x\nFAIL w_x (runtime error: boom)\n' > "$SB/v_bare_q.txt"
check_out "a bare PASS beside a qualified FAIL is NO_OBSERVATION (only the exactness rule sees this)" 0 "NO_OBSERVATION" bash -c "source '$H'; claim_classify '$SB/v_bare_q.txt' OK w_x"
check_out "UNOBSERVED is NO_OBSERVATION" 0 "NO_OBSERVATION" bash -c "source '$H'; claim_classify '$SB/v_unobs.txt' OK w_x"
check_out "PANICKED is NO_OBSERVATION" 0 "NO_OBSERVATION" bash -c "source '$H'; claim_classify '$SB/v_panic.txt' OK w_x"
check_out "no outcome line for the name is NO_OBSERVATION" 0 "NO_OBSERVATION" bash -c "source '$H'; claim_classify '$SB/v_none.txt' OK w_x"
check_out "a longer name's line is not counted for a shorter name" 0 "NO_OBSERVATION" bash -c "source '$H'; claim_classify '$SB/v_prefix.txt' OK w_x"
check_out "a bare PASS with rc 0 still holds" 0 "HELD" bash -c "source '$H'; printf 'PASS w_x\\n' > '$SB/v_held.txt'; claim_classify '$SB/v_held.txt' OK w_x"
check_out "a bare FAIL with rc 1 is still a falsification" 0 "FAILED" bash -c "source '$H'; printf 'FAIL w_x\\n' > '$SB/v_fals.txt'; claim_classify '$SB/v_fals.txt' OK w_x"

check_out "the shipped leg_rc_expect wants 0 when every claim passed" 0 "OK" bash -c "source '$H'; printf 'PASS w_x\\n' > '$SB/v_held.txt'; leg_rc_expect '$SB/v_held.txt' 0 w_x"
check_out "the shipped leg_rc_expect wants 1 when a claim failed" 0 "OK" bash -c "source '$H'; printf 'FAIL w_x\\n' > '$SB/v_fals.txt'; leg_rc_expect '$SB/v_fals.txt' 1 w_x"
check_out "the shipped leg_rc_expect refuses an inconsistent pair" 0 "MISMATCH" bash -c "source '$H'; printf 'PASS w_x\\n' > '$SB/v_held.txt'; leg_rc_expect '$SB/v_held.txt' 1 w_x | grep -o MISMATCH"
check_out "a clean claim in a leg whose rc disagreed is NO_OBSERVATION" 0 "NO_OBSERVATION" bash -c "source '$H'; printf 'PASS w_x\\n' > '$SB/v_held.txt'; claim_classify '$SB/v_held.txt' \"\$(leg_rc_expect '$SB/v_held.txt' 1 w_x)\" w_x"
check_out "the same claim in a consistent leg is HELD, so the pair above is not vacuous" 0 "HELD" bash -c "source '$H'; printf 'PASS w_x\\n' > '$SB/v_held.txt'; claim_classify '$SB/v_held.txt' \"\$(leg_rc_expect '$SB/v_held.txt' 0 w_x)\" w_x"


# ---------------------------------------------------------------- v7: the per-claim decision, gate exit codes,
# ---------------------------------------------------------------- the post-leg gates, the argument gate (manager msg_ac20c593)
echo "== v7: the refusal-position audit; no foreign state variables; the exit decision per claim"
for f in "$P4" "$P5" "$P6"; do
  b=$(basename "$f")
  # A gate called as X=$(fn …) || nox … decides by exit code. In v6 wall_gate and producer_gate printed a
  # refusal and returned 0, so the refusal existed only as a string in a variable. Any bare `return` arm
  # in such a function inherits the status of the last command, which after an echo is 0.
  fns=$(grep -vE '^[[:space:]]*#' "$f" | grep -E '\|\| (nox|\{)' | grep -oE '\$\([a-z_]+' | sed 's/$(//' | sort -u | while read -r c; do grep -q "^$c() {" "$f" && printf '%s\n' "$c"; done)
  nf=$(printf '%s ' $fns | wc -w)
  if [ "$nf" -ge 6 ]; then ok; else bad "$b: only $nf gates found in refusal position: $fns"; fi
  for fn in $fns; do
    body=$(awk -v fn="$fn" 'index($0, fn"() {")==1 {p=1} p && /^[^[:space:]#]/ && index($0, fn"() {")!=1 {exit} p {print}' "$f")
    if [ -z "$body" ]; then bad "$b: $fn is used in refusal position but has no definition at column 0"; continue; fi
    bare=$(printf '%s\n' "$body" | grep -cE '; return(;|\}|$)|[^-a-z_]return;')
    r1=$(printf '%s\n' "$body" | grep -cE 'return 1([^0-9]|$)')
    if [ "$bare" = 0 ] && [ "$r1" -ge 1 ]; then ok; else bad "$b: gate $fn (X || nox) has bare-return arms=$bare non-zero arms=$r1"; fi
  done
  printf '%s\n' $fns | grep -qx wall_gate || bad "$b: wall_gate is not in the refusal-position set"
  printf '%s\n' $fns | grep -qx producer_gate || bad "$b: producer_gate is not in the refusal-position set"
  printf '%s\n' $fns | grep -qx out_gate || bad "$b: the new argument gate is not in the refusal-position set"
  # v7 blocker 1: the verdict named another row's state variables. Prose may mention them; code may not.
  hits=$(grep -vE '^[[:space:]]*#' "$f" | grep -cE 'S4A|S4B|\bS5\b|\bS6\b')
  if [ "$hits" = 0 ]; then ok; else bad "$b: $hits non-comment lines still name a row state variable"; fi
  # the wiring, by name: post-leg gates before the decision, the pre-leg hash, the argument gate, the
  # bounded self-run, and CARGO_HOME under the evidence dir.
  for needle in 'post_leg_gate "$WT" "$RUNNER" "$PREBIN"' 'final_verdict' 'out_gate "$OUT" "$J"' 'timeout "$T_SELFRUN" bash "$0" --classifier-fixtures' 'T_SELFRUN=' 'export CARGO_HOME=$OUT/cargo-home' 'gate-read-$GV' 'leg_rc_expect' 'claims classified:'; do
    grep -qF "$needle" "$f" && ok || bad "$b: missing wiring: $needle"
  done
  grep -qE 'PREBIN[456]=\$\(bin_sha' "$f" && ok || bad "$b: the runner is not hashed before the legs (PREBINn=\$(bin_sha …))"
  grep -qE 'WT=\$OUT/wt-r[45]|WT=\$W; RUNNER=' "$f" && ok || bad "$b: post_leg_gate would point at no worktree (WT is never set)"
  grep -qE 'RUNNER=\$BIN[456]' "$f" && ok || bad "$b: post_leg_gate would point at no runner binary (RUNNER is never set)" 
  # order matters: the post-leg gate must run after the legs and before the exit decision
  ln_legs=$(grep -n 'leg R' "$f" | tail -1 | cut -d: -f1)
  ln_post=$(grep -n 'post_leg_gate "$WT"' "$f" | cut -d: -f1)
  ln_final=$(grep -n '^final_verdict$' "$f" | cut -d: -f1)
  if [ -n "$ln_legs" ] && [ -n "$ln_post" ] && [ -n "$ln_final" ] && [ "$ln_legs" -lt "$ln_post" ] && [ "$ln_post" -lt "$ln_final" ]; then
    ok
  else
    bad "$b: the post-leg gate must run after the last leg and before final_verdict (legs=$ln_legs post=$ln_post final=$ln_final)"
  fi
done
for hpair in "$SB/h4.sh:$P4" "$SB/h5.sh:$P5" "$SB/h6.sh:$P6"; do
  h=${hpair%%:*}; p=${hpair##*:}; b=$(basename "$p")
  # the packet's own exit decision, driven end to end: 0 held, 1 falsified, 2 no-observation, and the
  # precedence rule (falsified outranks no-observation; a post-leg cause outranks both).
  # v8: final_verdict decides against the REGISTERED claim set, so a probe that drives it has to
  # register one - otherwise the probe measures the refusal instead of the decision.
  fvrc() { # $1=dir $2=classes as a printf format $3=post-leg cause ('' = none) $4=claimed names (default a b)
    local d=$1 rc n4
    rm -rf "$d"; mkdir -p "$d/logs"
    printf '%b\n' "$2" > "$d/LEGX.classes"
    : > "$d/claims-registered.txt"; for n4 in ${4:-a b}; do printf 'LEGX %s\n' "$n4" >> "$d/claims-registered.txt"; done
    if [ -n "$3" ]; then printf '%s\n' "$3" > "$d/post-leg.cause"; else : > "$d/post-leg.cause"; fi
    rc=$(OUT="$d" O="$d/receipt.txt" bash -c "source '$h'; LEGS_BOOKED='LEGX'; CLAIMS_ALL='${4:-a b}'; CLAIMS_LEGX='${4:-a b}'; N_CLAIMS_BOOKED=$(printf '%s\n' ${4:-a b} | wc -l | tr -d ' '); final_verdict" >/dev/null 2>&1; echo $?)
    echo "$rc"; }
  check "$b: every claim held exits 0" 0 test "$(fvrc "$SB/fv_a_$b" 'a HELD\nb HELD' '')" = 0
  check "$b: one falsified claim exits 1" 0 test "$(fvrc "$SB/fv_b_$b" 'a HELD\nb FAILED' '')" = 1
  check "$b: one unobserved claim exits 2" 0 test "$(fvrc "$SB/fv_c_$b" 'a HELD\nb NO_OBSERVATION' '')" = 2
  check "$b: falsified outranks unobserved" 0 test "$(fvrc "$SB/fv_d_$b" 'a FAILED\nb NO_OBSERVATION\nc NO_OBSERVATION' '' 'a b c')" = 1
  check "$b: a post-leg cause voids even a bare FAIL" 0 test "$(fvrc "$SB/fv_e_$b" 'a HELD\nb FAILED' 'the runner changed during the run')" = 2
  check "$b: a post-leg cause with everything held is still 2" 0 test "$(fvrc "$SB/fv_f_$b" 'a HELD\nb HELD' 'the worktree is not clean')" = 2
  check_out "$b: the verdict names the claims it decided" 0 'claims classified: a=HELD b=FAILED' bash -c "rm -rf '$SB/fv_g_$b'; mkdir -p '$SB/fv_g_$b/logs'; printf 'a HELD\nb FAILED\n' > '$SB/fv_g_$b/LEGX.classes'; printf 'LEGX a\nLEGX b\n' > '$SB/fv_g_$b/claims-registered.txt'; : > '$SB/fv_g_$b/post-leg.cause'; ( source '$h'; LEGS_BOOKED='LEGX'; CLAIMS_ALL='a b'; CLAIMS_LEGX='a b'; N_CLAIMS_BOOKED=2; OUT='$SB/fv_g_$b' O='$SB/fv_g_$b/receipt.txt'; final_verdict ) >/dev/null 2>&1; grep -o 'claims classified:.*' '$SB/fv_g_$b/receipt.txt'"
  # the argument gate, on the shipped helpers
  check "$b: out_gate accepts the booked shape" 0 bash -c "source '$h'; out_gate /home/x/runs/J/out J"
  check "$b: out_gate refuses a relative OUT" 1 bash -c "source '$h'; out_gate runs/J/out J"
  check "$b: out_gate refuses another job's directory" 1 bash -c "source '$h'; out_gate /home/x/runs/other/out J"
  check "$b: out_gate refuses a bare /out" 1 bash -c "source '$h'; out_gate /out J"
  check "$b: out_gate refuses a path below /out" 1 bash -c "source '$h'; out_gate /home/x/runs/J/out/deeper J"
  # the post-leg gate, on the shipped helpers: clean passes, a dirty worktree or a changed runner refuses
  plg_case() { # $1=dir $2=clean|dirtywt|binchanged -> the cause text
    local d=$1 wt=$1/wt pre
    rm -rf "$d"; mkdir -p "$d/logs" "$wt"
    git -c init.defaultBranch=main -c user.email=fixture@selfcheck -c user.name=selfcheck init -q "$wt" </dev/null >/dev/null 2>&1
    printf 'x\n' > "$wt/file"; git -C "$wt" add file </dev/null >/dev/null 2>&1
    git -C "$wt" commit -q -m seed </dev/null >/dev/null 2>&1
    : > "$d/bin"; pre=$(sha256sum "$d/bin" | cut -d' ' -f1)
    case "$2" in
      dirtywt) printf 'left behind\n' > "$wt/untracked" ;;
      binchanged) printf 'the runner changed\n' > "$d/bin" ;;
    esac
    OUT="$d" O="$d/receipt.txt" bash -c "source '$h'; post_leg_gate '$wt' '$d/bin' '$pre'" >/dev/null 2>&1
    if [ -s "$d/post-leg.cause" ]; then cut -c1-24 "$d/post-leg.cause"; else echo NO-CAUSE; fi; }
  check_out "$b: a clean worktree and an unchanged runner leave no cause" 0 "NO-CAUSE" plg_case "$SB/plg_clean_$b" clean
  check_out "$b: a dirty worktree names the cause" 0 "the worktree" plg_case "$SB/plg_dirty_$b" dirtywt
  check_out "$b: a changed runner names the cause" 0 "the runner changed" plg_case "$SB/plg_bin_$b" binchanged
done
echo "== negative controls (a mutated packet must FAIL, or the checks are decorative)"
cat > "$SB/mut15.py" <<'PY15'
import sys,re
p=sys.argv[1]; s=open(p).read()
s2 = re.sub(r"^RUNTIME_S=[0-9]+;", "RUNTIME_S=9900;", s, count=1, flags=re.M)
assert s2 != s, "no RUNTIME_S assignment to inflate"
assert "RUNTIME_S=9900" in s2
open(p,"w").write(s2)
PY15

cat > "$SB/mut13.py" <<'PY13'
import sys
p=sys.argv[1]; s=open(p).read()
old = 'm=$(producer_gate "$W/$PROD") || nox "producer stream gate: $m" producer'
assert old in s
open(p,"w").write(s.replace(old,"",1))
PY13

cat > "$SB/mut14.py" <<'PY14'
import sys
p=sys.argv[1]; s=open(p).read()
a = s.index("  # ---- THE COMPLETION RECORD REACHES THE GATE")
b = s.index("  ck completion-predicates-empty-refuses", a)
b = s.index(chr(10), b)
open(p,"w").write(s[:a] + "  # (control removed by the mutation)\n" + s[b:])
PY14


cat > "$SB/mut16.py" <<'PY16'
# weaken the leg exit-code agreement: never count a bare FAIL, so an rc of 1 with a FAIL line (or 0 with
# none) looks consistent. Every contract fixture and the classifier controls must then refuse.
import sys, re
p=sys.argv[1]; s=open(p).read()
old = 'for n in $names; do q=$(grep -c "^FAIL $n\\$" "$f" 2>/dev/null); [ "${q:-0}" = 1 ] && fails=$((fails+1)); done'
assert old in s
open(p,"w").write(s.replace(old, 'for n in $names; do q=0; done', 1))
PY16

cat > "$SB/mut17.py" <<'PY17'
# v7 blocker 2, the exact v6 defect: wall_gate prints its refusal and returns 0.
import sys, re
p=sys.argv[1]; s=open(p).read()
m = re.search(r'(?m)^wall_gate\(\) \{.*?\n\n', s, re.S)
assert m, 'wall_gate'
b = m.group(0).replace('; return 1; }', '; return; }')
assert b != m.group(0), 'no wall_gate refusal arm to weaken'
open(p,"w").write(s[:m.start()] + b + s[m.end():])
PY17

cat > "$SB/mut18.py" <<'PY18'
# the same defect in the producer gate: a refusal that returns 0 is a gate that never closes.
import sys, re
p=sys.argv[1]; s=open(p).read()
m = re.search(r'(?m)^producer_gate\(\) \{.*?\n\n', s, re.S)
assert m, 'producer_gate'
b = m.group(0).replace('; return 1; }', '; return; }').replace('; return 1 ;;', '; return ;;')
assert b != m.group(0), 'no producer_gate refusal arm to weaken'
open(p,"w").write(s[:m.start()] + b + s[m.end():])
PY18

cat > "$SB/mut19.py" <<'PY19'
# final_verdict ignores the post-leg cause: a dirty worktree or a swapped runner would still report the
# legs' verdicts as if nothing had changed under them.
import sys
p=sys.argv[1]; s=open(p).read()
old = '[ -s "$OUT/post-leg.cause" ] && pc=$(cat "$OUT/post-leg.cause")'
assert old in s
open(p,"w").write(s.replace(old, ':', 1))
PY19

cat > "$SB/mut20.py" <<'PY20'
# leg() stops asking whether the process exit agrees with the leg's bare verdicts: rc 0 with a FAIL, or
# rc 1 with none, would then classify as if it were an observation.
import sys
p=sys.argv[1]; s=open(p).read()
old = 'rcst=$(leg_rc_expect "$OUT/$L.out" "$rc" "$3")'
assert old in s
open(p,"w").write(s.replace(old, 'rcst=OK', 1))
PY20

cat > "$SB/mut21.py" <<'PY21'
# the exit precedence inverted: a packet that has a real falsification AND an unobserved leg would report
# NO_OBSERVATION (exit 2) and the finding would be lost.
import sys
p=sys.argv[1]; s=open(p).read()
a = 'if [ "$anyF" = 1 ]; then'
b = 'if [ "$anyN" = 1 ]; then'
assert s.count(a) == 1 and s.count(b) == 1, (s.count(a), s.count(b))
s = s.replace(a, '@@B@@', 1).replace(b, a, 1).replace('@@B@@', b, 1)
open(p,"w").write(s)
PY21

cat > "$SB/mut22.py" <<'PY22'
# the argument gate stops refusing a relative OUT: the evidence dir would resolve against whatever cwd
# the launcher happened to start in, and the row's receipts would land outside the booked run.
import sys, re
p=sys.argv[1]; s=open(p).read()
m = re.search(r'(?m)^out_gate\(\) \{.*?\n\n', s, re.S)
assert m, 'out_gate'
b = m.group(0).replace('; return 1;; esac', ';; esac', 1)
assert b != m.group(0), 'no relative-OUT arm to weaken'
open(p,"w").write(s[:m.start()] + b + s[m.end():])
PY22

cat > "$SB/mut23.py" <<'PY23'
# the post-leg worktree check disabled: a leg that wrote into the pinned checkout would go unnoticed.
import sys
p=sys.argv[1]; s=open(p).read()
old = 'if [ "${dirty:-1}" != 0 ]; then'
assert old in s
open(p,"w").write(s.replace(old, 'if [ "${dirty:-1}" = 99999 ]; then', 1))
PY23

cat > "$SB/mut24.py" <<'PY24'
# the post-leg runner-identity check disabled: a binary that changed between the pre-leg hash and the
# legs would still have its verdicts attributed to the pinned source.
import sys
p=sys.argv[1]; s=open(p).read()
old = 'if [ "$post" != "$pre" ] || [ "$pre" = unreadable ]; then'
assert old in s
open(p,"w").write(s.replace(old, 'if [ -z "$post" ]; then', 1))
PY24

negctl() { # negctl <label> <base packet> <mutation script>: the mutated copy must fail its OWN fixtures
  local label=$1 base=$2 mut=$3
  cp "$base" "$SB/neg.sh"
  if ! python3 "$mut" "$SB/neg.sh"; then bad "$label: the mutation did not apply"; return; fi
  local out rc; out=$(bash "$SB/neg.sh" --classifier-fixtures 2>&1); rc=$?
  if [ "$rc" = 0 ] && printf '%s\n' "$out" | tail -1 | grep -q 'fail=0 \(of '; then
    bad "$label: the mutated packet still passes its own fixtures"
  elif printf '%s\n' "$out" | grep -qE '^FIXTURE FAIL'; then
    echo "  $label -> $(printf '%s\n' "$out" | grep -m1 '^FIXTURE FAIL') | $(printf '%s\n' "$out" | tail -1) (rc=$rc)"; ok
  else
    bad "$label: the mutated packet did not fail on its OWN fixtures (rc=$rc) — the control would be decorative"
  fi; }
cat > "$SB/mut1.py" <<'PY1'
import sys
p=sys.argv[1]; s=open(p).read()
old = "CMD_LEG_R4A='\"$BIN4\" --source-root"
new = "CMD_LEG_R4A='\"$BIN4\" claim_batch --source-root"
assert old in s
open(p,"w").write(s.replace(old,new,1))
PY1
cat > "$SB/mut2.py" <<'PY2'
import sys
p=sys.argv[1]; s=open(p).read()
old = 'N4B="witness_r4_refuses_unit_graph_cycle_holds"'
new = 'N4B="witness_r4_refuses_unit_graph_cycle_holds witness_injected with-space"'
assert old in s
open(p,"w").write(s.replace(old,new,1))
PY2
cat > "$SB/mut3.py" <<'PY3'
import sys
p=sys.argv[1]; s=open(p).read()
old = "BM=$(bounds_check \"/sys/fs/cgroup$CG\") || nox \"booked bounds: $BM\" bounds"
assert old in s
open(p,"w").write(s.replace(old,'BM=""',1))
PY3
cat > "$SB/mut4.py" <<'PY4'
import sys
p=sys.argv[1]; s=open(p).read()
old = "if ( set -C; printf"
assert old in s
open(p,"w").write(s.replace(old,"if ( ",1))
PY4
cat > "$SB/mut5.py" <<'PY5'
import sys
p=sys.argv[1]; s=open(p).read()
old = 'echo "NO_OBSERVATION: $1 (reason=$2)" >&2; exit 2; }'
assert old in s
open(p,"w").write(s.replace(old,'echo "gate refused" >&2; exit 3; }',1))
PY5
cat > "$SB/mut6.py" <<'PY6'
import sys
p=sys.argv[1]; s=open(p).read()
old = "k1=$(oom_snap \"$EV\" \"$L-leg-after\"); LO=$(oom_pair_verdict \"$k0\" \"$k1\"); LASTOOM=$k1"
assert old in s
open(p,"w").write(s.replace(old,'k1="$k0"; LO=none; LASTOOM=$k1',1))
PY6
cat > "$SB/mut7.py" <<'PY7'
import sys
p=sys.argv[1]; s=open(p).read()
old = "CMD_LEG_R6='\"$BIN6\" --source-root"
new = "CMD_LEG_R6='\"$BIN6\" claim_batch --source-root"
assert old in s
open(p,"w").write(s.replace(old,new,1))
PY7
cat > "$SB/mut8.py" <<'PY8'
import sys
p=sys.argv[1]; s=open(p).read()
old = 'pc=$(pp_check "$OUT/logs/$L.err")'
assert old in s
open(p,"w").write(s.replace(old,'pc="OK skipped by the mutation"',1))
PY8
cat > "$SB/mut9.py" <<'PY9'
import sys
p=sys.argv[1]; s=open(p).read()
old = '[ "${any:-0}" = 1 ] || { echo NO_OBSERVATION; return; }'
assert old in s
open(p,"w").write(s.replace(old,':',1))
PY9
cat > "$SB/mut10.py" <<'PY10'
import sys
p=sys.argv[1]; s=open(p).read()
old = 'p=$(grep -c "^PASS $n\\$" "$f" 2>/dev/null); q=$(grep -c "^FAIL $n\\$" "$f" 2>/dev/null)'
assert old in s
open(p,"w").write(s.replace(old,'q=$(grep -c "^FAIL $n" "$f" 2>/dev/null)',1))
PY10
cat > "$SB/mut12.py" <<'PY12'
import sys
p=sys.argv[1]; s=open(p).read()
old = 'pc=$(pp_check "$OUT/logs/$L.err")'
assert old in s
open(p,"w").write(s.replace(old,'pc=$(pp_check "$OUT/$L.out")',1))
PY12

cat > "$SB/mut11.py" <<'PY11'
import sys
p=sys.argv[1]; s=open(p).read()
old = '[ "$n" = 1 ] || { echo "completion records:'
assert old in s
open(p,"w").write(s.replace(old,'[ "$n" -ge 1 ] || { echo "completion records:',1))
PY11
for mu in 3 4 5 6 8 9 10 11 12 13 16 17 18 19 20 21 22 23 24; do negctl "mutation $mu applied to the R4 job" "$P4" "$SB/mut$mu.py"; done
for mu in 3 4 5 6 8 9 10 11 12 13 16 17 18 19 20 21 22 23 24; do negctl "mutation $mu applied to the R5 job" "$P5" "$SB/mut$mu.py"; done
for mu in 3 4 5 6 8 9 10 11 12 13 16 17 18 19 20 21 22 23 24; do negctl "mutation $mu applied to the R6 job" "$P6" "$SB/mut$mu.py"; done
negctl "defect (1) reintroduced: a literal claim_batch token after the binary" "$P4" "$SB/mut1.py"
negctl "defect (2) reintroduced: a space-separated name in the CSV" "$P4" "$SB/mut2.py"
negctl "defect (1) in the R6 packet as well" "$P6" "$SB/mut7.py"
negctl "the end-to-end leg control deleted from the packet fixtures" "$P4" "$SB/mut14.py"
negctl "the end-to-end leg control deleted from the R5 job too" "$P5" "$SB/mut14.py"
# v6's own new rule: a wall booked above the installed launcher cap must be refused by the packet's own gate
negctl "RUNTIME_S booked at 9900, above the installed launcher cap (v6)" "$P4" "$SB/mut15.py"
negctl "…the same over-cap booking in the R5 job" "$P5" "$SB/mut15.py"
negctl "…and in the R6 job" "$P6" "$SB/mut15.py"
# mut3 and mut5 change the run path rather than a helper; the packet's self-structure section is what
# notices, so they are exercised as fixture controls too. The greps below are a second, independent signal.
cp "$P4" "$SB/neg3.sh"; python3 "$SB/mut3.py" "$SB/neg3.sh" && ok || bad "bounds-gate mutation did not apply"
grep -q 'BM=""' "$SB/neg3.sh" && ok || bad "the bounds gate is still present in the mutated copy"
grep -q 'BM=\$(bounds_check' "$SB/neg3.sh" && bad "the mutated copy still gates on bounds_check in its run path" || ok
cp "$P4" "$SB/neg5.sh"; python3 "$SB/mut5.py" "$SB/neg5.sh" && ok || bad "exit-code mutation did not apply"
n3=$(grep -vE '^[[:space:]]*#' "$SB/neg5.sh" | grep -cE '\bexit 3\b'); [ "$n3" = 1 ] && ok || bad "expected the mutated copy to reintroduce an exit 3, got $n3"
check "…which is the rule the self-test applies to the shipped packets (no exit 3 anywhere)" 0 bash -c "[ \$(grep -vE '^[[:space:]]*#' '$P4' | grep -cE '\bexit 3\b') = 0 ] && [ \$(grep -vE '^[[:space:]]*#' '$P6' | grep -cE '\bexit 3\b') = 0 ]"
check "…and exactly one executable exit 1 per shipped packet" 0 bash -c "[ \$(grep -vE '^[[:space:]]*#' '$P4' | grep -cE '\bexit 1\b') = 1 ] && [ \$(grep -vE '^[[:space:]]*#' '$P6' | grep -cE '\bexit 1\b') = 1 ]"


# ---------------------------------------------------------------- v8 (calm-eagle-670 msg_8ed9b50b): the
# ---------------------------------------------------------------- exit decision is anchored to a registered claim set
echo "== v8: the booked claim set, the registration, and the gate under final_verdict"
for f in $PKTS; do
  b=$(basename "$f")
  for fn in cs_check register_claims book_case_set restore_booked_set; do grep_in "$b defines $fn" "$f" "$fn() {"; done
  grep_in "$b declares LEGS_BOOKED" "$f" 'LEGS_BOOKED='
  grep_in "$b declares the booked claim list" "$f" 'CLAIMS_ALL='
  grep_in "$b counts the booked claims" "$f" 'N_CLAIMS_BOOKED=$(printf'
  grep_in "$b snapshots the booked set so a fixture can restore it" "$f" 'BOOK_LEGS="$LEGS_BOOKED"'
  grep_in "$b registers into the run directory, not somewhere implied" "$f" '$OUT/claims-registered.txt'
  grep_in "$b states what v8 changes" "$f" '# WHAT v8 CHANGES (calm-eagle-670 msg_8ed9b50b)'
  grep_in "$b says plainly what the fail-open was" "$f" 'printed VERDICT=AS_EXPECTED and exited 0'
done
grep_in "R4 books its two legs" "$P4" 'LEGS_BOOKED="R4a R4b"'
grep_in "R4's booked claims are the same names its leg commands name" "$P4" 'CLAIMS_R4a="$N4A"; CLAIMS_R4b="$N4B"'
grep_in "R5 books its one leg" "$P5" 'LEGS_BOOKED="R5"'
grep_in "R5's booked claims are the same names its leg command names" "$P5" 'CLAIMS_R5="$N5"'
grep_in "R6 books its one leg" "$P6" 'LEGS_BOOKED="R6"'
grep_in "R6's booked claim is the same name its leg command names" "$P6" 'CLAIMS_R6="$N6"'
# the booked set must be exactly the claim names the leg commands carry, computed, not just grepped
cl4=$(grep -m1 '^CLAIMS_ALL=' "$P4"); eval "$cl4"
[ "$(printf '%s\n' $CLAIMS_ALL | sort | tr '\n' ' ')" = "$(printf '%s\n' $N4A $N4B | sort | tr '\n' ' ')" ] && ok || bad "R4's booked claim set is not the set its leg commands name: got [$(printf '%s ' $CLAIMS_ALL)] want [$(printf '%s ' $N4A $N4B)]"
cl5=$(grep -m1 '^CLAIMS_ALL=' "$P5"); eval "$cl5"
[ "$(printf '%s\n' $CLAIMS_ALL | sort | tr '\n' ' ')" = "$(printf '%s\n' $N5 | sort | tr '\n' ' ')" ] && ok || bad "R5's booked claim set is not the set its leg command names"
cl6=$(grep -m1 '^CLAIMS_ALL=' "$P6"); eval "$cl6"
[ "$(printf '%s\n' $CLAIMS_ALL | sort | tr '\n' ' ')" = "$(printf '%s\n' $N6 | sort | tr '\n' ' ')" ] && ok || bad "R6's booked claim set is not the set its leg command names"
for f in $PKTS; do
  n=$(grep -m1 '^N_CLAIMS_BOOKED=' "$f"); c=$(grep -m1 '^CLAIMS_ALL=' "$f"); eval "$c"
  wc_=$(printf '%s\n' $CLAIMS_ALL | grep -c .)
  grep -q "N_CLAIMS_BOOKED=\$(printf" "$f" && ok || bad "$(basename "$f") does not compute its booked claim count"
  case "$(basename "$f")" in *_r4_*) want=6;; *_r5_*) want=5;; *) want=1;; esac
  [ "$wc_" = "$want" ] && ok || bad "$(basename "$f") books $wc_ claims, this row has $want"
done
# ordering: register before any leg writes evidence; cs_check before anything is summed
for f in $PKTS; do
  b=$(basename "$f")
  lreg=$(grep -n 'regmsg=$(register_claims 2>&1)' "$f" | head -1 | cut -d: -f1)
  lcs=$(grep -n '^cs_check() {' "$f" | head -1 | cut -d: -f1)
  lfv=$(grep -n 'final_verdict() {' "$f" | head -1 | cut -d: -f1)
  lgate=$(grep -n 'if ! cs_check; then' "$f" | head -1 | cut -d: -f1)
  lsum=$(awk -v s="$lfv" 'NR>s && /for f in "\$OUT"\/\*\.classes; do/ {print NR; exit}' "$f")
  lloop=$(grep -n '^leg \|^leg  ' "$f" | head -1 | cut -d: -f1)
  [ -n "$lreg" ] && ok || bad "$b never registers the booked claim set in its run path"
  [ -n "$lloop" ] && [ "$lreg" -lt "$lloop" ] && ok || bad "$b registers the claim set after its first leg call"
  [ -n "$lgate" ] && [ "$lgate" -gt "$lfv" ] && ok || bad "$b's final_verdict does not call cs_check"
  [ -n "$lgate" ] && [ "$lgate" -lt "$lsum" ] && ok || bad "$b sums classes files before consulting the claim-set gate"
  grep_in "$b also gates that the booked leg count is the leg count it runs" "$f" 'nbl_booked='
  grep_in "$b refuses a registration that disagrees with its own constants" "$f" 'the booked claim set could not be registered'
done
# the shipped cs_check, extracted and fed synthetic states, independent of the packet's own fixture harness
csx() { # csx <label> <expected rc> <expected substring> <setup>
  local label=$1 wantrc=$2 wantpat=$3 setup=$4 out rc
  rm -rf "$SB/cs"; mkdir -p "$SB/cs"
  out=$(bash -c "set -u; source '$SB/h4.sh'; OUT='$SB/cs'; O='$SB/cs/receipt.txt'
    LEGS_BOOKED='LEGX LEGY'; CLAIMS_ALL='a b c d'; CLAIMS_LEGX='a b'; CLAIMS_LEGY='c d'; N_CLAIMS_BOOKED=4; N_LEGS=2
    $setup
    cs_check; printf 'rc=%s cause=[%s]\\n' \"\$?\" \"\$CS_CAUSE\"" 2>&1)
  rc=$(printf '%s\n' "$out" | sed -n 's/^rc=\([0-9]*\) .*/\1/p')
  if [ "$rc" = "$wantrc" ] && printf '%s\n' "$out" | grep -q -- "$wantpat"; then
    echo "  ok   $label"; PASS_N=$((PASS_N+1))
  else
    bad "$label (want rc $wantrc and [$wantpat]; got: $(printf '%s\n' "$out" | tail -1))"; return
  fi; }
csx "a complete set is countable" 0 'cause=\[\]' '
  printf "%s\n" "LEGX a" "LEGX b" "LEGY c" "LEGY d" > "$OUT/claims-registered.txt"
  printf "%s\n" "a HELD" "b HELD" > "$OUT/LEGX.classes"
  printf "%s\n" "c HELD" "d HELD" > "$OUT/LEGY.classes"'
csx "no classes files at all is NOT countable" 1 classes-file-count '
  printf "%s\n" "LEGX a" "LEGX b" "LEGY c" "LEGY d" > "$OUT/claims-registered.txt"'
csx "one booked leg has no classes file at all" 1 leg-file-missing:LEGY '
  printf "%s\n" "LEGX a" "LEGX b" "LEGY c" "LEGY d" > "$OUT/claims-registered.txt"
  printf "%s\n" "a HELD" "b HELD" > "$OUT/LEGX.classes"'
csx "a booked leg whose classes file is short of its claims is NOT countable" 1 claim-count:LEGX '
  printf "%s\n" "LEGX a" "LEGX b" "LEGY c" "LEGY d" > "$OUT/claims-registered.txt"
  printf "%s\n" "a HELD" > "$OUT/LEGX.classes"
  printf "%s\n" "c HELD" "d HELD" > "$OUT/LEGY.classes"'
csx "an extra, unregistered claim is NOT countable" 1 claim-unregistered '
  printf "%s\n" "LEGX a" "LEGX b" "LEGY c" "LEGY d" > "$OUT/claims-registered.txt"
  printf "%s\n" "a HELD" "b HELD" "zzz HELD" > "$OUT/LEGX.classes"
  printf "%s\n" "c HELD" "d HELD" > "$OUT/LEGY.classes"'
csx "a claim named twice is NOT countable" 1 claim-duplicated '
  printf "%s\n" "LEGX a" "LEGX b" "LEGY c" "LEGY d" > "$OUT/claims-registered.txt"
  printf "%s\n" "a HELD" "b HELD" "b HELD" > "$OUT/LEGX.classes"
  printf "%s\n" "c HELD" "d HELD" > "$OUT/LEGY.classes"'
csx "a file for a leg that was never booked is NOT countable" 1 classes-file-not-booked '
  printf "%s\n" "LEGX a" "LEGX b" "LEGY c" "LEGY d" > "$OUT/claims-registered.txt"
  printf "%s\n" "a HELD" "b HELD" > "$OUT/LEGX.classes"
  printf "%s\n" "c HELD" "d HELD" > "$OUT/LEGY.classes"
  printf "%s\n" "a HELD" > "$OUT/LEGZ.classes"'
csx "a short or tampered registration is refused" 1 registration-claim-count '
  printf "%s\n" "LEGX a" "LEGX b" > "$OUT/claims-registered.txt"'
csx "a registration naming a claim this packet does not book is refused" 1 claim-not-booked '
  printf "%s\n" "LEGX a" "LEGX zzz" "LEGY c" "LEGY d" > "$OUT/claims-registered.txt"'
csx "no registration at all is refused" 1 registration-missing-or-empty 'true'
csx "a booked leg whose registered claims are absent from the registration is refused" 1 claim-not-registered '
  printf "%s\n" "LEGX a" "LEGX b" "LEGY c" "LEGY c" > "$OUT/claims-registered.txt"'
# the writer itself: exactly the booked pairs, and it refuses sets it cannot produce
rm -rf "$SB/csw"; mkdir -p "$SB/csw"
check_out "the writer produces exactly the booked registration" 0 "LEGX a LEGX b LEGY c LEGY d" bash -c "source '$SB/h4.sh'
  OUT='$SB/csw'; LEGS_BOOKED='LEGX LEGY'; CLAIMS_ALL='a b c d'; CLAIMS_LEGX='a b'; CLAIMS_LEGY='c d'; N_CLAIMS_BOOKED=4
  register_claims >/dev/null 2>&1; tr '\n' ' ' < \"\$OUT/claims-registered.txt\""
check_out "the writer refuses a booked leg that declares no claims" 1 "declares no claims" bash -c "source '$SB/h4.sh'
  OUT='$SB/csw'; LEGS_BOOKED='LEGX LEGY'; CLAIMS_ALL='a b'; CLAIMS_LEGX='a b'; CLAIMS_LEGY=''; N_CLAIMS_BOOKED=2
  register_claims 2>&1"
check_out "the writer refuses a registration whose count disagrees with the booked set" 1 "names 1 claims" bash -c "source '$SB/h4.sh'
  OUT='$SB/csw2'; mkdir -p '$SB/csw2'; LEGS_BOOKED='LEGX'; CLAIMS_ALL='a b'; CLAIMS_LEGX='a'; N_CLAIMS_BOOKED=2
  register_claims 2>&1"
# negative controls: each must make the packet's own fixtures fail
cat > "$SB/mut25.py" <<'PY25'
import sys
p=sys.argv[1]; s=open(p).read()
old = '  if ! cs_check; then'
assert old in s
open(p,"w").write(s.replace(old,'  if false; then',1))
PY25
cat > "$SB/mut26.py" <<'PY26'
import sys
p=sys.argv[1]; s=open(p).read()
old = "  local nbl; nbl=$(printf '%s\\n' $LEGS_BOOKED | grep -c .)"
assert old in s
open(p,"w").write(s.replace(old,'  local nbl=0',1))
PY26
cat > "$SB/mut27.py" <<'PY27'
import sys
p=sys.argv[1]; s=open(p).read()
old = "      printf '%s %s\\n' \"$L\" \"$n\" >> \"$OUT/claims-registered.txt\""
assert old in s
open(p,"w").write(s.replace(old,'      true',1))
PY27
for mu in 25 26 27; do
  negctl "cs_check/registration neutered (mutation $mu applied to the R4 packet)" "$P4" "$SB/mut$mu.py"
  negctl "cs_check/registration neutered (mutation $mu applied to the R5 packet)" "$P5" "$SB/mut$mu.py"
  negctl "cs_check/registration neutered (mutation $mu applied to the R6 packet)" "$P6" "$SB/mut$mu.py"
done
# the run-path registration call is not exercised by fixtures, so it is checked by grep on a mutated copy
cp "$P4" "$SB/negcs.sh"
python3 - "$SB/negcs.sh" <<'PY28'
import sys
p=sys.argv[1]; s=open(p).read()
old = 'regmsg=$(register_claims 2>&1)'
assert old in s
open(p,"w").write(s.replace(old,'regmsg=""',1))
PY28
grep -q 'regmsg=$(register_claims 2>&1)' "$P4" && ok || bad "the shipped R4 packet does not register the claim set in its run path"
grep -q 'regmsg=$(register_claims 2>&1)' "$SB/negcs.sh" && bad "the mutated copy still registers" || ok
grep -q 'regmsg=$(register_claims 2>&1)' "$P5" && grep -q 'regmsg=$(register_claims 2>&1)' "$P6" && ok || bad "R5 or R6 lost the run-path registration"

echo
echo "SELF-TEST: $PASS_N passed, $FAIL_N failed, $SKIP_N skipped"
if [ "$FAIL_N" -ne 0 ]; then printf 'FAILED: %s\n' "${FAILED[@]}"; exit 1; fi
exit 0
