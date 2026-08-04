# `.dag` annotation prose — the representation defect

**Status:** audit open, measured at `64aed6007e`, 2026-08-04. **Nothing deleted, migrated, or reconciled.**
**This document deliberately stops at the audit.** It does not answer its own decisions and does not begin
a corpus pass; both wait on the source-annotation destination and the non-trivia parser model being
designed (operator instruction, 2026-08-04).

**Instrument:** a scratchpad Python extractor (three tiers, ≥200 B decoded string values, matching
[dag-note-prose-census.md](dag-note-prose-census.md) §6's grain). **Not committed** — see §8. Verified
against an independent count on `dag/gunbc/ci_layer_roots.dag`: 90 sites vs 91 (~1%).

---

## 0. The one-paragraph answer

**The disease is lost source-level intent. The prose fusion measured below is downstream of it.**
Banning `//` made comment *syntax* unwritable; it did not make commentary unwritable. It removed the only
structural signal that said *this text is commentary, not program data* — so the corpus kept writing
commentary and smuggled it into `data …: String` rows, where intent is mechanically indistinguishable
from program data. §1 shows that displacement as three merged PRs. The consequence is that **no
classifier, lens, census, or migration can decide what a given string was for**, which is why every pass
over this material has been an English-reading exercise. So the correct move is not a deletion pass and
not (only) a split: it is to **restore `//` as a modeled, non-trivia source annotation** — quarantining
prose in a carrier that is real data but not *program* data — and only then reconcile the existing
mountain against that destination.

---

## 1. The displacement chain — measured, not inferred

Three merged PRs, verified in this tree by commit and title:

| PR | commit | title |
|---|---|---|
| **#5579** | `f9cc238976` | Parser-wall: remove DAG comment trivia rules (fail-closed by construction) |
| **#6262** | `9e7c3c1d11` | Unbreak v1 self-resolution after #6242: **hoist .dag `//` comments to typed data rows**, dedup dag_collect registry |
| **#6424** | `c14e001a34` | **Sweep dead prose data-String rows (comments smuggled in strings): 215 rows across ~130 files** |

Read in order that is a closed causal loop: **comments banned → commentary smuggled into `data` rows →
intent becomes mechanically indistinguishable → recurring cleanup.** #6262 is not an accident; it is the
compensating convention adopted *deliberately* to keep the tree resolving after the ban. #6424 is the
first cleanup the convention made necessary, and it reported that the habit stayed live because no wall
covered `data` declarations.

The wall from #5579 is real and executing today — `v2.test.round_trip.dag_comment_wall_test` sends
synthetic modules through the production compile path and refuses both line and block comments, with an
accepted control and a `//`-inside-a-string-literal control. **The wall is not the defect.** The defect is
that it has no counterpart destination, so the pressure it creates has nowhere to go but `String`.

---

## 2. The destination: `//` as a modeled source annotation

The proposal is *not* to restore comments as lexer trivia. `v2.std.compilers.lexing` `TriviaRule` consumes
a lexeme without emitting a token, so a restored trivia rule would be readable to humans and invisible to
every parser, lens, census, migration, and SCM operation — the same semantic blindness with nicer syntax.
Nor should `//` lower to a binding (`let` or `data`): that puts prose in the namespace, makes it
referenceable, and re-classifies it as program data — the very move #6262 was forced into.

The intended reading, with provisional names and load-bearing laws:

```
type SourceAnnotationPurpose = SourceAnnotationModelingDebt | SourceAnnotationRationale

type SourceAnnotation {
  occurrence: OccurrenceId      // the annotation's own identity
  subject:    OccurrenceId      // the structural occurrence it annotates
  purpose:    SourceAnnotationPurpose
  text:       String
}
```

**Real data, but not program data.** The object program cannot bind, import, reference, call, evaluate, or
inspect an annotation. Compiler, lens, formatter, and SCM machinery *can* enumerate it through an
annotation-specific projection — that exception is necessary, since without it neither the census nor the
migration below could exist. **Quarantine, not erasure.**

**Sidecar, not `Node`.** v1's `Node` carries a `properties: List<Node>` field (`v1.00_core`) and attaching
there looks attractive. It should not be the first cut: `properties` participates in the semantic carrier,
so annotations there risk moving structural equality, content hashes, resolver and inference walks,
affected-set computation, emitted bytes, and cache keys. `v2.compiler.02_parse` already returns
`ParseArtifact { tree, span_index }`; an annotation graph belongs **beside** the tree and provenance, not
inside the semantic tree. v2 already mints occurrence identity (`v2.std.node` `NodeOccurrenceId` /
`MintedOccurrence`), which is the seam for naming annotation subjects without inventing file identity.

**There are no file comments.** A comment annotates a structural occurrence — a module, an import, a
declaration, eventually an expression or field — never "this file". The normalized row carries no path and
no line number; source spans stay diagnostic provenance in the existing `SpanIndex`, never semantic
identity.

**Two mandatory projections.** Semantically inert without being source-invisible:

- **semantic** — `semantic(parse(src with comment)) == semantic(parse(src without comment))`. Adding or
  removing a comment must not move declarations, references, resolution, inference, evaluation, the
  semantic IR hash, or target bytes.
- **authored-source** — `annotations(…with) != annotations(…without)`. Comments *do* affect source
  rendering, the annotation census, source-level SCM, and authored-source identity.

That pair buys an exact distinction later: a comment-only change alters authored source while leaving
semantic IR unchanged — much stronger than classifying a whole file as docs-like by path.

**First cut is deliberately narrow:** standalone leading `//` lines, attached to the next structural
occurrence in the same containment scope; comments before `module` attach to the module occurrence;
pending comments at scope end or EOF are a typed `UnattachedSourceAnnotation` refusal; **trailing
end-of-line and block comments stay fail-closed** until their attachment rules are separately modeled.
That defers the genuinely ambiguous case (`fn a() -> Int { 1 } // …` — function, return expression, or next
item?) instead of guessing at it.

**Default purpose is `SourceAnnotationModelingDebt`.** Plain `//` should say *this prose was quarantined
because it is not currently represented in the executable model*. A `why:` rationale escape hatch should
**not** ship in the first cut — offered early, every unmodeled invariant acquires a `why:` prefix and
declares itself permanent.

---

## 3. What the measurement still establishes (and what it does not)

The audit below measured **survivors**, and that bound is load-bearing:

> **#6424 had already swept 215 mechanically dead prose rows across ~130 files before this sample was
> drawn.** The surviving population is therefore predictably *enriched* for referenced, mixed, and
> load-bearing text. This sample says something real about survivors; it **cannot** retroactively refute
> the measured dead population, and it settles nothing about representation.

48 sites drawn **byte-weighted** without replacement (seed `20260804`), each read in full and scored on
one axis: *what would be lost by deleting this?*

| disposition | sites | bytes | share of sampled bytes |
|---|---|---|---|
| **KEEP** — irreducible rationale | 24 | 20,530 | **33.8%** |
| **MIGRATE** — live, but belongs in a typed carrier | 14 | 13,580 | **22.4%** |
| **MIXED mega-note** — keep core + deletable tail | 8 | 24,698 | **40.7%** |
| NOT PROSE — payload contamination | 2 | 1,861 | 3.1% |
| **DELETE** — worthless as authored | **0** | **0** | **0.0%** |

**Zero primary-delete in 48 draws** (rule of three: ≤6% of *sites* at 95%). The KEEP specimens are
load-bearing — why a predicate is deliberately concrete rather than generic and what breaks if you lift it
(`std.effects` `generic_predicate_frontier_note`); why `ABSENT` and `UNOBSERVED` are different answers
about a required executable (`gunbc.roadmap_dashboard_instance_apply` `executable_survey_note`); why a
stale-socket host differs from a never-provisioned one, at a measured cost of a week's silent uncaching
(`gunbc.build_cache_endpoint_path` `endpoint_path_state_authority_note`); why RFC 7519's registered claims
are all optional, including that the first cut got it backwards (`extdeps.auth.jwt`
`jwt_registered_claims_note`).

**The crucial reading: valuable and irreducible does not mean correctly represented.** Every one of those
notes is worth keeping *and* wrong as `data Foo: String`. Value and representation are independent axes,
and conflating them is what made "delete vs keep" feel like the whole question.

### 3b. The fusion measurement — still the sharpest available signal

| marker | sites | % sites | KiB | **% bytes** |
|---|---|---|---|---|
| ISO date (`2026-08-01`) | 709 | 19.7% | 810.6 | **29.9%** |
| dissolve-on / trigger | 506 | 14.1% | 560.9 | 20.7% |
| PR/issue ref (`#1234`) | 362 | 10.1% | 424.3 | 15.7% |
| review id (`review 45213`) | 324 | 9.0% | 389.0 | 14.4% |
| git SHA (≥7 hex) | 156 | 4.3% | 214.2 | 7.9% |
| session name (`calm-heron-729`) | 129 | 3.6% | 148.2 | 5.5% |
| CI run id (`run 30702499883`) | 77 | 2.1% | 123.7 | 4.6% |
| `LANDED`/`MERGED`/`SUPERSEDED` | 65 | 1.8% | 106.9 | 3.9% |
| **any of the above** | **1,535** | **42.7%** | **1,518.0** | **56.0%** |

**56% of annotation bytes carry a fact that goes stale without anyone touching it** — the same class DESIGN
§3 rules on for citations and the operator ruled on for #7710. Under §2's model these are exactly the facts
that must *not* become annotations: they belong in typed carriers, and `//` must not become their new home.

### 3c. Denominator and concentration

| population | sites | bytes | share |
|---|---|---|---|
| **ANNOTATION** | 3,592 | **2,708.4 KiB** | 74.8% |
| DOC_AUTHORITY — `design_document`, `roadmap_authority`, `plans/`, `site/` | 1,541 | 824.7 KiB | 22.8% |
| PAYLOAD — emitted source, golden fixtures, language templates | 117 | 89.4 KiB | 2.5% |

**`DOC_AUTHORITY` is not annotation and must never be swept.** `roadmap_authority.dag` holds 115.7 KiB of
`_note`-suffixed declarations that are *the authored body of ROADMAP.md*; `design_document.dag` builds
DESIGN.md from `p(text:)` / `h1(text:)` / `li(text:)` constructors. The `_note` suffix names two unrelated
things, and a name filter and a path filter each miss a different half. Size: **mega-notes (≥2,000 B) are
3.8% of sites but 15.5% of bytes**, and every MIXED specimen in the sample was one. Concentration: 50% of
annotation bytes in **108 of 1,119** files; 50% of the §3b marker mass in **63 of 617** files, with 124
mega-notes carrying a marker between them.

---

## 4. The target, restated

Not *"delete 90% of bytes."* The measurable target is **100% representation classification, zero anonymous
comment-smuggling.** The first partition is by *what the text is*, before any keep/migrate/delete judgment:

```
ProgramData | DocumentPayload | TypedOperationalFact | SourceAnnotationSmuggledAsString
```

Only within that partition does disposition become a fold rather than an English-reading exercise. The
policy that follows — *a comment may explain why a nearby construction deliberately has its current shape;
it must not restate what the declaration structurally says, must not carry when-facts, and is never
evidence that a machine claim holds* — is only enforceable once the text is structurally known to be
commentary.

---

## 5. Decisions this audit needs

Superseding an earlier D1–D4 that asked about deletion rates and ceilings — the wrong questions, because
they presumed the current representation:

- **D-A — carrier.** Confirm `SourceAnnotation` as a **sidecar on the parse artifact**, not `Node.properties`,
  not a binding, not restored trivia. Confirm the home after a concept DFS (candidate:
  `dag/std/source_annotation.dag`, language-agnostic, naming no `.dag` syntax).
- **D-B — attachment.** Confirm the narrow first cut: standalone-leading only; next structural occurrence
  in the same containment scope; pre-`module` attaches to the module; `UnattachedSourceAnnotation` refusal
  at scope end/EOF; trailing and block comments stay `FailClosed`.
- **D-C — erasure.** Confirm the two-projection law, and that the semantic projection must be proven
  byte-identical (emitted target bytes with and without the comment) rather than argued.
- **D-D — migration.** Confirm the §4 partition and that `SourceAnnotationModelingDebt` is the default
  purpose, with `SourceAnnotationRationale` withheld from the first cut.

**Not asked, deliberately:** the delete rate, the size ceilings, and the instrument's home. The first two
are downstream of representation; the third dissolves if the annotation census replaces the lexical one.

---

## 6. Sequence after sign-off (nothing here is started)

1. **Slice 1 — authority + pure laws.** Annotation occurrence, structural subject, purpose, graph,
   attachment refusals, erasure projection, same-scope attachment law. Names no `.dag` syntax.
2. **Slice 2 — `.dag` syntax realization.** A line-comment **`TokenRule`, never `TriviaRule`**, reusing the
   existing `LineCommentTextChar` lexical class; split comment fidelity out of
   `v2.extdeps.languages.dag` `DagTriviaNormalization` (a preserved comment is no longer trivia) so
   `dag_line_comment_fidelity` becomes `Modeled` while `dag_block_comment_fidelity` stays
   `FailClosed { feature: DagBlockCommentFailClosed }`. The v1 bridge needs a new `TokenShape` variant
   (`v1.00_core` — the `Sh*` convention has no comment variant today), tokenize-rather-than-skip in
   `01_tokenize`, pending-annotation attachment in `02_parse`, and a **regenerated** stage0 mirror.
3. **Slice 3 — discriminating execution.** Recut `v2.test.round_trip.dag_comment_wall_test` rather than
   invert it. Controls: annotation appears exactly once with the right subject; semantic IR equal with and
   without; census differs with and without; a `TriviaRule` treatment reds the visibility witness; a
   binding treatment reds the non-binding witness; `//` inside a string yields zero annotations; block and
   trailing still refuse; pending-at-EOF refuses; **emitted Rust bytes identical**; authored-source
   identity differs.
4. **Slice 4 — make recurrence unwritable.** Extend an existing liveness kernel (`v2.lens.inert_carrier`
   covers type carriers, not this `data`-declaration class) rather than minting a fresh prose lens. The
   decidable class: *a declaration with zero real program/document/typed-carrier consumers cannot survive
   merely to hold prose.* Deadness and consumer identity are **graph facts** — resolved declaration and
   reference facts, not `_note` names and not regexes.
5. **Slice 5 — reconcile the mountain**, per the §4 partition, splitting mixed rows by fact before
   disposition.

---

## 7. Proposed DESIGN.md paragraph (NOT landed here)

Authored by the operator, 2026-08-04; carried here for review. It would land through
`gunbc.design_document` with DESIGN.md regenerated — **deliberately not done in this PR**, because
DESIGN.md is the canonical authority and this document stops at the audit.

> Source annotations are data, but not program data. `.dag` line-comment syntax lowers to a typed source
> annotation attached to a graph-local structural occurrence; it is never discarded lexer trivia, never a
> namespace binding, and never a fact about a file. The object program cannot reference, evaluate, or emit
> annotation text. Authoring, formatting, SCM, and lens consumers may enumerate the annotation graph.
> Ordinary compilation derives from an annotation-erased semantic projection, while authored-source
> identity retains annotations. Plain comments are modeling debt by default. A comment may preserve
> irreducible human rationale about why a construction exists; any fact needed by a machine
> consumer—including invariants, receipts, events, rulings, citations, status, counts, and dissolution
> conditions—belongs in a typed carrier. An ordinary String declaration used solely to simulate commentary
> is misplaced or dead data.

---

## 8. Honesty bound

- **The sample measures survivors.** #6424 removed 215 dead rows first (§3); the 0/48 result is a statement
  about what survived that sweep, not about the original population, and **not** an argument that the
  current representation should stand.
- **Scoring was mine and unblinded.** I drew, read, and scored against a rubric I wrote. A second reader
  scoring the same 48 blind is the control this audit does not have.
- **48 sites** — §3's shares carry roughly ±14pp at 95%; ≤6% is a *site* bound, not a byte bound.
- **The extractor over-counts by ~3%** (payload leakage) and ~1% against an independent count.
- **§3b's markers are lexical.** Whether a note's core is irreducible is a judgment, stated as one.
- **§2's type names are provisional**; the laws beside them are the substance.
- **Nothing was deleted, split, migrated, or reconciled**, and no decision in §5 is answered here.

**Scaffold, with a dissolution trigger:** this document deletes when the source-annotation carrier lands
with its executed controls and the §4 partition is complete — at which point a lexical audit over prose is
superseded by a fold over the annotation graph and the typed populations beside it.
