#!/bin/bash
# SMOKING GUN (parent msg_e0f346d2 decision rule): equality vs unifiability, measured on
# the shipped seed binary. Creates the probe test ON THE RUNNER (throwaway), registers it
# in lib.rs backed up first, runs it, restores lib.rs. No git checkout, no git clean.
set -uo pipefail
export PATH="/opt/cargo/bin:/usr/local/bin:/usr/local/sbin:$PATH"
export RUSTUP_HOME=/opt/rustup
cd .
echo "SG: base=$(git rev-parse HEAD) seedU=$(grep -c UnifyGenericsAcc src/v1/stage0/src/v1_compiler_infer.rs)"
cat > src/v1/tests/src/sg_probe_test.rs <<'SGEOF'
//! Throwaway smoking-gun probe (NOT for commit; created on the runner only).
//! Parent decision rule (msg_e0f346d2): a generic binding needs UNIFIABILITY, not
//! equality. Two positions, same seed binary, DECLARED types only (no literals), and
//! the printed message tells which judgment refused:
//!   (1) NON-GENERIC: value declared `b` passed at a formal declared `a` -- the corpus's
//!       own compatibility judgment (alias unfold, the declared text route, head/applied).
//!   (2) GENERIC: two formals declared `a` and `b` forwarded into one type variable T
//!       through the fold (node_type_equals), the exact shape of the reject control.
//! Reading (per pair, from the printed messages):
//!   accepted at (1) + refused-at-(2)-by-"bound as"  => fold chose the WRONG RELATION
//!   refused at BOTH by the same non-fold diagnostic => equality/pair defect, separate run
//!   CONTROL pairs must show "bound as" at (2) AND a refusal at (1) (P<-Q is genuinely
//!   not assignable) -- a repaired relation must keep refusing them at (2).
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
            prelude: "module sg1\ntype Edge<T>\n",
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
        Pair {
            label: "string_vs_freemonoid_char",
            prelude: "module sg7\ntype Char\n",
            a: "String",
            b: "FreeMonoid<Char>",
        },
        Pair {
            // same head, DIFFERENT arguments: must refuse at the generic position (the
            // reject-control class -- Box<Int> vs Box<Bool>); a repaired relation that
            // accepts this has become unsound, not merely looser.
            label: "CONTROL_FreeMonoid_Int_vs_Bool",
            prelude: "module sg8\n",
            a: "FreeMonoid<Int>",
            b: "FreeMonoid<Bool>",
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

SGEOF
cp src/v1/tests/src/lib.rs /tmp/lib_backup_sg.rs
echo "mod sg_probe_test;" >> src/v1/tests/src/lib.rs
cargo test -p v1-compiler-tests --lib sg_probe -- --nocapture 2>&1 | grep -E "^SG |test result|^error" 
cp /tmp/lib_backup_sg.rs src/v1/tests/src/lib.rs
rm -f src/v1/tests/src/sg_probe_test.rs
cmp -s src/v1/tests/src/lib.rs /tmp/lib_backup_sg.rs && echo "SG: lib.rs restored"
echo "SG_DONE"
