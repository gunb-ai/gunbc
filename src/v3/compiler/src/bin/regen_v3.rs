use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process::ExitCode;

use v3_compiler::{
    compare_stage_snapshots, compile_stage_snapshots, default_fixed_point_source,
    FixedPointMismatch, StageSnapshotError,
};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            let _ = writeln!(io::stderr(), "{message}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let mut source_path: Option<String> = None;
    let mut file_name: Option<String> = None;
    let mut inject_stage: Option<String> = env::var("GUNBC_REGEN_INJECT_STAGE").ok();

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--source" => {
                source_path = Some(
                    args.next()
                        .ok_or_else(|| "--source requires a path".to_string())?,
                );
            }
            "--file" => {
                file_name = Some(
                    args.next()
                        .ok_or_else(|| "--file requires a name".to_string())?,
                );
            }
            "--inject-stage" => {
                inject_stage = Some(
                    args.next()
                        .ok_or_else(|| "--inject-stage requires a stage name".to_string())?,
                );
            }
            other => return Err(format!("unknown argument `{other}`")),
        }
    }

    let (source, file) = match source_path {
        Some(path) => {
            let source = fs::read_to_string(&path)
                .map_err(|err| format!("failed to read {}: {err}", path))?;
            let file = file_name.unwrap_or_else(|| {
                Path::new(&path)
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("input.v3")
                    .to_string()
            });
            (source, file)
        }
        None => (
            default_fixed_point_source().to_string(),
            file_name.unwrap_or_else(|| "fixed_point_input.v3".to_string()),
        ),
    };

    let pass1 = compile_stage_snapshots(&source, &file).map_err(render_snapshot_error)?;
    let mut pass2 = compile_stage_snapshots(&source, &file).map_err(render_snapshot_error)?;

    if let Some(stage_name) = inject_stage {
        let snapshot = pass2
            .iter_mut()
            .find(|snapshot| snapshot.stage == stage_name)
            .ok_or_else(|| format!("unknown stage `{stage_name}`"))?;
        snapshot
            .bytes
            .extend_from_slice(b"\n# synthetic divergence\n");
        snapshot.dag = None;
    }

    compare_stage_snapshots(&pass1, &pass2).map_err(render_mismatch)?;
    println!("fixed-point verified across {} stages", pass1.len());
    Ok(())
}

fn render_snapshot_error(error: StageSnapshotError) -> String {
    match error {
        StageSnapshotError::Compile(error) => format!("compile failed: {error:?}"),
        StageSnapshotError::Emit(error) => format!("emit failed: {error:?}"),
        StageSnapshotError::Pipeline(error) => format!("pipeline authority failed: {error}"),
    }
}

fn render_mismatch(mismatch: FixedPointMismatch) -> String {
    format!(
        "FIXED-POINT FAILURE at stage `{}`: {}",
        mismatch.stage, mismatch.detail
    )
}
