# Design Decisions

Canonical record of architectural choices made during gunbc development.

## 1. Import Node Design: Structural Inference (Option C)

**Decision**: No `NodeBody::Import` variant. Import boundaries are inferred structurally during lowering.

**Mechanism**: `lower.rs` identifies boundary ports by finding internal nodes with input ports that have no incoming edges within the SubDAG. These "open" ports become the SubDAG's import surface. If two internal nodes share an open port name, lowering returns `AmbiguousSourcePort` rather than guessing.

**Rationale**: Keeps `NodeBody` to exactly two variants (`Opaque` and `SubDag`), avoiding a third variant that would need special handling in every match arm. Structural inference is sufficient because contract verification already ensures port names are unambiguous before codegen produces the SubDAG builder.

## 2. Codegen Architecture: Library Crate

**Decision**: `gunbc-codegen` is a library crate that emits Rust source strings. It is not a monolithic binary or build script.

**Mechanism**: Functions like `emit_subdag_builder()` and `emit_io_structs()` take contract structs and return `String`. Per-tool contracts live in `#[cfg(test)]` modules within each tool crate (e.g., `gunbc-gistgen/src/contracts.rs`), and codegen verification runs as part of `cargo test`.

**Rationale**: Library approach allows each tool to own its contracts and verification tests without a central binary that must know about all tools. Adding a new tool means adding a new crate with its own contracts module — no changes to `gunbc-codegen` itself.

## 3. Interface Derivation: Unsatisfied Inner Inputs (Option 2)

**Decision**: Wrapper node inputs are derived from unsatisfied inner inputs, not explicitly listed in `PatternContract`.

**Mechanism**: The lowering phase (`lower.rs`) finds source ports — inner nodes whose input ports have no incoming edges — and maps them to the wrapper's input ports by matching port names. `PatternContract` declares slots, edges, and an export slot, but not wrapper inputs.

**Rationale**: Avoids redundant declarations. The inner DAG topology already implies which ports need external wiring. Declaring them again in the contract would create a synchronization burden and a new class of bugs (contract says X inputs, inner DAG actually needs Y).
