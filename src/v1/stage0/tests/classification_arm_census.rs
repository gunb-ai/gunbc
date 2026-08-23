//! LOCAL DIAGNOSTIC PROBE — not for commit. Population of every ExprVarClass member over the
//! whole prepared corpus, and specimens of NamesNothingKnown. Decides whether the "fits nothing"
//! arm can be a hard refusal or whether its population is a finding in its own right.

use v1_compiler::cli_run::{
    claim_scope_for, floor_prepared_subject_exclusions, prepare_repository_once,
};

#[test]
#[ignore = "whole-corpus prepare; local diagnosis only"]
fn classification_arm_population() {
    let roots = vec!["dag".to_string(), "src/v2".to_string()];
    let (prepared, _views) =
        prepare_repository_once(&roots, &floor_prepared_subject_exclusions()).expect("prepare");
    // Building any scope forces the reference-closure index, which prints the class tally and
    // the unclassified population to stderr.
    let scope = claim_scope_for(&prepared, "test.claim.host_phase_status").expect("scope");
    println!(
        "SCOPE\ttest.claim.host_phase_status\tmodules={}\tambiguous={}",
        scope.module_count, scope.ambiguous_bare_names
    );
    for m in [
        "v2.test.extdeps_shape_transport_policy.corpus.cargo_fmt_policy_leak",
        "test.claim.design_argument_witness",
        "test.claim.realization_reconcile_witness",
    ] {
        let s = claim_scope_for(&prepared, m).expect("scope");
        println!(
            "SCOPE\t{m}\tmodules={}\tambiguous={}",
            s.module_count, s.ambiguous_bare_names
        );
    }
}
