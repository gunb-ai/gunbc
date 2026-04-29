# T-Ground-CrossTarget-Meta — L6 substrate-load completeness (cross-target uniformity meta-spec)

**Status:** PROPOSAL — **brief authoring dispatchable now** (per R2 Grounding Manager inbox #1203 / #1133, 2026-04-29). **Implementation gated** on **PR-J** merge with **Q5 no-op confirmation** (cardinality-as-connectives axis already collapsed via PR-PreF `Interval<Cardinal>`; PR-J is the single cadence trigger before worker impl dispatch).

**Lane:** T-Ground-CrossTarget-Meta (S) — item **9** of 11 in [`r2-grounding-manager.md`](r2-grounding-manager.md) (lane description line 35, lane row line 68, pending list line 146).

**Manager:** R2 Grounding Manager ([`r2-grounding-manager.md`](r2-grounding-manager.md)).

**Lineage / authorities consumed (no re-litigation):**
- R2 manager lane row + cadence row + acceptance gate: [`r2-grounding-manager.md`](r2-grounding-manager.md) lines 35, 52 (PR-J → this lane), 68, 128 (`cross_target_meta_l6_load_completeness_landed`), 146.
- Modeling problem 7 + L6 placement + cross-product shape: [`docs/design-emission-model.md`](../design-emission-model.md) — search **Cross-target uniformity**, **Modeling problem 7**, **L6**, **`l6_structural_form_coverage`**, **Shape A**, **connectives × behaviors × targets**, Q5 / cardinality-as-connectives (lines ~1243–1280, ~408–410, ~818–833, ~959).
- **Not-a-lens authority:** [`docs/design-lens-framework.md`](../design-lens-framework.md) §**D3 / I6** — L6 is **not** a `Lens<C>` instance; input space is **per-(substrate form cell × Shape A target)**, not `Lens<C>.read: (Dag, Behavior) → Witness<C>`.
- Q1 / PR-PreF consolidation: `Interval<D>` parent; cardinality on connectives as `Interval<Cardinal>` (`design-emission-model.md` ~1019–1024, ~1038, ~1265–1268). **L6 row/column substrate authority** (P2): **`TypeConnective`** + **`Behavior`** in [`src/v3/std/substrate.dag`](../../src/v3/std/substrate.dag) — see §Scope (six + five variants).
- Fail-closed surface: [`INVARIANTS.md`](../../INVARIANTS.md) **C-8** (P3) + substrate-fact introduction **§P1** for any new types.
- Sibling brief shapes: [`t-ground-diagnostic.md`](t-ground-diagnostic.md), [`t-ground-languagespec.md`](t-ground-languagespec.md), [`t-ground-lifetime-analyzer.md`](t-ground-lifetime-analyzer.md).

---

## Framing question this lane answers

At **substrate load time** (before folding arbitrary user programs), is every **structural form cell** that Tier-1 emission must support **declared complete** for **every R2 Shape A language target** (Rust / Python / Go) — so cross-target **portability requirements** are structural facts, not silent policy gaps?

A “yes” closes Modeling problem 7 (`design-emission-model.md` ~211–222): the meta-spec prevents a target language spec from omitting an inhabitance / emission path for a portable case without a **typed, fail-closed** diagnostic.

---

## Not-a-lens framing (load-bearing)

Per [`docs/design-lens-framework.md`](../design-lens-framework.md) §D3 / I6 and [`docs/design-emission-model.md`](../design-emission-model.md) ~959:

| Primitive | Input space | This lane |
|-----------|-------------|-----------|
| **`Lens<C>` instance** | Per-**Behavior** substrate reads: `(Dag, Behavior) → Witness<C>` | **Out of scope** — no lens-instance authoring. |
| **L6 substrate-load completeness** | Per **(structural form cell × Shape A target)** — walk substrate + loaded language specs; verify **emission-path declarations** exist for each cell | **In scope** — a **runtime / load-phase predicate** over declared carriers, not a structural lens fold. |

**Different physics:** lenses classify per-node, per-behavior evidence during analytic fold; L6 classifies **coverage of the declared cross-product** at the boundary where specs are loaded. Do **not** implement L6 as `Lens<EmissionPathPresent>` or any other `Lens<C>` read channel — that was explicitly rejected as structurally wrong (input-space mismatch).

---

## Q5 lock + PR-J (single impl gate)

Per [`r2-grounding-manager.md`](r2-grounding-manager.md) lines 51–52, 91–92 and [`docs/design-emission-model.md`](../design-emission-model.md) Q5 (~1243–1280):

- **Q5 recommendation (a) — locked:** **cardinality is the connectives axis** (`design-emission-model.md` illustrates with shapes like `List<T>` → `Interval<Cardinal>::Unbounded`; fixed-arity products → exact cardinal intervals). There is **no separate L6 “cardinality variant” axis** beyond what is already carried on the substrate connectives — including the dedicated **`TypeConnective::Cardinality`** arm in `substrate.dag`.
- **PR-PreF reinforcement:** with `Interval<D>` as shared parent, cardinality lands on connectives via **`Interval<Cardinal>`** instances on those connectives — the L6 cross-product is **`TypeConnective` variant × `Behavior` variant × Shape A target** (design doc shorthand **90 cells** = 6 × 5 × 3), not `(… × cardinality_variants …)` as a **separate** axis beyond the six `TypeConnective` arms (cardinality is already modeled under **`Cardinality`** + `Interval<Cardinal>`, not a seventh connective tag).
- **PR-J:** cadence PR for Q5 cardinality enumeration — **expected no-op** once PR-PreF + Q5 lock are merged and recorded. **Worker implementation dispatch** for this lane waits on **PR-J merge as the confirmation artifact** that Q5 did not re-open a separate cardinality enumeration — even when the substantive work is empty.

---

## Scope — “form × target” (R2)

### Structural **form** dimension

After the Q5 collapse above, each **form cell** is one tuple in the **substrate-declared** cross-product:

- **Type connectives** — the **six** variants of **`TypeConnective`** in [`src/v3/std/substrate.dag`](../../src/v3/std/substrate.dag) (reflected-substrate authority; at brief authoring: **`Atom`**, **`Conj`**, **`Disj`**, **`Arrow`**, **`Cardinality`**, **`Instantiation`**). **Single authority (P2):** the L6 form axis **must** enumerate this sum — not informal names from Q5 prose in [`docs/design-emission-model.md`](../design-emission-model.md) (~1250–1251) such as `Singleton` / `ListOf<T>` / `ArrowOf<…>`, which describe modeling intent but **do not** match the durable connective tag set and would let the grid miss real declarations. **`Cardinality`** carries `element` + `CardinalityBound`; PR-PreF **`Interval<Cardinal>`** refines cardinality facts without adding a seventh connective arm.
- **L1 behaviors** — the **five** variants of **`Behavior`** in the same [`src/v3/std/substrate.dag`](../../src/v3/std/substrate.dag) coproduct (at brief authoring: **`Value`**, **`Transform`**, **`Branch`**, **`Loop`**, **`Bind`**). **Single authority (P2):** the L6 behavior axis **must** enumerate this sum — not prose-only nicknames from [`docs/design-emission-model.md`](../design-emission-model.md) (~408–409, ~959) alone. Those sections remain **intent** authority for *why* the fold walks five behavior classes; **row keys** for implementation are the substrate tags above (which align with the lens-framework’s L1 operations per [`docs/design-lens-framework.md`](../design-lens-framework.md) §`Lens<C>` primitive).

**Worker lands** the precise enumeration tables as **substrate facts** (or generated tables) in the lane PR with **P1 receipts** — this brief does not restate every cell ID.

### **Target** dimension (Shape A only for R2 L6)

Per [`docs/design-emission-model.md`](../design-emission-model.md) (~219, ~408, ~833) and [`docs/thesis/what-else-falls-out.md`](../thesis/what-else-falls-out.md) (~145–186):

- **In scope (R2 L6 predicate):** **Shape A** — compiler **language targets** the R2 program grounds: **Rust, Python, Go** (the portability triple in the manager table).
- **Out of scope (different thesis tracks):** **Shape B** — user-program **artifact** emitters (OpenAPI, SQL DDL, Terraform, …) live under **R3** / omni lanes (`docs/r3-structure.md`, T-Omni-Shape-B) — not part of this lane’s load-completeness predicate.
- **Out of scope (phase):** **Modeling problem 9** — first-class **language spec as compiler emission subject** (“does the compiler emit Shape A for its own `.dag` spec?” — `design-emission-model.md` ~346–354) is **post-R3 / dogfooding**; this lane does **not** extend L6 to that surface until Director reopens it.

**Naming note:** manager dispatch used an **A/B/C emission-shape** shorthand. Canonical tokens in-repo are **Shape A** vs **Shape B** as above; there is **no separate “Shape C”** heading in `design-emission-model.md`. This brief treats **R2 L6** as **Shape A × form cells** only and defers B + spec-as-subject explicitly.

### Predicate + failure surface

- **When:** substrate / language-spec **load** phase (same structural moment `design-emission-model.md` ~833 names for cross-target portability meta-spec).
- **What:** for each **(form cell × Shape A target)**, verify **sufficient declarations** exist in substrate so Coercion-Fold (sibling lane) could emit (emission path / inhabitance / operator / construction pattern — exact checklist is **consumer of T-Ground-LanguageSpec** rows; this lane **owns the completeness predicate**, not duplicate LanguageSpec authoring).
- **Failure:** **fail-closed** — no silent omission. Per `design-emission-model.md` ~959, gaps surface as a **typed diagnostic** with kind **`MissingEmissionPath`** (or equivalent substrate-declared name) **unless** P1 Step 1 attaches the same failure shape to **`EmissionDiagnostic`** with a distinct variant — **worker must choose one authority** in the landing PR and cite P1 (coordinate with **T-Ground-Diagnostic** so fold-time vs load-time carriers do not fork parallel string vocabularies).

**Outputs:**

1. **Per-(form cell × target) verdict** — pass / fail with **structured pointer** to the missing substrate row (connective id, behavior id, target id).
2. **On incomplete load:** structured failure aligned with the chosen carrier (`Diagnostic` vs `EmissionDiagnostic`) above — **never** silent accept.

---

## Dependencies / gates

| Gate | Role for this lane |
|------|---------------------|
| **PR-PreF** (`Interval<D>`, including `Interval<Cardinal>` on connectives) | **Consumed** — makes Q5 collapse structurally true; prerequisite substrate shape for declaring “cardinality on connective” without a phantom axis. |
| **PR-J** (Q5 enumeration — **likely no-op**) | **Implementation dispatch trigger** — merge confirms cadence; lane **does not** ship worker impl until PR-J lands. |
| **T-Ground-LanguageSpec** (PR-I gated) | **Sibling** — owns per-target primitive tables, axes, construction/operator facts L6 **reads** for completeness; this lane **must not** duplicate LanguageSpec authority (P2). |
| **T-Ground-Diagnostic** | **Sibling** — owns `EmissionDiagnostic` + ordering facts; this lane **coordinates** load-time failure shape (see Scope). |
| **T-Ground-Coercion-Fold** | **Downstream consumer** — assumes L6 + LanguageSpec already guarantee no silent cross-target gap at fold entry (or fold receives typed failure from load). |

---

## Out of scope (do NOT do)

- **Authoring `Lens<C>` instances** for L6 — structurally wrong (see Not-a-lens).
- **Per-target localization / UX copy** for diagnostics — renderer lanes.
- **Cross-target *uniformity claims* that lift to CostLens / R3** — e.g. proving equivalent **cost** or **runtime** behavior across targets (`design-emission-model.md` L5 territory); this lane is **substrate-load completeness**, not L5 corpus equivalence.
- **Shape B artifact emitters** or **Modeling problem 9** spec-as-subject emission — deferred per §Scope.
- **Re-litigating Q5, PR-J no-op expectation, or L6’s R2-vs-R3 placement** — locked in `design-emission-model.md` + `r2-structure.md` / `r3-structure.md`.

---

## Sizing

**S** per [`r2-grounding-manager.md`](r2-grounding-manager.md) lane row — predicate + wiring + TestClaims; heavy LanguageSpec population stays in **T-Ground-LanguageSpec**.

---

## Substrate-fact introduction — P1 procedure (`INVARIANTS.md` §P1)

Every **new** substrate type / variant / field introduced for L6 verdict carriers, missing-path payloads, or load-phase hooks requires full **P1** receipts in the implementation PR (DAG-ancestor, coproduct-vs-coordinate, primitive-vs-lens-extensible). **Escalate to manager (#1133)** if Step 1 shows L6 would require a **second** parallel completeness vocabulary already carried by LanguageSpec.

---

## Test plan

Hermetic, behavior-driven, unit-first ([`TESTING.md`](../../TESTING.md)); sub-second where feasible.

1. **Happy path** — fully-populated stub specs: L6 reports **all cells satisfied** for Rust/Python/Go.
2. **Single missing emission path** — fixture drops one declared row for one `(cell, target)` → **exactly one** structured failure; kind matches landed carrier (`MissingEmissionPath` or agreed `EmissionDiagnostic` variant).
3. **No silent pass** — regression: removing a substrate row never maps to success.
4. **Q5 collapse sanity** — no free-standing “cardinality axis” table beyond connective-carried `Interval<Cardinal>` (PR-J no-op discipline).
5. **`cargo test` / `clippy` / `fmt`** per workspace rules.

Sketches in `design-emission-model.md` (~1262, ~1225–1228) for `l6_emission_path_declared_*` / structural coverage — lift to `.dag` `TestClaim` names in the implementation PR.

---

## Cross-lane convergence

- **T-Ground-LanguageSpec** — this lane **consumes** the axis vocabulary + per-target tables LanguageSpec **authors**; L6 is a **read-only completeness witness** over that substrate (P2 single-authority). Same convergence pattern as **T-Ground-Diagnostic** consuming declared ordering **data** from LanguageSpec without owning ordering authority (`t-ground-diagnostic.md` §Scope B).
- **T-Ground-Lifetime-Analyzer** — respects **R2 vs R3 cut** (`t-ground-lifetime-analyzer.md`, `design-emission-model.md` ~635): L6 does **not** expand into closure/async/`Pin` territory; lifetime lane’s **R2** outputs remain the binding-level facts Coercion-Fold consumes; no annotation surface.
- **T-Ground-Diagnostic** — like the Diagnostic brief named the **lane-local `EmissionDiagnostic` mirror** in `v3-grounding-lifetime` (#1206) as **staging debt** with a substrate migration path, this brief names **load-time vs fold-time** diagnostic carriers as a **convergence point**: one typed family for “missing structural prerequisite” failures, or an explicit **non-overlapping** split justified in P1 — **not** parallel string vocabularies.

---

## Acceptance — `.dag` gate

Per [`r2-grounding-manager.md`](r2-grounding-manager.md) line 128:

> `cross_target_meta_l6_load_completeness_landed` — L6 per-(form × target) substrate-load completeness check fires by construction

Authored as a `.dag` `TestClaim` per **structural-acceptance-per-lane-close** discipline (`r2-grounding-manager.md` §Reporting cadence / `r2-structure.md`).

---

## Hand-off discipline

Escalate to manager (#1133) if:

- LanguageSpec and this lane **both** must author the same row keys without a dissolution plan.
- PR-J **does** introduce a non-no-op Q5 axis — L6 enumeration must be **revised in the same PR sequence** before claiming the 90-cell model.
- Load-time failure **requires** extending **`CompilerDiagnosticKind`** — likely violates Q6.5 consumer discipline; coordinate with Substrate / Diagnostic lanes instead.

---

## Cross-refs

- Parent: [`r2-grounding-manager.md`](r2-grounding-manager.md) (lane 9 of 11)
- L6 + Modeling problem 7: [`docs/design-emission-model.md`](../design-emission-model.md)
- **`TypeConnective` + `Behavior` authority:** [`src/v3/std/substrate.dag`](../../src/v3/std/substrate.dag) (`TypeConnective`, `Behavior`)
- Lens vs L6: [`docs/design-lens-framework.md`](../design-lens-framework.md) §D3, I6
- Shape A vs B thesis: [`docs/thesis/what-else-falls-out.md`](../thesis/what-else-falls-out.md)
- R3 verification split: [`docs/r3-structure.md`](../r3-structure.md) (L6 removed from R3 surface)
- L2 / R2–R3 mapping: [`docs/thesis/r2-r3-thesis-mapping.md`](../thesis/r2-r3-thesis-mapping.md) (L6 row)
- Sibling briefs: [`t-ground-diagnostic.md`](t-ground-diagnostic.md), [`t-ground-languagespec.md`](t-ground-languagespec.md), [`t-ground-lifetime-analyzer.md`](t-ground-lifetime-analyzer.md)
