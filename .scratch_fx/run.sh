W=dag/test/claim/generic_arity_n_descent_witness.dag
ARM=$1
M=src/v1/stage0/src/v1_compiler_infer.rs
if [ "$ARM" = mutation ]; then
python3 - <<'PY'
p='src/v1/stage0/src/v1_compiler_infer.rs'
s=open(p).read()
i=s.index('pub fn unify_generic_child_pairs(')
j=s.index('pub fn unify_callable_generics(')
b=s[i:j]
b=b.replace('match actuals.clone().first().cloned()','match actuals.clone().last().cloned()')
b=b.replace('Rc::new(v1_rt::list_skip(&actuals, 1))','Rc::new(actuals[..actuals.len() - 1].to_vec())')
assert b!=s[i:j]
open(p,'w').write(s[:i]+b+s[j:])
PY
fi
if [ "$ARM" = revert ]; then
cp .scratch_fx/base_04_infer.dag src/v1/04_infer.dag
cp .scratch_fx/base_v1_compiler_infer.rs $M
cp .scratch_fx/base_std_source_annotation.rs src/v1/stage0/src/std_source_annotation.rs
fi
echo "##### ARM $ARM"
cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -1
one() { r=$(timeout 900 ./target/release/gunbc run --source-root dag --entry dag/test/claim/generic_arity_n_descent_witness.dag --function $1 --claim-run 2>&1 | tail -2 | tr '\n' ' ' | cut -c1-160); echo "RESULT $1 => $r"; }
export -f one
grep -oP '^test fn \K\w+' $W | xargs -P 8 -I{} bash -c 'one {}'
echo "##### END $ARM"
