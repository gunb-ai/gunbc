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

CGSELF=$(awk -F: '{ if ($1=="0") {print $3; exit} else if ($2=="memory") {print $3; exit} }' /proc/self/cgroup)
if grep -q '^0::' /proc/self/cgroup; then CGBASE=/sys/fs/cgroup; else CGBASE=$(awk -F: '$2=="memory"{print "/sys/fs/cgroup/memory"; exit}' /proc/self/cgroup); fi
[ -n "${CGBASE:-}" ] || CGBASE=/sys/fs/cgroup
CGPATH="$CGBASE${CGSELF}"
BOUND=$(cat "$CGPATH/memory.max" 2>/dev/null || cat "$CGPATH/memory.limit_in_bytes" 2>/dev/null || echo unreadable)
echo "CGROUP_INHERITED path=$CGPATH bound=$BOUND (accept 42..128 GiB; systemd-run MemoryMax=64G answers 68719476736)"
case "$BOUND" in ''|max|unreadable) echo "ABORT_CGROUP_UNBOUND bound=$BOUND"; exit 42 ;; esac
awk -v b="$BOUND" 'BEGIN { exit !(b+0 >= 45035996262 && b+0 <= 137438953472) }' || { echo "ABORT_CGROUP_RANGE bound=$BOUND"; exit 42; }
export RAVEN_CGROUP_PATH="$CGPATH"

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
aW1wb3J0IGpzb24sIHJlLCBzeXMKcGF0aCA9IHN5cy5hcmd2WzFdCnN1YmogPSBzZXQoc3lzLmFyZ3ZbMl0uc3BsaXQoIiwiKSkKcGF0ID0gcmUuY29tcGlsZShyIl5cW25hdGl2ZS1jZW5zdXMtaW5mZXJcXSAoXHsuKlx9KVxzKiQiKQpyb3dzID0gW10KdG90YWwgPSAwCnByZXNlbnQgPSAwCndpdGggb3BlbihwYXRoLCBlbmNvZGluZz0idXRmLTgiLCBlcnJvcnM9InJlcGxhY2UiKSBhcyBmaDoKICAgIGZvciBsaW5lIGluIGZoOgogICAgICAgIG0gPSBwYXQubWF0Y2gobGluZS5yc3RyaXAoKSkKICAgICAgICBpZiBub3QgbToKICAgICAgICAgICAgY29udGludWUKICAgICAgICB0b3RhbCArPSAxCiAgICAgICAgdHJ5OgogICAgICAgICAgICBqID0ganNvbi5sb2FkcyhtLmdyb3VwKDEpKQogICAgICAgIGV4Y2VwdCBFeGNlcHRpb246CiAgICAgICAgICAgIHByaW50KCJCQURKU09OICIgKyBsaW5lWzozMDBdKQogICAgICAgICAgICBjb250aW51ZQogICAgICAgIHJvd3MuYXBwZW5kKGopCiAgICAgICAgaWYgai5nZXQoInJhdmVuX2ZhdGFsIik6CiAgICAgICAgICAgIHByZXNlbnQgKz0gMQoKZGVmIGJyaWVmKHYsIGxpbT05MDApOgogICAgcyA9IGpzb24uZHVtcHModiwgc2VwYXJhdG9ycz0oIiwiLCAiOiIpKQogICAgcmV0dXJuIHMgaWYgbGVuKHMpIDw9IGxpbSBlbHNlIHNbOmxpbV0gKyAiLi4uVFJVTkMiCgpkZWYgcmVhbF9ub2RlKGQpOgogICAgYXQgPSBkLmdldCgiYXQiKSBvciB7fQogICAgaWYgbm90IGlzaW5zdGFuY2UoYXQsIGRpY3QpOgogICAgICAgIHJldHVybiBOb25lCiAgICBhID0gYXQuZ2V0KCJhbmNob3IiKQogICAgaWYgaXNpbnN0YW5jZShhLCBkaWN0KSBhbmQgaXNpbnN0YW5jZShhLmdldCgiYXQiKSwgZGljdCk6CiAgICAgICAgcmV0dXJuIGFbImF0Il0KICAgIGlmIGlzaW5zdGFuY2UoYSwgZGljdCk6CiAgICAgICAgcmV0dXJuIGEKICAgIHJldHVybiBOb25lCgpkZWYga2luZF9icmllZihrKToKICAgIGlmIG5vdCBpc2luc3RhbmNlKGssIGRpY3QpOgogICAgICAgIHJldHVybiBzdHIoaykKICAgIHYgPSBrLmdldCgiX3ZhcmlhbnQiKQogICAgaWYgdiA9PSAiQXRvbSI6CiAgICAgICAgcmV0dXJuICJBdG9tOiIgKyBzdHIoay5nZXQoImNvbm5lY3RpdmUiLCB7fSkuZ2V0KCJpZGVudGl0eSIpIGlmIGlzaW5zdGFuY2Uoay5nZXQoImNvbm5lY3RpdmUiKSwgZGljdCkgZWxzZSBrLmdldCgiY29ubmVjdGl2ZSIpKQogICAgYyA9IGsuZ2V0KCJjb25uZWN0aXZlIikKICAgIGlmIGlzaW5zdGFuY2UoYywgZGljdCk6CiAgICAgICAgcmV0dXJuIHN0cih2KSArICI6IiArIHN0cihjLmdldCgiX3ZhcmlhbnQiKSkgKyAiOiIgKyBzdHIoYy5nZXQoImlkZW50aXR5IikpWzo2MF0KICAgIHJldHVybiBzdHIodikKCmRlZiBub2RlX3N1bW1hcnkoZCk6CiAgICBuID0gcmVhbF9ub2RlKGQpCiAgICBpZiBuIGlzIE5vbmU6CiAgICAgICAgcmV0dXJuICJhdD0iICsgYnJpZWYoZC5nZXQoImF0IiksIDIwMCkKICAgIGtpZHMgPSBuLmdldCgiY2hpbGRyZW4iKSBvciBbXQogICAga2lkX2Rlc2NzID0gW10KICAgIGZvciBrIGluIGtpZHNbOjhdOgogICAgICAgIGlmIGlzaW5zdGFuY2UoaywgZGljdCk6CiAgICAgICAgICAgIGxibCA9IGsuZ2V0KCJsYWJlbCIpIG9yIHt9CiAgICAgICAgICAgIGxuYW1lID0gbGJsLmdldCgibmFtZSIpIGlmIGlzaW5zdGFuY2UobGJsLCBkaWN0KSBlbHNlIE5vbmUKICAgICAgICAgICAgdCA9IGsuZ2V0KCJ0YXJnZXQiKSBvciB7fQogICAgICAgICAgICBraWRfZGVzY3MuYXBwZW5kKHN0cihsbmFtZSkgKyAiLT4iICsgc3RyKHQuZ2V0KCJuYW1lIikpICsgIi8iICsga2luZF9icmllZih0LmdldCgia2luZCIpKSArICIvIiArIG9jY192ZXJiYXRpbSh0LmdldCgib2NjdXJyZW5jZV9pZCIpKSkKICAgIHJldHVybiAoImtpbmQ9IiArIGtpbmRfYnJpZWYobi5nZXQoImtpbmQiKSkgKwogICAgICAgICAgICAiIG9jYz0iICsgb2NjX3ZlcmJhdGltKG4uZ2V0KCJvY2N1cnJlbmNlX2lkIikpICsKICAgICAgICAgICAgIiBjaGlsZHJlbj0iICsgc3RyKGxlbihraWRzKSkgKwogICAgICAgICAgICAiIGtpZHM9IiArIGJyaWVmKGtpZF9kZXNjcywgOTAwKSkKCmRlZiBvY2NfdmVyYmF0aW0odik6CiAgICByZXR1cm4ganNvbi5kdW1wcyh2LCBzZXBhcmF0b3JzPSgiLCIsICI6IikpCgpkZWYgZmF0YWxfb2NjdXJyZW5jZXMoZCk6CiAgICBuID0gcmVhbF9ub2RlKGQpIG9yIHt9CiAgICBvdXQgPSBbImFuY2hvcl9vY2M9IiArIG9jY192ZXJiYXRpbShuLmdldCgib2NjdXJyZW5jZV9pZCIpKSArICIgYW5jaG9yX2tpbmQ9IiArIGtpbmRfYnJpZWYobi5nZXQoImtpbmQiKSldCiAgICBjID0gZC5nZXQoImNvcnJlY3Rpb24iKSBvciB7fQogICAgYW4gPSBjLmdldCgibm9kZSIpIG9yIHt9CiAgICBmb3IgZSBpbiAoYW4uZ2V0KCJjaGlsZHJlbiIpIG9yIFtdKToKICAgICAgICBpZiBub3QgaXNpbnN0YW5jZShlLCBkaWN0KToKICAgICAgICAgICAgY29udGludWUKICAgICAgICBsYmwgPSBlLmdldCgibGFiZWwiKSBvciB7fQogICAgICAgIG5hbWUgPSBsYmwuZ2V0KCJuYW1lIikgaWYgaXNpbnN0YW5jZShsYmwsIGRpY3QpIGVsc2UgTm9uZQogICAgICAgIHQgPSBlLmdldCgidGFyZ2V0Iikgb3Ige30KICAgICAgICBpZiBpc2luc3RhbmNlKHQsIGRpY3QpOgogICAgICAgICAgICBvdXQuYXBwZW5kKHN0cihuYW1lKSArICJ8dGFyZ2V0X29jYz0iICsgb2NjX3ZlcmJhdGltKHQuZ2V0KCJvY2N1cnJlbmNlX2lkIikpICsgInx0YXJnZXRfa2luZD0iICsga2luZF9icmllZih0LmdldCgia2luZCIpKSkKICAgIHJldHVybiAiIHx8ICIuam9pbihvdXQpCgpkZWYgY29ycmVjdGlvbl9ub2RlX2VkZ2VzKGQpOgogICAgYyA9IGQuZ2V0KCJjb3JyZWN0aW9uIikgb3Ige30KICAgIGlmIGMuZ2V0KCJfdmFyaWFudCIpICE9ICJTdWdnZXN0ZWQiOgogICAgICAgIHJldHVybiAiY29ycj0iICsgYnJpZWYoYywgMTAwKQogICAgYW4gPSBjLmdldCgibm9kZSIpIG9yIHt9CiAgICBraWRzID0gYW4uZ2V0KCJjaGlsZHJlbiIpIG9yIFtdCiAgICBwYXJ0cyA9IFtdCiAgICBmb3IgZSBpbiBraWRzOgogICAgICAgIGlmIG5vdCBpc2luc3RhbmNlKGUsIGRpY3QpOgogICAgICAgICAgICBjb250aW51ZQogICAgICAgIGxibCA9IGUuZ2V0KCJsYWJlbCIpIG9yIHt9CiAgICAgICAgbmFtZSA9IGxibC5nZXQoIm5hbWUiKSBpZiBpc2luc3RhbmNlKGxibCwgZGljdCkgZWxzZSBOb25lCiAgICAgICAgdCA9IGUuZ2V0KCJ0YXJnZXQiKSBvciB7fQogICAgICAgIGlmIG5vdCBpc2luc3RhbmNlKHQsIGRpY3QpOgogICAgICAgICAgICB0ID0ge30KICAgICAgICBwYXJ0cy5hcHBlbmQoc3RyKG5hbWUpICsgIj0+e2tpbmQ9IiArIGtpbmRfYnJpZWYodC5nZXQoImtpbmQiKSkgKwogICAgICAgICAgICAgICAgICAgICAiIG5hbWU9IiArIHN0cih0LmdldCgibmFtZSIpKSArICIgb2NjPSIgKyBvY2NfdmVyYmF0aW0odC5nZXQoIm9jY3VycmVuY2VfaWQiKSkgKyAifSIpCiAgICByZXR1cm4gImNvcnJfZWRnZXNbIiArIHN0cihsZW4ocGFydHMpKSArICJdICIgKyBicmllZihwYXJ0cywgMjYwMCkKCnByaW50KCJMT0NfVE9UQUxfTElORVMgIiArIHN0cih0b3RhbCkgKyAiIFJBVkVOX0ZBVEFMX1BSRVNFTlQgIiArIHN0cihwcmVzZW50KSkKZm9yIGogaW4gcm93czoKICAgIGQgPSBqLmdldCgicmF2ZW5fZmF0YWwiKQogICAgaWYgbm90IGQ6CiAgICAgICAgY29udGludWUKICAgIGNoID0gai5nZXQoInJhdmVuX2NoYWluIikgb3Ige30KICAgIG5saW5rcyA9IDEgKyBsZW4oY2guZ2V0KCJ0YWlsIikgb3IgW10pIGlmIGNoLmdldCgiaGVhZCIpIGVsc2UgMAogICAgcHJpbnQoIkxPQ3xtb2R1bGU9IiArIHN0cihqLmdldCgibW9kdWxlIikpICsgInxvdXRjb21lPSIgKyBzdHIoai5nZXQoIm91dGNvbWUiKSkgKwogICAgICAgICAgInxsaW5rcz0iICsgc3RyKG5saW5rcykgKyAifHJlYXNvbj0iICsgc3RyKGQuZ2V0KCJyZWFzb24iKSkgKyAifGZhdGFsX2F0PSIgKyBub2RlX3N1bW1hcnkoZCkpCnByaW50KCItLS0gZmF0YWwgb2NjdXJyZW5jZSBpZGVudGl0aWVzLCB2ZXJiYXRpbSAob2NjdXJyZW5jZV9pZCBmaXJzdCkgLS0tIikKT1JERVIgPSBbInYyLnN0ZC5vY2N1cnJlbmNlX2lkIiwgInYyLnN0ZC52ZXJkaWN0IiwgInYyLmNvbXBpbGVyLmFybV9yZXNvbHV0aW9uIiwgInYyLmNvbXBpbGVyLmJvZHlfcHJvZHVjZXJfZm9yd2FyZCIsCiAgICAgICAgICJndW5iYy5zdHJ1Y3R1cmFsX3JlYWxpemF0aW9uX2JpbmRpbmdzIiwgInYyLnN0ZC5mb2xkX2Fzc2VtYmx5IiwgInYyLnN0ZC5hbGdlYnJhX3N0cnVjdHVyZV9zaWduYXR1cmUiLAogICAgICAgICAiZ3VuYmMubm9uX2ZvbGRfcmVzaWR1ZSIsICJzdGQuZGVjbF9yZWYiLCAic3RkLm5hdCIsICJzdGQua2V5ZWRfcm93Il0KZGVmIHJhbmsobSk6CiAgICByZXR1cm4gT1JERVIuaW5kZXgobSkgaWYgbSBpbiBPUkRFUiBlbHNlIDk5CmZvciBqIGluIHNvcnRlZChyb3dzLCBrZXk9bGFtYmRhIHg6IHJhbmsoc3RyKHguZ2V0KCJtb2R1bGUiKSkpKToKICAgIGQgPSBqLmdldCgicmF2ZW5fZmF0YWwiKQogICAgaWYgbm90IGQ6CiAgICAgICAgY29udGludWUKICAgIHByaW50KCJPQ0N8bW9kdWxlPSIgKyBzdHIoai5nZXQoIm1vZHVsZSIpKSArICJ8cmVhc29uPSIgKyBzdHIoZC5nZXQoInJlYXNvbiIpKSArICJ8IiArIGZhdGFsX29jY3VycmVuY2VzKGQpKQpmcm9tIGNvbGxlY3Rpb25zIGltcG9ydCBDb3VudGVyIGFzIF9DCl90ID0gX0MoKHN0cihqLmdldCgibW9kdWxlIikpIGlmIHN0cihqLmdldCgibW9kdWxlIikpIGluIHN1YmogZWxzZSAiKG90aGVyKSIpICsgIi8iICsgc3RyKGouZ2V0KCJvdXRjb21lIikpIGZvciBqIGluIHJvd3MpCnByaW50KCItLS0gb3V0Y29tZSB0YWxseSAoZWFybHksIG5ldmVyIGRyb3duZWQpIC0tLSIpCmZvciBfayBpbiBzb3J0ZWQoX3QpOgogICAgcHJpbnQoIlRBTExZICIgKyBfayArICIgIiArIHN0cihfdFtfa10pKQpwcmludCgiLS0tIHBlci1saW5rIHJlYXNvbnMgKyBhbmNob3JzIChhbGwgbW9kdWxlcywgc2hvcnQpIC0tLSIpCmZvciBqIGluIHJvd3M6CiAgICBjaCA9IGouZ2V0KCJyYXZlbl9jaGFpbiIpCiAgICBpZiBub3QgY2ggb3Igbm90IGNoLmdldCgiaGVhZCIpOgogICAgICAgIGNvbnRpbnVlCiAgICBsaW5rcyA9IFtjaC5nZXQoImhlYWQiKV0gKyAoY2guZ2V0KCJ0YWlsIikgb3IgW10pCiAgICBpZiBsZW4obGlua3MpID4gMTI6CiAgICAgICAgcHJpbnQoIkxJTktUUlVOQ3xtb2R1bGU9IiArIHN0cihqLmdldCgibW9kdWxlIikpICsgInx0b3RhbD0iICsgc3RyKGxlbihsaW5rcykpICsgInxzaG93aW5nPTEyIChmdWxsIGNoYWluOiBDSEFJTnwgcm93cykiKQogICAgZm9yIGksIGxpbmsgaW4gZW51bWVyYXRlKGxpbmtzWzoxMl0pOgogICAgICAgIGlmIG5vdCBpc2luc3RhbmNlKGxpbmssIGRpY3QpOgogICAgICAgICAgICBjb250aW51ZQogICAgICAgIHByaW50KCJMSU5LfG1vZHVsZT0iICsgc3RyKGouZ2V0KCJtb2R1bGUiKSkgKyAifGk9IiArIHN0cihpKSArCiAgICAgICAgICAgICAgInxyZWFzb249IiArIHN0cihsaW5rLmdldCgicmVhc29uIikpICsgInxhdD0iICsgbm9kZV9zdW1tYXJ5KGxpbmspICsKICAgICAgICAgICAgICAifCIgKyBjb3JyZWN0aW9uX25vZGVfZWRnZXMobGluaykpCnByaW50KCItLS0gbG9jaSByb3dzIChyZWZ1c2VkIG1vZHVsZXMgb25seSkgLS0tIikKZm9yIGogaW4gcm93czoKICAgIGxyID0gai5nZXQoInJhdmVuX2xvY2lfcm93cyIpIG9yIFtdCiAgICBpZiBscjoKICAgICAgICBwcmludCgiTE9DSXxtb2R1bGU9IiArIHN0cihqLmdldCgibW9kdWxlIikpICsgInxuPSIgKyBzdHIobGVuKGxyKSkgKyAifCIgKyBicmllZihsciwgMTIwMCkpCnByaW50KCItLS0gZnVsbCBmYXRhbCBKU09OLCBzdWJqZWN0IG1vZHVsZXMgLS0tIikKZm9yIGogaW4gcm93czoKICAgIGQgPSBqLmdldCgicmF2ZW5fZmF0YWwiKQogICAgaWYgbm90IGQ6CiAgICAgICAgY29udGludWUKICAgIG1vZCA9IHN0cihqLmdldCgibW9kdWxlIikpCiAgICBpZiBtb2QgaW4gc3ViajoKICAgICAgICBjYXAgPSA0MDAwMDAgaWYgbW9kIGluICgic3RkLmRlY2xfcmVmIiwgInYyLnN0ZC5vY2N1cnJlbmNlX2lkIiwgImd1bmJjLnN0cnVjdHVyYWxfcmVhbGl6YXRpb25fYmluZGluZ3MiLCAidjIuc3RkLnZlcmRpY3QiKSBlbHNlIDgwMDAKICAgICAgICBwcmludCgiRlVMTHxtb2R1bGU9IiArIG1vZCArICJ8b3V0Y29tZT0iICsgc3RyKGouZ2V0KCJvdXRjb21lIikpICsgInwiICsgYnJpZWYoZCwgY2FwKSkKcHJpbnQoIi0tLSBmdWxsIGNoYWluIEpTT04sIHN1YmplY3QgbW9kdWxlcyAtLS0iKQpmb3IgaiBpbiByb3dzOgogICAgY2ggPSBqLmdldCgicmF2ZW5fY2hhaW4iKQogICAgaWYgbm90IGNoIG9yIG5vdCBjaC5nZXQoImhlYWQiKToKICAgICAgICBjb250aW51ZQogICAgbW9kID0gc3RyKGouZ2V0KCJtb2R1bGUiKSkKICAgIGlmIG1vZCBpbiBzdWJqOgogICAgICAgIGNhcCA9IDYwMDAwMCBpZiBtb2QgaW4gKCJzdGQuZGVjbF9yZWYiLCAidjIuc3RkLm9jY3VycmVuY2VfaWQiLCAiZ3VuYmMuc3RydWN0dXJhbF9yZWFsaXphdGlvbl9iaW5kaW5ncyIsICJ2Mi5zdGQudmVyZGljdCIpIGVsc2UgMTIwMDAKICAgICAgICBwcmludCgiQ0hBSU58bW9kdWxlPSIgKyBtb2QgKyAifCIgKyBicmllZihjaCwgY2FwKSkKcHJpbnQoIi0tLSBvdXRjb21lIHRhbGx5IC0tLSIpCmZyb20gY29sbGVjdGlvbnMgaW1wb3J0IENvdW50ZXIKYyA9IENvdW50ZXIoKHN0cihqLmdldCgibW9kdWxlIikpIGlmIHN0cihqLmdldCgibW9kdWxlIikpIGluIHN1YmogZWxzZSAiKG90aGVyKSIpICsgIi8iICsgc3RyKGouZ2V0KCJvdXRjb21lIikpIGZvciBqIGluIHJvd3MpCmZvciBrIGluIHNvcnRlZChjKToKICAgIHByaW50KCJUQUxMWSAiICsgayArICIgIiArIHN0cihjW2tdKSkK
B64X
python3 /tmp/n7_loc_extract.py /tmp/loc.err "std.decl_ref,std.nat,std.keyed_row,v2.std.verdict,gunbc.non_fold_residue,v2.std.algebra_structure_signature,v2.std.fold_assembly,v2.std.occurrence_id,v2.compiler.arm_resolution,v2.compiler.body_producer_forward,gunbc.structural_realization_bindings" > /tmp/loc_report.txt 2>&1
echo "extract_rc=$?"
grep -a "Traceback" /tmp/loc_report.txt | head -2 | sed 's/^/EXTRACT_TRACEBACK /' || true
grep -aq "Traceback" /tmp/loc_report.txt || echo EXTRACT_CLEAN
wc -l /tmp/loc_report.txt | sed 's/^/report_lines=/'
awk 'length($0)<=400 || /^(FULL|CHAIN|OCC|LOC|LINK|LOCI|TALLY)\|/' /tmp/loc_report.txt | head -400

echo "=== subject-module stderr lines verbatim (the five, one line each, head 1500 chars) ==="
for m in std.decl_ref std.nat std.keyed_row v2.std.verdict gunbc.non_fold_residue v2.std.algebra_structure_signature v2.std.fold_assembly v2.std.occurrence_id v2.compiler.arm_resolution v2.compiler.body_producer_forward gunbc.structural_realization_bindings; do
  grep -a "\[native-census-infer\]" /tmp/loc.err | grep -a "\"module\": \"$m\"\|\"module\":\"$m\"" | head -1 | cut -c1-1500
done

echo "=== RAWB64: full verbatim stderr lines, base64 (decode locally, re-extract without another remote run) ==="
for m in std.decl_ref std.nat std.keyed_row v2.std.verdict gunbc.non_fold_residue v2.std.algebra_structure_signature v2.std.fold_assembly v2.std.occurrence_id v2.compiler.arm_resolution v2.compiler.body_producer_forward gunbc.structural_realization_bindings; do
  grep -a "\[native-census-infer\]" /tmp/loc.err | grep -a "\"module\": \"$m\",\|\"module\":\"$m\"," | head -1 | base64 -w0 | sed "s|^|RAWB64\|$m\||"
  echo ""
done

echo "=== the whole-closure tally of the raven_fatal key ==="
grep -a "RAVEN_FATAL_PRESENT" /tmp/loc_report.txt

echo "=== DONE tag=$TAG ==="
