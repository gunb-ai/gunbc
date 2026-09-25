use im::vector as vec;
use std::rc::Rc;

use v1_compiled::v2_compiler_program_partition as emitted;
use v1_compiled::v2_std_node::{node_synthetic, Behavior, Connective, Edge, EdgeLabel, Node, NodeKind};

// NO SEED ORACLE, ON PURPOSE. This driver used to compare the prose constant
// partition_structural_value_carriers_dissolution_trigger against a hand copy in v1_compiler, which
// said nothing about what partition does. Its expected verdicts are read off the authority instead:
// src/v2/compiler/program_partition.dag partition_user_semantic_types_from_root collects, from a
// module's NAMED declaration edges, those whose target is (or contains, through Conj) a Disj or Conj
// type node, keeps the first per name, and skips value declarations and positional edges.

fn leaf(kind: NodeKind) -> Rc<Node> {
    node_synthetic(Rc::new(kind), Rc::new(vec![]))
}

fn type_node(connective: Connective) -> Rc<Node> {
    leaf(NodeKind::TypeNode {
        connective: Rc::new(connective),
    })
}

fn value_node() -> Rc<Node> {
    leaf(NodeKind::ComputationNode {
        behavior: Behavior::Value,
    })
}

fn edge(label: EdgeLabel, target: Rc<Node>) -> Rc<Edge> {
    Rc::new(Edge {
        label: Rc::new(label),
        target,
    })
}

fn named(name: &str) -> EdgeLabel {
    EdgeLabel::Named {
        name: name.to_string(),
    }
}

fn module_root() -> Rc<Node> {
    node_synthetic(
        Rc::new(NodeKind::TypeNode {
            connective: Rc::new(Connective::Conj),
        }),
        Rc::new(vec![
            edge(named("Shape"), type_node(Connective::Disj)),
            edge(named("value_decl"), value_node()),
            edge(named("Shape"), type_node(Connective::Conj)),
            edge(EdgeLabel::Positional, type_node(Connective::Disj)),
            edge(named("Pair"), type_node(Connective::Conj)),
        ]),
    )
}

// The planted fault expects the value declaration to be collected as well, so the fault run goes
// red only if emitted partition really skips it.
fn main() {
    let inject_fault = std::env::args().any(|a| a == "--inject-fault");
    let expected: Vec<&str> = if inject_fault {
        vec!["Shape", "value_decl", "Pair"]
    } else {
        vec!["Shape", "Pair"]
    };
    let collected = emitted::partition_user_semantic_types_from_root(module_root());
    let names: Vec<String> = collected.iter().map(|t| t.name.clone()).collect();
    let first_shape_is_disj = collected.iter().next().map_or(false, |t| {
        matches!(t.semantic.kind.as_ref(), NodeKind::TypeNode { connective } if matches!(connective.as_ref(), Connective::Disj))
    });
    let ok = names == expected && first_shape_is_disj;
    println!("partition user semantic types inject_fault={inject_fault} names={names:?} first_shape_is_disj={first_shape_is_disj} ok={ok}");
    if ok {
        println!("SELF_HOST_PROGRAM_PARTITION_BEHAVIORAL_RECEIPT: PASS");
        std::process::exit(0);
    }
    println!("SELF_HOST_PROGRAM_PARTITION_BEHAVIORAL_RECEIPT: FAIL");
    std::process::exit(1);
}
