#!/usr/bin/env bash
# Consolidation #4553 C9 substrate-equivalence gate.
#
# Runs witness_substrate_equivalence via `gunbc run --claim-run`.
# `--perturb-check` rewrites witness_substrate_equivalence data to `false` and
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
  local source_root="$1" entry="$2"
  "$bin" run \
    --source-root "$source_root" \
    --entry "$entry" \
    --function witness_substrate_equivalence \
    --claim-run
}

perturb_data_witness_to_false() {
  local file="$1" witness="$2"
  python3 - "$file" "$witness" <<'PY'
from pathlib import Path
import re
import sys

path = Path(sys.argv[1])
witness = sys.argv[2]
text = path.read_text(encoding="utf-8")
pattern = re.compile(
    rf"^data {re.escape(witness)}: Bool = .*$",
    re.MULTILINE,
)
if not pattern.search(text):
    raise SystemExit(f"{path}: missing data witness {witness}")
text = pattern.sub(f"data {witness}: Bool = false", text, count=1)
path.write_text(text, encoding="utf-8")
PY
}

echo "::group::substrate equivalence: witness_substrate_equivalence"
run_witness "src/v4" "$model"
echo "::endgroup::"

if [[ "$perturb" -eq 1 ]]; then
  tmp="$(mktemp -d)"
  trap 'rm -rf "$tmp"' EXIT
  mkdir -p "$tmp"
  cp -a src/v4 "$tmp/src"
  perturbed_entry="$tmp/src/${model#src/v4/}"
  perturb_data_witness_to_false "$perturbed_entry" witness_substrate_equivalence
  echo "::group::substrate equivalence perturb: witness_substrate_equivalence"
  if run_witness "$tmp/src" "$perturbed_entry"; then
    echo "::error::perturbed witness_substrate_equivalence still passed"
    exit 1
  fi
  echo "::endgroup::"
fi

echo "::notice title=substrate equivalence::witness_substrate_equivalence passed"
