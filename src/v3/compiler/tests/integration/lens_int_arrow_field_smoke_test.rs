//! **Layer:** integration
//!
//! Lens-instance migration **readiness** (inbox #1130 / #1139 follow-on):
//! after #1230/#1232, prove the **Prereq-1** path from
//! `docs/design-lens-fold-prerequisites.md` — Arrow-typed `Lens<C>` record
//! fields accept top-level `fn` **declaration references** via
//! `lower_structural_field_value` — without authoring `data complexity_lens`
//! or hand-Rust scaffolding.
//!
//! This is **not** a `complexity_lens_via_framework_correct` substitute:
//! `complexity_read` still needs Prereq-2 block-body / variant-constructor
//! lowering for a real behavioral `Witness<Int>` fold.

use v3_compiler::compile_to_dag;
use v3_compiler::dag::{Declaration, FieldValue, ValueBody};
use v3_compiler::CompileError;

const LENS_INT_ARROW_FIELDS: &str = r#"
module test.lens_int_arrow_field_smoke

import v3.std.lens { Lens }
import std.substrate { Dag, Behavior, LoopBound }
import std.types { Int }
import v3.std.dimensions { Witness, OptionalDiagnostic }

fn int_add(a: Int, b: Int) -> Int = a + b
fn int_max(a: Int, b: Int) -> Int = if a > b then a else b

fn dummy_read(d: Dag, b: Behavior) -> Witness<Int> = Inhabits(0)

fn dummy_iter(c: Int, bound: LoopBound) -> Int = c

fn dummy_val(d: Dag, c: Int) -> OptionalDiagnostic = NoDiagnostic

data smoke_lens: Lens<Int> = {
  name: "smoke",
  read: dummy_read,
  sequential: { op: int_add, identity: 0 },
  branch: int_max,
  iterate: dummy_iter,
  validate: dummy_val
}
"#;

fn field_value<'a>(decl: &'a Declaration, label: &str) -> Option<&'a FieldValue> {
    let ValueBody::Structural { fields } = decl.value_body.as_ref()? else {
        return None;
    };
    fields.iter().find(|(l, _)| l == label).map(|(_, v)| v)
}

#[test]
fn lens_int_data_assigns_arrow_fields_via_fn_declaration_references() {
    let dag = match compile_to_dag(LENS_INT_ARROW_FIELDS, "lens_int_arrow_field_smoke.v3") {
        Ok(d) => d,
        Err(CompileError::Semantic(d)) => panic!(
            "expected clean compile, got diagnostics: {:?}",
            d.diagnostics().iter().collect::<Vec<_>>()
        ),
        Err(e) => panic!("unexpected compile error: {e:?}"),
    };
    assert!(
        dag.diagnostics().is_empty(),
        "smoke module should be diagnostic-clean: {:?}",
        dag.diagnostics().iter().collect::<Vec<_>>()
    );

    let smoke = dag
        .declaration_by_name("smoke_lens")
        .expect("smoke_lens data declaration");

    assert!(
        matches!(field_value(smoke, "read"), Some(FieldValue::Reference(_))),
        "`read: fn(Dag, Behavior) -> Witness<Int>` field must lower to a fn Reference"
    );

    let seq = field_value(smoke, "sequential").expect("sequential field");
    let FieldValue::Record(seq_fields) = seq else {
        panic!("`sequential: Monoid<Int>` must lower to a nested record: {seq:?}");
    };
    let op = seq_fields
        .iter()
        .find(|(l, _)| l == "op")
        .map(|(_, v)| v)
        .expect("monoid.op field");
    assert!(
        matches!(op, FieldValue::Reference(_)),
        "`op: fn(Int, Int) -> Int` must lower to int_add Reference: {op:?}"
    );
}
