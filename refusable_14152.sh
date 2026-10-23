#!/bin/bash
# WHY DOES A CLEAN TREE REFUSE MY SOURCE? The question the parent has asked since msg_0e84a10b is
# whether the rebind refusal that turns the 02_parse regen red is a TRUE or FALSE positive. Every
# answer I gave to it so far was measured in the SHARED runner workspace, which sharp-boar-42 has now
# shown carries other sessions' uncommitted .dag edits -- so the honest possibility is that my source
# only ever converged against a tree that is not main, and that at clean main it refuses for real.
# This dispatch asks exactly one question, cheaply, and prints the answer verbatim: at a pinned clean
# clone of main with ONLY my seven source files applied, what are the hard diagnostics, in full, and
# which .dag files and spans do they name? NO suite and NO self-host here on purpose: a suite on an
# unconverged seed is the mistake my last two dispatches made, and this run exists to read a receipt.
#
# SOURCE COMES FROM ORIGIN, NOT FROM A DIRECTORY. My WIP branch is pushed at
# session/gentle-carp-16 @ d4410d980a9, so the applied source is attributable to a SHA. Only these
# seven files are taken, because the rest of my diff is stage0 (generated, must come from regeneration)
# or root-level probe .sh files that must not land at all.
set -uo pipefail
export PATH="/opt/cargo/bin:/usr/local/bin:/usr/local/sbin:$PATH"
BASE_SHA=59606870e8ba3001a845627696509e8aa16bb0f0
SRC_SHA=d4410d980a9
URL=https://github.com/gunb-ai/gunbc.git
FILES="src/v1/04_infer.dag src/v1/04_types.dag src/v1/tests/claim/generic_identity_census.dag src/v1/tests/src/applied_constructor_type_equals_test.rs src/v1/tests/src/cross_coproduct_generic_locate_test.rs src/v1/tests/src/cross_coproduct_probe_discharge_test.rs src/v1/tests/src/lib.rs"
D=/tmp/refuse-14152; rm -rf $D; mkdir -p $D; LOGS=$D/logs; mkdir -p $LOGS
cd $D
timeout 900 git clone --depth 1 $URL repo > $LOGS/clone.log 2>&1 || { echo "CLONE FAILED"; tail -5 $LOGS/clone.log; exit 20; }
cd repo
echo "main HEAD at clone time=$(git rev-parse HEAD)"
timeout 600 git fetch --depth 1 origin $BASE_SHA > $LOGS/fetchbase.log 2>&1 || { echo "BASE FETCH FAILED"; exit 25; }
git checkout -q -f FETCH_HEAD || { echo "CHECKOUT FAILED"; exit 26; }
SHA=$(git rev-parse HEAD); echo "PINNED main=$SHA"; [ "$SHA" = "$BASE_SHA" ] || { echo "WRONG TREE"; exit 27; }
echo "dirty before touching anything=$(git status --porcelain | wc -l) (want 0)"
timeout 900 git fetch --depth 1 origin session/gentle-carp-16 > $LOGS/fetchsrc.log 2>&1 || { echo "SRC FETCH FAILED"; exit 28; }
echo "source branch head=$(git rev-parse FETCH_HEAD) (want prefix $SRC_SHA)"
for f in $FILES; do git show FETCH_HEAD:$f > /tmp/_f 2>/dev/null || { echo "MISSING IN BRANCH: $f"; exit 29; }; cp /tmp/_f $f; done
git status --porcelain | sed 's/^/  applied: /'
echo "my source in place: fold=$(grep -ac UnifyGenericsAcc src/v1/04_infer.dag) myHelpers=$(grep -ac 'fn type_head_carrier' src/v1/04_types.dag) testFile=$(test -f src/v1/tests/src/cross_coproduct_generic_locate_test.rs && echo yes || echo NO)"
echo "########## ONE REGEN, RECEIPT VERBATIM ##########"
cargo build --release -p v1-compiler --bin claim_executor 2>&1 | grep -E "^error|error\[|Finished" | tail -3
rm -rf target/stage0-regen-candidate 2>/dev/null
target/release/claim_executor --required-regen --source-root dag --source-root src/v2 > $LOGS/rr.log 2>&1
echo "REGEN rc=$?"
echo "--- required-regen receipt lines:"; grep -a "^required-regen" $LOGS/rr.log | cut -c1-400
echo "--- HARD DIAGNOSTICS, verbatim (this is the artifact this dispatch exists for):"
grep -anE "^type mismatch|^call shape|arm decides|hard diagnostic|refus" $LOGS/rr.log | head -30 | cut -c1-320
echo "--- every .dag file the diagnostics name, with counts:"
grep -aoE "[a-zA-Z0-9_/]+\.dag:[0-9]+-[0-9]+" $LOGS/rr.log | sed 's/:[0-9]*-[0-9]*$//' | sort | uniq -c | sort -rn | head -12
echo "--- 02_parse specifically (the pair the whole bisect is about):"
grep -an "02_parse" $LOGS/rr.log | head -8 | cut -c1-320
echo "--- candidate tree:"; ls target/stage0-regen-candidate/src 2>&1 | head -8
echo "--- log size=$(wc -c < $LOGS/rr.log) lines=$(grep -c '' $LOGS/rr.log); first 25:"; head -25 $LOGS/rr.log | cut -c1-200
echo "--- last 20:"; tail -20 $LOGS/rr.log | cut -c1-200
cp $LOGS/rr.log ./rr.log
echo "=== BEGIN-TAR ==="; tar -c rr.log 2>/dev/null | base64 -w0; echo; echo "=== END-TAR ==="
echo REFUSE_DONE
