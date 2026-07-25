# The roadmap as a daily workspace — readable · observable · tactile

**Status:** DESIGN for operator review (directed 2026-07-24: "I need to be able to actually
work out of this soon"). Three pillars, each with exists/missing stated honestly. Authority
migrates to carriers as slices land; this doc is the discussion artifact.
**Normalization pass (post-merge review, 2026-07-24):** the doc accreted operator passes in
layers; this pass makes the END-TO-END CONTRACT the single controlling text (earlier layers
that disagreed — the P3b pulse, the P2b status-chip overload, the Sequencing order — are
corrected in place), repoints the dead squash hash, and records the two verified modeling
gaps (session terminal states have no observable source; the tmux read-back carries no
timestamps) so PR-B starts from the honest exists/missing line.

## Thesis

The dashboard stops being a rendering of the roadmap and becomes the operator's working
surface: every task readable at a glance, every dispatched session observable while it runs,
every interaction physically acknowledged. The governing laws already exist — the register
thesis (quiet at arm's length / responsive up close / every response true) and the
behavioral-intricacy law (coverage × consistency × timing — the Discord re-read, now joined
by the Animal Crossing reference: acknowledgment is immediate, physical, and *settles*).
This plan implements them on the workspace.

## Pillar 1 — Readable (largely LANDED 2026-07-24, residue named)

- **Landed** (#7169, merge `b867e9b58` — the branch hash this line first cited was destroyed
  by the squash-merge): rows render their derived lead with the full brief behind a native
  `<details>` disclosure (188 blocks, zero JS); superseded strikethrough live (the raw-text
  serialization fix); chips no longer overflow; containers centered.
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
  started × last-activity). The route is genuinely small (a `RoadmapServeRouteSpec` row + a
  handler variant on the fail-closed table in `roadmap_serve.dag`). **Honesty correction
  (verified 2026-07-24):** the existing read-back (`dispatch_live_sessions`) parses only
  `session_name / node_id / windows / lease_verdict` — `started` and `last-activity` do NOT
  exist today. The modeled tmux observation must grow (`#{session_created}` /
  `#{session_activity}` format fields + parse rows) on the fail-closed observe path — small,
  but real observation work, not a pure projection.
- **P2b — live rows:** a dispatched node's row shows its session state live, polled from
  P2a (poll first; push later). Session state renders in its **own session chip** — the
  exemplar's line-item layout is the authority; the status chip stays the node-status
  carrier (overloading it would conflate node status × session state, the house's own
  `Option`/`None` pattern; the earlier "the status chip becomes the live state carrier"
  wording is superseded). Stop + re-dispatch verbs land here (absorbing the filed row).
  **The terminal-state gap (G1, the load-bearing missing piece):** a session that exits
  DISAPPEARS from `tmux ls` — today the belt cannot distinguish done-success from crashed
  from reaped, so `done`/`red` have **no observable source** and rendering them from
  absence would fabricate (violates *every response true*). Decision required before PR-B,
  three options: (a) tmux `remain-on-exit` + dead-pane exit-status read (changes reap
  semantics — lingering sessions interact with the re-dispatch debris fix); (b) a
  worker-written completion receipt in the worktree, read at observe time (survives session
  death, composes with the teardown-owns-worktree fix, gives a typed red with exit code —
  **recommended**); (c) ship the honest vocabulary now — terminal state `gone` + the node's
  own status — and land exit modeling as its own named row. Whichever lands, absent-session
  stays a distinct state from done and from red, never collapsed.
- **P2c — progress depth:** when the progress-observation lane lands (its JSONL event
  stream names the dashboard as renderer N+1 *by contract*), an expanded row shows the live
  phase/heartbeat — "what is it doing right now" — consuming the same events as the CI log
  renderer. Explicit dependency; no second telemetry model may grow here. **Two-source
  composition rule:** the belt's supervisor view (tmux liveness, P2a) and the stream's
  self-report (P2c) are different subjects that can legitimately disagree (belt: running;
  stream: quiet 120s) — not a fork, but the row UI composes them, and the composition rule
  is a row (supervisor liveness gates existence; stream detail fills depth; disagreement
  renders as its own named state), never an ad hoc merge.

## Pillar 3 — Tactile (the feel register: defined once at the root, flows everywhere)

The operator's law: principles → design → implementation live in ONE root place and flow
through every surface. That root is the design-register library (`gunbc.design.*`), which
already carries scale (dimensions) and theme (color roles). It gains:

- **P3a — `gunbc.design.motion`:** duration tokens — **reuse the existing scale first**
  (`respond_fast` 90ms for press-acknowledge, `reveal_deliberate` 160ms for hover — the
  earlier "~150ms" was a near-miss mint of a token that already exists), state-blend
  12–16 frames reusing the theme-transition clock. `settle_spring` (~250ms, overshoot
  bezier) is a legitimate NEW token but its justification is the **easing axis** — it sits
  one row from `restore_gentle` (240ms ease) on duration, so the row records that the mint
  is the overshoot curve, not a tenth duration (Nintendo's feel is *overshoot-and-settle*,
  never linear). Transform vocabulary (press-scale, hover-lift), and the **acknowledgment
  law**: every interactive element responds to hover AND press AND every state change, in
  the same grammar, from these tokens only — a total assignment, censused and walled
  exactly like unthemed colors. **The law's "every state change" clause is a register law
  AMENDMENT, land it as one:** still-until-touched today means motion is user-caused;
  the dashboard extends it to *motion only on user touch OR observed state change* (analog
  honesty — a real mechanism moves when its state changes), with ambient motion still
  unwritable. Mechanically: `MotionTrigger` is `OnHover | OnFocusVisible | OnActive` —
  pseudo-classes — while state-change motion triggers on **state classes** (the
  `dispatch-*` class flips), a different selector algebra; the new trigger class is the
  real modeling work of P3a, landed as a law row + trigger variant with the button as its
  first consumer in the same PR (the consumption rule). **The total assignment includes a
  `prefers-reduced-motion` projection** (instant flips, zero travel — a register axis
  realized once, not a per-site afterthought).
- **P3b — dashboard as first consumer:** the dispatch button's full state theater on its
  EXISTING class flips: press-down acknowledgment, hover lift, `requested` still-and-
  distinct (**no pulse** — corrected to match the exemplar; a looping pulse is ambient
  motion and the keyframes wall holds), success settle, refused stop. "No new JS" restated
  honestly: the theater rides class flips in CSS, but the morph label (`session · spawned`)
  is a small emitted-TS text change in `roadmap_component` — no new client *architecture*,
  not literally zero JS edits. Disclosure open/close animates **via a named mechanism**
  (`::details-content` + `interpolate-size` — Chrome-target, fine for the operator
  dashboard; if that's declined, the disclosure stays unanimated rather than growing JS);
  row hover responds.
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
   includes the second dispatch). *Source honesty: `done` (and `red`) render only per the
   P2b terminal-state decision (G1) — never derived from session absence.*
7. **refused** — composed refusal: boundary border + a single sharp dip-and-return (scale
   0.97 → 1.0, transition-only — no shake keyframes), typed reason surfaced inline.

**Laws applied:** every edge in the state machine is a row (state × transition × timing
token — total, censused like coverage_gaps; a state pair without a row is a wall
violation); steady states are still, transitions animate (still-until-touched preserved —
the keyframes wall is NOT breached); every rendered stage is a projection of belt/session
fact (every response true).

**The detent table is load-bearing, pin it before any CSS (mechanical constraint, found in
review 2026-07-24):** CSS transitions fire only when a property CHANGES — an edge between
two states holding identical transforms cannot beat without keyframes (walled) or a JS
transient class. So the state rows must assign **each state a distinct stable transform
value** — the detent metaphor made literal (e.g. pressed 0.97 · requested ~0.985 "engaged,
in flight" · terminals 1.0 · each workflow stage its own resting value) — which makes every
edge's travel real and the settle beat *derivable* from the transition + overshoot curve,
never choreographed. The same discipline names the **morph mechanism** up front: the
button→chip morph needs `interpolate-size: allow-keywords` (or fixed measures per state) to
transition auto-width in pure CSS — decide which, don't discover it on landing day.

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
3. The row then shows the live stage — `working`, `quiet Nm`, and the terminal states per
   the G1 decision (`done`/`red` only once exit is modeled; `gone` until then) — updating
   without a page reload, each stage change animated, each steady state still.
   (P2a-mini: one GET route + the tmux observation growth from P2a + client poll)
4. A planted refusal shows blocked-travel + the typed reason inline. (same PR as 3)
5. After done/stop, clicking dispatches AGAIN, full arc — the re-dispatch one-shot bug is
   dead. (rides 3)
6. The state-machine totality witness reds when an edge without a timing row is planted;
   the keyframes wall extends to the dashboard emission (today's witness covers the
   moodboard only); and the live-page walkthrough (1–5) is recorded as the PR's acceptance
   receipt. **Walkthrough precondition:** the served-page fingerprint check from the
   2026-07-24 stale-styling incident (deploy refreshed files; the serve path's
   artifact/process refresh was unproven) lands with or before PR-B — otherwise the
   acceptance demo can show stale assets, the exact failure that motivated the check. File
   it as a belt-B row per `ts-intake-discipline`; it is currently prose-only.

**The single-line-item workflow representation (checkpoint 3's face):** the row IS the
workflow: `status-chip · lead · [session chip: stage · elapsed] · dispatch/stop`. One
line, at-rest still, stages as discrete detent positions — a car's gear indicator, not a
progress bar. `elapsed` is quantized to minutes — a per-second tick is ambient churn, and
quiet-at-arm's-length governs text exactly as it governs motion. The expanded brief
(details) gains the session's stage history as plain lines
(`spawned 14:02 · working 14:02 · done 14:31`) — projections of belt facts only.

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

## Sequencing — SUPERSEDED by the END-TO-END CONTRACT (kept as the record)

This section predates the exemplar contract and disagrees with it on two points, so the
contract controls: delivery is **PR-A then PR-B** (button theater first, sessions second —
not "P2a/P2b first"), and **P1b is parked** (not "land now"), along with P3c, sound, and
any second control's rows, until the exemplar ships. What survives from this section: P2c
waits on the observation PR by declared contract, and the serve-refresh finding rides PR-B
as a precondition of its live walkthrough (checkpoint 6). P3c lands with the register
library's Phase A when unparked.

## Non-goals

No dashboard rebuild (the emitted page + belt B stay the architecture); no second telemetry
model (P2c consumes the observation stream); no ambient animation; sound not before motion.
