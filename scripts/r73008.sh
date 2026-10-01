set -u
cd /home/briansrls/.worktrees/gunbc/royal-newt-820
export CARGO_TARGET_DIR=/tmp/claude-1000/royal-target; B=$CARGO_TARGET_DIR/release
bld(){ CTRL_BUILD_MODE=local RUSTC_WRAPPER= /opt/cargo/bin/cargo build --release -p v1-compiler --bin claim_batch --bin claim_executor --bin gunbc --bin infer_semantics_witness --bin v1_src_dag_parse 2>&1 | grep -E "^error|Finished"; }
echo "== emit at H"; rm -rf /tmp/claude-1000/emH; timeout 2400 $B/gunbc compile --source-root dag --source-root src/v2 --entry src/v2/compiler/00_compile.dag --target rust --output-dir /tmp/claude-1000/emH 2>&1 | grep blocking
for i in 1 2; do
 echo "== regen $i"; rm -rf target/stage0-regen-candidate; $B/claim_executor --required-regen --source-root dag --source-root src/v2 > /tmp/claude-1000/r73008_$i.txt 2>&1
 grep -E "first_generation|FAIL" /tmp/claude-1000/r73008_$i.txt | head -3 | cut -c1-200
 [ -d target/stage0-regen-candidate/src ] || { echo NO_CAND; grep -iE "error|refus" /tmp/claude-1000/r73008_$i.txt | head -8 | cut -c1-300; exit 1; }
 cp target/stage0-regen-candidate/src/*.rs src/v1/stage0/src/; echo "== build $i"; bld
done
echo "== emit new"; rm -rf /tmp/claude-1000/emN; timeout 2400 $B/gunbc compile --source-root dag --source-root src/v2 --entry src/v2/compiler/00_compile.dag --target rust --output-dir /tmp/claude-1000/emN 2>&1 | grep blocking
echo "emit diff files vs H: $(diff -rq /tmp/claude-1000/emH/src /tmp/claude-1000/emN/src | wc -l)"
for F in text_boundary_identity_wall_witness_test self_host_structural_text_witness_test text_alphabet_membership_witness_test self_host_symbol_identity_binding_witness_test emitter_string_order_present_binding_witness_test bare_name_ambiguity_wall_witness_test; do f=dag/test/claim/$F.dag; timeout 1500 $B/claim_batch --source-root dag --source-root src/v2 --entry $f --functions $(grep -oP '(?<=^test fn )\w+' $f | paste -sd,) 2>&1 | grep -E "^PASS|^FAIL|THREW|REFUS" | cut -c1-120; done > /tmp/claude-1000/claims_r73008.txt
echo "claims: $(cut -d' ' -f1 /tmp/claude-1000/claims_r73008.txt | sort | uniq -c | tr '\n' ' ')"
diff <(sort /tmp/claude-1000/claims_head2.txt | grep -E "^(PASS|FAIL)") <(sort /tmp/claude-1000/claims_r73008.txt | grep -E "^(PASS|FAIL)") && echo "claim set identical to pre-change"
cargo fmt --all --check >/dev/null 2>&1; echo fmt=$?
echo R_DONE
