# Parse-Population Baseline Workloads — pointer

This is a **pointer**, not an authority. The two fixed benchmark populations
for the `emission-real-consumer-profiles` row — the selection rule, the
membership, each member's byte size and population rank, and the pinned tree
sha — have their single home on the typed `.dag` carrier:

- **Carrier:** `dag/gunbc/parse_population_baseline_roster.dag`
  (module `gunbc.parse_population_baseline_roster`)
- **Rostered claim:** `dag/test/claim/parse_population_baseline_roster_witness.dag`
  (module `test.claim.parse_population_baseline_roster_witness`)

Per `gunbc.design_argument::planning_has_one_home`, the `.dag` carrier is the
only planning authority for this data; this markdown keeps no copy of the
selection rule or the roster, so it cannot drift from it.

## What the carrier holds (read it there, not here)

- `parse_population_baseline_pinned_sha` — the tree sha the populations were
  selected and measured over.
- `parse_population_baseline_rows()` — the full 170-member roster as typed
  `KeyedRow<String, ParsePopulationRow>` (path, `ByteSize`, rank, population),
  built through `std.keyed_roster` so a repeated path **refuses** rather than
  silently unioning — disjointness and uniqueness are checked, not asserted.
- The selection rule and the reproduce command are stated in the carrier's own
  header comment.

The rostered claim executes the carrier and asserts cardinality (5, 165, 170),
per-population byte totals, and a **named negative control** (the same fold with
a trivial always-equal `key_eq` must refuse), so a short or empty roster cannot
pass (`gunbc.design_argument::checks_must_prove_they_measured`).

## Measured results

The executed measurement cells (wall, RSS, refusals, min/median of repeats, and
the load alongside each) are the evidence record in
`docs/plans/parse-population-baseline-measurement-report.md`. This file carries
no measurement results either.
