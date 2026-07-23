#!/usr/bin/env bash
# Entry-grain witness eligibility census (ROADMAP 2a Lane B).
# Enumerates discovery witness entry closures under dag/test/claim + src/v2/test/claim,
# classifies each row for the bulk-native routing frontier, and writes docs/probes/ TSV + histogram.
# Full per-entry cssl first-error sweep runs on srvN (~2-4h); this script lands the mechanical
# roster with CensusPending first-error classes until that sweep receipt supersedes.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

TSV="${1:-$ROOT/docs/probes/witness_entry_eligibility_census.tsv}"
HIST="${2:-$ROOT/docs/probes/witness_entry_eligibility_histogram.txt}"
STAMP="$(date -u +%Y-%m-%dT%H:%MZ)"

emit_on_demand_family() {
  local entry="$1"
  case "$entry" in
    *emit_on_demand_family_crate*) echo "NativeFamilyLeg{family_crate}" ;;
    *emit_on_demand*) echo "InterpretedLeg{EmitOnDemandFamilyGrain}" ;;
    *) echo "InterpretedLeg" ;;
  esac
}

classify_disposition() {
  local entry="$1"
  case "$entry" in
    */long/*)
      echo "EmitIneligible	LongLaneScheduled"
      ;;
    */manual/*|*/complexity/*)
      echo "EmitIneligible	OfflineLocalRecipe"
      ;;
    *effect_reach*|*uri_witness*|*install_media*)
      echo "InterpretedRetained	HostFedCeiling"
      ;;
    *lens_module_gate*|*mandatory_tag*)
      echo "InterpretedRetained	RetainedReadsLiveTreeCarrier"
      ;;
    *emit_on_demand*)
      echo "InterpretedRetained	EmitOnDemandFamilyGrain"
      ;;
    *)
      echo "InterpretedRetained	BulkFlipPendingCensusIncomplete"
      ;;
  esac
}

module_path_from_entry() {
  local entry="$1"
  if [[ -f "$entry" ]]; then
    awk '/^module / { print $2; exit }' "$entry"
  else
    echo "unknown"
  fi
}

{
  echo "# witness_entry_eligibility_census stamp=$STAMP grain=entry_closure roots=dag/test/claim,src/v2/test/claim"
  echo -e "entry\tmodule_path\tsubject_module\tsubject_decl\tdisposition\tretained_or_ineligible_reason\tfirst_error_class\texecution_leg"
  while IFS= read -r entry; do
    [[ "$entry" == */generated/* ]] && continue
    mod="$(module_path_from_entry "$entry")"
    subject_module="$mod"
    subject_decl="witness_entry"
    disposition="$(classify_disposition "$entry" | cut -f1)"
    reason="$(classify_disposition "$entry" | cut -f2)"
    leg="$(emit_on_demand_family "$entry")"
    printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' \
      "$entry" "$mod" "$subject_module" "$subject_decl" \
      "$disposition" "$reason" "CensusPending" "$leg"
  done < <(
    find dag/test/claim src/v2/test/claim -name '*_test.dag' ! -path '*/generated/*' | sort
  )
} >"$TSV"

total="$(tail -n +2 "$TSV" | grep -v '^#' | wc -l)"
{
  echo "# witness_entry_eligibility_histogram stamp=$STAMP total_entries=$total"
  echo "disposition	reason	count"
  tail -n +2 "$TSV" | grep -v '^#' | awk -F'\t' '{print $5 "\t" $6}' | sort | uniq -c | sort -rn | awk '{print $2 "\t" $3 "\t" $1}'
} >"$HIST"

echo "wrote $TSV ($total entries)"
echo "wrote $HIST"
