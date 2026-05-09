# T-Numeric-Construction — `Magnitude` 6Q substrate-introduction audit

**Lane:** R3 #6 (T-Numeric-Construction). **Authority:** [`docs/design-numeric-construction.md`](../design-numeric-construction.md). **Subject:** `Magnitude` — the terminal opaque carrier for unbounded counting, foot of the `Magnitude → Nat → Int → Rational → Real` construction chain.

This audit runs `feedback_substrate_principle_audit` on the `Magnitude` introduction before authoring the carrier slice. Encoding design call is resolved here; carrier slice (Slice 1: bare `type Magnitude`) follows in the same PR.

## Encoding design call

Three candidate encodings from the design doc:

| Encoding | Shape | Q3 risk | Q6 risk | Verdict |
|---|---|---|---|---|
| Bit-stream | `Magnitude { bits: List<Bit> }` | Duplicates `dsl/std/bit.dag` bit-stream carriers (`Byte { bits: List<Bit> }`, etc.) — same shape, same `List<Bit>` field | Two structurally different shapes for unbounded counting (opaque algebra-witness vs explicit bit-list); ruined by aliasing or specialization | **Reject** |
| Word-stream | `Magnitude { words: List<Word64> }` | Pre-bakes `Word64` chunking; introduces a parallel structural carrier alongside existing `Word64`/`Word128` records | Same Q6 risk as bit-stream; chunk size is a target-storage detail, not an algebraic fact | **Reject** |
| **Abstract counting** | `type Magnitude` (opaque atom; no fields, no algebra) | None — no fields, nothing to duplicate | None — exactly one structural shape (the atom); refinements `Nat<N>` extend rather than duplicate | **Accept** |

**Decision: abstract counting** (Director's PM-recommendation in design doc §1; aligned with audit). Carrier shape stays opaque; refinements `Nat<N>` / `Int<N>` give it concrete bit-width at grounding time. Matches `feedback_compositional_not_templating` (refinement is a child, not a parallel representation) and `feedback_naming_is_aliasing` (`Magnitude` is the namespace; refinements specialize it).

Precedent in std: `dsl/std/constructors.dag:68,72` already declares `type Product` and `type Coproduct` as bare opaque atoms. `Magnitude` follows the same shape.

## The 6 questions

### Q1 — Cardinality invariants
Does the type admit `[]` when invariant says ≥1, or singletons when ≥2?

**Answer: N/A — opaque atom with no fields.** No list-typed fields means no cardinality invariants to lie about. PASS by construction.

### Q2 — Index/handle types
Does a raw `Int` / `NodeId` encode something with a domain restriction?

**Answer: N/A — no fields.** No raw indices to type. PASS by construction.

### Q3 — Duplicated fact
Does Field A duplicate what's derivable from Field B?

**Answer: PASS for the introduction itself.** `Magnitude` has no fields and shares no structural surface with existing carriers. `dsl/std/bit.dag`'s `Word64`/`Word128` are fixed-width *storage* carriers (records over `List<Byte>`); `Magnitude` is the *unbounded counting* carrier (algebraic atom). They are different concepts at different layers — storage vs algebraic-axiom layer.

**Sequencing note (not a Q3 violation of this slice):** today's `dsl/std/integer.dag:34-44` declares `type Int8 = OrderedRing<Byte>` … `type UInt128 = Semiring<Word128>` (algebra parameterized over storage carrier). Once `Nat = Semiring<Magnitude>` and `Int = AbelianGroup<Nat>` land in subsequent slices, the existing `OrderedRing<Word*>` chain becomes the legacy authority and is migrated per design doc §6 (consumer migration). For this slice (bare `Magnitude`), there are zero consumers, so no parallel-authority window opens. The migration ordering in the design doc is what dissolves the legacy chain; this audit is for the foundational introduction, not the migration.

### Q4 — Coproduct compression
Does one variant compress N distinct causes that downstream needs to distinguish?

**Answer: N/A — not a sum type.** `Magnitude` is a single-variant atom, not a coproduct. PASS by construction.

(Adjacent: a future Grothendieck encoding via `Sign = Pos | Neg` would be a Q4 question for that slice, but the design doc §3 chose Option 3 — abstract `Int = AbelianGroup<Nat>` via existing `algebra.dag` — which sidesteps the encoding entirely. Out of scope here.)

### Q5 — Construction authority
Are multiple call sites independently constructing the same fact?

**Answer: PASS — single authority by construction.** `Magnitude` is declared in exactly one place (`dsl/std/magnitude.dag`); no consumers in this slice. Once `Nat = Semiring<Magnitude>` lands in a subsequent slice, `Nat` becomes the unique authority for unbounded ℕ. The design doc §6 sequences consumer migration so the existing `OrderedRing<Word64>` chain is migrated to consume the new authority rather than coexist as a second one.

### Q6 — Representation duality
Can the same fact be expressed in two structurally different shapes that comparison treats differently?

**Answer: PASS for abstract-counting choice.** The encoding design call resolved this question explicitly: bit-stream and word-stream encodings would each introduce a structural representation parallel to existing bit.dag carriers (Q6 violation by construction); abstract counting has exactly one structural shape — the opaque atom. Refinements `Nat<N>` extend the namespace (parameterized child types) rather than provide a competing representation.

## Verdict

**PASS — proceed with Slice 1 (bare `type Magnitude` opaque atom).**

- Carrier shape: `type Magnitude` in `dsl/std/magnitude.dag`. No fields, no algebra inhabitance, no value-body.
- Sized: **S** — single new file, single new type declaration, bootstrap regen, no consumers.
- Hard boundaries: no edits to `LitInt` / tokenizer / literal grammar; no parallel magnitude carrier introduced (the abstract-counting choice is the structural answer, not a sidecar); no migration of existing `Int = Int64` / `OrderedRing<Word64>` chain (design doc §6 sequences this for a later slice).

## Out of scope (deferred slices)

Per `docs/design-numeric-construction.md` §6 sequencing:
- Slice 2 — `Nat = Semiring<Magnitude>` (consumes Magnitude; verifies `Semiring<T>` algebra carries the structural facts needed at `dsl/std/algebra.dag`).
- Slice 3 — `Int = AbelianGroup<Nat>` (verifies `AbelianGroup<T>` at `algebra.dag:132`; design-call already resolved to Option 3 in design doc §3).
- Slice 4 — `Rational = Field<Int>` (verifies `Field<T>` at `algebra.dag:198`).
- Slice 5 — `ApproximateField<F>` introduction (largest substrate piece; design doc §5).
- Slice 6 — consumer migration cascade (3 direct + 5 inherited types per design doc §6).
- Refinement syntax (`Int<N>`, `where bits <= N`) — gated on T-V2-Retirement per design doc §"path (a) coordination."

## Cross-refs

- `docs/design-numeric-construction.md` §1 (`Magnitude` substrate-introduction; PM recommendation: abstract counting).
- `feedback_substrate_principle_audit` (the 6Q rule).
- `feedback_compositional_not_templating`, `feedback_naming_is_aliasing` (refinement-as-child rationale).
- `dsl/std/constructors.dag:68,72` (precedent for bare opaque-atom type declaration).
