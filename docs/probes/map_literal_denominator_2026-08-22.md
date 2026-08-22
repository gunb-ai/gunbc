# The Map-literal migration denominator, pre-registered (2026-08-22)

The `Map { lookup: … }` literal population, enumerated **before** the migration that will erase it.
This is a measurement, not a diff: no literal is migrated here.

**Why it is pre-registered rather than checked afterwards.** The literal half of the T3 population
has no refusal-shaped oracle — literals emit byte-identically no matter what the declared type says,
and no `std`-side edit makes them red (measured:
[`t3_collection_realization_2026-08-22/denominator_experiment.md`](t3_collection_realization_2026-08-22/denominator_experiment.md)).
So "did the migration get all of them" is answerable only by source enumeration, and the
pre-migration population is the one thing that stops existing the moment the migration lands.

| | |
|---|---|
| ref | `967b5bc1b92ee66250e06a7870c132b48a16b80a` (`origin/main`) |
| roster | [`map_literal_denominator_2026-08-22/roster_967b5bc1b92.tsv`](map_literal_denominator_2026-08-22/roster_967b5bc1b92.tsv) — 145 rows |
| extractor | brace-balanced slice from each `(^\|[^A-Za-z_])Map \{` occurrence, read from the ref via `git show`, not from a working tree |
| identity | `(file, enclosing_symbol, ordinal_within_symbol)` — **collision-free across all 145** |

## The population

- **145 occurrences, 103 files, all under `src/v2`** — zero in `dag/`, zero in `src/v1`.
- **One shape.** Every one of the 145 has exactly one top-level field: `lookup`. There is no second
  literal form to handle.
- **114 distinct normalized bodies.** 31 occurrences duplicate another's text, which is why a body
  digest is carried *beside* the identity key and never as the key.

## The join rule — identity, not count

After the migration, re-run the same extractor at the new ref and require **all three**:

1. **Completeness.** `post ∩ pre = ∅` — every pre-registered `(file, symbol, ordinal)` is gone.
2. **No regression.** `post \ pre ⊆ {the gate control}` — no *new* literal appears anywhere.
3. **Terminal count.** `|post| = 1`, that one being
   `v2.test.claim.map_carrier_shape_gate` `record_shaped_map_after_insert`.

**A count equality would pass a migration that removed 145 sites and introduced 145 others**, which
is the failure a codemod actually has. Rules 1 and 2 are what make that unrepresentable; rule 3 is a
consequence, stated separately so a reader cannot mistake it for the test.

**The intended survivor is not an exception to the migration.** The gate witness (on
`session/bright-newt-108-step1`, so *not* at this ref and *not* in the 145) constructs a
record-shaped map deliberately, as the discriminating arm that reds if the `map_insert` delegate
lands while any real literal survives. It migrates only by inversion, when the delegate lands.

## Coverage bound — what this extractor cannot see, measured rather than asserted

Each row below is a probe that was run, not a caveat that was imagined.

| class | probe | result |
|---|---|---|
| suffix collision inflating the count | naive `Map \{` vs word-bounded | 190 → **145**; the 45 are `PureElementwiseMap {`, `DataDependentGatherMap {`, `PackageContactMap {`, … — over-count avoided, nothing missed |
| literal split across lines (`Map` then `{`) | every line ending in bare `Map`, plus its successor | 37 such lines, **all 37 followed by `}`** — they are import-list members; **zero** split literals |
| return-type annotation misread as a literal | `-> Map \{` | **0** |
| field/param annotation misread as a literal | `: Map \{` | 107, and all sampled are value positions (`facts: Map { … }`) — literals, correctly counted |
| the carrier reached under an alias spelling | aliases whose RHS is `Map<…>` → do they get `X {` construction? | 4 aliases (`DecodePositions`, `GenericSubstitution`, `LookupTable`, `SKeyMap`); 3 have `X {` occurrences, and **all 3 are `fn … -> X {` signatures, not literals** — 0 missed |
| the algebra spelled directly | `PartialFunction \{` | **0** |
| qualified spellings | `std.types.Map \{`, `v2.std.collection.Map \{`, `types.Map \{` | **0** each |

**The residual, stated plainly:** this extractor sees *spellings at the construction site*. A
Map-carrier value built without the `Map` spelling there would be invisible to it. The alias probe
above is the measured test of exactly that class and it found four candidate spellings and zero
literals — which bounds the class rather than closing it, because a future alias would reopen it. A
map produced by a helper is *not* in this residual: the helper's own literal is one of the 145, so
the population is closed under that indirection.

## What this does not establish

- Not a claim that all 145 are *finite* constructions. That classification is the roster in
  [`t3_collection_realization_2026-08-22/decomposition_scope.md`](t3_collection_realization_2026-08-22/decomposition_scope.md),
  where the 97 `lookup: <named fn>` delegates were **sampled, not exhaustively read**. This document
  enumerates the population; it does not re-adjudicate what each member is.
- Not a migration plan, and not an ordering claim. The order of record is that the literals migrate
  before the `map_insert` delegate, established by execution rather than by preference:
  the delegate removes the only arm that serves record-shaped maps
  (14 floor witnesses, gunbc#8887).
