# usage: qb_census.sh <label> <tree-dir> ; tree already has its mirror state
set -u
LABEL=$1; TREE=$2; ENTRIES=$PWD/docs/qb_entries.txt
cd $TREE
export CARGO_TARGET_DIR=$TREE/target/qb-$LABEL
cargo build --release -p v1-compiler --bin gunbc 2>&1 | grep -E "^error|Finished|could not compile" | head
G=$CARGO_TARGET_DIR/release/gunbc
one() {
  e=$1; tag=$(echo $e | tr '/.' '__'); out=$TREE/target/cen-$LABEL/$tag; rm -rf $out; mkdir -p $out
  timeout 900 $G compile --source-root dag --source-root src/v2 --entry $e --output-dir $out --target rust > $out.log 2>&1; rc=$?
  panics=$(grep -rhoE 'panic!\("unsupported cast from [^"]*"\)' $out --include=*.rs | sort | uniq -c | tr '\n' ';')
  refusals=$(grep -oE "cast cannot be emitted for target '[a-z]+' -- [^.]*" $out.log | sort | uniq -c | tr '\n' ';')
  echo "ROW $LABEL rc=$rc entry=$e files=$(find $out -name '*.rs' | wc -l) panics=[$panics] refusals=[$refusals] blocking=$(grep -oE '[0-9]+ blocking error' $out.log | tail -1)"
}
export -f one; export G TREE LABEL
sort -u $ENTRIES | xargs -P 4 -I{} bash -c 'one {}'
