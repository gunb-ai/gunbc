# Arc store-path migration — decision record (2026-08-06)

> **Status:** CLOSED for Arc-1 prelude flip; OPEN for consolidation-only land; PARKED for alias flip + C1 until v1-run-stability width>1 exit bar.
>
> **Lane:** loyal-ferret-892 / smart-badger-549 (CI Perf — actual benefits)
>
> **Operator verdict (msg_a1d9d4a4):** Do **not** land the prelude alias flip (`std::rc::Rc` → `std::sync::Arc as Rc`). Split: land wrapper-authority consolidation if no detectable regression at measurement resolution; park flip, Send+Sync frontier, and C1 in-memory store behind the un-latch trigger.

---

## 1. What was attempted

| Piece | Description | Disposition |
|-------|-------------|-------------|
| **Wrapper consolidation** | `sharing_wrap_ctor_for_target` / `sharing_type_is_wrapped_for_target` in `languages.dag`; emitter routes through `rust_shared_wrap_ctor` in `05_emit_rust.dag` | **Land if neutral** (verified §4) |
| **Prelude alias flip** | `emit_prelude` + stage0 regen: `use std::sync::Arc as Rc` | **Do not land** — carries the tax |
| **Send+Sync frontier** | `typecheck_module_result_assert_send_sync` | **Park** — requires flip |
| **C1 in-memory store** | Retire serde `Arc<Vec<u8>>` transport; share `Arc<TypecheckModuleResult>` handles | **Park** — no payoff until width>1 |
| **Serde scaffold** | Interim cross-worker byte bridge | **Keep** on main until un-latch |

---

## 2. Measured cost (Arc-1 alias flip)

**Receipt:** `docs/plans/receipts/arc-1-shareability-frontier/summary.json`

| Metric | Main (Rc) | Arc-1 (Arc as Rc) | Delta |
|--------|-----------|-------------------|-------|
| Cohort wall (median, 50 entries, width-1 serial) | 217,250 ms | 229,936 ms | **+5.84%** |
| `typecheck_compute_count` | 772 | 772 | identical |
| `Rc::new(` sites (stage0) | 8959 | 8946 | −13 (not driver) |

**Mechanism:** same work count, more wall → per-operation atomic refcount cost on ~8950 existing wrap sites. Not new over-wrapping.

**Production impact today:** width=1 inline drain → `cross_worker_store withheld` → **zero cross-worker benefit** while paying serial tax on every interpreted path.

---

## 3. Measured payoff shape (C1 prototype — parked)

**Receipt:** `docs/plans/receipts/c1-in-memory-arc-payoff/summary.json`

| Arm | `typecheck_compute_count` | Denominator |
|-----|---------------------------|-------------|
| Shared 2-thread (C1 Arc handles) | 745 | — |
| Naive 2-thread private caches | 1081 | **baseline for 31% claim** |
| Saved | 336 (31.1% of naive parallel) | — |

**Correct reading:** C1 removes the duplication tax **naive parallelism** would introduce. It does **not** beat serial width=1 production by 31% — serial already computes each node once (~745 class).

**If width>1 ever lands:** shared-N-thread total typecheck work stays near serial-1-thread; win is parallel speedup, with C1 preventing duplicate typechecking from eating it.

---

## 4. Consolidation-only neutrality (verified 2026-08-06)

**Method:** `origin/main` vs consolidation-only worktree — same `languages.dag` + `05_emit_rust.dag` authority routing, **prelude stays `std::rc::Rc`**, stage0 regen, 50-entry `p1_cohort_probe` serial cohort.

| Metric | Main | Consolidation-only | Delta |
|--------|------|-------------------|-------|
| Wall | 174,019 ms | 175,000 ms | **+0.56%** |
| `resolve_ms` (probe summary) | 79,781 ms | 80,187 ms | **+0.51%** |
| Prelude | `std::rc::Rc` | `std::rc::Rc` | unchanged |

**Measurement:** n=1 paired run per arm (no repeats).

**Conclusion:** No regression detectable at this resolution; both metrics within half a percent on a single paired run, which does not resolve an effect of that size either way. Half a percent does not change whether a single-authority wrap constructor is worth having (§3 grounds, independent of cost). Safe to land as a small PR independent of the alias flip.

**Split PR contents (land now):**
- `src/v1/languages.dag` — `wrap_ctor_template`, `wrap_type_prefix`, `sharing_wrap_ctor_for_target`, `sharing_type_is_wrapped_for_target`
- `src/v1/05_emit_rust.dag` — `rust_shared_wrap_ctor*` helpers; route existing wrap sites through authority (**prelude line stays `use std::rc::Rc`**)
- Regenerated `src/v1/stage0/**` from that emitter (no Arc prelude)

**Park on branch `session/loyal-ferret-892` (or dedicated park branch):**
- Prelude alias flip + full Arc stage0 regen
- `typecheck_module_result_assert_send_sync`
- C1 `shared_typecheck_store.rs` in-memory handles + `c1_payoff_probe`
- Hand-written `cli_run.rs` Arc-as-Rc imports (if any beyond serde scaffold)

---

## 5. Production is latched at width=1 (fact)

**Code (`cli_run.rs` ~21066–21088):** discovery samples `governor.current_target_width()` **once** at adaptive-pool entry. Governor starts at **1**. When `≤ 1`, inline drain reuses `process_shared_index` and **never enters the worker pool** — AIMD width growth never runs; `cross_worker_store` stays withheld.

**Independent receipts:**

| Source | `max_width_reached` / `target_width` | `cross_worker_store` |
|--------|----------------------------------------|----------------------|
| Fleet CI (run 29976989996, 16 GiB) | 1 | withheld |
| M2 falsifier (33.6 GiB uncapped, 842 entries) | 1 entire batch | withheld |
| P1 cohort probes (Aug 2026) | 1 | withheld |

**Latch is measured correct, not forgotten:** un-latched without shared index → 11.75 min GREEN serial vs 47 min+ unfinished or OOM (CI 29707161743 / 29714863168 / 29710324768).

**Historical `max_width_reached=9` (Track A, Jul 2026)** predates this latch — **not current production behavior**.

---

## 6. Resumption trigger (do not land flip before this)

**Owner:** v1-run-stability throughline — plural-worker outer ring, exact resident-accounting split/reservations, **width>1 fleet receipts** (OPEN per throughline BANKED block).

**Dissolve-on in code:** `cli_run.rs` comment at adaptive pool — `Rc→Arc / shared index` retires the width latch.

**When trigger fires, land in one motion (recommended):**
1. Prelude alias flip (`use std::sync::Arc as Rc`) + stage0 regen
2. `typecheck_module_result_assert_send_sync` (frontier proof)
3. C1 in-memory store (serde transport deleted)
4. Re-run: `cross_worker_shared_typecheck_cache_process_once_per_node`, purity, falsifier resolve-split at **governor-admitted** width>1

**Do not:** force width=2 measurement as payoff claim while production remains latched at width=1.

---

## 7. Trade summary (for operator escalation)

| | |
|---|---|
| **Certain cost** | +5.84% serial wall on interpreted paths (alias flip only) |
| **Repayment condition** | width>1 discovery + `cross_worker_store` armed |
| **Production today** | width=1 latched; cost paid, benefit zero |
| **Gating lane** | v1-run-stability width>1 exit bar (not CI Perf) |
| **Action** | Do not merge Arc-1 flip; land consolidation-only PR; park remainder |

---

## 8. Artifacts

| Path | Contents |
|------|----------|
| `docs/plans/receipts/arc-1-shareability-frontier/summary.json` | Arc vs Rc cohort cost |
| `docs/plans/receipts/c1-in-memory-arc-payoff/summary.json` | C1 payoff shape + width admission |
| `scripts/arc1_cohort_receipt.sh` | Cohort A/B harness |
| `session/loyal-ferret-892` branch | Full Arc-1 + C1 prototype (draft, do not merge whole) |
| `.worktrees/consolidation-only` | Consolidation-only verify worktree (local, ephemeral) |
