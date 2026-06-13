#!/usr/bin/env bash
# Must-pass affected-set node-frontier selection CI gate (affected-set-3a).
#
# Each row is a Bool witness run through `gunbc run --claim-run`. `--perturb-check`
# rewrites the wired witness body to `false` in a temp source-root and requires
# the same row to fail, so every wired green has a red-under-perturb receipt.
#
# CI splits green vs perturb: `v4_lens_ci` runs `--green-only`; the 15-row perturb
# fan-out (9 node-frontier + 6 testgen) runs in matrix job `v4_lens_ci_perturb`
# via `--perturb-check --shard 0|1|2|3|4`. Full `--perturb-check` (no --shard) is
# local-only across all five shards.

set -euo pipefail

root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$root"

bin="${V2_COMPILER:-target/release/gunbc}"
# Batch witness runner: resolves one shared --entry closure once and runs many
# witnesses in a single process. Used for the GREEN pass only; the perturb pass
# stays per-row through `$bin` (each row mutates a different function).
bin_batch="${CLAIM_BATCH:-target/release/claim_batch}"
perturb=0
green_only=0
perturb_shard=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --green-only)
      green_only=1
      shift
      ;;
    --perturb-check)
      perturb=1
      shift
      ;;
    --shard)
      perturb_shard="$2"
      shift 2
      ;;
    "")
      shift
      ;;
    *)
      echo "usage: $0 [--green-only | --perturb-check [--shard 0|1|2|3|4]]" >&2
      exit 2
      ;;
  esac
done

if [[ "$green_only" -eq 1 && "$perturb" -eq 1 ]]; then
  echo "error: --green-only and --perturb-check are mutually exclusive" >&2
  exit 2
fi

if [[ -n "$perturb_shard" ]]; then
  perturb=1
  if [[ ! "$perturb_shard" =~ ^[0-4]$ ]]; then
    echo "error: --shard must be 0|1|2|3|4 (got ${perturb_shard})" >&2
    exit 2
  fi
fi

if [[ ! -x "$bin" ]]; then
  echo "error: gunbc (v2 stage0 binary) not found at $bin" >&2
  exit 2
fi

if [[ "$green_only" -eq 1 || ( "$perturb" -eq 0 && -z "$perturb_shard" ) ]]; then
  if [[ ! -x "$bin_batch" ]]; then
    echo "error: claim_batch binary not found at $bin_batch (build with: cargo build -p v2-compiler --release --bin claim_batch)" >&2
    exit 2
  fi
fi

gate_model="src/v4/test/claim/workflow/affected_set_ci_runner.dag"
affected_testgen_gate_model="src/v4/test/claim/workflow/affected_testgen_ci_runner.dag"

# Per-phase wall-time notices (same pattern as v4-substrate-equivalence-gate.sh):
# the CI latency attack (2026-06-13) needs this job's wall broken into its green
# vs perturb phases, visible in the job summary. claim_batch's own
# [resolve]/[witness]/[resolve-summary] lines give the per-witness breakdown
# within each green phase. Helper takes the phase label + start SECONDS.
phase_notice() {
  local label="$1" started="$2"
  echo "::notice title=gate timing::${label} took $((SECONDS - started))s"
}

dag_string_data() {
  local model="$1"
  local name="$2"
  grep -E "^data ${name}: String = \"" "$root/$model" \
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

list_perturb_shard_members() {
  local shard="$1"
  local list_name
  if [[ "$shard" -le 2 ]]; then
    list_name="ci_runner_perturb_shard_${shard}_rows"
    awk -v list="$list_name" '
      $0 ~ "^data " list ": " { in_list = 1; next }
      in_list && /^\]/ { in_list = 0 }
      in_list && /^  ci_runner_gate_row_/ {
        gsub(/^  /, "")
        gsub(/,.*/, "")
        print
      }
    ' "$root/$gate_model"
  else
    list_name="affected_testgen_perturb_shard_${shard}_rows"
    awk -v list="$list_name" '
      $0 ~ "^data " list ": " { in_list = 1; next }
      in_list && /^\]/ { in_list = 0 }
      in_list && /^  affected_testgen_gate_row_/ {
        gsub(/^  /, "")
        gsub(/,.*/, "")
        print
      }
    ' "$root/$affected_testgen_gate_model"
  fi
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
    echo "::group::${title} (batch green): ${e}"
    "$bin_batch" --source-root src/v4 --entry "$e" --functions "$fns" --claim-run
    echo "::endgroup::"
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

perturb_rows() {
  local shard_label="$1"
  shift
  local rows=("$@")
  local row_count=0
  local row label entry function
  for row in "${rows[@]}"; do
    IFS=$'\t' read -r label entry function <<< "$row"
    tmp="$(mktemp -d)"
    trap 'rm -rf "$tmp"' EXIT
    mkdir -p "$tmp"
    cp -a src/v4 "$tmp/src"
    perturbed_entry="$tmp/src/${entry#src/v4/}"
    perturb_function_to_false "$perturbed_entry" "$function"
    echo "::group::${shard_label} perturb: ${label}" >&2
    if run_row "$tmp/src" "$perturbed_entry" "$function"; then
      echo "::error::perturbed witness still passed: ${label}" >&2
      exit 1
    fi
    echo "::endgroup::" >&2
    rm -rf "$tmp"
    trap - EXIT
    row_count=$((row_count + 1))
  done
  echo "$row_count"
}

collect_node_frontier_rows() {
  local -n _out=$1
  local member row
  _out=()
  while IFS= read -r member; do
    [[ -z "$member" ]] && continue
    row="$(project_list_member_row "$member")"
    if [[ -z "$row" ]]; then
      echo "error: list member $member missing AffectedSetNodeFrontierClaimRunRow binding in $gate_model" >&2
      exit 2
    fi
    _out+=("$row")
  done
}

collect_testgen_rows() {
  local -n _out=$1
  local member row
  _out=()
  while IFS= read -r member; do
    [[ -z "$member" ]] && continue
    row="$(project_affected_testgen_row "$member")"
    if [[ -z "$row" ]]; then
      echo "error: list member $member missing AffectedTestgenClaimRunRow binding in $affected_testgen_gate_model" >&2
      exit 2
    fi
    _out+=("$row")
  done
}

run_perturb_shard() {
  local shard="$1"
  local count_data model project_fn shard_label
  local -a shard_rows=()
  local member row expected_count row_count

  if [[ "$shard" -le 2 ]]; then
    model="$gate_model"
    count_data="ci_runner_perturb_shard_${shard}_row_count"
    project_fn=project_list_member_row
    shard_label="affected-set node-frontier shard ${shard}"
  else
    model="$affected_testgen_gate_model"
    count_data="affected_testgen_perturb_shard_${shard}_row_count"
    project_fn=project_affected_testgen_row
    shard_label="affected-testgen shard ${shard}"
  fi

  expected_count="$(dag_string_data "$model" "$count_data")"
  if [[ -z "$expected_count" ]]; then
    echo "error: missing ${count_data} in $model" >&2
    exit 2
  fi

  while IFS= read -r member; do
    [[ -z "$member" ]] && continue
    row="$($project_fn "$member")"
    if [[ -z "$row" ]]; then
      echo "error: shard ${shard} member $member missing row binding in $model" >&2
      exit 2
    fi
    shard_rows+=("$row")
  done < <(list_perturb_shard_members "$shard")

  if [[ "${#shard_rows[@]}" -eq 0 ]]; then
    echo "error: perturb shard ${shard} has no members" >&2
    exit 2
  fi

  perturb_started=$SECONDS
  row_count="$(perturb_rows "$shard_label" "${shard_rows[@]}")"
  if [[ "$row_count" -ne "$expected_count" ]]; then
    echo "error: perturb shard ${shard} projected ${row_count} rows; modeled count is ${expected_count}" >&2
    exit 2
  fi

  phase_notice "${shard_label} perturb pass (${row_count} rows)" "$perturb_started"
  echo "::notice title=${shard_label}::${row_count} perturb witness(es) passed"
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

if [[ "$perturb" -eq 1 && -n "$perturb_shard" ]]; then
  run_perturb_shard "$perturb_shard"
  exit 0
fi

if [[ "$perturb" -eq 1 ]]; then
  expected_shard_count="$(dag_string_data "$gate_model" ci_runner_perturb_shard_count)"
  if [[ -z "$expected_shard_count" ]]; then
    echo "error: missing ci_runner_perturb_shard_count in $gate_model" >&2
    exit 2
  fi
  for shard in $(seq 0 $((expected_shard_count - 1))); do
    run_perturb_shard "$shard"
  done
  exit 0
fi

expected_count="$(dag_string_data "$gate_model" ci_runner_node_frontier_claim_run_row_count)"
if [[ -z "$expected_count" ]]; then
  echo "error: missing ci_runner_node_frontier_claim_run_row_count in $gate_model" >&2
  exit 2
fi

node_frontier_rows=()
collect_node_frontier_rows node_frontier_rows < <(list_claim_run_row_members)

if [[ "${#node_frontier_rows[@]}" -eq 0 ]]; then
  echo "error: ci_runner_node_frontier_claim_run_rows has no members in $gate_model" >&2
  exit 2
fi

nf_green_started=$SECONDS
printf '%s\n' "${node_frontier_rows[@]}" | cut -f2,3 \
  | batch_green_pass "affected-set node-frontier"
phase_notice "node-frontier green pass" "$nf_green_started"

row_count="${#node_frontier_rows[@]}"
if [[ "$row_count" -ne "$expected_count" ]]; then
  echo "error: node-frontier gate projected ${row_count} rows; modeled count is ${expected_count}" >&2
  exit 2
fi

echo "::notice title=affected-set node-frontier::${row_count} discriminating witness(es) passed"

print_affected_testgen_real_diff_evidence

expected_affected_testgen_count="$(dag_string_data "$affected_testgen_gate_model" affected_testgen_claim_run_row_count)"
if [[ -z "$expected_affected_testgen_count" ]]; then
  echo "error: missing affected_testgen_claim_run_row_count in $affected_testgen_gate_model" >&2
  exit 2
fi

affected_testgen_rows=()
collect_testgen_rows affected_testgen_rows < <(list_affected_testgen_row_members)

if [[ "${#affected_testgen_rows[@]}" -eq 0 ]]; then
  echo "error: affected_testgen_claim_run_rows has no members in $affected_testgen_gate_model" >&2
  exit 2
fi

atg_green_started=$SECONDS
printf '%s\n' "${affected_testgen_rows[@]}" | cut -f2,3 \
  | batch_green_pass "affected-testgen"
phase_notice "affected-testgen green pass" "$atg_green_started"

affected_testgen_count="${#affected_testgen_rows[@]}"
if [[ "$affected_testgen_count" -ne "$expected_affected_testgen_count" ]]; then
  echo "error: affected-testgen gate projected ${affected_testgen_count} rows; modeled count is ${expected_affected_testgen_count}" >&2
  exit 2
fi

echo "::notice title=affected-testgen::${affected_testgen_count} discriminating witness(es) passed"
