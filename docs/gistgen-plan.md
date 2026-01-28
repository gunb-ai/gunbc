# Gistgen Plan (v0)

**Status**: Draft — January 2026
**Purpose**: Minimal, implementable plan for `gistgen` grounded in gunbc.
Small on purpose.

---

## 1. Goals / Non-Goals

**Goals**
- Emit a shareable Gist URL quickly.
- Auth is Upsert-shaped capability acquisition.
- `gistgen` is a non-idempotent Emit (never Upsert in v0).
- Treat GitHub/Git as opaque unless semantics are required.

**Non-goals (v0)**
- Cross-run dedupe (hash/commit based).
- Full Git object/refs semantics.

---

## 2. Concepts (only what v0 uses)

- **Context**: rooted observation source (env, args, time).
- **Repo**: repo identity (path, optional ref later).
- **SelectionSpec**: include/exclude rules.
- **RepoSnapshot**: selected files + contents (Repo + Snapshot).
- **Secret\<GithubToken\>**: capability needed for API calls.
- **GistUrl**: handle returned by GitHub.

---

## 3. Pattern Decisions (forced)

- `auth`: **Instantiated** (Upsert — capability resource).
- `gistgen`: **NotApplicable** — reason: "non-idempotent snapshot emission."

---

## 4. Program DAG (top-level)

```
Context ──env──> Auth (Upsert) ──token──────────┐
Context ──args──> ParseArgs ──repo──────────────┼──> Gistgen (Emit) ──> GistUrl
                            ──selection_spec────┘
```

Node contracts (minimal):
- `context`: `() -> (Env, Args, Time)`
- `parse_args`: `Args -> (Repo, SelectionSpec)`
- `auth`: `Env -> Secret<GithubToken>` — Upsert-shaped
- `gistgen`: `(Repo, SelectionSpec, Secret<GithubToken>) -> GistUrl` — NonIdempotent write

---

## 5. `gistgen` Emit Sub-DAG

```
enumerate_files → filter_files → read_files → compose_snapshot → upload_gist
```

- `enumerate_files`: Observe
- `filter_files`: Pure
- `read_files`: Observe
- `compose_snapshot`: Pure (Repo + Snapshot → RepoSnapshot)
- `upload_gist`: WritesWorld + NonIdempotent

---

## 6. Validation + Runtime Rules (exercised in v0)

- Every node must declare a `PatternDecision` (no silent defaults).
- `WritesWorld` nodes require explicit idempotency stance and policy approval.
- `Secret<T>` values are never displayed in logs, errors, or debug output.
- IR checks: type agreement, port saturation, acyclicity.
- Auth token exists → downstream proceeds; otherwise blocked.

---

## 7. Implementation Order (walking skeleton)

1. Minimal IR: Node, Dag, Port, Edge.
2. `PatternDecision` plumbing + validation.
3. Auth Upsert node (`Env -> Secret<GithubToken>`).
4. Gistgen emit pipeline ending in `upload_gist`.
5. CLI builds the DAG and executes it.

---

## 8. Future Refinements (explicitly out of v0)

- Open repo semantics (RepoRef, TreeHash, etc.).
- Dedupe by upgrading gistgen to Upsert (wrap Create).
- Factor a reusable Emit pattern if it emerges.
