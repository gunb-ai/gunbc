#![allow(clippy::disallowed_macros)]

#[allow(dead_code)]
const SCAFFOLD_NOTE: &str = "SCAFFOLD \u{2014} dissolve-on: when src/v1 .dag is parsed by the \
    modeled pipeline / when the seed shrinks to zero (\u{a7}7; src/v1 is the bootstrap seed; \
    parsing the seed currently requires the seed parser itself).";

use im::HashMap;
use std::process::ExitCode;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

fn main() -> ExitCode {
    let cwd = match std::env::current_dir() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("v1_src_dag_parse: current_dir: {e}");
            return ExitCode::from(1);
        }
    };

    let v1_dir = cwd.join("src/v1");
    // RECURSIVE, and the non-recursive predecessor is why this file changed at all.
    // `read_dir(src/v1)` sees the 46 top-level modules and nothing below them, so three
    // authored files were outside the only instrument that can see a v1 parse error:
    // gunbc/occurrence_binding_parser_walk.dag, gunbc/namespace_reference_derived_closure_
    // production_observations.dag, and tests/claim/checkpoint_identity_keying_witness_test.dag
    // -- the last holding every `test fn` src/v1 declares. The subtree most likely to carry
    // a defect was the subtree the check could not reach, and nothing reported that: a walk
    // that finds no files in a directory it never opened is indistinguishable from a clean one.
    let mut dag_paths: Vec<std::path::PathBuf> = Vec::new();
    let mut stack: Vec<std::path::PathBuf> = vec![v1_dir.clone()];
    while let Some(dir) = stack.pop() {
        let read_dir = match std::fs::read_dir(&dir) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("v1_src_dag_parse: read_dir {}: {e}", dir.display());
                return ExitCode::from(1);
            }
        };
        for entry in read_dir.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // `target/` under a nested Cargo.toml is build output, never authored source.
                if path.file_name().map(|n| n == "target").unwrap_or(false) {
                    continue;
                }
                // `tests/fixtures/` holds deliberately partial or malformed inputs
                // authored FOR the parser's own tests -- a fixture that fails to parse
                // is the fixture doing its job, not a defect. The existing corpus
                // discovery in `compiler_tests` excludes them on the same grounds.
                if path.file_name().map(|n| n == "fixtures").unwrap_or(false)
                    && dir.file_name().map(|n| n == "tests").unwrap_or(false)
                {
                    continue;
                }
                stack.push(path);
            } else if path.extension().map(|ext| ext == "dag").unwrap_or(false) {
                dag_paths.push(path);
            }
        }
    }
    dag_paths.sort();

    if dag_paths.is_empty() {
        eprintln!("v1_src_dag_parse: no .dag files found in src/v1/ — check run directory");
        return ExitCode::from(1);
    }

    let errors: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let count = Arc::new(AtomicUsize::new(0));

    // Each file gets its own Rc<HashMap> — no shared parse state — so parsing is
    // embarrassingly parallel. Thread panics propagate via scope and exit non-zero (fail-closed).
    std::thread::scope(|s| {
        for path in &dag_paths {
            let errors = Arc::clone(&errors);
            let count = Arc::clone(&count);
            let path = path.clone();
            s.spawn(move || {
                let content = match std::fs::read_to_string(&path) {
                    Ok(c) => c,
                    Err(e) => {
                        errors
                            .lock()
                            .unwrap()
                            .push(format!("read {}: {e}", path.display()));
                        return;
                    }
                };
                let result = v1_compiler::v1_compiler_parse::parse(
                    v1_compiler::v1_compiler_tokenize::tokenize(
                        content,
                        path.to_string_lossy().to_string(),
                    ),
                    Rc::new(HashMap::new()),
                );
                if let Some(ref err) = result.error {
                    errors.lock().unwrap().push(format!(
                        "parse error in {}: {}",
                        path.file_name().unwrap_or_default().to_string_lossy(),
                        v1_compiler::v1_std_core::diagnostic_to_message(err.diagnostic.clone(),),
                    ));
                } else {
                    count.fetch_add(1, Ordering::Relaxed);
                }
            });
        }
    });

    let errors = Arc::try_unwrap(errors).unwrap().into_inner().unwrap();
    if !errors.is_empty() {
        for e in &errors {
            eprintln!("v1_src_dag_parse: {e}");
        }
        return ExitCode::from(1);
    }

    let count = Arc::try_unwrap(count).unwrap().into_inner();
    eprintln!("v1_src_dag_parse: {count} file(s) parse-clean");
    ExitCode::SUCCESS
}
