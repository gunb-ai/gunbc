# Step-1 census: two readings, neither publishable — and the second is the dangerous one (2026-08-21)

**Session:** `royal-dove-436`. **Work item:** `node://adhoc-c735d227-60b`.

**No divergence count in this directory may be quoted as a result.** Neither reading has passed the
calibration control. The second reading is recorded because it is *plausible*, and a plausible
uncalibrated number is the specific hazard this lane was warned about.

## What IS established, by execution

`gunbc run --source-root dag --source-root src/v1 --source-root src/v2 --entry
src/v2/workflow/carrier_realization_census.dag --function census_smoke_receipt` drives the whole
chain: `Filesystem.Read` → `v1.compiler.compile` `front_end_sources` → a walk of the resulting v1
tree → both answers per occurrence → TSV. From **outside** the v1 seed, with **no edit to it**.

That closes the shape question: **(b′) works**, and it re-confirms by execution that there was never
a visibility change to make.

## Reading 1 — KNOWN WRONG, grossly

271 rows, **every one** `DivergesWithExactIdentity`. Two instrument defects:

1. **Malformed comparison.** `legacy_base_of` returned `""` for any occurrence the text short-circuit
   did not claim — `""` standing in for "the legacy answer" when it meant "I did not compute one".
   That is a state-space conflation, and it *answers* instead of refusing: the absorbing-fallback
   shape (DESIGN §5) relocated into the measuring apparatus.
2. **Wrong occurrence set.** `authored_name` held `T` (14), `fn` (12), `R` (3), `M` (3) — generic
   parameter names and the `fn` keyword. `v1_item_field_type_exprs` over a `type` declaration yields
   its function-typed children, and `item.params` over one yields **generic** parameters. The walk
   never reached a carrier reference.

Kept as [`smoke_first_reading_KNOWN_WRONG.tsv`](smoke_first_reading_KNOWN_WRONG.tsv).

## Reading 2 — plausible, differentiated, and STILL NOT PUBLISHABLE

Both defects repaired: the outcome is now **structural** (`legacy_claims_text && !authority_realizes`)
rather than string equality — comparing spellings made `String` equal `String` and the divergence
being measured compared *equal* — and the occurrence set is type-declaration fields (via
`child_type_node`) plus function-signature parameters guarded on `item.body != none`.

```
186 Agrees      3 DivergesWithExactIdentity      0 IdentityUnavailable

std.string_type  string_lex_compare                  fn_signature_param  String
                 dag/std/string_type.dag   legacy=String   authority=<structural>   Diverges
std.string_type  string_lex_compare                  fn_signature_param  String   (same)
std.string_type  string_is_lexicographically_before  fn_signature_param  String   (same)
```

The three divergences are **exactly the mechanism this lane traced**: a `String`-spelled parameter
whose resolved declaration is `dag/std/string_type.dag` — one of the two modules
`structural_declaration_modules_for` enrolls — where the short-circuit renders host `String` while
the authority answers `Unrealized`. Measured per occurrence, from outside the seed.

**And that is precisely why it must not be reported.** The reading is *believable*: real type names
(`String` 51, `fn` 49, `T` 21, `List` 13, `Int` 11), a sane distribution, no degenerate column, and
divergences that land on the right mechanism in the right module. Reading 1 was caught because its
defect was **gross**. Nothing in reading 2's shape would announce a subtle one.

**The open question it cannot answer about itself:** 51 `String` occurrences, 3 divergences. If the
short-circuit fires on every `String`-spelled reference and the roster enrolls that declaration, why
do 48 agree? Either their `decl_file` resolves to a module the roster does not enroll — plausible and
benign — or the walk is reaching them differently from the way the emitter does, which is traversal
drift producing a believable number. **This census cannot distinguish those two from the inside.**

## The bar, unchanged

No count publishes until the diagnostic-producing `DivergesWithExactIdentity` subset **equals the 25
arm-A sites** of
[`../t2_t3_realization_route_2026-08-21/arbiter_arms.tsv`](../t2_t3_realization_route_2026-08-21/arbiter_arms.tsv),
joined by source declaration + enclosing emitted declaration + operation, never by line. That control
is now the only thing standing between this instrument and a believable wrong number — including a
number that looks right.

Next: run against the 03_ingest closure, and calibrate. Until then, 3 is not a finding.
