# Abstraction rulings — what data we need

Companion to `gunbc.abstraction_ruling_corpus`. This answers one question: **what must be collected before any mechanism — deterministic detector or model — can be honestly scored?**

Written to be acted on. Each bucket names what a row is, where it comes from, roughly what it costs, and who is best placed to produce it.

## 0. What one labeled example actually is

Not `(pair of subjects) → verdict`. That teaches a classifier to pattern-match names.

A row is **five** fields, and the load-bearing one is the third:

| field | why it exists |
|---|---|
| `subjects` | the two (or more) declarations under judgment |
| `ruling` | one of 10 closed answers |
| **`discriminator`** | **the single fact that decided it** |
| `standing` | decidable from participation, or needs semantic judgment |
| `provenance` | observed-historical, observed-live-control, or manufactured |

A corpus of subjects-and-verdicts states *what we concluded*. A corpus of **discriminators** states *what evidence any mechanism must be able to see* — which is the thing we are actually trying to learn, and the thing that survives the corpus being re-authored.

`standing` is what makes "the model only re-derived what a structural reader already decides" a **measurable failure** rather than an impression. It cannot be recovered after the fact, so it is recorded per row at authoring time.

## 1. Current population — 5 rows

All in the unified-memory specimen. Coverage against the answer space:

- **rulings covered: 3 of 10** — `ProjectionMissing` (×2), `KeepDistinct`, `ExistingAuthority`
- **rulings with zero examples: 7** — `LikelyDuplicate`, `CandidateNewAbstraction`, `ReprimeCandidate`, `OverCollapseRisk`, `DemandAbsent`, `AuthoredButUnconsumed`, `UnknownBecauseEvidenceIncomplete`
- **standings covered: 2 of 3** — `UnknownBecauseParticipationIncomplete` has none
- **provenance: 2 manufactured, 1 live control, 1 historical, 1 recorded gap**

This is a specimen, not an instrument. It is enough to prove the *shape* is right and nowhere near enough to score anything.

## 2. The four buckets, cheapest first

### Bucket A — manufactured from principles *(cheap, high volume, you said this should be easy — agreed)*

Synthetic pairs constructed **to exhibit a chosen discriminator**, with the verdict known by construction.

This is cheap precisely because the principles are already written down. Worked example of the generator, from the affine-unit case already in tree:

> Two subjects share a name and a carrier type. One admits addition with itself; the other does not (position + position is meaningless, position + magnitude is not). → `KeepDistinct`, discriminator: *"the verb `add` is total on one and undefined on the other"*, standing: `DecidableFromParticipation`.

Vary one axis at a time to generate a family: same verbs/different governance, same governance/different verbs, identical extension but distinct consumers, etc.

**Why this is not sufficient alone, and the trap to avoid:** manufactured rows are generated *from* the criterion, so a mechanism scored only on them measures whether it implements the criterion we wrote — not whether the criterion is right. They are necessary for coverage of rare rulings and useless as the sole evidence.

**Volume target: ~15–20 per ruling variant**, so all 10 have a floor. Biggest need is the 7 uncovered rulings.

**Best producer: me**, mechanically, with your review of the *discriminators* rather than the verdicts.

### Bucket B — observed historical *(expensive, highest value)*

Real rulings this repo already made, harvested from git history. **These are the only rows where the verdict was decided by someone with the full context, before any criterion existed to bias it** — which makes them the honest test set.

**Measured, not estimated:** 1,639 of 4,443 commits since 2026-05-01 match de-forking keywords (`dissolve`, `nickname`, `single authority`, `duplicate`, `consolidat`, `collaps`, `forked`). But sampling the titles shows **signal density is low** — "Rich PDF citation anchors" matches and is not a ruling; "Retire the wall nickname" matches and is. So this is a **candidate pool requiring triage**, not 1,639 labels. My honest read is 10–30% real, i.e. roughly 160–500 genuine rulings.

The cost is not finding them. It is that **extracting the discriminator requires reading the change and understanding why it was made** — the commit message usually states *what* was consolidated, rarely the single fact that decided it.

**This is where your help is worth the most.** Not constructing rows — triaging which commits are real rulings, and supplying the "why" where you remember it.

**Volume target: 60–100 rows**, weighted toward rulings Bucket A cannot honestly manufacture.

### Bucket C — observed live controls *(medium cost, self-maintaining)*

Rows whose discriminator is established by an **executing witness against the live tree**, like the shared-pool encoding control already in the corpus: it proved by execution that one physical pool has two valid encodings and that the duplicated encoding double-counts.

These are the most durable rows — they re-verify themselves and red when the tree moves underneath them. They are also the only rows that can establish `UnknownBecauseParticipationIncomplete` honestly, since "the participation evidence is incomplete" is a claim about the live corpus.

**Volume target: 10–15.** Low count is correct; each one is a real witness.

### Bucket D — recorded gaps *(free, and do not skip)*

Rows that **could not be written**, recorded as findings. The corpus already carries one: the hardware-pool vs runtime-managed-allocation keep-distinct cannot be stated because no CUDA/SYCL authority exists in the tree, so its right-hand subject does not resolve.

Authoring a row with a fabricated symbol to make the population look complete is exactly the failure the instrument exists to detect. **Every gap encountered gets recorded rather than filled.**

## 3. The one rule that governs all of it

**No mechanism may be scored on rows it was fitted on.** Concretely:

- Bucket B (historical) is held as **test only** — never used to tune a detector or prompt.
- Buckets A and C may be used for development.
- Any row whose discriminator was *written by* a model is marked and excluded from scoring.

Without this the corpus silently becomes a mirror, and every measurement taken against it reads high.

## 4. Sequencing

1. **A** to floor all 10 rulings — I can proceed now, no input needed.
2. **B triage** — the ask: which commits are real rulings, and the "why" where you have it. Highest value, gated on you.
3. **C** opportunistically, as live controls come up in other lanes.
4. **D** continuously, whenever a row cannot be written.

## 5. Open question for you

The `standing` split assumes *decidable from participation* and *requires semantic judgment* are cleanly separable. The unified-memory specimen suggests they may not be: the catalog-capacity vs allocatable-memory `KeepDistinct` is labeled `RequiresSemanticJudgment`, but a mechanism that could see *"one changes with what is resident, the other never changes"* would decide it structurally.

If that boundary moves under examination, the standing field is measuring the current detector's reach rather than a property of the case — which is worth knowing early, since the whole point of the field is to prevent exactly that kind of self-flattery.
