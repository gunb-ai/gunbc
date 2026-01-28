# gunbc (Gunbai Bytecode)

Canonical internal intermediate representation (IR) for the Gunbai system. Implements a fractal DAG model as the normalization target for V2 contract specifications.

## Project Structure

```
crates/
  gunbc-ir/        # Core types: Node, Dag, Port, Edge, NodeBody
  gunbc-validate/  # Validation: cycles, types, port saturation, pattern decisions
  gunbc-exec/      # Execution: topo sort, guards, value propagation
  gunbc-gistgen/   # Example tool: creates GitHub Gists from repo files
docs/
  v3-contracts-minimal.md   # Design rationale
  v3-worked-examples.md     # Concrete examples
  gistgen-plan.md           # Implementation plan for example tool
  ac.pdf                    # Foundational theory (Abstraction Calculus)
SPEC.md            # Formal specification
```

## Key Concepts

- **Fractal DAG**: One recursive type at all abstraction levels (L0-L3)
  - `NodeBody<T>` = `Opaque(T)` | `SubDag(Dag<T>)`
- **Explicit opt-out**: Every tool must declare `Instantiated` or `NotApplicable` for patterns
- **Typed edges**: Data flows through ports with matching `TypeId`
- **Guards**: Conditional execution via port guards; `Skipped` values propagate downstream
- **Export nodes**: SubDAGs designate which inner node's outputs become wrapper outputs

## Pipeline

```
V2 contracts -> gunbc IR -> validate() -> execute() -> ExecutionLog
```

## Build & Test

```bash
cargo build
cargo test
```

## Running gistgen Example

```bash
cargo run -p gunbc-gistgen -- --path . --glob "*.rs" --dry-run
```
