#![allow(clippy::disallowed_macros)]

use std::process::ExitCode;

use v1_compiler::cli_run::reference_derived_closure_over_source_roots;

// ONLY A SETTLED CLOSURE IS A SUCCESSFUL RUN. An earlier version of this bin printed the refusal and
// exited zero, which makes it unusable as a gate and reads as the ordinary path having ACCEPTED the
// closure. Refusal and exhaustion are failures of the run; a bad invocation is distinct from both.
fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    let mut source_roots: Vec<String> = Vec::new();
    let mut entry: Option<String> = None;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--source-root" => {
                i += 1;
                match args.get(i) {
                    Some(v) => source_roots.push(v.clone()),
                    None => {
                        eprintln!(
                            "reference_derived_closure_probe: --source-root requires a value"
                        );
                        return ExitCode::from(2);
                    }
                }
            }
            "--entry" => {
                i += 1;
                match args.get(i) {
                    Some(v) => entry = Some(v.clone()),
                    None => {
                        eprintln!("reference_derived_closure_probe: --entry requires a value");
                        return ExitCode::from(2);
                    }
                }
            }
            other => {
                eprintln!(
                    "reference_derived_closure_probe: unknown argument: {}",
                    other
                );
                return ExitCode::from(2);
            }
        }
        i += 1;
    }
    let entry = match entry {
        Some(e) => e,
        None => {
            eprintln!("reference_derived_closure_probe: --entry is required");
            return ExitCode::from(2);
        }
    };
    if source_roots.is_empty() {
        eprintln!("reference_derived_closure_probe: provide at least one --source-root");
        return ExitCode::from(2);
    }
    let run = reference_derived_closure_over_source_roots(&entry, &source_roots);
    if run.is_settled() {
        println!("{}", run.report());
        ExitCode::SUCCESS
    } else {
        eprintln!("{}", run.report());
        ExitCode::from(1)
    }
}
