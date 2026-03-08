//! Multi-project registry and cross-project WIF binding derivation.

use crate::project_spec::{
    GcpProject, NamespaceSpec, ProjectSpec, ServiceAccountBinding, GUNBAI_SECRETS,
};
use std::collections::BTreeMap;
use std::sync::LazyLock;

/// Cross-project WIF binding recommendation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrossProjectWifBinding {
    pub source_project_key: String,
    pub target_project_key: String,
    pub namespace: String,
    pub binding: ServiceAccountBinding,
}

/// Project registry for infra orchestration across multiple ProjectSpecs.
#[derive(Debug, Clone, Default)]
pub struct ProjectRegistry {
    entries: BTreeMap<&'static str, &'static ProjectSpec>,
}

impl ProjectRegistry {
    pub fn from_entries(entries: BTreeMap<&'static str, &'static ProjectSpec>) -> Self {
        Self { entries }
    }

    pub fn default_registry() -> Self {
        let mut entries = BTreeMap::new();
        entries.insert("secrets", &GUNBAI_SECRETS);
        entries.insert("platform", &*GUNBAI_PLATFORM);
        Self { entries }
    }

    pub fn get(&self, key: &str) -> Option<&'static ProjectSpec> {
        self.entries.get(key).copied()
    }

    pub fn keys(&self) -> Vec<&'static str> {
        self.entries.keys().copied().collect()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&'static str, &'static ProjectSpec)> + '_ {
        self.entries.iter().map(|(k, v)| (*k, *v))
    }
}

/// Secondary project spec to exercise multi-project orchestration paths.
pub static GUNBAI_PLATFORM: LazyLock<ProjectSpec> = LazyLock::new(|| {
    let namespaces =
        clone_namespaces_for_project(GUNBAI_SECRETS.namespaces, "gunbai-platform", "314501921854");
    ProjectSpec {
        secrets_project: GcpProject {
            project_id: "gunbai-platform",
            project_number: 314501921854,
        },
        wif: GUNBAI_SECRETS.wif.clone(),
        namespaces,
        secrets: GUNBAI_SECRETS.secrets,
    }
});

/// Derive cross-project WIF bindings between registered specs.
pub fn derive_cross_project_wif_bindings(
    registry: &ProjectRegistry,
) -> Vec<CrossProjectWifBinding> {
    let mut bindings = Vec::new();
    for (source_key, source_spec) in registry.iter() {
        for (target_key, target_spec) in registry.iter() {
            if source_key == target_key {
                continue;
            }
            for source_ns in source_spec.namespaces {
                if let Some(target_ns) = target_spec.namespace(source_ns.name) {
                    let sa_email = target_ns.service_account_email();
                    let binding = ServiceAccountBinding {
                        role: "roles/iam.workloadIdentityUser".to_string(),
                        members: source_ns
                            .secrets_service_account
                            .wif_bindings
                            .iter()
                            .map(|m| m.to_string())
                            .collect(),
                    };
                    if !binding.members.is_empty() {
                        bindings.push(CrossProjectWifBinding {
                            source_project_key: source_key.to_string(),
                            target_project_key: target_key.to_string(),
                            namespace: source_ns.name.to_string(),
                            binding: ServiceAccountBinding {
                                role: binding.role,
                                members: vec![format!("serviceAccount:{sa_email}")]
                                    .into_iter()
                                    .chain(binding.members)
                                    .collect(),
                            },
                        });
                    }
                }
            }
        }
    }
    bindings
}

fn clone_namespaces_for_project(
    base: &'static [NamespaceSpec],
    project_id: &'static str,
    project_number: &'static str,
) -> &'static [NamespaceSpec] {
    let mut cloned = Vec::with_capacity(base.len());
    for ns in base {
        let mut copied = ns.clone();
        copied.project = project_id;
        copied.project_number = project_number;
        copied.secrets_project = project_id;
        cloned.push(copied);
    }
    Box::leak(cloned.into_boxed_slice())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_registry_contains_multiple_project_specs() {
        let registry = ProjectRegistry::default_registry();
        let keys = registry.keys();
        assert!(keys.contains(&"secrets"));
        assert!(keys.contains(&"platform"));
        assert!(registry.get("secrets").is_some());
        assert!(registry.get("platform").is_some());
    }

    #[test]
    fn cross_project_bindings_include_wif_membership_edges() {
        let registry = ProjectRegistry::default_registry();
        let bindings = derive_cross_project_wif_bindings(&registry);
        assert!(
            !bindings.is_empty(),
            "cross-project binding list should not be empty"
        );
        assert!(
            bindings
                .iter()
                .any(|b| b.source_project_key != b.target_project_key),
            "bindings should include source->target cross-project pairs"
        );
        assert!(
            bindings
                .iter()
                .all(|b| b.binding.role == "roles/iam.workloadIdentityUser"),
            "bindings should use workloadIdentityUser role"
        );
    }
}
