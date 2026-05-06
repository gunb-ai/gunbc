# R3 PB — BinShim / regen_lens chain blocker ledger (single-page state refresh)

**Status:** LEDGER (docs-only audit; no implementation). Authored 2026-05-02 by R3 PB continuation (witty-tern-193) per inbox #1134 ledger-refresh dispatch. **Mechanical HEAD refresh 2026-05-06** (warm-ant-877, PB dispatch [#1895](https://github.com/gunb-ai/gunbc/issues/1895)) pinned to `origin/main` **`8ca252983`** — `feat(std): ReferenceModel<T> phantom carrier (S7 PR-F Phase 2) (#1898)`. Rows **#1–#6** re-run via §Verification commands; chain ordering unchanged. **Row #3** narrative updated: `emit_rust_bin_shim.rs` now appears in the `BinShim` symbol sweep (hand-authored **shell** helper only — still **no** merge-blocking `.dag` emitter program under `dsl/extdeps/`).

**Owning manager:** Pure Bootstrap Manager (R2 → R3 continuation per `docs/r3-structure.md` §"Manager structure" Item 1).

**Purpose:** consolidate the current state of the six BinShim / `regen_lens` chain surfaces into one page so the next implementation dispatch does not re-audit stale blockers. This ledger is **derivative** — it does not introduce new design facts. Authority for each row remains the cited brief / README / source file.

## Verification commands (re-runnable on `origin/main`)

```bash
# Carrier
sed -n '1,25p' src/v3/std/bin_shim.dag
# Entry function
rg -n "^(fn|func) regen_lens_main|^(fn|func) .*_main.*ProcessExit" src/v3 dsl
# Instance row
ls dsl/std/runtime/bin_shims/
# Emitter program (carrier + compiler Rust; exclude generated mirrors)
rg -n "BinShim" dsl/extdeps src/v3/compiler/src --glob '!bootstrap_*generated*.rs'
# Extdeps slice alone — merge-blocking `.dag` emitter should appear here when live
# (expect **no** hits at current HEAD)
rg -n "BinShim" dsl/extdeps || true
# REGEN_OUTPUTS / SG-0 cutover for regen_lens.rs
rg -n "regen_lens|REGEN_OUTPUTS" src/v3/compiler/build.rs
# §7.2 equivalence script / comparison harness
ls scripts/ | rg -i 'binshim|regen_lens|equiv'
```

## Blocker ledger (six rows)

| # | Surface | State on `origin/main` | Owner / next mover | Blocking authority |
|---|---|---|---|---|
| 1 | `regen_lens_main` `.dag` entry function (`fn` or `func` `regen_lens_main() -> std.process.ProcessExit`; existing `dag run` entrypoints use `func`, e.g. `dsl/tools/purity_check.dag:150` `func run_purity_check() -> ProcessExit`) | **NOT LIVE** — `rg "^(fn\|func) regen_lens_main\|^(fn\|func) .*_main.*ProcessExit" src/v3 dsl` returns no match. Bare placeholder rejected by published STOP. | **Director + Substrate Manager** §P1 disposition for placeholder convention, **then** PB Manager dispatch (Shape A or C path). | `dsl/std/runtime/bin_shims/README.md` §"Substrate prerequisite (STOP+PING — refreshed post-#1361)"; `docs/briefs/r3-pb-binshim-emitter-readiness.md` §"Implementation slice STOP" Shape A. |
| 2 | `data regen_lens_shim: BinShim = { … }` instance row at `dsl/std/runtime/bin_shims/regen_lens.dag` | **NOT LIVE** — directory contains only `README.md`. Cannot be authored without row #1: `entry: DeclarationRef` would point at a non-existent `regen_lens_main`. | PB Manager — dispatch **after** row #1 lands (or Shape B "loader-first" lands a narrow `dsl/std/runtime/bin_shims/*.dag` allow-list). | `dsl/std/runtime/bin_shims/README.md` §"Substrate prerequisite"; `docs/briefs/r3-pb-binshim-emitter-readiness.md` §"Ordering dependencies" item 1. |
| 3 | BinShim Rust emitter (`.dag` program mirroring `dsl/extdeps/languages/rust/emit.dag` discipline) | **NOT LIVE** — No `.dag` emitter program under `dsl/extdeps/**` (`rg -n "BinShim" dsl/extdeps` → **no hits** at this HEAD). Outside `bootstrap_*generated*.rs`, `BinShim` appears in **`src/v3/std/bin_shim.dag`** (carrier), **`src/v3/compiler/src/emit_rust_bin_shim.rs`** (hand-authored **shell** text for generated `main.rs`; module docs: does **not** resolve `BinShim.entry` — row-#1 / DeclarationRef wiring still absent), and bootstrap mirrors. Loader gap unchanged: `dsl/std/runtime/**` is **not** in the `regen_bootstrap` full-bootstrap glob (`regen_bootstrap.rs` glob labels **`67–81`**). | PB Manager — dispatch **after** rows #1–#2 named on main (per `r3-pb-binshim-emitter-readiness.md` §"Ordering dependencies" gating clause "Until (1)–(2) are at least named on main, a merge-blocking `.dag` emitter wired into `cargo run` / `build.rs` would be fabricating integration"). | `docs/briefs/r3-pb-binshim-emitter-readiness.md` §"Smallest emit-pattern surface authorable now" + §"Honest blocker"; design doc §4.2 + §6 anti-bridge invariant #4. |
| 4 | §7.2 equivalence `TestClaim` + comparison script / artifacts (PB-Runtime ≡ emitted-Rust on `regen_lens`) | **NOT LIVE as runnable claim** — `docs/briefs/r3-pb-binshim-7-2-claim-shape.md` is live on main and intentionally contains the **docs-only** claim shape spec, but no runnable `.dag` `TestClaim` / `TestSuite` registration exists, no `scripts/*binshim*` / `scripts/*equiv*` comparison script exists, and design doc §7.2 fixture remains intent-only. The shape spec is authoritative for the future runnable claim; it is not itself the executable artifact. | **PB-assigned §7.2 worker** under the BinShim retirement program (explicitly **not** the readiness brief, **not** this ledger, **not** the framework README — see `r3-pb-binshim-emitter-readiness.md` §"STOP / escalation" and `r3-pb-binshim-7-2-claim-shape.md` shape spec). Dispatch gated on rows #1–#3 + Item 4 (PB-Runtime interpreter-as-data) so behavioral equivalence is a meaningful claim, not vacuous. | `docs/briefs/r3-pb-binshim-7-2-claim-shape.md`; `docs/briefs/r3-pb-binshim-retirement-worker.md` §"Acceptance"; `docs/briefs/r3-pb-binshim-emitter-readiness.md` §"Ordering dependencies" item 3. |
| 5 | `REGEN_OUTPUTS` / SG-0 cutover for `src/v3/compiler/src/bin/regen_lens.rs` (generated-partition flip) | **NOT LIVE** — `src/v3/compiler/build.rs::REGEN_OUTPUTS` (lines **479–513**) lists **25** entries at verification time (`origin/main` HEAD **`8ca252983`**); `src/v3/compiler/src/bin/regen_lens.rs` is **absent from that list** (the hand-Rust bin still exists at that path — see row #6 — but is not yet registered as generated-partition output). Per SG-0 partitioning, the bin is still classified hand-Rust producer-input until both row #3 emits it AND it is registered here under the locked `// AUTO-GENERATED from <path> — DO NOT EDIT.` header. | Retirement-PR worker (per BinShim retirement program; coordinates with build.rs / SG-0 owner). | `dsl/std/runtime/bin_shims/README.md` §"What does NOT belong here" (last bullet — `src/v3/compiler/build.rs::REGEN_OUTPUTS` routing); `docs/briefs/r3-pb-binshim-emitter-readiness.md` §"Ordering dependencies" item 4 ("retirement PR concern; not opened here"). |
| 6 | `regen_lens.rs` retirement PR (delete hand-Rust bin once rows #1–#5 hold) | **NOT LIVE** — bin still hand-authored under `src/v3/compiler/src/bin/`. Skeleton brief landed; no retirement branch / PR in `gh pr list`. | BinShim retirement worker — dispatch is the **terminal** step of this chain; depends on rows #1 → #2 → #3 → #4 → #5 in that order. | `docs/briefs/r3-pb-binshim-retirement-worker.md` §"First slice — `regen_lens.rs`"; `docs/briefs/r3-pb-t-lensproducer-sub3-regen-lens-retirement.md`; `docs/briefs/r3-pb-regen-lens-consumer-audit.md` (consumer call-surface inventory the retirement PR must clear). |

## Current next-unblock order

```
#1 (regen_lens_main entry fn)  ──►  #2 (instance row)  ──►  #3 (emitter)  ──►  #4 (§7.2 claim)  ──►  #5 (REGEN_OUTPUTS / SG-0 cutover)  ──►  #6 (retirement PR)
```

Row #1 is the head of the chain. Until Director / Substrate Manager record §P1 disposition for the placeholder-entry convention (Shape A) **or** Item 4 (PB-Runtime interpreter-as-data) reaches a state where a real `regen_lens_main` body is authorable (Shape C) **or** Substrate + build land Shape B's narrow `dsl/std/runtime/bin_shims/*.dag` loader allow-list, every later row in the chain remains gated.

## What this ledger does NOT do

- Does not introduce a new `<bin_name>_main` placeholder, instance row, emitter, §7.2 script, `REGEN_OUTPUTS` edit, retirement, or carrier-shape change. All such moves remain blocked by the published STOPs in the cited authority documents.
- Does not re-litigate Shape A vs Shape B vs Shape C — that disposition is `r3-pb-binshim-emitter-readiness.md` §"Smallest next-unblock PR shapes" + Director/Substrate call.
- Does not own §7.2 `TestClaim` text (PB-assigned §7.2 worker under retirement dispatch only).

## Cross-refs

- Framework / instance authority: [`dsl/std/runtime/bin_shims/README.md`](../../dsl/std/runtime/bin_shims/README.md).
- Parent program brief: [`docs/briefs/r3-pb-binshim-retirement-worker.md`](r3-pb-binshim-retirement-worker.md).
- Emitter readiness + Shape A/B/C unblock menu: [`docs/briefs/r3-pb-binshim-emitter-readiness.md`](r3-pb-binshim-emitter-readiness.md).
- §7.2 claim shape: [`docs/briefs/r3-pb-binshim-7-2-claim-shape.md`](r3-pb-binshim-7-2-claim-shape.md).
- Consumer audit (retirement-PR input): [`docs/briefs/r3-pb-regen-lens-consumer-audit.md`](r3-pb-regen-lens-consumer-audit.md).
- Sub-gate skeleton: [`docs/briefs/r3-pb-t-lensproducer-sub3-regen-lens-retirement.md`](r3-pb-t-lensproducer-sub3-regen-lens-retirement.md).
- Design lock: [`docs/design-pb-runtime-interpreter.md`](../design-pb-runtime-interpreter.md) §4 (Item 5 emit pattern), §4.2 (carrier sketch + emitter shape), §4.3 (dissolution path), §5.4 (PB / Substrate / Evaluator boundary), §6 (anti-bridge invariants), §7.2 (BinShim equivalence fixture).
- Carrier source: [`src/v3/std/bin_shim.dag`](../../src/v3/std/bin_shim.dag).
- Carrier ratchet: [`src/v3/compiler/tests/integration/m2_substrate_inhabitance_test.rs`](../../src/v3/compiler/tests/integration/m2_substrate_inhabitance_test.rs) (`bin_shim_carrier_has_locked_three_field_shape` — ~`:2870` at verification HEAD).
- INVARIANTS §P1 substrate-fact-introduction procedure: [`INVARIANTS.md`](../../INVARIANTS.md).
