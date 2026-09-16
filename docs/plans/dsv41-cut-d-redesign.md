# DSV41 Cut D — redesign against the observed Group A, not the assumed one

Cut D of DSV41-ENGRAM-RUNTIME-0 was specified as a bounded replacement transaction against a
live V4 Flash incumbent: drain its offer, stop it, launch V4.1 TP4, probe, then **promote or
restore V4 Flash**, with promotion gated on *rollback demonstrated* — an incumbent request
served again after the experiment.

**That incumbent does not exist.** Every terminal in the original cut that names V4 Flash has
no subject, so the cut cannot be executed as written, and executing a lightly-edited version
would silently drop safety properties the original bought. This document records what was
observed, what follows from it, and the cut that replaces it.

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

So the ranks are not carrying residue. They are carrying **the declared pair-serving
realization, converged onto the hosts this cut wants to take.**

**The consequence is a design break, not a wording fix.** If Group A remains in the desired set,
an existing apply path can recreate exactly what D1 would call residue. "Return to a released
empty state" is then *not a converged state*: a converger reasserting the desired realization
would undo D1's terminal, and nothing in the transaction would notice. A terminal that another
authority can silently reverse is not a terminal.

So the question is not *may we restore the occupant* — it is **under whose authority do the
hosts sit while V4.1 is on them**, and the plan must answer it explicitly:

- **withdraw or suspend Group A's pair-serving desired authority** for the experiment's
  duration, so the released state is a converged state rather than a race; and
- record that withdrawal as part of the authorization, so the hosts are never in a position
  where two authorities both believe they own them.

That is a real decision with its own consequences — it means Group A is deliberately not
pair-serving while this runs — and it belongs in the authorization, not in a footnote.

**THREE SUBJECTS, THREE NAMES.** An earlier draft used one word — "baseline" — for the state
D0 records and the state D1 returns to. Those are mutually exclusive, and the ambiguity sat in
the one sentence that tells a rollout worker where to leave the hosts.

| subject | what it is | who produces it | is it a return target |
|---|---|---|---|
| **`EntryStateReceipt`** | the ranks as found — occupant present, ~108 GiB held | D0 | **no** |
| **`ReleasedBaselineSpec`** | the *desired* post-experiment state: no ranks, no engine, no serving offer, no seat, no host claim, no occupying container, no live device allocation | declared before D1 runs | it is the target |
| **`ReleasedBaselineReceipt`** | a readback establishing that the spec holds | D1's terminal | — |

A spec and its readback are two facts, and collapsing them is how "we returned to baseline"
becomes an assertion instead of a reading.

**What may remain.** The released state releases *processes, claims, seats and live memory* —
not bytes at rest. The exact runtime image and the content-addressed row stores may stay on
local storage. Rematerialising hundreds of gigabytes to satisfy a definition would be redundant
work, and immutable content-addressed artifacts carry no live runtime across the authorization
boundary. The spec says so explicitly rather than leaving a later reader to decide whether a
staged row store violates "released".

The **released state** remains the return target rather than the entry state, but now for a
stated reason: the entry state is a converged instance of a desired authority we are
withdrawing, so returning to it means *restoring that authority*, which is a separate decision
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

**D0 takes the exclusive claim FIRST, or brackets its readings with one.** "No incumbent was
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
| `DeclaredOccupantObserved` | it is the declared realization, converged | proceed via the authority withdrawal of §2 |
| `DeclaredOccupantDrifted` | declared, but not as declared | proceed, and record the drift; do not treat the drifted state as a target |
| `ForeignOccupantObserved` | something we do not declare | **refuse** — not ours to reclaim |
| `OccupancyUnread` | the question could not be answered | **refuse** — unread is not empty |

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
→ RETURN TO THE RELEASED STATE, unconditionally
→ read back the released state
```

D1 **always** ends at the released state — not at the entry state, which it deliberately does
not restore. It does not promote. That is what makes its idempotence receipt
meaningful — there is one desired end state, and a rerun that mutates nothing is checkable
against it.

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

**Model-independent, and genuinely a precondition.** Does the enrolled route reach a listening
process at all? `serving_enrollment` declares an endpoint, a port and a turn ceiling for
`FabricGroupA`, and nothing answers there today. Whether that path carries a request is a fact
about routing, not about V4.1, and it can be established against any responder bound to the
enrolled endpoint. It is worth doing first precisely because it is cheap and because
discovering a broken route inside a live transaction is the failure this document exists to
prevent.

Terminal: a request issued through the enrolled route reaches a process on the declared
endpoint and is answered.

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
- a second convergence run mutates no unit, image, row store or route.

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
