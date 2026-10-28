set -u
export CARGO_TARGET_DIR=$PWD/target/qb
cargo build --release -p v1-compiler --bin gunbc 2>&1 | grep -E "^error|Finished" | head
G=$CARGO_TARGET_DIR/release/gunbc
pop() { e=$1
  tag=$(echo $e | tr '/.' '__'); out=/tmp/pop-$tag; rm -rf $out
  timeout 900 $G compile --source-root dag --source-root src/v2 --entry $e --output-dir $out --target rust > $out.log 2>&1
  echo "## $e rc=$? $(grep -oE '[0-9]+ blocking error' $out.log | tail -1)"
  grep -rn 'unsupported cast from' $out --include=*.rs | sed "s#$out/src/v1/stage0/src/##" | sed -E 's/^\s+//' | sort -u | cut -c1-420
}
export -f pop; export G
printf '%s\n' dag/gunbc/claim_coverage_census_live.dag dag/gunbc/instruments/roster_reach_inhabitance.dag dag/gunbc/instruments/known_red_derivation.dag src/v2/workflow/roster_import_sweep.dag src/v2/workflow/namespace_xl2_rehearsal_native.dag dag/gunbc/instruments/native_materialization_store_closure.dag dag/extdeps/linux/cgroup_v2.dag dag/extdeps/units/iso8601_calendar.dag | xargs -P 4 -I{} bash -c 'pop {} > /tmp/pop-$(echo {} | tr "/." "__").txt 2>&1'
cat /tmp/pop-*.txt
