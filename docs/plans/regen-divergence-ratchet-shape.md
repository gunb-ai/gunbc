# RED-job counted-divergence ratchet — shape note (#6350)

The two-job split moves `regen_verify` + `self_host` staleness-arm into a **non-required RED job**.
A non-required red job is, by default, an *escape hatch*: debt can grow silently behind it (§5 absorbing
fallback — the deficit's frequency zeroed by construction). The ratchet is what makes the split §5-legal.
It is **not** optional for #6350.

## The three properties (clever-koi's review checks)

- **(a) monotone-shrink guard** — a *growing* divergence count is a hard failure, not silently absorbed.
- **(b) auto-promote at 0** — when divergence == 0 the gate is a normal hard gate again, bound to the
  byte-diff==0 oracle, with **no manual flip** anywhere in the path.
- **(c) count is a real execution output** — the number is `regen`'s actual `mismatches.len()`, not a
  re-asserted literal.

## Count source (grounds (c))

`regen_stage0 --verify` (`verify_stage0_matches`, regen_stage0.rs:1213) already computes
`mismatches: Vec<String>` over `GENERATED_STAGE0_FILES` and, on divergence, returns
`Err("Stage0 is stale: {mismatches.len()} generated file(s) differ …")`. The count exists; today only
`result.success` (Bool) escapes the transport (`run_regen_verify`, regen_verify_transport.dag:43).

**Change:** regen_stage0 emits the count as a *structured* line (`regen_divergence_count=<N>` on a stable
stream), and the ratchet transport captures `<N>` as an `Int`. This is a real per-run execution output —
NOT stdout heuristic-scraping of the human message, NOT a literal. (Alternative considered and rejected:
parse "N generated file(s) differ" from the prose message — that is a nickname/heuristic, §4-illegal.)

## Baseline (single authority)

`data regen_divergence_baseline: Int = 31` — one committed number, the sanctioned divergence.
Lowering it is the debt-paydown action; it is the *only* place the sanctioned count lives.

## The gates

Two distinct checks, deliberately in different jobs:

1. **RED job (non-required):** runs the full `regen_verify` byte-identical check → honest RED, shows the
   current divergence (the 31-file list). Visible, tracked, non-blocking. This is the *audit* — it reports,
   it never greens.

2. **GREEN job (required): `regen_divergence_ratchet`** — the enforcement.
   - `actual := regen_divergence_count()` (real execution output, (c))
   - PASS iff `actual <= regen_divergence_baseline`.
   - GREEN today (31 <= 31). A new seed hand-edit that grows divergence → `actual = 32 > 31` → **FAIL**,
     and because this check is *required*, it blocks merge. That is (a): growth fails, in the required lane,
     so the non-required red job cannot absorb it.
   - When `regen_divergence_baseline` is lowered to `0` (debt fully paid), the required check becomes
     `actual <= 0` ⟺ `actual == 0` ⟺ byte-identical ⟺ the original hard gate. **(b)**: auto-promotion is
     structural — the gate hardens the moment the baseline reaches 0, no manual flip, bound to the same
     `mismatches`-empty oracle.

## The one open decision for review — baseline-raise

`actual <= baseline` alone has a loophole: raising the baseline (31 → 32) lets `actual` grow without
failing. Growth is then *not* enforced. Options:

- **B1 (proposed): tightness + main-monotone.** Require `actual == baseline` (baseline must track actual
  exactly, so every divergence change is a visible baseline edit) AND a small required check
  `baseline(HEAD) <= baseline(origin/main)` (drift-style, reads main's committed value via the git fetch the
  floor already does). Baseline may only shrink; growth is impossible without failing the main-monotone check.
  Fully mechanical, no reviewer diligence.
- **B2 (lighter): diff-visibility only.** `actual <= baseline`; a baseline raise is a visible one-line diff
  in CI substrate that review catches. Mechanically weaker (relies on the reviewer), but simpler.

I lean **B1** — §5 wants mechanical enforcement, not reviewer diligence; a baseline-raise loophole is
exactly the "one config edit from silent" escape hatch the operator ruled against (2026-07-05). Flagging
for your call since it adds the `baseline(main)` read.

## What is NOT in this note

The self_host staleness-arm shares the same oracle (byte-diff==0) but is a separate witness
(`self_host_realized_comparison_staleness_gate_holds`). It rides the RED job for visibility; whether it
also gets a counted ratchet or just tracks the regen ratchet's promotion is a secondary question — regen is
the primary counted surface. Proposed: one ratchet (regen's count) governs promotion of the whole
self-host-fixed-point class, since both arms dissolve on the same `fresh-emit == committed-seed` event.
