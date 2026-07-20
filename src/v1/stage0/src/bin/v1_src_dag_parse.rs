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
    let read_dir = match std::fs::read_dir(&v1_dir) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("v1_src_dag_parse: read_dir {}: {e}", v1_dir.display());
            return ExitCode::from(1);
        }
    };

    let mut entries: Vec<_> = read_dir.flatten().collect();
    entries.sort_by_key(|e| e.file_name());

    let dag_paths: Vec<_> = entries
        .into_iter()
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "dag")
                .unwrap_or(false)
        })
        .map(|e| e.path())
        .collect();

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
