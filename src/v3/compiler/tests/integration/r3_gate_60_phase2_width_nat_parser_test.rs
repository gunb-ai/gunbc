//! R3 gate #60 Phase 2.1 — parser + lower receipt for `Algebra<N>` surface sugar
//! (`Int<64>` → `Compose<Int, MachineWidth<64>>` with literal-Nat phantom slot).
//!
//! Full gate #60 closure still requires follow-on slices Z/D/E/F per `docs/audit/r3-gate-60-decomposition.md`.

use v3_compiler::compile_to_dag;
use v3_compiler::dag::{AtomPayload, Dag, DeclarationId, TypeConnective};
use v3_compiler::parse_surface;
use v3_compiler::{parse_for_test, tokenize_for_test};

use crate::common::substrate_receipts::bind_value_type_decl;

const FILE: &str = "r3_gate_60_phase2_width_nat.v3";

fn assert_compose_algebra_machine_width_literal(
    dag: &Dag,
    bind_name: &str,
    algebra_name: &str,
    expected_width_decimal: &str,
) {
    let ty = bind_value_type_decl(dag, bind_name);
    let decl = dag.declaration(ty);
    let TypeConnective::Instantiation { template, arguments } = &decl.connective else {
        panic!("expected root Instantiation for `{bind_name}`, got {:?}", decl.connective);
    };
    let compose_id = dag
        .declaration_by_name("Compose")
        .unwrap_or_else(|| panic!("Compose"))
        .id;
    assert_eq!(*template, compose_id, "`{bind_name}` root template");
    assert_eq!(arguments.len(), 2, "`{bind_name}` Compose arity");

    let algebra_id = dag
        .declaration_by_name(algebra_name)
        .unwrap_or_else(|| panic!("{algebra_name}"))
        .id;
    assert_eq!(
        arguments[0].value, algebra_id,
        "`{bind_name}` slot-1 algebra"
    );

    let mw_decl = dag.declaration(arguments[1].value);
    let TypeConnective::Instantiation {
        template: mw_template,
        arguments: mw_args,
    } = &mw_decl.connective
    else {
        panic!(
            "expected MachineWidth instantiation for `{bind_name}`, got {:?}",
            mw_decl.connective
        );
    };
    let machine_width_id = dag
        .declaration_by_name("MachineWidth")
        .unwrap_or_else(|| panic!("MachineWidth"))
        .id;
    assert_eq!(*mw_template, machine_width_id);
    assert_eq!(mw_args.len(), 1);
    let inner = dag.declaration(mw_args[0].value);
    match &inner.connective {
        TypeConnective::Atom(AtomPayload::Literal(LiteralBits::Int(s))) => {
            assert_eq!(s, expected_width_decimal);
        }
        other => panic!("expected literal Nat width, got {other:?}"),
    }
}

#[test]
fn gate_60_phase2_parse_accepts_algebra_angle_width_nat() {
    let source = "\
import std.integer { Int, UInt }
import std.float { Real }
import std.nat { Nat }

let width_int: Int<64> = 0
let width_uint: UInt<32> = 0
let width_real: Real<64> = 0
let width_nat: Nat<8> = 0
";
    let tokens = tokenize_for_test(source, FILE).expect("tokenize");
    let parsed = parse_for_test(&tokens, FILE).expect("parse");
    let m: &parse_surface::SurfaceModule = &parsed;

    fn assert_param_width(
        items: &[parse_surface::SurfaceItem],
        name: &str,
        surface: &str,
        width: &str,
    ) {
        let item = items
            .iter()
            .find_map(|it| match it {
                parse_surface::SurfaceItem::Let {
                    name: n,
                    type_ann,
                    ..
                } if n == name => Some(type_ann.as_ref().expect("type ann")),
                _ => None,
            })
            .unwrap_or_else(|| panic!("let `{name}`"));
        let parse_surface::SurfaceType::Parameterized {
            name: tycon,
            args,
            ..
        } = item
        else {
            panic!("expected Parameterized type for `{name}`");
        };
        assert_eq!(tycon, surface);
        assert_eq!(args.len(), 1);
        match &args[0] {
            parse_surface::SurfaceType::WidthNatLiteral { decimal, .. } => {
                assert_eq!(decimal, width);
            }
            other => panic!("expected WidthNatLiteral for `{name}`, got {other:?}"),
        }
    }

    assert_param_width(&m.items, "width_int", "Int", "64");
    assert_param_width(&m.items, "width_uint", "UInt", "32");
    assert_param_width(&m.items, "width_real", "Real", "64");
    assert_param_width(&m.items, "width_nat", "Nat", "8");
}

#[test]
fn gate_60_phase2_int_64_lowers_to_compose_int_machine_width_literal() {
    let source = "\
import std.integer { Int }

let probe: Int<64> = 0
";
    let dag = compile_to_dag(source, FILE).expect("compile");
    assert_compose_algebra_machine_width_literal(&dag, "probe", "Int", "64");
}

#[test]
fn gate_60_phase2_nat_8_lowers_via_uint_slot() {
    let source = "\
import std.nat { Nat }

let probe: Nat<8> = 0
";
    let dag = compile_to_dag(source, FILE).expect("compile");
    assert_compose_algebra_machine_width_literal(&dag, "probe", "UInt", "8");
}

#[test]
fn gate_60_integer_routing_witness_accepts_literal_nat_machine_width() {
    let source = "\
import std.integer { Int }

let probe: Int<64> = 0
";
    let dag = compile_to_dag(source, FILE).expect("compile");
    let ty = bind_value_type_decl(&dag, "probe");
    assert!(
        v3_compiler::integer_literal_routing_witness(&dag, ty).is_some(),
        "expected integer literal routing witness for lowered Int<64>"
    );
}
