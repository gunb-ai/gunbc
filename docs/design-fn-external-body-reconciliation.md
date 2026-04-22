> Part of: [post-l15-phase-plan.md](./post-l15-phase-plan.md) | Status: minor clarification, not a redesign

# Design DB-16 — `FnExternalBody` semantic reconciliation

**Design blocker:** DB-16 (clarify the distinct use cases of `FnExternalBody` / `ArrowBody::Unparsed`)
**Scope:** documentation + one small invariant test. **No substrate change.**
**Origin:** discovered mid-research for DB-14 (PR #497) — tonight's debt audit flagged it as the one unacknowledged scaffold semantic drift.

---

## Problem

`SurfaceItem::FnExternalBody` and its lowered `ArrowBody::Unparsed` partner carry multiple use cases that share the same variant and post-parse shape. **DB-16** documents the **pipeline-relevant** split (cases 1, 2a, 2c). A future reader — including the next session working on M2 parser extensions — could conflate them and dissolve case (1) in a way that breaks pipeline bootstrap or ordering authority.

### Case 1: Parse lag

`std/` block-bodied fn declarations whose bodies contain forms M1(2.7) parser doesn't yet handle (match/lambda/pipe/etc.).

```dag
// dsl/std/logic.dag — case 1
fn classical_not(b: Bool) -> Bool {
  match b { True => False, False => True }
}
```

Parser produces `SurfaceItem::FnExternalBody`; lowering produces a `Declaration` with `ArrowBody::Unparsed(body_span)`. The body source is preserved by span so M2+ parser extensions can complete the lowering.

**Dissolution trigger:** when the M2 parser adopts match/lambda/pipe, re-parsing logic.dag produces a regular `SurfaceItem::Fn` with a full `SurfaceExpr` body. `FnExternalBody` usages in std/ vanish mechanically. The `ArrowBody::Unparsed` variant can then be retired.

### Case 2: Target-native

Compiler-internal fns whose body is fundamentally Rust (or some other host runtime). `src/v3/compiler/pipeline.dag` is the production example:

```dag
// src/v3/compiler/pipeline.dag — case 2
fn parse(source: String, file: String) -> Dag {
  host parse
}
```

At parse time, indistinguishable from case 1: block body that isn't a `SurfaceExpr`. Landed as `SurfaceItem::FnExternalBody` → `ArrowBody::Unparsed(body_span)`.

**Divergence at bootstrap.** A separate bootstrap pass (`bootstrap.rs:238-252`) walks `PipelineStageBinding` declarations and rewrites each stage's Arrow body from `Unparsed` to `ExternalRealization(realization_id)`.

**Dissolution trigger: NEVER via parser growth.** `{ host parse }` is not an unparseable `.dag` expression waiting for M2+ parser grammar. It's a host-runtime bridge; there is no `.dag` body to produce. If the parser grew to parse `host <symbol>` as regular syntax, these fns would still need the `ExternalRealization` bootstrap rewrite — the parser extension just changes the intermediate shape, not the dissolution story.

Substrate accessor callables (DB-14) are **out of scope** for this design doc: do not document them here as a second “legitimate” meaning of `ArrowBody::Unparsed`. **`INVARIANTS.md` §E-9** still governs; the structural follow-up is tracked in **`ROADMAP.md`** (deferral: E-9 substrate accessor bootstrap rewrite).

### Case 2c: `compile` orchestrator (ordering authority)

`pipeline.dag` also declares `fn compile(...) -> String { parse \n lower \n infer \n ... }`. Like other block-bodied compiler fns, it parses as `FnExternalBody` → `ArrowBody::Unparsed(body_span)`. There is **no** `PipelineStageBinding` for `compile` itself — only per-stage fns get that rewrite — so **`Unparsed` persists for `compile` after bootstrap.**

**Runtime authority is structural (PR #637).** Pipeline stage order is read from the declaration order of `PipelineStageBinding` records in the Dag. `ordered_pipeline_stages` walks `dag.declarations()`, filters by `meta_tag == PipelineStageBinding`, and returns them in declaration order. No text slicing, no `body_span` on the runtime path.

**`compile` body role: surface contract, fail-closed cross-check.** The `fn compile { parse \n lower \n ... }` body is the **human-readable pipeline contract**: a reader sees the pipeline in one glance rather than reconstructing it from a binding table. `ordered_pipeline_stages` fail-closes (`reconcile_with_compile_body`) on any drift between the body and the bindings, so the two surfaces cannot silently diverge. Bindings are the single **runtime** authority (P2); the body is the **surface** the bindings commit to; the reconcile keeps that commitment honest (P3).

**Receipt — bridge shape (scheduled, not terminal):** 2c is **not** an interim bridge to `ExternalRealization` (nothing to "realize" for `compile` itself). PR #637 narrowed the prior "body-span is the ordering authority" framing to "bindings are the runtime authority; body is the human-readable contract; reconcile keeps them consistent." That is a real P2/P3 improvement over the prior shape, but the two authored carriers (`compile` body and `PipelineStageBinding` declaration order) still need a behavioral bridge (`reconcile_with_compile_body`) to stay consistent — so 2c remains scheduled debt. **Dissolution trigger:** derivation collapses the two carriers to a single authored source — e.g., a regen/emit step writes the `compile` body from binding declaration order at regen time (or an inverse substrate makes `compile` the sole authored carrier with bindings derived from it). When one surface is emitted from the other, the reconcile retires with it.

### Why the conflation is a real risk

The current doc comment on `SurfaceItem::FnExternalBody` (parse.rs:64-91) describes only case 1's dissolution trigger:

> "`FnExternalBody` has its own dissolution trigger (when match/lambda and block-body parsing land in the parser, block bodies become real `Fn` items with full `SurfaceExpr` bodies)."

A future session that "dissolves `FnExternalBody` per its dissolution trigger" might correctly remove case 1's usages but leave case 2's pipeline.dag stages broken (because their block bodies aren't `SurfaceExpr`-able — they're host bridges). Or they might try to force case 2 into a `SurfaceExpr`, which inverts the layering (compiler-internal bootstrap concepts leaking into the parser).

---

## Design (minimal)

### 1. Update parse.rs doc comment to distinguish the two cases

Extend the `FnExternalBody` variant's doc comment from:

> "`FnExternalBody` has its own dissolution trigger (when match/lambda and block-body parsing land in the parser…)"

to something like:

> "`FnExternalBody` covers two semantically distinct cases that share the parser's post-parse shape (both land as `ArrowBody::Unparsed`):
>
> **Case 1 — parse lag.** std/ block bodies (e.g., logic.dag's `classical_not { match b { ... } }`) whose forms the M1(2.7) parser doesn't yet handle. Dissolves when the parser grows: re-parse produces a regular `Fn` with a `SurfaceExpr` body. `ArrowBody::Unparsed` retires with this case.
>
> **Case 2a — target-native (per-stage).** Compiler-internal fns whose body is a host runtime (e.g., pipeline.dag's `fn parse(...) -> Dag { host parse }`). Dissolves via bootstrap: `PipelineStageBinding` rewrites the Arrow body from `Unparsed` to `ExternalRealization(realization_id)`.
>
> **Case 2c — `compile` orchestrator.** `fn compile(...) { ... }` in `pipeline.dag`: **`Unparsed` persists**. Runtime ordering authority is the declaration order of `PipelineStageBinding` records (`ordered_pipeline_stages`); the body is the human-readable pipeline contract, cross-checked fail-closed against the bindings (`reconcile_with_compile_body`). Not dissolved by `PipelineStageBinding`: 2c is **bridge shape** (two authored carriers kept consistent by reconcile), scheduled until derivation collapses them to one authored source.
>
> The parser does not distinguish these cases — all are 'block body that isn't a SurfaceExpr.' The disambiguator is **downstream role** (binding rewrite vs ordering authority vs parse lag), not "no binding ⇒ case 1."

### 2. Update ROADMAP

Note parse lag vs per-stage pipeline rewrites vs persisted `compile`. Track substrate accessor `Arrow.body` / E-9 alignment under an explicit **deferral** entry (not mixed into DB-16 narrative).

### 3. Add one invariant test

A small acceptance test asserting the structural invariant:

```rust
// src/v3/compiler/tests/m1_fn_external_body_reconciliation_test.rs
#[test]
fn pipeline_stages_lower_to_external_realization_not_unparsed() {
    let dag = Dag::new();
    let stages = v3_compiler::pipeline_compile_order_stage_names()
        .expect("pipeline.dag `compile` body must list stages");
    for stage in stages {
        // Names come from `PipelineStageBinding` declaration order — excludes `compile` itself (case 2c: Unparsed persists).
        let decl = dag.declaration_by_name(&stage).unwrap();
        // ... assert ArrowBody::ExternalRealization for each listed stage
    }
}
```

Locks in that per-stage case 2a's dissolution path runs at bootstrap. `compile` is intentionally **not** in this loop — it stays `Unparsed` for ordering authority (case 2c).

---

## Decision: do NOT split the variant

Considered: split `SurfaceItem::FnExternalBody` into two variants (`FnExternalBody::Unparsed` vs `FnExternalBody::HostBacked`) to make the cases structurally distinct at parse time.

**Rejected because:**
- The parser cannot distinguish the two at parse time. `fn parse(...) { host parse }` and `fn classical_not(...) { match b { ... } }` are both "block body that isn't a `SurfaceExpr`." Any distinction the parser tried to make (e.g., "does the body contain the keyword `host`?") would be a string-level heuristic, not a structural fact.
- The divergence genuinely happens downstream, at bootstrap. Splitting at parse time forces the parser to know about compiler-internal concepts (pipeline stages) that are properly below it.
- One parse-time variant + two bootstrap paths (parser-growth rewrite of `Unparsed` vs `PipelineStageBinding`-style rewrite to `ExternalRealization`) is the right layering.

The proper disambiguator is **downstream bootstrap / authority role**, not a single boolean: `PipelineStageBinding` for per-stage pipeline fns; persisted `Unparsed` + fail-closed reconcile against binding order for `compile`; absence of those *and* no special pipeline role implies parse lag (case 1).

---

## Out of scope

- Splitting `SurfaceItem::FnExternalBody` into per-case variants. Rejected above.
- Changing `ArrowBody::Unparsed`'s documentation beyond DB-16 scope. Per-stage pipeline stages reach `ExternalRealization` before inference; **`compile` (case 2c) keeps `Unparsed`** for ordering authority — dag.rs must distinguish case 1 vs 2a outcome vs 2c (not “all `Unparsed` is parse lag”).
- Dissolving case 1. That's a full M2 parser work item, not DB-16's scope.
- Auditing DOWNSTREAM_REQUIREMENTS.md entries that reference FnExternalBody — deferred to the docs-pruning PR.

---

## Acceptance

- [ ] Doc comment on `SurfaceItem::FnExternalBody` in parse.rs updated per §1 above — distinguishes cases 1, 2a, 2c and names their dissolution / persistence story (no accessor “second meaning” narrative).
- [ ] Doc comment on `ArrowBody::Unparsed` in dag.rs lists case 1, 2c, and per-stage 2a rewrite — not "case 1 only."
- [ ] Invariant test `pipeline_stages_lower_to_external_realization_not_unparsed` added and green.
- [ ] No substrate shape change committed.

---

## Associations

- **`ROADMAP.md`** — deferral entry for E-9 substrate accessor bootstrap rewrite (accessor `Arrow.body` → `ExternalRealization(ref)`); substrate accessor semantics are **not** canonically documented in DB-16.
- **Pipeline** — `PipelineStageBinding` + `materialize_pipeline_realizations` (case 2a).
- **`src/v3/compiler/src/bootstrap.rs`** — `materialize_pipeline_realizations`
- **`src/v3/compiler/pipeline.dag`** — case 2a (per-stage `host` fns) and case 2c (`fn compile { ... }` ordering surface)
- **`dsl/std/logic.dag`** — case 1 example (`classical_not`, `classical_and`, `classical_or`)
