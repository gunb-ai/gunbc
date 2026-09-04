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
