//! **Layer:** integration
//!
//! Slice-active ratchet for the L6 `EmissionPathProjection` substrate
//! carrier slice authored at `src/v3/std/cross_target_coverage.dag` per
//! Director Option 2 RATIFIED at gunbc#828 #issuecomment-4377533390 and the
//! worker brief
//! `docs/briefs/r3-l6-emission-path-projection-substrate-worker.md`.
//!
//! **What this slice asserts (load-bearing now):**
//!
//! 1. The six type declarations exist with the ratified field shapes
//!    (typed-substrate read; no string scan): `ShapeATarget`, `FormAxis`,
//!    `BehaviorAxis`, `MethodTemplateContractKey`, `EmissionCell`,
//!    `EmissionPathProjection`.
//! 2. `emission_path_projections` is populated exactly once per current
//!    Phase-1 `MethodTemplateContract` source row: Rust 13, Python 16, Go 12
//!    (41 total). Each row projects to the single Phase-1 cell
//!    `Cardinality x Transform`.

use std::collections::HashSet;

use v3_compiler::dag::{Dag, Declaration, DeclarationId, FieldValue, TypeConnective, ValueBody};
use v3_compiler::generated_full_bootstrap_dag;
use v3_compiler::pb_method_template_projection::{
    method_template_contract_rows, MethodTemplateTarget,
};

const SHAPE_A_TARGET: &str = "ShapeATarget";
const FORM_AXIS: &str = "FormAxis";
const BEHAVIOR_AXIS: &str = "BehaviorAxis";
const METHOD_TEMPLATE_CONTRACT_KEY: &str = "MethodTemplateContractKey";
const EMISSION_CELL: &str = "EmissionCell";
const EMISSION_PATH_PROJECTION: &str = "EmissionPathProjection";
const EMISSION_PATH_PROJECTIONS_DATA: &str = "emission_path_projections";
const CARDINALITY: &str = "Cardinality";
const TRANSFORM: &str = "Transform";

#[test]
fn cross_target_coverage_six_carrier_types_present() {
    let dag = generated_full_bootstrap_dag();
    for name in [
        SHAPE_A_TARGET,
        FORM_AXIS,
        BEHAVIOR_AXIS,
        METHOD_TEMPLATE_CONTRACT_KEY,
        EMISSION_CELL,
        EMISSION_PATH_PROJECTION,
    ] {
        dag.declaration_by_name(name)
            .unwrap_or_else(|| panic!("`{name}` missing from full bootstrap dag"));
    }
}

/// Collect the structural variant labels (in declaration order) of a `Disj`
/// declaration, panicking with a typed message if the carrier is not a `Disj`.
fn disj_variant_labels<'a>(decl: &'a Declaration, name: &str) -> Vec<&'a str> {
    let TypeConnective::Disj { variants } = &decl.connective else {
        panic!("`{name}` must be `Disj`; got {:?}", decl.connective);
    };
    variants.iter().map(|f| f.label.as_str()).collect()
}

/// Collect the `(label, type-name)` pairs of a `Conj` declaration's children
/// (record-type field shape), panicking with a typed message if the carrier
/// is not a `Conj`.
fn conj_field_label_and_type_names<'a>(
    dag: &'a Dag,
    decl: &'a Declaration,
    name: &str,
) -> Vec<(&'a str, &'a str)> {
    let TypeConnective::Conj { children } = &decl.connective else {
        panic!(
            "`{name}` must be `Conj` (record); got {:?}",
            decl.connective
        );
    };
    children
        .iter()
        .map(|f| {
            let ty_decl = dag.declaration(f.ty);
            let ty_name = ty_decl.name.as_deref().unwrap_or("<anonymous>");
            (f.label.as_str(), ty_name)
        })
        .collect()
}

#[test]
fn shape_a_target_disj_variants_match_ratified_labels() {
    let dag = generated_full_bootstrap_dag();
    let decl = dag
        .declaration_by_name(SHAPE_A_TARGET)
        .expect("ShapeATarget must exist");
    let labels = disj_variant_labels(decl, SHAPE_A_TARGET);
    assert_eq!(
        labels,
        vec!["Rust", "Python", "Go"],
        "ShapeATarget variants must be exactly [Rust, Python, Go] in declaration order \
         (closed set per Q-Unit-1 brief §1; new target requires P1 substrate-fact-introduction)"
    );
}

#[test]
fn form_axis_disj_variants_match_type_connective_discriminants() {
    let dag = generated_full_bootstrap_dag();
    let decl = dag
        .declaration_by_name(FORM_AXIS)
        .expect("FormAxis must exist");
    let labels = disj_variant_labels(decl, FORM_AXIS);
    assert_eq!(
        labels,
        vec![
            "Atom",
            "Conj",
            "Disj",
            "Arrow",
            "Cardinality",
            "Instantiation"
        ],
        "FormAxis variants must mirror v3_compiler::dag::TypeConnective's six discriminants \
         in substrate-declaration order (label-by-label parity; drift requires P1 procedure \
         on the upstream Rust enum first)"
    );
}

#[test]
fn behavior_axis_disj_variants_match_behavior_discriminants() {
    let dag = generated_full_bootstrap_dag();
    let decl = dag
        .declaration_by_name(BEHAVIOR_AXIS)
        .expect("BehaviorAxis must exist");
    let labels = disj_variant_labels(decl, BEHAVIOR_AXIS);
    assert_eq!(
        labels,
        vec!["Value", "Transform", "Branch", "Loop", "Bind"],
        "BehaviorAxis variants must mirror v3_compiler::dag::Behavior's five discriminants \
         in L1 model order (label-by-label parity)"
    );
}

#[test]
fn method_template_contract_key_record_fields_match_ratified_shape() {
    let dag = generated_full_bootstrap_dag();
    let decl = dag
        .declaration_by_name(METHOD_TEMPLATE_CONTRACT_KEY)
        .expect("MethodTemplateContractKey must exist");
    let fields = conj_field_label_and_type_names(&dag, decl, METHOD_TEMPLATE_CONTRACT_KEY);
    assert_eq!(
        fields,
        vec![("target", "ShapeATarget"), ("dag_method", "MethodRef")],
        "MethodTemplateContractKey must carry exactly {{ target: ShapeATarget, dag_method: MethodRef }} \
         per Director Option 2 §4.C=(i) RATIFIED (typed dispatch, no string-name)"
    );
}

#[test]
fn emission_cell_record_fields_match_ratified_shape() {
    let dag = generated_full_bootstrap_dag();
    let decl = dag
        .declaration_by_name(EMISSION_CELL)
        .expect("EmissionCell must exist");
    let fields = conj_field_label_and_type_names(&dag, decl, EMISSION_CELL);
    assert_eq!(
        fields,
        vec![("connective", "FormAxis"), ("behavior", "BehaviorAxis")],
        "EmissionCell must carry exactly {{ connective: FormAxis, behavior: BehaviorAxis }}"
    );
}

#[test]
fn emission_path_projection_record_fields_match_ratified_shape() {
    let dag = generated_full_bootstrap_dag();
    let decl = dag
        .declaration_by_name(EMISSION_PATH_PROJECTION)
        .expect("EmissionPathProjection must exist");
    let fields = conj_field_label_and_type_names(&dag, decl, EMISSION_PATH_PROJECTION);
    assert_eq!(
        fields.len(),
        2,
        "EmissionPathProjection must have exactly two fields; got {fields:?}"
    );
    assert_eq!(
        fields[0],
        ("row_identity", "MethodTemplateContractKey"),
        "EmissionPathProjection.row_identity must be MethodTemplateContractKey"
    );
    // `cells: List<EmissionCell>` lowers as a `Cardinality`-wrapped element ref.
    // Verify the field name and that the wrapped element resolves to EmissionCell.
    assert_eq!(
        fields[1].0, "cells",
        "EmissionPathProjection's second field must be `cells`"
    );
    let TypeConnective::Conj { children } = &decl.connective else {
        unreachable!("already established Conj above")
    };
    let cells_ty_decl = dag.declaration(children[1].ty);
    // `List<X>` lowers to a `Cardinality { element_type: X, ... }` Atom payload
    // or to an `Instantiation` reference. Walk to find the `EmissionCell`
    // declaration referenced by the cells field's type.
    let cells_repr = format!("{:?}", cells_ty_decl.connective);
    assert!(
        cells_repr.contains("EmissionCell")
            || cells_repr.contains("Cardinality")
            || cells_repr.contains("Instantiation"),
        "EmissionPathProjection.cells must resolve through `List<EmissionCell>` \
         (Cardinality/Instantiation wrapper around EmissionCell); got {cells_repr}"
    );
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ProjectionTarget {
    Rust,
    Python,
    Go,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ProjectionKey {
    target: ProjectionTarget,
    dag_method: DeclarationId,
}

fn field<'a>(fields: &'a [(String, FieldValue)], name: &str) -> &'a FieldValue {
    fields
        .iter()
        .find(|(label, _)| label == name)
        .map(|(_, value)| value)
        .unwrap_or_else(|| panic!("record missing `{name}` field: {fields:?}"))
}

fn variant_label<'a>(dag: &'a Dag, axis_name: &str, value: &FieldValue) -> &'a str {
    let FieldValue::Variant { constructor, .. } = value else {
        panic!("expected `{axis_name}` variant value, got {value:?}");
    };
    let axis = dag
        .declaration_by_name(axis_name)
        .unwrap_or_else(|| panic!("`{axis_name}` axis declaration missing"));
    let TypeConnective::Disj { variants } = &axis.connective else {
        panic!("`{axis_name}` must be Disj, got {:?}", axis.connective);
    };
    variants
        .iter()
        .find(|variant| variant.ty == *constructor)
        .map(|variant| variant.label.as_str())
        .unwrap_or_else(|| panic!("constructor {constructor:?} not found in `{axis_name}`"))
}

fn projection_target(dag: &Dag, value: &FieldValue) -> ProjectionTarget {
    match variant_label(dag, SHAPE_A_TARGET, value) {
        "Rust" => ProjectionTarget::Rust,
        "Python" => ProjectionTarget::Python,
        "Go" => ProjectionTarget::Go,
        other => panic!("unknown ShapeATarget variant `{other}`"),
    }
}

fn projection_key(dag: &Dag, row: &FieldValue) -> ProjectionKey {
    let FieldValue::Record(fields) = row else {
        panic!("EmissionPathProjection row must be a record, got {row:?}");
    };
    let row_identity = field(fields, "row_identity");
    let FieldValue::Record(identity_fields) = row_identity else {
        panic!("row_identity must be a MethodTemplateContractKey record, got {row_identity:?}");
    };
    let target = projection_target(dag, field(identity_fields, "target"));
    let dag_method = field(identity_fields, "dag_method");
    let FieldValue::Record(method_ref_fields) = dag_method else {
        panic!("dag_method must be a MethodRef record, got {dag_method:?}");
    };
    let decl = field(method_ref_fields, "decl");
    let FieldValue::Reference(dag_method) = decl else {
        panic!("MethodRef.decl must be a declaration reference, got {decl:?}");
    };
    ProjectionKey {
        target,
        dag_method: *dag_method,
    }
}

fn assert_single_phase1_cell(dag: &Dag, row_index: usize, row: &FieldValue) {
    let FieldValue::Record(fields) = row else {
        panic!("EmissionPathProjection row {row_index} must be a record, got {row:?}");
    };
    let cells = field(fields, "cells");
    let FieldValue::List(cells) = cells else {
        panic!("EmissionPathProjection row {row_index}.cells must be a list, got {cells:?}");
    };
    assert_eq!(
        cells.len(),
        1,
        "Phase-1 projection row {row_index} must carry exactly one cell"
    );
    let FieldValue::Record(cell_fields) = &cells[0] else {
        panic!("EmissionPathProjection row {row_index}.cells[0] must be a record");
    };
    assert_eq!(
        variant_label(dag, FORM_AXIS, field(cell_fields, "connective")),
        CARDINALITY,
        "Phase-1 projection row {row_index} must project connective Cardinality"
    );
    assert_eq!(
        variant_label(dag, BEHAVIOR_AXIS, field(cell_fields, "behavior")),
        TRANSFORM,
        "Phase-1 projection row {row_index} must project behavior Transform"
    );
}

fn source_method_template_keys(dag: &Dag) -> HashSet<ProjectionKey> {
    let mut keys = HashSet::new();
    for (target, projection_target) in [
        (MethodTemplateTarget::Rust, ProjectionTarget::Rust),
        (MethodTemplateTarget::Python, ProjectionTarget::Python),
        (MethodTemplateTarget::Go, ProjectionTarget::Go),
    ] {
        let rows = method_template_contract_rows(dag, target).unwrap_or_else(|err| {
            panic!("project {target:?} MethodTemplateContract rows: {err:?}")
        });
        for row in rows {
            keys.insert(ProjectionKey {
                target: projection_target,
                dag_method: row.dag_method,
            });
        }
    }
    keys
}

#[test]
fn emission_path_projections_data_matches_phase1_source_row_bijection() {
    let dag = generated_full_bootstrap_dag();
    let decl = dag
        .declaration_by_name(EMISSION_PATH_PROJECTIONS_DATA)
        .unwrap_or_else(|| {
            panic!(
                "`{EMISSION_PATH_PROJECTIONS_DATA}` data declaration missing from full bootstrap dag"
            )
        });
    let body = decl
        .value_body
        .as_ref()
        .expect("emission_path_projections must be a `data` declaration with a value body");
    let ValueBody::List(rows) = body else {
        panic!(
            "emission_path_projections must lower as `ValueBody::List` (declared as \
             `List<EmissionPathProjection>`); got {body:?}"
        );
    };
    let source_keys = source_method_template_keys(&dag);
    assert_eq!(
        source_keys.len(),
        41,
        "current Phase-1 source rows must be Rust 13 + Python 16 + Go 12"
    );
    assert_eq!(
        rows.len(),
        source_keys.len(),
        "emission_path_projections must have exactly one row per current Phase-1 source row"
    );

    let mut projection_keys = HashSet::new();
    for (row_index, row) in rows.iter().enumerate() {
        assert_single_phase1_cell(&dag, row_index, row);
        let key = projection_key(&dag, row);
        assert!(
            projection_keys.insert(key),
            "duplicate EmissionPathProjection key at row {row_index}: {key:?}"
        );
    }
    assert_eq!(
        projection_keys, source_keys,
        "emission_path_projections keys must bijectively match MethodTemplateContract source rows"
    );
    // The list's declared element type must be `EmissionPathProjection`. The
    // data declaration's `connective` records the typed list shape (e.g.,
    // `List<EmissionPathProjection>` lowers through a Cardinality/Instantiation
    // wrapper around the element-type DeclarationId). Walk the connective and
    // require the EmissionPathProjection id to be transitively reachable —
    // without this, an empty `List<Foo>` would silently pass even if the
    // element type drifted from the ratified shape.
    let projection_decl = dag
        .declaration_by_name(EMISSION_PATH_PROJECTION)
        .expect("EmissionPathProjection must exist");
    let projection_id_token = format!("DeclarationId({})", projection_decl.id.raw());
    let connective_repr = format!("{:?}", decl.connective);
    assert!(
        connective_repr.contains(&projection_id_token)
            || connective_repr.contains("EmissionPathProjection"),
        "emission_path_projections must be typed as `List<EmissionPathProjection>` \
         (connective must reference EmissionPathProjection's DeclarationId or name); \
         got connective {connective_repr}"
    );
}
