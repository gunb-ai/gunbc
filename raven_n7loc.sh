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
echo "PROBE_BASE_DISCLOSURE the look-ahead head ee7438b33823 is NOT round-green (4 seed-inferable diagnostics, occurrence_binding_candidates.dag 954:12 960:14 1499:12 1505:14, set_contains/set_insert receivers through the map_get->map_lookup rename, fail-closed reference_closure); the probe runs at the NARROWING head ea460adcbc8f (round-green lineage) and rows are labelled by THIS RUNNER_HEAD"
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
aW1wb3J0IGpzb24sIHJlLCBzeXMKcGF0aCA9IHN5cy5hcmd2WzFdCnN1YmogPSBzZXQoc3lzLmFyZ3ZbMl0uc3BsaXQoIiwiKSkKcGF0ID0gcmUuY29tcGlsZShyIl5cW25hdGl2ZS1jZW5zdXMtaW5mZXJcXSAoXHsuKlx9KVxzKiQiKQpyb3dzID0gW10KdG90YWwgPSAwCnByZXNlbnQgPSAwCndpdGggb3BlbihwYXRoLCBlbmNvZGluZz0idXRmLTgiLCBlcnJvcnM9InJlcGxhY2UiKSBhcyBmaDoKICAgIGZvciBsaW5lIGluIGZoOgogICAgICAgIG0gPSBwYXQubWF0Y2gobGluZS5yc3RyaXAoKSkKICAgICAgICBpZiBub3QgbToKICAgICAgICAgICAgY29udGludWUKICAgICAgICB0b3RhbCArPSAxCiAgICAgICAgdHJ5OgogICAgICAgICAgICBqID0ganNvbi5sb2FkcyhtLmdyb3VwKDEpKQogICAgICAgIGV4Y2VwdCBFeGNlcHRpb246CiAgICAgICAgICAgIHByaW50KCJCQURKU09OICIgKyBsaW5lWzozMDBdKQogICAgICAgICAgICBjb250aW51ZQogICAgICAgIHJvd3MuYXBwZW5kKGopCiAgICAgICAgaWYgai5nZXQoInJhdmVuX2ZhdGFsIik6CiAgICAgICAgICAgIHByZXNlbnQgKz0gMQoKZGVmIGJyaWVmKHYsIGxpbT05MDApOgogICAgcyA9IGpzb24uZHVtcHModiwgc2VwYXJhdG9ycz0oIiwiLCAiOiIpKQogICAgcmV0dXJuIHMgaWYgbGVuKHMpIDw9IGxpbSBlbHNlIHNbOmxpbV0gKyAiLi4uVFJVTkMiCgpkZWYgbm9kZV9zdW1tYXJ5KGQpOgogICAgYXQgPSBkLmdldCgiYXQiKSBvciB7fQogICAgaWYgbm90IGlzaW5zdGFuY2UoYXQsIGRpY3QpOgogICAgICAgIHJldHVybiAiYXQ9IiArIGJyaWVmKGF0LCAyMDApCiAgICB2YXIgPSBhdC5nZXQoIl92YXJpYW50IiwgIj8iKQogICAgYW5jaG9yID0gYXQuZ2V0KCJhbmNob3IiKQogICAgaWYgbm90IGlzaW5zdGFuY2UoYW5jaG9yLCBkaWN0KToKICAgICAgICByZXR1cm4gdmFyICsgIiBhdD0iICsgYnJpZWYoYXQsIDMwMCkKICAgIG4gPSBhbmNob3IKICAgIGtpZHMgPSBuLmdldCgiY2hpbGRyZW4iKSBvciBbXQogICAga2lkX25hbWVzID0gW3N0cihrLmdldCgibmFtZSIpKSBmb3IgayBpbiBraWRzWzo2XSBpZiBpc2luc3RhbmNlKGssIGRpY3QpXQogICAga2luZCA9IG4uZ2V0KCJraW5kIikKICAgIHBhcmFtcyA9IG4uZ2V0KCJwYXJhbXMiKSBvciBbXQogICAgcmV0dXJuICh2YXIgKyAiIG5vZGUgbmFtZT0iICsgc3RyKG4uZ2V0KCJuYW1lIikpICsKICAgICAgICAgICAgIiBpZGVudD0iICsgc3RyKG4uZ2V0KCJpZGVudCIpKSArCiAgICAgICAgICAgICIga2luZD0iICsgYnJpZWYoa2luZCwgMjIwKSArCiAgICAgICAgICAgICIgcGFyYW1zPSIgKyBzdHIobGVuKHBhcmFtcykpICsKICAgICAgICAgICAgIiBjaGlsZHJlbj0iICsgc3RyKGxlbihraWRzKSkgKwogICAgICAgICAgICAiIGtpZF9uYW1lcz0iICsgYnJpZWYoa2lkX25hbWVzLCAyNjApICsKICAgICAgICAgICAgIiBvY2M9IiArIGJyaWVmKG4uZ2V0KCJvY2N1cnJlbmNlX2lkZW50aXR5IiksIDE2MCkpCgpkZWYgb2NjX3ZlcmJhdGltKHYpOgogICAgcmV0dXJuIGpzb24uZHVtcHModiwgc2VwYXJhdG9ycz0oIiwiLCAiOiIpKQoKZGVmIGZhdGFsX29jY3VycmVuY2VzKGQpOgogICAgIyBNMyBtc2dfZmRlZTdlYzI6IG9jY3VycmVuY2UgaWRlbnRpdHkgVkVSQkFUSU0gKE1pbnRlZC9QZW5kaW5nL1Byb2plY3RlZC9TeW50aGV0aWMpIGZvciB0aGUKICAgICMgZmF0YWwgYW5jaG9yIG5vZGUgQU5EIGZvciBvYmxpZ2F0aW9uLmRlY2xhcmVkIGFuZCBvYmxpZ2F0aW9uLnByb2R1Y2VkIChjb3JyZWN0aW9uIGVkZ2UgdGFyZ2V0cykuCiAgICBhdCA9IGQuZ2V0KCJhdCIpIG9yIHt9CiAgICBuID0gYXQuZ2V0KCJhbmNob3IiKSBpZiBpc2luc3RhbmNlKGF0LmdldCgiYW5jaG9yIiksIGRpY3QpIGVsc2Uge30KICAgIG91dCA9IFsiYW5jaG9yX29jYz0iICsgb2NjX3ZlcmJhdGltKG4uZ2V0KCJvY2N1cnJlbmNlX2lkZW50aXR5IikpICsgIiBhbmNob3JfbmFtZT0iICsgc3RyKG4uZ2V0KCJuYW1lIikpXQogICAgYyA9IGQuZ2V0KCJjb3JyZWN0aW9uIikgb3Ige30KICAgIGFuID0gYy5nZXQoIm5vZGUiKSBvciB7fQogICAgZm9yIGUgaW4gKGFuLmdldCgiY2hpbGRyZW4iKSBvciBbXSk6CiAgICAgICAgaWYgbm90IGlzaW5zdGFuY2UoZSwgZGljdCk6CiAgICAgICAgICAgIGNvbnRpbnVlCiAgICAgICAgbGFiZWwgPSBlLmdldCgibGFiZWwiKSBvciB7fQogICAgICAgIG5hbWUgPSBsYWJlbC5nZXQoIm5hbWUiKSBpZiBpc2luc3RhbmNlKGxhYmVsLCBkaWN0KSBlbHNlIE5vbmUKICAgICAgICB0ID0gZS5nZXQoInRhcmdldCIpIG9yIHt9CiAgICAgICAgaWYgaXNpbnN0YW5jZSh0LCBkaWN0KSBhbmQgc3RyKG5hbWUpIGluICgicHJvZHVjZWRfZGVjbGFyZWRfY2FycmllciIsICJwb3NpdGlvbiIpOgogICAgICAgICAgICBwYXNzCiAgICAgICAgaWYgaXNpbnN0YW5jZSh0LCBkaWN0KToKICAgICAgICAgICAgb3V0LmFwcGVuZChzdHIobmFtZSkgKyAifHRhcmdldF9vY2M9IiArIG9jY192ZXJiYXRpbSh0LmdldCgib2NjdXJyZW5jZV9pZGVudGl0eSIpKSkKICAgIHJldHVybiAiIHx8ICIuam9pbihvdXQpCgpkZWYgY29ycmVjdGlvbl9ub2RlX2VkZ2VzKGQpOgogICAgIyBzZWF0IDE2OiB0aGUgZW1pdC1zaXRlIGRpc2NyaW1pbmF0b3IgcmlkZXMgdGhlIGNvcnJlY3Rpb24ncyBhdXRob3JlZC1ub2RlIGVkZ2VzCiAgICAjIChpbmhhYml0YW5jZS5kYWc6Nzk3LTgzMTogQ29ycmVjdGlvbjo6U3VnZ2VzdGVke25vZGV9IGNoaWxkcmVuID0KICAgICMgIEVkZ2V7bGFiZWw6e192YXJpYW50OkF1dGhvcmVkLG5hbWV9LHRhcmdldDpOb2RlfSB4IFtwb3NpdGlvbiwgcGFyYW1ldGVyX2lkZW50aXR5LT5kZWNsYXJlZCwKICAgICMgIF5pbmhhYml0YW5jZV9wcm9kdWNlZF90eXBlLT5wcm9kdWNlZCwgZmluZF93aXRuZXNzX3JlYXNvbi0+cHJvZHVjZWRdICsrIHBvc2l0aW9uIGNvbnRleHQgZWRnZXMpCiAgICBjID0gZC5nZXQoImNvcnJlY3Rpb24iKSBvciB7fQogICAgaWYgYy5nZXQoIl92YXJpYW50IikgIT0gIlN1Z2dlc3RlZCI6CiAgICAgICAgcmV0dXJuICJjb3JyPSIgKyBicmllZihjLCAxMjApCiAgICBhbiA9IGMuZ2V0KCJub2RlIikgb3Ige30KICAgIHBhcnRzID0gW10KICAgIGZvciBlIGluIChhbi5nZXQoImNoaWxkcmVuIikgb3IgW10pOgogICAgICAgIGlmIG5vdCBpc2luc3RhbmNlKGUsIGRpY3QpOgogICAgICAgICAgICBjb250aW51ZQogICAgICAgIGxhYmVsID0gZS5nZXQoImxhYmVsIikgb3Ige30KICAgICAgICBuYW1lID0gbGFiZWwuZ2V0KCJuYW1lIikgaWYgaXNpbnN0YW5jZShsYWJlbCwgZGljdCkgZWxzZSBOb25lCiAgICAgICAgdCA9IGUuZ2V0KCJ0YXJnZXQiKSBvciB7fQogICAgICAgIHRuID0gdC5nZXQoIm5hbWUiKSBpZiBpc2luc3RhbmNlKHQsIGRpY3QpIGVsc2UgTm9uZQogICAgICAgIHRrID0gdC5nZXQoImtpbmQiKSBpZiBpc2luc3RhbmNlKHQsIGRpY3QpIGVsc2UgTm9uZQogICAgICAgIHRvY2MgPSB0LmdldCgib2NjdXJyZW5jZV9pZGVudGl0eSIpIGlmIGlzaW5zdGFuY2UodCwgZGljdCkgZWxzZSBOb25lCiAgICAgICAgcGFydHMuYXBwZW5kKHN0cihuYW1lKSArICI9PnsiICsgc3RyKHRuKSArICIga2luZD0iICsgYnJpZWYodGssIDE0MCkgKyAiIG9jYz0iICsgYnJpZWYodG9jYywgNjApICsgIn0iKQogICAgcmV0dXJuICJjb3JyX2VkZ2VzWyIgKyBzdHIobGVuKHBhcnRzKSkgKyAiXSAiICsgYnJpZWYocGFydHMsIDE4MDApCgpwcmludCgiTE9DX1RPVEFMX0xJTkVTICIgKyBzdHIodG90YWwpICsgIiBSQVZFTl9GQVRBTF9QUkVTRU5UICIgKyBzdHIocHJlc2VudCkpCmZvciBqIGluIHJvd3M6CiAgICBkID0gai5nZXQoInJhdmVuX2ZhdGFsIikKICAgIGlmIG5vdCBkOgogICAgICAgIGNvbnRpbnVlCiAgICBjaCA9IGouZ2V0KCJyYXZlbl9jaGFpbiIpIG9yIHt9CiAgICBubGlua3MgPSAxICsgbGVuKGNoLmdldCgidGFpbCIpIG9yIFtdKSBpZiBjaC5nZXQoImhlYWQiKSBlbHNlIDAKICAgIHByaW50KCJMT0N8bW9kdWxlPSIgKyBzdHIoai5nZXQoIm1vZHVsZSIpKSArICJ8b3V0Y29tZT0iICsgc3RyKGouZ2V0KCJvdXRjb21lIikpICsKICAgICAgICAgICJ8bGlua3M9IiArIHN0cihubGlua3MpICsgInxyZWFzb249IiArIHN0cihkLmdldCgicmVhc29uIikpICsgInxmYXRhbF9hdD0iICsgbm9kZV9zdW1tYXJ5KGQpKQpwcmludCgiLS0tIGZhdGFsIG9jY3VycmVuY2UgaWRlbnRpdGllcywgdmVyYmF0aW0gKG9jY3VycmVuY2VfaWQgZmlyc3QpIC0tLSIpCk9SREVSID0gWyJ2Mi5zdGQub2NjdXJyZW5jZV9pZCIsICJ2Mi5zdGQudmVyZGljdCIsICJ2Mi5jb21waWxlci5hcm1fcmVzb2x1dGlvbiIsICJ2Mi5jb21waWxlci5ib2R5X3Byb2R1Y2VyX2ZvcndhcmQiLAogICAgICAgICAiZ3VuYmMuc3RydWN0dXJhbF9yZWFsaXphdGlvbl9iaW5kaW5ncyIsICJ2Mi5zdGQuZm9sZF9hc3NlbWJseSIsICJ2Mi5zdGQuYWxnZWJyYV9zdHJ1Y3R1cmVfc2lnbmF0dXJlIiwKICAgICAgICAgImd1bmJjLm5vbl9mb2xkX3Jlc2lkdWUiLCAic3RkLmRlY2xfcmVmIiwgInN0ZC5uYXQiLCAic3RkLmtleWVkX3JvdyJdCmRlZiByYW5rKG0pOgogICAgcmV0dXJuIE9SREVSLmluZGV4KG0pIGlmIG0gaW4gT1JERVIgZWxzZSA5OQpmb3IgaiBpbiBzb3J0ZWQocm93cywga2V5PWxhbWJkYSB4OiByYW5rKHN0cih4LmdldCgibW9kdWxlIikpKSk6CiAgICBkID0gai5nZXQoInJhdmVuX2ZhdGFsIikKICAgIGlmIG5vdCBkOgogICAgICAgIGNvbnRpbnVlCiAgICBwcmludCgiT0NDfG1vZHVsZT0iICsgc3RyKGouZ2V0KCJtb2R1bGUiKSkgKyAifHJlYXNvbj0iICsgc3RyKGQuZ2V0KCJyZWFzb24iKSkgKyAifCIgKyBmYXRhbF9vY2N1cnJlbmNlcyhkKSkKcHJpbnQoIi0tLSBwZXItbGluayByZWFzb25zICsgYW5jaG9ycyAoYWxsIG1vZHVsZXMsIHNob3J0KSAtLS0iKQpmb3IgaiBpbiByb3dzOgogICAgY2ggPSBqLmdldCgicmF2ZW5fY2hhaW4iKQogICAgaWYgbm90IGNoIG9yIG5vdCBjaC5nZXQoImhlYWQiKToKICAgICAgICBjb250aW51ZQogICAgbGlua3MgPSBbY2guZ2V0KCJoZWFkIildICsgKGNoLmdldCgidGFpbCIpIG9yIFtdKQogICAgZm9yIGksIGxpbmsgaW4gZW51bWVyYXRlKGxpbmtzKToKICAgICAgICBpZiBub3QgaXNpbnN0YW5jZShsaW5rLCBkaWN0KToKICAgICAgICAgICAgY29udGludWUKICAgICAgICBwcmludCgiTElOS3xtb2R1bGU9IiArIHN0cihqLmdldCgibW9kdWxlIikpICsgInxpPSIgKyBzdHIoaSkgKwogICAgICAgICAgICAgICJ8cmVhc29uPSIgKyBzdHIobGluay5nZXQoInJlYXNvbiIpKSArICJ8YXQ9IiArIG5vZGVfc3VtbWFyeShsaW5rKSArCiAgICAgICAgICAgICAgInwiICsgY29ycmVjdGlvbl9ub2RlX2VkZ2VzKGxpbmspKQpwcmludCgiLS0tIGxvY2kgcm93cyAocmVmdXNlZCBtb2R1bGVzIG9ubHkpIC0tLSIpCmZvciBqIGluIHJvd3M6CiAgICBsciA9IGouZ2V0KCJyYXZlbl9sb2NpX3Jvd3MiKSBvciBbXQogICAgaWYgbHI6CiAgICAgICAgcHJpbnQoIkxPQ0l8bW9kdWxlPSIgKyBzdHIoai5nZXQoIm1vZHVsZSIpKSArICJ8bj0iICsgc3RyKGxlbihscikpICsgInwiICsgYnJpZWYobHIsIDEyMDApKQpwcmludCgiLS0tIGZ1bGwgZmF0YWwgSlNPTiwgc3ViamVjdCBtb2R1bGVzIC0tLSIpCmZvciBqIGluIHJvd3M6CiAgICBkID0gai5nZXQoInJhdmVuX2ZhdGFsIikKICAgIGlmIG5vdCBkOgogICAgICAgIGNvbnRpbnVlCiAgICBtb2QgPSBzdHIoai5nZXQoIm1vZHVsZSIpKQogICAgaWYgbW9kIGluIHN1Ymo6CiAgICAgICAgcHJpbnQoIkZVTEx8bW9kdWxlPSIgKyBtb2QgKyAifG91dGNvbWU9IiArIHN0cihqLmdldCgib3V0Y29tZSIpKSArICJ8IiArIGJyaWVmKGQsIDQwMDApKQpwcmludCgiLS0tIGZ1bGwgY2hhaW4gSlNPTiwgc3ViamVjdCBtb2R1bGVzIC0tLSIpCmZvciBqIGluIHJvd3M6CiAgICBjaCA9IGouZ2V0KCJyYXZlbl9jaGFpbiIpCiAgICBpZiBub3QgY2ggb3Igbm90IGNoLmdldCgiaGVhZCIpOgogICAgICAgIGNvbnRpbnVlCiAgICBtb2QgPSBzdHIoai5nZXQoIm1vZHVsZSIpKQogICAgaWYgbW9kIGluIHN1Ymo6CiAgICAgICAgcHJpbnQoIkNIQUlOfG1vZHVsZT0iICsgbW9kICsgInwiICsgYnJpZWYoY2gsIDYwMDApKQpwcmludCgiLS0tIG91dGNvbWUgdGFsbHkgLS0tIikKZnJvbSBjb2xsZWN0aW9ucyBpbXBvcnQgQ291bnRlcgpjID0gQ291bnRlcigoc3RyKGouZ2V0KCJtb2R1bGUiKSkgaWYgc3RyKGouZ2V0KCJtb2R1bGUiKSkgaW4gc3ViaiBlbHNlICIob3RoZXIpIikgKyAiLyIgKyBzdHIoai5nZXQoIm91dGNvbWUiKSkgZm9yIGogaW4gcm93cykKZm9yIGsgaW4gc29ydGVkKGMpOgogICAgcHJpbnQoIlRBTExZICIgKyBrICsgIiAiICsgc3RyKGNba10pKQo=
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
