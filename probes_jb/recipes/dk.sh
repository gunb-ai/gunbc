cd /tmp/jb/di
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
F=$(grep -oE "^test fn [a-z0-9_]+" dag/test/claim/defork_type_name_census_witness.dag | awk '{print $3}' | paste -sd,)
timeout 2400 ./target/release/claim_batch --source-root dag --source-root src/v2 --entry $PWD/dag/test/claim/defork_type_name_census_witness.dag --functions "$F" > /tmp/dk.log 2>&1
echo "rc=$?"; git rev-parse HEAD
grep -vE '^\[pre-entry\]|typecheck' /tmp/dk.log | cut -c1-300 | tail -15
echo ==END
