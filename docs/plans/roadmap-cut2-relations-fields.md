# Roadmap Cut 2: relation and field projections

This cut addresses PR #12387 review 5328293087. The command/event append boundary,
OAuth routes, compiler resolver, and live deployment remain separate lanes.

## Owning facts and consumers

| Fact | Authority | Derived consumers |
| --- | --- | --- |
| Must complete before start | `RoadmapEdge` / `declared_roadmap_edges` | Scheduling, unmet blockers, inverse blocking links |
| Delivery ownership | `RoadmapMembership` / `declared_roadmap_memberships` | Project grouping and counts, issue parent/children, alignment, supervisor target, capture/integration envelope |
| Ticket field meaning | `TicketField` and `ticket_field_value` over `TicketFields` | Disclosure, typed section selection, worker brief fields |
| Fleet enrollment | Runner hosts and fabric host subjects | Tracker host components |
| Component placement | `fleet_component_placements` over the fabric group observation | Serving-arm component paths |
| Tracker component identity | `fleet_component_identities` | Stable allocated host/serving-component IDs; never array positions or derived slugs |

Membership declares the operator's ten-task shell migration batch. Other
prerequisites do not acquire an owner merely because another task needs them.
Changing membership moves the issue and worker projections together. Ambiguous
membership refuses worker ownership; the UI names the ambiguity rather than
choosing the first edge. Alignment also refuses missing ticket rows and cycles.
Dependency links do not contribute to a project's membership denominator.

Authority-owned local issue IDs remain unchanged. This cut introduces no new
qualified identifier or alternate component-path encoding: tracker components
retain the upstream `Component.path` representation and its existing browse-tree
projection. The broader implied-namespace-level work in
`one-namespace-hierarchy.md` remains distinct from declaring delivery ownership;
this cut does not claim that filesystem namespaces are issue ownership.

`DisclosureField` retains a typed field identity alongside its display label.
Section selection reads that identity, so a label change cannot erase acceptance
criteria. The issue Description reads the brief, while legacy rows retain their
available authored body. Acceptance prose remains separate from execution and
acceptance receipts; the existing witness/artifact identity-join gap is unchanged.

`IssueRevisionCandidate` and its editor are still a target in
`planning-work-contract.md`, not an implemented consumer in this branch. That
consumer should reuse `TicketField` and `ticket_field_value`; this cut does not
create a second planning or issue-write mechanism.

Fleet projection refuses absent/ambiguous ID allocations, duplicate identities,
unenrolled placement subjects, and missing serving placement. Google extdeps
retain only the upstream interface and generic path-tree behavior.

## Boundary controls

`roadmap_cut2_witness_test` exercises independent membership/blocking inverses,
shared issue/worker ancestry, cyclic membership refusal, label-independent
acceptance fields, stable component IDs under placement changes, and missing
identity/placement refusal. Existing presentation controls cover a shared
prerequisite with no ownership, explicit ownership despite another project's
blocking edge, ambiguous ownership without first-choice placement, and distinct
prerequisite links and membership counts. Existing page controls check the brief
in the Description body and the retired header framing.

The allocation background fix and header cleanup are separate commits for
cherry-picking. Executed CSS derivation remains `d43dd7a918f31c5f`: allocation CSS
is appended separately from `roadmap_css`, so the shared CSS digest does not move.
