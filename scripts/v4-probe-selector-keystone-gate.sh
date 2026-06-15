#!/usr/bin/env bash
# ProbeSelector keystone CI gate (PS-0 host-health / availability conditioning).
#
# Host authority: src/v4/test/claim/workflow/probe_selector_ci_runner.dag
# (ProbeSelectorKeystoneClaimRunRow roster). GREEN uses claim_batch (one resolve
# per entry); --perturb-check rewrites each witness body to false in a temp tree
# and requires the row to fail (red-under-perturb).
#
# Modes:
#   (no arg) / --green-only   GREEN batch pass + modeled row-count guard
#   --perturb-check           GREEN + full per-row perturb fan-out

set -euo pipefail

root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$root"

bin="${V2_COMPILER:-target/release/gunbc}"
bin_batch="${CLAIM_BATCH:-target/release/claim_batch}"
perturb=0

case "${1:-}" in
  --perturb-check) perturb=1 ;;
  --green-only) perturb=0 ;;
  "")
    ;;
  *)
    echo "usage: $0 [--perturb-check | --green-only]" >&2
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

gate_model="src/v4/test/claim/workflow/probe_selector_ci_runner.dag"

dag_string_data() {
  local name="$1"
  grep -E "^data ${name}: String = \"" "$root/$gate_model" \
    | sed -n "s/^data ${name}: String = \"\\(.*\\)\"/\\1/p" \
    | head -1
}

list_claim_run_row_members() {
  awk '
    /data probe_selector_claim_run_rows:/ { in_list = 1; next }
    in_list && /^\]/ { in_list = 0 }
    in_list && /^  probe_selector_gate_row_/ {
      gsub(/^  /, "")
      gsub(/,.*/, "")
      print
    }
  ' "$root/$gate_model"
}

project_list_member_row() {
  local name="$1"
  awk -v n="$name" '
    $0 ~ "^data " n ": ProbeSelectorKeystoneClaimRunRow" { in_row = 1; label = ""; entry = ""; fn = "" }
    in_row && /label: "/ {
      sub(/.*label: "/, "")
      sub(/".*/, "")
      label = $0
    }
    in_row && /entry: / {
      if ($0 ~ /entry: probe_selector_gate_entry/) {
        entry = "src/v4/test/claim/workflow/probe_selector_ci_runner.dag"
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

batch_green_pass() {
  local title="$1"
  local entry="" functions=""
  local _label _entry fn
  while IFS=$'\t' read -r _label _entry fn; do
    [[ -z "$_entry" ]] && continue
    if [[ -z "$entry" ]]; then
      entry="$_entry"
    elif [[ "$entry" != "$_entry" ]]; then
      echo "error: probe-selector keystone gate expects a single shared entry; got ${entry} and ${_entry}" >&2
      exit 2
    fi
    if [[ -n "$functions" ]]; then
      functions+=","
    fi
    functions+="$fn"
  done
  echo "::group::${title} (batch green): ${entry}"
  "$bin_batch" --source-root src/v4 --entry "$entry" --functions "$functions" --claim-run
  echo "::endgroup::"
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
            end = i
            break
if end is None:
    raise SystemExit(f"{path}: unterminated body for {function}")
path.write_text(text[: brace + 1] + "\n  false\n}" + text[end + 1 :], encoding="utf-8")
PY
}

perturb_one_row() {
  local label="$1" entry="$2" function="$3"
  local tmp
  tmp="$(mktemp -d)"
  trap 'rm -rf "$tmp"' EXIT
  mkdir -p "$tmp"
  cp -a src/v4 "$tmp/v4"
  local perturbed_entry="$tmp/v4/${entry#src/v4/}"
  perturb_function_to_false "$perturbed_entry" "$function"
  echo "::group::probe-selector keystone perturb: ${label}"
  if run_row "$tmp/v4" "$perturbed_entry" "$function"; then
    echo "::error::perturbed witness still passed: ${label}"
    exit 1
  fi
  echo "::endgroup::"
  rm -rf "$tmp"
  trap - EXIT
}

expected_count="$(dag_string_data probe_selector_claim_run_row_count)"
if [[ -z "$expected_count" ]]; then
  echo "error: missing probe_selector_claim_run_row_count in $gate_model" >&2
  exit 2
fi

rows=()
while IFS= read -r member; do
  [[ -z "$member" ]] && continue
  row="$(project_list_member_row "$member")"
  if [[ -z "$row" ]]; then
    echo "error: list member $member missing ProbeSelectorKeystoneClaimRunRow binding in $gate_model" >&2
    exit 2
  fi
  rows+=("$row")
done < <(list_claim_run_row_members)

if [[ "${#rows[@]}" -eq 0 ]]; then
  echo "error: probe_selector_claim_run_rows has no members in $gate_model" >&2
  exit 2
fi

if [[ "${#rows[@]}" -ne "$expected_count" ]]; then
  echo "error: probe-selector gate projected ${#rows[@]} rows; modeled count is ${expected_count}" >&2
  exit 2
fi

green_started=$SECONDS
batch_green_pass "probe-selector keystone" < <(printf '%s\n' "${rows[@]}")
echo "::notice title=gate timing::probe-selector keystone green pass took $((SECONDS - green_started))s"

if [[ "$perturb" -eq 1 ]]; then
  perturb_started=$SECONDS
  for row in "${rows[@]}"; do
    IFS=$'\t' read -r label entry function <<< "$row"
    perturb_one_row "$label" "$entry" "$function"
  done
  echo "::notice title=gate timing::probe-selector keystone perturb pass (${#rows[@]} rows) took $((SECONDS - perturb_started))s"
fi

echo "::notice title=probe-selector keystone::${#rows[@]} discriminating witness(es) passed (perturb=${perturb})"
