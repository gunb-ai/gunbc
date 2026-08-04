# `.dag` annotation prose — policy audit

**Status:** audit open, measured at `64aed6007e`, 2026-08-04. **Nothing deleted, nothing migrated.**
This document exists to settle one question before any deletion pass starts: *how much of the
annotation prose is actually low-value?*

**The short answer: far less than the brief assumed, and the deletable mass is not where anyone
is looking for it.** The brief's estimate — "90% of the low value stuff we can bankrupt/delete" —
is not supported by a byte-weighted hand-read. But the brief's *instinct* that something is wrong
is supported, and §5 names what it actually is.

**Instrument:** a scratchpad Python extractor (three tiers, ≥200 B decoded string values, matching
[dag-note-prose-census.md](dag-note-prose-census.md) §6's grain). **Not committed** — see §8. Verified
against an independent count on `dag/gunbc/ci_layer_roots.dag`: 90 sites vs 91 (~1%).

**Relation to prior censuses.** [dag-note-prose-census.md](dag-note-prose-census.md) measured *semantic
class* (what kind of statement is this); censuses A/B/C measured *dissolution markers* in three
subtrees. This audit measures a different axis — **value**: would deleting this cost anyone anything?
The axes are orthogonal and this one is the one the brief asks about.

---

## 0. The one-paragraph answer

Of a byte-weighted random sample of 48 annotation sites read by hand, **zero** were primarily
deletable-as-worthless. Half were irreducible rationale — a non-obvious *why*, a measured finding, or
a counterfactual that would be re-litigated if it vanished. The real defect is not worthless prose;
it is **fusion**: 56% of annotation bytes carry a time-bound marker (a date, PR number, review id,
SHA, session name, CI run id, or `LANDED` status word) welded into a note whose core is irreducible.
That residue is what makes a reader open a note and conclude most of it is not why-this-does-this —
which is exactly the brief's experience, correctly observed and misattributed. So the move is **not a
deletion pass**. It is a **split**: the why stays, the time-bound half becomes a typed row, and a
policy stops the fusion recurring at the ~2 KiB/PR the corpus is currently appending.

---

## 1. Denominator — what counts as annotation prose

A naive scan of every ≥200 B string in the corpus returns 3,622 KiB, and that number is wrong for this
question. Two populations must come out first:

| population | sites | bytes | share |
|---|---|---|---|
| **ANNOTATION** (the subject of this audit) | 3,592 | **2,708.4 KiB** | 74.8% |
| DOC_AUTHORITY — `design_document`, `roadmap_authority`, `plans/`, `site/` | 1,541 | 824.7 KiB | 22.8% |
| PAYLOAD — emitted source, golden fixtures, language templates | 117 | 89.4 KiB | 2.5% |
| **total ≥200 B strings** | 5,250 | 3,622.5 KiB | 100% |

**`DOC_AUTHORITY` is not annotation and must never be swept.** `roadmap_authority.dag` holds 115.7 KiB
of `_note`-suffixed declarations that are the *authored body of ROADMAP.md* — a generated artifact's
content, not a comment on a declaration. The `_note` suffix names two unrelated things in this corpus,
which is a §3 nicknaming instance in its own right and the first trap any automated pass would fall into.

**Payload leakage is real but small.** Two of the 48 sampled sites (3.1% of sampled bytes) were golden
bash fixtures inside `bash_program_fold_test.dag`, not prose — my filter missed them. So the annotation
denominator above is overstated by roughly 3%, and every share below inherits that error.

**Concentration:** 50% of annotation bytes sit in **108 of 1,119** files.

---

## 2. Reconciling "90% deletable" with the prior census's "<1%"

These two figures never disagreed — they measure different things, and neither answers the brief:

- The prior census's **0.6% "crisp deletable"** counts only prose that is *lexically decidable as dead*:
  a fired dissolution trigger, or a dated snapshot superseded by a later sibling in the same file. It is
  a floor on machine-detectable death, not an estimate of value.
- The brief's **90%** reads on *the experience of opening a note* — most of what you read is not the
  rationale you came for.

Both are consistent with what the hand-read found: the prose is **valuable and adulterated**. Almost none
of it is dead; most of it is mixed.

---

## 3. The hand-read — method and result

48 annotation sites drawn **byte-weighted** without replacement (seed `20260804`, so the draw is
reproducible), each read in full and scored on one axis: *what would be lost by deleting this?*

| disposition | sites | bytes | share of sampled bytes |
|---|---|---|---|
| **KEEP** — irreducible rationale | 24 | 20,530 | **33.8%** |
| **MIGRATE** — live, but belongs in a typed carrier | 14 | 13,580 | **22.4%** |
| **MIXED mega-note** — keep core + deletable tail | 8 | 24,698 | **40.7%** |
| NOT PROSE — payload contamination | 2 | 1,861 | 3.1% |
| **DELETE** — worthless as authored | **0** | **0** | **0.0%** |

**Zero primary-delete in 48 draws.** By the rule of three that bounds the true site-rate at roughly
**≤6%** with 95% confidence — not zero, but nowhere near 90%. Reading the specimens is more convincing
than the number: the KEEP set includes why a predicate is deliberately concrete rather than generic and
what breaks if you lift it (`std/effects.dag`); why `ABSENT` and `UNOBSERVED` are different answers about
a required executable, and the operator-facing bug that conflating them produced
(`roadmap_dashboard_instance_apply.dag`); why a stale-socket host and a never-provisioned host are
different states, with a week of silent uncaching as the measured cost (`build_cache_endpoint_path.dag`);
and why RFC 7519's registered claims are all optional, including the note that the first cut got it
backwards by generalizing from one observed token (`extdeps/auth/jwt.dag`).

**None of that is recoverable from the code.** Deleting it does not remove a comment; it removes the
reason a future author will not repeat the mistake — and in at least three of the sampled cases the note
explicitly records that the mistake *was already made once*.

---

## 4. What is mechanically decidable — the time-bound marker measurement

Unlike value, this is decidable, corpus-wide, today:

| marker | sites | % sites | KiB | **% bytes** |
|---|---|---|---|---|
| ISO date (`2026-08-01`) | 709 | 19.7% | 810.6 | **29.9%** |
| dissolve-on / trigger | 506 | 14.1% | 560.9 | 20.7% |
| PR/issue ref (`#1234`) | 362 | 10.1% | 424.3 | 15.7% |
| review id (`review 45213`) | 324 | 9.0% | 389.0 | 14.4% |
| git SHA (≥7 hex) | 156 | 4.3% | 214.2 | 7.9% |
| session name (`calm-heron-729`) | 129 | 3.6% | 148.2 | 5.5% |
| CI run id (`run 30702499883`) | 77 | 2.1% | 123.7 | 4.6% |
| `LANDED`/`MERGED`/`SUPERSEDED` status | 65 | 1.8% | 106.9 | 3.9% |
| **any of the above** | **1,535** | **42.7%** | **1,518.0** | **56.0%** |

**56% of annotation bytes carry at least one fact that goes stale without anyone touching it.** This is
the same class DESIGN §3 rules on for citations (*cite the symbol, not the position*) and the same class
the operator ruled on for #7710 (*a design criterion embedding a SHA or a pass count has to be edited
every time the world moves*). It is already repository law for two narrow cases; the measurement says the
general case is more than half the prose mass.

---

## 5. The actual disease: fusion, concentrated in mega-notes

| band | sites | KiB | % bytes |
|---|---|---|---|
| 200–500 B | 1,400 | 465.9 | 17.2% |
| 500–1,000 B | 1,401 | 971.3 | 35.9% |
| 1,000–2,000 B | 653 | 852.7 | 31.5% |
| 2,000–4,000 B | 119 | 303.1 | 11.2% |
| ≥4,000 B | 19 | 115.5 | 4.3% |

**Mega-notes (≥2,000 B) are 3.8% of sites but 15.5% of bytes** — and *every* MIXED case in the sample was
one. The largest sampled note is 7,265 B (`src/v1/04_infer.dag` `declared_type_conformance_note`); it
contains four genuinely load-bearing measured false-positive classes **and** a running account of six
review round-trips by id. The first must survive; the second is a receipt.

That is the shape of the whole problem: **a mega-note is an unmodeled changelog with a good essay inside
it.** The brief's "90% is low value" is what that feels like from the reading end. The measurement says
the ratio is nearer 1:1 by bytes, and — decisively — that the two halves are *separable*, because the
deletable half is exactly the mechanically-detectable half in §4.

---

## 6. Proposed policy (needs operator sign-off — §7 D1)

Four rules, each decidable by a reader without judgment calls, and each traceable to an existing DESIGN
principle rather than newly invented:

> **P1 — A note says *why*, never *what*.** If a sentence restates what the declaration, type, field, or
> variant names already say, it is a second representation of one fact — delete it (§2/§3). The test:
> *could a reader derive this sentence from the code beside it?* If yes, it is not a note.
>
> **P2 — A note carries no time-bound fact.** No dates, PR/issue numbers, review ids, session names, CI
> run ids, SHAs, or `LANDED`/`MERGED` status words. Those are receipts; they belong in a typed row that a
> consumer reads, cited by symbol (§3 cite-the-symbol; the #7710 ruling generalized). A note states what
> is *true of the declaration*, in the present tense, with no clock in it.
>
> **P3 — One note, one job, with a ceiling.** A note past ~1,200 B is a mixture and must be split: the
> irreducible why stays; a spec becomes a type or a lens; a trigger becomes a typed dissolve-on row; a
> measurement becomes a receipt row. **≥2,000 B is a hard refusal** — 15.5% of bytes sit above that line
> and every mixed specimen in the sample was there.
>
> **P4 — A note is attached to what it explains.** It lives beside the declaration it is about and names
> it. Lane state, migration sequencing, and cross-session coordination belong in the lane's plan or
> roadmap authority — not in a substrate module.

**The single acceptance test, for authors and reviewers:** *would deleting this note cause someone to
re-litigate a settled decision or reintroduce a defect?* If yes it stays and P2/P3 apply to its form. If
no, it does not land. That test is what the KEEP set in §3 passes and what the time-bound residue in §4
fails.

**What P1–P4 do NOT license.** They do not authorize a deletion pass over the corpus. Under this policy
the sampled prose is overwhelmingly *reformatted and split*, not removed — §3 measured the removable
fraction at ≤6% of sites. Anyone reading this policy as permission to bankrupt notes wholesale has
inverted its finding.

---

## 6b. Where the reconciliation starts

The §4 population is as concentrated as the prose itself: **1,400 KiB across 1,370 sites in 617 files,
with 50% of the marker mass in 63 of them.** A split pass can cover half the problem in ~63 files, and
**124 mega-notes carry a marker between them (381 KiB)** — that intersection is the highest-density
target in the corpus and the natural first slice.

| # | KiB | sites | file |
|---|---|---|---|
| 1 | 57.9 | 21 | `src/v1/04_infer.dag` |
| 2 | 42.2 | 44 | `dag/gunbc/ci_layer_roots.dag` |
| 3 | 42.0 | 21 | `dag/gunbc/ci_spec.dag` |
| 4 | 32.5 | 15 | `dag/gunbc/commit_workflow.dag` |
| 5 | 24.6 | 11 | `dag/gunbc/ci_workflow.dag` |
| 6 | 23.3 | 13 | `dag/gunbc/ci_materialization.dag` |
| 7 | 21.5 | 13 | `dag/extdeps/pin.dag` |
| 8 | 19.9 | 17 | `src/v2/workflow/ci_floor_plan.dag` |

`ci_layer_roots.dag` is already censused at row grain by
[dissolution-census-a](dissolution-census-a-ci-layer-roots.md), so it is the one file where the split
pass can start from typed rows rather than a fresh read — which makes it the cheapest proof of the
policy, not the biggest win. **Note that `src/v1/04_infer.dag` and `dag/gunbc/ci_spec.dag` are
load-bearing per DESIGN §7 and the Building-&-checks section**; they head the list by mass but must not
lead the pass.

---

## 7. Decisions this audit needs

- **D1 — is the 90%-delete premise withdrawn?** The evidence says the deletable-as-worthless fraction is
  ≤6% of sites and ~0% of sampled bytes. Recommend: yes, replace "bankrupt 90%" with "split the 56%".
- **D2 — is P2 enforced, and by what?** A lens over the `Node` tree can decide every marker in §4
  mechanically (they are all lexical, on String values the tree already carries). That is the cheapest
  available wall and it is construction-adjacent, not judgment. Needs a home and a receipt carrier for
  the evicted facts before it can refuse.
- **D3 — the P3 ceiling number.** 1,200 B split / 2,000 B refuse are drawn from §5's distribution, not
  from first principles. If the ceiling should be lower, the population above it grows accordingly.
- **D4 — instrument home.** This audit's extractor is uncommitted, so §1's and §4's counts are not
  independently re-derivable — the same specification-without-execution weakness
  [dag-note-prose-census.md](dag-note-prose-census.md) §6 flagged and left open. Either widen the modeled
  Python allowlist (a third variant beside the two already there) or write the `.dag` lens D2 needs
  anyway. Recommend the latter: it discharges D2 and D4 together and deletes this document's instrument.

---

## 8. Honesty bound

- **The sample is 48 sites.** Shares in §3 carry roughly ±14pp at 95%; the ≤6% delete-rate bound is the
  rule of three on 0/48 and is a *site* bound, not a byte bound.
- **Scoring was mine and unblinded.** I drew the sample, read it, and scored it against a rubric I wrote.
  A second reader scoring the same 48 blind is the control this audit does not have.
- **The extractor over-counts by ~3%** (payload leakage, §1) and by ~1% against an independent count.
- **§4's markers are lexical.** `review 45213` is decidable; whether a note's *core* is irreducible is
  not, and no rule in §6 pretends otherwise — P1's test is a judgment, stated as one.
- **Nothing here was deleted, split, or migrated.** This is evidence for a decision, not a change.

**Scaffold, with a dissolution trigger:** this document deletes when P1–P4 land as an enforced policy
(D2's lens plus a receipt carrier for the evicted time-bound facts) and the reconciliation pass over the
§4 population completes — at which point a hand-read audit over prose is superseded by a fold over typed
rows, exactly as [dag-note-prose-census.md](dag-note-prose-census.md) anticipates for its own.
