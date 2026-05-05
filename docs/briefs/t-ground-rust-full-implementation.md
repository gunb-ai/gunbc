# T-Ground-Rust — Full Rust target-primitive implementation

**Status:** PROPOSAL — dispatchable when **PR-F** (Q1 `BoundDeclaration` consumer + Q2 Rust structural axes via `ReferenceModel<T>`) merges. PR-F is the **sole hard primary gate** for §A-§E (the Rust primitive structural rows this lane authors). Conditional gates apply only if specific rows are reached: a Substrate parent decision for the §B `Option<T>` row (no top-level `Option` substrate parent at HEAD); the substrate `HigherOrderMethodSpec` shape decision (#1130) only if a primitive declaration requires higher-order method rows (§G, otherwise out of scope). PR-I (Q3 \`RealizationCost\`) is **NOT** a gate on this lane — \`RealizationCost\` population is owned by T-Ground-LanguageSpec per §F (out of scope here). Authored 2026-05-05 ahead of PR-F to keep the lane queue warm; consistent with `r2-grounding-manager.md:142` and the manager's directive that brief authoring is the only Day-1-ready Grounding item once host git is restored. No code lands until PR-F clears AND host git is restored AND the manager re-authorizes dispatch.

**Lane:** T-Ground-Rust (XL) — item 2 of 11 in [`r2-grounding-manager.md`](r2-grounding-manager.md) (line 28 + lane row line 63).

**Manager:** R2 Grounding Manager ([`r2-grounding-manager.md`](r2-grounding-manager.md)) — T-Ground sub-program lives under R2 Grounding per the live authority docs.

**Lineage / authorities consumed (no re-litigation):**
- Engine-reframe spec: [`docs/design-emission-model.md`](../design-emission-model.md) — Q1 `BoundDeclaration` (lines 994-1067), Q2 `ReferenceModel<T>` (lines 1068-1117), Q3 `RealizationCost` (lines 1143-1208), Q4 universal four-property gate (line 1234), cadence table (lines 1271-1282).
- Two-authority discipline: `grounding-manager.md:60-74` — **(a) Rust Reference §Types** authority (<https://doc.rust-lang.org/reference/types.html>) for language-level structural types; **(b) std-library carriers** authority (std documentation) for `String` / `Vec<T>` / `Box<T>` / `Rc<T>` / `Arc<T>` / `HashMap<K,V>` / `BTreeMap<K,V>` / `HashSet<T>` / `BTreeSet<T>` / `Option<T>` / `Result<T, E>`. Each per-primitive row cites its own authority — mixing is a faithfulness violation per `r2-grounding-manager.md:124` (Q4 four-property gate).
- THESIS: `THESIS.md:171` — "Coercion = emission. No separate coercion engine." This lane lands the substrate facts the fold reads; it does NOT introduce selection logic.
- Substrate-fact-introduction: [`INVARIANTS.md`](../../INVARIANTS.md) §P1 (3-step procedure for every new field on `RustPrimitive` or sibling per-primitive carriers).
- Sibling brief shape: [`t-ground-languagespec.md`](t-ground-languagespec.md).
- Pilot precedent: [`grounding-pilot-receipt.md`](grounding-pilot-receipt.md) (variant-aware partition `IntegerPrimitive | NonIntegerPrimitive` locked per codex P2 adjudication 2026-04-25; widening to flat record out of scope).

---

## Framing question this lane answers

Does Rust's full target-primitive surface (Rust Reference §Types + std-library carriers) declare structurally in `.dag` against `BoundDeclaration` + `ReferenceModel<T>` (per-primitive `RealizationCost` is **NOT** in this lane's scope — see §F; owned by T-Ground-LanguageSpec) with each primitive citing its own authority — so that the inhabitance walk `src/v3/grounding_engine/src/lib.rs:59` can validate every Rust primitive without falling back to `dsl/extdeps/languages/rust/types.dag` table lookup?

A "yes" populates the substrate that Coercion-Fold reads, retires the table-driven scaffolding under T-Ground-Dissolve, and produces the structural acceptance gate `rust_target_primitives_declared_structurally` (per `r2-grounding-manager.md:124`).

A "no" — or any discovery that scope is mis-modeled / authorities have drifted / the two-authority split forces a substrate distinction the substrate doesn't carry — escalates to manager per the discipline reminders below; do NOT paper over.

---

## Scope

### A. Rust Reference §Types primitives — structural declarations against authority (a)

Author per-primitive structural rows in `dsl/extdeps/languages/rust/primitives.dag` (extending the existing `RustPrimitive` type at line 180 + `rust_pilot_primitives` data at line 208). Authority: <https://doc.rust-lang.org/reference/types.html>. Coverage required:

- **Numeric — integer family.** Pilot already covers `i8`–`i64`, `u8`–`u64`, plus `i128` (LANDED — manager correction 2026-05-05; `i128` exists in `primitives.dag` and the pilot mirror at HEAD). Remaining: `u128`, `isize`, `usize`. The `isize`/`usize` rows consume Q1's `PlatformDependent` variant of `BoundDeclaration` (`src/v3/std/substrate.dag:136`); this is the first non-pilot exercise of `PlatformDependent` and validates PR-F's Q1 consumer end-to-end.
- **Numeric — floating-point family.** `f32`, `f64`. **NOT DISPATCHABLE AT HEAD — STOP-AND-ESCALATE row.** No live float inhabitance exists at HEAD that the Rust f32/f64 rows can honestly consume:
  - **Live float substrate declares an inadequate parent**: `dsl/std/float.dag:14-18` reads `type Float32 = Field<Word32>` / `type Float64 = Field<Word64>` / `type Float = Float64` — exact-`Field` over a fixed-width word, which elides IEEE-754's rounding/NaN/signed-zero/subnormal-policy facts entirely. Authoring Rust f32/f64 against this live parent would propagate the elision into the grounding substrate.
  - **`ApproximateField<F>` exists as the *intended future parent*** at `src/v3/std/approximate_field.dag:75` but no live `inhabits` declaration ties Float to it. The "Float inhabits ApproximateField" line at `dsl/std/algebra.dag:66` is **prose in a comment block**, NOT a declaration. The Float→ApproximateField migration is owned by T-NumericConstruction-ApproximateField and has not landed.
  - **Real / base-carrier STOP** is also active (`src/v3/std/approximate_field.dag:11-14`; audit doc: `docs/audit/t-numeric-construction-approximate-field-real-parameter-stop.md`): even after migration, `F` in `ApproximateField<F>` has no honest substrate target.
  - **Therefore the Rust f32/f64 row is held**: worker STOPs and escalates to manager. **Do NOT** phrase Rust f32/f64 as inhabiting or consuming `ApproximateField<F>` at HEAD; `ApproximateField<F>` is the *post-gate candidate parent*, not a dispatchable consumer. **Discipline preserved post-gate**: Rust f32/f64 must NOT claim `OrderedRing` / `Semiring` inhabitance (IEEE-754 fails ring axioms — NaN, signed zero, non-associative addition); the no-ring-axiom discipline applies regardless of which gated parent eventually resolves. Authority cited: Rust Reference §Floating-point types + IEEE-754 §3 + Real-parameter STOP audit + Float-migration ownership in T-NumericConstruction-ApproximateField.
- **Textual.** `char` (32-bit Unicode scalar value) and `str` (UTF-8 byte sequence, dynamically sized). Per `design-emission-model.md:89, :530-532` (locked) **encoding is carried by algebra choice, not a refinement axis** — `str` inhabits `FreeMonoid<Char>` (UTF-8 by Rust's definition of `str`); raw-byte sequences (`[u8]`) inhabit `FreeMonoid<Byte>`. Authoring an `encoding` axis on textual rows would be a P2 parallel-authority violation. Dynamically-sized status remains a structural axis.
- **Never.** `!` — uninhabited. Algebra inhabitance is restricted to algebras that require **no value witnesses** — i.e. operation-only algebras whose witnesses are functions out of the carrier (the elimination `absurd: fn(!) -> T` vacuously satisfies any `fn(!, ...) -> _` shape). `Magma<!>` is constructible because its only field is `op: fn(!, !) -> !`. Algebras with stored value witnesses — `Monoid<T>` requires `identity: T` (`dsl/std/algebra.dag:113`), `Group<T>` requires `identity: T` and `inverse: fn(T) -> T` (`:127`), etc. — are **NOT inhabited** by `!` because no value of type `!` can be supplied for `identity`. The brief's prior universal-bottom framing was incorrect; per Q4 four-property gate (Faithful), claiming inhabitance for algebras whose witnesses cannot be constructed is a faithfulness violation. Per `INVARIANTS.md` §P1, the worker MUST cite which algebras `!` inhabits and which it doesn't, with the witness-constructibility receipt per algebra. Authority: Rust Reference §Never type.
- **Tuple.** Variadic structural product. Each tuple arity is not a separate primitive; the row declares the tuple constructor with arity as a refinement axis. Unit `()` is the 0-arity tuple — already in pilot.
- **Array.** `[T; N]` — fixed-cardinality structural product. **Cardinality carrier — substrate gap noted:** `Interval<Cardinal>` is NOT a landed substrate type at HEAD (`grep '^type Cardinal' dsl/std/ src/v3/std/` empty); only `Interval<Int>` is instantiated (`src/v3/std/substrate.dag:129`), and the live cardinality carrier is `CardinalityBound = Exact(Int) | AtMostOne | Unbounded` (`src/v3/std/substrate.dag:143`). Until `Interval<Cardinal>` lands (PR-PreF cascade per `design-emission-model.md:1023, :1038`), array rows consume `CardinalityBound::Exact(N)`. **Bridge + dissolution trigger:** when the `Cardinal` ordered-domain instance lands, retrofit additively to `Interval<Cardinal>::BoundedInterval { lower: N, width: ZeroWidth }` per the Q5 collapse note (`design-emission-model.md:1267`); dissolution trigger is the `Cardinal` substrate landing PR.
- **Slice.** `[T]` — dynamically-sized structural product. Cardinality consumes `CardinalityBound::Unbounded` at HEAD (same substrate gap as Array); retrofits to `Interval<Cardinal>::Unbounded` post-`Cardinal`. Authority axis: dynamically-sized.
- **Struct / enum / union — base shapes.** Per `r2-grounding-manager.md:265` ("Struct / enum / union primitives"). These are *constructor schemas*, not concrete primitives; the row declares the structural shape (named-field record / tagged sum / untagged union with safety-required-by-construction).
- **Function item.** Each named `fn` (or `impl` method, or tuple-struct/enum-variant constructor) has a unique anonymous **function-item type** carrying *identity* + signature. Identity is per-instantiation, NOT per-decl: `foo::<i32>` and `foo::<u64>` have *distinct* function-item types. The identity record carries `{ item_decl, type_args, early_bound_lifetimes, constructor_case }` (see §C); `constructor_case` distinguishes ordinary fns from tuple-struct / enum-variant constructors (also function items per Rust Reference). Zero-sized. Coerces to a function pointer with the same signature; identity is lost on coercion. Authority: Rust Reference §Function item types.
- **Function pointer.** `fn(...) -> ...` — signature-only carrier; pointer-sized; multiple distinct function items with the same signature can be type-erased to the same `fn` pointer. Carries qualifiers as **coordinates on a record** (NOT a sum): `unsafe: Bool` and `abi: AbiTag` are independent — `unsafe extern "C" fn(...)` co-inhabits both, so they cannot collapse into a coproduct. Authority: Rust Reference §Function pointer types.
- **Closure.** Anonymous compiler-generated type carrying captures. Trait inhabitance is a *cumulative tower* over `FnOnce ⊇ FnMut ⊇ Fn`, derived from how the closure body uses each capture (read / mutate / move-out), not from the `move` keyword (which only controls capture mode by-ref vs by-value). Authority: Rust Reference §Closure types. See §C below for the full derivation rule.

These three are **structurally distinct**, NOT three rows under one `FunctionKind` enum: function-item identity (one row per fn item) is incompatible with function-pointer signature-only shape, and closure captures can't fit either. Worker MUST keep them separate at the `RustPrimitive` variant level (see §C below); collapsing item-identity into pointer-signature loses faithfulness (Q4) at the inhabitance step.
- **Reference.** `&T` (shared, immutable, lifetime-bounded), `&mut T` (exclusive, mutable, lifetime-bounded). Both inhabit `ReferenceModel<T>` with axes (`mutability`, `lifetime`) populated; ownership axis is `Borrowed`. Lifetime is **structural, not annotation-driven** per T-Ground-Lifetime-Analyzer authority (LANDED #1206 / #1218 / #1220) — this lane consumes the lifetime axis as substrate, does NOT re-author it.
- **Raw pointer.** `*const T`, `*mut T`. `ReferenceModel<T>` axes (`mutability`, `representation`); ownership axis is `Raw`; no lifetime. The unsafe/safe distinction is carried by `representation`, not a separate `safety` axis (Q2 lock declares the four-axis set `{lifetime, mutability, ownership, representation}` — workers MUST NOT introduce a parallel `safety` coordinate).
- **Trait object.** `dyn Trait` — dynamically-sized, vtable-bearing. Authority: Rust Reference §Trait object types.
- **`impl Trait` — split into two rows per position** (Rust Reference §Impl Trait distinguishes them as separate spec facts):
  - **Argument-position `impl Trait`** — anonymous type parameter; caller chooses the concrete type; structurally equivalent to introducing a fresh universal type parameter `<T: Trait>` at the call site. Carries the trait-bound set as a structural fact; no opacity (the caller knows the concrete type). Authority: Rust Reference §Impl Trait → Anonymous type parameters.
  - **Return-position `impl Trait`** — abstract/opaque return type; *callee* chooses the concrete type and the caller sees only the trait-bound interface. Carries trait-bound set + `opaque_captures` (captured generic params + optional `use<>` precise-capture restriction) + `edition_capture_policy` (the **edition-sensitive default-capture rule** per RFC 3498 — Rust 2015/2018/2021 do NOT capture lifetimes by default; Rust 2024+ captures all generics in scope). Same source text expands to different captures across editions, so the substrate row MUST carry edition. `use<>` legality: legal only on `fn` items and inherent methods, NOT on trait method definitions (RFC 3617). Existential, not universal. Authority: Rust Reference §Impl Trait → Abstract return types + RFC 3498 (default-capture rules) + RFC 3617 (precise capturing).

Collapsing these into one row drops the universal-vs-existential distinction (caller-chooses vs callee-chooses) — a P1/M3 spec-fidelity violation per the reviewer pointer. §C below carries two sibling `RustPrimitive` variants.

Each primitive row cites its **authority URL** (Rust Reference section or std-doc URL) in a comment adjacent to the row per the brief-authoring-checklist convention.

### B. std-library carriers — structural declarations against authority (b)

Authority: <https://doc.rust-lang.org/std/> per type. Coverage required:

- **`String`** — owned, growable, UTF-8 char sequence. Inhabits `FreeMonoid<Char>` (per `design-emission-model.md:534`); the algebra choice IS the encoding distinction (`FreeMonoid<Byte>` would be raw bytes — `Vec<u8>` territory, not a `String` candidate). Refinement axes: ownership = `Owned`; growability = `Growable`; lifetime = `self`. **No `encoding` axis** — that would duplicate the algebra-carried fact.
**std-carrier full-signature discipline.** Per Rust std documentation, every collection carrier carries hidden generic parameters (allocator `A`, hasher `S`) that the brief MUST surface as structural axes. Dropping them loses extdeps-fidelity facts before grounding (P1 violation). Each row enumerates the full Rust signature, with hasher/allocator axes carried alongside K/V/T:

- **`Vec<T, A = Global>`** — owned, growable, contiguous heap buffer. Cardinality: `CardinalityBound::Unbounded` at HEAD (same `Cardinal`-substrate gap as Array/Slice in §A; retrofits to `Interval<Cardinal>::Unbounded` when `Cardinal` lands). Ownership: `Owned`; growability: `Growable`. **Allocator axis** `A` (default `Global`) is a structural fact of the carrier signature, not a defaulted shorthand — distinct allocators produce distinct realization rows.
- **`Box<T, A = Global>`** — single-owner heap pointer. `ReferenceModel<T>` ownership axis: `Owned`; no lifetime; representation: `Safe` (safe/unsafe distinction on the `representation` axis per Q2 four-axis lock). Allocator axis `A` (default `Global`).
- **`Rc<T, A = Global>`** — shared-ownership reference-counted pointer (single-threaded). `ReferenceModel<T>` ownership: `SharedRefCounted { thread_safe: false }`. Allocator axis `A` (default `Global`; nightly-stable on standard Rust toolchain at the time of authoring — flag if the targeted edition predates allocator stability).
- **`Arc<T, A = Global>`** — shared-ownership atomic-reference-counted pointer (thread-safe). `ReferenceModel<T>` ownership: `SharedRefCounted { thread_safe: true }`. Allocator axis `A` (same nightly-stable note as `Rc<T, A>`).
- **`HashMap<K, V, S = RandomState, A = Global>`** — hash-table-backed associative array. Inhabits `PartialFunction<K, V>` (`dsl/std/algebra.dag:428`). Refinement axes: `ordering: None`; **key-admissibility** `K: Hash + Eq`; **hasher axis** `S: BuildHasher` (default `RandomState`); **allocator axis** `A` (default `Global`). Hasher choice is a structural fact (e.g., `FxHashMap` vs default `RandomState` differ on collision-resistance vs throughput tradeoffs) — distinct hashers produce distinct realization rows. Trait-bound axes are NOT optional: "hash-backed admissibility" is the structural fact that distinguishes `HashMap` from `BTreeMap` at the realization step, not just ordering.
- **`BTreeMap<K, V, A = Global>`** — B-tree-backed ordered associative array. Inhabits `PartialFunction<K, V>`; refinement axes: `ordering: Sorted`; **key-admissibility** `K: Ord`; **allocator axis** `A` (default `Global`). No hasher axis — B-tree ordering doesn't require one. `Hash + Eq` and `Ord` are *distinct* admissibility shapes (a key can be `Ord` without `Hash` — `f64` is `PartialOrd`-only and inhabits neither cleanly, which is itself a structural fact).
- **`HashSet<T, S = RandomState, A = Global>`** — inhabits `Set<T> = BooleanAlgebra<T>` (`dsl/std/types.dag:212`, per M9 DFS). Refinement axes: `ordering: None`; element-admissibility `T: Hash + Eq`; **hasher axis** `S: BuildHasher` (default `RandomState`); **allocator axis** `A` (default `Global`).
- **`BTreeSet<T, A = Global>`** — inhabits `Set<T> = BooleanAlgebra<T>`. Refinement axes: `ordering: Sorted`; element-admissibility `T: Ord`; **allocator axis** `A` (default `Global`). No hasher axis. **Do NOT route either Set carrier through `PartialFunction<T, Unit>`** — P2 violation (parallel authority for `Set<T> = BooleanAlgebra<T>`).
- **`Option<T>`** — sum carrier `Some(T) | None`. **M9 DFS receipt + substrate-gap gate:** searched `dsl/std/` for an existing optional/cardinality parent — `dsl/std/algebra.dag:517` carries `OptionalOf { inner: AlgebraTypeTemplate }` as an `AlgebraTypeTemplate` *return-type* variant (used for `first` / `last` / `get` etc.), but **no top-level substrate `type Option<T>` / `Optional<T>` / `Maybe<T>` declaration exists at HEAD** (`grep '^type (Option|Optional|Maybe)' dsl/std/` empty). The natural parent is "cardinality `AtMostOne` over a coproduct of (some-inhabitant, none)" — `CardinalityBound::AtMostOne` (`src/v3/std/substrate.dag:144`) carries the cardinality half. **This lane does NOT introduce a new top-level `Option` substrate type** (would be a P1-Step-1 violation given the existing partial machinery). Instead, the `Option<T>` row is **GATED** on a substrate decision Substrate Manager owns: either (a) declare a top-level `type Option<T>` substrate parent (cardinality-`AtMostOne` + coproduct) under R2 Substrate, or (b) extend `OptionalOf` from algebra-template-only to a full substrate carrier. Worker STOP-and-escalate at this row; do not author the row before Substrate adjudication. (`Result<T, E>` does NOT carry this gate — its parent is live at `dsl/std/error_primitives.dag:12`; see next bullet.)
- **`Result<T, E>`** — sum carrier `Ok(T) | Err(E)`. **M9 DFS receipt — substrate parent FOUND:** `dsl/std/error_primitives.dag:12` declares `type Result<ok, err> = Ok { value: ok } | Err { value: err }` — this IS the existing top-level `Result` substrate carrier (my earlier DFS missed it; corrected via reviewer pointer). Rust `Result<T, E>` therefore inhabits the existing `dsl/std/error_primitives.dag` `Result<ok, err>` parent directly — no Substrate-gap gate, no parallel parent. The two-authority discipline still applies (Rust Reference + std citations stay separate). **`Option<T>` does NOT have an analogous existing parent** at HEAD (verified: `grep '^type (Option|Optional|Maybe)' dsl/std/` empty); only `Option<T>` retains the substrate-gap gate above.

**Two-authority discipline:** A and B rows cite distinct authority URLs. If row layout in a single `primitives.dag` file would mix authorities ambiguously (i.e., a reader can't tell which row claims which authority), split into `dsl/extdeps/languages/rust/primitives.dag` (authority a) + `dsl/extdeps/languages/rust/std_carriers.dag` (authority b). This split is **a discovered necessity, not a default** — author in one file first; split only if the faithfulness review surfaces the ambiguity. Escalate the split decision to manager before landing.

### C. `RustPrimitive` type extension + grounding-engine walker arms

The current `RustPrimitive` partition (`primitives.dag:180`) is `IntegerPrimitive { algebra: IntegerAlgebra, .. } | NonIntegerPrimitive { algebra: NonIntegerAlgebra, .. }`. Per `grounding-pilot-receipt.md` lock, the partition is fixed-shape; widening to flat record is out of scope. New variants required for full coverage:

- `FloatPrimitive { .. }` — **NOT DISPATCHABLE AT HEAD; row shape is post-gate candidate only.** The variant slot is reserved (so the partition can include it once gates clear), but the field set is held until live Float inhabitance exists. Post-gate candidate shape (informational, not dispatchable): a single `approximate_field: ApproximateField<F>` field once the Float→ApproximateField migration AND the Real/base-carrier decision both land; binary32 (`f32`) vs binary64 (`f64`) would then differ on `(precision, special_values, subnormal_policy)` coordinate values, not parent identity. Until both gates clear (see §A), worker STOPs at this variant. Distinct from `NonIntegerPrimitive` because pilot's `NonIntegerAlgebra` was Bool/Unit-only.
- `TextualPrimitive { algebra: TextualAlgebra, sized: SizedKind, .. }` — for `char` / `str`. `TextualAlgebra` selects between `FreeMonoid<Char>` (UTF-8 char sequence — `str`) and the per-codepoint shape for `char`; encoding is **not** a separate field per the algebra-carries-encoding lock (`design-emission-model.md:89, :530-532`).
- `NeverPrimitive { .. }` — empty-type marker.
- `CompoundPrimitive { kind: CompoundKind, .. }` — tuple / array / slice / struct / enum / union shapes. CompoundKind is a sub-sum; per Step 2 receipt, these alternate (a value is a tuple OR an array, never both), so sum-shape is correct.
- `FunctionItemPrimitive { identity: FunctionItemIdentity, signature: FnSignature, .. }` — one row per named function (carries identity); zero-sized. The `identity` record carries the full Rust Reference §Function-item-types spec facts: `{ item_decl: DeclarationId, type_args: List<Type>, early_bound_lifetimes: List<Lifetime>, constructor_case: ConstructorCase }`. `ConstructorCase` is a sum because tuple-struct constructors and enum-variant constructors are also function items (per Rust Reference) but with structurally distinct origin shapes — collapsing them under `item_decl` alone loses the constructor distinction. `type_args` is required because a generic function `fn foo<T>` has a *distinct* function-item type per instantiation `foo::<i32>` vs `foo::<u64>`; identity is per-instantiation, not per-decl. Early-bound lifetimes likewise distinguish item types.
- `FunctionPointerPrimitive { signature: FnSignature, unsafe: Bool, abi: AbiTag, .. }` — signature-only carrier; pointer-sized; multiple function items collapse to the same row when signatures match. Qualifiers are **independent record coordinates** (not a sum) — `unsafe extern "C" fn(...)` co-inhabits both, so `FnQualifiers` is NOT a coproduct (corrects the §H Step-2 example below).
- `ClosurePrimitive { signature: FnSignature, captures: CaptureSet, .. }` — anonymous compiler-generated. **`CaptureSet` shape per Rust Reference §closure-capture:** `List<Capture>` where each `Capture { path: PlaceExpr, mode: CaptureMode, body_use: CaptureBodyUse }`. The `path` is the **place expression** the capture refers to (`x` whole binding, `x.field`, `x.field.subfield` — Rust 2021+ disjoint captures resolve at field-precision, not whole-binding); the `mode` is one of the **four Rust closure capture modes** per Rust Reference §Closure types: `SharedBorrow` (`&T`) / `UniqueImmutableBorrow` (`&uniq T`, used internally for cases like `&mut x[..]` slice access where the closure needs unique-but-immutable access — distinct from both shared and mutable borrow) / `MutableBorrow` (`&mut T`) / `ByValue` (move). Mode is controlled by `move` and by use-site; the unique-immutable-borrow mode cannot be expressed at user syntax but is observable in borrow-checker behavior, so the substrate row MUST carry it as a distinct variant or the closure model fails P1 spec fidelity; the `body_use` is read / mutate / consume. **Capture-path precision is required** — collapsing `x.field` and `x.other_field` into a single `x` capture loses the disjoint-borrow facts the borrow checker depends on. The cumulative tower per Rust Reference §Closure types §call-traits-and-coercions: every closure inhabits `FnOnce`; it additionally inhabits `FnMut` if the body does not move out of any capture's place; it additionally inhabits `Fn` if the body also does not mutate any capture's place. The `move` keyword controls capture *mode* (by-value vs by-reference) and on its own neither precludes `Fn` nor `FnMut` — a `move` closure that only reads its captures still inhabits `Fn`. The capability set is the *cumulative* derivation over `body_use` and is **NOT a stored field** on this primitive — a stored `fn_trait` field would author a parallel representation of a fact already determined by `captures` (P2 violation).

These are **three sibling variants**, not one variant with a `kind: FunctionKind` enum — see §A item/pointer/closure justification.
- `ReferencePrimitive { model: ReferenceModel<T>, .. }` — references / raw pointers / `Box` / `Rc` / `Arc` (B's pointer-family carriers re-home here per A's `ReferenceModel<T>` axis).
- `TraitObjectPrimitive { trait_bounds: TraitBoundSet, .. }`.
- `ImplTraitArgPrimitive { trait_bounds: TraitBoundSet, .. }` — argument-position `impl Trait` (anonymous type parameter, caller-chooses; universal).
- `ImplTraitReturnPrimitive { trait_bounds: TraitBoundSet, opaque_captures: OpaqueCaptureSet, edition: RustEdition, .. }` — return-position `impl Trait` (abstract return type, callee-chooses; existential/opaque). The `opaque_captures` record carries the captured generic parameters that flow into the opaque type: `{ type_params: List<TypeParamRef>, const_params: List<ConstParamRef>, lifetime_params: List<LifetimeParamRef>, precise_capture_restriction: Option<UseList> }`. **`edition: RustEdition` is stored; `default_capture` is DERIVED from `edition` by structural rule** — Rust 2015/2018/2021 → \"lifetimes NOT captured by default; type/const params ARE captured\"; Rust 2024+ (RFC 3498) → \"ALL generic params in scope captured by default.\" The default-capture rule is determined by the edition, so storing both fields would admit illegal states (e.g., edition=2021 paired with all-generics-default, which is the 2024 rule) — P2 illegal-states violation. The substrate carries the *primary* fact (edition) and the resolver derives the rule. The substrate row MUST carry the edition because the *same source text* expands to different `opaque_captures` sets across editions. **`use<>` legality constraints**: `precise_capture_restriction` is legal only on `fn` items and inherent methods, not on trait method definitions (per RFC 3617); the row MUST validate that the precise-capture restriction site supports it before declaring `Some(_)`. Authority: Rust Reference §Impl Trait → Abstract return types + RFC 3498 + RFC 3617.

These are sibling variants per the §A position-split; do NOT collapse into one `ImplTraitPrimitive { kind: Position }` (that would re-introduce the parallel-authority shape the position-split specifically dissolves).
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

Per Q2 lock (`design-emission-model.md:1100`) and `r2-grounding-manager.md:90`: pointer/reference family shares the `ReferenceModel<T>` parametric parent with the **complete four-axis set** (`lifetime`, `mutability`, `ownership`, `representation`). **Every row populates ALL four axes — no row is allowed to omit an axis.** Per the illegal-states discipline (state-space-vs-behavioral-invariants): a partial record admits combinations that should be structurally impossible. Rust's pointer-family rows:

- `&T`: `{ ownership: Borrowed, mutability: Immutable, lifetime: Bound, representation: Safe }`.
- `&mut T`: `{ ownership: Borrowed, mutability: Mutable, lifetime: Bound, representation: Safe }`.
- `*const T`: `{ ownership: Raw, mutability: Immutable, lifetime: None, representation: Unsafe }`.
- `*mut T`: `{ ownership: Raw, mutability: Mutable, lifetime: None, representation: Unsafe }`.
- `Box<T>`: `{ ownership: Owned, mutability: ContainerControlled, lifetime: None, representation: Safe }` — `ContainerControlled` means mutability is determined by the binding/container (`Box<T>` itself doesn't constrain), not by the pointer; this is a real fourth value of the `mutability` axis, not a defaulted shorthand.
- `Rc<T>`: `{ ownership: SharedRefCounted { thread_safe: false }, mutability: Immutable, lifetime: None, representation: Safe }` — `Rc<T>` is shared-immutable by structural construction; interior mutability requires `RefCell<T>` wrapping (a separate primitive, out of scope for this row).
- `Arc<T>`: `{ ownership: SharedRefCounted { thread_safe: true }, mutability: Immutable, lifetime: None, representation: Safe }` — same structural shape as `Rc<T>` modulo the thread-safe flag.

The `ReferenceModel<T>` shared-parent declaration itself lives in `dsl/std/` (substrate-owned per Q2 lock; PR-F lands it). This lane consumes; if PR-F's landed shape doesn't accommodate Rust's coverage as enumerated, STOP and escalate.

### F. Q3 `RealizationCost` per-primitive population — OUT OF SCOPE (owned by T-Ground-LanguageSpec)

Per `t-ground-languagespec.md:73` (sibling brief, scope item B): T-Ground-LanguageSpec lane owns per-primitive `RealizationCost` population for **all three targets** (Rust + Python + Go) — that lane consumes the per-target primitive sets the present lane (T-Ground-Rust) lands and attaches the cost coordinates. Authoring `RealizationCost` rows from this T-Ground-Rust brief would create a second lane authority for the same substrate fact (P2 violation: facts flow forward, single authority per substrate fact).

**This lane's responsibility is limited to:**
- Authoring the structural Rust primitive rows (§A-§E above) **without** `RealizationCost` fields. T-Ground-LanguageSpec adds those fields downstream when PR-I lands.
- If a Rust primitive row's shape forces a coordinate that `RealizationCost` cannot represent, escalate to manager `#1745` (cross-lane signal — T-Ground-LanguageSpec owns `RealizationCost` shape per `t-ground-languagespec.md:73`; this lane only flags). No T-Ground-Rust-internal STOP condition for this case (deferred to T-Ground-LanguageSpec scope per the §F authority split).
- No Phase B / Phase 4 backfill on this lane. The earlier brief revision had a misallocated "Phase 4: RealizationCost backfill" entry — removed in this commit.

### G. Higher-order MethodTemplateContract rows (Phase 1.5 — separately gated)

Per `r2-grounding-manager.md:67` Phase 1.5: Rust higher-order rows are gated on Substrate's shape decision for the existing dual-template `HigherOrderMethodSpec` carrier (`dsl/extdeps/languages/rust/emit.dag:265`); cross-manager request live to jolly-ram-908 (#1130). **OUT OF SCOPE for this lane's primary slice**; surfaced here so the worker doesn't accidentally absorb it. If higher-order Rust rows are needed by a primitive declaration this lane authors (e.g., `Vec<T>::map` requires a higher-order spec), STOP and escalate — likely the substrate-shape decision hasn't landed and this lane is blocked on it.

### H. P1 procedure receipts

Per `INVARIANTS.md` §P1 (lines 94-129), worker MUST cite receipts in the PR body for every new variant of `RustPrimitive` and every new field on existing variants:

- **Step 1 (DAG-ancestor):** which existing parent does the new fact attach to? (Worked example: `FloatPrimitive` — does `NonIntegerPrimitive` already host floats? Pilot's `NonIntegerAlgebra` was Bool/Unit-only, so NO; `FloatPrimitive` is a sibling new variant, not a refinement of an existing one.)
- **Step 2 (Coproduct-vs-coordinate):** for each new sum (e.g., `CompoundKind`, `ContainerAlgebra`), do all variants ever co-inhabit (→ record) or alternate (→ sum)? **`FnQualifiers` is NOT a coproduct** — `unsafe` and `extern "ABI"` co-inhabit (per §A function-pointer row), so they are independent record coordinates (`unsafe: Bool`, `abi: AbiTag`); do NOT introduce them as sum variants. **`FnTrait` (Fn / FnMut / FnOnce) is NOT a stored field anywhere** — derived from closure captures per §C `ClosurePrimitive`, so it is not subject to Step-2 sum-vs-record analysis (it has no representation to choose).
- **Step 3 (Primitive-vs-lens-extensible):** for new leaves (e.g., `SizedKind`, `CompoundKind`, `AbiTag`), are they substrate primitives or lens-extensible labels? **`Encoding` is NOT a permissible leaf** under this lane — encoding is carried by algebra choice (`FreeMonoid<Char>` vs `FreeMonoid<Byte>`) per the locked decision at §A textual / §B `String` rows above; introducing an `Encoding` leaf would re-open the parallel-authority shape that lock closes.

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
| **PR-I** (Q3 `RealizationCost` + Q4 universal four-property gate) | not landed | NOT a hard gate for this lane — T-Ground-LanguageSpec consumes PR-I and attaches `RealizationCost` to Rust primitive rows downstream (§F). Q4 four-property gate is consumed by §A-§E inhabitance receipts. |
| **Substrate `HigherOrderMethodSpec` shape decision** (cross-manager #1130 to jolly-ram-908) | in flight | Required only if a primitive declaration needs higher-order rows; otherwise out of scope (G) |
| **T-Ground-Lifetime-Analyzer R2 scope** | LANDED (#1206 / #1218 / #1220) | Lifetime axis available as substrate consumer |
| **#1129 / #1156 / #1162 (Tier 1 locks)** | LIVE on main | Consumed (Q1 / reflection-completeness / Q6.5) |
| **`Cardinal` ordered-domain substrate** (PR-PreF cascade per `design-emission-model.md:1023, :1038`) | NOT landed at HEAD (`grep '^type Cardinal' dsl/std/ src/v3/std/` empty); only `IntInterval = Interval<Int>` instantiated | Array/Slice/Vec rows consume `CardinalityBound` at HEAD as a forward-bridge; retrofit to `Interval<Cardinal>` on landing (dissolution trigger = the `Cardinal` substrate landing PR) |
| **Option top-level substrate parent** (cardinality-`AtMostOne` + coproduct, or `OptionalOf` carrier promotion) | NOT landed at HEAD (`grep '^type (Option\|Optional\|Maybe)' dsl/std/` empty); only `OptionalOf` algebra-template variant exists | §B `Option<T>` row STOPs-and-escalates; Substrate Manager owns the parent decision. **`Result<T, E>` parent IS landed** at `dsl/std/error_primitives.dag:12`; that row dispatches normally. |
| **Float migration from `Field<Word*>` to `ApproximateField<F>`** (live float substrate at `dsl/std/float.dag:14-18` declares the wrong parent) | NOT migrated at HEAD — current declarations are `Float32 = Field<Word32>` / `Float64 = Field<Word64>` (exact-Field over fixed-width word; structurally inadequate for IEEE-754) | §A `f32`/`f64` rows STOP-and-escalate; T-NumericConstruction-ApproximateField slice owns the migration |
| **`ApproximateField<F>` Real / base-carrier decision** (`docs/audit/t-numeric-construction-approximate-field-real-parameter-stop.md`) | active STOP — no `Real` alias at HEAD; `F` parameter has no honest substrate target | §A `f32`/`f64` rows STOP-and-escalate (in addition to the Float migration gate above); T-NumericConstruction-ApproximateField slice owns the carrier convention decision |
| **Repository / git prerequisites** | implementation starts when repo/git prerequisites are satisfied (a clean working tree on the dispatching worker's environment) | Required before any code edits |

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
- F — Out of scope (owned by T-Ground-LanguageSpec). No sizing contribution.
- G — Higher-order rows: out of scope this lane (Phase 1.5).
- H — P1 receipts: included in each PR body.

**Recommended slicing** (manager confirms at dispatch):
- **Phase 1 (gated on PR-F):** smallest-meaningful-slice — `u128` + `isize` + `usize` + walker arms + pilot-mirror update. Validates PR-F's Q1 + Q2 locks end-to-end on non-pilot integer primitives; first exercise of `PlatformDependent`. Per the manager's 2026-05-05 correction, `i128` is NOT in this slice (already landed). **Floats are NOT in Phase 1**: `f32`/`f64` row authoring + `FloatPrimitive` variant population remain held until both Float-substrate gates clear (Float migration from `Field<Word*>` to `ApproximateField<F>` + Real / base-carrier decision; see §A and STOP condition #7). Phase 1 does not depend on either gate.
- **Phase 2 (within Phase 1 PR or follow-up; manager call):** textual + never + tuple + array + slice.
- **Phase 3:** struct/enum/union/function/closure + trait object + `impl Trait`.
- ~~Phase 4 (RealizationCost backfill)~~ — REMOVED. T-Ground-LanguageSpec owns per-primitive `RealizationCost` population for all targets (`t-ground-languagespec.md:73`); this lane's primitive rows ship without `RealizationCost` fields.
- **Phase 5:** std-library carriers (B) — scheduled after Phase 1's shape is proven; some carriers (`Box` / `Rc` / `Arc`) may move earlier if they're trivially in `ReferenceModel<T>` axis space.

If Phase 1's `PlatformDependent` consumer surfaces an unanticipated substrate gap, escalate to manager before splitting further.

---

## Test plan

Per `TESTING.md` — hermetic, behavior-driven, unit-first; sub-second per `feedback_test_timeout_2s.md`.

Acceptance lifted to a `.dag` `TestClaim` (gate: `rust_target_primitives_declared_structurally` per `r2-grounding-manager.md:124`):

1. **Per-primitive structural-load test** — every Rust primitive declared in A and B loads through reflection without per-consumer projection (per `design-reflection-completeness.md:103`); the `Dag::rust_pilot_primitives()` accessor (or its post-LanguageSpec successor) walks each row and reaches `validate_loaded_rust_primitive_type_structure` without `StructureMismatch`.
2. **Authority-citation completeness** — every row carries an authority URL comment; each URL resolves to either Rust Reference §Types or a std-doc page; no row mixes both.
3. **Q1 `PlatformDependent` exercise** — `isize` and `usize` rows match against an emit-target's `BoundDeclaration` per the asymmetric match rule (Q1 lock); test asserts `PlatformDependent` resolves to a concrete interval at target-platform time and never collapses into `StaticBound`.
4. **Q2 `ReferenceModel<T>` axis coverage** — every Rust pointer-family row (`&T` / `&mut T` / `*const T` / `*mut T` / `Box<T>` / `Rc<T>` / `Arc<T>`) carries a complete axis tuple per E above; missing axis = `StructureMismatch`.
5. **Variant-partition discipline** — `RustPrimitive`'s variants partition (every Rust value inhabits exactly one variant); the test enumerates a representative value per Rust primitive and asserts a unique walker arm matches.
6. **Float-row STOP assertion (held)** — at HEAD, the test asserts that NO `f32`/`f64` row is authored (i.e., the `FloatPrimitive` variant carries no populated rows) until both Float-substrate gates clear (Float migration from `Field<Word*>` to `ApproximateField<F>` + Real / base-carrier decision). Post-gate, this test converts to a coordinate-distinguishability check: `f32`/`f64` rows must NOT claim `OrderedRing` / `Semiring` inhabitance (no-ring-axiom discipline preserved per §A); once a parent is honest, `f32` and `f64` must differ on `(precision, special_values, subnormal_policy)` coordinate values per IEEE-754 §3, not on parent identity.
7. **Mirror-consistency probe (held)** — `validate_first_rust_pilot_row_matches_mirror` continues to fire on intentional drift between `Dag::rust_pilot_primitives()` and the Rust mirror until T-Ground-LanguageSpec retires the mirror (Reflective Pattern E retirement). This lane keeps the probe green during variant expansion.
8. ~~`RealizationCost` sparseness~~ — moved to T-Ground-LanguageSpec test plan; not a T-Ground-Rust acceptance check.

---

## STOP conditions (escalate to manager `#1745`)

Worker stops + escalates if any of the following occur. Do NOT paper over.

1. **PR-F's landed Q2 axes can't express a Rust pointer-family row** as enumerated in E (e.g., `ReferenceModel<T>`'s axis set lacks a coordinate Rust requires). Signals PR-F under-specified.
2. **Q1's `Interval<Int>` carrier can't represent a Rust integer range literal** (e.g., `2^127 - 1` lowers to `ValueBody::Unparsed`). Signals a substrate-lowering gap escalation, sibling to the loader-close gap that produced #776.
3. **Two-authority split forces a substrate distinction the substrate doesn't carry** — e.g., a Rust Reference type and a std-library carrier collapse into the same structural shape under `RustPrimitive`'s partition, but their authorities require distinct citations. Signals a substrate-shape escalation.
4. **A primitive declaration requires a higher-order `MethodTemplateContract` row** (G — out of scope this lane). Signals Phase 1.5 dependency hit; verify `HigherOrderMethodSpec` Substrate decision (#1130) has landed before resuming.
5. **An emit-pipeline call site reading `dsl/extdeps/languages/rust/types.dag` cannot be left intact while structural rows land** — i.e., the structural row's introduction breaks an existing reader before T-Ground-Dissolve's planned cleanup. Signals T-Ground-Dissolve sequencing conflict; manager routes.
6. **`grounding-pilot-receipt.md`'s variant-aware partition lock breaks** under variant expansion (e.g., a Rust primitive doesn't fit any sibling variant cleanly). Re-opening that lock is a Director-routed scope change, not a worker call.
7. **Float row reached without BOTH Float-substrate gates cleared** — two distinct gates per §A: (a) **Float migration** from `Field<Word*>` to `ApproximateField<F>` (live `dsl/std/float.dag:14-18` declares the inadequate `Field<Word*>` parent; migration to `ApproximateField` hasn't landed); (b) **Real / base-carrier decision** (`docs/audit/t-numeric-construction-approximate-field-real-parameter-stop.md`) — `ApproximateField<F>`'s `F` parameter has no honest substrate target at HEAD. Worker STOP-and-escalates at the §A `f32`/`f64` row until BOTH clear; do NOT pick a placeholder for `F`, do NOT consume the live `Field<Word*>` parent. Separate sub-condition: even after both gates clear, if a Rust-required precision/rounding/special-values/subnormal-policy combination doesn't fit the existing `(precision, rounding, special_values, subnormal_policy)` coordinate space, escalate again — that's a substrate-shape change on `ApproximateField<F>` itself.
8. **Apparent-multi-inhabitance** (e.g., `String` vs `Box<str>` vs `&str` vs `Cow<str>`) requires axis disposition that conflicts with T-Ground-LanguageSpec's apparent-multi-inhabitance audit (scope item F in `t-ground-languagespec.md`). Coordinate via manager; do NOT pre-empt that lane.
9. **`Option<T>` row reached without Substrate parent decision** — no top-level `type Option<T>` substrate parent exists at HEAD; either Substrate Manager has declared one or extended `OptionalOf` (`dsl/std/algebra.dag:517`) to a full carrier, or this row stays unauthored. Worker MUST NOT introduce a new top-level `Option` substrate type unilaterally (P1-Step-1 violation). (`Result<T, E>` does NOT trigger this STOP — its parent at `dsl/std/error_primitives.dag:12` is live.)
10. **`Cardinal` ordered-domain substrate not landed when retrofit attempted** — array/slice/Vec rows author against `CardinalityBound` at HEAD as a forward-bridge. If a worker reaches retrofit without `Cardinal` having landed, escalate before introducing `Interval<Cardinal>` references in `.dag` data (would break the dissolution trigger).
11. **Non-default hasher or allocator instantiation reached** — §B std-carrier rows enumerate default-bearing signatures (e.g., `Vec<T, A = Global>`, `HashMap<K, V, S = RandomState, A = Global>`) with hasher/allocator as structural axes. Phase-1 dispatch authors only the default-instance rows (`A = Global`, `S = RandomState`); a non-default instantiation (`Vec<T, MyAllocator>`, `HashMap<K, V, FxBuildHasher>`, etc.) is a *distinct realization row*, not a refinement of the default — escalate before authoring, since it expands the row inventory and may force a substrate decision on whether allocator/hasher carriers themselves are first-class declarations or lens-extensible labels.
12. **Repo / git prerequisites unsatisfied** at dispatch time (no clean working tree on the dispatching worker's environment). Implementation cannot start.

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
