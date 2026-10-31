# Text port: route rules (DESIGN, no applies)

Input: `scratch/text-port/host_text_site_inventory.md`. It lists 118 ambient call sites of `split`, `replace`, `starts_with` and `ends_with` at origin/main 500d8491b2a8. That list is discovery only. The port's completeness evidence is its own compile: any crossing the inventory missed must refuse when the port lands. The count proves nothing (sleek-owl-397, via tidy-ant-251 msg_673c6d23).

Authorities read for this draft: `std.host_text` (module header, `chars`, `text_of_chars`); `v2.std.text` (`string_split`, `string_replace`); `std.coercion` (`unicode_scalar_unfold`, `unicode_scalar_fold`); `v1.compiler.infer` (`text_crossing_or_type_mismatch_error`, `host_text_crossing_call`, `host_text_crossing_refused`, `inference_error`); DESIGN section 4, ruling `text_crossings_go_through_the_unfold`.

## 0. A conflict to rule before anything else

`std.host_text`'s own header says: "Only an operation that needs the host carrier lives here: equality, split, join, replacement and folds over text are structural and stay with structural text." My G2 section B proposed `split`, `replace` and `starts_with` as `std.host_text` declarations. Those two cannot both stand.

These rules follow the module header, which is the authority already landed: the ops are structural, and host text reaches them only by crossing. Section B's `split` and `replace` rows must be withdrawn, or the header must be re-ruled. **Ask: confirm the header governs.**

## R0. The call-argument InternalError arm (first; it gates every class RED)

**Observed.** fierce-ibex-674's wall witness (`dag/test/claim/host_text_structural_to_host_wall.dag`, #15220) gets two different diagnostic classes for the same mistake:
- an implicit crossing at a RETURN refuses with class `TextCrossingHasNoImplicitRoute`, naming the route;
- the same crossing at a CALL ARGUMENT refuses with class `InternalError`.

The witness deliberately asserts only "refuses" at the argument position.

**Read so far (source inspection, not yet executed):**
- The argument checker in `v1.compiler.infer` emits the typed `TextCrossingHasNoImplicitRoute` for host text into a structural formal (it calls `text_crossing_route_error` directly). It also emits it for other mismatches through `text_crossing_or_type_mismatch_error`. So the ordinary direct-call path is already typed.
- `InternalError` is what `inference_error` mints. On the crossing path its caller is `host_text_crossing_refused`. All seven conversion-plan refusals in `host_text_crossing_call` go through it: no plan, several plans, absent, ambiguous, endpoint mismatch, endpoint not text, route resolved elsewhere. Each is a real, typed refusal reason flattened into an untyped class.
- Builtin (ambient) call arguments are typed by nothing (population 3). So a structural value at an ambient `split(...)` argument plausibly reaches neither typed arm.

**Rule.** First, one normal run of the witness's argument row with the diagnostic printed, to name which producer fires. This is a measurement that falsifies the reading above, not a census. Then repair at that producer: each plan-refusal reason becomes a typed `CompilerDiagnostic` variant. Either extend `TextCrossingHasNoImplicitRoute` with a reason, or add one variant for a plan that does not run. That choice is a §3 naming question for the reviewer.

After R0, the witness row is upgraded to assert the class. That upgrade is R0's RED, with the existing no-crossing control kept. No port PR asserts a crossing class before R0 lands.

## R1. Which operation a site calls

The operation follows the value's representation. It never follows the spelling.

| Site's value (read from the PR-head compile, not the grep) | Route |
|---|---|
| structural text (`v2.std.text String` = `FreeMonoid<Char>`) | the structural op: `string_split(source, delimiter: FreeSemigroup<Char>)` or `string_replace(source, needle: FreeSemigroup<Char>, replacement)`. No crossing. |
| host text (kernel `String` / `std.host_text HostText`) | an explicit crossing to structural text through `chars` (`std.coercion unicode_scalar_unfold`), then the structural op. Where the consumer needs host text back, the result goes through `text_of_chars` (`unicode_scalar_fold`). Each crossing is a call written in the source; none is implicit. |
| neither, or unidentified (population 2) | refuse. Classify the site under its own population before porting it. |

**No fallback.** A site whose value the compile cannot place refuses. It is never ported by spelling.

**Delimiter / needle.** The structural ops take `FreeSemigroup<Char>`, which is non-empty. A literal delimiter introduces through the literal unfold row (`literal_homomorphism_rows` `UnicodeScalarSequenceUnfold`). A non-literal delimiter needs a constructed non-emptiness, per ruling `nonemptiness_is_constructed_or_proved`, or the site refuses. There is no default and no silent empty-delimiter case. The seed's behaviour on an empty delimiter is a correction row, one row per divergence (G2 ruling A, condition 5).

**Cost.** A host-text site now pays one unfold, plus one fold where host text is returned: linear in the text, where the host op was also linear. The cost shape is the same; the constant is not. Two exceptions:
- A crossing inside a loop over the same text is authored duplication (DESIGN section 2, question 1). Hoist the unfold to the value's producer; do not cache it.
- A site where both crossings sit on a hot path is the trigger to convert that value's PRODUCER to structural text. Converting at the producer removes the crossing; a host-realized op beside the structural one would be a second path.

## R2. starts_with / ends_with

**Correction.** The earlier draft said there was no structural prefix predicate; my DFS search missed one. `std.algebra` `list_starts_with<T>(xs, prefix, eq: fn(T, T) -> Bool)` already exists and takes element equality as a declared parameter. It has three call sites. Only the suffix half is missing.

**Ruled (tidy-ant-251, msg_2797b012):** the generic pair lives in `std.algebra`, with element equality as a declared parameter.
- The prefix predicate is `list_starts_with`, which already exists.
- The PR adds the missing `list_ends_with<T>(xs, suffix, eq)` beside it.
- The same PR repoints each specialised sibling to the pair, or keeps it with a stated reason.

Fork census for that PR (grep + inspection):
- `std.effect_grant` `path_is_prefix`
- `std.occurrence_identity` `occurrence_id_list_is_prefix_of`
- `std.materialization_ladder` `frame_path_is_prefix`
- `gunbc.publication_policy` `segment_prefix_is_prefix`
- `std.declaring_identity_spelling` `authored_is_suffix_of_canonical` (suffix; read before repointing)

Text `starts_with` and `ends_with` sites then route through `chars`, as in R1.

## R3. Per-port-PR acceptance (tidy-ant-251 ruling, msg_68a9518b)

- **v2-bootstrap** passes at the PR head.
- **An executed RED plus a positive control.** The RED: an unimported use of each ported name refuses with `resolve_reason_unbound_symbol`, and a structural value at a host op refuses with the class R0 established. The control: the ported, imported form is accepted.
- **An exact-head regeneration fixed point** whenever mirrors move.
- **Side-chat LAND.**
- **The PR body states it:** completeness is this compile. A missed crossing refuses on landing; the inventory's count is not evidence.

## Order

1. Confirm section 0.
2. R0: the measurement, then the typed arm.
3. R2's home ruling.
4. R1 ports, grouped by file. Applies wait on zesty-badger-599's sweep-demand fix.
