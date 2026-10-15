use v1_compiler::std_algebra::{
    algebra_method_template_name, algebra_templates_for_profile, kernel_algebra_profile,
};
use v1_compiler::v1_compiler_infer_method::{
    bare_call_non_declaration_binding, NonDeclarationBinding,
};

// Compile fixtures live in test.claim.kernel_method_free_call_census_skip_witness
// and test.claim.census_identity_callable_not_a_type_witness. This seed test is
// only the classification subset those witnesses cannot state: KernelMethodOnly
// ⊆ algebra_method_template_name.
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
