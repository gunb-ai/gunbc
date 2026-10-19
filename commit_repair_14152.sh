#!/usr/bin/env bash
# Commit the producer repair to the WIP tree: the .dag change from the patcher, and the
# mirror files EXACTLY as the remote regeneration emitted them (the sanctioned way a mirror
# changes -- mirrors are never hand-edited or line-merged). Records every seed so the commit
# can be tied to the run that produced it.
#
#   commit_repair_14152.sh <changed.tgz> <measured-sha> <log-dir>
#
# Refuses to commit unless:
#   * the patcher's marker is present in the .dag and the mirror (the regeneration actually
#     carried the change -- a moved .dag with an unmoved mirror is the StaleBinary trap);
#   * the mirror's UnifyGenericsAcc presence count is unchanged (the fold survived);
#   * the local parse gate says the .dag parses;
#   * the working tree has no other modifications.
set -u -o pipefail
TGZ="${1:?usage: commit_repair_14152.sh <changed.tgz> <measured-sha> <log-dir>}"
MEASURED_AT="${2:?need the measured SHA the regeneration ran at}"
LOGD="${3:?need the dispatch log dir}"
REPO=/home/briansrls/.worktrees/gunbc/gentle-carp-16
INSTR=/home/briansrls/.worktrees/gunbc/gentle-carp-16-scratch
INFER_DAG=src/v1/04_infer.dag
MIRROR=src/v1/stage0/src/v1_compiler_infer.rs

cd "$REPO" || exit 1
[ -z "$(git status --porcelain | grep -v '^??')" ] || { echo "FATAL working tree is dirty"; git status --porcelain | grep -v '^??' | head; exit 1; }
BASE=$(git rev-parse HEAD)
echo "WIP_BASE $BASE"

# 1. the .dag change, from the same patcher the remote ran (byte-for-byte the same input).
python3 "$INSTR/patch_producer_instantiate.py" "$REPO" || { echo "FATAL patch"; exit 1; }

# 2. the mirrors, from the regeneration's own tarball.
tar -tzf "$TGZ" | sed 's/^/TARBALL /'
tar -xzf "$TGZ" -C "$REPO" || { echo "FATAL extract"; exit 1; }

# 3. post-conditions.
grep -c "fresh_call_variables" "$INFER_DAG" | sed 's/^/DAG_MARKS /'
[ "$(grep -c 'fresh_call_variables' "$INFER_DAG")" -ge 2 ] || { echo "FATAL marker absent in .dag"; exit 1; }
[ "$(grep -c 'fresh_call_variables' "$MIRROR")" -ge 1 ] || { echo "FATAL the regenerated mirror does not carry the repair -- do not commit"; exit 1; }
[ "$(grep -c 'UnifyGenericsAcc' "$MIRROR")" -ge 30 ] || { echo "FATAL the fold vanished from the mirror"; exit 1; }
grep -c 'UnifyGenericsAcc' "$MIRROR" | sed 's/^/FOLD_REFS /'
python3 "$INSTR/parsecheck.py" "$REPO/$INFER_DAG" || { echo "FATAL parse"; git checkout -- "$INFER_DAG" 2>/dev/null; exit 1; }

echo "SEEDS $(sha256sum "$INFER_DAG" "$MIRROR" | cut -c1-16 | tr '\n' ' ')"
git diff --stat | tail -6 | sed 's/^/DIFFSTAT /'
git diff --name-only | sed 's/^/WILL_COMMIT /'

# Anything outside the .dag and generated mirrors must not be committed by this script.
bad=$(git diff --name-only | grep -vE "^src/v1/04_infer\.dag$|^src/v1/stage0/src/.*\.rs$" || true)
if [ -n "$bad" ]; then
  echo "FATAL unexpected files in the diff:"; echo "$bad" | sed 's/^/  /'
  git checkout -- $bad 2>/dev/null || true
  exit 1
fi

cat > /tmp/repair_commit_msg.txt <<'MSG'
fix(14152): instantiate a generic call's result when the actuals cannot solve its parameters

A call to a generic function with NO value parameters -- `fn empty_map<K, V>() -> Map<K, V>`
(dag/std/algebra.dag:1406; 1489 corpus calls, every one spelled bare `empty_map()`) -- takes
the call arm's `Present { value: s }` branch, which substitutes only what the actuals solved.
With no actuals the substitution is empty, substitute_generics_apply short-circuits on an empty
subst, and the callee's DECLARED result type is handed over as this call expression's type with
K and V still naming the DECLARATION's own parameters. The generic unification fold then
refused the real instance that arrived later:

  type mismatch: expected type variable A bound as Map(Primitive(K),Primitive(V)) (first
  argument), got Map(Primitive(String),Primitive(Int)) (later argument)

39 of the 46 Map-family rows in the two-closure gate are this one node, in 14 modules. The
other 7 are `bare_map_node` minting Map children named with std's own declared K/V; they do
not pass through this arm, remain a named residual with their own boundary, and are not a
reason to revert this fix.

The repair seeds a fresh UNSETTLED variable for every declared parameter the actuals did not
solve, through the instantiation this file already uses for unbound owner parameters
(`unbound_owner_param_variables`: fold over names, init empty_map(), map_insert(name,
type_variable_node(...))) -- no new mechanism, no new instantiation path.

The fresh id carries THIS CALL EXPRESSION'S OWN SPAN, not the bare parameter name. Naming it
`K` would be the `bare_map_node` defect reproduced at the call site: two calls to the same
generic function in one body would share variable K and could unify each other's types -- a
silent wrong binding instead of a loud conflict. The comparison that decides this:
`unbound_owner_param_variables`' substitution is consumed locally, one line later, as a
record-literal field type, and never escapes; here the substituted type escapes as the call's
`inferred` into the body environment, where other call sites' types meet it. So the precedent's
scope-safety does not transfer and the id must be per call site.

Non-generic calls are byte-identical: the seed folds over `call_sig_split.generic_names`, which
is empty when the callee declares no type parameters, so map_merge is the identity and
substitute_generics_apply short-circuits exactly as before (parent ruling msg_1249d701).

The explanation lives in this message rather than as a comment in the function body: a comment
placed between a line and its line-initial `|>` continuation breaks the pipeline -- measured at
the identical position, one file at a time, 241 hard diagnostics and a parse against 3114 with
`expected expression, found PipeArrow`.

Measured (dispatch artifacts, regenerated mirrors, not the .dag -- build.rs does not regenerate
from .dag and the tests read the checked-in mirror):
  RED  <see PR body: two-closure gate per module by identity, and the collision rows>
  A/B  <see PR body>
MSG
echo "commit message staged at /tmp/repair_commit_msg.txt (RED/A/B lines filled from the artifacts)"
echo "READY: review WILL_COMMIT list, then run:"
echo "  git add -A && git commit -F /tmp/repair_commit_msg.txt"
