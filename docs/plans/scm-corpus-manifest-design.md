# SCM `add`/`commit` — a commit freezes an authored corpus; a program is a projection of it

Subject of gunbc#9891 finding 6, settled with the designated SCM reviewer before implementation.
The governing sentence:

> **A repository commit freezes an authored corpus manifest. A semantic program is a separately
> bound ingestion projection of that corpus, not an alternative kind of commit subject.**

## 1. The gap

`gunbc.scm.cli` exposes `scm_log`, `scm_status`, `scm_init` — no `add`, no `commit`. `cli.dag`
documents the consequence about itself: it passes `empty_proposal()` because "the entry does not
carry what is staged", "a REAL limitation rather than a placeholder". So the kernel has no producer,
which is the §5 specification-without-execution gap the SCM design note also names.

## 2. Two corrections to this note's own earlier drafts

**Ingestion is modeled, not absent.** An earlier draft asserted otherwise. It searched
`dag/gunbc/scm`, found nothing, and let an empty subtree speak for the repository — *a partial
observer returning the negative value of a total observer*, the class recorded in
`gunbc.namespace_wave_admission`'s ledger. `src/v2/compiler` models `01_tokenize`, `02_parse`,
`03_ingest`, `00_compile`.

**Most of the proposed vocabulary exists.** `v2.compiler.source_authority` declares
`SourceRef { path, source_root, content_hash }` and `SourceRootIngest`, with
`source_root_ingest_build_from_source_refs`. Minting a manifest of `path -> content identity` would
be a second name for it. Finding 6's *substance* stands; its proposed vocabulary does not.

## 3. `semantic_root` does not belong in the manifest

Finding 6 phrases the manifest as `path -> semantic_root -> authored_source_identity`. That is
wrong for a deeper reason than present-day constructibility: **`semantic_root` is a derived
ingestion result**, depending on the source object, corpus context and imports, the ingestion rule,
language and rule versions, and possibly source-root and target configuration. Putting it in the
manifest fuses an input with a realization result, and makes the manifest unauthorable for
malformed or not-yet-ingested source — precisely the corpora a source-control system must still be
able to freeze.

The authored manifest is therefore only:

    canonical corpus path -> exact authored-source target

and the semantic relation lives in a separate carrier, whose load-bearing properties are that it
names the manifest it is about, names the ingestion rule/version, keeps each per-file semantic
result bound to the exact source identity, and derives the global program root rather than
accepting one supplied beside the corpus:

    type CorpusIngestionProjection { corpus, ingestion_rule, program_root, files }
    type FileIngestionProjection   { path, source, semantic_root }

This preserves the case that proves the split: a comment-only edit changes authored bytes, so
manifest A ≠ manifest B, while the semantic root is unchanged.

## 4. Object-store membership and edge admissibility are separate axes

The manifest joins `ScmObject` **without** becoming a legal `StoredEdge` target:

    type ScmObject = SemanticNodeObject(ObjectRecord)
                   | AuthoredSourceObject(AuthoredSourceRecord)
                   | CorpusManifestObject(CorpusManifestRecord)

`StoredEdge.target` remains exactly `SemanticNodeTarget`, so #9891's distinction is untouched. Each
kind gets its own target and its own three-way find outcome, so a walk cannot read one kind's
absence as another's: the manifest walk follows only authored-source targets, a semantic node only
semantic targets, an authored source nothing.

`CommitClosure` is **not** silently widened — its subject is a semantic root. Either a separate
corpus closure, or a parameterized mechanism whose exported outcomes still distinguish
semantic-node requirements from authored-source requirements.

## 5. `commit` must not refuse for want of ingestion

This corrects an earlier proposal in this note that `commit` be withheld until ingestion is green.

Freezing what was authored is **fully answerable today**: paths are known, bytes are known, source
identities and the manifest identity derive from them, and the exact authored state is
reconstructible. The missing fact appears only when a *semantic* question is asked — reconstruct the
node, integrate role requirements, validate, produce semantic closure. Those answer
`SemanticProjectionUnavailable { corpus }` or an ingestion refusal carrying real diagnostics.

Refusing `commit` would fuse **recording state** with **establishing semantic validity**, which the
SCM design keeps apart. An invalid corpus, or one this compiler cannot ingest, must still be
committable if `commit` is to remain distinct from certification.

So the canonical record becomes approximately:

    type RepositoryCommit sole_constructor {
      reference: RepositoryCommitRef
      corpus: CorpusManifestObjectRef
      message: String
      ancestry: CommitAncestry
    }

Two repositories may differ in *projection availability* while agreeing entirely about the commit.
Disagreement about which manifest a commit names stays forbidden; two projections under one
`(manifest, ingestion rule)` key yielding different roots must refuse as an authority collision.

Compatibility with the existing semantic-root format belongs in a **versioned decoder**, not in the
canonical writer — and absent a measured durable population, root-migrate rather than keep a
permanent legacy arm.

## 6. A naming fork to close before both sides gain consumers

`gunbc.scm.checkout` declares a bare `Commit`, which role-requirement integration returns; the
repository record is `RepositoryCommit`. They currently meet on a semantic root. Once repository
history is corpus-rooted they stop meaning the same thing, so the bare one becomes
`SemanticCommit` / `ProgramRoot` before both gain production consumers — the same §3 fork this
lane's previous PR closed for `merge`.

## 7. Scope, stated no larger than it is

`add`/`commit` makes load-bearing: the authored-source store, the manifest, repository history,
persistence, `log`, `status`, and the ancestry spine.

It does **not** make role-requirement integration load-bearing. That operation returns the separate
semantic commit whose only content is a node root; it does not produce a `RepositoryCommit`.
Joining those two worlds still needs ingestion, and eventually semantic-to-authored emission.

## 8. Sequence

1. **Model**: the manifest object, its ref and target, the three-way find outcome, the corpus
   closure, `RepositoryCommit.corpus`, the `Commit` rename, the versioned decoder.
2. **Verbs**: `add` (stage), `status` (report staged), `commit` (freeze the manifest).

Pending state is not option B: a *committed* manifest must be reachable from the commit, while a
*pending* one is intentionally not — `RepositoryEnvelope.checked_out` is already local workspace
state rather than authoritative history.

## 9. Where a manifest is wrong, and where it is merely present

Adding a third arm to `ScmObject` forces every exhaustive match over it to answer for a manifest,
including the commit-closure census and the v3 JSON encoder. Two of the three available answers are
wrong for reasons worth recording, because each is a failure class this lane has already committed
once.

**Refusing when a manifest exists anywhere in the store** makes "serialize the semantic closure
rooted at P" fail on unrelated repository inventory. A manifest is wrong **at a
`SemanticNodeTarget`**; it is not wrong **in an `ObjectStore`**. That is the same
membership-versus-admissibility split that keeps `StoredEdge.target` exactly `SemanticNodeTarget`,
applied one layer up — and collapsing it is the absorbing fallback of §5, refusing a superset
because the precise subject was not derived.

**Bumping the closure format tag** was rejected here on the grounds that the wire schema was
unchanged — and that reasoning was superseded by what the cut turned out to require. The v4 bump
that landed is the OPPOSITE claim: the writer can no longer emit `{source}` and the reader refuses
one, so the language got **narrower**, and holding a tag while the language shrinks is the same
false claim as holding it while it grows. `gunbc-scm-commit-closure-v4`, `gunbc-scm-repository-v3`
and `scm-object-identity-v2` are what shipped.

The cut is neither of the two arms above. `commit_closure_json_v2` already declared the obligation
this change discharges: every record the store holds is serialized, which it states is correct
*while nothing produces unreachable objects*. `CorpusManifestObject` is exactly the event that makes
that false, so the deferral's own dissolution trigger has fired.

So the admitted carrier stops carrying the raw store and carries the **root-reachable semantic
closure**. The design draft above proposed a two-armed `ClosureObject` with a
`ClosureAuthoredSource` arm; that arm was found to be **uninhabited** during implementation, because
no typed path reaches an authored source from a semantic root, and the reviewer's ruling changed to
nodes only. What shipped is:

- `WellKindedClosure { root: SemanticNodeTarget, nodes: List<ObjectRecord> }` — one kind, not a
  coproduct, because there is nothing else to be reached;
- the admission walks from the root; at each `SemanticNodeTarget` a node is included and recursed
  through, an authored source and a manifest are each their own named wrong-kind population;
- the encoder's parameter therefore cannot name a manifest at all: unconstructible rather than
  validated — §4b rung 4 rather than a commented-over rung 3.

Two consequences bind the implementation, and one more was found by review. First, **one derivation
authority**: both `admit_well_kinded_closure` and the partial route must consume the same
`derive_semantic_closure(store, root)` result. Second, `well_kinded_store` **loses its name**:
returning a node list under a store-shaped name would say the carrier still holds a store.

Third, and this is the one the draft missed: `sole_constructor` seals the record LITERAL, and `.dag`
has no module privacy, so a public mint beside it re-opens everything. The module shipped
`well_kinded_image(closure)`, which took an unchecked closure, discarded the derivation's census and
returned encoder input anyway. **The mint is deleted**; the literal appears once, inside the
else-branch of the census that authorizes it, and the admitted arm carries the unresolved population
so nothing walks the closure twice to re-derive it.

Unrelated authored sources leaving the document is a consequence, not a regression. "It happened to
be in the repository store" was never semantic reachability.

### The witness bar for this cut

1. **Unrelated manifest control** — semantic root, valid node closure, an unrelated manifest in the
   store: admission succeeds and the manifest is absent from the admitted closure.
2. **Manifest-at-node RED** — a `SemanticNodeTarget` whose locator holds a `CorpusManifestObject`:
   a named wrong-kind refusal, never `absent`.
3. **Unrelated source control** — an authored source not reachable from the root is absent from the
   emitted population.
4. **Partial path parity** — the ordinary and partial routes derive the same population; mutating
   either back to the raw store takes a manifest fixture red.

## 10. What the corpus closure does *not* get yet

`CorpusClosure` is **not** modelled. An earlier revision of this section said it was — naming its
complete/partial/wrong-kind outcomes, its identity behaviour and its witnesses as landed — and no
such carrier exists. What exists is `ManifestSourceCensus`, which answers the manifest-source
reachability question over a finished store in three populations (absent, node-occupied,
manifest-occupied) and is consumed by both the checked repository writer and the repository reader.
That is the minimum the write-safety question needed; it is not a closure carrier and does not
pretend to be one.

It gets no JSON codec here either. A named codec earns an actual boundary, and nothing yet persists
or transports a corpus closure; building one would add a representation with no consumer immediately
before a lane whose stated purpose is to stop doing that.

The deferral does not extend to the repository codec. The moment `add`/`commit` persists a
repository holding manifest objects, commits naming them, or pending corpus state, the repository
format necessarily advances and must encode manifests — and that lands atomically with the first
path that saves such a repository, not before and not after. If the repository envelope turns out to
be the only representation `add`/`commit` needs, a standalone corpus document should never be built
at all.

## 11. Corrections this document owes to its own earlier revisions

Recorded here rather than silently rewritten, because a plan that quietly agrees with whatever
shipped is not evidence of anything.

- **The `SourceRef` duplication warning (§2) was about a different construction.** The manifest that
  shipped holds `(path, AuthoredSourceTarget)` and mints no second source identity, so the warning
  stands as a rule and does not describe this head.
- **The manifest was specified as an ordered list and is now a function.** Ordered hashing made a
  host directory traversal order part of the manifest identity, and admitted one path naming two
  sources. Entries canonicalize by path and a duplicate path refuses.
- **`ClosureAuthoredSource` was an uninhabited arm**, as above.
- **The witness bar in §9 is met, with its second item widened.** A manifest at a semantic
  requirement is a named wrong-kind refusal in the closure, the checked writer and the reader — the
  draft only anticipated the closure.
- **That last sentence overclaimed the reader for one revision, and this is where it did.** When it
  was first written the reader adjudicated one subject: a MANIFEST ENTRY's authored source. A
  SEMANTIC CHILD naming a manifest or a file by digest still reached `RepositoryDecoded`, so the
  sentence was true of the writer and only half true of the reader. Both subjects are adjudicated
  now, against the finished decoded table, and the discriminators for each occupying kind execute in
  `test.claim.scm_repository_envelope_witness` — but the claim preceded its second half, which is
  the failure this document is a ledger of.
- **A wrong-kind requirement reached two different standings depending on how its reference was
  spelled.** `uncontained` classified as `LoadRequirementAtWrongKind`; the same predicament reached
  through a contained position classified as `LoadDocumentMalformed`, which is the one standing that
  permits the supersession step. The predicament decides the standing now, and the paired claims run
  the decoder, the classifier and the permission table rather than constructing standings by hand.
- **A "the occupant arrives after the referrer" witness never presented that order.** It changed
  store insertion order, but `emit_in_dependency_order` follows each reference through the
  kind-agnostic `find_object` and emits the occupant first regardless, so both fixtures produced the
  same bytes. The controlled version rewrites the encoded `objects` array and asserts the order took
  effect before asserting the refusal.
