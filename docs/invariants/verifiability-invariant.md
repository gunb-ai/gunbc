## Verifiability Invariant

**Design direction.** All `.dag` programs should be verifiable by
construction. Unverifiable programs should be structurally
unrepresentable — every construct carries enough structural information
for the compiler to derive its verification obligations.

**Current implementation:** Coercion data tests are auto-generated from
TypeCheckpoint/InhabitantDecl declarations (L0). Weather.dag L4 PoC
proves emitted code runs with structural witnesses. Witness generation,
algebraic law testing, and constraint oracle evaluation are not yet
implemented. See `src/v2/tests/testing-strategy.md` for the full level map.

This is the testing analog of the Decidability Invariant. Decidability
says: the structure makes unbounded computation impossible. Verifiability
says: the structure makes untestable code impossible.

### Structural proof from type system

Verifiability is a consequence of the type system, not a per-function
opt-in. The proof has three parts:

**Part 1: Every type has a constructible witness.**

Base values have canonical witnesses: `Bit` → `false`, `Int` → `0`,
`String` → `""`. Every constructor preserves witness-constructibility:

| Constructor | Witness | Constructible? |
|---|---|---|
| Product (Conj) | All fields present with child witnesses | Yes — product of constructible = constructible |
| Coproduct (Disj) | First variant with child witness | Yes — at least one variant is constructible |
| Optional | Both: present(witness) AND absent | Yes — two witnesses |
| Collection | Empty + one-element with child witness | Yes — two witnesses |
| Node | Children are a finite list of witnesses | Yes — finite list of constructible = constructible |

There is no type without a constructible witness. A type with 2^100
cardinality combinations still has a canonical witness — the compiler
doesn't enumerate all values, it constructs one representative per
structural form.

**Part 2: Every function is exercisable.**

A function takes typed parameters and returns a typed result. Since
every type has a constructible witness (Part 1), the compiler can:
- Construct input values from parameter type witnesses
- Call the function
- Check the output inhabits the return type

This is structural: the function signature IS the test specification.
The parameter types determine the inputs. The return type determines
the oracle. No hand-written test data needed.

**Part 3: Every algebra declares its own laws.**

Algebraic structures (Monoid, Ring, FreeMonoid, etc.) carry structural
laws: identity, associativity, commutativity. When a type inhabits an
algebra, the laws become verification obligations. The compiler generates
property tests from the laws and exercises them with witness values.

| Algebra | Laws | Generated test |
|---|---|---|
| Monoid | `op(identity, x) == x` | Call with identity + witness, assert equal |
| Ring | `add(zero, x) == x`, `mul(one, x) == x` | Call with zero/one + witness, assert equal |
| FreeMonoid | `concat(empty, xs) == xs`, `filter(xs, p) \|> all(p)` | Concat with empty, filter with predicate |

No composition of typed constructs produces an unverifiable program.
The type system is closed under composition for verifiability. QED.

### What this replaces

Without this invariant, testing is an obligation the developer manages
separately from the code. Tests are written after the fact, coverage
is tracked by external tools, and untested code silently ships.

With this invariant, testing is structural. The same way a `.dag`
developer cannot write an infinite loop (the structure prevents it),
they cannot write untested code (the structure generates the tests).
`under_specified` is not a status the compiler detects — it is a state
the structure makes impossible to represent.

### The one boundary

The only boundary where verifiability requires external evidence is
**integration with external systems** — real HTTP endpoints, real
databases, real cloud APIs. The compiler proves the mock contract
matches the type signature (structural). It generates integration
test artifacts for live verification (Tier 3). But it cannot prove
the real service's behavior matches the mock — that requires running
the test against the live system.

Inside the compiler's proof envelope: verification by construction.
Outside (external systems): generated tests with structural oracles.

### Relationship to decidability

Decidability and verifiability are the same structural guarantee
applied to different properties:

| Property | Mechanism | Structural source |
|---|---|---|
| Decidability | Every iteration bounded | Node.children is finite, 3 bounded primitives |
| Verifiability | Every construct testable | Types have witnesses, algebras have laws |

Both follow from the same root: `.dag` has no opaque types, no opaque
recursion, no opaque behavior. The compiler can see through all
structure. What it can see, it can prove. What it can prove, it can
verify.

