#!/usr/bin/env bash
# #14776 regen re-probe (arm G redo, clean invocation): main_wet_one on the fold path
# at head d2fbb8a and at base 3da8211 (both expect GeneratedArtifactPathUnrostered rc!=0),
# plus DESIGN.md control run for gate health.
set -u
REPO=/root/workspace/repo-root
[ -d /tmp/c4v/pr14575 ] && REPO=/tmp/c4v/pr14575
cd $REPO || exit 9
echo "REPO=$REPO"
git remote set-url origin https://github.com/gunb-ai/gunbc 2>/dev/null || true
git fetch -q origin +refs/heads/session/nimble-badger-800:refs/heads/session/nimble-badger-800 2>/dev/null || true
git checkout -q -f -B session/nimble-badger-800 origin/session/nimble-badger-800 2>/dev/null || true
echo "CGROUP_CHECK:"; cat /sys/fs/cgroup/memory.max 2>/dev/null || echo "(no own memory.max)"
export CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
WFE=src/v2/workflow/wet_evidence.dag
GATE=dag/gunbc/instruments/generated_artifact_gate.dag
export CARGO_TARGET_DIR=/tmp/tf776r
GB=$CARGO_TARGET_DIR/release/gunbc
rm -rf /tmp/wf776c /tmp/wf776d
git fetch -q origin +refs/pull/14776/head:refs/verifier/f776c || { echo FETCH_FAIL; exit 9; }
[ "$(git rev-parse refs/verifier/f776c)" = "d2fbb8a3389ff6e46500902084172672590e99fe" ] || { echo "PR_REF_MOVED fetched=$(git rev-parse refs/verifier/f776c) -- STOP"; exit 9; }
git worktree add -q --detach /tmp/wf776c d2fbb8a3389ff6e46500902084172672590e99fe || { echo WT_FAIL; exit 9; }
cd /tmp/wf776c || exit 9
[ "$(git rev-parse HEAD)" = "d2fbb8a3389ff6e46500902084172672590e99fe" ] || { echo "WT_NOT_AT_HEAD"; exit 9; }
cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -1
[ -x $GB ] || { echo "NO GB -- nothing below is a result"; exit 9; }
echo "GB_SHA=$(sha256sum $GB | cut -d' ' -f1) BUILD_COMMIT=$(git rev-parse HEAD) built $(date -u +%H:%M:%S)Z"
echo "=== fold @ head: $(date -u +%H:%M:%S)Z"
timeout 600 $GB run --source-root dag --source-root src/v2 --entry $GATE --function main_wet_one --arg path=$WFE > /tmp/g776_fold_head.out 2>&1; echo "[regen-fold@head] rc=$?"
echo "  bytes=$(wc -c < /tmp/g776_fold_head.out)"; tail -5 /tmp/g776_fold_head.out | cut -c1-240 | sed 's/^/  | /'
echo "  fold file changed? $([ "$(sha256sum $WFE | cut -d' ' -f1)" = "$(git rev-parse HEAD:$WFE | head -c1 >/dev/null; git cat-file blob HEAD:$WFE | sha256sum | cut -d' ' -f1)" ] && echo NO || echo YES)"
echo "=== DESIGN.md control @ head (gate health): $(date -u +%H:%M:%S)Z"
cp -f DESIGN.md /tmp/g776_design_orig
timeout 600 $GB run --source-root dag --source-root src/v2 --entry $GATE --function main_wet_one --arg path=DESIGN.md > /tmp/g776_design.out 2>&1; echo "[regen-DESIGN@head] rc=$?"
echo "  bytes=$(wc -c < /tmp/g776_design.out)"; tail -4 /tmp/g776_design.out | cut -c1-240 | sed 's/^/  | /'
[ "$(sha256sum DESIGN.md | cut -d' ' -f1)" = "$(sha256sum /tmp/g776_design_orig | cut -d' ' -f1)" ] && echo "  DESIGN no-op write (byte-identical)" || { echo "  DESIGN CHANGED -- restoring"; cp -f /tmp/g776_design_orig DESIGN.md; }
echo "=== fold @ base 3da8211: $(date -u +%H:%M:%S)Z"
cd $REPO
git worktree add -q --detach /tmp/wf776d 3da8211eaf95fdde05fed47ec5d550f5f3d47a6c || { echo BASE_WT_FAIL; exit 9; }
cd /tmp/wf776d || exit 9
[ "$(git rev-parse HEAD)" = "3da8211eaf95fdde05fed47ec5d550f5f3d47a6c" ] || { echo BASE_NOT_AT_BASE; exit 9; }
timeout 600 $GB run --source-root dag --source-root src/v2 --entry $GATE --function main_wet_one --arg path=$WFE > /tmp/g776_fold_base.out 2>&1; echo "[regen-fold@base] rc=$?"
echo "  bytes=$(wc -c < /tmp/g776_fold_base.out)"; tail -5 /tmp/g776_fold_base.out | cut -c1-240 | sed 's/^/  | /'
echo DONE_14776R $(date -u +%H:%M:%S)Z
