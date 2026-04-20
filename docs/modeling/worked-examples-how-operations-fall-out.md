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

