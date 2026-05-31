# P1 "Every Other Scheduled Task" Roster — 2026-05-30

**Manager session:** `sharp-otter-407` (Close/Receipt lane).
**Authority:** PR #3938 §11.3 lane map (Close/Receipt owns ledger coherence); PR #3973 §1 (P1 anchor `src/v4/TASKS.md:806-812`); PR #4013 burn-down (P1 = YELLOW, drift-proof — *"resolved against the plan as it stands at close time — never a hardcoded list or count that can omit in-scope work"*).
**Audit HEAD:** `origin/main` post-#4050.
**P1 statement (TASKS.md:806-812 verbatim, condensed):** *"Every other scheduled task in this plan complete — every task this plan schedules, T-15 itself excepted … never a hardcoded list or count that can omit in-scope work."*

---

## §1. Roster discipline

This roster classifies the live `src/v4/TASKS.md` task list against the P1 close gate. It is a **classification artifact**, not a scope expansion: per the safety rule (operator-stated 2026-05-31 00:30Z, relayed via PM), any out-of-scope or removal disposition is **PROPOSED**; only the operator or this lane on operator authorization ratifies a row to `ODR` ("operator-decided removal") or `NOT-PROMISED`.

**Schema** (per operator's framework):

| Column | Meaning |
| ------ | ------- |
| `task_id` | The TASKS.md section header anchor (e.g., `T-1`, `T-4.6`). |
| `in_v4_done_scope` | `yes` if the task counts toward P1 (i.e., is in the "every other" set); `no` if explicitly excluded by P1's text (T-15) or by operator decision. |
| `ship_disposition` | `PROVEN` / `GAP` / `ODR` / `NOT-PROMISED` per operator framework. |
| `blocking_receipt` | What artifact would flip the row to PROVEN — verbatim where the section names it; lane-summary otherwise. For `PROVEN` rows: the receipt that already exists (PR # or merge commit). For `ODR` / `NOT-PROMISED`: which operator decision the disposition rests on. |

**`ODR` is reserved.** Rows proposed as `ODR` are flagged explicitly as **PROPOSED — operator/Close-Receipt ratifies**. This roster does NOT remove tasks from P1 scope unilaterally.

**Drift-proof contract.** Per PR #4013 P1-burn-down framing and the TASKS.md predicate text itself, this roster is **not a frozen list**: re-running the audit against future `main` HEADs will produce a new row set as tasks are added, dissolved, or ratified out. The roster's value is the dispositions on the named HEAD, not a permanent enumeration.

---

## §2. Roster (against `src/v4/TASKS.md` on `origin/main` post-#4050)

| task_id | in_v4_done_scope | ship_disposition | blocking_receipt |
| ------- | ---------------- | ---------------- | ---------------- |
| `T-1` (std/node.dag — substrate root) | yes | `GAP` | Substrate fact-fill receipts per §T-1 wave; no DONE tag in section. |
| `T-2` (std/algebra.dag) | yes | `GAP` | §T-2 substrate-fill receipts; no DONE tag in section. |
| `T-3` (std/* supporting: cardinality / witness / diagnostic / collection / verification + scalar+numeric stack) | yes | `GAP` | §T-3 Wave A/B receipts; `verification.dag` still scaffold per §T-3 + §T-4.11 cross-link. |
| `T-4` (extdeps/languages/{rust,python,go,cpp,typescript}.dag) | yes | `GAP` | §T-4 wave-2 fact-bundle landings; #4000 widening landed but T-4 §body not marked DONE. |
| `T-4.5` (extdeps/posix.dag + file_system.dag) | yes | `GAP` (engineering_state SUBSTRATE_LANDED) | `[SUBSTRATE LANDED]` tag in header; full activation per §T-4.5 §body notes still scheduled. |
| `T-4.6` (extdeps/formats/* — 7 files: json/yaml/csv/toml/json_schema/openapi/sql) | yes | `GAP` | Per-format substrate fill per §T-4.6; `sql.dag` added 2026-05-17 per Theme-A #4 fork (a); no DONE. |
| `T-4.7` (extdeps/frameworks/react.dag) | yes | `GAP` | §T-4.7 substrate fill; no DONE tag. |
| `T-4.8` (extdeps/coordination.dag) | yes | `GAP` | §T-4.8 substrate fill; load-bearing for T-16 per §T-4 lead text. |
| `T-4.9` (extdeps/languages/verilog.dag) | yes | `GAP` | §T-4.9 substrate fill; `SL-3229-VERILOG-D3200` dissolution arrival owned here. |
| `T-4.10` (extdeps/formats/spice.dag) | yes | `GAP` | §T-4.10 substrate fill — B2-OMNI falsification probe pending. |
| `T-4.11` (test/claim/boundary/english_ingest_fail_closed.dag) | yes | `GAP` | §T-4.11 boundary claim — needs verification.dag fill + T-4.19 landing first. |
| `T-4.12` (extdeps/languages/llvm_ir.dag) | yes | `GAP` | §T-4.12 substrate fill — B2-OMNI down-stack probe. |
| `T-4.13` (extdeps/languages/machine_code.dag) | yes | `GAP` | §T-4.13 substrate fill — bottom-of-stack fail-closed probe. |
| `T-4.14` (extdeps/languages/ptx.dag) | yes | `GAP` | §T-4.14 substrate fill — SIMT data-parallel probe. |
| `T-4.15` (extdeps/protocols/{rest,graphql,grpc}.dag) | yes | `GAP` (engineering_state SCHEDULED) | `[SCHEDULED]` tag in header; T-16 cross-link does NOT depend on it per §T-4.15. |
| `T-4.16` (extdeps/formatters/*.dag — formatter config substrate) | yes | `GAP` (engineering_state ACTIVE) | `[ACTIVE]` tag in header; per-formatter substrate landings ongoing. |
| `T-4.17` (Extended language set: full bidirectional ingest, Wave 2a + 2b) | yes | `GAP` | §T-4.17 per-language emit/parse receipts. |
| `T-4.18` (Probe language ingest completion: verilog/spice/llvm_ir/machine_code/ptx) | yes | `GAP` | §T-4.18 ingest-completion receipts; #3796 closed an early slice. |
| `T-4.19` (English formal-subset language model) | yes | `GAP` | §T-4.19 formal-subset substrate; reversal ratified 2026-05-27 per §body. |
| `T-5` (work-direction meta-layer) | no | `ODR` (PROPOSED — ratified 2026-05-15) | Section header verbatim: `T-5: REMOVED — work-direction meta-layer cut (ratified 2026-05-15)`. Ratification already operator-on-record; row's `ODR` therefore not a fresh proposal — included for completeness. |
| `T-6` (compiler/01_tokenize.dag) | yes | `GAP` | §T-6 fact-fill receipts; #T-6.1 LexRule query dissolution open. |
| `T-7` (compiler/02_parse.dag) | yes | `GAP` | §T-7 grammar substrate; #T-7.1 GrammarExpr node projection dissolution open. |
| `T-8` (compiler/03_normalize.dag + 03_resolve.dag) | yes | `GAP` | §T-8 receipts; SG-1/SG-2 dissolutions cross-cut this stage. |
| `T-9` (compiler/04_infer.dag) | yes | `GAP` | §T-9 receipts. |
| `T-10` (compiler/05_emit.dag + 00_compile.dag — emission + orchestrator) | yes | `GAP` | §T-10 receipts; cross-cuts P3 burn-down YELLOW (PR #4013). |
| `T-11` (emit per-target specialization) | yes | `GAP` | §T-11 per-target receipts. |
| `T-12` (lens/complexity.dag + lens/cost.dag) | yes | `GAP` | §T-12 lens-fact-fill receipts. |
| `T-13` (lens/{parallelism,effect,ownership,idempotency,structural_resolution}.dag) | yes | `GAP` | §T-13 lens substrate receipts. |
| `T-14` (test/claim/* + test/fixture/* TestClaim corpus) | yes | `GAP` (engineering_state CORPUS_FILLED) | `[CORPUS FILLED]` tag in header; full TestClaim execution gated on T-22 + T-38 per §body. |
| `T-15` (bin/main.dag + self-host fixed-point) | **no** | `GAP` | **Explicitly excepted by P1's own text** (`:806-812`); T-15 closes P4/P6, not P1. Per PR #3973 §1 P1 row footnote. |
| `T-16` (Full-stack omni-emission demo) | yes | `GAP` | §T-16 cross-deps on T-4.7 React + T-4.8 coordination per §T-16. |
| `T-17` (lens/synthesis.dag + std/report.dag) | yes | `PROVEN` | Header: `[DONE #3768]`. Merged. |
| `T-18` (lens/coverage.dag) | yes | `GAP` | §T-18 lens substrate; coverage discipline meta-lens. |
| `T-19` (lens/testgen.dag) | yes | `PROVEN` | Header: `[DONE]`. |
| `T-20` (workflow/bootstrap.dag) | yes | `GAP` | §T-20 bootstrap-as-data receipts; cross-cuts P2/P3 burn-down YELLOW. |
| `T-21` (lens/affected_set.dag) | yes | `GAP` | §T-21 lens substrate; relates to F2 active-skip gating shape this lane just proposed. |
| `T-22` (compiler/05_eval.dag — interpreter) | yes | `GAP` | §T-22 receipts; cross-cuts P5 burn-down YELLOW; #3905 typed-input bridge advanced; runner still scheduled per T-38. |
| `T-23` (lens/application.dag) | yes | `GAP` | §T-23 lens-application receipts. |
| `T-24` (workflow/ci.dag — CI pipeline AS DATA) | yes | `GAP` | §T-24 ci-as-data receipts; SG-7 dissolution closed via #4014 + PR #4050 (helps T-24 progress but does NOT close T-24). |
| `T-25` (std/ value-predicate refinement substrate) | yes | `GAP` (engineering_state SUBSTRATE_LANDED) | `[SUBSTRATE LANDED]` tag; per PR #3949 §4 anti-shelfware policy this row carries activation debt unless wired. |
| `T-26` (std/ boundary carriers: net-address / URL / HttpMethod) | yes | `GAP` (engineering_state SUBSTRATE_LANDED) | `[SUBSTRATE LANDED]` tag; consumer wiring per §T-26. |
| `T-27` (extdeps version / semver / edition lattice) | no | `ODR` (ratified 2026-05-15 per section header) | Header: `[DROPPED]`. Operator-ratified removal already on record. |
| `T-28` (std/ module-graph substrate) | yes | `PROVEN` (via dissolution) | Header: `[DISSOLVED]`. The dissolution itself is the close receipt; T-28-B carries the modeled-extraction follow-on. |
| `T-28-B` (Extract module-root admission from `03_resolve.dag`) | yes | `GAP` (engineering_state MODELED) | `[MODELED]` tag; modeled-not-yet-executable per §T-28-B. |
| `T-29` (extdeps C++ ABI / target data-model) | yes | `PROVEN` | Header: `[DONE]`. |
| `T-30` (std/ structural fact-density / hollow-alias gate) | yes | `GAP` (engineering_state ENFORCEMENT_GATE_LANDED) | `[ENFORCEMENT GATE LANDED]` tag; full close requires the gate to fire on PRs without bridge — cross-cuts P2 close. |
| `T-31` (de-prose / de-templating backward sweep) | yes | `GAP` (engineering_state SCHEDULED) | `[SCHEDULED]` tag. |
| `T-32` (minimum never-hand-edited bootstrap seed) | yes | `GAP` (engineering_state SCHEDULED) | `[SCHEDULED]` tag; cross-cuts P6 burn-down GRAY. |
| `T-33` (std/model_core.dag — shared substrate factoring) | yes | `PROVEN` | Header: `[DONE]`. |
| `T-33-Q10` (std/model_core.dag effect / partiality carriers) | yes | `GAP` (engineering_state SCHEDULED) | `[SCHEDULED]` tag; effect/partiality wave-2b ongoing per §T-33-Q10. |
| `T-34` (std/runtime.dag + extdeps/runtimes/*.dag) | yes | `PROVEN` | Header: `[DONE]`. |
| `T-QN-1` (QualifiedName infrastructure — Change 1, prerequisite for T-35) | yes | `PROVEN` | Header: `[DONE]` (ratified 2026-05-27 per §body). `QualifiedName` substrate landed in `std/qualified_name.dag`. |
| `T-35` (virtual module-loader + ModuleBatch — filesystem-free ingest) | yes | `GAP` (engineering_state SCHEDULED) | `[SCHEDULED]` tag. |
| `T-36` (Omni ingest demo: round-trip fidelity claim) | yes | `GAP` (engineering_state IN_PROGRESS) | `[IN PROGRESS — PR open]` tag; round-trip eval now live at `05_eval.dag:1895-1907` per PR #4032; full claim receipt pending. |
| `T-37` (v2 DAG artifact serializer fix) | yes | `PROVEN` | Header: `[DONE — #3791]`. |
| `T-38` (TestClaim execution harness) | yes | `GAP` (engineering_state SCHEDULED) | `[SCHEDULED]` tag; #3902 wedge landed but full harness blocked per §T-38; cross-cuts P5 close + MW-D8 C4 unblock path. |

**Row count:** **56 tasks enumerated** (T-1, T-2, T-3, T-4, T-4.5, T-4.6, T-4.7, T-4.8, T-4.9, T-4.10, T-4.11, T-4.12, T-4.13, T-4.14, T-4.15, T-4.16, T-4.17, T-4.18, T-4.19, T-5, T-6, T-7, T-8, T-9, T-10, T-11, T-12, T-13, T-14, T-15, T-16, T-17, T-18, T-19, T-20, T-21, T-22, T-23, T-24, T-25, T-26, T-27, T-28, T-28-B, T-29, T-30, T-31, T-32, T-33, T-33-Q10, T-34, T-QN-1, T-35, T-36, T-37, T-38). Cross-checked against `grep -E '^### T-' src/v4/TASKS.md` on `origin/main` (51 section headers, with `### T-4.9 … T-4.14` umbrella expanded into 6 per-task rows because each has its own `#### T-4.X` body subsection).

---

## §3. Tallies (P1-relevant only — excludes T-15 per P1's own text)

| Disposition | Count | Tasks |
| ----------- | -----:| ----- |
| `PROVEN` | **8** | T-17, T-19, T-28 (via dissolution), T-29, T-33, T-34, T-QN-1, T-37 |
| `GAP` | **45** | the in-flight / scheduled / active corpus |
| `ODR` (in §2; out of P1 scope per `in_v4_done_scope: no`) | **0 in scope** | T-5 and T-27 carry `ship_disposition: ODR` in §2 but are `in_v4_done_scope: no`, so they fall into the **excluded** bucket below, not into in-scope ODR. |
| `NOT-PROMISED` | **0** | none surfaced on this HEAD |
| **excluded** (`in_v4_done_scope: no`) | **3** | T-15 (P1's own text excepts it); T-5 (REMOVED 2026-05-15); T-27 (DROPPED 2026-05-15) |
| **Total** | **53 in P1 scope + 3 excluded = 56** | |

**P1-headline:** **8 / 53 PROVEN** in P1 scope on this HEAD. 45 GAP rows are the in-flight corpus the live PR / sub-task waves are working through. No row stays GAP for "no owner" reasons; every GAP row's `blocking_receipt` cites a named substrate / consumer / activation receipt or a `[SCHEDULED]` / `[ACTIVE]` header.

---

## §3.5. P1-B per-GAP manager routing (forced dispatch 2026-05-31)

Per PM dispatch 2026-05-31 00:30Z (forced-dispatch item 2 of receipt-unblock wave). For each of the 45 in-scope GAP rows in §2, this section names the responsible manager (per PR #3938 §11.3 lane map) and a concrete close-receipt shape that would flip the row to `PROVEN` under PR #3949 §1's closure invariant. **No fresh `ODR` / `NOT-PROMISED` proposals** — those remain operator-ratification territory per the §1 safety rule.

**Per-row re-adjudication (PROVEN re-pass):** walked the 45 GAP rows against `origin/main` post-#4060 for any DONE-evidence missed in the original roster pass. Result: **0 GAP-to-PROVEN flips**. Every GAP row's `[SCHEDULED]` / `[ACTIVE]` / "no DONE tag" status from §2 holds on current `main`; no rows had stealth-DONE evidence.

### §3.5.1 Routing table

Columns: `task_id` × `primary manager (PR #3938 §11.3 lane)` × `secondary manager` × `close-receipt shape` (what would flip this row to `PROVEN`).

| task_id | primary | secondary | close-receipt shape (would flip to PROVEN) |
| ------- | ------- | --------- | ------------------------------------------ |
| `T-1` | Modeling DFS | Compiler Spine | std/node.dag fact-fill complete + consumers wired; receipt = §T-1 wave-close PR with no `🟡` substrate-fill markers remaining. |
| `T-2` | Modeling DFS | Compiler Spine | std/algebra.dag fact-fill + consumer activation receipt. |
| `T-3` | Modeling DFS | Runtime/TestClaim | std/* sub-substrates (`cardinality`, `witness`, `diagnostic`, `collection`, `verification`, scalar+numeric stack) all filled + verification.dag scaffold dissolved. |
| `T-4` | Modeling DFS | Target Realization | T-4 Wave-2 fact-bundles complete for {rust,python,go,cpp,typescript} per-language; per-language LanguageModel sections close. |
| `T-4.5` | Modeling DFS | Self-host/Release | posix.dag + file_system.dag activation receipts (consumers wired beyond the SUBSTRATE_LANDED marker). |
| `T-4.6` | Modeling DFS | Target Realization | All 7 format substrates (`json`, `yaml`, `csv`, `toml`, `json_schema`, `openapi`, `sql`) at full-fill; per-format parse/emit consumer wiring. |
| `T-4.7` | Modeling DFS | Target Realization | react.dag substrate fill; loadability for T-16 omni-emission demo. |
| `T-4.8` | Modeling DFS | Target Realization | coordination.dag substrate fill; T-16 demo consumes. |
| `T-4.9` | Modeling DFS | Target Realization | verilog.dag substrate fill; `SL-3229-VERILOG-D3200` dissolution-arrival receipt. |
| `T-4.10` | Modeling DFS | Target Realization | spice.dag substrate fill; B2-OMNI falsification probe receipt. |
| `T-4.11` | Runtime/TestClaim | Modeling DFS | boundary claim receipt; gates on T-3 verification.dag + T-4.19 english.dag landings. |
| `T-4.12` | Modeling DFS | Target Realization | llvm_ir.dag substrate fill; B2-OMNI down-stack probe receipt. |
| `T-4.13` | Modeling DFS | Target Realization | machine_code.dag substrate fill; bottom-of-stack fail-closed probe receipt. |
| `T-4.14` | Modeling DFS | Target Realization | ptx.dag substrate fill; SIMT data-parallel probe receipt. |
| `T-4.15` | Modeling DFS | Target Realization | rest/graphql/grpc.dag transport substrate fill (SCHEDULED → ACTIVE → DONE arc). |
| `T-4.16` | Modeling DFS | Target Realization | per-formatter config substrates complete (ACTIVE landings ongoing per §T-4.16 body). |
| `T-4.17` | Target Realization | Modeling DFS | Per-language full bidirectional ingest receipts (Wave 2a + 2b) — per-language emit + parse round-trip. |
| `T-4.18` | Target Realization | Modeling DFS | verilog/spice/llvm_ir/machine_code/ptx ingest completion; #3796 closed an early slice but full set pending. |
| `T-4.19` | Modeling DFS | Runtime/TestClaim | english.dag formal-subset substrate + boundary claim wire-up per §T-4.19 reversal. |
| `T-6` | Compiler Spine | Modeling DFS | 01_tokenize.dag fact-fill + T-6.1 LexRule token-class query dissolution receipt. |
| `T-7` | Compiler Spine | Modeling DFS | 02_parse.dag grammar substrate + T-7.1 GrammarExpr node projection dissolution. |
| `T-8` | Compiler Spine | Modeling DFS | 03_normalize.dag + 03_resolve.dag wave-close; SG-1 / SG-2 dissolutions land at this stage. |
| `T-9` | Compiler Spine | Modeling DFS | 04_infer.dag wave-close. |
| `T-10` | Compiler Spine | Target Realization | 05_emit.dag + 00_compile.dag wave-close; cross-cuts P3 burn-down. |
| `T-11` | Target Realization | Compiler Spine | Per-target emit specialization receipts (Rust + Python + Go for v0.1.0 supported tier; others for alpha/WIP). |
| `T-12` | Compiler Spine | Modeling DFS | complexity.dag + cost.dag lens-fact-fill + first executed lens receipt against fixture. |
| `T-13` | Compiler Spine | Modeling DFS | parallelism/effect/ownership/idempotency/structural_resolution.dag lens substrate fill. |
| `T-14` | Runtime/TestClaim | Modeling DFS | Full TestClaim corpus execution via T-22 + T-38 (currently CORPUS_FILLED engineering state only). |
| `T-16` | Target Realization | Self-host/Release | Full-stack omni-emission demo runnable end-to-end with T-4.7 React + T-4.8 coordination cross-deps. |
| `T-18` | Compiler Spine | Modeling DFS | coverage.dag meta-lens substrate + first coverage-disciplined claim receipt. |
| `T-20` | Self-host/Release | Compiler Spine | bootstrap.dag as-data; resolves the resolve-posture bridge per P2 close path. |
| `T-21` | Compiler Spine | Close/Receipt | affected_set.dag substrate + executable incremental-frontier receipt (also relevant to F2 active-skip per `m1_rust_emit_probe_execution` proposal). |
| `T-22` | Runtime/TestClaim | Compiler Spine | 05_eval.dag full execution path; runner harness wiring (cross-cuts P5 close); typed-input bridge landed #3905, runner still scheduled per T-38. |
| `T-23` | Compiler Spine | Modeling DFS | application.dag apply_lens surface + opt-in depth fact-fill. |
| `T-24` | Compiler Spine | Self-host/Release | ci.dag as-data wave-close; SG-7 dissolution landed via #4014 + PR #4050 but T-24 broader scope (CI step substrates) ongoing. |
| `T-25` | Modeling DFS | Close/Receipt | Refinement substrate consumer activation (PR #3949 §4 anti-shelfware deadline applies to this row's SUBSTRATE_LANDED state). |
| `T-26` | Modeling DFS | Close/Receipt | Boundary-carrier consumer activation (`net-address` / `URL` / `HttpMethod` wired in extdeps consumers). |
| `T-28-B` | Modeling DFS | Compiler Spine | Module-root admission extraction from 03_resolve.dag executable receipt (currently MODELED only). |
| `T-30` | Compiler Spine | Modeling DFS | Structural fact-density gate firing on PRs without the resolve-posture bridge — gate-landed but bridge-masked; honest activation requires bridge dissolution per P2. |
| `T-31` | Self-host/Release | Modeling DFS | de-prose / de-templating backward sweep — corpus-level census plus per-pass receipts. |
| `T-32` | Self-host/Release | Compiler Spine | Minimum never-hand-edited bootstrap seed (cross-cuts P6 burn-down GRAY; gated on T-15 fixed-point + emit authority). |
| `T-33-Q10` | Modeling DFS | Runtime/TestClaim | std/model_core.dag effect + partiality carrier substrate fill (Wave-2b ongoing). |
| `T-35` | Compiler Spine | Modeling DFS | Virtual module-loader + ModuleBatch substrate (T-QN-1 prerequisite now PROVEN, removing one blocker). |
| `T-36` | Target Realization | Runtime/TestClaim | Omni ingest demo round-trip fidelity claim — round-trip eval now live at `05_eval.dag:1895-1907` (per PR #4032); full claim receipt pending. |
| `T-38` | Runtime/TestClaim | Compiler Spine | TestClaim execution harness wave-close; #3902 wedge landed but full harness blocked per §T-38; unblock path for MW-D8 C4 + P5 close. |

### §3.5.2 Routing summary by manager (45 GAP rows distributed)

| Primary manager | GAP rows owned | Tasks |
| --------------- | -------------- | ----- |
| **Modeling DFS** | 20 | T-1, T-2, T-3, T-4, T-4.5, T-4.6, T-4.7, T-4.8, T-4.9, T-4.10, T-4.12, T-4.13, T-4.14, T-4.15, T-4.16, T-4.19, T-25, T-26, T-28-B, T-33-Q10 |
| **Compiler Spine** | 13 | T-6, T-7, T-8, T-9, T-10, T-12, T-13, T-18, T-21, T-23, T-24, T-30, T-35 |
| **Target Realization** | 5 | T-11, T-4.17, T-4.18, T-16, T-36 |
| **Runtime/TestClaim** | 4 | T-4.11, T-14, T-22, T-38 |
| **Self-host/Release** | 3 | T-20, T-31, T-32 |
| **Close/Receipt** (none primary) | 0 | (secondary only — T-21, T-25, T-26) |
| **Ladder/Fixture** (none) | 0 | |
| **TOTAL** | **45** | matches the §2 GAP count |

Modeling DFS carries the largest load (~44% of GAP rows). This matches PR #3938 §11's expectation that substrate-fill is the dominant front. Compiler Spine is the second-largest concentration (~29% — compiler stage + lens substrate). Target Realization, Runtime/TestClaim, and Self-host/Release split the remaining smaller slices.

### §3.5.3 Cross-receipt notes (informational)

- **MW-D8 C4** (`ci_selection_receipt_shadow`, MW-D8 ledger PR #4017+#4050) is owned by `smart-stag-871` post-#4014. Closing it does NOT directly close any §3.5.1 row but relates to **T-21** (affected_set.dag) and **T-24** (ci.dag as-data) — its receipt strengthens both rows' progress, though neither flips PROVEN from C4 alone.
- **F2 active-skip** (`m1_rust_emit_probe_execution`, this lane ratified 2026-05-31) is also primary-related to **T-21** (affected_set.dag) — the receipt-generator IS the affected-set lens' first PROVEN public-facing application.
- **T-15** explicitly excluded by P1 text but, per PR #3973 mapping, its closure path (P4 + P6) interacts with **T-20**, **T-31**, and **T-32**; T-15's progress accelerates those rows but does NOT close them in P1 terms.

---

## §4. Out-of-scope dispositions PROPOSED (operator ratification)

This roster does NOT propose any new `ODR` rows beyond the two operator-already-ratified entries (T-5 REMOVED 2026-05-15; T-27 DROPPED 2026-05-15). Per the safety rule (classify-don't-expand), proposing fresh removals is outside this lane's authority.

**If operator wishes to ratify additional `ODR` rows from the current GAP corpus** (e.g., to narrow P1 scope to a closeable Jun 1 subset), that decision lives with the operator + this lane's ratification step; it does NOT flow from this roster's authoring.

**No `NOT-PROMISED` rows proposed.** Every task on TASKS.md was promised in some operator-ratified planning round; absent an explicit retraction, the row stays in scope as `GAP` or `PROVEN`, not `NOT-PROMISED`.

---

## §5. Anti-shelfware notes (informational)

Per PR #3949 §4 per-lens-family deadline policy:

- `T-25` (refinement substrate) and `T-26` (boundary carriers) carry `SUBSTRATE LANDED` engineering_state with no co-landed activation; this lane's policy says these qualify as activation-debt entries with the per-family deadline window from PR #3949 §4.2.
- `T-30` (structural fact-density gate) is `ENFORCEMENT_GATE_LANDED` but still cross-cuts P2 close — the gate exists but bridge-masked CI still hides the close failure; honest activation requires bridge dissolution.

These notes are informational; the anti-shelfware debt tracking happens in PR #3949 §4 territory, not in this roster.

---

## §6. What this roster is NOT

- **Not a P1 close receipt.** It enumerates the corpus and dispositions; it does NOT flip P1 from YELLOW (per PR #4013 burn-down) to GREEN. P1 GREEN requires every GAP row to flip to PROVEN per its own `blocking_receipt`.
- **Not a TASKS.md amendment.** No section text in `src/v4/TASKS.md` is altered. The roster reads existing headers + bodies and re-publishes their state under the P1 schema.
- **Not a dispatch instrument.** No worker brief follows from this roster directly. Lane managers (Compiler Spine, Target Realization, Runtime/TestClaim, Self-host/Release) own the per-task dispatch.
- **Not Wave 1 / Wave 2 planning.** PR #3983 §5 owns wave sequencing; this roster is orthogonal — Wave 1 / 2 close receipts will *contribute* to PROVEN row counts but don't replace the per-task roster.

## §7. Related artifacts

- `src/v4/TASKS.md:806-812` — P1 predicate text (operational authority, untouched).
- PR #3973 (`docs/planning/v4-done-predicate-tasks-mapping-2026-05-30.md`) — P1 anchor (the `:806-812` range with the no-enumeration discipline this roster honors).
- PR #4013 (`docs/planning/v4-done-predicate-burn-down-2026-05-30.md`) — P1 YELLOW burn-down framing.
- PR #3949 (`docs/planning/v4-close-receipt-manager-pass-2026-05-30.md`) — two-axis vocabulary + anti-shelfware policy this roster's notes consume.
- PR #4017 + #4050 (`docs/planning/v4-mw-d8-wave1-exit-ledger-2026-05-30.md`) — MW-D8 Wave 1 ledger; complementary, not overlapping.
- PR #4032 (`docs/release/v0.1.0-v4-ship-disposition.md` §1.5) — Run-status discipline; relevant when GAP rows propose PROVEN flips that rest on emit-only evidence.
