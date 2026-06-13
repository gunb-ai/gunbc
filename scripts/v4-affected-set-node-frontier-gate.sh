#!/usr/bin/env bash
# Must-pass affected-set node-frontier selection CI gate (affected-set-3a).
#
# Each row is a Bool witness run through `gunbc run --claim-run`. `--perturb-check`
# rewrites the wired witness body to `false` in a temp source-root and requires
# the same row to fail, so every wired green has a red-under-perturb receipt.

set -euo pipefail

root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$root"

bin="${V2_COMPILER:-target/release/gunbc}"
# Batch witness runner: resolves one shared --entry closure once and runs many
# witnesses in a single process. Used for the GREEN pass only; the perturb pass
# stays per-row through `$bin` (each row mutates a different function).
bin_batch="${CLAIM_BATCH:-target/release/claim_batch}"
perturb=0

case "${1:-}" in
  --perturb-check)
    perturb=1
    ;;
  "")
    ;;
  *)
    echo "usage: $0 [--perturb-check]" >&2
    exit 2
    ;;
esac

if [[ ! -x "$bin" ]]; then
  echo "error: gunbc (v2 stage0 binary) not found at $bin" >&2
  exit 2
fi

if [[ ! -x "$bin_batch" ]]; then
  echo "error: claim_batch binary not found at $bin_batch (build with: cargo build -p v2-compiler --release --bin claim_batch)" >&2
  exit 2
fi

# Per-phase wall-time notices: v4_lens_ci cost is dominated by per-row perturb
# resolves; without explicit durations the job summary hides whether a regression
# is in green-batch, node-frontier perturb, or affected-testgen perturb.
phase_name=""
phase_started=0
phase_begin() {
  phase_name="$1"
  phase_started=$SECONDS
  echo "::group::${phase_name}"
}
phase_end() {
  echo "::endgroup::"
  echo "::notice title=gate timing::${phase_name} took $((SECONDS - phase_started))s"
}

gate_started=$SECONDS

gate_model="src/v4/test/claim/workflow/affected_set_ci_runner.dag"
affected_testgen_gate_model="src/v4/test/claim/workflow/affected_testgen_ci_runner.dag"

dag_string_data() {
  local name="$1"
  grep -E "^data ${name}: String = \"" "$root/$gate_model" \
    | sed -n "s/^data ${name}: String = \"\\(.*\\)\"/\\1/p" \
    | head -1
}

list_claim_run_row_members() {
  awk '
    /data ci_runner_node_frontier_claim_run_rows:/ { in_list = 1; next }
    in_list && /^\]/ { in_list = 0 }
    in_list && /^  ci_runner_gate_row_/ {
      gsub(/^  /, "")
      gsub(/,.*/, "")
      print
    }
  ' "$root/$gate_model"
}

project_list_member_row() {
  local name="$1"
  awk -v n="$name" '
    $0 ~ "^data " n ": AffectedSetNodeFrontierClaimRunRow" { in_row = 1; label = ""; entry = ""; fn = "" }
    in_row && /label: "/ {
      sub(/.*label: "/, "")
      sub(/".*/, "")
      label = $0
    }
    in_row && /entry: / {
      if ($0 ~ /entry: ci_runner_gate_entry/) {
        entry = "src/v4/test/claim/workflow/affected_set_ci_runner.dag"
      } else if ($0 ~ /entry: "/) {
        sub(/.*entry: "/, "")
        sub(/".*/, "")
        entry = $0
      }
    }
    in_row && /function: "/ {
      sub(/.*function: "/, "")
      sub(/".*/, "")
      fn = $0
    }
    in_row && /\}/ {
      if (label != "" && entry != "" && fn != "") {
        print label "\t" entry "\t" fn
      }
      in_row = 0
    }
  ' "$root/$gate_model"
}

run_row() {
  local source_root="$1" entry="$2" function="$3"
  "$bin" run --source-root "$source_root" --entry "$entry" --function "$function" --claim-run
}

# GREEN pass for a stream of "<entry>\t<function>" lines on stdin: group the
# functions by their shared --entry and resolve each entry's import closure ONCE
# (via claim_batch), running all of that entry's witnesses in a single process.
# This collapses the N-rows-share-one-entry green pass from N full-tree resolves
# to one. Fail-closed: a non-zero claim_batch exit (any witness red) aborts.
batch_green_pass() {
  local title="$1"
  local -A entry_fns=()
  local -a entry_order=()
  local entry function
  while IFS=$'\t' read -r entry function; do
    [[ -z "$entry" ]] && continue
    if [[ -z "${entry_fns[$entry]+x}" ]]; then
      entry_order+=("$entry")
    fi
    entry_fns[$entry]+="${function},"
  done
  local e fns
  for e in "${entry_order[@]}"; do
    fns="${entry_fns[$e]%,}"
    phase_begin "${title} (batch green): ${e}"
    "$bin_batch" --source-root src/v4 --entry "$e" --functions "$fns" --claim-run
    phase_end
  done
}

perturb_function_to_false() {
  local file="$1" function="$2"
  python3 - "$file" "$function" <<'PY'
from pathlib import Path
import sys

path = Path(sys.argv[1])
function = sys.argv[2]
text = path.read_text(encoding="utf-8")
needle = f"fn {function}("
start = text.find(needle)
if start < 0:
    raise SystemExit(f"{path}: missing function {function}")
brace = text.find("{", start)
if brace < 0:
    raise SystemExit(f"{path}: missing body for {function}")
depth = 0
end = None
for i in range(brace, len(text)):
    ch = text[i]
    if ch == "{":
        depth += 1
    elif ch == "}":
        depth -= 1
        if depth == 0:
            end = i + 1
            break
if end is None:
    raise SystemExit(f"{path}: unterminated body for {function}")
path.write_text(text[:brace] + "{\n  false\n}" + text[end:], encoding="utf-8")
PY
}

expected_count="$(dag_string_data ci_runner_node_frontier_claim_run_row_count)"
if [[ -z "$expected_count" ]]; then
  echo "error: missing ci_runner_node_frontier_claim_run_row_count in $gate_model" >&2
  exit 2
fi

# Collect every row first so the GREEN pass can resolve each shared entry once.
node_frontier_rows=()
while IFS= read -r member; do
  [[ -z "$member" ]] && continue
  row="$(project_list_member_row "$member")"
  if [[ -z "$row" ]]; then
    echo "error: list member $member missing AffectedSetNodeFrontierClaimRunRow binding in $gate_model" >&2
    exit 2
  fi
  node_frontier_rows+=("$row")
done < <(list_claim_run_row_members)

if [[ "${#node_frontier_rows[@]}" -eq 0 ]]; then
  echo "error: ci_runner_node_frontier_claim_run_rows has no members in $gate_model" >&2
  exit 2
fi

# GREEN pass: one resolve per shared entry, all that entry's witnesses in it.
printf '%s\n' "${node_frontier_rows[@]}" | cut -f2,3 \
  | batch_green_pass "affected-set node-frontier"

# PERTURB pass + count: one mutated resolve per row (each mutates a different fn).
row_count=0
for row in "${node_frontier_rows[@]}"; do
  IFS=$'\t' read -r label entry function <<< "$row"

  if [[ "$perturb" -eq 1 ]]; then
    tmp="$(mktemp -d)"
    trap 'rm -rf "$tmp"' EXIT
    mkdir -p "$tmp"
    cp -a src/v4 "$tmp/src"
    perturbed_entry="$tmp/src/${entry#src/v4/}"
    perturb_function_to_false "$perturbed_entry" "$function"
    phase_begin "affected-set node-frontier perturb: ${label}"
    if run_row "$tmp/src" "$perturbed_entry" "$function"; then
      echo "::error::perturbed witness still passed: ${label}"
      exit 1
    fi
    phase_end
    rm -rf "$tmp"
    trap - EXIT
  fi
  row_count=$((row_count + 1))
done

if [[ "$row_count" -ne "$expected_count" ]]; then
  echo "error: node-frontier gate projected ${row_count} rows; modeled count is ${expected_count}" >&2
  exit 2
fi

echo "::notice title=affected-set node-frontier::${row_count} discriminating witness(es) passed"

affected_testgen_dag_string_data() {
  local name="$1"
  grep -E "^data ${name}: String = \"" "$root/$affected_testgen_gate_model" \
    | sed -n "s/^data ${name}: String = \"\\(.*\\)\"/\\1/p" \
    | head -1
}

list_affected_testgen_row_members() {
  awk '
    /data affected_testgen_claim_run_rows:/ { in_list = 1; next }
    in_list && /^\]/ { in_list = 0 }
    in_list && /^  affected_testgen_gate_row_/ {
      gsub(/^  /, "")
      gsub(/,.*/, "")
      print
    }
  ' "$root/$affected_testgen_gate_model"
}

project_affected_testgen_row() {
  local name="$1"
  awk -v n="$name" '
    $0 ~ "^data " n ": AffectedTestgenClaimRunRow" { in_row = 1; label = ""; entry = ""; fn = "" }
    in_row && /label: "/ {
      sub(/.*label: "/, "")
      sub(/".*/, "")
      label = $0
    }
    in_row && /entry: / {
      if ($0 ~ /entry: affected_testgen_gate_entry/) {
        entry = "src/v4/test/claim/workflow/affected_testgen_ci_runner.dag"
      } else if ($0 ~ /entry: "/) {
        sub(/.*entry: "/, "")
        sub(/".*/, "")
        entry = $0
      }
    }
    in_row && /function: "/ {
      sub(/.*function: "/, "")
      sub(/".*/, "")
      fn = $0
    }
    in_row && /\}/ {
      if (label != "" && entry != "" && fn != "") {
        print label "\t" entry "\t" fn
      }
      in_row = 0
    }
  ' "$root/$affected_testgen_gate_model"
}

print_affected_testgen_real_diff_evidence() {
  local base_ref="${AFFECTED_TESTGEN_BASE_REF:-origin/${GITHUB_BASE_REF:-main}}"
  local diff_base=""
  local diff_label=""

  if [[ -n "${GITHUB_BASE_REF:-}" ]]; then
    git fetch --no-tags --depth=200 origin \
      "+refs/heads/${GITHUB_BASE_REF}:refs/remotes/origin/${GITHUB_BASE_REF}" >/dev/null 2>&1 || true
    base_ref="refs/remotes/origin/${GITHUB_BASE_REF}"
  fi

  if [[ -n "${GITHUB_REF:-}" ]]; then
    git fetch --no-tags --depth=200 origin \
      "+${GITHUB_REF}:refs/remotes/origin/affected-testgen-current" >/dev/null 2>&1 || true
  fi

  if git rev-parse --verify "$base_ref" >/dev/null 2>&1; then
    if diff_base="$(git merge-base "$base_ref" HEAD 2>/dev/null)"; then
      diff_label="${base_ref}...HEAD"
    else
      diff_base="$base_ref"
      diff_label="${base_ref}..HEAD"
      echo "::notice title=affected-testgen evidence::base ref ${base_ref} has no merge base in checkout; using direct ${diff_label}"
    fi
  fi

  if [[ -z "$diff_base" ]]; then
    if git rev-parse --verify "HEAD^1" >/dev/null 2>&1; then
      diff_base="HEAD^1"
      diff_label="HEAD^1..HEAD"
      echo "::notice title=affected-testgen evidence::base ref ${base_ref} has no merge base; using ${diff_label}"
    else
      echo "::notice title=affected-testgen evidence::no usable base ref; skipping real diff evidence"
      return 0
    fi
  fi

  echo "::group::affected-testgen evidence: real PR diff files"
  echo "diff-range: ${diff_label}"
  git diff --name-only "$diff_base" HEAD -- \
    scripts/v4-affected-set-node-frontier-gate.sh \
    src/v4/test/claim/workflow/affected_testgen_ci_runner.dag \
    .github/workflows/ci.yml
  echo "::endgroup::"

  echo "::group::affected-testgen evidence: real PR changed lines"
  echo "diff-range: ${diff_label}"
  git diff --unified=0 "$diff_base" HEAD -- \
    scripts/v4-affected-set-node-frontier-gate.sh \
    src/v4/test/claim/workflow/affected_testgen_ci_runner.dag \
    .github/workflows/ci.yml \
    | sed -n '1,220p'
  echo "::endgroup::"
}

print_affected_testgen_real_diff_evidence

expected_affected_testgen_count="$(affected_testgen_dag_string_data affected_testgen_claim_run_row_count)"
if [[ -z "$expected_affected_testgen_count" ]]; then
  echo "error: missing affected_testgen_claim_run_row_count in $affected_testgen_gate_model" >&2
  exit 2
fi

# Collect every row first so the GREEN pass can resolve each shared entry once.
affected_testgen_rows=()
while IFS= read -r member; do
  [[ -z "$member" ]] && continue
  row="$(project_affected_testgen_row "$member")"
  if [[ -z "$row" ]]; then
    echo "error: list member $member missing AffectedTestgenClaimRunRow binding in $affected_testgen_gate_model" >&2
    exit 2
  fi
  affected_testgen_rows+=("$row")
done < <(list_affected_testgen_row_members)

if [[ "${#affected_testgen_rows[@]}" -eq 0 ]]; then
  echo "error: affected_testgen_claim_run_rows has no members in $affected_testgen_gate_model" >&2
  exit 2
fi

# GREEN pass: one resolve per shared entry, all that entry's witnesses in it.
printf '%s\n' "${affected_testgen_rows[@]}" | cut -f2,3 \
  | batch_green_pass "affected-testgen"

# PERTURB pass + count: one mutated resolve per row (each mutates a different fn).
affected_testgen_count=0
for row in "${affected_testgen_rows[@]}"; do
  IFS=$'\t' read -r label entry function <<< "$row"

  if [[ "$perturb" -eq 1 ]]; then
    tmp="$(mktemp -d)"
    trap 'rm -rf "$tmp"' EXIT
    mkdir -p "$tmp"
    cp -a src/v4 "$tmp/src"
    perturbed_entry="$tmp/src/${entry#src/v4/}"
    perturb_function_to_false "$perturbed_entry" "$function"
    phase_begin "affected-testgen perturb: ${label}"
    if run_row "$tmp/src" "$perturbed_entry" "$function"; then
      echo "::error::perturbed witness still passed: ${label}"
      exit 1
    fi
    phase_end
    rm -rf "$tmp"
    trap - EXIT
  fi
  affected_testgen_count=$((affected_testgen_count + 1))
done

if [[ "$affected_testgen_count" -ne "$expected_affected_testgen_count" ]]; then
  echo "error: affected-testgen gate projected ${affected_testgen_count} rows; modeled count is ${expected_affected_testgen_count}" >&2
  exit 2
fi

echo "::notice title=affected-testgen::${affected_testgen_count} discriminating witness(es) passed"
echo "::notice title=gate timing::node-frontier gate total took $((SECONDS - gate_started))s"
