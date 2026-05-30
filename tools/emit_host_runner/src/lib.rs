//! v4 `extdeps/runtimes/emit_host.dag` Rust host row — compile + execute emitted artifacts.
//!
//! **Modeled authority:** `run_emit_host_rust`, `runtime_value_parse_rust` in
//! `src/v4/extdeps/runtimes/emit_host.dag`. Substrate eval returns `Rejected` until this
//! transport is invoked from CI/scripts; this crate is the executable boundary.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// Typed exit: success only when the child process exited 0.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExitOk {
    pub code: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HostExit {
    Ok(ExitOk),
    Err(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildLog {
    pub lines: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmitHostRunReceipt {
    pub source_text: String,
    pub exit: HostExit,
    pub stdout_bytes: Vec<u8>,
    pub stderr_bytes: Vec<u8>,
    pub build_log: BuildLog,
}

/// MVP-2 / eval_runtime_mvp alignment: five stdout bytes denote runtime value `5`.
pub fn runtime_value_parse_rust(bytes: &[u8]) -> Result<(), String> {
    if bytes.len() == 5 {
        Ok(())
    } else {
        Err(format!(
            "runtime_value_parse_rust: expected 5 stdout bytes, got {}",
            bytes.len()
        ))
    }
}

fn output_to_log(output: &Output) -> BuildLog {
    let mut lines = Vec::new();
    if !output.stdout.is_empty() {
        lines.push(format!(
            "stdout: {}",
            String::from_utf8_lossy(&output.stdout)
        ));
    }
    if !output.stderr.is_empty() {
        lines.push(format!(
            "stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    lines.push(format!("status: {}", output.status));
    BuildLog { lines }
}

/// Compile `source` as a Rust binary crate in `work_dir`, run it, capture stdout/stderr.
pub fn run_emit_host_rust(source: &str, work_dir: &Path) -> Result<EmitHostRunReceipt, String> {
    fs::create_dir_all(work_dir).map_err(|e| format!("create work_dir: {e}"))?;
    let src_dir = work_dir.join("src");
    fs::create_dir_all(&src_dir).map_err(|e| format!("create src: {e}"))?;

    let cargo_toml = work_dir.join("Cargo.toml");
    let manifest = "[package]\nname = \"emit_host_fixture\"\nversion = \"0.0.0\"\nedition = \"2021\"\n\n[[bin]]\nname = \"fixture\"\npath = \"src/main.rs\"\n";
    let mut f = fs::File::create(&cargo_toml).map_err(|e| format!("create Cargo.toml: {e}"))?;
    f.write_all(manifest.as_bytes())
        .map_err(|e| format!("write Cargo.toml: {e}"))?;

    let main_rs = src_dir.join("main.rs");
    fs::write(&main_rs, source).map_err(|e| format!("write main.rs: {e}"))?;

    let build = Command::new("cargo")
        .args(["build", "--quiet", "--manifest-path"])
        .arg(&cargo_toml)
        .output()
        .map_err(|e| format!("cargo build spawn: {e}"))?;
    let build_log = output_to_log(&build);
    if !build.status.success() {
        return Ok(EmitHostRunReceipt {
            source_text: source.to_string(),
            exit: HostExit::Err(format!("cargo build failed: {}", build.status)),
            stdout_bytes: build.stdout,
            stderr_bytes: build.stderr,
            build_log,
        });
    }

    let bin_path = work_dir.join("target/debug/fixture");
    let run = Command::new(&bin_path)
        .output()
        .map_err(|e| format!("run fixture: {e}"))?;
    let run_log = output_to_log(&run);
    let mut lines = build_log.lines;
    lines.extend(run_log.lines);
    let exit = if run.status.success() {
        HostExit::Ok(ExitOk {
            code: run.status.code().unwrap_or(0),
        })
    } else {
        HostExit::Err(format!("fixture run failed: {}", run.status))
    };
    Ok(EmitHostRunReceipt {
        source_text: source.to_string(),
        exit,
        stdout_bytes: run.stdout,
        stderr_bytes: run.stderr,
        build_log: BuildLog { lines },
    })
}

/// Default temp directory under `std::env::temp_dir()`.
pub fn default_work_dir(prefix: &str) -> PathBuf {
    std::env::temp_dir().join(prefix)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_value_parse_rust_accepts_five_bytes() {
        assert!(runtime_value_parse_rust(&[0, 0, 0, 0, 0]).is_ok());
        assert!(runtime_value_parse_rust(&[1, 2, 3]).is_err());
    }
}
