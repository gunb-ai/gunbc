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

## 4b. A scoped hit does not mean an authorized edge

**The most likely misreading of this document is that the scoped arm is the good arm.** It is not,
and the distinction decides how a fallback count should be read once someone has it.

`census` is built **per source root**. So a scoped-unique hit means *this name is unique within its
own tree* — it does **not** mean anyone declared an edge to it. After the cut, essentially all
resolution is name-derived, and a scoped/fallback split is therefore **same-tree coincidence versus
cross-tree coincidence: both coincidence, differing only in radius.**

Concretely: crisp-crab-430's baseline of 37389 scoped versus 733 pool-fallback is a 97.9 / 2.1 split,
and **97.9% is not a proportion of edges that were authorized.** Left implicit, that number reads as
reassurance and would be cited as such. The count being asked for in §4 sizes the *cross-tree* half;
sizing the authorized half is a different measurement that nobody has proposed, because on the
current mechanism the answer may simply be zero.

## 4c. The same census produces a second finding with a different owner

crisp-crab-430 has split the 733 by direction:

    510   src/v1 -> dag
    223   dag    -> src/v1

**The second half is not a resolution defect at all.** The substrate has **zero** `to_string`
declarations under `dag/` or `src/v2`; the only one in the tree is a hand-written decimal loop in the
v1 seed's emit-support module; all 721 uses are call-position; and **194 substrate files depend on
it.** That is a *missing authority* in the substrate, not a bad resolution — the pool is answering
correctly for a definer that has no substrate home.

No guard fixes it, and it would survive every repair proposed in §4. It is recorded here because one
census produced two findings whose owners differ, and separating them is what stops the smaller one
being swept into the larger.

## 4d. The cost forecast, and the two wrong versions this document carried first

**The forecast is real, and it took three rounds and three authors to land it on the right term.**
Recorded with its history because the correction pattern is itself the subject of this document.

    round 1 (smart-ram-730)  "pool_parse's frequency changes by a large factor"   WRONG TERM
    round 2 (deep-ant-102)   "pool_bare_census memoizes, so it is a step
                              function to certainty, not a multiplication"        RIGHT MEMO,
                                                                                  WRONG SUBJECT
    round 3 (witty-lark-109) the multiplier is on bare_eligible                   CORRECT

**Why round 2's "today pays zero" is backwards.** The *scoped* census forces the parse, and it does so
**unconditionally, above the loop that contains the fallback** — `bare_reference_pull_paths_for_source`
(~7754) calls `tree_bare_census_for_root` at ~7764, while the fallback in §2 is at ~7870. So a process
whose names all resolve in the scoped census pays the whole-corpus parse in full; building that census
is what requires it. The receipt is an ordinary run with `bare_eligible=699` and
`tree_census_misses=2` — scoped hits throughout, parse paid anyway.

The memo observation is correct but lands on a **different term**: `pool_bare_census`'s own whole-pool
symbol index, built over every pool module (3875, against 2818 and 1893 for the two root censuses).
That is the largest index the loader can construct and the only one no successful resolve pays for.
Memoized, so it is 0-or-1 per index — bounded, and magnitude unmeasured.

**The real multiplier is on a third term nobody had named**, and it needs no post-cut estimate because
it is the negation of the syntax being deleted:

```rust
fn source_declares_import_lines(content: &str) -> bool {
    content.lines().any(|l| l.trim_start().starts_with("import "))
}
```

Bare-eligibility is that predicate's negation — a file is scanned by the per-file bare half exactly
when it declares **no** imports. **The cut deletes those lines corpus-wide, so after it every file is
eligible** and `bare_eligible` goes from a measured 699 toward the whole corpus, roughly 5.5x on the
population the per-file bare half is denominated in. It lands on `edge_index_bare_candidates`,
`edge_index_bare_name_universe` and `edge_index_bare_resolve_loop` — **whose per-file cost is
unmeasured**, and that is the open gap.

**The pattern, stated because it caught three people in one thread.** Round 1 had the right *shape* on
the wrong *term*. Round 2 fixed the term to a different wrong one and, in doing so, **deleted the
shape**. The shape was real all along, on a term neither round had looked at. Nobody measured anything
incorrectly at any point; each round the comparand moved and the claim did not. **The repair for a
claim attached to the wrong subject is to MOVE it, not to soften it** — round 2 escaped a wrong term by
discarding a true forecast, which cost more than the original error.

## 5. What is not claimed

- **No fallback count on the cut corpus is asserted here.** That is the measurement being asked for;
  quoting a number would be the fabrication this document exists to prevent.
- The 37389/733 baseline is from an import-bearing tree and is **not** a prediction for the cut
  corpus. It establishes that the counter is cheap and that fallback is already non-zero, nothing
  more.
- **This is not a finding that the cut is wrong.** It is a finding that a property everyone has been
  assuming — that a name-derived loader resolves by *derivation* — is not what the code does, and
  that the instrument which would have surfaced the difference measures a neighbouring property.
- **The cost forecast is NOT carried by `pool_parse`, and two earlier drafts of this bullet said it
  was.** See §4d, which states the corrected three-part shape. Both wrong versions are recorded there
  rather than deleted, because the way the claim moved is the finding.
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
