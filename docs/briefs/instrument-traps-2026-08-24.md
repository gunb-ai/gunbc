# Instrument traps measured on 2026-08-24

**Read this section and stop, unless you are looking for a specimen.** The rest of the document is
evidence, and it grew past a thousand lines in a day because the class kept recurring while it was
being written.

## The rule

**Check the comparands, not the comparison.**

Every instance below produced output that was **TRUE**. None was a bug in a tool. In each case the
instrument answered a **narrower or neighbouring question than the one asked**, and the reader
supplied the missing binding without noticing.

The diagnostic question, which is mechanical and is the whole of the method:

> **What is the instrument supplying that the subject would have to supply for itself?**

    compile-clean          supplies the whole-tree pool          -> "this module is fine" is not what it said
    a stale binary         supplies its own identity             -> two questions, one TRUE byte
    a capped list          supplies the unseen 3865 rows         -> empty and truncated are one output
    two lanes, two refs    supplies the branch the ruling names  -> both right, neither on the subject
    an ambiguity counter   supplies "unique means authorized"    -> unique-but-wrong reports zero

## Four things that transfer

1. **A false absence is worse than a false green.** A green is caught by whatever depends on it; an
   absence *terminates the investigation*, because there is nothing downstream to trip over.
2. **"Check when the answer surprises you" is exactly inverted** — it fires on the results least
   likely to be corrupted and stays silent where corruption is indistinguishable from truth.
3. **The repair for a claim on the wrong subject is to MOVE it, not to soften it.** Hedging reads as
   rigour while discarding signal (§ *the repair for a claim on the wrong subject*).
4. **Attention is not a remedy.** The strongest evidence in this document is that its own authors
   committed these errors *while writing it* — one within the hour, on the change recording the class,
   and three of us in a single thread that was explicitly about it.

**Not one instance in this document was caught by its own output.** Every one was caught by someone
re-deriving what they had already said. That is not a run of bad luck with tools; it is what it means
for a distinction to be absent from a channel.

---

## Specimens

What follows is the evidence, roughly in the order it was found — across two sessions and nine lanes.
Each is recorded with what was measured rather than with advice, because the advice is always the
same and always insufficient ("check your instrument") and what transfers is the specific shape.

---

## 1. `gh run rerun` is not a second observation

A rerun reuses the **original merge commit**, so it re-measures the same stale base and
reproduces the identical error. Only a new head recomputes the merge ref.

**Why it is expensive:** the failure looks exactly like a successful reproduction — same
error, same place, twice — which is normally the strongest confirmation signal available.
The defect defeats the instinct rather than requiring carelessness.

*Found by bold-raven-901, who asserted the wrong remedy on a PR and corrected it there.*

## 2. ctrl-build has TWO head behaviours and nothing in the output distinguishes them

- fetch-a-named-commit → `HEAD` genuinely is that commit
- apply-a-diff-onto-the-pushed-base → the echoed head is **main's**, not yours

Both were observed in this session, same tool, same day. The only distinguishing evidence
is the log preamble.

**The rule is not "remember which mode you are in."** It is: *never let a git-derived field
carry the attribution.* Assert the subject **in-tree** — e.g. a pair of counters proving the
tree contains the fix and does not contain the defect — so the run proves it measured what
you think it measured regardless of which commit it believes it is on.

*Found by lively-koi-546, whose `PROBEHEAD` column was main's sha on one run.*

## 3. `--depth=1` makes ancestry checks uninformative, not false

`git merge-base --is-ancestor` fails closed on a shallow clone, so a remote probe reporting
`ANCESTOR=absent` is describing the CLONE, not the history. Fails closed, so not dangerous —
but reporting it as a fact rather than an artifact is silence-as-zero.

## 4. `fn` is not the only declaration keyword — `func` is too

269 `func` against 36654 `fn`. A census pattern of `^fn ` undercounted a population by a
factor of 2.7, and **two sessions produced the same wrong number independently** from the
same defective pattern.

A neighbouring failure the same hour: a pattern with no END anchor counted
`witness_verdict_diagnostic_companion` (the derivation itself) and
`lens_verdict_diagnostic_locus_module` (substring only) as producers of a channel that has
none.

**Four defective greps in one night between two sessions.** The only one that reached nobody
was caught because the finding happened to be interesting enough to double-check before
sending — that is not a discipline anything can rely on. Match both keywords and anchor both
ends.

## 5. A dashboard-only review is invisible to `gh`

`dashboard-ops reviews <pr>` rows can carry `dashboard_only: true`. Those do **not** appear on
the GitHub PR — `gh pr view --json reviews` cannot see them at all.

So a lane checking readiness through `gh` sees `appr=1 rc=0` while the dashboard says
`rc=1`. This is why the working agreements name `dashboard-ops reviews` as the source of
truth, and it is not a preference.

**And the join still has to be done by hand:** `merge_criteria` has itself published
`1/1 approvals` under a `head_sha` that the only approving row did not sit on. Join
`reviews[].sha` against `merge_criteria.head_sha` yourself, every time.

**The instrument can also be BEHIND a push** — a join that passes right now may be joining an
approval against the previous commit. Confirm `head_sha` is the commit you mean.

## 6. Archival is hard-blocked on open PRs

`dashboard-ops archive <session>` returns `HTTP 409 archive-refused (reason: open-pr)`.

So closing lanes does **not** free child slots while their PRs are open. With manual
operator merges, subtree throughput is bounded by **merge rate**, not by lanes. "Close lanes
to free capacity" frees attention, which is real, but is not what the queue is short of.

## 7a. The side-chat read returns ONLY the other side — your own messages are never in it

`dashboard-ops side-chat` returns a `turns` array containing the assistant/operator turns.
**Your own posts do not appear in it, ever.**

So "my message is not in the thread" is *not* evidence that the send failed. It is what the
view always shows. This was measured the hard way: a post was reported as failed on exactly
that reasoning, twice, to a peer who then recorded the wrong lesson — while the operator's
next turn quoted the post's specific content back, including a commit sha only that message
had supplied.

**And a send can complete asynchronously long after its foreground call gives up.** A send
that produced no output, and appeared absent from a subsequent read, had in fact landed; the
token counter decremented later, which was the only true signal available and was initially
dismissed as stale.

**Confirmed structurally, not inferentially.** The first account of this reasoned from the
operator quoting back a commit sha only one message had supplied — sound, but it establishes
that *one* send landed. A second session then checked the endpoint itself: across three
separate captures of its own thread (3, 4 and 6 turns), **zero of its four posted messages
appeared in any `turns` array**, including one carrying `sent:true` with a counter decrement.
`role` is `None` on every entry, so there is no field to filter on. The array simply does not
contain your side.

Correct readings, in order of reliability: the **token counter decrementing**; the other
side **responding to your content**; the returned turns — which cannot answer the question at
all.

**The third is not weak, it is INCAPABLE, and the distinction decides behaviour.** A weak
signal invites more sampling; an incapable one must be abandoned. Six retries is what you do
to a weak signal — and six retries is exactly what was spent here before anyone asked whether
the instrument could answer at all. The action that survives every reading is the same: **never re-send on a missing
receipt**, and under the asynchronous-landing reading that rule gets *stronger*, because a
missing receipt now carries no information whatsoever about whether the send will land.

## 7. The side chat's read path needs a long BACKGROUND budget

Foreground calls (100s, 110s, 400s) return `HTTP 502: conversation fetch failed` or hang with
zero bytes and no stderr. One success in this fleet came from a **900s backgrounded** call.

**Two different sessions produced two different symptoms** for the same surface — a 502 in one,
zero-bytes-no-stderr in the other — so neither should generalise from its own.

Side chats are **per session**. One session cannot read another's, so relaying is not
available no matter whose instrument is healthy.

**The state to hold explicitly:** *never successfully read in this session* means you have no
baseline, so you cannot distinguish a channel that broke from one you never reached. Until a
read succeeds, the absence of a reply is not information — treating it as one is the
empty-observation narrow with your own observation as the victim.

## 8. A caveat can travel without changing anyone's behaviour

A measured figure (12278 declared `test fn`) was published together with an explicit caveat
that it was **not** the right denominator, naming the correct one (~10439 routed). The caveat
was received, agreed with in writing, and the caveated figure was then quoted — by both
readers, repeatedly, including into the one channel with a confirmed receipt, with the caveat
dropped in transit. The number was corrected by a third party who had to notice the error from
scratch.

**This is not a grep failure and no scan discipline prevents it.** The correction existed, was
correct, was acknowledged, and did nothing. What propagated was the figure; what did not
propagate was the *not-to-use-this* attached to it.

The mitigation that would have worked is structural rather than attentive: **do not publish a
number you have already established is the wrong one.** If the right denominator is not yet
measured, publish the gap, not the figure with a warning stapled to it. A caveat is a request
that every future reader do work; the readers here were two sessions that had just spent a
night refusing exactly that kind of request in code.

## 9. A timeout kills the payload and the shell reports success

`timeout N ctrl-build -- <payload>; echo done` exits **0** when the timeout fires. The shell
answers *did the last command in my pipeline succeed*, and you asked *did the payload
complete*. Those differ exactly when a timeout kills the thing you care about.

**The remedy is a planted completion marker, and it is better than vigilance** — the payload's
last statement echoes a sentinel, and its ABSENCE is what you read. That is positive evidence
you cannot fail to notice, rather than an omission you have to spot.

> **THIS CLASS HAS A CANONICAL HOME AND THIS ENTRY DEFERS TO IT: gunbc#9066**, authored by
> silent-gull-867, who hit it independently — a dispatch returning exit 0 with the payload
> never executed, the streamed log ending after `git apply`. Their write-up carries the
> distinguishing table (`EMITTED=0` means it ran and produced nothing; *no* `EMITTED=` line
> means it never ran, so the dispatch is **void, not negative**), places the class beside its
> neighbours (`cargo` exiting 0 without compiling, a pipe to `tail` masking a status, `grep -c`
> returning 0 for a missing file), and names how it differs from all of them: those are
> *corrupted* or *absent* statuses, this is an **honest** status answering a narrower question
> than the reader asks of it.
>
> That last formulation is the same one this document arrives at in "The shape they share",
> reached independently the same night from a different specimen. If #9066 lands, this entry
> should shrink to a pointer rather than restate it — two documents for one class is the §3
> violation these notes exist to catch.

## 10. ctrl-build cleans `target/` on every dispatch — so splitting an overrunning job is worse

Every remote dispatch runs `git clean -x -d --force`, which removes `target/`. So a cold
release build is paid **per dispatch**, not once.

This inverts the obvious remedy. A job that overruns the wall looks like it should be split
into N smaller dispatches; that costs **N cold builds** instead of amortising one, and usually
overruns worse. Either fit it in a single long dispatch or choose a different shape — do not
shard it.

Measured on a 70-module standalone-compile sweep: a cold build plus seventy compiles does not
fit a 55-minute wall, and no split of it would have.

---

## The shape they share

Not one of these is an instrument lying. Each answers a real question correctly, and the
question is narrower than the one the reader asked:

| you asked | it answered |
|---|---|
| does the defect still reproduce | does the base still fail |
| what commit did this measure | what commit does the runner believe it is on |
| is this an ancestor | can this clone see the history |
| how many companions exist | how many `fn`-declared companions exist |
| is this PR approved | is it approved *on GitHub* |
| is a slot free | is the lane closed |
| did the operator reply | did the fetch succeed |
| did my message send | did the last command in my pipeline exit 0 |

The transferable move is not vigilance. It is to **make the run assert its own subject**, so
that a narrower answer cannot be read as the wider one.

---

## Trap 11 — the completion marker matched the ECHOED COMMAND, not the output

Measured 2026-08-24 09:15, in the remedy for trap 9, roughly forty minutes after adopting it.

I planted `MARKER_ALL_DONE` as the last line of a remote payload and waited on
`grep -q 'MARKER_ALL_DONE' out.txt`. The wait returned in seconds. The payload had not run —
the dispatch was still applying patches, and no line of my script had executed.

`ctrl-build` echoes the command it is about to run, verbatim, into the same stream:

```
ctrl-build: command: bash -lc $'...echo "MARKER_ALL_DONE"\n'
```

So the marker was present in the file **because I had asked for it**, not because anything
produced it. `grep -q` answered *does this string appear* when I was asking *did the payload
finish* — the shared shape of every trap in this document, arrived at from inside the fix for
one of them.

**What makes this the sharpest specimen here:** trap 9's remedy is right, and this is not a
retraction of it. Planting a marker is still better than reading a status. But the remedy
introduces a *new* way for the reader to supply an unstated binding — that an occurrence of
the marker is an occurrence of the marker's PRODUCTION — and the header echo falsifies exactly
that binding, silently, on every single dispatch. The remedy for the class re-instantiated the
class.

**The correction, and it is one character:** `grep -qx 'MARKER_ALL_DONE'` — anchor the match to
a whole line. The echoed header contains the marker inside a longer line and never as a line of
its own. Alternatively, compose the marker at runtime so it does not appear literally in the
command text (`echo "MARKER_${PHASE}_DONE"`), which additionally defeats any future wrapper that
quotes the script back at you.

**The general rule, one level up from trap 9:** *a marker is evidence only if the channel it
arrives on cannot also carry the request for it.* Where request and response share a stream —
and with a command-echoing dispatcher they always do — the marker needs a shape the request
cannot have. Anchoring to a whole line is the cheapest such shape.

Recorded as a numbered trap rather than an edit to trap 9 because the two are different
findings: trap 9 is *a status can be honest and narrow*; trap 11 is *your own instrument's
output can be contaminated by your own instrument's input*. Fixing 9 does not fix 11, and I
found 11 only because the void reading was so obviously wrong — had the payload merely been
slow, I would have read the false green and reported a passing test that never ran.

---

## Trap 12 — the witness went where its siblings live, at a grain blind to the case

Contributed by vivid-boar-345 on gunbc#9058, found twice in one day by the same lane, and
recorded here because it is an instrument trap in the strict sense: the measurement was placed
correctly by every convention except the one that decides whether it can measure anything.

Repairing an empty-record spelling fork, they wrote the natural witness row beside its
siblings — a `decl_facts` assertion that both empty spellings carry one shape — and **it passed
on the pre-change binary.** `concept_decl_node` marshals a `NoConnective` item through
`unit_type_node`, which is a `TypeNode { connective: Conj }` with zero children: byte-identical
to what an empty record marshals to. So the row was permanently green *whichever way the parser
answered*. It would have shipped as coverage for precisely the thing it cannot see.

They caught it by running the RED arm first. Nothing else would have — the row is green after
the fix, green before it, and green under any mutation of the behaviour it names.

**Their generalization, which is the part worth keeping:** *where the behaviour is, not where
the neighbours are.* The pull toward the siblings' grain is strong and it is usually right; it
is wrong exactly when the sibling grain marshals away the distinction under test. `decl_facts`
returns a declaration's CHILDREN, and both the seal and the empty/unit distinction live in its
PROPERTIES — so the whole family of questions about properties is invisible at the grain the
family of witnesses uses. Their replacement asserts through a sealed empty pair instead, because
the seal is the one thing that still differs between a nominal empty record and an alias to unit;
pre-change exactly one of seven rows red, post-change seven green.

**Why this is trap 11's sibling and not a repeat of it.** Trap 11 is *the channel carrying your
answer also carried your question*. Trap 12 is *the instrument's resolution is coarser than the
distinction you are asking about* — and in both, the output is TRUE and the reader supplies the
missing binding. The distinguishing question is cheap and should be asked of every new witness
before it is enrolled: **can this row go red?** Not *does it pass* — DESIGN §4b already says a
check whose RED is unauthorable is a decoration, worse than absent because it is cited as
coverage. Trap 12 is what that rule looks like when the unauthorable-RED is caused not by the
corpus but by the marshalling in the assertion's own read path.

---

## Trap 11a — anchoring one term of a disjunction leaves the disjunction unanchored

Same session, ~30 minutes after writing trap 11 and its correction.

Having established that `grep -q MARKER` matches ctrl-build's echo of my own command, I re-armed
the wait as:

```sh
until grep -qx 'PHASE_RED_DONE' out.txt \
   || grep -q 'Remote run completed\|PATTERN_MISS\|NO_BINARY' out.txt
do sleep 20; done
```

The success marker is anchored. The **failure sentinels are not** — and `PATTERN_MISS` and
`NO_BINARY` are strings *in the script*, so they appear in the echoed header exactly like the
marker did. The wait returned immediately, against a dispatch that was still building.

**Why this is worth a numbered entry rather than a footnote to trap 11.** I did not forget the
lesson; I applied it to the term I was thinking about. The fix in trap 11 is phrased as a fact
about *the marker* — anchor the marker to a whole line — and I implemented exactly that
sentence. The disjunction's other arms were never re-examined, because they are not "the
marker", they are the error cases, and the correction as written did not reach them.

That is the general failure of a remedy stated at the wrong grain: **a rule about one term does
not survive being embedded in a compound predicate.** The rule that does survive is a rule about
the *channel*: on a stream that carries the request alongside the response, **every** literal you
match must have a shape the request cannot have — success markers, failure sentinels, error
strings, all of them. Not the one you happened to be reasoning about.

Corrected form, with every arm anchored:

```sh
until grep -qxE 'PHASE_RED_DONE|PATTERN_MISS|NO_BINARY' out.txt; do sleep 20; done
```

Cost this time: one wasted read and a wrong "still in progress" reading, caught within a minute
because I checked for the markers instead of trusting the wait's return. Had I reported off it,
I would have said a mutation probe had not started when it had — the inverse of trap 11's
failure, from the identical cause.

---

## Reviewer trap — naming the mechanism instead of the property, and the correction to that correction

Two instructions I gave tonight were wrong *as specified*, in the same way, hours apart:

- To gunbc#9075: *put these rows in a module whose honest disposition is hermetic.* No such module
  can exist — any `compile_dag_diagnostic_census` probe resolves against the live checkout. The
  author, given an impossible instruction, produced a false declaration to satisfy it.
- To gunbc#9063: *attest against membership in `rust_identifier_tokens(s: emitted_source)` — same
  machinery, no new concept.* That function splits `<>,()[]&:` and space, with no `;`, because it
  was written for a **rendered type string**. Against a whole module, `pub type X = Int;` tokenizes
  to `Int;`, so membership answers NO for a name the emitter demonstrably wrote. The swap would
  have dropped required use-lines and reproduced the exact defect the code exists to prevent.

In both the **property** was right — evidence must execute on the gating path; attestation must
match whole identifiers — and the **mechanism I named** was wrong. And in both I reached for the
mechanism because it looked *convenient*: an existing module, an existing function. Reusing a
function outside the domain it was written for is not DRY; it is the hollow-alias failure, and the
reviewer is unusually prone to it because the reviewer sees the shape of the machinery and not its
preconditions.

**The obvious lesson — state the property, let the author choose the mechanism — is half right, and
the author of #9063 supplied the other half:**

> I would not have found the delimiter problem if you had not named the exact function to reuse —
> checking whether it fit its new domain was a cheap question that only got asked because there was
> a concrete proposal to check. The proposal being wrong in a checkable way is worth more than a
> vaguer one that was not.

That is correct and it prevents an over-correction. A property stated alone (*match whole
identifiers*) is unfalsifiable at review time and hands the author an open problem; a named
mechanism is **checkable in one grep**, and being wrong in a checkable way is a service. The
failure was not the concreteness — it was the **grammatical mood**.

**The synthesis: name the property, offer the mechanism as a candidate to check, never as the
instruction.** *Attestation must match whole identifiers rather than substrings; `rust_identifier_tokens`
looks like the right tool — check its delimiter set against a whole module before using it.* Same
concreteness, same one-grep falsifiability, and the author is left holding the domain question
instead of an order. The #9075 case fails the same test in the other direction: had I written
*the evidence must execute on the gating path, or be declared authored-not-executing — is there a
module whose honest disposition gets it there?*, the answer would have come back **no**, which is
the correct answer and was unreachable from the instruction I actually gave.

---

## Trap 13 — when your predictions are absences, a dead harness confirms them

Contributed by `deep-ant-102`, from a truncated floor probe. This is the sharpest member of the
family recorded here and it subsumes part of traps 9 and 11.

Their dispatch hit ctrl-build's 45-minute default cap mid-floor. Output ended at their own header:

```
===== FLOOR: real roots + probe fixture =====
[exited with code 0]
```

No arm lines, no summary, no exit code from the subject. **And their grep for probe arms would have
returned empty** — which is not a null result here, it is a **confirming** one:

| pre-registered prediction | what a dead harness returns |
|---|---|
| `declined_live` +0 | nothing found → looks like +0 |
| four files in the roster, not five (arm4 invisible) | nothing found → looks like invisible |
| zero `test data` identities planned | nothing found → looks like zero |
| **the run goes red on arm5** | nothing found → **fails** |

Three of four predictions apparently confirmed by a run that never executed.

**The general shape: an absence-shaped prediction is confirmed by the absence of the experiment.**
A probe that predicts *X should not appear* cannot, without further construction, distinguish
*X did not appear* from *nothing appeared, including the experiment*. Every trap in this document is
some form of a signal answering a narrower question than asked; this is the form where the signal
answers **the very question you asked, correctly, about a universe that does not exist**.

**Why the liveness control is not optional, and why it is the one prediction that cannot be faked.**
Arm 5 was added for a different reason — to catch a harness that runs and evaluates nothing. It
happens also to be the only **presence** prediction in the set, and therefore the only one a dead
harness cannot satisfy. That is not a coincidence to note in passing; it is the design rule:

> **A pre-registration made mostly of absences needs at least one presence prediction, or the
> experiment cannot fail in a way you will notice.**

Count the moods in your predictions before you run. If they are all *should not appear*, *should be
zero*, *should be unchanged*, then the null hypothesis and the broken instrument return the same
string and the run cannot inform you.

**And the marker rule gains its final clause.** Trap 11 established: anchor the marker to a whole
line, because the request shares the channel. Trap 13 adds: **plant it AFTER the subject, not only
before.** A header echo proves the dispatch started; only a trailing marker proves the subject
finished. Their header printed; that is precisely why the truncation was legible as truncation
rather than as data.

Paired with silent-gull-867's `#9103` false negative (`mirror-drift-four-lanes-2026-08-24.md`),
these are the two ways a run agrees with you for the wrong reason: **the treatment never reached
the instrument**, and **the instrument never reached the subject**. Both return the control's
answer. Neither raises an error.

---

## Trap 14 — exculpatory attribution: the explanations that let the line restart

Four instances in one morning, across three lanes and one of them mine. It is the author-side
absorbing fallback (DESIGN.md §5) with a specific and predictable shape, and naming the shape is
what makes it catchable, because in the moment each instance feels like ordinary diagnosis.

**The shape.** A run fails. Several explanations are available. Some place the cause *inside* the
change and some place it *outside* — a race, pre-existing breakage, an unrelated phase, missing
infrastructure, someone else's PR. The outside explanations are the ones that let the line restart
without work, so they are the ones reached for first, and — this is the whole trap — **they are
reached for without being checked**, because checking is a cost you only pay for a hypothesis you
expect to act on.

**The four instances.**

1. *smart-wolf-868, #9075:* a failing run attributed to a race, because the run started in the same
   minute as a push. The run carried `sha=4e3c2327860` — its own pushed commit. Runs are pinned;
   a concurrent push cannot change a checkout already made. One `gh run view` away.
2. *Same lane, same run:* reported as a **regen** failure. The ledger read
   `regen first_generation_equal=true` and `FAILED PHASE floor`. The subsequent push "to force a
   clean candidate comparison" therefore fixed a phase that had passed.
3. *Same lane, same run:* the floor diagnostics called pre-existing. Main's last six runs were all
   `success`, and the PR's base was `bd84f6696` — **the exact commit whose own run passed**.
4. *silent-gull-867, #9076:* `spawn rustfmt: No such file or directory` attributed to "the runner
   missing a binary." Eight sibling runs in the same window on the same runner image: zero such
   errors. Their run was the only one, twice.
5. *Mine, in the message correcting instance 3:* "the other ~71 PRs are BLOCKED for a different
   reason." Measured: 47 CLEAN, 19 BLOCKED, 4 DIRTY, 1 UNSTABLE. Wrong on both halves, asserted in
   the same paragraph as an instruction to check before commissioning work.

**Why instance 5 matters most.** The trap is not a competence failure and it is not confined to the
lane under pressure. I walked into it one paragraph after describing it, against a number I could
have measured with the command I had just run. Any rule of the form *be more careful* would not have
caught it, because I was being careful — about the other lane's claim, not my own aside.

**The operative rule, which is mechanical rather than attitudinal:**

> **An explanation that places the cause outside your change is a hypothesis with a cheap test.
> Run the test before you act on the explanation, and especially before you tell anyone.**

The corollary is what makes it affordable: these tests are nearly free. A pinned SHA is in
`gh run view`. A phase verdict is in the ledger you already opened. "Pre-existing" is answered by
the base commit's own CI run — **which already exists, was already paid for, and is a pre-executed
control on the exact tree** (this is worth internalizing on its own: every PR ships with arm B of
its own two-arm comparison, and lanes routinely commission a 30-minute build to reproduce it).
"Infra" is answered by siblings in the same window. None of the four instances above needed a build.

**The reviewer-side tell**, since the author cannot always see it: an attribution to something
outside the diff, stated without the measurement that would establish it. The words are usually
*pre-existing*, *unrelated*, *flake*, *infra*, *race*, *stale*, *not mine*. Each is a real category —
that is exactly why the class is durable, and why "distrust these words" is the wrong rule. Ask for
the measurement instead. In all four cases here the measurement existed and took under a minute; in
three of the four it reversed the conclusion.

**Relation to the rest of this brief.** Traps 11–13 are ways an instrument returns the control's
answer while you believe it measured the treatment. Trap 14 is the step before the instrument: the
decision not to measure at all, taken because a costless explanation was already available. It has
the same signature as the others — no error is raised, the output is true as far as it goes, and the
reader supplies the binding that makes it wrong.

### Trap 14a — the sharper sub-form: your instrument cannot express the answer

Instance 5 above was mine: an unmeasured assertion. Within the hour I produced a worse one, and it
deserves separating because the rule from trap 14 would not have caught it — **I did measure.**

I set out to check whether the floor names the six roster rows it reports as `now PASS and must be
removed`. I searched a 337KB log by extracting `//[a-z_0-9/]+:[a-z_0-9]+`, the label format used by
the per-claim `PASS`/`FAIL` lines. Zero matches. I concluded the floor counts them without naming
them, called it a §5 violation — typed and counted but not located — reported it upward as a defect
worth fixing, and another lane offered to fix it.

The floor names all six:

```
required-floor: STALE-QUARANTINE test.claim.samsung_dram_module.generation_is_ddr4 is enrolled
  as expected-red and PASSED — remove it from v2.workflow.floor_expected_red
```

Six lines, with remediation text, in the file I had already downloaded, emitted by a loop that sits
beside seven siblings doing the same for `ROUTE-GAP`, `HOST-TOOL-UNRESOLVED`,
`INTERRUPTED-BEFORE-VERDICT` and the rest. These carry **dotted qualified names**. My pattern
required a `//`-prefixed, colon-separated label. It could not have matched a true positive.

> **A search that returns nothing has told you about your pattern until you have shown the pattern
> can match a true positive.**

The failure is not carelessness about whether to measure; it is skipping the question of whether the
instrument's vocabulary matches the subject's. A regex is a claim about *format*, and I was asking a
question about *presence*. Zero results answered the format question, and I read the answer as
presence — the same substitution trap 11 makes with a marker and trap 13 makes with a truncated
dispatch, now one level further out: not the instrument reaching the wrong subject, but the
**instrument being unable to represent the finding at all**, returning ⊥ that reads as *absent*.

The positive control was free and I skipped it: grep for the bare token `STALE-QUARANTINE`, or for
any known-present string, and confirm the harness can find something before trusting that it found
nothing. One `grep -c` would have shown 6.

**And the evidence that should have stopped me was already in view.** `[floor-known-red-causes]`
prints per-item detail immediately below the counter line I was doubting. I read that as precedent
for what *should* exist rather than as evidence about what *does* — an author who prints a per-cause
census three lines down is not an author who omits a per-identity list. Corroborating structure was
sitting adjacent to my conclusion and I used it to motivate the finding instead of to test it.

**Cost, and why it is the real argument for the positive control:** the false finding was relayed
upward inside four minutes and a second lane volunteered to fix a defect that does not exist. An
unverified negative does not stay in one head; it recruits.

**The salvage, which is the honest outcome:** all six stale rows are `test.claim.samsung_dram_module.*`
— one module, six roster lines to delete, a real and trivial unblock. The correct finding was smaller
than the invented one. That is the usual shape.

### Trap 14b — a spec tells you what a mechanism does, not what invokes it

The third sub-form, produced an hour after 14a, and it completes the taxonomy: **14** is not measuring;
**14a** is measuring with an instrument that cannot express the answer; **14b** is reading an authority
correctly and drawing a conclusion it does not license.

I measured that 20 of 50 mergeable PRs touch the stage0 mirror, and that **12 of them modify one file**,
`src/v1/stage0/src/cli_run.rs`. That part was real. I then reasoned from DESIGN.md's Building-&-checks
section, which records that the generated-artifact merge driver *refuses* rather than answering `true`,
leaving the path unmerged with a regeneration recipe. Git invokes a low-level driver exactly when both
sides changed a path since the merge base — true of every pair of the twelve. Conclusion: the twelve
serialize, one merge plus one regeneration cycle each, and the queue's cost is quadratic in the drought.

I sent that upward as a merge-strategy recommendation, labelled derived-rather-than-measured, and then
measured it. **Both halves are wrong.**

```
$ git check-attr merge -- src/v1/stage0/src/cli_run.rs
src/v1/stage0/src/cli_run.rs: merge: unspecified
```

`.gitattributes` marks four specific generated artifacts — not the stage0 mirror at large. The refusing
driver never engages for this file. And the merge itself, two of the twelve onto `origin/main` in
sequence:

```
--- merge A ---   Auto-merging src/v1/stage0/src/cli_run.rs    rc=0
--- merge B ---   13 files changed, 957 insertions(+), 40 deletions(-)   rc=0
```

Clean both times. No ordering constraint exists.

> **A specification describes a mechanism's behaviour. It does not enumerate the inputs the mechanism
> is applied to. Scope of application is a property of the tree, and only the tree can answer it.**

Everything I read was accurate: the driver does refuse, and git does invoke low-level drivers under
exactly that condition. The invented step was the silent premise that this file is one of the driver's
inputs — which no sentence I read claimed, and which one command falsifies. This is DESIGN.md's own
**authority substitution** in the reader's direction: fact F (how the driver behaves) lives in one
carrier, operation O (how *these paths* merge) is governed by another, and I let the first answer for
the second because both halves checked out and only the arrow between them was missing.

**It is the most dangerous of the three**, because 14 and 14a produce claims that feel thin while you
are making them, and this one felt like *reasoning from the authority*. The correct move is not more
skepticism toward DESIGN.md; it is noticing that "what does X do" and "does X apply here" are different
questions with different oracles, and the second one is nearly always cheaper.

**What made it recoverable** was labelling the claim derived-not-measured when sending it. That label is
what sent me back to check twenty minutes later instead of leaving a false mechanism in someone's merge
plan. **Marking a claim's evidential status is not hedging — it is the thing that schedules its own
verification.**

**And the residue is better than the claim.** Text-merging two independently regenerated halves of a
generated file produces a mirror no single emit produced. Whether it still equals a fresh emit is what
`--required-regen` checks on main *after* the merge — so the failure mode, if it exists, is main going
red on regen after a multi-mirror merge, invisible at merge time. Untested here (a full regen is a
CI-sized job) and recorded as an open question, not a finding. It is presumably why the driver covers
the files it does.

---

## The rule that subsumes traps 14, 14a and 14b — check the comparands, not the comparison

Five instances in one day, across two sessions. They were catalogued above as separate sub-forms,
and that framing is now superseded: **they are one class, and it has no cheap wall in front of it.**
Arrived at jointly with deep-ant-102, who put it best — every failure was on the axis of *what am I
holding*, never on the axis of *what did I measure*.

> **Before interpreting a measurement, establish that the things being compared are what you think
> they are.**

Every instance below is a measurement that was CORRECT, interpreted against a comparand that was
assumed. In each case one cheap query — usually one command, always under a minute — identified the
comparand, and in each case it was not run because the question felt already answered.

| the assumption | the free query | what it returned |
|---|---|---|
| the merge driver applies to this path | `git check-attr merge -- <path>` | `merge: unspecified` — it does not |
| these floor errors are pre-existing on main | the base commit's own CI run | main green at that exact sha — they are not |
| my branch is current, so this delta is drift | `git rev-list --count HEAD..origin/main` | `13` — the delta was my staleness |
| my search found nothing, so nothing is there | `grep -c <known-present-token>` | `6` — my pattern could not match the format |
| this list is the population | is the view truncated? | `tail -12` was a window, not a total |

**Why "be more careful" does not address it.** In four of the five the author *was* being careful —
measuring, controlling, labelling evidential status. The trap sits one step before the care: the
comparand is supplied by memory and never enters the evidence. And the class is symmetric, which is
why the earlier sub-form rules each caught only half of it:

- **negatives** — a filtered or truncated view returns ⊥, and ⊥ reads as *absent*;
- **positives** — a diff returns a real difference, and the difference is attributed to the wrong side.

An absence rule (*show the view could contain it*) misses the second; a presence rule (*identify both
sides*) misses nothing, which is why it is the one to keep.

**The operational half, and it is deep-ant-102's, kept verbatim because it is the only reliable exit:**

> **When a filtered view yields a negative, go get something that could positively contradict it.**

Absence never contradicts anything, so no amount of staring at a negative result will overturn it.
The two escapes today were both positive artifacts that made the standing conclusion impossible: a
base64 payload containing the very row I had just concluded was missing, and a `git status` line
naming the file I had just concluded was never written.

### What it cost, and the near-miss that is the real argument

Four of the five produced a wrong claim that reached another session inside minutes; two of those
recruited a second party to act on them. The one that matters most never reached anyone.

Holding a regenerated DESIGN.md, I was one commit from landing it. My branch was 13 commits behind
main, so those bytes would have **reverted two of main's paragraphs** — inside a PR whose stated
purpose was adding one row, in the repository's canonical authority document, with the drift gate
that would have caught it unguarded since the floor cut. Every individual step was sound: regenerate
from the authority rather than hand-edit the artifact, commit authority and output together, verify
the output contains the intended change. It was correct work aimed at a comparand nobody had checked.

`git rev-list --count HEAD..origin/main` is the whole wall, and it costs nothing.

**Corollary for reviewers**, since the author's own care demonstrably does not catch this: when a
diff, count, or absence is offered as evidence, ask what it was compared *against* and whether that
was verified or remembered. It is a different question from "is the measurement right", and today it
was the only one that mattered.

### The case neither rule covers: a complete file that is a partial record

The comparand rule above was written one instance too early. deep-ant-102 produced a sixth case the
same afternoon that **has no filter and no comparison**, and it is the sharpest of the set.

They reported that a wet regeneration pass wrote exactly twelve files, none of them DESIGN.md. I
attributed it to a truncating `tail`. They checked, correctly, and refused: their command carried no
`tail`, and `grep -c` over the **whole file** returned 12. Both statements were true.

The clean-tree run settled it:

```
[file] write DESIGN.md (94037 bytes)
```

Eighth in a list of at least 69 traces, on an unmodified tree. The twelve were the **final twelve**
of that list. The task-output capture had elided the *middle* of a long stream, keeping the first six
lines and the last twelve — a 31-line file standing in for a 69-line record. `grep -c` over a
complete FILE returned a complete count of an incomplete RECORD.

> **Does the artifact I am reading claim to be complete, and could it know?**

A file has no way to tell you it is short. Nothing inside it is wrong, nothing is filtered, and no
comparison is being made — so neither the absence rule nor the comparand rule fires. **File
completeness is not record completeness**, and the gap between them is invisible from inside the
file.

Only two defences exist, and both must be arranged *before* the run:

- **a terminal marker you planted yourself** — its absence proves truncation, because you know it was
  emitted last (this is the same discipline as the dispatch-marker rule in `#9066`, and both of us had
  written that rule down before violating it);
- **a count you can predict independently** — fifteen committed artifacts means fifteen-plus traces,
  and twelve should have been recognisable as short.

### And a meta-error worth more than either rule

deep-ant named this one against themselves and it is the reason the question stayed open an extra
forty minutes:

> **A wrong mechanism does not refute a correct conclusion.**

My diagnosis (*your twelve is a truncation*) was right; my explanation of it (*you must have used
`tail`*) was wrong. They checked the explanation, found it false, and discarded the conclusion with
it. That is an easy inversion to make when the explanation is the only checkable part of what you
were told — and the cost is that a settled question reopens.

The reviewer's form: when you refute someone's *reason*, you have not yet touched their *claim*.
Say which one you are refuting.

---

## A pre-registration does not protect you when the null and the not-run are the same string

*silent-gull-867, 2026-08-24. The sharpest statement of this class so far, and it indicts the very
technique the rest of this brief recommends.*

Everything above assumes that pre-registering a prediction disciplines the reading. It does — **but
only when the predicted null is distinguishable from the instrument never running.**

The setup: measure whether adding one alias row to a carrier changes floor behaviour. Success
criterion, registered in advance: *counts unchanged*. Sound as far as it goes.

The failure: **a `.dag` edit reaches the compiler only through regen.** Before the mirror was
installed, the probe measured a binary that did not contain the row. And "counts unchanged" is
exactly the string a stale binary hands back.

> The probe's positive result and its instrument-never-loaded were **the same observation**.

The pre-registration could not tell them apart, because the predicted no-effect and the
never-executed produce identical output. What actually caught it was a marker asserting the
instrument *contained the change* — `MARKER_MIRROR_PP_BEFORE=0` — not the prediction.

**So the discipline has an ordering, and it is the opposite of the intuitive one:**

1. **First** assert the instrument carries the treatment. A marker, a digest, a `grep -c` on the
   *installed* artifact — something only a loaded instrument can produce.
2. **Then** read the pre-registered prediction.

Reversing them makes step 2 unfalsifiable whenever step 1 has silently failed. This is trap 13's
absence-mood problem generalised: it is not enough that *some* prediction be a presence — the
**instrument's own loadedness** must be a presence assertion, separate from and prior to whatever
the experiment predicts.

And it is the exact shape as the trap-14 family: the comparand — *which binary am I measuring* — was
supplied by memory rather than by evidence.

**The residue when it is done right** (from the same lane, on the repaired run): the row's effect
was confirmed absent by identical counts across every field, plus two checks that the agreement was
not manufactured — `declined_live` identical, so nothing agreed by declining to execute; and floor
**preparation** clean on both arms, which is where the failure would surface, ahead of the fold
where no decline arm can reach it. That is what a null result looks like when it carries information.

## A rename with no diff and no refusal

*Same lane, worth its own entry because nothing in this brief would have caught it.*

`container_alias_canonical_spelling` returns the **first sorted key** mapping to an algebra. So
adding a key that sorts ahead of the incumbent silently becomes the canonical emitted spelling for
that algebra **corpus-wide** — adding `"FinitelySupportedFunction"` would sort ahead of `"Map"` and
rename it everywhere.

No diff shows it: the change is one added row. No refusal fires: the lookup is total and the new key
is legitimate. No test asserts the incumbent, because the incumbent was never chosen — it was the
alphabetical accident of a single-member set.

The general shape: **a total function whose answer depends on an ordering nobody declared.** The
ordering is an implementation detail until a second element exists, at which point it silently
becomes policy. Neither the comparand rule nor the marker rule reaches this one; the only defence is
noticing that a lookup returns *a* member of a set rather than *the* member, and asking what decides
which.

Only `"PartialFunction"` was added, which competes with nothing.

## The strongest instance of the comparand rule: a stale binary and a correct one produce the same green

*Reported by silent-gull-867, 2026-08-24, against the regen fixed-point loop on #9059 — and the
reason it closes this brief rather than sitting mid-list is the timing, recorded below.*

The regen loop is: install a mirror, rebuild, re-run, read the verdict. One round ran without the
rebuild — the mirror was installed and then measured **against the binary built before it**. The
round went green.

That green is the comparand rule's purest specimen. The question asked was *does the committed
mirror match what the compiler emits*. The question answered was *does the committed mirror match
what the compiler emitted **before this mirror existed***. Both are real questions, both are
answerable, and **both answer `true` with the same byte.** Nothing in the output distinguishes them,
because the binary's identity is not one of the comparands the verdict is computed over — it is the
thing computing the verdict, and an instrument does not report itself.

What makes it worse than the traps above: those had a channel that *could* have carried the
distinction and didn't. Here there is no such channel. The round's output is structurally incapable
of encoding which binary produced it. The only channel that could catch it is a re-read of the
commands that produced the run — which is exactly how it was caught, and it is not a channel that
scales or that a reviewer has access to.

**Why the damage was zero, stated precisely, because the precise version is the alarming one:** the
round's answer happened not to depend on the reinstall. Not *the loop is robust to this*, not *the
check was redundant anyway* — the inputs that round happened not to differ across the two binaries.
That is luck. A finding whose cost was zero because of what the run happened to contain has not been
handled; it has been survived.

**The timing is the argument.** This was committed by the author who had written up the probe-staleness
class **within the hour, on the change that records it.** Documenting the class did not immunise its
own documenter against it, one hour later, with the text open. Any proposal whose remedy is "readers
will be careful" is refuted by this single data point — the most primed possible reader, at the moment
of maximum priming, on the most relevant possible change, still committed it and still needed a manual
command re-read to notice.

So the remedy cannot be attention. It has to be that the run **names the binary it ran** — putting the
instrument's identity into the receipt makes the two questions above textually different, which is the
minimum condition for the output channel to be able to carry the distinction at all. Until that lands,
every green from this loop is a green about an unnamed compiler.

## The same defect pointing the other way: a silently capped list, where absence is the answer

*deep-ant-102, 2026-08-24, deciding whether #9059 intersects #8282's changed set. Recorded next to
the entry above because they are one defect read in opposite directions.*

The query was: do this PR's paths intersect the cut's? The first run came back **EMPTY** — no
intersection, PR is clear. It is wrong.

    gh pr view 8282 --json files         ->  100 rows
    gh pr view 8282 --json changedFiles  ->  3965

Same call, same PR. The file list is capped at 100 and **does not say it was capped**. An intersection
computed against those 100 is an intersection against 2.5% of the subject, and it returns exactly what
a genuine non-intersection returns: nothing.

**Why this belongs beside the stale-binary green rather than in its own section.** That trap answers
`true` to a narrower question than the one asked. This one answers *empty* to a narrower question than
the one asked. In both cases the output is byte-identical to the correct answer for the real question,
and in both cases the instrument's own limitation is not among the things the output can express. The
stale binary cannot report which binary it was; the capped list cannot report that it was capped.

**The direction matters for how it gets caught, and it is the more dangerous direction.** A false green
is caught by the next thing that depends on it. A **false absence terminates the investigation** — there
is nothing downstream to trip over, because the finding was "nothing here." It was caught here only
because empty *contradicted an expectation*. Had the expectation been "this PR is probably clear," the
empty result would have confirmed it, the PR would have been cleared to land, and it would have landed
into the cut it shares 24 of 25 files with — including `src/v1/04_infer.dag`, where the cut's two
remaining blocking diagnostics live and a hypothesis about them was mid-measurement.

**So the guard cannot be "check when the answer surprises you."** That guard is exactly inverted: it
fires on the results least likely to be the truncated ones, and stays silent on the results where
truncation is indistinguishable from truth. An absence is only evidence if the instrument that produced
it can be shown to have looked at the whole subject.

**The operative rule:** never take a file list from `gh pr view` on a large PR. Use
`gh api --paginate .../files?per_page=100`. And note that even the paginated form stopped at **3000 of
3965**, so the honest statement of the result is `>= 24`, never `= 24`. Reporting the exact figure would
have been a second instance of the same trap inside the fix for the first.

## The adversarial check that refutes the checker: measuring to defend a claim, and finding a third fact

*silent-gull-867, 2026-08-24, answering a REQUEST_CHANGES on #9059. Recorded because the causality
is the lesson, and it is not the causality the rest of this brief teaches.*

Every other entry here says: **measure your own claims, because the instrument may be answering a
narrower question than you asked.** This one is different, and the difference is the point.

A review said the PR expanded two maps and deferred their join — a §3 fork grown rather than
dissolved. The author expected the review to be wrong, and went to measure the two key sets **in
order to demonstrate that the maps were not really one fact.**

The measurement confirmed the reviewer on the substance (profile keys 7 -> 11, alias keys 6 -> 8,
both grew) and **refuted the manager's framing** on something neither the reviewer nor the review had
raised: the two maps are not the same key set and never were. `Int`, `Float`, `Bool` and `String`
sit in the profile map and are *correctly* absent from the alias map — scalars with a method surface
that are not containers. So there is no derive-one-from-the-other relation for a row to dissolve.
The follow-up had been described, by me, as a *join*. It is a replacement migration with a census:
a new authority carrying per-algebra spellings AND profile AND container-resolution, retiring two
authorities across four files in both v1 and dag.

**Three parties, three states, and no two of them the same:**

    the reviewer   right on the substance, silent on the framing
    the author     wrong about the review, right to go and check
    the manager    wrong about the shape of the work, and unchallenged until now

**The lesson is about what produced the third fact.** It was not diligence and it was not carefulness
— the author says so plainly, and the honesty matters because a lesson that flatters the finder
teaches nothing. It was *adversarial intent*: going to measure in order to WIN an argument reaches
for evidence that a defensive re-read never touches. Nobody re-derives a key set to confirm what they
already believe. They derive it to refute someone.

So: **"the reviewer was right, and checking why produced a third fact neither of us had" is a
different lesson from "measure your claims."** The first is about being wrong. The second is about
the *mechanism that makes disagreement productive* — the check you run to defend yourself is the one
that reaches material no agreement would have surfaced. A lane that concedes reviews gracefully and a
lane that argues them with measurements produce the same merge outcome and very different corpora.

**The corollary that binds the manager, which is me.** My "join" framing had been sitting in a
follow-up description, named and apparently owned, for long enough that someone could have picked it
up and built it. It would have been built to the wrong size, and — per the ordering constraint in the
same finding — a flattened lookup would have silently moved which stage refuses. **An unchallenged
framing from someone senior is a defect with no red.** It survived precisely because it was never
worth arguing with.

## The highest-stakes instance: two lanes both right, about different refs

*deep-ant-102, 2026-08-24. Recorded last because it is the comparand rule at the largest blast
radius reached today, and because the author found it in their own message rather than in someone
else's.*

Two reports about the namespace cut, apparently contradicting each other:

    this lane      #8282 is draft, DIRTY, CONFLICTING at 13:05Z  ->  "the cut is not close"
    crisp-crab-430 the cut is clean: 94 sources, 0 blocking      ->  "step 2 is met"

Both were true. Neither was about the same thing:

    #8282 head ref  = integration/namespace-cut        416727a98   draft, CONFLICTING
    the 94/0 push   = integration/namespace-cut-fresh  c6d3a3809

    merge-base = 416727a98      cut ahead of fresh: 0      fresh ahead of cut: 101

`-fresh` is a strict descendant carrying `-cut` plus the 101 commits that include the repair taking
it to zero. **The work is finished and no mergeable artifact carries it**, because the PR still
points at the ancestor.

**Why this is the same trap and not merely a mix-up.** Nobody measured anything incorrectly. Every
number reported was accurate for the ref it was taken from. What was missing is the thing this brief
keeps finding missing: **the comparands were never checked against the question.** The question was
*is the cut ready to land*, which is a question about the ref the ruling names — and one report
answered it about a descendant branch while the other answered it about the PR's head.

**The consequence is the reason it outranks the rest.** The 94/0 was carried to the operator as
"step 2 met", in the message whose entire purpose was to open their merge window. Accurate sentences,
wrong comparand, at the exact moment a decision was being requested. And the induced error propagated
the other way too: this lane read draft-and-conflicting as *the window is hours away* and argued from
it that idling six stopped lanes was the expensive choice. **The truth was closer to the opposite** —
the tree is ready, only the artifact is missing, and that is a small mechanical step. An inference
drawn correctly from a comparand that was never the subject.

**Two properties worth keeping.**

First, **agreement was never available as a check here.** The two reports contradicted each other,
which is the *lucky* case — it forced someone to look. Had crisp-crab not pushed 94/0, this lane's
reading would have stood unchallenged and been just as wrong about the window.

Second, and this is why it closes the brief: it was caught by the author re-reading **their own**
message, the same channel that caught the stale-binary green. Across a full day of instances, in this
corpus, **not one was caught by its output.** Every single one was caught by someone re-deriving what
they had already said. That is not a run of bad luck with instruments; it is what it means for a
distinction to be absent from a channel.

## The class stated as a rule, by someone who was not writing about the class

*keen-tern-667, 2026-08-24, in the body of gunbc#9083 — a PR about a single module's import header.*

> **Whole-tree compile-clean is not evidence that a module can be emitted alone.**

Every other entry in this brief is a war story: an instrument answered a narrower question, someone
caught it, here is what it cost. That sentence is the same content as a **rule**, written by someone
who was diagnosing one module and not writing about instruments at all.

The mechanism behind it is the one this brief keeps describing. `v2.compiler.program_assembly`
declares two imports and uses about twenty-six names homed in nine other modules. Whole-tree
compile-clean passes, because **the definers are in the pool — put there by some other module's
imports — so the unlisted names bind by coincidence.** The module's own header can be arbitrarily
wrong and nothing goes red. Scope the closure to the module's own declared edges, which is exactly
what emitting it alone does, and every definer drops at once: twenty-six diagnostics from one root.

**Green compile-clean and emittable-standalone are different properties, and the first was being
read as the second.** No measurement was wrong. The comparands were never checked against the
question.

### Why it belongs at the end of this brief

It supplies the **positive form** that the rest of the entries only approach negatively. The other
traps are recognised by their symptoms — a green that cannot go red, an empty that cannot be
distinguished from truncated, a true that is the same byte for two questions. This one names the
general defect directly: *a property that holds of an aggregate is being cited as a property of a
member*, and the aggregate supplies from elsewhere exactly what the member is missing.

That framing makes the whole class searchable, because the question it licenses is mechanical:
**what is the instrument supplying that the subject would have to supply for itself?** For
compile-clean it is the pool. For the stale binary it is the compiler's identity. For the capped
list it is the unseen 3865 rows. For two lanes reading two refs it is the branch the ruling names.
In every case the instrument was making up a difference the subject could not, and reporting a
result that read as the subject's own.

**And the corroboration arrived from the opposite direction the same day.** crisp-crab-430 traced
the namespace cut's two remaining blockers to `src/v1/04_infer.dag` pulling `List` by pool-fallback
and resolving to the `FreeMonoid` alias instead of the kernel `List` that imports used to shadow.
Same mechanism, worse symptom — the name finds the *wrong* definer rather than none, so it resolves
and continues. A refusal announces itself; a wrong-definer binding does not.

## The repair for a claim on the wrong subject is to MOVE it, not to soften it

*Three authors, three rounds, one afternoon, on the cost forecast for the namespace cut. Recorded
because the correction pattern is sharper than any of the three claims.*

    round 1  smart-ram-730   "pool_parse's frequency changes by a large factor"    right shape, wrong term
    round 2  deep-ant-102    "pool_bare_census memoizes -> a step function to       right memo,  wrong subject
                              certainty, not a multiplication"
    round 3  witty-lark-109  the multiplier is on bare_eligible                     correct

**Round 1** attached a real forecast to `pool_parse`. **Round 2** checked the memo — correctly — and
concluded there was no multiplier, only a step function. **Round 3** checked the producer and found
both wrong, in different ways:

- *"Today pays zero" is backwards.* The **scoped** census forces the whole-corpus parse, and it does
  so unconditionally. Both calls live in one function — `v1_compiler.cli_run`
  `bare_reference_pull_paths_for_source` — with `tree_bare_census_for_root` at its **head** and the
  `pool_bare_census` fallback **inside the resolve loop below it**, so the census runs before any
  name is looked up at all. A process whose names all resolve in the scoped census pays the parse in
  full. The receipt is an ordinary run: `bare_eligible=699`, `tree_census_misses=2` — scoped
  hits throughout, parse paid anyway.
- *The memo is real but bounds a different term* — `pool_bare_census`'s own whole-pool symbol index,
  the largest of the three and the only one no successful resolve pays for.
- *The multiplier exists on a third term nobody had named.* Bare-eligibility is the **negation** of
  `source_declares_import_lines`, and the cut deletes exactly those lines corpus-wide — so
  `bare_eligible` goes from 699 toward the whole corpus. **It needs no post-cut estimate, because the
  population is defined as the absence of the syntax being removed.**

### The lesson, which is round 3's and not mine

> **Deleting a true shape to escape a wrong term loses more than the error did.**

Round 2's correction was *locally* right — the memo does bound what it was pointed at — and it
destroyed a true forecast in passing. The instinct that a cost was about to change was correct from
the first message; only its subject was wrong. **The repair for a claim attached to the wrong subject
is to move it, not to soften it**, and "soften it" is the tempting move precisely because it feels
like rigour: hedging reads as caution while it is quietly discarding signal.

**Nobody measured anything incorrectly at any point in this thread.** Every number in all three
rounds was accurate. Each round, the comparand moved and the claim did not — which is this brief's
whole subject, now demonstrated three times inside a single conversation *about* this brief's
subject, by three people who had all read it.

That last fact is the strongest argument in this document. The class does not yield to knowing about
it.

## The class is in the escalation channel itself, in both directions

*Established jointly with deep-ant-102 over the afternoon, from opposite ends of the same tool.
Recorded last because it is the one instance that degrades every other finding in this brief.*

Everything above is about instruments that report on the *code*. This one is about the channel used
to escalate what those instruments find:

    a SEND can fail silently   429, client timeout, no token charged
                               -> indistinguishable from success at the caller
    a READ can fail silently   exit 0, empty payload
                               -> indistinguishable from "the thread is empty"

**Both failure modes were hit today, by both parties, and neither announced itself.** A message
believed delivered was confirmed only because a *subsequent* send returned `429: wait 241s` — the
rate limiter was the sole positive receipt available, and it arrived by accident. In the other
direction a read returned exit 0 with nothing, which cannot distinguish between *no reply exists*,
*a reply exists and the read failed*, and *a truncated success*.

**The only positive confirmations that exist on this channel** are the token counter for sends and a
non-empty rendered payload for reads. Neither is an acknowledgement, and neither is checked by
default.

### Why this outranks the rest of the brief

Every other entry describes a claim that could be wrong. **This one describes the medium those claims
travel through** — so a silent failure here does not produce a wrong belief about the code, it
produces a wrong belief about *what has been communicated and agreed*. Two specific errors today came
from exactly that: both of us treated a stale thread as a silent operator, and each of us reported
findings as "raised" that were only ever *sent*.

The distinction that closed it is worth keeping as a phrasing rule, and it is deep-ant's:

> **My read failed.** That does not establish that the reply is unreadable, that the thread is
> broken, or that the operator said nothing.

Reporting **"empty"** rather than reporting **nothing** is the whole discipline. An empty result is a
statement about the reader; silence is a statement about the world; and a channel that cannot
distinguish them will convert the first into the second every time, unless the person holding it
refuses to.

### The correction that closes it: the read was SLOW, not broken

**Both of us concluded the read path was gone. It was not — it is timeout-sensitive.** A 300s read
returned empty; a 540s read on the same thread returned 6879 bytes of rendered content. So the
failing reads were **timeouts reported as emptiness**, and `timeout 540` recovers the capability.

**This is the entry's own class, one level up, and it caught the two people writing the entry.**
Every earlier empty result was a true statement about a reader that had given up, and both of us read
it as a statement about the channel — the exact conversion the section above warns against, committed
while documenting it. I had additionally broadcast "side-chat reads are down" as a status fact.

**What each observation actually licensed:**

    exit 0, empty at 300s   ->   "my read did not complete"        <- all it ever supported
    what we concluded       ->   "the read path is gone"           <- a claim about the channel
    what was true           ->   "the read needs a longer timeout" <- neither of us tested

**Nobody tried a longer timeout before concluding**, and that is the whole failure: an empty result
was treated as terminal when it was a *parameter* of the call. The distinguishing test cost one
command.

**Keep the phrasing rule; it survives intact and is what made the correction possible.** deep-ant
reported *"my read returned empty"* rather than *"reads are broken"*, explicitly noting that an empty
payload at exit 0 cannot distinguish *no reply exists* from *the read failed* from *a truncated
success*. Having stated it that way, retrying at a longer timeout was the obvious next move. **The
weaker report is what left room for the stronger answer** — had it been filed as "the channel is
down", nobody would have retried it.

### The residual, stated because it is not fixed

The silent-failure shape is unrepaired: there is still no delivery receipt and no read-vs-empty
discriminator, and a send can still fail silently (429, client timeout, no token charged). What
changed is only that one *cause* of empty reads turned out to be recoverable. The positive
confirmations remain the token counter for sends and a non-empty rendered payload for reads —
neither is an acknowledgement, and neither is checked by default.

## Verification does not generalise

*My own, caught in review (`review 55425`) on gunbc#9113 — the document reporting the loader finding.
Its front matter read:*

> **Status of every claim: verified by reading the live tree at `bd84f669681`.**

That was **true of the mechanism claims and false of the measurement figures.** I had verified three
loader facts line by line — what the fallback arm does, what `pool_bare_census` is built over, what
`[floor-bare-name-ambiguity]` actually reports — and then let one sentence extend that standing over
`0 PoolAmbiguous` and `37389 scoped versus 733 pool-fallback`, which were **lane reports carrying no
run, artifact, or repository receipt** and unverifiable from the tree I had named.

The reviewer's phrasing of the harm is the precise one: those figures *"cannot be verified from the
cited tree and are used to motivate the requested work."*

### The rule, which is deep-ant-102's

> **Standing earned on three checked facts does not extend to a fourth unchecked one in the same
> sentence.**

Verification attaches to *assertions*, not to documents, authors, or paragraphs. A blanket status
line is a **quantifier over a set the reader cannot see** — and the moment one member of that set was
taken on trust, the line became a stronger evidentiary claim than the evidence supports, which is
exactly what §4b(1) forbids.

The comparand that moved here was neither a tree nor a number. It was **which assertions the word
"verified" ranged over.**

### Two things about the repair, because the tempting fix was the wrong one

**I could have gone and re-measured the two figures and cited my own run.** That would have removed
the reviewer's objection and been the worse outcome: the document's entire ask is *nobody has this
measurement, here is the one line that would produce it*, so quietly producing a partial version
would have blurred the request into a half-answer. **Narrowing the claim is the honest repair;
generating a receipt to rescue a sentence is the other kind.**

**And the narrowed header had to demonstrate rather than assert its own irrelevance.** It now states
that if every reported figure is wrong, the mechanism finding and the ask are unaffected — and, more
usefully, that a wrong `0 PoolAmbiguous` would *strengthen* the argument, since the whole point of
that section is that the counter **cannot report anything else for this class**. A narrowed claim
that still needs its dropped evidence has not been narrowed, only hedged.

*Kept because it happened in the front matter of a document about instruments answering narrower
questions than they were asked, written by someone who had spent the day cataloguing the class, and
was caught by a reviewer rather than by its author.*

## The predicate with no domain — and the two authors who applied it to everyone but themselves

*The afternoon's capstone, and the one entry whose instances are all the people writing this document.*

A correct rule was issued: **no PR may land between the prerequisite and the namespace cut if it
alters the cut's conflict set.** Unambiguous, operator-ruled, with a decidable test — does this PR's
file set intersect the cut's.

**It named its predicate and not its DOMAIN.** It said which PRs must hold without saying over what
set to evaluate that — so the domain silently defaulted to **"the PRs someone happened to mention."**

    largest list anyone had named        6
    one subtree, rebuilt from gh pr list 7 ready, 6 intersecting
    computed across all open PRs        41 of 69 intersecting

The six were real. They were also 15% of the population, and the gap was invisible because every PR
in it was individually fine — approved, green, mergeable. **Nothing was wrong with any of them
except that no one had evaluated the predicate against them.**

### Then both authors of the census found their own PRs on it

One session published the intersecting list. The other read it and found **two of their own PRs on
it**, having spent the afternoon telling six other sessions that ready-and-intersecting means hold —
they had read their own PRs' *status field* and run the intersection test only on other people's.

Ten minutes later the session that **built** the list found **their own PR at row 41 of it**, which
they had written and not noticed.

**Both PRs turned out to touch the same file: `dag/gunbc/design_document.dag`, the DESIGN.md
authority.** Two people, one authority, each believing only the other's PR was in scope.

### The near-miss inside the near-miss

Reasoning from that collision, one of them asserted a mechanism: *the two PRs merge cleanly and the
generated DESIGN.md silently loses a row, with the drift gates removed by the floor cut.* The other
adopted it and called it the strongest available argument for restoring the gate.

**It was false, and nobody had run the merge.** One `grep '^@@'` on each diff settles it:

    #9002   @@ -137,7 +137,7 @@ DESIGN.md   |  @@ -168,7 +168,7 @@ design_document.dag
    #9098   @@ -137,7 +137,7 @@ DESIGN.md   |  @@ -168,7 +168,7 @@ design_document.dag

Byte-identical. Both edited the same long paragraph, so the second to land goes CONFLICTING under any
merge algorithm. **The author of the claim was the instrument that answered a narrower question than
it was asked** — "could this in principle be silent" rather than "is *this one* silent."

What survives is better than what was claimed, and stronger for being honest: **the case that would
have been silent is one line-number away from the case that was loud.** Different lines in the same
authority merge cleanly while the committed artifact is neither side's regeneration of the merged
authority — untested by anyone, with the near-miss as the only evidence.

### The ownership column, one field over

    author = briansrls    on every PR in the repository

Ownership had been read off `author`, which is the human's account and identical everywhere. **Not
unreliable — deterministically wrong.** Unreliable would be *better*: it eventually disagrees with
itself and someone notices. A constant wrong answer survives every consistency check anyone would
think to run, which is why two senders misrouted the same session's PRs to the same wrong recipient
without either guessing badly. Ownership derives from `headRefName` and nothing else.

**A census with a wrong ownership column is a correct set delivered to the wrong people** — the same
failure as an uncomputed domain, one field over.

### What actually fixed it

Not a better rule. **A computed domain, and holds that live on the artifact:**

- the intersection recomputed per PR (one API call each, ~2 minutes for 69)
- an explicit HOLD comment on all 41, carrying the cap and the `>=` inline so the rows cannot travel
  without them — because **the merge hand reads the PR, not the thread**
- the two self-caught PRs converted to **draft**, making the hold structural rather than readable

And two limits kept attached, because both are the difference between a list and a false clearance:

> **Sound for holding, unsound for releasing.** An intersection found is an intersection; a zero
> means "none among the 3000 of 3965 fetched." A PR overlapping only in the 965 unfetched reads as
> *actively safe* by sitting on a list someone takes as cleared.

> **"Computed" reads as "solved" and it is only "solved as of a timestamp."** Every PR opened after
> the census starts unheld.

### Why this closes the brief

Every other entry is an instrument that misled its reader. This one is a **rule** that misled the
people enforcing it — and it caught both of them, on the same file, inside an hour, while they were
actively cataloguing the class.

**A rule whose domain is uncomputed is not a weak rule. It is a rule with a silent exemption list,
and the people most confident it is being applied are the ones most likely to be on it.**

## The fix was more dangerous than the defect: a wrong query outranks a wrong list

*Immediately after the entry above, and by its own authors. This is the one to read if you read only
two.*

The previous entry ends by replacing a remembered list of held PRs with a **computed query** — *every
open PR carrying a HOLD comment* — on the reasoning that a list drifts and a query cannot.

**The query returned 11 of 41. Nothing said so.**

    my pattern  startswith("## HOLD")   limit 200   ->  13
    corrected   test("^#*\s*HOLD")      limit 200   ->  41   (matches the census exactly)
    corrected   test("^#*\s*HOLD")      DEFAULT     ->  11

**Two independent silent narrows, composing.**

1. **The pattern.** My holds begin `## HOLD`; the other author's begin `HOLD — do not merge…` with no
   `##`. Two spellings of one concept, and `startswith("## HOLD")` cannot distinguish *no holds here*
   from *holds spelled the other way*.
2. **The default limit.** `gh pr list` caps at 30 without being asked and without saying so — the same
   silent truncation as `gh pr view --json files` at 100, in a different command, hit again by people
   who had documented the first one that morning.

Either alone under-reports. Together they returned **27%** of the domain with the confident shape of a
complete answer.

### Why this is worse than the list it replaced

**A wrong list is obviously incomplete. A wrong query looks authoritative.**

Had we acted on it, it would have released 11 PRs and left **30 frozen permanently** — and the freeze
would have been invisible, because the artifact that would have shown the omission is the very query
that omitted them. A list at least invites the question *is anything missing?*; a query answers that
question, wrongly, and is believed.

**The generalisation is uncomfortable and it is the point:** *"stop maintaining the set, recompute
it"* is correct and is not sufficient. Recomputation moves the failure from **staleness**, which is
loud and expected, to **an under-specified predicate**, which is silent and trusted. The computed
domain still has a domain — the pattern, the limit, the corpus the tool consents to search — and none
of those announce themselves.

### The practice that caught the second half

The same exchange corrected a related claim of mine. I had written that one hold rationale — *do not
move a file under a running measurement* — survives independently and reaches **#9026 alone**. True of
my 13. Across all 41 it reaches **seven**: `9079 9075 9028 9026 9024 9007 8974`.

It was caught for one reason, in the other author's words:

> I would not have caught that if you had not stated your number **with its denominator attached** —
> *"on my 13"* is what made the gap visible.

**A count carrying its denominator is falsifiable by anyone holding a larger one.** The same count
stated bare — *"it reaches #9026 alone"* — is not wrong so much as **unaskable**, because nothing in
it invites the question *of what?* That is the cheapest habit in this entire document and the only one
that has repeatedly caught errors before they cost anything.

---

## The self-testing capability: when the instrument and the subject are the same object

The last specimen of the day is the only one where the comparand that was substituted is not a
subject at all. It is **the moment**.

I told my manager, as a present-tense fact, *"my sends work, only my reads are gone."* What I
actually held was two past observations — one send confirmed by a token counter dropping, one by a
later rate-limit refusal — and no mechanism connecting either to the present. The next send hung
for five hundred seconds and returned nothing.

**Nothing looked wrong, and that is the point.** Every other entry in this document substitutes one
subject for a neighbouring one, and the substitution is visible once you name both. Here the subject
is genuinely identical: same command, same session, same channel, same account. The only comparand
that moved was **time**, and *"worked at 12:52"* is not *"works now"* — a capability that held twice
is not a capability that holds. There is no join to check, no denominator to state, no second list
to diff. The habit that catches the rest of the document does not fire on this one at all.

### The sharper half: the retry was the probe

The recovery move made it worse in a way worth stating separately, because it is a **shape** rather
than an incident.

I could not determine whether the hung send had landed. The only available test was **to send
again** — so the instrument and the subject were the same object. That leaves exactly three
outcomes, and only one of them is informative:

    fast rate-limit refusal   ->  the first one landed          (informative)
    clean success             ->  the first one did not         (informative)
    a hang                    ->  neither, and now there may be two

I got the third. Eight minutes spent, no information gained, and the population of possibly-sent
messages went from one to two — which is the failure mode to actually fear here, because the
correction for *"I am not sure it arrived"* is to send it five times, and then discover that three
did.

**The general form: any capability whose only test is its own exercise cannot report its own state.**
It has no observer that is independent of it, so a failure to answer is indistinguishable from a
failure to ask. This is not a property of one flaky channel. It is the condition that makes
*"verify before asserting"* — the instruction every other entry here resolves to — **unexecutable**,
and it is the one case in this document where the standing advice has no purchase.

What is left when the advice does not apply is not a better probe but a **narrower claim**:

    do not say   the channel works
    say          two messages landed earlier; the most recent is unknown and I cannot resolve it

The second sentence is worth more than the first even though it promises less, because a reader can
plan against it. The manager receiving it did exactly that — routed around me rather than waiting on
me — which is the whole return on the correction.

### The symmetry, recorded because both of us made it inside the hour

My manager, reading the correction, found the same error pointing the other way in their own log:
they had recorded having informed the operator about my channel, and had not — their message was
entirely about a different subject. They caught it only by **re-reading what they had actually
sent** rather than what they remembered sending.

    I asserted a capability I had not verified holding.
    They asserted an action they had not verified taking.

Same hour, same class, opposite halves. Neither was caught by a check; both were caught by someone
else's correction forcing a re-read of the primary artifact. **That is the honest summary of this
entire document: the mechanism that found nearly every specimen in it was a second party re-reading
the source, and no instrument in the repository substitutes for it.**

---

## The marker that exists before the run: counting a string in a log that also contains the command

Two sessions were caught by this within an hour, from opposite ends, and it is the most mechanical
trap in this document — no judgement required to avoid it, only knowing it exists.

`ctrl-build` **echoes the dispatched script into the log** before executing it. So any marker you
grep for is present in the log **at time zero**, put there by the echo rather than by the run:

    grep -c "cost-partition" run.log   ->  2

Which reads as *the line landed twice*. Both matches were the echoed script. The payload emitted
nothing at all — the run was killed at 45 minutes without producing a single partition line.

### Why it is worse than a wrong count

A miscount is recoverable; you notice the number is odd and look. **The severe form is a watcher
anchored on presence**, because the marker is there *before anything starts*:

    wait until log contains "DONE"   ->  fires immediately, on the echo
                                     ->  reports a run as settled that has not begun

That is not a delayed wrong answer, it is an **instantaneous** one, and it arrives during the window
where the correct answer is *"no information yet."* The same session lost an hour to a watcher firing
on its own end-marker in the echoed script.

### The rule

**A marker must be something only a real run could emit.** A literal in the script fails that by
construction — the script is data that reaches the log intact. So the marker has to carry a value the
run *computes*:

    bad    echo "DONE"                       a literal; present in the echo
    good   echo "DONE files=$(wc -l < out)"  carries a computed value
    good   echo "EXIT=$?"                    carries an outcome
    good   anchor on a timestamp the run stamps, not a word the script contains

Failing that, match with an anchor the echo cannot satisfy — line-start on a line the echo indents,
or a count that must *exceed* the number of times the string appears in the script itself. The second
is worth stating because it is the honest general form: **when the instrument's input is visible in
the instrument's output, every measurement has a floor contributed by the input**, and you either
subtract that floor or make the marker unforgeable by it.

### Its neighbour, from the same failure

The run that produced the miscount also could not say whether it was slow or hung. **The harness
emits its cost partition only on termination**, so a 45-minute silence and a deadlock are
byte-identical from outside — the only evidence of progress is the artifact that finishing produces.

That is the self-testing shape from the previous entry, and this one is the **better** case, because
here the repair exists and is cheap: emit per-phase progress markers so that liveness has a signal
distinct from completion. The narrowed re-dispatch does exactly that — one root at a time, each
capped, printing begin/exit stamps per root — which converts *"the run failed"* into *"this root
failed"*, and makes the timeout informative instead of merely negative.

**Where the two entries meet:** an instrument that reports only at the end cannot report on itself,
and an instrument whose marker is a literal reports before it starts. **Both produce a confident
reading during the interval when nothing is known** — one by silence that looks like a value, one by
a value that precedes the measurement.

### The generalisation, which is larger than logs

The framing that makes the previous entry a class rather than a `ctrl-build` quirk, in the words of
the session that hit it:

> What made the echo dangerous is not that the marker appeared. It is that **it appeared at time
> zero**. Presence-based watchers all have a defined answer before the subject exists.

**Any check whose predicate is satisfiable by the setup of the experiment is green before the
experiment.** The log echo is one instance; the shape does not need a log:

    a file the harness creates empty, checked for existence
    a table the migration creates, checked for presence rather than contents
    a metric registered at startup at zero, checked for "is it reported"
    a marker string in a script, checked by grep against the script's own output

In every case the check passes at t=0, and the interval where it is wrong is exactly the interval you
built it to observe. **This is the decoration failure from DESIGN §4b arriving on a timeline instead
of in a corpus** — not a check whose RED is unauthorable, but one whose GREEN precedes its subject.
The repair is the same in both: make the passing state something only the real event could produce.

### A specimen of the same trap in a different instrument, caught before it was reported

Checking whether a commit was present in a branch's history:

    git log --oneline <head> | grep -ci "9090"        ->  1        reads as PRESENT
    git log --oneline <head> | grep -E "\(#9090\)"    ->  nothing  actually ABSENT

The single match was **`e19090e107` — a commit hash containing `9090` as a substring**, on an
unrelated PR. **A bare PR-number grep over `git log` has a false-positive rate nobody accounts for,
because hashes are hex and PR numbers are decimal digits that occur inside them.** Every short hash
in the log is a lottery ticket against every PR number you might search for, and the log is long.

What makes it worth recording is the counterfactual: reporting the first number would have said a
cost row *was* present and therefore that three separate measurements *were* comparable — the exact
unrecoverable error the sibling session had written a message an hour earlier to prevent. **The
instrument would have been used to confirm the very claim it was built to refute.**

The rule is the delimiter, not the number: match `(#9090)`, never `9090`. Same shape as the marker
rule above — **the substring is satisfiable by things that are not the subject**, and narrowing the
pattern until only the subject can satisfy it is the construction move. Anchoring (`^RUN_EXIT=`) is
the validation move and remains defeatable, because an echoed script can begin a line that way too.

### A first-person receipt for this entry, collected while writing it

Within the hour of recording the rule above, the author hit it from the other side. A read was
dispatched to the background and the harness reported:

    Background command "... dashboard-ops side-chat 2 > /tmp/sc9.txt ..." completed (exit code 0)

**The payload had failed.** The file it wrote:

    RC=124
    0 /tmp/sc9.txt

`124` is `timeout`'s signal that the command was killed at its limit, and the output was empty. The
reported `exit code 0` belonged to the **wrapper** — the `echo` and `wc` that ran *after* the timeout
and succeeded at printing the failure. **The status was honest about a narrower subject than the
notification's phrasing invited**: "completed" was true of the shell; nothing about it spoke to
whether the read returned.

Two properties worth extracting, because the specimen is unusually clean:

- **The failure was legible only because the payload wrote its own status into the file.** `RC=$?`
  is exactly the marker rule above — a value the run computes, not a literal — and it is the only
  reason the wrapper's green was catchable. Without it, `completed (exit code 0)` and a successful
  read are the same artifact.
- **The last thing in the pipeline determines the reported status**, so appending any reporting step
  after a failing one converts a failure into a success. `cmd; echo done` exits 0 whatever `cmd` did.
  This is the same shape as the log-echo trap: the instrument's own machinery contributing the
  evidence that the instrument reads.

Recorded here rather than paraphrased because the counterfactual is exact: the notification was read,
the green was **not** believed, the file was opened, and the failure was found. The habit that did
that is the one this document keeps arriving at — **do not read the status, read something only the
subject could have produced.**

---

## Two ways to be confidently wrong from real source: the scoping error and the sampling error

These arrived within an hour of each other, from two sessions reading the same file, and the pair is
worth more than either — because they look identical in the transcript and have **different
remedies**.

Both sessions were tracing whether a profile lookup misses for a carrier spelled `FreeMonoid`.

### The scoping error — a row is not the map that contains it

    cited:   `"FreeMonoid": "FreeMonoid"` is in container_template_alias_rows
    actual:  it is in container_template_algebra_rows — a DIFFERENT map, 14 rows to the other's 6

Found by `grep 'FreeMonoid' types.dag`, which returns matching lines with **no indication of which
declaration they belong to**. The line was real, the quote was exact, and the container was never
checked.

**This is the same error as verifying a code snippet without checking its enclosing function** — the
same session had done exactly that earlier the same day, with a `git log`-visible offset attached
that made it look *more* verified. **The remedy is: resolve the containing scope, not the match.**
`grep` is a line instrument being asked a containment question.

### The sampling error — a map's first row is not the map

The other session made a **different** mistake with the same outcome. They bound the function to the
**right** map — grepped `container_template_algebra`, saw `map_get(container_template_algebra_rows,
name)`, correct — and then asserted that map's *contents* from the one line of it visible on screen:

> "rows are keyed by SPELLING: `List` -> `FreeMonoid`"

**True of the rows they had seen. False of the map**, which also carries `"FreeMonoid":
"FreeMonoid"` and the other algebra-name identity keys. The name resolution was right; the **payload
was filled in from a partial view**, and the partial view was consistent with the conclusion, so
nothing felt wrong.

**The remedy is different: print the whole thing.** One command settled it — both maps enumerated,
sizes compared, subset relation checked. It was not run, because the visible rows already agreed
with the expected answer.

### Why separating them matters

    scoping   the match was real; the CONTAINER was assumed        -> resolve the container
    sampling  the container was right; the CONTENTS were assumed   -> enumerate the contents

**Neither is carelessness and neither is caught by re-reading your own work**, because in both cases
what you looked at was true. They are caught by a second party tracing the same path, which is what
happened here in both directions inside an hour. **The confident-wrong-answer-from-real-source class
has at least these two members, and a reviewer who knows only one of them will miss the other.**

### The defect the argument uncovered, which outlived both mistakes

Printing the maps produced a finding neither session was looking for:

    alias   = 6 rows    List Map Set list map set
    algebra = 14 rows   those six PLUS BooleanAlgebra FreeMonoid PartialFunction PointwisePower
                        and their snake_case forms
    alias is a strict subset of algebra          True
    values agree on every shared key             True

So the six-row map is **exactly the fourteen-row map minus its algebra-name keys**, with identical
values everywhere they overlap. **Its entire semantic content is the subset relation** — it exists to
answer *"is this a surface spelling rather than an algebra name,"* which is a question about what the
**other** map contains.

That is a single-authority defect in a form worth naming separately: **not two authorities
disagreeing, but one authority whose whole content is a statement about another, spelled as an
independent table.** Two independent readers, both looking directly at the source, both wrong within
an hour, is the measured cost — and it is a better argument for a canonical authority than either
mechanism the sessions were chasing.

**Both mechanisms they were chasing turned out to be dead.** The inverse *is* reachable
(`container_kind_canonical("FreeMonoid")` → `"List"`, so the profile **hits**), which killed one
session's proposed defect and the other's hypothesis in the same message. **The durable output of the
exchange was the thing found while checking, not the thing being checked.**

---

## Two more from the same hour, both in tooling the author wrote to avoid the first two

A notification job — deliver "your work is unblocked" to eleven sessions, paced against a rate
limit — produced two specimens in ten minutes. Both are entries already in this document, committed
by someone who had just written them.

### `pgrep -f` matches the process asking the question

Checking whether the job was still alive:

    pgrep -f 'relbody|rel2' >/dev/null && echo "STILL RUNNING" || echo "none running"
    -> STILL RUNNING

**Nothing was running.** `pgrep -f` matches against full command lines, and the command line
*containing the pattern* is the shell running the check. The instrument matched itself.

This is the log-echo trap exactly — **the instrument's own input appearing in the space it
searches** — and the same repair applies: the pattern must be something only the subject could
satisfy. `ps -eo args | grep foo | grep -v grep` is the folk remedy, and it works for the same reason
the marker rule does: it removes the searcher from the searched set. Better still, have the job write
its own liveness (a pid file, a heartbeat line) rather than inferring it from a process table that
contains the question.

**Cost here: a false "still running" that nearly led to a duplicate job being launched alongside a
dead one** — which would have double-sent to sessions already notified.

### A retry loop that treats a permanent refusal as a transient one

The job's failure arm backed off and retried on *any* non-success:

    for try in 1..5; do
      out=$(send ...)
      case "$out" in *delivered*) ok; return;; esac
      sleep 70
    done

Two retries burned before anyone looked at `$out`. The actual response:

    HTTP 400: {"error":"recipient session not found: sleek-fox-685"}

**The session did not exist.** No amount of waiting was going to change that, and the loop would have
spent five minutes to report `FAIL` — a word that reads identically to a rate-limit failure and
would have sent the operator looking for a budget problem that was not there.

**This is the absorbing fallback, in the author's own tooling, in the same afternoon the author
reviewed a PR for it.** The failure arm *widened* — retry, wait, retry — instead of refusing.
The deficit's frequency was zeroed: a nonexistent recipient and a throttled one produced the same
observable, so the distinction that mattered could not surface.

**The repair is a typed arm per cause**, and it is three lines:

    *'"delivered":true'*            -> OK       (succeeded)
    *'recipient session not found'* -> GONE     (permanent; author unreachable, report it)
    *'429'*/budget                  -> RETRY    (transient; back off)

The `GONE` arm is the one that carries information: **it says an author exists who cannot be
reached**, which is a fact the coverage report must state rather than absorb into a retry count. With
the arm present, the job reports *"delivered 10, unreachable 1"*; without it, *"delivered 10, failed
1"* — and only the first tells anyone that a lane is stranded with no channel to it.

**Both defects were in code written specifically to deliver corrections about undelivered
corrections.** That is not irony worth savouring; it is the measurement this document keeps taking —
**knowing a class does not stop you instantiating it**, because the trap is never labelled at the
moment you commit it. What changes outcomes is not knowledge but the habit of opening the artifact:
`ps` instead of `pgrep`, `$out` instead of the exit status, the map instead of the row.

---

## A count that answers "ever" while the reader asks "now" — and a join applied in only one direction

The readiness payload for a pull request reports, among other fields:

    head_sha           f75fb625217
    approval_count     1  ['claude']

    55428  claude  approve          sha=c6584caf3   STALE
    55358  claude  approve          sha=223397bd8   STALE
    55351  claude  approve          sha=06b7e7ebd   STALE
    55349  codex   request_changes  sha=b8ecca0b2   STALE
    55314  claude  approve          sha=f12d1dbb2   STALE

**Four approvals, none of them on the head, and the count says one.** The field appears to answer
*"has a distinct provider ever approved this PR"* while every reader consults it to answer *"is this
head approved."* Those questions coincide exactly until someone pushes, and diverge silently
thereafter — no error, no staleness marker on the number itself, no signal that the head moved.

### The asymmetry is the finding, not the lag

The same payload reports `stale_providers: ['codex']` and **correctly discounts the
REQUEST_CHANGES** as stale, because that review's sha is not the head.

**So the join exists.** The machinery for comparing a review's sha against the head is present,
implemented, and demonstrably working — **applied to refusals and not to approvals.** That is worse
than an absent feature, because absence is uniform and this is directional:

    stale REQUEST_CHANGES  ->  head-joined, discounted   -> removes a blocker
    stale APPROVE          ->  not joined, still counted -> preserves a green

**Both defaults favour merging.** No single decision here is unreasonable on its own; the pair is
what produces a readiness verdict biased in one direction, and neither half looks wrong in
isolation. A reviewer auditing the refusal path would find a correct head join and conclude the
payload is head-aware.

### The instance where it was RIGHT, which is the part worth keeping

This document's author read that field at ~15:00, saw `approval_count: 1` with `ready: true`, and
escalated the PR as unblocked. **At that moment the counted approval was genuinely on the head** —
the claim was true when made. The head moved afterwards.

So the report was accurate, then became false, **and nothing about the field could distinguish the
two states.** It is the same shape as reporting a capability as working because it worked twice:
**the comparand is time**, and a bare count cannot carry a head. The number is identical in the
sound case and the unsound one.

### The rule

**Never quote an approval count without joining `reviews[].sha` against the head and reporting the
head-bound number.** If the two differ, report both. A count is not an answer to a question about a
specific revision unless it carries that revision — and a field that silently answers the more
permissive of two readings is exactly the instrument this document keeps finding: honest about a
narrower question than anyone asks of it.

**The operator's own phrasing anticipated this** — a ruling issued the same day required the PR to
obtain *a head-bound approval*, with an explicit warning not to merge an uncomposed head "just
because old checks are green." That qualifier reads as pedantic until you see the payload; the two
specimens behind this entry are why it was necessary.

---

## The capstone: in a self-hosted compiler, the tool is a variable too

Everything above concerns an instrument answering a narrower question than the reader asks. This is
the same failure with the largest blast radius the day produced, and it took four sessions an
afternoon to notice.

A branch's required run refused with **71 hard diagnostics**, and its floor produced **361 lines**.
Four lanes bucketed the symptoms into families and authored mechanism theories: a carrier/profile
lookup miss, a missing inverse in an alias table, a lost type argument, a gap in a call-target
vocabulary, a changed arm selection in expression inference. **Every theory was internally
coherent.** Several were refuted by execution; the survivors were refuted by reading. Nothing fit.

Then one dispatch settled it. The compiler was rebuilt from the branch's exact merge-base, **the same
sources** were checked out, and the run repeated:

    seeded from the branch's own earlier binary   71 hard diagnostics
    seeded from a merge-base-built binary          ZERO

**Same sources. Different compiler. The refusals had no source cause at all.** A binary built from an
earlier state of that branch had been regenerating a mirror that carried a defect forward, and every
diagnostic under investigation was that defect's output.

### The reasoning error, which was valid and still wrong

The attribution had been declared *settled* on this argument:

    main is green at the merge-base
    the branch is merge-base + one delta
    therefore the delta causes the diagnostics

**The syllogism is valid. The premise was incomplete.** Two trees were compared while a second
variable moved silently — main's CI seeds its compiler *from main*; the branch seeded its compiler
*from itself*. **In a self-hosted repository the compiler is not a constant across a source
comparison**, and treating the source as the only variable is the one assumption that can never hold
there.

### The rule, and its scope — the scope matters as much as the rule

**"I changed the compiler and the compiler broke" has two readings, and nothing in the source
distinguishes them.** The discriminator is one dispatch: rebuild the tool from a known-good commit,
hold the sources fixed, re-run.

**But the test is whether the COMPILER WAS HELD FIXED ACROSS THE ARMS — not whether the claim
involves diagnostics:**

    CROSS-BINARY          tree A measured with binary A, tree B with binary B.
                          The tool moved alongside the variable under test.   DEAD.

    PAIRED WITHIN-BINARY  both arms measured with the SAME binary, one variable changed.
                          Contamination shifts both arms equally, so the DELTA survives
                          even when the absolutes are wrong.                  NOT KILLED.

A module refusing standalone and passing with its import header restored, **under one binary**, is
the second shape: for contamination to explain it, the contaminated compiler would have to refuse one
arm and accept the other *on exactly the axis under test*, which is the claimed mechanism rather than
an artifact of it.

**Second-order, and most likely to be skipped: a paired comparison is immune to a compiler that is
WRONG, not to one that is IRRELEVANT.** If the binary measuring both arms was itself built from a
contaminated mirror, the delta is real *for that compiler* and says nothing about the one the
repository ships. Such a result is stated as *"X changes behaviour in this compiler"*, never *"in
gunbc"* — and re-running the pair under a merge-base-built seed upgrades it only as far as **that
seed's revision** — "in the merge-base compiler" — never to "in gunbc" unconditionally. **A claim
generalises exactly as far as the provenance of the binary that produced it, and no further.**
Reaching "in the compiler this repository ships" requires the pair to be run under that compiler,
which is a different and usually unavailable control.

**Stating the rule without the scope would have been the next over-correction**, sending lanes to
retract paired comparisons that were never contaminated. The unscoped version was written first, by
the author of this document, and corrected by the session it was sent to.

### The asymmetry, which is the day's actual finding

Auditing every claim four sessions made against that boundary produced a clean split:

    SURVIVED every correction, untouched      facts established by READING SOURCE
      an alias declaration; a map's contents; which parameter a function takes;
      a law recorded in a module; the shape of a fall-through

    NEEDED retraction or narrowing            facts established by MEASUREMENT
      a diagnostic census; a population bound; a per-file cost; an attribution;
      a symptom taxonomy

**Every claim that had to be withdrawn came from a measurement. Every claim that survived came from
reading the source.**

This is not an argument against measuring — measurement answers questions reading cannot. It is that
**a measurement carries provenance obligations a source read does not**, and the two had been treated
as interchangeable evidence. A source read is reproducible by anyone with the tree, carries its own
context, and cannot be invalidated by a tool. A measurement depends on which binary, which base,
which head, which roots, run when — **and it reports a number either way**, with the same confidence
in both cases.

**The number does not know it is wrong.** That sentence covers every entry in this document.

---

## Why this class recurs: the checking artifact is one step further away than the reasoning

Every entry above is an instance. This is an attempt at the generating condition, and it comes from
two sessions comparing error logs at the end of the day and finding the same shape three times each.

**Session A's three:** a code snippet attributed to the wrong enclosing function; an attribution
declared *settled* from a syllogism whose premise omitted a second variable; a **size** argument
offered against a **structure** objection.

**Session B's three:** an emitter defect partitioned by POSITION when the wall's coverage is over
PRODUCERS; a fallback count quoted as bounding "the defect" when it bounded one of two paths; "194
files depend on this declaration" when the census had counted a **spelling**.

Six errors, two sessions, one afternoon, arrived at independently. **Every one was locally valid and
answered a neighbouring question.** And every one was **a single fetch from being right** — read the
enclosing function, ask which binary, open the ruling text, check what the wall ranges over, ask
whether the count is of names or of resolutions.

### The condition, stated as a claim rather than a moral

**The error appears when the artifact that would check a claim is one step further away than the
artifact the claim was reasoned from.**

Not further in difficulty — further in *steps*. In every case above, the reasoning input was already
open: a grep hit, a count, a review comment, a green run. The checking input required one more
action: resolve the container, name the binary, fetch the ruling, read the producer. **One step.**

That gap is small enough to be invisible and large enough to be skipped, and it is skipped
*precisely when a lane is moving well* — because the reasoning input is sufficient to produce a
confident, coherent, communicable claim. **Nothing about the moment feels like a gap.** The output is
fluent and internally consistent, which is exactly the artifact this document opened by warning
about.

### Why "be more careful" is the wrong remedy

Six instances by two competent sessions in one afternoon is not a diligence failure; it is what the
default produces. The remedy is not vigilance but **shortening the distance to the checking
artifact**, or making its absence loud:

    reasoning from a grep hit          -> the check is: what declaration contains it
    reasoning from a count             -> the check is: over what population, produced by what
    reasoning from a green run         -> the check is: which workflow, which binary, which head
    reasoning from a review's claim    -> the check is: the ruling text, not its summary
    reasoning from another lane's PR   -> the check is: ask that lane, do not infer from files

**Each is one command or one message.** The entries in this document are, almost without exception,
the record of that one action not being taken — and of a second party taking it later.

### The corollary that makes it operational

**A claim's confidence should be bounded by the distance to its checking artifact, not by the
coherence of the reasoning that produced it.** Coherence is available for free on the wrong premise;
it is not evidence and it never was. When the check is one step away and untaken, the honest form is
not the claim but the claim plus its unfetched premise: *"the snippet matches — I have not checked
what contains it."*

That sentence is longer, weaker, and would have prevented every entry in this document.

---

## The ruling: a measurement identity, and why a structure beat our rule

The bootstrap finding above was escalated, and the ruling that came back is better than the boundary
two sessions had negotiated. It is recorded here verbatim-where-quoted because it supersedes the
prose rule, not merely endorses it.

### The scope

> **A stop-the-line correction for cross-binary source attribution, not a blanket invalidation of
> paired experiments.**

Retired: the source-causal interpretation of the diagnostic census; the symptom partition derived
from it; and the five mechanism explanations **insofar as their only evidence was that output**. Also
retired — and this is the part a lane would have missed — **any queue ordering based on those
families or their apparent sizes.** A retraction that leaves the decisions the retracted evidence
drove is not a retraction.

### The sentence

> **The explanations can remain as hypotheses only if they have independent controls. Their internal
> coherence gives them no residual evidentiary weight.**

**That is the sharpest statement of this document's subject produced by anyone.** Five explanations
survived an afternoon of scrutiny by competent sessions, and every one was internally consistent,
mechanically plausible, and grounded in real code. **Coherence was doing all the work, and coherence
is available for free on a false premise.** It is not weak evidence; it is *no* evidence, and the
instinct to keep a well-argued hypothesis "on the list" after its observations are withdrawn is the
instinct this sentence forbids.

### The structure, which is the real correction

    M { source_tree, compiler_binary_identity, command_and_configuration,
        measured_subject, terminal_phase }

    "A SOURCE SHA ALONE IS NOT A MEASUREMENT IDENTITY."

With that shape, the two topologies are not a rule to remember — they are visible in the record:

    CROSS-BINARY DIAGONAL   M(source_A, compiler_A) vs M(source_B, compiler_B)
                            TWO fields differ. Inadmissible for source causation, by inspection.
    PAIRED WITHIN A BINARY  M(source_A, compiler_X) vs M(source_B, compiler_X)
                            ONE field differs. Internally causal.

**We wrote a rule about when a comparison is admissible. They wrote a structure in which the
inadmissible comparison is legible as two fields moving.** Ours requires a reader to remember and
apply it at the moment of temptation; theirs makes the defect a property of the record's shape.

**That is construction-over-validation applied to our own epistemics** — the preference this
repository's design doctrine states as its central move — and two sessions who had spent the day
citing that doctrine at each other reached for prose when the substrate move was available. **The
document you are reading is a catalogue of instruments answering narrower questions than they were
asked; it was written without noticing that its own central finding was being carried in a form that
could not enforce itself.**

### The postscript that belongs here rather than anywhere else

Within an hour of the section above being written, its author took a sibling session's report of a
capability failure — a state change the sibling had marked as certain, and which turned out to be a
byte-count read from a file that was still being written — generalised it from one session to three,
and delivered it to the one reader whose behaviour it would have changed.

**The checking artifact was not one step away. It arrived unbidden, within minutes, in both
directions.** Neither session waited for it.

**Knowing the class does not confer immunity to it.** That is not a rueful observation; it is the
argument for the structural move over the prose one, made by the two people who had just written the
prose.

## A second postscript, one day later: two clocks on one object

The postscript above says knowing the class confers no immunity. Here is the receipt, in the same
hand, twenty-four hours later, and it is worth recording because it is the cheapest specimen in this
document — the whole error and its correction fit inside eleven minutes and four commands.

### What happened

A subtree digest carried a child's title beginning `MAIN RED`. Checking it rather than filing it was
correct, and the check found something real: main's last *completed* witness run was fourteen hours
old, and the run on the current head was `in_progress`.

The arithmetic was `now - run.createdAt = 142 minutes`, against the ~30 minute floor figure DESIGN.md
records. A 4.7x overrun. That went to the parent session as a report, hedged carefully about *cause*
— explicitly refusing to attribute the stall to the commit under test, naming that refusal as the
correlation this fleet keeps mistaking for a cause — and carrying a suggested next step: pull the log,
find the phase that is not returning, and note that the per-witness eval deadline is on DESIGN.md's
unguarded list, so an unbounded witness is exactly what that rung drop stopped catching.

Every hedge in that message was about the *cause*. None was about the *measurement*.

### The measurement

`run.createdAt` was `15:14:04Z`. The job's own `started_at` was `16:49:56Z`. Ninety-five minutes of
the 142 was **queue** — waiting for a runner. Execution was 44 minutes, sitting in a step that now
carries three phases against a figure recorded for one.

There was no overrun. There was probably nothing wrong at all.

### The shape, which is this document's subject exactly

Two timestamps hang off one object and answer different questions:

- `run.createdAt` answers **when was this work requested**
- `job.started_at` answers **when did this work begin executing**

The reader's question was *how long has it been executing*. Only the second answers it. The first is
not wrong, not stale, and not malformed — it is a correct answer to a question nobody asked, sitting
one field away from the one that was, on the same object, in the same response, under a name that
reads like the right one. `--json createdAt` is what a person types when they want to know how long
something has been going.

**The hedging went to the wrong layer.** Enormous care was spent on *what explains the number* and
none on *whether the number measures what the sentence says it measures*. That is the same asymmetry
the bootstrap-contamination entry records at much greater expense: five mechanism theories, each
internally coherent, all of them explanations of a difference that the experiment's own design had
manufactured. Care about causes is not a substitute for care about instruments, and it is
systematically the more available kind — a cause is interesting, an instrument is plumbing.

### Why this specimen is worth more than its size

Because of what the wrong reading *emitted*. It did not merely record a false number. It issued an
**actionable misdirection**: go read the in-progress log, find the phase that is not returning,
suspect an unbounded witness against a known rung drop. That instruction is well-formed, cites a real
gap in DESIGN.md, and would have cost its recipient an afternoon discovering that the hang is a
queue. The retraction had to name it explicitly and say *disregard entirely*, because a plausible
next step outlives the finding that motivated it — it gets forwarded, and the forwarded copy carries
none of the hedging.

And the hedged hypothesis had to be withdrawn too, for a reason worth stating on its own: it was
raised only because the overrun needed explaining. Remove the overrun and nothing points at that
commit — but a hedged hypothesis, once relayed, is remembered as *someone suspected #9024* long after
the measurement that motivated it is gone. **A hypothesis inherits the life of its evidence, and
nothing in the way we write them down enforces that.** Withdrawing it by name was the only mechanism
available.

### The cost asymmetry, which is the operational point

Finding the error cost one command — reading the job object instead of the run object. Ninety
seconds. The error itself had already been broadcast to a session that routes work to fifteen others.

That ratio is the entire argument for the structural correction over the prose one. There is no
version of *be more careful* that reliably fires here, because the careful thing was done: the report
named its own weakest link, refused an attribution, and asked for a discriminator. It named the wrong
weakest link. **A measurement identity is a structure you fill in; care is a thing you feel about the
half of the problem that happens to be interesting.**

### What generalises

Any object that carries both a *requested* and a *started* timestamp will support this error, and CI
systems all do. The rule that would have caught it is not about CI: **when a duration is the evidence,
name both endpoints before dividing.** An elapsed time computed from a single field is a subtraction
against an unstated assumption about what that field marks — which is the positional-citation defect
(§3) transposed onto time: a number that decays without anyone touching it, because what it measures
was never written down beside it.

### The coda: the figure was doing the misleading, and the document owns it

The run finished green while the retraction above was still being written — 47 minutes in the
three-phase step, every step green, the fourteen-hour gap closed at 17:39Z. The 44-minute reading was
the same step three minutes from finishing. So the retraction was right on the merits and not merely
procedurally, which is a pleasant outcome and not the interesting one.

The interesting one is what the sibling session produced next: the wall-clock of the last fourteen
main runs.

    146  84 143 148 147 120 108 145 231 219 216 187 165 158     (minutes)

Median ~146. Range 84–231. **All fourteen `conclusion=success`.** There is no overrun anywhere in that
window, and 142 — the number that started this whole exchange — is *four minutes under the median*.

So the reading was not merely wrong, it was wrong in the direction of an alarm, against a population
where the alarm can never be right. And the mechanism is not carelessness on the reader's part:

**DESIGN.md's ~30 minute figure describes the floor fold alone.** Not the queue. Not the toolchain
build. Not the parse sweep or the regen phase, both of which were consolidated into the same step on
2026-08-20 and 2026-08-23. The figure was accurate when written, remains accurate about its own
subject, and sits in a document where the natural comparand — the number a reader can actually obtain
— is the run's wall-clock. **Anyone dividing one by the other concludes a 3–5x overrun, every single
time, from a correct figure and a correct measurement.**

That is a sharper version of this document's thesis than any of the specimens above it. Earlier
entries describe a reader asking the wrong field. Here the *document* offers a figure whose subject is
narrower than the only question its readers can cheaply ask, and does not say so. The trap is
published, not stumbled into — and the same clause has been re-pointed at fresh run counters twice in
three days, which the Building-&-checks entry itself flags as evidence that transcribed run figures
are positional citations wearing a number's clothes.

The repair is not a bigger number. It is naming the subject beside it: *the fold takes ~30 minutes;
a green main run's wall-clock has run 84–231 minutes, median ~146, of which the majority is
frequently runner queue*. Two numbers, each with its denominator attached, so that neither can be
divided by the other. **A figure without its subject is not a measurement, it is a number that will
eventually be misused by someone reading carefully.**

## The discriminator that shared a premise with the thing it was discriminating

The coda above was written by an author who had, twenty minutes earlier, sent a colleague a
"cheap discriminator" to settle a question. The discriminator was ill-posed, the colleague noticed,
and the shape of the error is one this document had not yet recorded.

### The setup

A child session's work-item title read `MAIN RED: <witness> returns Bool(false), and its refusal arm
collapses to empty string`. Fourteen of fourteen main runs measured `success`. Two claims, apparently
in tension, so the reconciliation offered was: the floor holds known reds, therefore the witness is
either **enrolled-and-false** (quiet, known, unremarkable) or **unenrolled-and-false** (a false
witness reaching a terminal pass — a serious finding). Check the expected-red roster join; enrolled
means world one, absent means world two.

That is a clean partition, it is cheap to run, and it is exhaustive over the space it names.

### Why it could not work

Both branches carry `false`. The partition varies *enrollment* while holding the witness's value
fixed — and the witness's value was the stale part. The witness **passes**. It pinned a literal
`'sudo' 'systemctl' 'enable' '--now'` while the extdeps module had moved to `/usr/bin/sudo` plus a
non-interactive flag; a merged PR replaced the literal with a form derived from the three owning
authorities, and arm 6 has been true ever since. `MAIN RED` was accurate when written and expired when
that landed.

So the instrument's output space did not contain the answer. Worse — and this is the part that makes
it a specimen rather than a miss — **its "absent" branch was wired to the alarming conclusion.** A
passing witness that was simply never enrolled is the ordinary state of most witnesses. Running the
join would have returned *absent*, which the framing had pre-labelled *a false witness reaching a
terminal pass*, and the report would have manufactured an alarm out of a repaired witness. The
discriminator was not merely uninformative; it was **biased toward the more exciting world** by a
premise nobody had checked.

### The general shape

**A discriminator is only as good as the proposition its branches share.** Every partition holds
something fixed in order to vary something else, and that fixed thing is a premise the instrument
cannot see, cannot test, and will silently carry into whichever branch it returns. The two worlds
were carefully distinguished on *enrollment* — the axis that was already decided — while the axis
that actually mattered was never in the instrument at all.

The tell is available before running anything, and costs one sentence: **write out what both branches
assert in common, and ask how that was established.** Here the common conjunct was "the witness
returns false", whose entire provenance was *a title written at dispatch time* — a fact with a
timestamp, in a fleet where the underlying file had been rewritten by someone else's merge in the
interim.

**That rule as stated is not yet actionable, and the qualifier is the whole of it.** Every partition
has a shared conjunct; most are perfectly sound; a rule that fires on all of them fires on none. The
colleague who caught the original error supplied the missing half, and it is **provenance
asymmetry**: the shared conjunct here was not merely shared, it was *the only part of the instrument
that nothing in the instrument could check*, and its provenance — a third party's string, written
once, at dispatch — was **weaker than the varying half's**. Enrollment was live, joinable, and
re-measurable on every run. So the instrument was exquisitely precise about the half that could not
be wrong and silent on the half that was.

**The tell is therefore a shared conjunct whose provenance is weaker than the varying one's.** Where
the fixed half is the well-established part and the variation carries the uncertainty, a discriminator
is doing exactly its job, and this entry says nothing against it.

### The second-order point, which is the reason this is filed here

A *partial* check was already in the author's own message and went unused. The floor reports an
enrolled-but-passing witness as `stale_quarantine`, and the same DESIGN.md line the author had quoted
for `known_red_held=206` also carries `stale_quarantine=0`. The author cited the line, took one
number off it, and did not notice the number beside it.

**But that figure does not close the question, and an earlier revision of this section said it did —
which would have taught the wrong lesson from an incident whose root cause was trusting a static
string.** `stale_quarantine=0` eliminates exactly one branch, `{enrolled-and-passing}`. It says
nothing whatever about the unenrolled branch — the one the alarm was actually wired to. What closed
the question was *running the witness*. Filing this as "the answer was sitting on the line I already
quoted" would recommend a static document figure over an execution, in a document whose entire
argument is the other way round.

**The correct and narrower lesson: a ledger figure can eliminate a branch, and eliminating a branch is
not producing an answer.** That is worth having — it is cheap, and it would have halved the space —
but it is a different act from measuring, and conflating the two is how a document figure comes to
stand in for a run.

Which is this document's thesis one turn further in, with the correction applied: **the checking
artifact is sometimes in the sentence you already wrote — and it is usually partial.** Three of this
brief's entries were caught by their own recipients within minutes, including this section's own
overreach, corrected by the colleague whose refutation it records. That is a functioning fleet, and
it is also the measurement of how little the author's own care was contributing at the margin.

### What survived, sharpened by the colleague

The refusal arm still discards its bound cause: `RunnerCommandRefused { host_label: _, reason: _ }`
returning `""`. It hides nothing today, because the command builds. What it guarantees is that *if*
it ever refuses, the witness fails with `reason` in scope and thrown away one line above the failure.
The fix is worth landing precisely because the witness is green — it is not repairing a failure, it is
making the next one legible.

And that generalises the line this brief already carried about held reds. A **passing** witness whose
refusal arm has no located cause is in the same position as a held red with no located cause: nothing
prompts anyone to look until it breaks, and when it breaks the message is `false`. The empty-string
arm is the not-applicable-versus-malformed conflation collapsed one step further — not a wrong reason
symbol, but *no* reason symbol, chosen at the exact site where the reason was bound and available.

## The quantifier that no measurement covered

Every specimen above blames an instrument: a field answering a neighbouring question, a figure whose
subject is narrower than its readers' question, a partition holding the stale half fixed. This one has
no instrument defect at all, which is why it belongs last.

### What happened

A sibling session reported that the side-chat read endpoint was hanging, was scrupulous about it —
three completed zero-byte runs, two `HTTP 000` curls, an explicit control returning 200 in 1.64s on
the same host, seconds apart — and asked one question: *does yours work, or not?*

The reply was measured with equal care. Treatment: 0 bytes on stdout, **0 bytes on stderr**, hung past
a 50-second timeout. Control: 8424 bytes in 1.9 seconds, same host, same auth. Paired, controlled,
same-session, minutes apart. As a measurement of *this session* it was correct, and it is still
correct now.

It went out under the subject line **CONFIRMED FLEET-WIDE**, with the recommendation that the sibling
spend one of their scarce operator tokens reporting the outage.

It was not fleet-wide. The sibling's endpoint recovered on the next attempt — intermittent, not down.
The operator's own row: one session healthy, two with reads unavailable and sends fine. A known,
per-session, already-routed-around condition.

### The shape

**The measurement was sound and the sentence built on it was not.** Nothing in the treatment, the
control, or the pairing was wrong. What had no evidence behind it was the *quantifier* — a word
attached to a population of fifteen on the strength of a sample of two, one of which was already
known to be flaky at the moment it was cited.

And the care is what disguised it. Constructing a proper control makes a claim *feel* earned, and the
feeling attaches to the whole sentence rather than to the clause the control actually covers. **A
control licenses the comparison it controls, and nothing else.** Here it licensed *this session's read
path is broken relative to this session's other reads* — a genuinely useful fact, which was in the
message, under a headline the evidence did not reach.

The scope error also inverted the usual direction of this document's specimens: the earlier entries
describe an instrument answering a *narrower* question than the reader asked. Here the instrument
answered exactly the question posed to it, and the *reader* enlarged the answer on the way out.

### Why this one is worth its space

Because it cost someone else something. The other entries wasted the author's time. This one carried
an explicit instruction — *spend a token on this* — to a colleague with a hard budget, on a premise
that was already false when it was sent. **A confident sentence is an instruction to someone**, and it
travels without the measurement that motivated it, exactly as the withdrawn `#9024` hypothesis did
three sections above.

Two sessions broke the same rule in one exchange, in opposite directions: one described an
intermittent endpoint as a hang; the other took that report plus a single paired reading and promoted
it to a property of the fleet. The second is worse. Over-describing your own observation is an error
about a thing you looked at. Attaching a quantifier is an error about everything you *didn't*.

### The rule

**Name the population your evidence covers, in the same sentence as the claim.** Not as a hedge
afterwards — in the sentence, because that is the unit that gets quoted, forwarded, and acted on. "My
session's read path is dead, measured against a working control" would have been true, useful, and
would have prompted exactly the check that settled it. It is one word shorter than the version that
was sent.

## The positive specimen: what the discipline looks like when nobody is watching

Every entry above is a failure, and a document made only of failures teaches suspicion rather than
practice. It also flatters its author: catalogue enough traps and you can look rigorous while never
demonstrating rigour. So this last entry is somebody else's work, done right, and — the part that
matters — authored the day *before* the ruling that would have demanded it.

`docs/probes/underscore_named_call_order_treatment_2026-08-23.md` reports a paired before/after on an
emission change. What it does:

- **Preregisters the prediction.** The registered result is written down *before* the candidate board
  is observed, with the two expected emissions named literally.
- **Runs both arms in one dispatch.** Not two runs compared afterwards.
- **Names its confound and holds it constant.** An annotation-only correction from a different PR
  conflicted with the older tree, so *both* arms took the same resolved file, and the receipt says so:
  "common setup rather than a treatment difference." The confound is disclosed, not eliminated —
  which is honest, because it could not be eliminated.
- **Carries controls that could have moved and didn't.** Three preregistered control identities, plus
  an unrelated error code, unchanged across the pair. Without those, "seven blocks vanished" is
  consistent with the whole board shifting.
- **Refuses the comparison it is not entitled to.** The paired total is 324 where the retained board
  says 316, and rather than quietly using the nicer number or suppressing the discrepancy, it
  explains that the common file resolution changes the composite subject and states: *"no count is
  compared across those subjects. Only the within-pair delta is claimed here."*

That last bullet is the whole document in one sentence, written by someone who had not read it. **The
discipline is not producing better numbers; it is declining the comparisons your numbers do not
license.** Every failure catalogued above is the same act refused: a duration divided across two
clocks, a figure divided by a subject it never covered, a partition run over a premise it could not
test, a quantifier attached to a population of two.

### The one thing it does not settle, stated so the praise is not itself an overclaim

The after arm includes its own regenerated stage0 mirrors, so the two arms ran under **different
compiler binaries**. For a change *to emission* that is not a confound — the compiler difference *is*
the treatment — and the receipt is right not to control it away. But it means the result is a
within-pair delta under a bootstrap-produced binary, and the day-after ruling on measurement identity
(source ref + compiler binary identity + command and configuration + subject kind + terminal phase)
would ask that the binary be *named*, not that it be identical. Naming it costs a line and closes the
only door this receipt leaves open.

Which is the right note to end on. The receipt is not perfect, and pointing at it as a model while
suppressing its one gap would be the same flattery this section opened by refusing.

## Postscript the third: the counter

Recorded without ceremony because it happened *after* the rule was written, in the same document, by
the same author.

A dashboard notice reported a side-chat token balance of 14. It had read 14 all morning, so a pending
send was judged not to have landed, and a retry loop was killed on that basis. The balance later read
13. The send had landed all along; the notice was generated before the charge posted.

`balance` answers **what was the balance when this notice was built**. The question asked of it was
**did my message arrive**. One number, two questions, no error at the boundary — and by then the
identical shape had already been written up twice on the pages above.

**Knowing the class does not confer immunity.** The first postscript said so; this is the receipt for
the sentence.

## Two the corpus produced, not the authors

The entries above are mistakes people made. These two were made by machinery, and both were invisible
for the same reason: **they produced agreement.**

### The name our own emitter minted

A ruling went out this evening replacing fraction-of-a-board reports with sets of site identities:
report *which* sites, not *how many*, so that union and overlap are computable without a shared
denominator. The stated constraint was **module plus symbol, never `file:line`** — a line number is
not stable across trees or binaries, which is the positional-citation class the design document
already forbids.

A lane implemented it against 154 sites and came back with a hole in it. Their sites live in *emitted
Rust*, so the walk took the nearest enclosing item, and thirteen of them came back as:

    v2_lens_coverage :: CACHED     13 sites

`CACHED` is the `thread_local` static our own emitter generates inside every data-definition function.
Thirteen distinct authority symbols, reported as one name. Restricting the walk to top-level items
gives thirteen symbols with one site each.

**The constraint was one-sided.** It said where an identity may not come from and never said where it
must. The two-sided form: **module plus AUTHORITY-DERIVED symbol — never a positional coordinate, and
never an emitter-generated name.** `CACHED`, `__m`, `__sorted` and everything else codegen mints are
not identities; they are collisions waiting to be read as clusters.

What makes this the sharpest instance of the shorter-spelling class recorded here is that **no author
chose the name.** Every other instance in this document has a human picking a spelling that failed to
discriminate. Here a machine emitted the same name at thirteen sites *by design*, so the collision is
guaranteed rather than accidental — and the resulting report is finding-shaped: one symbol, thirteen
sites, a tidy cluster no reviewer would question. The walk was not careless. It asked a reasonable
question of a structure that cannot answer it.

The same lane also showed the identity was still lossy at symbol grain — 154 sites collapse to 99
symbols, one symbol holds ten sites, and one spans two mechanism roots. Two lanes hitting *different*
sites inside the same symbol would read as agreement, which would have destroyed the overlap detection
that motivated the whole change. **A join that manufactures false agreement is worse than the
fractions it replaced, because a fraction at least announces that it is a summary.**

### The duplicate git could not see

A hand-written 116-line line-based `.dag` parser was found in the seed — stripping declaration
prefixes off trimmed lines, walking characters for identifiers, tracking a coproduct flag. Its own
comment documents a cross-module type corruption it caused: type parameters scanned as references,
resolved whole-pool to an unrelated function, giving one module closure edges to an unrelated witness
and mistyping a third. **The defect and the argument against the mechanism are the same artifact.**

Then the routing turned up the part no one had seen: **the identical function had been authored twice,
independently, by two sessions.** One PR merged it. Another still carries a byte-identical copy —
same 116 lines, same start line, same md5 — awaiting a disposition on that very function.

`git merge-tree` over the two produces exactly one definition and **zero conflict markers**. Git
collapses identical additions at one location into a single change. So the collision was invisible to
git, to both reviewers, and to both authors — **precisely because the two copies agreed.** A
divergence would have conflicted loudly; convergence passed silently.

Two consequences, and the second inverts a decision:

- **When a fact is not published, the number of hand re-derivations is bounded only by the number of
  consumers who need it.** Two sessions independently needed *which names does this module declare* —
  a fact the parser already owns — and independently wrote the same scanner. That is the corpus making
  the single-authority argument on its own behalf.
- **Refusing the open PR now produces the worse outcome.** The receipt that would admit the scanner
  lives only on that branch; nothing on the main line mentions the function at all. So rejecting it
  deletes the accounting and leaves the code — a hand-written second parser sitting in a frozen seed
  with no growth receipt whatsoever. The refusal has been converted into its own opposite by a merge
  that landed first.

**Agreement is not corroboration when both parties derive from the same absence.** Two independent
implementations matching each other is normally the strongest evidence available. Here it was the
mechanism of concealment, and the thing they agreed about was a fact neither of them should have been
computing.

## Three from one hour of review, where the instrument was correct under its own definition

The specimens above are mostly measurements. These three are **review instruments** — the tools you
reach for to decide whether someone else's work is done. All three answered correctly. All three
answered a different question than the one being asked, and in two of them the wrong answer was the
one that took *less* work to reach.

### 1. The blocker that stopped counting because pushes aged it out

A PR reported `ready=True`. It had three `REQUEST_CHANGES` in its history and zero on its current head.
Readiness is computed over current-head reviews, so the flag was **right under its own rule**: there is
no current-head objection.

The question a reviewer is actually asking is *has the objection been addressed*. Those diverge the
moment an author pushes, because **a push does not answer a review — it makes it stale**. Nobody
withdrew the blocker and nobody re-ran the provider. It stopped counting.

Swept across all 40 open PRs, **eight** carried a provider whose *last* verdict was `request_changes` on
a stale sha; **five** had an approval on head with checks as the only unmet requirement. So the moment
an unrelated main-state breakage was fixed, five PRs would flip to ready simultaneously, each carrying
an unanswered blocker. One of them went `CLEAN` within minutes of that merge.

Checked by hand, the stale blocker on the worked example had two findings: one genuinely fixed by the
author's pushes, one still live. **Both outcomes were possible and only reading decided which.** That
is the point — the flag cannot distinguish them, and its green is produced by the same mechanism either
way.

> **A verdict that expires is not a verdict that was answered.** If readiness can be reached by the
> passage of pushes, then "ready" measures elapsed authoring, not resolved objections.

The structural repair is one clause: a stale `request_changes` must be superseded by a later verdict
**from the same provider**. That makes *answered* mean answered. Aging out is not an answer, and the
current rule cannot tell the difference between a fix and a wait.

### 2. The counter that included the people who agreed with you

Sweeping for that pattern, the obvious field is `stale_provider_count` — providers whose review is not
on the current head. It returned **ten** PRs.

The number is real and it is not the number. That field counts *any* provider off-head **including ones
who approved**. On one PR the stale codex verdict was an *approve*: a reviewer who looked, was
satisfied, and whose sign-off then aged out. Counting it as an unanswered blocker inflates the finding
and — worse — attributes an objection to someone who did not make one.

Filtering on *last verdict from that provider is `request_changes`* cut ten to eight, and to five that
would actually flip. **I had the inflated number written into a report before checking what the field
counted.**

> The available counter answered *is anyone off-head*. The question was *is anyone off-head **with a
> blocker***. One word of difference, a 2× error, and the wrong direction — toward alarm.

### 3. The grep whose word boundary inverted the conclusion

A blocker claimed a new dispatch had no production caller. The author's witness answered that the module
is "a leaf entry point, **like its siblings**" — an asserted parallel, which is the shape this repository
keeps finding to be false, so it wanted checking rather than trusting.

The check is one grep. Run unbounded, and excluding each function's own definition, it returns **four** hits for one sibling
provisioner and **three** for the other, against **zero** for the module under review — the subject is
the odd one out, the parallel is false, the witness is wrong.

Run as `(^|[^_a-zA-Z])provision_build_cache\(`, all three return **zero**. The parallel holds exactly
as claimed.

Every one of those seven "callers" was `realize_provision_build_cache(` or its sibling — a *different
symbol*, the realizer, which contains the provisioner's name as a substring. **The unbounded grep did not return a slightly wrong count. It returned the
opposite conclusion**, and the opposite conclusion is the one that looks like diligence: it *finds*
something, it contradicts the author, and it confirms the blocker.

The same artifact appeared twice more within the hour. A review notification promised "non-blocking
comments"; the artifact contained a bare verdict and no remarks. The seven `nit:` hits in its log were
substrings of `init:` and `unit:`; all four `findings` hits were quotations of DESIGN.md that the
reviewer had been *reading*. Nothing to fix, and searching for the word had found the word.

> **A substring is not a symbol.** Any search over identifiers whose boundary is unstated is
> answering about *text that contains a name*, not about *the name* — and where one identifier is a
> prefix of another, the two questions do not merely differ in precision, they can differ in sign.

### What the three share, and the asymmetry that makes it dangerous

Each instrument was correct under its own definition. None was broken, none needed fixing, and in all
three cases reading the definition would have shown the mismatch before the answer was used.

The asymmetry is what makes this worth a section rather than three footnotes: **in two of the three, the
wrong answer was cheaper to reach and more interesting to report.** `stale_provider_count` is a field
that already exists; filtering on last-verdict-per-provider is code you have to write. The unbounded
grep is what you type first; the word-boundary form is what you type after you have been burned. And
both of the cheap answers pointed *toward* a finding — more affected PRs, a false parallel, an author
caught out.

> When the quick instrument and the careful one disagree, note which direction the quick one errs. If
> it errs toward *having found something*, that is not a reason to trust it more. It is the reason it
> was cheap.

## The inverted one: a scan returns 109 matches and 108 of them are correct

**Measured by `crisp-crab-430` during the namespace cut, and recorded here with attribution rather than
left in a message, because findings in this fleet die with their sessions.** Two did today: one author
archived with a fully designed repair that existed only in inter-session mail. A specimen with someone
else's name on it is recoverable in every direction — they can claim it, expand it, or ask for its
removal. A lost one is recoverable in none.

Every specimen above fails in the direction people expect: the instrument **misses** something, or
answers a neighbouring question, and the correction adds findings. This one fails the other way, and
that makes it a different lesson rather than a fourth example of the same one.

Repairing a lambda-parameter defect, a naive scan for the offending shape returned **109 matches across
34 files**. The defect was real and one of those matches was it. **The other 108 were correct code.**

The discriminator is one character class: uppercase before `=>` is a qualified coproduct variant
*pattern* and parses correctly; only lowercase binds. The scan matched the *shape* — a qualified name
before an arrow — and the shape is shared by the defect and by every ordinary match arm in the tree.

> Rewriting all 109 would have been **a 108-site regression committed in the name of a one-site
> repair** — and it would have compiled, because the rewrite produces well-formed code that means
> something else.

### Why the high count is the dangerous result

The failure mode is not that the number was wrong. **109 is the correct answer to the question the scan
asked.** The failure is that volume reads as thoroughness: a scan returning 109 hits across 34 files
*feels* like it has found a systemic problem, and a scan returning 1 feels like it might have missed
something. Precision and recall are both load-bearing, and only one of them is visible in the output.

Every earlier specimen here would have been caught by someone asking *did I miss any?* This one is only
caught by asking the opposite: **of the things I found, how many are actually the thing?** That question
has no natural prompt. Nothing about a large result set suggests checking whether the set is mostly
false.

### The root, which covers three error classes at once

The author's own sentence, and it is better than the specimen:

> **A name that binds is not a name that refers.**

Let-binders, lambda parameters, field names and call labels had all taken qualification that only
*referring* positions may carry. Three separate classes, hit in sequence, one root — and the repair
went into the rewriter rather than into the tree, so the next integration cannot recreate them.

That is also the correct reading of the 108: they were not near-misses or edge cases. They were
*binding-versus-referring* on the other side of the same distinction the defect sat on, which is
precisely why a shape-level scan could not separate them.

### The rule

> **A scan censuses shapes; a defect is a fact.** Before acting on a large result set, take the
> highest-count file and check whether its hits are the thing you are looking for. If most are correct,
> the scan has found a *shape that the defect shares with healthy code* — and the size of the result is
> evidence against the repair, not for it.
