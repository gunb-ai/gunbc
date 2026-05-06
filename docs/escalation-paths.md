# Escalation Paths — gunbc Compiler

**Status:** ACTIVE. Authoritative **union receipt + conflict map** for "if X happens, escalate" clauses across briefs and authority docs. The source briefs remain authoritative on their own escalation clauses (per §"How to use" below); this doc owns the union view + cross-brief consistency surfacing.

**Last refresh:** 2026-04-26 against main HEAD `407a8bcb1`. Sweep boundary, refresh discipline + 3 surgical fixes documented below.

## Purpose

Every brief that says "STOP-AND-ESCALATE" or "if X, escalate to Y" creates an implicit three-part contract: trigger condition + escalation target + action expected. Without a single map, drift accumulates — a trigger can name a target that doesn't exist, or two briefs route the same trigger class to different targets.

This doc is the union map at any point in time. Maintained as the receipt for **"no escalation surprises"** — every escalation clause shipped in a brief should be traceable here, with evidence verified at HEAD.

## How to use

- **Authoring a new brief:** check this map for existing escalations on similar trigger classes; align target + action with established pattern unless dissolving by structural argument.
- **Reviewing a PR:** verify any new STOP-AND-ESCALATE clause grounds against this map; flag if the trigger is unmeasurable or the target is unattested on main.
- **Escalation in practice:** if a worker hits a STOP-AND-ESCALATE condition, this map is the lookup for who to surface to and how.
- **Authority of this doc:** **descriptive, not prescriptive.** The source briefs remain authoritative on their own escalation clauses; this doc is the union receipt and conflict-surfacing tool. If a trigger conflicts with this map, fix the brief, then refresh this map.

## Sweep methodology

Patterns swept (case-insensitive):
- `STOP[- ]AND[- ]ESCALATE` (literal, hyphen / space variants)
- `escalat` (catches escalate, escalation, escalates, etc.)
- `surface to (Director|user|manager)`
- `Director arbitrates` / `Director adjudicates` / `Director resolution`
- `out of scope.*escalate` / `out of scope.*Director`

Locations swept (main HEAD only):
- `INVARIANTS.md`, `ROADMAP.md`, `THESIS.md`
- `docs/r2-structure.md`
- `docs/briefs/*.md`
- `docs/design-*.md` top-level docs
- `docs/thesis/*.md`

**Open-PR content (PR #835 PM briefs, PR #836 Director worker briefs) is NOT in this sweep.** Both PRs add escalation clauses; second-pass sweep folds those in once the PRs merge.

## Groundedness vocabulary

- **GROUNDED:** trigger condition is concrete + measurable — specific gate name, PR number, file:line that exists, mechanical search verifiable.
- **SOFT:** trigger condition is a judgment call during execution — scope creep, semantic comparison, consumer discovery. Legitimate but requires human judgment to fire.

73% of clauses are GROUNDED (32 of 45); 27% SOFT (13 of 45).

## Map — 45 clauses across main HEAD

### Escalation target: Director (most common)

| # | Source | Trigger | Action | Groundedness |
|---|---|---|---|---|
| 1 | `docs/briefs/extdeps-loader-close-worker.md:65` | SG-0 stance choice reveals loader logic cannot fit PB-1's emerging pattern | Confirm ratchet-bump explicitly before PR commits | SOFT |
| 2 | `docs/briefs/extdeps-loader-close-worker.md:68` | Coverage scope (rust-only vs all extdeps languages) reveals divergent loader patterns per language | Surface divergence; pick rust-only + re-dispatch other languages | SOFT |
| 3 | `docs/briefs/extdeps-loader-close-worker.md:69` | Public accessor shape requires extending Dag API surface beyond existing accessors | Surface extension proposal; coordinate with Zero-Floor Manager | GROUNDED |
| 4 | `docs/briefs/extdeps-loader-close-worker.md:70` | `bootstrap.rs:16-19` comment update reveals cross-doc consistency implications | Surface cross-doc check; route to docs-cascade if needed | SOFT |
| 5 | `docs/briefs/extdeps-loader-close-worker.md:71` | DB-8 `self_host_fixed_point` drifts | STOP immediately | GROUNDED |
| 6 | `docs/briefs/t-substrate-valuebody-map-worker.md:69` | Parser sub-lane has not landed (sequencing error) | STOP; verify parser sub-lane PR merged before proceeding | GROUNDED |
| 7 | `docs/briefs/t-substrate-valuebody-map-worker.md:70` | Element-shape choice reveals consumer needing non-string keys | Director-call on whether to land general form upfront | SOFT |
| 8 | `docs/briefs/t-substrate-valuebody-map-worker.md:71` | `ValueBody` exhaustive-match wildcard `_` swallowing new variant found | STOP; convert to exhaustive in this PR or follow-up | GROUNDED |
| 9 | `docs/briefs/t-substrate-valuebody-map-worker.md:72` | substrate.dag declaration changes surface | Coordinate with PB-Substrate; STOP | SOFT |
| 10 | `docs/briefs/t-substrate-valuebody-map-worker.md:73` | Consumer-mirror retirement breaks downstream patterns | Audit reveals work, not just retirement; STOP | SOFT |
| 11 | `docs/briefs/t-substrate-valuebody-map-worker.md:74` | DB-8 fixed-point drifts | STOP immediately | GROUNDED |
| 12 | `docs/briefs/t-substrate-valuebody-map-worker.md:75` | Serializer / cementer / fixed-point machinery doesn't extend | STOP | GROUNDED |
| 13 | `docs/briefs/b2-lower-fn-body-arrow-rederive-worker.md:52` | Audit reveals legitimate runtime path that produces non-`Arrow` connectives | STOP; brief framing is wrong | GROUNDED |
| 14 | `docs/briefs/b2-lower-fn-body-arrow-rederive-worker.md:54` | Removing fallback breaks v2-compiler-tests (not v3 tests) | STOP; v2-compiler-tests may exercise legacy hedge intentionally; surface for design call | SOFT |
| 15 | `docs/briefs/b2-lower-fn-body-arrow-rederive-worker.md:55` | DB-8 drifts | STOP immediately | GROUNDED |
| 16 | `docs/briefs/p0-bug-no-profile-sentinel.md:61` | Removing sentinel reveals multiple callers relying on fabricated string format | STOP; list callers; propose whether real bugs or need absence-handling | GROUNDED |
| 17 | `docs/briefs/p0-bug-no-profile-sentinel.md:62` | `container_param_name_required` is called from hot paths that can't easily return `Option` | STOP; may need broader refactor of caller's signature | GROUNDED |
| 18 | `docs/briefs/p0-bug-no-profile-sentinel.md:63` | Additional `__BUG_*` / `__EMIT_BUG_*` sentinels found in broader grep | STOP; list separately as named follow-ups | GROUNDED |
| 19 | `docs/briefs/t-impossiblebugs-nested-optional-flatten-worker.md:72` | Surface-upstream investigation reveals `T??` parses to something other than nested `OptionalOf` | STOP; Director-call on which surface to dissolve | GROUNDED |
| 20 | `docs/briefs/t-impossiblebugs-nested-optional-flatten-worker.md:72` | Only some-but-not-all consumers see nested form | STOP; surface for dissolution decision | GROUNDED |
| 21 | `docs/briefs/t-impossiblebugs-nested-optional-flatten-worker.md:73` | Substrate-attachment requires inventing fundamental new vocabulary beyond cardinality-substrate | STOP; may indicate lane is mis-scoped | SOFT |
| 22 | `docs/design-substrate-carrier-port-program.md:120` (Lane E-T) | Carrier requires substrate connective not already present | Escalate to C1 lane (Director opens C1 substrate-capability lane — see Fix 3 below) | GROUNDED |
| 23 | `docs/design-substrate-carrier-port-program.md:126` (Lane E-C) | `kernel_algebra_profile` has a v3 gap | Surface, don't paper over | SOFT |
| 24 | `docs/design-substrate-carrier-port-program.md:135` (Lane E-I) | Master-theorem machinery doesn't lower | Surface as decidability/emit gap | GROUNDED |
| 25 | `docs/design-substrate-carrier-port-program.md:148` (Lane E-P) | Option (a) requires new substrate connective | Escalate to C1 lane (Director opens C1 substrate-capability lane — see Fix 3 below) | GROUNDED |
| 26 | `docs/design-substrate-carrier-port-program.md:148` (Lane E-P) | Option (b) requires lens capability v3 doesn't yet have | Surface emit gap | SOFT |
| 27 | `docs/design-substrate-carrier-port-program.md:148` (Lane E-P) | Any option reveals `TransformTarget` distinctions collapse information v2's `ExprCall` preserved | Modeling discovery, escalate | SOFT |
| 28 | `docs/design-pure-bootstrap-zero.md` (implicit) | Migration path for a file in 35-file audit reveals path isn't structurally achievable | Escalate to PB-tier redesign | SOFT |
| 29 | `docs/r2-structure.md:147` | Manager discovers program needs to expand (e.g., class-of-pattern dissolution under single-item brief) | Director adjudicates whether to expand program or split new lane (decision artifact format — see Fix 2 below) | SOFT |
| 30 | `ROADMAP.md` (Grounding lane) | Scope changes to Grounding Manager's program | Route to director; amendments to THESIS.md require director-authored PRs | SOFT |
| 31 | `INVARIANTS.md#p5-progress-is-dissolution` | Paired-dispatch or per-PR gate violations / scaffold without named dissolution trigger | Release Manager surfaces violations; per-brief enforcement at authoring manager's point | SOFT |
| 32 | `INVARIANTS.md#p5-progress-is-dissolution` | Velocity tripwire fires (≥3:1 ratio scaffolds:deletions in 7-day window across all managers) | Release Manager surfaces to Director; indicates systemic violation pattern | SOFT (interpretation) / GROUNDED (calculation) |

### Escalation target: Substrate Manager (post-R2-spawn) (1 clause)

| # | Source | Trigger | Action | Groundedness |
|---|---|---|---|---|
| 33 | `docs/r2-structure.md:114` | T-ImpossibleBugs class surfaces substrate gap | Escalate to Substrate Manager rather than expanding T-ImpossibleBugs scope | SOFT |

### Escalation target: Surface Manager (1 clause)

| # | Source | Trigger | Action | Groundedness |
|---|---|---|---|---|
| 34 | `docs/briefs/b2-lower-fn-body-arrow-rederive-worker.md:53` | Diagnostic-emission requires substrate work beyond C-8's existing shape | STOP; coordinate diagnostic taxonomy | SOFT |

### Escalation target: Zero-Floor Manager (program-internal) (4 clauses)

| # | Source | Trigger | Action | Groundedness |
|---|---|---|---|---|
| 35 | `docs/briefs/pb-1-e-residual-scaffold-retirement-worker.md:64` | None of (i)/(ii)/(iii) mechanism options preserve DB-8's no-compromise property | STOP; weigh in before any PR commits to weaker DB-8 | SOFT |
| 36 | `docs/briefs/pb-1-e-residual-scaffold-retirement-worker.md:65` | Retiring `load_runtime_bootstrap_authorities` reveals consumers beyond named scaffold paths | STOP; brief's "scaffold-only" framing was wrong | GROUNDED |
| 37 | `docs/briefs/pb-1-e-residual-scaffold-retirement-worker.md:66` | New DB-8 mechanism (i) snapshot-split surfaces unexpected substrate-shape questions | STOP; may belong in PB-Substrate proper | SOFT |
| 38 | `docs/briefs/pb-1-e-residual-scaffold-retirement-worker.md:67` | DB-8 fixed-point drifts | STOP immediately | GROUNDED |

### Escalation target: DB-author + Director (DB revision) (3 clauses)

| # | Source | Trigger | Action | Groundedness |
|---|---|---|---|---|
| 39 | `docs/design-db20-lane2-stage2e-parallelism-lens.md:78` | Extending `WorkflowEffect` beyond DB-18's four variants becomes necessary | Escalate to DB revision | GROUNDED |
| 40 | `docs/design-db20-lane2-stage2e-parallelism-lens.md:81` | `ParallelEffect.branches` cardinality must change | Escalate to DB revision | GROUNDED |
| 41 | `docs/design-db20-lane2-stage2e-parallelism-lens.md:82` | A stored witness field becomes provably non-derivable from `OperationEffect` | Escalate to DB revision | SOFT |

### Escalation target: program-internal (manager-only) (4 clauses)

| # | Source | Trigger | Action | Groundedness |
|---|---|---|---|---|
| 42 | `docs/briefs/pb-1-e-residual-scaffold-retirement-worker.md:68` | Pilot scope balloons beyond Deliverables A+B | Zero-Floor Manager STOP | SOFT |
| 43 | `docs/briefs/extdeps-loader-close-worker.md:69` | Public accessor shape coordination | Zero-Floor Manager (alongside Director) | GROUNDED |
| 44 | (cross-cutting) | Bottleneck watch — workers idle >7 days waiting for manager-authored briefs | Surface to Director from Release Manager via velocity-tripwire | GROUNDED (calculation) / SOFT (cause attribution) |
| 45 | `docs/r2-structure.md` Substrate Manager | Substrate becomes new bottleneck — split B4 into dedicated B4 Identity-Carrier Manager | Director call | GROUNDED (trigger metric) / SOFT (split decision) |

## Authority-conflict notes

Three minor naming-tightening opportunities surfaced; **no real conflicts** on main:

1. **Substrate carrier-escalation target naming inconsistency.** `docs/design-substrate-carrier-port-program.md` (E-T / E-C / E-I / E-P) routes substrate gaps to "C1 lane" depending on complexity. `docs/r2-structure.md §"Impossible-Bugs Manager"` routes substrate gaps to "Substrate Manager." Same trigger class, different vocabulary. Compatible scoping (C1 = lane vehicle; Substrate Manager = human target) but readers will trip on it. Addressed by **Fix 3** below.

2. **DB-revision vs Director.** `docs/design-db20-*.md` routes WorkflowEffect / ParallelEffect / witness-field escalations to "DB-author + Director" (DB revision). On main there is no standing "DB-author" manager role; DB authorship is attributed to "user" in ROADMAP. **Resolution:** these are pre-R2 authority documents (DB-history domain). Post-R2 promotion, DB amendments route to Director per single-authority discipline; no real orphan. Worth a housekeeping note at R2 promotion to update DB-revision escalation language.

3. **Ratchet-bump escalation path.** `extdeps-loader-close-worker.md` frames "ratchet-bump is STOP-AND-ESCALATE" to Director. `pb-1-e-residual-scaffold-retirement-worker.md` routes to Zero-Floor Manager as first escalation. Correct specialization (extdeps is cross-program → Director; pb-1-e is program-internal → manager); not a conflict but worth noting.

## Three surgical fixes (applied alongside this doc)

The sweep surfaced three clauses where the trigger is grounded but the resolution path is implicit. Each is a 1-2 line edit to source authority docs, applied in the same PR as this map:

### Fix 1 — Signal channel naming (in `docs/r2-structure.md` §"Manager structure")

**Gap:** Briefs say "STOP — surface to Director" without naming the channel. Practiced via session inbox issues today, but no authority doc says so.

**Edit:** add a sentence to the "Manager structure" preamble naming the signal channel: GitHub session-inbox issue comment for human-target escalations (Director / specific manager); cross-manager queue for inter-manager signals per the R1 `Cross-manager notifications queued` brief pattern.

### Fix 2 — Director decision-artifact format (in `docs/r2-structure.md` §"Director (cross-program coordinator)")

**Gap:** `:147` says "Director adjudicates whether to expand the program or split a new lane" but doesn't name where the decision lands.

**Edit:** add a sentence stating: "Director's decision lands as either (a) an amendment PR to the brief that surfaced the question, with explicit justification in the PR description; or (b) a sibling brief if the decision creates a new program scope. Both reference the originating discovery PR."

### Fix 3 — C1 lane explicit owner (in `docs/design-substrate-carrier-port-program.md`)

**Gap:** STOP clauses in E-T (line 120) and E-P (line 148) say "escalate to C1 lane" but no C1-lane manager exists on main. Director fields the escalation in practice.

**Edit:** rewrite each clause from `escalate to C1 lane` to `escalate to Director (Director opens a C1 substrate-capability lane if escalation requires substrate work)`. Director becomes the explicit receiver; C1 lane becomes the optional dispatch outcome.

## Refresh discipline

- **When to refresh:**
  - At every release transition (R1→R2 close-and-spawn; R2→R3 escape-hatch invocation if it fires).
  - On user direction.
  - On PR review when ≥3 new escalation clauses author into briefs in a single PR.
- **Refresh process:** sweep main + open-PR-on-deck content; rebuild table; flag changes in PR; surface authority conflicts and orphan escalations as PR review items. Land alongside any source-doc fixes the sweep surfaces.
- **Sweep boundary:** main only at refresh time. Open-PR content swept separately during PR review and folded into this map on PR merge.
- **Outstanding sweep debt:** PR #835 (PM portion, 6 R2 manager briefs) and PR #836 (Director portion, 14 worker briefs) are NOT in this map's current sweep. Second-pass sweep + map merge planned on those PRs' merge to main.

## Cross-refs

- Authority docs swept: `INVARIANTS.md`, `ROADMAP.md`, `THESIS.md`, `docs/r2-structure.md`
- Brief locations swept: `docs/briefs/*.md`, `docs/design-*.md`, `docs/thesis/*.md`
- Discipline framework: `INVARIANTS.md#p5-progress-is-dissolution` "Dispatch-Discipline Mechanisms"
- Cross-manager signal pattern: R1 "Cross-manager notifications queued" brief pattern (per `docs/r2-structure.md`)
- Outstanding sweep targets: PR #835 (`pm/r2-manager-briefs`), PR #836 (`session/zesty-bear-812`)
