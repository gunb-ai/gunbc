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
