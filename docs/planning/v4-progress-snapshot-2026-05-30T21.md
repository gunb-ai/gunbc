# v4 progress snapshot — 2026-05-30T21:00Z

> **Author:** `merry-badger-222` (Self-host/Release / v4-done predicate burn-down owner)  
> **Audience:** operator + PM (internal; stripped from public snapshot via `docs/planning/` glob)  
> **Tree HEAD:** `117dc50ff` (`main` 2026-05-30T22:00Z) — includes #3957 SG-5, #4030 burn-down maintenance, #4025 SUPPORTED.md, #3991 release snapshot  
> **Companion artifacts:** [`v4-done-predicate-burn-down-2026-05-30.md`](v4-done-predicate-burn-down-2026-05-30.md) (maturation tracker); [`v4-mw-d8-wave1-exit-ledger-2026-05-30.md`](v4-mw-d8-wave1-exit-ledger-2026-05-30.md) (Close/Receipt authority for MW-D8 rows); [`docs/audit/v4-rustc-error-catalog-2026-05-29.md`](../audit/v4-rustc-error-catalog-2026-05-29.md) (rustc baseline)

**Release framing (D-REL-1 flavor iv):** v4 ships public **alpha/WIP-labeled** at v0.1.0 regardless of rows below. This doc is **internal truth** + v0.1.1 maturation cadence — not a tag gate.

---

## §1. Headline state

| Metric | Value | Notes |
| ------ | ----- | ----- |
| **v4-done predicates PROVEN** | **0 / 6** | Unchanged. No §10.0 executable close receipt on `main`. |
| **Burn-down colors** | 5 YELLOW + 1 GRAY | P1–P5 active lanes; P6 gated on P4/P3. |
| **Strongest mover today** | **P5** (TestClaim / leaf-model) | R1 (#3972), R2/R3 (#4000), W2.6 python R1 (#4022). |
| **Second mover** | **P3** (emit → binary) | SG-2 substrate (#3962), **SG-5 landed (#3957)**; SG-1 still draft (#3956/#3964). |

### Recent advances (predicate touch)

| PR | Landed (UTC) | Predicates | Effect |
| -- | ------------ | ---------- | ------ |
| [#3972](https://github.com/gunb-ai/gunbc/pull/3972) | 18:24 | **P5** | First R1 leaf-model verdict (MW-D8 C1) |
| [#4000](https://github.com/gunb-ai/gunbc/pull/4000) | 19:55 | **P5** | R2a/R2b/R3-external widening (MW-D8 C5) |
| [#4014](https://github.com/gunb-ai/gunbc/pull/4014) | 21:01 | **P2**, **P5** | SG-7 ci.dag offset projection dissolved (MW-D8 C2 **impl** — ledger re-adjudication pending) |
| [#4022](https://github.com/gunb-ai/gunbc/pull/4022) | 21:05 | **P3**, **P5** | W2.6 python.dag R1 cross-target verification |
| [#4018](https://github.com/gunb-ai/gunbc/pull/4018) | 21:35 | **P4**, **P5** | W2.5 Phase 4 fixtures (branch_dispatch, loop_linear_bound) |
| [#3957](https://github.com/gunb-ai/gunbc/pull/3957) | ~22:00 | **P3** | SG-5 TargetCollectionRealization substrate landed |
| [#4025](https://github.com/gunb-ai/gunbc/pull/4025) | ~21:40 | *(public)* | `docs/SUPPORTED.md` — Rust+Python+Go supported; v4 alpha/WIP |

### Named blockers (honest)

| Blocker | Predicates | Status |
| ------- | ---------- | ------ |
| Resolve-posture bridge (`v4-bootstrap-resolve-posture-gate.sh` + `ci.yml`) | **P2** | **OPEN** — masks honest compile-of-record close |
| SG-1 emit class (~2978 E0423) | **P3** | **OPEN** — #3956/#3964 drafts; dominant Pareto |
| T-15 runner + B1 pins | **P4**, **P6** | **OPEN** — structural harness only |
| T-38 structural corpus bridge (`v4-testclaim-corpus-gate.sh`) | **P5** | **OPEN** — modeled runner not yet CI authority |
| Whole-plan open tasks (`T-35`, `T-38`, `T-31`, `T-32`, …) | **P1** | **OPEN** — meta-gate |

**No predicate flipped GREEN today.**

---

## §2. Error count (rustc full-tree v4 emit)

### Baseline (authoritative probe)

| Metric | 2026-05-29 probe | Source |
| ------ | ----------------: | ------ |
| `rustc` `error[E####]` lines | **7951** | [`v4-rustc-error-catalog-2026-05-29.md`](../audit/v4-rustc-error-catalog-2026-05-29.md) |
| Files with errors | **262** / 294 emitted | same |
| Top code | **E0423** (2978, 37.5%) | SG-1 Symbol/Atom value emission |
| Second | **E0308** (1191) | SG-3 type mismatch |
| Third | **E0107** + **E0282** (1219 + 743) | SG-2 generic arity |

**Fresh probe:** **not re-run this session** (build/probe interrupted; ~10min cargo path). Count below is **projected**, not measured on `117dc50ff`.

### Since today's landings — projected delta

| Landing | SG class | Projected rustc impact on `main` |
| ------- | -------- | -------------------------------- |
| #3962 SG-2 substrate | SG-2 | **0 lines closed yet** — substrate only; emitter consumer not wired |
| #4014 SG-7 ci.dag dissolve | *(complexity)* | **Indirect** — clears 24 v2 compile diagnostics blocking T-38 scaffold honesty; **does not** reduce 7951 rustc lines by itself |
| #3957 SG-5 collection realization | SG-5 | **0 lines closed yet** — carrier landed; translate consumer pending |
| #3956/#3964 SG-1 (when lands) | SG-1 | **~−2978 E0423** projected — dominant single PR lever |
| Python+Go emit fix workers | emit | **TBD** — smart-stag re-dispatch in flight; no measured delta |

**Honest headline:** full-tree rustc error population is **still ~7951 ± unmeasured** on current `main`. Substrate landings today **prepare** closure; they do **not** yet move the M1 probe meter until emitter lanes consume them.

### v2 compile gate (separate from 7951)

24 `complexity: same-argument recursion` diagnostics in `ci.dag` / pipeline_rejections — **#4014 addresses SG-7 slice**; T-22 complexity lane owns remainder. Blocks honest T-38-PR1 zero-diagnostic receipt.

---

## §3. Wave 1 MW-D8 exit

**Close/Receipt authority:** [`v4-mw-d8-wave1-exit-ledger-2026-05-30.md`](v4-mw-d8-wave1-exit-ledger-2026-05-30.md) (last formal update [#4017](https://github.com/gunb-ai/gunbc/pull/4017), 20:25Z).

| # | Condition | Ledger row (formal) | Operator-facing status |
| - | --------- | ------------------- | ---------------------- |
| C1 | R1 leaf-model verdict | **PROVEN** (#3972) | Closed |
| C2 | SG-7 ci.dag recursion dissolved | **GAP** (ledger stale — cites #4014 open) | **Impl landed** [#4014](https://github.com/gunb-ai/gunbc/pull/4014) 21:01Z — **awaiting Close/Receipt re-adjudication** |
| C3 | Upsert\<T\> landed or blocked | **PROVEN** (OR-arm worksheet) | Closed |
| C4 | ci_selection_receipt_shadow | **GAP** | **OPEN** — W1.5 on `smart-stag-871` queue |
| C5 | R2/R3 claim authoring | **PROVEN** (mixed-arm, #4000) | Closed |

**Formal ledger headline:** **3/5 PROVEN** (per #4017 — C2 row not yet updated post-#4014).  
**Operator-facing headline:** **4/5 MET** once Close/Receipt flips C2 after #4014 adjudication. **Remaining:** C4 (W1.5 receipt shadow).

> PM note: treat #4017 as authority for **row text**; this snapshot flags the **#4014 → C2 lag** explicitly so operator is not misled by stale GAP on C2.

---

## §4. Wave 2 — landed vs in flight

### Landed today (high signal)

| Bucket | PRs | Wave item |
| ------ | --- | --------- |
| M6 enforcement | #3998, #3999, #4001 | Substrate PR review gate + L1.4 lens |
| Phase 1.5 CI substrate | #3989 | `CiUpsertStep<T>` shape |
| SG-7 dissolution | #4014 | W1.1 / MW-D8 C2 impl |
| W2.6 cross-target | #4022 | python.dag R1 verification |
| W2.5 Phase 4 fixtures | #4018, #4028, #4034–#4039 | branch_dispatch, loop_linear_bound, field_patch_monoid specs + fixtures |
| Predicate / ledger docs | #4021, #4030, #4017 | burn-down + maintenance + MW-D8 C5 update |
| Release / public tier | #4023, #4025, #4031, #3991 | ship-disposition, SUPPORTED.md, README disclaimer |
| Target Realization | **#3957** | SG-5 TargetCollectionRealization (**just landed**) |

### In flight

| Item | Named PR / owner | Predicate touch |
| ---- | ---------------- | --------------- |
| SG-1 TargetAtomRealization | #3956 (held), #3964 (re-dispatch) | **P3** — ~2978 E0423 |
| W2.3 CiUpsertStep migration | proud-pike worksheet (~4h ETA) | **P2**, **P5** |
| W1.5 ci_selection_receipt_shadow | smart-stag-871 queue | MW-D8 C4 |
| Python+Go emit fixes | smart-stag re-dispatch | **P3** |
| Release prep | #3991 (merged), silent-bee SUPPORTED follow-ons | public tier |

---

## §5. Release implication (flavor iv)

- v0.1.0 tags with **alpha/WIP v4 in-tree** — no predicate closure required.
- [`docs/SUPPORTED.md`](../SUPPORTED.md) (#4025): Rust + Python + Go **supported**; v4 surfaces **alpha/WIP — not on support contract**.
- [`docs/release/v0.1.0-v4-ship-disposition.md`](../release/v0.1.0-v4-ship-disposition.md) (#4023): per-surface tier labels.
- This snapshot + burn-down feed **v0.1.1 narrative** when predicates move.

---

## §6. Anti-shelfware (revisit-by)

Per PR #3949 two-axis vocabulary. Deadlines from burn-down tracker; unchanged unless predicate moves.

| Predicate | Revisit-by | If unchanged |
| --------- | ---------- | ------------ |
| P1 | 2026-06-06 | Close/Receipt mechanical census vs plan graph |
| P2 | 2026-06-02 | Bridge deletion plan or explicit blocker worksheet |
| P3 | 2026-06-02 | SG-2 → emit consumer receipt or named block |
| P4 | 2026-06-03 | T-15 B1 pin + runner realization slice |
| P5 | 2026-06-01 | R1 stable + structural bridge deletion **schedule** |
| P6 | 2026-06-06 | Re-evaluate only if P4 moves; else GRAY |
| MW-D8 C2 ledger row | 2026-05-31 12:00Z | Close/Receipt must re-adjudicate post-#4014 |
| MW-D8 C4 | 2026-06-02 12:00Z | Receipt shadow generator or named block |

---

## §7. What this doc is NOT

- Not a fresh rustc probe receipt — baseline is 2026-05-29 until M1 lane re-runs `scripts/v4-m1-rust-emit-probe.sh`.
- Not a TASKS.md amendment and not predicate narrowing.
- Not a substitute for Close/Receipt MW-D8 row adjudication — flags stale C2 row only.

## §8. Cross-links

- Maturation tracker: [`v4-done-predicate-burn-down-2026-05-30.md`](v4-done-predicate-burn-down-2026-05-30.md)
- MW-D8 ledger: [`v4-mw-d8-wave1-exit-ledger-2026-05-30.md`](v4-mw-d8-wave1-exit-ledger-2026-05-30.md)
- Rustc catalog: [`docs/audit/v4-rustc-error-catalog-2026-05-29.md`](../audit/v4-rustc-error-catalog-2026-05-29.md)
- Wave posture: [`v4-merge-wave-and-next-waves-2026-05-30.md`](v4-merge-wave-and-next-waves-2026-05-30.md)
