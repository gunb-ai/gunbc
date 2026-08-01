# The roadmap presentation seam — one projection between the graph and the page

**Status:** DESIGN for the A2 slice (operator-directed 2026-08-01, following the A1 subtraction
pass, PR #7540). A1 removed false emphasis; this slice adds the missing layer that decides *what
matters, why it is shown, and how strongly it speaks*. The governing framing (operator):

> A stronger interface makes fewer claims, makes each claim at the right altitude, and shows the
> evidence or consequence that makes the claim matter.

A1 improved the first part. This slice owns the second and third. Doctrine line that names the
gap: **truthful atoms do not automatically compose into a truthful or intelligible presentation**
— hierarchy, causality, temporal relation, and relevance must themselves be modeled.

## 1. The falsifier specimen (acceptance test #1, ahead of all other presentation work)

A live session on the daily workspace. Verified mechanics of the failure, against the tree at
A1's head:

- `gunbc.roadmap_component` `roadmap_row_archetype` declares five regions — StatusChip, Title,
  Meta, Disclosure, Actuator — and places `ActuatorRegion` at `HeadTrack{AutoTrack}`. The derived
  grid is `auto 1fr auto` (`roadmap_row_grid_template`).
- `roadmap_row_archetype_note` says the fusion was deliberate: workflow strip, process evidence,
  cleanup, and the dispatch control all land inside the one `workflow-controls` wrapper "to
  preserve the derived auto 1fr auto grid" and to avoid "an implicit fourth grid item."
- An `auto` grid track takes the intrinsic width its content demands. With a live attempt the
  wrapper holds: seven obligation chips + the activity text + Session/Attempt lamps + the
  `session · present` caption + the `process exited` chip + `clear`. The track expands until the
  `1fr` title receives the scraps — a realistic title collapses to ~one word per line **on a wide
  screen**. `flex-wrap` inside the wrapper cannot help; the wrapper itself sizes the track.
- The stacked realization (`workflow_controls_compact_css`) fires only below
  `workflow_controls_compact_breakpoint` (640px), so the pathological wide state never reaches it.
- The active-state content is **client-painted**: `dispatch_client_program` writes each
  obligation's observed state from `/workflow.json` into the head-track cell
  (`workflow_progress_note`: presentation over evidence, single-flight polling). So the seam must
  govern both the server projection and the client program's render targets.

The screenshot therefore falsifies the **archetype's region model**, not its spacing: evidence
about an action and the action itself occupy one semantic region because the layout had one cell
left. The current representation says preserving `auto 1fr auto` outranks the primary work item's
readability. That is backwards.

**A2 is not complete until this specimen — active attempt, all seven obligation nodes, latest
activity text, session present, attempt present, process exited, cleanup action, realistically
long title — produces a coherent hierarchy at wide, medium, and narrow widths.**

### Prohibited cheap fixes (each preserves the weak argument while making it fit)

No smaller type · no tighter gaps · no further stage abbreviation · no later breakpoint alone ·
no horizontal scrolling · no squeezing the title · no ellipsis-hiding · no wrapping the same
undifferentiated facts into two equally noisy lines.

## 2. What the specimen row currently claims, and why that is a weak sentence

Simultaneously, as peers: open · ready · three obligations evidenced · four pending · agent turn
completed · a session container exists · an attempt exists · the worker process exited · the
session can be cleared · the prior dispatch produced a session. Every atom individually true;
composed as peers they have no main proposition. Four named defects:

1. **An obligation ledger presented as a progress meter.** Environment/Worktree/Agent/Verify/
   Publish/Review/Audit are independently evidenced obligations (`workflow_progress_note`: a
   provider turn completing cannot advance downstream obligations) — not seven interchangeable
   increments. The horizontal stepper visually asserts "stage 3 of 7 ≈ 43%", a scalar the graph
   has not established. **The view model deliberately has no `progress_percent` field**; a scalar
   appears only if the graph ever establishes one.
2. **Action and observation fused.** `session · present` occupies the control position the
   dispatch button owns; the same region morphs between operation-available, request-acknowledged,
   and observation. A button must not turn into a status report because both fit the cell.
3. **Evidence pairs shipped uninterpreted.** "session container present" beside "process exited"
   requires tmux semantics to reconcile. The projection states the consequence — *"Agent turn
   completed; the retained session is no longer running"* — and holds the container/process facts
   as disclosed evidence.
4. **Active work buried in the idle queue.** A row with an accepted dispatch, live evidence, and
   remaining obligations is the most temporally salient item on the page, yet renders as the first
   ordinary member of `ready · 10 open`. The node may remain structurally Ready; the **view**
   derives attention: `Active work · 1` above `Ready · 9`. The graph stays authoritative; the
   camera chooses.

## 3. The model boundary

One projection module (working name `gunbc.roadmap_presentation`) between the authorities and the
renderers. The renderer receives an **already-decided presentation** — no punctuation splitting,
no title reconstruction, no policy on string content, no availability-implies-emphasis, no
evidence smuggled into an actuator region, no global counts under local headings.

```
DailyWorkspaceView { context, attention, active, ready, upcoming, done, superseded, program_progress }

WorkRowView {
  primary            — headline (+ optional purpose-specific supporting claim)
  scheduling         — frontier position (ready/upcoming/…)
  lifecycle          — open/review/done/superseded
  activity           — ActivityView
  action             — ActionView
  supporting_facts   — owner, sizing, carriers (disclosed)
  disclosure         — complete detail
}

ActivityView
  = NoActiveWork
  | ActiveWork { summary, current_obligations, remaining_obligation_count,
                 latest_observation, attention, evidence }
  | ActiveWorkRefused { summary, located_reason, evidence }

ActionView { primary_action, secondary_actions }
  — rendered prominence derives from ActionAvailability × ActionPriority;
    availability never implies visual primacy.

ObservationImpactView { primary_claim, affected_channels, evidence, diagnostic_detail }
  — one root cause narrated once, not once per observation channel; three altitudes
    (what the user cannot know/do · which observation failed · command/path/stderr).
```

Semantic regions, and the correction that dissolves the specimen's failure:

```
PrimaryRegion    — what this work is
ActivityRegion   — what is happening to it        ← NOT a kind of ActuatorRegion
ActionRegion     — what the user can do
EvidenceRegion   — why the activity claim is believed
DisclosureRegion — complete detail
```

Structurally: `roadmap_row_archetype` gains `ActivityRegion` placed **`UnderHead`** (the placement
vocabulary already exists — Disclosure uses it), and `ActuatorRegion` shrinks to actions only. The
head grid stays `auto 1fr auto` with a *small* third track; the activity summary and ledger render
under the head where width is not contested with the title.

TicketFields already carries `headline` and `brief` as typed fields; `WorkRowView.primary`
consumes them directly, deleting the `**headline** — brief` recompose-and-reparse round trip
(`line_title`/`line_lead`) for ticket rows. Legacy line variants keep the lead heuristic as a
named residue inside the projection, nowhere else.

## 4. The attention rule (strict)

- **No active attempt** → no workflow furniture at all: no hollow lamps, no seven pending chips.
  Absence is routine and earns no permanent row area.
- **Active attempt, ordinary** → one activity summary ("Agent working" / "Agent turn completed ·
  verification next"); the obligation ledger by expansion.
- **Blocked / failed / refused** → the relevant evidence expands automatically with the located
  cause.
- **Completed** → the receipt-backed outcome, not the machinery.

This is the interface form of the existing altitude law (`gunbc.roadmap_altitude`): routine
collapses, active work is legible, anomaly expands.

## 5. First discriminating consumers (the slice's witness set)

1. The falsifier specimen renders a coherent hierarchy at three widths (sandbox composition
   fixture — constructed, never live-population; the first composition exhibit in a sandbox that
   currently audits only atoms).
2. Headline and summary remain distinct typed fields end-to-end (no punctuation recovery).
3. Scheduling and lifecycle remain distinct axes.
4. Override availability does not imply primary prominence.
5. Program-local progress never consumes global roadmap counts.
6. The daily workspace begins with operational attention, not strategic narrative.
7. One failure affecting several observation channels is narrated once (ObservationImpactView),
   with the located cause under disclosure.
8. No-attempt rows render zero activity/evidence DOM.
9. A work family is stated once; rows carry the discriminating subject (family grouping may stage
   behind the core seam if the slice grows too large — staged, named, never silently dropped).

## 6. What must not harden (scope guard)

The flat row list, `ticket` vocabulary, always-visible telemetry, the single green pill, and
fragment-list page order are **provisional renderer facts**, not product ontology (carried on
`gunbc.roadmap_page.a1_composition_provisional_note`). The product vision's constraint governs:
the graph is the world, the UI is a camera; projections derive attention; the core vocabulary
stays Intent/Node/Dependency/Actor/Capability/Operation/Observation/Refusal/Receipt/Revision/
Projection. The same view facts must later support a compact list, a mission view, semantic zoom,
actor-local projections — the list is one renderer, never the model.

## 7. Sequencing

Single PR on top of A1 (#7540): design note → seam module + archetype ActivityRegion →
`roadmap_page` consumes (serialize-only rows) → `dispatch_client_program` renders ActivityView
under-head with the action/evidence split → specimen fixture + witnesses + screenshot receipts.
Related earlier artifacts: [roadmap workspace UX plan](roadmap-workspace-ux-plan.md) ·
[workspace remodel plan](roadmap-workspace-remodel-plan.md) ·
[gunbc-served dashboard design](gunbc-served-dashboard-design.md).
