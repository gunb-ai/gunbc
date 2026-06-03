# v2 Stage0 Migration Worksheet

> **Status:** CLASS 1 DESIGN ONLY — no stage0 replacement, no `src/v2/` deletion, no hand-Rust
> cementing in this note.
> **Work item:** `node://adhoc-485fa011-b45` — `quick-moth-47`.
> **Authority anchor:** PR #1446 (T-V2-Retirement G-1 readiness receipt + STOP+PING, 2026-05-01) —
> established the G-1 gate chain, STOP rules for Population B test consumers, and the next-unblock
> order before any honest v2-oracle migration could begin. S-1 later landed (#1711); G-1 mechanical
> closure is green on current `main` (see §2). This worksheet records the **forward v2 stage0
> migration shape** that PR #1446's receipt deferred: how the load-bearing bootstrap seed evolves
> without conflating T-V2-Retirement, Branch E self-host, substrate-fill, or perf lanes.
> **Gate:** implementation workers may proceed only on the lane named in their dispatch; this note
> does not authorize cross-lane shortcuts.

## Why this is a worksheet (not a receipt)

v2 stage0 migration spans **four independent programs** that share the same tree but have different
acceptance shapes. Conflating them produces the classic failure modes: deleting the bootstrap seed
before self-host fixed point (G-2 ahead of E.1), forking emit authority for CI perf (#4171
violation), or hand-editing generated stage0 to keep a census ratchet green (INVARIANTS P2/P5).
This note names the lane boundaries, live authorities, and gate chain so each worker knows which
receipt it owns.

Live authorities this worksheet consumes (do not duplicate):

- `BOOTSTRAP.md` — regen/verify procedure, bootstrap-breaking vs regen-safe classification, seed-bump
  guardrails.
- `src/v2/stage0/src/bin/regen_stage0.rs` — generated-output registry + hand-maintained partition.
- `src/v4/workflow/bootstrap.dag` — four-stage plan `seed → stage0 → stage1 → stage2`; `compiled_by`
  chain (`seed ← v2` used once, `self0 ← stage0`, `self1 ← stage1`; v2 never re-enters).
- `docs/planning/v4-branch-e-e1-stage0-self-host-worksheet-2026-06-03.md` — E.1 bit-identical
  re-emit acceptance shape (PromotionWitness demotes stage0).
- `docs/planning/v4-incremental-bootstrap-ci-perf-rr-l-worksheet-2026-06-02.md` — RR-L perf/cache
  laws for v2 stage0 paths.
- `docs/planning/v4-ci-rust-dag-shared-closure-worksheet-2026-06-01.md` — #4171 single-authority
  emit + parity receipt for rust+dag orchestration.

## §10.0-adapted worksheet

```text
Migration class:        V2-STAGE0-MULTI-LANE (bootstrap seed maintenance + self-host handoff +
                        T-V2-Retirement sequencing)
Representative failure:  Treating T-V2-Retirement G-2 (`src/v2/` deletion) as equivalent to Branch E
                        E.1 (self-host fixed point), or executing either before the other lane's
                        prerequisites land — e.g. deleting `src/v2/stage0` while
                        `bootstrap.dag` still names `seed ← v2`, or declaring self-host achieved on
                        placeholder Hash pins while v2 stage0 remains compiler-of-record.
Immediate local patch:   Hand-edit generated `src/v2/stage0/src/*.rs` to satisfy a census ratchet;
                         fork emit/parse/infer logic in stage0 `main.rs` for CI speed; delete
                         `src/v2/` because G-1 test-oracle consumers are gone while v4 bootstrap
                         still compiles through the v2 seed; or skip `regen_stage0 --verify` after
                         a `.dag` authority change.
Why forbidden:           INVARIANTS P2 (single authority) + P5 (progress is dissolution). v2 stage0
                        is load-bearing bootstrap seed territory (SELF_HOSTING.md §1, bootstrap.dag
                        header). The census ratchet is downstream of substrate migration, not a path
                        to it. G-2 deletion and E.1 PromotionWitness are distinct receipts with
                        incompatible timing unless explicitly sequenced below.
DFS path:
  v2 stage0 authority (CONSUME — do not fork):
    - src/v2/*.dag — parse/tokenize/infer/emit/compile pipeline source
    - src/v2/stage0/src/v2_compiler_compile.rs — compile_to_resolved, emit_resolved_for_target
    - src/v2/stage0/src/bin/regen_stage0.rs — regen/verify + generated/hand-maintained registries
    - src/v2/stage0/src/{cli_run,rest_transport_facts,v2_compiler_dag_collect,v2_interpreter}.rs —
      hand-maintained stage0 companions (excluded from fixed-point diff)
  v4 bootstrap orchestration (CONSUME):
    - src/v4/workflow/bootstrap.dag — seed-used-once plan, hash pins (🟡 placeholder until T-15 B1)
    - src/v4/compiler/self_host.dag — FixedPointEqual, BootstrapRoundtripEqual, PromotionWitness
  T-V2-Retirement gates (CONSUME — sequencing only):
    - src/v3/compiler/tests/integration/v2_oracle_no_remaining_test_consumers_test.rs — G-1 mechanical
      ratchet (gate #41 / `v2_oracle_no_remaining_test_consumers`)
    - Cargo.toml [workspace].members — `src/v2/stage0`, `src/v2/tests` remain until G-2
  adjacent closure (CONSUME):
    - docs/planning/v4-branch-e-e1-stage0-self-host-worksheet-2026-06-03.md — E.1 acceptance
    - docs/planning/v4-incremental-bootstrap-ci-perf-rr-l-worksheet-2026-06-02.md — RR-L L.1–L.4
    - docs/planning/v4-ci-rust-dag-shared-closure-worksheet-2026-06-01.md — #4171 emit parity
Deepest unsound boundary:
  v2 stage0 is simultaneously (a) the committed bootstrap seed for v4's four-stage plan and (b) the
  target of T-V2-Retirement G-2 deletion. These are not the same migration: G-2 removes the v2 tree
  from the workspace; E.1 demotes v2 to a one-time seed by proving v4 .dag pipeline self-host.
  Executing G-2 before E.1 (or before an alternate seed is modeled and proven) removes the only
  compiler that can bootstrap v4. Executing E.1 without live digests + runtime execution (Rung 1,
  T-15 B1) produces a green PromotionWitness over placeholder data — explicitly forbidden in the
  E.1 worksheet.
Systemic fix:
  Partition v2 stage0 work into named lanes with independent receipts; enforce sequencing:
    Lane S (substrate-fill / regen): `.dag` authority change → `regen_stage0` → `make stage0-freshness-check`.
    Lane P (perf, RR-L): observationally-equivalent rewrites inside existing authorities only.
    Lane O (orchestration, #4171): CLI/CI sequences targets; emit stays in v2_compiler_compile.
    Lane E (self-host, Branch E): E.1 PromotionWitness on computed digests + runtime execution.
    Lane R (retirement, T-V2): G-1 oracle removal (done) → G-2 `src/v2/` deletion ONLY after Lane E
      proves v4 stage0 is compiler-of-record OR an explicit alternate seed is ratified in bootstrap.dag.
Non-goals:
  - No `src/v2/` or workspace-member deletion from this worksheet (Lane R / G-2).
  - No self-host runner realization or stage0 replacement (Lane E / W4.7).
  - No new parser/infer/emit substrate or parallel Rust compiler path.
  - No hand-Rust edits to generated stage0 outside seed-bump workflow (BOOTSTRAP.md).
  - No claiming G-2 green because G-1 oracle consumers are gone — bootstrap seed dependency remains.
Falsification probe:
  After any v2 stage0 migration PR lands, these MUST hold or the PR is rejected:
    a. `make stage0-freshness-check` (regen_stage0 --verify) passes unless the PR is an documented
       seed bump with BOOTSTRAP.md justification.
    b. No new `emit_resolved_for_target` / `compile_sources` fork outside v2_compiler_compile.rs.
    c. G-1 ratchet `v2_oracle_no_remaining_test_consumers` still passes (no new v2 crate deps
       outside `src/v2/`).
    d. bootstrap.dag `seed ← v2` row remains until PromotionWitness Holds with computed digests.
    e. Perf changes carry RR-L R1–R9 equivalence receipts, not timing-only claims.
Metric allowed only as secondary:
  Hand-maintained stage0 file count / generated-line census. Secondary to regen verify, emit parity,
  and PromotionWitness runtime receipts; census pressure is downstream, never the migration path.
```

## §1 Live State (2026-06-03, `main` HEAD)

| Item | Status | Verification |
|------|--------|--------------|
| v2 stage0 workspace member | LIVE | `Cargo.toml` `[workspace].members` includes `src/v2/stage0`, `src/v2/tests` |
| `regen_stage0` registries | LIVE | `GENERATED_STAGE0_FILES` (63 entries) + `HAND_MAINTAINED_STAGE0_FILES` (4 entries) in `regen_stage0.rs` |
| Stage0 freshness CI gate | LIVE | `make stage0-freshness-check` → `cargo run -p v2-compiler --bin regen_stage0 -- --verify` |
| G-1 oracle consumers | GREEN | `v2_oracle_no_remaining_test_consumers_test.rs` — no substantive `v2_compiler` deps outside `src/v2/` |
| v3→v2 Cargo edges | REMOVED | `src/v3/compiler/Cargo.toml` has no `v2-compiler` path deps (PR #1446 §3.3 target met) |
| Substrate-fill #3477 | MERGED | Set/BTreeSet migration; regen verify GREEN after drift from #3471 |
| bootstrap.dag seed row | LIVE | `seed ← v2` used once; hash pins 🟡 placeholder (T-15 B1 pending) |
| E.1 acceptance worksheet | MERGED | #4368 — bit-identical re-emit shape; Rung 1 (#4353) hard blocker |
| RR-L perf laws | RATIFIED | #4282/#4281/#4324 consumed; L.1–L.4 dispatch authorized |
| #4171 shared emit closure | MERGED | `emit_resolved_for_target` single authority |

Remaining `v2-compiler` string references outside `src/v2/` are **gate/smoke tests** that assert
absence or model CI facts — not G-1 oracle consumers. Do not "clean them up" as migration work;
they are receipts.

## §2 PR #1446 Anchor — What Changed Since the STOP

PR #1446 (#1446, 2026-05-01) recorded that S-1 (PM-authored T-V2-Retirement worker brief) was **NOT
MET**, blocking honest G-1 implementation. The receipt's next-unblock order:

```text
S-1 lands → G-1 dispositions authorized → G-1 implementation (§3.1, §3.2, §3.3) → G-1 green
  → G-2 prereqs (S-1..S-4 + G-1) → G-2 implementation
```

**Current disposition (2026-06-03):**

| Step | PR #1446 expectation | Current state |
|------|------------------------|---------------|
| S-1 worker brief | NOT MET at receipt time | Landed #1711 (2026-05-04) |
| G-1 §3.1/§3.2 test migration | STOP until S-1 | Complete — Population B consumers removed/replaced |
| G-1 §3.3 Cargo edges | Mechanical after §3.1+§3.2 | Complete — no v3→v2 path deps |
| G-1 mechanical ratchet | — | `v2_oracle_no_remaining_test_consumers` gate test landed |
| G-2 `src/v2/` deletion | After S-1..S-4 + G-1 | **NOT STARTED** — blocked on Lane E + prereqs |

PR #1446's constraints remain binding for forward work:

- No `kernel_algebra_profile` migration without Substrate-side authority (cross-program).
- No `verification.dag` convergence decision from PB territory.
- G-2 is **not** authorized by G-1 green alone — bootstrap seed dependency persists.

## §3 Migration Lane Map

| Lane | Scope | Authority | Acceptance receipt | Blocks |
|------|-------|-----------|-------------------|--------|
| **S** Substrate-fill / regen | `.dag` changes that alter stage0 output | `src/v2/*.dag` + `regen_stage0` | `make stage0-freshness-check` GREEN; bootstrap-breaking changes follow BOOTSTRAP.md bridge/seed-bump | Nothing downstream if verify fails |
| **P** Perf (RR-L) | Cache/tokenizer/CI budget inside v2 authorities | RR-L §2–§4 | R1–R9 falsification table; regen verify | — |
| **O** Orchestration (#4171) | CLI/CI multi-target sequencing | `v2_compiler_compile::emit_resolved_for_target` | Byte/diagnostic parity vs standalone `--target dag` | — |
| **E** Self-host (Branch E) | v4 .dag pipeline becomes compiler-of-record | `bootstrap.dag` + `self_host.dag` | E.1 PromotionWitness on computed digests + runtime execution | G-2 |
| **R** Retirement (T-V2 G-2) | Remove `src/v2/` workspace members | T-V2 gate #41 + G-2 prereqs | `src/v2/` deleted; workspace builds without v2 members | Requires Lane E OR ratified alternate seed |

**Lane interaction law:** Lanes S/P/O may proceed in parallel. Lane E consumes S green + Rung 1 +
T-15 B1. Lane R is strictly downstream of Lane E (or an explicit bootstrap.dag amendment naming a
non-v2 seed with its own honesty witness).

## §4 v2 Stage0 Internal Shape (regen partition)

```text
src/v2/*.dag + dsl/
  --(regen_stage0)--> src/v2/stage0/src/*.rs (generated, 63 files)
                     + hand-maintained (4 files, excluded from fixed-point diff):
                       cli_run.rs, rest_transport_facts.rs,
                       v2_compiler_dag_collect.rs, v2_interpreter.rs
  --(cargo build)--> v2-compiler binary (gunbc)
       |
       +-- v4 bootstrap seed (bootstrap.dag: seed ← v2, used once)
       +-- v4 CI M1 rust+dag emit probe (#4171 shared closure)
       +-- make stage0-freshness-check / CI verify gate
```

**Regen-safe vs bootstrap-breaking** (from `BOOTSTRAP.md`): every `.dag` PR must declare its
classification. Regen-safe: ordinary `regen_stage0` + commit. Bootstrap-breaking: two-step bridge
or documented seed bump — never silent hand-edits to generated files.

## §5 Gate Chain (dispatch readiness)

| Gate | Provides | Status (2026-06-03) | Owner |
|------|----------|---------------------|-------|
| G-1 oracle removal | No v2 crate deps outside `src/v2/` | ✅ GREEN | T-V2-Retirement |
| regen verify | Committed stage0 matches fresh self-compile | ✅ GREEN (post-#3477) | Lane S |
| #4171 emit parity | Single `emit_resolved_for_target` authority | ✅ MERGED | Lane O |
| RR-L ratification | Perf/cache laws for v2 paths | ✅ RATIFIED | Lane P |
| Rung 1 Execution-Runnable | `gunbc test` executes TestClaims at runtime | 🔴 OPEN (#4353) | Branch A |
| T-15 B1 `content_hash` | Merkle digests replace placeholder Hash pins | 🔴 pending | T-15 |
| E.1 PromotionWitness | Bit-identical re-emit fixed point | 🔴 blocked on above | Branch E |
| G-2 prereqs S-2..S-4 | Legacy emit chain, verification.dag routing | 🟡 partial | PM/Substrate |
| G-2 `src/v2/` deletion | Workspace member removal | 🔴 NOT STARTED | T-V2-Retirement |

**Hard rule:** Lane R (G-2) cannot dispatch until E.1 gate row is ✅ OR bootstrap.dag is amended with
a ratified non-v2 seed + honesty witness. G-1 green does not satisfy this row.

## §6 Landing Order

```text
1. This worksheet merged — workers cite lane (S/P/O/E/R) in PR title/body.
2. Lane S: substrate-fill PRs — each carries regen verify + BOOTSTRAP.md classification.
3. Lane P: RR-L L.1/L.2 perf PRs — semantic equivalence receipts (R1–R9).
4. Lane O: orchestration PRs — #4171 parity only; no emit fork.
5. Rung 1 (#4353) + T-15 B1 content_hash land.
6. Lane E: E.1 body fill — PromotionWitness on computed digests + runtime execution (W4.7).
7. G-2 prereqs S-2..S-4 close (parallel where independent).
8. Lane R: G-2 `src/v2/` deletion — ONLY after step 6 (or ratified alternate seed).
```

## §7 Boundaries

- **Branch E (`royal-badger-408`):** owns E.1/E.4 self-host acceptance; this worksheet does not
  duplicate the E.1 bit-identical re-emit shape (#4368).
- **CI Manager (`silent-crane-669`):** owns RR-L L.4 CI budget; Lane P workers consume RR-L laws.
- **T-V2-Retirement:** owns G-1/G-2 gates; G-2 blocked on bootstrap seed replacement per §5.
- **Substrate Manager:** owns cross-program authority migrations (e.g. `kernel_algebra_profile`) —
  PB cannot retire parity tests before Substrate-side authority lands (PR #1446 §3.2 STOP rule).
- **Load-bearing pipeline stages** (`emit`/`lower`/`infer`/`parse`): escalate before editing under
  a brief that pre-dates the relevant L2.5 model PR.

## §8 Forbidden Patterns

| Pattern | Why forbidden |
|---------|---------------|
| Hand-edit generated `src/v2/stage0/src/*.rs` outside seed-bump workflow | Breaks regen verify single authority |
| Delete `src/v2/` because G-1 is green | Removes bootstrap seed before E.1 replacement |
| Fork emit in stage0 `main.rs` for CI speed | #4171 / INVARIANTS P2 violation |
| Placeholder-digest PromotionWitness | bootstrap.dag header: structural wiring ≠ convergence proof |
| Perf cache keyed on mutable intern id | RR-L R1 / #4282 lesson |
| Skip `stage0-freshness-check` after `.dag` authority change | CI gate + BOOTSTRAP.md requirement |

## §9 Class 1 Acceptance Checklist

- [x] Four migration lanes named with independent receipts (S/P/O/E/R).
- [x] PR #1446 anchor recorded; delta since STOP documented in §2.
- [x] G-1 green vs G-2 blocked distinction explicit (bootstrap seed persists).
- [x] Live authorities cited; no dependency on docs removed in #4192 public cleanup.
- [x] Gate chain enumerated with 2026-06-03 status; Rung 1 + T-15 B1 named as E.1 blockers.
- [x] Forbidden shapes named: seed-bump gaming, G-2 ahead of E.1, emit fork, placeholder fixed point.
- [x] Falsification probe fails closed on regen drift, emit fork, G-1 regression, premature G-2.
- [x] No stage0 replacement, `src/v2/` deletion, or runner realization in this PR.

## Related Artifacts

- gunb-ai/gunbc#1446 — G-1 readiness receipt + STOP+PING (authority anchor)
- gunb-ai/gunbc#1711 — S-1 T-V2-Retirement worker brief (unblocked G-1)
- gunb-ai/gunbc#3477 — v2 stage0 substrate-fill (Set/BTreeSet; regen verify GREEN)
- gunb-ai/gunbc#4171 — CI Rust+DAG shared closure (Lane O)
- gunb-ai/gunbc#4368 — Branch E.1 stage0 self-host worksheet (Lane E acceptance shape)
- gunb-ai/gunbc#4353 — Rung 1 Execution-Runnable (E.1 hard blocker)
- `BOOTSTRAP.md` — regen/seed-bump procedure
- `src/v4/workflow/bootstrap.dag` — four-stage bootstrap plan
- `src/v2/stage0/src/bin/regen_stage0.rs` — generated/hand-maintained registries
