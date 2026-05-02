# T-FixedPoint P3+ readiness re-audit (slice beyond P1 / P2)

**Status:** READINESS NOTE (planning artifact; not a dispatch order).  
**Date:** 2026-05-02  
**Trigger:** PB continuation audit after `docs/briefs/r3-pb-t-fixedpoint-p1-p2-readiness-2026-05-02.md` — next slice is **P3** (T-FixedPoint worker cadence) and the **DB-8 transitions (1)–(4)** on top of the landed ratchet.

**Authority (repo-root paths — verify in checkout):** `docs/briefs/r3-pb-t-fixedpoint-worker.md` (matrix §P3, §Dispatch preconditions (1)–(4), §Relationship to DB-8 transitions 1–4, §Acceptance gate); `docs/r3-structure.md` (joint R3 worker dispatch precondition); mechanical signals below; prior STOP conclusions in `docs/briefs/r3-pb-t-fixedpoint-p1-p2-readiness-2026-05-02.md`.

### Authority path gate (same branch as this note)

Every path this note treats as load-bearing must exist **before** the audit sentences stand. From repo root:

```bash
for p in \
  docs/briefs/r3-pb-t-fixedpoint-worker.md \
  docs/briefs/r3-pb-t-fixedpoint-p1-p2-readiness-2026-05-02.md \
  docs/briefs/r3-pb-t-fixedpoint-p3plus-readiness-2026-05-02.md \
  docs/r3-structure.md \
  src/v3/compiler/src/bin/self_host_fixed_point.rs \
  .github/workflows/ci.yml \
  src/v3/std/verification.dag
do test -f "$p" || echo "MISSING $p"; done
```

Expect **no** `MISSING` lines. If any path is absent on your checkout, **STOP** — retarget or land the missing authority first; do not treat this note as evidence.

---

## What was re-read (in-repo, current `main`)

| Surface | Role in audit |
|--------|----------------|
| `docs/briefs/r3-pb-t-fixedpoint-worker.md` | P3 row, joint dispatch ledger, DB-8 transitions 1–4, acceptance gate |
| `docs/r3-structure.md` | Director-locked joint dispatch precondition (Evaluator + Rust + Python grounding) |
| `docs/briefs/r3-pb-t-fixedpoint-p1-p2-readiness-2026-05-02.md` | Carried P1/P2 STOP baseline for this continuation |
| `src/v3/compiler/src/bin/self_host_fixed_point.rs` | Staging vs full `compiler.dag` cycle (DB-8 transition 1) |
| `.github/workflows/ci.yml` | `self_host_ratchet` merge-blocking policy (DB-8 transition 2) |
| `src/v3/std/verification.dag` | Live `TestPredicate` carrier: confirm `FixedPointConverges` / `RatchetZero` variant lines; confirm no strong-suite string in **this** file (DB-8 transition 3) |

**Live path inventory (repo root):** same paths as §Authority path gate above — duplicates the table’s first column so each path is spelled literally once here. No file-relative markdown URLs.

---

## Joint ledger + staging preconditions (authoritative STOP)

Per `docs/briefs/r3-pb-t-fixedpoint-worker.md` §Dispatch preconditions **(1)–(4)** (PB Manager ledger read), aligned with the **Evaluator + Rust + Python** floor in `docs/r3-structure.md` §"R3 worker dispatch precondition", **T-FixedPoint worker dispatch** still requires a **single** R2 Release Manager closure-ledger read showing **(1)–(4)** simultaneously (R2 close, R2-Evaluator landed and stable, T-LensProducer-Retirement / SG-0 = 0, **R2-Grounding-Rust and R2-Grounding-Python both closed**).

The P1/P2 readiness note already concluded **P1** and **P2** are **not** clear for T-FixedPoint-owned promotion work. The worker brief explicitly states **P3 does not start** while P1 or P2 is incomplete. This P3+ slice therefore **inherits** those STOP gates: the **ledger preconditions (1)–(4)** are still **unmet** for purposes of authorizing a bounded **P3 implementation** cadence.

---

## DB-8 transitions (1)–(4) vs current tree (staging audit)

These are the four **on-top-of-DB-8** transitions the brief assigns to T-FixedPoint (not a substitute for (1)–(4) above).

| # | Transition (brief §Relationship to DB-8) | Mechanical signal in this checkout | Verdict |
|---|--------------------------------------------|--------------------------------------|---------|
| **1** | Promote `default_fixed_point_source` → required `dsl/gunbc/compiler.dag` in `self_host_fixed_point` | Module docs still describe **pipeline snapshot** on `default_fixed_point_source`, **probe/conditional** `compiler.dag`, full slice gated on parse + re-emittable CLI (`src/v3/compiler/src/bin/self_host_fixed_point.rs`, file head). | **Not promoted** |
| **2** | Graduate `self_host_ratchet` to merge-blocking | Job `self_host_ratchet` and listed steps still use **`continue-on-error: true`** (`.github/workflows/ci.yml`, `self_host_ratchet` job and step comments). | **Not graduated** |
| **3** | Author `pb_self_compile_fixed_point_strong` `TestSuite` in `verification.dag` | **Carrier:** `src/v3/std/verification.dag` defines `type TestPredicate` (sum starts ≈109) with **many** variants (`Compiles`, `CensusBoundCheck`, … — not a two-variant file). The worker brief’s **P0 substrate pin** for the *future* strong suite is that **`FixedPointConverges`** and **`RatchetZero`** exist in that sum; in this checkout they appear at **219–226**. **Absence (transition 3):** `rg "pb_self_compile_fixed_point" src/v3/std/verification.dag` and `rg "pb_self_compile_fixed_point" src/v3/std/` return **no** matches — no `pb_self_compile_fixed_point_strong` materialization under `src/v3/std/`. *(R1 horizon `pb_self_compile_fixed_point` still appears in **fixture** `.dag` under `src/v3/compiler/tests/fixtures/`; that is orthogonal to the P3 “strong suite in std layer” gate.)* | **Not authored** |
| **4** | Row B per-target emission verifier (non-Rust) | Brief still names an **extension** of the DB-8 binary (or sibling step); no audit evidence in this pass that a named Row-B harness for Python/Go is landed alongside Row A. | **Not landed** (planning expectation unchanged) |

---

## P3 matrix row (brief) vs this audit

**Brief P3 row:** Joint ledger §Dispatch preconditions satisfied → deliverable *shape* is `pb_self_compile_fixed_point_strong` + second-pass byte identity + ledger close (§Acceptance gate; §Relationship to DB-8 + `self_host_fixed_point` staging).

**This audit:** Preconditions are **not** satisfied (P1/P2 + joint ledger); transitions **(1)–(4)** are all **still staging / absent** in-tree as tabulated. No bounded **P3-owned** slice (promote `compiler.dag`, flip CI, add strong `TestSuite`, Row B verifier) is **ready to implement** under the brief’s own dispatch rules without first clearing the ledger and upstream lanes.

---

## STOP / next shape (for PB dispatch)

| Question | Answer |
|----------|--------|
| Is a **bounded T-FixedPoint P3 implementation slice** (transitions 1–4) **open now**? | **STOP** — joint **ledger (1)–(4)** still authoritative; **staging** signals for (1)(2) unchanged; **(3)(4)** not present. **Docs-only** for this dispatch; no implementation PR claimed by this note. |
| Smallest **honest** next work (unchanged from P1/P2 note)? | **Upstream:** R2-Evaluator + joint grounding; **XL:** lens producer retirement + SG-0 = 0; **Lane 1e / emit:** determinism debt per DB-8 / ratchet comments. |

Cross-reference: `docs/briefs/r3-pb-t-fixedpoint-worker.md` §Dispatch preconditions remains the single dispatch authority; re-run this style of audit when the **closure ledger**, **SG-0 census**, or **DB-8 transition** surfaces materially change.
