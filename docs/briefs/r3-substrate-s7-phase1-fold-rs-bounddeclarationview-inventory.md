# S7 Phase 1 — `BoundDeclarationView` / `match_bound` consumer inventory (`fold.rs`)

**Purpose:** Forcing-function artifact (Director ratified). Catalog of the
`BoundDeclarationView` + `match_bound` consumer surface in
`src/v3/grounding_coercion_fold/src/fold.rs` before Slice 2.5 implementation.
**Authority:** `docs/briefs/r3-substrate-s7-pr-f-bounddeclaration-consumer-worker.md`.

This file mirrors the PR-body inventory for traceability in-tree.

---

## 1. Grep catalog

Command:

```bash
git grep "BoundDeclarationView\|match_bound" src/v3/grounding_coercion_fold/
```

Output at inventory time:

```
src/v3/grounding_coercion_fold/src/fold.rs:41:enum BoundDeclarationView {
src/v3/grounding_coercion_fold/src/fold.rs:72:    bound: BoundDeclarationView,
src/v3/grounding_coercion_fold/src/fold.rs:86:fn design_doc_example_8_program_bound() -> BoundDeclarationView {
src/v3/grounding_coercion_fold/src/fold.rs:87:    BoundDeclarationView::StaticBound(Interval::BoundedInterval {
src/v3/grounding_coercion_fold/src/fold.rs:95:fn match_bound(
src/v3/grounding_coercion_fold/src/fold.rs:96:    program: &BoundDeclarationView,
src/v3/grounding_coercion_fold/src/fold.rs:101:            BoundDeclarationView::StaticBound(_),
src/v3/grounding_coercion_fold/src/fold.rs:105:            BoundDeclarationView::StaticBound(program_interval),
src/v3/grounding_coercion_fold/src/fold.rs:115:            BoundDeclarationView::StaticBound(_),
src/v3/grounding_coercion_fold/src/fold.rs:118:        (BoundDeclarationView::PlatformDependent, _) => BoundMatch::DiffersKind,
src/v3/grounding_coercion_fold/src/fold.rs:123:    program: &BoundDeclarationView,
src/v3/grounding_coercion_fold/src/fold.rs:129:            BoundDeclarationView::StaticBound(program_interval),
src/v3/grounding_coercion_fold/src/fold.rs:321:    program_bound: &BoundDeclarationView,
src/v3/grounding_coercion_fold/src/fold.rs:346:        .filter(|row| match_bound(&intent.bound, &row.bound) == BoundMatch::Matches)
src/v3/grounding_coercion_fold/src/fold.rs:399:    bound: BoundDeclarationView,
src/v3/grounding_coercion_fold/src/fold.rs:429:fn design_doc_example_2_program_bound() -> BoundDeclarationView {
src/v3/grounding_coercion_fold/src/fold.rs:430:    BoundDeclarationView::StaticBound(Interval::BoundedInterval {
src/v3/grounding_coercion_fold/src/fold.rs:513:        &BoundDeclarationView::StaticBound(program_bound),
src/v3/grounding_coercion_fold/src/fold.rs:545:        bound: BoundDeclarationView::StaticBound(program_bound),
```

No other matches under `src/v3/grounding_coercion_fold/` at this grep.

---

## 2. Per-call-site narrative (broadening vs `design-emission-model.md` example surface)

**Module preamble (`fold.rs` lines 1–14):** States that worked examples from
`docs/design-emission-model.md` are behavioral targets, but only Examples **1,
2, 5, 6, and 8** run on the `ScratchIntExamples` checkpoint path; others stay
`FoldNotImplemented`. **Broadening needed:** T-Ground-Rust Phase 1 / pilot-mirror
requires moving beyond `ScratchIntExamples` — **u128**, **isize**, **usize**
walker arms and declared inhabitance matching must eventually consume the same
`BoundDeclarationView` / `match_bound` predicate as the full emission surface,
not only scratch-int drivers.

**`BoundDeclarationView` enum (`L41–44`):** Today:
`StaticBound(Interval<i64>) | PlatformDependent` (platform variant currently
`#[allow(dead_code)]`). Substrate `BoundDeclaration` is
`StaticBound(Interval<Int>) | PlatformDependent` per design authority.
**Broadening:** View must align with **full** `Interval<Int>`-carried program
facts and target inhabitance rows for **Rust i128/u128/isize/usize** (and any
other Phase-1 kernel integers), not only `i64` scratch literals. **Pilot-mirror:**
tests/helpers that synthesize `Interval<i64>` (`L513`, `L545`) must grow with
whatever canonical interval representation Phase 1 uses for wider primitives.

**`ProgramIntegerIntent.bound` (`L72`):** Carries program-side bound into
selection. **Broadening:** Intent extraction from real program DAG (Slice C /
#1133 / #1286 track) must populate this field for all Phase-1 examples — not
fixed scratch closures.

**`design_doc_example_8_program_bound` (`L86–93`):** Hard-codes Example 8’s
i32 exact interval as `StaticBound`. **Broadening:** Representative of “exact
static interval” programs; same pattern must generalize to other declared ranges
(u128/isize/usize) once facts are wired.

**`match_bound` (`L95–119`):** Single structural predicate vs
`TargetIntegerInhabitanceBoundView` (`BoundUnspecified`, `StaticBoundFact`).
Implements design-doc rules: target `Unbounded` accepts any program `StaticBound`;
exact interval equality; `BoundUnspecified` and `PlatformDependent` mismatch
kinds. **Broadening:** Ensure **interval equality** and **kind** dispatch remain
correct when `StaticBoundFact` carries intervals outside signed-64 scratch
encoding (wider literals, usize platform rows). **Walker arms:** Every new
target row shape that participates in Phase 1 must still funnel through this
predicate or an explicit extension beside it (no parallel emission predicate).

**`exact_static_bound_match` (`L122–132`):** Refinement for disambiguation after
`match_bound` filtering. **Broadening:** Same interval-type alignment as
`match_bound`.

**`select_example_8_declared_inhabitance` / `select_declared_inhabitance`
(`L318–368`, filter at `L346`):** **Primary consumer** — filters declared
`TargetIntegerTypeInhabitance` DAG rows using `match_bound`. **Broadening:** Row
set must include **u128 / isize / usize** (and related) inhabitations when DAG
and scratch/bootstrap data expand; otherwise Phase 1 examples cannot select.

**`example_8_program_intent` (`L396–420`):** Wires kernel/algebra/realization
names per scratch target language. **Broadening:** Additional arms or generalized
lookup when walker exercises non–Example-8 kernels.

**`design_doc_example_2_program_bound` + `fold_design_doc_example_2_semiring_u32`
(`L429–458`):** Example 2 Semiring u32 bound. **Broadening:** Confirms Semiring
lane uses same bound machinery; **u128** / other Semiring inhabitants must follow
once declared rows exist.

**`fold_design_doc_example_8_*` (`L470–492`):** Per-target Example 8 entry.
**Broadening:** Parallel **walker/pilot** paths for isize/u128/usize should call
the same selection stack (`select_example_8_declared_inhabitance` pattern).

**Test hooks (`L494–552`):** `fold_design_doc_example_8_for_testing`,
`select_program_integer_intent_for_testing` — construct
`BoundDeclarationView::StaticBound(program_bound)` from `Interval<i64>`.
**Broadening:** Test fixtures may need wider interval types or DAG-driven bounds
to mirror production extraction.

---

## 3. Per-site STOP triggers (new `BoundDeclaration` substrate variant)

Per S7 brief **STOP-AND-ESCALATE**: if Phase 1 requires a **new `BoundDeclaration`
variant** beyond **`StaticBound` + `PlatformDependent`**, stop — P1
substrate-fact-introduction + Substrate Mgr (#1739).

| Location | Trigger |
|----------|---------|
| **`BoundDeclarationView` (`L41–44`)** | Adding a third view arm because substrate gained a new `BoundDeclaration` variant — **STOP** (confirm substrate change + P1 before editing consumer). |
| **`match_bound` (`L95–119`)** | New substrate variant implies new match arms and likely `TargetIntegerInhabitanceBoundView` / DAG parse changes — **STOP** unless brief explicitly permits and substrate is already landed. |
| **`parse_target_integer_inhabitance_bound` / row parsing (`L250–307` region)** | If inhabitance rows gain a new `bound:` shape — coordinate with substrate; may be STOP if it implies new `BoundDeclaration` variant. |
| **`select_declared_inhabitance` (`L327–368`)** | Symptom surface for “no matching row” after substrate expansion — diagnose substrate vs consumer gap before ad-hoc variants. |

If broadening can be done **without** new substrate variants (e.g., wider
`Interval<Int>` payloads inside existing `StaticBound`, more rows in DAG), **no
STOP** — proceed within Phase 1 consumer work.

---

## 4. Brief reference

- Worker brief: `docs/briefs/r3-substrate-s7-pr-f-bounddeclaration-consumer-worker.md`
- Design emission examples / `BoundDeclaration` predicate: `docs/design-emission-model.md` (bound-matching sections; Rust/Python/Go triple walk)
