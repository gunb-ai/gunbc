# wc-probe-v7 — v6->v7 diff (1/3)

Frozen packet 258449f32d82f63f6b68315def188ae3be25e01f55fbba5058171524cef6575e vs v6 b070af6b4a5b292186d4b634c9e144fe3b76f25c633a823e632d3364634b117b. 476 insertions, 236 deletions, one file.

```diff
diff --git a/packet-wc-probe-v6.sh b/packet-wc-probe-v7.sh
@@ -1,5 +1,31 @@
 #!/bin/bash
-# FROZEN wc-verdict diagnostic packet wc-probe-v6 (author sleek-seal-226, #15695 lane; parent draft approval msg_bf5a7d3c).
+# FROZEN wc-verdict diagnostic packet wc-probe-v7 (author sleek-seal-226, #15695 lane; parent drafts msg_bf5a7d3c / msg_b8f24de9 / msg_bf3d2085 / msg_32d414bd).
+# v7 (v6->v7; parent msg_bf3d2085 + addendum msg_32d414bd):
+# - B1 FIXED: the reader source is written to $RS (a separate mktemp dir) BEFORE any cleanup and copied
+#   to $R/.probe_read.py from there; the pre-fetch path now reaches probe_leg (v6 died at REFUSED_SETUP
+#   because $FX was rm -rf'd at the fixture tail before the copy). An END-TO-END pre-fetch run (cgroup
+#   files shimmed, fetch stubbed with a marker) is posted with this packet and reaches the fetch stub.
+# - B2 FIXED: in-probe CONTROL fields. The probe now emits ' ctl_ok=r=<v> ctl_out=r=<v>' (seed 312fd6dd):
+#   ctl_ok = reason_token(^resolve_reason_unbound_symbol) (declared 0), ctl_out =
+#   reason_token(^resolve_reason_binder_duplicate_in_frame) (outside the 24; declared X). r=X means ONLY:
+#   'matched none of the 24 compared caret symbols (outside-table OR comparison failure); loses which
+#   reason, and whether comparison works'. The control settles whether caret '==' discriminates AT ALL.
+# - The bound is table-composed (widest row = 218; widest token 38). Check kinds BOUND/GRAMMAR/CONSISTENCY
+#   are distinct and asserted. al is bounded by the i64 max via a table-generated staircase regex; the
+#   a=- implies al=0 rule is a CONSISTENCY refusal. PG_RF is NOT byte-identical to v5's (it carries the
+#   control fields); PG text is generated from the ONE table and the embedded reader constants are
+#   asserted BYTE-IDENTICAL to them (one authority, enforced at setup).
+# - OBSERVED BYTES ONLY: the v5 run's single typed line (raw claims-A-probe.log, sha256
+#   6315352c64af90fe105a337cc70222cc1f030b465860efafb996a6fe7efda56e) is retained as bytes and as a
+#   REFUSED fixture (it lacks the control fields); it is NOT a reading and nothing about wc follows from it.
+# - ESCAPING/NEWLINE: no field text can contain a newline, ')' or any brace/comma outside the fixed
+#   payload positions (every glyph is a fixed literal in the probe file or a digit from decimal_digit_glyph,
+#   dag/std/nat.dag:219-231; the '-' branch of int_to_decimal_string is unreachable for a fold count).
+#   Malformed-newline and two-outcome-line fixtures pass through BOTH the real validator and the runtime
+#   caller and fail closed.
+# - UNVERIFIED (stated, not assumed): whether Symbol '==' against caret literals discriminates at
+#   f08a6ae4 (the control settles it); the 37/24/14 reference-site census; whether symbol_lexeme includes
+#   the caret (affects al only — the payload is counted, never rendered).
 # PURPOSE: ONE observation — WHICH WcVerdict each of the four wildcard_arm_resolve fixtures actually takes at seed A. The v4 join
 #   measured 0/4 BARE FAILs for those claims at BOTH pins (claims-A-wc.log sha db53f7b0, claims-B-wc.log sha cee20bfd); bare verdict
 #   lines carry no cause, so the WcVerdict tag (WcResolved | WcAbsent | WcFileRefused | WcRefused{reason,at_atom,sole_chain} |
@@ -8,15 +34,16 @@
 #   'FAIL <fn> (returned ProcessExit::ExitFailure, code <n>: <reason>)'). NO binary change, NO observer change, NO copied fixture
 #   text: the probe entry imports wc_outcomes from the existing claim module, so the inputs are the same bytes by construction.
 # SEED: branch session/sleek-seal-226-wc-probe, ONE commit above main-side A. Its ONLY change from f08a6ae4 is
-#   src/v2/test/claim/namespace_xl0/wc_verdict_probe.dag (blob bb6ded7e6533df101e3343dac604ba8dffebdcf4). Gated below: parent commit,
+#   src/v2/test/claim/namespace_xl0/wc_verdict_probe.dag (blob 0a2f577ae6ad25247e8ed5a93b4c10063e61ff03). Gated below: parent commit,
 #   tree, and a name-only diff of exactly one path. The frozen claim file is gated UNCHANGED at its blob
 #   aba6d2be15f1510629d7d8f8d47267e162bf2047 (byte-identical at f08a6ae4 and at the probe commit).
 #   A = f08a6ae4e44dc0636eda71f25f0009e97ffde1d1  tree 2549a8e6545468a55fd8de3d0a082049bae19b32 (the v4a leg-A pin)
-#   P = bce90e1396d767093aee240c2e4a4623fba64113  tree c911d73665d0549bfcc86398c53c83d2ba78b4bb (A + the one probe file)
+#   P = 312fd6ddae2ccea13eb9953a6e3a4757172c73b1  tree a17db83d4c7003725cba6039f1150bf0b7322711 (bce90e13 + the ctl control fields)
 # PARSE-DEFECT NOTE (v5, parent msg_ec43981d): the probe .dag parse defect ('wc_verdict_probe.dag:83:12: expected FatArrow,
 #   found Arrow') was LATENT in v3 (masked: the missing --functions refusal preceded entry parsing) and EXPOSED in v4 (the
 #   entry parse during --functions validation refused it; raw log sha256 ef780ebc6cd9c2a37bf0061db94a62a12183ab7ea89d55d07c20957590227d70).
-#   It is FIXED AT THE SEED (six match arms '->' -> '=>'; blob bb6ded7e6533df101e3343dac604ba8dffebdcf4), evidenced by the
+#   It is FIXED AT THE SEED (six match arms '->' -> '=>' at bce90e13; ctl fields added at 312fd6dd;
+#   blob 0a2f577ae6ad25247e8ed5a93b4c10063e61ff03), evidenced by the
 #   exhaustive pre-freeze syntax audit vs the pinned parser, PROBE_DAG_AUDIT.md sha256 59402390bfbf5f38d333bd3c106f1798b915866c688acbce9e71ebefb7430ed0
 #   (posted as a PR comment). No local gunbc run was used anywhere in that audit.
 # GRAMMAR-MISMATCH NOTE (v6, parent msg_b8f24de9): v5 was SPENT on a validator/grammar mismatch. The emitted line
@@ -80,10 +107,10 @@
 # MEASUREMENT ONLY: no acceptance follows from exit 0; this packet judges no claim and changes no frozen expectation from the v4
 #   join (v4a 265609d4..., v4b 8a2bd47e...), which stand as measured.
 set -uo pipefail
-JOB=sleek-seal-226-wc-probe-v6
+JOB=sleek-seal-226-wc-probe-v7
 UNIT="$JOB.scope"
 PIN_A=f08a6ae4e44dc0636eda71f25f0009e97ffde1d1; TREE_A=2549a8e6545468a55fd8de3d0a082049bae19b32
-PIN_P=bce90e1396d767093aee240c2e4a4623fba64113; TREE_P=c911d73665d0549bfcc86398c53c83d2ba78b4bb; BLOB_PROBE=bb6ded7e6533df101e3343dac604ba8dffebdcf4; BLOB_WC=aba6d2be15f1510629d7d8f8d47267e162bf2047
+PIN_P=312fd6ddae2ccea13eb9953a6e3a4757172c73b1; TREE_P=a17db83d4c7003725cba6039f1150bf0b7322711; BLOB_PROBE=0a2f577ae6ad25247e8ed5a93b4c10063e61ff03; BLOB_WC=aba6d2be15f1510629d7d8f8d47267e162bf2047
 die2() { echo "NO OBSERVATION (exit 2): $*"; exit 2; }
 # F3 scope gate: exactly one cgroup record, matched against a LITERAL suffix with a case glob (the quoted part is literal; no regex, no unescaped dot)
 scope_ok() { case "$1" in 0::*/"$UNIT") return 0;; *) return 1;; esac; }
@@ -139,6 +166,7 @@ tree_clean() { local o; o="$(git -C "$1" status --porcelain 2>/dev/null)" || ret
 REAL_COMPLETION='[process-partition] total_ms=340188 pre_run_ms=0 args_ms=22 index_build_ms=154362 edge_index_warm_ms=35617 output_policy_ms=12509 roster_ms=0 published_mock_precompute_ms=1761 entries_ms=135911 reporting_ms=2 entries_resolve_ms=113955 entries_witness_ms=20622'
 [ "$(printf %s "$REAL_COMPLETION" | sha256sum | cut -d' ' -f1)" = "7ac24cf28973511b7902ce3ba621e8c46b060ad7855ea895b1f5a157bf1835b2" ] || die2 "REFUSED_SETUP: the real-line fixture is not the verbatim bytes"
 FX="$(mktemp -d)" || die2 "mktemp"
+RS="$(mktemp -d)" || die2 "mktemp"   # reader source lives OUTSIDE $FX (parent msg_bf3d2085 B1: survives the fixture cleanup)
 r() { if "$@"; then echo accepted; else echo refused; fi; }
 # (1) parser
 r_ev() { oomread "$1" oom >/dev/null && oomread "$1" oom_kill >/dev/null; }
@@ -160,7 +188,7 @@ printf 'PASS x\n%s\n[process-partition-exit] run_returned_ms=340188 main_end_ms=
 # (3) exit contract
 [ "$(r rc_in_contract 0) $(r rc_in_contract 1) $(r rc_in_contract 2) $(r rc_in_contract 124) $(r rc_in_contract 137) $(r rc_in_contract 3)" = "accepted accepted accepted refused refused refused" ] || die2 "REFUSED_SETUP: exit-contract fixture misfire"
 # (4) scope glob literal suffix, restore cleanliness
-[ "$(r scope_ok '0::/a/sleek-seal-226-wc-probe-v6.scope') $(r scope_ok '0::/a/sleek-seal-226-wc-probe-v5Xscope') $(r scope_ok '0::/a/xsleek-seal-226-wc-probe-v6.scope') $(r scope_ok '0::/a/sleek-seal-226-wc-probe-v6.scope.x')" = "accepted refused refused refused" ] || die2 "REFUSED_SETUP: scope fixture misfire"
+[ "$(r scope_ok '0::/a/sleek-seal-226-wc-probe-v7.scope') $(r scope_ok '0::/a/sleek-seal-226-wc-probe-v5Xscope') $(r scope_ok '0::/a/xsleek-seal-226-wc-probe-v7.scope') $(r scope_ok '0::/a/sleek-seal-226-wc-probe-v7.scope.x')" = "accepted refused refused refused" ] || die2 "REFUSED_SETUP: scope fixture misfire"
 git init -q "$FX/g" && : > "$FX/g/f" || die2 "fixture git init"
 git -C "$FX/g" add f && git -C "$FX/g" -c user.name=fx -c user.email=fx@fx commit -qm fx || die2 "fixture commit"
 A="$(r tree_clean $FX/g)"; : > "$FX/g/u"; B="$(r tree_clean $FX/g)"
@@ -202,80 +230,152 @@ probe_classify() {
 # reader all derive from these constants, and fixtures assert the agreement. v5 died because a hand-written
 # grammar and a hand-written header disagreed; composed from one table they cannot.
 # ---- ONE AUTHORITY TABLE (v6, parent msg_b8f24de9) -------------------------------------------
-# The declared-form set, the grammar, and the reading table are generated from THESE lines and
-# nothing else. Each alternative is a PLAIN token emitted verbatim into the row layout composed by
-# probe_row_ok / probe_row_rf / probe_token; the grammar below is composed from the SAME constants.
-# A form the producer cannot emit must not appear here; a form the producer can emit must appear here.
-# Producer value set, read off the frozen probe source at the seed (blob bb6ded7e, file sha256
-# 5ea381d721620cf9e475262bb2dfcb05821036d4dd0400a8cde0ffa5c4c5dd45), quoted in the v6 report:
+# ---- THE ONE AUTHORITY TABLE (v7; parent msg_bf3d2085 + msg_b8f24de9 + msg_32d414bd) ----
+# The grammar, the row composition, and the reading lookup are GENERATED from the constants below and
+# nothing else. The python reader's two regex constants are asserted byte-identical to the generated
+# grammars at setup (cross-check fixtures below), so there is exactly one textual authority: these lines.
+# Producer-side vocabulary, quoted from the probe source at the seed (wc_verdict_probe.dag, blob 0a2f577a,
+# file sha256 0810abe3150a44d6f55f41d704fdd7e0ad418d82dee368c97154931636bade22), quoted in the v7 report:
 #   verdict_text: WcResolved->'R'  WcAbsent->'A'  WcFileRefused->'F'
-#     WcRefused{reason,at_atom,sole_chain}-> 'X' '=' reason_token ',' 'a' '=' atom_token ',' 's' '='
+#     WcRefused{reason,at_atom,sole_chain}-> 'X' '{' reason_token ',' 'a' '=' atom_token ',' 's' '='
 #       bool_token ',' 'al' '=' int_to_decimal_string '}'
+#   ctl_text (v7 control): ' ctl_ok=' reason_token(^resolve_reason_unbound_symbol) ' ctl_out='
+#     reason_token(^resolve_reason_binder_duplicate_in_frame)   [computed UNCONDITIONALLY, both arms]
 #   reason_token: 24 equality arms against ^resolve_reason_* (the 24 names listed in the report),
 #     else -> 'r=X'. The lane vocabulary at the seed is 37 DISTINCT ^resolve_reason_* reference-site
-#     symbols across src/v2/compiler/resolve.dag + src/v2/compiler/name_resolve.dag; 14 of the 37 are
-#     OUTSIDE the probe's 24, so r=X is REACHABLE by construction. One compared symbol
-#     (resolve_reason_qualified_target_identity_unrepresentable) occurs only in comment lines at those
-#     files: its r=23 arm is INERT to any reason the reference-site census can produce.
-#   atom_token: Present{value: ==^wc_undeclared_body}->'U' ; Present otherwise->'?' ; Absent->'-'
-#   bool_token: 'T' | 'F'.  al: int_to_decimal_string over the scalar char count (canonical decimal,
-#     no leading zero, at most 19 digits by the i64 bound).
-# 'ctx' is a field in its own right; its two alternatives are the two row shapes.
-TT_ctx='RF|OK'
-TT_r='0|1|2|3|4|5|6|7|8|9|10|11|12|13|14|15|16|17|18|19|20|21|22|23|X'
-TT_a='U|?|-'
-TT_s='T|F'
-# al: canonical decimal, 1..19 digits, no leading zero; '0' is the only zero form.
-TT_al_FORM='0|[1-9][0-9]{0,18}'
-TT_al_MAX='1234567890123456789'
-# Layout constants, single authority. Key labels WITHOUT '=': the producer composition and the
-# grammar BOTH append '$EQ', so a key can appear exactly once and cannot drift between them.
+#     symbols (resolve.dag + name_resolve.dag, comment lines excluded), so the chain has 13+ arms with
+#     no candidate and 'r=X' is REACHABLE BY CONSTRUCTION. What 'r=X' means (parent msg_02bc6f7c B2,
+#     required wording): 'matched none of the 24 compared caret symbols (outside-table OR comparison
+#     failure); loses which reason, and whether comparison works'. It is NOT evidence of an outside-table
+#     reason. Whether Symbol '==' against caret literals discriminates at f08a6ae4 is UNVERIFIED; the
+#     ctl_ok/ctl_out control fields settle it (see PROBE_CTL_READINGS). UNVERIFIED also: the 37/24/14
+#     census itself, and whether symbol_lexeme includes the caret (it affects al; the payload is counted,
+#     never rendered, so only the COUNT is observable).
+EQ='='; CTX_PRE='WCVERIFY ctx='; CTX_OK="${CTX_PRE}OK"; CTX_RF="${CTX_PRE}RF"
+SEP_ctl_ok=' ctl_ok'; SEP_ctl_out=' ctl_out'; TOK_CTL='r='
 SEP_arm=' arm'; SEP_field=' field'; SEP_binder=' binder'; SEP_body=' body'
-SEP_r='r'; SEP_a=',a'; SEP_s=',s'; SEP_al=',al'
-TOK_OPEN='X'; TOK_IN='{'; TOK_CLOSE='}'
-EQ='='
-CTX_PRE='WCVERIFY ctx='
-CTX_OK="${CTX_PRE}OK"; CTX_RF="${CTX_PRE}RF"
-# probe_token_r R -> 'r=<alt>' for a declared alternative only.
-probe_token_r() { local v; case "|$TT_r|" in *"|$1|"*) printf 'r%s%s' "$EQ" "$1"; return 0;; esac; return 1; }
-# probe_token R A S AL -> the full verdict token in declared layout; refuses non-declared alternatives.
+SEP_r='r'; SEP_a=',a'; SEP_s=',s'; SEP_al=',al'; TOK_OPEN='X'; TOK_IN='{'; TOK_CLOSE='}'
+TT_plain='R|A|F'                                     # verdict_text plain tags (R | A | F)
+TT_r='0|1|2|3|4|5|6|7|8|9|10|11|12|13|14|15|16|17|18|19|20|21|22|23|X'   # reason_token, 24 arms + X
+TT_ctl="$TT_r"                                       # ctl tokens: SAME forms (generated, one source)
+TT_a='U|?|-'                                         # atom_token
+TT_s='T|F'                                           # bool_token
+TT_al_BOUND='9223372036854775807'                    # i64 max: int_to_decimal_string can never exceed it
+# The al alternation is GENERATED from TT_al_BOUND as a lexicographic staircase: '0', every canonical
+# decimal of length 1..n-1, then same-length prefixes below the bound digit by digit, then the bound
+# itself. Canonical decimals only (no leading zero; no '-'): the producer emits digits only
+# (dag/std/nat.dag:219-231 decimal_digit_glyph; the '-' branch of integer.dag:316-335 is unreachable
+# because atom_len is a fold_list count, never negative).
+regex_le() {
+  local B="$1" n="${#1}" i d pre cls out=('0')
+  for ((i=1;i<n;i++)); do
+    if [ "$i" = 1 ]; then out+=('[1-9]'); else out+=("[1-9][0-9]{$((i-1))}"); fi
+  done
+  for ((i=0;i<n;i++)); do
+    d="${B:i:1}"
+    [ "$d" = 0 ] && continue
+    pre="${B:0:i}"
+    if [ "$i" = 0 ]; then cls="[1-$((d-1))]"; else cls="[0-$((d-1))]"; fi
+    if [ "$i" = $((n-1)) ]; then out+=("$pre$cls"); else out+=("$pre$cls[0-9]{$((n-i-1))}"); fi
+  done
+  out+=("$B")
+  local IFS='|'; printf '%s' "${out[*]}"
+}
+TT_al_FORM="$(regex_le "$TT_al_BOUND")"
+RD_R='WcResolved'; RD_A='WcAbsent'; RD_F='WcFileRefused'
+RD_atom_U='atom == ^wc_undeclared_body (the frozen candidate)'; RD_atom_Q='atom outside the candidate set'; RD_atom_D='atom ABSENT (no locus)'
+# Frozen reading table (parent msg_bf5a7d3c); 'observation only' lines are DATA, never acceptance.
+RDX_NOCTX='READING (frozen table): ctx=RF — all four collapsed at the context build. The boundary is INGEST ADMISSION (native_test_context_from_ingest Rejected). Per-module rows do not exist for this shape.'
+RDX_ALLF='FROZEN READING: all four F => the boundary is the LANE FILE-REFUSAL policy.'
+RDX_ALLA='FROZEN READING: all four A => the fold never matched the census entries (census-name/path boundary).'
+RDX_FALS='FALSIFIER FIRES: producer agrees with all four claims => the boundary is the claim wiring/shape, NOT the producer (H-wc FALSE).'
+RDX_X='reason matched none of the 24 compared caret symbols (outside-table OR comparison failure); loses which reason, and whether comparison works'
+# CONTROL readings (generated semantics; data, never acceptance). ctl_ok on ^resolve_reason_unbound_symbol
+# (arm 0 of the chain), ctl_out on ^resolve_reason_binder_duplicate_in_frame (outside the 24).
+ctl_ok_reading() {
+  case "$1" in
+    0) printf '%s\n' 'CONTROL HOLDS: the known symbol matched its own arm (0): caret "==" discriminates';;
+    X) printf '%s\n' 'CONTROL VIOLATION: the known symbol matched NOTHING: caret "==" comparison is BROKEN (or the frozen 24-arm chain moved); every r=X in this line is then a comparison failure, not an outside-table reason';;
+    *) printf '%s\n' "CONTROL VIOLATION: the known symbol matched arm $1 instead of 0: the 24-arm chain is PERMUTED (or the symbol is no longer arm 0)";;
+  esac
+}
+ctl_out_reading() {
+  case "$1" in
+    X) printf '%s\n' 'CONTROL HOLDS: the outside-table symbol matched nothing (X), as declared: the chain does not compare symbols outside its 24 names';;
+    *) printf '%s\n' "CONTROL VIOLATION: the outside-table symbol matched arm $1: equality COLLAPSED or the chain compares a symbol outside its 24 names";;
+  esac
+}
+# probe_token_r R -> 'r=R', from the table. Values outside the declared set refuse.
+probe_token_r() { case "|$TT_r|" in *'|'"$1"'|'**) printf '%s%s%s' "$SEP_r" "$EQ" "$1";; *) return 1;; esac; }
+# probe_ctl FIELD VALUE -> ' FIELD=r=VALUE' (ctl tokens are reason_token outputs: 'r=' + value).
+probe_ctl() { case "|$TT_ctl|" in *'|'"$2"'|'**) printf '%s%s%s%s' "$1" "$EQ" "$TOK_CTL" "$2";; *) return 1;; esac; }
+# probe_token R A S AL -> the composed braced token, from the table only.
 probe_token() {
   local r="$1" a="$2" s="$3" al="$4" t
-  case "|$TT_r|" in *"|$r|"*) :;; *) return 1;; esac
-  case "|$TT_a|" in *"|$a|"*) :;; *) return 1;; esac
-  case "|$TT_s|" in *"|$s|"*) :;; *) return 1;; esac
+  case "|$TT_r|" in *'|'"$r"'|'**) :;; *) return 1;; esac
+  case "|$TT_a|" in *'|'"$a"'|'**) :;; *) return 1;; esac
+  case "|$TT_s|" in *'|'"$s"'|'**) :;; *) return 1;; esac
   [[ "$al" =~ ^(${TT_al_FORM})$ ]] || return 1
-  [ "${#al}" -le 19 ] || return 1
+  [ "${#al}" -le "${#TT_al_BOUND}" ] || return 1
   t="$SEP_r$EQ$r$SEP_a$EQ$a$SEP_s$EQ$s$SEP_al$EQ$al"
   printf '%s' "$TOK_OPEN$TOK_IN$t$TOK_CLOSE"
 }
-# probe_row_ok T1 T2 T3 T4 / probe_row_rf -> complete reason strings from the layout table only.
-probe_row_ok() { printf '%s%s%s%s%s%s%s%s%s%s' "$CTX_OK" "" "$SEP_arm" "$EQ$1" "$SEP_field" "$EQ$2" "$SEP_binder" "$EQ$3" "$SEP_body" "$EQ$4"; }
-probe_row_rf() { printf '%s%s%s%s-%s%s-%s%s-%s%s-' "$CTX_PRE" "RF" "$SEP_arm" "$EQ" "$SEP_field" "$EQ" "$SEP_binder" "$EQ" "$SEP_body" "$EQ"; }
-# The grammar, DERIVED from the same constants. Field order fixed by the layout; every alternative is
-# the verbatim token, so any row that validates is byte-composable from the table alone.
-PG_PLAIN='(R|A|F)'
-PG_X="${TOK_OPEN}\\{${SEP_r}${EQ}(0|[1-9]|1[0-9]|2[0-3]|X)${SEP_a}${EQ}(U|\\?|-)${SEP_s}${EQ}(T|F)${SEP_al}${EQ}(${TT_al_FORM})\\}"
+# probe_row_ok COK COUT T1 T2 T3 T4 / probe_row_rf COK COUT -> complete reason strings, table only.
+probe_row_ok() {
+  local c1 c2; c1="$(probe_ctl "$SEP_ctl_ok" "$1")" || return 1; c2="$(probe_ctl "$SEP_ctl_out" "$2")" || return 1
+  printf '%s%s%s%s%s%s%s%s%s%s%s%s%s' "$CTX_OK" "$c1" "$c2" "$SEP_arm" "$EQ$3" "$SEP_field" "$EQ$4" "$SEP_binder" "$EQ$5" "$SEP_body" "$EQ$6"
+}
+probe_row_rf() {
+  local c1 c2; c1="$(probe_ctl "$SEP_ctl_ok" "$1")" || return 1; c2="$(probe_ctl "$SEP_ctl_out" "$2")" || return 1
+  printf '%s%s%s%s%s%s%s%s%s%s%s%s%s%s%s%s' "$CTX_RF" "$c1" "$c2" "$SEP_arm" "$EQ" "-" "$SEP_field" "$EQ" "-" "$SEP_binder" "$EQ" "-" "$SEP_body" "$EQ" "-"
+}
+# THE GRAMMARS, derived from the same constants. Every field alternative is the verbatim declared set,
+# so any row that validates is byte-composable from the table alone (the split/reconstruct gate below
+# proves it per row). Escapes ('{' '}' '?') are the ones ERE and python re accept IDENTICALLY; the
+# byte-equality fixture against the reader constants enforces it.
+PG_PLAIN="(${TT_plain})"
+ALT_r="(${TT_r})"; ALT_a_RAW="$(printf '%s' "$TT_a" | sed 's/?/\\?/g')"; ALT_a="($ALT_a_RAW)"
+ALT_s="(${TT_s})"; ALT_ctl="(${TT_ctl})"; ALT_al="(${TT_al_FORM})"
+PG_X="${TOK_OPEN}\\{${SEP_r}${EQ}${ALT_r}${SEP_a}${EQ}${ALT_a}${SEP_s}${EQ}${ALT_s}${SEP_al}${EQ}${ALT_al}\\}"
 PG_V="(${PG_PLAIN}|${PG_X})"
-PG_OK="^${CTX_OK}${SEP_arm}${EQ}${PG_V}${SEP_field}${EQ}${PG_V}${SEP_binder}${EQ}${PG_V}${SEP_body}${EQ}${PG_V}\$"
-PG_RF="^${CTX_RF}${SEP_arm}${EQ}-${SEP_field}${EQ}-${SEP_binder}${EQ}-${SEP_body}${EQ}-\$"
-# probe_validate REASON -> 0 accepted / 1 refused. Length first, then the anchored grammar.
+PG_OK="^${CTX_OK}${SEP_ctl_ok}${EQ}${TOK_CTL}${ALT_ctl}${SEP_ctl_out}${EQ}${TOK_CTL}${ALT_ctl}${SEP_arm}${EQ}${PG_V}${SEP_field}${EQ}${PG_V}${SEP_binder}${EQ}${PG_V}${SEP_body}${EQ}${PG_V}\$"
+PG_RF="^${CTX_RF}${SEP_ctl_ok}${EQ}${TOK_CTL}${ALT_ctl}${SEP_ctl_out}${EQ}${TOK_CTL}${ALT_ctl}${SEP_arm}${EQ}-${SEP_field}${EQ}-${SEP_binder}${EQ}-${SEP_body}${EQ}-\$"
+# The BOUND is not a hand-written number: it is the LENGTH OF THE TABLE-COMPOSED WIDEST ROW.
+WIDEST_TOKEN="$(probe_token 23 '?' F "$TT_al_BOUND")" || die2 "REFUSED_SETUP: the widest declared token refused its own table"
+WIDEST_ROW="$(probe_row_ok 23 23 "$WIDEST_TOKEN" "$WIDEST_TOKEN" "$WIDEST_TOKEN" "$WIDEST_TOKEN")" || die2 "REFUSED_SETUP: the widest declared row refused its own table"
+WIDEST_LEN="${#WIDEST_ROW}"; BOUND="$WIDEST_LEN"
+# probe_validate REASON -> 0 accepted / 1 refused; PROBE_VALIDATE_KIND records WHICH check refused
+# (BOUND | GRAMMAR | CONSISTENCY). Check order is fixed: bound, grammar, consistency. The bound check
+# may be bypassed ONLY via PROBE_BOUND_BYPASS=1, used by one fixture to prove the grammar itself
+# refuses an over-long row, so the two checks are demonstrably distinct.
 probe_validate() {
   local r="$1"
-  [ "${#r}" -le 193 ] || return 1
-  if [[ "$r" =~ $PG_RF ]]; then return 0; fi
-  if [[ "$r" =~ $PG_OK ]]; then return 0; fi
-  return 1
+  PROBE_VALIDATE_KIND=OK
+  if [ "${PROBE_BOUND_BYPASS:-0}" != 1 ]; then
+    if [ "${#r}" -gt "$BOUND" ]; then PROBE_VALIDATE_KIND=BOUND; return 1; fi
+  fi
+  if [[ "$r" =~ $PG_RF ]]; then :
+  elif [[ "$r" =~ $PG_OK ]]; then :
+  else PROBE_VALIDATE_KIND=GRAMMAR; return 1; fi
+  # CONSISTENCY (table rule TT_rule_a_minus_al_zero=1): a braced token with a='-' must carry al='0'.
+  # RF rows have no braced tokens; OK rows are split once and each token parsed with the SAME parser.
+  local c1 c2 t_arm t_field t_binder t_body tok t1 t2 t3 t4 a al
+  if [[ "$r" =~ $PG_OK ]]; then
+    probe_split_ok "$r" c1 c2 t_arm t_field t_binder t_body || { PROBE_VALIDATE_KIND=GRAMMAR; return 1; }
+    for tok in "$t_arm" "$t_field" "$t_binder" "$t_body"; do
+      case "$tok" in "$TOK_OPEN$TOK_IN"*)
+        probe_parse_token "$tok" t1 t2 t3 t4 || { PROBE_VALIDATE_KIND=GRAMMAR; return 1; }
+        a="$t2"; al="$t4"
+        if [ "$a" = '-' ] && [ "$al" != 0 ]; then PROBE_VALIDATE_KIND=CONSISTENCY; return 1; fi
+      esac
+    done
+  fi
+  return 0
 }
-# ---- the READING TABLE, same tables (frozen per-module rows; parent msg_bf5a7d3c + msg_b8f24de9) ----
-RD_R='WcResolved'; RD_A='WcAbsent'; RD_F='WcFileRefused'
-RD_atom_U='atom == ^wc_undeclared_body (the frozen candidate)'; RD_atom_Q='atom outside the candidate set'; RD_atom_D='atom ABSENT (no locus)'
-RDX_NOCTX='READING (frozen table): ctx=RF — all four collapsed at the context build. The boundary is INGEST ADMISSION (native_test_context_from_ingest Rejected). Per-module rows do not exist for this shape.'
-RDX_ALLF='FROZEN READING: all four F => the boundary is the LANE FILE-REFUSAL policy.'
-RDX_ALLA='FROZEN READING: all four A => the fold never matched the census entries (census-name/path boundary).'
-RDX_FALS='FALSIFIER FIRES: producer agrees with all four claims => the boundary is the claim wiring/shape, NOT the producer (H-wc FALSE).'
 # probe_parse_token TOKEN RVAR AVAR SVAR ALVAR -> 0 and sets the four variables; refuses (1) on any
 # token that is not composed of declared alternatives with the declared separators, in order, once each.
+# The closing-brace checks are REACHABLE via direct-call fixtures (v7-p1..p4): on grammar-validated
+# rows the anchored grammar already excludes those shapes, so this defense is exercised directly.
 probe_parse_token() {
   local v="$1" __r __a __s __al rest
   local -n __R="$2" __A="$3" __S="$4" __AL="$5"
@@ -285,7 +385,7 @@ probe_parse_token() {
     F) __r='__plain'; __a='F'; __s='__tag'; __al='F';;
     "$TOK_OPEN$TOK_IN"*)
       rest="${v#"$TOK_OPEN$TOK_IN"}"
-      [ "$rest" != "${v%"$TOK_CLOSE"}" ] || return 1            # must end with the single TOK_CLOSE
+      case "$v" in *"$TOK_CLOSE") :;; *) return 1;; esac           # must end with the single TOK_CLOSE
       rest="${rest%"$TOK_CLOSE"}"
       case "$rest" in *"$TOK_CLOSE"*) return 1;; esac           # a second '}' anywhere is illegal
       case "$rest" in "$SEP_r$EQ"*) :;; *) return 1;; esac

```
