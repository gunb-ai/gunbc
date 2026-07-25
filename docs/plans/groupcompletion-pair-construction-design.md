# Design — the completion pattern: `GroupCompletion<M>` and `FieldOfFractions<R>`

**Status:** `GroupCompletion<M>` SIGNED (law grain, sharp-bee-290, 2026-07-25, mandate `msg_6fc2ba88-549b-491e-9b6f-ab949539d682`) and IMPLEMENTED — PR #7197. §2 model fix + §3-eval construction-side collapse + (b) emitter checkpoint-order fix landed together per the mandate's hard condition #1. `FieldOfFractions<R>` MODEL grounding SIGNED under the same completion doctrine (law grain, sharp-bee-290, 2026-07-25, mandate `msg_7a83651e-f370-4b2a-8786-e5c0d81aeaa5` — nod 1: "one grounding doctrine, not two"); its REALIZATION section (§7.3 below) is drafted but **HELD, awaiting explicit sign** before implementation — it is materially new, not the checkpoint trick. Linked from [DESIGN.md](../../DESIGN.md) open threads (numeric-tower bullet).

Owner: eager-crane-304 (completion-pattern grounding lane). Sequence: **GroupCompletion (done)** → **FieldOfFractions model (this note) → FieldOfFractions realization (pending sign, §7.3)** → hollow-alias WALL (design-note-first, capstone, generalizes both specimens — separate note) → Bool → C.

## 0. The completion pattern, stated once (operator ruling, mandate `msg_7a83651e`)

`GroupCompletion<M>` and `FieldOfFractions<R>` are not two anemias of the same *shape* — they are **one algebraic move**, instantiated at two different algebraic strengths, and the operator's ruling is to state the move once and derive both from it rather than risk the two definitions drifting apart (exactly the redundancy §2/§3 forbid):

> **The completion construction.** Given an algebraic structure `S` with a partial "inverse" operation missing (a commutative monoid `M` lacking subtraction; a commutative ring `R` lacking division by non-zero-divisors), freely adjoin the missing inverses by taking **ordered pairs** `(a, b) ∈ S×S` — read as "`a` combined with the formal inverse of `b`" — quotiented by the equivalence relation that says two pairs denote the same completed value iff cross-combining with the *other* pair's second component agrees:
> - **Additive strength (Grothendieck group completion):** `M` a commutative monoid, pair `(pos, neg)` denotes `pos - neg`, `(a,b) ~ (c,d) iff a+d = b+c` (uses only `M`'s `+`).
> - **Multiplicative strength (field of fractions):** `R` a commutative ring (integral domain, no zero divisors), pair `(num, denom)` denotes `num / denom`, `(a,b) ~ (c,d) iff a*d = c*b` (uses only `R`'s `*`).

Both are the **same categorical construction** (a left adjoint / free completion functor) read under the additive vs. multiplicative interpretation of the ambient structure's single binary operation — which is *why* they get one doctrine, not two: the equivalence-relation shape (`a∘d = b∘c` for the ambient op `∘`), the record-of-two-fields carrier shape, and the "record carries the pair, doc-string carries the law, no structural law-checking machinery" precedent (§7.1 below) are identical between them; only which operation of `M`/`R` is being freely inverted differs. A reader who has understood one instantiation should be able to predict the other without re-deriving it — that is the test that this is one law, not two coincidentally-similar ones.

Realization, in contrast, **does not transfer between the two instantiations** — see §7.3.

## 7.1 Specimen 1 — `GroupCompletion<M>` (additive strength, IMPLEMENTED)

### 1. The anemia (confirmed by reading, not assumed)

`dag/std/algebra.dag:38` declares

```
type GroupCompletion<M>
```

with **no body and no `=` alias RHS** — a genuinely hollow declaration, zero fields. `dag/std/integer.dag:21` builds the whole integer tower on it (`type Int = AbelianGroup<GroupCompletion<Nat>>`, chained through `dag/std/nat.dag:6 Nat = CommutativeSemiring<Magnitude>` — the `dag/std` tower's own algebra-witness-composition style). `src/v2/std/integer.dag:22-23` builds a **second, independently-declared, equally hollow** `type GroupCompletion<M>` / `type Int = GroupCompletion<Nat>`, this one chained through `v2.std.nat`'s real Peano `Nat = Zero | Succ{prev:Nat}`.

Confirmed root cause of the emitter symptom: a type declared with no body renders in `src/v1/05_emit_rust.dag` as a `PhantomData` marker (traced: `rust_phantom_marker_inner`/`rust_phantom_field_name`, lines 3480-3490, 4378, 4634, 8178, 9660 — a bodyless generic type becomes a Rust tuple struct or field wrapping `PhantomData<M>`, carrying **no data at all**). A `GroupCompletion<Nat>` (= `Int`) value therefore has nothing to hold an actual integer, which is the direct cause of the `expected i64 found GroupCompletion<Rc<Nat>>` E0308 cluster and the riding E0369 (missing `Add`/`Sub`/`Mul`/`Ord` impls — nothing to implement them *over*) measured in the deep-seven probe (`docs/probes/gate1_repr_mismatch_e0308_diagnosis_2026-07-24.md`, `COPRODUCT_NATIVE_NUMERIC` bucket, Root 4).

**A second, independent defect stacked on top:** `src/v2/std/integer.dag:22`'s local `type GroupCompletion<M>` is a **§3 fork** — a second declaration of the same concept `std.algebra` already names, not imported. This duplicates the hollowness (fixing only `dag/std/algebra.dag`'s copy would leave the v2 corpus's own copy — the one the deep-seven probe actually compiles through — still bodyless).

### 2. The model fix — Grothendieck pair construction

Ground `GroupCompletion<M>` at its single authority, `dag/std/algebra.dag:38`, as the standard construction that freely completes a commutative monoid to a group: pairs `(pos, neg) ∈ M×M` denoting `pos - neg`, quotiented by `(a,b) ~ (c,d) iff a+d = b+c`.

```
type GroupCompletion<M> {
  pos: M
  neg: M
}
```

This mirrors the file's existing convention exactly — `Group<T>`/`AbelianGroup<T>`/`Ring<T>` etc. are all plain records of their operations, none of them encode their algebraic *laws* structurally (no law-checking machinery exists for Monoid/Group associativity either) — so a record carrying the pair with the equivalence relation documented as a doc-string, not enforced structurally, is consistent with every neighboring declaration in this file, not a new pattern invented for this one type.

**De-fork (§3, same PR):** delete `src/v2/std/integer.dag:22`'s local `type GroupCompletion<M>`; add `GroupCompletion` to its existing `import std.algebra { Cons, Empty, FreeMonoid, AbelianGroup, OrderedRing, Ordering, Less, Equal, Greater }` (line 3). One declaration, both towers reference it.

`dag/std/algebra.dag:43`'s `type FieldOfFractions<R>` is the second instantiation of the §0 completion pattern (the analogous construction for fields — pairs `(num, denom)` with `(a,b)~(c,d) iff a*d = c*b`) — designed in §7.2/§7.3 below, folded into this same note per the operator's nod-1 ruling rather than a separate design file.

### 3. Construction-side native grounding — mirroring #5428 exactly

`v1_interpreter.rs` grounds Peano `Nat` construction-side today: `eval_var` (`Zero` bound as a `VariantValueBinding` → `Value::Int(0)`, line 2041) and `eval_record_lit` (`Succ{prev: Value::Int(p)}` → `Value::Int(p+1)` directly, no boxed `Value::Variant` ever built, line 4256-4261) — the coproduct is the *model*, native `Value::Int` is the *realization*, collapsed at the moment of construction, verified by `match_pattern`'s symmetric `Value::Int(n)` destructuring against `Zero`/`Succ{prev}` patterns (lines 2742-2770).

The mirrored construction-side collapse for the pair: in `eval_record_lit`, when constructing `GroupCompletion{pos, neg}` (no `parent_enum` — this is a plain record type, not a coproduct variant, so it takes the `else` branch at line 4268 today) and both `pos`/`neg` evaluate to native `Value::Int`, collapse directly to `Value::Int(pos - neg)` rather than building a boxed `Value::Record`. Symmetric `match_pattern` support (destructuring a native `Value::Int(n)` against a `GroupCompletion{pos, neg}` pattern) needs the canonical-representative choice for the inverse direction — `(max(n,0), max(-n,0))` — analogous to how `Succ{prev}` reconstructs `n-1` from `Value::Int(n)`. I have not found any corpus site that destructures `GroupCompletion{pos,neg}` directly today (same as Nat's `Succ{prev}` pattern match was presumably rare pre-#5428), so this arm is defensive completeness, not a currently-exercised path — flagging so you can confirm it's worth landing now vs. deferring to an actual witness.

### 4. Open question — the deep-seven Rust-emission side (need your ruling before I implement this part)

§3 is unconditional (the v1 seed interpreter has no `corpus_repr` axis — it just executes `.dag`). The **emitted-Rust** side is where I'm not confident which shape you intend, and your doctrine text ("native realization grounds to the machine-width Int axis") is consistent with more than one construction:

- **(a)** Once `GroupCompletion<M>` has a real body, it emits as a genuine 2-field Rust struct under `FaithfulFreeMonoid` (the deep-seven probe's corpus_repr) instead of `PhantomData` — this alone kills the E0308 (`expected i64 found GroupCompletion<Rc<Nat>>` becomes a real, non-phantom, non-mismatched type), but `int_add(a,b){ a + b }` (`src/v2/std/integer.dag:863`, and `int_sub`/`int_compare`/etc., all written directly against Rust's `+`/`-`/`<`/`==` operators) then needs `impl Add/Sub/Mul/PartialOrd/PartialEq for GroupCompletion<M>` generically derived from `M`'s own impls (pairwise on `pos`/`neg`, canonicalizing through the equivalence relation). That is **exactly** Root 4 / sub-wall #2 from the gate1 diagnosis ("trait-derive-completeness predicate... same shape as the #6776 wrap-decision predicate, applied to a different axis") — already identified, not yet owned, not part of what I've scoped here.
- **(b)** `Int` already has its own Rust checkpoint (`dag/extdeps/languages/rust/types.dag:16`, `{dag_name: "Int", target_type: "i64", ...}`, single-authority, precedent for `Int8`…`UInt128`'s `Compose<Int, MachineWidth<N>>` rows grounding to bounded machine ints by *derivation* rather than by body). The checkpoint lookup fires on the leaf name reached *after* alias unfolding; today unfolding walks past `Int` down to `GroupCompletion`, so the existing `Int → i64` row never gets consulted. If `Int` is meant to stay unconditionally native (always `i64`, regardless of `corpus_repr`) — which is what "grounds to the machine-width Int axis" most directly reads as — the fix is making checkpoint lookup consult **each alias-unfolding step**, not only the fully-peeled leaf, so the already-registered `Int` row wins before reaching the (now real, but not machine-native) `GroupCompletion` struct underneath it. This is *not* a new name-match rule on `GroupCompletion` (nothing new keys off that name) — it reuses the single authority that already exists for `Int` — but I want your confirmation this isn't the shape of patch your mandate rules out, since it still touches `05_emit_rust.dag`.

I did not want to guess between (a) and (b) and land the wrong one under a "model-before-implement" mandate — this is the one place your ruling changes what I build. My read: (b) is what actually burns down the deep-seven NUMERIC_TOWER count measurably (native `i64`, zero new trait-impl surface); (a) alone converts E0308 into a differently-shaped E0369 until sub-wall #2 lands separately, which may not be this lane's job to also build.

### 5. Discriminating RED (green-by-execution, not typecheck)

Witness, modeled on `ct_diagnostics_carrier_grounding_test` (`src/v1/compiler_tests_rust.dag:553-593`) plus the pair-shaped fixture pattern in `ct_rust_btree_set_ord_eligibility_test` (`shaped_type_node`/`named_type_node`, lines 598-623):

- **Interpreter witness** (§3): evaluate `GroupCompletion{pos: 5, neg: 2}` and `GroupCompletion{pos: 1, neg: 4}` through `eval_record_lit`; assert both collapse to native `Value::Int(3)` / `Value::Int(-3)` (not `Value::Record`); assert `nat_add`-style combination of the two matches native `Value::Int` subtraction — the same shape as `cross_representation_equality_test` (`src/v1/tests/src/cross_representation_equality_test.rs`), red if the pair ever surfaces as a boxed record where arithmetic expects a native int.
- **Emitter witness** (§4, once (a) vs (b) is settled): render `GroupCompletion<Nat>` (built via `shaped_type_node("GroupCompletion", [named_type_node("Nat")])`) through `render_rust_type(..., RustCorpusRepr::FaithfulFreeMonoid, ...)` and assert the emitted string is no longer `GroupCompletion<Rc<Nat>>`-with-`PhantomData`-body (path (a): assert real `pos`/`neg` fields present; path (b): assert `"i64"`).
- **Deep-seven burn-down measurement — DONE, measured post-fix (2026-07-25):** reran the classifier-v3 probe (`gunbc compile --source-root dag --source-root src/v2 --entry <module> --target rust --dependency-pool-index primary-precedence` → `cssl_assemble` → `cargo build --release --lib`) across 5 of the 7 modules probed directly — `06_translate`, `04_infer`, `05_eval`, `emit_host`, `materialization_carriers`; `05_emit` and `emit_module` skipped as byte-identical to `06_translate` per the 2026-07-24 baseline doc, an assumption re-verified this pass by running `05_emit` directly post-drift (byte-identical histogram, `GroupCompletion` mentions = 0). Baseline (2026-07-24, pre-fix): 3 occurrences of the `expected i64 found GroupCompletion<Rc<Nat>>` shape in `06_translate` alone, 1–5% share across all 6 deep-family modules. **Measured post-fix: the `GroupCompletion<Rc<Nat>>` COPRODUCT_NATIVE_NUMERIC marker is zero across all 7** (a clean, drift-immune signal vs. raw aggregate E0308 counts, which are confounded by ~5 unrelated commits that landed on `main` between the baseline and this measurement). **Residue found, not absorbed:** `materialization_carriers` retains 3 `GroupCompletion<...CommutativeSemiring<Magnitude>...>` mismatches (`std/measure.dag`'s `Measure`, a non-Int base) — Root-4 arithmetic-trait-derivation, out of this lane's scope per §4(a) above, tracked separately by sharp-bee-290 (confirmed distinct from silent-badger-23's #7174 scope). This rerun is the receipt; a reused-authority-pattern-plus-cargo-checks-pass without it would not have been accepted as a fix per the mandate.

### 6. Scope boundary

`v2.std.integer`'s own modeled arithmetic (`int_add`, `int_sub`, `int_compare`, … lines 863-995) is **not touched** — those already delegate to native operators, which is exactly what this design makes valid again. `v2.std.nat`'s real Peano `Nat = Zero | Succ{prev:Nat}` is **not touched** — it stays the faithfully-modeled recursive type; only `GroupCompletion<M>` (its own separate concept, one layer up) gets a body. Nothing in this design proposes routing by the literal name `"GroupCompletion"` at the emitter as a special case disconnected from the model — every consumer (interpreter, emitter, checkpoint table) reads the same single grounded declaration.

## 7.2 Specimen 2 — `FieldOfFractions<R>` (multiplicative strength, MODEL SIGNED)

### 1. The anemia (confirmed by reading)

`dag/std/algebra.dag:43` declares

```
type FieldOfFractions<R>
```

— no body, no `=` alias RHS, the same genuinely-hollow shape as `GroupCompletion<M>` was before §7.1. Two consumers chain through it, both in `dag/std`:

- `dag/std/rational.dag:4` — `type Rational = Field<FieldOfFractions<Int>>`
- `dag/std/float.dag:10` — `type Real = ApproximateField<FieldOfFractions<Int>>` (and `Real32`/`Real64`/`Float32`/`Float64`/`Float` compose on top of `Real`)

**No §3 fork exists for `FieldOfFractions`** — confirmed by corpus-wide grep (`grep -rn "FieldOfFractions" --include=*.dag .` and a direct scan of `src/v2/std/`): the only declaration is `dag/std/algebra.dag:43`; `Rational`/`Real` both reference it via `import std.algebra { Field, FieldOfFractions }`, no independent redeclaration anywhere in `src/v2`. This is a genuine difference from the `GroupCompletion` specimen, not an oversight on my part — the §7.1 de-fork step (§7.1.2, "delete `src/v2/std/integer.dag:22`'s local declaration") **has no analogue here**: there is nothing to de-fork, only a body to add.

The emitter symptom this anemia would cause is the same `PhantomData` collapse as §7.1.1 — a bodyless `FieldOfFractions<Int>` renders as a marker carrying no data — but it is **not yet measured** in the deep-seven probe the way `GroupCompletion` was, because `Rational`/`Real`/`Float` are not among the 7 modules that probe covers; this note does not claim a measured pre-fix baseline for `FieldOfFractions` the way §7.1.5 has one for `GroupCompletion` (flagged honestly rather than reusing the `GroupCompletion` numbers by association — the two anemias are the same *pattern*, not the same *measurement*).

### 2. The model fix — field of fractions pair construction

Ground `FieldOfFractions<R>` at its single authority, `dag/std/algebra.dag:43`, as the §0 completion pattern under multiplicative strength: pairs `(num, denom) ∈ R×R` denoting `num / denom`, quotiented by `(a,b) ~ (c,d) iff a*d = c*b`.

```
type FieldOfFractions<R> {
  num: R
  denom: R
}
```

Same convention-fit argument as §7.1.2: `Field<T>`/`Ring<T>`/`OrderedRing<T>` etc. in `algebra.dag` are all plain records of operations with laws documented, not structurally enforced — a record carrying `(num, denom)` with the cross-multiplication equivalence as a doc-string is the existing house style, not a new pattern. No de-fork step is needed (§7.2.1) — this is a pure add-body.

**What this model fix does NOT do:** it does not choose a canonical/reduced representative (e.g. dividing by `gcd(num, denom)`), does not forbid `denom = 0` structurally (no refinement-type machinery exists in this corpus to express "non-zero" as part of the type, matching `GroupCompletion`'s equally-unenforced law), and does not specify construction-side collapse behavior — that is §7.3, held pending sign because (unlike §7.1.3) there is no native scalar for it to collapse *into*.

### 3. The realization — materially NEW, not the checkpoint trick (HELD pending sign)

This is the section the operator asked to see before I implement anything here (mandate `msg_7a83651e`, nod 1). It is presented for sign, not landed.

**Why §7.1's realization path does not transfer:** `GroupCompletion<Nat>` (= `Int`) had an escape hatch `FieldOfFractions<Int>` does not — `Int` already owns a registered Rust checkpoint (`dag/extdeps/languages/rust/types.dag:16`, `Int → i64`), so §7.1's realization fix was "make the existing checkpoint lookup fire earlier in the alias-unfolding walk" (§7.1.4(b)) — zero new trait-impl surface, because native `i64` arithmetic was already correct. **`Rational` has no machine-scalar checkpoint** — there is no `rational → f64`-style row in `dag/extdeps/languages/rust/types.dag` today, and inventing one would be dishonest: `f64` is lossy (not the field of fractions over `Int`, a different real-number approximation entirely — that's precisely what `Real`/`ApproximateField` already names as a *distinct* type from `Rational`/`Field`). So the (b) collapse-to-native-scalar move is **not available** for this specimen; a fabricated checkpoint here would be exactly the "forbidden dodge" the mandate names (§5's workaround trap — routing around a genuine modeling gap with a plausible-looking shortcut).

**What realization actually requires:** `Rational`/`Real` values must realize as genuine `{num, denom}` structs, in both places a value lives:

- **Interpreter (`v1_interpreter.rs`):** `eval_record_lit` constructing `FieldOfFractions{num, denom}` builds a real `Value::Record` — **no collapse arm**, unlike §7.1.3's `GroupCompletion → Value::Int`. This is a smaller change than §7.1.3, not a larger one: the "no fix needed" branch (`eval_record_lit`'s existing `else` path building `Value::Record`) is already correct once the type has a body; nothing new to write here beyond confirming no accidental collapse gets added by analogy-copying §7.1.3's code without checking the checkpoint precondition first.
- **Emitted Rust (`05_emit_rust.dag` / `v1_compiler_emit_rust.rs`):** once `FieldOfFractions<R>` has a body, it stops emitting as `PhantomData` and starts emitting as a real 2-field struct (the §7.1.4(a) path, not (a)+(b) — there is no (b) here). `v2.std.rational`'s modeled arithmetic (if/when it exists — not yet audited as part of this note) would need `impl Add/Sub/Mul/PartialOrd/PartialEq for FieldOfFractions<R>` generically derived from `R`'s own `Ring`/`OrderedRing` witnesses, cross-multiplying per the §0 equivalence relation. **This is the same Root-4 arithmetic-trait-derivation gap** the `materialization_carriers`/`Measure` residue (§7.1.5) already surfaced and sharp-bee-290 is routing separately — `FieldOfFractions<R>` realization is a *second site* that couples to that same missing derivation, not a new independent gap.

**Staging (this is the ask for your sign):** land the §7.2 MODEL grounding (body only) now, under the already-signed completion doctrine — it is decidable, bounded, and has no dependency on Root-4. **Stage the emit/arithmetic REALIZATION burn-down behind Root-4**, with a declared dissolution trigger: *when the Root-4 generic-operator-trait-derivation predicate lands (silent-badger-23 / gate1 sub-wall #2), `FieldOfFractions<R>`'s `Add`/`Sub`/`Mul`/`Ord` impls are generated by that same predicate, not hand-written per-type here.* Until then, a `FieldOfFractions<R>` value type-checks and interprets correctly (§7.2.3 interpreter path) but is not claimed to compile to arithmetic-correct emitted Rust — that residue is named and counted, exactly as §7.1.5 named the `Measure` residue, never silently absorbed.

### 4. Discriminating RED (for the model half only — realization RED is written when §7.3 realization is signed)

- **Model witness:** construct `FieldOfFractions{num: 3, denom: 4}` through the interpreter and assert it evaluates to a `Value::Record{type_name: "FieldOfFractions", fields: [("num", 3), ("denom", 4)]}` — i.e. explicitly assert it does **NOT** collapse to a native scalar (the negative-space complement of §7.1.5's interpreter witness — red if some future edit accidentally adds a collapse arm here by pattern-matching §7.1.3 without checking the no-checkpoint precondition).
- **De-fork non-regression witness:** assert `grep -rn "type FieldOfFractions" --include=*.dag .` returns exactly one declaration (`dag/std/algebra.dag`) — red if a future edit reintroduces a v2-local fork the way `GroupCompletion` had one.
- **Realization RED is deferred to §7.3 sign-off** — no emitter witness is written until the realization section above is signed, per DESIGN §5 ("done means green-by-execution," which does not apply to work not yet authorized to start).
