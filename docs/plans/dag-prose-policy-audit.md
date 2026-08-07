# `.dag` annotation prose — the representation defect

**Status: audit complete; source-annotation policy ruled; M1C-1 implemented in #7955 on jwt.dag and the annotation carrier modules; corpus reconciliation ongoing.**
Measured at `64aed6007e`, 2026-08-04. **M1C-1 (#7955) implements the two deferred jwt.dag prose migrations, self-applies the feature to `dag/std/source_annotation.dag`, `src/v1/annotation_bind.dag`, and `src/v1/tests/claim/v1_annotation_round_trip_test.dag`, and leaves oidc.dag's four legacy rows for a follow-on.** The canonical policy
landed in DESIGN.md §4c through `gunbc.design_document` (operator ruling, 2026-08-04); §5 records the
D-A–D-D rulings and the four structural corrections that shaped them. Slice status is reconciled in §6.

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

### 2a. A third lexical channel — not a `TokenRule`

Rejecting `TriviaRule` is necessary but not sufficient. Making a comment an ordinary `TokenRule` still
classifies it as part of the **semantic** token stream, which buys avoidable parser filtering, token-order
effects, and occurrence-allocation risk. `v2.std.compilers.lexing` `LexRule` today is exactly two variants;
the model needs a third:

```
type LexRule
  = TokenRule      { token_class: Symbol, pattern: LexPattern }
  | TriviaRule     { pattern: LexPattern }
  | AnnotationRule { annotation_class: Symbol, pattern: LexPattern }

type LexArtifact { tokens: TokenStream, annotations: List<UnboundSourceAnnotation> }
```

| source material | result |
|---|---|
| program syntax | semantic token |
| whitespace | discarded trivia |
| `//` annotation | **captured authored-source data** |

That is the actual quarantine: it neither pretends annotation text is syntax nor erases it. Semantic
consumers keep receiving `TokenStream` through a projection; annotation-aware parsing takes the whole
`LexArtifact`.

### 2b. Wrap the semantic artifact — do not widen it

"Sidecar on `ParseArtifact`" is directionally right but structurally weak as a new *field*, because every
semantic consumer would then receive a carrier that can hold annotations. The construction is a wrapper:

```
type AuthoredParseArtifact { semantic: ParseArtifact, annotations: SourceAnnotationGraph }

fn authored_parse_semantic(artifact: AuthoredParseArtifact) -> ParseArtifact { artifact.semantic }
```

Ordinary compilation then receives a type that **cannot** contain annotations, so semantic hashing cannot
begin hashing annotation text, and inference, resolution, affected-set, and emission cannot inspect the
sidecar even accidentally. Erasure becomes structural rather than convention-plus-witness. Not
`Node.properties` (v1 `00_core`): that field participates in the semantic carrier, so annotations there
would move structural equality, content hashes, resolver walks, affected-set computation, emitted bytes,
and cache keys.

**A nuance the earlier draft got wrong.** `ParseArtifact` carries `tree` *and* `span_index`, and inserting a
comment necessarily moves the following declaration's byte range — so the whole artifact cannot be identical
with and without the comment. The law must be stated over the **semantic graph projection**, excluding
textual provenance:

| | with vs without a comment |
|---|---|
| semantic structure and identities | **unchanged** |
| source provenance | **may move, must remain correct** |
| annotation graph | **changes** |

### 2c. An annotation must not consume `OccurrenceId` — the blocking correction

The earlier sketch gave the annotation its own `occurrence: OccurrenceId`. **That is unsafe.** Production
parsing threads a graph-scoped occurrence allocator across parsed sources, so if annotations draw from it,
inserting a comment can shift the semantic identities of declarations later in the same source — or in later
modules in the graph. `SpanIndex` cannot absorb this transparently either: it is keyed by `OccurrenceId`, so
reusing it either shares the semantic identity domain or needs an unsafe partitioning convention.

The first carrier therefore has **no independently addressable annotation identity**:

```
type SourceAnnotationDebt { subject: OccurrenceId, text: String, origin: Locus }
type SourceAnnotationGraph = FreeMonoid<SourceAnnotationDebt>
```

The ordered graph preserves multiplicity and source order; `origin` is provenance, **not** identity. A
dedicated `SourceAnnotationId` arrives only when a real consumer must address one annotation independently.

> **The load-bearing law: adding, deleting, or reordering annotations never consumes, advances, resets, or
> otherwise influences the semantic occurrence allocator.**

Two controls are required, and **target-byte equality cannot substitute for either** — the emitter may
ignore occurrence ids and stay byte-identical while the identity graph silently moves:

1. a comment before the first declaration preserves every semantic occurrence identity in that module;
2. a comment in the first parsed module preserves identities in **every subsequently parsed module**.

### 2d. Real data, not program data

The object program cannot bind, import, reference, call, evaluate, or inspect an annotation. Compiler, lens,
formatter, and SCM machinery *can* enumerate it through an annotation-specific projection — necessary, since
without it neither the census nor the migration could exist. **Quarantine, not erasure.**

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

### 2e. Attachment grain: module-item only

"Next structural occurrence in the same containment scope" still promises too much — it reaches expressions,
fields, arms, and parameters whose annotation meaning is undecided. The first realization admits only **a
maximal block of standalone leading `//` lines attached to the following module-scope declaration or
import**:

| position | result |
|---|---|
| before `module` | module subject |
| before an import | import subject |
| before a top-level declaration | that declaration's subject |
| inside function/type bodies | **refuse** |
| after code on the same line | **refuse** |
| at scope end or EOF | `UnattachedSourceAnnotation` |
| block-comment form | **refuse** |

That covers the dominant migration form — top-level `data …_note` rows sitting above their subject —
without claiming expression-level semantics.

**A lexical requirement this needs.** Because whitespace and newlines are consumed as trivia,
`fn a() -> Int { 1 } // text` followed by `fn b() …` becomes indistinguishable from a leading annotation on
`b` once whitespace is gone. The channel must therefore carry an explicit line-placement observation:

```
type AnnotationPlacement = LeadingAfterLineIndent | TrailingAfterSemanticToken
```

Only `LeadingAfterLineIndent` enters the pending-attachment graph; trailing placement refuses. Consecutive
leading lines group as **one ordered annotation block**, never one annotation per physical line.

### 2f. Debt-only, by construction

The first carrier means modeling debt **by construction** — the type's standing law is that all inhabitants
are debt. `SourceAnnotationRationale` is **not** declared in the first cut: a variant with no authoring
path, no distinct consumer, and no lifecycle is speculative vocabulary, and offered early every unmodeled
invariant acquires a `why:` prefix and declares itself permanent. A permanent rationale category arrives
only when migration produces a real second population *and* its consumer is modeled. Until then even
genuinely irreducible rationale is retained safely but classified honestly as unresolved debt.

Every first-cut annotation has exactly two fixed consumers — **annotation-aware source rendering** and the
**annotation-debt census** — so no per-comment "consumer" string is needed.

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
are all optional, including that the first cut got it backwards (`extdeps.auth.jwt` module annotation on
`JwtRegisteredClaims`).

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

## 5. Rulings (operator, 2026-08-04)

D1–D4 (delete rate, ceilings) were withdrawn as the wrong questions — they presumed the current
representation. D-A–D-D are now **ruled**, each with the correction that shaped it:

- **D-A — carrier: approved with correction.** An annotation-specific **lexical channel** (§2a) returning an
  **authored wrapper around an unchanged semantic artifact** (§2b). Never `Node.properties`, never an
  ordinary token, never a binding, never restored trivia, and **never a semantic `OccurrenceId`** (§2c). The
  module home still owes a concept DFS whose first dependency is the generic lexing artifact — so it must
  include `v2.std.compilers.lexing`, `v2.std.provenance`, and the source-artifact family before
  `dag/std/source_annotation.dag` is chosen.
- **D-B — attachment: approved, narrowed.** Leading annotation blocks at **module-item grain only** (§2e).
  Body, field, expression, trailing, unattached, and block forms refuse. Do not generalize to "next
  structural occurrence" until a second subject grain actually needs it.
- **D-C — erasure: approved, strengthened.** Seven required proofs: (1) semantic structure equal;
  (2) **semantic occurrence identities equal, including across later modules**; (3) semantic IR/content hash
  equal; (4) emitted target bytes equal; (5) annotation graph different; (6) source provenance still correct
  after shifted byte ranges; (7) **annotation-aware round trip** — parse → render → parse preserves the
  block. (7) matters because a comment the formatter silently deletes is not a durable carrier.
- **D-D — migration: approved, debt-only.** Keep the §4 partition; **do not declare
  `SourceAnnotationRationale` in Slice 1** (§2f).

**Canonical policy landed:** DESIGN.md §4c, via `gunbc.design_document` `section_4c_blocks`, regenerated —
not hand-edited. The rule it states: **prose is not forbidden; unclassified prose is.**

---

## 6. Sequence after sign-off (slice status reconciled)

1. **Slice 1 — authority + pure laws.** **Landed in M1A.** `std.source_annotation` carries structural
   subject, annotation block, ordered graph, attachment refusals, erasure projection, module-item
   attachment law, and the allocator-disjointness law (§2c). Provisional carriers named in §2 were
   grounded under their final names in-tree.
2. **Slice 2 — `.dag` syntax realization.** **Landed in M1A.** The v1 seed realizes line-comment capture
   through `UnboundAnnotationCapture`, placement observation, and delimiter normalization in
   `v1.compiler.annotation_bind`; v2 modeled syntax remains a separate follow-on.
3. **Slice 3 — discriminating execution.** **M1A + M1B complete at declared scope.** Binding controls,
   admission routing, blank-line grouping, and the round-trip roster exercise parse → render → parse
   through the production admission seam.
4. **Slice 4 — make recurrence unwritable.** **Path-scoped introduction wall landed; corpus-wide wall
   open.** `gunbc.prose_row_frontier` refuses new `data *_note` rows on enrolled paths; the roster grows
   with each migrated batch rather than waiting for directories to drain first.
5. **Slice 5 — reconcile the mountain.** **M1C-0 merged; M1C-1 implemented in #7955; corpus drain
   ongoing.** jwt.dag is prose-free; four grandfathered rows remain in oidc.dag; the three annotation
   carrier modules self-apply with lifecycle/status facts retained in typed rows where no dedicated home
   exists yet.

---

## 7. Canonical policy — LANDED in DESIGN.md §4c

Landed through `gunbc.design_document` `section_4c_blocks` with DESIGN.md **regenerated** (never
hand-edited), per the operator ruling of 2026-08-04. The authority text is DESIGN.md §4c itself; it is not
restated here, because a second hand-maintained copy of one rule is exactly the dual representation this
lane exists to remove. The section states the rule and the measured reason it exists (the
#5579 → #6262 → #6424 chain), and it names the two author-binding consequences: an annotation may preserve
irreducible *why* and must not restate what the declaration structurally says; and an annotation is never
evidence that a machine claim holds, because no `Accepted` program can read one.

**The long-term rule, in one line: prose is not forbidden; unclassified prose is.** `//` becomes the
explicit quarantine boundary, and the compiler retains enough structure to migrate the text later without
ever having pretended it was program data.

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
- **§2's provisional names are largely grounded in-tree.** `std.source_annotation` and
  `v1.compiler.annotation_bind` realize the carriers this audit originally named provisionally; v2 modeled
  `AnnotationRule` syntax and the full D-C wall over v2 ingest remain open follow-ons.
- **Allocator disjointness has executed controls.** M1A binding and round-trip witnesses exercise
  attachment without minting annotation identities; the D-C controls were written to prove the law directly
  rather than to rely on reasoning alone.
- **Migration has started, not finished.** jwt.dag and the three self-applied annotation modules are
  implemented in #7955; oidc.dag still carries four grandfathered prose rows; the §4 partition and corpus
  drain remain open.
- **This audit document remains until the deletion trigger closes.** The scaffold deletes when the
  source-annotation carrier, its executed controls, and the §4 partition are complete — at which point a
  lexical audit over prose is superseded by a fold over the annotation graph and the typed populations
  beside it.
