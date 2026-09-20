#!/usr/bin/env bash
# ENCODING-0 hex-syntax consolidation (gunbc#11782): the scoped claim invocation for this PR's head.
#
# SCOPE IS DERIVED, NOT TRANSCRIBED. Per the interim gate pinned on gunbc#11626 the scope is every
# witness module this diff touches plus every witness module that DIRECTLY imports one of the
# modules it changes. Both halves are facts the source tree already owns, so this script reads them
# at run time: an earlier revision hand-copied the resulting roster of modules and test-fn names,
# which is the projection DESIGN section 6 says the model should generate (a claim added later would
# have dropped silently out of "scope" with no refusal). review 68970.
#
# CLAIM_BATCH=echo ./this-script prints the argv without running it.
#
# SCAFFOLD, and the trigger that retires it: `gunbc test <label>` is the modeled route for invoking a
# measurement, and DESIGN's Building & checks rules that a new one is A ROW rather than a flag. This
# file dissolves when an instrument row derives a diff's claim scope and runs it -- at which point
# the scope derivation below belongs in that row's fold, not in shell. It exists meanwhile because
# the receipt gate (gunbc#11742) landed before that row, and a receipt must be runnable today.
set -euo pipefail
: "${CLAIM_BATCH:=target/release/claim_batch}"

CHANGED_MODULES=(
  std.content_hash
  extdeps.network.mac
  gunbc.auth.approval_decision_store
  extdeps.git.object_store
  extdeps.numeric.base16
)
TOUCHED_WITNESSES=(dag/test/claim/base16_rfc4648_witness_test.dag)

in_scope() {
  local file=$1 m
  for m in "${TOUCHED_WITNESSES[@]}"; do [ "$file" = "$m" ] && return 0; done
  for m in "${CHANGED_MODULES[@]}"; do
    if grep -qE "^import[[:space:]]+${m}([[:space:]]|\{|$)" "$file"; then return 0; fi
  done
  return 1
}

args=(--claim-run --hermetic --source-root dag --source-root src/v2)
modules=0
claims=0
while IFS= read -r file; do
  in_scope "$file" || continue
  fns=$(grep -oE '^test fn [a-z0-9_]+' "$file" | awk '{print $3}' | paste -sd, || true)
  [ -n "$fns" ] || continue
  args+=(--entry "$file" --functions "$fns")
  modules=$((modules + 1))
  claims=$((claims + $(printf '%s' "$fns" | tr ',' '\n' | wc -l)))
done < <(find dag/test/claim -name '*.dag' | sort)

echo "scope: ${modules} witness module(s), ${claims} claim(s)" >&2
"$CLAIM_BATCH" "${args[@]}"
