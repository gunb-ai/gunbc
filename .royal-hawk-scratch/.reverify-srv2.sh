set -euo pipefail
verification_root=/home/briansrls/royal-hawk-241
verification_repo="$verification_root/verification.RAVFdi/repo"
verification_run=$(mktemp -d "$verification_root/reverification.XXXXXX")
printf '%s\n' "$verification_run/verification.log"
cd "$verification_repo"
test -z "$(git status --porcelain)"
git fetch origin royal-hawk-241/share-native-context
git checkout --detach 5b6e10bfb8
mkdir "$verification_run/runner-temp"
test ! -e src/v2/local_shared_context_diagnostic.dag
cat > src/v2/local_shared_context_diagnostic.dag <<'DAG_DIAGNOSTIC'
module local.shared_context_diagnostic

import v2.test.name_resolve.admission_fail_closed { fc_dual_import_roots, fc_qn }
import v2.compiler.name_resolve { Admission, ResolutionSubject, resolution_context, resolve_with_admission_context_policy }
import v2.extdeps.languages.dag { dag_language_model_surface_empty_prelude }
import v2.std.cross_tree.resolution { source_root_index_empty }
import v2.std.diagnostic { Accepted, Rejected }
import v2.std.logic { Bool }
import v2.std.resolution_policy { ImportScoped }

test fn main() -> Bool {
  match fc_dual_import_roots() {
    Rejected { diagnostics: _ } => false
    Accepted { value: roots, diagnostics: _ } =>
      match resolve_with_admission_context_policy(
        context: resolution_context(lm: dag_language_model_surface_empty_prelude(), roots: roots),
        admission: Admission { subject: ResolutionSubject { name: fc_qn(seg: ^fc_import_a_seg) }, imports: [] },
        index: source_root_index_empty(),
        active_roots: [],
        policy: ImportScoped
      ) {
        Accepted { value: _, diagnostics: _ } => false
        Rejected { diagnostics: d } => d.head.reason == ^resolve_reason_unbound_symbol
      }
  }
}
DAG_DIAGNOSTIC
trap 'rm -f src/v2/local_shared_context_diagnostic.dag' EXIT
systemd-run --user --scope -p MemoryMax=24G env RUNNER_TEMP="$verification_run/runner-temp" CTRL_BUILD_MODE=local bash -c '
set -euo pipefail
cargo build --release -p v1-compiler --bin gunbc
failed=0
if env RUNNER_TEMP="$(mktemp -d "$RUNNER_TEMP/diagnosis.XXXXXX")" ./target/release/gunbc run --source-root dag --source-root src/v2 --entry src/v2/local_shared_context_diagnostic.dag --function main --claim-run; then
  printf "DIAGNOSIS export_provider_has_unbound_service_marker PASS\n"
else
  printf "DIAGNOSIS export_provider_has_unbound_service_marker FAIL\n"
  failed=1
fi
rm src/v2/local_shared_context_diagnostic.dag
for witness in shared_context_duplicate_root_refusal_preserves_every_diagnostic_for_each_subject shared_context_malformed_root_refusal_preserves_every_diagnostic_for_each_subject shared_context_resolves_two_subjects_and_preserves_missing_subject_location resolve_dual_imports_do_not_collide_on_header_metadata resolve_with_admission_threads_validate_diagnostics_on_accept; do
  if env RUNNER_TEMP="$(mktemp -d "$RUNNER_TEMP/witness.XXXXXX")" ./target/release/gunbc run --source-root dag --source-root src/v2 --entry src/v2/test/claim/name_resolve/admission_fail_closed_test.dag --function "$witness" --claim-run; then
    printf "VERIFY %s PASS\n" "$witness"
  else
    result=$?
    printf "VERIFY %s FAIL rc=%s\n" "$witness" "$result"
    failed=1
  fi
done
for witness in shared_root_validation_refusal_stays_at_prepare_for_each_identity the_requested_declaration_is_selected_and_evaluated a_nested_same_named_graft_edge_is_not_selected an_absent_declaration_is_not_found a_sibling_modules_declaration_is_not_selected; do
  if env RUNNER_TEMP="$(mktemp -d "$RUNNER_TEMP/witness.XXXXXX")" ./target/release/gunbc run --source-root dag --source-root src/v2 --entry src/v2/test/native_decl_selection_test.dag --function "$witness" --claim-run; then
    printf "VERIFY %s PASS\n" "$witness"
  else
    result=$?
    printf "VERIFY %s FAIL rc=%s\n" "$witness" "$result"
    failed=1
  fi
done
exit "$failed"
' > "$verification_run/verification.log" 2>&1
printf '%s\n' "$verification_run/verification.log"
