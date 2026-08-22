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
