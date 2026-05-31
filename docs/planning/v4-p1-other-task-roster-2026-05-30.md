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
| `ODR` | **2** | T-5 (REMOVED), T-27 (DROPPED) — both operator-ratified-already, no fresh proposals |
| `NOT-PROMISED` | **0** | none surfaced on this HEAD |
| **excluded** | **1** | T-15 (P1's own text excepts it) |
| **Total** | **55 in P1 scope + 1 excluded = 56** | |

**P1-headline:** **8 / 55 PROVEN** in P1 scope on this HEAD. 45 GAP rows are the in-flight corpus the live PR / sub-task waves are working through. No row stays GAP for "no owner" reasons; every GAP row's `blocking_receipt` cites a named substrate / consumer / activation receipt or a `[SCHEDULED]` / `[ACTIVE]` header.

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
