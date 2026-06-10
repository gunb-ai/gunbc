#!/usr/bin/env bash
# Consolidation #4553 C9 substrate-equivalence gate.
#
# 1. Projects distributed BoolWitnessClaim markers (fail-closed on empty).
# 2. Runs witness_substrate_equivalence (modality / marker law).
# 3. --perturb-check: empty-discovery guard must fail; witness perturb must fail.

set -euo pipefail

root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$root"

bin="${V2_COMPILER:-target/release/gunbc}"
model="src/v4/test/claim/workflow/unified_test_claim_substrate_equivalence.dag"
law_model="src/v4/test/claim/workflow/glob_discovery.dag"
project_sh="$root/scripts/v4-glob-discovery-project.sh"
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

# shellcheck source=scripts/v4-glob-discovery-project.sh
source "$project_sh"

dag_string_data() {
  local file="$1" name="$2"
  grep -E "^data ${name}: String = \"" "$root/$file" \
    | sed -n "s/^data ${name}: String = \"\\(.*\\)\"/\\1/p" \
    | head -1
}

run_witness() {
  local source_root="$1" entry="$2" function="$3"
  "$bin" run \
    --source-root "$source_root" \
    --entry "$entry" \
    --function "$function" \
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

run_discovery_projection_or_fail() {
  local claims_root="${1:-}"
  if [[ -n "$claims_root" ]]; then
    v4_glob_discovery_project_distributed_markers "$claims_root"
  else
    v4_glob_discovery_project_distributed_markers
  fi
  local expected_count
  expected_count="$(dag_string_data "$law_model" glob_discovered_smoke_marker_count)"
  if [[ -z "$expected_count" ]]; then
    echo "error: missing glob_discovered_smoke_marker_count in $law_model" >&2
    exit 2
  fi
  if [[ "$V4_GLOB_DISCOVERY_ROW_COUNT" -ne "$expected_count" ]]; then
    echo "error: discovery projected ${V4_GLOB_DISCOVERY_ROW_COUNT} rows; modeled count is ${expected_count}" >&2
    exit 2
  fi
}

echo "::group::substrate equivalence: distributed marker projection"
run_discovery_projection_or_fail
echo "::notice title=substrate equivalence::discovered ${V4_GLOB_DISCOVERY_ROW_COUNT} distributed BoolWitnessClaim marker(s)"
echo "::endgroup::"

echo "::group::substrate equivalence: witness_substrate_equivalence"
run_witness "src/v4" "$model" witness_substrate_equivalence
echo "::endgroup::"

if [[ "$perturb" -eq 1 ]]; then
  echo "::group::substrate equivalence perturb: empty discovery projection"
  if v4_glob_discovery_project_distributed_markers "$root/src/v4/test/claim/impossible_bug" 2>/dev/null; then
    echo "::error::empty-discovery perturb still projected ${V4_GLOB_DISCOVERY_ROW_COUNT} row(s)"
    exit 1
  fi
  echo "::endgroup::"

  tmp="$(mktemp -d)"
  trap 'rm -rf "$tmp"' EXIT
  mkdir -p "$tmp"
  cp -a src/v4 "$tmp/src"
  perturbed_entry="$tmp/src/${model#src/v4/}"
  perturb_data_witness_to_false "$perturbed_entry" witness_substrate_equivalence
  echo "::group::substrate equivalence perturb: witness_substrate_equivalence"
  if run_witness "$tmp/src" "$perturbed_entry" witness_substrate_equivalence; then
    echo "::error::perturbed witness_substrate_equivalence still passed"
    exit 1
  fi
  echo "::endgroup::"
fi

echo "::notice title=substrate equivalence::witness_substrate_equivalence passed"
