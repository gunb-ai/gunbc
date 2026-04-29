# T-Ground-Diagnostic — `EmissionDiagnostic` substrate + declared diagnostic ordering

**Status:** PROPOSAL — dispatchable now. Q6.5 two-layer diagnostic authority is **LIVE** on main via PR [#1129](https://github.com/gunb-ai/gunbc/pull/1129); this lane is a **Layer-1 consumer** of `CompilerDiagnosticKind` and does **not** widen that closed sum. Authored 2026-04-29 (R2 Grounding Manager redirect from sibling dispatch).

**Lane:** T-Ground-Diagnostic (S) — item **8** of 11 in [`r2-grounding-manager.md`](r2-grounding-manager.md) (lane description line 34, lane row line 67, pending list line 144).

**Manager:** R2 Grounding Manager ([`r2-grounding-manager.md`](r2-grounding-manager.md)).

**Lineage / authorities consumed (no re-litigation):**
- R2 manager lane row + acceptance gate: [`r2-grounding-manager.md`](r2-grounding-manager.md) lines 34, 67, 127 (`diagnostic_structural_ordering_landed`), 144.
- Q6.5 two-layer authority: [`docs/design-lens-framework.md`](../design-lens-framework.md) §**"Q6.5 — Two-layer authority for diagnostic kinds"** — Layer 1 `CompilerDiagnosticKind` is Substrate-owned; **this lane consumes; does NOT extend**; anti-bridge: lens-instance kinds never enter `CompilerDiagnosticKind`.
- Engine-reframe + fold failures: [`docs/design-emission-model.md`](../design-emission-model.md) — Modeling problems **4** (ordering is diagnostic-only; lines ~152–164) and **5** (fail-closed diagnostic surface; lines ~165–188); `EmissionDiagnostic` worked shapes (e.g. `UnderRefined`, `NoInhabitant`; search **EmissionDiagnostic** in that doc); lane table row ~386 (`T-Ground-Diagnostic` owns carrier + resolution-hint structure); **UnderRefined** worked receipts **Example 1** (bound / unrefined `Int`, lines ~417–464) **and Example 5** (algebra ambiguity, `unspecified_axis: "algebra"`, lines ~639–680) **plus** Example 6 as lifted test targets.
- Fail-closed compilation: [`INVARIANTS.md`](../../INVARIANTS.md) **C-8** (P3) + C-series sentinels — no silent fabrication when the fold cannot determine.
- Substrate-fact introduction: [`INVARIANTS.md`](../../INVARIANTS.md) §P1 — mandatory for every new substrate type / variant / field this lane authors.
- Brief shape templates: [`t-ground-languagespec.md`](t-ground-languagespec.md), [`t-ground-lifetime-analyzer.md`](t-ground-lifetime-analyzer.md).

---

## Framing question this lane answers

When the structural fold (Coercion-Fold) cannot emit a unique target — because the program is under-refined on an axis, because candidates genuinely do not cover the program’s refinement, or because upstream lanes surface contradictory structural facts — **what typed carrier** names the failure, carries **resolution hints**, and (where needed) carries **declared diagnostic-only enumeration order** so the user sees alternatives without the fold smuggling “minimum satisfier” engine policy?

A “yes” lands the substrate completion Modeling problem 5 calls for (`design-emission-model.md:165-188`): the diagnostic is a **structural fact**, not a stringly-typed escape hatch.

---

## Q6.5 framing (load-bearing)

Per [`docs/design-lens-framework.md`](../design-lens-framework.md) §Q6.5:

| Authority | Owner | This lane |
|-----------|-------|-----------|
| **Layer 1** — `CompilerDiagnosticKind` closed sum (`src/v3/std/diagnostics.dag`) | Substrate Manager | **Consumer only.** Do not add variants here. |
| **Layer 2** — lens-instance `DiagnosticKindDecl` / `LensInstanceKindWitness` | Lens authors / Evaluator lanes | **Out of scope** — no lens-instance kind authoring in this lane. |
| **`EmissionDiagnostic` (fold / emission failure carrier)** | **T-Ground-Diagnostic (this lane)** | **Author** the typed sum and attach it to the fold’s fail-closed boundary (see Scope). |

**Anti-bridge:** extending `CompilerDiagnosticKind` to carry fold-specific failures is forbidden — it would violate Q6.5 and P2 single-authority. Fold failures use **`EmissionDiagnostic`** (or a substrate-declared synonym that is **not** a `CompilerDiagnosticKind` variant); mapping into `Diagnostic { kind: AnyDiagnosticKind, ... }` is a **consumer** concern coordinated with Substrate / Coercion-Fold wiring, not an extension of the Layer-1 closed sum.

---

## Scope

### A. Typed `EmissionDiagnostic` carrier (substrate authoring)

Author a **closed sum** (or equivalent substrate record + tagged variants) for emission/fold failures named in [`docs/design-emission-model.md`](../design-emission-model.md), including at minimum:

- **`UnderRefined`** — program intent or a structural axis is under-specified; candidate set (or axis name) + **resolution hints** per Modeling problem 5. Align field names with worked examples: **Example 1** (bound / refinement gap on `Int`), **Example 5** (algebra ambiguity — `EmissionDiagnostic::UnderRefined { unspecified_axis: "algebra", .. }` per `fold_dag_int_ambiguous_algebra_fails_closed` in `design-emission-model.md` ~672–676), and growability / encoding axes cited in Examples 3–4 where applicable. **Do not collapse** bound-under-refinement and algebra-under-refinement into a single acceptance test — they are distinct `UnderRefined` shapes.
- **`NoInhabitant`** — substrate does not declare a candidate covering the program’s stated refinement (Example 6 pattern).
- **Contradiction / multi-site conflict** — when upstream analysis yields incompatible structural constraints (e.g. lifetime analyzer’s `ContradictoryUse` pattern; Coercion-Fold meet on facts).

Exact variant set and payload shapes are **landed in the lane PR** with P1 receipts; the authority is `design-emission-model.md` Examples + Modeling problem 5 — not ad hoc strings.

**Canonical home (P1 Step 1 — worker lands in PR):** default position is adjacent to existing diagnostic substrate (`src/v3/std/diagnostics.dag` or `emit_model.dag` per DAG-ancestor check). **Escalate to manager (#1133)** if Step 1 shows a parent type that cannot host the sum without parallel authority.

### B. Declared structural ordering for diagnostic enumeration

Per Modeling problem 4 corrected (`design-emission-model.md:152-164`): ordering is **not** used to choose emission; it is a **declared substrate fact** used only when **constructing fail-closed diagnostics** (e.g. enumerating `Int8..Int128` for an unrefined `Int`).

This lane authors or wires the **substrate declarations** that make that ordering explicit (per-category / per-algebra as the design doc requires). If T-Ground-LanguageSpec already carries per-target tables that subsume a subset of ordering data, this lane **consumes** those declarations rather than duplicating them (P2 single-authority); gaps escalate via manager.

### C. Consumer wiring boundary (not this lane’s body)

Mapping `EmissionDiagnostic` → runtime `Diagnostic` / `AnyDiagnosticKind`, renderer strings, and target-localized messages is **Coercion-Fold + emit pipeline** work. This lane **authors the carrier + declarative ordering facts**; downstream lanes **consume**.

---

## Outputs

- Substrate-declared **`EmissionDiagnostic`** sum with typed variants for fold failures (`UnderRefined`, `NoInhabitant`, contradiction / conflict, …) per `design-emission-model.md`.
- **Declared ordering** artifacts for diagnostic enumeration (diagnostic-only), referenced by `UnderRefined` (and similar) payloads where the design doc requires ordered candidate lists.
- **P1 procedure receipts** (§G) in the landing PR for every new substrate type / variant / field.

---

## Fail-closed discipline (INVARIANTS.md C-8)

Every Coercion-Fold path that today would “guess,” return `None`, or silently pick a default must instead yield **`EmissionDiagnostic`** (or `Result<_, EmissionDiagnostic>` at the fold boundary) with enough structure to name **what failed**, **what candidates existed (if any)**, and **what would resolve** the failure. No silent meet-to-default; no stringly-typed parallel diagnostic vocabulary outside the declared carrier.

---

## Cross-lane convergence — T-Ground-Lifetime-Analyzer (#1206)

PR [#1206](https://github.com/gunb-ai/gunbc/pull/1206) introduced `v3-grounding-lifetime` with a **lane-local Rust mirror** of fold failures (`ContradictoryUse`, `UnderRefined`, `OutOfR2Scope`) to keep the analyzer **SG-0 isolated** from `src/v3/compiler/`.

**Dissolution path:** when this lane lands the substrate `EmissionDiagnostic`, **migrate** the lifetime analyzer (and other R2 producers) to the **substrate-declared** sum — retire the lane-local mirror in the same sequencing pass as Coercion-Fold consumer wiring (coordinate with T-Ground-Coercion-Fold). Until then, the mirror is an intentional staging debt (named in #1206 PR body).

---

## Dependencies / gates

| Gate | Status | Lane impact |
|------|--------|-------------|
| **Q6.5 LIVE (#1129)** | merged | `Diagnostic.kind` / `AnyDiagnosticKind` substrate exists; this lane consumes Layer-1 authority without extending it. |
| **Coercion-Fold consumer** | sibling / downstream | First production consumer of `EmissionDiagnostic`; sequencing per `design-emission-model.md` §"Affected lanes" option (c). |
| **T-Ground-LanguageSpec** | sibling | May supply candidate / ordering **data** this lane references; avoid duplicate ordering authority (P2). |

---

## Out of scope (do NOT do)

- **Extending `CompilerDiagnosticKind`** — Substrate Manager only (Q6.5 anti-bridge).
- **Authoring lens-instance diagnostic kinds** — Layer 2; lives in lens `.dag` files, not here.
- **Coercion-Fold mechanical body** — sibling lane T-Ground-Coercion-Fold.
- **Per-target diagnostic localization / UX copy** — emit/renderer lanes; this lane stays on **typed carriers + declarative ordering facts**.
- **Re-litigating Q6.5**, Modeling problems 4–5, or C-8.

---

## Sizing

**S** per [`r2-grounding-manager.md`](r2-grounding-manager.md) lane row (line 67) and `design-emission-model.md` lane table (~386): carrier + ordering declarations are bounded; heavy fold logic stays out of this lane.

---

## Substrate-fact introduction — P1 procedure (`INVARIANTS.md` §P1)

Worker MUST run the 3-step procedure for **every** new substrate type / variant / field introduced by this lane and cite receipts in the PR body:

1. **DAG-ancestor** — does a parent declaration already carry this failure shape (e.g. widen existing `Diagnostic` vs new sum)? Name attachment target.
2. **Coproduct-vs-coordinate** — `EmissionDiagnostic` variants are **alternatives** (one failure mode at a time); resolution hints inside a variant are **coordinates** (record fields), not new sum arms without P1.
3. **Primitive-vs-lens-extensible** — fold failure kinds are **substrate-declared** computational primitives for the compiler pipeline, not lens-extensible user vocabulary.

---

## Test plan

Hermetic, behavior-driven, unit-first (`TESTING.md`); sub-second per `feedback_test_timeout_2s.md`.

1. **UnderRefined — bound axis (Example 1)** — lift `design-emission-model.md` (~417–464): unrefined `Int` ⇒ `UnderRefined` with enumerated candidates + `unspecified_axis` / hint structure matching the `fold_dag_int_unrefined_fails_closed` `TestClaim` sketch (`unspecified_axis: "bound"` in that doc’s worked shape).
2. **UnderRefined — algebra ambiguity (Example 5)** — lift `design-emission-model.md` (~639–680): program intent under-determines **which algebra** (distinct from “algebra known, bound missing” in Example 1) ⇒ `UnderRefined` with **`unspecified_axis: "algebra"`** and payload matching `fold_dag_int_ambiguous_algebra_fails_closed` / `expected_diagnostic: matches(EmissionDiagnostic::UnderRefined { unspecified_axis: "algebra", .. })`. **Both** Example 1 and Example 5 **must** land as separate `.dag` `TestClaim` receipts before implementation dispatch treats UnderRefined acceptance as complete.
3. **NoInhabitant parity** — Example 6 (`design-emission-model.md` ~684–731): refinement present, no covering candidate ⇒ `NoInhabitant` (or equivalent authored name) with structured payload.
4. **Contradiction / conflict** — two incompatible structural constraints ⇒ typed contradiction variant (align with lifetime analyzer migration path).
5. **Ordering is diagnostic-only** — regression asserting fold emission path does **not** consult ordering tables for selection; diagnostics may reference declared order for enumeration only (tie to Modeling problem 4).
6. **Q6.5 non-extension** — automated or manual guard: `CompilerDiagnosticKind` variant set unchanged by this lane’s diff (Layer-1 closed sum ratchet).
7. **`cargo test` / `clippy` / `fmt`** gates per workspace rules.

---

## Dissolution claim

When this lane merges:

- Fold failures are **typed structural facts** (`EmissionDiagnostic`), not ad hoc strings or engine exceptions — receipt: **Examples 1, 5, and 6** each lifted to at least one `.dag` `TestClaim` (Example 1 = bound `UnderRefined`; Example 5 = algebra `UnderRefined` with `unspecified_axis: "algebra"`; Example 6 = `NoInhabitant`).
- **Diagnostic-only ordering** is substrate-declared — receipt: ordering tables / fields are data, not hidden engine policy (`design-emission-model.md:160-163`).
- **Lane-local `EmissionDiagnostic` mirrors** (e.g. `v3-grounding-lifetime`) have a **named migration path** onto the substrate carrier — receipt tracked in PR sequence with Coercion-Fold / analyzer crates.

---

## Acceptance — `.dag` gate

Per [`r2-grounding-manager.md`](r2-grounding-manager.md) line 127:

> `diagnostic_structural_ordering_landed` — declared enumeration order; fail-closed diagnostic surface

Authored as a `.dag` `TestClaim` per **structural-acceptance-per-lane-close** discipline (`r2-grounding-manager.md` §Reporting / `r2-structure.md`).

---

## Hand-off discipline

Escalate to manager (#1133) if:

- P1 Step 1 shows **no attachment seam** for `EmissionDiagnostic` without violating P2.
- LanguageSpec and this lane **both** claim the same ordering authority without a dissolution plan.
- A fold failure shape **requires** extending `CompilerDiagnosticKind` — that is a **Q6.5 violation**; stop and reroute.

---

## Cross-refs

- Parent: [`r2-grounding-manager.md`](r2-grounding-manager.md) (lane 8 of 11)
- Q6.5: [`docs/design-lens-framework.md`](../design-lens-framework.md) §Q6.5
- Modeling 4–5 + `EmissionDiagnostic`: [`docs/design-emission-model.md`](../design-emission-model.md)
- INVARIANTS: [`INVARIANTS.md`](../../INVARIANTS.md) C-8, §P1
- Sibling briefs: [`t-ground-languagespec.md`](t-ground-languagespec.md), [`t-ground-lifetime-analyzer.md`](t-ground-lifetime-analyzer.md)
- Lifetime staging PR: [#1206](https://github.com/gunb-ai/gunbc/pull/1206)
