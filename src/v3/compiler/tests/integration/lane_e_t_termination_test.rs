//! **Layer:** integration

use std::collections::HashMap;

use v3_compiler::dag::{
    evidence_rank, join_evidence, map_evidence_merge_at, merge_evidence, optional_evidence_meet,
    promote_to_strict, ArrowBody, Dag, DescentEvidence, TypeConnective,
};

fn find_named(dag: &Dag, name: &str) -> v3_compiler::dag::DeclarationId {
    dag.declaration_by_name(name)
        .unwrap_or_else(|| panic!("declaration `{name}` not found"))
        .id
}

fn record_fields(dag: &Dag, name: &str) -> Vec<String> {
    let id = find_named(dag, name);
    match &dag.declaration(id).connective {
        TypeConnective::Conj { children } => {
            children.iter().map(|field| field.label.clone()).collect()
        }
        other => panic!("expected `{name}` to lower to a Conj, got {other:?}"),
    }
}

fn sum_variants(dag: &Dag, name: &str) -> Vec<(String, Vec<String>)> {
    let id = find_named(dag, name);
    match &dag.declaration(id).connective {
        TypeConnective::Disj { variants } => variants
            .iter()
            .map(|variant| {
                let payload = match &dag.declaration(variant.ty).connective {
                    TypeConnective::Conj { children } => {
                        children.iter().map(|field| field.label.clone()).collect()
                    }
                    other => panic!(
                        "expected variant `{}` under `{name}` to lower to a Conj payload, got {other:?}",
                        variant.label
                    ),
                };
                (variant.label.clone(), payload)
            })
            .collect(),
        other => panic!("expected `{name}` to lower to a Disj, got {other:?}"),
    }
}

fn arrow_body(dag: &Dag, name: &str) -> ArrowBody {
    let id = find_named(dag, name);
    match &dag.declaration(id).connective {
        TypeConnective::Arrow { body, .. } => body.clone(),
        other => panic!("expected `{name}` to lower to an Arrow, got {other:?}"),
    }
}

#[test]
fn termination_carriers_bootstrap_from_v3_std() {
    let dag = Dag::new();
    assert!(
        dag.diagnostics().is_empty(),
        "bootstrap should load termination carriers cleanly: {:?}",
        dag.diagnostics()
    );

    assert_eq!(
        sum_variants(&dag, "DescentEvidence"),
        vec![
            (String::from("Strict"), Vec::new()),
            (String::from("NonIncreasing"), Vec::new()),
            (String::from("DescentUnknown"), Vec::new()),
        ]
    );
    assert_eq!(
        sum_variants(&dag, "RankingDimension"),
        vec![
            (String::from("TreeSize"), vec![String::from("param")]),
            (String::from("ListLength"), vec![String::from("param")]),
            (String::from("ArithmeticValue"), vec![String::from("param")]),
            (String::from("TokenPosition"), vec![String::from("param")]),
            (String::from("SetCardinality"), vec![String::from("param")]),
        ]
    );
    assert_eq!(
        sum_variants(&dag, "DescentSource"),
        vec![
            (
                String::from("ChildAccessor"),
                vec![String::from("accessor")]
            ),
            (String::from("ListShrink"), vec![String::from("amount")]),
            (
                String::from("ArithmeticDecrease"),
                vec![String::from("op"), String::from("by")],
            ),
            (String::from("ParserAdvance"), vec![String::from("witness")]),
            (String::from("SetRemoval"), vec![String::from("element")]),
            (String::from("FoldIteration"), Vec::new()),
        ]
    );
    assert_eq!(record_fields(&dag, "TerminationProof"), vec!["dimensions"]);
    assert_eq!(
        record_fields(&dag, "ProofEdge"),
        vec!["caller", "callee", "evidence"]
    );
}

#[test]
fn termination_lattice_functions_lower_with_bodies() {
    let dag = Dag::new();

    for name in [
        "evidence_rank",
        "merge_evidence",
        "join_evidence",
        "promote_to_strict",
        "optional_evidence_meet",
        "map_evidence_merge_at",
    ] {
        assert!(
            matches!(arrow_body(&dag, name), ArrowBody::Unparsed(_)),
            "`{name}` should preserve its v3 std body span until std block bodies lower"
        );
    }
}

#[test]
fn termination_lattice_rust_mirror_matches_dag_authority() {
    use DescentEvidence::{DescentUnknown, NonIncreasing, Strict};

    assert_eq!(evidence_rank(Strict), 2);
    assert_eq!(evidence_rank(NonIncreasing), 1);
    assert_eq!(evidence_rank(DescentUnknown), 0);

    for evidence in [Strict, NonIncreasing, DescentUnknown] {
        assert_eq!(merge_evidence(Strict, evidence), evidence);
        assert_eq!(merge_evidence(evidence, Strict), evidence);
        assert_eq!(join_evidence(DescentUnknown, evidence), evidence);
        assert_eq!(join_evidence(evidence, DescentUnknown), evidence);
    }

    assert_eq!(merge_evidence(Strict, Strict), Strict);
    assert_eq!(merge_evidence(Strict, NonIncreasing), NonIncreasing);
    assert_eq!(merge_evidence(NonIncreasing, NonIncreasing), NonIncreasing);
    assert_eq!(
        merge_evidence(NonIncreasing, DescentUnknown),
        DescentUnknown
    );

    assert_eq!(join_evidence(NonIncreasing, Strict), Strict);
    assert_eq!(join_evidence(NonIncreasing, NonIncreasing), NonIncreasing);
    assert_eq!(join_evidence(Strict, DescentUnknown), Strict);

    assert_eq!(promote_to_strict(NonIncreasing), Strict);
    assert_eq!(promote_to_strict(Strict), Strict);
    assert_eq!(promote_to_strict(DescentUnknown), DescentUnknown);

    assert_eq!(optional_evidence_meet(None, Some(Strict)), Some(Strict));
    assert_eq!(
        optional_evidence_meet(Some(Strict), Some(NonIncreasing)),
        Some(NonIncreasing)
    );

    let mut base = HashMap::new();
    base.insert(String::from("n"), Strict);
    let merged = map_evidence_merge_at(base, String::from("n"), NonIncreasing);
    assert_eq!(merged.get("n"), Some(&NonIncreasing));
    let inserted = map_evidence_merge_at(merged, String::from("m"), Strict);
    assert_eq!(inserted.get("m"), Some(&Strict));
}
