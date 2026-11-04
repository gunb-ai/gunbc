# Roadmap-as-spawner

**Goal:** make `.dag` roadmap structure drive ctrl work. **Method:** gunbc derives graph readiness from main HEAD and observed acceptance; ctrl deduplicates against live sessions and invokes its existing work-item poller. **Validation:** satisfied dependencies produce ready nodes, unsatisfied ones do not; paused/missing/malformed control produces zero spawns; restarts do not duplicate sessions.

`roadmap_authority`/`roadmap_model` own nodes, dependencies and acceptance. Structure changes by PR/commit to main; reread each tick. ctrl/GitHub own runtime facts, read as evidence by gunbc without copying standing into the graph. gunbc computes not-done ∧ all-deps-done; ctrl alone checks no-active-session-for-node. ctrl never authors graph structure or recomputes readiness.

## Wire contract

`roadmap-spawn-request/v2` and `roadmap-dispatch/v2` replace v1. The request carries `anchor_commit` and graph-ready rows: stable `ProcessNodeId` as `node_id`, `title`, `repo`, nullable composite `parent_node_id`, `plan_doc`, and `acceptance` (`prs_merged` with PRs, or `manual` for MVP).

`intricacy` (low/medium/high) and `volume` (small/medium/large) are **paired optional fields**, present only when `gunbc.roadmap.roadmap_sizing` derives them. The recorded stub refuses every subject; missing is never minimum/zero/default sizing. ctrl must explicitly accept v2, enforce pairing and choose its own no-size policy. A schema-checking v1 reader can refuse v2; this producer cannot claim how an unobserved consumer behaves. Derived keys may return under v2 without another bump. `rung_drop.roadmap_sizing_authored_to_derived` retires only with task-reading derivation, not a function returning a constant cell.

`roadmap-spawn-control/v1` carries `paused`. Default true; missing file, parse error or fresh deployment stays paused. The operator can pause/resume in one ctrl action without a gunbc commit/regen. Poll every tick; when paused, read/report the held ready count and spawn nothing. The bridge defines the command/dashboard toggle.

## Delivery

1. gunbc: `next_spawnable(authority, merged_prs)` pure fold, existing `authored_merged_prs()` evidence, v2 CLI projection and positive/negative dependency controls. The original authored-size/v1 proposal is superseded by the contract above.
2. ctrl: poll control, consume ready rows, dedup sessions, then `POST /api/internal-work-items` under a designated manager. Reuse the existing auto-spawn poller, capacity gate and respawn circuit breaker; report held/spawned counts. Acceptance remains PrsMerged/Manual; no automated acceptance claim.
3. Later, ctrl’s nodes become a projection/cache; `.dag` models SpawnSession/CloseSession through the host-effect `apply` seam. Existing `code_change_workflow` supplies lifecycle, `process_algebra` supplies Leaf/Composite/Bucket and Declare/Decompose/Close. Host handlers realize these effects; delete the displaced JS. The original JS bridge is a stage, not the terminal planning authority.
