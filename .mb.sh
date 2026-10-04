git log --oneline -1 | cat; cargo build --release --bin claim_batch 2>&1 | tail -1
FILES="dag/test/claim/gcp_iam_converge_witness_test.dag dag/test/claim/auth/gcp_iam_bootstrap_witness_test.dag"
for f in $FILES; do FNS=$(grep "^test fn" $f | sed "s/^test fn //; s/(.*//" | paste -sd,); N=$(echo "$FNS"|tr , "\n"|wc -l); timeout 2400 ./target/release/claim_batch --source-root dag --source-root src/v2 --entry $f --functions "$FNS" > /tmp/o.txt 2>&1; rc=$?; echo "##### POSITIVE $f rc=$rc roster=$N pass=$(grep -cE "^PASS" /tmp/o.txt)"; grep -E "^FAIL|error" /tmp/o.txt | head -6 | cut -c1-400; done
