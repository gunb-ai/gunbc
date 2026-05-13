---
status: Mgr canvas (substrate-shape question for Director-or-Mgr ratification; surfaced per feedback_substrate_shape_belongs_in_mgr_canvas)
authority parent: R3 Substrate Manager (warm-wolf-698)
authoring date: 2026-05-13
gate: §1.8 ledger row #62 `substrate_gap_file_ingestion_closed`
roadmap row: T-Workflow-As-Data Class 3 (file-ingestion)
authority docs:
  - docs/r3-program-plan.md §1.8 row #62 (line 290) — DECLARED
  - docs/r3-program-plan.md §2.4 (line 411) — Class 3 sequenced post-T-Lens-Behavioral-Parity (#73 now closed via PR #2797)
  - docs/r3-program-plan.md §4.3 (lines 499-506) — Class 3 closure framing
  - bright-otter-731 audit receipt msg_e85224dc — `rg include_str! dsl` → no matches at HEAD
---

# Gate #62 `substrate_gap_file_ingestion_closed` — substrate-shape canvas

## §0. Status

DECLARED at `docs/r3-program-plan.md:290` (NEW 2026-05-06). T-Lens-Behavioral-Parity precondition cleared 2026-05-13 via PR #2797 (keen-crab-424), unblocking Class 3 dispatch. No worker brief authored before this canvas; bright-otter-731 was auto-spawned and surfaced the substrate-shape question via clean audit. This canvas frames the substrate-shape question for ratification before brief authoring.

## §1. Source authority (verbatim)

### §1.8 row #62 criterion (line 290)

> `.dag` program ingests external file w/o `include_str!`

### §4.3 Class 3 closure framing (line 505)

> **Closes via**: T-Workflow-As-Data file-ingestion substrate (`workflow_substrate_carriers_landed` extended to file-attachment).

### Sequencing (line 411)

> **Class 3 (file-ingestion)**: closes with T-Workflow-As-Data file-ingestion grammar (post-T-Lens-Behavioral-Parity).

## §2. State at HEAD (bright-otter-731 audit)

- `rg include_str! dsl/` → **no matches** (no Rust macro side-channel in dsl/ tree). The literal-text reading of the row criterion ("w/o `include_str!`") is trivially passing if the criterion is absence-shaped.
- `docs/briefs/r3-substrate-t-workflow-as-data-slice-1-worker.md` cites #62 in closure predicate (carrier slice) but is not gate-#62-scoped.
- PR #2819 candidate-shape lands `read_utf8_file` declared in `dsl/std/render.dag` + id-keyed expansion in `lower.rs` + Compiles fixture/tests + INVARIANTS §P5 SG-0 row. This is **compile-time UTF-8 read** plumbing.

## §3. Substrate-shape question — two candidate shapes

### Candidate A — compile-time UTF-8 read (PR #2819 direction)

Shape:
- `extern func read_utf8_file(path: String): String` (or similar arity) declared in `dsl/std/`
- Lower.rs expansion at compile time reads file bytes, embeds as data
- Closes the "no include_str!" criterion by giving the DSL a first-class file-read it can compile to inline data

Pros:
- Concrete, lands quickly, plumbing receipt already drafted
- INVARIANTS §P5 SG-0 row addition is substrate-progress evidence

Cons:
- **Doesn't match §4.3 framing** — §4.3 says "extend `workflow_substrate_carriers_landed` (#53) to file-attachment", which is a **workflow-substrate** extension, not a compile-time compiler intrinsic
- Risk of parallel-authority debt: if the workflow-substrate shape also lands later, gate #62 has two answers
- Compile-time read is a P5-style intrinsic; row's lane is T-Workflow-As-Data (substrate-modeled, runtime/workflow-shaped)

### Candidate B — workflow-modeled file-attachment substrate

Shape:
- Extend `std.workflow` carriers (gate #53 `workflow_substrate_carriers_landed`, partial at PR #2160) with a `FileAttachment` (or similar-shaped) carrier
- DSL programs declare file dependencies as **workflow-substrate values**, not via compile-time reads
- Closes #62 by giving file-ingestion a **workflow-substrate** modeled form

Pros:
- Matches §4.3 framing verbatim ("extend `workflow_substrate_carriers_landed` to file-attachment")
- Sits in the same lane (T-WAD) as the existing carriers (WorkflowSecret, CronExpression)
- Avoids compile-time intrinsic posture — file-ingestion is **modeled** in the workflow, not handled as a compiler side-channel

Cons:
- Larger scope (carrier authoring + lower-stage + consumer evidence)
- T-WAD lane just closed its Wave-1 substrate work (#2747/#2762/#2774/#2798/#2808); reopening for one carrier
- May need a use-case existence proof (concrete `.dag` program ingesting a file) to anchor the carrier shape

### Candidate C — strict-mirror of an existing read-path

Examine if there's an existing read-path precedent in the codebase that gate #62 should mirror (e.g., timing observation set ingestion, build-time asset read). If so, gate-62 closure = adopt that precedent. STOP-and-grep before authoring either A or B.

## §4. Question for ratification

**Which candidate shape closes gate #62?**

- (a) Candidate A — compile-time UTF-8 read via `read_utf8_file` extern (PR #2819 direction)
- (b) Candidate B — workflow-substrate `FileAttachment` carrier extending #53 (§4.3 verbatim framing)
- (c) Candidate C — strict-mirror of existing read-path (specify which precedent)
- (d) Both A + B in distinct slices (A as compile-time bootstrap intrinsic, B as runtime workflow-attachment substrate) — gate #62 closes on B; A is independent compiler-side scope

## §5. Practice 4 (coproduct dissolution) discipline

No new sum-type proposed in this canvas. Candidate A introduces an extern func declaration (no new coproduct). Candidate B extends an existing carrier set (additive to #53). Candidate C mirrors precedent.

## §6. Cost-of-change accounting

Per `INVARIANTS.md` "Cost of Change":

| Candidate | Files edited to add a new file-ingest site |
|---|---|
| A (compile-time read) | 1 (the `.dag` site calls `read_utf8_file(path)`) |
| B (workflow-attachment) | 1 (the `.dag` site declares a `FileAttachment` value) |
| C (mirror precedent) | depends on precedent shape |

Both A + B satisfy the cost-of-change-1 target. The discriminator is **§4.3 framing fidelity** + **lane-fit** (Workflow-As-Data vs compiler intrinsic), not cost-of-change.

## §7. Ratification ask

Director or Substrate-Mgr-tier ratification on §4 question (a/b/c/d). On ratification, brief authoring proceeds:
- If (a): brief frames PR #2819's `read_utf8_file` as gate-62 closure shape; ratify the extern signature + lower expansion
- If (b): brief frames `FileAttachment` carrier extending #53; specify type + data + consumer; carrier-shape question delegated to Director per §3.2-style canvas precedent
- If (c): brief points to the precedent + authorizes strict-mirror landing
- If (d): both briefs author independently; #62 closure pinned to B

PR #2819 currently held in draft per bright-otter-731 disposition path (a) at msg_e5cde2eb. No merge until ratification.

## §8. Reference

- bright-otter-731 audit receipt: msg_e85224dc-7c7e-495d-911f-8bace75a7bcd (preserved as diagnostic-trail per `feedback_redirect_noop_prs` + `feedback_alert_dont_workaround_for_diagnostic_capture`)
- PR #2819 (held in draft) — Candidate A interim plumbing
- `docs/briefs/r3-substrate-t-workflow-as-data-slice-1-worker.md` — cites #62 in closure predicate but is not gate-62-scoped
- §4.3 lines 499-506 — Class 3 closure framing

---

**Authored by**: warm-wolf-698 (R3 Substrate Mgr)
**Date**: 2026-05-13
