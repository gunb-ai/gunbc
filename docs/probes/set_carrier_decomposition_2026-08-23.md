# The predicate half of the Set/Map carrier fork, decomposed (2026-08-23)

**Work item:** `node://adhoc-66adece2-31f`. **Session:** `silent-gull-867`.
Measurement this rests on and does not restate:
[`t3_collection_realization_2026-08-22.md`](t3_collection_realization_2026-08-22.md) (the E0308 half)
and [`board_2026-08-23/README.md`](board_2026-08-23/README.md) (the retained log this brief's 27
blocks were partitioned from).

## The 27 blocks, at `.dag` grain

Subject `src/v2/compiler/03_ingest.dag`, ref `98b18cdc81e`, board `CARGO_ERROR_TOTAL=330`. The brief
names 27 blocks in two shapes; both are the same fork read from two positions.

| shape | code | blocks | what rustc saw |
|---|---|---:|---|
| the algebra name leaks as a Rust type | `E0560` | 17 | `struct Rc<PointwisePower<_>> has no field named member` (8) · `struct Rc<PartialFunction<_, _>> has no field named lookup` (9) |
| the carrier's own field read off the realized type | `E0609` | 10 | `no field member on type Rc<OrdSet<…>>` |

Both come from `std.types` naming one spelling for two carriers. A **type** position takes the
`container_template_alias_rows` hop on to the target inhabitant row and lowers `Set<T>` to
`im::OrdSet<T>`; a **record-literal** position takes the `container_template_algebra` hop and stops
at the modeled algebra struct, `Rc<PointwisePower<_>>`. Where one compiled closure holds both, the
two meet as a refusal.

## What this change does

Every corpus site that **constructs by characteristic function** or **reads `.member`** now names
`std.algebra` `PointwisePower` directly, under its own authority, instead of borrowing `Set`. The
populations were read individually, not sampled:

| | count |
|---|---:|
| `Set { member: … }` literals in `src/v2` before | 37 |
| — of them, characteristic functions | 37 |
| — of them, finite enumerations mis-spelled | 0 |
| `Set { … }` literals in `src/v2` after | 0 |
| `.member(` call sites re-pointed | 26 |
| `Set<…>` declarations re-typed to `PointwisePower<…>` | 128 occurrences across 20 files |

Nothing in the finite-`Set` population moved: `std.graph` `visited`, `std.authorization_profile`
`EnumeratedAudience.members`, `std.occurrence_binding_candidates`, `std.syllogism`,
`gunbc.package_delivery`, and the `dag/`-rooted emit templates keep `Set` and keep the `OrdSet`
realization, because every one of them is fed by `empty_set` / `set_insert` and read by
`set_contains` — the finite surface, never `member`.

This closes the 18 blocks of the 27 that are Set-shaped (8 `E0560` + all 10 `E0609`). The 9
remaining are `Map { lookup: … }`, and they are **not** closed here — see below.

## What is deliberately left open, and why

1. **The Map half.** `Map` needs no retarget — `PartialFunction`'s template roster already *is* the
   finite map surface — but its 145 corpus literals do need re-authoring onto the construction route
   `std.primitives` already declares (`empty_map_contract`, `map_insert_contract`; the second is
   bound in `v2.std.collection` `map_insert_host_binding` and then not used, the function writing a
   closure chain instead). 97 of those 145 are `lookup: <named fn>` delegates whose key list has to
   be read out of the delegate body one at a time. That is a lane, not a rider on this one.
2. **The alias itself.** `std.types` still reads `type Set<element> = PointwisePower<element>`, so
   `Set { member: … }` is still *writable* — nothing refuses a new one. Making it unwritable is a
   retarget of that row onto the finite carrier, which is a `std` authority change whose only
   verification is a whole-corpus compile, and which has to move together with the Map row. The
   obligation is recorded on the declaration itself (`dag/std/types.dag`, the annotation above
   `type Set`), with the next-rung trigger named there: the Map-side decomposition landing. Until
   then the class sits at **mitigatable** (DESIGN §4b) and only review sees a regression.

So the honest statement of the rung: this change removes the fork's instances on the Set side and
does not yet make the Set side's invalid state unwritable.

## Controls

- **The re-typed population is exactly the predicate population.** Two independent instruments
  agree: a grep for `Set {` literals (37, each read) and a grep for `.member(` consumers (26, each
  read). No site was found that constructs a `Set` literal and then enumerates it, and no finite-
  surface call (`set_contains` / `set_insert` / `empty_set`) appears in any re-typed declaration's
  module against a re-typed value.
- **The rewrite could not touch string data or prose.** The edit was applied outside double-quoted
  spans and outside `//` annotation lines; the two `"Set"` string literals in
  `v2.extdeps.languages.rust` (`target_collection_set_carrier` rows) are unchanged, and the two
  prose blocks that cite the grounding (`v2.std.grammar`'s sync-token note and
  `v2.compiler.02_parse`'s convergence-measure note) were updated by hand, not by the rewrite.
- **Not verified here:** the emitted-Rust block count. Re-measuring the board requires the probe
  dispatch, and this change is landed against the `.dag` acceptance path (`gunbc compile --entry
  src/v2/compiler/03_ingest.dag`) rather than against a re-run of `curated_cargo_probe_one.sh`. The
  claim "18 of 27 close" is derived from the site-to-literal correspondence above, and is a
  prediction about the next board, not a measurement of one.
