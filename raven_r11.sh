#!/usr/bin/env bash
# raven_r5.sh <tag> -- R1: binder marking BEFORE resolution (manager ruling msg_38fd5b61).
#
# Phase A builds the round host from the COMMITTED mirrors; the round then regenerates the mirrors from
# the .dag authority and installs them; phase B rebuilds from what the round installed and measures.
# Every measurement therefore comes from the REGENERATED code, not from a mirror I typed.
#
# THE CONTROLS, as ruled:
#   RED at main              the two dag/std/measure.dag rows (measure_sub forwarding Q:Quantity into a
#                            Quantity slot and S:Scale into a Scale slot) refuse, class Inhabitants, and
#                            std.measure is abandoned before its items.
#   POSITIVE at the fix      those two rows are GONE (the judgment identifies the forwarded parameter by
#                            declaration and admits it), so the module's env errors are 0 and its items
#                            are analyzed.
#   NEGATIVES stay refused   Qbad / Qsbad (different kind) and Quk / Qsuk (unkinded) must still refuse,
#                            each under its own named cause.
#   FIXED POINT              the round reports no mirror drift once the mirrors are landed.
set -u
TAG="${1:-noname}"
REVERT="${2:-no}"
cd "$(dirname "$0")"

MT_KB=$(awk '/MemTotal/{print $2}' /proc/meminfo)
echo "tag=$TAG MemTotal_MB=$((MT_KB/1024))"
echo "RUNNER_HEAD=$(git rev-parse HEAD)"
echo "RUNNER_BRANCH=$(git rev-parse --abbrev-ref HEAD)"
git status --short | head -8
test "$MT_KB" -ge 41943040 || { echo ABORT_MEMORY; exit 42; }

# The cgroup binding is REQUIRED and READ BACK. An unbound run continues with no budget and the round
# refuses on its own terms (HostBudgetUnreadable) roughly half an hour in; failing here costs seconds.
ROOT=$(awk -F: '$2=="memory"{print $3}' /proc/self/cgroup)
case "$ROOT" in /) CGD=/sys/fs/cgroup;; *) CGD=/sys/fs/cgroup${ROOT};; esac
test -w "$CGD/cgroup.subtree_control" && printf '+memory\n' > "$CGD/cgroup.subtree_control" 2>/dev/null || true
CG="$CGD/raven_r5_${TAG}"
mkdir -p "$CG" 2>/dev/null || { echo ABORT_CGROUP; exit 42; }
printf '45097156608\n' > "$CG/memory.max" 2>/dev/null || echo cgroup_max_not_set
printf '%s\n' $$ > "$CG/cgroup.procs" 2>/dev/null || echo cgroup_procs_not_set
BOUND=$(cat "$CG/memory.max" 2>/dev/null || echo unreadable)
echo "CGROUP_BOUND path=$CG memory.max=$BOUND"
case "$BOUND" in 45097156608) : ;; *) echo "ABORT_CGROUP_UNBOUND memory.max=$BOUND"; exit 42 ;; esac

export GITHUB_ACTIONS=true GITHUB_SHA=$(git rev-parse HEAD)
echo "env GITHUB_ACTIONS=$GITHUB_ACTIONS GITHUB_SHA=$GITHUB_SHA"
unset GUNBC_MEMORY_BUDGET_BYTES GUNBC_MEMORY_ENVELOPE_REQUIRED GUNBC_BIND_MEMORY_CGROUP_BYTES
export TMPDIR=/tmp
PATH=$PATH:/usr/local/cargo/bin

# This lane's .dag edits touch four authorities, so FOUR mirrors are expected to change.
GEN="v1_std_core.rs v1_compiler_parse.rs v1_compiler_infer.rs v1_compiler_infer_resolve.rs"

echo "=== what the COMMITTED mirrors hold (before the round) ==="
grep -c "with_type_param_declaration_form_property" src/v1/stage0/src/v1_compiler_parse.rs | sed 's/^/committed_marker=/'
grep -c "__param_declaration_form" src/v1/stage0/src/v1_std_core.rs | sed 's/^/committed_marker_name=/'
grep -c "binder_marked_module_item" src/v1/stage0/src/v1_compiler_infer.rs | sed 's/^/committed_hoist=/'
grep -c "item_type_param_names" src/v1/stage0/src/v1_compiler_infer_resolve.rs | sed 's/^/committed_roster=/'

echo "=== the .dag authority carries all four ==="
grep -c "with_type_param_declaration_form_property" src/v1/02_parse.dag | sed 's/^/authority_marker_uses=/'
grep -c "fn type_param_declaration_form_property_name" src/v1/00_core.dag | sed 's/^/authority_marker_name_def=/'
grep -c "binder_marked_module_item" src/v1/04_infer.dag | sed 's/^/authority_hoist_uses=/'
grep -c "fn item_type_param_names" src/v1/04_resolve.dag | sed 's/^/authority_roster_def=/'
grep -c "authored_name_at(source_indices: source_indices, node: param_node_type_expr" src/v1/04_resolve.dag | sed 's/^/authority_text_equality_filter_must_be_0=/'
# HYGIENE: this is the PR branch, and it must carry no probe. The reading instrument lives on
# agent/v1-param-binder-probe and is never merged.
grep -rc "RAVEN" src/v1/*.dag 2>/dev/null | grep -v ":0$" | sed 's/^/PROBE_LEFTOVER /'
echo "probe_leftover_files_must_be_0=$(grep -rc 'RAVEN' src/v1/*.dag 2>/dev/null | grep -vc ':0

echo "=== phase A: build the round host from the committed mirrors ==="
cargo build --release -p v1-compiler --bins > /tmp/b1.log 2>&1; echo "build1_rc=$?"
if grep -qaE "^error(\[|:)" /tmp/b1.log; then echo ABORT_BUILD1; grep -aE -A 14 "^error(\[|:)" /tmp/b1.log | head -90 | fold -w 195; exit 46; fi
test -x ./target/release/claim_executor || { echo ABORT_NO_EXEC; exit 46; }

echo "=== the round: regenerate the mirrors from the .dag authority ==="
rm -rf target/stage0-regen-candidate
if [ "$REVERT" = yes ]; then base64 -d > /tmp/revert_judgment.patch <<'B64R'
ZGlmZiAtLWdpdCBhL3NyYy92MS8wNF9yZXNvbHZlLmRhZyBiL3NyYy92MS8wNF9yZXNvbHZlLmRhZwppbmRleCBlNWViOGQ1ZGFjNzcuLjUyNzNlYTRkNjlhNCAxMDA2NDQKLS0tIGEvc3JjL3YxLzA0X3Jlc29sdmUuZGFnCisrKyBiL3NyYy92MS8wNF9yZXNvbHZlLmRhZwpAQCAtMjI2LDggKzIyNiw3IEBAIGZuIHR5cGVfYXJnX2tpbmRfaW5oYWJpdGFuY2UoYXJnOiBOb2RlLCBraW5kX25vZGU6IE5vZGUsIGVudjogVHlwZUVudiwgbW9kdWxlX25hCiAgIC8vIG5hbWUta2V5ZWQgcmVhZGluZyB0aGlzIGFybSBleGlzdHMgdG8gcmVwbGFjZS4gVGhlIGNhcnJpZXIgaXMgdGhlIGZhY3Q7IGluZmVyZW5jZSBpcyBhIGhpbnQuCiAgIC8vIEEgdmFyaWFibGUgdGhhdCBjYXJyaWVzIG5vdGhpbmcgYW5zd2VycyBGd2RQYXJhbWV0ZXJVbmlkZW50aWZpZWQ6IGJlaW5nIGFuIGluZmVyZW5jZSB1bmtub3duCiAgIC8vIGlzIG5vdCBldmlkZW5jZSBvZiBpbmhhYml0aW5nIGFueXRoaW5nLCBhbmQgdGhpcyBhcm0gYWRtaXRzIG5vYm9keS4KLSAgaWYgbm9kZV9pc190eXBlX3BhcmFtZXRlcl9yZWZlcmVuY2UobjogYXJnKSB7IGZvcndhcmRlZF9wYXJhbWV0ZXJfa2luZF9tYXRjaGVzKGFyZzogYXJnLCBzbG90X2tpbmRfbm9kZToga2luZF9ub2RlLCBlbnY6IGVudiwgbW9kdWxlX25hbWU6IG1vZHVsZV9uYW1lKSB9Ci0gIGVsc2UgaWYgYXJnX2lzX3R5cGVfdmFyIHsgRndkUGFyYW1ldGVyVW5pZGVudGlmaWVkIH0KKyAgaWYgbm9kZV9pc190eXBlX3BhcmFtZXRlcl9yZWZlcmVuY2UobjogYXJnKSB8fCBhcmdfaXNfdHlwZV92YXIgeyBLaW5kSW5oYWJpdGVkIH0KICAgZWxzZSBpZiBleHByX2xpdGVyYWxfaW50X29wdGlvbmFsKGV4cHI6IGFyZykgIT0gbm9uZSB7IEtpbmRJbmhhYml0ZWQgfQogICBlbHNlIHsKICAgICBtYXRjaCBsb29rdXBfdHlwZV9ieV9uYW1lKGVudjogZW52LCBuYW1lOiBhdXRob3JlZF9uYW1lKGVudjogZW52LCBub2RlOiBraW5kX25vZGUpKSB7Cg==
B64R
git apply /tmp/revert_judgment.patch; echo "revert_judgment_rc=$?"
grep -c forwarded_parameter_kind_matches src/v1/04_resolve.dag | sed 's/^/revert_dispatch_uses_of_judgment=/'; fi
timeout 2100 ./target/release/claim_executor --regen-round-cost --source-root dag --source-root src/v2 > /tmp/round.out 2>/tmp/round.err
echo "round_rc=$?"
tail -12 /tmp/round.out | fold -w 195
echo "--- round.err: compile-shaped lines, if any ---"
grep -aE "error\[|error:|panicked|No such file" /tmp/round.err | head -25 | fold -w 195
echo "--- round.err tail 20 ---"; tail -20 /tmp/round.err | fold -w 195
echo "ALGEBRA_FIELD_REFUSALS=$(grep -ac "no field 'key' on type" /tmp/round.err)"
CAND=target/stage0-regen-candidate/src
CAND_FILES=$(cd "$CAND" 2>/dev/null && ls *.rs 2>/dev/null | tr '\n' ' ')
echo "CANDIDATE_FILES=$(echo $CAND_FILES | wc -w): $CAND_FILES"

echo "=== ASSERT: every carrier is in the bytes that are about to be installed ==="
for pair in "v1_compiler_parse.rs:with_type_param_declaration_form_property" "v1_std_core.rs:__param_declaration_form" "v1_compiler_infer.rs:binder_marked_module_item" "v1_compiler_infer_resolve.rs:item_type_param_names"; do
  f="${pair%%:*}"; tok="${pair##*:}"
  n=$(grep -ac "$tok" "$CAND/$f" 2>/dev/null || echo MISSING)
  echo "CANDIDATE_HAS $tok in $f = $n"
  test "$n" -ge 1 2>/dev/null || { echo "ABORT_CARRIER_ABSENT $tok $f"; exit 48; }
done
for pair in "v1_compiler_infer.rs:RAVENPROG|N|" "v1_compiler_emit_rust.rs:RAVENPROG|W|"; do
  f="${pair%%:*}"; tok="${pair##*:}"
  n=$(grep -acF "$tok" "$CAND/$f" 2>/dev/null || echo MISSING)
  echo "PROBE_CARRIER_HAS $tok in $f = $n"
  test "$n" -ge 1 2>/dev/null || { echo "ABORT_PROBE_CARRIER_ABSENT $tok $f"; exit 48; }
done
# the deleted spelling test must be gone from the generated code too
grep -ac "param_node_type_expr(n: p)" "$CAND/v1_compiler_infer_resolve.rs" 2>/dev/null | sed 's/^/candidate_text_equality_must_be_0=/'
for sym in raven_probe_kind_link measure_hit_text type_arg_kind_inhabitance ForwardedParameterKindUnresolved; do
  files=$(grep -rlF "$sym" "$CAND" 2>/dev/null | sed "s|$CAND/||" | tr "\n" ",")
  echo "CANDIDATE_HAS $sym in $files"
  test -n "$files" || { echo "ABORT_CANDIDATE_MISSING $sym"; exit 47; }
done


echo "=== install EVERY file the round produced ==="
test -n "$CAND_FILES" || { echo ABORT_NO_CANDIDATE_TREE; exit 48; }
for f in $CAND_FILES; do cp "$CAND/$f" "src/v1/stage0/src/$f"; done
echo "MIRROR_DRIFT_FILES=$(git diff --name-only -- src/v1/stage0/src/ | wc -l)"
git diff --name-only -- src/v1/stage0/src/ | sed 's/^/DRIFT /'

echo "=== phase B: rebuild from the installed mirrors ==="
cargo build --release -p v1-compiler --bins > /tmp/b2.log 2>&1; echo "build2_rc=$?"
if grep -qaE "^error(\[|:)" /tmp/b2.log; then echo ABORT_BUILD2; grep -aE -A 14 "^error(\[|:)" /tmp/b2.log | head -90 | fold -w 195; exit 49; fi
test -x ./target/release/gunbc || { echo ABORT_NO_GUNBC; exit 49; }

base64 -d > /tmp/analyze.py <<'B64A'
aW1wb3J0IHJlLCBzeXMsIG9zLCBjb2xsZWN0aW9ucwojIENBVVNFUyBhcmUga2V5ZWQgb24gdGhlIG1lc3NhZ2UgdGV4dCB0aGUg
Y2Vuc3VzIHByaW50cywgdGFrZW4gdmVyYmF0aW0gZnJvbSB0aGUgZGlhZ25vc3RpYyBtZXNzYWdlCiMgZm9sZCBpbiBzcmMvdjEv
MDBfY29yZS5kYWc6IHRoZSBmaXZlIGZvcndhcmRlZCBjYXVzZXMgdGhpcyBQUiBhZGRzLCB0aGUgZGlmZmVyIGNhc2UsIGFuZCBt
YWluJ3Mgb3duCiMgaW5oYWJpdGFudC1saXN0IHJlZnVzYWwuIE1hdGNoaW5nIGlzIGJ5IHN1YnN0cmluZyBvbiBwdXJwb3NlOiB0
aGUgcm93IHRleHQgaXMgdGhlIG1lc3NhZ2UuCkNBVVNFUyA9IFsKICAoIlVuaWRlbnRpZmllZCIsICAgImNhcnJpZXMgbm8gdHlw
ZS1wYXJhbWV0ZXIgZGVjbGFyYXRpb24iKSwKICAoIk93bmVyVW5yZXNvbHZlZCIsImRpZCBub3QgcmVzb2x2ZSB0byBvbmUgZGVj
bGFyaW5nIG1vZHVsZSIpLAogICgiTm90RGVjbGFyZWQiLCAgICAiaXMgbm90IGEgcGFyYW1ldGVyIG9mIHRoZSBkZWNsYXJhdGlv
biIpLAogICgiVW5raW5kZWQiLCAgICAgICAiZGVjbGFyZXMgbm8ga2luZCIpLAogICgiS2luZFVucmVzb2x2ZWQiLCAiZGlkIG5v
dCByZXNvbHZlIHRvIGEgc2luZ2xlIGRlY2xhcmluZyBwb3NpdGlvbiIpLAogICgiS2luZERpZmZlcnMiLCAgICAicmVzb2x2ZXMg
dG8gYSBkaWZmZXJlbnQgZGVjbGFyYXRpb24gdGhhbiB0aGUgc2xvdCIpLAogICgiSW5oYWJpdGFudHMiLCAgICAiaXMgbm90IG9u
ZSBvZiBpdHMgaW5oYWJpdGFudHMiKSwKXQpNQVJLID0gKCJkb2VzIG5vdCBpbmhhYml0IHRoZSBkZWNsYXJlZCBraW5kIiwgImZv
cndhcmRlZCB0eXBlIGFyZ3VtZW50IikKQVBQTFkgPSByZS5jb21waWxlKHIiYXBwbHlpbmcgJyg/UDxhcHA+W14nXSspJyIpClBB
UkFNID0gcmUuY29tcGlsZShyInBhcmFtZXRlciAnKD9QPHBhcj5bXiddKyknIikKREVDTCAgPSByZS5jb21waWxlKHIiaXMgZGVj
bGFyZWQgJyg/UDxraW5kPlteJ10rKSciKQpTVVAgICA9IHJlLmNvbXBpbGUociIoPzphbmQgJ3xzdXBwbGllZCBpcyAnKSg/UDxz
dXA+W14nXSspIikKU1BBTiAgPSByZS5jb21waWxlKHIiXCgoP1A8ZmlsZT5bXHcuLy1dK1wuZGFnKTooP1A8bG8+XGQrKSIpCmRl
ZiByb3dzKHBhdGgpOgogICAgb3V0ID0gW10KICAgIGZvciBsaW5lIGluIG9wZW4ocGF0aCwgZW5jb2Rpbmc9InV0Zi04IiwgZXJy
b3JzPSJyZXBsYWNlIik6CiAgICAgICAgaWYgbm90IGFueShtIGluIGxpbmUgZm9yIG0gaW4gTUFSSyk6IGNvbnRpbnVlCiAgICAg
ICAgYXAgPSBBUFBMWS5zZWFyY2gobGluZSk7IHBhID0gUEFSQU0uc2VhcmNoKGxpbmUpOyBkZSA9IERFQ0wuc2VhcmNoKGxpbmUp
CiAgICAgICAgc3UgPSBTVVAuc2VhcmNoKGxpbmUpOyBzcCA9IFNQQU4uc2VhcmNoKGxpbmUpCiAgICAgICAgY2F1c2UgPSBuZXh0
KChjIGZvciBjLCBzaWcgaW4gQ0FVU0VTIGlmIHNpZyBpbiBsaW5lKSwgIk9USEVSIikKICAgICAgICBvdXQuYXBwZW5kKGRpY3Qo
Y2F1c2U9Y2F1c2UsCiAgICAgICAgICAgICAgICAgICAgICAgIGFwcD1hcC5ncm91cCgiYXBwIikgaWYgYXAgZWxzZSAiPyIsIHBh
cj1wYS5ncm91cCgicGFyIikgaWYgcGEgZWxzZSAiPyIsCiAgICAgICAgICAgICAgICAgICAgICAgIGtpbmQ9ZGUuZ3JvdXAoImtp
bmQiKSBpZiBkZSBlbHNlICI/IiwKICAgICAgICAgICAgICAgICAgICAgICAgc3VwPXN1Lmdyb3VwKCJzdXAiKVs6NDAwMF0gaWYg
c3UgZWxzZSAiPyIsCiAgICAgICAgICAgICAgICAgICAgICAgIGZpbGU9c3AuZ3JvdXAoImZpbGUiKSBpZiBzcCBlbHNlICI/Iiwg
bGluZT1zcC5ncm91cCgibG8iKSBpZiBzcCBlbHNlICI/IikpCiAgICByZXR1cm4gb3V0CmRlZiByZXBvcnQodGFnLCBwYXRoKToK
ICAgIGlmIG5vdCBvcy5wYXRoLmV4aXN0cyhwYXRoKToKICAgICAgICBwcmludChmIj09IHt0YWd9OiBNSVNTSU5HX0xPRyB7cGF0
aH0iKTsgcmV0dXJuCiAgICBycyA9IHJvd3MocGF0aCkKICAgIHByaW50KGYiPT0ge3RhZ306IHRvdGFsX3JlZnVzYWxfcm93cz17
bGVuKHJzKX0iKQogICAgZm9yIGMsIF8gaW4gQ0FVU0VTOgogICAgICAgIHByaW50KGYiICAge3RhZ31fY2F1c2Vbe2N9XT17c3Vt
KDEgZm9yIHIgaW4gcnMgaWYgclsnY2F1c2UnXT09Yyl9IikKICAgIHByaW50KGYiICAge3RhZ31fY2F1c2VbT1RIRVJdPXtzdW0o
MSBmb3IgciBpbiBycyBpZiByWydjYXVzZSddPT0nT1RIRVInKX0iKQogICAgZyA9IGNvbGxlY3Rpb25zLkNvdW50ZXIoKHJbImZp
bGUiXSwgclsiYXBwIl0sIHJbInBhciJdLCByWyJraW5kIl0sIHJbImNhdXNlIl0pIGZvciByIGluIHJzKQogICAgcHJpbnQoZiIg
ICB7dGFnfV9jYXVzZV9ncm91cHM9e2xlbihnKX0iKQogICAgZm9yIGssIG4gaW4gZy5tb3N0X2NvbW1vbigyNCk6CiAgICAgICAg
cHJpbnQoIiAgIEdST1VQIiwgbiwgaykKICAgIHByaW50KGYiICAge3RhZ31fYnlfZmlsZToiKQogICAgZm9yIGssIG4gaW4gY29s
bGVjdGlvbnMuQ291bnRlcihyWyJmaWxlIl0gZm9yIHIgaW4gcnMpLm1vc3RfY29tbW9uKDE0KToKICAgICAgICBwcmludCgiICAg
ICBGSUxFIiwgbiwgaykKICAgIHUgPSBjb2xsZWN0aW9ucy5Db3VudGVyKChyWyJmaWxlIl0sIHJbImFwcCJdLCByWyJwYXIiXSwg
clsic3VwIl0pIGZvciByIGluIHJzIGlmIHJbImNhdXNlIl0gPT0gIlVuaWRlbnRpZmllZCIpCiAgICBwcmludChmIiAgIHt0YWd9
X2FybTFfdW5pZGVudGlmaWVkX3Jvd3M9e3N1bSh1LnZhbHVlcygpKX0gZ3JvdXBzPXtsZW4odSl9IikKICAgIGZvciBrLCBuIGlu
IHUubW9zdF9jb21tb24oMjApOgogICAgICAgIHByaW50KCIgICAgIEFSTTEiLCBuLCBrKQogICAgcHJpbnQoIiAgIFZFUkJBVElN
IHBlciBjYXVzZToiKQogICAgc2VlbiA9IHNldCgpCiAgICBmb3IgciBpbiByczoKICAgICAgICBpZiByWyJjYXVzZSJdIGluIHNl
ZW46IGNvbnRpbnVlCiAgICAgICAgc2Vlbi5hZGQoclsiY2F1c2UiXSkKICAgICAgICBwcmludCgiICAgICAiLCByWyJjYXVzZSJd
LCAifCIsIHJbImZpbGUiXSwgclsibGluZSJdLCAifCIsIHJbImFwcCJdLCByWyJwYXIiXSwgclsia2luZCJdLCAifCIsIHJbInN1
cCJdWzo0MDAwXSkKcmVwb3J0KCJVTksiLCAiL3RtcC91bmsudHh0IikKcmVwb3J0KCJLSU4iLCAiL3RtcC9raW4udHh0IikK
B64A
wc -c /tmp/analyze.py | sed 's/^/analyze_bytes=/'

echo "=== RUNG 1: unkinded (no kinded-kv patch) -- baseline at main was 0 rows ==="
timeout 1500 ./target/release/gunbc test //gunbc/instruments:v2-native-census-scoped > /tmp/cen_unk.out 2>/tmp/cen_unk.err
echo "census_unk_rc=$?"
tail -12 /tmp/cen_unk.err | fold -w 195 | sed "s/^/CENUERR /"; grep -ac "panic\|Panicked" /tmp/cen_unk.err | sed "s/^/CENU_PANIC_LINES=/"
cat /tmp/cen_unk.out /tmp/cen_unk.err > /tmp/unk.txt
python3 /tmp/analyze.py > /tmp/analysis_unk.txt 2>&1; echo "analyze_unk_rc=$?"
grep -a "UNK_total\|UNK_cause\|GROUP" /tmp/analysis_unk.txt | head -14 | cut -c1-190
grep -a native-census-infer /tmp/unk.txt | grep -a "std.measure\|std.realization" | cut -c1-400 | sed "s/^/PROG_UNK /"
grep -a "RAVENPROG" /tmp/unk.txt | cut -c1-400 | sed "s/^/MINT_UNK /" | sort | uniq -c
echo "--- [native-census-infer] watched-module outcomes, unkinded rung (pre-existing stderr relay, zero minting) ---"
grep -a native-census-infer /tmp/unk.txt | grep -a "std.measure\|std.realization" | cut -c1-400 | sed "s/^/PROGUNK /"

echo "=== RUNG 2: kinded (kinded-kv patch applied) ==="
base64 -d > /tmp/kv90.patch <<'B64K'
ZGlmZiAtLWdpdCBhL2RhZy9zdGQvbWVhc3VyZS5kYWcgYi9kYWcvc3RkL21lYXN1cmUuZGFnCmluZGV4IDNlYzIyYzNiMTZmLi41MzlhY2YxODgxMyAxMDA2NDQKLS0tIGEvZGFnL3N0ZC9tZWFzdXJlLmRhZworKysgYi9kYWcvc3RkL21lYXN1cmUuZGFnCkBAIC0yMzQsNDIgKzIzNCw0MiBAQCBmbiBtZW1vcnlfc2NhbGVfZmFjdG9yX2J5dGVzKHM6IFNjYWxlKSAtPiBOYXQ/IHsKICAgfQogfQogCi10eXBlIE1lYXN1cmU8USwgUywgTT4gewordHlwZSBNZWFzdXJlPFE6IFF1YW50aXR5LCBTOiBTY2FsZSwgTT4gewogICBjb3VudDogTQogfQogCi1mbiBtZWFzdXJlX2NvdW50PFEsIFMsIE0+KG06IE1lYXN1cmU8USwgUywgTT4pIC0+IE0geworZm4gbWVhc3VyZV9jb3VudDxROiBRdWFudGl0eSwgUzogU2NhbGUsIE0+KG06IE1lYXN1cmU8USwgUywgTT4pIC0+IE0gewogICBtLmNvdW50CiB9CiAKLWZuIG1lYXN1cmVfc2NhbGVfZnJhY3Rpb25fZmxvb3I8USwgUz4obTogTWVhc3VyZTxRLCBTLCBOYXQ+LCBudW06IE5hdCwgZGVuOiBOYXQpIC0+IE1lYXN1cmU8USwgUywgTmF0PiB7CitmbiBtZWFzdXJlX3NjYWxlX2ZyYWN0aW9uX2Zsb29yPFE6IFF1YW50aXR5LCBTOiBTY2FsZT4obTogTWVhc3VyZTxRLCBTLCBOYXQ+LCBudW06IE5hdCwgZGVuOiBOYXQpIC0+IE1lYXN1cmU8USwgUywgTmF0PiB7CiAKICAgTWVhc3VyZSB7IGNvdW50OiBpZiBkZW4gPT0gMCB7IGRlbiB9IGVsc2UgeyAobS5jb3VudCAqIG51bSkgLyBkZW4gfSB9CiB9CiAKLWZuIG1lYXN1cmVfZml0X2NvdW50X2Zsb29yPFEsIFM+KGNhcGFjaXR5OiBNZWFzdXJlPFEsIFMsIE5hdD4sIGVhY2g6IE1lYXN1cmU8USwgUywgTmF0PikgLT4gTmF0IHsKK2ZuIG1lYXN1cmVfZml0X2NvdW50X2Zsb29yPFE6IFF1YW50aXR5LCBTOiBTY2FsZT4oY2FwYWNpdHk6IE1lYXN1cmU8USwgUywgTmF0PiwgZWFjaDogTWVhc3VyZTxRLCBTLCBOYXQ+KSAtPiBOYXQgewogCiAgIGxldCBlYWNoX2NvdW50ID0gZWFjaC5jb3VudAogICBpZiBlYWNoX2NvdW50ID09IDAgeyBlYWNoX2NvdW50IH0gZWxzZSB7IGNhcGFjaXR5LmNvdW50IC8gZWFjaF9jb3VudCB9CiB9CiAKLWZuIG1lYXN1cmVfc2NhbGVfZnJhY3Rpb25fY2VpbDxRLCBTPihtOiBNZWFzdXJlPFEsIFMsIE5hdD4sIG51bTogTmF0LCBkZW46IE5hdCkgLT4gTWVhc3VyZTxRLCBTLCBOYXQ+IHsKK2ZuIG1lYXN1cmVfc2NhbGVfZnJhY3Rpb25fY2VpbDxROiBRdWFudGl0eSwgUzogU2NhbGU+KG06IE1lYXN1cmU8USwgUywgTmF0PiwgbnVtOiBOYXQsIGRlbjogTmF0KSAtPiBNZWFzdXJlPFEsIFMsIE5hdD4gewogCiAgIE1lYXN1cmUgeyBjb3VudDogaWYgZGVuID09IDAgeyBtLmNvdW50ICogbnVtIH0gZWxzZSB7ICgobS5jb3VudCAqIG51bSkgKyAoZGVuIC0gMSkpIC8gZGVuIH0gfQogfQogCi1mbiBtZWFzdXJlX2FkZDxRLCBTPihhOiBNZWFzdXJlPFEsIFMsIE5hdD4sIGI6IE1lYXN1cmU8USwgUywgTmF0PikgLT4gTWVhc3VyZTxRLCBTLCBOYXQ+IHsKK2ZuIG1lYXN1cmVfYWRkPFE6IFF1YW50aXR5LCBTOiBTY2FsZT4oYTogTWVhc3VyZTxRLCBTLCBOYXQ+LCBiOiBNZWFzdXJlPFEsIFMsIE5hdD4pIC0+IE1lYXN1cmU8USwgUywgTmF0PiB7CiAgIE1lYXN1cmUgeyBjb3VudDogYS5jb3VudCArIGIuY291bnQgfQogfQogCiAvLyBUcnVuY2F0ZWQgc3VidHJhY3Rpb24gaXMgbm90IHRoZSBkZWZhdWx0LiBOYXR1cmFscyBhcmUgbm90IGNsb3NlZCB1bmRlciBzdWJ0cmFjdGlvbjsgdGhlCiAvLyBQZWFubyBmb3JtIGlzIHYyLnN0ZC5uYXQgbmF0X3N1YiAoTmF0RGlmZmVyZW5jZSB8IE5hdFN1YnRyYWhlbmRFeGNlZWRzTWludWVuZCkuIEEgZmxvb3IgYXQKIC8vIHplcm8gd291bGQgZmFicmljYXRlIGEgZGlmZmVyZW5jZSBvZiBlcXVhbCBtZWFzdXJlcy4KLXR5cGUgTWVhc3VyZVN1YnRyYWN0aW9uPFEsIFM+Cit0eXBlIE1lYXN1cmVTdWJ0cmFjdGlvbjxROiBRdWFudGl0eSwgUzogU2NhbGU+CiAgID0gTWVhc3VyZURpZmZlcmVuY2UgeyB2YWx1ZTogTWVhc3VyZTxRLCBTLCBOYXQ+IH0KICAgfCBNZWFzdXJlU3VidHJhaGVuZEV4Y2VlZHNNaW51ZW5kCiAKLWZuIG1lYXN1cmVfc3ViPFEsIFM+KGE6IE1lYXN1cmU8USwgUywgTmF0PiwgYjogTWVhc3VyZTxRLCBTLCBOYXQ+KSAtPiBNZWFzdXJlU3VidHJhY3Rpb248USwgUz4geworZm4gbWVhc3VyZV9zdWI8UTogUXVhbnRpdHksIFM6IFNjYWxlPihhOiBNZWFzdXJlPFEsIFMsIE5hdD4sIGI6IE1lYXN1cmU8USwgUywgTmF0PikgLT4gTWVhc3VyZVN1YnRyYWN0aW9uPFEsIFM+IHsKICAgaWYgYS5jb3VudCA8IGIuY291bnQgewogICAgIE1lYXN1cmVTdWJ0cmFoZW5kRXhjZWVkc01pbnVlbmQKICAgfSBlbHNlIHsK
B64K
git apply /tmp/kv90.patch; echo "kv90_rc=$?"
echo "matrix=OFF (corpus population must stay pure)"
timeout 1500 ./target/release/gunbc test //gunbc/instruments:v2-native-census-scoped > /tmp/cen_k.out 2>/tmp/cen_k.err
echo "census_kind_rc=$?"
echo "--- cen_k.err tail (r15+ relay) ---"; tail -30 /tmp/cen_k.err | fold -w 195 | sed "s/^/CENKERR /"; echo "--- cen_k.out head ---"; head -8 /tmp/cen_k.out | fold -w 195 | sed "s/^/CENKOUT /"; grep -ac "panic\|Panicked" /tmp/cen_k.err | sed "s/^/CENK_PANIC_LINES=/"; grep -a "panic\|Panicked" /tmp/cen_k.err | head -6 | fold -w 300 | sed "s/^/CENKPANIC /"
cat /tmp/cen_k.out /tmp/cen_k.err > /tmp/kin.txt
python3 /tmp/analyze.py > /tmp/analysis_kin.txt 2>&1; echo "analyze_kin_rc=$?"
grep -a "KIN_total\|KIN_cause\|GROUP" /tmp/analysis_kin.txt | head -20 | cut -c1-190
grep -a native-census-infer /tmp/kin.txt | grep -a "std.measure\|std.realization" | cut -c1-400 | sed "s/^/PROG_KIN /"
grep -a "RAVENPROG" /tmp/kin.txt | cut -c1-400 | sed "s/^/MINT_KIN /" | sort | uniq -c
grep -ao "|K [^)]*" /tmp/kin.txt | cut -c1-70 | sort | uniq -c | sort -rn | head -30 | sed "s/^/KINDLINK /"
grep -ao "|K [^)]*" /tmp/unk.txt | cut -c1-70 | sort | uniq -c | sort -rn | head -12 | sed "s/^/KINDLINK_UNK /"
echo "KIN_capture_bytes=$(wc -c < /tmp/cen_k.out) $(wc -c < /tmp/cen_k.err)"
echo "KIN_infer_lines_out=$(grep -ac native-census-infer /tmp/cen_k.out) KIN_infer_lines_err=$(grep -ac native-census-infer /tmp/cen_k.err)"
echo "--- [native-census-infer] watched-module outcomes, kinded rung (pre-existing stderr relay, zero minting) ---"
grep -a native-census-infer /tmp/kin.txt | grep -a "std.measure\|std.realization" | cut -c1-400 | sed "s/^/PROGKIN /"
echo "--- registry reach: effect-summary-incomplete rows naming std.measure callees ---"
echo "REGISTRY_INCOMPLETE_TOTAL=$(grep -ac "effect summary incomplete" /tmp/kin.txt)"
grep -a "effect summary incomplete" /tmp/kin.txt | cut -c1-300 | sed "s/^/EFFECTE /"
echo "REGISTRY_DISTINCT_MEASURE_CALLEES=$(grep -a \"effect summary incomplete\" /tmp/kin.txt | grep -oE \"std.measure.[a-z_]+\" | sort -u | wc -l)"

echo "=== THE RED/POSITIVE CONTROL: the two dag/std/measure.dag rows ==="
echo "rows mentioning std/measure.dag (expect NONE after the fix; 2 class-Inhabitants rows at main):"
grep -a "dag/std/measure.dag" /tmp/kin.txt | grep -av "RAVEN" | head -10 | cut -c1-600
echo "MEASURE_PATH_ROWS=$(grep -ac 'dag/std/measure.dag' /tmp/kin.txt)"
echo "the two measure_sub offsets 6248 / 6251 (expect absent):"
grep -a "|P o=" /tmp/kin.txt | sort -u | head -40 | cut -c1-600

echo "=== WITNESS ROWS: the positive and the three negatives ==="
for r in a_forwarded_parameter_declaring_the_slots_kind_is_admitted \
         a_forwarded_parameter_declaring_a_different_kind_must_refuse \
         a_forwarded_unkinded_parameter_must_refuse \
         a_forwarded_parameter_whose_kind_only_looks_like_the_slots_kind_must_refuse \
         a_forwarded_parameter_whose_kind_comes_by_import_is_admitted \
         a_forwarded_parameter_naming_an_unimported_kind_must_refuse; do
  timeout 600 ./target/release/gunbc test "//test/claim/type_argument_kind_inhabitance_witness:$r" > /tmp/w_$r.out 2>/tmp/w_$r.err
  echo "WITNESS $r rc=$?"
  tail -3 /tmp/w_$r.out | fold -w 190 | sed 's/^/    /'
  grep -a "refus\|did not hold\|FAILED" /tmp/w_$r.err | head -2 | fold -w 190 | sed 's/^/    err: /'
done

# The regenerated mirrors are the round's own bytes and they have to come home: ctrl-build copies
# nothing back across the runner boundary, so the diff is emitted gz+base64 on stdout with its byte
# count and md5, and the local side lands it as a commit and re-runs to prove the fixed point.
echo "@@MIRROR_B64_BEGIN bytes=$(git diff -- src/v1/stage0/src/ | wc -c) md5=$(git diff -- src/v1/stage0/src/ | md5sum | cut -d' ' -f1)"
git diff -- src/v1/stage0/src/ | gzip -9 | base64 -w0
echo ""
echo "@@MIRROR_B64_END"

git checkout -- dag/std/measure.dag src/v1/stage0/src/ 2>/dev/null
echo "R5_DONE tag=$TAG"
)"

echo "=== phase A: build the round host from the committed mirrors ==="
cargo build --release -p v1-compiler --bins > /tmp/b1.log 2>&1; echo "build1_rc=$?"
if grep -qaE "^error(\[|:)" /tmp/b1.log; then echo ABORT_BUILD1; grep -aE -A 14 "^error(\[|:)" /tmp/b1.log | head -90 | fold -w 195; exit 46; fi
test -x ./target/release/claim_executor || { echo ABORT_NO_EXEC; exit 46; }

echo "=== the round: regenerate the mirrors from the .dag authority ==="
rm -rf target/stage0-regen-candidate
if [ "$REVERT" = yes ]; then base64 -d > /tmp/revert_judgment.patch <<'B64R'
ZGlmZiAtLWdpdCBhL3NyYy92MS8wNF9yZXNvbHZlLmRhZyBiL3NyYy92MS8wNF9yZXNvbHZlLmRhZwppbmRleCBlNWViOGQ1ZGFjNzcuLjUyNzNlYTRkNjlhNCAxMDA2NDQKLS0tIGEvc3JjL3YxLzA0X3Jlc29sdmUuZGFnCisrKyBiL3NyYy92MS8wNF9yZXNvbHZlLmRhZwpAQCAtMjI2LDggKzIyNiw3IEBAIGZuIHR5cGVfYXJnX2tpbmRfaW5oYWJpdGFuY2UoYXJnOiBOb2RlLCBraW5kX25vZGU6IE5vZGUsIGVudjogVHlwZUVudiwgbW9kdWxlX25hCiAgIC8vIG5hbWUta2V5ZWQgcmVhZGluZyB0aGlzIGFybSBleGlzdHMgdG8gcmVwbGFjZS4gVGhlIGNhcnJpZXIgaXMgdGhlIGZhY3Q7IGluZmVyZW5jZSBpcyBhIGhpbnQuCiAgIC8vIEEgdmFyaWFibGUgdGhhdCBjYXJyaWVzIG5vdGhpbmcgYW5zd2VycyBGd2RQYXJhbWV0ZXJVbmlkZW50aWZpZWQ6IGJlaW5nIGFuIGluZmVyZW5jZSB1bmtub3duCiAgIC8vIGlzIG5vdCBldmlkZW5jZSBvZiBpbmhhYml0aW5nIGFueXRoaW5nLCBhbmQgdGhpcyBhcm0gYWRtaXRzIG5vYm9keS4KLSAgaWYgbm9kZV9pc190eXBlX3BhcmFtZXRlcl9yZWZlcmVuY2UobjogYXJnKSB7IGZvcndhcmRlZF9wYXJhbWV0ZXJfa2luZF9tYXRjaGVzKGFyZzogYXJnLCBzbG90X2tpbmRfbm9kZToga2luZF9ub2RlLCBlbnY6IGVudiwgbW9kdWxlX25hbWU6IG1vZHVsZV9uYW1lKSB9Ci0gIGVsc2UgaWYgYXJnX2lzX3R5cGVfdmFyIHsgRndkUGFyYW1ldGVyVW5pZGVudGlmaWVkIH0KKyAgaWYgbm9kZV9pc190eXBlX3BhcmFtZXRlcl9yZWZlcmVuY2UobjogYXJnKSB8fCBhcmdfaXNfdHlwZV92YXIgeyBLaW5kSW5oYWJpdGVkIH0KICAgZWxzZSBpZiBleHByX2xpdGVyYWxfaW50X29wdGlvbmFsKGV4cHI6IGFyZykgIT0gbm9uZSB7IEtpbmRJbmhhYml0ZWQgfQogICBlbHNlIHsKICAgICBtYXRjaCBsb29rdXBfdHlwZV9ieV9uYW1lKGVudjogZW52LCBuYW1lOiBhdXRob3JlZF9uYW1lKGVudjogZW52LCBub2RlOiBraW5kX25vZGUpKSB7Cg==
B64R
git apply /tmp/revert_judgment.patch; echo "revert_judgment_rc=$?"
grep -c forwarded_parameter_kind_matches src/v1/04_resolve.dag | sed 's/^/revert_dispatch_uses_of_judgment=/'; fi
timeout 2100 ./target/release/claim_executor --regen-round-cost --source-root dag --source-root src/v2 > /tmp/round.out 2>/tmp/round.err
echo "round_rc=$?"
tail -12 /tmp/round.out | fold -w 195
echo "--- round.err: compile-shaped lines, if any ---"
grep -aE "error\[|error:|panicked|No such file" /tmp/round.err | head -25 | fold -w 195
echo "--- round.err tail 20 ---"; tail -20 /tmp/round.err | fold -w 195
echo "ALGEBRA_FIELD_REFUSALS=$(grep -ac "no field 'key' on type" /tmp/round.err)"
CAND=target/stage0-regen-candidate/src
CAND_FILES=$(cd "$CAND" 2>/dev/null && ls *.rs 2>/dev/null | tr '\n' ' ')
echo "CANDIDATE_FILES=$(echo $CAND_FILES | wc -w): $CAND_FILES"

echo "=== ASSERT: every carrier is in the bytes that are about to be installed ==="
for pair in "v1_compiler_parse.rs:with_type_param_declaration_form_property" "v1_std_core.rs:__param_declaration_form" "v1_compiler_infer.rs:binder_marked_module_item" "v1_compiler_infer_resolve.rs:item_type_param_names"; do
  f="${pair%%:*}"; tok="${pair##*:}"
  n=$(grep -ac "$tok" "$CAND/$f" 2>/dev/null || echo MISSING)
  echo "CANDIDATE_HAS $tok in $f = $n"
  test "$n" -ge 1 2>/dev/null || { echo "ABORT_CARRIER_ABSENT $tok $f"; exit 48; }
done
for pair in "v1_compiler_infer.rs:RAVENPROG|N|" "v1_compiler_emit_rust.rs:RAVENPROG|W|"; do
  f="${pair%%:*}"; tok="${pair##*:}"
  n=$(grep -acF "$tok" "$CAND/$f" 2>/dev/null || echo MISSING)
  echo "PROBE_CARRIER_HAS $tok in $f = $n"
  test "$n" -ge 1 2>/dev/null || { echo "ABORT_PROBE_CARRIER_ABSENT $tok $f"; exit 48; }
done
# the deleted spelling test must be gone from the generated code too
grep -ac "param_node_type_expr(n: p)" "$CAND/v1_compiler_infer_resolve.rs" 2>/dev/null | sed 's/^/candidate_text_equality_must_be_0=/'
for sym in raven_probe_kind_link measure_hit_text type_arg_kind_inhabitance ForwardedParameterKindUnresolved; do
  files=$(grep -rlF "$sym" "$CAND" 2>/dev/null | sed "s|$CAND/||" | tr "\n" ",")
  echo "CANDIDATE_HAS $sym in $files"
  test -n "$files" || { echo "ABORT_CANDIDATE_MISSING $sym"; exit 47; }
done


echo "=== install EVERY file the round produced ==="
test -n "$CAND_FILES" || { echo ABORT_NO_CANDIDATE_TREE; exit 48; }
for f in $CAND_FILES; do cp "$CAND/$f" "src/v1/stage0/src/$f"; done
echo "MIRROR_DRIFT_FILES=$(git diff --name-only -- src/v1/stage0/src/ | wc -l)"
git diff --name-only -- src/v1/stage0/src/ | sed 's/^/DRIFT /'

echo "=== phase B: rebuild from the installed mirrors ==="
cargo build --release -p v1-compiler --bins > /tmp/b2.log 2>&1; echo "build2_rc=$?"
if grep -qaE "^error(\[|:)" /tmp/b2.log; then echo ABORT_BUILD2; grep -aE -A 14 "^error(\[|:)" /tmp/b2.log | head -90 | fold -w 195; exit 49; fi
test -x ./target/release/gunbc || { echo ABORT_NO_GUNBC; exit 49; }

base64 -d > /tmp/analyze.py <<'B64A'
aW1wb3J0IHJlLCBzeXMsIG9zLCBjb2xsZWN0aW9ucwojIENBVVNFUyBhcmUga2V5ZWQgb24gdGhlIG1lc3NhZ2UgdGV4dCB0aGUgY2Vuc3VzIHByaW50cywgdGFrZW4gdmVyYmF0aW0gZnJvbSB0aGUgZGlhZ25vc3RpYyBtZXNzYWdlCiMgZm9sZCBpbiBzcmMvdjEvMDBfY29yZS5kYWc6IHRoZSBmaXZlIGZvcndhcmRlZCBjYXVzZXMgdGhpcyBQUiBhZGRzLCB0aGUgZGlmZmVyIGNhc2UsIGFuZCBtYWluJ3Mgb3duCiMgaW5oYWJpdGFudC1saXN0IHJlZnVzYWwuIE1hdGNoaW5nIGlzIGJ5IHN1YnN0cmluZyBvbiBwdXJwb3NlOiB0aGUgcm93IHRleHQgaXMgdGhlIG1lc3NhZ2UuCkNBVVNFUyA9IFsKICAoIlVuaWRlbnRpZmllZCIsICAgImNhcnJpZXMgbm8gdHlwZS1wYXJhbWV0ZXIgZGVjbGFyYXRpb24iKSwKICAoIk93bmVyVW5yZXNvbHZlZCIsImRpZCBub3QgcmVzb2x2ZSB0byBvbmUgZGVjbGFyaW5nIG1vZHVsZSIpLAogICgiTm90RGVjbGFyZWQiLCAgICAiaXMgbm90IGEgcGFyYW1ldGVyIG9mIHRoZSBkZWNsYXJhdGlvbiIpLAogICgiVW5raW5kZWQiLCAgICAgICAiZGVjbGFyZXMgbm8ga2luZCIpLAogICgiS2luZFVucmVzb2x2ZWQiLCAiZGlkIG5vdCByZXNvbHZlIHRvIGEgc2luZ2xlIGRlY2xhcmluZyBwb3NpdGlvbiIpLAogICgiS2luZERpZmZlcnMiLCAgICAicmVzb2x2ZXMgdG8gYSBkaWZmZXJlbnQgZGVjbGFyYXRpb24gdGhhbiB0aGUgc2xvdCIpLAogICgiSW5oYWJpdGFudHMiLCAgICAiaXMgbm90IG9uZSBvZiBpdHMgaW5oYWJpdGFudHMiKSwKXQpNQVJLID0gKCJkb2VzIG5vdCBpbmhhYml0IHRoZSBkZWNsYXJlZCBraW5kIiwgImZvcndhcmRlZCB0eXBlIGFyZ3VtZW50IikKQVBQTFkgPSByZS5jb21waWxlKHIiYXBwbHlpbmcgJyg/UDxhcHA+W14nXSspJyIpClBBUkFNID0gcmUuY29tcGlsZShyInBhcmFtZXRlciAnKD9QPHBhcj5bXiddKyknIikKREVDTCAgPSByZS5jb21waWxlKHIiaXMgZGVjbGFyZWQgJyg/UDxraW5kPlteJ10rKSciKQpTVVAgICA9IHJlLmNvbXBpbGUociIoPzphbmQgJ3xzdXBwbGllZCBpcyAnKSg/UDxzdXA+W14nXSspIikKU1BBTiAgPSByZS5jb21waWxlKHIiXCgoP1A8ZmlsZT5bXHcuLy1dK1wuZGFnKTooP1A8bG8+XGQrKSIpCmRlZiByb3dzKHBhdGgpOgogICAgb3V0ID0gW10KICAgIGZvciBsaW5lIGluIG9wZW4ocGF0aCwgZW5jb2Rpbmc9InV0Zi04IiwgZXJyb3JzPSJyZXBsYWNlIik6CiAgICAgICAgaWYgbm90IGFueShtIGluIGxpbmUgZm9yIG0gaW4gTUFSSyk6IGNvbnRpbnVlCiAgICAgICAgYXAgPSBBUFBMWS5zZWFyY2gobGluZSk7IHBhID0gUEFSQU0uc2VhcmNoKGxpbmUpOyBkZSA9IERFQ0wuc2VhcmNoKGxpbmUpCiAgICAgICAgc3UgPSBTVVAuc2VhcmNoKGxpbmUpOyBzcCA9IFNQQU4uc2VhcmNoKGxpbmUpCiAgICAgICAgY2F1c2UgPSBuZXh0KChjIGZvciBjLCBzaWcgaW4gQ0FVU0VTIGlmIHNpZyBpbiBsaW5lKSwgIk9USEVSIikKICAgICAgICBvdXQuYXBwZW5kKGRpY3QoY2F1c2U9Y2F1c2UsCiAgICAgICAgICAgICAgICAgICAgICAgIGFwcD1hcC5ncm91cCgiYXBwIikgaWYgYXAgZWxzZSAiPyIsIHBhcj1wYS5ncm91cCgicGFyIikgaWYgcGEgZWxzZSAiPyIsCiAgICAgICAgICAgICAgICAgICAgICAgIGtpbmQ9ZGUuZ3JvdXAoImtpbmQiKSBpZiBkZSBlbHNlICI/IiwKICAgICAgICAgICAgICAgICAgICAgICAgc3VwPXN1Lmdyb3VwKCJzdXAiKVs6NjBdIGlmIHN1IGVsc2UgIj8iLAogICAgICAgICAgICAgICAgICAgICAgICBmaWxlPXNwLmdyb3VwKCJmaWxlIikgaWYgc3AgZWxzZSAiPyIsIGxpbmU9c3AuZ3JvdXAoImxvIikgaWYgc3AgZWxzZSAiPyIpKQogICAgcmV0dXJuIG91dApkZWYgcmVwb3J0KHRhZywgcGF0aCk6CiAgICBpZiBub3Qgb3MucGF0aC5leGlzdHMocGF0aCk6CiAgICAgICAgcHJpbnQoZiI9PSB7dGFnfTogTUlTU0lOR19MT0cge3BhdGh9Iik7IHJldHVybgogICAgcnMgPSByb3dzKHBhdGgpCiAgICBwcmludChmIj09IHt0YWd9OiB0b3RhbF9yZWZ1c2FsX3Jvd3M9e2xlbihycyl9IikKICAgIGZvciBjLCBfIGluIENBVVNFUzoKICAgICAgICBwcmludChmIiAgIHt0YWd9X2NhdXNlW3tjfV09e3N1bSgxIGZvciByIGluIHJzIGlmIHJbJ2NhdXNlJ109PWMpfSIpCiAgICBwcmludChmIiAgIHt0YWd9X2NhdXNlW09USEVSXT17c3VtKDEgZm9yIHIgaW4gcnMgaWYgclsnY2F1c2UnXT09J09USEVSJyl9IikKICAgIGcgPSBjb2xsZWN0aW9ucy5Db3VudGVyKChyWyJmaWxlIl0sIHJbImFwcCJdLCByWyJwYXIiXSwgclsia2luZCJdLCByWyJjYXVzZSJdKSBmb3IgciBpbiBycykKICAgIHByaW50KGYiICAge3RhZ31fY2F1c2VfZ3JvdXBzPXtsZW4oZyl9IikKICAgIGZvciBrLCBuIGluIGcubW9zdF9jb21tb24oMjQpOgogICAgICAgIHByaW50KCIgICBHUk9VUCIsIG4sIGspCiAgICBwcmludChmIiAgIHt0YWd9X2J5X2ZpbGU6IikKICAgIGZvciBrLCBuIGluIGNvbGxlY3Rpb25zLkNvdW50ZXIoclsiZmlsZSJdIGZvciByIGluIHJzKS5tb3N0X2NvbW1vbigxNCk6CiAgICAgICAgcHJpbnQoIiAgICAgRklMRSIsIG4sIGspCiAgICB1ID0gY29sbGVjdGlvbnMuQ291bnRlcigoclsiZmlsZSJdLCByWyJhcHAiXSwgclsicGFyIl0sIHJbInN1cCJdKSBmb3IgciBpbiBycyBpZiByWyJjYXVzZSJdID09ICJVbmlkZW50aWZpZWQiKQogICAgcHJpbnQoZiIgICB7dGFnfV9hcm0xX3VuaWRlbnRpZmllZF9yb3dzPXtzdW0odS52YWx1ZXMoKSl9IGdyb3Vwcz17bGVuKHUpfSIpCiAgICBmb3IgaywgbiBpbiB1Lm1vc3RfY29tbW9uKDIwKToKICAgICAgICBwcmludCgiICAgICBBUk0xIiwgbiwgaykKICAgIHByaW50KCIgICBWRVJCQVRJTSBwZXIgY2F1c2U6IikKICAgIHNlZW4gPSBzZXQoKQogICAgZm9yIHIgaW4gcnM6CiAgICAgICAgaWYgclsiY2F1c2UiXSBpbiBzZWVuOiBjb250aW51ZQogICAgICAgIHNlZW4uYWRkKHJbImNhdXNlIl0pCiAgICAgICAgcHJpbnQoIiAgICAgIiwgclsiY2F1c2UiXSwgInwiLCByWyJmaWxlIl0sIHJbImxpbmUiXSwgInwiLCByWyJhcHAiXSwgclsicGFyIl0sIHJbImtpbmQiXSwgInwiLCByWyJzdXAiXVs6NzBdKQpyZXBvcnQoIlVOSyIsICIvdG1wL3Vuay50eHQiKQpyZXBvcnQoIktJTiIsICIvdG1wL2tpbi50eHQiKQo=
B64A
wc -c /tmp/analyze.py | sed 's/^/analyze_bytes=/'

echo "=== RUNG 1: unkinded (no kinded-kv patch) -- baseline at main was 0 rows ==="
timeout 1500 ./target/release/gunbc test //gunbc/instruments:v2-native-census-scoped > /tmp/cen_unk.out 2>/tmp/cen_unk.err
echo "census_unk_rc=$?"
tail -12 /tmp/cen_unk.err | fold -w 195 | sed "s/^/CENUERR /"; grep -ac "panic\|Panicked" /tmp/cen_unk.err | sed "s/^/CENU_PANIC_LINES=/"
cat /tmp/cen_unk.out /tmp/cen_unk.err > /tmp/unk.txt
python3 /tmp/analyze.py > /tmp/analysis_unk.txt 2>&1; echo "analyze_unk_rc=$?"
grep -a "UNK_total\|UNK_cause\|GROUP" /tmp/analysis_unk.txt | head -14 | cut -c1-190
grep -a native-census-infer /tmp/unk.txt | grep -a "std.measure\|std.realization" | cut -c1-400 | sed "s/^/PROG_UNK /"
grep -a "RAVENPROG" /tmp/unk.txt | cut -c1-400 | sed "s/^/MINT_UNK /" | sort | uniq -c
echo "--- [native-census-infer] watched-module outcomes, unkinded rung (pre-existing stderr relay, zero minting) ---"
grep -a native-census-infer /tmp/unk.txt | grep -a "std.measure\|std.realization" | cut -c1-400 | sed "s/^/PROGUNK /"

echo "=== RUNG 2: kinded (kinded-kv patch applied) ==="
base64 -d > /tmp/kv90.patch <<'B64K'
ZGlmZiAtLWdpdCBhL2RhZy9zdGQvbWVhc3VyZS5kYWcgYi9kYWcvc3RkL21lYXN1cmUuZGFnCmluZGV4IDNlYzIyYzNiMTZmLi41MzlhY2YxODgxMyAxMDA2NDQKLS0tIGEvZGFnL3N0ZC9tZWFzdXJlLmRhZworKysgYi9kYWcvc3RkL21lYXN1cmUuZGFnCkBAIC0yMzQsNDIgKzIzNCw0MiBAQCBmbiBtZW1vcnlfc2NhbGVfZmFjdG9yX2J5dGVzKHM6IFNjYWxlKSAtPiBOYXQ/IHsKICAgfQogfQogCi10eXBlIE1lYXN1cmU8USwgUywgTT4gewordHlwZSBNZWFzdXJlPFE6IFF1YW50aXR5LCBTOiBTY2FsZSwgTT4gewogICBjb3VudDogTQogfQogCi1mbiBtZWFzdXJlX2NvdW50PFEsIFMsIE0+KG06IE1lYXN1cmU8USwgUywgTT4pIC0+IE0geworZm4gbWVhc3VyZV9jb3VudDxROiBRdWFudGl0eSwgUzogU2NhbGUsIE0+KG06IE1lYXN1cmU8USwgUywgTT4pIC0+IE0gewogICBtLmNvdW50CiB9CiAKLWZuIG1lYXN1cmVfc2NhbGVfZnJhY3Rpb25fZmxvb3I8USwgUz4obTogTWVhc3VyZTxRLCBTLCBOYXQ+LCBudW06IE5hdCwgZGVuOiBOYXQpIC0+IE1lYXN1cmU8USwgUywgTmF0PiB7CitmbiBtZWFzdXJlX3NjYWxlX2ZyYWN0aW9uX2Zsb29yPFE6IFF1YW50aXR5LCBTOiBTY2FsZT4obTogTWVhc3VyZTxRLCBTLCBOYXQ+LCBudW06IE5hdCwgZGVuOiBOYXQpIC0+IE1lYXN1cmU8USwgUywgTmF0PiB7CiAKICAgTWVhc3VyZSB7IGNvdW50OiBpZiBkZW4gPT0gMCB7IGRlbiB9IGVsc2UgeyAobS5jb3VudCAqIG51bSkgLyBkZW4gfSB9CiB9CiAKLWZuIG1lYXN1cmVfZml0X2NvdW50X2Zsb29yPFEsIFM+KGNhcGFjaXR5OiBNZWFzdXJlPFEsIFMsIE5hdD4sIGVhY2g6IE1lYXN1cmU8USwgUywgTmF0PikgLT4gTmF0IHsKK2ZuIG1lYXN1cmVfZml0X2NvdW50X2Zsb29yPFE6IFF1YW50aXR5LCBTOiBTY2FsZT4oY2FwYWNpdHk6IE1lYXN1cmU8USwgUywgTmF0PiwgZWFjaDogTWVhc3VyZTxRLCBTLCBOYXQ+KSAtPiBOYXQgewogCiAgIGxldCBlYWNoX2NvdW50ID0gZWFjaC5jb3VudAogICBpZiBlYWNoX2NvdW50ID09IDAgeyBlYWNoX2NvdW50IH0gZWxzZSB7IGNhcGFjaXR5LmNvdW50IC8gZWFjaF9jb3VudCB9CiB9CiAKLWZuIG1lYXN1cmVfc2NhbGVfZnJhY3Rpb25fY2VpbDxRLCBTPihtOiBNZWFzdXJlPFEsIFMsIE5hdD4sIG51bTogTmF0LCBkZW46IE5hdCkgLT4gTWVhc3VyZTxRLCBTLCBOYXQ+IHsKK2ZuIG1lYXN1cmVfc2NhbGVfZnJhY3Rpb25fY2VpbDxROiBRdWFudGl0eSwgUzogU2NhbGU+KG06IE1lYXN1cmU8USwgUywgTmF0PiwgbnVtOiBOYXQsIGRlbjogTmF0KSAtPiBNZWFzdXJlPFEsIFMsIE5hdD4gewogCiAgIE1lYXN1cmUgeyBjb3VudDogaWYgZGVuID09IDAgeyBtLmNvdW50ICogbnVtIH0gZWxzZSB7ICgobS5jb3VudCAqIG51bSkgKyAoZGVuIC0gMSkpIC8gZGVuIH0gfQogfQogCi1mbiBtZWFzdXJlX2FkZDxRLCBTPihhOiBNZWFzdXJlPFEsIFMsIE5hdD4sIGI6IE1lYXN1cmU8USwgUywgTmF0PikgLT4gTWVhc3VyZTxRLCBTLCBOYXQ+IHsKK2ZuIG1lYXN1cmVfYWRkPFE6IFF1YW50aXR5LCBTOiBTY2FsZT4oYTogTWVhc3VyZTxRLCBTLCBOYXQ+LCBiOiBNZWFzdXJlPFEsIFMsIE5hdD4pIC0+IE1lYXN1cmU8USwgUywgTmF0PiB7CiAgIE1lYXN1cmUgeyBjb3VudDogYS5jb3VudCArIGIuY291bnQgfQogfQogCiAvLyBUcnVuY2F0ZWQgc3VidHJhY3Rpb24gaXMgbm90IHRoZSBkZWZhdWx0LiBOYXR1cmFscyBhcmUgbm90IGNsb3NlZCB1bmRlciBzdWJ0cmFjdGlvbjsgdGhlCiAvLyBQZWFubyBmb3JtIGlzIHYyLnN0ZC5uYXQgbmF0X3N1YiAoTmF0RGlmZmVyZW5jZSB8IE5hdFN1YnRyYWhlbmRFeGNlZWRzTWludWVuZCkuIEEgZmxvb3IgYXQKIC8vIHplcm8gd291bGQgZmFicmljYXRlIGEgZGlmZmVyZW5jZSBvZiBlcXVhbCBtZWFzdXJlcy4KLXR5cGUgTWVhc3VyZVN1YnRyYWN0aW9uPFEsIFM+Cit0eXBlIE1lYXN1cmVTdWJ0cmFjdGlvbjxROiBRdWFudGl0eSwgUzogU2NhbGU+CiAgID0gTWVhc3VyZURpZmZlcmVuY2UgeyB2YWx1ZTogTWVhc3VyZTxRLCBTLCBOYXQ+IH0KICAgfCBNZWFzdXJlU3VidHJhaGVuZEV4Y2VlZHNNaW51ZW5kCiAKLWZuIG1lYXN1cmVfc3ViPFEsIFM+KGE6IE1lYXN1cmU8USwgUywgTmF0PiwgYjogTWVhc3VyZTxRLCBTLCBOYXQ+KSAtPiBNZWFzdXJlU3VidHJhY3Rpb248USwgUz4geworZm4gbWVhc3VyZV9zdWI8UTogUXVhbnRpdHksIFM6IFNjYWxlPihhOiBNZWFzdXJlPFEsIFMsIE5hdD4sIGI6IE1lYXN1cmU8USwgUywgTmF0PikgLT4gTWVhc3VyZVN1YnRyYWN0aW9uPFEsIFM+IHsKICAgaWYgYS5jb3VudCA8IGIuY291bnQgewogICAgIE1lYXN1cmVTdWJ0cmFoZW5kRXhjZWVkc01pbnVlbmQKICAgfSBlbHNlIHsK
B64K
git apply /tmp/kv90.patch; echo "kv90_rc=$?"
echo "matrix=OFF (corpus population must stay pure)"
timeout 1500 ./target/release/gunbc test //gunbc/instruments:v2-native-census-scoped > /tmp/cen_k.out 2>/tmp/cen_k.err
echo "census_kind_rc=$?"
echo "--- cen_k.err tail (r15+ relay) ---"; tail -30 /tmp/cen_k.err | fold -w 195 | sed "s/^/CENKERR /"; echo "--- cen_k.out head ---"; head -8 /tmp/cen_k.out | fold -w 195 | sed "s/^/CENKOUT /"; grep -ac "panic\|Panicked" /tmp/cen_k.err | sed "s/^/CENK_PANIC_LINES=/"; grep -a "panic\|Panicked" /tmp/cen_k.err | head -6 | fold -w 300 | sed "s/^/CENKPANIC /"
cat /tmp/cen_k.out /tmp/cen_k.err > /tmp/kin.txt
python3 /tmp/analyze.py > /tmp/analysis_kin.txt 2>&1; echo "analyze_kin_rc=$?"
grep -a "KIN_total\|KIN_cause\|GROUP" /tmp/analysis_kin.txt | head -20 | cut -c1-190
grep -a native-census-infer /tmp/kin.txt | grep -a "std.measure\|std.realization" | cut -c1-400 | sed "s/^/PROG_KIN /"
grep -a "RAVENPROG" /tmp/kin.txt | cut -c1-400 | sed "s/^/MINT_KIN /" | sort | uniq -c
grep -ao "|K [^)]*" /tmp/kin.txt | cut -c1-70 | sort | uniq -c | sort -rn | head -30 | sed "s/^/KINDLINK /"
grep -ao "|K [^)]*" /tmp/unk.txt | cut -c1-70 | sort | uniq -c | sort -rn | head -12 | sed "s/^/KINDLINK_UNK /"
echo "KIN_capture_bytes=$(wc -c < /tmp/cen_k.out) $(wc -c < /tmp/cen_k.err)"
echo "KIN_infer_lines_out=$(grep -ac native-census-infer /tmp/cen_k.out) KIN_infer_lines_err=$(grep -ac native-census-infer /tmp/cen_k.err)"
echo "--- [native-census-infer] watched-module outcomes, kinded rung (pre-existing stderr relay, zero minting) ---"
grep -a native-census-infer /tmp/kin.txt | grep -a "std.measure\|std.realization" | cut -c1-400 | sed "s/^/PROGKIN /"
echo "--- registry reach: effect-summary-incomplete rows naming std.measure callees ---"
echo "REGISTRY_INCOMPLETE_TOTAL=$(grep -ac "effect summary incomplete" /tmp/kin.txt)"
grep -a "effect summary incomplete" /tmp/kin.txt | cut -c1-300 | sed "s/^/EFFECTE /"
echo "REGISTRY_DISTINCT_MEASURE_CALLEES=$(grep -a \"effect summary incomplete\" /tmp/kin.txt | grep -oE \"std.measure.[a-z_]+\" | sort -u | wc -l)"

echo "=== THE RED/POSITIVE CONTROL: the two dag/std/measure.dag rows ==="
echo "rows mentioning std/measure.dag (expect NONE after the fix; 2 class-Inhabitants rows at main):"
grep -a "dag/std/measure.dag" /tmp/kin.txt | grep -av "RAVEN" | head -10 | cut -c1-600
echo "MEASURE_PATH_ROWS=$(grep -ac 'dag/std/measure.dag' /tmp/kin.txt)"
echo "the two measure_sub offsets 6248 / 6251 (expect absent):"
grep -a "|P o=" /tmp/kin.txt | sort -u | head -40 | cut -c1-600

echo "=== WITNESS ROWS: the positive and the three negatives ==="
for r in a_forwarded_parameter_declaring_the_slots_kind_is_admitted \
         a_forwarded_parameter_declaring_a_different_kind_must_refuse \
         a_forwarded_unkinded_parameter_must_refuse \
         a_forwarded_parameter_whose_kind_only_looks_like_the_slots_kind_must_refuse \
         a_forwarded_parameter_whose_kind_comes_by_import_is_admitted \
         a_forwarded_parameter_naming_an_unimported_kind_must_refuse; do
  timeout 600 ./target/release/gunbc test "//test/claim/type_argument_kind_inhabitance_witness:$r" > /tmp/w_$r.out 2>/tmp/w_$r.err
  echo "WITNESS $r rc=$?"
  tail -3 /tmp/w_$r.out | fold -w 190 | sed 's/^/    /'
  grep -a "refus\|did not hold\|FAILED" /tmp/w_$r.err | head -2 | fold -w 190 | sed 's/^/    err: /'
done

# The regenerated mirrors are the round's own bytes and they have to come home: ctrl-build copies
# nothing back across the runner boundary, so the diff is emitted gz+base64 on stdout with its byte
# count and md5, and the local side lands it as a commit and re-runs to prove the fixed point.
echo "@@MIRROR_B64_BEGIN bytes=$(git diff -- src/v1/stage0/src/ | wc -c) md5=$(git diff -- src/v1/stage0/src/ | md5sum | cut -d' ' -f1)"
git diff -- src/v1/stage0/src/ | gzip -9 | base64 -w0
echo ""
echo "@@MIRROR_B64_END"

git checkout -- dag/std/measure.dag src/v1/stage0/src/ 2>/dev/null
echo "R5_DONE tag=$TAG"
