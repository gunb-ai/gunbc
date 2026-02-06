# URGENT: Codegen Upsert Key Missing

**Status**: Done (2026-02-06)
**Date**: 2026-02-05
**Outcome**: Manifest-based freshness for codegen inputs/outputs is implemented (resource manifest updates in `core/codegen/src/main.rs`, freshness checks in `gunbc-dag/src/codegen/ops.rs`).

## Problem Statement

The CI pipeline's codegen existence check is broken. It currently checks for an arbitrary file (`target/codegen/bin/deps/main.rs`) to determine if codegen has run. This is:

1. **Brittle** - file choice is arbitrary, breaks if that tool is renamed/removed
2. **Incomplete** - doesn't verify ALL generated files exist
3. **Stale-blind** - can't detect when inputs changed and outputs need regeneration

This is fundamentally an **upsert problem**: we want to generate outputs if they don't exist OR if inputs have changed. Without a proper upsert key, we either:
- Skip codegen when we shouldn't (stale outputs)
- Run codegen when we don't need to (wasted CI time)

## Current Behavior

```
PrepareCodegenExistsCheck: "does target/codegen/bin/deps/main.rs exist?"
  → yes: skip codegen
  → no: run codegen
```

## Desired Behavior

```
ComputeInputHash: hash(registry files, graph sources, templates, codegen tool version)
CheckManifest: "does manifest exist AND manifest.input_hash == computed_hash?"
  → yes: skip codegen (outputs are fresh)
  → no: run codegen, write new manifest
```

## Suggested Solutions

### Option 1: Content Hash Manifest

Codegen computes a hash of its input files and writes a manifest:

```
target/codegen/.manifest.json
{
  "input_hash": "sha256:abc123...",
  "codegen_version": "0.1.0",
  "generated_at": "2026-02-05T10:00:00Z",
  "files": [
    "bin/deps/main.rs",
    "bin/gist/main.rs",
    ...
  ]
}
```

CI check: compute current input hash, compare to manifest hash.

**Pros**: Precise, handles any input change
**Cons**: Need to enumerate all input files, hash computation overhead

### Option 2: Git-based Key

Use git tree hash of the directories containing codegen inputs:

```rust
let key = git_tree_hash(&["core/codegen/", "gunbc-dag/src/*/registry.rs"]);
```

**Pros**: Simple, leverages git's content addressing
**Cons**: Only works in git repos, doesn't catch uncommitted changes

### Option 3: Cargo-style Fingerprinting

Similar to how Cargo tracks when to rebuild - hash of (source files + compiler version + flags).

**Pros**: Battle-tested pattern
**Cons**: More complex to implement

### Option 4: Always Run, Let Codegen Short-Circuit

Remove the CI-level check. Codegen itself checks if outputs are fresh and exits early.

**Pros**: Single source of truth (codegen owns its freshness logic)
**Cons**: Still need freshness logic somewhere

## Recommendation

**Option 1 (Content Hash Manifest)** is the most robust. Implementation:

1. Add `ComputeCodegenInputHash` op that hashes relevant source files
2. Codegen writes `.manifest.json` with the hash after successful generation
3. CI check becomes: "compute hash, compare to manifest, run if different"

This follows the resource acquisition pattern - the manifest IS the upsert key, and the hash IS the version.

## Files Involved

- `gunbc-dag/src/ci/ops.rs` - current broken check (`execute_prepare_codegen_exists_check`)
- `gunbc-dag/src/ci/graph.rs` - CI graph structure
- `core/codegen/src/main.rs` - codegen entry point (would write manifest)
- New: manifest schema, hash computation logic

## Temporary Workaround (Current)

Checking for `target/codegen/bin/deps/main.rs` - works but brittle. Acceptable short-term because the GitHub Actions workflow runs codegen unconditionally before the CI DAG anyway.
