### v4 retroactive dissolution audit — symbol-marked inventory

Per `docs/modeling-discipline.md` Practice 10 (the derived-operations registry
and dissolution-findings family). Classification per file, per finding:
🟢 clean / 🟡 substrate-sequencing (tracked) / 🔴 fix-now.

Scope: `src/v4/compiler/*.dag`, `src/v4/std/*.dag`, `src/v4/extdeps/**/*.dag`
at `88ae56d2a`. Five finding classes audited:

- **walker** — hand-rolled structural recursion (registry row 1; substrate
  primitive `fold_node`).
- **traverse** — `fold` body that hand-inlines effect threading
  (registry row 2; substrate primitive `traverse` / `sequence`).
- **predicate** — `match` on kind/symbol whose purpose is to *derive* a
  property (registry row 7).
- **carrier** — local coproduct that clones an existing `std/` carrier
  (sharpened Practice 4 sub-case; e.g. an `Outcome<T>` re-invention).
- **emit/template** — string-templated emitter (registry row 6).

Coproduct dissolution (registry row 0, already enforced via the per-coproduct
🟢/🟡/🔴 emoji + `DECISIONS.md` ledger) is **out of scope** here.

The disposition records the *finding's* state; the matching in-file `.dag`
tag lands with the fix, per migration PR — not retro-applied across all v4
files at once (Practice 10).

---

#### Lane-wide

- **carrier dissolution** — 🟢 lane-wide. No local `... { value: T } | ... {
  diagnostic: Diagnostic }` clone of `Outcome<T>` outside `std/diagnostic.dag`
  itself; every consumer imports and uses the std carrier.
- **emit/template dissolution** — 🟢 lane-wide. No literal `template: "..."`
  field exists in any v4 `.dag`; emit substrate is grammar-as-data
  (`extdeps/languages/*.dag` LanguageModel + `compiler/05_emit.dag`).

---

#### `src/v4/std/`

**`std/node.dag`**
- `node_well_formed` (113) — 🟡 **walker** — hand-rolled structural recursion
  over `Node` via `fold(n.children, …, fn(acc, e) { … node_well_formed(e.target) })`.
  Substrate-gap: `fold_node` / a `Node` catamorphism is not yet in `std/`
  (Practice 10 registry row 1). Tracked: T-1 substrate-extension, named
  primitive `fold_node`.
- `all_edges_named` (71), `all_edges_positional` (77) — 🟡 **traverse** —
  `fold` body is `acc && pred(e)`; `forall` / `all` over a `FreeMonoid<T>`
  belongs. Substrate-gap: `all : FreeMonoid<T> -> (T -> Bool) -> Bool` not in
  `std/collection.dag` (Wave-A2). Tracked.
- `name_occurrences` (82), `all_names_distinct` (90) — 🟡 **traverse** —
  count-by-predicate / nested `fold ∘ match`. Substrate-gap: `count_where`,
  `unique` over `FreeMonoid<T>` not in `std/collection.dag`. Tracked.
- `connective_edge_discipline` (48) — 🟡 **predicate** (property projection,
  registry row 7) — six `Connective` arms each mapping to one of three
  `EdgeDiscipline` values. The discipline IS a fact on `Connective`;
  hand-rolled as a 6-arm `match`-to-derive. Substrate-gap: per-`Connective`
  `discipline: EdgeDiscipline` field (or a discipline-of-connective lens) —
  same shape as `feature_disposition` in `llvm_ir.dag`.
- `edge_is_named` (58), `edge_is_positional` (64) — 🟢 — naked constructor
  inspection of `EdgeLabel`; reading the model fact rather than deriving
  anything (a consumer's `match e.label` is the canonical form, not a smell).
- `edges_conform` (98) — 🟢 — each `EdgeDiscipline` arm does structurally
  distinct work (count check / labelling rule / position check); the call
  graph is not the data graph.
- `node_locally_well_formed` (106) — 🟢 — composes the above with a
  `NodeKind` constructor split that does distinct work per arm.

**`std/algebra.dag`**
- `free_monoid_length` (84) — 🟡 **walker** — hand-rolled recursion over
  `FreeMonoid<T>`. The canonical catamorphism on the type. Substrate-gap:
  `fold : FreeMonoid<T> -> b -> (T -> b -> b) -> b` not yet declared in
  `std/algebra.dag` (Wave-A2; the substrate operation `length` reduces to).
  Tracked.
- `free_monoid_is_empty` (76) — 🟢 — reading the constructor of the
  coproduct that *defines* the type; boundary case (the predicate IS the
  model fact, not a derivation from it).

**`std/nat.dag`**
- `nat_add` (15), `nat_mul` (21) — 🟢 — Peano-recursive primitives that
  *define* arithmetic on `Nat`. The catamorphism IS what `nat_add` is; not
  a hand-rolled derived operation (registry row 1 names recursion *over* a
  type's structure that re-implements a substrate primitive — `nat_add`
  has no upstream substrate it shadows; it IS the substrate).

**`std/float.dag`**
- `nat_compare` (52) — 🟡 **walker** — structural recursion on `Nat`.
  Substrate-gap: a `fold` / `cata` over `Nat` not yet declared; tracked
  with the `FreeMonoid` `fold` (same Wave-A2 obligation). Co-located in
  `float.dag` is a placement quirk — belongs on `Nat`.
- `float_finite_magnitude_zero` (109) — 🟡 **predicate** — nested `Nat`
  match-to-`Bool`; should be `nat_is_zero(e) && nat_is_zero(f)`.
  Substrate-gap: `nat_is_zero : Nat -> Bool` primitive missing.
- `float_body_is_nan` (102) — 🟢 — naked constructor inspection of
  `FloatSpecial` over `FloatBody` (reading the model fact).
- `ordering_invert` (70) — 🟢 — symmetry of the 3-element `Ordering`;
  primitive operation on the type.
- `sign_rank_lex` (64), `float_special_rank` (92) — 🟢 — encode the
  IEEE-754 totalizer canonical ordering of sign / specials. The
  per-constructor rank IS the model fact this file declares; the `match`
  is the *constructor* of the ranking, not a `match`-to-derive of a
  pre-existing one (irregularity escape hatch — these ARE the data the
  semantic-total order is defined against).
- `float_finite_unsigned_field_order` (77), `float_body_compare_*` (118,
  161) — 🟢 — IEEE-754-specific lexicographic comparison ladders. Each
  arm does structurally distinct semantic work (sign-rank, magnitude
  ladder, NaN-rejection). Genuinely irregular per Practice 10's
  irregularity escape hatch — the call graph is not the data graph.

**`std/integer.dag`**
- `int_add` (59), `int_negate` (63), `int_mul` (67), `int_compare` (71),
  `int_is_zero` (83), `int_div` (124), `int_mod` (134) — 🟢 — primitives
  over the kernel-ambient `Int`; not derivable from a structural model
  (`Int` is opaque kernel, not a coproduct).
- `integer_divide_by_zero_diagnostic` (109), `integer_modulo_by_zero_diagnostic`
  (117) — 🟢 — pure data constructors for `Diagnostic` values.

**`std/text.dag`**
- `string_is_empty` (16) — 🟢 — thin delegation to `free_monoid_is_empty`
  (the canonical primitive on the type it aliases).

**`std/logic.dag`**
- `bool_boolean_algebra` (15) instance lambdas (`meet` / `join` /
  `complement`) — 🟢 — primitive Boolean operations defined on the 2-arm
  `Bool` coproduct; the catamorphism IS the operation, not a hand-roll.

**`std/diagnostic.dag`** — schema only (no fns). 🟢.
**`std/witness.dag`** — schema only. 🟢.
**`std/collection.dag`** — schema only (alias + `Map.lookup` signature). 🟢.
**`std/cardinality.dag`** — schema only. 🟢.
**`std/machine.dag`** — schema only. 🟢.
**`std/verification.dag`** — schema only (test-as-data). 🟢.
**`std/report.dag`** — scaffold. 🟢.

---

#### `src/v4/compiler/`

**`compiler/01_tokenize.dag`**
- `lex_rules_node_is_conj_empty_root` (76) — 🔴 **predicate** —
  `match rules.kind { TypeNode{ Conj } => count(children)==0 ; … => false }`.
  A naked discriminator-plus-empty check. **Duplicated** in
  `compiler/02_parse.dag` as `grammar_node_is_conj_empty_root` (73) — same
  predicate, different consumer. The model fact "this Node is an empty
  conj-rooted TypeNode" belongs once, in `std/node.dag` (e.g.
  `node_is_empty_conj_root : Node -> Bool` or as a structural-shape
  refinement). Fix-now: extract to `std/node.dag`, both compiler stages
  call the shared primitive.
- `tokenize` (83) — 🟡 — three-arm Wave-1 scaffold cascade
  (`well-formed → empty-conj-root → empty-source`); structurally distinct
  diagnostic branches, but the full lexical walk is unrealized
  (`tokenize_lexical_walk_not_realized`). Substrate-gap: lexical-walk
  substrate (T-6). Tracked.

**`compiler/02_parse.dag`**
- `grammar_node_is_conj_empty_root` (73) — 🔴 **predicate** — duplicate of
  the tokenize finding above; same fix-now (extract once into `std/node.dag`).
- `parse` (80) — 🟡 — Wave-1 scaffold cascade, mirror of `tokenize`.
  Tracked: parse-walk substrate (T-7).

**`compiler/00_compile.dag`** — 8-line scaffold. 🟢.
**`compiler/03_normalize.dag`** — scaffold (no fns). 🟢.
**`compiler/03_resolve.dag`** — scaffold (no fns). 🟢.
**`compiler/04_infer.dag`** — scaffold. 🟢.
**`compiler/05_emit.dag`** — scaffold. 🟢.
**`compiler/05_eval.dag`** — scaffold. 🟢.

---

#### `src/v4/extdeps/`

**`extdeps/languages/llvm_ir.dag`**
- `terminator_is_catchswitch` (526) — 🔴 **predicate** — naked
  `match t { CatchSwitch{…} => true ; _ => false }`. Exactly the registry
  row 7 shape ("which kind is this?"). The single consumer
  `block_well_formed` (533) should inline the `match` on `Terminator`
  (no `_ => false` fall-through, since the rule "catchswitch body must be
  empty" is constructor-specific). Fix-now.
- `block_well_formed` (533) — 🟡 **predicate** — depends on the
  `terminator_is_catchswitch` discriminator. After the fix-now above,
  this becomes a direct `match b.terminator { CatchSwitch{…} => count(b.body)==0 ; _ => true }`
  — the catch-switch-body invariant lives on the `Terminator` constructor
  itself. Substrate-gap (small): the well-formedness rule belongs as a
  per-`Terminator` invariant on the type, not in a separate predicate.
  Tracked behind the `terminator_is_catchswitch` fix.
- `feature_disposition` (565) — 🟡 **predicate** (property projection,
  registry row 7) — 12-arm `FidelityFeature -> FidelityDisposition` map.
  The disposition IS a fact on each `FidelityFeature`; hand-rolled as a
  `match`-to-derive. Substrate-gap: per-feature `disposition` field on
  `FidelityFeature` (or a paired-construction `(feature, disposition)`
  carrier). Same shape as `node.dag` `connective_edge_discipline`.
- `llvm_instruction_cost` (584) — 🟡 **predicate** (property projection)
  — 22-arm `LlvmInstruction -> Int` cost table. Same shape: cost IS a
  fact per instruction; should be a model-fact field or a lens, not a
  hand-rolled `match`. Substrate-gap: cost-of-instruction model fact /
  lens (`lens/cost.dag` T-12 is the named owner). Tracked.
- `block_successors` (505) — 🟢 — each `Terminator` arm has structurally
  distinct successor-extraction work (different field shapes:
  `[t]` / `[d, …map(cs, …)]` / `concat([f], it)` / `unwind_successors(u)`).
  The mapping is constructor-driven projection, not a `match`-to-derive
  of a pre-existing fact.
- `unwind_successors` (498) — 🟢 — two arms, each reading the
  constructor's own fields; canonical primitive on `UnwindDest`.

**`extdeps/languages/dag.dag`**
- `dag_wave1_e0_void_lex` (78), `dag_wave1_g0_void_grammar` (88),
  `dag_language_model_wave1_void` (98) — 🟢 — pure data constructors
  (zero-production `Conj` leaves); no derived operation.

**`extdeps/languages/rust.dag`, `go.dag`, `python.dag`, `cpp.dag`,
`verilog.dag`, `typescript.dag`, `ptx.dag`, `machine_code.dag`, `lean.dag`**
— pure type / data declarations (LanguageModel grounding); zero `fn`
bodies. 🟢 for all five dissolution classes.

**`extdeps/formats/*.dag`** (`spice`, `toml`, `yaml`, `json`, `csv`,
`openapi`, `json_schema`) — pure data models (RFC grammars as data).
Zero `fn` bodies. 🟢.

**`extdeps/frameworks/react.dag`**, **`extdeps/coordination.dag`**,
**`extdeps/file_system.dag`**, **`extdeps/process.dag`** — pure data
models (scaffolds or filled). Zero `fn` bodies. 🟢.

---

#### Summary

| class | 🔴 fix-now | 🟡 substrate-sequencing (tracked) | 🟢 clean |
|---|---|---|---|
| walker | — | 3 (`std/node.dag node_well_formed`, `std/algebra.dag free_monoid_length`, `std/float.dag nat_compare`) | rest |
| traverse | — | 4 (`std/node.dag` × 4: `all_edges_named`, `all_edges_positional`, `name_occurrences`, `all_names_distinct`) | rest |
| predicate | 3 (`compiler/01_tokenize.dag lex_rules_node_is_conj_empty_root`, `compiler/02_parse.dag grammar_node_is_conj_empty_root` — same duplicate; `extdeps/languages/llvm_ir.dag terminator_is_catchswitch`) | 5 (`std/node.dag connective_edge_discipline`, `std/float.dag float_finite_magnitude_zero`, `extdeps/languages/llvm_ir.dag` × 3: `block_well_formed`, `feature_disposition`, `llvm_instruction_cost`) | rest |
| carrier | — | — | lane-wide |
| emit/template | — | — | lane-wide |

**Substrate primitives the 🟡 tier names as missing** (the tracked,
upstream obligations — the conditions for landing each 🟡 per Practice 10's
disposition legend):

1. `fold_node` — `Node` catamorphism in `std/node.dag` (closes
   `node_well_formed`).
2. `fold` / `cata` over `FreeMonoid<T>` and `Nat` in `std/algebra.dag` /
   `std/nat.dag` (closes `free_monoid_length`, `nat_compare`).
3. `all`, `count_where`, `unique` over `FreeMonoid<T>` in
   `std/collection.dag` (closes the four `std/node.dag` traverse findings).
4. `nat_is_zero : Nat -> Bool` primitive (closes
   `float_finite_magnitude_zero`).
5. Property-projection model facts (registry row 7) for: `Connective`
   discipline, `FidelityFeature` disposition, `LlvmInstruction` cost,
   `Terminator` well-formedness invariant. Each is a "carry the fact on
   the constructor" substrate move — same shape, four occurrences.

**Fix-now (🔴) inventory** — three findings collapse to two distinct fixes:

1. `is_empty_conj_root : Node -> Bool` — extract into `std/node.dag`,
   replace both compiler `01_tokenize.dag` and `02_parse.dag` duplicates.
2. Inline `terminator_is_catchswitch` into `block_well_formed` in
   `extdeps/languages/llvm_ir.dag`; delete the discriminator predicate.

Per Practice 10, the matching in-file `.dag` tag lands with the fix
(per migration PR), not retro-applied here. This inventory is the
classification.
