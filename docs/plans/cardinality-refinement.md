# Plan — Cardinality refinement (the decidable fold-propagated refinement axis)

**Status:** scoping (Feature-1 from the byte-grounding thread). **DESIGN.md + carriers are authority** (§6 no parallel ledger); each item dissolves into a wired parser/checker change + a green witness when it lands.

**Verified against the live tree 2026-06-22.** Line numbers are receipts; re-check before acting.

## 0. Thesis — cardinality is ONE axis, and it is the *decidable* refinement fragment

"Is it empty", "does it have N", "did `Int64` overflow" are the same question — a **cardinality constraint** — answered today by scattered manual `if list_length(..) == 0` / `count` / bound checks. Model cardinality as a refinement **axis** and two things fall out:

1. **Decidability (why this fragment, not general refinement).** Arbitrary value predicates (`admits: fn(B) -> Bool`, `refinement.dag:34`) are undecidable, so they can only be checked at a runtime constructor boundary. **Cardinality** predicates — `length == N`, `≥ 1`, `magnitude < 2^width` — are linear arithmetic over counts: **decidable**, hence checkable *statically* and fold-propagated. Scoping refinement to cardinality is what keeps it inside the §4 bounded/decidable substrate; general refinement would break it.
2. **Fold-propagation (the payoff).** A catamorphism that carries the cardinality means folding a `List<Bit>` yields its length, adding two bounded `Int64` yields the combined bound (and **overflow is a typed `Rejected`, not a silent wrap**). "When we fold, it's handled automatically" — the empty/count/width checks stop being hand-written.

## 1. What is ALREADY built (this is wiring, not greenfield)

- **Value-level refinement substrate — `v2.std.refinement`:** `Validation<B> { reason, admits: fn(B)->Bool }`, `Refined<B> { base }`, and `refine<B>(base, by, at) -> Outcome<Refined<B>>` (`:74`) which checks `admits(base)` and returns `Accepted{Refined}` / `Rejected` — **fail-closed, already.** Plus hoisted int refinement factories and iteration-ordering refinements.
- **Its own documented gaps:** `🟡 feature:refinement-opaque-carrier / T-25-tail` (`:65`) — bare `Refined { base }` can still bypass `refine`; construction is **not compiler-enforced** yet.
- **The phantom-width gap — `std.machine_constraints`:** `type MachineWidth<bits>` (`:47`) is a phantom; its TRACKED PARAMETER GAP (`:34-46`) names the exact trigger: *"substrate grammar for bounded phantom parameters tying `bits` to `Nat`, or a non-phantom `MachineWidth` indexed by an explicit `Nat` carrier."* No reflection of `N` to a value.
- **The structural carriers exist, unchecked — `std.bit`:** `Byte { bits: List<Bit> }` (`bit.dag:25`), header concedes *"cardinality is not enforced … no lowered field refinement, type-alias `where` is skipped in the handwritten parser."*
- **A waiting consumer — `std.measure`:** `Refined<Measure<…>, predicate>` is explicitly **deferred** (`measure.dag:8,245`), waiting on exactly this.

## 2. The gap, decomposed (each piece → a real surface)

- [ ] **P1 — surface `where` syntax, desugaring to `refine`/`Validation`.** Parser: extend the type-RHS path (`parse_type_rhs_after_eq`, named in `machine_constraints.dag:109`) and the record-field path to accept `where <cardinality-pred>`; lex the `where` keyword. Normalize: at the `^dag_surface_type_alias_rhs` hook (`03_normalize.dag:90`) lower `where P` into a `Validation` + a **refined-construction obligation** on the type. MVP = alias/field-level `where`; the predicate is from the closed cardinality vocabulary (P3), not an arbitrary expression.
- [ ] **P2 — compiler-enforced construction (closes T-25-tail).** Checker: at a construction site of a refined type, require the obligation discharged — bare `Byte { bits: … }` outside the sanctioned `refine`/smart-constructor is a located `Rejected`, never silent. This is what turns "documented intent" into "illegal-state-unwritable."
- [ ] **P3 — the closed cardinality predicate vocabulary (the decidable fragment).** A small closed set grounded in `Validation.admits`: `Length<N>` (`== N`), `NonEmpty` (`≥ 1`), `Bounded<Lo,Hi>`, `Width<N>` (the `MachineWidth` bound). All are linear-arithmetic over `list_length` (`types.dag:233`) / magnitude — decidable. `Byte = { bits: List<Bit> } where Length<8>`; `Int64`'s magnitude is `Bounded<0, 2^64>`. **No arbitrary predicate enters static checking.**
- [ ] **P4 — fold-propagation (the novel, high-value piece).** Extend the catamorphism (`fold_node` / the reduce) so a cardinality fact is COMPUTED through a fold: `Cons/Empty` over a list yields its length; combining two `Bounded` magnitudes yields the combined bound, and a bound exceeding the `Width` is a typed overflow `Rejected`. Connect to the existing cost-through-folds algebra (`induction.dag` PolyCost/exponents) — cardinality is the same shape (a count tracked through a catamorphism) as the cost lens already computes.
- [ ] **P5 (stretch) — type-level-Nat reflection.** Reflect `MachineWidth<N>`'s `N` to a value (the phantom→value bridge named in `machine_constraints.dag:43-46`), so `bits_per_byte` dissolves into `width(Byte)` and `Int64`'s bound derives from its type rather than a literal.

## 3. Where it plugs into the pipeline

`tokenize` (lex `where`) → `parse` (`parse_type_rhs_after_eq`: parse the cardinality pred, attach to the type-decl `Node`) → `normalize` (`dag_surface_type_alias_rhs`: desugar to `Validation` + the construction obligation) → `infer` (`04_infer`: discharge the obligation at construction sites — the decidable cardinality check — and fold-propagate cardinality facts) → `emit` (refinements erase, or ground to a target assert; they are compile-time).

## 4. The decidability boundary (the one rule that keeps this §4-legal)

Two tiers, and the split is the whole discipline:

- **General predicates** (`admits: fn(B)->Bool`) stay **runtime** `refine` (already built) — arbitrary, undecidable, checked at a constructor boundary.
- **The cardinality fragment** (P3) is **decidable** → it is the only thing lifted to *static* checking and *fold-propagation*. Do not let an arbitrary predicate into P3/P4; the moment static checking admits an undecidable predicate, the bounded-execution axiom is broken.

## 5. MVP / phasing (smallest green-by-execution first)

- **MVP-1 — ground `Byte`.** `where Length<8>` on the `bits` field, desugared to a cardinality `Validation`, construction routed through `refine`. Witness: an 8-bit byte is `Accepted`, a **7-bit byte is a typed `Rejected`** (the discriminating red). At this point `bits_per_byte`'s `8` lives once, in `Length<8>`, and `bit.dag`'s "length not checked" comment dissolves.
- **MVP-2 — one fold case.** Fold `List<Bit>` → a length cardinality fact; the scattered empty/length checks at that consumer dissolve into the fold result.
- **Phase-2 — `NonEmpty` / `Bounded`; the overflow case** (`Int64` magnitude bound through `add` → overflow as `Rejected`).
- **Phase-3 — P5 reflection; lift cardinality checks fully static** where the count is known at type level.

## 6. What one feature unblocks (the ROI)

Byte=8bits; the scattered empty/count/length checks → cardinality facts; `Int64` overflow → a bound through the fold; `Measure<Q,S>` inhabitance (the deferred `Refined<Measure>`); typed `|>` coercion (the Measure-inhabitance gap blocked it last week); `NonEmptyStr`/`NonEmptyDiagnostics` → `NonEmpty`. One axis, many groundings.

## 7. Risks / hard parts

- **Decidability discipline** (§4) — the failure mode is P3/P4 quietly admitting a non-cardinality predicate; gate it to the closed vocabulary.
- **Fold-propagation soundness** — the cardinality a fold computes must be *provably* the real one (a fail-closed witness: a fold that miscounts goes RED).
- **The parser is handwritten** (`bit.dag` header) — `where` lexing/parsing touches the seed parser (load-bearing; sequence behind the §0 lock-down, not during it).
- **P2 is broad** — construction-enforcement touches every construction site of a refined type.

## Dissolution trigger (DESIGN §6)

Delete this doc when `Byte = { bits: List<Bit> } where Length<8>` is authored, its construction is compiler-enforced, and a 7-bit byte is a typed `Rejected` green-by-execution — at which point `bit.dag`'s "length not checked", `bits_per_byte`'s centralized `8`, and the `MachineWidth` phantom gap all dissolve into the carriers, and this scoping is redundant.
