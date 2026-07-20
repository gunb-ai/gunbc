# v1-Delete Bar Confidence Probe — Phase 1 Report

**Session:** swift-bee-614 (confidence-probe lane)  
**Parent:** sharp-bee-290 (Weak → Strong Self Host)  
**HEAD:** `73eea76dd7c9`  
**Probed at:** 2026-07-20T16:30:00Z  
**Scope:** READ-ONLY diagnostics, NO fixes landed.

---

## Executive Summary

| Gate | Question | Verdict | Confidence |
|------|----------|---------|------------|
| **Gate 1** — Emitter fixed point | Does faithful `--emit-fresh` cargo-check? | **GREEN** (0 errors after libc peel-ahead; 3 E0433 baseline) | **HIGH** |
| **Gate 2** — Honest frontier | Do per-module CSSL probes cargo-green? | **0/27 green**, all refuse | **HIGH** (measurement is honest; frontier is red) |

---

## Gate 1: Emitter Fixed Point

**Harness:** `regen_stage0 --emit-fresh` + `cargo check`  
**Receipt:** [confidence_probe_gate1_histogram_2026-07-20.md](confidence_probe_gate1_histogram_2026-07-20.md)  
**Peel-ahead:** [confidence_probe_gate1_peel_ahead_2026-07-20.md](confidence_probe_gate1_peel_ahead_2026-07-20.md)

### E-Code Histogram (baseline)

| E-Code | Count | Sites | Mechanism |
|--------|------:|------:|-----------|
| E0433 | 3 | 2 | `libc` missing from emit-fresh `Cargo.toml` |

### Counted Facts

- **Total errors:** 3 (down from 1667 cited in ROADMAP 2026-07-05 pre-#6243)
- **Unique E-codes:** 1
- **Emitter surface defects:** 0
- **Hand-maintained drift:** 1 (`main.rs` DRIFT), 27 NO-CANDIDATE (host-physics pins)
- **Peel-ahead (libc added):** 0 errors → **cargo-green confirmed**

### Gate 1 Confidence: HIGH

The emitter fixed point is real. The remaining 3 errors are a manifest assembly gap, not emit-surface or Rc/Optional ownership defects.

---

## Gate 2: Honest Frontier

**Harness:** `CSSL_STD_SEED_LINK=1` + `gunbc compile` + `cssl_assemble` + `cargo build --release --lib` per module  
**Receipt:** [confidence_probe_gate2_histogram_2026-07-20.md](confidence_probe_gate2_histogram_2026-07-20.md) (aggregate)  
**Per-module:** [confidence_probe_gate2_per_module_2026-07-20.tsv](confidence_probe_gate2_per_module_2026-07-20.tsv)

### Module Verdict Distribution

| Verdict | Count | Modules |
|---------|------:|---------|
| cargo green | 0 | — |
| cargo refuse | 27 | all 27 roster modules |
| emit fail | 0 | — |
| harness refuse | 0 | — |

### Aggregate E-Code Histogram (27 modules, summed per-module — closure overlap expected)

| E-Code | Count | Blocker Class |
|--------|------:|---------------|
| E0308 | 98,168 | mismatched types (dominant — emit surface / Rc/Optional) |
| E0277 | 4,881 | trait bound not satisfied |
| E0425 | 2,752 | cannot find value in scope (namespace) |
| E0599 | 2,743 | method not found |
| E0369 | 1,386 | binary op cannot be applied |
| E0063 | 373 | missing struct fields |
| E0631 | 308 | type annotations needed |
| E0422 | 305 | struct/enum/union not found |
| E0282 | 281 | type annotations needed |
| E0433 | 269 | unresolved crate/module |
| E0107 | 258 | wrong number of type arguments |
| E0109 | 212 | type arguments not allowed |
| E0614 | 197 | type cannot be dereferenced |
| E0597 | 151 | borrowed value does not live long enough |
| E0061 | 170 | wrong number of function arguments |
| E0609 | 88 | no field on type |
| *(6 more codes ≤ 46 each)* | 112 | — |

**Total aggregate errors (per-module sum):** 112,541  
**Unique E-codes across frontier:** 22

### Per-Module Error Range

| Tier | Error Range | Module Count | Examples |
|------|------------|-------------|----------|
| Deep closure (5k–8k) | 5,000–8,000 | 14 | `00_compile`, `03_ingest`, `program_assembly` |
| Mid closure (2.5k–5k) | 2,500–5,000 | 8 | `04_infer`, `05_emit`, `06_translate` |
| Shallow (300–1.2k) | 300–1,200 | 5 | `01_tokenize`, `discovery_enumeration`, `self_host` |

### Self-Emitted Module (`03_body_producer`)

| Metric | Value |
|--------|------:|
| Cargo verdict | refuse |
| Total errors | 413 |
| Unique E-codes | 7 |
| Top codes | E0308:294, E0599:80, E0277:33 |

Self-emitted disposition does NOT imply cargo-green on the CSSL probe harness. Behavioral-equivalence receipts are a separate axis.

### Gate 2 Confidence: HIGH

The frontier is honestly red. No phantom greens. The dominant blocker is E0308 (mismatched types) across all 27 modules — consistent with emit-surface / Rc/Optional ownership gaps (#6775/#6776), not namespace-only or harness-artifact std-dup (prior `#6883`/`#6911` receipts cleared std-dup for deep lanes).

---

## Cross-Gate Analysis

```
Gate 1 (whole emit-fresh crate)     Gate 2 (per-module CSSL probes)
─────────────────────────────       ─────────────────────────────────
3 errors (libc only)                112,541 aggregate errors (22 E-codes)
0 emitter defects                   E0308 dominates (87% of counted errors)
Peel-ahead → 0 errors               0/27 modules cargo-green
```

**Interpretation:** Gate 1 and Gate 2 measure different things and are NOT contradictory.

- **Gate 1** assembles the *faithful full regen closure* (seed-linked, hand-maintained pins included) — nearly green.
- **Gate 2** probes each module's *import closure in isolation* via CSSL emit+assemble — each module's closure re-emits std types that collide with seed-linked std, producing the E0308 cascade.

The per-module probe errors are **closure-denominated** (each module's import closure is compiled independently), not a count of distinct fix sites. The honest signal is: **0 modules pass**, dominant class is **E0308**, and the self-emitted module (`03_body_producer`) has the lowest error count (413) — consistent with tractability ranking.

---

## Recommendations (informational, NOT landing)

1. **Gate 1 unblock:** Add `libc` to emit-fresh manifest assembly (1-line, manifest lane).
2. **Gate 2 burn-down:** E0308 ownership/wrap predicate (#6775/#6776 Rc-ownership wrap-decision design) is the single highest-count class across all 27 modules.
3. **E0425 namespace residue:** 2,752 counted — FreeMonoid/import-emission pass is live but not complete across all closure shapes.
4. **Do NOT use first_error probes for sizing:** Full histogram reveals E0308 is 87% of errors; first_error would surface E0432/E0425 on shallow modules and understate the dominant class.

---

## Artifacts

| File | Description |
|------|-------------|
| `confidence_probe_report_2026-07-20.md` | This report |
| `confidence_probe_gate1_histogram_2026-07-20.md` | Gate 1 full histogram |
| `confidence_probe_gate1_peel_ahead_2026-07-20.md` | Gate 1 throwaway peel-ahead (libc) |
| `confidence_probe_gate2_histogram_2026-07-20.md` | Gate 2 aggregate E-code histogram + sites |
| `confidence_probe_gate2_per_module_2026-07-20.tsv` | Gate 2 per-module histogram (slim) |
| `scripts/_throwaway_gate2_full_histogram_probe.sh` | Repro script (dissolve-on: modeled transport) |

## Reproduce

```bash
# Gate 1
OUT=$(mktemp -d) && target/release/regen_stage0 --emit-fresh "$OUT" && (cd "$OUT" && cargo check 2>&1 | rg 'error\[E')

# Gate 2 (full 27-module sweep, ~57min)
./scripts/_throwaway_gate2_full_histogram_probe.sh
```
