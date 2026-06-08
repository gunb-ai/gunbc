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

# Discriminating lens witnesses (host roster; formerly workflow/ci.dag lens_ci_claim_run_rows).
LENS_CI_ROWS=(
  "lens_cost/atom_zero|src/v4/test/claim/lens_cost/atom_zero.dag|atom_zero_claim_holds"
  "lens_synthesis/polynomial_dominates_linear|src/v4/test/claim/lens_synthesis/polynomial_dominates_linear.dag|polynomial_dominates_linear_claim_holds"
  "lens_coverage/hollow_type_defect_key|src/v4/test/claim/lens_coverage/hollow_type_defect_key.dag|hollow_type_defect_key_claim_holds"
  "lens_structural_resolution/binds_to_resolved|src/v4/test/claim/lens_structural_resolution/binds_to_resolved.dag|binds_to_resolved_claim_holds"
)

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

row_count=0
for row in "${LENS_CI_ROWS[@]}"; do
  IFS='|' read -r label entry function <<< "$row"
  echo "::group::v4 lens CI: ${label}"
  run_row "src/v4" "$entry" "$function"
  echo "::endgroup::"

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

if [[ "$row_count" -eq 0 ]]; then
  echo "error: lens CI roster is empty" >&2
  exit 2
fi

echo "::notice title=v4 lens CI::${row_count} discriminating lens witness(es) passed"
