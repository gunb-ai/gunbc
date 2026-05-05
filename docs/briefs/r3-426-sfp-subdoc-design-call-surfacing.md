---
status: design-call surfacing packet (NOT a worker brief)
authority parent: R3 Substrate Manager (#1739)
escalates to: Director (#828)
roadmap row: ROADMAP.md "`Strict Forward Progress` subdoc ↔ heading drift"
---

# R3 :426 — SFP subdoc design-call surfacing packet

## Why this is not a worker brief

The row's dissolution sentence offers two structurally different
shapes:

- (A) **Rename** `docs/invariants/strict-forward-progress.md` to
  reflect its actual bounded-execution content, **and** author a
  new dissolution-progress subdoc under the SFP rule name; or
- (B) **Split** the existing subdoc into two — one for bounded
  execution, one for dissolution progress — both retained under
  appropriately distinct names.

Either path is a single short PR. The choice between them is an
**SFP discipline shape question** (what does "Strict Forward
Progress" name in reviewer usage going forward? does the rule keep
its current name as the dissolution-progress invariant, or does the
name retire in favor of two co-equal subdocs?) — not a worker-tier
call. R3 Substrate Manager cannot ratify it without Director input.

## Facts on the ground (verified at HEAD, 2026-05-05)

- `docs/invariants/strict-forward-progress.md` exists; its content
  is bounded-execution discipline.
- `INVARIANTS.md` P4 and P5 prose flag the drift explicitly as
  future cleanup ("future cleanup post-#620").
- Reviewer usage of the SFP rule name in recent PR review prose
  (per parent #828's framing) carries the dissolution-progress
  meaning, not the bounded-execution meaning the subdoc actually
  holds.

## Decision the Director needs to make

Pick one:

1. **Rename + author new** — `strict-forward-progress.md` becomes
   `bounded-execution.md`; a fresh `strict-forward-progress.md` is
   authored carrying the dissolution-progress meaning that matches
   reviewer usage. Pro: keeps SFP name attached to the
   load-bearing reviewer-usage meaning. Con: any cross-reference
   to the old subdoc URL breaks; mechanical follow-up sweep.
2. **Split into two** — `strict-forward-progress.md` retains the
   SFP name and the bounded-execution content; a new
   `dissolution-progress.md` carries the dissolution-progress
   meaning. Pro: no cross-reference churn; both subdocs retain
   their existing prose. Con: SFP name continues to mean
   "bounded execution" in subdoc but "dissolution progress" in
   reviewer usage — the drift is not actually resolved, just
   renamed-around.
3. **Split + rename** — both: `strict-forward-progress.md`
   becomes `bounded-execution.md` *and* a new
   `dissolution-progress.md` is authored. Pro: removes ambiguity
   of the SFP name entirely (it retires as a subdoc title; both
   meanings get distinct co-equal titles). Con: largest
   cross-reference sweep; reviewer-usage prose adjusts too.

Worker brief authoring requires the Director to ratify one of (1) /
(2) / (3) — the brief's Slice section, file moves, and
cross-reference sweep all branch on the choice.

## What this packet provides

- Concrete enumeration of the three options with cross-reference
  cost framing.
- HEAD-verified fact baseline (subdoc content, INVARIANTS prose
  flags, reviewer-usage drift).
- Authority chain: ROADMAP row → INVARIANTS P4/P5 future-cleanup
  flags → reviewer usage post-#620.

## Next step

R3 Substrate Manager forwards this packet to Director (#828) when
the dispatch queue has Director attention. After ratification,
worker brief is authored picking up the ratified shape (`Slice`
section reflects the picked option's exact file moves and
cross-reference sweep scope).

## Provenance

Drafted during R3 host-block wait window (2026-05-05) per parent
session #828 instruction. The parent flagged that this row needs
Director ratification before worker brief authoring can begin;
this packet captures the ratification call cleanly.
