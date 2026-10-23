//! Throwaway smoking-gun probe (NOT for commit; created on the runner only).
//! Parent decision rule (msg_e0f346d2): "a generic binding needs UNIFIABILITY ... not
//! equality" -- if node_type_equals says DISTINCT on pairs the corpus accepts elsewhere,
//! the earliest unjustified boundary is the FOLD choosing equality; if equality is wrong
//! on truly identical types, that is a separate defect in node_type_equals itself.
//! Each pair runs through two positions on the SAME seed binary, both using DECLARED
//! positions only (no literals, so nothing but the judgment can refuse):
//!   (1) NON-GENERIC: value declared `b` passed at a formal declared `a` -- the corpus's
//!       own compatibility judgment (alias unfold, the declared text route, head/applied).
//!   (2) GENERIC: the same two spellings bound through one type variable T (what the fold
//!       judges with node_type_equals), the exact shape of the shipped reject control.
//! Reading: accepted at (1) + refused at (2) => the fold chose the wrong RELATION.
//! Refused at BOTH => equality itself, or the pair, needs the separate corpus run.
//! The P-vs-Q and Int-vs-Bool controls must refuse at BOTH (a repaired relation keeps
//! refusing them) -- that is the repair's acceptance bar, measured here first.
use crate::helpers::compile_dag;
use v1_compiler::v1_std_core::is_interpreter_blocking_diagnostic;

fn blocking_messages(source: &str) -> Vec<String> {
    let result = compile_dag(source);
    result
        .diagnostics
        .iter()
        .filter(|d| is_interpreter_blocking_diagnostic(d.diagnostic.clone()))
        .map(|d| v1_compiler::v1_std_core::diagnostic_to_message(d.diagnostic.clone()))
        .map(|m| m.chars().take(170).collect::<String>())
        .collect()
}

struct Pair {
    label: &'static str,
    prelude: &'static str,
    a: &'static str,
    b: &'static str,
}

fn non_generic_source(c: &Pair) -> String {
    format!(
        "{}\nfn takes(x: {}) -> Bool {{ true }}\nfn probe(q: {}) -> Bool {{ takes(x: q) }}\n",
        c.prelude, c.a, c.b
    )
}

fn generic_source(c: &Pair) -> String {
    format!(
        "{}\nfn same_arm<T>(a: T, b: T) -> Bool {{ true }}\nfn probe(p: {}, q: {}) -> Bool {{ same_arm(a: p, b: q) }}\n",
        c.prelude, c.a, c.b
    )
}

#[test]
fn sg_probes_print_both_positions() {
    let cases = vec![
        Pair {
            label: "head_vs_applied_FreeMonoid",
            prelude: "module sg1\ntype EdgeLabel\ntype Node\ntype Edge<T>\n",
            a: "FreeMonoid",
            b: "FreeMonoid<Edge<Int, Bool>>",
        },
        Pair {
            label: "alias_QualifiedName_vs_FreeMonoidSymbol",
            prelude: "module sg3\ntype Symbol\ntype QualifiedName = FreeMonoid<Symbol>\n",
            a: "QualifiedName",
            b: "FreeMonoid<Symbol>",
        },
        Pair {
            label: "outcome_head_vs_applied",
            prelude: "module sg4\ntype Outcome<T>\n",
            a: "Outcome",
            b: "Outcome<Int>",
        },
        Pair {
            label: "CONTROL_P_vs_Q",
            prelude: "module sg5\ntype P = P1 { x: Int } | P2 { y: Int }\ntype Q = Q1 { x: Int } | Q2 { y: Int }\n",
            a: "P",
            b: "Q",
        },
        Pair {
            label: "CONTROL_Int_vs_Bool",
            prelude: "module sg6\n",
            a: "Int",
            b: "Bool",
        },
    ];
    for c in cases {
        let ng = blocking_messages(&non_generic_source(&c));
        let g = blocking_messages(&generic_source(&c));
        eprintln!(
            "SG {} | NON-GENERIC({} <- {}) => {} | GENERIC(T: {} vs {}) => {}",
            c.label,
            c.a,
            c.b,
            if ng.is_empty() { "ACCEPTED".to_string() } else { format!("{:?}", ng) },
            c.a,
            c.b,
            if g.is_empty() { "ACCEPTED".to_string() } else { format!("{:?}", g) }
        );
    }
}
