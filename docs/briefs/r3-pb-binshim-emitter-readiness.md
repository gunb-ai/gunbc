# R3 PB — BinShim Rust emitter readiness (PB-owned planning slice)

**Status:** PROPOSAL (planning artifact, dispatch-gated). Authored 2026-05-01 by PB Manager continuation per inbox #1150 / #1134 — **emitter boundary** after `BinShim` carrier landing (#1361), **before** `regen_lens_shim` instance merge is required and **without** authoring the §7.2 equivalence `TestClaim` (owned by quick-newt).

**Owning manager:** Pure Bootstrap Manager (R2 → R3 continuation per `docs/r3-structure.md` §"Manager structure" Item 1).

**Lane boundary:** PB owns the **bin-shim Rust emitter** program shape per [`docs/design-pb-runtime-interpreter.md`](../design-pb-runtime-interpreter.md) §4.2 (Item 5) + §6 anti-bridge invariant #4 (mirror `dsl/extdeps/languages/rust/emit.dag` discipline, not parallel emit logic). This brief does **not** own the `BinShim` carrier record (Substrate / `src/v3/std/bin_shim.dag`), per-shim `data <bin>_shim: BinShim` instance rows under `dsl/std/runtime/bin_shims/` (first slice coordinated with neat-boar), `regen_lens.rs` retirement, or §7.2 fixture text.

## Smallest emit-pattern surface authorable **now** (docs + authority pins only)

These are **readiness pins**, not dispatch greens:

| Pin | Authority | “Ready” means |
|---|---|---|
| **Rust shell template** | Design doc §4.2 (Rust listing: `AUTO-GENERATED` header, `main` → `ProcessExit` dispatch) | Template parameters identified: per-shim `entry_fn_qualified_name`, source `.dag` path for header comment, `description` text for file doc comment. |
| **Carrier field names** | Live `type BinShim` at [`src/v3/std/bin_shim.dag`](../../src/v3/std/bin_shim.dag) (`entrypoint_name`, `description`, `entry`) | Instance rows and emitter metadata use **`entrypoint_name`** (not the older design-doc sketch field `name`). |
| **`ProcessExit` substrate** | `dsl/std/process.dag` | Unchanged consumer; emitter-emitted Rust matches §4.2 host convention (`ExitSuccess` / `ExitFailure`). |
| **Anti-bridge #4** | Design doc §6 | Emitter stays **one** `.dag` program alongside other emitters; no hand-rolled duplicate Rust templates in compiler bins. |
| **Bootstrap / parse surface** | `src/v3/compiler/src/bin/regen_bootstrap.rs` — glob is top-level `dsl/std/*.dag` + `src/v3/std/*.dag` + … | `dsl/std/runtime/**` is **outside** the full-bootstrap concatenation today. **Any** future emitter `.dag` under `dsl/std/runtime/` needs an explicit inclusion / load story (regen_bootstrap extension, staged loader, or alternate path) — **STOP** and coordinate with Substrate/build before assuming “drop file + it’s in bootstrap.” |

## Ordering dependencies (do not assume parallel branches merged)

1. **`data regen_lens_shim: BinShim = { … }`** in `dsl/std/runtime/bin_shims/regen_lens.dag` — **neat-boar** / coordinated instance slice; emitter **consumes** field values + `entry` `DeclarationRef`, it does not invent them.
2. **Item 4 (PB-Runtime interpreter-as-data)** — per [`r3-pb-binshim-retirement-worker.md`](r3-pb-binshim-retirement-worker.md) §"Dependencies" + design doc §5.4: the `.dag` entry body (`regen_lens_main` shape in §4.2) must be evaluable/foldable under the locked PB-Runtime emit pattern **before** emitted Rust can be trusted as behaviorally equivalent (§7.2 ultimately consumes this).
3. **§7.2 `TestClaim`** — **quick-newt**; this slice **must not** author it.
4. **`REGEN_OUTPUTS` + SG-0** — retirement PR concern per BinShim brief; not opened here.

Until (1)–(2) are at least **named on main**, a merge-blocking `.dag` emitter wired into `cargo run` / `build.rs` would be **fabricating** integration. **STOP** there; use this brief + [`dsl/std/runtime/bin_shims/README.md`](../../dsl/std/runtime/bin_shims/README.md) §"Emitter readiness" for handoff.

## STOP / escalation

- **Carrier shape pressure** — `INVARIANTS.md` §P1 to Substrate Manager; PB does not edit `src/v3/std/bin_shim.dag`.
- **Need new `TestPredicate` for §7.2** — §P1; not in emitter slice.
- **Emitter diverges from `extdeps.languages.rust.emit` shape** — design doc §6 #4; STOP and realign with PB Manager + Substrate.

## Cross-refs

- Design lock: [`docs/design-pb-runtime-interpreter.md`](../design-pb-runtime-interpreter.md) §4.2, §4.3, §6, §7.2 (intent only).
- Parent program: [`r3-pb-binshim-retirement-worker.md`](r3-pb-binshim-retirement-worker.md).
- Instance framework: [`dsl/std/runtime/bin_shims/README.md`](../../dsl/std/runtime/bin_shims/README.md).
- Consumer / call-surface audit: [`r3-pb-regen-lens-consumer-audit.md`](r3-pb-regen-lens-consumer-audit.md).
- Carrier ratchet: `src/v3/compiler/tests/integration/bin_shim_carrier_test.rs`.
