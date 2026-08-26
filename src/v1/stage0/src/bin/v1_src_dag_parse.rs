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

// HOW TO REACH IT WHEN A PARSE REFUSAL IS UNATTRIBUTED, recorded here because it was
// rediscovered the hard way and the rediscovery cost two lanes an evening. The composed
// `--required-ci` phase prints its refusals with no position, and the annotation diagnostics
// carry BYTE OFFSETS rather than line numbers, so reading a CI log tells you the rule fired and
// not where. This binary prints ONE LINE PER REFUSAL NAMING THE FILE, which is enough to bisect.
//
// It does not need a whole-tree local compile (OOM in a session container, swap disabled) and it
// does not need a place in the CI queue. Build and run it in ONE remote dispatch, because the
// runners are amd64 and a binary built there will not execute in an arm64 session:
//
//   ctrl-build --remote -- bash -lc \
//     'cargo build --release -p v1-compiler --bin v1_src_dag_parse && ./target/release/v1_src_dag_parse'
//
// ~4 minutes cold, seconds warm. USE IT WITH A DISCRIMINATING CONTROL: a clean sweep from an
// instrument nobody falsified is worth nothing -- a bin that read no files would report the same
// thing. Append one unattached trailing `//` line to any `.dag`, re-run, and confirm exactly one
// refusal naming exactly that file before believing a green. Then restore the file: a probe that
// leaves its subject mutated turns the next measurement into a lie.

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
        Ok(sweep) => {
            let population =
                v1_compiler::cli_run::declaration_index::index_population(&sweep.index);
            let findings = v1_compiler::cli_run::declaration_index::corpus_findings(&sweep.index);
            eprintln!(
                "v1_src_dag_parse: {} file(s) parse-clean; declarations modules={} \
                 declared={} import_members={} citations={} debt={} in_fixtures={} outside_index={} kernel_named={} \
                 lens_modules={}",
                sweep.parse_clean,
                population.modules,
                population.declarations,
                population.import_members,
                population.citations,
                population.citations_pre_existing_debt,
                population.citations_in_fixtures,
                population.citations_outside_index,
                population.import_members_kernel_named,
                population.lens_modules,
            );
            for finding in &findings {
                eprintln!(
                    "v1_src_dag_parse: {}",
                    v1_compiler::cli_run::declaration_index::render_finding(&cwd, finding)
                );
            }
            if findings.is_empty() {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            }
        }
        Err(errors) => {
            for e in &errors {
                eprintln!("v1_src_dag_parse: {e}");
            }
            ExitCode::from(1)
        }
    }
}
