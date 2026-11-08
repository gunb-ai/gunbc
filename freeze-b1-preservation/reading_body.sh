# ---- ONE AUTHORITY TABLE (v6, parent msg_b8f24de9) -------------------------------------------
# The declared-form set, the grammar, and the reading table are generated from THESE lines and
# nothing else. Each alternative is a PLAIN token emitted verbatim into the row layout composed by
# probe_row_ok / probe_row_rf / probe_token; the grammar below is composed from the SAME constants.
# A form the producer cannot emit must not appear here; a form the producer can emit must appear here.
# Producer value set, read off the frozen probe source at the seed (blob bb6ded7e, file sha256
# 5ea381d721620cf9e475262bb2dfcb05821036d4dd0400a8cde0ffa5c4c5dd45), quoted in the v6 report:
#   verdict_text: WcResolved->'R'  WcAbsent->'A'  WcFileRefused->'F'
#     WcRefused{reason,at_atom,sole_chain}-> 'X' '=' reason_token ',' 'a' '=' atom_token ',' 's' '='
#       bool_token ',' 'al' '=' int_to_decimal_string '}'
#   reason_token: 24 equality arms against ^resolve_reason_* (the 24 names listed in the report),
#     else -> 'r=X'. The lane vocabulary at the seed is 37 DISTINCT ^resolve_reason_* reference-site
#     symbols across src/v2/compiler/resolve.dag + src/v2/compiler/name_resolve.dag; 14 of the 37 are
#     OUTSIDE the probe's 24, so r=X is REACHABLE by construction. One compared symbol
#     (resolve_reason_qualified_target_identity_unrepresentable) occurs only in comment lines at those
#     files: its r=23 arm is INERT to any reason the reference-site census can produce.
#   atom_token: Present{value: ==^wc_undeclared_body}->'U' ; Present otherwise->'?' ; Absent->'-'
#   bool_token: 'T' | 'F'.  al: int_to_decimal_string over the scalar char count (canonical decimal,
#     no leading zero, at most 19 digits by the i64 bound).
# 'ctx' is a field in its own right; its two alternatives are the two row shapes.
TT_ctx='RF|OK'
TT_r='0|1|2|3|4|5|6|7|8|9|10|11|12|13|14|15|16|17|18|19|20|21|22|23|X'
TT_a='U|?|-'
TT_s='T|F'
# al: canonical decimal, 1..19 digits, no leading zero; '0' is the only zero form.
TT_al_FORM='0|[1-9][0-9]{0,18}'
TT_al_MAX='1234567890123456789'
# Layout constants, single authority. Key labels WITHOUT '=': the producer composition and the
# grammar BOTH append '$EQ', so a key can appear exactly once and cannot drift between them.
SEP_arm=' arm'; SEP_field=' field'; SEP_binder=' binder'; SEP_body=' body'
SEP_r='r'; SEP_a=',a'; SEP_s=',s'; SEP_al=',al'
TOK_OPEN='X'; TOK_IN='{'; TOK_CLOSE='}'
EQ='='
CTX_PRE='WCVERIFY ctx='
CTX_OK="${CTX_PRE}OK"; CTX_RF="${CTX_PRE}RF"
# probe_token_r R -> 'r=<alt>' for a declared alternative only.
probe_token_r() { local v; case "|$TT_r|" in *"|$1|"*) printf 'r%s%s' "$EQ" "$1"; return 0;; esac; return 1; }
# probe_token R A S AL -> the full verdict token in declared layout; refuses non-declared alternatives.
probe_token() {
  local r="$1" a="$2" s="$3" al="$4" t
  case "|$TT_r|" in *"|$r|"*) :;; *) return 1;; esac
  case "|$TT_a|" in *"|$a|"*) :;; *) return 1;; esac
  case "|$TT_s|" in *"|$s|"*) :;; *) return 1;; esac
  [[ "$al" =~ ^(${TT_al_FORM})$ ]] || return 1
  [ "${#al}" -le 19 ] || return 1
  t="$SEP_r$EQ$r$SEP_a$EQ$a$SEP_s$EQ$s$SEP_al$EQ$al"
  printf '%s' "$TOK_OPEN$TOK_IN$t$TOK_CLOSE"
}
# probe_row_ok T1 T2 T3 T4 / probe_row_rf -> complete reason strings from the layout table only.
probe_row_ok() { printf '%s%s%s%s%s%s%s%s%s%s' "$CTX_OK" "" "$SEP_arm" "$EQ$1" "$SEP_field" "$EQ$2" "$SEP_binder" "$EQ$3" "$SEP_body" "$EQ$4"; }
probe_row_rf() { printf '%s%s%s%s-%s%s-%s%s-%s%s-' "$CTX_PRE" "RF" "$SEP_arm" "$EQ" "$SEP_field" "$EQ" "$SEP_binder" "$EQ" "$SEP_body" "$EQ"; }
# The grammar, DERIVED from the same constants. Field order fixed by the layout; every alternative is
# the verbatim token, so any row that validates is byte-composable from the table alone.
PG_PLAIN='(R|A|F)'
PG_X="${TOK_OPEN}\\{${SEP_r}${EQ}(0|[1-9]|1[0-9]|2[0-3]|X)${SEP_a}${EQ}(U|\\?|-)${SEP_s}${EQ}(T|F)${SEP_al}${EQ}(${TT_al_FORM})\\}"
PG_V="(${PG_PLAIN}|${PG_X})"
PG_OK="^${CTX_OK}${SEP_arm}${EQ}${PG_V}${SEP_field}${EQ}${PG_V}${SEP_binder}${EQ}${PG_V}${SEP_body}${EQ}${PG_V}\$"
PG_RF="^${CTX_RF}${SEP_arm}${EQ}-${SEP_field}${EQ}-${SEP_binder}${EQ}-${SEP_body}${EQ}-\$"
# probe_validate REASON -> 0 accepted / 1 refused. Length first, then the anchored grammar.
probe_validate() {
  local r="$1"
  [ "${#r}" -le 193 ] || return 1
  if [[ "$r" =~ $PG_RF ]]; then return 0; fi
  if [[ "$r" =~ $PG_OK ]]; then return 0; fi
  return 1
}
# ---- the READING TABLE, same tables (frozen per-module rows; parent msg_bf5a7d3c + msg_b8f24de9) ----
RD_R='WcResolved'; RD_A='WcAbsent'; RD_F='WcFileRefused'
RD_atom_U='atom == ^wc_undeclared_body (the frozen candidate)'; RD_atom_Q='atom outside the candidate set'; RD_atom_D='atom ABSENT (no locus)'
RDX_NOCTX='READING (frozen table): ctx=RF — all four collapsed at the context build. The boundary is INGEST ADMISSION (native_test_context_from_ingest Rejected). Per-module rows do not exist for this shape.'
RDX_ALLF='FROZEN READING: all four F => the boundary is the LANE FILE-REFUSAL policy.'
RDX_ALLA='FROZEN READING: all four A => the fold never matched the census entries (census-name/path boundary).'
RDX_FALS='FALSIFIER FIRES: producer agrees with all four claims => the boundary is the claim wiring/shape, NOT the producer (H-wc FALSE).'
# probe_parse_token TOKEN RVAR AVAR SVAR ALVAR -> 0 and sets the four variables; refuses (1) on any
# token that is not composed of declared alternatives with the declared separators, in order, once each.
probe_parse_token() {
  local v="$1" __r __a __s __al rest
  local -n __R="$2" __A="$3" __S="$4" __AL="$5"
  case "$v" in
    R) __r='__plain'; __a='R'; __s='__tag'; __al='R';;
    A) __r='__plain'; __a='A'; __s='__tag'; __al='A';;
    F) __r='__plain'; __a='F'; __s='__tag'; __al='F';;
    "$TOK_OPEN$TOK_IN"*)
      rest="${v#"$TOK_OPEN$TOK_IN"}"
      [ "$rest" != "${v%"$TOK_CLOSE"}" ] || return 1            # must end with the single TOK_CLOSE
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
      case "|$TT_r|" in *"|$__r|"*) :;; *) return 1;; esac
      case "|$TT_a|" in *"|$__a|"*) :;; *) return 1;; esac
      case "|$TT_s|" in *"|$__s|"*) :;; *) return 1;; esac
      [[ "$__al" =~ ^(${TT_al_FORM})$ ]] || return 1
      [ "${#__al}" -le 19 ] || return 1;;
    *) return 1;;
  esac
  if [ "$__r" = __plain ]; then __R="$__al"; __A=''; __S=''; __AL=''; else __R="$__r"; __A="$__a"; __S="$__s"; __AL="$__al"; fi
  return 0
}
# probe_tok_reading TOKEN -> the frozen reading of one verdict token; refuses (1) on an illegal token.
probe_tok_reading() {
  local v="$1" r a s al
  case "$v" in R|A|F) printf '%s\n' "$(case "$v" in R) echo "$RD_R";; A) echo "$RD_A";; *) echo "$RD_F";; esac)"; return 0;; esac
  probe_parse_token "$v" r a s al || return 1
  local rs; if [ "$r" = X ]; then rs='reason OUTSIDE the 24-symbol comparison table (an ^resolve_reason_* symbol the probe compares nothing against)'; else rs="reason = table index $r (the 24-name resolve_reason list)"; fi
  local at; case "$a" in U) at="$RD_atom_U";; \?) at="$RD_atom_Q";; -) at="$RD_atom_D";; *) return 1;; esac
  local sc; if [ "$s" = T ]; then sc='sole_chain=True (rest of the diagnostic chain Empty)'; else sc='sole_chain=False'; fi
  printf 'WcRefused{%s; %s; %s; atom_lexeme_len=%s}\n' "$rs" "$at" "$sc" "$al"
}
# probe_strip REST KEY -> the token that follows 'KEY=' up to the next key or end of line.
# Field splitting is delimiter-based (no prefix/suffix pattern removals): the grammar anchors the
# full layout, so each key occurs EXACTLY once and in order; splitting on the keys is unambiguous
# and byte-exact. probe_split_ok ROW A1 A2 A3 A4 fills the four token variables (and returns 1 on a
# missing key or wrong count).
probe_split_ok() {
  local r="$1" line
  local -n _A1="$2" _A2="$3" _A3="$4" _A4="$5"
  case "$r" in "$CTX_OK"*) :;; *) return 1;; esac
  # shellcheck disable=SC2206
  local parts=()
  # split on the four keys, keeping only the pieces between them
  local tmp="$r" p
  for p in "$SEP_arm" "$SEP_field" "$SEP_binder"; do
    case "$tmp" in *"$p$EQ"*) :;; *) return 1;; esac
  done
  line="$(printf '%s\n' "$r" | sed -e "s/^$CTX_OK$SEP_arm$EQ//" -e "s/$SEP_field$EQ/\n/" -e "s/$SEP_binder$EQ/\n/" -e "s/$SEP_body$EQ/\n/" | tr -d ' ')"
  [ "$(printf '%s\n' "$line" | wc -l)" = 4 ] || return 1
  _A1="$(printf '%s\n' "$line" | sed -n 1p)"
  _A2="$(printf '%s\n' "$line" | sed -n 2p)"
  _A3="$(printf '%s\n' "$line" | sed -n 3p)"
  _A4="$(printf '%s\n' "$line" | sed -n 4p)"
  # the reconstruction must reproduce the original row byte-for-byte (layout proof, one authority)
  local rebuilt; rebuilt="$(probe_row_ok "$_A1" "$_A2" "$_A3" "$_A4")"
  [ "$rebuilt" = "$r" ] || return 1
  return 0
}
# probe_read_line REASON: the reading-path lookup, mirroring probe_read.py (shipped in the packet and
# sha-gate-checked below). Observation only; it never implies acceptance.
probe_read_line() {
  local r="$1"
  if [ "${#r}" -gt 193 ]; then echo 'REFUSAL: over the stated 193-char bound; NO OBSERVATION'; return 2; fi
  if [[ "$r" =~ $PG_RF ]]; then echo "$RDX_NOCTX"; return 0; fi
  if ! [[ "$r" =~ $PG_OK ]]; then echo 'REFUSAL: does not match the anchored grammar — treat as NO OBSERVATION'; return 2; fi
  local arm field binder body f v rd allf=1 alla=1
  probe_split_ok "$r" arm field binder body || { echo 'REFUSAL: layout parse failed (keys missing, duplicated, or out of order)'; return 2; }
  echo 'PER-MODULE READINGS (MIXED tags => read each row on its own; no single-boundary conclusion follows):'
  for f in arm field binder body; do
    v="${!f}"
    rd="$(probe_tok_reading "$v")" || { echo "REFUSAL: illegal token in field $f: $v"; return 2; }
    printf '  %-7s %-44s -> %s\n' "$f" "$v" "$rd"
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
