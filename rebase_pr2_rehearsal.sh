# PR-2 (#14152 Acc) rebase-onto-landed-main — PROVEN rehearsal chain (PRED-J green 2026-10-21,
# dispatch j7d7e43f9, 44m). Run at #14594-landing ping. Takes: no args (fetches origin/main +
# session/quiet-hawk-507 fresh into throwaway /tmp/pj).
# !! R1 CAVEAT: the drop-HEAD-side 04_infer heuristic PASSED pre-C1.2c (the conflict there was
#    the #14497 trace-frontier row, not the unify region). After C1.2c lands, the 04_infer conflict
#    is the unify_generic_child_pairs / arity-N region. Re-derive R1 against the landed hunk using
#    the pre-staged resolution on origin/session/gentle-carp-16-fbp @4a76af1a09b8 as the reference
#    (keep BOTH: C1.2c's unify_generic_child_pairs descent threading text+module_name into the Acc
#    fold AND the Acc arms). Verify the merged tree BUILDS (J_BUILD_RC=0) before trusting regen.
# Env is load-bearing: unset CARGO_TARGET_DIR (regen host reads workspace target/release/claim_executor),
# GITHUB_ACTIONS=1 + GUNBC_MEMORY_BUDGET_BYTES escape hatch. heal_stage0_overlap_refusal_wet RC=0 is
# the seed-build-writer ownership authority (overrides any crude git-status hand-maint grep).
set -u
export CTRL_BUILD_BYPASS_SHIMS=1
export PATH=/opt/cargo/bin:/usr/local/bin:/usr/local/sbin:$PATH
export RUSTUP_HOME=/opt/rustup
export GITHUB_ACTIONS=1
export GUNBC_MEMORY_BUDGET_BYTES=10737418240
rm -rf /tmp/pj; mkdir -p /tmp/pj
git clone -q https://github.com/gunb-ai/gunbc.git /tmp/pj/repo
cd /tmp/pj/repo
unset CARGO_TARGET_DIR
git fetch -q origin session/quiet-hawk-507 && git checkout -q FETCH_HEAD
echo "J_PR2 $(git rev-parse HEAD) MAIN $(git rev-parse origin/main)"
git merge --no-commit --no-ff origin/main > /tmp/pj/merge.log 2>&1; echo "J_MERGE_RC=$?"
git status --porcelain | grep -cE "^(UU|AA|DU|UD|DD|AU|UA)" | xargs echo "J_conflict_count:"
git rm -q dag/test/claim/constructor_tag_route_probe_red.dag 2>/dev/null; echo "R4_RC=$?"
git checkout origin/main -- src/v1/stage0/src; git add src/v1/stage0/src; echo R5_DONE
python3 - <<'PY'
p='src/v1/04_infer.dag'; L=open(p).read().split('\n')
out=[]; i=0; n=len(L)
while i<n:
    if L[i].startswith('<<<<<<<'):
        j=i
        while not L[j].startswith('======='): j+=1
        k=j
        while not L[k].startswith('>>>>>>>'): k+=1
        out.extend(L[j+1:k])
        i=k+1
    else:
        out.append(L[i]); i+=1
open(p,'w').write('\n'.join(out))
PY
git add src/v1/04_infer.dag; echo R1_DONE
python3 - <<'PY'
p='src/v1/04_types.dag'; L=open(p).read().split('\n')
out=[]; i=0; n=len(L)
while i<n:
    if L[i].startswith('<<<<<<<'):
        j=i
        while not L[j].startswith('======='): j+=1
        k=j
        while not L[k].startswith('>>>>>>>'): k+=1
        ours=L[i+1]; theirs=L[j+1]
        inner=[x.strip() for x in ours[ours.index('{')+1:ours.rindex('}')].split(',') if x.strip()]
        for t in [x.strip() for x in theirs[theirs.index('{')+1:theirs.rindex('}')].split(',') if x.strip()]:
            if t not in inner: inner.append(t)
        out.append('import v1.compiler.infer_env { ' + ', '.join(inner) + ' }')
        i=k+1
    else:
        out.append(L[i]); i+=1
open(p,'w').write('\n'.join(out))
PY
git add src/v1/04_types.dag; echo R2_DONE
sed -i '/^<<<<<<< HEAD$/d; /^=======$/d; /^>>>>>>> origin\/main$/d' dag/gunbc/recurring_failure_mode/one_type_variable_bound_to_two_distinct_coproducts.dag
git add dag/gunbc/recurring_failure_mode/one_type_variable_bound_to_two_distinct_coproducts.dag; echo R3_DONE
echo "J_remaining:"; git diff --name-only --diff-filter=U | head -5
cargo build --release -p v1-compiler --bin gunbc --bin claim_executor > /tmp/pj/b2.log 2>&1; echo "J_BUILD_RC=$?"
grep -aE "^error" /tmp/pj/b2.log | head -6
BIN=./target/release/gunbc; CX=./target/release/claim_executor
ls -la target/release/gunbc target/release/claim_executor | head -3
for ROUND in 1 2 3; do
  "$CX" --regen-round-cost --source-root dag --source-root src/v2 > /tmp/pj/r$ROUND.log 2>&1; echo "J_REGEN${ROUND}_RC=$?"
  echo "--- r$ROUND refusal lines:"
  grep -anE "refused|hard diagnostic|digest mismatch" /tmp/pj/r$ROUND.log | head -8
  echo "--- r$ROUND tail:"
  tail -8 /tmp/pj/r$ROUND.log
  if [ "$(grep -ac refused /tmp/pj/r$ROUND.log)" = "0" ]; then echo J_REGEN_IDEMPOTENT_AT=$ROUND; break; fi
done
"$BIN" run --source-root dag --source-root src/v2 --entry dag/gunbc/heal_candidate_stage0.dag --function heal_stage0_overlap_refusal_wet > /tmp/pj/r4.log 2>&1; echo "J_HEAL_RC=$?"
tail -4 /tmp/pj/r4.log
"$BIN" run --source-root dag --source-root src/v2 --entry dag/gunbc/instruments/seed_build_artifacts_regen.dag --function main_wet > /tmp/pj/r5.log 2>&1; echo "J_SEEDGEN_RC=$?"
tail -4 /tmp/pj/r5.log
echo "--- stage0 status diff:"; git status --porcelain -- src/v1/stage0/src | head -14
git status --porcelain -- src/v1/stage0/src | grep -E "cli_run.rs|module_path_index" && echo J_VIOLATION_HANDMAINT || echo J_OK_handmaint_clean
cargo build --release -p v1-compiler --bin gunbc --bin claim_executor > /tmp/pj/b5.log 2>&1; echo "J_REBUILD_RC=$?"
grep -aE "^error" /tmp/pj/b5.log | head -8
cat >> dag/test/claim/applied_constructor_type_equals.dag <<'ADD'

// AN APPLICATION AGAINST THE SAME SPELLING WITH DIFFERENT ARGUMENTS REFUSES (parent
// ruling msg_d6381674; the class PRED-C measured CLOSED by arm 2). Box<Int> and Box<Bool>
// share one declaration, so head identity cannot separate them -- the descent must compare
// the applied arguments. On main and on the #14056 leaf/binder head both were ACCEPTED
// (census count 0, a silent wrong-type accept); at the PR-2 head the applied-children
// descent refuses it. The positive above (Box<Int> vs Box<Int>) keeps the refusal from
// collapsing into over-rejection of one head.
test fn box_int_against_box_bool_is_refused() -> Bool {
  blocking_count(source: "module locate.boxneq\ntype Box<T>\nfn same_arm<A>(a: A, b: A) -> Bool { true }\nfn probe(x: Box<Int>, y: Box<Bool>) -> Bool {\n  same_arm(a: x, b: y)\n}\n") >= 1
}
ADD
"$BIN" run --claim-run --entry dag/test/claim/applied_constructor_type_equals.dag --source-root dag > /tmp/pj/c6.log 2>&1; echo "J_CLAIM_RC=$?"
grep -aE "^PASS|^FAIL" /tmp/pj/c6.log
grep -aE "NoSuchFunction|ResolveError|cannot wrap|panic" /tmp/pj/c6.log | head -3
echo PRED_J_DONE
