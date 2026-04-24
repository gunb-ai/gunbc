# Target Grounding Proposal — Structural Coercion at the Realization Boundary

> **Mode:** `PROPOSAL`

## Status of this proposal

**Parallel to R1. Non-blocking.** This doc proposes an architecture
change to the *realization-grounding* side of the two-groundings
framework per `docs/thesis/two-groundings-static-validation-vs-efficient-realization.md`.
R1 ships on the current (table-driven) realization design; this
proposal describes the successor. Promotion to committed work
requires a follow-up PR amending `THESIS.md` §"Correctness
dimensions" / §"What falls out" to name the structural-coercion
design as the target end-state.

**Why surfacing now.** The current realization design is
operationally correct and bootstrap-friendly, but the `carrier:
String` field on `TypeRealization` is a mapping/lookup table —
"checkpointing `.dag` models to known-good values." That's fair,
but not in the spirit of the thesis, which says information
becomes structure rather than declaration at every layer. Setting
the standard up front — even as a proposal committed in-tree —
makes the compiler's realization boundary match the rest of the
epistemic stack.

## Current state referenced

`[live]` — the current design is table-driven:

- `TypeRealization` declared at `src/v3/std/emit_model.dag:15` —
  carries `carrier: String` field.
- Per-target spec files declare data:
  `data rust_int: TypeRealization = { target: Int, carrier: "i64", ... }`
  (`src/v3/spec/rust.dag:103`); same shape in `python.dag`,
  `go.dag`.
- Rust emitter at `src/v3/compiler/src/emit/rust_target.rs` reads
  the table and pastes the `carrier` string.
- The two-groundings doc explicitly scopes realization grounding as
  "shallow — one hop to target primitive" and differentiates it
  from the deep static grounding.
- L4 verification — the consistency-check between the two
  groundings — is listed as partial in
  `docs/thesis/what-falls-out.md:33`.

## The proposal

`[proposed]` — model target primitives structurally in `.dag`, the
same way user types are modeled structurally, and let coercion at
the realization boundary be a **structural inhabitance-search**
rather than a table lookup.

**Shape of the change:**

- Target primitives get their own `.dag` models. For Rust, `i8`,
  `i16`, `i32`, `i64`, `i128`, `u*`, `f32`, `f64`, `bool`, `char`,
  `String`, `str`, `Vec<T>`, `[T; N]`, `&[T]`, `Option<T>`,
  `Result<T, E>` each get declared with their algebra + carrier +
  target-specific qualifiers (signedness, overflow behavior,
  ownership, memory layout, ...).
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

## Six-layer scope

`[proposed]` — the full implementation partitions into six layers:

| Layer | Content | Character |
|---|---|---|
| 1. Language-spec model | Each target language's primitives declared structurally in `.dag` — algebra, carrier width, overflow semantics, ownership, layout intent | Data entry sourced from language references |
| 2. Compiler-contract model | Target-compiler-specific behavior not pinned by the language spec — Rust edition defaults, `rustc` overflow-check flags, Python minor-version differences, Go generics era | Smaller data entry; updates when toolchains shift |
| 3. Platform-contract model | Target-runtime specifics: pointer width, endianness, wasm constraints, `usize` variability | Smallest layer; per-(target, platform) pair |
| 4. Coercion engine | Inhabitance-search over target primitives + minimum-satisfier selection + tie-breaking + diagnostics | Substrate work in the compiler |
| 5. Coercion-routing tests | For every user type, assert the resolved target primitive. For every target primitive, assert it's reachable via some user type (no orphans) | One-time test-suite build |
| 6. Migration | Current `TypeRealization` declarations become derived projections; `carrier: String` deprecates. Parallel run until outputs match | Incremental; no flag day |

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

**User side** (`[live]` via structural cardinality; `[target]` for
`NonEmpty` per `ROADMAP.md:305` substrate work):

```dag
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
| 6. Migration | Shadow-run until parity with current table; deprecate `carrier: String` | | | **~2 weeks** |

**Aggregate:** roughly 2–3 months if one person owns it
end-to-end; cleanly parallelizable across targets at layers 1–3.
Bigger than an R1 lane; roughly the size of `T-LaneE`
(`ROADMAP.md:50`). Natural fit for a milestone after R1 ships,
once the substrate is stable and the current `TypeRealization`
table has taught us which carriers are hardest to replace (likely
`String`, given the UTF-8 encoding dimension).

## Non-preclusion: what to preserve during R1

`[proposed]` — three discipline items during R1 work so this
migration stays possible:

1. **Keep `TypeRealization.carrier` as a settable field, but treat
   it as cache, not authority, in any new code.** Schema unchanged;
   semantics shift when layer 6 lands.
2. **Don't let new consumers read `carrier` as a format-string with
   placeholders.** Current consumers read it as a leaf name; if
   that contract holds, the shift from declared-string to
   computed-string is local to the emitter.
3. **Treat new target spec entries as additions, not
   reformulations, during R1.** Existing declarations stay; new
   ones are added structurally alongside only after layer 1
   begins. No flag day.

None of these require R1 to change course. They're discipline
about what *not* to build into the current path.

## Open design questions

`[proposed]` — decisions that need to be resolved when this
promotes to committed work:

- **Tie-breaking.** When two candidates satisfy equally (e.g.,
  both `RustI64` and `RustI128` satisfy `OrderedRing<Word64>` by
  supertyping), what's the rule? Proposal: narrowest carrier
  wins. Edge: what if one is `native_cpu_op` and another is
  `emulated_cpu_op`? Proposal: prefer native. Formalize.
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

## Relationship to the current design

`[proposed]` — the current design's L4 verification (differential
test of two groundings) is the existing answer to "how do we know
the target primitive satisfies the declared algebra?" Structural
coercion obviates most of L4: if the target primitive's algebra
is structurally declared and the coercion search matched on that
algebra, consistency is by construction, not by differential
test. L4 would simplify to "does the named target primitive's
emitted behavior match the declared quirks?" — a narrower check
than the current whole-algebra differential.

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
