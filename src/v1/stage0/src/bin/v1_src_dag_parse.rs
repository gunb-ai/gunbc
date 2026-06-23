#![allow(clippy::disallowed_macros)]

#[allow(dead_code)]
const SCAFFOLD_NOTE: &str = "SCAFFOLD \u{2014} dissolve-on: when src/v1 .dag is parsed by the \
    modeled pipeline / when the seed shrinks to zero (\u{a7}7; src/v1 is the bootstrap seed; \
    parsing the seed currently requires the seed parser itself).";

use std::collections::HashMap;
use std::process::ExitCode;
use std::rc::Rc;

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

    let mut count = 0usize;

    for entry in &entries {
        let path = entry.path();
        if !path.extension().map(|e| e == "dag").unwrap_or(false) {
            continue;
        }

        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("v1_src_dag_parse: read {}: {e}", path.display());
                return ExitCode::from(1);
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
            eprintln!(
                "v1_src_dag_parse: parse error in {}: {}",
                entry.file_name().to_string_lossy(),
                v1_compiler::v1_std_core::diagnostic_to_message(err.diagnostic.clone())
            );
            return ExitCode::from(1);
        }

        count += 1;
    }

    if count == 0 {
        eprintln!("v1_src_dag_parse: no .dag files found in src/v1/ — check run directory");
        return ExitCode::from(1);
    }

    eprintln!("v1_src_dag_parse: {count} file(s) parse-clean");
    ExitCode::SUCCESS
}
