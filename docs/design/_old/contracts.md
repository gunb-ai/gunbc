# Contract System Design

**Status**: Design  
**Last updated**: January 2026

---

## Overview

Contracts declare what a node requires and what it provides. The compiler
verifies all requirements are satisfied, catching wiring errors at compile
time rather than runtime.

---

## Motivation

### The Problem

Without explicit contracts, wiring errors appear at runtime:
- Node A outputs `ResourceRef`, Node B expects `ResourceState`
- Node C requires a capability that no prior node established
- Missing dependency causes cryptic failure deep in execution

### The Solution

Every node declares:
- **Requires**: What must exist before this node runs
- **Provides**: What this node establishes after running

The compiler verifies the contract graph is satisfiable.

---

## Core Types

### PrerequisiteId

A namespaced identifier for something a node requires or provides:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PrerequisiteId {
    namespace: PrerequisiteNamespace,
    name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PrerequisiteNamespace {
    /// Data dependency - a value that must be produced
    Data,
    
    /// Capability - a tool or API that must be available
    Capability,
    
    /// State - a runtime condition that must hold
    State,
    
    /// Resource - a resource that must be acquired
    Resource,
    
    /// Integration - external service access
    Integration,
    
    /// File - a file that must exist
    File,
}
```

Examples:
- `data:gist_url` - the gist URL value
- `cap:gh_cli` - the GitHub CLI tool
- `state:authenticated` - user is authenticated
- `resource:dpkg_lock` - apt/dpkg lock acquired
- `integration:github_api` - GitHub API access
- `file:Cargo.toml` - Cargo.toml exists

### NodeContract

The contract declared by a node:

```rust
#[derive(Debug, Clone, Default)]
pub struct NodeContract {
    /// What must exist before this node runs
    pub requires: Vec<PrerequisiteId>,
    
    /// What this node establishes after running
    pub provides: Vec<PrerequisiteId>,
    
    /// Resource claims (for capacity-limited resources)
    pub claims: Vec<ResourceClaim>,
    
    /// Data exports (named outputs, typed)
    pub exports: Vec<DataExport>,
    
    /// Data imports (named inputs, typed)
    pub imports: Vec<DataImport>,
}
```

### ResourceClaim

For capacity-limited resources (mutex, semaphore):

```rust
#[derive(Debug, Clone)]
pub struct ResourceClaim {
    pub resource: ResourceId,
    pub amount: usize,  // How much capacity needed
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResourceId(pub String);
```

Examples:
- `resource:dpkg_lock` with capacity 1 → mutex (one at a time)
- `resource:cpu_cores` with capacity 4 → use 4 cores

### DataExport / DataImport

Typed data flow declarations:

```rust
#[derive(Debug, Clone)]
pub struct DataExport {
    pub name: String,
    pub type_id: TypeId,
}

#[derive(Debug, Clone)]
pub struct DataImport {
    pub name: String,
    pub type_id: TypeId,
    pub from: Option<NodeId>,  // Explicit source (optional)
}
```

---

## Contract Validation

### Algorithm

```rust
fn validate_contracts(dag: &Dag) -> Result<(), Vec<ContractError>> {
    let mut errors = vec![];
    let mut provided: HashSet<PrerequisiteId> = HashSet::new();
    
    // Process in topological order
    for node_id in topo_sort(dag) {
        let node = dag.get_node(&node_id);
        let contract = node.contract();
        
        // Check all requirements are satisfied
        for req in &contract.requires {
            if !provided.contains(req) {
                errors.push(ContractError::UnsatisfiedRequirement {
                    node: node_id.clone(),
                    requires: req.clone(),
                    available: provided.clone(),
                });
            }
        }
        
        // Add what this node provides
        for prov in &contract.provides {
            provided.insert(prov.clone());
        }
    }
    
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}
```

### Error Types

```rust
#[derive(Debug)]
pub enum ContractError {
    /// A requirement is not satisfied by any prior node
    UnsatisfiedRequirement {
        node: NodeId,
        requires: PrerequisiteId,
        available: HashSet<PrerequisiteId>,
    },
    
    /// Resource claim exceeds available capacity
    InsufficientCapacity {
        node: NodeId,
        resource: ResourceId,
        requested: usize,
        available: usize,
    },
    
    /// Data import type doesn't match export type
    TypeMismatch {
        consumer: NodeId,
        import_name: String,
        expected: TypeId,
        actual: TypeId,
    },
    
    /// Circular dependency in provides/requires
    CircularDependency {
        cycle: Vec<NodeId>,
    },
}
```

---

## Contract Inference

Contracts can be inferred from edge structure (convenience, not required):

```rust
fn infer_contract(node: &Node, edges: &[Edge]) -> NodeContract {
    let mut contract = NodeContract::default();
    
    // Infer imports from incoming edges
    for edge in edges.iter().filter(|e| e.to.0 == node.id) {
        contract.imports.push(DataImport {
            name: edge.to.1.clone(),
            type_id: node.input_type(&edge.to.1),
            from: Some(edge.from.0.clone()),
        });
    }
    
    // Infer exports from outgoing edges
    for edge in edges.iter().filter(|e| e.from.0 == node.id) {
        contract.exports.push(DataExport {
            name: edge.from.1.clone(),
            type_id: node.output_type(&edge.from.1),
        });
    }
    
    contract
}
```

Explicit contracts take precedence over inferred ones.

---

## Integration with Patterns

Patterns define contract templates:

### Upsert Contract

```rust
fn upsert_contract_template() -> PatternContract {
    PatternContract {
        slots: vec![
            SlotContract {
                role: SlotRole::Check,
                requires: vec![],  // No prerequisites
                provides: vec![prereq!("state:resource_checked")],
            },
            SlotContract {
                role: SlotRole::Create,
                requires: vec![prereq!("state:resource_checked")],
                provides: vec![prereq!("state:resource_created")],
            },
            SlotContract {
                role: SlotRole::Resolve,
                requires: vec![
                    prereq!("state:resource_checked"),
                    // OR state:resource_created (guard handles this)
                ],
                provides: vec![prereq!("data:resolved_handle")],
            },
        ],
    }
}
```

When a tool instantiates Upsert, the pattern's contract constraints are
enforced on the bound implementations.

---

## Contract Display

For debugging and documentation:

```rust
impl Display for NodeContract {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if !self.requires.is_empty() {
            writeln!(f, "  requires:")?;
            for req in &self.requires {
                writeln!(f, "    - {}", req)?;
            }
        }
        if !self.provides.is_empty() {
            writeln!(f, "  provides:")?;
            for prov in &self.provides {
                writeln!(f, "    - {}", prov)?;
            }
        }
        Ok(())
    }
}
```

Output:
```
node: tool/zstd/create
  requires:
    - state:zstd_checked
    - cap:homebrew
  provides:
    - state:zstd_installed
    - data:zstd_binary_path
```

---

## Example: deps Tool

The `deps` tool with explicit contracts:

```rust
fn build_deps_graph() -> Dag<Executable> {
    DagBuilder::new()
        .node("check_brew")
            .output("installed", TypeId::bool())
            .contract(|c| c
                .provides(prereq!("state:brew_checked"))
            )
            .body(check_brew_installed())
        .node("install_brew")
            .input("needed", TypeId::bool(), guard_eq(true))
            .contract(|c| c
                .requires(prereq!("state:brew_checked"))
                .requires(prereq!("cap:curl"))
                .provides(prereq!("cap:homebrew"))
            )
            .body(install_homebrew())
        .node("install_tool")
            .input("tool_name", TypeId::string())
            .contract(|c| c
                .requires(prereq!("cap:homebrew"))
                .provides(prereq!("state:tool_installed"))
            )
            .body(brew_install())
        .edge("check_brew", "installed", "install_brew", "needed")
        .build()
}
```

Validation catches:
- If `install_tool` runs before `install_brew` (missing `cap:homebrew`)
- If `install_brew` runs before `check_brew` (missing `state:brew_checked`)

---

## Future: Effect Integration

Contracts will integrate with effect classification:

```rust
pub struct NodeContract {
    pub requires: Vec<PrerequisiteId>,
    pub provides: Vec<PrerequisiteId>,
    pub effect: Effect,           // Pure, Read, Write
    pub idempotency: Idempotency, // Idempotent, WithKey, Not
}
```

The executor uses this for:
- Parallel safety (Pure nodes always safe to parallelize)
- Retry logic (only retry Idempotent nodes)
- Dry-run behavior (intercept Write nodes)
