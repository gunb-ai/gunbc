# (7) probe classifier fixtures (a focused failing input for each misread class), before any fetch.
# All WCVERIFY payloads carry the v7 control fields (ctl_ok/ctl_out); the classifier is shape-agnostic
# and its contract is unchanged — only the embedded reason lines take the new declared layout.
W_OK='ctl_ok=r=0 ctl_out=r=X'
printf 'FAIL %s (returned ProcessExit::ExitFailure, code 3: WCVERIFY ctx=OK %s arm=R field=R binder=R body=X{r=0,a=?,s=T,al=1})\n' "$PFN" "$W_OK" > "$FX/p1"
printf 'PASS %s\nFAIL %s (returned ProcessExit::ExitFailure, code 3: WCVERIFY ctx=OK %s arm=R field=R binder=R body=X{r=0,a=?,s=T,al=1})\n' "$PFN" "$PFN" "$W_OK" > "$FX/p2"
printf 'PASS %s\n' "$PFN" > "$FX/p3"
printf 'FAIL %s\n' "$PFN" > "$FX/p4"
printf 'FAIL %s (returned ProcessExit::ExitFailure, code 1: boom)\n' "$PFN" > "$FX/p5"
printf 'UNOBSERVED %s (returned ProcessExit::ExitFailure, code 2: nothing observed)\n' "$PFN" > "$FX/p6"
printf 'FAIL %s (returned ProcessExit::ExitFailure, code 3: WCVERIFY ctx=RF %s arm=- field=- binder=- body=-\n' "$PFN" "$W_OK" > "$FX/p7"
printf 'FAIL %s (returned ProcessExit::ExitFailure, code 33: WCVERIFY ctx=OK %s arm=R field=R binder=R body=R)\n' "$PFN" "$W_OK" > "$FX/p8"
printf 'FAIL %s (returned `Int`, not Bool; --claim-run entries must return Bool)\n' "$PFN" > "$FX/p9"
printf 'FAIL %s (returned ProcessExit::ExitFailure, code 4: WCVERIFY BOUND_VIOLATED len=140)\n' "$PFN" > "$FX/p10"
[ "$(probe_verdict $FX/p1 $PFN) $(probe_verdict $FX/p2 $PFN) $(probe_verdict $FX/p3 $PFN) $(probe_verdict $FX/p4 $PFN) $(probe_verdict $FX/p5 $PFN) $(probe_verdict $FX/p6 $PFN) $(probe_verdict $FX/p7 $PFN) $(probe_verdict $FX/p8 $PFN) $(probe_verdict $FX/p9 $PFN) $(probe_verdict $FX/p10 $PFN)" = "EXIT3 NONE PASS BARE OTHER UNOBSERVED OTHER OTHER OTHER EXIT4" ] || die2 "REFUSED_SETUP: probe-classifier fixture misfire (valid, duplicate, PASS, bare, code1, UNOBSERVED, unclosed, code-33 boundary, NotBool, code-4)"
printf 'FAIL %s (returned ProcessExit::ExitFailure, code 3: WCVERIFY ctx=OK %s arm=R field=R binder=R body=X{r=0,a=?,s=T,al=1})\n' "$PFN" "$W_OK" > "$FX/pv"
probe_verdict "$FX/pv" "$PFN" >/dev/null
[ "$PROBE_REASON" = "WCVERIFY ctx=OK $W_OK arm=R field=R binder=R body=X{r=0,a=?,s=T,al=1}" ] || die2 "REFUSED_SETUP: probe reason extraction is not verbatim"

# (8) bound self-check: the BOUND is the LENGTH OF THE TABLE-COMPOSED WIDEST ROW (no hand-written
#   number survives anywhere). Printed for the run record.
echo "PROBE_BOUND: widest-declared=$WIDEST_LEN bound=$BOUND widest-token=${#WIDEST_TOKEN} ctl=($((${#SEP_ctl_ok} + 1 + ${#TOK_CTL} + 2)),$((${#SEP_ctl_out} + 1 + ${#TOK_CTL} + 2))) al-bound=${TT_al_BOUND}"

# (9) grammar fixtures through the ACTUAL path: a full typed line -> probe_verdict -> probe_validate.
PGD="$FX"
pv_line() { printf 'FAIL %s (returned ProcessExit::ExitFailure, code 3: %s)\n' "$PFN" "$1" > "$PGD/gline"; probe_verdict "$PGD/gline" "$PFN" >/dev/null; probe_validate "$PROBE_REASON"; echo $?; }
pv_kind() { printf 'FAIL %s (returned ProcessExit::ExitFailure, code 3: %s)\n' "$PFN" "$1" > "$PGD/gline"; probe_verdict "$PGD/gline" "$PFN" >/dev/null; probe_validate "$PROBE_REASON" >/dev/null; echo "$PROBE_VALIDATE_KIND"; }
# valid OK row and valid RF row (table-composed)
[ "$(pv_line "$(probe_row_ok 0 X R R R "$(probe_token 0 ? T 1)")")" = 0 ] || die2 "REFUSED_SETUP: grammar fixture (valid OK) refused"
[ "$(pv_line "$(probe_row_rf 0 X)")" = 0 ] || die2 "REFUSED_SETUP: grammar fixture (valid RF) refused"
# garbage reason; extra trailing field; unknown tag
[ "$(pv_line 'hello world')" = 1 ] || die2 "REFUSED_SETUP: grammar fixture (garbage) accepted"
[ "$(pv_line "$(probe_row_ok 0 X R R R R) extra=R")" = 1 ] || die2 "REFUSED_SETUP: grammar fixture (extra field) accepted"
[ "$(pv_line "$(probe_row_ok 0 X Q R R R)")" = 1 ] || die2 "REFUSED_SETUP: grammar fixture (unknown tag) accepted"
# malformed tokens, built OUTSIDE the table functions on purpose (the table cannot produce them):
BADTOK="X{r=24,a=U,s=T,al=1}"
[ "$(pv_line "$(probe_row_ok 0 X "$BADTOK" R R R)")" = 1 ] || die2 "REFUSED_SETUP: grammar fixture (reason index 24) accepted"
[ "$(pv_line "$(probe_row_ok 0 X "X{r=07,a=U,s=T,al=1}" R R R)")" = 1 ] || die2 "REFUSED_SETUP: grammar fixture (r=07 leading zero) accepted"
[ "$(pv_line "$(probe_row_ok 0 X "X{r=0,a=U,s=T,al=01}" R R R)")" = 1 ] || die2 "REFUSED_SETUP: grammar fixture (al=01 leading zero) accepted"
[ "$(pv_line "$(probe_row_ok 0 X X R R R)")" = 1 ] || die2 "REFUSED_SETUP: grammar fixture (bare tag X) accepted"
[ "$(pv_line 'WCVERIFY ctx=RF')" = 1 ] || die2 "REFUSED_SETUP: grammar fixture (bare RF) accepted"
# CONTROL-field fixtures: missing, out of order, illegal values, bare form
[ "$(pv_line "WCVERIFY ctx=OK ctl_out=r=X arm=R field=R binder=R body=R")" = 1 ] || die2 "REFUSED_SETUP: a row without ctl_ok was accepted"
[ "$(pv_line "WCVERIFY ctx=RF ctl_out=r=X arm=- field=- binder=- body=-")" = 1 ] || die2 "REFUSED_SETUP: an RF row without ctl_ok was accepted"
[ "$(pv_line "WCVERIFY ctx=OK ctl_out=r=X ctl_ok=r=0 arm=R field=R binder=R body=R")" = 1 ] || die2 "REFUSED_SETUP: ctl fields out of order were accepted"
[ "$(pv_line "$(probe_row_ok 24 X R R R R)")" = 1 ] || die2 "REFUSED_SETUP: ctl_ok=r=24 was accepted"
[ "$(pv_line "$(probe_row_ok 0 24 R R R R)")" = 1 ] || die2 "REFUSED_SETUP: ctl_out=r=24 was accepted"
[ "$(pv_line "WCVERIFY ctx=OK ctl_ok=X ctl_out=r=X arm=R field=R binder=R body=R")" = 1 ] || die2 "REFUSED_SETUP: a bare ctl token (no 'r=') was accepted"
# i64 staircase through the real path: max accepted; max+1 and 19x9 refused by GRAMMAR
[ "$(pv_line "$(probe_row_ok 0 X R R R "$(probe_token 23 U T 9223372036854775807)")")" = 0 ] || die2 "REFUSED_SETUP: i64-max al refused"
[ "$(pv_kind "$(probe_row_ok 0 X R R R "X{r=23,a=U,s=T,al=9223372036854775808}")")" = GRAMMAR ] || die2 "REFUSED_SETUP: i64max+1 must be a GRAMMAR refusal"
[ "$(pv_kind "$(probe_row_ok 0 X R R R "X{r=23,a=U,s=T,al=9999999999999999999}")")" = GRAMMAR ] || die2 "REFUSED_SETUP: 19x9 must be a GRAMMAR refusal"
# CONSISTENCY rule: a='-' with al!=0 refused as CONSISTENCY; with al=0 accepted
[ "$(pv_kind "$(probe_row_ok 0 X R R R "X{r=5,a=-,s=F,al=5}")")" = CONSISTENCY ] || die2 "REFUSED_SETUP: a=- with al!=0 must be a CONSISTENCY refusal"
[ "$(pv_line "$(probe_row_ok 0 X R R R "$(probe_token 5 - F 0)")")" = 0 ] || die2 "REFUSED_SETUP: a=- with al=0 refused"
# BOUND vs GRAMMAR are demonstrably distinct: widest accepted; widest+1 refused as BOUND; with the
# bound check bypassed (fixture-only toggle) the SAME row is refused as GRAMMAR.
LONG="${WIDEST_ROW}X"
[ "${#LONG}" = $((WIDEST_LEN + 1)) ] || die2 "REFUSED_SETUP: the over-long fixture is not widest+1"
[ "$(pv_kind "$LONG")" = BOUND ] || die2 "REFUSED_SETUP: widest+1 must be a BOUND refusal"
printf 'FAIL %s (returned ProcessExit::ExitFailure, code 3: %s)\n' "$PFN" "$LONG" > "$PGD/gline"
probe_verdict "$PGD/gline" "$PFN" >/dev/null
PROBE_BOUND_BYPASS=1 probe_validate "$PROBE_REASON" >/dev/null; [ "$PROBE_VALIDATE_KIND" = GRAMMAR ] || die2 "REFUSED_SETUP: with the bound bypassed the same row must be a GRAMMAR refusal"
# the v5 OBSERVED-BYTES line (raw claims-A-probe.log, sha256 6315352c...): kept ONLY as bytes; under
# the v7 grammar it is refused (no control fields), which is exactly why the v5 run was NO_OBSERVATION.
V5LINE='WCVERIFY ctx=OK arm=X{r=X,a=-,s=T,al=0} field=X{r=X,a=-,s=T,al=0} binder=X{r=X,a=-,s=T,al=0} body=X{r=X,a=-,s=T,al=0}'
[ "$(pv_line "$V5LINE")" = 1 ] || die2 "REFUSED_SETUP: the v5 observed-bytes row (no control fields) must be refused under the v7 grammar"
printf '%s\n' "$V5LINE" > "$PGD/observed-v5.txt"

# (10) PER-FORM SWEEP through the REAL validator AND the reading path: every declared alternative of
#   every field, one line each, each asserted ACCEPTED and mapped to its declared reading.
sw() {  # sw REASON NEEDLE...  -> assert accepted + reading-path contains each needle
  local r="$1"; shift
  [ "$(pv_line "$r")" = 0 ] || die2 "REFUSED_SETUP: sweep row refused: $r"
  local out; out="$(probe_read_line "$r")" || die2 "REFUSED_SETUP: sweep reading refused: $r"
  local needle
  for needle in "$@"; do printf '%s\n' "$out" | grep -qF "$needle" || die2 "REFUSED_SETUP: sweep reading missing '$needle' for: $r"; done
}
for k in 0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23; do
  TOK="$(probe_token "$k" U T 0)"; ROW="$(probe_row_ok 0 X "$TOK" R R R)"
  if [ "$k" = 0 ]; then sw "$ROW" "table index 0"; else sw "$ROW" "table index $k"; fi
done
TOK="$(probe_token X U T 0)"; sw "$(probe_row_ok 0 X "$TOK" R R R)" "outside-table OR comparison failure"
for a in U ? -; do
  TOK="$(probe_token 0 "$a" T 0)"; ROW="$(probe_row_ok 0 X R R R "$TOK")"
  case "$a" in U) sw "$ROW" 'atom == ^wc_undeclared_body';; \?) sw "$ROW" 'atom outside the candidate set';; -) sw "$ROW" 'atom ABSENT (no locus)';; esac
done
for s in T F; do
  TOK="$(probe_token 0 U "$s" 0)"; ROW="$(probe_row_ok 0 X R R R "$TOK")"
  if [ "$s" = T ]; then sw "$ROW" 'sole_chain=True'; else sw "$ROW" 'sole_chain=False'; fi
done
for al in 0 1 7 99 999999999 9223372036854775806 9223372036854775807; do
  TOK="$(probe_token 0 U T "$al")"; sw "$(probe_row_ok 0 X R R R "$TOK")" "atom_lexeme_len=$al"
done
for k in 0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23; do
  ROW="$(probe_row_ok "$k" X R R R R)"; OUT="$(probe_read_line "$ROW")"
  printf '%s\n' "$OUT" | grep -qF "CONTROL: ctl_ok=$k" || die2 "REFUSED_SETUP: ctl_ok sweep row missing: $k"
  if [ "$k" = 0 ]; then printf '%s\n' "$OUT" | grep -qF 'CONTROL HOLDS: the known symbol matched its own arm' || die2 "REFUSED_SETUP: ctl_ok=0 must read HOLDS"
  else printf '%s\n' "$OUT" | grep -qF 'PERMUTED' || die2 "REFUSED_SETUP: ctl_ok=$k must read PERMUTED"; fi
  ROW="$(probe_row_ok 0 "$k" R R R R)"; OUT="$(probe_read_line "$ROW")"
  printf '%s\n' "$OUT" | grep -qF "CONTROL: ctl_out=$k" || die2 "REFUSED_SETUP: ctl_out sweep row missing: $k"
  printf '%s\n' "$OUT" | grep -qF 'equality COLLAPSED' || die2 "REFUSED_SETUP: ctl_out=$k (a matched arm) must read COLLAPSED"
done
ROW="$(probe_row_ok 0 X R R R R)"; printf '%s\n' "$(probe_read_line "$ROW")" | grep -qF 'CONTROL HOLDS: the outside-table symbol matched nothing' || die2 "REFUSED_SETUP: ctl_out=X must read HOLDS"
ROW="$(probe_row_ok X X R R R R)"; printf '%s\n' "$(probe_read_line "$ROW")" | grep -qF 'COMPARISON FAILURE SIGNATURE' || die2 "REFUSED_SETUP: ctl_ok=X must yield the comparison-failure signature"

# (11) aggregate readings through the shipped reader, plus direct parse_token reachability fixtures
#   (the closing-brace checks are unreachable on grammar-validated rows, so they are exercised here).
probe_parse_token 'X{r=0,a=U,s=T,al=0' t1 t2 t3 t4 && die2 "REFUSED_SETUP: token missing the closing brace was accepted"
probe_parse_token 'X{r=0,a=U,s=T,al=0}}' t1 t2 t3 t4 && die2 "REFUSED_SETUP: token with a doubled closing brace was accepted"
probe_parse_token 'X{a=U,s=T,al=0}' t1 t2 t3 t4 && die2 "REFUSED_SETUP: token without the r key was accepted"
probe_parse_token 'X{r=0,s=T,a=U,al=0}' t1 t2 t3 t4 && die2 "REFUSED_SETUP: out-of-order token was accepted"
probe_parse_token 'X{r=0,a=U,s=T,al=0}' t1 t2 t3 t4 || die2 "REFUSED_SETUP: the well-formed token was refused"
[ "$t1" = 0 ] && [ "$t2" = U ] && [ "$t3" = T ] && [ "$t4" = 0 ] || die2 "REFUSED_SETUP: parse_token returned wrong fields for the well-formed token"
printf '%s\n' "$(probe_read_line "$(probe_row_ok 0 X F F F F)")" | grep -qF "LANE FILE-REFUSAL" || die2 "REFUSED_SETUP: all-F frozen reading missing"
printf '%s\n' "$(probe_read_line "$(probe_row_ok 0 X A A A A)")" | grep -qF "census-name/path" || die2 "REFUSED_SETUP: all-A frozen reading missing"
printf '%s\n' "$(probe_read_line "$(probe_row_rf 0 X)")" | grep -qF "INGEST ADMISSION" || die2 "REFUSED_SETUP: ctx=RF frozen reading missing"
printf '%s\n' "$(probe_read_line "$(probe_row_ok 0 X R R R "$(probe_token 0 U T 16)")")" | grep -qF "FALSIFIER FIRES" || die2 "REFUSED_SETUP: the falsifier row must fire"

# (12) ESCAPING/NEWLINE and CONFLICT fixtures through the real validator AND the runtime caller
#   (addendum msg_32d414bd item 2). The producer cannot emit these bytes (header charset audit); the
#   instrument must still fail closed if they ever appear.
# (a) malformed: a NEWLINE inside the reason. Validator: refused. Runtime caller: the FAIL line no
#     longer closes with ')' at its end, so probe_verdict must classify it NONE (=> NO_OBSERVATION).
BADNL="WCVERIFY ctx=OK $W_OK arm=R field=R binder=R body=X{r=0,a=U,s=T,
al=0}"
probe_validate "$BADNL" >/dev/null; [ "$PROBE_VALIDATE_KIND" != OK ] || die2 "REFUSED_SETUP: a reason containing a newline was accepted by the validator"
printf 'FAIL %s (returned ProcessExit::ExitFailure, code 3: WCVERIFY ctx=OK %s arm=R field=R binder=R body=X{r=0,a=U,s=T,\nal=0})\n' "$PFN" "$W_OK" > "$FX/nl"
NL_TOK="$(probe_verdict "$FX/nl" "$PFN")"
case "$NL_TOK" in NONE|OTHER) :;; *) die2 "REFUSED_SETUP: the newline-injected log line was not failed closed by the runtime caller (got $NL_TOK)";; esac
[ "$NL_TOK" != EXIT3 ] || die2 "REFUSED_SETUP: a reason spanning lines must never be classified EXIT3"
# (b) conflicting: TWO typed outcome lines, both individually valid, different reasons. The exactly-
#     one-outcome gate must refuse (NONE) — a reading requires a unique outcome.
printf 'FAIL %s (returned ProcessExit::ExitFailure, code 3: WCVERIFY ctx=OK %s arm=R field=R binder=R body=R)\nFAIL %s (returned ProcessExit::ExitFailure, code 3: WCVERIFY ctx=OK %s arm=A field=A binder=A body=A)\n' "$PFN" "$W_OK" "$PFN" "$W_OK" > "$FX/conflict"
[ "$(probe_verdict "$FX/conflict" "$PFN")" = NONE ] || die2 "REFUSED_SETUP: two conflicting outcome lines were not refused"

# (13) reader cross-checks: constants BYTE-IDENTICAL to the generated grammars; behavior agrees.
RD_SRC="$RS/.probe_read-src.py"
PY_RE_OK="$(sed -n 's/^RE_OK = re\.compile(r"\(.*\)")$/\1/p' "$RD_SRC")"
PY_RE_RF="$(sed -n 's/^RE_RF = re\.compile(r"\(.*\)")$/\1/p' "$RD_SRC")"
PY_BOUND="$(sed -n 's/^BOUND = \([0-9]*\)$/\1/p' "$RD_SRC")"
[ "$PY_RE_OK" = "$PG_OK" ] || die2 "REFUSED_SETUP: the embedded reader RE_OK is not byte-identical to the generated grammar"
[ "$PY_RE_RF" = "$PG_RF" ] || die2 "REFUSED_SETUP: the embedded reader RE_RF is not byte-identical to the generated grammar"
[ "$PY_BOUND" = "$BOUND" ] || die2 "REFUSED_SETUP: the embedded reader BOUND is not the table-composed widest length"
printf '%s\n' "$(probe_row_ok 0 X R R R "$(probe_token 0 U T 0)")" > "$PGD/xrow1.txt"
printf '%s\n' "$(probe_row_ok 3 X F A R "$(probe_token 12 ? T 42)")" > "$PGD/xrow2.txt"
printf '%s\n' "$(probe_row_rf 0 X)" > "$PGD/xrow3.txt"
printf '%s\n' "$(probe_row_ok X X R R R R)" > "$PGD/xrow4.txt"
if command -v python3 >/dev/null 2>&1; then
  for xf in xrow1 xrow2 xrow3 xrow4; do
    diff <(probe_read_line "$(cat "$PGD/$xf.txt")") <(python3 "$RD_SRC" "$PGD/$xf.txt") >/dev/null || die2 "REFUSED_SETUP: the embedded python reader disagrees with the shell reader on $xf"
  done
  echo "READER_PY=cross-checked (constants byte-identical; 4 rows behaviour-identical)"
else
  echo "READER_PY=absent-in-build-env (use the embedded copy at result/.probe_read.py offline)"
fi
rm -rf "$FX"
# B1 closure AT v6's failure point: the reader source must survive the fixture cleanup and still hash
# to the frozen constant (v6 died precisely here, at the probe_leg cp from the deleted $FX copy).
[ -f "$RS/.probe_read-src.py" ] || die2 "REFUSED_SETUP: the reader source did not survive the fixture cleanup (the v6 defect)"
[ "$(sha256sum "$RS/.probe_read-src.py" | cut -d' ' -f1)" = "$PROBE_READ_PY_SHA" ] || die2 "REFUSED_SETUP: the surviving reader source is not the frozen bytes"
