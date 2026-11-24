#!/usr/bin/env bash
# Producer for docs/measurements/sig_unresolved_fallthrough_census.tsv
#
# Census: bare calls to KERNEL METHOD TEMPLATE names with labeled args where the callee is
# NEITHER locally declared NOR imported in the calling module. Those calls reach
# CallableUnresolved -> the method bridge -> no direct-call shape wall (the blind spot this
# row measures; see dag/test/claim/sig_unresolved_fallthrough_witness.dag).
#
# CONSUMER: this file is regenerated into the committed TSV; the witness doc comment and the
# roadmap row dag/gunbc/roadmap/roadmap_authority.dag (node call-signature-resolution-coverage,
# red_control "fallthrough count monotonically falls") consume the committed baseline.
#
# THE NAME ROSTER IS DERIVED, NOT TYPED. Kernel method template names come from the algebra
# authority itself -- dag/std/algebra.dag, the AlgebraFieldTemplate rows (`name: "<n>"`) that
# the compiler's algebra_method_template_name consults. The census can only ever name names
# the authority actually declares; adding a kernel method to the authority extends the census
# automatically. Removing one shrinks it. There is no second source of truth here.
#
# Output: TSV rows (module_path \t name \t count), sorted by name then path, no header, no
# timestamps, no absolute paths -- byte-deterministic so the committed file's sha256 is a
# meaningful drift check. Invoke as:
#   scripts/sig_unresolved_fallthrough_census.sh > docs/measurements/sig_unresolved_fallthrough_census.tsv
# (run from the repo root; stderr carries the aggregate lines TOTAL/MODULES/NAMES/NAMES_READ)
set -euo pipefail
export LC_ALL=C
cd "$(git rev-parse --show-toplevel)"

ALGEBRA_AUTHORITY="dag/std/algebra.dag"
[ -f "$ALGEBRA_AUTHORITY" ] || { echo "AUTHORITY_MISSING: $ALGEBRA_AUTHORITY" >&2; exit 2; }

# Derive the roster from the authority: names of AlgebraFieldTemplate rows. Print the count
# read so a regression in the derivation is visible in the run record, not silent.
KERNEL_NAMES=$(grep -oE 'name: "[a-zA-Z_]+", param_types' "$ALGEBRA_AUTHORITY" \
  | sed -E 's/name: "([a-zA-Z_]+)", param_types/\1/' | sort -u)
NAMES_READ=$(printf '%s\n' "$KERNEL_NAMES" | wc -l)
echo "NAMES_READ=$NAMES_READ (from $ALGEBRA_AUTHORITY)" >&2
[ "$NAMES_READ" -gt 0 ] || { echo "ROSTER_EMPTY: derivation found no template rows" >&2; exit 2; }

OUT=$(mktemp)
trap 'rm -f "$OUT"' EXIT
: > "$OUT"

for name in $KERNEL_NAMES; do
  # Files that LOCALLY DECLARE the name as a free fn: those calls resolve via func_env.local
  # and the shape wall fires -- protected, not fallthrough.
  git grep -l "^fn ${name}(" -- dag/ src/v1/ src/v2/ 2>/dev/null > /tmp/sig_census_decl.$$ || true
  # Files that IMPORT the name: those calls resolve via parent_closure_callable_candidates and
  # the shape wall fires -- protected, not fallthrough (executed control: probe 1, the licensed
  # twin; see the run link in the PR).
  git grep -l "^import.*{.*[^a-zA-Z0-9_]${name}[,} ]" -- dag/ src/v1/ src/v2/ 2>/dev/null > /tmp/sig_census_importers.$$ || true
  # Bare labeled calls: name( preceded by non-identifier, non-dot (excludes method form x|>name(,
  # declarations, comments, import lines.
  git grep -nP "(^|[^a-zA-Z0-9_.])${name}\\([a-z_]+:" -- dag/ src/v2/ 2>/dev/null \
    | grep -v "fn ${name}(" | grep -v "|> ${name}(" | grep -v ": *//" | grep -v ": *import " \
    > /tmp/sig_census_calls.$$ || true
  [ -s /tmp/sig_census_calls.$$ ] || continue
  cut -d: -f1 /tmp/sig_census_calls.$$ | sort -u | while IFS= read -r f; do
    grep -qxF "$f" /tmp/sig_census_decl.$$ 2>/dev/null && continue
    grep -qxF "$f" /tmp/sig_census_importers.$$ 2>/dev/null && continue
    count=$(awk -F: -v p="$f" '$1 == p' /tmp/sig_census_calls.$$ | wc -l)
    [ "$count" -gt 0 ] && printf "%s\t%s\t%s\n" "$f" "$name" "$count" >> "$OUT"
  done
done

rm -f /tmp/sig_census_decl.$$ /tmp/sig_census_importers.$$ /tmp/sig_census_calls.$$ 2>/dev/null || true

# Sorted by name then path -- byte-deterministic order, fixed locale.
sort -t"$(printf '\t')" -k2,2 -k1,1 "$OUT"

echo "TOTAL=$(awk -F'\t' '{s+=$3}END{print s}' "$OUT")" >&2
echo "MODULES=$(cut -f1 "$OUT" | sort -u | wc -l)" >&2
echo "NAMES=$(cut -f2 "$OUT" | sort -u | wc -l)" >&2
echo "SHA256=$(sha256sum "$OUT" | cut -d' ' -f1)" >&2
