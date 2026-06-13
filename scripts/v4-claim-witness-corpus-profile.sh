#!/usr/bin/env bash
# Profile claim-witness corpus CI gate (shards a|b): per-witness dag-eval wall
# with the same overhead lens as sibling CI profilers (lens / floor / corpus).
#
# Decomposes each ExpectPass witness into:
#   cold_run_s     — standalone `gunbc run --claim-run` (resolve + eval, full subprocess)
#   green_batch_s  — amortized share of `claim_batch` for the witness's entry group
#   spot_perturb_s — temp-copy + perturb + cold run (CI spot-perturb path)
#   overhead_s     — cold_run_s minus spot_perturb_s (subprocess resolve delta vs
#                    perturb path; negative when temp-copy dominates)
#   batch_savings_s — cold_run_s minus green_batch_s (claim_batch amortization win;
#                    empty when --skip-cold)
#
# ExpectFail rows get cold_run_s only (no batch green / spot perturb).
#
# Usage:
#   scripts/v4-claim-witness-corpus-profile.sh --shard a|b [--out DIR] [--skip-cold]
#   scripts/v4-claim-witness-corpus-profile.sh --shard a|b --both   # run a then b
#   --skip-cold: skip per-witness standalone gunbc runs (green batch + spot only;
#                use for full-shard coverage when cold subprocess tax is sampled separately)
#
# Requires: target/release/gunbc, target/release/claim_batch
set -euo pipefail

root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$root"

bin="${V2_COMPILER:-target/release/gunbc}"
bin_batch="${CLAIM_BATCH:-target/release/claim_batch}"
shard=""
out_dir=""
run_both=0
skip_cold=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --shard)
      shard="$2"
      shift 2
      ;;
    --both)
      run_both=1
      shift
      ;;
    --out)
      out_dir="$2"
      shift 2
      ;;
    --skip-cold)
      skip_cold=1
      shift
      ;;
    "")
      shift
      ;;
    *)
      echo "usage: $0 --shard a|b [--out DIR] [--skip-cold] | --both [--out DIR] [--skip-cold]" >&2
      exit 2
      ;;
  esac
done

if [[ ! -x "$bin" ]]; then
  echo "error: gunbc not found at $bin (cargo build -p v2-compiler --release --bin gunbc)" >&2
  exit 2
fi
if [[ ! -x "$bin_batch" ]]; then
  echo "error: claim_batch not found at $bin_batch" >&2
  exit 2
fi

gate_model="src/v4/test/claim/workflow/claim_witness_corpus_ci_runner.dag"

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

wall_secs() {
  local start="$1"
  local end
  end=$(date +%s)
  echo $((end - start))
}

run_row() {
  local source_root="$1" entry="$2" function="$3"
  "$bin" run --source-root "$source_root" --entry "$entry" --function "$function" --claim-run >/dev/null 2>&1
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

time_spot_perturb() {
  local row="$1"
  IFS=$'\t' read -r _label entry function _expect _bind <<< "$row"
  local tmp start end
  tmp="$(mktemp -d)"
  start=$(date +%s)
  mkdir -p "$tmp"
  cp -a src/v4 "$tmp/src"
  local perturbed_entry="$tmp/src/${entry#src/v4/}"
  perturb_function_to_false "$perturbed_entry" "$function"
  run_row "$tmp/src" "$perturbed_entry" "$function" || true
  rm -rf "$tmp"
  end=$(date +%s)
  echo $((end - start))
}

profile_shard() {
  local s="$1"
  local rows_data="claim_witness_corpus_shard_${s}_rows"
  local count_data="claim_witness_corpus_shard_${s}_row_count"
  local tsv tsv_final

  local -a all_rows=()
  local -a pass_rows=()
  local -a fail_rows=()
  local member row
  while IFS= read -r member; do
    [[ -z "$member" ]] && continue
    row="$(project_list_member_row "$member")"
    if [[ -z "$row" ]]; then
      echo "error: list member $member missing row in $gate_model" >&2
      exit 2
    fi
    all_rows+=("$row")
    IFS=$'\t' read -r _l _e _f expect _b <<< "$row"
    if [[ "$expect" == pass ]]; then
      pass_rows+=("$row")
    else
      fail_rows+=("$row")
    fi
  done < <(list_claim_run_row_members "$rows_data")

  local expected_count
  expected_count="$(dag_string_data "$count_data")"
  if [[ -n "$out_dir" ]]; then
    mkdir -p "$out_dir"
  fi
  if [[ "${#all_rows[@]}" -ne "$expected_count" ]]; then
    echo "error: shard ${s}: projected ${#all_rows[@]} rows; modeled count is ${expected_count}" >&2
    exit 2
  fi

  # GREEN batch: time per entry group, record amortized seconds per witness.
  declare -A entry_batch_total=()
  declare -A entry_fn_count=()
  declare -A witness_green_amort=()

  local -A entry_fns=()
  local -a entry_order=()
  local entry function e fns start end batch_s n_fns amort
  for row in "${pass_rows[@]}"; do
    IFS=$'\t' read -r _label entry function _expect _bind <<< "$row"
    if [[ -z "${entry_fns[$entry]+x}" ]]; then
      entry_order+=("$entry")
    fi
    entry_fns[$entry]+="${function},"
    entry_fn_count[$entry]=$((${entry_fn_count[$entry]:-0} + 1))
  done

  for e in "${entry_order[@]}"; do
    fns="${entry_fns[$e]%,}"
    start=$(date +%s)
    "$bin_batch" --source-root src/v4 --entry "$e" --functions "$fns" --claim-run >/dev/null
    end=$(date +%s)
    batch_s=$((end - start))
    entry_batch_total[$e]=$batch_s
    n_fns=${entry_fn_count[$e]}
    amort=$((batch_s / n_fns))
    # remainder to last witness in entry (deterministic)
    local remainder=$((batch_s % n_fns))
    local idx=0
    for row in "${pass_rows[@]}"; do
      IFS=$'\t' read -r label r_entry r_fn _expect _bind <<< "$row"
      [[ "$r_entry" != "$e" ]] && continue
      local share=$amort
      idx=$((idx + 1))
      if [[ "$idx" -eq "$n_fns" ]]; then
        share=$((share + remainder))
      fi
      witness_green_amort["${label}"]=$share
    done
  done

  local tsv_final
  if [[ -n "$out_dir" ]]; then
    tsv_final="$out_dir/corpus_shard_${s}_profile.tsv"
    tsv="$(mktemp "${tsv_final}.XXXXXX")"
  else
    tsv_final=/dev/stdout
    tsv=/dev/stdout
  fi

  printf 'shard\tlabel\tentry\tfunction\texpect\tcold_run_s\tgreen_batch_s\tspot_perturb_s\toverhead_s\tbatch_savings_s\n' >"$tsv"

  local total_cold=0 total_green=0 total_spot=0 total_overhead=0 total_fail_cold=0 total_batch_savings=0
  local label r_entry r_fn expect cold_s green_s spot_s overhead_s batch_savings_s

  for row in "${pass_rows[@]}"; do
    IFS=$'\t' read -r label r_entry r_fn expect _bind <<< "$row"
    if [[ "$skip_cold" -eq 1 ]]; then
      cold_s=""
      batch_savings_s=""
    else
      start=$(date +%s)
      run_row "src/v4" "$r_entry" "$r_fn"
      cold_s=$(wall_secs "$start")
      total_cold=$((total_cold + cold_s))
    fi
    green_s=${witness_green_amort[$label]:-0}
    spot_s=$(time_spot_perturb "$row")
    if [[ -n "$cold_s" ]]; then
      overhead_s=$((cold_s - spot_s))
      batch_savings_s=$((cold_s - green_s))
      total_overhead=$((total_overhead + overhead_s))
      total_batch_savings=$((total_batch_savings + batch_savings_s))
    else
      overhead_s=""
      batch_savings_s=""
    fi
    printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' \
      "$s" "$label" "$r_entry" "$r_fn" "$expect" \
      "${cold_s:--}" "$green_s" "$spot_s" "${overhead_s:--}" "${batch_savings_s:--}" >>"$tsv"
    total_green=$((total_green + green_s))
    total_spot=$((total_spot + spot_s))
    echo "::notice title=corpus profile ${s}::${label} cold=${cold_s:-skip} green_amort=${green_s}s spot=${spot_s}s overhead=${overhead_s:-skip} batch_savings=${batch_savings_s:-skip}" >&2
  done

  for row in "${fail_rows[@]}"; do
    IFS=$'\t' read -r label r_entry r_fn expect _bind <<< "$row"
    start=$(date +%s)
    run_row "src/v4" "$r_entry" "$r_fn" || true
    cold_s=$(wall_secs "$start")
    printf '%s\t%s\t%s\t%s\t%s\t%s\t0\t0\t0\t0\n' \
      "$s" "$label" "$r_entry" "$r_fn" "$expect" "$cold_s" >>"$tsv"
    total_cold=$((total_cold + cold_s))
    total_fail_cold=$((total_fail_cold + cold_s))
    echo "::notice title=corpus profile ${s}::${label} (ExpectFail) cold=${cold_s}s" >&2
  done

  local green_batch_wall=0
  for e in "${entry_order[@]}"; do
    green_batch_wall=$((green_batch_wall + entry_batch_total[$e]))
  done

  echo "::notice title=corpus profile ${s} summary::rows=${#all_rows[@]} pass=${#pass_rows[@]} fail=${#fail_rows[@]} green_batch_wall=${green_batch_wall}s total_cold=${total_cold}s total_green_amort=${total_green}s total_spot=${total_spot}s total_overhead=${total_overhead}s total_batch_savings=${total_batch_savings}s skip_cold=${skip_cold}" >&2

  if [[ -n "$out_dir" ]]; then
    mv -f "$tsv" "$tsv_final"
    tsv="$tsv_final"
  fi

  if [[ -n "$out_dir" ]]; then
    {
      echo "# claim-witness corpus shard ${s} profile"
      echo "generated: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
      echo "skip_cold: ${skip_cold}"
      echo "rows: ${#all_rows[@]} (${#pass_rows[@]} ExpectPass, ${#fail_rows[@]} ExpectFail)"
      echo "green_batch_wall_s: ${green_batch_wall}"
      echo "sum_cold_run_s: ${total_cold}"
      echo "sum_green_amort_s: ${total_green}"
      echo "sum_spot_perturb_s: ${total_spot}"
      echo "sum_overhead_s: ${total_overhead}"
      echo "sum_batch_savings_s: ${total_batch_savings}"
      echo "entry_batch_wall_s:"
      for e in "${entry_order[@]}"; do
        echo "  ${e}: ${entry_batch_total[$e]}s (${entry_fn_count[$e]} witness(es))"
      done
      echo "ci_spot_perturb_estimate_s: $(( ${#pass_rows[@]} > 0 ? (total_spot * 2 / ${#pass_rows[@]}) : 0 )) (2/${#pass_rows[@]} of full spot sum)"
      echo "ci_phase_a_estimate_s: $((green_batch_wall + total_fail_cold)) (green batch + ExpectFail cold runs)"
    } >"$out_dir/corpus_shard_${s}_summary.txt"
  fi
}

if [[ "$run_both" -eq 1 ]]; then
  profile_shard a
  profile_shard b
elif [[ "$shard" =~ ^[ab]$ ]]; then
  profile_shard "$shard"
else
  echo "error: --shard a|b or --both required" >&2
  exit 2
fi
