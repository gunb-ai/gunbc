# Lifetime axes vocabulary — audit refresh (string-namespaced substrate post-#1465)

**Status:** AUDIT REFRESH (doc-only; cites substrate authority in-tree).  
**Dispatch:** [gunbc#1411](https://github.com/gunb-ai/gunbc/issues/1411) — re-engagement after partial substrate landing; converge with `docs/audit/coercion-fold-lifetime-analyzer-convergence.md` and **#1130** (Substrate Manager).  
**Ground truth:** Substrate string-family axis sums landed in **[#1465](https://github.com/gunb-ai/gunbc/issues/1465)** — walk `src/v3/std/emit_model.dag` at merge-base with `main` (see Section 1.6).  
**Pattern precedent:** Same “refresh spec against landed DAG” discipline as the calm-tern **[#1454](https://github.com/gunb-ai/gunbc/issues/1454)** audit refresh (parallel slice narrative; cite that issue when extending other axis docs).

**Parallel pattern (unchanged):** `src/v3/std/integer_diagnostic_order.dag` (T-Ground-Diagnostic slice 2) — structural facts in substrate + lockstep ratchet for lane-local Rust until codegen.

This document **specs** how axis vocabulary should line up after string-family rows reference substrate values **structurally**. **Substrate Manager** authors the `.dag`; **Grounding** consumes and retires mirrors. The original **#1442** pre-audit assumed **non-namespaced** shared sums (`Ownership`, `LifetimeScope`, …); **#1465** chose **`String*Axis` namespacing** under emit-model authority instead — this refresh records that split and the remaining product questions.

---

## 1. Current axis vocabulary at HEAD (lane-local Rust)

**Authority today:** `src/v3/grounding_lifetime/src/facts.rs` defines program-side `LifetimeFacts` and per-axis closed sums. Docstrings cite P1 posture (`INVARIANTS.md` P1) and `design-emission-model.md` Examples 3–4 / section 635 for R3 deferrals.

### 1.1 `Ownership`

| Variant | Role |
|--------|------|
| `Owned` | Value must be owned at use boundaries satisfied by the fold today. |
| `Borrowed` | Value may be borrowed for the analyzed sites. |

No `Conditional` / other arms in this crate — those stay target inhabitance / R3 per crate docs.

### 1.2 `LifetimeScope`

| Variant | Role |
|--------|------|
| `Self_` | Module/static-like / self-contained duration (top-level data, returns). |
| `Caller` | Parameter bounded by caller frame. |

### 1.3 `Growability`

| Variant | Role |
|--------|------|
| `Yes` | Growth mutation witnessed structurally. |
| `No` | No growth witnessed (including empty-use “no growth” case where axis is load-bearing). |
| `NotApplicable` | Axis does not apply to this binding’s shape (e.g. borrowed parameter path where growability is not in play — see `analyze.rs`). |

### 1.4 `Encoding`

| Variant | Role |
|--------|------|
| `Utf8FreeMonoidChar` | `.dag` `String` / `FreeMonoid<Char>` UTF-8 (Examples 3–4). |

Single-variant **stub** until LanguageSpec expands encoding vocabulary (`facts.rs` Practice 4 note).

### 1.5 Test-side / config — **not** substrate-declared axis sums

**`LanguageSpecAxes`** (`axes.rs`): `string_growability_axis_load_bearing: bool` — toggles whether the fold **requires** resolving `Growability` to `Yes`/`No` vs allowing analyzer shortcuts / different failure surfaces. It is **not** a declaration-ref into substrate; it mirrors intent that should eventually come from **LanguageSpec rows** projected from target specs (**#1130**).

**Diagnostics:** `EmissionDiagnostic::UnderRefined { axis: String }` uses **lowercase** axis labels `"growability"`, `"ownership"`, `"encoding"` (`analyze.rs`) — **stringly** bridge to substrate `unspecified_axis: String` (`diagnostics.dag`). This is exactly the vocabulary-risk the convergence audit flags for **target rows**: substrate axis **declarations** must not be confused with these diagnostic strings.

### 1.6 Ground-truth #1465 — `emit_model.dag` string-family axis sums

**Authority:** `module v3.std.emit_model` in **`src/v3/std/emit_model.dag`** ([**#1465**](https://github.com/gunb-ai/gunbc/issues/1465)). Comments in that file mark these sums as **terminal** at string-family diagnostic-ordering scope (and encoding scope for `StringEncodingAxis`). If your branch predates that merge, walk **`origin/main`**’s copy of the file (or the **#1465** diff) — the excerpt below matches **`main`** post-landing.

Exact structural shapes (labels as landed):

```text
type StringOwnershipAxis
  = Owned
  | Borrowed

type StringLifetimeAxis
  = SelfContained
  | Caller

type StringGrowabilityAxis
  = Growable
  | Fixed
  | NotApplicable

type StringEncodingAxis
  = Utf8FreeMonoidChar
```

**Namespace decision:** Sum **type names** are prefixed `String…Axis`, not the shared non-namespaced names the **#1442** sketch assumed. **Variant labels** intentionally align where they overlap with analyzer enums (`Owned`, `Borrowed`, `Caller`, `NotApplicable`, `Utf8FreeMonoidChar`) but **differ** where the substrate avoided leaking Rust’s `Self_` (`SelfContained` vs `Self_`) and where growability uses **capacity semantics** on the substrate side (`Growable` / `Fixed`) vs the analyzer’s **witness** vocabulary (`Yes` / `No`).

**Subset relation:** Treat **`StringOwnershipAxis` ⊆ hypothetical canonical `Ownership`** as **not** the right story — they are **parallel families**: analyzer `Ownership` classifies **program-wide** facts; `StringOwnershipAxis` classifies **string-family target/diagnostic-ordering** candidates in emit-model. Inclusion should be argued at the **bridge** (projection / fold), not as a DAG subtyping edge.

---

## 2. Landed substrate vocabulary vs prior sketch (#1442)

**Prior sketch (#1442):** One closed sum per axis with shared names (`Ownership`, `LifetimeScope`, `Growability`, `Encoding`) — optionally in `dsl/std/lifetime_axes.dag` or `src/v3/std/lifetime_axes.dag`.

**Landed reality (#1465):** String-family axes live **inside** `emit_model.dag` as **`StringOwnershipAxis`**, **`StringLifetimeAxis`**, **`StringGrowabilityAxis`**, **`StringEncodingAxis`**. They are **emit-model** facts today, not a separate `lifetime_axes` module.

**Implications:**

| Topic | Landed posture |
|--------|----------------|
| **Ownership / lifetime labels** | `Owned` / `Borrowed` match the spec; `SelfContained` / `Caller` match the intent of `Self_` / `Caller` with a substrate-friendly **Self** spelling. |
| **Growability labels** | Substrate: `Growable` / `Fixed` / `NotApplicable`. Analyzer: `Yes` / `No` / `NotApplicable`. **Not** a label-identical copy — any “lockstep” must admit a **mapping** (see Section 6). |
| **Encoding** | Single arm `Utf8FreeMonoidChar` on `StringEncodingAxis` — same stub shape as analyzer `Encoding` until LanguageSpec grows. |

**Dissolution / extension:** Additional arms still land as **substrate amendments** with P1 receipts — not ad hoc strings on rows.

---

## 3. Row reference shape (structural — no free-form lowercase strings)

**Risk (convergence audit):** Per-target rows that carry `borrowed`, `self`, `yes` as **untyped strings** diverge from analyzer **enum** spellings and invite parallel normalization tables in Grounding (**STOP → #1130**).

**Required pattern:**

- **Inhabitance / LanguageSpec rows** for **string-family** candidates should store **references** to **`String*Axis`** variant declarations, e.g. `DeclarationRef` resolved to `StringOwnershipAxis_Owned`, `StringGrowabilityAxis_NotApplicable`, etc. — analogous to `TypeRealization.target: DeclarationRef` in `emit_model.dag`.
- Where a row needs “this axis is constrained but unspecified”, that is **`UnderRefined`** / fold failure — **not** a fake row value.
- **Diagnostic surface** may remain string `unspecified_axis` until a follow-up slice types it; **row vocabulary** must not depend on matching those strings.

**Sketch:**

```text
data rust_string_family_candidate: StringFamilyInhabitanceRow = {
  target_type: DeclarationRef
  ownership: DeclarationRef   // resolves to StringOwnershipAxis::* arm
  lifetime_scope: DeclarationRef
  growability: DeclarationRef
  ...
}
```

Exact record names live with Substrate / LanguageSpec authoring (**#1130**).

---

## 4. `NotApplicable` and growability vocabulary divergence

**Requirement (#1440 / convergence audit):** **Absence ≠ “not applicable”.** Omitting a field cannot distinguish “does not apply” from “forgot to populate”.

**Spec:**

- **`StringGrowabilityAxis::NotApplicable`** (substrate) and **`Growability::NotApplicable`** (analyzer) are both **explicit** arms for “axis not in play” — keep that symmetry at any fold bridge.
- Rows that mean “growability does not apply” **must** reference the substrate **`NotApplicable`** arm, not omit the column.
- **Growability bridge:** When projecting analyzer facts into string-family rows, map **`Yes` → `Growable`**, **`No` → `Fixed`**, **`NotApplicable` → `NotApplicable`** (labels are **not** identical across sides — document the mapping in the bridge, not only in tests).

**Structural rationale (not an arbitrary relabeling):** Analyzer **`Growability`** records **what the fold witnessed** about growth-bearing mutations (`Yes` / `No`) versus **explicit non-applicability** (`NotApplicable`). Substrate **`StringGrowabilityAxis`** names **target-facing capacity disposition** for string-family rows (**`Growable`** / **`Fixed`**) plus the same explicit **`NotApplicable`**. The pairing is the **unique bijection between matching partitions**: witnessed growth ⇒ capacity may grow; witnessed absence of growth under a load-bearing axis ⇒ fixed capacity; axis out of scope ⇒ **`NotApplicable`** on both sides. If those semantics ever diverged (new analyzer arm or new substrate arm), the bridge table must change — the ratchet holds that table as **shared truth**, not a free permutation.

---

## 5. P1 receipts (refresh after partial landing)

Per **`INVARIANTS.md`** (substrate-fact introduction):

| Step | Application (updated) |
|------|------------------------|
| **1. DAG-ancestor** | **String-family** axis sums **landed** under **`v3.std.emit_model`** ([#1465](https://github.com/gunb-ai/gunbc/issues/1465)). A separate **`std.lifetime_axes`** module remains a **valid future** if the program decides to hoist **shared** non-namespaced sums — it is **not** required for the string slice that already lives in emit-model. Avoid **semantic** duplicates (two different spellings for the same row slot) without an explicit bridge doc — the **`String*Axis` names** are intentional **namespacing**, not an accident to “fold away” silently. |
| **2. Coproduct-vs-coordinate** | Each axis is a **sum** (one arm at a time). `LifetimeFacts`-shaped **records** in target rows are **coordinates** (`ownership`, `lifetime`, `growability`, `encoding`) — four fields, not four unrelated sum types at the top level without a parent record. |
| **3. Primitive-vs-lens-extensible** | **String-family** axis sums in substrate are **closed** (extend via amendment). **Encoding** may remain stub-small today; if future encodings are **language-spec extensible**, that extension path must be explicit (lens vs substrate). |

---

## 6. Lockstep ratchet (mirror of #1431 / `emission_diagnostic_lockstep`)

**Problem:** `grounding_lifetime/src/facts.rs` enums and `emit_model.dag` **`String*Axis`** sums are **parallel authorities** until codegen. The **#1442** ratchet sketch assumed **same type names** (`Ownership`, …) on both sides — **#1465** removed that shortcut.

**Chosen shape for today’s namespace split — label parity + explicit growability mapping (option b):**

A practical `lifetime_axes_lockstep` (name illustrative) should **not** assert `declaration_by_name("Ownership")` vs `pub enum Ownership` — those names **diverge**. Instead:

1. **Per overlapping axis**, assert that **variant identifiers that must align across the bridge** match between:
   - substrate `StringOwnershipAxis`, `StringLifetimeAxis`, `StringEncodingAxis` (and **`NotApplicable`** on `StringGrowabilityAxis`), and  
   - the corresponding analyzer enums in `facts.rs` (**`Owned`/`Borrowed`**, map **`Self_` ↔ `SelfContained`**, **`Caller`**, **`Utf8FreeMonoidChar`**, **`NotApplicable`**).
2. **Growability:** add a **small fixed mapping table** in the test (`Yes`↔`Growable`, `No`↔`Fixed`, `NotApplicable`↔`NotApplicable`) and assert both sides list exactly those pairs — i.e. **not** raw label equality for the growability axis. The table is the **documented bijection** from Section 4, not a disposable rename table.
3. **Implementation sketch:** `include_str!("../../grounding_lifetime/src/facts.rs")` plus bootstrap DAG (`generated_full_bootstrap_dag()` / `declaration_by_name` on `StringOwnershipAxis`, …); Cargo package **`v3-grounding-tests`**, path **`src/v3/grounding_tests/`** (hyphen vs underscore — same crate as `emission_diagnostic_lockstep.rs`). Wire `#[cfg(test)] mod lifetime_axes_lockstep;` in `src/v3/grounding_tests/src/lib.rs`.

**Why not (a) alone:** String-family-specific analyzer enums **do not exist** yet; waiting on them would block Slice 2. **Option (b)** still catches accidental drift in **shared** labels and forces the **growability** mapping to stay explicit — aligned with **snappy-koi-58** Slice 2 dispatch.

**Retirement:** When Rust enums are generated from `.dag`, delete mirror enums + textual ratchet (same narrative as `EmissionDiagnostic`).

---

## 7. Grounding recommendation (canonical shared vs string-namespaced)

**Question:** Does Grounding still need **one canonical shared** substrate vocabulary (`Ownership`, …), or is **per-axis string namespacing** sufficient if bridges stay disciplined?

**Recommendation:** Keep **analyzer facts program-wide** (`Ownership`, `LifetimeScope`, … in `facts.rs`). **Do not** “namespace-mirror” by renaming analyzer enums to `StringOwnership` — that would falsely imply string-only scope and is a **backward** step for non-string surfaces. For **string-family rows**, reference **`String*Axis`** in substrate and implement a **single projection layer** that maps analyzer facts → row refs (including **`Self_` → `SelfContained`** and growability **Yes/No → Growable/Fixed**).

If the program later **hoists** shared non-namespaced sums, treat that as a **substrate refactor** with explicit migration of `DeclarationRef` fields — not something this audit forces immediately.

---

## 8. Substrate Manager design call (#1130)

**Open design question (surface only — no decision here):** For **Substrate Manager** routing on **#1130**: should **`DeclarationRef` targets** carried by LanguageSpec / diagnostic-ordering / string-family rows continue to resolve to **per-family substrate sums** (**`String*Axis`**, emit-model authority post-#1465), with **program-wide analyzer enums unchanged** and **projection at the fold** — or should substrate introduce a **shared canonical sum layer** (non-namespaced `Ownership` / `Growability`-shaped declarations) that families **compose or view**, reducing duplicate sum names at the cost of migration and of **merging scopes** that today stay deliberately separate?

| Direction | Upside | Downside |
|-----------|--------|----------|
| **Per-family namespaced axes** (today) | Clear ownership of scope; avoids pretending string axes subsume all programs; analyzer lane stays one vocabulary for all surfaces. | More bridge tables; more names for reviewers to hold. |
| **Canonical + composed** | One declaration graph for “lifetime axes” across families. | Higher migration cost; risk of over-unifying program facts with target-only axes. |

Route the call on **#1130**; this doc only frames the trade-off for substrate planning. **Neither option requires renaming analyzer `Ownership` to `StringOwnership`** — the fork is **where substrate declarations live and how rows point at them**, not **collapsing the analyzer fact model into string-only names**.

---

## 9. Dispatch boundary

| Owner | Responsibility |
|-------|----------------|
| **Substrate Manager (#1130)** | Authors `.dag` axis sums + LanguageSpec / string-family row shapes that reference **`String*Axis`** structurally (and any future hoisted shared sums if adopted). |
| **T-Ground-Diagnostic / Grounding** | Consumes substrate facts; **does not** introduce a second lowercase normalization table; fails closed → **#1130** if rows ship without canonical refs. |
| **Lifetime-Analyzer lane** | Retires Rust mirrors when codegen/reflection exists; until then, lockstep tests hold **label/mapping** parity per Section 6. |

**Routing:** Cross-program escalation for vocabulary or row shape lives on **#1130** per convergence audit STOP discipline.

---

## References

- **Substrate (ground truth):** `src/v3/std/emit_model.dag` — `StringOwnershipAxis`, `StringLifetimeAxis`, `StringGrowabilityAxis`, `StringEncodingAxis` ([#1465](https://github.com/gunb-ai/gunbc/issues/1465))
- **Analyzer:** `src/v3/grounding_lifetime/src/facts.rs`, `axes.rs`, `analyze.rs`, `diagnostic.rs`
- `docs/audit/coercion-fold-lifetime-analyzer-convergence.md` (axis alignment table, canonical vocabulary prerequisite)
- `docs/design-emission-model.md` (Examples 3–4, growability / ownership derivation)
- `INVARIANTS.md` (substrate-fact introduction)
- `src/v3/grounding_tests/src/emission_diagnostic_lockstep.rs` (#1431 pattern)
- Dispatch / refresh context: [#1411](https://github.com/gunb-ai/gunbc/issues/1411), pattern precedent [#1454](https://github.com/gunb-ai/gunbc/issues/1454)
