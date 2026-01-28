# Agent Guidance for gunbc

This document helps AI agents understand the gunbc architecture and avoid common mistakes when implementing features.

## Core Principle: Graph-Time Decisions, Not Runtime Flags

**The most important pattern in this codebase**: Behavioral variations (like dry-run vs real execution) are achieved by **swapping operations at graph build time**, not by passing runtime flags through operations.

### Wrong Pattern (runtime flag)

```rust
// DON'T DO THIS
pub enum MyOp {
    WriteFile {
        path: String,
        dry_run: bool,  // ← Runtime flag inside operation
    },
}

impl Executable for MyOp {
    fn execute(&self, inputs: ...) -> ... {
        match self {
            MyOp::WriteFile { path, dry_run } => {
                if *dry_run {
                    println!("Would write to {}", path);
                } else {
                    std::fs::write(path, content)?;
                }
            }
        }
    }
}
```

### Correct Pattern (operation swapping)

```rust
// DO THIS
pub enum MyOp {
    WriteFile { path: String },      // Actually writes
    PreviewWrite { path: String },   // Just prints
}

// In graph builder:
let sink_op = if dry_run {
    MyOp::PreviewWrite { path }
} else {
    MyOp::WriteFile { path }
};
```

## The Upsert Pattern

The upsert pattern (SPEC.md §3.1) has three slots:

1. **Check** — Observes current state, outputs `present` and `needs_create`
2. **Create** — Guarded on `needs_create == true`, performs the action
3. **Resolve** — Combines results, outputs final `ok`

For dry-run support, the **Create slot** is where operation swapping happens:
- Real mode: `InstallCommand`, `WriteFile`, etc.
- Dry-run mode: `PreviewInstall`, `PrintStdout`, etc.

## Existing Implementations to Study

### gunbc-makegen (simplest)

Location: `crates/gunbc-makegen/src/graph.rs`

```rust
let sink_op = if dry_run {
    MakegenOp::PrintStdout
} else {
    MakegenOp::WriteFile
};
```

### gunbc-deps (with mode + dry-run)

Location: `crates/gunbc-deps/src/graph.rs`

```rust
let install_op = match (mode, self.dry_run) {
    (Mode::Check, _) => DepOp::FailIfMissing { name },
    (Mode::Upsert, false) => DepOp::InstallCommand { name, cmd },
    (Mode::Upsert, true) => DepOp::PreviewInstall { name, cmd },
};
```

### gunbc-gistgen (SubDAG swapping)

Location: `crates/gunbc-gistgen/src/graph.rs`

```rust
let gist_subdag = match mode {
    UnderstandingMode::Real => build_gist_real(GistgenOp::Gist),
    UnderstandingMode::Mock => build_gist_mock(GistgenOp::Gist),
};
```

## Known Deficit: Manual Threading

Currently, all implementations manually thread `dry_run` through the call chain. The ideal architecture (not yet implemented) would:

1. Have operations declare `writes_world() -> bool`
2. Have operations provide `to_preview() -> Self`
3. Have infrastructure automatically swap at build time or execution time

See `docs/postmortem-dry-run.md` for details.

## When Adding a New Tool

1. **Identify external boundaries** — What operations touch the filesystem, network, or other external state?
2. **Create paired operations** — For each world-writing operation, create a preview variant
3. **Swap at build time** — The graph builder decides which variant based on mode
4. **Don't pass flags into operations** — Operations should be "closed" — they do one thing

## Operation Classification

Document your operations by their nature:

| Classification | Description | Example |
|----------------|-------------|---------|
| **Pure** | No I/O, deterministic | `ComposeMakefile`, `Resolve` |
| **Observe** | Reads external state | `Check`, `CheckCommand` |
| **WritesWorld** | Mutates external state | `WriteFile`, `InstallCommand` |

Only **WritesWorld** operations need preview variants.

## Common Mistakes to Avoid

1. **Threading runtime flags** — Don't add `dry_run: bool` to operation structs
2. **Checking mode at execution time** — The graph should be different, not the execution logic
3. **Forgetting preview variants** — Every WritesWorld operation needs a preview counterpart
4. **Inconsistent output shapes** — Preview variants must output the same ports as real variants

## Questions to Ask Before Implementing

1. Does this feature require different graph shapes? → Operation swapping
2. Does this feature affect execution behavior? → Probably graph-time decision
3. Am I adding a flag to an operation? → Stop, reconsider the pattern
