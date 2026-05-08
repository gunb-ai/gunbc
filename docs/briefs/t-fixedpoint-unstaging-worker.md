# T-FixedPoint unstaging worker — CLOSED AS REDUNDANT (routing doc)

> **Status**: CLOSED AS REDUNDANT 2026-05-08. This file is a **routing doc**, not
> an authority. Single-authority for T-FixedPoint per INVARIANTS.md P2 lives at
> [`r3-pb-t-fixedpoint-worker.md`](r3-pb-t-fixedpoint-worker.md) (PB Manager,
> authored 2026-04-29; P0/P1/P2/P3 dispatch-gated brief).

## What happened

This file was originally authored 2026-05-08 by bright-raven-819 at PB Mgr
direction (parent inbox #2074 / dashboard inbox #2136 #issuecomment-4403050144) as
a "pre-staged self-compile validation pipeline brief, mechanical-dispatch-ready
for SG-0 zero arrival." The 5-question pre-author audit (per
[`brief-authoring-checklist.md`](brief-authoring-checklist.md)) Q2 ran a `docs/briefs/`
grep for `fixed.point|self.host|self.comp`, which did NOT match the canonical brief's
filename token form `t-fixedpoint` (hyphen instead of dot/space) — so the canonical
authority was missed and a parallel brief landed. INVARIANTS P2 violation surfaced
by API review at PR #2206 inline comment 2026-05-08T03:42:39Z.

## Authority routing

All T-FixedPoint scope — Slice, Acceptance, STOP-AND-ESCALATE, dispatch
preconditions, two-horizon framing, P0 readiness, P1 Evaluator, P2 SG-0,
P3 worker, TC3 handoff, relationship to DB-8 — is owned by
[`r3-pb-t-fixedpoint-worker.md`](r3-pb-t-fixedpoint-worker.md).

Sibling readiness pins live at:
- [`r3-pb-t-fixedpoint-p1-p2-readiness-2026-05-02.md`](r3-pb-t-fixedpoint-p1-p2-readiness-2026-05-02.md)
- [`r3-pb-t-fixedpoint-p3plus-readiness-2026-05-02.md`](r3-pb-t-fixedpoint-p3plus-readiness-2026-05-02.md)

These are the canonical surfaces. Do not consume this file as authority.

## Pending PB Mgr decision (not authority — proposal)

The original draft of this file proposed a tactical S1-S5 unstaging slice
(SG-0 precondition check → `compiler.dag` parse-probe → two-cycle byte-diff →
§1.8 row 16 promotion → CI `continue-on-error` lift). That slice content is
parked at git history of this file (commit 5e94bc6fd on session/bright-raven-819)
and will NOT live as an authority anywhere unless PB Mgr decides one of:

- **Option A** — fold the S1-S5 dispatch-mechanics into
  `r3-pb-t-fixedpoint-worker.md` §"P3 — T-FixedPoint worker (future dispatch)" as
  an addendum or expanded acceptance section.
- **Option B** — keep the dispatch-mechanics out of canvas; PB Mgr re-derives
  them at gate-clear from the canonical brief's existing acceptance criteria
  (the canonical brief already names §"Acceptance gate", §"Relationship to DB-8",
  §"Dispatch preconditions" — these may be sufficient without a separate
  S1-S5 enumeration).
- **Option C** — discard. Substrate already exists; no enumeration needed.

bright-raven-819 awaits PB Mgr decision on inbox #2136. No additional authoring
proceeds without that decision.

## Why not just delete this file

INVARIANTS P2 + transparency: the closed-as-redundant routing doc preserves a
durable receipt of the missed-grep failure mode (filename-token form
sensitivity) and points future readers at the canonical authority. Deletion
loses the receipt; conversion to a routing doc keeps it.

## Pre-author audit failure-mode receipt (for `feedback_grep_verify_*` discipline)

Failure: `brief-authoring-checklist.md` Q2 grep used `fixed.point|self.host|self.comp`
as the disjunction. This matches "fixed-point", "fixed_point", "fixed.point",
"self-host", "self_host", "self-comp", but NOT "fixedpoint" (single token).
The canonical brief filename `r3-pb-t-fixedpoint-worker.md` uses the `fixedpoint`
form, so the grep returned empty.

Lesson: when running `Q2`-style "does an existing brief cover this" discipline,
collapse separators in the search regex — e.g.
`fixed[-_. ]?point|fixedpoint|self[-_. ]?host|selfhost|self[-_. ]?compile|selfcompile`.
Or `ls docs/briefs/ | grep -iE 'fixed|self'` first, as a low-precision wide
sweep, before narrowing.
