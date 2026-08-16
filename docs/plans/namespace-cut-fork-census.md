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
| `row` | 2 | **579 diagnostics. VERIFIED mechanical.** A markdown table row vs an SSH known-hosts row — different parameters, return types and domains; zero shared concept. Measured: **zero** of the 579 come from the file declaring the local `row`, confirming it resolves lexically and never entered the census; every top refusing file is `dag/gunbc/plans/*`, exactly `md_helpers`' domain. The open question is the *form*, not the target — see below |

## Class 3 — candidate §3 forks (do NOT qualify before a ruling)

Same name, same *neighbourhood* of meaning, two authorities. Qualification here
would cement the fork by making it invisible.

| name | declarers | why it is a candidate |
|---|---|---|
| ~~`OccurrenceId`~~ | 3 (2 + 1 alias) | **RESOLVED — not a fork, and not a decision. Moved to mechanical.** DESIGN already rules this shape in the fleet-reconcile spine thread, and it rules *for* distinctness: `std.occurrence_identity` is "specifically about SOURCE occurrences… its allocator is coupled to `AuthoredTokenOrdinalSpace`", and "sharing one `Int` space across two subjects would let `occurrence_id_eq` answer `true` on a numeric coincidence — the cross-family comparison the ContentHash family grounding closed." So separate carriers for separate subjects is **correct**, and collapsing them would reintroduce the defect that grounding closed. The unification that *is* owed is already registered as a trigger ("one subject-parameterized minted-identity carrier, the `Vendor<Domain>` shape") and belongs to a modeling lane, not to an import deletion. **Consequence for this cut:** each of the 347 sites means exactly one subject, determined by what the site is about — qualify per-site, never unify, never pick a winner. A site that is genuinely ambiguous about which occurrence space it inhabits is a real finding worth reporting individually, because that is the wild instance of the defect DESIGN warns about |
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

## The open question is the qualification FORM, not the target

Once `row` and `OccurrenceId` moved to mechanical, the decision bucket collapsed to
**seventeen names at 1–5 diagnostics each, ~35 diagnostics total** — and the gate on
starting mechanical work lifts with it.

What replaced it is a question the corpus cannot answer from evidence, because every
reference is bare and **no qualified call site exists anywhere in the tree**. The
`row` consumers are *siblings* of their declarer (`gunbc.plans.X` calling
`gunbc.plans.md_helpers.row`), which is the case DESIGN describes as "a sibling module
is visible, its members projected" — implying one projection rather than a full path:

```
FORM A   md_helpers.row(..)                    sibling projection; design-conformant reading, UNPROVEN
FORM B   gunbc.plans.md_helpers.row(..)        full path; PROVEN by the LiveTreeDisposition probe
```

579 sites ride on this, so it is probed rather than assumed — one file per form, one
census run decides both. Note that bare `row` was only ever resolving through the
import list; under namespace-only resolution a sibling's member was always going to
need a projection, so this is the cut exposing a form the corpus never had to state.

## Sequencing consequence

Class 3 is now eight names, all small, none blocking. The rule that survives is
narrower and sharper than "settle the decisions first": **a mechanical pass must not
run over a name whose declarers mean different subjects**, because qualification is
precisely what makes that difference unobservable again. `OccurrenceId` is the live
example — mechanical per-site, but only because each site's subject is determined by
what the site is about, and a site that cannot say which occurrence space it inhabits
must be reported rather than qualified.
