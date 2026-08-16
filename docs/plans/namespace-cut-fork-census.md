# Namespace cut — the multi-declarer fork census

**Status:** measured, undecided. No qualification has been applied to any name below.
**Instrument:** `gunbc compile --source-root dag --source-root src/v2 --target dag`,
baseline SHA `0fad0cec741`, 2360 resolved sources / 1353 indexed-not-compiled.
Declarer counts from a `^type` / `^fn|func` index validated in both directions
(unique name → 1; known v1/v2 twin → 2; independently reproduced `row` = 2).

## Why this census exists, and why it is time-limited

The import cut made every cross-module reference refuse. That is expensive, but it
has one free side effect worth banking before it is spent: **a name declared in two
places is currently visible, and qualification is exactly the act that hides it
again.** After the corpus is qualified, `a.X` and `b.X` coexist silently forever —
no diagnostic points at them, and nothing distinguishes "two subjects that share a
word" from "one concept forked in two" (DESIGN §3).

So the 19 multi-declarer names are recorded here **before** any of them is qualified.
Qualification resolves the *resolution* problem and preserves the *modeling* one;
that is the correct division of labour, but only if the modeling residue is written
down rather than absorbed.

Weight: 961 of the 3423 surviving diagnostics, of which two names carry 926.

## Class 1 — deliberate fixtures (qualify; nothing to decide)

These exist *to be* ambiguous. Qualifying them is correct and changes no meaning.

| name | declarers | note |
|---|---|---|
| `CensusProbeSubject` | 2 | `record_construction_census/specimens.dag` + `homonym.dag` — the second file is literally named for the property |
| `SharedBareArm` | 2 | `decl_facts_reflection/ambiguous_shared_a.dag` + `_b.dag` |

## Class 2 — homonyms and copied helpers (qualify, but §2 applies to one)

Same word, unrelated subjects, or a utility duplicated per consumer. No §3 fork of
meaning — but a copied body is still redundancy.

| name | declarers | reading |
|---|---|---|
| `list_contains_string` | **7** | **identical signature `(xs: List<String>, needle: String) -> Bool` in all seven.** Not a fork of meaning — one utility copy-pasted seven times. §2 redundancy, own decision, independent of this cut |
| `census_of` | 5 | four test-local `(source: String) -> CompileDiagnostic…` helpers + one unrelated `(rows) -> WitnessClassCensus` |
| `observation_for` | 3 | three `spark_*` modules, plausibly one concept — inspect before assuming homonym |
| `reasons_contain` | 3 | two test-local + `frontier_ingestion_probe` |
| `list_contains_path` | 2 | test-local + `dag_compile_clean_shard_roster` |
| `refusal_count` | 2 | different argument types (`PartAdmission` vs `RefusalAggregate`) |
| `census_is_complete` | 2 | different argument types |
| `row` | 2 | **579 diagnostics.** `md_helpers.row(cells)` vs a 3-arg test-local `row(host, endpoint, key)`. Different arity, different return type. The test-local one should resolve lexically inside its own file; everything else wants `gunbc.plans.md_helpers.row` |

## Class 3 — candidate §3 forks (do NOT qualify before a ruling)

Same name, same *neighbourhood* of meaning, two authorities. Qualification here
would cement the fork by making it invisible.

| name | declarers | why it is a candidate |
|---|---|---|
| `OccurrenceId` | 3 (2 + 1 alias) | **347 diagnostics.** `std/occurrence_identity` `{value: Int}` (SOURCE occurrence) vs `std/observation` `{attempt, id}` (observation occurrence). `src/v2/std/node.dag` is an alias of the first, not a third declaration. DESIGN already warns about sharing one `Int` space across two occurrence subjects — this is a second instance of that hazard, one layer up. Refusing consumers are overwhelmingly `occurrence_binding_*`, i.e. source occurrences, so the *target* is determinable even though the *fork* is not resolved by choosing it |
| `TeardownRefusalCause` | 2 | `gunbc/membership_reconcile` vs `gunbc/live_deploy/spec`. DESIGN already records `live_deploy` re-grounding onto `membership_reconcile`'s ownership as pass 2 with a declared dissolution trigger — so this is a **known pending consolidation surfacing as a name collision**, not a new finding |
| `ObservationProducer` | 2 | `std/observation` vs `gunbc/auth/github_apps` — a consumer redeclaring a std authority |
| `ObservationGrain` | 2 | `std/observation` vs an `extdeps/physics` paper module |
| `IntegerOverflow` | 2 | `std/checked_arithmetic` vs `extdeps/languages/rust/primitives`. Plausibly legitimate: the std concept and the Rust-realization concept are different subjects under the interface/realization split |
| `PowerRefusal` | 2 | `product/rack_power` vs `product/compute_board/power_distribution` |
| `SparkManagedAccessStanding` | 2 | `spark_managed_access_bootstrap` vs `fleet_converge_cli` |
| `DimmSlotPosition` | 2 | `extdeps/cpu/…/platform` vs `product/altra_motherboard/minimal_design` |

## What this census does not establish

- **It is not a fork verdict.** Two declarations of one word can be two subjects.
  Each Class 3 row needs its bodies compared before anything is called a fork; only
  `OccurrenceId` has had that done here.
- **It sees only names that currently refuse.** A name declared twice whose every
  reference already resolves is absent from this list. The census is bounded by the
  refusal population, not by the corpus — so it is a lower bound on multi-declarer
  names, never an inventory of them.
- **`^type` cannot see coproduct arms.** 52 type-position names (138 diagnostics)
  have zero declarers by this index and read as arms — `Do`, `ObservationUnavailable`,
  `Refused`, `IdentityUnknown`, `Missing`, `Failed`. That is a limit of the
  instrument, not a claim those names are undeclared.

## Sequencing consequence

Nothing mechanical should start before Class 3 is dispositioned. `row` and
`OccurrenceId` alone are 926 of the 961 decision-weighted diagnostics, and
`OccurrenceId` is the one whose qualification target is determinable while its
modeling question stays open — which is exactly the shape that gets silently
resolved by a mechanical pass if the pass runs first.
