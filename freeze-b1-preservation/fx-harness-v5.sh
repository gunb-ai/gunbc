#!/bin/bash
# FROZEN wc-verdict diagnostic packet wc-probe-v5 (author sleek-seal-226, #15695 lane; parent draft approval msg_bf5a7d3c).
# PURPOSE: ONE observation — WHICH WcVerdict each of the four wildcard_arm_resolve fixtures actually takes at seed A. The v4 join
#   measured 0/4 BARE FAILs for those claims at BOTH pins (claims-A-wc.log sha db53f7b0, claims-B-wc.log sha cee20bfd); bare verdict
#   lines carry no cause, so the WcVerdict tag (WcResolved | WcAbsent | WcFileRefused | WcRefused{reason,at_atom,sole_chain} |
#   WcContextRefused) is invisible to the logs. This packet runs ONE diagnostic entry that reports those tags through the EXISTING
#   claim_batch typed-exit channel (src/v1/stage0/src/bin/claim_batch.rs:421-428 prints
#   'FAIL <fn> (returned ProcessExit::ExitFailure, code <n>: <reason>)'). NO binary change, NO observer change, NO copied fixture
#   text: the probe entry imports wc_outcomes from the existing claim module, so the inputs are the same bytes by construction.
# SEED: branch session/sleek-seal-226-wc-probe, ONE commit above main-side A. Its ONLY change from f08a6ae4 is
#   src/v2/test/claim/namespace_xl0/wc_verdict_probe.dag (blob bb6ded7e6533df101e3343dac604ba8dffebdcf4). Gated below: parent commit,
#   tree, and a name-only diff of exactly one path. The frozen claim file is gated UNCHANGED at its blob
#   aba6d2be15f1510629d7d8f8d47267e162bf2047 (byte-identical at f08a6ae4 and at the probe commit).
#   A = f08a6ae4e44dc0636eda71f25f0009e97ffde1d1  tree 2549a8e6545468a55fd8de3d0a082049bae19b32 (the v4a leg-A pin)
#   P = bce90e1396d767093aee240c2e4a4623fba64113  tree c911d73665d0549bfcc86398c53c83d2ba78b4bb (A + the one probe file)
# PARSE-DEFECT NOTE (v5, parent msg_ec43981d): the probe .dag parse defect ('wc_verdict_probe.dag:83:12: expected FatArrow,
#   found Arrow') was LATENT in v3 (masked: the missing --functions refusal preceded entry parsing) and EXPOSED in v4 (the
#   entry parse during --functions validation refused it; raw log sha256 ef780ebc6cd9c2a37bf0061db94a62a12183ab7ea89d55d07c20957590227d70).
#   It is FIXED AT THE SEED (six match arms '->' -> '=>'; blob bb6ded7e6533df101e3343dac604ba8dffebdcf4), evidenced by the
#   exhaustive pre-freeze syntax audit vs the pinned parser, PROBE_DAG_AUDIT.md sha256 59402390bfbf5f38d333bd3c106f1798b915866c688acbce9e71ebefb7430ed0
#   (posted as a PR comment). No local gunbc run was used anywhere in that audit.
# FRAME: the v4 frame verbatim — the same hardened v17-lineage scope gate, bound checks, OOM snap/judge with raw log, the verdict
#   classifier, the completion parser (the REAL line verbatim, sha 7ac24cf2... checked), the rc contract, tree_clean, exclusive OUT,
#   blob/roster gates, build caps, and binary sha before/after. Adapted: single leg, single entry, and a probe classifier for the
#   typed-exit line; its fixtures (valid/duplicate/PASS/bare FAIL/other codes/UNOBSERVED/malformed/code-33 boundary) run in the
#   fixtures block BEFORE any fetch, printing 'fixtures: all held'.
# BUDGET (parent msg_bf5a7d3c + HOLD fix msg_e86484b4): ONE fetch capped 600 (depth 2 of the single-parent probe commit; it also
#   carries A, verified locally with git cat-file -e on the parent commit AND its tree, rc checked — there is NO second fetch)
#   + build 2400 + 1 entry 900 = 3900, + 300 margin => RUNTIME_S 4200, driver 4500 (= RUNTIME_S + 300).
#   RUNTIME_S 4200 FITS the installed launcher calmcrab-udrv v4 (855920ac) bound RUNTIME_S 1..7200.
# PRE-REGISTERED READING TABLE (written before any run; never edited after output; the reason STRING is the datum, its PASS/FAIL is not a verdict):
#   exactly one line 'FAIL wc_verdict_probe_reports (returned ProcessExit::ExitFailure, code 3: WCVERIFY ctx=... arm=... field=...
#   binder=... body=...)' is the EXPECTED observation (EXIT3). Per-module field: R=WcResolved, A=WcAbsent, F=WcFileRefused,
#   X{r=<idx|X>,a=<U|?|->,s=<T|F>,al=<digits>}=WcRefused with reason-table index, atom-vs-^wc_undeclared_body, sole_chain, atom
#   lexeme char-count. Readings, per module, FROZEN:
#     ctx=RF on the row (all four collapse)      -> the boundary is INGEST ADMISSION (native_test_context_from_ingest Rejected).
#     all four F                                 -> the boundary is the LANE FILE-REFUSAL policy.
#     any module R while its claim FAILed        -> the PRODUCER agrees with the claim; the boundary is the claim's own wiring/shape.
#     a module X with r/a not matching the claim's expectation -> the PRODUCER REFUSES at a boundary the claim does not name; r/a/al
#                                                   localize it (r=0 & a=? & al=1 is '_' at the arm/field/binder fixtures).
#     MIXED tags across modules                  -> read each module by its own row above; no single-boundary conclusion follows.
#   code 4 (WCVERIFY BOUND_VIOLATED len=...) = the probe's stated length bound was violated: the reading is INVALID, no conclusion.
#   PASS / bare FAIL / code 1 / NotBool / malformed outcome lines = OBSERVED-BUT-UNEXPECTED (exit 1, reported, never interpreted).
#   UNOBSERVED (code 2), or zero/duplicate outcome lines = SETUP REFUSAL (exit 2, no observation).
# EXIT CONTRACT (v3, per msg_e86484b4): 0 = the single EXIT3 line observed, its reason passes the anchored grammar/length check,
#   the batch rc is consistent (rc 1 for the FAIL-class typed exit), and all frame gates are clean (the observation is VALID; the
#   READING is offline against the frozen table). 2 = NO OBSERVATION for EVERYTHING ELSE: any setup, gate, fetch, build, timeout,
#   OOM rise, counter incoherence, out-of-contract status, missing completion line, zero/duplicate/unclassifiable outcome line,
#   blob/tree/parent/tree-of-parent mismatch, an invalid reading (code 4), or any no-WCVERIFY event (PASS, bare FAIL, code 1,
#   NotBool, malformed, UNOBSERVED) — those are recorded in probe-nonreading.txt as EVENTS, and NO domain result is inferred from
#   them. 1 is unused. Terminal lines: 'PROBE_MISSING=0|1 FAILS=<n>' then 'DONE'; no DONE means incomplete = NO OBSERVATION.
# MEASUREMENT ONLY: no acceptance follows from exit 0; this packet judges no claim and changes no frozen expectation from the v4
#   join (v4a 265609d4..., v4b 8a2bd47e...), which stand as measured.
set -uo pipefail
CGSHIM="$(mktemp -d)"
printf '34359738368\n' > "$CGSHIM/memory.max"; printf '30064771072\n' > "$CGSHIM/memory.high"
printf '0\n' > "$CGSHIM/memory.swap.max"; printf '0\n' > "$CGSHIM/memory.swap.current"
printf '4096\n' > "$CGSHIM/pids.max"; printf '3200000 100000\n' > "$CGSHIM/cpu.max"
printf '0\n' > "$CGSHIM/memory.peak"; printf 'oom 0\noom_kill 0\n' > "$CGSHIM/memory.events"
JOB=sleek-seal-226-wc-probe-v5
UNIT="$JOB.scope"
PIN_A=f08a6ae4e44dc0636eda71f25f0009e97ffde1d1; TREE_A=2549a8e6545468a55fd8de3d0a082049bae19b32
PIN_P=bce90e1396d767093aee240c2e4a4623fba64113; TREE_P=c911d73665d0549bfcc86398c53c83d2ba78b4bb; BLOB_PROBE=bb6ded7e6533df101e3343dac604ba8dffebdcf4; BLOB_WC=aba6d2be15f1510629d7d8f8d47267e162bf2047
die2() { echo "NO OBSERVATION (exit 2): $*"; exit 2; }
# F3 scope gate: exactly one cgroup record, matched against a LITERAL suffix with a case glob (the quoted part is literal; no regex, no unescaped dot)
scope_ok() { case "$1" in 0::*/"$UNIT") return 0;; *) return 1;; esac; }
printf '0::/%s\n' "$UNIT" > "$CGSHIM/cgroup"; [ "$(grep -c '' "$CGSHIM/cgroup")" = 1 ] && scope_ok "$(cat "$CGSHIM/cgroup")" || die2 "REFUSED_SETUP: /proc/self/cgroup is not exactly one 0:: line ending in /$UNIT"
CG="$CGSHIM"
# PRE-WORK ACTUAL-BOUND CHECK (before any fetch): every bound read from the cgroup itself and compared to the booked value; mismatch or unreadable = exit 2
want() { local f="$CG/$1" v; v="$(cat "$f" 2>/dev/null)" || die2 "REFUSED_SETUP: $f unreadable"; [ "$v" = "$2" ] || die2 "REFUSED_SETUP: $1 is [$v], booked [$2]"; }
want memory.max 34359738368
want memory.high 30064771072
want memory.swap.max 0
want pids.max 4096
want cpu.max "3200000 100000"
oomread() { # CONTENT KEY -> prints digits, or returns 1
  local n m; n=$(printf '%s\n' "$1" | awk -v k="$2" '$1==k{c++} END{print c+0}'); m=$(printf '%s\n' "$1" | grep -acxE "$2 [0-9]+")
  [ "$n" = 1 ] && [ "$m" = 1 ] || return 1
  printf '%s\n' "$1" | grep -axE "$2 [0-9]+" | cut -d' ' -f2
}
mem() { echo "mem[$1] peak=$(cat "$CG/memory.peak" 2>&1) events=$(grep -aE '^oom' "$CG/memory.events" 2>&1 | tr '\n' ',') swap=$(cat "$CG/memory.swap.current" 2>&1)"; }
rawlog() { # TAG CONTENT: the RAW memory.events text behind a boundary decision, timestamped, appended under OUT ($R/oom-raw.log) so every OOM verdict re-derives from raw bytes
  printf '%s %s\n%s\n=====\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "$1" "$2" >> "$R/oom-raw.log" || die2 "cannot write $R/oom-raw.log"; }
snap() { local E; E="$(cat "$CG/memory.events" 2>/dev/null)" || die2 "REFUSED_SETUP: memory.events unreadable"
  rawlog "snap-before:${1:-?}" "$E"
  O0=$(oomread "$E" oom) && K0=$(oomread "$E" oom_kill) || die2 "REFUSED_SETUP: memory.events oom/oom_kill not each exactly one strict '<key> <digits>' line"; }
judge_oom() { # a RISE is OOM; a FALL cannot happen within one cgroup's life, so it is an incoherent counter -- both exit 2, labelled apart
  local E O1 K1; E="$(cat "$CG/memory.events" 2>/dev/null)" || die2 "counter-incoherent: memory.events unreadable after $1"
  rawlog "judge-after:$1" "$E"
  O1=$(oomread "$E" oom) && K1=$(oomread "$E" oom_kill) || die2 "counter-incoherent: memory.events oom/oom_kill not strict after $1"
  { [ "$O1" -ge "$O0" ] && [ "$K1" -ge "$K0" ]; } || die2 "counter-incoherent (a FALL, not OOM) during $1: oom $O0->$O1, oom_kill $K0->$K1"
  { [ "$O1" -eq "$O0" ] && [ "$K1" -eq "$K0" ]; } || die2 "OOM (a RISE) during $1: oom $O0->$O1, oom_kill $K0->$K1"
}
# classify LINE NAME -> PASS|FAIL|NONE. ONLY the BARE forms are verdicts: "PASS <fn>" and "FAIL <fn>" (ClaimOutcome::Fail = a Bool-false claim).
# report_outcome prints the REASONED form "FAIL <fn> (<reason>)" for NON-claim outcomes (not-Bool return, runtime error, host tool unresolved, not-attempted, ExitFailure != 2),
# so a reasoned FAIL, "UNOBSERVED <fn> (...)" and "PANICKED <fn> (...)" are all NONE (exit 2, the line is printed in the leg's verdict lines). Exact on the name after stripping trailing whitespace.
classify() { local l; l="$(printf '%s' "$1" | sed 's/[[:space:]]*$//')"; if [ "$l" = "PASS $2" ]; then echo PASS; elif [ "$l" = "FAIL $2" ]; then echo FAIL; else echo NONE; fi; }
# verdict LOG NAME -> PASS|FAIL|NONE. EXACTLY ONE outcome line (any ALLCAPS keyword) may name the function; zero, a duplicate, a conflict (PASS+PANICKED), or an unknown keyword is NONE.
verdict() {
  local L; L="$(grep -aE "^[A-Z]+ $2( |\$)" "$1")"
  [ -n "$L" ] && [ "$(printf '%s\n' "$L" | grep -ac '')" = 1 ] || { echo NONE; return; }
  classify "$L" "$2"
}
# completion evidence: exactly one '[process-partition] total_ms=<digits>' line (claim_batch run() prints it at its end, Ok path only)
# Completion (keen-otter msg_a9e94d36): count EVERY record starting '[process-partition] total_ms' (the '[process-partition-exit]' line does not start so);
#   exactly one must exist, and it must match '^\[process-partition\] total_ms=[0-9]+( |$)'. A valid line beside a malformed competitor refuses.
completed() { local n m; n=$(grep -ac '^\[process-partition\] total_ms' "$1"); m=$(grep -acE '^\[process-partition\] total_ms=[0-9]+( |$)' "$1"); [ "$n" = 1 ] && [ "$m" = 1 ]; }
# hash_of FILE -> exactly one 64-hex sha256, or return 1: the sha256sum read must SUCCEED and yield one 64-hex token (two FAILED reads must never compare equal as two empty strings)
hash_of() { local o h; o="$(sha256sum -- "$1" 2>/dev/null)" || return 1; h="${o%% *}"; [[ $h =~ ^[0-9a-f]{64}$ ]] || return 1; printf '%s' "$h"; }
# exit-status contract: only 0/1/2 are in contract; 124 (timeout), >128 (signal) or anything else is out of contract
rc_in_contract() { case "$1" in 0|1|2) return 0;; *) return 1;; esac; }
out_reusable() { [ -e "$1" ] || [ -L "$1" ]; }   # true = path is taken in ANY form (dir, file, symlink incl. dangling)
tree_clean() { local o; o="$(git -C "$1" status --porcelain 2>/dev/null)" || return 1; [ -z "$o" ]; }   # a FAILED git status (non-repo, error) is NOT clean: capture, require rc 0, THEN test emptiness
# ---- FIXTURE self-checks (a focused failing input for each defect class), before any fetch ----
# The REAL completion line, copied verbatim from calm-crab-645's G1 cpr log (sha a0e1f1f6), with sha256 checked below.
REAL_COMPLETION='[process-partition] total_ms=340188 pre_run_ms=0 args_ms=22 index_build_ms=154362 edge_index_warm_ms=35617 output_policy_ms=12509 roster_ms=0 published_mock_precompute_ms=1761 entries_ms=135911 reporting_ms=2 entries_resolve_ms=113955 entries_witness_ms=20622'
[ "$(printf %s "$REAL_COMPLETION" | sha256sum | cut -d' ' -f1)" = "7ac24cf28973511b7902ce3ba621e8c46b060ad7855ea895b1f5a157bf1835b2" ] || die2 "REFUSED_SETUP: the real-line fixture is not the verbatim bytes"
FX="$(mktemp -d)" || die2 "mktemp"
r() { if "$@"; then echo accepted; else echo refused; fi; }
# (1) parser
r_ev() { oomread "$1" oom >/dev/null && oomread "$1" oom_kill >/dev/null; }
[ "$(r r_ev $'oom 0\noom_kill 0')" = accepted ] || die2 "REFUSED_SETUP: fixture: valid-only events refused"
[ "$(r r_ev $'oom 0 garbage\noom_kill 0')" = refused ] || die2 "REFUSED_SETUP: fixture: extra field accepted"
[ "$(r r_ev $'oom 0\noom_kill 0\noom bad')" = refused ] || die2 "REFUSED_SETUP: fixture: malformed duplicate accepted"
[ "$(r r_ev $'oom 0\noom 0\noom_kill 0')" = refused ] || die2 "REFUSED_SETUP: fixture: valid duplicate accepted"
[ "$(r r_ev $'oom 0\noom_kill 0\noom')" = refused ] || die2 "REFUSED_SETUP: fixture: bare oom line accepted"
[ "$(r r_ev $'oom 0\noom_kill 0\noom_kill')" = refused ] || die2 "REFUSED_SETUP: fixture: bare oom_kill line accepted"
[ "$(r r_ev $'oom\t0\noom_kill 0')" = refused ] || die2 "REFUSED_SETUP: fixture: tab-separated oom accepted"
[ "$(r r_ev $'oom 0\noom_kill\t0')" = refused ] || die2 "REFUSED_SETUP: fixture: tab-separated oom_kill accepted"
[ "$(r r_ev $'oom\tbad\noom 0\noom_kill 0')" = refused ] || die2 "REFUSED_SETUP: fixture: oom<TAB>bad accepted"
[ "$(r r_ev $'oom_kill 0')" = refused ] || die2 "REFUSED_SETUP: fixture: absent key accepted"
# (2) verdict + completion
printf 'PASS x\nPASS x\n' > "$FX/v1"; printf 'PASS x\nPANICKED x (r)\n' > "$FX/v2"; printf 'PASS x\n' > "$FX/v3"; printf 'WEIRD x\n' > "$FX/v4"; printf 'FAIL x (runtime error [c]: m)\n' > "$FX/v5"; printf 'FAIL x\n' > "$FX/v7"; printf 'PASS xy\n' > "$FX/v6"
[ "$(verdict $FX/v1 x) $(verdict $FX/v2 x) $(verdict $FX/v3 x) $(verdict $FX/v4 x) $(verdict $FX/v5 x) $(verdict $FX/v6 x) $(verdict $FX/v7 x)" = "NONE NONE PASS NONE NONE NONE FAIL" ] || die2 "REFUSED_SETUP: verdict fixture misfire"
printf 'PASS x\n%s\n[process-partition-exit] run_returned_ms=340188 main_end_ms=340188\n' "$REAL_COMPLETION" > "$FX/c1"; printf 'PASS x\n' > "$FX/c2"; printf '[process-partition] total_ms=5\n' > "$FX/c3"; printf '[process-partition] total_ms=\n' > "$FX/c4"; printf '[process-partition] total_ms=abc pre_run_ms=0\n' > "$FX/c5"; printf '[process-partition] total_ms=5 a_ms=1\n[process-partition] total_ms=6 a_ms=1\n' > "$FX/c6"; printf '[process-partition] total_ms=5 a_ms=1\n[process-partition] total_msX\n' > "$FX/c7"; printf '[process-partitions] total_ms=5\n' > "$FX/c8"
[ "$(r completed $FX/c1) $(r completed $FX/c2) $(r completed $FX/c3) $(r completed $FX/c4) $(r completed $FX/c5) $(r completed $FX/c6) $(r completed $FX/c7) $(r completed $FX/c8)" = "accepted refused accepted refused refused refused refused refused" ] || die2 "REFUSED_SETUP: completion fixture misfire (real, absent, bare, missing total, non-numeric, duplicate, valid+malformed, wrong marker)"
# (3) exit contract
[ "$(r rc_in_contract 0) $(r rc_in_contract 1) $(r rc_in_contract 2) $(r rc_in_contract 124) $(r rc_in_contract 137) $(r rc_in_contract 3)" = "accepted accepted accepted refused refused refused" ] || die2 "REFUSED_SETUP: exit-contract fixture misfire"
# (4) scope glob literal suffix, restore cleanliness
[ "$(r scope_ok '0::/a/sleek-seal-226-wc-probe-v5.scope') $(r scope_ok '0::/a/sleek-seal-226-wc-probe-v5Xscope') $(r scope_ok '0::/a/xsleek-seal-226-wc-probe-v5.scope') $(r scope_ok '0::/a/sleek-seal-226-wc-probe-v5.scope.x')" = "accepted refused refused refused" ] || die2 "REFUSED_SETUP: scope fixture misfire"
git init -q "$FX/g" && : > "$FX/g/f" || die2 "fixture git init"
git -C "$FX/g" add f && git -C "$FX/g" -c user.name=fx -c user.email=fx@fx commit -qm fx || die2 "fixture commit"
A="$(r tree_clean $FX/g)"; : > "$FX/g/u"; B="$(r tree_clean $FX/g)"
[ "$A $B" = "accepted refused" ] || die2 "REFUSED_SETUP: restore-cleanliness fixture misfire ($A $B)"
# (5) output path reuse in any form
mkdir "$FX/d"; : > "$FX/f"; ln -s "$FX/nowhere" "$FX/dangling"
[ "$(r out_reusable $FX/d) $(r out_reusable $FX/f) $(r out_reusable $FX/dangling) $(r out_reusable $FX/absent)" = "accepted accepted accepted refused" ] || die2 "REFUSED_SETUP: output-path fixture misfire"
mkdir "$FX/nonrepo"; [ "$(r tree_clean $FX/nonrepo)" = refused ] || die2 "REFUSED_SETUP: fixture: tree_clean accepted a non-repo dir (failed git status read as clean)"
printf x > "$FX/hf"; H1="$(hash_of "$FX/hf")"; [ "${#H1}" = 64 ] || die2 "REFUSED_SETUP: fixture: hash_of did not yield 64-hex"
if hash_of "$FX/missing" >/dev/null; then die2 "REFUSED_SETUP: fixture: hash_of accepted a missing file"; fi

# ---- PROBE CLASSIFIER (adapted; the v4 classifier above stays verbatim and unused for this entry) ----
# Probe function name (the single witness):
PFN=wc_verdict_probe_reports
# probe_classify LINE NAME -> PASS|BARE|EXIT3|EXIT4|UNOBSERVED|OTHER. The typed-exit forms are claim_batch.rs:409-431 verbatim:
#   'FAIL <fn> (returned ProcessExit::ExitFailure, code <n>: <reason>)' for n not 2; 'UNOBSERVED <fn> (returned ProcessExit::ExitFailure, code 2...)'.
# 'code 33' must not match 'code 3': the prefix test requires 'code 3: ' exactly (space-colon-space), and the suffix ')' must close the line.
PROBE_REASON=""
PROBE_TOKEN=NONE
probe_classify() {
  local l; l="$(printf '%s' "$1" | sed 's/[[:space:]]*$//')"
  local P="FAIL $2 (returned ProcessExit::ExitFailure, code 3: "
  local Q="FAIL $2 (returned ProcessExit::ExitFailure, code 4: "
  if [ "$l" = "PASS $2" ]; then PROBE_TOKEN=PASS; echo PASS; return; fi
  if [ "$l" = "FAIL $2" ]; then PROBE_TOKEN=BARE; echo BARE; return; fi
  case "$l" in
    "$P"*) if [ "${l: -1}" = ")" ]; then PROBE_REASON="${l#"$P"}"; PROBE_REASON="${PROBE_REASON%?}"; PROBE_TOKEN=EXIT3; echo EXIT3; else PROBE_TOKEN=OTHER; echo OTHER; fi; return;;
    "$Q"*) PROBE_TOKEN=EXIT4; echo EXIT4; return;;
  esac
  case "$l" in
    "UNOBSERVED $2 "*) PROBE_TOKEN=UNOBSERVED; echo UNOBSERVED; return;;
    "FAIL $2 "* | "PANICKED $2 "*) PROBE_TOKEN=OTHER; echo OTHER; return;;
  esac
  PROBE_TOKEN=NONE; echo NONE
}
# probe_verdict LOG NAME -> token; the SINGLE-outcome-line rule mirrors v4 verdict(): zero, a duplicate, or a conflict is NONE.
# The declared grammar, ANCHORED (parent byte review, msg_fdb10bd9). Two shapes only:
#   RF row (fixed literal, the 'other fields absent' shape is NOT allowed): WCVERIFY ctx=RF arm=- field=- binder=- body=-
#   OK row: WCVERIFY ctx=OK arm=<v> field=<v> binder=<v> body=<v>, each <v> exactly:
#     R | A | F | X{r=<0..23|X>,a=<U|?|->,s=<T|F>,al=<1..19 digits>}
#   r digits are the frozen table indices as plain decimal (0|[1-9]|1[0-9]|2[0-3]); al is a plain
#   decimal count of at most 19 digits (the i64 bound of int_to_decimal_string).
PG_V='(R|A|F|X\{r=(0|[1-9]|1[0-9]|2[0-3]),a=(U|\?|-),s=(T|F),al=[0-9]{1,19}\})'
PG_OK="^WCVERIFY ctx=OK arm=${PG_V} field=${PG_V} binder=${PG_V} body=${PG_V}$"
PG_RF='^WCVERIFY ctx=RF arm=- field=- binder=- body=-$'
# probe_validate REASON -> 0 accepted / 1 refused. Length first, then the anchored grammar.
probe_validate() {
  local r="$1"
  [ "${#r}" -le 193 ] || return 1
  if [[ "$r" =~ $PG_RF ]]; then return 0; fi
  if [[ "$r" =~ $PG_OK ]]; then return 0; fi
  return 1
}
PROBE_TOKEN=NONE
probe_verdict() {
  local L n; L="$(grep -aE "^[A-Z]+ $2( |\$)" "$1")"
  if [ -z "$L" ]; then PROBE_TOKEN=NONE; echo NONE; return; fi
  n="$(printf '%s\n' "$L" | grep -ac '')"
  if [ "$n" != 1 ]; then PROBE_TOKEN=NONE; echo NONE; return; fi
  probe_classify "$L" "$2" >/dev/null; echo "$PROBE_TOKEN"
}

# (7) probe classifier fixtures (a focused failing input for each misread class), before any fetch
printf 'FAIL %s (returned ProcessExit::ExitFailure, code 3: WCVERIFY ctx=OK arm=R field=R binder=R body=X{r=0,a=?,s=T,al=1})\n' "$PFN" > "$FX/p1"
printf 'PASS %s\nFAIL %s (returned ProcessExit::ExitFailure, code 3: WCVERIFY ctx=OK arm=R field=R binder=R body=X{r=0,a=?,s=T,al=1})\n' "$PFN" "$PFN" > "$FX/p2"
printf 'PASS %s\n' "$PFN" > "$FX/p3"
printf 'FAIL %s\n' "$PFN" > "$FX/p4"
printf 'FAIL %s (returned ProcessExit::ExitFailure, code 1: boom)\n' "$PFN" > "$FX/p5"
printf 'UNOBSERVED %s (returned ProcessExit::ExitFailure, code 2: nothing observed)\n' "$PFN" > "$FX/p6"
printf 'FAIL %s (returned ProcessExit::ExitFailure, code 3: WCVERIFY ctx=RF arm=- field=- binder=- body=-\n' "$PFN" > "$FX/p7"
printf 'FAIL %s (returned ProcessExit::ExitFailure, code 33: WCVERIFY ctx=OK arm=R field=R binder=R body=R)\n' "$PFN" > "$FX/p8"
printf 'FAIL %s (returned `Int`, not Bool; --claim-run entries must return Bool)\n' "$PFN" > "$FX/p9"
printf 'FAIL %s (returned ProcessExit::ExitFailure, code 4: WCVERIFY BOUND_VIOLATED len=140)\n' "$PFN" > "$FX/p10"
[ "$(probe_verdict $FX/p1 $PFN) $(probe_verdict $FX/p2 $PFN) $(probe_verdict $FX/p3 $PFN) $(probe_verdict $FX/p4 $PFN) $(probe_verdict $FX/p5 $PFN) $(probe_verdict $FX/p6 $PFN) $(probe_verdict $FX/p7 $PFN) $(probe_verdict $FX/p8 $PFN) $(probe_verdict $FX/p9 $PFN) $(probe_verdict $FX/p10 $PFN)" = "EXIT3 NONE PASS BARE OTHER UNOBSERVED OTHER OTHER OTHER EXIT4" ] || die2 "REFUSED_SETUP: probe-classifier fixture misfire (valid, duplicate, PASS, bare, code1, UNOBSERVED, unclosed, code-33 boundary, NotBool, code-4)"
printf 'FAIL %s (returned ProcessExit::ExitFailure, code 3: WCVERIFY ctx=OK arm=R field=R binder=R body=X{r=0,a=?,s=T,al=1})\n' "$PFN" > "$FX/pv"
probe_verdict "$FX/pv" "$PFN" >/dev/null
[ "$PROBE_REASON" = "WCVERIFY ctx=OK arm=R field=R binder=R body=X{r=0,a=?,s=T,al=1}" ] || die2 "REFUSED_SETUP: probe reason extraction is not verbatim"
# (8) probe bound self-check: the stated maximum, from the frozen layout arithmetic, is within the claim-side bound 193
[ $(( 15 + 5 + 7 + 8 + 6 + 4 * 38 )) -le 193 ] || die2 "REFUSED_SETUP: the stated probe layout exceeds its stated bound"

# (9) grammar fixtures through the ACTUAL path: a full typed line -> probe_verdict -> probe_validate
PGD="$FX"
pv_line() { printf 'FAIL %s (returned ProcessExit::ExitFailure, code 3: %s)\n' "$PFN" "$1" > "$PGD/gline"; probe_verdict "$PGD/gline" "$PFN" >/dev/null; if [ "$PROBE_TOKEN" != EXIT3 ]; then echo 1; return; fi; probe_validate "$PROBE_REASON"; echo "$?"; }
# valid OK row
[ "$(pv_line 'WCVERIFY ctx=OK arm=R field=R binder=R body=X{r=0,a=?,s=T,al=1}')" = 0 ] || die2 "REFUSED_SETUP: grammar fixture (valid OK) refused"
# valid RF row (fixed literal)
[ "$(pv_line 'WCVERIFY ctx=RF arm=- field=- binder=- body=-')" = 0 ] || die2 "REFUSED_SETUP: grammar fixture (valid RF) refused"
# garbage reason
[ "$(pv_line 'hello world')" = 1 ] || die2 "REFUSED_SETUP: grammar fixture (garbage) accepted"
# extra trailing field
[ "$(pv_line 'WCVERIFY ctx=OK arm=R field=R binder=R body=R extra=R')" = 1 ] || die2 "REFUSED_SETUP: grammar fixture (extra field) accepted"
# unknown tag
[ "$(pv_line 'WCVERIFY ctx=OK arm=Q field=R binder=R body=R')" = 1 ] || die2 "REFUSED_SETUP: grammar fixture (unknown tag) accepted"
# unknown reason index
[ "$(pv_line 'WCVERIFY ctx=OK arm=X{r=24,a=U,s=T,al=1} field=R binder=R body=R')" = 1 ] || die2 "REFUSED_SETUP: grammar fixture (reason index 24) accepted"
# unsorted/duplicate-free digit forms: leading zero rejected
[ "$(pv_line 'WCVERIFY ctx=OK arm=X{r=07,a=U,s=T,al=01} field=R binder=R body=R')" = 1 ] || die2 "REFUSED_SETUP: grammar fixture (leading zero) accepted"
# RF with the other fields absent is NOT the declared shape
[ "$(pv_line 'WCVERIFY ctx=RF')" = 1 ] || die2 "REFUSED_SETUP: grammar fixture (bare RF) accepted"
# nested parens inside the reason cannot smuggle through: the ')' is positional
[ "$(pv_line 'WCVERIFY (x) ctx=OK arm=R field=R binder=R body=R')" = 1 ] || die2 "REFUSED_SETUP: grammar fixture (nested paren) accepted"
# valid MAXIMUM length: 193 chars, every field at its widest
MX='X{r=23,a=?,s=F,al=1234567890123456789}'
MAXROW="WCVERIFY ctx=OK arm=${MX} field=${MX} binder=${MX} body=${MX}"
[ "${#MAXROW}" = 193 ] || die2 "REFUSED_SETUP: the constructed maximum row is not 193 chars (${#MAXROW})"
[ "$(pv_line "$MAXROW")" = 0 ] || die2 "REFUSED_SETUP: grammar fixture (valid maximum 193) refused"
# length 194 through the length check directly (a grammatical 194 cannot exist; the bound is belt-and-braces)
probe_validate "${MAXROW}X" && die2 "REFUSED_SETUP: a 194-char reason passed the length bound"
probe_validate "$(printf 'W%.0s' {1..193})X" && die2 "REFUSED_SETUP: a 194-char garbage reason passed the length bound"
# a 20-digit al is refused by the grammar (not just by length)
[ "$(pv_line "WCVERIFY ctx=OK arm=X{r=1,a=U,s=T,al=12345678901234567890} field=R binder=R body=R")" = 1 ] || die2 "REFUSED_SETUP: grammar fixture (20-digit al) accepted"
rm -rf "$FX"
echo "fixtures: all held"
# ---- arg check + exclusive OUT (v17 semantics, verbatim) ----
# B1: the OUTPUT dir is the one argument (the launcher maps it to <R>/out). It must be ABSENT in every form: no dir (even empty), file or symlink (dangling included).
[ "$#" = 1 ] && [ -n "$1" ] || die2 "REFUSED_SETUP: usage: $0 <output-dir>"
if out_reusable "$1"; then die2 "REFUSED_SETUP: output path $1 already exists (dir, file or symlink); reuse is refused"; fi
mkdir -- "$1" || die2 "mkdir $1 (exclusive, no -p: an entry that appeared since the absence check is refused)"
cd "$1" || die2 "cd $1"
R="$PWD"

# ---- ENTRY: the single diagnostic file on the probe commit (blob gated after checkout) ----
PROBE_PATH="src/v2/test/claim/namespace_xl0/wc_verdict_probe.dag"
PFN=wc_verdict_probe_reports

# ---- PRE-REGISTERED EXPECTATION (intended observation, written 2026-11-06 BEFORE this packet ever ran; never edited after output) ----
# The witness is a MEASUREMENT INSTRUMENT: it always returns ProcessExit code 3, so exactly one EXIT3 line is pre-registered.
# Its content is read OFFLINE against the frozen reading table in the header; PASS/bare-FAIL/other-code outcomes are setup refusals.
declare -A EXPECT
EXPECT["A.probe.$PFN"]=EXIT3

declare -A OBS
MISSING=0
DEVIATIONS=0

probe_run() { # BIN : one claim_batch invocation of the probe entry; frame contracts identical to v4 run_entry
  local BIN="$1"
  local L="$R/claims-A-probe.log" RC VE VD
  snap "run A probe"
  timeout 900 "$BIN" --claim-run --source-root dag --source-root src/v1 --source-root src/v2 --entry "$PROBE_PATH" --functions "$PFN" > "$L" 2>&1
  RC=$?; echo "claim_batch_exit[A probe]=$RC"; mem "after_run_A-probe"; judge_oom "run A probe"
  rc_in_contract "$RC" || die2 "leg A probe: exit status $RC is out of contract (0/1/2 only; 124 timeout, >128 signal)"
  completed "$L" || die2 "leg A probe: no single '[process-partition] total_ms=<digits>' completion line; batch incomplete whatever verdict lines printed"
  echo "verdict lines (A probe):"; grep -aE "^(PASS|FAIL|UNOBSERVED|PANICKED) " "$L"
  probe_verdict "$L" "$PFN" > "$R/.probe.vd" || true; VD="$(cat "$R/.probe.vd")"; echo "A probe $PFN=$VD"
  [ "$PROBE_TOKEN" = "$VD" ] || die2 "leg A probe: classifier token disagreement ($VD vs $PROBE_TOKEN)"
  if [ "$VD" = NONE ]; then MISSING=1; die2 "leg A probe: '$PFN' has no SINGLE outcome line (absent, duplicate, or unclassifiable)"; fi
  if [ "$VD" = EXIT3 ]; then
    RAW3="$(grep -aE "^FAIL $PFN \(returned ProcessExit::ExitFailure, code 3: " "$L")"
    echo "RAW: $RAW3"
    probe_validate "$PROBE_REASON" || die2 "leg A probe: NO_OBSERVATION: the reason fails the anchored grammar or the 193-char bound; raw line preserved in claims-A-probe.log and here: $RAW3"
    printf '%s\n' "$PROBE_REASON" > "$R/probe-reason.txt" || die2 "cannot write probe-reason.txt"
    echo "PROBE_REASON: $PROBE_REASON"
    echo "PROBE_REASON_BYTES=${#PROBE_REASON}"
    VE=1
  else
    # ANY non-EXIT3 token (EXIT4 invalid reading, UNOBSERVED, PASS, BARE, OTHER incl. code 1/NotBool/malformed):
    # there is NO WCVERIFY observation. Recorded as an event, then NO OBSERVATION (exit 2). No domain result is inferred.
    { echo "token=$VD claim_batch_rc=$RC"; grep -aE "^(PASS|FAIL|UNOBSERVED|PANICKED) " "$L"; } > "$R/probe-nonreading.txt" 2>&1
    die2 "leg A probe: NO WCVERIFY READING (token $VD, claim_batch rc $RC); raw lines preserved in claims-A-probe.log and probe-nonreading.txt; no domain result follows"
  fi
  # the only survivors are validated EXIT3 rows: the batch rc MUST be consistent with a FAIL-class typed exit (claim_batch.rs sets any_failed)
  [ "$RC" = 1 ] || die2 "leg A probe: claim_batch returned $RC, but a typed code-3 FAIL class requires rc 1"
  OBS["A.probe.$PFN"]="EXIT3"
}

probe_leg() { # fetch (600), gates (parent/delta/blobs), build (2400, own target dir), one entry (900), post-checks
  local HT BIN BIN_SHA HB
  git init -q "wt-A" && cd "wt-A" || die2 "git init wt-A"
  timeout 600 git fetch -q --depth 2 https://github.com/gunb-ai/gunbc "$PIN_P" || die2 "leg A: fetch $PIN_P failed or timed out (600 s cap)"
  git checkout -q FETCH_HEAD || die2 "leg A: checkout failed"
  [ "$(git rev-parse HEAD)" = "$PIN_P" ] || die2 "leg A: HEAD is not the probe pin"
  HT="$(git rev-parse 'HEAD^{tree}')" || die2 "leg A: cannot read the tree of HEAD"
  [ "$HT" = "$TREE_P" ] || die2 "leg A: REFUSED_SETUP: tree is $HT, not the pinned $TREE_P"
  tree_clean . || die2 "leg A: REFUSED_SETUP: checkout is not clean"
  # the one-file-delta gates: parent is exactly A, and the name-only diff to A is exactly the probe path.
  # NO second fetch: the depth-2 fetch of the single-parent probe commit already carries A. The parent commit and
  # ITS TREE must be present locally; fail closed (exit 2) if not.
  [ "$(git rev-parse 'HEAD^1')" = "$PIN_A" ] || die2 "leg A: REFUSED_SETUP: the probe commit's parent is not the A pin $PIN_A"
  git cat-file -e "$PIN_A^{commit}" 2>/dev/null || die2 "leg A: REFUSED_SETUP: parent commit $PIN_A is absent locally (the single depth-2 fetch must provide it)"
  git cat-file -e "$PIN_A^{tree}"   2>/dev/null || die2 "leg A: REFUSED_SETUP: the tree of parent $PIN_A is absent locally"
  local DN; DN="$(git diff --name-only "$PIN_A" HEAD 2>/dev/null | tr '\n' '|')"
  [ "$DN" = "$PROBE_PATH|" ] || die2 "leg A: REFUSED_SETUP: the delta from A is '$DN', not exactly the one probe path"
  HB="$(git hash-object "$PROBE_PATH" 2>/dev/null)" || die2 "leg A: cannot hash $PROBE_PATH"
  [ "$HB" = "$BLOB_PROBE" ] || die2 "leg A: $PROBE_PATH blob is $HB, not the pinned $BLOB_PROBE"
  HB="$(git hash-object src/v2/test/claim/namespace_xl0/wildcard_arm_resolve.dag 2>/dev/null)" || die2 "leg A: cannot hash the frozen wc claim file"
  [ "$HB" = "$BLOB_WC" ] || die2 "leg A: the wc claim file blob is $HB, not the frozen $BLOB_WC"
  echo "leg A probe pin ok head=$PIN_P tree=$HT parent=$PIN_A delta=1 blob=$BLOB_PROBE wc=$BLOB_WC"
  snap "build-A"
  CARGO_TARGET_DIR="$R/target-A" CARGO_BUILD_JOBS=32 timeout 2400 cargo build -j 32 --locked --release -p v1-compiler --bin claim_batch > "$R/build-A.log" 2>&1
  local RC=$?; judge_oom "build-A"
  [ "$RC" = 0 ] || die2 "leg A: build failed rc=$RC (see build-A.log)"
  BIN="$R/target-A/release/claim_batch"
  BIN_SHA="$(hash_of "$BIN")" || die2 "leg A: cannot hash the claim_batch binary"
  echo "leg A claim_batch sha256=$BIN_SHA"
  probe_run "$BIN"
  tree_clean . || die2 "leg A: post-run git status --porcelain is not empty"
  local BIN_NOW; BIN_NOW="$(hash_of "$BIN")" || die2 "leg A: post-run cannot hash the binary"; [ "$BIN_NOW" = "$BIN_SHA" ] || die2 "leg A: claim_batch binary sha changed during the run"
  cd "$R" || die2 "leg A: cd back to OUT"
}

# ---- (10) RUNTIME-PATH fixtures, driven through probe_run itself with a STUB claim_batch (canned log + chosen rc).
# EXPECTED, stated before any run: valid3&rc1 -> 0; valid3&rc0 (batch-rc mismatch) -> 2; exit4 -> 2; malformed reason -> 2;
# NotBool -> 2; code 1 -> 2; PASS(rc0) -> 2 (no WCVERIFY reading); bare FAIL(rc1) -> 2. All non-validated paths are exit 2.
# v4 ARGV CONTRACT (msg_b02cda13): every stub asserts argv == "--claim-run --source-root dag --source-root src/v1 --source-root src/v2 --entry <probe path> --functions wc_verdict_probe_reports";
# any dropped or altered argument -> stub prints STUB_ARGV_REFUSAL and exits 99 -> claim_batch rc 3 with no outcome/completion lines -> exit 2 (NO OBSERVATION) -> the fixture gate FAILS (a dropped or altered flag can never pass fixtures).
# argvok  : exact-argv valid case -> 0;  argvdropfn: stub expects argv WITHOUT --functions while probe_run passes it -> mismatch -> 3 -> exit 2;
# argvbadsrc: --source-root dag altered -> 3 -> exit 2;  argvwrongfn: --functions value WRONG -> 3 -> exit 2;
# fnless: probe_run variant invoked WITHOUT --functions against the full-contract stub -> stub refusal -> fixture gate fires (parent-mandated case).
RTD="$(mktemp -d)" || die2 "mktemp RTD"
CL='[process-partition] total_ms=1000 pre_run_ms=0 args_ms=0 index_build_ms=0 edge_index_warm_ms=0 output_policy_ms=0 roster_ms=0 published_mock_precompute_ms=0 entries_ms=0 reporting_ms=0 entries_resolve_ms=0 entries_witness_ms=0'
RT_CONTRACT="--claim-run --source-root dag --source-root src/v1 --source-root src/v2 --entry $PROBE_PATH --functions $PFN"
# RT_WANT (env, default RT_CONTRACT) sets the argv string this stub must see; any positional arg is a log line.
mkstub() { local kind="$1" erc="$2" want="${RT_WANT-$RT_CONTRACT}"; unset RT_WANT 2>/dev/null || true; shift; shift; : > "$RTD/rt-log.$kind"; for ln in "$@"; do printf '%s\n' "$ln" >> "$RTD/rt-log.$kind"; done
  printf '#!/bin/bash\nif [ "$*" != "%s" ]; then echo "STUB_ARGV_REFUSAL expected=[%s] got=[$*]"; exit 3; fi\ncat "%s"\nexit %s\n' "$want" "$want" "$RTD/rt-log.$kind" "$erc" > "$RTD/rt-cb-$kind"; chmod +x "$RTD/rt-cb-$kind"; }
pr_run() { ( R="$RTD/rt-$1"; mkdir -p "$R"; RAW="$R/oom-raw.log"; OOM_SEEN=""; declare -A OBS; MISSING=0; DEVIATIONS=0
  probe_run "$RTD/rt-cb-$1" >/dev/null 2>&1 ); echo "$?"; }
V3L="FAIL $PFN (returned ProcessExit::ExitFailure, code 3: WCVERIFY ctx=OK arm=R field=R binder=R body=X{r=0,a=?,s=T,al=1})"
mkstub valid3 1 "$V3L" "$CL"
mkstub rc0   0 "$V3L" "$CL"
mkstub e4    1 "FAIL $PFN (returned ProcessExit::ExitFailure, code 4: WCVERIFY BOUND_VIOLATED len=140)" "$CL"
mkstub mal   1 "FAIL $PFN (returned ProcessExit::ExitFailure, code 3: WCVERIFY ctx=OK arm=Q field=R binder=R body=R)" "$CL"
mkstub nb    1 "FAIL $PFN (returned \`Int\`, not Bool; --claim-run entries must return Bool)" "$CL"
mkstub c1    1 "FAIL $PFN (returned ProcessExit::ExitFailure, code 1: boom)" "$CL"
mkstub pass  0 "PASS $PFN" "$CL"
mkstub bare  1 "FAIL $PFN" "$CL"
[ "$(pr_run valid3)" = 0 ] || die2 "REFUSED_SETUP: runtime fixture valid3&rc1 must exit 0"
[ "$(pr_run rc0)"   = 2 ] || die2 "REFUSED_SETUP: runtime fixture batch-rc mismatch must exit 2"
[ "$(pr_run e4)"    = 2 ] || die2 "REFUSED_SETUP: runtime fixture EXIT4 (invalid reading) must exit 2"
[ "$(pr_run mal)"   = 2 ] || die2 "REFUSED_SETUP: runtime fixture malformed reason must exit 2"
[ "$(pr_run nb)"    = 2 ] || die2 "REFUSED_SETUP: runtime fixture NotBool must exit 2"
[ "$(pr_run c1)"    = 2 ] || die2 "REFUSED_SETUP: runtime fixture code 1 must exit 2"
[ "$(pr_run pass)"  = 2 ] || die2 "REFUSED_SETUP: runtime fixture PASS must exit 2 (no WCVERIFY reading)"
[ "$(pr_run bare)"  = 2 ] || die2 "REFUSED_SETUP: runtime fixture bare FAIL must exit 2 (no WCVERIFY reading)"
mkstub argvok 1 "$V3L" "$CL"
RT_WANT="--claim-run --source-root dag --source-root src/v1 --source-root src/v2 --entry $PROBE_PATH" mkstub argvdropfn 1 "$V3L" "$CL"
RT_WANT="--claim-run --source-root WRONG --source-root src/v1 --source-root src/v2 --entry $PROBE_PATH --functions $PFN" mkstub argvbadsrc 1 "$V3L" "$CL"
RT_WANT="--claim-run --source-root dag --source-root src/v1 --source-root src/v2 --entry $PROBE_PATH --functions NOT_THE_PROBE_FN" mkstub argvwrongfn 1 "$V3L" "$CL"
# fnless: the parent-mandated case — probe_run invoked WITHOUT --functions must fail the fixtures.
pr_run_fnless() { ( R="$RTD/rt-fnless"; mkdir -p "$R"; RAW="$R/oom-raw.log"; OOM_SEEN=""; declare -A OBS; MISSING=0; DEVIATIONS=0
  timeout 900 "$RTD/rt-cb-argvok" --claim-run --source-root dag --source-root src/v1 --source-root src/v2 --entry "$PROBE_PATH" > "$R/claims-A-probe.log" 2>&1
  RC=$?; echo "claim_batch_exit[A probe]=$RC" >/dev/null
  rc_in_contract "$RC" "A probe" || true; completed "A probe" "$R/claims-A-probe.log" || true
  probe_verdict "$R/claims-A-probe.log" "$PFN" > "$R/.probe.vd" || true
  [ "$RC" = 1 ] || exit 2 ); echo "$?"; }
[ "$(pr_run argvok)"      = 0 ] || die2 "REFUSED_SETUP: runtime fixture argvok (exact argv) must exit 0"
[ "$(pr_run argvdropfn)"  = 2 ] || die2 "REFUSED_SETUP: runtime fixture argvdropfn (stub expects argv WITHOUT --functions) must exit 2 via stub exit 99"
[ "$(pr_run argvbadsrc)"  = 2 ] || die2 "REFUSED_SETUP: runtime fixture argvbadsrc (altered --source-root) must exit 2 via stub argv refusal"
[ "$(pr_run argvwrongfn)" = 2 ] || die2 "REFUSED_SETUP: runtime fixture argvwrongfn (WRONG --functions value) must exit 2 via stub argv refusal"
[ "$(pr_run_fnless)"      = 2 ] || die2 "REFUSED_SETUP: runtime fixture fnless (invoked WITHOUT --functions) must fire the fixture gate (stub argv refusal -> exit 2)"
grep -aq STUB_ARGV_REFUSAL "$RTD/rt-argvdropfn/claims-A-probe.log" 2>/dev/null || die2 "REFUSED_SETUP: argvdropfn did not exercise the stub argv refusal"
grep -aq STUB_ARGV_REFUSAL "$RTD/rt-argvwrongfn/claims-A-probe.log" 2>/dev/null || die2 "REFUSED_SETUP: argvwrongfn did not exercise the stub argv refusal"
grep -aq STUB_ARGV_REFUSAL "$RTD/rt-fnless/claims-A-probe.log" 2>/dev/null || die2 "REFUSED_SETUP: fnless did not exercise the stub argv refusal"
[ -s "$RTD/rt-pass/probe-nonreading.txt" ] || die2 "REFUSED_SETUP: the PASS event was not preserved in probe-nonreading.txt"
echo "runtime-path fixtures: all held"
echo FXRUN_OK_V5
