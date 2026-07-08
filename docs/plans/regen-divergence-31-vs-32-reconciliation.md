# Regen divergence 31 vs 32 — historical audit record

**Status: HISTORICAL AUDIT RECORD** (loyal-owl-307, 2026-07-07). The interim
`regen_divergence` ratchet and `ci_regen_ratchet` job were **removed** by operator
directive, landed in **#6355** (`e470393dcd` — deletes `dag/tools/regen_divergence_ratchet.dag`,
regenerates `ci.yml` to `ci` / `ci_regen` / `rust_tests` only). This document retains
standing value as the execution-grounded audit of the counting-universe split that
informed that decision: baseline 31 was pinned against a stale pre-#6348 measurement
while executed actual on main was 30; the ratchet was judged overcomplicated relative to
the finding.

Authority for the receipts below: `regen_stage0 --verify` / `--emit-fresh --verify`
execution on `main@1a79ce1759` (pre-#6355).

## Two counters, one oracle family

| Counter | What it measured | Where it lived |
| --- | --- | --- |
| **`regen_divergence_count` (ratchet baseline: 31)** | Byte mismatches over **`GENERATED_STAGE0_FILES` only** (`verify_stage0_matches`, `regen_stage0.rs:1215–1231`) | Structured stdout line; baseline in `dag/tools/regen_divergence_ratchet.dag` (**deleted #6355**) |
| **32 managed files (lively-raven close-out scope)** | Files accept-fresh reconciled before the gen-2 fixpoint | lively-raven-355 session report |

These were **not the same denominator**. The +1 was expected, not a counting bug.

## The one-file delta (31 → 32)

**`main.rs`** — the sole `HAND_MAINTAINED` file with **DRIFT** on `--emit-fresh --verify`:

```
regen_stage0 hand-maintained verification: 0 match / 1 drift / 7 no-candidate / 0 unverifiable
  DRIFT: main.rs
```

- `regen_divergence_count` **excluded** `HAND_MAINTAINED_STAGE0_FILES` by construction.
- lively-raven's accept-fresh lane **included** `main.rs` (im_rc `Vector` prelude / newline-escaping conventions; gen-2 rebuild closed the loop).
- The other seven hand-maintained pins were **NO CANDIDATE** — terminal host-physics.

So: **32 managed = 31 `regen_divergence_count` + 1 hand-maintained drift (`main.rs`)**.

## Stale-pin finding (drove ratchet removal)

Executed on `main@1a79ce1759`:

```bash
regen_stage0 --emit-fresh <dir> --verify
# regen_divergence_count=30
# hand-maintained: 1 drift (main.rs)
```

Ratchet baseline was pinned **31** at #6352 landing; executed actual was **30** on the
same SHA (CI run `28887607471`, tightness arm). Hypothesis verified: baseline measured
pre-#6348; #6348 (`emit -> realization`) changed emitter output, moving actual. The
ratchet's tightness echo did not print observed `actual=` (diagnostic gap identified
during this audit; moot after ratchet deletion).

Leading hypothesis on the 31-vs-32 question: lively-raven's **32** count was post-#6348
actual on the managed-files axis (31 GENERATED + `main.rs`), not a ratchet arithmetic error.

## Doc-vs-config drift (crisp-bear handover, 2026-07-07)

The two-job split notes **declared** the `regen_divergence` ratchet a REQUIRED branch-protection
check, but that was **never realized in config**. Verified via operator UI + rulesets API
`16178731`: the only protection-required check is **`ci`** (the compile wall). So the ratchet
red was **status-red, never merge-blocking**; its removal in #6355 changed no required-check
surface. This explains why main could carry `ci_regen_ratchet` failures while other PRs merged
on `ci` + `rust_tests` — and why the hour spent treating ratchet tightness as the merge
blocker was reasoning from the wrong premise (substrate doc aspiration ≠ live branch rules).

## lively-raven #6357 verification checklist (live items)

Ratchet/baseline atomicity items are **moot** (no baseline exists post-#6355). Remaining
live gates for approval:

1. PR body cites **gen-2 fixpoint receipt** (`regen_stage0 --verify` EXIT=0, byte-identical).
2. **`cargo test --workspace`** green or each skip/ignore explicitly dispositioned — not a silent pass if result is missing.
3. `fail_closed_non_dag_file_forces_run_all` **deleted**, naming sibling `non_dag_only_diff_is_structural_empty_frontier_not_refusal` (#6269).
4. **sigterm** disposition line present (pre-existing #6256, `phase_profile` untouched).
5. All **non-seed files** accounted in the PR body (count grew with `.dag` boundary fixes: `stage0_crates.dag`, `05_emit_rust.dag`, etc.).

Reviewer lane only — do not re-run accept-fresh.
