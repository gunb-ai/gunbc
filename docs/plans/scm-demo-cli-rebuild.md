# SCM demo CLI — rebuild plan and API inventory

A working `gunbc scm` CLI existed and was lost with an uncommitted `/tmp` worktree. This doc is
the durable record so the rebuild does not start from a survey again.

It is a PLAN AND A HISTORICAL LEDGER, and the two are separated by section. Everything above
`## LANDED` is the plan and the baseline survey, written BEFORE this work and describing main as
it stood then; it claims nothing is built. `## LANDED` onward is the ledger, and it carries
execution receipts for what has actually shipped. Read a present-tense statement in the plan half
as a statement about the BASELINE, not about the tree today.

## What already exists on main (verified by reading the modules)

The read side is modelled and already reaches `CliWireResponse`:

| module | symbol | shape |
| --- | --- | --- |
| `gunbc.scm.read_command` | `ScmReadResult<T>` | `ScmReadAnswered { path, answer }` \| `ScmReadRepositoryUnavailable { cause }` |
| `gunbc.scm.render` | `scm_log_cli_response(result, target, capability)` | `-> CliWireResponse` |
| `gunbc.scm.render` | `scm_status_cli_response(result, target, capability)` | `-> CliWireResponse` |
| `gunbc.scm.render` | `scm_read_exit<T>(result)` | `-> ProcessExit` |
| `gunbc.scm.log` | `repository_log(envelope)` | `-> CommitLog` |
| `gunbc.scm.status` | `repository_status(envelope, pending)` | `-> RepositoryStatus` |
| `gunbc.scm.repository_load` | `load_repository(path)` | `-> RepositoryLoad` |
| `gunbc.scm.repository_load` | `repository_load_refusal_lines(cause)` | `-> List<String>` (takes `cause` ONLY, not `path`) |
| `gunbc.scm.repository_save` | `save_repository(path, repo)` | `-> RepositorySave` |
| `gunbc.scm.repository_envelope` | `empty_repository()` | `-> RepositoryEnvelope` |
| `gunbc.scm.repository_envelope` | `mint_repository_commit(...)` | `-> RepositoryCommitMint` |
| `gunbc.scm.repository_envelope` | `commit_at_reference(commits, reference)` | `-> RepositoryCommit?` |
| `gunbc.scm.checkout` | `checkout_repository_at(repo, reference)` | `-> RepositoryCheckout` |
| `gunbc.scm.authoring` | `empty_proposal()`, `author_requirement(proposal, requirement)` | `-> Proposal` |
| `gunbc.scm.object_store` | `empty_store()`, `store_contains`, `find_object`, `store_object_count` | — |

## The gap, stated precisely

**This was the baseline gap, and this change closes it.** As of the survey above,
`scm_log_cli_response` and `scm_status_cli_response` had NO consumer outside
`dag/test/claim/scm/scm_render_witness_test.dag` — the unwired-renderer state
`gunbc.cli_dispatch_surface` recorded: the answer computed and discarded because no host bound it.
`gunbc.scm.cli` is that consumer, and the `LANDED` section below carries its receipts.

`dag/gunbc/cli_dispatch_surface.dag` declares an `scm` verb with an operand and `--path`, marked
`AbsentFromEmitMainRs`.

## Two host shapes, and why the idiomatic one is not sufficient

The corpus's established instrument shape is `fn check(subject) -> ProcessExit`, returning
`exit_failure(reason: join(lines, "\n"))` — see `tools.bare_name_fork_check`. `gunbc run` prints
that reason. **It can only emit text on FAILURE.** `log` and `status` must print on SUCCESS, so
that shape cannot express them, and returning `CliWireResponse` from `gunbc run` refuses (its
not-`ProcessExit` refusal prints the value, which is not a CLI).

So the host binding is REQUIRED, and it is the piece to rebuild:

1. DONE for read verbs. Still to do for WRITE verbs (`init`, `add`, `commit`, `checkout`):
   they need `Document` builders, which only log and status have today, and `save_repository`
   goes through `Filesystem.Write`, so they are host-effect entries rather than pure reads.
   Original note: `dag/gunbc/scm/cli.dag` — one entry per verb returning `CliWireResponse`, composing
   `load_repository` -> verb -> render, with a `RenderCapability { color: false, tier: Ascii,
   cursor_addressable: false }` for a plain terminal. Write verbs (`init`, `add`, `commit`,
   `checkout`) need their own `Document` builders; only log/status have them today.
2. Host side. **NOT the shape that landed, and deliberately so — this bullet is a FUTURE
   ERGONOMIC SURFACE, not unfinished work required by the binding that shipped.** The original
   plan was a `Commands::Scm` variant in `src/v1/stage0/src/main.rs` plus a `scm_verb` evaluating
   the entry and writing `CliWirePrintable { bytes, exit }` to stdout. What landed instead binds
   `CliWireResponse` ONCE in the generic `run_verb` outcome seam, so EVERY wire-returning entry
   becomes reachable rather than only the scm family — one binding instead of one per verb. A
   dedicated `gunbc scm` subcommand would be a nicer surface over the same answer; it is not
   needed for the answer to reach an operator, and it is not part of this change.

## LANDED — the read side reaches an operator

`dag/gunbc/scm/cli.dag` (`scm_log`, `scm_status`) plus the `run_verb` outcome-seam binding.
Receipts, by execution against `dag/test/fixture/scm_repository_load/empty_repository.json`:

    $ gunbc run --entry dag/gunbc/scm/cli.dag --function scm_log --arg path=<fixture>
    no commits yet
    exit 0

    $ gunbc run --entry dag/gunbc/scm/cli.dag --function scm_status --arg path=<fixture>
    nothing checked out
    0 commits
      nothing staged

    $ gunbc run --entry dag/gunbc/scm/cli.dag --function scm_log --arg path=/tmp/no-such-repo
    cannot read repository at /tmp/no-such-repo
      No such file or directory (os error 2)
    repository unavailable
    exit 1

The third is the load-bearing one: bytes printed AND a nonzero exit, the case an absorbing
implementation turns into silence or a spurious 0.

### `init` — LANDED separately, via the create-only write (#10026)

**init was CUT from the log/status change and landed on its own, as gunbc#10026.** The receipts
below were taken against the ORIGINAL prototype and are kept as its historical record; the shape
that actually landed differs, and the differences are the point.

Why it could not ride along: the prototype observed an absence and then called `save_repository`,
which writes UNCONDITIONALLY. A racing actor between the observation and the write makes it
truncate the very bytes it claims to refuse — a TOCTOU the four-arm fold does not close, because
the fold decides on a fact that can stop being true before the write happens. Closing it needed an
atomic create-only write (`O_CREAT|O_EXCL`) as a new modeled file-transport verb, which is why it
became its own change with its own review.

What #10026 landed, and what review then found still open in it: `Filesystem.WriteCreateNew` makes
the existence test and the creation one syscall, and `create_repository` routes through it. Review
`5089156132` then found six further defects in the init MODEL — among them that `ScmInitialized`
could contain a FAILED save, and that the `FilesystemEstablishedAbsence` authorizing the write was
matched as `_` and discarded while the actuation used an independently supplied path. Those are
being repaired in the follow-up on the same branch; do not read this subsection as describing a
finished init.

`scm_init` takes a DIRECTORY and a NAME rather than a path, because presence is a fact about a
directory listing and recovering the directory by splitting a string would be a second, positional
naming scheme for something the caller already knows. It routes through
`extdeps.filesystem.filesystem_io` `filesystem_file_observation`, so absence is established from a
listing that succeeded and did not name the entry — never from a failed read, whose success channel
cannot separate absent from unreadable. Only the established-absence arm reaches `save_repository`.

Two earlier revisions were fail-open and both were caught by external review before merge: the first
called `save_repository` unconditionally and NAMED the overwrite in a comment; the second proceeded
on `RepositoryFileUnreadable`, which is a guess about the host's permission model — a file can be
unreadable and perfectly truncatable — and again named the residual instead of refusing it
(review 58060 on gunbc#9864).

Receipts, one per arm, `gunbc run --entry dag/gunbc/scm/cli.dag --function scm_init --arg
directory=<d> --arg name=repo.json`:

    fresh directory       [file] list … / [file] read … / [file] write …/repo.json (166 bytes)
                          initialized repository at /tmp/scminit/fresh/repo.json

    repository present    refusing to initialize /tmp/scminit/existing/repo.json
                            a repository is already there, and init would replace its whole history

    foreign file present  refusing to initialize /tmp/scminit/foreign/repo.json
                            a file is already there that is not a repository, and init would destroy it
                          and the file read back afterwards still holds its original bytes

    directory unlistable  refusing to initialize /tmp/scminit/nodir/repo.json
                            the path could not be observed, so nothing about it is established --
                            the directory /tmp/scminit/nodir could not be listed, so the absence of
                            repo.json is not established -- Permission denied (os error 13)

The fourth is the load-bearing one: an observation that could not be made refuses rather than
widening to "nothing is there", which is the absorbing fallback DESIGN §5 names.

Those init claims were hermetically enrolled in `test.claim.scm.scm_cli_witness` at the time — no
init refusal rendering as a success exit, the three refusals rendering differently, the unobserved
arm carrying the host's cause through. **They left with init and are NOT in this PR's witness**;
they travel with `scm-init-create-only`. Verified: `scm_cli_witness` on this branch contains no
init claim at all.

### Evidence boundary, stated rather than implied

Enrolled and executing in CI (`rust-unit-tests`, `cargo test -p v1-compiler --lib`): the five
`cli_wire_outcome_tests` covering the total map — bytes with the response's own exit, a rendered
answer that still fails, a renderer refusal not reading as empty success, the non-wire
fall-through, and the inherited `ExitFailure { code: 0 }` refusal.

NOT enrolled: the end-to-end runs above are a MANUAL receipt. An integration test that executes
the binary would live in `src/v1/tests/`, which `clippy --all-targets` compiles and no CI step
runs (DESIGN: no CI step executes test targets outside `--lib`). A `.dag` witness cannot stand in
either — `scm_log` reads a file, and the SCM witnesses are `SubstrateInputsOnly`, refusing at the
hermetic boundary. So the class is `mitigatable` on the e2e path and the next-rung trigger is a
CI step that runs an integration target, not another test file.

## Next increment: `add` and `commit`, and the one design step they need

Still settled, because it is a fact about the HOST rather than about any object model:

- **A host WRITE is permitted from `gunbc run`** — established by the init prototype above, which
  created a file, and since confirmed by #10026 landing the create-only write. It is not a claim
  that init's model is finished. The remaining write verbs are a modeling question, not a
  permissions one.

### SUPERSEDED BY #9891 — the recipe below is historical reasoning, not the current model

**This section previously listed the following as "settled by execution". #9891 replaced the
identity model underneath it, so the recipe is now WRONG at exactly the boundary add/commit needs,
and it is kept only as the reasoning that led to the replacement.** It survived on main because a
plan that says "settled" is read as an instruction; that is the failure this heading exists to
stop.

The superseded recipe said: content goes in via `store_node(store, n: Node) -> StoreOutcome`; a
file is built as a synthetic semantic `Node` via `node_synthetic`; a runtime name becomes a
`Symbol` with `symbol_intern_lexeme`; and
`mint_repository_commit(repository, root: ObjectId, ...)` requires `store_contains(root)`, so `add`
precedes `commit` and `commit` re-derives an added object's `ObjectId` by rebuilding the same node.

**What #9891 changed, and why the recipe cannot survive it:**

- Authored source is its OWN object arm, not a semantic node wearing a node costume. A file has
  somewhere to live that is not a synthetic `Node`, which is the whole point of that change.
- A semantic-node reference and an authored-source reference are DISTINCT, so "rebuild the same
  node and re-derive its `ObjectId`" no longer identifies one thing.
- `mint_repository_commit` takes a `SemanticNodeTarget`, not a bare `ObjectId`. Handing it an
  authored-source identity is not a runtime refusal to be checked for — it does not typecheck.

**SUPERSEDED — this paragraph became the dead recipe it warned about, and is corrected in place.**
It said `add` and `commit` are blocked on building a `CorpusManifestObject` shaped
`path → semantic_root → authored_source_identity`. Both halves are now false:

- **The object exists.** `gunbc.scm.object_store` declares `CorpusManifestObject`,
  `CorpusManifestRecord`, and `CorpusManifestEntry`, and `gunbc.scm.object_table_json` codes them.
- **Its shape is `{ path, source }` — path to AUTHORED SOURCE, and deliberately not to a semantic
  root.** `object_store` states why: a semantic root is a derived ingestion result, and fusing it
  into the entry would make a manifest unauthorable for malformed or not-yet-ingested source, which
  is precisely the corpus a source-control system must still freeze. #10522 §5 ruled the same
  separation — the authored snapshot is authoritative, structural interpretation is a derived view.

**The actual boundary** is a *pending authored-source snapshot* authority shared by three consumers:
`add` updates it, `status` reads it, and `commit` consumes it. It is workspace-scoped and must
survive between invocations; it need not live in the repository document.

Two constraints that paragraph got right and one it got wrong. Right: immediate object-store
insertion is not by itself the `add` model, and `ScmWriteOutcome` is still the right shape for the
verb's result. Wrong: `add` is not blocked ON THE MANIFEST OBJECT, which is the specific dependency this
paragraph asserted and which no longer exists. It remains blocked on real work — host acquisition of
file bytes, path admission, a durable selection to persist into, and conditional update of that
selection — and `status.dag`'s `StagedRole` is role-grain, so it is not that place. A removed
dependency is not a finished verb.

**`status` moves with this cut rather than being an unchanged downstream reader.** Today
`scm_status` passes `empty_proposal()` and renders "nothing staged". Once source staging exists,
leaving that path in place would report "nothing staged" while authored-source changes are staged —
an observation correct about its input and wrong about the subject the user asked about. Paths must
not be translated into `StagedRole` to keep the existing renderer.

**Staging carries the same conditional-write obligation as publication.** One `add` can race
another, and a `commit` can race a newer stage. Updates are conditional on the observed stage; a
commit consumes the exact staged snapshot rather than re-reading working files; and clearing after
commit consumes only the stage generation actually committed, so newer work is never erased. Under
the no-rebase workflow a moved target is a named stale-base condition, never permission to apply an
old stage to a different base. `save_repository` does not supply this — it checks encoding and then
performs an unconditional `Filesystem.Write`.

Two further distinctions the verbs owe: an established unchanged candidate is not the same as an
unavailable or unreadable stage; and staged-versus-unstaged reporting needs a separate working-tree
observation, because comparing the stage to its base says nothing about whether the working files
changed again.

### The design step — SUPERSEDED as a recipe; one principle survives

**The composition below is no longer the instruction for the next increment, and the sketch under it
is unsound. Both are kept visible rather than deleted, because the sketch is the more instructive
half.**

It said `add` composes `store_node` with `save_repository`, and `commit` composes
`mint_repository_commit` with the same save, over one outcome coproduct:

    type ScmWriteOutcome
      = ScmWriteRepositoryUnavailable { cause: RepositoryLoadRefusal }
      | ScmWriteRefusedByStore { collision: StoreOutcome }
      | ScmWriteRefusedByMint { refusal: RepositoryCommitMint }
      | ScmWritePersisted { save: RepositorySave }

**Why the sketch is wrong: the outer classification and the inner payload can disagree.**
`RepositorySave = RepositorySaved | RepositorySaveRefusedByCodec | RepositoryFileUnwritable |
RepositoryWriteByteCountUnrepresentable`, so `ScmWritePersisted { save: RepositoryFileUnwritable {
path: "repo.json", error: "permission denied" } }` is constructible — a renderer selecting on the
outer `Persisted` arm announces persistence while its own payload says the file was unwritable. The
inverse is equally expressible: `ScmWriteRefusedByMint` accepts the successful `RepositoryCommitMinted`
arm. This is a fabricated-plausible-output shape, which makes it validation-where-construction-was-
available rather than a modelling nicety.

**The governing requirement instead:**

> A success outcome carries established success evidence for that exact operation. A refusal carries
> an actual refusal cause. Where publication cannot be established, that uncertainty remains an
> explicit outcome rather than being resolved into either.

**What survives** is only the original motivation: compose operation outcomes without positional
ambiguity, so a renderer never encodes "which one failed" by argument position. **What does not
survive** is this coproduct, and also the composition itself — `store_node` does not preserve
authored source as authored source, and `save_repository`'s write is unconditional, so neither
supplies the staging contract stated above. Narrowing the payload to a successful save would not fix
that: an ordinary save and a *conditional* stage update are different guarantees.

The shared result type is left undesigned here deliberately. A write-verb outcome invented at a call
site is the anemic modelling this repository keeps paying for, and so is one certified in a plan
before its operations have contracts.

### `add` has no staging authority — SUPERSEDED, and its conclusion was the wrong branch

This section's PREMISE stands and its CONCLUSION is withdrawn. The premise: `repository_status` takes
`pending: Proposal` as a parameter because what is staged is not a fact the repository document
carries, so a literal stage-now-commit-later has nowhere to persist to. That remains true, and it is
why `status` moves as part of this cut.

The conclusion — that `add` therefore writes an object into the store immediately, with no staging,
and that this is "the honest reading of `add` for this substrate" — is **withdrawn**. It picked the
branch that was cheapest to build rather than the one that answers the question, and immediate
insertion is not staging: it cannot say which paths are selected for the next commit, and it cannot
be revised or cleared. The other branch was the right one — a staging authority has to exist first —
and it is workspace-scoped, not necessarily a repository-document member, so the premise never
implied its absence.

## Constraints learned the hard way

- A runtime name becomes a `Symbol` via `symbol_intern_lexeme(lexeme: name)`
  (`v2.std.compilers.lexing`). `name as Symbol` REFUSES; an earlier conclusion that runtime names
  were inexpressible was wrong.
- `RepositoryCommitRef` has no `ordinal` field. Read it with `minted_id_ordinal(id: ref.identity)`
  and construct via `RepositoryCommitRef { identity: MintedId { ordinal } }`.
- Annotations inside a declaration body are parse errors; hoist them above the declaration.
  An indented `//` is also a parse error — annotations are module-item grain only.
- There is no two-commit merge in the model. `gunbc.scm.role_requirement_integration`
  (formerly `gunbc.scm.merge`, renamed because one spelling was carrying two materially different
  contracts) integrates authored role requirements into ONE target commit; its vocabulary lives in
  `gunbc.scm.proposal`. The operator's ask for practising merges/conflict resolution needs
  commit-merge DESIGN first; it is not a wiring job.

## Not in scope here

Mirroring this repository's history through `gunbc scm` onto srv2 stays blocked on model work:
`ScmObject = SemanticNodeObject | AuthoredSourceObject | CorpusManifestObject` with one content
identity — **which now exists**; the manifest carries `{ path, source }` to authored source, NOT
path + semantic_root + authored_source_identity as this sentence originally said (see the corrected
boundary above) — and git correspondence confined to a mirror-layer
`GitMirrorCursor { source_commit, native_commit }`.
