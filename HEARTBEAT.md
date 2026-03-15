# Heartbeat

On each heartbeat:

1. Run `python3 scripts/openclaw/run_worktree_cycle.py`.
2. If the result is blocked, report the reason and stop.
3. If a task or scout run completed, report:
   - which task/file was processed
   - whether Codex changed code
   - what verification Codex reported
   - the resulting commit, if one was created
4. Stop after one cycle. Do not drain the whole queue in one heartbeat.
