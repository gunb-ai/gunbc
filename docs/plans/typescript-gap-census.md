# TypeScript first-class gap census — discriminating construct audit (Lane C / Track T)

**Status:** audit-first discriminating census · **approved 2026-07-01** (loyal-bee-794) · linked from `ROADMAP.md` `5-ts-first-class` · companion to `v2-self-hosting.md` Track T. **`dag/gunbc/plans/typescript_gap_census.dag` is the authority** (DESIGN §6); `docs/plans/typescript-gap-census.md` is its generated projection (PlanArtifact).

**Method:** enumerate construct families `src/v2` actually uses (compiler substrate behaviors in `src/v2/compiler/*` + TargetModel emit surfaces in `src/v2/extdeps/languages/typescript.dag`). For each family report three independent bars: (a) grammar-inverse row enrolled on the committed TargetModel node, (b) VEP / pipeline emits the expected **TypeScript source string** (witness PASS), (c) **`tsc` / emit_host accepts the output** (consumer green-by-execution). Every FAIL names a witness or authority site — not a stub count.

> **Two-bar clarification (parent reframe 2026-07-01).** The GREEN column below is **bar (b) only** — VEP produces the expected TS source string. Bar (c) — `tsc` actually compiles the emitted output — is **UNMET even for `add`**: `typescript_descriptor_node_ts_node_run_add_holds` (`src/v2/test/claim/manual/typescript_descriptor_node_run_test.dag`) and `emit_host_gate_passes` (`dag/tools/emit_host_gate.dag`) both **FAIL wet** in execution. A construct is not genuinely first-class until bar (c) is green. **Do not stack breadth on a red tsc foundation.**

---

## 1. Baseline (merged on main, not stub PRs)

- #5695 — Char atom-realization + green typed-fn emit
- #5701 — FieldAccess + RecordConstruct + `tsc --noEmit` oracle (source-string witnesses; bar (c) still red — see #19)
- #5704 — operators by inhabitance (test-catalog only, not default TargetModel)
- #5938 — collection-method grammar rows
- ~55 `typescript_*` / `ts_*` test fns under `src/v2/test/claim/` (manual + execution trees)

## 2. Census table

### VEP source-string GREEN (bar (b) witness PASS)

1. **#1 fn add** — grammar-inverse row + full infer→emit pipeline. Witness: `add_body_ts_emit_add_fn_accepts_holds`, `typescript_add_emit_add_fn_accepts_holds` (`add_body_emit_typescript_test.dag`, `typescript_add_emit_translate_test.dag`).
2. **#2 field access** — VEP. Witness: `ts_field_access_emit_source_holds` → `"o.x"` (`typescript_field_access_emit_test.dag`).
3. **#3 record construct** — VEP. Witness: `ts_record_construct_emit_source_holds` (`typescript_record_construct_emit_test.dag`).
4. **#4 bind let IIFE** — VEP. Witness: `ts_bind_let_iife_emit_exact_holds` (`typescript_bind_let_iife_emit_test.dag`).
5. **#5 closure (value-producing)** — VEP. Witness: `ts_closure_emit_by_execution_exact_holds` (`typescript_closure_emit_by_execution_test.dag`).
6. **#6 enum / disj union** — VEP. Witness: `ts_enum_union_emit_by_execution_exact_holds` (`typescript_enum_union_emit_by_execution_test.dag`).
7. **#7 module import** — VEP. Witness: `ts_import_emit_by_execution_exact_holds` (`typescript_import_emit_by_execution_test.dag`).
8. **#8 effect apply** — VEP. Witness: `ts_effect_io_emit_holds` (`typescript_effect_io_emit_test.dag`).
9. **#9 fold_call body via translate** — VEP. Witness: `fold_call_closure_emit_keystone_holds` (`fold_call_closure_emit_test.dag`).
10. **#10 operators (arith/cmp)** — VEP with **ad-hoc test catalog only** (`add_body_ts_emit_catalog_minus_discriminates`, `add_body_ts_emit_missing_catalog_rejects`). Default `ts_operator_realizations_catalog_node()` enrolls **OpAdd only** (`typescript.dag:640`) — see #16.

### FAIL-CLOSED (emit explicitly Rejected — named)

1. **#11 loop in statement-sequenced block mode** — `ts_loop_statement_sequenced_fails_closed_holds` (`typescript_bind_let_iife_emit_test.dag`; also `typescript_closure_emit_by_execution_test.dag`). `.dag` loop emits; TS `StatementSequenced` mode rejects.
2. **#12 closure in statement-sequenced block mode** — `ts_closure_statement_sequenced_fails_closed_holds` (same files).

### FAIL-OPEN (compiler-scale; no green bar (b)+(c) on default model today)

1. **#13 match / coproduct dispatch** — `match_form.match_token = ^ts_token_unwired_match` (`typescript.dag:623`); no TS match witness; compiler uses `Match` heavily (`05_eval.dag`).
2. **#14 bind-in scoping** — `let_form.in_token = ^ts_token_unwired_bind_in` (`typescript.dag:606`).
3. **#15 loop form wiring** — `loop_form.loop_token = ^ts_token_unwired_loop` (`typescript.dag:609`).
4. **#16 default operator catalog** — `ts_operator_realizations_catalog_node()` rows = `[ts_add_operator_realization_row()]` only (`typescript.dag:640–641`). Algebra ops miss on default model (`add_body_ts_emit_missing_catalog_rejects`).
5. **#17 grammar-inverse rows beyond add** — committed `ts_mvp1_translation_rules_node()` has **1** child (`fn_add`); witness bundle `ts_mvp1_translation_rules_witness()` expects **3** (add + type_alias + pr3) (`typescript.dag:1381–1414`).
6. **#18 branch dispatch** — compiler uses `Branch` (`04_infer` / `05_eval`); zero TS emit witnesses.
7. **#19 tsc-green / emit_host oracle** — §5 spec-without-execution crack. Sole consumer path: `ts_host_transport_mvp1_descriptor` + `emit_host_gate.dag`. **`typescript_descriptor_node_ts_node_run_add_holds` FAIL** (wet `claim_batch`); **`emit_host_gate_passes` FAIL** (wet). Bar (c) red even for add.
8. **#20 whole `src/v2` → TS** — no module; Route-A tsc analogue not started (terminal slice E).

## 3. Discriminating counts

- **VEP source-string GREEN families:** 9 (#1–#9; #10 green only with ad-hoc catalog)
- **FAIL-CLOSED:** 2 (#11–#12)
- **FAIL-OPEN compiler-scale gaps:** 8 (#13–#20)
- **tsc-green (bar c):** 0/N wet — add fixture attempted, red in execution (#19)
- **Grammar-inverse rows:** 1 committed / 3 in witness bundle / ~15 wave2a productions defined

## 4. Size call

**LARGE multi-slice** — not a single row PR. Cheap tranche = grammar-row extensions (#17) + operator catalog derivation onto default TargetModel if row-only (#16). Load-bearing tranche = Match (#13) + loop/bind forms (#14–#15) + Branch (#18) — **HOLD for parent sign** before authoring TargetModel surfaces. Whole-tree tsc (#20) is terminal after per-construct gaps close.

## 5. Sequencing (revised 2026-07-01)

1. **A. Land this census** (this plan) — audit-first record, generated md.
2. **A2. tsc-green oracle REAL on existing VEP-green slices (#19 pulled forward)** — per-construct `tsc` acceptance on #1–#9 before stacking breadth. Converts string-green families into compile-green. Highest-value foundation work.
3. **B. Cheap row extensions (#17)** — enroll type_alias + pr3 rows into committed `ts_mvp1_translation_rules_node`; rows only, no new TargetModel surface.
4. **C. Operator catalog (#16)** — proceed **only** if genuinely row-derivation onto the default model; if it needs a new TargetModel surface → load-bearing → sign first.
5. **D. HOLD for parent sign:** Match (#13) + loop/bind (#14–#15) + Branch (#18) — compiler-scale TargetModel surfaces.
6. **E. Whole-tree Route-A tsc (#20)** — terminal.

**Priority note:** Lane C is language-breadth; operator retains burn-down sequencing call. Proceed A → A2 → B meanwhile; HOLD D for both parent sign and operator priority.

## Dissolution trigger (DESIGN §6)

Delete this census once every FAIL-OPEN row (#13–#20) is closed with execution-grounded witnesses and bar (c) tsc-green holds on the default TargetModel — the audit record is superseded by the substrate facts it enumerated.
