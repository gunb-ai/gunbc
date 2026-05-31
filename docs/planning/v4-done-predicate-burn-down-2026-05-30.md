# v4-done — six-predicate burn-down (operator view)

> **Status:** MATURATION TRACKER — v0.1.1 narrative source; **not** a v0.1.0 release gate (D-REL-1 flavor iv). Companion: [`v4-done-predicate-tracker-2026-05-30.md`](v4-done-predicate-tracker-2026-05-30.md).  
> **Authority:** `src/v4/TASKS.md:805-817`; PR [#3938](https://github.com/gunb-ai/gunbc/pull/3938) §8.D4 (all six collectively).  
> **Session:** `merry-badger-222` (Self-host/Release Manager) · **Parent:** `nimble-dove-733`  
> **Tree HEAD:** `01b26df8c` (`main` 2026-05-31T02:56Z) — post #4075 maintenance; W1.5 C4 shadow [#4073](https://github.com/gunb-ai/gunbc/pull/4073) landed (Close/Receipt ledger re-adjudication pending).  
> **Wave posture:** MW-D8 **4/5 PROVEN** formal ledger; **C4 impl landed** [#4073](https://github.com/gunb-ai/gunbc/pull/4073) — operator-facing **5/5 impl complete**, awaiting Close/Receipt row flip. Wave 2 active.  
> **Public framing:** D-REL-1 flavor (iv) — `src/v4` ships **alpha/WIP-labeled**; see §Public ship disposition + [`docs/release/v0.1.0-v4-ship-disposition.md`](../release/v0.1.0-v4-ship-disposition.md) (#4023).

## Operator summary (read this first)

| Metric | Count | Notes |
| ------ | ----- | ----- |
| **GREEN** (predicate **PROVEN** — §10.0 executable receipt) | **0 / 6** | No predicate has a close receipt on `main` yet. |
| **YELLOW** (in flight — named owner + active PR/work) | **5 / 6** | P1–P5 have resolving lanes + today’s substrate/CI advances; receipts incomplete. |
| **RED** (no owner / blocked with no named unblock) | **0 / 6** | P1 is meta-gate (all lanes); not “unowned,” but not mechanically closed. |
| **GRAY** (gated upstream — progress blocked on another predicate/lane) | **1 / 6** | **P6** gated on P4 fixed-point + reproduction path + SG-0 census trend. |

**One-liner:** **We are 0/6 v4-done predicates proven; 5 are YELLOW (active lanes); P6 is GRAY (upstream on P4/P3).** Wave 1 landings today advance **substrate and partial gates** — especially **P5 (TestClaim)** and **P3 (emit)** — but **MW-D8** forbids treating merge volume as predicate closure.

**Gated on (cross-cutting):**

- **Resolve-posture bridge** (INVARIANTS A3/P5 — *not* predicate P5): blocks honest **P2** close until deleted (`scripts/v4-bootstrap-resolve-posture-gate.sh`; CI step `.github/workflows/ci.yml:378-385` — "v2 → v4 bootstrap resolve-posture gate (CI emit-wall bridge)").
- **Wave 1 exit** (MW-D8): **4/5 PROVEN** formal ledger (C1, C2, C3, C5). **C4 impl landed** [#4073](https://github.com/gunb-ai/gunbc/pull/4073) (`ci_selection_receipt_shadow` shadow mode) — Close/Receipt (`sharp-otter-407`) re-adjudication pending before row flips to **5/5**. Maturation signal only — not a v0.1.0 gate.
- **D-REL-1 / v0.1.0 (flavor iv, operator 2026-05-30T20:23Z):** `src/v4` **ships public AS-IS**, labeled **alpha / WIP** — honest state documented, not gated. Prior STRIP-`src/v4` recommendation **superseded**. This burn-down feeds `SUPPORTED.md` + `docs/release/v0.1.0-v4-ship-disposition.md` (Close/Receipt `sharp-otter-407`). Closures upgrade v0.1.1 narrative; they do **not** block v0.1.0 tag.

---

## Jun 1 forecast (YELLOW → GREEN)

**Headline:** **0/6 predicates forecast GREEN by 2026-06-01.** No §10.0 executable close receipt is on a credible Jun 1 landing path. Several YELLOW rows can **strengthen** (substrate + partial gates); none satisfy the collective v4-done bar.

| Predicate | Jun 1 color forecast | Rationale |
| --------- | -------------------- | --------- |
| **P1** | **YELLOW** (unchanged) | Meta-gate; `T-35`, `T-38`, `T-31`, `T-32`, `T-36` still open. Wave 2 CI migration does not dissolve per-task receipts. |
| **P2** | **YELLOW** (unchanged) | Resolve-posture bridge still live; v4 compiler-of-record not proven. [#4014](https://github.com/gunb-ai/gunbc/pull/4014) (W1.1 / MW-D8 C2) advances ci.dag authority, not corpus compile-of-record. |
| **P3** | **YELLOW** (stronger possible) | Wave 2 primary: [#3964](https://github.com/gunb-ai/gunbc/pull/3964) SG-1 re-dispatch + held [#3956](https://github.com/gunb-ai/gunbc/pull/3956). ~2978 E0423 class — dominant Pareto; landing by Jun 1 is possible but **emit→binary PROVEN** is not. |
| **P4** | **YELLOW** (unchanged) | T-15 runner scaffold; B1 pins open. W2.5 ladder fixtures support path only; bit-identical fixpt is Wave 3 (W3.5). |
| **P5** | **YELLOW** (strongest Jun 1 mover) | [#3972](https://github.com/gunb-ai/gunbc/pull/3972) R1 PROVEN; [#4000](https://github.com/gunb-ai/gunbc/pull/4000) R2a/R2b/R3-external landed. Jun 1 anti-shelfware asks structural-bridge **deletion schedule**, not full T-38 GREEN. `scripts/v4-testclaim-corpus-gate.sh` likely still live Jun 1. |
| **P6** | **GRAY** (unchanged) | Upstream on P4 + P3; no Jun 1 path. |

**D-REL-1 flavor (iv) — public framing:** v0.1.0 ships v4 substrate **alpha/WIP-labeled** regardless of predicate color below. Honest public state at tag time: **0/6 PROVEN**, **5 YELLOW + 1 GRAY**, ~7951 rustc errors (full-tree v4 Rust emit per diagnosis lane). Any predicate that flips **GREEN by Jun 1 morning** upgrades from alpha/WIP to **partial-PROVEN** in release notes (`snappy-bee-513`); v0.1.0 tag is **not** blocked on closures.

---

## Public ship disposition (D-REL-1 flavor iv)

**Authority:** operator via PM `nimble-dove-733` (2026-05-30T20:23Z) — *"release an alpha v4 assuming it's compilable — note the errors/wherever we are — WIP instead of arbitrarily gating it."*

| Surface | Source artifact | Role |
| ------- | --------------- | ---- |
| Per-predicate status | **This doc** (`§Jun 1 forecast`, `§Per-predicate burn-down`) | Honest 0/6 PROVEN / YELLOW / GRAY ledger |
| Per-surface PROVEN/GAP labels | [`docs/release/v0.1.0-v4-ship-disposition.md`](../release/v0.1.0-v4-ship-disposition.md) (Close/Receipt `sharp-otter-407`, #4023 merged) | `SUPPORTED.md` substrate |
| Release notes alpha/WIP prose | `snappy-bee-513` lane | User-facing GH Release body |

**Jun 1 morning flag protocol:** Self-host/Release pings Release lane with any predicate that flipped GREEN overnight — those rows move from alpha/WIP to partial-PROVEN framing in release notes. Forecast at authoring: **none** (see §Jun 1 forecast).

**Do NOT:** add wholesale `src/v4` to `publish-snapshot.sh` `STRIP_PATHS` — substrate goes public under flavor (iv).

---

## Wave 2 dispatch — named PR per predicate

Wave 2 items per [`v4-merge-wave-and-next-waves-2026-05-30.md`](v4-merge-wave-and-next-waves-2026-05-30.md) §5. **Dispatch gate:** MW-D8 Wave 1 exit (all five conditions `PROVEN`). PM owns formal Wave 2 fan-out; table maps **named in-flight / held PRs** to predicate touch.

| Wave 2 item | Owner lane | Named PR(s) | Primary predicate touch | Jun 1 touch? |
| ----------- | ---------- | ----------- | ----------------------- | ------------ |
| **W2.1** SG-1 TargetAtomRealization | Target Realization | [#3956](https://github.com/gunb-ai/gunbc/pull/3956) (held draft), [#3964](https://github.com/gunb-ai/gunbc/pull/3964) (re-dispatch) | **P3** (emit → binary) | Yes — highest-signal Wave 2 PR |
| **W2.2** SG-5 / SG-6 | Target Realization | [#3957](https://github.com/gunb-ai/gunbc/pull/3957) **MERGED** ~22:00Z; SG-6 TBD | **P3** | Landed — carrier + translate consumer; rustc meter unchanged until emit consumes |
| **W2.3** Phase 1.5 `CiUpsertStep<T>` | Modeling DFS + Compiler Spine | [#4055](https://github.com/gunb-ai/gunbc/pull/4055) Bucket A **MERGED**; [#4066](https://github.com/gunb-ai/gunbc/pull/4066) Bucket B **MERGED**; [#4067](https://github.com/gunb-ai/gunbc/pull/4067) Bucket C **MERGED**; [#4059](https://github.com/gunb-ai/gunbc/pull/4059) partition table; Bucket D in flight | **P2**, **P5** (ci.dag authority) | Partial — migration slices landing; not full W2.3 close |
| **W2.4** Phase 1b A3–A14 + `check-*` deletion | Compiler Spine | *(dispatch TBD post-W2.3)* | **P2**, **P5** | No — post-Jun 1 |
| **W2.5** Phase 4 fixture widening | Ladder/Fixture | [#4018](https://github.com/gunb-ai/gunbc/pull/4018) **MERGED** (branch_dispatch, loop_linear_bound); [#4028](https://github.com/gunb-ai/gunbc/pull/4028) field_patch_monoid; rung specs #4034–#4039 | **P4**, **P5** | Partial — ladder path, not T-15 fixpt |
| **W2.6** Cross-target leaf-model (python.dag) | Modeling DFS + TR + TestClaim | [#4022](https://github.com/gunb-ai/gunbc/pull/4022) **MERGED** 21:05Z | **P3**, **P5** | Landed — python R1 mirror of rust.dag |

**Wave 1 → Wave 2 bridge (still Wave 1, predicates-adjacent):**

| Item | Named PR | MW-D8 | Predicate touch |
| ---- | -------- | ----- | --------------- |
| W1.1 SG-7 impl | [#4014](https://github.com/gunb-ai/gunbc/pull/4014) **MERGED** 21:01Z; C2 **PROVEN** [#4050](https://github.com/gunb-ai/gunbc/pull/4050) | C2 closed | **P2**, **P5** |
| W1.7 R2a/R2b/R3 | [#4000](https://github.com/gunb-ai/gunbc/pull/4000) MERGED | C5 | **P5** |

---

## Color legend (burn-down)

| Color | Meaning | Maps to tracker `ship_disposition` |
| ----- | ------- | ----------------------------------- |
| **GREEN** | Close receipt landed on `main` | `PROVEN` |
| **YELLOW** | Named owner + in-flight PR/task; partial gates | `GAP` + `PARTIAL_*` / `SCAFFOLD_*` |
| **RED** | No owner or blocked with no named unblock | `GAP` + blocked |
| **GRAY** | Upstream predicate/lane must move first | `GAP` + gated |

---

## Per-predicate burn-down

### P1 — Whole plan minus T-15

| Field | Value |
| ----- | ----- |
| **Burn-down** | **YELLOW** |
| **Owner** | **Close/Receipt** (`sharp-otter-407`) + all implementation lanes |
| **Blocking receipt** | Drift-proof census: every scheduled task `DONE` / dissolved / ratified `DROPPED` except T-15 |
| **Evidence today** | Plan still open (`T-35`, `T-38`, `T-31`, `T-32`, `T-36` IN PROGRESS, …) — see tracker §Predicate 1 |
| **Today’s PRs** | *None single-handedly close P1.* Merge-wave + Wave 1 landings add dissolved tasks (e.g. T-37) but not whole-plan close. W2.3/W2.4 CI migration touches plan surface only. |
| **Re-check** | **2026-06-06** — if no Close/Receipt census delta, escalate drift-proof gate to PM |

### P2 — v4 compiles `src/v4/compiler/*.dag` end-to-end

| Field | Value |
| ----- | ----- |
| **Burn-down** | **YELLOW** |
| **Owner** | **Compiler Spine** (`smart-stag-871`) |
| **Blocking receipt** | v4 compiler-of-record full pipeline; **no** resolve-posture bridge masking failure |
| **Evidence today** | T-37 landed (#3791); bridge **OPEN** (`scripts/v4-bootstrap-resolve-posture-gate.sh`; `.github/workflows/ci.yml:378-385` — resolve-posture gate step; line numbers drift-prone) |
| **Today’s PRs** | **#4014** SG-7 ci.dag dissolve **MERGED** 21:01Z (MW-D8 C2 impl). **#3981**, **#3989** Upsert substrate. |
| **Wave 2 PR** | W2.3 consumer PR TBD; W2.4 `check-*` deletion TBD |
| **Re-check** | **2026-06-02** — bridge + SG-7 dissolution (W1.1) status |

### P3 — v4 emits Rust that compiles to a binary

| Field | Value |
| ----- | ----- |
| **Burn-down** | **YELLOW** |
| **Owner** | **Compiler Spine** + **Target Realization** (`keen-heron-687`) |
| **Blocking receipt** | v4-emitted Rust `cargo`-clean for release binary; bootstrap `compiled:` under v4 emit |
| **Evidence today** | SG-1 class still open (~E0423); **#3957** SG-5 carrier + translate consumer landed — rustc meter unchanged until emit consumes |
| **Today’s PRs** | **#3962** SG-2 substrate (merged). **#3957** SG-5 **MERGED** ~22:00Z. **#3964** SG-1 re-dispatch OPEN — **primary Wave 2 P3 PR**. Held **#3956** canonical SG-1 draft. |
| **Wave 2 PR** | **#3964** (active), **#3956** (held); **#3957** SG-5 **landed** |
| **Re-check** | **2026-06-02** — SG-2 consumer dispatch + emit-error program (#3934 class) |

### P4 — Bit-identical self-output (stage1 == stage2)

| Field | Value |
| ----- | ----- |
| **Burn-down** | **YELLOW** |
| **Owner** | **Self-host/Release** (this lane) + **Compiler Spine** |
| **Blocking receipt** | B1 `content_hash` pins; `self_host.dag` runner realized; `claim_t15` + `t_15_self_host_fixed_point` **executable** fixpt |
| **Evidence today** | T-15 structural harness PASS; runner still scaffold; digest placeholders |
| **Today’s PRs** | **#3958** W2 host harness, **#3960** RoundTripClaim W1, **#3990** rung-4 spec (planning). No T-15 B1 pin or runner realization PR in flight. |
| **Wave 2 PR** | W2.5 fixture widening (dispatch TBD); T-15 fixpt remains **Wave 3** (W3.5) |
| **Re-check** | **2026-06-03** — any T-15-affecting merge must run `t_15_self_host_fixed_point` |

### P5 — TestClaim suite passes

| Field | Value |
| ----- | ----- |
| **Burn-down** | **YELLOW** (strongest movement today) |
| **Owner** | **Runtime/TestClaim** (`quick-lark` / spawn) + **Compiler Spine** |
| **Blocking receipt** | Modeled T-22 eval + structured `TestClaimRun` verdicts in CI; delete `scripts/v4-testclaim-corpus-gate.sh` |
| **Evidence today** | Structural corpus bridge still live; **#3972** R1 PROVEN; **#4000** R2/R3 merged; **#4022** python.dag R1 landed |
| **Today’s PRs** | **#4022** W2.6 python R1 mirror (P3/P5). **#4015** rung gate (P4/P5-adj). Does **not** delete structural bridge or close T-38. |
| **Wave 2 PR** | W2.6 **landed** (#4022); T-38 modeled runner dispatch is Runtime/TestClaim lane |
| **Re-check** | **2026-06-01** — W1.3 / Step 4 R1 stable + T-38 runner dispatch (MW-D2 expedite) |

### P6 — Hand-authored Rust not editable authority (reproduction-proven)

| Field | Value |
| ----- | ----- |
| **Burn-down** | **GRAY** |
| **Owner** | **Self-host/Release** + **Close/Receipt** (SG-0 census) |
| **Blocking receipt** | Rebuild-from-(.dag + frozen seed) reproduces pin; INVARIANTS A3/P5 interim harnesses dissolved |
| **Evidence today** | Gated on P4 fixed-point + P3 emit authority; SG-0 hand-test census not trending to zero |
| **Today’s PRs** | *No direct P6 receipt today.* #3962 / #3972 improve upstream emit/TestClaim surfaces only. |
| **Re-check** | **2026-06-06** — only after P4 moves off scaffold-only evidence |

---

## Wave 2 landing log (maintenance cadence)

Post-#4021 merges — predicate attribution for maturation tracker (not v0.1.0 gate):

| PR | Merged (UTC) | Wave item | Predicate touch | Burn-down effect |
| -- | ------------ | --------- | --------------- | ---------------- |
| [#4014](https://github.com/gunb-ai/gunbc/pull/4014) | 21:01 | W1.1 SG-7 | **P2**, **P5** | YELLOW stronger — ci.dag offset projection dissolved; C2 closed [#4050](https://github.com/gunb-ai/gunbc/pull/4050) |
| [#4022](https://github.com/gunb-ai/gunbc/pull/4022) | 21:05 | W2.6 python.dag | **P3**, **P5** | YELLOW stronger — cross-target R1 leaf-model path; not GREEN |
| [#4015](https://github.com/gunb-ai/gunbc/pull/4015) | 21:06 | W2.5 rung gate | **P4**, **P5** (ladder-adj) | YELLOW — nat_semiring rung gate parse-only alignment |
| [#4023](https://github.com/gunb-ai/gunbc/pull/4023) | 20:54 | Release coord | *(public tier)* | flavor (iv) ship-disposition supplement — feeds SUPPORTED.md |
| [#3957](https://github.com/gunb-ai/gunbc/pull/3957) | ~22:00 | W2.2 SG-5 | **P3** | YELLOW stronger — TargetCollectionRealization landed; not GREEN (emit consumer pending) |
| [#4018](https://github.com/gunb-ai/gunbc/pull/4018) | 21:35 | W2.5 fixtures | **P4**, **P5** | YELLOW — Phase 4 branch_dispatch + loop_linear_bound |
| [#4028](https://github.com/gunb-ai/gunbc/pull/4028) | ~21:50 | W2.5 fixtures | **P4**, **P5** | YELLOW — field_patch_monoid rung 0–2 spec |
| [#4025](https://github.com/gunb-ai/gunbc/pull/4025) | ~21:40 | Release | *(public tier)* | `docs/SUPPORTED.md` — Rust+Python+Go supported; v4 alpha/WIP |
| [#3991](https://github.com/gunb-ai/gunbc/pull/3991) | ~21:30 | Release | *(public tier)* | v0.1.0 consolidated state snapshot for review |
| [#4030](https://github.com/gunb-ai/gunbc/pull/4030) | ~21:45 | Maintenance | *(tracker)* | Post-#4021 landing log refresh |
| [#4044](https://github.com/gunb-ai/gunbc/pull/4044) | ~22:15 | Operator digest | *(coordination)* | [`v4-progress-snapshot-2026-05-30T21.md`](v4-progress-snapshot-2026-05-30T21.md) — internal truth rollup |
| [#4043](https://github.com/gunb-ai/gunbc/pull/4043) | ~22:20 | PM digest | *(coordination)* | [`v4-progress-snapshot-2026-05-30T22.md`](v4-progress-snapshot-2026-05-30T22.md) — PM visibility companion |
| [#4049](https://github.com/gunb-ai/gunbc/pull/4049) | 22:57 | Maintenance | *(tracker)* | Post-#4044 landing log + SG-5/W2.5 refresh |
| [#4045](https://github.com/gunb-ai/gunbc/pull/4045) | ~22:52 | W2.3 worksheet | **P2**, **P5** | YELLOW — CiUpsertStep migration worksheet; ready-for-worker-dispatch |
| [#4048](https://github.com/gunb-ai/gunbc/pull/4048) | ~22:52 | W2.5 planning | **P4**, **P5** (ladder-adj) | YELLOW — rung-4 branch/loop fixture spec tightening |
| [#4050](https://github.com/gunb-ai/gunbc/pull/4050) | ~23:05 | MW-D8 C2 receipt | **P2**, **P5** | MW-D8 **4/5** — C2 falsification receipt + ledger flip (Close/Receipt authority) |
| [#4051](https://github.com/gunb-ai/gunbc/pull/4051) | 23:10 | Maintenance | *(tracker)* | Post-#4049 W2.3 worksheet + landing log refresh |
| [#4053](https://github.com/gunb-ai/gunbc/pull/4053) | 23:21 | Maintenance | *(tracker)* | MW-D8 4/5 sync — operator burn-down aligned to #4050 ledger |
| [#4054](https://github.com/gunb-ai/gunbc/pull/4054) | ~23:25 | Maintenance | *(tracker)* | Tree HEAD `b1ba7b8f0` + #4053 landing-log backfill |
| [#4046](https://github.com/gunb-ai/gunbc/pull/4046) | ~23:30 | Wave A/D2 | **P4**, **P5** (ladder-adj) | YELLOW — rung-3/4 runtime value rows (empty W3 wedges per #3958); not T-15 fixpt |
| [#4056](https://github.com/gunb-ai/gunbc/pull/4056) | 23:55 | Maintenance | *(tracker)* | Post-#4054 landing log + #4046 attribution |
| [#4061](https://github.com/gunb-ai/gunbc/pull/4061) | ~00:30 | Maintenance | *(tracker)* | Resolve-posture bridge citation fix (`ci.yml:378-385`) |
| [#4058](https://github.com/gunb-ai/gunbc/pull/4058) | ~01:00 | Planning | *(coordination)* | [`v4-predicate-dependency-graph`](v4-predicate-dependency-graph-2026-05-30.md) — 0/6→6/6 PROVEN path |
| [#4060](https://github.com/gunb-ai/gunbc/pull/4060) | ~01:00 | Close/Receipt | **P1** (meta) | P1 every-other-task roster — drift-proof classification |
| [#4065](https://github.com/gunb-ai/gunbc/pull/4065) | ~01:15 | Close/Receipt | **P1** (meta) | P1-B per-GAP manager routing (§3.5) |
| [#4047](https://github.com/gunb-ai/gunbc/pull/4047) | ~01:20 | W3 | **P4**, **P5** | YELLOW — `run_emit_host_rust` transport + L4 verdict receipts |
| [#4055](https://github.com/gunb-ai/gunbc/pull/4055) | ~02:00 | W2.3 Bucket A | **P2**, **P5** | YELLOW — ci_pipeline CiUpsertStep rows + shadow receipt |
| [#4059](https://github.com/gunb-ai/gunbc/pull/4059) | ~01:30 | W2.3 planning | **P2**, **P5** | YELLOW — CiStepId partition table (buckets B/C/D) |
| [#4066](https://github.com/gunb-ai/gunbc/pull/4066) | ~02:30 | W2.3 Bucket B | **P2**, **P5** | YELLOW — testclaim CiUpsertStep row |
| [#4064](https://github.com/gunb-ai/gunbc/pull/4064) | ~02:15 | W3.6 | **P5** | YELLOW stronger — rung-8 nat_semiring corpus execution minimum |
| [#4040](https://github.com/gunb-ai/gunbc/pull/4040) | ~02:00 | emit | **P3** | YELLOW — v2 Python TCO + match/if (weather, nat_semiring) |
| [#4067](https://github.com/gunb-ai/gunbc/pull/4067) | ~02:20 | W2.3 Bucket C | **P2**, **P5** | YELLOW — lens_ci_registry_execution CiUpsertStep row |
| [#4063](https://github.com/gunb-ai/gunbc/pull/4063) | ~02:25 | W3.4 | **P5** | YELLOW — post-emit algebra-law preservation (additive-Monoid tranche-1) |
| [#4071](https://github.com/gunb-ai/gunbc/pull/4071) | ~02:45 | Maintenance | *(tracker)* | Post-#4061 W2.3 buckets + extended landing log |
| [#4073](https://github.com/gunb-ai/gunbc/pull/4073) | ~02:52 | W1.5 C4 | **P2**, **P5** | MW-D8 C4 impl — `ci_selection_receipt_shadow` shadow fixture (ledger flip pending) |
| [#4075](https://github.com/gunb-ai/gunbc/pull/4075) | 02:56 | Maintenance | *(tracker)* | Post-#4071 Bucket C + landing log refresh |

---

## State at tag time (Jun 1 courtesy note)

For release-notes / `snappy-bee-513` — honest snapshot at v0.1.0 tag under flavor (iv):

```text
v4-done predicates: 0/6 PROVEN | 5 YELLOW | 1 GRAY (P6)
MW-D8 Wave 1 exit:  4/5 PROVEN formal (C4 impl #4073 landed; Close/Receipt pending)
Wave 2 landed:      W2.6 #4022, W2.2 #3957 (SG-5), W2.5 #4018/#4028/#4015, W1.1 #4014, W1.5 #4073 (C4 shadow)
Wave 2 in flight:   W2.1 #3956/#3964 (SG-1), W2.3 Bucket D
Public tier:        alpha/WIP — no compile-clean guarantee on v4 surfaces
Strongest mover:    P5 (TestClaim / leaf-model path)
No predicate GREEN flips forecast at tag time.
```

---

## Today’s landing map (PM spot-check — 2026-05-30 evening baseline)

| PR | Merged (UTC) | Primary predicate touch | Burn-down effect |
| -- | ------------ | ----------------------- | ---------------- |
| [#4000](https://github.com/gunb-ai/gunbc/pull/4000) | 2026-05-30 ~19:55 | **P5** | YELLOW → stronger P5 (R2a/R2b/R3-external); MW-D8 C5; not GREEN |
| [#4012](https://github.com/gunb-ai/gunbc/pull/4012) | ~15:50 | *(coordination)* | MW-D8 ledger — no predicate flip |
| [#4014](https://github.com/gunb-ai/gunbc/pull/4014) | 21:01 | **P2/P5** | Wave 1 C2 — SG-7 impl **MERGED** |
| [#3972](https://github.com/gunb-ai/gunbc/pull/3972) | ~18:24 | **P5** | YELLOW → stronger P5 (first R1 verdict path); MW-D8 C1; not GREEN |
| [#3981](https://github.com/gunb-ai/gunbc/pull/3981) | ~15:58 | **P2/P5** (CI substrate) | YELLOW P2/P5 — Upsert primitive for W1.2; not compile-of-record |
| [#3989](https://github.com/gunb-ai/gunbc/pull/3989) | ~18:10 | **P2/P5** (ci.dag types) | YELLOW — Phase 1.5 shape; not active gating / not full migration |
| [#3958](https://github.com/gunb-ai/gunbc/pull/3958) | ~15:12 | **P5** (rung 4); ladder **P4-adjacent** | YELLOW P5; does not close P4 T-15 fixpt |
| [#3962](https://github.com/gunb-ai/gunbc/pull/3962) | ~18:19 | **P3** (emit projection) | YELLOW P3 — SG-2 substrate; SG-1 still blocks full emit |

---

## Merged-today context (30+ PRs — non-exhaustive)

High-signal buckets for predicate burn-down (full inventory: `docs/planning/v4-merge-wave-and-next-waves-2026-05-30.md` §2 + `git log main --since=2026-05-30`):

| Bucket | Examples | Predicate signal |
| ------ | -------- | ---------------- |
| Planning / ledger | #3938, #3948, #3949, #3973, #3975, #3983, #3984 | Coordination only — **no GREEN** |
| CI / modeling Wave 1 | #3981, #3989, #3947, #3987 | P2/P5 substrate — **YELLOW** |
| TestClaim / leaf model | #3972, #3961, #3958, #3960, #3970 | **P5** primary — **YELLOW** |
| Target Realization | #3962, #3971 | **P3** — **YELLOW** |
| Ladder / fixture | #3946, #3955, #3990, #4003 | Ladder rungs 0–4 — supports **P4/P5** path, not v4-done |
| Release (v0.1.0) | #4004, #4006, #4016, snappy-bee subtree | **D-REL-1 flavor (iv):** v4 ships public alpha/WIP; burn-down feeds honest state docs |

---

## Anti-shelfware deadlines

| Predicate | If unchanged by | Action |
| --------- | --------------- | ------ |
| P1 | 2026-06-06 | PM ping: Close/Receipt mechanical census vs plan graph |
| P2 | 2026-06-02 | Compiler Spine: bridge deletion plan or explicit blocker worksheet |
| P3 | 2026-06-02 | Target Realization: SG-2 → emit consumer receipt or named block |
| P4 | 2026-06-03 | Self-host/Release: T-15 B1 pin + runner realization slice |
| P5 | 2026-06-01 | Runtime/TestClaim: R1 stable + structural bridge deletion schedule |
| P6 | 2026-06-06 | Re-evaluate only if P4 evidence moves; else remain GRAY |

---

## Cross-links

- Detailed evidence rows: [`v4-done-predicate-tracker-2026-05-30.md`](v4-done-predicate-tracker-2026-05-30.md)  
- Line anchors: [`v4-done-predicate-tasks-mapping-2026-05-30.md`](v4-done-predicate-tasks-mapping-2026-05-30.md)  
- Wave posture: [`v4-merge-wave-and-next-waves-2026-05-30.md`](v4-merge-wave-and-next-waves-2026-05-30.md) §7 (MW-D1–D8)  
- Wave 1 exit ledger: [`v4-mw-d8-wave1-exit-ledger-2026-05-30.md`](v4-mw-d8-wave1-exit-ledger-2026-05-30.md) (**4/5 PROVEN**; C2 closed [#4050](https://github.com/gunb-ai/gunbc/pull/4050); C4 open)
- Predicate dependency graph: [`v4-predicate-dependency-graph-2026-05-30.md`](v4-predicate-dependency-graph-2026-05-30.md) ([#4058](https://github.com/gunb-ai/gunbc/pull/4058))  
- Close ledger: `docs/audit/v4-close-ledger-2026-05-30.md` (346 probes; 0/346 `PROVEN` on last spot-check)  
- Public ship disposition: `docs/release/v0.1.0-v4-ship-disposition.md` (Close/Receipt `sharp-otter-407` — SUPPORTED.md substrate)  
- Release maintainer snapshot: `docs/RELEASE_v0.1.0.md` ([#3991](https://github.com/gunb-ai/gunbc/pull/3991) — flavor-iv consolidated snapshot landed)
- Operator progress snapshots: [`v4-progress-snapshot-2026-05-30T21.md`](v4-progress-snapshot-2026-05-30T21.md) (#4044), [`v4-progress-snapshot-2026-05-30T22.md`](v4-progress-snapshot-2026-05-30T22.md) (#4043)

## What this doc is NOT

- Not a TASKS.md amendment and not a predicate narrowing.  
- Not a substitute for §10.0 `PROVEN` receipts — **GREEN** only when executable proof exists.  
- Not a v0.1.0 **gate** — flavor (iv) ships v4 alpha/WIP regardless; closures upgrade v0.1.1 narrative. Release packaging authority: `snappy-bee-513`; per-surface labels: `sharp-otter-407`.
