# Git plumbing as an extdeps authority — the exact-ref-mutation lift

DRAFT, design-note-first. No code lands from this note yet. It scopes one PR (`GIT-REF-EXACT-0`)
and states what it deliberately does not decide.

## 1. The finding

`extdeps.git` models Git's **porcelain and read** surface. `gunbc.devboot` privately models Git's
**object-store plumbing** surface, as raw argv. Neither fact is written down anywhere, and together
they are one authority split at the surface grain (DESIGN §3), not one un-lifted operation.

Measured against the current tree:

- `extdeps.git` `git` service declares ~35 operations — `CommitInRepo`, `MergeNoEditInRepo`,
  `RevParseInRepo`, `LsFilesUnmergedInRepo`, the diff family, config, fetch. It declares **no**
  `write-tree`, `commit-tree`, `hash-object`, `update-ref`, `checkout-index`. The sole `update-ref`
  spelling in `extdeps/` is an argv helper that hardcodes `FETCH_HEAD` and carries no expected-old.
- `gunbc.devboot` carries the plumbing as argv: `update-ref` ×6, `hash-object` ×5, `cat-file` ×5,
  `read-tree` ×3, `write-tree` ×2, `unpack-file` ×2, `rev-parse` ×2, `commit-tree` ×1.
- `extdeps.git.object_store` `git_model_update_ref_exact` is the **pure** exact-old transition, and
  it is already rich: recursive symbolic-ref dereference with the symbolic ref retained, stale,
  missing, duplicate, symbolic cycle, repository-invalid, format mismatch, target unavailable, and
  target-kind-refused arms. Its own `git_modeled_ref_transition_boundary_note` already rules that
  P3 must consume an executing transport observation and never promote the model to `Applied`.

So the pure model exists and is good; the *operation* does not exist; the only realization in the
tree is private to one service.

## 2. Why this is a mis-aimed dissolution, not merely missing work

`gunbc.devboot.transport` is disciplined: all 30 raw argv sites are registered `RetainedWitnessRun`
rows with a reason, and the module note claims no unmarked `WitnessBin.Run` call survives review.
Every row declares the same dissolution target — `gunbc.host_effect_realize` `host_effect_apply`,
shell→intent Phase 2.

That target is correct for roughly 13 rows (`rm_rf`, `mkdir`, `cp`, `mv`, `chmod`, `sleep`, `uname`,
`ldd`, `sha`, `cargo_build`, `rustc_version`, `execute_artifact`, `rm_file`) and **wrong for
roughly 17** (the git plumbing above). A Git operation's terminus is a modeled `extdeps.git`
service operation — DESIGN §3 (a), the dependency's interface shape, which extdeps owns — while
`host_effect_apply` is §3 (b), a transport. Routing `git write-tree` into a generic host-effect
actuator dissolves the *transport* and leaves Git's interface unmodeled permanently.

The roster therefore conflates two dissolution classes under one target, and the git half's trigger
cannot fire correctly. This PR discharges declared debt rather than adding surface.

## 3. The operation family

One closed mutation family, three arms. Create, exact advance and exact delete are distinct
transitions with distinct preconditions in Git's own interface; they are not distinct authorities.

```
GitExactRefMutation
  = GitCreateRefIfAbsent    { name, new }
  | GitAdvanceRefIfExpected { name, expected_old, new }
  | GitDeleteRefIfExpected  { name, expected_old }
```

`expected_old: Optional<GitObjectId>` is **refused**. An absent expected-old would conflate four
states with different remedies — create-only, update unconditionally, the caller did not supply the
fact, and the caller does not know the current value — which is the state-space conflation DESIGN §5
names. The empty string devboot passes today is a transport encoding of Git's protocol, never a
semantic value, and it disappears from the model.

The pure model generalizes to the family while preserving every existing arm; `git_model_update_ref_exact`
becomes the advance arm's caller-facing spelling, not a second authority. Create and delete must
inherit — not re-derive — repository decode, symbolic dereference, format consistency and
target-kind validation.

### Outcome and read-back

The operation does not decode Git's stderr into semantic causes. Devboot already establishes why:
losing a race and failing to reach the store both exit 128 (`produce.dag`
records this by measurement), so exit codes do not discriminate and messages are not a structural
interface. The outcome is decided by **observation**, not by the process result alone:

```
exit 0   + desired state observed        => Applied
nonzero  + expected state no longer held => PreconditionNotHeld
nonzero  + expected state still held     => ExecutionRefused
any exit + state not observable          => ObservationRefused
exit 0   + desired state not observed    => ReadBackMismatch
```

`Applied` is unreachable without an independent read-back, which is what the existing boundary note
already requires of P3. Consumers project their own remedies from the common fact — a refused
advance whose observed value differs is a stale parent and re-evaluates internally; a refused create
whose ref is present is an attach.

## 4. The migration is the discriminator

Replacing devboot's argv sites is the point of the PR, not a cleanup tail. Each site must declare
which transition it is, and one site already fails that test:

**`gunbc.devboot.build` `publish_produced_tree` writes the produced-answer ref unconditionally.**
The ref is per-request-token and holds one build answer; the function is reached from both the
success path and `serve_refusal`. A re-serve of one token therefore silently overwrites a previous
answer, and a client that read the old answer and a client reading the new one disagree with no
signal. The ref is write-once by intent, so the correct transition is `GitCreateRefIfAbsent`, and a
second publish for one token is a real conflict that must refuse loudly.

No unconditional arm is added to preserve that call. A site that can name neither create nor exact
advance refuses until its intended transition is modeled.

## 5. Scope

In:

1. The three-arm pure model in `extdeps.git.object_store`, preserving every existing refusal arm.
2. The Git service operations, with typed observation and read-back.
3. Executed receipts against a real repository: create succeeds; second create refused with the
   original value intact; exact advance succeeds; stale advance refused; exact delete succeeds;
   stale delete refused; every success independently read back.
4. Migration of devboot's `update-ref` sites, with `publish_produced_tree` converted to create.
5. Re-aiming the ~17 git rows in `gunbc.devboot.transport` to the extdeps operations they actually
   dissolve into, leaving the ~13 host-effect rows pointing at `host_effect_apply`.

Out:

- The remaining plumbing operations (`write-tree`, `commit-tree`, `hash-object`, `read-tree`,
  `checkout-index`, `cat-file`, `unpack-file`). They are the same class and the same lift, but ref
  mutation is the one with a transactional guarantee to prove; sequencing them after keeps this PR
  one conceptual change. The census in §1 is their work-list.
- Any SCM behavior. This PR has two consumers available; it makes no claim about either.
- Devboot's materialization question (§6).

## 6. Deliberately undecided

Whether `gunbc.devboot` should adopt `std.materialization_provider` is **open**, and the roster
argues against assuming it. All seven rows of `gunbc.materialization_provider_targets` are
compiler-internal caches identified by a `CacheInterfaceId` from a cited catalog row; devboot is a
cargo-build artifact service with no such row and a different grain. The roster already carries a
declared exclusion of this exact shape — `extdeps.realization.artifact_store_fs` sits *under* the
contract as a transport and cannot inhabit the target's cache field at all — and devboot's Git
object database plausibly occupies the same position. `ArtifactRequest` is additionally a closed
coproduct with an explicit `request_fold`, so a new arm edits every consumer.

The question to answer first is the discriminator: what fact establishes devboot as a *consumer* of
the materialization contract rather than a realization sitting beneath it? If a cargo build is a
host-effect realization whose content-addressed store happens to be Git, devboot's uniformity work
is exactly §5's lift and nothing more.

## Dissolution trigger (DESIGN §6)

This note dissolves into the carriers it names when the family lands: the model and operations in
`extdeps.git`, the re-aimed rows in `gunbc.devboot.transport`, and the remaining-plumbing work-list
as its own registered row.
