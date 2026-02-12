//! Discovery ops: prepare/parse pairs for GCP API calls that enumerate infrastructure.
//!
//! These ops use the typed service traits from `services/` to build REST requests
//! for discovering projects, WIF pools/providers, service accounts, secrets, and buckets.
//! The results are assembled into a `GcpInfraSpec`.

use gunbc_exec::{require_str, ExecError, Executable, OutputMap};
use gunbc_ir::transport::gcp::*;
use gunbc_ir::transport::rest::RestRequest;
use gunbc_ir::transport::TransportResponse;
use gunbc_ir::Value;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Write;

/// Discovery ops for enumerating GCP infrastructure resources.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GcpDiscoveryOps {
    // ----- Resource Manager -----
    /// Prepare list-projects request.
    PrepareListProjects,
    /// Parse list-projects response.
    ParseListProjects,

    // ----- Workload Identity -----
    /// Prepare list-WIF-pools request.
    PrepareListWifPools,
    /// Parse list-WIF-pools response.
    ParseListWifPools,
    /// Prepare list-WIF-providers request.
    PrepareListWifProviders,
    /// Parse list-WIF-providers response.
    ParseListWifProviders,

    // ----- IAM -----
    /// Prepare list-service-accounts request.
    PrepareListServiceAccounts,
    /// Parse list-service-accounts response.
    ParseListServiceAccounts,

    // ----- Secret Manager -----
    /// Prepare list-secrets request.
    PrepareListSecrets,
    /// Parse list-secrets response.
    ParseListSecrets,

    // ----- Storage -----
    /// Prepare list-buckets request.
    PrepareListBuckets,
    /// Parse list-buckets response.
    ParseListBuckets,

    // ----- IAM Policy -----
    /// Prepare get-project-IAM-policy request.
    PrepareGetIamPolicy,
    /// Parse get-project-IAM-policy response.
    ParseGetIamPolicy,

    // ----- Assembly -----
    /// Assemble discovered resources into a GcpInfraSpec.
    AssembleInfraSpec,
    /// Generate CloudConfigSpec TOML from InfraSpec.
    GenerateConfigSpec,
}

impl Executable for GcpDiscoveryOps {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            // =================================================================
            // Resource Manager: list projects
            // =================================================================
            GcpDiscoveryOps::PrepareListProjects => {
                let access_token = require_str(&inputs, "access_token")?;
                let req = RestRequest::get(
                    "https://cloudresourcemanager.googleapis.com/v1/projects",
                )
                .bearer(access_token);
                OutputMap::new()
                    .request("request", req.into())
                    .bool("skip", false)
                    .ok()
            }
            GcpDiscoveryOps::ParseListProjects => {
                let rest = extract_rest_response(&inputs)?;
                let projects_json = rest
                    .body
                    .get("projects")
                    .and_then(|v| v.as_array())
                    .cloned()
                    .unwrap_or_default();
                let projects: Vec<GcpProject> = projects_json
                    .iter()
                    .filter_map(|p| {
                        Some(GcpProject {
                            project_id: p.get("projectId")?.as_str()?.to_string(),
                            project_number: p
                                .get("projectNumber")
                                .and_then(|v| v.as_str())
                                .and_then(|s| s.parse().ok()),
                            display_name: p.get("name").and_then(|v| v.as_str()).map(String::from),
                            state: p
                                .get("lifecycleState")
                                .and_then(|v| v.as_str())
                                .map(String::from),
                            labels: parse_labels(p.get("labels")),
                        })
                    })
                    .collect();
                let json = serde_json::to_value(&projects)
                    .map_err(|e| ExecError::new(format!("serialize projects: {e}")))?;
                OutputMap::new().value("projects", Value::Json(json)).ok()
            }

            // =================================================================
            // WIF: list pools
            // =================================================================
            GcpDiscoveryOps::PrepareListWifPools => {
                let access_token = require_str(&inputs, "access_token")?;
                let project = require_str(&inputs, "project")?;
                let url = format!(
                    "https://iam.googleapis.com/v1/projects/{}/locations/global/workloadIdentityPools",
                    project
                );
                let req = RestRequest::get(url).bearer(access_token);
                OutputMap::new()
                    .request("request", req.into())
                    .bool("skip", false)
                    .ok()
            }
            GcpDiscoveryOps::ParseListWifPools => {
                let rest = extract_rest_response(&inputs)?;
                let pools_json = rest
                    .body
                    .get("workloadIdentityPools")
                    .and_then(|v| v.as_array())
                    .cloned()
                    .unwrap_or_default();
                let pools: Vec<GcpWifPool> = pools_json
                    .iter()
                    .filter_map(|p| {
                        let name = p.get("name")?.as_str()?.to_string();
                        let pool_id = name.rsplit('/').next()?.to_string();
                        Some(GcpWifPool {
                            name: name.clone(),
                            pool_id,
                            project_number: extract_project_number(&name).unwrap_or(0),
                            state: p
                                .get("state")
                                .and_then(|v| v.as_str())
                                .unwrap_or("ACTIVE")
                                .to_string(),
                            display_name: p
                                .get("displayName")
                                .and_then(|v| v.as_str())
                                .map(String::from),
                            description: p
                                .get("description")
                                .and_then(|v| v.as_str())
                                .map(String::from),
                        })
                    })
                    .collect();
                let json = serde_json::to_value(&pools)
                    .map_err(|e| ExecError::new(format!("serialize pools: {e}")))?;
                OutputMap::new().value("wif_pools", Value::Json(json)).ok()
            }

            // =================================================================
            // WIF: list providers
            // =================================================================
            GcpDiscoveryOps::PrepareListWifProviders => {
                let access_token = require_str(&inputs, "access_token")?;
                let pool_name = require_str(&inputs, "pool_name")?;
                let url = format!(
                    "https://iam.googleapis.com/v1/{}/providers",
                    pool_name
                );
                let req = RestRequest::get(url).bearer(access_token);
                OutputMap::new()
                    .request("request", req.into())
                    .bool("skip", false)
                    .ok()
            }
            GcpDiscoveryOps::ParseListWifProviders => {
                let rest = extract_rest_response(&inputs)?;
                let providers_json = rest
                    .body
                    .get("workloadIdentityPoolProviders")
                    .and_then(|v| v.as_array())
                    .cloned()
                    .unwrap_or_default();
                let providers: Vec<GcpWifProvider> = providers_json
                    .iter()
                    .filter_map(|p| {
                        let name = p.get("name")?.as_str()?.to_string();
                        let provider_id = name.rsplit('/').next()?.to_string();
                        Some(GcpWifProvider {
                            name: name.clone(),
                            provider_id,
                            display_name: p
                                .get("displayName")
                                .and_then(|v| v.as_str())
                                .map(String::from),
                            attribute_mappings: p
                                .get("attributeMapping")
                                .and_then(|v| {
                                    serde_json::from_value::<HashMap<String, String>>(v.clone()).ok()
                                })
                                .unwrap_or_default(),
                            attribute_condition: p
                                .get("attributeCondition")
                                .and_then(|v| v.as_str())
                                .map(String::from),
                            oidc_issuer_uri: p
                                .get("oidc")
                                .and_then(|o| o.get("issuerUri"))
                                .and_then(|v| v.as_str())
                                .map(String::from),
                        })
                    })
                    .collect();
                let json = serde_json::to_value(&providers)
                    .map_err(|e| ExecError::new(format!("serialize providers: {e}")))?;
                OutputMap::new()
                    .value("wif_providers", Value::Json(json))
                    .ok()
            }

            // =================================================================
            // IAM: list service accounts
            // =================================================================
            GcpDiscoveryOps::PrepareListServiceAccounts => {
                let access_token = require_str(&inputs, "access_token")?;
                let project = require_str(&inputs, "project")?;
                let url = format!(
                    "https://iam.googleapis.com/v1/projects/{}/serviceAccounts",
                    project
                );
                let req = RestRequest::get(url).bearer(access_token);
                OutputMap::new()
                    .request("request", req.into())
                    .bool("skip", false)
                    .ok()
            }
            GcpDiscoveryOps::ParseListServiceAccounts => {
                let rest = extract_rest_response(&inputs)?;
                let accounts_json = rest
                    .body
                    .get("accounts")
                    .and_then(|v| v.as_array())
                    .cloned()
                    .unwrap_or_default();
                let accounts: Vec<GcpServiceAccount> = accounts_json
                    .iter()
                    .filter_map(|a| {
                        Some(GcpServiceAccount {
                            email: a.get("email")?.as_str()?.to_string(),
                            project_id: a.get("projectId")?.as_str()?.to_string(),
                            display_name: a
                                .get("displayName")
                                .and_then(|v| v.as_str())
                                .map(String::from),
                            description: a
                                .get("description")
                                .and_then(|v| v.as_str())
                                .map(String::from),
                            unique_id: a
                                .get("uniqueId")
                                .and_then(|v| v.as_str())
                                .map(String::from),
                        })
                    })
                    .collect();
                let json = serde_json::to_value(&accounts)
                    .map_err(|e| ExecError::new(format!("serialize accounts: {e}")))?;
                OutputMap::new()
                    .value("service_accounts", Value::Json(json))
                    .ok()
            }

            // =================================================================
            // Secret Manager: list secrets
            // =================================================================
            GcpDiscoveryOps::PrepareListSecrets => {
                let access_token = require_str(&inputs, "access_token")?;
                let project = require_str(&inputs, "project")?;
                let url = format!(
                    "https://secretmanager.googleapis.com/v1/projects/{}/secrets",
                    project
                );
                let req = RestRequest::get(url).bearer(access_token);
                OutputMap::new()
                    .request("request", req.into())
                    .bool("skip", false)
                    .ok()
            }
            GcpDiscoveryOps::ParseListSecrets => {
                let rest = extract_rest_response(&inputs)?;
                let secrets_json = rest
                    .body
                    .get("secrets")
                    .and_then(|v| v.as_array())
                    .cloned()
                    .unwrap_or_default();
                let secrets: Vec<GcpSecret> = secrets_json
                    .iter()
                    .filter_map(|s| {
                        let name = s.get("name")?.as_str()?.to_string();
                        let secret_id = name.rsplit('/').next()?.to_string();
                        let project_id = extract_project_from_resource_name(&name)?;
                        Some(GcpSecret {
                            name: name.clone(),
                            secret_id,
                            project_id,
                            labels: parse_labels(s.get("labels")),
                            replication: s
                                .get("replication")
                                .map(|r| {
                                    if r.get("automatic").is_some() {
                                        "AUTOMATIC".to_string()
                                    } else {
                                        "USER_MANAGED".to_string()
                                    }
                                }),
                        })
                    })
                    .collect();
                let json = serde_json::to_value(&secrets)
                    .map_err(|e| ExecError::new(format!("serialize secrets: {e}")))?;
                OutputMap::new().value("secrets", Value::Json(json)).ok()
            }

            // =================================================================
            // Storage: list buckets
            // =================================================================
            GcpDiscoveryOps::PrepareListBuckets => {
                let access_token = require_str(&inputs, "access_token")?;
                let project = require_str(&inputs, "project")?;
                let req = RestRequest::get("https://storage.googleapis.com/storage/v1/b")
                    .query("project", project)
                    .bearer(access_token);
                OutputMap::new()
                    .request("request", req.into())
                    .bool("skip", false)
                    .ok()
            }
            GcpDiscoveryOps::ParseListBuckets => {
                let rest = extract_rest_response(&inputs)?;
                let items_json = rest
                    .body
                    .get("items")
                    .and_then(|v| v.as_array())
                    .cloned()
                    .unwrap_or_default();
                let buckets: Vec<GcpBucket> = items_json
                    .iter()
                    .filter_map(|b| {
                        Some(GcpBucket {
                            name: b.get("name")?.as_str()?.to_string(),
                            location: b
                                .get("location")
                                .and_then(|v| v.as_str())
                                .unwrap_or("US")
                                .to_string(),
                            storage_class: b
                                .get("storageClass")
                                .and_then(|v| v.as_str())
                                .map(String::from),
                            labels: parse_labels(b.get("labels")),
                            versioning_enabled: b
                                .get("versioning")
                                .and_then(|v| v.get("enabled"))
                                .and_then(|v| v.as_bool()),
                            uniform_bucket_level_access: b
                                .get("iamConfiguration")
                                .and_then(|c| c.get("uniformBucketLevelAccess"))
                                .and_then(|u| u.get("enabled"))
                                .and_then(|v| v.as_bool()),
                        })
                    })
                    .collect();
                let json = serde_json::to_value(&buckets)
                    .map_err(|e| ExecError::new(format!("serialize buckets: {e}")))?;
                OutputMap::new().value("buckets", Value::Json(json)).ok()
            }

            // =================================================================
            // IAM policy: get project IAM policy
            // =================================================================
            GcpDiscoveryOps::PrepareGetIamPolicy => {
                let access_token = require_str(&inputs, "access_token")?;
                let project = require_str(&inputs, "project")?;
                let url = format!(
                    "https://cloudresourcemanager.googleapis.com/v1/projects/{}:getIamPolicy",
                    project
                );
                let req = RestRequest::post(url)
                    .json(serde_json::json!({}))
                    .bearer(access_token);
                OutputMap::new()
                    .request("request", req.into())
                    .bool("skip", false)
                    .ok()
            }
            GcpDiscoveryOps::ParseGetIamPolicy => {
                let rest = extract_rest_response(&inputs)?;
                let project = require_str(&inputs, "project")?;
                let bindings_json = rest
                    .body
                    .get("bindings")
                    .and_then(|v| v.as_array())
                    .cloned()
                    .unwrap_or_default();

                let mut bindings = Vec::new();
                for b in &bindings_json {
                    let role = match b.get("role").and_then(|v| v.as_str()) {
                        Some(r) => r.to_string(),
                        None => continue,
                    };
                    let members = b
                        .get("members")
                        .and_then(|v| v.as_array())
                        .cloned()
                        .unwrap_or_default();
                    for m in &members {
                        if let Some(member) = m.as_str() {
                            bindings.push(GcpIamBinding {
                                role: role.clone(),
                                member: member.to_string(),
                                condition: b.get("condition").and_then(|c| {
                                    Some(GcpIamCondition {
                                        title: c.get("title")?.as_str()?.to_string(),
                                        expression: c.get("expression")?.as_str()?.to_string(),
                                        description: c
                                            .get("description")
                                            .and_then(|v| v.as_str())
                                            .map(String::from),
                                    })
                                }),
                            });
                        }
                    }
                }

                let policy = GcpIamPolicy {
                    resource: format!("projects/{}", project),
                    bindings,
                    etag: rest
                        .body
                        .get("etag")
                        .and_then(|v| v.as_str())
                        .map(String::from),
                };
                let json = serde_json::to_value(vec![policy])
                    .map_err(|e| ExecError::new(format!("serialize policy: {e}")))?;
                OutputMap::new()
                    .value("iam_policies", Value::Json(json))
                    .ok()
            }

            // =================================================================
            // Assembly: combine discovered resources into InfraSpec
            // =================================================================
            GcpDiscoveryOps::AssembleInfraSpec => {
                let project = require_str(&inputs, "project")?;
                let projects: Vec<GcpProject> = extract_json_list(&inputs, "projects")?;
                let wif_pools: Vec<GcpWifPool> = extract_json_list(&inputs, "wif_pools")?;
                let wif_providers: Vec<GcpWifProvider> =
                    extract_json_list(&inputs, "wif_providers")?;
                let service_accounts: Vec<GcpServiceAccount> =
                    extract_json_list(&inputs, "service_accounts")?;
                let secrets: Vec<GcpSecret> = extract_json_list(&inputs, "secrets")?;
                let buckets: Vec<GcpBucket> = extract_json_list(&inputs, "buckets")?;
                let iam_policies: Vec<GcpIamPolicy> =
                    extract_json_list(&inputs, "iam_policies")?;

                let spec = GcpInfraSpec {
                    projects,
                    wif_pools,
                    wif_providers,
                    service_accounts,
                    secrets,
                    buckets,
                    iam_policies,
                    discovered_at: chrono_now_iso8601(),
                    source_project: project.to_string(),
                };

                let json = serde_json::to_value(&spec)
                    .map_err(|e| ExecError::new(format!("serialize infra spec: {e}")))?;
                OutputMap::new()
                    .value("infra_spec", Value::Json(json))
                    .ok()
            }

            // =================================================================
            // Config generation: InfraSpec -> CloudConfigSpec TOML
            // =================================================================
            GcpDiscoveryOps::GenerateConfigSpec => {
                let spec_json = inputs
                    .get("infra_spec")
                    .ok_or_else(|| ExecError::new("missing infra_spec input"))?;
                let spec: GcpInfraSpec = match spec_json {
                    Value::Json(j) => serde_json::from_value(j.clone())
                        .map_err(|e| ExecError::new(format!("deserialize infra spec: {e}")))?,
                    _ => return Err(ExecError::new("infra_spec must be Json")),
                };

                let config = generate_config_from_infra(&spec);
                let toml_str = toml_serialize(&config)
                    .map_err(|e| ExecError::new(format!("TOML serialize: {e}")))?;

                OutputMap::new()
                    .str("config_toml", &toml_str)
                    .value(
                        "config_spec",
                        Value::Json(
                            serde_json::to_value(&config)
                                .map_err(|e| ExecError::new(format!("serialize config: {e}")))?,
                        ),
                    )
                    .ok()
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn extract_rest_response(
    inputs: &HashMap<String, Value>,
) -> Result<&gunbc_ir::transport::rest::RestResponse, ExecError> {
    let response = match inputs.get("response") {
        Some(Value::Skipped) => {
            return Err(ExecError::new("response was skipped"));
        }
        Some(Value::Response(r)) => r,
        _ => return Err(ExecError::new("missing or invalid 'response' input")),
    };
    match response {
        TransportResponse::Rest(r) => Ok(r),
        other => Err(ExecError::new(format!(
            "expected REST response, got {:?}",
            other
        ))),
    }
}

fn extract_json_list<T: serde::de::DeserializeOwned>(
    inputs: &HashMap<String, Value>,
    key: &str,
) -> Result<Vec<T>, ExecError> {
    match inputs.get(key) {
        Some(Value::Json(j)) => serde_json::from_value(j.clone())
            .map_err(|e| ExecError::new(format!("deserialize {key}: {e}"))),
        _ => Ok(Vec::new()),
    }
}

fn parse_labels(value: Option<&serde_json::Value>) -> HashMap<String, String> {
    value
        .and_then(|v| serde_json::from_value::<HashMap<String, String>>(v.clone()).ok())
        .unwrap_or_default()
}

fn extract_project_number(resource_name: &str) -> Option<i64> {
    // "projects/{number}/locations/global/..."
    let mut parts = resource_name.split('/');
    if parts.next() == Some("projects") {
        parts.next().and_then(|s| s.parse().ok())
    } else {
        None
    }
}

fn extract_project_from_resource_name(name: &str) -> Option<String> {
    // "projects/{project}/secrets/{secret}"
    let parts: Vec<&str> = name.split('/').collect();
    if parts.len() >= 2 && parts[0] == "projects" {
        Some(parts[1].to_string())
    } else {
        None
    }
}

fn chrono_now_iso8601() -> String {
    // Simple ISO 8601 timestamp without chrono dependency
    // This is a placeholder; in production, this would use proper time handling
    "2026-02-08T00:00:00Z".to_string()
}

/// Generate a CloudConfigSpec from discovered infrastructure.
///
/// Derives namespaces from secret prefix patterns and WIF configurations.
fn generate_config_from_infra(
    spec: &GcpInfraSpec,
) -> gunbc_ir::transport::cloud::CloudConfigSpec {
    use gunbc_ir::transport::cloud::{CloudConfigSpec, CloudNamespace, CloudProviderKind};

    // Find unique prefixes from secret names
    let prefixes: Vec<String> = {
        let mut set = std::collections::BTreeSet::new();
        for s in &spec.secrets {
            let mut parts = s.secret_id.splitn(2, '-');
            if let (Some(prefix), Some(_)) = (parts.next(), parts.next()) {
                set.insert(prefix.to_string());
            }
        }
        set.into_iter().collect()
    };

    // Find the first WIF provider audience
    let wif_provider = spec.wif_providers.first().map(|p| p.name.clone());

    // Find service accounts matching secret-accessor patterns
    let sa_email = spec
        .service_accounts
        .iter()
        .find(|sa| sa.email.contains("secrets"))
        .map(|sa| sa.email.clone());

    // Build base namespace
    let mut namespaces = vec![CloudNamespace {
        name: "base".to_string(),
        inherits_from: None,
        provider: Some(CloudProviderKind::Gcp),
        secrets_project: Some(spec.source_project.clone()),
        wif_provider,
        service_account: sa_email,
        impersonate_account: None,
    }];

    // Build per-prefix namespaces
    for prefix in &prefixes {
        namespaces.push(CloudNamespace {
            name: prefix.clone(),
            inherits_from: Some("base".to_string()),
            provider: None,
            secrets_project: None,
            wif_provider: None,
            service_account: None,
            impersonate_account: None,
        });
    }

    CloudConfigSpec {
        namespaces,
        default_namespace: prefixes.first().cloned(),
        generated_at: Some(spec.discovered_at.clone()),
        source_project: Some(spec.source_project.clone()),
    }
}

/// Simple TOML serializer for CloudConfigSpec.
///
/// Avoids pulling in a full TOML crate by doing manual formatting.
fn toml_serialize(
    config: &gunbc_ir::transport::cloud::CloudConfigSpec,
) -> Result<String, String> {
    let mut out = String::new();

    if let Some(ref ns) = config.default_namespace {
        write!(out, "default_namespace = \"{}\"\n", ns).unwrap();
    }
    if let Some(ref at) = config.generated_at {
        write!(out, "generated_at = \"{}\"\n", at).unwrap();
    }
    if let Some(ref proj) = config.source_project {
        write!(out, "source_project = \"{}\"\n", proj).unwrap();
    }
    out.push('\n');

    for ns in &config.namespaces {
        out.push_str("[[namespaces]]\n");
        write!(out, "name = \"{}\"\n", ns.name).unwrap();
        if let Some(ref inherits) = ns.inherits_from {
            write!(out, "inherits_from = \"{}\"\n", inherits).unwrap();
        }
        if let Some(ref provider) = ns.provider {
            write!(out, "provider = \"{}\"\n", provider.as_str()).unwrap();
        }
        if let Some(ref proj) = ns.secrets_project {
            write!(out, "secrets_project = \"{}\"\n", proj).unwrap();
        }
        if let Some(ref wif) = ns.wif_provider {
            write!(out, "wif_provider = \"{}\"\n", wif).unwrap();
        }
        if let Some(ref sa) = ns.service_account {
            write!(out, "service_account = \"{}\"\n", sa).unwrap();
        }
        if let Some(ref imp) = ns.impersonate_account {
            write!(out, "impersonate_account = \"{}\"\n", imp).unwrap();
        }
        out.push('\n');
    }

    Ok(out)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::transport::rest::RestResponse;

    #[test]
    fn test_parse_list_projects() {
        let mut inputs = HashMap::new();
        inputs.insert(
            "response".to_string(),
            Value::Response(TransportResponse::Rest(RestResponse::ok(serde_json::json!({
                "projects": [
                    {
                        "projectId": "gunbai-secrets",
                        "projectNumber": "314501921854",
                        "name": "gunbai-secrets",
                        "lifecycleState": "ACTIVE"
                    }
                ]
            })))),
        );
        let outputs = GcpDiscoveryOps::ParseListProjects.execute(inputs).unwrap();
        let projects: Vec<GcpProject> =
            serde_json::from_value(outputs["projects"].as_json().unwrap().clone()).unwrap();
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].project_id, "gunbai-secrets");
    }

    #[test]
    fn test_parse_list_secrets() {
        let mut inputs = HashMap::new();
        inputs.insert(
            "response".to_string(),
            Value::Response(TransportResponse::Rest(RestResponse::ok(serde_json::json!({
                "secrets": [
                    {
                        "name": "projects/gunbai-secrets/secrets/dev-github-token",
                        "replication": { "automatic": {} }
                    },
                    {
                        "name": "projects/gunbai-secrets/secrets/prod-api-key",
                        "labels": { "env": "prod" }
                    }
                ]
            })))),
        );
        let outputs = GcpDiscoveryOps::ParseListSecrets.execute(inputs).unwrap();
        let secrets: Vec<GcpSecret> =
            serde_json::from_value(outputs["secrets"].as_json().unwrap().clone()).unwrap();
        assert_eq!(secrets.len(), 2);
        assert_eq!(secrets[0].secret_id, "dev-github-token");
        assert_eq!(secrets[0].project_id, "gunbai-secrets");
    }

    #[test]
    fn test_generate_config_spec_produces_toml() {
        let spec = GcpInfraSpec {
            projects: vec![GcpProject {
                project_id: "gunbai-secrets".to_string(),
                project_number: Some(314501921854),
                display_name: None,
                state: None,
                labels: HashMap::new(),
            }],
            wif_pools: vec![],
            wif_providers: vec![GcpWifProvider {
                name: "projects/314501921854/locations/global/workloadIdentityPools/github-pool/providers/github".to_string(),
                provider_id: "github".to_string(),
                display_name: None,
                attribute_mappings: HashMap::new(),
                attribute_condition: None,
                oidc_issuer_uri: None,
            }],
            service_accounts: vec![GcpServiceAccount {
                email: "gunbai-dev-secrets@gunbai-secrets.iam.gserviceaccount.com".to_string(),
                project_id: "gunbai-secrets".to_string(),
                display_name: None,
                description: None,
                unique_id: None,
            }],
            secrets: vec![
                GcpSecret {
                    name: "projects/gunbai-secrets/secrets/dev-github-token".to_string(),
                    secret_id: "dev-github-token".to_string(),
                    project_id: "gunbai-secrets".to_string(),
                    labels: HashMap::new(),
                    replication: None,
                },
                GcpSecret {
                    name: "projects/gunbai-secrets/secrets/prod-api-key".to_string(),
                    secret_id: "prod-api-key".to_string(),
                    project_id: "gunbai-secrets".to_string(),
                    labels: HashMap::new(),
                    replication: None,
                },
            ],
            buckets: vec![],
            iam_policies: vec![],
            discovered_at: "2026-02-08T00:00:00Z".to_string(),
            source_project: "gunbai-secrets".to_string(),
        };

        let mut inputs = HashMap::new();
        inputs.insert(
            "infra_spec".to_string(),
            Value::Json(serde_json::to_value(&spec).unwrap()),
        );
        let outputs = GcpDiscoveryOps::GenerateConfigSpec.execute(inputs).unwrap();
        let toml = outputs["config_toml"].as_str().unwrap();
        assert!(toml.contains("gunbai-secrets"));
        assert!(toml.contains("[[namespaces]]"));
        assert!(toml.contains("name = \"dev\""));
        assert!(toml.contains("name = \"prod\""));
    }
}
