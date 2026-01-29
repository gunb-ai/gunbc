# RFC: Types as DAGs — Fractal Unification

> **Status**: Draft  
> **Goal**: Types are DAGs. Type checking is DAG composition. The contract tower emerges from structure.

---

## 0. Philosophy: Modeling Cause and Effect

### The Fundamental Insight

**We are modeling cause-and-effect systems. Causality is inherently a DAG.**

Every computation, workflow, validation, or transformation can be understood as:
- **Causes** (inputs) that flow into
- **Effects** (outputs) that result from
- **Transformations** (nodes) that connect them

This is not a design choice — it's the nature of the domain. Effects cannot precede their causes. Dependencies must be acyclic. Information flows forward through time.

### Therefore: Everything is a DAG

If causality is a DAG, then everything we model should be a DAG:

| What We Model | Causal Interpretation |
|---------------|----------------------|
| **Workflows** | Cause: inputs → Effect: outputs |
| **Types** | Cause: raw value → Effect: validated value |
| **Validation** | Cause: predicate → Effect: pass/fail |
| **Resources** | Cause: acquire → Effect: release |
| **Tests** | Cause: mock inputs → Effect: expected outputs |

### The Goal: Push Everything Into the Causal DAG

If causality is our fundamental model, we should push as much as possible into it:

| Instead of... | Push into DAG as... |
|---------------|---------------------|
| Runtime type checks | Type DAGs composed at compile time |
| Configuration files | DAG parameters |
| Test harnesses | Mock boundaries on DAGs |
| Documentation | DAG structure is self-documenting |
| Resource management | Edges carrying resource constraints |

### Examples from Our Codebase

**1. Workflow Causality (gist tool)**

```
ListFiles → FilterFiles → ReadFiles → RenderMarkdown → PrepareRequest → ExecuteTransport
   ↓            ↓            ↓             ↓                ↓                 ↓
 (cause)     (cause)      (cause)       (cause)          (cause)          (effect)
                                                                         [boundary]
```

Each node's outputs are effects of its inputs. The boundary (ExecuteTransport) is where our causal chain meets the external world.

```rust
// From lib/tools/gist/src/graph.rs
dag.add_edge(Edge::new("list_files", "files", "filter_files", "files"));
dag.add_edge(Edge::new("filter_files", "files", "read_files", "files"));
dag.add_edge(Edge::new("read_files", "contents", "render_markdown", "contents"));
// ... causality flows through edges
```

**2. Conditional Causality (upsert pattern)**

```
          ┌───────┐    ┌────────┐    ┌─────────┐
          │ Check │───▶│ Create │───▶│ Resolve │
          └───────┘    └────────┘    └─────────┘
              │            │
              └── guard ───┘
              (exists=false)
```

The guard models conditional causality: "Create" only executes if "Check" returns false. The causal relationship is explicit in the DAG structure.

```rust
// From core/ir/src/patterns/upsert.rs
dag.add_node(Node::opaque(
    "create",
    vec![
        Port::scalar("resource_id", "String"),
        Port::guarded_with_cardinality(
            "exists", "Bool", Cardinality::One,
            Guard::Eq(Value::Bool(false)),  // Conditional cause
        ),
    ],
    // ...
));
dag.add_edge(Edge::new("check", "exists", "create", "exists"));
```

**3. Cardinality is Causal Multiplicity**

Cardinality expresses how many effects a cause produces:

| Cardinality | Meaning |
|-------------|---------|
| `Zero` | No effect (signal only) |
| `One` | Single effect |
| `ZeroOrOne` | Maybe an effect |
| `ZeroOrMore` | Any number of effects |
| `OneOrMore` | At least one effect |

When we check `ZeroOrMore.satisfies(OneOrMore) = false`, we're asking: "Can a cause that might produce zero effects satisfy a requirement for at least one effect?" No.

**4. Boundaries are Causal Horizons**

Boundary detection finds where our causal model meets the external world:

```rust
// From lib/tools/deps/src/graph.rs
// Node: ExecuteInstalls (BOUNDARY - world write)
dag.add_node(Node::opaque(
    "execute_installs",
    vec![scalar("install_script", "String")],
    vec![scalar("executed", "Bool"), scalar("script", "String")],
    DepsOp::ExecuteInstalls,  // Effect crosses into the world
));
```

Inside the DAG: pure causality we control.
At boundaries: causality extends into the world (we can mock it, but can't eliminate it).

### Why Types Should Be DAGs

If types are also causal systems (raw value → validated value), they should use the same model:

```
Type "Url" = Causal Chain {
    Cause: String (raw)
        ↓
    [Validate: NonEmpty]  → Effect: non-empty string
        ↓
    [Validate: UrlPattern] → Effect: URL-shaped string
        ↓
    Effect: Url (validated)
}
```

This is not an analogy — it's the same structure. Type validation IS a causal chain. Using `Dag<TypeOp>` makes this explicit and lets us reuse all our infrastructure.

### The Fractal Principle

Because causality is self-similar at all scales:

- A **workflow** is a `Dag<WorkflowOp>`
- A **type** is a `Dag<TypeOp>`
- A **test** is a `Dag<MockOp>` (mocked boundaries)
- A **nested workflow** is a `Node::SubDag(Dag<WorkflowOp>)`
- A **nested type** is a `Node::SubDag(Dag<TypeOp>)`

All the way down. Same structure, same validation, same composition rules.

---

## 1. Core Insight

**Types are DAGs that construct validated values.**

```
Type "Url" = DAG {
    Input: String (raw)
        ↓
    [Validate: NonEmpty]
        ↓
    [Validate: UrlPattern]
        ↓
    Output: Url (validated)
}
```

When you connect `A.out:Url -> B.in:Url`:
- You're composing A's output type DAG with B's input type DAG
- If they compose, the edge is valid
- The composed DAG is the "proof" of type compatibility

---

## 2. Type DAG Specification

### 2.1 Key Principle: Types ARE Regular DAGs

**Types use the exact same `Dag<T>` infrastructure as workflows.**

```rust
// Types are just DAGs with TypeOp as the operation type
pub type TypeDag = Dag<TypeOp>;

// That's it. No separate structure.
// All Dag<T> machinery (validation, composition, lowering) applies.
```

This means:
- `validate_dag()` works on types
- SubDags work (nested types = `NodeBody::SubDag`)
- Edges, ports, cardinalities — all reused
- Type composition = DAG composition

### 2.2 TypeOp — The Only New Thing

```rust
/// Operations in a type DAG.
/// This is just another operation type like GistOp, DepsOp, etc.
pub enum TypeOp {
    /// Identity (pass-through, no validation)
    Identity,
    
    /// Validation predicate (fails if predicate returns false)
    Validate(Predicate),
    
    /// Transformation (coercion between base types)
    Transform(Coercion),
}

/// Predicates that can be validated
pub enum Predicate {
    NonEmpty,
    Matches(String),           // Regex pattern
    InRange { min: i64, max: i64 },
    All(Box<Predicate>),       // All elements satisfy (for lists)
    Custom(String),            // Named custom predicate
}

/// Coercions between base types
pub struct Coercion {
    pub from: BaseType,
    pub to: BaseType,
    // At codegen time, this becomes a function
}

/// Base types (the "shape" of data)
pub enum BaseType {
    Unit,
    Bool,
    Int,
    String,
    List(Box<BaseType>),
    Option(Box<BaseType>),
    Map(Box<BaseType>, Box<BaseType>),
    Json,
    Named(String),  // Opaque/user-defined
}
```

### 2.3 Types as Nodes with Inputs/Outputs

A type DAG has:
- **One input port**: The raw/unvalidated value
- **One output port**: The validated value
- **Internal nodes**: Validation/transformation steps

```rust
// A "Url" type is just a Dag<TypeOp>
fn url_type() -> Dag<TypeOp> {
    let mut dag = Dag::new();
    
    // Input: raw string
    dag.add_node(Node::opaque(
        "input",
        vec![],  // entrypoint
        vec![Port::scalar("raw", "String")],
        TypeOp::Identity,
    ));
    
    // Validate: non-empty
    dag.add_node(Node::opaque(
        "check_non_empty",
        vec![Port::scalar("value", "String")],
        vec![Port::scalar("value", "String")],
        TypeOp::Validate(Predicate::NonEmpty),
    ));
    
    // Validate: URL pattern
    dag.add_node(Node::opaque(
        "check_pattern",
        vec![Port::scalar("value", "String")],
        vec![Port::scalar("value", "Url")],  // Output is refined type
        TypeOp::Validate(Predicate::Matches(r"https?://.*".into())),
    ));
    
    // Wire it up
    dag.add_edge(edge("input", "raw", "check_non_empty", "value"));
    dag.add_edge(edge("check_non_empty", "value", "check_pattern", "value"));
    
    dag
}
```

### 2.4 Type Composition = DAG Composition

Since types are DAGs, type composition uses the same pattern as workflow composition:

```rust
/// Compose two type DAGs.
/// This is just DAG composition with TypeOp as the operation type.
fn compose_types(
    from_type: &Dag<TypeOp>,
    to_type: &Dag<TypeOp>,
) -> Result<Dag<TypeOp>, ValidationError> {
    // Find boundaries of from_type (output)
    let from_boundaries = detect_boundaries(from_type);
    
    // Find entrypoints of to_type (input)
    let to_entrypoints = detect_entrypoints(to_type);
    
    // Check cardinality compatibility (already implemented!)
    let from_card = from_boundaries[0].cardinality;
    let to_card = to_entrypoints[0].cardinality;
    if !from_card.satisfies(to_card) {
        return Err(ValidationError::CardinalityMismatch { ... });
    }
    
    // Compose: create SubDag containing both
    // This is the same pattern used for workflow composition!
    let mut composed = Dag::new();
    composed.add_node(Node::subdag("from", ..., from_type.clone()));
    composed.add_node(Node::subdag("to", ..., to_type.clone()));
    composed.add_edge(edge("from", "output", "to", "input"));
    
    // Validate the composed DAG (uses existing validate_dag!)
    validate_dag(&composed)?;
    
    Ok(composed)
}
```

**Key insight**: Type composition is just workflow composition where `T = TypeOp`.

---

## 3. Contract Tower from Type DAG

The contract tower **emerges** from the `Dag<TypeOp>` structure using standard DAG operations:

```rust
/// Extract contract levels from a type DAG.
/// These are just queries on a regular Dag<TypeOp>.
pub mod contract {
    use super::*;
    
    /// L1: Cardinality — from boundary port
    pub fn cardinality(type_dag: &Dag<TypeOp>) -> Cardinality {
        let boundaries = detect_boundaries(type_dag);
        boundaries.first()
            .map(|p| p.cardinality)
            .unwrap_or(Cardinality::One)
    }
    
    /// L2: Base type — from boundary port's type_id
    pub fn base_type(type_dag: &Dag<TypeOp>) -> String {
        let boundaries = detect_boundaries(type_dag);
        boundaries.first()
            .map(|p| p.type_id.0.clone())
            .unwrap_or_else(|| "Unknown".into())
    }
    
    /// L3: Predicates — from Validate nodes
    pub fn predicates(type_dag: &Dag<TypeOp>) -> Vec<Predicate> {
        type_dag.nodes.iter()
            .filter_map(|n| match &n.body {
                NodeBody::Opaque(TypeOp::Validate(p)) => Some(p.clone()),
                _ => None,
            })
            .collect()
    }
    
    /// L4: Witnesses — stored as metadata or derived
    /// (Could be node metadata, or generated from predicates)
    pub fn witnesses(type_dag: &Dag<TypeOp>) -> Vec<Value> {
        // Option 1: Store in DAG metadata
        // Option 2: Generate from predicate constraints
        // Option 3: QuickCheck-style generation
        vec![]
    }
}
```

**No new abstraction** — just functions over `Dag<TypeOp>`.

---

## 4. Integration with Workflow DAGs

### 4.1 Ports Can Reference Type DAGs

Two options for how ports reference types:

**Option A: Inline Type DAG (Embedded)**
```rust
pub struct Port {
    pub name: PortName,
    pub type_id: TypeId,           // For simple cases
    pub type_dag: Option<Dag<TypeOp>>,  // Full type DAG when needed
    pub cardinality: Cardinality,
}
```

**Option B: Named Type Registry (Recommended)**
```rust
/// Types are registered in a global/workspace registry
pub struct TypeRegistry {
    pub types: HashMap<TypeId, Dag<TypeOp>>,
}

impl TypeRegistry {
    pub fn get(&self, id: &TypeId) -> Option<&Dag<TypeOp>> {
        self.types.get(id)
    }
    
    pub fn register(&mut self, id: TypeId, dag: Dag<TypeOp>) {
        self.types.insert(id, dag);
    }
}

// Port just references by name
pub struct Port {
    pub name: PortName,
    pub type_id: TypeId,  // Looks up in registry
    pub cardinality: Cardinality,
}
```

**Why registry?** Same pattern as workflows — types can be shared, composed, reused.

### 4.2 Edge Validation = Type Composition

```rust
/// Validate an edge by composing type DAGs.
/// Uses the same validate_dag infrastructure!
fn validate_edge_with_types(
    registry: &TypeRegistry,
    from_port: &Port,
    to_port: &Port,
) -> Result<(), ValidationError> {
    // Look up type DAGs
    let from_type = registry.get(&from_port.type_id)
        .ok_or(ValidationError::UnknownType(from_port.type_id.clone()))?;
    let to_type = registry.get(&to_port.type_id)
        .ok_or(ValidationError::UnknownType(to_port.type_id.clone()))?;
    
    // Compose the type DAGs (this is just DAG composition!)
    compose_types(from_type, to_type)?;
    
    Ok(())
}
```

### 4.3 Type DAG Execution = Runtime Validation

```rust
/// Execute a type DAG to validate a value at runtime.
/// Uses the same execute() infrastructure!
fn validate_value(
    type_dag: &Dag<TypeOp>,
    value: Value,
) -> Result<Value, ExecError> {
    // Lower the type DAG (same as workflow lowering)
    let lowered = lower(type_dag)?;
    
    // Execute with the value as input
    let mut inputs = HashMap::new();
    inputs.insert("input".into(), value);
    
    // Use the same execute() function!
    execute(&lowered, inputs)
}
```

**No new execution model** — type validation is just DAG execution.

---

## 5. Built-in Type Library

The type library is just helper functions that build `Dag<TypeOp>`:

```rust
/// Type library: helper functions that build Dag<TypeOp>.
/// These use the exact same Dag/Node/Port API as workflows.
pub mod types {
    use crate::dag::{Dag, Edge, Port};
    use crate::node::Node;
    use crate::types::Cardinality;
    
    /// Helper to create an identity type (no validation)
    pub fn identity(type_name: &str) -> Dag<TypeOp> {
        let mut dag = Dag::new();
        dag.add_node(Node::opaque(
            "identity",
            vec![Port::scalar("in", type_name)],
            vec![Port::scalar("out", type_name)],
            TypeOp::Identity,
        ));
        dag.add_edge(Edge::internal("identity", "in", "identity", "out"));
        dag
    }
    
    /// Helper to create a refined type (with validation)
    pub fn refined(type_name: &str, predicates: Vec<Predicate>) -> Dag<TypeOp> {
        let mut dag = Dag::new();
        let mut prev_node = "input";
        
        // Input node
        dag.add_node(Node::opaque(
            "input",
            vec![],  // Entrypoint
            vec![Port::scalar("value", type_name)],
            TypeOp::Identity,
        ));
        
        // Chain validation nodes
        for (i, pred) in predicates.into_iter().enumerate() {
            let node_id = format!("validate_{}", i);
            dag.add_node(Node::opaque(
                &node_id,
                vec![Port::scalar("in", type_name)],
                vec![Port::scalar("out", type_name)],
                TypeOp::Validate(pred),
            ));
            dag.add_edge(Edge::new(prev_node, "value", &node_id, "in"));
            prev_node = Box::leak(node_id.into_boxed_str());  // Static lifetime for chaining
        }
        
        dag
    }
    
    // === Primitive Types ===
    
    pub fn string() -> Dag<TypeOp> { identity("String") }
    pub fn bool() -> Dag<TypeOp> { identity("Bool") }
    pub fn int() -> Dag<TypeOp> { identity("Int") }
    pub fn unit() -> Dag<TypeOp> { identity("Unit") }
    pub fn json() -> Dag<TypeOp> { identity("Json") }
    
    // === Refined Types ===
    
    pub fn non_empty_string() -> Dag<TypeOp> {
        refined("String", vec![Predicate::NonEmpty])
    }
    
    pub fn url() -> Dag<TypeOp> {
        refined("String", vec![
            Predicate::NonEmpty,
            Predicate::Matches(r"https?://.*".into()),
        ])
    }
    
    pub fn file_path() -> Dag<TypeOp> {
        refined("String", vec![
            Predicate::NonEmpty,
            Predicate::Matches(r"^[/~].*|^[a-zA-Z]:.*".into()),
        ])
    }
    
    // === Collection Types (using SubDag for element type) ===
    
    pub fn list(element_type: Dag<TypeOp>) -> Dag<TypeOp> {
        let mut dag = Dag::new();
        
        // The element type is a SubDag!
        dag.add_node(Node::subdag(
            "element",
            vec![Port::list("items", "Any")],
            vec![Port::list("validated", "Any")],
            element_type,
        ));
        
        dag
    }
    
    pub fn non_empty_list(element_type: Dag<TypeOp>) -> Dag<TypeOp> {
        let mut dag = list(element_type);
        
        // Add NonEmpty validation
        dag.add_node(Node::opaque(
            "check_non_empty",
            vec![Port::non_empty_list("items", "Any")],
            vec![Port::non_empty_list("validated", "Any")],
            TypeOp::Validate(Predicate::NonEmpty),
        ));
        
        dag
    }
}
```

### 5.1 Using Type Library in Workflows

Types are just `Dag<TypeOp>`, so they compose naturally:

```rust
use gunbc_ir::types;

// Register types in the workspace registry
fn register_types(registry: &mut TypeRegistry) {
    registry.register("String".into(), types::string());
    registry.register("Url".into(), types::url());
    registry.register("FilePath".into(), types::file_path());
    registry.register("FileList".into(), types::list(types::file_path()));
    registry.register("NonEmptyFileList".into(), types::non_empty_list(types::file_path()));
}

// Workflow nodes reference types by name
fn build_gist_graph() -> Dag<GistOp> {
    let mut dag = Dag::new();
    
    dag.add_node(Node::opaque(
        "list_files",
        vec![Port::optional("repo_path", "FilePath")],
        vec![Port::list("files", "FilePath")],  // ZeroOrMore
        GistOp::ListFiles,
    ));
    
    dag.add_node(Node::opaque(
        "read_files",
        vec![Port::non_empty_list("files", "FilePath")],  // OneOrMore
        vec![Port::scalar("contents", "Json")],
        GistOp::ReadFiles,
    ));
    
    // This edge will fail validation!
    // When we look up types and compose:
    //   FileList (ZeroOrMore) → NonEmptyFileList (OneOrMore)
    //   ZeroOrMore.satisfies(OneOrMore) = false
    dag.add_edge(edge("list_files", "files", "read_files", "files"));
    
    dag
}
```

---

## 6. Audit: Current State vs Types-as-DAGs

### 6.1 What We Have

| Component | Current | Types-as-DAGs Vision |
|-----------|---------|----------------------|
| `Dag<T>` | ✓ Complete | Reuse as `Dag<TypeOp>` |
| `Node<T>` | ✓ Complete | Reuse directly |
| `SubDag` | ✓ Complete | Reuse for nested types |
| `Port` | Has `type_id: String` | Add optional `type_dag` ref |
| `Cardinality` | ✓ Complete | Reuse directly |
| `validate_dag` | ✓ Complete | Reuse for type validation |
| `lower` | ✓ Complete | Reuse for type flattening |
| `execute` | ✓ Complete | Reuse for runtime validation |

### 6.2 Building Blocks Already Present

**✓ Everything for `Dag<TypeOp>` exists!**

```rust
// core/ir/src/dag.rs — REUSE
pub struct Dag<T> { pub nodes: Vec<Node<T>>, pub edges: Vec<Edge> }

// core/ir/src/node.rs — REUSE
pub struct Node<T> { pub id, pub inputs, pub outputs, pub body }
pub enum NodeBody<T> { Opaque(T), SubDag(Dag<T>) }

// core/ir/src/types.rs — REUSE
pub enum Cardinality { Zero, One, ZeroOrOne, ZeroOrMore, OneOrMore }
impl Cardinality { pub fn satisfies(&self, input: Cardinality) -> bool }

// core/ir/src/validate.rs — REUSE
pub fn validate_dag<T>(dag: &Dag<T>) -> Result<(), ValidationResult>

// core/exec/src/lower.rs — REUSE
pub fn lower<T>(dag: &Dag<T>) -> Result<Dag<T>, LowerError>

// core/exec/src/execute.rs — REUSE (for runtime validation)
pub fn execute<T: Executable>(dag: &Dag<T>, inputs) -> Result<outputs, ExecError>
```

### 6.3 What's Missing (Minimal!)

**Only these new types needed:**

```rust
// 1. TypeOp enum (~30 lines)
pub enum TypeOp {
    Identity,
    Validate(Predicate),
    Transform(Coercion),
}

// 2. Predicate enum (~50 lines)
pub enum Predicate {
    NonEmpty,
    Matches(String),
    InRange { min: i64, max: i64 },
    All(Box<Predicate>),
    Custom(String),
}

// 3. Coercion struct (~20 lines)
pub struct Coercion {
    pub from: BaseType,
    pub to: BaseType,
}

// 4. BaseType enum (~30 lines) — or derive from Value
pub enum BaseType {
    Unit, Bool, Int, String,
    List(Box<BaseType>),
    Option(Box<BaseType>),
    Map(Box<BaseType>, Box<BaseType>),
    Json, Named(String),
}

// 5. TypeRegistry (~30 lines)
pub struct TypeRegistry {
    pub types: HashMap<TypeId, Dag<TypeOp>>,
}

// 6. Type library helpers (~100 lines)
pub mod types {
    pub fn string() -> Dag<TypeOp> { ... }
    pub fn url() -> Dag<TypeOp> { ... }
    // etc.
}
```

**Total new code: ~260 lines** (not 1300!)

### 6.4 Why So Little?

Because types ARE just DAGs with a specific operation type:

```rust
pub type TypeDag = Dag<TypeOp>;  // That's it!
```

All existing infrastructure applies:
- `validate_dag(&my_type)` — validates type DAG structure
- `lower(&my_type)` — flattens nested type DAGs
- `detect_boundaries(&my_type)` — finds output type
- `detect_entrypoints(&my_type)` — finds input type
- Edge validation — already has cardinality checking

### 6.5 Migration Strategy (Simplified)

**Phase 1**: Add TypeOp and type library (non-breaking)
```rust
// New file: core/ir/src/type_op.rs
pub enum TypeOp { Identity, Validate(Predicate), Transform(Coercion) }
pub enum Predicate { ... }

// New file: core/ir/src/type_lib.rs
pub mod types {
    pub fn string() -> Dag<TypeOp> { ... }
}
```

**Phase 2**: Add TypeRegistry (non-breaking)
```rust
// core/ir/src/registry.rs
pub struct TypeRegistry {
    pub types: HashMap<TypeId, Dag<TypeOp>>,
}
```

**Phase 3**: Add optional type lookup to validation
```rust
// In validate.rs, optionally compose types if registry provided
fn validate_dag_with_types<T>(
    dag: &Dag<T>,
    registry: Option<&TypeRegistry>,
) -> Result<(), ValidationResult> { ... }
```

**Phase 4**: Migrate tools incrementally
- Register types in each tool's graph module
- No breaking changes to existing code

### 6.6 Distance Summary

| Category | Status | New Code |
|----------|--------|----------|
| DAG infrastructure | ✓ 100% complete | 0 lines |
| Validation | ✓ 100% complete | 0 lines |
| Execution | ✓ 100% complete | 0 lines |
| TypeOp enum | ✗ Missing | ~30 lines |
| Predicate enum | ✗ Missing | ~50 lines |
| BaseType enum | ✗ Missing | ~30 lines |
| TypeRegistry | ✗ Missing | ~30 lines |
| Type library | ✗ Missing | ~100 lines |
| Impl `Executable` for `TypeOp` | ✗ Missing | ~50 lines |
| **Total** | | **~290 lines** |

**We're ~95% there.** The DAG infrastructure is complete; we just need the TypeOp operation type and helpers.

---

## 7. Summary

### Core Principle

**Types are `Dag<TypeOp>`, not a separate abstraction.**

This means:
- All DAG infrastructure (validate, lower, execute, boundary detection) works on types
- Type composition = DAG composition
- Type validation = DAG execution
- Contract tower = queries on type DAG structure

### What This Gives Us

| Feature | Benefit |
|---------|---------|
| Unified model | Types and workflows use the same `Dag<T>` |
| Zero duplication | Reuse validate, lower, execute, detect_* |
| Emergent contracts | L1-L4 tower from `Dag<TypeOp>` structure |
| Composable types | SubDag for nested types (List<Url>, etc.) |
| Runtime validation | Execute type DAGs to validate values |
| Fractal all the way | Workflows, types, predicates — all DAGs |

### Current Distance

| What | Status |
|------|--------|
| DAG infrastructure | ✓ 100% complete |
| TypeOp + Predicate | ✗ ~80 lines to add |
| Type library helpers | ✗ ~100 lines to add |
| TypeRegistry | ✗ ~30 lines to add |
| Executable impl | ✗ ~50 lines to add |
| **Total** | **~260 lines** |

### Recommended Next Steps

1. **Add `TypeOp` enum** in `core/ir/src/type_op.rs`
2. **Add `Predicate` enum** in same file
3. **Add `types::*` library** with helper functions
4. **Implement `Executable` for `TypeOp`** (for runtime validation)
5. **Add `TypeRegistry`** for named type lookup
6. **Migrate one tool** (gist) as proof of concept
7. **Iterate** based on experience

### Non-Goals

- No separate "TypeDag" struct — types are just `Dag<TypeOp>`
- No duplicate validation logic — reuse `validate_dag`
- No duplicate execution logic — reuse `execute`
- No parallel hierarchy — types are a veneer over DAGs
