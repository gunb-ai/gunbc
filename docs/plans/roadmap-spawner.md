# Roadmap-as-spawner (gunbc → ctrl)

Turn the gunbc roadmap `.dag` from a post-hoc PR-merge projection into the **work-tracking
DAG that drives ctrl session spawns**. The roadmap authority becomes the single source for
*what work exists and what is ready*; ctrl becomes a thin runtime that *executes* the emitted
ready-set and reports runtime facts back. This kills the dual authority between "our work" and
"the docs" (DESIGN §3) and is the first inhabitant of migrating ctrl onto the substrate (§7:
the realization is a seed that shrinks to zero; host-effect-orchestration.md Phase D/E).

## The structure/runtime split (why this is single-authority)

Two *different kinds* of fact, each with exactly one home — they are not two copies of one fact:

- **Graph structure** — what work exists, dependencies, sizing, acceptance conditions. Home:
  the gunbc `.dag` authority (`dsl/gunbc/roadmap_authority.dag` + `roadmap_model.dag`). Edited
  by committing the `.dag` to main. ctrl never authors structure.
- **Runtime state** — is a node actually done, is a session live. Home: ctrl/GitHub. Flows
  **one direction, ctrl → gunbc, read-only**, as acceptance *evidence* (PR merged, session
  archived). gunbc reads it; it does not store it.

So: **planning edits = git commits to the `.dag`**; if a plan is wrong you fix the `.dag` and
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

This split mirrors the structure/runtime split exactly, so neither side duplicates the other.

## The frozen interface contract (build both lanes against THIS)

### 1. Spawn-request artifact — gunbc emits, ctrl consumes

A gunbc CLI entry evaluates `next_spawnable` over the authority at HEAD and writes JSON:

```json
{
  "schema": "roadmap-spawn-request/v1",
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
- `parent_node_id` — null for top-level, else the composite parent (`RoadmapEdge`).
- `acceptance` — `{kind:"prs_merged", prs:[...]}` or `{kind:"manual"}` for MVP. The coproduct
  generalizes later (`witness_green`, `artifact_exists`).
- Only **graph-ready** nodes appear (not-done ∧ deps-done). Runtime dedup is the bridge's job.

### 2. Pause control — the kill switch (fail-closed, §5)

A single control the operator flips in **one action, no gunbc commit/regen required** (so a
"things got weird" pause is instant):

```json
// ctrl-side control file, polled by the bridge each tick
{ "schema": "roadmap-spawn-control/v1", "paused": true }
```

- **Default `paused: true`** — fail-closed: the loop spawns NOTHING until the operator
  explicitly un-pauses. A fresh deploy, a parse error, a missing control file → treated as
  paused, never as "spawn freely."
- One-liner to pause/resume (bridge lane defines the exact command + dashboard toggle).
- When paused, the bridge reads the ready-set but emits zero spawns and logs the held count.

## Stage 1 (this MVP) — slowly spawn + monitor

1. **gunbc lane** — add `intricacy`/`volume`/`repo` to `RoadmapNode`; an `Acceptance` field
   (MVP arms `PrsMerged`/`Manual`); `next_spawnable(authority, merged_prs)` pure fold; a CLI
   entry emitting the `roadmap-spawn-request/v1` JSON. Reuse `authored_merged_prs()` for the
   merged facts. Witness: a fixture authority with a satisfied-dep node appears in `ready`, an
   unsatisfied-dep node does not (discriminating RED).
2. **ctrl bridge lane** — a thin loop (ctrl JS, Stage 1): poll the pause control; if unpaused,
   read the gunbc ready-set, dedup against live sessions, `POST /api/internal-work-items`
   (owner = a designated manager session) with title + intricacy + volume + repo; ctrl's
   EXISTING auto-spawn poller spawns + monitors. Zero new spawn code — reuse the poller,
   capacity gate, and respawn circuit-breaker. Honor `paused`. Log held/spawned counts.

Acceptance stays `PrsMerged`/`Manual` in Stage 1 — no acceptance automation yet. You watch the
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

The roadmap node already maps almost 1:1 onto ctrl's work-item fields. `ctrl.process_algebra`
is already the `.dag` mirror of ctrl's `nodes` table (same Leaf/Composite/Bucket, same
Declare/Decompose/Close). The only genuine adds are sizing, the `Acceptance` coproduct, and the
emit seam — small and additive. That 1:1 mapping IS the evidence the integration is real.
