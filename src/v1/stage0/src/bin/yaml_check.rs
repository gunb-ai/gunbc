//! Parse-check a YAML file (exit 0 = valid, 1 = invalid/missing).
//!
//! Hand-written CI seed bin — fallback transport for `gunbc.ci_yaml_validate` when
//! PyYAML and actionlint are not on PATH. The parse *requirement* is modeled in
//! `dsl/gunbc/ci_yaml_validate.dag`; this bin is one of three shell handlers.
//!
//! SCAFFOLD — dissolves when self-hosted runners guarantee actionlint or PyYAML (drop
//! this fallback from `ci_yml_parse_script`), or when parse moves to a total extdeps
//! host effect without a v1-compiler bin (DESIGN §7 seed → zero).

#![allow(clippy::disallowed_macros)]

use std::{env, fs, process::ExitCode};

fn main() -> ExitCode {
    let Some(path) = env::args().nth(1) else {
        eprintln!("usage: yaml_check <path>");
        return ExitCode::from(2);
    };
    let text = match fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("yaml_check: read {path}: {e}");
            return ExitCode::from(1);
        }
    };
    match serde_yaml::from_str::<serde_yaml::Value>(&text) {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("yaml_check: parse {path}: {e}");
            ExitCode::from(1)
        }
    }
}
