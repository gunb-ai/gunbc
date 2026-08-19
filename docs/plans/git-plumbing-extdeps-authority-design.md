# Git plumbing as an extdeps authority

DRAFT, design-note-first. No code lands from this note yet. It scopes one PR (`GIT-PLUMBING-0`) —
the dissolution of a split Git authority — and states what it deliberately does not decide.

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

## 5. The cut is the whole plumbing surface, not one operation

DESIGN §3's replacement-migration rule is greedy root-first: attempt the maximal cut and descend
only on an exact refusal — Y-incomplete, an escaping consumer, or opaque — because a separate
deeper cut pays admission surfaces, receipts and X-compatibility work the root cut never pays.

An earlier draft of this note cut at ref mutation alone and left the other seven plumbing families
private to `gunbc.devboot`. That had no refusal behind it. Ref mutation is the operation with an
interesting transactional guarantee, which is a reason to sequence the *proof* there — not a reason
to leave the authority split standing. Descoping would have left devboot the private authority for
Git plumbing while each later family paid its own admission round, which is the attractor the rule
exists to prevent.

So the cut is the split itself: **no `gunbc.devboot` declaration constructs a Git plumbing argv
list, and the corresponding `RetainedWitnessRun` rows delete rather than being repointed.** The
census in §1 is the deletion population, not a work-list for later.

The upstream-shaped surface, joining `extdeps/git/`'s existing decomposition rather than arriving
as one generic operation:

```
object database   HashObjectWrite · CatFileExists · CatFileSize · CatFileRead · UnpackObject
index and tree    ReadTreeIntoIndex · CheckoutIndexToPrefix · WriteTreeFromIndex
commit            CommitTree { tree, parents: NoParents | Parents { first, remaining }, message }
ref store         ObserveRef · CreateRefIfAbsent · AdvanceRefIfExpected · DeleteRefIfExpected
```

Repository-scoped operations that already exist — `RevParseInRepo`, `CatFileBlobInRepo` — are
consumed, never reintroduced under a plumbing module.

Acceptance:

1. The three-arm pure ref model in `extdeps.git.object_store`, preserving every existing refusal arm
   (repository-invalid, stale, missing, duplicate, symbolic cycle, format mismatch, target
   unavailable, target-kind refused). Create and delete inherit that decode and dereference; they do
   not re-derive it.
2. Repository, index path, ref, OID, parent set and output path are typed operation inputs. Git's
   empty-old encoding exists only in the `CreateRefIfAbsent` transport projection.
3. Executed against real repositories, with independent read-back on every success: create succeeds;
   a second create refuses with the original value intact; exact advance succeeds; stale advance
   refuses; exact delete succeeds; stale delete refuses.
4. Devboot's behavior survives unchanged except where the migration intentionally exposes a defect
   (§4).
5. The ~17 Git rows leave `gunbc.devboot.transport`; the ~13 host-effect rows remain, still pointing
   at `host_effect_apply`.

Out of scope: any SCM behavior. This PR makes the plumbing an authority with one real consumer cut
over; M0 is the second consumer and proves the surface is not devboot-specific, in its own PR.

## 6. Deliberately undecided

Whether `gunbc.devboot` should adopt `std.materialization_provider` is **open**, and the roster
argues against assuming it. All seven rows of `gunbc.materialization_provider_targets` are
compiler-internal caches identified by a `CacheInterfaceId` from a cited catalog row; devboot is a
cargo-build artifact service with no such row and a different grain. The roster already carries a
declared exclusion of this exact shape — `extdeps.realization.artifact_store_fs` sits *under* the
contract as a transport and cannot inhabit the target's cache field at all — and devboot's Git
object database plausibly occupies the same position. `ArtifactRequest` is additionally a closed
coproduct with an explicit `request_fold`, so a new arm edits every consumer.

Two facts found while scoping this note argue that the question is not merely open but currently
**unanswerable in the adoption direction**, and both are recorded here so the next author does not
rediscover them. Devboot decides reuse by `build_subject_equal` over the whole `BuildSubject` after
the digest narrows to a candidate — the digest is documented there as an index, not a decision —
whereas `provider_serve` decides on `ContentHash` equality alone. And `MaterializedArtifact` retains
only `request_key` plus its parts, so the contract holds no subject that could differ from a
request's: the same-key-different-subject state is not representable, which is why the provider's
witness suite contains no collision test. Routing devboot into the contract as it stands would
therefore lower a wall rather than share one.

That is a finding about `std.materialization_provider`, not about devboot or about this PR, and it
belongs to its own lane with its own operator ruling — the module is contract-stable after four
review passes and seven consumers are scheduled to adopt it. Nothing here depends on its outcome:
if the contract later grows an exact-comparison door, devboot's fold is the reference
implementation; if it does not, devboot remains a build-domain materializer whose storage
realization is Git. Either way its uniformity work is exactly §5's cut and nothing more.

## Dissolution trigger (DESIGN §6)

This note dissolves into the carriers it names when the family lands: the model and operations in
`extdeps.git`, and the deleted rows in `gunbc.devboot.transport`. The §6 materialization finding
does not dissolve with it; it is owed a lane of its own.
