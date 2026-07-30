# Inner cost lanes — scoping

Scoping only. No code lands from this note. It exists so the two remaining CI-cost lanes can be
picked up without re-deriving what they are, and so the broad "why is CI slow?" investigation stays
closed (operator ruling 2026-07-30: every subsequent PR carries only its own discriminating
before/after receipt).

These are the lanes left after the workflow-graph cleanup. The workflow work removed *duplicated
process startup*; these two are about *the work itself* — what the frontend does per byte, and what
resolve does per entry.

---

## Lane A — frontend construction

**Contained slice: `process_escapes_loop`.**

`src/v1/stage0/src/v1_compiler_tokenize.rs:882`. It is the one function in its file still walking a
raw `String` by index while the rest of the file has migrated to the pre-decoded `SourceRef` /
`source_chars` idiom. That is the whole defect: the surrounding file already has the fix.

### The cost shape, read off the primitives

Both primitives it calls are branch-on-ASCII (`src/v1/stage0/src/v1_rt.rs:251,266`):

| call | ASCII | non-ASCII |
|---|---|---|
| `char_at(s, pos)` | `bytes[pos]`, O(1) | `s.chars().nth(pos)`, **O(pos)** |
| `string_length(s)` | `s.len()`, O(1) | `s.chars().count()`, **O(n)** |

`process_escapes_loop` calls `string_length` on the *same unchanging source* on every iteration —
several times per iteration across its branches — and `char_at` at up to four offsets per iteration.

So the realized cost is:

- **ASCII input: O(n)** in time, but with one heap `String` allocated *per character*, accumulated
  into `Rc<Vec<String>>` and `join`ed at the end.
- **Non-ASCII input: O(n²)**, twice over — once through `char_at`'s `nth`, once through the
  re-counted `string_length`.

The non-ASCII path is the one that bites, and it is not exotic: any string literal containing a
non-ASCII character anywhere flips the entire scan for that literal onto the quadratic branch,
including the ASCII portions of it.

### The fix

Migrate the loop onto `source_chars` (the pre-decoded array the rest of the file already builds), so
both `char_at` and `string_length` become O(1) regardless of encoding, and replace the
`Rc<Vec<String>>` accumulator with a single `String` buffer. That removes the quadratic *and* the
per-character allocation in one motion — the accumulator-copy class DESIGN §6 already rules is always
fixed regardless of realized n, because "n is small here" is not a time-stable fact.

### Oracle

Byte-identical token streams over the corpus is the *equivalence* half. It is not sufficient on its
own — it is satisfied by not changing anything — so it needs a discriminating input beside it: a long
string literal with non-ASCII content and escapes, which separates the two implementations
asymptotically, plus a RED that a wrong escape decode changes the output. Both halves execute, per
DESIGN §5: a typecheck and a grep are not consumers.

### Beyond the slice

The named generalization is `SourceCursor` / `TokenBuilder` / `FrozenTokenStream` and enforceable
generated-boundary cost rules — the point being that a *rule* makes the next such function unwritable
rather than found later. That is downstream of the slice, not a prerequisite for it, and it should be
priced against a second measured instance rather than authored speculatively (the purity trap, §6).
Task #3 tracks auditing the v2 parser for the same class; if it is clean, the generalization is worth
less and should say so.

---

## Lane B — union resolve

**This is a program, not a PR.** Saying so is the useful part of the scoping.

### The measurement

One entry resolve, measured this session on `dag/tools/floor_effect_gate_witness.dag`:

```
[resolve] 107422ms (528 modules, 12067 resolved items in closure)
[resolve-split] load=38491.9ms parse=5026.3ms resolve=184.0ms normalize=331.3ms
                typecheck=74677.6ms parent_envs=7.4ms reconcile_assembly=4418.3ms
```

**These are inclusive counters, not an exclusive partition, and the difference matters before anyone
ranks work by them.** `load + typecheck` alone is 113169.5ms against a 107422ms total, and all
displayed components sum to roughly 115% — so the phases overlap or nest, and no component's share
of the total can be quoted as its exclusive cost. An earlier draft of this note did exactly that
("typecheck is 69%, load is 36%"); those figures are withdrawn.

What survives the correction is the ordering claim, which does not depend on an exclusive partition:
the step literally named `resolve` is 184ms against a 107422ms whole, so whatever the overlap
semantics, it is not where the time goes. The cost is in loading and typechecking, and the lane name
is a nickname for the whole entry-graph construction. Anyone attacking "resolve" by its name will
optimize the wrong thing.

Deriving an exclusive partition — or documenting which counters nest inside which — is the first
thing the measurement slice below should produce, because the union case has to be priced against
real per-phase attribution rather than against inclusive totals.

### The claim

The floor reconstructs a `ResolvedGraph` **per entry**. The affected-set corpus batch selects
hundreds of entries whose closures overlap heavily — 528 modules for one entry, against a corpus
where a typical main-push selects ~600 entries. Resolving the *union* of the selected entries once,
then serving each entry from it, is the same work performed once rather than per-entry.

### Why it is a program

It is not a local optimization with a local oracle:

- It changes the retention profile, which is the axis the floor already fails on. DESIGN's v1
  run-stability thread records the root cause as *retention, not footprint* — multiplicative
  inductive-field duplication along the import DAG — and the eviction work (M2) is explicitly shelved
  behind measured triggers. A union resolve holds *more* live at once by construction, so it collides
  with exactly the constraint that is already binding.
- Its acceptance is fleet memory, not wall time: same verdicts, no material increase in cgroup peak,
  hard backoff, or throttle-wall, at the arm64 slot's real budget. That verdict cannot be produced in
  a dev container with a different memory regime.
- It interacts with the shared-index and memo work already in flight (`walk_memo`, the entry-closure
  memo landed at #6999), which serve part of the same purpose and must not be forked.

### Sequencing

Do not open this until the floor's retention story has a measured floor to build on. The honest first
step is a *measurement*, not a change: instrument how much of the per-entry resolve cost is genuinely
shared across the selected set, so the prize is known before the architecture moves. If the overlap
is smaller than assumed, the program is worth less and the memo lane already captured most of it.

---

## What is deliberately not here

No ranked lever table. The previous attribution document's ranked table went circular — it named
byte-cursor work as lever 1 while its own later sections had located the root in frontend
construction — and that circularity is why the investigation was closed. These two lanes are named,
priced, and sequenced; the next receipt should come from a PR that changes one of them, not from
another census.
