//! **Layer:** integration
//!
//! Acceptance for the PB-Bootstrap-Process consumer hook
//! (`pb_method_template_projection`) lands as part of **R3 row 85 / PB #1560
//! Gap 4** per `docs/decisions/r3-row85-method-template-read-surface.md`.
//!
//! What this exercises:
//!
//! 1. The hook reads the **same full bootstrap `Dag` projection** that the
//!    row-authority decision names (`generated_full_bootstrap_dag()`); no
//!    parallel template-text source.
//! 2. All five `MethodTemplateContract` fields (`dag_method`,
//!    `runtime_template`, `emit_template`, `wraps_result`,
//!    `placeholder_convention`) land on every projected row (Gap 4 A4).
//! 3. The sum carriers (`MethodEmitTemplate`, `PlaceholderConvention`) are
//!    preserved as constructor-tagged enums, not flattened to strings — so
//!    no `Map<String, String>` template-text authority leaks through the
//!    public API (Gap 4 A2; type-level proof, not a grep ratchet).
//! 4. Spot-check: the `count` row for the Rust target has the legacy
//!    runtime/emit pair drift documented in
//!    `src/v3/std/rust_method_template_contracts.dag` (`{recv}.len()` vs
//!    `({recv}.len() as i64)`). This anchors the projection against real
//!    row text without reproducing template strings as test fixtures.
//!
//! Out of scope (per the dispatch / packet §5.3):
//! - `LanguageSpec` rewrite (Gap 5).
//! - v2 leaf-emit migration.
//! - Row population (the `string_contains` / Go `chars` parity gaps).

use v3_compiler::generated_full_bootstrap_dag;
use v3_compiler::pb_method_template_projection::{
    method_template_contract_row, method_template_contract_rows, MethodDeclarationBindingViolation,
    MethodEmitTemplateProjection, MethodTemplateProjectionError, MethodTemplateTarget,
    PlaceholderConventionProjection,
};

#[test]
fn projects_every_target_with_all_five_fields() {
    let dag = generated_full_bootstrap_dag();

    for target in [
        MethodTemplateTarget::Rust,
        MethodTemplateTarget::Python,
        MethodTemplateTarget::Go,
    ] {
        let rows = method_template_contract_rows(&dag, target).unwrap_or_else(|err| {
            panic!("projection for {target:?} returned typed error: {err:?}")
        });
        assert!(
            !rows.is_empty(),
            "projection for {target:?} returned zero rows — \
             per-target list authority must be populated in the bootstrap snapshot"
        );

        for (index, row) in rows.iter().enumerate() {
            // dag_method is a structural reference into the bootstrap Dag.
            // Resolving it must succeed; it is the substrate identity for the
            // row's MethodDeclaration.
            let _decl = dag.declaration(row.dag_method);

            // runtime_template is a non-empty string for every populated row.
            assert!(
                !row.runtime_template.is_empty(),
                "{target:?} row {index}: runtime_template must not be empty"
            );

            // emit_template's variant payload carries non-empty templates.
            // The match preserves the substrate sum identity; consumers see
            // the variant, not a flattened string.
            match &row.emit_template {
                MethodEmitTemplateProjection::Single { template } => {
                    assert!(
                        !template.is_empty(),
                        "{target:?} row {index}: SingleTemplate.template must not be empty"
                    );
                }
                MethodEmitTemplateProjection::HigherOrder {
                    inline_template,
                    fn_ref_template,
                } => {
                    assert!(
                        !inline_template.is_empty(),
                        "{target:?} row {index}: HigherOrderTemplates.inline_template must not be empty"
                    );
                    assert!(
                        !fn_ref_template.is_empty(),
                        "{target:?} row {index}: HigherOrderTemplates.fn_ref_template must not be empty"
                    );
                }
            }

            // placeholder_convention's variant identity must be one of the
            // two declared inhabitants. Pattern match enforces this
            // exhaustively at the type level.
            let _convention = match row.placeholder_convention {
                PlaceholderConventionProjection::IndexedArgs => "IndexedArgs",
                PlaceholderConventionProjection::NamedArg => "NamedArg",
            };

            // wraps_result is a typed bool. Asserting via a redundant pattern
            // simply records that the field is statically present in the
            // projection — a five-field acceptance receipt.
            let _wraps: bool = row.wraps_result;
        }
    }
}

#[test]
fn projection_preserves_per_target_dag_method_uniqueness() {
    // Mirrors the substrate-side claim
    // `method_template_contract_per_target_dag_method_unique` (named in the
    // row-85 decision doc §"Non-Fork Ratchet" item 2) but verifies it
    // through the public projection API rather than walking the Dag's value
    // body directly. If the projection drifts from the substrate carrier,
    // this test fails before any v2 consumer would.
    let dag = generated_full_bootstrap_dag();

    for target in [
        MethodTemplateTarget::Rust,
        MethodTemplateTarget::Python,
        MethodTemplateTarget::Go,
    ] {
        let rows = method_template_contract_rows(&dag, target).expect("projection");
        let mut seen = std::collections::HashSet::new();
        for (index, row) in rows.iter().enumerate() {
            assert!(
                seen.insert(row.dag_method),
                "{target:?} row {index}: duplicate dag_method — projection \
                 contradicts per-target uniqueness substrate claim"
            );
        }
    }
}

#[test]
fn rust_count_row_anchors_runtime_emit_drift() {
    // Spot-check: the Rust `count_method` row's documented runtime/emit
    // template drift is preserved by the projection. This grounds the
    // projection on a row that human reviewers can verify by reading
    // `src/v3/std/rust_method_template_contracts.dag` lines 84–90.
    let dag = generated_full_bootstrap_dag();
    let rows = method_template_contract_rows(&dag, MethodTemplateTarget::Rust).expect("projection");

    let count_method_id = dag
        .declaration_by_name("count_method")
        .expect("count_method MethodDeclaration in bootstrap Dag")
        .id;

    let count_row = rows
        .iter()
        .find(|row| row.dag_method == count_method_id)
        .expect("count_method row in rust_method_template_contracts");

    assert_eq!(count_row.runtime_template, "{recv}.len()");
    let MethodEmitTemplateProjection::Single { template } = &count_row.emit_template else {
        panic!("count row emit_template must be SingleTemplate");
    };
    assert_eq!(template, "({recv}.len() as i64)");
    assert!(!count_row.wraps_result);
    assert_eq!(
        count_row.placeholder_convention,
        PlaceholderConventionProjection::NamedArg
    );
}

#[test]
fn lookup_helper_three_distinct_states() {
    // The `(target, dag_method)` direct lookup must distinguish three
    // states cleanly so Gap 5 / leaf-emit consumers cannot conflate them:
    //
    //   1. Hit:     valid `MethodDeclaration` whose target list has a row.
    //   2. Miss:    valid `MethodDeclaration` whose target list has no row.
    //   3. Invalid: `dag_method` is not a `MethodDeclaration` at all
    //              → typed `LookupKeyNotMethodDeclaration` error.
    //
    // (1) and (2) are the legitimate `Ok(Some)` / `Ok(None)` axes; (3) is
    // the typed error the helper enforces at the public boundary until
    // `DeclarationRef<MethodDeclaration>` refinement-typing lands.
    let dag = generated_full_bootstrap_dag();

    // (1) Hit: `count_method` is in `dsl/std/methods.dag` and has a row in
    // `rust_method_template_contracts`.
    let count_method_id = dag
        .declaration_by_name("count_method")
        .expect("count_method MethodDeclaration in bootstrap Dag")
        .id;
    let hit = method_template_contract_row(&dag, MethodTemplateTarget::Rust, count_method_id)
        .expect("projection")
        .expect("count_method row present for Rust target");
    assert_eq!(hit.runtime_template, "{recv}.len()");

    // (2) Miss: `add_method` is in `dsl/std/methods.dag` (a real
    // `MethodDeclaration`) but has no row in
    // `rust_method_template_contracts`. Must return `Ok(None)`, not a
    // typed error.
    let add_method_id = dag
        .declaration_by_name("add_method")
        .expect("add_method MethodDeclaration in bootstrap Dag")
        .id;
    let miss = method_template_contract_row(&dag, MethodTemplateTarget::Rust, add_method_id)
        .expect("projection");
    assert!(
        miss.is_none(),
        "valid MethodDeclaration with no Rust row must return None, not fabricate a row"
    );

    // (3) Invalid: `MethodTemplateContract` itself is a type declaration,
    // not a `MethodDeclaration`. Must surface
    // `LookupKeyNotMethodDeclaration` rather than `Ok(None)` (which would
    // conflate it with case 2).
    let non_method_decl_id = dag
        .declaration_by_name("MethodTemplateContract")
        .expect("MethodTemplateContract type")
        .id;
    let invalid =
        method_template_contract_row(&dag, MethodTemplateTarget::Rust, non_method_decl_id);
    match invalid {
        Err(MethodTemplateProjectionError::LookupKeyNotMethodDeclaration {
            decl_id,
            reason,
        }) => {
            assert_eq!(decl_id, non_method_decl_id);
            // `MethodTemplateContract` is a type declaration (Conj), not
            // an Instantiation — connective sub-check fails first.
            assert_eq!(
                reason,
                MethodDeclarationBindingViolation::ConnectiveNotInstantiation
            );
        }
        other => panic!(
            "lookup with a non-MethodDeclaration key must surface LookupKeyNotMethodDeclaration, got {other:?}"
        ),
    }
}
