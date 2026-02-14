//! Environment resource acquisition ops.

use crate::filename::{FilesystemHandle, Scope};
use crate::network::NetworkHandle;
use gunbc_exec::{env_single_output, EnvNode, ExecError};
use gunbc_ir::{
    Timestamp, Value, API_NETWORK_HANDLE_PORT, FILE_HANDLE_READ_PORT, FILE_HANDLE_WRITE_PORT,
};
use std::collections::HashMap;

/// Filesystem environment — acquires a FilesystemHandle.
#[derive(Debug, Clone)]
pub struct FsEnv {
    pub scope: Scope,
}

impl FsEnv {
    pub const READ_PORT: &'static str = FILE_HANDLE_READ_PORT;
    pub const WRITE_PORT: &'static str = FILE_HANDLE_WRITE_PORT;

    pub fn new(scope: Scope) -> Self {
        Self { scope }
    }

    pub fn output_port(&self) -> &'static str {
        match self.scope {
            Scope::Read => Self::READ_PORT,
            Scope::Write => Self::WRITE_PORT,
        }
    }

    /// Mock outputs for DryRun/testgen.
    pub fn mock_outputs(&self) -> HashMap<String, Value> {
        self.mock_output_map()
    }

    fn outputs(&self) -> HashMap<String, Value> {
        let fs = FilesystemHandle::cross_platform(self.scope);
        env_single_output(self.output_port(), fs)
    }

    fn mock_output_map(&self) -> HashMap<String, Value> {
        self.outputs()
    }
}

impl EnvNode for FsEnv {
    fn env_outputs(&self) -> Result<HashMap<String, Value>, ExecError> {
        Ok(self.outputs())
    }

    fn mock_outputs(&self) -> HashMap<String, Value> {
        self.mock_output_map()
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
        self.mock_output_map()
    }

    fn outputs(&self) -> HashMap<String, Value> {
        let ts = Timestamp::now();
        env_single_output(self.output_port(), ts)
    }

    fn mock_output_map(&self) -> HashMap<String, Value> {
        let ts = Timestamp::from_system_time(std::time::SystemTime::UNIX_EPOCH);
        env_single_output(self.output_port(), ts)
    }
}

impl EnvNode for ClockEnv {
    fn env_outputs(&self) -> Result<HashMap<String, Value>, ExecError> {
        Ok(self.outputs())
    }

    fn mock_outputs(&self) -> HashMap<String, Value> {
        self.mock_output_map()
    }
}

/// Network environment — acquires a NetworkHandle.
#[derive(Debug, Clone, Copy)]
pub struct NetEnv;

impl NetEnv {
    pub const PORT: &'static str = API_NETWORK_HANDLE_PORT;

    pub fn output_port(&self) -> &'static str {
        Self::PORT
    }

    /// Mock outputs for DryRun/testgen.
    pub fn mock_outputs(&self) -> HashMap<String, Value> {
        self.mock_output_map()
    }

    fn outputs(&self) -> HashMap<String, Value> {
        env_single_output(self.output_port(), NetworkHandle)
    }

    fn mock_output_map(&self) -> HashMap<String, Value> {
        self.outputs()
    }
}

impl EnvNode for NetEnv {
    fn env_outputs(&self) -> Result<HashMap<String, Value>, ExecError> {
        Ok(self.outputs())
    }

    fn mock_outputs(&self) -> HashMap<String, Value> {
        self.mock_output_map()
    }
}
