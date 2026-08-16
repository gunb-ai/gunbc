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

The fold from per-site verdicts to `ValueMaterialization` **must preserve the ladder's three
plurality readings** — `take_count` (Consumed only, the affine axis), `value_access_plurality`
(`CarryAccess` excluded, the reference-eligibility axis) and `borrow_count` (Read + Carry, not
Project, the movability axis). `CarryAccess` and `ProjectAccess` are the reduce-spine and
field-projection axes the ladder separates deliberately.

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
- **Mechanism measured, share unmeasured.** A benchmark replicating the emitted fold shape
  (`Rc<HashMap<K,Rc<V>>>` threaded through `rc_map_insert(acc.clone(), ..)` vs. moving `acc`)
  gives clone slope 2.0 against move slope 1.0, 638× at m=4000. That establishes the
  **mechanism** is quadratic and says nothing about its share of any phase wall. The two
  claims never travel in one sentence.
- **Behavioral equivalence, never byte-matching** (§7). A byte-identical fixed point would
  force the replacement to reproduce the seed's accidents.

## Quarry, not authority

`docs/plans/emitter-ownership-defork.md` and `docs/plans/rc-ownership-wrap-decision-design.md`
are read as area maps only. Both were reasoned while the per-binding authority was alive, so
their sequencing carries its gravity; per the 2026-08-15 ruling an existing design is evidence
about X, never authority over Y.
