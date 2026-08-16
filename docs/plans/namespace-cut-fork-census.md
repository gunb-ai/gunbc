# Namespace cut — the multi-declarer fork census

**Status:** pass 1 applied (1504 edits, 632 files). Remaining work is blocked on a
compiler defect, not on effort — see the result and boundary at the end.
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

## `OccurrenceId` — the 347 sites resolve by file, and only 3 cross subjects

The ruling says qualify per-site to the subject the site is about. Measured, that is
far more tractable than the count suggests: **17 distinct files carry all 347**, and
scoring each for source-occurrence vocabulary (`occurrence_binding`, `OccurrenceCategory`,
`ContainmentPath`, `AuthoredTokenOrdinal`) against observation vocabulary (`attempt`,
`ObservationProducer`, `std.observation`) separates them cleanly:

```
168  occurrence_binding_candidates_witness_test   src=30  obs=0
 29  occurrence_binding_resolve                   src=38  obs=0
 25  occurrence_binding_candidates                src=46  obs=2
 19  type_reference_binding_context_witness_test  src=10  obs=0
 …   (namespace / source-annotation / attribution files, all source-domain)
  3  recorded_observation_envelope_witness_test   src=0   obs=33   <- the only observation subject
```

**344 → `std.occurrence_identity`, 3 → `std.observation`.** No file mixes the two
subjects in a way that makes a site undecidable, so the wild instance of the defect
DESIGN warns about does **not** appear here — which is itself worth recording, because
"we looked for ambiguous sites and found none" is a different claim from "we did not
look". The neutral-scoring files (`attribution`, `source_annotation`,
`namespace_clause_e_projection_law`) are source-domain by their subject matter, not by
keyword count, and were classified by reading rather than by the score.

## Diagnostic spans do not reliably locate the failing reference

Discovered while building the repair, and it constrains how any mechanical pass can
be written. The obvious safe instrument is **span-directed rewriting**: the compiler
has already located every refusal at byte grain, so the rewrite is driven by the
instrument's own output rather than by a regex over source text (a regex is a *model*
of the resolver, and this lane has already paid for one of those). Each edit verifies
that the span's bytes equal the diagnostic's name before writing anything.

That verification immediately refused a third of the corpus:

| diagnostic class | total | span points at the name |
|---|---|---|
| `unresolved type` | 1745 | 1095 (62%) |
| `undefined variable` | 1073 | 756 (70%) |
| `function … not found` | 1678 | **517 (30%)** |

Worked example — `unresolved type 'ModuleIdentity' (repo_atlas_projection.dag:10819-10833)`
points at `AtlasChangeMar`, the *enclosing declaration's* name, while the failing
reference sits ~28 bytes later inside the record body. Both names are 14 bytes, so a
length check would have passed and a naive rewrite would have corrupted the declaration
while leaving the reference untouched.

**This is a §5 locatedness gap**, not merely an inconvenience: "typed, located
diagnostic" is the standard, and a span that names a reference's container has located
the wrong thing. The root fix belongs in `04_infer` — report the reference's own span —
and it converges with the repair already owed there.

**The pass converges in ONE round, and the reason refutes the obvious plan.** The
expectation was an iterative loop — qualify what verifies, re-run, let newly-resolving
declarations expose the residue with fresh spans. Measured, pass 2 yields **four edits**,
because span accuracy does not refresh; it **depletes**:

| class | accuracy before pass 1 | after |
|---|---|---|
| `function` | 30% | **1%** |
| `undefined variable` | 70% | 36% |
| `unresolved type` | 62% | 11% |
| all | — | 11% (281 of 2412) |

A pass can only edit the accurately-located sites, so it consumes exactly those and
leaves the inaccurate residue behind. The stopping condition ("a round that stops
shrinking") was right; the mechanism was wrong — it stops **immediately and
structurally**, not through diminishing returns. Whatever the spans can locate is
extracted in one pass, and everything else is blocked until the spans are fixed.

The safety clause survives unchanged and matters more, not less: **a round that stops
shrinking is the signal to fix the spans, never to widen the rewriter.** A rewriter
widened past its own verification is the absorbing fallback with an edit attached, and
it would write corruption at precisely the sites where the instrument is least reliable.

## The right declaration index already exists — do not re-model it a fourth time

`src/v1/stage0/src/source_closure.rs` carries `DeclarationIndex` /
`build_declaration_index`, and it is strictly better than the `^type|fn|func` regex
this census used:

- **it indexes variant arms**, discriminating them correctly (children are variants
  only when the item's `connective == Disj`, so record fields are excluded) — which is
  exactly the 1246 "no declarer" refusals the regex cannot see;
- **it keeps homonyms plural** by construction (`by_name: name → set of modules`), with
  an in-code note that collapsing it would silently pick a winner before the resolver
  ever sees the ambiguity — the discipline the declarer-count classification here had
  to rebuild by hand.

It was believed deleted and was not: only its probe test was removed, by this lane.
That false belief has already produced one withdrawn measurement (a grep-based
declared-vs-undeclared classification, wrong in both directions) and one second-rate
index (this one). **It now has a single caller, a test helper**, so an ordinary
consumer-join would read it as dead — a zero created by this program's own deletions,
not by the corpus.

If a later pass needs arm-aware declarer counts, the move is to **restore its probe**
and use the real index, never to re-model it again.

## Sequencing consequence

Class 3 is now eight names, all small, none blocking. The rule that survives is
narrower and sharper than "settle the decisions first": **a mechanical pass must not
run over a name whose declarers mean different subjects**, because qualification is
precisely what makes that difference unobservable again. `OccurrenceId` is the live
example — mechanical per-site, but only because each site's subject is determined by
what the site is about, and a site that cannot say which occurrence space it inhabits
must be reported rather than qualified.

## Result and boundary

Pass 1: **1504 edits removed 2033 diagnostics**, 4801 → 2768 (42%). The denominator was
stable across both runs — 2360 resolved / 1353 census-only — so this is a real reduction
rather than a shrinking measurement.

The echo held, confirmed a second time and at scale, on names **never edited**:

```
LiveTreeDisposition   582 → 8     575 edits, qualified directly
SubstrateInputsOnly   509 → 7     not edited — collapsed with its annotation
ReadsLiveTree          70 → 1     not edited
```

571 arm diagnostics vanished for free. The arm bucket was never independent work and the
expected-type filter was never missing.

By class: `unresolved type` 1744→731 · `function` 1673→1183 · `undefined variable`
1074→498 · `no field` 230→213 · `method` 52→46. Note `no field` went **down**, so the
newly-revealed-defect effect is smaller than the single-file probe suggested; it is not
claimed as a rise that cannot be seen.

**The remaining 2768 partition by blocker, not by effort:**

| count | blocker |
|---|---|
| 1708 | span mismatch — hard-blocked on the `04_infer` locatedness defect |
| 659 | no declarer — coproduct arms; needs the arm-aware `DeclarationIndex` |
| 41 | undispositioned multi-declarer — the small fork candidates above |
| 4 | available now; not worth a census run alone |

So the span defect is not a side finding. It is the **sole blocker on 1708 sites**, and
with the arm index accounts for 2367 of the 2768 — the critical path, not a §4b row for
someone to file eventually.

**Stopped deliberately here.** Both remaining moves are a different kind of work from a
migration edit: the span fix changes `04_infer`, a pipeline stage the brief names
load-bearing; the arm index means restoring a deleted probe or writing a bin, both
net-new inside a deletion lane. Each is a decision, not a default.

Applied-pass audit: zero edits inside `//` annotation lines, zero declarations rewritten,
zero self-qualifications. That last check took two attempts — the first used **unescaped
dots**, so `extdeps.github.app.` matched the diff's own `+++ b/dag/extdeps/github/app.dag`
header and reported three false positives. The check was a model of the thing it checked,
and it was wrong before the code was.

## Deletion hygiene: what a narrowed search universe hid

Recorded because the method error is more transferable than the specific repair.

The cut deleted 12 files. Checking whether any was still owed, the first search used
`--include=*.dag` over `dag` and `src/v2` — excluding `.md`, `docs/`, all of `src/v1`,
and every `.rs`. It returned clean. **Re-run unfiltered over all 4568 tracked files,
every one of the 12 subjects is referenced in 1–7 files.**

A positive control (`claim_batch` → 200 files) passed in both runs, which is the point:
**a positive control validates the predicate, not the scope.** "Zero consumers" always
means "zero consumers in the files I searched", and a zero from a narrowed universe is
the empty-observation narrow — ⊥-as-answer conflated with ⊥-as-ignorance — failing
silently in the direction of deletion.

What it hid: `class_b_trim_specimen_test.rs` was deleted while three live carriers named
it — the retirement model declared it `RetainedLegacyRustTestModule`, a disposition
carrier named its test function as `legacy_discovery_known_positive`, and a fixture cited
it as an executing receipt. The retirement model's contract is *executed identities must
equal declared identities*, so a roster naming an absent file is a latent red.

**Restoration was attempted and rejected on evidence**: the file re-enrols, but four of
the five `cli_run` symbols it imports are gone, and its purpose row says its whole
subject is explicit-import binding in a narrow pool — semantics this cut makes a parse
error. The deletion was correct; the **receipt** was the omission. The model needed no
retirement row either, being a *retention* roster by design ("the complement needs no
rows at all", explicitly rejecting "a cemetery of authored classifications"), so the fix
was removing the row. `declared == enrolled` now holds, verified by **set diff, not
count** — two counts can agree while their members differ.

Restoration also surfaced unrelated debris: the original deletion left **three orphaned
`#[cfg(test)]` attributes** stacked on one `mod` and stripped another's, so
`decl_facts_dimensionless_projection_test` was building unconditionally.

**The one genuine vanishing guard**, and the generalisable part: the known-positive
control's own comment says it names an identity that "exists independently of this
migration". Its author guarded it against the *test* migration; it was not independent of
the *import cut*. Had it gone false, a discovery scanner failing its known-positive reads
as *the scanner is broken* — not as *an unrelated lane deleted the control's subject* —
so the failure would have pointed at the wrong mechanism entirely. Repointed to an
algebraic subject with the reasoning recorded at the site.

The residual defect is not repaired and is worth naming: **"independent of this
migration" is a property relative to one adversary, and no carrier records which.** A
control chosen to survive migration A is not thereby robust to migration B, so the next
lane inherits the same trap unless independence claims name what they are independent of.

## Known residue: one user-visible dangling citation, deliberately not patched

`src/v1/stage0/src/main.rs` prints, on every compile with advisory diagnostics:

```
N blocking error(s), M advisory diagnostic(s) (policy: gunbc.compile_clean_diagnostic_policy)
```

That module was deleted by this cut, so the citation names nothing — a stale citation in
a **user-visible** string, which is worse than one in prose.

It is **not** patched here, for a reason worth stating rather than leaving as an
omission. The file carries `// Generated by v1 compiler -- do not edit`, but no `.dag`
in the tree produces the string (`advisory diagnostic` has zero `.dag` hits), so it is
seed-only text inside a file marked generated. Without knowing which generator owns it,
hand-editing the artifact is the exact move that gets reverted by the next regen — and
"hand-patched at the artifact by someone in a hurry" is the failure this document has
been cataloguing all night.

Checked and found **already correct**, so it needs nothing: `cli_run.rs`
`compile_clean_diagnostic_is_hard`'s doc comment, which was initially suspected of the
same defect. It is a proper retirement receipt — it names the deleted arm and row, the
authorization, and the date, and explains that every class now delegates to the one
`00_core.dag` authority. That is what a retired citation should look like.
