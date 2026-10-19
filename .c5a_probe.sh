export RUNNER_TEMP=$(mktemp -d)
git config --global --add safe.directory '*' 2>/dev/null
cargo build --release -p v1-compiler --bin gunbc >/dev/null 2>&1
F=src/v2/extdeps/languages/dag.dag
cp $F /tmp/fixed.dag
gen(){ rm -rf $RUNNER_TEMP/*; ./target/release/gunbc test //gunbc/instruments:v2-native-cli >/dev/null 2>&1; cp $(ls $RUNNER_TEMP/* | head -1) /tmp/$1; }
gen post
git show HEAD:$F > $F
gen pre
cp /tmp/fixed.dag $F
printf 'module std.c5x\n\ndata five: Int = 5\n\nfn f() -> Int {\n  let x = 5\n  x(3)\n}\n' > dag/std/c5x.dag
for b in pre post; do echo "== c5x $b"; /tmp/$b emit --entry std.c5x --source-root dag --source-root src/v2 --target rust 2>&1 | head -c 3500; echo; done
echo DONE6
