# Compositional Modeling in gunbc — How Conventions Become Compiler Checks

> **Mode:** `MIXED` (narrative doc spanning live + target material; per-claim tagging applied throughout)

## Claim-status summary

This doc walks from primitive types up to multi-service integration,
showing how `.dag` models correctness as a composition of meaning.
Some mechanisms shown here are live in the current tree; some are
target state with structural paths already named. The table below
summarizes before the body so readers can place each section.

| Section | Key mechanism shown | Status | Evidence / gap |
|---|---|---|---|
| Part 1 — primitives | `Int64 = OrderedRing<Word64>` | `[live]` | `dsl/std/bit.dag`, `dsl/std/integer.dag`, `dsl/std/algebra.dag` |
| Part 1 — primitives | Operations fall out of algebra attachment | `[live]` | same |
| Part 2 — refinements | `Nat = Int where x >= 0` | `[live]` | DB-3 lowered; `test_3a3_*` acceptance tests |
| Part 2 — refinements | Refinement preserves through arithmetic | `[live]` partially, `[target]` for full composition law | DB-3 supports parameter/generic `where` (`ROADMAP.md:231`); alias-RHS `where` still skipped in `src/v3/compiler/src/parse.rs` `skip_where_clause` — tracked under DB-11 (`ROADMAP.md:231`) |
| Part 3 — arity | `List<T>`, `Option<T>`, `NonEmpty<T>` as cardinality tags | `[live]` for List/Option; `[target]` for NonEmpty | NonEmpty as a first-class type composes on cardinality-substrate work tracked at `ROADMAP.md:305` ("Fixed-width types aren't structurally fixed" — cardinality not substrate-enforced until alias `where` parses/lowers per DB-11 gap) |
| Part 3 — arity | Nested-optional flatten by composition law | `[target]` | gated on cardinality-substrate row (`ROADMAP.md:305`) + DB-11 alias-RHS closure (`ROADMAP.md:231`) |
| Part 3 — arity | Testgen generating boundary tests from cardinality | `[target]` | DB-15 schema landed (`ROADMAP.md:235`); runner + `MockBackedInvariant` wiring remain under T-TestGen lane (`ROADMAP.md:51`, `:65`) |
| Part 4 — custom types | `Duration<Unit>`, `Money<Currency>` via dimensions | `[live]` for framework; `[target]` for unit-mismatch enforcement consumer | `src/v3/std/dimensions.dag` TERMINAL; Dimension wiring for lens consumers is deferred under the v3 lens honesty pass (`ROADMAP.md:333`) + DB-7 (`ROADMAP.md:235`). A Duration/Money unit-mismatch enforcement lens is not yet its own ledger row — see "Unscheduled gaps" below |
| Part 4 — custom types | `Secret<T>` as opaque nominal type | `[target]` | currently `Secret = String` alias per `dsl/std/types.dag:237`; no ROADMAP row schedules the nominal-wrapper graduation today — see "Unscheduled gaps" below |
| Part 5 — reconciliation | Cross-team AuthUser reconciliation | `[target]` | composes NonEmpty (cardinality substrate, `ROADMAP.md:305`) + Secret nominal wrapper (unscheduled, below) + enforced refinement preservation (DB-11, `ROADMAP.md:231`) |
| Part 6 — testgen | Generated integration tests for under-modeled boundaries | `[target]` | DB-15 runner + `MockBackedInvariant` wiring under T-TestGen (`ROADMAP.md:51`, `:65`, `:235`) |
| Part 7 — scale | Multi-service workflow with typed boundaries | `[target]` | composes Parts 3–6 targets; each specific gap cites its ROADMAP row above |

**Unscheduled gaps surfaced by this doc.** Two `[target]` items
above do not yet have their own tracked-debt row in `ROADMAP.md`
and are filed as follow-ups to add:
(a) Duration/Money unit-mismatch enforcement consumer (adjacent to
`ROADMAP.md:333`), and (b) `Secret<T>` nominal-wrapper graduation
(adjacent to `dsl/std/types.dag:237`). Per the doc-authority
single-ledger rule, these warrant ledger rows before the doc's
claims on them should be treated as scheduled.

**Reading guide.** Sections 1 and 2 describe the current tree.
Section 3 is half-and-half (cardinality exists; some compositional
laws are target state). Sections 4 through 7 are predominantly
target state with named structural paths. Per-claim tags in the body
mark each specific claim.

## The core idea

**Conventions are information. Types make information compiler-visible.**

Every bug class that shows up in production is information that
*should* have been in a type but wasn't. It was in a comment. A
README. A code-review conversation. A tribal agreement about what
`None` meant in this particular module. The information existed;
it just wasn't somewhere the compiler could read.

gunbc's approach is to make it easy to put the information in the
type itself — easy enough that you do it by default, not as a
heroic act of discipline. Once the information is in the type, the
compiler can reason about it, composing it across function
boundaries, across service boundaries, across teams. The bugs that
come from the lost information stop existing.

This document walks up from primitive types to multi-service
workflows, showing at each level how composition turns conventions
into checks.

---

## Part 1. Primitives are compositions of meaning

Most languages treat `int` as primitive — given, atomic, not up for
discussion. In `.dag`, even `Int` is composed. This matters because
the composition mechanism that builds `Int` is the same one you'll
use for your domain types. There is no special tier for "what the
compiler already knows."

### The layers

```dag
// dsl/std/bit.dag
type Bit = Classical
type Byte  { bits:  List<Bit> }     // Byte is structurally 8 bits
type Word64 { bytes: List<Byte> }   // Word64 is 8 bytes
```

Bit is the primitive. Byte is a structure over Bit. Word64 is a
structure over Byte. None of these carry any notion of "integer" —
they're just bits arranged in a shape. `[live]` — see
`dsl/std/bit.dag:20`, `:26`, `:31`.

```dag
// dsl/std/integer.dag
type Int8   = OrderedRing<Byte>
type Int16  = OrderedRing<Word16>
type Int32  = OrderedRing<Word32>
type Int64  = OrderedRing<Word64>

type UInt64 = Semiring<Word64>      // unsigned: no negate

type Int  = Int64
type UInt = UInt64
```

This is the load-bearing move. `Int64` isn't "a 64-bit integer." It's
a **carrier** (`Word64`) **with evidence that it inhabits `OrderedRing`**.
The integer isn't atomic; it's a pairing of a storage shape and an
algebra. `[live]` — `dsl/std/integer.dag:31-44`.

The algebra is where the behavior comes from:

```dag
// dsl/std/algebra.dag (abbreviated)

// Unary (one operation):

type Magma<T>             { op: fn(T, T) -> T }
// a way to combine two T's and get a T back.

type Semigroup<T>         extends Magma<T>    { /* associative */ }
// combining is associative — grouping doesn't matter: (a·b)·c == a·(b·c).

type Monoid<T>            extends Semigroup<T> { identity: T }
// there's a "do-nothing" element: combining with it leaves the other unchanged.

type CommutativeMonoid<T> extends Monoid<T>   { /* commutative */ }
// order doesn't matter: a·b == b·a.

// Ring-like (additive + multiplicative monoids + distribution):

type Semiring<T>     { add: CommutativeMonoid<T>, mul: Monoid<T>, /* distributive */ }
// two combines, add and mul; mul distributes over add. No subtraction.

type Ring<T>         extends Semiring<T> { /* additive inverses: negate */ }
// Semiring plus subtraction — every element has a negation.

type OrderedRing<T>  extends Ring<T>     { compare: fn(T, T) -> Ordering }
// Ring plus a total order — you can ask "which is smaller?"
```

The jump to Ring-like isn't a single next link in the unary chain —
it's the composition of **two** unary algebras (an additive
commutative monoid and a multiplicative monoid) with a distribution
law tying them together. That shape is what `Semiring` names. `Ring`
promotes the additive side to an abelian group (adds `negate`);
`OrderedRing` adds a total order. Each level adds exactly one
axiom's worth of structure. `[live]` — `dsl/std/algebra.dag:13-45`
(hierarchy diagram), `:145-150` (Semiring), `:154-160` (Ring),
`:176-193` (OrderedRing).

`OrderedRing<Word64>` gives you `+`, `-`, `*`, `negate`, `compare`,
`<`, `>`. These fall out of the algebra, as structural facts on
`Word64`. Not declared per-type; declared once on the algebra and
attached by the `inhabits` edge. `[live]` — `dsl/std/algebra.dag:176-193`.

### What this buys you

**Signed vs unsigned is a structural distinction, not a runtime
bit.** `Int64` inhabits `OrderedRing` (has `negate`); `UInt64`
inhabits `Semiring` (doesn't). Subtraction on `UInt64` is not
well-formed — the operation isn't in the algebra. `[live]`. In C,
unsigned subtraction silently wraps; the "it's unsigned" property
lives only in comments.

**Operations for your new type fall out for free.** If you declare
`type Money = OrderedRing<Int64>` and attach the algebra, you get
`+`, `-`, `*`, `<`, `>` without writing `Add`, `Sub`, `Ord` impls
manually. `[live]` — inhabits edges work today. Rust requires
explicit `Add<Money>`, `Sub<Money>`, `Ord` for `Money` as a newtype
over `i64` — about ten lines of boilerplate per type.

**Generic operations work on anything that inhabits the right
algebra.** A `sum` that takes any `Monoid` works on `List<Int>`,
`List<String>` (String inhabits FreeMonoid via concat +
empty), `List<Duration<Second>>` (if Duration inhabits
Monoid), `List<Bool>` (if you pick `(Bool, false, ||)` or
`(Bool, true, &&)` as the Monoid). One function; works on every
Monoid; no runtime dispatch; no boilerplate. `[live]`.

**Cross-target emission is determined by the carrier + algebra.**
Int64 in Rust becomes `i64`; in Python becomes arbitrary-precision
`int`; in Go becomes `int64`; in TypeScript becomes `bigint` (JS
`number` can't hold full Int64). Emitter reads the carrier + algebra
+ target spec and picks the correct native. `[live]` for Rust; see
`src/v3/spec/rust.dag`.

The pattern — **carrier + algebra attached by an `inhabits` edge** —
is the same pattern that extends through the rest of this document.
Everything is built this way.

---

## Part 2. Refinements attach conventions to types

A refinement is a rule you want the type to carry. "This integer is
always non-negative." "This string is always at least one
character." "This duration is always in seconds." In most languages,
refinements live in comments and are policed by vigilance. In
`.dag`, a refinement is syntactic: `type T = BaseType where rule`.

```dag
type Nat       = Int where x >= 0
type Positive  = Int where x > 0
```

`[live]` for parameter/generic refinement (DB-3 landed;
`test_3a3_*` acceptance tests). `[target]` for alias-RHS `where`
fully — per ROADMAP:107, `parse_type_rhs_after_eq` still drops the
alias-form `where` clause. The examples below are how it composes
once the alias gap closes; the parameter case works today.

### Refinements compose through the algebra

`Nat + Nat : Nat`. The compiler reads the algebra — `+` on
`OrderedRing` is total and closed — and combined with the refinements
`x >= 0 ∧ y >= 0 → x + y >= 0`, derives that the result is `Nat`.
The refinement is preserved because the operation is structurally
closed on the refinement. `[target]` — composition-preserves-
refinement is gated on the alias-refinement lowering landing.

`Positive * Positive : Positive`. Same logic: `x > 0 ∧ y > 0 → xy > 0`.
`[target]`.

`Positive - Positive : Int` (not `Positive`). Subtraction of two
positives can produce zero or negative. The compiler drops the
refinement because it isn't preserved by the operation. `[target]`.

This is the meta-beat: **refinements aren't annotations the compiler
trusts blindly.** They're structural facts, and composition preserves
them only where the algebra allows. You don't lie to the compiler;
the compiler drops information that isn't provably preserved.

### The difference from Rust newtypes

In Rust, `pub struct Nat(pub u32);` doesn't preserve the "non-
negative" property through arithmetic unless you write every
operation yourself. `Nat::new(x + y)` where `x, y: Nat` requires
you to extract the inner values, add as `u32`, wrap back — and the
compiler doesn't check that the result is still in range unless
`Nat::new` validates at runtime.

In gunbc, the preservation is a compile-time fact derived from the
algebra, not a runtime check you write. `[target]` — composition
law needed.

### The generalization

Any convention you have in your head — "this port number is between
1024 and 65535," "this username is at most 40 characters," "this
email must match an RFC 5322 pattern" — can become a refinement.
Once typed, the convention is information the compiler carries.

```dag
type Port            = Int where 1024 <= x && x <= 65535
type Username        = String where length(x) <= 40
type Email           = String where rfc5322_valid(x)
type BoundedString<N> = String where length(x) <= N
```

`[target]` for all of these — examples of what refinement-lowering
will enable once the alias-RHS parsing closes per ROADMAP DB-11
(generic-parameter `where` already lowers; alias-RHS is the gap —
`src/v3/compiler/src/parse.rs` `skip_where_clause`).

### Compile time vs runtime

A traditional-language engineer will object: "I can already do this
with a smart constructor."

```rust
struct Port(u16);
impl Port {
    pub fn new(n: u16) -> Result<Port, PortError> {
        if n < 1024 || n > 65535 { return Err(PortError::OutOfRange); }
        Ok(Port(n))
    }
}
```

True — and this works. But the guarantee is **runtime**.
`Port::new(80)` compiles cleanly; the error only surfaces when the
constructor executes. Every caller either `.unwrap()`s (deferring
the failure to production) or threads a `Result` — so the
constraint becomes control flow replicated at every callsite. The
type system itself has no opinion on whether `n` is in range; it
trusts the constructor to do the runtime work and to keep doing it
as the code evolves.

In gunbc, `type Port = Int where 1024 <= x && x <= 65535` puts the
constraint on the type itself, not on an ephemeral constructor. The
compiler rejects `let p: Port = 80` as a type error. Crossing a
boundary from an unrefined `Int` into a `Port` requires an explicit
narrowing that the compiler *sees* happen — not a `try/catch` that
the type system can't reason about. Runtime validation doesn't
disappear: it still lives at the actual boundary with the outside
world — a config parse, a socket read, a user form. But it lives
**only** there. Internal code uses `Port` with compile-time
confidence. `[target]` for literal-level and flow-level alias-form
enforcement once DB-11 lands; generic-parameter `where` is live
today.

The move: a discipline traditional languages enforce at runtime is
promoted to a compile-time property of the type. It's not a new
feature on top of the language — it's a relocation of where the
same guarantee lives.

### The pattern

**Every convention in your head is a candidate type.** The question
becomes: which conventions are so important that they warrant
compiler-level enforcement? In practice, many of them are — and
encoding them once at declaration time is cheaper than enforcing
them at every call site in review.

---

## Part 3. Arity is the layer that remembers

This is the part that will feel visceral. The parser-team story.

### The story

A team is building a parser. They have a `ParseTree` type and a
convention: *if the tree has zero nodes, deallocate it and set the
reference to null.* This convention saves a pointer indirection. It
lives in the original engineer's head and a comment at the top of
the parser file.

Two downstream engineers consume the parser's output.

**Engineer A** reads `Option<ParseTree>` and writes:

```dag
match parse_result {
  Some(tree) => process_tree(tree),    // assumes tree has ≥1 node
  None       => skip(),
}
```

Engineer A's assumption — `Some(tree)` means "tree with at least one
node" — is consistent with the parser author's convention. A's code
works.

**Engineer B** reads the same type and writes:

```dag
match parse_result {
  Some(tree) => process_tree(tree),
  None       => tree_was_never_parsed(),   // assumes None = not invoked
}
```

Engineer B's assumption — `None` means "parse was never called" —
is *inconsistent* with the parser author's convention. `None` could
also mean "was called, produced zero nodes, got cleaned up." B's
code has a latent bug that appears only when the parser processes
input with zero statements (empty file, or all comments, or
macro-expanded away).

Both engineers' code compiles. Both engineers' tests pass in their
local reasoning. The bug surfaces in production, months after the
two engineers finished their features.

### What went wrong, in type-theoretic terms

Three distinct semantic states existed:

- **S1:** "No tree was ever created" (parse never invoked)
- **S2:** "A tree exists, has at least one node"
- **S3:** "A tree exists, has zero nodes" (empty)

The parser author's convention *compressed* S3 into S1 — empty tree
deallocated, reference nulled. This saved an indirection but **lost
information**: once you have `None`, you cannot tell whether you're
in S1 or S3.

Engineer A's code assumes the compression was one-way (S3 never
reappears once deallocated) and that `Some(t)` implies S2. Correct
under the convention.

Engineer B's code assumes `None` means S1 only. Incorrect: the
convention means `None` represents S1 ∪ S3.

This isn't a logic error. It's an **information loss inside the
type system**. The type `Option<ParseTree>` cannot distinguish S1
from S3, so the information has to live in human agreements. The
agreements drift. New engineers join. People make reasonable-
looking assumptions that contradict the silent convention.

### What gunbc's arity types do

The fix is to **pick a type where each distinct semantic state has
a distinct structural shape**. The convention becomes a compiler
check.

**Shape 1: tree always exists; nodes may be empty.**

```dag
type ParseTree { nodes: List<Node> }    // List<T> admits empty
```

The parser author cannot implement the "deallocate empty tree"
convention. The tree is never null. Reading `tree.nodes` gives a
`List<Node>` that might be empty; pattern-match or fold handles
that. `[live]` — List admits empty by default; every caller gets a
list they can't assume is non-empty.

**Shape 2: tree always has ≥ 1 node; absence is its own state.**

```dag
type ParseTree { nodes: NonEmpty<Node> }      // AtLeast(1)
let maybe_tree: Option<ParseTree> = parse(src)
```

`NonEmpty<Node>` is `Cardinality<Node, AtLeast(1)>` — a type-level
tag saying "at least one Node." `ParseTree` structurally cannot hold
zero nodes. `Some(t)` carries the guarantee `t.nodes.length >= 1`.
`[target]` — NonEmpty as a first-class type is a gap today; `.first()`
on NonEmpty returning `T` (not `Option<T>`) requires cardinality
composition.

With Shape 2:
- Engineer A's code works: `Some(t)` does mean ≥1 node.
- The parser author's convention (empty → null) is expressible —
  it's the thing Shape 2 *forces*.
- Engineer B's assumption (`None` → never parsed) is the natural
  reading: if the tree has no nodes, the only way to represent
  that is `None`, so `None` carries the S1 ∪ S3 meaning
  *explicitly at the type*.

Still one small ambiguity: `None` is S1 ∪ S3. If that matters,
Shape 3 separates them.

**Shape 3: all three states distinct.**

```dag
type TreeState = NeverCreated
               | Active(NonEmpty<Node>)
               | Emptied
```

Pattern-match forces handling all three. No convention required;
the compiler enforces the distinction. `[target]`.

### Cardinality as a substrate axis

`List<T>`, `Option<T>`, `NonEmpty<T>`, `BoundedList<T, N..M>` are
all values of the same axis: **cardinality**, the type-level
information about "how many." This isn't a Rust-style enum of
library types; it's a structural property of the substrate that
propagates through composition.

```
List<T>            = Cardinality<T, Unbounded>
Option<T>          = Cardinality<T, AtMost(1)>
NonEmpty<T>        = Cardinality<T, AtLeast(1)>
FixedArray<T, n>   = Cardinality<T, Exact(n)>
BoundedList<T, a..b> = Cardinality<T, Between(a, b)>
```

When you compose operations on cardinality-bearing values, the
compiler tracks what happens to the cardinality:

- `map(f, List<T>) : List<U>` — cardinality unchanged.
- `filter(p, List<T>) : List<T>` — refinement weakens: `NonEmpty<T>`
  after filter drops to `List<T>` (filter may produce empty). `[target]`.
- `fold(init, f, List<T>) : U` — no longer cardinality-bearing.
- `first(NonEmpty<T>) : T` — returns a value, not `Option<T>`,
  because the cardinality guarantees at least one. `[target]`.
- `first(List<T>) : Option<T>` — returns Option because List may
  be empty.

### Nested optionals flatten structurally

In Rust, `config.get("db").and_then(|db| db.get("primary")).and_then(|p| p.get("host"))`
produces `Option<Option<Option<String>>>` if you forget to chain
with `and_then`, and you're expected to call `.flatten()` manually.

In `.dag`, cardinality composition has a law:

```
Cardinality<Cardinality<T, AtMost(1)>, AtMost(1)>  ≡  Cardinality<T, AtMost(1)>
```

Nested optionals collapse by the composition law, not by user
action. `[target]` — this is the cardinality substrate work gated
behind ROADMAP class 3 (nested-optional flatten).

### Testgen as the last line of defense

Cardinality works well for internal types. But at external
boundaries — a REST API you didn't design, a legacy database, a
third-party event stream — the data has structure your types can't
fully capture. The JSON field `tags` is sometimes a string,
sometimes an array. The GraphQL response sometimes has `null`
where the schema says "non-null." You can't fix the external shape
from inside.

gunbc's answer is **generated integration tests**. The compiler
reads the type boundaries, enumerates the shapes the data could
take (cardinality transitions: empty / one / many; refinement
boundaries: just-at-limit / just-over; optional variants: present /
absent), and generates tests exercising each. You don't write them;
they come from the type structure. `[target]` — DB-15 schema is
landed; the runner + mock-backed harness is the follow-up.

The reframe this enables:

- **Good modeling** → short testgen output, because the type already
  proves most of what matters. You only need tests for what the type
  couldn't capture (the external shape).
- **Poor modeling** (under-modeled external API, stringly-typed
  data, optionality hiding in JSON) → long testgen output, because
  the type leaves more ground uncovered. Testgen covers the
  residual automatically.

Testing turns from "what should I write?" into "what doesn't the
type already prove?" The better your model, the less testgen
generates — because the type is carrying the information that tests
would otherwise have to cover.

This is why the arity discipline matters even for boundaries you
don't control. The part of the shape you *do* model is covered by
the compiler; the part you *can't* model is covered by generated
tests. No ground is uncovered. No hand-written boundary tests.

---

## Part 4. Any convention can become a type

Primitives are compositions of meaning. Refinements attach
conventions structurally. Cardinality retains information. These
aren't three separate mechanisms — they're instances of one idea:
**types are information carriers, and the composition is how the
information propagates.**

Every convention your team has can be a type.

### Money with currency

```dag
type Money<Currency> = OrderedGroup<Int64>
```

`[target]` — depends on user-defined parametric algebra attachment,
ROADMAP DB-18 territory.

`Money<USD>` and `Money<EUR>` are different types because the
phantom parameter differs. The algebra (addition, negation,
comparison) is inherited. `Money<USD> + Money<USD> : Money<USD>`.
`Money<USD> + Money<EUR>` is a compile error. `Money<USD> *
Money<USD>` is a compile error — `*` isn't in `OrderedGroup`. In
C/Rust you'd typically have `Money` as a newtype over `i64` and
get `*` by default, producing nonsense like "5 USD × 3 USD = 15 USD
squared."

The phantom parameter is how conventions about identity propagate
through arithmetic.

### Duration with units

```dag
type Duration<Unit> = Dimension<Int64, Unit>
```

`[live]` for the `Dimension<Carrier>` framework per
`src/v3/std/dimensions.dag` (🟢 TERMINAL for structural surface).
`[target]` for the unit-mismatch enforcement lens — the consumer
that rejects `Duration<Second> + Duration<Millisecond>` without
explicit conversion. DB-3 shipped the framework; the enforcement
wire-up is follow-up.

`Duration<Second>` and `Duration<Millisecond>` are distinct types.
Addition respects dimension. Conversion is an explicit operation.
Rust's `uom` crate does the same thing, but opt-in and library-
level; gunbc does it substrate-level with one declaration.

### Secrets with nominal opacity

```dag
type Secret<T> { value: T }
  where only std.secrets::acquire may construct
        no Show instance
        no String coercion
        no Debug derivation
```

`[target]` — `Secret = String` is a type alias today per
`dsl/std/types.dag:237`; nominal opaque wrapper is a gap closed by
the T-Secret lane.

Once `Secret<Token>` is nominally opaque:
- `println("token=" + secret)` → compile error (no String coercion)
- `info!("{:?}", secret)` → compile error (no Debug)
- `serialize_json(secret)` → compile error (no Show)
- Construction only through `std.secrets::acquire(vault_key)`
  (typed secrets manager); no raw constructor

The convention "don't log secrets" becomes a structural fact,
enforced at every call site, not at code review.

### The common pattern

Each of these is the **same shape** as Int64 = OrderedRing<Word64>:
a carrier + an algebra + refinements + phantom parameters. The
author writes the declarations once; consumers inherit the
information carried by the type; the compiler reasons about
composition.

**Users don't learn a new mechanism for each domain.** The
mechanism for Int is the mechanism for Duration is the mechanism
for Secret is the mechanism for AuthUser. Composition all the way
up.

---

## Part 5. Teams compose; the compiler reconciles

Here's where the pitch lands at enterprise scale.

Software at any non-trivial scale involves multiple teams, multiple
codebases, multiple conventions. Team A models `Order` one way;
Team B consumes `Order` and models it slightly differently because
their concerns are different. Traditionally, this is "reconciled"
in one of three unsatisfying ways:

1. **A shared type authority** (protobuf / OpenAPI / a common
   crate). One team's shape wins; the other team silently downgrades
   at the boundary. Bugs appear when the loser's convention was
   load-bearing.
2. **Hand-rolled converters** at every call site. Each team writes
   `team_a_order_to_team_b_order(o)`. The converter catches the
   obvious mismatches and silently loses the subtle ones (an
   `Option<Customer>` becomes a `Customer` via `.unwrap_or(...)`
   with a fallback that was wrong in edge cases).
3. **A canonical "third" shape** that represents neither team's
   mental model. Both teams write adapters; neither team's
   invariants hold in the canonical shape; bugs appear at both
   boundaries.

gunbc's approach is different: **each team models what they mean.
The compiler arbitrates at integration points.**

### Two teams, two AuthUser types

**Team A** (order-management):

```dag
type AuthUser {
  id:      UserId
  scopes:  List<Scope>         // convention: empty scopes = read-only / guest
}
```

**Team B** (analytics pipeline, consumes Team A's events):

```dag
type AuthUser {
  id:      UserId
  scopes:  NonEmpty<Scope>     // convention: every analytics event has ≥1 scope
}
```

Both types are legitimately called "AuthUser." Both describe
"the authenticated user who took this action." Both teams' mental
models are *internally* correct for their domain. The conventions
disagree.

### At the boundary

Team B writes:

```dag
let a_user: TeamA.AuthUser = receive_from_orders()
let b_user: TeamB.AuthUser = a_user          // compile error
```

The compiler error:

```
cannot convert TeamA.AuthUser to TeamB.AuthUser:
  field `scopes`: List<Scope> is not NonEmpty<Scope>

TeamA.AuthUser admits empty scopes (guest-checkout shape);
TeamB.AuthUser requires non-empty scopes (analytics-event shape).

Explicit reconciliation required. Choose one:
  - Reject at boundary:  NonEmpty.from_list(a_user.scopes)?
                         (returns Option; caller handles empty)
  - Fall back to default: NonEmpty.from_list_or(a_user.scopes, [Scope.Read])
  - Model as distinct types; pick per-call-site.
```

`[target]` — this kind of cross-team structural reconciliation error
composes existing mechanisms (NonEmpty, Option, refinement
preservation) that are target state today.

**Each team continues to use their own convention internally. The
boundary is the one place the reconciliation happens, and the
compiler forces it to be an author-time choice.**

- No silent downgrade ("empty → []" without the programmer noticing)
- No shared-crate ceremony ("let's put AuthUser in a third crate
  and hope") 
- No hand-rolled converter that quietly does the wrong thing

### Why this matters for principal engineers

Enterprise software grows by integration. Two teams at your
company. A contractor. A vendor. A legacy system. An acquisition.
Each brings models. Each has conventions. Integration is where
your bugs live — not inside any one team's code, but at the
seams between teams.

Traditional languages make you adopt a shared-authority strategy
to bridge teams. gunbc lets each team own its model and makes the
compiler the arbiter at boundaries. **This is strictly more
sustainable** than shared authority, because:

- No team has to compromise their internal convention
- No "canonical" third shape that satisfies no one
- The integration surface becomes a set of reconciliation points,
  each compile-checked
- Adding a new team means declaring their model + the compiler
  computes the reconciliation edges — not refactoring six
  downstream teams to adopt the new shape

---

## Part 6. Testgen for what types can't cover

Types carry the information you can express. Some information is
external and you can't fix the shape — a REST API from a vendor; a
legacy database; an event stream whose schema drifts monthly.

For these boundaries, gunbc generates integration tests from the
type structure. `[target]` — DB-15 schema landed; runner +
mock-backed harness is follow-up work.

### How it works

A `TestClaim` is declared in `.dag` (schema at `src/v3/std/verification.dag`):

```dag
data auth_user_roundtrip_test: TestClaim = {
  predicate:  MockBackedInvariant(
    subject:  receive_from_orders_v2,
    mock:     orders_api_simulator,
    invariant: response_matches_boundary_contract,
  )
}
```

The testgen runner:
1. Enumerates the type transitions at the boundary
2. Generates mock responses covering each cardinality + refinement
   case (empty scopes / one scope / many scopes; just-at-limit /
   just-over / far-below)
3. Runs the workflow against the mock for each
4. Asserts the workflow's output matches its declared contract

Every cardinality transition you encoded in a type becomes a test
the compiler generates. You didn't write the test; the type did.

### The modeling-to-testing pipeline

The relationship between modeling and testing becomes:

- **Fully modeled boundary** → testgen generates zero tests for it
  (the type proves what matters; no residual coverage needed)
- **Partially modeled boundary** → testgen generates boundary tests
  for what the type doesn't cover
- **Unmodeled boundary** (just `String` or `Json`) → testgen
  generates many tests because the type leaves most ground
  uncovered

The programmer's leverage: **spend effort on the type**, and
testgen output shrinks automatically. Traditional tradeoff was
"more types = more work up front; simpler types = more tests
later." gunbc flips this: types are cheap to extend; tests are
derived from them; the work is frontloaded into modeling *once*,
not into test-writing *repeatedly*.

### The last line of defense

"Last line of defense for under-modeled interfaces" isn't a
marketing slogan; it's the explicit role of testgen in gunbc's
architecture. Testgen is what catches what the modeling didn't.
If your types are complete, testgen is mostly silent. If your
types are sketchy, testgen lights up and covers the gaps.

No boundary is "unprotected." No bug class is "oh we'll catch it
in prod." Either the type caught it, or testgen did.

---

## Part 7. What this unlocks at scale

When five services compose in one workflow — say, a GitHub
issue-classifier hitting GitHub, Anthropic, Postgres, GCS, Slack —
every mechanism described above applies in the same way. `[target]`
— the five-service workflow example depends on typed service
boundaries (classes 1, 2, 7 in the impossible-bug-classes audit),
full cardinality composition, the reconciliation story, and
testgen's mock-backed runner.

Typical integration code in Rust: ~2,400 lines across 20 files
covering five services × their types × their retry policies × their
error unions × their mocks × their test coverage. Every line is a
place for a bug, and every pair of services is a potential
reconciliation miss.

In gunbc, the same workflow is ~150 lines of type declarations +
~30 lines of workflow intent. That's not because gunbc is terse;
it's because ~2,000 of the Rust lines exist to carry information
that `.dag` types carry by default. Retry semantics, error unions,
schema consistency, IAM scoping, secret flow tracking, mock
harnesses, boundary tests — all fall out of the modeling.

**Integration effort in gunbc is linear per service.** Each service
costs roughly its own declarations. Composition is the compiler's
problem, not yours.

**Integration effort in Rust is super-linear.** Each new service
pair introduces a new boundary, a new type-sharing question, a new
set of reconciliation decisions, a new mock to write, a new test
suite to add. The matrix of service pairs grows quadratically; the
bugs live in the off-diagonal entries.

This is the sustainability pitch. At 5 services, gunbc looks like
"nice but marginal." At 15 services, gunbc looks like "yes,
obviously." At 50 services — modern enterprise scale — gunbc looks
like the only way a human being can reason about the whole thing.

---

## Closing

The core thesis rewritten at the end:

**Conventions are information. Types make information compiler-
visible. Every layer of type structure — primitives, refinements,
cardinality, phantom parameters, service boundaries — is a
convention made explicit. What the compiler can see, it can reason
about. What it can't see, it can't help with, but testgen covers
the residual.**

gunbc's claim is not "we prevent 26 bug classes." It's **"we turn
the conventions your team has been carrying in readmes, comments,
and discipline into types, and then the bugs that come from
dropped conventions stop existing."**

The sustainability argument: types are cheap to extend and compose
because the mechanism is uniform — Bit all the way up to AuthUser
uses the same pattern. As your system grows, your types grow; the
compiler's leverage grows with it. Tests you would have written by
hand in Rust are either not needed (type proved it) or generated
(testgen did). Convention reconciliation across teams is a compile-
time step, not a ceremony.

A principal engineer adopting gunbc isn't signing up for more
up-front modeling work. They're signing up for less downstream
reconciliation, fewer production bugs from dropped conventions,
less hand-written glue code, and less tribal knowledge in readmes.
The work moves from repetitive (every integration, every test,
every review) to once (the declaration).

That's the pitch, told end to end.

## Related documents

- `THESIS.md` — parent thesis claims
- `ROADMAP.md §"Release R1 Program"` — what's on deck for R1
- `docs/thesis/doc-authority.md` — governance for this doc
- `docs/thesis/epistemic-stacking.md` — why every concept grounds
  in primitives
- `docs/thesis/correctness-dimensions.md` — the dimension mechanism
  referenced in Part 4
- `ROADMAP.md` tracked-debt ledger — where each `[target]` item
  traces for structural path
