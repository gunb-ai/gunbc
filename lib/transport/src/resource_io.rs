//! Transport-backed Resource I/O implementation.
//!
//! This adapter routes all filesystem and shell access through the transport
//! execution boundary.

use crate::ops::execute_request;
use gunbc_ir::resource::{ResourceError, ResourceIo};
use gunbc_ir::transport::{FileRequest, ShellRequest, TransportRequest, TransportResponse};
use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Transport-backed I/O resolver for resource hashing and manifest storage.
#[derive(Debug, Clone, Copy, Default)]
pub struct TransportIo;

impl TransportIo {
    pub fn new() -> Self {
        Self
    }
}

impl ResourceIo for TransportIo {
    fn read_file(&self, path: &Path) -> Result<Vec<u8>, ResourceError> {
        let request = TransportRequest::File(FileRequest::read(path.to_string_lossy()));
        let response = execute_request(&request).map_err(exec_to_resource)?;
        match response {
            TransportResponse::File(file) if file.success => {
                Ok(file.content.unwrap_or_default().into_bytes())
            }
            TransportResponse::File(file) => Err(ResourceError::Io(io::Error::other(
                file.error.unwrap_or_else(|| "file read failed".to_string()),
            ))),
            other => Err(ResourceError::Io(io::Error::other(format!(
                "unexpected transport response for read: {:?}",
                std::mem::discriminant(&other)
            )))),
        }
    }

    fn write_file(&self, path: &Path, contents: &[u8]) -> Result<(), ResourceError> {
        let content = String::from_utf8(contents.to_vec())
            .map_err(|e| ResourceError::Io(io::Error::other(e.to_string())))?;
        let request = TransportRequest::File(FileRequest::write(path.to_string_lossy(), content));
        let response = execute_request(&request).map_err(exec_to_resource)?;
        match response {
            TransportResponse::File(file) if file.success => Ok(()),
            TransportResponse::File(file) => Err(ResourceError::Io(io::Error::other(
                file.error
                    .unwrap_or_else(|| "file write failed".to_string()),
            ))),
            other => Err(ResourceError::Io(io::Error::other(format!(
                "unexpected transport response for write: {:?}",
                std::mem::discriminant(&other)
            )))),
        }
    }

    fn file_exists(&self, path: &Path) -> Result<bool, ResourceError> {
        let request = TransportRequest::File(FileRequest::exists(path.to_string_lossy()));
        let response = execute_request(&request).map_err(exec_to_resource)?;
        match response {
            TransportResponse::File(file) if file.success => Ok(file.exists.unwrap_or(false)),
            TransportResponse::File(file) => Err(ResourceError::Io(io::Error::other(
                file.error
                    .unwrap_or_else(|| "file exists check failed".to_string()),
            ))),
            other => Err(ResourceError::Io(io::Error::other(format!(
                "unexpected transport response for exists: {:?}",
                std::mem::discriminant(&other)
            )))),
        }
    }

    fn glob_paths(&self, pattern: &str) -> Result<Vec<PathBuf>, ResourceError> {
        let request = TransportRequest::File(FileRequest::glob(pattern));
        let response = execute_request(&request).map_err(exec_to_resource)?;
        match response {
            TransportResponse::File(file) if file.success => {
                let mut paths = Vec::new();
                if let Some(content) = file.content {
                    for line in content.lines() {
                        let trimmed = line.trim();
                        if !trimmed.is_empty() {
                            paths.push(PathBuf::from(trimmed));
                        }
                    }
                }
                Ok(paths)
            }
            TransportResponse::File(file) => Err(ResourceError::Io(io::Error::other(
                file.error.unwrap_or_else(|| "glob failed".to_string()),
            ))),
            other => Err(ResourceError::Io(io::Error::other(format!(
                "unexpected transport response for glob: {:?}",
                std::mem::discriminant(&other)
            )))),
        }
    }

    fn command_output(&self, command: &str, args: &[String]) -> Result<Vec<u8>, ResourceError> {
        let request = ShellRequest::new(command)
            .args(args.iter().cloned())
            .into_transport_request();
        let response = execute_request(&request).map_err(exec_to_resource)?;
        match response {
            TransportResponse::Shell(shell) if shell.success() => Ok(shell.stdout.into_bytes()),
            TransportResponse::Shell(shell) => Err(ResourceError::Io(io::Error::other(format!(
                "command failed (exit {}): {}",
                shell.exit_code, shell.stderr
            )))),
            other => Err(ResourceError::Io(io::Error::other(format!(
                "unexpected transport response for shell: {:?}",
                std::mem::discriminant(&other)
            )))),
        }
    }

    fn file_mtime(&self, path: &Path) -> Result<SystemTime, ResourceError> {
        let request = TransportRequest::File(FileRequest::metadata(path.to_string_lossy()));
        let response = execute_request(&request).map_err(exec_to_resource)?;
        match response {
            TransportResponse::File(file) if file.success => {
                let content = file.content.ok_or_else(|| {
                    ResourceError::Io(io::Error::other("metadata missing content"))
                })?;
                let millis = content.trim().parse::<i64>().map_err(|e| {
                    ResourceError::Io(io::Error::other(format!("metadata parse failed: {}", e)))
                })?;
                if millis >= 0 {
                    Ok(UNIX_EPOCH + Duration::from_millis(millis as u64))
                } else {
                    Ok(UNIX_EPOCH)
                }
            }
            TransportResponse::File(file) => Err(ResourceError::Io(io::Error::other(
                file.error.unwrap_or_else(|| "metadata failed".to_string()),
            ))),
            other => Err(ResourceError::Io(io::Error::other(format!(
                "unexpected transport response for metadata: {:?}",
                std::mem::discriminant(&other)
            )))),
        }
    }
}

fn exec_to_resource(err: gunbc_exec::ExecError) -> ResourceError {
    ResourceError::Io(io::Error::other(err.to_string()))
}
