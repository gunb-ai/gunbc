# Canonical lifetime-axes substrate vocabulary — pre-audit (string-family row landing)

**Status:** PRE-AUDIT (doc-only; no `.dag` authority in this file)  
**Dispatch:** T-Ground-Diagnostic axis-vocabulary substrate (Director Option 3), post-#1438 / convergence with `docs/audit/coercion-fold-lifetime-analyzer-convergence.md` and **#1130** (Substrate Manager).  
**Parallel pattern:** `src/v3/std/integer_diagnostic_order.dag` (T-Ground-Diagnostic slice 2) — structural facts in substrate + lockstep ratchet for lane-local Rust until codegen.

This document **specs** the substrate slice that should land before per-target string-family rows reference axis values **structurally** (single authority). **Substrate Manager** authors the `.dag`; **Grounding** consumes and retires mirrors.

---

## 1. Current axis vocabulary at HEAD (lane-local Rust)

**Authority today:** `src/v3/grounding_lifetime/src/facts.rs` defines program-side `LifetimeFacts` and per-axis closed sums. Docstrings cite P1 posture (`INVARIANTS.md` §P1) and `design-emission-model.md` Examples 3–4 / §635 for R3 deferrals.

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

**Diagnostics:** `EmissionDiagnostic::UnderRefined { axis: String }` uses **lowercase** axis labels `"growability"`, `"ownership"`, `"encoding"` (`analyze.rs`) — **stringly** bridge to substrate `unspecified_axis: String` (`diagnostics.dag`). This is exactly the vocabulary-risk the convergence audit flags for **target rows**: substrate canonical axis **declarations** must not be confused with these diagnostic strings.

---

## 2. Substrate-declared canonical vocabulary (sketch — **do not author here**)

**Goal:** One closed sum per axis in substrate, shared by:

- Lifetime-Analyzer (eventually reflected or codegen’d Rust),
- LanguageSpec / string-family **inhabitance** rows,
- Any diagnostic projection that names an axis **structurally**.

### 2.1 Placement options (same tradeoff as slice 2)

| Option | Pros | Cons |
|--------|------|------|
| **`dsl/std/lifetime_axes.dag`** (new module, e.g. `std.lifetime_axes`) | Lives next to other **dsl** conceptual types; std bootstrap seed loads without `v3.spec.v3_l1`. | If rows need `DeclarationRef`-typed fields pointing at **these** variants from **v3** specs, ensure load order / imports allow resolution. |
| **`src/v3/std/lifetime_axes.dag`** | Same pattern as `integer_diagnostic_order.dag` when sentinel types (`DeclarationRef`) or cross-links to emit_model require **v3** module boundaries. | Must appear in staged `src/v3/std/*.dag` bundle. |

**Likely shape (illustrative names):**

```text
module std.lifetime_axes   // or v3.std.lifetime_axes

type Ownership = Owned | Borrowed

type LifetimeScope = Self_ | Caller   // dag spelling TBD vs Rust keyword `Self`

type Growability = Yes | No | NotApplicable

type Encoding = Utf8FreeMonoidChar   // grows when LanguageSpec adds encodings
```

**Naming:** Substrate variant **labels** should be stable identifiers (PascalCase or a single naming convention locked in the landing PR). Rust keyword escapes (`Self_`) remain **Rust-only**; `.dag` may use `SelfScoped` / `ModuleSelf` — **one** canonical substrate label must be chosen and mapped at codegen.

**Dissolution / extension:** Additional arms (`Conditional`, `Source`, …) land as **substrate amendments** with P1 receipts — not ad hoc strings on rows.

---

## 3. Row reference shape (structural — no free-form lowercase strings)

**Risk (convergence audit):** Per-target rows that carry `borrowed`, `self`, `yes` as **untyped strings** diverge from analyzer **enum** spellings and invite parallel normalization tables in Grounding (**STOP → #1130**).

**Required pattern:**

- **Inhabitance / LanguageSpec rows** that describe program or target facts on these axes should store **references** to the substrate variant declarations, e.g. fields typed `DeclarationRef` resolved to `Ownership_Owned`, `Growability_NotApplicable`, etc. — analogous to `TypeRealization.target: DeclarationRef` in `emit_model.dag`.
- Where a row needs “this axis is constrained but unspecified”, that is **`UnderRefined`** / fold failure — **not** a fake row value.
- **Diagnostic surface** may remain string `unspecified_axis` until a follow-up slice types it; **row vocabulary** must not depend on matching those strings.

**Sketch:**

```text
data rust_string_family_candidate: StringFamilyInhabitanceRow = {
  target_type: DeclarationRef
  ownership: DeclarationRef   // resolves to Ownership::* arm
  lifetime_scope: DeclarationRef
  growability: DeclarationRef
  ...
}
```

Exact record names live with Substrate / LanguageSpec authoring (**#1130**).

---

## 4. `NotApplicable` as load-bearing fact

**Requirement (#1440 / convergence audit):** **Absence ≠ “not applicable”.** Omitting a field cannot distinguish “does not apply” from “forgot to populate”.

**Spec:**

- **`Growability::NotApplicable`** exists as an **explicit** substrate variant (sum arm), not as “missing `growability` column”.
- Rows that mean “growability does not apply to this candidate/binding” **must** reference `Growability_NotApplicable` (name illustrative).
- Analyzer logic that today maps borrowed parameters to `Growability::NotApplicable` (`analyze.rs`) remains valid; reflected substrate should preserve that **explicit** arm when projecting facts to rows or diagnostics.

---

## 5. P1 receipts (sketch — when substrate lands)

Per **`INVARIANTS.md` §94–129**:

| Step | Application |
|------|-------------|
| **1. DAG-ancestor** | Axis sums attach under the **lifetime / string-family modeling** branch — either extend an existing parent module (e.g. alongside `emit_model` carriers) or introduce **`std.lifetime_axes`** as the named home for **closed vocabulary** shared by LanguageSpec + analyzer. Do **not** spawn parallel `StringOwnership` / `RustOwnership` sibling types for the same concept. |
| **2. Coproduct-vs-coordinate** | Each axis is a **sum** (one arm at a time). `LifetimeFacts`-shaped **records** in target rows are **coordinates** (`ownership`, `lifetime`, `growability`, `encoding`) — four fields, not four unrelated sum types at the top level without a parent record. |
| **3. Primitive-vs-lens-extensible** | **Ownership / LifetimeScope / Growability** for Shape-A string-family analysis are **substrate-primitive** closed sums (extend only via substrate amendment). **Encoding** may remain stub-small today; if future encodings are **language-spec extensible**, that extension path must be explicit (lens vs substrate) — today’s single-arm `Encoding` is closed until LanguageSpec grows. |

---

## 6. Lockstep ratchet (mirror of #1431 / `emission_diagnostic_lockstep`)

**Problem:** After substrate declares `Ownership`, `LifetimeScope`, `Growability`, `Encoding` sums, `grounding_lifetime/src/facts.rs` enums become **parallel-authority mirrors** (same failure mode as lane-local `EmissionDiagnostic`).

**Spec:**

1. **Add** `v3-grounding-tests/src/lifetime_axes_lockstep.rs` (name illustrative):
   - `include_str!("../../grounding_lifetime/src/facts.rs")` (or a minimal excerpt listing `pub enum` variant names only if file grows).
   - Bootstrap `Dag::new()` / `generated_full_bootstrap_dag()`: for each substrate sum type name (`Ownership`, …), read `TypeConnective::Disj` variant labels from `declaration_by_name`.
   - **Assert:** mirror enum variant identifiers ⊆ substrate variant labels for each axis sum (same subset discipline as `emission_diagnostic_lockstep.rs`).
2. **Negative test:** synthetic mirror-only variant name fails ratchet (optional — mirrors `emission_diagnostic_lockstep` negative case).
3. **Wire** `#[cfg(test)] mod lifetime_axes_lockstep;` in `grounding_tests/src/lib.rs`.

**Retirement:** When Rust enums are generated from `.dag`, delete mirror enums + textual ratchet (same narrative as `EmissionDiagnostic`).

---

## 7. Dispatch boundary

| Owner | Responsibility |
|-------|------------------|
| **Substrate Manager (#1130)** | Authors canonical `.dag` axis sums + LanguageSpec / string-family row shapes that reference them structurally. |
| **T-Ground-Diagnostic / Grounding** | Consumes substrate facts; **does not** introduce a second lowercase normalization table; fails closed → `#1130` if rows ship without canonical refs. |
| **Lifetime-Analyzer lane** | Retires Rust mirrors when codegen/reflection exists; until then, lockstep tests hold parity. |

**Routing:** Cross-program escalation for vocabulary or row shape lives on **#1130** per convergence audit STOP discipline.

---

## References

- `src/v3/grounding_lifetime/src/facts.rs`, `axes.rs`, `analyze.rs`, `diagnostic.rs`
- `docs/audit/coercion-fold-lifetime-analyzer-convergence.md` (axis alignment table, canonical vocabulary prerequisite)
- `docs/design-emission-model.md` (Examples 3–4, growability / ownership derivation)
- `INVARIANTS.md` §P1 (substrate-fact introduction)
- `src/v3/grounding_tests/src/emission_diagnostic_lockstep.rs` (#1431 pattern)
