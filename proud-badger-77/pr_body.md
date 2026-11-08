# v2 lowering: one nominal type-application presentation, so both sides carry the same child-edge labels

Node: `adhoc-afb0f800-78c`. Base: `main`. Not stacked on #15868.

## What breaks today

A nominal type-application is produced by **two** lowerings. The node standing in a declared position
carries one set of child `Edge` labels; the node standing in the produced position for the *same*
application carries another. `v2.std.exact_structural_equality_zip_fold_predicate` decides a child pair
by its **Edge label before its target**, and drops the head edge before zipping the rest by ordinal. A
label divergence therefore decides the comparison at that index and never descends into the target --
so a **lowering mismatch is reported as a type mismatch on every nominal application**. That is silent
wrongness (DESIGN §5), not a diagnostic-quality bug: `Pair<Int, String>` against a value that really
*is* `Pair<Int, String>` refuses.

## Why these cells and not just the behaviour
Scoped observation (a search, not a proof - DESIGN §6): on main at 3afbd9a62bd1179bdbcc7fa6cf47ba50c452452d,
    git grep -l "all_edges_positional" 3afbd9a6... -- 'src/v2/test/claim/**/*.dag'        # no files
    git grep -l "connective_edge_discipline" 3afbd9a6... -- 'src/v2/test/claim/**/*.dag'  # no files
So **no claim module names either symbol**. The discipline is nonetheless reachable indirectly: three claim
modules do name `edges_conform` (`emit/produced_item_parameter_order`, `emit/realized_closure_parameter_order`,
`self_host_contract/normalize_emitted_contract`, one call site each), and a cell can constrain the discipline
without naming it, so this search does not show that the discipline is unexercised - only that nothing names the
two symbols that decide this bug. What the five contract cells add is therefore specific: they assert, through
the substrate's own predicates (`node_edge_discipline` and `edges_conform`, never a hard-coded arm), that the
two sides of one nominal application conform to **one** discipline and carry equal labels per argument index -
a proposition no existing cell states, whether or not any existing cell happens to lean on the discipline.

## The slice (producer -> consumer)

Consumer: `exact_structural_child_pair` (label before target) via `inhabitance.dag`'s
`exact_structural_comparison`, which also builds the refusal position from the answering arm's
`argument_ordinal_path`.

Producers inspected, named per DESIGN §6b root generalization (the inspection is complete; only
*which link is repaired* awaits the measured table, because choosing it from a reading is exactly the
shortcut that produced silent wrongness here): `body_lower_type_wrap_instantiation`,
`body_lower_record_construct` / `construct_node`, `body_lower_anonymous_record_type_optional`,
`body_lower_type_expr_lowered_optional`, `infer_type_instantiated`, `infer_frame_instantiated`,
`infer_coproduct_type_applied_open`, `resolve_instantiation_children`, `symbol_index_fill_declared_child`,
`declaration_denotation_of_type`, `type_application_conformance`. The repaired link is: **TODO-FROM-TABLE**.

Repaired **once**, at the producer. No label-normalizing step was added inside the comparison: that
would be a symptom-link patch.

## Stated deviation from the brief (DESIGN §3b)

The brief said "the declared-parameter label is the authority; positional is the leaf". The substrate
puts a limit on that, and the measurement decides which side of the limit we are on, so this section
states the limit rather than a conclusion.

`v2.std.node`'s `connective_edge_discipline` assigns `PositionalEdges` to `Instantiation`, and
`edges_conform` for `PositionalEdges` requires `all_edges_positional` strictly. So **one** shape is
condemned outright: a node whose connective is `Instantiation` while its child edges carry `Authored`
labels. It is ill-formed by the substrate's own `well_formed`, and no measurement can license it.

That is all the mandate settles. It does **not** condemn a `Conj` carrying `Authored` member labels --
`LabeledEdges` is precisely what `connective_edge_discipline` requires of `Conj`, so such a node is
legal on its own terms. The predicted failure mode is a **kind** mismatch: one side alias-unfolded into
a `Conj` with member labels, the other left an `Instantiation` with positional edges. In that outcome
neither node breaks any discipline, the mandate selects nothing, and the question is *where
normalization is applied* -- a different fix from the lowering. Whether this PR is a lowering repair or
a normalization repair is therefore decided by the measured table, not by this paragraph, and the PR
will not be written before that table exists.

Those member names are `Authored { name }` edges - minted by `v2.std.node`'s `edge_declared` through
`v2.std.node_query`'s `binder_edge_declared` - and not `StructuralLabel`. The scoped negative, with its
command and revision: `git grep -n "StructuralLabel" 3afbd9a62bd1179bdbcc7fa6cf47ba50c452452d --
"src/v2/compiler/**/*.dag"` returns nothing, so **no minting site was found in the compiler tree** --
which is a narrower claim than the arm not existing, because it does exist: `v2.std.node` declares
`EdgeLabel` with three arms (`Positional`, `Authored { name }`, `StructuralLabel { label }`) and mints a
structural one itself in `arrow_signature_order_label`. The brief's word "structural" describes the role
those labels play, not the `EdgeLabel` arm they carry. `connective_edge_discipline` is **not** widened.
Where the table shows one side standing for the same nominal under a second connective, that is a §3
meaning fork and **this change deletes the second presentation; it does not translate between them**.

## Claims carried

`src/v2/test/claim/compiler/nominal_application_child_edge_contract.dag` -- permanent regression
controls, not a probe: they survive the terminal architecture and are consumed by it (§6 survival test),
which is why no dissolution condition is stated.

- **positive controls**: identical applications are *accepted* (`Pair<K, String>` vs `Pair<K, String>`,
  `Pair<Int, String>` vs `Pair<Int, String>`) -- a correct program being refused is a soundness regression.
- **discriminators**: same head, divergence at **argument index 1** (`Pair<K, String>` vs `Pair<K, Int>`,
  `Pair<Int, String>` vs `Pair<Int, Int>`), plus a nested variant (`Pair<List<K>, String>` vs
  `Pair<List<K>, Int>`), each with its accepted twin one term away.
- **contract cells**, asserting through the substrate's own authority (`node_edge_discipline`,
  `edges_conform`) and never naming a discipline: each side conforms to its **own** found connective's
  discipline; the two sides are the **same kind of node**, so the discipline the substrate mandates for
  them is one value (`kind_edge_discipline` is a function of kind and child count, and the kind test is
  the same comparison `node_equal_ignoring_provenance` makes when it compares kinds first); child
  `Edge` labels are equal at every argument index. A missing side **fails** rather than passing vacuously.
  An earlier formulation asked each side to conform to the **other's** discipline; that was dropped,
  because it is unsatisfiable as soon as the kinds differ -- no child set satisfies `LabeledEdges` and
  `PositionalEdges` at once -- so it refused by definition and measured nothing. A claim that can never
  pass is not a conservative claim, it is a blind one.

The measurement-only `nal_node_*` cells from the instrument branch are deliberately **not** carried: by
their own review they must never enter main, where they would be unconsumed declarations (§3c).

## Declared frontier (not dropped, not weakened)

`Pair<K, String>` vs `Pair<K, Int>` refusing as **`roots_differ` at path `[1]`** -- trigger: **#15868
merges**; carrier: **the second PR to land, the same writer as the site-6 re-point**. It needs the
`ComparisonRootsDiffer{path}` arm, which does not exist on `main`; this PR does **not** fork that verdict
type to work around its absence. On `main` a refusal carries no argument index at all: the verdict reason is
chosen by position alone by `v2.std.inhabitance`'s `inhabitance_no_candidate_reason`, and the structural test
behind it is the Bool `v2.std.node`'s `node_equal_ignoring_provenance`. What is asserted here is stronger
than a class name and needs no arm.

## Evidence

- Measurement of the mismatch: the granted hermetic `claim_batch` run of the instrument at head
  `021bbda11cfb0a15dcb200151a0389293eb9dc9d` (42 cells) -- that head is `5b02c6dd3d963cb9863b163a69cba56e900f8698`
  fast-forwarded by one commit, which repairs the instrument's own synthesised prelude: its `std.algebra`
  fixture spelled an opaque nominal `opaque type FreeMonoid<element>`, which cannot parse (no keyword
  carries that text, and `dag_grammar_type_decl_expr` requires the declaration to begin with the `type`
  keyword), and because every fixture imports `std.types`, which imports `std.algebra`, a broken
  `std.algebra` would have left the whole roster resolving against a missing prelude. Grammar acceptance
  of the corrected spelling is a **static** judgement from `fn dag_grammar_type_decl_expr` (keyword, name,
  optional generics, modifier slot, optional body) plus the bodyless lowering arm
  `body_lower_eq_type_decl` accepting `type_opaque_wrapper(binders)`; it is **not** execution evidence,
  and no fixture string of this exact shape has ever been executed, because the required lane runs no
  claims. Receipts **TODO-ATTACH** (raw verdict lines,
  ledger TSV, binary sha before/after, memory peak, `claim_rc`).
- Execution of the carried claims: targeted claim run **TODO-GRANT** (the required lane executes no
  claims -- `gunbc.rung_drop v1_required_lanes_withdrawn`), plus required CI.
- RFM: `dag/gunbc/recurring_failure_mode/nominal_application_child_edges_disagree_between_producers.dag` **TODO** -- written *from* the measured table, with
  specimen receipts naming date/session/head; the class starts at `RungUnread { cause: NoRungStated }`.

## Coordination

- #15868 (`valiant-deer-27`) carries its own site-6 claim; **whichever of the two PRs lands second
  performs the site-6 re-point**, under one writer.
- No host runs, no in-container builds; all execution through CI or a named grant.
