#!/usr/bin/env bash
# Must-pass affected-set node-frontier selection CI gate (affected-set-3a).
#
# Each row is a Bool witness run through `gunbc run --claim-run`. `--perturb-check`
# rewrites the wired witness body to `false` in a temp source-root and requires
# the same row to fail, so every wired green has a red-under-perturb receipt.
#
# Modes:
#   (no arg) / --green-only   GREEN batch pass only (the must-pass witnesses; no
#                             perturb). Used by the v4_lens_ci job — the perturb
#                             fan-out lives in the parallel v4_lens_ci_perturb
#                             matrix (see below).
#   --perturb-check           GREEN + the FULL per-row perturb fan-out.
#                             Local full run (and any non-sharded caller).
#   --perturb-shard K N       PERTURB ONLY the rows of the combined
#                             node-frontier++testgen list (deterministic order)
#                             whose global index i satisfies i % N == K. GREEN
#                             passes + testgen evidence are NOT re-run here (they
#                             stay in v4_lens_ci). Fail-closed: a shard that maps
#                             to zero rows, or runs fewer rows than its slice,
#                             aborts — coverage can never silently shrink. The
#                             union of shards 0..N-1 is a complete cover, and the
#                             ci aggregator requires every matrix leg to succeed,
#                             so a skipped/cancelled shard fails the gate.

set -euo pipefail

root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$root"

bin="${V2_COMPILER:-target/release/gunbc}"
# Batch witness runner: resolves one shared --entry closure once and runs many
# witnesses in a single process. Used for the GREEN pass only; the perturb pass
# stays per-row through `$bin` (each row mutates a different function).
bin_batch="${CLAIM_BATCH:-target/release/claim_batch}"
perturb=0
# shard_n > 0 selects --perturb-shard mode; shard_k is the 0-based leg index.
shard_k=-1
shard_n=0

case "${1:-}" in
  --perturb-check)
    perturb=1
    ;;
  --green-only)
    perturb=0
    ;;
  --perturb-shard)
    shard_k="${2:-}"
    shard_n="${3:-}"
    if ! [[ "$shard_k" =~ ^[0-9]+$ && "$shard_n" =~ ^[1-9][0-9]*$ ]]; then
      echo "usage: $0 --perturb-shard <shard-index K, 0-based> <shard-count N, >=1>" >&2
      exit 2
    fi
    if (( shard_k >= shard_n )); then
      echo "error: shard index ${shard_k} out of range for shard count ${shard_n} (expect 0 <= K < N)" >&2
      exit 2
    fi
    ;;
  "")
    ;;
  *)
    echo "usage: $0 [--perturb-check | --green-only | --perturb-shard K N]" >&2
    exit 2
    ;;
esac

if [[ ! -x "$bin" ]]; then
  echo "error: gunbc (v2 stage0 binary) not found at $bin" >&2
  exit 2
fi

# claim_batch is the GREEN batch runner (batch_green_pass). The --perturb-shard
# path never calls it — perturb is per-row `gunbc run --claim-run` — so a shard
# only requires gunbc. Requiring claim_batch on a perturb leg would either force
# building a binary the leg never uses or fail the leg vacuously on a gunbc-only
# restore. This is a presence check for an uninvoked tool, NOT a coverage gate:
# fail-closed-on-skip is unaffected (the ci aggregator requires every leg to
# succeed, and each shard still asserts its non-empty slice ran in full).
if [[ "$shard_n" -eq 0 && ! -x "$bin_batch" ]]; then
  echo "error: claim_batch binary not found at $bin_batch (build with: cargo build -p v2-compiler --release --bin claim_batch)" >&2
  exit 2
fi

gate_model="src/v4/test/claim/workflow/affected_set_ci_runner.dag"
affected_testgen_gate_model="src/v4/test/claim/workflow/affected_testgen_ci_runner.dag"

# Per-phase wall-time notices (same pattern as v4-substrate-equivalence-gate.sh;
# added by #4837 for the CI latency attack). claim_batch's own
# [resolve]/[witness]/[resolve-summary] lines give the per-witness breakdown
# within each green phase. Helper takes the phase label + start SECONDS. The
# perturb-shard path uses it too, so each matrix leg emits its own wall — that
# is the on-wave per-shard receipt the perturb split is measured by.
phase_notice() {
  local label="$1" started="$2"
  echo "::notice title=gate timing::${label} took $((SECONDS - started))s"
}

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

# Perturb one row's wired witness to `false` in a fresh temp source-root and
# require the witness to flip red. Fail-closed: a perturbed witness that still
# passes aborts the gate. Identical mutation for node-frontier and testgen rows
# (both mutate a function body in their own entry file).
perturb_one_row() {
  local label="$1" entry="$2" function="$3" title="$4"
  local tmp
  tmp="$(mktemp -d)"
  mkdir -p "$tmp"
  cp -a src/v4 "$tmp/src"
  local perturbed_entry="$tmp/src/${entry#src/v4/}"
  perturb_function_to_false "$perturbed_entry" "$function"
  echo "::group::${title} perturb: ${label}"
  if run_row "$tmp/src" "$perturbed_entry" "$function"; then
    echo "::error::perturbed witness still passed: ${label}"
    rm -rf "$tmp"
    exit 1
  fi
  echo "::endgroup::"
  rm -rf "$tmp"
}

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

# ---- Collect both modeled row-sets up front. There is NO projected-vs-modeled count cross-check:
# the gate projects the row-set directly from the typed list, so a dropped row simply is not
# enrolled and its witness is not run; a parallel-ledger *_row_count datum cannot detect that and
# only drifts under concurrent merges. Retired per the CLAUDE.md ledger principle. The empty-roster
# guards below stay (a totally-empty list fails closed), and per-row pass/perturb keeps the teeth. ----
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

if [[ "$shard_n" -gt 0 ]]; then
  # ---- --perturb-shard mode (one parallel v4_lens_ci_perturb matrix leg) ----
  # Combined node-frontier (first) ++ testgen (second) in deterministic order;
  # this leg perturbs exactly the rows where global-index % shard_n == shard_k.
  combined_rows=("${node_frontier_rows[@]}" "${affected_testgen_rows[@]}")
  total="${#combined_rows[@]}"

  # Rows this shard owns under a complete mod-N partition. Zero is a
  # misconfiguration (shard count exceeds row count, or a bad index) — fail
  # closed rather than report a vacuous green.
  expected_shard=0
  for ((i = 0; i < total; i++)); do
    if (( i % shard_n == shard_k )); then
      expected_shard=$((expected_shard + 1))
    fi
  done
  if [[ "$expected_shard" -eq 0 ]]; then
    echo "::error::perturb shard ${shard_k}/${shard_n} maps to 0 of ${total} rows (shard count exceeds row count?); failing closed to avoid pass-by-omission" >&2
    exit 2
  fi

  shard_started=$SECONDS
  ran=0
  for ((i = 0; i < total; i++)); do
    (( i % shard_n == shard_k )) || continue
    IFS=$'\t' read -r label entry function <<< "${combined_rows[$i]}"
    perturb_one_row "$label" "$entry" "$function" "affected-set lens (shard ${shard_k}/${shard_n})"
    ran=$((ran + 1))
  done

  if [[ "$ran" -ne "$expected_shard" ]]; then
    echo "::error::perturb shard ${shard_k}/${shard_n} ran ${ran} rows; expected ${expected_shard} -- coverage incomplete, failing closed" >&2
    exit 1
  fi

  # On-wave per-shard receipt: this leg's wall for its ${ran} perturb rows.
  phase_notice "perturb shard ${shard_k}/${shard_n} (${ran} rows)" "$shard_started"
  echo "::notice title=affected-set lens perturb shard::shard ${shard_k}/${shard_n} verified ${ran}/${total} discriminating witness(es) red-under-perturb"
  exit 0
fi

# ---- (default / --green-only) or --perturb-check mode ----
# GREEN pass: one resolve per shared entry, all that entry's witnesses in it.
nf_green_started=$SECONDS
printf '%s\n' "${node_frontier_rows[@]}" | cut -f2,3 \
  | batch_green_pass "affected-set node-frontier"
phase_notice "node-frontier green pass" "$nf_green_started"

# PERTURB pass (full --perturb-check only): one mutated resolve per row.
if [[ "$perturb" -eq 1 ]]; then
  nf_perturb_started=$SECONDS
  for row in "${node_frontier_rows[@]}"; do
    IFS=$'\t' read -r label entry function <<< "$row"
    perturb_one_row "$label" "$entry" "$function" "affected-set node-frontier"
  done
  phase_notice "node-frontier perturb pass (${#node_frontier_rows[@]} rows)" "$nf_perturb_started"
fi

echo "::notice title=affected-set node-frontier::${#node_frontier_rows[@]} discriminating witness(es) passed"

print_affected_testgen_real_diff_evidence

# GREEN pass: one resolve per shared entry, all that entry's witnesses in it.
atg_green_started=$SECONDS
printf '%s\n' "${affected_testgen_rows[@]}" | cut -f2,3 \
  | batch_green_pass "affected-testgen"
phase_notice "affected-testgen green pass" "$atg_green_started"

# PERTURB pass (full --perturb-check only): one mutated resolve per row.
if [[ "$perturb" -eq 1 ]]; then
  atg_perturb_started=$SECONDS
  for row in "${affected_testgen_rows[@]}"; do
    IFS=$'\t' read -r label entry function <<< "$row"
    perturb_one_row "$label" "$entry" "$function" "affected-testgen"
  done
  phase_notice "affected-testgen perturb pass (${#affected_testgen_rows[@]} rows)" "$atg_perturb_started"
fi

echo "::notice title=affected-testgen::${#affected_testgen_rows[@]} discriminating witness(es) passed"
