# Impossible Bug Classes — Worked Examples

> **Parent:** `THESIS.md` §"Enumerable impossible-bug classes" + §"Correctness is structural, not behavioral (meta-claim)"

This document is the completeness-spec audit for gunbc's core thesis claim:
that entire bug classes engineers struggle with in traditional languages are
either *impossible by construction* or *caught at compile time* in gunbc.

## Governing rule

The list below is a **completeness specification**, not a demo suite. If a
bug class is not handled, the response is not "schedule it" — it's a **design
audit**: where did the structure miss this? Can the structure be enhanced so
the compiler takes on more of the work?

R1 ships when every class in this list has a satisfactory structural story:
either it's already IBC/CE, or there is a named structural path to make it
so.

## Classification rubric

Each bug class is tagged with one of four states:

| Tag | Meaning |
|---|---|
| **IBC** | *Impossible by construction.* The structure itself does not admit the bug — you cannot write it even if you try. |
| **CE** | *Compile error.* The structure admits the bug, but the compiler rejects it at compile time. (Weaker than IBC; preferred fallback when structural impossibility is impractical.) |
| **PARTIAL** | Handled in some cases; others slip through. Named gap. |
| **GAP** | Neither IBC nor CE yet. Concept exists in the ontology but the enforcement is missing. Structural path must be named. |

Every `GAP` row is a design-audit trigger. "We'll catch it at runtime" is
not an acceptable answer.

## How this doc is used

- THESIS commits to the list of bug classes (the *what*).
- This doc records how the structure prevents each class (the *how*) and
  the current state (IBC / CE / PARTIAL / GAP).
- Gaps are traced to ROADMAP follow-up lanes.
- When a new bug class surfaces externally (user reports, review feedback,
  audit), add it here first; fix structure second.

---

## Group A — Null / optional family

### 1. Null dereference / use-before-init

**Traditional example (Java).**
```java
User u = findUser(id);   // may return null
String name = u.name();  // NullPointerException at runtime
```

**Why traditional compilers miss it.** Most mainstream languages treat
`null` / nil as a value of every reference type. The type system doesn't
distinguish "definitely a value" from "maybe a value."

**gunbc status: IBC.**
No raw null exists. Optionality is expressed via `Cardinality<T, AtMost(1)>`
(surface syntax `T?`). You cannot assign `None` to a non-optional field
because there is no null value to assign. Extraction is through pattern
match only; there is no forced unwrap.

**Evidence:** `dsl/std/algebra.dag` (no unwrap method on Option);
`dsl/std/types.dag` (Option via Cardinality); `src/v3/compiler/src/infer.rs`
(pattern-match is the only extractor).

---

### 2. Empty-list head access

**Traditional example (Python).**
```python
items = []
first = items[0]   # IndexError at runtime
```

**Why traditional compilers miss it.** List types don't carry cardinality.
`[]` and `[1, 2, 3]` have the same static type.

**gunbc status: CE** (trending toward IBC).
`first(List<T>)` returns `Option<T>`, not `T`. The compiler forces the
caller to pattern-match the result. You cannot extract a value that might
not be there.

**Path to IBC.** A `NonEmpty<T>` = `Cardinality<T, AtLeast(1)>` would let
a producer promise non-emptiness and a consumer require it. Then `first` on
`NonEmpty<T>` returns `T` directly. The `List<T>` case stays CE.

**R1 status.** CE is sufficient; NonEmpty is polish. No gap.

---

### 3. Nested-optional flatten

**Traditional example (Rust).**
```rust
let config: Option<Option<PortNumber>> = load_config();
let port = config.flatten().unwrap_or(8080);  // user flattens by hand
```

**Why traditional compilers miss it.** Nesting is permitted by generics;
flattening is user work.

**gunbc status: GAP.**
Users can currently construct `Option<Option<T>>`. Cardinality refinement
(which would make the nesting collapse to `AtMost(1)` automatically) is
deferred.

**Structural path.** Extend `Cardinality` to compose:
`Cardinality<Cardinality<T, AtMost(1)>, AtMost(1)> ≡ Cardinality<T, AtMost(1)>`.
This is substrate work — the compiler's type system must recognize the
composition law. Once in place, nested optionals are IBC.

**R1 status.** GAP → must close for R1. Lane-level work item.

---

### 4. Force-unwrap panic

**Traditional example (Rust).**
```rust
let port: Option<u16> = env_port();
let p = port.unwrap();  // panics if None at runtime
```

**Why traditional compilers miss it.** `unwrap()` is a library method that
panics; it's syntactically valid at compile time.

**gunbc status: IBC.**
There is no `unwrap()` method on Option in `dsl/std/`. Pattern matching is
the only extractor. You cannot write the bug because the extractor doesn't
exist.

---

## Group B — Type / shape family

### 5. Enum-variant non-exhaustive match

**Traditional example (Go).**
```go
switch color {
  case Red:   ...
  case Blue:  ...
  // Green silently falls through to nothing
}
```

**Why traditional compilers miss it.** Go's switch is not exhaustive by
default. Rust warns but doesn't error; most languages don't check.

**gunbc status: CE.**
`match` over `Disj` must cover every variant. Missing arms produce a
compile error naming the missing variants.

**Evidence:** `src/v3/compiler/src/infer.rs:492-531` — exhaustiveness
checker with message `"non-exhaustive match: missing arm(s) for
variant(s)"`.

---

### 6. Schema drift between client and server

**Traditional example (TypeScript + Python).** Frontend expects
`user.age: string` (typo in TS types); backend returns `age: number`.
Runtime JSON-parse succeeds; coercion bugs appear in UI.

**Why traditional compilers miss it.** Two codebases, two type systems, no
shared source of truth. Schema validators help but are out-of-band.

**gunbc status: CE (trending IBC).**
Client, server, and any other target derive from the same `.dag`
declaration. A field's type is declared once; all targets emit from it.
Drift is impossible because there is no second authority to drift against.

**Evidence:** `dsl/std/coercion.dag` (single-authority type checkpoint);
THESIS §"Omni-emission" and §"Target realization efficiency" (one spec,
many targets, zero per-target drift).

---

### 7. Transport / type drift (REST path mismatched with typed route)

**Traditional example (Express + fetch).** Server declares
`app.get('/orders/:id', ...)` where `:id` is expected to be a UUID;
client fetches `/orders/${someNumber}`. No compile-time check.

**Why traditional compilers miss it.** Route templates are strings on both
sides; type information doesn't flow through the URL.

**gunbc status: CE.**
Path templates parse into typed tokens (`std.http_path::PathTemplate`).
Parameter types are declared at the `service` level and consumed at call
sites. Type mismatch between path parameter and caller argument is a
compile error.

**Evidence:** `dsl/std/http_path.dag:18-79` (PathTemplate + typed tokens);
`dsl/std/effects.dag:73-91` (KeySource typed discriminator).

---

## Group C — Runtime safety (Tier 2)

### 8. Array out-of-bounds

**Traditional example (C).**
```c
int arr[10];
int x = arr[15];   // undefined behavior / memory corruption
```

**Why traditional compilers miss it.** Array bounds aren't part of the
type. Indices aren't proven in range.

**gunbc status: GAP.**
No `FixedArray<T, n>` with bounded index types yet. `List<T>` access is
through iteration / pattern-match, which is safe; direct indexing with
unchecked integers is absent.

**Structural path.** `Cardinality(element, Exact(n))` as a type-level
length constraint; indices typed as `BoundedInt<0, n>`. Then index-in-bounds
is a type-check.

**R1 status.** GAP → must close for R1. Subsumed by the Tier-2 substrate
program (see class 9).

---

### 9. Division by zero

**Traditional example (Python).**
```python
rate = total / count   # ZeroDivisionError if count == 0
```

**Why traditional compilers miss it.** Integer/float division is defined
for any pair, including `x / 0`. Runtime check required.

**gunbc status: GAP.**
Field operations admit division but don't require a non-zero-divisor
proof. Tier 2 runtime safety is named in THESIS but the substrate doesn't
land yet.

**Structural path.** Two options:
- (a) **Total division.** `div(a, b)` returns `Option<T>`; callers
  pattern-match. IBC via API shape.
- (b) **Proof-requiring division.** Divisor typed `NonZero<T>`; compiler
  proves non-zero from surrounding structure or requires an explicit
  refinement. CE via type wall.

Option (a) is cheaper; option (b) is stronger.

**R1 status.** GAP → must close for R1. Part of the Tier-2 substrate
program (T-Tier2 lane).

---

### 10. Integer overflow

**Traditional example (C / Java).**
```java
int a = Integer.MAX_VALUE;
int b = a + 1;   // silently wraps to Integer.MIN_VALUE
```

**Why traditional compilers miss it.** Fixed-width integer semantics wrap
(C, Java) or silently promote (Python, Ruby). Some compilers warn on
constant expressions; none structurally prevent it.

**gunbc status: GAP.**
`Int` / `Word64` are fixed-width; `OrderedRing<Word64>` admits addition
without overflow proof.

**Structural path.** Either (a) default to unbounded integers (`BigInt`)
with explicit fixed-width opt-in, or (b) make fixed-width arithmetic
return `Option<T>` / require overflow proofs. Option (a) is cheaper and
likely correct: fixed-width is an emission concern, not a semantics
concern.

**R1 status.** GAP → must close for R1. Tier-2 substrate.

---

## Group D — Units / dimensions

### 11. Unit / dimension mismatch

**Traditional example (any language).**
```python
timeout_ms = 5000
sleep(timeout_ms)   # sleep expects seconds → sleeps 5000s, not 5s
```

**Why traditional compilers miss it.** Integers are integers. Units live
in variable names and comments.

**gunbc status: GAP.**
THESIS §"Correctness dimensions" commits to user-declared dimensions
(e.g., `Duration<Second>` vs `Duration<Millisecond>` as distinct types).
The mechanism is conceptually declared; the substrate support for
dimension type parameters hasn't landed yet.

**Structural path.** Dimension as a first-class type parameter, with
conversion functions explicit at type boundaries. Implicit coercion
between different dimensions is a compile error.

**R1 status.** GAP → must close for R1. Dedicated lane (T-Dimensions, L
size).

---

### 12. Suboptimal complexity (contract violation)

**Traditional example.** Engineer writes `fn dedupe(xs)` with documented
O(n log n); three refactors later, it's O(n²). Nothing catches it until
production.

**Why traditional compilers miss it.** Complexity isn't part of the type.
Review catches it sometimes; profilers catch it in production.

**gunbc status: CE in v2, PROXY in v3.**
v2's complexity lens derives `CostExpr(work, span, asymptotic_class,
certainty)` structurally and rejects functions exceeding declared bounds.
v3's complexity lens currently produces a single integer depth per port
— structurally terminal but behaviorally weaker. Lane E is the port
program to restore v2 parity.

**Evidence:** `docs/v3-lens-capability-register.md:40-41` (PROXY
downgrade); `src/v3/lenses/complexity.dag` (v3 proxy);
`src/v2/complexity.dag` (v2 CE).

**R1 status.** v3 must reach v2 parity (Lane E, T-LaneE XL — already in
R1 scope).

---

## Group E — Effects / boundaries

### 13. Secret leak to logs

**Traditional example (Node.js).**
```javascript
console.log(`auth failed for token=${token}`);   // token now in CloudWatch
```

**Why traditional compilers miss it.** A secret is a string; `toString()`
is universal; logging accepts any stringifiable value.

**gunbc status: GAP.**
`dsl/std/types.dag:237` declares `Secret = String` as a **type alias**,
not an opaque nominal type. `dsl/std/coercion.dag:114` documents that
alias casts are identity at emit — a `Secret` value can be freely cast
back to `String` and logged.

**Structural path.** Make `Secret<T>` a nominal opaque wrapper (not an
alias). Allow construction only through authenticated sinks; disallow
coercion back to the underlying type. Logging / string-concat of a
`Secret<T>` becomes a compile error.

**R1 status.** GAP → must close for R1. Dedicated lane (T-Secret, S-M
size — type-system work, not substrate extension).

---

### 14. Unenumerated effects

**Traditional example (Python service).**
```python
@doc("reads from S3")
def process(data):
    s3.get(...)
    slack.post(...)   # actually posts to Slack; docstring is wrong
    return data
```

**Why traditional compilers miss it.** No language tracks "what this
function actually touches." Docstrings and type hints are out-of-band
from the compiler.

**gunbc status: GAP.**
`dsl/std/effects.dag` and `src/v3/std/effects.dag` have effect-shape
carriers (ReadEffect, UpsertEffect, etc.) and composition logic. But
there is **no declared-effect syntax** and **no compiler pass** that
walks a function body, aggregates effects up the call graph, and
compares to the declared set.

**Structural path.** Three pieces must land:
1. Declared-effect annotation syntax: `fn foo(...) @effects [Http, Db] ...`
2. Effect-enumeration walker: per function, aggregates effects from body
   + called functions.
3. Comparator: enumerated ≠ declared → compile error naming the delta.

**R1 status.** GAP → must close for R1. Dedicated lane (T-Effects, L
size).

---

### 15. Idempotency violation

**Traditional example (retry wrapper around workflow).**
```python
@retry(max_attempts=3)
def run_workflow():
    fetch_issues()
    summarize()
    post_to_slack(summary)   # not idempotent; duplicates on retry
```

**Why traditional compilers miss it.** Retry is a library concern.
Idempotency is a property of the body, invisible to the compiler.

**gunbc status: IBC (v3 BEHAVIORALLY COMPLETE).**
Idempotency is a property of the effect shape's algebraic structure.
`is_idempotent_effect(shape)` pattern-matches on EffectShape;
`compose_effects` checks composition. `retry_on_failure` applied to a
workflow with non-idempotent tail steps is a compile error.

**Evidence:** `dsl/std/effects.dag:108-150`;
`docs/v3-lens-capability-register.md:43` (v3 COMPLETE);
`src/v3/lenses/idempotency.dag`.

**R1 status.** Done.

---

### 16. Resource leak (file / connection not closed)

**Traditional example (Go).**
```go
file, _ := os.Open("data.txt")
// forgot to defer file.Close()
```

**Why traditional compilers miss it.** File handles are values; closing
is discipline. Linters warn sometimes.

**gunbc status: PARTIAL.**
`dsl/std/resources.dag` declares `ResourceHandle` with acquire/release
semantics (opaque-by-documentation, though see class 13 — enforcement is
weaker than intended). But there is no compile-time check that every
acquire is paired with a release.

**Structural path.** Linear/affine typing on `ResourceHandle<T>`: a
handle can be used exactly once (or once-to-release). Forgetting to
release is a "value not consumed" compile error. v2 has an ownership
lens that proves no-aliased-mutation — this extends it.

**R1 status.** PARTIAL → must close for R1. Lane (T-Resource-Ownership,
M size — extends the existing ownership lens).

---

## Group F — Injection / contract

### 17. SQL injection (string-interpolation family)

**Traditional example (everywhere).**
```python
query = f"SELECT * FROM users WHERE id = {user_id}"
cursor.execute(query)
```

**Why traditional compilers miss it.** Strings are strings; interpolation
is built into the language; compilers don't distinguish
"developer-authored literal" from "user input."

**gunbc status: IBC.**
User code cannot concatenate or interpolate strings. The surface language
has no `+` for strings, no template-literal syntax, no `.concat()` in
`dsl/std/` exposed to user programs. Database queries must be expressed
as structured data (typed operation + parameters), not string assembly.

**Note.** The same construction-level guarantee covers XSS, shell
injection, log-format-string attacks, and the broader family of
content-type-confusion bugs. All are one structural property.

---

### 18. Race condition on shared mutable state

**Traditional example (Go).**
```go
var counter int = 0
go func() { counter++ }()
go func() { counter++ }()   // data race
```

**Why traditional compilers miss it.** Most languages allow mutable
shared state; thread/async models bolt on without compile-time ownership
checks. Rust is the exception and has earned its reputation for it.

**gunbc status: IBC.**
Functions are pure by default. The language has no mutable shared cell —
`.dag` programs compose over the five L1 behaviors (Value, Transform,
Branch, Loop, Bind), none of which carry mutable references. Parallelism
is default; sequential is what requires justification. Races are
impossible because the state they would race on doesn't exist.

**Evidence:** THESIS §"Core abstraction" (parallelism default,
sequential requires data dependency); `dsl/std/computation.dag` (five L1
behaviors, no mutable state).

---

## Summary table

| # | Bug class | v2 | v3 | Gap note (if any) |
|---|---|---|---|---|
| 1 | Null dereference | IBC | IBC | — |
| 2 | Empty-list head | CE | CE | NonEmpty<T> is polish |
| 3 | Nested-optional flatten | GAP | GAP | **Cardinality refinement** |
| 4 | Force-unwrap panic | IBC | IBC | — |
| 5 | Non-exhaustive match | CE | CE | — |
| 6 | Schema drift | CE | CE | — |
| 7 | Transport / type drift | CE | CE | — |
| 8 | Array out-of-bounds | GAP | GAP | **Cardinality + bounded indices** |
| 9 | Division by zero | GAP | GAP | **Tier 2 substrate** |
| 10 | Integer overflow | GAP | GAP | **Tier 2 substrate** (or unbounded Int default) |
| 11 | Unit / dimension mismatch | GAP | GAP | **Dimension type parameters** |
| 12 | Suboptimal complexity | CE | PROXY | **Lane E port** (in R1) |
| 13 | Secret leak to logs | GAP | GAP | **Secret nominal type** (not alias) |
| 14 | Unenumerated effects | GAP | PARTIAL | **Declared-effects syntax + enumeration walker** |
| 15 | Idempotency violation | CE | COMPLETE | — |
| 16 | Resource leak | PARTIAL | PARTIAL | **Linear/affine typing on ResourceHandle** |
| 17 | SQL / shell / XSS injection | IBC | IBC | — |
| 18 | Race condition | IBC | IBC | — |

**Score.**
- **IBC:** 1, 4, 17, 18 (4/18)
- **CE:** 2, 5, 6, 7 (4/18)
- **Special CE (v3 regression):** 12 (Lane E restores)
- **PARTIAL / GAP with structural path named:** 3, 8, 9, 10, 11, 13, 14, 16 (8/18)

**Aggregate.** 9/18 fully handled today. 9/18 have named structural paths.
Zero require runtime-only mitigation.

## R1 scope implications

Per §"Governing rule" above, every GAP must close for R1. The structural
paths above name seven distinct substrate/lane additions beyond what the
R1 program committed in PR #669:

| Lane | Closes | Relative size |
|---|---|---|
| **T-Cardinality** | 3 (nested-optional), 8 (array bounds — partial) | M-L |
| **T-Tier2** | 8 (bounds), 9 (div-by-zero), 10 (overflow) | XL |
| **T-Dimensions** | 11 (unit mismatch) | L |
| **T-Secret** | 13 (secret leak) | S-M |
| **T-Effects** | 14 (unenumerated effects) | L |
| **T-Resource-Ownership** | 16 (resource leak) | M |
| **T-LaneE** | 12 (complexity port) | XL (already in R1) |

Rough aggregate of new work: one XL (Tier-2), three L/M-L, two M/S-M.

Critical-path implications:
- **T-Tier2 becomes a third XL lane** alongside T-LaneE and T-PB-A.
  Probably its own manager (tentative: M5).
- **T-Effects and T-Dimensions are L-sized** each; could be one manager
  or split across M1/M2b depending on substrate vs. lens weight.
- **T-Cardinality and T-Secret** are smaller; likely fold into M1 as
  extensions of T-Sub.
- **T-Resource-Ownership** folds into M4 (ownership discipline is
  adjacent to the typed-carrier pattern).

## Maintaining this doc

- **New bug class surfaced externally** (user report, security audit,
  design-review finding): add a row with traditional-example +
  why-compilers-miss + current status + structural path. If status is
  GAP, escalate to the ROADMAP debt ledger and R1 scope review.
- **Structural addition lands:** update status (GAP → CE / IBC), link
  the closing PR / lane.
- **v2 / v3 divergence resolved:** collapse to one status column (this
  doc currently shows both because v3 is a trajectory from v2).

This doc is parallel authority with THESIS §"Enumerable impossible-bug
classes" on the *bug-class list*. When they diverge, THESIS is the list
authority; this doc is the how/status authority. Adding a class means
touching both.
