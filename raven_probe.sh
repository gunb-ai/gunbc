#!/usr/bin/env bash
# raven_probe.sh <tag> -- run the PRINT-ONLY probe on the scratch branch agent/v1-param-binder-probe.
#
# WHY THIS SHAPE, measured in the first probe attempt (run p2a):
#  - The probe mints DO print. The round's own refusal stderr carried them verbatim:
#      dag/std/algebra.dag:1618:1: error: B|RAVENPASS|std.algebra|list_init|kind=fn|binders=T,
#      dag/std/types.dag:158:1: error: B|RAVENPASS|std.types|Map|kind=alias|binders=key,value,
#    so the carriers (InternalError / UnlistedImportUse) and the payload rendering are proven.
#  - But the round REFUSED at its seed build (the probe's blocking carrier makes that build fail) and
#    a refusal RESTORES THE CHECKPOINT: p2a logged MIRROR_DRIFT_FILES=0 and regen_mirror_probe_rows=0,
#    so build2 rebuilt the UNPROBED mirror and the census saw nothing. The null was instrument
#    plumbing, not an absence of wall visits.
#  - The fix is r3's harvest: the round writes target/stage0-regen-candidate/src BEFORE it rolls back,
#    and that candidate tree is the round's own bytes. Install it, then build from it.
#  - No hand-Rust arms are needed here: the probe mints only variants that already exist and are
#    already rostered (InternalError, UnlistedImportUse), and the hand side is copied from the working
#    tree by the round, not generated.
#  - The probe is GATED to three modules (raven_probe_here): the two the fired corpus rows live in and
#    the one the nine-shape matrix is mounted in. Ungated it minted for 9494 distinct owners.
#
# WHAT DECIDES: for each wall visit, is the argument node the wall holds carrying the
# decl_type_parameter_ref stamp that binder_marked_type writes (decl=mod::Decl::<P>)? Read against the
# PASS row's roster for the same (module,item):
#   roster lacks the name                     -> the roster never named it (roster defect)
#   roster has it, wall shows decl=none       -> the pass never reached that position (coverage defect)
#   roster has it, wall shows a stamp         -> the wall judged a different node
# And the manager's discriminator: the 20 rows that DO fire (realization_measurement, applying
# Measure, parameter S) versus the 9 matrix shapes that do not -- visiting item's FORM, and whether
# that item's own PASS row exists.
set -u
TAG="${1:-noname}"
cd "$(dirname "$0")"
CAND=target/stage0-regen-candidate/src
GEN="v1_std_core.rs v1_compiler_infer_resolve.rs std_decl_ref.rs"
MT_KB=$(awk '/MemTotal/{print $2}' /proc/meminfo)
echo "tag=$TAG MemTotal_MB=$((MT_KB/1024))"
echo "RUNNER_HEAD=$(git rev-parse HEAD)"
echo "RUNNER_BRANCH=$(git rev-parse --abbrev-ref HEAD)"
test "$MT_KB" -ge 41943040 || { echo ABORT_MEMORY; exit 42; }
ROOT=$(awk -F: '$2=="memory"{print $3}' /proc/self/cgroup)
case "$ROOT" in /) CGD=/sys/fs/cgroup;; *) CGD=/sys/fs/cgroup${ROOT};; esac
# The binding is REQUIRED, not advisory, and it must be VERIFIED before anything expensive runs.
# Measured twice: p2f and p2g got "Permission denied" writing memory.max, the script printed a note and
# carried on UNBOUNDED, and 30 minutes later the round refused with HostBudgetUnreadable -- "no cgroup
# memory.high/memory.max or v1 hierarchical_memory_limit binds this process ... refusing rather than
# admit against the widest signal available" (dag/gunbc/host/host_budget_source.dag). That refusal is
# the compiler doing its job: an unknown budget cannot be defaulted, because the old default OOM-killed
# the process at rc=137. So a failed bind is fatal, and it must be caught at second 1, not minute 30.
# The '+memory' write to the parent's cgroup.subtree_control is what makes a child's memory.max
# writable at all; p2b..p2e happened to inherit a parent that already had it enabled, which is why the
# same block succeeded there and failed on two later runners.
test -w "$CGD/cgroup.subtree_control" && printf '+memory\n' > "$CGD/cgroup.subtree_control" 2>/dev/null || true
CG="$CGD/raven_probe_${TAG}"; mkdir -p "$CG" 2>/dev/null || { echo ABORT_CGROUP; exit 42; }
printf '45097156608\n' > "$CG/memory.max" 2>/dev/null || echo cgroup_max_not_set
printf '%s\n' $ > "$CG/cgroup.procs" 2>/dev/null || echo cgroup_procs_not_set
BOUND=$(cat "$CG/memory.max" 2>/dev/null || echo unreadable)
echo "CGROUP_BOUND path=$CG memory.max=$BOUND"
case "$BOUND" in
  45097156608) : ;;
  *) echo "ABORT_CGROUP_UNBOUND memory.max=$BOUND -- the round will refuse with HostBudgetUnreadable"; exit 42 ;;
esac
export GITHUB_ACTIONS=true GITHUB_SHA=$(git rev-parse HEAD)
unset GUNBC_MEMORY_BUDGET_BYTES GUNBC_MEMORY_ENVELOPE_REQUIRED GUNBC_BIND_MEMORY_CGROUP_BYTES
export TMPDIR=/tmp; PATH=$PATH:/usr/local/cargo/bin

echo "=== the probe is in the authority, and gated ==="
grep -c "RAVENWALL" src/v1/04_resolve.dag | sed 's/^/authority_wall_mints=/'
grep -c "RAVENPASS" src/v1/04_resolve.dag | sed 's/^/authority_pass_mints=/'
grep -c "fn raven_probe_here" src/v1/04_resolve.dag | sed 's/^/authority_gate_def=/'
grep -c "raven_probe_here(module_name: module_name)" src/v1/04_resolve.dag | sed 's/^/authority_gate_uses_must_be_2=/'
grep -c "RAVENWALL\|RAVENPASS" src/v1/stage0/src/v1_compiler_infer_resolve.rs | sed 's/^/committed_mirror_probe_must_be_0=/'

echo "=== step 1: build the round host from the committed (unprobed) mirror ==="
cargo build --release -p v1-compiler --bins > /tmp/b1.log 2>&1; echo "build1_rc=$?"
if grep -qaE "^error(\[|:)" /tmp/b1.log; then echo ABORT_BUILD1; grep -aE -A 14 "^error(\[|:)" /tmp/b1.log | head -90 | fold -w 195; exit 46; fi
test -x ./target/release/claim_executor || { echo ABORT_NO_EXEC; exit 46; }

echo "=== step 2: the round regenerates the mirror from the probed authority ==="
# A refusal at the seed build is EXPECTED and is in fact the probe printing: the blocking carrier
# makes that build fail, which restores the checkpoint. The candidate tree written at
# regen.mirror_write is what step 3 installs.
rm -rf target/stage0-regen-candidate
timeout 2100 ./target/release/claim_executor --regen-round-cost --source-root dag --source-root src/v2 > /tmp/round.out 2>/tmp/round.err
echo "round_rc=$?"
echo "MIRROR_WRITE_MARKS=$(grep -ac 'regen.mirror_write' /tmp/round.err)"
echo "PROBE_ROWS_IN_ROUND_STDERR=$(grep -ac 'RAVENPASS\|RAVENWALL' /tmp/round.err)"
echo "ALGEBRA_FIELD_REFUSALS=$(grep -ac "no field 'key' on type" /tmp/round.err)"
echo "--- probe lines the round itself printed (proof the carriers render) ---"
grep -ao "B|RAVEN[A-Z]*|[^ ]*" /tmp/round.err | head -12 | fold -w 195
echo "--- round.out tail 8 ---"; tail -8 /tmp/round.out | fold -w 195
# p2f aborted at step 3 with MIRROR_WRITE_MARKS=0 and no candidate tree, and could not say why, because
# this step printed counters but not the round's own stderr. A run that fails for want of a diagnostic
# is a wasted 35 minutes; print the tail and any compile-looking error lines.
echo "--- round.err tail 40 ---"; tail -40 /tmp/round.err | fold -w 195
echo "--- round.err compile-shaped lines ---"; grep -aE "error\[|error:|panicked|Killed|OOM|No such file" /tmp/round.err | head -25 | fold -w 195
echo "--- cgroup state (a memory.max that failed to set leaves the round unbounded and killable) ---"
cat "$CG/memory.max" 2>/dev/null | sed 's/^/CGROUP_MEMORY_MAX=/' || echo CGROUP_MEMORY_MAX=unreadable

echo "=== step 3: assert on, then INSTALL, the candidate tree ==="
CAND_FILES=$(cd "$CAND" && ls *.rs 2>/dev/null | tr '\n' ' ')
test -n "$CAND_FILES" || { echo ABORT_NO_CANDIDATE_TREE; exit 48; }
for f in $CAND_FILES; do test -f "$CAND/$f" || { echo "ABORT_NO_CANDIDATE_FILE $f"; exit 48; }; echo "CAND_BYTES $f=$(wc -c < "$CAND/$f")"; done
grep -c "RAVENWALL\|RAVENPASS" "$CAND/v1_compiler_infer_resolve.rs" | sed 's/^/candidate_resolve_probe_rows_must_be_nonzero=/'
# Each probe event lives in a specific shard; assert the carrier is in the bytes before installing them.
# p2e's RAVENMOD=0 was untrustworthy precisely because nothing checked this. The list adapts to what the
# AUTHORITY actually carries, so the same script can run the control at main, where only the module row
# exists (no wall/pass probe in 04_resolve.dag) and an assertion for a deliberately absent carrier would
# abort a run that is measuring exactly what it should.
CARRIERS=""
grep -q "RAVENWALL" src/v1/04_resolve.dag && CARRIERS="$CARRIERS v1_compiler_infer_resolve.rs:RAVENWALL"
grep -q "RAVENPASS" src/v1/04_resolve.dag && CARRIERS="$CARRIERS v1_compiler_infer_resolve.rs:RAVENPASS"
grep -q "RAVENMOD" src/v1/04_infer.dag && CARRIERS="$CARRIERS v1_compiler_infer.rs:RAVENMOD"
echo "CARRIER_LIST$CARRIERS"
test -n "$CARRIERS" || { echo ABORT_NO_CARRIERS_IN_AUTHORITY; exit 42; }
for pair in $CARRIERS; do
  f="${pair%%:*}"; tok="${pair##*:}"
  n=$(grep -ac "$tok" "$CAND/$f" 2>/dev/null || echo MISSING)
  echo "CANDIDATE_HAS $tok in $f = $n"
  test "$n" -ge 1 2>/dev/null || { echo "ABORT_CARRIER_ABSENT $tok $f"; exit 48; }
done
grep -c "raven_probe_here" "$CAND/v1_compiler_infer_resolve.rs" | sed 's/^/candidate_gate_calls_must_be_2=/'
# Install EVERY file the round produced, not a hard-coded list. p2e measured why: the list named three
# shards (v1_std_core, v1_compiler_infer_resolve, std_decl_ref) which is right for a 00_core/04_resolve
# edit, but this branch also probes 04_infer.dag, whose mirror is v1_compiler_infer.rs -- it was never
# installed (MIRROR_DRIFT_FILES=1), so RAVENMOD printed nothing and that nothing was an artifact of the
# script, not a measurement. A probe whose carrier is silently absent reads exactly like a clean zero.
for f in $CAND_FILES; do cp "$CAND/$f" "src/v1/stage0/src/$f"; done
echo "CANDIDATE_FILES_INSTALLED=$(echo $CAND_FILES | wc -w)"
echo "MIRROR_DRIFT_FILES=$(git diff --name-only -- src/v1/stage0/src/ | wc -l)"
git diff --name-only -- src/v1/stage0/src/ | sed 's/^/DRIFT /'

echo "=== step 4: build the seed from the probed mirror ==="
cargo build --release -p v1-compiler --bins > /tmp/b2.log 2>&1; echo "build2_rc=$?"
if grep -qaE "^error(\[|:)" /tmp/b2.log; then echo ABORT_BUILD2; grep -aE -A 14 "^error(\[|:)" /tmp/b2.log | head -90 | fold -w 195; exit 49; fi
test -x ./target/release/gunbc || { echo ABORT_NO_GUNBC; exit 49; }

echo "=== step 5: mount the kinded rung and the nine-shape matrix ==="
base64 -d > /tmp/kv90.patch <<'B64K'
ZGlmZiAtLWdpdCBhL2RhZy9zdGQvbWVhc3VyZS5kYWcgYi9kYWcvc3RkL21lYXN1cmUuZGFnCmluZGV4IDNlYzIyYzNiMTZmLi41MzlhY2YxODgxMyAxMDA2NDQKLS0tIGEvZGFnL3N0ZC9tZWFzdXJlLmRhZworKysgYi9kYWcvc3RkL21lYXN1cmUuZGFnCkBAIC0yMzQsNDIgKzIzNCw0MiBAQCBmbiBtZW1vcnlfc2NhbGVfZmFjdG9yX2J5dGVzKHM6IFNjYWxlKSAtPiBOYXQ/IHsKICAgfQogfQogCi10eXBlIE1lYXN1cmU8USwgUywgTT4gewordHlwZSBNZWFzdXJlPFE6IFF1YW50aXR5LCBTOiBTY2FsZSwgTT4gewogICBjb3VudDogTQogfQogCi1mbiBtZWFzdXJlX2NvdW50PFEsIFMsIE0+KG06IE1lYXN1cmU8USwgUywgTT4pIC0+IE0geworZm4gbWVhc3VyZV9jb3VudDxROiBRdWFudGl0eSwgUzogU2NhbGUsIE0+KG06IE1lYXN1cmU8USwgUywgTT4pIC0+IE0gewogICBtLmNvdW50CiB9CiAKLWZuIG1lYXN1cmVfc2NhbGVfZnJhY3Rpb25fZmxvb3I8USwgUz4obTogTWVhc3VyZTxRLCBTLCBOYXQ+LCBudW06IE5hdCwgZGVuOiBOYXQpIC0+IE1lYXN1cmU8USwgUywgTmF0PiB7CitmbiBtZWFzdXJlX3NjYWxlX2ZyYWN0aW9uX2Zsb29yPFE6IFF1YW50aXR5LCBTOiBTY2FsZT4obTogTWVhc3VyZTxRLCBTLCBOYXQ+LCBudW06IE5hdCwgZGVuOiBOYXQpIC0+IE1lYXN1cmU8USwgUywgTmF0PiB7CiAKICAgTWVhc3VyZSB7IGNvdW50OiBpZiBkZW4gPT0gMCB7IGRlbiB9IGVsc2UgeyAobS5jb3VudCAqIG51bSkgLyBkZW4gfSB9CiB9CiAKLWZuIG1lYXN1cmVfZml0X2NvdW50X2Zsb29yPFEsIFM+KGNhcGFjaXR5OiBNZWFzdXJlPFEsIFMsIE5hdD4sIGVhY2g6IE1lYXN1cmU8USwgUywgTmF0PikgLT4gTmF0IHsKK2ZuIG1lYXN1cmVfZml0X2NvdW50X2Zsb29yPFE6IFF1YW50aXR5LCBTOiBTY2FsZT4oY2FwYWNpdHk6IE1lYXN1cmU8USwgUywgTmF0PiwgZWFjaDogTWVhc3VyZTxRLCBTLCBOYXQ+KSAtPiBOYXQgewogCiAgIGxldCBlYWNoX2NvdW50ID0gZWFjaC5jb3VudAogICBpZiBlYWNoX2NvdW50ID09IDAgeyBlYWNoX2NvdW50IH0gZWxzZSB7IGNhcGFjaXR5LmNvdW50IC8gZWFjaF9jb3VudCB9CiB9CiAKLWZuIG1lYXN1cmVfc2NhbGVfZnJhY3Rpb25fY2VpbDxRLCBTPihtOiBNZWFzdXJlPFEsIFMsIE5hdD4sIG51bTogTmF0LCBkZW46IE5hdCkgLT4gTWVhc3VyZTxRLCBTLCBOYXQ+IHsKK2ZuIG1lYXN1cmVfc2NhbGVfZnJhY3Rpb25fY2VpbDxROiBRdWFudGl0eSwgUzogU2NhbGU+KG06IE1lYXN1cmU8USwgUywgTmF0PiwgbnVtOiBOYXQsIGRlbjogTmF0KSAtPiBNZWFzdXJlPFEsIFMsIE5hdD4gewogCiAgIE1lYXN1cmUgeyBjb3VudDogaWYgZGVuID09IDAgeyBtLmNvdW50ICogbnVtIH0gZWxzZSB7ICgobS5jb3VudCAqIG51bSkgKyAoZGVuIC0gMSkpIC8gZGVuIH0gfQogfQogCi1mbiBtZWFzdXJlX2FkZDxRLCBTPihhOiBNZWFzdXJlPFEsIFMsIE5hdD4sIGI6IE1lYXN1cmU8USwgUywgTmF0PikgLT4gTWVhc3VyZTxRLCBTLCBOYXQ+IHsKK2ZuIG1lYXN1cmVfYWRkPFE6IFF1YW50aXR5LCBTOiBTY2FsZT4oYTogTWVhc3VyZTxRLCBTLCBOYXQ+LCBiOiBNZWFzdXJlPFEsIFMsIE5hdD4pIC0+IE1lYXN1cmU8USwgUywgTmF0PiB7CiAgIE1lYXN1cmUgeyBjb3VudDogYS5jb3VudCArIGIuY291bnQgfQogfQogCiAvLyBUcnVuY2F0ZWQgc3VidHJhY3Rpb24gaXMgbm90IHRoZSBkZWZhdWx0LiBOYXR1cmFscyBhcmUgbm90IGNsb3NlZCB1bmRlciBzdWJ0cmFjdGlvbjsgdGhlCiAvLyBQZWFubyBmb3JtIGlzIHYyLnN0ZC5uYXQgbmF0X3N1YiAoTmF0RGlmZmVyZW5jZSB8IE5hdFN1YnRyYWhlbmRFeGNlZWRzTWludWVuZCkuIEEgZmxvb3IgYXQKIC8vIHplcm8gd291bGQgZmFicmljYXRlIGEgZGlmZmVyZW5jZSBvZiBlcXVhbCBtZWFzdXJlcy4KLXR5cGUgTWVhc3VyZVN1YnRyYWN0aW9uPFEsIFM+Cit0eXBlIE1lYXN1cmVTdWJ0cmFjdGlvbjxROiBRdWFudGl0eSwgUzogU2NhbGU+CiAgID0gTWVhc3VyZURpZmZlcmVuY2UgeyB2YWx1ZTogTWVhc3VyZTxRLCBTLCBOYXQ+IH0KICAgfCBNZWFzdXJlU3VidHJhaGVuZEV4Y2VlZHNNaW51ZW5kCiAKLWZuIG1lYXN1cmVfc3ViPFEsIFM+KGE6IE1lYXN1cmU8USwgUywgTmF0PiwgYjogTWVhc3VyZTxRLCBTLCBOYXQ+KSAtPiBNZWFzdXJlU3VidHJhY3Rpb248USwgUz4geworZm4gbWVhc3VyZV9zdWI8UTogUXVhbnRpdHksIFM6IFNjYWxlPihhOiBNZWFzdXJlPFEsIFMsIE5hdD4sIGI6IE1lYXN1cmU8USwgUywgTmF0PikgLT4gTWVhc3VyZVN1YnRyYWN0aW9uPFEsIFM+IHsKICAgaWYgYS5jb3VudCA8IGIuY291bnQgewogICAgIE1lYXN1cmVTdWJ0cmFoZW5kRXhjZWVkc01pbnVlbmQKICAgfSBlbHNlIHsK
B64K
git apply /tmp/kv90.patch; echo "kv90_rc=$?"
base64 -d > /tmp/shapes.dag <<'B64S'
Ly8gLS0tLSByYXZlbl9wMiBTSEFQRSBNQVRSSVggKHNjcmF0Y2g7IHRoZSBzY3JpcHQgcmVtb3ZlcyBpdCkgLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tCi8vIEVBQ0ggU0hBUEUgTkFNRVMgSVRTIEhFQURFUiBQQVJBTUVURVIgRElGRkVSRU5UTFkuIFRoZSBtb2R1bGUgZW52aXJvbm1lbnQgYmluZHMgYSBwYXJhbWV0ZXIgTkFNRQovLyBvbmNlIHBlciBtb2R1bGUgKG1lYXN1cmVkOiBldmVyeSB3YWxsIGFyZ3VtZW50IGNhcnJpZWQgYXJnX2RlY2w9Tm9uZSBhbmQgdGhlIGxvb2t1cCB0aGF0IGZvdW5kCi8vIHNvbWV0aGluZyB3YXMgdGhlIG1vZHVsZS1sZXZlbCBvbmUpLCBzbyBuaW5lIHNoYXBlcyBzaGFyaW5nIHRoZSBuYW1lIFEgd291bGQgYmUgZGVjaWRlZCBieSBPTkUKLy8gYmluZGVyIC0tIHRoZSBtYXRyaXggd291bGQgbWVhc3VyZSB0aGF0IGJpbmRlciwgbm90IHRoZSBuaW5lIHNoYXBlcy4KdHlwZSBQcm9iZVdpZHRoUmVzb2x1dGlvbgogID0gUHJvYmVTdGF0aWNXaWR0aEluZGV4CiAgfCBQcm9iZVBvaW50ZXJXaWR0aAoKLy8gRGVjbGFyZXMgRVhBQ1RMWSBQcm9iZVdpZHRoUmVzb2x1dGlvbidzIGFybSBuYW1lcy4gQSBjaGVjayBjb21wYXJpbmcga2luZHMgYnkgc3BlbGxpbmcsIG9yIGJ5IHRoZQovLyBzZXQgb2YgaW5oYWJpdGFudCBuYW1lcywgd291bGQgYWRtaXQgYSBmb3J3YXJkIGludG8gdGhlIHdyb25nIHNsb3Q7IGlkZW50aXR5IG11c3QgcmVmdXNlIGl0Lgp0eXBlIFByb2JlTG9va2FsaWtlCiAgPSBQcm9iZVN0YXRpY1dpZHRoSW5kZXgKICB8IFByb2JlUG9pbnRlcldpZHRoCgp0eXBlIFByb2JlUGhhbnRvbQoKLy8gVEhFIEFQUExJRUQgR0VORVJJQyBJUyBBIFJFQ09SRCwgTk9UIEFOIEFMSUFTLiBUaGUgMjAgY29ycHVzIHJvd3MgdGhhdCBETyBjYXJyeSBteSBuYW1lZCBjYXVzZSBhbGwKLy8gYXBwbHkgc3RkLm1lYXN1cmUuTWVhc3VyZSwgd2hpY2ggaXMgYSBSRUNPUkQgKGRhZy9zdGQvbWVhc3VyZS5kYWc6MjM3IGB0eXBlIE1lYXN1cmU8USwgUywgTT4gewovLyBjb3VudDogTSB9YCkuIEEgcGFyYW1ldHJpYyBBTElBUyAoYHR5cGUgUHJvYmVXaWR0aDxiPiA9IFByb2JlUGhhbnRvbWApIHJlc29sdmVzIHRocm91Z2ggaXRzCi8vIHJpZ2h0LWhhbmQgc2lkZSAocmVzb2x2ZV9ub21pbmFsX2FsaWFzX3JocywgMDRfcmVzb2x2ZS5kYWc6NzQ4KSBhbmQgbmV2ZXIgcmVhY2hlcyB0aGUga2luZCB3YWxsIGF0Ci8vIGFsbCAtLSBtZWFzdXJlZCBpbiBwMmIsIHdoZXJlIHRoZSB0aHJlZSBmbiBzaGFwZXMgdXNpbmcgdGhlIGFsaWFzIGZvcm0gcHJvZHVjZWQgTk8gcm93cyB3aGF0c29ldmVyCi8vIHdoaWxlIHRoZSBjb3JwdXMgZm4gc2hhcGUgcHJvZHVjZWQgdHdlbnR5LiBTbyBhbiBhbGlhcyBoZXJlIHdvdWxkIGhhdmUgbWVhc3VyZWQgdGhlIGFsaWFzIHBhdGgsIG5vdAovLyB0aGUgd2FsbC4KdHlwZSBQcm9iZVdpZHRoPGJpdHM6IFByb2JlV2lkdGhSZXNvbHV0aW9uPiB7CiAgbWFya2VyOiBOYXQKfQoKLy8gLS0tIHRoZSBBRE1JVFRFRCBzaGFwZSwgaW4gZWFjaCBvZiB0aGUgdGhyZWUgZGVjbGFyYXRpb24gZmFtaWxpZXMgLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLQp0eXBlIFByb2JlQm94T2s8UW9rOiBQcm9iZVdpZHRoUmVzb2x1dGlvbj4gewogIHZhbHVlOiBQcm9iZVdpZHRoPFFvaz4KfQoKdHlwZSBQcm9iZVN1bU9rPFFzb2s6IFByb2JlV2lkdGhSZXNvbHV0aW9uPgogID0gUHJvYmVXcmFwT2sgeyB2YWx1ZTogUHJvYmVXaWR0aDxRc29rPiB9CiAgfCBQcm9iZVN1bU5vdGhpbmcKCmZuIHByb2JlX2ZuX29rPFFmb2s6IFByb2JlV2lkdGhSZXNvbHV0aW9uPih3OiBQcm9iZVdpZHRoPFFmb2s+KSAtPiBQcm9iZVdpZHRoPFFmb2s+IHsKICB3Cn0KCi8vIC0tLSB0aGUgV1JPTkctS0lORCBzaGFwZSwgaW4gZWFjaCBmYW1pbHkgKGxvb2thbGlrZSBhcm1zLCBzbyBvbmx5IGlkZW50aXR5IHJlZnVzZXMpIC0tLS0tLS0tLS0tLQp0eXBlIFByb2JlQm94QmFkPFFiYWQ6IFByb2JlTG9va2FsaWtlPiB7CiAgdmFsdWU6IFByb2JlV2lkdGg8UWJhZD4KfQoKdHlwZSBQcm9iZVN1bUJhZDxRc2JhZDogUHJvYmVMb29rYWxpa2U+CiAgPSBQcm9iZVdyYXBCYWQgeyB2YWx1ZTogUHJvYmVXaWR0aDxRc2JhZD4gfQogIHwgUHJvYmVTdW1Ob3RoaW5nMgoKZm4gcHJvYmVfZm5fYmFkPFFmYmFkOiBQcm9iZUxvb2thbGlrZT4odzogUHJvYmVXaWR0aDxRZmJhZD4pIC0+IFByb2JlV2lkdGg8UWZiYWQ+IHsKICB3Cn0KCi8vIC0tLSB0aGUgVU5LSU5ERUQgc2hhcGUsIGluIGVhY2ggZmFtaWx5IC0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLQp0eXBlIFByb2JlQm94VW5raW5kZWQ8UXVrPiB7CiAgdmFsdWU6IFByb2JlV2lkdGg8UXVrPgp9Cgp0eXBlIFByb2JlU3VtVW5raW5kZWQ8UXN1az4KICA9IFByb2JlV3JhcFVua2luZGVkIHsgdmFsdWU6IFByb2JlV2lkdGg8UXN1az4gfQogIHwgUHJvYmVTdW1Ob3RoaW5nMwoKZm4gcHJvYmVfZm5fdW5raW5kZWQ8UWZ1az4odzogUHJvYmVXaWR0aDxRZnVrPikgLT4gUHJvYmVXaWR0aDxRZnVrPiB7CiAgdwp9Cg==
B64S
cat /tmp/shapes.dag >> dag/std/measure.dag; echo "shapes_probe_decls=$(grep -c 'Probe' dag/std/measure.dag)"

echo "=== step 6: the census on the probed binary ==="
timeout 1500 ./target/release/gunbc test //gunbc/instruments:v2-native-census-scoped > /tmp/cen_k.out 2>/tmp/cen_k.err
echo "census_kind_rc=$?"
cat /tmp/cen_k.out /tmp/cen_k.err > /tmp/kin.txt
cp /tmp/kin.txt /tmp/unk.txt
wc -c /tmp/kin.txt | sed 's/^/census_output_bytes=/'

echo "=== BASELINE PERTURBATION CHECK (must stay 20 Unkinded + 4 Inhabitants) ==="
base64 -d > /tmp/analyze.py <<'B64P'
aW1wb3J0IHJlLCBzeXMsIG9zLCBjb2xsZWN0aW9ucwojIENBVVNFUyBhcmUga2V5ZWQgb24gdGhlIG1lc3NhZ2UgdGV4dCB0aGUgY2Vuc3VzIHByaW50cywgdGFrZW4gdmVyYmF0aW0gZnJvbSB0aGUgZGlhZ25vc3RpYyBtZXNzYWdlCiMgZm9sZCBpbiBzcmMvdjEvMDBfY29yZS5kYWc6IHRoZSBmaXZlIGZvcndhcmRlZCBjYXVzZXMgdGhpcyBQUiBhZGRzLCB0aGUgZGlmZmVyIGNhc2UsIGFuZCBtYWluJ3Mgb3duCiMgaW5oYWJpdGFudC1saXN0IHJlZnVzYWwuIE1hdGNoaW5nIGlzIGJ5IHN1YnN0cmluZyBvbiBwdXJwb3NlOiB0aGUgcm93IHRleHQgaXMgdGhlIG1lc3NhZ2UuCkNBVVNFUyA9IFsKICAoIlVuaWRlbnRpZmllZCIsICAgImNhcnJpZXMgbm8gdHlwZS1wYXJhbWV0ZXIgZGVjbGFyYXRpb24iKSwKICAoIk93bmVyVW5yZXNvbHZlZCIsImRpZCBub3QgcmVzb2x2ZSB0byBvbmUgZGVjbGFyaW5nIG1vZHVsZSIpLAogICgiTm90RGVjbGFyZWQiLCAgICAiaXMgbm90IGEgcGFyYW1ldGVyIG9mIHRoZSBkZWNsYXJhdGlvbiIpLAogICgiVW5raW5kZWQiLCAgICAgICAiZGVjbGFyZXMgbm8ga2luZCIpLAogICgiS2luZFVucmVzb2x2ZWQiLCAiZGlkIG5vdCByZXNvbHZlIHRvIGEgc2luZ2xlIGRlY2xhcmluZyBwb3NpdGlvbiIpLAogICgiS2luZERpZmZlcnMiLCAgICAicmVzb2x2ZXMgdG8gYSBkaWZmZXJlbnQgZGVjbGFyYXRpb24gdGhhbiB0aGUgc2xvdCIpLAogICgiSW5oYWJpdGFudHMiLCAgICAiaXMgbm90IG9uZSBvZiBpdHMgaW5oYWJpdGFudHMiKSwKXQpNQVJLID0gKCJkb2VzIG5vdCBpbmhhYml0IHRoZSBkZWNsYXJlZCBraW5kIiwgImZvcndhcmRlZCB0eXBlIGFyZ3VtZW50IikKQVBQTFkgPSByZS5jb21waWxlKHIiYXBwbHlpbmcgJyg/UDxhcHA+W14nXSspJyIpClBBUkFNID0gcmUuY29tcGlsZShyInBhcmFtZXRlciAnKD9QPHBhcj5bXiddKyknIikKREVDTCAgPSByZS5jb21waWxlKHIiaXMgZGVjbGFyZWQgJyg/UDxraW5kPlteJ10rKSciKQpTVVAgICA9IHJlLmNvbXBpbGUociIoPzphbmQgJ3xzdXBwbGllZCBpcyAnKSg/UDxzdXA+W14nXSspIikKU1BBTiAgPSByZS5jb21waWxlKHIiXCgoP1A8ZmlsZT5bXHcuLy1dK1wuZGFnKTooP1A8bG8+XGQrKSIpCmRlZiByb3dzKHBhdGgpOgogICAgb3V0ID0gW10KICAgIGZvciBsaW5lIGluIG9wZW4ocGF0aCwgZW5jb2Rpbmc9InV0Zi04IiwgZXJyb3JzPSJyZXBsYWNlIik6CiAgICAgICAgaWYgbm90IGFueShtIGluIGxpbmUgZm9yIG0gaW4gTUFSSyk6IGNvbnRpbnVlCiAgICAgICAgYXAgPSBBUFBMWS5zZWFyY2gobGluZSk7IHBhID0gUEFSQU0uc2VhcmNoKGxpbmUpOyBkZSA9IERFQ0wuc2VhcmNoKGxpbmUpCiAgICAgICAgc3UgPSBTVVAuc2VhcmNoKGxpbmUpOyBzcCA9IFNQQU4uc2VhcmNoKGxpbmUpCiAgICAgICAgY2F1c2UgPSBuZXh0KChjIGZvciBjLCBzaWcgaW4gQ0FVU0VTIGlmIHNpZyBpbiBsaW5lKSwgIk9USEVSIikKICAgICAgICBvdXQuYXBwZW5kKGRpY3QoY2F1c2U9Y2F1c2UsCiAgICAgICAgICAgICAgICAgICAgICAgIGFwcD1hcC5ncm91cCgiYXBwIikgaWYgYXAgZWxzZSAiPyIsIHBhcj1wYS5ncm91cCgicGFyIikgaWYgcGEgZWxzZSAiPyIsCiAgICAgICAgICAgICAgICAgICAgICAgIGtpbmQ9ZGUuZ3JvdXAoImtpbmQiKSBpZiBkZSBlbHNlICI/IiwKICAgICAgICAgICAgICAgICAgICAgICAgc3VwPXN1Lmdyb3VwKCJzdXAiKVs6NjBdIGlmIHN1IGVsc2UgIj8iLAogICAgICAgICAgICAgICAgICAgICAgICBmaWxlPXNwLmdyb3VwKCJmaWxlIikgaWYgc3AgZWxzZSAiPyIsIGxpbmU9c3AuZ3JvdXAoImxvIikgaWYgc3AgZWxzZSAiPyIpKQogICAgcmV0dXJuIG91dApkZWYgcmVwb3J0KHRhZywgcGF0aCk6CiAgICBpZiBub3Qgb3MucGF0aC5leGlzdHMocGF0aCk6CiAgICAgICAgcHJpbnQoZiI9PSB7dGFnfTogTUlTU0lOR19MT0cge3BhdGh9Iik7IHJldHVybgogICAgcnMgPSByb3dzKHBhdGgpCiAgICBwcmludChmIj09IHt0YWd9OiB0b3RhbF9yZWZ1c2FsX3Jvd3M9e2xlbihycyl9IikKICAgIGZvciBjLCBfIGluIENBVVNFUzoKICAgICAgICBwcmludChmIiAgIHt0YWd9X2NhdXNlW3tjfV09e3N1bSgxIGZvciByIGluIHJzIGlmIHJbJ2NhdXNlJ109PWMpfSIpCiAgICBwcmludChmIiAgIHt0YWd9X2NhdXNlW09USEVSXT17c3VtKDEgZm9yIHIgaW4gcnMgaWYgclsnY2F1c2UnXT09J09USEVSJyl9IikKICAgIGcgPSBjb2xsZWN0aW9ucy5Db3VudGVyKChyWyJmaWxlIl0sIHJbImFwcCJdLCByWyJwYXIiXSwgclsia2luZCJdLCByWyJjYXVzZSJdKSBmb3IgciBpbiBycykKICAgIHByaW50KGYiICAge3RhZ31fY2F1c2VfZ3JvdXBzPXtsZW4oZyl9IikKICAgIGZvciBrLCBuIGluIGcubW9zdF9jb21tb24oMjQpOgogICAgICAgIHByaW50KCIgICBHUk9VUCIsIG4sIGspCiAgICBwcmludChmIiAgIHt0YWd9X2J5X2ZpbGU6IikKICAgIGZvciBrLCBuIGluIGNvbGxlY3Rpb25zLkNvdW50ZXIoclsiZmlsZSJdIGZvciByIGluIHJzKS5tb3N0X2NvbW1vbigxNCk6CiAgICAgICAgcHJpbnQoIiAgICAgRklMRSIsIG4sIGspCiAgICB1ID0gY29sbGVjdGlvbnMuQ291bnRlcigoclsiZmlsZSJdLCByWyJhcHAiXSwgclsicGFyIl0sIHJbInN1cCJdKSBmb3IgciBpbiBycyBpZiByWyJjYXVzZSJdID09ICJVbmlkZW50aWZpZWQiKQogICAgcHJpbnQoZiIgICB7dGFnfV9hcm0xX3VuaWRlbnRpZmllZF9yb3dzPXtzdW0odS52YWx1ZXMoKSl9IGdyb3Vwcz17bGVuKHUpfSIpCiAgICBmb3IgaywgbiBpbiB1Lm1vc3RfY29tbW9uKDIwKToKICAgICAgICBwcmludCgiICAgICBBUk0xIiwgbiwgaykKICAgIHByaW50KCIgICBWRVJCQVRJTSBwZXIgY2F1c2U6IikKICAgIHNlZW4gPSBzZXQoKQogICAgZm9yIHIgaW4gcnM6CiAgICAgICAgaWYgclsiY2F1c2UiXSBpbiBzZWVuOiBjb250aW51ZQogICAgICAgIHNlZW4uYWRkKHJbImNhdXNlIl0pCiAgICAgICAgcHJpbnQoIiAgICAgIiwgclsiY2F1c2UiXSwgInwiLCByWyJmaWxlIl0sIHJbImxpbmUiXSwgInwiLCByWyJhcHAiXSwgclsicGFyIl0sIHJbImtpbmQiXSwgInwiLCByWyJzdXAiXVs6NzBdKQpyZXBvcnQoIlVOSyIsICIvdG1wL3Vuay50eHQiKQpyZXBvcnQoIktJTiIsICIvdG1wL2tpbi50eHQiKQo=
B64P
wc -c /tmp/analyze.py | sed 's/^/analyze_bytes=/'
python3 /tmp/analyze.py > /tmp/analysis.txt 2>&1; echo "analyze_rc=$?"
grep -a "KIN_cause\|KIN_total\|UNK_total\|VERBATIM" /tmp/analysis.txt | head -14 | cut -c1-200

echo "=== WHICH CARRIER PRINTED? ==="
echo "WALL_ROWS_ALL=$(grep -ac 'RAVENWALL' /tmp/kin.txt)"
echo "WALL_ROWS_BLOCKING=$(grep -ac 'B|RAVENWALL' /tmp/kin.txt)"
echo "WALL_ROWS_ADVISORY=$(grep -ac 'A|RAVENWALL' /tmp/kin.txt)"
echo "PASS_ROWS_ALL=$(grep -ac 'RAVENPASS' /tmp/kin.txt)"
echo "PASS_ROWS_BLOCKING=$(grep -ac 'B|RAVENPASS' /tmp/kin.txt)"
echo "PASS_ROWS_ADVISORY=$(grep -ac 'A|RAVENPASS' /tmp/kin.txt)"

echo "=== the census output's own row format (first 25 lines, unfiltered) ==="
head -25 /tmp/kin.txt | cut -c1-200
echo "=== THE 24: the census's OWN rows for dag/std/measure.dag, by class (manager msg_8be1fe9d) ==="
# The class key is not printed by any .dag-callable function: the census's single authority for it is the
# total match over v1.std.core CompilerDiagnostic in the HAND-RUST projection
# (cli_run::compile_clean_diagnostic_histogram_key), and the census already emits '<Class> | <path> <offset>
# | <subject> | <message>' rows. So the class list is READ from the instrument rather than re-derived in
# the probe -- re-deriving it here would be a second source for a key that has exactly one, and a second
# source that could drift from the first.
grep -a "dag/std/measure.dag" /tmp/kin.txt | grep -av "RAVEN" | head -60 | cut -c1-235
echo "--- tally by class over the whole closure, for reference ---"
grep -aoE "^\s+[A-Z][A-Za-z]+ \| " /tmp/kin.txt | sed 's/[ |]//g' | sort | uniq -c | sort -rn | head -20
echo "--- the measure.dag rows' own count, and how many are blocking ---"
grep -ac "dag/std/measure.dag" /tmp/kin.txt | sed 's/^/MEASURE_PATH_ROWS=/'
echo "=== DECISION TABLE 0: the module itself (was it abandoned before items?) ==="
grep -aoE "RAVENMOD\|[^ ]*" /tmp/kin.txt | sort | uniq -c | head -10 | fold -w 195
echo "=== DECISION TABLE 0a: EVERY wall entry in the gate (did the shape reach the wall at all?) ==="
grep -aoE "RAVENENTER\|[^ ]*" /tmp/kin.txt | sort | uniq -c | head -40 | fold -w 195
echo "=== DECISION TABLE 0b: parameters the wall DECLINED for lack of a kind property ==="
grep -aoE "RAVENSKIP\|[^ ]*" /tmp/kin.txt | sort | uniq -c | head -30 | fold -w 195
echo "=== carrier totals by verdict (the answer, in one place) ==="
grep -ao "verdict=[a-z_]*" /tmp/kin.txt | sort | uniq -c | sort -rn | head -12 | fold -w 195
echo "=== DECISION TABLE 1: the nine matrix shapes, at the wall ==="
grep -ao "RAVENWALL|std.measure|Probe[^ ]*" /tmp/kin.txt | sort | uniq -c | head -30 | fold -w 195
echo "=== DECISION TABLE 2: those shapes' PASS rows (what roster did their item's marking have?) ==="
grep -ao "RAVENPASS|std.measure|Probe[^ ]*" /tmp/kin.txt | sort | uniq -c | head -30 | fold -w 195
echo "=== DECISION TABLE 3: a FIRED corpus row's wall+pass pair (the 20) ==="
grep -ao "RAVENWALL|std.realization_measurement|[^ ]*" /tmp/kin.txt | sort | uniq -c | head -8 | fold -w 195
grep -ao "RAVENPASS|std.realization_measurement|[^ ]*" /tmp/kin.txt | sort | uniq -c | head -8 | fold -w 195
echo "=== DECISION TABLE 4: the 4 fall-through rows (realization_schedule) ==="
grep -ao "RAVENWALL|std.realization_schedule|[^ ]*" /tmp/kin.txt | sort | uniq -c | head -8 | fold -w 195
grep -ao "RAVENPASS|std.realization_schedule|[^ ]*" /tmp/kin.txt | sort | uniq -c | head -8 | fold -w 195
echo "=== DECISION TABLE 5: the 2 Measure refusals ==="
grep -ao "RAVENWALL|std.measure|Measure[^ ]*" /tmp/kin.txt | sort | uniq -c | head -8 | fold -w 195
grep -ao "RAVENPASS|std.measure|Measure[^ ]*" /tmp/kin.txt | sort | uniq -c | head -8 | fold -w 195
echo "=== how many wall rows carry a stamp at all, across the gate ==="
grep -ao "RAVENWALL|[^ ]*" /tmp/kin.txt | grep -ac "decl=[^n]" | sed 's/^/WALL_ROWS_with_stamp=/'
grep -ao "RAVENWALL|[^ ]*" /tmp/kin.txt | grep -ac "decl=none" | sed 's/^/WALL_ROWS_without_stamp=/'
echo "=== the full text of every matrix wall row (untruncated, first 24) ==="
grep -a "RAVENWALL|std.measure|Probe" /tmp/kin.txt | sed -E 's/.*(RAVENWALL\|.*)/\1/' | sort -u | head -24 | cut -c1-260

git checkout -- dag/std/measure.dag src/v1/stage0/src/ 2>/dev/null
echo "PROBE_DONE tag=$TAG"
