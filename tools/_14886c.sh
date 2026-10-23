#!/usr/bin/env bash
# #14886 C: RED arm full-line receipt (A truncated at 190c). Plant, run, print the UNBOUNDED line in FULL.
set -u
REPO=/root/workspace/repo-root; [ -d /tmp/c4v/disp2 ] && REPO=/tmp/c4v/disp2
cd $REPO || exit 9
sha() { sha256sum "$1" | cut -d' ' -f1; }
git worktree prune >/dev/null 2>&1 || true
git fetch -q origin +refs/pull/14886/head:refs/verifier/f14886c || { echo FETCH_FAIL; exit 9; }
FULL=$(git rev-parse refs/verifier/f14886c)
[ "${FULL:0:12}" = "a87a952f4492" ] || { echo "PR_REF_MOVED got=$FULL -- STOP"; exit 9; }
echo "HEAD_VERIFIED=$FULL"
export CARGO_INCREMENTAL=0 RUSTC_WRAPPER= CARGO_TARGET_DIR=/root/workspace/repo-root/.t886c
T=/tmp/w886c_h
git worktree prune >/dev/null 2>&1; rm -rf $T
git worktree add -q --force --detach $T "$FULL" || { echo WT_FAIL; exit 9; }
(cd $T && cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -1)
GUNBC=/root/workspace/repo-root/.t886c/release/gunbc
[ -x $GUNBC ] || { echo NO_BINARY; exit 9; }
echo "GUNBC_SHA=$(sha $GUNBC) (must equal A's c0a1468b... = same binary)"
ROSTER=dag/gunbc/witness/witness_deferral_freeze.dag
cp $T/$ROSTER /tmp/w886c_roster_orig
python3 - <<'PY'
import re
orig=open('/tmp/w886c_roster_orig').read()
m=re.search(r'^data frozen_path_deferrals: List<FrozenPathDeferral> = \[\n', orig, re.M)
assert m
end=orig.index('\n]', m.end())
ins='''  FrozenPathDeferral {
    entry: "dag/test/claim/artifact_store_fs_witness.dag",
    functions: [
      "verifier_planted_identity_886_holds"
    ]
  },
'''
out=orig[:end+1] + ins + orig[end+1:]
assert out.count('FrozenPathDeferral {')==orig.count('FrozenPathDeferral {')+1
open('/tmp/w886c_p_roster','w').write(out)
print("PLANT ok")
PY
cp /tmp/w886c_p_roster $T/$ROSTER || exit 9
(cd $T && env GUNBC_CI_DIFF_BASE=HEAD timeout 2400 $GUNBC test '//gunbc/instruments:witness-deferral-freeze-population' > /tmp/w886c_red.out 2>&1); rc=$?
echo "[inst red-full] rc=$rc BYTES=$(wc -c < /tmp/w886c_red.out)"
grep -a 'UNBOUNDED' /tmp/w886c_red.out | sed 's/^/    FULL| /'
grep -ac 'verifier_planted_identity_886_holds' /tmp/w886c_red.out | sed 's/^/NAMES_FN_OCCURRENCES=/'
grep -ac 'artifact_store_fs_witness' /tmp/w886c_red.out | sed 's/^/NAMES_ENTRY_OCCURRENCES=/'
echo DONE_14886C
