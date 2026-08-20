# A value in argument position can never be moved — so it is always cloned

**Subject.** `src/v1/ownership.dag` `walk_expr` / `make_decision` / `build_movable_set`, and the
`emit_info.movable` seam in `src/v1/05_emit_rust.dag`.
**Status.** Mechanism **confirmed by execution** — an additive probe on the compiled path flipped the
emitted output exactly as predicted, prediction stated before the result. **No repair has landed.**
**Ownership.** `src/v1/ownership.dag` has **no sole-write declaration** anywhere in `dag/gunbc`, and
git attribution names no current owner (last human commit 2026-07-06; everything since is
bot-squashed). There was nobody to route this to, which is why it is written down.

---

## 1. The mechanism

```
ExprVar { binding_kind: bk } =>
  if in_tail { record_use(…, kind: Consumed, site: "return", …) }
  else       { record_use(…, kind: Read,     site: "read",   …) }

fn semantic_consumer_count(usage) -> Int {
  usage.consumers |> filter(c => match c.kind { Consumed => true, _ => false }) |> count }

fn make_decision(usage) -> OwnershipDecision {
  let sc = semantic_consumer_count(usage: usage)
  if sc == 1 { SoleOwner { … } }
  else { if sc > 1 { SharedError { … } } else { Unclassified { reason: "no consumers found" } } } }
```

**`SoleOwner` requires exactly one `Consumed` edge, and `Consumed` is recorded only in tail
position.** Call arguments are walked with `in_tail: false`:

```
texpr.children |> fold(init: accum, f: (acc, a) =>
  walk_expr(accum: acc, texpr: arg_value(n: a), in_tail: false, si: si))
```

Across the walker: **17 `in_tail: false` sites, 2 `in_tail: true`.**

> **Therefore a value passed as a call argument is recorded `Read`, has `sc == 0`, decides
> `Unclassified "no consumers found"`, and can never be movable. It is cloned.**

It fails a **second** time independently: `whole_value_borrow_count` counts `Read` and `Threaded`
edges, and `build_movable_set` requires that count to be `0` — so the same `Read` that denied
`SoleOwner` also disqualifies on the borrow axis.

## 2. Confirmed by execution

Additive probe on the **compiled mirror** (`v1_compiler_ownership.rs` — see §6): `build_movable_set`'s
entire filter forced to `true`. Guards: probe marker present, the `SoleOwner` gate **gone**,
`lambda-capture` **restored to 1** so `movable` was the only variable, subject digest unchanged at
`812d95660a6dd909`.

```rust
BEFORE  Rc::new(move |x| bind_outcome(f(x.clone()), g.clone()))
AFTER   Rc::new(move |x| bind_outcome(f(x), g))

clone() in carrier: 6 → 2        total clone() across the whole emit: 329  (sensitivity control)
```

**The prediction was stated before the run.** `movable` governs the clone, and argument position is
what denies membership.

**Corroboration from the same run:** `cached_stage` became
`resolve_probe(lookup(x), stage, x)` — `x` moved **twice**, which is unsound. That is expected of a
maximal-permission arm (it is a *probe*, not a proposed repair) and it independently confirms that
`cached_stage`'s `x` genuinely needs a clone on its first use.

## 3. What this explains — including the evidence that refuted the earlier account

| specimen | position of the value | outcome |
|---|---|---|
| `then_outcome` `fn(x) { bind_outcome(f(x), …) }` | **argument** to `f` | `Read`, `sc==0` → cloned |
| `resolve_probe` `stage(x.clone())` | **argument** — and `x` is an ordinary fn param | `Read`, `sc==0` → cloned |
| witness Arm A `identity_direct(w) { w }` | **tail** | `Consumed`, `sc==1` → `SoleOwner` → **bare** |
| witness Arm B `fn(u) { w }` | tail *of the lambda*, then swept by the capture collapse | `Read` at enclosing scope → cloned |

**The existing witness is internally consistent and always was.** Its two arms differ on
*tail-vs-capture*, and **neither exercises argument position** — the position almost every value in
the corpus actually occupies.

`resolve_probe` is the cleanest specimen: an **ordinary function parameter**, used once, cloned, with
**no lambda anywhere**. It is in `param_names`, so the `is_owned_local || param_names` gate passes;
the suppression is entirely `make_decision`.

## 4. Population — not lambda-shaped

If argument position denies `SoleOwner`, this reaches **every value passed to any call, corpus-wide**.
That has nothing to do with closures, and it is far larger than any lambda-shaped count.

**Measured on this subject** (`staging.dag` `812d95660a6dd909`, 14 emitted files in **both** arms;
baseline guarded as UNPROBED — probe markers `0`, `SoleOwner` gate `1`, `lambda-capture` `1`):

```
BASELINE total clone() across the emit:  775
FORCED-MOVABLE comparator:               329
DELTA:                                   446        BASELINE total Clone bounds: 93
```

> **AT MOST 446 of 775 clone sites in this emit exist because a value could not be proven movable,
> and the achievable reduction is somewhere below that and unmeasured.**

**The qualifier is part of the sentence, not a footnote**, for two compounding reasons:

1. **329 is a floor no correct repair can reach.** The arm that produced it emits **unsound** Rust —
   `cached_stage`'s `x` moved twice, measured in that same run — so some of the 446 are load-bearing.
2. **This counts clone SITES, not `movable` membership.** It bounds the *consequence*; it does not
   observe the *cause*.

**Do not restate this as "58% of clones are spurious."** That is the sentence the figure decays into
once the qualifier is separated from it.

**What it does NOT settle — inert vs partial.** A near-empty `movable` set would mean the ownership
analysis is **inert** (coverage by illusion at subsystem scale); a substantially-populated one that
merely omits argument position means it is **partial** — ordinary, and much narrower. **This delta
cannot separate them**, because it measures the consequence. It establishes only that *whatever*
`movable` currently contains, the emitted output is dominated by values outside it. The inert claim
needs **set sizes** and its own evidence; it does not inherit this delta's.

**To get a floor beside this ceiling:** partition the 446 by single-use vs multi-use in the emitted
body — single-use sites are plausibly recoverable, multi-use ones (like `cached_stage`'s first use)
are not. That needs one more dispatch that **diffs the two emitted trees**, since both emits landed
in `mktemp` dirs that did not survive. Not free; not yet run.

## 4b. The single-use argument falls through BOTH sets

`movable` is not the only exclusion. `build_read_only_params` rejects it too, for a **different**
reason:

```
fn build_read_only_params(proof, param_names) -> Set<String> {
  proof.bindings |> map_values |> filter(usage =>
    set_contains(param_names, usage.name)
    && is_owned_local(kind: usage.binding_kind)
    && binding_fan_out(usage: usage) > 1
    && usage.consumers |> all(c => match c.kind { Read | Projected | Threaded => true, _ => false })) … }
```

`binding_fan_out` counts every non-`Threaded` edge, so a parameter used **once** has fan-out `1` and
fails `> 1`.

| | `resolve_probe`'s `x` — one `Read` edge |
|---|---|
| movable? | **no** — `sc == 0`, `Unclassified` |
| read-only? | **no** — `fan_out == 1` |
| result | neither category → emitted owned, and **cloned** |

> **The two sets do not partition the space, and the hole is exactly the single-use argument** — the
> case that should most obviously move, and the most common shape in the corpus. A parameter used
> **twice** by reference *does* reach `read_only`. Used **once**, it is cloned. That is backwards,
> and it is a second independent confirmation: two different predicates, two different reasons, same
> victim.

## 4c. The root: `in_tail` is a positional proxy for a semantic property

Consumption is not a fact about **where** an expression sits. Passing a value **by value** consumes
it, tail position or not; passing it where the callee takes a **reference** does not, tail position
or not. The walker uses `in_tail` to choose `Consumed`-vs-`Read`, so it answers *"am I being
returned"* and records the answer as if it were *"am I being consumed"*. **Those coincide only in the
return case — the one case the witness's Arm A exercises.**

**So the fix is NOT to thread `in_tail` through arguments.** That errs the other way: an argument to
a by-reference parameter genuinely *is* a borrow, and marking it `Consumed` would emit moves that do
not typecheck. **The discriminator is the CALLEE'S PARAMETER MODE, not the argument's position.**

And that discriminator **already exists beside the one that needs it**: `read_only_params_index` is
built in the *same fold* as `ownership_index`, keyed the same way, threaded into the same
`emit_info`. Not directly usable as-is — the walk that decides consumption runs while building the
proofs that produce the index, so a naive use is circular for mutually-referencing functions — but it
is an **unapplied** capability, not a missing one. Check it before designing a new mechanism.

## 5. TWO AUTHORITIES ANSWER ONE QUESTION, AND THE DIAGNOSTIC IS THEIR DISAGREEMENT

```
type params carrying Clone:  5  BEFORE  →  5  AFTER   (unchanged)
clone() call sites:          6  BEFORE  →  2  AFTER
```

**Four clone sites disappeared and not one `Clone` bound was shed.** That is not an anomaly — it is
what the structure predicts:

| authority | reads | produces |
|---|---|---|
| **clone emission** | the ownership walk → `emit_info.movable` | whether a `.clone()` is emitted |
| **bound synthesis** | the `.dag` **body shape** (AST predicates) | whether `T: Clone` appears |

Verified structurally: `src/v1/trait_bound_witness.dag` (73 lines) contains **zero** references to
`movable` or `ownership_index` — and **zero** to `emit_info` at all, which is the positive control
that it is not merely using a different name for the same input. The two are **disjoint**.

> **So two independent mechanisms answer one question — *does this type parameter need `Clone`* —
> from two different inputs, with nothing reconciling them. The E0599 is their DISAGREEMENT.**

The divergence runs both ways and **only one direction is visible**:

| | |
|---|---|
| bound says **no**, emitter clones | **E0599** — the diagnostic we chased |
| bound says **yes**, emitter does not | an unnecessary bound on the surface — **SILENT** |

And the silent direction is now **measured rather than hypothesised**: in the after-state above, four
clone sites vanished while five bounds stayed, so at least some of those five are bounds with no
corresponding clone.

**This is DESIGN §3's forked-authority shape**, and it is the correct statement of the defect —
neither *"a spurious clone"* nor *"a missing bound"*.

**Consequences for repair, which is the actionable part:**

- Fixing the **ownership** side alone leaves the bound side stale — *measured above*.
- Fixing the **bound** side alone entrenches the clones — §10's ordering argument.
- **Neither side is repairable in isolation.** A fix that does not make one side **derive** from the
  other leaves the divergence intact and merely relocates which rows are silent.
- The single-authority direction is forced: **the emitted clone sites are ground truth** — they are
  what the code actually does — and the bound is the requirement that *follows* from them. Bounds
  should be **derived from the clone decision**, not computed by a parallel AST predicate. That also
  dissolves the call-forwarding-widening programme: if bounds follow clones, a wrapper needs `Clone`
  exactly when it emits one, and there is nothing to propagate. *(A larger claim than this packet;
  it belongs to whoever owns bound synthesis.)*

**Consequence for the obvious cascade experiment** (*"repair the mechanism, count which bounds
disappear"*): it presumes bounds follow clones. They do not, so such a run reports **zero bounds
shed** and reads as *"no cascade"* — when what it actually measured is whether the two authorities
are connected at all. Separate those questions before running it.

**Surviving form of the board's blind spot:** clone counts and bound counts are independent axes, so
a corpus can carry a redundant clone *and* an unnecessary bound that no diagnostic names.
**Diagnostic counts are a FLOOR on defects, never a measure of them.**

## 6. Method — three instruments that could not answer, kept because the sequence is the lesson

**(a) The probe never reached the compiler.** `src/v1/ownership.dag` has a **stage0 mirror**;
`cargo build -p v1-compiler` compiles `src/v1/stage0/src/v1_compiler_ownership.rs`, **not the
`.dag`**. Measured with no build: probe markers `0` in the mirror, `lambda-capture` `1` (positive
control). Every provenance column was *true* and none answered the question — the `.dag` digest
changed (the edit arrived *on disk*), `Checking patch src/v1/ownership.dag` appeared in the runner
log (applied to a file nothing reads), and the rebuild was genuine (*of the unchanged mirror*). The
guard asserted the probe was present **in the source tree**; for a self-hosted compiler, *present in
the tree* and *present in the executed path* are different questions.

**(b) A subtractive probe cannot discriminate against an inclusion set.** `build_movable_set` folds
**qualifying names in**, so removing the capture collapse removes the entries *entirely* — and
**absence yields the same emitted clone as swept-as-capture**. So even with the collapse verifiably
gone from the executed path (`lambda-capture` = 0 in the mirror, guard passed), the null was
unreadable. **A control is not made decisive by being maximal.** The decisive direction was additive.

**(c) Reasoning about signatures while treating bodies as illustration.** An earlier draft made
`cached_stage` a zero-diagnostic/two-defect specimen. Counting the uses of `x` — `lookup(x.clone())`
*and* `x.clone()`, **twice** — shows its `A: Clone` is **required**. Retracted. The discriminator was
in the bodies, which were quoted and never examined.

**The guard that replaces (a) and (b):** assert on the **compiled** artifact, in both directions —
the probe marker must be present **and the construct being removed must be absent** — plus a positive
control so a zero is readable, and a **sensitivity control** (here: total `clone()` across the whole
emit) so "nothing changed" is distinguishable from "the probe did nothing".

## 7. The capture collapse — a real defect, but NOT this diagnostic's cause

Retained because it is real, demoted because the attribution was withdrawn on evidence:

```
ExprLambda =>
  let inner = walk_expr(accum: empty_usage_accum(), texpr: body, in_tail: false, si: si)
  let binding_merged = inner.bindings |> map_values |> fold(init: accum, f: (acc, usage) =>
    record_use(accum: acc, name: usage.name, kind: Read, site: "lambda-capture", …))
```

Every binding used in the body — **including the lambda's own binders** — is re-recorded as a
capture `Read` at the **enclosing** scope. Nothing subtracts the binder names, though
`lambda_param_names_at` is imported in that very file and used further down it. `ExprForEach` has the
identical shape (`"foreach-capture"`), the arm the witness note already flags **UNMEASURED**.

**This would matter for a binder in TAIL position.** It did not matter for `then_outcome`, whose
binder is in argument position and is denied by §1 regardless.

Three things learned while chasing it, all still true and all independent of the attribution:

- **`movable` is an inclusion set**, and a binder is *additionally* barred by
  `is_owned_local(binding_kind) || set_contains(param_names, …)` — the collapse records
  `binding_kind: none`, and a lambda binder is not a function parameter. **A subtraction-shaped
  repair was a no-op on its own terms.**
- **The analysis has one scope per FUNCTION; the language has one per CALLABLE.**
  `analyze_ownership(func_name, params, body, si)` produces one proof, and `ownership_index` is keyed
  by function name. Yet `04_infer.dag` already states the opposing rule — *"every callable boundary
  is a new declaration"* — which the return-conformance walk was fixed to respect and ownership never
  was. The repair is to do for a lambda what `analyze_ownership` does for a `fn`: seed the binders as
  that scope's params. **The seeding pattern already exists**, and note that function params are
  seeded with `binding_kind: none` and qualify via a **per-callable name set**, not a kind.
- **The emitter already re-scopes `movable` at descent points**, twice:
  `movable: set_insert(emit_info.movable, acc_name)` (fold accumulator) and
  `with(emit_info, { movable: tco_movable })` (TCO). A lambda descent is the same construction. **Do
  not key lambda scopes into `ownership_index`** — that would mint a name for a nameless entity;
  carry the set structurally at the descent point, as `tco_movable` does.

## 8. Identity — name-keying, and why it cuts both ways

`record_use` does `map_insert(accum.bindings, name, …)`; `span_start` is threaded in and lands on the
**edge**, never on the binding identity; `SoleOwner { binding: String }` carries a name; and
`effective_kind` is **first-non-`none`-writer-wins, permanently**. `ExprLet` walks the RHS into the
same accum, so `let x = x` folds an outer read and a new local into one entry.

**Shadowing is total, and that is load-bearing.** `04_infer.dag`'s `extend_scope` /
`extend_scope_with_params` both `map_insert` into **one flat map keyed by name** — no scope chain, no
stack — so binding a lambda param **overwrites** any outer entry for the whole lambda scope. Not "no
counterexample found": *no representation in which both are visible*.

> **Same root, opposite valence.** One name-keyed map with insert-overwrite makes the resolver
> totally shadowing **and** makes the walker unable to distinguish two same-spelled bindings.
> **Occurrence-keying the bindings map would make partial shadowing representable and therefore
> falsify any repair that relies on total shadowing** — if it ever lands, such repairs must be
> re-derived, not carried over.

Residual, narrower, and owed a **countable** guard rather than a silent partial: a `let` shadowing a
binder *inside its own body* (`fn(x) { let x = 5  x }`).

## 9. Fixtures

| arm | shape | assertion |
|---|---|---|
| **RED-if-broken** | ordinary fn param, single use, **argument position** (`resolve_probe`'s shape) | **no** clone — fails today |
| **REGRESSION** | value captured by an **escaping** `Fn` closure | clone **survives** |
| **PREMISE** | inner binder sharing a spelling with an outer binding | outer unreachable from the body |

The REGRESSION closure must genuinely **escape**; one invoked immediately keeps its clone for the
wrong reason and passes either way — a control that cannot fail. The PREMISE arm passes today by
construction and **goes red in the diff that breaks it**, in front of the person breaking it, rather
than relying on a future reader noticing a note.

## 10. Ordering

**Any binder/argument-clone repair must land BEFORE a widening of call-forwarding.** Doing
call-forwarding first would **consume the evidence**: once it manufactures the bounds, the spurious
clones compile and become invisible, and the only signal they were wrong is a diagnostic that no
longer fires — the absorbing-fallback shape of DESIGN §5, with a *repair* as the absorbing arm.

---

## Provenance

`ctrl-build` replays the working tree as patches onto a fetched base, so `git rev-parse HEAD` on the
runner names the **base** and is not provenance in either direction. Every run here is identified by
**content digest**: subject `staging.dag` `812d95660a6dd909` throughout; probe mirrors
`0548ebb56a2243ac` (subtractive) and `2460edde31b1e574` (additive). Source-level claims count
declarations and read the walk, so they do not rot when the emitter changes.
