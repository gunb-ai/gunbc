#![allow(clippy::disallowed_macros)]

#[allow(dead_code)]
const SCAFFOLD_NOTE: &str = "SCAFFOLD \u{2014} dissolve-on: when src/v1 .dag is parsed by the \
    modeled pipeline / when the seed shrinks to zero (\u{a7}7; src/v1 is the bootstrap seed; \
    parsing the seed currently requires the seed parser itself).";

// A THIN CALLER, NOT THE IMPLEMENTATION. The walk lives in `cli_run::run_v1_src_dag_parse` so
// that the composed `claim_executor --required-ci` run can hold it as one phase beside regen and
// the floor, in one process, instead of the CI job holding it as one more YAML step whose order
// and precondition are facts about a workflow file.
//
// This binary is kept because running the parse sweep ALONE is a real local action — it is the
// cheapest check in the tree and the one worth reaching for while editing any `.dag`. It is not
// a second authority: it calls the same function the composed run calls, over the same shared
// root roster (`cli_run::DAG_PARSE_SWEEP_ROOTS`), so there is one walk and one subject.
//
// THE NAME IS NOW NARROWER THAN THE SUBJECT, and that is declared rather than quietly lived
// with: the sweep covers `src/v1`, `dag` and `src/v2`, so `v1_src_dag_parse` names a third of
// what it does. It is NOT renamed here because the name is a member of the release-bins pack
// roster (`gunbc.ci_release_bins` `witness_declared_release_bins`), from which
// `.github/workflows/fleet-converge.yml` is GENERATED — renaming it is a regeneration, not an
// edit, and bundling one into a coverage fix is how a generated artifact drifts from its
// authority. RENAME TRIGGER: the next change that regenerates that workflow for its own
// reasons carries the rename with it.

use std::process::ExitCode;

fn main() -> ExitCode {
    let cwd = match std::env::current_dir() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("v1_src_dag_parse: current_dir: {e}");
            return ExitCode::from(1);
        }
    };
    match v1_compiler::cli_run::run_dag_parse_sweep(
        &cwd,
        &v1_compiler::cli_run::DAG_PARSE_SWEEP_ROOTS,
    ) {
        Ok(count) => {
            eprintln!("v1_src_dag_parse: {count} file(s) parse-clean");
            ExitCode::SUCCESS
        }
        Err(errors) => {
            for e in &errors {
                eprintln!("v1_src_dag_parse: {e}");
            }
            ExitCode::from(1)
        }
    }
}
