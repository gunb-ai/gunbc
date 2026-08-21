# Step-1 census: mechanism proven end to end, instrument KNOWN WRONG, no count published (2026-08-21)

**Session:** `royal-dove-436`. **Work item:** `node://adhoc-c735d227-60b`.

**Read this before the TSV beside it.** The first reading of this census is **wrong**, it is published
here **as** a wrong reading, and **no divergence count from it may be quoted anywhere**. It is kept
because what it demonstrates is worth more than the number it failed to produce.

## What IS established, by execution

`gunbc run --source-root dag --source-root src/v1 --source-root src/v2 --entry
src/v2/workflow/carrier_realization_census.dag --function census_smoke_receipt` runs the whole chain:
`Filesystem.Read` of the subject sources → `v1.compiler.compile` `front_end_sources` → a walk of the
resulting v1 tree → both answers per occurrence → TSV out. 271 lines of receipt, produced from
outside the v1 seed with **no edit to it**.

That closes the shape question for good: **(b′) works.** A module outside `src/v1` can import
`v1.compiler.05_emit_rust` `is_host_text_carrier_type`, `v1.compiler.coercion`
`type_realization_decision` / `type_reference_decl_file`, `v1.compiler.compile` `front_end_sources`,
and `v1.compiler.trait_derive_emit` `v1_item_field_type_exprs`, and evaluate them over a real subject
closure. It also re-confirms by execution that there was never a visibility change to make.

## What is WRONG with the reading, named exactly

The receipt classifies **every** row `DivergesWithExactIdentity`. That is not a finding; it is two
defects in the instrument:

1. **The comparison is malformed.** `legacy_base_of` returns `""` for any occurrence the text-carrier
   short-circuit does not claim. `""` is not "the legacy answer" — it is "not the text
   short-circuit". Comparing it against the authority's `<structural>` makes every non-text
   occurrence a spurious divergence. The legacy base for those occurrences is whatever the
   checkpoint/container route yields, which this version never computes.
2. **The occurrence set is wrong.** The `authored_name` column contains `T` (14), `fn` (12), `R` (3),
   `M` (3) — generic parameter names and the `fn` keyword, not carrier type references.
   `v1_item_field_type_exprs` over a `type` declaration like `Magma` yields its function-typed
   children, and `item.params` over a `type` declaration yields its **generic** parameters, not a
   function signature's parameters. The walk never reached a carrier reference at all.

## Why this is recorded rather than quietly fixed

**This is the traversal drift the calibration control exists to catch, and it was caught on the first
reading rather than after publication.** The design named the risk in the abstract — "the walk may
visit occurrences the emitter never renders, or miss ones it does; a subtly wrong occurrence set
makes every count wrong while looking perfectly well-formed" — and the very first run produced
exactly that: 271 well-formed rows, a clean TSV, and a 100% divergence rate that is an artifact of
the instrument.

Had this walked a plausible-looking subset instead of an obviously wrong one, the failure would have
been a believable number. It is worth stating plainly: **the reason this was caught is that the
defect was gross, not that the process caught a subtle one.** The calibration control against the
known 25 is what would catch a subtle one, and it has not run yet.

## Standing constraint, unchanged

No count from this census is publishable until its diagnostic-producing
`DivergesWithExactIdentity` subset **equals the 25 arm-A sites** of
[`../t2_t3_realization_route_2026-08-21/arbiter_arms.tsv`](../t2_t3_realization_route_2026-08-21/arbiter_arms.tsv),
joined by source declaration + enclosing emitted declaration + operation, never by line. This reading
does not approach that bar and is not offered as approaching it.

Next: compute the real legacy base for non-text occurrences, and reach actual carrier type-reference
nodes rather than generic-parameter and `fn`-keyword nodes.
