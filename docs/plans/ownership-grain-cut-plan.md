# Ownership grain cut: clone-vs-move is a use-site property

Delete-first replacement (DESIGN §3). Integration branch `integration/ownership-grain-cut`,
forked from `origin/main` `7f5fa6a94e`; root deletion is the first commit (`132359ba11`).
The branch is **deliberately red** — the deletion is the census, and greenness would damage
its primary artifact.

## The root

`v1.compiler.ownership` `make_decision` answers per **binding**. Clone-vs-move is a property of
a **use site**: a fold accumulator is borrowed at uses 1..n-1 and moved at use n. A per-binding
authority structurally cannot express that.

The three predicates above it are not peers of the defect — they are the compensating
structures the grain error forced into existence, each a different guess at the missing
per-use answer:

```
build_movable_set        narrows make_decision with whole_value_borrow_count == 0
build_read_only_params   a second, independent predicate for param borrows
owned_bindings           ad-hoc, emit-time, never derived from ownership.dag
```

None answers for a fold accumulator, so all three fall through to one default: clone. That
default is the §5 violation — *could not determine ownership* rendered as *clone everything*.
Correct, silent, unpriced, and therefore never ranked for fixing. Deleting the default inverts
the burden from "prove this clone is removable" to "prove this clone is required", and `rustc`'s
E0382/E0505 are the typed, located refusals that prove them one at a time.

## Terminal shape

| role | carrier |
|---|---|
| **primary** | per-use-site verdict — the grain the defect is at |
| **derived** | `ValueMaterialization` as a **fold over** per-site verdicts |
| **deleted** | `src/v1/ownership.dag` — per-binding, carries no site anywhere |

`std.materialization_ladder` is **not** deletion population. It already carries
`ValueConsumerEdge { access, site }` — the per-site *input* — and discards it in the fold. Only
the fold order was wrong: collapse **after** deciding, never before. The 2026-07-09
consolidation onto materialization survives unchanged.

### `UseSiteVerdict` and `ProviderTier` are two concepts, not one named twice

`Borrow`/`ReferenceTier` and `CloneShared`/`CopyTier` look like nicknames and are not. The
relation is **selection, not identity**: the verdict is the agnostic decision, the tier is the
realization it selects. `ProviderTier` additionally carries `MemoTier`, `ArtifactTier` and
`CasTier`, which have no use-site ownership reading at all — it is the §2 horizontal axis
spanning nanosecond memoization through content-addressed storage. Fusing them would put the
dispatch inside the agnostic decision, which §3 forbids at every interface/realization seam.

The arity mismatch is the tell, in both directions: `AccessMode` has four arms
(`ReadAccess | ConsumeAccess | CarryAccess | ProjectAccess`) against `UseSiteVerdict`'s three
usable arms plus `Unclassified`. Two coproducts of different arity are not the same coproduct.
**Do not rename either carrier.** §3 forbids two names for one concept; consolidating here
would delete a real distinction, and net concepts must not shrink by fusion any more than they
may grow by re-invention.

Placement: `UseSiteVerdict` needs nothing target-specific — it imports only `v2.std.node`,
`v2.std.collection`, `v2.std.algebra`, and `MoveField { field: Symbol }` names a field with a
substrate `Symbol`. Move/borrow/copy are affine readings of access; Rust *enforces* them, which
is why they were discovered there, but it did not author them. Its `v2.compiler` home is an
accident of authorship, not a dependency.

## Required property of the fold

The fold from per-site verdicts to `ValueMaterialization` **must derive its three counts
separately** — `verdict_take_count`, `verdict_copy_count`, `verdict_reader_count` — never one
count reused three ways.

**Superseded, and recorded rather than silently edited:** an earlier revision of this paragraph
required preserving X's two *exclusions* — `CarryAccess` excluded from plurality, `ProjectAccess`
excluded from movability. Those exclusions are X's rule, and parity with X was never the bar
(§3). Y drops both deliberately — a live projection borrow blocks a whole-value move, and a
carried value is borrowed at the site that carries it — declared in
`value_site_verdict_semantic_change_note` with the affected population proved empty. This
paragraph survived that decision and went on stating the old requirement, so for a while the plan
and the carrier disagreed about the rule: the §3 stale-claim class, in the document naming it.

Folding through a single count re-collapses the three readings into one — the original defect
wearing the new model's clothes. This is a **required property, not a preference**, and it is
what the discriminating evidence must witness: a fold that preserves three readings, with a RED
that catches a collapse to one.

## Census instrument — do not "simplify" this to `cargo build`

The census runs through `regen_stage0`, which compiles the `src/v1` `.dag` corpus.

**`cargo build` alone reports GREEN here.** Deleting the `.dag` authority leaves its emitted
twin `v1_compiler_ownership.rs` untouched on disk, so the Rust build never sees the deletion —
a false clear at the vehicle layer rather than in any gate. The subject of this cut is a
*generator*, so the standing question is not "did I break a caller" but "what did I change the
bytes of, and who checks those bytes".

Related and known-partial: 143 artifacts are declared-emitted and 2 are registry-checked, so a
green drift gate is not proof that emission is unchanged where it should be.

## Bounds held

- **The 359 clone sites are not a burn-down.** The operator verdict of 2026-07-29 on
  `dag/tools/e0599_emitter_decision_census.dag` stands: the count records where the emitter
  inserted a clone, never that any site is removable. `Rc::make_mut` is clone-on-write and
  clones whenever another strong `Rc` is live, so it requires the very bound the census counts.
- **This is not the fix for the 90-minute CI run.** Reconcile is the ~10-minute shared
  preparation item; per-witness resolve is the rest, where `reconcile_assembly` measures 1–3%.
  The payoff is corpus-wide, not reconcile-specific.
- **Mechanism confirmed, magnitude corrected, share unmeasured.** The shape is real and fires:
  `acc.clone()` holds the strong count at 2 across the insert, so `Rc::make_mut` copies on every
  iteration (`strong_count == 2`, allocation moves — measured). The **magnitude is ~9×, a
  constant factor, not an order of magnitude.**

  A benchmark of mine reported 638× at m=4000 and is **WITHDRAWN as evidence about the emitted
  fold**: it used `std::collections::HashMap`, while `v1_rt.rs:6` reads
  `use im::{HashMap, OrdSet as BTreeSet, Vector as Vec}`. A whole-container copy is O(m) in
  `std` and O(log m) in an `im` persistent map — which is why `im` exists. Same mechanism,
  different cost class. Measured side by side on one binary: `im` holds ~8–9× flat from
  n=1000 to n=4000 while `std` climbs 196× → 817×.

  **The repro trap, stated so it is not repeated:** a repro reproduces the code you *read*, and
  the imports are not in the code you read. Re-authoring a `use` line as the std default is the
  Rust reflex, and here the std default is the slower asymptotic — so the bias inflates. Assert
  the repro's type against the subject's type before running any grid.

  Mechanism and share still never travel in one sentence: none of this measures the defect's
  share of any phase wall.
- **Behavioral equivalence, never byte-matching** (§7). A byte-identical fixed point would
  force the replacement to reproduce the seed's accidents.

## Quarry, not authority

`docs/plans/emitter-ownership-defork.md` and `docs/plans/rc-ownership-wrap-decision-design.md`
are read as area maps only. Both were reasoned while the per-binding authority was alive, so
their sequencing carries its gravity; per the 2026-08-15 ruling an existing design is evidence
about X, never authority over Y.

## Standing: decision-complete, producer-incomplete

Classified against DESIGN §3's three-way — *surviving obligation* / *existing reliance* /
*obsolete behaviour* — not a two-way retain-or-delete. The two-way reading fails toward
"cut complete", which is the comfortable arm.

| | |
|---|---|
| **superseded, discharged** | the per-binding *decision* — `make_decision` → `value_site_verdicts`, landed and witnessed |
| **deleted with obligations open** | the *producer* — nothing derives `ValueUsage` from the emitter's graph |

Every `ValueUsage` in the tree is authored by a witness. The emitter still consults the deleted
authority's shape, and these are the counted sites:

```
src/v1/05_emit_rust.dag   build_movable_set call            (ownership_index construction)
src/v1/05_emit_rust.dag   fn_movable lookup                 (per-fn movable read)
src/v1/04_emit_info.dag   movable: empty_set()
src/v1/04_infer.dag       movable: empty_set()
src/v1/05_emit_rust.dag   movable: empty_set()
```

### The obligation — durable home, ruled 2026-08-16

This document is the **primary home** for the obligation below, deliberately: a closing lane
cannot hold an open obligation, and a PR body evaporates from view the moment the PR merges.
Ruled: record here at symbol grain; **do not** mint a carrier under a load-bearing module, and
**do not** spawn a work item yet — the ownership question crosses a lane boundary and is a
program decision.

```
owed         a ValueUsage producer over the real emitter graph, so value_site_verdicts has a
             SUPPLY rather than witness-authored inputs
blocked on   which emitter exists after v1-cut. src/v1/05_emit_rust.dag — the graph the
             producer would derive from — is in v1-cut's deletion set, with the emitted .rs
             frozen. The CAPABILITY is owed by Y and outlives that; only its INPUT SOURCE is
             undetermined.
not owed by  any retention roster, and not by the frozen-emitter row. A retention row says
             someone may still BUILD an artifact; this says someone must still EARN a
             behaviour somewhere else.
```

Starting work on it before the emitter has a post-cut home would mean deriving from a graph
being deleted underneath the work.

**The obligation, stated so it is countable rather than implied:** derive `ValueUsage`
(per-site `access` + `site`) from the emitter's graph, so `value_site_verdicts` has a real
producer and the three `empty_set()` constructions are replaced by a derivation rather than by
a default. Owned by this cutting lane — not by any retention roster, since *a retention row says
someone may still build an artifact; an open obligation says someone must still earn a behaviour
somewhere else*, and parking the second inside the first hides it from the lane that owes it.

> **A landed decision is not a landed producer.** Evidence that the right answer is computable is
> not evidence that anything computes it on the real input. The fold landed green with a
> discriminating RED, which is exactly what made this invisible.
