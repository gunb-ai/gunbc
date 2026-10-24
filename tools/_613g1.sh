set -u
cd /root/workspace/repo-root 2>/dev/null || true
# #14613 @ e36e8124af18 (GO msg_76ebec15). Dispatch 1: build + 18/18 + RED1 typed-template revert + RED2 swap revert + final.
OID=e36e8124af184e8243ddac366d71e92d5ba68fb2
echo "HEADSHA_from_gh=$OID START $(date -u +%H:%M:%S)Z"
git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/w613g
git fetch -q origin +refs/pull/14613/head:refs/verifier/14613g 2>&1 | tail -1
git rev-parse -q --verify refs/verifier/14613g >/dev/null || { echo FETCH_FAILED; exit 9; }
[ "$(git rev-parse refs/verifier/14613g)" = "$OID" ] || { echo "PR_REF_MOVED=$(git rev-parse refs/verifier/14613g) -- STOP"; exit 9; }
git worktree add -q --detach /tmp/w613g $OID || { echo WT_FAILED; exit 9; }
WT=/tmp/w613g
[ "$(git -C $WT rev-parse HEAD)" = "$OID" ] || { echo WT_MISMATCH; exit 9; }
cd $WT || exit 9
sha() { sha256sum "$1" | cut -d' ' -f1; }
AUTH=dag/gunbc/instruments/instrument_dispatch_workflow.dag
WIT=dag/test/claim/instruments/instrument_dispatch_workflow_witness.dag
cp $AUTH /tmp/613g_auth.orig
echo "BUILD_COMMIT=$(git rev-parse HEAD)"
export CARGO_TARGET_DIR=/root/workspace/repo-root/.t14613g CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
cargo build --release -p v1-compiler --bin claim_batch --bin gunbc 2>&1 | tail -1
CB=/root/workspace/repo-root/.t14613g/release/claim_batch
[ -x $CB ] || { echo "NO claim_batch -- nothing below is a result"; exit 9; }
[ "$(git rev-parse HEAD)" = "$OID" ] || { echo BUILT_AT_WRONG_COMMIT; exit 9; }
echo "CB_SHA=$(sha $CB)"
FNS=$(grep -aoE '^test fn [a-z0-9_]+' $WIT | sed 's/^test fn //' | sort | paste -sd,)
N=$(echo $FNS | tr ',' '\n' | wc -l)
echo "witness claim_count=$N (expect 18)"
run() {
  local tag=$1
  echo "-------- [$tag] $(date -u +%H:%M:%S)Z"
  timeout 1200 $CB --claim-run --source-root dag --source-root src/v2 --source-root src/v1 --entry $WIT --functions "$FNS" > /tmp/c613g_$tag.out 2> /tmp/c613g_$tag.err; local rc=$?
  [ "$rc" = "124" ] && { echo "  ROW $tag rc=124 TIMEOUT UNOBSERVED"; return 0; }
  local NP NF NU; NP=$(grep -ac '^PASS ' /tmp/c613g_$tag.out); NF=$(grep -ac '^FAIL ' /tmp/c613g_$tag.out); NU=$(grep -ac '^UNOBSERVED ' /tmp/c613g_$tag.out)
  echo "  ROW $tag rc=$rc PASS=$NP FAIL=$NF UNOBS=$NU"
  grep -aE '^(PASS|FAIL|UNOBSERVED) ' /tmp/c613g_$tag.out | sed 's/^/  V| /' | cut -c1-170
  [ "$rc" = "0" ] && [ "$((NP+NF+NU))" = "0" ] && echo "  ROW $tag !! EXIT 0 HAVING OBSERVED NOTHING -- VOID"
  [ "$rc" = "2" ] && [ "$((NP+NF+NU))" = "0" ] && echo "  ROW $tag !! rc=2 zero-verdict VOID: $(head -3 /tmp/c613g_$tag.err | tr '\n' ' ' | cut -c1-200)"
  return 0
}
echo "=== A. clean at head ==="; run clean
echo "=== B. RED1: revert typed group template to fixed head only (drop ref+label interpolations) ==="
grep -c 'instrument_dispatch_concurrency_group_template: ExpressionTemplate' $AUTH | xargs echo "  anchor template (expect 1):"
grep -c 'Interpolation { expression: ContextAccess { context: Github, path: \["ref"\] } },' $AUTH | xargs echo "  anchor ref-interp (expect 1):"
grep -c 'Interpolation { expression: ContextAccess { context: Inputs, path: \[instrument_dispatch_label_input\] } }' $AUTH | xargs echo "  anchor label-interp (expect 1):"
python3 - <<'PY'
import re,sys
p="dag/gunbc/instruments/instrument_dispatch_workflow.dag"
s=open(p).read()
pat=re.compile(r'data instrument_dispatch_concurrency_group_template: ExpressionTemplate = \{.*?\n\}\n', re.S)
m=pat.search(s)
assert m and s.count("instrument_dispatch_concurrency_group_template")>=2, "template block not found"
# mutation: keep the data declaration but strip BOTH interpolations and the separators -> fixed head only
fixed='data instrument_dispatch_concurrency_group_template: ExpressionTemplate = {\n  segments: [\n    LiteralText { text: instrument_dispatch_group_prefix }\n  ]\n}\n'
s=s[:m.start()]+fixed+s[m.end():]
open(p,"w").write(s)
t=open(p).read()
assert "Interpolation" not in t.split("instrument_dispatch_concurrency_group_template")[1].split("data instrument_dispatch_concurrency:")[0], "MUT1 interpolations remain"
print("  MUT1 applied: group template reduced to fixed head")
PY
[ $? -eq 0 ] || { echo MUT1_FAILED; exit 9; }
run mut_group
git cat-file blob HEAD:$AUTH > $AUTH; cmp -s /tmp/613g_auth.orig $AUTH && echo "  RESTORED_BYTE_IDENTICAL auth" || echo "  !!RESTORE_FAILED auth"
echo "=== C. RED2: revert the swap bound read (assign + its two print words) ==="
grep -c 'bash_build_assign(name: "memory_swap_max", value: instrument_dispatch_cgroup_read(file: "memory.swap.max")),' $AUTH | xargs echo "  anchor assign (expect 1):"
grep -c 'bash_build_word_lit(text: " memory.swap.max="),' $AUTH | xargs echo "  anchor print (expect 1):"
sed -i '/bash_build_assign(name: "memory_swap_max", value: instrument_dispatch_cgroup_read(file: "memory.swap.max")),/d' $AUTH
sed -i '/bash_build_word_lit(text: " memory.swap.max="),/d; /bash_build_word_var(name: "memory_swap_max"),/d' $AUTH
[ "$(grep -c 'memory_swap_max' $AUTH)" -lt 4 ] && echo "  MUT2 applied (remaining mentions: $(grep -c memory_swap_max $AUTH))" || { echo MUT2_NOT_APPLIED; exit 9; }
run mut_swap
git cat-file blob HEAD:$AUTH > $AUTH; cmp -s /tmp/613g_auth.orig $AUTH && echo "  RESTORED_BYTE_IDENTICAL auth" || echo "  !!RESTORE_FAILED auth"
echo "=== E. witness after restores ==="; run final
cmp -s /tmp/613g_auth.orig $AUTH && echo FINAL_MATCH_auth || echo FINAL_MISMATCH_auth
echo DONE_613_G1
cd /root/workspace/repo-root
git worktree remove --force $WT >/dev/null 2>&1 || true
git worktree prune >/dev/null 2>&1 || true
