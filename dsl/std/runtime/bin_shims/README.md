# `dsl/std/runtime/bin_shims/` — PB-owned `BinShim` instance declarations (framework, frozen-v3)

Framework directory for PB-owned per-shim `BinShim` instance declarations. **No instance
`.dag` files are on main** — this directory holds the canonical home + naming convention
only. The per-shim retirement program that would populate it is part of the **frozen v3
line** (v2 is the active phase — see [THESIS.md](../../../../THESIS.md)); its design and
coordination briefs are not carried in this public tree.

## Live anchors

- **`BinShim` carrier** — [`src/v3/std/bin_shim.dag`](../../../../src/v3/std/bin_shim.dag)
  (`module v3.std.bin_shim`; fields `entrypoint_name: NonEmptyStr`, `description: String`,
  `entry: DeclarationRef`).
- **`ProcessExit`** — [`dsl/std/process.dag`](../../process.dag)
  (`type ProcessExit = ExitSuccess | ExitFailure { ... }`); a shim's `entry` references a
  `() -> std.process.ProcessExit` function.
- **Carrier-shape evolution** escalates via [`INVARIANTS.md`](../../../../INVARIANTS.md)
  §P1 (substrate-fact-introduction). PB does not extend the carrier from this directory.

## Naming convention

- **File:** `dsl/std/runtime/bin_shims/<bin_name>.dag` — one declaration per file;
  `<bin_name>` matches the hand-Rust bin under `src/v3/compiler/src/bin/`.
- **Declaration:** `data <bin_name>_shim: BinShim = { entrypoint_name: "<bin_name>", description: "…", entry: <bin_name>_main }`.
- **Module:** `module std.runtime.bin_shims.<bin_name>`.

## What does NOT belong here

- The `BinShim` carrier-type declaration (Substrate Manager territory).
- The bin-shim emit pattern / `.dag` emitter program (lives under the language emit modules).
- Hand-Rust bin shims (`src/v3/compiler/src/bin/` — the retirement targets) and the
  generated Rust output from the emit pattern.
