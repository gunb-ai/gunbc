# Tracker bundle handoff — 2026-09-27

Branch: `codex/tracker-bundle`, based on `01a30ad3fc` (`test/main-plus-roadmap-daily`).
Worktree: `/home/briansrls/scratch-wt/tracker-bundle`.

## Result

- One uniform table row per issue. TYPE and the separate project/task-count column are removed, along with the table's project-only payload. Existing recursive traversal still enumerates every issue. Bug/Task/Feature/Epic/Chore remain optional descriptive vocabulary; they confer no nesting or execution semantics.
- Headers expose modeled sort keys, ascending/descending sorting, stable ties, and missing values last in both directions. Topological order puts prerequisites first using stable Kahn traversal over all table rows, including filtered rows. A cycle refuses before changing the DOM. Column sorting exits topological mode; Undo restores the prior order and indicators.
- The assignee chip or `--` is the inline control. Session state comes from `/auth/session`. Fresh choices, CSRF proof, and revision come from `GET /issue/{node_id}/assignment`, a session-bound, non-cacheable response. Fabric is pinned first, then self, then deduplicated ledger recents. Legacy email-only recents are visible but disabled as unverified identities.
- Inline and detail assignment share the suggestion feed and `POST /issue/{node_id}/assign`. Cookies and CSRF travel together. Human assignment stays a plain claim. Fabric follows the existing launch-admission path; the complete response stays on the affected row, including a recorded assignment followed by launch refusal. No automatic POST retries. Successful updates reuse the canonical server-rendered avatar chip.
- Empty revision is now a real initial revision, not a bypass of the assignment stale-write check. Anonymous controls remain disabled with a named reason.

## Coordination and boundaries

New browser behavior is in `roadmap_issue_sort.dag` and `roadmap_inline_assignment_client.dag`; context projection is in `roadmap_inline_assignment.dag`. The `roadmap_component.dag` change imports/binds these programs. Page changes are in the assignment-form, suggestions, and issue-table regions. Styles are in the issue-table region. The account-header implementation was not edited.

The live checkout, deployment, services, containers, and fleet were not changed. Live assignment/worker execution remains to be observed after the owner lands and deploys the branch.

## Findings and remaining questions

1. The old task counter counted prerequisites after collapsing single-child dependency chains. It did not establish a separate containment relation. The counter is removed. If recursive containment progress is wanted later, decide explicitly whether containment is distinct from blockers; no containment facts were inferred here.
2. Priority and last-modified have no current row facts. Their cells remain empty and sort stably; do not substitute scheduling readiness or unrelated timestamps.
3. The directory/profile producer is not bound for human display names. Self can be labeled “Me”; other authenticated recents retain the existing principal-display fallback. Legacy emails cannot mint an authenticated principal.
4. At 390px, the existing header overflows and the dense table clips titles/status heavily. The header overlaps agent-52's account-area scope. Desktop layout, sorting controls, row feedback, and avatar preservation were inspected in emitted-page screenshots.
5. `witness_ticket_brief_budget_holds_and_reds` also fails on a clean checkout of base `01a30ad3fc` at `/home/briansrls/scratch-wt/tracker-bundle-baseline` (receipt: `/tmp/tracker-baseline-budget.log`). Its rule and ticket corpus were not changed by this branch.

## Validation

Final source validation ran 169 witnesses in one compiler load under `MemoryMax=6G`: 168 passed and the same pre-existing brief-budget witness failed. Breakdown: tracker model 36/36, new bundle 4/4, serving routes 44/44, page 83/84, CSS digest 1/1. Two artifact/diagnostic exports also passed. All 171 requested function verdicts were captured before stopping the interpreter, which continued consuming resources after emitting them; the command exit therefore reflects termination, not a green suite. The stylesheet pin was re-derived as `b2267bf9edb82b47`.

The final emitted JavaScript passed `node --check` and the Playwright interaction run: sorting directions and ties, numeric/missing keys, prerequisite ordering, cycle refusal without mutation, repeated Undo, filtered rows, Fabric refusal preserved on-row, human assignment, stale-write refusal, cookie + CSRF + revision POST, avatar preservation, Escape cancellation, disabled unverified recents, inert anonymous controls, and detail-form reuse. Local browser fixtures use mocked HTTP responses and a fixture cookie; no live issue is claimed. Artifacts are in `/tmp/tracker-preview` (desktop, mobile, and night screenshots); browser runner: `/tmp/tracker-browser-test.py`; final witness receipt: `/tmp/tracker-final-validation.log`.

The unchanged authored briefs responsible for the baseline red are:

- `2-scm-native-authority-program`: 134 words
- `floor-ceiling-roster-cut-completeness`: 123 words
- `frontend-eval-training-program`: 177 words
- `fleet-mtcollins1-first-host`: 153 words
- `fleet-slot-retirement-wet-proof`: 116 words
- `cardinality-vertical-slice`: 119 words

The budget remains 100 words. No ticket was shortened and no witness was weakened to make the branch appear green.
