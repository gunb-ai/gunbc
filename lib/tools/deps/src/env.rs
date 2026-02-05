//! Environment ops for deps graphs.

use crate::platform::Platform;
use gunbc_exec::{ExecError, Executable, OutputMap};
use gunbc_ir::Value;
use std::collections::HashMap;

/// Platform environment — detects current platform.
#[derive(Debug, Clone, Copy)]
pub struct PlatformEnv;

impl PlatformEnv {
    pub fn output_port(&self) -> &'static str {
        "platform"
    }

    pub fn mock_outputs(&self) -> HashMap<String, Value> {
        let platform = Platform::detect();
        OutputMap::new()
            .value(self.output_port(), platform.into())
            .build()
    }
}

impl Executable for PlatformEnv {
    fn execute(
        &self,
        _inputs: HashMap<String, Value>,
    ) -> Result<HashMap<String, Value>, ExecError> {
        let platform = Platform::detect();
        OutputMap::new()
            .value(self.output_port(), platform.into())
            .ok()
    }
}
