use v1_compiler::cli_run::{compile_dag_multi_module_fixture, MultiModuleCompileFixtureOutcome};
use v1_compiler::std_algebra::{
    algebra_method_template_name, algebra_templates_for_profile, kernel_algebra_profile,
};
use v1_compiler::v1_compiler_infer_method::{
    bare_call_non_declaration_binding, NonDeclarationBinding,
};

fn completed_without_blocking(o: &MultiModuleCompileFixtureOutcome) -> bool {
    match o {
        MultiModuleCompileFixtureOutcome::CompileCompleted { diagnostics, .. } => {
            diagnostics.iter().all(|r| !r.blocking)
        }
        _ => false,
    }
}

fn refused_with_blocking(o: &MultiModuleCompileFixtureOutcome) -> bool {
    match o {
        MultiModuleCompileFixtureOutcome::CompileRefused { diagnostics, .. } => {
            diagnostics.iter().any(|r| r.blocking)
        }
        _ => false,
    }
}

const DECOY_MAP: &str =
    "module kmfx.decoy\nfn map(xs: List<Int>, f: fn(Int) -> Int) -> Int { 0 }\n";
const PROBE_KERNEL: &str =
    "module kmfx.probe\nfn mapped(xs: List<Int>) -> List<Int> { map(xs, x => x) }\nfn joined() -> String { join([\"a\", \"b\"], \",\") }\n";
const PROBE_MISSING_KERNEL: &str =
    "module kmfx.probe\nfn broken() -> Int { definitely_not_a_kernel_method(1) }\n";

const DECOY_IDENTITY: &str = "module kmfx.ident_decoy\n\ntype ProcessExit = ExitSuccess | ExitFailure { reason: String }\n\ntype ProbeBox = { n: Int }\n\nfn census_exit(reason: String) -> ProcessExit {\n  ExitFailure { reason: reason }\n}\n";
const PROBE_IDENTITY_AND_CTOR: &str = "module kmfx.ident_probe\n\nimport kmfx.ident_decoy { ProcessExit, ExitSuccess, ProbeBox }\n\nfn pick(flag: Bool) -> ProcessExit {\n  if flag { ExitSuccess } else { census_exit(\"x\") }\n}\n\nfn boxed() -> ProbeBox { ProbeBox { n: 1 } }\n";
const PROBE_MISSING_CALLEE: &str =
    "module kmfx.ident_probe\n\nfn broken() -> Int { definitely_not_a_callee(1) }\n";

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

#[test]
fn unique_census_map_does_not_bind_a_bare_kernel_map_call() {
    let outcome = compile_dag_multi_module_fixture(
        &["kmfx_decoy.dag".to_string(), "kmfx_probe.dag".to_string()],
        &[DECOY_MAP.to_string(), PROBE_KERNEL.to_string()],
        "kmfx_probe.dag",
    );
    assert!(
        completed_without_blocking(&outcome),
        "bare map/join must type as kernel methods, not the decoy census map, got {outcome:?}"
    );
}

#[test]
fn a_missing_free_call_still_refuses() {
    let outcome = compile_dag_multi_module_fixture(
        &["kmfx_decoy.dag".to_string(), "kmfx_probe.dag".to_string()],
        &[DECOY_MAP.to_string(), PROBE_MISSING_KERNEL.to_string()],
        "kmfx_probe.dag",
    );
    assert!(
        refused_with_blocking(&outcome),
        "an unknown free call must refuse, got {outcome:?}"
    );
}

#[test]
fn a_census_identity_exit_call_inhabits_process_exit() {
    let outcome = compile_dag_multi_module_fixture(
        &[
            "kmfx_ident_decoy.dag".to_string(),
            "kmfx_ident_probe.dag".to_string(),
        ],
        &[
            DECOY_IDENTITY.to_string(),
            PROBE_IDENTITY_AND_CTOR.to_string(),
        ],
        "kmfx_ident_probe.dag",
    );
    assert!(
        completed_without_blocking(&outcome),
        "census_exit(...) must inhabit ProcessExit opposite ExitSuccess, and ProbeBox must construct, got {outcome:?}"
    );
}

#[test]
fn a_missing_census_identity_callee_still_refuses() {
    let outcome = compile_dag_multi_module_fixture(
        &[
            "kmfx_ident_decoy.dag".to_string(),
            "kmfx_ident_probe.dag".to_string(),
        ],
        &[DECOY_IDENTITY.to_string(), PROBE_MISSING_CALLEE.to_string()],
        "kmfx_ident_probe.dag",
    );
    assert!(
        refused_with_blocking(&outcome),
        "a missing callee must refuse, got {outcome:?}"
    );
}
