# dashboard-shadow-plan — §8 READ-ONLY external-upstream shadows

**Lane:** §8 (ROADMAP "Session dashboard on `.dag`"), managed by bold-ant-53.
**Discipline:** DESIGN.md §3 shape/transport/policy three-way de-fuse + selection rule from neat-boar/#1797.

## What this lane does (and does not do)

The session dashboard (in the private `ctrl/` repo, `scripts/session-dashboard/`) calls several real external services. This lane models those external APIs as read-only `.dag` shapes in `dsl/extdeps/`, following the same pattern as existing `extdeps/github/`, `extdeps/docker/`, etc.

**Does NOT model:**
- The dashboard's own store (session DB, work-items, `node_overrides.json`, placement reservations) — self-API, layer inversion, DEFERRED per neat-boar/#1797 selection rule.
- The dashboard's own API surface (`dashboard-ops`, `dashboard-message`).
- Any write effects or control paths in the dashboard.

## Selection rule (de-fuse per DESIGN §3)

For each operation the dashboard calls:
- **(a) interface SHAPE** (what the external API returns) → `dsl/extdeps/<service>/` — this lane.
- **(b) TRANSPORT** (REST endpoint, GraphQL query, `gh` CLI argv) → a Realization handler, one of N, NOT a shape fact. Transport goes in the `service { operation { transport ... } }` block or a handler in `dsl/extdeps/<service>/`.
- **(c) POLICY** (which base ref, poll cadence, `origin/main...HEAD` literal, token source) → a workflow fact in the dashboard's own layer. The tell of leaked policy: an argv carrying a literal it should receive as a parameter.

## Slices (ranked by external-boundary clarity)

### A. GitHub review/merge-state reads (this PR — bold-ant-53)

The dashboard reads GitHub to determine PR merge readiness (`dashboard-ops reviews`). Missing shapes:

- `dsl/extdeps/github/checks.dag` — GitHub Checks REST API (`/commits/{ref}/check-runs`).
  Types: `CheckStatus`, `CheckConclusion`, `CheckOutput`, `CheckRun`, `CheckRunList`.
  Service: `github.Checks.ListForRef` (readonly).
- `dsl/extdeps/github/merge_state.dag` — PR mergeable state.
  Types: `MergeableState` (REST `PullRequest.mergeable` field) + `MergeStateStatus` (GraphQL-only enum; determines CLEAN/BLOCKED/DIRTY/etc.).

**What already exists (reuse, do NOT re-coin):** `extdeps.github.pulls` (`PullRequest`, `PullReview`, `ReviewState`), `extdeps.github.github` (`GitHubUser`, `default_api_base`), `extdeps.github.errors` (`GitHubErrorShape`).

**Dissolution trigger:** `merge_state_graphql_residual` scaffold dissolves when a GraphQL transport module lands in `dsl/extdeps/github/` (the `MergeStateStatus` shape is complete now; only its GraphQL transport handler is deferred).

### B. Docker/container lifecycle + stats (child worker — TBD)

The dashboard reads `docker ps` and `docker stats` for placement/spawn decisions. Shapes needed:
- `dsl/extdeps/docker/container_stats.dag` — `docker stats` output: `CpuStats`, `MemoryStats`, `ContainerStats`.
- `dsl/extdeps/docker/ps.dag` extension — `ContainerInfo` with `Status`, `State` (running/exited/paused).

Grounds the anemic fixed-cap fix: the per-session working-set comes from measured container stats, not a hardcoded 4g.

### C. Host metrics (child worker — TBD)

The dashboard reads host metrics for memory-aware placement:
- `/proc/meminfo` → `dsl/extdeps/os/proc_meminfo.dag` or extension of existing `os/` modules.
- `systemctl is-active` → extend `dsl/extdeps/tools/systemd.dag` if it exists, else new file.
- `free -b` → `dsl/extdeps/os/free.dag` (host free memory command).

Feeds `memory_aware_spawn_width` (§1 scheduling).

## Fan-out after slice-A validation

Dispatch one child worker per slice (B and C). Each child:
- DFS existing `dsl/extdeps/` before naming any new type.
- Models only the SHAPE (what the external API actually returns) — no policy, no transport bundling.
- Tags any scaffold with a named dissolution trigger.
- Writes a `_test.dag` witness proving the shape accepts valid data and rejects a discriminating bad input.

## Done condition

Each slice: new `.dag` shape in `dsl/extdeps/`, at least one `_test.dag` witness with pass+fail tooth, `cargo test --workspace` green.
