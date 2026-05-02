# R3 PB — BinShim row #2 (`regen_lens_shim` instance) convention packet — STOP+PING

**Status:** STOP+PING (docs-only; no instance row authoring, no entry function, no placeholder, no emitter, no §7.2 runnable claim, no carrier edits). Authored 2026-05-02 by R3 PB continuation (witty-tern-193) per inbox #1134 row-#2 dispatch (corrected). Verified against `origin/main` HEAD `9cf6dd223`.

**Owning manager:** Pure Bootstrap Manager (R2 → R3 continuation).

**Verdict:** **STOP** — row-#2 convention packet cannot land before the row-#1 §P1 disposition (PR [#1511](https://github.com/gunb-ai/gunbc/pull/1511) `docs/briefs/r3-pb-binshim-row-1-decision-packet.md`) records Director/Substrate's chosen Shape A vs B vs C. The row-#1 disposition materially changes what a row-#2 convention packet would say, and authoring now would either (a) unilaterally commit to one shape — violating the published STOP+PING at [`dsl/std/runtime/bin_shims/README.md`](../../dsl/std/runtime/bin_shims/README.md) §"Substrate prerequisite (STOP+PING — refreshed post-#1361)" — or (b) write three parallel sub-packets that all collapse into the row-#1 disposition.

## Verified row-#2 state on `origin/main`

```bash
$ ls dsl/std/runtime/bin_shims/
README.md                       # framework only; no per-shim .dag yet
$ rg -n "^(fn|func) regen_lens_main|^(fn|func) .*_main.*ProcessExit" src/v3 dsl
# (no match — row #1 still NOT-LIVE; row #2 cannot resolve `entry: regen_lens_main`)
```

Per [`docs/briefs/r3-pb-binshim-blocker-ledger.md`](r3-pb-binshim-blocker-ledger.md) chain ordering `#1 → #2 → #3 → #4 → #5 → #6`, row #2 (`data regen_lens_shim: BinShim = { … entry: regen_lens_main, … }` at `dsl/std/runtime/bin_shims/regen_lens.dag`) cannot be authored without a live `regen_lens_main` declaration — the carrier's `entry: DeclarationRef` field has nothing to point at.

## Why row-#1 disposition gates row-#2 convention text

Authoring a row-#2 packet that takes a position on the convention requires committing to one of the three row-#1 shapes:

- **Under Shape A (fail-closed placeholder convention):** row #2 lands as a vacuous instance with `entry` pointing at `func regen_lens_main() -> ProcessExit { exit_failure(…) }`. The row-#2 convention packet's body — what `description` says (does it cite the placeholder-rejection-window?), whether row #2 lands in the same PR as row #1's placeholder, whether `entry`'s `DeclarationRef` resolution path differs from the eventual real-body resolution path — depends entirely on the §P1-recorded placeholder convention.
- **Under Shape B (loader-first allow-list):** row #1 + row #2 are **co-authored** in the same PR under the new `dsl/std/runtime/bin_shims/*.dag` loader extension. The row-#2 convention packet collapses into the Shape B disposition: there is no separate row-#2 authoring step, so a separate row-#2 packet is structurally redundant.
- **Under Shape C (Item 4-first real body):** row #2 follows naturally once Item 4 reaches the milestone where `Lens<C>`-fold of the regen pipeline is expressible. The convention shape — particularly what `description` documents and whether `entry`'s declared type can be tightened beyond `DeclarationRef` (the carrier scaffold comment at [`src/v3/std/bin_shim.dag:14-18`](../../src/v3/std/bin_shim.dag) explicitly defers `DeclarationRef<fn () -> std.process.ProcessExit>` to a future substrate refinement) — depends on the live PB-Runtime fold vocabulary at that milestone.

In all three cases the row-#2 packet's load-bearing content is **derivative** of the row-#1 disposition. Writing it now is structurally premature.

## What is already locked at row #2 (does NOT depend on row-#1 disposition)

The row-#2 surfaces below are stable across all three shapes and are already documented at [`dsl/std/runtime/bin_shims/README.md`](../../dsl/std/runtime/bin_shims/README.md) §"Naming convention". This packet does not duplicate that authority; it points at it as the canonical row-#2 reference until disposition unblocks the rest:

- **File path:** `dsl/std/runtime/bin_shims/<bin_name>.dag` (one declaration per file; one file per existing hand-Rust bin under `src/v3/compiler/src/bin/`). For `regen_lens.rs`: `dsl/std/runtime/bin_shims/regen_lens.dag`.
- **Module:** `module std.runtime.bin_shims.<bin_name>` (mirrors path).
- **Declaration name:** `data <bin_name>_shim: BinShim = { … }` — `<bin_name>` matches the bin's existing hand-Rust filename without `.rs`. For `regen_lens.rs`: `data regen_lens_shim: BinShim = { … }`.
- **Carrier shape (locked, three-field):** `entrypoint_name: NonEmptyStr`, `description: String`, `entry: DeclarationRef` — pinned by the ratchet `bin_shim_carrier_has_locked_three_field_shape` at [`src/v3/compiler/tests/integration/m2_substrate_inhabitance_test.rs`](../../src/v3/compiler/tests/integration/m2_substrate_inhabitance_test.rs).
- **Required imports for any per-shim row:** `import v3.std.bin_shim { BinShim }`, `import std.process { ProcessExit }`, plus the `<bin_name>_main` entry-function declaration (the row-#1 surface).
- **Trivially expressible row-#2 fields:** `entrypoint_name` (per Cargo `[[bin]] name = "regen_lens"`) and `description` (per the bin's existing docstring). The `entry` field is the row-#1-gated one.

## Re-dispatch criteria

Re-dispatch the row-#2 convention packet after **all** of:

1. PR #1511 (Shape A/B/C decision packet) is merged into `main`.
2. Director / Substrate Manager record the §P1 disposition (chosen shape + module path + placeholder-rejection-window OR loader-allow-list shape OR Item-4 milestone gate) per the recommended-next-unblock step in PR #1511.
3. PB Manager confirms the recorded disposition is sufficient to make the row-#2-specific convention questions answerable without further authority calls.

**Important — row-#1 merge alone is insufficient.** Merging PR #1511 only lands the *decision packet*, not the *decision*. Row #2 does **not** become independently authorable on row-#1 merge; it still depends on the §P1 record of (a) the chosen shape (A vs B vs C) and (b) the module-path decision that follows from that shape. A row-#2 packet authored on the strength of row-#1 merge alone — without the recorded shape + module-path disposition — would re-introduce the same unilateral commitment this STOP+PING is preventing.

If any of (1)–(3) are missing at re-dispatch time, the row-#2 packet remains STOP+PING.

## What this packet does NOT do

- Does not author `data regen_lens_shim: BinShim` in any form, even a "draft" or "to be filled in" stub — that would commit to a shape under one of A/B/C.
- Does not duplicate the locked naming-convention content already in `dsl/std/runtime/bin_shims/README.md` §"Naming convention".
- Does not edit the carrier (`src/v3/std/bin_shim.dag`).
- Does not extend the blocker ledger — the row-#2 ledger row already exists at [`docs/briefs/r3-pb-binshim-blocker-ledger.md`](r3-pb-binshim-blocker-ledger.md) row #2 with current state + owner + blocking authority; this packet is the explanation for why a row-#2 *convention* packet is structurally gated, not a replacement for the ledger row.

## Cross-refs

- Row-#1 decision packet (gating prerequisite): [`docs/briefs/r3-pb-binshim-row-1-decision-packet.md`](r3-pb-binshim-row-1-decision-packet.md) (PR [#1511](https://github.com/gunb-ai/gunbc/pull/1511)).
- Blocker ledger (chain ordering): [`docs/briefs/r3-pb-binshim-blocker-ledger.md`](r3-pb-binshim-blocker-ledger.md) row #2.
- Locked row-#2 surfaces (file path / module / declaration name / imports): [`dsl/std/runtime/bin_shims/README.md`](../../dsl/std/runtime/bin_shims/README.md) §"Naming convention".
- Published STOP+PING that gates row-#1 disposition: [`dsl/std/runtime/bin_shims/README.md`](../../dsl/std/runtime/bin_shims/README.md) §"Substrate prerequisite (STOP+PING — refreshed post-#1361)".
- Shape A/B/C menu source: [`docs/briefs/r3-pb-binshim-emitter-readiness.md`](r3-pb-binshim-emitter-readiness.md) §"Implementation slice STOP".
- Carrier (locked, three-field) + scaffold comment deferring `DeclarationRef<fn () -> ProcessExit>` refinement: [`src/v3/std/bin_shim.dag`](../../src/v3/std/bin_shim.dag) lines 14–23.
- Carrier ratchet: [`src/v3/compiler/tests/integration/m2_substrate_inhabitance_test.rs`](../../src/v3/compiler/tests/integration/m2_substrate_inhabitance_test.rs) (`bin_shim_carrier_has_locked_three_field_shape`).
