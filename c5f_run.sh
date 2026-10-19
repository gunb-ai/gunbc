cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -2
F=src/v2/extdeps/languages/dag.dag
T=src/v2/test/claim/parse/call_suffix_line_break.dag
fns=$(grep -oP '(?<=^test fn )\w+' $T | paste -sd,)
run(){ echo "== $1"; ./target/release/claim_batch --source-root dag --source-root src/v2 --entry $T --functions $fns > /tmp/cb.$1 2>&1; echo "rc=$?"; grep -v "typed_cache_eviction\|floor-phase\|floor-drain" /tmp/cb.$1 | tail -30; }
run head
cp $F /tmp/fixed.dag; cp c5f_old_dag.txt $F; run main_grammar; cp /tmp/fixed.dag $F
echo DONE
