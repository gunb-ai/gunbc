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

Producers inspected, named per DESIGN §6b root generalization: **TODO-FROM-TABLE** -- `body_lower_type_wrap_instantiation`,
`body_lower_record_construct` / `construct_node`, `body_lower_anonymous_record_type_optional`,
`body_lower_type_expr_lowered_optional`, `infer_type_instantiated`, `infer_frame_instantiated`,
`infer_coproduct_type_applied_open`, `resolve_instantiation_children`, `symbol_index_fill_declared_child`,
`declaration_denotation_of_type`, `type_application_conformance`. The repaired link is: **TODO-FROM-TABLE**.

Repaired **once**, at the producer. No label-normalizing step was added inside the comparison: that
would be a symptom-link patch.

## Stated deviation from the brief (DESIGN §3b)

The brief said "the declared-parameter label is the authority; positional is the leaf". The
substrate rules the other way and it wins: `v2.std.node` sets
`connective_edge_discipline(Instantiation) = PositionalEdges` and `edges_conform` for
`PositionalEdges` requires `all_edges_positional` **strictly**, so an `Instantiation` whose
argument edges carry the alias's member names is ill-formed by the substrate's own
`well_formed`. Those member names are `Authored { name }` edges - minted by `v2.std.node`'s
`edge_declared` through `v2.std.node_query`'s `binder_edge_declared` - and **not**
`StructuralLabel`: on main at 3afbd9a62bd1179bdbcc7fa6cf47ba50c452452d the string
`StructuralLabel` does not appear anywhere under `src/v2/compiler/**/*.dag` (`git grep -n
"StructuralLabel" 3afbd9a62bd -- "src/v2/compiler/**/*.dag"` returns nothing), and no `Edge`
construction there takes a variable label (the one `label: [a-z_]` hit is a comment in
`src/v2/compiler/self_host/stage0_crate_layout.dag`). The brief's word "structural" describes
the role those labels play, not the `EdgeLabel` arm they carry. `connective_edge_discipline` is
**not** widened. The repair is that both sides present the application through **one** nominal-
application lowering; where one side stood for the same nominal under a second connective, that
was a §3 meaning fork -- **this change deletes the second presentation, it does not translate
between them**.

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
  discipline; the two sides conform to **one another's** (mutual conformance is how "one discipline" is
  stated without hard-coding `Positional`, and a second presentation of one nominal fails it); child
  `Edge` labels are equal at every argument index. A missing side **fails** rather than passing vacuously.

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
  `5b02c6dd3d963cb9863b163a69cba56e900f8698` (42 cells), receipts **TODO-ATTACH** (raw verdict lines,
  ledger TSV, binary sha before/after, memory peak, `claim_rc`).
- Execution of the carried claims: targeted claim run **TODO-GRANT** (the required lane executes no
  claims -- `gunbc.rung_drop v1_required_lanes_withdrawn`), plus required CI.
- RFM: `dag/gunbc/recurring_failure_mode/nominal_application_child_edges_disagree_between_producers.dag` **TODO** -- written *from* the measured table, with
  specimen receipts naming date/session/head; the class starts at `RungUnread { cause: NoRungStated }`.

## Coordination

- #15868 (`valiant-deer-27`) carries its own site-6 claim; **whichever of the two PRs lands second
  performs the site-6 re-point**, under one writer.
- No host runs, no in-container builds; all execution through CI or a named grant.
