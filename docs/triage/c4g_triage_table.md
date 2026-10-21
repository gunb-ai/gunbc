# C4-G triage table (keen-owl-704 for warm-wolf-570)

Base: origin/main 647d770c78c. Working notes, not an authority.

| # | claim | culprit | owner | status |
|---|-------|---------|-------|--------|
| 1 | quarantine_probe_disposition_witness (2 claims) | 4 QuarantineProbeExpectRed rows held by nothing (constructor_tag_route_probe_red x2, map_literal ml_* x2); fails already at parent of #14164 | silent-bee-194 (#14532) | FIXED BY #14532: at head 7371b408d5f the disposition claims PASS (wet): derives_exactly_one_disposition, every_admitted_probe_row_has_at_least_one_holder, the_former_live_tree_declined_row_is_now_expected_red; the second red claim was renamed there. No separate fix. |
| 2 | roadmap_hierarchy_view_witness :326 | #14351 (dead Document arm after review 82244) | keen-owl-704 | fixed: #14629 (approved), claims PASS at head |
| 3 | roadmap_authority witness_accepted_parents_precede_dependents | projection succeeds; an accepted node has a non-accepted parent | keen-owl-704 | locating node |
| 4 | react_jsx_emit | n/a | jolly-boar-495 (#14623) | owned elsewhere |
| 5 | #14185 record_escapes pair un-enrolled | #14185 (eb0b1dc8d13) | keen-owl-704 | not started |
| 6 | runner_slot_cap_observe_route_witness | stale `import v2.std.operation_argv` after re-home by #14449 (00ad7044178) | fix forward: branch c4g-slot-cap-import (1-line import) | verifying base FAIL / head PASS |
| 7 | normalize/service_spine: a_service_namespace_does_not_bind_bare_on_the_resolve_route_holds, an_imported_service_resolves_through_the_import_holds | pre-existing on main (found by jolly-boar-495 on #14630, identical at base and head) | unassigned | queued: reproduce at origin/main, find culprit by identity |
| 8 | value_base_projection: vbp_undeclared_field_off_a_call_result_is_carried_on_the_direct_infer_outcome | same source as row 7 | unassigned | queued |
| 9 | map_literal: ml_the_key_law_names_both_occurrences_holds (runs on no schedule) | found by sunny-ibex-149's grain fix on #14469 | unassigned | queued; may be same family as row 1 ml_* rows |
