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

## Issue-row anatomy (VERIFIED — saved results page 2026-09-24)
Source: `/home/briansrls/chromium-565764076/test - Chromium.html` (50 live rows).
Results table column headers, in order: **P (priority) · TYPE · TITLE · ASSIGNEE ·
STATUS · D VIEWS · ID · LAST MODIFIED**. A column picker exists
(`bv2-column-picker-container`) — the column set is user-adjustable. Rows are dense
single lines; the ID cell links to the detail page; the star cell toggles inline.
Sample verified titles include the ordinary chromium row shapes.

## Create-issue form (VERIFIED — saved page 2026-09-24)
Source: `/home/briansrls/chromium-565764076/New issue - Issue Tracker.html`.
Fields: **Title** and **Description** (Markdown-supported, with a "Use Markdown for
this comment" hint). Actions: **Create** · **Create & Start Another** · **Discard**.
(The captured form shows the minimal state; component/assignee/priority are the same
metadata fields from the detail page's editable sidebar, assigned after creation.)

## Issue detail anatomy (VERIFIED — saved page capture 2026-09-24)
Source: `/home/briansrls/chromium-565764076/` (real issue page: header + tabs + metadata + comments).

- **Header**: "Issue <id>" title row; component breadcrumb (`IO (Do Not Use, Use Subcomponents) > Keyboard`); the issue id repeated; vote/star counts (+1 / 0); "Mark as Duplicate" action.
- **Tab row**: `Comments` (default) · `Dependencies` · `Duplicates (n)` · `Blocking` · `Resources` — the issue's relations live in their own tabs.
- **Main column**: Description (the body) then numbered comments (`#1 …`, each with author + timestamp + body).
- **Right sidebar**: metadata field rows (class `bv2-issue-metadata-field` / `bv2-issue-metadata-list-field`): Status (e.g. New), Priority (P3), Severity, Assignee, CC, Reporter, Components, Blocking/Blocked-by, Hotlists, Created/Modified, plus field-level edit affordances (`issue-metadata-field-edit-icon`, `bv2-field-user-action-footer`).
- The metadata fields are editable in place (bv2-editable-title / bv2-editable-value) — the detail page IS the edit surface.

## Mapping notes for the frontend (workflow policy, not extdeps)
- Issue = roadmap node; Component = domain placement (owner/lane); Blocking edges =
  dependency edges; Stars/CC/Upvotes = per-user subscription facts (new modeled state).
- Status = our derived present-tense explanation mapped to the product's standings
  (open/new/assigned/accepted/fixed/verified), never a flattened label.
- "To be verified" (stock view) = our verification-owed candidates.
