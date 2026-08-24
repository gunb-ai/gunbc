# The name-derived loader resolves by falling through to a whole-tree pool, and the instrument watching it is blind to the failure that bit

**Status of every claim: verified by reading the live tree at `bd84f669681` (2026-08-24).** Symbols
are named; the two line references are given because they sit inside a function body, where §3's
citation rule admits a position beside the symbol.

---

## 1. The question

`v2.compiler.program_assembly` refuses with 26 name-resolution diagnostics when emitted alone
(gunbc#9083). Its author's diagnosis generalises past the module:

> The definers sit in the pool because some *other* module imported them, so the unlisted names bind
> by **pool coincidence**, and the module's own header can be arbitrarily wrong without anything
> going red.

The namespace cut (gunbc#8282) deletes import headers corpus-wide in favour of a name-derived
loader. So after the cut **every module is in that position by construction**. Which raises exactly
one question:

> **What does the name-derived loader do that the scoped closure does not?**

If it derives edges structurally from name homes, pool coincidence is removed. If it searches a
wider pool, pool coincidence is *promoted from an accident to the mechanism*.

## 2. The answer, from the loader

`v1_compiler.cli_run` `build_both_closure_edge_index` (`src/v1/stage0/src/cli_run.rs`, ~7870):

```rust
let target_module = match resolve_in(&census) {
    Some(m) => Some(m),
    None    => resolve_in(&pool_bare_census(index)?),
};
```

`census` is the **scoped** index. `pool_bare_census` (same file, ~14832) is memoized over
`pool.nodes_by_file` — **every node in every file in the pool**:

```rust
let nodes: im::Vector<Rc<Node>> = pool.nodes_by_file.iter().map(|(_, node)| node.clone()).collect();
```

So when the scoped lookup misses, resolution falls through to a whole-tree symbol index. **After the
cut the scoped miss is the normal case**, because there are no headers left to scope by.

**The answer is therefore the bad one: it searches a wider pool.** The cut generalises pool
coincidence rather than removing it, and `04_infer`'s wrong-definer blocker is the first *observed*
instance of a class rather than a one-off to repair.

## 3. Why the existing instrument cannot see it

There is an instrument. `[floor-bare-name-ambiguity]` reports, under a comment describing "the
silent pick, counted... a resolution nothing the author wrote authorizes — reported, not refused":

```
[floor-bare-name-ambiguity] scopes_affected={} of {} names_total={} worst_scope={}
```

Its operands are `scopes_with_ambiguity`, `scope_constructions`, `ambiguous_total`, `ambiguous_max`.
**Every one is a measure of AMBIGUITY — two or more definers offering themselves for one name.**

The failure that actually bit is not ambiguous. Once the kernel layer is bypassed there is exactly
**one** `List` in the pool, so the lookup returns a unique binding and the ambiguity counters report
zero. crisp-crab-430 measured precisely that — **0 PoolAmbiguous under the regen roots** — and it
was read across several lanes as reassuring.

**Unique-but-wrong and ambiguous are different states, and only the second is counted.**

    the counter answers   "did the pool offer a choice nobody authorized?"
    the question is       "did the pool answer AT ALL, where no declared edge exists?"

Every scoped-miss-then-pool-hit is an unauthorized edge **whether or not it was ambiguous**. The
loader does not count those, so the population is unmeasured and its observed size is zero by
construction.

**This is the empty-observation narrow in its most expensive form.** A zero was produced by an
instrument structurally incapable of producing anything else for this class, and was read as
evidence of absence. A false green is caught by whatever depends on it; **a false absence terminates
the investigation**, because there is nothing downstream to trip over.

## 4. The ask, which is cheaper than any repair currently contemplated

**Count the fallbacks, not the ambiguities.** The second arm of the match in §2 is one line;
instrumenting it yields the true size of the class in a single run — *how many names in the cut
corpus resolve only because the pool answered*.

The baseline exists for comparison: crisp-crab-430 measured **37389 scoped versus 733 pool-fallback**
on an import-**bearing** tree. The same measurement on the cut corpus is the population, and nobody
has it.

**Why this ranks above repairing the two blockers.** If the fallback count on the cut corpus is
large, then "no headers, all resolution name-derived" means "**most edges are unauthorized pool
hits**", and the two known blockers were simply the two that happened to be loud. That changes what
the cut *is*, not merely what it currently fails at — and it is decidable today, before the cut
lands, by one counter.

## 5. What is not claimed

- **No fallback count on the cut corpus is asserted here.** That is the measurement being asked for;
  quoting a number would be the fabrication this document exists to prevent.
- The 37389/733 baseline is from an import-bearing tree and is **not** a prediction for the cut
  corpus. It establishes that the counter is cheap and that fallback is already non-zero, nothing
  more.
- **This is not a finding that the cut is wrong.** It is a finding that a property everyone has been
  assuming — that a name-derived loader resolves by *derivation* — is not what the code does, and
  that the instrument which would have surfaced the difference measures a neighbouring property.
- The rung is **mitigatable**: the fallback is silent and uncounted, so nothing refuses and nothing
  ranks it. The next-rung trigger is the counter in §4; the rung after that is a typed refusal when
  a name resolves only through the pool, at which point an unauthorized edge becomes unwritable
  rather than merely observable.

## 6. Provenance

Three lanes, none of which had the whole picture, and none of which had the loader in front of them:

| | what they held |
|---|---|
| `keen-tern-667` (#9083) | the sharpest statement of the mechanism — pool coincidence, and that whole-tree compile-clean is not evidence a module can be emitted alone |
| `crisp-crab-430` | the live specimen (`04_infer` -> `FreeMonoid`) and the 0-PoolAmbiguous measurement that looked reassuring |
| `deep-ant-102` | read the loader and joined them |

The pairing is the finding. Each lane's own result was correct and individually unalarming; the
alarm exists only in the join. **Two of the three had independently concluded the matter was out of
scope for their change** — which is the documented way a known defect becomes nobody's.
