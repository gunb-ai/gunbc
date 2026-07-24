# The roadmap as a daily workspace — readable · observable · tactile

**Status:** DESIGN for operator review (directed 2026-07-24: "I need to be able to actually
work out of this soon"). Three pillars, each with exists/missing stated honestly. Authority
migrates to carriers as slices land; this doc is the discussion artifact.

## Thesis

The dashboard stops being a rendering of the roadmap and becomes the operator's working
surface: every task readable at a glance, every dispatched session observable while it runs,
every interaction physically acknowledged. The governing laws already exist — the register
thesis (quiet at arm's length / responsive up close / every response true) and the
behavioral-intricacy law (coverage × consistency × timing — the Discord re-read, now joined
by the Animal Crossing reference: acknowledgment is immediate, physical, and *settles*).
This plan implements them on the workspace.

## Pillar 1 — Readable (largely LANDED 2026-07-24, residue named)

- **Landed** (`2ba4d8f60c`): rows render their derived lead with the full brief behind a
  native `<details>` disclosure (188 blocks, zero JS); superseded strikethrough live (the
  raw-text serialization fix); chips no longer overflow; containers centered.
- **P1b residue:** the 3 rows whose *lead itself* exceeds 300 chars (authoring nit — tighten
  the leads on their carriers); superseded/done **sections** collapse by default with counts
  in the header ("superseded · 14"); done recedes further. Small, no dependencies.

## Pillar 2 — Observable dispatch (the stateful workflow — the daily-work blocker)

What exists: belt B has spawn/observe/reap (#6836), the single dispatch authority (#6914),
and the button already flips typed per-dispatch states (`requested / ok / refused`). What's
missing is exactly what the operator sensed: **no live session state after the dispatch
moment** — no GET /sessions surface, no Stop verb, re-dispatch after stop one-shot-broken
(all already filed on `ts-dispatch-redispatch`).

- **P2a — sessions surface:** belt B gains `GET /sessions` — (session × node × state ×
  started × last-activity), projected from the observe half's existing tmux read-back. A
  route + projection on the modeled `ServedRoute` machinery; no new architecture.
- **P2b — live rows:** a dispatched node's row shows its session state live
  (`dispatched → running → quiet Nm → done/red`), polled from P2a (poll first; push later).
  The status chip becomes the live state carrier. Stop + re-dispatch verbs land here
  (absorbing the filed row).
- **P2c — progress depth:** when the progress-observation lane lands (its JSONL event
  stream names the dashboard as renderer N+1 *by contract*), an expanded row shows the live
  phase/heartbeat — "what is it doing right now" — consuming the same events as the CI log
  renderer. Explicit dependency; no second telemetry model may grow here.

## Pillar 3 — Tactile (the feel register: defined once at the root, flows everywhere)

The operator's law: principles → design → implementation live in ONE root place and flow
through every surface. That root is the design-register library (`gunbc.design.*`), which
already carries scale (dimensions) and theme (color roles). It gains:

- **P3a — `gunbc.design.motion`:** duration tokens (press-acknowledge ~90ms · hover ~150ms ·
  settle ~250ms · state-blend 12–16 frames, reusing the theme-transition clock), easing
  tokens (standard + a settle curve — Nintendo's feel is *overshoot-and-settle*, never
  linear), transform vocabulary (press-scale, hover-lift), and the **acknowledgment law**:
  every interactive element responds to hover AND press AND every state change, in the same
  grammar, from these tokens only — a total assignment, censused and walled exactly like
  unthemed colors. Still-until-touched holds: all motion is user-caused; ambient motion
  stays unwritable (MotionTrigger has no ambient variant).
- **P3b — dashboard as first consumer:** the dispatch button's full state theater on its
  EXISTING class flips (pure CSS, no new JS): press-down acknowledgment, hover lift,
  in-flight pulse while `requested`, success settle, refused stop. Disclosure open/close
  animates; row hover responds.
- **P3c — every surface consumes the same rows:** the public site and future surfaces take
  the identical tokens (the register library's lift-not-fork discipline). **Sound is a
  named later axis** in the same shape — event-class → sound rows, one authority, opt-in
  with a modeled mute state — staged after motion because motion carries zero
  annoyance/permission risk.

## The exemplar — the dispatch button's story (operator-directed 2026-07-24: one example,
end to end, perfect transitions, representing workflow stages)

**The root principle (operator, 2026-07-24, for `gunbc.design.principles`): simulate real
analog behavior in a digital space.** Real things move and make noise; websites never get
that right. Every control is an individual physical instrument — the car-knob rule. The
consequences are mechanical, not decorative: a control has **travel** (press = downward
travel; release = spring return, which is why the settle curve overshoots), **mass**
(durations derive from implied size — small control, short travel, fast but never 0ms),
**detents** (the settle beat is the click catching), and **mechanical state** (a knob
physically sits in its position — states are stable configurations, and an idle mechanism
is STILL, which is why still-until-touched and the keyframes wall are analog honesty, not
austerity). A control that can't engage gives blocked travel — the refusal dip. Operation
sound is the mechanism's click, not a notification chime — the named later axis, now with
its design language fixed in advance.

What already exists (the register is ahead of the critique): `gunbc.design.interaction`
models verbs × responses × timing tokens with a coverage law, and the button already has
rows — Approach → border-brighten (hover+focus, 90ms) and Dispatch → press receipt
(OnActive background flip). What's missing is physical motion (responses are color-only),
an overshoot easing, and everything AFTER the click. The story:

1. **rest** — quiet confidence: figure-role border, still. *Approach*: border brightens +
   1px lift (extends the existing row with a transform response).
2. **pressed** — acknowledgment on pointer-DOWN, action on UP (the video-game rule): scale
   0.97, 90ms sharp. Extends the existing Receipt row with the transform.
3. **requested** — the POST is in flight: label → `…`, dimmed, **still** (no pulse — the
   register's keyframes wall is deliberate and stays; a looping pulse is ambient motion.
   Distinctness carries the state, not movement). Re-clicks inert.
4. **spawned** — the belt accepted: the settle beat — scale 1.0 → 1.04 → 1.0 on the new
   `settle_spring` token (~250ms, back-out curve `cubic-bezier(0.34, 1.56, 0.64, 1)`) —
   and the button MORPHS into the workflow-stage chip: `session · spawned`.
5. **working** — live stages from the sessions surface (P2a): `working → quiet 4m → …`,
   each stage CHANGE animating on `reveal` timing; each steady state still. This is where
   the button becomes the workflow representation the operator asked for.
6. **done** — settle beat + check, then the row's done-recede takes over; button returns
   to rest, re-armed (the filed re-dispatch one-shot bug is in scope — "perfect lifecycle"
   includes the second dispatch).
7. **refused** — composed refusal: boundary border + a single sharp dip-and-return (scale
   0.97 → 1.0, transition-only — no shake keyframes), typed reason surfaced inline.

**Laws applied:** every edge in the state machine is a row (state × transition × timing
token — total, censused like coverage_gaps; a state pair without a row is a wall
violation); steady states are still, transitions animate (still-until-touched preserved —
the keyframes wall is NOT breached); every rendered stage is a projection of belt/session
fact (every response true).

### END-TO-END CONTRACT (operator, 2026-07-24: "very clear — no design that is never
consumed/finished." Scope = the dispatch button + the workflow representation of ONE line
item. Everything else in Pillar 3 is PARKED until this ships.)

**The consumption rule, binding:** no register vocabulary lands ahead of its consumer —
`settle_spring`, the transform responses, and the lifecycle rows land IN THE SAME PR as the
button CSS that uses them and the witness that walls them. A register row with zero
consumers at merge is a defect, not a foundation.

**FINISHED means these six checkpoints, each observable by a person clicking, in order:**

1. Hovering the button visibly lifts/brightens it; pressing visibly depresses it —
   before release. (register vocabulary + CSS, no belt work)
2. Clicking a real node on the live srv1 dashboard: the button settles-with-overshoot on
   accept and becomes `session · spawned`. (same PR as 1)
3. The row then shows the live stage — `working`, `quiet Nm`, `done`/`red` — updating
   without a page reload, each stage change animated, each steady state still.
   (P2a-mini: one GET route projecting the belt's existing observe read-back + client poll)
4. A planted refusal shows blocked-travel + the typed reason inline. (same PR as 3)
5. After done/stop, clicking dispatches AGAIN, full arc — the re-dispatch one-shot bug is
   dead. (rides 3)
6. The state-machine totality witness reds when an edge without a timing row is planted,
   and the live-page walkthrough (1–5) is recorded as the PR's acceptance receipt.

**The single-line-item workflow representation (checkpoint 3's face):** the row IS the
workflow: `status-chip · lead · [session chip: stage · elapsed] · dispatch/stop`. One
line, at-rest still, stages as discrete detent positions — a car's gear indicator, not a
progress bar. The expanded brief (details) gains the session's stage history as plain
lines (`spawned 14:02 · working 14:02 · done 14:31`) — projections of belt facts only.

**SOUND IS IN the exemplar (operator override 2026-07-24: "I want to see/hear it
working").** Checkpoint 7: pressing the button produces the mechanism's click — a short
synthesized tick on pointer-down (a user gesture, so browser autoplay policy permits it),
a lower settle tone on spawn-accept, a dull thud on refusal. Analog law applied: these are
operation sounds of the mechanism being worked, never notification chimes; steady states
are silent exactly as idle machines are. Rows in the register (event-class → sound
parameters: frequency/duration/envelope — synthesized via WebAudio in the modeled client
program, no audio assets), same total-assignment discipline. A small mute toggle
(persisted, default on) is the minimal mute model — a real instrument can be muted, not
argued with.

**Explicitly parked until the exemplar ships:** P3c (other surfaces), section collapse
(P1b), any second control's rows. Two PRs total: PR-A = checkpoints 1–2 + 7's press-click
(register vocabulary + button theater + sound rows, no belt dependency); PR-B =
checkpoints 3–6 + 7's lifecycle tones (sessions read + poll + re-dispatch fix). If PR-B
stalls, PR-A alone must still be a strict visible-and-audible improvement — that is the
anti-shelf-ware test.

## Sequencing

P1b and P3a/P3b have no dependencies — land now. P2a/P2b are belt-B-lane work (the serve
process refresh finding from 2026-07-24 rides along). P2c waits on the observation PR by
declared contract. Order of daily-work value: **P2a/P2b first** (observability is what
"work out of this" needs), P1b + P3b as the polish pass, P3c with the register library's
Phase A.

## Non-goals

No dashboard rebuild (the emitted page + belt B stay the architecture); no second telemetry
model (P2c consumes the observation stream); no ambient animation; sound not before motion.
