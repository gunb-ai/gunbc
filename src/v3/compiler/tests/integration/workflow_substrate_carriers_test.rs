//! **Layer:** integration
//!
//! Structural acceptance for R3 §1.8 gate #53 `workflow_substrate_carriers_landed`:
//! Slice 1 workflow substrate carriers landed at `dsl/extdeps/github/actions.dag` +
//! `dsl/extdeps/cron/schedule_model.dag` per Director β-ratification at
//! gunbc#828 #issuecomment-4395945465 (PR #2160). Carriers ratcheted here:
//!
//!   - `WorkflowSecret { name: SecretName, scope: SecretScope }` (provider-typed,
//!     opaque-at-rest secret reference scoped by step / job / workflow)
//!   - `SecretScope = StepScope | JobScope | WorkflowScope`
//!   - `CronSchedule` (5 typed fields — minute / hour / day_of_month / month / day_of_week)
//!   - `CronField` (5 variants — Wildcard / Exact / Listed / Ranged / Step)
//!
//! Sibling-carrier ratchet to `file_attachment_substrate_carrier_test.rs` (gate #62).

use v3_compiler::dag::{Dag, DeclarationId, TypeConnective};
use v3_compiler::generated_full_bootstrap_dag;

fn conj_field_labels(dag: &Dag, name: &str) -> Vec<String> {
    let decl = dag
        .declaration_by_name(name)
        .unwrap_or_else(|| panic!("`{name}` missing from full bootstrap"));
    match &decl.connective {
        TypeConnective::Conj { children } => children.iter().map(|f| f.label.clone()).collect(),
        other => panic!("`{name}` is not a Conj: {other:?}"),
    }
}

fn conj_field_ty(dag: &Dag, name: &str, field: &str) -> DeclarationId {
    let decl = dag
        .declaration_by_name(name)
        .unwrap_or_else(|| panic!("`{name}` missing from full bootstrap"));
    match &decl.connective {
        TypeConnective::Conj { children } => {
            children
                .iter()
                .find(|f| f.label == field)
                .unwrap_or_else(|| panic!("`{name}` missing `{field}` field"))
                .ty
        }
        other => panic!("`{name}` is not a Conj: {other:?}"),
    }
}

fn disj_variant_labels(dag: &Dag, name: &str) -> Vec<String> {
    let decl = dag
        .declaration_by_name(name)
        .unwrap_or_else(|| panic!("`{name}` missing from full bootstrap"));
    match &decl.connective {
        TypeConnective::Disj { variants } => variants.iter().map(|v| v.label.clone()).collect(),
        other => panic!("`{name}` is not a Disj: {other:?}"),
    }
}

#[test]
fn workflow_secret_shape_locked() {
    let dag = generated_full_bootstrap_dag();
    let mut labels = conj_field_labels(&dag, "WorkflowSecret");
    labels.sort();
    assert_eq!(
        labels,
        vec!["name".to_string(), "scope".to_string()],
        "WorkflowSecret field set drifted from β-ratification (gate #53 / PR #2160)"
    );

    let secret_name = dag
        .declaration_by_name("SecretName")
        .expect("`SecretName` missing from full bootstrap (std.types authority)")
        .id;
    let secret_scope = dag
        .declaration_by_name("SecretScope")
        .expect("`SecretScope` missing from full bootstrap")
        .id;
    assert_eq!(
        conj_field_ty(&dag, "WorkflowSecret", "name"),
        secret_name,
        "`WorkflowSecret.name` must be `SecretName` (cross-provider std.types authority)"
    );
    assert_eq!(
        conj_field_ty(&dag, "WorkflowSecret", "scope"),
        secret_scope,
        "`WorkflowSecret.scope` must be `SecretScope`"
    );
}

#[test]
fn secret_scope_variants_locked() {
    let dag = generated_full_bootstrap_dag();
    let mut variants = disj_variant_labels(&dag, "SecretScope");
    variants.sort();
    assert_eq!(
        variants,
        vec![
            "JobScope".to_string(),
            "StepScope".to_string(),
            "WorkflowScope".to_string(),
        ],
        "SecretScope variants drifted from β-ratification (gate #53)"
    );
}

#[test]
fn cron_schedule_shape_locked() {
    let dag = generated_full_bootstrap_dag();
    let mut labels = conj_field_labels(&dag, "CronSchedule");
    labels.sort();
    assert_eq!(
        labels,
        vec![
            "day_of_month".to_string(),
            "day_of_week".to_string(),
            "hour".to_string(),
            "minute".to_string(),
            "month".to_string(),
        ],
        "CronSchedule field set drifted (gate #53 STOP+PING — >5 fields requires Director ratification)"
    );

    let cron_field = dag
        .declaration_by_name("CronField")
        .expect("`CronField` missing from full bootstrap")
        .id;
    for field in ["minute", "hour", "day_of_month", "month", "day_of_week"] {
        assert_eq!(
            conj_field_ty(&dag, "CronSchedule", field),
            cron_field,
            "`CronSchedule.{field}` must be `CronField`"
        );
    }
}

#[test]
fn cron_field_variants_locked() {
    let dag = generated_full_bootstrap_dag();
    let mut variants = disj_variant_labels(&dag, "CronField");
    variants.sort();
    assert_eq!(
        variants,
        vec![
            "Exact".to_string(),
            "Listed".to_string(),
            "Ranged".to_string(),
            "Step".to_string(),
            "Wildcard".to_string(),
        ],
        "CronField variants drifted (gate #53)"
    );
}
