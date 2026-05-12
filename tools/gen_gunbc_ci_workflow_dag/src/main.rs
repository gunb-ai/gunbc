//! Regenerate `dsl/gunbc/ci_github_actions_workflow.dag` from `.github/workflows/ci.yml`.
//!
//! Run from repo root (local cargo, bypass ctrl-build shims):
//!   CTRL_BUILD_WRAP_CARGO=0 cargo run -q -p gen_gunbc_ci_workflow_dag -- .github/workflows/ci.yml \\
//!     > dsl/gunbc/ci_github_actions_workflow.dag

use std::env;
use std::fs;
use std::io::{self, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = env::args()
        .nth(1)
        .ok_or("usage: gen_gunbc_ci_workflow_dag <path-to-ci.yml>")?;
    let raw = fs::read_to_string(&path)?;
    let out = gen_gunbc_ci_workflow_dag::emit_ci_github_actions_workflow_module(&path, &raw)?;
    io::stdout().write_all(out.as_bytes())?;
    Ok(())
}
