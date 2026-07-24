# Progress & observation — process→outcome discipline, one event model, N renderers

**Status:** DESIGN for review (operator-directed 2026-07-23). Authority migrates to the `.dag`
carriers when P0 lands (the laws become data rows; this doc is the design record and dissolves
per its doc-graph bind). **Reference implementation studied by execution:** `gunb-ai/gunb.ai`
`tools/terminal` (progress.go / box.go / emoji.go) — tests run green, driven live in TTY,
non-TTY, and failure modes 2026-07-23.

## 1. Thesis and displaced cost

The register thesis applied to process output: **quiet at arm's length; responsive up close;
every response true.** A progress line is a *projection of a fact the process already has* —
never a hand-authored string with a hand-chosen level.

Displaced costs already paid (the receipts that price this lane):

- The 2026-07-23 crawl window: ~10 minutes of `[floor-memory]` vitals with **zero activity
  lines** while single modules typechecked for 607s/506s — the operator watched a quiet log
  and could not tell what was running (run 30044816605).
- A wrong triage verdict (the "eviction never armed" misread) made **because the drain receipt
  only writes at walk end** and the walk timed out — one 55-minute CI cycle burned on an
  ambiguity a single unconditional arm-line would have closed.
- Workers hand-rolling the missing discipline ad hoc (#7129's `b295aa6`: unconditional arm
  line + early per-entry drain emission) — dialect pressure; the standard should land before
  three private dialects grow.
- The operator's standing complaint: messy interleaved CI logs, unattributed concurrency,
  things "going quiet."

## 2. The model (P0 — no rendering work until these carriers exist)

One event vocabulary, emitted by the process, consumed by every renderer:

- **`ObservationEvent`** = `subject` (a **containment path**: run ⊃ batch ⊃ entry ⊃ module ⊃
  phase — the tree the floor already walks) × `transition` × measured facts (wall, RSS,
  counts). Events are data: the stream is **replayable by construction**, which is what makes
  the flagship acceptance (re-render a captured run) trivial.
- **Transitions:** `Begin | Step { k, n } | Outcome`. Outcomes are a closed sum:
  `Done { facts } | Refused { diagnostic } | Failed { error, output } | TimedOut | Skipped |
  Final`. **`Refused` is distinct from `Failed`** — typed refusals are the house's §5 spine
  and get their own glyph and rendering (the reference implementation conflates these; we must
  not).
- **`BlockedOn { resource, holders, remaining }`** — lifted from the reference's
  `TaskContention`, with one upgrade: in gunbc these facts are **derived from scheduler and
  governor state, never hand-set** — the governor already knows when the cgroup throttles, so
  the crawl renders as `⏳ typecheck v2.compiler.normalized_tree — blocked: memory.high
  reclaim (high_events +90k/min)`.
- **`AttentionLevel` is derived, never chosen per site:** `Ambient | Notable (threshold
  crossing) | Anomaly (refusal, divergence, orphaned begin)`. Event-class → presentation is a
  **total assignment** — censused and walled like unthemed colors.
- **One glyph/material authority:** status → glyph (`⟳ ✓ ✗ ⛔ ⏳ ⏭` + the reward-animal rows
  for `Final`) → register color role, as data rows. Terminal ANSI materials are the register's
  roles realized for one more surface; the reward pattern (random animal on terminal success)
  lands as rows in this table, not a hardcoded function.

### The five laws (data rows, operator-signed 2026-07-23)

1. **Process → outcome, no orphans.** Every subject at entry grain and above emits a matched
   `Begin` and exactly one `Outcome`. An orphaned `Begin` is a detectable defect: a watchdog
   converts quiet-past-budget into a typed line (this also closes the batch-wall hang gap —
   a batch that never completes currently rides silently to the step cap).
2. **The heartbeat carries identity, not just vitals.** The per-minute telemetry line gains
   the current activity: `phase= entry= module= k/n`. No phase may exceed T seconds without a
   line naming its current subject.
3. **Attention escalates by dwell time, recursively.** A subject quiet past T surfaces its
   current child; past 2T the grandchild; recursively to the leaf. Same tree, same events —
   the collapse rule run in reverse under time pressure. (The reference implements one level
   of this — `autoExpandThreshold = 30s`, TTY-only; we generalize it and make it first-class
   in the CI renderer, where the pain actually lives.) T is a data row whose basis is the
   measured quiet-time distribution, not taste.
4. **Quiet at arm's length.** Sub-threshold work is silent; per-witness lines collapse to
   per-entry summaries; a red or refusal expands fully and names itself. The asymmetry is the
   design (`autoExpandMaxPending`-style bounds so a 200-task group never dumps).
5. **Every response true.** Every rendered number is a projection of a receipt fact. No vibes
   strings, no per-site log levels.

## 3. Renderers (P1+) — the contexts, with format contracts

One event stream, N realizations (§2). Each context gets a **format contract** — the
expectations below are the acceptance spec, not illustrations.

### 3a. CI log (GitHub Actions — the floor's context, built FIRST)

Append-only; no repaint. The reference implementation's biggest gap is here (its non-TTY mode
has no heartbeat, no escalation, no contention — the quiet-window problem verbatim), so this
renderer is where every law must be first-class.

```
::group::batch 3 — discovery (602 entries)
→ batch 3: discovery begin (602 entries, 2314 witnesses)
✓ entry effect_reach_test.dag (14 witnesses, 230ms)
… [collapsed: 213 green entries]
♥ t=29m phase=discovery entry=214/602 (emit/rust_binop_emit_test.dag) module=34/61 | rss=16.0G swap=32G(sat) high_events=+90k/min
⏱ entry 214 quiet 120s → typecheck v2.compiler.normalized_tree (34/61)          [T: child surfaced]
⏱ normalized_tree quiet 240s → blocked: memory.high reclaim (psi 9.0)           [2T: cause surfaced]
⛔ REFUSED batch 3: FLOOR-BATCH-OVER-BUDGET wall_ms=… budget_ms=… (budget row: …)
::endgroup::
✗ batch 3 — discovery: refused (1 refusal, 213/602 entries complete, 41m12s)
```

Contract: every `Begin` at entry grain+ has a matched outcome line **with duration**; heartbeat
per minute with identity; dwell escalation *appends* deeper-subject lines at T/2T/4T; anomalies
always render immediately and are never inside a collapsed group; a final summary table (the
reference's binary-cache table pattern) plus failed/refused boxes re-rendered at the end so
they survive log truncation; concurrency is attributed (every line carries its subject path or
lives inside its subject's `::group::`).

### 3b. Interactive TTY (`gunbc` CLI local runs)

The reference implementation's strengths, kept: preamble box (title + description required —
no anonymous processes); in-place repaint with spinner; group lines `⟳ › Discovery [214/602]
(current-entry)`; dwell escalation expands **in place**; `BlockedOn` renders inline with holder
and remaining time; failed/refused tasks render bordered boxes with captured output at final;
`Final` renders the reward animal. Upgrades over the reference: durations on outcome lines,
`Refused` distinct from `Failed`, escalation recursive rather than one-level.

### 3c. Receipt / file (JSONL of the event stream)

The same events as typed rows — the replay source for the flagship acceptance, and the
convergence point with the existing receipts: D4's discovery phase rows, the floor-drain early
emission, and `[floor-memory]` become projections of this stream. **Existing receipt file
formats do not change** (they have consumers); they become derived views, dissolving only by
their own triggers.

### 3d. Dashboard (belt B) — later consumer, same stream; out of scope here, must not be
precluded (the event schema is the wire contract).

### 3e. Non-TTY local pipe — the CI contract minus workflow grouping markers.

## 4. Migration and the wall

Census every existing emit site (`[floor-memory]`, `[typecheck-attribution]`, `[gantt]`,
`claim_executor` prints, shell echoes): each is classified **event-projection** (migrated) or a
counted frontier row (reason + dissolve-on) — the unthemed-color-census pattern. Once migrated,
a lens walls new bare prints outside the projection. The two hand-rolled precursors (#7129's
arm line and early drain emission) are the first migrations, not exceptions.

## 5. Phases, acceptance, REDs

- **P0 — model + laws + glyph authority** as `.dag` carriers. Witnesses: event→presentation
  totality; laws-as-data lockstep with the Rust emitter; glyph table single-authority
  (perturbation moves every renderer).
- **P1 — the CI renderer on the floor** (the pain point). Flagship acceptance: **re-render the
  captured crawl window** (run 30044816605's log) through the escalation law and produce the
  §3a trace — named activity where there was silence. REDs: a planted silent-phase (quiet past
  T with no subject line) reds the responsiveness witness; an orphaned `Begin` reds the
  watchdog; a `Refused` outcome rendered inside a collapsed group reds.
- **P2 — the TTY renderer**, sharing every carrier; REDs re-proven per contract.
- **P3 — census wall**: zero unclassified emit sites or counted frontier; the lens goes live.

Each phase green-by-execution; enrollment through the D5 batch wall (rendering must be
cost-negligible; the receipt says so, not the author).

## 6. Reference-implementation ledger (what we lift / fix / decline)

**Lift:** containment-tree state with derived group status (their `SetGroupStatus` is a
deprecated no-op — status is computed, a §5 touch worth keeping); nested sub-DAG progress;
auto-expand (generalized per Law 3); `TaskContention` (upgraded to derived facts);
required preamble; failure boxes with captured output; `PrintFinal` returning shown-failures;
the reward animal; TTY/CI dual rendering; bounded expansion.

**Fix:** no CI heartbeat/escalation/contention (TTY-only features — our floor lives in CI);
unattributed concurrent interleaving in the line protocol; no durations on outcomes;
`Failed`/`Refused` conflation; imperative mutation API instead of replayable events;
the emoji table hand-mirrored between Go and shell (a maintained dual representation — ours is
one authority projected everywhere).

**Decline:** importing the Go module itself; gunbc's renderers are emitted from the modeled
protocol. Long-term convergence: the event schema is the shared wire contract, and gunb.ai's
Go module can become one more renderer of it — cross-repo UX consistency by shared authority,
not by imitation.

## 6b. Interaction with the CI two-tier rework (addendum 2026-07-24)

The P0 model and the five laws are context-independent and change NOT AT ALL. What changes is
the CI renderer's contract (§3a) and the sequencing:

- **Three new event classes to render.** (a) Derived-clamp refusals (the rework replaces
  hand-set batch budgets with `overhead + units × avg`): render the refusal WITH its
  arithmetic — units, coefficient, overhead, actual wall, implied s/unit — so budget triage
  stops being a hand computation (every triage this week recomputed s/entry manually).
  (b) Placement dispositions: a run's summary names the PrTier/Gauntlet split — what ran,
  what deferred, and where the deferred work executes. (c) Lens findings (the door
  reintroduction): Anomaly-class, render immediately, never collapsed.
- **The heartbeat's identity line keys on clamp units:** `k/n units · s/unit vs signed avg` —
  the regression dial, live in the log.
- **AttentionLevel grounds on the signed constants** (§9.8 of the redesign): over-average =
  Notable, over-max/refused = Anomaly. No invented thresholds.
- **The pain point migrates to the gauntlet.** Post-rework, PR-tier runs are short and mostly
  Ambient; the long quiet runs live in the gauntlet/falsifier context (whole-corpus, cold) —
  that context is the escalation law's primary home, and the flagship crawl-window replay is
  exactly a gauntlet-profile run, so P1's acceptance is unchanged.
- **Sequencing + delivery (operator ruling 2026-07-24): ONE atomic PR** — P0 through P3 land
  together, entirely, after the atomic CI rework PR (both touch the floor's emit sites;
  churning them twice is the migration class the operator ruled out). P3's census enumerates
  the POST-rework emit sites. D4's deletion removes the selection-control step as a subject.
- **Human-legibility contract (operator, 2026-07-24 — the census bar, not a style note):**
  every run opens with a preamble naming what/why/how-much (run id, trigger, diff size,
  affected set, placement split); the heartbeat is at most one line per minute,
  **identity-first, vitals-suffix**, in human units (GiB, not raw bytes); raw telemetry dumps
  (`current=16111669248`-style) are census violations, not projections; readiness probes and
  shell echoes are Ambient (invisible at arm's length, expandable up close); per-witness PASS
  lines collapse to counts. `[floor-memory]`'s current shape — 60+ identical
  context-free byte dumps per hour — is the named negative example the census wall reds.
  **Selection prominence (operator, 2026-07-24): the diff→runs chain is the preamble's
  centerpiece.** Users must see the causal chain from THEIR files to what CI actually runs:
  a per-file attribution block (each touched file → the entries/witnesses it selected, docs
  files → "nothing"), the skip count with its audit pointer ("1,738 of 2,316 skipped as
  unaffected — audited cold every 4h"), the now-vs-later placement split, and — when
  selection widens — the widening named in plain language WITH the file that caused it
  ("cli_run.rs changes the compiler itself, so nothing can be skipped"). Per-file attribution
  is the same selection authority projected per touched file, not new telemetry. A refusal
  that traces to selection (a hub-file full-corpus run over budget) names the causing file in
  its explanation.
  **Attribution grain is the DECLARATION, not the file (operator, 2026-07-24):** under each
  touched file, list the qualified names the diff actually touches (diff hunks ∩ declaration
  spans — both already exist) with their change kind (edited / new / removed), then the
  witness count that follows. Honesty note: witness counts are attributed at the grain
  selection actually computes (module-level import closure today), stated as "via the N
  modules that reference these" — the display must never imply decl-grain selection before it
  exists. When the namespace lane's containment tree lands decl-grain selection, the SAME
  display shrinks the sets with zero format change — the UI leads, selection catches up. This
  is the containment-tree authority surfacing in the UX: a qualified name is a position, and
  the user reads their diff as positions touching witnesses.
  **Change-kind coloring (operator, 2026-07-24): git-diff convention** — added = green,
  modified = yellow, removed = red — as rows in the one glyph/material authority (register
  color roles realized as terminal ANSI; GitHub Actions logs render ANSI). Color is never the
  only channel: the textual kind tag (`edited` / `new` / `removed`) always prints beside it,
  so pipes and colorless contexts degrade losslessly.
  **No-ops are a closed sum, never a bare "nothing" (operator, 2026-07-24).** "Nothing"
  meaning several different things is the house's own state-space-conflation pattern
  (an Option/None carrying >2 meanings) applied to UX — each kind gets its own named line
  because their remedies differ:
  - `docs-policy` — documentation never selects executable work (fine, quiet);
  - `uncovered` — a touched declaration that NO witness references: rendered as a visible
    nudge (⚠️, "no witness covers this change"), because it is a coverage gap surfaced at PR
    time, not a comfort;
  - `no-decls-touched` — only comments/whitespace intersected (fine, says so);
  - `generated-artifact` — the file is a projection; its check is the drift gate against its
    authority, not witnesses (named, so "nothing" never appears next to ci.yml);
  - `deletion` — NOT a no-op: departed non-docs paths widen to baseline today, and the line
    says so with the cause ("a deleted module can't be scoped").
  A bare unlabeled "→ nothing" is a census violation.
  **Tone (operator, 2026-07-24):** arm's-length lines are plain sentences a human can read
  without decoding — "still in witness discovery: entry 214 of 602, pace 2.1s per entry
  (expected ~1s)" — never dense `key=value` chains; the dense form lives only in receipt
  boxes and files. Glyphs are **real emojis** from the one glyph authority (✅ ❌ 🚫 ⏳ ⏭️ 🚨,
  reward-animal rows for Final — the gunb.ai emoji pattern, kept); the periodic status line
  uses 🕐, not ♥.

## 6c. Relation to the frontend / visualization / UX lanes

Same house, different organs — related by shared authority, not by overlap:

- **The thesis IS the register thesis** (site-subsumption lane): *quiet at arm's length;
  responsive up close; every response true* — §1 applies it to process output deliberately.
- **The glyph/material table is a design-register instance:** status → glyph → **register
  color role**, with terminal ANSI as one more material realization of the same roles the
  site themes route. When the `gunbc.design.*` library (design-register lift) lands, the
  glyph table's color roles re-ground on it — a named dissolution trigger, NOT a blocking
  dependency (local role rows land first). The AttentionLevel→presentation total assignment
  mirrors the site's total role→material theme assignments, totality witnessed the same way.
- **The dashboard (belt B) is renderer N+1 of the same stream** — §3d already names it: the
  JSONL event schema is the wire contract; no second telemetry model may grow there.
- **gunb.ai's `tools/terminal` Go module** is declined as a dependency and named a future
  renderer of the shared schema — cross-repo UX consistency by shared authority, not
  imitation.

## 7. Non-goals

## 7. Non-goals

No external logging framework; no dashboard rebuild; no new telemetry sources in P0–P1 (render
only what is already computed — the single exception is plumbing subject identity to the
heartbeat); receipt file formats unchanged; the batch-wall in-flight deadline is its own row
(Law 1's watchdog complements it, does not replace it).
