#!/bin/bash
# PMM-PROBE: print the EXACT blocking messages for the three reject fixtures so the
# dag census subjects are MEASURED, not guessed. Probe file created ON THE RUNNER
# only (throwaway). lib.rs registration backed up and restored (runner HEAD is main
# tip, so never use git checkout there).
set -uo pipefail
export PATH="/opt/cargo/bin:/usr/local/bin:/usr/local/sbin:$PATH"
export RUSTUP_HOME=/opt/rustup
cd .
echo "PMM2: HEAD(runner)=$(git rev-parse HEAD)"
grep -c cross_coproduct src/v1/tests/src/lib.rs | sed 's/^/PMM2: lib.rs cross_coproduct mods=/'
cat > src/v1/tests/src/pmm_probe_test.rs <<'PMMEOF'
//! Throwaway probe (NOT for commit): prints the blocking diagnostic messages for the
//! three reject fixtures, so the dag census subjects are MEASURED, not guessed.
//! The census row key is (class, subject_name, blocking) and TypeMismatch's subject
//! is the diagnostic's `got` string (src/v1/stage0/src/cli_run/compile_clean.rs:1391);
//! the fold's got is concat(node_type_shape(actual), " (later argument)")
//! (src/v1/04_infer.dag:14458-14459).
use crate::helpers::compile_dag;
use v1_compiler::v1_std_core::is_interpreter_blocking_diagnostic;

fn blocking_messages(source: &str) -> Vec<String> {
    let result = compile_dag(source);
    result
        .diagnostics
        .iter()
        .filter(|d| is_interpreter_blocking_diagnostic(d.diagnostic.clone()))
        .map(|d| v1_compiler::v1_std_core::diagnostic_to_message(d.diagnostic.clone()))
        .collect()
}

#[test]
fn pmm_probe_print_the_three_fixtures() {
    let cases: Vec<(&str, String)> = vec![
        (
            "PQ",
            format!(
                "module locate.cross\n\ntype P = P1 {{ x: Int }} | P2 {{ y: Int }}\ntype Q = Q1 {{ x: Int }} | Q2 {{ y: Int }}\n\nfn same_arm<T>(a: T, b: T) -> Bool {{ true }}\nfn probe() -> Bool {{ same_arm(a: P, b: Q) }}\n"
            ),
        ),
        (
            "INTBOOL",
            "module locate.ib\nfn same_arm<T>(a: T, b: T) -> Bool { true }\nfn probe() -> Bool { same_arm(a: 1, b: true) }\n".to_string(),
        ),
        (
            "BOXARGS",
            "module locate.arg\ntype Box<T>\nfn same_arm<A>(a: A, b: A) -> Bool { true }\nfn probe(x: Box<Int>, y: Box<Bool>) -> Bool { same_arm(a: x, b: y) }\n".to_string(),
        ),
    ];
    for (label, src) in cases {
        let msgs = blocking_messages(&src);
        eprintln!("PMMPROBE {label} => {msgs:?}");
    }
}

PMMEOF
cp src/v1/tests/src/lib.rs /tmp/lib_backup_14152.rs
echo "mod pmm_probe_test;" >> src/v1/tests/src/lib.rs
cargo test -p v1-compiler-tests --lib pmm_probe -- --nocapture 2>&1 | grep -E "PMMPROBE|test result|^error" 
cp /tmp/lib_backup_14152.rs src/v1/tests/src/lib.rs
rm -f src/v1/tests/src/pmm_probe_test.rs
cmp -s src/v1/tests/src/lib.rs /tmp/lib_backup_14152.rs && echo "PMM2: lib.rs restored"
echo "PMM2_DONE"
