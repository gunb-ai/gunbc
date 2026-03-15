# gunbc OpenClaw Workspace

This repo root is the control plane for the OpenClaw -> Codex maintenance loop.

## Canonical files

- `WORKBOARD.md` is the repo-root queue, findings log, and tree snapshot.
- `INVARIANTS.md` is the grading rubric for scouting and fixes.
- `src/v2/WORKBOARD.md` remains the v2 compiler planning board.

## Operating rules

- Use `python3 scripts/openclaw/run_worktree_cycle.py` for autonomous runs.
- Use `python3 scripts/openclaw/sync_workboard.py` when you only need to refresh
  the root workboard.
- The root workboard is the task source. Keep it current on the branch you want
  OpenClaw to follow.
- The runner executes code changes in an isolated git worktree, not in this
  checkout.
- Do not hand-edit the managed sections in `WORKBOARD.md`; the scripts own them.
- When no manual task is open, scout exactly one unchecked file from the managed
  scout queue.
- Keep fixes narrow: target one task or one file plus directly necessary
  tests/docs.
