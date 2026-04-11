> Part of: [THESIS.md](THESIS.md) — modeling guidelines ensure that
> every `.dag` construct is grounded in external fact, so the causal
> engine can validate it.

# DAG Modeling Guidelines

Companion to [INVARIANTS.md](INVARIANTS.md) (compiler invariants).
This document covers the **modeling quality** of `.dag` source
files — the domain models themselves, not the compiler that
processes them.

---

## Core principle: shared facts, not preferences

Every node in a `.dag` model should be either:
- **An axiom** — a fact cited from a standard, specification, or API doc
- **A derivation** — composed from axioms via an objective relationship

The modeling is a deductive system, not a design document. If someone
disputes a cross-section of the DAG, the resolution is "here's the spec"
— not "here's why I think this is a good abstraction."

At any cross-section of any DAG in the codebase, the content should be
**non-controversial** — a shared fact that people actually agree on.

### No meta-language on top

The `.dag` language is itself the meta-language for formalizing
intersubjective programmer agreements. Adding annotations or metadata
on top of `.dag` would create a meta-meta-language — another dimension
of intersubjectivity ("annotate this to fix this concept I don't like").

When a fact needs structural representation, define a proper `.dag`
structure with proper transforms. Algebraic laws are `.dag` functions,
not string annotations. Type facts are `.dag` data fields, not comments
or decorators.

The test: if you find yourself reaching for an annotation, you've found
a fact that the current structure doesn't capture. The fix is to extend
the structure — new type fields, new edge relationships, new function
signatures — not to paper over the gap with metadata.

### Start with the fact

Every new construct should begin by identifying the external fact it
models. This is not aspirational — it is the entry point for all design
work in this codebase.

| Kind of construct | Fact source |
|---|---|
| Algebraic structure | Mathematical axioms (associativity, identity, etc.) |
| Type definition | Language specification, standard, or structural derivation |
| External service | API documentation, protocol spec |
| Cross-language mapping | Shared algebraic structure (both targets inhabit the same algebra) |
| Refinement type | Narrowing predicate derivable from the base type's definition |

If you cannot name the fact, the construct is not ready. This is the
difference between this codebase and a design document: a design document
expresses preferences; this codebase expresses facts. An ungrounded
construct is a preference masquerading as a fact.

## Foundational primitive: truth-valued structure

### The single primitive

The preference is deliberate: **primitives should be as small as
possible.** The smaller the primitive, the less it hides.

In digital computing, the smallest unambiguous primitive is
**truth/falseness** — a proposition that holds or doesn't. Classical
bivalent logic: true or false, 1 or 0.

This is the only primitive in the system. Everything above it —
String, Int, Float, List, Map, services, resources — is explicit
composition built on top of logical structure.

### Why classical logic (and not something else)

The primitive depends on the computational model. Classical
bivalent logic is not universally "the" foundation — it's the right
foundation for classical digital systems:

| Computational model | Primitive | Algebra |
|---|---|---|
| Classical digital | truth/false | classical logic |
| Continuous/analog | real quantity | real analysis |
| Probabilistic | degree of belief [0,1] | probability theory |
| Quantum | complex amplitude | linear algebra over Hilbert spaces |

gunbc targets digital computing, so the foundation is classical
logic. But the composition layer (Node, children, connectives) is
**model-independent** — AND/OR/IMPLIES have analogs in every algebra:

- AND: conjunction / min / joint probability / tensor product
- OR: disjunction / max / union probability / direct sum
- IMPLIES: entailment / order / conditional probability / subspace

The DAG structure doesn't care which algebra is underneath. It
composes things with connectives. What those connectives *mean*
depends on the foundation you install. Classical logic is a
parameter of the system, not a hardwired assumption.

This matters because it means the architecture is sound even if
the foundation changes. A probabilistic extension (fuzzy types,
confidence intervals) wouldn't require rearchitecting the
composition layer — it would require a different foundation
algebra and kernel types, but Nodes and children and connectives
would still work.

### Why Int and String are too wide

| "Primitive" | What it actually hides |
|---|---|
| `Int` | Bit width? Signed? Two's complement? Arbitrary precision? |
| `String` | Encoding? Null-terminated? Length-prefixed? Byte-level or code-point-level? |
| `Float` | IEEE 754 binary32? binary64? Decimal? Platform-dependent? |
| `Bool` | Unambiguous — this IS the primitive. |

Every compiler answers these questions differently. C's `int` is
platform-dependent. Rust's `i32` is exactly 32-bit two's complement.
Python's `int` is arbitrary precision. JavaScript's `number` is IEEE
754 double. The "primitive" is doing hidden work — implicit decisions
masquerading as axioms.

The fix: make the decisions explicit. Define Int as a composition with
a precise specification. The composition IS the definition. The
representation is the backend's job.

### The four-layer model

The compiler operates at four levels. Each layer is built on the one
below. No layer skips.

```
Surface sugar:      service, fn, type, operation    (user intent)
Composition layer:  Node, children, edges           (how things connect)
Semantic kernel:    types, effects, contracts        (what flows through nodes)
Foundation:         logical algebra                  (why it's sound)
```

**Foundation** — classical logic: truth values plus connectives (AND,
OR, NOT, IMPLIES) plus rules (associativity, commutativity, entailment).
This is the denotational ground truth. Not "a bit" (which is a carrier)
but a logical algebra over truth-valued structure. Everything else is
encoded composition.

**Semantic kernel** — types, effects, contracts: the structural algebra
that the compiler reasons about. Product (AND), Coproduct (OR), Refined
(AND with constraint), Function (IMPLIES). This is richer than raw logic
— the compiler works at this level, not at the bit level. But every
construct in the kernel is justified by the foundation.

**Composition layer** — Nodes and edges: the universal container for
connecting things. A Node composes semantic kernel objects (types, values,
effects) into a graph. The composition layer says HOW things connect.
The semantic kernel says WHAT is flowing through the connections.

**Surface sugar** — keywords and syntax: how the user expresses intent.
`service`, `fn`, `type` are ergonomic ways to say "build me a Node with
these structural properties." The sugar informs the parser what fields
to expect. It does not flow into the compiler core as identity.

### Foundational vs engineering primitives

The foundation says: everything is logic. String is a composition of
code points, which is a composition of bits.

The compiler says: String is a named semantic unit I can reason about.

Both are true. The compiler uses **engineering primitives** — named
types that it treats as units for reasoning (typechecking, inference,
emission). These are the semantic kernel. The foundation **justifies**
the kernel (String's definition is derivable from logic) but the
compiler doesn't expand String to bits at every use.

The practical rule: the compiler works at the semantic kernel level.
The foundation is the denotational story — it tells you what a type
MEANS, not how the compiler represents it. If someone asks "what IS
a String?", the answer is in the foundation (a composition of code
points). If the compiler needs to typecheck a String, it uses the
engineering primitive.

This is why `Primitive { name: "String" }` in the compiler IR is
acceptable as a scaffold — it's the engineering primitive. But it
should be traceable: there should be a .dag definition that shows
String's compositional structure, and the compiler should be able
to verify that the engineering primitive is consistent with the
definition. The name is a shorthand for the composition, not a
replacement for it.

### Worked examples: how operations fall out

These examples use real .dag files from the codebase to show how
set operations, arithmetic, service calls, and resource capabilities
emerge from logical composition rather than being added as features.

#### Type refinement as set narrowing

From `dsl/std/types.dag`:

```dag
type CommitSha   = String where pattern("^[a-f0-9]{40}$")
type NonEmptyStr = String where non_empty
type IntentId    = NonEmptyStr where brand("IntentId")
type IssueId     = NonEmptyStr where brand("IssueId")
```

Every `where` clause is AND — intersecting the base set with a
predicate. Every step narrows:

```
String          = all character sequences
NonEmptyStr     = String AND non_empty            ⊂ String
CommitSha       = String AND pattern(hex40)        ⊂ String
IntentId        = NonEmptyStr AND brand("IntentId") ⊂ NonEmptyStr ⊂ String
```

What falls out without any special machinery:

- **Subtyping is set inclusion.** `CommitSha` can be passed where
  `String` is expected because `CommitSha ⊂ String`. This is
  AND-elimination: if `x` satisfies `String AND pattern(hex40)`,
  then `x` satisfies `String`.

- **Brand creates disjointness.** `IntentId ∩ IssueId = ∅` even
  though both are subsets of `NonEmptyStr`. A value can't satisfy
  `brand("IntentId") AND brand("IssueId")` — the conjunction is
  unsatisfiable. Disjointness from AND + NOT, no special nominal
  typing rule.

- **Refinement chains are transitive.** `IntentId ⊂ NonEmptyStr ⊂ String`
  because AND is transitive in subset terms. If the inner AND
  holds, the outer AND holds.

#### filter/map/count as set operations

From `dsl/shared/dag_util.dag`:

```dag
fn aggregate_results(stages: List<StageResult>) -> Summary {
  let passed_list = filter(stages, s => s.success)
  let failed_list = filter(stages, s => !s.success && !s.skipped)
  { total: count(stages), passed: count(passed_list), failed: count(failed_list) }
}
```

Logical interpretation:

```
stages : List<StageResult>
       = recursive OR(empty, head: StageResult AND tail: List<StageResult>)

filter(stages, s => s.success)
       = { x ∈ stages | x.success = True }
       = set comprehension — the subset where the predicate holds

filter(stages, s => !s.success && !s.skipped)
       = { x ∈ stages | NOT x.success AND NOT x.skipped }
       = set comprehension with compound predicate (AND, NOT)

count(stages) = |stages| — cardinality of the set
```

The expression `!s.success && !s.skipped` IS logic — `NOT` and
`AND` applied to propositions. The `filter` function IS set
comprehension — `{ x ∈ S | P(x) }`. The `map` function IS function
image — `{ f(x) | x ∈ S }`. These aren't library functions bolted
onto the type system; they're what AND/OR/NOT mean when applied to
collections.

| Collection operation | Set operation | Logical form |
|---|---|---|
| `filter(xs, p)` | comprehension `{ x ∈ S \| P(x) }` | AND(membership, predicate) |
| `map(xs, f)` | image `{ f(x) \| x ∈ S }` | IMPLIES applied to each element |
| `count(xs)` | cardinality `\|S\|` | count of OR branches taken |
| `any(xs, p)` | existence `∃x ∈ S: P(x)` | OR over predicate results |
| `all(xs, p)` | universality `∀x ∈ S: P(x)` | AND over predicate results |
| `fold(xs, init, f)` | reduction | chained IMPLIES |
| `concat(xs, ys)` | union `S ∪ T` | OR(elements of S, elements of T) |

#### Addition as bit-level logic

From the foundation chain:

```dag
// logic.dag
type Classical = True | False

// bit.dag
type Bit = Classical where width(1)
type Byte = List<Bit> where length(8)
type Word64 = List<Byte> where length(8)

// integer.dag
type Int64 = Word64 where signed
type Int = Int64
```

When `dag_util.dag` computes `count(passed_list)`, the result is
`Int` — which is `Int64` — which is `Word64 where signed` — which
is 64 bits with two's complement interpretation.

Addition of two Int64 values is a chain of full adders:

```
full_adder(a: Bit, b: Bit, carry_in: Bit) -> { sum: Bit, carry: Bit }
  sum   = (a AND NOT b AND NOT carry_in)
       OR (NOT a AND b AND NOT carry_in)
       OR (NOT a AND NOT b AND carry_in)
       OR (a AND b AND carry_in)
  carry = (a AND b) OR (carry_in AND (a XOR b))

add(x: Int64, y: Int64) -> Int64
  = chain 64 full_adders, threading carry through each bit position
```

Addition IS a composition of AND/OR/NOT over bits. No arithmetic
primitive needed — it's derived from logic.

The compiler doesn't actually expand `+` to 64 full adders — that's
the engineering primitive (the hardware `ADD` instruction). But the
*meaning* is grounded: the foundation tells you exactly what `+`
means for `Int64` vs `Int32` vs arbitrary-precision. This is why
`Int` as an unqualified primitive is ambiguous — the bit width IS
the definition, and the foundation chain makes it explicit.

#### Service operation as conditional coproduct

From `dsl/extdeps/git.dag`:

```dag
operation CurrentBranch {
  input {}
  output { branch: String }
  readonly
  transport shell { argv: ["git", "rev-parse", "--abbrev-ref", "HEAD"] }
  exit {
    0 => Unit
    128 => String "Not a git repository"
  }
}
```

Logical interpretation:

```
CurrentBranch : Unit ->
    (exit=0   IMPLIES { branch: String })
  OR
    (exit=128 IMPLIES String)
```

The return type is a coproduct — a disjunction of possible outcomes.
In the type system this is already expressible:

```dag
type CurrentBranchResult
  = Ok { branch: String }
  | GitError { message: String }
```

What falls out:

- **The return type is OR.** The operation either succeeds with a
  branch name OR fails with an error. This is a sum type — the
  disjunction is already in the type system.

- **Exit codes are dispatch, not types.** The mapping `0 → Ok`,
  `128 → GitError` is runtime computation (which variant based on
  the exit code). The type structure (the coproduct) is logic. The
  dispatch is rendering. They live in different layers.

- **Readonly is a predicate.** `readonly` asserts that this
  operation does not modify world state. Logically: AND(the
  operation, NOT(writes)). It constrains the operation's behavior
  the same way `where non_empty` constrains a string's values.

- **Transport is grounding.** `transport shell { ... }` is what
  makes this an axiom backed by the external world rather than a
  theorem proven by a body expression. The shell command is the
  connection to reality — it's where the DAG touches something
  outside itself.

#### REST service as typed I/O contract

From `dsl/extdeps/github/gists.dag`:

```dag
service github.Gist {
  config {
    endpoint: "https://api.github.com"
    auth: BearerToken
    rate_limit: { requests: 5000, per: hour, scope: core }
    retry: { max_attempts: 3, backoff: exponential }
  }

  operation Create {
    input {
      description: String
      content: String
      public: Bool = false
      auth_token: Secret
    }
    output { id: GistId, html_url: Url }
    transport rest { method: POST, path: "/gists", body: { ... } }
    response {
      201 => Gist
      401 => GitHubErrorShape
      403 => GitHubErrorShape
    }
  }
}
```

Logical interpretation:

```
github.Gist : a bundle of grounded propositions (AND)
  - config: endpoint AND auth AND rate_limit AND retry
  - Create: one of the sub-propositions

github.Gist.Create :
  (description: String AND content: String AND public: Bool AND auth_token: Secret)
  IMPLIES
  (  (status=201 IMPLIES { id: GistId, html_url: Url })
   OR (status=401 IMPLIES GitHubErrorShape)
   OR (status=403 IMPLIES GitHubErrorShape)
  )
```

This decomposes into things the type system already has:

```dag
// The precondition (input) — a product (AND)
{ description: String, content: String, public: Bool, auth_token: Secret }

// The postcondition (output) — a coproduct (OR) of conditional outcomes
type CreateResult
  = Created { id: GistId, html_url: Url }
  | Unauthorized { error: GitHubErrorShape }
  | Forbidden { error: GitHubErrorShape }

// The full type: precondition IMPLIES postcondition
Create : CreateInput -> CreateResult
```

The service structure:

```
github.Gist (Node, connective: AND)
  ├── config (AND: endpoint AND auth AND rate_limit AND retry)
  ├── transport: RestBinding (grounding — connects to HTTP)
  └── children:
      └── Create (Node, IMPLIES: input -> conditional coproduct)
          ├── params: [description, content, public, auth_token]
          └── return_type: Created | Unauthorized | Forbidden
```

What falls out:

- **A service is AND(config, transport, children).** All parts exist
  simultaneously. The config constrains the connection. The transport
  grounds it in HTTP. The children are the available operations.

- **Each operation is IMPLIES.** Given inputs (precondition),
  produce one of the possible outcomes (postcondition as coproduct).

- **Response mappings decompose into type + dispatch.** The type
  part (coproduct of outcomes) lives in the type system. The dispatch
  part (status 201 → Created variant) lives in the rendering layer.

- **Rate limiting and retry are refinement predicates on the
  service.** `rate_limit: { requests: 5000, per: hour }` constrains
  the operation's temporal behavior — AND(the service, at most 5000
  requests per hour). `retry: { max_attempts: 3 }` constrains error
  recovery — AND(the service, retry up to 3 times on failure).

#### Resource capability as narrowed grounding

From `dsl/std/resources.dag`:

```dag
resource Filesystem {
  capability probe {
    input { path: FilePath }
    output { classification: FileClassification }
  }
  capability read {
    input { path: TextFilePath }
    output { content: String }
  }
  capability write {
    input { path: FilePath, content: String }
    output { written: Bool }
  }
}
```

And from `dsl/std/patterns.dag`:

```dag
pattern classify_files(paths: List<FilePath>)
  -> { readable: List<FilePath>, skipped: List<FilePath> }
  uses fs: Filesystem(mode: Read)
{
  entries = for path in paths { fs.probe(path: path) }
  ...
}
```

Logical interpretation:

```
Filesystem : AND(probe, read, write)
  probe : FilePath IMPLIES FileClassification        (grounded in OS)
  read  : TextFilePath IMPLIES String                (grounded in OS)
  write : (FilePath AND String) IMPLIES Bool          (grounded in OS)

TextFilePath ⊂ FilePath
  — read's precondition is narrower than probe's
  — you must prove the file is text before you can read it as text

Filesystem(mode: Read) : AND(probe, read) — NOT(write)
  — only the read-mode capabilities are available
  — the uses clause restricts which axioms the proof may invoke
```

What falls out:

- **Capabilities are IMPLIES (functions) grounded in I/O.** `probe`
  takes a `FilePath` and produces a `FileClassification`. Same
  structure as a function, different grounding — its truth comes
  from the OS rather than from a body expression.

- **The refinement TextFilePath ⊂ FilePath enforces safety.** You
  can't call `read` on an arbitrary `FilePath` — you need a
  `TextFilePath`, which is `FilePath AND content(Text)`. The subset
  relationship prevents reading binary files as text at compile time.
  This is just set narrowing — the same mechanism as
  `CommitSha ⊂ String`.

- **Mode restriction is set narrowing on capabilities.** `mode: Read`
  means AND(probe, read) — only the read capabilities are available.
  The `write` capability is excluded. This is the same AND/NOT
  mechanism that narrows types, applied to capabilities instead.

- **`uses` is a dependency declaration.** "This proof depends on
  these axioms." The function `classify_files` can't exist without
  the Filesystem axiom — its truth requires grounding that only the
  resource provides. The `uses` clause makes this dependency explicit.

- **`for path in paths { fs.probe(...) }` is function image.**
  `{ probe(p) | p ∈ paths }` — applying IMPLIES to each element
  of a set. Same as `map(paths, p => fs.probe(path: p))`. The
  `for` comprehension is set-image in surface syntax.

#### Workflow composition as chained implication

From `dsl/tools/build.dag`:

```dag
func build_all() -> { overall_success: Bool, summary: Summary, report: String } {
  build = cargo.Build.Build()
  test = cargo.Build.Test()    [after build, when build.success]
  clippy = cargo.Build.Clippy() [after build, when build.success]

  stages = [
    stage_from_output(name: "build",  success: build.success, ...),
    stage_from_output(name: "test",   success: test.success, ...),
    stage_from_output(name: "clippy", success: clippy.success, ...)
  ]
  summary = aggregate_results(stages: stages)
  report = format_report(summary: summary, stages: stages)

  return {
    overall_success: build.success && test.success && clippy.success,
    summary: summary,
    report: report
  }
}
```

Logical interpretation:

```
build_all : Unit IMPLIES { overall_success: Bool, summary: Summary, report: String }

Proof structure:
  1. build : Unit IMPLIES BuildOutput        (axiom — grounded in cargo)
  2. test  : build.success IMPLIES TestOutput (conditional — only if build proved success)
  3. clippy: build.success IMPLIES ClippyOutput (parallel conditional)
  4. stages: [build, test, clippy] — a list (recursive OR) of results
  5. summary: List<StageResult> IMPLIES Summary (function application)
  6. report: (Summary AND List<StageResult>) IMPLIES String (function application)
  7. overall_success: build.success AND test.success AND clippy.success

[after build, when build.success]:
  = this step's proof depends on build's proof
  = AND: temporal ordering (after) AND conditional (when success)
  = the "when" clause is a logical guard — the step only executes
    if the guard proposition holds
```

What falls out:

- **`[after X, when P]` is guarded implication.** "If X has been
  proven AND P holds, then proceed." This is `(X proven AND P)
  IMPLIES next_step` — chained implication with guards.

- **Parallel steps are AND.** `test` and `clippy` both depend on
  `build.success` but not on each other. They're independent
  conjuncts — both must hold for `overall_success`. The DAG
  structure captures the parallelism naturally: no ordering edge
  between test and clippy means they can execute simultaneously.

- **`&&` in `overall_success` IS conjunction.** `build.success AND
  test.success AND clippy.success` — the overall proof requires all
  three sub-proofs. This is AND at the value level, using the same
  connective as AND at the type level (products) and AND at the
  composition level (services with multiple operations).

- **The workflow is a proof tree.** Each step either produces a
  witness (the output value) or fails. The final result is a
  conjunction of all the witnesses. The DAG edges are the dependency
  structure of the proof — you can't use `test.success` until
  `test` has produced its witness.

#### The foundation chain as a complete derivation

Tracing `CommitSha` from actual .dag files back to the axiom:

```
Classical = True | False                            logic.dag     (axiom)
Bit = Classical where width(1)                      bit.dag       (AND)
Byte = List<Bit> where length(8)                    bit.dag       (AND)
Word64 = List<Byte> where length(8)                 bit.dag       (AND)
Int64 = Word64 where signed                         integer.dag   (AND)
Int = Int64                                         integer.dag   (alias)
String = { bytes: List<Byte>, encoding: Encoding }  string_type.dag (AND)
CommitSha = String where pattern("^[a-f0-9]{40}$") types.dag     (AND)
```

Every step is AND (adding a constraint) or OR (offering alternatives).
Each `where` clause is a conjunction with a predicate. Each `=` is a
definition. The chain is traceable to `Classical = True | False`.

Operations on CommitSha:

```
filter(commits, c => starts_with(c.sha, "abc"))
  = set comprehension { x ∈ commits | starts_with(x.sha, "abc") }
  = AND(membership-in-commits, predicate)

count(commits) = |commits| — cardinality

map(commits, c => c.sha) = { sha(x) | x ∈ commits } — function image

sha1 == sha2 = bitwise comparison
  = AND over all bit positions: bit_i(sha1) = bit_i(sha2)
  = AND of 320 bit equalities (40 hex chars × 4 bits × 2 for equality check)
```

The logical foundation generates these operations — they're
compositions of AND/OR/NOT over the same axiom.

### Worked examples: how test generation falls out

In circuit design (PSPICE), the schematic IS the test specification.
You don't write tests separately — the testbench is derived from the
circuit model. Synthesis produces the circuit. Testbenching stress-tests
it against the spec. The designer writes one artifact; two things
fall out.

The same principle applies here. A .dag file contains:

- Type constraints (what values are valid)
- Response mappings (what outcomes are possible)
- Error codes (what can go wrong)
- Rate limits (how fast you can go)
- Retry policies (how to recover)
- Mock fixtures (example valid responses)
- Dependency structure (what depends on what)

Every one of these is a testable proposition. The test generator
reads the model and derives: what inputs to try, what outputs to
expect, what error paths to exercise, what boundary conditions to
probe. No separate test specification needed.

#### Type constraints generate test values

From `dsl/std/types.dag`:

```dag
type CommitSha    = String where pattern("^[a-f0-9]{40}$")
type RetryCount   = Int where range(min: 1, max: 5)
type HttpStatus   = Int where range(min: 100, max: 599)
type PositiveInt  = Int where range(min: 1)
type NonEmptyStr  = String where non_empty
```

Each refinement predicate generates test values at the boundaries:

```
CommitSha:
  valid:    "a" * 40, "f" * 40, "0123456789abcdef" * 2 + "01234567"
  invalid:  "" (too short), "g" * 40 (wrong chars), "a" * 39 (too short),
            "a" * 41 (too long), "A" * 40 (uppercase)

RetryCount:
  valid:    1 (min), 3 (mid), 5 (max)
  invalid:  0 (below min), 6 (above max), -1 (negative)

HttpStatus:
  valid:    100 (min), 200, 301, 404, 500, 599 (max)
  invalid:  99 (below), 600 (above), 0, -1

NonEmptyStr:
  valid:    "a", "hello", " " (whitespace counts)
  invalid:  "" (empty)
```

The generation rule for each predicate type:

| Predicate | Valid values | Invalid values |
|---|---|---|
| `range(min, max)` | min, max, midpoint | min-1, max+1, 0, negative |
| `pattern(regex)` | strings matching regex | empty, wrong chars, wrong length |
| `non_empty` | "a", "test" | "" |
| `brand(name)` | valid base type values | values of different brand |

This is mechanical. The predicate IS the test spec.

#### Product types generate field-combination tests

From `dsl/std/types.dag`:

```dag
type AccessToken {
  token: Secret
  scheme: AuthScheme
  expires_at: Timestamp?
}

type AuthScheme = Bearer | Header { name: String } | Basic { username: String }
```

A product (AND) requires all fields. Test generation:

```
AccessToken tests:
  // All fields valid (happy path)
  { token: "sk-test", scheme: Bearer, expires_at: "2025-01-01T00:00:00" }

  // Each optional field absent
  { token: "sk-test", scheme: Bearer, expires_at: none }

  // Each sum-type field with each variant
  { token: "sk-test", scheme: Bearer, expires_at: ... }
  { token: "sk-test", scheme: Header { name: "X-Api-Key" }, expires_at: ... }
  { token: "sk-test", scheme: Basic { username: "admin" }, expires_at: ... }

  // Each required field invalid (expect rejection)
  { token: "", scheme: Bearer, ... }  // empty Secret
```

The rule: for a product of N fields, generate one test with all
fields valid, one test per optional field absent, one test per
sum-type variant, one test per field with an invalid value. This is
AND-decomposition — testing each conjunct independently.

For a coproduct (OR), generate one test per variant:

```
AuthScheme tests:
  Bearer                         // variant 1
  Header { name: "X-Api-Key" }  // variant 2, with valid field
  Header { name: "" }           // variant 2, with invalid field
  Basic { username: "admin" }   // variant 3, with valid field
```

This is OR-decomposition — testing each disjunct. The coproduct
tells you exactly how many cases to cover.

#### Operation contracts generate integration tests

From `dsl/extdeps/git.dag`:

```dag
operation CurrentBranch {
  input {}
  output { branch: String }
  readonly
  transport shell { argv: ["git", "rev-parse", "--abbrev-ref", "HEAD"] }
  exit {
    0 => Unit
    128 => String "Not a git repository"
  }
}
```

The contract defines all possible outcomes. Each is a test case:

```
Test 1 — happy path (exit 0):
  precondition: inside a git repository
  execute: git rev-parse --abbrev-ref HEAD
  assert: exit code = 0
  assert: output.branch is a non-empty string
  assert: output.branch matches known branch format

Test 2 — error path (exit 128):
  precondition: NOT inside a git repository (e.g., /tmp)
  execute: git rev-parse --abbrev-ref HEAD
  assert: exit code = 128
  assert: error message contains "Not a git repository"
```

One test per exit code. The exit mapping IS the test matrix.

For the readonly modifier:

```
Test 3 — readonly assertion:
  precondition: known repository state (files, branches)
  execute: CurrentBranch
  assert: repository state is unchanged after execution
  (no files modified, no refs changed, no staging area changes)
```

The modifier IS the assertion. `readonly` means "the world state
before and after must be identical." That's a testable proposition.

#### Response mappings generate API test matrices

From `dsl/extdeps/github/gists.dag`:

```dag
operation Create {
  input {
    description: String
    content: String
    public: Bool = false
    auth_token: Secret
  }
  output { id: GistId, html_url: Url }
  transport rest { method: POST, path: "/gists", body: { ... } }
  response {
    201 => Gist
    401 => GitHubErrorShape
    403 => GitHubErrorShape
  }
  mock_response {
    status: 201
    body: { id: "abc123", html_url: "https://gist.github.com/abc123" }
  }
}
```

Three response codes → three test tiers:

```
Tier 1 — Mock tests (no network, from mock_response):
  input: { description: "test", content: "hello", auth_token: "token" }
  mock: { status: 201, body: { id: "abc123", html_url: "..." } }
  assert: result matches Gist type
  assert: result.id matches GistId constraints
  assert: result.html_url matches Url constraints

Tier 2 — Contract tests (per response code):
  Test 2a (201 Created):
    input: valid description, content, auth_token
    assert: response body deserializes to Gist
    assert: Gist.id is GistId (non-empty, matches pattern)
    assert: Gist.html_url is Url (matches URL pattern)

  Test 2b (401 Unauthorized):
    input: valid description, content, INVALID auth_token
    assert: response body deserializes to GitHubErrorShape
    assert: GitHubErrorShape.message is non-empty

  Test 2c (403 Forbidden):
    input: valid everything but insufficient scopes
    assert: response body deserializes to GitHubErrorShape

Tier 3 — Real execution (with actual GitHub API):
    input: valid description, content, REAL auth_token
    assert: gist created, response matches contract
    cleanup: delete the created gist
```

The response mapping IS the test matrix. Each status code is a test
case. The mock_response IS the fixture. The output type constraints
ARE the assertions.

#### Rate limits and retry generate stress tests

From `dsl/extdeps/github/gists.dag`:

```dag
config {
  rate_limit: { requests: 5000, per: hour, scope: core }
  retry: { max_attempts: 3, backoff: exponential, retry_on: [429, 500, 502, 503, 504] }
}
```

Rate limit test:

```
Test — rate limit boundary:
  execute: N requests in rapid succession (where N approaches limit)
  assert: first N requests succeed (status != 429)
  assert: request N+1 returns 429 (Too Many Requests)
  assert: retry-after header is present
  assert: system respects retry-after before continuing

  // The rate_limit spec tells us exactly:
  //   - how many requests to try (5000)
  //   - the time window (1 hour)
  //   - what scope is limited (core)
```

Retry test:

```
Test — retry on each retryable status:
  for each status in [429, 500, 502, 503, 504]:
    simulate: server returns {status} on first call
    assert: client retries (up to 3 attempts)
    assert: backoff is exponential (delay doubles)
    simulate: server returns 201 on retry
    assert: final result is success

Test — retry exhaustion:
  simulate: server returns 500 on all 3 attempts
  assert: after 3 attempts, error is propagated (not silently swallowed)
  assert: error contains enough context to diagnose

Test — non-retryable status:
  for each status NOT in [429, 500, 502, 503, 504]:
    simulate: server returns {status}
    assert: NO retry (immediate failure)
    // e.g., 401 should fail immediately, not retry 3 times
```

The retry spec IS the test spec. `retry_on: [429, 500, 502, 503, 504]`
tells you exactly which codes to test retry behavior for AND which
codes to test immediate-failure behavior for (everything NOT in
the list).

#### Workflow structure generates integration tests

From `dsl/tools/build.dag`:

```dag
func build_all() -> { overall_success: Bool, summary: Summary, report: String } {
  build  = cargo.Build.Build()
  test   = cargo.Build.Test()    [after build, when build.success]
  clippy = cargo.Build.Clippy()  [after build, when build.success]
  ...
  return { overall_success: build.success && test.success && clippy.success, ... }
}
```

The dependency graph IS the test structure:

```
Test — all succeed:
  mock: build succeeds, test succeeds, clippy succeeds
  assert: overall_success = true
  assert: summary.passed = 3, summary.failed = 0

Test — build fails (guards block downstream):
  mock: build fails
  assert: test is NOT executed (guarded by build.success)
  assert: clippy is NOT executed (guarded by build.success)
  assert: overall_success = false
  assert: summary.failed >= 1

Test — build succeeds, test fails:
  mock: build succeeds, test fails, clippy succeeds
  assert: test executed (guard satisfied)
  assert: clippy executed (independent of test)
  assert: overall_success = false (because test.success = false)
  assert: summary.passed = 2, summary.failed = 1

Test — build succeeds, clippy fails:
  mock: build succeeds, test succeeds, clippy fails
  assert: overall_success = false
  assert: summary.passed = 2, summary.failed = 1
```

The `[after X, when Y]` annotations tell you:
- What to mock (the dependency)
- What guard to test both ways (when=true, when=false)
- What to assert about downstream effects (skipped vs executed)

For N steps with M guards, the test matrix is derived mechanically.

#### Composition: testing a service tests its types

The test tiers compose the same way the model composes:

```
Level 1 — Type tests (innermost):
  Generate valid/invalid values for each refined type.
  GistId satisfies its pattern? CommitSha is 40 hex chars?
  Boundary values for RetryCount, HttpStatus, etc.

Level 2 — Operation tests (contract level):
  For each operation: one test per response code.
  Inputs generated from Level 1 (valid GistId, valid CommitSha).
  Outputs validated against Level 1 (response body matches type).

Level 3 — Service tests (integration level):
  Mock the transport layer, replay mock_response fixtures.
  Test rate limit boundaries.
  Test retry behavior for each retryable status code.

Level 4 — Workflow tests (end-to-end):
  Mock each service call at Level 3.
  Test dependency graph: each guard true/false.
  Test parallel independence: order shouldn't matter.
  Test error propagation: failure at step N → correct skip at step N+1.

Level 5 — Real execution (system test):
  Replace mocks with real I/O.
  Verify the actual system behaves as the contract claims.
  Cleanup after mutating operations (delete created gists, etc.).
```

Each level tests the propositions at that layer. Level 1 tests
type predicates (AND constraints). Level 2 tests operation
contracts (IMPLIES + OR of outcomes). Level 3 tests service
properties (rate limits, retry). Level 4 tests workflow logic
(guarded implication chains). Level 5 validates grounding (do the
axioms actually hold in the real world?).

This is the PSPICE analogy:
- Level 1 = component characterization (does this resistor match spec?)
- Level 2 = subcircuit verification (does this amplifier stage work?)
- Level 3 = block-level simulation (does the power supply regulate?)
- Level 4 = system simulation (does the full board work?)
- Level 5 = hardware test (does the physical board match simulation?)

#### What the test generator needs from the model

No additional test specification is needed. The model already contains:

| Model element | What it generates | Test tier |
|---|---|---|
| `where pattern(...)` | valid + invalid strings | Type |
| `where range(min, max)` | boundary values: min, max, min-1, max+1 | Type |
| `where non_empty` | valid: "a", invalid: "" | Type |
| `where brand(...)` | nominal distinctness checks | Type |
| Product `{ fields }` | one test per field valid/invalid | Type |
| Coproduct `A \| B \| C` | one test per variant | Type |
| Optional `T?` | test with Some, test with None | Type |
| `response { 201 => T, 401 => E }` | one test per status code | Operation |
| `exit { 0 => T, 128 => E }` | one test per exit code | Operation |
| `mock_response { ... }` | fixture for mock-level test | Operation |
| `readonly` / `idempotent` | state-preservation assertion | Operation |
| `rate_limit: { N, per: T }` | N+1 request boundary test | Service |
| `retry: { on: [...] }` | retry/no-retry per status code | Service |
| `[after X, when P]` | guard true + guard false paths | Workflow |
| `[after X]` (no when) | ordering dependency | Workflow |

The test generator walks the DAG model, reads these elements, and
emits test code at each tier. The propositions in the model ARE the
assertions in the tests. Writing a .dag file simultaneously writes
its test specification.

### Set operations as compositions on truth

A set is a proposition: `x ∈ S` means "the proposition S(x) holds."
A set IS its characteristic function — a function from values to
truth values.

```
S = { x | P(x) }     where P : T -> Classical
                      P(x) = True means x is in the set
                      P(x) = False means x is not
```

Every type in the system is a set: the set of values that satisfy
the type's proposition. `String` is `{ x | x is a valid character
sequence }`. `CommitSha` is `{ x | x ∈ String AND matches(x, hex40) }`.

The set operations are direct compositions of AND/OR/NOT:

```
Union          A ∪ B  = { x | A(x) OR  B(x) }       Coproduct
Intersection   A ∩ B  = { x | A(x) AND B(x) }       Product / Refined
Complement     Aᶜ     = { x | NOT A(x) }             Negation
Difference     A \ B  = { x | A(x) AND NOT B(x) }    Refined with negated pred
Subset         A ⊆ B  ≡ ∀x: A(x) IMPLIES B(x)       Subtyping
Empty set      ∅      = { x | False }                 Void / Never (no inhabitants)
Universal set  U      = { x | True }                  Top type (all values)
Membership     x ∈ S  = S(x) = True                   Type judgment (x : S)
Cardinality    |S|    = count of x where S(x) = True  Count
```

No new primitives. These are AND, OR, NOT, IMPLIES applied to
propositions about values. The set operations are already in the
logical foundation.

#### Collection operations are sugar on set operations

`filter`, `map`, `fold`, `any`, `all` are the surface syntax for
set-theoretic operations:

```
filter(S, P)     = S ∩ { x | P(x) }                  intersection
                 = { x ∈ S | P(x) }                   comprehension

map(S, f)        = { f(x) | x ∈ S }                   function image

fold(S, init, f) = f(... f(f(init, s₁), s₂) ..., sₙ) chained IMPLIES

any(S, P)        = ∃x ∈ S: P(x)                       existential (big OR)
                 = OR over { P(x) | x ∈ S }

all(S, P)        = ∀x ∈ S: P(x)                       universal (big AND)
                 = AND over { P(x) | x ∈ S }

count(S)         = |S|                                  cardinality

concat(S, T)     = S ∪ T                               union
                 = { x | x ∈ S OR x ∈ T }

flat_map(S, f)   = ⋃ { f(x) | x ∈ S }                 union of images

first(S)         = min element (by position)            selection

unique(S)        = the underlying set of S              idempotent membership
                   (multiset → set)
```

These aren't library functions bolted onto the language. They are
what AND, OR, NOT, IMPLIES mean when applied to collections.

#### Grounded example: aggregate_results

From `dsl/shared/dag_util.dag`:

```dag
fn aggregate_results(stages: List<StageResult>) -> Summary {
  let passed_list = filter(stages, s => s.success)
  let failed_list = filter(stages, s => !s.success && !s.skipped)
  { total: count(stages), passed: count(passed_list), failed: count(failed_list) }
}
```

Set-theoretic reading:

```
stages       = S                            (the full set)
passed_list  = S ∩ { s | s.success }        (intersection with predicate)
failed_list  = S ∩ { s | NOT s.success AND NOT s.skipped }
                                             (intersection with compound predicate)
total        = |S|                           (cardinality of full set)
passed       = |S ∩ Success|                 (cardinality of intersection)
failed       = |S ∩ (NOT Success AND NOT Skipped)|
```

The test properties fall out of the set-theoretic definitions:

```
Property 1 (intersection subset):
  passed_list ⊆ stages                     -- every passed stage was in the input
  failed_list ⊆ stages                     -- every failed stage was in the input

Property 2 (predicate satisfaction):
  ∀s ∈ passed_list: s.success = True       -- every element satisfies the predicate
  ∀s ∈ failed_list: NOT s.success AND NOT s.skipped

Property 3 (completeness):
  ∀s ∈ stages: s.success → s ∈ passed_list -- no qualifying element was missed
  ∀s ∈ stages: (NOT s.success AND NOT s.skipped) → s ∈ failed_list

Property 4 (cardinality conservation):
  passed + failed + skipped = total        -- partition (no double-counting)
  where skipped = |S ∩ { s | NOT s.success AND s.skipped }|

Property 5 (empty set base case):
  aggregate_results([]) = { total: 0, passed: 0, failed: 0 }
```

These five properties are derivable from the definition. The test
generator reads `filter(stages, s => s.success)`, sees it's an
intersection, and emits assertions for subset, satisfaction,
completeness, and partition.

Generated test inputs:

```
// From StageResult's type structure (AND of fields):
Test 1 — all succeed:
  stages = [{ success: true, skipped: false }, { success: true, skipped: false }]
  assert: passed = 2, failed = 0

Test 2 — all fail:
  stages = [{ success: false, skipped: false }, { success: false, skipped: false }]
  assert: passed = 0, failed = 2

Test 3 — mixed with skips:
  stages = [{ success: true, skipped: false },
            { success: false, skipped: false },
            { success: false, skipped: true }]
  assert: passed = 1, failed = 1, total = 3
  assert: passed + failed = 2 (not 3 — skipped doesn't count as failed)

Test 4 — empty input:
  stages = []
  assert: passed = 0, failed = 0, total = 0
```

The test inputs come from the type structure (StageResult has Bool
fields → test both True and False for each) combined with the
collection structure (test empty, one element, multiple elements).

#### Grounded example: service registry construction

From `src/v2/04_typecheck.dag`:

```dag
let local_svc_registry = fold(resolved_items, init: empty_map(), f: (acc, item) =>
  if item.transport != none && item.children |> count > 0 {
    let entries = item.children
      |> filter(c => c.port_contract != none)
      |> map(c => match c.port_contract.value {
        OperationContract { outputs: outs, ... } => OpEntry { name: c.name, outputs: outs }
        CapabilityContract { outputs: outs } => OpEntry { name: c.name, outputs: outs }
      })
    map_insert(acc, item.name, entries)
  } else {
    acc
  }
)
```

Set-theoretic reading:

```
fold(items, ∅, f)
  = items iterated, accumulating a map

  for each item:
    guard: item.transport ≠ ∅ AND |item.children| > 0
    (only items that are services — have transport AND children)

    entries = item.children
      |> filter(port_contract ≠ ∅)         -- intersection with "has contract"
      |> map(c => OpEntry from c)           -- function image (projection)

    acc ∪ { item.name → entries }           -- map extension (union with singleton)
```

This is: for each service (AND(transport, children)), extract its
operation entries (intersection then image), and build a lookup
table (fold as iterated union).

Test properties:

```
Property 1 (guard correctness):
  ∀name ∈ registry: the item with that name has transport AND children
  (only services are in the registry)

Property 2 (entry extraction):
  ∀(name, entries) ∈ registry:
    ∀entry ∈ entries: ∃child ∈ item.children:
      child.port_contract ≠ none AND entry.name = child.name

Property 3 (completeness):
  ∀item with transport AND children: item.name ∈ registry
  (no service was missed)

Property 4 (map semantics):
  registry with 0 service items = empty_map()
  registry with 1 service item = { item.name → entries }
```

Generated test inputs:

```
Test 1 — no services:
  items = [type_node, fn_node]  (no transport, no children)
  assert: registry = empty_map()

Test 2 — one service, one operation:
  items = [service_node { transport: shell, children: [op_with_contract] }]
  assert: registry has 1 entry
  assert: entry has 1 OpEntry

Test 3 — service with mixed children:
  items = [service_node { children: [op_with_contract, child_without_contract] }]
  assert: entry has 1 OpEntry (filtered out the child without contract)

Test 4 — multiple services:
  items = [service_a { 2 ops }, service_b { 1 op }]
  assert: registry has 2 entries
  assert: entry_a has 2 OpEntries, entry_b has 1
```

#### Grounded example: unique_strings as set conversion

From `src/v2/05_emit.dag`:

```dag
fn unique_strings(items: List<String>) -> List<String> {
  items |> fold(init: [], f: fn(acc, item) {
    if acc |> any(s => s == item) {
      acc
    } else {
      concat(acc, [item])
    }
  })
}
```

Set-theoretic reading:

```
unique_strings = multiset → set
               = for each element, check membership (any/∃),
                 if not present add (union with singleton)

  any(acc, s => s == item)    = item ∈ acc            membership test
  concat(acc, [item])         = acc ∪ {item}          union with singleton
  the fold                    = ⋃ { {x} | x ∈ items } idempotent union
```

This is literally "convert a bag to a set by iterative union with
membership check." The test properties:

```
Property 1 (idempotence):
  unique_strings(unique_strings(xs)) = unique_strings(xs)
  (applying unique twice is the same as once — set is already a set)

Property 2 (subset):
  unique_strings(xs) ⊆ xs
  (every output element was in the input)

Property 3 (completeness):
  ∀x ∈ xs: x ∈ unique_strings(xs)
  (every input element appears in the output)

Property 4 (no duplicates):
  ∀i,j: i ≠ j → unique_strings(xs)[i] ≠ unique_strings(xs)[j]

Property 5 (cardinality):
  |unique_strings(xs)| ≤ |xs|

Property 6 (empty base):
  unique_strings([]) = []
```

These are the standard set axioms. The test generator doesn't need
to know this is a "dedup function" — it reads the structure (fold
with any + concat) and emits the algebraic properties of the
set operations involved.

#### Grounded example: flat_map as union of images

From `src/v2/04_typecheck.dag`:

```dag
let arg_diags = flat_map(arg_results, r => r.diagnostics)
```

Set-theoretic reading:

```
flat_map(S, f) = ⋃ { f(x) | x ∈ S }         union of images

arg_diags = ⋃ { r.diagnostics | r ∈ arg_results }
          = all diagnostics from all results, unioned together
```

This is the union of a family of sets indexed by the results. The
test properties:

```
Property 1 (union containment):
  ∀r ∈ arg_results: r.diagnostics ⊆ arg_diags
  (every individual diagnostic list is contained in the result)

Property 2 (nothing extra):
  ∀d ∈ arg_diags: ∃r ∈ arg_results: d ∈ r.diagnostics
  (every output diagnostic came from some input result)

Property 3 (empty preservation):
  flat_map([], f) = []
  (union over empty family is empty set)

Property 4 (single element):
  flat_map([r], f) = f(r)
  (union over singleton is just the image)
```

Generated tests:

```
Test 1 — all results have empty diagnostics:
  arg_results = [{ diagnostics: [] }, { diagnostics: [] }]
  assert: arg_diags = []

Test 2 — one result has diagnostics:
  arg_results = [{ diagnostics: [diag1] }, { diagnostics: [] }]
  assert: arg_diags = [diag1]

Test 3 — multiple results with diagnostics:
  arg_results = [{ diagnostics: [d1, d2] }, { diagnostics: [d3] }]
  assert: arg_diags = [d1, d2, d3]
  assert: |arg_diags| = 3
```

#### Grounded example: chained filter-map-count

From `src/v2/05_emit_rust.dag` (service test generation):

```dag
fn emit_typed_service_tests(item: TypedNode, registry: Map<String, ItemInfo>) -> List<String> {
  if item.children |> count > 0 {
    let safe_name = sanitize_service_name(name: item.name)
    item.children
      |> filter(c => c.port_contract != none)
      |> filter(c => match c.port_contract.value {
        OperationContract { outputs: _, response: _, mock_response: mr, ... } => mr |> count > 0
        CapabilityContract { outputs: _ } => false
      })
      |> map(c => emit_operation_test(service_name: safe_name, op_node: c, registry: registry))
  } else { [] }
}
```

Set-theoretic reading:

```
Step 1: item.children                       = S (all children)
Step 2: filter(port_contract != none)       = S ∩ HasContract
Step 3: filter(has OperationContract with   = S ∩ HasContract ∩ HasMocks
         non-empty mock_response)
Step 4: map(emit_operation_test)            = { emit_test(c) | c ∈ S ∩ HasContract ∩ HasMocks }
```

This is: start with the full set of children, intersect with
"has a contract," intersect with "has mock responses," then apply
the test-emission function to each. Three set operations chained.

Test properties of the chain:

```
Property 1 (monotone narrowing):
  |step 4 result| ≤ |step 3 result| ≤ |step 2 result| ≤ |step 1 result|
  each filter can only shrink or preserve, never grow

Property 2 (predicate conjunction):
  ∀c in final result: c.port_contract != none
                   AND c.port_contract is OperationContract
                   AND c.port_contract.mock_response |> count > 0
  (all three predicates hold simultaneously — AND of filters is AND of predicates)

Property 3 (no operation without mocks):
  operations with empty mock_response are excluded
  (the second filter's job)

Property 4 (no capabilities):
  CapabilityContract children are excluded by the second filter
  (returns false for that match arm)
```

Generated tests:

```
Test 1 — service with no mocked operations:
  children = [op_node { mock_response: [] }]
  assert: result = [] (filtered out — no mocks)

Test 2 — service with one mocked operation:
  children = [op_node { mock_response: [mock1] }]
  assert: result = [one test string]

Test 3 — service with mixed operations:
  children = [op_with_mocks, op_without_mocks, capability_node]
  assert: |result| = 1 (only the mocked operation)

Test 4 — service with no children:
  children = []
  assert: result = [] (early return from count check)
```

#### The derivation rule

For any expression using collection operations, the test generator:

1. **Identifies the set operation.** `filter` → intersection.
   `map` → image. `fold` → iterated operation. `any` → existence.

2. **Reads the predicate/function.** `s => s.success` is the
   characteristic function of the "success" subset.

3. **Emits algebraic properties.** Each operation has standard
   properties (subset, satisfaction, completeness, cardinality
   bounds). These are the test assertions.

4. **Generates inputs from the element type.** StageResult has
   Bool fields → test True and False. List → test empty, one,
   many. Optional → test Some and None.

5. **Combines via chain composition.** A chain of operations
   (filter then map then count) produces a chain of properties.
   The properties of the composition follow from the properties
   of each step.

| Operation | Standard properties (auto-generated assertions) |
|---|---|
| `filter(S, P)` | result ⊆ S, ∀x∈result: P(x), ∀x∈S: P(x)→x∈result |
| `map(S, f)` | \|result\| = \|S\|, ∀i: result[i] = f(S[i]) |
| `fold(S, init, f)` | fold([], init, f) = init, fold([x], init, f) = f(init, x) |
| `any(S, P)` | result ↔ ∃x∈S: P(x), any([], P) = false |
| `all(S, P)` | result ↔ ∀x∈S: P(x), all([], P) = true |
| `count(S)` | result ≥ 0, count([]) = 0 |
| `concat(S, T)` | result ⊇ S, result ⊇ T, \|result\| = \|S\|+\|T\| |
| `flat_map(S, f)` | ∀x∈S: f(x) ⊆ result, ∀y∈result: ∃x∈S: y∈f(x) |
| `unique(S)` | result ⊆ S, ∀x∈S: x∈result, no duplicates |
| chain `f∘g` | properties of f AND properties of g AND monotonicity |

This table is the test generation kernel. Every use of a collection
operation in any .dag file produces tests by looking up the operation
in this table and instantiating the properties with the specific
predicate/function/types from the call site.

### Abstraction as surface choice

The core is fixed: AND, OR, NOT, IMPLIES, composition, grounding.
The surface is a choice. Different communities can work at whatever
level of abstraction they find appropriate, and it all compiles
down to the same logical structure.

```
Surface                  Abstraction level        Compiles to
───────────────────     ─────────────────────    ──────────────────
∀x ∈ S: P(x)            set theory / functions   AND over predicates
all(items, p => valid)   developer / collections  fold with AND
type T = A | B           developer / types        OR of variants
service git.Core { }     developer / services     AND(transport, children)
pipeline build { }       domain / orchestration   chained IMPLIES with guards
drag-and-drop graph      visual / no code         Node + edges
```

The compiler doesn't care which surface produced a Node. It sees
the logical structure. This means:

- **Mathematicians** can operate at set theory / function level.
  `{ x ∈ S | P(x) }` is `filter(S, P)` is `S ∩ P` is AND. They
  work with the foundation directly.

- **Developers** can work with types, functions, services, resources.
  `type`, `fn`, `service` are ergonomic keywords that produce Nodes
  with specific structural properties. They work with the surface.

- **Domain experts** can define their own abstractions. A finance
  team defines `ledger`, `transaction`, `settlement` keywords that
  produce Nodes with domain-appropriate constraints. New surface,
  same core.

- **Visual builders** can compose graphs without text. A node editor
  that connects boxes with wires produces the same Node + children +
  connective structure. Different surface, same compiler.

The key: no level is more "real" than another. `service git.Core`
and `AND(transport, children)` are the same proposition. The surface
determines ergonomics. The core determines semantics. Adding a new
abstraction level means writing a new surface (parser), not changing
the compiler.

This is the same architecture as hardware:
- A physicist models transistor characteristics (foundation)
- A digital designer works in gates and flip-flops (semantic kernel)
- A system architect draws block diagrams (composition)
- An FPGA user configures in a GUI (surface sugar)

All synthesize to the same silicon. The abstraction level is a
human choice. The physics is fixed.

### What qualifies as a shared fact

- Mathematical definitions (classical logic, set theory)
- Hardware standards (IEC 80000-13 for byte = 8 bits)
- IEEE specifications (754 for floating-point)
- Unicode Standard (code point ranges, display widths)
- Protocol specifications (RFC 6750 for Bearer tokens, RFC 6749 for OAuth 2.0)
- API documentation (Anthropic Messages API, GitHub REST API, GCP IAM)
- Language references (Rust Reference for comment syntax, Go spec for naming)

### What does NOT qualify

- Invented taxonomies (e.g., "PrerequisiteKind = Capability | Credential | ...")
- Canonicalizations across providers (e.g., a unified "AuthError" that no real API returns)
- Abstractions that don't map to any real system's API
- Policy decisions (e.g., "timeout = 30 seconds")
- The modeler's interpretation of how things should be organized

### Objective relationships

The compositional stacking between types must itself be factual:

```
logic.dag:       Classical = True | False           ← bivalent logic (math)
    ↓
bit.dag:         Bit = Classical where width(1)     ← definitional
                 Byte = List<Bit> where length(8)   ← IEC 80000-13
    ↓
integer.dag:     Int64 = Word64 where signed        ← two's complement
    ↓
float.dag:       Float64 = Word64 where ieee754     ← IEEE 754
    ↓
string_type.dag: String = { bytes, encoding }       ← definitional
                 Char = Int where range(0, 1114111)  ← Unicode scalar range
    ↓
unicode.dag:     block ranges from Unicode Standard  ← Unicode 15.0
```

Each relationship is a fact, not a design choice. "A byte IS 8 bits"
is IEC 80000-13. "IEEE 754 binary64 IS a 64-bit word" is the spec.
The relationship itself is non-controversial.

Cross-domain relationships follow the same rule. GitHub has a branching
concept that IS Git's branching model — that's documented in GitHub's
own docs. So `github.dag` should reference types from `git.dag` where
the relationship is real. The test: can you cite the documentation that
establishes the relationship?

### Layering

**Foundation (`std/`):** Shared facts only. Standards, specifications,
mathematical definitions. No policy, no preference. This is already
strong: `logic → bit → integer → float → string → unicode → filesystem`.

**External dependencies (`extdeps/`):** Spec-grounded models of real
systems. Each type comes from actual API documentation. Shared concepts
across providers (like `Role` in LLM APIs) are valid when both providers
independently document the same concept. Reference the documentation.

**Application layer:** Policy, calibration, team decisions. Legitimate
but clearly separated from factual layers. Deferred until the foundation
is solid.

---

## Principles

### M1: Types are compositional facts

A type decomposes into smaller types that each assert one fact.
Products combine independent facts. Coproducts enumerate mutually
exclusive alternatives. Containers express cardinality.

### M2: No duplicate type authorities

Every type is defined in exactly one file. If two files define the same
concept, they will diverge. Changing a fact should require editing one file.

### M3: Extdeps model specs, not abstractions

Every `dsl/extdeps/` module models a **real external system** from its
actual API documentation. Real names, real endpoints, real field shapes.
If you can't link to a spec, you're inventing an abstraction.

### M4: Closed sets are enums, not strings

When a field's values are drawn from a fixed set known at design time,
model it as a sum type. Strings are for genuinely open-ended data.

### M5: Silence is fabrication

A lookup that returns a default on miss is a fabrication fallback.
Missing data should propagate as `None` or produce a diagnostic.

### M6: One result pattern, not N result types

Parser and typechecker result types should follow a single generic
pattern. 42 bespoke result types is a maintenance multiplier.

### M7: Data tables are single-authority

When a fact exists as both a `data` declaration and a function body,
delete the function and derive from the data.

### M8: Predicates and dispatch are structural

Pattern matching should operate on type structure, not string extraction.

### M9: DFS the ontology — every construct attaches to first principles

The `std/` library is an **ontology** — a connected DAG of concepts
rooted in first-principles logic. Not just algebra (structures with
operations), but the complete inventory of what exists and how things
relate: logic, construction, algebra, iteration, termination,
discrimination, coercion. Every concept in the codebase traces back
through this ontology to `Classical` (True/False).

```
Classical (logic.dag)
├── Bit → Word8..Word64 (bit.dag)
│   ├── Int = Word64 + OrderedRing witness (integer.dag)
│   ├── Float = Word64 + ApproximateField witness (float.dag)
│   └── Bool = Classical itself
├── Product / Coproduct (constructors.dag)
│   ├── Node = recursive Product with Coproduct discriminant (00_core.dag)
│   └── every .dag type
├── Monoid → Semiring → Ring → Field (algebra.dag)
│   ├── FreeMonoid<T> → List, String (algebra.dag, string_type.dag)
│   ├── PartialFunction<K,V> → Map (algebra.dag)
│   ├── BooleanAlgebra<T> → Set (algebra.dag)
│   └── Lattice → BoundedLattice (algebra.dag)
│       └── DescentEvidence = BoundedLattice (termination.dag)
├── fold / descend / repeat (iteration.dag)
│   └── every loop, every recursion
└── Ordering = Less | Equal | Greater (algebra.dag)
    └── well-founded orderings → termination proofs (termination.dag)
```

**The methodology:** when implementing or changing code, think in terms
of DFS through the ontology. Start from the concept you need, walk DOWN
to its root. The root tells you what the concept ACTUALLY IS. Then walk
back UP from the closest existing concept in `std/` to find your
attachment point. The ontology has branches beyond algebra:

| Branch | Root concept | std/ file | What it covers |
|--------|-------------|-----------|----------------|
| **Logic** | Truth/Falseness | `logic.dag` | Propositions, connectives, entailment |
| **Construction** | Product/Coproduct | `constructors.dag` | Type forming, records, enums |
| **Algebra** | Monoid → Ring → Field | `algebra.dag` | Operations that emerge from structure |
| **Iteration** | fold/descend/repeat | `iteration.dag` | All bounded computation |
| **Termination** | Well-founded orderings | `termination.dag` | Proof that computation halts |
| **Observation** | Pattern discrimination | (needs `discrimination.dag`) | Matching, case analysis |
| **Coercion** | Algebraic sidecast | `coercion.dag` | Cross-language type mapping |

If your concept doesn't fit any branch, you've likely found a new
branch of the ontology — add it to std/ with an external authority.

**The process:**
1. "I need a cost expression type." → DFS down: what IS cost? It's a
   value in a semiring (add, multiply, zero, one) with a lattice join
   (max). → Walk up from `std/algebra.dag` Semiring + Lattice. Found.
   Don't invent CostExpr; use the existing algebraic structure.
2. "I need a progress tracking type." → DFS down: what IS progress?
   It's an ordering: strict decrease, same, or unknown. → Walk up from
   `std/termination.dag` DescentEvidence. Found. Don't invent
   ProgressKind; it's the same BoundedLattice.
3. "I need a parse result type." → DFS down: what IS a parse result?
   It's a value + state + errors. → Walk up: this is a state monad
   (threaded state) with error accumulation (writer). → If std/ has
   no monad type, ADD it with authority citation (Moggi 1989). Then
   use it everywhere instead of defining 36 bespoke result types.

**The test for any new type:**
- Can you point to its parent in the concept DAG?
- Does that parent already exist in std/?
- If yes: import and compose. If no: add it with an external authority
  citation, THEN import and compose.
- If you can't find ANY parent: you've likely invented an abstraction
  rather than discovered a concept. Reconsider.

**Why this works:** concepts rooted in first principles NEVER need
refactoring — they are what everything else refactors TOWARD. If
something competes with a concept in the DAG, the competing thing is
what needs to change, not the concept. A Semiring will always be a
Semiring. A BoundedLattice will always be a BoundedLattice. Code
grounded in these is permanent.

**Worked examples from the pipeline audit:**

| Ad-hoc type | DFS root | std/ attachment point | Cost of not doing DFS |
|---|---|---|---|
| CostExpr (7 variants) | Semiring + Lattice | `std/algebra.dag` line 145 | 11 walker functions, 30 CX violations |
| SizeExpr (5 variants) | CommutativeMonoid + Lattice | sub-algebra of CostExpr | Separate type doubling walker code |
| ProgressKind (3 variants) | BoundedLattice | `std/termination.dag` line 57 | Duplicate of DescentEvidence |
| 36 parse result types | State × Writer monad | needs std/ addition | 36 types instead of 1 |
| 22 resolve/infer result types | Writer monad | needs std/ addition | 22 types instead of 1 |
| AlgebraTypeTemplate (9 variants) | Free algebra of type constructors | Node (universal carrier) | Separate recursive type |
| InferScope ≅ ModuleContext | Product type (context) | same concept | 2 types for 1 concept |

**When you find a concept not in the DAG:** add it to `std/` with an
external authority citation. The citation is the proof that you
discovered something real, not invented something ad-hoc. Examples:
- `std/termination.dag` cites Floyd (1967), Lee/Jones/Ben-Amram (2001)
- `std/algebra.dag` cites ring theory, lattice theory
- `std/iteration.dag` cites catamorphism theory
- A new `std/discrimination.dag` would cite pattern calculus, tree automata
- A new `std/graph.dag` would cite Cormen et al., Tarjan (1972)

### Navigating the concept DAG: where to start

The `dsl/std/` directory IS the concept DAG. Files are ordered by
dependency depth. Start at the roots and follow imports.

**Layer 0 — Foundations (no imports):**
| File | Concept | External authority |
|------|---------|-------------------|
| `logic.dag` | Classical bivalent truth | Mathematical logic |
| `constructors.dag` | Product, Coproduct | Category theory |
| `algebra.dag` | Monoid → Semiring → Ring → Field, Lattice, BooleanAlgebra, FreeMonoid, PartialFunction | Abstract algebra |
| `iteration.dag` | fold, descend, repeat (bounded computation) | Catamorphism theory |
| `syntax.dag` | BinOp, Literal, Token, ExpectedToken | BNF grammar theory |

**Layer 1 — Compositions (import from Layer 0):**
| File | Concept | Imports from |
|------|---------|-------------|
| `bit.dag` | Word8..Word64 | logic |
| `integer.dag` | Int = Word64 + OrderedRing | algebra, bit |
| `float.dag` | Float = Word64 + ApproximateField | algebra, bit |
| `string_type.dag` | String = FreeMonoid<Char> | algebra, types |
| `types.dag` | Kernel types, container types | algebra |
| `termination.dag` | DescentEvidence, RankingDimension, TerminationProof | algebra |

**Layer 2 — Domain vocabularies (import from Layer 0-1):**
| File | Concept | For whom |
|------|---------|---------|
| `languages.dag` | Language specs (Rust, Python, Go, ...) | Emission |
| `coercion.dag` | TypeCheckpoint, InhabitantDecl | Type rendering |
| `primitives.dag` | PrimitiveContract (43 operation costs) | Complexity analysis |
| `unicode.dag` | Unicode blocks, display width | String handling |
| `resources.dag` | Acquirable capabilities | Service modeling |

**Layer 3 — Application domains (import from Layer 0-2):**
`cloud.dag`, `credentials.dag`, `encoding.dag`, `errors.dag`,
`fermi.dag`, `fidelity.dag`, `filesystem.dag`, `patterns.dag`,
`render.dag`, `behavioral.dag`

**The compiler pipeline** (`src/v2/`) imports from `std/` at Layer 0-1:
- `00_core.dag` ← std.types, std.syntax
- `04_types.dag` ← std.algebra (AlgebraTypeTemplate, profiles)
- `complexity.dag` ← std.types (kernel identity)

**Missing links (to be added):**
- `algebra.dag` should note conceptual dependency on `logic.dag` and `constructors.dag`
- `iteration.dag` should note that `fold` is a catamorphism over `FreeMonoid` from `algebra.dag`
- `stack.dag` should import `algebra.dag` (Stack is a FreeMonoid)
- `containers.dag` should import `algebra.dag` or merge into it (documentation-only today)

---

## Exemplary models

### Foundation chain (reference implementation)

| File | What it models | Authority |
|------|---------------|-----------|
| `std/logic.dag` | Classical bivalent logic | Mathematical definition |
| `std/bit.dag` | Bit, Byte, Word16/32/64/128 | IEC 80000-13 |
| `std/integer.dag` | Signed/unsigned integers from bit widths | Two's complement standard |
| `std/float.dag` | IEEE 754 binary32/binary64 | IEEE 754-2019 |
| `std/string_type.dag` | String as byte sequence + encoding | Structural definition |
| `std/unicode.dag` | Unicode blocks, display widths | Unicode Standard 15.0 |
| `std/filesystem.dag` | POSIX filesystem semantics | POSIX.1-2017 |
| `std/languages.dag` | 13 language specs (comment syntax, naming, types) | Per-language reference docs |

### Other strong models

| File | What it does well |
|------|-------------------|
| `std/patterns.dag` | Generic higher-order compositions: `ensure<Check, Action>`, `upsert`. |
| `std/symbols.dag` | Three-tier encoding (emoji/unicode/ascii). Data table with pure resolution. |
| `extdeps/git.dag` | Faithful Git object model from git-scm docs. Author/committer separation. |
| `extdeps/llm/anthropic.dag` | ContentBlock tagged union matches Anthropic Messages API spec. |
| `src/v2/00_core.dag` | TypeExpr as structural values. Transport bindings carry typed schemas. |
| `src/v2/01_tokenize.dag` | Explicit state threading. Keywords as data table. |

---

## Per-file findings

### dsl/std/

**types.dag** — 7.5/10
- M2: GCP types duplicated (`ProjectId` vs `GcpProjectId`)
- M1: `CloudSecretConfig` embeds policy defaults — policy belongs at call sites
- M2: `ContentEncoding` may overlap with `encoding.dag` — reconcile to one authority

**encoding.dag** — part of foundation chain
- Authority for `Encoding` type (imported by `string_type.dag`)
- Reconcile with any `ContentEncoding` in `types.dag` — one definition only

**containers.dag** — 4/10
- Skeletal, no type definitions — either define container types or delete

**errors.dag** — 7/10 (after cleanup)
- Provider-specific shapes are spec-grounded (GitHub, GCP, Anthropic, OpenAI)
- Generic types removed (HttpErrorShape, AuthError, etc. were invented canonicalizations)

**resources.dag** — 7/10
- M1: `ResourceHandle.type` and `.resource_id` are strings — should be branded
- Good: opaque handles with capabilities, explicit I/O boundaries

**patterns.dag** — 8/10
- Incomplete: `retry` is a stub
- Good: compositional `ensure`, `upsert`, `transaction` patterns

**symbols.dag** — 8/10
- M4: `SymbolId` is a 35-variant flat enum — no structural grouping
- M5: `resolve_symbol` returns empty string on miss instead of erroring

**fidelity.dag** — 6.5/10
- M5: Wildcard `_ => Xl` in transport_depth — silent fallback
- Cost mappings lack justification (why 30s for Xs?)

**fermi.dag** — 6.5/10
- M7: Timeout data duplicated as both `data` and function body
- Good: ordinal pattern, composition via `fermi_max`

**render.dag** — 7.5/10
- Dead code: `RenderMode` enum never referenced
- Good: two-layer architecture, Fragment sum type

**filesystem.dag** — 8/10
- Good: layered tautology, exhaustive matching, no wildcards

**languages.dag** — 8/10
- Good: 13 faithful language models from real language specs

### dsl/extdeps/

**git.dag** — 8.5/10
- M4: `GitRemote.fetch_refspec` as String — could encode grammar
- Good: faithful object model from git-scm documentation

**cargo.dag** — 7/10
- M4: `CargoFeature.dependencies` as `List<String>` — should reference features
- Missing: structured error types for build/test failures

**github/github.dag** — 7.5/10
- M4: `Scopes` as `List<String>` — should reference `GitHubScope` enum
- Should import Git types where GitHub concepts reference Git (e.g., branches, commits)

**github/gists.dag** — 8/10
- M1: `files` is `List<GistFile>` but GitHub API returns `Map<filename, GistFile>`
- Good: comprehensive mock responses

**github/auth.dag** — 4/10
- Very minimal, magic string `"github-token"`, no composition

**cloud/gcp/gcp.dag** — 8/10
- Hardcoded regions data will go stale (GCP adds regions)
- Good: dual identity, precise service account model, real scope URIs

**llm/anthropic.dag** — 8/10
- M4: `ThinkingConfig.type` as String — should be enum
- Good: ContentBlock tagged union, cache_control, precise token budgets

**llm/openai.dag** — 8/10
- Nested destructuring via string paths (`"content/0/text"`) is fragile
- Good: ResponseFormat tagged union, ToolChoice tagged union

**llm/llm.dag** — 7/10
- `Role`, `StopReason`, `TokenUsage` are shared concepts documented by both providers — valid
- M1: `LlmMessage.content` as String — doesn't model multimodal content (both providers support richer content)

### src/v2/ (compiler)

**00_core.dag** — 8.5/10
- M4: `AuthConfig.scheme` as String — should be enum
- Good: TypeExpr is exemplary, predicates compositional

**01_tokenize.dag** — 8.5/10
- M1: `Unknown` conflates invalid chars and unterminated strings
- Good: explicit state threading, keywords as data

**02_parse.dag** — 6/10 (CRITICAL)
- M6: **42 result types** — needs generic `Result<T>`
- M8: `kind_tag(token)` string comparison — fragile
- M7: `keyword_to_name` duplicates tokenizer keyword table

**03_resolve.dag** — 8/10
- M5: Wildcard import `"*"` sentinel — should be `Optional<List<String>>`
- Good: Kahn's algorithm, diagnostic aggregation

**04_typecheck.dag** — 5.5/10 (CRITICAL)
- M5: **`lookup_in_scope` silently returns `unit_type()` on miss** — fabrication
- M5: **`lookup_field_type` also silently returns `unit_type()`**
- M8: `infer_method_call_type` dispatches on string method names

**05_emit.dag** — 6.5/10
- M5: **Anonymous products → `serde_json::Value`** — silent data loss
- M8: `needs_reference` hardcodes type names as strings

**06_pipeline.dag** — 8/10
- Good: clean linear pipeline, explicit error gating

---

## Deleted files (this session)

| File | Reason |
|------|--------|
| `std/policy.dag` | Duplicate of `types.dag`, 7-line file |
| `std/cloud.dag` | Duplicate of `types.dag`, structurally incompatible AuthScheme |
| `std/behavioral.dag` | Moved to `shared/behavioral.dag` to keep the vocabulary shared but outside `std/`. |

Behavioral imports and `operation_behaviors` data blocks removed from
8 extdeps files (cargo, git, gists, anthropic, openai, iam, secret_manager, sts).

---

## Known future work

### P0 — Correctness (fabrication fallbacks)

The v2 compiler inherits v1's worst anti-pattern: silent defaults on
lookup miss. These mask real errors that cascade into wrong generated code.

**04_typecheck.dag** (21+ instances):
- `lookup_field_type` → `unit_type()` on missing field or non-Product type
- `infer_method_call_type` → `unit_type()` for unknown methods
- `infer_expr` wildcard `_ =>` → `unit_type()` for unhandled expression types
- Missing RecordLit type → placeholder `Named` instead of error

**05_emit.dag**:
- Anonymous products/coproducts → `serde_json::Value` (silent structure erasure)
- `emit_data_value_json` wildcard `_ => "null"` (variables/calls silently become null)
- `extract_service_name` → `"Unknown"` string on fail
- `.expect("valid data definition")` swallows JSON parse errors

**03_resolve.dag**:
- `find_index_in_list` → `-1` sentinel (should be Optional)
- `get_at_index_int` → `0` for missing in-degree (**corrupts topo sort**)
- `get_at_index` → `[]` for out-of-bounds (hides missing data)

**Fix:** Return `Optional` or `Result` types. Propagate `None` + diagnostic.

### P1 — Sustainability (structural modeling)

**Generic Result type** — highest leverage single change:
- 02_parse.dag defines **57 bespoke result types**
- 04_typecheck.dag defines **13+ more**
- All follow `{ value: T, state/diagnostics }` — one generic eliminates 70 types

**Structural token matching** — 02_parse.dag:
- **48+ uses of `kind_tag()`** extracting string from TokenKind, comparing with `==`
- Should be direct pattern matching on TokenKind variants

**Keyword table duplication** — between 01_tokenize.dag and 02_parse.dag:
- `keyword_to_name` (28 if-else clauses) + `keyword_to_arg_label` (23 more)
- Duplicates the tokenizer's `data keywords` table — derive from single source

**Method/predicate dispatch** — string chains that should be enums:
- `infer_method_call_type`: 12-branch if-else on string method names
- `parse_single_predicate`: 8 string matches on predicate names
- `emit_method_call`: hardcoded method name → Rust method mapping
- `emit_primitive_type` / `needs_reference` / `is_primitive_numeric`: string checks on type names

### P1 — Dummy sentinel values (02_parse.dag)

**20+ dummy node constructions** — empty string names, null spans as
error recovery. `Field { name: "", type_expr: Named { name: "" } }` is
an invalid state that should be unrepresentable. Downstream can't
distinguish "valid empty" from "error recovery." Fix: return Result,
never construct invalid AST.

### P2 — Anemic types and missing structure

**ParserState** (02_parse.dag): missing filename, module context,
recovery hints. Error diagnostics have `module_name: none`.

**Wildcard import**: both parse.dag and resolve.dag use `"*"` string
sentinel. Should be `ImportNames = All | Specific { names: List<String> }`.

**Resolve exports**: variant names conflated with type names — should
separate `ExportedNames { types, constructors, functions }`.

**Pipeline result types** (06_pipeline.dag): `ParseResult` has
independent `module?` and `error?` — should be sum `Ok | Err`.

### P2 — DSL std cleanup

- Reconcile `ContentEncoding` (types.dag) with `Encoding` (encoding.dag) — M2
- `containers.dag`: skeletal, no type definitions — define or delete
- `fidelity.dag`: wildcard `_ => Xl` (silent fallback), cost mappings lack justification
- `fermi.dag`: timeout data duplicated as both `data` and function body
- `render.dag`: `RenderMode` enum is dead code
- `symbols.dag`: `resolve_symbol` returns empty string on miss
- `types.dag`: GCP type duplicates, policy defaults embedded in types

### P2 — Extdeps improvements

- `github.dag`: should import Git types for shared concepts (branches, commits)
- `github/auth.dag`: minimal, magic string, no composition
- `llm/llm.dag`: `LlmMessage.content` as String — doesn't model multimodal
- String fields that should be enums: `AuthConfig.scheme`, `ThinkingConfig.type`,
  `GitRemote.fetch_refspec`, pagination cursors, `GistFile.language`
- Stale hardcoded data: model lists (anthropic, openai), GCP regions

### Accepted debt (dies with self-hosting)

Rust-specific constructs in 05_emit.dag (S81 from SUSTAINABILITY.md):
- `#[derive(...)]` and `#[serde(tag = ...)]` attributes
- Primitive type → Rust type mapping (Int→i64, etc.)
- `NonEmptyVec<T>`, `NonEmptyBTreeSet<T>` as raw Rust strings
- Hardcoded reqwest and std::process::Command
- `serde_json::json!(...)` macro

Not worth fixing — the v2 emitter replaces all of this.

---

## Appendix: Preferred implementations

Concrete design targets for each major issue. These are the models
someone should implement — the "what it should look like" for each fix.

### A1: Generic result types (replaces P1 result proliferation)

Two patterns exist in the codebase. Both should be generic.

```dag
// Pattern 1: Parsing — threads state, may fail
type ParseResult<T>
  = Ok { value: T, state: ParserState }
  | Err { error: Diagnostic, state: ParserState }

// Pattern 2: Analysis — accumulates diagnostics
type Checked<T> {
  value: T
  diagnostics: List<Diagnostic>
}
```

**ParseResult as a sum type** is the key insight. The current design
has independent `value: T?` and `error: Diagnostic?` — four states
(both, neither, one, other) where only two are valid. The sum type
makes illegal states unrepresentable.

Replaces in 02_parse.dag: `ExprResult`, `ItemResult`, `TypeResult`,
`NameResult`, `FieldResult`, `VariantResult`, `ParamResult`,
`ImportResult`, `ModuleResult`, ... (57 types → 1).

Replaces in 04_typecheck.dag: `ResolveResult`, `InferResult`,
`TypedItemResult`, `AccessCheckResult`, ... (13 types → 1).

Helper for the common "try then continue" pattern:

```dag
fn try_parse<T>(r: ParseResult<T>) -> ParseResult<T> {
  // Identity — but makes intent explicit at call sites.
  // The sum type forces callers to match Ok/Err.
  r
}

// Example usage (current):
//   let r = parse_expr(state: state)
//   if has_err(err: r.err) { return XxxResult { ..., err: r.err } }
//   // use r.value
//
// Becomes:
//   match parse_expr(state: state) {
//     Ok { value: expr, state: s } => // use expr and s
//     Err { error: e, state: s } => Err { error: e, state: s }
//   }
```

### A2: Structural token dispatch (replaces P1 kind_tag)

The problem: `kind_tag` extracts a string from `TokenKind` (a sum type),
then callers compare strings. This defeats the type system.

```dag
// Option A: TokenTag enum (parallel to TokenKind but without payloads)
type TokenTag
  = TagKwModule | TagKwImport | TagKwType | TagKwResource
  | TagKwCapability | TagKwOperation | TagKwPattern
  | TagKwInput | TagKwOutput | TagKwData | TagKwMatch
  | TagKwService | TagKwFn | TagKwFunc | TagKwExtern
  | TagKwLet | TagKwReturn | TagKwIf | TagKwElse
  | TagKwFor | TagKwIn | TagKwWhere | TagKwWith
  | TagKwTrue | TagKwFalse | TagKwImport | TagKwModule
  | TagKwInterface | TagKwPipeline | TagKwProfile
  | TagKwIdempotent | TagKwReadonly | TagKwHermetic
  | TagIdent | TagLitStr | TagLitInt | TagLitFloat | TagLitNull
  | TagLBrace | TagRBrace | TagLParen | TagRParen
  | TagLBracket | TagRBracket
  | TagColon | TagComma | TagDot | TagEq | TagFatArrow
  | TagPipeArrow | TagPlus | TagMinus | TagStar | TagSlash
  | TagPercent | TagBang | TagQuestion
  | TagAnd | TagOr | TagLt | TagGt | TagLtEq | TagGtEq
  | TagEqEq | TagBangEq | TagNewline | TagEof | TagUnknown

fn token_tag(kind: TokenKind) -> TokenTag {
  match kind {
    KwModule => TagKwModule
    Ident { name: _ } => TagIdent
    LitStr { value: _ } => TagLitStr
    LBrace => TagLBrace
    // ... exhaustive — compiler catches missing variants
  }
}

fn check(state: ParserState, expected: TokenTag) -> Bool {
  match peek(state: state) {
    Some { value: t } => token_tag(kind: t.kind) == expected
    None => false
  }
}
```

**Option B (preferred if language supports it):** Skip `TokenTag`
entirely and match patterns directly in callers:

```dag
fn check_kind(state: ParserState, expected: TokenKind) -> Bool {
  match peek(state: state) {
    Some { value: t } => matches_variant(t.kind, expected)
    None => false
  }
}

// Usage: check_kind(state: s, expected: LBrace)
// Requires: `matches_variant` intrinsic or pattern-match sugar
```

### A3: Optional returns in typecheck (replaces P0 fabrication)

Every lookup that currently returns `unit_type()` on miss should
return `TypeExpr?` and force the caller to handle absence.

```dag
// CURRENT (fabrication):
fn lookup_in_scope(scope: InferScope, name: String) -> TypeExpr {
  // ... search locals, params, types ...
  unit_type()  // miss → silent Unit
}

// PREFERRED:
fn lookup_in_scope(scope: InferScope, name: String) -> TypeExpr? {
  let local = find(scope.locals, b => b.name == name)
  match local {
    Some { value: binding } => Some { value: binding.resolved }
    None =>
      let param = find(scope.func_params, p => p.name == name)
      match param {
        Some { value: p } => Some { value: p.type_expr }
        None => lookup_type(env: scope.type_env, name: name)
          // returns TypeExpr? — None propagates naturally
      }
  }
}

// Callers become explicit:
fn infer_var(scope: InferScope, name: String, span: SourceSpan) -> Checked<TypedExpr> {
  match lookup_in_scope(scope: scope, name: name) {
    Some { value: te } =>
      Checked {
        value: TypedExpr { expr: Var { name: name }, resolved_type: te },
        diagnostics: []
      }
    None =>
      Checked {
        value: TypedExpr { expr: Var { name: name }, resolved_type: unit_type() },
        diagnostics: [Diagnostic {
          severity: Error,
          message: concat("undefined variable: ", name),
          span: Some { value: span },
          module_name: scope.module_name
        }]
      }
  }
}
```

Same pattern for `lookup_field_type`:

```dag
// CURRENT:
fn lookup_field_type(type_expr: TypeExpr, field_name: String) -> TypeExpr {
  match type_expr {
    Product { fields: fields } =>
      // ... search ... else unit_type()
    _ => unit_type()
  }
}

// PREFERRED:
fn lookup_field_type(type_expr: TypeExpr, field_name: String) -> TypeExpr? {
  match type_expr {
    Product { name: _, fields: fields } =>
      match find(fields, f => f.name == field_name) {
        Some { value: f } => Some { value: f.type_expr }
        None => None  // field not found — caller decides what to do
      }
    Optional { inner: inner } =>
      lookup_field_type(type_expr: inner, field_name: field_name)
    _ => None  // not a product — caller must report error
  }
}
```

### A4: Method and predicate enums (replaces P1 string dispatch)

```dag
// Method names are a closed set — model them as such
type PipeMethod
  = Map | Filter | Fold | First | Last
  | Count | Join | Any | Contains
  | Enumerate | Sum | Chars | Split

// Method type inference becomes structural
fn infer_method_result(receiver: TypeExpr, method: PipeMethod) -> TypeExpr? {
  match method {
    Map => Some { value: receiver }
    Filter => Some { value: receiver }
    Fold => None  // depends on accumulator — caller resolves from args
    First => extract_element_type(container: receiver)
    Last => extract_element_type(container: receiver)
    Count => Some { value: Primitive { name: "Int", span: no_span() } }
    Join => Some { value: Primitive { name: "String", span: no_span() } }
    Any => Some { value: Primitive { name: "Bool", span: no_span() } }
    Contains => Some { value: Primitive { name: "Bool", span: no_span() } }
    Enumerate => Some { value: receiver }  // List<T> → List<(Int, T)>
    Sum => Some { value: Primitive { name: "Int", span: no_span() } }
    Chars => Some { value: Container { kind: "List", element: Primitive { name: "String", span: no_span() }, span: no_span() } }
    Split => Some { value: Container { kind: "List", element: Primitive { name: "String", span: no_span() }, span: no_span() } }
  }
}

// Predicate kinds — also a closed set
type PredicateKind
  = PatternPred | FormatPred | BrandPred
  | ContentPred | DomainPred | RangePred | NonEmptyPred

// Parser produces PredicateKind, not strings:
fn parse_predicate_kind(name: String) -> PredicateKind? {
  match name {
    "pattern" => Some { value: PatternPred }
    "format" => Some { value: FormatPred }
    "brand" => Some { value: BrandPred }
    "content" => Some { value: ContentPred }
    "domain" => Some { value: DomainPred }
    "range" => Some { value: RangePred }
    "non_empty" => Some { value: NonEmptyPred }
    _ => None  // unknown predicate — caller emits diagnostic
  }
}
```

### A5: Import names as sum type (replaces P2 wildcard sentinel)

```dag
// CURRENT: uses ["*"] as sentinel for "import all"
type Import {
  module_path: String
  names: List<String>  // ["*"] means all, [] means empty block
  span: SourceSpan
}

// PREFERRED:
type ImportNames
  = ImportAll                          // import foo.bar
  | ImportSpecific { names: List<String> }  // import foo.bar { X, Y }

type Import {
  module_path: String
  names: ImportNames
  span: SourceSpan
}

// Parser:
//   bare import (no braces) → ImportAll
//   import foo { X, Y }    → ImportSpecific { names: ["X", "Y"] }
//   import foo { }          → ImportSpecific { names: [] }
//
// Resolver:
//   match import.names {
//     ImportAll => export all names from target module
//     ImportSpecific { names } => validate each name exists
//   }
```

### A6: Resolve index lookups (replaces P0 sentinel values)

```dag
// CURRENT: returns -1, 0, [] as sentinels
fn find_index_in_list(names: List<String>, target: String) -> Int {
  // ... None => -1
}

// PREFERRED:
fn find_index_in_list(names: List<String>, target: String) -> Int? {
  let matches = names |> enumerate |> filter(pair => pair.second == target)
  match matches |> first {
    Some { value: pair } => Some { value: pair.first }
    None => None
  }
}

// For in-degree lookup (Kahn's algorithm):
fn get_in_degree(in_degrees: List<Int>, index: Int) -> Int {
  match get_at_index_safe(items: in_degrees, index: index) {
    Some { value: n } => n
    None => panic("get_in_degree: index out of bounds")
    // Panic is correct here — out-of-bounds in Kahn's is a
    // programming error, not a data error. Silent 0 corrupts
    // the topological sort.
  }
}
```

### A7: Pipeline result as sum type (replaces P2 anemic pipeline)

```dag
// CURRENT:
type ParseFileResult {
  module: Module?    // independent optionals —
  error: Diagnostic? // four states, only two valid
}

// PREFERRED:
type ParseFileResult
  = ParseOk { module: Module, state: ParserState }
  | ParseFail { error: Diagnostic, state: ParserState }

// Pipeline becomes:
fn compile_file(source: String) -> ParseFileResult {
  let tokens = tokenize(source: source)
  match parse_module(tokens: tokens) {
    Ok { value: module, state: s } => ParseOk { module: module, state: s }
    Err { error: e, state: s } => ParseFail { error: e, state: s }
  }
}

// Compile pipeline error gating:
fn compile(sources: List<SourceFile>) -> CompileResult {
  let parse_results = map(sources, s => compile_file(source: s.content))

  // Type-safe error check — no list length comparison needed
  let failures = filter(parse_results, r => match r {
    ParseFail { error: _, state: _ } => true
    _ => false
  })

  if count(failures) > 0 {
    CompileResult {
      files: [],
      diagnostics: map(failures, f => f.error)
    }
  } else {
    let modules = map(parse_results, r => r.module)
    // ... continue pipeline with guaranteed-valid modules
  }
}
```

### A8: ParserState with context (replaces P2 anemic state)

```dag
// CURRENT:
type ParserState {
  tokens: List<Token>
  pos: Int
}

// PREFERRED:
type ParserState {
  tokens: List<Token>
  pos: Int
  filename: String       // for error messages
  module_path: String    // for qualified name context
}

// Diagnostic construction gets context automatically:
fn parse_error(state: ParserState, msg: String) -> Diagnostic {
  Diagnostic {
    severity: Error,
    message: msg,
    span: current_span(state: state),
    module_name: Some { value: state.module_path }
  }
}
```

### A9: Keyword table as shared data (replaces P1 duplication)

```dag
// In 01_tokenize.dag (already exists):
data keywords: Map<String, TokenKind> = [
  { key: "type", value: KwType },
  { key: "resource", value: KwResource },
  // ... 30+ entries
]

// In 00_core.dag or shared module — derive reverse mapping:
data keyword_names: Map<TokenKind, String> = reverse_map(keywords)
// or: fn keyword_name(kind: TokenKind) -> String? = lookup(keyword_names, kind)

// 02_parse.dag uses the shared table:
fn keyword_to_name(kind: TokenKind) -> String? {
  lookup(keyword_names, key: kind)
}

// Eliminates: 28 if-else clauses in keyword_to_name
// Eliminates: 23 if-else clauses in keyword_to_arg_label
// Single authority: the keywords data table in tokenize.dag
```

### A10: Export separation in resolve (replaces P2 conflation)

```dag
// CURRENT: variant names mixed with type names in flat list

// PREFERRED:
type ExportedNames {
  types: List<String>          // type Foo, type Bar
  constructors: List<String>   // Foo's variants: A, B, C
  functions: List<String>      // fn baz, func qux
  data: List<String>           // data constants
}

fn collect_exports(module: Module) -> ExportedNames {
  ExportedNames {
    types: map(filter(module.items, i => is_type_def(item: i)), i => i.name),
    constructors: flat_map(
      filter(module.items, i => is_coproduct(item: i)),
      i => get_variant_names(item: i)
    ),
    functions: map(filter(module.items, i => is_fn_def(item: i)), i => i.name),
    data: map(filter(module.items, i => is_data_def(item: i)), i => i.name)
  }
}

// Resolver can now validate: "did you import a type, a constructor,
// or a function?" — different validation rules per kind.
```

### A11: Encoding type reconciliation (P2 — M2 violation)

`types.dag` defines `ContentEncoding`, `encoding.dag` defines `Encoding`.
Both claim authority over the same concept. `string_type.dag` imports
from `encoding.dag`, making it the foundation chain's authority.

```dag
// encoding.dag — KEEP as single authority
// Ref: IANA Character Sets registry, MIME charset parameter
type Encoding = ASCII | UTF8 | Latin1 | Binary

// types.dag — DELETE ContentEncoding definition, import instead:
import std.encoding { Encoding }

// NOTE: encoding.dag currently has Text and Unknown variants that
// don't correspond to real encodings. Remove them:
//   Text   → not an encoding, it's a classification (use is_text_readable)
//   Unknown → violates M5 (silence is fabrication) — if encoding
//             is unknown, that's an error, not a valid state
```

### A12: Emit anonymous record fallback (P0 — fabrication)

```dag
// CURRENT in 05_emit.dag:
fn emit_product_type_expr(name: String?, fields: List<Field>) -> String {
  match name {
    Some { value: n } => n
    None =>
      if count(fields) == 1 { emit_type_expr(type_expr: first(fields).type_expr) }
      else { "serde_json::Value" }  // <-- silent structure erasure
  }
}

// PREFERRED — anonymous records MUST be named before emission.
// The fix belongs in the typechecker, not the emitter:

// In 04_typecheck.dag — name anonymous records during type resolution:
fn name_anonymous_record(fields: List<Field>, context: String) -> TypeExpr {
  // Context is the enclosing function/let binding name.
  // { left: Int, right: Int } in fn parse_binop → BinopRecord
  Product {
    name: Some { value: synthesize_name(fields: fields, context: context) },
    fields: fields,
    span: no_span()
  }
}

// The emitter then NEVER sees unnamed products. If one arrives,
// it's a bug — fail loudly:
fn emit_product_type_expr(name: String?, fields: List<Field>) -> String {
  match name {
    Some { value: n } => n
    None => panic("emit_product_type_expr: unnamed record reached emitter")
  }
}
```

### A13: Symbol resolution without fabrication (P2 — M5)

```dag
// CURRENT in std/symbols.dag:
fn resolve_symbol(id: SymbolId, tier: SymbolTier) -> String {
  let matches = filter(standard_symbols, s => s.id == id)
  match first(matches) {
    Some { value: entry } => // extract tier
    None => ""  // <-- silent empty string on miss
  }
}

// PREFERRED:
fn resolve_symbol(id: SymbolId, tier: SymbolTier) -> String? {
  let matches = filter(standard_symbols, s => s.id == id)
  match first(matches) {
    Some { value: entry } =>
      match tier {
        Emoji => Some { value: entry.emoji }
        Unicode => Some { value: entry.unicode }
        Ascii => Some { value: entry.ascii }
      }
    None => None  // caller decides: fallback to ascii? error?
  }
}
```

### A14: Fidelity without wildcard fallback (P2 — M5)

```dag
// CURRENT in std/fidelity.dag:
fn transport_depth(tc: TransportClass) -> FermiDepth {
  match tc {
    LocalDirect => Xs
    ShellLocal => Sm
    FileBoundary => Md
    RestNetwork => Lg
    InterfaceStub => Xs
    _ => Xl  // <-- silent fallback: new transport class → Xl
  }
}

// PREFERRED — exhaustive, no wildcard:
fn transport_depth(tc: TransportClass) -> FermiDepth {
  match tc {
    LocalDirect => Xs
    ShellLocal => Sm
    FileBoundary => Md
    RestNetwork => Lg
    InterfaceStub => Xs
    Unknown => Xl
  }
  // If a new TransportClass variant is added, the compiler
  // forces you to add a case here. No silent default.
}
```

### A15: Fermi timeout — single authority (P2 — M7)

```dag
// CURRENT: same mapping as both data AND function
data fermi_timeouts: List<FermiTimeout> = [
  { depth: Xs, timeout_ms: 30000, label: "30 seconds" },
  { depth: Sm, timeout_ms: 300000, label: "5 minutes" },
  // ...
]

fn timeout_for_depth(depth: FermiDepth) -> Int {
  match depth {
    Xs => 30000    // <-- DUPLICATE of data above
    Sm => 300000
    // ...
  }
}

// PREFERRED — derive function from data:
data fermi_timeouts: List<FermiTimeout> = [
  { depth: Xs, timeout_ms: 30000 },
  { depth: Sm, timeout_ms: 300000 },
  { depth: Md, timeout_ms: 600000 },
  { depth: Lg, timeout_ms: 1800000 },
  { depth: Xl, timeout_ms: 3600000 }
]

fn timeout_for_depth(depth: FermiDepth) -> Int {
  let matches = filter(fermi_timeouts, t => t.depth == depth)
  match first(matches) {
    Some { value: t } => t.timeout_ms
    None => panic("timeout_for_depth: unknown depth")
  }
}
// Single authority: fermi_timeouts data table.
// label field removed — derive from timeout_ms if needed.
```

### A16: GitHub → Git type references (P2 — objective relationships)

GitHub's branching, commit, and diff models are built on Git's.
This relationship is documented in GitHub's own docs. The DAG
modeling should reflect it.

```dag
// In extdeps/github/github.dag:
import extdeps.git { CommitSha, GitRef, GitCommit, DiffHunk }

// CURRENT:
type Repository {
  owner: String
  name: String
  full_name: String
  default_branch: String  // <-- bare string
  // ...
}

// PREFERRED:
type Repository {
  owner: GitHubUser
  name: String
  full_name: String
  default_branch: GitRef  // ← references Git's branch model
  // ...
}

// Pull request references Git concepts directly:
type PullRequest {
  number: Int
  head: GitRef       // ← Git branch
  base: GitRef       // ← Git branch
  merge_commit: CommitSha?  // ← Git commit SHA
  // ...
}

// The relationship is factual: GitHub's API docs say
// "head" and "base" are Git refs. Not interpretation.
```

### A17: LLM multimodal content (P2 — M1 faithfulness)

Both Anthropic and OpenAI support rich content beyond plain strings.
The shared `LlmMessage` type should reflect this.

```dag
// CURRENT in extdeps/llm/llm.dag:
type LlmMessage {
  role: Role
  content: String  // <-- doesn't model multimodal
}

// PREFERRED — content is a list of typed blocks:
type ContentBlock
  = TextContent { text: String }
  | ImageContent { source: ImageSource }

type ImageSource
  = Base64Image { media_type: String, data: String }
  | UrlImage { url: String }

type LlmMessage {
  role: Role
  content: List<ContentBlock>
}

// This matches what BOTH providers actually accept:
// - Anthropic: content is List<ContentBlock> (text, image, tool_use, tool_result)
// - OpenAI: content is string OR array of {type: "text"/"image_url", ...}
//
// Provider-specific block types (ToolUseBlock, etc.) stay in
// anthropic.dag / openai.dag. The shared type covers the
// intersection that both providers document.
```

### A18: String fields → enums across extdeps (P3 — M4)

```dag
// In 00_core.dag — AuthConfig:
// CURRENT:  scheme: String
// PREFERRED:
type AuthScheme = Bearer | ApiKey | Basic | Custom { header: String }

type AuthConfig {
  scheme: AuthScheme  // closed set, not open string
  // ...
}

// In extdeps/llm/anthropic.dag — ThinkingConfig:
// CURRENT:  type: String  (always "enabled")
// PREFERRED:
type ThinkingMode = Enabled | Disabled

type ThinkingConfig {
  mode: ThinkingMode
  budget_tokens: Int?
}

// In extdeps/github/github.dag — scopes:
// CURRENT:  scopes: List<String>
// PREFERRED:
type GitHubScope
  = RepoRead | RepoWrite | RepoAdmin
  | GistRead | GistWrite
  | UserRead | UserEmail
  | OrgRead | OrgAdmin
  | Workflow
  // Ref: https://docs.github.com/en/apps/oauth-apps/building-oauth-apps/scopes-for-oauth-apps

type GitHubAuthToken {
  token: Secret
  scopes: List<GitHubScope>
}
```
