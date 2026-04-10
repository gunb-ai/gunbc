# Bugs Impossible by Construction

Traditional compilers ask: "Can I convert this to machine code?" and "Does it violate local rules?" gunbc asks: "Does this line of code violate ANY structural invariant?" Many bugs that traditional compilers accept are impossible to construct in `.dag` programs — not because the compiler catches them, but because the buggy code is never written by a human. It is generated from structural declarations.

Each section below describes a category of bug, shows what happens in traditional languages, and explains how gunbc prevents it. Tests are in `src/v2/tests/src/impossible_bugs.rs`.

---

## CS-1: Impossible Typos (Generated Code)

### The bug
A developer writes `user.naem` instead of `user.name`. Or `"auth_tolen"` instead of `"auth_token"` in a JSON payload. Or `Rainey` instead of `Rainy` in a string comparison.

### Traditional compilers
Python: `AttributeError` at runtime. JavaScript: returns `undefined`, the caller crashes downstream. Go: caught at compile time for struct fields, but not for JSON keys or map lookups.

### gunbc
Field names, variant names, API paths, and JSON keys are declared **once** in `.dag` source. The emitter generates all downstream code — Rust structs, Python dataclasses, Go structs, JSON serialization, API call construction. The human never hand-writes the field name in generated code. The typo is not caught — it is **impossible to construct**.

For field access in `.dag` source itself, the compiler checks structurally: `u.naem` on a type with fields `name` and `email` produces a diagnostic at compile time.

### Code

**Rejected** — misspelled field access in `.dag`:
```dag
type User { name: String  email: String }

fn greet(u: User) -> String {
  u.naem    // FieldNotFound — 'naem' does not exist on User
}
```

**Accepted** — correct field access; emitted code uses the exact declared name:
```dag
type User { name: String  email: String }

fn greet(u: User) -> String {
  u.name    // emitter generates `pub name: String` in Rust, never `naem`
}
```

### Test evidence
`cs1_generated_field_names_match_declaration`, `cs1_misspelled_field_access_rejected` in `impossible_bugs.rs`.

---

## CS-2: Exhaustive Matches

### The bug
An enum gains a new variant. Existing `match`/`switch` statements don't handle it. At runtime, the new variant hits a default case (silent wrong behavior) or throws an unhandled exception.

### Traditional compilers
Python/JS: no match exhaustiveness checking. Go: `switch` without `default` silently falls through. Java: `switch` on strings has no exhaustiveness. Rust: catches this for its own enums, but only in Rust code.

### gunbc
Match expressions on coproduct (enum) types are checked for exhaustiveness at the infer stage. Missing variants produce `NonExhaustiveMatch` with the list of uncovered variants. This check applies regardless of target language — a Go program generated from `.dag` gets the guarantee even though Go's `switch` doesn't enforce it.

### Code

**Rejected** — missing variant:
```dag
type Status = Active | Inactive | Suspended

fn describe(s: Status) -> String {
  match s {
    Active   => "on"
    Inactive => "off"
    // NonExhaustiveMatch: missing Suspended
  }
}
```

**Accepted** — all variants covered:
```dag
type Status = Active | Inactive | Suspended

fn describe(s: Status) -> String {
  match s {
    Active    => "on"
    Inactive  => "off"
    Suspended => "paused"
  }
}
```

### Test evidence
`cs2_added_variant_breaks_existing_match` in `impossible_bugs.rs`. Also: `match_on_coproduct_missing_variant_produces_diagnostic`, `match_on_coproduct_all_variants_no_diagnostic` in `pipeline.rs`.

---

## CS-3: Termination Proofs

### The bug
A recursive function passes the wrong argument to its recursive call — the original value instead of the smaller sub-value. The function loops forever.

This is often a one-character typo. The developer meant `process(items: tail)` but wrote `process(items: items)`. Every traditional compiler accepts this.

### Traditional compilers
No mainstream compiler proves termination. Rust, Go, Java, Python — all accept `f(n) { f(n) }` without complaint. The bug manifests as a stack overflow or infinite loop in production.

### gunbc
The complexity analyzer proves **structural descent** for every recursive function. Each recursive call must pass a **strict sub-value** of the original parameter (e.g., the tail of a list, a child of a tree). If the argument is the same value (`PreservedValue`) or unknown (`SubValueUnknown`), the function gets `ComplexityUnknown` — a violation.

### Code

**Rejected** — same-argument recursion (the typo):
```dag
type IntList = Nil | Cons { head: Int, tail: IntList }

fn sum_list(items: IntList) -> Int {
  match items {
    Nil => 0
    Cons { head: h, tail: t } => h + sum_list(items: items)
    //                           typo: should be `t`, not `items`
    //                           ComplexityUnknown: same-argument recursion
  }
}
```

**Accepted** — correct structural descent:
```dag
type IntList = Nil | Cons { head: Int, tail: IntList }

fn sum_list(items: IntList) -> Int {
  match items {
    Nil => 0
    Cons { head: h, tail: t } => h + sum_list(items: t)
    //                           `t` is a strict sub-value of `items`
  }
}
```

### Test evidence
`cs3_recursive_typo_rejected`, `cs3_correct_descent_accepted` in `impossible_bugs.rs`. Also: `soundness_same_argument_stays_violation`, `cx_forever_bound_produces_violation` in `pipeline.rs`.

---

## CS-4: Bare Container Types

### The bug
A developer writes `List` when they meant `List<User>`. The container has no element type — everything is `any`/`object`/`interface{}`. Downstream code accesses `.name` on an element and gets a runtime type error.

### Traditional compilers
Python: `list` has no type parameter at runtime. JavaScript: arrays are always untyped. Go: caught at compile time (generics require parameters). Java: raw types are allowed with a warning, not an error.

### gunbc
The normalize stage checks arity: `List` expects 1 type parameter. `List` without `<T>` produces `ArityMismatch { expected: 1, got: 0 }`. There is no implicit `any` type.

### Code

**Rejected**:
```dag
type UserCache {
  entries: List       // ArityMismatch: List expects 1 type parameter
}
```

**Accepted**:
```dag
type UserCache {
  entries: List<User>  // fully instantiated
}
```

### Test evidence
`bare_container_type_detected`, `parameterized_container_no_false_positive` in `diagnostics.rs`.

---

## CS-5: Branch Type Unification

### The bug
An `if/else` expression returns different types from each branch. The caller expects one type, gets another.

### Traditional compilers
Python/JS/Ruby: both branches can return anything. The caller gets `1` or `"error"` depending on the condition — type error downstream. Go: caught at compile time (explicit return types). Rust: caught at compile time.

### gunbc
The infer stage checks that both branches of an `if/else` unify to the same type. Mismatched branches produce a `TypeMismatch` diagnostic.

### Code

**Rejected**:
```dag
fn pick(flag: Bool) -> Int {
  if flag { 1 } else { "x" }   // TypeMismatch: Int vs String
}
```

**Accepted**:
```dag
fn pick(flag: Bool) -> Int {
  if flag { 1 } else { 2 }     // both branches are Int
}
```

### Test evidence
`cs5_branches_unified_accepted` in `impossible_bugs.rs`. Also: `if_else_branch_type_mismatch` in `pipeline.rs`.

---

## CS-6: Map Key Type Mismatch

### The bug
A `Map<String, User>` is indexed with an `Int`. The lookup silently returns the wrong result or throws a runtime exception.

### Traditional compilers
JavaScript: `obj[42]` silently coerces `42` to `"42"` — potentially the wrong key. Python: `dict[42]` when keys are strings raises `KeyError` at runtime. Go: caught at compile time.

### gunbc
The infer stage checks that index expressions use the correct key type. `Map<String, Int>[42]` produces a diagnostic because the key type is `String`, not `Int`.

### Code

**Rejected**:
```dag
fn lookup(m: Map<String, Int>, id: Int) -> Int? {
  m[id]     // key type mismatch: expected String, got Int
}
```

**Accepted**:
```dag
fn lookup(m: Map<String, Int>, key: String) -> Int? {
  m[key]    // key type matches
}
```

### Test evidence
`cs6_map_wrong_key_type_rejected`, `cs6_map_correct_key_type_accepted` in `impossible_bugs.rs`. Also: `map_index_with_wrong_key_type_reports_error` in `infer_semantics.rs`.

---

## CS-7: Coercion Completeness

### The bug
A code generator targets multiple languages. When a new container type is added, someone forgets to add its mapping for one of the targets. The Go emitter silently falls back to `[]interface{}` or emits broken code.

### Traditional compilers
This isn't a compiler bug — it's a code-generator bug. Traditional multi-target generators rely on developer discipline to keep all backends in sync.

### gunbc
Every container type maps to an **algebraic inhabitant declaration** per target language. `List<T>` inhabits `FreeMonoid`, and each language declares its inhabitant: `Vec<T>` (Rust), `list[T]` (Python), `[]T` (Go). If a container has no inhabitant declaration for a target, the emitter has no template to use — **fail-closed**. There is no heuristic fallback.

The coercion test suite auto-generates assertions from all `InhabitantDecl` entries, ensuring that every declared algebra has a concrete mapping for every target language.

### Test evidence
Auto-generated coercion tests from `extract_coercion_tests()`. Per-language inhabitant declarations in `dsl/extdeps/languages/{rust,python,go}/types.dag`.

---

## CS-8: Ownership / Double-Use

### The bug
A binding (variable) is used in two consuming positions. In most languages, both consumers share a reference. If one mutates the data, the other sees corrupted state.

### Traditional compilers
Python/JS: both consumers get the same reference. Mutations from one corrupt the other. Go: slices share underlying arrays — appending in one goroutine corrupts another. Rust: caught by the borrow checker (move semantics).

### gunbc
The ownership analyzer counts semantic consumers for each binding. When a non-Copy binding (like `Map` or `List`) is consumed by multiple call sites, it's classified as `SharedError`. For fold accumulators, this makes the fold ineligible for unwrap optimization, forcing a safe (cloned) path.

### Code

**Detected** — double-consumer of `acc.data`:
```dag
type Accum { data: Map<String, Bool> }
fn process(items: List<String>) -> Accum {
  items |> fold(init: Accum { data: empty_map() }, f: (acc, item) =>
    let a = map_insert(acc.data, item, true)    // first consumer
    let b = map_insert(acc.data, item, false)   // second consumer
    Accum { data: b }
  )
}
// Ownership: acc.data has 2 consumers → ineligible for unwrap optimization
```

**Accepted** — single-consumer per field:
```dag
type Accum { table: Map<String, Int>, label: String }
fn summarize(items: List<String>) -> Accum {
  items |> fold(init: Accum { table: empty_map(), label: "" }, f: (acc, item) =>
    Accum { table: map_insert(acc.table, item, 1), label: item }
  )
}
// Ownership: each field consumed once → eligible for unwrap optimization
```

### Test evidence
`cs8_double_consumer_detected`, `cs8_single_consumer_accepted` in `impossible_bugs.rs`. Also: `fold_struct_accumulator_linear_ownership`, `fold_struct_accumulator_rejects_multi_move` in `pipeline.rs`.

---

# Integration Case Studies: Long-Distance Dependencies

The case studies above are local — one function, one type, one bug. The truly expensive bugs in production systems are **long-distance**: Team A changes a type, Module D in Team C's code breaks silently, nobody notices until a customer reports it weeks later.

These bugs survive code review (the reviewer doesn't know about the distant dependency), pass CI (test fixtures use the old shape), and only manifest in production under specific conditions.

In gunbc, all modules are compiled together. A structural change in one module propagates through the type system to every dependent module. There is no "stale import" — either the entire program is consistent, or it doesn't compile.

---

## CS-9: Schema Evolution — Field Rename Across Modules

### The bug
Team A renames `total` to `amount` in the shared `Order` type. Team B's billing code still reads `order.total`. The rename is in a PR that Team B never reviews.

### Traditional languages
Python: `order.total` raises `AttributeError` — in production, from a customer's checkout. JavaScript: returns `undefined`, which propagates silently through arithmetic (`undefined * 1.1 = NaN`), eventually corrupting an invoice. Go: caught at compile time if both modules are in the same binary, but not across microservice boundaries.

### gunbc
The compiler resolves `order.total` structurally against the imported `Order` type. After the rename, `Order` has fields `customer`, `amount`, `status` — no `total`. The field access produces a diagnostic in Module B at compile time.

### Code

**Module A** (after rename):
```dag
module types
type Order { customer: String  amount: Float  status: String }
```

**Module B** (stale — still uses old name):
```dag
module billing
import types { Order }

fn invoice_total(order: Order) -> Float {
  order.total    // FieldNotFound — field was renamed to `amount`
}
```

### The traditional equivalent (Python)
```python
# types.py — Team A's change
@dataclass
class Order:
    customer: str
    amount: float   # renamed from `total`
    status: str

# billing.py — Team B's code, unchanged
def invoice_total(order: Order) -> float:
    return order.total  # AttributeError at runtime
    # Python: no error until this line executes in production
    # mypy: catches this IF billing.py is checked AND Order is imported from types.py
    # But if Order comes from JSON deserialization? No static check at all.
```

### Test evidence
`cs9_field_rename_breaks_downstream_consumer`, `cs9_field_rename_consistent_compiles` in `impossible_bugs.rs`.

---

## CS-10: Variant Addition — Distant Match Sites Break

### The bug
Team A adds `Refunded` to `PaymentStatus`. Teams B and C both have `match`/`switch` statements on `PaymentStatus` in their modules. Neither team is aware of the new variant.

### Traditional languages
Go: `switch` without `default` silently does nothing for the new variant — the refunded payment is ignored. Python: `elif` chain falls through, no error. Java: `switch` on enum catches this, but only if you don't have a `default` case. JavaScript: no exhaustiveness checking at all.

### gunbc
Every `match` on a coproduct is checked for exhaustiveness at the infer stage. The check runs against the **current structural definition** — not a snapshot from when the consumer was written. Adding `Refunded` to the type definition causes `NonExhaustiveMatch` in **every** module that matches on `PaymentStatus` without covering it.

### Code

**Module A** (data model — 4 variants now):
```dag
module types
type PaymentStatus = Pending | Approved | Declined | Refunded
```

**Module B** (billing — only handles 3):
```dag
module billing
import types { PaymentStatus }

fn can_charge(s: PaymentStatus) -> Bool {
  match s {
    Pending  => false
    Approved => true
    Declined => false
    // NonExhaustiveMatch: missing Refunded
  }
}
```

**Module C** (reporting — also only handles 3):
```dag
module reporting
import types { PaymentStatus }

fn status_label(s: PaymentStatus) -> String {
  match s {
    Pending  => "waiting"
    Approved => "complete"
    Declined => "failed"
    // NonExhaustiveMatch: missing Refunded
  }
}
```

### The traditional equivalent (Go)
```go
// types.go — Team A's change
type PaymentStatus int
const (
    Pending PaymentStatus = iota
    Approved
    Declined
    Refunded  // new — Team A added this
)

// billing.go — Team B, unchanged
func CanCharge(s PaymentStatus) bool {
    switch s {
    case Pending:  return false
    case Approved: return true
    case Declined: return false
    }
    // Go: no error. Refunded falls through silently.
    // The refund is neither charged nor flagged — it vanishes.
    return false
}
```

### Test evidence
`cs10_variant_addition_breaks_multiple_consumers` in `impossible_bugs.rs`.

---

## CS-11: Record Literal Completeness — New Required Field

> **Status: planned guarantee.** The compiler does not yet check for missing fields in record literal construction. The test exists as `#[ignore]` to track this gap. When implemented, it will catch this class of bug at compile time.

### The bug
Someone adds `priority: Int` to the shared `Config` type. Every module that constructs a `Config` value is now missing a required field. In dynamic languages, the missing field is simply absent — no error until someone reads it, possibly in a completely different module, possibly weeks later.

### Traditional languages
Python: `Config(retries=3, timeout=30)` works if `Config` is a dataclass with a default for `priority`. Without a default, `TypeError` at construction — but only at runtime. JavaScript: missing fields are `undefined`. Go: caught at compile time (struct literals require all fields, or you get zero values — which may be silently wrong).

### gunbc (planned)
Record literal construction will be checked against the type definition. Every field in the type must appear in the literal. Missing fields will produce a diagnostic at the construction site.

### Code

**Module A** (type definition — 3 fields now):
```dag
module types
type Config { retries: Int  timeout: Int  priority: Int }
```

**Module B** (constructs Config with only 2 fields):
```dag
module consumer
import types { Config }

fn defaults() -> Config {
  Config { retries: 3, timeout: 30 }
  // Missing field: priority
}
```

### The traditional equivalent (JavaScript)
```javascript
// types.js — Team A adds priority
// (No enforcement — it's just a convention)

// consumer.js — Team B, unchanged
function defaults() {
    return { retries: 3, timeout: 30 };
    // No error. priority is missing.
    // Downstream: config.priority === undefined
    // Math: undefined * 2 === NaN
    // The NaN propagates silently through the priority queue.
}
```

### Test evidence
`cs11_new_required_field_breaks_constructor`, `cs11_complete_constructor_compiles` in `impossible_bugs.rs`.

---

## CS-12: Cross-Language Atomic Update

### The bug
A polyglot system has the same data type in Rust, Python, and Go. Someone renames a field in the Rust code but forgets to update the Python client. The services now disagree on field names, and JSON deserialization silently drops the renamed field.

### Traditional approach
Each language has its own type definition. Keeping them in sync requires discipline, code review, or a schema registry (protobuf, OpenAPI). Even with protobuf, the `.proto` file is separate from the implementations, and regeneration is a manual step that can be forgotten.

### gunbc
One `.dag` type compiles to **all** target languages atomically. Rename a field once → Rust struct, Python dataclass, and Go struct all update in the same compilation. There is no separate "sync step" to forget.

### Code

**One declaration, three targets:**
```dag
module invoices

type Invoice {
  invoice_id: String
  line_items: List<String>
  total_cents: Int
}
```

Emits:
- **Rust:** `pub struct Invoice { pub invoice_id: String, pub line_items: Vec<String>, pub total_cents: i64 }`
- **Python:** `class Invoice: invoice_id: str; line_items: list[str]; total_cents: int`
- **Go:** `type Invoice struct { InvoiceId string; LineItems []string; TotalCents int64 }`

All three use the same field names from the same declaration. There is no path for drift.

### Test evidence
`cs12_type_emits_consistently_across_all_targets` in `impossible_bugs.rs`.

---

## CS-13: Diamond Dependency — Type Identity Preserved

### The bug
Module C imports from both A and B, which both import `UserId` from a shared module. Are `a.owner` and `b.buyer` the same `UserId` type? In microservice architectures with duplicated protobuf definitions, they silently diverge — one team adds a field, the other doesn't regenerate, and the serialized bytes are incompatible.

### Traditional systems
Protobuf: each service has its own copy of the `.proto` file. Drift is possible and silent until serialization fails at runtime. TypeScript: re-exported types maintain identity within one compilation, but across packages, `UserId` from `@company/auth` and `UserId` from `@company/orders` are structurally equal but nominally different. Go: same package identity is preserved, but vendored dependencies can create distinct types with the same name.

### gunbc
The module graph deduplicates imports: `UserId` imported by A and `UserId` imported by B resolve to the same node in the module graph. There is one definition, one identity. Module C can use values from A and B interchangeably because the type is structurally the same — guaranteed by construction.

### Code

```dag
module shared
type UserId { value: String }

module a
import shared { UserId }
type AccountRef { owner: UserId }

module b
import shared { UserId }
type OrderRef { buyer: UserId }

module main
import a { AccountRef }
import b { OrderRef }

fn same_user(a: AccountRef, o: OrderRef) -> Bool {
  a.owner.value == o.buyer.value
  // This compiles: a.owner and o.buyer are the SAME UserId type.
  // No "incompatible types from different packages" error.
}
```

### Test evidence
`cs13_diamond_dependency_preserves_type_identity` in `impossible_bugs.rs`.
