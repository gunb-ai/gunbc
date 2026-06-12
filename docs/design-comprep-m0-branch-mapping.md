# COMPREP M0 Branch Mapping — Design Table (dep-graph-2026-06-12 §2)

> **Status: FAN-IN (i) GREEN.** gunbc#4699 merged (main 7e126d1a7c); M0-B1 eval-by-execution
> keystone `comprep_branch_eval_by_execution_keystone_holds` passes via `--claim-run`.
> Keystone source: `fn one_or_two() -> Int { if true then 1 else 2 }` (literal Bool cond;
> param-ref M0-B1a deferred). Review artifact for COMPREP wave-2 keystone scoping
> (Branch first in the §2 sequence Branch → Bind → Loop). Mirrors the add-keystone row shape
> used for COMPREP wave-1 (`03_body_producer`, `06_value_expression`, `comprep_eval_by_execution`).

## 1. M0 keystone (what “done” means)

**Green criterion (analogous to wave-1 add):** a source-ingested `if`-then-else body (not a
hand-built fixture) flows parse → body_producer → resolve → infer → eval; `run_test_claim`
compares executed result to expected **by execution**; the discriminating red is a condition
perturbation (`true` → `false`) flipping the outcome.

**M0 scope gate (E-10):** expression-bodied `if … then … else …` on a **Bool** condition
only. Block-bodied `if`, `match`, and nested/multi-arm forms are catalogued below as
follow-on rows with explicit substrate gates — not M0 blockers.

## 2. Authority anchors (M9 DFS)

| Layer | Authority | Role for Branch |
|---|---|---|
| Substrate behavior | `std/node.dag` — `Behavior::Branch`, `PositionalEdges`, `count(children) >= 1` | closed coproduct arm; edge discipline is positional-only today |
| Eval dispatch | `compiler/05_eval.dag` — `eval_branch_node`, `eval_first_runtime_argument` | fold evaluates positional children → `args`; branch interpreter receives `args[0]` as condition |
| Runtime algebra slot | `std/runtime.dag` — `BranchInterpreter.choose_branch` | `(Node, RuntimeValue, EvaluationEnvironment) → Outcome<Node>` selects arm subgraph |
| Value emit kind (planned) | `design-value-emit-schema.md` §4.1 — `TargetValueExprConditional` | lands with producer (E-10); not in `target_model.dag` yet |
| Surface grammar | `extdeps/languages/dag.dag` — `dag_production_if_then_form`, `dag_production_if_expr` | source forms the body producer must recognize |
| v3 lowering precedent | `v3/compiler/src/lower.rs` — `SurfaceExpr::If` → `BranchNode` with `input` + 2 `BranchPath`s | semantic reference; v4 compresses to flat positional children (§3) |

## 3. Proposed v4 Branch child layout (M0 convention)

v3 `BranchNode` carries `input` + labeled `paths` with `BranchPattern`. v4 `Branch` today
admits only **positional** children (`behavior_edges_conform`). M0 adopts a **fixed positional
layout** for Bool `if`-then-else without opening the edge-discipline coproduct:

```
ComputationNode { behavior: Branch }
children (positional, count == 3):
  [0] condition : <expr Behavior sub-DAG>
  [1] then_arm  : <expr Behavior sub-DAG>
  [2] else_arm  : <expr Behavior sub-DAG>
```

**Eval wiring (already in `05_eval.dag`):**

1. `fold_node` evaluates children left-to-right; each positional child value is appended to `args`.
2. `eval_branch_node` takes `args[0]` as the condition (`eval_first_runtime_argument`).
3. `choose_branch(node, condition, env)` returns one of `node.children[*].target` (arm subgraph).
4. `eval_runtime_node` executes the chosen arm (second evaluation of that subgraph — see §8 Q-B3).

**Relation to `phase4/branch_dispatch` fixture:** that corpus row is a **2-child degenerate**
Branch (two `Transform` arms, no condition child). It exercises well-formed / content_hash only;
it is **not** the M0 semantic shape and must not be used as the producer contract.

## 4. Mapping table

Columns: **row id**, **source form**, **body_producer output (substrate)**, **target_model
algebra row**, **eval case**, **discriminating claim**.

### 4.1 M0 keystone rows (in scope for first landing PR)

| Row | Source form (dag) | Behavior subgraph (substrate) | target_model algebra row | Eval case | Discriminating claim |
|---|---|---|---|---|---|
| **M0-B1** | `if <expr> then <expr> else <expr>` — `dag_production_if_then_form` | `Branch` with 3 positional children `[cond, then, else]` per §3; each child is the recursive body_producer output for the corresponding parse subtree | **Kind:** `TargetValueExprConditional` (new, substrate). **Wire:** `TargetValueExpression { kind: TargetValueExprConditional, node: … }` with named edges on the wire atom: `branch_condition`, `branch_then_arm`, `branch_else_arm` → projected `TargetValueExpression` children. **Projection row field:** `conditional_form: TargetConditionalShape { if_token, then_token, else_token }` (expr-bodied; tokens are `Symbol` classes, not spellings). **Per-language serialize:** derived `FormalProduction` from `conditional_form` + recursive child tokens (bidir §6). | `eval_branch_node` + canonical `choose_branch_bool_if_else` (new runtime fn): `count(children)==3` → if condition is runtime Bool true pick child `[1]`, false pick `[2]`; else `eval_rejected_branch_argument_absent` / typed refusal. Condition value = `args[0]` from child `[0]` evaluation. | **Eval:** `fn f(b: Bool) = if b then 1 else 2` — `f(true)==1`, `f(false)==2`; perturb condition literal. **Emit (wave 3 coupled):** same body, perturb `then`/`else` operand (`+`↔`-`) changes emitted source. |
| **M0-B1a** | Same surface; condition is a **parameter ref** (`dag_production_ident` in cond position) | Same 3-child `Branch`; child `[0]` is `Value`/param-ref leaf (binding resolved to `binding_id` / interim ident atom) | Same `TargetValueExprConditional`; `branch_condition` projects via `TargetValueExprBindingRef` | Same `choose_branch_bool_if_else`; condition from runtime binding lookup when param ref evaluated | **Eval:** `fn pick(b: Bool) = if b then x else y` with call-site arg perturbation (mirrors add operand-swap receipt) |

### 4.2 Catalogued follow-on rows (same Behavior arm; gated past M0)

| Row | Source form | Substrate shape | target_model row | Eval case | Gate |
|---|---|---|---|---|---|
| **M0-B2** | `if <expr> <block> else <block>` — `dag_production_if_block_form` | 3-child `Branch`; block children are `Bind` chains or statement sequences | `TargetValueExprConditional` + **statement wrapper** on `conditional_form` (Q-V2: wrapping owned by function-decl production, not value projection) | Same Bool `choose_branch`; block children evaluate via `Bind`/`Loop` rows | **Bind producer** (§2 next behavior); Q-V2 statement context |
| **M0-B3** | `if <expr> <block> else if …` — nested `dag_production_if_expr` in else | Nested `Branch` in else arm (child `[2]` is another `Branch`) | Recursive `TargetValueExprConditional` on else arm | Recursive `eval_branch_node` | M0-B1 green + value-projection totality on nested shape |
| **M0-B4** | `match <expr> { }` — empty arms | Ill-formed / reject at producer | — | — | Parser/grammar rejects or producer fail-closed |
| **M0-B5** | `match <scrutinee> { <pat> => <body>, … }` — `dag_production_match_expr` | **Needs pattern authority on arms** — positional-only `Branch` cannot carry `BranchPattern` today (v3: `BranchPath.pattern`) | `TargetValueExprConditional` generalization or `TargetValueExprMatch` kind | `choose_branch` = pattern dispatch against runtime aggregate/primitive scrutinee | **Escalate Q-B1:** extend `Branch` edge discipline (named pattern edges) vs encode pattern in arm wrapper vs reintroduce path record |
| **M0-B6** | `match` on `Bool` with `True`/`False` literal patterns (2 arms) | Degenerate **M0-B5** with closed pattern set; encodable as M0-B1 if producer lowers `match b { True => t, False => e }` → `if`-shaped `Branch` | Same as M0-B1 (desugar row, not separate kind) | Same as M0-B1 | Producer desugar policy — preferred for M0 to avoid Q-B1 |
| **M0-B7** | Rust/TS block `if` without `then` keyword | Target-specific grammar only (`rust_production_if_expr`) | `conditional_form` block variant per language projection | Same eval | Language projection row only; substrate shape still M0-B2 |

## 5. target_model algebra row detail (M0-B1)

Substrate-owned additions (land with producer + template twin per E-10 / bidir §6.2):

```dag
// std/compilers/target_model.dag (planned)
type TargetValueExpressionKind
  | …
  | TargetValueExprConditional   // wave-2 / M0-B1

type TargetConditionalShape {
  if_token: Symbol
  then_token: Symbol
  else_token: Symbol
}

type TargetValueExpressionProjection {
  …
  conditional_form: TargetConditionalShape   // new field; lands with M0-B1 consumer
}

// Wire atom + named edges (mirrors TargetValueExprPrimitiveApply pattern)
data target_value_expr_field_branch_condition: Symbol = …
data target_value_expr_field_branch_then_arm: Symbol = …
data target_value_expr_field_branch_else_arm: Symbol = …
```

**Projection fold row (value_expression algebra):** one arm for `ComputationNode { behavior: Branch }`
when `count(positional children)==3` and inferred condition type is `Bool` (M0 gate):

```
Behavior Branch [cond, then, else]
  → TargetValueExprConditional {
      branch_condition: project(cond),
      branch_then_arm:  project(then),
      branch_else_arm:  project(else)
    }
```

**Serialize row:** `target_value_expression_to_concrete_tokens` gains
`TargetValueExprConditional` arm reading `projection.conditional_form` tokens + recursive child
emission (same structure as `TargetValueExprPrimitiveApply`).

## 6. Eval algebra row detail (M0-B1)

**Stage fold arm (conceptual — lands in model/runtime, not hand-match in `05_eval`):**

| Algebra slot | Row shape | Operation |
|---|---|---|
| `InterpretationAlgebra.branch` | `BranchInterpreter { choose_branch: choose_branch_bool_if_else }` | `fn(node, condition, env) -> Outcome<Node>` |
| `choose_branch_bool_if_else` | `count(node.children)==3` + `runtime_bool(condition)` | `true` → `list_nth(children, 1).target`; `false` → `list_nth(children, 2).target` |
| `eval_branch_node` | (existing) | `args[0]` = evaluated condition; delegate arm pick to `choose_branch`; `eval_runtime_node` on chosen arm |

**Canonical runtime home:** `extdeps/runtimes/v4_evaluator.dag` (replace
`v4_eval_wave1_choose_branch` reject stub for M0 claims) with dissolution mark tying stub
deletion to M0-B1 claim green.

**Refusal cases (fail-closed):** wrong child count; condition not runtime Bool; arm index
out of range — each maps to existing or new `eval_rejected_*` reason symbols.

## 7. body_producer algebra row (conceptual)

| Parse surface | Producer row | Output |
|---|---|---|
| `if_then_form` AST node | `produce_branch_bool_if_else(cond, then, else)` | `ComputationNode { behavior: Branch }` + 3 positional child edges |
| Recursive sub-expressions | delegate to existing value/transform producers (wave-1) | param refs, `Transform` primitive-apply, literals |

Producer lands in `03_body_producer` as **data-row dispatch** (stage-fold §1.4 pin), not a new
parallel `if` matcher outside the algebra.

## 8. Open questions — escalate before implementation

- **Q-B1 — match / pattern authority.** v4 `Branch` is positional-only; v3 `BranchPath.pattern`
  has no v4 home. **Recommendation for M0:** desugar Bool `match` to M0-B1 `if` (row M0-B6);
  defer sum-type `match` (M0-B5) until edge-discipline extension is decided.
- **Q-B2 — child[0] typing.** M0 gate requires infer to admit `Bool` on condition child; arm
  types must unify at `Branch` merge. Confirm infer fold row exists or is part of M0 PR.
- **Q-B3 — eager child evaluation.** `eval_fold` evaluates all positional children before
  `eval_branch_node`; arms run before `choose_branch` selects. Accept for M0 (matches current
  fold semantics) or introduce short-circuit policy in `eval_edge_is_runtime_argument`?
- **Q-B4 — block-bodied `if`.** Requires `Bind` (statement sequence). Sequence with Bind M0
  mapping doc; do not conflate into Branch M0 keystone.

## 9. Non-goals (this doc)

- `Bind` / `Loop` mapping tables (sibling §2 work items).
- Compiler-stage edits before #4699 merge.
- `TargetValueExprConditional` kinds or projection fields without producer (E-10).
- v3 `BranchEmitParticipation::UserMatch` port — emit-tier concern for M0-B5+.

## 10. Implementation sequencing (post-#4699)

**PR model (LOCKED 2026-06-12 — FINAL, coordinator authoritative):** gunbc#4730 =
**fan-in (i) standalone-merge** at 2 api-review approvals (literal-Bool keystone row is stable;
no `06_translate`; no load-bearing design-sign). **(ii) emit** + **(iii) round-trip** are
**separate follow-on carrier PRs**. pick(b) / Q-B3 lazy-arm is **(ii)/B1a downstream work** —
not a gate on #4730.

| Fan-in | PR | Status |
|--------|-----|--------|
| **(i) eval-by-execution** | gunbc#4730 | **GREEN** — merge at 2 approvals |
| **B1a + Q-B3** | follow-on (before/with ii) | pick(b) repro; lazy-arm fix if eager-both-arms |
| **(ii) emit** | follow-on carrier PR | ONE `TargetValueExprConditional` row + `06_translate` |
| **(iii) round-trip** | follow-on carrier PR | byte-identical; gates §3-deep spawn |

1. ~~Land `choose_branch_bool_if_else` + M0-B1 eval claim.~~ **Done (#4730).**
2. ~~Land body_producer row + source-bridged parse fixture.~~ **Done (#4730).**
3. Repro `pick(b)`; resolve Q-B3 before (ii) emit depends on it.
4. Land substrate kinds + wire validators (`target_model.dag`) **(fan-in ii PR).**
5. Land value_expression projection row **(fan-in ii PR).**
6. Round-trip keystone **(fan-in iii PR).**

## 11. Fan-in (ii) emit — design-sign routing (LOCKED 2026-06-12)

Load-bearing `06_translate` + `target_model` algebra row **requires design sign before merge**.

**Authority route (effective now — do NOT deviate):**

- **NOT** still-raven (archived/unreachable — messages bounce).
- When fan-in (ii) diff is ready: send diff + design question to **sharp-fox-370** (program coordinator).
- Coordinator escalates to **parent (snappy-crab)** to spawn a live design-ruling work-item and bind a signing authority (same mechanism as tidy-stag-172 fold ruling after still-raven archival).
- Merge fan-in (ii) PR only after live authority sign on the structural diff.

Tripwire unchanged: ONE `TargetValueExprConditional` row; STOP/escalate if >1 row or multi-site translate edits.
