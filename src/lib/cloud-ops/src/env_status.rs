//! Environment status helper for cloud credential flows.

use crate::env_requirements::{collect_missing_requirements, detect_cloud_env_requirements};
use gunbc_exec::{EnvNode, ExecError, OutputMap};
use gunbc_ir::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, Default)]
pub struct CloudEnvStatus;

impl CloudEnvStatus {
    pub fn new() -> Self {
        Self
    }

    pub fn output_port(&self) -> &'static str {
        "status"
    }
}

fn env_truthy(name: &str) -> bool {
    std::env::var(name)
        .map(|v| matches!(v.as_str(), "1" | "true" | "TRUE"))
        .unwrap_or(false)
}

fn build_status_message() -> String {
    let req = detect_cloud_env_requirements();
    let prefix = format!(
        "Cloud env ({}/{})",
        req.provider.as_str(),
        req.runtime.as_str()
    );
    let is_ci = env_truthy("GITHUB_ACTIONS") || env_truthy("GITLAB_CI");
    let missing = collect_missing_requirements(&req);
    let missing_groups: Vec<String> = missing
        .missing_any_of
        .iter()
        .map(|group| group.join(" | "))
        .collect();

    let mut parts: Vec<String> = Vec::new();
    if !missing.missing_required.is_empty() {
        parts.push(format!("missing: {}", missing.missing_required.join(", ")));
    }
    if !missing_groups.is_empty() {
        parts.push(format!("missing any-of: {}", missing_groups.join(", ")));
    }

    if parts.is_empty() {
        let mut msg = format!("{prefix}: OK");
        if let Some(notes) = req.notes {
            msg.push_str(" — ");
            msg.push_str(notes);
        }
        msg
    } else {
        let mut msg = format!("{prefix}: {}", parts.join("; "));
        if !is_ci {
            msg.push_str(" — local tests will skip live calls");
        }
        if let Some(notes) = req.notes {
            msg.push_str(" — ");
            msg.push_str(notes);
        }
        msg
    }
}

impl EnvNode for CloudEnvStatus {
    fn env_outputs(&self) -> Result<HashMap<String, Value>, ExecError> {
        let status = build_status_message();
        Ok(OutputMap::new().str(self.output_port(), status).build())
    }

    fn mock_outputs(&self) -> HashMap<String, Value> {
        OutputMap::new()
            .str(self.output_port(), "Cloud env (gcp/github): OK (mock)")
            .build()
    }
}
