# URGENT: Platform + Toolchain Modeling Gaps (Linux / GNU / QEMU)

**Status**: Active  
**Date**: 2026-02-18  
**Priority**: High
**DSL Alignment**: Canonical platform/target/toolchain model required for DSL portability
**Track**: C — Modeling Foundation

## Short Answer To The Variant Question

Today, these variants are **not** modeled thoroughly:

- `linux` is modeled in multiple incompatible ways
- `gnu` (ABI/env in target triples) is mostly not modeled at all
- `qemu` exists as hardcoded command strings, not as a first-class runtime/emulator concept

## Problem Pattern

The codebase currently has **fragmented platform models** plus **stringly-typed toolchain/runtime branches**.  
This creates repeated logic and makes it hard to add a new variant without touching many files.

## Fragmentation Map (Current State)

1. DSL platform enum: `dsl/std/types.dag` (`type Platform = Linux | MacOS | Windows`)
2. Deps runtime platform enum: `lib/tools/deps/src/platform.rs` (`Linux | Macos | Windows | Unknown`)
3. Tool satisfiability platform model: `core/ir/src/transport/tool.rs` (`PlatformDef`, `PlatformRegistry`, `linux/ubuntu/debian/alpine/macos`)
4. CI runner models:
   - `core/ir/src/transport/github_actions.rs` (`RunnerImage` with runner labels + tools, no explicit os/arch fields)
   - `core/ir/src/transport/ci/runner.rs` (`Runner` trait with string ids/tools)
5. Codegen target model: `core/daglang/daglang-driver/src/lib.rs` (`CodegenTarget = Rust|Go|C|Mips`) models language backend, not platform/ABI/runtime

## Critical Gaps

- [ ] **No first-class target-triple model (`arch-vendor-os-env`)**
  - Evidence: `CodegenTarget` only captures backend language in `core/daglang/daglang-driver/src/lib.rs`.
  - Impact: cannot represent `x86_64-unknown-linux-gnu` vs `x86_64-unknown-linux-musl` without string conventions.

- [ ] **`gnu` / ABI layer is missing**
  - Evidence: no shared enum/type for `gnu`, `musl`, `msvc`; platform enums stop at OS.
  - Impact: ABI-sensitive install/build/runtime logic stays ad-hoc.

- [ ] **`qemu`/emulator is not modeled as execution environment**
  - Evidence: MIPS parity path hardcodes `mips-linux-gnu-as`, `mips-linux-gnu-ld`, `qemu-mips` in `core/daglang/daglang-cli/tests/codegen_parity.rs`.
  - Impact: emulator support cannot be reused or reasoned about by tool/resource planning.

- [ ] **Environment layer is missing (Native vs WSL vs Container vs CI vs Emulator)**
  - Evidence: WSL/macOS/Linux branching is inline in `gunbc-dag/src/dag_viz/graph.rs` (`execute_open_browser`).
  - Impact: every feature needing environment-aware behavior repeats custom detection/branching.

- [ ] **Platform IDs are stringly-typed in install modeling**
  - Evidence:
    - `lib/tools/deps/src/manifest.rs` uses `HashMap<String, PlatformInstall>`
    - `core/ir/src/transport/github/cli.rs` returns `Vec<(&str, InstallMethod)>`
    - `lib/tools/deps/src/tool_upsert.rs` has a hardcoded PM→platform mapping marked as simplified
  - Impact: no compile-time guarantees around supported platform keys.

- [ ] **Path resolution bypasses shared platform model**
  - Evidence: `lib/transport/src/cli.rs` branches directly on `which` vs `where`.
  - Impact: host-platform behaviors are not centralized.

- [ ] **Generated test mocks hardcode linux defaults**
  - Evidence: `core/codegen/src/testgen/codegen.rs` maps `"Platform"` mocks to `"linux"`.
  - Impact: generated tests under-exercise platform variant behavior.

- [ ] **DSL layer currently encodes platform behavior as fixed shell commands**
  - Evidence: `dsl/tools/dag_viz.dag` browser service is `@shell(["xdg-open", "{path}"])`.
  - Impact: cross-platform support in DSL authoring remains non-portable.

## What To Borrow From `../the-gunbai`

- `../the-gunbai/crates/gunbai-integrations-contracts/src/understanding/rust_targets.rs`
  - first-class target-triple constants and mapping helpers
- `../the-gunbai/crates/gunbai-integrations-contracts/src/understanding/platform.rs`
  - explicit OS/arch detection + normalization assumptions/unknowns
- `../the-gunbai/crates/gunbai-integrations-contracts/src/understanding/github_actions_runner.rs`
  - structured runner spec (`os`, `distro`, `version`) instead of raw labels only

## Canonical Model Direction

- [ ] Introduce one shared platform model in `core/ir` and consume it everywhere: _(2026-02-18: foundation types landed in `core/ir/src/platform.rs`; migration/consumption still in progress.)_
  - `Arch`, `Vendor`, `Os`, `AbiEnv` (`gnu|musl|msvc|...`)
  - `TargetTriple { arch, vendor, os, env }`
  - `ExecutionEnv` (`Native`, `Wsl`, `Container`, `Ci`, `Emulator`)
  - `RuntimePlatform { host: TargetTriple, env: ExecutionEnv }`

- [ ] Model toolchain components as resources/tools, not command literals:
  - assembler, linker, runtime/emulator (`qemu-*`)

- [ ] Make install/run resolution data-driven from the canonical model:
  - no free-form platform string keys in manifests/registries

## Phased Implementation Checklist

### Phase 1: Foundation Types

- [x] Add canonical platform/target/env types in `core/ir` (single source of truth). _(2026-02-18: added `Arch`/`Vendor`/`Os`/`AbiEnv` + `TargetTriple` + `ExecutionEnv` + `RuntimePlatform`.)_
- [x] Add parsing/formatting helpers for target triples and env variants. _(2026-02-18: `TargetTriple::parse`/`Display` + enum parse/format helpers + host/env detection.)_
- [ ] Add compatibility adapters from existing enums (`deps::Platform`, DSL platform type). _(2026-02-18: `deps::Platform` adapter landed; DSL adapter pending.)_

### Phase 2: Highest-ROI Migrations

- [ ] Replace hardcoded MIPS assembler/linker/qemu strings with modeled toolchain resources
- [x] Replace inline browser open branching with environment-aware resolver utility. _(2026-02-18: moved to shared `browser_open_request(..)` in `lib/primitives/src/browser.rs` and migrated dag-viz prepare node to use it.)_
- [x] Switch deps install and GH install platform keys to typed platform IDs. _(2026-02-18: `tool_upsert` PM→platform mapping now emits typed OS tokens via canonical `Os`; `transport/github/cli.rs` now models install methods as `(Os, InstallMethod)`.)_

### Phase 3: DSL + Testgen Alignment

- [ ] Align DSL `Platform`/`CodegenTarget` vocabulary with canonical types
- [ ] Remove linux-hardcoded mock defaults in testgen and generate per-platform variants
- [ ] Add conformance tests for `linux-gnu` vs other env/ABI variants and qemu executor selection
