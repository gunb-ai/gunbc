#!/usr/bin/env bash
# Must-pass v4 lens-analysis CI gate.
#
# Each row is a Bool witness run through `gunbc run --claim-run`. `--perturb-check`
# rewrites the wired witness body to `false` in a temp source-root and requires
# the same row to fail, so every wired green has a red-under-perturb receipt.

set -euo pipefail

root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$root"

bin="${V2_COMPILER:-target/release/gunbc}"
# Batch witness runner: builds the module source index once and resolves each
# entry's closure in a single process. Used for the GREEN pass only; the
# perturb pass stays per-row through `$bin` (each row mutates a different fn).
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

ci_model="src/v4/workflow/lens_ci_gate.dag"

dag_string_data() {
  local name="$1"
  grep -E "^data ${name}: String = \"" "$root/$ci_model" \
    | sed -n "s/^data ${name}: String = \"\\(.*\\)\"/\\1/p" \
    | head -1
}

# List member names from `lens_ci_claim_run_rows` authority in lens_ci_gate.dag.
list_claim_run_row_members() {
  awk '
    /data lens_ci_claim_run_rows:/ { in_list = 1; next }
    in_list && /^\]/ { in_list = 0 }
    in_list && /^  lens_ci_claim_run_row_/ {
      gsub(/^  /, "")
      gsub(/,.*/, "")
      print
    }
  ' "$root/$ci_model"
}

# Project one list member binding:
# `data <name>: LensCiClaimRunRow = LensCiClaimRunRow { ... }`.
project_list_member_row() {
  local name="$1"
  awk -v n="$name" '
    $0 ~ "^data " n ": LensCiClaimRunRow" { in_row = 1; label = ""; entry = ""; fn = "" }
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
    in_row && /\}/ {
      if (label != "" && entry != "" && fn != "") {
        print label "\t" entry "\t" fn
      }
      in_row = 0
    }
  ' "$root/$ci_model"
}

run_row() {
  local source_root="$1" entry="$2" function="$3"
  "$bin" run --source-root "$source_root" --entry "$entry" --function "$function" --claim-run
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

expected_count="$(dag_string_data lens_ci_claim_run_row_count)"
if [[ -z "$expected_count" ]]; then
  echo "error: missing lens_ci_claim_run_row_count in $ci_model" >&2
  exit 2
fi

# Collect all rows before running so the GREEN pass can issue one claim_batch
# call instead of N separate gunbc-run invocations.
lens_rows=()
while IFS= read -r member; do
  [[ -z "$member" ]] && continue
  row="$(project_list_member_row "$member")"
  if [[ -z "$row" ]]; then
    echo "error: list member $member missing LensCiClaimRunRow binding in $ci_model" >&2
    exit 2
  fi
  lens_rows+=("$row")
done < <(list_claim_run_row_members)

if [[ "${#lens_rows[@]}" -eq 0 ]]; then
  echo "error: lens_ci_claim_run_rows has no members in $ci_model" >&2
  exit 2
fi

# GREEN pass: one claim_batch call with all entry/function pairs. The module
# source index is built once; each entry's closure is resolved in the same
# process instead of spawning a new gunbc process per row.
green_args=(--source-root src/v4)
for row in "${lens_rows[@]}"; do
  IFS=$'\t' read -r label entry function <<< "$row"
  green_args+=(--entry "$entry" --function "$function")
done
echo "::group::v4 lens CI green pass (batch)"
"$bin_batch" "${green_args[@]}" --claim-run
echo "::endgroup::"

# PERTURB pass + count: one mutated resolve per row (each mutates a different fn).
row_count=0
for row in "${lens_rows[@]}"; do
  IFS=$'\t' read -r label entry function <<< "$row"

  if [[ "$perturb" -eq 1 ]]; then
    tmp="$(mktemp -d)"
    trap 'rm -rf "$tmp"' EXIT
    mkdir -p "$tmp"
    cp -a src/v4 "$tmp/src"
    perturbed_entry="$tmp/src/${entry#src/v4/}"
    perturb_function_to_false "$perturbed_entry" "$function"
    echo "::group::v4 lens CI perturb: ${label}"
    if run_row "$tmp/src" "$perturbed_entry" "$function"; then
      echo "::error::perturbed witness still passed: ${label}"
      exit 1
    fi
    echo "::endgroup::"
    rm -rf "$tmp"
    trap - EXIT
  fi
  row_count=$((row_count + 1))
done

if [[ "$row_count" -ne "$expected_count" ]]; then
  echo "error: lens CI transport projected ${row_count} rows; modeled count is ${expected_count}" >&2
  exit 2
fi

echo "::notice title=v4 lens CI::${row_count} discriminating lens witness(es) passed"
