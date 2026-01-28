use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

use gunbc_exec::{ExecError, Executable, Value};

#[derive(Debug, Clone)]
pub enum DepOp {
    CheckCommand {
        name: String,
        cmd: &'static str,
    },
    CheckPath {
        name: String,
        path: &'static str,
    },
    InstallCommand {
        name: String,
        cmd: CommandSpec,
    },
    PreviewInstall {
        name: String,
        cmd: CommandSpec,
    },
    FailIfMissing {
        name: String,
    },
    ResolveUpsert {
        name: String,
    },
    Gate {
        name: String,
    },
}

#[derive(Debug, Clone, Copy)]
pub struct CommandSpec {
    pub linux: Option<&'static str>,
    pub macos: Option<&'static str>,
    pub windows: Option<&'static str>,
}

impl CommandSpec {
    pub const fn linux(cmd: &'static str) -> Self {
        Self {
            linux: Some(cmd),
            macos: None,
            windows: None,
        }
    }

    pub const fn macos(cmd: &'static str) -> Self {
        Self {
            linux: None,
            macos: Some(cmd),
            windows: None,
        }
    }

    pub const fn windows(cmd: &'static str) -> Self {
        Self {
            linux: None,
            macos: None,
            windows: Some(cmd),
        }
    }

    pub const fn all(cmd: &'static str) -> Self {
        Self {
            linux: Some(cmd),
            macos: Some(cmd),
            windows: Some(cmd),
        }
    }

    pub fn for_current(&self) -> Result<&'static str, ExecError> {
        match std::env::consts::OS {
            "linux" => self.linux.ok_or_else(|| ExecError("no linux install command".into())),
            "macos" => self.macos.ok_or_else(|| ExecError("no macos install command".into())),
            "windows" => self.windows.ok_or_else(|| ExecError("no windows install command".into())),
            other => Err(ExecError(format!("unsupported platform: {other}"))),
        }
    }
}

impl Executable for DepOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            DepOp::CheckCommand { name, cmd } => {
                let present = run_shell(cmd).is_ok();
                Ok(outputs_check(name, present))
            }
            DepOp::CheckPath { name, path } => {
                let present = Path::new(path).exists();
                Ok(outputs_check(name, present))
            }
            DepOp::InstallCommand { name, cmd } => {
                if let Some(dep_ok) = inputs.get("deps_ok") {
                    if !is_true(dep_ok) {
                        return Err(ExecError(format!("{} prerequisites not satisfied", name)));
                    }
                }

                let command = cmd.for_current()?;
                run_shell(command).map_err(|e| {
                    ExecError(format!("{} install failed: {}", name, e.0))
                })?;

                let mut outputs = HashMap::new();
                outputs.insert("installed".to_string(), Value::Bool(true));
                Ok(outputs)
            }
            DepOp::PreviewInstall { name, cmd } => {
                if let Some(dep_ok) = inputs.get("deps_ok") {
                    if !is_true(dep_ok) {
                        return Err(ExecError(format!("{} prerequisites not satisfied", name)));
                    }
                }

                let command = cmd.for_current()?;
                println!("[dry-run] {}: {}", name, command);

                let mut outputs = HashMap::new();
                outputs.insert("installed".to_string(), Value::Bool(true));
                Ok(outputs)
            }
            DepOp::FailIfMissing { name } => {
                Err(ExecError(format!(
                    "{} missing and installs are disabled (use upsert mode)",
                    name
                )))
            }
            DepOp::ResolveUpsert { name } => {
                let present = inputs.get("present").ok_or_else(|| {
                    ExecError(format!("{} resolve missing 'present' input", name))
                })?;
                let installed = inputs.get("installed").ok_or_else(|| {
                    ExecError(format!("{} resolve missing 'installed' input", name))
                })?;

                let present_ok = is_true(present);
                let installed_ok = is_true(installed);

                if !present_ok && !installed_ok {
                    return Err(ExecError(format!(
                        "{} unresolved (not present and install did not run)",
                        name
                    )));
                }

                let mut outputs = HashMap::new();
                outputs.insert("ok".to_string(), Value::Bool(true));
                Ok(outputs)
            }
            DepOp::Gate { name } => {
                let mut missing = Vec::new();
                for (k, v) in &inputs {
                    if !is_true(v) {
                        missing.push(k.clone());
                    }
                }

                if !missing.is_empty() {
                    missing.sort();
                    return Err(ExecError(format!(
                        "{} prerequisites failed: {}",
                        name,
                        missing.join(", ")
                    )));
                }

                let mut outputs = HashMap::new();
                outputs.insert("ok".to_string(), Value::Bool(true));
                Ok(outputs)
            }
        }
    }
}

fn outputs_check(name: &str, present: bool) -> HashMap<String, Value> {
    let mut outputs = HashMap::new();
    outputs.insert("present".to_string(), Value::Bool(present));
    outputs.insert("needs_create".to_string(), Value::Bool(!present));
    let _ = name;
    outputs
}

fn is_true(value: &Value) -> bool {
    matches!(value, Value::Bool(true))
}

fn run_shell(cmd: &str) -> Result<(), ExecError> {
    let status = Command::new("bash")
        .args(["-c", cmd])
        .status()
        .map_err(|e| ExecError(format!("failed to launch shell: {e}")))?;

    if status.success() {
        Ok(())
    } else {
        Err(ExecError(format!("command failed: {cmd}")))
    }
}
