#!/usr/bin/env bash
# Claim-witness corpus CI gate — enrollment over claim_witness_corpus_ci_runner.dag.
#
# Reuses the lens CI machinery: claim_batch for ExpectPass rows (one resolve per shared
# entry), gunbc --claim-run for ExpectFail baseline rows.
# CI uses --shard a|b --spot-perturb-check (1–2 rotating ExpectPass rows keyed on
# GITHUB_RUN_NUMBER); full --perturb-check remains for local audit (run both shards).
#
# Sign runner receipt (2026-06-13, CARGO_BUILD_JOBS=2): monolith 20-row
# --spot-perturb-check measured 1108s (~18.5m) — exceeds 13m uncontended gate.
# Sharded ~10-row jobs are the recorded recovery path (~10m/shard ×2 ≤ 20m ceiling).
# Fails closed on:
#   ExpectPass + actual false  → regression
#   ExpectFail + actual true   → stale manifest (flip row to ExpectPass in same PR)
#   ExpectFail + non-false     → infra/compile/runtime failure

set -euo pipefail

root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$root"

bin="${V2_COMPILER:-target/release/gunbc}"
bin_batch="${CLAIM_BATCH:-target/release/claim_batch}"
perturb_mode="none"
shard=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --spot-perturb-check)
      perturb_mode="spot"
      shift
      ;;
    --perturb-check)
      perturb_mode="full"
      shift
      ;;
    --shard)
      shard="$2"
      shift 2
      ;;
    "")
      shift
      ;;
    *)
      echo "usage: $0 --shard a|b [--spot-perturb-check | --perturb-check]" >&2
      exit 2
      ;;
  esac
done

if [[ ! "$shard" =~ ^[ab]$ ]]; then
  echo "error: --shard a|b is required" >&2
  exit 2
fi

if [[ ! -x "$bin" ]]; then
  echo "error: gunbc (v2 stage0 binary) not found at $bin" >&2
  exit 2
fi

if [[ ! -x "$bin_batch" ]]; then
  echo "error: claim_batch binary not found at $bin_batch (build with: cargo build -p v2-compiler --release --bin claim_batch)" >&2
  exit 2
fi

gate_model="src/v4/test/claim/workflow/claim_witness_corpus_ci_runner.dag"
rows_data="claim_witness_corpus_shard_${shard}_rows"

dag_string_data() {
  local name="$1"
  grep -E "^data ${name}: String = \"" "$root/$gate_model" \
    | sed -n "s/^data ${name}: String = \"\\(.*\\)\"/\\1/p" \
    | head -1
}

list_claim_run_row_members() {
  local list_name="$1"
  awk -v list="$list_name" '
    $0 ~ "^data " list ": " { in_list = 1; next }
    in_list && /^\]/ { in_list = 0 }
    in_list && /^  corpus_ci_gate_row_/ {
      gsub(/^  /, "")
      gsub(/,.*/, "")
      print
    }
  ' "$root/$gate_model"
}

project_list_member_row() {
  local name="$1"
  awk -v n="$name" '
    $0 ~ "^data " n ": ClaimWitnessCorpusClaimRunRow" { in_row = 1; label = ""; entry = ""; fn = ""; expect = ""; bind = "" }
    in_row && /label: "/ {
      sub(/.*label: "/, "")
      sub(/".*/, "")
      label = $0
    }
    in_row && /entry: "/ {
      sub(/.*entry: "/, "")
      sub(/".*/, "")
      entry = $0
    }
    in_row && /function: "/ {
      sub(/.*function: "/, "")
      sub(/".*/, "")
      fn = $0
    }
    in_row && /expected: ExpectPass/ {
      expect = "pass"
    }
    in_row && /expected: ExpectFail/ {
      expect = "fail"
    }
    in_row && /bind_anchor: "/ {
      sub(/.*bind_anchor: "/, "")
      sub(/".*/, "")
      bind = $0
    }
    in_row && /\}/ {
      if (label != "" && entry != "" && fn != "" && expect != "") {
        print label "\t" entry "\t" fn "\t" expect "\t" bind
      }
      in_row = 0
    }
  ' "$root/$gate_model"
}

run_row() {
  local source_root="$1" entry="$2" function="$3"
  "$bin" run --source-root "$source_root" --entry "$entry" --function "$function" --claim-run
}

# Derives the --source-root for an entry path by matching its prefix.
# Fails hard on an unrecognized prefix so new roots must be added explicitly.
derive_source_root() {
  local entry="$1"
  case "$entry" in
    src/v4/*) echo "src/v4" ;;
    dsl/*)    echo "dsl" ;;
    *)
      echo "error: cannot derive source-root from entry: ${entry}" >&2
      exit 2
      ;;
  esac
}

classify_stdout() {
  local out="$1"
  if printf '%s\n' "$out" | grep -qx 'true'; then
    echo true
    return 0
  fi
  if printf '%s\n' "$out" | grep -qx 'false'; then
    echo false
    return 0
  fi
  echo error
}

run_resolve_fail_repro_if_bound() {
  local bind_anchor="$1"
  # Extension point: an ExpectFail row may bind a dual-root resolve-fail repro here
  # (keyed on its bind_anchor) to prove the expected failure is genuinely a resolve
  # failure, not a hardcoded false. None are currently bound -- P-PROBE-CF-IMPORT
  # (adhoc-20b17ff7-932) resolved and retired to a genuine ExpectPass corpus row.
  case "$bind_anchor" in
    *) ;;
  esac
}

# GREEN pass: group functions by (source-root, entry), then one claim_batch call per
# unique source-root so the module index is built once per root.  The prior version
# hardcoded --source-root src/v4; this version derives the root from each entry's path
# prefix (see derive_source_root), enabling dsl/-rooted entries alongside src/v4 ones.
batch_green_pass() {
  local title="$1"
  local -A entry_fns=()
  local -A entry_root=()
  local -A root_entries=()
  local -a root_order=()
  local -A root_seen=()
  local entry function source_root

  while IFS=$'\t' read -r entry function; do
    [[ -z "$entry" ]] && continue
    source_root="$(derive_source_root "$entry")"
    if [[ -z "${entry_fns[$entry]+x}" ]]; then
      entry_root[$entry]="$source_root"
      if [[ -z "${root_seen[$source_root]+x}" ]]; then
        root_seen[$source_root]=1
        root_order+=("$source_root")
        root_entries[$source_root]=""
      fi
      root_entries[$source_root]+="${entry}"$'\n'
    fi
    entry_fns[$entry]+="${function},"
  done

  local e fns args
  for source_root in "${root_order[@]}"; do
    args=(--source-root "$source_root")
    while IFS= read -r e; do
      [[ -z "$e" ]] && continue
      fns="${entry_fns[$e]%,}"
      args+=(--entry "$e" --functions "$fns")
    done <<< "${root_entries[$source_root]}"
    echo "::group::${title} (batch green, root=${source_root})"
    "$bin_batch" "${args[@]}" --claim-run
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

perturb_one_row() {
  local row="$1"
  IFS=$'\t' read -r label entry function _expect _bind <<< "$row"
  local source_root
  source_root="$(derive_source_root "$entry")"
  local tmp
  tmp="$(mktemp -d)"
  trap 'rm -rf "$tmp"' EXIT
  mkdir -p "$tmp"
  cp -a "$source_root" "$tmp/root"
  local perturbed_entry="$tmp/root/${entry#${source_root}/}"
  perturb_function_to_false "$perturbed_entry" "$function"
  echo "::group::claim witness corpus perturb: ${label}"
  if run_row "$tmp/root" "$perturbed_entry" "$function"; then
    echo "::error::perturbed witness still passed: ${label}"
    exit 1
  fi
  echo "::endgroup::"
  rm -rf "$tmp"
  trap - EXIT
}

pick_spot_perturb_indices() {
  local n="$1"
  local -n _out=$2
  _out=()
  if [[ "$n" -eq 0 ]]; then
    return 0
  fi
  # Stable rotation: GITHUB_RUN_NUMBER cycles which ExpectPass rows get spot-perturb.
  local seed="${GITHUB_RUN_NUMBER:-1}"
  local i0=$(( seed % n ))
  local i1=$(( (seed + 1) % n ))
  _out=("$i0")
  if [[ "$n" -gt 1 && "$i1" -ne "$i0" ]]; then
    _out+=("$i1")
  fi
}

all_rows=()
pass_rows=()
fail_rows=()
while IFS= read -r member; do
  [[ -z "$member" ]] && continue
  row="$(project_list_member_row "$member")"
  if [[ -z "$row" ]]; then
    echo "error: list member $member missing ClaimWitnessCorpusClaimRunRow binding in $gate_model" >&2
    exit 2
  fi
  all_rows+=("$row")
  IFS=$'\t' read -r _label _entry _function expect bind_anchor <<< "$row"
  if [[ "$expect" == pass ]]; then
    pass_rows+=("$row")
  elif [[ "$expect" == fail ]]; then
    if [[ -z "$bind_anchor" ]]; then
      echo "error: ExpectFail row $member missing bind_anchor in $gate_model" >&2
      exit 2
    fi
    fail_rows+=("$row")
  else
    echo "error: row $member has unknown expectation arm" >&2
    exit 2
  fi
done < <(list_claim_run_row_members "$rows_data")

if [[ "${#all_rows[@]}" -eq 0 ]]; then
  echo "error: ${rows_data} has no members in $gate_model" >&2
  exit 2
fi

if [[ "${#pass_rows[@]}" -gt 0 ]]; then
  printf '%s\n' "${pass_rows[@]}" | cut -f2,3 | batch_green_pass "claim witness corpus"
fi

failures=0
for row in "${fail_rows[@]}"; do
  IFS=$'\t' read -r label entry function _expect bind_anchor <<< "$row"
  echo "::group::claim witness corpus (ExpectFail): ${label}"
  out="$(run_row "$(derive_source_root "$entry")" "$entry" "$function" 2>&1)" || true
  echo "$out"
  actual="$(classify_stdout "$out")"
  echo "::endgroup::"
  if [[ "$actual" == false ]]; then
    run_resolve_fail_repro_if_bound "$bind_anchor"
    continue
  fi
  failures=$((failures + 1))
  if [[ "$actual" == true ]]; then
    echo "::error::claim witness ${label}: STALE MANIFEST ExpectFail got true — flip row to ExpectPass in claim_witness_corpus_ci_runner.dag in the same PR (bind: ${bind_anchor})"
  else
    echo "::error::claim witness ${label}: ExpectFail row did not print false (infra/compile/runtime failure)"
  fi
done

if [[ "$perturb_mode" == full ]]; then
  for row in "${pass_rows[@]}"; do
    perturb_one_row "$row"
  done
elif [[ "$perturb_mode" == spot ]]; then
  spot_indices=()
  pick_spot_perturb_indices "${#pass_rows[@]}" spot_indices
  for idx in "${spot_indices[@]}"; do
    perturb_one_row "${pass_rows[$idx]}"
  done
  echo "::notice title=claim witness corpus spot perturb::shard=${shard} perturbed ${#spot_indices[@]}/${#pass_rows[@]} ExpectPass row(s) (run_number=${GITHUB_RUN_NUMBER:-local})"
fi

# Row count is reported in the notice below for visibility, but is NOT cross-checked against a
# modeled *_row_count datum: the gate folds over the typed shard List, so a dropped row simply is
# not enrolled (and its witness is not run) — a parallel-ledger count datum cannot detect that and
# only drifts under concurrent merges (reddened main on #4947). Retired per the CLAUDE.md ledger
# principle; the gate keeps its teeth via the per-row pass/perturb checks below.
row_count="${#all_rows[@]}"

if [[ "$failures" -ne 0 ]]; then
  echo "error: $failures ExpectFail row(s) drifted from manifest" >&2
  exit 1
fi

echo "::notice title=claim witness corpus::shard=${shard} ${row_count} manifest row(s) matched (${#pass_rows[@]} ExpectPass, ${#fail_rows[@]} ExpectFail; perturb=${perturb_mode})"
