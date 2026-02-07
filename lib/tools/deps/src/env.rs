//! Environment ops for deps graphs.

use crate::platform::Platform;
use gunbc_exec::{env_single_output, EnvNode, ExecError};
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
        self.outputs()
    }

    fn outputs(&self) -> HashMap<String, Value> {
        let platform = Platform::detect();
        env_single_output(self.output_port(), platform)
    }
}

impl EnvNode for PlatformEnv {
    fn env_outputs(&self) -> Result<HashMap<String, Value>, ExecError> {
        Ok(self.outputs())
    }

    fn mock_outputs(&self) -> HashMap<String, Value> {
        self.outputs()
    }
}
