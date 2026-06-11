#!/usr/bin/env bash
# scripts/v4-testclaim-smoke-roster.sh
#
# Wave-A consolidated smoke-roster transport. Row authority is resolved-type owned-data
# discovery (discover_owned_data + modeled glob_discovery witnesses) — not grep/name patterns.

set -euo pipefail

root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$root"

bin="${V2_COMPILER:-target/release/gunbc}"
discover_bin="${DISCOVER_OWNED_DATA:-target/release/discover_owned_data}"
discover_sh="$root/scripts/v4-discover-owned-data.sh"
law_model="src/v4/test/claim/workflow/glob_discovery.dag"
manifest=""

if [[ ! -x "$bin" ]]; then
  echo "error: gunbc (v2 stage0 binary) not found at $bin" >&2
  exit 2
fi

if [[ ! -x "$discover_bin" ]]; then
  echo "error: discover_owned_data binary not found at $discover_bin" >&2
  exit 2
fi

manifest="$("$discover_sh")"

claim_run() {
  local label="$1" entry="$2" function="$3"
  echo "::group::v4 smoke roster: ${label}"
  "$bin" run --source-root src/v4 --entry "$entry" --function "$function" --claim-run
  echo "::endgroup::"
}

run_law_witness() {
  local function="$1"
  "$bin" run \
    --source-root src/v4 \
    --source-root "$(dirname "$manifest")" \
    --entry "$law_model" \
    --function "$function" \
    --claim-run
}

echo "::group::v4 smoke roster: glob discovery nonempty law (resolved-type)"
run_law_witness witness_glob_discovery_smoke_set_is_nonempty
run_law_witness witness_discovered_bool_witness_claim_count_is_positive
echo "::endgroup::"

transport_tsv="${manifest}.transport.tsv"
if [[ ! -s "$transport_tsv" ]]; then
  echo "error: missing discovery transport sidecar: $transport_tsv" >&2
  exit 2
fi

transport_count=0
# Stream transport rows directly — command substitution strips trailing newlines and
# `while read` then drops the final row when the buffer has no terminating newline.
while IFS=$'\t' read -r label entry function || [[ -n "${label:-}" ]]; do
  [[ -z "$label" ]] && continue
  claim_run "$label" "$entry" "$function"
  transport_count=$((transport_count + 1))
done <"$transport_tsv"

echo "::notice title=v4 smoke roster::${transport_count} claim-run witness(es) passed"
