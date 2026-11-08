# wc-probe-v7 — v6->v7 diff (2/3)

Frozen packet 258449f32d82f63f6b68315def188ae3be25e01f55fbba5058171524cef6575e vs v6 b070af6b4a5b292186d4b634c9e144fe3b76f25c633a823e632d3364634b117b. 476 insertions, 236 deletions, one file.

```diff
@@ -296,67 +396,81 @@ probe_parse_token() {
       __a="${rest%%"$SEP_s$EQ"*}"; rest="${rest#*"$SEP_s$EQ"}"
       case "$rest" in *"$SEP_al$EQ"*) :;; *) return 1;; esac
       __s="${rest%%"$SEP_al$EQ"*}"; __al="${rest#*"$SEP_al$EQ"}"
-      case "|$TT_r|" in *"|$__r|"*) :;; *) return 1;; esac
-      case "|$TT_a|" in *"|$__a|"*) :;; *) return 1;; esac
-      case "|$TT_s|" in *"|$__s|"*) :;; *) return 1;; esac
+      case "|$TT_r|" in *'|'"$__r"'|'**) :;; *) return 1;; esac
+      case "|$TT_a|" in *'|'"$__a"'|'**) :;; *) return 1;; esac
+      case "|$TT_s|" in *'|'"$__s"'|'**) :;; *) return 1;; esac
       [[ "$__al" =~ ^(${TT_al_FORM})$ ]] || return 1
-      [ "${#__al}" -le 19 ] || return 1;;
+      [ "${#__al}" -le "${#TT_al_BOUND}" ] || return 1;;
     *) return 1;;
   esac
   if [ "$__r" = __plain ]; then __R="$__al"; __A=''; __S=''; __AL=''; else __R="$__r"; __A="$__a"; __S="$__s"; __AL="$__al"; fi
   return 0
 }
-# probe_tok_reading TOKEN -> the frozen reading of one verdict token; refuses (1) on an illegal token.
+# probe_tok_reading TOKEN -> the reading of one verdict token; refuses (1) on an illegal token.
 probe_tok_reading() {
   local v="$1" r a s al
   case "$v" in R|A|F) printf '%s\n' "$(case "$v" in R) echo "$RD_R";; A) echo "$RD_A";; *) echo "$RD_F";; esac)"; return 0;; esac
   probe_parse_token "$v" r a s al || return 1
-  local rs; if [ "$r" = X ]; then rs='reason OUTSIDE the 24-symbol comparison table (an ^resolve_reason_* symbol the probe compares nothing against)'; else rs="reason = table index $r (the 24-name resolve_reason list)"; fi
+  local rs; if [ "$r" = X ]; then rs="$RDX_X"; else rs="reason = table index $r (the 24-name resolve_reason list)"; fi
   local at; case "$a" in U) at="$RD_atom_U";; \?) at="$RD_atom_Q";; -) at="$RD_atom_D";; *) return 1;; esac
   local sc; if [ "$s" = T ]; then sc='sole_chain=True (rest of the diagnostic chain Empty)'; else sc='sole_chain=False'; fi
   printf 'WcRefused{%s; %s; %s; atom_lexeme_len=%s}\n' "$rs" "$at" "$sc" "$al"
 }
-# probe_strip REST KEY -> the token that follows 'KEY=' up to the next key or end of line.
-# Field splitting is delimiter-based (no prefix/suffix pattern removals): the grammar anchors the
-# full layout, so each key occurs EXACTLY once and in order; splitting on the keys is unambiguous
-# and byte-exact. probe_split_ok ROW A1 A2 A3 A4 fills the four token variables (and returns 1 on a
-# missing key or wrong count).
+# probe_split_ok ROW C1 C2 T_ARM T_FIELD T_BINDER T_BODY — delimiter-based split on the SIX keys, in
+# order, each exactly once; the reconstruction via probe_row_ok must reproduce the row BYTE-FOR-BYTE
+# (layout proof; one authority). Values are bare (ctl values stripped of the 'r=' prefix here only).
 probe_split_ok() {
   local r="$1" line
-  local -n _A1="$2" _A2="$3" _A3="$4" _A4="$5"
+  local -n _C1="$2" _C2="$3" _A1="$4" _A2="$5" _A3="$6" _A4="$7"
   case "$r" in "$CTX_OK"*) :;; *) return 1;; esac
-  # shellcheck disable=SC2206
-  local parts=()
-  # split on the four keys, keeping only the pieces between them
-  local tmp="$r" p
-  for p in "$SEP_arm" "$SEP_field" "$SEP_binder"; do
-    case "$tmp" in *"$p$EQ"*) :;; *) return 1;; esac
+  local p
+  for p in "$SEP_ctl_ok" "$SEP_ctl_out" "$SEP_arm" "$SEP_field" "$SEP_binder"; do
+    case "$r" in *"$p$EQ"*) :;; *) return 1;; esac
   done
-  line="$(printf '%s\n' "$r" | sed -e "s/^$CTX_OK$SEP_arm$EQ//" -e "s/$SEP_field$EQ/\n/" -e "s/$SEP_binder$EQ/\n/" -e "s/$SEP_body$EQ/\n/" | tr -d ' ')"
-  [ "$(printf '%s\n' "$line" | wc -l)" = 4 ] || return 1
-  _A1="$(printf '%s\n' "$line" | sed -n 1p)"
-  _A2="$(printf '%s\n' "$line" | sed -n 2p)"
-  _A3="$(printf '%s\n' "$line" | sed -n 3p)"
-  _A4="$(printf '%s\n' "$line" | sed -n 4p)"
-  # the reconstruction must reproduce the original row byte-for-byte (layout proof, one authority)
-  local rebuilt; rebuilt="$(probe_row_ok "$_A1" "$_A2" "$_A3" "$_A4")"
+  line="$(printf '%s\n' "$r" | sed -e "s/^${CTX_OK}//" -e "s/$SEP_ctl_ok$EQ/\n/" -e "s/$SEP_ctl_out$EQ/\n/" -e "s/$SEP_arm$EQ/\n/" -e "s/$SEP_field$EQ/\n/" -e "s/$SEP_binder$EQ/\n/" -e "s/$SEP_body$EQ/\n/" | tr -d ' ' | sed '/^$/d')"
+  [ "$(printf '%s\n' "$line" | wc -l)" = 6 ] || return 1
+  _C1="$(printf '%s\n' "$line" | sed -n 1p)"; _C2="$(printf '%s\n' "$line" | sed -n 2p)"
+  _C1="${_C1#"$TOK_CTL"}"; _C2="${_C2#"$TOK_CTL"}"
+  _A1="$(printf '%s\n' "$line" | sed -n 3p)"; _A2="$(printf '%s\n' "$line" | sed -n 4p)"
+  _A3="$(printf '%s\n' "$line" | sed -n 5p)"; _A4="$(printf '%s\n' "$line" | sed -n 6p)"
+  local rebuilt; rebuilt="$(probe_row_ok "$_C1" "$_C2" "$_A1" "$_A2" "$_A3" "$_A4")"
   [ "$rebuilt" = "$r" ] || return 1
   return 0
 }
 # probe_read_line REASON: the reading-path lookup, mirroring probe_read.py (shipped in the packet and
-# sha-gate-checked below). Observation only; it never implies acceptance.
+# sha-gate-checked below; its grammar constants are byte-checked against PG_OK/PG_RF at setup).
+# Observation only; it never implies acceptance.
 probe_read_line() {
   local r="$1"
-  if [ "${#r}" -gt 193 ]; then echo 'REFUSAL: over the stated 193-char bound; NO OBSERVATION'; return 2; fi
-  if [[ "$r" =~ $PG_RF ]]; then echo "$RDX_NOCTX"; return 0; fi
+  if [ "${#r}" -gt "$BOUND" ]; then echo "REFUSAL: over the stated ${BOUND}-char bound (widest table-composed row); NO OBSERVATION"; return 2; fi
+  local c1 c2 arm field binder body f v rd allf=1 alla=1
+  if [[ "$r" =~ $PG_RF ]]; then
+    # RF rows still carry the control; extract it with the same byte-exact discipline.
+    local line; line="$(printf '%s\n' "$r" | sed -e "s/^${CTX_RF}//" -e "s/$SEP_ctl_ok$EQ/\n/" -e "s/$SEP_ctl_out$EQ/\n/" -e "s/$SEP_arm$EQ/\n/" -e "s/$SEP_field$EQ/\n/" -e "s/$SEP_binder$EQ/\n/" -e "s/$SEP_body$EQ/\n/" | tr -d ' ' | sed '/^$/d')"
+    [ "$(printf '%s\n' "$line" | wc -l)" = 6 ] || { echo 'REFUSAL: RF control split failed'; return 2; }
+    c1="$(printf '%s\n' "$line" | sed -n 1p)"; c2="$(printf '%s\n' "$line" | sed -n 2p)"
+    c1="${c1#"$TOK_CTL"}"; c2="${c2#"$TOK_CTL"}"
+    case "|$TT_ctl|" in *'|'"$c1"'|'**) :;; *) echo 'REFUSAL: ctl_ok is not a declared form'; return 2;; esac
+    case "|$TT_ctl|" in *'|'"$c2"'|'**) :;; *) echo 'REFUSAL: ctl_out is not a declared form'; return 2;; esac
+    [ "$(printf '%s\n' "$line" | sed -n 3p)" = - ] && [ "$(printf '%s\n' "$line" | sed -n 4p)" = - ] &&
+    [ "$(printf '%s\n' "$line" | sed -n 5p)" = - ] && [ "$(printf '%s\n' "$line" | sed -n 6p)" = - ] ||
+      { echo 'REFUSAL: RF row carries non-dash payload'; return 2; }
+    echo "CONTROL (ctx=RF row): ctl_ok=$c1 -> $(ctl_ok_reading "$c1")"
+    echo "CONTROL (ctx=RF row): ctl_out=$c2 -> $(ctl_out_reading "$c2")"
+    echo "$RDX_NOCTX"; return 0
+  fi
   if ! [[ "$r" =~ $PG_OK ]]; then echo 'REFUSAL: does not match the anchored grammar — treat as NO OBSERVATION'; return 2; fi
-  local arm field binder body f v rd allf=1 alla=1
-  probe_split_ok "$r" arm field binder body || { echo 'REFUSAL: layout parse failed (keys missing, duplicated, or out of order)'; return 2; }
+  probe_split_ok "$r" c1 c2 arm field binder body || { echo 'REFUSAL: layout parse failed (keys missing, duplicated, or out of order)'; return 2; }
+  case "|$TT_ctl|" in *'|'"$c1"'|'**) :;; *) echo 'REFUSAL: ctl_ok is not a declared form'; return 2;; esac
+  case "|$TT_ctl|" in *'|'"$c2"'|'**) :;; *) echo 'REFUSAL: ctl_out is not a declared form'; return 2;; esac
+  echo "CONTROL: ctl_ok=$c1 -> $(ctl_ok_reading "$c1")"
+  echo "CONTROL: ctl_out=$c2 -> $(ctl_out_reading "$c2")"
+  if [ "$c1" = 0 ] && [ "$c2" = X ]; then echo 'CONTROL VERDICT (observation only): caret "==" comparison DISCRIMINATES for the two control symbols.'; else echo 'CONTROL VERDICT (observation only): COMPARISON FAILURE SIGNATURE — read every r=X as a possible broken equality, never as an outside-table reason, until located.'; fi
   echo 'PER-MODULE READINGS (MIXED tags => read each row on its own; no single-boundary conclusion follows):'
   for f in arm field binder body; do
     v="${!f}"
-    rd="$(probe_tok_reading "$v")" || { echo "REFUSAL: illegal token in field $f: $v"; return 2; }
-    printf '  %-7s %-44s -> %s\n' "$f" "$v" "$rd"
+    rd="$(probe_tok_reading "$v")" || { echo "REFUSAL: token $v is not a declared form"; return 2; }
+    printf '  %-7s %-52s %s\n' "$f" "$v" "$rd"
     [ "$v" = F ] || allf=0
     [ "$v" = A ] || alla=0
   done
@@ -377,6 +491,7 @@ probe_read_line() {
   echo 'Observation only; acceptance never follows from this reading.'
   return 0
 }
+
 PROBE_TOKEN=NONE
 probe_verdict() {
   local L n; L="$(grep -aE "^[A-Z]+ $2( |\$)" "$1")"

```
