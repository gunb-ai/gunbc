set +e
export CARGO_TARGET_DIR=/root/workspace/cargo-target-persist
mk(){ echo "[$(date +%H:%M:%S)] $*"; }
R=/root/workspace/probe-root
rm -rf $R; mkdir -p $R; cd $R || { echo NO_DIR; exit 0; }
git init -q . && git remote add origin https://github.com/gunb-ai/gunbc.git
E=dag/test/claim/serving_caller_backlog_witness.dag
F=w_persist_before_offer_and_duplicate_persist_is_one_waiting_record
FEAT=text_lookup_work_counter,interp_test_witness,test_hooks
checkout(){ SHA=$1; timeout 600 git fetch -q --depth=1 origin $SHA || { echo "FETCH_FAIL $SHA"; return 1; }; git checkout -q FETCH_HEAD; [ "$(git rev-parse HEAD)" = "$SHA" ] && echo "TREE_OK $SHA" || { echo "TREE_BAD $(git rev-parse HEAD) want $SHA"; return 1; }; }
cb_probe(){ SHA=$2; echo "== CLAIM_BATCH $SHA"; cargo build -p v1-compiler --release --features $FEAT --bin claim_batch 2>&1 | tail -1; timeout 900 $CARGO_TARGET_DIR/release/claim_batch --source-root $PWD/dag --source-root $PWD/src/v2 --entry $E --functions "$(head -33 /dev/null; true)" >/dev/null 2>&1; }
compile_probe(){ SHA=$1; echo "== GUNBC_COMPILE $SHA t=$(date +%H:%M:%S)"; timeout 900 $CARGO_TARGET_DIR/release/gunbc compile --entry $E --source-root $PWD/dag --source-root $PWD/src/v2 > /tmp/gc_${SHA:0:8}.log 2>&1; echo "COMPILE_RC=$? lines=$(wc -l < /tmp/gc_${SHA:0:8}.log)"; grep -aE 'type mismatch|refus|error' /tmp/gc_${SHA:0:8}.log | head -4 | cut -c1-220; }
cb_full(){ SHA=$1; CSV=$(awk '/^test fn/{sub(/^test fn /,"");sub(/\(.*/,"");print}' $E | paste -sd,); echo "== CLAIM_BATCH33 $SHA t=$(date +%H:%M:%S)"; timeout 900 $CARGO_TARGET_DIR/release/claim_batch --source-root $PWD/dag --source-root $PWD/src/v2 --entry $E --functions "$CSV" > /tmp/cb_${SHA:0:8}.log 2>&1; echo "CB_RC=$? PASS=$(grep -acE '^PASS ' /tmp/cb_${SHA:0:8}.log) FAIL=$(grep -acE '^FAIL ' /tmp/cb_${SHA:0:8}.log)"; grep -aE 'type mismatch|refus|error' /tmp/cb_${SHA:0:8}.log | head -4 | cut -c1-220; }
checkout 3b620b17da63e0e412fcb3cbea9ad103d606046b && { cargo build -p v1-compiler --release --features $FEAT --bin claim_batch --bin gunbc 2>&1 | tail -1; mk "3B62 BUILD_RC=$?"; cb_full 3b620b17da63e0e412fcb3cbea9ad103d606046b; compile_probe 3b620b17da63e0e412fcb3cbea9ad103d606046b; }
checkout 6ef957f196c04e91d7e5374720adaf2413c51a1c && { cargo build -p v1-compiler --release --features $FEAT --bin gunbc 2>&1 | tail -1; mk "MAIN BUILD_RC=$?"; compile_probe 6ef957f196c04e91d7e5374720adaf2413c51a1c; }
checkout 2d79743e2498bda7a95e5b6253ca2e7fd73aa6a7 && { cargo build -p v1-compiler --release --features $FEAT --bin gunbc 2>&1 | tail -1; mk "PARENT BUILD_RC=$?"; compile_probe 2d79743e2498bda7a95e5b6253ca2e7fd73aa6a7; }
checkout 242ac98b5feff52f11fe56a7732cffb9897f1813 && { cargo build -p v1-compiler --release --features $FEAT --bin gunbc 2>&1 | tail -1; mk "SUSPECT BUILD_RC=$?"; compile_probe 242ac98b5feff52f11fe56a7732cffb9897f1813; }
checkout 0f60a41429cd4d53a8a4b5264c604bd241d21403 && { cargo build -p v1-compiler --release --features $FEAT --bin claim_batch 2>&1 | tail -1; mk "0F60 BUILD_RC=$?"; cb_full 0f60a41429cd4d53a8a4b5264c604bd241d21403; }
mk ALL_DONE_F9
