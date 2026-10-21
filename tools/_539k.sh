set -u
cd /root/workspace/repo-root 2>/dev/null || true
# #14539 @ aef4b0138d3 -- clean HEAD-BUILT arms for the exit split.
# Attempt-5 lesson (disclosed): its first cargo build ran BEFORE cd $WT, so P1-P11 ran the MAIN-built
# binary (main prints 'FAIL x (entry resolve failed: <path>)' at claim_batch.rs:917 -- a string that does
# not exist at head). Only M1a/R1 there were head-built. Here: the ONLY build happens after cd $WT, and
# the source is hashed before and after the build.
OID=aef4b0138d3077866fa601001ea9a2bb75cd19e8  # gh pr view 14539 --json headRefOid
echo "HEADSHA_from_gh=$OID"
echo "PWD=$(pwd) RUNNER_HEAD=$(git rev-parse HEAD | cut -c1-12) START $(date -u +%H:%M:%S)Z"
git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/wtk
git fetch -q origin refs/pull/14539/head 2>&1 | tail -1
[ "$(git rev-parse FETCH_HEAD)" = "$OID" ] || { echo "PR_REF_MOVED fetched=$(git rev-parse FETCH_HEAD) want=$OID -- STOP"; exit 9; }
git worktree add -q --detach /tmp/wtk $OID || { echo "WORKTREE_ADD_FAILED"; exit 9; }
WT=/tmp/wtk
[ "$(git -C $WT rev-parse HEAD)" = "$OID" ] || { echo "WORKTREE_NOT_AT_HEAD"; exit 9; }
echo "WORKTREE_HEAD=$(git -C $WT rev-parse HEAD)"
sha() { sha256sum "$1" | cut -d' ' -f1; }
CBRS=src/v1/stage0/src/bin/claim_batch.rs
cd $WT || exit 9
[ "$(sha $CBRS)" = "b74e7b5e4dee235fd37b0eb900e57bdb318f489c23930ab8b563b6f6acd32980" ] || { echo "SOURCE_NOT_HEAD_BLOB -- STOP"; exit 9; }
mkdir -p /root/workspace/repo-root/tools/stage && cp -f $CBRS /root/workspace/repo-root/tools/stage/kb_head.rs
export CARGO_TARGET_DIR=/root/workspace/repo-root/.tk14539 CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
CB=/root/workspace/repo-root/.tk14539/release/claim_batch
[ -x $CB ] || { echo "NO BINARY - nothing below is a result"; exit 9; }
[ "$(git rev-parse HEAD)" = "$OID" ] || { echo "BUILT_AT_WRONG_COMMIT"; exit 9; }
echo "CB_SHA=$(sha $CB | cut -c1-12) BUILT_AT $(date -u +%H:%M:%S)Z (built INSIDE the head worktree)"
ls -l --time-style=+%H:%M:%S $CB | awk '{print "  binary mtime",$6}'
# the binary must contain head's string and NOT main's
echo "strings gate: closure-diagnostics=$(strings $CB | grep -c 'closure diagnostics in') main-entry-resolve=$(strings $CB | grep -c 'entry resolve failed')"

V=src/v2/test/claim/verifier539k
mkdir -p $V tools/verifier539k
cat > $V/okmod.dag <<'DAG'
module v2.test.claim.verifier539k.okmod

test fn ok_helper() -> Bool {
  true
}
DAG
cat > $V/good_entry.dag <<'DAG'
module v2.test.claim.verifier539k.good_entry

import v2.test.claim.verifier539k.okmod { ok_helper }

test fn good_entry_holds() -> Bool {
  ok_helper()
}
DAG
cat > $V/island_badname.dag <<'DAG'
module v2.test.claim.verifier539k.island_badname

import v2.test.claim.verifier539k.okmod { zz_not_a_real_name_539 }

test fn island_claim() -> Bool {
  true
}
DAG
cat > $V/island_badtype.dag <<'DAG'
module v2.test.claim.verifier539k.island_badtype

test fn island_type_claim() -> Bool {
  7
}
DAG
# a host-capability claim: hermetic => route gap (UNOBSERVED, 2); wet => PASS (0)
cat > $V/hostcap.dag <<'DAG'
module v2.test.claim.verifier539k.hostcap

import extdeps.github.users { GitHubUser }

@host_service(name: "github.users.get")
fn github_users_get(login: String) -> GitHubUser {
  GitHubUser { login: login, id: 1 as Int, name: "x" } with mock_response: [
    GitHubUser { login: "octocat", id: 1 as Int, name: "The Octocat" }
  ]
}

test fn verifier_plant_hostcap_reads_the_mock() -> Bool {
  let u = github_users_get(login: "octocat")
  u.login == "octocat" && u.id == 1 as Int
}
DAG
# an entry that EXISTS but sits outside every --source-root: not in the module index
cat > tools/verifier539k/outside_index.dag <<'DAG'
module verifier539k.outside_index

test fn outside_claim() -> Bool {
  true
}
DAG
git add -f $V tools/verifier539k >/dev/null
echo "planted: $(ls $V | tr '\n' ' ')|$(ls tools/verifier539k)  tracked=$(git ls-files $V tools/verifier539k | wc -l)/5"

arm() { # label route roots... -- entry fn [entry fn ...]
  local label=$1; shift
  local route=$1; shift
  local roots=""
  while [ "${1:-}" != "--" ]; do roots="$roots $1"; shift; done; shift
  local argv=()
  while [ "$#" -ge 2 ]; do argv+=(--entry "$1" --functions "$2"); shift 2; done
  echo "-------- [$label] $route roots:$roots $(date -u +%H:%M:%S)Z"
  echo "  argv: $CB --claim-run $route $roots $*" | cut -c1-230
  timeout 900 "$CB" --claim-run $route $roots ${argv[@]+"${argv[@]}"} > /tmp/k_$label.out 2> /tmp/k_$label.err
  local rc=$?
  if [ "$rc" = "124" ]; then echo "  rc=124 TIMEOUT -- EXCLUDED, not a result"; return 0; fi
  echo "  rc=$rc stdout_bytes=$(wc -c < /tmp/k_$label.out) stderr_bytes=$(wc -c < /tmp/k_$label.err)"
  echo "  --- stdout:"; sed 's/^/    O| /' /tmp/k_$label.out | head -12 | cut -c1-200
  local ab=$(grep -ac 'claim_batch: ' /tmp/k_$label.err)
  local em=$(grep -ac '^UNOBSERVED .*entry resolve failed' /tmp/k_$label.out)
  local rg=$(grep -ac '^UNOBSERVED .*route gap' /tmp/k_$label.out)
  local ri=$(grep -ac '^UNOBSERVED .*resolve did not reach a verdict' /tmp/k_$label.out)
  local cd=$(grep -ac '^FAIL .*closure diagnostics in' /tmp/k_$label.out)
  local pf=$(grep -ac 'resolved against the process working directory' /tmp/k_$label.err)
  local md=$(grep -acE 'mode|hermetic|wet' /tmp/k_$label.out)
  echo "  counters: PASS=$(grep -ac '^PASS ' /tmp/k_$label.out) FAIL=$(grep -ac '^FAIL ' /tmp/k_$label.out) UNOBSERVED=$(grep -ac '^UNOBSERVED ' /tmp/k_$label.out) | producers: closureDiagnostics-FAIL=$cd entryMissing=$em routeGap=$rg infra=$ri cli-boundary-missing-file=$pf cli-refusal=$ab out-mode-lines=$md"
  [ "$cd" != "0" ] && echo "  >>> CLOSURE-DIAGNOSTICS (finding, expect rc=1)"
  [ "$rg" != "0" ] && echo "  >>> ROUTE GAP (unobserved, expect rc=2)"
  [ "$em" != "0" ] && echo "  >>> ENTRY MISSING (unobserved, expect rc=2)"
  if [ "$rc" != "0" ] && [ "$cd$em$rg$ri" = "0000" ]; then echo "  !! NONZERO EXIT WITH NO TYPED VERDICT LINE -- earlier layer; read stderr tail:"; tail -4 /tmp/k_$label.err | sed 's/^/    Et| /' | cut -c1-190; fi
  [ "$rc" = "0" ] && [ "$(grep -ac '^PASS ' /tmp/k_$label.out)" = "0" ] && echo "  !! EXIT 0 HAVING OBSERVED NOTHING"
  return 0
}
W="--source-root dag --source-root src/v2 --source-root src/v1"
echo "=== K1 control: healthy entry => PASS rc=0"; arm K1 --hermetic $W -- $V/good_entry.dag good_entry_holds
echo "=== K2 name error in the entry closure => FAIL(closure diagnostics) rc=1"; arm K2 --hermetic $W -- $V/island_badname.dag island_claim
echo "=== K3 type error in the entry closure => FAIL(closure diagnostics) rc=1"; arm K3 --hermetic $W -- $V/island_badtype.dag island_type_claim
echo "=== K4 MIXED: healthy finding group + closure-diagnostics group => rc=1 (1 beats 1)"; arm K4 --hermetic $W -- $V/good_entry.dag good_entry_holds $V/island_badname.dag island_claim
echo "=== K5 MIXED order swapped"; arm K5 --hermetic $W -- $V/island_badname.dag island_claim $V/good_entry.dag good_entry_holds
echo "=== K6 unobserved alone: entry outside every --source-root => UNOBSERVED rc=2"; arm K6 --hermetic $W -- tools/verifier539k/outside_index.dag outside_claim
echo "=== K7 host capability, hermetic => route gap UNOBSERVED rc=2 (NOT FAIL)"; arm K7 --hermetic $W -- $V/hostcap.dag verifier_plant_hostcap_reads_the_mock
echo "=== K8 host capability, wet => PASS rc=0"; arm K8 --wet $W -- $V/hostcap.dag verifier_plant_hostcap_reads_the_mock
echo "=== K9 nonexistent --entry file (the declared CLI-boundary refusal, exit 2)"; arm K9 --hermetic $W -- $V/no_such_file_539.dag whatever_claim
echo "=== K10 MIXED: a finding (name error) beside an unobserved subject (outside index) => rc=1"; arm K10 --hermetic $W -- $V/island_badname.dag island_claim tools/verifier539k/outside_index.dag outside_claim

echo "=== U1 the mixed-composition unit test, by name, selection line required"
timeout 1500 cargo test -p v1-compiler --bin claim_batch exit_composition_is_failed_over_unobserved_over_clean -- --nocapture > /tmp/k_u1.log 2>&1
echo "  [U1] rc=$? selected=[$(grep -aoE 'running [0-9]+ test' /tmp/k_u1.log | head -1)] result=[$(grep -a 'test result:' /tmp/k_u1.log | head -1 | cut -c1-70)]"
grep -aE '^test [a-z]' /tmp/k_u1.log | head -3 | sed 's/^/    T| /' | cut -c1-160
tail -12 /tmp/k_u1.log | grep -aE 'error|FAILED|panicked' | sed 's/^/    Tt| /' | cut -c1-170

echo "=== SUMMARY-LINE mode print: what a passing run prints"
grep -aE 'mode|MODE|hermetic|wet' /tmp/k_K1.out | head -5 | sed 's/^/    S| /' | cut -c1-190
echo DONE_14539_K
cd /root/workspace/repo-root
git worktree remove --force $WT >/dev/null 2>&1 || true
git worktree prune >/dev/null 2>&1 || true
