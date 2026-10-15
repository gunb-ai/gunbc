#![allow(clippy::disallowed_macros)]
use std::rc::Rc;
use v1_compiler::v1_compiler_compile::{compile_to_resolved, SourceFile};
use v1_compiler::v1_std_core::{InferredNode, Node};

fn dump(n: &Node, depth: usize, ind: usize, tag: &str) {
    let pad = " ".repeat(ind * 2);
    let inf = match &n.inferred {
        None => "none".to_string(),
        Some(i) => match &**i {
            InferredNode::Resolved { node } => format!("Resolved({})", node.name),
            InferredNode::TypeVariable { .. } => "TypeVariable".to_string(),
            _ => "other".to_string(),
        },
    };
    println!(
        "{pad}{tag} name={:?} conn={:?} nch={} nparams={} inferred={} decl={}",
        n.name,
        n.connective,
        n.children.len(),
        n.params.len(),
        inf,
        n.declaration.is_some()
    );
    if depth == 0 {
        return;
    }
    for c in n.children.iter() {
        dump(c, depth - 1, ind + 1, "child");
    }
    for p in n.params.iter() {
        dump(p, depth - 1, ind + 1, "param");
    }
    if let Some(i) = &n.inferred {
        if let InferredNode::Resolved { node } = &**i {
            dump(node, depth - 1, ind + 1, "RESOLVED");
        }
    }
    if let Some(b) = &n.body {
        dump(b, depth - 1, ind + 1, "body");
    }
}

fn main() {
    let path = std::env::args().nth(1).expect("path");
    let content = std::fs::read_to_string(&path).unwrap();
    let r = compile_to_resolved(Rc::new(vec![Rc::new(SourceFile { path, content })].into()));
    println!("diagnostics={}", r.diagnostics.len());
    let g = r.graph.clone().expect("graph");
    for m in g.modules.iter() {
        for it in m.items.iter() {
            if it.name == "first_of" || it.name == "build_idx" {
                println!("== ITEM {}", it.name);
                dump(it, 7, 0, "item");
            }
        }
    }
}
