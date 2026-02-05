//! Authentication environment node.

use gunbc_exec::{require_str, ExecError, Executable, OutputMap};
use gunbc_ir::{AuthToken, SecretString, Value};
use std::collections::HashMap;

/// Auth environment — resolves an env var to an AuthToken.
#[derive(Debug, Clone)]
pub struct AuthEnv {
    mode: AuthEnvMode,
    output_port: String,
}

#[derive(Debug, Clone)]
enum AuthEnvMode {
    Static { service: String, env_var: String },
    FromInputs,
}

impl AuthEnv {
    pub fn new(service: impl Into<String>, env_var: impl Into<String>) -> Self {
        let service = service.into();
        let env_var = env_var.into();
        let output_port = format!("auth:{}", service);
        Self {
            mode: AuthEnvMode::Static { service, env_var },
            output_port,
        }
    }

    pub fn from_inputs(output_port: impl Into<String>) -> Self {
        Self {
            mode: AuthEnvMode::FromInputs,
            output_port: output_port.into(),
        }
    }

    pub fn output_port(&self) -> &str {
        &self.output_port
    }

    /// Mock outputs for DryRun/testgen.
    pub fn mock_outputs(&self) -> HashMap<String, Value> {
        let (service, env_var) = match &self.mode {
            AuthEnvMode::Static { service, env_var } => (service.clone(), env_var.clone()),
            AuthEnvMode::FromInputs => {
                let service = self
                    .output_port
                    .strip_prefix("auth:")
                    .unwrap_or("auth")
                    .to_string();
                let env_var = service.to_uppercase();
                (service, env_var)
            }
        };
        let token = AuthToken::new(service, env_var, SecretString::new("mock-token"));
        let port = self.output_port();
        OutputMap::new().value(port, token.into()).build()
    }

    fn resolve_service_and_env(
        &self,
        inputs: &HashMap<String, Value>,
    ) -> Result<(String, String), ExecError> {
        match &self.mode {
            AuthEnvMode::Static { service, env_var } => Ok((service.clone(), env_var.clone())),
            AuthEnvMode::FromInputs => {
                let service = require_str(inputs, "service")?;
                let env_var = require_str(inputs, "env_var")?;
                Ok((service.to_string(), env_var.to_string()))
            }
        }
    }
}

impl Executable for AuthEnv {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let (service, env_var) = self.resolve_service_and_env(&inputs)?;
        let value = std::env::var(&env_var)
            .map_err(|_| ExecError::new(format!("missing env var '{}'", env_var)))?;
        let token = AuthToken::new(service, env_var, SecretString::new(value));
        let port = self.output_port();

        OutputMap::new().value(port, token.into()).ok()
    }
}
