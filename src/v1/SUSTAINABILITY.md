# Sustainability Ledger (Archive)

As of 2026-03-17, `ROADMAP.md` is the single live planning document for active
compiler work.

This file no longer carries open-work tracking. The active contents of the old
sustainability ledger were consolidated into:

- `ROADMAP.md`, **Track S** for current v2 stabilization and residual
  performance work
- `ROADMAP.md`, **Track B** for Node convergence
- `ROADMAP.md`, **Track C** for language-emission/extdep work
- `ROADMAP.md`, **Track D** for the new runtime complexity analysis track
- `ROADMAP.md`, **Consolidated Backlog** for the remaining lower-priority
  structural debt that still matters

What this file now means:

- it is an archive marker, not a second roadmap
- if an item is still active, it should appear in `ROADMAP.md`
- historical ledger detail is preserved in git history prior to this
  consolidation

The governing sustainability metric remains the same: when the language grows by
one type, one expression form, or one transport, the sustainable compiler is
the one where the number of required edits is 1.
