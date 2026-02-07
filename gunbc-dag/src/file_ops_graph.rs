//! Generic graph op wrapper for file-based DAGs.
//!
//! This unifies the common "file ops" graph shape used by makegen, pragma,
//! bootstrap, and testgen. The domain op type is provided by the caller.

use gunbc_exec::{ExecError, Executable};
use gunbc_ir::Value;
use gunbc_lib_blob::BlobOps;
use gunbc_lib_transport::TransportOps;
use gunbc_primitives::{FsEnv, PrepareFileReadOp, PrepareFileWriteOp};
use std::collections::HashMap;

/// Generic graph op enum for file-based DAGs.
#[derive(Debug, Clone)]
pub enum FileOpsGraph<D> {
    /// Domain-specific operations.
    Domain(D),
    /// Filesystem environment (resource acquisition).
    FsEnv(FsEnv),
    /// Prepare file read (primitive - PURE).
    PrepareFileRead(PrepareFileReadOp),
    /// Prepare file write (primitive - PURE).
    PrepareFileWrite(PrepareFileWriteOp),
    /// Blob operations (compare content - PURE).
    Blob(BlobOps),
    /// Transport operations (boundary - actual I/O).
    Transport(TransportOps),
}

impl<D: Executable> Executable for FileOpsGraph<D> {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            FileOpsGraph::Domain(op) => op.execute(inputs),
            FileOpsGraph::FsEnv(op) => op.execute(inputs),
            FileOpsGraph::PrepareFileRead(op) => op.execute(inputs),
            FileOpsGraph::PrepareFileWrite(op) => op.execute(inputs),
            FileOpsGraph::Blob(op) => op.execute(inputs),
            FileOpsGraph::Transport(op) => op.execute(inputs),
        }
    }
}
