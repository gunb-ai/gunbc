# Seat 15 design note: split/join ReceiverSelf under ContainerOf (NOT FOR MERGE)

Status: HELD by keen-heron-809 pending admission by sleek-owl-397. No infer.dag edits until admitted.

## Reach
No N7 member (v2/test/claim/parse/expression_bodied_fn_decl_parse:all) mentions split/join. N7 wall is list_push at resolve. This is typing-floor work.

## Current native behavior (fail-closed, typed)
A split/join call head resolves to std.primitives split_contract/join_contract; infer refuses it
(infer_application_callee_is_primitive_authority -> infer_reason_primitive_without_declared_signature_authority).
fold_lowering collection_roster_template_type answers Absent for ContainerOf. Roster rows are concat/count only.

## Authority
std.primitive_identity primitive_signature(PrimitiveSemanticContract) -> SignatureResolved{parameters,result} is the
signature authority; it has zero v2 infer consumers. Roster rows for split/join would be a second path beside it.

## Profile question (answered)
std.primitives PrimitiveContract = {name, work, output_size, certainty, carrier_cost}: NO profile, NO signature.
The profile lives only on std.primitive_identity PrimitiveSemanticContract {canonical_name, signature: GroundedInAlgebraTemplate{profile}}.
`join` has 3 template rows (boolean algebra, collection, free-monoid scalar), so the consumer needs a PrimitiveContract.name ->
PrimitiveSemanticContract join (declared definitions in primitive_identity). If that join does not exist for split/join it is a
modeling gap to report, never to guess.

## Design
1. One template->type reader (generalize fold_lowering collection_roster_template_type) with a general ContainerOf arm:
   Named List -> List<reader(element)>, SameAsReceiver -> carrier container; OptionalOf/TupleOf by the same rule; unreadable arms stay Absent (refusal). Roster concat/count call the same reader.
2. infer consumes primitive_signature for a primitive-contract callee: SignatureResolved -> Arrow via the reader; Ambiguous/Absent/NotGrounded keep the typed refusal.
3. Nothing else deleted.

## Tests
Base: untracked src/v2/test/claim/compiler/infer_receiver_self_under_container_witness.dag in worktree jolly-dove-417 (9 tests; swap point rsu_algebra_peer_source).
RED (assemble+infer): split result at Bool; split-derived element at Int; join over List<Int>; List<String> at Bool/enum.
POSITIVE: split element at String; join over List<String>; List<String> at List<String>.
MUTATION: drop ContainerOf arm -> positives refuse; make element opaque -> REDs accept.
