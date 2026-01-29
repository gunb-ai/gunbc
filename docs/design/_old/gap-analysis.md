# Gap Analysis: Current Implementation vs Contract Tower

> **Purpose**: Audit whether the current code actually implements the design.

---

## Current State

### L1: Cardinality ✓ IMPLEMENTED

**Status**: Working correctly

```rust
// Port builders automatically set cardinality
Port::scalar("name", "String")       // Cardinality::One
Port::optional("name", "String")     // Cardinality::ZeroOrOne
Port::list("name", "StrList")        // Cardinality::ZeroOrMore
Port::non_empty_list("name", "StrList")  // Cardinality::OneOrMore
```

**Validation**: `Cardinality::satisfies()` checks edge compatibility at compile time.

**Evidence**: `validate.rs` line ~256
```rust
if let Err(mismatch) = from_port.cardinality.check_satisfies(to_port.cardinality) {
    result.add(ValidationError::CardinalityMismatch { ... });
}
```

### L2: Type ⚠️ PARTIAL

**Status**: Type IDs exist but are just strings with no semantic content.

```rust
// Current: TypeId is just a String wrapper
pub struct TypeId(pub String);

// Usage: Types are opaque strings
port("url", "String")     // No semantic meaning
port("url", "Url")        // Also just a string, no different!
port("files", "StrList")  // Happens to be a list, but type doesn't know that
```

**Problem**: Types don't carry:
- Structure information (is `StrList` actually a list?)
- Predicates (is `Url` validated?)
- Relationship to cardinality (does `StrList` imply `ZeroOrMore`?)

**Validation**: Only string equality check.
```rust
if from_port.type_id != to_port.type_id {  // Just string comparison!
    result.add(ValidationError::TypeMismatch { ... });
}
```

### L3: Predicates ⚠️ PARTIAL (Input Only)

**Status**: `Guard` exists for input ports only, no output predicates.

```rust
// Current: Guards on input ports
pub struct Port {
    pub guard: Option<Guard>,  // Only for inputs!
}

pub enum Guard {
    Equals(Value),       // x == v
    GreaterThan(Value),  // x > v
    LessThan(Value),     // x < v
    // No general predicates!
}
```

**Problem**:
- No predicates on outputs ("I guarantee this is a valid URL")
- No predicate composition (can't say "NonEmpty AND ValidUrl")
- No entailment checking (can't prove A's output satisfies B's predicate)

### L4: Witnesses ✗ SEPARATE SYSTEM

**Status**: MockSpec exists but is disconnected from Port/Type system.

```rust
// Current: MockSpec is a separate concept
pub fn gist_mock_spec() -> MockSpec {
    MockSpec::new("gist")
        .boundary("execute_transport", "url", Value::Str("https://...".into()))
}

// Port knows nothing about this mock!
port("url", "String")  // Where's the connection to MockSpec?
```

**Problem**:
- Mocks aren't derived from types
- No automatic mock generation from type + predicates
- Contract tower levels are disconnected

---

## Gap: Types Don't Carry Contracts

**Root cause**: `TypeId` is just a string.

```rust
// What we have:
port("url", "String")  // Just a string named "String"
port("files", "StrList")  // Just a string named "StrList"

// What we need:
port("url", Type::Url)  // Type carries: base=String, predicates=[ValidUrl, NonEmpty]
port("files", Type::List(Type::String))  // Type carries: cardinality=ZeroOrMore
```

---

## Solution: Rich Type System

### Option A: Type Registry with Predicates

```rust
pub struct TypeDef {
    pub id: TypeId,
    pub base: BaseType,           // String, Int, Bool, List<T>, etc.
    pub predicates: Vec<Predicate>,
    pub default_cardinality: Cardinality,
    pub default_witnesses: Vec<Value>,
}

// Register types once
let registry = TypeRegistry::new()
    .register("Url", TypeDef {
        base: BaseType::String,
        predicates: vec![Predicate::NonEmpty, Predicate::ValidUrl],
        default_cardinality: Cardinality::One,
        default_witnesses: vec![Value::Str("https://example.com".into())],
    })
    .register("StrList", TypeDef {
        base: BaseType::List(Box::new(BaseType::String)),
        predicates: vec![],
        default_cardinality: Cardinality::ZeroOrMore,
        default_witnesses: vec![Value::StrList(vec!["a".into(), "b".into()])],
    });
```

### Option B: Structural Types (Preferred)

```rust
pub enum Type {
    // Primitives
    String,
    Int,
    Bool,
    Json,
    
    // Compound
    List(Box<Type>),
    Option(Box<Type>),
    Map(Box<Type>, Box<Type>),
    
    // Refined (type + predicates)
    Refined {
        base: Box<Type>,
        predicates: Vec<Predicate>,
    },
}

impl Type {
    /// Get the implied cardinality for this type.
    pub fn cardinality(&self) -> Cardinality {
        match self {
            Type::List(_) => Cardinality::ZeroOrMore,
            Type::Option(_) => Cardinality::ZeroOrOne,
            Type::Refined { base, predicates } => {
                if predicates.contains(&Predicate::NonEmpty) {
                    // NonEmpty list = OneOrMore
                    match base.as_ref() {
                        Type::List(_) => Cardinality::OneOrMore,
                        _ => base.cardinality(),
                    }
                } else {
                    base.cardinality()
                }
            }
            _ => Cardinality::One,  // Scalars
        }
    }
    
    /// Get predicates implied by this type.
    pub fn predicates(&self) -> Vec<Predicate> {
        match self {
            Type::Refined { predicates, .. } => predicates.clone(),
            Type::List(_) => vec![],  // No predicates, just structure
            Type::Option(_) => vec![],
            _ => vec![],  // Scalars have no inherent predicates
        }
    }
}
```

### Option C: Builder DSL (Minimal Change)

Keep `TypeId` as string but add builder helpers that set cardinality + predicates together:

```rust
// New: Typed port builders that set everything
pub fn url_port(name: &str) -> Port {
    Port::new(name, "Url")
        .with_cardinality(Cardinality::One)
        .with_predicates(vec![Predicate::NonEmpty, Predicate::ValidUrl])
        .with_witness(Value::Str("https://example.com".into()))
}

pub fn file_list_port(name: &str) -> Port {
    Port::new(name, "StrList")
        .with_cardinality(Cardinality::ZeroOrMore)
        .with_predicates(vec![Predicate::All(Box::new(Predicate::ValidPath))])
        .with_witness(Value::StrList(vec!["/path/a".into()]))
}

pub fn non_empty_file_list_port(name: &str) -> Port {
    Port::new(name, "StrList")
        .with_cardinality(Cardinality::OneOrMore)
        .with_predicates(vec![
            Predicate::NonEmpty,
            Predicate::All(Box::new(Predicate::ValidPath)),
        ])
        .with_witness(Value::StrList(vec!["/path/a".into()]))
}
```

---

## Gap: Validation Doesn't Check Predicates

**Current validation**:
```rust
// validate.rs - check_edges()
if from_port.type_id != to_port.type_id {  // L2 only
    // Error
}
if !from_port.cardinality.satisfies(to_port.cardinality) {  // L1 only
    // Error
}
// L3 predicates? Not checked!
// L4 witnesses? Not checked!
```

**Needed validation**:
```rust
// L1: Cardinality
if !from_port.cardinality.satisfies(to_port.cardinality) { ... }

// L2: Type
if !from_port.type_id.compatible_with(&to_port.type_id) { ... }

// L3: Predicates
for required in &to_port.predicates {
    match from_port.predicates.entails(required) {
        Entailment::Proven => {},
        Entailment::Refuted => { /* compile error */ },
        Entailment::Unknown => { /* generate test */ },
    }
}

// L4: Witnesses (deferred to test generation)
```

---

## Gap: MockSpec Not Derived from Types

**Current**: MockSpec is manually written for each tool.

```rust
// Manual, error-prone, can get out of sync
pub fn gist_mock_spec() -> MockSpec {
    MockSpec::new("gist")
        .boundary("execute_transport", "url", Value::Str("https://...".into()))
}
```

**Needed**: Witnesses should come from types.

```rust
// Automatic: Type carries default witness
port("url", Type::Url)  // Type::Url has default_witness

// MockSpec becomes just a collection of the witnesses from ports
pub fn mock_spec_from_dag(dag: &Dag) -> MockSpec {
    let mut spec = MockSpec::new(dag.name);
    for boundary in detect_boundaries(dag) {
        for port in boundary.outputs {
            spec.add_boundary(
                boundary.node,
                port.name,
                port.type_def.default_witness(),  // From type!
            );
        }
    }
    spec
}
```

---

## Migration Path

### Phase 1: Add Predicates to Port (Non-Breaking)

```rust
pub struct Port {
    pub name: PortName,
    pub type_id: TypeId,
    pub cardinality: Cardinality,
    pub guard: Option<Guard>,
    pub predicates: Vec<Predicate>,  // NEW
    pub witnesses: Vec<Value>,       // NEW
}
```

### Phase 2: Add Typed Port Builders

```rust
// New builders that set everything correctly
pub mod typed {
    pub fn url(name: &str) -> Port { ... }
    pub fn file_path(name: &str) -> Port { ... }
    pub fn file_list(name: &str) -> Port { ... }
    pub fn non_empty_file_list(name: &str) -> Port { ... }
    // etc.
}
```

### Phase 3: Update Validation

```rust
// Add predicate entailment to validation
fn check_edges(...) {
    // ... existing type/cardinality checks ...
    
    // NEW: Predicate checks
    check_predicate_entailment(from_port, to_port)?;
}
```

### Phase 4: Auto-Generate MockSpec

```rust
// MockSpec becomes derived from DAG types
impl<T> Dag<T> {
    pub fn mock_spec(&self) -> MockSpec {
        // Auto-generate from port witnesses
    }
}
```

---

## Summary: What's Missing

| Level | Have | Need |
|-------|------|------|
| L1 Cardinality | ✓ `satisfies()` | ✓ Done |
| L2 Type | String equality | Structural types or registry |
| L3 Predicates | Guards (input only) | Output predicates + entailment |
| L4 Witnesses | Separate MockSpec | Derived from types |

**Root cause**: Types are stringly-typed. The contract tower can't emerge because `TypeId("Url")` and `TypeId("String")` are both just strings.

**Solution**: Make types carry their contracts, then everything follows.
