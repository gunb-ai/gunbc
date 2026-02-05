# Codegen DAG Pipeline (Design)

## Status
Implemented in `gunbc-dag` (codegen DAG + CLI + makegen wiring). **Needs verification.**
Previously noted blockers (resolved in this branch):
- `core/ir/src/resource.rs`: `SecretString` now derives `Eq`.
- `core/codegen/src/testgen/*`: compile errors fixed (missing fields/methods, `T: Clone` bounds, missing `Clone`/`Display` impls).

Verification steps once blockers are cleared:
1) `cargo run -p gunbc-dag --bin gunbc-codegen-dag --release`
2) `cargo run -p gunbc-dag --bin gunbc-makegen --release` and confirm `make codegen` invokes codegen-dag
3) `make codegen` then `make gist-dry` (should include `--bin gunbc-gist`)

## Problem
Makefile tool targets assume generated CLI entrypoints exist under
`target/codegen/bin/*`. When those files are missing, `cargo run` fails.
We need an **upsert-style** workflow that creates missing codegen outputs
before running tools.

## Constraints
- `gunbc-codegen` is the bootstrapper and **cannot** use the transport pattern.
- The new workflow should be a DAG to keep I/O visible and interceptable.
- Avoid dependency cycles: the DAG tool must be handwritten.

## Proposed Solution
Add a **codegen DAG tool** inside `gunbc-dag` with a handwritten CLI:
`gunbc-codegen-dag`.

### Pipeline
```
PrepareCodegenExists -> Execute -> ParseCodegenExists
                                     |
                                     v
PrepareCodegenCmd  -> Execute -> ParseCodegenResult
                                     |
                                     v
PrepareStampWrite  -> Execute
```

### Existence Check
Compute expected CLI outputs from the codegen registry and check for:

`target/codegen/bin/<tool>/main.rs` for every tool in `all_tools()`.

If any are missing, `codegen_needed = true`.

### Codegen Command
Run the bootstrapper:

`cargo run -p gunbc-codegen --release -- codegen`

This avoids recursion (the DAG tool is **not** the bootstrapper).

### Stamp File
Write a stamp only on success:

`target/codegen/.codegen-stamp`

The repo already ignores `.*-stamp`, so this stays out of git.

### Makefile Integration
`make codegen` should run `gunbc-codegen-dag`.
Tool targets depend on `codegen`, ensuring the CLI files are present.

## Non-Goals (for now)
- Staleness detection beyond presence checks.
- Hashing tool registry into the stamp.
- Cross-platform shell abstraction (we assume POSIX shell for now).
