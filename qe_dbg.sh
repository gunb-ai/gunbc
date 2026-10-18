cd "$(git rev-parse --show-toplevel)"
export CARGO_TARGET_DIR=$PWD/target
python3 - <<'P'
p='src/v1/stage0/src/v1_compiler_infer.rs'
s=open(p).read()
a="                let resolved_node = if ((!is_present_ctor.clone()\n"
assert s.count(a)==1
b='                eprintln!("QE_ABS present_ctor={} parent_std={} expected={:?}", is_present_ctor, parent_is_std_optional, expected.as_ref().map(|e| (e.name.clone(), e.return_cardinality.clone(), e.inferred.is_some())));\n'
open(p,'w').write(s.replace(a,b+a))
P
/opt/cargo/bin/cargo build --release --bin gunbc 2>&1 | tail -2
target/release/gunbc run --source-root dag --source-root src/v2 --entry dag/test/claim/qe_scratch.dag --function qe_main > /tmp/qe_dbg.out 2>&1; echo rc=$?
grep -a QE_ABS /tmp/qe_dbg.out | grep -av "Required, \(true\|false\)))$" | sort | uniq -c | head -30 | cut -c1-300; grep -ac QE_ABS /tmp/qe_dbg.out
tail -3 /tmp/qe_dbg.out | cut -c1-300
git checkout src/v1/stage0/src/v1_compiler_infer.rs
