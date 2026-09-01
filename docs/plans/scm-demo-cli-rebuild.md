# SCM demo CLI — rebuild plan and API inventory

A working `gunbc scm` CLI existed and was lost with an uncommitted `/tmp` worktree. This doc is
the durable record so the rebuild does not start from a survey again. It is a plan, not a
receipt: nothing here claims to be built.

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

`scm_log_cli_response` and `scm_status_cli_response` have NO consumer outside
`dag/test/claim/scm/scm_render_witness_test.dag`. That is the unwired-renderer state
`gunbc.cli_dispatch_surface` already records: the answer is computed and discarded because no
host binds it.

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
2. Host side in `src/v1/stage0/src/main.rs`: a `Commands::Scm` variant and a `scm_verb` that
   evaluates the entry and writes the `CliWirePrintable { bytes, exit }` bytes to stdout,
   refusing on `CliWireUnprintable { cause }`. `serve_wire_fields` / `classify_exit` in
   `cli_run.rs` are the precedent for reading wire values out of an evaluated `Value`.

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

- **A host WRITE is permitted from `gunbc run`** — `scm_init` created a file. The remaining write
  verbs are a modeling question, not a permissions one.
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
