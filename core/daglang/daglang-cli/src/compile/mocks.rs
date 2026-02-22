//! Makegen execution mock helpers.
//!
//! These provide `BoundaryMocks` for various execution modes of makegen DAGs.

use gunbc_exec::BoundaryMocks;
use gunbc_ir::transport::{FileResponse, TransportResponse};
use gunbc_ir::Value;

/// Input mocks for the makegen entrypoint.
pub fn makegen_entrypoint_mocks(output_path: &str) -> BoundaryMocks {
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
    input_mocks
}

/// Dry-run mocks: intercept transport boundary nodes so no I/O occurs.
pub fn makegen_dry_run_transport_mocks(output_path: &str) -> BoundaryMocks {
    let mut dry_run_mocks = BoundaryMocks::new();
    dry_run_mocks.set_value(
        "fs_env",
        "file:write",
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
pub fn makegen_check_mode_transport_mocks(output_path: &str) -> BoundaryMocks {
    let mut check_mode_mocks = BoundaryMocks::new();
    check_mode_mocks.set_value(
        "fs_env",
        "file:write",
        Value::Str("filesystem://check-mode".to_string()),
    );
    let existing_content = read_existing_content(output_path);
    check_mode_mocks.set_value(
        "execute_read_makegen",
        "response",
        Value::Response(TransportResponse::File(FileResponse::read_ok(
            output_path.to_string(),
            existing_content,
        ))),
    );
    check_mode_mocks.set_value("execute_makegen_transport", "response", Value::Skipped);
    check_mode_mocks
}

/// Read existing file content, returning empty string on any error.
#[allow(clippy::disallowed_methods)]
fn read_existing_content(output_path: &str) -> String {
    std::fs::read_to_string(output_path).unwrap_or_default()
}
