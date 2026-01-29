# RFC: Contract Tower — Multi-Level Node Boundary Specifications

> **Status**: Draft  
> **Goal**: Define epistemic/ontological contracts at node boundaries across multiple abstraction levels, all verifiable at compile/test time.

---

## 1. Core Insight

Every node boundary should have a **contract tower** — specifications at multiple levels of abstraction that together describe "what you're getting into":

```
                    ┌─────────────────┐
                    │    BEHAVIORAL   │  ← Mock values, examples
                    │   (witnesses)   │
                    ├─────────────────┤
                    │     LOGICAL     │  ← Predicates, invariants
                    │  (properties)   │
                    ├─────────────────┤
                    │      TYPE       │  ← Shape, structure
                    │   (what kind)   │
                    ├─────────────────┤
                    │  SET-THEORETIC  │  ← Cardinality, existence
                    │   (how many)    │
                    └─────────────────┘
```

**Key property**: Each level *entails* the one below it. A witness (mock value) proves the predicate, which proves the type, which proves the cardinality.

---

## 2. The Four Levels

### Level 1: Set-Theoretic (Cardinality)

**Question**: How many values can flow through this port?

```
Cardinality := Zero | One | ZeroOrOne | ZeroOrMore | OneOrMore
```

**What it proves**:
- Existence guarantees (will there be a value?)
- Emptiness possibilities (might it be empty?)
- Multiplicity bounds (at most one? at least one?)

**Verification**: Compile-time (current `satisfies()`)

### Level 2: Type (Structure)

**Question**: What shape/kind of value is it?

```
Type := String | Int | Bool | List<T> | Map<K,V> | Json | ...
```

**What it proves**:
- Structural compatibility (can A connect to B?)
- Coercion requirements (needs conversion?)
- Memory layout (for codegen)

**Verification**: Compile-time (current `TypeId` matching)

### Level 3: Logical (Predicates)

**Question**: What properties does the value satisfy?

```
Predicate := 
  | NonEmpty                      -- length > 0
  | ValidUrl                      -- matches URL pattern
  | InRange(lo, hi)               -- lo ≤ x ≤ hi
  | Matches(Regex)                -- matches pattern
  | All(P)                        -- ∀x ∈ list. P(x)
  | Implies(P, Q)                 -- P → Q
  | Custom(name, fn)              -- user-defined
```

**What it proves**:
- Domain constraints (valid URL, positive number)
- Relational properties (sorted, unique)
- Invariants (non-empty after filter? depends on input)

**Verification**: Compile-time (predicate entailment) + Test-time (property testing)

### Level 4: Behavioral (Witnesses)

**Question**: What are concrete example values?

```
Witness := Value  -- A concrete value that satisfies all lower levels
```

**What it proves**:
- Satisfiability (the predicates are achievable)
- Realism (the mock represents actual behavior)
- Chain compatibility (A's output works as B's input)

**Verification**: Test-time (mock spec tests, chain validation)

---

## 3. Contract Tower Type

```rust
/// A complete contract for a port boundary.
pub struct PortContract {
    /// L1: Set-theoretic — how many values?
    pub cardinality: Cardinality,
    
    /// L2: Type — what shape?
    pub type_id: TypeId,
    
    /// L3: Logical — what properties?
    pub predicates: Vec<Predicate>,
    
    /// L4: Behavioral — example values?
    pub witnesses: Vec<Witness>,
}

/// A witness is a value with its provenance.
pub struct Witness {
    pub value: Value,
    pub scenario: String,  // "happy path", "empty input", "error case"
}
```

---

## 4. Entailment Relations

### 4.1 Witness Entails Predicate

A witness `w` entails predicate `P` iff `P(w) = true`.

```
entails_predicate : Witness → Predicate → Bool
entails_predicate(w, P) = P.check(w.value)
```

**Compile-time check**: For each witness, verify all predicates hold.

### 4.2 Predicate Entails Type

A predicate `P` entails type `T` iff all values satisfying `P` have type `T`.

```
entails_type : Predicate → Type → Bool
entails_type(ValidUrl, String) = true      -- URLs are strings
entails_type(InRange(0, 100), Int) = true  -- Range implies int
entails_type(NonEmpty, List<T>) = true     -- NonEmpty implies list
```

**Compile-time check**: Predicate domain ⊆ type domain.

### 4.3 Type Entails Cardinality

A type `T` entails cardinality `C` based on the type's structure.

```
entails_cardinality : Type → Cardinality → Bool
entails_cardinality(String, One) = true           -- Scalar
entails_cardinality(Option<T>, ZeroOrOne) = true  -- Optional
entails_cardinality(List<T>, ZeroOrMore) = true   -- List
entails_cardinality(NonEmptyList<T>, OneOrMore) = true  -- Non-empty
```

**Compile-time check**: Type structure implies cardinality bounds.

### 4.4 Contract Composition

Two contracts compose (A's output → B's input) iff each level satisfies:

```
composes : PortContract → PortContract → Bool
composes(A, B) = 
    A.cardinality ⊑ B.cardinality ∧           -- L1
    A.type_id = B.type_id ∧                   -- L2
    (∀P ∈ B.predicates. A.predicates ⊢ P) ∧  -- L3
    (∀w ∈ A.witnesses. B.accepts(w))          -- L4
```

---

## 5. Predicate Logic

### 5.1 Core Predicates

```rust
pub enum Predicate {
    // Cardinality-level (L1)
    NonEmpty,                    // |x| > 0
    
    // Type-level (L2)
    IsType(TypeId),              // x : T
    
    // Value-level (L3)
    Equals(Value),               // x = v
    InRange { lo: i64, hi: i64 },// lo ≤ x ≤ hi
    Matches(String),             // x ~ regex
    StartsWith(String),          // x.starts_with(s)
    
    // Collection-level (L3)
    All(Box<Predicate>),         // ∀y ∈ x. P(y)
    Any(Box<Predicate>),         // ∃y ∈ x. P(y)
    Unique,                      // all elements distinct
    Sorted,                      // elements in order
    Length { min: usize, max: Option<usize> },
    
    // Relational (L3)
    SubsetOf(Vec<Value>),        // x ⊆ S
    SupersetOf(Vec<Value>),      // x ⊇ S
    
    // Logical combinators
    And(Vec<Predicate>),         // P₁ ∧ P₂ ∧ ...
    Or(Vec<Predicate>),          // P₁ ∨ P₂ ∨ ...
    Not(Box<Predicate>),         // ¬P
    Implies(Box<Predicate>, Box<Predicate>),  // P → Q
    
    // Custom
    Custom { name: String, check: fn(&Value) -> bool },
}
```

### 5.2 Predicate Entailment

```
⊢ : Vec<Predicate> → Predicate → Bool

-- Basic rules
[P, ...] ⊢ P                                    -- Assumption
Ps ⊢ P, Ps ⊢ Q  ⟹  Ps ⊢ And([P, Q])            -- And-intro
Ps ⊢ And([P, Q])  ⟹  Ps ⊢ P                    -- And-elim
Ps ⊢ P  ⟹  Ps ⊢ Or([P, Q])                     -- Or-intro

-- Domain rules
[NonEmpty] ⊢ Length { min: 1, max: None }       -- NonEmpty implies length ≥ 1
[InRange(0, 100)] ⊢ InRange(0, 200)             -- Narrower implies wider
[All(P)] ⊢ Any(P)  when NonEmpty                -- If all satisfy and non-empty, some satisfy
[Matches("https://.*")] ⊢ StartsWith("https")   -- Regex implies prefix
```

### 5.3 Predicate Checking

```rust
impl Predicate {
    /// Check if a value satisfies this predicate.
    pub fn check(&self, value: &Value) -> bool {
        match self {
            Predicate::NonEmpty => !value.is_empty(),
            Predicate::InRange { lo, hi } => {
                value.as_int().map(|i| *lo <= i && i <= *hi).unwrap_or(false)
            }
            Predicate::All(p) => {
                value.as_list().map(|xs| xs.iter().all(|x| p.check(x))).unwrap_or(false)
            }
            Predicate::And(ps) => ps.iter().all(|p| p.check(value)),
            Predicate::Or(ps) => ps.iter().any(|p| p.check(value)),
            Predicate::Not(p) => !p.check(value),
            // ...
        }
    }
    
    /// Check if this predicate entails another (compile-time).
    pub fn entails(&self, other: &Predicate) -> Entailment {
        // Returns Proven, Refuted, or Unknown
        // Unknown requires test-time verification
    }
}
```

---

## 6. Example: URL Output Contract

```rust
// A node that produces URLs
let url_output_contract = PortContract {
    // L1: Always produces exactly one URL
    cardinality: Cardinality::One,
    
    // L2: It's a string
    type_id: TypeId::from("String"),
    
    // L3: It satisfies these predicates
    predicates: vec![
        Predicate::NonEmpty,
        Predicate::StartsWith("https://"),
        Predicate::Matches(r"https://[a-z]+\.[a-z]+/.*"),
    ],
    
    // L4: Example values (witnesses)
    witnesses: vec![
        Witness {
            value: Value::Str("https://gist.github.com/abc123".into()),
            scenario: "happy path".into(),
        },
        Witness {
            value: Value::Str("https://api.github.com/error".into()),
            scenario: "error response".into(),
        },
    ],
};
```

---

## 7. Contract Propagation Through DAG

### 7.1 Node Contract

A node has input and output contracts:

```rust
pub struct NodeContract {
    pub inputs: HashMap<PortName, PortContract>,
    pub outputs: HashMap<PortName, PortContract>,
    
    /// How outputs relate to inputs (optional, for propagation)
    pub transfer: Option<TransferFunction>,
}

/// Describes how input properties transfer to outputs.
pub enum TransferFunction {
    /// Output inherits input cardinality
    PreservesCardinality,
    /// Output may reduce cardinality (filter)
    MayReduce,
    /// Output always non-empty if input non-empty
    PreservesNonEmpty,
    /// Custom transfer logic
    Custom(fn(&[&PortContract]) -> PortContract),
}
```

### 7.2 Edge Validation

An edge is valid iff contracts compose at all levels:

```rust
pub fn validate_edge(
    from_contract: &PortContract,
    to_contract: &PortContract,
) -> EdgeValidation {
    let mut errors = vec![];
    
    // L1: Cardinality
    if !from_contract.cardinality.satisfies(to_contract.cardinality) {
        errors.push(EdgeError::CardinalityMismatch { ... });
    }
    
    // L2: Type
    if from_contract.type_id != to_contract.type_id {
        errors.push(EdgeError::TypeMismatch { ... });
    }
    
    // L3: Predicates
    for required in &to_contract.predicates {
        match from_contract.predicates.entails(required) {
            Entailment::Proven => {},
            Entailment::Refuted => errors.push(EdgeError::PredicateViolation { ... }),
            Entailment::Unknown => {
                // Defer to test-time
                errors.push(EdgeError::PredicateUnproven { ... });
            }
        }
    }
    
    // L4: Witnesses (test-time)
    for witness in &from_contract.witnesses {
        if !to_contract.accepts_witness(witness) {
            errors.push(EdgeError::WitnessRejected { ... });
        }
    }
    
    EdgeValidation { errors }
}
```

---

## 8. Verification Timeline

| Level | When Verified | How |
|-------|--------------|-----|
| L1: Cardinality | Compile | `satisfies()` lattice check |
| L2: Type | Compile | `TypeId` equality |
| L3: Predicates | Compile (if provable) | Entailment rules |
| L3: Predicates | Test (if unknown) | Property-based testing |
| L4: Witnesses | Test | Mock chain validation |

### 8.1 Compile-Time Guarantees

If predicate entailment is **proven**, we get a compile-time guarantee:

```rust
// Compile error: NonEmpty cannot satisfy All(ValidUrl) without proof
dag.add_edge(edge("filter", "out", "validate", "in"));
//                 ↑ predicates: [NonEmpty]
//                                          ↑ requires: [All(ValidUrl)]
// ERROR: Cannot prove filter output satisfies validate input predicate
```

### 8.2 Test-Time Guarantees

If predicate entailment is **unknown**, we generate tests:

```rust
// Generated test:
#[test]
fn test_filter_output_satisfies_validate_input() {
    let filter_witnesses = filter_mock_spec().witnesses;
    let validate_contract = validate_input_contract();
    
    for witness in filter_witnesses {
        assert!(
            validate_contract.accepts(&witness.value),
            "Filter witness {:?} rejected by validate",
            witness.scenario
        );
    }
}
```

---

## 9. Unification with MockSpec

The current `MockSpec` becomes a contract tower:

```rust
// Current (behavioral only):
pub fn gist_mock_spec() -> MockSpec {
    MockSpec::new("gist")
        .boundary("execute_transport", "url", 
            Value::Str("https://gist.github.com/mock/123".into()))
}

// New (full contract tower):
pub fn gist_contracts() -> NodeContract {
    NodeContract::new("gist")
        .output("execute_transport", "url", PortContract {
            cardinality: Cardinality::One,
            type_id: TypeId::from("String"),
            predicates: vec![
                Predicate::NonEmpty,
                Predicate::StartsWith("https://gist.github.com/"),
            ],
            witnesses: vec![
                Witness::new("https://gist.github.com/mock/123", "success"),
                Witness::new("https://gist.github.com/error/500", "api_error"),
            ],
        })
}
```

**Benefits**:
- Witnesses are still mocks (L4 compatibility)
- Predicates enable compile-time checking (L3)
- Type and cardinality unchanged (L1, L2)

---

## 10. Summary: What You Know at Each Level

| Level | What You Know | Example |
|-------|--------------|---------|
| L1 | "There will be at least one value" | `OneOrMore` |
| L2 | "It will be a string" | `String` |
| L3 | "It will be a valid HTTPS URL" | `StartsWith("https://")` |
| L4 | "It will look like this" | `"https://gist.github.com/abc"` |

**Epistemic hierarchy**: L4 ⟹ L3 ⟹ L2 ⟹ L1

If you have a witness, you know everything. If you only have cardinality, you know the least.

---

## 11. Implementation Path

1. **Add predicates to ports** (backward compatible)
2. **Implement predicate entailment** (basic rules)
3. **Generate tests for unproven predicates**
4. **Unify MockSpec with PortContract**
5. **Add transfer functions for predicate propagation**

---

## Appendix: Predicate Entailment Rules

```
────────────────────────────────────
         BASIC RULES
────────────────────────────────────

Γ, P ⊢ P                                    [Assumption]

Γ ⊢ P    Γ ⊢ Q
──────────────────                           [And-Intro]
Γ ⊢ And(P, Q)

Γ ⊢ And(P, Q)
──────────────────                           [And-Elim-L]
Γ ⊢ P

Γ ⊢ P
──────────────────                           [Or-Intro-L]
Γ ⊢ Or(P, Q)


────────────────────────────────────
         DOMAIN RULES
────────────────────────────────────

Γ ⊢ NonEmpty
──────────────────────────────────           [NonEmpty-Length]
Γ ⊢ Length { min: 1, max: None }

Γ ⊢ InRange(a, b)    a ≤ c    d ≤ b
──────────────────────────────────           [Range-Widen]
Γ ⊢ InRange(c, d)

Γ ⊢ Matches(r₁)    r₁ ⊆ r₂
──────────────────────────────────           [Regex-Subset]
Γ ⊢ Matches(r₂)

Γ ⊢ StartsWith(s)
──────────────────────────────────           [Prefix-NonEmpty]
Γ ⊢ NonEmpty

Γ ⊢ All(P)    Γ ⊢ NonEmpty
──────────────────────────────────           [All-Any]
Γ ⊢ Any(P)
```
