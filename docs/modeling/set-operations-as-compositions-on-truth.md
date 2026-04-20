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

