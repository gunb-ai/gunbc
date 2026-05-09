# R3 Debt-Paydown program — per-PR discipline + lane coordination brief `(M, R3-standing)`

> **R3 Debt-Paydown Manager spin-up brief.** Per [`docs/r3-structure.md` §"Standing program — R3 Debt-Paydown"](../r3-structure.md) (NEW 2026-05-02; 9th standing R3 manager) + Director ratification 2026-05-02 ([gunbc#828 comment 4362742638](https://github.com/gunb-ai/gunbc/issues/828#issuecomment-4362742638)) + user directive *"R3 to clean up any and all debt we come across WITHIN R3 — even if it means dedicated debt paydown lanes/managers"*. Authored 2026-05-02 by R3 PB continuation (witty-tern-193) per inbox #1134 re-task. Ready for Debt-Paydown Mgr at spawn.

> **Pre-spawn discipline.** This brief locks the per-PR discipline rule + lane coordination shape now so the standing manager does not re-derive it at spawn. The manager owns enforcement, cadence, and worker dispatch — not re-authoring the rule. Per `feedback_standing_managers_need_owned_deliverables`: a standing manager with no owned program degenerates into Director-bottleneck.

## Read first

- **[`docs/r3-structure.md` §"Standing program — R3 Debt-Paydown" (lines 163-180)](../r3-structure.md)** — the parent scope statement: closure gate (`r3_debt_paydown_zero_remaining`), authority discipline, hybrid mechanism (per-PR rule + standing capacity), cross-program coordination shape.
- **[`INVARIANTS.md` §P5 "Progress Is Dissolution" Dispatch-Discipline Mechanisms](../../INVARIANTS.md)** lines 290-326 — the load-bearing prior art. The R3 program **layers on top of P5**, it does not duplicate. P5(b) per-PR gate covers SG-0 hand-Rust scaffold dissolution; this brief extends that gate's *shape* (single-checkable-receipt) to ROADMAP debt-row retirement. P5(c) velocity tripwire is the manager's standing reporting cadence.
- **[`.github/PULL_REQUEST_TEMPLATE.md`](../../.github/PULL_REQUEST_TEMPLATE.md)** — existing P5(b) per-PR gate UI; the R3 program extends it with a **debt-receipt** section.
- **[`docs/briefs/debt-paydown-synthesis-2026-04-25.md`](debt-paydown-synthesis-2026-04-25.md)** — synthesis-doc that surfaced the original layered-mechanism design (PR #810 §4); the R3 standing program is its standing-manager realization.
- **`feedback_standing_managers_need_owned_deliverables`** — standing manager territory needs complete owned programs + autonomous dispatch; Director-ad-hoc bottleneck is what this manager exists to dissolve.
- **`feedback_construction_over_ratchets`** — debt is dissolved structurally, not patched; if a debt row appears unretirable within R3 it surfaces as a substrate gap requiring a named R3 lane, not a deferral.

## Frame — standing capacity around an unconditional closure gate

The closure gate `r3_debt_paydown_zero_remaining` is **unconditional**: no tracked-debt rows survive R3 close, no post-R3 deferral path. Per `docs/r3-structure.md` line 173: *"a tracked-debt row deferred past R3 close is the bridge-as-steady-state pattern P5 explicitly forbids; an escape hatch for 'rare structurally-justified deferrals' reintroduces that pattern at lower cadence and must not exist."*

The discipline question is therefore not "which debt rows do we retire" — it is **"how do we wire R3's PR cadence so every tracked-debt row reaches a retirement PR receipt before R3 close, with the standing manager surfacing structurally-unretirable rows as substrate gaps rather than letting them silently slip past?"**

Three load-bearing mechanisms (lifted from P5 layered-discipline shape, scoped to R3 debt-row retirement):

1. **Per-PR debt receipt** (the early warning, every PR). Layered on P5(b)'s existing single-checkable-receipt gate; extends from "hand-Rust scaffold" coverage to "tracked-debt row" coverage. Vague deferrals rejected per the same author-facing checklist already shipped in `.github/PULL_REQUEST_TEMPLATE.md`.
2. **Standing manager territory** (the steady-state capacity). Owns systemic debt that doesn't fit organic per-PR cleanup; spawns workers for retirement work as needed; reports cadence-aligned to Director's autonomous-loop pattern.
3. **Velocity tripwire** (the late warning, window grain). Per P5(c): introduction:dissolution PR ratio ≥3:1 in any 7-day window puts ad-hoc lane dispatch under Director review. Manager surfaces tripwire readings to Director on cadence; this is **reporting**, not new gate authoring.

## Definition — tracked-debt row (canonical)

A **tracked-debt row** is an entry that satisfies the following predicate:

> The entry enumerates a specific item (a bug, scaffold, dissolution trigger, deferral, or follow-up) **and** carries at least one of: a named owner, a named dissolution mechanism, a retirement-PR pointer, or a "remove once X" condition. Preamble / framing / classification / lane-overview sections that describe categories without enumerating specific items are **not** tracked-debt rows.

**Authoritative location** is ROADMAP.md by default. When ROADMAP.md contains a *delegating pointer* — a non-enumerative section that explicitly forwards the row inventory to a named external file (e.g. `## Scheduled deletions` forwards to `docs/history/roadmap-scheduled-deletions.md`; `## Active deferrals` forwards the full ledger to `docs/history/roadmap-active-deferrals.md` while keeping summary lines inline) — the predicate-passing rows in the *delegate file* count as tracked-debt rows. Both the ROADMAP pointer-section and the delegate file's row container are cited together so the manager's ledger reaches the actual rows, not the redirect.

The current ROADMAP heading shapes that satisfy this predicate (enumerated explicitly so the manager has a concrete starting set, not an inferential one):

- `### Post-merge debt (...)` sections.
- `## Tracked debts — <period> analyses` section's predicate-passing sub-rows only — currently:
  - `### PR #<N>` ship-with-debt-receipt sub-rows.
  - [`### P0 — real bugs (silent wrong execution)`](../../ROADMAP.md#p0--real-bugs-silent-wrong-execution) / [`### P1 — fabrication / fail-open boundaries`](../../ROADMAP.md#p1--fabrication--fail-open-boundaries) / [`### P2 — structural compression (biggest ROI)`](../../ROADMAP.md#p2--structural-compression-biggest-roi) / [`### P3 — modeling gaps`](../../ROADMAP.md#p3--modeling-gaps) / [`### P4 — type refinement / modeling faithfulness`](../../ROADMAP.md#p4--type-refinement--modeling-faithfulness) priority-category sections (real bugs / fabrication boundaries / structural compression / modeling gaps / type refinement), nested under this `## Tracked debts` parent.

  The parent `## Tracked debts` container is **not itself** a debt row, and predicate-failing sub-rows under it (e.g. [`### Debt classification — framing`](../../ROADMAP.md#debt-classification--framing), or any future "practice, not a debt" / "reviewer-noise class" / similar non-enumerative subheading) are excluded per the predicate above even though they sit under a debt-bearing parent.
- [`## Active deferrals — follow-up work from merged PRs`](../../ROADMAP.md#active-deferrals--follow-up-work-from-merged-prs) (`DB-*` deferral ledger — `DB-1` through `DB-20` summary lines are inline in ROADMAP; full deferral ledger at [`docs/history/roadmap-active-deferrals.md`](../history/roadmap-active-deferrals.md). Per-DB design notes at `docs/db-history/db-<N>.md`).
- ROADMAP delegating pointer [`## Scheduled deletions — scaffolds with named dissolution triggers`](../../ROADMAP.md#scheduled-deletions--scaffolds-with-named-dissolution-triggers) → predicate-passing rows live in [`docs/history/roadmap-scheduled-deletions.md` `### Table`](../history/roadmap-scheduled-deletions.md) (the markdown row table is the actual debt-row container — `ArrowBody::Pending`, `ArrowBody::Unparsed` cases, `ValueBody::Unparsed`, `TransformTarget::Operator`, `LogicalOp` / `OperatorKind::Logical`, `ResolvedByName` AtomPayload, compiler-internal `declaration_by_name` call sites, `Node.name` field, and any future scaffold rows added to the table). The ROADMAP section by itself is non-enumerative; the ledger inventory must reach into the delegate file to enumerate rows, not stop at the pointer.
- [`### §6a MethodContract per-field dissolution triggers (R2 Release follow-through)`](../../ROADMAP.md#6a-methodcontract-per-field-dissolution-triggers-r2-release-follow-through).

**Explicitly excluded** (predicate-fails): [`### Debt classification — framing`](../../ROADMAP.md#debt-classification--framing) (preamble explaining categories, not a debt-row container); `### Goals / Lanes / Lane acceptance / Dependency DAG / Principles / Sketch vs Oracle framing / Status at a glance / Architecture / What NOT to build yet / Open design questions` and similar lane-overview / framing headings; any heading whose body is prose explanation rather than enumerated items.

**Future tracked-debt headings** must satisfy the predicate above to count; the manager amends this canonical definition section in a tracked PR when the ROADMAP grows a new debt-bearing heading shape (the predicate is the authority, not the enumeration — the enumeration is a concrete starting set that gets extended in lockstep with ROADMAP growth).

This definition is the canonical scope for the per-PR rule, the Slice 1 ledger inventory, and the Acceptance closure gate below; downstream sections refer back here rather than re-enumerating, so the union cannot drift between sites.

## Per-PR debt-receipt rule (load-bearing deliverable)

**Every R3 PR's description includes a "Debt receipt" section** with **exactly one** checkable disposition of any tracked-debt row the PR touches (per the canonical definition above). Three valid dispositions, one of which must be picked:

1. **Debt paid** — the PR retires a specific tracked-debt row. Cite the row by repo-relative path + heading/anchor or permalink to the ROADMAP entry, AND name the dissolution mechanism (deleted file, dissolved scaffold, structural-test landing, etc.). Reviewer must be able to open the cited row in one hop and confirm it disappears (or is marked RESOLVED) post-merge.
2. **Debt found, routed** — the PR introduces or surfaces a tracked-debt row but does not retire it. Cite the new ROADMAP row (path + anchor) AND name the **paydown-lane retirement PR** (issue/PR number + brief title) that owns the retirement. The retirement PR must exist or be filed in this same author session.
3. **No debt touched** — the PR neither pays down nor introduces tracked debt. Affirmative statement: *"No tracked-debt row touched in this PR; no debt-receipt entry required."* This disposition is rejected if any reviewer surfaces a debt row the PR actually touched — the author re-routes to (1) or (2).

**Rejected** as debt-receipt content (same shape as P5(b) gate's "Insufficient" list):

- "see ROADMAP" / "TBD" / "tracked elsewhere" / "follow-up PR" without an issue or PR number.
- A debt-row name without a path/anchor link a reviewer can open in one hop.
- A "debt found" disposition without a routed retirement PR (the routing **is** the receipt — without it, the row reduces to a vague deferral).
- Mixing dispositions (1) and (2) in a single bullet — the manager cannot enforce single-checkable-receipt on a compound entry. Two debt rows touched → two debt-receipt bullets, each independently checkable.

**PR-template extension** (the manager's first owned authoring after spawn): extend `.github/PULL_REQUEST_TEMPLATE.md` with a `## Debt receipt (R3 standing program)` section after the existing `## Per-PR dissolution gate (...)` section. Single-bullet shape, three-disposition author-facing checklist mirroring P5(b) gate ergonomics. **Not authored in this brief** — the manager owns the template-extension PR as its spawn-time Slice 1 deliverable so the rule lands with manager-author identity, not pre-spawn-author identity.

## Lane coordination — cross-program queue + closure-ledger receipts

Per `docs/r3-structure.md` line 178: *"Debt-Paydown Mgr coordinates with all 8 other R3 managers via the standard cross-manager queue + closure-ledger receipts."* Concrete mechanism:

| Coordination surface | Owner | Shape |
|---|---|---|
| **Cross-manager queue** | R3 Release Manager (existing R3 surface) | Debt-Paydown Mgr files queue items per debt row needing a lane-owning manager's worker (e.g. a lens debt row routes to Substrate or Verification Mgr). Standard shape: queue item names the row, the gate it blocks (`r3_debt_paydown_zero_remaining`), the suggested lane, and the no-deferral reminder per `docs/r3-structure.md` line 173. |
| **Closure-ledger receipts** | Each lane-owning Mgr | When a lane-owning Mgr's worker retires a tracked-debt row, the worker PR cites the row + Debt-Paydown Mgr's tracking entry. Debt-Paydown Mgr reads the merged PR's debt-receipt section (per the per-PR rule above) and updates its tracking ledger. The ledger surface is owned by Debt-Paydown Mgr; per `feedback_standing_managers_need_owned_deliverables` it is a real owned deliverable, not a Director-side artifact. |
| **Velocity tripwire reporting** | Debt-Paydown Mgr → Director | Cadence-aligned to Director's autonomous-loop pattern (per `feedback_director_30min_cadence`). Manager reports the introduction:dissolution PR ratio over the 7-day window; ≥3:1 triggers the Director-review escalation per P5(c). Below 3:1 is advisory. **No new gate authoring** — this is surface-and-route, not adjudicate. |
| **Substrate-gap escalation** | Debt-Paydown Mgr → Director | Per `docs/r3-structure.md` line 173: if a row appears unretirable within R3 it surfaces as a substrate gap requiring a named R3 lane, **not a deferral**. Manager files a substrate-gap escalation; Director opens the named R3 lane (or routes to existing lane-owning Mgr); per `feedback_construction_over_ratchets`, this is the "model first, violations dissolve" path — the gap names the missing structural concept, not the workaround. |

**Authority discipline** (verbatim from `docs/r3-structure.md` line 175): *"Manager authors per-PR rule documentation + standing reporting cadence. Does not author lane-level structural-acceptance gates (those owned by lane-owning managers). Does not adjudicate cross-program scope conflicts (those route to Director). Does enforce: every tracked-debt row gets a retirement PR before R3 close."*

## Slice — manager spawn → owned deliverables

Per `feedback_standing_managers_need_owned_deliverables`, the manager's spawn Slice 1 is the per-PR-rule lockdown. Subsequent slices are standing capacity.

**Read this brief as policy/coordination only.** The Slice 1 entries below describe **manager-owned post-spawn deliverables** — they are not authoring instructions already in motion, and this brief does not pre-author the template extension, the ledger, or the cadence cron. Each Slice 1 deliverable lands under manager identity in a separate PR after manager spawn.

1. **Slice 1 (spawn-PR; M-sized)** — Debt-Paydown Mgr authors:
   - **PR-template extension** at `.github/PULL_REQUEST_TEMPLATE.md` (the `## Debt receipt (R3 standing program)` section per the rule above).
   - **Initial debt-row inventory ledger** at `docs/debt/r3-paydown-ledger.md` (or similar — manager picks final path) — enumeration of every tracked-debt row in ROADMAP per the canonical definition above (§"Definition — tracked-debt row"). Each row carries: row name, source heading + line/anchor, current state (open / routed / retired), routed-lane (if applicable), retirement-PR number (if applicable). The ledger is the manager's primary owned-deliverable surface; its scope is the canonical union, so the closure gate cannot false-green by ledger-scope drift.
   - **Cadence cron** — recurring routine (per `/schedule` discipline) for tripwire reading + ledger sweep + Director cadence report. Frequency: aligned to Director autonomous-loop (30min per `feedback_director_30min_cadence`) for ledger sweep; weekly for tripwire window report.
2. **Slice 2+ (standing-capacity; ongoing)** — Debt-Paydown Mgr dispatches retirement workers per ledger row, files cross-manager queue items per debt row needing a lane-owning Mgr's worker, escalates substrate-gap rows to Director. Each retired row updates the ledger; closure gate `r3_debt_paydown_zero_remaining` flips when the ledger has zero open rows.

## Acceptance — what the standing program must deliver before R3 close

- [ ] PR-template extension landed at `.github/PULL_REQUEST_TEMPLATE.md` (Slice 1).
- [ ] Initial debt-row inventory ledger landed at the manager-picked path under `docs/debt/` (Slice 1).
- [ ] Cadence cron created (Slice 1).
- [ ] Every tracked-debt row in ROADMAP per the canonical definition above (§"Definition — tracked-debt row") reaches **retirement-PR-merged + ledger row marks RETIRED**. Substrate-gap escalation to a named R3 lane is an **interim state only** — the row remains OPEN in the ledger until the named lane's retirement PR merges and lands a debt-receipt citing the row. Per `docs/r3-structure.md` line 173 + §P5 Progress Is Dissolution, routing is not dissolution; a routed row that has not been retired by R3 close is the same forbidden post-R3 deferral pattern under a different label. **Zero rows survive R3 close** means zero open ledger rows, including zero routed-but-not-retired rows.
- [ ] Velocity tripwire reading falls below 3:1 introduction:dissolution ratio at R3 close (or, if above, Director-review escalation is active and cited).
- [ ] R3 close gate `r3_debt_paydown_zero_remaining` flips green — the ledger has zero open rows and the close-gate test (owned by R3 Release Manager) reads the manager's ledger as zero-open.

## STOP-AND-ESCALATE

Per [`docs/escalation-paths.md`](../escalation-paths.md):

- **A debt row appears unretirable within R3** (no structural dissolution path, no lane-owning Mgr can retire it) → STOP. Surface as substrate-gap escalation per `docs/r3-structure.md` line 173. Director opens a named R3 lane; the ledger row stays OPEN until that named lane's retirement PR lands. Routing is **not** terminal — the row only marks RETIRED when the lane's retirement-PR debt-receipt cites it. **Do not record a deferral** — the directive that motivated this manager's creation forbids it, and routing-as-terminal would re-introduce the same deferral pattern under a different label (per the Acceptance section above).
- **A PR's debt-receipt section is rejected by reviewer + the author cannot route to (1)/(2)/(3) without authoring new substrate** → STOP. Manager surfaces the gap to lane-owning Mgr (the substrate question is theirs); the PR holds at gate-fail until the route lands.
- **Velocity tripwire ≥3:1 in the 7-day window AND the cause is a coordinated multi-PR slice that introduces scaffolds dissolved later in the slice** → STOP **caution** (not full STOP). P5(c) wording is *"after a manual sweep for dissolution-bearing feature PRs to avoid heuristic false positives"* — manager's tripwire report includes the manual-sweep step before flipping to Director-review escalation. Below-threshold-after-sweep is advisory.
- **Manager finds itself authoring lane-level structural-acceptance gates** → STOP. Per `docs/r3-structure.md` line 175 authority-discipline: those are lane-owning Mgr territory. Manager re-routes the gate-authoring to the lane-owning Mgr and tracks it in the ledger as a routed row, not as own work.

## Cross-refs

- Parent: [`docs/r3-structure.md` §"Standing program — R3 Debt-Paydown"](../r3-structure.md) lines 163-180.
- Authority lock: [`docs/r3-structure.md` §"Manager structure" item 4](../r3-structure.md) line 193.
- Discipline anchor: [`INVARIANTS.md` §P5 Dispatch-Discipline Mechanisms](../../INVARIANTS.md) lines 290-326 (the load-bearing prior-art).
- Existing PR-template gate (P5(b)): [`.github/PULL_REQUEST_TEMPLATE.md`](../../.github/PULL_REQUEST_TEMPLATE.md).
- Synthesis source: [`docs/briefs/debt-paydown-synthesis-2026-04-25.md`](debt-paydown-synthesis-2026-04-25.md).
- Standing-manager discipline: feedback memory `feedback_standing_managers_need_owned_deliverables`, `feedback_director_30min_cadence`.
- Construction-over-ratchets discipline (substrate-gap escalation path): feedback memory `feedback_construction_over_ratchets`.
- Director ratification: [gunbc#828 comment 4362742638](https://github.com/gunb-ai/gunbc/issues/828#issuecomment-4362742638) (2026-05-02).
- Cross-program coordination contracts (queue + closure-ledger): [`docs/r3-structure.md` §"Manager structure"](../r3-structure.md) (R3 Release Mgr owns the queue; lane-owning Mgrs file closure receipts).
