set -u
export CARGO_TARGET_DIR=$PWD/target RUSTC_WRAPPER=
echo "HEAD $(git rev-parse HEAD)"
cargo build -q --release -p v1-compiler --bin gunbc --bin claim_executor || { echo BUILD0-FAIL; exit 2; }
rm -rf target/stage0-regen-candidate
./target/release/claim_executor --required-regen --source-root dag --source-root src/v2 > regen1.log 2>&1; echo "regen1 exit=$?"; grep -E 'FAIL|first_generation_equal' regen1.log | cut -c1-200 | head -3
n=0; while IFS= read -r f; do rel=${f#target/stage0-regen-candidate/src/}; if ! cmp -s "$f" "src/v1/stage0/src/$rel"; then cp "$f" "src/v1/stage0/src/$rel"; echo "  installed $rel"; n=$((n+1)); fi; done < <(find target/stage0-regen-candidate/src -type f | sort)
cargo build -q --release -p v1-compiler --bin gunbc --bin claim_executor || { echo BUILD1-FAIL; exit 2; }
rm -rf target/stage0-regen-candidate
./target/release/claim_executor --required-regen --source-root dag --source-root src/v2 > regen2.log 2>&1; echo "regen2 exit=$?"
grep -E 'refused|hard diagnostic' regen2.log | cut -c1-200 | head -3
grep -c TOSTRCENSUS regen2.log
echo "=== V1 CENSUS LINES"; grep TOSTRCENSUS regen2.log | sort -u
