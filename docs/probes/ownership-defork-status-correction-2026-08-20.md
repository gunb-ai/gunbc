# Emitter ownership de-fork: a status correction, a population bound, and three instruments that could not answer

**This is not a defect report.** An earlier draft was, and
[`docs/plans/emitter-ownership-defork.md`](../plans/emitter-ownership-defork.md) answers its central
claim — **against it**. What survives is a status correction, one number, and a method note.

**Subject.** `src/v1/ownership.dag`, `src/v1/05_emit_rust.dag`, and the de-fork plan.
**Digests.** Subject `staging.dag` `812d95660a6dd909` in every arm; probe mirrors
`0548ebb56a2243ac` (subtractive), `2460edde31b1e574` (additive), baseline `432de5408dffa412`.
`ctrl-build` replays the worktree as patches, so a runner's `HEAD` names the *base* — digests are the
provenance, not SHAs.

---

## 1. What was NOT found — retracted in full

The draft claimed a defect: `make_decision` requires exactly one `Consumed` edge; `walk_expr` records
`Consumed` **only in tail position**; therefore any value in **argument position** is `Read`,
`sc == 0`, `Unclassified`, never in `movable`, and always cloned.

**The mechanism description is accurate. Calling it a defect is wrong.** The plan's increment-1
*Soundness* paragraph states the rule and its justification:

> `movable` gates **every** whole-value var-ref emission … not just tail returns, so a per-name move
> license is sound only if the binding has exactly **one** whole-value use site. `Consumed` is
> recorded only at tail position; `Read`/`Threaded` edges are whole-value positions emitted *before*
> it, so any such edge plus a licensed move = use-after-move. Hence the rule: SoleOwner **and** zero
> Read/Threaded edges.

So tail-only `Consumed` is **the wall that makes per-name licensing sound at all**, and the plan
records that an earlier draft admitting Read-then-Consume was a latent use-after-move, since fixed.

**The probe run contained the proof of this and it was misread.** Forcing `build_movable_set` to
admit everything emitted `resolve_probe(lookup(x), stage, x)` — `x` moved **twice**. That is not an
incidental artifact of a maximal arm; it is precisely the failure mode the wall exists to prevent,
observed and filed as a curiosity.

**Also already documented, in `src/v1/trait_bound_witness.dag`'s own note** — that the Clone-bound
chain is blind to function bodies because `emit_fn_def` calls
`v1_generic_params_needing_clone_bound` with params/ret/bounds but **never with the body** — and
already censused: `dag/tools/e0599_emitter_decision_census.dag`, cause `CloneSharedRequirement`,
lowering `DerefCloneWholeValue`, **139 counted occurrences**, with the carrier question open among
three named candidates including that plan's `UseSiteVerdict.CloneShared`.

## 2. STATUS CORRECTION — the designed recovery is absent from the tree

The plan's recovery for exactly these sites is per-name → **per-site** licensing (Perceus-style
last-use), increment 2, whose status line reads *"proof-side machinery AND whole-value emitter
consumption are implemented"*.

**Measured in this tree — zero, corpus-wide across all `.dag` and `.rs`:**

| symbol | hits | | positive control (same method, same paths) | hits |
|---|---|---|---|---|
| `build_move_site_licenses` | **0** | | `build_movable_set` | 2 |
| `move_sites_index` | **0** | | `make_decision` | 1 |
| `move_licensed_at_site` | **0** | | `build_read_only_params` | 2 |
| `take_owned_counted` | **0** | | `owned_bindings` | 3 |
| | | | `FoldAccUnwrapProof` | 1 |

**It was BUILT, then deliberately DELETED — not never built.** `git log -S move_sites_index --all`:

```
3a1b87d6e0  2026-07-06  #6307  "Persistent value carriers: lists+sets -> im_rc;
                                delete clone-fallback guard + move licenses (atomic)"

move_sites_index          before 3 files → after 0
build_move_site_licenses  before 2       → after 0
move_licensed_at_site     before 1       → after 0
```

**The distinction is load-bearing.** *"Absent while the status line claims it landed"* reads as
*someone forgot to build it*, and the next person picks it up as greenfield work. *"Built, then
deleted atomically with the im_rc carrier migration"* says two things they need: **the
implementation exists in history at `3a1b87d6e08^`** — a backward liveness walk in reverse
evaluation order, span-keyed, with the guards the plan describes, so nobody has to design it again —
**and it was removed for a reason that may still hold.**

**The doc and the code were never coupled:** the plan entered main via **#6249**; **#6248 — the PR
the plan says increment 1 is "in this PR" — is CLOSED, not merged**; and then a third PR removed the
implementation. Increment 1's *core* did land by some other route (`build_movable_set` today filters
on `make_decision(usage) == SoleOwner` and admits params via `param_names`, as described).

> **So the plan is not merely stale. It describes as WIRED a mechanism a later merged PR
> deliberately removed, and has said so in main since 2026-07-05** — worse than an un-updated plan,
> because a reader who greps the symbols finds nothing and concludes the plan was aspirational, when
> it is a description of code that ran and was withdrawn.

### What was deleted — both designs derived here, already written

`git show 3a1b87d6e08 -- src/v1/ownership.dag`: **150 deletions, ZERO additions, one file.** Not a
rewrite, not a migration to another module — pure excision, with no follow-up in the six weeks since.

```
type LiveState { name, whole: Bool, any_field: Bool, fields: Set<String> }
type MoveLicenseAccum { live: Map<String, LiveState>, licensed: … }
fn move_site_key(name: String, span_start: Int) -> String
fn record_whole_use_site(acc, name, span_start, in_capture: Bool) -> MoveLicenseAccum
  let movable_here = !in_capture && span_start != 0 && !st.whole && !st.any_field
fn record_field_use_site(acc, name, field, span_start, in_capture: Bool) -> MoveLicenseAccum
```

- **`move_site_key(name, span_start)` is span-keyed occurrence identity** — the "construction fix"
  for the flat name-keyed bindings map, proposed during this investigation as a wider-better cleanup
  nobody had done. It had been written and deleted six weeks earlier.
- **`in_capture` is a parameter of both recorders** — the lambda-binder-vs-free-variable distinction
  also derived here, already threaded through, with captures poisoning licensing exactly as the
  plan's guard paragraph describes.
- **`span_start != 0`** is the fail-closed synthetic-node guard the plan names.

**Both designs independently derived during this investigation were in those 150 lines.**

### Why it was deleted — OPEN, three readings

**The commit states what it deleted, not why.** Its bullet is *"Persistent carriers: lists+sets ->
im_rc Vector/OrdSet; delete clone-fallback guard + per-site move licenses"*; the body's rationale
paragraphs are all about the clone-fallback guard, none about licensing.

1. **im_rc made licensing unnecessary.** The guard and the licensing both existed to avoid expensive
   clones — the guard by refusing them, the licensing by proving them unneeded. If persistent
   carriers made clones cheap, both lose their motivation together, which would explain an atomic
   deletion the commit felt no need to justify. *Consistent with everything read here; not confirmed.*
2. **im_rc made licensing unsound** — if the carrier change altered what a whole-value use site means.
3. **The two could not be landed together** — which the word *"atomic"* in the title equally
   supports. **Weakened by the diff:** a pure deletion with no replacement and no re-add attempt in
   six weeks is not what a deferred landing looks like.

**Evidence against the comfortable reading (1):** a *reasoned* obsolescence would plausibly have
updated the plan in the same pass. It did not — the plan still says "wired" six weeks on. That is a
pattern argument, not proof, and it points away from *deliberate and reasoned* toward *expedient
during a large migration, never revisited*. **The absence of a stated reason is itself evidence
about the kind of decision it was.**

**Which one holds decides whether re-implementing it is correct or reintroduces a bug** — and the
150 lines are recoverable, so the question is worth answering before anyone rewrites them.

## 3. POPULATION BOUND — at most 446 of 775 clone sites

Same subject, 14 emitted files in both arms; baseline guarded UNPROBED (probe markers `0`,
`SoleOwner` gate `1`, `lambda-capture` `1`).

```
BASELINE total clone() across the emit:  775
FORCED-MOVABLE comparator:               329
DELTA:                                   446        BASELINE total Clone bounds: 93
```

> **AT MOST 446 of 775 clone sites in this emit exist because a value could not be proven movable.**

**The qualifier is part of the sentence**, for two compounding reasons: 329 is a floor **no correct
repair can reach** (that arm emits unsound Rust — §1), and this counts clone **sites**, not `movable`
membership, so it bounds the *consequence* rather than observing the *cause*. **Do not restate it as
"58% of clones are spurious."**

**It is a different quantity from the census's 139**, which counts E0599-causing lowering sites.

> **IT IS ALSO NOT A COST MEASURE, and an earlier draft of this document wrongly made it one.** That
> draft paired the 446 with the plan's cost denomination — whole-tree `compile --target dag` at
> ~72 min, ~85–90% emit, `Rc::make_mut` copy-on-writing whole containers, O(n²). **That denomination
> is PRE-`im_rc`.** The live carriers are persistent —
> `use im::{vector as vec, HashMap, OrdSet as BTreeSet, Vector as Vec}` — on which a clone is a
> **refcount bump with structural sharing**, not a deep copy. Joining the two silently re-inflates
> cheap clones into expensive ones. The 446 bounds clone **sites**; it says nothing about what they
> cost.
>
> **And do not over-correct into "clones are cheap now" either.** That holds for the **persistent
> carriers specifically** — `im` `Vector`/`OrdSet`/`HashMap`. A clone of a `String`, a record, a
> tuple, or any non-persistent `T` is still a real copy, and **nothing in the 446 distinguishes
> carrier clones from value clones.** The post-`im_rc` cost of these sites has **never been
> measured**, in either direction.
>
> **Two partitions would answer two different questions, and they are not the same query:** by
> **cloned type** (persistent carrier vs value) answers *cost*; by **single-use vs multi-use**
> answers *recoverability*.

**What it does not settle:** whether `movable` is **inert** (near-empty — coverage by illusion) or
merely **partial**. The delta measures the consequence and cannot separate them; that needs set
sizes and its own evidence.

## 4. Three instruments that could not answer

Nothing in the plan contains this, and unlike the mechanism it is not re-derivable from the code.
**Two of the three looked like results.**

**(a) The probe never compiled.** `src/v1/ownership.dag` has a **stage0 mirror**;
`cargo build -p v1-compiler` compiles `src/v1/stage0/src/v1_compiler_ownership.rs`, **not the
`.dag`**. Detected with one grep, no build: probe markers `0` in the mirror, `lambda-capture` `1` as
the positive control. Every provenance column was *true* and none answered the question — the `.dag`
digest changed (the edit arrived *on disk*), `Checking patch src/v1/ownership.dag` appeared in the
runner log (applied to a file nothing reads), and the rebuild was genuine (*of the unchanged
mirror*). **The guard asserted the probe was present in the SOURCE TREE; for a self-hosted compiler,
*present in the tree* and *present in the executed path* are different questions.**

**(b) A subtractive probe cannot discriminate against an inclusion set.** `build_movable_set` folds
qualifying names **in**, so removing the capture collapse removes entries *entirely* — and absence
yields the same emitted clone as exclusion. Even with the collapse verifiably gone from the executed
path (`lambda-capture` = 0 in the mirror, guard passed), the null was unreadable. **A control is not
made decisive by being maximal.** The decisive direction was additive.

**(c) Reasoning about signatures while treating bodies as illustration.** A draft made `cached_stage`
a zero-diagnostic/two-defect specimen. Counting the uses of `x` — `lookup(x.clone())` *and*
`x.clone()`, **twice** — shows its `A: Clone` is **required**. The discriminator was in a two-line
body that had been quoted repeatedly and never read.

**The guards that replace (a) and (b):** assert on the **compiled** artifact in *both* directions —
the probe marker present **and the construct being removed absent** — plus a positive control so a
zero is readable, and a **sensitivity control** (total `clone()` across the whole emit) so "nothing
changed" is distinguishable from "the probe did nothing".

**A corpus finding, not a confession — the repository's most complete analyses are stored where
nothing reads them.** The `data …_note: String` rows at the top of `.dag` modules run to one–three
thousand words, are written once at peak context, and are surfaced by nothing: no lens reads them,
no gate checks them, and reading the module's *code* does not show them. DESIGN §4c requires prose
to be classified rather than loose; **it says nothing about whether classified prose is ever read.**
Both of the rediscoveries here were of text sitting in exactly that position — one in a plan, one in
the note of a file under the author's own sole-write ownership.

**And the meta-failure this document is itself an instance of:** four separate findings here were
rediscoveries of things already written down — in the plan, and in the note at the top of a file
under the author's own sole-write ownership, whose functions were quoted repeatedly without its note
being read. **Read the note on the file you are about to claim a finding about.**

## 5. Genuinely open, and deliberately small

- **`resolve_probe`'s `x`**: an ordinary function parameter, used **once**, in argument position,
  cloned. Under §1's wall this is expected — one `Read`, no `Consumed`. Whether per-site licensing
  (§2, unlanded) would recover it is exactly the class increment 2 exists for. No lambda is
  involved, which makes it the cleanest specimen for that increment.
- **`movable` / `read_only_params` do not partition the space.** `sc == 0` bars the single-use
  argument from the first; `binding_fan_out > 1` bars it from the second (plus `is_owned_local`,
  whose `effective_kind` is first-non-`none`-writer-wins over a seed of `none` — an order
  dependence). A param used **twice by reference** qualifies as read-only; used **once by value** it
  is cloned.
- **The `ExprLambda`/`ExprForEach` capture collapse** re-records every binding used in the body —
  including the lambda's **own binders** — as a capture `Read` at the *enclosing* scope, with nothing
  subtracting binder names, though `lambda_param_names_at` is imported in that very file. Under §1
  this is **not** the cause of the clones investigated here, and the plan's increment-2 note says
  lambda bodies "license nothing and poison their captures live" by design. Recorded as an
  observation, **not** as a defect claim.

---

**Ownership.** `src/v1/ownership.dag` has no sole-write declaration in `dag/gunbc`, and git
attribution names no current owner (last human commit 2026-07-06). The live authority for this area
is the de-fork plan and its lane (`node://adhoc-0717d295-672`), **not this document** — which exists
to correct that plan's status line, contribute one bounded number, and record the instruments.
