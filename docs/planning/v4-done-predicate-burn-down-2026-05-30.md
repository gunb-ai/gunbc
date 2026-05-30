# v4-done — six-predicate burn-down (operator view)

> **Status:** PLANNING — operator burn-down companion to [`v4-done-predicate-tracker-2026-05-30.md`](v4-done-predicate-tracker-2026-05-30.md).  
> **Authority:** `src/v4/TASKS.md:805-817`; PR [#3938](https://github.com/gunb-ai/gunbc/pull/3938) §8.D4 (all six collectively).  
> **Session:** `nimble-crane-490` (Self-host/Release Manager) · **Parent:** `nimble-dove-733`  
> **Tree HEAD:** `887f0f2ed` (`main` 2026-05-30T19:40Z) — post–merge-wave + afternoon landings (#3972, #3981, #3989, #3962, …).

## Operator summary (read this first)

| Metric | Count | Notes |
| ------ | ----- | ----- |
| **GREEN** (predicate **PROVEN** — §10.0 executable receipt) | **0 / 6** | No predicate has a close receipt on `main` yet. |
| **YELLOW** (in flight — named owner + active PR/work) | **5 / 6** | P1–P5 have resolving lanes + today’s substrate/CI advances; receipts incomplete. |
| **RED** (no owner / blocked with no named unblock) | **0 / 6** | P1 is meta-gate (all lanes); not “unowned,” but not mechanically closed. |
| **GRAY** (gated upstream — progress blocked on another predicate/lane) | **1 / 6** | **P6** gated on P4 fixed-point + reproduction path + SG-0 census trend. |

**One-liner:** **We are 0/6 v4-done predicates proven; 5 are YELLOW (active lanes); P6 is GRAY (upstream on P4/P3).** Wave 1 landings today advance **substrate and partial gates** — especially **P5 (TestClaim)** and **P3 (emit)** — but **MW-D8** forbids treating merge volume as predicate closure.

**Gated on (cross-cutting):**

- **Resolve-posture bridge** (INVARIANTS A3/P5 — *not* predicate P5): blocks honest **P2** close until deleted (`.github/workflows/ci.yml:293-300`).
- **Wave 1 exit** (MW-D8): R1 verdict + SG-7 + Upsert landed-or-blocked + receipt shadow + R2/R3 status — necessary for several YELLOW rows, not sufficient for any GREEN.

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
| **Today’s PRs** | *None single-handedly close P1.* Merge-wave volume (#3938–#3990) adds tasks done (e.g. T-37) but not whole-plan close. |
| **Re-check** | **2026-06-06** — if no Close/Receipt census delta, escalate drift-proof gate to PM |

### P2 — v4 compiles `src/v4/compiler/*.dag` end-to-end

| Field | Value |
| ----- | ----- |
| **Burn-down** | **YELLOW** |
| **Owner** | **Compiler Spine** (`smart-stag-871`) |
| **Blocking receipt** | v4 compiler-of-record full pipeline; **no** resolve-posture bridge masking failure |
| **Evidence today** | T-37 landed (#3791); bridge **OPEN** (`ci.yml:293-300`, `v4-bootstrap-resolve-posture-gate.sh`) |
| **Today’s PRs** | **#3981** Upsert\<T\> substrate — **Wave 1 / CI modeling**, not P2 close. **#3989** `CiUpsertStep<T>` — same. **#3987** CI job split — ops only. Advances **future** ci.dag authority (W1.4/W1.5), not v4-of-record compile receipt. |
| **Re-check** | **2026-06-02** — bridge + SG-7 dissolution (W1.1) status |

### P3 — v4 emits Rust that compiles to a binary

| Field | Value |
| ----- | ----- |
| **Burn-down** | **YELLOW** |
| **Owner** | **Compiler Spine** + **Target Realization** (`keen-heron-687`) |
| **Blocking receipt** | v4-emitted Rust `cargo`-clean for release binary; bootstrap `compiled:` under v4 emit |
| **Evidence today** | SG-1 class still open (~E0423); emit scaffolds widespread |
| **Today’s PRs** | **#3962** SG-2 `TargetTypeExpressionProjection` — **substrate for emit/type projection** (P3 enabler, not P3 close). **#3970** / **#3971** LeafModelClaim shape/spec — P5/R1 path, indirect P3. **#3996** v2-emit fix — unblocks CI red, not v4 emit authority. |
| **Re-check** | **2026-06-02** — SG-2 consumer dispatch + emit-error program (#3934 class) |

### P4 — Bit-identical self-output (stage1 == stage2)

| Field | Value |
| ----- | ----- |
| **Burn-down** | **YELLOW** |
| **Owner** | **Self-host/Release** (this lane) + **Compiler Spine** |
| **Blocking receipt** | B1 `content_hash` pins; `self_host.dag` runner realized; `claim_t15` + `t_15_self_host_fixed_point` **executable** fixpt |
| **Evidence today** | T-15 structural harness PASS; runner still scaffold; digest placeholders |
| **Today’s PRs** | **#3958** W2 host harness (rung 4 / nat_semiring) — **ladder/TestClaim path**, not T-15 bit-identical receipt. **#3960** RoundTripClaim W1 — ingest eval substrate, not stage1==stage2. **#3990** rung-4 acceptance spec — planning only. |
| **Re-check** | **2026-06-03** — any T-15-affecting merge must run `t_15_self_host_fixed_point` |

### P5 — TestClaim suite passes

| Field | Value |
| ----- | ----- |
| **Burn-down** | **YELLOW** (strongest movement today) |
| **Owner** | **Runtime/TestClaim** (`quick-lark` / spawn) + **Compiler Spine** |
| **Blocking receipt** | Modeled T-22 eval + structured `TestClaimRun` verdicts in CI; delete `scripts/v4-testclaim-corpus-gate.sh` |
| **Evidence today** | Structural corpus bridge still live (`ci.yml:315-319`); **#3972** lands first R1 leaf-model verdict path |
| **Today’s PRs** | **#3972** — **primary P5 advance** (R1 fixture + rustc runner + `Verdict` capture). **#3961** verdict surface contract. **#3958** W2 rung-4 host harness. **#3960** RoundTripClaim W1. Does **not** yet delete structural bridge or satisfy full T-38 corpus bar. |
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

## Today’s landing map (PM spot-check)

| PR | Merged (UTC) | Primary predicate touch | Burn-down effect |
| -- | ------------ | ----------------------- | ---------------- |
| [#3972](https://github.com/gunb-ai/gunbc/pull/3972) | 2026-05-30 ~18:24 | **P5** | YELLOW → stronger P5 (first R1 verdict path); not GREEN |
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
| Release (v0.1.0) | #4004, snappy-bee subtree | **Out of v4-done scope** (v2 public slice) |

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
- Close ledger: `docs/audit/v4-close-ledger-2026-05-30.md` (346 probes; 0/346 `PROVEN` on last spot-check)

## What this doc is NOT

- Not a TASKS.md amendment and not a predicate narrowing.  
- Not a substitute for §10.0 `PROVEN` receipts — **GREEN** only when executable proof exists.  
- Not v0.1.0 release authority — see Assignment 2 / `snappy-bee-513` lane.
