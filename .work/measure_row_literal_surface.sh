#!/bin/bash
# Print the CompileDiagnosticCensusRow literal surface (occurrence-based). Excludes the
# 'pub struct' declaration and '-> CompileDiagnosticCensusRow {' return-type braces, so what
# remains are the sites that name every field and must each gain the new field.
set -u
REF=${1:-HEAD}
git diff --quiet || { echo "REFUSE: uncommitted changes"; exit 1; }
COMMIT=$(git rev-parse "$REF"); echo "REF=$REF COMMIT=$COMMIT"
echo "--- per file: matches / return-type-braces / struct-decls / = constructions ---"
DW=0; CW=0; RC_=0; SD=0
for f in $(git ls-tree -r --name-only "$REF" | grep -E '\.(dag|rs)$'); do
  txt=$(git show "$REF:$f" 2>/dev/null) || continue
  m=$(printf '%s' "$txt" | grep -c 'CompileDiagnosticCensusRow {' || true)
  [ "$m" = "0" ] && continue
  rb=$(printf '%s' "$txt" | grep -c -- '-> CompileDiagnosticCensusRow {' || true)
  sd=$(printf '%s' "$txt" | grep -c 'pub struct CompileDiagnosticCensusRow' || true)
  cons=$(( m - rb - sd ))
  printf '  %2d -   %d -   %d =   %2d  %s\n' "$m" "$rb" "$sd" "$cons" "$f"
  case "$f" in
    src/v2/test/claim/*) CW=$((CW+cons));;
    dag/*) DW=$((DW+cons));;
    *) [ "$sd" != "0" ] && SD=$((SD+sd)); RC_=$((RC_+cons));;
  esac
done
WT=$((DW+CW)); ES=$((WT+RC_+SD))
echo "--- totals ---"
echo "dag_witness=$DW  canary_witness=$CW  WITNESS_TOTAL=$WT"
echo "rust_construction=$RC_  struct_declaration=$SD  EDIT_SITES=$ES"
echo "--- fold + projection + extractor sites, symbol-cited ---"
git grep -n 'fn compile_diagnostic_census_rows(' "$REF" -- src/v1/stage0/src/cli_run.rs
git grep -n 'fn compile_diagnostic_census_row_value(' "$REF" -- src/v1/stage0/src/v1_interpreter.rs
git grep -n 'pub fn compile_clean_diagnostic_histogram_key(' "$REF" -- 'src/v1/stage0/src/**'
echo "--- carrier declaration ---"
git grep -n 'CompileDiagnosticCensusRow\b' "$REF" -- dag/gunbc/compile_diagnostic_census.dag | head -4
echo "--- node_type_shape authored + emitted ---"
git grep -n 'n_name, "<"' "$REF" -- src/v1/04_types.dag | head -2
git grep -n 'pub fn node_type_shape' "$REF" -- src/v1/stage0/src/v1_compiler_infer_types.rs | head -2
