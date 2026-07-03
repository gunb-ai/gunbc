# Dispatch + maintain Claude Code sessions from the srv1 roadmap (simple MVP)

Operator brief (2026-07-03). A button per READY roadmap item that actually spawns a `claude`
session on srv1 to work it, plus a minimal panel to see/stop live sessions. This supersedes the
"ctrl bridge lane" of [roadmap-spawner.md](roadmap-spawner.md) Stage 1: the actuator is the
srv1 gunbhub/roadmap server itself — no ctrl dashboard in the loop. The structure/runtime split
and the spawn-request contract from that doc still hold (graph readiness comes from
`roadmap_spawner.dag`; the actuator adds only runtime dedup + execution).

## What exists (verified against the live tree, 2026-07-03)

- **Frontier + brief**: `dag/gunbc/roadmap_spawner.dag` — `spawn_frontier` (ready/upcoming/done),
  per-node metadata JSON, `dispatch_brief_template`. Served today at
  `/target/roadmap-dispatch.json`; the roadmap page renders Ready/Upcoming/Done buckets.
- **The srv1 server**: emitted static Node server — `node_http_server_emit.dag`
  (`emit_node_http_static_server_program`) folds a `ServedStaticRouteTable` into a
  `http.createServer` program; `live_deploy/` writes it to `/opt/gunbc/server.js` under systemd
  `gunbc-roadmap.service` (port 8080, `tailscale serve`), deployed by CI job
  `deploy_dashboard_srv1` on push to main.
- **The gap**: the emit lane is static-only. `NodeHttpServerRouteBinding` /
  `NodeHttpRequestListener.handler_name` (`gunbhub_serve.dag`, `extdeps/http/server.dag`) are
  modeled but inert — their dissolution trigger names exactly this lane ("P3 dynamic-handler
  serving lane … becomes live when a realization handler binds a handler_name dispatch").
  No POST route or request-body handler exists anywhere in the corpus.
- **Double-spawn gate**: `dag/gunbc/session_lease.dag` — observation→verdict→plan
  (`Start` / `DrainMatchingStaleThenStart` / `RefuseForeignOwner`) over `std.upsert_decision`.
  Reuse the same shape keyed on tmux session name per node_id (a session lease, not a port
  lease — same classifier discipline: observation carries facts, classifier decides).
- **Lifecycle binding** (`dag/ctrl/code_change_workflow.dag`): phase 2, explicitly out of MVP.

## The crux: how `claude` is invoked (studied from ctrl `scripts/session-dashboard`)

ctrl's claude provider (`providers/claude.mjs`) is the actuator template. The load-bearing facts:

- Spawn shape (inside a detached tmux session, cwd = the worktree):

  ```
  tmux new-session -d -s gunbc-dispatch-<node_id> -c <worktree> \
    "CLAUDE_CODE_NO_FLICKER=1 claude \
       --dangerously-skip-permissions \
       --settings '{\"skipDangerousModePermissionPrompt\":true}' \
       --effort <tier> [--model <model>] \
       --session-id <uuid> -n <node_id> \
       --append-system-prompt '<filled dispatch brief>'"
  ```

- `--dangerously-skip-permissions` is the strong form (actually bypasses);
  `--allow-dangerously-skip-permissions` only makes bypass toggleable — wrong flag for
  unattended sessions.
- `skipDangerousModePermissionPrompt: true` via `--settings` is REQUIRED on a fresh host:
  without it claude renders a full-screen one-time bypass disclaimer, blocks on a keypress,
  exits, and the spawn loops (ctrl incident keen-seal-347).
- `--session-id <uuid>` pins a NEW transcript (errors if it exists); `--resume <uuid>` is the
  re-spawn form. MVP only spawns fresh → always `--session-id` with a fresh uuid.
- The brief is seeded with `--append-system-prompt` (agent sees it from turn 1, no synthetic
  user message) plus an initial positional prompt to start it working; effort/model map from
  the node's `intricacy`/`volume` (MVP: a fixed small map, e.g. high→high effort, else medium;
  model left to account default).
- Worktree: `git worktree add <dispatch-root>/<node_id> -b dispatch/<node_id> origin/main`
  in the node's repo clone on srv1 (MVP: one host, `repo` must be `gunbc`; refuse others).

## Three pieces (MVP scope)

1. **Spawn** — `POST /dispatch/<node_id>`: resolve node from the READY frontier (404/refuse if
   not ready — graph readiness stays gunbc's, per roadmap-spawner.md); lease-gate on tmux
   session name (`Refuse` if a live session owns the node); `git worktree add`; fill
   `dispatch_brief_template` from node metadata; `tmux new-session -d` running the claude
   command above. Response: JSON receipt (node_id, tmux session name, worktree, uuid) or a
   typed refusal — never a silent 200.
2. **Maintain** — `GET /sessions`: `tmux ls` filtered to the `gunbc-dispatch-` prefix, joined
   with lease verdicts. `POST /sessions/<id>/stop`: `tmux kill-session` (worktree left in
   place — teardown of worktrees is not MVP).
3. **Frontend** — a Dispatch button per READY item (fetch POST + show receipt/refusal); a
   "Live sessions" panel (fetch GET /sessions) with a Stop button per row.

## How it lands in the substrate (model-before-implement)

The server is emitted from `.dag`; the actuator must be too — no hand-written server.js patch.

- **M1 — dynamic-route emit lane** (the substrate piece; everything else hangs off it).
  A `ServedDynamicRoute` row: method + path template (with params) + a modeled host command
  (argv rows, path params spliced as arguments never shell-interpolated) + response shape.
  Extend `emit_node_http_static_server_program`'s handler fold: dynamic rows dispatch to a
  `child_process.execFile` call and return `{status, json}`. This inhabits the inert
  `NodeHttpServerRouteBinding` lane (dissolve its trigger). Per DESIGN §3(b) the exec transport
  is one realization handler bound to the route shape; the route/interface shape stays in the
  model. Witness: emitted server source contains the dispatch arm; an emitted toy server
  executes a harmless command on POST and 404s on GET (RED: drop the row → route gone).
- **M2 — dispatch actuator model** (`dag/gunbc/roadmap_dispatch_actuator.dag`): brief fill
  (template × node metadata → filled brief string), spawn command construction (worktree +
  tmux + claude argv as data), tmux session naming (`gunbc-dispatch-<node_id>`), lease
  classifier reuse, intricacy/volume → effort map, `tmux ls` output parse for GET /sessions,
  stop command. All pure folds with claim witnesses (RED: a not-ready node classifies Refuse;
  a live lease classifies Refuse; a hostile node_id — shell metacharacters — is rejected by
  construction since argv is never a shell string).
- **M3 — route wiring + frontend**: POST /dispatch/:node_id, GET /sessions,
  POST /sessions/:id/stop rows in the roadmap-dashboard route table; buttons/panel markup
  (small emitted JS for fetch — the `js_site_emit` lane exists).
- **M4 — live receipt on srv1** (T3→T4 per the roadmap acceptance rule): deploy via the
  existing `live_deploy` path; press Dispatch on one real READY item; a real claude session
  comes up in a worktree working the item; it shows in the panel; Stop tears it down.
  Independent read-back: `tmux ls` on srv1 + the transcript file existing under
  `~/.claude/projects/...`, not our own write echoed back.

## Fail-closed posture (§5)

- Dispatch refuses (typed) unless: node is in `ready`, repo is the srv1-cloned repo, no live
  lease, worktree add succeeded. Any exec failure → 5xx with stderr in the receipt, never 200.
- node_id is validated against the frontier (the authority), not sanitized — an unknown or
  unready id is a refusal, so path-param injection has no surface; argv construction (no shell
  string concat) closes the rest.
- The pause posture of roadmap-spawner.md is preserved by construction: nothing spawns without
  an operator click. (The old "spawn bridge stays fail-closed paused" referred to the
  *automatic* ctrl loop; a human-initiated button is the un-pause action itself.)

## Non-goals (explicit)

One host (srv1) · tmux+claude+worktree is the whole spawn (no containers/systemd-per-session/
scheduling/auto-restart) · no lifecycle-state binding (`code_change_workflow.dag`) yet · no
worktree GC on stop · no resume/restore of dead sessions · no acceptance automation.

## Roadmap placement

"session dashboard + roadmap-as-spawner" sits in ROADMAP §4 shelved; this brief un-shelves the
actuator slice. This doc lands with the ROADMAP.md edit moving a sized node into the active
sections (operator-directed 2026-07-03), per the reset rule that un-shelving is a PR with a
displaced-cost justification. Displaced cost: every dispatch today is the operator hand-writing
a brief and hand-spawning a session; the READY frontier already computes what to work next —
the missing piece is only the actuator.
