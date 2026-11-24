#!/usr/bin/env bash
# Producer: sig_unresolved_fallthrough_census.tsv
# Counts bare calls to kernel method template names with labeled args where the callee
# is NEITHER locally declared NOR imported (CallableUnresolved → method bridge → no wall).
# Output: TSV (module_path, name, count) sorted by name then path.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

KERNEL_NAMES="add all any append bottom chars clamp compare complement concat contains count diff empty ends_with enumerate filter first flat_map fold get intersect is_empty join last length list_push lookup map map_contains_key map_get map_has map_insert map_keys map_merge map_values meet member mul negate one reciprocal replace reverse skip sort_by split starts_with substring take to_int to_lower top to_string to_upper trim union with zero"

> /tmp/sig_unresolved_census.tsv

for name in $KERNEL_NAMES; do
  git grep -l "^fn ${name}(" -- dag/ src/v1/ src/v2/ 2>/dev/null > /tmp/sig_census_decl.txt || true
  git grep -l "^import.*{.*[^a-zA-Z0-9_]${name}[,} ]" -- dag/ src/v1/ src/v2/ 2>/dev/null > /tmp/sig_census_importers.txt || true
  git grep -nP "(^|[^a-zA-Z0-9_.])${name}\\([a-z_]+:" -- dag/ src/v2/ 2>/dev/null \
    | grep -v "fn ${name}(" | grep -v "|> ${name}(" | grep -v ": *//" | grep -v ": *import " \
    > /tmp/sig_census_calls.txt || true
  [ -s /tmp/sig_census_calls.txt ] || continue
  cut -d: -f1 /tmp/sig_census_calls.txt | sort -u | while IFS= read -r f; do
    grep -qxF "$f" /tmp/sig_census_decl.txt 2>/dev/null && continue
    grep -qxF "$f" /tmp/sig_census_importers.txt 2>/dev/null && continue
    count=$(awk -F: -v p="$f" '$1 == p' /tmp/sig_census_calls.txt | wc -l)
    [ "$count" -gt 0 ] && printf "%s\t%s\t%s\n" "$f" "$name" "$count" >> /tmp/sig_unresolved_census.tsv
  done
done

echo "TOTAL=$(awk -F'\t' '{s+=$3}END{print s}' /tmp/sig_unresolved_census.tsv)"
echo "MODULES=$(cut -f1 /tmp/sig_unresolved_census.tsv | sort -u | wc -l)"
echo "NAMES=$(cut -f2 /tmp/sig_unresolved_census.tsv | sort -u | wc -l)"
echo "SHA256=$(sha256sum /tmp/sig_unresolved_census.tsv | cut -d' ' -f1)"
