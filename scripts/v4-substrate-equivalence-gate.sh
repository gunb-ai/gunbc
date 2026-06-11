#!/usr/bin/env bash
# Consolidation #4553 C9 substrate-equivalence gate.
#
# Post-delete standing gates (no committed full-corpus transport golden):
# 1. Resolved-type owned-data discovery: nonempty + complete transport projection.
# 2. Hermetic discover fixture slice (regression guard, not hand-bumped corpus oracle).
# 3. Runs witness_substrate_equivalence (modality / sample-marker law).
# 4. --perturb-check: empty-discovery guard must fail; type-head perturb flips membership;
#    witness_substrate_equivalence perturb must fail.

set -euo pipefail

root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$root"

bin="${V2_COMPILER:-target/release/gunbc}"
discover_bin="${DISCOVER_OWNED_DATA:-target/release/discover_owned_data}"
discover_sh="$root/scripts/v4-discover-owned-data.sh"
model="src/v4/test/claim/workflow/unified_test_claim_substrate_equivalence.dag"
law_model="src/v4/test/claim/workflow/glob_discovery.dag"
manifest=""
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

if [[ ! -x "$discover_bin" ]]; then
  echo "error: discover_owned_data binary not found at $discover_bin" >&2
  exit 2
fi

manifest="$("$discover_sh")"

run_witness() {
  local source_root="$1" entry="$2" function="$3"
  "$bin" run \
    --source-root "$source_root" \
    --source-root "$(dirname "$manifest")" \
    --entry "$entry" \
    --function "$function" \
    --claim-run
}

run_law_witness_with_manifest() {
  local manifest_path="$1"
  local function="$2"
  "$bin" run \
    --source-root src/v4 \
    --source-root "$(dirname "$manifest_path")" \
    --entry "$law_model" \
    --function "$function" \
    --claim-run
}

perturb_type_head_bool_witness_to_node_corpus() {
  local file="$1" decl="$2"
  python3 - "$file" "$decl" <<'PY'
from pathlib import Path
import re
import sys

path = Path(sys.argv[1])
decl = sys.argv[2]
text = path.read_text(encoding="utf-8")
pattern = re.compile(
    rf"^data {re.escape(decl)}: UnifiedTestClaim = BoolWitnessClaim \{{\n"
    r"  witness: BoolWitness \{\n"
    r'    entry: "[^"]+",\n'
    r"    function: [^\n]+\n"
    r"  \}\n"
    r"\}",
    re.MULTILINE,
)
replacement = (
    f"data {decl}: UnifiedTestClaim = NodeCorpus {{\n"
    f"  claim: edit_locus_narrow_resolution_claim_passes,\n"
    f"  transport: EvalOnly\n"
    f"}}"
)
if not pattern.search(text):
    raise SystemExit(f"{path}: missing BoolWitnessClaim marker {decl} for type-head perturb")
if "EvalOnly" not in text:
    text = text.replace(
        "import v4.std.verification {\n  BoolWitness,\n  BoolWitnessClaim,\n  UnifiedTestClaim\n}",
        "import v4.std.verification {\n  BoolWitness,\n  BoolWitnessClaim,\n  EvalOnly,\n  NodeCorpus,\n  UnifiedTestClaim\n}",
    )
text = pattern.sub(replacement, text, count=1)
path.write_text(text, encoding="utf-8")
PY
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

run_discovery_nonempty_or_fail() {
  run_witness "src/v4" "$law_model" witness_glob_discovery_smoke_set_is_nonempty
  run_witness "src/v4" "$law_model" witness_discovered_bool_witness_claim_count_is_positive
  run_witness "src/v4" "$law_model" witness_glob_discovery_bool_witness_transport_is_complete
  run_witness "src/v4" "$law_model" witness_discovered_bool_witness_transport_projection_is_complete
}

verify_discovery_completeness_fixture() {
  local fixture_dir="$root/scripts/fixtures/v4_discovery_completeness_slice"
  local expected_count=5
  local actual_count

  if [[ ! -d "$fixture_dir" ]]; then
    echo "error: missing hermetic discovery fixture: $fixture_dir" >&2
    exit 2
  fi

  actual_count="$("$discover_bin" \
    --source-root src/v4 \
    --scan-dir "$fixture_dir" \
    --format transport-tsv \
    | sed '/^$/d' \
    | wc -l \
    | tr -d ' ')"

  if [[ "$actual_count" -ne "$expected_count" ]]; then
    echo "error: hermetic discovery fixture projected ${actual_count} row(s); expected ${expected_count}" >&2
    exit 1
  fi
}

discovered_transport_count() {
  local transport_tsv="${manifest}.transport.tsv"
  if [[ ! -s "$transport_tsv" ]]; then
    echo "error: missing discovery transport sidecar: $transport_tsv (re-run v4-discover-owned-data.sh)" >&2
    exit 2
  fi
  sed '/^$/d' "$transport_tsv" | wc -l | tr -d ' '
}

echo "::group::substrate equivalence: resolved-type owned-data discovery"
run_discovery_nonempty_or_fail
transport_count="$(discovered_transport_count)"
echo "::notice title=substrate equivalence::discovered ${transport_count} BoolWitnessClaim transport row(s) with complete projection (no committed corpus golden)"
echo "::endgroup::"

echo "::group::substrate equivalence: hermetic discovery completeness fixture"
verify_discovery_completeness_fixture
echo "::notice title=substrate equivalence::hermetic fixture transport row count matches by execution"
echo "::endgroup::"

echo "::group::substrate equivalence: witness_substrate_equivalence"
run_witness "src/v4" "$model" witness_substrate_equivalence
echo "::endgroup::"

if [[ "$perturb" -eq 1 ]]; then
  echo "::group::substrate equivalence perturb: empty discovery projection"
  tmp_empty="$(mktemp -d)"
  mkdir -p "$tmp_empty/empty_slice"
  empty_manifest="$tmp_empty/v4-discovered-owned-data-manifest.dag"
  "$discover_bin" \
    --source-root src/v4 \
    --scan-dir "$tmp_empty/empty_slice" \
    --emit-dag-manifest "$empty_manifest" \
    --format json >/dev/null
  if "$discover_bin" \
    --source-root src/v4 \
    --scan-dir "$tmp_empty/empty_slice" \
    --format transport-tsv 2>/dev/null | grep -q .; then
    echo "::error::empty-discovery perturb still projected transport rows"
    exit 1
  fi
  if "$bin" run \
    --source-root src/v4 \
    --source-root "$tmp_empty" \
    --entry "$law_model" \
    --function witness_glob_discovery_smoke_set_is_nonempty \
    --claim-run; then
    echo "::error::empty-discovery perturb: modeled nonempty witness still passed"
    exit 1
  fi
  echo "::endgroup::"

  echo "::group::substrate equivalence perturb: type-head flip excludes unified-claim arm"
  tmp_type_head="$(mktemp -d)"
  fixture_dir="$root/scripts/fixtures/v4_discovery_completeness_slice"
  fixture_copy="$tmp_type_head/slice"
  mkdir -p "$fixture_copy"
  cp -a "$fixture_dir/." "$fixture_copy/"
  fixture_entry="$fixture_copy/edit_locus_resolver.dag"
  type_head_manifest="$tmp_type_head/v4-discovered-owned-data-manifest.dag"
  baseline_transport="$("$discover_bin" \
    --source-root src/v4 \
    --scan-dir "$fixture_copy" \
    --format transport-tsv \
    | sed '/^$/d' \
    | wc -l \
    | tr -d ' ')"
  "$discover_bin" \
    --source-root src/v4 \
    --scan-dir "$fixture_copy" \
    --emit-dag-manifest "$type_head_manifest" \
    --format json >/dev/null
  if ! run_law_witness_with_manifest "$type_head_manifest" witness_discovery_type_head_perturb_fixture_decl_is_bool_witness_arm; then
    echo "::error::type-head perturb baseline: fixture decl not discovered as BoolWitnessClaim arm"
    exit 1
  fi
  perturb_type_head_bool_witness_to_node_corpus "$fixture_entry" unified_claim_edit_locus_narrow
  perturbed_transport="$("$discover_bin" \
    --source-root src/v4 \
    --scan-dir "$fixture_copy" \
    --format transport-tsv \
    | sed '/^$/d' \
    | wc -l \
    | tr -d ' ')"
  if [[ "$perturbed_transport" -ge "$baseline_transport" ]]; then
    echo "::error::type-head perturb did not drop BoolWitnessClaim transport rows (${baseline_transport} -> ${perturbed_transport})"
    exit 1
  fi
  "$discover_bin" \
    --source-root src/v4 \
    --scan-dir "$fixture_copy" \
    --emit-dag-manifest "$type_head_manifest" \
    --format json >/dev/null
  if run_law_witness_with_manifest "$type_head_manifest" witness_discovery_type_head_perturb_fixture_decl_is_bool_witness_arm; then
    echo "::error::type-head perturb: modeled membership witness still passed after NodeCorpus flip"
    exit 1
  fi
  echo "::endgroup::"

  tmp="$(mktemp -d)"
  trap 'rm -rf "$tmp" "$tmp_empty" "$tmp_type_head"' EXIT
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
