#![allow(clippy::disallowed_macros)]

use std::process::ExitCode;

use v3_compiler::generated_full_bootstrap_dag;
use v3_compiler::pb_method_template_projection_dag_emit::write_method_template_projection_dag;

fn main() -> ExitCode {
    let mut args = std::env::args_os();
    let program = args
        .next()
        .and_then(|arg| arg.into_string().ok())
        .unwrap_or_else(|| "emit_method_template_projection".to_string());
    let Some(out_dir) = args.next() else {
        eprintln!("usage: {program} <out-dir>");
        return ExitCode::from(2);
    };
    if args.next().is_some() {
        eprintln!("usage: {program} <out-dir>");
        return ExitCode::from(2);
    }

    let dag = generated_full_bootstrap_dag();
    match write_method_template_projection_dag(&dag, std::path::Path::new(&out_dir)) {
        Ok(path) => {
            eprintln!("wrote {}", path.display());
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("failed to emit method template projection: {err:?}");
            ExitCode::FAILURE
        }
    }
}
