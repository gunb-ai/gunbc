set -u
cd /root/workspace/repo-root 2>/dev/null || true
# #14613 RE-VERIFY at new printed head 1149df92c31 (delta vs my verified 34ad3659 = exactly 1
# DESIGN.md line: the regen commit). Manager scope: (1) DESIGN.md diff is exactly 1 line [static
# done locally: 1 insertion], (2) the regen is idempotent. Plus: witness 17/17 by name at the new
# head, and the documentary_refs verdict (standing finding) at the new head.
OID=1149df92c314f79937b8e8a8437b2db2ddb839da  # from gh pr view 14613 --json headRefOid
echo "HEADSHA_from_gh=$OID START $(date -u +%H:%M:%S)Z"
git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/w613h
git fetch -q origin +refs/pull/14613/head:refs/verifier/14613h 2>&1 | tail -1
[ "$(git rev-parse refs/verifier/14613h)" = "$OID" ] || { echo "PR_REF_MOVED=$(git rev-parse refs/verifier/14613h) -- STOP"; exit 9; }
git worktree add -q --detach /tmp/w613h $OID || { echo WT_FAILED; exit 9; }
WT=/tmp/w613h
cd $WT || exit 9
[ "$(git rev-parse HEAD)" = "$OID" ] || { echo BUILT_AT_WRONG_COMMIT; exit 9; }
sha() { sha256sum "$1" | cut -d' ' -f1; }
WIT=dag/test/claim/instruments/instrument_dispatch_workflow_witness.dag
GATE=dag/gunbc/instruments/generated_artifact_gate.dag
DOC=dag/test/claim/documentary_refs_witness.dag
export CARGO_TARGET_DIR=/root/workspace/repo-root/.t14613 CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
cargo build --release -p v1-compiler --bin claim_batch --bin gunbc 2>&1 | tail -1
CB=/root/workspace/repo-root/.t14613/release/claim_batch
GB=/root/workspace/repo-root/.t14613/release/gunbc
[ -x $CB ] && [ -x $GB ] || { echo NO_BINARIES; exit 9; }
echo "CB_SHA=$(sha $CB|cut -c1-12) GB_SHA=$(sha $GB|cut -c1-12) (prior head: 18213db69cd1 / d15780d5f66b; delta touched no .dag, so identical expected)"
echo "HEAD_SHA256_613H=$(sha $CB)|$(sha $GB)"
echo "=== A. witness at NEW head, names from the FILE ==="
FNS=$(grep -aoE '^test fn [a-zA-Z0-9_]+' $WIT | sed 's/^test fn //' | sort | paste -sd,)
echo "claim_count=$(echo $FNS | tr ',' '\n' | wc -l)"
timeout 1200 $CB --claim-run --source-root dag --source-root src/v2 --source-root src/v1 --entry $WIT --functions "$FNS" > /tmp/c613h_wit.out 2>&1; rc=$?
echo "  rc=$rc PASS=$(grep -ac '^PASS ' /tmp/c613h_wit.out) FAIL=$(grep -ac '^FAIL ' /tmp/c613h_wit.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/c613h_wit.out)"
grep -aE '^FAIL ' /tmp/c613h_wit.out | sed 's/^/    V| /' | cut -c1-170
[ "$rc" = "0" ] && [ "$(grep -ac '^PASS ' /tmp/c613h_wit.out)" = "0" ] && echo "  !! EXIT 0 HAVING OBSERVED NOTHING"
echo "=== B. REGEN IDEMPOTENCY: route over path=DESIGN.md at head, committed bytes must NOT move ==="
BEFORE=$(sha DESIGN.md); POR0=$(git status --porcelain DESIGN.md)
timeout 900 $GB run --source-root dag --source-root src/v2 --entry $GATE --function main_wet_one --arg path=DESIGN.md > /tmp/c613h_r0.out 2>&1; rc=$?
AFTER=$(sha DESIGN.md)
echo "  rc=$rc before=$BEFORE after=$AFTER equal=$([ "$BEFORE" = "$AFTER" ] && echo YES || echo NO)"
echo "  porcelain_before='$POR0' porcelain_after='$(git status --porcelain DESIGN.md)'"
grep -aE 'refus|Refus|Unrostered|unrostered|error' /tmp/c613h_r0.out | head -4 | sed 's/^/    R0| /' | cut -c1-190
echo "=== C. DRIFT-REPAIR CONTROL: delete the new line, route must write it back byte-exact ==="
tail -1 DESIGN.md > /tmp/c613h_lastline.txt; LINES0=$(wc -l < DESIGN.md)
head -n $((LINES0-1)) DESIGN.md > /tmp/c613h_trunc.txt && cp /tmp/c613h_trunc.txt DESIGN.md
echo "  seeded: lines $LINES0 -> $(wc -l < DESIGN.md), sha $(sha DESIGN.md) (was $BEFORE)"
timeout 900 $GB run --source-root dag --source-root src/v2 --entry $GATE --function main_wet_one --arg path=DESIGN.md > /tmp/c613h_r1.out 2>&1; rc=$?
echo "  rc=$rc repaired_sha=$(sha DESIGN.md) expect_head=$(git rev-parse $OID:DESIGN.md >/dev/null 2>&1; git cat-file blob $OID:DESIGN.md | sha256sum | cut -d' ' -f1)"
git cat-file blob $OID:DESIGN.md > /tmp/c613h_headblob.txt
cmp /tmp/c613h_headblob.txt DESIGN.md && echo "  REPAIR_BYTE_EXACT=YES" || echo "  REPAIR_BYTE_EXACT=NO"
echo "  new_line_present=$(grep -ac 'instrument_measurement_queues_run_in_parallel' DESIGN.md) (expect >=1)"
grep -aE 'refus|Refus|error' /tmp/c613h_r1.out | head -3 | sed 's/^/    R1| /' | cut -c1-190
echo "=== D. documentary_refs witness by name at NEW head (standing finding re-check) ==="
DFNS=$(grep -aoE '^test fn [a-zA-Z0-9_]+' $DOC | sed 's/^test fn //' | sort | paste -sd,)
echo "  doc claims from file: $DFNS"
timeout 900 $CB --claim-run --source-root dag --source-root src/v2 --source-root src/v1 --entry $DOC --functions "$DFNS" > /tmp/c613h_doc.out 2>&1; drc=$?
echo "  rc=$drc PASS=$(grep -ac '^PASS ' /tmp/c613h_doc.out) FAIL=$(grep -ac '^FAIL ' /tmp/c613h_doc.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/c613h_doc.out)"
grep -aE '^(PASS|FAIL|UNOBSERVED) ' /tmp/c613h_doc.out | sed 's/^/    V| /' | cut -c1-160
[ "$drc" = "0" ] && [ "$(grep -ac '^PASS ' /tmp/c613h_doc.out)" = "0" ] && echo "  !! EXIT 0 HAVING OBSERVED NOTHING"
echo "=== E. registry evidence for the correction (static, at head) ==="
grep -n "DesignArtifact" dag/gunbc/generated_artifact_registry.dag | head -2 | sed 's/^/    REG| /' | cut -c1-150
grep -c "^- \*\*" DESIGN.md | sed 's/^/    DESIGNMD_bullet_lines=/;s/^/    E| /'
echo DONE_613_H
cd /root/workspace/repo-root
git worktree remove --force $WT >/dev/null 2>&1 || true
git worktree prune >/dev/null 2>&1 || true
