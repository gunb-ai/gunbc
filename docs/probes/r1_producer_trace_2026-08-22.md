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
| **`Nat`/`Int`** (emits `i64` when the native alias fires, `Nat` when it does not) | **OUTER + TYPE-ARGUMENT** | **15** |
| `LetBinding` | OUTER | 4 |
| `Edge` | OUTER | 2 |
| `DeriveGrammarRelationTokensProgress` | OUTER | 2 |
| `PortReading` | **ELEMENT** | 2 |
| `NarrowingReason` | **ELEMENT** | 2 |
| `Finding` | **ELEMENT** | 1 |
| `ComplexityLowering` | **ELEMENT** | 1 |
| `Refined<Rc<Artifact>>`, `Refined<_>`, `MachineShapeWalkResult`, `AdmitExportsFold`, `SourceRefReadResult` | OUTER | 1 each |
| `Vector<Rc<Token>>`, `HashMap<_, _>` | OUTER+CONTAINER | 1 each |

**`Nat`/`Int` is the only cross-depth declaration** — 15 of 36 sites (42%), the largest carrier in
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

A `Nat`/`Int` perturbation adjudicates **OUTER vs TYPE-ARGUMENT and nothing else**. Element depth
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
| both OUTER and TYPE-ARGUMENT `Nat`/`Int` sites move | outer and type-argument share one recursively consumed reference-layer authority |
| OUTER moves, TYPE-ARGUMENT does not | outer and nested are separate producer roots |
| TYPE-ARGUMENT moves, OUTER does not | the recursive type renderer has its own producer; the top-level path differs |
| sites change **direction** rather than resolving | the selected verdict is still wrong, or declaration and value consumers disagree |
| **any ELEMENT site moves** | **REJECT** — no `Nat`/`Int` site exists at element depth, so element movement proves the intervention was not declaration-scoped |
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

**Findings 1 and 2 survive and strengthen.** `Nat`/`Int` remains the only cross-depth declaration —
OUTER 4 + TYPE-ARGUMENT 8 = 12 sites, now 38% of true R1. Element depth remains fully disjoint
(`PortReading` ×2, `NarrowingReason` ×2, `Finding`); `ComplexityLowering` left via the C
reclassification. No X exists at all three depths.

## Delta-shape invariance — the call, made before the run

**Observation to explain:** the delta shape is identical across the base-realization fork —
`Rc<i64>` vs `i64` where the numeric alias fires, `Rc<Nat>` vs `Nat` where it does not.

**CALL: invariance HOLDS. The reference-layer decision does not consult base realization, and the
fork is an orthogonal axis.** R1's `Nat`/`Int` subpopulation is a reference-layer problem *despite*
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

Perturb **(`Nat`/`Int`, natively-realizing `decl_file`)** — the alias-fired subset, which spans
OUTER 2 and TYPE-ARGUMENT 8 with one base realization held fixed.

| observation | conclusion |
|---|---|
| both OUTER and TYPE-ARGUMENT move | outer and type-argument share one recursively consumed authority |
| OUTER moves, TYPE-ARGUMENT does not | separate producer roots |
| TYPE-ARGUMENT moves, OUTER does not | the recursive renderer has its own producer |
| sites change direction rather than resolving | verdict still wrong, or declaration and value consumers disagree |
| **`v2_lens_cost.rs:312/315` move** | **REJECT** — keyed on the name, not the pair; base realization was not held fixed |
| **any ELEMENT site moves** | **REJECT** — no `Nat`/`Int` site exists at element depth |
| any non-R1 cluster moves | **REJECT** — more than one axis changed |
