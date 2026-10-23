#!/bin/bash
# TC2 EMITTER IDENTITY (parent msg_92f01032: "the probe plus the emitter identity in
# one dispatch"). Question: which site emits the verbatim same-text TypeMismatch pair
# at src/v1/02_parse.dag (spans 83460-83467 / 84172-84179, both sides printing
# Container(FreeMonoid,Primitive(element)))? The fold's own refusal has a unique text
# ('bound as'), so a bare same-text pair must come from one of the 21 OTHER
# type_mismatch_error call sites in the seed. Method: instrument the single formatter
# v1_compiler_infer::type_mismatch_error to print span + backtrace when it is handed a
# pair whose expected/got print IDENTICALLY. Throwaway seed patch; mirrors restored by
# cp from seedgreen/ (NEVER git checkout on the runner).
set -uo pipefail
export PATH="/opt/cargo/bin:/usr/local/bin:/usr/local/sbin:$PATH"
export RUSTUP_HOME=/opt/rustup
cd .
SEED=src/v1/stage0/src/v1_compiler_infer.rs
MIRROR_FILES="src/v1/stage0/src/v1_compiler_infer.rs src/v1/stage0/src/v1_compiler_infer_types.rs src/v1/stage0/src/v1_compiler_emit_rust.rs src/v1/stage0/src/v1_tests_claim_generic_identity_census.rs"
restore_mine() { for f in $MIRROR_FILES; do cp "seedgreen/$(basename $f)" "$f"; done; }
restore_mine
echo "TC2: seedU=$(grep -c UnifyGenericsAcc $SEED) sha=$(sha256sum $SEED | cut -c1-16) (want 5e9c37585fe088ac)"
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
                "EMITTRACE same-text span={}:{}-{} module={} expected={:?} got={:?}\\n{}",
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
if ! [ -x target/release/claim_executor ]; then cargo build --release -p v1-compiler --bin gunbc --bin claim_executor 2>&1 | tail -2; fi
cargo build --release -p v1-compiler --bin gunbc --bin claim_executor 2>&1 | tail -2
echo "########## TC2 [merged .dag + instrumented fold seed, regen round on dag corpus] ##########"
GUNBC_EMIT_TRACE=1 target/release/claim_executor --regen-round-cost --source-root dag --source-root src/v2 > /tmp/tc2_regen.log 2>&1
echo "rc=$? sametext=$(grep -c "expected 'Container(FreeMonoid,Primitive(element))', got 'Container(FreeMonoid,Primitive(element))'" /tmp/tc2_regen.log) bound-as=$(grep -c 'bound as' /tmp/tc2_regen.log)"
echo "--- 02_parse spans present?"; grep -c "02_parse.dag" /tmp/tc2_regen.log
echo "--- EMITTRACE count:"; grep -c "^EMITTRACE" /tmp/tc2_regen.log
echo "--- EMITTRACE lines for 02_parse (max 4), verbatim:"
grep -A28 "^EMITTRACE" /tmp/tc2_regen.log | grep -B0 -A28 "02_parse" | head -60 | cut -c1-200
echo "--- EMITTRACE distinct producing frames (top):"
grep -A24 "^EMITTRACE" /tmp/tc2_regen.log | grep -oE "v1_compiler[a-z_:]*[a-z_]+" | sort | uniq -c | sort -rn | head -12
restore_mine
echo "TC2: restored; seedU=$(grep -c UnifyGenericsAcc $SEED) sha=$(sha256sum $SEED | cut -c1-16) tracked-diff=$(git status --porcelain | grep -vc '^??')"
echo TC2_DONE
