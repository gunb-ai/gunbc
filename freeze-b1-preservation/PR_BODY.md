## Why

`std/diagnostic.dag:289 diagnostics_fatal` / `:293 diagnostics_fatal_reason` are THE one authority for which element of a diagnostics chain is the cause (a `Diagnostic::Diagnostic` — a fatal; advisories ride AHEAD of it). resolve.dag:1543 states the contract verbatim: *"A consumer that read the HEAD as the cause read an advisory whenever a pending had been merged ahead: such a reading was never the contract."* Precedent: #15997 `loop_outcome_reason` already uses the authority.

The wc census probe (PR #15695 lane) observed head reasons outside the full 37-name resolve lane census at v17 (bare FAILs, `a=- s=T`). Leading candidate MECHANISM, labelled **source reading / HYPOTHESIS**: `declarer_refused_diagnostics` (resolve.dag:1436-1441) unconditionally prepends a refused provider row (`ModuleUnavailable`/`DeclarationRefused` rows, acquisition/ingest vocabulary) ahead of the in-census fatal `resolve_reason_declarer_refused`. This PR does NOT assert that mechanism; it stands on contract alone. Whether the mechanism fired stays a HYPOTHESIS until the later granted packet reports, per module, `fatal == ^resolve_reason_declarer_refused` (caret `==`, an executed shape).

## Scope

Every A-class consumer that reads a chain HEAD as THE cause — `reason:` / `cause:` / `at_atom:` (and the co-read `at_path:`) fields, plus the tokenize `refusal_of` observation — now routes through `diagnostics_fatal_reason(d: …)` / `diagnostics_fatal(d: …).at`. No new fold. `v2.std.diagnostic` was already imported by every touched file.

### A-site classification table (18 files; classification of each read site)

| file (src/v2/test/claim/) | fn | sites | classification |
|---|---|---|---|
| namespace_xl0/wildcard_arm_resolve.dag | wc_verdict_of | reason, at_atom | **cause read** (census refusal cause) |
| namespace_xl0/return_tail_position.dag | rtp_verdict_of | reason, at_atom | **cause read**; the FileRefused arm already used the authority (native_test_file_refusal_fatal_reason) — the asymmetry this PR removes |
| namespace_xl0/value_position_whole_read.dag | vpw_verdict_of | reason, at_atom | **cause read** |
| namespace_xl0/sequence_operand_resolve_refusal.dag | sqo_verdict_of | reason, at_atom | **cause read** |
| namespace_xl0/list_literal_value_lowering.dag | llv_self_verdict_of | reason, at_atom, at_path | **cause read** (all three fields must come from the SAME element — the fatal) |
| namespace_xl0/let_match_early_return.dag | lme_verdict_of | reason, at_atom | **cause read** |
| namespace_xl0/if_arm_and_statement_lowering_refusal.dag | iasl_verdict_of | reason, at_atom | **cause read** |
| namespace_xl0/else_arm_nested_if.dag | eam_verdict_of | reason, at_atom | **cause read** |
| namespace_xl0/call_argument_value_resolve_refusal.dag | cav_verdict_of | reason, at_atom | **cause read** |
| namespace_xl0/call_argument_resolve_refusal.dag | xl2_verdict_of | reason, at_atom | **cause read** |
| namespace_xl0/bound_value_resolve_refusal.dag | bvr_verdict_of | reason, at_atom | **cause read** |
| resolve/fold_handler_binder_resolve.dag | fhb_verdict_of | reason, at_atom | **cause read** |
| callexec/let_match_early_return_eval.dag | lmee_result | reason ×4 stages | **cause read** ("a refusal is reported where it happened"; the comment's wording 'its first reason' is position-language for the cause — no claim here deliberately asserts the head element) |
| body_lowering/function_value_body_route.dag | fvb_verdict_in + Fvb(Hiding)ContextRefused | reason×3, at_atom | **cause read** (file already imported diagnostics_fatal_reason for FvbEmitRefused — the split authority this PR unifies) |
| body_lowering/fold_operand_structure.dag | fos fold sites | reason ×2 | **cause read** |
| body_lowering/enclosing_expression_structure.dag | ees fold sites | reason ×2 | **cause read** |
| tokenize/lex_mode_stack.dag | refusal_of | reason ×6, at | **cause read** (claims assert specific lexer fatal reasons e1/e2/e5; none asserts 'the head' as such) |
| tokenize/lex_eof_mode_check.dag | refusal_of | reason ×6, at | **cause read** (same) |

No touched site deliberately asserts the head: each names a specific expected fatal reason. The tokenize/body_lowering sites were individually read to confirm this.

### Exclusions (named by symbol)

- `name_resolve.dag admit_imported_module_existence`'s head **re-stamp** (name_resolve.dag:892 at f08, :908 at main): a PRODUCER preserving `d.head.reason` while restamping locus — not a cause consumer. Excluded.
- `source_root_eval.dag` `head_reason:` field (:4032 at main): an **honest label** (head observation, not cause). Excluded.
- `lens_common/resolve_fixture.dag claim_refused_provider_of` (:246/:255): **fixture observation** of claim-authored singleton rows — safe today, same text pattern; left alone (owner decision on record).
- ce (`self_host/closure_emission.dag`): OUT of scope — already uses `diagnostics_fatal_reason` at both read sites (:363/:383). The arm-ORDER question of `closure_emission_first_refused` is tracked as a separate source-only note.

## Pre-registration (per claim; frozen at PR opening — no edits to expectations after any run)

Legend — (a) INTENDED verdict from the claim's contract; (b) LAST OBSERVED verdict + source; (c) PREDICTION after the repair + reason. Any departure from (c) is a FINDING to explain, never an expectation to edit.

**wc (namespace_xl0/wildcard_arm_resolve.dag), verdict fn wc_verdict_of — 4 claims.** Observed (b): FAIL at f08a6ae4 (matrix A) AND at 495da688 PR head (matrix B / packet cc1-15695-495d-v17) — the same 4:
- a_match_exhaustive_through_a_wildcard_arm_resolves — (a) PASS (b) FAIL A+B (c) FAIL -> PASS if the fatal-read hypothesis holds (its refusal cause is expected to be no fatal at all: the verdict must reach Resolved; reading the fatal instead of a prepended advisory is the change under test)
- a_wildcard_in_a_constructor_field_resolves — (a) PASS (b) FAIL A+B (c) FAIL -> PASS under the hypothesis
- a_named_binder_arm_still_binds — (a) PASS (b) FAIL A+B (c) FAIL -> PASS under the hypothesis
- an_undeclared_name_in_a_wildcard_arm_body_is_the_sole_refusal_at_its_atom — (a) PASS (b) FAIL A+B (c) FAIL -> PASS under the hypothesis; asserts reason==^resolve_reason_unbound_symbol + at_atom from the FATAL now

**rtp (namespace_xl0/return_tail_position.dag), verdict fn rtp_verdict_of — 17 claims.** Observed (b) at A/B (matrix A/B, v17): the 6 refusal claims PASS: a_return_inside_a_let_bound_if_refuses_rather_than_binding_its_operand, a_return_followed_by_a_statement_refuses_as_not_in_tail_position, a_return_before_another_statement_in_a_fn_value_refuses, an_else_less_if_that_falls_through_still_refuses_as_unlowered (FAIL at A only — flipped PASS between A and B; noted, unattributed), an_else_less_if_as_the_last_statement_still_refuses_as_unlowered, a_guard_in_a_non_tail_spine_still_refuses_as_unlowered — (a) PASS (c) PASS expected unchanged (chains with no pending advisory: fatal == head; a flip here would be a hidden red in reverse — FINDING).
The 11 that failed (b) FAIL A+B, (a) PASS, (c) FAIL -> PASS under the hypothesis: a_tail_return_of_a_fn_body_is_accepted, a_tail_return_after_a_let_is_accepted, a_tail_return_of_a_fn_value_block_body_is_accepted, a_tail_return_operand_is_read_and_refuses_unbound_at_its_atom, a_return_in_an_arm_of_a_tail_if_is_accepted, a_guard_followed_by_statements_is_accepted, a_guard_arm_return_operand_is_read_and_refuses_unbound_at_its_atom, the_statements_after_a_guard_are_read_and_refuse_unbound_at_their_atom, a_return_in_an_arm_of_a_tail_match_is_accepted, a_tail_match_arm_return_operand_is_read_and_refuses_unbound_at_its_atom, a_return_in_an_arm_of_a_match_nested_in_a_tail_match_arm_is_accepted.

**Every other touched claim (files: 10 other namespace_xl0 + resolve/fold_handler_binder_resolve, callexec/let_match_early_return_eval, body_lowering trio, tokenize pair — 109 test fns listed in the owner's inventory).** (b) NEVER EXECUTED (not in any run roster; v17 ran only cpr/rtp/ce, and the 15 cpr claims live in parse/condition_position_record_parse.dag, untouched here). (a) PASS each (that is what claims mean). (c) PASS, with the caveat 'no prior observation' — their first-ever verdicts land in CI / the later packet. Within these, the claims whose verdict path reads the token this PR re-routes AND assert on it (fvb's three at_atom/reason tests, llv's at_path tests, fhb sole_refusal, the xl0 'refuses... at its atom' tests, tokenize 'refuses_holds' tests) are exactly the hidden-red candidates if any fixture chain carries a pending advisory; acceptance-side tests ('...resolves', '...lexes') are unaffected by the change.

**ce (self_host/closure_emission.dag): OUT of scope — no edits.** Its two FAILs (a_closure_member_whose_body_does_not_lower_is_its_own_typed_refusal_holds, the_first_refusal_of_a_closure_is_the_first_refused_arms_cause_holds; FAIL A+B) keep their observations untouched; the arm-ORDER question of closure_emission_first_refused is tracked as a separate source-only note.

The wc 4 + rtp 11 FAIL -> PASS prediction IS the hypothesis this PR tests; it does not soften 'no edits to expectations' — a departure from (c) is a FINDING, and the packet (roster: wc + rtp at the repair head vs its base, hardened frame, outcome-join law, per-module fatal == ^resolve_reason_declarer_refused caret ==) is the single granted evidence run.

## CI reach

CI builds claim_batch (fleet-converge) and runs the bootstrap/claim jobs; which of these 18 files the CI claim roster actually executes is not enumerable from here — CI green/red on this head IS the statement of reach. Claim-level before/after evidence for the wc+rtp rosters comes from ONE later granted packet (hardened frame, outcome-join law, per-module `fatal == ^resolve_reason_declarer_refused` caret `==`), prepared after this PR is up.

## Follow-up census (owner: sleek-seal-226; TRIGGER: this PR lands)

C-class list (309 hits at main): `X.head.reason == ^caret` predicates with no tail tolerance — each to be classified as 'means the cause' (route via the authority) or 'deliberately asserts the head' (leave, annotate). Full machine list: HEADREADER_CENSUS.txt (618 lines both pins, sha256 d6c9d390…), held by the owner.
