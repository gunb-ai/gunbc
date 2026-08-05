# C1 — SizeExpr / CostExpr authority in `v2.lens.cost`

> **Status: DRAFT for operator review (2026-08-05).** Design-note-first per lane sequencing: **no carrier lands from this note.** C0(b1) PR #7821 and C0(b2) PR #7840 remain open at the approval bar; C0(c) #7841 merged. Carrier authoring waits for operator signal after C0(b) lands.
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
| `SizeExpr`, `CostExpr`, binder-carrying sum, cost max, extern channel | **no** | |

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
- **`ValueIdentity`** is cited in parity note §6 but **does not exist as a type in the tree today**. Using `SizeVariable` is not a workaround — it is the live authority. When `ValueIdentity` lands (discrete-cost-derivation `CostSubject`), `SizeLength` may refine `of: ValueIdentity` without forking the expression algebra.
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

### 2.3 `Certainty` — deferred to C2

`Proven | Conservative` does **not** land in C1. C2's `ComplexitySummary` reconciles certainty with existing `DescentEvidence` and measured bases per parity note §6. C1 must not mint a third confidence vocabulary.

## 3. Typed countable refusal shape (replaces `CostUnknown { reason: String }`)

`UnknownCost { diagnostic: Diagnostic }` on `SymbolicCost` already exists and stays for the **projection** layer. The richer authority carries an explicit cause enum so frequency is talliable (DESIGN §5 — free text never ranks):

```dag
type CostExprRefusalCause
  = UnboundedIteration { at: v2.std.diagnostic.Locus }
  | UngroundedLogArgument { argument: SizeExpr, at: v2.std.diagnostic.Locus }
  | UnmodeledEffect { operation: DeclarationRef, at: v2.std.diagnostic.Locus }
  | UnresolvedSize { variable: SizeVariable, at: v2.std.diagnostic.Locus }
  | EffectModelAbsent { operation: DeclarationRef, at: v2.std.diagnostic.Locus }
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
| `CostLog { argument: SizeOf v }` | `LogCost { variable: v }` |
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

## 6. Regression baseline receipt (blocking wall)

**Requirement:** every enrolled `lens_cost/*` witness green **before** carrier merge and **after**, by execution.

**Roster** (13 modules under `src/v2/lens/cost/`, plus `complexity_gate/budget_roster_completeness_test.dag` as `BoundConsumerWitness`):

| Witness module | Discriminating claim |
| --- | --- |
| `atom_zero_test` | Atom → `zero_cost` |
| `loop_linear` | bounded loop → linear |
| `loop_unknown` | unbounded loop → `UnknownCost` |
| `loop_illegal_named` | illegal edge → fail-closed |
| `loop_iteration_floor_test` | iteration floor |
| `disj_max` | `Disj` → max branch |
| `disj_alternative_floor_test` | alternative floor |
| `nested_product` | nested cardinality → product |
| `map_linear` | cardinality → linear |
| `arrow_body_cost_domain_excluded` | domain excluded from fold (RED) |
| `copied_port_derivation_test` | derived port indices |
| `p9_llvm_instruction_cost_registry_owner` | registry ownership |
| `budget_roster_completeness_test` | blocking consumer |

**Execution status (2026-08-05, this worktree):** `gunbc run --claim-run` against the roster fails at compile with unresolved `Symbol` in `v2.std.grammar` / `target_model` — **environment defect**, not a cost-lens regression. Baseline must be re-run on a green host (CI floor / `claim_batch`) immediately before and after carrier lands; this note records the roster and command shape:

```
claim_batch --source-root dag --source-root src/v2 \
  --entry src/v2/lens/cost/atom_zero_test.dag --function atom_zero_claim_holds
# … repeat per roster row, or scan-dir src/v2/lens/cost
```

**Carrier acceptance adds** (C1 discriminating controls):

1. `CostSum` inhabitance witness — `cost_loop_expr` produces `CostSum`, projects to `LinearCost`.
2. RED control — projection of `CostSum` with zero body must **not** claim linear (stays `zero_cost`).
3. `CostRefused` cause variant count stable (planted duplicate variant → witness red).

## 7. Contradictions and open items found in the tree

| Item | Plan says | Tree says | Resolution |
| --- | --- | --- | --- |
| `ValueIdentity` for collection/function identity | §6 | **No `type ValueIdentity` anywhere** | Use `SizeVariable { source: Node }` now; refine when valuation carrier lands |
| `CostInternTable` keyed by `String` | migrate in C4 | v1 only | C1 does not touch; C4 uses `DeclarationRef`-keyed memo |
| `SymbolicCost` already has `SumCost`/`ProductCost` | C1 adds binder sum | true | C1 adds **binder-carrying** `CostSum` on `CostExpr`; `SymbolicCost.SumCost` remains flat binary — no rename collision |
| `UnknownCost` vs `CostRefused` | countable cause | `UnknownCost` already uses `Diagnostic` | Rich layer gets `CostExprRefusalCause`; projection bridges to existing `UnknownCost` |
| C0(b) census rows | inform disposition | PRs #7821/#7840 open | Carrier waits; census may add per-capability triggers — design compatible |
| Premature WIP carrier (PR #7879) | wait for C0 | landed as WIP commit | **Reverted** in `5c87cbb69`; PR returns to design-only until operator signal |

## 8. Carrier landing checklist (post-C0 signal)

1. Add §2 types + §3 `CostExprRefusalCause` to `v2.lens.cost` (submodule `cost/expr.dag`).
2. Add §4 normalization functions; **do not** edit `symbolic_cost_fold` / `cost_lens`.
3. Add witnesses §6; enroll in CI discovery if per-PR.
4. Run baseline roster green on CI.
5. `llvm_instruction_cost` — **no edit**.
6. Flip PR #7879 to ready only after (4).

## Dissolution trigger

Delete this note when `v2.lens.cost` carries grounded `SizeExpr`/`CostExpr`, `normalize_cost_expr_to_symbolic` is live, C1 witnesses execute green, the baseline roster is green post-merge, and parity note C1 acceptance bar is met — at which point the carriers state the capability and this note retires.
