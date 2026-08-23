Two below-floor fail-opens in v1 inference, closed together. Row (a) could not land without row (c), which it exposed; both are here with executing discriminating REDs and positive controls. Row (b) is deliberately **not** in this PR — it is the largest row and follows separately.

## What is implemented

**Row (a) — a record literal's field type is not enforced when the declaring type carries type parameters.** `type BoxI<T> { v: Int }` — parameter declared but unused, field a plain kernel type — accepted `BoxI { v: "s" }` silently. Below floor (§4b): "values inhabit declared types" is the ordinary compiler floor, so this is not a rung.

Cause, established by reading the producer rather than by repair-and-measure: a field carries its declared type in `sf.inferred` while `field_node_type_expr` (`children[0]`) is a stripped placeholder. `record_lit_instantiated_fields` substituted into the placeholder, so the instantiated path **preempted a working expectation with a nameless node**. Forcing a wrong-arity bailout restores the judgment on the same declaration, which is what proves preemption rather than absence.

Discriminating RED and positive control, both executing:

```
BoxI<Int> { v: "s" }   RC=1  type mismatch: expected 'Primitive(Int)', got 'Primitive(String)'
BoxI<Int> { v: 1 }     RC=0  clean
```

## Row (c) — SOLVED

**An unestablished return type must not become a lambda's body expectation.**

Found by instrumenting the lambda arm after two hypotheses died. The measurement:

```
[PROBE] lambda exp shape=Node(fn) params=1 inferred=Resolved(Primitive())
```

The substituted arrow's resolved return is a **nameless** node — `Primitive()`, the same spelling as the `ReceiverTypeUnestablished` class — and `is_fully_resolved` returns **true** for it, because `type_resolution_verdict` folds over children and a nameless node has none, so an empty fold yields `FullyResolved`. **Could-not-establish passes the established guard**, becomes the lambda's body expectation, and an empty list literal checked against it refuses with `expected type is not a collection`: a **fabricated refusal**, ⊥-as-ignorance used as ⊥-as-answer (§5).

The fix is positive establishment — the discipline the method wall already uses. `type_node_is_established`: a type node with no authored name, no children and no params carries no information and must not become an expectation. The arm declines to answer where it has nothing to answer with, rather than answering with a nameless node.

| arm | baseline | patched |
|---|---|---|
| minimal (c) repro — generic fold, `fn` field returning `R`, body `[]` | 1 diagnostic | **clean** |
| `dag/std/claim_evidence.dag` | 0 empty-list, 18 files / 11 diagnostics | **0 empty-list, 18 files / 11 diagnostics** |
| `dag/extdeps/git/object_store.dag` | 0 empty-list | **0 empty-list** |

Source roots for the per-entry arms: `dag` **and** `src/v2`, `--entry`-scoped.

## Residual at the same seam, found by crossing the two variables — NOT closed here

`type_node_is_established` has two variables — nameless, and has-children — and the corner neither is obviously about is a node that is **nameless but has children**: `Container(List, Primitive())`, which the instrumentation saw 3 times. My predicate calls that established, so it becomes the expectation. Running that corner:

```
type Fold<R> { a: fn(String) -> List<R>  b: fn(String) -> List<R> }
fn use_it(i: Inf) -> List<String> { go(i: i, f: Fold { a: fn(v) { [1] }, ... }) }
                                                              ^^^ List<Int> where List<String> is required
```

| arm | baseline | patched |
|---|---|---|
| element type correct (`[v]`) | clean | clean |
| element type **wrong** (`[1]`) | **clean** | **clean** |

**Establishment is shallow: the node has children, but its element is itself unestablished, so no mismatch is provable and the wrong element type is admitted silently.** This is pre-existing — baseline admits it identically — so it is not a regression introduced here, and it is *not* closed by this PR.

**The residual has no author, which is why no review would surface it.** Nothing decided that a container with an unestablished element counts as established: the guard asks a shallow question, the comparison is handed a node it can prove nothing about, and the admission is whatever the structural comparison happens to do. A silently-admitting position whose admission nobody wrote is invisible to review of any individual change, because no change introduced it.

It is stated rather than left to be found because the honest scope of (c) is **"removes a fabricated refusal at this seam"**, not "closes the class at this seam". Closing it needs a deeper *comparison*, not a deeper establishment check — widening the establishment guard can only admit fewer expectations, which is strictly more lenient. That comparison is the post-substitution generic field access lane `declared_type_conformance_note` already names as its own, and it belongs with row (b).

## The census rows: measured, then deleted

`object_store`'s frontier rows were **confounded** while that module also carried empty-list blockers, because a blocking error truncates the diagnostic set. With (c) fixed they fire on a clean module. Rather than edit counts to quiet CI, the declared-vs-observed numbers were measured:

| row | declared | observed |
|---|---|---|
| `extdeps.mercurial` `any` | 3 | **0** |
| `gunbc.scm_compatibility.mercurial` `map` | 3 | **0** |
| `extdeps.git.object_store` `map` | 2 | **0** |
| `extdeps.git.object_store` `flat_map` | 1 | **0** |

Not fewer — **fully dissolved**, which is the direction the row text distinguishes: *more* means "a new unresolved call has appeared and the receiver's type should be established rather than the count raised"; *fewer* means "the row must be lowered or deleted so the ratchet keeps its new ground." At zero the honest action is **deletion** — a row declaring a deficit that no longer exists is a false claim, and a lowered non-zero floor would leave headroom for a silent reintroduction.

This is also independent confirmation that the repair does what the chain predicted. `method_existence_wall_note` names two live instances of the `ReceiverTypeUnestablished` lambda-parameter class: the `StoreObjectFold` lambdas in `object_store` — **now closed** — and `LexMatchThunk` in tokenize, still open and consistent with `arm_b` staying red. Two prose notes citing the deleted rows as live examples are corrected in the same diff.

## Corpus defects the wall caught — fixed, not suppressed

`RealizationPlan<S> { target: ContentHash }` was handed a raw `String` at two sites, which `std.content_hash`'s own authority forbids in as many words: *"Fixtures needing stable identities hash their labels with `content_hash_of_value`, never by labeling arbitrary text as a digest."* Both now route through the sanctioned mint. These are **genuine below-floor defects**, silently admitted before this PR — the fail-open this lane exists to close.

One false refusal I did introduce, and fixed: `EffectPlan<E> { steps: FreeMonoid<...> }` with `steps: []`. The corpus writes both `[...]` and `[]` into `FreeMonoid` fields, so list-literal sugar is the intended inhabitant; the collection test consulted the **alias** table (surface spellings only) while `container_template_algebra_rows` — the same authority — already carries `"FreeMonoid": "FreeMonoid"`. Fixed by reading the authority that already knows, not by adding a row.

Per-entry verification, roots `dag` + `src/v2`: `accelerator_demo_plan` **0 blocking**, `realization_width_witness_test` **0 blocking**, `extdeps/mercurial` **0 blocking**, `host_effect_plan_witness_test` at its exact baseline pair with the empty-list gone.

## Receipts

- `--required-regen`: `first_generation_equal=true planned=132 executed=132 declared_divergent=1 [main.rs]`. The mirror on this branch is the **emitter's own candidate**, installed rather than hand-tuned — a hand-unioned mirror is consistent-looking and unattributable.
- The tree compiles (predicate 2). Regen green alone is a fixed-point predicate over the emitter, not a validity predicate over the emitted Rust; both are stated because neither implies the other.
- Exactly one mirror differs from `origin/main` (`v1_compiler_infer.rs`), so no measurement here is a hybrid of main's emitter against these sources.
- Precondition regime: these specimens red on a **stock** binary, not only on a tree carrying a deletion.

## Row (b), open, and the largest

Six seams plus `Pair<A,B>`. Joined this session to the `v2.compiler.tokenize` `apply` frontier row (7 occurrences) by a one-variable pair: `type Fold { delimited: fn(Thunk, Thunk) -> String }` compiles clean; `type Fold<R> { delimited: fn(R, R) -> String }` at `Fold<Thunk>` refuses with `receiver type 'Primitive()' establishes no method surface`. One cause covers both halves — a type-parameter-typed position loses its type at instantiation, whether that position is a field **value** (`Box<T> { v: T }`) or a lambda **parameter's declared type** (`open_r: R`). `LexMatchThunk` itself is concrete; the generic carrier is the algebra.

## Instrument notes, because two of them cost real time

- **Identical counts across two arms are not agreement — they are one instrument run twice.** A stage0 mirror patch staged in `/tmp` was destroyed by a container restart; the restore failed silently and an hour of "patched vs baseline" was baseline vs baseline, reporting 11/11 and 158/158. The `.dag` is the authority but **the mirror is what executes**; a patched `.dag` with an unpatched mirror builds clean and looks like a repair that changed nothing.
- **The whole-corpus compile now refuses by construction** rather than being SIGKILLed — `WholeCorpusCompileBudgetBelowMeasuredDemand` (`gunbc.whole_corpus_compile_admission`), which refused remotely at 7.06 GB available against 7.5 GB measured demand. Its own message names the trap: an exit-137 count is "a memorial to a killed process". Everything above is `--entry`-scoped for that reason.

## Pairing

The measurement of what this repair exposes is pre-registered in #8894 (population, `unexplained = 0`, file-grain join rule) **before** the exposure exists. eager-lark-892 is parked on this branch; they get the repair SHA and its parent before anything lands, so the population is never chosen after the answer is visible. The `algebra_genericity_pair` control is the chain's **terminal** red — `arm_b` red after (c) or (a) is expected and must not be weakened to green.

Ref: parent `19f0be86dd` via merge `ca11d6f683`. No number here is carried from an earlier ref.

