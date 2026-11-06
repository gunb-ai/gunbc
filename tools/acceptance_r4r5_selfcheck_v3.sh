#!/usr/bin/env bash
# =====================================================================
# acceptance_r4r5_selfcheck_v3 — the fixture self-test for the two v3 packets (rows R4/R5, and row R6 held).
# NOTHING FROM EITHER REPOSITORY IS EXECUTED and nothing is run on srv1: it feeds synthetic claim logs,
# synthetic cgroup and memory.events files, synthetic run dirs and the packets' OWN tokenized command lines
# to the packets' own functions (extracted verbatim from between the markers), runs each packet's embedded
# `--classifier-fixtures` mode, exercises the packets end-to-end up to the point where a gate refuses on THIS
# host (before any clone, fetch, build or store contact), and checks the frozen constants against local git
# objects and against the pinned claim_batch parser.
#
# Each v3 item the acceptance check named gets an assertion in BOTH directions, with negative controls at the
# end: (1) the launcher frame, (2) the pre-work bounds gate, (3) the strict OOM contract, (4) the three-code
# exit contract, (5) the "discriminates nothing" statement, (6) wall accounting.
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
P45="$HERE/acceptance_r4_r5_claim_run_v3.sh"
P6="$HERE/acceptance_r6_live_read_claim_run_v3.sh"
REPO="${1:-}"
[ -f "$P45" ] && [ -f "$P6" ] || { echo "packets not found next to the self-check" >&2; exit 1; }
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

SB="$(mktemp -d "${TMPDIR:-/tmp}/r4r5-selfcheck-v3-XXXXXX")"
chmod 700 "$SB"
trap 'rm -rf "$SB"' EXIT
S4="$SB/fake/wt-r4/target/release/claim_batch"; S5="$SB/fake/wt-r5/target/release/claim_batch"; S6="$SB/fake/wt-r6/target/release/claim_batch"

# ---------------------------------------------------------------- read the packets' own constants and commands
# the packets' own constants, read out by NAME (an open-ended assignment grep would also pick up runtime
# lines like `O=$OUT/receipt.txt`, which is not a constant and would explode here)
csv() { local s; s=$(printf '%s,' $1); printf '%s' "${s%,}"; }
CONSTS='J|U|H|H4|H5|T|T4|T5|E4A|E4B|E5|E6|B4A|B4B|B5|B6|N4A|N4B|N5|N6|C4A|C4B|C5|C6|BOOKED|T_SETUP_CLONE|T_SETUP_FETCH|T_SETUP_CHECKOUT|T_BUILD|T_LEG'
eval "$(cat "$P45" "$P6" | grep -E "^($CONSTS)=")"
export BOOKED   # a bash -c probe must see the booked string the packet freezes
load_cmds() { eval "$(grep -E "^CMD_[A-Z0-9_]+=" "$1")"; }
load_cmds "$P45"; load_cmds "$P6"
for v in J H4 H5 T4 T5 E4A E4B E5 E6 B4A B4B B5 B6 N4A N4B N5 N6 C4A C4B C5 C6 BOOKED \
         T_SETUP_CLONE T_SETUP_FETCH T_SETUP_CHECKOUT T_BUILD T_LEG \
         CMD_CLONE_4 CMD_FETCH_4 CMD_CHECKOUT_4 CMD_BUILD_4 CMD_LEG_R4A CMD_LEG_R4B \
         CMD_CLONE_5 CMD_FETCH_5 CMD_CHECKOUT_5 CMD_BUILD_5 CMD_LEG_R5 \
         CMD_CLONE CMD_FETCH CMD_CHECKOUT CMD_BUILD CMD_LEG_R6; do
  [ -n "${!v:-}" ] && ok || bad "constant $v did not read out of the packets"
done
# the two packets must agree on the shared vocabulary, not merely resemble each other
for v in BOOKED T_SETUP_CLONE T_SETUP_FETCH T_SETUP_CHECKOUT T_BUILD T_LEG; do :; done
[ "$(grep -m1 '^BOOKED=' "$P45")" = "$(grep -m1 '^BOOKED=' "$P6")" ] && ok || bad "the two packets freeze different bounds"

# ---------------------------------------------------------------- each packet's embedded fixtures must pass
for f in "$P45" "$P6"; do
  out=$(bash "$f" --classifier-fixtures 2>&1); rc=$?
  np=$(printf '%s\n' "$out" | grep -c '^FIXTURE PASS'); nf=$(printf '%s\n' "$out" | grep -c '^FIXTURE FAIL')
  if [ "$rc" = 0 ] && [ "$nf" = 0 ] && [ "$np" -gt 60 ]; then ok; else bad "$(basename "$f") embedded fixtures: rc=$rc pass=$np fail=$nf"; fi
  echo "  $(basename "$f"): $np embedded fixtures pass, $nf fail (rc=$rc)"
done

# ---------------------------------------------------------------- the helpers, verbatim, fed synthetic input
helpers() { sed -n '/# ---------------------------------------------------------------- gates and helpers/,/# ---------------------------------------------------------------- fixture mode/p' "$1" | sed '$d'; }
helpers "$P45" > "$SB/h45.sh"; helpers "$P6" > "$SB/h6.sh"
for f in "$SB/h45.sh" "$SB/h6.sh"; do
  for fn in scope_gate bounds_check out_absent bin_sha oom_parse oom_key oom_pair oom_verdict oom_pair_verdict oom_snap claim_classify leg_state stray_verdict_lines names_gate tm; do
    grep -q "^$fn()" "$f" && ok || bad "$(basename "$f") does not define $fn"
  done
done
H="$SB/h45.sh"

echo "== the classifier: bare PASS/FAIL only, one per name, and the runner's exit code checked apart"
printf 'PASS witness_inbox_stale_live_receipt_refuses\n' > "$SB/p"
check "a bare PASS line with rc 0 is an observation" 0 bash -c "source $H; [ \"\$(claim_classify '$SB/p' 0 witness_inbox_stale_live_receipt_refuses)\" = HELD ]"
printf 'FAIL witness_inbox_stale_live_receipt_refuses\n' > "$SB/f"
check "a bare FAIL line with rc 1 is an observation (a falsified expectation)" 0 bash -c "source $H; [ \"\$(claim_classify '$SB/f' 1 witness_inbox_stale_live_receipt_refuses)\" = FAILED ]"
check "a bare PASS with rc 1 is NOT an observation" 0 bash -c "source $H; [ \"\$(claim_classify '$SB/p' 1 witness_inbox_stale_live_receipt_refuses)\" = NO_OBSERVATION ]"
check "all-pass but exit 1 (the cost-accounting partition refused) is NOT an observation" 0 bash -c "source $H; [ \"\$(claim_classify '$SB/p' 1 witness_inbox_stale_live_receipt_refuses)\" = NO_OBSERVATION ]"
check "a bare FAIL with rc 0 is NOT an observation" 0 bash -c "source $H; [ \"\$(claim_classify '$SB/f' 0 witness_inbox_stale_live_receipt_refuses)\" = NO_OBSERVATION ]"
check "rc 124 (timeout) is NOT an observation" 0 bash -c "source $H; [ \"\$(claim_classify '$SB/p' 124 witness_inbox_stale_live_receipt_refuses)\" = NO_OBSERVATION ]"
check "rc 137 (signalled) is NOT an observation" 0 bash -c "source $H; [ \"\$(claim_classify '$SB/p' 137 witness_inbox_stale_live_receipt_refuses)\" = NO_OBSERVATION ]"
check "a missing rc is NOT an observation" 0 bash -c "source $H; [ \"\$(claim_classify '$SB/p' missing witness_inbox_stale_live_receipt_refuses)\" = NO_OBSERVATION ]"
check "an empty rc is NOT an observation" 0 bash -c "source $H; [ \"\$(claim_classify '$SB/p' '' witness_inbox_stale_live_receipt_refuses)\" = NO_OBSERVATION ]"
check "no output file at all is NOT an observation" 0 bash -c "source $H; [ \"\$(claim_classify '$SB/nothing' 0 witness_inbox_stale_live_receipt_refuses)\" = NO_OBSERVATION ]"
: > "$SB/e"; check "an empty output file is NOT an observation" 0 bash -c "source $H; [ \"\$(claim_classify '$SB/e' 0 witness_inbox_stale_live_receipt_refuses)\" = NO_OBSERVATION ]"
check "a name this leg did not ask for is NOT an observation" 0 bash -c "source $H; [ \"\$(claim_classify '$SB/p' 0 witness_other)\" = NO_OBSERVATION ]"
printf 'PASS x\nPASS x\n' > "$SB/dup"; check "a duplicate PASS line is NOT an observation" 0 bash -c "source $H; [ \"\$(claim_classify '$SB/dup' 0 x)\" = NO_OBSERVATION ]"
printf 'PASS x\nFAIL x\n' > "$SB/con"; check "PASS and FAIL for one name is NOT an observation" 0 bash -c "source $H; [ \"\$(claim_classify '$SB/con' 1 x)\" = NO_OBSERVATION ]"
printf '  PASS x\n' > "$SB/ind"; check "an indented verdict line is NOT an observation" 0 bash -c "source $H; [ \"\$(claim_classify '$SB/ind' 0 x)\" = NO_OBSERVATION ]"
for q in 'FAIL x (runtime error [E]: boom)' 'FAIL x (returned ProcessExit::ExitFailure, code 3: nope)' 'UNOBSERVED x (route gap; see remedy)' 'PANICKED x (the host unwound)'; do
  printf '%s\n' "$q" > "$SB/q"
  check "a qualified line is NOT an observation: ${q:0:12}" 0 bash -c "source $H; [ \"\$(claim_classify '$SB/q' 2 x)\" = NO_OBSERVATION ]"
  check_out "the stray detector sees the qualified line: ${q:0:12}" 0 "stray=1 exact=0" bash -c "source $H; stray_verdict_lines '$SB/q' 'x'"
done
printf 'PASS a\nFAIL b (runtime error [E]: boom)\nPASS c\n' > "$SB/mixed"
check_out "the stray detector fires on one qualified line among clean ones" 0 "stray=3 exact=2" bash -c "source $H; stray_verdict_lines '$SB/mixed' 'a b c'"
printf 'PASS a\nPASS b\nPASS c\n' > "$SB/clean"
check "a clean bare-verdict log passes the stray detector" 0 bash -c "source $H; [ \"\$(stray_verdict_lines '$SB/clean' 'a b c')\" = 0 ]"
printf 'PASS a\n' > "$SB/one"; check "a verdict for a name outside the leg is stray" 0 bash -c "source $H; [ \"\$(stray_verdict_lines '$SB/one' 'b')\" != 0 ]"

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
  check_out "bounds gate refuses a wrong $k with the value in the message" 2 "booked bounds" bash -c "source $H; BOOKED=\"\$(grep -m1 '^BOOKED=' $P45 | cut -d= -f2-)\"; m=\$(bounds_check '$SB/cg') || { echo \"booked bounds: \$m\"; exit 2; }; exit 0"
  case "$k" in memory.max) printf '34359738368\n' > "$SB/cg/$k";; memory.high) printf '30064771072\n' > "$SB/cg/$k";; memory.swap.max) printf '0\n' > "$SB/cg/$k";; pids.max) printf '4096\n' > "$SB/cg/$k";; cpu.max) printf '3200000 100000\n' > "$SB/cg/$k";; esac
done
printf 'max\n' > "$SB/cg/memory.max"
check "an unlimited cgroup (memory.max is the literal 'max') is refused, not adapted to" 0 bash -c "source $H; ! bounds_check '$SB/cg' >/dev/null"
printf '34359738368\n' > "$SB/cg/memory.max"; mv "$SB/cg/cpu.max" "$SB/cg/cpu.gone"
check "an unreadable counter is refused" 0 bash -c "source $H; ! bounds_check '$SB/cg' >/dev/null"
mv "$SB/cg/cpu.gone" "$SB/cg/cpu.max"
check "the bounds gate is a gate, not a dump: bounds_check output decides the exit" 0 bash -c "grep -q 'BM=\$(bounds_check' '$P45' && grep -q 'booked bounds' '$P45'"
check "and it runs before the first fetch" 0 bash -c "a=\$(grep -n 'BM=\$(bounds_check' '$P45' | cut -d: -f1); b=\$(grep -n 'CMD_FETCH_4\"' '$P45' | head -1 | cut -d: -f1); [ -n \"\$a\" ] && [ -n \"\$b\" ] && [ \"\$a\" -lt \"\$b\" ]"
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
for f in "$P45" "$P6"; do
  b=$(basename "$f")
  grep_in "$b names the calmcrab-udrv v4 launcher" "$f" "calmcrab-udrv v4"
  grep_in "$b states the full argv tuple" "$f" "<JOB> <RUNTIME> <MEMMAX> <MEMHIGH> <TASKS> <CPUQ> <STAGED> <SHA> <OUT>"
  grep_in "$b states the -res ledger row" "$f" "-res"
  grep_in "$b states the -drv ledger row" "$f" "-drv"
  grep_in "$b states the driver unit" "$f" "calmcrab-drv-"
  grep_in "$b states the scope unit" "$f" ".scope"
  grep_not "$b does not launch with bare systemd-run" "$f" "systemd-run --"
  grep_not "$b does not book anything itself" "$f" "systemctl "
  grep_in "$b states the full argv tuple" "$f" "<JOB> <RUNTIME> <MEMMAX> <MEMHIGH> <TASKS> <CPUQ> <STAGED> <SHA> <OUT>"
  grep_in "$b states the reservation ledger row" "$f" "$(grep -m1 '^J=' "$f" | cut -d= -f2)-res"
  grep_in "$b states the driver ledger row" "$f" "$(grep -m1 '^J=' "$f" | cut -d= -f2)-drv"
  grep_in "$b states the row ledger row" "$f" "$(grep -m1 '^J=' "$f" | cut -d= -f2)              cap"
  grep_in "$b states the driver unit" "$f" "calmcrab-drv-$(grep -m1 "^J=" "$f" | cut -d= -f2).service"
done
check "the R4/R5 job token is the row name" 0 bash -c "[ \"\$(grep -m1 '^J=' '$P45' | cut -d= -f2)\" = fierce-wren-19-r4r5-db56530f ]"
check "the R6 job token is the row name" 0 bash -c "[ \"\$(grep -m1 '^J=' '$P6' | cut -d= -f2)\" = fierce-wren-19-r6-dff57af7 ]"
check "JOB is derived from the launcher's OUT, not assumed" 0 bash -c "grep -q 'JOB=\$(basename \"\$(dirname \"\$OUT\")\"' '$P45' && grep -q 'JOB=\$(basename \"\$(dirname \"\$OUT\")\"' '$P6'"
check "and the derivation is enforced: a job dir mismatch refuses the run" 0 bash -c "source $H; ! scope_gate '0::/x/wrong-job.scope' wrong-job2 >/dev/null"

echo "== the exit contract (item 4): 0 held, 1 falsified, 2 everything else"
for f in "$P45" "$P6"; do
  b=$(basename "$f")
  code=$(grep -vE '^[[:space:]]*#' "$f")
  n1=$(printf '%s\n' "$code" | grep -cE '\bexit 1\b'); [ "$n1" = 1 ] && ok || bad "$b has $n1 executable 'exit 1' (want exactly 1: the falsified branch)"
  n3=$(printf '%s\n' "$code" | grep -cE '\bexit 3\b'); [ "$n3" = 0 ] && ok || bad "$b still has an 'exit 3' ($n3)"
  n2=$(printf '%s\n' "$code" | grep -cE '\bexit 2\b'); [ "$n2" -ge 8 ] && ok || bad "$b has only $n2 exit-2 refusals"
  grep_in "$b routes every gate refusal through the nox helper" "$f" "nox() {"
  check "$b's nox helper can only exit 2" 0 bash -c "grep 'nox() {' '$f' | grep -q 'exit 2; }' || grep -A3 'nox() {' '$f' | grep -q 'exit 2'"
done
check_out "usage with no argument is NO_OBSERVATION (exit 2)" 2 "NO_OBSERVATION" bash "$P45"
check_out "usage with two arguments is NO_OBSERVATION (exit 2)" 2 "NO_OBSERVATION" bash "$P45" a b
check_out "an unknown flag is not swallowed as a path (exit 2)" 2 "NO_OBSERVATION" bash "$P45" --nope
check_out "a run dir that is not a job's out dir is refused (exit 2)" 2 "does not end in /out" bash "$P45" "$SB/notout"
check "…and nothing was created for it" 0 bash -c "[ ! -e '$SB/notout' ]"
check_out "held R6 with no argument is NO_OBSERVATION (exit 2)" 2 "NO_OBSERVATION" bash "$P6"
check_out "held R6 with OUT but no grant reference refuses (exit 2)" 2 "msg_3407a188" bash "$P6" "$SB/r6job/out"
check "…and the hold leaves nothing behind (no OUT, no clone)" 0 bash -c "[ ! -e '$SB/r6job/out' ]"
check_out "held R6 refuses a flag where the grant reference belongs (exit 2)" 2 "execution-grant reference" bash "$P6" "$SB/r6job/out" --yes
check "…still nothing behind it" 0 bash -c "[ ! -e '$SB/r6job/out' ]"
check_out "held R6 --predict-only states the hold and exits 0" 0 "state=HELD" bash "$P6" --predict-only
check "…and --predict-only creates nothing" 0 bash -c "[ ! -e '$SB/predict' ] && cd '$SB' && bash '$P6' --predict-only >/dev/null"
check "the grant gate refuses blank, whitespace and flag-like references" 0 bash -c "source $(helpers "$P6" > $SB/h6b.sh; echo $SB/h6b.sh); grant_gate '' >/dev/null && exit 1; grant_gate '  ' >/dev/null && exit 1; grant_gate '--x' >/dev/null && exit 1; grant_gate 'keen-otter-cleared' >/dev/null"

echo "== the frame's own claims are testable: timeouts, wall arithmetic, no mutation"
for f in "$P45" "$P6"; do
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
check "the R6 header states the launcher's full argv and its own ledger rows" 0 bash -c "grep -q 'fierce-wren-19-r6-dff57af7-res' '$P6' && grep -q 'fierce-wren-19-r6-dff57af7-drv' '$P6'"
check "the R4/R5 header's stated capped sum equals its timeouts" 0 bash -c "source <(grep -E '^T_(SETUP|BUILD|LEG)' '$P45'); [ 8940 = \$(( 2*(T_SETUP_CLONE+T_SETUP_FETCH+T_SETUP_CHECKOUT) + 2*T_BUILD + 3*T_LEG )) ]"
check "the R6 header's stated capped sum equals its timeouts" 0 bash -c "source <(grep -E '^T_(SETUP|BUILD|LEG)' '$P6'); [ 4020 = \$(( T_SETUP_CLONE+T_SETUP_FETCH+T_SETUP_CHECKOUT + T_BUILD + T_LEG )) ]"
check "scope RuntimeMaxSec is at least the capped sum in both packets" 0 bash -c "grep -q 'wall_capped_sum_s=\$((.*)) scope_RuntimeMaxSec=9900' '$P45' && grep -q 'scope_RuntimeMaxSec=4800' '$P6'"
check "the driver wall is sized from the scope wall" 0 bash -c "grep -q 'driver_RuntimeMaxSec=10500' '$P45' && grep -q 'driver_RuntimeMaxSec=5400' '$P6'"
check "10500 > 9900 and 5400 > 4800" 0 bash -c "[ 10500 -gt 9900 ] && [ 5400 -gt 4800 ]"

echo "== the OOM contract as written: exclusive publication before parse, no grep-judged counters"
for f in "$P45" "$P6"; do
  b=$(basename "$f")
  grep_in "$b publishes the snapshot by exclusive create" "$f" "set -C"
  grep_not "$b never judges memory.events with a grep pattern" "$f" "grep -E '^(oom_kill|max) [1-9]"
  grep_not "$b never silently cats a judged counter" "$f" "cat /sys/fs/cgroup/memory.events"
  ns=$(grep -c 'oom_snap ' "$f"); nv=$(grep -c 'oom_pair_verdict' "$f")
  [ "$nv" -ge 2 ] && ok || bad "$b has $ns snapshots but only $nv verdicts"
done
for f in "$P45" "$P6"; do nv=$(grep -c 'oom_pair_verdict' "$f"); [ "$nv" -ge 4 ] && ok || bad "$(basename "$f") compares snapshots only $nv times (gate, build, legs, final)"; done
check "memory.peak is printed, never judged" 0 bash -c "grep -q 'memory.peak (printed, never judged)' '$P45' && grep -q 'memory.peak (printed, never judged)' '$P6'"
check "the R4/R5 packet compares the FINAL snapshot against the last leg" 0 bash -c "grep -q 'oom_pair_verdict \"\$LASTOOM\" \"\$FA\"' '$P45' && grep -q 'oom_pair_verdict \"\$LASTOOM\" \"\$FA\"' '$P6'"

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
for b in "$P45" "$P6"; do
  for v in $(grep -oE '^CMD_[A-Z0-9_]+' "$b"); do
    grep -q "\"\$$v\"" "$b" && ok || bad "$(basename "$b") never uses \$$v — the manifest would print a string the run does not execute"
  done
  grep -q 'leg R4a "\$CMD_LEG_R4A"' "$b" && ok || true
done
grep -q 'leg R4a "\$CMD_LEG_R4A"' "$P45" && ok || bad "the R4/a leg does not pass the manifest string to the leg runner"
grep -q 'leg R4b "\$CMD_LEG_R4B"' "$P45" && ok || bad "the R4/b leg does not pass the manifest string to the leg runner"
grep -q 'leg R5  "\$CMD_LEG_R5"' "$P45" && ok || bad "the R5 leg does not pass the manifest string to the leg runner"
grep -q 'leg R6 "\$CMD_LEG_R6"' "$P6" && ok || bad "the R6 leg does not pass the manifest string to the leg runner"
check "the built runner's directory is EXPORTED to the child shell a bash -c leg runs in" 0 bash -c "grep -q 'export OUT BIN4 BIN5' '$P45' && grep -q 'export BIN6' '$P6'"
check "…because a plain assignment does not reach a child shell (a latent v2 defect this rewrite fixes)" 0 bash -c "[ -z \"\$(v=1; bash -c 'printf %s \"\$v\"')\" ]"

echo "== the manifest: bytes and sha256 for every command string, printed by the packet itself"
M45=$(bash "$P45" --predict-only); M6=$(bash "$P6" --predict-only)
n=$(printf '%s\n' "$M45" | grep -c '^cmd '); [ "$n" = 11 ] && ok || bad "the R4/R5 manifest prints $n cmd lines (want 11)"
n=$(printf '%s\n' "$M6" | grep -c '^cmd '); [ "$n" = 5 ] && ok || bad "the R6 manifest prints $n cmd lines (want 5)"
n=$(printf '%s\n' "$M45" | grep -c '^blob '); [ "$n" = 3 ] && ok || bad "the R4/R5 manifest prints $n blob lines (want 3)"
n=$(printf '%s\n' "$M6" | grep -c '^blob '); [ "$n" = 1 ] && ok || bad "the R6 manifest prints $n blob lines (want 1)"
printf '%s\n' "$M45" | grep -qE "^pin_r4=$H4( |$)" && ok || bad "the R4/R5 manifest does not print the frozen R4 pin"
printf '%s\n' "$M45" | grep -qE "^pin_r5=$H5( |$)" && ok || bad "the R4/R5 manifest does not print the frozen R5 pin"
printf '%s\n' "$M45" | grep -qE "tree_r4=$T4" && ok || bad "the R4/R5 manifest does not print the frozen R4 tree"
printf '%s\n' "$M45" | grep -qE "tree_r5=$T5" && ok || bad "the R4/R5 manifest does not print the frozen R5 tree"
printf '%s\n' "$M6" | grep -qE "^pin=$H5( |$)" && ok || bad "the R6 manifest does not print its frozen pin"
printf '%s\n' "$M6" | grep -qE "tree=$T5" && ok || bad "the R6 manifest does not print its frozen tree"
printf '%s\n' "$M45" | grep -q '^cmd CMD_LEG_R4A bytes=[0-9]* sha256=[0-9a-f]\{64\}$' && ok || bad "a manifest line lacks bytes+sha256"
printf '%s\n' "$M45" | grep -q '^bounds=memory.max=34359738368' && ok || bad "the manifest does not print the booked bounds"
printf '%s\n' "$M45" | grep -q 'scope_RuntimeMaxSec=9900' && ok || bad "the manifest does not state the scope wall"
printf '%s\n' "$M6" | grep -q 'state=HELD' && ok || bad "the R6 manifest does not state the hold"
d=$(diff <(printf '%s\n' "$M45" | grep -E '^(cmd|blob|pin|tree|bounds|wall|claims|exit)') <(cd "$SB" && bash "$P45" --predict-only | grep -E '^(cmd|blob|pin|tree|bounds|wall|claims|exit)'))
[ -z "$d" ] && ok || bad "the manifest is not reproducible across cwds: $(printf '%s' "$d" | head -3)"
printf '%s\n' "$M45" | grep -A1 '^cmd CMD_LEG_R4A' | tail -1 | grep -qF -- "--functions \"$C4A\"" && ok || bad "the printed R4/a command lost the frozen CSV"
printf '%s\n' "$M45" | grep -A1 '^cmd CMD_LEG_R4A' | tail -1 | grep -qF 'claim_batch --source-root' && bad "the printed R4/a command carries the subcommand token again" || ok
printf '%s\n' "$M6" | grep -A1 '^cmd CMD_LEG_R6' | tail -1 | grep -qF -- "--entry \"$E6\" --functions \"$C6\"" && ok || bad "the printed R6 command lost its entry or CSV"
check "predict-only writes nothing anywhere under the run root" 0 bash -c "cd '$SB' && before=\$(find '$SB' -type f | wc -l) && bash '$P45' --predict-only >/dev/null && bash '$P6' --predict-only >/dev/null && after=\$(find '$SB' -type f | wc -l); [ \"\$before\" = \"\$after\" ] || { find '$SB' -type f -newer '$P45' | head; exit 1; }"

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
check_out "the shipped packet stops at a named missing-tool gate before creating OUT" 2 "/usr/bin/time" bash "$P45" "$SB/tooljob/out"
check "…and it created nothing" 0 bash -c "[ ! -e '$SB/tooljob/out' ]"
cp "$P45" "$SB/relaxed45.sh"; cp "$P6" "$SB/relaxed6.sh"
python3 - "$SB" <<'PY'
import sys,re
sb=sys.argv[1]
for f in (sb+"/relaxed45.sh", sb+"/relaxed6.sh"):
    s=open(f).read()
    s2=s.replace('[ -x /usr/bin/time ]','[ -x /bin/true ]')
    s2=s2.replace('/usr/bin/time -v bash -c','bash -c')
    assert s2!=s, f
    open(f,'w').write(s2)
PY
for pair in "$SB/relaxed45.sh:" "$SB/relaxed6.sh:grantish"; do
  f=${pair%%:*}; extra=${pair##*:}
  jb=$SB/probe-$(basename "$f")-job; mkdir -p "$jb"
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
grep_in "the R4/R5 header states the row discriminates nothing" "$P45" "DISCRIMINATES NOTHING"
grep_in "the R4/R5 header says presence observations" "$P45" "PRESENCE observations"
grep_in "the R4/R5 header says no mutation leg is run or proposed" "$P45" "NO mutation leg is run or proposed"
grep_in "the R6 header states the hold and the gate" "$P6" "msg_3407a188"
grep_in "the R6 header states a refusal of the subject is a finding" "$P6" "fails"
grep_in "both packets record the row exit as an observation, not a tool status" "$P45" "VERDICT="

# ---------------------------------------------------------------- negative controls: mutate a copy, expect failure
echo "== negative controls (a mutated packet must FAIL, or the checks are decorative)"
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
negctl "defect (1) reintroduced: a literal claim_batch token after the binary" "$P45" "$SB/mut1.py"
negctl "defect (2) reintroduced: a space-separated name in the CSV" "$P45" "$SB/mut2.py"
negctl "the booked-bounds gate deleted from the run path (item 2)" "$P45" "$SB/mut3.py"
negctl "noclobber removed from the OOM publication (item 3)" "$P45" "$SB/mut4.py"
negctl "the no-observation refusal switched to a fourth exit code (item 4)" "$P45" "$SB/mut5.py"
negctl "the last-leg OOM pair faked as unchanged (item 3)" "$P45" "$SB/mut6.py"
negctl "defect (1) in the R6 packet as well" "$P6" "$SB/mut7.py"
# mut3 and mut5 change the run path rather than a helper; the packet's self-structure section is what
# notices, so they are exercised as fixture controls too. The greps below are a second, independent signal.
cp "$P45" "$SB/neg3.sh"; python3 "$SB/mut3.py" "$SB/neg3.sh" && ok || bad "bounds-gate mutation did not apply"
grep -q 'BM=""' "$SB/neg3.sh" && ok || bad "the bounds gate is still present in the mutated copy"
grep -q 'BM=\$(bounds_check' "$SB/neg3.sh" && bad "the mutated copy still gates on bounds_check in its run path" || ok
cp "$P45" "$SB/neg5.sh"; python3 "$SB/mut5.py" "$SB/neg5.sh" && ok || bad "exit-code mutation did not apply"
n3=$(grep -vE '^[[:space:]]*#' "$SB/neg5.sh" | grep -cE '\bexit 3\b'); [ "$n3" = 1 ] && ok || bad "expected the mutated copy to reintroduce an exit 3, got $n3"
check "…which is the rule the self-test applies to the shipped packets (no exit 3 anywhere)" 0 bash -c "[ \$(grep -vE '^[[:space:]]*#' '$P45' | grep -cE '\bexit 3\b') = 0 ] && [ \$(grep -vE '^[[:space:]]*#' '$P6' | grep -cE '\bexit 3\b') = 0 ]"
check "…and exactly one executable exit 1 per shipped packet" 0 bash -c "[ \$(grep -vE '^[[:space:]]*#' '$P45' | grep -cE '\bexit 1\b') = 1 ] && [ \$(grep -vE '^[[:space:]]*#' '$P6' | grep -cE '\bexit 1\b') = 1 ]"

echo
echo "SELF-TEST: $PASS_N passed, $FAIL_N failed, $SKIP_N skipped"
if [ "$FAIL_N" -ne 0 ]; then printf 'FAILED: %s\n' "${FAILED[@]}"; exit 1; fi
exit 0
