# ---- D3: BOOKING PENDING keen-otter. Every value below is UNBOOKED; the row refuses (2) until each is a number. ----
CAP_FETCH=UNBOOKED      # s, the shallow fetch of the pin
CAP_BUILD=UNBOOKED      # s, the cold seed build
CAP_LABEL=UNBOOKED      # s, the label step (route emit + cold cargo build of the emitted closure + adjudicate)
CAP_COLLECT=UNBOOKED    # s, each collection step (restore, hash), bounded the same way
GRACE=UNBOOKED          # s, timeout -k grace after TERM before KILL, per phase
MEM_MAX=UNBOOKED; MEM_HIGH=UNBOOKED; PIDS_MAX=UNBOOKED; CPU_MAX=UNBOOKED   # cgroup files, exact strings ('_' = space)
RUNTIME_MAX_USEC=UNBOOKED   # the launcher's RuntimeMaxSec, in us as systemctl reports it; must be >= the closed sum
SWAP_MAX=0                  # NOT a sizing choice: gunbc.host.memory_envelope refuses on any swap (DESIGN, memory envelope)
LABEL=//v2/test/claim/parse/expression_bodied_fn_decl_parse:all
BASE=prepare/resolve_reason_declarer_refused
booking_complete() { local v; for v in "$CAP_FETCH" "$CAP_BUILD" "$CAP_LABEL" "$CAP_COLLECT" "$GRACE" "$RUNTIME_MAX_USEC"; do
    [[ "$v" =~ ^[1-9][0-9]*$ ]] || return 1; done
  for v in "$MEM_MAX" "$MEM_HIGH" "$PIDS_MAX"; do [[ "$v" =~ ^[1-9][0-9]*$ ]] || return 1; done
  [[ "$CPU_MAX" =~ ^[1-9][0-9]*_[1-9][0-9]*$ ]]; }
# THE CLOSED SUM (D2): every phase that can take time is under a timeout with a kill grace, and the sum of
#   (cap + grace) over fetch, build, label and three collection steps (cargo tree, restore, hash) is what RuntimeMaxSec
#   must cover; the script checks it.
closed_sum() { echo $(( CAP_FETCH + CAP_BUILD + CAP_LABEL + 3*CAP_COLLECT + 6*GRACE )); }
bounds_check() { # $1=cgroup dir; prints the first mismatch and fails
  local k v got
  for kv in "memory.max=$MEM_MAX" "memory.high=$MEM_HIGH" "memory.swap.max=$SWAP_MAX" "pids.max=$PIDS_MAX" "cpu.max=$CPU_MAX"; do
    k=${kv%%=*}; v=${kv#*=}; v=${v//_/ }
    [ -r "$1/$k" ] || { echo "$k unreadable"; return 1; }
    got=$(cat "$1/$k" 2>/dev/null) || { echo "$k unreadable"; return 1; }
    [ "$got" = "$v" ] || { echo "$k is '$got', booked '$v'"; return 1; }
  done; }
# capped: $1=cap s, rest=command. GNU timeout without --foreground signals its whole process GROUP, so a TERM-ignoring
#   child or grandchild (cargo, rustc, the route's spawned binary) is KILLed GRACE s later. The raw status is returned.
capped() { local c=$1; shift; timeout -k "$GRACE" "$c" "$@"; }
# THE COLD-BUILD ORACLE (D6, fix (b), sleek-owl msg_8df9f22d): it tests the PRECONDITION, not a printed side effect.
#   (1) target_fresh: CARGO_TARGET_DIR does not exist before the build.
#   (2) compiled_set_matches: the DISTINCT "Compiling <name> v<ver>" lines of the build equal the DISTINCT packages of
#       `cargo tree -p v1-compiler -e normal,build --target <host> --prefix none --no-dedupe --locked` at the pin. That is
#       the same package, edge kinds, platform and feature resolution as the build. It is used instead of
#       `cargo metadata`, whose resolve unifies workspace features and lists other platforms' deps, which would make the
#       expected count wrong for this one-package build. Both sides are normalized to "name vver" and deduplicated.
#       Unequal sets (a missing crate means it was not compiled, i.e. reused; an extra one means a different graph)
#       are NO_OBSERVATION.
target_fresh() { ! { [ -e "$1" ] || [ -L "$1" ]; }; }
compiling_set() { grep -E '^\s*Compiling [A-Za-z0-9_-]+ v[0-9]' "$1" | sed -E 's/^\s*Compiling ([A-Za-z0-9_-]+) (v[^ ]+).*/\1 \2/' | sort -u; }
tree_set() { grep -E '^[A-Za-z0-9_-]+ v[0-9]' "$1" | sed -E 's/^([A-Za-z0-9_-]+) (v[^ ]+).*/\1 \2/' | sort -u; }
compiled_set_matches() { [ -n "$(tree_set "$2")" ] && [ "$(compiling_set "$1")" = "$(tree_set "$2")" ]; }
# wrapper_absent $1=build.log $2=inherited wrapper value (may be empty): with -v cargo prints every rustc invocation as a
#   "Running `...`" line; a wrapper would appear in it. Fails (wrapper seen) if any Running line names sccache or the
#   inherited wrapper; also fails if there are NO Running lines, because then -v did not take effect and nothing was seen.
wrapper_absent() { local n; n=$(grep -cE '^\s*Running `' "$1"); [ "$n" -ge 1 ] || return 1
  grep -E '^\s*Running `' "$1" | grep -qE 'sccache' && return 1
  if [ -n "$2" ]; then grep -E '^\s*Running `' "$1" | grep -qF -- "$2" && return 1; fi; return 0; }
# n7_classify $1=rows file (the route's persisted child stdout) $2=stderr $3=rc $4=members out
#   prints "OBSERVED ..." or "NO_OBSERVATION:<why>". THE CONTRACT IT READS, cited: a passing member prints NO
#   [native-verdict] line (std.compiler_entry, the SourceRootEvalDriver annotation; and the EXHAUSTIVE match
#   `NativeTestVerdict::NativeTestPassed => {}` in the adjudicate main rendered by src/v1/05_emit_rust.dag), and EVERY member
#   is a persisted JSON row {identity:{module,declaration}, verdict} in the child's stdout, which the seed does NOT relay:
#   cli_run/native_lane_runner.rs run_native_binary writes it to <workspace>/target/v2-native-lane/driver-rows.jsonl.
#   So "no verdict line" is never counted as a pass by itself: each pass must be a JSON row whose verdict is NativeTestPassed.
n7_classify() { python3 -I - "$1" "$2" "$3" "$4" "$LABEL" <<'PY'
import json, re, sys
rows_p, err_p, rc, mem_p, label = sys.argv[1:6]
def no(why): print("NO_OBSERVATION:" + why); sys.exit(0)
if rc not in ("0", "1", "2"): no("rc=" + rc)
try: rows_txt = open(rows_p, encoding="utf-8").read()
except Exception: no("rows_file_unreadable")
try: err = open(err_p, encoding="utf-8").read().splitlines()
except Exception: no("stderr_unreadable")
terms, members = [], []
for ln in rows_txt.splitlines():
    if not ln.strip(): continue
    try: o = json.loads(ln)
    except Exception: no("rows_line_not_json")
    if not isinstance(o, dict): no("rows_line_not_object")
    if "_terminal" in o: terms.append(o)
    elif set(o) == {"identity", "verdict"}: members.append(o)
    elif set(o) in ({"file_refusal"}, {"accepted_file_advisories"}): pass
    else: no("rows_line_unknown_shape")
if len(terms) != 1: no("terminal_markers=%d" % len(terms))
t = terms[0]
if t.get("_terminal") != "complete" or t.get("mode") != "adjudicate": no("terminal_not_complete_adjudicate")
N = t.get("rows"); TU = t.get("universe")
if type(N) is not int or type(TU) is not int: no("terminal_rows_or_universe_not_int")
cen = [l for l in err if l.startswith("[native-occurrence-census] universe=")]
if len(cen) != 1: no("census_lines=%d" % len(cen))
m = re.match(r"^\[native-occurrence-census\] universe=(\d+) observed=(\d+) ", cen[0])
if not m: no("census_unparsed")
U, O = int(m.group(1)), int(m.group(2))
if U < 1: no("universe_zero")
if O != U: no("observed=%d!=universe=%d" % (O, U))
if N != U or TU != U: no("terminal_rows=%d_universe=%d!=census_universe=%d" % (N, TU, U))
if len(members) != U: no("json_member_rows=%d!=universe=%d" % (len(members), U))
KIND = {"NativeTestPassed": "PASS", "NativeTestReturnedFalse": "RETURNED_FALSE", "NativeTestReturnedOther": "NON_BOOLEAN"}
rowk = {}
for r in members:
    i = r["identity"]
    if not (isinstance(i, dict) and set(i) == {"module", "declaration"} and all(isinstance(i[k], str) and i[k] for k in i)): no("json_identity_shape")
    qid = i["module"] + "." + i["declaration"]
    if qid in rowk: no("duplicate_json_member")
    v = r["verdict"]
    if isinstance(v, str) and v in KIND: k = KIND[v]
    elif isinstance(v, dict) and list(v) == ["NativeTestRefused"]: k = "REFUSED"
    else: no("json_verdict_shape")
    rowk[qid] = k
if sum(1 for k in rowk.values() if k == "PASS") != U - sum(1 for l in err if l.startswith("[native-verdict] ")): no("passed!=universe-verdict_lines")
seen = {}
for l in err:
    if not l.startswith("[native-verdict] "): continue
    rest = l[len("[native-verdict] "):]
    qid, _, tail = rest.partition(" ")
    if not re.match(r"^[A-Za-z_][A-Za-z0-9_]*(\.[A-Za-z_][A-Za-z0-9_]*)+$", qid): no("verdict_id_unparsed")
    if tail == "returned false": now, k = "RETURNED_FALSE", "RETURNED_FALSE"
    elif tail == "returned a non-Boolean value": now, k = "NON_BOOLEAN", "NON_BOOLEAN"
    else:
        mm = re.match(r"^refused at (context|prepare|entry|eval) cause=([a-z][a-z0-9_]*) chain:( |$)", tail)
        if not mm: no("verdict_unparsed")
        now, k = mm.group(1) + "/" + mm.group(2), "REFUSED"
    if qid in seen: no("duplicate_verdict_line")
    if qid not in rowk: no("verdict_line_without_json_row")
    if rowk[qid] != k: no("verdict_kind_disagrees_with_json_row")
    seen[qid] = now
for qid, k in rowk.items():
    if k != "PASS" and qid not in seen: no("nonpass_json_row_without_verdict_line")
msg = sum(1 for l in err if ("native test route: " + label + " adjudicated") in l)
if msg != 1: no("route_message=%d" % msg)
with open(mem_p, "w") as f:
    for qid in sorted(rowk): f.write("%s %s\n" % (qid, seen.get(qid, "PASS")))
print("OBSERVED universe=%d rows=%d verdict_lines=%d passed=%d" % (U, N, len(seen), U - len(seen)))
PY
}
if [ "${1:-}" = "--classifier-fixtures" ]; then
  T=$(mktemp -d); okn=0; badn=0
  ck() { if [ "$3" = "$2" ]; then echo "FIXTURE PASS $1"; okn=$((okn+1)); else echo "FIXTURE FAIL $1 expected '$2' got '$3'"; badn=$((badn+1)); fi; }
  M=v2.test.claim.parse.expression_bodied_fn_decl_parse
  jrow() { printf '{"identity":{"module":"%s","declaration":"%s"},"verdict":%s}\n' "$M" "$1" "$2"; }
  mk() { # 3 members: a refused, b passed, c passed
    { jrow a '{"NativeTestRefused":{"stage":"Prepare","diagnostics":[]}}'; jrow b '"NativeTestPassed"'; jrow c '"NativeTestPassed"'
      echo '{"_terminal":"complete","mode":"adjudicate","rows":3,"universe":3,"admitted":false}'; } > $T/rows
    { echo "[native-occurrence-census] universe=3 observed=3 unobserved=0 colliding_modules=0"
      echo "[native-verdict] $M.a refused at prepare cause=resolve_reason_unbound_symbol chain: x"
      echo "native test route: $LABEL adjudicated — a selected test evaluated and did not hold; q — s"; } > $T/err; }
  C() { n7_classify $T/rows $T/err "${1:-1}" $T/mem; }
  mk; ck ok "OBSERVED universe=3 rows=3 verdict_lines=1 passed=2" "$(C)"
  ck ok-members "$M.a prepare/resolve_reason_unbound_symbol|$M.b PASS|$M.c PASS" "$(tr '\n' '|' < $T/mem | sed 's/|$//')"
  mk; ck rc124 NO_OBSERVATION:rc=124 "$(C 124)"; ck rc137 NO_OBSERVATION:rc=137 "$(C 137)"; ck rcmissing NO_OBSERVATION:rc=missing "$(C missing)"
  mk; rm $T/rows; ck rows-absent NO_OBSERVATION:rows_file_unreadable "$(C)"
  mk; sed -i '/_terminal/d' $T/rows; ck no-terminal NO_OBSERVATION:terminal_markers=0 "$(C)"
  mk; sed -i 's/"rows":3/"rows":4/' $T/rows; ck rows-mismatch NO_OBSERVATION:terminal_rows=4_universe=3!=census_universe=3 "$(C)"
  mk; sed -i '/"declaration":"c"/d' $T/rows; ck json-row-missing NO_OBSERVATION:json_member_rows=2!=universe=3 "$(C)"
  mk; sed -i 's/"declaration":"c"},"verdict":"NativeTestPassed"/"declaration":"c"},"verdict":"NativeTestReturnedFalse"/' $T/rows
    ck nonpass-row-no-stderr NO_OBSERVATION:passed!=universe-verdict_lines "$(C)"
  mk; sed -i 's/"declaration":"c"},"verdict":"NativeTestPassed"/"declaration":"c"},"verdict":"NativeTestReturnedFalse"/' $T/rows
    sed -i "s/\[native-verdict\] $M.a refused.*/[native-verdict] $M.b returned false/" $T/err; sed -i 's/"declaration":"b"},"verdict":"NativeTestPassed"/"declaration":"b"},"verdict":"NativeTestReturnedFalse"/' $T/rows
    ck nonpass-row-unmatched NO_OBSERVATION:passed!=universe-verdict_lines "$(C)"
  mk; sed -i 's/"declaration":"b"},"verdict":"NativeTestPassed"/"declaration":"b"},"verdict":"NativeTestReturnedFalse"/' $T/rows
    echo "[native-verdict] $M.c returned false" >> $T/err; ck verdict-for-a-passed-row NO_OBSERVATION:verdict_kind_disagrees_with_json_row "$(C)"
  mk; sed -i 's/"NativeTestPassed"}$/"Weird"}/' $T/rows; ck verdict-shape NO_OBSERVATION:json_verdict_shape "$(C)"
  mk; echo '{"something":1}' >> $T/rows; ck unknown-row NO_OBSERVATION:rows_line_unknown_shape "$(C)"
  mk; echo 'not json' >> $T/rows; ck not-json NO_OBSERVATION:rows_line_not_json "$(C)"
  mk; sed -i 's/universe=3 observed=3/universe=0 observed=0/' $T/err; ck universe0 NO_OBSERVATION:universe_zero "$(C)"
  mk; sed -i 's/observed=3/observed=2/' $T/err; ck unobserved NO_OBSERVATION:observed=2!=universe=3 "$(C)"
  mk; sed -i '/native test route/d' $T/err; ck no-message NO_OBSERVATION:route_message=0 "$(C)"
  mk; echo "[native-verdict] $M.a refused at prepare cause=x chain: y" >> $T/err; ck dup-verdict NO_OBSERVATION:passed!=universe-verdict_lines "$(C)"
  mk; sed -i 's/refused at prepare/refused at resolve/' $T/err; ck bad-stage NO_OBSERVATION:verdict_unparsed "$(C)"
  mk; jrow b '"NativeTestPassed"' >> $T/rows; sed -i 's/"rows":3,"universe":3/"rows":4,"universe":4/' $T/rows; sed -i 's/universe=3 observed=3/universe=4 observed=4/' $T/err
    ck dup-json-row NO_OBSERVATION:duplicate_json_member "$(C)"
  # D1 kill-grace fixtures: a TERM-ignoring child AND grandchild are KILLed by the group signal after GRACE
  GRACE=2; s=$(date +%s); capped 1 bash -c 'trap "" TERM; (trap "" TERM; echo $BASHPID > '"$T"'/gc.pid; exec sleep 60) & sleep 60'; r=$?; e=$(( $(date +%s) - s ))
  ck term-ignore-rc 137 "$r"; ck term-ignore-bounded yes "$([ $e -le 10 ] && echo yes || echo no:$e)"
  sleep 1; ck grandchild-killed gone "$(kill -0 "$(cat $T/gc.pid)" 2>/dev/null && echo alive || echo gone)"
  s=$(date +%s); capped 1 sleep 60; r=$?; ck term-honoured-rc 124 "$r"
  capped 5 bash -c 'exit 3'; ck raw-rc-preserved 3 "$?"
  ( set -C; : > $T/marker ) 2>/dev/null; ( set -C; : > $T/marker ) 2>/dev/null && ck marker-second refused accepted || ck marker-second refused refused
  ck booking-unbooked-refuses refused "$(booking_complete && echo accepted || echo refused)"
  ( CAP_FETCH=1 CAP_BUILD=1 CAP_LABEL=1 CAP_COLLECT=1 GRACE=1 RUNTIME_MAX_USEC=1 MEM_MAX=1 MEM_HIGH=1 PIDS_MAX=1 CPU_MAX=1_1
    booking_complete && echo accepted || echo refused ) > $T/bk; ck booking-complete-accepts accepted "$(cat $T/bk)"
  ck scope-ok /user.slice/a/x.scope "$(scope_gate "0::/user.slice/a/x.scope" x)"
  scope_gate "0::/a/yx.scope" x >/dev/null && ck scope-suffix refused accepted || ck scope-suffix refused refused
  ( MEM_MAX=68719476736 MEM_HIGH=64424509440 PIDS_MAX=4096 CPU_MAX=3200000_100000; mkdir -p $T/cg
    printf '68719476736\n' > $T/cg/memory.max; printf '64424509440\n' > $T/cg/memory.high; printf '0\n' > $T/cg/memory.swap.max
    printf '4096\n' > $T/cg/pids.max; printf '3200000 100000\n' > $T/cg/cpu.max
    bounds_check $T/cg >/dev/null && echo ok || echo refused; printf 'max\n' > $T/cg/memory.max
    bounds_check $T/cg >/dev/null && echo accepted || echo refused ) > $T/bc
  ck bounds-ok ok "$(sed -n 1p $T/bc)"; ck bounds-envelope-absent refused "$(sed -n 2p $T/bc)"
  ck oompair-none none "$(oom_pair_verdict 0/0 0/0)"; ck oompair-oom OOM_KILLED "$(oom_pair_verdict 0/0 0/1)"; ck oompair-fall COUNTER_INCOHERENT "$(oom_pair_verdict 1/0 0/0)"
  printf '   Compiling a v1.0.0\n   Compiling b v2.1.0 (/w/b)\n   Compiling a v1.0.0\n    Finished release\n' > $T/bl
  printf 'a v1.0.0\nb v2.1.0 (/w/b)\na v1.0.0 (*)\n' > $T/tr; compiled_set_matches $T/bl $T/tr && ck set-equal match match || ck set-equal match mismatch
  printf 'a v1.0.0\nb v2.1.0 (/w/b)\nc v0.3.0\n' > $T/tr2; compiled_set_matches $T/bl $T/tr2 && ck count-mismatch-missing mismatch match || ck count-mismatch-missing mismatch mismatch
  printf 'a v1.0.0\n' > $T/tr3; compiled_set_matches $T/bl $T/tr3 && ck count-mismatch-extra mismatch match || ck count-mismatch-extra mismatch mismatch
  : > $T/tr4; compiled_set_matches $T/bl $T/tr4 && ck empty-tree-refused mismatch match || ck empty-tree-refused mismatch mismatch
  mkdir $T/tgt; target_fresh $T/tgt && ck target-preexisting refused accepted || ck target-preexisting refused refused
  ln -s $T/nowhere $T/tgl; target_fresh $T/tgl && ck target-dangling-link refused accepted || ck target-dangling-link refused refused
  target_fresh $T/no-such-target && ck target-absent accepted accepted || ck target-absent accepted refused
  printf '   Compiling a v1\n     Running `/h/.rustup/toolchains/x/bin/rustc --crate-name a`\n' > $T/r1; wrapper_absent $T/r1 "" && ck wrap-none absent absent || ck wrap-none absent seen
  printf '     Running `/usr/bin/sccache /h/bin/rustc --crate-name a`\n' > $T/r2; wrapper_absent $T/r2 "" && ck wrap-sccache seen absent || ck wrap-sccache seen seen
  printf '     Running `/opt/w/mywrap /h/bin/rustc --crate-name a`\n' > $T/r3; wrapper_absent $T/r3 /opt/w/mywrap && ck wrap-inherited seen absent || ck wrap-inherited seen seen
  printf '   Compiling a v1\n' > $T/r4; wrapper_absent $T/r4 "" && ck wrap-no-verbose seen absent || ck wrap-no-verbose seen seen
  echo "FIXTURES pass=$okn fail=$badn"; [ $badn -eq 0 ]; exit $?
fi
export PATH=$HOME/.cargo/bin:$PATH
J=wise-moth-707-n7-bacae349; H=bacae34989ed860df74a852419e5b74b575703b9; HT=0eee4e8493663c0c0766405fd3cb447c6f3c9857
[ $# = 1 ] && [ -n "$1" ] || { echo "NO_OBSERVATION: usage: script.sh <OUT> (one argument)" >&2; exit 2; }
booking_complete || { echo "NO_OBSERVATION: booking incomplete (D3 pending keen-otter)" >&2; exit 2; }
D=$1; R=$(dirname "$D"); W=$D/wt; O=$D/receipt.txt; U=https://github.com/gunb-ai/gunbc
( set -C; : > "$R/one-attempt" ) 2>/dev/null || { echo "NO_OBSERVATION: one-attempt marker $R/one-attempt exists or cannot be made" >&2; exit 2; }
out_absent "$D" || { echo "NO_OBSERVATION: OUT $D already exists" >&2; exit 2; }
mkdir -- "$D" || { echo "NO_OBSERVATION: exclusive mkdir of $D failed" >&2; exit 2; }
MEMDIR=$D/memevents; mkdir -- "$MEMDIR" "$D/rtmp" "$D/tmp" || { echo "NO_OBSERVATION: cannot make OUT subdirs" >&2; exit 2; }
export CARGO_TARGET_DIR=$W/target RUNNER_TEMP=$D/rtmp TMPDIR=$D/tmp
# the workspace root is DISCOVERED from cwd ($W) via git rev-parse (cli_run resolve_process_workspace_root), never a
# spawned override: a pre-set GUNBC_WORKSPACE_ROOT would redirect it, so it is removed
unset GUNBC_WORKSPACE_ROOT
# BUILD WARM/COLD STATE IS OBSERVED, NOT ASSUMED (tidy-ant-251 msg_e3cc9b46). The inherited cargo/sccache environment is
#   RECORDED first (written to OUT/env-inherited.txt and hashed in the receipt), then PINNED: no rustc wrapper (env and
#   config both disabled), no incremental, and the target dir is the fresh one above. The build is then checked COLD from
#   precondition (target_fresh, then compiled_set_matches against cargo tree), and a warm build is NO_OBSERVATION.
env | grep -E '^(RUSTC_WRAPPER|RUSTC_WORKSPACE_WRAPPER|CARGO_[A-Z_]*|SCCACHE_[A-Z_]*|RUSTFLAGS|CARGO_INCREMENTAL)=' | sort > "$D/env-inherited.txt" || :
{ for f in "$HOME/.cargo/config.toml" "$HOME/.cargo/config"; do [ -e "$f" ] && { echo "## $f"; grep -nE 'wrapper|incremental|target-dir|jobs' "$f" || echo "(no wrapper/incremental/target-dir/jobs keys)"; }; done; } > "$D/cargo-config-inherited.txt" 2>&1 || :
IW=${RUSTC_WRAPPER:-${CARGO_BUILD_RUSTC_WRAPPER:-}}
# WRAPPER DISABLING ON DOCUMENTED SEMANTICS ONLY (cargo book, "Environment variables" -> "Environment variables Cargo
#   reads", https://doc.rust-lang.org/cargo/reference/environment-variables.html): RUSTC_WRAPPER "Setting this to the
#   empty string overwrites the config and resets cargo to not use a wrapper"; RUSTC_WORKSPACE_WRAPPER, the same "for
#   workspace members". So both are EXPORTED EMPTY (not unset). CARGO_BUILD_RUSTC_[WORKSPACE_]WRAPPER have NO documented
#   empty or precedence-against-RUSTC_WRAPPER semantics, so they are UNSET, and the build passes NO --config wrapper
#   override: an empty build.rustc-wrapper value is undocumented (config.html #buildrustc-wrapper), and --config takes
#   precedence over environment variables (config.html #command-line-overrides), so an undocumented value there would
#   outrank the documented one. The -v Running lines (wrapper_absent) observe the result.
unset CARGO_BUILD_RUSTC_WRAPPER CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER CARGO_BUILD_JOBS
export RUSTC_WRAPPER= RUSTC_WORKSPACE_WRAPPER= CARGO_INCREMENTAL=0
echo "job=$J host=$(hostname) start=$(date -u +%FT%TZ)" > $O
echo "env_inherited_sha256=$(sha256sum < "$D/env-inherited.txt" | cut -d' ' -f1) cargo_config_sha256=$(sha256sum < "$D/cargo-config-inherited.txt" | cut -d' ' -f1)" >> $O
sed 's/^/  env| /' "$D/env-inherited.txt" "$D/cargo-config-inherited.txt" | cut -c1-300 >> $O
echo "pinned: RUSTC_WRAPPER='' RUSTC_WORKSPACE_WRAPPER='' (documented: disables, overrides config); CARGO_BUILD_RUSTC_*WRAPPER unset; no --config wrapper override; CARGO_INCREMENTAL=0; CARGO_TARGET_DIR=$CARGO_TARGET_DIR; inherited_wrapper='$IW'" >> $O
nox() { echo "$1" >> $O; echo "VERDICT=NO_OBSERVATION reason=$2" >> $O; echo "DONE $(date -u +%FT%TZ)" >> $O; exit 2; }
JOB=$(basename "$R"); [ "$JOB" = "$J" ] || nox "JOB $JOB is not $J" job
CG=$(scope_gate "$(cat /proc/self/cgroup)" "$JOB") || nox "scope gate: $(tr '\n' ' ' < /proc/self/cgroup)" scope
IID0=$(systemctl --user show -p InvocationID --value "$JOB.scope" 2>/dev/null); [[ "$IID0" =~ ^[0-9a-f]{32}$ ]] || nox "InvocationID at start unreadable: '$IID0'" invocation-id
RMU=$(systemctl --user show -p RuntimeMaxUSec --value "$JOB.scope" 2>/dev/null)
[ "$RMU" = "$RUNTIME_MAX_USEC" ] || nox "RuntimeMaxUSec is '$RMU', booked '$RUNTIME_MAX_USEC'" envelope
[ $(( RUNTIME_MAX_USEC / 1000000 )) -ge "$(closed_sum)" ] || nox "RuntimeMaxSec below the closed sum $(closed_sum) s" envelope
echo "invocation_id=$IID0 scope=$CG runtime_max_usec=$RMU closed_sum_s=$(closed_sum)" >> $O
EV=/sys/fs/cgroup$CG/memory.events; [ "$(oom_pair_verdict "$(oom_snap $EV gate-a)" "$(oom_snap $EV gate-b)")" = none ] || nox "memory.events unreadable at $EV" envelope
BM=$(bounds_check /sys/fs/cgroup$CG) || nox "booked bounds (envelope): $BM" envelope
echo "bounds ok" >> $O
tm() { grep -hE 'Maximum resident|Elapsed' "$1" | tr -s ' ' | tr '\n' ' '; }
git init -q $W || nox "init failed" clone
capped "$CAP_FETCH" git -C $W fetch -q --depth 1 "$U" $H; FX=$?; [ $FX = 0 ] || nox "shallow fetch of $H failed (exit $FX; 124/137 = cap/kill)" clone
git -C $W checkout -q --detach FETCH_HEAD || nox "checkout failed" clone
cd $W && [ "$(git rev-parse HEAD)" = $H ] && [ "$(git rev-parse HEAD^{tree})" = $HT ] && tree_clean || nox "tree not clean at $H / $HT" clone
echo "source=$H tree=$HT (clean)" >> $O
target_fresh "$CARGO_TARGET_DIR" || nox "CARGO_TARGET_DIR $CARGO_TARGET_DIR exists before the build (not a fresh target)" build-warm
echo "target_dir_absent_before_build=yes" >> $O
k0=$(oom_snap $EV build-before)
capped "$CAP_BUILD" /usr/bin/time -v cargo build -v --locked --release -j 32 -p v1-compiler --bin gunbc > $D/build.log 2>&1; BX=$?
k1=$(oom_snap $EV build-after); BO=$(oom_pair_verdict $k0 $k1)
echo "== build exit=$BX oom=$BO $(tm $D/build.log)" >> $O
case $BX in 124|137) nox "build hit its cap (exit $BX)" build-timeout;; esac
[ $BX = 0 ] && [ -x target/release/gunbc ] || nox "build exit=$BX or binary missing" build
[ "$BO" = none ] || nox "build oom=$BO" build-oom
HOSTT=$(rustc -vV 2>/dev/null | sed -n 's/^host: //p'); [ -n "$HOSTT" ] || nox "rustc host triple unreadable" build-warm
capped "$CAP_COLLECT" cargo tree -p v1-compiler -e normal,build --target "$HOSTT" --prefix none --no-dedupe --locked --offline > $D/tree.txt 2> $D/tree.err; TX=$?
[ $TX = 0 ] || nox "cargo tree exit=$TX (see tree.err)" build-warm
echo "compiled_distinct=$(compiling_set $D/build.log | grep -c .) expected_distinct=$(tree_set $D/tree.txt | grep -c .) host=$HOSTT" >> $O
compiled_set_matches $D/build.log $D/tree.txt || { diff <(compiling_set $D/build.log) <(tree_set $D/tree.txt) | head -20 | sed 's/^/  set| /' >> $O; nox "compiled package set != cargo tree set (not a full cold build of this graph)" build-warm; }
wrapper_absent $D/build.log "$IW" || nox "a rustc wrapper appears in the -v build log, or no Running line was printed" build-wrapper
echo "wrapper_absent=yes (checked over $(grep -cE '^\s*Running `' $D/build.log) Running lines)" >> $O
BSHA=$(bin_sha target/release/gunbc) || nox "binary hash read failed" binary-hash; echo "binary_sha256=$BSHA" >> $O
cp -- target/release/gunbc "$D/gunbc.bin" && [ "$(bin_sha "$D/gunbc.bin")" = "$BSHA" ] || nox "binary copy failed or differs" binary-hash
ROWS=$W/target/v2-native-lane/driver-rows.jsonl
[ -e "$ROWS" ] && nox "rows file $ROWS exists before the label step" rows-stale
L0=$(oom_snap $EV label-before)
( capped "$CAP_LABEL" /usr/bin/time -v "$D/gunbc.bin" test "$LABEL" > $D/label.out 2> $D/label.err; echo $? > $D/label.rc )
L1=$(oom_snap $EV label-after); LO=$(oom_pair_verdict $L0 $L1)
RC=$(cat $D/label.rc 2>/dev/null || echo missing)
cp -- "$ROWS" "$D/driver-rows.jsonl" 2>/dev/null || : > "$D/driver-rows.jsonl.absent"
echo "== label rc=$RC oom=$LO $(tm $D/label.err)" >> $O
grep -hE '^v2-native-route: |^\[native-occurrence-census\]|^\[native-cost-partition\]|native test route: ' $D/label.out $D/label.err | cut -c1-400 | sed 's/^/  | /' >> $O
CLS=$(n7_classify "$D/driver-rows.jsonl" $D/label.err "$RC" $D/members.txt)
echo "classify: $CLS" >> $O
if [ "${CLS%% *}" = OBSERVED ]; then
  echo "== per member: BASELINE(3bb5cbff retained history) -> NOW" >> $O
  while read -r id now; do
    if [ "$now" = "$BASE" ]; then echo "  SAME    $id $BASE" >> $O; else echo "  CLEARED $id $BASE -> next stop $now" >> $O; fi
  done < $D/members.txt
fi
if capped "$CAP_COLLECT" git checkout -q -- . ; then RESTORED=$(tree_changes) || RESTORED=status-failed; else RESTORED=checkout-failed; fi; echo "restored: $RESTORED changed" >> $O
FS=$(oom_snap $EV final); FO=$(oom_pair_verdict $L1 $FS)
echo "memory.peak=$(cat /sys/fs/cgroup$CG/memory.peak 2>/dev/null || echo unreadable) (printed, not judged) final=$FS" >> $O
if ASHA=$(capped "$CAP_COLLECT" sha256sum -- "$D/gunbc.bin" | cut -d' ' -f1) && [[ "$ASHA" =~ ^[0-9a-f]{64}$ ]]; then
  echo "binary_sha256_after=$ASHA" >> $O; [ "$ASHA" = "$BSHA" ] || RESTORED="$RESTORED+binary-changed"
else RESTORED="$RESTORED+binary-hash-failed"; fi
IID1=$(systemctl --user show -p InvocationID --value "$JOB.scope" 2>/dev/null); echo "invocation_id_end=$IID1" >> $O
if [ "${CLS%% *}" != OBSERVED ]; then VX=NO_OBSERVATION; WHY=${CLS#NO_OBSERVATION:}
elif [ "$LO" != none ] || [ "$FO" != none ]; then VX=NO_OBSERVATION; WHY=oom-$LO-$FO
elif [ "$RESTORED" != 0 ]; then VX=NO_OBSERVATION; WHY=restore-$RESTORED
elif [ "$IID1" != "$IID0" ]; then VX=NO_OBSERVATION; WHY=invocation-id-changed
else VX=OBSERVED; WHY=complete; fi
echo "VERDICT=$VX reason=$WHY" >> $O
echo "DONE $(date -u +%FT%TZ)" >> $O
[ $VX = OBSERVED ] && exit 0 || exit 2
