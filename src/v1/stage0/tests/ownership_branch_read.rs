//! Ownership branch-join controls on the emitted seed.
//! Execute: cargo test -p v1-compiler --test ownership_branch_read.
//! Required clippy --all-targets compiles this target; no CI step executes it.
//! The declared drop is gunbc.rung_drop.rust_unit_tests_off_the_merge_path.
//! Return the fixtures to .dag when a required seed-rooted claim phase supplies
//! same-identity, candidate-bound terminals to the disposition receipt.

use std::rc::Rc;
use v1_compiler::v1_compiler_artifact::RenderTarget;
use v1_compiler::v1_compiler_compile::{compile_sources, SourceFile};
use v1_compiler::v1_compiler_ownership::{
    build_movable_set, max_usage_by_fan_out, BindingUsage, EdgeClassification, EdgeKind,
    OwnershipProof,
};

fn usage(edges: &[(EdgeKind, &str, i64)]) -> Rc<BindingUsage> {
    Rc::new(BindingUsage {
        name: "item".into(),
        binding_kind: None,
        consumers: Rc::new(
            edges
                .iter()
                .map(|(kind, site, span)| {
                    Rc::new(EdgeClassification {
                        kind: *kind,
                        site: (*site).into(),
                        span_start: *span,
                    })
                })
                .collect(),
        ),
    })
}

fn movable(binding: Rc<BindingUsage>) -> bool {
    let proof = Rc::new(OwnershipProof {
        func_name: "fixture".into(),
        bindings: Rc::new(im::hashmap! { "item".into() => binding }),
        decisions: Rc::new(im::Vector::new()),
        fold_acc_unwrap: Rc::new(im::Vector::new()),
    });
    build_movable_set(proof, Rc::new(im::ordset! { "item".into() })).contains("item")
}

#[test]
fn branch_read_survives_larger_projection_path() {
    use EdgeKind::{Consumed, Projected, Read};
    let read_path = usage(&[(Consumed, "return", 1), (Read, "read", 2)]);
    let projection_path = usage(&[
        (Consumed, "return", 1),
        (Projected, ".a", 3),
        (Projected, ".b", 4),
    ]);
    assert!(!movable(max_usage_by_fan_out(read_path, projection_path)));
}

#[test]
fn projected_before_consumed_stays_movable() {
    use EdgeKind::{Consumed, Projected};
    let path = usage(&[(Projected, ".a", 1), (Consumed, "return", 2)]);
    assert!(movable(max_usage_by_fan_out(path.clone(), path)));
}

#[test]
fn whole_read_then_consume_stays_non_movable() {
    use EdgeKind::{Consumed, Read};
    let path = usage(&[(Read, "read", 1), (Consumed, "return", 2)]);
    assert!(!movable(max_usage_by_fan_out(path.clone(), path)));
}

#[test]
fn authored_constructor_orders_emit_clones() {
    for (case, early, fields) in [
        (
            "baseline",
            "",
            "child: Present { value: item }, text: item.text",
        ),
        (
            "move_first",
            "if early { return item }",
            "child: Present { value: item }, text: item.text",
        ),
        (
            "borrow_first",
            "if early { return item }",
            "text: item.text, child: Present { value: item }",
        ),
    ] {
        let source = format!(
            "module move_probe\ntype Item {{ child: Item? text: String other: String }}\n\
             fn build(item: Item, early: Bool) -> Item {{ {early} if early {{\
             Item {{ {fields}, other: \"\" }} }} else {{\
             Item {{ child: item.child, text: item.text, other: item.other }} }} }}"
        );
        let result = compile_sources(
            Rc::new(im::vector![Rc::new(SourceFile {
                path: "move_probe.dag".into(),
                content: source
            })]),
            RenderTarget::Rust,
        );
        assert!(
            result.diagnostics.is_empty(),
            "{case}: {:?}",
            result.diagnostics
        );
        let module = result
            .files
            .iter()
            .find(|file| file.content.contains("pub fn build("))
            .expect("emitted probe module");
        assert!(
            module.content.contains("child: Some(item.clone())"),
            "{case}: {}",
            module.content
        );
        assert!(
            module.content.contains("text: item.text.clone()"),
            "{case}: {}",
            module.content
        );
    }
}
