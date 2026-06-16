# v2 Encapsulation Audit — the "touch-once contract"

**Date:** 2026-06-05
**Lane:** v2 encapsulation audit (read-only feed lane)
**Status:** Read-only. Every finding is cited to `file:line`; nothing is inferred. Regenerable from the cited sources.
**Anchors:** `INVARIANTS.md`, `THESIS.md`, `MODELING.md`, `src/v3/SELF_HOSTING.md`.

---

## 0. What this audits

For each foundational concept, a consumer should touch only its **interface**, with the
representation **hidden**, so a bug is fixable in **one** place. That is information hiding —
the foundation. The derived "free translation" homomorphism (rust → python emit as a fold over
a closed structure) is a *special case on top of* that hiding: it is only as sound as the hiding
beneath it. This dossier measures how close v2 is to the contract, concept by concept.

The worked exemplar of a violation (from the brief): v2 `SourceSpan { start, end: Int }`
(`dsl/std/types.dag:292`) is a bare untyped span with no char-vs-byte unit; the slow primitive
`v1_rt::substring` (char-offset, O(file_len) on non-ASCII) was reached by ~27 `.dag` callers
directly. It was fixed 7× in the tokenizer over 3 months but never generalized — because the
representation leaked to ~27 consumers. It has a *nominal* single authority (`source_text_at`),
so it **passes** "single-authority" yet **fails** "opacity." That distinction is the heart of
this audit. The v2 mirror (`Token { start, end: Int }`, `src/v2/std/lexing.dag:62-68`) replicates
the structural cause; the symptom is not yet triggered (v2 has no span→text consumer yet).

### Enforcement tiers (the question is "can you write the bug?")

For each concept, *try to write the wrong interaction*:

| Tier | Name | Test |
|---|---|---|
| 0 | Convention | discipline only; the v2 "7 fixes" prove discipline fails |
| 1 | Discriminated-but-visible | distinct type exists, but representation is visible to consumers |
| 2 | Detection | a predicate / runtime constructor catches it *after*; the wrong thing is still expressible |
| 3 | Impossibility | the wrong value is unrepresentable; it does not type-check |

**Impossibility = conjunction of four properties** (reported per concept):

1. **Opacity** — representation hidden (necessary, not sufficient).
2. **Correct-by-construction** — illegal states unrepresentable; no raw/unitless constructor.
3. **Closure / totality** — no escape hatch back to a bare primitive.
4. **Derived coercion** — relations derived from one declaration, not hand-restated.

---

## 1. HEADLINE — alias-vs-atom is the single best predictor of tier in v2

The substrate gets **derived-coercion** and **closed-coproduct** right almost everywhere.
It gets **opacity** right **only where the carrier is a substrate _atom_**, and **wrong wherever the
carrier is a transparent _alias_ to a structural default**:

| Carrier is an… | Examples | Opacity | Tier |
|---|---|---|---|
| opaque **atom** | `Symbol`, `Hash` (`node.dag:10-11`) | representation invisible to `.dag` | **3** |
| transparent **alias** | `Char = Nat` (`text.dag:13`), `String = FreeMonoid<Char>` (`text.dag:14`), offsets `= Int` (`lexing.dag:65-66`) | representation fully visible / arithmetic | **1** |

`Hash` is the proof that v2 **can** reach tier 3 today: a consumer cannot forge or destructure
a `Hash` from `.dag`. `String`/`Char`/offsets are the proof that the *default* modeling move
(alias a structural type) **defeats** the access pattern, exactly as the v2 exemplar predicts.

**The fix shape is therefore not "add `where`/`brand` refinement" — it is "stop aliasing; make
the carrier opaque."** See §2 for *how* opacity is reachable.

---

## 2. CAPABILITY VERDICT (highest priority) — v2 has THREE opacity mechanisms, not one

The pivotal question: *can v2 enforce correct-by-construction / hide a representation at all, and
by what mechanism?* There are three distinct mechanisms with three different verdicts. Conflating
them produces a wrong ceiling.

### 2.1 `where` / `brand` refinement — **BLOCKED for v2's own concepts**

- The **v3 bootstrap compiler that ingests v2 today** *does* statically parse and fail-closed
  enforce `where`-refinements on **scalar literals**: parser `KwWhere` →
  `parse_type_alias_where_tail` (`src/v3/compiler/src/parse_generated.rs:1096`); lowering
  synthesizes real `Bool` bodies for `gt_zero` / `unicode_scalar` / `range`
  (`src/v3/compiler/src/lower.rs:1125-1146`); rejection at
  `scalar_literal_must_reject_for_refinement` (`lower.rs:2157`). **Proof test:**
  `int_literal_range_narrowing_does_not_bypass_refinement_discharge`
  (`src/v3/compiler/tests/integration/int_literal_cardinality_test.rs:846`) — even
  `requires_positive(1)` against `type PositiveInt = Int where PositiveInt > 0` is **rejected**
  for missing discharge evidence. So an illegal (or even unproven-legal) refined **literal** does
  not compile = tier-3 **at the literal boundary**.
- **But this does not help v2's target concepts**, for three compounding reasons:
  1. It is **literal-only** (compile-time known values), not runtime-computed values.
  2. `brand` — the exact `CharOffset != ByteOffset` nominal-disjointness case — lowers to a
     **vacuously-true** reflexive equality and contributes **no** literal-side narrowing; the source
     itself flags this as tech debt: "rather than pretending reflexive Eq does that work"
     (`lower.rs:1130-1142`). The unit-disjointness target is the **weakest** predicate.
  3. v2's own substrate uses **zero** `where`-refinements (grep: 0 hits in `src/v2/`), and v2's
     self-hosted infer (`src/v2/compiler/04_infer.dag`) evaluates **no** `where`/`brand` predicate
     at all — it does grounding / constraint / algebra-inhabitance facts. So when v2 stops being
     compiled by v3, this capability is **not modeled** in v2 infer.

### 2.2 Representation-LESS atom (`Symbol`, `Hash`) — **a PRIVILEGE of the closed primitive registry**

- `type Symbol` / `type Hash` are bare opaque abstract types (`src/v2/std/node.dag:10-11`): no
  fields, no `.dag` constructor, no `.dag` destructure.
- Their representation + sole producers live in the **v2 runtime**: `pub type Hash = String` with
  `atom_identity_hash` / `hash_combine` as Rust primitives
  (`src/v1/stage0/src/v1_compiler_runtime_rust.rs:101`; mirrored `src/v1/runtime_rust.dag:312-315`).
  Binding is via a closed checkpoint registry, `coerce_primitive_type` / `lookup_checkpoint`
  (`src/v1/coercion.dag:105`), exercised for `"Symbol"` / `"Hash"`
  (`src/v1/stage0/src/compiler_tests.rs:670,694`).
- **Consequence:** a *new* bodyless `type X` not in that registry is **uninhabited** (no `.dag`
  constructor, no runtime producer). Getting a Hash-style representation-*less* atom for a new
  concept needs a runtime type + checkpoint entry = a **substrate/capability change**, not a pure
  `.dag` modeling choice.

### 2.3 `nominal_opaque` wrapper — **a `.dag`-EXPRESSIBLE modeling choice, compiler-enforced, UNUSED by v2**

This is the mechanism that changes the recommendation.

- `nominal_opaque` is a **type modifier in the surface language** the same compiler parses for v2:
  `TokenKind::Ident("nominal_opaque")` sets the opacity flag (`parse_generated.rs:894-906`).
- It is **enforced fail-closed in infer**: a field-projection of a `nominal_opaque` type from a
  **non-permitted accessor** yields `Diagnostic::NominalOpacityViolation` and leaves the projected
  port **`Unresolved`** (`src/v3/compiler/src/infer.rs:4392`; passing test
  `nominal_opaque_field_project_fails_closed_before_structural_descent`, `infer.rs:5457-5524`).
  The `NominalOpacity { permitted_accessors }` allowlist (`infer.rs:5495-5497`, populated example
  `infer.rs:6848`) is the **single-path interface** — the only way to read the representation.
- This needs **no runtime binding** and **no `where`/`brand`**. It wraps a representation (e.g.
  offset fields) but the compiler makes the representation **unreadable** from outside the permitted
  accessors. Information hiding here is **real and compiler-checked**, not convention.
- **v2 uses it zero times** (grep: 0 hits for `nominal_opaque` in `src/v2/`).

**Caveat (honest scope):** the enforced axis is **projection / destructure** (the *read* side).
I found no construction-side guard, so `nominal_opaque` alone does **not** prove
correct-by-construction (an external record-literal may still build the wrapper). For
source-position the dominant threat is *destructure-then-substring*, which projection-blocking kills
outright; correct-by-construction (no raw constructor) is a separate axis (combine with a smart
constructor / `refine` boundary, §3 Span).

### 2.4 Corrected ceiling

| Route | Reaches tier 3? | For v2 today |
|---|---|---|
| `where` / `brand` refinement (§2.1) | literal-only; brand vacuous | **blocked** for units/source-position |
| representation-less primitive atom (§2.2) | yes (Hash/Symbol) | **runtime privilege** (closed registry) |
| `nominal_opaque` wrapper (§2.3) | yes, on the **read/opacity** axis | **available `.dag` modeling choice, UNUSED** |

**Verdict:** tier-3 **opacity** for source-position / `Span` is achievable in v2 **today** via
`nominal_opaque` — the gap is **choice, not capability**. The earlier "tier-2 ceiling" reading was
an artifact of looking only at the refinement route (§2.1). Correct-by-construction (the second of
the four properties) remains the genuinely harder axis and is where a `where`/`brand` or
smart-constructor uplift still matters.

---

## 3. Per-concept ladder (ranked by materiality = foundational × deep)

Each concept: **current tier**, the **four-property read**, **cited evidence**, **near-term ceiling**,
**uplift to tier 3**, and the **MAP+LOCK** test that would pin it (tripwire = static reference check;
registry = single-authority registry; metamorphic = representation-swap, the real proof — two
implementations behind one interface, assert consumers identical by execution).

### 3.1 NODE — the homomorphism spine — **Tier 2** (well-formedness)

The universal IR: `Node { kind: NodeKind, children: List<Edge> }` (`node.dag:86-89`);
`NodeKind = TypeNode { connective } | ComputationNode { behavior }` (`node.dag:73-75`).

- **Opacity:** ✗ — `Node` is a public record; `kind` and `children` are freely visible/constructible.
- **Correct-by-construction:** ✗ — `Node { kind: TypeNode { connective: Arrow }, children: [Named{…}] }`
  **constructs** even though `Arrow` requires `PositionalEdges`. `well_formed` (`node.dag:259`) /
  `edges_conform` (`node.dag:220`) are **post-hoc `Bool` predicates**, not type constraints: you must
  *remember* to call them. This is the structural twin of the v2 exemplar — a nominal authority
  (`well_formed`) that does not hide the representation.
- **Closure/totality:** ◐ — `Connective` / `Behavior` / `NodeKind` are `🟢 TERMINAL` closed coproducts
  (`node.dag:13,21,73`) — good — but `children: List<Edge>` is unconstrained.
- **Derived coercion:** ✓✓ (**what v2 gets RIGHT**) — `content_hash` (`node.dag:1267`), `fold_node`
  (`node.dag:94`), and per-connective edge discipline `connective_edge_discipline` (`node.dag:106`)
  **all derive from the single `Node` declaration**. The rust→target homomorphism rides this. This is
  the template the leaky concepts should imitate.
- **Near-term ceiling:** tier 2. **Uplift to 3:** a smart `Node` constructor that enforces
  `edges_conform` at build time (returns `Outcome`), or a `nominal_opaque` `Node` whose `children` is
  reachable only through discipline-checked accessors.
- **MAP+LOCK:** *metamorphic* — two `Node` builders (one enforcing `edges_conform`, one not) behind a
  single smart constructor; assert every downstream consumer is identical by execution. Plus a
  *tripwire* that no call site builds a bare `Node { … }` bypassing the smart constructor.
  *registry*: `content_hash` / `fold_node` are already single-authority — keep them so.

### 3.2 SOURCE-POSITION / SPAN — the headline concept — **Tier 1** (capability-available, choice-blocked)

`Token { class, lexeme, file: Symbol, start: Int, end: Int }` (`lexing.dag:62-68`).

- **Opacity:** ✗ — `start` / `end` are bare `Int`, fully visible and arithmetic.
- **Correct-by-construction:** ✗ — no `CharOffset != ByteOffset` discrimination; any `Int` builds
  either end. A char-offset and a byte-offset are the **same type** — precisely the v2 defect.
- **Closure:** ✗ (bare primitive). **Derived coercion:** none.
- **Note — the byte-offset machinery is a red herring for this concept:** `node.dag:436-1228` carries
  extensive `byte_offset_*` code, but it is **`Int`-keyed cache-digest hashing**, not a typed
  `ByteOffset` carrier; `node.dag:449-451` explicitly states "Full P2 on unbounded Int offsets
  requires dissolve-on bounded `ByteOffset` carrier." The typed offset **does not exist**; it is
  deferred. So the unit-disjointness target is absent.
- **Near-term ceiling:** **tier 3 is reachable today** via §2.3. **Uplift:** model `Span` / `Source`
  as a `nominal_opaque` wrapper whose `permitted_accessors` are `{ span constructor, source.text(span) }`.
  Offsets become **non-destructurable** → char-vs-byte is **moot at the interface** (consumers never
  see offsets) → needs **no** `where`/`brand` and **no** new runtime primitive. Pair with a smart
  constructor (text only via `source.text(span)`) to also close the construction axis.
- **MAP+LOCK:** *metamorphic* (the real proof) — implement `Span` over `CharOffset` and over
  `ByteOffset` behind one `nominal_opaque` interface with `source.text(span)`; assert all consumers
  identical by execution. *tripwire*: no consumer projects `Token.start` / `Token.end` outside the
  source-text authority (this is the static check that would have prevented the v2 27-caller leak).

### 3.3 SYMBOL — **Tier 2-3 as a carrier** (strong opacity) — **but identity-uniqueness is fail-open**

- **Opacity:** ✓✓ — `type Symbol` (`node.dag:10`) is a bare opaque atom; consumers cannot tell if it
  is a `String` / interned id / `Int`. Best opacity template alongside `Hash`.
- **Correct-by-construction / closure:** ✓ — values minted via `data x: Symbol = x`; compared by `==`;
  representation lives in the v2 runtime, unreachable from `.dag` (§2.2).
- **CAVEAT — opacity ≠ uniqueness:** the symbol/variant **name-resolution** layer is multi-authority /
  fail-open: a bare-name constructor map, last-write-wins, ~37 colliding names (e.g. `Bits64` ×16)
  (cross-ref `v2_variant_name_resolution_multi_authority`; ctrl#1449; namespacing PR #4418). So the
  *type* is opaque (tier 3) while the *identity/disjointness contract over symbols* is tier 1
  (collisions possible).
- **MAP+LOCK:** *registry* — a single-authority symbol-minting registry that fails closed on two
  `data … : Symbol` of the same spelling across modules (catches the 37-collision multi-authority).
  *tripwire*: no parallel bare-name constructor maps.

### 3.4 HASH — **Tier 3** — the strongest "got-it-right" template

- **Opacity / closure:** ✓✓ — `type Hash` (`node.dag:11`); only `atom_identity_hash` /
  `hash_combine` / `content_hash` produce it; enforced 16-hex at the v2 primitive boundary
  (`node.dag:1266` note; runtime `is_hash_digest` / `expect_hash_digest`,
  `v1_compiler_runtime_rust.rs:101`). A consumer cannot forge or destructure a `Hash` from `.dag`.
- **Correct-by-construction:** ✓ — `content_hash` (`node.dag:1267`) is the single authority; no escape
  hatch.
- This concept is the existence proof for §1 / §2.2: opacity reaches tier 3 when the carrier is a
  substrate atom rather than a transparent alias.
- **MAP+LOCK:** *tripwire* — static check that no `.dag` destructures `Hash`; *metamorphic* — swap the
  digest function (FNV1a → other) behind the interface and assert consumers identical
  (existing stability test `src/v1/tests/src/b1_hash_primitive_test.rs:11`).

### 3.5 TYPE — **Tier 2** (uniform-with-computation; right shape, detection well-formedness)

- Types **are** `Node`s: `NodeKind = TypeNode { connective }` (`node.dag:73-74`). There is no separate
  "type" carrier — the substrate thesis (single `Node` authority; rust↔target homomorphism treats
  types and computations uniformly). **What v2 gets right.**
- Type-ness is discriminated via the closed `NodeKind` coproduct (✓); well-formedness inherits §3.1's
  tier-2 detection (same `connective_edges_conform`, `node.dag:230`).
- **MAP+LOCK:** same as §3.1 (Node).

### 3.6 IDENTIFIER / QUALIFIED NAME — **Tier 2** (discriminated-from-Symbol; fail-closed parse)

- `QualifiedName = QnEmpty | QnCons { head: Symbol, tail }` (`qualified_name.dag:27-29`) — a free
  monoid over `Symbol`, **distinct type from `Symbol`** (discriminated ✓). Currently hand-rolled,
  gated to become `FreeMonoid<Symbol>` once the v2 generic-alias limitation lifts (`qualified_name.dag:26`).
- **Correct-by-construction:** ◐ — the `Node → QualifiedName` boundary is **fail-closed**:
  `qualified_name_from_node` (`qualified_name.dag:188`) returns `QnFoldError` /
  `qualified_name_structure_invalid` on shape mismatch (tier-2 detection at the parse boundary). A
  `QualifiedName` value itself is freely constructible from arbitrary `Symbol`s.
- **MAP+LOCK:** *registry* — `qualified_name_from_node` is the sole `Node → QualifiedName` authority;
  *tripwire*: no parallel hand-rolled identifier parse.

### 3.7 CHAR / STRING / TEXT — **Tier 1** (the alias leak, in full)

- `Char = Nat` (`text.dag:13`) — bare alias; a `Char` is interchangeable with a count and admits
  arithmetic. No `unicode_scalar` discrimination (contrast `dsl/std/types.dag:193`
  `type Char = Int where unicode_scalar, brand "Char"`).
- `String = FreeMonoid<Char>` (`text.dag:14`) — **transparent alias** whose `List` representation
  **leaks**: `string_is_empty` / `string_head` / `string_tail` call `is_empty` / `fold_list_right` /
  `list_tail` **directly on the monoid** (`text.dag:36-43`). This is exactly the
  "`FreeMonoid<Char>` defeats the access pattern" risk — the structural sibling of the v2
  `substring`-leak (a consumer that sees the sequence representation can walk it the slow way).
- **Near-term ceiling:** tier 1. **Uplift:** a `nominal_opaque` `String` carrier (representation
  reachable only through text authority accessors) + `Char` as a branded/opaque scalar. Contrast
  `Hash` (§3.4): same substrate, opposite modeling move.
- **MAP+LOCK:** *metamorphic* — implement `String` over `FreeMonoid<Char>` and over an opaque rope
  behind one interface; assert consumers identical by execution. *tripwire*: no consumer calls list
  primitives on a `String` outside the `text.dag` authority.

### 3.8 FILE — **Tier 1** (Symbol-conflated)

- `Token.file: Symbol` (`lexing.dag:65`). A file identity is a bare `Symbol`, **indistinguishable**
  from a node tag / edge-name / any other symbol (atom identity). No discrimination between "file
  symbol" and other symbols.
- **Uplift:** a distinct `FileId` carrier (opaque or `nominal_opaque`), not a bare `Symbol`.
- **MAP+LOCK:** *registry* — a single file-identity authority returning `FileId`; *tripwire*: no
  consumer treats a `Symbol` as a file handle.

---

## 4. Right-ledger and ratio read

**What v2 gets RIGHT (templates to imitate):**

| Concept | What's right | Cite |
|---|---|---|
| `Hash` | tier-3 opaque atom; single producer; no `.dag` destructure | `node.dag:11,1267` |
| `Symbol` (as carrier) | opaque atom; representation hidden | `node.dag:10` |
| `Node` algebra | `content_hash` / `fold_node` / edge discipline all derive from one decl | `node.dag:94,106,1267` |
| `NodeKind`/`Connective`/`Behavior` | closed `🟢 TERMINAL` coproducts | `node.dag:13,21,73` |
| `QualifiedName` | discriminated from `Symbol`; fail-closed `Node→Qn` parse | `qualified_name.dag:27,188` |
| Type-as-Node | uniform with computation; single `Node` authority | `node.dag:73` |

**Ratio read.** v2 gets **derived-coercion** and **closed-coproduct** right *almost everywhere*; it
gets **opacity** right *only at substrate atoms* (`Hash`, `Symbol`) and *wrong at every transparent
alias* (`String`, `Char`, offsets, `file`). The decisive axis is **alias-vs-atom** (§1). Of the ten
seed concepts: tier-3 today = 2 (`Hash`, `Symbol`-as-carrier); tier-2 = 3 (`Node`, `Type`,
`QualifiedName`); tier-1 = 4 (source-position/`Span`, `Char`/`String`, `file`) — with the crucial
qualification that **source-position/`Span` can move to tier-3 opacity by a modeling choice
(`nominal_opaque`), not a capability change** (§2.3, §3.2).

---

## 5. Bottom line for the lane

1. v2 **can** make units / source-position **impossible-to-misread today** — via `nominal_opaque`
   (§2.3), not `where`/`brand`. The gap on the *opacity* axis is **choice, not capability**.
2. The single highest-leverage move is **stop aliasing structural defaults**: model `Span` / `Source`,
   `String`, and `FileId` as opaque carriers, imitating `Hash`. One opaque `Span` makes char-vs-byte
   moot at the interface and pre-empts the v2 `substring` 27-caller leak from ever re-forming in v2.
3. **Correct-by-construction** (no raw constructor) remains the genuinely harder axis: `nominal_opaque`
   closes the *read* side; closing the *construct* side still wants a smart constructor / `refine`
   boundary (or v2-infer `where`/`brand` enforcement, which today does not exist).

*All findings above are cited to current `main`-tracked sources at the listed `file:line`. This
document makes no source edits.*
