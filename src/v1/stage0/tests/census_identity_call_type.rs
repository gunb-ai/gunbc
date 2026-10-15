use std::rc::Rc;
use v1_compiler::std_algebra::{
    algebra_method_template_name, algebra_templates_for_profile, kernel_algebra_profile,
};
use v1_compiler::std_induction::SubValueRelation;
use v1_compiler::std_occurrence_identity::NodeOccurrenceIdentity;
use v1_compiler::v1_compiler_infer::type_match_for_direct_call_target;
use v1_compiler::v1_compiler_infer_env::{
    empty_type_env, lookup_type_by_name, TypeBinding, TypeEnv,
};
use v1_compiler::v1_compiler_infer_items::{item_kind, ItemKind};
use v1_compiler::v1_compiler_infer_method::{
    bare_call_non_declaration_binding, NonDeclarationBinding,
};
use v1_compiler::v1_compiler_infer_sigs::CallTargetOutcome;
use v1_compiler::v1_std_core::{
    make_pattern_binder_declaration_node, no_span, DeclaredCallableIdentity,
};

// Compile fixtures live in test.claim.kernel_method_free_call_census_skip_witness
// and test.claim.census_identity_callable_not_a_type_witness. This seed test is
// only the classification subset those witnesses cannot state: KernelMethodOnly
// ⊆ algebra_method_template_name, plus the IdentityResolved type-lookup skip.
#[test]
fn kernel_method_only_names_are_algebra_method_templates() {
    for profile in kernel_algebra_profile().values() {
        for t in algebra_templates_for_profile(profile.clone()).iter() {
            if matches!(
                bare_call_non_declaration_binding(t.name.clone()),
                NonDeclarationBinding::KernelMethodOnly
            ) {
                assert!(
                    algebra_method_template_name(t.name.clone()),
                    "KernelMethodOnly name {} must be an algebra method-template name",
                    t.name
                );
            }
        }
    }
    assert!(matches!(
        bare_call_non_declaration_binding("census_exit".to_string()),
        NonDeclarationBinding::DeclarationOnly
    ));
}

fn other_item_census_node(name: &str) -> Rc<v1_compiler::v1_std_core::Node> {
    make_pattern_binder_declaration_node(
        Rc::new(NodeOccurrenceIdentity::OccurrenceSynthetic),
        name.to_string(),
        no_span(),
    )
}

fn env_with_str_binding(name: String, node: Rc<v1_compiler::v1_std_core::Node>) -> Rc<TypeEnv> {
    let mut env = (*empty_type_env()).clone();
    env.str_bindings = Rc::new((*env.str_bindings).update(
        name.clone(),
        Rc::new(TypeBinding {
            name,
            resolved: node,
            provenance: Rc::new(SubValueRelation::SubValueUnknown),
            alias_rhs: None,
        }),
    ));
    Rc::new(env)
}

#[test]
fn identity_resolved_type_lookup_skip_covers_func_sig_unresolved_other_item() {
    let name = "census_heads".to_string();
    let node = other_item_census_node(&name);
    assert_eq!(item_kind(node.clone()), ItemKind::OtherItem);
    assert!(node.params.is_empty());
    assert!(node.inferred.is_none());
    assert!(node.type_annotation.is_none());

    let env = env_with_str_binding(name.clone(), node.clone());
    assert!(
        lookup_type_by_name(env.clone(), name.clone()).is_some(),
        "control requires lookup_type_by_name to steal the OtherItem if the skip is deleted"
    );

    let call_target = Rc::new(CallTargetOutcome::DeclaredCallableIdentityResolved {
        declared: Rc::new(DeclaredCallableIdentity {
            owner_module_path: "kmfx.ident_decoy".to_string(),
            decl_name: name.clone(),
        }),
        borrowed_declaration: node,
    });
    assert!(
        type_match_for_direct_call_target(env, name, call_target).is_none(),
        "IdentityResolved must skip type lookup; deleting the skip returns Named(census_heads)"
    );
}
