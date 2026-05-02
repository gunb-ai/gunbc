# T-FixedPoint P1 / P2 readiness re-audit (PB)

**Status:** READINESS NOTE (planning artifact; not a dispatch order).  
**Date:** 2026-05-02  
**Trigger:** PB re-audit after P0 receipt work (`#1425` / `#1447` / `#1463`) and recent Evaluator / R3 expansion landings on `main`.

**Authority (repo-root paths — verify in checkout):** `docs/briefs/r3-pb-t-fixedpoint-worker.md` (execution matrix §P1–P2, §Dispatch preconditions, §Relationship to DB-8); `docs/r3-structure.md` (lane table + R3 worker dispatch precondition); mechanical signals in the table below and in §P1 / §P2.

---

## What was re-read (in-repo, current `main`)

| Surface | Role in audit |
|--------|----------------|
| `docs/briefs/r3-pb-t-fixedpoint-worker.md` | P1/P2 definitions, dispatch joint precondition, DB-8 relationship |
| `docs/r3-structure.md` | T-FixedPoint lane row, Evaluator / Lens dependencies |
| `.github/workflows/ci.yml` | `self_host_ratchet` job policy |
| `src/v3/compiler/src/bin/self_host_fixed_point.rs` | Staged vs full `compiler.dag` cycle |
| `src/v3/compiler/src/self_host_receipt_p0.rs` | P0 receipt contract (unchanged relevance) |
| `src/v3/compiler/tests/integration/sg0_census_test.rs` | SG-0 hand-authored non-test census (P2 proxy) |

**Live path inventory (repo root, verified present in-tree at authoring):** `docs/briefs/r3-pb-t-fixedpoint-worker.md`, `docs/briefs/r3-pb-t-fixedpoint-p1-p2-readiness-2026-05-02.md` (this file), `docs/r3-structure.md`, `.github/workflows/ci.yml`, `src/v3/compiler/src/bin/self_host_fixed_point.rs`, `src/v3/compiler/src/self_host_receipt_p0.rs`, `src/v3/compiler/tests/integration/sg0_census_test.rs` — duplicates the table’s first column so each path is spelled literally once here (avoids tooling that skips table cells). §P0 also cites `docs/db-history/db-8.md` and `docs/design-fixed-point-ratchet.md`. No file-relative markdown URLs.

Incremental Evaluator substrate landings (e.g. PR-E lineage called out in program mail) were **not** treated as a substitute for the brief’s **ledger-level** “R2-Evaluator landed” gate; see P1 conclusion.

---

## P1 — Evaluator substrate / runnable `compiler.dag` fixed-point cycle

**Brief definition (P1 row):** Preconditions include **R2-Evaluator landed**; deliverable *shape* is a **runnable `compiler.dag` fixed-point cycle** (see matrix + §Dependencies (1)).

**Dispatch precondition (joint):** Per `docs/briefs/r3-pb-t-fixedpoint-worker.md` §Dispatch preconditions, **T-FixedPoint worker dispatch** still waits on a closure-ledger read showing **R2-Evaluator landed and stable** *and* **R2-Grounding-Rust + R2-Grounding-Python** (and the other rows). Host-side compiler improvements alone do not rewrite that authority.

**Mechanical signal (`self_host_fixed_point`):** Module docs still state the full emit → `rustc` → run → byte-diff cycle on `dsl/gunbc/compiler.dag` requires **v3-parseable** `compiler.dag` **and** an emitted CLI that can re-run emission; until Lane 3 Stage 3c the binary remains **staged** on `default_fixed_point_source` with a **probe/conditional** `compiler.dag` slice (`src/v3/compiler/src/bin/self_host_fixed_point.rs`, module docs at file head).

**CI signal:** Job `self_host_ratchet` remains **`continue-on-error: true`** at job and step level (`.github/workflows/ci.yml`, `self_host_ratchet` job), matching the brief’s P0 pin that **P0 work does not graduate** the ratchet to merge-blocking.

**Conclusion — P1:** **Not** “green enough” to authorize a **T-FixedPoint P1 worker implementation slice** that would **promote** `compiler.dag` to a required gate, **flip** CI to blocking, or **author** `pb_self_compile_fixed_point_strong` / edit `verification.dag`—those remain **P3 / dispatch-gated** per the worker brief. Bounded **pre-dispatch** engineering may continue on upstream lanes (Evaluator program close, emit/Lane 1e determinism, parse/emit closure for `compiler.dag`); this note does not assert ledger greens.

---

## P2 — T-LensProducer-Retirement / SG-0

**Brief definition (P2 row):** Preconditions include **T-LensProducer-Retirement (XL)** + PB-1 shim pattern; three producer files retired; **SG-0 non-test = 0** census signal.

**Census signal (live literals, `EXPECTED_HAND_AUTHORED_NON_TEST`):** the ratchet array in `sg0_census_test.rs` still includes the three T-LensProducer retirement targets named in the worker brief:

- `src/v3/compiler/src/bin/regen_lens.rs`
- `src/v3/compiler/src/lens_apply.rs`
- `src/v3/compiler/src/lens_testgen.rs`

**Mechanical anchor:** `EXPECTED_HAND_AUTHORED_NON_TEST` is the Rust `const` slice in `src/v3/compiler/tests/integration/sg0_census_test.rs` (not a path on disk). Each bullet above is a **string literal** in that slice; the corresponding `.rs` files exist under `src/v3/compiler/` (e.g. `lens_apply.rs` is present at `src/v3/compiler/src/lens_apply.rs`).

Re-verify after any SG-0 census edit by searching those strings inside `EXPECTED_HAND_AUTHORED_NON_TEST` (line numbers drift; the sorted path list is the authority).

**Conclusion — P2:** **Still blocked** on T-LensProducer-Retirement / SG-0 choreography per the brief; no evidence in this audit that the XL lane’s retirement sub-gates are cleared or that SG-0 non-test has dropped to zero.

---

## P0 carry-forward (unchanged role)

**P0 code authorities (repo root, live at authoring):** `src/v3/compiler/src/self_host_receipt_p0.rs` (stable `receipt.json` top-level key literals + `validate_receipt_json_always_emitted_keys`); `src/v3/compiler/src/bin/self_host_fixed_point.rs` (DB-8 driver — assembles `target/self_host/receipt.json`, calls validate before `write_receipt`). **P0 doc authorities:** `docs/db-history/db-8.md`, `docs/design-fixed-point-ratchet.md` (plus the worker brief P0 checklist under `docs/briefs/r3-pb-t-fixedpoint-worker.md`).

Receipt-key pin + **validate-before-`write_receipt`** (`self_host_receipt_p0` + `self_host_fixed_point`) remains the correct **DB-8 / P0** bounded surface; it does **not** substitute for P1/P2/P3 dispatch.

---

## STOP / next shape (for PB dispatch)

| Question | Answer |
|----------|--------|
| Is a **bounded T-FixedPoint P1 implementation slice** (promotion / strong suite / CI flip) **open now**? | **STOP** — still gated on **brief dispatch preconditions** + staged `compiler.dag` / CI policy above; do not treat Evaluator substrate merges as automatic P1 clearance. |
| Is **P2** open for T-FixedPoint-owned implementation? | **STOP** — still gated on **lens producer retirement** + **SG-0 = 0** per brief; T-FixedPoint **consumes** that signal, it does not close it. |
| Smallest **honest** next work PB can steer (outside this lane owning closure)? | **Upstream:** R2-Evaluator program + joint grounding ledger items; **XL:** lens producer retirement + PB-1 shim; **Lane 1e / emit:** determinism debt called out in DB-8 / `self_host_ratchet` comments. |

Cross-reference for PB Manager: keep using `docs/briefs/r3-pb-t-fixedpoint-worker.md` §Dispatch preconditions as the single dispatch authority; re-run this style of audit when the **closure ledger** or **SG-0 census** materially changes.
