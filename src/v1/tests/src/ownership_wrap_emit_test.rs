//! Seed emitter wrap-decision gate: catalog carriers wrap per use-site;
//! shared_types members absent from the catalog (e.g. user structs) stay bare.
//!
//! Behavioral witness (compile_dag_target): OWNERSHIP-68 /
//! docs/plans/rc-ownership-wrap-decision-design.md — fn-sig, struct-field,
//! alias-RHS, and data-def emit paths.
//!
//! SCAFFOLD deferral (§7 hand-Rust gate): corpus census shrink is NOT in this
//! PR — deferred to lane sharp-bee-290 / Weak→Strong Self Host Gate 3
//! (compile-green stage0 regen fixed point; ROADMAP.md §④ regen_verify +
//! rc-ownership-wrap-decision-design.md implementation sequence step 3).
//! Dissolve-on: regen_verify green with ownership_wrap witnesses enrolled on
//! the affected-set corpus (not fixture-only scoped emit).

use crate::helpers::compile_dag_target;
use v1_compiler::v1_compiler_artifact::RenderTarget;

fn emit(source: &str) -> String {
    compile_dag_target(source, RenderTarget::Rust)
        .files
        .iter()
        .map(|f| f.content.clone())
        .collect::<Vec<_>>()
        .join("\n")
}

fn return_sig(emitted: &str, fn_name: &str) -> String {
    let needle = format!("fn {fn_name}");
    let start = emitted
        .find(&needle)
        .unwrap_or_else(|| panic!("fixture fn `{fn_name}` was not emitted:\n{emitted}"));
    let rest = &emitted[start..];
    let body_open = rest
        .find(" {")
        .unwrap_or_else(|| panic!("no body opener for `{fn_name}`:\n{emitted}"));
    rest[..body_open].to_string()
}

#[test]
fn catalog_node_return_wraps_rc_param_stays_owned() {
    let source = "module ownwrap.fixture\n\ntype Node = Product { child: Node? }\n\nfn pass_node(n: Node) -> Node {\n  n\n}\n";
    let sig = return_sig(&emit(source), "pass_node");
    assert!(
        sig.contains("n: Node") && !sig.contains("n: Rc<Node>"),
        "Node fn param must be owned (catalog row), got:\n{sig}"
    );
    assert!(
        sig.contains("-> Rc<Node>") || sig.contains("-> Rc<"),
        "Node fn return must be Rc-wrapped (catalog row), got:\n{sig}"
    );
}

#[test]
fn shared_type_not_in_catalog_struct_field_stays_bare() {
    let source = "module ownwrap.fixture\n\ntype DigitStep { value: Int }\n\ntype Holder { step: DigitStep }\n";
    let emitted = emit(source);
    assert!(
        !emitted.contains("Rc<DigitStep>"),
        "user struct fields must not get blanket shared_types Rc wrap, got:\n{emitted}"
    );
    assert!(
        emitted.contains("step: DigitStep"),
        "struct field must emit bare DigitStep, got:\n{emitted}"
    );
}

#[test]
fn catalog_node_struct_field_wraps_box() {
    let source =
        "module ownwrap.fixture\n\ntype Node { child: Node? }\n\ntype Tree { child: Node }\n";
    let emitted = emit(source);
    assert!(
        emitted.contains("child: Box<Node>") || emitted.contains("child: Box<"),
        "Node struct field must be Box-wrapped (catalog row), got:\n{emitted}"
    );
    assert!(
        !emitted.contains("child: Rc<Node>"),
        "Node struct field must not be Rc-wrapped, got:\n{emitted}"
    );
}

#[test]
fn catalog_node_struct_field_literal_wraps_box_new() {
    let source = "module ownwrap.fixture\n\ntype Node { child: Node? }\n\ntype Tree { child: Node }\n\ndata tree: Tree = Tree { child: Node { child: none } }\n";
    let emitted = emit(source);
    assert!(
        emitted.contains("Box::new(Node {") || emitted.contains("child: Box::new("),
        "Node struct-field literal value must Box::new-wrap (catalog row), got:\n{emitted}"
    );
}

#[test]
fn catalog_node_generic_struct_field_wraps_box_once() {
    let source =
        "module ownwrap.fixture\n\ntype Node { child: Node? }\n\ntype Tree<T> { child: Node }\n";
    let emitted = emit(source);
    assert!(
        emitted.contains("child: Box<Node>") || emitted.contains("child: Box<"),
        "generic struct Node field must be Box-wrapped once (catalog row), got:\n{emitted}"
    );
    assert!(
        !emitted.contains("Box<Box<Node>>"),
        "generic struct Node field must not double-box, got:\n{emitted}"
    );
}

#[test]
fn alias_rhs_applied_shared_type_no_blanket_rc_wrap() {
    let source =
        "module ownwrap.fixture\n\ntype Node { child: Node? }\n\ntype NodeList = List<Node>\n";
    let emitted = emit(source);
    assert!(
        !emitted.contains("type NodeList = Rc<"),
        "alias RHS must not blanket-wrap shared_types via site-blind Rc, got:\n{emitted}"
    );
    assert!(
        emitted.contains("type NodeList = Vec<Node>")
            || emitted.contains("type NodeList = List<Node>"),
        "alias RHS must emit bare applied List<Node>, got:\n{emitted}"
    );
}

#[test]
fn data_def_catalog_node_return_wraps_rc() {
    let source = "module ownwrap.fixture\n\ntype Node { child: Node? }\n\ndata root_node: Node = Node { child: none }\n";
    let emitted = emit(source);
    let sig = return_sig(&emitted, "root_node");
    assert!(
        sig.contains("-> Rc<Node>") || sig.contains("-> Rc<"),
        "catalog Node data def return must be Rc-wrapped, got:\n{sig}"
    );
}

#[test]
fn data_def_non_catalog_shared_type_stays_bare() {
    let source = "module ownwrap.fixture\n\ntype DigitStep { value: Int }\n\ndata step_zero: DigitStep = DigitStep { value: 0 }\n";
    let emitted = emit(source);
    let sig = return_sig(&emitted, "step_zero");
    assert!(
        !sig.contains("Rc<DigitStep>"),
        "non-catalog data def return must stay bare, got:\n{sig}"
    );
    assert!(
        sig.contains("-> DigitStep"),
        "non-catalog data def return must emit bare DigitStep, got:\n{sig}"
    );
}

#[test]
fn non_catalog_variant_value_no_blanket_rc_new() {
    let source =
        "module ownwrap.fixture\n\ntype Color = Red | Green\n\nfn red() -> Color {\n  Red\n}\n";
    let emitted = emit(source);
    assert!(
        !emitted.contains("Rc::new(Red)") && !emitted.contains("Rc::new(Color::"),
        "non-catalog enum variant value must not blanket-wrap Rc::new, got:\n{emitted}"
    );
    assert!(
        emitted.contains("Red") || emitted.contains("Color::Red"),
        "enum variant value must emit bare variant, got:\n{emitted}"
    );
}

#[test]
fn catalog_node_variant_value_wraps_rc_new() {
    let source = "module ownwrap.fixture\n\ntype Node { child: Node? }\n\ntype Color = Red | Green\n\nfn leaf() -> Node {\n  Node { child: none }\n}\n";
    let emitted = emit(source);
    assert!(
        emitted.contains("Rc::new(Node {") || emitted.contains("Rc::new("),
        "catalog Node construction at binding projection must Rc-wrap, got:\n{emitted}"
    );
}
