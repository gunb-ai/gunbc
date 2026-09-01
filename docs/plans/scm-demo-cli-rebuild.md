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

1. `dag/gunbc/scm/cli.dag` — one entry per verb returning `CliWireResponse`, composing
   `load_repository` -> verb -> render, with a `RenderCapability { color: false, tier: Ascii,
   cursor_addressable: false }` for a plain terminal. Write verbs (`init`, `add`, `commit`,
   `checkout`) need their own `Document` builders; only log/status have them today.
2. Host side in `src/v1/stage0/src/main.rs`: a `Commands::Scm` variant and a `scm_verb` that
   evaluates the entry and writes the `CliWirePrintable { bytes, exit }` bytes to stdout,
   refusing on `CliWireUnprintable { cause }`. `serve_wire_fields` / `classify_exit` in
   `cli_run.rs` are the precedent for reading wire values out of an evaluated `Value`.

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
