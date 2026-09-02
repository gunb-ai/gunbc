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

Settled by execution, so not open questions any more:

- **A host WRITE is permitted from `gunbc run`** — established by the init prototype above, which
  created a file, and since confirmed by #10026 landing the create-only write. It is a fact about
  the HOST, not a claim that init's model is finished. The remaining write verbs are a modeling
  question, not a permissions one.
- **Content goes in via `store_node(store, n: Node) -> StoreOutcome`** (`gunbc.scm.object_store`).
- **A Node is built with `node_synthetic`** (`v2.std.node`):
  `node_synthetic(kind: TypeNode { connective: Atom { identity: sym } }, children: [])`. The `atom`
  helpers in the test modules are LOCAL helpers, not a shared authority — do not import one.
- **A runtime name becomes a Symbol** with `symbol_intern_lexeme(lexeme: name)`
  (`v2.std.compilers.lexing`). `name as Symbol` refuses.
- **`mint_repository_commit(repository, root: ObjectId, message, parent: RepositoryCommitRef?)`**
  requires `store_contains(root)` — so `add` must precede `commit`, and because identity is derived
  from content, `commit` can re-derive an added object's `ObjectId` by rebuilding the same node.

### The design step, stated so it is not improvised

`add` composes `store_node` (`Stored | LocatorCollision`) with `save_repository` (4 arms);
`commit` composes `mint_repository_commit` (4 arms) with the same save. A renderer taking two
outcome values is the wrong shape — it would have to encode "which one failed" positionally, and
the arms multiply.

What is wanted is one modeled write-verb outcome, something like:

    type ScmWriteOutcome
      = ScmWriteRepositoryUnavailable { cause: RepositoryLoadRefusal }
      | ScmWriteRefusedByStore { collision: StoreOutcome }
      | ScmWriteRefusedByMint { refusal: RepositoryCommitMint }
      | ScmWritePersisted { save: RepositorySave }

with one renderer per verb over it. That coproduct is the increment's real content; the wiring
after it is mechanical. It is deliberately NOT sketched into `cli.dag` here, because a write-verb
outcome invented at a call site is the anemic modeling this repository keeps paying for.

### `add` has no staging authority, and that is not a wiring gap

`repository_status` takes `pending: Proposal` as a PARAMETER because what is staged is not a fact
the repository document carries. So a literal `git add` — stage now, commit later — has nowhere to
persist to. Either `add` writes an object into the store immediately (content-addressed, no
staging), or a staging authority has to exist first. The first is what the object store supports
today and is the honest reading of `add` for this substrate.

## Constraints learned the hard way

- A runtime name becomes a `Symbol` via `symbol_intern_lexeme(lexeme: name)`
  (`v2.std.compilers.lexing`). `name as Symbol` REFUSES; an earlier conclusion that runtime names
  were inexpressible was wrong.
- `RepositoryCommitRef` has no `ordinal` field. Read it with `minted_id_ordinal(id: ref.identity)`
  and construct via `RepositoryCommitRef { identity: MintedId { ordinal } }`.
- Annotations inside a declaration body are parse errors; hoist them above the declaration.
  An indented `//` is also a parse error — annotations are module-item grain only.
- `gunbc.scm.merge` is roles/requirements/supersession, NOT two-commit merging. The operator's
  ask for practising merges/conflict resolution needs merge DESIGN first; it is not a wiring job.

## Not in scope here

Mirroring this repository's history through `gunbc scm` onto srv2 stays blocked on model work:
`ScmObject = SemanticNodeObject | AuthoredSourceObject | CorpusManifestObject` with one content
identity, `CorpusManifest` carrying path + semantic_root + authored_source_identity, and git
correspondence confined to a mirror-layer `GitMirrorCursor { source_commit, native_commit }`.
