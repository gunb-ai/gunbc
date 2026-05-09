# T-FixedPoint P3+ readiness re-audit (slice beyond P1 / P2)

**Status:** READINESS NOTE (planning artifact; not a dispatch order).  
**Date:** 2026-05-02  
**Trigger:** PB continuation audit after the **P1 / P2** slice in `docs/briefs/r3-pb-t-fixedpoint-worker.md` §"Post-R2 / R3-continuation execution matrix" (optional cross-read: `docs/briefs/r3-pb-t-fixedpoint-p1-p2-readiness-2026-05-02.md` when that file exists on your branch). This note’s next slice is **P3** (T-FixedPoint worker cadence) and the **DB-8 transitions (1)–(4)** on top of the landed ratchet.

**Authority (repo-root paths — spellings for navigation, not an existence claim):** `docs/briefs/r3-pb-t-fixedpoint-worker.md` (matrix §P3, §Dispatch preconditions (1)–(4), §Relationship to DB-8 transitions 1–4, §Acceptance gate); `docs/r3-structure.md` (joint R3 worker dispatch precondition); mechanical signals below; optional carried-forward STOP text in `docs/briefs/r3-pb-t-fixedpoint-p1-p2-readiness-2026-05-02.md`.

### Path gates (same branch as this note)

**Mechanical gate — DB-8 / staging signals (compiler + CI + std schema):** these three paths are the **only** files the transition table below reads literally for **(1)–(3)**. From repo root:

```bash
for p in \
  src/v3/compiler/src/bin/self_host_fixed_point.rs \
  .github/workflows/ci.yml \
  src/v3/std/verification.dag
do test -f "$p" || echo "MISSING $p"; done
```

**Brief gate — ledger / matrix narrative:** required to treat §Joint ledger + §P3 matrix text as anchored to in-repo prose (full clone or include `docs/briefs/` + `docs/r3-structure.md`):

```bash
for p in \
  docs/briefs/r3-pb-t-fixedpoint-worker.md \
  docs/briefs/r3-pb-t-fixedpoint-p1-p2-readiness-2026-05-02.md \
  docs/r3-structure.md \
  docs/briefs/r3-pb-t-fixedpoint-p3plus-readiness-2026-05-02.md
do test -f "$p" || echo "MISSING $p"; done
```

Interpretation:

- **Mechanical gate prints nothing** → rows **(1)–(3)** in §DB-8 transitions are checkable from Rust / YAML / `.dag` text on disk.
- **Mechanical gate prints `MISSING`** → do **not** treat the staging verdicts in this note as verified for this checkout.
- **Brief gate reports `MISSING`** → do **not** cite absent paths as authority; either land the missing files on your branch or read **only** from paths that passed the gate (typically `docs/briefs/r3-pb-t-fixedpoint-worker.md` alone carries P1–P3 + dispatch **(1)–(4)**). The optional P1/P2 readiness note is **not** load-bearing for the ledger STOP: the worker brief already states **P3 does not start** while P1 or P2 is incomplete.

---

## What was re-read (in-repo, current `main`)

| Surface | Role in audit |
|--------|----------------|
| `docs/briefs/r3-pb-t-fixedpoint-worker.md` | P3 row, joint dispatch ledger, DB-8 transitions 1–4, acceptance gate |
| `docs/r3-structure.md` | Director-locked joint dispatch precondition (Evaluator + Rust + Python grounding) |
| `docs/briefs/r3-pb-t-fixedpoint-p1-p2-readiness-2026-05-02.md` | Optional: carried P1/P2 STOP narrative for this continuation |
| `src/v3/compiler/src/bin/self_host_fixed_point.rs` | Staging vs full `compiler.dag` cycle (DB-8 transition 1) |
| `.github/workflows/ci.yml` | `self_host_ratchet` merge-blocking policy (DB-8 transition 2) |
| `src/v3/std/verification.dag` | Live schema: **`type TestPredicate`** (line **109**) carries **`FixedPointConverges`** (**219–222**) and **`RatchetZero`** (**223–226**); **`type TestSuite`** is separate at **307** — do not conflate the two. Confirm no `pb_self_compile_fixed_point*` substring under `src/v3/std/` (DB-8 transition 3). |

**Path spelling index (repo root — not an existence proof):** literals above duplicate the first column for search / tooling; **existence** is established only by running §Path gates, never by this sentence alone. No file-relative markdown URLs.

---

## Joint ledger + staging preconditions (authoritative STOP)

Per `docs/briefs/r3-pb-t-fixedpoint-worker.md` §Dispatch preconditions **(1)–(4)** (PB Manager ledger read), aligned with the **Evaluator + Rust + Python** floor in `docs/r3-structure.md` §"R3 worker dispatch precondition", **T-FixedPoint worker dispatch** still requires a **single** R2 Release Manager closure-ledger read showing **(1)–(4)** simultaneously (R2 close, R2-Evaluator landed and stable, T-LensProducer-Retirement / SG-0 = 0, **R2-Grounding-Rust and R2-Grounding-Python both closed**).

Where `docs/briefs/r3-pb-t-fixedpoint-p1-p2-readiness-2026-05-02.md` is present, it reaches the same P1/P2 STOP for T-FixedPoint-owned promotion work. Independently, `docs/briefs/r3-pb-t-fixedpoint-worker.md` already states **P3 does not start** while P1 or P2 is incomplete and enumerates dispatch **(1)–(4)**. This P3+ slice therefore **inherits** those STOP gates: the **ledger preconditions (1)–(4)** are still **unmet** for purposes of authorizing a bounded **P3 implementation** cadence (brief authority; not re-proven from a live ledger in this note).

---

## DB-8 transitions (1)–(4) vs current tree (staging audit)

These are the four **on-top-of-DB-8** transitions the brief assigns to T-FixedPoint (not a substitute for (1)–(4) above).

| # | Transition (brief §Relationship to DB-8) | Mechanical signal in this checkout | Verdict |
|---|--------------------------------------------|--------------------------------------|---------|
| **1** | Promote `default_fixed_point_source` → required `dsl/gunbc/compiler.dag` in `self_host_fixed_point` | Module docs still describe **pipeline snapshot** on `default_fixed_point_source`, **probe/conditional** `compiler.dag`, full slice gated on parse + re-emittable CLI (`src/v3/compiler/src/bin/self_host_fixed_point.rs`, file head). | **Not promoted** |
| **2** | Graduate `self_host_ratchet` to merge-blocking | Job `self_host_ratchet` and listed steps still use **`continue-on-error: true`** (`.github/workflows/ci.yml`, `self_host_ratchet` job and step comments). | **Not graduated** |
| **3** | Author `pb_self_compile_fixed_point_strong` `TestSuite` in `verification.dag` | **Carrier (read the right `type`):** `src/v3/std/verification.dag` declares **`type TestPredicate`** at **109** (large sum of variants — `Compiles`, `CensusBoundCheck`, …). **`FixedPointConverges`** is at **219–222** and **`RatchetZero`** at **223–226** inside that sum. **`type TestSuite`** is a **different** declaration at **307** (`name` + `claims` record) — transition (3) is *not* claiming those variants live under `TestSuite` at 109; there is no `TestSuite` at 109. **Re-anchor after churn:** `rg -n 'type TestPredicate|FixedPointConverges|RatchetZero|type TestSuite' src/v3/std/verification.dag`. **Absence:** `rg "pb_self_compile_fixed_point" src/v3/std/` returns **no** matches — no strong-suite materialization under `src/v3/std/`. *(R1 horizon `pb_self_compile_fixed_point` still appears in **fixture** `.dag` under `src/v3/compiler/tests/fixtures/`; orthogonal to the P3 std-layer gate.)* | **Not authored** |
| **4** | Row B per-target emission verifier (non-Rust) | **Requires brief gate:** `docs/briefs/r3-pb-t-fixedpoint-worker.md` §Relationship to DB-8 still names an **extension** of the DB-8 binary (or sibling step); no audit evidence in this pass that a named Row-B harness for Python/Go is landed alongside Row A. | **Not landed** (planning expectation unchanged) |

---

## P3 matrix row (brief) vs this audit

**Brief P3 row:** Joint ledger §Dispatch preconditions satisfied → deliverable *shape* is `pb_self_compile_fixed_point_strong` + second-pass byte identity + ledger close (§Acceptance gate; §Relationship to DB-8 + `self_host_fixed_point` staging).

**This audit:** Preconditions are **not** satisfied (P1/P2 + joint ledger); transitions **(1)–(4)** are all **still staging / absent** in-tree as tabulated. No bounded **P3-owned** slice (promote `compiler.dag`, flip CI, add strong `TestSuite`, Row B verifier) is **ready to implement** under the brief’s own dispatch rules without first clearing the ledger and upstream lanes.

---

## STOP / next shape (for PB dispatch)

| Question | Answer |
|----------|--------|
| Is a **bounded T-FixedPoint P3 implementation slice** (transitions 1–4) **open now**? | **STOP** — joint **ledger (1)–(4)** still authoritative; **staging** signals for (1)(2) unchanged; **(3)(4)** not present. **Docs-only** for this dispatch; no implementation PR claimed by this note. |
| Smallest **honest** next work (same as worker-brief P1/P2 dependency list)? | **Upstream:** R2-Evaluator + joint grounding; **XL:** lens producer retirement + SG-0 = 0; **Lane 1e / emit:** determinism debt per DB-8 / ratchet comments. |

Cross-reference: `docs/briefs/r3-pb-t-fixedpoint-worker.md` §Dispatch preconditions remains the single dispatch authority; re-run this style of audit when the **closure ledger**, **SG-0 census**, or **DB-8 transition** surfaces materially change.
