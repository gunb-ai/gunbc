# Dissolution Lens — design spec

> **Status: design (rework-tracker PR #3240 task B1).** `modeling-discipline.md`
> Practice 10 carries the *rules*; this doc carries the *enforcement
> mechanism* — the checker / lens that makes the rules mechanical. Per
> Practice 10's own scoping: "the enforcement mechanism … is design work,
> specified in the planned `docs/design-dissolution-lens.md`."

A **lens** is a deterministic structural projection over the `.dag`
model — no LLM, no judgment. A finding either matches a lens's structural
signature or it does not. Run as a hard CI gate, a lens catches a finding
*every time*, instead of hoping a reviewer notices.

## 1. The core invariant

This doc operationalizes `modeling-discipline.md` Practice 10 — it does
not introduce a parallel rulebook. The invariants below are enforcement
scaffolding for Practice 10's rule, not a competing source of authority.

> **A0 — Every semantic fact must resolve to exactly one canonical
> structural witness path in the model.** A fact may be *derived* from
> a canonical carrier's shape (discriminant, catamorphism, traverse),
> *witnessed* by typed data (`data ... : Algebra<T>`, refinement clause,
> constructor restriction, alias-identity edge, canonical-carrier
> registry row), or *rejected as unknown* through diagnostic flow
> (`Outcome::Rejected`). Multiple structural artifacts pointing at one
> canonical authority (alias / re-export, retirement-ledger row, derived
> operation reading the same witness) are not multiplicities — they are
> the path. It may not be re-derived locally, asserted in prose / name /
> string form, duplicated as a competing authority, or guessed through
> a plausible default.

A0 is the umbrella. A1 is the operation-specific specialization that
originally seeded the lens suite:

> **A1 — Do not hand-roll a derived operation.** If a function's behavior
> is fixed entirely by the shape of a modeled type, it is re-deriving
> something the compiler already derives. *Inverse:* do not nominalize an
> operation as a type — an operation is a function, not a domain
> structure.

A0/A1 are proposed, pending operator ratification (rework-tracker task
A1). Once ratified into Practice 10, A0/A1 become citable hard rules;
this doc remains the enforcement mechanism.

## 2. Two tracks

The lens is not the cure — it is the net.

- **Track 1 — lenses (enforcement).** Catch hand-rolling at CI. This doc.
- **Track 2 — substrate-derivation (the cure).** Make hand-rolling
  *impossible*: the substrate derives an algebraic type's canonical
  operations (variant discriminants, catamorphism) from its declaration,
  so there is nothing left to hand-roll. The burn-down substrate nodes
  (the discriminant-predicate node; `fold_node` / catamorphism nodes)
  are Track 2.

The original seed findings exposed one substrate gap: the substrate
lets you *declare* an algebraic type but does not *derive its canonical
operations*, so workers hand-roll discriminants and catamorphisms.

The broader Layer-1 suite generalizes that lesson: when the substrate
lacks a canonical derived operation, witness table, authority map,
refinement edge, or diagnostic-flow carrier, workers encode the missing
fact locally — in prose, names, strings, duplicate homes, or plausible
defaults. Track 1 holds the line until Track 2 makes those witnesses
derivable or required.

## 3. Methodology — how a lens is derived

Every lens in this doc was derived, and every future lens is derived, the
same way:

> **slipped-by issue → core invariant violated → substrate root cause →
> mechanical lens.**

A lens is added from *real evidence* — a finding that reached merge-ready
despite being a defect (§8). The lens's signature is the smallest
structural pattern that catches that finding's class with zero false
positives on the clean shape.

## 4. Layer 0 — standard compiler hygiene

The floor every mainstream compiler (rustc, clang, tsc, the ML family)
enforces. **On by default in every profile** (§7) — Layer 0 is table
stakes, not a strictness dial. Shipping below this is a regression from a
normal compiler. Build-first: Layer 1 composes Layer-0 primitives.

**Group A — Unused / dead**
- **L0.1 Unused variable** — a bound name never read.
- **L0.2 Unused parameter** — a function/lambda parameter never read.
  *(Layer-1 reuse: the constant-algebra check, L1.1, is this applied to a
  fold algebra.)*
- **L0.3 Unused import.**
- **L0.4 Unused declaration** — a `type` / `fn` / `data` constant
  declared but never referenced. *(Layer-1 reuse: the hollow-type lens,
  L1.3, is this + "no inhabitance edge.")*
- **L0.5 Unused field** — a struct field never read.
- **L0.6 Unreachable code** — statements/arms after a diverging point.
- **L0.7 Dead / constant branch** — an `if`/`match` arm a static check
  proves is never reached.

**Group B — Binding integrity**
- **L0.8 Unbound name** — use of an undefined identifier.
- **L0.9 Use-before-definition** — a binding read before assignment.
- **L0.10 Duplicate definition** — a name declared twice in one scope.
- **L0.11 Shadowing** — a binding silently shadowing an outer one.

**Group C — Match / exhaustiveness** *(gunbc has coproducts + `match` —
directly load-bearing)*
- **L0.12 Non-exhaustive match** — a `match` missing a coproduct variant.
  *(Relates to Practice 1, fail-closed: an unhandled variant is a silent
  gap.)*
- **L0.13 Redundant / unreachable arm** — an arm subsumed by an earlier
  one.

**Group D — Result / fact discipline**
- **L0.14 Ignored result** — a produced value, especially a
  diagnostic-carrying `Outcome`, silently dropped. *(This is Practice 3,
  "facts flow forward," made mechanical.)*

**Group E — Control flow**
- **L0.15 Missing return** — a value-typed function with a path that
  yields no value.

Layer 0 is not all new: L0.12 and L0.14 already have gunbc Practice
analogs (Practices 1, 3) — part of Layer 0 is making an existing Practice
a mechanical lens.

## 5. Layer 1 — dissolution lenses

The gunbc-specific modeling-honesty checks. Each: the finding it kills,
its `.dag` signature, decidability, and the escape valve (the legitimate
non-finding).

### 5.0 Three levels — invariant, theme, lens

The design has three explicit levels. Each plays a different role and
they are not interchangeable:

| level | example | role |
|---|---|---|
| **Invariant** | A0 / A1 (§1) | *the reason* a finding is wrong; citable hard rule |
| **Theme** | derive / witness / canonical-home / fail-closed | *the recurring pattern* a finding belongs to; explanatory tag |
| **Lens** | L1.x (this section) | *the mechanical detector* — structural signature, decidable, with test corpus and escape valve |

> Themes are explanatory tags only. They do not define CI gates,
> test-corpus boundaries, or implementation passes. The mechanically
> enforced unit remains the L1.x lens signature. Two lenses that share
> a theme do not share machinery — they are distinct detectors with
> distinct signatures, decidability arguments, and escape valves, and
> they remain distinct precisely because the §3 methodology says each
> lens's signature must be the smallest structural pattern that catches
> its finding's class with zero false positives.

**Lens → theme catalog:**

| lens | theme(s) |
|---|---|
| L1.1 Discriminant-predicate | derive |
| L1.2 Degenerate-type | witness |
| L1.3 Hollow-type | witness |
| L1.4 Carrier-clone | canonical-home / witness |
| L1.5 Catamorphism | derive |
| L1.6 *(merged into L1.10 — Textual-bypass)* | — |
| L1.7 Off-substrate-fact | witness |
| L1.8 Wrong-home | canonical-home |
| L1.9 Vacuous-arm | fail-closed / witness |
| L1.10 Textual-bypass *(lens family — see §5.0 exception)*: L1.10.a `TemplateHole`, L1.10.b `CanonicalCarrier` | witness / canonical-home |
| L1.11 Plausible-fallback | fail-closed |
| L1.12 Parallel-authority | canonical-home |

The only mechanical merge in this layout is **L1.6 → L1.10** — the
prior doc already stated that L1.10 generalizes L1.6, so the two were
sibling labels for one consolidated signature space.

### L1.1 Discriminant-predicate lens — kills *predicate dissolution*

- *Signature:* a `Bool`-returning `fn` over a coproduct whose body is a
  single `match` with **literal `true`/`false` arms** per variant.
  **Extended (the #3249 lesson):** also a `fold`/`cata` call whose algebra
  is **constant in its recursive/accumulator argument** — a discriminant
  laundered through a fold. (The constant-algebra tell is L0.2,
  unused-parameter, applied to a fold algebra.)
- *Decidable:* fully — name + param type + body shape.
- *Verdict:* hard error on `std/` / substrate / reusable-helper files.
- *Escape:* a `match` whose arms do *computed* work (not literal
  `true`/`false`) is genuinely distinct → clean 🟢.
- *Kills:* `nat_is_zero` (#3255), `free_monoid_non_empty` (#3250),
  `free_monoid_is_empty`-via-fold (#3249), the `float_body_is_nan` /
  `edge_is_named` saturation class.

**Concrete match — the basic shape (`nat_is_zero` (#3255)):**
```dag
type Nat = Zero | Succ { prev: Nat }

fn nat_is_zero(n: Nat) -> Bool {
  match n {
    Zero               => true   // ← literal Bool per variant
    Succ { prev: _ }   => false
  }
}
```
The function's behavior is entirely determined by `n`'s top constructor.
The compiler already knows the variant from the parsed `match` — the
function is re-deriving the discriminant the substrate could expose for
free.

**Concrete match — the laundered shape (`free_monoid_is_empty` (#3249)):**
```dag
fn free_monoid_is_empty(m: FreeMonoid<T>) -> Bool {
  free_monoid_fold(
    m:    m,
    init: true,
    step: fn(_acc: Bool, _elem: T) -> Bool { false }   // ← acc never read
  )
}
```
A fold whose algebra ignores its recursive/accumulator argument is not
a catamorphism — it's a discriminant in disguise. The constant-algebra
tell is L0.2 (unused-parameter) applied to the fold's `step`. Without
this extension L1.1 would miss it.

**Clean shape:** delete the function; consume the substrate-derived
discriminant directly. The `match n { Zero => ..., Succ => ... }` form
already gives the caller what they need — `nat_is_zero` was an extra
layer they didn't need to write.

### L1.2 Degenerate-type lens — kills *nominalization*

- *Signature:* (a) a struct whose **every field is function-typed** with
  **no `data` instance**; (b) **N near-identical single-field structs**.
- *Decidable:* yes — field types, cross-ref for instances, similarity
  scan.
- *Verdict:* hard error on the wrapper shape.
- *Escape:* a genuine algebraic structure — non-function data fields,
  multiple `data` inhabitants (`Monoid`) — does not match.
- *Kills:* #3256's 26 `type ListMap { apply: fn }` wrappers; the
  `{ spelling: String }` ×N degenerate-wrapper class.

**Concrete match — (a) struct-of-functions (`ListMap` wrappers (#3256)):**
```dag
type ListMap<A, B> {
  apply: fn(List<A>) -> List<B>      // ← every field is function-typed
}
// no `data foo: ListMap<...> = ...`  ← no instance gives it algebraic content
```
`ListMap<A, B>` *is* `fn(List<A>) -> List<B>` wearing a name. The wrapper
adds no structure: there's no second field, no constructor laws, no
multiple inhabitants. It's an operation nominalized as a type.

**Concrete match — (b) N near-identical single-field structs:**
```dag
type Keyword    { spelling: String }
type Identifier { spelling: String }
type Symbol     { spelling: String }
// three "types" with structurally identical shape, distinguished only
// by the name — the closed set is doing coproduct's job
```

**Clean shape:**
```dag
// (a) — let the operation be a function, not a type
fn list_map<A, B>(xs: List<A>, f: fn(A) -> B) -> List<B> { ... }

// (b) — model the closed set as a coproduct, factor the shared field
type LexicalKind = Keyword | Identifier | Symbol
type LexicalToken { kind: LexicalKind, spelling: String }
```
Genuine algebraic structures (a `Monoid<T>` with `unit` + `combine` and
multiple `data` inhabitants like `additive_monoid_int`, `string_monoid`)
do not match — they have non-function content and real multiplicity.

### L1.3 Hollow-type lens — kills *hollow declarations* (Practice 8's hollow-alias finding, at the type level)

- *Signature:* a declared type **nothing inhabits** — no `data` instance,
  no `fn` returning it, no alias-identity to a substrate type. (This is
  L0.4, unused-declaration, + "no inhabitance edge.")
- *Decidable:* yes — cross-reference whether anything constructs it.
- *Verdict:* hard error.
- *Escape:* a type that *is* constructed, or aliases a substrate carrier,
  passes.

**Concrete match — declared, never inhabited:**
```dag
type ParseError {
  message: String
  span:    SourceSpan
}
// no `fn ... -> ParseError`, no `data ... : ParseError = ...`,
// no `type T = ParseError` alias, no record field of type ParseError.
// The type is a name with no edges into the rest of the model.
```
The author intended `ParseError` to mean something, but no code path
produces a value of it. From the substrate's perspective the type is
inert — it asserts an intention without committing structurally.

**Clean shape:** either delete the declaration (if it was speculative)
or make at least one inhabitance edge real:
```dag
fn parse(s: String) -> Outcome<Ast> {
  Rejected { diagnostic: ParseError { message: ..., span: ... } }   // ← inhabitance
}
```
Aliasing a substrate carrier also discharges the lens:
```dag
type ParseError = Diagnostic   // ← alias-identity is a structural edge
```

### L1.4 Carrier-clone lens — kills *carrier dissolution*

- *Signature:* a locally-declared coproduct **structurally isomorphic to
  a `std/` carrier** (`Foo { value: T } | FooRejected { diagnostic }` ≅
  `Outcome<T>`).
- *Decidable:* yes — type-shape match against the `std/` carrier set.
- *Verdict:* hard error.
- *Escape:* a coproduct carrying a payload the std carrier genuinely
  cannot express passes.

**Concrete match — `Outcome<T>` clone (the F2 / `NormalizeChildrenResult` shape):**
```dag
// in src/v4/compiler/03_normalize.dag
type NormalizeChildrenResult
  = NormalizedChildren        { children: List<Edge> }   // ← Produced
  | NormalizeChildrenRejected { diagnostic: Diagnostic } // ← Rejected
```
Structurally identical to `Outcome<List<Edge>>` — same two-variant
shape, same payload kinds, just renamed. The local coproduct adds no
information the canonical `Outcome<T>` can't express.

**Clean shape:**
```dag
// in src/v4/std/diagnostic.dag (canonical)
type Outcome<T> = Produced { value: T } | Rejected { diagnostic: Diagnostic }

// in src/v4/compiler/03_normalize.dag
fn normalize_children(...) -> Outcome<List<Edge>> { ... }
```
A coproduct that genuinely *can't* be expressed by the std carrier
passes — e.g. a three-variant `Cached | Produced | Rejected` where
`Cached` carries information `Outcome<T>` doesn't model.

### L1.5 Catamorphism lens — kills *walker / traverse dissolution*

- *Signature:* a `fn` recursing over a structural type by `match`ing its
  variants + self-calling on the sub-structure; or a `fold` body that is a
  `match acc { Rejected => propagate ; Ok => continue }` short-circuit
  ladder.
- *Decidable:* on the clean shape — recursion mirrors the data shape.
- *Verdict:* hard error on the clean shape; **reviewer-confirm** on
  genuinely-irregular recursion (call graph ≠ data graph) → clean 🟢.
- *Kills:* `ci.dag`'s hand-rolled `List` combinators (#3213); the
  resolve/normalize walkers (#3225).

**Concrete match — clean recursion mirrors the data shape (`ci.dag` `member` (#3213)):**
```dag
// in src/v4/workflow/ci.dag
fn ci_member(s: Symbol, xs: List<Symbol>) -> Bool {
  match xs {
    Nil                       => false
    Cons { head: h, tail: t } =>
      match symbol_eq(a: s, b: h) {
        True  => true
        False => ci_member(s: s, xs: t)   // ← recurse on the sub-structure
      }
  }
}
```
The function recurses by matching `List`'s variants (`Nil` / `Cons`)
and self-calling on `tail`. That recursion *is* the catamorphism over
`List` — the substrate's `fold` would discharge it. Same shape for
`any`, `all`, `count_if`, `find`: each is a fold-with-a-different-algebra.

**Concrete match — short-circuit fold ladder (resolve/normalize walkers (#3225)):**
```dag
fn resolve_children(...) -> Outcome<List<Node>> {
  match resolve(head) {
    Rejected { diagnostic: d } => Rejected { diagnostic: d }   // ← propagate
    Produced { value: h }      => match resolve_children(tail) {
      Rejected { diagnostic: d } => Rejected { diagnostic: d } // ← propagate
      Produced { value: t }      => Produced { value: cons(h, t) }
    }
  }
}
```
The `match acc { Rejected => propagate ; Ok => continue }` ladder is
`Outcome`'s monadic traverse over `List` — also a substrate-derivable
shape.

**Clean shape:** consume the substrate-derived combinator instead.
```dag
fn ci_member(s: Symbol, xs: List<Symbol>) -> Bool =
  list_any(xs, fn(h) { symbol_eq(a: s, b: h) })

fn resolve_children(xs: List<Node>) -> Outcome<List<Node>> =
  traverse_outcome(xs, resolve)
```
Genuinely-irregular recursion — the call graph does *not* mirror the
data graph, e.g. a graph walker that revisits visited nodes via a
side-table — falls under *reviewer-confirm* rather than hard-error.

### L1.6 Deprecated alias — see L1.10.a `TemplateHole`

L1.6 is retained as a deprecated alias to keep prior test names,
slipped-by ledger references, and external citations traceable. The
original L1.6 "Emit/template lens" — catching string literals carrying
positional placeholders (`{0}`, `{1}`, …) used as emitters — is now
[L1.10.a `TemplateHole`](#l110-textual-bypass-lens--kills-typed-model-bypass-via-string-proposed-merged-l16),
a sub-signature of the L1.10 Textual-bypass lens family. The
`TemplateHole` sub-signature preserves L1.6's registry-free
decidability for placeholder literals.

### L1.7 Off-substrate-fact lens — kills *prose-asserted facts* (proposed)

> **Status: proposed.** Derived from the 2026-05-18 ingest of findings F3
> (hand-rolled lattice merges with prose-only inhabitance), F4 (fixed
> widths carried by identifier, not structure), F11 (`ResourceHandle`
> opacity claimed in a comment over a freely-constructible record).
> Generalizes the standing "machine-readable inhabitance is the bar"
> ruling.

- *Signature:* a `.dag` declaration whose header comment or identifier
  contains a fact-bearing token from a closed vocabulary
  (`inhabits <Algebra>`, `opaque`, `non-empty`, `Word<N>`, `Float<N>`,
  `bounded`, `non-forgeable`) **without a matching structural artifact**:
  - claimed algebra inhabitance → no `data ... : Algebra<T>` row in scope;
  - claimed cardinality / width → **no recursively-discharged refinement
    chain** on the carrier. A name-encoded width fact must be witnessed
    all the way down: `Word64` requires `bytes: List<Byte> where len(_) == 8`
    AND every `Byte` reachable from a `Word64` carries
    `bits: List<Bit> where len(_) == 8`. A single outer refinement that
    bottoms out at an unconstrained carrier (`List<Byte>` whose `Byte`
    has `List<Bit>` with no length clause) does not discharge `Word64`'s
    64-bit claim; an arbitrary-bit-count `Byte` still inhabits the
    "well-formed" `Word64`. Same recursion for `Float32`/`Float64`: the
    width fact must distinguish the two structurally (different
    exponent/significand width refinements), not by reusing one
    unconstrained `FloatBody`;
  - claimed opacity / non-forgeability → no constructor restriction
    (the type is a record whose fields are all freely constructible from
    user-reachable substrate values).
- *Decidable:* yes — the claim vocabulary is a closed set; the structural
  counterpart is locatable (data table, refinement clause, constructor
  visibility). Width-claim discharge is decidable by **recursive descent
  through the carrier**: walk every field whose type is a substrate
  collection, require a length refinement at each level until the
  recursion bottoms out at a fixed-cardinality leaf or a primitive bit.
- *Verdict:* hard error on `std/` and substrate files.
- *Escape:* prose without fact-bearing tokens (rationale, anchors,
  examples) passes. A claim *with* the structural counterpart present
  passes.
- *Kills:* the lattice-without-witness shape, the width-in-the-name
  shape, the opacity-in-the-comment shape.

**Concrete match — F3 (`dsl/std/fermi.dag`):**
```dag
// FermiDepth inhabits Lattice<FermiDepth>.   ← claim
fn fermi_meet(lhs: FermiDepth, rhs: FermiDepth) -> FermiDepth { ... }
fn fermi_join(lhs: FermiDepth, rhs: FermiDepth) -> FermiDepth { ... }
// no `data fermi_lattice: Lattice<FermiDepth> = { meet: fermi_meet, ... }`
```

**Concrete match — F4 (`src/v4/std/machine.dag`):**
```dag
type Word64 { bytes: List<Byte> }   // `64` only in the name
type Byte   { bits:  List<Bit>  }   // any length representable
```

**Concrete match — F11 (`dsl/std/resources.dag`):**
```dag
// Opaque proof of resource acquisition. Only the compiler's acquire
// nodes can mint these -- user code cannot construct handles directly.
type ResourceHandle {                 // ← claim says opaque
  type: String                        // ← record, all fields constructible
  resource_id: String
  key: String
  cap: Secret
}
```

**Clean shape (the cure):** width refinements discharge recursively;
`Float32` and `Float64` are distinguished by their refinement clauses,
not by sharing one carrier.
```dag
// machine.dag — recursive refinement chain
type Bit
type Byte   { bits:  List<Bit>  where len(_) == 8 }
type Word64 { bytes: List<Byte> where len(_) == 8 }   // 8 × 8 = 64 ✓

// float.dag — structurally distinct float widths
type Float32 {
  sign:                  Bit
  biased_exponent:       List<Bit> where len(_) == 8
  trailing_significand:  List<Bit> where len(_) == 23
}
type Float64 {
  sign:                  Bit
  biased_exponent:       List<Bit> where len(_) == 11
  trailing_significand:  List<Bit> where len(_) == 52
}

// fermi.dag — algebra membership as a typed witness, not prose
data fermi_lattice: Lattice<FermiDepth> = { meet: fermi_meet, join: fermi_join }
```

### L1.8 Wrong-home lens — kills *orphan operations* (proposed)

> **Status: proposed.** Derived from finding F5 (`nat_compare` defined in
> `src/v4/std/float.dag` rather than `src/v4/std/nat.dag` or
> `src/v4/std/algebra.dag`). Mechanizes MODELING M9 (DFS the concept
> DAG).

- *Signature:* the lens derives the function's **primary concept** —
  the single type the function is "about" — and requires the function
  to live in that type's home file. The primary concept is selected
  structurally, in priority order:
  1. **Declared witness target.** If `f` appears as a field of a
     `data ... : Algebra<T> = { ... f: ... }` witness, the primary
     concept is `T` (the algebra's type parameter). `f`'s home is `T`'s
     home file, regardless of what its arguments look like.
  2. **Same-type closure.** If `f`'s argument types and return type are
     all the same type `T` (the `fn(T, T) -> T` / `fn(T) -> T` /
     `fn(T, T) -> Bool` closure shape), the primary concept is `T`.
  3. **Upstream argument convergence.** Otherwise, if every argument
     type and the return type are declared in a single file `X`, and
     `f`'s current file `Y` imports `X`, the primary concept is the
     type in `X` and the home is `X`.
  4. **No primary concept (cross-cutting).** If none of (1)–(3) selects
     a single owning type, the function is genuinely cross-cutting and
     the lens does not fire.
  The lens fires when (1), (2), or (3) selects a primary-concept home
  `X` and `f` lives in `Y ≠ X`. Symmetric rule for `data` declarations:
  primary concept comes from the declared algebra's type parameter
  first, then key/payload type.
- *Decidable:* yes — witness-field membership, argument/return type
  uniformity, and the import graph are all queryable from the parsed
  model. The four-rule selector is a closed structural cascade with no
  judgment calls.
- *Verdict:* hard error in `std/` and `extdeps/`.
- *Escape:* (4) genuinely cross-cutting functions pass without
  annotation. (3) admits an `// Anchor: cross-cutting { because: <token> }`
  override where the token is drawn from a closed vocabulary
  (`bridge`, `coercion`, `display`) — operator-confirm.

**Concrete match — F5 (`src/v4/std/float.dag:52-62`):**
```dag
// in float.dag, but every argument lives in nat.dag
fn nat_compare(a: Nat, b: Nat) -> Ordering {
  match a {
    Zero => match b { Zero => Equal, Succ { prev: _ } => Less }
    Succ { prev: ap } => match b {
      Zero => Greater
      Succ { prev: bp } => nat_compare(a: ap, b: bp)
    }
  }
}
```

**Clean shape:** move `nat_compare` into `src/v4/std/nat.dag` (or
`std/algebra.dag` as a `TotalOrder<Nat>` witness); `float.dag` imports it.

### L1.9 Vacuous-arm lens — kills *exhaustive-but-empty match* (proposed)

> **Status: proposed.** Derived from finding F1
> (`node_locally_well_formed` discharges every `ComputationNode { behavior: _ }`
> with `=> true`). Distinct from L0.12 (non-exhaustive match): the arm is
> present, but its body does no work. The closed set is *named* but not
> actually *checked*.

- *Signature (structural, no name-suffix vocabulary):* a single `match`
  on a coproduct whose arms are **asymmetric in body shape**:
  - **at least one arm's RHS is a trivial literal** of the function's
    return type (`true`, `false`, `Unit`, the matched input itself,
    `None` / `Empty`); and
  - **at least one sibling arm's RHS does non-trivial structural work**
    (calls another function, recurses, constructs a typed value with
    fields derived from the input).
  The finding is the asymmetry *within a single match* over a closed
  coproduct — not the function's name. A match where *every* arm is
  trivial is a different shape (likely L0.7 dead/constant branch or a
  genuinely-constant function) and is out of scope; a match where every
  arm does real work passes. The discipline-role of the function is
  inferred from the structural fact that *the author already wrote real
  work for some variants*, which is what makes the trivial siblings a
  vacuum rather than an honest constant.
- *Decidable:* yes — arm-body shape (literal-vs-call/recursion/ctor) is
  a structural property of the parsed match. No name inspection.
- *Verdict:* hard error in substrate files.
- *Escape:* the trivial arm carries an `// Anchor: trivially-true { because: <token> }`
  pinning the justification (`{ because: variant-has-no-children }`,
  `{ because: identity-on-Unit }`, etc.) drawn from a closed vocabulary
  — operator-confirm.

**Concrete match — F1 (`src/v4/std/node.dag:115-120`):**
```dag
fn node_locally_well_formed(n: Node) -> Bool {
  match n.kind {
    TypeNode { connective: c } =>
      edges_conform(children: n.children, d: connective_edge_discipline(c: c))
    ComputationNode { behavior: _ } => true   // ← arm exists, body vacuous
  }
}
```

**Clean shape:** introduce a sibling `behavior_edge_discipline(Behavior)`
in `std/node.dag` and dispatch both arms uniformly.

### L1.10 Textual-bypass lens family — kills *typed-model bypass via String* (proposed, merged L1.6)

> **Status: proposed.** Derived from findings F6
> (`CiCommand::ShellCommand { command: String }` while
> `extdeps/process.dag` already models a typed
> `Command { program, argv0, args, env }`) and F8 (string-template
> emitters such as `list_template: "Vec<{0}>"` in
> `dsl/std/languages.dag`). Absorbs the original L1.6 as a
> sub-signature. The two cases are different mechanical detectors but
> share one structural finding: **a string-valued artifact is carrying
> a typed fact that has, or should have, a model carrier.**

> **Exception to §5.0:** L1.10 is a *lens family*, not a single lens.
> Its mechanically enforced units are L1.10.a `TemplateHole` and L1.10.b
> `CanonicalCarrier`; they share a finding family and reporting label,
> but keep separate signatures, decidability arguments, escape valves,
> and test corpora — exactly the per-lens discipline §5.0 requires.

#### L1.10.a `TemplateHole` (registry-free)

- *Signature:* a field or value that is a **template string literal** —
  a string literal carrying positional placeholders (`{0}`, `{1}`, …)
  used as an emitter, where grammar-as-declarative-bidirectional-data
  belongs.
- *Decidable:* yes — a literal string with interpolation placeholders
  is a structural match (the keystone decidability table already
  classifies it "structural — a literal template-string field"). No
  registry required.
- *Verdict:* hard error on the literal-template shape.
- *Escape:* a plain string constant with no placeholders, or genuine
  string *data* that is not an emitter template, passes.

**Concrete match — F8 type-construction templates (`dsl/std/languages.dag`):**
```dag
type TypeMapping {
  string:            String
  int:               String
  list_template:     String   // ← positional placeholders inside
  optional_template: String
  map_template:      String
}

data rust_type_mapping: TypeMapping = {
  string:            "String",
  int:               "i64",
  list_template:     "Vec<{0}>",          // ← {0} is an emission hole
  optional_template: "Option<{0}>",
  map_template:      "HashMap<{0}, {1}>", // ← {0}, {1} are emission holes
}
```
Emission is happening — but it's a fill-in-the-hole string substitution
the lens can't structurally validate. The placeholder vocabulary
(`{0}`, `{1}`) is parallel to the model rather than part of it.

**Clean shape:** grammar-as-declarative-bidirectional-data — the target
type's construction is modeled, and emit is a fold over the model:
```dag
type RustTypeRealization
  = RustGeneric { ctor: RustIdent, args: List<RustTypeRealization> }
  | RustAtom    { ident: RustIdent }

data rust_list_realization: TypeRealization<List> = fn(elem) {
  RustGeneric { ctor: "Vec", args: [elem] }
}
```
Plain string constants without placeholders (e.g. a fixed `"i64"`
atom) pass — they're data, not a templated emitter.

#### L1.10.b `CanonicalCarrier` (substrate-declared registry)

- *Signature (substrate-declared registry, no opt-in tag):* the lens
  reads a **canonical-carrier registry** authored as data in the
  substrate. A typed carrier declares its coverage — the set of field
  names it claims authority over — as a structural witness:
  ```dag
  // in src/v4/extdeps/process.dag
  data process_command_canonical: CanonicalCarrier<process.Command> = {
    supersedes_string_at_field_named: { command, shell_command, invocation }
  }
  ```
  The lens fires on **any** field of type `String` whose field name
  appears in any registered `CanonicalCarrier::supersedes_string_at_field_named`
  set, whenever that carrier's target type is in scope. The trigger is
  not author-controlled — the author cannot bypass by omitting a tag,
  because the registry-declared field-name set drives the gate
  unconditionally. The name set lives in substrate `data`, not in the
  lens body, so adding a new typed carrier is a registry edit, not a
  lens edit.
- *Decidable:* yes — `CanonicalCarrier` registry membership, the
  registry's `supersedes_string_at_field_named` set, the offending
  field's declared name, and the target carrier's in-scope status are
  all structural facts in the parsed model.
- *Verdict:* hard error.
- *Escape (structural, registered exemption):* a `String` field whose
  name *does not* appear in any in-scope registry entry passes
  naturally. For fields whose name happens to collide with a registry
  entry but whose value is legitimately raw (e.g. an opaque ID), the
  carrier author can add a structural exemption row to the same
  registry, e.g.
  `data raw_command_exempt: CanonicalCarrier.Exemption = { at_field: command, in_carrier: opaque_log_record, because: opaque-id }`,
  read as data, not as comment. Author-side opt-out via comment marker
  or omitted tag does not pass.

**Concrete match — F6 (`src/v4/workflow/ci.dag:23-28`):**
```dag
type CiCommand
  = LintCommand
  | TestCommand
  | IgnoredTestCommand { test_name: String }
  | BootstrapStageCompile { produces: Symbol }
  | ShellCommand { command: String }    // ← unannotated; field name = `command`
```
With `data process_command_canonical: CanonicalCarrier<process.Command> = { supersedes_string_at_field_named: { command, ... } }`
in `extdeps/process.dag`, the lens reads the registry, sees that any
`String` field named `command` has a typed canonical home in scope
(`process.Command`), and fires — independent of whether the author
opted in to any annotation.

**Clean shape:** consume the typed carrier directly.
```dag
import v4.extdeps.process as process
type CiCommand = ... | ShellCommand { command: process.Command }
```

### L1.11 Plausible-fallback lens — kills *fabricated-sibling fallthrough* (proposed)

> **Status: proposed.** Derived from finding F10
> (`derive_effect_shape`'s `None => CreateEffect` arm for DELETE/PUT/PATCH
> when no path key exists). Sibling rule to P3 (fail-closed): the
> function returns a typed enum and the missing-info case returns a
> *different valid constructor* of that enum — a plausible guess —
> instead of escalating through `Outcome::Rejected`.

- *Signature:* a `match` arm of shape `None => Ctor` / `Empty => Ctor` /
  `[] => Ctor` where `Ctor` is a constructor of the function's return
  type AND the function's return type is **not** `Outcome<_>` (does not
  carry a diagnostic variant).
- *Decidable:* yes — return-type shape + arm-RHS constructor membership +
  matched-sub-pattern is a "nothing-here" variant.
- *Verdict:* hard error. The fix is to lift the return type to
  `Outcome<T>` and return `Rejected { diagnostic: DerivationUnknown }`.
- *Escape:* the missing-info case has a *uniquely correct* answer (e.g.
  `or_default(opt: Option<Nat>, default: Nat)` style helpers where
  `None => default` is the function's definition) — operator-confirm.

**Concrete match — F10 (`dsl/std/effects.dag:268-282`):**
```dag
DELETE =>
  match last_path_param(template: path) {
    Some { value: p } => DeleteEffect { key_source: PathParam { param: p } }
    None              => CreateEffect          // ← fabricated sibling
  }
PUT =>
  match last_path_param(template: path) {
    Some { value: p } => UpsertEffect { key_source: PathParam { param: p } }
    None              => CreateEffect          // ← same pattern
  }
```

**Clean shape:**
```dag
fn derive_effect_shape(...) -> Outcome<EffectShape> {
  DELETE =>
    match last_path_param(template: path) {
      Some { value: p } => Produced { value: DeleteEffect { ... } }
      None              => Rejected { diagnostic: DerivationUnknown { ... } }
    }
}
```

### L1.12 Parallel-authority lens — kills *unmarked duplicate concept homes* (proposed)

> **Status: proposed.** Derived from finding F9 (`Bool`, `Char`, `Url`,
> machine words declared in both `dsl/std/` and `src/v4/std/` with no
> marker indicating which is canonical). The D2-resolver gap is
> related but **deliberately not collapsed into this lens** — see the
> "Cross-reference — D2-resolver" note below for why the dangling-import
> shape is a separate finding.

- *Signature:* a type name `T` introduced by **any `type T` declaration
  form** in two different `.dag` files. The form is irrelevant — sum /
  alias (`type T = A | B`), record (`type T { f1: ..., f2: ... }`),
  unit (`type T`), and generic (`type T<X> = ...`) all count as
  introductions of the name `T`. The lens fires on the *name* being
  introduced twice, not on a specific syntactic form. (Scope
  clarification — the *planned-but-absent home* case is a different
  finding shape: an unresolved-reference / fail-closed P3 violation,
  not a duplicate-authority one. The lens that catches it is L0.8
  unbound-name extended to dangling import paths — separately
  classified so root-cause precision is preserved. L1.12 is
  duplicate-declaration only, and it covers `Bool = True | False`,
  `Word64 { bytes: List<Byte> }`, `Url { scheme: ..., ... }`, and any
  other type-introduction form on equal footing.)
- *Decidable:* yes — name uniqueness across the corpus is queryable
  from the parsed model. No comment/prose inspection.
- *Verdict:* hard error.
- *Escape (structural — applies the same rule L1.7 enforces against
  itself):* the lens does **not** accept a comment marker as authority,
  because prose-as-authority is exactly the shape L1.7 kills. The only
  passing shapes are themselves structural:
  1. **Alias / re-export.** The non-canonical file does not redeclare
     `type T`; it `import`s the canonical declaration and exposes it via
     a `type T = <canonical-module>.T` alias-identity edge.
  2. **Structural retirement record.** A `data` row in a designated
     retirement ledger names the historical declaration and a
     dissolution trigger, e.g.
     `data bool_dsl_std_retired: HistoricalDeclaration = { type: dsl.std.types.Bool, dissolves_when: <trigger> }`.
     The lens reads the ledger as data, not as prose.
  3. **Deletion / migration.** The historical declaration is removed in
     the same change and consumers are repointed at the canonical home.
  Comment markers — including a `// Authority: canonical` header — do
  not satisfy the escape, by construction.

**Concrete match — F9 (`dsl/std/types.dag:173` and `src/v4/std/logic.dag:14`):**
```dag
// dsl/std/types.dag:163-173   (legacy-scanner anchor, no authority designator)
// v3 Path A (Lane 1e-2b): `Bool` still parses here for the legacy scanner ...
type Bool = True | False

// src/v4/std/logic.dag:13-14   (dissolution classification, no authority designator)
// 🟢 coproduct dissolution — DECISIONS.md classification ledger: Bool.
type Bool = True | False
```

Both declarations *are* annotated, but neither annotation discharges
the lens — comment markers never do, by construction (see the
*Escape* clause). The existing tags classify the finding shape
(dissolution status, scanner anchor); none of them is a structural
alias edge, retirement-ledger row, or deletion.

**Cross-reference — D2-resolver:** the D2-resolver gap is a mix of
shapes that **do not collapse into L1.12**:
- The *planned-but-absent* `extdeps/languages/resolver.dag` is an
  unresolved-reference / dangling-import finding (L0.8 extended), not a
  duplicate-declaration one. Reporting it as a duplicate-authority
  finding would name the wrong root cause.
- If, separately, `GroundingMap` were declared in two language files
  (e.g. both `rust.dag` and `python.dag` redeclared `type GroundingMap`),
  *that* would be an L1.12 match in its own right.

The fix sequence for D2-resolver is therefore: first resolve the
dangling-import finding by landing the canonical home; only then is
L1.12 the right lens for any residual duplicate declarations.

**Clean shape (structural, not prose):**
```dag
// src/v4/std/logic.dag  — canonical declaration
type Bool = True | False

// dsl/std/types.dag  — alias-identity edge, no redeclaration
import v4.std.logic as canonical_logic
type Bool = canonical_logic.Bool

// OR: a structural retirement record (read as data, not prose)
// some/retirement_ledger.dag
data bool_dsl_std_retired: HistoricalDeclaration = {
  type:            dsl.std.types.Bool,
  dissolves_when:  <substrate-trigger>,
}
```
For D2-resolver: land the planned `extdeps/languages/resolver.dag`
canonically and rewrite `rust.dag`'s `GroundingMap` declaration into an
import/alias of the resolver's authoritative shape.

## 6. The discriminant / catamorphism distinction

L1.1 and L1.5 enforce one algebraic fact worth stating directly: a
coproduct has **two distinct derived operations** that must never be
conflated.

- The **discriminant** — "which variant?" — inspects the top constructor.
  Non-recursive, O(1).
- The **catamorphism** — the structural fold. Consumes the whole
  structure, O(n).

`free_monoid_is_empty` is a discriminant; expressing it through
`free_monoid_fold` (a catamorphism) is a category error — O(n) for an
O(1) fact — and *camouflages* the predicate-dissolution finding by moving
the variant-discriminant out of a `match` into fold-algebra arguments,
where the un-extended L1.1 would not see it. Hence L1.1's constant-algebra
extension: **a fold whose algebra ignores its recursive argument is not a
catamorphism — it is a discriminant, and is the same finding.**

## 7. The selective-profile model

A lens is a derived projection — composable and scoped. "Strictness" is
not a global dial; it is a **profile** — a set of lenses assigned to a
scope.

| profile | scope | lenses |
|---|---|---|
| **substrate** (strictest) | `src/v4/std/`, `src/v4/compiler/` | Layer 0 + all of Layer 1, all hard gates |
| **target** | `src/v4/extdeps/`, target models | Layer 0 + Layer 1, but L1.5 reviewer-confirms (target modeling has more legitimately-irregular recursion) |
| **scaffold** | early-milestone code marked `// scaffold:` | Layer 0 only |

Rules:
- **Layer 0 is on in every profile**, scaffold included. "Lens-shaped"
  does not mean "optional"; selectivity governs Layer-1 strictness only.
- **A profile downgrade is never silent and never self-service.** A file
  cannot quietly dial its own lens set down — that turns "selectively
  applied" into "selectively ignored." The `// scaffold:` marker is
  permitted, but it is the *in-file record of a reviewer-approved,
  scope-level decision*, not a worker's self-applied dial: it is
  **ratchet-only** — it may not be added to a file to dodge a lens that
  currently fires. Strict-by-default; loosen only deliberately and
  visibly.
- **Turning Layer 1 off does not turn the *discipline* off.** A
  dissolution finding in scaffold code still carries its 🟡 disposition
  tag with a bound plan — the keystone's universal "every 🟡 binds a
  dissolution plan" mandate is profile-independent. The scaffold profile
  suppresses the *CI hard-error*, never the *modeling obligation*.
  Otherwise `// scaffold:` becomes lens-off + no-tag = invisible debt —
  exactly the escape hatch this section forbids.
- The compiler runs the **substrate** profile on its own
  `src/v4/**/*.dag`. The compiler does not exempt itself from the
  discipline it enforces — that is the example others follow.

## 8. The slipped-by ledger

A living record. Every finding that reaches merge-ready despite a defect
is logged here, root-caused by the §3 methodology — so the lens set grows
from real evidence, not speculation.

| date | PR | finding | core invariant violated | lens |
|---|---|---|---|---|
| 2026-05-18 | #3250 | `free_monoid_non_empty` hand-rolled discriminant | A coproduct's variant-discriminant is a derived operation | L1.1 |
| 2026-05-18 | #3255 | `nat_is_zero` hand-rolled discriminant | same | L1.1 |
| 2026-05-18 | #3256 | 26 combinators nominalized into single-field wrapper types | an operation is a function, not a type | L1.2 |
| 2026-05-18 | #3249 | `free_monoid_is_empty` laundered through a fold | discriminant ≠ catamorphism; do not conflate | L1.1 (extended) |
| 2026-05-18 | ingest | `node_locally_well_formed` discharges every `ComputationNode { behavior: _ } => true` (`src/v4/std/node.dag:115-120`) | exhaustive-in-shape, vacuous-in-content; closed-set named but not checked | L1.9 (proposed) |
| 2026-05-18 | ingest | `merge_evidence` / `encoding_meet` / `fermi_meet` hand-rolled while inhabitance claimed in a `// ` comment, no `data ... : Lattice<T>` row (`dsl/std/{termination,encoding,fermi}.dag`) | algebra inhabitance is a typed witness, not prose | L1.7 (proposed) |
| 2026-05-18 | ingest | `Word64 { bytes: List<Byte> }`, `Float32`/`Float64` share unconstrained `FloatBody` (`src/v4/std/{machine,float}.dag`) | cardinality / width is a refinement, not a name | L1.7 (proposed) |
| 2026-05-18 | ingest | `ResourceHandle` is a freely-constructible record under prose claiming opacity (`dsl/std/resources.dag:17-25`) | non-forgeability is a constructor restriction, not a comment | L1.7 (proposed) |
| 2026-05-18 | ingest | `nat_compare(Nat, Nat)` defined in `src/v4/std/float.dag:52-62` while `nat.dag` and `algebra.dag` exist | an operation lives in its argument-type's home (M9 / DFS the concept DAG) | L1.8 (proposed) |
| 2026-05-18 | ingest | `CiCommand::ShellCommand { command: String }` while `extdeps/process.dag` models a typed `Command` (`src/v4/workflow/ci.dag:23-28`) | String escape hatch for a domain that has a typed model in scope | L1.10.b `CanonicalCarrier` (proposed) |
| 2026-05-18 | ingest | `list_template: "Vec<{0}>"`, `optional_template: "Option<{0}>"`, `map_template: "HashMap<{0}, {1}>"` etc. in `dsl/std/languages.dag` + per-language emit tables | string literal with positional placeholders used as emitter; grammar-as-bidirectional-data belongs | L1.10.a `TemplateHole` (proposed; absorbs original L1.6) |
| 2026-05-18 | ingest | `derive_effect_shape` `DELETE/PUT/PATCH None => CreateEffect` (`dsl/std/effects.dag:268-282`) | missing info must escalate through `Outcome::Rejected`, not return a different valid sibling | L1.11 (proposed) |
| 2026-05-18 | ingest | `Bool`, `Char`, `Url`, machine words declared in both `dsl/std/` and `src/v4/std/` (`type T = ...` redeclared, no structural alias/retirement/migration) | one concept must have one home (structural alias, retirement-ledger row, or migration) | L1.12 (proposed) |
| 2026-05-18 | ingest | `extdeps/languages/resolver.dag` referenced via imports but file does not exist; provisional `GroundingMap` lives in `extdeps/languages/rust.dag` | unresolved reference / dangling import — fail-closed P3, distinct root cause from duplicate authority | L0.8-extended (planned-but-absent home; deliberately not L1.12) |

Pattern from the seed PR rows: the original four are burn-down
*substrate* PRs — the lane built to remove dissolution debt produced
it. Each was *mostly* correct with one dissolution defect; #3249's was
invisible to reviewers because the fold-laundering hid it. This is why
the lens suite (mechanical, every time) and the burn-down pre-gate
(catch at the source) both exist. The later `ingest` rows extend the
ledger to A0's broader territory (prose / name / string / canonical-home /
plausible-fallback findings); they are not all derived-operation
defects but share the same root-cause pattern — a missing structural
witness that workers backfill locally.

## 9. Build path — model-derived only

The lens is **compiler-integral**: a modeled `src/v4/lens/` projection
(joining `complexity.dag` et al.) that the compiler itself runs and
rejects on. There is **no interim hand-written-script form.** A
`scripts/check-*` script that text-scans `.dag` is itself a hand-rolled
`.dag`-walker — the exact anti-pattern this lens exists to remove. The
dissolution lens cannot be hand-rolled either; it is a `fold` over the
*parsed* model, a consumer of the substrate-first sequence.

Consequence: the lens is **gated on the v4 front-end** (CP-1) being able
to parse `.dag` into a model, plus the v4 lens stage that runs
projections over it. Until then there is no *mechanical* dissolution
enforcement — the interim net is the reviewer prompts and the burn-down
pre-gate (human/agent, reactive), not a script. The lens's timeline is
CP-1's timeline.

Build order once the machinery exists: Layer 0 first (Layer 1 composes
its primitives — L0.2 → L1.1, L0.4 → L1.3), then Layer 1.

## 10. Open — audit of current coverage

To be filled: an audit of which Layer-0 checks the v4 compiler enforces
today vs. the gap. The v4 compiler is early-stage (the pipeline is still
being modeled), so Layer 0 is expected to be a substantial current gap —
B1 should state that plainly once audited.
