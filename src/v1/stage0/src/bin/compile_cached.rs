//! Hand-written CI transport: whole-tree compile with resolved_graph_cache kernel.
//! Authority: dsl/tools/dsl_compile_clean_memo.dag (NOT a parallel cache).

use std::path::PathBuf;
use std::process::ExitCode;
use std::rc::Rc;

use clap::Parser;
use v1_compiler::cli_run;
use v1_compiler::v1_compiler_artifact::RenderTarget;
use v1_compiler::v1_compiler_compile::PipelineResult;
use v1_compiler::v1_std_core::{diagnostic_to_message, diagnostic_to_span, is_error_diagnostic};

#[derive(Parser)]
#[command(name = "compile_cached")]
struct Args {
    #[arg(long)]
    source_root: PathBuf,
    #[arg(long, default_value = "target/compile_cached_out")]
    output_dir: PathBuf,
    #[arg(long, default_value = "rust")]
    target: String,
}

fn hard_errors(result: &PipelineResult) -> bool {
    result
        .diagnostics
        .iter()
        .any(|d| is_error_diagnostic(d.diagnostic.clone()))
}

fn main() -> ExitCode {
    let args = Args::parse();
    let render_target = match args.target.as_str() {
        "rust" => RenderTarget::Rust,
        "dag" => RenderTarget::Dag,
        "go" => RenderTarget::Go,
        "python" => RenderTarget::Python,
        other => {
            eprintln!("error: unsupported --target {other}");
            return ExitCode::from(1);
        }
    };

    let sources = Rc::new(cli_run::load_compile_sources(&[
        args.source_root.to_string_lossy().to_string(),
    ]));
    eprintln!(
        "compile_cached: {} sources from {:?} (target: {})",
        sources.len(),
        args.source_root,
        args.target
    );

    let result = cli_run::compile_sources_with_resolved_graph_cache(sources, render_target);
    std::fs::create_dir_all(&args.output_dir).unwrap_or_else(|e| {
        eprintln!("error: failed to create output dir {:?}: {}", args.output_dir, e);
        std::process::exit(1);
    });
    for file in result.files.iter() {
        let path = args.output_dir.join(file.path.as_str());
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap_or_else(|e| {
                eprintln!("error: failed to create {:?}: {}", parent, e);
                std::process::exit(1);
            });
        }
        std::fs::write(&path, file.content.as_str()).unwrap_or_else(|e| {
            eprintln!("error: failed to write {:?}: {}", path, e);
            std::process::exit(1);
        });
    }
    eprintln!(
        "compiled: {} files emitted, {} diagnostics",
        result.files.len(),
        result.diagnostics.len()
    );
    for d in result.diagnostics.iter() {
        if is_error_diagnostic(d.diagnostic.clone()) {
            let span = diagnostic_to_span(d.diagnostic.clone());
            eprintln!(
                "{}: error: {}",
                span.file,
                diagnostic_to_message(d.diagnostic.clone())
            );
        }
    }
    if hard_errors(&result) {
        return ExitCode::from(1);
    }
    if result.files.is_empty() {
        eprintln!("error: no files emitted");
        return ExitCode::from(1);
    }
    ExitCode::SUCCESS
}
