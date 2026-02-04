# Type System Evolution

**Status**: Aspirational / Design
**Date**: 2026-02-03

Longer-term directions for making the type system more structural.
These build on existing work (cardinality algebra, `TypeContract`,
`type_id == "List"` migration) but aren't blocking current work.

---

## 1. Ports reference type contracts, not raw `type_id` strings

**Current**: Ports store `type_id: String` (e.g., `"String"`, `"Bool"`,
`"Json"`). Type compatibility is checked by string equality.

**Goal**: Ports reference a `TypeContract` (or type DAG) that carries
base type + cardinality + predicates as structured data.

`TypeContract::from_type_dag` already exists (`core/ir/src/contract.rs`)
and extracts cardinality, base type, predicates, and wrapper kind from
a type DAG. This is the foundation.

**What it enables**:
- Edge compatibility becomes compositional (contract subsumption)
- Proof obligations can be generated from contracts
- CLI generation derives `repeatable` from `cardinality.max > 1`
  instead of `type_id == "List"` string match
- Testgen witness generation: given a contract, generate boundary
  values automatically (0/1/many of base type T)

**Migration path**: Start with `TypeId` newtype wrapping `String`
to make all type references greppable. Then evolve `TypeId` to carry
a reference to a type DAG / contract.

---

## 2. `Value::is_empty()` — centralized emptiness semantics

**Current**: Emptiness checks are scattered and inconsistent:
- `OutputMatcher::NonEmpty` codegen uses `as_str().map(|s| s.is_empty())`
  which is wrong for non-strings (see TODO_hacks.md Hack #3)
- Runtime `OutputMatcher::check()` has a correct match-based check
- Cardinality Empty tests use concrete empty values, not absence
  (see TODO_hacks.md Hack #5)

**Goal**: Single `Value::is_empty() -> bool` method on the `Value` enum:

```rust
impl Value {
    pub fn is_empty(&self) -> bool {
        match self {
            Value::Str(s) => s.is_empty(),
            Value::StrList(v) | Value::List(v) => v.is_empty(),
            Value::Unit | Value::Skipped => true,
            _ => false, // Int, Bool, Request, Response, Json, Map — non-empty by existence
        }
    }
}
```

**What it enables**:
- `NonEmpty` codegen becomes `assert!(!output.is_empty())`
- Input constraints can use `!value.is_empty()` for required ports
- Boundary generation: empty = `is_empty() == true` value for the type
- Contract validation: "non-empty" predicate maps to `!is_empty()`

---

## 3. Typed matcher variants (logic language for assertions)

**Current**: `OutputMatcher::Satisfies { predicate, description }` carries
a closure that can't be serialized to generated Rust code. Codegen emits
a comment instead of an assertion (see TODO_hacks.md Hack #1).

**Goal**: Replace most `Satisfies` uses with typed matcher variants that
form a finite logic language — serializable, composable, and amenable
to proof generation:

```rust
pub enum OutputMatcher {
    Exact(Value),
    Contains(String),
    NonEmpty,
    Any,

    // New typed variants:
    IsType(ValueType),           // assert!(matches!(output, Value::Bool(_)))
    IntRange { min: Option<i64>, max: Option<i64> },
    MatchesRegex(String),
    IsNonEmptyList,              // List with >=1 element
    ListLength { min: usize, max: Option<usize> },

    // Escape hatch (keep for truly custom predicates):
    Satisfies { predicate: fn(&Value) -> bool, description: String },
}
```

**What it enables**:
- Generated tests actually assert things (no more comment-only matchers)
- Matchers become a small logic you can reason about statically
- Future: generate matchers from type contracts (contract → matcher)
- Future: counterexample generation (given matcher, find failing value)

**Migration**: Audit all `Satisfies` uses, replace with typed variants
where possible. Most current uses are `is_bool`, `is_non_negative_int`,
`matches_type` patterns — these map directly to `IsType` and `IntRange`.

---

## 4. Tape order semantics

**Current**: `Value::List` uses derived `PartialEq`, which is
order-sensitive. `vec![a, b] != vec![b, a]`.

**Question**: If "tape = set/bag" is a design goal, should list equality
be order-insensitive? This affects:
- `OutputMatcher::Exact` on list values
- Edge compatibility (does `[a, b]` satisfy a port expecting `[b, a]`?)
- Testgen: generated assertions with `assert_eq!` on lists

**Options**:
1. **Lists are ordered** — current behavior. Simple, predictable.
   "Tape" is a metaphor for sequential processing, not set membership.
2. **Lists are unordered (set/bag)** — add canonicalization (sort)
   before equality checks. Need `Value: Ord` or custom comparison.
   Add "Ordered" predicate to type contracts for ports that care.
3. **Both** — `Value::List` is ordered by default. Add
   `Value::Set(BTreeSet<Value>)` for unordered collections.
   Cardinality applies to both.

**Recommendation**: Defer. Keep order-sensitive equality for now.
Only revisit if a concrete use case needs set semantics. The
current design works and avoids complexity.

---

## 5. Boundary witness generation from type contracts

**Current**: Cardinality boundary tests (Bucket B.3) hardcode mock
values via `cardinality_case_mock_value()`. Witnesses are manually
specified per type_id.

**Goal**: Given a `TypeContract`, automatically generate boundary
witnesses:

```rust
fn boundary_witnesses(contract: &TypeContract) -> Vec<Value> {
    let base_witnesses = match contract.base_type.as_deref() {
        Some("String") => vec![Value::Str("".into()), Value::Str("test".into())],
        Some("Bool")   => vec![Value::Bool(false), Value::Bool(true)],
        Some("Int")    => vec![Value::Int(0), Value::Int(1), Value::Int(-1)],
        _ => vec![],
    };

    contract.cardinality.boundary_cases().iter().flat_map(|case| {
        match case {
            Empty   => vec![/* absent — requires Hack #5 fix */],
            One     => base_witnesses.clone(),
            Many(n) => vec![Value::List(base_witnesses[..n].to_vec())],
        }
    }).collect()
}
```

**What it enables**:
- B.3 tests generated from contracts, not hardcoded tables
- Property-based testing: "for all boundary witnesses, DAG doesn't crash"
- Fuzz-like coverage with finite, deterministic test sets

**Blocked on**: Hack #5 (absence representation), `type_id == "List"`
migration (§5 in consolidation.md), `Value::is_empty()` (§2 above).

---

## 6. Cardinality-driven CLI generation

**Current**: `gunbc-dag/src/makegen/registry.rs:419` detects repeatable
CLI flags via `ep.type_id == "List"`.

**Goal**: `repeatable = cardinality.max > 1 || cardinality.max == None`
(unbounded). This falls out of the `type_id == "List"` migration
(consolidation.md §5) but is worth calling out as a concrete
improvement to CLI codegen correctness.

**What it enables**:
- CLI parsing correctness derived from the type system, not string matching
- New cardinality intervals (e.g., `[2, 5]`) automatically produce
  correct CLI constraints (repeatable with min/max validation)

---

## Tasks

- [ ] Introduce `TypeId` newtype wrapping `String` (greppable, future-proof)
- [x] Add `Value::is_empty()` method to `core/ir`
- [ ] Audit `Satisfies` uses and define typed matcher variants
- [ ] Decide on tape order semantics (document decision, defer if no use case)
- [ ] Prototype boundary witness generation from `TypeContract`
- [ ] Replace `type_id == "List"` in CLI gen with cardinality check

## Notes

- These items are aspirational, not blocking. Current system works.
- The order of priority roughly follows the dependency chain:
  `Value::is_empty()` → typed matchers → witness generation.
- `TypeContract::from_type_dag` is the key existing primitive. Most
  of these goals amount to "use contracts everywhere instead of
  raw strings."
- Don't conflate "tape semantics" with "set semantics." Tapes are
  sequential. If we want unordered collections, add them explicitly
  rather than changing list equality.
