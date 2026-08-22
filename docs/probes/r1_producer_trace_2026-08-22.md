# R1 producer trace — static provenance, cross-depth candidate, and the committed prediction table

**OBSERVED ON:** `docs/probes/e0308_partition_2026-08-22/sites_classified.tsv` from gunbc#8884
(branch `session/eager-lark-892`, `d770f389c6`), measured at `967b5bc1b92ee66250e06a7870c132b48a16b80a`.
**CLAIM ABOUT:** the R1 cluster on that subject (`src/v2/compiler/03_ingest.dag`, M=1) only.
**STATUS: measurement only.** No producer is perturbed here and no production repair is opened.

---

## Finding 0 — R1 is 36 sites, not 39: three rows are not bare↔`Rc` deltas

Before any depth question, the cluster's membership has to hold. Three of the 39 labelled rows
carry a different delta entirely:

| site | expected | found | actual delta |
|---|---|---|---|
| `v2_std_fold_assembly.rs:258` | `Rc<Outcome<Rc<Vector<Rc<Vector<Rc<Node>>>>>>>` | `Rc<Outcome<Rc<Vector<Rc<Node>>>>>` | an **extra collection nesting layer** — `Vector<Vector<Node>>` vs `Vector<Node>` |
| `v2_compiler_body_lowering_fold.rs:3176` | `Rc<Outcome<Option<Rc<Node>>>>` | `Rc<Outcome<Rc<Node>>>` | **`Option` presence** — R2's axis |
| `v2_compiler_body_lowering_fold.rs:3176` | `Rc<Outcome<Option<Rc<_>>>>` | `Rc<Outcome<Rc<_>>>` | same |

In all three the `Rc` structure is *identical on both sides*; what differs is a collection layer or
an `Option`. They inflate R1 and deplete R2/ELEM-COLL. **R1's true size is 36**, and the two
`body_lowering_fold` rows belong to the cluster the brief defers precisely because `Option` mixes
several axes.

## Depth partition (true R1)

| depth | `delta` value | sites |
|---|---|---:|
| OUTER | `rc_depth` | 17 |
| TYPE-ARGUMENT | `generic_arg` | 11 |
| COLLECTION-ELEMENT | `generic_arg+container` | 6 |
| OUTER+CONTAINER | `rc_depth+container` | 2 |

The brief's "17 outer / 14 type-argument / 6 element" sums to 37 against a cluster of 39; the
reconciliation is that `rc_depth+container` (2) is a fourth `delta` value, and 3 of the
`generic_arg` rows are the misclassifications above.

---

## Finding 1 — exactly ONE declaration crosses depths, and it does NOT reach element depth

Carrier extracted per site as the type that gains or loses an `Rc` wrapper:

| carrier | depths | sites |
|---|---|---:|
| **`Nat`** (emits `i64` when the native alias fires, `Nat` when it does not) | **OUTER + TYPE-ARGUMENT** | **15** |
| `LetBinding` | OUTER | 4 |
| `Edge` | OUTER | 2 |
| `DeriveGrammarRelationTokensProgress` | OUTER | 2 |
| `PortReading` | **ELEMENT** | 2 |
| `NarrowingReason` | **ELEMENT** | 2 |
| `Finding` | **ELEMENT** | 1 |
| `ComplexityLowering` | **ELEMENT** | 1 |
| `Refined<Rc<Artifact>>`, `Refined<_>`, `MachineShapeWalkResult`, `AdmitExportsFold`, `SourceRefReadResult` | OUTER | 1 each |
| `Vector<Rc<Token>>`, `HashMap<_, _>` | OUTER+CONTAINER | 1 each |

**`Nat` is the only cross-depth declaration** — 15 of 36 sites (42%), the largest carrier in
the cluster, spanning OUTER (4) and TYPE-ARGUMENT (11).

**COLLECTION-ELEMENT is disjoint.** Every element-depth carrier (`PortReading`, `NarrowingReason`,
`Finding`, `ComplexityLowering`) occurs at element depth *and nowhere else*. No declaration in this
cluster appears at element depth and at any other depth.

Per the brief's own Step 2 rule — *if no exact declaration crosses depths, that is already the
answer* — this is that answer **for the element arm specifically**: element depth cannot be joined
to outer or type-argument by any shared declaration, so it is evidence that element depth is a
separate producer root rather than the same authority consumed one level deeper.

## Finding 2 — the decision table cannot be executed as written

The brief's first row is *"all three depths for X move → one recursively consumed reference-layer
authority."* **That row is unreachable.** The only cross-depth declaration has **zero element-depth
sites**, so no single-declaration perturbation can move all three depths — not because the
hypothesis is false, but because no such X exists in this cluster.

A `Nat` perturbation adjudicates **OUTER vs TYPE-ARGUMENT and nothing else**. Element depth
needs a separate instrument, and this cluster does not contain one.

## Finding 3 — a confound *inside* the chosen declaration

`v1.compiler.coercion` `rust_seed_host_numeric_alias(name, decl_file)` returns `"i64"` when the name
is `Nat` or `Int` **and** `decl_file_realizes_natively(decl_file)` holds, and `none` otherwise. Both
outcomes are live inside R1:

- alias **fired** → `i64` — 13 sites (`std_checked_arithmetic.rs:305`, `std_measure.rs:488`, and 11
  at type-argument depth inside `Measure<_, _, M>`)
- alias **did not fire** → `Nat` — 2 sites (`v2_lens_cost.rs:312`, `:315`)

So one declaration sits under **two different base realizations** within the cluster, selected by
`decl_file`. The brief requires the perturbation to hold base realization fixed and change only the
reference-layer decision. Here the second axis is *interior to the candidate*: a perturbation keyed
on the declaration, not on (declaration, decl_file), would silently vary base realization too. That
is the brief's own REJECT row, arriving from inside the chosen X rather than from an unrelated
cluster.

---

## The prediction table — committed before any perturbation runs

Registered now, so no stray movement can be adjudicated after the fact.

| observation | conclusion |
|---|---|
| both OUTER and TYPE-ARGUMENT `Nat` sites move | outer and type-argument share one recursively consumed reference-layer authority |
| OUTER moves, TYPE-ARGUMENT does not | outer and nested are separate producer roots |
| TYPE-ARGUMENT moves, OUTER does not | the recursive type renderer has its own producer; the top-level path differs |
| sites change **direction** rather than resolving | the selected verdict is still wrong, or declaration and value consumers disagree |
| **any ELEMENT site moves** | **REJECT** — no `Nat` site exists at element depth, so element movement proves the intervention was not declaration-scoped |
| any non-R1 cluster moves | **REJECT** — more than one axis changed; not a valid discriminator |
| the `Nat`-spelled sites (`v2_lens_cost.rs:312/315`) move differently from the `i64`-spelled ones | base realization was not held fixed — see Finding 3 |

The last two rows are the ones that protect the experiment, and the element row is registered as a
**rejection** criterion rather than a confirmation one precisely because Finding 1 says it cannot
legitimately fire.

## Behaviour-preservation control for the trace (mandatory, not yet run)

Emitted bytes must be **identical** with tracing on and off, and the cargo histogram unchanged. If
tracing perturbs the artifact, the trace describes a different program than the board does. The
control is a byte comparison of the emitted tree plus an equality check on the coded-error
histogram, both arms in one dispatch.

## Why this stops here

The brief instructs: find a cross-depth declaration; if none exists, report that as the finding and
stop. The result is *partial* in a way that changes the next step — a cross-depth candidate exists
for two depths and **cannot exist** for the third — so the single-axis perturbation adjudicates less
than the brief's decision table assumes. Building the dynamic tracer on the assumption of a
three-depth join would instrument a question the cluster cannot answer. Adjudication of Findings 1–3
comes before that.

---

# CORRECTION: R1 is 32, not 36 — and the delta-shape-invariance call

## The corrected arithmetic

Finding 0 above reported **3** misclassifications, found by inspection. A mechanical rule finds
**7**. The rule: *a site is R1 only if erasing every `Rc` wrapper from both sides makes them equal*;
any residue is a different axis. Decidable, and independent of what anyone notices.

| site | after `Rc`-erasure | actual axis |
|---|---|---|
| `v2_compiler_body_lowering_fold.rs:3176` ×2 | `Outcome<Option<Node>>` vs `Outcome<Node>` | R2 — `Option` presence |
| `v2_std_fold_assembly.rs:258` | `Outcome<Vector<Vector<Node>>>` vs `Outcome<Vector<Node>>` | ELEM-COLL — extra nesting layer |
| `v2_lens_complexity_lowering.rs:48` | `Vector<()>` vs `Vector<ComplexityLowering>` | C — carrier collapses to `()` |
| `std_realization_schedule.rs:64` ×2, `:83` | `Measure<(), S, i64>` vs `Measure<(), _, i64>` | type-parameter binding (`()`/`S`/`_`) |

**True R1 = 32**: OUTER 17, TYPE-ARGUMENT 8, ELEMENT 5, OUTER+CONTAINER 2.

*A near-miss worth recording:* the first eraser was `re.sub(r'Rc<([^<>]*)>', …)`, which silently
fails on `Rc` wrapping a generic — `Rc<Refined<Artifact>>` survives untouched and looks like a
residual difference, so the rule dropped **11**. It was caught only by self-testing the eraser on
known pairs before reading any count off it. Testing the instrument on inputs whose answer you
already know, before trusting its output, is the same discipline as an instrument control.

**Findings 1 and 2 survive and strengthen.** `Nat` remains the only cross-depth declaration —
OUTER 4 + TYPE-ARGUMENT 8 = 12 sites, now 38% of true R1. Element depth remains fully disjoint
(`PortReading` ×2, `NarrowingReason` ×2, `Finding`); `ComplexityLowering` left via the C
reclassification. No X exists at all three depths.

## Delta-shape invariance — the call, made before the run

**Observation to explain:** the delta shape is identical across the base-realization fork —
`Rc<i64>` vs `i64` where the numeric alias fires, `Rc<Nat>` vs `Nat` where it does not.

**CALL: invariance HOLDS. The reference-layer decision does not consult base realization, and the
fork is an orthogonal axis.** R1's `Nat` subpopulation is a reference-layer problem *despite*
the fork, not because of it.

**Grounds — the two decisions key on different things:**

| decision | authority | key |
|---|---|---|
| reference-layer wrap | `v1.compiler.emit_rust`, `set_contains(shared_types, leaf)` | the bare **name** |
| base realization | `v1.compiler.coercion` `rust_seed_host_numeric_alias(name, decl_file)` | **(name, decl_file)** |

`rust_type_is_rc_wrapped` is *not* the wrap decision — it delegates to
`v1.languages` `sharing_type_is_wrapped_for_target`, which is a **prefix test on the rendered
string** (does this spelling already begin with the target's wrap prefix). It is an
already-wrapped guard, not a policy. The policy is set membership on the leaf name, which never sees
`decl_file`.

Since both the `i64`-realized and `Nat`-realized sites descend from the same name, a name-keyed wrap
decision must treat them identically — which is exactly the invariant delta shape observed.

**What would falsify this call:** the two `Nat`-spelled sites (`v2_lens_cost.rs:312`, `:315`) moving
under a perturbation keyed on `(declaration, decl_file)` restricted to the natively-realizing arm.
They are outside that key and must not move. If they do, the wrap decision is reachable from base
realization after all and the 12 sites belong to a different cluster.

**Verified vs not:** the key difference is read from the two functions' signatures and bodies. What
is *not* established is how `shared_types` is populated — if its membership were itself derived from
`decl_file` upstream, the two keys would be coupled through a path this reading does not cover.
That is the one way the call could be wrong, and it is named rather than left implicit.

## Instrument design, given all of the above

Perturb **(`Nat`, natively-realizing `decl_file`)** — the alias-fired subset, which spans
OUTER 2 and TYPE-ARGUMENT 8 with one base realization held fixed.

| observation | conclusion |
|---|---|
| both OUTER and TYPE-ARGUMENT move | outer and type-argument share one recursively consumed authority |
| OUTER moves, TYPE-ARGUMENT does not | separate producer roots |
| TYPE-ARGUMENT moves, OUTER does not | the recursive renderer has its own producer |
| sites change direction rather than resolving | verdict still wrong, or declaration and value consumers disagree |
| **`v2_lens_cost.rs:312/315` move** | **REJECT** — keyed on the name, not the pair; base realization was not held fixed |
| **any ELEMENT site moves** | **REJECT** — no `Nat` site exists at element depth |
| any non-R1 cluster moves | **REJECT** — more than one axis changed |

---

# The brief's question has a third answer: ~25 producers, one policy

The brief asks whether R1 is **one recursively-consumed reference-layer authority** or **three
position-specific producers wearing one delta**. Read statically, it is neither.

`v1.compiler.emit_rust` contains **~25 independent `set_contains(shared_types, …)` wrap tests**.
At every one of them the key is a **bare authored name** — `leaf`, `type_name`, `name`,
`enum_name`, `ty_name`, `rc_name`, `ctor_name`, `acc_type_name`,
`authored_name_at(…)`. What gets wrapped varies by position (`scalar`, `base`, `applied_ty`,
`rendered`); the *key* does not, and none of them consults `decl_file`.

So R1 is **one policy applied at ~25 sites**, not one authority and not three producers:

- **One policy** — a single predicate over a single set, keyed on the name, everywhere.
- **Not one authority** — no single function decides; twenty-five sites each decide for themselves.
- **Not three producers** — the count is an order of magnitude larger than the depth count, and the
  sites do not partition along outer / type-argument / element.

This strengthens the invariance call rather than weakening it: not one name-keyed decision that must
treat both base realizations alike, but twenty-five, all using the same key, none of which can see
`decl_file`.

**And it reframes the repair.** Changing the wrap policy means changing 25 sites, or extracting the
policy to one authority *first* and then changing it once. That is a §2/§3 consolidation — one
concept, one authority — and it is prior to any behavioural fix. A repair that edits some subset of
the 25 leaves the rest applying the old policy, which is the forked-logic trap.

## One structural asymmetry, found at the decision point

`rust_render_checkpoint_scalar_bare` (`05_emit_rust.dag`) short-circuits:

```
match rust_seed_host_numeric_alias(name: leaf, decl_file: type_reference_decl_file(n: n)) {
  Present { value: numeric } => Present { value: numeric }     // alias fired: returns i64, wrap NEVER consulted
  Absent => … if set_contains(shared_types, leaf) && !rust_type_is_rc_wrapped(scalar) { wrap } …
}
```

On the alias-fired path **this producer cannot wrap at all** — it returns before the wrap test. So
the `Rc<i64>` sites in R1 are not produced here; they must reach a wrap test by a path that does not
route through this function.

That is a *live question for the trace*, and it is the first thing the tracer should answer: **which
of the ~25 sites emits each of the 10 `i64` R1 sites?** The static reading cannot say, because it
cannot see which producer runs at a given reference.

It also qualifies the invariance call without overturning it. The grounds — wrap keyed on name,
realization keyed on the pair — hold at every site. But the `Nat`-spelled and `i64`-spelled sites may
be emitted by *different producers*, in which case the invariant delta shape is two producers
independently applying one policy rather than one producer treating both alike. Same conclusion for
the experiment; a different reason, and the trace distinguishes them.

---

## Correction — the carrier is `Nat` alone, and every assignment is now per-site

Every earlier revision of this document named the cross-depth carrier `Nat`/`Int`. **`Int` has zero
sites in R1.** The joint naming came from assigning some rows by *module ratio* — reading which of
`Nat` or `Int` a module mostly used — and `smart-ram-730` refuted one such row by reading the raw
block: `std_checked_arithmetic.rs:305` is `value: nat_magnitude(a.clone())`, whose field is declared
`Nat` and whose `fn nat_magnitude(a: Int) -> Nat` takes `Int` only as a *parameter*. The ratio had
made it an `Int` site.

The rule that follows, and the reason this is a correction rather than an edit: **a module-level
ratio is not evidence about a site, at either confidence level.** The row I had flagged as weakest
(`std_measure.rs:488`) turned out correct; the row I had not doubted was the one that failed. So
every `i64` row was re-derived from its raw cargo block plus the declaring `.dag` type:

| site | emitted expression | declaring type | carrier |
|---|---|---|---|
| `std_checked_arithmetic.rs:305` | `value: nat_magnitude(a.clone())` | field `Nat`; `nat_magnitude` **returns** `Nat` | `Nat` |
| `std_measure.rs:488` | `predecessor: predecessor.clone()` | `PositiveMeasureSuccessor { predecessor: Nat }` (`measure.dag:495`) | `Nat` |
| `std_cache_interface.rs:652` | `recompute_saved: recompute.time` | time measure → `Measure<Time, _, Nat>` | `Nat` |
| `std_cache_interface.rs:667` | `nanosecond_count(recompute.time)` | same | `Nat` |
| `std_realization_measurement.rs:96` | `time: time.clone()` | `time: Measure<Time, S, Nat>` (`realization_measurement.dag:53`) | `Nat` |
| `std_realization_schedule.rs:74` | `time: time.clone()` | `time: Measure<Time, S, Nat>` (`realization_schedule.dag:33`) | `Nat` |

`std/measure.dag` is **not** safely `Nat` by ratio in any case — it carries
`CelsiusDelta = Measure<…, Int>` and `PositiveMeasureCountNonPositive { observed: Int }`. The ratio
agreed there by luck.

A second correction, to a chain of my own: I had reported `cache_interface` reaching `Measure`
through `CapacityValue<ByteSize>`. The raw block shows `recompute.time`, so it arrives through the
**time** measure (`Nanosecond`), not `ByteSize`. Same carrier by a different mechanism — recorded
because a stated chain that happens to reach the right answer is the one that gets reused.

## Four `Measure` sites are EXCLUDED from the instrument — a documented emitter defect, not wrap policy

Reading all ten together surfaced the same carrier disagreeing in **opposite directions** at one
depth:

| sites | expected | found | direction |
|---|---|---|---|
| `cache_interface` 652, 667 | `Rc<Measure<(), (), i64>>` | `Rc<Measure<(), (), Rc<i64>>>` | expected **un**wrapped, params **collapsed** |
| `realization_measurement` 96, `realization_schedule` 74 | `Rc<Measure<(), S, Rc<i64>>>` | `Rc<Measure<(), S, i64>>` | expected **wrapped**, param `S` **retained** |

This is **not** a non-confluence in the wrap policy. The two declarations differ in *spelling* —

- `dag/std/cache_interface.dag:381` — `recompute_saved: Nanosecond` (**named alias** to an applied generic)
- `dag/std/realization_schedule.dag:33` — `time: Measure<Time, S, Nat>` (**direct** applied generic)

— and the collapse is an already-documented defect with a registered dissolution condition, at
`dag/std/measure.dag:755`, verified in-tree:

> `stage0 alias emission collapses applied-generic Measure aliases to concrete Measure<(), (), Nat>
> while fn/data return sites still reference the un-erased alias params (E0107). Dissolve-on: stage0
> Measure-alias emitter preserves return types at data/fn sites.`

`Measure<(), (), Nat>` is exactly the `cache_interface` shape.

**Consequence for the instrument.** The `cache_interface` declared types came out of a *different and
broken* emission path, so a perturbation on `Nat`'s `shared_types` membership will not move them the
way it moves the direct-spelled sites. Admitting them would make any split movement ambiguous
between the decision table's rows and this third cause — the confound-inside-the-candidate problem
arriving from a new direction. They are therefore **excluded and registered here**, not silently
dropped. `Nat`'s TYPE-ARGUMENT 8 is **4 clean** (`realization_*`) and **4 alias-contaminated**
(`cache_interface`); the instrument runs **OUTER 4 against TYPE-ARGUMENT 4** — smaller, and
homogeneous, which is the point.

**One qualification on the alias reading.** The direct-spelled sites do not merely differ in
spelling: they sit inside *generic functions* where `S` is a live parameter (`fn
cost_account_measured<S>`, `fn cost_account_measured_from_time<S>`). `S` is retained there because
`S` is genuinely in scope, not necessarily because the direct path preserves parameters better. The
two paths differ on a second axis, and the trace should establish which of them it is distinguishing
before the alias reading is treated as settled.

**Scope beyond R1.** `realization_schedule:64` and `:83` were dropped from R1 by the erase-`Rc` rule
as "type-param binding, `()` vs `S` vs `_`" and classified `D` — the same collapse signature seen
from the other side. One documented defect is generating rows in at least two clusters, and the
repartition vocabulary has no arm that names it. Reported to `eager-lark-892` as a repartition input.

**Not claimed:** that the alias path *causes* the collapse. The evidence is the note plus a shape
match on four rows; no measurement of mine establishes causation. The trace can confirm or refute it
by showing which producer emitted each declaration — which is now a more valuable question than the
original attribution one.

## What survives unchanged

`Nat` as the sole cross-depth carrier; TYPE-ARGUMENT reached via `Measure<…, Nat>`; the
base-realization fork (Finding 3) sitting **inside `Nat` alone**, with `v2_lens_cost.rs:312/315` as
same-declaration controls; element depth disjoint (Finding 1); the decision table's first row
unreachable (Finding 2). `Nat`'s span is now OUTER 4 + TYPE-ARGUMENT 4 clean, one declaration, no
ratio-derived rows anywhere in it.

---

## The trace RAN — and the answer to the brief's question is "neither"

Subject ref `4f0c90559d668b405c7a5e631282a8c17235a94c`. Instrument: every
`set_contains(shared_types, X)` in the emitted stage0 mirror rewritten to a tracing wrapper carrying
the original line as a site id, so each of the 24 wrap sites reports `(site, key, verdict)`.

### Behaviour-preservation control — PASSES

Mandatory per the brief, and it required a discriminator rather than a single comparison:

| arms | result |
|---|---|
| tracing OFF vs tracing ON | **IDENTICAL** |
| OFF vs OFF (same binary, same env) | **DIFFER** |
| ON vs ON | **DIFFER** |

**Tracing perturbs nothing** — the traced and untraced trees are byte-identical, so the instrument is
admissible. The OFF-vs-OFF row is a separate finding about the emitter, recorded below; it is *not*
an instrument defect, and an earlier single-run-per-arm attempt could not have told the two apart.
That is why the discriminator was worth a dispatch.

### Result — the wrap verdict is CONFLUENT

| | |
|---|---|
| `Int` ever `true`, anywhere | **0** |
| `Nat` ever `false`, anywhere | **0** |

`Nat` is in `shared_types` at **every** producer that consults it — type-position (`1755`, `1781`,
`1967`) and value-position (`12596`, `17474`, `18653`, `19919`, `25623`) alike. `Int` is in it at
**none**.

**So the brief's one-vs-three question has a third answer: neither.** The producers do not disagree
about `Nat`; there is no position-specific verdict to reconcile. R1's divergence is therefore **not
produced by the wrap policy forking** — one policy, one verdict, and the disagreement lives
downstream of the wrap decision.

This also corroborates *from an independent direction* that `Int` has zero R1 sites: `Int` is not a
shared type at all, so no `Int` site can exhibit a bare↔`Rc` wrap delta.

### What it does to the alias reading

Confluence does not confirm the alias-collapse cause — it **removes its only competitor**. Had the
wrap policy been forking, the opposite directions at `cache_interface` vs `realization_*` would have
had a live explanation *inside* the policy. It is not forking, so the direction split originates
upstream of the wrap decision, which is exactly where `measure.dag:755` places the alias-emission
defect. **Still not established:** which producer emitted each declaration. The trace answers the
wrap *predicate*, not the declaration *emitter*; that is a different instrument.

### Consolidation — the premise, verified against `origin/main`

| | |
|---|---|
| `decl_file` in `src/v1/04_emit_info.dag` (`TypeSummary`'s home) | **0 occurrences** |
| `build_shared_types(type_summaries, recursive_type_set, target)` | **no `decl_file` parameter** |
| `set_contains(shared_types, …)` sites | **24** |
| `decl_file` occurrences in `src/v1/05_emit_rust.dag` | **45** |

The finding is **not** "`decl_file` is unavailable here." It is available and heavily threaded
through the same module (`lookup_checkpoint(target:, dag_name:, decl_file:)`, the
`rust_scalar_checkpoint_reference_base` / `_grounding_base` split). The finding is that **one policy
at 24 sites keys on the bare authored name while the identity-keyed authority sits in the same
module.** That is a stronger argument for consolidating the wrap decision into one authority than the
unavailability reading would have been.

## Emitter nondeterminism — the mechanism, and the retraction of my own hazard claim

Same binary, same environment, same tree, consecutive emits of `src/v2/compiler/03_ingest.dag`
differ. I first reported this as a possible `--required-regen` flake source before characterising it.
**That escalation is withdrawn.** The mechanism removes it.

### Mechanism — pure `use`-statement reordering

Three emits of the unchanged tree, all three pairwise different, and the entire difference is one
import line moving one position:

```
10d9
<  pub use crate::v2_std_qualified_name::{QualifiedName};
11a11
>  pub use crate::v2_std_qualified_name::{QualifiedName};
```

Line counts identical across all three runs (336). Sorted-identical test on both files: **pure line
reordering**, no content difference at all. So this is import-emission *order* — a set or map
iteration — not a value or content nondeterminism.

**The churn set is not stable.** The first run set churned two files; a second run set on the same
tree churned only `v2_lens_enforcement_vocab.rs`, with `v2_std_cross_tree_resolution.rs` identical
across all three. *Which* files churn is itself run-dependent while the shape is always reordering.
A single run per arm therefore cannot even establish the churn population, let alone a difference.

### Why the hazard does not fire — verified by execution

`rustfmt` normalizes `use`-statement order. Checked with a discriminating control — two files
differing only in the order of two `pub use` lines, formatted, converging byte-identically:

```
RUSTFMT NORMALIZES use-ORDER: two orderings converge
```

Any comparison over a rustfmt-normalized artifact cannot see this class. The regen gate compares
normalized artifacts. The hazard required the nondeterminism to *survive* normalization; this one
does not, so there is no coin flip for a gate to install — both arms normalize to the same bytes.

This also explains the seed's `--emit-fresh` twice-zero result being rustfmt-normalized rather than
natively deterministic. For this mechanism that green is genuinely closed, not papering over a
difference: normalization is the fixed point in which the difference does not exist.

**Not retracted:** the emitter *is* nondeterministic on raw output — measured, and it stands. The
laundering argument also stands as reasoning for any nondeterminism that *does* survive `rustfmt`.
What is withdrawn is the claim that this specimen is an instance of it. The lesson is the one this
document keeps re-learning: a hazard asserted from an uncharacterised mechanism is a claim about
something not yet observed.

### Consequence for the consolidation oracle

"Emitted bytes identical before and after" is unusable as stated on raw output. Two options, and the
second is better by §5:

1. Establish the nondeterministic set with N≥2 emits of the unchanged tree and **exclude** it,
   stating the excluded population — validation, and the exclusion list is itself unsound if derived
   from one run (see churn instability above).
2. Compare **rustfmt-normalized** emitted output, which makes this entire class *unrepresentable* in
   the oracle rather than subtracted from it — construction over validation, with no exclusion
   population to state or maintain.

Option 2 depends on a claim I have **not** closed: that no *other* nondeterminism survives
normalization. Establishing that is the same N≥2 control, run over normalized rather than raw output.

---

# R1's ROOT — a key collision, and the confluence result reinterpreted

## Two declarations spell `Nat`

Verified on `origin/main`:

| declaration | shape | realization |
|---|---|---|
| `dag/std/nat.dag:6` | `type Nat = CommutativeSemiring<Magnitude>` | **natively realizing** — emits `i64` |
| `src/v2/std/nat.dag:13` | `type Nat = Zero \| Succ { prev: Nat }` | **Peano coproduct** — genuinely shared |

And the set that decides wrapping **cannot represent the distinction**:

| | key |
|---|---|
| `maybe_mark_shared_type` | `set_insert(acc, summary.name)` — the **bare name** |
| `TypeSummary` | `name`, `repr`, `field_summaries`, `field_type_map`, `field_import_surface_names`, `variant_name_set`, `generic_param_names`, `has_fn_fields` — **no declaring file** |
| every wrap site | `set_contains(shared_types, leaf)` — bare leaf |

Both declarations collide on one key. One membership bit is returned for two carriers that need
opposite answers.

The one guard that could have separated them **cannot, structurally**:
`fn is_grounded_coproduct_native_alias(name: String) -> Bool` takes a *name*, so it can exclude a
spelling entirely or not at all; excluding one of two declarations that share a spelling is not
expressible in its signature. It covers only the Vec/Optional/Diagnostics aliases in any case.

## The confluence result was read backwards

This document reported `Nat` true at all 8 producers, `Int` never true, and called the wrap verdict
**confluent** — presented as a clean negative result retiring the one-vs-three question. The
confluence is real, but it is **not a healthy signal: it is the defect's signature.** The producers
agree because the set they consult *has no way to disagree* — one key, one bit, two carriers. A
perfectly confluent policy over a key that conflates its subjects is exactly what you measure when
the fork lives in the **key** rather than the consumers.

So the brief's question does not have the answer "neither one producer nor three." It has the answer
**the divergence is not in the producers at all — it is in what they are asked to look up.**

## Both declarations are in the traced closure — Finding 3 was the proof

`rust_seed_host_numeric_alias(name, decl_file)` returns `i64` iff `name ∈ {Nat, Int}` **and**
`decl_file_realizes_natively(decl_file)`. Finding 3 records that R1's `Nat` sites split by exactly
this arm — alias **fired** (emitting `i64`) vs alias **declined** (`v2_lens_cost.rs:312`, `:315`,
emitting `Nat`). The two subsets therefore differ precisely in **declaring file** while sharing the
name `Nat`, in one closure, in one run. Both declarations are present and both are reached.

That makes the collision sharper than the collision alone: two arms of the same emitter sit in the
**same expression** and disagree about what they may know —

| arm | key | can separate the two `Nat`s? |
|---|---|---|
| numeric alias | `(name, decl_file)` | **yes**, and does |
| sharing membership | `(name)` | **no** — one bit for both |

The identity is available at the call site, threaded there, and consulted one line above. The
emitter does not lack the discriminator; **the `shared_types` lookup discards it.**

## The ceiling is already declared in-tree — with a next-rung trigger

The header of `rust_carrier_is_at_shared_layer` (`src/v1/05_emit_rust.dag`) records a **prior
consolidation** — four call sites that each answered "is this carrier at the shared layer?"
differently, collapsed into one predicate — and states its own rung honestly:

> **RUNG: MITIGATABLE, and no higher.** The invalid state is still writable … the scalar arm keys on
> a NAME plus a declaring file, and sharing membership keys on a bare leaf name, so **two
> declarations sharing a spelling and differing in sharing are still conflated.**

That sentence *is* R1. The note even measured the same both-directions signature (18 `expected Nat,
found Rc<Nat>` against 20 the reverse in one board) and explicitly identifies its `Nat` as the **v2
Peano coproduct, not** the natively-realizing `dag/std/nat.dag` declaration.

Its next-rung trigger: the modeled layer transition `v2.compiler.wrap_decision` names, whose
`TargetReferenceLayer` and `target_layer_transition` carriers already exist in
`src/v2/std/compilers/target_model.dag` — not reachable because the v1 seed's source roots do not
include `src/v2`, and lifting the layer model to a root both compilers import is **a carrier move
that belongs ahead of this pipeline edit rather than inside it.**

## Therefore: the consolidation proposal is NOT written, and that is the finding

Consolidating the 24 `set_contains(shared_types, …)` sites onto one authority is real work that
would **change nothing about R1**: every site would consult one predicate, which consults one set,
which is keyed on a name that conflates the two `Nat` declarations. It is DESIGN §6's local
subsystem patch where the root is a carrier move — it would make the tree *look* consolidated while
the defect sat exactly where it sits, buying a rung of apparent progress with no change in the rung
that matters. The prior consolidation already demonstrated this: it collapsed four authorities into
one and landed at **mitigatable**, because collapsing consumers cannot raise a ceiling set by the key.

**What R1 is:** not a repair lane, but a **population on an already-declared ceiling** whose trigger
is the carrier move. What this lane adds to that declared row is a better-characterised population —
the per-site depth split, `Int` having zero sites *because `Int` is not a shared type at all*, and
the alias-collapse subset that is a different defect wearing the same delta.
