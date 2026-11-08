## wc-probe-v2 vs wc-probe-v1 (review diff)

Left = the posted v1 bytes (sha 11acdca5..., superseded, never launched); right = wc-probe-v2 (sha e0160950...). Only the probe-side code changed: the job-name literals, PROBE_TOKEN/PROBE_REASON same-shell globals + token-agreement gate in the run path, the anchored grammar PG_V/PG_OK/PG_RF + probe_validate, its call before the reason is accepted (failure = NO_OBSERVATION exit 2, raw line preserved and echoed), and the new fixtures (7)-(9). The v4 frame sections are untouched.

```
--- packet-v1-posted.sh	2026-11-06 09:32:57.492832158 +0000
+++ packet-wc-probe-v2.sh	2026-11-06 09:32:44.078745131 +0000
@@ -1,5 +1,5 @@
 #!/bin/bash
-# FROZEN wc-verdict diagnostic packet wc-probe-v1 (author sleek-seal-226, #15695 lane; parent draft approval msg_bf5a7d3c).
+# FROZEN wc-verdict diagnostic packet wc-probe-v2 (author sleek-seal-226, #15695 lane; parent draft approval msg_bf5a7d3c).
 # PURPOSE: ONE observation — WHICH WcVerdict each of the four wildcard_arm_resolve fixtures actually takes at seed A. The v4 join
 #   measured 0/4 BARE FAILs for those claims at BOTH pins (claims-A-wc.log sha db53f7b0, claims-B-wc.log sha cee20bfd); bare verdict
 #   lines carry no cause, so the WcVerdict tag (WcResolved | WcAbsent | WcFileRefused | WcRefused{reason,at_atom,sole_chain} |
@@ -42,7 +42,7 @@
 # MEASUREMENT ONLY: no acceptance follows from exit 0; this packet judges no claim and changes no frozen expectation from the v4
 #   join (v4a 265609d4..., v4b 8a2bd47e...), which stand as measured.
 set -uo pipefail
-JOB=sleek-seal-226-wc-probe-v1
+JOB=sleek-seal-226-wc-probe-v2
 UNIT="$JOB.scope"
 PIN_A=f08a6ae4e44dc0636eda71f25f0009e97ffde1d1; TREE_A=2549a8e6545468a55fd8de3d0a082049bae19b32
 PIN_P=5fb8ed7e913c5b7a2fd3e100ab24dd40c7b49baf; TREE_P=d917abc098c0ba5bd3593f181f939a6ecbe056d8; BLOB_PROBE=084b051209fdf262691c3b1899f2f98aa2e2f9a3; BLOB_WC=aba6d2be15f1510629d7d8f8d47267e162bf2047
@@ -122,7 +122,7 @@
 # (3) exit contract
 [ "$(r rc_in_contract 0) $(r rc_in_contract 1) $(r rc_in_contract 2) $(r rc_in_contract 124) $(r rc_in_contract 137) $(r rc_in_contract 3)" = "accepted accepted accepted refused refused refused" ] || die2 "REFUSED_SETUP: exit-contract fixture misfire"
 # (4) scope glob literal suffix, restore cleanliness
-[ "$(r scope_ok '0::/a/sleek-seal-226-wc-probe-v1.scope') $(r scope_ok '0::/a/sleek-seal-226-wc-probe-v1Xscope') $(r scope_ok '0::/a/xsleek-seal-226-wc-probe-v1.scope') $(r scope_ok '0::/a/sleek-seal-226-wc-probe-v1.scope.x')" = "accepted refused refused refused" ] || die2 "REFUSED_SETUP: scope fixture misfire"
+[ "$(r scope_ok '0::/a/sleek-seal-226-wc-probe-v2.scope') $(r scope_ok '0::/a/sleek-seal-226-wc-probe-v2Xscope') $(r scope_ok '0::/a/xsleek-seal-226-wc-probe-v2.scope') $(r scope_ok '0::/a/sleek-seal-226-wc-probe-v2.scope.x')" = "accepted refused refused refused" ] || die2 "REFUSED_SETUP: scope fixture misfire"
 git init -q "$FX/g" && : > "$FX/g/f" || die2 "fixture git init"
 git -C "$FX/g" add f && git -C "$FX/g" -c user.name=fx -c user.email=fx@fx commit -qm fx || die2 "fixture commit"
 A="$(r tree_clean $FX/g)"; : > "$FX/g/u"; B="$(r tree_clean $FX/g)"
@@ -141,29 +141,48 @@
 #   'FAIL <fn> (returned ProcessExit::ExitFailure, code <n>: <reason>)' for n not 2; 'UNOBSERVED <fn> (returned ProcessExit::ExitFailure, code 2...)'.
 # 'code 33' must not match 'code 3': the prefix test requires 'code 3: ' exactly (space-colon-space), and the suffix ')' must close the line.
 PROBE_REASON=""
+PROBE_TOKEN=NONE
 probe_classify() {
   local l; l="$(printf '%s' "$1" | sed 's/[[:space:]]*$//')"
   local P="FAIL $2 (returned ProcessExit::ExitFailure, code 3: "
   local Q="FAIL $2 (returned ProcessExit::ExitFailure, code 4: "
-  if [ "$l" = "PASS $2" ]; then echo PASS; return; fi
-  if [ "$l" = "FAIL $2" ]; then echo BARE; return; fi
+  if [ "$l" = "PASS $2" ]; then PROBE_TOKEN=PASS; echo PASS; return; fi
+  if [ "$l" = "FAIL $2" ]; then PROBE_TOKEN=BARE; echo BARE; return; fi
   case "$l" in
-    "$P"*) if [ "${l: -1}" = ")" ]; then PROBE_REASON="${l#"$P"}"; PROBE_REASON="${PROBE_REASON%?}"; echo EXIT3; else echo OTHER; fi; return;;
-    "$Q"*) echo EXIT4; return;;
+    "$P"*) if [ "${l: -1}" = ")" ]; then PROBE_REASON="${l#"$P"}"; PROBE_REASON="${PROBE_REASON%?}"; PROBE_TOKEN=EXIT3; echo EXIT3; else PROBE_TOKEN=OTHER; echo OTHER; fi; return;;
+    "$Q"*) PROBE_TOKEN=EXIT4; echo EXIT4; return;;
   esac
   case "$l" in
-    "UNOBSERVED $2 "*) echo UNOBSERVED; return;;
-    "FAIL $2 "* | "PANICKED $2 "*) echo OTHER; return;;
+    "UNOBSERVED $2 "*) PROBE_TOKEN=UNOBSERVED; echo UNOBSERVED; return;;
+    "FAIL $2 "* | "PANICKED $2 "*) PROBE_TOKEN=OTHER; echo OTHER; return;;
   esac
-  echo NONE
+  PROBE_TOKEN=NONE; echo NONE
 }
 # probe_verdict LOG NAME -> token; the SINGLE-outcome-line rule mirrors v4 verdict(): zero, a duplicate, or a conflict is NONE.
+# The declared grammar, ANCHORED (parent byte review, msg_fdb10bd9). Two shapes only:
+#   RF row (fixed literal, the 'other fields absent' shape is NOT allowed): WCVERIFY ctx=RF arm=- field=- binder=- body=-
+#   OK row: WCVERIFY ctx=OK arm=<v> field=<v> binder=<v> body=<v>, each <v> exactly:
+#     R | A | F | X{r=<0..23|X>,a=<U|?|->,s=<T|F>,al=<1..19 digits>}
+#   r digits are the frozen table indices as plain decimal (0|[1-9]|1[0-9]|2[0-3]); al is a plain
+#   decimal count of at most 19 digits (the i64 bound of int_to_decimal_string).
+PG_V='(R|A|F|X\{r=(0|[1-9]|1[0-9]|2[0-3]),a=(U|\?|-),s=(T|F),al=[0-9]{1,19}\})'
+PG_OK="^WCVERIFY ctx=OK arm=${PG_V} field=${PG_V} binder=${PG_V} body=${PG_V}$"
+PG_RF='^WCVERIFY ctx=RF arm=- field=- binder=- body=-$'
+# probe_validate REASON -> 0 accepted / 1 refused. Length first, then the anchored grammar.
+probe_validate() {
+  local r="$1"
+  [ "${#r}" -le 193 ] || return 1
+  if [[ "$r" =~ $PG_RF ]]; then return 0; fi
+  if [[ "$r" =~ $PG_OK ]]; then return 0; fi
+  return 1
+}
+PROBE_TOKEN=NONE
 probe_verdict() {
   local L n; L="$(grep -aE "^[A-Z]+ $2( |\$)" "$1")"
-  [ -n "$L" ] || { echo NONE; return; }
+  if [ -z "$L" ]; then PROBE_TOKEN=NONE; echo NONE; return; fi
   n="$(printf '%s\n' "$L" | grep -ac '')"
-  [ "$n" = 1 ] || { echo NONE; return; }
-  probe_classify "$L" "$2"
+  if [ "$n" != 1 ]; then PROBE_TOKEN=NONE; echo NONE; return; fi
+  probe_classify "$L" "$2" >/dev/null; echo "$PROBE_TOKEN"
 }
 
 # (7) probe classifier fixtures (a focused failing input for each misread class), before any fetch
@@ -183,6 +202,38 @@
 [ "$PROBE_REASON" = "WCVERIFY ctx=OK arm=R field=R binder=R body=X{r=0,a=?,s=T,al=1}" ] || die2 "REFUSED_SETUP: probe reason extraction is not verbatim"
 # (8) probe bound self-check: the stated maximum, from the frozen layout arithmetic, is within the claim-side bound 193
 [ $(( 15 + 5 + 7 + 8 + 6 + 4 * 38 )) -le 193 ] || die2 "REFUSED_SETUP: the stated probe layout exceeds its stated bound"
+
+# (9) grammar fixtures through the ACTUAL path: a full typed line -> probe_verdict -> probe_validate
+PGD="$FX"
+pv_line() { printf 'FAIL %s (returned ProcessExit::ExitFailure, code 3: %s)\n' "$PFN" "$1" > "$PGD/gline"; probe_verdict "$PGD/gline" "$PFN" >/dev/null; if [ "$PROBE_TOKEN" != EXIT3 ]; then echo 1; return; fi; probe_validate "$PROBE_REASON"; echo "$?"; }
+# valid OK row
+[ "$(pv_line 'WCVERIFY ctx=OK arm=R field=R binder=R body=X{r=0,a=?,s=T,al=1}')" = 0 ] || die2 "REFUSED_SETUP: grammar fixture (valid OK) refused"
+# valid RF row (fixed literal)
+[ "$(pv_line 'WCVERIFY ctx=RF arm=- field=- binder=- body=-')" = 0 ] || die2 "REFUSED_SETUP: grammar fixture (valid RF) refused"
+# garbage reason
+[ "$(pv_line 'hello world')" = 1 ] || die2 "REFUSED_SETUP: grammar fixture (garbage) accepted"
+# extra trailing field
+[ "$(pv_line 'WCVERIFY ctx=OK arm=R field=R binder=R body=R extra=R')" = 1 ] || die2 "REFUSED_SETUP: grammar fixture (extra field) accepted"
+# unknown tag
+[ "$(pv_line 'WCVERIFY ctx=OK arm=Q field=R binder=R body=R')" = 1 ] || die2 "REFUSED_SETUP: grammar fixture (unknown tag) accepted"
+# unknown reason index
+[ "$(pv_line 'WCVERIFY ctx=OK arm=X{r=24,a=U,s=T,al=1} field=R binder=R body=R')" = 1 ] || die2 "REFUSED_SETUP: grammar fixture (reason index 24) accepted"
+# unsorted/duplicate-free digit forms: leading zero rejected
+[ "$(pv_line 'WCVERIFY ctx=OK arm=X{r=07,a=U,s=T,al=01} field=R binder=R body=R')" = 1 ] || die2 "REFUSED_SETUP: grammar fixture (leading zero) accepted"
+# RF with the other fields absent is NOT the declared shape
+[ "$(pv_line 'WCVERIFY ctx=RF')" = 1 ] || die2 "REFUSED_SETUP: grammar fixture (bare RF) accepted"
+# nested parens inside the reason cannot smuggle through: the ')' is positional
+[ "$(pv_line 'WCVERIFY (x) ctx=OK arm=R field=R binder=R body=R')" = 1 ] || die2 "REFUSED_SETUP: grammar fixture (nested paren) accepted"
+# valid MAXIMUM length: 193 chars, every field at its widest
+MX='X{r=23,a=?,s=F,al=1234567890123456789}'
+MAXROW="WCVERIFY ctx=OK arm=${MX} field=${MX} binder=${MX} body=${MX}"
+[ "${#MAXROW}" = 193 ] || die2 "REFUSED_SETUP: the constructed maximum row is not 193 chars (${#MAXROW})"
+[ "$(pv_line "$MAXROW")" = 0 ] || die2 "REFUSED_SETUP: grammar fixture (valid maximum 193) refused"
+# length 194 through the length check directly (a grammatical 194 cannot exist; the bound is belt-and-braces)
+probe_validate "${MAXROW}X" && die2 "REFUSED_SETUP: a 194-char reason passed the length bound"
+probe_validate "$(printf 'W%.0s' {1..193})X" && die2 "REFUSED_SETUP: a 194-char garbage reason passed the length bound"
+# a 20-digit al is refused by the grammar (not just by length)
+[ "$(pv_line "WCVERIFY ctx=OK arm=X{r=1,a=U,s=T,al=12345678901234567890} field=R binder=R body=R")" = 1 ] || die2 "REFUSED_SETUP: grammar fixture (20-digit al) accepted"
 rm -rf "$FX"
 echo "fixtures: all held"
 # ---- arg check + exclusive OUT (v17 semantics, verbatim) ----
@@ -216,9 +267,13 @@
   rc_in_contract "$RC" || die2 "leg A probe: exit status $RC is out of contract (0/1/2 only; 124 timeout, >128 signal)"
   completed "$L" || die2 "leg A probe: no single '[process-partition] total_ms=<digits>' completion line; batch incomplete whatever verdict lines printed"
   echo "verdict lines (A probe):"; grep -aE "^(PASS|FAIL|UNOBSERVED|PANICKED) " "$L"
-  VD="$(probe_verdict "$L" "$PFN")"; echo "A probe $PFN=$VD"
+  probe_verdict "$L" "$PFN" > "$R/.probe.vd" || true; VD="$(cat "$R/.probe.vd")"; echo "A probe $PFN=$VD"
+  [ "$PROBE_TOKEN" = "$VD" ] || die2 "leg A probe: classifier token disagreement ($VD vs $PROBE_TOKEN)"
   if [ "$VD" = NONE ]; then MISSING=1; die2 "leg A probe: '$PFN' has no SINGLE outcome line (absent, duplicate, or unclassifiable)"; fi
   if [ "$VD" = EXIT3 ]; then
+    RAW3="$(grep -aE "^FAIL $PFN \(returned ProcessExit::ExitFailure, code 3: " "$L")"
+    echo "RAW: $RAW3"
+    probe_validate "$PROBE_REASON" || die2 "leg A probe: NO_OBSERVATION: the reason fails the anchored grammar or the 193-char bound; raw line preserved in claims-A-probe.log and here: $RAW3"
     printf '%s\n' "$PROBE_REASON" > "$R/probe-reason.txt" || die2 "cannot write probe-reason.txt"
     echo "PROBE_REASON: $PROBE_REASON"
     echo "PROBE_REASON_BYTES=${#PROBE_REASON}"

```
