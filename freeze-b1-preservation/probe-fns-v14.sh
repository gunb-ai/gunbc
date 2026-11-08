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
# The closing-brace checks are REACHABLE via direct-call fixtures (v8-p1..p4): on grammar-validated
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
  local rs _nm
  if [ "$r" = X ]; then rs="$RDX_X"; else
    _nm="${CENSUS37[$r]:-MISSING-INDEX-IN-TABLE}"
    rs="reason = table index $r (name from the one 37-name table: $_nm)"
    [ "$_nm" = MISSING-INDEX-IN-TABLE ] && rs="$rs — READER DEFECT: index outside the table"
  fi
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
    if [ "$c1" = 0 ] && [ "$c2" = X ]; then echo "$RDX_NOCTX"; else echo "$CTL_VOID"; fi; return 0
  fi
  if ! [[ "$r" =~ $PG_OK ]]; then echo 'REFUSAL: does not match the anchored grammar — treat as NO OBSERVATION'; return 2; fi
  probe_split_ok "$r" c1 c2 arm field binder body || { echo 'REFUSAL: layout parse failed (keys missing, duplicated, or out of order)'; return 2; }
  case "|$TT_ctl|" in *'|'"$c1"'|'**) :;; *) echo 'REFUSAL: ctl_ok is not a declared form'; return 2;; esac
  case "|$TT_ctl|" in *'|'"$c2"'|'**) :;; *) echo 'REFUSAL: ctl_out is not a declared form'; return 2;; esac
  echo "CONTROL: ctl_ok=$c1 -> $(ctl_ok_reading "$c1")"
  echo "CONTROL: ctl_out=$c2 -> $(ctl_out_reading "$c2")"
  if [ "$c1" != 0 ] || [ "$c2" != X ]; then
    for f in arm field binder body; do probe_tok_reading "${!f}" >/dev/null || { echo "REFUSAL: token ${!f} is not a declared form"; return 2; }; done
    echo "$CTL_VOID"
    echo 'Observation only; acceptance never follows from this reading.'
    return 0
  fi
  echo 'CONTROL VERDICT (observation only): caret "==" comparison DISCRIMINATES for the two control symbols.'
  echo 'CLAIM EXPECTATIONS (the frozen wc claims; the body expectation names r=0, a=U and s=T — al is a datum, not an expectation):'
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
  printf '  %-7s %-47s observed %-44s %s\n' arm 'expect R' "$arm" "$([ "$okarm" = 1 ] && echo MATCH || echo MISMATCH)"
  printf '  %-7s %-47s observed %-44s %s\n' field 'expect R' "$field" "$([ "$okfield" = 1 ] && echo MATCH || echo MISMATCH)"
  printf '  %-7s %-47s observed %-44s %s\n' binder 'expect R' "$binder" "$([ "$okbinder" = 1 ] && echo MATCH || echo MISMATCH)"
  printf '  %-7s %-47s observed %-44s %s\n' body 'expect X with r=0, a=U and s=T (sole unbound refusal at ^wc_undeclared_body)' "$body" "$([ "$okbody" = 1 ] && echo MATCH || echo MISMATCH)"
  if [ "$okarm" = 1 ] && [ "$okfield" = 1 ] && [ "$okbinder" = 1 ] && [ "$okbody" = 1 ]; then echo "$RDX_FALS"; fi
  echo 'Observation only; acceptance never follows from this reading.'
  return 0
}

PROBE_TOKEN=NONE
probe_verdict() {
  local L n; L="$(grep -aE "^[A-Z]+ $2( |\$)" "$1")"
  if [ -z "$L" ]; then PROBE_TOKEN=NONE; echo NONE; return; fi
  n="$(printf '%s\n' "$L" | grep -ac '')"
  if [ "$n" != 1 ]; then PROBE_TOKEN=NONE; echo NONE; return; fi
  probe_classify "$L" "$2" >/dev/null; echo "$PROBE_TOKEN"
}

# (6b) THE SHIPPED OFFLINE READER, embedded verbatim (quoted heredoc delimiter: zero expansion at write),
#   and sha-gated at run time into $R. The offline reading path IS the embedded copy; the fixtures exercise the
#   shipped shell reader, and a cross-check fixture (when python3 exists in the build env) asserts BOTH agree.
PROBE_READ_PY_SHA=eef548c551bb99ebccda83c590a02456c38dce19a849ed5545f432b33509293f

# ---- PRE-FETCH SELF-CONSISTENCY (parent msg_3152d6ec, ERS latent defect: an embedded roster with a trailing
# newline sha'd wrong and every real run would die AFTER the fetch; a pre-fetch E2E cannot see post-fetch gates).
# Every post-fetch gate whose inputs are EMBEDDED in this packet is exercised HERE, over those exact bytes:
#   (a) declared hex constants are well-formed (the post-fetch pin/tree/blob gates consume them verbatim);
#   (b) the roster name passes the outcome-join token regex and the single-name set behaves under EXACT
#       membership (multi-token, empty, hyphenated, keyword and '<name> extra' non-members all refuse;
#       count equals roster size) — the ERS v4 class, refused before the fetch;
#   (c) PROBE_READ_PY_SHA is well-formed (the embedded source itself is sha-gated before any fetch below).
for _c in "$PIN_P" "$TREE_P" "$BLOB_PROBE" "$BLOB_WC" "$SEED_PARENT"; do
  [[ "$_c" =~ ^[0-9a-f]{40}$ ]] || die2 "REFUSED_SETUP: embedded 40-hex constant '$_c' is malformed (post-fetch pin/blob gates would die after the fetch)"
done
[[ "$PROBE_READ_PY_SHA" =~ ^[0-9a-f]{64}$ ]] || die2 "REFUSED_SETUP: PROBE_READ_PY_SHA is malformed"
# v11 (parent msg_b897e1be): the post-fetch parent gate needs SEED_PARENT == PIN_A; checked pre-fetch over
# the embedded constants, so a wrong-parent seed is refused BEFORE the fetch instead of after it.
[ "$SEED_PARENT" = "$PIN_A" ] || die2 "REFUSED_SETUP: embedded SEED_PARENT $SEED_PARENT != PIN_A $PIN_A (the depth-2 fetch could not satisfy HEAD^1 == PIN_A)"
[[ "$PFN" =~ ^[a-z0-9_]+$ ]] || die2 "REFUSED_SETUP: the roster name PFN does not pass the outcome-join token regex"
{ declare -A SELF_ROSTER=([$PFN]=1)
  [ "${#SELF_ROSTER[@]}" = 1 ] || die2 "REFUSED_SETUP: embedded roster size != 1"
  [ -n "${SELF_ROSTER[$PFN]+x}" ] || die2 "REFUSED_SETUP: roster name not an exact member of its own set"
  for _bad in 'a b' '' 'x-y' 'PASS' "${PFN} extra"; do
    [ -z "${SELF_ROSTER[$_bad]+x}" ] || die2 "REFUSED_SETUP: roster set accepted non-member '$_bad' (substring/loose matching defect)"
  done
}
echo "SELF_CONSISTENCY=held (hex constants incl SEED_PARENT, SEED_PARENT==PIN_A, PROBE_READ_PY_SHA form, roster EXACT membership)"
cat > "$RS/.probe_read-src.py" <<'PROBE_READ_EOF'
#!/usr/bin/env python3
# probe_read.py v8 — the offline reader for the wc verdict probe (sleek-seal-226, PR #15695).
# Usage: python3 probe_read.py <file-containing-exactly-one-WCVERIFY-reason-line>
# Contract: exactly one line; if it does not validate, print REFUSAL and exit 2. Output is OBSERVATION
# ONLY; acceptance never follows. The two regex constants below are asserted BYTE-IDENTICAL to the
# grammars generated from the ONE authority table in the packet (fixtures at setup), and BOUND equals
# the length of the table-composed widest row; any drift is REFUSED_SETUP in the packet before any run.
CENSUS37 = [
'resolve_reason_unbound_symbol',
'resolve_reason_ambiguous_symbol',
'resolve_reason_declarer_refused',
'resolve_reason_pattern_atom_names_visible_value',
'resolve_reason_test_code_referenced',
'resolve_reason_generated_reference_declaration_absent',
'resolve_reason_malformed_imported_root',
'resolve_reason_type_parameter_shadows_visible_name',
'resolve_reason_reference_category_mismatch',
'resolve_reason_realized_declaration_mismatch',
'resolve_reason_where_predicate_unbound',
'resolve_reason_type_parameter_in_value_position',
'resolve_reason_realized_declaration_shadowed',
'resolve_reason_malformed_tree',
'resolve_reason_primitive_row_name_disagrees',
'resolve_reason_lexical_reference_unkeyed',
'resolve_reason_construct_tag_not_a_constructor',
'resolve_reason_catch_all_binder_unread',
'resolve_reason_binder_hides_visible_value',
'resolve_reason_anonymous_parameter_referenced',
'resolve_reason_type_application_arity_mismatch',
'resolve_reason_type_argument_kind_not_inhabited',
'resolve_reason_requirement_not_a_resource',
'resolve_reason_binder_duplicate_in_frame',
'resolve_reason_forwarded_parameter_kind_differs',
'resolve_reason_forwarded_parameter_kind_unresolved',
'resolve_reason_forwarded_parameter_unkinded',
'resolve_reason_kernel_type_spelling_names_a_foreign_declaration',
'resolve_reason_member_kind_duplicate_declaration',
'resolve_reason_namespace_provider_admission_conflict',
'resolve_reason_observation_incomplete',
'resolve_reason_optional_declaration_incompatible',
'resolve_reason_optional_declaration_unavailable',
'resolve_reason_parameter_path_not_indexed',
'resolve_reason_requirement_repeated',
'resolve_reason_type_parameter_kind_unresolved',
'resolve_reason_type_parameter_order_malformed',
]
assert len(CENSUS37)==37 and len(set(CENSUS37))==37
assert CENSUS37[0]=='resolve_reason_unbound_symbol'
assert CENSUS37[36]=='resolve_reason_type_parameter_order_malformed'
import re, sys

BOUND = 218

RE_OK = re.compile(r"^WCVERIFY ctx=OK ctl_ok=r=(0|1|2|3|4|5|6|7|8|9|10|11|12|13|14|15|16|17|18|19|20|21|22|23|24|25|26|27|28|29|30|31|32|33|34|35|36|X) ctl_out=r=(0|1|2|3|4|5|6|7|8|9|10|11|12|13|14|15|16|17|18|19|20|21|22|23|24|25|26|27|28|29|30|31|32|33|34|35|36|X) arm=((R|A|F)|X\{r=(0|1|2|3|4|5|6|7|8|9|10|11|12|13|14|15|16|17|18|19|20|21|22|23|24|25|26|27|28|29|30|31|32|33|34|35|36|X),a=(U|\?|-),s=(T|F),al=(0|[1-9]|[1-9][0-9]{1}|[1-9][0-9]{2}|[1-9][0-9]{3}|[1-9][0-9]{4}|[1-9][0-9]{5}|[1-9][0-9]{6}|[1-9][0-9]{7}|[1-9][0-9]{8}|[1-9][0-9]{9}|[1-9][0-9]{10}|[1-9][0-9]{11}|[1-9][0-9]{12}|[1-9][0-9]{13}|[1-9][0-9]{14}|[1-9][0-9]{15}|[1-9][0-9]{16}|[1-9][0-9]{17}|[1-8][0-9]{18}|9[0-1][0-9]{17}|92[0-1][0-9]{16}|922[0-2][0-9]{15}|9223[0-2][0-9]{14}|92233[0-6][0-9]{13}|922337[0-1][0-9]{12}|92233720[0-2][0-9]{10}|922337203[0-5][0-9]{9}|9223372036[0-7][0-9]{8}|92233720368[0-4][0-9]{7}|922337203685[0-3][0-9]{6}|9223372036854[0-6][0-9]{5}|92233720368547[0-6][0-9]{4}|922337203685477[0-4][0-9]{3}|9223372036854775[0-7][0-9]{2}|922337203685477580[0-6]|9223372036854775807)\}) field=((R|A|F)|X\{r=(0|1|2|3|4|5|6|7|8|9|10|11|12|13|14|15|16|17|18|19|20|21|22|23|24|25|26|27|28|29|30|31|32|33|34|35|36|X),a=(U|\?|-),s=(T|F),al=(0|[1-9]|[1-9][0-9]{1}|[1-9][0-9]{2}|[1-9][0-9]{3}|[1-9][0-9]{4}|[1-9][0-9]{5}|[1-9][0-9]{6}|[1-9][0-9]{7}|[1-9][0-9]{8}|[1-9][0-9]{9}|[1-9][0-9]{10}|[1-9][0-9]{11}|[1-9][0-9]{12}|[1-9][0-9]{13}|[1-9][0-9]{14}|[1-9][0-9]{15}|[1-9][0-9]{16}|[1-9][0-9]{17}|[1-8][0-9]{18}|9[0-1][0-9]{17}|92[0-1][0-9]{16}|922[0-2][0-9]{15}|9223[0-2][0-9]{14}|92233[0-6][0-9]{13}|922337[0-1][0-9]{12}|92233720[0-2][0-9]{10}|922337203[0-5][0-9]{9}|9223372036[0-7][0-9]{8}|92233720368[0-4][0-9]{7}|922337203685[0-3][0-9]{6}|9223372036854[0-6][0-9]{5}|92233720368547[0-6][0-9]{4}|922337203685477[0-4][0-9]{3}|9223372036854775[0-7][0-9]{2}|922337203685477580[0-6]|9223372036854775807)\}) binder=((R|A|F)|X\{r=(0|1|2|3|4|5|6|7|8|9|10|11|12|13|14|15|16|17|18|19|20|21|22|23|24|25|26|27|28|29|30|31|32|33|34|35|36|X),a=(U|\?|-),s=(T|F),al=(0|[1-9]|[1-9][0-9]{1}|[1-9][0-9]{2}|[1-9][0-9]{3}|[1-9][0-9]{4}|[1-9][0-9]{5}|[1-9][0-9]{6}|[1-9][0-9]{7}|[1-9][0-9]{8}|[1-9][0-9]{9}|[1-9][0-9]{10}|[1-9][0-9]{11}|[1-9][0-9]{12}|[1-9][0-9]{13}|[1-9][0-9]{14}|[1-9][0-9]{15}|[1-9][0-9]{16}|[1-9][0-9]{17}|[1-8][0-9]{18}|9[0-1][0-9]{17}|92[0-1][0-9]{16}|922[0-2][0-9]{15}|9223[0-2][0-9]{14}|92233[0-6][0-9]{13}|922337[0-1][0-9]{12}|92233720[0-2][0-9]{10}|922337203[0-5][0-9]{9}|9223372036[0-7][0-9]{8}|92233720368[0-4][0-9]{7}|922337203685[0-3][0-9]{6}|9223372036854[0-6][0-9]{5}|92233720368547[0-6][0-9]{4}|922337203685477[0-4][0-9]{3}|9223372036854775[0-7][0-9]{2}|922337203685477580[0-6]|9223372036854775807)\}) body=((R|A|F)|X\{r=(0|1|2|3|4|5|6|7|8|9|10|11|12|13|14|15|16|17|18|19|20|21|22|23|24|25|26|27|28|29|30|31|32|33|34|35|36|X),a=(U|\?|-),s=(T|F),al=(0|[1-9]|[1-9][0-9]{1}|[1-9][0-9]{2}|[1-9][0-9]{3}|[1-9][0-9]{4}|[1-9][0-9]{5}|[1-9][0-9]{6}|[1-9][0-9]{7}|[1-9][0-9]{8}|[1-9][0-9]{9}|[1-9][0-9]{10}|[1-9][0-9]{11}|[1-9][0-9]{12}|[1-9][0-9]{13}|[1-9][0-9]{14}|[1-9][0-9]{15}|[1-9][0-9]{16}|[1-9][0-9]{17}|[1-8][0-9]{18}|9[0-1][0-9]{17}|92[0-1][0-9]{16}|922[0-2][0-9]{15}|9223[0-2][0-9]{14}|92233[0-6][0-9]{13}|922337[0-1][0-9]{12}|92233720[0-2][0-9]{10}|922337203[0-5][0-9]{9}|9223372036[0-7][0-9]{8}|92233720368[0-4][0-9]{7}|922337203685[0-3][0-9]{6}|9223372036854[0-6][0-9]{5}|92233720368547[0-6][0-9]{4}|922337203685477[0-4][0-9]{3}|9223372036854775[0-7][0-9]{2}|922337203685477580[0-6]|9223372036854775807)\})$")

RE_RF = re.compile(r"^WCVERIFY ctx=RF ctl_ok=r=(0|1|2|3|4|5|6|7|8|9|10|11|12|13|14|15|16|17|18|19|20|21|22|23|24|25|26|27|28|29|30|31|32|33|34|35|36|X) ctl_out=r=(0|1|2|3|4|5|6|7|8|9|10|11|12|13|14|15|16|17|18|19|20|21|22|23|24|25|26|27|28|29|30|31|32|33|34|35|36|X) arm=- field=- binder=- body=-$")

RDX_NOCTX = 'READING (frozen table): ctx=RF — all four collapsed at the context build. The boundary is INGEST ADMISSION (native_test_context_from_ingest Rejected). Per-module rows do not exist for this shape.'
RDX_ALLF = 'FROZEN READING: all four F => the boundary is the LANE FILE-REFUSAL policy.'
RDX_ALLA = 'FROZEN READING: all four A => the fold never matched the census entries (census-name/path boundary).'
RDX_FALS = 'FALSIFIER FIRES: producer agrees with all four claims => the boundary is the claim wiring/shape, NOT the producer (H-wc FALSE).'
RDX_X = 'reason matched none of the 37 compared caret symbols (outside-table OR comparison failure); loses which reason, and whether comparison works'
CTL_VOID = 'CONTROL VERDICT: CONTROL FAILED — CLAIM EXPECTATIONS, FALSIFIER AND ALL TABLE-INDEX READINGS ARE VOID (control failed; nothing concluded)'

def ctl_ok_reading(v):
    if v == '0':
        return 'CONTROL HOLDS: the known symbol matched its own arm (0) [name: %s]: caret "==" discriminates' % CENSUS37[0]
    if v == 'X':
        return 'CONTROL VIOLATION: the known symbol matched NOTHING: caret "==" comparison is BROKEN; every r=X in this line is then a comparison failure, not an outside-table reason'
    return 'CONTROL FAILED: the known symbol matched arm %s instead of 0; VOID' % v

def ctl_out_reading(v):
    if v == 'X':
        return 'CONTROL HOLDS: the outside-table symbol matched nothing (X), as declared: the chain does not compare symbols outside its 37 names'
    return 'CONTROL VIOLATION: the outside-table symbol matched arm %s: equality COLLAPSED or the chain compares a symbol outside its 37 names' % v

RD_R = 'WcResolved'; RD_A = 'WcAbsent'; RD_F = 'WcFileRefused'
RD_atom_U = 'atom == ^wc_undeclared_body (the frozen candidate)'
RD_atom_Q = 'atom outside the candidate set'
RD_atom_D = 'atom ABSENT (no locus)'

def tok_reading(v):
    if v in ('R', 'A', 'F'):
        return {'R': RD_R, 'A': RD_A, 'F': RD_F}[v]
    m = re.match(r'^X\{r=([^,]+),a=([^,]+),s=([^,]+),al=([0-9]+)\}$', v)
    if not m:
        return None
    r, a, s, al = m.groups()
    if r == 'X':
        rs = RDX_X
    else:
        _i = int(r)
        _nm = CENSUS37[_i] if 0 <= _i < 37 else 'MISSING-INDEX-IN-TABLE'
        rs = 'reason = table index %s (name from the one 37-name table: %s)' % (r, _nm)
        if _nm == 'MISSING-INDEX-IN-TABLE':
            rs += ' — READER DEFECT: index outside the table'
    at = {'U': RD_atom_U, '?': RD_atom_Q, '-': RD_atom_D}[a]
    sc = 'sole_chain=True (rest of the diagnostic chain Empty)' if s == 'T' else 'sole_chain=False'
    return 'WcRefused{%s; %s; %s; atom_lexeme_len=%s}' % (rs, at, sc, al)

def split_row(r):
    # Six fields in order; the reconstruction via the declared layout must reproduce the row EXACTLY.
    m = re.match(r'^WCVERIFY ctx=OK ctl_ok=r=([^ ]+) ctl_out=r=([^ ]+) arm=(\S+) field=(\S+) binder=(\S+) body=(\S+)$', r)
    if not m:
        return None
    c1, c2, arm, field, binder, body = m.groups()
    rebuilt = ('WCVERIFY ctx=OK ctl_ok=r=%s ctl_out=r=%s arm=%s field=%s binder=%s body=%s'
               % (c1, c2, arm, field, binder, body))
    if rebuilt != r:
        return None
    return c1, c2, arm, field, binder, body

def main():
    if len(sys.argv) != 2:
        print('usage: probe_read.py <file with exactly one WCVERIFY reason line>'); return 2
    try:
        raw = open(sys.argv[1], 'rb').read().decode('utf-8')
    except Exception as e:
        print('REFUSAL: cannot read file: %s' % e); return 2
    if raw.endswith('\n'):
        raw = raw[:-1]
    if '\n' in raw:
        print('REFUSAL: more than one line in the file (the reason is a SINGLE line, and no reason field may contain a newline)'); return 2
    if len(raw) > BOUND:
        print('REFUSAL: over the stated %d-char bound (widest table-composed row); NO OBSERVATION' % BOUND); return 2
    if RE_RF.match(raw):
        m = re.match(r'^WCVERIFY ctx=RF ctl_ok=r=([^ ]+) ctl_out=r=([^ ]+) arm=- field=- binder=- body=-$', raw)
        c1, c2 = m.group(1), m.group(2)
        print('CONTROL (ctx=RF row): ctl_ok=%s -> %s' % (c1, ctl_ok_reading(c1)))
        print('CONTROL (ctx=RF row): ctl_out=%s -> %s' % (c2, ctl_out_reading(c2)))
        print(RDX_NOCTX if (c1 == '0' and c2 == 'X') else CTL_VOID)
        return 0
    if not RE_OK.match(raw):
        print('REFUSAL: does not match the anchored grammar — treat as NO OBSERVATION'); return 2
    sp = split_row(raw)
    if sp is None:
        print('REFUSAL: layout parse failed (keys missing, duplicated, or out of order)'); return 2
    c1, c2, arm, field, binder, body = sp
    print('CONTROL: ctl_ok=%s -> %s' % (c1, ctl_ok_reading(c1)))
    print('CONTROL: ctl_out=%s -> %s' % (c2, ctl_out_reading(c2)))
    if not (c1 == '0' and c2 == 'X'):
        for v in (arm, field, binder, body):
            if tok_reading(v) is None:
                print('REFUSAL: token %s is not a declared form' % v); return 2
        print(CTL_VOID)
        print('Observation only; acceptance never follows from this reading.')
        return 0
    print('CONTROL VERDICT (observation only): caret "==" comparison DISCRIMINATES for the two control symbols.')
    print('CLAIM EXPECTATIONS (the frozen wc claims; the body expectation names r=0, a=U and s=T — al is a datum, not an expectation):')
    print('PER-MODULE READINGS (MIXED tags => read each row on its own; no single-boundary conclusion follows):')
    allf = alla = True
    for name, v in (('arm', arm), ('field', field), ('binder', binder), ('body', body)):
        rd = tok_reading(v)
        if rd is None:
            print('REFUSAL: token %s is not a declared form' % v); return 2
        print('  %-7s %-52s %s' % (name, v, rd))
        if v != 'F': allf = False
        if v != 'A': alla = False
    if allf: print(RDX_ALLF)
    if alla: print(RDX_ALLA)
    ok = {'arm': arm == 'R', 'field': field == 'R', 'binder': binder == 'R',
          'body': bool(re.match(r'^X\{r=0,a=U,s=T,al=[0-9]+\}$', body))}
    body_expect = 'expect X with r=0, a=U and s=T (sole unbound refusal at ^wc_undeclared_body)'
    for name, v, e in (('arm', arm, 'expect R'), ('field', field, 'expect R'), ('binder', binder, 'expect R'), ('body', body, body_expect)):
        print('  %-7s %-47s observed %-44s %s' % (name, e, v, 'MATCH' if ok[name] else 'MISMATCH'))
    if all(ok.values()): print(RDX_FALS)
    print('Observation only; acceptance never follows from this reading.')
    return 0

sys.exit(main())
PROBE_READ_EOF
probe_read_offline() { if command -v python3 >/dev/null 2>&1; then python3 "$RS/.probe_read-src.py" "$1"; else echo "READER_PY=absent; use $R/.probe_read.py offline"; return 3; fi; }

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

# (7b) CENSUS37 setup self-checks (the one table; fail closed before any fetch).
[ "${#CENSUS37[@]}" = 37 ] || die2 "REFUSED_SETUP: CENSUS37 is not 37 entries"
[ "$(printf '%s\n' "${CENSUS37[@]}" | sort -u | wc -l)" = 37 ] || die2 "REFUSED_SETUP: CENSUS37 has duplicates"
for _n in "${CENSUS37[@]}"; do [[ "$_n" =~ ^resolve_reason_[a-z0-9_]{1,64}$ ]] || die2 "REFUSED_SETUP: CENSUS37 name outside the closed class: $_n"; done
[ "${CENSUS37[0]}" = resolve_reason_unbound_symbol ] || die2 "REFUSED_SETUP: CENSUS37[0] is not the ctl_ok symbol"
[ "${CENSUS37[36]}" = resolve_reason_type_parameter_order_malformed ] || die2 "REFUSED_SETUP: CENSUS37[36] wrong"
[ -z "${CENSUS37[37]+x}" ] || die2 "REFUSED_SETUP: CENSUS37 has an index 37"
if printf '%s\n' "${CENSUS37[@]}" | grep -qx resolve_reason_qualified_target_identity_unrepresentable; then die2 "REFUSED_SETUP: the excluded qualified arm is in CENSUS37"; fi
if printf '%s\n' "${CENSUS37[@]}" | grep -qx wc_undeclared_body; then die2 "REFUSED_SETUP: the ctl_out symbol is inside the 37"; fi
echo "CENSUS37: 37 unique, class-clean, [0]=unbound_symbol, [36]=type_parameter_order_malformed, qualified excluded, ctl_out symbol outside"

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
BADTOK="X{r=37,a=U,s=T,al=1}"
[ "$(pv_line "$(probe_row_ok 0 X "$BADTOK" R R R)")" = 1 ] || die2 "REFUSED_SETUP: grammar fixture (reason index 37, outside 0..36) accepted"
[ "$(pv_line "$(probe_row_ok 0 X "X{r=07,a=U,s=T,al=1}" R R R)")" = 1 ] || die2 "REFUSED_SETUP: grammar fixture (r=07 leading zero) accepted"
[ "$(pv_line "$(probe_row_ok 0 X "X{r=0,a=U,s=T,al=01}" R R R)")" = 1 ] || die2 "REFUSED_SETUP: grammar fixture (al=01 leading zero) accepted"
[ "$(pv_line "$(probe_row_ok 0 X X R R R)")" = 1 ] || die2 "REFUSED_SETUP: grammar fixture (bare tag X) accepted"
[ "$(pv_line 'WCVERIFY ctx=RF')" = 1 ] || die2 "REFUSED_SETUP: grammar fixture (bare RF) accepted"
# CONTROL-field fixtures: missing, out of order, illegal values, bare form
[ "$(pv_line "WCVERIFY ctx=OK ctl_out=r=X arm=R field=R binder=R body=R")" = 1 ] || die2 "REFUSED_SETUP: a row without ctl_ok was accepted"
[ "$(pv_line "WCVERIFY ctx=RF ctl_out=r=X arm=- field=- binder=- body=-")" = 1 ] || die2 "REFUSED_SETUP: an RF row without ctl_ok was accepted"
[ "$(pv_line "WCVERIFY ctx=OK ctl_out=r=X ctl_ok=r=0 arm=R field=R binder=R body=R")" = 1 ] || die2 "REFUSED_SETUP: ctl fields out of order were accepted"
[ "$(pv_line "$(probe_row_ok 37 X R R R R)")" = 1 ] || die2 "REFUSED_SETUP: ctl_ok=r=37 was accepted"
[ "$(pv_line "$(probe_row_ok 0 37 R R R R)")" = 1 ] || die2 "REFUSED_SETUP: ctl_out=r=37 was accepted"
[ "$(pv_line "$(probe_row_ok 38 X R R R R)")" = 1 ] || die2 "REFUSED_SETUP: ctl_ok=r=38 was accepted"
[ "$(pv_line "$(probe_row_ok 07 X R R R R)")" = 1 ] || die2 "REFUSED_SETUP: leading-zero r=07 was accepted"
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
# the v8 grammar it is refused (no control fields), which is exactly why the v5 run was NO_OBSERVATION.
V5LINE='WCVERIFY ctx=OK arm=X{r=X,a=-,s=T,al=0} field=X{r=X,a=-,s=T,al=0} binder=X{r=X,a=-,s=T,al=0} body=X{r=X,a=-,s=T,al=0}'
[ "$(pv_line "$V5LINE")" = 1 ] || die2 "REFUSED_SETUP: the v5 observed-bytes row (no control fields) must be refused under the v8 grammar"
printf '%s\n' "$V5LINE" > "$PGD/observed-v5.txt"

