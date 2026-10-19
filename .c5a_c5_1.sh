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
if [ "$1" = "0" ]; then
printf 'module std.c5x\n\ndata five: Int = 5\n\nfn f() -> Int {\n  let x = 5\n  x(3)\n}\n' > dag/std/c5x.dag
for b in pre post; do echo "== c5x $b"; /tmp/$b emit --entry std.c5x --source-root dag --source-root src/v2 --target rust 2>&1 | head -c 3500; echo; done
fi
one(){ f=$1; m=$(sed -n '1s/^module //p' $f); for b in pre post; do (timeout 420 /tmp/$b emit --entry $m --source-root dag --source-root src/v2 --target rust > /tmp/o.$b.$$ 2>&1; echo "R $m $b rc=$? $(md5sum < /tmp/o.$b.$$ | cut -c1-8) $(grep -o 'did not front-end: [a-z_]*' /tmp/o.$b.$$ | head -1 | cut -c1-90) $(head -c 1 /tmp/o.$b.$$)") & done; wait; rm -f /tmp/o.*.$$; }
export -f one
cat .c5a_sh1.txt | xargs -P 3 -I{} bash -c 'one {}'
echo DONE5
