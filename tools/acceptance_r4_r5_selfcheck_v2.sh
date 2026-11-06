#!/usr/bin/env bash
# =====================================================================
# acceptance_r4_r5_selfcheck_v2 — the fixture self-test for the frozen R4/R5 packet and the held
# R6 packet. NOTHING FROM THE REPOS IS EXECUTED and NOTHING IS RUN ON srv1: it feeds synthetic
# claim logs, synthetic run dirs, synthetic cgroup files and synthetic memory.events to the packets'
# OWN gate functions (extracted verbatim from between the FUNCTIONS markers), TOKENIZES the packets'
# own leg command strings exactly as `bash -c` will and checks the argv against the accepted flag
# set of claim_batch's real parser, and checks the frozen constants against local git objects.
#
# Every gate is tested in BOTH directions, and each refusal is checked for its EXIT CODE, not just
# its text. v2 adds the two things v1 lacked, both of which were fatal in v1: the argv-vs-parser
# check (a literal `claim_batch` subcommand token, and space-separated --functions, would each have
# burned a ~3h grant for nothing), and the launcher run-dir/scope convention.
#
# Content checks are gated on byte identity, never on proximity: a working-tree file is used as the
# pinned bytes only when `git hash-object` of it equals the blob id the packets freeze AT the pin;
# otherwise the check reports SKIPPED, never passed. (This repo is a blob:none partial clone:
# `git cat-file blob <pin>:<path>` would lazily FETCH, and a fetch needing credentials hangs — so
# pinned bytes are reached through local tree objects plus hash-equality, and every git call here
# gets /dev/null on stdin.)
#
# usage: acceptance_r4_r5_selfcheck_v2.sh [repo-with-pinned-commits]
set -uo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
PACKET="$HERE/acceptance_r4_r5_claim_run_v2.sh"
R6PACKET="$HERE/acceptance_r6_live_read_claim_run_v2.sh"
REPO="${1:-}"
[ -f "$PACKET" ] && [ -f "$R6PACKET" ] || { echo "packets not found next to the self-check" >&2; exit 1; }
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

SB="$(mktemp -d "${TMPDIR:-/tmp}/r4r5-selfcheck-v2-XXXXXX")"
chmod 700 "$SB"
trap 'rm -rf "$SB"' EXIT
# the fake binary path a leg would exec; the argv checks require argv[0] to be exactly this, so a
# packet that pointed at a different path, or exec'd a wrapper, fails here
SENTINEL4="$SB/fake/wt-r4/target/release/claim_batch"
SENTINEL5="$SB/fake/wt-r5/target/release/claim_batch"

# ---------------------------------------------------------------- constants, as the packets freeze them
eval "$(cat "$PACKET" "$R6PACKET" | grep -E '^(PIN_R4|PIN_R5|PIN|BLOB_R4_STAGE0|BLOB_R4_CRATE|BLOB_R5_INBOX|BLOB_LIVE_WITNESS|ENTRY_R4_STAGE0|ENTRY_R4_CRATE|ENTRY_R5_INBOX|ENTRY|ROWS_R4A|ROWS_R4B|ROWS_R5|CLAIM_R6|EXPECT_R4A|EXPECT_R4B|EXPECT_R5|JOBS|ORIGIN|JOB_HINT)=' | sed 's/^/declare -g /')"
for v in PIN_R4 PIN_R5 PIN BLOB_R4_STAGE0 BLOB_R4_CRATE BLOB_R5_INBOX BLOB_LIVE_WITNESS ENTRY_R4_STAGE0 ENTRY_R4_CRATE ENTRY_R5_INBOX ENTRY CLAIM_R6 ROWS_R4A ROWS_R4B ROWS_R5 EXPECT_R4A EXPECT_R4B EXPECT_R5 JOBS ORIGIN JOB_HINT; do
  [ -n "${!v:-}" ] && ok || bad "constant $v did not read out of the packets"
done
# The frozen command strings are read out of the packets as VALUES, not as source lines: every CMD_*
# is a single-quoted literal whose $BIN/$BIN5 stay unexpanded until the packet evals them, and whose
# $ENTRY/$ROWS the packet splices at definition time. So: define the constants, then eval the
# packet's own CMD_* definition lines, and nothing here can drift from what the packet runs.
load_cmds() { # load_cmds <packet>
  eval "$(grep -E "^CMD_[A-Z0-9_]+='" "$1")"
}
load_cmds "$PACKET"; load_cmds "$R6PACKET"
for v in CMD_CLONE_R4 CMD_FETCH_R4 CMD_CHECKOUT_R4 CMD_BUILD_R4 CMD_LEG_R4A CMD_LEG_R4B CMD_LEG_R5 CMD_LEG_R6; do
  [ -n "${!v:-}" ] && ok || bad "$v did not read out of the packets as a value"
done
# every name is read BACK out of the CMD_LEG_* value, so the test cannot drift from the packet

csv_of() { # csv_of <var> -> the --functions value, taken from the tokenized command
  local var="$1"
  [ -n "${!var:-}" ] || { printf ''; return; }
  ( BIN="$SENTINEL4"; BIN5="$SENTINEL5"; eval "set -- ${!var}"
    while [ $# -gt 0 ]; do if [ "$1" = "--functions" ]; then printf '%s' "$2"; return; fi; shift; done )
}
for pair in "CMD_LEG_R4A:$EXPECT_R4A" "CMD_LEG_R4B:$EXPECT_R4B" "CMD_LEG_R5:$EXPECT_R5"; do
  v="${pair%%:*}"; want="${pair##*:}"
  csv=$(csv_of "$v")
  n=$(printf '%s' "$csv" | awk -F',' '{print NF}')
  [ "$n" = "$want" ] && ok || bad "$v names $n CSV functions ('$csv'), the packet expects $want verdicts"
done
csv=$(csv_of CMD_LEG_R6)
[ "$csv" = "$CLAIM_R6" ] && ok || bad "the R6 leg's --functions is '$csv', the packet freezes '$CLAIM_R6'"
# the claim names in the CSV must be exactly the ROWS_* the packet freezes
for pair in "CMD_LEG_R4A:ROWS_R4A" "CMD_LEG_R4B:ROWS_R4B" "CMD_LEG_R5:ROWS_R5" "CMD_LEG_R6:CLAIM_R6"; do
  v="${pair%%:*}"; r="${pair##*:}"
  [ "$(csv_of "$v")" = "${!r}" ] && ok || bad "$v's --functions is not the frozen $r"
done

# ---------------------------------------------------------------- the functions under test, verbatim
extract() { sed -n '/# ---8< FUNCTIONS BEGIN/,/# ---8< FUNCTIONS END/p' "$1"; }
extract "$PACKET" > "$SB/fn_r4r5.sh"
extract "$R6PACKET" > "$SB/fn_r6.sh"
for f in "$SB/fn_r4r5.sh" "$SB/fn_r6.sh"; do
  for gate in require_dir_gate require_scope_gate require_bound_gate require_blob_gate classify raw_verify ev_parse mem_events scope_ok; do
    grep -q "^$gate()" "$f" && ok || bad "marker region of $f does not define $gate"
  done
done

new_case() { # new_case <tag> — the launcher's shape: $HOME-ish runs/<JOB>/out
  local tag="$1"
  local base="$SB/case-$tag"
  mkdir -p "$base/runs/$tag/out" 2>/dev/null
  # the out dir is created by the packet; new_case prepares the JOB dir only when ask_out=0
  rm -rf "$base/runs/$tag/out"
  RUN_ID="$tag"; RUN_DIR="$base/runs/$tag/out"; JOB="$tag"
  BINDING_HOST="$(hostname)"; BINDING_IDENTITY="selfcheck-id"
}

for f in "$PACKET" "$R6PACKET"; do
  bash -n "$f" && ok || bad "$(basename "$f") does not parse (bash -n)"
done
echo "== the ARGV, tokenized as bash -c tokenizes it, checked against claim_batch's accepted flags"
# The accepted flag set and its arity, read from the parser at the pin (checked there below), not
# invented here.
FLAGS_ONE="--source-root --entry --function --functions --eval-budget-ms --fixture-store --outcomes-out"
FLAGS_ZERO="--claim-run --wet --hermetic --record --pre-push --print-entry-closure"
tokenize() { # tokenize <CMDVAR> -> one token per line, exactly as `bash -c "$CMD_…"` hands them to exec
  local var="$1"
  [ -n "${!var:-}" ] || return 0
  ( BIN="$SENTINEL4"; BIN5="$SENTINEL5"
    eval "set -- ${!var}" 2>/dev/null
    for t in "$@"; do printf '%s\n' "$t"; done )
}
check_argv() { # check_argv <label> <CMDVAR> <expected-claim-count>
  local label="$1" var="$2" wantn="$3"
  local n=0 i=0 bad_tok="" fns="" nfn=0 name
  local -a T; mapfile -t T < <(tokenize "$var")
  if [ "${#T[@]}" -eq 0 ]; then bad "$label: $var did not tokenize (it does not parse as a command line)"; return; fi
  if [ "${T[0]}" != "$SENTINEL4" ] && [ "${T[0]}" != "$SENTINEL5" ]; then
    bad "$label: argv[0] is '${T[0]}' — the packet must exec the binary it builds, at the path it builds"; return
  fi
  case "${T[0]}" in */claim_batch) ;; *) bad "$label: argv[0] does not end in /claim_batch: '${T[0]}'"; return ;; esac
  if [ "${T[1]:-}" = "claim_batch" ]; then bad "$label: argv[1] is the literal token 'claim_batch' (the parser's other => arm exits 2)"; return; fi
  while [ $i -lt ${#T[@]} ]; do
    local t="${T[$i]}"
    if [ "$i" = 0 ]; then i=$((i+1)); continue; fi
    case " $FLAGS_ONE " in *" $t "*)
      n=$((n+1))
      if [ $((i+1)) -ge ${#T[@]} ]; then bad "$label: $t has no value"; return; fi
      if [ "$t" = "--functions" ]; then
        local csvv="${T[$((i+1))]}"
        IFS=',' read -r -a parts <<<"$csvv"
        for name in "${parts[@]}"; do
          nfn=$((nfn+1))
          case "$name" in
            *[[:space:]]*) bad "$label: --functions name '$name' contains a space (the parser splits on ',' and trims; a space-named function matches nothing)"; return ;;
          esac
          if [ -z "$name" ]; then bad "$label: empty field in --functions CSV '$csvv'"; return; fi
          if ! printf '%s' "$name" | grep -qE '^[a-z0-9_]+$'; then bad "$label: --functions name '$name' is outside ^[a-z0-9_]+$"; return; fi
          fns="$fns$name "
        done
        if printf '%s' "$csvv" | grep -q ' ,\|, '; then bad "$label: --functions CSV '$csvv' has spaces around a comma"; return; fi
      fi
      i=$((i+2)); continue ;;
    esac
    case " $FLAGS_ZERO " in *" $t "*) i=$((i+1)); continue ;; esac
    case "$t" in
      --*) bad_tok="$t"; break ;;
      *) bad "$label: bare non-flag argument '$t' before any flag was consumed"; return ;;
    esac
  done
  if [ -n "$bad_tok" ]; then bad "$label: token '$bad_tok' is outside claim_batch's accepted flag set"; return; fi
  [ "$nfn" -eq "$wantn" ] || { bad "$label: $nfn function names, expected $wantn"; return; }
  ok
}
check_argv "R4/a argv" CMD_LEG_R4A "$EXPECT_R4A"
check_argv "R4/b argv" CMD_LEG_R4B "$EXPECT_R4B"
check_argv "R5 argv"   CMD_LEG_R5   "$EXPECT_R5"
check_argv "R6 argv"   CMD_LEG_R6 1
# the source-root triple the grant requires, present exactly three times per leg
for v in CMD_LEG_R4A CMD_LEG_R4B CMD_LEG_R5 CMD_LEG_R6; do
  n=$(tokenize "$v" | grep -c '^--source-root$')
  [ "$n" = 3 ] && ok || bad "$v passes $n --source-root values (want 3: dag, src/v1, src/v2)"
done
for v in CMD_LEG_R4A CMD_LEG_R4B CMD_LEG_R5 CMD_LEG_R6; do
  for want in dag src/v1 src/v2; do
    tokenize "$v" | grep -qxF -- "$want" && ok || bad "$v is missing the source root '$want'"
  done
done
# the entry path per leg must be the frozen entry constant, not a string literal that drifted
entry_of() { # entry_of <var> -> the --entry value of the tokenized command
  local var="$1"
  [ -n "${!var:-}" ] || { printf ''; return; }
  ( BIN="$SENTINEL4"; BIN5="$SENTINEL5"; eval "set -- ${!var}"
    while [ $# -gt 0 ]; do if [ "$1" = "--entry" ]; then printf '%s' "$2"; return; fi; shift; done )
}
for pair in "CMD_LEG_R4A:ENTRY_R4_STAGE0" "CMD_LEG_R4B:ENTRY_R4_CRATE" "CMD_LEG_R5:ENTRY_R5_INBOX" "CMD_LEG_R6:ENTRY"; do
  v="${pair%%:*}"; c="${pair##*:}"
  [ "$(entry_of "$v")" = "${!c}" ] && ok || bad "$v's --entry is '$(entry_of "$v")', not the frozen $c (${!c})"
done

echo "== classify: the bare-verdict rule"
printf 'PASS a_holds\nPASS b_holds\nPASS c_holds\nPASS d_holds\nPASS e_holds\n' > "$SB/leg_pass"
check "classify accepts five bare PASS" 0 bash -c "source '$SB/fn_r4r5.sh'; classify '$SB/leg_pass' 5 >/dev/null"
printf 'FAIL a_holds\nFAIL b_holds\nFAIL c_holds\nFAIL d_holds\nFAIL e_holds\n' > "$SB/leg_fail"
check "classify accepts five bare FAIL (a falsified grant is still an observation)" 0 bash -c "source '$SB/fn_r4r5.sh'; classify '$SB/leg_fail' 5 >/dev/null"
printf 'PASS x\n' > "$SB/leg_one"
check_out "classify accepts the R6 one-claim leg and echoes its verdict" 0 "PASS" bash -c "source '$SB/fn_r6.sh'; classify '$SB/leg_one' 1"
printf 'PASS a\nFAIL b\npanic: index out of bounds\nPASS c\nPASS d\nPASS e\n' > "$SB/leg_noise"
check_out "classify refuses a non-verdict line (exit 2, never a bare FAIL)" 2 "NOT AN OBSERVATION" bash -c "source '$SB/fn_r4r5.sh'; classify '$SB/leg_noise' 5"
printf 'PASS a\nPASS b\nPASS c\nPASS d\n' > "$SB/leg_short"
check_out "classify refuses too few verdicts (INCONCLUSIVE, exit 2)" 2 "INCONCLUSIVE" bash -c "source '$SB/fn_r4r5.sh'; classify '$SB/leg_short' 5"
: > "$SB/leg_empty"
check_out "classify refuses an empty leg" 2 "INCONCLUSIVE" bash -c "source '$SB/fn_r4r5.sh'; classify '$SB/leg_empty' 5"
printf 'PASS a\nPASS b\nPASS c\nPASS d\nPASS e\n  PASS f_indented\n' > "$SB/leg_indent"
check_out "classify refuses a non-bare verdict line among five good ones" 2 "NOT AN OBSERVATION" bash -c "source '$SB/fn_r4r5.sh'; classify '$SB/leg_indent' 5"

echo "== ev_parse / ev_sum: the absent-log hole"
new_case evparse; mkdir -p /tmp/keep 2>/dev/null
RD="$SB/case-evparse/runs/evparse/out"; mkdir -p "$RD/raw"
printf 'PASS a\nPASS b\n' > "$RD/raw/ev-leg_R5_after.raw"
printf 'boundary=leg_R5_before\n' > "$RD/raw/ev-leg_R5_before.raw"
check "ev_parse counts only the boundaries it is given" 0 bash -c "source '$SB/fn_r4r5.sh'; RUN_DIR='$RD'; [ \"\$(ev_parse ev-leg_R5_after.raw 'leg_R5_before')\" = 0 ]"
check "ev_parse counts the PASS lines of a present boundary" 0 bash -c "source '$SB/fn_r4r5.sh'; RUN_DIR='$RD'; [ \"\$(ev_parse ev-leg_R5_after.raw 'leg_R5_after')\" = 2 ]"
check_out "ev_parse refuses an empty log" 2 "EMPTY CLAIM LOG" bash -c "source '$SB/fn_r4r5.sh'; RUN_DIR='$RD'; ev_parse no-such.raw 'start end'"
check_out "ev_parse refuses a missing boundary rather than counting it as 0" 2 "MISSING RAW BOUNDARY" bash -c "source '$SB/fn_r4r5.sh'; RUN_DIR='$RD'; ev_parse ev-leg_R5_after.raw 'start absent_boundary'"
check_out "ev_sum on an absent boundary fails loudly instead of summing 0" 1 "syntax error" bash -c "source '$SB/fn_r4r5.sh'; RUN_DIR='$RD'; ev_sum leg_R5_nosuch >/dev/null"
check "ev_sum over a present boundary counts its PASS lines" 0 bash -c "source '$SB/fn_r4r5.sh'; RUN_DIR='$RD'; [ \"\$(ev_sum leg_R5_after)\" = 2 ]"

echo "== raw_verify: every boundary the packet defines, and none it does not"
new_case rawverify
RD="$SB/case-rawverify/runs/rawverify/out"; mkdir -p "$RD/raw"
for b in start r4_clone_fetch_after r4_build_before r4_build_after leg_R4a_before leg_R4a_after leg_R4b_before leg_R4b_after r5_clone_fetch_after r5_build_before r5_build_after leg_R5_before leg_R5_after; do
  printf 'boundary=%s\n' "$b" > "$RD/raw/ev-$b.raw"
done
check "raw_verify accepts the full R4/R5 boundary set" 0 bash -c "source '$SB/fn_r4r5.sh'; RUN_DIR='$RD'; raw_verify"
rm -f "$RD/raw/ev-r5_clone_fetch_after.raw"
check_out "raw_verify refuses a missing clone boundary" 2 "RAW MEMORY EVIDENCE MISSING" bash -c "source '$SB/fn_r4r5.sh'; RUN_DIR='$RD'; raw_verify"
printf 'x\n' > "$RD/raw/ev-r5_clone_fetch_after.raw"
for extra in mutation r4a_mutation r6_clone_fetch_after leg_R6_before; do
  printf 'x\n' > "$RD/raw/ev-$extra.raw"
  check_out "raw_verify refuses a stowaway boundary ($extra) this packet never defines" 2 "STOWAWAY" bash -c "source '$SB/fn_r4r5.sh'; RUN_DIR='$RD'; raw_verify"
  rm -f "$RD/raw/ev-$extra.raw"
done
check "raw_verify accepts the set again once the stowaways are gone" 0 bash -c "source '$SB/fn_r4r5.sh'; RUN_DIR='$RD'; raw_verify"
RD6="$SB/case-rawverify/runs/rawverify6/out"; mkdir -p "$RD6/raw"
for b in start clone_fetch_after build_before build_after leg_R6_before leg_R6_after; do printf 'x\n' > "$RD6/raw/ev-$b.raw"; done
check "R6 raw_verify accepts the R6 boundary set" 0 bash -c "source '$SB/fn_r6.sh'; RUN_DIR='$RD6'; raw_verify"
printf 'x\n' > "$RD6/raw/ev-mutation.raw"
check_out "R6 raw_verify refuses a mutation boundary it never defines" 2 "STOWAWAY" bash -c "source '$SB/fn_r6.sh'; RUN_DIR='$RD6'; raw_verify"

echo "== scope_ok: this process must be inside the booked <JOB>.scope (end-anchored)"
scope_case() { local name="$1" content="$2" job="$3" want="$4"
  local out; out=$(bash -c "source '$SB/fn_r4r5.sh'; scope_ok $(printf '%q' "$content") '$job'")
  if [ "$want" = OK ] && [ "$out" = OK ]; then ok
  elif [ "$want" != OK ] && [ "$out" != OK ] && printf '%s' "$out" | grep -qF -- "$want"; then ok
  else bad "$name (got '$out', wanted '$want')"; fi; }
scope_case "accepts the booked scope path" "0::/user.slice/user-1000.slice/user@1000.service/app.slice/acceptance-r4r5-srv1.scope" "acceptance-r4r5-srv1" OK
scope_case "refuses the root cgroup" "0::/" "acceptance-r4r5-srv1" "root cgroup"
scope_case "refuses a foreign scope that merely contains the token" "0::/x/foo-acceptance-r4r5-srv1.scope" "acceptance-r4r5-srv1" "does not end in"
scope_case "refuses a token that only matches as a prefix of the segment" "0::/x/acceptance-r4r5-srv1-extra.scope" "acceptance-r4r5-srv1" "does not end in"
scope_case "refuses two cgroup lines" "$(printf '0::/a/x.scope\n1::/b')" "x" "expected exactly 1"
scope_case "refuses a non-0:: line" "1::/x.scope" "x" "does not start with 0::"
scope_case "refuses an empty cgroup file" "" "x" "expected exactly 1"

echo "== require_dir_gate: the launcher's out dir, and only the pinned clones inside it"
new_case dirgate
RD="$SB/case-dircase/runs/dircase/out"
mkdir -p "$RD/wt-r4"
g init -q "$RD/wt-r4"; g -C "$RD/wt-r4" -c user.email=s@selfcheck -c user.name=s commit -q --allow-empty -m seed
HEAD_SHA=$(g -C "$RD/wt-r4" rev-parse HEAD)
DG="source '$SB/fn_r4r5.sh'; RUN_DIR='$RD'; require_dir_gate"
check "dir gate accepts the pinned clone inside the launcher's out dir" 0 bash -c "$DG '$RD/wt-r4' '$HEAD_SHA' 2"
check_out "dir gate refuses a clone whose HEAD drifted off the pin" 3 "HEAD" bash -c "$DG '$RD/wt-r4' 'deadbeefdeadbeefdeadbeefdeadbeefdeadbeef' 2"
mkdir -p "$SB/wt-outside"
check_out "dir gate refuses a clone outside the run dir" 3 "not inside the run dir" bash -c "$DG '$SB/wt-outside' '$HEAD_SHA' 2"
rmdir "$SB/wt-outside"
check_out "dir gate refuses a missing clone" 3 "missing" bash -c "$DG '$RD/wt-nope' '$HEAD_SHA' 2"
mkdir -p "$RD/wt-r9" "$RD/wt-r10"
check_out "dir gate refuses a third clone (no unproven tree joins the run)" 3 "more than 2 pinned clones" bash -c "$DG '$RD/wt-r4' '$HEAD_SHA' 2"
rmdir "$RD/wt-r9" "$RD/wt-r10"
mkdir -p "$SB/case-dircase/runs/dircase/other-out/out"
check_out "dir gate refuses a SECOND out dir under the booked job dir (two runs, one evidence tree)" 3 "more than one out dir" bash -c "$DG '$RD/wt-r4' '$HEAD_SHA' 2"
rm -rf "$SB/case-dircase/runs/dircase/other-out"
check "dir gate accepts the tree again once the duplicate out dir is gone" 0 bash -c "$DG '$RD/wt-r4' '$HEAD_SHA' 2"
# R6 allows exactly one clone
RD6B="$SB/case-dircase/runs/dircase6/out"; mkdir -p "$RD6B/wt-r6" "$RD6B/wt-extra"
g init -q "$RD6B/wt-r6"; g -C "$RD6B/wt-r6" -c user.email=s@selfcheck -c user.name=s commit -q --allow-empty -m seed
HEAD6=$(g -C "$RD6B/wt-r6" rev-parse HEAD)
check_out "R6 dir gate refuses a second clone" 3 "more than 1 pinned clones" bash -c "source '$SB/fn_r6.sh'; RUN_DIR='$RD6B'; require_dir_gate '$RD6B/wt-r6' '$HEAD6' 1"
rmdir "$RD6B/wt-extra"
check "R6 dir gate accepts its one clone" 0 bash -c "source '$SB/fn_r6.sh'; RUN_DIR='$RD6B'; require_dir_gate '$RD6B/wt-r6' '$HEAD6' 1"

echo "== require_scope_gate: the grant's entry files are tracked in the checkout"
mkdir -p "$RD/wt-r4/$(dirname "$ENTRY_R4_STAGE0")" "$RD/wt-r4/$(dirname "$ENTRY_R4_CRATE")"
printf 'module x\n' > "$RD/wt-r4/$ENTRY_R4_STAGE0"
printf 'module y\n' > "$RD/wt-r4/$ENTRY_R4_CRATE"
g -C "$RD/wt-r4" add -A; g -C "$RD/wt-r4" -c user.email=s@selfcheck -c user.name=s commit -q -m entries
check "scope gate accepts both tracked entry paths" 0 bash -c "source '$SB/fn_r4r5.sh'; require_scope_gate '$RD/wt-r4' '$ENTRY_R4_STAGE0' '$ENTRY_R4_CRATE'"
check_out "scope gate refuses when one entry path is untracked" 3 "SCOPE GATE" bash -c "source '$SB/fn_r4r5.sh'; require_scope_gate '$RD/wt-r4' '$ENTRY_R4_STAGE0' 'no/such/entry.dag'"

echo "== require_bound_gate: the bound host, and memory that decided the run"
RD2="$SB/case-rawverify/runs/rawverify/out"   # reuse: this one has a raw/ dir
LOCAL_HOST=$(hostname)
BG="source '$SB/fn_r4r5.sh'; RUN_DIR='$RD2'; BINDING_HOST='$LOCAL_HOST'; require_bound_gate"
check "bound gate accepts the bound host" 0 bash -c "$BG"
check_out "bound gate refuses the wrong host" 3 "BOUND GATE: host is" bash -c "source '$SB/fn_r4r5.sh'; RUN_DIR='$RD2'; BINDING_HOST='not-this-host'; require_bound_gate"
printf 'oom 0\noom_kill 1\nmax 0\nlow 0\nhigh 0\nfail 0\n' > "$RD2/raw/mem.events"
check_out "bound gate refuses when memory.events reports oom_kill" 3 "oom_kill" bash -c "$BG"
printf 'oom 1\noom_kill 0\nmax 1\nlow 0\nhigh 0\nfail 0\n' > "$RD2/raw/mem.events"
check_out "bound gate refuses an unkillable cgroup at a limit breach" 3 "unkillable" bash -c "$BG"
printf 'boundary=start\n### memory.current (/sys/fs/cgroup/memory.current)\n  9000000000\n### memory.max (/sys/fs/cgroup/memory.max)\n  8000000000\n' > "$RD2/raw/ev-start.raw"
printf 'oom 0\noom_kill 0\nmax 3\nlow 0\nhigh 4\nfail 0\n' > "$RD2/raw/mem.events"
check_out "bound gate refuses a limit breach at the current limit" 3 "at its limit" bash -c "$BG"
printf '### memory.current (/sys/fs/cgroup/memory.current)\n  1000000000\n### memory.max (/sys/fs/cgroup/memory.max)\n  8000000000\n' > "$RD2/raw/ev-start.raw"
check "bound gate accepts a limit reported but not reached" 0 bash -c "$BG"
printf '### memory.max (/sys/fs/cgroup/memory.max)\n  max\n' > "$RD2/raw/ev-start.raw"
check "bound gate survives an unlimited cgroup (memory.max is the literal 'max')" 0 bash -c "$BG"
printf 'oom 0\noom_kill 0\nmax 0\nlow 0\nhigh 0\nfail 0\n' > "$RD2/raw/mem.events"
check "bound gate accepts a clean memory.events" 0 bash -c "$BG"
printf 'Memory cgroup out of memory: Killed process 4242 (claim_batch)\n' > "$RD2/raw/ev-leg_R4a_after.oomjournal"
check_out "bound gate refuses an OOM line naming this run's own claim_batch" 3 "MEMORY DECIDED THE RUN" bash -c "$BG"
printf 'Killed process 999 (unrelated_daemon)\n' > "$RD2/raw/ev-leg_R4a_after.oomjournal"
check "bound gate ignores an OOM line about a process this run never started" 0 bash -c "$BG"
rm -f "$RD2/raw/ev-leg_R4a_after.oomjournal"

echo "== require_blob_gate: the bytes at the pinned HEAD"
RDB="$SB/case-rawverify/runs/blobcase/out"; mkdir -p "$RDB/wt-r4/$(dirname "$ENTRY_R4_STAGE0")"
printf 'module x\n' > "$RDB/wt-r4/$ENTRY_R4_STAGE0"
g -C "$RDB/wt-r4" init -q; g -C "$RDB/wt-r4" add -A; g -C "$RDB/wt-r4" -c user.email=s@selfcheck -c user.name=s commit -q -m e
LOCAL_BLOB=$(g -C "$RDB/wt-r4" rev-parse "HEAD:$ENTRY_R4_STAGE0")
check "blob gate accepts the pinned blob" 0 bash -c "source '$SB/fn_r4r5.sh'; RUN_DIR='$RDB'; require_blob_gate '$RDB/wt-r4' '$ENTRY_R4_STAGE0:$LOCAL_BLOB'"
check_out "blob gate refuses a different blob (exit 2, not a verdict)" 2 "BLOB GATE" bash -c "source '$SB/fn_r4r5.sh'; RUN_DIR='$RDB'; require_blob_gate '$RDB/wt-r4' '$ENTRY_R4_STAGE0:0000000000000000000000000000000000000000'"
check_out "blob gate refuses a path absent from the checkout" 2 "BLOB GATE" bash -c "source '$SB/fn_r4r5.sh'; RUN_DIR='$RDB'; require_blob_gate '$RDB/wt-r4' 'no/such.dag:$LOCAL_BLOB'"

echo "== the packets as command lines: the launcher convention, and what refuses before touching anything"
check_out "R4/R5 packet with no argument refuses (usage, exit 2)" 2 "usage:" bash "$PACKET"
check_out "R4/R5 packet with two positional args refuses (usage, exit 2)" 2 "usage:" bash "$PACKET" one two
check_out "R4/R5 packet with an unknown flag refuses (usage, exit 2)" 2 "usage:" bash "$PACKET" --yolo
check_out "R4/R5 refuses a run dir that begins with a dash (never a path)" 2 "usage:" bash "$PACKET" -- /tmp/nope/out
check_out "R4/R5 --predict-only says what it does not do" 0 "PREDICT-ONLY" bash "$PACKET" --predict-only
check "R4/R5 --predict-only creates no run dir" 0 bash -c "cd '$SB' && bash '$PACKET' --predict-only >/dev/null && [ -z \"\$(find '$SB' -maxdepth 3 -name 'out' -newer '$PACKET')\" ]"
# a run dir that does not end in /out is refused (the launcher passes <JOB>/out)
check_out "R4/R5 refuses a run dir that is not a job's out dir" 2 "does not end in /out" bash "$PACKET" "$SB/badcase/wrongname"
check "…and nothing was created under the bad path" 0 bash -c "[ ! -e '$SB/badcase' ]"
# an already-existing out dir is refused with the gate code, before any clone
mkdir -p "$SB/case-dircase/runs/reused/out"
check_out "R4/R5 refuses an already-existing out dir (a run dir is never reused)" 3 "already exists" bash "$PACKET" "$SB/case-dircase/runs/reused/out"
check "…and no clone was started in it" 0 bash -c "[ -z \"\$(find '$SB/case-dircase/runs/reused/out' -mindepth 1)\" ]"
# the execute path stops at the scope gate on a host whose cgroup is not <JOB>.scope: no clone
mkdir -p "$SB/case-dircase/runs/scopefail"
check_out "R4/R5 refuses outside the booked scope before any fetch" 3 "SCOPE GATE" bash "$PACKET" "$SB/case-dircase/runs/scopefail/out"
check "…and the refusal happened before any clone (no wt-* in the run dir)" 0 bash -c "[ ! -e '$SB/case-dircase/runs/scopefail/out/wt-r4' ]"
check_out "held R6 packet with no argument refuses (usage, exit 2)" 2 "usage:" bash "$R6PACKET"
check_out "held R6 --execute with no grant reference refuses (exit 2)" 2 "execution-grant-ref" bash "$R6PACKET" --execute
check_out "held R6 --execute with a grant but no out dir refuses (usage, exit 2)" 2 "usage:" bash "$R6PACKET" --execute some-grant-ref
check_out "held R6 --predict-only states the hold" 0 "the held R6 packet" bash -c "bash '$R6PACKET' --predict-only"
check "held R6 --predict-only creates no run dir" 0 bash -c "cd '$SB' && bash '$R6PACKET' --predict-only >/dev/null && [ -z \"\$(find '$SB' -maxdepth 3 -name 'out' -newer '$R6PACKET')\" ]"
grep -q "msg_3407a188" "$R6PACKET" && ok || bad "the R6 packet does not name the gate it is held behind"
grep -q "R6 REFUSED" "$R6PACKET" && ok || bad "the R6 packet has no refusal branch for an ungranted --execute"
# no mutation leg, no ssh, one attempt, exit-code vocabulary
for f in "$PACKET" "$R6PACKET"; do
  for bad_word in "ssh " "git push" "git commit" "cargo test" "scp " "rsync "; do
    grep -qF -- "$bad_word" "$f" && bad "$(basename "$f") mentions '$bad_word'" || ok
  done
  grep -qE 'for attempt|while true|retry' "$f" && bad "$(basename "$f") retries a leg" || ok
  grep -q 'mkdir -- "\$RUN_DIR"' "$f" && ok || bad "$(basename "$f") does not create its run dir with an exclusive mkdir --"
  grep -q 'mkdir -p "\$RUN_DIR"' "$f" && bad "$(basename "$f") run-dir mkdir is not exclusive" || ok
done
grep -qE '^ROWS_.*mutation' "$PACKET" && bad "a row set names a mutation leg" || ok
grep -q 'leg_R4a_mutation\|leg_R5_mutation\|leg_R6_mutation' "$PACKET" "$R6PACKET" && bad "a mutation boundary is defined in a packet" || ok
for code in "exit 1" "exit 2" "exit 3"; do grep -q -- "$code" "$PACKET" && ok || bad "the R4/R5 packet never uses '$code'"; done
for code in "exit 2" "exit 3"; do grep -q -- "$code" "$R6PACKET" && ok || bad "the R6 packet never uses '$code'"; done
check_ord() { local label="$1" f="$2" a="$3" b="$4" la lb
  la=$(grep -n -F -- "$a" "$f" | head -1 | cut -d: -f1); lb=$(grep -n -F -- "$b" "$f" | head -1 | cut -d: -f1)
  if [ -n "$la" ] && [ -n "$lb" ] && [ "$la" -lt "$lb" ]; then ok; else bad "$label (earlier at ${la:-missing}, later at ${lb:-missing})"; fi; }
check_ord "R4: the fetch is eval'd before the r4_clone_fetch_after capture" "$PACKET" 'eval "$CMD_FETCH_R4"' 'ev_capture r4_clone_fetch_after'
check_ord "R5: the fetch is eval'd before the r5_clone_fetch_after capture" "$PACKET" 'eval "$CMD_FETCH_R5"' 'ev_capture r5_clone_fetch_after'
check_ord "R4: the packet cd's into the clone before the build" "$PACKET" 'cd "$RUN_DIR/wt-r4"' 'CMD_BUILD_R4" >'
check_ord "R5: the packet cd's into the clone before the build" "$PACKET" 'cd "$RUN_DIR/wt-r5"' 'CMD_BUILD_R5" >'
check_ord "R6: the packet cd's into the clone before the build" "$R6PACKET" 'cd "$RUN_DIR/wt-r6"' 'CMD_BUILD" >'
check_ord "R6: the fetch is eval'd before the clone_fetch_after capture" "$R6PACKET" 'eval "$CMD_FETCH"' 'ev_capture clone_fetch_after'
check_ord "the scope gate runs before any clone" "$PACKET" 'scope_ok "$CGRAW"' 'eval "$CMD_CLONE_R4"'
check_ord "…same in the held R6 packet" "$R6PACKET" 'scope_ok "$CGRAW"' 'eval "$CMD_CLONE"'
check_ord "predict-only returns before the run dir exists" "$PACKET" 'exit 0' 'mkdir -- "$RUN_DIR"'
check_ord "…same in the held R6 packet" "$R6PACKET" 'exit 0' 'mkdir -- "$RUN_DIR"'
for pair in "CMD_LEG_R4A:leg_R4a_before" "CMD_LEG_R4B:leg_R4b_before" "CMD_LEG_R5:leg_R5_before"; do
  v="${pair%%:*}"; b="${pair##*:}"
  check_ord "the $b capture precedes its leg's exec" "$PACKET" "ev_capture $b" "bash -c \"\$${v}\""
done
check_ord "leg_R6_before precedes the R6 leg exec" "$R6PACKET" 'ev_capture leg_R6_before' 'bash -c "$CMD_LEG_R6"'

echo "== the command manifest: every command string, its bytes, its sha256, printed by the packet"
MAN_A=$(bash "$PACKET" --predict-only)
n=$(printf '%s\n' "$MAN_A" | grep -c '^cmd '); [ "$n" -eq 11 ] && ok || bad "the R4/R5 manifest prints $n cmd lines (want 11)"
n=$(printf '%s\n' "$MAN_A" | grep -c '^blob '); [ "$n" -eq 3 ] && ok || bad "the R4/R5 manifest prints $n blob lines (want 3)"
printf '%s\n' "$MAN_A" | grep -q "^pin_r4=$PIN_R4$" && ok || bad "the manifest does not print the frozen R4 pin"
printf '%s\n' "$MAN_A" | grep -q "^pin_r5=$PIN_R5$" && ok || bad "the manifest does not print the frozen R5 pin"
printf '%s\n' "$MAN_A" | grep -q "^cmd CMD_LEG_R4A bytes=[0-9]* sha256=[0-9a-f]\{64\}$" && ok || bad "a manifest line lacks bytes+sha256"
d=$(diff <(printf '%s\n' "$MAN_A" | grep -E '^(cmd|blob|pin)') <(cd "$SB" && bash "$PACKET" --predict-only | grep -E '^(cmd|blob|pin)'))
[ -z "$d" ] && ok || bad "the manifest is not reproducible across cwds: $(printf '%s' "$d" | head -3)"
printf '%s\n' "$MAN_A" | grep -A1 '^cmd CMD_LEG_R4A' | tail -1 | grep -qF -- "--functions \"$ROWS_R4A\"" && ok || bad "the printed R4/a command is not the frozen CSV claim set"
printf '%s\n' "$MAN_A" | grep -A1 '^cmd CMD_LEG_R4A' | tail -1 | grep -qF -- "--source-root dag --source-root src/v1 --source-root src/v2" && ok || bad "the printed R4/a command lost the source roots"
printf '%s\n' "$MAN_A" | grep -A1 '^cmd CMD_LEG_R4A' | tail -1 | grep -qF 'claim_batch --source-root' && bad "the printed R4/a command carries the subcommand token v1 got wrong" || ok
printf '%s\n' "$MAN_A" | grep -q '^launch: systemd-run --user --scope --unit=<JOB>' && ok || bad "the manifest does not state the launcher convention"
printf '%s\n' "$MAN_A" | grep -q 'runs/<JOB>/out' && ok || bad "the manifest does not name \$HOME/runs/<JOB>/out"
MAN_R6=$(bash "$R6PACKET" --predict-only)
n=$(printf '%s\n' "$MAN_R6" | grep -c '^cmd '); [ "$n" -eq 5 ] && ok || bad "the R6 manifest prints $n cmd lines (want 5)"
n=$(printf '%s\n' "$MAN_R6" | grep -c '^blob '); [ "$n" -eq 1 ] && ok || bad "the R6 manifest prints $n blob lines (want 1)"
printf '%s\n' "$MAN_R6" | grep -q "^pin=$PIN$" && ok || bad "the R6 manifest does not print its frozen pin"
grep -q 'bash -c "$CMD_LEG_R4A"' "$PACKET" && ok || bad "the R4/a leg does not execute the manifest string"
grep -q 'bash -c "$CMD_LEG_R4B"' "$PACKET" && ok || bad "the R4/b leg does not execute the manifest string"
grep -q 'bash -c "$CMD_LEG_R5"' "$PACKET" && ok || bad "the R5 leg does not execute the manifest string"
grep -q 'bash -c "$CMD_LEG_R6"' "$R6PACKET" && ok || bad "the R6 leg does not execute the manifest string"
grep -q 'bash -c "$CMD_BUILD' "$PACKET" && ok || bad "the builds do not execute the manifest string"
grep -vE '^[[:space:]]*#' "$PACKET" | grep -qF '/opt/gunbc' && bad "the R4/R5 packet still ASSIGNS an instance root" || ok
grep -vE '^[[:space:]]*#' "$R6PACKET" | grep -qF '/opt/gunbc' && bad "the R6 packet still ASSIGNS an instance root" || ok
grep -qE '^[A-Z_]+=("/?opt/gunbc|[^"]*gunbc-workspaces)' "$PACKET" && bad "the R4/R5 packet defines a workspaces root" || ok

echo "== the frozen constants, against local objects and byte-identical working files"
pinned_at() { local id; id=$(g -C "$REPO" rev-parse "$1:$2" 2>/dev/null) || true; printf '%s' "${id:-}"; }
worktree_file_equals_blob() { local h; h=$(g -C "$REPO" hash-object -- "$1" 2>/dev/null) || true; [ -n "$h" ] && [ "$h" = "$2" ]; }
check_name_in_blob() { local label="$1" pin="$2" path="$3" blob="$4" name="$5" id
  id=$(pinned_at "$pin" "$path")
  if [ "$id" != "$blob" ]; then bad "$label: $path is blob ${id:-missing} at $pin, the packet freezes $blob"; return; fi
  if ! worktree_file_equals_blob "$path" "$blob"; then skip "$label: cannot read the pinned bytes of $path offline (partial clone)"; return; fi
  grep -qE "^test fn $name\(" "$REPO/$path" && ok || bad "$label: '$name' is not a test fn in $path at $pin ($blob)"; }
grep_pattern_in_blob() { local label="$1" pin="$2" path="$3" lit="$4" id
  id=$(pinned_at "$pin" "$path")
  if [ -z "$id" ]; then bad "$label: $path is absent at $pin"; return; fi
  if ! worktree_file_equals_blob "$path" "$id"; then skip "$label: $path differs at $pin ($id vs working tree)"; return; fi
  grep -qF -- "$lit" "$REPO/$path" && ok || bad "$label: '$lit' not found in $path at $pin ($id)"; }
if [ -n "$REPO" ] && g -C "$REPO" rev-parse --git-dir >/dev/null 2>&1; then
  for pin in "$PIN_R4" "$PIN_R5"; do
    [ "$(g -C "$REPO" cat-file -t "$pin" 2>/dev/null)" = commit ] && ok || bad "pin $pin is not a commit in $REPO"
  done
  check "PIN_R5 (the #15936 squash) is an ancestor of PIN_R4 (current main)" 0 bash -c "git -C '$REPO' merge-base --is-ancestor '$PIN_R5' '$PIN_R4'"
  ORIGIN_MAIN=$(g -C "$REPO" rev-parse origin/main)
  check "PIN_R4 is an ancestor of (or equal to) origin/main" 0 bash -c "git -C '$REPO' merge-base --is-ancestor '$PIN_R4' origin/main"
  echo "  (origin/main now reads $ORIGIN_MAIN; the packet freezes $PIN_R4)"
  for pair in "$ENTRY_R4_STAGE0:$BLOB_R4_STAGE0" "$ENTRY_R4_CRATE:$BLOB_R4_CRATE" "$ENTRY_R5_INBOX:$BLOB_R5_INBOX" "$ENTRY:$BLOB_LIVE_WITNESS"; do
    p="${pair%%:*}"; want="${pair##*:}"
    [ "$(pinned_at "$ORIGIN_MAIN" "$p")" = "$want" ] && ok || bad "origin/main blob of $p is $(pinned_at "$ORIGIN_MAIN" "$p"), the packet freezes $want"
  done
  [ "$(pinned_at "$PIN_R4" "$ENTRY_R4_STAGE0")" = "$BLOB_R4_STAGE0" ] && ok || bad "PIN_R4 blob of ENTRY_R4_STAGE0 is not the frozen id"
  [ "$(pinned_at "$PIN_R4" "$ENTRY_R4_CRATE")" = "$BLOB_R4_CRATE" ] && ok || bad "PIN_R4 blob of ENTRY_R4_CRATE is not the frozen id"
  [ "$(pinned_at "$PIN_R5" "$ENTRY_R4_STAGE0")" = "$BLOB_R4_STAGE0" ] && ok || bad "the packet claims both pins carry ENTRY_R4_STAGE0 identically; PIN_R5 disagrees"
  [ "$(pinned_at "$PIN_R5" "$ENTRY_R4_CRATE")" = "$BLOB_R4_CRATE" ] && ok || bad "the packet claims both pins carry ENTRY_R4_CRATE identically; PIN_R5 disagrees"
  [ "$(pinned_at "$PIN_R5" "$ENTRY_R5_INBOX")" = "$BLOB_R5_INBOX" ] && ok || bad "PIN_R5 blob of ENTRY_R5_INBOX is not the frozen id"
  [ "$(pinned_at "$PIN" "$ENTRY")" = "$BLOB_LIVE_WITNESS" ] && ok || bad "R6 packet: $ENTRY at $PIN is not the frozen id"
  # every claim name in the packets' CSV, as a `test fn` in the pinned blob
  csv=$(csv_of CMD_LEG_R4A); IFS=',' read -r -a nm <<<"$csv"
  [ "${#nm[@]}" -eq "$EXPECT_R4A" ] || bad "the R4/a name list from the packet is empty or short (${#nm[@]})"
  for n in "${nm[@]}"; do check_name_in_blob "R4/a" "$PIN_R4" "$ENTRY_R4_STAGE0" "$BLOB_R4_STAGE0" "$n"; done
  csv=$(csv_of CMD_LEG_R4B); IFS=',' read -r -a nm <<<"$csv"
  [ "${#nm[@]}" -eq "$EXPECT_R4B" ] || bad "the R4/b name list from the packet is empty or short (${#nm[@]})"
  for n in "${nm[@]}"; do check_name_in_blob "R4/b" "$PIN_R4" "$ENTRY_R4_CRATE" "$BLOB_R4_CRATE" "$n"; done
  csv=$(csv_of CMD_LEG_R5); IFS=',' read -r -a nm <<<"$csv"
  [ "${#nm[@]}" -eq "$EXPECT_R5" ] || bad "the R5 name list from the packet is empty or short (${#nm[@]})"
  for n in "${nm[@]}"; do check_name_in_blob "R5" "$PIN_R5" "$ENTRY_R5_INBOX" "$BLOB_R5_INBOX" "$n"; done
  check_name_in_blob "R6" "$PIN" "$ENTRY" "$BLOB_LIVE_WITNESS" "$CLAIM_R6"
  # THE PARSER, AT THE PIN: the flag set this packet's argv was checked against is the one the pinned
  # binary accepts, and the CSV claim is the pinned parser's own behaviour.
  RBIN=src/v1/stage0/src/bin/claim_batch.rs
  for flag in --source-root --entry --function --functions --claim-run --wet --hermetic --record --eval-budget-ms --fixture-store --pre-push --print-entry-closure --outcomes-out; do
    grep_pattern_in_blob "parser flags" "$PIN_R4" "$RBIN" "\"$flag\" =>"
  done
  grep_pattern_in_blob "parser CSV" "$PIN_R4" "$RBIN" "csv.split(',')"
  grep_pattern_in_blob "parser unknown-arg arm" "$PIN_R4" "$RBIN" "unknown argument"
  grep_pattern_in_blob "parser builds no subcommand" "$PIN_R4" "$RBIN" "let mut i = 1;"
  # the R6 audit's load-bearing lines, each against the bytes AT THE PIN
  grep_pattern_in_blob "R6 audit" "$PIN_R5" "dag/gunbc/fabric/fabric_storage_binding.dag" \
    "FabricStoragePlaced { host: _, instance: _, endpoint, door_socket: _ } => FabricStorageServed {"
  grep_pattern_in_blob "R6 audit" "$PIN_R5" "dag/gunbc/deployment_risk.dag" \
    "data prod_role_selection: ProdRoleSelection = ProdRoleHeldBy { deployment: srv1_daily_workspace_deployment }"
  grep_pattern_in_blob "R6 audit" "$PIN_R5" "dag/gunbc/roadmap/roadmap_dashboard_instance.dag" \
    "fn srv1_live_dashboard_instance() -> HostDashboardInstance {"
  grep_pattern_in_blob "R6 audit" "$PIN_R5" "dag/gunbc/roadmap/roadmap_dashboard_instance.dag" \
    "    deployment: srv1_daily_workspace_deployment,"
  grep_pattern_in_blob "R6 audit" "$PIN_R5" "dag/gunbc/roadmap/roadmap_operator_acceptance.dag" \
    "FabricStorageBindingUnplaced => load_roadmap_acceptance_event_history"
  grep_pattern_in_blob "R6 audit" "$PIN_R5" "dag/gunbc/roadmap/roadmap_operator_acceptance.dag" \
    "fn roadmap_operator_acceptance_history_load(path: String, store: FabricStorageBinding)"
  grep_pattern_in_blob "R6 audit" "$PIN_R5" "dag/gunbc/fabric/fabric_event_log.dag" \
    "fn event_log_read_partition_with<P>(store: FabricStorageBinding, partition: PartitionId, budget: Nat, decode: fn(String) -> EventDecode<P>) -> PartitionRead<P>"
  grep_pattern_in_blob "R6 audit" "$PIN_R5" "dag/gunbc/fabric/fabric_event_log.dag" \
    "match fabric_storage_closure(binding: store, name: partition as String as NonEmptyStr, bound: budget) {"
  grep_pattern_in_blob "R6 audit" "$PIN_R5" "dag/gunbc/fabric/fabric_storage_client.dag" \
    "match served_post(endpoint: e, operation: \"closure\", request: join([name as String, \" \", to_string(bound)], \"\")) {"
  grep_pattern_in_blob "R6 audit" "$PIN_R5" "$ENTRY" \
    "data live_tree_disposition: LiveTreeDisposition = ReadsLiveTree"
else
  echo "  (no repo argument: the constant/object/parser checks are SKIPPED, not passed)"
fi

echo
echo "SELF-TEST: $PASS_N passed, $FAIL_N failed, $SKIP_N skipped"
if [ "$FAIL_N" -ne 0 ]; then printf 'FAILED: %s\n' "${FAILED[@]}"; exit 1; fi
exit 0
