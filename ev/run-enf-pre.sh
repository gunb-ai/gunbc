set -u
ROOT=$(pwd)
git fetch -q origin c4-enforcement-consistency; HEADSHA=$(git rev-parse FETCH_HEAD)
PRE=115143a4d8a1e870124e7848fa5084c3fcfe1c38
echo "HEAD $HEADSHA PRE $PRE"
git worktree add -q --detach /tmp/co3p $HEADSHA && cd /tmp/co3p && echo "CO_HEAD $(git rev-parse HEAD)"
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | grep -aE "^error|could not" | head
B=target/release/claim_batch; echo "BIN $(sha256sum $B | cut -c1-16)"
R="--wet --source-root dag --source-root src/v2"
cb(){ $B $R --entry "$1" --functions "$2" 2>&1 | grep -aE '^(PASS|FAIL|UNOBS)' | cut -c1-230; }
G=src/v2/test/claim/enforcement_live_witness.dag
FILES=$(grep -rlE "import v2\.lens\.(registry|enforcement)" --include=*.dag src dag | grep -E "test|claim|witness" | xargs grep -l '^test fn' | sort)
sweep(){ for f in $FILES; do fns=$(grep -oE '^test fn [a-z0-9_]+' $f | awk '{print $3}' | paste -sd,); [ -f $f ] && cb $f $fns | sed "s#^#$f #"; done; }
PRE=115143a4d8a1e870124e7848fa5084c3fcfe1c38
git checkout -q $PRE -- src dag; git clean -fdq src dag
echo "== PRE sweep"; sweep > /tmp/pre.sweep; echo "pre: $(grep -c ' PASS ' /tmp/pre.sweep) PASS $(grep -c ' FAIL ' /tmp/pre.sweep) FAIL"; grep -a ' FAIL ' /tmp/pre.sweep | cut -c1-120
