set -u
cd /root/workspace/repo-root 2>/dev/null || true
# #14613 @ b7854d98d76 GO extras (msg_e450a455): (1) rfm-rung-census at head ADMITS the edited row;
# (2) row static check: no digits/number-words describing the population; (3) population re-derivable by the row's own search.
OID=b7854d98d76584c65b04df7ce1a971b1ab7322fc
ROWF=dag/gunbc/recurring_failure_mode/workflow_expressions_assembled_from_string_fragments.dag
echo "HEADSHA_from_gh=$OID START $(date -u +%H:%M:%S)Z"
git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/w613i
git fetch -q origin +refs/pull/14613/head:refs/verifier/14613g 2>&1 | tail -1
[ "$(git rev-parse refs/verifier/14613g 2>/dev/null)" = "$OID" ] || { echo "PR_REF_MOVED=$(git rev-parse refs/verifier/14613g 2>/dev/null) -- STOP"; exit 9; }
git worktree add -q --detach /tmp/w613i $OID || { echo WT_FAILED; exit 9; }
WT=/tmp/w613i
[ "$(git -C $WT rev-parse HEAD)" = "$OID" ] || { echo WT_MISMATCH; exit 9; }
cd $WT || exit 9
sha() { sha256sum "$1" | cut -d' ' -f1; }
echo "BUILD_COMMIT=$(git rev-parse HEAD)"
export CARGO_TARGET_DIR=/root/workspace/repo-root/.t14613i CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -1
GB=/root/workspace/repo-root/.t14613i/release/gunbc
[ -x $GB ] || { echo "NO gunbc -- nothing below is a result"; exit 9; }
echo "GB_SHA=$(sha $GB)"
echo "-------- I. rfm-rung-census at head (UNBOUND, own tree) $(date -u +%H:%M:%S)Z"
( cd $WT && timeout 2400 $GB test //gunbc/instruments:rfm-rung-census ) > /tmp/c613i_census.out 2>&1; rc=$?
NP=$(grep -ac '^PASS ' /tmp/c613i_census.out); NF=$(grep -ac '^FAIL ' /tmp/c613i_census.out); NU=$(grep -ac '^UNOBSERVED ' /tmp/c613i_census.out)
echo "  ROW census rc=$rc PASS=$NP FAIL=$NF UNOBS=$NU"
grep -aE '^(PASS|FAIL|UNOBSERVED) ' /tmp/c613i_census.out | sed 's/^/  V| /' | cut -c1-170
grep -aiE 'rows=|read=|roster' /tmp/c613i_census.out | tail -4 | sed 's/^/  S| /' | cut -c1-180
[ "$rc" = "0" ] && [ "$((NP+NF+NU))" = "0" ] && echo "  ROW census !! rc=0 zero-verdict VOID -- not a pass"
grep -a 'workflow_expressions_assembled_from_string_fragments' /tmp/c613i_census.out | head -4 | sed 's/^/  R| /' | cut -c1-190
echo "-------- J. row static: digits + number words $(date -u +%H:%M:%S)Z"
echo "  J1 digit-lines (context, every one must be provenance/pointer, none a population figure):"
grep -naE '[0-9]' $ROWF | sed 's/^/  D| /' | cut -c1-230
echo "  J2 number-word occurrences (population words nearby = flag; verdict in receipt):"
grep -naiE '\b(one|two|three|four|five|six|seven|eight|nine|ten|eleven|twelve|dozen|dozens|hundred|thousands?|millions?|several|many|multiple|count of|[0-9]+ sites?|[0-9]+ files?)\b' $ROWF | sed 's/^/  W| /' | cut -c1-230
echo "  J3 'the population' sentences (must be re-derive language, no figure):"
grep -naiE 'population' $ROWF | sed 's/^/  P| /' | cut -c1-230
echo "-------- K. population re-derivable by the row's own search $(date -u +%H:%M:%S)Z"
grep -rlE '"[^"]*\$\{\{' dag/ --include=*.dag 2>/dev/null | grep -v dag/test/claim | grep -v extdeps/github/expressions | sort > /tmp/c613i_pop.head.txt
CNT=$(wc -l < /tmp/c613i_pop.head.txt)
echo "  ROW K search_count=$CNT (row writes NO figure; the search is the only producer)"
head -5 /tmp/c613i_pop.head.txt | sed 's/^/  K| /'
grep -c 'fleet_converge' /tmp/c613i_pop.head.txt | xargs echo "  K fleet_converge hits (row claims 'carries the bulk'):"
grep -acE 'instrument_dispatch_workflow|heal_workflow' /tmp/c613i_pop.head.txt | xargs echo "  K converted-model presence (expect 0 — this PR converted them):"
echo "  census-vs-row admission: $(grep -ac 'workflow_expressions_assembled_from_string_fragments' /tmp/c613i_census.out) census lines name the edited identity"
git status --porcelain | head -3 | sed 's/^/  STALE| /'
echo DONE_613_I
cd /root/workspace/repo-root
git worktree remove --force $WT >/dev/null 2>&1 || true
git worktree prune >/dev/null 2>&1 || true
