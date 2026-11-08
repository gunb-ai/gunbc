# PRE_REGISTRATION ROW R-15695-1 — REVISION 5 (pre-run; supersedes sha b77f206d2bc05cec2ae04b962d0ce4ef176ddb7f697361e80ddb363c9a8e06e2)

Written 2026-11-07, BEFORE any freeze-2 run, at tidy-ant-251's instruction (msg_4af5d271, the ruling on the probe-execution finding reported in msg_38d28c95; rev3 itself was written under msg_5c0d91b2). The superseded
revision is `PRE_REG_ROW_15695_freeze2_rev2.md` (sha 3a5974e2..., posted as comments 6699660743 + byte-freeze
6699681734), left on disk unedited so its hash stays checkable. Chain: a31b8bb6... -> 3a5974e2... -> c89ad90e... (rev3, comments 6699712544 + 6699712781) -> this
revision. The rev3 file is left on disk unedited so its hash stays checkable. Nothing here is an observation; expectations may not be edited after output without a separately
justified contract review. This row is #15695's; the status-file rung-honesty defect is the parent's separate
follow-up and is deliberately NOT part of #15695 or #15999.

## Supersession - what rev5 changes and what it does NOT (tidy-ant-251 ruling A, msg_8dd63ef4)

rev5 supersedes **rev4, sha `b77f206d2bc05cec2ae04b962d0ce4ef176ddb7f697361e80ddb363c9a8e06e2`** (comment
6703139348, which is left untouched and remains byte-checkable on disk as
`PRE_REG_ROW_15695_freeze2_rev4.md`). The reason is the **B2 canonical-arm repair** required by sleek-owl-397's
packet review (msg_812a5b43): the two probe match arms moved from `optional_present(value: s) =>` /
`optional_absent() =>` to the corpus-canonical `Present { value: s } =>` / `Absent =>`, because at the base
**pattern-position** lexeme arms number **zero** across all `*.dag` while the seed carried exactly two - and all
21 rtp names share **one** resolve, so an unprecedented pattern shape there could spend the single attempt with
no rtp verdicts at all. The seed is therefore re-frozen as commit `4a9efe081d8046969eb4e9ef06d219e27ce31113`
(tree `a0de248f1918aff315145392e2057db6ecc66c1a`, parent `57ce2e78...`), its probe diff is
`SEED_PROBE_SET_rebase_candidate_v2.diff` sha `5ad5a0ee2c404fc23bd6e258ee5922891d1eeed2792a2736ba85e20f02a623cc`,
and the probe block (still 45 lines) digests to `e07e18f2c6f6feec061c3b2d55255d05832c070801e414a425734dedeb0aebd0`.

**What did not change: the ledger itself.** All **34** rows - the 17 rtp witnesses, the 13 ce witnesses, and the
4 seed-only probes - are byte-identical in substance, as are their executor rows, their expected outcomes, the
at-most-one-passes law, the NO-OBSERVATION and VOID rules, and the four frozen probe literals. The probe NAMES
and LITERALS are untouched by B2 (the minimality check on the diff shows exactly 2 added lines removed and 2
added, both being arm text only), so no expectation moves. Nothing in rev5 is an observation, and the standing
rule stands: expectations may not be edited after output without a separately justified contract review.

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

## Executor ledger — all 34 witnesses of the two claim lanes (21 rtp, of which 4 are the seed-only reason probes, + 13 ce), pre-run

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
| `src/v2/test/claim/namespace_xl0/return_tail_position.dag::rtp_ctl_spelling_is_resolved` | probe (reason-witness, seed-only) | `claim_batch --claim-run --source-root dag --source-root src/v1 --source-root src/v2 --entry src/v2/test/claim/namespace_xl0/return_tail_position.dag --functions rtp_ctl_spelling_is_resolved` | NO LANE | UNOBSERVED — seed-only witness, never present in v17; no observation may be claimed for it |
| `src/v2/test/claim/namespace_xl0/return_tail_position.dag::rtp_ctl_spelling_is_absent` | probe (reason-witness, seed-only) | `claim_batch --claim-run --source-root dag --source-root src/v1 --source-root src/v2 --entry src/v2/test/claim/namespace_xl0/return_tail_position.dag --functions rtp_ctl_spelling_is_absent` | NO LANE | UNOBSERVED — seed-only witness, never present in v17; no observation may be claimed for it |
| `src/v2/test/claim/namespace_xl0/return_tail_position.dag::rtp_ctl_spelling_is_file_refused` | probe (reason-witness, seed-only) | `claim_batch --claim-run --source-root dag --source-root src/v1 --source-root src/v2 --entry src/v2/test/claim/namespace_xl0/return_tail_position.dag --functions rtp_ctl_spelling_is_file_refused` | NO LANE | UNOBSERVED — seed-only witness, never present in v17; no observation may be claimed for it |
| `src/v2/test/claim/namespace_xl0/return_tail_position.dag::rtp_ctl_spelling_is_resolve_refused` | probe (reason-witness, seed-only) | `claim_batch --claim-run --source-root dag --source-root src/v1 --source-root src/v2 --entry src/v2/test/claim/namespace_xl0/return_tail_position.dag --functions rtp_ctl_spelling_is_resolve_refused` | NO LANE | UNOBSERVED — seed-only witness, never present in v17; no observation may be claimed for it |
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

Grouped form, one group per entry file (this is the shape the v17 packet used; the ce group runs all 13, and the rtp group now runs all **21** — the 17 registered witnesses **plus the 4 seed-only reason probes**, one entry, therefore one resolve). The grouped form is what freeze-2 executes; each ledger row above keeps its own per-witness `--functions <name>` group as its registration unit, so both forms are on the record.

```
claim_batch --claim-run --source-root dag --source-root src/v1 --source-root src/v2 --entry src/v2/test/claim/namespace_xl0/return_tail_position.dag --functions a_tail_return_of_a_fn_body_is_accepted,a_tail_return_after_a_let_is_accepted,a_tail_return_of_a_fn_value_block_body_is_accepted,a_return_in_an_arm_of_a_tail_if_is_accepted,a_guard_followed_by_statements_is_accepted,a_return_in_an_arm_of_a_tail_match_is_accepted,a_return_in_an_arm_of_a_match_nested_in_a_tail_match_arm_is_accepted,a_tail_return_operand_is_read_and_refuses_unbound_at_its_atom,a_guard_arm_return_operand_is_read_and_refuses_unbound_at_its_atom,the_statements_after_a_guard_are_read_and_refuse_unbound_at_their_atom,a_tail_match_arm_return_operand_is_read_and_refuses_unbound_at_its_atom,a_return_inside_a_let_bound_if_refuses_rather_than_binding_its_operand,a_return_followed_by_a_statement_refuses_as_not_in_tail_position,a_return_before_another_statement_in_a_fn_value_refuses,an_else_less_if_that_falls_through_still_refuses_as_unlowered,an_else_less_if_as_the_last_statement_still_refuses_as_unlowered,a_guard_in_a_non_tail_spine_still_refuses_as_unlowered,rtp_ctl_spelling_is_resolved,rtp_ctl_spelling_is_absent,rtp_ctl_spelling_is_file_refused,rtp_ctl_spelling_is_resolve_refused
claim_batch --claim-run --source-root dag --source-root src/v1 --source-root src/v2 --entry src/v2/test/claim/self_host/closure_emission.dag --functions a_closure_member_whose_body_does_not_lower_is_its_own_typed_refusal_holds,the_first_refusal_of_a_closure_is_the_first_refused_arms_cause_holds,an_acquired_closure_is_emitted_as_exactly_one_arm_per_member_holds,a_file_in_the_closure_that_does_not_parse_refuses_the_emission_holds,two_files_declaring_one_module_refuse_the_emission_at_both_files_holds,an_entry_the_ingest_does_not_hold_is_refused_as_not_acquired_holds,a_name_the_ingest_does_not_bind_refuses_its_member_and_never_widens_the_closure_holds,an_importer_over_a_module_the_closure_holds_is_emitted_holds,an_import_of_a_module_absent_from_the_closure_still_refuses_resolve_module_not_found_holds,the_emission_renders_through_its_codec_and_reads_back_the_same_identities_holds,a_refused_row_carries_the_unbound_spelling_and_its_authored_position_holds,a_refused_row_without_an_unbound_mention_has_an_explicit_absent_spelling_holds,an_emitted_row_carries_no_diagnostic_holds
```

## What rev4 adds, and the finding that forced it

`claim_batch` evaluates **exactly** the names handed to `--functions`; it never scans the entry file
(`src/v1/stage0/src/bin/claim_batch.rs`: a group with neither `--function` nor `--functions` is refused with rc 2
at L1165-1172; `total_witnesses` is the sum of `group.functions.len()` at L1174; the evaluation loop is
`for function in &group.functions { run_claim_timed(...) }` at L1254; `validate_explicit_functions` refuses any
requested name the entry does not declare at L235-270). Corroboration from the frozen logs: the rtp entry declares
17 `test fn` and v17 printed 17 witnesses; the ce entry declares 13 and v17 printed 13.

The freeze-2 seed adds four reason probes to the rtp entry file (`SEED_PROBE_SET.diff` sha 6ee91462...,
re-anchored and re-formed to the canonical match-arm shape (B2) as sha 5ad5a0ee2c404fc23bd6e258ee5922891d1eeed2792a2736ba85e20f02a623cc, probe block digest e07e18f2c6f6feec061c3b2d55255d05832c070801e414a425734dedeb0aebd0). Registered under their own
names only, they would sit in the seed **never selected and never evaluated** — and a missing probe line is
indistinguishable from a probe that was never selected. So they run in the **same batch, same group**: the rtp
grouped form above names all 21. A separate group would pay a second resolve (~300 s observed in v17) out of the
one-attempt budget for no evidential gain.

### Expectations for the four probe rows — derived only from the frozen spec (bbcd32e9...); nothing new
1. the run MUST print exactly four probe witness lines; **a missing line is NO-OBSERVATION, never ABSENT**;
2. **at most one** of the four may PASS;
3. all four FAIL ⇒ **NO-OBSERVATION** (the probe set is silent) — not a verdict on either hypothesis;
4. more than one PASS ⇒ the probe set is broken; the row set is **VOID**, never a pass;
5. the four compared literals are the frozen ones verbatim — `"RESOLVED"`, `"ABSENT"`,
   `"file_refused:<none>"`, `"resolve_refused:return_not_in_tail_position"` — provenance-pinned to the probe block
   whose digest is e07e18f2c6f6feec061c3b2d55255d05832c070801e414a425734dedeb0aebd0.

### Registered pre-run checks (run and recorded BEFORE the packet is granted; any failure stops the attempt)
- `git apply --check` with **no fuzz** of `SEED_PROBE_SET_rebase_candidate_v2.diff` (sha 5ad5a0ee2c404fc23bd6e258ee5922891d1eeed2792a2736ba85e20f02a623cc) against the
  FINAL seed base — the base that also carries the main merge. Fuzz is never an accepted landing for a one-attempt
  seed; if the check fails, stop and re-anchor (a re-anchor that leaves the probe block byte-identical is not a
  content change; anything inside the block is, and comes back for contract review first).
- `grep -c '^test fn '` on the seed's src/v2/test/claim/namespace_xl0/return_tail_position.dag MUST equal **21** (17 + 4). Any other number means the seed lost or
  duplicated the probe block.
- the seed's probe block, recomputed with `probe_block_digest.py`, MUST print e07e18f2c6f6feec061c3b2d55255d05832c070801e414a425734dedeb0aebd0.


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
