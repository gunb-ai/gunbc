# SCM `add`/`commit` — what the corpus already models, and what is declared red

Subject of gunbc#9891 finding 6. This note settles the model before anything is built. Two of its
findings correct claims I made earlier in this analysis, and both corrections came from measuring
rather than searching.

## 1. The gap

`gunbc.scm.cli` exposes three verbs — `scm_log`, `scm_status`, `scm_init`. There is no `add` and no
`commit`. `cli.dag` documents the consequence about itself: it passes `empty_proposal()` because
"the entry does not carry what is staged", and calls that "a REAL limitation rather than a
placeholder".

So the kernel — object store, checkout, identity, role-requirement integration, ancestry, JSON
round-trip — has no producer. That is the specification-without-execution gap DESIGN §5 names, and
the SCM design note names it too: *nothing has ever consumed this kernel; a witness suite is not a
consumer.*

## 2. Correction: ingestion is modeled, not absent

An earlier draft of this note asserted that ingestion is not reachable from `.dag`. **That was
wrong**, and wrong in a specific way: it searched `dag/gunbc/scm`, found nothing, and let an empty
subtree speak for the repository — a partial observer returning the negative value of a total
observer, which is the class `gunbc.namespace_wave_admission`'s ledger records from this same
branch.

`src/v2/compiler` models the pipeline: `01_tokenize` (`lex_walk_artifact(source, file, rules)`),
`02_parse`, `03_ingest`, and `00_compile` with `compile(source: CoreNode, mode)` and
`compile_source_root_ingest_with_admission(ingest: SourceRootIngest, …)`.

## 3. Correction: the manifest vocabulary largely exists

Finding 6 describes a `CorpusManifestObject` of `path -> semantic_root -> authored_source_identity`.
`v2.compiler.source_authority` already declares:

    type SourceRef             { path: String, source_root: SourceRootRef, content_hash: ContentHash }
    type SourceStorageIdentity { path: String, source_root: SourceRootRef }
    type SourceRootIngest      = FreeMonoid<DagSourceReadWitness>

with `source_root_ingest_build_from_source_refs(refs: List<SourceRef>)` in `source_authority_read`.

**Minting a manifest carrying `path -> content identity` would be a second name for most of
`SourceRef`** — the §3 nicknaming violation. Finding 6's phrasing predates, or overlooked, this
authority; the finding's *substance* stands, its proposed vocabulary does not.

## 4. The measurement: ingest is modeled AND DECLARED RED

`v2.test.program_assembly.real_ingest` is the instrument. It builds a `SourceRootIngest` from the
host's real source refs and asserts the parse. Executed at main with `claim_batch`:

| claim | result |
|---|---|
| `program_assembly_real_ingest_module_roots_parse_holds` | **FAIL** |
| `program_assembly_real_ingest_host_manifest_receipt_holds` | **FAIL** |
| `program_assembly_real_ingest_validate_module_roots_red_on_parsed_roots` | passes (expecting-red control) |

Both failures are **declared**: `src/v2/workflow/floor_expected_red.dag` enrolls
`…real_ingest.program_assembly_real_ingest_module_roots_parse_holds` and
`…_host_manifest_receipt_holds` as expected red. The local result matches the declared expectation,
so these are the corpus's stated position, not a regression.

**So `scm` cannot obtain a semantic program root from files today** — not because the machinery is
missing, but because the claims demonstrating it parsing real source are enrolled as expected
failures.

## 5. What that leaves

A commit's subject is a semantic program (`RepositoryCommit.root: SemanticNodeObjectRef`,
`mint_repository_commit(root: SemanticNodeTarget, …)`). With §4 red, `commit` cannot honestly
produce one, and a `commit` that always refuses is specification-without-execution wearing a verb.

**Proposed slice — `add` and `status`, not `commit`:**

- `add` stages `SourceRef`s, reusing `v2.compiler.source_authority` rather than minting a manifest;
- `status` reports what is staged, which repairs the limitation `cli.dag` documents about itself.

That is a real executed consumer for the staging half — `add` writes, `status` reads it back —
without inventing vocabulary and without pretending the program half exists.

**`commit`'s trigger, named as a capability rather than an artifact:** when the corpus can parse
real source into module roots — i.e. when the §4 claims leave `floor_expected_red` — `commit` can
produce a genuine program root. Until then it is not built, rather than built and refusing.

## 6. Open question

Is staging-without-`commit` a coherent slice, or a half-verb that should be refused until the whole
vertical can land? The case for it is that it creates the kernel's first real consumer and fixes a
documented limitation; the case against is that `add` whose product nothing can commit is itself
unconsumed, one level up.
