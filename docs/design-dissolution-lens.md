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
despite being a defect (§8), or a checkable review / audit receipt that
names the same defect shape, its live substrate home, and its dissolve
trigger. The lens's signature is the smallest structural pattern that
catches that finding's class with zero false positives on the clean shape.

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
| L1.13 Skeleton-collapse | derive / canonical-home |

The only mechanical merge in this layout is **L1.6 → L1.10** — the
prior doc already stated that L1.10 generalizes L1.6, so the two were
sibling labels for one consolidated signature space.

### 5.1 Canonical L1.x acceptance-key names (rebase target for downstream consumers)

Downstream consumers that need to register acceptance / coverage rows
per Layer-1 lens (e.g. `src/v4/lens/coverage.dag`'s `coverage_defect_*`
rows) MUST use the canonical key names enumerated here. The lens suite
is the single authority; key sets in other files are projections of this
enumeration.

| Lens / sub-signature | Canonical acceptance-key name |
|---|---|
| L1.1 Discriminant-predicate | `coverage_defect_discriminant_predicate` |
| L1.2 Degenerate-type | `coverage_defect_degenerate_type` |
| L1.3 Hollow-type | `coverage_defect_hollow_type` |
| L1.4 Carrier-clone | `coverage_defect_carrier_clone` |
| L1.5 Catamorphism | `coverage_defect_catamorphism` |
| ~~L1.6 Emit/template~~ | **retired — see L1.10.a below; no `coverage_defect_emit_template` key** |
| L1.7 Off-substrate-fact | `coverage_defect_off_substrate_fact` |
| L1.8 Wrong-home | `coverage_defect_wrong_home` |
| L1.9 Vacuous-arm | `coverage_defect_vacuous_arm` |
| **L1.10.a** `TemplateHole` (sub-signature of Textual-bypass family) | `coverage_defect_template_hole` |
| **L1.10.b** `CanonicalCarrier` (sub-signature of Textual-bypass family) | `coverage_defect_canonical_carrier` |
| L1.11 Plausible-fallback | `coverage_defect_plausible_fallback` |
| L1.12 Parallel-authority | `coverage_defect_parallel_authority` |
| L1.13 Skeleton-collapse | `coverage_defect_skeleton_collapse` *(reserved-proposed — enforcement not active until skeleton extraction + classifier + clearing receipts land)* |

**Migration notes for existing downstream consumers:**
- Any consumer carrying `coverage_defect_emit_template` is **stale**;
  rename to `coverage_defect_template_hole` (the renamed
  sub-signature).
- Any consumer carrying `coverage_defect_string_escape_hatch` (the
  pre-merge name) is **stale**; replace with the two sub-signature
  rows `coverage_defect_template_hole` AND
  `coverage_defect_canonical_carrier`. L1.10 itself is no longer
  a single mechanical unit — it's a lens family per §5.0's exception.
- These names are stable as of this revision; further changes to the
  L1.x taxonomy will require a corresponding update to this enumeration
  and a coordinated downstream rebase.

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

- *Signature:* the lens derives the function's **primary (domain)
  concept** — the unique receiver type the function operates over —
  and requires the function to live in that type's home file.
  Observation codomains (Bool, Ordering, comparison results) are
  *results of* the receiver concept, not peer home signals, and are
  classified structurally via a substrate registry:
  ```dag
  // canonical observation carriers — substrate data, not a hardcoded list
  data canonical_observations: Set<CanonicalObservation> = {
    CanonicalObservation { type: Bool,     role: predicate-result   },
    CanonicalObservation { type: Ordering, role: comparison-result  },
    CanonicalObservation { type: Unit,     role: side-effect-result },
    ...
  }
  ```
  The primary concept is selected by this priority cascade, all
  structurally decidable from the parsed model:
  1. **Declared witness target.** If `f` appears as a field of a
     `data ... : Algebra<T> = { ... f: ... }` witness, the primary
     concept is `T` (the algebra's type parameter). `f`'s home is `T`'s
     home file, regardless of what its arguments or return look like.
  2. **Domain-typed function.** Strip observation codomains and look at
     the remaining argument/return types. If the **non-observation
     types** are all the same type `T` (the function operates over `T`
     and returns either `T` or an observation of `T`), the primary
     concept is `T`. `fn(Nat, Nat) -> Ordering` selects Nat; `fn(Nat) -> Nat`
     selects Nat; `fn(Nat, Nat) -> Bool` selects Nat.
  3. **Upstream argument convergence.** Otherwise, if every
     non-observation type in the signature is declared in a single
     file `X`, and `f`'s current file `Y` imports `X`, the primary
     concept is the type in `X` and the home is `X`.
  4. **No primary concept (cross-cutting).** If none of (1)–(3)
     selects a single owning type, the function is genuinely
     cross-cutting and the lens does not fire.
  The lens fires when (1), (2), or (3) selects a primary-concept home
  `X` and `f` lives in `Y ≠ X`. Symmetric rule for `data` declarations.
- *Decidable:* yes — `canonical_observations` registry membership,
  witness-field membership, non-observation-type uniformity, and the
  import graph are all structural facts in the parsed model. The
  four-rule selector is a closed structural cascade.
- *Verdict:* hard error in `std/` and `extdeps/`.
- *Escape (structural — no comment anchors):* (4) cross-cutting
  functions pass via rule (4) without an exemption. For rule
  (1)/(2)/(3) cases where the lens fires but the function legitimately
  belongs in its current file, the exemption is itself a substrate
  data row read by the lens:
  ```dag
  data foo_wrong_home_exemption: WrongHomeExemption = {
    function: foo_fn,
    because:  bridge,                  // closed vocabulary
                                       // (bridge / coercion / display)
  }
  ```
  Comment anchors do not satisfy the escape, by construction. The
  vocabulary token set (`bridge`, `coercion`, `display`) is itself a
  closed coproduct declared structurally — extending it is a data
  edit, not a doc edit.

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
- *Escape (structural — no comment anchors):* the exemption is itself
  a substrate data row read by the lens:
  ```dag
  data unit_is_unit_vacuous_arm_exemption: VacuousArmExemption = {
    function:   unit_is_unit,
    at_variant: Unit,
    because:    variant-has-no-children,   // closed vocabulary
                                           // (variant-has-no-children /
                                           //  identity-on-Unit /
                                           //  proven-unreachable)
  }
  ```
  Comment anchors do not satisfy the escape, by construction. The
  `because` vocabulary is itself a closed coproduct declared
  structurally — extending it is a data edit.

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

- *Signature (no return-type carve-out — RHS constructor structural check):*
  a `match` arm of shape `None => Ctor` / `Empty => Ctor` / `[] => Ctor`
  where `Ctor` is a constructor of the function's return type AND
  `Ctor` is **not** a structurally-registered fail-closed-diagnostic
  variant. Fail-closed-diagnostic variants are declared by the
  substrate:
  ```dag
  data outcome_rejected_variant: FailClosedDiagnostic = {
    type: Outcome,
    ctor: Rejected,
  }
  ```
  The lens fires on **both** the bare-return case and the
  Outcome-wrapped case:
  - `fn(...) -> EffectShape; None => CreateEffect` — return is not
    Outcome, `CreateEffect` is not registered → fires (F10).
  - `fn(...) -> Outcome<EffectShape>; None => Produced { value: ... }`
    — return is Outcome, `Produced` is not registered → fires
    (fabricated success — the prior carve-out's false negative).
  - `fn(...) -> Outcome<T>; None => Rejected { diagnostic: ... }` —
    `Rejected` IS registered as `FailClosedDiagnostic` → passes.
- *Decidable:* yes — return-type shape + arm-RHS constructor membership
  + matched-sub-pattern is a "nothing-here" variant + registry
  membership of the RHS constructor are all structural facts.
- *Verdict:* hard error. Fix depends on which case fired:
  - **Bare-return case** (`fn(...) -> T; None => Ctor`): lift the
    return type to `Outcome<T>` and return
    `Rejected { diagnostic: DerivationUnknown }` on the missing-info
    arm.
  - **Outcome-wrapped case** (`fn(...) -> Outcome<T>; None => Produced { value: ... }`):
    the return type is already `Outcome<_>` — replace the
    `Produced` constructor on the missing-info arm with
    `Rejected { diagnostic: DerivationUnknown }` (the registered
    fail-closed-diagnostic variant).
- *Escape (structural — no operator-confirm via prose):* for genuine
  total-by-design helpers (`or_default(opt, default)` etc.) where the
  `None` branch's RHS is the function's *definitional* result, the
  exemption is a substrate data row:
  ```dag
  data or_default_total: PlausibleFallbackExemption = {
    function:    or_default,
    arm_pattern: None,
    because:     total-by-design,    // closed vocabulary
                                     // (total-by-design /
                                     //  domain-restricted /
                                     //  proven-saturating)
  }
  ```
  Comment markers do not satisfy the escape, by construction. Same
  shape as L1.8 `WrongHomeExemption` and L1.9 `VacuousArmExemption` —
  the exemption is read as substrate data, not as prose.

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

- *Signature (concept-level — two triggers, union):* the lens fires
  on **parallel authority for a concept**, which is detectable in
  two mechanically-distinct ways. The lens has **two triggers**;
  EITHER triggers the resolution check. P2 and Practice 5 demand
  single-authority *at the concept level*, not at the lexical-name
  level — both triggers exist so the lens detects the violation in
  the shapes the substrate can mechanically see.
  - **Trigger A (lexical):** a cross-file lexical-name collision —
    two `type T` declarations in different files sharing the same
    simple name.
  - **Trigger B (concept-graph):** two `type T1` and `type T2`
    declarations in different files that are **co-members of a
    `CanonicalConcept` row**, regardless of whether their lexical
    names match. This catches the "two parallel homes for one
    concept under different names" shape, which Trigger A misses by
    construction.
  Once either trigger fires, the lens checks for one of **five
  mechanically-distinct outcomes** — three passing and two firing —
  and the decision table is closed (no escape outside this
  enumeration):
  Each outcome is defined by which **substrate-data resolution
  shape(s) are present** — they are independent passing conditions,
  NOT compound predicates. `CanonicalConcept` co-membership is one
  resolution shape; an alias-identity edge is another; a
  `HistoricalDeclaration` retirement row is another. Any single
  passing shape's presence is enough.
  1. **Alias-identity edge.** The non-canonical declaration is a
     structural alias (`import` + `type T = <canonical-module>.T`)
     of another declaration. The alias IS the resolution — it
     re-exports rather than re-declares — regardless of whether a
     `CanonicalConcept` row is also present. → **passes.**
  2. **Retirement-record.** A `HistoricalDeclaration` row names one
     of the two declarations as retired with a `dissolves_when`
     trigger:
     ```dag
     data bool_dsl_std_retired: HistoricalDeclaration = {
       type:           dsl.std.types.Bool,
       dissolves_when: <trigger>,
     }
     ```
     The retirement ledger is substrate data read by the lens — its
     presence resolves the lens regardless of whether a
     `CanonicalConcept` row also exists. → **passes.**
  3. **Distinct-concepts disambiguation.** A `ConceptDisambiguation`
     row names the two declarations as legitimately distinct
     concepts in different namespaces:
     ```dag
     data network_vs_normalize_result: ConceptDisambiguation = {
       names:   { network.Result, compiler.normalize.Result },
       because: distinct-domain-concepts,
     }
     ```
     → **passes.** (Note: applies to Trigger A only. Under Trigger B,
     a `ConceptDisambiguation` row that *contradicts* a present
     `CanonicalConcept` co-membership row is a registry-inconsistency
     finding — caught by L0-class duplicate-data-row checks, not
     L1.12.)
  4. **CanonicalConcept co-membership without alias or retirement.**
     A `CanonicalConcept` row claims the two declarations as
     co-members, AND neither outcome (1) alias nor outcome (2)
     retirement is present → **fires** (the duplicate-authority
     case; reachable via either trigger). The CanonicalConcept row
     by itself asserts the concept identity but does not resolve
     the parallel authority — outcome (1) or (2) is required to
     resolve, otherwise the substrate is asserting "these two
     declarations are the same concept" while keeping both as
     authoritative declarations, which is the violation.
  5. **Silence.** Triggered lexically (Trigger A), AND none of
     outcomes (1) / (2) / (3) / (4) apply (no alias, no
     retirement-record, no disambiguation, no CanonicalConcept
     row) → **fires as unresolved-duplicate.** The substrate must
     take a position. (Not reachable via Trigger B — Trigger B's
     premise is that a CanonicalConcept row IS present, so this
     case becomes outcome (4) or one of the passing outcomes
     instead.)
  Note: **deletion / migration** (removing the redeclaration in the
  same change so only one `type T` remains across the corpus) is
  not a sixth resolution — it removes the trigger condition (the
  lexical collision and/or the registry co-membership) entirely, so
  the lens never engages.
  **Decidability boundary (explicit):** the lens catches concept-level
  parallel authority when EITHER (a) the duplicate uses the same
  lexical name (Trigger A) OR (b) the substrate has registered the
  concept identity via `CanonicalConcept` (Trigger B). It does NOT
  catch the case where two homes use *different* lexical names AND
  no `CanonicalConcept` row registers them as the same concept —
  that case is a P2 violation but is mechanically undetectable from
  parsed substrate alone; closing it requires either operator
  judgment or an extension primitive (e.g., a structural similarity
  fold) that the current substrate does not provide. This is the
  honest decidability boundary, and per the §3 methodology a lens's
  signature must catch its finding class with zero false positives —
  Trigger B's CanonicalConcept-driven gate is the structural surface
  the lens can decidably enforce today.
  Unresolved-silence fails closed. Same lexical name in different
  concepts passes via (3); the F9 motivating case (Bool in two
  files with no rows anywhere) fires via (5) until one of (1),
  (2), or (3) lands.
- *Decidable:* yes — lexical-name uniqueness, `CanonicalConcept`
  registry membership, structural-alias edges,
  `HistoricalDeclaration` registry membership, and
  `ConceptDisambiguation` registry membership are all structural
  facts in the parsed model. No comment/prose inspection.
- *Verdict:* hard error.
- *Escape (structural — same rule L1.7 enforces against itself):*
  the passing shapes are outcomes (1), (2), and (3) in the
  decision table above — alias edge, retirement-ledger row, or
  ConceptDisambiguation row — all substrate data read by the
  lens. Comment markers — including a `// Authority: canonical`
  header — do not satisfy the escape, by construction. (Scope
  clarification — the *planned-but-absent home* case is a
  different finding shape: unresolved reference / fail-closed P3,
  caught by L0.8 extended to dangling import paths, deliberately
  not L1.12.)

**Concrete match — F9 (`dsl/std/types.dag:173` and `src/v4/std/logic.dag:14`):**
```dag
// dsl/std/types.dag:163-173
type Bool = True | False

// src/v4/std/logic.dag:13-14
type Bool = True | False

// (no CanonicalConcept row, no ConceptDisambiguation row anywhere)
```

This is the **unresolved-duplicate** case — the trigger fires (lexical
collision across two files) and silence in both registries means the
substrate has not taken a position. Outcome (5): the lens fires.

A `CanonicalConcept` row alone is not enough — it asserts co-membership
but without a structural alias edge from `dsl.std.types.Bool` to
`v4.std.logic.Bool` and no `HistoricalDeclaration` retirement row,
outcome (4) "same-concept-without-alias-or-retirement" still fires:
```dag
data bool_concept: CanonicalConcept = {
  canonical_home: v4.std.logic.Bool,
  members:        { dsl.std.types.Bool },
}
// dsl.std.types still has `type Bool = True | False` (a redeclaration,
// not an alias, not retired) → outcome (4) → fires.
```

To pass, the historical declaration must become an alias-identity edge
(or get retired via `HistoricalDeclaration` ledger, or get deleted in
the same change). The hypothetical `network.Result` vs
`compiler.normalize.Result` case passes via resolution (3) — a
`ConceptDisambiguation` row marking them as legitimately distinct.

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

#### L1.12.b Structural-similarity trigger — closes L1.12's decidability gap (the *nickname* case)

> **Status: proposed.** Closes the gap L1.12's decidability boundary
> already names: "two homes use *different* lexical names AND no
> `CanonicalConcept` row registers them as the same concept" — the
> *unregistered nickname*. The boundary text itself points at "an
> extension primitive (e.g., a structural similarity fold)" — this
> sub-signature IS that primitive, scoped narrowly so the lens stays
> mechanically decidable. Same 5-outcome resolution table as the
> parent.

- *Trigger C (structural similarity):* fires when a declaration's
  structural shape matches a known `std/` (or `core/`) carrier above
  a threshold defined by the layers below, AND no `CanonicalConcept`,
  alias-identity edge, or `HistoricalDeclaration` row connects them.
  Two scopes — both fire the same trigger:
  - **Type-scope.** A `type Foo = …` declaration in a non-`std/`
    file whose **variant set + field shape** matches an existing
    `std/` `type Bar = …`, modulo variant renames. Catches model
    nicknames (`data DominanceResult = Win | Lose | Tie` when
    `Witness<Carrier>` or an existing dominance carrier already
    exists) and refinements declared as parallel coproducts
    (`RegisterStateSpace` enumerating a refined subset of
    `StateSpace` instead of binding via refinement).
  - **Fn-scope.** A `fn helper(…) -> …` whose body is structurally
    equivalent to an existing `std/` fn after parameter-rename + body
    α-renaming. Catches helper nicknames (a hand-rolled `find_*`
    that's `find_witness` with refinement; a hand-rolled `lookup_*`
    that's `fold_right` over a chain).

- *Decidable:* yes — three mechanically-distinct layers, ordered by
  workhorse-first. The lens fires when **any** layer's structural
  fact crosses its threshold; the 5-outcome resolution table from
  L1.12 then resolves.
  - **(C1) Signature-shape match (workhorse).** For type-scope: the
    parsed model carries the carrier's **variant set + per-variant
    field shape**; two carriers match when there's a bijection
    between their variant sets preserving field arity + field-type
    coproduct identity (modulo coproduct-of-leaves identity per
    Practice 11). For fn-scope: the parsed model carries the fn's
    **signature shape** (input coproduct, output coproduct,
    parameter-arity, return-type identity); two signatures match
    when there's a parameter-bijection preserving type identity.
    Substrate-readable, no heuristic — fires when the signature is
    an exact structural match against a known `std/` carrier.
  - **(C2) Catamorphism-equivalence (stricter sub-rule for
    fold-shape fns).** A fn that performs structural recursion over
    `T` (one arm per `T`-variant, recursive calls only at
    `T`-variant positions) is, by the homomorphism heuristic,
    equivalent to `fold_T` with its algebra. If `std/T/fold_T` (or
    the `T` carrier's algebra carrier) is present in the registry,
    the un-derived hand-roll IS the nickname. This is Practice 10's
    homomorphism heuristic mechanized — same defect class, machine-
    readable trigger.
  - **(C3) Token-vocabulary overlap (triage, not a fire).** Names
    of a non-`std/` decl and a `std/` decl share above a vocabulary-
    overlap threshold (Jaccard ≥ 0.5 on the token-set extracted
    from identifier + variant/field names), as a heuristic surface
    for human cheap-glance. **Cheap-glance only — never fires the
    lens by itself; presented as advisory list alongside firings.**
    Same rule as L1.13's "advisory" tier being illegal: enforcement
    requires (C1) or (C2) substantiation.

  **Order is intentional.** (C1) fires on the structurally-identical
  case (the workhorse); (C2) fires on the un-derived-fold case (a
  separate defect class with its own clean resolution); (C3) is
  triage-only. The lens never fires on (C3) alone — operator-direct
  ruling on Practice 11 sub-rules + the no-fourth-tier rule.

- *Verdict:* hard error. Same 5-outcome resolution table as the
  parent L1.12 (alias-identity edge / retirement-record /
  ConceptDisambiguation / CanonicalConcept-without-resolution /
  silence-fails-closed). The substrate adds whichever resolution
  shape is honest for the case.

- *Escape (clean 🟢):*
  - **Refinement-not-parallel.** A `type RegisterStateSpace = StateSpace
    refined { variant ∈ {Reg, Const, Global, Local} } ∪ { ResParam { … } }`
    declaration (or its substrate equivalent) makes the relationship
    explicit; the parallel-coproduct nickname becomes a refinement
    edge. Equivalent to outcome (1) "alias-identity edge" extended
    to refinements.
  - **Practice-11-parameterized.** If the (C1) hit is "the same fn
    shape appears N times across files" (e.g., the `count_named_bind`
    quadruplet — identical body across 4 files), the parameterized
    move per Practice 11 is to define the fn once in a canonical home
    and import it elsewhere; the redeclarations become alias-identity
    edges (outcome (1)). Test-fixture redeclarations get the same
    treatment — copies are not exempted from L1.12 by being in a
    fixture file.
  - **Distinct domains.** A `ConceptDisambiguation` row marking the
    similarly-shaped types as legitimately distinct concepts in
    different namespaces (e.g., `network.Result` vs
    `compiler.normalize.Result`) — outcome (3) directly.
  - **Template → generated artifact.** A generated `.dag` produced by
    `build.rs` (or any structural emission step) from a template
    authority is, by construction, a derivative of the authority —
    not an independent reinvention. The substrate carries the
    generation edge explicitly (template header marker + `build.rs`
    splice site, or in the long term a `GeneratedFrom` registry
    row). The lens reads the generation edge as a resolution shape
    — same flow as outcome (1) alias-identity, scoped to mechanical
    generation. Firing on a generated artifact vs its template is a
    false positive by construction; reading the generation edge
    prevents it. Worked example: `r1_gates.dag` is generated from
    `r1_gates.template.dag` via `build.rs emit_r1_gates_fixture`
    splicing `src/v3/lenses/named_function_count.dag` into the
    `source:` field — the pair is a template→generated relationship,
    not parallel authority.

- *Decidability boundary (explicit):* (C1) and (C2) are mechanical and
  decidable over parsed substrate; (C3) is triage and never fires
  alone. The lens does **not** catch:
  - Two homes that share neither structural shape (C1 miss) nor
    fold-shape equivalence (C2 miss) but ARE conceptually the same.
    That residual case requires operator judgment and a
    `CanonicalConcept` row landing post-hoc, at which point Trigger
    B of the parent L1.12 takes over.
  - Behavioral equivalence in the general sense (undecidable). The
    lens does not attempt it; the homomorphism heuristic (C2) is the
    only behavioral-shape claim made, and it is structural, not
    semantic.

  This is the honest fold-bound: (C1) catches structurally-identical
  declarations regardless of name; (C2) catches un-derived
  catamorphisms regardless of how the body is spelled; everything
  else routes through operator judgment via the parent's resolution
  table.

- *Clearing receipt:* the substrate carries one of the parent's
  five resolution shapes (alias edge / retirement record /
  disambiguation row / canonical-concept row with one of the prior
  three / deletion). The lens re-fires on the same head until the
  resolution lands. **Fix-confidence: templated auto-apply** —
  for (C1) hits, the alias-identity rewrite is mechanical (the lens
  emits a `Diff` rewriting the redeclaration into an `import` +
  `type T = canonical.T`); for (C2) hits, the rewrite is the
  `fold_T` call with the algebra extracted from the original arms
  (the L1.5 catamorphism-derivation pattern, applied here as the
  fix shape). Reviewer overrides the canonical-home or
  algebra-binding names before commit.

- *Kills (real corpus):*
  - **`count_named_bind` corpus.** Identical
    `fn count_named_bind(behavior: Behavior) -> Int = match behavior
    { Value(v) => 0; Transform(t) => 0; Branch(b) => 0; ... }`
    appears in four places, but only two are independent declarations:
    - `src/v3/lenses/named_function_count.dag:15` — canonical lens
      (the program-text authority).
    - `src/v3/compiler/tests/t_demo/t_demo_fixtures.dag:28` —
      hand-authored test fixture: **fires on (C1)** as an
      independent declaration with identical shape and body.
    - `src/v3/compiler/tests/fixtures/r1_gates.template.dag:139` and
      `src/v3/compiler/tests/fixtures/r1_gates.dag:139` — a
      template→generated pair (build.rs splices the lens source into
      the template). **Resolved by the template-generated Escape rule
      above**, not a parallel-authority finding. Cited here to show
      the boundary works in practice.
    Practice-11-parameterized clean shape: one canonical home
    (`lenses/named_function_count.dag`), one import-alias from
    `t_demo_fixtures.dag`. The template→generated pair stays as-is.
  - **`RegisterStateSpace` parallel to `StateSpace`** in
    `src/v4/extdeps/languages/ptx.dag:34` and `:102`. `StateSpace`
    enumerates `Reg | SReg | Const | Global | Local | ...`;
    `RegisterStateSpace` enumerates `ResReg | ResConst | ResGlobal
    | ResLocal | ResParam{...}` — a refined subset with one
    extension (the `ResParam` carrying scope). Fires on (C1) —
    variant bijection holds for the shared variants; the extension
    + refinement should be made explicit via a refinement-edge or
    a `CanonicalConcept` row binding the two carriers. Refinement-
    not-parallel clean shape per the Escape rule above.
  - **`lookup_chain` (resolver) vs `fold_right` over scope chain.**
    `src/v4/compiler/03_resolve.dag:205` defines
    `fn lookup_chain(s: Scope, name: Symbol) -> Symbol?` as a
    hand-rolled structural recursion over `Scope = ScopeFrame {
    locals, outer } | ScopeRoot { module }`. Fires on (C2) — the fn
    is exactly `fold_T` with `T = Scope`, base case at `ScopeRoot`,
    inductive step at `ScopeFrame`, returning the first `map_get`
    hit. Should call `fold_right` (or the `Scope` carrier's algebra)
    instead. Note: this candidate is **(C2)-fires-only**, NOT (C1) —
    the signature is not identical to `fold_right`'s; it's the
    homomorphism shape that catches it.

- *Producer stage (see §10):* a new derived stage
  `v4.lens.structural_similarity` produces the shared index — the
  per-decl structural-shape facts (C1's variant-set + field-shape
  for types, C1's signature shape for fns, C2's catamorphism-form
  classifier, C3's token-set) that the lens consumes facts-flow-
  forward without re-walking declarations.

### L1.13 Skeleton-collapse lens — kills *parametric-arm duplication via constructor-name template* (proposed)

> **Status: proposed.** Derived from finding F14 (`feature_disposition` at
> `src/v4/extdeps/languages/llvm_ir.dag:570-585` — 12-arm match over
> `FidelityFeature` with 3 distinct RHS skeletons in a 1:5:6 distribution)
> and finding F15 (PR #3452 `complexity_bound_from_class` — 9 arms with
> 3 skeletons in a 1:1:7 distribution). Sibling-of L1.1 — where L1.1
> catches Bool-returning matches whose arms are literal `true`/`false`,
> L1.13 catches non-Bool matches whose arms are *templated by
> constructor name* (the RHSes share a skeleton, varying only by which
> constructor they apply). Practice 11's runtime-symptom catcher:
> Practice 11 stops parametric duplication at design time; L1.13 catches
> the symptom when Practice 11 was missed at design time.

> **(find, transform) discipline — auto-fix per dissolution.** Per §6
> convolution view: every L1.x lens is structurally a `(find, transform)`
> pair. L1.13's `find` half is the distribution-shape classifier; its
> `transform` half is the per-shape dissolution. Where the transform
> is **mechanically unambiguous from the find** (no human-design hooks
> needed), the lens emits a typed `Diff` (PR #3364 vocabulary —
> `Diff = List<Edit { at: Path, replacement: Node }>`) that goes into
> the candidate-state gate: `candidate_dag = apply_diff(dag, Diff)`,
> run lenses on candidate, commit if green. This is the auto-fix flow
> the operator framed: "every diagnostic can generate the correct code
> solution (and eventually apply it) — assuming we can safely infer
> user intent." Each distribution shape below carries an explicit
> **Fix-confidence** field stating whether the auto-fix is *direct*
> (no naming decisions, fully specified Diff), *templated* (Diff with
> name-holes the reviewer can override before commit), or — in future
> sub-signatures — *structural sketch* (lens identifies the kind of
> transform; concrete Diff requires human design). The fix-confidence
> axis is what makes L1.13 actionable rather than just diagnostic.

- *Signature:* a `match` expression over a closed coproduct with `N` arms
  whose RHSes collapse to `K` distinct skeletons under the substitution
  rule below, with a distribution shape meeting one of the four
  classifier definitions (PureTemplate / Outlier / MultiOutlier /
  Categorical). The diagnostic is the **(K, histogram)** pair where
  the histogram is the sorted list of group sizes; threshold
  definitions for each shape are exact and listed in the Distribution
  shapes section. Only `N ≥ 4` matches are considered for findings
  beyond PureTemplate / Outlier (smaller matches lack enough arms for
  Categorical / MultiOutlier reads to be load-bearing).
- *Decidable:* yes. The skeleton-extraction algorithm is:
  - **Identity source:** the lens consumes post-resolve / `InferredTree`
    canonical identities (qualified, alias-resolved). Source spelling
    variation, import renames, and module-qualification differences do
    NOT affect findings — two arms whose RHSes resolve to the same
    canonical identity have the same skeleton at that position.
  - **Substitution rule:** replace **every occurrence of the matched-arm
    constructor identity** (when used as a value or constructor in the
    RHS) with a per-arm hole; α-rename pattern-bound variables (their
    names don't affect skeleton equality); do NOT collapse unrelated
    free names, literals, or call arguments — those remain
    distinguishing.
  - **Skeleton equality:** two skeletons are equal iff they are
    structurally identical after the above substitution + α-renaming.
  - **Histogram:** sorted list of group sizes (largest first) where
    each group is a maximal set of arms whose RHSes share a skeleton.
    Distribution-shape classification below operates on the histogram.
- *Verdict:* hard error (🔴 dissolve-now). Per the universal three-disposition
  rule (§5.0 / Practices 4 + 10 — 🔴/🟡/🟢, "there is no fourth"), L1.13
  produces exactly one of those three. No "advisory" tier; legitimate
  exceptions pass via Escape (clean 🟢) below, not via a softer verdict.
- *Distribution shapes (closed enumeration with exact thresholds):*
  - **PureTemplate** — `K = 1`, `N > 1`. Histogram = `[N]`. All arms
    identical-modulo-constructor; dispatch doing zero work.
    **Dissolution:** delete the match; the operation doesn't depend on
    the variant. (Rare — usually a real bug.) **Clearing receipt:** the
    match is deleted OR replaced with the single shared RHS at the call
    site. **Fix-confidence: direct auto-apply.** Lens emits a typed `Diff`
    (PR #3364 vocabulary) replacing the `match` expression with its
    single arm's RHS. No naming decisions, no human-design hooks. Diff
    enters the candidate-state gate, runs lenses on candidate, commits
    if green.
  - **Outlier** — `K = 2`, histogram = `[N-1, 1]` (one base case + one
    uniform group). `N ≥ 3`. The match is performing a binary
    discriminator dressed as N-way dispatch. **Dissolution:** consume
    the discriminator *structurally* via match patterns + guards on the
    existing coproduct, OR substructure the coproduct so the one-vs-many
    distinction becomes a top-level variant. The function body becomes
    a 2-arm match of the form `match x { <Special_pattern> => special;
    _ => default }`, with the discriminator expressed as a sub-pattern
    or guard on the `Special_pattern`. **DO NOT extract a named `Bool`
    helper** — that is L1.1's predicate-dissolution anti-pattern.
    **Clearing receipt:** the function body is a 2-arm match (or
    pattern-with-guard) where the special/default split is *structural*
    in the pattern or sub-coproduct definition, NOT a `Bool` helper
    over the original coproduct. Recent precedent: PR #3359
    `connective_spec_fact` (6 arms → 2 skeletons, 5:1) — resolved via
    inline match-pattern with guard (`Atom { identity: id } if
    is_kernel_ambient(id) => ...; _ => default`), NOT via a named
    `Bool` helper. **Fix-confidence: split by RHS form** — skeleton
    equivalence is a comparison-time notion, not executable syntax, so
    the auto-fix has to materialize an actual replacement that compiles:
    - **(a) Constructor identity does NOT appear in the uniform RHS**
      (e.g. PR #3359 `connective_spec_fact`'s 5 uniform arms all
      executing `classify_density(facts)` — the matched constructor
      doesn't show up in the RHS at all). **Direct auto-apply:** the
      catch-all is literally `_ => <uniform_RHS>`; no constructor
      reconstruction needed.
    - **(b) Constructor identity DOES appear in the uniform RHS** (e.g.
      RHSes like `Disposition { feature: <matched_constructor> }` —
      this is more naturally a MultiOutlier shape, but Outlier
      examples exist). **Templated auto-apply:** bind the matched
      value in the catch-all (e.g. `c => <RHS_with_c_substituted_for_the_constructor>`)
      so the executable replacement is a real value-binding, NOT a
      skeleton hole. The lens emits the Diff with the binding name
      templated (default `c`/`x`/etc.; reviewer may override).
    - The naive "substitute the constructor with a wildcard `_`" form
      is a skeleton-equality device for COMPARING arms, not for
      generating executable replacement code; cases that require it
      should downgrade from direct-auto-apply to templated-auto-apply,
      not emit syntactically invalid `_ => Foo { field: _ }` RHSes.
  - **MultiOutlier** — `K ≥ 2`, histogram contains both singleton(s)
    AND non-singleton group(s). `N ≥ 4`. Multiple base-cases + one or
    more uniform groups. Covers F14 (1:5:6 — one singleton +
    two uniform groups) and F15 (1:1:7 — two singletons + one uniform
    group). **Dissolution:** substructure the coproduct so each
    singleton becomes its own top-level variant AND each uniform group
    becomes a single wrapped variant (`SizedKind { kind: SizedSubKind }`
    or similar). The function collapses to a number of arms equal to
    the number of distinct skeletons. **Clearing receipt:** the input
    coproduct's type definition carries the categorization
    structurally — singleton variants exist as top-level variants;
    uniform-group variants share a wrapping variant carrying the
    sub-discriminator. NOT acceptable: a hand-rolled `category_of(x):
    Category` function with N arms (just moves the L1.13 violation to
    a new name). Recent precedent: PR #3452 `complexity_bound_from_class`
    (9 arms → 3 skeletons, 1:1:7); F14 `feature_disposition`
    (`llvm_ir.dag:570-585`, 12 arms → 3 skeletons, 1:5:6).
    **Fix-confidence: templated auto-apply (name-templated, structure-direct).**
    Structural transform is unambiguous from the histogram (singleton →
    own top-level variant; uniform group → wrapped variant carrying a
    sub-coproduct). Lens emits a typed `Diff` with templated holes for
    NEW type names (wrapping variant name, sub-coproduct name, sub-variant
    names). Default templates derive from existing constructor names —
    e.g. F14's `DeclaredNormalized` uniform group (5 variants) → wrapping
    variant `NormalizedFeature` with sub-coproduct `NormalizedKind`.
    Singleton groups stay fieldless (no `{ kind: ... }` wrapper) per
    the dissolution rule below — only non-singleton groups get wrapped.
    Reviewer may override names before the candidate-state commit; same
    flow as any typo-fix Diff. Auto-apply when reviewer accepts default
    templates; one round of reviewer-edit when names need to differ.
  - **Categorical** — `K ≥ 2`, histogram contains ONLY non-singleton
    groups (every group size ≥ 2). `K ≤ ⌊N/2⌋`, `N ≥ 4`. Pure category
    projection with no singleton base case. Rarer than MultiOutlier in
    practice. **Dissolution:** push the categorization into the
    coproduct definition as N nested variants (each top-level variant
    wraps one of K sub-coproducts). **Clearing receipt:** identical
    discipline to MultiOutlier — the categorization lives in the input
    type's structure, not in a parallel hand-rolled projection function.
    **Fix-confidence: templated auto-apply (same as MultiOutlier).**
    Each uniform group → one wrapped variant carrying a sub-coproduct;
    templated names derive from group composition. Reviewer reviews the
    candidate `Diff`'s names before commit.
  - **Mixed** — `K` close to `N` (i.e., does NOT meet any of the four
    classifier thresholds above). Each arm does distinguishable work.
    **No finding** — legitimate per-variant dispatch.
- *Escape (clean 🟢):* `Mixed` distribution passes — distinct skeletons
  per arm IS legitimate per-variant dispatch. Two other clean-🟢 cases
  follow directly from the decidability rule:
  - **Per-arm distinct literal data.** A match whose arms each construct
    distinct literal data via per-arm references (e.g., test fixtures or
    compile-time data tables where arm `Variant_X` references
    `data variant_x_constant: T = ...`) is Mixed under skeleton
    extraction — each arm's literal IS a distinct skeleton because the
    reference is a free name, not bound by the arm constructor. Such
    matches are legitimate-per-variant enumeration and pass naturally;
    no separate verdict tier needed.
  - **Distinct call arguments per arm.** Matches whose arms call the
    same function with different concrete arguments (literals or distinct
    references) are Mixed for the same reason — the skeleton
    parameter-substitutes the leading constructor only, not call
    arguments (see decidability boundary below).
- *Kills:* `feature_disposition` (`llvm_ir.dag:570-585` — MultiOutlier
  1:5:6, arm-constructor name echoed in RHS) and `complexity_bound_from_class`
  (PR #3452 — MultiOutlier 1:1:7, inner `match size_var` block shared
  across 7 arms with only the outer `Bound` constructor differing).
  `manual_test_claim_for_manual_anchor` (`testgen.dag:253-270`) is a
  related but *distinct* pattern (see "Borderline case" below) — it
  passes the strict L1.13 decidability rule (Mixed) because the
  per-arm `claim_<name>` references are distinct literals; recognizing
  it as a finding requires either a sub-signature (proposed L1.13.b)
  that detects the arm-name-parameterizes-reference pattern, or a
  separate lens for the match-as-typed-table shape. Listed as a
  borderline case, NOT a current L1.13 kill.

**Decidability boundary (explicit):** RHS skeleton extraction collapses
the leading constructor name to a hole AND α-renames bound field names,
but does **NOT** collapse calls with distinct arguments. So:

```dag
A => f(x)
B => f(y)
```

is `K=1` (same skeleton: `f(<bound>)`); but

```dag
A => f(x, "literal-1")
B => f(x, "literal-2")
```

is `K=2` (literals are part of the skeleton). This matches the
real-world distinction — arms doing different literal work are doing
distinguishable work; arms doing the same call shape over different
bound names are duplicating.

**Connection to Practice 11.** L1.13 catches at implementation time
exactly what Practice 11 catches at design time: parametric duplication.
Practice 11 says "two declarations differing only by a typed parameter
are one parameterized declaration with the difference as a parameter."
L1.13 says "N match arms differing only by their constructor identity
are one parameterized operation with the constructor identity (or its
category) as a parameter." Same nerve, different layer.

**Concrete match — F14 (`src/v4/extdeps/languages/llvm_ir.dag:570-585`):**
```dag
fn feature_disposition(f: FidelityFeature) -> FidelityDisposition {
  match f {
    StructuralCore                  => Modeled
    SsaValueSpelling                => DeclaredNormalized { feature: SsaValueSpelling }
    BlockTextualOrder               => DeclaredNormalized { feature: BlockTextualOrder }
    LexicalTrivia                   => DeclaredNormalized { feature: LexicalTrivia }
    TargetConfigToken               => DeclaredNormalized { feature: TargetConfigToken }
    AdvisoryMetadata                => DeclaredNormalized { feature: AdvisoryMetadata }
    UnmodeledInlineAsm              => FailClosed { feature: UnmodeledInlineAsm }
    UnmodeledSemanticAttribute      => FailClosed { feature: UnmodeledSemanticAttribute }
    UnmodeledType                   => FailClosed { feature: UnmodeledType }
    UnmodeledInstruction            => FailClosed { feature: UnmodeledInstruction }
    UnmodeledConstExpr              => FailClosed { feature: UnmodeledConstExpr }
    MalformedStructure              => FailClosed { feature: MalformedStructure }
  }
}
```

12 arms, 3 distinct skeletons: `Modeled` (1), `DeclaredNormalized { feature: <constructor> }` (5),
`FailClosed { feature: <constructor> }` (6). **MultiOutlier, 1:5:6** (one singleton + two uniform groups; per the classifier definition above, Categorical requires non-singleton-only groups — the `Modeled` arm is a singleton, so this is MultiOutlier).

**Clean shape:** push the categorization into `FidelityFeature`.
**Singleton groups become fieldless top-level variants** (no `{ kind: _ }`
wrapping — a sub-coproduct of one element is empty overhead); **only
non-singleton groups get `{ kind: ... }` wrappers** carrying their
sub-coproduct:
```dag
type FidelityFeature
  = ModeledFeature                                 // singleton (StructuralCore) — fieldless
  | NormalizedFeature    { kind: NormalizedKind }  // 5 normalized variants
  | UnmodeledFeature     { kind: UnmodeledKind }   // 6 fail-closed variants

type NormalizedKind
  = SsaValueSpelling | BlockTextualOrder | LexicalTrivia
  | TargetConfigToken | AdvisoryMetadata

type UnmodeledKind
  = UnmodeledInlineAsm | UnmodeledSemanticAttribute | UnmodeledType
  | UnmodeledInstruction | UnmodeledConstExpr | MalformedStructure

fn feature_disposition(f: FidelityFeature) -> FidelityDisposition {
  match f {
    ModeledFeature                  => Modeled
    NormalizedFeature { kind: _ }   => DeclaredNormalized { feature: f }
    UnmodeledFeature  { kind: _ }   => FailClosed         { feature: f }
  }
}
```
3 arms instead of 12. The N-to-3 projection lives in `FidelityFeature`'s
type definition (its correct home — the category IS a property of the
feature), not as a hand-rolled function. **General rule for the
MultiOutlier / Categorical dissolution** — applies to every shape with
mixed singletons + non-singleton groups: singleton-group variants are
fieldless; only non-singleton groups carry `{ kind: <SubKind> }`
wrappers. Wrapping a singleton is pure structural overhead with no
discrimination work to do.

**Concrete match — F15 (PR #3452 `complexity_bound_from_class`):**
9 arms over `AsymptoticClass`, 3 distinct skeletons: `Constant` (1),
`unknown_complexity()` (1), `match size_var { Holds {v} => Bound { size_var: v, ...}; Violates _ => unknown_complexity() }` (7).
**MultiOutlier, 1:1:7** (two singletons + one uniform group; per the classifier definition above, Categorical requires non-singleton-only groups — both `Constant` and `unknown_complexity()` arms are singletons, so this is MultiOutlier).

**Clean shape:** collapse `ComplexityBound` from 9 parallel variants to
3 structural variants, wrapping `AsymptoticClass` for the 7 size-dependent
classes:
```dag
type ComplexityBound
  = UnknownComplexity { diagnostic: Diagnostic }
  | ConstantComplexity
  | SizedComplexity   { class: AsymptoticClass, size_var: SizeVariable }
```
Function collapses from 9 arms to 3 outer arms (`ClassConstant`,
`ClassUnknown`, `_`) — the `_` arm contains an inner 2-arm match over
`size_var`, so the total arm count is 4 if inner+outer arms are
flattened. The structurally meaningful reduction is 9 → 3 at the outer
dispatch. Full worked example in the PR review at #3452 comment.

**Borderline case — F16 (`src/v4/lens/testgen.dag:253-270`):**
13 arms over `T19ManualAnchorKey`. Strict skeleton extraction (per the
decidability boundary above) treats each arm's RHS as a distinct skeleton
because the per-arm `claim_<name>` references are distinct free literals
NOT bound by the arm constructor — under the strict rule, K = N → Mixed
→ **not an L1.13 finding as the lens is currently signed**.

Recognized as a borderline because the human pattern recognition does
catch it: the references follow a `claim_<arm_constructor_lower_case>`
naming convention that PARAMETERIZES BY the arm constructor. Catching
this structurally requires either:
- A sub-signature **L1.13.b** (future) that recognizes
  per-arm-named references as bound parameterizations (the lens would
  detect "every arm references a single `data` declaration whose name
  encodes the arm constructor"), OR
- A separate lens entry — "match-as-typed-table" — whose signature is
  "N arms each referencing N distinct typed `data` declarations with
  one-to-one correspondence to the matched variants."

The clean shape for the F16 pattern (sketched below) is what either
of those future lenses would target. F16 is documented here as a
related-pattern example so the boundary stays explicit, NOT as a
current L1.13 kill.


**Clean shape:** the 12 identical-skeleton arms are a typed registry
mapping anchor → claim. Refactor to a **totality-checked** table consumed
via direct lookup — `Option` + reader-convention "unreachable" comments
would be a fail-open smell here (Practice 2 — illegal states
unrepresentable). The closed enum already guarantees the lookup hits;
the substrate's job is to make that guarantee structural, not
commentary-based:

```dag
// Substructure the enum: split the outlier (Absent) from the lookup-
// carrying variants. T19ManualNonAbsent's k field is the closed sum of
// the 12 claim-bearing constructors.
type T19ManualAnchorKey
  = T19ManualAnchorAbsent
  | T19ManualNonAbsent { k: T19ManualNonAbsentKind }

type T19ManualNonAbsentKind
  = T19ManualTcConjEmpty
  | T19ManualTcDisjEmpty
  | ... // 12 variants, exactly the claim-keys

// TotalMap<K, V> for closed-coproduct K is a typed primitive whose
// construction is well-formed iff every variant of K appears as a key.
data manual_anchor_claims: TotalMap<T19ManualNonAbsentKind, ManualTestClaim> = {
  T19ManualTcConjEmpty            -> claim_conj_empty_compiles,
  T19ManualTcDisjEmpty            -> claim_disj_empty_compiles,
  ...
}

fn manual_test_claim_for_manual_anchor(key: T19ManualAnchorKey) -> Outcome<TestClaim> {
  match key {
    T19ManualAnchorAbsent           => Rejected { diagnostic: ... }
    T19ManualNonAbsent { k: kind }  => Produced { value: total_lookup(manual_anchor_claims, kind) }
  }
}
```

The 13 arms reduce to a 2-arm match over the substructured enum + one
totality-checked data table. `total_lookup` returns `ManualTestClaim`
directly (not `Option<ManualTestClaim>`) because `TotalMap`'s
well-formedness check makes the absence case unrepresentable. No
"unreachable" arm guarded by comment.

**Substrate dependency (scoped to L1.13.b / future match-as-typed-table lens, NOT base L1.13).**
`TotalMap<K, V>` for closed-coproduct K is a typed primitive whose
well-formedness check is "every K-variant appears as a key." This
primitive is **load-bearing for the future L1.13.b sub-signature** (or
a separate match-as-typed-table lens) that would catch the F16 pattern
— it is NOT a dependency of base L1.13. Base L1.13 (PureTemplate /
Outlier / MultiOutlier / Categorical on F14 + F15) has enough substrate
to run today via `fold_node` + skeleton extraction; it does NOT need
`TotalMap` to fire or clear. Scoping `TotalMap` to L1.13.b prevents
table-cleanup substrate work from blocking enforcement of the simpler
base lens.

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

| date | receipt | finding | core invariant violated | lens |
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

## 10. Dependency model — lenses are pipeline stages

The lens framework is **not separate infrastructure**. There is no
"lens engine" with its own dependency model. Lenses are `.dag` stages
in the existing compiler pipeline, and they participate in the same
`consumes:` / `produces:` machinery every other stage uses. A second
dependency system would itself be the L1.12-class parallel-authority
hazard the lens suite exists to kill — self-application: the lens
framework cannot violate the discipline it enforces.

### 10.1 Shared indices come from existing pipeline stages

Each compiler stage already produces structural facts as outputs. The
shared indices the lenses need are not a new layer — they're outputs
of stages that run anyway for typecheck / normalize / resolve / infer.
A lens consumes them via the same edges any other downstream stage
uses.

| Shared index | Produced by (existing stage) | Lenses that consume |
|---|---|---|
| AST + spans | `v4.compiler.02_parse` | universal |
| Symbol resolution (name → declaration site) | `v4.compiler.03_resolve` | L0.8, L1.8, L1.12, plus most L1.x |
| Coproduct variant list (type → variants) | `v4.compiler.02_parse` / `03_resolve` | L1.1, L1.5, L1.9, L0.12, L0.13 |
| Inhabitance edge set (type → {ctors, returns, fields, aliases}) | `v4.compiler.03_resolve` / `04_infer` | L1.2, L1.3, L1.4 |
| Witness / registry index (`data ... : Algebra<T>`, `CanonicalCarrier`, `HistoricalDeclaration`, `CanonicalConcept`, `canonical_observations`) | `v4.compiler.03_resolve` | L1.7, L1.8, L1.10.b, L1.12 |
| Refinement-clause index (type → length / domain refinements) | `v4.compiler.04_infer` | L1.7 |
| Import graph + target existence | `v4.compiler.02_parse` | L0.8, L1.8 |
| Return-type → fail-closed-carrier? | `v4.compiler.03_resolve` | L0.14, L1.11 |
| Match-arm RHS skeleton + per-arm group membership + per-skeleton constructor-hole presence (normalized RHS after α-renaming + matched-arm constructor substitution; groups expose arm-ids so the auto-fix consumes them as facts, not by re-walking) | `v4.lens.match_arm_skeleton` (new derived stage — see §10.2) | L1.13 |
| Per-decl structural-shape facts (type-decl: variant set + per-variant field shape; fn-decl: signature shape + body catamorphism-form classifier + identifier token-set) | `v4.lens.structural_similarity` (new derived stage — see §10.2) | L1.12.b |

### 10.2 Four small derived stages cover what the pipeline doesn't already expose

These are themselves `.dag` stages — small folds with declared
`consumes:` edges — and they're reusable across multiple lenses:

```dag
module v4.lens.match_arm_shape
consumes: v4.compiler.02_parse
produces: Map<MatchArmId, ArmShape>     // {trivial-literal | structural-work | identity-passthrough}
// reusable by: L1.1, L1.9, L1.11, L0.7, L0.13

module v4.lens.closed_vocab_scan
consumes: v4.compiler.02_parse
produces: Map<DeclId, Set<ClaimToken>>  // fact-bearing tokens in comments/identifiers
// reusable by: L1.7

module v4.lens.concept_home
consumes: v4.compiler.03_resolve, v4.lens.canonical_observations_index
produces: Map<FnId, File>               // each fn's primary-concept home file
// reusable by: L1.8

module v4.lens.structural_similarity
consumes: v4.compiler.02_parse, v4.compiler.03_resolve
produces: Map<DeclId, StructuralShape>
// StructuralShape = TypeShape | FnShape
// TypeShape = {
//   variant_set: [VariantShape],            // sorted; bijection-comparable
//   total_variants: Nat,
// }
// VariantShape = {
//   field_arity: Nat,
//   field_type_idents: [TypeId],            // by coproduct identity, not by name
// }
// FnShape = {
//   signature: SignatureShape,              // input coproduct identity, output coproduct identity, parameter arity
//   catamorphism_form: CatamorphismForm,    // None | StructuralFoldOver(TypeId)
//   token_set: Set<Token>,                  // identifier + variant/field names (C3 triage only)
// }
// SignatureShape comparison: bijection on parameter slots preserving
// type identity (NOT names); output type by coproduct identity.
// CatamorphismForm extraction: scan fn body for one-arm-per-T-variant
// match with recursive calls only at T-variant positions; classify
// `StructuralFoldOver(T)` if so, `None` otherwise.
// Facts Flow Forward (Practice 3): the lens emits everything L1.12.b
// needs for (C1) signature-match, (C2) catamorphism-equivalence, and
// (C3) token-vocabulary triage. No consumer re-walks declarations or
// re-derives shape equivalence — single authority (P2), one
// mechanism, multiple downstream projections.
// reusable by: L1.12.b, future cross-substrate-version drift lens,
// future module-graph homomorphism lens
// Algorithm: walk each parsed declaration; for `type` decls emit
//   TypeShape with the sorted normalized variant set; for `fn` decls
//   emit FnShape with signature normalized (parameter-bijection
//   canonical form), body classified for catamorphism form, and the
//   token-set extracted from the AST identifiers. Substrate-only,
//   no comment/prose inspection.

module v4.lens.match_arm_skeleton
consumes: v4.compiler.02_parse, v4.compiler.03_resolve
produces: Map<MatchExprId, SkeletonReport>
// SkeletonReport = {
//   arm_count: Nat,
//   distinct_skeletons: Nat,
//   histogram: [Nat],                // sorted group-sizes, largest first
//   classifier_shape: ClassifierShape, // PureTemplate | Outlier | MultiOutlier | Categorical | Mixed
//   groups: [SkeletonGroup],         // one entry per distinct skeleton — per-arm membership
// }
// SkeletonGroup = {
//   skeleton: RhsSkeleton,                       // the normalized RHS expression tree
//   arm_ids: [MatchArmId],                       // arms in this skeleton-equivalence class
//   constructor_hole_present: Bool,              // does the skeleton contain the constructor-hole at any
//                                                // position? (distinguishes Outlier sub-cases a/b
//                                                // without re-walking — drives Direct vs Templated
//                                                // auto-apply per the L1.13 entry above)
//   matched_constructors: [ConstructorId],       // the constructors whose arms collapsed to this skeleton
//                                                // (lens reads to derive templated names for the
//                                                // MultiOutlier/Categorical wrapping-variant default)
// }
// Facts Flow Forward (Practice 3): the lens emits everything the L1.13
// auto-fix and any future L1.13.b sub-signature need. No consumer
// re-walks the arms or re-derives skeleton equivalence — single
// authority (P2), one mechanism, multiple downstream projections.
// reusable by: L1.13, future L1.13.b (per-arm-name-parameterized-reference sub-signature),
// future match-as-typed-table lens
// Algorithm: tree-walk each arm's RHS, α-rename pattern-bound names,
//   substitute every occurrence of the matched-arm constructor identity
//   with a per-arm hole, structurally compare; group arms by skeleton
//   (each group records arm_ids + matched_constructors + whether the
//   skeleton contains the constructor-hole anywhere); sort group sizes
//   (largest first) to form histogram; classify distribution-shape per
//   L1.13's thresholds (PureTemplate / Outlier / MultiOutlier /
//   Categorical / Mixed).
```

Each is a single deterministic fold. Once landed, multiple lenses
share the result — landing `match_arm_shape` unblocks five lenses,
`match_arm_skeleton` unblocks L1.13 + the future L1.13.b and
match-as-typed-table sub-signatures.

### 10.3 A lens is a stage with declared dependencies

```dag
module v4.lens.dissolution
consumes:
  v4.compiler.02_parse
  v4.compiler.03_resolve
  v4.compiler.04_infer
  v4.lens.match_arm_shape
  v4.lens.match_arm_skeleton
  v4.lens.structural_similarity
  v4.lens.closed_vocab_scan
  v4.lens.concept_home
produces:
  Set<Diagnostic>
```

The compiler's existing stage-ordering — same machinery that ensures
`04_infer` runs after `03_resolve` — automatically schedules the lens
stage after its dependencies. **Adding a new lens = land a `.dag`
stage with its own `consumes:` declaration; pipeline ordering and
parallelism are derived, not configured.**

### 10.4 Parallelism falls out of the dependency graph

By §5.0's design, every lens is a deterministic structural projection
with no cross-lens mutable state. That gives two natural parallelism
axes derived directly from the dependency graph:

- **Per file.** Each `.dag` file's parse/resolve/infer state is
  independent of every other file's; the lens-stage fan-out is
  embarrassingly parallel — one task per file in scope.
- **Per lens within a file.** Two lenses reading the same shared
  indices have no contention (indices are read-only by the time
  lenses run); they fan out at the predicate-evaluation level.

Combined with `v4.lens.affected_set` (already in the pipeline; only
re-lens files whose downstream closure has changed), CI cost scales
with PR size, not with corpus size.

### 10.5 What this gives you

- **One dependency model** for the whole compiler — pipeline stages
  and lenses live in the same graph.
- **Adding a lens** is a `.dag` stage land, not a framework
  extension.
- **Adding an index** (when a new lens needs facts current stages
  don't expose) is a small derivation stage between an existing
  producer and the lens — and reusable by any future lens that
  shares the same dependency.
- **No "lens framework"** — there's just the pipeline, and lenses
  are stages in it.
- **Self-application is clean**: the pipeline is `.dag`-modeled,
  the lens stage participates as a peer, and the compiler enforces
  the discipline it follows.

## 11. Open — audit of current coverage

To be filled: an audit of which Layer-0 checks the v4 compiler enforces
today vs. the gap. The v4 compiler is early-stage (the pipeline is still
being modeled), so Layer 0 is expected to be a substantial current gap —
B1 should state that plainly once audited.
