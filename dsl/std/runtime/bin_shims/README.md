# `dsl/std/runtime/bin_shims/` — PB-owned `BinShim` instance declarations (framework)

Canonical home for PB-owned per-shim `BinShim` instance declarations per [`docs/design-pb-runtime-interpreter.md`](../../../../docs/design-pb-runtime-interpreter.md) §4 (Item 5: PB-1 generated bin-shim emit pattern) and [`docs/briefs/r3-pb-binshim-retirement-worker.md`](../../../../docs/briefs/r3-pb-binshim-retirement-worker.md) §"First slice — `regen_lens.rs`".

## Status

**FRAMEWORK ONLY — instance `.dag` files not yet on main.** The `BinShim` **carrier** is live at [`src/v3/std/bin_shim.dag`](../../../../src/v3/std/bin_shim.dag) (`module v3.std.bin_shim`; landed #1361). Per-shim `data <bin>_shim: BinShim = { … }` rows still wait on coordinated instance authoring (first slice: `regen_lens.dag` — neat-boar / PB ordering per [`docs/briefs/r3-pb-binshim-emitter-readiness.md`](../../../../docs/briefs/r3-pb-binshim-emitter-readiness.md)). This README locks paths + naming; see §"Emitter readiness" for the Rust emitter boundary.

## Ownership boundary (per design-doc §5.4)

- **PB owns:** instance-row authoring under this directory (one `.dag` file per existing PB-owned hand-Rust bin — e.g. `regen_lens.dag` declaring `data regen_lens_shim: BinShim = { ... }`); per-shim retirement dispatch.
- **Substrate Manager owns:** the `BinShim` carrier-type shape itself (the `type BinShim { ... }` declaration; carrier-field evolution; signature refinement of `entry: () -> std.process.ProcessExit`).
- **Evaluator Manager owns:** the runtime-value model that PB-Runtime mirrors; transitive prereq for the emit pattern.

Generalized carrier-shape evolution (e.g. additional fields, refining `entry`'s type signature beyond what design-doc §4.2 sketches) escalates via [`INVARIANTS.md`](../../../../INVARIANTS.md) §P1 substrate-fact-introduction procedure to Substrate Manager. PB workers MUST NOT extend the carrier from this directory.

## Naming convention

Per design-doc §4.2, aligned to the **live** carrier at `src/v3/std/bin_shim.dag` (`entrypoint_name`, `description`, `entry` — the design sketch historically used `name`; use `entrypoint_name` for instances):

`data regen_lens_shim: BinShim = { entrypoint_name: "regen_lens", description: "…", entry: regen_lens_main, … }`

- **File path:** `dsl/std/runtime/bin_shims/<bin_name>.dag` (one declaration per file; one file per existing hand-Rust bin under `src/v3/compiler/src/bin/`).
- **Declaration name:** `data <bin_name>_shim: BinShim = { ... }` — `<bin_name>` matches the bin's existing hand-Rust filename without the `.rs` extension. Example: `regen_lens.rs` → `dsl/std/runtime/bin_shims/regen_lens.dag` declaring `data regen_lens_shim: BinShim = { ... }`.
- **Module:** `module std.runtime.bin_shims.<bin_name>` (mirrors path).
- **Imports:** `BinShim` from `v3.std.bin_shim` / [`src/v3/std/bin_shim.dag`](../../../../src/v3/std/bin_shim.dag); `std.process.ProcessExit` from `dsl/std/process.dag:39` (live).

The naming convention is locked here so per-shim retirement workers (per the sub-gate skeletons at [`docs/briefs/r3-pb-t-lensproducer-sub3-regen-lens-retirement.md`](../../../../docs/briefs/r3-pb-t-lensproducer-sub3-regen-lens-retirement.md) and forward) have a consistent target without re-deriving paths at dispatch time.

## Carrier + instance authoring status

- **`type BinShim`** — **LIVE** at [`src/v3/std/bin_shim.dag`](../../../../src/v3/std/bin_shim.dag) (`module v3.std.bin_shim`; landed #1361). **Generalized carrier-shape evolution** (extra fields, refining `entry` beyond the locked three-field record) remains Substrate Manager territory per design-doc §5.4 + `INVARIANTS.md` §P1 — PB does not edit the carrier from this directory.
- **Per-shim `data <bin>_shim: BinShim` rows** — **not yet on main** here; first slice `regen_lens.dag` per [`docs/briefs/r3-pb-binshim-retirement-worker.md`](../../../../docs/briefs/r3-pb-binshim-retirement-worker.md) §"First slice" (coordinate instance authoring with neat-boar / PB Manager ordering).
- **`std.process.ProcessExit`** — LIVE at [`dsl/std/process.dag`](../../process.dag) (see line 39; `type ProcessExit`).

## Emitter readiness

Handoff surface for the **Item 5 bin-shim Rust emitter** (`.dag` emitter program per [`docs/design-pb-runtime-interpreter.md`](../../../../docs/design-pb-runtime-interpreter.md) §4.2 + §6 anti-bridge invariant #4 — **not** stored in this directory; see §"What does NOT belong here"):

- **Planning brief:** [`docs/briefs/r3-pb-binshim-emitter-readiness.md`](../../../../docs/briefs/r3-pb-binshim-emitter-readiness.md) — prerequisite pins, ordering dependencies, **`regen_bootstrap` / `dsl/std/*.dag` glob does not include `dsl/std/runtime/**`** loader gap, STOP lines.
- **§7.2 equivalence `TestClaim`:** authored only by a **PB-assigned §7.2 worker** under the BinShim retirement program ([`r3-pb-binshim-retirement-worker.md`](../../../../docs/briefs/r3-pb-binshim-retirement-worker.md) §"Acceptance"); do not route §7.2 work through this README or the emitter-readiness brief.

## What does NOT belong here

- The `BinShim` carrier-type declaration itself (Substrate Manager territory).
- The bin-shim emit pattern / `.dag` emitter program (lives under the language emit modules, analogous to `dsl/extdeps/languages/rust/emit.dag` per design-doc §4.2 + anti-bridge invariant #4).
- Hand-Rust bin shims (those live at `src/v3/compiler/src/bin/` and are the retirement targets, not the canonical authoring surface).
- Generated Rust output from the emit pattern (lives under `src/v3/compiler/src/bin/` with the locked `// AUTO-GENERATED from <path> — DO NOT EDIT.` header per design-doc §4.2; routed through `REGEN_OUTPUTS` in `src/v3/compiler/build.rs` so SG-0 census partitions it as generated).
- The `no_new_bin_shim_hand_rust` §7.3 fixture (Substrate Manager's §P1 disposition picks where it lands; not this directory).

## Cross-refs

- Parent design lock: [`docs/design-pb-runtime-interpreter.md`](../../../../docs/design-pb-runtime-interpreter.md) §4 (Item 5 emit pattern), §4.2 (`type BinShim` sketch + emitter shape), §4.3 (dissolution path), §5.1 (sub-gate decomposition), §5.4 (PB / Substrate / Evaluator boundary), §6 (anti-bridge invariants), §7.2 (BinShim equivalence fixture), §7.3 (No-new-bin-shim-hand-Rust fixture).
- Parent planning brief: [`docs/briefs/r3-pb-binshim-retirement-worker.md`](../../../../docs/briefs/r3-pb-binshim-retirement-worker.md).
- Emitter readiness (Item 5 `.dag` emitter boundary): [`docs/briefs/r3-pb-binshim-emitter-readiness.md`](../../../../docs/briefs/r3-pb-binshim-emitter-readiness.md).
- Sub-gate skeleton (consumer of this framework): [`docs/briefs/r3-pb-t-lensproducer-sub3-regen-lens-retirement.md`](../../../../docs/briefs/r3-pb-t-lensproducer-sub3-regen-lens-retirement.md).
- Sibling LensProducer sub-gate skeletons (different mechanism): [`docs/briefs/r3-pb-t-lensproducer-sub1-lens-apply-retirement.md`](../../../../docs/briefs/r3-pb-t-lensproducer-sub1-lens-apply-retirement.md), [`docs/briefs/r3-pb-t-lensproducer-sub2-lens-testgen-retirement.md`](../../../../docs/briefs/r3-pb-t-lensproducer-sub2-lens-testgen-retirement.md).
- Substrate-fact-introduction procedure (escalation path): [`INVARIANTS.md`](../../../../INVARIANTS.md) §P1 — carrier-shape evolution, new `CensusListConstant` / filter dispositions, and related substrate questions escalate here (PB does not extend the carrier from this directory).
- PB Manager brief: [`docs/briefs/r2-pure-bootstrap-manager.md`](../../../../docs/briefs/r2-pure-bootstrap-manager.md) line 37 (BinShim instances + emit pattern + retirement dispatch lane).
- Live `ProcessExit`: [`dsl/std/process.dag`](../../process.dag) line 39.
- Hand-Rust bins targeted for retirement (do not edit until dispatch): `src/v3/compiler/src/bin/regen_lens.rs` (first slice); other `regen_*` drivers per design-doc §4.1 (broader class).
