# T-Ground-Rust — Full Rust target-primitive implementation

**Status:** PROPOSAL — dispatchable when **PR-F** (Q1 `BoundDeclaration` consumer + Q2 Rust structural axes via `ReferenceModel<T>`) merges. Authored 2026-05-05 ahead of PR-F to keep the lane queue warm; consistent with `r2-grounding-manager.md:142` ("T-Ground-Rust full implementation (gated on PR-F)") and the manager's directive that brief authoring is the only Day-1-ready Grounding item once host git is restored. Implementation gated on PR-F / PR-I / Substrate `HigherOrderMethodSpec` shape decision per the dependency table below; no code lands until each named gate clears AND host git is restored AND the manager re-authorizes dispatch.

**Lane:** T-Ground-Rust (XL) — item 2 of 11 in [`r2-grounding-manager.md`](r2-grounding-manager.md) (line 28 + lane row line 63).

**Manager:** R3 Grounding Manager ([`r2-grounding-manager.md`](r2-grounding-manager.md); lane authority continuous through R2→R3 topology shift).

**Lineage / authorities consumed (no re-litigation):**
- Engine-reframe spec: [`docs/design-emission-model.md`](../design-emission-model.md) — Q1 `BoundDeclaration` (lines 994-1067), Q2 `ReferenceModel<T>` (lines 1068-1117), Q3 `RealizationCost` (lines 1143-1208), Q4 universal four-property gate (line 1234), cadence table (lines 1271-1282).
- Two-authority discipline: `grounding-manager.md:60-74` — **(a) Rust Reference §Types** authority (<https://doc.rust-lang.org/reference/types.html>) for language-level structural types; **(b) std-library carriers** authority (std documentation) for `String` / `Vec<T>` / `Box<T>` / `Rc<T>` / `Arc<T>` / `HashMap<K,V>` / `BTreeMap<K,V>` / `HashSet<T>` / `BTreeSet<T>` / `Option<T>` / `Result<T, E>`. Each per-primitive row cites its own authority — mixing is a faithfulness violation per `r2-grounding-manager.md:124` (Q4 four-property gate).
- THESIS: `THESIS.md:171` — "Coercion = emission. No separate coercion engine." This lane lands the substrate facts the fold reads; it does NOT introduce selection logic.
- Substrate-fact-introduction: [`INVARIANTS.md`](../../INVARIANTS.md) §P1 (3-step procedure for every new field on `RustPrimitive` or sibling per-primitive carriers).
- Sibling brief shape: [`t-ground-languagespec.md`](t-ground-languagespec.md).
- Pilot precedent: [`grounding-pilot-receipt.md`](grounding-pilot-receipt.md) (variant-aware partition `IntegerPrimitive | NonIntegerPrimitive` locked per codex P2 adjudication 2026-04-25; widening to flat record out of scope).

---

## Framing question this lane answers

Does Rust's full target-primitive surface (Rust Reference §Types + std-library carriers) declare structurally in `.dag` against `BoundDeclaration` + `ReferenceModel<T>` + `RealizationCost` with each primitive citing its own authority — so that the inhabitance walk `src/v3/grounding_engine/src/lib.rs:59` can validate every Rust primitive without falling back to `dsl/extdeps/languages/rust/types.dag` table lookup?

A "yes" populates the substrate that Coercion-Fold reads, retires the table-driven scaffolding under T-Ground-Dissolve, and produces the structural acceptance gate `rust_target_primitives_declared_structurally` (per `r2-grounding-manager.md:124`).

A "no" — or any discovery that scope is mis-modeled / authorities have drifted / the two-authority split forces a substrate distinction the substrate doesn't carry — escalates to manager per the discipline reminders below; do NOT paper over.

---

## Scope

### A. Rust Reference §Types primitives — structural declarations against authority (a)

Author per-primitive structural rows in `dsl/extdeps/languages/rust/primitives.dag` (extending the existing `RustPrimitive` type at line 180 + `rust_pilot_primitives` data at line 208). Authority: <https://doc.rust-lang.org/reference/types.html>. Coverage required:

- **Numeric — integer family.** Pilot already covers `i8`–`i64`, `u8`–`u64`, plus `i128` (LANDED — manager correction 2026-05-05; `i128` exists in `primitives.dag` and the pilot mirror at HEAD). Remaining: `u128`, `isize`, `usize`. The `isize`/`usize` rows consume Q1's `PlatformDependent` variant of `BoundDeclaration` (`src/v3/std/substrate.dag:136`); this is the first non-pilot exercise of `PlatformDependent` and validates PR-F's Q1 consumer end-to-end.
- **Numeric — floating-point family.** `f32`, `f64`. New `RustPrimitive` variant or refinement of `NonIntegerPrimitive` per the variant-aware partition lock (`grounding-pilot-receipt.md`). Algebra inhabitance is **not** `OrderedRing` / `Semiring` (IEEE-754 fails ring axioms — NaN, signed zero, non-associative addition); per Modeling problem 2 corrected, model the structural axis (a `FloatAlgebra` carrier or equivalent under `dsl/std/algebra.dag`) rather than mis-claiming ring inhabitance. Authority cited: Rust Reference §Floating-point types + IEEE-754.
- **Textual.** `char` (32-bit Unicode scalar value) and `str` (UTF-8 byte sequence, dynamically sized). The `str` row carries an encoding axis (UTF-8) per Q2 `ReferenceModel<T>` axis discipline; dynamically-sized status is a structural axis, not a separate primitive.
- **Never.** `!` — uninhabited. Algebra inhabitance is universal-bottom (the empty type inhabits every algebra trivially via vacuous quantification). Authority: Rust Reference §Never type.
- **Tuple.** Variadic structural product. Each tuple arity is not a separate primitive; the row declares the tuple constructor with arity as a refinement axis. Unit `()` is the 0-arity tuple — already in pilot.
- **Array.** `[T; N]` — fixed-cardinality structural product. Cardinality is `Interval<Cardinal>::BoundedInterval { lower: N, width: IntervalWidth::ZeroWidth }` per the Q5 collapse note (`design-emission-model.md:1267`) consumed against HEAD's `Interval<D>` shape (`src/v3/std/substrate.dag:123`).
- **Slice.** `[T]` — dynamically-sized structural product. Cardinality `Interval<Cardinal>::Unbounded`. Authority axis: dynamically-sized.
- **Struct / enum / union — base shapes.** Per `r2-grounding-manager.md:265` ("Struct / enum / union primitives"). These are *constructor schemas*, not concrete primitives; the row declares the structural shape (named-field record / tagged sum / untagged union with safety-required-by-construction).
- **Function item / function pointer / closure.** `fn(...) -> ...` (function item, zero-sized), `fn(...) -> ...` (function pointer, pointer-sized), closure (anonymous, captures-bearing). Three structurally distinct rows; the closure row carries a captures axis.
- **Reference.** `&T` (shared, immutable, lifetime-bounded), `&mut T` (exclusive, mutable, lifetime-bounded). Both inhabit `ReferenceModel<T>` with axes (`mutability`, `lifetime`) populated; ownership axis is `Borrowed`. Lifetime is **structural, not annotation-driven** per T-Ground-Lifetime-Analyzer authority (LANDED #1206 / #1218 / #1220) — this lane consumes the lifetime axis as substrate, does NOT re-author it.
- **Raw pointer.** `*const T`, `*mut T`. `ReferenceModel<T>` axes (`mutability`, `safety`); ownership axis is `Raw`; no lifetime.
- **Trait object.** `dyn Trait` — dynamically-sized, vtable-bearing. Authority: Rust Reference §Trait object types.
- **`impl Trait`.** Existential-position type; the row carries an opaqueness axis. Authority: Rust Reference §Impl Trait.

Each primitive row cites its **authority URL** (Rust Reference section or std-doc URL) in a comment adjacent to the row per the brief-authoring-checklist convention.

### B. std-library carriers — structural declarations against authority (b)

Authority: <https://doc.rust-lang.org/std/> per type. Coverage required:

- **`String`** — owned, growable, UTF-8 byte buffer. Inhabits `FreeMonoid<Char>` via UTF-8 encoding axis. Ownership: `Owned`; growability: `Growable`; encoding: `UTF-8`.
- **`Vec<T>`** — owned, growable, contiguous heap buffer. Cardinality: `Interval<Cardinal>::Unbounded`. Ownership: `Owned`; growability: `Growable`.
- **`Box<T>`** — single-owner heap pointer. `ReferenceModel<T>` ownership axis: `Owned`; no lifetime; safety: `Safe`.
- **`Rc<T>`** — shared-ownership reference-counted pointer (single-threaded). `ReferenceModel<T>` ownership: `SharedRefCounted { thread_safe: false }`.
- **`Arc<T>`** — shared-ownership atomic-reference-counted pointer (thread-safe). `ReferenceModel<T>` ownership: `SharedRefCounted { thread_safe: true }`.
- **`HashMap<K, V>`** — hash-table-backed associative array. Inhabits `PartialFunction<K, V>` (`dsl/std/algebra.dag:428`); ordering axis: `None`.
- **`BTreeMap<K, V>`** — B-tree-backed ordered associative array. Inhabits `PartialFunction<K, V>`; ordering axis: `Sorted` (consumes `K: Ord`).
- **`HashSet<T>`**, **`BTreeSet<T>`** — set carriers; structurally a degenerate `PartialFunction<T, Unit>` or a sibling set carrier per the apparent-multi-inhabitance audit (LanguageSpec scope item F; this lane resolves the Rust slice).
- **`Option<T>`** — sum carrier `Some(T) | None`. Inhabits the optional-value algebra; sum-shape per Q1 receipt (Step 2 coproduct).
- **`Result<T, E>`** — sum carrier `Ok(T) | Err(E)`. Same partition.

**Two-authority discipline:** A and B rows cite distinct authority URLs. If row layout in a single `primitives.dag` file would mix authorities ambiguously (i.e., a reader can't tell which row claims which authority), split into `dsl/extdeps/languages/rust/primitives.dag` (authority a) + `dsl/extdeps/languages/rust/std_carriers.dag` (authority b). This split is **a discovered necessity, not a default** — author in one file first; split only if the faithfulness review surfaces the ambiguity. Escalate the split decision to manager before landing.

### C. `RustPrimitive` type extension + grounding-engine walker arms

The current `RustPrimitive` partition (`primitives.dag:180`) is `IntegerPrimitive { algebra: IntegerAlgebra, .. } | NonIntegerPrimitive { algebra: NonIntegerAlgebra, .. }`. Per `grounding-pilot-receipt.md` lock, the partition is fixed-shape; widening to flat record is out of scope. New variants required for full coverage:

- `FloatPrimitive { algebra: FloatAlgebra, .. }` — distinct from `NonIntegerPrimitive` because IEEE-754's algebra is structurally distinct from Bool/Unit.
- `TextualPrimitive { encoding: Encoding, sized: SizedKind, .. }` — for `char` / `str`.
- `NeverPrimitive { .. }` — empty-type marker.
- `CompoundPrimitive { kind: CompoundKind, .. }` — tuple / array / slice / struct / enum / union shapes. CompoundKind is a sub-sum; per Step 2 receipt, these alternate (a value is a tuple OR an array, never both), so sum-shape is correct.
- `FunctionPrimitive { kind: FunctionKind, .. }` — function item / fn pointer / closure.
- `ReferencePrimitive { model: ReferenceModel<T>, .. }` — references / raw pointers / `Box` / `Rc` / `Arc` (B's pointer-family carriers re-home here per A's `ReferenceModel<T>` axis).
- `TraitObjectPrimitive { .. }`, `ImplTraitPrimitive { .. }`.
- `ContainerPrimitive { algebra: ContainerAlgebra, .. }` — `Vec` / `String` / `HashMap` / `BTreeMap` / `HashSet` / `BTreeSet` / `Option` / `Result` (B's non-pointer carriers).

Walker arms in `src/v3/grounding_engine/src/lib.rs:59` (`validate_loaded_rust_primitive_type_structure`) extend with a match arm per new `RustPrimitive` variant. Each arm returns `StructureMismatch` on shape violation (per the existing pattern); no new error variants introduced (typed `EmissionDiagnostic` is T-Ground-Diagnostic scope).

The `src/v3/grounding_pilot/src/lib.rs:106` mirror updates in lockstep until T-Ground-LanguageSpec retires it (Reflective Pattern E retirement scope item D in `t-ground-languagespec.md`); this lane keeps the mirror consistent during expansion, does NOT retire it.

### D. Q1 `BoundDeclaration` consumer wiring

Each integer primitive row consumes `BoundDeclaration` (`src/v3/std/substrate.dag:136`) per Q1 lock (`design-emission-model.md:994`). HEAD's substrate shape (`src/v3/std/substrate.dag:123-138`) is `Interval<D> = BoundedInterval { lower: D, width: IntervalWidth } | Unbounded` and `BoundDeclaration = StaticBound(Interval<Int>) | PlatformDependent`. Asymmetric match rule (Q1 lock per `r2-grounding-manager.md:56`) phrased against the landed shape: target's `Unbounded` universally accepts; target's `StaticBound(BoundedInterval { lower, width })` requires structural equality on the `(lower, width)` payload (`PlatformDependent` remains a distinct outer variant — it never collapses into `StaticBound`).

- `i8`–`i128` rows: `StaticBound(BoundedInterval { lower: -2^(N-1), width: PositiveWidth(2^N - 1) })`.
- `u8`–`u128` rows: `StaticBound(BoundedInterval { lower: 0, width: PositiveWidth(2^N - 1) })`.
- `isize` / `usize` rows: `PlatformDependent`. **First non-pilot exercise of the `PlatformDependent` path.**

If walking PR-F's locked `BoundDeclaration` consumer surface surfaces a structural mismatch with the Rust integer family's actual ranges (e.g., the `Interval<Int>` carrier can't express `2^127 - 1` as a literal because `Int` lowering hasn't settled), STOP and escalate — that is a substrate-shape escalation to Substrate Manager via the manager (`#1745`), not a paper-over.

### E. Q2 `ReferenceModel<T>` axes — Rust per-family axis declarations

Per Q2 lock (`design-emission-model.md:1100`) and `r2-grounding-manager.md:90`: pointer/reference family shares the `ReferenceModel<T>` parametric parent with axes (`lifetime`, `mutability`, `ownership`, `representation`). Rust's pointer-family rows declare which axis combinations they cover:

- `&T`: `{ ownership: Borrowed, mutability: Immutable, lifetime: Bound }`.
- `&mut T`: `{ ownership: Borrowed, mutability: Mutable, lifetime: Bound }`.
- `*const T`, `*mut T`: `{ ownership: Raw, mutability: { Immutable | Mutable }, lifetime: None, safety: Unsafe }`.
- `Box<T>`: `{ ownership: Owned, mutability: { Immutable | Mutable } via container, lifetime: None }`.
- `Rc<T>`: `{ ownership: SharedRefCounted { thread_safe: false }, lifetime: None }`.
- `Arc<T>`: `{ ownership: SharedRefCounted { thread_safe: true }, lifetime: None }`.

The `ReferenceModel<T>` shared-parent declaration itself lives in `dsl/std/` (substrate-owned per Q2 lock; PR-F lands it). This lane consumes; if PR-F's landed shape doesn't accommodate Rust's coverage as enumerated, STOP and escalate.

### F. Q3 `RealizationCost` per-primitive population

Per Q3 lock (`design-emission-model.md:1143`) — gated on **PR-I** which is downstream of PR-F. Each Rust primitive row attaches `RealizationCost { storage: Cost<Bits>, access: Map<AlgebraOp, Cost<CPUCycles>> }`. Sparse fail-closed (`design-emission-model.md:1206`) — missing op = `Witness.Violates`; no silent zero-cost.

**This may slice separately from A/B/C/D/E** if PR-I lands later than PR-F. Manager guidance (sibling brief precedent: `t-ground-languagespec.md` Phase 1 / Phase 2 split): land A-E as Phase 1 once PR-F merges; land F as Phase 2 once PR-I merges. Bundle into one PR if both gates have cleared at dispatch time.

### G. Higher-order MethodTemplateContract rows (Phase 1.5 — separately gated)

Per `r2-grounding-manager.md:67` Phase 1.5: Rust higher-order rows are gated on Substrate's shape decision for the existing dual-template `HigherOrderMethodSpec` carrier (`dsl/extdeps/languages/rust/emit.dag:265`); cross-manager request live to jolly-ram-908 (#1130). **OUT OF SCOPE for this lane's primary slice**; surfaced here so the worker doesn't accidentally absorb it. If higher-order Rust rows are needed by a primitive declaration this lane authors (e.g., `Vec<T>::map` requires a higher-order spec), STOP and escalate — likely the substrate-shape decision hasn't landed and this lane is blocked on it.

### H. P1 procedure receipts

Per `INVARIANTS.md` §P1 (lines 94-129), worker MUST cite receipts in the PR body for every new variant of `RustPrimitive` and every new field on existing variants:

- **Step 1 (DAG-ancestor):** which existing parent does the new fact attach to? (Worked example: `FloatPrimitive` — does `NonIntegerPrimitive` already host floats? Pilot's `NonIntegerAlgebra` was Bool/Unit-only, so NO; `FloatPrimitive` is a sibling new variant, not a refinement of an existing one.)
- **Step 2 (Coproduct-vs-coordinate):** for each new sum (e.g., `CompoundKind`, `FunctionKind`, `ContainerAlgebra`), do all variants ever co-inhabit (→ record) or alternate (→ sum)?
- **Step 3 (Primitive-vs-lens-extensible):** for new leaves (e.g., `Encoding`, `SizedKind`), are they substrate primitives or lens-extensible labels?

---

## Out of scope (do NOT do)

- **Coercion-Fold body.** This lane lands the substrate facts; the fold itself is T-Ground-Coercion-Fold (S; held per `design-emission-model.md` option (c) until LanguageSpec lands).
- **Lifetime / ownership derivation from program use.** T-Ground-Lifetime-Analyzer (LANDED). This lane consumes the lifetime axis as substrate; does NOT re-author derivation logic.
- **`EmissionDiagnostic` carrier authoring.** T-Ground-Diagnostic (S). This lane uses `StructureMismatch` per existing walker pattern; no new error variants.
- **`MethodTemplateContract` higher-order rows.** Phase 1.5; gated on Substrate shape decision (#1130).
- **Track-13 dissolution.** `TypeCheckpoint` / `InhabitantDecl` / `carrier: String` / `dsl/extdeps/languages/rust/types.dag` deletion stays in T-Ground-Dissolve.
- **Pilot-crate deletion.** T-Ground-Dissolve.
- **`RUST_PILOT_PRIMITIVES` mirror retirement.** T-Ground-LanguageSpec (Reflective Pattern E retirement, scope item D in `t-ground-languagespec.md`). This lane keeps the mirror consistent during variant expansion; does NOT retire it.
- **Touching `src/v3/compiler/`.** SG-0 ratchet.
- **Python / Go target work.** T-Ground-Python / T-Ground-Go (sibling lanes; gated on PR-G / PR-H respectively).
- **Re-litigating Q1 / Q2 / Q3 / Q4 / Q5 / Q6.5 locks.**
- **Re-litigating the variant-aware partition lock** per `grounding-pilot-receipt.md` (codex P2 adjudication 2026-04-25). Adding sibling variants is in scope; flattening to a record is out of scope.
- **Mixing authority (a) and authority (b) into one citation per row.** Faithfulness violation.

---

## Dependencies / gates

| Gate | Status (at brief authoring 2026-05-05) | Lane impact |
|---|---|---|
| **PR-PreF** (Substrate; `Interval<D>` consolidation) | LANDED — `src/v3/std/substrate.dag:123` | Q1 instance available |
| **PR-F** (Q1 `BoundDeclaration` consumer + Q2 Rust `ReferenceModel<T>` axes) | **PRIMARY GATE — not landed** (`grep ReferenceModel dsl/std/*.dag dsl/extdeps/languages/rust/*.dag` empty at audit 2026-05-05) | Required for D + E |
| **PR-I** (Q3 `RealizationCost` + Q4 universal four-property gate) | not landed | Required for F (may slice as Phase 2) |
| **Substrate `HigherOrderMethodSpec` shape decision** (cross-manager #1130 to jolly-ram-908) | in flight | Required only if a primitive declaration needs higher-order rows; otherwise out of scope (G) |
| **T-Ground-Lifetime-Analyzer R2 scope** | LANDED (#1206 / #1218 / #1220) | Lifetime axis available as substrate consumer |
| **#1129 / #1156 / #1162 (Tier 1 locks)** | LIVE on main | Consumed (Q1 / reflection-completeness / Q6.5) |
| **Host worktree git plumbing** | **BROKEN** at `/home/briansrls/.worktrees/proud-lark-674` (`fatal: not a git repository`); fix is host-side, outside this lane | Required before any code edits |

**Cross-program signals:**
- **Substrate Manager — ValueBody-list/sum + std.unicode bootstrap:** NOT a hard gate for this lane (Coercion-Fold consumes it). T-Ground-Rust can land structural rows without it.
- **R3 Grounding Manager (`#1745`):** lane closure signal; STOP-and-escalate target for any of the STOP conditions below.

---

## Sizing

**XL** per `r2-grounding-manager.md:63` and `ROADMAP.md:194`. Distribution (informal; bundle policy per `feedback_bundle_workstreams_per_pr.md`):

- A — Rust Reference primitive rows (full coverage minus pilot): L (~12 primitive families × authority + axis citations).
- B — std-library carriers: M (11 carriers; some collapse onto `ReferenceModel<T>` from A).
- C — `RustPrimitive` variant extension + walker arms + pilot-mirror lockstep: M.
- D — Q1 `BoundDeclaration` consumer wiring (integer family + `PlatformDependent`): S.
- E — Q2 `ReferenceModel<T>` Rust axis population: M.
- F — Q3 `RealizationCost` per-primitive population: M (likely separate phase).
- G — Higher-order rows: out of scope this lane (Phase 1.5).
- H — P1 receipts: included in each PR body.

**Recommended slicing** (manager confirms at dispatch):
- **Phase 1 (gated on PR-F):** smallest-meaningful-slice — `u128` + `isize` + `usize` + `f32` + `f64` + accompanying `RustPrimitive` shape extension (new `FloatPrimitive` variant) + walker arms + pilot-mirror update. Validates PR-F's Q1 + Q2 locks end-to-end on a non-pilot primitive set; first exercise of `PlatformDependent`. Per the manager's 2026-05-05 correction, `i128` is NOT in this slice (already landed).
- **Phase 2 (within Phase 1 PR or follow-up; manager call):** textual + never + tuple + array + slice.
- **Phase 3:** struct/enum/union/function/closure + trait object + `impl Trait`.
- **Phase 4 (gated on PR-I):** `RealizationCost` backfill on all rows landed in Phases 1-3 + std-library carriers from B that need cost coordinates.
- **Phase 5:** std-library carriers (B) — scheduled after Phase 1's shape is proven; some carriers (`Box` / `Rc` / `Arc`) may move earlier if they're trivially in `ReferenceModel<T>` axis space.

If Phase 1's `FloatPrimitive` variant or `PlatformDependent` consumer surfaces an unanticipated substrate gap, escalate to manager before splitting further.

---

## Test plan

Per `TESTING.md` — hermetic, behavior-driven, unit-first; sub-second per `feedback_test_timeout_2s.md`.

Acceptance lifted to a `.dag` `TestClaim` (gate: `rust_target_primitives_declared_structurally` per `r2-grounding-manager.md:124`):

1. **Per-primitive structural-load test** — every Rust primitive declared in A and B loads through reflection without per-consumer projection (per `design-reflection-completeness.md:103`); the `Dag::rust_pilot_primitives()` accessor (or its post-LanguageSpec successor) walks each row and reaches `validate_loaded_rust_primitive_type_structure` without `StructureMismatch`.
2. **Authority-citation completeness** — every row carries an authority URL comment; each URL resolves to either Rust Reference §Types or a std-doc page; no row mixes both.
3. **Q1 `PlatformDependent` exercise** — `isize` and `usize` rows match against an emit-target's `BoundDeclaration` per the asymmetric match rule (Q1 lock); test asserts `PlatformDependent` resolves to a concrete interval at target-platform time and never collapses into `StaticBound`.
4. **Q2 `ReferenceModel<T>` axis coverage** — every Rust pointer-family row (`&T` / `&mut T` / `*const T` / `*mut T` / `Box<T>` / `Rc<T>` / `Arc<T>`) carries a complete axis tuple per E above; missing axis = `StructureMismatch`.
5. **Variant-partition discipline** — `RustPrimitive`'s variants partition (every Rust value inhabits exactly one variant); the test enumerates a representative value per Rust primitive and asserts a unique walker arm matches.
6. **Float algebra discipline** — `f32`/`f64` rows do NOT claim `OrderedRing` / `Semiring` inhabitance (IEEE-754 fails ring axioms); the test asserts the row's algebra carrier is the float-specific algebra, not an integer algebra.
7. **Mirror-consistency probe (held)** — `validate_first_rust_pilot_row_matches_mirror` continues to fire on intentional drift between `Dag::rust_pilot_primitives()` and the Rust mirror until T-Ground-LanguageSpec retires the mirror (Reflective Pattern E retirement). This lane keeps the probe green during variant expansion.
8. **(Phase 4 only) `RealizationCost` sparseness fail-closed** — missing `AlgebraOp` in the `access` map for any Rust primitive produces `Witness.Violates`, not silent zero-cost.

---

## STOP conditions (escalate to manager `#1745`)

Worker stops + escalates if any of the following occur. Do NOT paper over.

1. **PR-F's landed Q2 axes can't express a Rust pointer-family row** as enumerated in E (e.g., `ReferenceModel<T>`'s axis set lacks a coordinate Rust requires). Signals PR-F under-specified.
2. **Q1's `Interval<Int>` carrier can't represent a Rust integer range literal** (e.g., `2^127 - 1` lowers to `ValueBody::Unparsed`). Signals a substrate-lowering gap escalation, sibling to the loader-close gap that produced #776.
3. **Two-authority split forces a substrate distinction the substrate doesn't carry** — e.g., a Rust Reference type and a std-library carrier collapse into the same structural shape under `RustPrimitive`'s partition, but their authorities require distinct citations. Signals a substrate-shape escalation.
4. **A primitive declaration requires a higher-order `MethodTemplateContract` row** (G — out of scope this lane). Signals Phase 1.5 dependency hit; verify `HigherOrderMethodSpec` Substrate decision (#1130) has landed before resuming.
5. **An emit-pipeline call site reading `dsl/extdeps/languages/rust/types.dag` cannot be left intact while structural rows land** — i.e., the structural row's introduction breaks an existing reader before T-Ground-Dissolve's planned cleanup. Signals T-Ground-Dissolve sequencing conflict; manager routes.
6. **`grounding-pilot-receipt.md`'s variant-aware partition lock breaks** under variant expansion (e.g., a Rust primitive doesn't fit any sibling variant cleanly). Re-opening that lock is a Director-routed scope change, not a worker call.
7. **Float algebra modeling discovers an algebra hierarchy gap** in `dsl/std/algebra.dag` (e.g., no existing carrier matches IEEE-754's structure). Signals a substrate-shape escalation; Q4 four-property gate may dictate the resolution.
8. **Apparent-multi-inhabitance** (e.g., `String` vs `Box<str>` vs `&str` vs `Cow<str>`) requires axis disposition that conflicts with T-Ground-LanguageSpec's apparent-multi-inhabitance audit (scope item F in `t-ground-languagespec.md`). Coordinate via manager; do NOT pre-empt that lane.
9. **Host worktree git plumbing remains broken** at dispatch time. Implementation cannot start.

---

## Cross-refs

- Parent: `r2-grounding-manager.md` (lane row line 63; pending list line 142).
- Engine-reframe spec: `docs/design-emission-model.md` (Q1-Q5 locks; lane row line 384 if present).
- Q6-Q8 (lens framework): `docs/design-lens-framework.md`.
- INVARIANTS substrate-fact-introduction procedure: `INVARIANTS.md` §P1.
- Pilot precedent: `docs/briefs/grounding-pilot-receipt.md`.
- Sibling brief shape: `docs/briefs/t-ground-languagespec.md`.
- Pre-cascade context (historical only): `docs/briefs/grounding-manager.md` (archives on R2 promotion); `docs/briefs/t-ground-engine-phase-1-typestructure.md` (Phase 2 framing held per `design-emission-model.md` option (c)).
- Audit receipts: #1745 comments 4377431312 (initial) + 4377437266 (delta with expanded-scope evidence); manager correction at #1773 comment 4377448850 (`i128` already landed).
