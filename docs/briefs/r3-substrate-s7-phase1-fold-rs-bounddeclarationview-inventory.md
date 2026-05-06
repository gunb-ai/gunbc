# S7 Phase 1 — `BoundDeclarationView` / `match_bound` consumer inventory (`fold.rs`)

**Purpose:** Forcing-function artifact (Director ratified). Catalog of the
`BoundDeclarationView` + `match_bound` consumer surface in
`src/v3/grounding_coercion_fold/src/fold.rs` before Slice 2.5 implementation.

**Authority:** `docs/briefs/r3-substrate-s7-pr-f-bounddeclaration-consumer-worker.md`
*(canonical S7 worker brief — **tracked on `main`** at this path; this inventory
PR does not add or rename that file; STOP / Phase boundaries defer to it.)*

**Merge-base audit (not in this PR diff — auditable on `main`):** That path is
present on `origin/main` as git blob **`ac843c5817358785979409dc30deaba94fb28de5`**
(path cited above; introduced **`7153efb51`** “R3 Substrate”, **#1825**).
Mechanical check before merge:  
`git cat-file -t ac843c5817358785979409dc30deaba94fb28de5` → `blob`, or  
`git show origin/main:docs/briefs/r3-substrate-s7-pr-f-bounddeclaration-consumer-worker.md | head -n 5`.

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

## 2. Orthogonal tracks — interval widening vs platform-kind (do not conflate)

Phase-1 consumer work spans **two substrate gates** that must stay separate in
dispatch planning:

**Track A — Static intervals / numeric width (`StaticBound`, `Interval<Int>`):**
Scratch encoding uses `Interval<i64>` today; broader **i128 / u128** program
ranges and matching **`StaticBoundFact`** rows require widening interval
representation and declared `TargetIntegerTypeInhabitance` facts — still within
the **`StaticBound(Interval<Int>)`** substrate kind.

**ROADMAP Pattern D — STOP+PING on bare u128 width work:** Per `ROADMAP.md`
§ reflective integration patterns (**Pattern D — numeric work changed philosophy
mid-window** / T-Numeric-Construction reframe): the next slice must **not** add
**u128** (nor UInt128 / fixed-width chain expansion) **merely as another table
row** unless explicitly tied to the **`Int<N>` / `Nat<N>`** refinement path;
**STOP+PING at brief-finalization** for bare width-row additions; owner R3
Substrate (T-Numeric-Construction). This fold consumer **follows** substrate +
numeric lane receipts — it does not bypass Pattern D upstream.

**Track B — Platform-dependent kinds (`PlatformDependent`, kind-only match):**
Targets such as Rust **`usize`** (and Go **`int`**) carry **platform-sized /
kind-only** bounds per `design-emission-model.md` — matched via the outer
`BoundDeclaration` sum (`PlatformDependent`), **not** by pretending their
semantics are “wider `Interval<Int>` literals.” **`match_bound`** already has a
**kind-only** arm for `BoundDeclarationView::PlatformDependent` (`fold.rs`
`L118`).

**Not wider `StaticBoundFact` (Q1):** Under **`design-emission-model.md` Q1**
consolidation, **`PlatformDependent` is a distinct outer kind**, not an
`Interval<Int>` edge case. **`usize` / architecture-dependent rows are not**
“broader `TargetIntegerInhabitanceBoundView::StaticBoundFact` intervals.” Encoding
them that way would **split substrate authority** (same failure mode Q1 calls out
for “`BoundDeclaration = Interval<Int>` only”). **P1 modeling faithfulness:**
extend the **target inhabitance carrier** (`TargetIntegerTypeInhabitance` **`bound:`**
field + **`parse_target_integer_inhabitance_bound`**) so platform targets **name
`PlatformDependent` structurally** — **STOP** if consumer work invents interval
width where substrate requires the platform-kind gate.

**Explicit gate:** Phase 1 Track B work lands **substrate + row facts** first,
then consumers — **never** merge Track B into Track A as “interval widening.”

**`isize` boundary:** Classify per declared inhabitance rows — **`isize`** is **not**
automatically Track B like **`usize`**; substrate may use **`StaticBound` exact
intervals** for a fixed pointer-sized signed range **or** other declared shapes.
Do **not** lump **isize** with Track A u128 widening **or** Track B usize matching
without reading each row’s **`BoundDeclaration`** kind.

---

## 3. Per-call-site narrative (broadening vs `design-emission-model.md` example surface)

**Module preamble (`fold.rs` lines 1–14):** States that worked examples from
`docs/design-emission-model.md` are behavioral targets, but only Examples **1,
2, 5, 6, and 8** run on the `ScratchIntExamples` checkpoint path; others stay
`FoldNotImplemented`. **Broadening needed:** T-Ground-Rust Phase 1 / pilot-mirror
requires moving beyond `ScratchIntExamples`. **Track A:** walker / declared
inhabitance for wider **static** intervals (**i128 / u128** and Pattern D–aligned
substrate). **Track B:** **usize** / platform-kind rows and matching rules.
**isize:** per-row classification. All must eventually use the same
`BoundDeclarationView` / `match_bound` predicate (**no parallel emission
predicate**), but implementation tasks stay split by track above.

**`BoundDeclarationView` enum (`L41–44`):** Today:
`StaticBound(Interval<i64>) | PlatformDependent` (platform variant currently
`#[allow(dead_code)]`). Substrate `BoundDeclaration` is
`StaticBound(Interval<Int>) | PlatformDependent` per design authority.
**Track A broadening:** align the view with full **`Interval<Int>`** program
facts for wider **static** ranges (i128/u128 path; Pattern D upstream). **Track
B broadening:** exercise **`PlatformDependent`** on the program side where design
authority requires — distinct from interval widening. **Pilot-mirror:** tests/helpers
that synthesize `Interval<i64>` (`L513`, `L545`) must grow with Phase 1’s
canonical interval representation for Track A.

**`ProgramIntegerIntent.bound` (`L72`):** Carries program-side bound into
selection. **Broadening:** Intent extraction from real program DAG (Slice C /
#1133 / #1286 track) must populate this field for all Phase-1 examples — not
fixed scratch closures.

**`design_doc_example_8_program_bound` (`L86–93`):** Hard-codes Example 8’s
i32 exact interval as `StaticBound`. **Broadening:** Representative of “exact
static interval” programs (**Track A**); generalize to wider **static** ranges per
Pattern D–aligned substrate. **Track B** examples use **`PlatformDependent`**
where the design doc calls for platform-bound targets — not the same extension as
wider `Interval<Int>` literals.

**`match_bound` (`L95–119`):** Single structural predicate vs
`TargetIntegerInhabitanceBoundView` (`BoundUnspecified`, `StaticBoundFact`).
Implements design-doc rules: target `Unbounded` accepts any program `StaticBound`;
exact interval equality; `BoundUnspecified` and `PlatformDependent` mismatch
kinds. **Broadening:** Ensure **interval equality** and **kind** dispatch remain
correct when `StaticBoundFact` carries intervals outside signed-64 scratch
encoding (**Track A**: wider `StaticBoundFact` intervals). **Track B:** do not
collapse platform-kind targets into “wider intervals” — use substrate rows +
kind dispatch. **Walker arms:** Every new target row shape that participates in
Phase 1 must still funnel through this predicate or an explicit extension beside
it (no parallel emission predicate).

**`exact_static_bound_match` (`L122–132`):** Refinement for disambiguation after
`match_bound` filtering. **Broadening:** Same interval-type alignment as
`match_bound`.

**`select_example_8_declared_inhabitance` / `select_declared_inhabitance`
(`L318–368`, filter at `L346`):** **Primary consumer** — filters declared
`TargetIntegerTypeInhabitance` DAG rows using `match_bound`. **Broadening:** Row
set must include **Pattern D–aligned u128/i128 static** inhabitants (**Track A**),
**platform-kind** rows for **usize** / related targets (**Track B**), and
correctly shaped rows for **isize** per substrate — when DAG and bootstrap data
expand; otherwise Phase 1 examples cannot select.

**`example_8_program_intent` (`L396–420`):** Wires kernel/algebra/realization
names per scratch target language. **Broadening:** Additional arms or generalized
lookup when walker exercises non–Example-8 kernels.

**`design_doc_example_2_program_bound` + `fold_design_doc_example_2_semiring_u32`
(`L429–458`):** Example 2 Semiring u32 bound. **Broadening:** Confirms Semiring
lane uses same bound machinery; **u128** / wider Semiring inhabitants follow
**Pattern D–aligned** substrate rows once declared (not ad-hoc width expansion).

**`fold_design_doc_example_8_*` (`L470–492`):** Per-target Example 8 entry.
**Broadening:** Parallel **walker/pilot** paths for **Track A / Track B / isize**
targets should call the same selection stack (`select_example_8_declared_inhabitance`
pattern) once corresponding `TargetIntegerTypeInhabitance` rows exist.

**Test hooks (`L494–552`):** `fold_design_doc_example_8_for_testing`,
`select_program_integer_intent_for_testing` — construct
`BoundDeclarationView::StaticBound(program_bound)` from `Interval<i64>`.
**Broadening:** Test fixtures may need wider interval types or DAG-driven bounds
to mirror production extraction.

---

## 4. Per-site STOP triggers (new `BoundDeclaration` substrate variant)

Per S7 brief **STOP-AND-ESCALATE**: if Phase 1 requires a **new `BoundDeclaration`
variant** beyond **`StaticBound` + `PlatformDependent`**, stop — P1
substrate-fact-introduction + Substrate Mgr (#1739).

| Location | Trigger |
|----------|---------|
| **`BoundDeclarationView` (`L41–44`)** | Adding a third view arm because substrate gained a new `BoundDeclaration` variant — **STOP** (confirm substrate change + P1 before editing consumer). |
| **`match_bound` (`L95–119`)** | New substrate variant implies new match arms and likely `TargetIntegerInhabitanceBoundView` / DAG parse changes — **STOP** unless brief explicitly permits and substrate is already landed. |
| **`parse_target_integer_inhabitance_bound` / row parsing (`L250–307` region)** | If inhabitance rows gain a new `bound:` shape — coordinate with substrate; may be STOP if it implies new `BoundDeclaration` variant. |
| **`select_declared_inhabitance` (`L327–368`)** | Symptom surface for “no matching row” after substrate expansion — diagnose substrate vs consumer gap before ad-hoc variants. |
| **Pattern D / u128–width substrate** | Adding UInt128/u128 **width** without **Int<N>/Nat<N>** tie-in — **STOP+PING** per `ROADMAP.md` Pattern D; resolve in **T-Numeric-Construction** briefs before treating as fold-consumer-only work. |

If broadening can be done **without** new substrate variants (e.g., wider
`Interval<Int>` payloads inside existing `StaticBound`, more rows in DAG), **no
STOP** — proceed within Phase 1 consumer work.

---

## 5. Brief reference

- Worker brief: `docs/briefs/r3-substrate-s7-pr-f-bounddeclaration-consumer-worker.md`
  *(canonical path on `main`; STOP-AND-ESCALATE section authoritative).*
- Design emission examples / `BoundDeclaration` predicate: `docs/design-emission-model.md` (bound-matching sections; Rust/Python/Go triple walk)
- ROADMAP Pattern D (numeric / u128 discipline): `ROADMAP.md` — § reflective integration patterns, **Pattern D**
