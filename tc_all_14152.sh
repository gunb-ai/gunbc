#!/bin/bash
# ONE dispatch, two measurements (parent msg_92f01032):
#  [1] TC  : same-text-refused FIELD AXIS probe against the shipped seed (node_type_equals
#            vs node_type_shape, field-by-field).
#  [2] TC2 : EMITTER IDENTITY for the verbatim same-text TypeMismatch pair at
#            src/v1/02_parse.dag spans 83460-83467 / 84172-84179 -- instrument the single
#            formatter type_mismatch_error and print span + backtrace when handed a pair
#            whose expected/got print identically. The fold's refusal has distinct text
#            ('bound as'), so a bare same-text pair must come from another of the 21 sites.
# NO git checkout / git clean on the runner (its HEAD is the main tip; it wiped the seed
# carriers once). lib.rs edited in place, restored from a /tmp backup; mirrors restored
# by cp from seedgreen/.
set -uo pipefail
export PATH="/opt/cargo/bin:/usr/local/bin:/usr/local/sbin:$PATH"
export RUSTUP_HOME=/opt/rustup
cd .
MIRROR_FILES="src/v1/stage0/src/v1_compiler_infer.rs src/v1/stage0/src/v1_compiler_infer_types.rs src/v1/stage0/src/v1_compiler_emit_rust.rs src/v1/stage0/src/v1_tests_claim_generic_identity_census.rs"
restore_mine() { for f in $MIRROR_FILES; do cp "seedgreen/$(basename $f)" "$f"; done; }
restore_mine
echo "TC: seedU=$(grep -c UnifyGenericsAcc src/v1/stage0/src/v1_compiler_infer.rs) sha=$(sha256sum src/v1/stage0/src/v1_compiler_infer.rs | cut -c1-16) (want 5e9c37585fe088ac)"

echo "########## [1] TC FIELD AXIS ##########"
cp src/v1/tests/src/lib.rs /tmp/lib.rs.bak.tc
cp tc_probe_body.rs src/v1/tests/src/tc_probe_test.rs
grep -q 'mod tc_probe_test;' src/v1/tests/src/lib.rs || echo 'mod tc_probe_test;' >> src/v1/tests/src/lib.rs
cargo test -p v1-compiler-tests --lib tc_ -- --nocapture 2>&1 | grep -E "^TC |^error|test result|panicked" | head -60
cp /tmp/lib.rs.bak.tc src/v1/tests/src/lib.rs
rm -f src/v1/tests/src/tc_probe_test.rs
echo "TC: restored lib.rs"

echo "########## [2] TC2 EMITTER IDENTITY ##########"
python3 - <<'PY'
p="src/v1/stage0/src/v1_compiler_infer.rs"
s=open(p,encoding='utf-8').read()
anchor="""pub fn type_mismatch_error(
    expected: String,
    got: String,
    span: Rc<SourceSpan>,
    module_name: String,
) -> Rc<ErrorNode> {
"""
assert s.count(anchor)==1, "anchor count %d" % s.count(anchor)
inject = anchor + """    if std::env::var("GUNBC_EMIT_TRACE").is_ok() {
        let e0 = expected.replace(" (first argument)", "").replace(" (later argument)", "");
        let g0 = got.replace(" (later argument)", "");
        if e0 == g0 {
            eprintln!(
                "EMITTRACE span={}:{}-{} module={} expected={:?} got={:?}\\n{}",
                span.file, span.start, span.end, module_name, expected, got,
                std::backtrace::Backtrace::force_capture()
            );
        }
    }
"""
s=s.replace(anchor, inject)
open(p,'w',encoding='utf-8').write(s)
print("TC2: instrumented type_mismatch_error")
PY
cargo build --release -p v1-compiler --bin gunbc --bin claim_executor 2>&1 | tail -3
echo "TC2: running regen round (dag corpus) with trace armed"
GUNBC_EMIT_TRACE=1 timeout 2400 target/release/claim_executor --regen-round-cost --source-root dag --source-root src/v2 > /tmp/tc2_regen.log 2>&1
echo "rc=$? sametext=$(grep -c "expected 'Container(FreeMonoid,Primitive(element))', got 'Container(FreeMonoid,Primitive(element))'" /tmp/tc2_regen.log) bound-as=$(grep -c 'bound as' /tmp/tc2_regen.log) 02parse_lines=$(grep -c '02_parse.dag' /tmp/tc2_regen.log)"
echo "--- verbatim same-text diagnostic lines (02_parse):"
grep -m4 "expected 'Container(FreeMonoid,Primitive(element))', got 'Container(FreeMonoid,Primitive(element))'" /tmp/tc2_regen.log | cut -c1-260
echo "--- EMITTRACE count:"; grep -c "^EMITTRACE" /tmp/tc2_regen.log
echo "--- EMITTRACE headers:"; grep "^EMITTRACE" /tmp/tc2_regen.log | cut -c1-220 | head -8
echo "--- producing frames (v1_compiler symbols, ranked):"
grep -A30 "^EMITTRACE" /tmp/tc2_regen.log | grep -oE "v1_compiler[a-z_0-9]*::[a-z_0-9]+" | sort | uniq -c | sort -rn | head -15
restore_mine
echo "TC2: restored; seedU=$(grep -c UnifyGenericsAcc src/v1/stage0/src/v1_compiler_infer.rs) sha=$(sha256sum src/v1/stage0/src/v1_compiler_infer.rs | cut -c1-16) tracked-diff=$(git status --porcelain | grep -vc '^??')"
echo ALL_DONE
