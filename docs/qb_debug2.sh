set -u
export CARGO_TARGET_DIR=$PWD/target/qb
B() { cargo build --release -p v1-compiler --bin gunbc --bin claim_executor 2>&1 | grep -E "^error|Finished|could not compile" | head -20; }
B
rm -rf target/stage0-regen-candidate
./target/qb/release/claim_executor --required-regen --source-root dag --source-root src/v2 > /tmp/r.log 2>&1; echo "regen exit=$?"
cp target/stage0-regen-candidate/src/v1_compiler_emit.rs src/v1/stage0/src/v1_compiler_emit.rs
B
G=$CARGO_TARGET_DIR/release/gunbc
for e in qb_git_remote_name_cast qb_local_alias_cast; do
  rm -rf /tmp/$e; $G compile --source-root dag --source-root src/v2 --entry dag/test/fixture/$e.dag --output-dir /tmp/$e --target rust > /tmp/$e.log 2>&1; echo "== $e rc=$?"
  grep -rhn "unsupported cast" /tmp/$e --include=*.rs | grep -o 'panic!("[^)]*' | sort -u | cut -c1-1200
done
