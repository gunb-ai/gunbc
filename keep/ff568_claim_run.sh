set -uo pipefail
export PATH="/opt/cargo/bin:/usr/local/bin:$PATH"
export RUSTUP_HOME=/opt/rustup
export GITHUB_SHA="$2"
export GITHUB_RUN_ID="ff568-claim-run"
export GUNBC_MEMORY_BUDGET_BYTES=60000000000
EXPECT=$2
MUT=${3:-0}

echo "=== ff568 claim run | head=$2 | mutation=$MUT ==="

# --- fresh clone (manager: 'a fresh clone at a printed SHA'); runner has no GITHUB_TOKEN, so the
# --- clone comes from the runner workspace's object store, then checks out the explicit PR head ---
rm -rf /tmp/ff568-head
git clone -q --no-checkout /root/workspace/repo-root /tmp/ff568-head 2>&1 | head -2
cd /tmp/ff568-head
git fetch -q https://github.com/gunb-ai/gunbc session/fierce-fox-568 2>&1 | head -2
git checkout -qf "$EXPECT" 2>&1 | head -2 || { echo "CHECKOUT_FAILED"; exit 9; }
echo "HEAD=$(git rev-parse HEAD)"
echo "EXPECT=$EXPECT"
[ "$(git rev-parse HEAD)" = "$EXPECT" ] && echo "PIN_OK" || { echo "PIN_MISMATCH"; exit 9; }
echo "tracked_worktree_dirty=$(git status --porcelain | wc -l) (0 = the judge reads the committed tree only)"
echo "claim_file_sha256=$(sha256sum dag/test/claim/declared_type_kernel_inhabitance_witness.dag | cut -c1-16)"

echo "gate_seed=$(grep -o 'actual_base.children.clone().len()' src/v1/stage0/src/v1_compiler_infer.rs | wc -l)"
md5sum src/v1/stage0/src/v1_compiler_infer.rs
if [ "$MUT" = "1" ]; then
  echo "=== MUTATION: restoring the children-count bail in the SEED MIRROR ONLY (never committed) ==="
  python3 - <<'PY'
p = "src/v1/stage0/src/v1_compiler_infer.rs"
s = open(p).read()
old = """        if ((formal.connective.clone() == Connective::Arrow)
            || (actual.connective.clone() == Connective::Arrow))
        {
            break false;
        } else {"""
new = """        if (((formal.connective.clone() == Connective::Arrow)
            || (actual.connective.clone() == Connective::Arrow))
            || ((actual_base.children.clone().len() as i64) > 0))
        {
            break false;
        } else {"""
assert s.count(old) == 1, f"anchor count {s.count(old)}"
open(p, "w").write(s.replace(old, new))
print("mutation applied to", p)
PY
  echo "gate_seed_after_mutation=$(grep -o 'actual_base.children.clone().len()' src/v1/stage0/src/v1_compiler_infer.rs | wc -l) (want 1)"
fi

echo "=== BUILD claim_batch (release) ==="
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -2

ENTRY=/tmp/ff568-head/dag/test/claim/declared_type_kernel_inhabitance_witness.dag
FUNCS=(--entry "$ENTRY" \
  --function a_bool_variable_at_an_int_direct_call_argument_must_refuse \
  --function a_bool_literal_at_an_int_direct_call_argument_must_refuse \
  --function a_bool_variable_at_an_int_declared_return_must_refuse \
  --function a_bool_variable_at_an_int_record_field_must_refuse \
  --function a_bool_variable_at_an_int_let_annotation_must_refuse \
  --function a_bool_variable_at_a_free_monoid_of_int_formal_must_refuse \
  --function free_monoid_of_bool_at_a_free_monoid_of_int_formal_must_refuse \
  --function the_bool_through_int_harm_chain_must_refuse \
  --function a_bool_variable_at_a_float_formal_must_refuse \
  --function a_float_variable_at_an_int_formal_stays_admitted_by_the_declared_cast_route \
  --function an_int_variable_at_a_float_formal_stays_admitted_by_the_declared_cast_route \
  --function an_int_variable_at_a_nat_formal_stays_admitted_by_the_grounded_identity_row \
  --function a_string_at_a_free_monoid_of_char_formal_stays_admitted_by_the_declared_text_unfold \
  --function an_alias_of_int_at_an_int_formal_stays_admitted_by_name_equality \
  --function a_free_monoid_of_int_at_its_own_formal_stays_admitted \
  --function a_bool_variable_at_a_bool_formal_stays_admitted)

echo "=== BEGIN-CLAIM-RUN (mutation=$MUT) ==="
timeout 1200 ./target/release/claim_batch --source-root dag --source-root src/v2 "${FUNCS[@]}" --claim-run --wet > /tmp/ff568_claim.log 2>&1
rc=$?
grep -E "^(PASS|FAIL) " /tmp/ff568_claim.log | head -40
echo "claim_rc=$rc"
grep -cE "^PASS " /tmp/ff568_claim.log | sed 's/^/passes=/'
grep -cE "^FAIL " /tmp/ff568_claim.log | sed 's/^/fails=/'
tail -6 /tmp/ff568_claim.log | sed 's/^/tail: /'
echo "=== END-CLAIM-RUN ==="
echo "=== CLAIM RUN SCRIPT DONE (mutation=$MUT) ==="
