use std::rc::Rc;
use v1_compiler::std_algebra::{
    algebra_method_template_name, algebra_templates_for_profile, kernel_algebra_profile,
};
use v1_compiler::std_induction::SubValueRelation;
use v1_compiler::std_occurrence_identity::NodeOccurrenceIdentity;
use v1_compiler::v1_compiler_infer::{
    call_target_for_direct_call, nominal_ref_node, type_match_for_direct_call_target, InferScope,
};
use v1_compiler::v1_compiler_infer_env::{
    empty_type_env, lookup_type_by_name, GlobalBareLookupState, TypeBinding,
};
use v1_compiler::v1_compiler_infer_items::{item_kind, ItemKind};
use v1_compiler::v1_compiler_infer_lookup::{
    census_declaration_identity, func_sig_from_global_bare,
};
use v1_compiler::v1_compiler_infer_method::{
    bare_call_non_declaration_binding, NonDeclarationBinding,
};
use v1_compiler::v1_compiler_infer_sigs::{CallTargetOutcome, FuncSigLookup, ResolvedFuncEnv};
use v1_compiler::v1_std_core::{
    make_pattern_binder_declaration_node, no_span, Node, ParsedModuleItemKind,
};

// Compile fixtures live in test.claim.kernel_method_free_call_census_skip_witness
// and test.claim.census_identity_callable_not_a_type_witness. This seed test is
// only the classification subset those witnesses cannot state: KernelMethodOnly
// ⊆ algebra_method_template_name, plus the IdentityResolved type-lookup skip
// driven through census Present + FuncSigUnresolved + call_target_for_direct_call.
#[test]
fn kernel_method_only_names_are_algebra_method_templates() {
    for profile in kernel_algebra_profile().values() {
        for t in algebra_templates_for_profile(*profile).iter() {
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

fn other_item_census_node(name: &str) -> Rc<Node> {
    make_pattern_binder_declaration_node(
        Rc::new(NodeOccurrenceIdentity::OccurrenceSynthetic),
        name.to_string(),
        no_span(),
    )
}

fn identity_resolved_scope(name: String, node: Rc<Node>) -> Rc<InferScope> {
    let binding = Rc::new(TypeBinding {
        name: name.clone(),
        resolved: node,
        provenance: Rc::new(SubValueRelation::SubValueUnknown),
        alias_rhs: None,
    });
    let mut env = (*empty_type_env()).clone();
    env.str_bindings = Rc::new((*env.str_bindings).update(name.clone(), binding.clone()));
    let mut index = (*env.symbol_index).clone();
    index.global_bare = Rc::new((*index.global_bare).update(
        name,
        Rc::new(GlobalBareLookupState::GlobalBareUniqueBinding {
            module_path: "kmfx.ident_decoy".to_string(),
            binding,
        }),
    ));
    env.symbol_index = Rc::new(index);
    Rc::new(InferScope {
        type_env: Rc::new(env),
        func_env: Rc::new(ResolvedFuncEnv {
            name: "kmfx.ident_probe".to_string(),
            local: Rc::new(im::HashMap::new()),
            parents: Rc::new(im::Vector::new()),
        }),
        locals: Rc::new(im::HashMap::new()),
        body_locals: Rc::new(im::HashMap::new()),
        match_bound_names: Rc::new(im::HashMap::new()),
        module_name: "kmfx.ident_probe".to_string(),
        service_registry: Rc::new(im::HashMap::new()),
        item_registry: Rc::new(im::HashMap::new()),
        lambda_param_provenance: Rc::new(im::HashMap::new()),
        caller_decl_name: "".to_string(),
        in_flight_lambda_param_names: Rc::new(im::Vector::new()),
        enclosing_declared_type_param_names: Rc::new(im::Vector::new()),
    })
}

#[test]
fn identity_resolved_type_lookup_skip_covers_func_sig_unresolved_other_item() {
    let name = "census_heads".to_string();
    let node = other_item_census_node(&name);
    assert_eq!(item_kind(node.clone()), ItemKind::OtherItem);
    assert!(node.params.is_empty());
    assert!(node.inferred.is_none());
    assert!(node.type_annotation.is_none());
    assert!(matches!(
        bare_call_non_declaration_binding(name.clone()),
        NonDeclarationBinding::DeclarationOnly
    ));

    let scope = identity_resolved_scope(name.clone(), node);
    let env = scope.type_env.clone();
    assert!(
        lookup_type_by_name(env.clone(), name.clone()).is_some(),
        "control requires lookup_type_by_name to steal the OtherItem if the skip is deleted"
    );
    assert!(
        census_declaration_identity(env.clone(), name.clone()).is_some(),
        "census Present requires symbol_index.global_bare UniqueBinding"
    );
    assert!(matches!(
        (*func_sig_from_global_bare(env.clone(), name.clone())).clone(),
        FuncSigLookup::FuncSigUnresolved
    ));

    let target = call_target_for_direct_call(scope, name.clone());
    assert!(
        matches!(
            (*target.clone()).clone(),
            CallTargetOutcome::DeclaredCallableIdentityResolved { .. }
        ),
        "call_target_for_direct_call must mint IdentityResolved, got {:?}",
        target
    );
    assert!(
        type_match_for_direct_call_target(env, name, target).is_none(),
        "IdentityResolved must skip type lookup; deleting the skip returns Named(census_heads)"
    );
}

// A RESOURCE IS NOT A CALL TARGET. The census serves a `resource` declaration as a nominal
// reference (v1.compiler.infer local_binding_for_item -> nominal_ref_node). That reference was
// stamped NotAModuleItem, item_kind read it as OtherItem, and census_declaration_identity minted
// DeclaredCallableIdentityResolved for it. The live case: std.resources' `resource Clock` was the
// unique bare `Clock` in the tree, so the receiver of extdeps.clock's own `Clock.UnixMillis()`
// resolved to it and refused at constructor admission (no declaration indexed at
// std.resources.Clock). The two nodes below differ ONLY in the form the reference carries: the
// unstamped twin reproduces the pre-repair outcome, so the pair discriminates on the form.
fn census_nominal_ref(name: &str, kind: ParsedModuleItemKind) -> Rc<Node> {
    nominal_ref_node(name.to_string(), no_span(), None, kind)
}

#[test]
fn a_resource_census_reference_is_not_a_call_target() {
    let name = "Clock".to_string();
    let node = census_nominal_ref(&name, ParsedModuleItemKind::ModuleItemResource);
    assert_eq!(item_kind(node.clone()), ItemKind::ResourceItem);

    let scope = identity_resolved_scope(name.clone(), node);
    assert!(
        census_declaration_identity(scope.type_env.clone(), name.clone()).is_none(),
        "a resource must not be served as a census callable identity"
    );
    let target = call_target_for_direct_call(scope, name);
    assert!(
        matches!(
            (*target).clone(),
            CallTargetOutcome::CallableUnresolved { .. }
        ),
        "a bare reference to a resource name must not resolve to a declared callable, got {:?}",
        target
    );
}

#[test]
fn an_unstamped_census_reference_still_resolves_as_a_declared_callable() {
    let name = "Clock".to_string();
    let node = census_nominal_ref(&name, ParsedModuleItemKind::NotAModuleItem);
    assert_eq!(item_kind(node.clone()), ItemKind::OtherItem);
    let scope = identity_resolved_scope(name.clone(), node);
    let target = call_target_for_direct_call(scope, name);
    assert!(
        matches!(
            (*target).clone(),
            CallTargetOutcome::DeclaredCallableResolved { .. }
                | CallTargetOutcome::DeclaredCallableIdentityResolved { .. }
        ),
        "control: without the resource form the same reference still resolves to a declared \
         callable, which is what constructor admission then refused, got {:?}",
        target
    );
}
