> Part of: [post-l15-phase-plan.md](./post-l15-phase-plan.md) | Status: minor clarification, not a redesign

# Design DB-16 — `FnExternalBody` semantic reconciliation

**Design blocker:** DB-16 (clarify the two distinct use cases of `FnExternalBody` / `ArrowBody::Unparsed`)
**Scope:** documentation + one small invariant test. **No substrate change.**
**Origin:** discovered mid-research for DB-14 (PR #497) — tonight's debt audit flagged it as the one unacknowledged scaffold semantic drift.

---

## Problem

`SurfaceItem::FnExternalBody` and its lowered `ArrowBody::Unparsed` partner carry two semantically distinct use cases that share the same variant and post-parse shape. A future reader — including the next session working on M2 parser extensions — could conflate them and dissolve case (1) in a way that breaks case (2).

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
> **Case 2 — target-native.** Compiler-internal fns whose body is a host runtime (e.g., pipeline.dag's `fn parse(...) -> Dag { host parse }`). Dissolves via bootstrap: a `PipelineStageBinding`-style pass rewrites the Arrow body from `Unparsed` to `ExternalRealization(realization_id)`. Never becomes a parseable `.dag` body.
>
> The parser does not distinguish these cases — both are 'block body that isn't a SurfaceExpr.' The disambiguator is **the presence of a corresponding `*Binding` declaration** that rewrites the Arrow body at bootstrap; absence means case 1."

### 2. Update ROADMAP to note the two-category split

The ROADMAP's FnExternalBody dissolution reference (if any) should distinguish the two dissolution paths. Currently the DOWNSTREAM_REQUIREMENTS.md doc lists `FnExternalBody` scaffolds together; a separate PR pruning that file can categorize each entry as case 1 vs case 2.

### 3. Add one invariant test

A small acceptance test asserting the structural invariant:

```rust
// src/v3/compiler/tests/m1_fn_external_body_reconciliation_test.rs
#[test]
fn pipeline_stages_lower_to_external_realization_not_unparsed() {
    let dag = bootstrap_dag();
    for stage in &["parse", "lower", "infer", "compute_ownership",
                   "lens_complexity", "emit", "compile"] {
        let decl = dag.declaration_by_name(stage).unwrap();
        if let TypeConnective::Arrow { body, .. } = &decl.connective {
            match body {
                ArrowBody::ExternalRealization(_) => {},  // case 2 dissolved ✓
                ArrowBody::Unparsed(_) => panic!(
                    "pipeline stage `{stage}` still has Unparsed body — \
                     the PipelineStageBinding bootstrap pass did not run \
                     or is broken. This is the case 2 dissolution gate."
                ),
                other => panic!("pipeline stage `{stage}` has unexpected body {other:?}"),
            }
        }
    }
}
```

Locks in that case 2's dissolution path actually runs at bootstrap. If a future change breaks the `PipelineStageBinding` pass, the test fires.

---

## Decision: do NOT split the variant

Considered: split `SurfaceItem::FnExternalBody` into two variants (`FnExternalBody::Unparsed` vs `FnExternalBody::HostBacked`) to make the cases structurally distinct at parse time.

**Rejected because:**
- The parser cannot distinguish the two at parse time. `fn parse(...) { host parse }` and `fn classical_not(...) { match b { ... } }` are both "block body that isn't a `SurfaceExpr`." Any distinction the parser tried to make (e.g., "does the body contain the keyword `host`?") would be a string-level heuristic, not a structural fact.
- The divergence genuinely happens downstream, at bootstrap. Splitting at parse time forces the parser to know about compiler-internal concepts (pipeline stages, substrate accessor realizations — see DB-14) that are properly below it.
- One parse-time variant + two bootstrap paths (parser-growth rewrite of `Unparsed` vs `PipelineStageBinding`-style rewrite to `ExternalRealization`) is the right layering.

The proper disambiguator is **the presence or absence of a corresponding binding declaration** — `PipelineStageBinding` for pipeline stages, similar constructs for whatever else uses case 2 in the future (DB-14 uses the `meta_tag` marker instead; that's a parallel mechanism).

---

## Out of scope

- Splitting `SurfaceItem::FnExternalBody` into per-case variants. Rejected above.
- Changing `ArrowBody::Unparsed`'s documentation. Its doc in dag.rs:492-505 is clean — it says "surface-grammar lag," which is case 1. Case 2 reaches `ArrowBody::ExternalRealization` before inference runs, so `Unparsed` never persists for case 2 post-bootstrap.
- Dissolving case 1. That's a full M2 parser work item, not DB-16's scope.
- Auditing DOWNSTREAM_REQUIREMENTS.md entries that reference FnExternalBody — deferred to the docs-pruning PR.

---

## Acceptance

- [ ] Doc comment on `SurfaceItem::FnExternalBody` in parse.rs updated per §1 above — distinguishes the two cases and names their dissolution triggers.
- [ ] Doc comment on `ArrowBody::Unparsed` in dag.rs confirmed to reference "case 1 only" — or updated if it currently reads ambiguously.
- [ ] Invariant test `pipeline_stages_lower_to_external_realization_not_unparsed` added and green.
- [ ] No substrate shape change committed.

---

## Associations

- **DB-14** ([design-substrate-external-primitives.md](./design-substrate-external-primitives.md)) — substrate accessors also use the case 2 pattern (trivial body + bootstrap rewrite). DB-14 uses `meta_tag` as the disambiguator; pipeline.dag uses `PipelineStageBinding`. Both are valid; the common thread is "downstream binding/marker determines the rewrite."
- **`src/v3/compiler/src/parse.rs:64-91`** — `SurfaceItem::FnExternalBody` doc comment to update
- **`src/v3/compiler/src/dag.rs:492-505`** — `ArrowBody::Unparsed` doc comment to review
- **`src/v3/compiler/src/bootstrap.rs:238-252`** — the production case 2 bootstrap pass
- **`src/v3/compiler/pipeline.dag`** — the production case 2 example
- **`dsl/std/logic.dag`** — case 1 example (`classical_not`, `classical_and`, `classical_or`)
