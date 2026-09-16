# DSV41 Cut D — redesign against the observed Group A, not the assumed one

Cut D of DSV41-ENGRAM-RUNTIME-0 was specified as a bounded replacement transaction against a
live V4 Flash incumbent: drain its offer, stop it, launch V4.1 TP4, probe, then **promote or
restore V4 Flash**, with promotion gated on *rollback demonstrated* — an incumbent request
served again after the experiment.

**No incumbent was observed**, twice, 14 hours apart. Every terminal in the original cut that
names V4 Flash therefore has no subject *on those readings*, so the cut cannot be executed as
written, and executing a lightly-edited version would silently drop safety properties the
original bought.

**That absence is a dated observation, not a property of Group A** — D0 re-establishes it under
the transaction's own claim, because something can become live between a reading and a stop.
This document records what was observed, what follows, and the cut that replaces the original.

**HOW TO READ THIS DOCUMENT.** It went through nine review rounds and several of its own
premises were rejected along the way. Rejected premises are marked **⚠ REJECTED PREMISE** and
kept only so they are not re-derived; everything else is operative. Nothing here should require
inferring that a later section supersedes an earlier one.

## What was observed

Read directly from the four Group A ranks (`spark-a3ee`, `spark-3bd5`, `spark-c2b1`,
`spark-ac79`) over SSH, twice, ~14 hours apart:

| fact | reading |
|---|---|
| serving process | none — no `vllm`/serving systemd unit running |
| OpenAI endpoint on the serving port | no answer |
| occupant | one `gunbc-spark-pair` container, image `gunbc-spark-pair-runtime:ray-2.58.0`, created `2026-09-14T00:15:35Z`, up >2 days |
| memory held | 107–109 GiB of 121 GiB per rank; `nvidia-smi` attributes ~100.7 GiB to `ray::RayWorkerProc.run` |
| top process RSS | 2.6 GiB |
| free disk | 2.6–3.1 TB of 3.7 TB per rank |

Two details make that table read correctly rather than look contradictory. `nvidia-smi
--query-gpu=memory.total` returns `[N/A]` on a GB10 because there is no separate accelerator
pool — the figure it attributes to the Ray worker is a claim on the same unified RAM `free`
reports. And the top process RSS being 2.6 GiB while 108 GiB is used is not an inconsistency:
CUDA device-side allocations on a unified pool do not appear in RSS. A process listing
therefore makes these nodes look idle while they are 89% full.

The operator's statement that Group A "is not being used by anyone" is correct about
**traffic** and not about **occupancy**.

### These readings do not contradict the fold's input, and the document must say so

`gunbc.spark.pair_serving_observed` `group_rank_free_memory_observed` — the reading the fit
fold consumes — is qualified **"with the engine idle"**. The table above is a reading of an
**occupied** rank. They are answers to different questions and neither supersedes the other:

| question | condition | who answers it |
|---|---|---|
| would V4.1 at TP4 fit if these hosts were cleared? | engine **idle** | `group_rank_free_memory_observed`, consumed by `candidate_component_budget` |
| are these hosts clear right now? | **as found** | the table above, and D0 |

So the occupancy reading is not evidence that the fold's input is stale, and the fit verdict
stands on its own reading. **Presenting both without reconciling them would have been the
defect** — a reader comparing 116.9 GiB free against 107–109 GiB used would conclude one of
them is wrong, when what actually differs is whether anything was running at the time.

### Why these readings are prose here, stated as a divergence rather than left silent

§6 says name the instrument, never transcribe its output — and `pair_serving_observed` is the
modeled home, carrying `PairServingObservationProvenance` with `observed_on` / `observed_by` /
`sources`, and free memory as a `std.measure` interval with an `instrument` and a
`precision_caveat` rather than a hand-typed figure. A `FabricGroupA` occupancy row there is the
conforming move, and these figures are not in it.

**This is a §3b divergence and the reason is that D0 is the step that produces that row.** The
cut below exists precisely to take the claim, reconcile the occupancy against the declared
realization, and emit an entry-state receipt. Hand-authoring the row now would author by hand
the artifact the cut produces by execution — and it would do so from a one-off SSH procedure
rather than through the instrument the model would name, which is the weaker evidence of the
two.

So these figures are carried here as **what motivated the redesign**, not as the corpus's
record of Group A's occupancy. The corpus's record is `EntryStateReceipt`, D0 produces it, and
if this program stalls before D0 runs then the corpus correctly continues to have no modeled
observation of Group A being occupied — which is honest, because nothing executing has made
one.

## What follows, stated as consequences rather than edits

### 1. There is no drain, and calling a reclaim a drain would claim a property we do not have

The original sequence was *withdraw the incumbent offer → stop the incumbent*. A drain exists
to preserve continuity for live traffic. There is no live traffic and no offer being served,
so what the transaction actually performs is a **reclaim** of an occupying container.

The distinction is not cosmetic. A drain's safety property is "no in-flight request was lost";
a reclaim has no such property and needs none. Keeping the word would assert a guarantee the
step does not provide — §4b rung honesty applied to a transaction step.

**But dropping the drain obligation does not drop the QUIESCENCE obligation, and an earlier
draft read as though it did.** No answering front door means no demonstrated serving traffic.
It does not mean the occupant is safe to stop: those Ray workers hold ~100 GiB each, and what
they are holding it for is unread. Before the reclaim, D0 must establish or revoke — never
assume — Ray jobs, actors and placement groups; serving offers and new-admission eligibility;
acquired or stranded seats; host claims, leases and ownership; and any actuator that could
recreate the occupant after it is stopped. So the operation is:

```
fence new acquisition
→ establish or revoke active work, claims and seats
→ stop the occupying realization
→ verify resource release
```

A reclaim with no in-flight-request obligation still has a fencing one. And note that
enrollment says only that Group A is a route that *may be asked* — it does not say the route
is live, and availability is separately three-valued (Available / Unavailable / **Unread**).
Unread is where Group A sits, and unread is not empty.

### 2. The occupant is DECLARED, not residue — and that changes the rollback question entirely

An earlier draft of this document rejected "restore the occupant" on the grounds that *we hold
no declaration that produces that Ray container, so restoring it would mean reconstructing from
an observation*. **That sentence is false**, and the correction matters more than the error.

The corpus declares exactly what was observed:

- `extdeps.docker.images.sparkrun_vllm_ds4_gb10` `gunbc_spark_pair_runtime_image_tag` is
  `gunbc-spark-pair-runtime:ray-2.58.0` — the observed image, with its pinned base and its
  exact `ray[default]==2.58.0` requirement;
- `gunbc.spark.pair_serving_realization` `spark_pair_container_name` is `gunbc-spark-pair` —
  the observed container, with head and worker units and a one-container-per-host realization;
- `gunbc.spark.pair_serving_desired` `spark_pair_serving_groups` still contains
  **`FabricGroupA`**, and `spark_pair_realization` maps that desired set into realizations.

So the ranks are not carrying residue. They are carrying an instance with **declared lineage** —
image tag, container name, desired roster, realization and apply path all join.

**But "converged" is stronger than the evidence, and an earlier draft claimed it.** This
document's own observation table says no vLLM process, no serving unit running, no answer on
the enrolled endpoint — while the declared realization includes a head unit that starts Ray and
then vLLM, worker units under supervision, a head engine with `Restart=always`, and an apply
that treats an inactive unit as grounds for restart and finally requires `systemctl is-active`.
An instance matching that declaration would be answering. What the evidence supports is:

```
declared producer and authority        ESTABLISHED
full realization presently converged   NOT ESTABLISHED
observed instance                      apparently DRIFTED
```

So on the facts written here, the expected D0 answer is **`DeclaredOccupantDrifted`**, not
`DeclaredOccupantObserved`, unless D0 reads the rendered units, container specs, rank membership
and serving state and proves otherwise. That does not weaken this section's conclusion — the old
authority can still recreate its desired realization, which is the whole hazard — it only stops
container lineage being used as proof that the realization currently converges.

**The consequence is a design break, not a wording fix.** If Group A remains in the desired set,
an existing apply path can recreate exactly what D1 would call residue. "Return to a released
empty state" is then *not a converged state*: a converger reasserting the desired realization
would undo D1's terminal, and nothing in the transaction would notice. A terminal that another
authority can silently reverse is not a terminal.

So the question is not *may we restore the occupant* — it is **under whose authority do the
hosts sit while V4.1 is on them**, and the plan must answer it explicitly:

- **suspend Group A's pair-serving desired authority**, and record that suspension as part of
  the authorization, so the hosts are never in a position where two authorities both believe
  they own them. *Suspend*, not *withdraw*: different operations, and only withdrawal is
  expressible today.

**THE SUSPENSION'S LIFETIME IS ONE RULE, because an earlier draft gave three and they were not
interchangeable** — "for the experiment's duration", "record that withdrawal", and an
undifferentiated "claims released" each permitted a different implementation:

```
atomic transfer into suspension
→ the suspension AND the exact four-host reservation persist through
  QuiescentReservedBaseline — they do NOT end when D1 ends
→ they end only when one of:
     D2 promotion assumes the authority
     the prior authority is explicitly restored
     FleetReleasedBaseline is explicitly selected

D1 cleanup RELEASES      rank and process claims, offers, seats, live allocations
D1 cleanup RETAINS       the four-host reservation, under QuiescentReservedBaseline
```

"Claims released" is not one category: the execution claims must end and the host reservation
must not, and a cut that says only "claims released" leaves an implementer to guess which.

That is a real decision with its own consequences — it means Group A is deliberately not
pair-serving while this runs — and it belongs in the authorization, not in a footnote. It also
**cannot be performed today**: the operation it names does not exist, which is the gap this
section ends on.

**THE CORPUS HAS FACED THIS HAZARD BEFORE, THOUGH IT ANSWERED A DIFFERENT QUESTION.**
`gunbc.serving.serving_enrollment` records that one list used to answer two questions that move
for different reasons — *whose hosts this repository claims and converges*, and *which endpoints
a turn may be offered* — and that they stopped coinciding on 2026-09-08. The reason given is the
hazard this section is about: Group B's head is a host another authority also names, so leaving
B in the convergence roster left **two authorities claiming one machine, and the first
pair-serving convergence to run would have acted on a host it believed idle.** The operator
authorised removing the claim, and the module was split so the two questions could move apart.

That split is real and is observable in the current rosters — it is why a group can be enrolled
for serving without being claimed for convergence:

```
serving_enrolled_groups   = [FabricGroupA, FabricGroupB]   ← who may be ASKED
spark_pair_serving_groups = [FabricGroupA]                 ← whose hosts we CLAIM and converge
```

**⚠ REJECTED PREMISE, kept only so it is not re-derived.** An earlier draft concluded from this
that Group B is "precisely the state this cut needs for Group A", that withdrawal therefore
needs "no new mechanism", and that the precedent could simply be applied a second time. **That
is wrong and the next subsection says why.** Group B's withdrawal was a removal from a roster
that still had another member; Group A's would empty it, and Group A is the row that speaks for
srv7 and srv8 whereas Group B's hosts were claimed by another authority that wanted them. The
precedent establishes that the enrolled/claimed split exists and that a claim can be withdrawn —
it does **not** establish that this withdrawal is safe.

**Does the withdrawal also remove the host admissibility or launch authority D1 and D2 need?**
**Yes — and an earlier version of this section answered that wrongly.** It traced admissibility
to `gunbc.spark.host_commitment` `spark_serving_admissible_hosts`, found that removing a claim
*loosens* commitment, and concluded the withdrawal "revokes a competing claim, which is the
objective." That conclusion is false, because it asked who is let in and never asked who is let
out.

`host_commitment` derives every `MemberOfClaimedServingGroup` cause directly from
`spark_pair_serving_groups`, and uses those causes to decide **which serving subject owns each
host**. The cell-role roster assigns `SparkServingCell` to **srv5 and srv6 only** — the Group A
claim is what speaks for srv7 and srv8. So a literal withdrawal produces:

| hosts | after withdrawal |
|---|---|
| srv5, srv6 | cell role remains, Group A ownership cause gone → serving role held by **no serving subject** |
| srv7, srv8 | no cell-role row, no ownership cause → **uncommitted**, admissible to any serving subject, and eligible as "unplaced" effect hosts |

And V4.1's own candidate field derives its host population through `spark_serving_admissible_hosts`
for the **Group A pair-unit subject** — `PairServingUnitOn { FabricGroupA }`, deliberately
unchanged across the V4→V4.1 transition. **So the TP4 subject this program exists to run loses
two of its four hosts**, while srv7 and srv8 become available to unrelated lanes: the vLLM
source-build and runtime-image-probe roots both authorize effects through `admit_unplaced_host`,
and the latter pulls an image and starts a container.

The remedy would have destroyed the authority relation it exists to protect, and the plan's
prose-level "exclusive claim" would have disagreed with the production authority that build and
probe lanes actually consult. Loosening is not neutral when your own exclusivity is what is
being loosened.

**WITHDRAW AND SUSPEND ARE NOT INTERCHANGEABLE, and this document may not use them as though
they were.** Suspension is the right operation here. **No such modeled operation exists yet** —
that is the gap this cut must close before it can run, and it is a modeling obligation rather
than a sequencing detail. It needs a typed authority state whose projections differ per
consumer:

```
PairServingGroupAuthority
  = PairServingActive { group }
  | SuspendedForAuthorizedSuccessor { group, authorization, successor_subject, lease, cleanup }
  | ReleasedToFleet { group, release_receipt }
```

with each consumer told explicitly what it sees:

| consumer | Active | Suspended |
|---|---|---|
| pair realization / apply | acts | **does not act** |
| host commitment | all four hosts owned | all four hosts **still owned**, by the named successor |
| build / probe host admission | placed | **still placed**, never unclaimed |
| serving enrollment | unchanged | unchanged |
| capacity standing | ordinary result | a **typed suspended** result, never an empty population |

**One positive result from the trace:** serving enrollment is genuinely independent of pair
desired state. Group A stays enrolled as a route and the harness still probes it before
offering work, so suspending the old realization does not delete the front door.

### The withdrawal EMPTIES the roster, and that is not a local toggle

`spark_pair_serving_groups` is `[FabricGroupA]`. **Group A is its only member**, so withdrawing
it does not remove one entry — it leaves the list empty. Every fold over that roster then runs
over an empty domain, and the repository already carries this class by name:
`gunbc.recurring_failure_mode.predicate_vacuously_true_on_an_empty_domain`.

An `all(...)` over an empty list is **true**. So a check of the form *every claimed serving
group satisfies X* stops being evidence the moment the roster empties — it passes, loudly and
greenly, because there is nothing left to fail it. That is the opposite of what a safety check
should do when its subject disappears, and it would arrive exactly when the fleet is least
ordinary.

**The obligation lives on the roster, and this document deliberately does not restate it.** §6
says the mark on the carrier is the authority; an earlier draft then restated the obligation
here in its own words, which drifted from the carrier's as the carrier was corrected — the two
disagreed on whether it covers *folds* or *every direct and transitive consumer*, and on whether
the row is *ownership* or a *commitment input*. Restating an authority is how you end up
maintaining two of them.

So the obligation is the annotation above `spark_pair_serving_groups` in
`gunbc.spark.pair_serving_desired`, and it is authoritative over anything this document says
about it. Read it there.

The roster also reaches **build and probe admission** and **capacity**, not only convergence —
so "suspend the claim" is a fleet-wide change wearing the costume of a one-line edit. The
authorization must name what it empties, not merely what it withdraws.

**THREE SUBJECTS, THREE NAMES.** An earlier draft used one word — "baseline" — for the state
D0 records and the state D1 returns to. Those are mutually exclusive, and the ambiguity sat in
the one sentence that tells a rollout worker where to leave the hosts.

| subject | what it is | who produces it | is it a return target |
|---|---|---|---|
| **`EntryStateReceipt`** | the ranks as found — occupant present, ~108 GiB held | D0 | **no** |
| **`ReleasedBaselineSpec`** | the *desired* post-experiment state: no ranks, no engine, no serving offer, no seat, no occupying container, no live device allocation. **Whether a host reservation remains is the choice made below** — for the ordinary D1→D2 path it does | declared before D1 runs | it is the target |
| **`ReleasedBaselineReceipt`** | a readback establishing that the spec holds | D1's terminal | — |

A spec and its readback are two facts, and collapsing them is how "we returned to baseline"
becomes an assertion instead of a reading.

**RESERVED OR FREE — the spec must choose, and an earlier draft claimed the guarantees of
both.** It defined the released state as having *no host claim* while also suspending the old
authority only "for the experiment's duration". That leaves three readings, and two of them are
broken: if suspension ends when D1 ends, the old realization can be reapplied immediately and
the released terminal is invalidated; if suspension persists with no reservation, srv7 and srv8
become effect targets and srv5 and srv6 are held by nobody. Only the third is coherent:

```
QuiescentReservedBaseline          ← the target when D2 is expected to follow
    no rank processes, no engine, no offer, no seat, no live device allocation
    immutable verified artifacts may remain
    the exact four-host reservation REMAINS
```

That holds no runtime live across the authorization boundary — only the right to use the four
hosts, so an unrelated build or probe cannot occupy them before D2. If the intent is instead to
give the hosts back to the fleet, that is a **different** terminal and must be named as one:
`FleetReleasedBaseline`, where D2 must reacquire the hosts and may not assume the same four
remain available.

**What may remain.** The released state releases *processes, seats and live memory* — not bytes
at rest, and under `QuiescentReservedBaseline` not the reservation either. The exact runtime image and the content-addressed row stores may stay on
local storage. Rematerialising hundreds of gigabytes to satisfy a definition would be redundant
work, and immutable content-addressed artifacts carry no live runtime across the authorization
boundary. The spec says so explicitly rather than leaving a later reader to decide whether a
staged row store violates "released".

The **released state** remains the return target rather than the entry state, but now for a
stated reason: the entry state is an instance of a desired authority we are suspending, so returning to it means *restoring that authority*, which is a separate decision
from ending the experiment. D1 ends the experiment; restoring pair-serving authority is its own
step with its own evidence.

### 3. The risk profile inverts, and that makes P1 Cut 3 more important rather than less

The original cut's danger was **breaking a live service**. That danger is gone. The remaining
danger is **not being able to give the hosts back** — a partial or stalled launch leaving
ranks up, seats held, and 121 GiB per node unavailable, with no incumbent whose restoration
would incidentally clear it.

The original brief said P1 Cut 3 "should block" ordinary production promotion. Against an
incumbent, that hedge was defensible because restoring the incumbent was itself a cleanup path.
Without one it is not: **promotion must refuse.** But the condition is CONJUNCTIVE, and an
earlier draft joined two different problems with `or`:

```
(P1 Cut 3, or an exact seat-lifecycle equivalent)
AND
(a demonstrated rank / process / offer / claim cleanup path to ReleasedBaselineSpec)
```

P1 Cut 2 detects an exporter still live over an engine that has stopped advancing and
explicitly leaves held-seat invalidation to Cut 3. That is Cut 3's subject: **seat and
incarnation invalidation.** It is not the same fact as killing partial ranks, withdrawing
offers, releasing host claims, stopping containers and returning unified memory — those are
host and runtime cleanup. A single realization may discharge both, but only if its evidence
actually proves both, and writing `or` invites treating either one as sufficient.

### 4. The serving route is declared and unfulfilled, and the experiment must not paper over it

`gunbc.serving.serving_enrollment` enrolls `FabricGroupA` — `serving_enrolled_groups` carries
it, with an endpoint, a port and a turn ceiling. The model says Group A is a serving route.
Nothing answers there.

So *one bounded request through the normal route* is, today, unachievable for **any** model
rather than specifically for V4.1. A cut that lists it as a terminal without saying so would be
specifying an outcome nobody can currently produce, and the first lane to attempt it would
discover the gap under a live transaction. Establishing that the enrolled route actually
carries a request is its own precondition, and it is stated as one below.

## The replacement cut

### D0 — take the claim, reconcile the occupancy, then record the ENTRY state

**D0 performs an ATOMIC AUTHORITY TRANSFER, not an acquire-then-withdraw sequence.** An earlier
draft said D0 takes the exclusive claim first *and* that the hosts must never be owned by two
authorities. **Those cannot both hold without a transfer operation**, and writing both was a
contradiction rather than a plan:

```
acquire experiment claim, then withdraw old   → OVERLAP: two authorities own the hosts
withdraw old, then acquire experiment claim   → GAP: build/probe lanes may take them
atomic transfer                               → one active owner throughout
```

The gap arm is not theoretical: srv7 and srv8 become `admit_unplaced_host` targets the moment
their ownership cause disappears. So the transfer is the operation, and it is part of the
suspension gap named in §2 — there is no modeled transfer today.

**D0 still brackets its readings with the claim it holds.** "No incumbent was
observed" is a dated observation, not a property of Group A — the two SSH readings above are
14 hours apart and say nothing about the interval between the last one and the stop. Without a
claim held across that interval, something can become live between observing and reclaiming,
and the transaction would proceed on a stale premise. That is the same defect this whole
document exists to correct, one level down.

**D0 asks a reconciliation question, not an inventory question.** Not *what is here*, but:

```
does the observed container / image / unit population reconcile
with the declared pair-serving realization?
```

with four answers, each leading somewhere different:

| answer | meaning | disposition |
|---|---|---|
| `DeclaredOccupantObserved` | declared **and** shown converged | proceed via the atomic transfer into suspension of §2 — not a withdrawal |
| `DeclaredOccupantDrifted` | declared lineage, convergence not established — **the expected answer on the readings above** | proceed via the same transfer, record the drift; the drifted state is not a target |
| `ForeignOccupantObserved` | something we do not declare | **refuse** — not ours to reclaim |
| `OccupancyUnread` | the question could not be answered | **refuse** — unread is not empty |

**EVERY ARM NEEDS AN AUTHORITY-STATE TERMINAL, including the refusing ones.** The transfer
happens before the readings — that is what closes the overlap and gap arms — so a refusal fires
*after* the authority has already moved, and an earlier draft said nothing about what becomes of
it. Neither silent answer is acceptable: restoring `PairServingActive` may reactivate
convergence over a foreign or unread occupant, and leaving `SuspendedForAuthorizedSuccessor`
standing means a **refused** D0 has permanently changed a long-lived authority. So the transfer
lands in an intermediate state that inspection happens inside:

```
PairServingActive
→ SuspensionPendingReconciliation { transaction, lease,
                                    old actuator disabled,
                                    successor actuator disabled }
→ read and reconcile, then:
     SuspendedForAuthorizedSuccessor   declared eligible branch
   | PairServingActive                 only when restoration is shown safe
   | FencedRefusal                     foreign or unread — needs operator disposition
```

A versioned compare-and-swap protocol with the same dispositions would serve equally. The
requirement is that **no D0 answer leaves the authority state undefined**, and that both
actuators are disabled while the answer is being determined.

And it branches on the incumbent rather than assuming its absence:

```
live serving incumbent observed      → the original drain/preserve path, or refuse this one
no incumbent + quiescent declared occupant → the reclaim path below
foreign, active, or unread occupancy → refuse
```

Only then does it record the entry state per rank: occupant identity and creation time, memory
used and free, absence of a serving process, absence of an answer on the enrolled endpoint, and
free disk. That receipt is evidence of **what changed** and forensic record. It is not the
rollback target.

Terminal: a claim is held, occupancy is reconciled to one of the four answers, an entry-state
receipt exists for all four ranks, and no host state was altered.

### D1 — reclaim, bounded experiment, mandatory return to the RELEASED state

Authorization is exact and time-bounded over: candidate key, the four host identities, the
**produced** runtime image digest, model source identity, the four row-store identities, the
TP4 profile, the selected Engram route, the D0 entry-state receipt, and the cleanup path of §3.
Candidate identity is not authorization.

```
claim the four ranks
→ reclaim the occupant, read back that its memory is released
→ re-verify the staged artifacts against their declared identities
→ launch V4.1 TP4
→ establish all-rank agreement
→ complete model load
→ complete graph/runtime initialisation
→ answer semantic and differential probes
→ ONE NORMAL ROUTED REQUEST, while the candidate is still live
→ RETURN TO THE BASELINE THE AUTHORIZATION NAMED, unconditionally
   (QuiescentReservedBaseline on the ordinary D1→D2 path)
→ read back that baseline's receipt
```

D1 **always** ends at the released state — not at the entry state, which it deliberately does
not restore. It does not promote.

**AND THE IDEMPOTENCE CLAIM AN EARLIER DRAFT MADE HERE WAS FALSE.** It said a rerun of D1
mutates nothing. A rerun of D1 cannot mutate nothing: it reclaims hosts, launches V4.1,
initialises it, routes a request and tears it down. **The experiment is REPEATABLE; the cleanup
converger is IDEMPOTENT**, and those are different properties that one sentence was conflating.
What is actually checkable:

```
run the D1 experiment
→ converge the baseline (QuiescentReservedBaseline or FleetReleasedBaseline)
→ read the baseline receipt
→ run THAT CONVERGER again
→ require no mutation
```

The no-mutation property belongs to the converger and is asserted against it. Attaching it to
the experiment made the receipt unfalsifiable, because no honest rerun could have satisfied it.

Weight loading is not success. Acceptance begins after runtime initialisation and one valid
semantic completion. An HTTP 200 discharges nothing.

**The normal routed request happens INSIDE this window, and an earlier draft had it outside.**
The end-to-end join — router request, selected offer, authorization, four agreeing ranks,
produced completion — binds the exact realization D1 created. Deferring it past the teardown
would mean relaunching V4.1 to perform it, which is a second fleet-mutating experiment, or
performing it against a different model, which establishes nothing about this candidate's
offer and rank wiring. The original Cut D placed it correctly and this redesign keeps that
ordering.

**The cleanup closure D2 requires is needed HERE too**, not only at promotion: once D1
acquires a seat to serve that request, any failure after acquisition can strand exactly the
seat P1 Cut 3 exists to invalidate.

Every stage capable of preventing a later observation carries a terminal disposition: ranks
stopped, claims released, hosts at the released state, that state read back — or a typed refusal
naming the missing cleanup evidence. A failure arm that cannot reach it is the one outcome that must
stop the line loudly, because it is the risk this cut actually carries.

### D1a — the route reaches a process, which is the only model-independent half

An earlier draft claimed the routed request was separable from V4.1 and then wrote a terminal
requiring *four agreeing ranks and a produced completion* — which no model-independent step can
produce. **That terminal was not separable and the claim of separability was wrong.** The two
halves come apart like this:

**Model-independent, but NOT free, and an earlier draft called it cheap.** Whether the enrolled
route reaches a listening process is a fact about routing rather than about V4.1, and it can be
established against any responder. But **binding a responder to the enrolled endpoint means
starting a process on Group A's declared address and port** — that is fleet mutation, on the
hosts this entire cut exists to fence. It cannot happen before the authority transfer, and it
owes a teardown like any other launch:

```
after D0 reaches SuspendedForAuthorizedSuccessor
→ launch a bounded endpoint responder
→ issue the enrolled-route reachability request
→ stop the responder
→ re-establish QuiescentReservedBaseline
```

Its output is narrow and should be named so nobody reads it as more:
**`EnrolledEndpointReachabilityReceipt`** — it establishes router-to-endpoint transport and
listener reachability, and **nothing else**. Not offer selection, not seat acquisition, not rank
agreement, not model advertisement, not that any candidate serves. D1 and D2 still owe every one
of those joins.

**Model-dependent, and therefore NOT separable.** The end-to-end join — router request, to
selected offer, to authorization, to four agreeing ranks, to a produced completion — cannot
exist without a TP4 model actually loaded. It belongs inside D1 (as the experiment's semantic
terminal) and inside D2 (as the promotion terminal), and it is stated in both. Extracting it
into a precondition would be specifying a step nobody can perform alone.

### D2 — production promotion, separately authorized

A new authorization over the already-proven realization. Promotion **refuses** unless:

- the candidate is realized, not merely keyed — produced image digest, applied patch
  population, four row-store identities;
- row-store faithfulness is established over the **published** population, not a fixture;
- D1 completed and its `ReleasedBaselineReceipt` was read back;
- D1a established that the enrolled route reaches a process, and D1 produced the end-to-end join while the candidate was live;
- **P1 Cut 3 (or an exact seat-lifecycle equivalent) AND a demonstrated cleanup path to `ReleasedBaselineSpec`** — two facts, not one;
- **and the no-mutation rerun is a TERMINAL, not a precondition.** An earlier draft listed
  "a second convergence run mutates nothing" among the conditions promotion refuses without —
  but a no-mutation *promotion* rerun cannot precede the first promotion. The order is:

```
D2 preconditions satisfied
→ apply the production promotion
→ one exact normal routed request succeeds
→ apply the production desired state AGAIN
→ require no mutation
```

Capacity calibration is a follow-up. Record steady resident bytes, cache retention, local read
bytes, lookup hit/miss population, prefill and decode rates, TTFT and ITL — and do not turn any
of them into policy before correctness converges.

## What this redesign does not change

The identity discipline is untouched: checkpoint identity is not runtime identity, route B
versus D is not a different binary, TP4 is an execution-profile fact, patch order is semantic,
and build evidence does not transfer across a patch, tokenizer, checkpoint, row-store, topology
or route change. The published runtime's scoped `NoFeasibleRealization` stays true about the
published runtime; a file-backed runtime is a new candidate subject and does not weaken it.

## Why the program is still worth running

**Named, not transcribed.** An earlier draft re-derived the fit in prose and inverted it. The
correction is not a better number — it is that this document should not carry the number at
all.

The authority is `gunbc.spark.serving_deployment_selection` `candidate_component_budget`. The
reading it consumes is `gunbc.spark.pair_serving_observed` `group_rank_free_memory_observed`.
Its per-rank verdict at 8 and at 4 ranks, the component breakdown behind the budget, and the
corroborating figure from the rollout lane's independent instrument are all carried in the
annotation beside `test.claim.spark.serving_deployment_selection_witness_test`'s selecting
witness. Read them there; they move when the fold's inputs move, and a copy here would not.

What this document needs to state is the **shape** of that result, because the cut depends on
it: **route A at TP4 is excluded by a measured shortfall that exceeds the reading's own
precision** — an established exclusion, not a near miss.

Three things invert that answer if taken from outside the fold, and all three are mistakes this
document made before it was corrected:

- **the denominator is the observed idle-free reading, not the device total.**
  `candidate_component_budget` says so in as many words, and refuses rather than substituting
  the total when a reading is missing;
- **the budget is what remains after the artifact's other three components, not one.** The
  vision tower with its aligner and the DSpark draft model are the difference between a
  two-component sum showing headroom and the fold showing a shortfall;
- **the shortfall must clear the reading's precision.** The runtime prints one decimal of GiB,
  so the free figure is an interval, and a verdict drawn against it is established only if it
  exceeds that width. The fold's does.

**That is why Cut B exists.** Not because the published runtime nearly fits — it does not fit,
by a margin the measurement can carry — but because moving Engram off the resident budget is
the only change that alters the term the fold is short on.
