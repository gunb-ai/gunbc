---
status: draft (worker brief; cascade-clearance trigger fired post-#1782 merge 2026-05-06; dispatchable per S7 dispatch packet at gunbc#1764 #issuecomment-4385859464)
authority parent: R3 Substrate Manager (#1739)
ratification: cascade-clearance per Substrate canvas B3 + Grounding G1 + R3 design schedule §1 S7
roadmap row: ROADMAP "Substrate carrier port program" PR-F sub-lane
authority docs:
  - docs/r3-design-schedule-2026-05-06.md §1 S7
  - docs/design-emission-model.md §Q2 ("Reference/pointer concepts share a parent")
  - docs/r2-structure.md §Q2 row
  - Substrate canvas B3 (PR-F substrate carrier landing)
  - Grounding G1 (T-Ground-Rust Phase 1 consumer)
gates: unblocks Grounding T-Ground-Rust Phase 1 (`u128` / `isize` / `usize` / walker arms / pilot mirror)
worker pin: loyal-wolf-828 (#1764) — pre-#1782 reservation per Substrate canvas B3
---

# R3 Substrate S7 — PR-F (BoundDeclaration consumer + Rust `ReferenceModel<T>`) worker brief

## Context

S7 lands two carrier-side completions:

1. **BoundDeclaration consumer broadening** — extend the existing
   `BoundDeclaration` substrate (`src/v3/std/substrate.dag`:
   `StaticBound(Interval<Int>) | PlatformDependent`) and its
   partial `BoundDeclarationView` + `match_bound` consumer in
   `src/v3/grounding_coercion_fold/src/fold.rs` to the full
   `design-emission-model.md` example surface. Today's coverage
   wires `ScratchIntExamples` only per module docstring.

2. **Rust `ReferenceModel<T>`** substrate-fact-introduction — per
   `design-emission-model.md` §Q2 ("Reference/pointer concepts
   share a parent") + `r2-structure.md` §Q2. Currently NOT in
   any `.dag` (worker-verified per substrate-state-grep at
   gunbc#1764 #issuecomment-... 2026-05-06: `git grep
   ReferenceModel` hits design authority only, no `src/v3/std/*.dag`
   declarations).

Both completions co-locate per Substrate canvas B3 because they
share the closure predicate: **unblocks Grounding T-Ground-Rust
Phase 1** (`u128` / `isize` / `usize` / walker arms / pilot
mirror). Bundled landing avoids cross-program coordination
overhead during Grounding G1 dispatch.

Worker pre-flight inventory (per gunbc#1764 #issuecomment-...):
- `BoundDeclaration` substrate landed; partial consumer in
  `fold.rs`
- `ReferenceModel<T>` substrate-fact-introduction REQUIRED (P1
  procedure)
- T-E-P-Producer-Broadening (`r3-t-e-p-producer-broadening-worker.md`)
  is the brief-shape precedent

## Slice

### Phase 1 — BoundDeclaration consumer broadening

1. **Extend `BoundDeclarationView`** in
   `src/v3/grounding_coercion_fold/src/fold.rs` (or canonical
   equivalent — worker greps at dispatch) to cover the full
   `design-emission-model.md` example surface, not just
   `ScratchIntExamples`. Per module docstring on `fold.rs`,
   the partial scope was deliberate; this brief lands the full
   surface.

2. **Update `match_bound`** to dispatch on the full set of
   `BoundDeclaration` instantiations relevant to T-Ground-Rust
   Phase 1: `u128`, `isize`, `usize`, plus walker-arm consumers
   per pilot-mirror requirements.

3. **No new `BoundDeclaration` variants** unless absolutely
   required by Grounding G1 consumer. If a new variant is needed,
   STOP and surface — substrate-fact-introduction (P1 procedure)
   per `INVARIANTS.md#p1-modeling-faithfulness`.

### Phase 2 — Rust `ReferenceModel<T>` substrate-fact-introduction

1. **Author `ReferenceModel<T>` carrier** in `dsl/std/` (likely
   `src/v3/std/substrate.dag` adjacent to existing reference-
   adjacent declarations; worker greps for canonical location).
   Per `design-emission-model.md` §Q2 framing: "Reference/pointer
   concepts share a parent" — `ReferenceModel<T>` is the parent
   carrier; specific reference flavors (Rust `&T` / `&mut T` /
   `Box<T>` / `Rc<T>` etc.) are projections.

2. **P1 substrate-fact-introduction procedure** required:
   - DFS the concept DAG before introducing the new carrier (per
     `INVARIANTS.md#p1-modeling-faithfulness` Procedure)
   - Practice 4 classification per
     `docs/modeling-discipline.md#4-coproduct-dissolution`
     ("What to check": *"Any new Rust enum with N ≥ 2 variants
     must have a checkpoint comment naming its classification
     (🟢/🟡/🔴), with a ledger entry if GREEN or a named trigger
     if YELLOW."*)
   - **In-source checkpoint comments** on the live declaration
     (not just PR-body summary)
   - PR body documents P1 receipt + Practice-4 classification
     decision + named consumer demand (Grounding G1)

3. **Phantom-parameter shape vs sum-type shape**: `ReferenceModel<T>`
   is likely 🟢 PRIMITIVE phantom-parameter (specific reference-
   flavor instances are projections, not variants) — but worker
   confirms shape via DFS at dispatch. If sum-type form is
   required, worker authors with named dissolution trigger.

### Phase 3 — Cross-program coordination (Grounding G1)

Cross-program handoff to Grounding Mgr (#1745). G1 (T-Ground-Rust
Phase 1) consumes:
- `BoundDeclaration` per the broadened consumer (Phase 1)
- `ReferenceModel<T>` carrier (Phase 2) for Rust reference / pointer
  emission (`u128` / `isize` / `usize` walker arms; pilot mirror)

Worker documents handoff receipt in PR body. Grounding G1
implementation is downstream; this brief produces the substrate +
consumer infrastructure, Grounding consumes.

## Acceptance

- `BoundDeclarationView` + `match_bound` extended to full
  `design-emission-model.md` example surface; `ScratchIntExamples`
  no longer the partial-coverage limit
- `ReferenceModel<T>` carrier landed in `dsl/std/` (or canonical
  equivalent location) with:
  - Practice 4 in-source checkpoint comment
  - P1 substrate-fact-introduction receipt in PR body (DFS-of-
    concept-DAG; named consumer demand; carrier-shape rationale)
- Cross-program handoff receipt to Grounding Mgr (#1745) for G1
  (T-Ground-Rust Phase 1)
- `cargo test --workspace --exclude v2-compiler-tests` green
  (3 pre-existing v2-compiler --lib failures verified unrelated
  per earlier proud-lynx-311 + valiant-ibex-312 baselines)
- `cargo test -p v2-compiler-tests` green; strict-compile
  diagnostic ratchet at 0
- `cargo clippy --all-targets -- -D warnings` clean
- `cargo fmt --all --check` clean
- Citation discipline per `docs/briefs/brief-authoring-checklist.md`
  Citation discipline section: section anchors / rule-text quotes
  only; no bare `:NNN`
- 5-question authority audit in PR body

## STOP-AND-ESCALATE

- **Phase-1 BoundDeclaration consumer extension requires a new
  `BoundDeclaration` variant** beyond current `StaticBound /
  PlatformDependent`: STOP — substrate-fact-introduction (P1).
  Surface to Substrate Mgr (#1739) with proposed new variant +
  named consumer demand + DFS-of-concept-DAG receipt.
- **Phase-2 `ReferenceModel<T>` shape DFS reveals existing
  carrier**: re-frame as consumer migration not landing. Worker
  greps `dsl/std/` thoroughly before authoring.
- **`ReferenceModel<T>` requires sum-type form** (specific
  reference flavors as variants rather than phantom-parameter
  projections): named dissolution trigger required for SCAFFOLD
  classification per `feedback_construction_over_ratchets`. Do
  NOT silently grow without consumer demand naming.
- **Grounding G1 requires substrate beyond what S7 lands**: STOP
  — surface to Substrate Mgr; cross-program coordination boundary.
- **T-E-P-Producer-Broadening (S10 / quick-koi-190 #1763)
  introduces a substrate edit that conflicts with S7 Phase 1**:
  parallel lane; cross-reference at brief landing, not blocking.
  If conflict surfaces, coordinate via Substrate Mgr.

## Authority audit receipt

1. **Substrate exists?** Per loyal-wolf-828's pre-flight grep
   at gunbc#1764 #issuecomment-... 2026-05-06:
   - `BoundDeclaration` exists in `src/v3/std/substrate.dag`
     (StaticBound + PlatformDependent variants)
   - Partial `BoundDeclarationView` + `match_bound` consumer in
     `src/v3/grounding_coercion_fold/src/fold.rs`
   - `ReferenceModel<T>` does NOT exist in `dsl/std/` —
     substrate-fact-introduction required
   Worker re-greps at dispatch.
2. **Existing brief?** None for S7 specifically. T-E-P-Producer-
   Broadening (`r3-t-e-p-producer-broadening-worker.md`) is the
   brief-shape precedent per design schedule §1 S7.
3. **Design-doc match?** `design-emission-model.md` §Q2 +
   `r2-structure.md` §Q2 are the design surface. Worker re-reads
   each cited section at dispatch.
4. **Citations live?** Worker verifies cited authorities at HEAD
   before authoring; per loyal-wolf-828 pre-flight, all four are
   live as of 2026-05-06.
5. **Carrier dissolves the bridge?** Yes — the bridge is
   T-Ground-Rust Phase 1 needing full `BoundDeclaration` consumer
   coverage + `ReferenceModel<T>` substrate. Both completions
   co-locate per canvas B3. Cementing: Grounding G1 emission
   rules consume the new substrate (downstream PR; not in S7
   scope).

## Provenance

Drafted 2026-05-06 post-#1782 merge per cascade-clearance
trigger ratified at canvas B3. Worker pin loyal-wolf-828
(#1764). Dispatch packet at gunbc#1764 #issuecomment-4385859464
named the awaiting-brief posture; this brief satisfies that
prerequisite.

Cross-references S10 (T-E-P-Producer-Broadening) parallel-
dispatch to quick-koi-190 (#1763) — no hard dependency; both
are #1782-merge-cleared substrate-foundational lanes.
