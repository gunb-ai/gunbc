# R3 PB — BinShim Rust emitter readiness (PB-owned planning slice)

**Status:** PROPOSAL (planning artifact, dispatch-gated). Authored 2026-05-01 by PB Manager continuation per inbox #1150 / #1134 — **emitter boundary** after `BinShim` carrier landing (#1361), **before** `regen_lens_shim` instance merge is required and **without** authoring the §7.2 equivalence `TestClaim` (PB-assigned §7.2 worker under the BinShim retirement dispatch — not this readiness slice). **Mechanical HEAD refresh 2026-05-06** (warm-ant-877 / [#1895](https://github.com/gunb-ai/gunbc/issues/1895)): loader / build facts in §"Implementation slice STOP" re-pinned to `origin/main` **`831080dee`**.

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
3. **§7.2 `TestClaim`** — **PB-assigned §7.2 worker** (replacement for archived session routing); this readiness slice **must not** author it.
4. **`REGEN_OUTPUTS` + SG-0** — retirement PR concern per BinShim brief; not opened here.

Until (1)–(2) are at least **named on main**, a merge-blocking `.dag` emitter wired into `cargo run` / `build.rs` would be **fabricating** integration. **STOP** there; use this brief + [`dsl/std/runtime/bin_shims/README.md`](../../dsl/std/runtime/bin_shims/README.md) §"Emitter readiness" for handoff.

## Implementation slice STOP — emitter / entry function (2026-05-01 dispatch)

**Dispatch ask:** land the smallest **real** emitter / `<bin_name>_main` entry surface toward `regen_lens_shim` without fabricating runtime behavior.

**Loader / build audit (facts on `origin/main`):**

- **`regen_bootstrap.rs`** — full-bootstrap labels are still the `compile_full_bootstrap_*` glob strings at **`67–81`** (see `render_bootstrap_generated_rs` call sites: full bootstrap **`70–71`**, no-parse-surface **`79–80`**). **`dsl/std/runtime/**` is not concatenated** into that snapshot; per-shim instance files planned under `dsl/std/runtime/bin_shims/` do not enter `bootstrap_generated*.rs` unless the glob / seed pipeline is extended under Substrate + build coordination.
- **`REGEN_OUTPUTS`** (`src/v3/compiler/build.rs`) — gates **written** codegen outputs for SG-0 partition; adding generated `regen_lens.rs` from an emitter belongs to a **retirement** slice, not this STOP.
- **Parse prep harness** (`src/v3/compiler/tests/integration.rs`, **`mod parse_stage4_prep`**) — **`PARSE_CORPUS_MANIFEST`** / `include_str!("integration/parse_corpus_manifest.txt")` at **`814`**; **`fn parse_corpus_paths`** at **`849`** (`repo_root()` / `.expect(...)` at **`820–826`** are **below** the manifest — do not confuse with **`814`**). Enumerates a fixed `dsl/std` eight-file subset + **all** `src/v3/std/*.dag` + compiler/spec paths; a new `src/v3/std/*.dag` authority file is prep-visible, but **does not** solve `dsl/std/runtime/bin_shims/*.dag` loader inclusion for bootstrap.

**Honest blocker — why a “tiny” `regen_lens_main` is not a free commit today**

1. **Real body** — needs PB-Runtime **Item 4** so the `.dag` entry mirrors the existing regen pipeline per design §4.2 / §5.4; authoring pipeline semantics here would **fabricate** Item 5 + Item 4 convergence.
2. **Fail-closed stub only** — `std.process.exit_failure` exists, but the framework README (`dsl/std/runtime/bin_shims/README.md` §"Substrate prerequisite") currently routes **any** placeholder entry, even fail-closed `ProcessExit` shapes, through **§P1 substrate convention** before PB lands it unilaterally. Until that disposition is recorded (Director / Substrate / PB Manager), a stub `regen_lens_main` would **violate the published STOP discipline** on the instance framework README — not a silent docs tweak.
3. **Instance row** (`data regen_lens_shim: BinShim`) — still coordinated with neat-boar; **`entry: DeclarationRef` must resolve** to a live `regen_lens_main` in the same loader story as the row file.

**Smallest next-unblock PR shapes (pick at gate; not ordered here as a program):**

| Shape | What lands | Who gates |
|-------|------------|-----------|
| **A — §P1 disposition + fail-closed staging entry** | Record §P1 OK for a **single** fail-closed `regen_lens_main() -> ProcessExit` scaffold (e.g. `exit_failure` with a fixed reason string citing Item 4 STOP) in an agreed module path (`src/v3/std/...dag` **or** runtime subtree **after** loader extension), then `regen_bootstrap --verify` snapshot regen. Still **no** `data regen_lens_shim` until instance slice. | Director + Substrate note on placeholder convention; PB Manager for Item 5 sequencing |
| **B — Loader-first** | Narrow allow-list in `regen_bootstrap` / seed for **`dsl/std/runtime/bin_shims/*.dag`** (not blanket `dsl/std/runtime/**`), then instance + entry co-authored under that loader. | Substrate / build owners + PB Manager |
| **C — Item 4-first** | Land PB-Runtime interpreter slice that can host the real `regen_lens_main` fold; then emitter consumes it. | Item 4 program |

**This STOP path (this PR if docs-only):** record the above; **no** new `.dag` entry body, **no** emitter wiring, **no** §7.2, **no** `regen_lens.rs` retirement, **no** `BinShim` carrier edit.

## STOP / escalation

- **Carrier shape pressure** — `INVARIANTS.md` §P1 to Substrate Manager; PB does not edit `src/v3/std/bin_shim.dag`.
- **Need new `TestPredicate` for §7.2** — §P1; not in emitter slice. **§7.2 `TestClaim` text** is authored only by the **PB-assigned §7.2 worker** under BinShim retirement dispatch (see parent brief §"Acceptance"), not this readiness brief.
- **Emitter diverges from `extdeps.languages.rust.emit` shape** — design doc §6 #4; STOP and realign with PB Manager + Substrate.

## Cross-refs

- Design lock: [`docs/design-pb-runtime-interpreter.md`](../design-pb-runtime-interpreter.md) §4.2, §4.3, §6, §7.2 (intent only).
- Parent program: [`r3-pb-binshim-retirement-worker.md`](r3-pb-binshim-retirement-worker.md).
- Instance framework: [`dsl/std/runtime/bin_shims/README.md`](../../dsl/std/runtime/bin_shims/README.md).
- Consumer / call-surface audit: [`r3-pb-regen-lens-consumer-audit.md`](r3-pb-regen-lens-consumer-audit.md).
- Carrier ratchet: [`src/v3/compiler/tests/integration/m2_substrate_inhabitance_test.rs`](../../src/v3/compiler/tests/integration/m2_substrate_inhabitance_test.rs) (`bin_shim_carrier_*` — consolidated here post-#1370; standalone `bin_shim_carrier_test.rs` removed; three-field shape test ~`:2869` at 2026-05-06 HEAD).
