# Instrument traps measured on 2026-08-24

Six ways an instrument in this repository answered a **narrower question than it was
asked** in one night, across two sessions and nine lanes. Each one is recorded with what
was measured, not with advice — the advice is always the same and always insufficient
("check your instrument"), and what actually transfers is the specific shape.

Every one of these produced output that was TRUE. None of them was a bug in the tool.
In each case the reader supplied a binding the output did not state.

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
