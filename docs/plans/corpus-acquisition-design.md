# Corpus acquisition: the listing root cut, the descent measure, and the symlink rule

The corpus manifest landed in gunbc#10445 as a carrier with no producer: `store_corpus_manifest`
builds one from entries a caller already holds, and nothing builds one from a working tree. This
document is the design for that producer, settled **before** any of it is written.

It exists because the survey turned up two modeling questions the SCM brief does not answer, and
answering them unilaterally is how the previous PR acquired five rounds of findings. Both were put
to external review and ruled on; this records the ruling and the facts that forced it.

## 0. The two questions, and what the survey actually found

**Q1 — where does enumeration live?** `extdeps.filesystem.filesystem_io` returns a directory listing
whose `entries` member is a raw newline-joined `String`, with membership answered by a substring test
over that blob. It carries no file-versus-directory discriminator, so a corpus walk cannot ask the
one question it must ask at every entry: descend, or read.

Three facts qualify that, and the first two cut against widening:

- **No existing consumer enumerates.** Every one of the eight modules that consume the listing feeds
  it straight into `filesystem_entry_presence`, which answers *membership* — is this one name in this
  directory. Not one iterates the entries; not one needs `EntryKind`. ~~The blob is adequate for every
  consumer that exists.~~ **SUPERSEDED — see §10.** The blob is not adequate for its existing
  consumers: its producer silently narrows the population it returns, so the membership answers built
  on it are unsound on two paths. That existing consumers ask only membership does not save a
  population the producer already narrowed.
- ~~**So enumeration is a new capability, not a failing carrier.**~~ **SUPERSEDED — see §10.** It is
  both. Enumeration and `EntryKind` are new capability; the listing is *also* a failing carrier, and
  the failure is live on main. The repair therefore comes first and does not wait on the capability
  that discovered it.
- **But the blob is a workaround whose justification has lapsed.** `filesystem_io` justifies the
  substring test by saying newline-wrapping makes the test line-exact "without needing a split the
  interpreter does not offer". That is false today: `split` is a builtin string method — declared in
  the seed at `std_algebra` and dispatched by the v1 interpreter as `MethodCallSplit`, requiring no
  import — and `extdeps.languages.markdown` `md_split_blocks` calls `split(s:, delimiter:)` on a plain
  `String` now, with further splits in `extdeps.git.object_store` and `extdeps.systemd`.

The deciding argument is `filesystem_io`'s own. The comment above `filesystem_listing_names_entry`
pulls the membership predicate down beside the operation because two consumers had spelled it
independently over one wire format, "which is the second representation section 3 forbids: the
encoding is `List`'s fact, so the membership question over it is too." That argument does not weaken
when the question is enumeration instead of membership. Either the structure belongs at the
authority, or that comment is wrong about membership.

**Q2 — what does the walk descend on?** DESIGN §4 makes execution bounded and forward, with
termination *checked* rather than discovered. A directory tree offers no static bound and, with
symlinks, not even a guaranteed finite one.

## 1. RULING — root-widen the listing at its authority

Replace the lossy listing representation where it is produced and migrate its consumers together.
Names-only consumers project names from the richer result; they acquire no interest in ingestion,
MIME types, or semantic source validity. The unrelated read consumers do not migrate merely because
their operations share a module.

The migration population, corrected after the first census undercounted it:

| module | how it reaches the listing |
|---|---|
| `gunbc.generated_artifact_observation` | directly |
| `gunbc.deploy_transition` | directly |
| `gunbc.scm.init` | directly |
| `gunbc.roadmap.roadmap_verification_receipt` | directly |
| `gunbc.roadmap.roadmap_publication_helper` | directly |
| `gunbc.fleet.fleet_converge_plan_cli` | directly |
| `gunbc.devboot.build` | directly |
| `gunbc.roadmap.roadmap_belt_actuate` | directly |
| `gunbc.fabric.fabric_allocation_store_substrate` | only `FilesystemEstablishedAbsence` |
| `gunbc.scm.repository_save` | only `FilesystemEstablishedAbsence` |

The first census reported eight. It keyed on the listing's *type and constructor spellings*, so it
found every module that names the listing and missed the two that reach it through the derived
absence carrier. Those two move only if the widening changes the shape of `FilesystemEstablishedAbsence`
rather than the blob behind it, so they may be zero-diff — but they are in the subject population, and
a filter whose vocabulary is narrower than its subject reporting its own reach as the population is
the partial-observer class this lane keeps finding. It is recorded here rather than quietly fixed.

The cut is not eight call expressions: the listing folds, the witnesses, the transport realization
and the emitted representation belong to it too.

### Two properties must survive the replacement

**Enumeration failure stays distinct from successful exhaustion.** Opening a directory successfully
does not establish that enumerating it succeeded — a host iterator can fail while advancing. A result
accumulated before such an error must not become a complete listing, and so must not establish
absence for an entry it never reached. This is the absorbing-fallback rule (§5) at the listing
boundary: the failure arm refuses, it does not widen into "not there".

**Entry names and kind observations stay unconflated.** An entry's name can be known while observing
its kind fails. That entry is neither dropped nor called `Other`; the two facts are separately
carried, because the host interface itself separates a name from a fallible type observation.

And no newline-delimited membership may survive underneath the structured result as an independent
authority. `filesystem_listing_names_entry` is deleted by this cut, not kept beside it.

### A consequence the cut inherits: two of `FilesystemEntryName`'s refusals lose their reason

`admit_filesystem_entry_name` refuses a name containing `\n` and separately one containing `\r`, and
both refusals are justified **by the wire format** — the newline is `Filesystem.List`'s delimiter, so
such a name could make the membership test answer about a different spelling. Under a structured
listing that delimiter does not exist, and those two justifications lapse with it.

That does not automatically mean the refusals go. It means each must be re-decided on its own
grounds: either the structured representation preserves such a name losslessly and the refusal is
deleted, or the refusal stays as an explicit *unsupported spelling* with a new stated reason. What it
may not do is survive carrying a justification the cut has falsified — which would be this same class
a second time, created by the repair rather than found by it.

The `/`, `\` and NUL refusals are untouched: path traversal and C-string truncation are facts about
the host, not about the listing's encoding.

## 2. RULING — layering: acquisition, captured inputs, pure construction

```
bounded filesystem acquisition
          ↓
explicitly scoped captured inputs
          ↓
pure corpus construction / semantic ingestion
```

Build **both** layers. The already-enumerated interface is not eliminated by the root cut — it is
placed correctly. An explicit input list is the right subject for the pure operation, and it is *not*
evidence that anybody enumerated a directory successfully: a caller may intentionally select ten
files, which is a complete selection of those ten and not a complete capture of their parent.

The failure mode to refuse explicitly: using the lower layer to claim the upper producer has been
delivered.

## 3. RULING — finite work fuel is the descent measure

Admit a finite, nonnegative budget supplied by the acquisition policy. The walk carries `R(s)`,
remaining work fuel. Every continuing discovery transition establishes `R(s) > 0` and
`R(s') = R(s) - 1`.

That is a well-founded arithmetic descent even though discovering one directory adds several pending
children: **the worklist may grow; the rank does not.**

```
all acquisition obligations discharged      → completed capture
obligations remain, no work fuel remains    → located budget-exhaustion refusal
obligations remain, fuel positive           → one bounded transition, continue with less fuel
```

Prohibited: resetting the budget per sibling, replenishing it when a new directory appears, or
charging only newly discovered inodes while another continuation path runs unbounded. Each loop needs
its own demonstrable descent or must share the globally decreasing measure.

The refusal says *the admitted budget was exhausted before completion was established*. It does not
claim the corpus contains more than some number of files — that number was never observed. Reaching
the numeric limit after every obligation is discharged is not a reason to refuse.

**The budget is operational, not part of corpus selection.** Raising it must not change the identity
of a successfully captured unchanged corpus, and exhausting it must not mean "this now denotes the
prefix we happened to read" — that is the absorbing fallback wearing a limit's clothes.

### Why the three candidates I proposed are not this

| candidate | honest role | what it does not establish |
|---|---|---|
| remaining depth | can prove recursive descent when each child enumeration is a finite admitted input | any bound on total discovery work, directory width, or I/O |
| visited-object set | detects repeated expansion; supports traversal of a *fixed finite* graph | that the live universe of discoverable objects is fixed and finite |
| no symlink following | removes symlink-induced traversal into aliases and cycles | that remaining discovery terminates or fits a resource bound |

A visited set becomes a termination proof through a measure like `|U \ V|` only when `U` is an
established finite universe and each continuation removes an element from it. Maintaining `V` — what
has been seen — does not establish `U`. Directory contents can change during enumeration, and POSIX
leaves aspects of observing concurrent additions and removals unspecified.

A depth budget is legitimate as an *additional* policy limit with its own located refusal, and an
explicit worklist with work fuel also stops the host call stack from being the mechanism that
enforces it.

One qualification I had wrong: an unknown compile-time size is not the same as having no termination
argument. A materialized finite tree traverses structurally without a compile-time size, and a finite
list folds without a literal length. The problem here is specifically that a filesystem walk
**discovers more input through effects while it runs** — a child pathname is not a compiler-visible
subvalue of an already-held finite tree. `std.termination` already separates `TreeSize`,
`ListLength`, `ArithmeticValue` and `SetCardinality`; these are different arguments, not
interchangeable labels for "making progress".

### The fuel must reach the actual discovery mechanism

The boundary most likely to recreate the problem:

```
unbounded host enumeration → fully collected list → check the budget
```

A budget cannot bound work that already happened. The listing realization must either expose bounded
progression or perform a bounded collection internally, with explicit completion-versus-limit
outcomes. A new streaming framework is not required; a host-side bounded collector that **refuses
rather than truncates** satisfies the contract.

The same distinction applies to content: a bound on directory transitions does not bound a whole-file
read. Content acquisition is bounded separately, and I/O deadlines or cancellation stay in the effect
contract. An arithmetic rank bounds modeled continuations; it does not prove an arbitrary kernel or
remote-filesystem call returns within a wall clock.

And the decrement must be established by the compiler's actual enforcing path. `Strict` existing in
`std.termination` is not proof that the compiler derives it for this walk.

## 4. RULING — never follow a symlink implicitly

Do not follow file or directory symlinks during authored-corpus acquisition. Following a link
substitutes the referent's contents for the authored link, which changes the subject being captured.
A link to an external file, a link to a directory, and a dangling link must not acquire different
preservation semantics because one happens to resolve today.

Three facts stay separate:

```
the directory entry is a symlink
the symlink stores a particular target spelling
resolving that spelling currently produces some outcome
```

`std.filesystem.types.SymlinkTarget` is `TargetFile | TargetDir | Broken`. That records the *third*
fact only. It cannot serve as a persisted link payload, because it does not carry the stored target
spelling needed to reconstruct the link. A link-preserving representation needs the link's actual
stored target, read without substituting the referent's contents, and without accepting a truncated
target.

### What this producer does, given the manifest it actually has

`CorpusManifestEntry` is a path and an `AuthoredSourceTarget`, with no link discriminator. It cannot
express "this path denotes a symlink with this target". So for an in-scope symlink this producer
returns a **located unsupported-symlink acquisition refusal**. It must not:

- omit the link silently and report complete capture;
- store its target text as though it were an ordinary file;
- put `Broken` or a traversal error into the manifest as though it were authored content.

The refusal belongs to the acquisition outcome. The manifest's existing ability to represent a
*deliberate* partial corpus is not permission to label an *accidentally* incomplete acquisition
complete — that distinction is the whole of §5 at this boundary.

A regular-source-only producer with an explicit refusal is a valid narrower contract. It is not
full-directory support, and an unrestricted directory-freezing operation over a scope containing
links needs real link-preservation semantics — an explicit representation with its identity,
persistence and reconstruction behaviour — before it can claim that input.

### One existing helper is the wrong policy to reuse

`std.filesystem.is_text_readable` treats a symlink resolving to a text file as readable, and
`partition_entries` divides inputs into `readable` and `skipped`. That is a *readability* policy, not
an exhaustive authored-corpus capture policy. Consuming its `readable` population would both admit
link dereferences and discard other entries. Reuse the `EntryKind` vocabulary; do not inherit that
partition as the capture decision.

## 5. RULING — enforce no-follow at acquisition, not only at observation

This sequence is insufficient:

```
observe "regular file"  →  another actor replaces the entry with a symlink  →  read the pathname
```

The observation and the read then concern different objects; a prior `EntryKind` value does not
constrain subsequent pathname resolution.

The acquisition realization must enforce no-follow and root-resolution when it **opens** the object,
and inspect and read the object actually opened. On Linux `openat2` supplies resolution constraints
(`RESOLVE_NO_SYMLINKS`, `RESOLVE_BENEATH`); plain `O_NOFOLLOW` constrains only the final component,
not every component of a pathname. Those are realization facts modeled under their platform
authority, not assumptions attached to a generic string read.

This changes nothing for the other read consumers. It means the new corpus-acquisition path obtains
the capability it needs instead of assuming the generic `Read` already provides it — and where a
realization cannot supply it, the result is an explicit unsupported-capability refusal, never a
fallback to following the path.

## 6. RULING — completion must say what was completed

**"The walk exhausted its selected observations" is not "these files all existed together at one
instant."** A traversal over a mutable directory is not a snapshot; successful enumeration and
content reads do not supply that temporal guarantee, and POSIX's concurrent-enumeration semantics
rule out deriving it from ordinary listing.

This producer may freeze the exact admitted contents it captured, under an explicit selection and
observation contract. A point-in-time whole-tree claim additionally needs a snapshot or quiescence
mechanism. Snapshot infrastructure is not silently made a prerequisite for a smaller capture — and a
snapshot is not silently claimed either.

The completed-capture arm requires that every selected enumeration continuation reached successful
exhaustion, every selected entry was accounted for, and every required content acquisition completed.
Budget exhaustion, an inaccessible selected subtree, an unsupported in-scope entry, or an unresolved
acquisition failure must not reach that arm. Partial observations may remain for diagnostics; they do
not authorize publication as the completed capture.

## 7. The discriminating evidence

Witnesses target the distinctions above rather than demonstrating one successful walk.

| property | discriminating specimen |
|---|---|
| listing completeness | a directory yields one entry then fails: no complete listing, and no false absence for an unreached name |
| work descent | a synthetic provider keeps revealing new work: finite fuel refuses, with no replenishment and no successful prefix |
| budget boundary | a small capture completes; the same attempt with pending work at exhaustion refuses |
| symlink policy | file, directory, ancestor-cycle, external and dangling links cause **no referent acquisition** and receive the declared refusal |
| observation-to-open safety | an entry changes from regular file to symlink between observation and open: no implicit dereference |
| occurrence preservation | two selected paths naming the same underlying file: both paths stay represented |
| completion authority | an acquisition fails after earlier files succeeded: no completed-capture result and no publication under that claim |

Compiler-side evidence is separate: the intended loop is accepted with its real decreasing measure,
and the non-decreasing variant is **rejected by the path claimed to enforce termination** — because a
wall nothing red has ever hit is a decoration (§4b).

Inode deduplication must not erase path *occurrences*: object identity on Unix is device plus inode,
and several directory entries may name one file. The manifest needs both selected paths even when
captured content shares storage.

## 8. Sequencing

1. **The listing root cut.** Widen the listing at its authority, migrate the ten-module population,
   delete `filesystem_listing_names_entry`, re-decide the two `FilesystemEntryName` refusals whose
   justification the cut falsifies, and file the failure-mode row below. Self-contained, with real
   consumers today.
2. **Bounded acquisition.** Work fuel, the bounded collector that refuses rather than truncates,
   no-follow at open.
3. **The producer.** Captured inputs → `CorpusManifestObject`, with the completion contract of §6.
4. **`RepositoryCommit.corpus`**, then the verbs. `gunbc.scm.status`'s own header currently declares
   that it has no path and no working tree and calls those facts unproducible; the manifest is what
   makes a path-bearing status expressible, so that header is revised *with* the change, not around it.

## 9. The failure-mode row this survey owes

`a_workaround_outlives_the_constraint_that_justified_it` — a construction justified by a named absent
capability, where the capability now resolves and nothing re-reads the justification.

The three nearest rostered rows are each a different class. `unmarked_workaround` is a workaround with
*no* trigger; this one has a stated justification acting as an implicit one.
`trigger_satisfied_before_the_row_was_written` is a trigger already true *at filing time*; this is its
temporal sibling — true at filing, lifted later.
`a_live_authority_name_carries_a_superseded_claim` is a name refreshed while its claim died; here
nobody touched the sentence at all and its precondition moved underneath it.

Recognition rule, decidable: a comment justifying a construction by a named absent capability, where
the capability now resolves. Receipt: `filesystem_io`'s "a split the interpreter does not offer",
against `split` as a seed builtin dispatched as `MethodCallSplit`.

It lands with the cut in step 1, not as a bare roster append.

## 10. A live defect found while sizing the cut, and a correction to how I first reported it

The completeness property §1 requires is not merely unmodeled. It is **violated on main today**, in
the seed, where no `.dag` wall can see it. The file transport's `list` verb:

```rust
return match std::fs::read_dir(&path) {
    Ok(entries) => {
        let mut names: Vec<String> = entries
            .filter_map(|e| e.ok())
            .filter_map(|e| e.file_name().into_string().ok())
            .collect();
        ...
        Ok(FileResult { success: true, content: names.join("\n"), .. })
```

Two silent drops, both landing in a result whose `success` channel says `true`. **They are not
equally harmful, and my first report of them said they were.**

**The iterator drop is live.** `filter_map(|e| e.ok())` discards a `read_dir` iterator error
mid-enumeration and returns the prefix collected so far as a *complete* listing. It truncates the
list of ordinary, askable names, so `filesystem_entry_presence` finds no membership for an entry that
exists and constructs a `FilesystemEstablishedAbsence` for it. A wrong answer, with no diagnostic,
under every absence established in the tree — the absorbing fallback ("could not enumerate" widened
into "enumerated, nothing more") sitting underneath the module built to make absence honest.

**The non-UTF-8 drop is latent.** `into_string().ok()` discards any entry whose name is not valid
UTF-8, again into a `success: true` listing. But `filesystem_entry_presence` takes `name: String`, so
the asked name is always valid UTF-8 by construction and a dropped entry could never have matched it;
`admit_filesystem_entry_name` would refuse such a spelling independently. Since **no consumer
enumerates** — the fact §0 establishes — the dropped entry is currently unobservable.

It is still a success channel claiming a completeness it does not have, and it becomes load-bearing
the instant enumeration exists, which is this cut. But it is not a live wrong answer, and reporting it
as the second half of a live fail-open was the same defect this lane keeps finding, applied to my own
finding: asserting a population without checking whether the distinction reaches a consumer. The
correction is recorded rather than quietly edited, because a design document that silently agrees
with whatever was true last is not evidence of anything.

### A third defect, which review found and I had missed: fabricated presence

The two drops narrow the population. There is a third path that *widens* the answer, and it is the
exact dual.

While the newline wire stands, a returned name containing a line feed makes one entry
indistinguishable from two. These produce identical bytes:

```
one entry:    "prefix\nrepo.json"
two entries:  "prefix", "repo.json"
```

`filesystem_listing_names_entry` searches for a newline-delimited occurrence, so it answers
`repo.json` **present** in both cases. That is a fabricated presence, and admitting the *queried*
name does not protect against it — `admit_filesystem_entry_name` constrains what a caller may ask,
and the collision is in the name the directory *returned*.

So the repair has two directions to close, not one: a listing may not silently shrink, and it may not
silently claim an entry it does not hold. Both belong to 1A, because both are live under the current
encoding.

Carriage return is deliberately **not** given a matching refusal. It does not collide under the
exact-`\n` join and split, and minting a refusal for it here would be a restriction with no
demonstrated defect behind it — the inverse error of leaving one standing on a lapsed justification.
Its disposition belongs with the LF case to the cut that retires this encoding.

### What that implies for sequencing

The live repair is independent of enumeration, `EntryKind`, encodings and corpus manifests, and wants
to be reviewable by someone who does not have to hold this design in their head. The latent one has
no honest home except the cut that activates it. So step 1 of §8 splits:

- **1A — enumeration completeness.** The iterator error stops being discarded; the success channel
  stops claiming an exhaustion it did not reach; a partial enumeration can no longer establish an
  absence. Discriminating specimen: a directory that yields one entry and then fails.
- **1B — the structured listing.** `EntryKind`, lossless names, deletion of
  `filesystem_listing_names_entry`, and the re-decision of the two refusals whose justification the
  cut falsifies.

### The substrate constraint that shapes 1B

The effect boundary carries only flat scalars and lists of **string-alias** elements: every `List`
output in `extdeps` is `List<String>`, `List<FilePath>` or `List<CommitSha>`, all from
`stdout_lines`, and no operation anywhere returns a list of records. So `entries` cannot come back as
`List<{name, kind}>` directly. Three candidates:

| candidate | verdict |
|---|---|
| per-line encoding decoded in `filesystem_io` | viable; keeps a delimiter, so names stay restricted |
| two index-correlated parallel lists | **rejected here** — an unenforced index correlation is the conflation this module exists to remove |
| a JSON document on the `entries` channel, decoded once beside the operation | preferred |

The JSON branch is the only one that preserves a newline-bearing name losslessly, carries a per-entry
kind, *and* can carry an explicit enumeration-completed field — which is what makes the completeness
distinction **representable** rather than inferred from a success bool. Decoding it beside the
operation is the same move `filesystem_io` already makes for membership: the encoding is `List`'s
fact, so the structure over it is too.

## 11. What this cut does not claim

`filesystem_absence_establishment_adoption_standing` carries a next-rung trigger reading that the raw
`List` and `Read` result projections cease to be reachable outside this module's folds, so a consumer
cannot spell the conflation at all. **This cut does not satisfy that trigger** — the raw operations
stay callable — and must not be reported as retiring it. A trigger is retired by its trigger and by
nothing else.
