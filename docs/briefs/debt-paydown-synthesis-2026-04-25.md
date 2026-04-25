# Debt-Paydown + Scaffolding-Audit Synthesis (2026-04-25)

> **Status:** synthesis doc, not implementation. Authored from session
> `keen-wren-319` per ad-hoc Director dispatch. Inputs: three independent
> 2026-04-25 analyses (Reflective `gpt-5-5-pro@7f74f09`, Exploratory
> `gpt-5-4-pro@a8f0825`, Exploratory `gpt-5-5-pro@7f74f09`); PR #809
> (post-merge debt rows from those analyses, merged 2026-04-25 mid-PR);
> live ROADMAP §"Tracked debts"; live cited scaffold files. Output:
> course-of-action recommendation for Director on how to plan debt paydown
> alongside R2 + Pure Bootstrap to Zero.

> **Note on PR #809.** Merged 2026-04-25T18:13:39Z; line citations to
> `ROADMAP.md §"Tracked debts" → "Post-merge debt (2026-04-25 reflective
> + exploratory analyses)"` are authoritative. (At initial synthesis
> authoring time #809 was still open; this note retained for historical
> traceability.)

---

## 0. Headline finding (read this first)

**The dominant scaffold pattern in the 2026-04-25 analyses is a single
class, not eight independent items: string/path/name identity bridges.**

Per acceptance §STOP-AND-ESCALATE bullet 4 ("class-of-debt that should be
addressed structurally rather than item-by-item"), this is surfaced for
Director **as the central recommendation, not eight item-by-item
paydowns**. Concrete instances all sharing one missing-substrate-fact
("identity / role references not yet carried structurally enough for
consumers"):

1. `PROGRAM_INPUT_SENTINEL = "r1_lens_output_input_from_program"` — five
   sites in `src/v3/compiler/src/test_runner.rs` (`:1594, 1617, 1642,
   1709, 1855`).
2. Fixture-filename → bind-name routing in `test_runner.rs:47-48`.
3. `include_str!("../lenses/named_function_count.dag")` as canonical lens
   side-channel (`test_runner.rs:23`, with `:33` a sibling for complexity).
4. `span.file.as_str().ends_with("std/algebra.dag")` fold-path skip in
   `lens_apply.rs:38`.
5. `span.file == "dsl/std/types.dag"` type-alias refinement bridge in
   `lower.rs:836`.
6. `bind.span.file == "named_alias_emit_helper.v3"` /
   `branch.span.file == "match_emit_helper.v3"` emission special cases
   (`emit.rs:3181, 3206`).
7. `declaration_name_preference_rank(&span.file)` mirrored in
   `dag.rs:2735-2764` and `lower.rs:1451-1452, 1546-1547`.
8. `EXTDEPS_BOOTSTRAP_FIXTURES` manual fixture listing
   (per #809 entry; `bootstrap.rs::std_fixtures()`).

Every one has its own dissolution comment. Together they say: lowering
does not yet carry structural identity (`DeclarationRef`, explicit
input-value carriers, structural fold-step edges, structural emit-helper
carriers) end-to-end, so consumers reach for `span.file` /
fixture-filename / sentinel-string as the temporary identity. Eight
item-by-item paydowns will fight this every quarter; one substrate pass
on identity carriers dissolves the class.

**Director ask: treat #4 in the prioritization below ("Identity-carrier
substrate pass") as one M-scope program, not as eight unrelated cleanups
distributed across PB-* lanes.**

The non-class P3 fail-closed leaks (Go `UnknownVariant`,
`lower_fn_body_into_existing_decl` defensive Arrow re-derive, lens fold
ambiguous unique-candidate fallback) ARE genuinely independent and
prioritized separately below.

---

## 1. Scaffolding inventory (recent-window scaffolds)

Each row: scaffold · file:line cite · dissolution trigger present? ·
where tracked. "Recent-window" = scaffold landed or materially expanded
since 2026-04-18.

### 1.1 Test-runner harness (`src/v3/compiler/src/test_runner.rs`, 3028 LOC)

| Scaffold | File:line | Trigger present? | Tracked at |
|---|---|---|---|
| `PROGRAM_INPUT_SENTINEL` (5 sites) | `:1594, 1617, 1642, 1709, 1855` | ✅ — comment at `:1594-1620` names `DeclarationRef` substrate work | PR #809 "Filename / sentinel bridges" |
| Fixture-filename → bind-name routing | `:47-48` | ✅ — same row | PR #809 |
| `include_str!` of canonical lens | `:23-43` | ✅ — `:20-22` names `DeclarationRef` substrate dependency | PR #809 |
| `LensOutputEquals` lens-name `Some("named_function_count")` parallel-authority arm | `:1683-1716, :1763-1766` | partial — folded into `DeclarationRef` migration; comment at `:1611-1618` names the upstream fix | PR #809 |

All have triggers. Class belongs in §0 identity-carrier pass.

### 1.2 Lens evaluation harness (`src/v3/compiler/src/lens_apply.rs`, 1141 LOC)

| Scaffold | File:line | Trigger present? | Tracked at |
|---|---|---|---|
| `span.file ends_with "std/algebra.dag"` fold-skip | `:22-39, :372-383` | ✅ — comment names structural fold-shape carrier | PR #809 "Lens fold execution" |
| `find_fold_step_bind_via_instantiation` ambiguous unique-candidate fallback | `:105-148` | ✅ — same row, requires template-formal-edge structural identity | PR #809 (P1+P2) |
| Lossy `Behavior → FieldValue` reflection (Transform/Branch/Loop/Bind drop fields) | `:859-907` | ✅ — generate from `src/v3/std/substrate.dag` or consume canonical .dag values | PR #809 "Lossy user-lens reflection" |

Triggers all named. **Classification (matches §0 + Tier 0 #3):** row 1
(file-suffix fold-skip) participates in the §0 identity class —
dissolves with the structural fold-shape carrier. Row 2 (ambiguous
unique-candidate fallback) is an independent **P3 fail-closed leak**
prioritized at **Tier 0 #3** for direct dispatch (B3); it is NOT in
the §0 class because the fix is "remove the heuristic and require the
structural template-formal edge to identify the callable" — a local
fail-closed correction, not the substrate-wide identity-carrier
program. Row 3 (lossy reflection) is the structural-substrate-
reflection class (related to but distinct from §0).

### 1.3 Grounding (`src/v3/grounding_pilot/src/lib.rs` 584 LOC + `src/v3/grounding_engine/src/lib.rs` 557 LOC)

| Scaffold | File:line | Trigger present? | Tracked at |
|---|---|---|---|
| Rust constants mirroring `dsl/extdeps/languages/rust/primitives.dag` rows | `grounding_pilot/src/lib.rs` (entire file) | ✅ — Reflective Analysis A.3 names `ValueBody::List` / aggregate-body work as the trigger | ROADMAP §"Class 5 Gap 3" (`:365`); R2 Grounding Manager `crisp-seal-366` |
| Mirror-consistency checks instead of consuming row data | `grounding_engine/src/lib.rs` | ✅ — same trigger | same |

Triggers belong to substrate-capability lane (top-level `ValueBody::List`
worker brief #790 + bootstrap/load-set decision). **Cross-manager
coordination needed with R2 Grounding Manager.**

### 1.4 Generated-source patching (`src/v3/compiler/src/lib.rs:1143-1180`)

| Scaffold | File:line | Trigger present? | Tracked at |
|---|---|---|---|
| `patch_lower_helpers_generated_type_alias_refinement` exact-string rustfmt patching | `lib.rs:1143-1180` | ✅ — "first PB cleanup target once generated `lower_helpers` can emit the refinement field natively" | PR #809 (P5 Progress Is Dissolution); T-PB-A class |

Trigger present; class belongs to **Zero-Floor Manager (`stern-swift-335`)
PB-* lane**.

### 1.5 Host-Rust mirrors of std `.dag` carriers (`src/v3/compiler/src/dag.rs`)

| Scaffold | File:line | Trigger present? | Tracked at |
|---|---|---|---|
| Termination mirror (`DescentEvidence`, `PositiveDescentAmount`, `ProportionalDivisor`, `ShrinkFactor`, `evidence_rank`, `merge_evidence`) | `dag.rs:628-790` | ✅ — explicit `🟡 SCAFFOLD` markers + parity ratchets named at `:628-636` | PR #809 "src/v3/std vs dsl/std checklist undercount"; ROADMAP `:368` |
| Computation mirror (`SizeBound`, `RecursionShape`-related) | `dag.rs:839-915` | ✅ — `:840` names same staging contract as termination | same |
| Induction mirror (`RecursionShape`, `InductiveField`, `SubValueRelation`) | `dag.rs:916-980` | ✅ — `:919, :949` "🟡 SCAFFOLD. The .dag type remains the authority" | same |
| `kernel_algebra_profile` 7-variant mirror | `dag.rs:1383-1432` | ✅ — `:1426-1431` names parity ratchet test | T-Substrate carrier-port program (ROADMAP `:381`) |

All triggered + ratcheted. **Class:** these dissolve via the Zero-Floor
program substrate-evaluation surface (i.e., when v3 can lower & evaluate
.dag values into the runtime). Belongs to PB-Tier1-Sweep priority
ordering (Zero-Floor Manager).

### 1.6 Effect-carrier Rust mirror

| Scaffold | File:line | Trigger present? | Tracked at |
|---|---|---|---|
| `src/v3/compiler/src/dag/effects.rs` mirroring `src/v3/std/effects.dag` carriers | `dag/effects.rs` (216 LOC) | ✅ — broad PB-A dissolution; closed-system effects model already in std (memory `feedback_closed_system_effects`) | ROADMAP `:340` "effects.dag dual authority"; T-PB-A |
| `compose_operation_effects` Rust copy of `compose_effects` | `workflow_idempotency.rs` (105 LOC) | ✅ — same | same |

The user explicitly flagged effects in this synthesis dispatch: **the
model is right, the debt is the Rust mirror**. Trigger present. Belongs
to PB-* lane.

### 1.7 Other recent-window scaffolds with triggers (no novel inventory needed)

Summary citations to existing tracking:

- File-preference rank (`dag.rs:2735-2764` + `lower.rs:119-127`) — ROADMAP `:368`. Trigger: convergence of every duplicated `src/v3/std/` ↔ `dsl/std/` module. **PR #809 row catches checklist undercount (computation + induction + termination not in checklist).**
- Loop emission semantic invariant (Python/Go) — PR #809 row. **Risk-shaped:** comment-level invariant, not structural.
- `bootstrap.rs::patch_kernel_bool_boolean_algebra_inhabits` Rust patching (`bootstrap.rs:211-291`) — ROADMAP `:364` (Class 5 Gap 1). Trigger: structural `inhabits` edge.
- `EXTDEPS_BOOTSTRAP_FIXTURES` manual list — ROADMAP `:365` (Class 5 Gap 3 `std.unicode` bootstrap/load-set decision is the parent).

### 1.8 Inventory verdict

**Every recent-window scaffold has a named dissolution trigger.** The
STOP-AND-ESCALATE bullet 2 ("scaffold without named dissolution
trigger") does NOT fire. PR #809's debt rows + existing ROADMAP entries
cover every scaffold I inspected.

The two qualitative concerns:

- **Trigger-points-at-its-own-cause risk (⚠️ POTENTIALLY FUNDAMENTAL —
  borderline, not stopping):** the §0 identity-carrier class triggers all
  point at "structural `DeclarationRef` / structural fold-step edge / etc.
  in lowering" — which is not yet a single named program. This is the
  pattern user described as "trigger names work that IS the scaffold's
  upstream." Borderline because the upstream substrate work is
  identifiable (substrate-carrier-port + T-PB-A + ValueBody-extension);
  it just isn't named as ONE program. Director should consider whether
  the §0 class warrants its own program-level brief (M scope) parallel
  to PB-Tier1-Sweep.
- **Checklist drift:** PR #809 catches one instance (file-preference
  rank checklist undercount). The pattern likely recurs across other
  ratcheted scaffolds. Worth a sweep; not synthesis-doc scope.

---

## 2. Velocity audit (introduction vs dissolution, 7-day window)

Window: 2026-04-18T00:00 → 2026-04-25T23:59 (commit dates).

| Metric | Count |
|---|---|
| Total first-parent commits | 260 |
| Merge commits (~PRs landed on main) | 54 |
| Dissolution-shaped PRs (titles match retire/dissolve/delete/remove) | **8** |
| Substantial scaffold-introducing PRs (estimate) | **~13** |

**Ratio: ~1.6:1 introduction : dissolution.**

> **Calibration footnote.** This count uses a PR-title heuristic
> (`retire`/`dissolve`/`delete`/`remove`/`collapse`). It **undercounts
> dissolution work that ships *inside* feature PRs** — e.g., a T-LensAPI
> PR that adds a lens and retires three legacy paths in the same diff.
> The 1.6:1 reading is therefore a lower-bound on dissolution velocity.
> The §4 tripwire mechanism reflects this: before a ≥3:1 reading
> triggers Director review, the cadence pass does a manual sweep for
> dissolution-bearing feature PRs to avoid false positives.

### Dissolution-shaped PRs in window (verified)

`#787` PB-1-e retire `bootstrap_all_runtime` · `#729` retire lower
pass-through scaffold · `#730` retire infer payload span shim · `#724`
remove `lens_parallelism` compatibility alias · `#715` retire
`lens_parallelism` wrapper · `#699` retire `lens_idempotency` wrapper ·
`#664` compiler→std tranche 2 (delete `runtime_mirrors.dag`) · `#673`
stale-receipt sweep.

### Introduction-shaped PRs in window (sample)

`#741` T-LensAPI D1+D2+D3+D4 (lens_apply 1141 LOC) · `#722`
MockBackedInvariant wiring · `#717` LensOutputEquals dispatch · `#728`
AlgebraicLaw runner · `#740` AlgebraicLaw associativity recognizer ·
`#764` R1 Lane A (LaneE gates + mock-backed runner + T-Demo lens) ·
`#765` T-Ground Pilot (grounding_pilot 584 LOC) · `#788` Engine Phase 1
Typestructure · `#784` PB-Substrate AtomPayload · `#780` PB-Substrate
pilot v2 · `#742` E-P per-call descent evidence side table · `#703`
DB-11 type alias RHS where · `#736`/`#746` T-PB-B test infrastructure.

### Verdict

**STOP-AND-ESCALATE bullet 1 ("introduction-rate massively exceeds
dissolution-rate, e.g., 10:1") does NOT fire.** Ratio is ~1.6:1.

**But the velocity is clearly imbalanced for a phase that should be
"dissolution-first" per the user's concern + Reflective Analysis verdict
+ memory `feedback_redirect_noop_prs` / `feedback_construction_over_ratchets`.**

Honest reading: most of the 13 introductions are correctly-scoped fail-
closed scaffolds with named triggers (Reflective Analysis is right that
direction is sound). The issue is not that introductions are bad; it is
that introductions are 60% of a 7-day window during the same period
where R2 close demands accelerated dissolution. **Without a discipline
shift, the next 7-day window will likely run the same ratio.**

---

## 3. Priority ordering (top-N debt items)

Criteria applied (from brief §3): risk-shape > stable-but-ugly · R2-blocking
> orthogonal · accumulating > stable · cheap-dissolution > expensive · class
> one-off.

### Tier 0 — fail-closed P3 violations (immediate dispatch, S-scope each)

These are NOT scaffolds; they fabricate plausible output. Independent;
dispatch in parallel. Each PR is small.

1. **Go `UnknownVariant` fabrication** (`emit.rs:1456-1464`). Replace
   with `EmitError::VariantParentNotFound`. PR #809 entry. **Cheapest
   real-correctness win in inventory.**
2. **`lower_fn_body_into_existing_decl` defensive Arrow re-derive**
   (`lower.rs:3585-3611`). Replace with diagnostic; fix root cause in
   seed phase. PR #809 entry.
3. **Lens fold ambiguous unique-candidate fallback**
   (`lens_apply.rs:132-148`). Require structural template-formal edge
   identification; remove uniqueness heuristic. PR #809 entry. (The
   file-suffix special case in the same file is part of §0 class; do
   the structural fix once, not twice.)

### Tier 1 — class-of-pattern (single program, M-scope)

4. **Identity-carrier substrate pass** (§0 class). Eight surface
   instances; one upstream cause. Director-level brief recommended.
   **Frame (per `feedback_groundedness_gates_lenses.md` revised
   2026-04-25):** the language vocabulary is **primitives + namespacing
   / composition only** — there is no user-defined-primitive feature,
   no escape syntax, no annotation the compiler can't see through.
   Consequence: there is no "ungrounded user program" category; the
   lens contract is "applies to every program by construction." If a
   lens needs an ungrounded output path for user programs, the design
   has a leak. **Diagnosis of the §0 class:** the eight sentinels
   (`PROGRAM_INPUT_SENTINEL`, `span.file ==` checks, fixture-name
   routing, `include_str!` lens side-channels, file-preference rank,
   etc.) are NOT "ungrounded fallbacks the compiler labels." They are
   **the compiler itself failing to use the language's primitives +
   namespacing internally** — the compiler reaching for sentinel
   strings instead of structural carriers (`DeclarationRef`, structural
   template-formal edges for fold-step identity, explicit input-value
   carriers for `LensOutputEquals.input_ref`, structural emit-helper
   carriers in place of `bind.span.file`-keyed dispatch). **Tier 1
   brief framing:** "the compiler holds itself to the language's own
   vocabulary." Same eight dissolution sites (#1.1, #1.2,
   `lower.rs:836`, `emit.rs:3181/3206`, file-preference rank,
   `PROGRAM_INPUT_SENTINEL`, `EXTDEPS_BOOTSTRAP_FIXTURES`,
   `include_str!` lens side-channels); sharper diagnosis — these are
   compiler-internal vocabulary leaks, not user-surface ambiguity. The
   structural fix (the carriers named above) is what makes the
   compiler-internal usage match the user-surface vocabulary. **This
   is the highest-leverage paydown in the entire inventory.**

### Tier 2 — risk-shaped + R2-coupled (S-scope each, dispatch with R2 lanes)

5. **Loop emission semantic invariant** (PR #809 entry). Currently
   comment-level; degrades silently if any future PR adds another Loop
   source. R2 demos depend on Python/Go emission, so this is R2-coupled.
   **Brief MUST start with a construction-closure audit, NOT with a
   marker design.** Step 1: enumerate every `Behavior::Loop`
   construction site in `lower.rs` (and anywhere else); confirm or
   refute that all paths route through recursive-function lowering.
   Step 2 (conditional on the audit): if construction-closure holds, the
   brief becomes "document the closure invariant as a structural
   integration test; retire the speculative `LoopKind` marker idea" —
   no new substrate. If it does NOT hold, the marker/test framing
   applies and the brief turns into a `LoopKind` lowering-marker spec.
   Both proposed paths in the original PR #809 row are bridges; do not
   author the marker brief blind. Per `feedback_construction_over_ratchets`,
   prefer the structural-closure outcome.
6. **`src/v3/std` vs `dsl/std` checklist undercount fix** (PR #809
   entry). One-line fix to `dag.rs:2735-2764` + mirror in `lower.rs`.
   Or surface as "explain why these three modules are exempt." Belongs
   to whoever owns the file-preference rank scaffold; trivial dispatch.
7. **`patch_lower_helpers_generated_type_alias_refinement` retirement**
   (PR #809 entry). Belongs to Zero-Floor Manager PB-Tier1 work; the
   "first PB cleanup target" framing is already in the row.

### Tier 3 — substrate-deep, expensive, blocked on substrate capability (M+ scope)

8. **Lossy user-lens reflection** (PR #809 entry). Generate
   `Behavior → FieldValue` reflection from `src/v3/std/substrate.dag`
   structurally. Adjacent to #4; could compose. Don't pre-empt #4.
9. **Duplicated declaration walkers + `SubstStack`** (PR #809 entry).
   Real semantic drift documented (`infer.rs:181-187` vs
   `lower.rs:2469-2486` on `CardinalityBound::AtMostOne`). Single shared
   substrate query surface. M scope.
10. **Host-Rust mirrors of std termination/computation/induction**
    (§1.5). These dissolve when v3 can lower + evaluate `.dag` runtime
    values — same dependency as Grounding Engine row consumption
    (§1.3). One substrate-capability program unlocks both. Belongs to
    Zero-Floor + R2 Grounding coordination.
11. **LanguageSpec dual authority** (ROADMAP `:397`, novel from earlier
    exploratory). Carrier-merge design call; not cheap. Defer behind
    Tier 1.
12. **Effect-carrier Rust mirror** (§1.6). Mechanical PB dissolution
    once self-hosting reaches it; queue inside PB-Tier1-Sweep without
    special prioritization. **Effects model framing is correct — do
    not redesign.**

### Off priority list

- PR #726 SHIP_WITH_DEBT items (E-P/E-M behavioral wiring, peano
  unification): tracked at ROADMAP `:299` with dedicated checklist;
  proceeding in lane.
- All `[invariant-reveal]` items per ROADMAP `:307` framing — these are
  "evidence the language grew," not items to chase.

---

## 4. Debt-paydown framework recommendation

### Options weighed

- **(a) Capacity carve-out (X% per cycle).** Concrete, easy to enforce,
  but creates an "us vs them" framing between R2 lanes and debt lanes;
  Director would arbitrate which side eats a slow week. Doesn't solve
  the class-of-pattern problem (the 8 identity-carrier sites would still
  dispatch as 8 separate slots).
- **(b) Paired dispatch.** Each R2 implementation lane comes with a
  paired debt-paydown lane that touches adjacent debt. Strong because
  it forces dispatchers to identify adjacency at brief time, which is
  where class-of-pattern thinking happens naturally. Risk: pairing
  becomes performative ("any debt PR counts"); pairing might block R2
  lanes on unrelated debt readiness.
- **(c) Stop-the-line on velocity.** R2 lane dispatch pauses if scaffold
  introduction > dissolution in any week. Hardest to game; aligns with
  user's velocity concern. Risk: too brittle for a 1-week window —
  legitimate weeks where 4 R2 lanes ship vs 1 dissolution lane shouldn't
  pause R2 close. Also doesn't solve class-of-pattern.
- **(d) Dedicated debt manager.** Standing-role manager owning debt
  across R2 + Pure Bootstrap to Zero. Largest commitment; right answer
  if debt was the program. It isn't — debt is a property of two existing
  programs (R2 close + Zero-Floor self-hosting). Debt manager would
  duplicate the standing roles.

### Recommendation: **hybrid (b) + per-PR gate + (c)-lite**

**Primary mechanism — Paired-dispatch discipline at brief authoring time.**

Every Director-dispatched ad-hoc R2 worker brief that introduces a
scaffold (any new hand-Rust file, any new sentinel, any new Rust mirror
of a `.dag` authority) MUST name in its "Acceptance" section:

1. The dissolution trigger (already required by
   `feedback_construction_over_ratchets`).
2. **The adjacent debt row in ROADMAP it touches** — and whether the
   PR contributes to dissolving that adjacent row, or explicitly defers.
3. **If the brief introduces a string/path/name identity bridge**
   (sentinel, fixture-name routing, `span.file ==` check, `include_str!`
   side-channel): it MUST be authored against the §0 identity-carrier
   pass program, not as a one-off.

This catches class-of-pattern at the dispatch layer rather than during
debt review, which is where the 2026-04-22 → 2026-04-25 window
accumulated. The cost is brief-authoring discipline (Director +
worker-brief authors), not capacity.

**Secondary mechanism — Per-PR gate (the early warning).**

Paired-dispatch lives at brief-authoring time and depends on author
discipline; it can be bypassed in a hurry. Add a **per-PR gate** that
makes P5 (Progress Is Dissolution) into a reviewable line on every PR
description: **no new hand-Rust file in `v3/` lands without the PR
description naming the file or scaffold it deletes.** Forms accepted:
"deletes X" · "shrinks census line Y" · "explicit deferral to lane Z
with named row." If none of these are present, reviewer requests one.

Cheap to enforce (PR template line + reviewer check); no capacity
tradeoff; catches drift even when ad-hoc dispatch skips paired
discipline. This converts P5 from invariant-text into a per-PR
review-item.

**Tertiary mechanism — Velocity tripwire (not stop-the-line).**

Reflective + Exploratory analysis cadence is already a practice
(ROADMAP `:418`). Add: each cadence pass reports introduction:dissolution
ratio for the window. **If ratio ≥ 3:1 in any 7-day window**
(materially worse than the current 1.6:1), Director surfaces it for
program-level review and considers pausing new ad-hoc lanes until ratio
recovers. **Below 3:1, no automatic action** — analyses are advisory
input, not blocker.

This is (c) softened so it doesn't trigger on a normal week. The 3:1
threshold is calibrated: the current 1.6:1 already concerned the user;
2:1 would be borderline; ≥3:1 is the regression signal. **Calibration
caveat — see §2 footnote**: dissolution-shaped PR-title heuristic
undercounts dissolution work that ships *inside* feature PRs (e.g., a
T-LensAPI PR that adds a lens and retires three legacy paths in the
same diff). Before the tripwire triggers a Director review, the
cadence pass MUST do a manual sweep for dissolution-bearing feature
PRs in the window to avoid false positives. The gate is the early
warning; the tripwire is the late warning; the per-PR gate is the
finest grain.

**What does NOT change.**

- R2 structure (per `r2-structure.md` LIVE).
- Pure Bootstrap to Zero PB-* lane structure (per `pure-bootstrap-zero-manager.md`).
- Standing managers (Grounding `crisp-seal-366`, Zero-Floor
  `stern-swift-335`).

Director ad-hoc dispatch is where the discipline lives; standing
managers consume the discipline as part of their normal lane intake.

### Cross-manager coordination notes

- **Grounding Manager (`crisp-seal-366`):** §1.3 grounding scaffolds
  dissolve through Grounding Engine Phase 1+ work, gated on top-level
  `ValueBody` extension (substrate-capability). Synthesis recommends
  Grounding lane sequencing stays as planned. **No change requested.**
- **Zero-Floor Manager (`stern-swift-335`):** §1.4 (`patch_lower_helpers_*`)
  + §1.5 (host-Rust mirrors) + §1.6 (effect-carrier mirror) all dissolve
  through PB-* lanes. Synthesis recommends Tier 2 item #7
  (`patch_lower_helpers_*`) be lifted to PB-Tier1-Sweep priority since
  it is explicitly named "first PB cleanup target." **One priority hint
  requested.**

---

## 5. Course-of-action recommendation (Director-actionable)

**Do these, in this order.**

1. **PR #809 prerequisite is satisfied** — merged 2026-04-25; debt-row
   line citations to ROADMAP §"Post-merge debt (2026-04-25 reflective +
   exploratory analyses)" are now authoritative. Briefs B1-B7 below
   can be authored against current `main`.
2. **Dispatch Tier 0 fail-closed P3 fixes** (3 small parallel briefs):
   Go `UnknownVariant`, `lower_fn_body` Arrow re-derive, lens fold
   ambiguous fallback. Each S-scope, independent. **Expect:** 1 cycle.
3. **Author the Identity-Carrier Substrate Pass program brief**
   (Tier 1, item #4). Treats the §0 class as ONE M-scope program
   framed per `feedback_groundedness_gates_lenses.md` (revised):
   **language vocabulary is primitives + namespacing/composition only;
   there is no ungrounded user-program category; the compiler must
   hold itself to the language's own vocabulary.** The §0 sentinels
   are compiler-internal vocabulary leaks (the compiler reaching for
   sentinel strings instead of structural carriers); the fix is to
   replace them with `DeclarationRef`, structural template-formal
   edges, explicit `LensOutputEquals` input-value carriers, and
   structural emit-helper carriers. Enumerate the 8 surface
   dissolution sites; sequence substrate work before site-by-site
   dissolution. **This is the synthesis's primary recommendation.
   Expect:** 2-3 cycles for the program; sites collapse rapidly once
   substrate lands.
4. **Adopt the paired-dispatch + per-PR gate + velocity-tripwire
   discipline (§4)** as a checklist-line addition to Director ad-hoc
   dispatch + a PR-template-line addition for `v3/` hand-Rust files.
   Cost: brief-authoring discipline + one reviewer line-item.
5. **Dispatch Tier 2 risk-shaped items in parallel with R2 lanes**:
   Loop-emission **construction-closure audit first** (S, R2-coupled —
   the marker brief is conditional on the audit refuting closure per
   Tier 2 #5; if closure holds, deliverable is a structural integration
   test, not a marker), checklist undercount fix (S, trivial),
   `patch_lower_helpers_*` retirement (S, on PB-Tier1-Sweep priority
   hint to Zero-Floor Manager).
6. **Tier 3 items defer behind Tier 1.** Re-prioritize after Tier 1
   program lands; several Tier 3 items (lossy lens reflection;
   declaration walkers) likely simplify or compose with Tier 1 outputs.

**Total expected window:** 3-4 cycles for Tiers 0-2 plus discipline
adoption. R2 close not impacted. Pure Bootstrap to Zero lane sequencing
not changed (one priority hint to PB-Tier1-Sweep).

**What changes if scaffolding introduction outpaces dissolution again.**
Three layers, finest grain to coarsest. **Per-PR gate** (every `v3/`
hand-Rust PR names what it deletes or explicitly defers) catches drift
at the PR-review layer. **Paired-dispatch discipline** at brief authoring
time catches it at the dispatch layer. **Velocity tripwire** (≥3:1
introduction:dissolution ratio in a 7-day window, after manual sweep
for dissolution-bearing feature PRs per §2 footnote) puts ad-hoc lane
dispatch under Director review until the ratio recovers. The current
1.6:1 ratio is acceptable but warns the next imbalance is likely.

### Briefs recommended for Director authoring (post-framework approval)

- **B1.** Tier 0a: Go `UnknownVariant` → `EmitError::VariantParentNotFound` (S).
- **B2.** Tier 0b: `lower_fn_body_into_existing_decl` defensive fallback → diagnostic + seed-phase root cause (S).
- **B3.** Tier 0c: Lens fold ambiguous unique-candidate fallback → require structural template-formal edge (S).
- **B4.** **Tier 1 program brief: Identity-Carrier Substrate Pass (M, primary recommendation).** Class-of-pattern dissolution covering 8 surface sites; framed per `feedback_groundedness_gates_lenses.md` (revised): language has no escape syntax → no ungrounded-user-program category → lenses apply by construction. The §0 sentinels are compiler-internal vocabulary leaks; the brief frames as "the compiler holds itself to the language's own vocabulary."
- **B5.** Tier 2a: Loop-emission construction-closure audit FIRST; reframe to "document construction-closure invariant as structural test" if closure holds, marker brief only if audit refutes closure (S, R2-coupled).
- **B6.** Tier 2b: file-preference rank checklist completion (`computation`/`induction`/`termination`) (S, trivial).
- **B7.** Priority hint to Zero-Floor Manager: lift `patch_lower_helpers_*` retirement to PB-Tier1-Sweep priority.
- **Checklist + PR-template edit.** Director ad-hoc dispatch checklist gains paired-dispatch line; `v3/` PR template gains "names file/scaffold deleted (or defers)" line; velocity-tripwire reporting added to integration-reflection cadence.

---

## Acceptance checks

- [x] Scaffolding inventory authored — every recent-window scaffold cited (§1).
- [x] Velocity audit run — 1.6:1 introduction:dissolution ratio surfaced honestly (§2).
- [x] Priority ordering authored — 12 items across 4 tiers (§3).
- [x] Debt-paydown framework proposal authored — hybrid (b)+(c)-lite picked + justified (§4).
- [x] Course-of-action recommendation single-page summary authored (§5).
- [x] Doc-only diff; no code change.
- [x] Class-of-pattern STOP surfaced as Director-actionable §0 finding rather than item-by-item priority list.
