//! Concrete credential providers and the `CredentialOp` boundary node.
//!
//! Providers acquire [`Credential`]s from the environment.
//! [`CredentialOp`] is the DAG boundary node that drives them,
//! following the same environment-node pattern.

use gunbc_exec::{ExecError, Executable, OutputMap};
use gunbc_ir::{
    AuthScheme, Credential, CredentialError, CredentialProvider, Secret, SecretSource, Value,
};
use std::collections::HashMap;
use std::sync::Arc;

// ---------------------------------------------------------------------------
// GitHubEnvVarProvider
// ---------------------------------------------------------------------------

/// Reads a GitHub token from an environment variable.
#[derive(Debug, Clone)]
pub struct GitHubEnvVarProvider {
    env_var: String,
}

impl GitHubEnvVarProvider {
    /// Create a provider that reads `GITHUB_TOKEN` by default.
    pub fn new() -> Self {
        Self {
            env_var: "GITHUB_TOKEN".to_string(),
        }
    }

    /// Create a provider that reads a custom environment variable.
    pub fn with_env_var(env_var: impl Into<String>) -> Self {
        Self {
            env_var: env_var.into(),
        }
    }
}

impl CredentialProvider for GitHubEnvVarProvider {
    fn service_id(&self) -> &str {
        "github"
    }

    fn acquire(&self) -> Result<Credential, CredentialError> {
        let value = std::env::var(&self.env_var).map_err(|_| CredentialError::MissingEnvVar {
            var_name: self.env_var.clone(),
        })?;
        let secret = Secret::from_env_var(&self.env_var, value);
        Ok(Credential::new(secret, AuthScheme::Bearer))
    }
}

// ---------------------------------------------------------------------------
// LlmEnvVarProvider
// ---------------------------------------------------------------------------

/// Reads an LLM API key from an environment variable with the correct scheme.
#[derive(Debug, Clone)]
pub struct LlmEnvVarProvider {
    service: String,
    env_var: String,
    scheme: AuthScheme,
}

impl LlmEnvVarProvider {
    /// OpenAI: reads `OPENAI_API_KEY`, scheme = Bearer.
    pub fn openai() -> Self {
        Self {
            service: "openai".to_string(),
            env_var: "OPENAI_API_KEY".to_string(),
            scheme: AuthScheme::Bearer,
        }
    }

    /// Anthropic: reads `ANTHROPIC_API_KEY`, scheme = Header(`x-api-key`).
    pub fn anthropic() -> Self {
        Self {
            service: "anthropic".to_string(),
            env_var: "ANTHROPIC_API_KEY".to_string(),
            scheme: AuthScheme::Header {
                name: "x-api-key".to_string(),
            },
        }
    }
}

impl CredentialProvider for LlmEnvVarProvider {
    fn service_id(&self) -> &str {
        &self.service
    }

    fn acquire(&self) -> Result<Credential, CredentialError> {
        let value = std::env::var(&self.env_var).map_err(|_| CredentialError::MissingEnvVar {
            var_name: self.env_var.clone(),
        })?;
        let secret = Secret::from_env_var(&self.env_var, value);
        Ok(Credential::new(secret, self.scheme.clone()))
    }
}

// ---------------------------------------------------------------------------
// MockCredentialProvider
// ---------------------------------------------------------------------------

/// Returns a predetermined credential (for tests / DryRun).
#[derive(Debug, Clone)]
pub struct MockCredentialProvider {
    service: String,
    credential: Credential,
}

impl MockCredentialProvider {
    /// Create a mock provider that always returns the given credential.
    pub fn new(service: impl Into<String>, credential: Credential) -> Self {
        Self {
            service: service.into(),
            credential,
        }
    }
}

impl CredentialProvider for MockCredentialProvider {
    fn service_id(&self) -> &str {
        &self.service
    }

    fn acquire(&self) -> Result<Credential, CredentialError> {
        Ok(self.credential.clone())
    }
}

// ---------------------------------------------------------------------------
// CredentialOp
// ---------------------------------------------------------------------------

/// Internal mode for CredentialOp.
#[derive(Debug, Clone)]
enum CredentialOpMode {
    /// Acquire credentials from pre-configured providers.
    Static {
        providers: Vec<Arc<dyn CredentialProvider>>,
    },
    /// Construct a credential from DAG inputs at runtime.
    ///
    /// Reads `service`, `env_var`, `scheme` (optional, default "bearer"),
    /// `header_name` (optional) from inputs. Calls `std::env::var(env_var)`
    /// to get the secret, then emits on the configured output port.
    FromInputs {
        output_port: String,
    },
}

/// Boundary node that acquires credentials from providers and emits them
/// on `"credential:{service_id}"` output ports.
///
/// Supports two modes:
/// - **Static**: Pre-configured providers (original pattern)
/// - **FromInputs**: Reads service/env_var/scheme from DAG inputs at runtime
///
/// Follows the same environment-node pattern:
/// - `execute()` calls each provider's `acquire()`
/// - `mock_outputs()` returns mock credentials for DryRun interception
#[derive(Debug, Clone)]
pub struct CredentialOp {
    mode: CredentialOpMode,
}

impl CredentialOp {
    /// Create a new credential op with the given providers (Static mode).
    pub fn new(providers: Vec<Arc<dyn CredentialProvider>>) -> Self {
        Self {
            mode: CredentialOpMode::Static { providers },
        }
    }

    /// Create a credential op that reads inputs at runtime (FromInputs mode).
    ///
    /// Reads `service`, `env_var`, and optionally `scheme` / `header_name`
    /// from DAG inputs. Emits a Credential on the given output port.
    pub fn from_inputs(output_port: impl Into<String>) -> Self {
        Self {
            mode: CredentialOpMode::FromInputs {
                output_port: output_port.into(),
            },
        }
    }

    /// Output port name for a given service.
    pub fn output_port(service_id: &str) -> String {
        format!("credential:{service_id}")
    }

    /// Mock outputs for DryRun / testgen.
    pub fn mock_outputs(&self) -> HashMap<String, Value> {
        match &self.mode {
            CredentialOpMode::Static { providers } => {
                let mut builder = OutputMap::new();
                for provider in providers {
                    let port = Self::output_port(provider.service_id());
                    let secret =
                        Secret::new("mock-token".to_string(), SecretSource::Static, None);
                    let cred = Credential::new(secret, AuthScheme::Bearer);
                    builder = builder.value(&port, cred.into());
                }
                builder.build()
            }
            CredentialOpMode::FromInputs { output_port } => {
                let secret = Secret::new("mock-token".to_string(), SecretSource::Static, None);
                let cred = Credential::new(secret, AuthScheme::Bearer);
                OutputMap::new().value(output_port, cred.into()).build()
            }
        }
    }
}

impl Executable for CredentialOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match &self.mode {
            CredentialOpMode::Static { providers } => {
                let mut builder = OutputMap::new();
                for provider in providers {
                    let cred =
                        provider.acquire().map_err(|e| ExecError::new(e.to_string()))?;
                    let port = Self::output_port(provider.service_id());
                    builder = builder.value(&port, cred.into());
                }
                builder.ok()
            }
            CredentialOpMode::FromInputs { output_port } => {
                let env_var = gunbc_exec::require_str(&inputs, "env_var")?;

                let scheme_str = inputs
                    .get("scheme")
                    .and_then(Value::as_str)
                    .unwrap_or("bearer");
                let header_name = inputs
                    .get("header_name")
                    .and_then(Value::as_str)
                    .map(|s| s.to_string());

                let scheme = match scheme_str {
                    "bearer" => AuthScheme::Bearer,
                    "header" => {
                        let name = header_name.ok_or_else(|| {
                            ExecError::new(
                                "scheme 'header' requires 'header_name' input",
                            )
                        })?;
                        AuthScheme::Header { name }
                    }
                    other => {
                        return Err(ExecError::new(format!(
                            "unknown scheme '{}' (expected 'bearer' or 'header')",
                            other
                        )));
                    }
                };

                let value = std::env::var(env_var).map_err(|_| {
                    ExecError::new(format!("missing env var '{}'", env_var))
                })?;
                let secret = Secret::from_env_var(env_var, value);
                let cred = Credential::new(secret, scheme);

                OutputMap::new().value(output_port, cred.into()).ok()
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::SecretSource;

    #[test]
    fn github_env_var_provider_acquire_success() {
        // Set env var for the test
        let var = "GUNBC_TEST_GH_TOKEN_8273";
        std::env::set_var(var, "ghp_test_123");
        let provider = GitHubEnvVarProvider::with_env_var(var);

        let cred = provider.acquire().expect("should succeed");
        assert_eq!(cred.secret().expose(), "ghp_test_123");
        assert!(matches!(cred.scheme(), AuthScheme::Bearer));
        assert!(matches!(
            cred.secret().source(),
            SecretSource::EnvVar(ref v) if v == var
        ));

        std::env::remove_var(var);
    }

    #[test]
    fn github_env_var_provider_acquire_failure() {
        // Ensure env var is not set
        let var = "GUNBC_TEST_GH_MISSING_9182";
        std::env::remove_var(var);
        let provider = GitHubEnvVarProvider::with_env_var(var);

        let err = provider.acquire().unwrap_err();
        assert!(matches!(
            err,
            CredentialError::MissingEnvVar { ref var_name } if var_name == var
        ));
    }

    #[test]
    fn llm_env_var_provider_openai() {
        let var = "GUNBC_TEST_OPENAI_KEY_3847";
        std::env::set_var(var, "sk-test-openai");

        // Use a custom provider with the test env var
        let provider = LlmEnvVarProvider {
            service: "openai".to_string(),
            env_var: var.to_string(),
            scheme: AuthScheme::Bearer,
        };

        let cred = provider.acquire().expect("should succeed");
        assert_eq!(cred.secret().expose(), "sk-test-openai");
        assert!(matches!(cred.scheme(), AuthScheme::Bearer));

        std::env::remove_var(var);
    }

    #[test]
    fn llm_env_var_provider_anthropic() {
        let var = "GUNBC_TEST_ANTHROPIC_KEY_5921";
        std::env::set_var(var, "sk-ant-test");

        let provider = LlmEnvVarProvider {
            service: "anthropic".to_string(),
            env_var: var.to_string(),
            scheme: AuthScheme::Header {
                name: "x-api-key".to_string(),
            },
        };

        let cred = provider.acquire().expect("should succeed");
        assert_eq!(cred.secret().expose(), "sk-ant-test");
        assert!(matches!(
            cred.scheme(),
            AuthScheme::Header { ref name } if name == "x-api-key"
        ));

        std::env::remove_var(var);
    }

    #[test]
    fn mock_credential_provider_returns_expected() {
        let secret = Secret::static_value("mock-secret");
        let expected = Credential::new(secret, AuthScheme::Bearer);
        let provider = MockCredentialProvider::new("test-service", expected.clone());

        assert_eq!(provider.service_id(), "test-service");
        let cred = provider.acquire().expect("should succeed");
        assert_eq!(cred.secret().expose(), "mock-secret");
    }

    #[test]
    fn credential_op_execute_with_mock_providers() {
        let p1 = Arc::new(MockCredentialProvider::new(
            "github",
            Credential::new(Secret::static_value("gh-tok"), AuthScheme::Bearer),
        ));
        let p2 = Arc::new(MockCredentialProvider::new(
            "openai",
            Credential::new(Secret::static_value("oai-tok"), AuthScheme::Bearer),
        ));

        let op = CredentialOp::new(vec![p1, p2]);
        let outputs = op.execute(HashMap::new()).expect("should succeed");

        assert!(outputs.contains_key("credential:github"));
        assert!(outputs.contains_key("credential:openai"));

        // Verify round-trip through Value
        let gh_cred =
            Credential::try_from(outputs.get("credential:github").unwrap()).expect("round-trip");
        assert_eq!(gh_cred.secret().expose(), "gh-tok");

        let oai_cred =
            Credential::try_from(outputs.get("credential:openai").unwrap()).expect("round-trip");
        assert_eq!(oai_cred.secret().expose(), "oai-tok");
    }

    #[test]
    fn credential_op_from_inputs_bearer() {
        let var = "GUNBC_TEST_CRED_FROM_INPUTS_4821";
        std::env::set_var(var, "sk-test-from-inputs");

        let op = CredentialOp::from_inputs("credential:llm");
        let mut inputs = HashMap::new();
        inputs.insert("service".to_string(), Value::Str("openai".to_string()));
        inputs.insert("env_var".to_string(), Value::Str(var.to_string()));

        let outputs = op.execute(inputs).expect("should succeed");
        assert!(outputs.contains_key("credential:llm"));
        let cred =
            Credential::try_from(outputs.get("credential:llm").unwrap()).expect("round-trip");
        assert_eq!(cred.secret().expose(), "sk-test-from-inputs");
        assert!(matches!(cred.scheme(), AuthScheme::Bearer));

        std::env::remove_var(var);
    }

    #[test]
    fn credential_op_from_inputs_header_scheme() {
        let var = "GUNBC_TEST_CRED_HEADER_7392";
        std::env::set_var(var, "sk-ant-test-key");

        let op = CredentialOp::from_inputs("credential:llm");
        let mut inputs = HashMap::new();
        inputs.insert("service".to_string(), Value::Str("anthropic".to_string()));
        inputs.insert("env_var".to_string(), Value::Str(var.to_string()));
        inputs.insert("scheme".to_string(), Value::Str("header".to_string()));
        inputs.insert(
            "header_name".to_string(),
            Value::Str("x-api-key".to_string()),
        );

        let outputs = op.execute(inputs).expect("should succeed");
        let cred =
            Credential::try_from(outputs.get("credential:llm").unwrap()).expect("round-trip");
        assert_eq!(cred.secret().expose(), "sk-ant-test-key");
        assert!(matches!(
            cred.scheme(),
            AuthScheme::Header { ref name } if name == "x-api-key"
        ));

        std::env::remove_var(var);
    }

    #[test]
    fn credential_op_from_inputs_missing_env_var() {
        let var = "GUNBC_TEST_CRED_MISSING_9283";
        std::env::remove_var(var);

        let op = CredentialOp::from_inputs("credential:llm");
        let mut inputs = HashMap::new();
        inputs.insert("service".to_string(), Value::Str("openai".to_string()));
        inputs.insert("env_var".to_string(), Value::Str(var.to_string()));

        let err = op.execute(inputs).unwrap_err();
        assert!(err.0.contains("missing env var"));
    }

    #[test]
    fn credential_op_from_inputs_mock_outputs() {
        let op = CredentialOp::from_inputs("credential:llm");
        let mocks = op.mock_outputs();
        assert!(mocks.contains_key("credential:llm"));
        let cred = Credential::try_from(mocks.get("credential:llm").unwrap())
            .expect("mock should be valid Credential");
        assert_eq!(cred.secret().expose(), "mock-token");
    }

    #[test]
    fn credential_op_mock_outputs() {
        let p1 = Arc::new(MockCredentialProvider::new(
            "github",
            Credential::new(Secret::static_value("ignored"), AuthScheme::Bearer),
        ));
        let p2 = Arc::new(MockCredentialProvider::new(
            "openai",
            Credential::new(Secret::static_value("ignored"), AuthScheme::Bearer),
        ));

        let op = CredentialOp::new(vec![p1, p2]);
        let mocks = op.mock_outputs();

        assert!(mocks.contains_key("credential:github"));
        assert!(mocks.contains_key("credential:openai"));

        // DryRun credentials should be valid Value::Map with capability marker
        let gh = Credential::try_from(mocks.get("credential:github").unwrap())
            .expect("mock should be valid Credential");
        assert_eq!(gh.secret().expose(), "mock-token");
    }
}
