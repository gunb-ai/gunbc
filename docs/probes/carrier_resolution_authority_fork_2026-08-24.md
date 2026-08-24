# The carrier-resolution authority fork — census, scope and the constraint any dissolution must preserve

**Subject.** Whether a method call on a collection carrier resolves is decided by **two
independently-authored maps that nothing joins**. A name present in one and absent from the other
is admitted by the map that has it, dropped by the map that does not, and **fails at a stage that
names neither map**.

**Why this document exists rather than a PR.** The repair was attempted as rows inside
`gunbc#9059` (carrier split: `PointwisePower` / `FinitePowerSet` / `FinitelySupportedFunction` /
`PartialFunction`). That PR was closed by ruling: 24 of its 25 changed files intersect the #8282
namespace cut, so the *diff* cannot survive the cut and would be re-derived against a moved tree
anyway — and re-derived from a shape that had **grown** the fork. The *relationships* below are
invalidated by nothing the cut does, because they describe structure rather than lines. This
document is what the post-cut migration is built from.

**Status of every figure here: MEASURED**, at `origin/main` `bd84f669681` (2026-08-24), unless a
line says otherwise. Counts are from the tree, not from memory.

---

## 1. The two authorities

| | authority | question it answers | consequence of absence |
|---|---|---|---|
| A | `std.algebra` `kernel_algebra_profile_value` | **what methods does this have** | `establishes no method surface` |
| B | `std.types` `container_template_alias_rows` | **is this a container at all** | receiver resolves to its own record shape |

B is read by `04_types.dag` `is_declared_container_alias_spelling`, and `04_lookup.dag`
`resolve_method_receiver_type` **short-circuits** on that predicate to return the receiver *as
authored*, with its type arguments intact. Absent from B, the other branch is taken: the receiver
arrives at method lookup as a bare leaf, so `lookup`'s declared `OptionalOf { inner: ReceiverValue }`
has no `ReceiverValue` to substitute and the optional collapses.

**Both failures were observed, on one carrier, from one missing pair of rows** (`PartialFunction`,
during #9059):

- absent from **A** — `establishes no method surface` on a `PartialFunction` receiver
- absent from **B** — four × `variant 'Present' not found in type 'InferredFacts'`

Two diagnostics that look unrelated, from one missing join. **The second was predicted to be a
knock-on of the first and was not** — the prediction is recorded because a plausible mechanism is
exactly what stops you reading the producer.

## 2. The key sets, measured — and why they are *correctly* different

```
A  kernel_algebra_profile_value   Int Float Bool String List Set Map
B  container_template_alias_rows  List list Set set Map map
```

`Int`, `Float`, `Bool` and `String` are in **A** and **correctly absent from B**: they are scalars
with a method surface and they are not containers. The asymmetry is semantically right, not drift.

**This kills the "join" framing outright.** There is no derive-one-from-the-other relation for a
single row to dissolve — the two maps were never the same key set, and making them the same would
be wrong. Anything that "deduplicates" A and B by equating their keys has introduced a defect.

## 3. What a canonical authority must therefore carry

Per algebra, in one row:

- its admissible **spellings** (including the lowercase surface forms)
- its **method profile**
- whether it **resolves as a container**

with A and B becoming projections of it. That is **a new authority plus the retirement of two**.

**Scope — four files, spanning both `v1` and `dag`:**

| file | what it holds |
|---|---|
| `dag/std/algebra.dag` | `kernel_algebra_profile_value` (authority A) |
| `dag/std/types.dag` | `container_template_alias_rows` (authority B) |
| `src/v1/04_types.dag` | `is_declared_container_alias_spelling`, `container_alias_canonical_spelling`, `container_kind_canonical` |
| `src/v1/04_lookup.dag` | `resolve_method_receiver_type` — the short-circuit |

This is a **replacement migration with a census**, not a join and not a row. It is sized
accordingly, and it must not be attempted as an addition to whichever map is currently missing a
key — that is the attractor §3 names, and it makes the eventual cut larger every time it is used.

## 4. THE CONSTRAINT A DISSOLUTION MUST PRESERVE — the ordering, not just the answer

**The two questions are asked in order, not symmetrically.** The container question (B) is
short-circuited on *before* the profile (A) is consulted. That ordering is why a name in one map
and not the other fails at a stage naming neither.

**A canonical row that flattens A and B into one lookup would silently change which stage refuses.**
The refusal moves to a stage with a different diagnostic and a different owner — and it moves for
inputs that are *already broken today*, so nothing goes red to announce it. A dissolution that
preserves the answer but not the ordering is **not a dissolution; it is a behaviour change wearing
one**.

This is the hazard most likely to be walked into, because "merge two maps into one authority" reads
as obviously correct and the ordering is not visible in either map — it lives in a caller, in a
different layer, in a file neither map's author needs to open.

**Minimum bar for the migration:** it must preserve every refusal *and the stage each refusal comes
from*. Reproducing the set of refused programs is not sufficient.

## 5. A second trap in B, independent of the fork

`04_types.dag` `container_alias_canonical_spelling` folds over `sorted_map_keys` and returns the
**first sorted key** mapping to an algebra. While each algebra has exactly one spelling pair this
is invisible. Add a second and sort order silently becomes policy:

> a `"FinitelySupportedFunction"` key sorts ahead of `"Map"`, so every `Map` in the corpus acquires
> a new canonical spelling — **no diff names it, no refusal fires**, and the edit that caused it is
> one row in a different file.

This is why #9059's repair added only `"PartialFunction"` and deliberately left
`"FinitelySupportedFunction"` out, despite the symmetric row looking obviously correct.

**The general shape:** a total lookup returning *a* member of a set where the ordering is an
implementation detail until the set has two elements, at which point it becomes policy. The fix is
not to sort differently — it is for the rows to carry **which spelling is canonical**, so the answer
is authored rather than emergent. A canonical authority (§3) subsumes this: `spellings` with a
designated canonical one.

## 6. What is not claimed

- No count here is a budget, a ratchet, or an oracle. The key sets are the tree's contents at one
  ref and rot the moment either map is edited; re-measure rather than citing these.
- The rung is **mitigatable**. Nothing detects a name added to one map and not the other; the
  failure is discovered by whoever next authors a carrier, at a stage naming neither map. The
  next-rung trigger is the canonical authority above — at which point a spelling with no profile,
  or a profile with no spelling, becomes unrepresentable rather than merely undetected.
- The four-file scope is the scope of the *authorities and their readers*. It is not a claim about
  how many call sites change, which was not measured and which the #8282 cut will move.

## 7. Provenance

Found twice on 2026-08-24 from opposite directions — this lane via a carrier split, and
`quiet-pike-368` via the class it inverts — with each finder concluding it was out of scope for
their own change. That agreement is how a known defect becomes nobody's, which is the reason it is
written down here with a scope and a constraint instead of being carried in two threads.

The in-tree marker is `src/v1/04_infer.dag` `unresolved_method_frontier_note`, whose residue this
is upstream of: every row that note holds is an upstream *receiver-resolution* defect rather than a
method-existence fact, and a receiver decided by two unjoined maps is precisely that kind.
