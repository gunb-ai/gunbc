set -u
cd /root/workspace/repo-root 2>/dev/null || true
# #14623 BASE-side dispatch: refuse arms + other-shape control.
BASESHA=ef346f6ac57e3a85d326d727ba67e927dba16736
F=dag/test/claim/react_jsx_emit.dag
echo "BASESHA=$BASESHA START $(date -u +%H:%M:%S)Z"
git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/w623b
git cat-file -e "$BASESHA^{commit}" || git fetch -q origin $BASESHA
git worktree add -q --detach /tmp/w623b $BASESHA || { echo WT_B_FAIL; exit 9; }
sha() { sha256sum "$1" | cut -d' ' -f1; }
export CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
( cd /tmp/w623b && CARGO_TARGET_DIR=/tmp/w623b/.t623b cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1 )
CBB=/tmp/w623b/.t623b/release/claim_batch; [ -x $CBB ] || { echo NO_CBB; exit 9; }
echo "CBB_SHA=$(sha $CBB | cut -c1-12)"
FNS_B=$(grep -aoE '^test fn [a-zA-Z0-9_]+' /tmp/w623b/$F | sed 's/^test fn //' | sort | paste -sd,)
echo "FNS_BASE=$FNS_B"
run3 () { timeout 1500 $3 --claim-run --source-root "$2" --source-root src/v2 --source-root src/v1 --entry "$1" --functions "$5" > "$4" 2>&1; echo $?; }
cd /tmp/w623b || exit 9
echo "=== B. BASE: same 4 names, expect resolve refusal (0/4), named cause ==="
rc=$(run3 $F dag $CBB /tmp/c623_base.out "$FNS_B"); echo "  rc=$rc PASS=$(grep -ac '^PASS ' /tmp/c623_base.out) FAIL=$(grep -ac '^FAIL ' /tmp/c623_base.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/c623_base.out)"
echo "  cause_hits=$(grep -ac "no declaration is indexed at 'gunbc.gunbhub_serve.gunbhub_browse_page'" /tmp/c623_base.out)"
grep -aiE 'error|refus|no declaration' /tmp/c623_base.out | head -8 | sed 's/^/    E| /' | cut -c1-190
[ "$rc" = "0" ] && echo "  !! BASE DID NOT REFUSE (rc=0)"
[ "$rc" != "0" ] && [ "$(grep -ac "no declaration is indexed at 'gunbc.gunbhub_serve.gunbhub_browse_page'" /tmp/c623_base.out)" = "0" ] && echo "  !! REFUSED WITH WRONG CAUSE"
echo "=== C. OTHER-SHAPE CONTROL at BASE: gunbhub modules healthy ==="
BN1=$(grep -aoE '^test fn [a-zA-Z0-9_]+' dag/test/claim/gunbhub_serve_witness.dag | sed 's/^test fn //' | head -1)
BN2=$(grep -aoE '^test fn [a-zA-Z0-9_]+' dag/test/claim/gunbhub_react_browse_witness.dag | sed 's/^test fn //' | head -1)
echo "  control fns: serve=$BN1 browse=$BN2"
rc=$(run3 dag/test/claim/gunbhub_serve_witness.dag dag $CBB /tmp/c623_ctl1.out "$BN1"); echo "  serve_witness rc=$rc PASS=$(grep -ac '^PASS ' /tmp/c623_ctl1.out) FAIL=$(grep -ac '^FAIL ' /tmp/c623_ctl1.out)"
grep -aE '^(PASS|FAIL) ' /tmp/c623_ctl1.out | head -2 | sed 's/^/    V| /' | cut -c1-150
rc=$(run3 dag/test/claim/gunbhub_react_browse_witness.dag dag $CBB /tmp/c623_ctl2.out "$BN2"); echo "  react_browse(0 imports) rc=$rc PASS=$(grep -ac '^PASS ' /tmp/c623_ctl2.out) FAIL=$(grep -ac '^FAIL ' /tmp/c623_ctl2.out)"
grep -aE '^(PASS|FAIL) ' /tmp/c623_ctl2.out | head -2 | sed 's/^/    V| /' | cut -c1-150
echo DONE_623_B
cd /root/workspace/repo-root
git worktree remove --force /tmp/w623b >/dev/null 2>&1 || true
git worktree prune >/dev/null 2>&1 || true
