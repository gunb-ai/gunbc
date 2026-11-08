# Reading order for the #16061 rev7 table (42 cells, one batched claim run)

Gate 0 - receipts first (my checker must say OBSERVED): OUTCOME=OBSERVED rc=0 table_complete 42/42,
bare PASS/FAIL per cell, ledger TSV (gunbc-terminal-ledger/2), bin.sha256.before == after,
scope stdouts incl PREP_OK, claim_rc, no oom_kill rise, memory.peak.
Anything missing/suffixed/killed/no OUTCOME line = NO_OBSERVATION -> report, do NOT infer the fork,
do NOT write the fix. (closure.txt / closure.err no longer exist in rev7 - absence is NOT an anomaly.)

Step 1 - NODE reachability rows (4): nal_node_concrete_declared_side_is_reachable_holds,
nal_node_concrete_produced_side_is_reachable_holds, nal_node_generic_declared/produced_side_is_reachable_holds.
If a side is unreachable, every ARM/NODE row about that side is vacuous: say so, do not read it as
"positional absent by choice".

Step 2 - ARM rows decide the arm each side's FIRST ARGUMENT edge carries (exactly one of the three
passes per side/subject; all three failing = unreachable, go back to Step 1):
  subjects: concrete = gunbc.nal_concrete_ident (Pair<Int,String> vs Pair<Int,String>),
            generic  = gunbc.nal_generic_differ  (fn nal_take(p: Pair<K,String>) -> Pair<K,Int>)
  rows: nal_arm_<subj>_<produced|declared>_argument_<positional|authored|structural>_holds   (raw)
        nal_arm_generic_<produced|declared>_normalized_<positional|authored>_holds            (after denotation read)

Decision table (declared arm, produced arm) -> action:
  A. authored / positional        -> PREMISE CONFIRMED (with the label-arm correction: Authored, not
     StructuralLabel). The declared side is the alias-RHS presentation (TypeNode{Conj} +
     Authored{first,second} from body_lower_field_decl_block_payload -> binder_edge_declared).
     Repair = DELETE that second presentation as the node of an application, at its producer, so the
     declared side is the one application presentation (Instantiation + Positional, per
     connective_edge_discipline(Instantiation)=PositionalEdges). Never normalize in the comparison.
  B. positional / authored        -> same fork, mirrored producer: repair on the produced/inferred side
     (infer_type_instantiated copies labels, so the owner is whoever built that node).
  C. positional / positional, but CONTROL structural-equal cell fails -> labels are NOT the cause.
     Check *_is_instantiation rows: if kinds differ (Conj vs Instantiation) it is a KIND fork ->
     delete the Conj presentation. If kinds equal too, the divergence is at targets: stop, report,
     because the brief's premise (labels) would be wrong.
  D. authored / authored          -> both sides come from the alias path: the application presentation
     is the one that is missing; repair is still one link, but the authority claim flips. Report before
     building (brief: "if the slice shows the brief's premise is wrong, STOP and report").
  E. identical pair accepted AND differing pair refuses (CONTROL all PASS) -> nothing to fix:
     premise wrong. STOP and report; do not build.

Step 3 - denotation rows (nal_node_*_denotation_reads_holds, nal_node_*_is_instantiation_holds,
nal_node_*_arguments_positional_holds) plus the normalized ARM rows localize WHICH link:
raw declared = authored but normalized declared = positional means the alias denotation read is the
second producer - the earliest link that cannot justify its label choice is where the declared node is
handed to the comparison, and that is the single repair site.

Step 4 - only after Steps 1-3 do I touch code. One producer, one edit. Then the 12 contract cells
(src/v2/test/claim/compiler/nominal_application_child_edge_contract.dag) are the regression control,
executed by the targeted claim run (never --required-ci).

Kind-first caution (main): the consumer is node_equal_ignoring_provenance (node.dag:621) -
kind ==, then children zipped by ordinal with label == before target recursion; zip_eq is strict on
length; it returns Bool, so a false CONTROL row alone never proves WHICH test failed.

## Non-defects already cleared (do not re-investigate)
- `join([...], "")` called in my probe and contract file without an import: it is an implicit prelude
  name. Main's green claim modules do the same (src/v2/test/claim/compiler/equality_judged.dag:31,
  fold_domain_alias.dag:107, infer_sibling_position_inhabitance_witness.dag:55,
  infer_route_candidates_inhabitance_witness.dag:61, callback_argument_formal.dag:39,
  cli/population_verbs.dag:498) - no import, no local definition. The only `fn join` in the repo is an
  unrelated arity in dag/test/claim/host/boot_medium_served_readback_witness.dag:43.
- `connective_edge_discipline` / `nal_take` / `f` flagged by my callee scan: comment text, string
  literals inside fixture builders, and a lambda parameter respectively.
- Contract file structure re-checked on the unpushed branch: 12 `test fn` cells, 32 in-file defs,
  45 imported symbols, all resolving on current origin/main; merge-tree clean.

## Fixture semantics (probe, for reading rows correctly)
The probe does NOT use the repo's std.types. It ingests synthetic module sources:
  nal_algebra_source = "module std.algebra\n\nopaque type FreeMonoid<element>\n"
  nal_types_source   = "module std.types\n\nimport std.algebra { FreeMonoid }\n\n
                        type List<element> = FreeMonoid<element>\n\n
                        type Pair<A, B> = Mk { first: A, second: B }\n"
and the subject modules do `import std.types { Pair, List }` against that pool, so there is no
duplicate-declaration conflict. Deliberate control: Pair is a transparent alias whose body is a PRODUCT
(members carry first/second), List is a transparent alias of an OPAQUE nominal (no member labels at all).
"A mismatch that appears for Pair and not for List is the member labels." Subjects:
  nal_concrete_ident  fn nal_take(p: Pair<Int,String>) -> Pair<Int,String>   (accepted twin)
  nal_generic_differ  fn nal_take<K>(p: Pair<K,String>) -> Pair<K,Int>       (the DONE case, expects [1])
  nested variants use Pair<List<K>, ...>; the alias pair diverges at index 0 of the one-argument List.

## Sibling producers/consumers, label behavior VERIFIED on origin/main (for the PR body's DESIGN 6b list)
- construct_node (v2.std.node_query:484): TypeNode{Conj} whose children are [construct_tag_edge] ++ caller's
  field_edges. It MINTS NO LABELS - labels arrive from the caller (the record/construct path). Note the
  shape: a Conj construct is [tag, first, second] while an Instantiation application is [head, arg0, arg1],
  so an ordinal zip between the two pairs tag<->head, first<->arg0, second<->arg1. Read ARM rows with that
  offset in mind.
- symbol_index_fill_declared_child (v2.compiler.symbol_index_fill:108): no label construction anywhere in
  its body - declared children are carried through unchanged.
- declaration_denotation_of_type (v2.std.qualified_name:633): the Instantiation branch reads
  positional_child_target_nodes(node: t) to split head vs arguments; any non-Instantiation falls to a branch
  that reads declaration_reference_path_optional on the node itself and answers optional_absent when the node
  is a bare Conj. So the APPLICATION form is what the rest of the compiler can even talk about - this is the
  strongest static support for "positional is the authority", and it is what nal_node_*_denotation_reads tests.
- type_application_conformance (v2.std.type_binder:539): for TypeParamOrderDeclared{labels} it zip_maps the
  DECLARED PARAMETER names onto the supplied arguments by ordinal, so it is insensitive to the argument edges'
  own labels; type_binder_labels_conform guards the declared-parameter names. Label authority for conformance is
  the binder's own names, not the argument edge labels - another reason the argument labels must be the one
  canonical Positional shape and nothing else.
- ARM cells read children[1] (the FIRST ARGUMENT), not child 0: nal_argument_edges drops the head/tag child
  (shape.dag:258) and nal_first_argument_edge_of takes list_at_optional(..., 0) (:448). Same semantics as the
  comparison's exact_structural_argument_children, so an authored-vs-positional verdict here is directly
  comparable to what the judge sees.
- Ambiguity to respect: nal_on_produced / nal_on_declared answer false when the side is absent (:264-275), so
  "all three arms fail" means EITHER unreachable node OR a fourth shape - resolve with the reachability rows
  before concluding anything.

## Static PREDICTION (not a conclusion) - what the decisive rows are
With no where-clause, body_lower_binder_type_carrying_optional_where returns `base` unchanged, so a parameter
annotation and a return annotation of the same spelling reach the tree through the SAME lowering
(body_lower_type_expr_lowered_optional) and should carry identical Positional argument edges RAW. The pair my
probe compares is (parameter node from the symbol index, Arrow-body target of the same fn) - the two routes a
declared annotation travels - and the refusal cells measure ^arrow_body_does_not_inhabit_declared_return.
The only inspected step that turns an application into a Conj with Authored member labels is the ALIAS
UNFOLDING used by the normalized cells: infer_type_alias_normalized_at (imported from v2.compiler.infer at
shape.dag:37, applied at :305) unfolds Pair<K,String> to `Mk { first: K, second: String }` = TypeNode{Conj} +
Authored member labels.
=> If raw ARM rows say positional/positional and the NORMALIZED rows disagree between the sides, the second
   presentation is produced by asymmetric alias normalization (one side unfolded, the other left as an
   Instantiation), and the single repair link is where normalization is applied for one side and not the
   other - NOT the lowering, and NOT the comparison.
DECISIVE ROWS, in order: (1) the four reachability rows; (2) nal_node_*_is_instantiation (raw vs normalized,
both sides); (3) nal_arm_generic_{produced,declared}_argument_* vs nal_arm_generic_{produced,declared}_normalized_*;
(4) nal_node_*_denotation_reads. CONTROL rows alone cannot separate kind-mismatch from label-mismatch because
the judge is a Bool (node.dag:621 compares kind first).
If instead raw AND normalized rows are symmetric and CONTROL passes on the identical pair, the brief's premise
is wrong for these fixtures: STOP and report before building.

## The consumer chain my fixtures actually take (traced on origin/main)
obligation with produced_declared_carrier (inhabitance.dag:690)
  -> find_witness(source_facts: carrier, candidates: inhabitance_declared_singleton(declared: ...),
       predicate: PreservationPredicate{ preservation_rule: ^preservation_rule_exact_structural_equality_zip_fold },
       multiplicity: UniqueOnly)
  -> exact_structural_equality_zip_fold = node_equal_ignoring_provenance (Bool, node.dag:621)
  -> Rejected => exact_refusal, reason by POSITION ONLY: inhabitance_no_candidate_reason(position)
     (PositionDeclaredReturn -> ^arrow_body_does_not_inhabit_declared_return, :720/:756).
Two consequences:
1. On main the refusal carries NO argument index. inhabitance_position_context_edges (:736) adds only
   Authored context edges for record/let/scrutinee positions. So "refuses as roots_differ at path [1]" cannot
   be asserted on my base at all - it needs #15868's arm. That is the declared frontier, now backed by source.
2. This obligation path does NOT normalize either side itself, while the generic instantiation path normalizes
   BOTH (infer.dag:5395-5396 infer_type_alias_normalized_at on declared and produced). Asymmetry can therefore
   only come from where each side was built before reaching here - exactly the single link the table must name.
Also verified: infer_type_alias_unfolded_at requires declaration_denotation_of_type (Instantiation form only),
and unfolding REPLACES the node with the alias RHS (Conj + Authored member labels) instead of relabelling it;
the Instantiation branch of infer_type_alias_normalized_at preserves e.label. So no function rewrites labels -
labels are only ever minted (Positional at fold:2878, Authored at binder/record paths) or copied.

<!-- INTERNAL ONLY: line numbers here are convenience anchors from one checkout. Per keen-heron (DESIGN 3), cite SYMBOLS, never file:line, in PR text, comments, and the table write-up. -->

## The two candidate single-link repairs, and what picks between them
Both sides agree on a presentation, so the repair is to make the SHARED link normalize consistently. Exactly
two one-link candidates exist, and the table plus one extra datum decides:
  R1 normalize-both: the obligation path applies infer_type_alias_normalized_at to the produced side and the
     declared side, mirroring what the generic instantiation path already does to both. Outcome: both sides
     are the alias right-hand side (Conj + Authored member labels), identical pairs match, the differing pair
     reaches targets and refuses on the second argument.
  R2 normalize-neither: both sides stay applications (Instantiation + Positional arguments), identical pairs
     match, the differing pair again decides on the second argument.
Both satisfy the brief's DONE behaviour, so DONE does not discriminate them. What does: which side currently
arrives UNMATCHED, and which presentation the rest of the compiler can talk about. declaration_denotation_of_type
answers only for the Instantiation form, so R2 keeps every downstream reader working; R1 turns every compared
node into a record and makes those readers answer absent. Absent other evidence, R2 (one presentation: the
application) is the smaller and safer change, and it is also the one the substrate discipline already encodes
(Instantiation => PositionalEdges, checked strictly). State the choice and this reason in the PR body.
FORBIDDEN regardless: making the comparison tolerate mixed labels (symptom patch), widening
connective_edge_discipline, or translating one label vocabulary into the other (two presentations kept alive).
If the table shows a THIRD shape (e.g. an Instantiation-kind node whose argument edges are Authored), neither R1
nor R2 applies: that would mean something mints an application with record-shaped children, and the repair is
at that minter - report before building, because it also means the brief's premise needs restating.

## Blast radius for the repair (measured on origin/main, one checkout)
Files touching each mechanism (claim files / non-test modules):
  positional_child_target_nodes 26 / 28   - readers; unaffected under R2, which keeps the application form
  infer_type_alias_normalized_at  4 / 1   - the concentration point for either repair
  infer_type_alias_unfolded_at    2 / 1   - same
  edges_conform                     3 / 10
  all_edges_positional              0 / 5
  connective_edge_discipline        0 / 2
Two review points from this:
1. The CI-breakage candidates are exactly the three claim modules that call the normalization machinery
   (infer_free_monoid_element_inhabitance_witness, lambda_alias_field_key, width_literal_admission, width_literal_admission_real); anything else that moves did so through the readers, which my repair
   does not touch under R2.
2. SCOPE-LIMITED (search, not proof - DESIGN 6): on main at 3afbd9a62bd, no claim module NAMES
   all_edges_positional or connective_edge_discipline (git grep -l each over src/v2/test/claim/**/*.dag, empty).
   Three claim modules DO name edges_conform (emit/produced_item_parameter_order,
   emit/realized_closure_parameter_order, self_host_contract/normalize_emitted_contract), so the discipline may
   be exercised indirectly; do NOT claim "unenforced by any test". The defensible claim: no existing cell states
   the proposition the contract cells state (both sides of one application conform to ONE discipline and carry
   equal labels per argument index). Evidence transcript: /tmp/discipline_search_evidence.txt.

## SUBSTRATE MANDATE (found statically at 021bbda1, not execution evidence)

v2/std/node.dag, fn connective_edge_discipline:
  Conj => LabeledEdges; Disj => LabeledEdges; Arrow => ArrowBodyEdges; Instantiation => PositionalEdges.
fn kind_edge_discipline(kind, child_count) routes a TypeNode through
connective_edge_discipline_for_child_count, so a nominal application's child edges are REQUIRED
by the substrate to be Positional. Therefore the side carrying Authored labels on an
Instantiation is the violator, and the repair direction is not a choice between arms: the
Positional presentation is the mandated one.

SCOPE OF THE MANDATE (corrected after keen-heron's review). The mandate condemns exactly one
shape: a node whose connective is Instantiation while its child edges carry Authored labels. It
does NOT condemn a Conj carrying Authored member labels -- LabeledEdges is what
connective_edge_discipline requires of Conj, so that node is legal on its own terms. The
predicted failure mode is a KIND mismatch: one side alias-unfolded into Conj with member labels,
the other left an Instantiation with positional edges. In that case NEITHER side violates its own
discipline, the mandate selects nothing, and the question is where normalization is applied -- a
different fix from the lowering. So the mandate decides the Authored-on-Instantiation case only;
the table still decides the repair.

FACT (correcting a possible misreading of an earlier note): EdgeLabel in v2/std/node has exactly
three arms -- Positional, Authored { name }, StructuralLabel { label } -- and StructuralLabel
subdivides into CoreMarker { marker } and ProductionEdge { language, edge }. StructuralLabel is a
real arm that the substrate mints (node.dag's arrow_signature_order_label mints
StructuralLabel { label: CoreMarker { marker: ArrowSignatureOrderEdge } }). The scoped negative is
narrower than "does not exist": no minting site was found under src/v2/compiler/**/*.dag. The
probe's structural cells are therefore well-formed and exhaustive over the three arms; they are
expected to read FAIL for these fixtures, which is data, not a defect.

Consequence for the contract cells (redesigned in 784aa2f84ea):
- own-discipline cells assert edges_conform(n.children, node_edge_discipline(n)) -- meaningful,
  because the discipline comes from kind + child_count, independent of the labels present.
- the previous "mutual conformance" helper (each side conforming to the OTHER's discipline) was
  DROPPED: it is unsatisfiable whenever the two kinds differ (no child set satisfies both
  LabeledEdges and PositionalEdges), so it refused by definition and measured nothing.
- replaced by nal_one_lowering: left.kind == right.kind (the same comparison the consumer
  node_equal_ignoring_provenance makes) plus each side conforming to its own discipline. This is
  satisfiable, so it can pass after the repair and fail before it.
- cell renamed nal_sides_conform_to_one_another_discipline_holds -> nal_sides_share_one_discipline_holds;
  /tmp/roster_contract.csv updated; 12 cells still align exactly with the roster.

## WHY THE CONTROLS, NOT THE "DIFFER" CELLS, CARRY THE DISCRIMINATION ON MAIN

Scoped negative, with command and revision:
  git grep -ho "comparison_arguments_differ\|comparison_roots_differ\|arguments_differ\|roots_differ" 021bbda1 -- src/v2   -> 0 hits
  git grep -hoE "Comparison[A-Za-z]+" 021bbda1 -- src/v2/std/inhabitance.dag src/v2/std/coercion.dag src/v2/std/node.dag  -> 0 hits
So at this revision the substrate exposes NO vocabulary for "arguments differ at index i" versus
"roots differ at path p". The refusal my differ-fixtures must produce is therefore asserted through
the only reason that exists for it, ^arrow_body_does_not_inhabit_declared_return (declared in
v2/std/inhabitance.dag and already named by two executed claim modules: algebra_operator_derivation,
body_let_annotation).

Consequence, stated so nobody over-reads a green cell: on main a "differ" cell cannot tell a refusal
for the RIGHT reason from a refusal for the WRONG one, because both collapse to the same reason
symbol. The discriminating power on main lives entirely in the POSITIVE CONTROL - the identical pair
that the brief says is currently refused on every same-head nominal application, and which must be
ACCEPTED after the repair. The path-[1] / roots_differ assertion is not expressible until #15868
lands its arm; it stays a declared frontier with its trigger ("#15868 merges") and carrier ("the
second PR to land, same writer as the site-6 re-point"), not a silently dropped obligation.
