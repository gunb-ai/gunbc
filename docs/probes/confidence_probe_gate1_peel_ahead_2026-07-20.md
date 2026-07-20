# Gate 1 Peel-Ahead: libc Dependency (throwaway, NOT landing)

**Session:** swift-bee-614  
**HEAD:** `73eea76dd7`  
**Purpose:** Diagnostic only — verify Gate 1 blocker is a single harness manifest gap.

## Experiment

1. `regen_stage0 --emit-fresh <tmpdir>` (baseline: 3 × E0433)
2. Add `libc = "0.2"` to `[dependencies]` in assembled `Cargo.toml`
3. `cargo check`

## Result

| Step | Errors | E-Codes |
|------|-------:|---------|
| Baseline (no libc) | 3 | E0433 × 3 |
| Peel-ahead (+ libc) | **0** | — |

**Verdict:** Gate 1 emitter fixed point is **cargo-green** once the emit-fresh manifest includes `libc`. The 3 baseline errors are 100% attributable to a missing host-physics crate dep in the assembled `Cargo.toml`, not emitter surface defects.

## NOT landing

This fix is documented for confidence only. The proper fix belongs in the emit-fresh manifest assembly lane (`regen_stage0` / crate-layout authority), not in this diagnostic session.
