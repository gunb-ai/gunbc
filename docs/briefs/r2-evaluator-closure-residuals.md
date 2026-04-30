# R2 Evaluator — closure residuals (PR-D / PR-E / TC2)

**Status:** PROPOSAL — **docs-only**. Scoped to Evaluator Manager territory; intended for **R2 Release Manager** consumption when recording closure ledger notes ([`docs/r2-closure-ledger.md`](../r2-closure-ledger.md)). Does **not** introduce fixtures, Rust, substrate carriers, or new `TestClaim` variants; does **not** reopen implementation dispatch beyond what the cited briefs already gate.

**Parent:** [`r2-evaluator-manager.md`](r2-evaluator-manager.md). **Live gating companion:** [`r2-evaluator-cadence-verification-matrix.md`](r2-evaluator-cadence-verification-matrix.md).

## PR-C (reflection completeness) — intentionally out of scope here

**Landed:** PR-C **spec** authority is [`docs/design-reflection-completeness.md`](../design-reflection-completeness.md) (**LANDED via #1129**); Evaluator Manager’s **Lens application** / reflection sub-lane disposition already treats that structural work as landed. **Rust / compiler hooks** that consume the spec are likewise **landed** at the integration seams cited there and in the manager table.

**Remaining PR-C surface (not listed in this file):** follow-on work is **dissolution / structural-gate consumption** (how reflection completeness evidence wires into named `TestClaim` rows and the R2 closure ledger), **not** an open “implementation backlog” for PR-C itself. This note records **PR-D / PR-E / TC2** deferrals only so [`docs/r2-closure-ledger.md`](../r2-closure-ledger.md) consumers get a **bounded** Evaluator slice without mistaking this file for the full Evaluator closure residual inventory.

## PR-D — cross-target equivalence harness

**Landed (Evaluator-side, as of this note):** PR-D **slice 0** (named `.dag` claims + suite) and **slice 1** (`DifferentialEquals` scaffold alongside the existing harness primitives) per [`r2-pr-d-cross-target-equivalence-harness-primitives.md`](r2-pr-d-cross-target-equivalence-harness-primitives.md).

**Closure residual (explicit deferral):** **`ForAllTargets` / emit-scoped receipts** and **L5 corpus execution at R2** stay out of scope until **R2-T-Ground-LanguageSpec**, **all Shape A targets grounded**, and the **T-Verification-L4-L7-Direct corpus** dependencies in that brief’s Dependencies table are **all live and cited together**. Until then, the matrix’s PR-D row remains the authoritative “next allowed shape” boundary ([`r2-evaluator-cadence-verification-matrix.md`](r2-evaluator-cadence-verification-matrix.md) — PR-D row).

## PR-E — lens application on reflected program DAG

**Landed (Evaluator-side, as of this note):** reflection spine plus **reflect → apply** slice (`fold_lens_over_reflected_program`) per [`r2-pr-e-lens-application-over-reflected-program-dag.md`](r2-pr-e-lens-application-over-reflected-program-dag.md), with completeness still tracked against [`docs/design-reflection-completeness.md`](../design-reflection-completeness.md) as in the Evaluator manager sub-lane table.

**Closure residual (explicit deferral):** the **full** `Lens<C>` / `DimensionReport<C>` fold and **PB-Runtime-aligned body semantics** for the deeper fold remain deferred because an **executable evaluator strategy surface** and **body evaluator** are not yet landed at the boundary those semantics assume. **Higher-order (HO) and lens-instance** prerequisites for that deeper fold continue to be **tracked outside this Evaluator slice** (PR-E brief plus adjacent Modeling / lens-framework / substrate coordination as cited there)—this note records the deferral only; it does not subsume those lanes’ authority.

## TC2 — evaluation-order independence

**Landed (Evaluator-side, as of this note):** the **author-now** deferred fixture path (`evaluation_order_independent_lens_results` / `tc2_evaluation_order_independence_deferred.dag`) per Evaluator manager Acceptance + [`r2-pr-a-runtime-value-model.md`](r2-pr-a-runtime-value-model.md) cadence.

**Closure residual (explicit deferral):** a **strict output-equality / proof-grade** evaluation-order independence claim stays **blocked** until at least **two** executable evaluation **strategies** (or **input orders**) run through the **same** evaluator boundary so the predicate can compare outputs without pretending a single-strategy world is sufficient. The TC2 row in [`r2-evaluator-cadence-verification-matrix.md`](r2-evaluator-cadence-verification-matrix.md) remains the operational gate text (“predicate strengthening **blocked** until strategies exist”).
