# src/v2 per-file entry population: files OUTSIDE the dag-primary demand, compiled one at a time (--entry), same instrument and classifier as warm-crane-219's f11d2b019e7 leg.
set -u
export CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-$PWD/target} RUSTC_WRAPPER=
echo "ENTRY-RUN census instrument"
i=0
while IFS= read -r f; do i=$((i+1)); [ -n "$f" ] || continue
  timeout 900 ./target/release/gunbc compile --output-dir /tmp/zte_$i --source-root dag --source-root src/v2 --entry "$f" --target rust > zte_$i.log 2>&1; rc=$?
  k=$(grep -E "^error\[$f:" zte_$i.log | grep -c TOSTRCENSUS)
  why=""; [ $rc -ne 0 ] && [ $k -eq 0 ] && why=$(grep -vE '^error\[|^  |^warning|^advisory|^\[pre-entry|^\s*$' zte_$i.log | head -2 | tr '\n' ' ' | cut -c1-200)
  echo "ENTRY $f rc=$rc census_in_file=$k $why"
  grep -E "^error\[$f:" zte_$i.log | grep TOSTRCENSUS | sed 's/^/SITE /'
done < .census/tostring-v2-entry-files.txt
echo ENTRY-DONE
