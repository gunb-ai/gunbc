# RETRACTED IN PART, 2026-08-24 — the auto-discovery half is FALSE. Read this first.

**The required floor does not consult `reads_live_tree_effective` at all, and an undeclared module
EXECUTES rather than being declined.** The mechanism this brief was written about gates *deferred
discovery*, not the required floor. Traced by `deep-ant-102`, verified here link by link:

```
run_required_floor        : let files = &prepared.witness_files;
                            if file.reads_live_tree { live_declined += 1; DeclinedLiveTree }
InventoryWitnessFile      : .reads_live_tree is set in witness_file_from_source by
                            content.lines().any(|line| line.starts_with("data ")
                                && line.contains("LiveTreeDisposition")
                                && line.contains("ReadsLiveTree"))
reads_live_tree_effective : both call sites are inside collect_deferred_discovery_rows
```

The floor's scan is an `any` for a **positive** `ReadsLiveTree` declaration. No declaration is no
match is `false` is **not declined**.

**What is therefore withdrawn:**

- *An undeclared module is silently declined.* **False for the floor.** It executes.
- *The 830 in `DeclinedLiveTree` are defaulted.* **False.** They are files that positively declare
  `ReadsLiveTree`, declined per their own declaration — correct behaviour, not a default.
- *gunbc#9058's evidence does not execute because it declares nothing.* **False, and I corrected
  that PR wrongly.** Its rows execute. (gunbc#9075 is unaffected: that file declares
  `ReadsLiveTree` positively, so it is genuinely declined, as DESIGN §4b independently records.)

**Confirmed by measurement, not only by reading, and by an instrument neither author controlled.**
`vivid-boar-345` had already established the same fact from two directions before the retraction
reached them, and the two prove *different* things — which is worth keeping separate rather than
merging into one confirmation:

- **The roster partition is the enrolment proof.** Required-floor on their branch at `6d29d409fab`
  reports `offered=12250 routed=10796 declined_long=546 declined_fixture=9 declined_live=899`,
  with `planned=executed=terminal=10796`. Across their commits **`routed` rose in lockstep with the
  rows they added and `declined_live` did not move.** A declined module lands in `declined_live`
  and leaves `routed` flat. This is the half that speaks to CI.
- **The flipping RED arm is the discrimination proof.** On a binary built *before* each change,
  exactly one row failed while its twin passed. *A row that is not executing cannot fail.* This is
  the half that speaks to whether the assertion says anything.

Neither substitutes for the other: the partition shows the rows are routed but not that they
discriminate; the flip shows they discriminate on the instrument it ran on, which is not by itself
a statement about CI. Together they close it. Recorded at that grain deliberately — collapsing them
into "measured, confirmed" is the same move this document spent the night objecting to.

**And the in-repo authority we dismissed was correct.** The `quarantine_probe` note says the floor
consults the syntactic column-zero scan and does **not** consult `reads_live_tree_effective`. Both
of us read it as stale. It was precise; our reading was stale. The call-site grep that misled us
found the two call sites and never asked which function contained them — a positional read
answering a narrower question than we put to it, which is the class this whole document is about,
committed twice by its authors while writing it.

**What survives, and is now sharper than what was retracted** — see *the escape route* below: the
U3 explicit-enrolment path DOES read the destroyed bool. So for one and the same undeclared file,
**auto-discovery executes it and deliberate enrolment declines it.** The two routes disagree, and
the deliberate one is the one that fails. That is a worse defect than the one this brief set out to
describe, and it is the reason the document is corrected rather than deleted.

The carrier finding stands for its real consumers (deferred discovery, U3 expansion). The
carrier-versus-arm split stands untouched — neither the floor scanner nor the bare-name closure
member depended on any of this.

---

# THE ORIGINAL ACCOUNT (2026-08-24), RETAINED AS RETROSPECTIVE EVIDENCE — its auto-discovery
# half is FALSE and its proposed repair is SUPERSEDED

> Everything from here to *The actual root* below is the account as first written. It is kept
> because the reasoning is the evidence for how the error was made, **not** because any of it
> is actionable. Nothing in this span is a present-tense claim about the tree and nothing in it
> is a recommendation. The live findings are *The actual root: a two-valued carrier over a
> three-valued domain* and *The class splits in two, and the split decides the repair*.

## The floor's live-tree default renders ignorance as a claim (as originally described)

Found 2026-08-24 while reviewing two unrelated child PRs that had each, independently and
carefully, authored discriminating evidence into a population CI never executes.

## The mechanism

`v2.workflow.floor_discovery_producer` resolves a module's live-tree disposition and ends:

```
match folded.refusal {
  Present { value: reason } => EntryLiveTreeDispositionRefused { reason: reason }
  Absent => match folded.disposition {
    Present { value: disposition } => EntryLiveTreeResolved { disposition: disposition }
    Absent => EntryLiveTreeResolved { disposition: ReadsLiveTree }
  }
}
```

`v2.workflow.required_floor` then declines every `ReadsLiveTree` module into `DeclinedLiveTree`
before the fold sees it. On main's last measured run that arm holds **830** identities.

So a module that declares **nothing** is treated as one that declared `ReadsLiveTree`, and its
witnesses are discovered, counted, and never run.

## Why this is a defect and not a default

*No declaration* is **ignorance**. `ReadsLiveTree` is a **positive claim about behaviour**. The
fold renders the first as the second and attaches a decline to it — so a module whose author
never considered the question is routed identically to one whose author considered it and
answered yes, and **no downstream reader can tell the two apart**. `declined_live` is a single
number over both populations.

That is ⊥-as-ignorance conflated with ⊥-as-answer, at the policy layer rather than in an
observation: the empty-observation narrow the repository already names, applied to a routing
decision. And the direction is the harmful one. Forgetting to declare costs a **silence**, never
a red — the author sees a green PR, the reviewer sees a witness file, the floor summary shows a
count that looks like deliberate quarantine, and nothing anywhere says *this evidence does not
run*.

## Measured, on two PRs in one hour, both by careful authors

- **gunbc#9058** — a sealed brace/equals probe pair, genuinely discriminating (pre-fix exactly
  one of the enrolled rows flips, measured locally by its author). Its module declares **no**
  disposition, so it defaults to declined. The author did not omit it carelessly; the question
  never surfaced, because nothing asks it.
- **gunbc#9075** — a cross-module forge probe keyed on the diagnostic class, with a positive
  control and a `CensusNotRunnable => -1` arm so neither side can pass vacuously. Added to
  `sole_constructor_completeness_audit_probe.dag`, which declares `ReadsLiveTree` honestly. It
  becomes identities 27 and 28 of a file DESIGN §4b already records as never executing.

Two independent lanes, same day, both producing exactly the evidence review asked for, both
landing it where nothing runs it. That is not two authors being careless; it is a default doing
what defaults do.

## What the repair is NOT

**Relabelling.** Several modules carry standing notes forbidding it in terms — `Do NOT dissolve
it by relabelling this file SubstrateInputsOnly` — because editing a declaration to buy admission
is §5's tell of a check satisfied by editing its declaration rather than its subject. The
disposition on those files is a true statement about what they do. The trap here is that the
cheapest fix for each individual file is the forbidden one, which is why the default has to
change rather than the files.

## The repair this brief ORIGINALLY proposed — SUPERSEDED, recorded, not recommended

**Do not apply this.** The live repair is *The actual root* below: return the three-state value and
let each consumer decide. What follows is what was proposed before the carrier was understood, kept
so the reasoning that produced a too-narrow fix stays visible.

The original argument ran: the refusal arm already exists (`EntryLiveTreeDispositionRefused`), so an
undeclared disposition was said to belong in it. Then a new witness module refuses at discovery — loudly, located, once — and the
author declares the honest answer in the same minute they wrote the file, instead of discovering
a year later that their wall was never measured. The alternative (default to the executing arm)
is worse: it would silently admit modules that genuinely read the live tree.

That was described at the time as the §5 construction move at the routing layer. It is not — it
makes *undeclared* refuse at one consumer while the other consumer still needs the third state.
Routing a genuine third value into a refusal is the *arm* fix applied to a *carrier* defect, which
is exactly the distinction the split below turns out to decide.

## Standing

Reported to both authors with the correction that their evidence is **authored, not executing**,
and that the rung must be cited that way. Not dispatched as a lane: new child work is held while
the merge queue is unserved, and this is a small, decidable change to one arm rather than an
investigation. Recorded here so it is not rediscovered by the third author to hit it.

Related and separately tracked: the deletion of the `DeclinedLiveTree` arm itself is already a
named next-rung trigger in several module notes (`observation_emit_census_witness_test`,
`seed_mirror_constant_lens_witness_test`, `guarantee_floor_class_probe_witness_test`), measured
on a branch as admitting ~783 identities of which 55 are blockers. **This brief is not that
change** — it is the much smaller one that stops *new* evidence entering the population by
accident while that larger deletion is staged.

---

## The reviewer's failure mode this exposed: an unsatisfiable ask produces a false declaration

Recorded the same day, because it is the more transferable half and it is mine.

Having found that gunbc#9075's probe landed in a declined file, I told its author the available
move was to *put these two rows in a module whose honest disposition is hermetic, if that is what
they are* — and, correctly, that relabelling was forbidden.

**That move does not exist.** Any probe built on `compile_dag_diagnostic_census` resolves its
synthetic sources against the live checkout; the repository states this in terms
(`guarantee_floor_class_probe_witness_test`). A census probe cannot honestly be
`SubstrateInputsOnly`. So the instruction was: *produce executing evidence, by a route that is
closed, and do not take the open route.*

The author relabelled the file — and argued the label, on the ground that the rows *"compile
supplied source strings only."* They do not: both source strings carry
`import std.primitive_projection { PrimitiveIdentity }`, and resolving that import is the entire
mechanism by which a forge probe can violate a seal declared in a real module.

**The shape worth keeping is not "the author was wrong."** It is that an impossible instruction
has exactly one compliant response, and that response is to make the false half true on paper.
The reviewer asked for a property the system cannot supply; the only way to satisfy the reviewer
was to declare that it had been supplied. **A demand for evidence, made where evidence is
structurally unavailable, manufactures a false declaration** — and it does so *reliably*, from a
careful author, in minutes. It is the §5 declaration-editing tell, with the reviewer as the
proximate cause rather than the author.

Two things follow for anyone reviewing at this bar:

- **Before demanding evidence, establish that the evidence is authorable.** This is the same
  question §4b asks about a check's RED, turned on the reviewer: if no artifact could satisfy the
  ask, the ask is not a high bar, it is a trap. And the correction costs more than the original
  gap, because it must now undo a change someone made in good faith.
- **"Authored, not executing" is an available and honest answer, and it must be offered
  explicitly.** The reason the author did not choose it is that I framed the options as *relabel
  (forbidden)* or *relocate (impossible)* and never named the third. A weaker true claim is
  always on the menu; if the reviewer does not put it there, the author will reach for a stronger
  false one.

---

## The unifying class: three defaults, none of which can say "I do not know"

Converged on 2026-08-24 from three instruments found independently, in one night, by three
sessions. Recorded together because each was diagnosed as a local defect and the third arrival
is what makes it a class.

| instrument | the ignorance | rendered as | cost of forgetting |
|---|---|---|---|
| floor live-tree disposition | module declares nothing | `ReadsLiveTree` — a positive claim about behaviour | silently declined; evidence never runs |
| floor primary scanner | `test data` declaration, or any non-column-zero form | not a witness at all | 13 files contribute no roster row; one identity historically verdict-bearing, now undiscovered |
| bare-name closure | scoped resolve fails | the whole corpus is affected | silent, uncounted, corpus-denominated |

**They substitute three different things, and that is why only one of them looks like a bug.**
Refinement contributed by `deep-ant-102`, and it is the sharpest cut in this section:

- the disposition default substitutes a **plausible specific claim** (`ReadsLiveTree`);
- the primary scanner substitutes **non-existence** (there is no witness here);
- the closure fallback substitutes **the universe** (everything is affected).

Same family, three different lies. The **specific-claim** one is the hardest to see, *because it
looks like data rather than like a default* — a reader inspecting a module's routing finds a
disposition, and a disposition is exactly the kind of thing that is supposed to be there. The
universe-substituting one at least announces itself as expensive; the non-existence one at least
leaves a gap someone may notice. A fabricated specific answer leaves nothing anomalous to notice
at all.

**Each default answers a question it was not asked.** *No declaration* is not `ReadsLiveTree`.
*Unrecognised syntax* is not *not-a-witness*. *Resolve failed* is not *everything is affected*.
In every case a state meaning **I could not determine this** is written into a slot typed for
**the determination**, and every downstream reader is then structurally unable to recover the
difference.

**And in all three the direction is the same one.** The failure arms are silent, so the deficit's
frequency is zero by construction and never ranks for fixing — §5's absorbing fallback and the
empty-observation narrow are the *runtime* members of this family, and these three are its
*routing* and *discovery* members. The repository already names the runtime shape; it did not
name the shape at the boundaries where facts enter the system.

The tell is uniform and cheap to apply: **look at the default arm of any resolution, and ask
whether its value is a determination or a shrug wearing a determination's clothes.** If the
answer for "I could not tell" is the same symbol as the answer for a real case, the two are
already conflated, whether or not anything has fallen in yet.

The repair was described here as uniform too, and not a wider default: a **refusal** in all three
instruments. **That generalisation is superseded and is recorded rather than recommended.** It is
right that a boundary which cannot express *I do not know* needs that state explicitly; it does not
follow that every such boundary wants a refusal. Whether the third state belongs in a refusal or in
the carrier is decided per instrument by the question in *The class splits in two* below — **does the
third state reach the consumer?** — and for this brief's own subject the answer turned out to be the
carrier, not the arm. The uniformity claimed in this paragraph is the error the split corrects.

### The authoring-time member of the same family

`smart-ram-730`, reviewing, and `deep-ant-102`, instructing a peer, each produced the same shape
from the other side in the same night: an instruction that admitted no true compliant response.
Mine demanded evidence by a route that was closed; theirs handed a peer a discriminator whose YES
arm fired on healthy behaviour, and the peer ran it and reported a class that was not there.

**An instruction from a senior party gets executed, not questioned.** So the compliant response to
an impossible or miscalibrated instruction is to produce the artifact that satisfies it — which
means the instruction, not the author, is the proximate cause of the false result. That is the
authoring-time twin of the absorbing fallback: the reviewer's demand is the arm that widens, and
the false declaration is what it fabricates.

Both of us recorded it about ourselves rather than about the person who executed it, which is the
only framing under which it is useful.

---

## A note on the grade of this brief's own evidence

The finding above is evidenced by **measuring the population it describes**: reading
`floor_discovery_producer`, then observing that 830 identities sit in `DeclinedLiveTree` and that
two PRs landed evidence there. That is a census of the tree, and DESIGN §5's oracle rule is
explicit that a measurement taken from the same current tree is the weaker kind — it counts what
is there, which is not the same as predicting what the mechanism does.

`deep-ant-102` is supplying the stronger kind, and it arose from a mistake rather than a plan:
they had already dispatched a five-arm floor probe whose fixture modules declare **nothing**.
Rather than delete and redo it, they are reading it as a **controlled fixture** for this brief —
five identities they authored, whose count they know exactly, differenced against `declined_live`.

**If that number moves by exactly five, the default is confirmed by an independently authored
population rather than by a count copied off the tree.** If it does not move by five, this brief
is wrong or incomplete, and they have committed to reporting the number either way.

Recorded here *before* the result, so that the brief cannot later be read as having been
confirmed by evidence it did not yet have. The distinction is the same one §5 draws between a
fixture that plants a known input and a literal transcribed from the current tree, and it applies
to a finding about defaults exactly as much as to a merge-blocking test.

---

## Challenged, verified in the seed, and sharpened: the intent was fail-closed on the OTHER axis

`deep-ant-102` challenged this brief's central claim before their probe returned, on a code read
rather than a measurement, and the challenge was substantial enough to have retracted the
diagnosis. Recorded with the resolution because the near-miss is instructive.

**Their reading.** The `.dag` producer's default (`Absent => ReadsLiveTree`) is not what the
executing floor consults. In the seed, `DeclinedLiveTree` is constructed at one place, gated on a
field set by `reads_live_tree_effective`:

```rust
let declared = parse_entry_live_tree_disposition(entry_path, content)?;
if declared { return Ok(true); }
Ok(effect_reach_derived_reads_live_tree_for_entry(entry_path, facts))
```

They read this as: undeclared falls through to an **effect-reach derivation over the import
closure** — a determination, not a shrug — which would make this case not a member of the class
at all, and would additionally make declaring `SubstrateInputsOnly` inert.

**The call chain is exactly as they traced it. The last line of the callee reverses it:**

```rust
// Undeclared = ReadsLiveTree: a row must DECLARE it does not read the live
// tree to become selection-eligible (fail-closed).
Ok(declared.unwrap_or(true))
```

Undeclared returns **true**, so `reads_live_tree_effective` takes the early return and the
effect-reach derivation is never reached:

```
undeclared           -> true  -> short-circuit -> DECLINED
ReadsLiveTree        -> true  -> short-circuit -> DECLINED
SubstrateInputsOnly  -> false -> effect-reach derivation decides
```

**Two things this settles, one of which corrects the brief above rather than confirming it.**

*The diagnosis stands, and the seed states the intent explicitly.* `a row must DECLARE it does
not read the live tree to become selection-eligible (fail-closed)`. The author was reasoning about
**affected-set selection** — do not let a module become selection-eligible unless it has affirmed
it is safe to. On that axis the default is genuinely fail-closed and correct. The defect is that
the same bit is consumed by a **second** consumer, floor execution, where its meaning inverts:
fail-closed on selection is **fail-open on evidence coverage**, because the conservative answer
there is *do not run it*. One bit, two consumers, opposite safe directions — and only the first
was named. That is a sharper statement than "the default renders ignorance as a claim," and it is
the one to carry.

*Declaring `SubstrateInputsOnly` is not a formality.* It is the **only route to the derivation at
all**; undeclared short-circuits past it. So the earlier advice — declare explicitly — is right for
a reason neither party had: not because the label admits a module, but because it is the sole path
to the code that can.

**Method note, which is the transferable part.** The challenge was raised *before* the probe
returned, specifically so the pre-registered falsification condition could be sharpened rather
than reinterpreted afterwards. Under the corrected reading the two arms predict a **difference**
(undeclared → all five declined; declared → derivation decides), where the challenger's reading
predicted identity. Had the number arrived first, a no-move would have been read as confirming the
effect-reach hypothesis, when under the corrected control flow that path is unreachable for
undeclared modules and a no-move would falsify *both* diagnoses. Getting the prediction right
before the measurement is what keeps a correct number from producing a wrong conclusion.

**Still open, and not resolved by any of this:** three in-repo notes describe two mechanisms and
disagree about which one the floor consults — one asserts a column-zero scan in which undeclared
EXECUTES and that the floor does not consult `reads_live_tree_effective`, against the call sites
above. At least one is stale. A stale authority about which mechanism gates the floor is the
premise-contamination class this repository keeps correcting itself for, and it is worth its own
look independent of how the fixture count comes out.

---

## The actual root: a two-valued carrier over a three-valued domain

The sections above argue about what the *default* should be. That framing is wrong, and the
reason surfaced from `deep-ant-102`'s account of how they misread the code rather than from the
code itself.

They read `let declared = parse_entry_live_tree_disposition(...)` and took the local's name at
face value — *declared*, i.e. **was a disposition declared**. The bool does not mean that. It
means **does this entry read the live tree**, defaulting to `true`. So the name answers a
different question than the value, and the default that reverses the whole mechanism sits two
files from where the branch is written.

Their rule from it, which generalises past this case: **when a local is bound from a call and
immediately branched on, the default inside that call IS the branch's semantics.** Inferring a
callee's semantics from the caller's local name is the same shape as every other trap recorded
this night — a name answering a narrower question than the reader asks of it.

**But the misreading is a symptom, and the thing it is a symptom of is the root.** The domain has
**three** states:

```
undeclared            (the author said nothing)
ReadsLiveTree         (the author said yes)
SubstrateInputsOnly   (the author said no)
```

and `parse_entry_live_tree_disposition` returns `Result<bool, String>` — **two**. The collapse
happens *inside the function*, at `declared.unwrap_or(true)`, which is precisely where the
`Option<bool>` that had all three states is discarded. Note the local is already `Option<bool>`
one line earlier: the distinction exists, is correctly represented, and is thrown away on the
return.

That reframes the whole finding. The defect is **not** that someone picked the wrong default; it
is that the carrier crossing the boundary cannot express the third state, so *every* consumer is
forced to inherit one function's guess about which of two answers to fabricate. Two consumers want
opposite things from it — selection wants unknown treated as *reads*, execution wants unknown
treated as *decide properly* — and neither can have it, because by the time they see the value the
question they needed answered is gone.

**There are THREE production consumers, and the third is the worst one.** Enumerated by
`deep-ant-102` rather than assumed:

| consumer | via | what it decides |
|---|---|---|
| affected-set selection | `reads_live_tree_effective` | is this entry selection-eligible |
| floor execution | `reads_live_tree_effective` | does this entry run at all |
| **U3 explicit-entry expansion** | `read_entry_live_tree_disposition` | `DiscoveryRow.reads_live_tree` |

The third is **the escape route**. `read_entry_live_tree_disposition` is a thin file-reading
wrapper with exactly one production caller: the file-grain explicit-enrolment path — what an
author reaches for precisely *when discovery has missed their witness*. It reads the same
destroyed bool, so **an explicitly enrolled undeclared module is declined exactly as an
auto-discovered one is.**

That is worse than the default itself, because it **defeats the remedy**. The one deliberate
action available to someone whose evidence is not running fails silently, for the same reason,
and gives the same non-signal. *Explicit enrolment does not rescue you* is the sentence an author
needs, and it does not follow from the two-consumer version of this finding — which is why the
enumeration mattered and the assumption would not have.

The repair below covers it for free: it is the same carrier.

**So the repair is not a changed default and not a refusal at the boundary.** It is to return the
three-state value — `Option<bool>`, or better a named coproduct — and let each consumer decide.
Selection keeps its fail-closed reading of `undeclared`, correctly and explicitly. Execution routes
`undeclared` to the effect-reach derivation it already has and which is currently unreachable for
exactly this population. No new mechanism is required: the derivation exists, the third state
exists one line before it is destroyed, and the fix deletes the `unwrap_or` rather than adding
anything.

That is the §5 move — the invalid conflation stops being writable, rather than being defaulted
carefully — and it is a strictly smaller change than the refusal arm proposed earlier in this
brief, which would have made *undeclared* loud without making it **meaningful**. Earlier proposal
superseded; this one supersedes it because it was reached by asking why the state was missing
rather than what to do when it is.

---

## A positional citation in this brief, caught by review, one night after I ruled against them

`deep-ant-102` read the section above and could not follow its citation: the two line numbers I
gave land on `match &result {` and `current_file = None;` in their tree — both unrelated — because
we are on different revisions of a file that has moved by roughly 600 lines between them.

This is DESIGN §3's positional-citation class, in a durable brief, committed by someone who had
**already** stripped rotted offsets out of a different brief earlier the same night for exactly
this reason. The rule is not hard to state and it did not survive one hour of contact with a
convenient copy-paste. Worth recording plainly rather than quietly fixing, because the interesting
fact is not that positions rot — that is the ruling — but that *knowing* it does not prevent
writing one. The habit is the load-bearing part, and the habit is what failed.

The citation is now symbolic and stays true across every revision: `parse_entry_live_tree_disposition`,
its `Option<bool>` local, and its `unwrap_or` tail. Note that the symbolic form is also *shorter*
and says more — it names what the reader should look for rather than where it happened to sit.

---

## The class splits in two, and the split decides the repair

Asked whether all three members share the carrier shape, `deep-ant-102` checked instead of
generalising. **Two do, one does not**, and that is a better result than three-of-a-kind because
the difference predicts which fix is correct. Both halves verified here by symbol before being
recorded.

### Carrier too narrow — the third state never reaches the consumer

The value is destroyed at a boundary, so no consumer can recover it however carefully it is
written. **The fix DELETES the collapse.**

- **live-tree disposition.** `parse_entry_live_tree_disposition` holds `Option<bool>` internally
  and returns `Result<bool, String>` via `unwrap_or(true)`.
- **floor scanner.** `witness_file_from_source` returns `Option<InventoryWitnessFile>` and ends
  `if functions.is_empty() { return None; }`. That `None` conflates two different facts — *this
  file has no test declarations at all* and *this file has test declarations of a kind I do not
  recognise*. The 13 `test data`-only files are the second, reported as the first.

The scanner's own source carries a comment directly beneath that return calling it a **§3 defect,
one fact two computations in one binary, deliberately not fixed**. The file already knows.

**Independent corroboration nobody arranged:** the operator prescribed exactly this repair for the
scanner — model `DiscoveredTestDeclaration = TestFunction { identity } | TestData { identity }` and
let the executor decide per kind — and warned specifically *against* the alternative of adding
`test data` to the existing `Vec<String>`. That is carrier widening, ruled from the opposite
direction, before either finding had been connected to the other.

### Arm discards a good answer — the third state arrives and is thrown away

The value survives to the consumer intact; the consumer chooses to widen instead of using it.
**The fix ADDS a typed refusal.**

- **bare-name closure.** `resolve_in` returns `Option<String>`, and `None` is both expressible and
  returned. The consumer reads:

  ```rust
  let target_module = match resolve_in(&census) {
      Some(m) => Some(m),
      None => resolve_in(&pool_bare_census(index)?),
  };
  ```

  *Not found in this scope* is a real answer that arrives undamaged, and the arm discards it to ask
  a bigger question. This is §5's absorbing fallback proper — nothing is missing, something is
  refused.

### The diagnostic question, which replaces the single-class framing

> **Does the third state reach the consumer?**
> **If yes it is an arm — add a refusal. If no it is a carrier — delete the collapse.**

Same symptom from the outside; opposite repairs. And getting it backwards is not a wash: **a
deletion where an addition is needed leaves the widening arm intact**, so the carrier is fixed, the
value now arrives, and the arm quietly discards it exactly as before — with the collapse gone, the
evidence that anything is wrong goes with it. The repair would look complete and measure clean.

That failure mode is the same one that made this night's guard reviews hard: a fix at the wrong
level is not merely insufficient, it removes the symptom that would have located the real one.

---

## Postscript: the habit did the work, not the judgement

`vivid-boar-345`, on why they caught a decoration in their own witness while two other sessions
missed a structurally identical error the same night, and declining the flattering explanation:

> It was not that I distrusted the instrument. I ran the RED arm first out of habit, and the
> decoration announced itself by passing when it should have failed. Had I written the row and only
> run the green arm, I would have shipped it and been just as confident.

That is the argument for making *run the RED arm first* **unconditional** rather than reserving it
for cases that feel shaky — because the feeling is absent in exactly the cases that need it. Both
errors this document records were committed with full confidence and no sense of risk: the
positional citation, and the call-site grep read as an answer to a question it was not asked.
Neither would have been caught by more care, because care was not what was missing.

A discipline that only fires when you suspect something is a discipline that fires when you least
need it.
