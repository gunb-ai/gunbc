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

## Sequencing

P1b and P3a/P3b have no dependencies — land now. P2a/P2b are belt-B-lane work (the serve
process refresh finding from 2026-07-24 rides along). P2c waits on the observation PR by
declared contract. Order of daily-work value: **P2a/P2b first** (observability is what
"work out of this" needs), P1b + P3b as the polish pass, P3c with the register library's
Phase A.

## Non-goals

No dashboard rebuild (the emitted page + belt B stay the architecture); no second telemetry
model (P2c consumes the observation stream); no ambient animation; sound not before motion.
