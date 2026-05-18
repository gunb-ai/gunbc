# v4 retroactive dissolution audit — consolidated inventory

Per `docs/modeling-discipline.md` Practice 10 (the derived-operations
registry and dissolution-findings family) **and** the unified
Dissolution dispositions vocabulary in PR #3244 (Practice 4 head, used
by Practices 4 / 8 / 10). Rework-tracker PR #3240 task C1.

**Status — DRAFT marks pending #3244 merge.** Symbol assignments below
already use the #3244 form (`🔴 dissolve-now` / `🟢 terminal` / `🟡 gated`
with `feature:`/`consumer:` gate kind, concrete named arrival, and
dissolve-on-arrival obligation). When #3244 lands, this artifact ships
as the consolidated authority for both halves of the dissolution-debt
directive; if #3244 changes a clause materially, the affected marks are
re-expressed in one follow-up commit.

Scope on `main` at `88ae56d2a` (plus the workflow sub-sweep added per
still-hawk-102 brief correction 2026-05-18 — `src/v4/workflow/*.dag`):

- **Part A** — sweep findings: `src/v4/compiler/*.dag`,
  `src/v4/std/*.dag`, `src/v4/extdeps/**/*.dag`, **`src/v4/workflow/*.dag`**,
  classified across the five dissolution-finding classes
  (walker / traverse / predicate / carrier / emit-template). Coproduct
  dissolution is out of scope — already enforced via the per-coproduct
  emoji + `DECISIONS.md` ledger.
- **Part B** — triage of pre-existing trackers: the 10 active
  `SL-3229-*` entries in `src/v4/DECISIONS.md` Part 6 + the 13 in-file
  🟡 cite-sites across `verilog`, `llvm_ir`, `json`, `yaml`, `toml`,
  `typescript`. Triage uses **four** dispositions:
  - **VALID-🟡** — stays 🟡, re-expressed in #3244 form (`feature:` /
    `consumer:` gate + concrete named arrival + dissolve-on-arrival
    obligation).
  - **STALE → 🔴** — the named arrival has *already landed*; the 🟡 is
    debt that should already be paid.
  - **VAGUE** — gate present but not concrete (a class of arrivals
    rather than one named owner+task; "later substrate"; "the parse
    body's job").
  - **INVALID-GATE** — the gate's named arrival was *cancelled or
    reshaped by a design reversal*; the thing referenced will not
    arrive as named. Distinct from VAGUE (gate too loose to check) and
    STALE (gate opened): the gate is well-formed but its target no
    longer exists in the form the entry promised. Needs re-gating
    against the post-reversal model.

**C1's role is mark + flag, not fix.** This inventory reports the
disposition of each entry; the actual re-gating of VAGUE /
INVALID-GATE entries, and the dissolve-now PRs for 🔴 findings, are
**downstream lane work** — each owned by the lane that owns the file
named, triggered by the operator's audit of this inventory. C1 does
*not* edit `DECISIONS.md` rows, *not* rewrite in-file 🟡 blocks, and
*not* land the dissolve-now fixes. Marks live in this artifact only
(Practice 10: the in-file `.dag` tag rewrite lands with each migration
PR, not retro-applied here).

**Sweep frame is `main`, not in-flight branches.** PR #3213 (workflow
T-20 + T-24) is HELD with its own dissolution pass in progress; its
helpers reach `main` only when #3213 merges. If #3213 merges before C1
ships, C1 picks them up from `main` at that point; if not, #3213's
helpers are covered by its own pass — no double-counting.

---

## Part A — sweep findings (new)

### A.1 Lane-wide

- **Carrier dissolution** — 🟢 lane-wide. No local
  `... { value: T } | ... { diagnostic: Diagnostic }` clone of
  `Outcome<T>` outside `std/diagnostic.dag`; every consumer imports the
  std carrier.
- **Emit/template dissolution** — 🟢 lane-wide. No literal
  `template: "..."` field in any v4 `.dag`; emit substrate is
  grammar-as-data (`extdeps/languages/*.dag` LanguageModel +
  `compiler/05_emit.dag`).

### A.2 `src/v4/std/`

**`std/node.dag`**
- `node_well_formed` (113) — 🟡 **walker** —
  `feature: fold_node (std/node.dag Node catamorphism — substrate-extension under T-1)`.
  Hand-rolled structural recursion via
  `fold(n.children, …, fn(acc, e) { … node_well_formed(e.target) })`;
  dissolves to `fold_node(n, …)` on arrival.
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

### A.3 `src/v4/compiler/`

**`compiler/01_tokenize.dag`**
- `lex_rules_node_is_conj_empty_root` (76) — 🔴 **predicate** —
  `match rules.kind { TypeNode{ Conj } => count(children)==0 ; … => false }`.
  A naked discriminator-plus-empty check. **Duplicated** verbatim in
  `compiler/02_parse.dag grammar_node_is_conj_empty_root` (73). The
  model fact "this Node is an empty conj-rooted TypeNode" belongs once,
  in `std/node.dag`. Dissolve now (substrate primitive
  `is_empty_conj_root : Node -> Bool` does not require an absent
  feature — it composes existing `std/node.dag` primitives).
- `tokenize` (83) — 🟡 — `feature: lexical-walk substrate (T-6 — TASKS.md)`.
  Three-arm Wave-1 scaffold cascade; full lexical walk unrealized.

**`compiler/02_parse.dag`**
- `grammar_node_is_conj_empty_root` (73) — 🔴 **predicate** — duplicate
  of the tokenize finding; same dissolve-now (extract to
  `std/node.dag`).
- `parse` (80) — 🟡 — `feature: parse-walk substrate (T-7 — TASKS.md)`.
  Mirror of `tokenize` scaffold.

**`compiler/00_compile.dag`**, **`03_normalize.dag`**, **`03_resolve.dag`**,
**`04_infer.dag`**, **`05_emit.dag`**, **`05_eval.dag`** — scaffolds (no
fns). 🟢.

### A.4 `src/v4/extdeps/`

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
- `llvm_instruction_cost` (584) — 🟡 **predicate** (property projection) —
  `feature: cost-of-instruction model fact / lens in lens/cost.dag (T-12 — TASKS.md)`.
  22-arm cost table; cost IS a fact per instruction. The named owner
  `lens/cost.dag` exists as a scaffold today.
- `block_successors` (505), `unwind_successors` (498) — 🟢 — each arm
  reads its own constructor fields; constructor-driven projection, not
  a `match`-to-derive of a pre-existing fact.

**`extdeps/languages/dag.dag`**
- `dag_wave1_e0_void_lex` (78), `dag_wave1_g0_void_grammar` (88),
  `dag_language_model_wave1_void` (98) — 🟢 — pure data constructors.

**Other language files** (`rust`, `go`, `python`, `cpp`, `verilog`,
`typescript`, `ptx`, `machine_code`, `lean`) — zero `fn` bodies. 🟢
for all five dissolution classes (carrier and emit-template covered
lane-wide).

**`extdeps/formats/*.dag`** (`spice`, `toml`, `yaml`, `json`, `csv`,
`openapi`, `json_schema`) — zero `fn` bodies. 🟢.

**`extdeps/frameworks/react.dag`**, **`extdeps/coordination.dag`**,
**`extdeps/file_system.dag`**, **`extdeps/process.dag`** — zero `fn`
bodies. 🟢.

### A.5 `src/v4/workflow/`

Sweep frame `main` @ `88ae56d2a`. (PR #3213 fills both files with
helper logic; that work is on the #3213 branch only, not in this sweep
— covered by #3213's own dissolution pass.)

**`workflow/bootstrap.dag`** — scaffold on `main` (84 lines, all
header prose + `module v4.workflow.bootstrap` declaration; zero `type`,
zero `data`, zero `fn`). 🟢 across all five finding classes.
**`workflow/ci.dag`** — scaffold on `main` (43 lines; same shape).
🟢 across all five finding classes.

Carrier and emit-template covered by lane-wide 🟢 (A.1).

---

## Part B — triage of pre-existing trackers

### B.1 `DECISIONS.md` Part 6 SL-3229-* triggers

Each row reports: the existing tracker's named arrival → triage result
(VALID / STALE / VAGUE) → re-expression in #3244 vocabulary (if VALID
in shape, only the *form* changes; if VAGUE, the gate needs concretizing).

**`SL-3229-INTEGER-GROUP-COMPLETION`** — `GroupCompletion<M>`
constrained-inhabitance gap. Named arrival: "v4 lands constrained
generic parameters / inhabitance bounds." Verified against
`std/nat.dag` / `std/algebra.dag` on main: no
`<M> where M : CommutativeMonoid<_>` syntax landed.
- **Triage: VALID.**
- **#3244 re-expression:** `🟡 gated — feature: constrained generic parameters / inhabitance-bound syntax (substrate extension; no owning task yet — needs an owning T-#)`.
- **Sub-flag:** the *owning task* is not enumerated; the gate names the
  feature but not the task. Under #3244 the missing-owning-task half
  makes this VAGUE-on-the-task — a minor concretize-required follow-up
  on this entry (a feature-gated 🟡 with no owning task is not yet a
  pre-committed obligation).

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
productions." Verified: T-4 Verilog `LanguageModel` constant_expression
sub-grammar has not landed.
- **Triage: VALID.**
- **#3244 re-expression:** `🟡 gated — feature: constant_expression sub-grammar in extdeps/languages/verilog.dag bundled LanguageModel (T-4 Verilog fact-bundle Phase-3)`.

**`SL-3229-VERILOG-COST`** — raw-`Int` Verilog cost axes
(`VerilogCost`). Named arrival: cardinality refinement (T-3).
- **Triage: VALID.**
- **#3244 re-expression:** `🟡 gated — feature: bounded-non-negative-Int refinement in std/cardinality.dag (T-3 Wave-A2)` (same family as the four LLVM/PTX cost/width gates above).

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

### B.2 In-file 🟡 cite-sites

Result of `grep -n "🟡" src/v4/**/*.dag` (excluding `verification.dag:128`,
which is descriptive prose about the 🟢/🟡/🔴 convention itself).

**`extdeps/languages/verilog.dag` × 5 cite-sites** (lines 24, 174, 207,
264, 473) — all read `// 🟡 coproduct dissolution — DECISIONS.md Part 6 ·
SL-3229-VERILOG-D3200.`
- **Triage: inherits VAGUE from `SL-3229-VERILOG-D3200` (B.1).**
- **Action queued:** when the SL-3229-VERILOG-D3200 entry is
  re-concretized, all 5 cite-sites update to the new gate text.

**`extdeps/languages/llvm_ir.dag:28`** — `// 🟡 coproduct dissolution —
DECISIONS.md Part 6 · SL-3229-LLVM-WIDTH.`
- **Triage: VALID** (inherits from B.1 SL-3229-LLVM-WIDTH VALID).
- **#3244 re-expression at cite-site:** can stay as-is (the ledger row
  itself is the authority; the in-file one-liner is a pointer). If the
  operator-mandated form requires the gate kind on the cite-site too,
  cite-site becomes `// 🟡 coproduct dissolution — feature:
  std/cardinality.dag refinement (T-3) — DECISIONS.md Part 6 · SL-3229-LLVM-WIDTH.`

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

### B.3 Summary table

| tracker | shape | gate-kind | concrete-arrival? | dissolve-on-arrival obligation? | triage |
|---|---|---|---|---|---|
| SL-3229-INTEGER-GROUP-COMPLETION | DECISIONS.md row | feature | partial (feature named, owning task TBD) | yes | VALID (concretize owning task) |
| SL-3229-LLVM-WIDTH | DECISIONS.md row | feature | yes (cardinality.dag T-3) | yes | VALID |
| SL-3229-LLVM-OPS | DECISIONS.md row | feature | yes (cardinality.dag T-3) | yes | VALID |
| SL-3229-PTX-DIM3 | DECISIONS.md row | feature | yes (cardinality.dag T-3) | yes | VALID |
| SL-3229-PTX-COST | DECISIONS.md row | feature | yes (cardinality.dag T-3) | yes | VALID |
| SL-3229-VERILOG-NONEMPTY | DECISIONS.md row | feature | yes (collection.dag Wave-A2) | yes | VALID |
| SL-3229-VERILOG-D3200 | DECISIONS.md row | consumer | **no** (class, not one name) | yes | **VAGUE** |
| SL-3229-VERILOG-VECTOR-RANGE | DECISIONS.md row | feature | yes (T-4 Verilog constant_expression) | yes | VALID |
| SL-3229-VERILOG-COST | DECISIONS.md row | feature | yes (cardinality.dag T-3) | yes | VALID |
| SL-3229-FLOAT-NOMINAL | DECISIONS.md row | feature | **partial** (straddles machine.dag + cardinality.dag) | yes | **VAGUE** |
| verilog.dag × 5 in-file cite-sites | one-liner | (inherits SL-3229-VERILOG-D3200) | — | — | **VAGUE** (inherits) |
| llvm_ir.dag:28 in-file cite-site | one-liner | feature | yes (inherits SL-3229-LLVM-WIDTH) | yes | VALID |
| json.dag × 3 in-file blocks | prose | mixed | **no** (class) | yes | **VAGUE** |
| yaml.dag × 6 in-file blocks | prose | mixed | **no** (class) | yes | **VAGUE** |
| toml.dag × 7+ in-file blocks | prose | mixed | **no** (class) | yes | **VAGUE** |
| typescript.dag × 4 in-file blocks | prose | feature (D2-shaped) | **named arrival cancelled by D2-reversal** | yes | **INVALID-GATE** (re-gate post-reversal) |

Counts: **8 VALID-🟡** (7 DECISIONS.md rows + 1 in-file cite-site —
`llvm_ir.dag:28`). **2 VAGUE DECISIONS.md rows** (`SL-3229-VERILOG-D3200`,
`SL-3229-FLOAT-NOMINAL`) + **~16 VAGUE in-file prose blocks** across
`json.dag`, `yaml.dag`, `toml.dag`, and the 5 `verilog.dag` cite-sites
inheriting VERILOG-D3200 ≈ **~18 VAGUE total**. **4 INVALID-GATE**
(typescript.dag D2-shaped gates). **0 STALE → 🔴** — no named arrival
has already landed.

**Headline finding:** no pre-existing tracker is STALE — the
cardinality-refinement / collection Wave-A2 / constrained-generics
arrivals are uniformly still ahead of us, verified against
`std/cardinality.dag`, `std/collection.dag`, `std/nat.dag`,
`std/algebra.dag` on `main` @ `88ae56d2a`. The pre-existing-tracker
debt is **overwhelmingly VAGUE prose-form gates that #3244 retires** —
concretizing the in-file prose blocks under one canonical
`feature: std/cardinality.dag refinement (T-3 Wave-A2)` (and a smaller
parser-side family `feature: T-6/T-7 parse/emit pipeline-stage
substrate`) collapses much of the VAGUE list to VALID. The 4
typescript.dag INVALID-GATE entries need substantive re-gating against
the post-D2-reversal fact-bundle model. **All re-gates and dissolve-now
fixes are downstream lane work, not C1's** — C1 marks and flags.

---

## Part A summary

| class | 🔴 dissolve-now | 🟡 gated (named feature: / consumer:) | 🟢 terminal |
|---|---|---|---|
| walker | — | 3 (`std/node.dag node_well_formed` → `fold_node`; `std/algebra.dag free_monoid_length` → `fold FreeMonoid`; `std/float.dag nat_compare` → `fold Nat`) | rest |
| traverse | — | 4 (`std/node.dag` × 4: → `forall` / `count_where` / `unique` over `FreeMonoid<T>` in `std/collection.dag` Wave-A2) | rest |
| predicate | 3 (`compiler/01_tokenize.dag` + `compiler/02_parse.dag` literal duplicate of `is_empty_conj_root`; `extdeps/languages/llvm_ir.dag terminator_is_catchswitch`) | 5 (`std/node.dag connective_edge_discipline` → per-`Connective` discipline fact; `std/float.dag float_finite_magnitude_zero` → `nat_is_zero`; `extdeps/languages/llvm_ir.dag` × 3: `block_well_formed` → cascade of dissolve-now; `feature_disposition` → per-feature disposition fact (T-4 fact-bundle); `llvm_instruction_cost` → `lens/cost.dag` T-12) | rest |
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
   fact-bundle), `LlvmInstruction` cost (lens/cost.dag T-12),
   `Terminator` well-formedness (cascade of the LLVM
   `terminator_is_catchswitch` dissolve-now).

**Dissolve-now (🔴) inventory** — three findings collapse to two
distinct fixes:

1. `is_empty_conj_root : Node -> Bool` — extract into `std/node.dag`,
   replace both `compiler/01_tokenize.dag` and `compiler/02_parse.dag`
   duplicates. Pure composition over existing `std/node.dag` primitives,
   no absent feature blocks the fix.
2. Inline `terminator_is_catchswitch` into `block_well_formed` in
   `extdeps/languages/llvm_ir.dag`; delete the discriminator predicate.

Per Practice 10, the matching in-file `.dag` tag lands with the fix
(per migration PR), not retro-applied here. This inventory is the
classification.
