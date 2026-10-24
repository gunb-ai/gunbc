set -u
export CARGO_TARGET_DIR=$PWD/target RUSTC_WRAPPER=
echo "ENTRY-RUN census instrument"
cargo build -q --release -p v1-compiler --bin gunbc --bin claim_executor || { echo BUILD0-FAIL; exit 2; }
rm -rf target/stage0-regen-candidate
./target/release/claim_executor --required-regen --source-root dag --source-root src/v2 > regen1.log 2>&1; echo "regen1 exit=$?"
n=0; while IFS= read -r f; do rel=${f#target/stage0-regen-candidate/src/}; if ! cmp -s "$f" "src/v1/stage0/src/$rel"; then cp "$f" "src/v1/stage0/src/$rel"; n=$((n+1)); fi; done < <(find target/stage0-regen-candidate/src -type f | sort); echo "installed $n"
cargo build -q --release -p v1-compiler --bin gunbc || { echo BUILD1-FAIL; exit 2; }
i=0
while IFS= read -r f; do i=$((i+1)); [ -n "$f" ] || continue
  timeout 900 ./target/release/gunbc compile --output-dir /tmp/te_$i --source-root dag --source-root src/v2 --entry "$f" --target rust > te_$i.log 2>&1; rc=$?
  k=$(grep -E "^error\[$f:" te_$i.log | grep -c TOSTRCENSUS)
  why=""; [ $rc -ne 0 ] && [ $k -eq 0 ] && why=$(grep -vE '^error\[|^  |^warning|^advisory|^\[pre-entry|^\s*$' te_$i.log | head -2 | tr '\n' ' ' | cut -c1-200)
  echo "ENTRY $f rc=$rc census_in_file=$k $why"
  grep -E "^error\[$f:" te_$i.log | grep TOSTRCENSUS | sed 's/^/SITE /'
done < .uncov_rest.txt
echo ENTRY-DONE
