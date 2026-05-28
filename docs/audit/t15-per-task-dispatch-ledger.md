# T-15 per-task dispatch ledger

**Owner session**: `vivid-boar-454` (T-15 close owner)
**Parent PM**: `nimble-koi-111`
**Authority**: `src/v4/TASKS.md` (task scope + prereqs); this doc is the **dispatch ledger only** (worker / PR / blocker / status). Structural task definitions stay in TASKS.md.
**Template**: `docs/audit/r3-plan-audit-2026-05-10.md` §R1 (per-gate dispatch ledger)
**Opened**: 2026-05-28
**Update cadence**: on operator dispatch, merge, or blocker change; weekly lane summary to parent PM.

**Role boundary (operator 2026-05-28)**: this session **does not** create dashboard work-items for implementation tasks. Operator dispatches workers; the close owner **records** worker session / PR / blocker / status here only.

---

## T-15 close predicates (verdict column)

| # | Predicate | Ledger column |
|---|---|---|
| 1 | Whole plan minus T-15 complete | `plan-complete` |
| 2 | v4 compiles `src/v4/compiler/*.dag` end-to-end | maps to spine + T-37 bridge |
| 3 | v4 emits Rust → binary | maps to T-10/T-11 + bootstrap |
| 4 | Bit-identical fixed-point (stage1 == stage2) | maps to T-20-fill, T-15 |
| 5 | TestClaim suite passes | maps to T-38, T-22, corpus |

Close audit (when imminent): run each predicate on HEAD → `GREEN` / `PARTIAL` / `PAPER-ONLY`.

---

## Lane A — critical path (sets T-15 timing)

Chain: **T-37 → T-38 → T-20-fill → T-24 → T-16 → T-36 → T-15**

| Task | Schedulable | Worker session | PR | Blocker | Status | Predicates |
|---|---|---|---|---|---|---|
| **T-37** | **yes** (no prereqs) | — | — | none | **UNASSIGNED** | 2, 3 |
| **T-38** | after T-22 runnable | — | — | T-37 trustworthy bridge; T-22 eval surface | **UNASSIGNED** | 5 |
| **T-20-fill** | **yes** (scaffold on main) | — | — | none for fill start | **UNASSIGNED** | 3, 4 |
| **T-24** | partial (T-21/T-23/T-10 done) | — | — | **T-20-fill** | **BLOCKED** | 3, 5 |
| **T-16** | no | — | — | T-24 + T-11 + extdeps demo deps | **BLOCKED** | 1, 3 |
| **T-36** | no | — | — | T-16 (per PM lane ordering) | **BLOCKED** | 1, 4 |
| **T-15** | no | — | — | Lane A + whole plan | **GATE** | all |

**Notes**

- T-37: pure v2 Rust; fix shape in `docs/audit/v2-dag-artifact-zip-fold-hang-2026-05-21.md`. Dissolves `scripts/v4-bootstrap-resolve-posture-gate.sh` SIGTERM pass-through.
- T-38: 40 manual claims compile only; `scripts/check-v4-host-eval-receipt.py` is string-match bridge. T-34 done (#3770). T-22 ~1121 lines authored.
- T-24: T-21 merged (#3747), T-23 merged (#3702), T-10 merged; **T-20-fill is the gating dep** for close (not just prep).
- TASKS.md T-37/T-38 formal entries: **pending** merge of PR #3783 (planning); ledger tracks dispatch regardless.

---

## Lane B — parallel fill (must all complete; non-blocking among peers)

| Task | Schedulable | Worker session | PR | Blocker | Status | Predicates |
|---|---|---|---|---|---|---|
| **T-19** | **yes** (T-1/2/3 done) | — | — | oversight: never dispatched (operator queue) | **UNASSIGNED** | 1, 5 |
| **T-22** | **yes** (T-9, T-34 done) | — | — | scaffold gates (B1 cache-hash non-blocking for T-38) | **UNASSIGNED** | 5 |
| **T-12** | needs T-9 | — | — | T-9 refine for real fold | **UNASSIGNED** | 1 |
| **T-13** | needs T-9 | — | — | same | **UNASSIGNED** | 1 |
| **T-17** | verify closeout | — | #3768 merged | confirm TASKS.md `[DONE]` | **VERIFY** | 1 |
| **T-18** | no | — | — | T-12, T-13 | **BLOCKED** | 1 |
| **T-31** | yes (mop-up) | — | — | none | **SCHEDULED** | 1 |
| **T-32** | design-first | — | — | operator Phase-1 ratification | **SCHEDULED** | 1 |
| **T-33** | yes | — | — | T-4 feeder | **SCHEDULED** | 1 |
| **T-33-Q10** | after T-33 | — | — | T-33 | **SCHEDULED** | 1 |
| **T-4.15** | deferred | — | — | omni-stack activation gate | **SCHEDULED** | 1 |
| **T-4.16** | yes | — | — | — | **ACTIVE** | 1 |
| **T-4.17** | partial | — | open PRs | T-2 #3748, per-language | **ACTIVE** | 1 |
| **T-4.18** | partial | — | — | probe fills | **ACTIVE** | 1 |
| **T-4.19** | partial | — | — | english boundary | **ACTIVE** | 1 |
| **T-QN-1** | partial | — | #3764 area | follow-on landed, not closed | **OPEN** | 1 |
| **T-35** | no | — | — | T-QN-1 | **BLOCKED** | 1 |

---

## Compiler spine + side branch (predicate 1 bulk)

| Task | Status | Worker / PR | Notes |
|---|---|---|---|
| T-1…T-3 | **LANDED** | — | substrate foundation |
| T-6…T-8 | **LANDED** (CP-1b tail) | — | merged stage files |
| T-9 | **SCAFFOLD / IMPL** | — | needs T-4 fact bundles |
| T-10 | **LANDED** (scaffold) | merged 00_compile, 05_emit | TargetModel naming drift tracked in TASKS |
| T-11 | **NOT STARTED** | — | blocks T-16 |
| T-4 | **HELD** | T-4 mgr | keystone cluster |
| T-14 | **CORPUS FILLED** | — | manual claims; T-19 generates |
| T-21 | **LANDED** | #3747 | |
| T-23 | **LANDED** | #3702 | |
| T-25-core, T-26, T-29, T-30, T-34 | **LANDED/DONE** | various | |
| T-25-tail, T-28, T-28-B | **MODELED/SCHEDULED** | — | |

---

## Predicate snapshot (2026-05-28, pre-close-audit)

| # | Verdict | Evidence / blocker |
|---|---|---|
| 1 | **PARTIAL** | Lane A/B rows above; T-11, T-4, T-16, T-36, dissolution sweep open |
| 2 | **PARTIAL** | Stages merged; full `src/v4` `--target dag` OOM → T-37 |
| 3 | **PARTIAL** | Emit scaffold landed; bootstrap binary path needs T-20-fill |
| 4 | **PAPER-ONLY** | `workflow/bootstrap.dag` scaffold; `self_host.dag` returns `SelfHostRunnerNotRealized` |
| 5 | **PAPER-ONLY** | Claims compile; no CI eval → T-38 |

---

## Change log (tracking only — no worker dispatch by close owner)

| When (UTC) | Action | Notes |
|---|---|---|
| 2026-05-28 | Opened ledger | `vivid-boar-454`; Lane A/B initial status |
| 2026-05-28 | TASKS.md bootstrap convergence section | execution-graph + convergence prose (PR #3783 aligned) |
| 2026-05-28 | Role correction | operator owns implementation dispatch; ledger UNASSIGNED rows |
| — | PR #3783 merge | full T-37/T-38 task-definition bodies land in TASKS.md |

---

## References

- `src/v4/TASKS.md` — task definitions
- PR [#3783](https://github.com/gunb-ai/gunbc/pull/3783) — surfaces T-37/T-38 + bootstrap convergence
- `docs/audit/v2-dag-artifact-zip-fold-hang-2026-05-21.md` — T-37 root cause
- `docs/v4-close-interrogation.md` — ship interrogation / questionnaire (close audit input)
