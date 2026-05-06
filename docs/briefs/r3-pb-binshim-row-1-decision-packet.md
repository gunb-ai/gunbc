# R3 PB — BinShim row #1 entry-function decision packet (Shape A/B/C disposition input)

**Status:** DECISION-PACKET (docs-only; no implementation, no entry function, no instance row, no emitter, no §7.2 runnable claim, no carrier edits). Authored 2026-05-02 by R3 PB continuation (witty-tern-193) per inbox #1134 dispatch. Verified against `origin/main` HEAD `9cf6dd223` ("feat(evaluator): PR-E E7 — public-API integration tests for analyze_complexity", #1503).

**Owning manager:** Pure Bootstrap Manager (R2 → R3 continuation). This packet **routes a [P1](../../INVARIANTS.md#p1-modeling-faithfulness) question to Director / Substrate Manager / PB Manager**; it does not unilaterally pick a shape.

**Purpose:** the head-of-chain blocker for the BinShim / `regen_lens` retirement chain is row #1 of [`docs/briefs/r3-pb-binshim-blocker-ledger.md`](r3-pb-binshim-blocker-ledger.md): no `regen_lens_main` (`fn` or `func`) entry function exists on main, and the published STOP at [`dsl/std/runtime/bin_shims/README.md`](../../dsl/std/runtime/bin_shims/README.md) §"Substrate prerequisite (STOP+PING — refreshed post-#1361)" rejects PB inventing even a fail-closed placeholder without [P1](../../INVARIANTS.md#p1-modeling-faithfulness) disposition. The Shape A / Shape B / Shape C menu is sketched in [`docs/briefs/r3-pb-binshim-emitter-readiness.md`](r3-pb-binshim-emitter-readiness.md) §"Implementation slice STOP" but never normalized into a single decision artifact. This packet is that artifact.

**Verified row-#1 state (re-run on `origin/main` HEAD `9cf6dd223`):**

```bash
$ rg -n "^(fn|func) regen_lens_main|^(fn|func) .*_main.*ProcessExit" src/v3 dsl
# (no match — row-#1 NOT-LIVE verdict unchanged)
$ ls dsl/std/runtime/bin_shims/
README.md
```

No real row-#1 implementation has become unblocked on main. This packet remains the right next slice.

## Shape comparison (one row per option)

| Field | **Shape A — fail-closed placeholder convention** | **Shape B — loader-first allow-list** | **Shape C — Item 4-first real body** |
|---|---|---|---|
| **One-line summary** | Director + Substrate record [P1](../../INVARIANTS.md#p1-modeling-faithfulness) OK for a single fail-closed `regen_lens_main() -> ProcessExit` (e.g. `exit_failure("regen_lens_main not yet authored — Item 4 STOP")`); chain proceeds with vacuous behavior until Item 4 lands. | Substrate + build extend the `regen_bootstrap` full-bootstrap glob with a narrow `dsl/std/runtime/bin_shims/*.dag` allow-list, **then** instance row + entry function are co-authored under that loader. | Wait for the PB-Runtime interpreter-as-data slice (Item 4 of design doc §5.4) to reach a state where a real `regen_lens_main` body — folding the existing regen pipeline as a `Lens<C>` — is authorable directly. |
| **Owner / next mover** | **Director** (placeholder-convention [P1](../../INVARIANTS.md#p1-modeling-faithfulness) call) + **Substrate Manager** (carrier/process invariant cosign) → then PB Manager dispatches placeholder-authoring worker. | **Substrate Manager** (loader/build owner) + **build-system owner** for `regen_bootstrap.rs` glob extension → then PB Manager dispatches co-authored row-1 + row-2 worker. | **Item 4 program** (PB-Runtime interpreter-as-data) — runs to a milestone where `Lens<C>`-fold of the regen pipeline is expressible → then PB Manager dispatches real-body row-1 worker. |
| **Exact file / module surface** | `src/v3/std/<...>.dag` **or** runtime subtree **after** loader extension — exact module path is part of the [P1](../../INVARIANTS.md#p1-modeling-faithfulness) disposition (the readiness brief explicitly does not pre-pick). Body shape: single `func regen_lens_main() -> ProcessExit { exit_failure("…") }` using `dsl/std/process.dag:50` helper or `ExitFailure` literal at `dsl/std/process.dag:41`. | `src/v3/compiler/src/bin/regen_bootstrap.rs` (full-bootstrap label list around lines 70–80 per emitter-readiness brief §"Loader / build audit") **plus** new `dsl/std/runtime/bin_shims/regen_lens.dag` co-authoring `regen_lens_main` + `data regen_lens_shim: BinShim`. | `dsl/std/runtime/bin_shims/regen_lens.dag` (when authored) — `func regen_lens_main() -> ProcessExit` body composed against the live PB-Runtime fold vocabulary. Loader path co-resolves with whichever Item 4 / Substrate sequencing lands. |
| **Prerequisite (what must already be true)** | `BinShim` carrier (LIVE), `ProcessExit` (LIVE at `dsl/std/process.dag:39`), framework directory (LIVE). The **only** missing prerequisite is the [P1](../../INVARIANTS.md#p1-modeling-faithfulness) disposition itself — a written Director + Substrate decision recorded in the [P1 Modeling Faithfulness](../../INVARIANTS.md#p1-modeling-faithfulness) ledger or an equivalent authority doc, naming the helper (`exit_failure(...)`) AND the module path AND the placeholder-rejection-window (i.e. when this becomes a hard error, not a permanent stub). | Substrate Manager + build owner agreement on glob extension shape (allow-list path pattern, ordering relative to existing `dsl/std/*.dag` and `src/v3/std/*.dag` globs, regen_bootstrap snapshot regeneration). No Director-level [P1](../../INVARIANTS.md#p1-modeling-faithfulness) placeholder convention required if the entry function lands as a real body, but loader allow-list itself is a Substrate [P1](../../INVARIANTS.md#p1-modeling-faithfulness) question (introduces a new authority directory to bootstrap). | Item 4 (PB-Runtime interpreter-as-data) progress to a sub-gate where `Lens<C>`-fold of an external pipeline (file-load → compile → file-write) is expressible. Per `r3-pb-binshim-retirement-worker.md` §"Dependencies / gates" item 2: "Item 5 (this lane) inherits Item 4's runtime-value vocabulary." |
| **Does this permit `regen_lens_shim` (row #2)?** | **Yes, but vacuously.** With a fail-closed placeholder body, `data regen_lens_shim: BinShim = { entry: regen_lens_main, ... }` resolves; the emitter (row #3) could be exercised against vacuous behavior; §7.2 equivalence (row #4) **cannot** meaningfully claim "PB-Runtime ≡ emitted-Rust" against a placeholder, so row #4 still blocks until real body lands — Shape A unblocks rows #2–#3 but not #4. | **Yes, with real body.** If row #1 + row #2 are co-authored under the new loader, the chain advances on real semantics rather than placeholder semantics. But Shape B does not by itself make a real `regen_lens_main` body authorable — that still depends on whether the regen pipeline is expressible in the current `.dag` vocabulary (overlap with Shape C's Item 4 question). | **Yes, with real body.** Row #2 follows naturally because `entry: regen_lens_main` resolves to a live behavioral function; row #4 (§7.2 equivalence) becomes a meaningful claim because PB-Runtime evaluation of the body equals what the hand-Rust bin does today. Shape C is the only shape that unblocks the **entire** chain through row #4 in a single move. |
| **STOP condition (when this shape is the wrong call)** | STOP if Director / Substrate decline to bless any placeholder convention (e.g. invariants doc treats `exit_failure`-stub-as-entrypoint as a forbidden bridge per `feedback_no_textual_enforcement_bridges` / `feedback_executable_emission` family) **or** if the placeholder-rejection-window cannot be made concrete (no Item-4 ETA → permanent stub risk). | STOP if Substrate / build owner finds the loader-glob extension non-trivial (e.g. introduces ordering hazards with the existing `bootstrap_generated*.rs` snapshot test; risks regen-bootstrap drift) **or** if a real `regen_lens_main` body still cannot be authored under the new loader (fall back to A or C). | STOP if Item 4's earliest milestone for "external-pipeline-`Lens<C>`-fold" is far enough out that downstream chain remains gated indefinitely; if so, Shape A becomes the bridge to keep the chain moving while Shape C is the eventual destination. |

## Recommended next unblock

The existing docs **do not unilaterally support a single shape**. The README STOP+PING (`dsl/std/runtime/bin_shims/README.md` §"Substrate prerequisite") explicitly routes the placeholder-convention question to "Director / Substrate Manager / PB Manager call" — i.e. Shape A is **conditional on a [P1](../../INVARIANTS.md#p1-modeling-faithfulness) disposition that has not been recorded**. The emitter-readiness brief (`r3-pb-binshim-emitter-readiness.md` §"Implementation slice STOP") presents Shape A/B/C as a non-ordered menu and does not pick.

**Therefore the recommended next unblock is not a Shape choice — it is a Director / Substrate disposition step:**

1. **Director** records (e.g. in the [P1 Modeling Faithfulness](../../INVARIANTS.md#p1-modeling-faithfulness) ledger or a `docs/decisions/` entry referenced from `INVARIANTS.md`) which of the three shapes is the sanctioned next move for `regen_lens_main`, with explicit acknowledgment of:
   - whether `exit_failure(...)`-stub-as-entrypoint is or is not a forbidden bridge under existing closed-system / executable-emission discipline (feedback memory series: `feedback_no_textual_enforcement_bridges`, `feedback_executable_emission`, `feedback_construction_over_ratchets`);
   - the placeholder-rejection-window (Shape A) **or** the loader-allow-list shape + snapshot story (Shape B) **or** the Item-4 milestone gate (Shape C);
   - the module path the chosen shape lands at.
2. **Substrate Manager** cosigns: (a) carrier-shape unchanged in all three shapes (already locked at `src/v3/std/bin_shim.dag:19`, three-field ratchet `bin_shim_carrier_has_locked_three_field_shape` at `src/v3/compiler/tests/integration/m2_substrate_inhabitance_test.rs`), and (b) for Shape B specifically, the loader allow-list does not violate the `dsl/std/runtime/**` is-not-bootstrapped invariant cited in the emitter-readiness brief without a tracked snapshot regen.
3. **PB Manager** dispatches the chosen-shape worker only after (1) and (2) are recorded on main.

Until step (1) lands, **PB cannot make a unilateral move on row #1** without violating the published STOP+PING. The smallest PB-side action that does not require Director/Substrate disposition is **this packet** — surface the decision in one place so the dispositioning conversation can run on a single artifact rather than re-deriving the menu each time.

## What this packet does NOT do

- Does not pick Shape A vs B vs C — that is Director + Substrate authority per the README STOP+PING.
- Does not author `regen_lens_main` (any shape), `data regen_lens_shim`, the BinShim emitter, the §7.2 runnable `TestClaim` / comparison script, `REGEN_OUTPUTS` edits, or any retirement.
- Does not edit `src/v3/std/bin_shim.dag` or any other carrier.
- Does not introduce new substrate facts; every claim cites an existing brief / README / source-file authority.

## Cross-refs

- Head-of-chain blocker context: [`docs/briefs/r3-pb-binshim-blocker-ledger.md`](r3-pb-binshim-blocker-ledger.md) row #1.
- Framework / instance authority + STOP+PING: [`dsl/std/runtime/bin_shims/README.md`](../../dsl/std/runtime/bin_shims/README.md) §"Substrate prerequisite (STOP+PING — refreshed post-#1361)".
- Shape A/B/C menu source: [`docs/briefs/r3-pb-binshim-emitter-readiness.md`](r3-pb-binshim-emitter-readiness.md) §"Implementation slice STOP" + §"Smallest next-unblock PR shapes".
- Parent program brief: [`docs/briefs/r3-pb-binshim-retirement-worker.md`](r3-pb-binshim-retirement-worker.md) §"Dependencies / gates", §"First slice — `regen_lens.rs`".
- Carrier (locked, three-field): [`src/v3/std/bin_shim.dag:19`](../../src/v3/std/bin_shim.dag).
- Carrier ratchet: [`src/v3/compiler/tests/integration/m2_substrate_inhabitance_test.rs`](../../src/v3/compiler/tests/integration/m2_substrate_inhabitance_test.rs) (`bin_shim_carrier_has_locked_three_field_shape`).
- `ProcessExit` substrate (Shape A helpers): [`dsl/std/process.dag`](../../dsl/std/process.dag) lines 39 (`type ProcessExit`), 41 (`ExitFailure` literal), 50 (`exit_failure(...)` helper).
- [P1](../../INVARIANTS.md#p1-modeling-faithfulness) substrate-fact-introduction procedure: [`INVARIANTS.md`](../../INVARIANTS.md).
- Design lock: [`docs/design-pb-runtime-interpreter.md`](../design-pb-runtime-interpreter.md) §4.2 (carrier sketch + emitter shape), §4.3 (dissolution path), §5.4 (PB / Substrate / Evaluator boundary), §6 anti-bridge invariant #4.
