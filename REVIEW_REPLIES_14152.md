# #14152 — replies to the five stale REQUEST_CHANGES

All five were written against superseded heads (81022@fa06f6b4, 81061@2751b200,
81093@5d750054, 81163@b5bb104e, 81191@60b2bd7f). Each reply names the commit that resolves
it; every claim below was re-checked against the tree, not remembered.

---

## 81022 (head fa06f6b4) — 3 findings

**F1 — "the refusal logic is in the .dag but never runs; the seed mirror is not updated."**
Resolved by `2751b200` (the regeneration commit) and re-established at every subsequent
regeneration, most recently `8d8d8aac0e1`. `src/v1/stage0/src/v1_compiler_infer.rs`
`unify_generics` now returns `Rc<UnifyGenericsAcc>` — 41 occurrences of the symbol in the
mirror at the current head, 0 at fa06f6b4. The mirrors in this PR are produced only by
regeneration (claim_executor to a fixed point), never hand-merged, so the two authorities
cannot drift. Your finding was correct at fa06f6b4; the row you quoted is the pre-heal text.

**F2 — "the new tests can't fail on the seed; accept-only, no discriminating red."**
Resolved by `5d750054b34` and `b5bb104e3ba`. The Rust controls now assert the fold's *own*
diagnostic, and the `.dag` reject rows no longer use a bare `blocking_count(...) > 0`: they
use `refusal_rows`, a count scoped to `diagnostic_class` **and** `subject_name`, where the
subject is the fold's own `concat(node_type_shape(actual), " (later argument)")` string —
measured by printing the blocking messages for these exact fixtures on the shipped seed
(throwaway probe, remote dispatch jb2430a08 at b5bb104e3ba), not guessed. The accepted
controls and the refusals are the same generic, same arity, same call shape; only the two
actuals' identity differs, so the differential is the discriminator.

**F3 (advisory) — "hand-written Rust has no receipt."**
The rung-drop row `dag/gunbc/rung_drop/unify_generics_rebind_unenforced_on_emitted_and_required_paths.dag`
is the carrier: it names the tests, states that no required lane runs them, and names the
rulings that withdrew those lanes (`v1_required_lanes_withdrawn`,
`rust_unit_tests_off_pull_requests`). See 81163 F3 for the deferral.

---

## 81061 (head 2751b200) — 3 findings

**F1 — "the failure-mode row describes the seed wrongly after your own heal commit."**
Accepted, and that is the rot you named. The receipt's rung statements are now generated
from the instrument rather than transcribed: the RFM row cites the fold by symbol and the
rung is stated as measured once, by the named harness, with the enforcement gap declared in
`gunbc.rung_drop` instead of a number carried in prose.

**F2 — "the quarantine's exit condition is met but neither discharged nor explained."**
Both. The v1 copy is dissolved with its two conjuncts verified on the shipped seed
(`cross_coproduct_probe_discharge_test.rs` requires the refusal to BE the rebind refusal,
not any blocking diagnostic, with `probe_same_tag_side_by_side_is_still_accepted` as its
positive control); the v2 copy stays enrolled and says so — see 81163 F1.

**F3 (advisory) — "hand-Rust tests have no receipt; the WIP: title suggests unfinished."**
Receipt: the rung-drop row above. The `WIP:` prefixes are commit-message hygiene on
intermediate commits, not PR state; the PR body carries the A/B suite results, the census
split by identity and the gate tables.

---

## 81093 (head 5d750054) — 3 findings

**F1 — "the rung drop is declared in the wrong place (a String inside the RFM row)."**
Resolved by `29f17f288d9` / `60b2bd7f5ad`. There is now its own roster file,
`dag/gunbc/rung_drop/unify_generics_rebind_unenforced_on_emitted_and_required_paths.dag`,
and the mis-citation is gone — `section 6b meta-obligation` no longer appears anywhere in
the RFM row (grep count 0); the meta-obligations are cited as §4b.

**F2 — "the rung-3 claim rests on controls no gate runs."**
Taken, and fixed in the direction you offered first: the row states the rung as measured
once and inherits the existing Rust-tests-off-the-merge-path drop by name rather than
claiming a standing rung 3. `NO DROP HAPPENED HERE` is in the row (60b2bd7f5ad) — see 81163
F2, which was the same objection from the other side.

**F3 — "the `.dag` reject claims don't discriminate."**
Accepted verbatim; resolved by `b5bb104e3ba`. `same_arm_two_distinct_coproduct_values_refuse`
and its siblings now call `refusal_rows(source, wanted_class, wanted_subject)` — a census
count keyed on `TypeMismatch` **plus** the fold's own subject string — and the module comment
records that the census is the whole compile's closure, which is exactly why a bare
`> 0` was not evidence. It also records that the first three subject strings were unverified
guesses and were replaced by measured ones (`Bool` prints as `Coproduct(Bool)` in this corpus).

---

## 81163 (head b5bb104e) — 3 findings

**F1 — "the v2 known-red quarantine and its probe are deleted without touching v2; the v2
replacement is a print that cannot fail."**
Accepted; this was a real defect in my ledger text and it is fixed in `60b2bd7f5ad`. Current
state, re-checked: `src/v2/test/claim/manual/constructor_tag_route_probe_red.dag` **exists**,
its `known_red_probe` row remains in `explicit_witness_admission.dag`, the v2 identity stays
in `transitional_admission_exception`, and the RFM receipt now reads *QUARANTINE GONE FOR THE
V1 COPY ONLY* with `THE V2 COPY STAYS ENROLLED` naming
`v2.compiler.infer infer_match_generic_formal` as unrepaired. The receipt also says plainly
that `locate_v2_infer_print` returns a String, asserts nothing, cannot go red, and therefore
**is not** the v2 replacement — so the print is no longer cited as discharging anything.
Only `dag/test/claim/constructor_tag_route_probe_red.dag` (the v1 copy) is deleted, and only
because its runtime-bool form cannot express the compile-time exit condition.

**F2 — "`declaration.previous: StructurallyGuaranteed` records a rung the class never held."**
Accepted; resolved by `60b2bd7f5ad`. The row now states `NO DROP HAPPENED HERE` and reports
the honest rung rather than a drop from a rung that was never held.

**F3 (advisory) — hand-Rust receipt names no owning lane.**
The rung-drop row is the carrier and names the withdrawing rulings. If a named owning lane is
required rather than the withdrawal record, say which roster should carry it and I will enrol
it there rather than invent one.

---

## 81191 (head 60b2bd7f) — 2 findings

**F1 — "the QUARANTINE GONE receipt contradicts the PR: the v2 copy is still there."**
Resolved by `29f17f288d9`, the commit titled for this review. The receipt now claims v1-only
dissolution, keeps `same_arm_one_t_two_coproducts_must_refuse` in the evidence list as the
live v2 RED, and keeps the v2 row and its `transitional_admission_exception` identity
enrolled. (Your read was against 60b2bd7f, which had repaired the v2 *file* but not the
receipt that said otherwise.)

**F2 — "carriers cite code by line number where a symbol exists."**
Resolved by `29f17f288d9`. `04_infer.dag:<n>` and `v1_compiler_infer.rs:<n>` citations are
gone from `dag/test/claim/*.dag`, the RFM row and the rung-drop row (grep count 0 at the
current head); they name `v1.compiler.infer` `unify_generics` / `type_mismatch_error`
instead.
