# ---- THE ONE AUTHORITY TABLE (v7; parent msg_bf3d2085 + msg_b8f24de9 + msg_32d414bd) ----
# The grammar, the row composition, and the reading lookup are GENERATED from the constants below and
# nothing else. The python reader's two regex constants are asserted byte-identical to the generated
# grammars at setup (cross-check fixtures below), so there is exactly one textual authority: these lines.
# Producer-side vocabulary, quoted from the probe source at the seed (wc_verdict_probe.dag, blob 0a2f577a,
# file sha256 0810abe3150a44d6f55f41d704fdd7e0ad418d82dee368c97154931636bade22), quoted in the v7 report:
#   verdict_text: WcResolved->'R'  WcAbsent->'A'  WcFileRefused->'F'
#     WcRefused{reason,at_atom,sole_chain}-> 'X' '{' reason_token ',' 'a' '=' atom_token ',' 's' '='
#       bool_token ',' 'al' '=' int_to_decimal_string '}'
#   ctl_text (v7 control): ' ctl_ok=' reason_token(^resolve_reason_unbound_symbol) ' ctl_out='
#     reason_token(^resolve_reason_binder_duplicate_in_frame)   [computed UNCONDITIONALLY, both arms]
#   reason_token: 24 equality arms against ^resolve_reason_* (the 24 names listed in the report),
#     else -> 'r=X'. The lane vocabulary at the seed is 37 DISTINCT ^resolve_reason_* reference-site
#     symbols (resolve.dag + name_resolve.dag, comment lines excluded), so the chain has 13+ arms with
#     no candidate and 'r=X' is REACHABLE BY CONSTRUCTION. What 'r=X' means (parent msg_02bc6f7c B2,
#     required wording): 'matched none of the 24 compared caret symbols (outside-table OR comparison
#     failure); loses which reason, and whether comparison works'. It is NOT evidence of an outside-table
#     reason. Whether Symbol '==' against caret literals discriminates at f08a6ae4 is UNVERIFIED; the
#     ctl_ok/ctl_out control fields settle it (see PROBE_CTL_READINGS). UNVERIFIED also: the 37/24/14
#     census itself, and whether symbol_lexeme includes the caret (it affects al; the payload is counted,
#     never rendered, so only the COUNT is observable).
EQ='='; CTX_PRE='WCVERIFY ctx='; CTX_OK="${CTX_PRE}OK"; CTX_RF="${CTX_PRE}RF"
SEP_ctl_ok=' ctl_ok'; SEP_ctl_out=' ctl_out'; TOK_CTL='r='
SEP_arm=' arm'; SEP_field=' field'; SEP_binder=' binder'; SEP_body=' body'
SEP_r='r'; SEP_a=',a'; SEP_s=',s'; SEP_al=',al'; TOK_OPEN='X'; TOK_IN='{'; TOK_CLOSE='}'
TT_plain='R|A|F'                                     # verdict_text plain tags (R | A | F)
TT_r='0|1|2|3|4|5|6|7|8|9|10|11|12|13|14|15|16|17|18|19|20|21|22|23|X'   # reason_token, 24 arms + X
TT_ctl="$TT_r"                                       # ctl tokens: SAME forms (generated, one source)
TT_a='U|?|-'                                         # atom_token
TT_s='T|F'                                           # bool_token
TT_al_BOUND='9223372036854775807'                    # i64 max: int_to_decimal_string can never exceed it
# The al alternation is GENERATED from TT_al_BOUND as a lexicographic staircase: '0', every canonical
# decimal of length 1..n-1, then same-length prefixes below the bound digit by digit, then the bound
# itself. Canonical decimals only (no leading zero; no '-'): the producer emits digits only
# (dag/std/nat.dag:219-231 decimal_digit_glyph; the '-' branch of integer.dag:316-335 is unreachable
# because atom_len is a fold_list count, never negative).
regex_le() {
  local B="$1" n="${#1}" i d pre cls out=('0')
  for ((i=1;i<n;i++)); do
    if [ "$i" = 1 ]; then out+=('[1-9]'); else out+=("[1-9][0-9]{$((i-1))}"); fi
  done
  for ((i=0;i<n;i++)); do
    d="${B:i:1}"
    [ "$d" = 0 ] && continue
    pre="${B:0:i}"
    if [ "$i" = 0 ]; then cls="[1-$((d-1))]"; else cls="[0-$((d-1))]"; fi
    if [ "$i" = $((n-1)) ]; then out+=("$pre$cls"); else out+=("$pre$cls[0-9]{$((n-i-1))}"); fi
  done
  out+=("$B")
  local IFS='|'; printf '%s' "${out[*]}"
}
TT_al_FORM="$(regex_le "$TT_al_BOUND")"
RD_R='WcResolved'; RD_A='WcAbsent'; RD_F='WcFileRefused'
RD_atom_U='atom == ^wc_undeclared_body (the frozen candidate)'; RD_atom_Q='atom outside the candidate set'; RD_atom_D='atom ABSENT (no locus)'
# Frozen reading table (parent msg_bf5a7d3c); 'observation only' lines are DATA, never acceptance.
RDX_NOCTX='READING (frozen table): ctx=RF — all four collapsed at the context build. The boundary is INGEST ADMISSION (native_test_context_from_ingest Rejected). Per-module rows do not exist for this shape.'
RDX_ALLF='FROZEN READING: all four F => the boundary is the LANE FILE-REFUSAL policy.'
RDX_ALLA='FROZEN READING: all four A => the fold never matched the census entries (census-name/path boundary).'
RDX_FALS='FALSIFIER FIRES: producer agrees with all four claims => the boundary is the claim wiring/shape, NOT the producer (H-wc FALSE).'
RDX_X='reason matched none of the 24 compared caret symbols (outside-table OR comparison failure); loses which reason, and whether comparison works'
# CONTROL readings (generated semantics; data, never acceptance). ctl_ok on ^resolve_reason_unbound_symbol
# (arm 0 of the chain), ctl_out on ^resolve_reason_binder_duplicate_in_frame (outside the 24).
ctl_ok_reading() {
  case "$1" in
    0) printf '%s\n' 'CONTROL HOLDS: the known symbol matched its own arm (0): caret "==" discriminates';;
    X) printf '%s\n' 'CONTROL VIOLATION: the known symbol matched NOTHING: caret "==" comparison is BROKEN (or the frozen 24-arm chain moved); every r=X in this line is then a comparison failure, not an outside-table reason';;
    *) printf '%s\n' "CONTROL VIOLATION: the known symbol matched arm $1 instead of 0: the 24-arm chain is PERMUTED (or the symbol is no longer arm 0)";;
  esac
}
ctl_out_reading() {
  case "$1" in
    X) printf '%s\n' 'CONTROL HOLDS: the outside-table symbol matched nothing (X), as declared: the chain does not compare symbols outside its 24 names';;
    *) printf '%s\n' "CONTROL VIOLATION: the outside-table symbol matched arm $1: equality COLLAPSED or the chain compares a symbol outside its 24 names";;
  esac
}
# probe_token_r R -> 'r=R', from the table. Values outside the declared set refuse.
probe_token_r() { case "|$TT_r|" in *'|'"$1"'|'**) printf '%s%s%s' "$SEP_r" "$EQ" "$1";; *) return 1;; esac; }
# probe_ctl FIELD VALUE -> ' FIELD=r=VALUE' (ctl tokens are reason_token outputs: 'r=' + value).
probe_ctl() { case "|$TT_ctl|" in *'|'"$2"'|'**) printf '%s%s%s%s' "$1" "$EQ" "$TOK_CTL" "$2";; *) return 1;; esac; }
# probe_token R A S AL -> the composed braced token, from the table only.
probe_token() {
  local r="$1" a="$2" s="$3" al="$4" t
  case "|$TT_r|" in *'|'"$r"'|'**) :;; *) return 1;; esac
  case "|$TT_a|" in *'|'"$a"'|'**) :;; *) return 1;; esac
  case "|$TT_s|" in *'|'"$s"'|'**) :;; *) return 1;; esac
  [[ "$al" =~ ^(${TT_al_FORM})$ ]] || return 1
  [ "${#al}" -le "${#TT_al_BOUND}" ] || return 1
  t="$SEP_r$EQ$r$SEP_a$EQ$a$SEP_s$EQ$s$SEP_al$EQ$al"
  printf '%s' "$TOK_OPEN$TOK_IN$t$TOK_CLOSE"
}
# probe_row_ok COK COUT T1 T2 T3 T4 / probe_row_rf COK COUT -> complete reason strings, table only.
probe_row_ok() {
  local c1 c2; c1="$(probe_ctl "$SEP_ctl_ok" "$1")" || return 1; c2="$(probe_ctl "$SEP_ctl_out" "$2")" || return 1
  printf '%s%s%s%s%s%s%s%s%s%s%s%s%s' "$CTX_OK" "$c1" "$c2" "$SEP_arm" "$EQ$3" "$SEP_field" "$EQ$4" "$SEP_binder" "$EQ$5" "$SEP_body" "$EQ$6"
}
probe_row_rf() {
  local c1 c2; c1="$(probe_ctl "$SEP_ctl_ok" "$1")" || return 1; c2="$(probe_ctl "$SEP_ctl_out" "$2")" || return 1
  printf '%s%s%s%s%s%s%s%s%s%s%s%s%s%s%s%s' "$CTX_RF" "$c1" "$c2" "$SEP_arm" "$EQ" "-" "$SEP_field" "$EQ" "-" "$SEP_binder" "$EQ" "-" "$SEP_body" "$EQ" "-"
}
# THE GRAMMARS, derived from the same constants. Every field alternative is the verbatim declared set,
# so any row that validates is byte-composable from the table alone (the split/reconstruct gate below
# proves it per row). Escapes ('{' '}' '?') are the ones ERE and python re accept IDENTICALLY; the
# byte-equality fixture against the reader constants enforces it.
PG_PLAIN="(${TT_plain})"
ALT_r="(${TT_r})"; ALT_a_RAW="$(printf '%s' "$TT_a" | sed 's/?/\\?/g')"; ALT_a="($ALT_a_RAW)"
ALT_s="(${TT_s})"; ALT_ctl="(${TT_ctl})"; ALT_al="(${TT_al_FORM})"
PG_X="${TOK_OPEN}\\{${SEP_r}${EQ}${ALT_r}${SEP_a}${EQ}${ALT_a}${SEP_s}${EQ}${ALT_s}${SEP_al}${EQ}${ALT_al}\\}"
PG_V="(${PG_PLAIN}|${PG_X})"
PG_OK="^${CTX_OK}${SEP_ctl_ok}${EQ}${TOK_CTL}${ALT_ctl}${SEP_ctl_out}${EQ}${TOK_CTL}${ALT_ctl}${SEP_arm}${EQ}${PG_V}${SEP_field}${EQ}${PG_V}${SEP_binder}${EQ}${PG_V}${SEP_body}${EQ}${PG_V}\$"
PG_RF="^${CTX_RF}${SEP_ctl_ok}${EQ}${TOK_CTL}${ALT_ctl}${SEP_ctl_out}${EQ}${TOK_CTL}${ALT_ctl}${SEP_arm}${EQ}-${SEP_field}${EQ}-${SEP_binder}${EQ}-${SEP_body}${EQ}-\$"
# The BOUND is not a hand-written number: it is the LENGTH OF THE TABLE-COMPOSED WIDEST ROW.
WIDEST_TOKEN="$(probe_token 23 '?' F "$TT_al_BOUND")" || die2 "REFUSED_SETUP: the widest declared token refused its own table"
WIDEST_ROW="$(probe_row_ok 23 23 "$WIDEST_TOKEN" "$WIDEST_TOKEN" "$WIDEST_TOKEN" "$WIDEST_TOKEN")" || die2 "REFUSED_SETUP: the widest declared row refused its own table"
WIDEST_LEN="${#WIDEST_ROW}"; BOUND="$WIDEST_LEN"
# probe_validate REASON -> 0 accepted / 1 refused; PROBE_VALIDATE_KIND records WHICH check refused
# (BOUND | GRAMMAR | CONSISTENCY). Check order is fixed: bound, grammar, consistency. The bound check
# may be bypassed ONLY via PROBE_BOUND_BYPASS=1, used by one fixture to prove the grammar itself
# refuses an over-long row, so the two checks are demonstrably distinct.
probe_validate() {
  local r="$1"
  PROBE_VALIDATE_KIND=OK
  if [ "${PROBE_BOUND_BYPASS:-0}" != 1 ]; then
    if [ "${#r}" -gt "$BOUND" ]; then PROBE_VALIDATE_KIND=BOUND; return 1; fi
  fi
  if [[ "$r" =~ $PG_RF ]]; then :
  elif [[ "$r" =~ $PG_OK ]]; then :
  else PROBE_VALIDATE_KIND=GRAMMAR; return 1; fi
  # CONSISTENCY (table rule TT_rule_a_minus_al_zero=1): a braced token with a='-' must carry al='0'.
  # RF rows have no braced tokens; OK rows are split once and each token parsed with the SAME parser.
  local c1 c2 t_arm t_field t_binder t_body tok t1 t2 t3 t4 a al
  if [[ "$r" =~ $PG_OK ]]; then
    probe_split_ok "$r" c1 c2 t_arm t_field t_binder t_body || { PROBE_VALIDATE_KIND=GRAMMAR; return 1; }
    for tok in "$t_arm" "$t_field" "$t_binder" "$t_body"; do
      case "$tok" in "$TOK_OPEN$TOK_IN"*)
        probe_parse_token "$tok" t1 t2 t3 t4 || { PROBE_VALIDATE_KIND=GRAMMAR; return 1; }
        a="$t2"; al="$t4"
        if [ "$a" = '-' ] && [ "$al" != 0 ]; then PROBE_VALIDATE_KIND=CONSISTENCY; return 1; fi
      esac
    done
  fi
  return 0
}
# probe_parse_token TOKEN RVAR AVAR SVAR ALVAR -> 0 and sets the four variables; refuses (1) on any
# token that is not composed of declared alternatives with the declared separators, in order, once each.
# The closing-brace checks are REACHABLE via direct-call fixtures (v7-p1..p4): on grammar-validated
# rows the anchored grammar already excludes those shapes, so this defense is exercised directly.
probe_parse_token() {
  local v="$1" __r __a __s __al rest
  local -n __R="$2" __A="$3" __S="$4" __AL="$5"
  case "$v" in
    R) __r='__plain'; __a='R'; __s='__tag'; __al='R';;
    A) __r='__plain'; __a='A'; __s='__tag'; __al='A';;
    F) __r='__plain'; __a='F'; __s='__tag'; __al='F';;
    "$TOK_OPEN$TOK_IN"*)
      rest="${v#"$TOK_OPEN$TOK_IN"}"
      case "$v" in *"$TOK_CLOSE") :;; *) return 1;; esac           # must end with the single TOK_CLOSE
      rest="${rest%"$TOK_CLOSE"}"
      case "$rest" in *"$TOK_CLOSE"*) return 1;; esac           # a second '}' anywhere is illegal
      case "$rest" in "$SEP_r$EQ"*) :;; *) return 1;; esac
      rest="${rest#"$SEP_r$EQ"}"
      case "$rest" in *"$SEP_a$EQ"*) :;; *) return 1;; esac
      __r="${rest%%"$SEP_a$EQ"*}"; rest="${rest#*"$SEP_a$EQ"}"
      case "$rest" in *"$SEP_s$EQ"*) :;; *) return 1;; esac
      __a="${rest%%"$SEP_s$EQ"*}"; rest="${rest#*"$SEP_s$EQ"}"
      case "$rest" in *"$SEP_al$EQ"*) :;; *) return 1;; esac
      __s="${rest%%"$SEP_al$EQ"*}"; __al="${rest#*"$SEP_al$EQ"}"
      case "|$TT_r|" in *'|'"$__r"'|'**) :;; *) return 1;; esac
      case "|$TT_a|" in *'|'"$__a"'|'**) :;; *) return 1;; esac
      case "|$TT_s|" in *'|'"$__s"'|'**) :;; *) return 1;; esac
      [[ "$__al" =~ ^(${TT_al_FORM})$ ]] || return 1
      [ "${#__al}" -le "${#TT_al_BOUND}" ] || return 1;;
    *) return 1;;
  esac
  if [ "$__r" = __plain ]; then __R="$__al"; __A=''; __S=''; __AL=''; else __R="$__r"; __A="$__a"; __S="$__s"; __AL="$__al"; fi
  return 0
}
# probe_tok_reading TOKEN -> the reading of one verdict token; refuses (1) on an illegal token.
probe_tok_reading() {
  local v="$1" r a s al
  case "$v" in R|A|F) printf '%s\n' "$(case "$v" in R) echo "$RD_R";; A) echo "$RD_A";; *) echo "$RD_F";; esac)"; return 0;; esac
  probe_parse_token "$v" r a s al || return 1
  local rs; if [ "$r" = X ]; then rs="$RDX_X"; else rs="reason = table index $r (the 24-name resolve_reason list)"; fi
  local at; case "$a" in U) at="$RD_atom_U";; \?) at="$RD_atom_Q";; -) at="$RD_atom_D";; *) return 1;; esac
  local sc; if [ "$s" = T ]; then sc='sole_chain=True (rest of the diagnostic chain Empty)'; else sc='sole_chain=False'; fi
  printf 'WcRefused{%s; %s; %s; atom_lexeme_len=%s}\n' "$rs" "$at" "$sc" "$al"
}
# probe_split_ok ROW C1 C2 T_ARM T_FIELD T_BINDER T_BODY — delimiter-based split on the SIX keys, in
# order, each exactly once; the reconstruction via probe_row_ok must reproduce the row BYTE-FOR-BYTE
# (layout proof; one authority). Values are bare (ctl values stripped of the 'r=' prefix here only).
probe_split_ok() {
  local r="$1" line
  local -n _C1="$2" _C2="$3" _A1="$4" _A2="$5" _A3="$6" _A4="$7"
  case "$r" in "$CTX_OK"*) :;; *) return 1;; esac
  local p
  for p in "$SEP_ctl_ok" "$SEP_ctl_out" "$SEP_arm" "$SEP_field" "$SEP_binder"; do
    case "$r" in *"$p$EQ"*) :;; *) return 1;; esac
  done
  line="$(printf '%s\n' "$r" | sed -e "s/^${CTX_OK}//" -e "s/$SEP_ctl_ok$EQ/\n/" -e "s/$SEP_ctl_out$EQ/\n/" -e "s/$SEP_arm$EQ/\n/" -e "s/$SEP_field$EQ/\n/" -e "s/$SEP_binder$EQ/\n/" -e "s/$SEP_body$EQ/\n/" | tr -d ' ' | sed '/^$/d')"
  [ "$(printf '%s\n' "$line" | wc -l)" = 6 ] || return 1
  _C1="$(printf '%s\n' "$line" | sed -n 1p)"; _C2="$(printf '%s\n' "$line" | sed -n 2p)"
  _C1="${_C1#"$TOK_CTL"}"; _C2="${_C2#"$TOK_CTL"}"
  _A1="$(printf '%s\n' "$line" | sed -n 3p)"; _A2="$(printf '%s\n' "$line" | sed -n 4p)"
  _A3="$(printf '%s\n' "$line" | sed -n 5p)"; _A4="$(printf '%s\n' "$line" | sed -n 6p)"
  local rebuilt; rebuilt="$(probe_row_ok "$_C1" "$_C2" "$_A1" "$_A2" "$_A3" "$_A4")"
  [ "$rebuilt" = "$r" ] || return 1
  return 0
}
# probe_read_line REASON: the reading-path lookup, mirroring probe_read.py (shipped in the packet and
# sha-gate-checked below; its grammar constants are byte-checked against PG_OK/PG_RF at setup).
# Observation only; it never implies acceptance.
probe_read_line() {
  local r="$1"
  if [ "${#r}" -gt "$BOUND" ]; then echo "REFUSAL: over the stated ${BOUND}-char bound (widest table-composed row); NO OBSERVATION"; return 2; fi
  local c1 c2 arm field binder body f v rd allf=1 alla=1
  if [[ "$r" =~ $PG_RF ]]; then
    # RF rows still carry the control; extract it with the same byte-exact discipline.
    local line; line="$(printf '%s\n' "$r" | sed -e "s/^${CTX_RF}//" -e "s/$SEP_ctl_ok$EQ/\n/" -e "s/$SEP_ctl_out$EQ/\n/" -e "s/$SEP_arm$EQ/\n/" -e "s/$SEP_field$EQ/\n/" -e "s/$SEP_binder$EQ/\n/" -e "s/$SEP_body$EQ/\n/" | tr -d ' ' | sed '/^$/d')"
    [ "$(printf '%s\n' "$line" | wc -l)" = 6 ] || { echo 'REFUSAL: RF control split failed'; return 2; }
    c1="$(printf '%s\n' "$line" | sed -n 1p)"; c2="$(printf '%s\n' "$line" | sed -n 2p)"
    c1="${c1#"$TOK_CTL"}"; c2="${c2#"$TOK_CTL"}"
    case "|$TT_ctl|" in *'|'"$c1"'|'**) :;; *) echo 'REFUSAL: ctl_ok is not a declared form'; return 2;; esac
    case "|$TT_ctl|" in *'|'"$c2"'|'**) :;; *) echo 'REFUSAL: ctl_out is not a declared form'; return 2;; esac
    [ "$(printf '%s\n' "$line" | sed -n 3p)" = - ] && [ "$(printf '%s\n' "$line" | sed -n 4p)" = - ] &&
    [ "$(printf '%s\n' "$line" | sed -n 5p)" = - ] && [ "$(printf '%s\n' "$line" | sed -n 6p)" = - ] ||
      { echo 'REFUSAL: RF row carries non-dash payload'; return 2; }
    echo "CONTROL (ctx=RF row): ctl_ok=$c1 -> $(ctl_ok_reading "$c1")"
    echo "CONTROL (ctx=RF row): ctl_out=$c2 -> $(ctl_out_reading "$c2")"
    echo "$RDX_NOCTX"; return 0
  fi
  if ! [[ "$r" =~ $PG_OK ]]; then echo 'REFUSAL: does not match the anchored grammar — treat as NO OBSERVATION'; return 2; fi
  probe_split_ok "$r" c1 c2 arm field binder body || { echo 'REFUSAL: layout parse failed (keys missing, duplicated, or out of order)'; return 2; }
  case "|$TT_ctl|" in *'|'"$c1"'|'**) :;; *) echo 'REFUSAL: ctl_ok is not a declared form'; return 2;; esac
  case "|$TT_ctl|" in *'|'"$c2"'|'**) :;; *) echo 'REFUSAL: ctl_out is not a declared form'; return 2;; esac
  echo "CONTROL: ctl_ok=$c1 -> $(ctl_ok_reading "$c1")"
  echo "CONTROL: ctl_out=$c2 -> $(ctl_out_reading "$c2")"
  if [ "$c1" = 0 ] && [ "$c2" = X ]; then echo 'CONTROL VERDICT (observation only): caret "==" comparison DISCRIMINATES for the two control symbols.'; else echo 'CONTROL VERDICT (observation only): COMPARISON FAILURE SIGNATURE — read every r=X as a possible broken equality, never as an outside-table reason, until located.'; fi
  echo 'PER-MODULE READINGS (MIXED tags => read each row on its own; no single-boundary conclusion follows):'
  for f in arm field binder body; do
    v="${!f}"
    rd="$(probe_tok_reading "$v")" || { echo "REFUSAL: token $v is not a declared form"; return 2; }
    printf '  %-7s %-52s %s\n' "$f" "$v" "$rd"
    [ "$v" = F ] || allf=0
    [ "$v" = A ] || alla=0
  done
  if [ "$allf" = 1 ]; then echo "$RDX_ALLF"; fi
  if [ "$alla" = 1 ]; then echo "$RDX_ALLA"; fi
  local okarm=0 okfield=0 okbinder=0 okbody=0
  [ "$arm" = R ] && okarm=1
  [ "$field" = R ] && okfield=1
  [ "$binder" = R ] && okbinder=1
  case "$body" in "${TOK_OPEN}${TOK_IN}${SEP_r}${EQ}0${SEP_a}${EQ}U${SEP_s}${EQ}T${SEP_al}${EQ}"*) okbody=1;; esac
  :
  echo 'CLAIM EXPECTATIONS (the frozen wc claims; the body expectation names r=0, a=U and s=T — al is a datum, not an expectation):'
  printf '  %-7s %-47s observed %-44s %s\n' arm 'expect R' "$arm" "$([ "$okarm" = 1 ] && echo MATCH || echo MISMATCH)"
  printf '  %-7s %-47s observed %-44s %s\n' field 'expect R' "$field" "$([ "$okfield" = 1 ] && echo MATCH || echo MISMATCH)"
  printf '  %-7s %-47s observed %-44s %s\n' binder 'expect R' "$binder" "$([ "$okbinder" = 1 ] && echo MATCH || echo MISMATCH)"
  printf '  %-7s %-47s observed %-44s %s\n' body 'expect X with r=0, a=U and s=T (sole unbound refusal at ^wc_undeclared_body)' "$body" "$([ "$okbody" = 1 ] && echo MATCH || echo MISMATCH)"
  if [ "$okarm" = 1 ] && [ "$okfield" = 1 ] && [ "$okbinder" = 1 ] && [ "$okbody" = 1 ]; then echo "$RDX_FALS"; fi
  echo 'Observation only; acceptance never follows from this reading.'
  return 0
}
