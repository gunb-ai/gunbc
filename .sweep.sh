# usage: sweep.sh MODE SHARD NSHARDS
export GUNBC_MEMORY_BUDGET_BYTES=30064771072
MODE=$1; SH=$2; N=$3
FILES=$(awk -v s=$SH -v n=$N "NR%n==s" .sweep-files.txt)
if [ "$MODE" = base ]; then git checkout -q HEAD~1 -- src dag && git clean -fdq -- src dag; fi
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
for f in $FILES; do
  [ -f "$f" ] || { echo "ABSENT $f"; continue; }
  fns=$(grep -h '^test fn' $f | sed -E 's/^test fn ([A-Za-z0-9_]+).*/\1/' | paste -sd,)
  [ -n "$fns" ] || continue
  timeout 900 ./target/release/claim_batch --source-root dag --source-root src/v2 --entry $f --functions $fns > /tmp/o.txt 2>&1; rc=$?
  grep -E "^(PASS|FAIL) " /tmp/o.txt | sed "s|^|R $MODE $f |"
  grep -q "resolve failed" /tmp/o.txt && echo "RESOLVEFAIL $MODE $f $(grep -A1 'resolve failed' /tmp/o.txt | tail -1 | cut -c1-200)"
  echo "RC $MODE $f $rc"
done
