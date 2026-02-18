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

1. DSL platform enum: `dsl/std/types.dag` (`type Platform = Linux | Macos | Windows`)
2. Deps runtime platform enum: `lib/tools/deps/src/platform.rs` (`Linux | Macos | Windows | Unknown`)
3. Tool satisfiability platform model: `core/ir/src/transport/tool.rs` (`PlatformDef`, `PlatformRegistry`, `linux/ubuntu/debian/alpine/macos`)
4. CI runner models:
   - `core/ir/src/transport/github_actions.rs` (`RunnerImage` with runner labels + tools, no explicit os/arch fields)
   - `core/ir/src/transport/ci/runner.rs` (`Runner` trait with string ids/tools)
5. Codegen target model: `core/daglang/daglang-driver/src/lib.rs` (`CodegenTarget = Rust|Go|C|Mips`) models language backend, not platform/ABI/runtime

## Critical Gaps

- [x] **No first-class target-triple model (`arch-vendor-os-env`)**
  - Update (2026-02-18): added canonical `TargetTriple` + `Arch`/`Vendor`/`Os`/`AbiEnv` in `core/ir/src/platform.rs`.
  - Impact reduction: target/ABI variants are now representable in shared typed model.

- [x] **`gnu` / ABI layer is missing**
  - Update (2026-02-18): shared `AbiEnv` enum added (`gnu`, `musl`, `msvc`, etc.).
  - Impact reduction: ABI-sensitive behavior now has a common typed vocabulary.

- [x] **`qemu`/emulator is not modeled as execution environment**
  - Update (2026-02-18): `ExecutionEnv::Emulator` and `ToolchainCommands::mips_linux_gnu()` added; parity path now consumes modeled toolchain commands.
  - Impact reduction: emulator-related behavior is now modeled centrally instead of string literals.

- [x] **Environment layer is missing (Native vs WSL vs Container vs CI vs Emulator)**
  - Update (2026-02-18): shared `ExecutionEnv` + `RuntimePlatform` detection path added and used by browser-open resolver.
  - Impact reduction: environment-aware resolution is now centralized and reusable.

- [ ] **Platform IDs are stringly-typed in install modeling**
  - Evidence:
    - `lib/tools/deps/src/manifest.rs` uses `HashMap<String, PlatformInstall>`
    - `core/ir/src/transport/github/cli.rs` now uses `Vec<(Os, InstallMethod)>` *(improved)*
    - `lib/tools/deps/src/tool_upsert.rs` now maps PM→canonical `Os` tokens *(improved)*
    - `core/ir/src/transport/tool.rs` now uses typed `PlatformId` keys *(improved)*
  - Impact: no compile-time guarantees around supported platform keys.

- [ ] **Path resolution bypasses shared platform model**
  - Evidence: `lib/transport/src/cli.rs` branches directly on `which` vs `where`.
  - Impact: host-platform behaviors are not centralized.

- [x] **Generated test mocks hardcode linux defaults**
  - Update (2026-02-18): testgen platform mocks now derive host-first canonical variants and cycle across linux/macos/windows seeds (`platform_mock_token` in `core/codegen/src/testgen/codegen.rs`).
  - Impact reduction: generated tests now exercise platform variation instead of a fixed linux literal.

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
- [x] Add compatibility adapters from existing enums (`deps::Platform`, DSL platform type). _(2026-02-18: `deps::Platform` adapter landed earlier; `Os::parse_dsl_platform` / `to_dsl_platform_variant` now provide canonical DSL Platform compatibility in `core/ir/src/platform.rs`.)_

### Phase 2: Highest-ROI Migrations

- [x] Replace hardcoded MIPS assembler/linker/qemu strings with modeled toolchain resources. _(2026-02-18: introduced `ToolchainCommands::mips_linux_gnu()` in `core/ir/src/platform.rs`; daglang parity test now consumes modeled commands instead of string literals.)_
- [x] Replace inline browser open branching with environment-aware resolver utility. _(2026-02-18: moved to shared `browser_open_request(..)` in `lib/primitives/src/browser.rs` and migrated dag-viz prepare node to use it.)_
- [x] Switch deps install and GH install platform keys to typed platform IDs. _(2026-02-18: `tool_upsert` PM→platform mapping now emits typed OS tokens via canonical `Os`; `transport/github/cli.rs` now models install methods as `(Os, InstallMethod)`.)_

### Phase 3: DSL + Testgen Alignment

- [x] Align DSL `Platform`/`CodegenTarget` vocabulary with canonical types. _(2026-02-18: `dsl/std/types.dag` now includes canonical `Arch`/`Vendor`/`Os`/`AbiEnv`/`ExecutionEnv` + `TargetTriple`/`RuntimePlatform`, and extends `CodegenTarget` with canonical target/runtime fields + `CodegenBackend`.)_
- [x] Remove linux-hardcoded mock defaults in testgen and generate per-platform variants. _(2026-02-18: replaced fixed `"linux"` platform mock with host-aware canonical variant cycling.)_
- [x] Add conformance tests for `linux-gnu` vs other env/ABI variants and qemu executor selection. _(2026-02-18: added `TargetTriple` conformance tests for linux-gnu vs linux-musl and windows-msvc; qemu selection is covered by modeled `ToolchainCommands::mips_linux_gnu()` test.)_
