# R3 PB — `regen_lens.rs` first BinShim target: retirement-readiness checklist

**Status:** PROPOSAL (planning / readiness artifact only). Authored 2026-04-30 per R3 PB dispatch (session `quick-newt-150`).

**Purpose.** One place to answer: “What must be true before `src/v3/compiler/src/bin/regen_lens.rs` may be deleted and replaced by emitted-from-`.dag` Rust?” This doc **does not** authorize implementation, deletion, or `.dag` instance rows; it tracks **owners** and **STOP** routing so PB Manager can clear dispatch without re-deriving facts from three parent briefs.

**Non-authority.** Actual retirement waits for **PB-Runtime Item 4** (transitive emitter discipline), **declaration framework + instance authoring** under `dsl/std/runtime/bin_shims/` (`regen_lens_main` / `data regen_lens_shim` — separate worker scope), **BinShim emitter**, **§7.2 equivalence**, then a **retirement PR** that lands census deltas together with behavioral proof. The **`BinShim` substrate carrier is already on `main`** ([`src/v3/std/bin_shim.dag`](../../src/v3/std/bin_shim.dag)); carrier introduction is **not** a remaining gate — remaining gaps are emit pattern, instance row, §7.2, Item 4 convergence, and §7.3 disposition (see §Required design objects and §Dispatch-ready row). Until dispatch-ready criteria hold, treat every row below as “not green for retirement.”

**Scope boundary.** This checklist intentionally **does not** specify `.dag` syntax for `data regen_lens_shim: BinShim = { … }` or touch `dsl/std/runtime/bin_shims/**` contents — that write scope is assigned elsewhere (declaration framework + instances). This file is readiness + accountability only.

## Canonical references

- Design lock: [`docs/design-pb-runtime-interpreter.md`](../design-pb-runtime-interpreter.md) — section 4.1 (bin-shim class), 4.2 (`BinShim`, `ProcessExit`, emit pattern), 4.3 (dissolution steps), 5.1 (sub-gate 3), 6 (anti-bridge invariants), 7.2–7.3 (TestClaims).
- BinShim program brief: [`r3-pb-binshim-retirement-worker.md`](r3-pb-binshim-retirement-worker.md) — dependencies, acceptance, STOP, dispatch preconditions.
- Sub-gate 3 skeleton: [`r3-pb-t-lensproducer-sub3-regen-lens-retirement.md`](r3-pb-t-lensproducer-sub3-regen-lens-retirement.md) — `regen_lens_dot_rs_retired`, acceptance shape, STOP.
- Live hand target (unchanged until retirement PR): `src/v3/compiler/src/bin/regen_lens.rs`.
- SG-0 authorities: `src/v3/compiler/tests/integration/sg0_census_test.rs` (`EXPECTED_HAND_AUTHORED_NON_TEST`, generated partition), `src/v3/compiler/build.rs` (`REGEN_OUTPUTS`).

## Required design objects (first target: `regen_lens.rs`)

| Object | What “live” means | Retirement blocker until |
|--------|-------------------|---------------------------|
| **`BinShim` carrier** | Substrate-declared record type per design doc section 4.2; live shape: `entrypoint_name`, `description`, `entry: DeclarationRef` in [`src/v3/std/bin_shim.dag`](../../src/v3/std/bin_shim.dag). | **Landed on `main`** — verified `origin/main @ 194ddb7a8` (warm-ant-877 mechanical audit, 2026-05-06). **`BinShim` carrier introduction is not a remaining retirement blocker.** Retirement remains gated on Item 5 emit pattern, `regen_lens_main` + instance row, §7.2, Item 4 convergence, §7.3 disposition — see §Dispatch-ready row. **STOP:** carrier-shape pressure during instance authoring → Substrate Manager P1 per design doc section 5.4 — **do not** invent fields ad hoc from the PB lane. |
| **`std.process.ProcessExit`** | Entry functions return the existing substrate carrier (design doc section 4.2 cites `dsl/std/process.dag`). | **Substrate** (already authoritative on `main` at design-lock authoring; re-verify at dispatch). Owner for regression: **Substrate Manager**. |
| **Item 5 emit pattern** | `.dag` emitter translates each `BinShim` data instance to one `AUTO-GENERATED` Rust `main` template (design doc section 4.2); mirrors other Rust emit modules (anti-bridge invariant #4). | **PB Manager** — BinShim brief deliverable 2; implementation worker TBD at dispatch. **STOP:** parallel emit logic / divergence from `dsl/extdeps/languages/rust/emit.dag` shape → escalate per [`r3-pb-binshim-retirement-worker.md`](r3-pb-binshim-retirement-worker.md) STOP section. |
| **`data regen_lens_shim: BinShim = { … }`** | Pure-data witness of the current `regen_lens` pipeline under `dsl/std/runtime/bin_shims/` (paths TBD per BinShim program). | **PB-assigned worker** owning `dsl/std/runtime/bin_shims/` (declaration framework + per-shim rows per BinShim brief deliverable 1). *Do not* conflate this checklist’s author with that write scope. |
| **Equivalence gate** | Locked `TestClaim` name: `regen_lens_bin_shim_emits_behaviorally_equivalent_to_hand_rust` (design doc section 7.2) — behavioral match (exit / stdout / filesystem on fixed inputs), **not** byte-identical sources. | **Retirement / fixture worker** (post-dispatch) composes from existing `TestPredicate` variants in `src/v3/std/verification.dag`. **STOP:** no fitting predicate → **Substrate Manager** P1 (do not invent variants from PB lane). |
| **Sub-gate closure identifier** | `regen_lens_dot_rs_retired` — hand file deleted; emitted shim is the authority (sub-gate 3 brief). | **PB Manager** reports to closure ledger / R3 Release Manager when authored. |

## SG-0 census + `REGEN_OUTPUTS` deltas (same retirement PR)

When the hand-written `regen_lens.rs` retires and the emitted file becomes the shipped bin:

| Delta | Authority | Owner at retirement PR |
|-------|-----------|-------------------------|
| **`EXPECTED_HAND_AUTHORED_NON_TEST`** | `sg0_census_test.rs` list must lose `src/v3/compiler/src/bin/regen_lens.rs` (exact path per live ratchet). | **PB retirement PR author** — must net-decrease bin-shim hand count by 1 ([`r3-pb-binshim-retirement-worker.md`](r3-pb-binshim-retirement-worker.md) SG-0 bullets; **STOP** if census drifts wrong way). |
| **`REGEN_OUTPUTS` / generated partition** | Emitted `regen_lens.rs` must be treated as generated, not editable authority (`// AUTO-GENERATED from <path> — DO NOT EDIT.` per design doc section 4.2). | **PB retirement PR author** — add emit output path to `src/v3/compiler/build.rs` `REGEN_OUTPUTS` so SG-0 counts the file as generated ([BinShim brief acceptance section](r3-pb-binshim-retirement-worker.md)). |
| **Header discipline** | SG-0 census already rejects hand-authored files pretending to be `AUTO-GENERATED`. | **PB retirement PR author** — verify emitted header matches census rules. |

## Item 4 (PB-Runtime) — transitive gate only

Sub-gate 3 (`regen_lens.rs`) is **not** gated on the same mechanism as sub-gates 1–2, but Item 4 still **blocks the BinShim emitter path** by anti-bridge invariant #4 (“no parallel emit logic”; emitter shares fold-over-substrate pattern). Track separately:

| Prerequisite | Owner | Notes |
|--------------|-------|--------|
| `pb_runtime_equivalent_to_evaluator_on_corpus` green (design doc section 7.1) | **Item 4 / Evaluator convergence workers** | Out of scope for `regen_lens` retirement PR except as a **dispatch precondition** per BinShim brief Dependencies section. |
| PB-Runtime `.dag` + R2-Evaluator mirror stable | **Evaluator Manager** + **PB Manager** (PB-Runtime worker) | Per design doc sections 5.3–5.4; not re-derived here. |

## `no_new_bin_shim_hand_rust` (design doc section 7.3) — program-level closed set, not first-slice author work

| Artifact | Owner | Readiness note |
|----------|-------|----------------|
| Locked name `no_new_bin_shim_hand_rust` (design doc section 7.3) | **Future** retirement / substrate worker | **Substrate prerequisite not live at design lock:** `CensusListConstant` / `CensusSubsetCount` disposition for bin-shim subset of `expected_hand_authored_non_test` — **Substrate Manager** picks P1 shape. |
| Authoring the section 7.3 `TestClaim` | **Blocked** until disposition lands | **STOP:** do not implement section 7.3 from the first-target retirement PR alone; BinShim brief explicitly warns lane cannot fully close without it ([`r3-pb-binshim-retirement-worker.md`](r3-pb-binshim-retirement-worker.md) Dependencies item 5 + STOP section). |

First-target retirement can still land **one** shim + section 7.2 equivalence if dispatch allows, but **closed-set enforcement** remains Substrate-gated until section 7.3 is authorable.

## STOP conditions (aggregate — who to ping)

Escalation targets are **PB Manager** unless noted; PB Manager routes cross-program per briefs.

| Symptom | Route to |
|---------|----------|
| `BinShim` type cannot express `regen_lens` entry signature or pipeline composition | **Substrate Manager** P1 (carrier shape); **STOP** per sub-gate 3 + design doc section 5.4. |
| Section 7.2 behavioral equivalence cannot be expressed with existing `TestPredicate` variants | **Substrate Manager** P1. |
| Emitter work duplicates Rust emit logic instead of folding like `dsl/extdeps/languages/rust/emit.dag` | **PB Manager** / design review — **STOP** (anti-bridge #4). |
| Attempting to add new hand-Rust under `src/v3/compiler/src/bin/` or relax census without retirement | **STOP** — defect; anti-bridge invariant #3 ([`docs/design-pb-runtime-interpreter.md`](../design-pb-runtime-interpreter.md) section 6). |
| Section 7.3 disposition missing but someone tries to “finish” closed-set story ad hoc | **Substrate Manager** + R3 Release Manager surface ([BinShim brief](r3-pb-binshim-retirement-worker.md) STOP section). |

## Dispatch-ready row (PB Manager single check)

Green for **sub-gate 3 retirement worker dispatch** when **all** are true (cumulative; same as sub-gate 3 + BinShim brief, consolidated):

1. R2 close — **done** (#1275 per sub-gate 3 brief).
2. R2-Evaluator stable — **Evaluator Manager**.
3. Item 4 landed; `pb_runtime_equivalent_to_evaluator_on_corpus` evaluates true — **PB-Runtime + Evaluator owners**.
4. Substrate-owned `BinShim` carrier on `main` — **Substrate Manager**. **Mechanically verified** on `origin/main @ 194ddb7a8` (`src/v3/std/bin_shim.dag`; warm-ant-877, 2026-05-06) — this criterion is **not** the active blocker for the retirement chain.
5. `std.process.ProcessExit` verified on `main` — **Substrate Manager** (verification).
6. BinShim emit pattern landed — **PB Manager** (deliverable 2).
7. `data regen_lens_shim: BinShim` authored under `dsl/std/runtime/bin_shims/` — **PB-assigned bin_shims worker** (declaration framework + instance scope).
8. Design doc section 7.3 substrate disposition picked (for program-level `no_new_bin_shim_hand_rust` authorability) — **Substrate Manager** (BinShim brief dispatch precondition (5)); **does not block authoring this checklist** but **blocks declaring the whole BinShim program closed**.

Until row (3)–(7) are green, treat **`regen_lens.rs` deletion** as out of scope regardless of local enthusiasm.

## Cross-refs

- Manager table row: [`r2-pure-bootstrap-manager.md`](r2-pure-bootstrap-manager.md) — R3 “BinShim instances + emit pattern + retirement dispatch”.
- Sibling planning artifacts: [`r3-pb-binshim-retirement-worker.md`](r3-pb-binshim-retirement-worker.md), [`r3-pb-t-lensproducer-sub3-regen-lens-retirement.md`](r3-pb-t-lensproducer-sub3-regen-lens-retirement.md).
