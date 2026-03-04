//! Makegen execution mock helpers.
//!
//! These provide `BoundaryMocks` for various execution modes of makegen DAGs.

use gunbc_exec::BoundaryMocks;
use gunbc_ir::transport::{FileResponse, TransportResponse};
use gunbc_ir::Value;

/// Input mocks for the makegen entrypoint.
///
/// Injects the output path and pre-computed Makefile content.  DSL fn bodies
/// don't evaluate at runtime (DeclaredOutputCallableOp passthrough), so we
/// pre-compute the content via direct fn body evaluation and inject it as an
/// output mock — mirroring the generated binary's embedded asset approach.
pub fn makegen_entrypoint_mocks(output_path: &str) -> Result<BoundaryMocks, String> {
    let mut input_mocks = BoundaryMocks::new();
    input_mocks.set_input(
        "tools.makegen::makegen",
        "path",
        Value::Str(output_path.to_string()),
    );
    input_mocks.set_input(
        "param_source_tools_makegen_makegen_path",
        "path",
        Value::Str(output_path.to_string()),
    );

    // Pre-compute Makefile content and inject as output mock for
    // render_makefile_content.  Without this, the passthrough callable
    // forwards the literal DSL template string ("{header}{body}") instead
    // of the evaluated Makefile.
    let makefile_content = gunbc_dag::compute_makegen_content()
        .map_err(|e| format!("failed to compute makegen content for mocks: {e}"))?;
    input_mocks.set_value(
        "tools.makegen::render_makefile_content",
        "return",
        Value::Str(makefile_content),
    );

    Ok(input_mocks)
}

/// Dry-run mocks: intercept transport boundary nodes so no I/O occurs.
pub fn makegen_dry_run_transport_mocks(output_path: &str) -> BoundaryMocks {
    let mut dry_run_mocks = BoundaryMocks::new();
    dry_run_mocks.set_value(
        "fs_env",
        "FilesystemHandle",
        Value::Str("filesystem://dry-run".to_string()),
    );
    dry_run_mocks.set_value(
        "execute_read_makegen",
        "response",
        Value::Response(TransportResponse::File(FileResponse::read_ok(
            output_path.to_string(),
            "<dry-run>",
        ))),
    );
    dry_run_mocks.set_value("execute_makegen_transport", "response", Value::Skipped);
    dry_run_mocks
}

/// Check-mode mocks: read existing content and intercept the write transport.
pub fn makegen_check_mode_transport_mocks(output_path: &str) -> Result<BoundaryMocks, String> {
    let mut check_mode_mocks = BoundaryMocks::new();
    check_mode_mocks.set_value(
        "fs_env",
        "FilesystemHandle",
        Value::Str("filesystem://check-mode".to_string()),
    );
    let existing_content = read_existing_content(output_path)?;
    check_mode_mocks.set_value(
        "execute_read_makegen",
        "response",
        Value::Response(TransportResponse::File(FileResponse::read_ok(
            output_path.to_string(),
            existing_content,
        ))),
    );
    check_mode_mocks.set_value("execute_makegen_transport", "response", Value::Skipped);
    Ok(check_mode_mocks)
}

/// Read existing file content.
///
/// Returns empty string when the file does not yet exist, and errors on other
/// filesystem failures.
#[allow(clippy::disallowed_methods)]
fn read_existing_content(output_path: &str) -> Result<String, String> {
    match std::fs::read_to_string(output_path) {
        Ok(content) => Ok(content),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
        Err(error) => Err(format!(
            "failed to read existing makegen output at {}: {}",
            output_path, error
        )),
    }
}
