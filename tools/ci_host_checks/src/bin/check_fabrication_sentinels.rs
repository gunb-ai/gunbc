#![allow(clippy::disallowed_macros)]

use std::process::ExitCode;

use ci_host_checks::{check_fabrication_sentinels, repo_root_from_manifest_dir};

fn main() -> ExitCode {
    let root = repo_root_from_manifest_dir(std::path::Path::new(env!("CARGO_MANIFEST_DIR")));
    match check_fabrication_sentinels(&root) {
        Ok(()) => {
            eprintln!("check-fabrication-sentinels: ok");
            ExitCode::SUCCESS
        }
        Err(violations) => {
            for line in &violations {
                eprintln!("{line}");
            }
            eprintln!(
                "check-fabrication-sentinels: failed ({} file(s))",
                violations.len()
            );
            ExitCode::from(1)
        }
    }
}
