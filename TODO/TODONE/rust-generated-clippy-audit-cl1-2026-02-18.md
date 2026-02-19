# CL1 Completion: Generated Rust Clippy Audit

Date: 2026-02-18
Task: `CL1`

## What Was Audited

Layer-1 generated Rust crates from:

- `dsl/tools/makegen.dag`
- `dsl/tools/pragma.dag`

Commands:

- `target/debug/daglang compile <module> --layer 1 --out <dir>`
- `cargo clippy --offline --manifest-path <dir>/Cargo.toml -- -D warnings`

## Result

- No clippy warnings/errors for the audited generated crates.

## Findings / Constraints

- Layer-1 Rust generation currently fails for additional modules (example:
  `dsl/tools/build.dag`) with:
  - `cannot resolve node 'tools.build::build_all' for exec-runtime`
  - root cause: missing runtime-op classification for that callable path

This is a codegen/runtime-classification support gap, not a clippy warning in
already-supported generated crates.
