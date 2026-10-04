# Buganizer visual contract (captured 2026-09-22)

Authority for the issue-tracker frontend's look and feel. Owner's directive: anchor
on Google's Issue Tracker — no homegrown UX. Provenance per section; anything marked
CONVENTION is public product convention pending a saved results page.

## Captures on disk
- `~/Assigned to me - Issue Tracker.html` (259KB — client-rendered DOM, empty state + advanced search open)
- `~/Assigned to me - Issue Tracker_files/` (assets incl. main CSS bundle `m=c` 6.3MB)

## Status vocabulary (VERIFIED — the product's own query builder)
open · new · assigned · accepted · closed · fixed · verified · duplicate · inactive ·
infeasible · intended behavior · not reproducible · obsolete

## Sidebar roster (VERIFIED — owner's session)
Assigned to me · Starred by me · Upvoted by me · CC'd to me · Collaborating ·
Reported by me · To be verified · Bookmark groups · Saved searches · Hotlists
(Create hotlist) · Create bookmark group · Browse components

## App shell (VERIFIED)
- Header: product wordmark "Issue Tracker"; search input with "Search help" + "Save";
  "Create Issue" primary button; right side: theme switcher, help, Google Account block.
- Content region: search builder (Match all/any, + OR rows), "Issue search results" heading.
- Empty state: "No issues match your search." + "Learn more about searching" link.

## Design tokens (VERIFIED — extracted from the CSS bundle)
- Text: `#202124` primary · `#5f6368` secondary · `#3c4043` tertiary
- Borders/dividers: `#dadce0` · `#bdc1c6` · `#e8eaed`
- Action blue: `#1a73e8` · hover `#1967d2` · pressed `#185abc`
- Error/red: `#c5221f` · `#ea4335`
- Backgrounds: `#f1f3f4` · `#f8f9fa` · highlight blue `#e8f0fe` · `#d2e3fc`
- Type: Roboto, Arial, sans-serif · Google Sans, Roboto, Arial, sans-serif (headings)

## Issue-row anatomy (CONVENTION — pending a saved results page)
Results table columns: ID (linked) · Component · Title · Status · Priority (P0–P4) ·
Severity (S0–S4) · Assignee · Stars (+1 count) · Modified. Row is one line, dense,
single-click to detail; star toggles inline.

## Issue detail anatomy (CONVENTION)
Header: issue id + title + action overflow. Field panel: Status, Priority, Severity,
Assignee, CC, Reporter, Components, Blocking/Blocked-by, Stars/Votes, Created/Modified.
Tabs: Comments (default, the discussion) · History (field-change log) · plus
duplicate/mark actions. Our Work tab maps onto the History half + execution receipts.

## Mapping notes for the frontend (workflow policy, not extdeps)
- Issue = roadmap node; Component = domain placement (owner/lane); Blocking edges =
  dependency edges; Stars/CC/Upvotes = per-user subscription facts (new modeled state).
- Status = our derived present-tense explanation mapped to the product's standings
  (open/new/assigned/accepted/fixed/verified), never a flattened label.
- "To be verified" (stock view) = our verification-owed candidates.
