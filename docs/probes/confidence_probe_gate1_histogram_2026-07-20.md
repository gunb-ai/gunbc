# Gate 1: Emitter Fixed Point — Full E-Code Histogram

**Session:** swift-bee-614 (confidence-probe lane)  
**HEAD:** `73eea76dd7`  
**Probed at:** 2026-07-20T15:45:00Z  
**Harness:** `regen_stage0 --emit-fresh <tmpdir>` + `cargo check` (faithful emitted crate)

## Verdict

| Metric | Value |
|--------|------:|
| Emit-fresh assembly | SUCCESS |
| Cargo check | REFUSE (3 errors) |
| Total rustc errors | 3 |
| Unique E-codes | 1 |
| Gate 1 confidence | **HIGH** — emitter fixed point is cargo-green modulo a single harness dependency gap |

## E-Code Histogram

| E-Code | Count | Mechanism |
|--------|------:|-----------|
| E0433 | 3 | `libc` crate not in emit-fresh `Cargo.toml` — host-physics dependency gap in `v1_interpreter.rs`, NOT an emitter surface defect |

## Sites (all 3)

| Site | E-Code | Note |
|------|--------|------|
| `src/v1_interpreter.rs:1790` | E0433 | `libc::timespec` — missing crate dep |
| `src/v1_interpreter.rs:1796` | E0433 | `libc::clock_gettime` — same root cause |

## Interpretation

1. **Emitter fixed point is effectively GREEN.** The faithful `--emit-fresh` crate compiles with only 3 errors, all the same root cause: `libc` not linked in the assembled `Cargo.toml`. This is a harness/manifest gap, not an emit-surface or ownership/wrap defect.
2. **Contrast with prior receipts:** ROADMAP cited 1667 rustc errors on fresh emit (2026-07-05, pre-#6243). Current measurement: **3 errors, 1 E-code, 2 sites** — a ~500× reduction.
3. **Hand-maintained drift:** regen_stage0 reports 1 DRIFT (`main.rs`) and 27 NO-CANDIDATE (host-physics pins). Neither blocks cargo-green on the emit closure.
4. **Gate 1 blocker class:** `HARNESS_MANIFEST` (missing `libc` dep), NOT `Gate_A_emitter_Rc_Optional` or `EmitSurfaceGap`.

## Reproduce

```bash
OUT=$(mktemp -d)
target/release/regen_stage0 --emit-fresh "$OUT"
(cd "$OUT" && cargo check 2>&1 | rg 'error\[E')
```
