#![allow(clippy::disallowed_macros)]

// Control witness for B0 review 49798 / operator ruling on #7943: proves the
// GENERATED stage0 Rust realization of
// std_occurrence_binding_candidates::declaration_exposure_from_containment
// under DeclarationExposureGrounding::NamespaceStructuralRootExposure — not
// the interpreted .dag source — dispatches to three distinct outcomes. This
// is the compiled binary calling the compiled generated function directly,
// so a collapsed if/else in the emitter (the EMIT-NESTED-PATTERN-0 defect
// class, node://adhoc-b4da0e4d-cf3) would surface here even when the
// interpreted-source witnesses stay green.

use std::process::ExitCode;
use std::rc::Rc;

use v1_compiler::std_occurrence_binding_candidates::{
    declaration_exposure_from_containment, DeclarationExposure, DeclarationExposureGrounding,
};
use v1_compiler::std_occurrence_identity::{
    OccurrenceCategory, OccurrenceContainmentPath, OccurrenceId,
};

fn fail(msg: impl std::fmt::Display) -> ExitCode {
    eprintln!("namespace_structural_root_exposure_generated_witness: {msg}");
    ExitCode::from(1)
}

fn occ(value: i64) -> OccurrenceId {
    OccurrenceId { value }
}

fn containment(ancestor_values: &[i64], terminal_value: i64) -> Rc<OccurrenceContainmentPath> {
    Rc::new(OccurrenceContainmentPath {
        ancestors: Rc::new(ancestor_values.iter().copied().map(occ).collect()),
        terminal: occ(terminal_value),
    })
}

fn main() -> ExitCode {
    let module_path = "witness.module".to_string();
    let mut failures: Vec<String> = Vec::new();

    // Empty ancestors -> RootExposure. No enclosing scope: a namespace-root
    // structural declaration.
    let empty = declaration_exposure_from_containment(
        module_path.clone(),
        containment(&[], 1),
        DeclarationExposureGrounding::NamespaceStructuralRootExposure,
        OccurrenceCategory::TypeOccurrence,
    );
    match &*empty {
        DeclarationExposure::RootExposure => println!("PASS empty_ancestors_root_exposure"),
        other => failures.push(format!(
            "empty_ancestors_root_exposure: expected RootExposure, got {:?}",
            other
        )),
    }

    // One ancestor -> ModuleExposure. The sole ancestor is the module itself
    // (its own ancestors are empty), so the declaration is a direct module
    // child.
    let one = declaration_exposure_from_containment(
        module_path.clone(),
        containment(&[10], 11),
        DeclarationExposureGrounding::NamespaceStructuralRootExposure,
        OccurrenceCategory::TypeOccurrence,
    );
    match &*one {
        DeclarationExposure::ModuleExposure { module } if *module == module_path => {
            println!("PASS one_ancestor_module_exposure")
        }
        other => failures.push(format!(
            "one_ancestor_module_exposure: expected ModuleExposure{{module: {module_path:?}}}, got {:?}",
            other
        )),
    }

    // Two ancestors -> LexicalExposure. LOAD-BEARING: this is the arm the
    // generic emitter previously collapsed away (EMIT-NESTED-PATTERN-0) —
    // the enclosing scope itself has a nonempty ancestor chain, so the
    // declaration is lexically nested inside another declaration, not a
    // direct module child.
    let two = declaration_exposure_from_containment(
        module_path.clone(),
        containment(&[20, 21], 22),
        DeclarationExposureGrounding::NamespaceStructuralRootExposure,
        // A lexical binder, not a type: nesting alone no longer decides this arm. A declaration
        // nested inside another is lexically scoped only when its category is not module-scope
        // exposed, so a nested TYPE or CONSTRUCTOR now derives ModuleExposure and would fail here
        // for the right reason.
        OccurrenceCategory::LexicalValueOccurrence,
    );
    match &*two {
        DeclarationExposure::LexicalExposure { exposing_scope } => {
            if exposing_scope.terminal == occ(21) && exposing_scope.ancestors.len() == 1 {
                println!("PASS two_ancestors_lexical_exposure");
            } else {
                failures.push(format!(
                    "two_ancestors_lexical_exposure: unexpected exposing_scope {:?}",
                    exposing_scope
                ));
            }
        }
        other => failures.push(format!(
            "two_ancestors_lexical_exposure: expected LexicalExposure, got {:?}",
            other
        )),
    }

    if failures.is_empty() {
        println!("namespace_structural_root_exposure_generated_witness: all cases PASS");
        ExitCode::SUCCESS
    } else {
        for f in &failures {
            eprintln!("FAIL {}", f);
        }
        fail(format!("{} case(s) failed", failures.len()))
    }
}
