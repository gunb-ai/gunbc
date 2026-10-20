# PR-2a (node_type_equals_core widening): can its RED be authored on an executed route?

PARENT RULE msg_3e7c1548 (b): the widening must land alone, first, "with its own RED and control".
This note asks, BEFORE building, whether such a RED exists on a route CI executes -- the PR-1 pin
taught me that a check whose red cannot appear is a decoration (review 83183; DESIGN §4b).

## What the widening does (read from origin/session/quiet-hawk-507, src/v1/04_types.dag)
Adds type_head_carrier / type_head_is_declared_carrier_alias / type_heads_agree, and replaces the
container branch `left_name != right_name { false }` inside node_type_equals_core with
`if !type_heads_agree(...) { false }` -- so carrier aliases (per the hardcoded
carrier_container_algebra_rows table: List -> FreeMonoid) compare by carrier, then arguments pairwise.
Blast radius on main: 6 read-sites of node_type_equals(_core) across src/v1 (04_access.dag,
04_emit_info.dag, 04_types.dag internals). Small, genuinely separable.

## The only claimed observable, and what it measures
The witness for this class is dag/test/claim/algebra_receiver_alias_witness.dag
(alias_return_is_a_refusal_fixture: `fn asks() -> List<Int> { [1,2] |> fold(init: Empty, f: ...) }`),
whose rows assert blocking_subject_count_for >= 1 for class MethodNotFound, subject
Coproduct(FreeMonoid) -- the refusal expected at variant_reference_inferred_node.

MEASURED (this session, probe/alias-seam A/B, one binary two settings, j02338b64 + j67fc0a57):
through the fixture's OWN builtin (compile_dag_diagnostic_census), alias_return yields
ARM=Observed rows=24 blocking_rows=0 blocking_total=0 -- on MAIN, guard live. The asserted refusal
does not appear. The same class appears NOWHERE in the two-closure logs either: the comparator's
identities at the Acc head are all (module, param, expected-head, got-head) type-mismatch tuples;
no MethodNotFound/Coproduct(FreeMonoid) identity exists at head or main (instrument
gate_two_closures_14152.sh cmp_*.hd.json, pin head=97e47f8e562 main=13ac960a93f).

## Consequence, stated as a decision for stern, not a guess
1. Enrolling the witness rows as ExpectedRed (#14532-style) would enroll a claim whose failure mode
   is "measured 0 blocking", NOT "refused as owed" -- the ExpectedRed instrument would report the
   red held for a claim that never sees the seam. That is the declared-not-executed class stern
   already routed to Charter 4 for this very file.
2. The fold-level alternative (call node_type_equals directly with two Node trees, like the mint
   control) runs only in v1 Rust unit tests, which are OFF the required path
   (DESIGN.md; dag/gunbc/rung_drop/rust_unit_tests_off_the_merge_path.dag) -- green by vacancy,
   which parent's (d) already rejects for PR-2's discriminators.
3. So PR-2a has no executed RED I can author today. Options for stern: (A) PR-2a holds until the
   variant_reference_inferred_node seam is executable by some instrument (its owner:
   gunbc.guarantee_stall.algebra_application_owner_resolution_stall, ceiling StructurallyImpossible);
   (B) PR-2a lands as documentation + pin like PR-1, missing next-rung carrier named;
   (C) stern accepts a fold-level unit control knowing it is off the required path (my prediction:
   a future review will call it a decoration, as review 83183 did).
NOT BUILDING until this is answered; the widening itself is small and ready to extract.
