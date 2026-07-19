# Declared source refs as a selection-edge source — the cssl flagship

**Status:** DESIGN NOTE for review. No code lands from this document.
**Lane:** module-identity vs storage (wise-bee-768), task 5. Rulings: parent 2026-07-19.
**Parent:** [module identity vs storage design](module-identity-storage-binding-design.md) §3.
**Builds on:** `SourceRef` (#6889, declaration only) · `ModuleStorageIndex` (#6890).

---

## 1. The defect, and why it is not a one-file fix

`dag/tools/self_host_03_normalize_behavioral_transport.dag` declares its input as a
bare string:

```
data sn_source_rel: String = "src/v2/compiler/03_normalize.dag"
```

That is a load-bearing dependency edge living in a string literal — invisible to the
affected set, the resolver, and content-addressing (§4: a program is `Node` + `Edge`).
The priced incident is on record: #6775's 705-error regression shipped through this
class and was caught by hand.

**Measured scope (survey by shape, both source roots):**

```
pattern: data <name>: String = "<src|dag>/....dag"
=> 116 instances
   dag/tools 32 · dag/test/claim 31 · src/v2/workflow 16 · src/v2/test/claim 7 · dag/gunbc 5
```

So the flagship is **1 of 116**. It proves the shape; it does not exhaust the class.

**The ref must be storage-shaped, not `.dag`-shaped.** The flagship itself carries
`.rs` paths — `sn_shim_lib_rel`, `sn_shim_driver_rel`, and every
`CuratedSeedLinkedShimWrite.source_rel`. A ref admitting only `.dag` would fail its own
flagship. This matches the Q2 ruling already recorded for `SourceRef`: it names a
*storage realization* (path + source root + `ContentHash`), and module identity joins
through the binding rather than being carried inside.

## 2. The blocking half: there is no consumer

Verified read-only against `main`:

- **`SourceRef` has zero consumers.** One hit corpus-wide — its own declaration. Stage 0
  landed the type only, exactly as scoped.
- **`v2.lens.effect_reach` covers this class today** by string-literal census, and names
  this work as its own dissolution trigger.
- **`effect_reach` has no consumer in `src/v2/workflow` or `dag/gunbc`.** The census
  *classifies*; the selection path never *reads* it. `effect_reach_touched_by_paths(classification, touched_paths)`
  is the right-shaped predicate and nothing in the CI/affected-set path calls it.

Therefore the fixed acceptance criterion — *receipt selected on emitter-touch, skipped on
unrelated, both by execution* — **cannot be met by swapping a `String` for a `SourceRef`.**
Declaring a typed ref that nothing consumes moves the string into a nicer box and changes
no behaviour: the receipt would still be selected by the fail-closed default rather than by
its declared dependency, and the acceptance test would pass for the wrong reason.

The missing half is a **selection-edge source**: a path by which declared refs reach the
affected-set decision. That is this task.

## 3. Shape

**Selection reads declared refs joined through `ModuleStorageIndex` — never a
hand-maintained path list.** The ref names a storage realization; the index is the derived
authority mapping storage to modules (#6890). Selection asks "does the touched set
intersect the refs this row declares", resolving each ref through the index.

The task-3 keystroke rule applies verbatim and is the review question for this PR too:
**what decides correctness is what flows in, not the arity.** A selection path fed a
constant path list would satisfy the same signature and be a parallel ledger. The refs must
flow from the declaring rows, resolved through the derived index.

## 4. The anti-#6775 guard: the default never flips

**Per-row opt-in migration.** A row *without* a typed ref keeps the
`ReadsLiveTree` / never-predict-skip default. Precise selection exists **only where refs are
declared**.

This is the constraint that makes the dangerous failure unwritable rather than merely
unlikely. The catastrophic direction is a **false skip** — a PR touching the emitter whose
receipt is skipped because selection believed a wrong key. That is cache impurity in its
purest form: *the key is wrong before the cache exists*. Because undeclared rows never
become skip-eligible, a row can only be skipped on evidence it declared itself; silence
never buys precision.

Corollary: this work cannot regress the 115 rows it does not touch. They keep exactly the
behaviour they have today.

## 5. Failure arms — typed, counted, and widening only where declared

Every arm refuses; none silently narrows.

| condition | arm |
|---|---|
| row declares no refs | `ReadsLiveTree` default — never predict-skip (§4 above) |
| ref declared but unresolvable through the index | **refuse to the RUN side**, typed and counted |
| ref resolves, touched set intersects | selected — run |
| ref resolves, no intersection | skipped |

The second row deserves its name. Widening-to-run *is* the fail-closed arm here, and it is
the one case where widening is correct — but only because it is **declared, typed, and
counted**, so its frequency is observable and prioritizable. That is the distinction §5
draws between a refusal and an absorbing fallback: the absorbing form is silent and its
deficit's frequency is zero by construction; this one is counted, so a rising count is a
signal that the ref model is inadequate.

**The falsifier cold cadence is the divergence catcher.** It runs the corpus cold with
predictions recorded, so a *missing* selection edge — a row that should have been selected
and was not — surfaces as a counted divergence within one cadence window rather than as a
regression someone finds by hand. This is what makes per-row opt-in safe to grow.

## 6. Acceptance — two directions, both by execution

Neither direction alone is evidence:

1. **Touch the emitter** (`src/v2/compiler/03_normalize.dag`) → the cssl receipt is
   **selected**.
2. **Touch something unrelated** → the receipt is **skipped**.

Direction 2 is the one that can pass vacuously and must not: if selection is broken open, it
also skips when it should run, and only direction 1 catches that. Both are witnessed by
execution, not asserted.

## 7. Non-goals — stated so scope cannot drift

- **No 116-row sweep.** Flagship only. The class-wide de-string is a **counted follow-on
  lane** (115 remaining rows after the flagship), not this task.
- **No content-hash caching.** `SourceRef` carries a `ContentHash`, but using it as a
  selection *key* belongs to the resolve-cost lane.
- **No rewrite of the affected-set fold.** This work **adds a selection-edge source**. If the
  seam turns out to require surgery on the fold itself, that is a routing question:
  **stop and report back** rather than widening this task into the affected-set lane.

## 8. The effect-reach lens stays live

Flagship de-string is **not** class extinction — 115 rows keep their bare strings, so the
census lens remains the backstop for all of them.

**Checked (constraint (f)):** neither trigger fires on flagship-only. Both are phrased as
class-extinction conditions —

- `v2.lens.effect_reach` `effect_reach_law`: *"Dissolves when typed SourceRef at host
  boundaries makes bare-string file dependencies **unwritable**."*
- `v2.std.effect_reach` `effect_reach_authority_note`: *"…until typed SourceRef boundaries
  make bare-string dependencies **unwritable**."*

"Unwritable" is the correct bar and one flagship does not meet it, so the wording is already
safe and needs no change on that axis.

**One ambiguity worth tightening in the implementing PR:** stage 0 landed *a typed
`SourceRef`*, so a reader could take "typed `SourceRef` at host boundaries" as already
satisfied and fire the trigger early. It is not — the trigger needs the boundary *enforced*
and the class *migrated*, not the type *declared*. The tightening is to say so explicitly,
naming declaration-vs-enforcement, so the marker cannot be read as fired while 115 rows
still write bare strings.

## 9. Staging

0. **This note reviewed.** (Model before implement — the order that made stage 0 land clean.)
1. **De-string the flagship** onto `SourceRef`. Small, and inert on its own.
2. **Add the selection-edge source** — declared refs resolved through `ModuleStorageIndex`,
   per-row opt-in, arms per §5. The real work.
3. **Prove both directions by execution** (§6), plus the lens-wording tightening (§8).

Stage 1 is safe alone; stage 2 is where the risk is. The interim is already fail-closed —
today's default is never-skip — so the only price of arriving deliberately is scale, not
correctness.
