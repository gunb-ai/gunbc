# v4-done — six-predicate tracker

> **Status:** PLANNING — first artifact for Self-host/Release Manager (`nimble-crane-490`).  
> **Authority:** `src/v4/TASKS.md:801-815` (definition); PR [#3938](https://github.com/gunb-ai/gunbc/pull/3938) §8.D4 + §10.0 + §11.1 (disposition vocabulary + lane map).  
> **Session:** `nimble-crane-490` · **Parent:** `nimble-dove-733` (PM May 29)  
> **Tree HEAD spot-check:** `55ad5f3d3` (2026-05-30, post-#3938 `b129ce3f2` on main) — rebased `session/nimble-crane-490-predicate-tracker`.

## Non-negotiables

- **Cannot narrow v4-done.** Per PR #3938 §8.D4: any predicate relaxation requires an explicit `src/v4/TASKS.md:801-815` amendment with named operator rationale — not a PM or manager call.
- **Closure vocabulary (§10.0):** `ship_disposition: PROVEN` only with an executable receipt (+ falsification when the probe asks). Substrate alone → `GAP` + `engineering_state: SUBSTRATE_PRESENT` (or finer states below).
- **§8 ratified:** PR #3938 merged (`b129ce3f2` on main); ladder + disposition vocabulary authoritative via `docs/planning/v4-correctness-ladder-2026-05-30.md`. Worker dispatch still follows per-class DFS worksheet approval (PR #3938 §11.4) — this tracker does not self-dispatch.

## Summary matrix

| # | Predicate (`TASKS.md:801-815`) | `ship_disposition` | `engineering_state` | Primary TASKS anchors | Blocking receipt (close) | Resolving manager lane |
| - | ------------------------------ | ------------------ | --------------------- | --------------------- | ------------------------ | ---------------------- |
| 1 | Every other scheduled task complete (whole plan minus T-15) | `GAP` | `CENSUS_NOT_RUN` (drift-proof; no mechanical census) | Plan graph `src/v4/TASKS.md:7-265`; Summary `src/v4/TASKS.md:1233-1239` | Per-task `DONE` / dissolution receipts against live plan at close time — not a frozen list | **Close/Receipt** (ledger) + **all lanes** (implementation) |
| 2 | v4 compiles `src/v4/compiler/*.dag` end-to-end | `GAP` | `PARTIAL_GATE_PRESENT` | `T-6`…`T-10`, `T-37` `src/v4/TASKS.md:474-686`, `:2246-2271` | v4 compiler-of-record executes full `src/v4/compiler/*.dag` pipeline without v2 OOM/SIGTERM mask; P5 resolve-posture bridge deleted | **Compiler Spine** |
| 3 | v4 emits Rust that compiles to a binary | `GAP` | `PARTIAL_GATE_PRESENT` | `T-10`, `T-11`, `T-20`, `T-32` `src/v4/TASKS.md:612-686`, `:1057-1078`, `:2155-2216` | Emitted Rust is `cargo`-clean for release binary; bootstrap `compiled:` under v4 emit (not v2-only structural compile) | **Compiler Spine** + **Target Realization** |
| 4 | Binary on `src/v4/compiler/*.dag` → bit-identical output (stage1==stage2) | `GAP` | `SCAFFOLD_PRESENT` | `T-15` `src/v4/TASKS.md:768-815`; `T-20`, `T-32`, `T-36` | B1 merkle `content_hash` pins replace digest placeholders; `self_host.dag` runner realized; `claim_t15` + `t_15_self_host_fixed_point` execute real fixpt (not structural-only) | **Self-host/Release** (this lane) + **Compiler Spine** |
| 5 | TestClaim suite passes | `GAP` | `PARTIAL_GATE_PRESENT` | `T-14`, `T-22`, `T-38` `src/v4/TASKS.md:734-766`, `:1130-1151`, `:2277-2309` | Modeled T-22 eval over manual corpus + structured `TestClaimRun` verdicts in CI; delete `scripts/v4-testclaim-corpus-gate.sh` | **Runtime/TestClaim** (pending spawn) + **Compiler Spine** |
| 6 | Hand-authored Rust not editable authority (reproduction-proven) | `GAP` | `PARTIAL_GATE_PRESENT` | `T-15`, `T-32`, INVARIANTS A3 / P5 | Rebuild-from-(.dag + frozen seed) reproduces pinned artifact hash; SG-0 hand-test census trends to dissolution; P5 bridges removed | **Self-host/Release** + **Close/Receipt** (census ratchet) |

**Release bar:** predicates **1–6 collectively** (PR #3938 §8.D4). Ladder rungs 7–8 map to predicates 4–5; predicate 1 is strictly broader than any single rung.

---

## Predicate 1 — Whole plan minus T-15

**Authority text:** `src/v4/TASKS.md:802-808` — drift-proof close gate; never a hardcoded task count.

| Field | Value |
| ----- | ----- |
| `ship_disposition` | `GAP` |
| `engineering_state` | `CENSUS_NOT_RUN` |
| Owner sub-tasks | **Meta-gate** — each `T-*` / `T-4.*` row in `src/v4/TASKS.md:7-265` and task bodies `:299-2310` |
| Blocking receipt | At close time: every in-scope scheduled task shows landed dissolution (`[DONE]`, scaffold dissolved, or explicit `[DROPPED]` tombstone per operator ratification) except **T-15** |
| Resolving lane | **Close/Receipt** maintains the ledger; implementation ownership is per-task (see §Lane routing below) |

**Spot-check (2026-05-30):** Plan still carries substantial open surface — examples with explicit non-DONE status in task bodies:

| Anchor | Status signal |
| ------ | ------------- |
| `T-35` `src/v4/TASKS.md:1679` | `[SCHEDULED]` |
| `T-38` `src/v4/TASKS.md:2277` | `[SCHEDULED]` |
| `T-31` `src/v4/TASKS.md:2102` | `[SCHEDULED]` |
| `T-32` `src/v4/TASKS.md:2155` | `[SCHEDULED]` |
| `T-36` `src/v4/TASKS.md:2220` | `[IN PROGRESS — PR open]` |
| `T-24` `src/v4/TASKS.md:1172` | CI/YAML authority bridge open per task body |
| `T-4.17` `src/v4/TASKS.md:2000` | Wave 2a/2b active |

**Landed (examples):** `T-19`, `T-29`, `T-33`, `T-34`, `T-37` marked `[DONE]` in task bodies; `T-25-core` / `T-26` `[SUBSTRATE LANDED]`.

**Note:** `TASKS.md:815` Close-status snapshot (`main@678bb8bbd`) predates current HEAD; treat this tracker + live `TASKS.md` as operational — refresh Close-status when operator ratifies the P1–P6 numbering fix (forwarded via PM).

---

## Predicate 2 — Corpus compiles end-to-end

**Authority text:** `src/v4/TASKS.md:809`.

| Field | Value |
| ----- | ----- |
| `ship_disposition` | `GAP` |
| `engineering_state` | `PARTIAL_GATE_PRESENT` |
| Owner sub-tasks | `T-6`, `T-7`, `T-8`, `T-9`, `T-10`, `T-37` (serializer unblock) |
| Blocking receipt | v4 pipeline compiles `src/v4/compiler/*.dag` as compiler-of-record; no P5 bridge masking compile failure |
| Resolving lane | **Compiler Spine** |

**Evidence (file:line):**

- **T-37 landed:** `src/v4/TASKS.md:2246-2271` — #3791; dissolution trigger (b) met on probe.
- **P5 bridge still live:** `scripts/v4-bootstrap-resolve-posture-gate.sh:1-12` (dissolve when (a) or (b) + 14-day soak); CI `.github/workflows/ci.yml:273` invokes it.
- **Structural compile today:** `scripts/v4-testclaim-corpus-gate.sh:3-18` compiles `src/v4` via **v2** `gunbc` (`:33-37`) — not the v4 self-host binary chain.
- **Emit scaffold gates:** widespread `🟡` on compiler stages; SG-1 emit class still open per PR #3938 §10.1.

**Interrogation:** `docs/audit/v4-close-interrogation-validation-2026-05-30.md` — 0/346 `PROVEN`; global blocker cites missing executable `TestClaimRun` verdicts (`:12-14`).

---

## Predicate 3 — Emitted Rust compiles to a binary

**Authority text:** `src/v4/TASKS.md:810`.

| Field | Value |
| ----- | ----- |
| `ship_disposition` | `GAP` |
| `engineering_state` | `PARTIAL_GATE_PRESENT` |
| Owner sub-tasks | `T-10`, `T-11`, `T-20` (`bootstrap.dag`), `T-32` (minimum seed) |
| Blocking receipt | Release-path `cargo build` of v4-emitted Rust succeeds; bootstrap viability reaches `compiled:` under v4 emit authority |
| Resolving lane | **Compiler Spine** (emit) + **Target Realization** (`TargetAtomRealization` / projection rows — SG-1 class) |

**Evidence:**

- `src/v4/workflow/bootstrap.dag:3` — `bootstrap-content-hash-pins` still `🟡`; placeholder `Hash` aliases until T-15 B1 operands land.
- `src/v4/compiler/self_host.dag:3-4` — runner `scaffold-against-contract`; `self_host_runner_not_realized` at `:67-77`, `:148`.
- Open emit-error program: PR #3938 §10.1 SG-1 (~2978 E0423 class) — **Target Realization** worker surface after DFS worksheet approval.

---

## Predicate 4 — Bit-identical self-output (stage1 == stage2)

**Authority text:** `src/v4/TASKS.md:811`; falsification probe `src/v4/TASKS.md:780-799`.

| Field | Value |
| ----- | ----- |
| `ship_disposition` | `GAP` |
| `engineering_state` | `SCAFFOLD_PRESENT` |
| Owner sub-tasks | **`T-15`** (primary), **`T-20`**, **`T-32`**, **`T-36`** (round-trip prerequisite) |
| Blocking receipt | Real B1 `content_hash` on stage1/stage2 emitted Rust; executed `Equals` over digests; bootstrap fixpt without placeholder `Node` stubs |
| Resolving lane | **Self-host/Release** (T-15 ceremony) + **Compiler Spine** (emit determinism) + **Ladder/Fixture** (rung 7 gate shape) |

**Evidence:**

| Artifact | Location | Status |
| -------- | -------- | ------ |
| Trampoline authority | `src/v4/bin/main.dag:10-21` | Landed — `include!` string + digest **placeholders** (`Conj`/`Disj` empty nodes) |
| Equals claim | `src/v4/test/claim/self_host/claim_t15_self_host_fixed_point.dag:2-4` | Wired; **execution deferred** to T-22 / host harness |
| Bootstrap fixpt shape | `src/v4/workflow/bootstrap.dag` | `FixptStage1Stage2` + `bootstrap_plan_well_formed` require stage1==stage2 (harness-verified) |
| Host harness | `src/v3/compiler/tests/integration/v4_t15_self_host_fixed_point_harness_test.rs:190-198` | **PASS** on HEAD — **structural** receipt (parse/bootstrap wiring), not executed self-host |
| CI gate | `.github/workflows/ci.yml:203` | `t_15_self_host_fixed_point` runs on affected set |
| Lane closeout | `docs/briefs/t15-bin-main-dag-lane-closeout-receipt.md` | #3897 merged — explicitly **does not** close full T-15 program |

**T-36 (feeds predicate 4):** `src/v4/TASKS.md:2220-2242` — fixture + claim landed; `RoundTripClaim` eval **`Deferred`** at `src/v4/compiler/05_eval.dag:1732-1736`.

**Interrogation §4.2:** 4/4 probes `GAP` / `SUBSTRATE_PRESENT` — `docs/audit/v4-close-interrogation-validation-2026-05-30.md:422-431`.

---

## Predicate 5 — TestClaim suite passes

**Authority text:** `src/v4/TASKS.md:812`.

| Field | Value |
| ----- | ----- |
| `ship_disposition` | `GAP` |
| `engineering_state` | `PARTIAL_GATE_PRESENT` |
| Owner sub-tasks | **`T-38`** (CI harness), **`T-22`** (eval), **`T-14`** (corpus), **`T-24`** (ci.dag wiring) |
| Blocking receipt | CI runs T-22 eval on `src/v4/test/claim/manual/*.dag`; structured `TestClaimRun` verdicts; shell bridge deleted |
| Resolving lane | **Runtime/TestClaim** (primary) + **Compiler Spine** (T-22 surface) |

**Evidence:**

- Structural bridge: `scripts/v4-testclaim-corpus-gate.sh` + `.github/workflows/ci.yml:290`.
- Modeled runner scaffold: `src/v4/test/claim/workflow/testclaim_corpus_runner.dag:2-4` — `🟡 gated — feature:t38-testclaim-corpus-eval`.
- `src/v4/workflow/ci.dag:117` — `TestClaimCorpusEvalCommand` dissolution step declared; not yet replacing shell gate.
- `src/v4/TASKS.md:2277-2301` — T-38 close conditions 1–3 still open.

---

## Predicate 6 — Hand-Rust not editable authority (reproduction)

**Authority text:** `src/v4/TASKS.md:813` (A3 reproduction, not count).

| Field | Value |
| ----- | ----- |
| `ship_disposition` | `GAP` |
| `engineering_state` | `PARTIAL_GATE_PRESENT` |
| Owner sub-tasks | **`T-15`**, **`T-32`**, SG-0 / INVARIANTS P5 dissolution |
| Blocking receipt | Frozen-seed rebuild reproduces pinned content hash; interim P5 harnesses dissolved; trampoline remains build-dir-transient only |
| Resolving lane | **Self-host/Release** + **Close/Receipt** (census / interrogation §2.1) |

**Evidence:**

- Interim harnesses enumerated in `src/v3/compiler/tests/integration/sg0_census_test.rs:839-842` (`v4_t15_self_host_fixed_point_harness_test.rs`, `v4_bin_main_dag_smoke_test.rs`) — **P5 Mechanism (b)** receipts, not reproduction proof.
- `src/v4/bin/main.dag:3` — single-line trampoline authority in `.dag`; generated `include!` target is non-authoritative build artifact per task text.
- **T-32** `src/v4/TASKS.md:2155-2216` — minimum never-hand-edited seed; design-first; blocks shrinking bootstrap seed without ratified Phase-1 definition.

---

## T-15 program tracker (sub-gates)

T-15 **program close** ≠ bin/main.dag lane structural receipt (#3897 / #3929).

| Sub-gate | TASKS anchor | State | Owner lane |
| -------- | ------------- | ----- | ---------- |
| Structural harness + CI | `src/v4/TASKS.md:799`, closeout `docs/briefs/t15-bin-main-dag-lane-closeout-receipt.md` | **Landed** on main | Self-host/Release |
| T-37 serializer / emit viability | `src/v4/TASKS.md:2246-2271` | **DONE** (#3791) | Compiler Spine |
| P5 resolve-posture bridge removal | `src/v4/TASKS.md:191-193`, `:272` | **OPEN** | Compiler Spine / T-8 closeout |
| T-38 claim eval execution | `src/v4/TASKS.md:194`, `:2277-2309` | **OPEN** | Runtime/TestClaim |
| B1 content_hash pins in bootstrap | `src/v4/workflow/bootstrap.dag:3`, `src/v4/TASKS.md:291-293` | **OPEN** | Self-host/Release + Compiler Spine |
| T-32 footprint constructs walk | `src/v4/TASKS.md:294-295` | **OPEN** (PR #3907 in flight) | Self-host/Release |
| Predicate 1 (whole plan minus T-15) | `src/v4/TASKS.md:802-808` | **OPEN** | All lanes |

---

## T-36 round-trip tracker

| Field | Value |
| ----- | ----- |
| TASKS | `src/v4/TASKS.md:2220-2242` |
| Fixture | `src/v4/test/fixture/dag_round_trip_mvp1.dag` (landed) |
| Claim | `src/v4/test/claim/round_trip/dag_ingest_round_trip.dag` |
| Eval | `RoundTripClaim` → `Deferred` (`src/v4/compiler/05_eval.dag:1732-1736`) |
| `ship_disposition` | `GAP` |
| `engineering_state` | `SUBSTRATE_PRESENT` |
| Blocking receipt | T-38 executes round-trip claim with Pass/Fail verdict; unblocks meaningful T-15 fixpt loop |
| Resolving lane | **Compiler Spine** (T-6–T-10) + **Runtime/TestClaim** (eval) |

---

## Lane routing (who resolves which predicate gap)

```text
Predicate 1 ──► Close/Receipt (ledger) + every implementation lane
Predicate 2 ──► Compiler Spine
Predicate 3 ──► Compiler Spine + Target Realization
Predicate 4 ──► Self-host/Release + Compiler Spine + Ladder/Fixture (rung 7)
Predicate 5 ──► Runtime/TestClaim + Compiler Spine
Predicate 6 ──► Self-host/Release + Close/Receipt
```

**Sibling managers (coordinate on interface touch):**

| Session | Lane | Touch predicates |
| ------- | ---- | ---------------- |
| `sharp-otter-407` | Close/Receipt | 1, 6; adjudicates this tracker |
| `keen-crab-361` | Ladder/Fixture | 4–5 (rung 7–8 receipts) |
| `proud-pike-680` | Modeling DFS | 2–3 (SG worksheets before Target Realization dispatch) |
| `smart-stag-871` | Compiler Spine | 2–4 |
| `keen-heron-687` | Target Realization | 2–3 |
| *(pending)* | Runtime/TestClaim | 5; T-36 eval |

---

## PR #3938 spot-checks (manager verification pass)

Claims checked against tree at `e332fc27b`:

| PR #3938 claim | Verdict | Evidence |
| -------------- | ------- | -------- |
| §8.D4 — six predicates are release gate | **Confirmed** | Matches `src/v4/TASKS.md:801-813` |
| §10.0 — 0/346 PROVEN on main | **Confirmed** | `docs/audit/v4-close-interrogation-validation-2026-05-30.md:10-14` |
| §10.1 SG-1 blocks emit correctness | **Confirmed** | Open PR #3934 WIP; planning §10.1 |
| T-15 harness is structural not executable fixpt | **Confirmed** | `claim_t15_self_host_fixed_point.dag:4`, closeout brief `:31-37` |
| `RoundTripClaim` deferred | **Confirmed** | `05_eval.dag:1732-1736` |
| P5 bridge still present | **Confirmed** | `ci.yml:273`, `v4-bootstrap-resolve-posture-gate.sh:1-12` |

**Issue surfaced to PM:** `TASKS.md:815` Close-status paragraph uses predicate numbering (1–5) that does not label the six bullets at `:802-813` inline — risks mis-read during lane dispatch. Recommend a follow-on docs PR mapping P1–P6 ↔ bullets after §8 ratification (Close/Receipt lane).

---

## Next actions (this manager — post-§8)

1. **#3948** — rebased onto main post-#3938; awaiting operator merge (2 dashboard APPROVE, CI green).
2. **TASKS.md:815** — numbering fix forwarded to operator via PM; land separately when assigned (Close/Receipt or operator pick).
3. **Post-merge:** refresh predicate rows from live `main`; coordinate Runtime/TestClaim spawn for predicate 5; standing `t_15_self_host_fixed_point` on T-15-affecting merges.

## What this doc is NOT

- Not a TASKS.md amendment and not a predicate narrowing.
- Not a worker brief — dispatch waits on DFS worksheet approval per PR #3938 §11.4.
- Not a substitute for `docs/audit/v4-close-interrogation-validation-2026-05-30.md` (346-probe ledger stays with Close/Receipt).
