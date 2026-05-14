//! **Layer:** integration
//!
//! Structural acceptance for R3 §1.8 gate **`section_ref_substrate_landed`** (T-Lens-Application-Surface):
//! `SectionRef = DeclarationScope { declaration } | NodeScope { declaration, node }` in
//! [`src/v3/std/lens_application.dag`](../../../std/lens_application.dag), per
//! [`docs/design-lens-application-surface.md`](../../../../docs/design-lens-application-surface.md) §1.2–§2.

use v3_compiler::dag::{Dag, DeclarationId, TypeConnective};
use v3_compiler::generated_full_bootstrap_dag;

fn peel_declaration_alias_head(dag: &Dag, mut id: DeclarationId) -> DeclarationId {
    const MAX: usize = 64;
    for _ in 0..MAX {
        match &dag.declaration(id).connective {
            TypeConnective::Instantiation {
                template,
                arguments,
            } if arguments.is_empty() => {
                id = *template;
            }
            TypeConnective::Atom(ap) => {
                let Some(next) = ap.resolved_id() else {
                    break;
                };
                id = next;
            }
            _ => break,
        }
    }
    id
}

fn section_ref_decl(dag: &Dag) -> DeclarationId {
    // Name matches import stubs elsewhere (e.g. `lenses.cost`); the substrate
    // authority for this receipt is the declaration that lowers as the
    // disjoint sum itself—not `ResolvedByName` forwarding atoms.
    let mut substrates: Vec<DeclarationId> = dag
        .declarations()
        .iter()
        .filter(|d| d.name.as_deref() == Some("SectionRef"))
        .filter(|d| matches!(&d.connective, TypeConnective::Disj { .. }))
        .map(|d| d.id)
        .collect();
    substrates.sort_by_key(|id| id.raw());
    match substrates.as_slice() {
        [only] => *only,
        [] => panic!(
            "bootstrap must define `SectionRef` as a disjoint sum (gate #89 section_ref_substrate_landed)"
        ),
        ambiguous => panic!(
            "multiple `SectionRef` disjoint-sum authorities in bootstrap ({ambiguous:?}); expected exactly one substrate definition"
        ),
    }
}

fn disj_variant_conj(dag: &Dag, disj: DeclarationId, label: &str) -> DeclarationId {
    let TypeConnective::Disj { variants } = &dag.declaration(disj).connective else {
        panic!("`SectionRef` must be a disjoint sum");
    };
    let v = variants
        .iter()
        .find(|x| x.label == label)
        .unwrap_or_else(|| panic!("SectionRef missing `{label}` variant"));
    v.ty
}

fn conj_field_entry(dag: &Dag, conj: DeclarationId, field: &str) -> DeclarationId {
    let TypeConnective::Conj { children } = &dag.declaration(conj).connective else {
        panic!("expected Conj payload for SectionRef variant");
    };
    children
        .iter()
        .find(|f| f.label == field)
        .unwrap_or_else(|| panic!("conj missing `{field}` field"))
        .ty
}

#[test]
fn gate_89_section_ref_is_disjoint_sum_from_lens_application_authority() {
    let dag = generated_full_bootstrap_dag();
    let section_ref = section_ref_decl(&dag);

    let TypeConnective::Disj { variants } = &dag.declaration(section_ref).connective else {
        panic!("SectionRef must lower to a Disj");
    };
    let labels: Vec<&str> = variants.iter().map(|v| v.label.as_str()).collect();
    assert_eq!(
        labels,
        vec!["DeclarationScope", "NodeScope"],
        "SectionRef variant labels + order must match design doc §1.2 (`DeclarationScope` then `NodeScope`)"
    );

    let decl_id = dag
        .declaration_by_name("DeclarationId")
        .expect("DeclarationId missing from bootstrap")
        .id;
    let node_id = dag
        .declaration_by_name("NodeId")
        .expect("NodeId missing from bootstrap")
        .id;

    let ds_conj = disj_variant_conj(&dag, section_ref, "DeclarationScope");
    let ds_labels: Vec<String> = match &dag.declaration(ds_conj).connective {
        TypeConnective::Conj { children } => children.iter().map(|f| f.label.clone()).collect(),
        other => panic!("DeclarationScope payload must be Conj: {other:?}"),
    };
    assert_eq!(
        ds_labels,
        vec!["declaration".to_string()],
        "DeclarationScope payload fields drifted"
    );
    assert_eq!(
        peel_declaration_alias_head(&dag, conj_field_entry(&dag, ds_conj, "declaration")),
        peel_declaration_alias_head(&dag, decl_id),
        "`DeclarationScope.declaration` must be substrate `DeclarationId`"
    );

    let ns_conj = disj_variant_conj(&dag, section_ref, "NodeScope");
    let ns_labels: Vec<String> = match &dag.declaration(ns_conj).connective {
        TypeConnective::Conj { children } => children.iter().map(|f| f.label.clone()).collect(),
        other => panic!("NodeScope payload must be Conj: {other:?}"),
    };
    assert_eq!(
        ns_labels,
        vec!["declaration".to_string(), "node".to_string()],
        "NodeScope payload fields drifted"
    );
    assert_eq!(
        peel_declaration_alias_head(&dag, conj_field_entry(&dag, ns_conj, "declaration")),
        peel_declaration_alias_head(&dag, decl_id),
        "`NodeScope.declaration` must be substrate `DeclarationId`"
    );
    assert_eq!(
        peel_declaration_alias_head(&dag, conj_field_entry(&dag, ns_conj, "node")),
        peel_declaration_alias_head(&dag, node_id),
        "`NodeScope.node` must be substrate `NodeId`"
    );
}
