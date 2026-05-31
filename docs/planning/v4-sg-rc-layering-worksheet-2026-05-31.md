# v4 SG-RC-LAYERING Worksheet — Per-use-site reference layering (`Rc` / `Box` / owned)

> **Status:** **PENDING §8** — Modeling DFS Manager sign-off (`proud-pike-680`). Worksheet authored 2026-05-31 (`sharp-lark-878`).
> **Date:** 2026-05-31
> **Dispatch anchor:** `docs/audit/v4-rustc-error-catalog-2026-05-31.md` §4.1 (~700 E0308 Rc/Box band, ~10% post-SG-1 residual); `#4086` routing; PR #3938 §10.0.
> **Canonical home:** `src/v4/std/target_model.dag` (`v4.std.target_model`) — same Option A placement as `docs/design-target-realization-canonical-home.md` §1 (extends realization carrier family; does **not** amend SG-1 worksheet).
> **Implementation lane:** Modeling DFS (worksheet + §8) → **Target Realization** (carrier + `extdeps/languages/rust.dag` rows) + **Compiler Spine** (`06_translate` boundary consumers).

---

## Mechanical dispatch rule

> **No SG-RC-LAYERING implementation worker may land until this worksheet is complete and Modeling DFS Manager–approved.**

Acceptance is **§10.6 bidirectional falsification + forbidden-register grep**, not E0308 count reduction on the M1 probe.

---

## §10.0-adapted worksheet

```text
SG class:               SG-RC-LAYERING
Representative emitted failure:
  // Return boundary — signature expects shared heap, body emits owned aggregate
  pub fn foo() -> Rc<Diagnostics> { Diagnostics { ... } }
  // rustc: expected `Rc<Diagnostics>`, found `Diagnostics` (277× in catalog band)

  // Layer mismatch — Box indirection vs Rc share at same boundary
  pub fn bar(x: Rc<Node>) -> Box<Rc<Node>> { ... }
  // rustc: expected `Box<_>`, found `Rc<Node>` (121×) / `expected `Node`, found `Rc<Node>`` (108×)

Immediate local patch:
  - In 06_translate: if emitted type text contains "Diagnostics" | "Node" | "ModelCore"
    wrap value expr with Rc::new(...) or strip Rc::new(...) heuristically per site.
  - Port v2 `build_shared_types` / `set_contains(shared_types, name)` name set into v4.
  - Fix only return types (leave parameters/fields) to shave E0308 count.
Why that patch is forbidden:
  - Name-keyed tables duplicate facts already carried on substrate carriers (INVARIANTS P2 /
    P3 — parallel authority; fabricated plausible output at sites the model never proved).
  - Context-split patches (return-only vs arg-only) preserve the disagreement that produced
    the catalog band — type-emit and value-emit keep deriving layering independently.
  - v2 `shared_types: Set<String>` is explicitly a bootstrap-era heuristic (05_emit_rust.dag
    RC2); re-porting it calcifies spelling-keyed sharing as v4 authority.
  - Does NOT scale to new carriers (TestClaim, FreeMonoid, Outcome, …) without emitter edits.
  - Conflates with SG-COLLECTION-PROJECTION (FreeMonoid vs Vec<Rc<T>>) — a different missing
    fact; must not be folded into this dispatch.
DFS path:
  std/ authority:
    - Diagnostic / Outcome / Witness carriers: src/v4/std/diagnostic.dag, witness.dag, …
    - Node / ModelCore / algebra carriers: src/v4/std/node.dag, model_core.dag, algebra.dag
    - FreeMonoid: src/v4/std/algebra.dag (collection spine — consumer of RC layer at boundaries,
      NOT authority for whether to wrap; SG-COLLECTION-PROJECTION owns monoid→Vec projection)
  extdeps/language authority:
    - extdeps/languages/rust.dag — per-carrier rows on TargetModel bundle (no parallel
      name-keyed shared_types map); Rc/Box surface tokens live in language tables only
  compiler stage consuming it:
    - src/v4/compiler/06_translate.dag — function signatures, parameters, struct fields,
      binding projections (type path + value path must consult the same row)
    - NOT v4.lens.ownership (T-13 dependency classification — analysis lens, not emit authority)
  existing scaffold/dissolution notes:
    - 🟡 sg-2-mvp1-projection-absent-shim on translate_node (06_translate.dag) — inner types
      inside Rc<T> must still route through SG-2 TargetTypeExpressionProjection before RC
      consumer lands; dissolve-on: delete dual translate paths when all TargetModel bundles
      carry type_expression_projection
    - v2 RC2 block in src/v2/05_emit_rust.dag:986+ documents legacy shared_types authority
      (reference for migration intent only — forbidden to copy verbatim into v4)
Deepest unsound boundary:
  Substrate declares *what* carriers exist (Node, Diagnostics, ModelCore, …) and SG-2 declares
  *how* to spell generic/applied types, but no single-authority fact states, per **use site**
  (function return, parameter, struct field, binding projection), whether the shipped Rust type
  is owned `T`, shared `Rc<T>`, or indirect `Box<T>`. Type-emit and value-emit therefore pick
  incompatible layers at the same boundary.
Systemic fix:
  TargetUseSiteOwnershipRealization — one row per (source_carrier, target_model, use_site):
    projects reference_layer: Owned | RcWrapped | BoxWrapped at that use site only.
  Inner `T` spelling comes from existing SG-2 TargetTypeExpression / atom rows — RC row wraps
  that projection; does NOT coin a parallel type vocabulary or a new Node.kind enum.
  Bundle edge on TargetModel (catalog): use_site_ownership_realizations.
  06_translate type + value paths resolve the row for each boundary before emit/serialize.
Non-goals:
  - Amending docs/planning/v4-sg1-target-atom-realization-worksheet-2026-05-30.md (SG-1-FOLLOWON /
    SG-1b are separate dispatch classes for String↔Symbol signature bands).
  - SG-COLLECTION-PROJECTION (~170 FreeMonoid vs Vec<Rc<T>> errors) in this PR.
  - SG-2 generic-arity / Instantiation work (E0107/E0282) — prerequisite consumer only.
  - Using v4.lens.ownership OwnershipFact / OwnershipMode as emit authority.
  - ci.dag / CI schema / Upsert migration.
  - M1 rustc residual reduction as primary acceptance metric.
  - Adding a new TargetTypeExprKind or Node.kind enum to encode “is Rc” (forbidden per §1).
Falsification probe:
  See §4 table — (F1)–(F6) are mandatory receipt rows for worker PROVEN.
Metric allowed only as secondary:
  ~700 E0308 in Rc/Box/raw band per docs/audit/v4-rustc-error-catalog-2026-05-31.md §4.1.
```

---

## §1 Single-authority fact

| Field | Value |
| ----- | ----- |
| **Fact name** | Per-use-site ownership realization at `Rc` / `Box` / owned boundaries |
| **Carrier (substrate)** | `TargetUseSiteOwnershipRealization` |
| **Canonical home** | `src/v4/std/target_model.dag` (`v4.std.target_model`) |
| **Bundle edge** | `target_model_edge_use_site_ownership_realizations` (name illustrative — worker lands exact symbol with bundle decode helpers co-located) |
| **Per-language rows** | `extdeps/languages/rust.dag` on each `TargetModel` bundle |
| **Primary consumer** | `src/v4/compiler/06_translate.dag` (type + value boundary paths) |
| **Explicitly NOT** | A new `Node.kind` enum, `TargetTypeExprKind` arm, or spelling-keyed `Set<String>` shared-types table |

### 1.1 Carrier sketch (substrate — not implementation in worksheet PR)

```dag
// v4.std.target_model — co-located with TargetAtomRealization / TargetTypeExpressionProjection /
// TargetCollectionRealization. Field names illustrative; shapes constrained by §10.6 + §3 gates.

type TargetOwnershipUseSite
  = OwnershipAtFunctionReturn
  | OwnershipAtFunctionParameter
  | OwnershipAtStructField
  | OwnershipAtBindingProjection
  // Coproduct of boundary *roles* — NOT a new Node.kind enum.

type TargetReferenceLayer
  = ReferenceLayerOwned
  | ReferenceLayerRc
  | ReferenceLayerBox

type TargetUseSiteOwnershipRealization {
  source_carrier: Node              // canonical carrier identity (Diagnostics node, ModelCore, …)
  target_model: TargetModel
  use_site: TargetOwnershipUseSite   // which boundary role this row governs
  reference_layer: TargetReferenceLayer
  // Inner type spelling: compose with SG-2 TargetTypeExpression (and SG-1 atom rows where
  // applicable) — do NOT duplicate type_form inside this row unless §8 ratifies a tighter join.
}
```

**Cross-section invariant:** `reference_layer` selects wrapping **around** the SG-2-projected inner type. SG-2 answers `Rc<Foo<X,Y>>` arity/spelling; SG-RC answers whether this **site** ships `Foo<…>`, `Rc<Foo<…>>`, or `Box<Foo<…>>`.

**Dissolve v2 heuristic:** any `shared_types` / `set_contains(shared_types, …)` / per-name `Rc::new` branch in translate touched by the worker must either consume `TargetUseSiteOwnershipRealization` or be deleted with a 🟡 dissolve-on citation to this worksheet.

---

## §2 Parser / substrate gates (dispatch prerequisites)

| Gate ID | Owner | Requirement before SG-RC worker starts | Unblock signal |
| ------- | ----- | -------------------------------------- | -------------- |
| **P-RC-WS** | Modeling DFS | This worksheet §8 **CLOSED** | proud-pike-680 sign-off |
| **P-RC-SG2-INNER** | Target Realization | `TargetTypeExpressionProjection` row present on Rust `TargetModel` bundle; inner `T` in `Rc<T>`/`Box<T>` from projection, not MVP1 shim-only path for touched carriers | `translate_node` uses `translate_node_with_projection` for those types OR explicit out-of-scope list in worker brief |
| **P-RC-SG1-BASELINE** | (met on main) | SG-1 #3956 landed — probe baseline `docs/audit/v4-rustc-error-catalog-2026-05-31.md` | E0423 class closed; RC band is separate |
| **P-RC-CARRIER** | Target Realization | `TargetUseSiteOwnershipRealization` + `TargetOwnershipUseSite` + `TargetReferenceLayer` parse in `target_model.dag`; bundle decode/lookup helpers | `cargo` parse of v4 std module (hermetic claim or compile receipt) |
| **P-RC-ROWS** | Target Realization | Rust rows for catalog representatives (`Diagnostics`, `Node`, `ModelCore`, `TestClaim`, `Outcome`, `FreeMonoid` carriers as named in §4.2) on `rust.dag` TargetModel | Rows address §4.1 top mismatch shapes |
| **P-RC-CONSUMER** | Compiler Spine | `06_translate` signature, parameter, field, and value coercions consult lookup — no ad-hoc wrap | Forbidden register §3 grep clean on touched files |
| **P-RC-BIDIR** | Runtime/TestClaim | §10.6: one manual claim changes `reference_layer` on a fixture row → **both** emitted type position and value position update | Claim transcript linked in PR |

**Parser gate discipline:** gates **P-RC-CARRIER** through **P-RC-BIDIR** are satisfied in the **implementation PR**, not in this worksheet PR. This document only names them so workers do not land substrate without consumers or falsification.

---

## §3 Spot-fix register (forbidden)

| Pattern | Where it shows up today | Why forbidden |
| ------- | ---------------------- | ------------- |
| `set_contains(shared_types, …)` / `build_shared_types` | v2 `05_emit_rust.dag` RC2 | Spelling-keyed parallel authority (P2); v4 must not re-port |
| `if type_name == "Diagnostics"` / `match name { "Node" => … }` wrap | translate template temptation | Name-keyed layering table |
| Return-type-only `Rc::new` patch without parameter/field rows | count-chasing | Preserves boundary disagreement; violates single fact |
| `Rc::new` / `Box::new` in `06_translate` without `TargetUseSiteOwnershipRealization` lookup | implementation smell | Spot-fix calcification |
| Using `v4.lens.ownership` `OwnershipFact` / `OwnershipMode` to choose `Rc` vs owned | lens/ownership.dag | Wrong authority plane — analysis, not target ship-type |
| New `Node.kind` / `TargetTypeExprKind` variant for “heap shared” | enum creep | Operator constraint: not a new kind enum |
| Steady-state `translate_node_mvp1` + projection path dual wrap rules | sg-2-mvp1 shim | Practice 4 — forbidden new ProjectionAbsent arms; must dissolve per existing 🟡 |
| Folding FreeMonoid→`Vec<Rc<T>>` projection into this PR | SG-COLLECTION-PROJECTION class | Different single-authority fact (~170 errors) |
| Amending SG-1 atom worksheet for signature String↔Symbol | SG-1-FOLLOWON / SG-1b | Separate dispatch per #4086 §3 |
| `ci.dag` / `CiUpsertStep` / workflow edits | out of scope | Explicit operator exclusion |
| M1 probe error-count delta as PR acceptance | probe | Evidence only (§10.0 metric rule) |

**Forbidden grep (implementation PR — literal strings on touched `06_translate` / new `target_model` consumer paths):**

```text
shared_types
build_shared_types
maybe_mark_shared_type
is_type_constant
set_contains(shared_types
"Diagnostics" =>   // name-keyed wrap arm (illustrative — any carrier-name string match table)
OwnershipFact       // as emit authority in translate (lens import forbidden)
OwnershipMode       // as emit authority in translate
translate_node_mvp1 // new match arms (dissolve only per existing gate)
```

Escalate to Modeling DFS: any gate in §2 cannot close without a new bundle edge shape; any forbidden grep hit; collision with SG-COLLECTION-PROJECTION requiring a single combined row (likely **no** — keep separate).

---

## §4 Falsification table (acceptance = all rows PASS)

| ID | Probe | Action | Pass criterion | Catalog tie-in |
| -- | ----- | ------ | -------------- | ---------------- |
| **F1** | Dual-boundary emit | On a fixture fn, change `Diagnostics` row `reference_layer` `RcWrapped` → `ReferenceLayerOwned` | Emitted **return type** and **return value expr** both drop `Rc<>` together (no mixed `Rc<Diagnostics>` / bare `Diagnostics`) | `expected Rc<Diagnostics>, found Diagnostics` (277) |
| **F2** | Cross-site consistency | Same carrier: return uses `RcWrapped`, parameter row `ReferenceLayerOwned` | Return still `Rc<…>`; param position bare `T`; neither site uses ad-hoc name table | `expected Node, found Rc<Node>` (108) vs param sites |
| **F3** | New carrier without emitter edit | Add `ProbeHeap` type + single `TargetUseSiteOwnershipRealization` row (`RcWrapped` at return only) | Emits `Rc<ProbeHeap>` at return with **no** new `if name == "ProbeHeap"` in translate | SG-2-style falsification |
| **F4** | Layer composition | Row `ReferenceLayerBox` on `Rc<Node>` boundary (where catalog shows `Box<Rc<Node>>` vs `Rc<Node>` mismatch) | Type and value both use `Box<…>` wrapping per row — not hardcoded in one path only | `expected Box<_>, found Rc<Node>` (121) |
| **F5** | Forbidden register | Run §3 grep on PR diff scope | Zero hits except 🟡-cited interim lines with dissolve-on to this worksheet | Mechanical anti-spot-fix |
| **F6** | Bidirectional read (§10.6) | Ingest emitted Rust signature for one fixture; decode to `TargetUseSiteOwnershipRealization` | Decoded `reference_layer` matches authored row; round-trip does not require parallel string parser | Close/Receipt vocabulary |

**Secondary evidence (not PASS):** M1 probe `scripts/v4-m1-rust-emit-probe.sh` Rc-band histogram movement toward §4.1 subtotal ~700.

---

## §4.1 Catalog mismatch coverage (worker row checklist)

Rows must cover at least these **source_carrier** families (Rust TargetModel catalog — additive only):

| Priority | Carrier family | Top catalog shape | ~Count |
| -------- | -------------- | ----------------- | -----: |
| P0 | `Diagnostics` | `Rc<Diagnostics>` vs `Diagnostics` | 277 |
| P0 | `Node` | `Rc<Node>` vs `Node` / `Box` vs `Rc` | 229 |
| P1 | `TestClaim` | `Rc<TestClaim>` vs `TestClaim` | 45 |
| P1 | `FreeMonoid<_>` | `Rc<FreeMonoid<_>>` vs `FreeMonoid` | 31 |
| P1 | `Outcome<_>` | `Rc<Outcome<_>>` vs `Outcome` | 20 |
| P2 | `ModelCore`, `AlgebraInhabitanceDecl`, tail | per §4.1 | ~66 |

Worker brief may land **P0 slice** first if full catalog is too large; slice must list covered carriers and falsification **F1–F6** hold within slice with explicit out-of-scope carriers named.

---

## §5 Landing order (implementation — not this PR)

```text
1. Modeling DFS §8 sign-off on this worksheet (P-RC-WS).
2. Target Realization — carrier + bundle edge + lookup API in v4.std.target_model (P-RC-CARRIER).
3. Target Realization — Rust TargetModel rows for §4.1 carriers (P-RC-ROWS); depends on SG-2 inner
   projection for wrapped types (P-RC-SG2-INNER).
4. Compiler Spine — 06_translate boundary consumer refactor (P-RC-CONSUMER); delete ad-hoc wrap on
   touched paths; dissolve sg-2-mvp1 dual path only per existing 🟡 when bundle complete.
5. Runtime/TestClaim — manual claim for F6 + F1 (P-RC-BIDIR).
6. Optional: M1 probe refresh as secondary evidence (Close/Receipt lane — not worker acceptance).
```

**Parallelism (operator #4094 decision 3):** SG-1-FOLLOWON / SG-1b may proceed in parallel — different missing fact (signature `String` vs `Symbol`). SG-COLLECTION-PROJECTION remains **route-pending** / extend SG-5/SG-6 unless Modeling DFS escalates a new worksheet.

**Lane split:** Target Realization owns §5 steps 2–3; Compiler Spine owns step 4; Runtime/TestClaim owns step 5 falsification transcript.

---

## §6 Downstream worker brief (dispatch downstream — no code in worksheet PR)

```text
Implement SG-RC-LAYERING per approved worksheet.

MUST:
  - Add TargetUseSiteOwnershipRealization (+ use_site + reference_layer coproducts) ONCE in
    v4.std.target_model; per-language rows on rust TargetModel bundle only.
  - Refactor 06_translate type + value boundary emission to consult row lookup per use_site.
  - Compose inner types via SG-2 TargetTypeExpressionProjection (and SG-1 atoms where applicable).
  - Pass falsification F1–F6 (§4 table).
  - Include variant-shape histogram for new coproducts (v4-substrate-pr-review-gate §3).

MUST NOT:
  - Any §3 forbidden pattern or grep hit (uncited).
  - Amend v4-sg1-target-atom-realization-worksheet-2026-05-30.md.
  - Touch ci.dag / ci.yml / shell gates.
  - Claim SG-COLLECTION-PROJECTION closure in this PR.
  - Use M1 error-count reduction as acceptance.

Escalate to Modeling DFS:
  - Need for spelling-keyed shared-types set after landing.
  - Need to merge RC layering with FreeMonoid→Vec projection in one row.
  - New TargetOwnershipUseSite arm without worksheet amendment.
```

---

## §7 Non-goals

- SG-1 atom worksheet amendment; SG-1b / SG-1-FOLLOWON signature bands
- SG-COLLECTION-PROJECTION (~170 errors) — separate dispatch
- SG-2 / SG-8 / SG-3-CASCADE primary work (may share PR only with explicit Modeling DFS ack)
- `ci.dag`, Upsert migration, structural-bridge replacement
- Implementation substrate rows or `06_translate` edits in the **worksheet-only** PR

---

## §8 Manager approval checklist (`proud-pike-680`) — OPEN

- [ ] Single-authority fact: `TargetUseSiteOwnershipRealization` in `v4.std.target_model` (per use site, not kind enum)
- [ ] Distinct from `v4.lens.ownership` (analysis) and from SG-COLLECTION-PROJECTION (monoid→Vec)
- [ ] Spot-fix forbidden: name-keyed `shared_types`, return-only wrap, lens-as-emit-authority
- [ ] Parser gates §2 named; implementation gates not confused with worksheet PR scope
- [ ] Falsification table §4 (F1–F6) accepted as worker acceptance
- [ ] Forbidden register §3 + grep literals accepted
- [ ] Landing order §5 + lane split (TR vs Compiler Spine vs TestClaim)
- [ ] **READY-FOR-WORKER-DISPATCH** after checkboxes closed

---

## Related artifacts

- `docs/audit/v4-rustc-error-catalog-2026-05-31.md` — §4.1 population table, repro §7
- `docs/planning/v4-correctness-ladder-2026-05-30.md` — §10.0 template, §10.6 bidirectionality
- `docs/design-target-realization-canonical-home.md` — Option A placement (extend `target_model.dag`)
- `docs/planning/v4-sg2-type-expression-projection-worksheet-2026-05-30.md` — inner type prerequisite
- `docs/planning/v4-predicate-dependency-graph-2026-05-31.md` — P3 elastic core routing
- `docs/planning/v4-substrate-pr-review-gate-2026-05-30.md` — coproduct histogram on implementation PR
- `src/v2/05_emit_rust.dag` — RC2 `shared_types` legacy (migrate intent, forbidden copy)
