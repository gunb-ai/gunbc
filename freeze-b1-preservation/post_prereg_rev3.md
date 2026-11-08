**PRE-REG (pre-run for nothing): PRE_REGISTRATION row R-15695-1, REVISION 3 — registers all 30 witnesses of the two claim lanes (tidy-ant-251 msg_5c0d91b2). SUPERSEDES sha 3a5974e2f975680d7b07b42d79eb383baeb9bac77abf45ab341ae54ec0db69d2.**

- File: `PRE_REG_ROW_15695_freeze2_rev3.md` — 20,062 B, sha256 `c89ad90e2e7e7d056a0c67a17199fe93c9590f1235b52b5d900555ac376ffec9`. Exact bytes follow in the paired byte-freeze comment (GZB64), same as rev2.
- Chain: a31b8bb6... -> 3a5974e2... -> c89ad90e... (this). Prior files untouched on disk, hashes still checkable.
- Still PRE-RUN: written before any freeze-2 run; no claim output informed any word. Expectations frozen.
- New: the 11 ce PASS witnesses now have executor rows (the ce lane is one entry group of 13: `claims-ce.log` sha 515596e7..., lines 3 and 17). Their expected value is the **v17 OBSERVATION carried forward, explicitly NOT a prediction under P**; v17's own pre-reg declared the ce files out of scope (`PRE_REGISTRATION_v17.md` :97), so no earlier prediction exists, and nothing was authored backwards from v17 output. They are non-discriminating: identical under both hypotheses; a flip at freeze-2 is a FINDING, never an expectation edit.
- Unchanged in substance: the R-15695-1 positive-control row, the 17 rtp rows, and the 2 ce FAIL rows (verified: the first 19 ledger rows of this file are byte-identical to rev2's 19).

# PRE_REGISTRATION ROW R-15695-1 — REVISION 3 (pre-run; supersedes sha 3a5974e2f975680d7b07b42d79eb383baeb9bac77abf45ab341ae54ec0db69d2)

Written 2026-11-07, BEFORE any freeze-2 run, at tidy-ant-251's instruction (msg_5c0d91b2). The superseded
revision is `PRE_REG_ROW_15695_freeze2_rev2.md` (sha 3a5974e2..., posted as comments 6699660743 + byte-freeze
6699681734), left on disk unedited so its hash stays checkable. Chain: a31b8bb6... -> 3a5974e2... -> this
revision. Nothing here is an observation; expectations may not be edited after output without a separately
justified contract review. This row is #15695's; the status-file rung-honesty defect is the parent's separate
follow-up and is deliberately NOT part of #15695 or #15999.

## What rev3 adds, and the one thing it does NOT do
The ce lane executes as ONE entry group of 13 witnesses (`claims-ce.log`, sha 515596e7..., line 3
`claim_batch: 1 entry group(s), 13 witness(es) total`; line 17 `resolving
src/v2/test/claim/self_host/closure_emission.dag once (13 witness(es))`). A pre-reg that registers only the 2
FAILs leaves 11 of the run's outputs unregistered, so they are now rows too. **Their expected value is the v17
OBSERVATION, carried forward and labelled as such — it is NOT a prediction under candidate P and NOT a
prediction under the dead-rung hypothesis.** The v17 pre-registration declared these files out of scope
(`PRE_REGISTRATION_v17.md` :97, "ce claim files: OUT OF SCOPE of this packet (not in the roster); their 2 FAILs
stay open"), so no earlier prediction exists for them; nothing was authored backwards from v17 output either.
They pass identically under both hypotheses and therefore carry no discriminating power: they are registered
for completeness, and any flip at freeze-2 is a FINDING to report verbatim, never an expectation edit.
The 17 rtp rows and the 2 ce FAIL rows are unchanged in substance from rev2.

## Why an executor column exists
`claim_batch` evaluates only the entries it is handed (`--entry <file> --functions <csv>`;
src/v1/stage0/src/bin/claim_batch.rs). A PRE_REG row that does not name the lane that executes its entry lets
"predicted" be read as "observed". Today the lane for every one of these entries is **NO LANE**: no workflow
invokes `claim_batch` (its only .github appearances are packaging/stat/sha checks in fleet-converge.yml);
`claim_executor` is invoked only as `--verify-build-artifacts` (fleet-converge.yml:688,1089) and
`--regen-round-cost --base-mirror` (heal.yml:113); its verdict modes `--required-ci`/`--required-lane`
(claim_executor.rs:244,259) are enrolled by no workflow. CI at 9c15c98c executed only
`gunbc test //gunbc/instruments:self-host` and `gunbc test //gunbc/instruments:v2-native-cli`.

## Row R-15695-1 — the positive control (unchanged in substance from a31b8bb6 / 3a5974e2)
`a_tail_return_of_a_fn_body_is_accepted`, fixture `rtp_tail_ctl_source` = `module v2.test.rtp_tail_ctl` +
imports {Bool, Int} + `fn rtp_probe(i: Int) -> Int { return i }` — no if, no match, no else-less if, so the
#15695 condition-production change cannot reach it. Expected verdict `NativeCensusModuleResolved`
(`rtp_verdict_of` :192-194).

| leg | tree | prediction under candidate P (head-read cause) | prediction under the dead-rung hypothesis |
|---|---|---|---|
| A (repair head) | the #15999 repair head after the main merge | **PASS** | **FAIL** (bare) |
| B (base leg) | the packet's base tree | PASS for this entry only (grammar-neutral fixture) | FAIL (bare) |

This single row separates the hypotheses because the fixture is unaffected by the parse fix: a PASS at A means
the acceptance path can resolve a module and the v17 head reading was the cause; a bare FAIL at A means nothing
on the acceptance path resolves and the dead-rung hypothesis stands. Both readings remain HYPOTHESES.

## Executor ledger — all 30 witnesses of the two claim lanes (17 rtp + 13 ce), pre-run

| entry | class | exact executor group (freeze-2, proposed) | lane today | observed so far (v17 packet only; log sha 5d485d14... / 515596e7...) |
|---|---|---|---|---|
| `src/v2/test/claim/namespace_xl0/return_tail_position.dag::a_tail_return_of_a_fn_body_is_accepted` | acceptance | `claim_batch --claim-run --source-root dag --source-root src/v1 --source-root src/v2 --entry src/v2/test/claim/namespace_xl0/return_tail_position.dag --functions a_tail_return_of_a_fn_body_is_accepted` | NO LANE | FAIL (bare) |
| `src/v2/test/claim/namespace_xl0/return_tail_position.dag::a_tail_return_after_a_let_is_accepted` | acceptance | `claim_batch --claim-run --source-root dag --source-root src/v1 --source-root src/v2 --entry src/v2/test/claim/namespace_xl0/return_tail_position.dag --functions a_tail_return_after_a_let_is_accepted` | NO LANE | FAIL (bare) |
| `src/v2/test/claim/namespace_xl0/return_tail_position.dag::a_tail_return_of_a_fn_value_block_body_is_accepted` | acceptance | `claim_batch --claim-run --source-root dag --source-root src/v1 --source-root src/v2 --entry src/v2/test/claim/namespace_xl0/return_tail_position.dag --functions a_tail_return_of_a_fn_value_block_body_is_accepted` | NO LANE | FAIL (bare) |
| `src/v2/test/claim/namespace_xl0/return_tail_position.dag::a_return_in_an_arm_of_a_tail_if_is_accepted` | acceptance | `claim_batch --claim-run --source-root dag --source-root src/v1 --source-root src/v2 --entry src/v2/test/claim/namespace_xl0/return_tail_position.dag --functions a_return_in_an_arm_of_a_tail_if_is_accepted` | NO LANE | FAIL (bare) |
| `src/v2/test/claim/namespace_xl0/return_tail_position.dag::a_guard_followed_by_statements_is_accepted` | acceptance | `claim_batch --claim-run --source-root dag --source-root src/v1 --source-root src/v2 --entry src/v2/test/claim/namespace_xl0/return_tail_position.dag --functions a_guard_followed_by_statements_is_accepted` | NO LANE | FAIL (bare) |
| `src/v2/test/claim/namespace_xl0/return_tail_position.dag::a_return_in_an_arm_of_a_tail_match_is_accepted` | acceptance | `claim_batch --claim-run --source-root dag --source-root src/v1 --source-root src/v2 --entry src/v2/test/claim/namespace_xl0/return_tail_position.dag --functions a_return_in_an_arm_of_a_tail_match_is_accepted` | NO LANE | FAIL (bare) |
| `src/v2/test/claim/namespace_xl0/return_tail_position.dag::a_return_in_an_arm_of_a_match_nested_in_a_tail_match_arm_is_accepted` | acceptance | `claim_batch --claim-run --source-root dag --source-root src/v1 --source-root src/v2 --entry src/v2/test/claim/namespace_xl0/return_tail_position.dag --functions a_return_in_an_arm_of_a_match_nested_in_a_tail_match_arm_is_accepted` | NO LANE | FAIL (bare) |
| `src/v2/test/claim/namespace_xl0/return_tail_position.dag::a_tail_return_operand_is_read_and_refuses_unbound_at_its_atom` | unbound-read | `claim_batch --claim-run --source-root dag --source-root src/v1 --source-root src/v2 --entry src/v2/test/claim/namespace_xl0/return_tail_position.dag --functions a_tail_return_operand_is_read_and_refuses_unbound_at_its_atom` | NO LANE | FAIL (bare) |
| `src/v2/test/claim/namespace_xl0/return_tail_position.dag::a_guard_arm_return_operand_is_read_and_refuses_unbound_at_its_atom` | unbound-read | `claim_batch --claim-run --source-root dag --source-root src/v1 --source-root src/v2 --entry src/v2/test/claim/namespace_xl0/return_tail_position.dag --functions a_guard_arm_return_operand_is_read_and_refuses_unbound_at_its_atom` | NO LANE | FAIL (bare) |
| `src/v2/test/claim/namespace_xl0/return_tail_position.dag::the_statements_after_a_guard_are_read_and_refuse_unbound_at_their_atom` | unbound-read | `claim_batch --claim-run --source-root dag --source-root src/v1 --source-root src/v2 --entry src/v2/test/claim/namespace_xl0/return_tail_position.dag --functions the_statements_after_a_guard_are_read_and_refuse_unbound_at_their_atom` | NO LANE | FAIL (bare) |
| `src/v2/test/claim/namespace_xl0/return_tail_position.dag::a_tail_match_arm_return_operand_is_read_and_refuses_unbound_at_its_atom` | unbound-read | `claim_batch --claim-run --source-root dag --source-root src/v1 --source-root src/v2 --entry src/v2/test/claim/namespace_xl0/return_tail_position.dag --functions a_tail_match_arm_return_operand_is_read_and_refuses_unbound_at_its_atom` | NO LANE | FAIL (bare) |
| `src/v2/test/claim/namespace_xl0/return_tail_position.dag::a_return_inside_a_let_bound_if_refuses_rather_than_binding_its_operand` | refusal | `claim_batch --claim-run --source-root dag --source-root src/v1 --source-root src/v2 --entry src/v2/test/claim/namespace_xl0/return_tail_position.dag --functions a_return_inside_a_let_bound_if_refuses_rather_than_binding_its_operand` | NO LANE | PASS (vacuous: expects FileRefused) |
| `src/v2/test/claim/namespace_xl0/return_tail_position.dag::a_return_followed_by_a_statement_refuses_as_not_in_tail_position` | refusal | `claim_batch --claim-run --source-root dag --source-root src/v1 --source-root src/v2 --entry src/v2/test/claim/namespace_xl0/return_tail_position.dag --functions a_return_followed_by_a_statement_refuses_as_not_in_tail_position` | NO LANE | PASS (vacuous) |
| `src/v2/test/claim/namespace_xl0/return_tail_position.dag::a_return_before_another_statement_in_a_fn_value_refuses` | refusal | `claim_batch --claim-run --source-root dag --source-root src/v1 --source-root src/v2 --entry src/v2/test/claim/namespace_xl0/return_tail_position.dag --functions a_return_before_another_statement_in_a_fn_value_refuses` | NO LANE | PASS (vacuous) |
| `src/v2/test/claim/namespace_xl0/return_tail_position.dag::an_else_less_if_that_falls_through_still_refuses_as_unlowered` | refusal | `claim_batch --claim-run --source-root dag --source-root src/v1 --source-root src/v2 --entry src/v2/test/claim/namespace_xl0/return_tail_position.dag --functions an_else_less_if_that_falls_through_still_refuses_as_unlowered` | NO LANE | PASS (vacuous) |
| `src/v2/test/claim/namespace_xl0/return_tail_position.dag::an_else_less_if_as_the_last_statement_still_refuses_as_unlowered` | refusal | `claim_batch --claim-run --source-root dag --source-root src/v1 --source-root src/v2 --entry src/v2/test/claim/namespace_xl0/return_tail_position.dag --functions an_else_less_if_as_the_last_statement_still_refuses_as_unlowered` | NO LANE | PASS (vacuous) |
| `src/v2/test/claim/namespace_xl0/return_tail_position.dag::a_guard_in_a_non_tail_spine_still_refuses_as_unlowered` | refusal | `claim_batch --claim-run --source-root dag --source-root src/v1 --source-root src/v2 --entry src/v2/test/claim/namespace_xl0/return_tail_position.dag --functions a_guard_in_a_non_tail_spine_still_refuses_as_unlowered` | NO LANE | PASS (vacuous) |
| `src/v2/test/claim/self_host/closure_emission.dag::a_closure_member_whose_body_does_not_lower_is_its_own_typed_refusal_holds` | closure arm | `claim_batch --claim-run --source-root dag --source-root src/v1 --source-root src/v2 --entry src/v2/test/claim/self_host/closure_emission.dag --functions a_closure_member_whose_body_does_not_lower_is_its_own_typed_refusal_holds` | NO LANE | FAIL (bare) |
| `src/v2/test/claim/self_host/closure_emission.dag::the_first_refusal_of_a_closure_is_the_first_refused_arms_cause_holds` | closure arm | `claim_batch --claim-run --source-root dag --source-root src/v1 --source-root src/v2 --entry src/v2/test/claim/self_host/closure_emission.dag --functions the_first_refusal_of_a_closure_is_the_first_refused_arms_cause_holds` | NO LANE | FAIL (bare) |
| `src/v2/test/claim/self_host/closure_emission.dag::an_acquired_closure_is_emitted_as_exactly_one_arm_per_member_holds` | ce PASS — OBSERVATION REGISTERED, not a prediction under P | `claim_batch --claim-run --source-root dag --source-root src/v1 --source-root src/v2 --entry src/v2/test/claim/self_host/closure_emission.dag --functions an_acquired_closure_is_emitted_as_exactly_one_arm_per_member_holds` | NO LANE | PASS (v17; log sha 515596e7...) |
| `src/v2/test/claim/self_host/closure_emission.dag::a_file_in_the_closure_that_does_not_parse_refuses_the_emission_holds` | ce PASS — OBSERVATION REGISTERED, not a prediction under P | `claim_batch --claim-run --source-root dag --source-root src/v1 --source-root src/v2 --entry src/v2/test/claim/self_host/closure_emission.dag --functions a_file_in_the_closure_that_does_not_parse_refuses_the_emission_holds` | NO LANE | PASS (v17; log sha 515596e7...) |
| `src/v2/test/claim/self_host/closure_emission.dag::two_files_declaring_one_module_refuse_the_emission_at_both_files_holds` | ce PASS — OBSERVATION REGISTERED, not a prediction under P | `claim_batch --claim-run --source-root dag --source-root src/v1 --source-root src/v2 --entry src/v2/test/claim/self_host/closure_emission.dag --functions two_files_declaring_one_module_refuse_the_emission_at_both_files_holds` | NO LANE | PASS (v17; log sha 515596e7...) |
| `src/v2/test/claim/self_host/closure_emission.dag::an_entry_the_ingest_does_not_hold_is_refused_as_not_acquired_holds` | ce PASS — OBSERVATION REGISTERED, not a prediction under P | `claim_batch --claim-run --source-root dag --source-root src/v1 --source-root src/v2 --entry src/v2/test/claim/self_host/closure_emission.dag --functions an_entry_the_ingest_does_not_hold_is_refused_as_not_acquired_holds` | NO LANE | PASS (v17; log sha 515596e7...) |
| `src/v2/test/claim/self_host/closure_emission.dag::a_name_the_ingest_does_not_bind_refuses_its_member_and_never_widens_the_closure_holds` | ce PASS — OBSERVATION REGISTERED, not a prediction under P | `claim_batch --claim-run --source-root dag --source-root src/v1 --source-root src/v2 --entry src/v2/test/claim/self_host/closure_emission.dag --functions a_name_the_ingest_does_not_bind_refuses_its_member_and_never_widens_the_closure_holds` | NO LANE | PASS (v17; log sha 515596e7...) |
| `src/v2/test/claim/self_host/closure_emission.dag::an_importer_over_a_module_the_closure_holds_is_emitted_holds` | ce PASS — OBSERVATION REGISTERED, not a prediction under P | `claim_batch --claim-run --source-root dag --source-root src/v1 --source-root src/v2 --entry src/v2/test/claim/self_host/closure_emission.dag --functions an_importer_over_a_module_the_closure_holds_is_emitted_holds` | NO LANE | PASS (v17; log sha 515596e7...) |
| `src/v2/test/claim/self_host/closure_emission.dag::an_import_of_a_module_absent_from_the_closure_still_refuses_resolve_module_not_found_holds` | ce PASS — OBSERVATION REGISTERED, not a prediction under P | `claim_batch --claim-run --source-root dag --source-root src/v1 --source-root src/v2 --entry src/v2/test/claim/self_host/closure_emission.dag --functions an_import_of_a_module_absent_from_the_closure_still_refuses_resolve_module_not_found_holds` | NO LANE | PASS (v17; log sha 515596e7...) |
| `src/v2/test/claim/self_host/closure_emission.dag::the_emission_renders_through_its_codec_and_reads_back_the_same_identities_holds` | ce PASS — OBSERVATION REGISTERED, not a prediction under P | `claim_batch --claim-run --source-root dag --source-root src/v1 --source-root src/v2 --entry src/v2/test/claim/self_host/closure_emission.dag --functions the_emission_renders_through_its_codec_and_reads_back_the_same_identities_holds` | NO LANE | PASS (v17; log sha 515596e7...) |
| `src/v2/test/claim/self_host/closure_emission.dag::a_refused_row_carries_the_unbound_spelling_and_its_authored_position_holds` | ce PASS — OBSERVATION REGISTERED, not a prediction under P | `claim_batch --claim-run --source-root dag --source-root src/v1 --source-root src/v2 --entry src/v2/test/claim/self_host/closure_emission.dag --functions a_refused_row_carries_the_unbound_spelling_and_its_authored_position_holds` | NO LANE | PASS (v17; log sha 515596e7...) |
| `src/v2/test/claim/self_host/closure_emission.dag::a_refused_row_without_an_unbound_mention_has_an_explicit_absent_spelling_holds` | ce PASS — OBSERVATION REGISTERED, not a prediction under P | `claim_batch --claim-run --source-root dag --source-root src/v1 --source-root src/v2 --entry src/v2/test/claim/self_host/closure_emission.dag --functions a_refused_row_without_an_unbound_mention_has_an_explicit_absent_spelling_holds` | NO LANE | PASS (v17; log sha 515596e7...) |
| `src/v2/test/claim/self_host/closure_emission.dag::an_emitted_row_carries_no_diagnostic_holds` | ce PASS — OBSERVATION REGISTERED, not a prediction under P | `claim_batch --claim-run --source-root dag --source-root src/v1 --source-root src/v2 --entry src/v2/test/claim/self_host/closure_emission.dag --functions an_emitted_row_carries_no_diagnostic_holds` | NO LANE | PASS (v17; log sha 515596e7...) |

Grouped form, one group per entry file (this is the shape the v17 packet used; the ce group runs all 13):

```
claim_batch --claim-run --source-root dag --source-root src/v1 --source-root src/v2 --entry src/v2/test/claim/namespace_xl0/return_tail_position.dag --functions a_tail_return_of_a_fn_body_is_accepted,a_tail_return_after_a_let_is_accepted,a_tail_return_of_a_fn_value_block_body_is_accepted,a_return_in_an_arm_of_a_tail_if_is_accepted,a_guard_followed_by_statements_is_accepted,a_return_in_an_arm_of_a_tail_match_is_accepted,a_return_in_an_arm_of_a_match_nested_in_a_tail_match_arm_is_accepted,a_tail_return_operand_is_read_and_refuses_unbound_at_its_atom,a_guard_arm_return_operand_is_read_and_refuses_unbound_at_its_atom,the_statements_after_a_guard_are_read_and_refuse_unbound_at_their_atom,a_tail_match_arm_return_operand_is_read_and_refuses_unbound_at_its_atom,a_return_inside_a_let_bound_if_refuses_rather_than_binding_its_operand,a_return_followed_by_a_statement_refuses_as_not_in_tail_position,a_return_before_another_statement_in_a_fn_value_refuses,an_else_less_if_that_falls_through_still_refuses_as_unlowered,an_else_less_if_as_the_last_statement_still_refuses_as_unlowered,a_guard_in_a_non_tail_spine_still_refuses_as_unlowered
claim_batch --claim-run --source-root dag --source-root src/v1 --source-root src/v2 --entry src/v2/test/claim/self_host/closure_emission.dag --functions a_closure_member_whose_body_does_not_lower_is_its_own_typed_refusal_holds,the_first_refusal_of_a_closure_is_the_first_refused_arms_cause_holds,an_acquired_closure_is_emitted_as_exactly_one_arm_per_member_holds,a_file_in_the_closure_that_does_not_parse_refuses_the_emission_holds,two_files_declaring_one_module_refuse_the_emission_at_both_files_holds,an_entry_the_ingest_does_not_hold_is_refused_as_not_acquired_holds,a_name_the_ingest_does_not_bind_refuses_its_member_and_never_widens_the_closure_holds,an_importer_over_a_module_the_closure_holds_is_emitted_holds,an_import_of_a_module_absent_from_the_closure_still_refuses_resolve_module_not_found_holds,the_emission_renders_through_its_codec_and_reads_back_the_same_identities_holds,a_refused_row_carries_the_unbound_spelling_and_its_authored_position_holds,a_refused_row_without_an_unbound_mention_has_an_explicit_absent_spelling_holds,an_emitted_row_carries_no_diagnostic_holds
```

## Obligations this revision imposes on freeze-2
1. Per row, `executor` must be filled with the group that ACTUALLY ran (or `NO LANE`), and `observed` may be
   filled only from a log with a measured sha. Until then the row reads UNOBSERVED.
2. Bare FAILs must stop being the evidence class: carry a reason-bearing witness for the acceptance entries
   (print each verdict's fatal_reason/at, or add `rtp_tail_ctl_fatal_reason_is_resolved`), so a FAIL
   distinguishes FileRefused(reason=...) from ResolveRefused(...) from RtpAbsent.
3. The 6 refusal-expecting rtp PASSes are labelled vacuous: they pass identically under an always-refuse fold
   and certify nothing about acceptance.
4. Qualification rows in `dag/gunbc/compiler_frontend_program_status.dag` owned by `v2.test.claim.*` decls
   (24 of 27 `PrerequisiteDelivered` rows; 18 with `measured_at` == `merge_sha`) read CLAIMED, not VERIFIED,
   until their entry carries an executor with an observed log sha. Tracked in the parent's separate lane; not
   a change to #15695 or #15999.
5. The 11 ce PASS rows are OBSERVATION REGISTRATIONS, not predictions; they are non-discriminating by
   construction. A flip at freeze-2 is a FINDING (report verbatim), never a licence to re-expect.
