# Buganizer visual contract (captured 2026-09-22)

Authority for the issue-tracker frontend's look and feel. Owner's directive: anchor
on Google's Issue Tracker — no homegrown UX. Provenance per section. The anatomy sections
are MEASURED (2026-10-04): the modeled values are in `extdeps.google.issue_tracker`, and the
captures and probe output they were read from are named below.

## Captures on disk
- `~/Assigned to me - Issue Tracker.html` (259KB — client-rendered DOM, empty state + advanced search open)
- `~/Assigned to me - Issue Tracker_files/` (assets incl. main CSS bundle `m=c` 6.3MB)
- `~/issue-tracker-shots/` (2026-10-04: issues.chromium.org signed out at 1440x1100; screenshots,
  computed-style probes, the instrument, and `element-spec.md`, the element-by-element comparison
  and the owner's rulings). Shared copy of the screenshots:
  https://drive.google.com/drive/folders/14rcDZQtJAHzrAYimoRNmm7rBS1HqV_9b?usp=sharing

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

## Issue-row anatomy (MEASURED 2026-10-04 — captures `06-grid`, `14-grid-default`, `03-deps-tree`)
Results grid columns: select · star · P · Type · Title · Assignee · Status · 7d views · ID ·
Last modified; rows are one dense line and the title links to the detail page. Row, header and
relation-table heights are `issue_tracker_measured_metrics`; the served column roster and its
owner amendments are `issue_results_columns`.

## Issue detail anatomy (MEASURED 2026-10-04 — captures `01-detail-comments`, `11-ancestor-menu`, `15-status-edit`)
Header: component path and ancestor chain as chips, then the title and the action buttons. Tabs:
Comments · Dependencies · Duplicates · Blocking · Resources, each with a count. Main column: a chip
row (status, type, priority), then cards (status update, description, comments). Field rail at
the trailing edge. Type scale, palette (light and dark), metrics, card, status chip and button
anatomy are the `issue_tracker_measured_*` rows; the dark token row of 2026-09-23 is replaced by
`issue_tracker_measured_palette_dark`. Our Work tab maps onto the History half + execution
receipts.

## Mapping notes for the frontend (workflow policy, not extdeps)
- Issue = roadmap node; Component = domain placement (owner/lane); Blocking edges =
  dependency edges; Stars/CC/Upvotes = per-user subscription facts (new modeled state).
- Status = our derived present-tense explanation mapped to the product's standings
  (open/new/assigned/accepted/fixed/verified), never a flattened label.
- "To be verified" (stock view) = our verification-owed candidates.
