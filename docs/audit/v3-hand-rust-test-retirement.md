# v3 Hand-Rust Test Retirement — Classification

Audit of hand-Rust tests under `EXPECTED_HAND_AUTHORED_TEST` (`src/v3/compiler/tests/integration/sg0_census_test.rs:380`).
**Operator-direct (2026-05-22):** categorize by value; **aggressive deletion** over 1:1 TestClaim replacement. **Read-only** — no files deleted in this dispatch.

## Authority boundaries (not a parallel ledger)

Per [`docs/modeling-discipline.md`](../modeling-discipline.md) (standing rule: no comment-duplicating maintained ledgers) and INVARIANTS P2/P5 single-authority discipline:

| Fact | Authoritative carrier |
|---|---|
| Which paths are hand-Rust tests | `EXPECTED_HAND_AUTHORED_TEST` in [`sg0_census_test.rs`](../src/v3/compiler/tests/integration/sg0_census_test.rs) |
| Retirement **execution** (what actually deletes) | Operator-approved dashboard work-items + execution PRs that shrink the census |
| Per-path dissolution triggers (ratified) | Inline comments in `sg0_census_test.rs` + PR review — not this doc |

This markdown file is **transient PR-review prose** (dispatch `vivid-deer-815`; **dissolve this file** once operator ratifies batch-1 retirement and execution PRs own the census shrink). Heuristic DELETE/REPLACE/KEEP **proposals** are not binding until ratified. **No checked-in per-test or per-path dispatch rows** — only aggregate counts, cluster narratives, and reason-class rollups below. Path/test worksheets: [`scripts/generate_v3_hand_rust_test_retirement_inventory.py`](../scripts/generate_v3_hand_rust_test_retirement_inventory.py) (`--check` fails closed on missing census files; `--by-file > /tmp/…` for ≤145 path-level rows).

## Census reconciliation (HEAD 2026-05-22)

| Grain | Brief cited | Measured at HEAD |
|---|---|---|
| `EXPECTED_HAND_AUTHORED_TEST` path literals | 204 entries | **145** paths |
| `#[test]` functions in those paths | (implied) | **1232** tests |

The PM figure **204 does not match** the live census (145 path literals / **1232** `#[test]` functions on `origin/main` and this worktree). Likely explanations: (1) stale planning count from an older, longer `EXPECTED_HAND_AUTHORED_TEST` list; (2) conflation with **file-path** entries (~160–204) vs per-function tests; (3) intent to name a first **DELETE batch size** rather than the full ratchet. The helper script assigns **one heuristic bucket per census path** and sums `#[test]` counts (1232 functions across 145 paths at HEAD). Operator ratification precedes any retirement PR.

**Operator DELETE-first wave (recommended):** **383** tests (31.1%) in the DELETE bucket — includes all pre-flagged v4 smoke/closeout (**78**), cementing Rust (**18**), and `m1_substrate_test.rs` (**115**) — without waiting for 1:1 TestClaim ports.

## Summary

- **DELETE:** 383 tests (31.1%)
- **REPLACE-VIA-TESTCLAIM:** 532 tests (43.2%)
- **KEEP-AS-RUST:** 317 tests (25.7%)
- **Total classified:** 1232 `#[test]` functions across **145** census paths


**Count verification:** `python3 scripts/generate_v3_hand_rust_test_retirement_inventory.py --check` (comment-stripped `#[test]` count per census path; path literals **145**).

**Bias applied:** on-the-fence DELETE vs REPLACE → DELETE; on-the-fence vs KEEP → non-KEEP.

## T-19 coordination (quiet-moth-603 / smart-boar-330)

**Landed `TestgenConcept` arms** (`src/v4/TASKS.md` T-19): `TypeConstruction`, `AlgebraLaw`, `DiagnosticExhaustiveness`, `LensApplicability`, `BidirectionalRoundtrip`, `LanguageBehaviorEquivalence`.

**Already exercised without v3 integration Rust:** `scripts/check_t19_testgen_activation.py` on generated LBE corpus; manual anchors in `src/v4/test/claim/manual/`.

**REPLACE bucket T-19 mapping (aggregate, path-level heuristic — sums to 532):**
- `TypeConstruction`: 333 tests
- `LensApplicability`: 128 tests
- `LanguageBehaviorEquivalence`: 39 tests
- `T-19-CATEGORY-MISSING: ModuleServiceParseSurface`: 8 tests (`services_carrier_shape_test.rs` + `ctrl_pr_digests_dag_smoke_test.rs`)
- `AlgebraLaw`: 9 tests
- `BidirectionalRoundtrip`: 8 tests
- `DiagnosticExhaustiveness`: 7 tests

Bucket totals (DELETE / REPLACE / KEEP) are what `--check` validates; T-19 sub-lines are illustrative routing hints only and can drift if path heuristics change.

**Categories flagged MISSING** (need design before port, not an excuse to KEEP Rust):
- `WorkspaceManifestStructuralAudit` — v2_oracle G-1 manifest walks
- `RepoFileTreeNegativeBridgeAudit` — gate #62 filesystem include_str! audit
- `ModuleServiceParseSurface` — ctrl `module … service …` until parser models it
- `Gate73_ReportPredicateCarriers` — ComplexitySummary Band-C (blocks deleting `complexity_lens_behavioral_completion.rs` until .dag predicate lands OR Rust deleted after .dag green)

---

## Pre-flagged clusters — confirm / refute

### (a) `v2_oracle_*` family — **REFUTE naive DELETE; KEEP short-term**

PM assumed "v3 matches v2 output" oracle tests. **Actual:** `v2_oracle_no_remaining_test_consumers_test.rs` is **G-1 excision** — scans `src/**` Rust + workspace `Cargo.toml` deps for `v2-compiler` spellings (gate #41 / T-V2-Retirement). **Not** behavioral differential testing.

- **10 tests**, buckets: {'KEEP-AS-RUST': 10}
- **Disposition:** KEEP until `src/v2/` deletion closes gate #41; then **DELETE entire file** (or replace with manifest model in substrate — low ROI).

### (b) `v4_*_smoke_test.rs` family — **CONFIRM DELETE**

Eight modules under `integration/v4_*_smoke_test.rs` (**71 tests**): v3 tokenize/parse smoke of v4 `.dag` while v4 self-host matures — inverted dependency. Per-test enumeration is **not** checked in; filter `--by-file` output locally. Parse receipts belong in v4 CI + `check_t19_testgen_activation.py`; none warrant KEEP.

### (c) `cementing/` (6 modules) — **CONFIRM DELETE** (Rust leg) with one carrier caveat

Band-C cementing Rust modules are **transitional** same-PR receipts for `regen.dag` COMPLETE rows; parallel `.dag` harnesses under `tests/dag/t_r3_gate_87_cementing_regen_*.dag` + `t_pb_b_1_dag_runner_test` are the migration target.

- **18 tests**, buckets: {'DELETE': 18}
- **`complexity_lens_behavioral_completion.rs`:** still cites `Gate73_ReportPredicateCarriers` for `ComplexitySummary` — **DELETE Rust** once `.dag` predicate exists OR after .dag harness passes (delete-bias: do not expand Rust).
- **Other five:** consumer/residual pins (`cost_lens_symbolic_consumer`, E-P descent, effect_enumeration, memory_peak, provenance origin) — **DELETE** when matching gate-87 `.dag` suite is green in CI.

### (d) `v4_test_bootstrap_infra_closeout_test.rs` — **CONFIRM DELETE**

- **7 tests**, buckets: {'DELETE': 7}
- Asserts parse + substring presence on v4 `testgen.dag`, `bootstrap.dag`, manual/generated claim files.
- **Structural reason:** duplicated by `scripts/check_t19_testgen_activation.py` (LBE + manifest) and v4-lane ownership; not v3 product behavior.
- **No remaining ratchet obligation** beyond v4 lane closeout gates already tracked in T-19/T-20/T-22 tasks.

---

## DELETE bucket (rollup by reason class)

Reason-class subtotals sum to **383** DELETE tests (matches summary and `--check`). Path-level membership: `--by-file` filtered to `bucket=DELETE`.

| Reason class | Tests | Notes |
|---|---:|---|
| Imperative substrate walks | 115 | `m1_substrate_test.rs` bulk |
| Milestone / m0 / m2 parity obsolete | 93 | `m0_acceptance`, `four_fixture`, `m2_feature_parity` |
| v4 inverted-dependency smoke/closeout | 78 | §(b) + §(d) clusters |
| Bridge/meta/blocker ratchets | 31 | canonical_lens, prereq_x, tc1_*, lens_behavioral_parity, … |
| Migration/freshness include_str! | 23 | `m2_lens_*_migration_test` family |
| Host drivers for .dag TestClaims | 20 | `r3_free_consequences_*`, skeleton, corpus seeds, … |
| Cementing transitional Rust | 18 | §(c); `.dag` gate-87 harness is authority |
| One-shot preflight / release-wrapper | 5 | `e_i_lane_*`, `r1_release_acceptance` |

---

## REPLACE-VIA-TESTCLAIM bucket

Port only where behavior is still load-bearing after DELETE waves. **Do not 1:1 port all 532** — operator intent is delete-first. T-19 routing aggregate is in §T-19 coordination above; path-level membership: `--by-file` with `bucket=REPLACE-VIA-TESTCLAIM`.

---

## KEEP-AS-RUST bucket (~317 tests)

Temporary until substrate/TestClaim coverage or census zero. Path-level list: `--by-file` with `bucket=KEEP-AS-RUST`.

| Heuristic class | ~Tests | Dissolution |
|---|---:|---|
| Class-5 emit boundaries (`tests/boundary/*`) | 116 | ExecuteCommand TestClaim per TESTING.md |
| TestRunner / `.dag` harness | 63 | Shrinks as `.dag` TestClaims subsume runner checks |
| SG-0 / SG-* census ratchet | 42 | Gate #84 — delete when `EXPECTED_HAND_AUTHORED_TEST` empty |
| Crate wiring (`integration.rs`, `determinism_test`) | 42 | Infrastructure — last to shrink |
| G-1 v2 excision (`v2_oracle_*`) | 10 | DELETE file when `src/v2/` removed (gate #41) |
| Gate #62 filesystem tree audit | 5 | No substrate walker yet |
| Host / PB-1 / self-host bridges | 11 | Gates #71 / PB-1 snapshot |
| Shared `integration/common/*` helpers | 28 | Retire with last importer |

---

## Local re-measurement (not checked in)

Reconcile census drift without maintaining a dispatch ledger:

```bash
python3 scripts/generate_v3_hand_rust_test_retirement_inventory.py --check
python3 scripts/generate_v3_hand_rust_test_retirement_inventory.py --summary
# optional path-level worksheet (≤145 rows; never commit):
python3 scripts/generate_v3_hand_rust_test_retirement_inventory.py --by-file > /tmp/v3-hand-rust-by-file.jsonl
```

The script applies **one heuristic bucket per census path** (not per `#[test]`). Per-test retirement belongs in operator-ratified work-items after batch approval.

---

## Recommended retirement batch order (for downstream dispatch)

1. **v4 smoke + v4 bootstrap closeout** (~DELETE cluster) — zero v3 behavioral loss.
2. **Cementing Rust modules** after gate-87 `.dag` CI green.
3. **`m1_substrate_test.rs` bulk DELETE** (115 tests) — largest ROI; retain ≤20 TestClaims.
4. **Host `.dag` claim drivers** (`r3_free_consequences_*`, corpus seeds, skeleton).
5. **m2_lens_*_migration_test** freshness duplicates.
6. **REPLACE wave** only for KEEP-adjacent load-bearing gates (anthropic wire, gate receipts still open).
7. **KEEP inventory** shrinks last (SG-0 census, TestRunner, boundary, gate #62/#71).
