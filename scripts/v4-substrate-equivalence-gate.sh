#!/usr/bin/env bash
# Consolidation #4553 C9 substrate-equivalence gate.
#
# Runs witness_substrate_equivalence via `gunbc run --claim-run`.
# `--perturb-check` rewrites substrate_equivalence_holds to return false and
# requires the witness to fail, so every green has a red-under-perturb receipt.

set -euo pipefail

root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$root"

bin="${V2_COMPILER:-target/release/gunbc}"
model="src/v4/test/claim/workflow/unified_test_claim_substrate_equivalence.dag"
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

run_witness() {
  local source_root="$1"
  "$bin" run \
    --source-root "$source_root" \
    --entry "$model" \
    --function witness_substrate_equivalence \
    --claim-run
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

echo "::group::substrate equivalence: witness_substrate_equivalence"
run_witness "src/v4"
echo "::endgroup::"

if [[ "$perturb" -eq 1 ]]; then
  tmp="$(mktemp -d)"
  trap 'rm -rf "$tmp"' EXIT
  mkdir -p "$tmp"
  cp -a src/v4 "$tmp/src"
  perturbed_model="$tmp/src/test/claim/workflow/unified_test_claim_substrate_equivalence.dag"
  perturb_function_to_false "$perturbed_model" substrate_equivalence_holds
  echo "::group::substrate equivalence perturb: witness_substrate_equivalence"
  if run_witness "$tmp/src"; then
    echo "::error::perturbed witness_substrate_equivalence still passed"
    exit 1
  fi
  echo "::endgroup::"
fi

echo "::notice title=substrate equivalence::witness_substrate_equivalence passed"
