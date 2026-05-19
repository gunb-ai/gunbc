# v4 retroactive dissolution audit — consolidated inventory

Per `docs/modeling-discipline.md` Practice 10 (the derived-operations
registry and dissolution-findings family) **and** the unified
Dissolution dispositions vocabulary in PR #3244 (Practice 4 head, used
by Practices 4 / 8 / 10). Rework-tracker PR #3240 task C1.

**Status — FINAL** (DRAFT→final flip 2026-05-18). PR #3244 (the
unified dissolution-disposition vocabulary used by Practices 4 / 8 /
10) squash-merged to `main` at commit `16191651a`. All symbol
assignments below are now expressed against the merged form — `🔴
dissolve-now` (substrate-exists, jumps queue) / `🟢 terminal` (nothing
to dissolve into) / `🟡 gated` (carrying gate kind `feature:` /
`consumer:`, the concrete named arrival including primitive + owning
task, and the dissolve-on-arrival obligation). The #3244 amendment
that **every 🟡 must bind a dissolution PLAN** — not merely a gate —
is supplied by Section 1's per-primitive substrate-PR rollup: a 🟡 is
valid only when it is rolled under a named P# whose substrate PR has
an owning task (the comment-graveyard case is P10, flagged
⛔ needs-concretization in § 1.1).

**This inventory is a dissolution PLAN, not a catalog.** Per operator
directive (#3244 commit `9b896f36d`, 2026-05-18): "I don't want these
comments to start piling up and never get dissolved." A 🟡 is transient
— a committed surface→dissolve loop with an exit to 🟢, never an
indefinite tracked comment. The plan in **Section 1** rolls every 🟡
up by the *missing primitive* it waits on, ranks substrate PRs by
🟡-count (highest first), and lists every finding each substrate PR
unblocks. The per-file catalog (Section 2) and the pre-existing-tracker
triage (Section 3) are the audit-traceability backing for the plan.

**This is also the requirements inventory for the rework-tracker #3240
S1 substrate track** (loyal-wren). Each substrate PR row in Section 1 is
a self-contained spec for one substrate-PR's `feature:` arrival, with
the count and concrete sites it unblocks; S1 consumes directly.

Scope on `main` at `ce0241039` (HEAD; baseline rolled forward from `88ae56d2a` to absorb merges of #3225, #3210, #3232, #3242 between initial sweep and final-ready) — **every `src/v4/**/*.dag`** (67 files
total) per still-hawk-102 scope-widening 2026-05-18: compiler/ + std/ +
extdeps/ + workflow/ + lens/ + bin/ + test/claim/. Sweep frame is
`main`, not in-flight branches — PR #3213 (workflow T-20 + T-24) is
HELD with its own dissolution pass; its helpers reach `main` only when
#3213 merges, and are covered by their own pass until then (no
double-counting).

**Dispositions** — the four #3244 vocabulary symbols:

- **🔴 dissolve-now** — substrate primitive exists; mechanical fix
  jumps the queue and lands in a follow-up PR. **Count:** **0 open**.
  § 1.0 records the two closed receipts: **R1 landed PR #3284**
  (`is_empty_conj_root`); **R2 landed PR #3245**
  (`terminator_is_catchswitch`).
- **🟡 gated** — substrate primitive does not exist yet; carries
  `feature:<primitive + owning task>` or `consumer:<named consumer>`
  + dissolve-on-arrival obligation. A 🟡 is a *committed*
  surface→dissolve loop, not a parking spot. **Count today: ~36
  bound to P1-P10 in § 1.1 (including the two #3225-merged findings
  added at final-ready — see § 1.1 P4 and P5 unblocks columns), plus
  ~23 pre-plan backlog (Section 3 VAGUE+INVALID-GATE) not yet
  rollable.**
- **🟢 terminal** — audited and not a dissolution finding.

**Pre-existing-tracker triage** (Section 3) adds two derived
dispositions:

- **STALE → 🔴** — the named arrival has *already landed*; the 🟡 is
  debt that should already be paid.
- **VAGUE** — gate present but not concrete (a class of arrivals
  rather than one named owner+task).
- **INVALID-GATE** — the gate's named arrival was *cancelled or
  reshaped by a design reversal*; the thing referenced will not arrive
  as named. Distinct from VAGUE (gate too loose) and STALE (gate
  opened).

**Status — FINAL.** PR #3244 merged at `16191651a` (2026-05-18). Marks
re-confirmed against the merged form; no clause changes from DRAFT
required this commit (marks already tracked the merged form per
still-hawk-102's vocabulary previews).

**C1's role is mark + flag + plan, not fix.** This inventory reports
the disposition of each entry and rolls them into a substrate-PR plan;
the actual re-gating of VAGUE / INVALID-GATE entries, the dissolve-now
PRs for 🔴 findings, and the substrate-PR work itself are **downstream
lane work** — each owned by the lane that owns the file or the S1
substrate track. C1 does *not* edit `DECISIONS.md` rows, *not* rewrite
in-file 🟡 blocks, and *not* land the dissolve-now fixes.

---

## Section 1 — Dissolution plan (rolled up by missing primitive)

**How this plan partitions the v4 dissolution surface:**

- **§ 1.0** — **🔴 dissolve-now receipts (jumped queue, no substrate gap).**
  Hand-rolled constructs whose substrate primitive *already exists* on
  `main`; when found, they land immediately ahead of P1. Both current
  entries have landed. (Per Practice 10 / #3244: 🔴 is a directive,
  never a standing state.)
- **§ 1.1** — **Ranked substrate-PR queue (P1-P10, 🟡 plan).** Every
  🟡 in this inventory waits on one of ten named arrivals; each row =
  one substrate PR, ranked by 🟡-count (highest first), with the
  finding list it unblocks. **Dissolution follow-ups dispatch
  immediately** on each substrate PR's landing — the surface→dissolve
  loop is what makes 🟡 transient.
- **§ 1.2** — 🟡 → 🟢 burn-down view.
- **Pre-plan concretization backlog** — Section 3's ~19 VAGUE + 4
  INVALID-GATE entries. **These are NOT in the P1-P10 plan**, because
  a VAGUE entry names no concrete primitive (cannot be rolled under
  any P#) and an INVALID-GATE entry's named arrival was cancelled by
  a design reversal (likewise unrollable). They are a **pre-plan
  backlog the burn-down lane drives first**: each gets re-concretized
  (then rolls under a P#) or is dissolved / re-dispositioned. They
  cannot enter the burn-down dependency DAG until concretized. P10
  has the same flag (named primitive but no owning task — see §
  1.1).

### 1.0 🔴 dissolve-now receipts (jumped queue, landed ahead of P1)

**R1 (empty-`Conj`-root duplicate predicates)** — **LANDED PR #3284**
(`src/v4/std/node.dag` `is_empty_conj_root`; disposition
`DECISIONS.md` §CP-1b item 12). **R2** below — **LANDED PR #3245** —
closed the LLVM discriminator predicate in the extdeps lane. Neither
needed upstream substrate work.

| # | finding | distinct fix | substrate available? |
|---|---|---|---|
| **R1** | WAS: `compiler/01_tokenize.dag lex_rules_node_is_conj_empty_root` **+** `compiler/02_parse.dag grammar_node_is_conj_empty_root` **+** `extdeps/languages/dag.dag dag_node_is_empty_conj_root` — three literal duplicates of the same empty-`Conj`-root shape query | **LANDED PR #3284:** `fn is_empty_conj_root` in `src/v4/std/node.dag`; all three sites import it (duplicate authority removed). **P5 / predicate disposition:** `src/v4/DECISIONS.md` §CP-1b item 12 — interim shared structural leg; forward dissolution under inventory §1.1 **P3** (T-6/T-7 walks) / **P5** (`fold_node`). | ✓ — pure composition over existing `std/node.dag` primitives (`Node`, `NodeKind`, `TypeNode`, `Conj`, `count`); all on main. |
| **R2** | WAS: `extdeps/languages/llvm_ir.dag terminator_is_catchswitch` (526) — naked `match … _ => false` discriminator | **LANDED PR #3245:** the `CatchSwitch` match is inlined into the sole consumer `block_well_formed`; the discriminator predicate is deleted. `block_well_formed` now owns the LangRef catchswitch-body invariant directly over the declared `Terminator` constructor. | ✓ — pure pattern match on `Terminator` (already declared in `llvm_ir.dag`); no new substrate. |

**R1** landed under the compiler/std lane (PR #3284). **R2** landed under
the `extdeps/languages/llvm_ir.dag` lane (PR #3245). Per "C1 marks +
flags + plans, does not fix," C1 did not author these fixes; it surfaces
the audit anchors.

### 1.1 Ranked substrate-PR queue

| # | substrate PR (`feature:` arrival) | owner / owning task | 🟡-count | unblocks |
|---|---|---|---|---|
| **P1** | `std/cardinality.dag` bounded-natural / refinement substrate | **substrate:** std / T-3 Wave-A2 — **`compile_to_dag` / extdeps import surface (flat v3 bootstrap, cardinality name resolution):** compiler / **T-32** (minimum never-hand-edited seed program; `src/v4/TASKS.md`) **+** receipt path per **T-30** interim mirror (`compile_to_dag` smoke harness / bootstrap collision notes in same §T-30) | **~19 live (+3 closed receipts)** | DECISIONS.md: `SL-3229-LLVM-OPS`, `SL-3229-FLOAT-NOMINAL` once re-gated under this canonical owner (`SL-3229-LLVM-WIDTH`, `SL-3229-PTX-DIM3`, `SL-3229-PTX-COST`, `SL-3229-VERILOG-COST` are **🟢 closed** on live carriers — see DECISIONS.md Part 6). **`Dim3`:** live `ptx.dag` uses `PositiveUpperBoundedNat`, closing the merge-base negative-axis and follow-on zero/unbounded-axis scaffold; PTX-version maxima are carried by the witness spine. In-file: the ~16 VAGUE prose blocks in `json.dag` / `yaml.dag` / `toml.dag` that concretize to this arrival (refinement-side family). **Receipt (PR #3310 / 2026-05-18 + follow-up + this P1 bundle):** `LlvmType` width payloads, `Dim3` positive bounded axes, `PtxCost` / `VerilogCost` non-negative axes. Remaining P1-family debt is **operand-relation refinement**, **float nominal/interchange re-gating**, and **import lowering / bootstrap extension** so cardinality refinements used in extdeps `.dag` files participate in `compile_to_dag`, not raw width/cost payload scaffolding alone. |
| **P2** | `std/collection.dag` Wave-A2: `List<T> where non_empty` refinement **plus** the List combinator algebra (`forall` / `count_where` / `unique` over `FreeMonoid<T>`) | std / T-3 Wave-A2 (coercion-design.md RQ-3) | **5 named + 29 sites** | Section 2: `std/node.dag` × 4 traverses (`all_edges_named`, `all_edges_positional`, `name_occurrences`, `all_names_distinct`). DECISIONS.md: `SL-3229-VERILOG-NONEMPTY` (one row, 29 verilog.dag back-pointer sites after P8). |
| **P3** | Compiler pipeline-stage substrate (lex-walk + parse-walk) | compiler / T-6, T-7 | **2 + ~7 in-file** | Section 2: `compiler/01_tokenize.dag tokenize`, `compiler/02_parse.dag parse`. In-file: the parser-side VAGUE prose blocks in `json.dag` / `yaml.dag` / `toml.dag` that concretize to T-6/T-7 (the operations-side family separate from P1). |
| **P4** | T-4 fact-bundle Phase-3 rework (post-D2-reversal model) | extdeps/languages / T-4 manager `vivid-carp-207` (5-feeder gate; keystone #3226 merged @`77b9e7d72`; 4 feeders open: T-3, T-29, T-30, T-25-core) | **4 + 1 row + 1 fn** | In-file: `typescript.dag` × 4 INVALID-GATE blocks (re-gate against this arrival, not pre-reversal D2). DECISIONS.md: `SL-3229-VERILOG-D3200` (if re-gated as `feature: T-4 fact-bundle Phase-3 rework` rather than `consumer:` form — see Section 3). Section 2: `extdeps/languages/dag.dag dag_language_model_wave1_void_canonical_symbols` (added in CP-1b #3225 — canonical_symbols set is a fact on DagLanguageModel/language-identity, not a hand-rolled function). |
| **P5** | `std/node.dag` `fold_node` — Node catamorphism (substrate-extension under T-1) | std / T-1 | **1 + 3 (03_resolve cascade only)** | **Substrate LANDED PR #3297:** `NodeFold<R>` + `fold_node` in `src/v4/std/node.dag`; `node_well_formed` consumes the shared `NodeFold<Bool>` algebra (burn-down closeout + #3297). **Cascade (open):** `compiler/03_resolve.dag merge_binding_self` (94, codex #3225) plus `add_module_named_exports` (99), `add_arrow_domain_named_params` (113), `add_bind_atom_binder` (140). These dissolve to `fold_node(root, ⟨binding-harvest algebra⟩)` only when a scoped harvest algebra lands without changing resolver scope semantics. |
| **P6** | `std/algebra.dag` / `std/nat.dag` `fold` / `cata` over `FreeMonoid<T>` and `Nat` (Wave-A2) | std / T-3 Wave-A2 | **2** | Section 2: `std/algebra.dag free_monoid_length`, `std/float.dag nat_compare`. (Sibling to P2's combinator algebra; could land in the same PR — kept separate because the underlying primitive is the catamorphism, distinct from `forall`/`count_where` which are derived from it.) |
| **P7** | `std/nat.dag nat_is_zero : Nat -> Bool` (Wave-A2) | std / T-3 Wave-A2 | **1** | Section 2: `std/float.dag float_finite_magnitude_zero`. |
| **P8** | `extdeps/languages/verilog.dag` bundled T-4 LanguageModel `constant_expression` sub-grammar | extdeps/languages / T-4 Verilog Phase-3 | **0** | Landed: `VectorRange` carries `ConstantExpression` endpoints; DECISIONS.md row `SL-3229-VERILOG-VECTOR-RANGE` is closed. |
| **P9** | `lens/cost.dag` cost-of-instruction model fact / lens | lens / T-12 | **0** | **LANDED:** `src/v4/lens/cost.dag` now owns `llvm_instruction_cost` (the 22-arm `LlvmInstruction -> Int` cost table); `extdeps/languages/llvm_ir.dag` owns only the LLVM instruction shape. |
| **P10 ⛔ needs-concretization** | Constrained generic parameters / inhabitance-bound syntax (`<M> where M : CommutativeMonoid<_>`) | substrate extension; **no owning task yet** | **1** | DECISIONS.md: `SL-3229-INTEGER-GROUP-COMPLETION` (`GroupCompletion<M>`). **P10 does NOT enter the burn-down DAG as a normal upstream node until concretized** — under #3244 a 🟡 whose substrate primitive has no committed PR/task is not a valid 🟡 (the comment-graveyard case). The single finding under P10 (`SL-3229-INTEGER-GROUP-COMPLETION`) is reclassified VAGUE in Section 3.1 until an owning T-# is assigned. Action owner: substrate / operator-or-S1 assignment. |

Plus property-projection model facts (Practice 10 row 7) that do not
roll up into a shared substrate PR — each is a per-type fact-bundle
extension under its own owner; named in Section 2 but not ranked here
because they don't amortize across multiple findings. The two such
findings are `std/node.dag connective_edge_discipline` (T-1) and
`extdeps/languages/llvm_ir.dag feature_disposition` (T-4 fact-bundle —
rolls into **P4** as a separate fact within that rework). The third —
`llvm_ir.dag block_well_formed` — is not really a substrate gap; it
cascaded off the `terminator_is_catchswitch` dissolve-now and closed in
PR #3245 without waiting on substrate.

### 1.2 🟡 → 🟢 burn-down

Total 🟡 in the v4 substrate today (Section 2 + Section 3 VALID-🟡 +
INVALID-GATE-once-re-gated):

| primitive PR | 🟡 today | landing event | 🟡 after landing |
|---|---|---|---|
| (baseline) | **~36** | — | — |
| P1 lands | ~19 | `std/cardinality.dag` refinement (+ import lowering / bootstrap for extdeps cardinality imports) | ~16 |
| P2 lands | 5 named (+ 29 verilog sites converge in one sweep) | `std/collection.dag` Wave-A2 | ~11 named |
| P3 lands | 2 named (+ ~7 in-file) | T-6 + T-7 pipeline substrate | ~9 named |
| P4 lands | 4 + 1 row + 1 fn (`dag.dag canonical_symbols` #3225) | T-4 fact-bundle Phase-3 | ~3 named |
| P5 cascade lands | 1 named (+ 3 walker sites in `03_resolve.dag` #3225) | scoped binding-harvest `fold_node` algebra (substrate: **#3297**) | ~1 |
| P6 lands | 2 | FreeMonoid/Nat catamorphism | ~0 |
| P7 lands | 1 | `nat_is_zero` | ~0 |
| P9 landed | 0 | `lens/cost.dag` T-12 | ~0 |
| P8 landed | 0 | Verilog constant_expression | ~0 |
| P10 lands | 1 (after concretization) | constrained-generics syntax | **0** |

(Residual column is illustrative — counts roll up imperfectly because
some entries are counted with cascades — e.g. P5's `merge_binding_self`
implicates its 3 named-harvest walker callers — and Section 3 VAGUE
entries that concretize to a P# only enter their column on re-gate.
The cumulative endpoint after all P1-P10 land is 0; the column shows
qualitative trajectory, not strict arithmetic.)

(R1 **landed** PR #3284; R2 **landed** PR #3245. These are not counted in
the 🟡 burn-down because they dissolved outside the substrate-gap queue.)

Caveats:

1. **"🟡 today" counts only entries already bound to a P# in the
   plan.** Section 2's fresh findings + Section 3's 3 VALID-🟡 are
   counted; **Section 3's ~19 VAGUE + 4 INVALID-GATE are NOT** —
   they are pre-plan backlog (no concrete primitive to roll under
   yet). The burn-down lane drives the backlog first; once a VAGUE/
   INVALID entry is re-concretized to a `feature:` arrival that
   matches an existing P#, it rolls in and the corresponding row's
   🟡-count grows. The numbers above project the post-concretization
   landing impact under the assumption that the pre-plan backlog
   concretizes uniformly to the canonical owners named in Section 3
   (e.g. refinement-side VAGUE → P1; parser-side VAGUE → P3).
2. **P10 lands** requires the owning task be assigned first; the
   "🟡 → 🟢 sweep for P10" only fires after concretization. The
   `⛔ needs-concretization` flag on P10 in § 1.1 is structurally
   blocking that row's DAG entry.

**P1 is the headline.** Landing the remaining `std/cardinality.dag`
refinement substrate dissolves more than half of the v4 substrate's
outstanding 🟡 debt in a single sweep — by 🟡-count it dominates every other
substrate PR by 4× or more. The S1 substrate track should prioritize
P1 ahead of P2-P10. R2 already jumped that queue and landed because it
needed no absent substrate.

---

## Section 2 — Per-file sweep findings (the catalog backing Section 1)

### 2.1 Lane-wide

- **Carrier dissolution** — 🟢 lane-wide. No local
  `... { value: T } | ... { diagnostic: Diagnostic }` clone of
  `Outcome<T>` outside `std/diagnostic.dag`; every consumer imports the
  std carrier.
- **Emit/template dissolution** — 🟢 lane-wide. No literal
  `template: "..."` field in any v4 `.dag`; emit substrate is
  grammar-as-data (`extdeps/languages/*.dag` LanguageModel +
  `compiler/05_emit.dag`).

### 2.2 `src/v4/std/`

**`std/node.dag`**
- `node_well_formed` (122) — ✓ **walker dissolved** —
  `fold_node` landed in `std/node.dag` and this function now consumes the
  shared `NodeFold<Bool>` algebra instead of hand-rolling recursive descent.
- `all_edges_named` (80), `all_edges_positional` (86) — 🟡 **traverse** —
  `feature: forall over FreeMonoid<T> in std/collection.dag (Wave-A2)`.
  `fold` body is `acc && pred(e)`; dissolves to
  `forall(children, pred)` on arrival.
- `name_occurrences` (91), `all_names_distinct` (99) — 🟡 **traverse** —
  `feature: count_where + unique over FreeMonoid<T> in std/collection.dag (Wave-A2)`.
  Dissolves to `count_where(children, pred)` / `unique(children)` on
  arrival.
- `connective_edge_discipline` (57) — 🟡 **predicate** (property
  projection, registry row 7) —
  `feature: per-Connective discipline fact on std/node.dag Connective (substrate-extension under T-1)`.
  Six-arm `match`-to-derive of an `EdgeDiscipline` per `Connective`;
  the discipline IS a fact on `Connective`. Dissolves to
  `c.discipline` on arrival (paired-construction at the type).
- `edge_is_named` (67), `edge_is_positional` (73) — 🟢 — naked
  constructor inspection of `EdgeLabel`; reading the model fact rather
  than deriving anything (a consumer's `match e.label` is the canonical
  form).
- `edges_conform` (107) — 🟢 — each `EdgeDiscipline` arm does
  structurally distinct work (count check / labelling rule / position
  check); irregularity escape hatch.
- `node_locally_well_formed` (115) — 🟢 — composes the above with a
  `NodeKind` constructor split that does distinct work per arm.

**`std/algebra.dag`**
- `free_monoid_length` (84) — 🟡 **walker** —
  `feature: fold over FreeMonoid<T> in std/algebra.dag (the canonical catamorphism — Wave-A2)`.
  Hand-rolled recursion that IS the canonical catamorphism on the
  type; dissolves to `fold(xs, 0, |acc, _| acc + 1)` on arrival.
- `free_monoid_is_empty` (76) — 🟢 — reading the constructor of the
  coproduct that *defines* the type (boundary primitive).

**`std/nat.dag`**
- `nat_add` (15), `nat_mul` (21) — 🟢 — Peano-recursive primitives that
  *define* arithmetic on `Nat`. The catamorphism IS `nat_add`; no
  upstream substrate they shadow.

**`std/float.dag`**
- `nat_compare` (52) — 🟡 **walker** —
  `feature: fold / cata over Nat in std/nat.dag (same Wave-A2 obligation as FreeMonoid fold)`.
  Structural recursion on `Nat`; placement quirk (belongs on `Nat`).
- `float_finite_magnitude_zero` (109) — 🟡 **predicate** —
  `feature: nat_is_zero : Nat -> Bool in std/nat.dag (Wave-A2 nat primitives)`.
  Nested `Nat` match-to-`Bool`; dissolves to
  `nat_is_zero(e) && nat_is_zero(f)` on arrival.
- `float_body_is_nan` (102) — 🟢 — naked constructor inspection of
  `FloatSpecial` over `FloatBody`.
- `ordering_invert` (70) — 🟢 — symmetry of the 3-element `Ordering`.
- `sign_rank_lex` (64), `float_special_rank` (92) — 🟢 — encode the
  IEEE-754 totalizer canonical ordering of sign / specials; the
  per-constructor rank IS the model fact (irregularity escape hatch).
- `float_finite_unsigned_field_order` (77), `float_body_compare_*`
  (118, 161) — 🟢 — IEEE-754-specific lexicographic comparison ladders;
  each arm does structurally distinct semantic work (genuinely irregular).

**`std/integer.dag`**
- `int_add` (59), `int_negate` (63), `int_mul` (67), `int_compare` (71),
  `int_is_zero` (83), `int_div` (124), `int_mod` (134),
  `integer_divide_by_zero_diagnostic` (109),
  `integer_modulo_by_zero_diagnostic` (117) — 🟢 — primitives over the
  kernel-ambient `Int` / pure data constructors.

**`std/text.dag`**
- `string_is_empty` (16) — 🟢 — thin delegation to
  `free_monoid_is_empty` (canonical primitive on the type it aliases).

**`std/logic.dag`**
- `bool_boolean_algebra` (15) instance lambdas — 🟢 — primitive Boolean
  operations on the 2-arm `Bool` coproduct.

**`std/diagnostic.dag`**, **`std/witness.dag`**, **`std/collection.dag`**,
**`std/cardinality.dag`**, **`std/machine.dag`**,
**`std/verification.dag`**, **`std/report.dag`** — schema only / scaffold.
🟢.

### 2.3 `src/v4/compiler/`

**`compiler/01_tokenize.dag`**
- Empty-`Conj`-root shape query — 🟢 **R1 landed (PR #3284)** — shared
  `v4.std.node.is_empty_conj_root` (was `lex_rules_node_is_conj_empty_root`;
  duplicate authority removed per inventory §1.0 R1). Disposition receipt:
  `DECISIONS.md` §CP-1b item 12.
- `tokenize` (83) — 🟡 — `feature: lexical-walk substrate (T-6 — TASKS.md)`.
  Three-arm Wave-1 scaffold cascade; full lexical walk unrealized.

**`compiler/02_parse.dag`**
- Empty-`Conj`-root shape query — 🟢 **R1 landed (PR #3284)** — same shared
  `is_empty_conj_root` (was `grammar_node_is_conj_empty_root`).
- `parse` (80) — 🟡 — `feature: parse-walk substrate (T-7 — TASKS.md)`.
  Mirror of `tokenize` scaffold.

**`compiler/03_resolve.dag`** — filled in CP-1b (#3225, merged
`b83d8ed6c` 2026-05-18) with 23 `fn` bodies + 6 `type` declarations.
**This file has moved from scaffold to filled between initial sweep
and final-ready; the full re-sweep of all 23 fns is a near-future
follow-up.** The two functions surfaced as #3225's un-cleared codex
REQUEST_CHANGES findings (threads `3255338394` / `3255338395`) are
captured here as the audit anchor for the rest:

- `merge_binding_self` (94) — 🟡 **walker** (the "sym↦sym module harvest"
  codex finding). The leaf helper is `map_insert(m, sym, sym)`, but it
  is the symbolic-merge primitive used inside three named-harvest
  walkers (`add_module_named_exports` at 99, `add_arrow_domain_named_params`
  at 113, `add_bind_atom_binder` at 140) — all three are `fold` over
  `Node.children` doing constructor-discriminated recursion.
  **Substrate dependency satisfied (PR #3297):** `fold_node` exists in
  `std/node.dag`. These four sites are the **P5 cascade** — they still
  need a scoped binding-harvest `NodeFold<Map<Symbol, Symbol>>` (or
  equivalent) that preserves resolver scope semantics; not a repeat
  T-1 substrate-extension PR. Dissolves to `fold_node(root, ⟨algebra⟩)`
  over that algebra; eliminates the three harvest walkers plus the
  implicit recursion in `merge_binding_self`'s callers.

- The other 22 fns in 03_resolve.dag (`empty_namespace`,
  `empty_canonical_symbol_set`, `build_program_namespace`,
  `scope_from_namespace`, `scope_root_module`, `lookup_chain`,
  `malformed_tree_diagnostic`, `unbound_symbol_diagnostic`,
  `namespace_has_canonical_symbol`, `canonical_atom`, `resolve_atom`,
  `resolve_node`, `resolve_children_homogeneous_scope`,
  `resolve_edges_first_outer_then_inner`, `resolve_bind_binder_target`,
  `resolve_bind_edges`, `resolve_bind_node`, `resolve`,
  `resolve_with_namespace`, …) are **not individually triaged here**
  — the full sweep is a **named follow-up for the burn-down lane**
  (`jolly-ibex-599`), not a C1 expansion. C1 is one-shot; the 21-fn
  re-pass is dispatched by the burn-down lane as a standing
  re-sweep work-item alongside the per-primitive DAG. Many entries
  will likely roll under P5
  (`fold_node`) as variations of the harvest-walker pattern; a few
  may surface new sub-classes that the burn-down lane folds into its
  DAG.

**`compiler/00_compile.dag`**, **`03_normalize.dag`**,
**`04_infer.dag`**, **`05_emit.dag`**, **`05_eval.dag`** — scaffolds
(no fns). 🟢.

### 2.4 `src/v4/extdeps/`

**`extdeps/languages/llvm_ir.dag`**
- `terminator_is_catchswitch` (526) — ✅ **landed** PR #3245 —
  deleted. The naked discriminator predicate dissolved into its sole
  consumer.
- `block_well_formed` (533) — 🟢 **predicate** —
  `CatchSwitch` is matched directly inside the consumer, where the
  LangRef catchswitch-body invariant is enforced as `count(b.body) == 0`.
  No parallel discriminator, no separate substrate gap.
- `feature_disposition` (565) — 🟡 **predicate** (property projection) —
  `feature: per-FidelityFeature disposition fact on FidelityFeature itself (LLVM-substrate fact-bundle rework — T-4 fact-bundle program)`.
  12-arm `FidelityFeature -> FidelityDisposition` map; the disposition
  IS a fact per feature.
- `llvm_instruction_cost` — 🟢 **moved to cost-lens authority** —
  `src/v4/lens/cost.dag` owns the 22-arm `LlvmInstruction -> Int`
  table as the P9 cost-of-instruction model fact; this file owns only
  the LLVM instruction data shape.
- `block_successors` (505), `unwind_successors` (498) — 🟢 — each arm
  reads its own constructor fields; constructor-driven projection, not
  a `match`-to-derive of a pre-existing fact.

**`extdeps/languages/dag.dag`**
- `dag_wave1_e0_void_lex`, `dag_wave1_g0_void_grammar`,
  `dag_language_model_wave1_void` — 🟢 — pure data constructors.
- `dag_language_model_wave1_void_canonical_symbols` (62, added in
  CP-1b #3225) — 🟡 **predicate** (property projection, registry
  row 7) — the "four C3 Atom identities" codex finding (thread
  `3255338394`). Returns a `Set<Symbol>` whose `member: fn(sym)` is a
  four-way disjunction over `dag_c3_surface_sugar_{service,fn,type,operation}`.
  The canonical-symbol set IS a fact on the `DagLanguageModel` (or
  on `dag_lm_identity_native_dag`), not a function that
  reverse-engineers them from a literal disjunction.
  `feature: per-LanguageModel canonical_symbols : Set<Symbol> model
  fact carried on DagLanguageModel (T-4 fact-bundle Phase-3 — same
  family as feature_disposition on FidelityFeature)` —
  rolls under **P4**. (`dag_node_is_empty_conj_root` **retired** — R1
  landed PR #3284: `dag_language_model_is_wave1_void_shape` imports
  `v4.std.node.is_empty_conj_root`; see `DECISIONS.md` §CP-1b item 12.)
- `dag_language_model_is_wave1_void_shape` (83),
  `dag_language_model_empty_canonical_symbol_set` (89),
  `dag_language_model_canonical_symbols` (98) — 🟢 — structurally
  distinct constructor inspection / pure data construction.

**Other language files** (`rust`, `go`, `python`, `cpp`, `verilog`,
`typescript`, `ptx`, `machine_code`, `lean`) — zero `fn` bodies. 🟢
for all five dissolution classes (carrier and emit-template covered
lane-wide).

**`extdeps/formats/*.dag`** (`spice`, `toml`, `yaml`, `json`, `csv`,
`openapi`, `json_schema`) — zero `fn` bodies. 🟢.

**`extdeps/frameworks/react.dag`**, **`extdeps/coordination.dag`**,
**`extdeps/file_system.dag`**, **`extdeps/process.dag`** — zero `fn`
bodies. 🟢.

### 2.5 `src/v4/workflow/`

Sweep frame `main` @ `ce0241039`. (PR #3213 fills both files with
helper logic; that work is on the #3213 branch only, not in this sweep
— covered by #3213's own dissolution pass.)

**`workflow/bootstrap.dag`** — scaffold on `main` (84 lines, all
header prose + `module v4.workflow.bootstrap` declaration; zero `type`,
zero `data`, zero `fn`). 🟢 across all five finding classes.
**`workflow/ci.dag`** — scaffold on `main` (43 lines; same shape).
🟢 across all five finding classes.

Carrier and emit-template covered by lane-wide 🟢 (2.1).

### 2.6 `src/v4/lens/`

11 files (`affected_set.dag`, `application.dag`, `complexity.dag`,
`cost.dag`, `coverage.dag`, `effect.dag`, `idempotency.dag`,
`ownership.dag`, `parallelism.dag`, `synthesis.dag`, `testgen.dag`)
— **every one a scaffold on `main`** (each carries a
`Status: scaffold — fill per TASKS.md T-##` line, header prose, and a
`module v4.lens.<name>` declaration). **Zero `type`, zero `data`, zero
`fn` bodies.** 🟢 across all five finding classes.

`parallelism.dag:83` carries a prose mention of `fold(xs,…)` inside a
comment ("Fold parallelizability is an ALGEBRA fact"); not an
implementation. 🟢.

When each lens fills (T-12 cost/complexity, T-13 effect/idempotency/
ownership/parallelism, T-17 synthesis, T-18 coverage, T-19 testgen,
T-21 affected_set, T-23 application), the dissolution audit applies
per the same Section 1 plan — the lenses will likely be major
consumers of the P1 (cardinality refinement) and P5 (`fold_node`)
substrate PRs.

### 2.7 `src/v4/bin/`

**`bin/main.dag`** — 21-line scaffold (T-15). Zero `type`/`data`/`fn`.
🟢.

### 2.8 `src/v4/test/claim/`

9 files: 2 `manual/` (`connective_anchors.dag`, `nat_law_anchors.dag`)
+ 1 `boundary/` (`english_ingest_fail_closed.dag`) + 6 `impossible_bug/`
(`idempotency_contract.dag`, `nested_optional_flatten.dag`,
`suboptimal_complexity.dag`, `transport_type_drift.dag`,
`unenumerated_effects.dag`, `unhandled_diagnostic_paths.dag`).

The `manual/*` pair carries `data` declarations only — pure
`TestClaim` literal values (e.g.
`data claim_nat_add_left_identity: TestClaim = TestClaim { … }`). Zero
`fn` bodies. By construction TestClaim instances cannot host
dissolution findings — they are the data that *gets fed into* the
compiler/lens stages whose dissolutions live elsewhere. 🟢.

All `boundary/*` and `impossible_bug/*` files are scaffolds (each
carries `Status: scaffold — fill per TASKS.md T-##`; zero `type` /
`data` / `fn`). 🟢.

---

## Section 3 — Pre-existing-tracker triage (the audit backing Section 1)

### 3.1 `DECISIONS.md` Part 6 SL-3229-* triggers

Each row reports: the existing tracker's named arrival → triage result
(VALID / STALE / VAGUE) → re-expression in #3244 vocabulary (if VALID
in shape, only the *form* changes; if VAGUE, the gate needs concretizing).

**`SL-3229-INTEGER-GROUP-COMPLETION`** — `GroupCompletion<M>`
constrained-inhabitance gap. Named arrival: "v4 lands constrained
generic parameters / inhabitance bounds." Verified against
`std/nat.dag` / `std/algebra.dag` on main: no
`<M> where M : CommutativeMonoid<_>` syntax landed.
- **Triage: VAGUE** (per still-hawk-102 tightened bar 2026-05-18 — a
  final VALID-🟡 requires a concrete gate AND binding to a named
  primitive + owning-task substrate PR in Section 1. This entry maps
  to P10, but P10 has no owning task; the substrate PR is not fully
  named).
- **What's missing for VALID:** owning T-# assignment for the
  constrained-generics substrate extension. Until assigned, this is a
  feature-gated 🟡 without a pre-committed obligation under #3244.
- **#3244 re-expression (after concretizing):** `🟡 gated — feature: constrained generic parameters / inhabitance-bound syntax — owning task <T-#>` (to be assigned).
- **Owning lane (concretize action — out of C1 scope):** substrate
  extension; needs operator / S1-track assignment of an owning T-#.

**`SL-3229-LLVM-WIDTH`** — raw-`Int` width payload scaffold
(`LlvmType` family). Named arrival: "std/cardinality.dag refinement
substrate lands (T-3)." Verified `std/cardinality.dag` on main:
`NonZeroNat`, `NatLeWitness`, `UpperBoundedNat`, `DescentEvidence`,
`Multiplicity`, … — **bounded-natural / strict-positivity refinement
carriers are consumable**; live `llvm_ir.dag` uses `NonZeroNat` for
`IntegerType.bits` and `VectorType.count` per DECISIONS.md disposition.
- **Triage: CLOSED** (merge-base raw-`Int` width payload class; LangRef
tightness beyond strict positivity stays producer-side per DECISIONS).
- **#3244 re-expression:** n/a — cite-sites on `LlvmType` are 🟢 terminal
coproduct rows; ledger remains as audit history.

**`SL-3229-LLVM-OPS`** — operation-specific operand constraints. Named
arrival: same as `SL-3229-LLVM-WIDTH` (cardinality refinement / operand
relation refinement).
- **Triage: VALID.**
- **#3244 re-expression:** `🟡 gated — feature: refined typed-value carrier in std/cardinality.dag (T-3 Wave-A2)`.

**`SL-3229-PTX-DIM3`** — `Dim3` kernel-ambient `Int` axis scaffold.
Named arrival: same cardinality-refinement family (T-3).
- **Triage: CLOSED.**
- **Closure receipt:** live `ptx.dag` uses `PositiveUpperBoundedNat` for
  `Dim3.x` / `y` / `z`. The negative-axis, zero-axis, and unbounded-axis
  illegal states are closed by the positive bounded witness; PTX-version
  maxima are witness facts, not side prose.

**`SL-3229-PTX-COST`** — raw-`Int` PTX cost axes (`PtxCost`). Named
arrival: cardinality refinement (T-3).
- **Triage: CLOSED.**
- **Closure receipt:** live `ptx.dag` uses `Nat` for `PtxCost` axes, so
  negative cost is no longer representable.

**`SL-3229-VERILOG-NONEMPTY`** — shared `List<T>` spec-non-empty
Wave-A2 deferral (29 sites after P8). Named arrival: `std/collection.dag`
Wave-A2 `List<T> where non_empty`. Verified `std/collection.dag` on
main: `type List<T> = FreeMonoid<T>` alias only — no `where non_empty`
refinement landed.
- **Triage: VALID.**
- **#3244 re-expression:** `🟡 gated — feature: List<T> where non_empty refinement in std/collection.dag (Wave-A2 — coercion-design.md RQ-3)`.

**`SL-3229-VERILOG-D3200`** — #3200 consumer-independent 🟡 coproducts
(first-consumer decomposition; 5 carriers: `NonTriregNetKind`,
`VariableDeclaration`, `OutputPortAnsiVariableTypeKind`,
`ParameterTypeKind`, `PrimitiveGateKind`). Named arrival:
"first meaning-consumer owes the structural decomposition" — D2 /
synthesis / elaboration consumers (a class, not a single named
consumer).
- **Triage: VAGUE.**
- **Why:** #3244 mandates `consumer:<named consumer>` (a single concrete
  name a reader and audit can check) — not a class of three potential
  consumers. Compounded by D2-reversal: the named consumer ("D2") was
  reshaped to the fact-bundle model — the original-form arrival no
  longer exists. Per the operator's D2-reversal directive plus
  #3244's "vague gate blocks merge", this entry must be
  re-concretized — either name *one* consumer with an owning task (and
  the other 4 carriers re-cite to that owner), or re-gate as a
  `feature:` (e.g. T-4 fact-bundle Phase-3 rework landing) and update
  the 5 in-file cite sites in `verilog.dag` to match.
- **Action queued (not in this PR):** a follow-up edit to
  `DECISIONS.md` Part 6 + the 5 in-file cite sites in `verilog.dag`.

**`SL-3229-VERILOG-VECTOR-RANGE`** — `VectorRange` lexeme-pair bridge.
Named arrival: "bundled T-4 LanguageModel `constant_expression`
productions." Verified: PR #3272 landed the Verilog
`constant_expression` carrier family in `extdeps/languages/verilog.dag`
and rewired `VectorRange` endpoints to `ConstantExpression`.
- **Triage: CLOSED.**
- **Closure receipt:** `VectorRange` no longer carries `msb_lexeme` /
  `lsb_lexeme`; the Part 6 row remains as audit history, not live 🟡 debt.

**`SL-3229-VERILOG-COST`** — raw-`Int` Verilog cost axes
(`VerilogCost`). Named arrival: cardinality refinement (T-3).
- **Triage: CLOSED.**
- **Closure receipt:** live `verilog.dag` uses `Nat` for `VerilogCost`
  axes, so negative cost is no longer representable.

**`SL-3229-FLOAT-NOMINAL`** — nominal width / interchange list-length
scaffold. Named arrival: "bounded refinement substrate in
`std/machine.dag` notes / Wave-A2" — straddles two named owners.
- **Triage: VAGUE.**
- **Why:** #3244 mandates one concrete named arrival. The current text
  cross-references `std/machine.dag`'s own scaffold block but the
  refinement substrate itself lives in `std/cardinality.dag`. Same
  family as the four cost/width gates; should re-express to the same
  single owner.
- **Action queued:** re-state arrival as
  `feature: bounded-natural refinement in std/cardinality.dag (T-3 Wave-A2)` — collapses to one canonical gate for the whole
  width/cost/nominal-field family (6 entries: LLVM-WIDTH, LLVM-OPS,
  PTX-DIM3, PTX-COST, VERILOG-COST, FLOAT-NOMINAL).

### 3.2 In-file 🟡 cite-sites

Result of `grep -n "🟡" src/v4/**/*.dag` (excluding `verification.dag:128`,
which is descriptive prose about the 🟢/🟡/🔴 convention itself).

**`extdeps/languages/verilog.dag` × 5 cite-sites** (lines 24, 174, 207,
264, 473) — all read `// 🟡 coproduct dissolution — DECISIONS.md Part 6 ·
SL-3229-VERILOG-D3200.`
- **Triage: inherits VAGUE from `SL-3229-VERILOG-D3200` (Section 3.1).**
- **Action queued:** when the SL-3229-VERILOG-D3200 entry is
  re-concretized, all 5 cite-sites update to the new gate text.

**`extdeps/languages/llvm_ir.dag` (`LlvmType` row)** — merge-base had a
🟡 `SL-3229-LLVM-WIDTH` cite immediately above `type LlvmType`; **HEAD**
carries `// 🟢 … CP-3229-GREEN-TERMINAL` on `LlvmType` with `NonZeroNat`
payloads (see §3.1 **`SL-3229-LLVM-WIDTH` CLOSED**).
- **Triage: STALE** (inventory line-number snapshot; no live 🟡 cite on
  `LlvmType` at HEAD).

**`extdeps/formats/json.dag` × 3 in-file blocks** (lines 47, 143, 236)
— pre-#3234 prose-form `// 🟡 TRACKED-SCAFFOLD` blocks (not
on-coproduct one-liners). Block at 47 cites "the three bridge
properties exactly the diagnostic.dag Locus(🟢)/ByteRange(🟡)
precedent"; block at 143 cites "the numeric substrate"; block at 236
cites "the operations ride substrate that is scaffold today."
- **Triage: VAGUE.**
- **Why:** named arrival is a class ("the numeric substrate" /
  "the operations ride substrate that is scaffold today") rather than
  one concrete owner+task. The numeric-substrate gate is concretizable
  to the same canonical
  `feature: std/cardinality.dag refinement (T-3 Wave-A2)` arrival; the
  operations-side gate concretizes to `feature: T-6/T-7 parse/emit
  pipeline-stage substrate`.
- **Action queued:** re-state each of the three in-file blocks against
  one concrete arrival.

**`extdeps/formats/yaml.dag` × 6 in-file blocks** (lines 44, 85, 122,
163, 245, 279) — pre-#3234 prose-form 🟡 blocks ("named owner +
dissolution trigger, NEVER improvised"). Mix of parser-side
(parse body's job) and refinement-substrate gates.
- **Triage: mostly VALID-in-intent / VAGUE-in-form.**
- **Why:** each cite-site names a class of arrival ("the parse body's
  job", "deferred parser", "canonical-key / refined-lexeme substrate")
  but not a concrete owner+task. The arrivals concretize to
  `feature: T-6/T-7 parse pipeline-stage substrate` (parser-side gates)
  and `feature: std/cardinality.dag refinement (T-3 Wave-A2)`
  (refinement gates).
- **Action queued:** re-state each block against one concrete arrival.

**`extdeps/formats/toml.dag` × 7 in-file blocks** (lines 41, 79, 86,
96, 126, 144, 181, 223, 315) — same pre-#3234 prose form as yaml.dag.
"DEFERRED (🟡, named owner + dissolution trigger)", "deferred parser
(🟡 (1) below)", "refinement, 🟡 below". Same shape: parser-side +
refinement-side gates.
- **Triage: same as yaml.dag — VALID-in-intent / VAGUE-in-form.**
- **Action queued:** same as yaml.dag.

**`extdeps/languages/typescript.dag` × 4 in-file blocks** (lines 21,
34, 67, 69) — `D2a(2) grounding-map facet is 🟡 operator-pending` /
`alias rows 🟡 TRACKED-SCAFFOLD per std/ ladders + refinement policy`.
- **Triage: INVALID-GATE.**
- **Why:** the named arrival ("D2a(2) grounding-map facet", "alias
  rows per std/ ladders") references the **D2 alias-identity model**.
  Per the operator's 2026-05-17 D2-reversal directive, D2 alias-identity
  was REJECTED in favor of the **fact-bundle model** — the named
  arrival as written *will not arrive*. This is not STALE (the gate
  has not "already opened") and not VAGUE (the gate is concretely
  named); it is **INVALID-GATE** — the gate's target was cancelled by
  the design reversal. Per #3244, an INVALID-GATE entry must be
  re-gated against the post-reversal model (the T-4 fact-bundle
  Phase-3 rework — keystone PR #3226 merged @`77b9e7d72`; Phase-3 is
  5-feeder-gated, four feeders open: T-3, T-29, T-30, T-25-core).
- **Owning lane (re-gate action — out of C1 scope per "C1 marks and
  flags, does not fix"):** the lane owning
  `extdeps/languages/typescript.dag` (Lane C / T-4 manager
  vivid-carp-207); the standing T-4-rework-PR HOLD directive applies.

### 3.3 Summary table

| tracker | shape | gate-kind | concrete-arrival? | dissolve-on-arrival obligation? | triage |
|---|---|---|---|---|---|
| SL-3229-INTEGER-GROUP-COMPLETION | DECISIONS.md row | feature | partial (feature named, owning task TBD) | yes | **VAGUE** (no owning task → not bound to a named substrate PR; tightened bar) |
| SL-3229-LLVM-WIDTH | DECISIONS.md row | feature | yes (cardinality.dag T-3) | yes | **CLOSED** (§3.1 — live `llvm_ir.dag` carriers; ledger row retained) |
| SL-3229-LLVM-OPS | DECISIONS.md row | feature | yes (cardinality.dag T-3) | yes | VALID |
| SL-3229-PTX-DIM3 | DECISIONS.md row | feature | yes (cardinality.dag T-3) | yes | VALID |
| SL-3229-PTX-COST | DECISIONS.md row | feature | yes (cardinality.dag T-3) | yes | CLOSED |
| SL-3229-VERILOG-NONEMPTY | DECISIONS.md row | feature | yes (collection.dag Wave-A2) | yes | VALID |
| SL-3229-VERILOG-D3200 | DECISIONS.md row | consumer | **no** (class, not one name) | yes | **VAGUE** |
| SL-3229-VERILOG-VECTOR-RANGE | DECISIONS.md row | feature | yes (T-4 Verilog constant_expression) | yes | CLOSED |
| SL-3229-VERILOG-COST | DECISIONS.md row | feature | yes (cardinality.dag T-3) | yes | CLOSED |
| SL-3229-FLOAT-NOMINAL | DECISIONS.md row | feature | **partial** (straddles machine.dag + cardinality.dag) | yes | **VAGUE** |
| verilog.dag × 5 in-file cite-sites | one-liner | (inherits SL-3229-VERILOG-D3200) | — | — | **VAGUE** (inherits) |
| `llvm_ir.dag` (`LlvmType` row; legacy `llvm_ir.dag:28` snapshot) | one-liner | feature | yes (superseded — §3.1 **`SL-3229-LLVM-WIDTH` CLOSED**) | n/a | **STALE** (§3.2 — 🟢 terminal cite at HEAD; not a second `VALID-🟡` authority) |
| json.dag × 3 in-file blocks | prose | mixed | **no** (class) | yes | **VAGUE** |
| yaml.dag × 6 in-file blocks | prose | mixed | **no** (class) | yes | **VAGUE** |
| toml.dag × 7+ in-file blocks | prose | mixed | **no** (class) | yes | **VAGUE** |
| typescript.dag × 4 in-file blocks | prose | feature (D2-shaped) | **named arrival cancelled by D2-reversal** | yes | **INVALID-GATE** (re-gate post-reversal) |

Counts (under the still-hawk-102 tightened bar 2026-05-18 — VALID-🟡
requires concrete gate AND binding to a named primitive+owning-task
substrate PR in Section 1):

- **3 VALID-🟡** (3 DECISIONS.md rows: LLVM-OPS, PTX-DIM3,
  VERILOG-NONEMPTY bound to P1/P2). **`SL-3229-LLVM-WIDTH`** is **CLOSED**
  (§3.1; §3.3 table row matches — not counted here). The legacy
  `llvm_ir.dag:28` cite snapshot is **STALE** (§3.2 / §3.3 — **not** a
  parallel `VALID-🟡` authority). P8 closed `VERILOG-VECTOR-RANGE`;
  `PTX-COST` and `VERILOG-COST` are **CLOSED**; those rows remain in
  DECISIONS.md as audit receipts, not live 🟡.
- **3 VAGUE DECISIONS.md rows** — `SL-3229-VERILOG-D3200` (consumer-class
  not named) + `SL-3229-FLOAT-NOMINAL` (straddles two owners) +
  **`SL-3229-INTEGER-GROUP-COMPLETION`** (no owning task — reclassified
  under the tightened bar; P10 in Section 1 carries this flag).
- **~16 VAGUE in-file prose blocks** across `json.dag`, `yaml.dag`,
  `toml.dag`, and 5 `verilog.dag` cite-sites inheriting VERILOG-D3200
  ≈ **~19 VAGUE total**.
- **4 INVALID-GATE** (typescript.dag D2-shaped gates).
- **1 STALE** (superseded cite-site snapshot only — `llvm_ir.dag`
  `LlvmType` row per §3.2; **does not** re-open §3.1 **`SL-3229-LLVM-WIDTH`
  CLOSED** or duplicate `VALID-🟡` counts above).

**Headline finding:** §3.1 / §3.2 / §3.3 agree on **`SL-3229-LLVM-WIDTH`**
**CLOSED** + cite-site **STALE**; remaining cardinality-refinement /
collection Wave-A2 / constrained-generics arrivals are still ahead of us,
while the raw non-negative cost rows closed once their live carriers moved
to `Nat`. Verified against
`std/cardinality.dag`, `std/collection.dag`, `std/nat.dag`,
`std/algebra.dag` on `main` @ `ce0241039`. The pre-existing-tracker
debt is **overwhelmingly VAGUE prose-form gates that #3244 retires** —
concretizing the in-file prose blocks under one canonical
`feature: std/cardinality.dag refinement (T-3 Wave-A2)` (and a smaller
parser-side family `feature: T-6/T-7 parse/emit pipeline-stage
substrate`) collapses much of the VAGUE list to VALID. The 4
typescript.dag INVALID-GATE entries need substantive re-gating against
the post-D2-reversal fact-bundle model. **All re-gates and dissolve-now
fixes are downstream lane work, not C1's** — C1 marks and flags.

---

## Section 2 summary (per-class roll-up)

| class | 🔴 dissolve-now | 🟡 gated (named feature: / consumer:) | 🟢 terminal |
|---|---|---|---|
| walker | — | 2 (`std/algebra.dag free_monoid_length` → `fold FreeMonoid`; `std/float.dag nat_compare` → `fold Nat`) + **P5 cascade** (`compiler/03_resolve.dag` binding-harvest sites → scoped `fold_node` algebra; substrate **#3297**) | rest |
| traverse | — | 4 (`std/node.dag` × 4: → `forall` / `count_where` / `unique` over `FreeMonoid<T>` in `std/collection.dag` Wave-A2) | rest |
| predicate | empty-`Conj` R1 predicate **landed** PR #3284 (`std/node.dag` `is_empty_conj_root`, `DECISIONS.md` §CP-1b item 12); LLVM R2 predicate **landed** PR #3245 (`terminator_is_catchswitch` deleted; `block_well_formed` consumes `CatchSwitch` directly) | 3 (`std/node.dag connective_edge_discipline` → per-`Connective` discipline fact; `std/float.dag float_finite_magnitude_zero` → `nat_is_zero`; `extdeps/languages/llvm_ir.dag feature_disposition` → per-feature disposition fact (T-4 fact-bundle)); `llvm_instruction_cost` **landed** under `lens/cost.dag`. | rest |
| carrier | — | — | lane-wide |
| emit/template | — | — | lane-wide |

**Substrate primitives the 🟡 tier names as missing** (the named
`feature:` arrivals — the conditions for landing each 🟡 per #3244's
dissolve-on-arrival rule):

1. ~~`fold_node` — `Node` catamorphism in `std/node.dag` (substrate-extension
   under T-1).~~ **LANDED PR #3297** (`NodeFold<R>`, `fold_node`; `node_well_formed` is a consumer). **P5 cascade** (binding-harvest in `03_resolve.dag`) remains open under a scoped algebra, not absent substrate.
2. `fold` / `cata` over `FreeMonoid<T>` and `Nat` in `std/algebra.dag` /
   `std/nat.dag` (Wave-A2).
3. `forall`, `count_where`, `unique` over `FreeMonoid<T>` in
   `std/collection.dag` (Wave-A2).
4. `nat_is_zero : Nat -> Bool` in `std/nat.dag` (Wave-A2).
5. Property-projection model facts (Practice 10 registry row 7) on
   `Connective` (T-1 substrate-extension) and `FidelityFeature` (T-4
   fact-bundle). `LlvmInstruction` cost landed under `lens/cost.dag`;
   `Terminator` well-formedness closed by the LLVM
   `terminator_is_catchswitch` dissolve-now landing.

**Dissolve-now (🔴) inventory** — two findings: **both landed**:

1. ~~`is_empty_conj_root : Node -> Bool` — extract into `std/node.dag`~~
   **DONE — PR #3284** (`src/v4/std/node.dag`; R1 receipt +
   `DECISIONS.md` §CP-1b item 12).
2. ~~Inline `terminator_is_catchswitch` into `block_well_formed` in
   `extdeps/languages/llvm_ir.dag`; delete the discriminator predicate~~
   **DONE — PR #3245** (`src/v4/extdeps/languages/llvm_ir.dag`;
   `block_well_formed` now matches `CatchSwitch` directly).

Per Practice 10, the matching in-file `.dag` tag lands with the fix
where the target file allows body comments; strict-deprose allowlisted
files carry the receipt in the ledger/audit instead. This inventory is
the classification.
