# `dsl/std/runtime/bin_shims/` — PB-owned `BinShim` instance declarations (framework)

Canonical home for PB-owned per-shim `BinShim` instance declarations per [`docs/design-pb-runtime-interpreter.md`](../../../../docs/design-pb-runtime-interpreter.md) §4 (Item 5: PB-1 generated bin-shim emit pattern) and [`docs/briefs/r3-pb-binshim-retirement-worker.md`](../../../../docs/briefs/r3-pb-binshim-retirement-worker.md) §"First slice — `regen_lens.rs`".

## Status

**FRAMEWORK ONLY.** No instance declarations on main yet — see §"Substrate prerequisite (STOP+PING)" below. The `BinShim` carrier is now live (post-#1361); the remaining gate for per-shim row authoring is the `<bin_name>_main` entry-function declaration each shim's `entry: DeclarationRef` field needs to point at. This directory holds the README so future per-shim declarations have a canonical location, naming convention, and dependency contract once that gap closes.

## Ownership boundary (per design-doc §5.4)

- **PB owns:** instance-row authoring under this directory (one `.dag` file per existing PB-owned hand-Rust bin — e.g. `regen_lens.dag` declaring `data regen_lens_shim: BinShim = { ... }`); per-shim retirement dispatch.
- **Substrate Manager owns:** the `BinShim` carrier-type shape itself (the `type BinShim { ... }` declaration; carrier-field evolution; signature refinement of `entry: () -> std.process.ProcessExit`).
- **Evaluator Manager owns:** the runtime-value model that PB-Runtime mirrors; transitive prereq for the emit pattern.

Generalized carrier-shape evolution (e.g. additional fields, refining `entry`'s type signature beyond what design-doc §4.2 sketches) escalates via [`INVARIANTS.md`](../../../../INVARIANTS.md) §P1 substrate-fact-introduction procedure to Substrate Manager. PB workers MUST NOT extend the carrier from this directory.

## Naming convention

Per the live carrier at `src/v3/std/bin_shim.dag` (landed via #1361) and design-doc §4.2:

- **File path:** `dsl/std/runtime/bin_shims/<bin_name>.dag` (one declaration per file; one file per existing hand-Rust bin under `src/v3/compiler/src/bin/`).
- **Declaration name:** `data <bin_name>_shim: BinShim = { ... }` — `<bin_name>` matches the bin's existing hand-Rust filename without the `.rs` extension. Example: `regen_lens.rs` → `dsl/std/runtime/bin_shims/regen_lens.dag` declaring `data regen_lens_shim: BinShim = { ... }`.
- **Module:** `module std.runtime.bin_shims.<bin_name>` (mirrors path).
- **Live carrier fields** (`src/v3/std/bin_shim.dag`): `entrypoint_name: NonEmptyStr`, `description: String`, `entry: DeclarationRef`. The `entry` field references a `.dag` `() -> std.process.ProcessExit` function declaration; per the carrier's own scaffold comment, "`entry` remains a plain `DeclarationRef` until the substrate can express `DeclarationRef<fn () -> std.process.ProcessExit>`" — the type-system constraint is by reviewer convention until that refinement lands.
- **Imports a per-shim row needs:** `import v3.std.bin_shim { BinShim }` (live), `import std.process { ProcessExit }` (live at `dsl/std/process.dag:39`), and the `<bin_name>_main` entry-function declaration (see §"Substrate prerequisite (STOP+PING)" — *not* yet live for any existing PB-owned bin).

The naming convention is locked here so per-shim retirement workers (per the sub-gate skeletons at [`docs/briefs/r3-pb-t-lensproducer-sub3-regen-lens-retirement.md`](../../../../docs/briefs/r3-pb-t-lensproducer-sub3-regen-lens-retirement.md) and forward) have a consistent target without re-deriving paths at dispatch time.

## Substrate prerequisite (STOP+PING — refreshed post-#1361)

**Verified on origin/main HEAD post-#1361 carrier landing:**

- **`type BinShim { ... }`** — **LIVE** at `src/v3/std/bin_shim.dag:19` (3 fields: `entrypoint_name: NonEmptyStr`, `description: String`, `entry: DeclarationRef`). Carrier-shape ratchet at `src/v3/compiler/tests/integration/m2_substrate_inhabitance_test.rs::bin_shim_carrier_has_locked_three_field_shape` pins this exact shape.
- **`std.process.ProcessExit`** — LIVE at `dsl/std/process.dag:39` (`type ProcessExit = ExitSuccess | ExitFailure { ... }`).
- **`dsl/std/runtime/bin_shims/`** — LIVE (framework directory + this README, landed via PR #1347).
- **`<bin_name>_main` `.dag` entry function for any PB-owned hand-Rust bin** — **NOT YET LIVE.** `grep -rn "^fn regen_lens_main\|^fn .*_main.*ProcessExit" src/v3/ dsl/` returns no match. Each shim's `entry: DeclarationRef` field needs a `.dag`-authored function `fn <bin_name>_main() -> std.process.ProcessExit` to point at; without that target, `data <bin_name>_shim: BinShim = { ... entry: <bin_name>_main, ... }` cannot resolve.

**Implication for first per-shim authoring (`regen_lens_shim`):** the `entrypoint_name` and `description` fields are trivially expressible (per Cargo `[[bin]] name = "regen_lens"` + the bin's docstring), but `entry` requires a live `fn regen_lens_main() -> ProcessExit` declaration that does NOT exist on main. Authoring a stub function locally would invent emit/runtime semantics for the future BinShim emitter — that crosses into emit/runtime work explicitly out of instance-declaration scope. **The `entry`-target gap is the new STOP+PING.**

Until a `<bin_name>_main` `.dag` entry function exists for a given bin, **its instance declaration cannot be authored without fabricating the entry target**. The framework directory + this README record the canonical home; per-shim rows wait on the entry-function authoring lane.

The cleanest path forward (Director / Substrate Manager / PB Manager call): either (a) land each `<bin_name>_main` entry function as part of the BinShim emitter / per-shim runtime work that authors its body, OR (b) define a "trivial-entry" Substrate convention where a stub `() -> ExitSuccess` function is the explicit placeholder the emitter later replaces. (a) is the design-doc §4.3 dissolution path's natural flow; (b) is a substrate-convention extension that should follow §P1 if surfaced.

When the entry-function gap closes for `regen_lens`, the first authoring slice is `regen_lens.dag` per the planning brief's "First slice — `regen_lens.rs`" path; subsequent shims (other `regen_*` drivers, `self_host_fixed_point.rs`-shaped bins per design-doc §4.1) follow the same template.

## What does NOT belong here

- The `BinShim` carrier-type declaration itself (Substrate Manager territory).
- The bin-shim emit pattern / `.dag` emitter program (lives under the language emit modules, analogous to `dsl/extdeps/languages/rust/emit.dag` per design-doc §4.2 + anti-bridge invariant #4).
- Hand-Rust bin shims (those live at `src/v3/compiler/src/bin/` and are the retirement targets, not the canonical authoring surface).
- Generated Rust output from the emit pattern (lives under `src/v3/compiler/src/bin/` with the locked `// AUTO-GENERATED from <path> — DO NOT EDIT.` header per design-doc §4.2; routed through `REGEN_OUTPUTS` in `src/v3/compiler/build.rs` so SG-0 census partitions it as generated).
- The `no_new_bin_shim_hand_rust` §7.3 fixture (Substrate Manager's §P1 disposition picks where it lands; not this directory).

## Cross-refs

- Parent design lock: [`docs/design-pb-runtime-interpreter.md`](../../../../docs/design-pb-runtime-interpreter.md) §4 (Item 5 emit pattern), §4.2 (`type BinShim` sketch + emitter shape), §4.3 (dissolution path), §5.1 (sub-gate decomposition), §5.4 (PB / Substrate / Evaluator boundary), §6 (anti-bridge invariants), §7.2 (BinShim equivalence fixture), §7.3 (No-new-bin-shim-hand-Rust fixture).
- Parent planning brief: [`docs/briefs/r3-pb-binshim-retirement-worker.md`](../../../../docs/briefs/r3-pb-binshim-retirement-worker.md).
- Sub-gate skeleton (consumer of this framework): [`docs/briefs/r3-pb-t-lensproducer-sub3-regen-lens-retirement.md`](../../../../docs/briefs/r3-pb-t-lensproducer-sub3-regen-lens-retirement.md).
- Sibling LensProducer sub-gate skeletons (different mechanism): [`docs/briefs/r3-pb-t-lensproducer-sub1-lens-apply-retirement.md`](../../../../docs/briefs/r3-pb-t-lensproducer-sub1-lens-apply-retirement.md), [`docs/briefs/r3-pb-t-lensproducer-sub2-lens-testgen-retirement.md`](../../../../docs/briefs/r3-pb-t-lensproducer-sub2-lens-testgen-retirement.md).
- Substrate-fact-introduction procedure (escalation path): [`INVARIANTS.md`](../../../../INVARIANTS.md) §P1 — used when the `BinShim` carrier lands and an authoring decision needs Substrate Manager input.
- PB Manager brief: [`docs/briefs/r2-pure-bootstrap-manager.md`](../../../../docs/briefs/r2-pure-bootstrap-manager.md) line 37 (BinShim instances + emit pattern + retirement dispatch lane).
- Live `ProcessExit`: [`dsl/std/process.dag`](../../process.dag) line 39.
- Hand-Rust bins targeted for retirement (do not edit until dispatch): `src/v3/compiler/src/bin/regen_lens.rs` (first slice); other `regen_*` drivers per design-doc §4.1 (broader class).
