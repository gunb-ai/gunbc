# Qualified subject identity (owner direction 2026-09-25)

Sits under `one-namespace-hierarchy.md`: that ruling says there is one
hierarchy; this one says every object in every context names itself with one
construction. Together: one algebra, one graph, three+ projections.

## The algebra

```text
QualifiedSubjectName
  = authority
  + namespace path
  + local identity

SubjectIncarnation
  = QualifiedSubjectName
  + revision / generation / invocation
```

This is NOT one flat namespace for every object. Every object uses the same
construction while preserving its owning authority:

```text
buganizer:gunbc/issue/<issue-id>
github:gunb-ai/gunbc/pr/12210
github:gunb-ai/gunbc/pr/12210@52c56a8
dag:gunb-ai/gunbc/test/<module>::<function>
dag:gunb-ai/gunbc/test/<module>::<function>@52c56a8
run:srv1/<invocation-id>
compiler:gunbc-v2/diagnostic/match_arm_navigation_refused
```

At the .dag level: reuse the existing qualified-name/namespace carrier. Do NOT
mint stringly per-kind identities (`BuganizerNodeName`, `PrNumber`,
`TestLabel`, `RunName`). Kind-specific types may wrap one qualified identity;
they must not reinvent its namespace semantics.

The `SubjectIncarnation` half is load-bearing: it prevents the UI from
attaching a result from main, an older PR head, or a stale run to the stable
subject as though the subject were timeless. Every displayed runtime claim is
bound to an exact incarnation.

## The relation vocabulary (two issue-to-issue dimensions, never collapsed)

| Relation                      | Meaning                        | Affects scheduling? |
| ----------------------------- | ------------------------------ | ------------------: |
| `UpstreamOf` / `DownstreamOf` | Issue decomposition or nesting |       Not inherently |
| `Blocks` / `BlockedBy`        | Must complete first            |                 Yes |
| `Implements`                  | Issue → PR                     |  No issue dependency |
| `ValidatedBy`                 | Issue → test/claim             |                  No |
| `ObservedBy`                  | Test/claim → exact run         |                  No |
| `ReportedDiagnostic`          | Exact run → diagnostic         |                  No |
| `BasedOn`                     | PR head → another PR head      |      Not automatically |

Laws:

- A stacked PR is `BasedOn`, not automatically `Blocks`. It becomes `Blocks`
  only when the associated issue genuinely cannot complete independently.
- A test is not automatically a child issue; a run is not a blocker node; a
  diagnostic is not an issue dependency; a merge-queue state is not an issue
  status; a numbered landing-order list is not a dependency graph.
- Never use PR numbers as issue node IDs. Each pairing is an explicit
  `BuganizerIssue --Implements--> GitHubPullRequest` link.
- Common causes collapse: N affected tests with one shared refusal render as
  ONE blocker plus an N-row impact population, never N independent blockers.

## Execution-milestone state machine (acceptance tracking)

```text
ContextRefused → ContextResolved → BodyReached → BodyExecuted
  → AssertionVerdictProduced → verdict
```

A population that has not reached its bodies is `BLOCKED`, not `FAIL`:

```text
Passed / Failed in body / Blocked before body / Not yet adjudicated
```

Milestones (e.g. `BodyReached`) are issue-local acceptance milestones, not
synthetic dependency nodes and not new issues. Do not pre-create "unknown
blocker" issues for failures not yet observed.

## Presentation rules (enforced on the tracker surfaces)

1. Never display a result without its incarnation (head sha / invocation id /
   exact run).
2. Distinct states: `PASS / FAIL / BLOCKED / NOT RUN / RUNNING / UNREADABLE` —
   no red "failed" styling for pre-body refusal.
3. Render a DAG, not an ordered prose list (independent branches proceed
   independently).
4. `Upstream / Downstream` and `Blocked by / Blocks` render in separate
   sections even when one issue pair carries both.
5. PRs, tests, runs, diagnostics attach to issue nodes through typed
   relationships as evidence (Implementation / Verification sections); they
   never masquerade as issue nodes.
6. Collapse common causes (above).
7. Never infer issue completion from PR state — "approved" / "merge queue" /
   "checks running" are properties of the implementing PR incarnation; the
   issue closes only by its own completion rule. Verification with unresolved
   attribution reads `pending attribution`, never a blamed subject.

## The invariant

> Every displayed object has one authority-owned qualified subject name; every
> runtime claim is bound to an exact incarnation; upstream/downstream expresses
> issue nesting; blockers express must-complete-first ordering; PRs, tests,
> runs, and diagnostics attach as evidence rather than silently becoming issue
> nodes.

## Where this binds current lanes

- Tracker frontend (agent-48 + follow-ups): assignee/issue semantics, the
  detail page's sections (Upstream/Downstream vs Blocked by/Blocks,
  Implementation, Verification), state vocabulary, DAG rendering.
- Issue-tracker extdeps model: the relation vocabulary above becomes the
  typed relation set; the roster gains incarnation fields on any rendered
  result.
- Alignment consolidation: alignment walks the namespace path of the
  QualifiedSubjectName; ambiguity/refusal uses the same carrier.
- SCM/belt surfaces: captures, receipts, and verdicts already carry exact
  heads — the display rule (1) makes that binding uniform on every surface.
