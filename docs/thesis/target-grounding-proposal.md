# Target Grounding Proposal — Worked Examples and Implementation Scope for Structural Coercion

> **Mode:** `PROPOSAL`

## Status of this proposal

**Parallel to R1. Non-blocking. Concrete elaboration of prior design.**

The architectural claim at the heart of this doc — that coercion at
the realization boundary should dissolve from table-driven lookup
(`TypeCheckpoint` / `InhabitantDecl` / `carrier: String`) into
structural algebra-homomorphism matching — is **not new here**. It
is committed design, authored in
[`docs/single-emitter-design.md`](../single-emitter-design.md) (481
lines). That doc is the **parent authority** for the dissolution
track; its §"The Insight: Algebra Homomorphism, Not Lookup Tables"
and §"What dissolves" are the source of record for the end state.

This proposal is the **concrete elaboration** layered on top of
`single-emitter-design.md`:

- Five worked examples walking both sides of the coercion (Int64,
  Bool, String, List<T>, Option<T>).
- Six-layer scope partition of the implementation work.
- Work estimate (~2–3 months; ~120–150 primitive declarations).
- R1-era non-preclusion discipline.
- Fail-closed tie-breaking semantics.
- L4 verification three-way split: (A) routing correctness via
  routing-stability tests + narrower choice-differential,
  (B) structural-shape consistency by construction, and
  (C) algebra-satisfaction certification preserved as L4
  witness-based against target-runtime behavior (target-side
  algebra-inhabitance claims remain theorem claims, not
  self-certifying).
- Open design questions (tie-breaking, escape hatches, cross-type
  coercion paths, failure diagnostics, interaction with DB-18 +
  cardinality-substrate).

Promotion to committed work requires a follow-up PR amending
`THESIS.md` §"Correctness dimensions" / §"What falls out" to
reference this doc as the concrete path for `single-emitter-design.md`'s
end state. The work-estimate table below is directional; a
promotion PR pins the Rust/Python/Go primitive enumeration to
actual declarations.

## Referenced authorities

- **[`docs/single-emitter-design.md`](../single-emitter-design.md)**
  — parent authority. "Coercion IS emission. Target types declare
  algebraic identity; compiler finds homomorphism. `TypeCheckpoint`
  and `InhabitantDecl` dissolve." Scheduled for dissolution under
  ROADMAP Track 13 ("single-emitter"). Everything this proposal
  describes is a concrete path toward the end state that doc
  names.
- **[`two-groundings-static-validation-vs-efficient-realization.md`](two-groundings-static-validation-vs-efficient-realization.md)**
  — frames the deep-static vs shallow-realization asymmetry. This
  proposal's L4-split section engages directly with that framing.
- **[`correctness-dimensions.md`](correctness-dimensions.md)** +
  **[`concept-unification.md`](concept-unification.md)** — source
  of "coercion = emission; coercion cost = complexity." Relevant
  to the "no separate coercion phase" discipline this proposal
  preserves.
- **[`coercion-design.md`](../coercion-design.md)** — bootstrap
  design describing the manual scaffolding
  (`TypeCheckpoint`/`InhabitantDecl`/`CallableRepr`/`CastSyntax`).
  Accurate for the transitional state; its header-level status
  note currently reads "Design sketch (no compiler changes)"
  which is stale — the schema landed in `dsl/std/coercion.dag`.
  That doc's content is the status-quo authority this proposal's
  worked examples walk against.
- **[`target-realization-efficiency.md`](target-realization-efficiency.md)**
  and **[`what-falls-out.md`](what-falls-out.md)** — adjacent on
  per-primitive realization cost and the L4-verification
  completeness gap.
- **[`../invariants/scaffold-boundaries.md`](../invariants/scaffold-boundaries.md)**
  — governance authority on substrate scaffolds (must carry
  dissolution trigger + unreachability gate). `TypeCheckpoint`/
  `InhabitantDecl` are scaffolds under this rule; `single-emitter-design.md`
  provides their dissolution path.
- **[`../invariants/verifiability-invariant.md`](../invariants/verifiability-invariant.md)**
  — adjacent on L4 verification; notes the weather.dag PoC and
  the witness-generation gap.

## What this proposal does NOT restate

These are already authoritatively covered in `single-emitter-design.md`
and are not re-argued here:

- Why table-driven coercion is bootstrap scaffolding, not the end
  state.
- The "coercion IS emission" framing.
- That `TypeCheckpoint`/`InhabitantDecl` dissolve in the end state.
- That coercion cost should be complexity (no separate lattice).
- The general thesis that target mapping falls out of algebraic
  identity on both sides.

Reading this proposal without reading `single-emitter-design.md`
first will make the worked examples feel ungrounded. The worked
examples are only meaningful in the context of the architectural
claim that doc already establishes.

## Scope of this proposal

`[proposed]` — model target primitives structurally in `.dag`, the
same way user types are modeled structurally, and let coercion at
the realization boundary be a **structural inhabitance-search**
rather than a table lookup. This is the same architectural move
`single-emitter-design.md` calls "Algebra Homomorphism, Not Lookup
Tables" — here with concrete examples, layer partition, and work
estimate.

**Shape of the change (concrete form of `single-emitter-design.md`'s §"What dissolves"):**

- Target primitives get their own `.dag` models. For Rust, `i8`,
  `i16`, `i32`, `i64`, `i128`, `u*`, `f32`, `f64`, `bool`, `char`,
  `String`, `str`, `Vec<T>`, `[T; N]`, `&[T]`, `Option<T>`,
  `Result<T, E>` each get declared with their algebra + carrier +
  target-specific qualifiers (signedness, overflow behavior,
  ownership, memory layout, ...). Each declaration has the same
  shape user types have.
- The emitter no longer reads `carrier: "i64"` from a declaration.
  Given a user type, it searches the target's primitives for the
  **minimum-satisfying** primitive — the narrowest declared
  primitive whose algebra and carrier satisfy the user type's
  declared algebra and carrier requirements.
- Coercion paths (e.g., `Char → List<Byte>` via UTF-8 when mapping
  `FreeMonoid<Char>` to Rust's `FreeMonoid<u8>`) become explicit,
  declared transforms derivable from the two models.
- Tests verify the routing is stable ("user's `Int64` resolves to
  Rust's `i64`, never `i128` or `Vec<Bit>`") rather than
  verifying behavior after the fact.

## Current state referenced

`[live]` — the bootstrap-scaffolding state this proposal's worked
examples walk against (per `single-emitter-design.md`'s "current
state" framing):

- `TypeRealization` declared at `src/v3/std/emit_model.dag:15` —
  carries `carrier: String` field.
- `TypeCheckpoint`/`InhabitantDecl`/`CallableRepr`/`CastSyntax`
  at `dsl/std/coercion.dag:38,59,77,98`.
- Per-target spec files declare data:
  `data rust_int: TypeRealization = { target: Int, carrier: "i64", ... }`
  (`src/v3/spec/rust.dag:103`); same shape in `python.dag`,
  `go.dag`.
- Per-target coercion data in `dsl/extdeps/languages/{rust,python,go,dag}/types.dag`
  instantiates the `TypeCheckpoint` / `InhabitantDecl` tables.
- Rust emitter at `src/v3/compiler/src/emit/rust_target.rs` reads
  the table and paste-renders via `render_named_template` at
  `:1461` (15+ call sites).
- L4 verification — the consistency-check between the two
  groundings — listed as partial in
  `docs/thesis/what-falls-out.md:33`.

## Six-layer scope

`[proposed]` — the full implementation partitions into six layers:

| Layer | Content | Character |
|---|---|---|
| 1. Language-spec model | Each target language's primitives declared structurally in `.dag` — algebra, carrier width, overflow semantics, ownership, layout intent | Data entry sourced from language references |
| 2. Compiler-contract model | Target-compiler-specific behavior not pinned by the language spec — Rust edition defaults, `rustc` overflow-check flags, Python minor-version differences, Go generics era | Smaller data entry; updates when toolchains shift |
| 3. Platform-contract model | Target-runtime specifics: pointer width, endianness, wasm constraints, `usize` variability | Smallest layer; per-(target, platform) pair |
| 4. Coercion engine | Inhabitance-search over target primitives + minimum-satisfier selection + tie-breaking + diagnostics | Substrate work in the compiler |
| 5. Coercion-routing tests | For every user type, assert the resolved target primitive. For every target primitive, assert it's reachable via some user type (no orphans) | One-time test-suite build |
| 6. Dissolution (not migration) | `TypeRealization.carrier: String` dissolves when Track 13 (single-emitter) lands per `single-emitter-design.md`'s "What dissolves" table. No deprecation period; no parallel authority. Explicit dissolution trigger: the Track 13 closure deletes both `TypeCheckpoint` and the `carrier: String` field. Interim verification (during the layers 1–5 build-up before dissolution fires): derived structural outputs match the declared-table outputs on existing entries. When parity holds, Track 13 fires and the table is deleted | Dissolution-triggered, not migration-paced |

## Worked examples

Five current types, each with its gunbc-side declaration, the
proposed target-side declarations, and the coercion walk.

### 1. `Int64`

**User side** (`[live]`, `dsl/std/integer.dag:34`):

```dag
type Word64 { bytes: List<Byte> }
type Int64 = OrderedRing<Word64>
```

**Rust side** (`[proposed]`):

```dag
// Rust signed integer family
type RustI8   = OrderedRing<Word8>
  where signed, wrap_on_overflow, native_cpu_op
type RustI16  = OrderedRing<Word16>  where signed, wrap_on_overflow, native_cpu_op
type RustI32  = OrderedRing<Word32>  where signed, wrap_on_overflow, native_cpu_op
type RustI64  = OrderedRing<Word64>  where signed, wrap_on_overflow, native_cpu_op
type RustI128 = OrderedRing<Word128> where signed, wrap_on_overflow, emulated_cpu_op
type RustISize = OrderedRing<WordPointer>
  // platform-contract edge carries WordPointer width
```

**Coercion walk:**

1. User's `Int64` inhabits `OrderedRing<Word64>`.
2. Search Rust primitives for `OrderedRing<Word_N>` where
   `N >= 64` and the primitive is signed.
3. Candidates: `RustI64`, `RustI128`.
4. Minimum-satisfier: `RustI64` (narrower carrier).
5. Emit: `i64`.

**What this catches the current design doesn't:** if gunbc ever
redeclared `Int64` to require saturation instead of wrap, the
search would fail to find a matching Rust primitive and surface a
diagnostic. Current design silently emits `i64`; behavior would
now disagree with declared algebra.

### 2. `Bool`

**User side** (`[live]`):

```dag
type Bool = BooleanAlgebra<Bit>
```

**Rust side** (`[proposed]`):

```dag
type RustBool = BooleanAlgebra<Bit>
  where byte_width(1), variants(True, False)
```

**Coercion:** 1:1 match on algebra. Emit `bool`. Trivial — and
deliberately so. The mechanism only does real work when there's
more than one candidate. Simple cases stay simple.

### 3. `String`

**User side** (`[live]`):

```dag
type Char   = Int where 0 <= x && x <= 0x10FFFF   // Unicode scalar
type String = FreeMonoid<Char>
```

**Rust side** (`[proposed]`):

```dag
type RustChar   = Int where 0 <= x && x <= 0x10FFFF
  where byte_width(4), excludes_surrogates
type RustString = FreeMonoid<RustByte>
  where utf8_encoded, owned, heap_allocated
type RustStr    = FreeMonoid<RustByte>
  where utf8_encoded, borrowed, unsized
```

**Coercion walk — the non-trivial case:**

1. User's `String` inhabits `FreeMonoid<Char>`.
2. Rust has no `FreeMonoid<Char>` directly. It has
   `FreeMonoid<RustByte>` with a UTF-8 invariant.
3. The coercion search finds a **two-step path**:
   `FreeMonoid<Char> → FreeMonoid<RustByte>` via a declared UTF-8
   encode transform (derivable from `Char`'s range and the UTF-8
   encoding rule).
4. Destination: `RustString` (owned) is minimum-satisfying; `RustStr`
   (borrowed) requires user signal of borrowed intent.
5. Emit: `String`.

**This is where the structural-coercion design has the most bite.**
The current `carrier: "String"` silently papers over the UTF-8
encoding dimension — `.dag String` is declared to realize as Rust
`String`, and no mechanism checks that the element types are
structurally reconcilable. Structural coercion makes UTF-8
encoding an explicit, derivable step, and the test suite verifies
the encoding round-trips cleanly.

### 4. `List<T>`

**User side** (`[live]`, `dsl/std/types.dag:211`):

```dag
type List<T> = FreeMonoid<T>
```

**Rust side** (`[proposed]`, multiple candidates):

```dag
type RustVec<T>      = FreeMonoid<T>
  where contiguous, owned, heap_allocated, cardinality_unbounded
type RustArray<T, N> = FreeMonoid<T>
  where cardinality_exact(N), stack_allocated
type RustSlice<T>    = FreeMonoid<T>
  where contiguous, borrowed, unsized
type RustVecDeque<T> = FreeMonoid<T>
  where double_ended_queue, heap_allocated
```

**Coercion walk — minimum-satisfier through multiple axes:**

1. User's `List<Int>` inhabits `FreeMonoid<Int>` with cardinality
   unbounded.
2. Narrow by cardinality: `RustArray<T, N>` requires
   `cardinality_exact(N)` — eliminated.
3. Narrow by ownership (default: owned): `RustSlice<T>` is
   borrowed — eliminated absent borrowed-intent signal.
4. Narrow by access pattern (default: sequential): `RustVecDeque<T>`
   is double-ended — not minimum-satisfying; eliminated.
5. Match: `RustVec<i64>` (with `Int` → `i64` via nested coercion).
6. Emit: `Vec<i64>`.

If user declares `FixedArray<Int, 8> = Cardinality<Int, Exact(8)>`,
the same search lands on `RustArray<i64, 8>` → emit `[i64; 8]`.
The choice is derivable from the user's declared cardinality, not
declared.

### 5. `Option<T>`

**User side** (`[proposed]` alias shape — no live `type Option<T>`
alias exists in `dsl/std/` today; optional-of-one cardinality
currently lives in the substrate / connective layer. The alias
form shown here is what the thesis-facing surface would look like
once the cardinality-substrate work lands, tracked at
`ROADMAP.md:305` + DB-11 at `:231`):

```dag
// [proposed] — target alias form; not a live declaration
type Option<T> = Cardinality<T, AtMost(1)>
```

**Rust side** (`[proposed]`):

```dag
type RustOption<T> = Cardinality<T, AtMost(1)>
  where variants(None, Some(T))
```

**Coercion:** 1:1 match. Emit `Option<T>`. Like `Bool` — shape
alignment, no search required.

## Work estimate

`[proposed]` — partitioning by layer:

| Layer | Rust | Python | Go | Total |
|---|---|---|---|---|
| 1. Language-spec primitives | ~40–60 decls (ints, floats, bool, char, str, String, Vec, Array, Slice, HashMap, HashSet, Option, Result, tuple, ...) | ~25–35 decls (int, float, bool, str, bytes, list, tuple, dict, set, None, ...) | ~35–45 decls (intN, uintN, float32/64, bool, string, []T, map[K]V, interface{}, struct, pointer, ...) | ~120–150 primitive declarations |
| 2. Compiler-contract | ~10 decls | ~5 decls | ~5 decls | ~20–30 decls |
| 3. Platform-contract | ~10 decls | ~5 decls | ~10 decls | ~25 decls, maintenance per release |
| 4. Coercion engine | Substrate work — inhabitance-search + minimum-satisfier + tie-breaking + diagnostics | | | **1–2 weeks** first-cut |
| 5. Coercion-routing tests | One-time test-suite build | | | **~1 week** |
| 6. Dissolution | Verify derived-structural parity with declared-table outputs as a precondition; Track 13 closure then deletes the table + `carrier: String` field in one step. No deprecation period; no shadow-running authority | | | **~2 weeks** for parity verification; dissolution itself is a single Track 13 PR |

**Aggregate:** roughly 2–3 months if one person owns it
end-to-end; cleanly parallelizable across targets at layers 1–3.
Bigger than an R1 lane; roughly the size of `T-LaneE`
(`ROADMAP.md:50`). Natural fit for a milestone after R1 ships,
once the substrate is stable and the current `TypeRealization`
table has taught us which carriers are hardest to replace (likely
`String`, given the UTF-8 encoding dimension).

## Non-preclusion: what to preserve during R1

`[proposed]` — three discipline items during R1 work so the
Track 13 dissolution stays reachable:

1. **Keep `TypeRealization.carrier` as a settable field, but treat
   it as cache, not authority, in any new code.** Schema unchanged;
   semantics shift when layer 6 (Track 13 dissolution) fires.
2. **Don't widen `carrier`'s placeholder grammar.** The current
   emitter already renders carriers as named-placeholder templates
   via `render_named_template` at
   `src/v3/compiler/src/emit/rust_target.rs:1461` (15+ call sites
   on HEAD) — e.g., `Vec<{element}>`, `HashMap<{key}, {value}>`.
   That existing contract is fine and compatible with the
   structural-coercion dissolution: a computed carrier can render
   the same template string the declared carrier did today. The
   discipline item is narrower: do not add *new* placeholder
   semantics (new placeholder names, new rendering rules) that
   the derivation pipeline would then have to reproduce. Keep the
   current placeholder set frozen during R1.
3. **Treat new target spec entries as additions, not
   reformulations, during R1.** Existing declarations stay; new
   ones are added structurally alongside only after layer 1
   begins. No flag day.

None of these require R1 to change course. They're discipline
about what *not* to build into the current path.

## Open design questions

`[proposed]` — decisions that need to be resolved when this
promotes to committed work:

- **Tie-breaking — fail-closed, not silent pick.** When two or more
  candidates satisfy identically on every declared dimension
  (same algebra, same carrier width, same qualifiers), the search
  surfaces a structured diagnostic naming the candidates and the
  constraint that would disambiguate — it does *not* silently
  select one via narrow-wins-heuristics. Silent pick is exactly
  the mystery-behavior class the thesis aims to eliminate. User
  disambiguates by either (a) adding a qualifier to the user type
  (`where native_op` signals intent) or (b) declaring a
  preference rule on the target side (`data rust_prefer_native_i_over_emulated: TargetPreferenceRule`
  as a single-authority override that itself is structural, not a
  compiler-internal heuristic). Note `RustI64` vs `RustI128` for
  `OrderedRing<Word64>` is *not* a genuine tie — `RustI64`'s
  carrier strictly-narrower satisfies the bound more tightly;
  `RustI128` is a wider-than-required supertype. True ties are
  rarer than they look, and should remain so by design.
- **Escape hatches for target-specific intent.** Sometimes the
  user wants a specific representation (e.g., boxed vs unboxed,
  stack vs heap). How is that signal expressed on the user side
  without leaking target specificity? Proposal: `where`-clause
  qualifiers on user types that match the target-side
  qualifiers.
- **Cross-type coercion paths.** UTF-8 encoding is one example
  (`Char → List<Byte>`); float-widening is another (`f32 → f64`).
  How are these paths declared? Proposal: `data X_to_Y:
  Coercion = { ... }` on the target side, invocable by the search
  when the direct match fails.
- **Failure diagnostics.** When no target primitive satisfies a
  user type, what's the error shape? Proposal: structured
  diagnostic naming the user type, the algebra requirement, and
  the closest candidates + why each failed.
- **Interaction with DB-18 (user-defined parametric algebra
  attachment).** If DB-18 ships first, user-defined algebras can
  attach to user types. Target primitives would also need to
  declare which user-declared algebras they inhabit. Timing
  question: does this proposal depend on DB-18, or can it ship
  independently for built-in algebras?
- **Interaction with the cardinality-substrate work
  (`ROADMAP.md:305` + DB-11).** The `Option<T>` example in
  worked-examples §5 is `[proposed]` specifically because
  `type Option<T> = Cardinality<T, AtMost(1)>` is not a live
  alias; cardinality-of-one lives in the substrate / connective
  layer today. When the cardinality-substrate work lands,
  `Option<T>` (and `NonEmpty<T>`, `BoundedList<T, N..M>`) become
  user-facing aliases, at which point this proposal's coercion
  search needs to walk the cardinality dimension symmetrically
  on both sides. Timing question: does this proposal depend on
  the cardinality-substrate landing, or can it ship first for
  non-cardinality-bearing types?

## L4 verification under structural coercion

This section refines `single-emitter-design.md`'s claim that
"coercion cost = complexity" and `what-falls-out.md:33`'s
"coercion completeness = fail-closed inhabitant lookup" into what
L4 verification specifically looks like once the dissolution lands.
Parent authorities leave this deliberately underspecified; the
refinement below is additive detail for the promotion PR.

`[proposed]` — L4 verification in the current framing does three
jobs at once, which structural coercion can separate cleanly:
(A) **routing correctness** ("did we pick the right target
primitive?"), (B) **structural-shape consistency** ("does the
chosen target primitive's declared algebra line up with the
user-side algebra the search matched on?"), and
(C) **algebra-satisfaction certification** ("does the chosen
target primitive's *actual runtime behavior* obey the declared
algebra's axioms?"). Structural coercion changes the shape of
each:

- **(A) Routing correctness** — structural coercion replaces
  declarative routing with search-based routing and is asserted
  by a new **routing-stability** property ("user's `Int64`
  always resolves to Rust's `i64`, never silently to `i128` or
  `Vec<Bit>`"). This is what the layer-5 coercion-routing tests
  cover. Distinct residual: a **legal search finding a target
  primitive the engineer didn't intend** (e.g., a post-hoc
  target declaration whose algebra legally satisfies the
  user-side constraint but whose semantics aren't what the
  engineer meant). The residual check is an end-to-end
  differential narrower than the current whole-algebra one — a
  sanity-check-on-choice, not a re-verification of the whole
  chain.
- **(B) Structural-shape consistency** — structural coercion
  discharges this half by construction. If the user-side and
  target-side both declare they inhabit `OrderedRing` and the
  search matched on that algebra, the shape-alignment doesn't
  need a separate runtime check; it's a structural fact of the
  search's success. This is the piece that genuinely becomes
  construction-certified.
- **(C) Algebra-satisfaction certification does NOT go away.**
  Target-side algebra assignment ("Rust's `i64` inhabits
  `OrderedRing`") is still **authored realization data**, not a
  tautology. Whether Rust's actual `i64` *behavior* at runtime
  really obeys `OrderedRing`'s axioms (associativity of `+`,
  existence of a two's-complement-wrapping `negate` that forms
  an additive inverse, etc.) is a theorem claim about the target
  runtime, and it still needs **L4 witness-based certification**
  per `docs/invariants/verifiability-invariant.md` and the
  "consistent by construction *+ verified by L4*" framing in
  [`two-groundings-static-validation-vs-efficient-realization.md`](two-groundings-static-validation-vs-efficient-realization.md)
  §"The two groundings must be consistent by construction +
  verified by L4." Structural coercion doesn't remove this
  obligation; it narrows its scope to the authored algebra
  declarations themselves (which targets genuinely satisfy the
  axioms they claim to), rather than the whole mapping chain.

Net effect: structural coercion discharges (B) by construction,
replaces (A) with routing-stability tests + a narrower
choice-correctness differential, and **preserves (C) as L4
witness-based certification of target-side algebra-inhabitance
claims**. Target-side algebra declarations remain **theorem claims
that need runtime-behavior evidence**, not self-certifying by
virtue of being structurally declared.

## When to promote

`[proposed]` — no fixed trigger. Natural windows to revisit:

- After R1 ships and T-LaneE is closed (substrate is stable).
- When a target-realization bug surfaces that the current
  table-driven design can't catch (e.g., a Rust edition change
  that alters `i64` semantics without updating the spec string).
- When a new target language is added and the per-target
  declaration overhead starts feeling like the thing worth
  dissolving.

Until promotion, this doc lives in `docs/thesis/` as a named
alternative design, not as committed work. Reviewers may reference
it when evaluating R1-era realization-layer decisions ("would this
change preclude the structural design?") without treating it as
authority.
