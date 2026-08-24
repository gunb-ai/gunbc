# The name-derived loader resolves by falling through to a whole-tree pool, and the instrument watching it is blind to the failure that bit

**Evidence status — two classes, and an earlier revision of this line wrongly covered both with one
claim.** Every reference below names a module and a symbol. **An earlier revision carried file
offsets beside them, defending it under §3's carve-out for "a line inside a blob"; they are removed.**
Two reviews rejected that reading on sibling documents, and the removal was an improvement rather
than a concession — naming the enclosing function surfaced that the scoped census and the pool
fallback live in the SAME function, head versus loop, which is the structural fact the argument in
§4d actually rests on and which two line numbers had obscured.

**MECHANISM CLAIMS — verified by reading the live tree at `bd84f669681` (2026-08-24).** Every
statement about what the loader does, what `pool_bare_census` is built over, what
`[floor-bare-name-ambiguity]` reports, what forces the whole-corpus parse and where, and how
bare-eligibility is decided. These are re-checkable from the tree by any reader and nothing in this
document's argument rests on anything else.

**MEASUREMENT FIGURES — REPORTED BY OTHER LANES, NOT VERIFIED HERE, AND CARRYING NO DURABLE
RECEIPT.** Specifically `0 PoolAmbiguous`, `37389 scoped versus 733 pool-fallback` and its
`510/223` directional split (crisp-crab-430), and `bare_eligible=699` /
`tree_census_misses=2` / the 14.24s `pool_parse` term (witty-lark-109, whose PR gunbc#9090 carries
its own receipt). **No run id, artifact or repository receipt is cited for the first group**, and it
cannot be reproduced from the tree named above. Treat every one as a lane report, re-measure before
relying on it, and do not cite this document as their source.

**This distinction is load-bearing, not a disclaimer.** The argument in §2–§4 stands entirely on the
mechanism claims. The reported figures set *scale* and supply the anecdote in §3 about a zero being
read as reassuring; **if every one of them is wrong, the mechanism finding and the §4 ask are
unaffected** — indeed a wrong `0 PoolAmbiguous` would strengthen §3 rather than weaken it.

**Recorded because it is this document's own subject, committed in this document's own header.** The
line previously read *"Status of every claim: verified by reading the live tree"*. That was true of
the mechanism half and false of the measurement half, and it was caught in review (`review 55425`)
rather than by me. A blanket verification claim covering figures taken on trust from another lane is
exactly the failure §4b(1) names — reporting a stronger evidentiary standing than the executed
evidence establishes — and it is the same shape as every specimen §4d records: **the claim did not
move, the comparand did.** Here the comparand was *which assertions the word "verified" ranged over*.

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

`v1_compiler.cli_run` `bare_reference_pull_paths_for_source`, inside its resolve loop:

```rust
let target_module = match resolve_in(&census) {
    Some(m) => Some(m),
    None    => resolve_in(&pool_bare_census(index)?),
};
```

`census` is the **scoped** index. `pool_bare_census` (same module) is memoized over
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
zero. crisp-crab-430 reported precisely that — **0 PoolAmbiguous under the regen roots**, a lane report
carrying no durable receipt — and it was read across several lanes as reassuring. **The anecdote does
not depend on the figure being right:** what §3 establishes is that the counter *cannot* report
anything else for this class, so any zero it produces is uninformative whether or not this particular
one was measured correctly.

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

The baseline exists for comparison, **as a lane report with no durable receipt** (see the evidence
status above): crisp-crab-430 measured **37389 scoped versus 733 pool-fallback** on an
import-**bearing** tree. The same measurement on the cut corpus is the population, and nobody
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
calls `tree_bare_census_for_root` at its HEAD, while the fallback in §2 sits inside its resolve
loop further down — the same function, census first, lookups after. So a process
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

## 5b. Operator ruling: this is declared debt, and the cut's green result may not be quoted against it

**Operator ruling, 2026-08-24, relayed verbatim through deep-ant-102.** The namespace cut's landing
window may proceed without folding a kernel-name fallback guard into the prerequisite, *provided the
cohort receipt states this debt's standing accurately*:

```text
WholePoolFallbackDebt:
  measured
  not closed
  not required for this landing
  restoration trigger retained
```

> Do not let the successful `94/0` subject be quoted as evidence that the fallback mechanism itself
> is safe. The landing can proceed without folding that separate guard into the prerequisite, but it
> must remain **declared debt rather than disappearing behind the green product result.**

That is this document's subject given a §4b(3) standing, and the second sentence is the operative
one: **a green product result on the cut corpus is not evidence about the mechanism in §2.** It is
precisely the inference §3 shows the ambiguity counter cannot license, arriving by a different route
— `94 sources / 0 blocking / 0 imports` says the corpus compiles, not that its edges were authorized.

**One word in that ruling needs reconciling with §4, and this document must not paper over it.**
The ruling says `measured`. Per the evidence status above, what has been measured is the
**import-bearing baseline** (`37389 / 733`, a lane report with no durable receipt) — **not** the
fallback population on the cut corpus, which §4 asks for and which nobody had at the time of the
ruling. Read `measured` as *the class has an observed instance and a baseline*, never as *its size
under the cut is known*. Stating it the other way would let the debt read as bounded when its
governing quantity is exactly the open one.

**That gap is now being closed rather than left open:** deep-ant-102 has dispatched
`measure_whole_tree_resolve` over `dag` + `src/v1` + `src/v2` on current main, which emits the
`[cost-partition]` line carrying `edge_index_bare_candidates`, `edge_index_bare_name_universe`,
`edge_index_bare_resolve_loop`, `edge_index_bare_eligible` and `edge_index_source_files` together —
giving per-eligible-file cost for the three unmeasured slots and the exact multiplier from the two
counts, in place of §4d's ~5.5x approximation. **This document should be updated with those figures
when they land, and its `~5.5x` retired.**

**RESTORATION TRIGGER for the debt**, so it is retained here as the ruling requires rather than only
in a cohort receipt: the debt closes when a name that resolves *only* through the whole-tree pool is
refused rather than silently bound — the §4 counter being the intermediate rung that makes the
population visible first.

## 5c. A misattribution corrected in this document, and how it survived three checks

An earlier revision placed the fallback arm in **`build_both_closure_edge_index`**. It is in
**`bare_reference_pull_paths_for_source`**.

The claim was relayed from another session, and I verified it — but I verified **the snippet**, by
grepping for `pool_bare_census` and reading the two lines around the hit. **I never checked which
function enclosed the line I had found.** The code was exactly as reported; its home was not. Both
authors then repeated the wrong function name in messages to four other sessions.

It surfaced only because a review forced the offsets out, and naming the enclosing symbol requires
computing the enclosing symbol:

    awk 'NR<=7870 && /^fn |^pub fn /{last=NR": "$0} END{print last}'
    -> 7754: fn bare_reference_pull_paths_for_source(

**The positional citation was concealing the error, not merely risking rot.** `~7870` is a true
statement about where the code is, and it let a false statement about *what the code is part of*
travel beside it unchallenged for hours — because a line number is checkable against the file while
being silent about the containment tree, which is precisely the structure §3 says the namespace
authority already names.

And the correction is not cosmetic: the enclosing function is *why* the argument in §4d works. The
scoped census and the pool fallback are in **one** function — census at the head, fallback in the
loop below — which is what makes the whole-corpus parse unconditional. Attributed to two different
functions, that reasoning is unavailable.

## 5d. Three narrowings that arrived after this document was written, and one population it now has

**CORRECTED 2026-08-24, after this section was first written as "four lanes converged".** It is
**two** lanes, and the two that dropped out were removed by measurement rather than by argument —
see §5e, which also carries a negative result that constrains this document's central claim. Read
§5e before this section; what follows is still true, but it was written believing a unification that
did not survive.

Lanes converged on this mechanism within a few hours of the probe landing. What they establish
changes the document's scope in three directions, all of which make the finding **smaller and more
actionable** than it was written as.

### The fallback is CROSS-ROOT ONLY, which bounds the blast radius

The scoped census is keyed per **source root**, not per import list:

    fn tree_bare_census_for_root(index: &MultiEntryIndex, root: &str) -> ...   memoized per ROOT

and the loader's own field note states `pool_bare_census` is *"consulted only on an own-tree miss, so
cross-tree homonyms cannot steal a same-tree name."*

**So the fallback can supply a wrong definer only when the right definer lives in a different source
root.** Stripping import headers did not make every bare name pool-resolved; it made every
**cross-root** bare name pool-resolved. Within `dag`, or within `src/v2`, the scoped census still
answers and the fallback is never reached.

This matters to how the document is read: written without it, the finding sounds corpus-wide.

### The population is measured — 733, not unbounded

    510   src/v1 -> dag     a seed module pulling substrate
    223   dag -> src/v1     substrate borrowing the seed

**The wrong-definer render is bounded above by 733.** An unbounded-sounding defect and a
733-bounded one earn very different priority, and this document originally implied the first by
saying nothing.

**This bound applies to the file-closure path only** — see the next narrowing, which is the one this
document got wrong first.

### There is a SECOND fallback, in a different layer, and the bound does not reach it

`v1.compiler.infer_lookup` `func_sig_from_global_bare` and `global_bare_callable_node` take a
`TypeEnv` and a name — **no root parameter at all** — so they scope by what the environment holds
rather than by a root. That is a distinct fallback in the signature/type layer, and the 733 figure
says nothing about it.

**The error worth recording is that this document's author handed the 733 bound to another lane as a
bound on a symptom family**, without checking that the family's names go through the mechanism the
bound was measured on. They do not. The instrument was measuring a neighbouring subject and the
reader supplied the binding — the class this probe's sibling brief is entirely about, committed by
its author, twice in one afternoon.

### What that second fallback does NOT explain, decided by execution

Algebra method-template names are **gated out** of the signature fallback. All six names in the
observed no-definer family (`split first skip length trim join`) are among a 58-name algebra template
roster, and `v1.compiler.infer` states the law explicitly: *the census fallback never serves an
algebra method-template name — a whole-pool census entry for the unloaded `v2.std.algebra` must not
preempt the known-method bridge*, with the incident that produced the gate recorded beside it (a
census-served signature typed `filter(xs, f)` as a call into a never-loaded module, and the runtime
died `NoSuchFunction`).

**So the loudest symptom family attributed to this mechanism does not take either fallback.** Its
cause is a third thing, and this document does not name it. That is stated as an open question rather
than closed with a plausible answer, because a plausible answer is exactly what let the original
misattribution stand for a day.

### What the wrong definer actually is, which the original document did not have

`dag/std/types.dag` does not merely alias — it **declares**:

    type List<element> = FreeMonoid<element>

with `Map -> PartialFunction` and `Set -> PointwisePower` the same shape. So a module whose bare
`List` reaches `std.types` by fallback genuinely receives `FreeMonoid`, and every downstream failure
is a carrier failure: variants that do not exist, non-exhaustive matches over the wrong coproduct,
incompatible branches. **The pool does not merely fail to find a definer; it supplies a plausible
wrong one**, and that is the render this document identified as the dangerous half without being able
to name its source.

A confirmed prior instance exists from before the namespace cut: the same alias family produced
`PartialFunction` appearing in type position where the kernel `Map` was meant. **The two-render model
is therefore not a fresh hypothesis — it has a precedent.**

## 5e. A refuted specimen and a negative result that bounds this document's claim

Two of the four lanes §5d credited to this mechanism were removed the same day, both by measurement.
They are recorded here rather than edited away, because one of them constrains what this probe is
entitled to claim.

### The specimen this document credited was never this mechanism

This probe, and the message traffic around it, cited `src/v1/04_infer.dag` pulling `List` by
pool-fallback and binding the `FreeMonoid` alias as **the** live specimen — and attributed that
diagnosis to the lane working on the namespace cut. **That lane had already refuted it.** In their
words: the `List`/`FreeMonoid` cross-tree pull *"was a SEPARATE hypothesis that I refuted."* The
actual blocker they found was an empty list literal with no expected type being judged as
`List<Unit>` — a different defect that happens to live in the same file.

**Nobody asked them.** Two sessions inferred a lane's diagnosis from what its PR touched, and
published the inference as that lane's finding. **"This lane works on that file" was substituted for
"this lane found that mechanism"** — the same substitution that invalidated a hold's predicate the
same afternoon, where *a session cited a law living in F* became *a measurement is running against
F*. **Reading and measuring are different relations to a file**, and neither is the relation
"diagnosed this cause".

### The negative result, which is the part that bounds this document

Installing a kernel guard removed **3141 pulls** and **moved the closure by zero modules**.

That is a strong result and it was available in that lane the whole time. **It does not refute this
probe, and stating why is the point of recording it here**, because the temptation is to read it as
either a refutation or as irrelevant, and it is neither:

    closure MEMBERSHIP    which modules get loaded        <- unchanged by 3141 removed pulls
    name RESOLUTION       which definer a name binds to   <- this document's subject

Every module a fallback pull would have brought in was **already reachable by another path**, so
removing the pulls changed nothing about what loads. **The fallback's effect is on which definer a
bare name resolves to, not on what the closure contains.** This document's claim survives that
distinction intact — but it survives *narrowly*, and any argument for repairing the fallback on
**closure-cost** grounds is refuted outright by this measurement. The cost case must be made on
resolution correctness, or not at all.

**What this changes about §4d's cost forecast:** nothing measured there is withdrawn, but the
forecast should not be read as implying that scoping the fallback would shrink the loaded closure.
Measured, it would not.

### What survives

    the mechanism                CONFIRMED, and by READING THE LOADER rather than from any
                                 diagnostic population. std.types DECLARES
                                 `type List<element> = FreeMonoid<element>`, and a cross-root
                                 bare name reaching it genuinely receives FreeMonoid.
    lanes on it                  one: this probe. The stripped-header finding (gunbc#9083) is
                                 its own measurement and is unaffected, but it is a different
                                 mechanism, not a second lane on this one.
    #8282's blocker              NOT an instance. A refuted hypothesis about the same file.
    the symptom families         WITHDRAWN AS EVIDENCE — see §5f. The diagnostic population
                                 this document treated as corroboration was produced by a
                                 contaminated seed binary and does not survive a clean build.

**Three candidates eliminated and none confirmed is the honest state**, and it is worth more than the
unification this document briefly carried — which would have sent repair work at a cause two of its
three cited lanes had already ruled out.

## 5f. The corroborating diagnostics were a bootstrap artifact, and are withdrawn

**Everything in §5d and §5e that leans on a diagnostic population from gunbc#9075 is withdrawn as
evidence for this document's claim.** The reason is not that the diagnostics were misread. It is
that they had no source cause at all.

### The measurement

One dispatch settled it. `claim_executor` was built from the exact merge-base `bd84f6696`, the
*same* merged sources were checked out, and required-regen was re-run:

    seeded from the branch's own earlier binary   71 hard diagnostics
    seeded from a merge-base-built binary          ZERO hard diagnostics

**Same sources. Different compiler. The refusals vanish.** They were emitted by a compiler built
from an earlier state of that branch, which regenerated a mirror carrying the defect forward.

### Why this document was fooled, and the error is instructive rather than embarrassing

This document's author argued the attribution was *settled*: main was green at the branch's exact
merge-base, the branch was that merge-base plus one delta, therefore the delta caused the
diagnostics. **The syllogism is valid and the premise was incomplete.** Two trees were compared
while a second variable moved silently:

    main's CI      seeds its compiler from main
    the branch     seeded its compiler from ITSELF

**In a self-hosted compiler the binary is not a constant across a source comparison.** Treating the
source as the only variable is exactly the assumption that cannot hold in a repository whose
compiler is its own output — and it was made in a document about instruments answering narrower
questions than they are asked.

### What this does NOT touch

**The mechanism this probe describes was established by reading `bare_reference_pull_paths_for_source`
and `dag/std/types.dag`, not by observing any failure.** The scoped census, the whole-tree fallback,
the per-root scoping, the 733 population, and the `List = FreeMonoid` declaration are all
source-level facts, independently verifiable, and unaffected by any binary. **They stand.**

What is gone is the claim that a particular observed diagnostic population *demonstrated* the
mechanism biting in production. **This probe describes a real hazard with no confirmed live victim**,
which is a weaker and more honest claim than the one §5d and §5e were drifting toward.

### The generalisable rule, and its scope — which matters as much as the rule

**In a self-hosted repository, "I changed the compiler and the compiler broke" has two readings, and
nothing in the source distinguishes them.** The discriminator is one dispatch — rebuild the tool from
a known-good commit, keep the sources fixed, re-run.

**But the rule has a boundary, and stating it without one produces the next over-correction.** The
discriminator is **whether the compiler was held fixed across the arms**, not whether the claim
involves diagnostics:

    CROSS-BINARY        tree A measured with binary A, tree B with binary B.
                        The compiler moved silently alongside the variable under test.
                        DEAD. This is the failure above.

    PAIRED WITHIN-BINARY  both arms measured with the SAME binary, one variable changed.
                        Contamination shifts both arms equally, so the DELTA survives even
                        when the absolute numbers are wrong. NOT killed.

A stripped-header module refusing standalone and passing with its header restored, **under one
binary**, is the second shape: for contamination to explain it, the contaminated compiler would have
to refuse one arm and accept the other on exactly the axis under test — which is the claimed
mechanism, not an artifact of it.

**The second-order caveat, which is the part most likely to be skipped: a paired comparison is
immune to a compiler that is WRONG, not to one that is IRRELEVANT.** If the binary measuring both
arms was itself built from a contaminated mirror, the delta is real *for that compiler* and says
nothing about the one the repository ships. So a surviving paired result should be stated as
*"header state changes closure behaviour **in this compiler**"* rather than *"in gunbc"* — and the
same one-dispatch fix upgrades it.

**Four lanes spent an afternoon authoring mechanism theories for output that had no source cause**,
and every one of those theories was internally coherent. Being coherent was never evidence.

### This document audited against that boundary

    SURVIVES — established by READING source, no compiler in the chain:
      `type List<element> = FreeMonoid<element>` in dag/std/types.dag
      the 14-row algebra map carrying "FreeMonoid": "FreeMonoid", so the profile HITS
      tree_bare_census_for_root is keyed per ROOT; func_sig_from_global_bare takes no root
      the scoped-census-then-whole-pool fall-through in bare_reference_pull_paths_for_source
      algebra method templates are gated out of the census fallback by explicit law

    SUSPECT — compiler-derived, provenance not established, CONFIDENCE WITHDRAWN:
      the 733 pool-fallback population and its 510/223 directional split (§5d).
      Nobody has established which binary produced it, or whether one binary produced all
      of it. **It should not bound anything until re-derived under a known-good build.**
      This is the second narrowing that figure has needed in one day; the first was scoping
      it to the file-closure path only.
      Likewise the per-eligible-file costs and the 3851/699 multiplier quoted in §5e are
      single-run absolutes, not a paired delta, and inherit the same question.

**Stating this as a per-claim audit rather than a blanket retraction is the point.** A blanket
retraction would discard the read-derived facts, which are the load-bearing ones and which no binary
can affect.

## 6. Provenance

Three lanes, none of which had the whole picture, and none of which had the loader in front of them:

| | what they held |
|---|---|
| `keen-tern-667` (#9083) | the sharpest statement of the mechanism — pool coincidence, and that whole-tree compile-clean is not evidence a module can be emitted alone |
| `crisp-crab-430` | the `510/223` directional split, the `37389/733` baseline, the 0-PoolAmbiguous measurement that looked reassuring — and the REFUTATION of the `04_infer` -> `FreeMonoid` specimen this document originally credited to them (§5e) |
| `deep-ant-102` | read the loader and joined them |

The pairing is the finding. Each lane's own result was correct and individually unalarming; the
alarm exists only in the join. **Two of the three had independently concluded the matter was out of
scope for their change** — which is the documented way a known defect becomes nobody's.
