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

**Amendment (gunbc PR #3299, 2026-05-18):** §3 pre-plan backlog items are
**concretized** in authoritative `src/v4/DECISIONS.md` Part 6 rows
(`SL-3229-VERILOG-D3200`, `SL-3229-FLOAT-NOMINAL`, and
`SL-3229-INTEGER-GROUP-COMPLETION` as a **named `feature:`** on **P10**
with **⛔ owning `TASKS.md` T-# still TBD**). The three T-4.6 format
carriers (`extdeps/formats/{json,yaml,toml}.dag`) record the matching
**#3244 / Practice-4** obligations as **terse `// 🟡 gated — feature: … —
DECISIONS.md Part 6 · <SL-3229-*>` lines** immediately **after** each
file's closed value coproduct — **no** separate `Deferred` / long-form
narrative block on current `main` + #3299; **single** authority path is
**DECISIONS.md Part 6** + those one-liners (Practice 5). Section 3
triage + §1.1 rollup language below reflects this amendment; live
**`feature:`** obligations are always the **DECISIONS.md** row text when
the two sources differ.

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
  jumps the queue, lands in a follow-up PR. **Count:** **R2 only**
  (1 finding, 1 fix) in § 1.0 — **R1 landed PR #3284** (`is_empty_conj_root`).
  Checked, not omitted.
- **🟡 gated** — substrate primitive does not exist yet; carries
  `feature:<primitive + owning task>` or `consumer:<named consumer>`
  + dissolve-on-arrival obligation. A 🟡 is a *committed*
  surface→dissolve loop, not a parking spot. **Count today: ~38
  bound to P1-P10 in § 1.1 (including the two #3225-merged findings
  added at final-ready — see § 1.1 P4 and P5 unblocks columns), plus
  **one** remaining pre-plan item (**P10 / INTEGER-GROUP-COMPLETION** —
  gate text concretized in `DECISIONS.md`, **⛔** pending owning **T-#**
  per PR #3299). Former §3 VAGUE / INVALID-GATE backlog is **rolled** —
  see amendment note + Section 3 refresh.**
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

- **§ 1.0** — **🔴 dissolve-now (jumps queue, no substrate gap).**
  Hand-rolled constructs whose substrate primitive *already exists* on
  `main`. Land immediately, ahead of P1. (Per Practice 10 / #3244: 🔴
  is a directive, never a standing state.)
- **§ 1.1** — **Ranked substrate-PR queue (P1-P10, 🟡 plan).** Every
  🟡 in this inventory waits on one of ten named arrivals; each row =
  one substrate PR, ranked by 🟡-count (highest first), with the
  finding list it unblocks. **Dissolution follow-ups dispatch
  immediately** on each substrate PR's landing — the surface→dissolve
  loop is what makes 🟡 transient.
- **§ 1.2** — 🟡 → 🟢 burn-down view.
- **Pre-plan concretization backlog** — Section 3 formerly listed ~19
  VAGUE + 4 INVALID-GATE entries **outside** the P1–P10 plan. **PR #3299
  concretized** the authoritative `DECISIONS.md` rows + format-file plan
  anchors; only **`SL-3229-INTEGER-GROUP-COMPLETION`** remains
  structurally **outside P1–P9** until an owning **T-#** binds the **P10**
  substrate row (the gate text itself is already a named `feature:`).

### 1.0 🔴 dissolve-now (jumps queue, lands ahead of P1)

**R1 (empty-`Conj`-root duplicate predicates)** — **LANDED PR #3284**
(`src/v4/std/node.dag` `is_empty_conj_root`; disposition
`DECISIONS.md` §CP-1b item 12). **R2** below remains the sole open
🔴 in this band — one distinct follow-up PR. Each substrate primitive
needed for R2 *already exists* on `main` @ `ce0241039` — no upstream
substrate work blocks it.

| # | finding | distinct fix | substrate available? |
|---|---|---|---|
| **R1** | WAS: `compiler/01_tokenize.dag lex_rules_node_is_conj_empty_root` **+** `compiler/02_parse.dag grammar_node_is_conj_empty_root` **+** `extdeps/languages/dag.dag dag_node_is_empty_conj_root` — three literal duplicates of the same empty-`Conj`-root shape query | **LANDED PR #3284:** `fn is_empty_conj_root` in `src/v4/std/node.dag`; all three sites import it (duplicate authority removed). **P5 / predicate disposition:** `src/v4/DECISIONS.md` §CP-1b item 12 — interim shared structural leg; forward dissolution under inventory §1.1 **P3** (T-6/T-7 walks) / **P5** (`fold_node`). | ✓ — pure composition over existing `std/node.dag` primitives (`Node`, `NodeKind`, `TypeNode`, `Conj`, `count`); all on main. |
| **R2** | `extdeps/languages/llvm_ir.dag terminator_is_catchswitch` (526) — naked `match … _ => false` discriminator | Inline the `match` into the sole consumer `block_well_formed` (533); delete the discriminator predicate. | ✓ — pure pattern match on `Terminator` (already declared in `llvm_ir.dag`); no new substrate. |

**R2** remains owned by the `extdeps/languages/llvm_ir.dag` lane. **R1**
landed under the compiler/std lane (PR #3284). Per "C1 marks + flags +
plans, does not fix," C1 does not author the R2 PR; it surfaces R2 as
the audit anchor.

### 1.1 Ranked substrate-PR queue

| # | substrate PR (`feature:` arrival) | owner / owning task | 🟡-count | unblocks |
|---|---|---|---|---|
| **P1** | `std/cardinality.dag` bounded-natural / refinement substrate | std / T-3 Wave-A2 | **~22** | DECISIONS.md: `SL-3229-LLVM-WIDTH`, `SL-3229-LLVM-OPS`, `SL-3229-PTX-DIM3`, `SL-3229-PTX-COST`, `SL-3229-VERILOG-COST`, **`SL-3229-FLOAT-NOMINAL`** (PR #3299 — canonical `feature:` row). In-file: `llvm_ir.dag:28` + `json.dag` / `yaml.dag` / `toml.dag` terse `// 🟡 gated — feature: … SL-3229-LLVM-WIDTH` lines (refinement-side family). |
| **P2** | `std/collection.dag` Wave-A2: `List<T> where non_empty` refinement **plus** the List combinator algebra (`forall` / `count_where` / `unique` over `FreeMonoid<T>`) | std / T-3 Wave-A2 (coercion-design.md RQ-3) | **5 named + 26 sites** | Section 2: `std/node.dag` × 4 traverses (`all_edges_named`, `all_edges_positional`, `name_occurrences`, `all_names_distinct`). DECISIONS.md: `SL-3229-VERILOG-NONEMPTY` (one row, 26 verilog.dag back-pointer sites). |
| **P3** | Compiler pipeline-stage substrate (lex-walk + parse-walk) | compiler / T-6, T-7 | **2 + ~7 in-file** | Section 2: `compiler/01_tokenize.dag tokenize`, `compiler/02_parse.dag parse`. In-file: `json.dag` / `yaml.dag` / `toml.dag` terse `// 🟡 gated — feature: …` lines citing **DECISIONS.md Part 6 · SL-3229-T4-FORMAT-T6T7** (parse/emit deferral family). |
| **P4** | T-4 fact-bundle Phase-3 rework (post-D2-reversal model) | extdeps/languages / T-4 manager `vivid-carp-207` (5-feeder gate; keystone #3226 merged @`77b9e7d72`; 4 feeders open: T-3, T-29, T-30, T-25-core) | **5 + 1 row + 1 fn** | **DECISIONS.md:** `SL-3229-VERILOG-D3200` (**PR #3299** — `feature:` T-4 Phase-3). **In-file:** `verilog.dag` × 5 coproduct one-liners (`… SL-3229-VERILOG-D3200`). `typescript.dag` sum carriers are **🟢** (`DECISIONS.md TS-D2`); file-level **🟡** scaffold is the **T-4 Phase-3** gate in the header `Status:` (see file). Section 2: `extdeps/languages/dag.dag dag_language_model_wave1_void_canonical_symbols` (CP-1b #3225). |
| **P5** | `std/node.dag` `fold_node` — Node catamorphism (substrate-extension under T-1) | std / T-1 | **1 + 3 walker callers** | `fold_node` + the `std/node.dag node_well_formed` migration landed in the burn-down closeout. Remaining: `compiler/03_resolve.dag merge_binding_self` (94, the codex #3225 "sym↦sym module harvest" finding) — together with its three named-harvest walker callers (`add_module_named_exports` at 99, `add_arrow_domain_named_params` at 113, `add_bind_atom_binder` at 140) that fold over `Node.children` with constructor-discriminated recursion. These dissolve to `fold_node(root, ⟨binding-harvest algebra⟩)` only when the scoped harvest algebra lands without changing resolver scope semantics. |
| **P6** | `std/algebra.dag` / `std/nat.dag` `fold` / `cata` over `FreeMonoid<T>` and `Nat` (Wave-A2) | std / T-3 Wave-A2 | **2** | Section 2: `std/algebra.dag free_monoid_length`, `std/float.dag nat_compare`. (Sibling to P2's combinator algebra; could land in the same PR — kept separate because the underlying primitive is the catamorphism, distinct from `forall`/`count_where` which are derived from it.) |
| **P7** | `std/nat.dag nat_is_zero : Nat -> Bool` (Wave-A2) | std / T-3 Wave-A2 | **1** | Section 2: `std/float.dag float_finite_magnitude_zero`. |
| **P8** | `extdeps/languages/verilog.dag` bundled T-4 LanguageModel `constant_expression` sub-grammar | extdeps/languages / T-4 Verilog Phase-3 | **1** | DECISIONS.md: `SL-3229-VERILOG-VECTOR-RANGE` (lexeme-pair bridge). |
| **P9** | `lens/cost.dag` cost-of-instruction model fact / lens | lens / T-12 | **0** | **LANDED:** `src/v4/lens/cost.dag` now owns `llvm_instruction_cost` (the 22-arm `LlvmInstruction -> Int` cost table); `extdeps/languages/llvm_ir.dag` owns only the LLVM instruction shape. |
| **P10 ⛔ needs-concretization** | Constrained generic parameters / inhabitance-bound syntax (`<M> where M : CommutativeMonoid<_>`) | substrate extension; **no owning task yet** | **1** | DECISIONS.md: `SL-3229-INTEGER-GROUP-COMPLETION` — **PR #3299** added the named `feature:` + **P10** roll-up text; **⛔ owning `TASKS.md` T-#** still required before the P10 row is merge-valid under #3244. |


Plus property-projection model facts (Practice 10 row 7) that do not
roll up into a shared substrate PR — each is a per-type fact-bundle
extension under its own owner; named in Section 2 but not ranked here
because they don't amortize across multiple findings. The two such
findings are `std/node.dag connective_edge_discipline` (T-1) and
`extdeps/languages/llvm_ir.dag feature_disposition` (T-4 fact-bundle —
rolls into **P4** as a separate fact within that rework). The third —
`llvm_ir.dag block_well_formed` — is not really a substrate gap; it
cascades off the `terminator_is_catchswitch` dissolve-now and closes
without waiting on substrate.

### 1.2 🟡 → 🟢 burn-down

Total 🟡 in the v4 substrate today (Section 2 + Section 3 VALID-🟡 +
INVALID-GATE-once-re-gated):

| primitive PR | 🟡 today | landing event | 🟡 after landing |
|---|---|---|---|
| (baseline) | **~38** | — | — |
| P1 lands | ~22 | `std/cardinality.dag` refinement | ~16 |
| P2 lands | 5 named (+ 26 verilog sites converge in one sweep) | `std/collection.dag` Wave-A2 | ~11 named |
| P3 lands | 2 named (+ ~7 in-file) | T-6 + T-7 pipeline substrate | ~9 named |
| P4 lands | 4 + 1 row + 1 fn (`dag.dag canonical_symbols` #3225) | T-4 fact-bundle Phase-3 | ~3 named |
| P5 lands | 1 named (+ 3 walker callers cascade in `03_resolve.dag` #3225) | `std/node.dag fold_node` | ~1 |
| P6 lands | 2 | FreeMonoid/Nat catamorphism | ~0 |
| P7 lands | 1 | `nat_is_zero` | ~0 |
| P8 lands | 1 | Verilog constant_expression | ~0 |
| P9 landed | 0 | `lens/cost.dag` T-12 | ~0 |
| P10 lands | 1 (after concretization) | constrained-generics syntax | **0** |

(Residual column is illustrative — counts roll up imperfectly because
some entries are counted with cascades — e.g. P5's `merge_binding_self`
implicates its 3 named-harvest walker callers — and Section 3 VAGUE
entries that concretize to a P# only enter their column on re-gate.
The cumulative endpoint after all P1-P10 land is 0; the column shows
qualitative trajectory, not strict arithmetic.)

(R1 **landed** PR #3284; **R2** in § 1.0 is not counted in the 🟡 burn-down
— it is 🔴, lands immediately, and dissolves outside the substrate-gap queue.)

Caveats:

1. **"🟡 today" counts only entries already bound to a P# in the
   plan.** Section 2's fresh findings + Section 3 VALID-🟡 are counted.
   **PR #3299** concretized the former §3 **VAGUE / INVALID-GATE**
   bucket into **named `feature:`** rows (`DECISIONS.md` Part 6) +
   format-file plan anchors — those entries **now roll** under **P1 /
   P3 / P4** as amended in §1.1 + §3.3. **Only** **`SL-3229-INTEGER-GROUP-COMPLETION`**
   remains structurally outside P1–P9 until an owning **T-#** binds
   **P10**. The §1.2 illustrative 🟡-count column is still **qualitative**
   (burn-down lane reconciles strict arithmetic).
2. **P10 lands** requires the owning task be assigned first; the
   "🟡 → 🟢 sweep for P10" only fires after concretization. The
   `⛔ needs-concretization` flag on P10 in § 1.1 is structurally
   blocking that row's DAG entry.

**P1 is the headline.** Landing the `std/cardinality.dag` refinement
substrate dissolves roughly 60% of the v4 substrate's outstanding 🟡
debt in a single sweep — by 🟡-count it dominates every other
substrate PR by 4× or more. The S1 substrate track should prioritize
P1 ahead of P2-P10. **R2** (§ 1.0) jumps the queue ahead of all
substrate PRs since it needs no absent substrate.

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
- `node_well_formed` (113) — ✓ **walker dissolved** —
  `fold_node` landed in `std/node.dag` and this function now consumes the
  shared `NodeFold<Bool>` algebra instead of hand-rolling recursive descent.
- `all_edges_named` (71), `all_edges_positional` (77) — 🟡 **traverse** —
  `feature: forall over FreeMonoid<T> in std/collection.dag (Wave-A2)`.
  `fold` body is `acc && pred(e)`; dissolves to
  `forall(children, pred)` on arrival.
- `name_occurrences` (82), `all_names_distinct` (90) — 🟡 **traverse** —
  `feature: count_where + unique over FreeMonoid<T> in std/collection.dag (Wave-A2)`.
  Dissolves to `count_where(children, pred)` / `unique(children)` on
  arrival.
- `connective_edge_discipline` (48) — 🟡 **predicate** (property
  projection, registry row 7) —
  `feature: per-Connective discipline fact on std/node.dag Connective (substrate-extension under T-1)`.
  Six-arm `match`-to-derive of an `EdgeDiscipline` per `Connective`;
  the discipline IS a fact on `Connective`. Dissolves to
  `c.discipline` on arrival (paired-construction at the type).
- `edge_is_named` (58), `edge_is_positional` (64) — 🟢 — naked
  constructor inspection of `EdgeLabel`; reading the model fact rather
  than deriving anything (a consumer's `match e.label` is the canonical
  form).
- `edges_conform` (98) — 🟢 — each `EdgeDiscipline` arm does
  structurally distinct work (count check / labelling rule / position
  check); irregularity escape hatch.
- `node_locally_well_formed` (106) — 🟢 — composes the above with a
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
  `Node.children` doing constructor-discriminated recursion. Same
  feature gate as `std/node.dag node_well_formed`:
  `feature: std/node.dag fold_node — Node catamorphism (substrate-extension under T-1)` —
  rolls under **P5**. Dissolves to `fold_node(root, ⟨algebra⟩)` over
  the binding-harvest algebra; eliminates the three harvest walkers
  plus the implicit recursion in `merge_binding_self`'s callers.

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
- `terminator_is_catchswitch` (526) — 🔴 **predicate** —
  `match t { CatchSwitch{…} => true ; _ => false }`. Naked
  discriminator-as-predicate; the sole consumer (`block_well_formed`)
  can inline the `match`. Dissolve now.
- `block_well_formed` (533) — 🟡 **predicate** —
  `consumer: post-dissolve-now inliner of terminator_is_catchswitch (this PR's follow-up)`.
  Cascades the dissolve-now above; the catchswitch-body invariant
  belongs as a per-`Terminator` constructor invariant.
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
- **Triage: CONCRETIZED — still ⛔ outside P1–P9 until T-#.** PR #3299
  adds the authoritative **`feature:`** + **P10** roll-up in
  `DECISIONS.md` Part 6. Under merged #3244 the row is **not** yet a
  fully merge-valid standing 🟡 substrate PR binding (no committed
  owning **T-#** / substrate PR for P10); it **does not** roll under
  P1–P9. **Action owner:** substrate / operator-or-S1 assigns **T-#**.
- **#3244 re-expression:** see **`DECISIONS.md` Part 6** (`SL-3229-INTEGER-GROUP-COMPLETION`).

**`SL-3229-LLVM-WIDTH`** — raw-`Int` width payload scaffold
(`LlvmType` family). Named arrival: "std/cardinality.dag refinement
substrate lands (T-3)." Verified `std/cardinality.dag` on main:
`DescentEvidence`, `RankingDimension`, `TerminationProof`,
`Multiplicity` modeled; **no bounded-natural / refinement primitive**.
- **Triage: VALID.**
- **#3244 re-expression:** `🟡 gated — feature: bounded-natural refinement in std/cardinality.dag (T-3 Wave-A2)`. Gate is concrete; owning task named.

**`SL-3229-LLVM-OPS`** — operation-specific operand constraints. Named
arrival: same as `SL-3229-LLVM-WIDTH` (cardinality refinement / operand
relation refinement).
- **Triage: VALID.**
- **#3244 re-expression:** `🟡 gated — feature: refined typed-value carrier in std/cardinality.dag (T-3 Wave-A2)`.

**`SL-3229-PTX-DIM3`** — `Dim3` kernel-ambient `Int` axis scaffold.
Named arrival: same cardinality-refinement family (T-3).
- **Triage: VALID.**
- **#3244 re-expression:** `🟡 gated — feature: bounded-positive-Int refinement in std/cardinality.dag (T-3 Wave-A2)`.

**`SL-3229-PTX-COST`** — raw-`Int` PTX cost axes (`PtxCost`). Named
arrival: cardinality refinement (T-3).
- **Triage: VALID.**
- **#3244 re-expression:** `🟡 gated — feature: bounded-non-negative-Int refinement in std/cardinality.dag (T-3 Wave-A2)`.

**`SL-3229-VERILOG-NONEMPTY`** — shared `List<T>` spec-non-empty
Wave-A2 deferral (26 sites). Named arrival: `std/collection.dag`
Wave-A2 `List<T> where non_empty`. Verified `std/collection.dag` on
main: `type List<T> = FreeMonoid<T>` alias only — no `where non_empty`
refinement landed.
- **Triage: VALID.**
- **#3244 re-expression:** `🟡 gated — feature: List<T> where non_empty refinement in std/collection.dag (Wave-A2 — coercion-design.md RQ-3)`.

**`SL-3229-VERILOG-D3200`** — #3200 consumer-independent 🟡 coproducts
(first-consumer decomposition; 5 carriers: `NonTriregNetKind`,
`VariableDeclaration`, `OutputPortAnsiVariableTypeKind`,
`ParameterTypeKind`, `PrimitiveGateKind`). Former merge-base arrival
was a **consumer class** / pre-reversal D2-shaped text (not one
checkable `consumer:<name>`).
- **Triage: CONCRETIZED → VALID under P4 (PR #3299).** Authoritative
  **`feature:`** + dissolve plan: **`DECISIONS.md` Part 6** row
  `SL-3229-VERILOG-D3200` (**T-4 fact-bundle Phase-3 rework**,
  vivid-carp-207). Live `verilog.dag` one-liners keep the strict
  de-prose canonical tail `… SL-3229-VERILOG-D3200` — the row is the
  gate authority.
- **#3244 re-expression:** see **`DECISIONS.md` Part 6**
  (`SL-3229-VERILOG-D3200`).

**`SL-3229-VERILOG-VECTOR-RANGE`** — `VectorRange` lexeme-pair bridge.
Named arrival: "bundled T-4 LanguageModel `constant_expression`
productions." Verified: T-4 Verilog `LanguageModel` constant_expression
sub-grammar has not landed.
- **Triage: VALID.**
- **#3244 re-expression:** `🟡 gated — feature: constant_expression sub-grammar in extdeps/languages/verilog.dag bundled LanguageModel (T-4 Verilog fact-bundle Phase-3)`.

**`SL-3229-VERILOG-COST`** — raw-`Int` Verilog cost axes
(`VerilogCost`). Named arrival: cardinality refinement (T-3).
- **Triage: VALID.**
- **#3244 re-expression:** `🟡 gated — feature: bounded-non-negative-Int refinement in std/cardinality.dag (T-3 Wave-A2)` (same family as the four LLVM/PTX cost/width gates above).

**`SL-3229-FLOAT-NOMINAL`** — nominal width / interchange list-length
scaffold. Former merge-base text straddled `std/machine.dag` notes vs
the real refinement substrate in `std/cardinality.dag`.
- **Triage: CONCRETIZED → VALID under P1 (PR #3299).** Authoritative
  single owner: **`DECISIONS.md` Part 6** row `SL-3229-FLOAT-NOMINAL`
  (`feature:` **`std/cardinality.dag`** bounded-natural / numeric
  refinement, T-3 Wave-A2); `std/machine.dag` remains cross-doc only.
- **#3244 re-expression:** see **`DECISIONS.md` Part 6**
  (`SL-3229-FLOAT-NOMINAL`).

### 3.2 In-file 🟡 cite-sites

Result of `grep -n "🟡" src/v4/**/*.dag` (excluding `verification.dag:128`,
which is descriptive prose about the 🟢/🟡/🔴 convention itself).

**`extdeps/languages/verilog.dag` × 5 cite-sites** (lines 24, 174, 207,
264, 473) — all read `// 🟡 coproduct dissolution — DECISIONS.md Part 6 ·
SL-3229-VERILOG-D3200.`
- **Triage: VALID** (inherits **P4** `feature:` gate from **Section 3.1 /
  `DECISIONS.md` `SL-3229-VERILOG-D3200`** — PR #3299; strict de-prose
  keeps the canonical one-liner tail).

**`extdeps/languages/llvm_ir.dag:28`** — `// 🟡 coproduct dissolution —
DECISIONS.md Part 6 · SL-3229-LLVM-WIDTH.`
- **Triage: VALID** (inherits from 3.1 SL-3229-LLVM-WIDTH VALID).
- **#3244 re-expression at cite-site:** can stay as-is (the ledger row
  itself is the authority; the in-file one-liner is a pointer). If the
  operator-mandated form requires the gate kind on the cite-site too,
  cite-site becomes `// 🟡 coproduct dissolution — feature:
  std/cardinality.dag refinement (T-3) — DECISIONS.md Part 6 · SL-3229-LLVM-WIDTH.`

**`extdeps/formats/json.dag`** — **Live (post–strict-deprose / #3299):**
**23 lines** total. After `JsonValue` (lines 12–18), **lines 21–22** are
exactly **two** terse `// 🟡 gated — feature: … — DECISIONS.md Part 6 ·
SL-3229-T4-FORMAT-T6T7` / `… SL-3229-LLVM-WIDTH` gates (parse/emit → **P3**
family + cardinality / numeric carriers → **P1**). The historical C1
sweep cited **multi-hundred-line** `// 🟡 TRACKED-SCAFFOLD` prose blocks
at old line numbers — **retired** on `main`; they are **not** present
unchanged on HEAD.
- **Triage: CONCRETIZED (PR #3299 + main de-prose).** Obligations are
  carried only as the **Part 6–slug one-liners** above — same story as
  §1.1 **P3** / **P1** rows.

**`extdeps/formats/yaml.dag`** — **Live:** **25 lines**. After `YamlValue`
(lines 12–19), **lines 22–24** are **three** terse `// 🟡 gated — feature:`
lines (`SL-3229-T4-FORMAT-T6T7`, `SL-3229-LLVM-WIDTH`,
`SL-3229-YAML-CANONICAL-KEYS`). No `Deferred` section.
- **Triage: CONCRETIZED (PR #3299 + main de-prose).** Same authority
  pattern as `json.dag`.

**`extdeps/formats/toml.dag`** — **Live:** **27 lines**. `TomlValue` is
preceded by `// 🟡 coproduct dissolution — … SL-3229-TOML-TABLE-SYNTAX`
(line 11); after the sum (lines 12–20), **lines 23–26** are **four**
terse `// 🟡 gated — feature:` lines (`SL-3229-T4-FORMAT-T6T7`,
`SL-3229-LLVM-WIDTH`, `SL-3229-T4-FORMAT-TOML-DATETIME`,
`SL-3229-TOML-TABLE-SYNTAX`). No `Deferred` section.
- **Triage: CONCRETIZED (PR #3299 + main de-prose).** Same authority
  pattern as `json.dag`.

**`extdeps/languages/typescript.dag` (historical §3.2 cite)** — prior
audit referenced ×4 INVALID-GATE prose blocks on **cancelled** D2
alias-identity arrivals (lines 21, 34, 67, 69 on old `main` snapshots).
- **Triage: RESOLVED on PR #3299 branch.** Live file is **37 lines**;
  sum coproducts carry **🟢** `DECISIONS.md TS-D2` one-liners; the file
  header carries a **single-line** `Status:` **🟡 gated — feature: T-4
  fact-bundle Phase-3 …** gate (terse header discipline). Remaining
  bundle work rolls under **§1.1 P4** — not INVALID-GATE debt.

### 3.3 Summary table

| tracker | shape | gate-kind | concrete-arrival? | dissolve-on-arrival obligation? | triage |
|---|---|---|---|---|---|
| SL-3229-INTEGER-GROUP-COMPLETION | DECISIONS.md row | feature | partial (named `feature:` + P10 roll-up; **⛔** owning T-# TBD) | yes | **CONCRETIZED — P10 pending T-#** |
| SL-3229-LLVM-WIDTH | DECISIONS.md row | feature | yes (cardinality.dag T-3) | yes | VALID |
| SL-3229-LLVM-OPS | DECISIONS.md row | feature | yes (cardinality.dag T-3) | yes | VALID |
| SL-3229-PTX-DIM3 | DECISIONS.md row | feature | yes (cardinality.dag T-3) | yes | VALID |
| SL-3229-PTX-COST | DECISIONS.md row | feature | yes (cardinality.dag T-3) | yes | VALID |
| SL-3229-VERILOG-NONEMPTY | DECISIONS.md row | feature | yes (collection.dag Wave-A2) | yes | VALID |
| SL-3229-VERILOG-D3200 | DECISIONS.md row | feature | yes (T-4 Phase-3 rework) | yes | **VALID (PR #3299)** |
| SL-3229-VERILOG-VECTOR-RANGE | DECISIONS.md row | feature | yes (T-4 Verilog constant_expression) | yes | VALID |
| SL-3229-VERILOG-COST | DECISIONS.md row | feature | yes (cardinality.dag T-3) | yes | VALID |
| SL-3229-FLOAT-NOMINAL | DECISIONS.md row | feature | yes (cardinality.dag T-3; machine.dag = cross-ref only) | yes | **VALID (PR #3299)** |
| verilog.dag × 5 in-file cite-sites | one-liner | feature | yes (inherits SL-3229-VERILOG-D3200 / P4) | — | **VALID** |
| llvm_ir.dag:28 in-file cite-site | one-liner | feature | yes (inherits SL-3229-LLVM-WIDTH) | yes | VALID |
| json.dag × 3 in-file blocks | prose | mixed | yes (plan anchor → P1+P3) | yes | **CONCRETIZED** |
| yaml.dag × 6 in-file blocks | prose | mixed | yes (plan anchor → P1+P3) | yes | **CONCRETIZED** |
| toml.dag × 7 in-file blocks | prose | mixed | yes (plan anchor → P1+P3) | yes | **CONCRETIZED** |
| typescript.dag (historical cite) | prose | feature | yes (P4 / TS-D2) | yes | **RESOLVED** |

Counts (**post-PR #3299** amendment to Section 3 — authoritative gates
in `DECISIONS.md` Part 6 when this inventory disagrees):

- **11+ VALID-🟡 bound to §1.1** — prior **8** (§3.3 pre-amendment
  footnote) **+** `SL-3229-VERILOG-D3200` + `SL-3229-FLOAT-NOMINAL` +
  **5** `verilog.dag` cite-sites now inherit the **P4** / **P1**
  roll-ups respectively (numeric reconciliation vs strict 🟡-count
  table in §1.2 remains the burn-down lane's job).
- **0 VAGUE DECISIONS.md rows** from the former §3.1 trio — D3200 and
  FLOAT-NOMINAL are **VALID**; INTEGER is **CONCRETIZED** but **⛔**
  pending **T-#** (not "VAGUE" — the `feature:` is named).
- **0 VAGUE in-file backlog** from the former `json`/`yaml`/`toml` /
  `verilog` §3.2 bucket — **CONCRETIZED** or **VALID** per above.
- **0 INVALID-GATE** — `typescript.dag` historical INVALID-GATE prose
  is **cleared** on the #3299 branch (🟢 coproducts + TS-D2).
- **0 STALE → 🔴** — unchanged.

**Headline finding (post-#3299):** the §3 **pre-plan** bucket that was
**not rollable under P1–P10** is **cleared** except **`SL-3229-INTEGER-GROUP-COMPLETION`**
awaiting an owning **T-#** for the **P10** row. Authoritative **`feature:`**
text for D3200 / FLOAT-NOMINAL / INTEGER lives in **`DECISIONS.md` Part 6**;
`extdeps/formats/{json,yaml,toml}.dag` carry **#3244 plan anchors** tying
Deferred prose to **P1** + **P3**. Downstream lane work is execution
(T-3/T-4/T-6/T-7 landings), not further §3 triage on these items.

**All re-gates and dissolve-now
fixes are downstream lane work, not C1's** — C1 marks and flags.

---

## Section 2 summary (per-class roll-up)

| class | 🔴 dissolve-now | 🟡 gated (named feature: / consumer:) | 🟢 terminal |
|---|---|---|---|
| walker | — | 3 (`std/node.dag node_well_formed` → `fold_node`; `std/algebra.dag free_monoid_length` → `fold FreeMonoid`; `std/float.dag nat_compare` → `fold Nat`) | rest |
| traverse | — | 4 (`std/node.dag` × 4: → `forall` / `count_where` / `unique` over `FreeMonoid<T>` in `std/collection.dag` Wave-A2) | rest |
| predicate | 1 (`extdeps/languages/llvm_ir.dag terminator_is_catchswitch`); empty-`Conj` R1 predicate **landed** PR #3284 (`std/node.dag` `is_empty_conj_root`, `DECISIONS.md` §CP-1b item 12) | 4 (`std/node.dag connective_edge_discipline` → per-`Connective` discipline fact; `std/float.dag float_finite_magnitude_zero` → `nat_is_zero`; `extdeps/languages/llvm_ir.dag` × 2: `block_well_formed` → cascade of dissolve-now; `feature_disposition` → per-feature disposition fact (T-4 fact-bundle)); `llvm_instruction_cost` **landed** under `lens/cost.dag`. | rest |
| carrier | — | — | lane-wide |
| emit/template | — | — | lane-wide |

**Substrate primitives the 🟡 tier names as missing** (the named
`feature:` arrivals — the conditions for landing each 🟡 per #3244's
dissolve-on-arrival rule):

1. `fold_node` — `Node` catamorphism in `std/node.dag` (substrate-extension
   under T-1).
2. `fold` / `cata` over `FreeMonoid<T>` and `Nat` in `std/algebra.dag` /
   `std/nat.dag` (Wave-A2).
3. `forall`, `count_where`, `unique` over `FreeMonoid<T>` in
   `std/collection.dag` (Wave-A2).
4. `nat_is_zero : Nat -> Bool` in `std/nat.dag` (Wave-A2).
5. Property-projection model facts (Practice 10 registry row 7) on
   `Connective` (T-1 substrate-extension), `FidelityFeature` (T-4
   fact-bundle), `Terminator` well-formedness (cascade of the LLVM
   `terminator_is_catchswitch` dissolve-now).

**Dissolve-now (🔴) inventory** — three findings: **one landed**, one
open:

1. ~~`is_empty_conj_root : Node -> Bool` — extract into `std/node.dag`~~
   **DONE — PR #3284** (`src/v4/std/node.dag`; R1 receipt +
   `DECISIONS.md` §CP-1b item 12).
2. Inline `terminator_is_catchswitch` into `block_well_formed` in
   `extdeps/languages/llvm_ir.dag`; delete the discriminator predicate.

Per Practice 10, the matching in-file `.dag` tag lands with the fix
(per migration PR), not retro-applied here. This inventory is the
classification.
