#!/usr/bin/env bash
# raven_n7loc.sh <tag> -- the N7 infer-wall LOCATOR probe run (Manager 3 msg_471b91d6).
#
# ONE instrument, all seats: the census-scoped CLI verb takes subjects from argv, so the run is
# ./target/release/gunbc census-scoped <labels...> dag src/v2 and the per-module stderr line carries
# the new raven_fatal key (probe commit) -- the FATAL Diagnostic, with its NodeLocus node, for every
# infer-refused module in the union closure. No new claim binary; no census rebuild per subject.
#
# THE READINGS THAT MUST APPEAR or the run is void, not a zero:
#   RUNNER_HEAD            = the pushed probe sha (b474f8f136ed lineage)
#   authority markers      the two .dag edits present in the authority AND in the regenerated mirror
#   build1/round/build2    rc=0 each
#   LOC_TOTAL_LINES>0      the [native-census-infer] lines came through (stderr, harness keeps it)
#   RAVEN_FATAL_PRESENT>0  at least one infer-refused module carries the fatal (subjects include std.decl_ref)
set -u
TAG="${1:-noname}"
cd "$(dirname "$0")"

MT_KB=$(awk '/MemTotal/{print $2}' /proc/meminfo)
echo "tag=$TAG MemTotal_MB=$((MT_KB/1024))"
echo "RUNNER_HEAD=$(git rev-parse HEAD)"
echo "RUNNER_BRANCH=$(git rev-parse --abbrev-ref HEAD)"
git status --short | head -8
test "$MT_KB" -ge 41943040 || { echo ABORT_MEMORY; exit 42; }

ROOT=$(awk -F: '$2=="memory"{print $3}' /proc/self/cgroup)
case "$ROOT" in /) CGD=/sys/fs/cgroup;; *) CGD=/sys/fs/cgroup${ROOT};; esac
printf '+memory\n' > "$CGD/cgroup.subtree_control" 2>/dev/null || true
CG="$CGD/raven_n7loc_${TAG}"
mkdir -p "$CG" || { echo ABORT_CGROUP; exit 42; }
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

echo "=== the .dag authority carries the probe, and the COMMITTED mirror does not have to ==="
grep -c "native_census_infer_fatal_probe" src/v2/compiler/source_root_eval.dag | sed 's/^/authority_helper=/'
grep -c "raven_fatal" src/v1/05_emit_rust.dag | sed 's/^/authority_key=/'

echo "=== phase A: build the round host from the committed mirrors ==="
cargo build --release -p v1-compiler --bins > /tmp/b1.log 2>&1; echo "build1_rc=$?"
if grep -qaE "^error(\[|:)" /tmp/b1.log; then echo ABORT_BUILD1; grep -aE -A 14 "^error(\[|:)" /tmp/b1.log | head -90 | fold -w 195; exit 46; fi
test -x ./target/release/claim_executor || { echo ABORT_NO_EXEC; exit 46; }

echo "=== the round: regenerate the mirrors from the .dag authority ==="
rm -rf target/stage0-regen-candidate
timeout 2100 ./target/release/claim_executor --regen-round-cost --source-root dag --source-root src/v2 > /tmp/round.out 2>/tmp/round.err
echo "round_rc=$?"
tail -6 /tmp/round.out | fold -w 195
grep -a "drift" /tmp/round.out | head -4 | fold -w 195
echo "--- round.err (head 60) ---"; head -60 /tmp/round.err | fold -w 195
echo "--- round.err tail (40) ---"; tail -40 /tmp/round.err | fold -w 195

echo "=== the REGENERATED mirror carries the probe (assert before trusting anything below) ==="
M=src/v1/stage0/src/v1_compiler_emit_rust.rs
test -f "$M" && echo "mirror_present $M" || echo "mirror_MISSING $M"
grep -c "native_census_infer_fatal_probe" "$M" 2>/dev/null | sed 's/^/mirror_fatal=/'
grep -c "raven_fatal" "$M" 2>/dev/null | sed 's/^/mirror_key=/'
grep -c "raven_chain" "$M" 2>/dev/null | sed 's/^/mirror_chain=/'
grep -c "native_census_infer_loci_rows" "$M" 2>/dev/null | sed 's/^/mirror_loci=/'

echo "=== phase B: rebuild from what the round installed ==="
cargo build --release -p v1-compiler --bins > /tmp/b2.log 2>&1; echo "build2_rc=$?"
if grep -qaE "^error(\[|:)" /tmp/b2.log; then echo ABORT_BUILD2; grep -aE -A 14 "^error(\[|:)" /tmp/b2.log | head -90 | fold -w 195; exit 46; fi
test -x ./target/release/gunbc || { echo ABORT_NO_GUNBC; exit 46; }

echo "=== sanctioned census route: the instrument (drives the emitted compiler's census-scoped verb; per-module stderr lines land in loc.err, the route M3 read for census.err). SUBJECTS is the EXTRACTION filter, not argv: the instrument's acquired closure (the census population, 164 modules) contains every module listed, so each one's [native-census-infer] line carries the raven_* keys ==="
timeout 3300 ./target/release/gunbc test //gunbc/instruments:v2-native-census-scoped > /tmp/loc.out 2>/tmp/loc.err
echo "loc_rc=$?"
echo "loc_stdout_bytes=$(wc -c < /tmp/loc.out) loc_stderr_bytes=$(wc -c < /tmp/loc.err)"
echo "stderr_infer_lines=$(grep -ac '\[native-census-infer\]' /tmp/loc.err)"
echo "--- loc.err head (30) ---"; head -30 /tmp/loc.err | fold -w 195
echo "--- loc.out head (10) ---"; head -10 /tmp/loc.out | fold -w 195
head -3 /tmp/loc.out | cut -c1-190
grep -a "census_scope" /tmp/loc.out | head -2 | cut -c1-190

echo "=== extraction: raven_fatal per module ==="
base64 -d > /tmp/n7_loc_extract.py <<'B64X'
aW1wb3J0IGpzb24sIHJlLCBzeXMKcGF0aCA9IHN5cy5hcmd2WzFdCnN1YmogPSBzZXQoc3lzLmFyZ3ZbMl0uc3BsaXQoIiwiKSkKcGF0ID0gcmUuY29tcGlsZShyIl5cW25hdGl2ZS1jZW5zdXMtaW5mZXJcXSAoXHsuKlx9KVxzKiQiKQpyb3dzID0gW10KdG90YWwgPSAwCnByZXNlbnQgPSAwCndpdGggb3BlbihwYXRoLCBlbmNvZGluZz0idXRmLTgiLCBlcnJvcnM9InJlcGxhY2UiKSBhcyBmaDoKICAgIGZvciBsaW5lIGluIGZoOgogICAgICAgIG0gPSBwYXQubWF0Y2gobGluZS5yc3RyaXAoKSkKICAgICAgICBpZiBub3QgbToKICAgICAgICAgICAgY29udGludWUKICAgICAgICB0b3RhbCArPSAxCiAgICAgICAgdHJ5OgogICAgICAgICAgICBqID0ganNvbi5sb2FkcyhtLmdyb3VwKDEpKQogICAgICAgIGV4Y2VwdCBFeGNlcHRpb246CiAgICAgICAgICAgIHByaW50KCJCQURKU09OICIgKyBsaW5lWzozMDBdKQogICAgICAgICAgICBjb250aW51ZQogICAgICAgIHJvd3MuYXBwZW5kKGopCiAgICAgICAgaWYgai5nZXQoInJhdmVuX2ZhdGFsIik6CiAgICAgICAgICAgIHByZXNlbnQgKz0gMQoKZGVmIGJyaWVmKHYsIGxpbT05MDApOgogICAgcyA9IGpzb24uZHVtcHModiwgc2VwYXJhdG9ycz0oIiwiLCAiOiIpKQogICAgcmV0dXJuIHMgaWYgbGVuKHMpIDw9IGxpbSBlbHNlIHNbOmxpbV0gKyAiLi4uVFJVTkMiCgpkZWYgbm9kZV9zdW1tYXJ5KGQpOgogICAgYXQgPSBkLmdldCgiYXQiKSBvciB7fQogICAgaWYgbm90IGlzaW5zdGFuY2UoYXQsIGRpY3QpOgogICAgICAgIHJldHVybiAiYXQ9IiArIGJyaWVmKGF0LCAyMDApCiAgICB2YXIgPSBhdC5nZXQoIl92YXJpYW50IiwgIj8iKQogICAgYW5jaG9yID0gYXQuZ2V0KCJhbmNob3IiKQogICAgaWYgbm90IGlzaW5zdGFuY2UoYW5jaG9yLCBkaWN0KToKICAgICAgICByZXR1cm4gdmFyICsgIiBhdD0iICsgYnJpZWYoYXQsIDMwMCkKICAgIG4gPSBhbmNob3IKICAgIGtpZHMgPSBuLmdldCgiY2hpbGRyZW4iKSBvciBbXQogICAga2lkX25hbWVzID0gW3N0cihrLmdldCgibmFtZSIpKSBmb3IgayBpbiBraWRzWzo2XSBpZiBpc2luc3RhbmNlKGssIGRpY3QpXQogICAga2luZCA9IG4uZ2V0KCJraW5kIikKICAgIHBhcmFtcyA9IG4uZ2V0KCJwYXJhbXMiKSBvciBbXQogICAgcmV0dXJuICh2YXIgKyAiIG5vZGUgbmFtZT0iICsgc3RyKG4uZ2V0KCJuYW1lIikpICsKICAgICAgICAgICAgIiBpZGVudD0iICsgc3RyKG4uZ2V0KCJpZGVudCIpKSArCiAgICAgICAgICAgICIga2luZD0iICsgYnJpZWYoa2luZCwgMjIwKSArCiAgICAgICAgICAgICIgcGFyYW1zPSIgKyBzdHIobGVuKHBhcmFtcykpICsKICAgICAgICAgICAgIiBjaGlsZHJlbj0iICsgc3RyKGxlbihraWRzKSkgKwogICAgICAgICAgICAiIGtpZF9uYW1lcz0iICsgYnJpZWYoa2lkX25hbWVzLCAyNjApICsKICAgICAgICAgICAgIiBvY2M9IiArIGJyaWVmKG4uZ2V0KCJvY2N1cnJlbmNlX2lkZW50aXR5IiksIDE2MCkpCgpkZWYgY29ycmVjdGlvbl9ub2RlX2VkZ2VzKGQpOgogICAgIyBzZWF0IDE2OiB0aGUgZW1pdC1zaXRlIGRpc2NyaW1pbmF0b3IgcmlkZXMgdGhlIGNvcnJlY3Rpb24gbm9kZSBhcyBhdXRob3JlZCBlZGdlcwogICAgYyA9IGQuZ2V0KCJjb3JyZWN0aW9uIikgb3Ige30KICAgIGFuID0gYy5nZXQoImF1dGhvcmVkX25vZGUiKSBvciB7fQogICAgaWYgbm90IGlzaW5zdGFuY2UoYW4sIGRpY3QpOgogICAgICAgIHJldHVybiAiY29ycl9ub2RlPSIgKyBicmllZihhbiwgMTIwKQogICAgZWRnZXMgPSB7fQogICAgZm9yIGUgaW4gKGFuLmdldCgiZWRnZXMiKSBvciBbXSk6CiAgICAgICAgaWYgaXNpbnN0YW5jZShlLCBkaWN0KToKICAgICAgICAgICAgcG4gPSBlLmdldCgicHJvcGVydHlfbmFtZSIpCiAgICAgICAgICAgIHQgPSBlLmdldCgidG8iKSBvciB7fQogICAgICAgICAgICBlZGdlc1tzdHIocG4pXSA9IGJyaWVmKHsibmFtZSI6IHQuZ2V0KCJuYW1lIiksICJpZGVudCI6IHQuZ2V0KCJpZGVudCIpLCAia2luZCI6IHQuZ2V0KCJraW5kIil9LCAyNDApCiAgICByZXR1cm4gImNvcnJfaGVhZD0iICsgc3RyKGFuLmdldCgiaGVhZF9uYW1lIikpICsgIiBlZGdlcz0iICsgYnJpZWYoZWRnZXMsIDE0MDApCgpwcmludCgiTE9DX1RPVEFMX0xJTkVTICIgKyBzdHIodG90YWwpICsgIiBSQVZFTl9GQVRBTF9QUkVTRU5UICIgKyBzdHIocHJlc2VudCkpCmZvciBqIGluIHJvd3M6CiAgICBkID0gai5nZXQoInJhdmVuX2ZhdGFsIikKICAgIGlmIG5vdCBkOgogICAgICAgIGNvbnRpbnVlCiAgICBjaCA9IGouZ2V0KCJyYXZlbl9jaGFpbiIpIG9yIHt9CiAgICBubGlua3MgPSAxICsgbGVuKGNoLmdldCgidGFpbCIpIG9yIFtdKSBpZiBjaC5nZXQoImhlYWQiKSBlbHNlIDAKICAgIHByaW50KCJMT0N8bW9kdWxlPSIgKyBzdHIoai5nZXQoIm1vZHVsZSIpKSArICJ8b3V0Y29tZT0iICsgc3RyKGouZ2V0KCJvdXRjb21lIikpICsKICAgICAgICAgICJ8bGlua3M9IiArIHN0cihubGlua3MpICsgInxyZWFzb249IiArIHN0cihkLmdldCgicmVhc29uIikpICsgInxmYXRhbF9hdD0iICsgbm9kZV9zdW1tYXJ5KGQpKQpwcmludCgiLS0tIHBlci1saW5rIHJlYXNvbnMgKyBhbmNob3JzIChhbGwgbW9kdWxlcywgc2hvcnQpIC0tLSIpCmZvciBqIGluIHJvd3M6CiAgICBjaCA9IGouZ2V0KCJyYXZlbl9jaGFpbiIpCiAgICBpZiBub3QgY2ggb3Igbm90IGNoLmdldCgiaGVhZCIpOgogICAgICAgIGNvbnRpbnVlCiAgICBsaW5rcyA9IFtjaC5nZXQoImhlYWQiKV0gKyAoY2guZ2V0KCJ0YWlsIikgb3IgW10pCiAgICBmb3IgaSwgbGluayBpbiBlbnVtZXJhdGUobGlua3MpOgogICAgICAgIGlmIG5vdCBpc2luc3RhbmNlKGxpbmssIGRpY3QpOgogICAgICAgICAgICBjb250aW51ZQogICAgICAgIHByaW50KCJMSU5LfG1vZHVsZT0iICsgc3RyKGouZ2V0KCJtb2R1bGUiKSkgKyAifGk9IiArIHN0cihpKSArCiAgICAgICAgICAgICAgInxyZWFzb249IiArIHN0cihsaW5rLmdldCgicmVhc29uIikpICsgInxhdD0iICsgbm9kZV9zdW1tYXJ5KGxpbmspICsKICAgICAgICAgICAgICAifCIgKyBjb3JyZWN0aW9uX25vZGVfZWRnZXMobGluaykpCnByaW50KCItLS0gbG9jaSByb3dzIChyZWZ1c2VkIG1vZHVsZXMgb25seSkgLS0tIikKZm9yIGogaW4gcm93czoKICAgIGxyID0gai5nZXQoInJhdmVuX2xvY2lfcm93cyIpIG9yIFtdCiAgICBpZiBscjoKICAgICAgICBwcmludCgiTE9DSXxtb2R1bGU9IiArIHN0cihqLmdldCgibW9kdWxlIikpICsgInxuPSIgKyBzdHIobGVuKGxyKSkgKyAifCIgKyBicmllZihsciwgMTIwMCkpCnByaW50KCItLS0gZnVsbCBmYXRhbCBKU09OLCBzdWJqZWN0IG1vZHVsZXMgLS0tIikKZm9yIGogaW4gcm93czoKICAgIGQgPSBqLmdldCgicmF2ZW5fZmF0YWwiKQogICAgaWYgbm90IGQ6CiAgICAgICAgY29udGludWUKICAgIG1vZCA9IHN0cihqLmdldCgibW9kdWxlIikpCiAgICBpZiBtb2QgaW4gc3ViajoKICAgICAgICBwcmludCgiRlVMTHxtb2R1bGU9IiArIG1vZCArICJ8b3V0Y29tZT0iICsgc3RyKGouZ2V0KCJvdXRjb21lIikpICsgInwiICsgYnJpZWYoZCwgNDAwMCkpCnByaW50KCItLS0gZnVsbCBjaGFpbiBKU09OLCBzdWJqZWN0IG1vZHVsZXMgLS0tIikKZm9yIGogaW4gcm93czoKICAgIGNoID0gai5nZXQoInJhdmVuX2NoYWluIikKICAgIGlmIG5vdCBjaCBvciBub3QgY2guZ2V0KCJoZWFkIik6CiAgICAgICAgY29udGludWUKICAgIG1vZCA9IHN0cihqLmdldCgibW9kdWxlIikpCiAgICBpZiBtb2QgaW4gc3ViajoKICAgICAgICBwcmludCgiQ0hBSU58bW9kdWxlPSIgKyBtb2QgKyAifCIgKyBicmllZihjaCwgNjAwMCkpCnByaW50KCItLS0gb3V0Y29tZSB0YWxseSAtLS0iKQpmcm9tIGNvbGxlY3Rpb25zIGltcG9ydCBDb3VudGVyCmMgPSBDb3VudGVyKChzdHIoai5nZXQoIm1vZHVsZSIpKSBpZiBzdHIoai5nZXQoIm1vZHVsZSIpKSBpbiBzdWJqIGVsc2UgIihvdGhlcikiKSArICIvIiArIHN0cihqLmdldCgib3V0Y29tZSIpKSBmb3IgaiBpbiByb3dzKQpmb3IgayBpbiBzb3J0ZWQoYyk6CiAgICBwcmludCgiVEFMTFkgIiArIGsgKyAiICIgKyBzdHIoY1trXSkpCg==
B64X
python3 /tmp/n7_loc_extract.py /tmp/loc.err "std.decl_ref,std.nat,std.keyed_row,v2.std.verdict,gunbc.non_fold_residue,v2.std.algebra_structure_signature,v2.std.fold_assembly,v2.std.occurrence_id,v2.compiler.arm_resolution,v2.compiler.body_producer_forward,gunbc.structural_realization_bindings" > /tmp/loc_report.txt 2>&1
echo "extract_rc=$?"
sed -n 1,60p /tmp/loc_report.txt

echo "=== subject-module stderr lines verbatim (the five, one line each, head 1500 chars) ==="
for m in std.decl_ref std.nat std.keyed_row v2.std.verdict gunbc.non_fold_residue v2.std.algebra_structure_signature v2.std.fold_assembly v2.std.occurrence_id v2.compiler.arm_resolution v2.compiler.body_producer_forward gunbc.structural_realization_bindings; do
  grep -a "\[native-census-infer\]" /tmp/loc.err | grep -a "\"module\": \"$m\"\|\"module\":\"$m\"" | head -1 | cut -c1-1500
done

echo "=== the whole-closure tally of the raven_fatal key ==="
grep -a "RAVEN_FATAL_PRESENT" /tmp/loc_report.txt

echo "=== DONE tag=$TAG ==="
