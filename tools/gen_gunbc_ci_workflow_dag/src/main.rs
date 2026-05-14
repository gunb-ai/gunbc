//! Regenerate `dsl/gunbc/ci_github_actions_workflow.dag` from `.github/workflows/ci.yml`, or
//! rewrite the YAML file to canonical `serde_yaml` bytes (hand-authority dissolution / gate #98).
//!
//! Emit `.dag`:
//!   CTRL_BUILD_WRAP_CARGO=0 cargo run -q -p gen_gunbc_ci_workflow_dag -- .github/workflows/ci.yml \\
//!     > dsl/gunbc/ci_github_actions_workflow.dag
//!
//! Canonicalize committed workflow YAML (then re-emit `.dag` as above):
//!   CTRL_BUILD_WRAP_CARGO=0 cargo run -q -p gen_gunbc_ci_workflow_dag \\
//!     -- --write-canonical-github-actions-yaml .github/workflows/ci.yml

use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

fn usage() -> &'static str {
    "usage:\n  gen_gunbc_ci_workflow_dag <path-to-ci.yml>  (emit `.dag` module to stdout)\n  gen_gunbc_ci_workflow_dag --write-canonical-github-actions-yaml <path-to-ci.yml>  (migrate YAML to deterministic serialization)"
}

fn write_canonical_github_actions_yaml(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let raw = fs::read_to_string(path)?;
    let v = gen_gunbc_ci_workflow_dag::parse_and_validate_github_actions_workflow_yaml(&raw)?;
    let mut canon = gen_gunbc_ci_workflow_dag::canonical_github_actions_yaml_string(&v)?;
    if !canon.ends_with('\n') {
        canon.push('\n');
    }
    fs::write(path, canon)?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    match (args.next().as_deref(), args.next()) {
        (Some("--write-canonical-github-actions-yaml"), Some(path)) => {
            write_canonical_github_actions_yaml(Path::new(&path))
        }
        (Some(flag), _) if flag.starts_with('-') && flag != "-" => Err(usage().into()),
        (Some(path), None) => {
            let raw = fs::read_to_string(path)?;
            let out =
                gen_gunbc_ci_workflow_dag::emit_ci_github_actions_workflow_module(path, &raw)?;
            io::stdout().write_all(out.as_bytes())?;
            Ok(())
        }
        _ => Err(usage().into()),
    }
}
