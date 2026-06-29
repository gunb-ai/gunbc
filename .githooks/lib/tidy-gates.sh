# SCAFFOLD — shared tidy-gate helpers for .githooks/pre-push.
# dissolve-on: emit from gunbc.local_tidy_spec + extdeps.git.hooks (model sketch: stern-moth-225).
# Gate entrypoints (single authority): doc_reachability_witness_test.dag,
# generated_artifact_gate.dag. Trigger globs below are interim hand-list; emit phase
# projects them from the same ci_spec / layer-roots roster CI reads.

tidy_gates_ensure_gunbc() {
  if [[ -n "${GUNBC_BIN:-}" ]] && [[ -x "$GUNBC_BIN" ]]; then
    return 0
  fi
  local root
  root="$(git rev-parse --show-toplevel)"
  if [[ -x "$root/target/release/gunbc" ]]; then
    GUNBC_BIN="$root/target/release/gunbc"
    return 0
  fi
  if [[ -x "$root/target/debug/gunbc" ]]; then
    GUNBC_BIN="$root/target/debug/gunbc"
    return 0
  fi
  echo "[pre-push] gunbc not built; compiling debug (one-time)..."
  (cd "$root" && CTRL_BUILD_WRAP_CARGO=0 cargo build -p v1-compiler --bin gunbc)
  GUNBC_BIN="$root/target/debug/gunbc"
}

tidy_gates_needs_doc_reachability() {
  local path
  for path in "$@"; do
    case "$path" in
      docs/*|ROADMAP.md|DESIGN.md) return 0 ;;
      *.dag|*.md) return 0 ;;
    esac
  done
  return 1
}

tidy_gates_needs_generated_artifact() {
  local path
  for path in "$@"; do
    case "$path" in
      .github/workflows/ci.yml|.gitignore|ROADMAP.md|DESIGN.md|.github/fleet-converge.sh)
        return 0
        ;;
      docs/plans/*.md) return 0 ;;
      dsl/*|src/v2/*) return 0 ;;
    esac
  done
  return 1
}

tidy_gates_run_doc_reachability() {
  echo "[pre-push] doc reachability (orphan-doc + dangling-link lens)"
  "$GUNBC_BIN" run --source-root dsl \
    --entry dsl/test/claim/doc_reachability_witness_test.dag \
    --function doc_graph_has_no_orphan_docs --claim-run
  "$GUNBC_BIN" run --source-root dsl \
    --entry dsl/test/claim/doc_reachability_witness_test.dag \
    --function doc_graph_has_no_dangling_links --claim-run
}

tidy_gates_stage_regen_diffs_only() {
  local path
  while IFS= read -r path; do
    [[ -n "$path" ]] && git add -u -- "$path"
  done < <(git diff --name-only --diff-filter=ACMR)
}

tidy_gates_run_generated_artifact() {
  local entry="dsl/tools/generated_artifact_gate.dag"
  echo "[pre-push] generated-artifact drift gate"
  if "$GUNBC_BIN" run --source-root dsl --entry "$entry" \
    --function run_generated_artifact_drift_gate_body; then
    return 0
  fi

  echo "[pre-push] generated-artifact drift detected; running main_wet regen"
  "$GUNBC_BIN" run --source-root dsl --entry "$entry" --function main_wet
  tidy_gates_stage_regen_diffs_only

  echo "[pre-push] re-checking generated-artifact drift after regen"
  "$GUNBC_BIN" run --source-root dsl --entry "$entry" \
    --function run_generated_artifact_drift_gate_body
}

# Run tidy gates when any changed file in the push range can affect them.
# Args: one path per line on stdin (push-range changed files).
tidy_gates_run_if_needed() {
  local -a changed=()
  local line
  while IFS= read -r line; do
    [[ -n "$line" ]] && changed+=("$line")
  done

  if [[ ${#changed[@]} -eq 0 ]]; then
    return 0
  fi

  local doc_gate=0 artifact_gate=0
  if tidy_gates_needs_doc_reachability "${changed[@]}"; then doc_gate=1; fi
  if tidy_gates_needs_generated_artifact "${changed[@]}"; then artifact_gate=1; fi
  if [[ "$doc_gate" == "0" && "$artifact_gate" == "0" ]]; then
    return 0
  fi

  tidy_gates_ensure_gunbc
  if [[ "$doc_gate" == "1" ]]; then
    tidy_gates_run_doc_reachability
  fi
  if [[ "$artifact_gate" == "1" ]]; then
    tidy_gates_run_generated_artifact
  fi
}
