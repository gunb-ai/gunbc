#!/usr/bin/env bash
# scripts/v4-discover-owned-data.sh
#
# Host transport for Consolidation #4553 resolved-type glob discovery.
# Invokes discover_owned_data to emit the ephemeral never-committed manifest consumed
# by modeled witnesses in glob_discovery.dag.
#
# Default scan excludes live in discover_owned_data (single authority); do not duplicate
# --exclude-subpath flags here. Also writes ${manifest}.transport.tsv in the same scan.

set -euo pipefail

root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$root"

discover_bin="${DISCOVER_OWNED_DATA:-target/release/discover_owned_data}"
manifest="${V4_DISCOVERED_OWNED_DATA_MANIFEST:-target/v4-discovered-owned-data-manifest.dag}"
transport_tsv="${manifest}.transport.tsv"
source_root="${V4_DISCOVERY_SOURCE_ROOT:-src/v4}"
scan_dir="${V4_DISCOVERY_SCAN_DIR:-src/v4/test/claim}"

if [[ ! -x "$discover_bin" ]]; then
  echo "error: discover_owned_data binary not found at $discover_bin" >&2
  exit 2
fi

# --max-resolves is the discovery latency ratchet: batched discovery merges
# entry closures into collision-free merged compiles; a new decl-name collision
# that forces an extra split fails here loudly (the binary prints the colliding
# decl + files) instead of silently re-inflating CI wall-time. Fix is to
# dissolve/rename the colliding decl, never to raise this number for latency
# headroom. Currently 2, not 1: `type OverflowDisposition` is declared in both
# src/v4/std/integer.dag and src/v4/extdeps/languages/rust.dag (unify/rename is
# a substrate modeling decision, MODELING.md M9).
"$discover_bin" \
  --source-root "$source_root" \
  --scan-dir "$scan_dir" \
  --emit-dag-manifest "$manifest" \
  --max-resolves 2 \
  --format transport-tsv >"$transport_tsv"

if git ls-files --error-unmatch "$manifest" >/dev/null 2>&1; then
  echo "error: ephemeral discovery manifest is tracked in git: $manifest" >&2
  exit 2
fi

printf '%s' "$manifest"
