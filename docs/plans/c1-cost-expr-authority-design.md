# C1 — SizeExpr / CostExpr authority in `v2.lens.cost`

> **Status: CARRIER CANDIDATE (open PR #7888, 2026-08-06).** Proposes `v2.lens.cost.expr` with `SizeExpr`/`CostExpr`, `normalize_cost_expr_to_symbolic`, closed `CostExprRefusalCause`, and witnesses in `bounded_summation_test.dag` / `expr_refusal_test.dag`. **Delivery** is when those symbols execute on main after merge — not while the PR is open. Design authority is this note (`docs/plans/c1-cost-expr-authority-design.md`); `cost_lens` / `symbolic_cost_fold` unchanged in C1.
>
> **Authority:** [`v2-complexity-capability-parity.md`](v2-complexity-capability-parity.md) §5–§7 item 2. This note is the C1 design deliverable requested by the lane brief.

## 1. What the tree carries today (verified 2026-08-05)

**`src/v2/lens/cost.dag`** (~671 lines, load-bearing, `WallNow` / `lens_contract_cost` / `BoundConsumerWitness`):

| Carrier | Present | Notes |
| --- | --- | --- |
| `SizeVariable { source: Node }` | yes | sole size identity today; no separate `SizeExpr` |
| `SymbolicCost` coproduct | yes | `ConstantCost` … `FactorialCost`, `SumCost`, `ProductCost`, `UnknownCost { diagnostic: Diagnostic }` |
| `AsymptoticClass`, `CostCompositionMode` | yes | dominance lattice + fold composition modes |
| `symbolic_cost_dominates`, `asymptotic_class_of_cost` | yes | comparison machinery on `SymbolicCost` |
| `cost_lens` / `symbolic_cost_fold` | yes | blocking wall over the Node kernel |
| `llvm_instruction_cost` | yes | **registered** owner `v2.lens.registry` `lens_owned_fn_llvm_instruction_cost` |
| `SizeExpr`, `CostExpr`, binder-carrying sum, cost max, extern channel | **candidate** (`v2.lens.cost.expr`) | absent on main until carrier merges; `CostEffect` replaces extern channel |

**`src/v1/complexity.dag`** lines 70–95 (promotion source, **not** port target):

```
SizeExpr  = SizeConst | SizeVar{name:String} | SizeLen{collection:String} | SizeAdd | SizeMax
CostExpr  = CostConst | CostAdd | CostMul | CostMax | CostSum{binder:String, upper:SizeExpr, body:CostExpr}
          | CostLog | CostExtern{name:String} | CostUnknown{reason:String}
Certainty = Proven | Conservative
```

**Five string-keyed identities** (parity note §6, all verified in v1): `SizeVar.name`, `SizeLen.collection`, `CostExtern.name`, `CostUnknown.reason`, `CostInternTable` keyed by `String`. None may survive the promotion.

## 2. Promoted type shapes (grounded identities)

Types land in **`v2.lens.cost`** (module file `cost/expr.dag` or a named section at the bottom of `cost.dag` — not a second cost module). `SizeVariable` is **reused**, not re-minted.

### 2.1 `SizeExpr`

```dag
type SizeExpr
  = SizeConst { value: v2.std.nat.Nat }
  | SizeOf { variable: SizeVariable }           // replaces SizeVar { name: String }
  | SizeLength { of: SizeVariable }             // replaces SizeLen { collection: String }
  | SizeAdd { left: SizeExpr, right: SizeExpr }
  | SizeMax { left: SizeExpr, right: SizeExpr }
```

**Grounding choices:**

- **`SizeOf` / `SizeLength`** both carry `SizeVariable { source: Node }`, the identity `v2.lens.cost` already owns. Collection length is not a separate string name; it is the size of a grounded value node (the same grain `loop_bound_measure` already uses).
- **INTERIM (C1):** size identity grounds on the existing `SizeVariable` carrier — not cited as the end state. Parity note §6 originally named `ValueIdentity`, which **does not exist in the tree** (corrected in `gunbc.plans.v2_complexity_capability_parity`). C1 uses `SizeVariable` with an explicit dissolution trigger: **`SizeLength.of` refines to `ValueIdentity` when `discrete-cost-derivation` `CostSubject.input` lands** — at which point `SizeVariable`-only collection length is deleted, not kept in parallel.
- **`DeclarationRef`** is reserved for *callable / effect* identity (`CostEffect`), not collection size. Mixing declaration paths with size expressions would re-open the string-nickname class under a typed wrapper.

### 2.2 `CostExpr`

```dag
type CostExpr
  = CostConst { value: v2.std.nat.Nat }
  | CostAdd { left: CostExpr, right: CostExpr }
  | CostMul { left: CostExpr, right: CostExpr }
  | CostMax { left: CostExpr, right: CostExpr }
  | CostSum { binder: SizeVariable, upper: SizeExpr, body: CostExpr }
  | CostLog { base: v2.std.nat.Nat, argument: SizeExpr }
  | CostEffect { operation: std.decl_ref.DeclarationRef }
  | CostRefused { cause: CostExprRefusalCause }
```

**Grounding choices:**

- **`CostSum.binder`** is `SizeVariable`, not `String`. The binder is the iteration witness identity; concrete evaluation (C3 / sibling valuation lane) binds `SizeVariable → Nat` in a valuation environment rather than doing string substitution.
- **`CostEffect`** replaces `CostExtern { name: String }`, joining `std.effects.ExecuteEffect` / discrete-cost `WorkAtom.ExecuteEffect` — one extern channel, keyed by `DeclarationRef`.
- **`CostRefused`** replaces `CostUnknown { reason: String }` with a **closed, countable** cause coproduct (§3). No free-text arm.

### 2.3 `Certainty` — deferred to C2 (constraint recorded now)

`Proven | Conservative` does **not** land in C1. C2's `ComplexitySummary` must reconcile certainty with the **existing** `DescentEvidence` and `Measured` bases per parity note §6 — **not** become a third confidence vocabulary. C2 acceptance bar, stated here so it cannot be re-litigated later:

- If descent is `Strict` / `NonIncreasing`, certainty derives `Proven` (or equivalent projection).
- If only measured evidence exists, certainty derives from the measured basis — not a fresh enum arm.
- `Conservative` is a **derived** widening over incomplete evidence, not a hand-stamped author field.
- Any C2 row that introduces `Certainty` without a total map from `DescentEvidence` + `Measured` → certainty **refuses at construction** (fail-closed).

## 3. Typed countable refusal shape (replaces `CostUnknown { reason: String }`)

`UnknownCost { diagnostic: Diagnostic }` on `SymbolicCost` already exists and stays for the **projection** layer. The richer authority carries an explicit cause enum so frequency is talliable (DESIGN §5 — free text never ranks):

```dag
type CostExprRefusalCause
  = UnboundedIteration { at: v2.std.diagnostic.Locus }
  | UngroundedLogArgument { argument: SizeExpr, at: v2.std.diagnostic.Locus }
  | InvalidLogBase { base: v2.std.nat.Nat, at: v2.std.diagnostic.Locus }
  | UnmodeledEffect { operation: DeclarationRef, at: v2.std.diagnostic.Locus }
  | UnresolvedSize { variable: SizeVariable, at: v2.std.diagnostic.Locus }
```

Projection: `CostRefused { cause }` → `UnknownCost { diagnostic: cost_expr_refusal_diagnostic(cause) }` where `cost_expr_refusal_diagnostic` is **total** over the coproduct and maps each variant to a `Diagnostic` whose `reason` symbol is the cause constructor (countable by variant, not by string contents).

**Relationship to existing `UnknownCost`:** containment, not synonymy (discrete-cost-derivation §7 naming constraint). `SymbolicCost` remains the asymptotic projection; `CostRefused` is the rich-layer honesty state before normalization collapses to lattice top.

## 4. `SymbolicCost` as normalized projection

### 4.1 Authority direction

```
CostExpr  ──normalize_cost_expr_to_symbolic──▶  SymbolicCost
SizeExpr  ──normalize_size_expr_to_bound────▶  SymbolicCost   (size-as-bound for products/sums)
```

- **`cost_lens` / `symbolic_cost_fold` are unchanged in C1.** They continue to produce `SymbolicCost` directly from the Node kernel. C1 adds the expression algebra and projection; it does **not** re-route the blocking fold through `CostExpr` (that is C2 when `ComplexitySummary.work` carries `CostExpr`).
- **`v2.lens.complexity`** continues to project `SymbolicCost → ComplexityBound` via `asymptotic_projection`. No second asymptotic path through `CostExpr`.

### 4.2 Normalization function (total on closed `CostExpr`)

Mirrors v1 `simplify_cost` / asymptotic reading, reusing existing `symbolic_sequential`, `symbolic_product`, `symbolic_max` from `v2.lens.cost`:

| `CostExpr` | `normalize_cost_expr_to_symbolic` |
| --- | --- |
| `CostConst` | `constant_cost` / `zero_cost` / `unit_cost` |
| `CostAdd` | `symbolic_sequential` |
| `CostMul` | `symbolic_product` |
| `CostMax` | `symbolic_max` |
| `CostSum { binder, upper, body }` | `symbolic_product(normalize_size(upper), normalize(body))` — iterative bound × per-iteration body |
| `CostLog { base: 0 or 1, … }` | `CostRefused { InvalidLogBase }` — degenerate log bases refuse (fail-closed) |
| `CostLog { argument: SizeOf v }` (valid base) | `LogCost { variable: v }` |
| `CostLog { argument: _ }` (compound) | `CostRefused { UngroundedLogArgument }` at projection — log of a non-atomic size is not normalized to a single `LogCost` without evaluation |
| `CostEffect` | `CostRefused { UnmodeledEffect }` until effect demand lands (C3) |
| `CostRefused` | `UnknownCost` via §3 diagnostic map |

**Key discriminating property:** `CostSum { binder, upper: SizeOf v, body: CostConst 1 }` normalizes to `LinearCost { variable: v }` — representable **exact summation** the current `SymbolicCost` fold cannot author (no binder-carrying sum). The fold may still *produce* `ProductCost(Linear, Constant)` for nested cardinality; C1's win is **authored** `CostSum` for loop bodies and interprocedural summaries (C4).

Helper (v1 parity): `cost_loop_expr(binder, iterations, body)` = `CostSum` unless `body` is zero.

## 5. Fate of existing dominance and composition machinery

**Unchanged and authoritative on `SymbolicCost`:**

- `symbolic_cost_dominates`, `asymptotic_class_of_cost`, `asymptotic_class_dominates`
- `symbolic_cost_lattice`, `symbolic_cost_sequential_monoid`
- `PolynomialDegree`, `PolyLogExponent`, `ExponentialBase` ladders
- `cost_fold_init` / `compose_child_cost` / `CostCompositionMode`

**Not duplicated on `CostExpr`.** Dominance comparisons for asymptotic questions run `normalize_cost_expr_to_symbolic` then reuse `symbolic_cost_dominates`. Concrete comparison (exact count) is C3 valuation — out of C1 scope.

**`llvm_instruction_cost`:** **no move in C1.** Stays registered under `v2.lens.cost` per parity note §6 registry fact. Re-home to cited per-target model is C5 (discrete-cost C4 calibration), requires `lens_owned_fn_llvm_instruction_cost` registry row update — explicitly out of C1 scope.

## 6. Regression baseline receipt (blocking wall) — **executed**

**Requirement:** every enrolled `lens_cost/*` witness green **before** carrier merge and **after**, by execution.

**Executed 2026-08-05** (pre-carrier) on worktree `quick-koi-569` at commit `c71592487`, via:

```
./target/release/claim_batch --source-root dag --source-root src/v2 \
  <14 entry groups, 22 witness functions — see receipt table>
```

**Result: 22/22 PASS, 0 FAIL** (pre-carrier authoring receipt).

**Acceptance (CI floor on this PR's merge head):** the enrolled `lens_cost/*` baseline roster and C1 carrier witnesses (`bounded_summation_test.dag` ×3, `expr_refusal_test.dag` ×4) are green on this PR's CI floor — the real discovery path for `*_test.dag` under `dag/` and `src/v2/`, not a worktree-local `claim_batch` receipt. Check the PR's **ci** check on the head this branch merges.

**Authoring history — mid-carrier local run (2026-08-06):** worktree `claim_batch` at `9c1532707` reported 22/22 + 7/7 pass before later carrier edits (NFR roster row, `CostRefused` routing); retained for traceability only — superseded by CI acceptance above.

| Entry | Function | Result |
| --- | --- | --- |
| `atom_zero_test.dag` | `atom_zero_claim_holds` | PASS |
| `loop_linear.dag` | `loop_linear_claim_holds` | PASS |
| `loop_unknown.dag` | `loop_unknown_claim_holds` | PASS |
| `loop_illegal_named.dag` | `loop_illegal_named_claim_holds` | PASS |
| `loop_iteration_floor_test.dag` | `loop_iteration_floor_projects_linear` | PASS |
| `loop_iteration_floor_test.dag` | `symbolic_product_absorbs_zero` | PASS |
| `disj_max.dag` | `disj_max_claim_holds` | PASS |
| `disj_alternative_floor_test.dag` | `disj_floor_projects_unit` | PASS |
| `disj_alternative_floor_test.dag` | `symbolic_max_absorbs_zero` | PASS |
| `nested_product.dag` | `nested_product_claim_holds` | PASS |
| `map_linear.dag` | `map_linear_claim_holds` | PASS |
| `arrow_body_cost_domain_excluded.dag` | `arrow_body_cost_domain_excluded_holds` | PASS |
| `copied_port_derivation_test.dag` | `derived_list_append_copied_port_index_is_zero` | PASS |
| `copied_port_derivation_test.dag` | `derived_list_snoc_copied_port_index_is_zero` | PASS |
| `copied_port_derivation_test.dag` | `derived_map_merge_copied_port_index_is_one` | PASS |
| `copied_port_derivation_test.dag` | `derived_registry_lookup_matches_hand_retired_indices` | PASS |
| `copied_port_derivation_test.dag` | `unpriceable_combiner_lookup_refuses_red_control` | PASS |
| `copied_port_derivation_test.dag` | `multi_linear_ambiguous_derivation_refuses_red_control` | PASS |
| `copied_port_derivation_test.dag` | `citation_retained_frontier_count_is_three` | PASS |
| `p9_llvm_instruction_cost_registry_owner.dag` | `p9_registry_owner_receipt_holds` | PASS |
| `src/v2/test/claim/complexity_gate/budget_roster_completeness_test.dag` | `complexity_budget_roster_unrated_declared_budget_semantic_red_holds` | PASS |
| `src/v2/test/claim/complexity_gate/budget_roster_completeness_test.dag` | `complexity_budget_roster_family_gate_holds` | PASS |

### Execution environment finding (not a tree defect)

`/usr/local/bin/gunbc run --claim-run` **fails on clean main** (fresh clone, untouched tree) with parse errors, first hits:

```
src/v2/lens/cost.dag:69:29: error: expected RBrace, found Dot
src/v2/extdeps/languages/llvm_ir.dag:71:36: error: expected RBrace, found Dot
src/v2/std/cardinality.dag:58:11: error: expected RBrace, found Dot
```

The installed PATH binary cannot parse current `.dag` syntax (qualified type paths in coproduct variants). **`claim_batch` built from the worktree** (`cargo build --release -p v1-compiler --bin claim_batch`) interprets the live tree and passes all 22 witnesses. This is an **install/PATH mismatch**, not a cost-lens regression and not explained by binary age (regeneration interprets `.dag` at run time per operator correction). C1 baseline command: **worktree `claim_batch`**, not `/usr/local/bin/gunbc run`.

**Carrier acceptance adds** (C1 discriminating controls, post-baseline):

1. `CostSum` inhabitance witness — `cost_loop_expr` produces `CostSum`, projects to `LinearCost`.
2. RED control — projection of `CostSum` with zero body must **not** claim linear (stays `zero_cost`).
3. `CostExprRefusalCause` arms stay in sync with `cost_expr_refusal_diagnostic` — exhaustive match is the enforcement (compile refusal on drift).
4. Invalid log bases — `CostLog` with base `Zero` or `Succ{Zero}` refuses `InvalidLogBase`; base `2` projects to `LogCost` (non-degeneracy control).
5. `CostEffect` — projects `UnmodeledEffect` refusal with typed diagnostic (witnessed in `expr_refusal_test.dag`).

## 7. Contradictions and open items found in the tree

| Item | Plan says | Tree says | Resolution |
| --- | --- | --- | --- |
| `ValueIdentity` for collection/function identity | parity note §6 | **No `type ValueIdentity` anywhere** | **Interim:** `SizeVariable`; dissolve when `CostSubject.input: ValueIdentity` lands |
| `CostInternTable` keyed by `String` | migrate in C4 | v1 only | C1 does not touch; C4 uses `DeclarationRef`-keyed memo |
| `SymbolicCost` already has `SumCost`/`ProductCost` | C1 adds binder sum | true | C1 adds **binder-carrying** `CostSum` on `CostExpr`; `SymbolicCost.SumCost` remains flat binary — no rename collision |
| `UnknownCost` vs `CostRefused` | countable cause | `UnknownCost` already uses `Diagnostic` | Rich layer gets `CostExprRefusalCause`; projection bridges to existing `UnknownCost` |
| C0 census / classification | inform disposition | `gunbc.v1_complexity_capability_census`, `gunbc.v1_complexity_decl_classification_roster` on main | C0(a–c) complete; carrier compatible |
| Carrier delivery | symbols on main | `v2.lens.cost.expr` candidate until merge | delivery when `normalize_cost_expr_to_symbolic` runs on main |

## 8. Carrier landing checklist (post-C0 signal)

1. Add §2 types + §3 `CostExprRefusalCause` to `v2.lens.cost` (`v2.lens.cost.expr`). **Candidate on branch; pending main delivery.**
2. Add §4 normalization functions; **do not** edit `symbolic_cost_fold` / `cost_lens`. **Candidate.**
3. Add witnesses §6 (`bounded_summation_test.dag`, `expr_refusal_test.dag`). **Candidate.**
4. Run 22/22 baseline roster green on CI at final head. **Green on this PR's CI floor** — baseline roster and C1 carrier witnesses execute via discovery on the merge head (check PR **ci** check). Local `claim_batch` receipts in §6 are authoring history only.
5. `llvm_instruction_cost` — **no edit**. **Unchanged.**
6. Design note on main (`docs/plans/c1-cost-expr-authority-design.md`). **Delivered.**

## Dissolution trigger

Delete this note when `v2.lens.cost` carries grounded `SizeExpr`/`CostExpr`, `normalize_cost_expr_to_symbolic` is live, C1 witnesses execute green, the baseline roster is green post-merge, and parity note C1 acceptance bar is met — at which point the carriers state the capability and this note retires.
