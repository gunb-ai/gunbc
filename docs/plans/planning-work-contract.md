# The planning work contract (owner directive 2026-09-25)

Supersedes the planning-ticket-type and Fabric·planner-role sketches from
conversation (never landed). Do NOT add Planning to the issue-type vocabulary;
do NOT introduce a second workflow role or principal.

## The model

**Planning is one first-class work contract**: a bounded effect envelope whose
result is an `IssueRevisionCandidate`, an investigation report, or a comment
response. It is the same generic work request assignment mints, with a
different envelope — not a different kind of issue, agent, or principal.

The durable invariants stay put:

- the issue remains the durable problem/outcome;
- component remains domain placement;
- assignment remains accountability;
- parent/child and blockers remain separate graph dimensions.

## Issue changes are versioned candidates with CAS application

Fabric may: comment, propose field changes, propose relation changes, propose
draft children. Fabric may NOT: directly rewrite the target issue, or edit
code, under a planning contract.

```text
IssueRevisionCandidate {
    issue
    base_issue_revision      -- the version it was derived against
    proposed_field_changes
    proposed_relation_changes
    proposed_draft_children
    author / work-request identity
}

apply(candidate, expected_revision = candidate.base_issue_revision)
    → Applied { new_revision }
    | RevisionConflict { current_revision }   -- CAS refusal
    | CandidateRefused { cause }
```

A proposal never mutates the issue until accepted through the CAS route. The
acceptor is the issue's accountable human (or a later explicit automation
ruling) — the candidate mechanism itself grants no write path.

## Comments are durable narrative events, never implicit commands

Ordinary prose has no workflow effect. The explicit operation is:

```text
AskFabric { issue, referencing_comment, asker }
    → mints the same generic work request assignment mints
       (bounded planning envelope)
    → the answer lands as a comment response in the thread
```

No mention-parsing, no prose-triggered turns: if it isn't an `AskFabric`
event, nothing runs.

## The ban

No operational meaning may be encoded in: planning-flavored components,
Planning issue types, labels, title prefixes, or role-encoded assignee aliases
(e.g. "fabric-planner@…"). Structure carries meaning; names don't.

Future recurring patterns are promoted only when they carry a DISTINCT effect
envelope, deliverable, admission rule, verification, or application route —
all five named, or it stays an ordinary work request.

## Queue position

Durable comments are already in flight (agent-52's vertical). This contract is
the next vertical behind it: the versioned IssueRevisionCandidate + CAS
application route, the AskFabric operation minting the generic work request,
and the bounded planning envelope (comment/report/candidate results only) as
the envelope vocabulary sibling of the execution envelope.
