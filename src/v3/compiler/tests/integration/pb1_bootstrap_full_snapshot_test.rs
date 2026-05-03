//! **Layer:** integration
//!
//! PB-1-e: in-tree DB-8 cross-check is structural — committed snapshots are
//! internally consistent; `Dag::new()` is diagnostic-clean and byte-stable across
//! clones; the std-only snapshot is a strict prefix-shape of the full snapshot.
//! The fresh-compile vs committed snapshot contract is enforced by CI
//! `regen_bootstrap --verify` (`--features bootstrap-regen-fresh`), not on every
//! `cargo test`. See `docs/briefs/pb-1-e-residual-scaffold-retirement-worker.md`.

use v3_compiler::{
    dag::{DeclarationId, FieldValue, TypeConnective, ValueBody},
    generated_full_bootstrap_dag, generated_full_bootstrap_without_parse_surface_dag,
    generated_std_bootstrap_dag,
    serialize::{first_difference, serialize_dag},
    Dag,
};

use std::collections::BTreeSet;

#[test]
fn full_bootstrap_extends_std_snapshot() {
    let std_only = generated_std_bootstrap_dag();
    let full = generated_full_bootstrap_dag();
    assert!(
        first_difference(&std_only, &full).is_some(),
        "full bootstrap unexpectedly identical to std-only snapshot"
    );
}

#[test]
fn generated_std_bootstrap_snapshot_has_no_diagnostics() {
    let dag = generated_std_bootstrap_dag();
    assert!(
        dag.diagnostics().is_empty(),
        "expected clean std snapshot bootstrap, got {:?}",
        dag.diagnostics()
    );
}

#[test]
fn generated_std_bootstrap_snapshot_includes_bool() {
    let dag = generated_std_bootstrap_dag();
    assert!(
        dag.declaration_by_name("Bool").is_some(),
        "std snapshot should include kernel Bool"
    );
}

#[test]
fn generated_full_bootstrap_snapshots_have_no_diagnostics() {
    for (label, dag) in [
        ("full", generated_full_bootstrap_dag()),
        (
            "without_parse_surface",
            generated_full_bootstrap_without_parse_surface_dag(),
        ),
    ] {
        assert!(
            dag.diagnostics().is_empty(),
            "{label}: expected clean generated bootstrap, got {:?}",
            dag.diagnostics()
        );
    }
}

#[test]
fn generated_full_bootstrap_snapshots_include_parse_stage() {
    for (label, dag) in [
        ("full", generated_full_bootstrap_dag()),
        (
            "without_parse_surface",
            generated_full_bootstrap_without_parse_surface_dag(),
        ),
    ] {
        assert!(
            dag.declaration_by_name("parse").is_some(),
            "{label}: expected pipeline `parse` stage in bootstrap Dag"
        );
    }
}

#[test]
fn dag_new_bootstrap_is_clean_and_byte_stable() {
    let first = Dag::new();
    assert!(
        first.diagnostics().is_empty(),
        "Dag::new() bootstrap should be clean, got {:?}",
        first.diagnostics()
    );

    let second = Dag::new();
    assert!(
        first_difference(&first, &second).is_none(),
        "Dag::new() should clone a stable committed bootstrap snapshot"
    );
    assert_eq!(
        serialize_dag(&first),
        serialize_dag(&second),
        "Dag::new() serialized bootstrap bytes should be stable across clones"
    );
}

#[test]
fn bootstrap_authority_rows_match_full_bootstrap_source_files() {
    let dag = generated_full_bootstrap_dag();
    let authority_rows = bootstrap_authority_rows(&dag);

    let authority_paths: BTreeSet<&str> = authority_rows
        .iter()
        .map(|(_, path)| path.as_str())
        .collect();
    let source_files: BTreeSet<&str> = dag
        .declarations()
        .iter()
        .map(|decl| decl.span.file.as_str())
        .collect();

    assert_eq!(
        authority_paths, source_files,
        "`bootstrap_authority` must match the committed full bootstrap snapshot's source-file membership"
    );

    for (kind, path) in authority_rows {
        match kind.as_str() {
            "StdAuthority" => assert!(
                path.starts_with("dsl/std/"),
                "StdAuthority row points outside dsl/std: {path}"
            ),
            "V3StdAuthority" => assert!(
                path.starts_with("src/v3/std/"),
                "V3StdAuthority row points outside src/v3/std: {path}"
            ),
            "V3SpecAuthority" => assert!(
                path.starts_with("src/v3/spec/"),
                "V3SpecAuthority row points outside src/v3/spec: {path}"
            ),
            "CompilerAuthority" => assert!(
                path.starts_with("src/v3/compiler/"),
                "CompilerAuthority row points outside src/v3/compiler: {path}"
            ),
            "ExtdepsFixtureAuthority" => assert!(
                path.starts_with("dsl/extdeps/"),
                "ExtdepsFixtureAuthority row points outside dsl/extdeps: {path}"
            ),
            other => panic!("unexpected BootstrapAuthorityKind row: {other}"),
        }
    }
}

#[test]
fn diagnostics_empty_after_bootstrap_for_bootstrap_authority() {
    let dag = generated_full_bootstrap_dag();
    let authority_rows = bootstrap_authority_rows(&dag);
    let mut failures = Vec::new();

    for (kind, path) in authority_rows {
        let diagnostics: Vec<_> = dag
            .diagnostics()
            .iter()
            .filter_map(|(port, diagnostic)| {
                (diagnostic.span().file == path)
                    .then(|| format!("port {port:?}: {:?}: {}", diagnostic, diagnostic.message()))
            })
            .collect();

        if !diagnostics.is_empty() {
            failures.push(format!(
                "{path} ({kind}) produced diagnostics after bootstrap:\n  {}",
                diagnostics.join("\n  ")
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "diagnostics_empty_after_bootstrap failed for bootstrap_authority rows:\n{}",
        failures.join("\n\n")
    );
}

fn bootstrap_authority_rows(dag: &Dag) -> Vec<(String, String)> {
    let decl = dag
        .declaration_by_name("bootstrap_authority")
        .expect("full bootstrap loads bootstrap_authority");
    let variant_labels = bootstrap_authority_variant_labels(dag);
    let Some(ValueBody::Map(rows)) = decl.value_body.as_ref() else {
        panic!("bootstrap_authority should lower to a structural map body");
    };

    rows.entries()
        .iter()
        .map(|(key, authority)| {
            let FieldValue::Variant {
                constructor,
                payload,
            } = authority
            else {
                panic!("bootstrap_authority map value should lower to a variant");
            };
            let kind = dag
                .declaration(*constructor);
            let kind = variant_labels
                .iter()
                .find_map(|(id, label)| (*id == *constructor).then(|| label.clone()))
                .unwrap_or_else(|| {
                    panic!(
                        "BootstrapAuthority variant constructor {:?} should appear in BootstrapAuthority",
                        kind.id
                    )
                });
            assert!(
                payload.is_empty(),
                "BootstrapAuthority variants should not carry duplicate path payloads"
            );
            (kind, key.clone())
        })
        .collect()
}

fn bootstrap_authority_variant_labels(dag: &Dag) -> Vec<(DeclarationId, String)> {
    let decl = dag
        .declaration_by_name("BootstrapAuthority")
        .expect("full bootstrap loads BootstrapAuthority");
    let TypeConnective::Disj { variants } = &decl.connective else {
        panic!("BootstrapAuthority should lower to a disjunction");
    };
    variants
        .iter()
        .map(|variant| (variant.ty, variant.label.clone()))
        .collect()
}
