# The issue page architecture (owner mandate 2026-09-25)

The per-issue page copies Buganizer's information architecture nearly
literally, retaining gunbc's stronger workflow evidence. The page must answer
at first glance: what is this, who owns it, what blocks it, what changed, what
happens next. It feels alive because people and Fabric visibly participate in
one narrative — not because more diagnostics are on screen.

## Layout law

- Title is the primary object/link; the canonical slug is secondary identity.
- Header carries ONLY planning status + type + priority (`[Done] [Task] [P2]`).
  No competing status strings (`done`, `claimed by…`, `accepted`, standing
  lines all at once is the defect).
- Default tab is **Comments**: description pinned above a chronological
  comments/activity stream with author avatars, comment numbers, timestamps,
  and a real composer. Tabs: `Comments · Dependencies · Work · History`
  (Duplicates later, when duplicate semantics exist).
- Right metadata rail: status, assignee (the editable control), reporter,
  component, parent, blocked-by/blocking, delivery, return-to, modified,
  observation.
- Authored fields are not seven equal rows: Description (brief/motivation/
  outcome), Acceptance criteria (red control, hand-back), Current plan
  (current state, first slice — collapsed once done), Scope (collapsible),
  Work (implementation instructions and evidence). An empty
  `verified · last verified` row never renders.
- Work tab: attempts, incarnations, provider selection, candidate heads,
  verification receipts, raw diagnostics. Comments tab: discussion, assignment
  changes, important blockers, publication, review handoff, status changes.
  Never raw provider/belt noise in Comments.

## One qualified-principal model

```text
QualifiedPrincipal { subject, primary_email, display_name,
                     kind: Human | Workflow, avatar }
```

Email is the user-facing selector, never the durable identity — aliases can
change without rewriting history. The assignee control is an email
autocomplete (search, recents first from durable assignment history, avatar +
display name + email + kind, keyboard navigation, dedupe by stable principal,
`fabric@gunb.ai` pinned near top). NO model names anywhere in the assignment
UI (no glm/kimi/codex); Fabric picks the worker through capability/cost/
capacity/serving authorities. The realization may appear in the Work tab.

Avatar resolution order: internal principal profile → Google Workspace
directory/Contacts photo → cached prior → deterministic initials. Served via
`/avatar/<qualified-principal>` with TTL/ETag. Standing:
`AvatarResolved | AvatarAbsent | AvatarUnread`; absent and unread render
initials and NEVER refuse assignment, comments, or rendering. Fabric gets a
stable service avatar with a workflow badge — it must not masquerade as human.

## Assignment IS the execution entry point

```text
AssignIssue { issue, expected_issue_revision, assigned_by, assignee, return_to }
```

- Assign to human → transfer responsibility, no dispatch.
- Assign to `fabric@gunb.ai` → transfer responsibility + durably request
  execution (`IssueAssignedToWorkflow → WorkRequested → launch admission`).
  No Start Work button anywhere.
- Admission refusal does NOT undo assignment: the issue stays assigned to
  Fabric and renders the located waiting condition ("blocked — no admissible
  serving offer; retry when an offer becomes available"), not "assignment
  failed".
- Re-assigning the same issue revision to Fabric joins the existing request
  rather than minting another attempt.

## Return-to-merge

Record `requester`, `assignee`, `return_to/merge_owner` explicitly (default
`return_to = requester` — never derived from whichever event was first). On
OBSERVED publication: durable publication event → one compact automated
comment with PR/head → status Ready for review → assignee becomes return_to.
Merge remains the human completion action; publication never closes the issue.

## Comments are durable events

`IssueCommentCreated / Edited / Deleted` — each comment carries issue id,
comment id, author principal, body (or body ref), created/edited times,
visibility. Edits/deletes append events, never invisible mutation. Stable deep
links `/issue/<id>#comment-7`. Composer with markdown/code/@mentions; retries
never duplicate. The stream interleaves only meaningful lifecycle events
(assignment, work accepted, attempt started, blocking decision needed,
candidate submitted, PR published, review requested, reassigned for merge,
closed/reopened).

## Dependencies stay dimensional

Dependencies tab preserves separate sections: parent/children, blocked
by/blocking, related, supersedes/superseded-by — never one flattened list.
Header may carry a compact summary (`Blocked by 2 · Blocking 1 · 4 children`)
that opens the tab.

## Acceptance controls (12)

1. Title primary link, slug secondary. 2. Assignee field opens avatar-backed
email autocomplete with fabric + humans + recents. 3. Human assignment never
dispatches. 4. Fabric assignment = exactly one work request, no Start Work.
5. Serving/capacity refusal leaves Fabric assigned with the located blocker.
6. Reload preserves assignments and comments. 7. Comment retries never
duplicate. 8. Missing avatars render initials, never block operations.
9. Publication posts one handoff comment and reassigns to the stored merge
owner. 10. Dependencies separate from parent/child. 11. No raw provider/belt
noise in Comments. 12. Work tab keeps exact attempt/candidate/verification/
publication/incarnation evidence.
