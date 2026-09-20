// DISCRIMINATING RED for the cross-scope intern-id read that `shared_typecheck_store`'s
// "Payload is name-keyed (no intern-table indices) -- safe to materialize on any worker
// index" comment asserts cannot happen.
//
// `Node` serializes `ident: Option<i64>` with no `serde(skip)`, so a decoded typed snapshot
// carries intern indices minted under the PRODUCING worker's table. `lookup_binding` resolves
// such an id against the CONSUMING env's table, with no scope check on either arm:
//
//   map_get(env.bindings, ident)  -> hit:  whatever binding sits at that index
//                                 -> miss: intern_str(env.intern_table, ident) -> wrong name
//
// Both arms answer with a WRONG BINDING rather than refusing, which is the DESIGN section 5
// fail-open shape. This test constructs the two-scope condition directly and pins the
// current (wrong) behaviour, so that a scope check landing upstream turns it red.
use std::rc::Rc;

use v1_compiler::std_induction::SubValueRelation;
use v1_compiler::std_occurrence_identity::occurrence_id_allocator_initial;
use v1_compiler::v1_compiler_infer_env::{empty_type_env, lookup_binding, TypeBinding, TypeEnv};
use v1_compiler::v1_std_core::{
    empty_intern_table, intern, intern_str, leaf_node_with_span, no_span,
};
use v1_compiler::std_occurrence_identity::node_occurrence_identity_minted;
use v1_compiler::std_occurrence_identity::OccurrenceId;

fn table_interning(names: &[&str]) -> Rc<v1_compiler::v1_std_core::InternTable> {
    let mut t = empty_intern_table();
    for n in names {
        t = intern(t, n.to_string()).table.clone();
    }
    t
}

fn node(name: &str) -> Rc<v1_compiler::v1_std_core::Node> {
    let _ = occurrence_id_allocator_initial();
    leaf_node_with_span(
        node_occurrence_identity_minted(OccurrenceId { value: 0 }),
        name.to_string(),
        no_span(),
    )
}

fn env_with(table: Rc<v1_compiler::v1_std_core::InternTable>, id: i64, name: &str) -> Rc<TypeEnv> {
    let base = empty_type_env();
    let binding = Rc::new(TypeBinding {
        name: name.to_string(),
        resolved: node(name),
        provenance: Rc::new(SubValueRelation::PreservedValue),
    });
    let mut bindings = (*base.bindings).clone();
    bindings.insert(id, binding);
    Rc::new(TypeEnv {
        bindings: Rc::new(bindings),
        intern_table: table,
        ..(*base).clone()
    })
}

/// Two intern scopes assign the SAME id to DIFFERENT names -- the exact condition a decoded
/// cross-worker snapshot creates. Producing scope means "Beta"; consuming scope reads "Alpha".
#[test]
fn foreign_scope_ident_resolves_to_the_wrong_binding_instead_of_refusing() {
    let consumer_table = table_interning(&["Alpha", "Beta"]);
    let producer_table = table_interning(&["Beta", "Alpha"]);

    let alpha_in_consumer = intern(consumer_table.clone(), "Alpha".to_string()).id;
    let beta_in_producer = intern(producer_table.clone(), "Beta".to_string()).id;

    // The premise: one id, two meanings. If this ever stops holding the test is vacuous.
    assert_eq!(
        alpha_in_consumer, beta_in_producer,
        "premise: the two scopes must collide on one id for this to be a real test"
    );
    assert_eq!(intern_str(consumer_table.clone(), alpha_in_consumer), "Alpha");
    assert_eq!(intern_str(producer_table.clone(), beta_in_producer), "Beta");

    let env = env_with(consumer_table, alpha_in_consumer, "Alpha");

    // A node decoded from the producing scope carries `ident = beta_in_producer`, meaning "Beta".
    let found = lookup_binding(env, beta_in_producer);

    match found {
        None => panic!(
            "UNEXPECTED REFUSAL -- if lookup_binding now refuses a foreign-scope id, the hazard \
             this test pins is fixed and the test should be rewritten as the wall's control"
        ),
        Some(b) => assert_eq!(
            b.name, "Alpha",
            "pins today's behaviour: the consuming env answers with ITS OWN binding for that \
             index, silently substituting Alpha where the producing scope meant Beta"
        ),
    }
}
