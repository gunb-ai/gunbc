# Lane A PREFIX — Acceptance artifact (**operator-signed gate for `DISPATCH_HOLD`**)

> **Companion:** `docs/briefs/r4-lane-a-lens-prefix-t23-t12-ci.md` (`PREFIX-LENS-CI-1`). **Authority:** `docs/design-lens-framework.md` §2 (`Witness<C>`, `DimensionOk` / `DimensionFail`, `DimensionReport`).

## Purpose

Immutable **witness + runnable acceptance table** for the PREFIX lens-CI slice. **`DISPATCH_HOLD`** on the worker brief lifts when this document is **committed on the Acceptance PR branch** and **signed by the operator** (per witty-cat methodology).

**Immutability:** implementation workers **do not** edit witness blocks or expectation rows except **red→green** transitions accompanied by an **operator-signed** amendment to **this** file on the Acceptance PR.

## Issue class (one sentence)

Real **lens application** over **v4 `.dag` programs** is **fail-closed** on typed **`Witness` / `DimensionFail`** outcomes, with **CI merge gates staying green** while **negative behavior** is proven by **passing** tests that **assert** those outcomes (never by requiring a failing workflow job for a diagnostic fixture).

## Runnable acceptance table (map every row to §2)

| ID | Runnable command / surface | Expected **§2** outcome | Notes |
|----|-----------------------------|-------------------------|--------|
| AC-0 | `v2-compiler compile --source-root src/v4` | Compile succeeds; **0** `Diagnostic`s | Spine bar; unchanged repo contract. |
| AC-1 | Whole-corpus driver step (Slice C — workflow) | **`DimensionOk`** aggregate per enumerated **green** corpus contract (driver completes; **exit 0** on policy match) | Thresholds / file sets listed below when operator fills **green corpus** pin. |
| AC-2 | `cargo test …` (PREFIX harness — **T-PB-B** receipt if new `EXPECTED_HAND_AUTHORED_TEST` path) | **`DimensionFail` / `Violates`** on **red** snippet; **test passes** (asserted structured failure) | **≥1 red** witness row in §Witnesses. |
| AC-3 | Same harness family on **green** snippet | **`DimensionOk` / `Inhabits`**; **test passes** | **≥1 green** witness row in §Witnesses. |

## Witnesses (**≥1 red + ≥1 green** — operator fills snippets)

### Red witness (expected `DimensionFail` / `Violates`)

- **Fixture id:** `_TODO_OPERATOR_`
- **Snippet / path:** `_TODO_OPERATOR_`
- **Asserted outcome (§2):** `_TODO_OPERATOR_`

### Green witness (expected `DimensionOk` / `Inhabits`)

- **Fixture id:** `_TODO_OPERATOR_`
- **Snippet / path:** `_TODO_OPERATOR_`
- **Asserted outcome (§2):** `_TODO_OPERATOR_`

## Green corpus pin (for AC-1)

Operator enumerates which `.dag` paths (or glob-derived sets) the **whole-corpus job** treats as **must-pass** for merge, and any explicit **allowlist / staging** markers. Until filled: reference the brief’s **Slice C** whole-tree glob story and **O(1) applications** citation (`docs/design-lens-application-surface.md` §5).

## P5 receipt note

Any **new** `src/v3/compiler/tests/**` integration file added to support AC-2/AC-3 MUST land with **INVARIANTS.md** §P5 **Mechanism (b)** row + **`ROADMAP.md`** **`T-PB-B`** / `pb_rust_tests_outside_residual_zero` citation + **`sg0_census_test.rs`** line in the **same PR** — see worker brief **§P5 / SG-0**.
