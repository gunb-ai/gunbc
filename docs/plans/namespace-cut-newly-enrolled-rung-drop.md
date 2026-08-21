# Rung drop: 15 newly-enrolled floor rows on integration/namespace-cut

**Declared 2026-08-21.** This is a §4b(3) rung-drop declaration, not a quarantine. It
enumerates every member individually, because a rung drop that lists its members is
auditable and one that states a count is not.

## What changed

The floor moved `failed=495` (roster 9834) to `failed=498` (roster 9909). An identity
join over both runs' complete failing sets — both reconstructed from the raw job-logs
API, both reconciling exactly to their reported counts — decomposes that delta:

| bucket | count | closes against |
|---|---|---|
| still failing | 483 | `483 + 12 = 495` (old run) |
| repaired | 12 | the twelve cut regressions, individually named |
| **newly enrolled** | **15** | `483 + 15 = 498` (new run) |

**The cut repaired twelve and broke none.** The 15 below are rows that entered the
roster in this interval; they are newly *present*, not newly *broken*.

## Previous rung, temporary rung, reason

- **Previous rung:** not applicable — these rows were not in the roster, so they held
  no rung. This is enrolment, not regression.
- **Temporary rung:** *mitigatable*. The failures are typed, located and counted; the
  floor refuses on them rather than passing them silently.
- **Reason:** the producing code is not this branch's. The namespace cut neither
  authored these witnesses nor the subjects they assert over; it absorbed them from
  main and made them reachable.

## Bounded population — every row

### `dag/test/claim/build_selection_witness_test.dag` (11)
Old run: 1 mention. New run: 23. Effectively unenrolled before this interval.

1. `w_a_candidate_in_an_undeclared_currency_does_not_rank` — ERROR 310843ns
2. `w_a_fully_owned_build_ranks_at_zero_cash` — ERROR 769567ns
3. `w_an_unbounded_ceiling_is_declared_missing_not_read_as_zero`
4. `w_an_unread_compatibility_is_pending_evidence_not_a_winner`
5. `w_a_pending_candidate_neither_wins_nor_poisons_the_field`
6. `w_a_refused_cash_answer_makes_the_candidate_incomparable`
7. `w_a_strictly_worse_candidate_is_dominated`
8. `w_a_tradeoff_is_not_a_domination`
9. `w_duplicate_candidate_identity_refuses_the_selection`
10. `w_the_better_candidate_stands_on_the_frontier`
11. `w_the_projected_entry_is_keyed_by_declaration_identity`

### `dag/test/claim/generic_item_clone_bound_witness_test.dag` (2)
Old run: 17 mentions. New run: 21.

12. `freemonoid_supplemental_enum_hand_written_impls` — FAILED 190ms
13. `freemonoid_supplemental_struct_hand_written_impls` — FAILED 192ms

These two are ~190ms, not sub-millisecond: the assertion RAN and returned false.
They read the emitted supplemental impls, which the qualified-type emitter gap
below is expected to affect.

### `dag/test/claim/fleet_desired_expectation_witness_test.dag` (2)
Old run: **ZERO mentions** — the file arrived with main's #8701 through this
branch's integration.

14. `a_refused_transport_licenses_nothing` — ERROR 41041ns
15. `an_undecodable_advertisement_licenses_nothing` — ERROR 24081ns

**These two are not a defect and must not be repaired.** Sub-millisecond ERROR is
the hermetic-boundary signature: the witness's subject is a host effect, and
`run_required_floor`'s hermetic envelope refuses it before the effect is reached.
Measured at **7 declared / 7 PASS / 0 FAIL** in a WET `claim_batch` run of the same
file. Both readings are correct in their own frame; DESIGN already states that
`executed` counts a witness reaching the fold, not its assertion running.

## Restoration trigger

- **Rows 14–15:** no restoration is owed. They are correctly refused at the hermetic
  boundary and pass wet. The obligation is to keep the frame labelled wherever either
  number is quoted.
- **Rows 12–13:** retire when the qualified-type emitter lands, since they assert over
  emitted supplemental impls and the emitter currently renders
  `std.algebra._free_monoid<K>` where the mirror has `Rc<FreeMonoid<K>>`.
- **Rows 1–11:** owned by whoever owns `build_selection_witness`. This branch does not
  author build selection and will not repair its assertions. Restoration trigger is
  that owner's disposition, not a namespace-cut event.

## What this declaration does NOT claim

It does not claim the 483 still-failing rows are acceptable, and it does not enroll
them. It covers exactly the 15 rows that entered the roster in this interval.
