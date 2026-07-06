#!/usr/bin/env bash
# Lever-a local proof path (operator ruling 2026-07-06): zero CI runner burn.
# Runs fast disposition/refusal receipts; GATE 1b/1c scoped-compile controls are manual
# (inject break inside vs outside disposition entry_paths, nightly whole-tree is catcher).
set -euo pipefail
ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$ROOT"

echo "== lever-a GATE 1d: refusal + scoping activation (Rust, seconds) =="
cargo test -p v1-compiler --lib scoped_plan_refuses compile_clean_scoping -- --nocapture

echo "== lever-a GATE 1a disposition witnesses (gunbc, pure .dag) =="
if [[ -x "$ROOT/target/release/gunbc" ]]; then
  GUNBC="$ROOT/target/release/gunbc"
else
  GUNBC=(cargo run -q -p v1-compiler --bin gunbc --)
fi
for fn in \
  witness_empty_touched_skips \
  witness_dot_slash_docs_only_touch_skips \
  witness_std_logic_touch_scopes \
  witness_rust_only_touch_requires_whole_tree \
  witness_ci_yaml_touch_requires_whole_tree; do
  echo "-- $fn"
  "${GUNBC[@]}" run --source-root dag --source-root src/v2 \
    --entry dag/tools/dag_compile_clean_scope.dag --function "$fn"
done

cat <<'NOTE'

GATE 1b/1c (manual, local/ctrl-build):
  1. Replay diff D; record disposition entry_paths E from compile_clean_scope_disposition_from_touched.
  2. Inject a type error into a module in closure(E) -> scoped compile over E must RED.
  3. Inject the same class of error outside closure(E) -> scoped compile over E must GREEN
     (nightly whole-tree backstop #6299 is the declared catcher).

GATE 2 cost table: replay ≥5 merged PR diffs locally; cite whole-tree baseline from runs
  28780252806 (~77m plan-resolve + batch-1 in-flight at 135m kill) and 28748751115
  (batch-1 resolve leg alone ~90m). Do NOT re-burn a whole-tree CI run.

Core-query soundness: cite #6274 (entry_affected_by_touched_paths ≡ Rust frontier, 25 entries).
NOTE
