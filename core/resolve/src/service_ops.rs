// Transitional extraction for C11: compile the implementation from its current
// source file while `gunbc-dag` consumes it through this crate boundary.
#[path = "../../../gunbc-dag/src/resolve_service.rs"]
mod service_ops_impl;

pub use service_ops_impl::*;
