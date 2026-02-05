//! Environment resource acquisition ops.

use crate::filename::{FilesystemHandle, Scope};
use gunbc_exec::{ExecError, Executable, OutputMap};
use gunbc_ir::{Timestamp, Value};
use std::collections::HashMap;

/// Filesystem environment — acquires a FilesystemHandle.
#[derive(Debug, Clone)]
pub struct FsEnv {
    pub scope: Scope,
}

impl FsEnv {
    pub fn new(scope: Scope) -> Self {
        Self { scope }
    }

    pub fn output_port(&self) -> &'static str {
        match self.scope {
            Scope::Read => "fs:read",
            Scope::Write => "fs:write",
        }
    }

    /// Mock outputs for DryRun/testgen.
    pub fn mock_outputs(&self) -> HashMap<String, Value> {
        let fs = FilesystemHandle::cross_platform(self.scope);
        OutputMap::new()
            .value(self.output_port(), fs.into())
            .build()
    }
}

impl Executable for FsEnv {
    fn execute(
        &self,
        _inputs: HashMap<String, Value>,
    ) -> Result<HashMap<String, Value>, ExecError> {
        let fs = FilesystemHandle::cross_platform(self.scope);
        OutputMap::new().value(self.output_port(), fs.into()).ok()
    }
}

/// Clock environment — captures a timestamp snapshot.
#[derive(Debug, Clone, Copy)]
pub struct ClockEnv;

impl ClockEnv {
    pub fn output_port(&self) -> &'static str {
        "clock"
    }

    /// Mock outputs for DryRun/testgen.
    pub fn mock_outputs(&self) -> HashMap<String, Value> {
        let ts = Timestamp::now();
        OutputMap::new()
            .value(self.output_port(), ts.into())
            .build()
    }
}

impl Executable for ClockEnv {
    fn execute(
        &self,
        _inputs: HashMap<String, Value>,
    ) -> Result<HashMap<String, Value>, ExecError> {
        let ts = Timestamp::now();
        OutputMap::new().value(self.output_port(), ts.into()).ok()
    }
}
