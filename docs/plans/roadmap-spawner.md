# Roadmap-as-spawner (gunbc → ctrl)

Turn the gunbc roadmap `.dag` from a post-hoc PR-merge projection into the **work-tracking
DAG that drives ctrl session spawns**. The roadmap authority becomes the single source for
*what work exists and what is ready*; ctrl becomes a thin runtime that *executes* the emitted
ready-set and reports runtime facts back. This kills the dual authority between "our work" and
"the docs" (DESIGN §3) and is the first inhabitant of migrating ctrl onto the substrate (§7:
the realization is a seed that shrinks to zero; host-effect-orchestration.md Phase D/E).

## The structure/runtime split (why this is single-authority)

Two *different kinds* of fact, each with exactly one home — not two copies of one fact:

- **Graph structure** — what work exists, dependencies, sizing, acceptance conditions. Home:
  the gunbc `.dag` authority (`dag/gunbc/roadmap/roadmap_authority.dag` + `roadmap_model.dag`). Edited
  by committing the `.dag` to main. ctrl never authors structure.
- **Runtime state** — is a node actually done, is a session live. Home: ctrl/GitHub. Flows
  **one direction, ctrl → gunbc, read-only**, as acceptance *evidence* (PR merged, session
  archived). gunbc reads it; it does not store it.

So: **planning edits = git commits to the `.dag`**; a wrong plan is fixed in the `.dag` and
the ready-set recomputes next tick. ctrl is a consumer + fact-reporter, never a co-author.

## Anchor

The spawner reads the roadmap authority at **main HEAD** and re-reads each tick (lockfile
pattern, as in the RoadmapGate work — `DerivableLine` derived from a committed merged-set).
Planning and code go through the same PR gate to main.

## Division of labor (the readiness computation splits cleanly)

- **gunbc** computes *graph readiness*: `not-done ∧ all-deps-done`. It knows the DAG and the
  merged-PR / acceptance facts. It does NOT know ctrl's live sessions.
- **the ctrl bridge** computes *runtime dedup*: `no-active-session-for-this-node`. It has the
  session list. It does NOT re-derive graph readiness.

This mirrors the structure/runtime split exactly, so neither side duplicates the other.

## The frozen interface contract (build both lanes against THIS)

### 0. v1 → v2: WHAT A ctrl-SIDE READER MUST DO (read this first)

**If your consumer pins `roadmap-spawn-request/v1` or `roadmap-dispatch/v1`, it will now REFUSE
the payload outright — it will not see a v1 object with two fields missing.** That refusal is
deliberate and is the whole point of the bump: a v1 consumer is entitled by this document to read
`intricacy` and `volume`, so handing it an object without them under the v1 name would be a silent
contract change. Refusing is loud; a missing field read as an absent tier is not.

**What changed:** sizing stopped being a field an author types on a roadmap node and became a fact
derived from the task (`gunbc.roadmap_sizing`). Its deriver is a stub that currently refuses for
every subject, so `intricacy` and `volume` are absent from **every** object today. They are never
defaulted or substituted — a substituted cell would publish the stub's silence as if it were a
measurement.

**What ctrl must do to move to v2**, in order:
1. Accept the schema strings `roadmap-spawn-request/v2` and `roadmap-dispatch/v2`.
2. Treat `intricacy` and `volume` as **conditional and paired**: both present, or both absent.
   Never one without the other.
3. Decide, explicitly and on ctrl's side, what the tier grid does with *no declared size*. That is
   a ctrl policy question and this document does not answer it — but it must be an authored
   decision, not a default that arises from reading a missing key as a zero or a minimum.
4. Expect the keys to **return** under v2 with no further schema change, the day the deriver
   answers. v2 declares them conditional, so their reappearance is within the contract.

**When do they come back?** When the drop `gunbc.rung_drop.roadmap_sizing_authored_to_derived`
retires. Its trigger is the derivation capability itself — a derivation that reads a task and
answers from it — and it explicitly cannot be satisfied by any function that merely returns a cell.

### 1. Spawn-request artifact — gunbc emits, ctrl consumes

A gunbc CLI entry evaluates `next_spawnable` over the authority at HEAD and writes JSON:

```json
{
  "schema": "roadmap-spawn-request/v2",
  "anchor_commit": "<the main sha the authority was read at>",
  "ready": [
    {
      "node_id": "5-cargo-green",
      "title": "emitted crate cargo-builds green (Route-A last mile)",
      "repo": "gunbc",
      "intricacy": "high",
      "volume": "medium",
      "parent_node_id": null,
      "plan_doc": "docs/plans/...md",
      "acceptance": { "kind": "prs_merged", "prs": [5777] }
    }
  ]
}
```

- `node_id` — the `ProcessNodeId` from the roadmap authority. Stable dedup key.
- `intricacy` ∈ {low,medium,high}, `volume` ∈ {small,medium,large} — drive ctrl's tier grid.
  **v2: PRESENT ONLY WHEN A SIZE WAS DERIVED, and absent together or not at all.** Sizing stopped
  being an authored field on the roadmap node and became a derived one (`gunbc.roadmap_sizing`);
  while its deriver is a stub that refuses, these two keys are absent from every object. The schema
  string moved v1 → v2 for exactly this reason: v1 entitles a consumer to read them, so emitting
  objects without them under the v1 name would be the DESIGN §3 meaning fork — one spelling, one
  declared version, materially different obligations. A v1-pinned consumer should refuse a v2
  payload rather than read an absent tier. The loss is declared at
  `gunbc.rung_drop.roadmap_sizing_authored_to_derived`, whose trigger is the derivation capability
  itself; when that lands, the keys return under v2 without another schema change, because v2
  declares them conditional.
- `parent_node_id` — null for top-level, else the composite parent (`RoadmapEdge`).
- `acceptance` — `{kind:"prs_merged", prs:[...]}` or `{kind:"manual"}` for MVP. The coproduct
  generalizes later (`witness_green`, `artifact_exists`).
- Only **graph-ready** nodes appear (not-done ∧ deps-done). Runtime dedup is the bridge's job.

### 2. Pause control — the kill switch (fail-closed, §5)

One control the operator flips in **one action, no gunbc commit/regen required**, so a
"things got weird" pause is instant:

```json
// ctrl-side control file, polled by the bridge each tick
{ "schema": "roadmap-spawn-control/v1", "paused": true }
```

- **Default `paused: true`** — fail-closed: the loop spawns NOTHING until the operator
  explicitly un-pauses. A fresh deploy, a parse error, a missing control file → paused, never
  "spawn freely."
- One-liner to pause/resume (bridge lane defines the exact command + dashboard toggle).
- When paused, the bridge reads the ready-set but emits zero spawns and logs the held count.

## Stage 1 (this MVP) — slowly spawn + monitor

1. **gunbc lane** — add `intricacy`/`volume`/`repo` to `RoadmapNode`; an `Acceptance` field
   (MVP arms `PrsMerged`/`Manual`); `next_spawnable(authority, merged_prs)` pure fold; a CLI
   entry emitting the `roadmap-spawn-request/v1` JSON (now v2 — see the contract note above). Reuse `authored_merged_prs()` for the
   merged facts. Witness: a fixture authority with a satisfied-dep node appears in `ready`, an
   unsatisfied-dep node does not (discriminating RED).
2. **ctrl bridge lane** — a thin loop (ctrl JS, Stage 1): poll the pause control; if unpaused,
   read the gunbc ready-set, dedup against live sessions, `POST /api/internal-work-items`
   (owner = a designated manager session) with title + intricacy + volume + repo; ctrl's
   EXISTING auto-spawn poller spawns + monitors. Zero new spawn code — reuse the poller,
   capacity gate, and respawn circuit-breaker. Honor `paused`. Log held/spawned counts.

Acceptance stays `PrsMerged`/`Manual` in Stage 1 — no acceptance automation yet. Watch the
dashboard tree; pause anytime.

## Stage 2+ (later, not this MVP)

- Move readiness fully into `.dag`; ctrl's `nodes` table becomes a projection/cache of the
  emitted ready-set (its early-iteration work-item DAG subordinated to the `.dag` one).
- Model spawn + session-management as `.dag` host-effects over the `apply()` seam from
  host-effect-orchestration.md — `apply(SpawnSession{...})`, `apply(CloseSession{...})`. The JS
  `POST /api/sessions` becomes one transport handler bound to a modeled effect (N handlers, §2).
  `code_change_workflow.dag` already models the PR/session lifecycle. ctrl JS shrinks to a thin
  runtime; metric = ctrl LOC deleted (Phase D "begin ctrl deletion").

## Why the integration is good (not a forced fit)

The roadmap node already maps almost 1:1 onto ctrl's work-item fields; `gunbc.process_algebra`
is already the `.dag` mirror of ctrl's `nodes` table (same Leaf/Composite/Bucket, same
Declare/Decompose/Close). The only genuine adds are sizing, the `Acceptance` coproduct, and the
emit seam — small and additive. That 1:1 mapping IS the evidence the integration is real.
