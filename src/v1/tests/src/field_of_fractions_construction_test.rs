use std::rc::Rc;

use v1_compiler::cli_run;
use v1_compiler::v1_compiler_compile::{compile_to_resolved, ResolvedPipelineResult, SourceFile};
use v1_compiler::v1_interpreter::{self, ExecutionMode, Value};
use v1_compiler::v1_std_core::Node;

use crate::helpers::{parse_source_named, workspace_root};

// Discriminating witness for the §7.2 FieldOfFractions model grounding (sharp-bee-290
// sign-off): `FieldOfFractions<R> = { num: R, denom: R }` is a real 2-field record at its
// single authority (dag/std/algebra.dag). Unlike GroupCompletion (#7197), this is
// deliberately negative-space: FieldOfFractions has no native Rust scalar checkpoint to
// collapse into (Rational has no lossless native representation — an f64 collapse would be
// dishonest, per the design doc), so `eval_record_lit` must NOT special-case it.
//
// Those are TWO claims with two different subjects, and this module states them separately
// rather than fusing them behind one whole-corpus resolve:
//
//   1. the real authority declares the pair -- read from dag/std/algebra.dag itself, so a
//      regression that hollows the type out is caught against the authority, not a copy;
//   2. a record literal of that name stays boxed -- a RUNTIME property of `eval_record_lit`,
//      which discriminates purely on the type name string (`type_name == "GroupCompletion"`,
//      `== "Succ"`). Nothing in that arm consults a resolved declaration, so a minimal
//      same-name specimen exercises the exact branch the real type would take.
//
// The fixture below is therefore a discriminating specimen, not a second authority: claim 1
// independently reads the real declaration, so the fixture can never become the only thing
// this suite asserts about the type's shape.
//
// What was removed, deliberately, and why it is not a coverage loss: claim 2 previously
// resolved the transitive `src/v2` + `dag` closure and asserted the whole context was
// diagnostic-clean before touching the interpreter at all. Whole-corpus resolution is not
// this witness's subject -- it is the compile-clean gate's -- and carrying it here made a
// runtime one-liner depend on a ~1,100-module resolve.

const ALGEBRA_REL: &str = "dag/std/algebra.dag";

// A local declaration of the same name. `eval_record_lit` keys its collapse on the type
// name alone, so this reaches the identical branch as the real authority would.
const RUNTIME_SPECIMEN: &str = r#"
module test.field_of_fractions_construction

type FieldOfFractions<R> {
  num: R
  denom: R
}

fn one_half() -> FieldOfFractions<Int> { FieldOfFractions { num: 1, denom: 2 } }
fn three_quarters() -> FieldOfFractions<Int> { FieldOfFractions { num: 3, denom: 4 } }
"#;

fn assert_resolved(resolved: &ResolvedPipelineResult) {
    let msgs: Vec<String> = resolved
        .diagnostics
        .iter()
        .map(|d| v1_compiler::v1_std_core::diagnostic_to_message(d.diagnostic.clone()))
        .filter(|m| !m.starts_with("complexity: "))
        .collect();
    assert!(
        msgs.is_empty() && resolved.graph.is_some(),
        "specimen should resolve cleanly, got {:?} (graph present: {})",
        msgs,
        resolved.graph.is_some(),
    );
}

/// Depth-first search for a declaration node carrying `name`.
fn find_decl(node: &Rc<Node>, name: &str) -> Option<Rc<Node>> {
    if node.name == name {
        return Some(node.clone());
    }
    for child in node.children.iter() {
        if let Some(found) = find_decl(child, name) {
            return Some(found);
        }
    }
    None
}

#[test]
fn field_of_fractions_authority_declares_num_and_denom() {
    let source = std::fs::read_to_string(workspace_root().join(ALGEBRA_REL))
        .unwrap_or_else(|e| panic!("read {ALGEBRA_REL}: {e}"));
    let parsed = parse_source_named(ALGEBRA_REL, &source);
    assert!(
        parsed.error.is_none(),
        "{ALGEBRA_REL} should parse, got {:?}",
        parsed.error
    );
    let root = parsed
        .module
        .as_ref()
        .unwrap_or_else(|| panic!("{ALGEBRA_REL} should produce a module"));
    let decl = find_decl(root, "FieldOfFractions")
        .unwrap_or_else(|| panic!("{ALGEBRA_REL} should declare FieldOfFractions"));
    let fields: Vec<&str> = decl.children.iter().map(|c| c.name.as_str()).collect();
    // A hollow (bodyless) declaration is the regression this guards: it is the shape the
    // type had before #7210 grounded it, and it emits as PhantomData rather than a pair.
    assert!(
        fields.contains(&"num") && fields.contains(&"denom"),
        "FieldOfFractions at its single authority should declare num and denom, got {fields:?}",
    );
}

/// Negative control for the assertion above.
///
/// The shape assertion reads whatever `find_decl` returns, so on its own a green result is
/// consistent with the walker having found some other node, or with a hollow declaration
/// whose emptiness nobody looks at. This applies the identical read to a deliberately
/// bodyless declaration of the same name and shows it reports NO fields -- so the green in
/// `field_of_fractions_authority_declares_num_and_denom` is a fact about
/// `dag/std/algebra.dag`, not about the read being unable to fail.
///
/// The hollow shape is not hypothetical: it is what the type was before #7210 grounded it,
/// and it emits as `PhantomData` rather than a pair.
#[test]
fn the_shape_assertion_reports_a_hollow_declaration_as_fieldless() {
    const HOLLOW: &str = "module test.hollow\n\ntype FieldOfFractions<R> {\n}\n";
    let parsed = parse_source_named("hollow.dag", HOLLOW);
    assert!(
        parsed.error.is_none(),
        "hollow specimen should parse, got {:?}",
        parsed.error
    );
    let root = parsed.module.as_ref().expect("hollow specimen module");
    let decl = find_decl(root, "FieldOfFractions").expect("hollow specimen declares the name");
    let fields: Vec<&str> = decl.children.iter().map(|c| c.name.as_str()).collect();
    assert!(
        !(fields.contains(&"num") && fields.contains(&"denom")),
        "a bodyless declaration must not satisfy the shape assertion, got {fields:?}",
    );
}

#[test]
fn field_of_fractions_pair_stays_boxed_record_not_native_collapse() {
    let source = Rc::new(SourceFile {
        path: "test.dag".to_string(),
        content: RUNTIME_SPECIMEN.to_string(),
    });
    let resolved = compile_to_resolved(Rc::new(vec![source].into()));
    assert_resolved(&resolved);
    let graph = resolved.graph.as_ref().expect("graph");
    let ctx =
        cli_run::make_eval_context(graph, resolved.source_indices.clone(), ExecutionMode::Wet);

    for (f, expected_num, expected_denom) in
        [("one_half", 1i64, 2i64), ("three_quarters", 3i64, 4i64)]
    {
        match v1_interpreter::run_in_context(&ctx, f, false) {
            Ok(Value::Record { type_name, fields }) => {
                assert!(
                    ctx.sym_eq(type_name, "FieldOfFractions"),
                    "{f}: expected type_name FieldOfFractions, got {}",
                    ctx.resolve(type_name)
                );
                match ctx.field(&fields, "num") {
                    Some(Value::Int(n)) if *n == expected_num => {}
                    other => panic!("{f}: num field mismatch, got {other:?}"),
                }
                match ctx.field(&fields, "denom") {
                    Some(Value::Int(n)) if *n == expected_denom => {}
                    other => panic!("{f}: denom field mismatch, got {other:?}"),
                }
            }
            other => panic!(
                "{f}: expected a boxed Value::Record{{num, denom}} — FieldOfFractions has \
                 no native Rust scalar to collapse into, so a regression that special-cases \
                 it into any native Value variant (Int, Float, or otherwise) surfaces here; \
                 got {other:?}"
            ),
        }
    }
}
