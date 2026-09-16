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

### 2. The rollback target must change, and the honest one is weaker

*Restore V4 Flash and prove one real request through it* is unsatisfiable: V4 Flash is not
there to restore. The available targets are:

- **restore the occupant** — reproduce the `gunbc-spark-pair` Ray container. Rejected: it is a
  leftover with no consumer, we did not create it and do not hold a declaration that produces
  it, so "restore" would mean *reconstruct from an observation*, which is not a restoration
  and cannot be verified against anything;
- **release to a verified empty baseline** — hosts carrying no serving process, no occupant,
  and memory returned to its idle reading. **This is the target.** It is weaker than the
  original but it is checkable, and it is the state the experiment must be able to reach from
  any failure arm.

So the rollback proof changes shape: from *the incumbent serves again* to *the hosts return to
a declared baseline and that baseline is read back*. Cut D must not claim the stronger one.

### 3. The risk profile inverts, and that makes P1 Cut 3 more important rather than less

The original cut's danger was **breaking a live service**. That danger is gone. The remaining
danger is **not being able to give the hosts back** — a partial or stalled launch leaving
ranks up, seats held, and 121 GiB per node unavailable, with no incumbent whose restoration
would incidentally clear it.

The original brief said P1 Cut 3 "should block" ordinary production promotion. Against an
incumbent, that hedge was defensible because restoring the incumbent was itself a cleanup path.
Without one it is not: **promotion must refuse** unless P1 Cut 3 lands or an equivalent cleanup
realization is modeled, authorized, and demonstrated to stop partial ranks, withdraw offers,
release every held seat, and return the hosts to baseline.

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

### D0 — capture the baseline as a receipt (no mutation)

Before anything is stopped, record per rank: occupant container identity and creation time,
memory used and free, absence of a serving process, absence of an answer on the enrolled
endpoint, and free disk. This is simultaneously the **rollback reference**, the evidence that
the transaction changed what it claims to have changed, and the record that makes the occupant
reconstructible-in-principle if the decision in §2 is ever revisited.

Terminal: a baseline receipt exists for all four ranks, and no host state was altered.

### D1 — reclaim, bounded experiment, mandatory return to baseline

Authorization is exact and time-bounded over: candidate key, the four host identities, the
**produced** runtime image digest, model source identity, the four row-store identities, the
TP4 profile, the selected Engram route, the D0 baseline receipt, and the cleanup path of §3.
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
→ RETURN TO BASELINE, unconditionally
→ read back the baseline
```

D1 **always** ends at baseline. It does not promote. That is what makes its idempotence receipt
meaningful — there is one desired end state, and a rerun that mutates nothing is checkable
against it.

Weight loading is not success. Acceptance begins after runtime initialisation and one valid
semantic completion. An HTTP 200 discharges nothing.

Every stage capable of preventing a later observation carries a terminal disposition: ranks
stopped, claims released, hosts at baseline, baseline read back — or a typed refusal naming the
missing cleanup evidence. A failure arm that cannot reach baseline is the one outcome that must
stop the line loudly, because it is the risk this cut actually carries.

### D1a — the routed request, as its own precondition

Establishing that the enrolled Group A route carries a request is separable from V4.1 and is
currently unproven for any model. It is listed before D2 because D2 depends on it and because
discovering it inside a live transaction is the failure this document exists to prevent.

Terminal: one request reaches a process on the enrolled endpoint and is answered, joined
end-to-end — router request, selected offer, authorization, four agreeing ranks, produced
completion.

### D2 — production promotion, separately authorized

A new authorization over the already-proven realization. Promotion **refuses** unless:

- the candidate is realized, not merely keyed — produced image digest, applied patch
  population, four row-store identities;
- row-store faithfulness is established over the **published** population, not a fixture;
- D1 completed and returned to baseline;
- D1a established that the enrolled route carries a request;
- **P1 Cut 3 or a demonstrated equivalent cleanup path exists**;
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
