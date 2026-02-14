//! Credential policy schema (v0) and profile inheritance resolution.
//!
//! This models policy-driven auth selection without tying callers to provider
//! mechanics. The schema is intentionally provider-neutral and supports
//! inherited profiles (`base -> dev/prod`) with per-intent overrides.

use super::cloud::{CloudProviderKind, CloudRuntimeKind};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Top-level credential policy document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CredentialPolicySpec {
    /// Schema version (starts at 0 during rollout).
    #[serde(default)]
    pub version: u32,
    /// Optional default profile when the caller does not specify one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_profile: Option<String>,
    /// Named profiles (e.g., base/dev/prod/local).
    #[serde(default)]
    pub profiles: Vec<CredentialPolicyProfile>,
}

impl CredentialPolicySpec {
    /// Look up a profile by name.
    pub fn profile(&self, name: &str) -> Option<&CredentialPolicyProfile> {
        self.profiles.iter().find(|p| p.name == name)
    }

    /// Validate profile and intent uniqueness.
    pub fn validate(&self) -> Result<(), CredentialPolicyError> {
        let mut seen_profiles = BTreeSet::new();
        for profile in &self.profiles {
            if !seen_profiles.insert(profile.name.clone()) {
                return Err(CredentialPolicyError::DuplicateProfile(
                    profile.name.clone(),
                ));
            }

            let mut seen_intents = BTreeSet::new();
            for intent in &profile.intents {
                if !seen_intents.insert(intent.intent.clone()) {
                    return Err(CredentialPolicyError::DuplicateIntent {
                        profile: profile.name.clone(),
                        intent: intent.intent.clone(),
                    });
                }
            }
        }
        Ok(())
    }

    /// Resolve a profile with inherited defaults and intent policies.
    pub fn resolve_profile(
        &self,
        name: &str,
    ) -> Result<ResolvedCredentialPolicyProfile, CredentialPolicyError> {
        self.validate()?;
        self.resolve_profile_inner(name, &mut Vec::new())
    }

    /// Resolve one intent policy for a given profile.
    pub fn resolve_intent_policy(
        &self,
        profile_name: &str,
        intent_key: &str,
    ) -> Result<ResolvedCredentialIntentPolicy, CredentialPolicyError> {
        let resolved = self.resolve_profile(profile_name)?;
        let mut policy = resolved.intents.get(intent_key).cloned().ok_or_else(|| {
            CredentialPolicyError::UnknownIntent {
                profile: profile_name.to_string(),
                intent: intent_key.to_string(),
            }
        })?;

        if policy.provider.is_none() {
            policy.provider = resolved.defaults.provider;
        }
        if policy.runtime.is_none() {
            policy.runtime = resolved.defaults.runtime;
        }
        if policy.version_selector.is_none() {
            policy.version_selector = resolved.defaults.version_selector.clone();
        }
        if policy.impersonation.is_none() {
            policy.impersonation = resolved.defaults.impersonation.clone();
        }

        if policy.secret.is_none() {
            return Err(CredentialPolicyError::MissingSecretBinding {
                profile: profile_name.to_string(),
                intent: intent_key.to_string(),
            });
        }

        if policy.version_selector.is_none() {
            policy.version_selector = Some(VersionSelector::Latest);
        }

        if policy.impersonation.is_none() {
            policy.impersonation = Some(ImpersonationPolicy::default());
        }

        Ok(policy)
    }

    fn resolve_profile_inner(
        &self,
        name: &str,
        stack: &mut Vec<String>,
    ) -> Result<ResolvedCredentialPolicyProfile, CredentialPolicyError> {
        if stack.iter().any(|n| n == name) {
            let mut cycle = stack.clone();
            cycle.push(name.to_string());
            return Err(CredentialPolicyError::InheritanceCycle(cycle));
        }

        let profile = self
            .profile(name)
            .ok_or_else(|| CredentialPolicyError::UnknownProfile(name.to_string()))?;

        stack.push(name.to_string());

        let mut resolved = if let Some(parent) = &profile.inherits_from {
            self.resolve_profile_inner(parent, stack)?
        } else {
            ResolvedCredentialPolicyProfile::new(profile.name.clone())
        };

        resolved.name = profile.name.clone();
        resolved.defaults = resolved.defaults.merged_with(&profile.defaults);

        for intent in &profile.intents {
            let merged = match resolved.intents.get(&intent.intent) {
                Some(parent_intent) => {
                    merge_intent_policy(parent_intent, intent, &resolved.defaults.scope_merge)
                }
                None => merge_intent_policy(
                    &ResolvedCredentialIntentPolicy::new(intent.intent.clone()),
                    intent,
                    &resolved.defaults.scope_merge,
                ),
            };
            resolved.intents.insert(intent.intent.clone(), merged);
        }

        let _ = stack.pop();
        Ok(resolved)
    }
}

/// One policy profile.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CredentialPolicyProfile {
    /// Profile name (e.g., base, local, dev, prod).
    pub name: String,
    /// Optional parent profile name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inherits_from: Option<String>,
    /// Profile-wide defaults.
    #[serde(default)]
    pub defaults: CredentialPolicyDefaults,
    /// Per-intent policy entries.
    #[serde(default)]
    pub intents: Vec<CredentialIntentPolicy>,
}

/// Profile-wide defaults applied to all intents unless overridden.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct CredentialPolicyDefaults {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<CloudProviderKind>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime: Option<CloudRuntimeKind>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub impersonation: Option<ImpersonationPolicy>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version_selector: Option<VersionSelector>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope_merge: Option<ScopeMergeMode>,
}

impl CredentialPolicyDefaults {
    fn merged_with(&self, child: &CredentialPolicyDefaults) -> CredentialPolicyDefaults {
        CredentialPolicyDefaults {
            provider: child.provider.or(self.provider),
            runtime: child.runtime.or(self.runtime),
            impersonation: child
                .impersonation
                .clone()
                .or_else(|| self.impersonation.clone()),
            version_selector: child
                .version_selector
                .clone()
                .or_else(|| self.version_selector.clone()),
            scope_merge: child.scope_merge.or(self.scope_merge),
        }
    }
}

/// Intent-level policy entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CredentialIntentPolicy {
    /// Stable intent key (example: "github.gist.create").
    pub intent: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<CloudProviderKind>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime: Option<CloudRuntimeKind>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret: Option<SecretBinding>,
    /// `None` means inherit parent/default scopes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_scopes: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope_merge: Option<ScopeMergeMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub impersonation: Option<ImpersonationPolicy>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version_selector: Option<VersionSelector>,
}

/// How scope lists merge during profile inheritance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum ScopeMergeMode {
    /// Child replaces parent scopes when `required_scopes` is present.
    #[default]
    Replace,
    /// Child scopes are unioned with parent scopes.
    Union,
}

/// Secret binding reference used by an intent policy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecretBinding {
    /// Logical secret name/id (without profile prefixing assumptions).
    pub name: String,
    /// Optional explicit prefix override.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,
    /// Optional delimiter override.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delimiter: Option<String>,
}

/// Policy-driven impersonation behavior.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum ImpersonationPolicy {
    Never,
    IfConfigured {
        #[serde(skip_serializing_if = "Option::is_none")]
        service_account: Option<String>,
    },
    Always {
        service_account: String,
    },
}

impl Default for ImpersonationPolicy {
    fn default() -> Self {
        Self::IfConfigured {
            service_account: None,
        }
    }
}

/// Secret version selector policy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum VersionSelector {
    Latest,
    Alias { name: String },
    Fixed { version: String },
}

/// Resolved profile with merged defaults/intents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedCredentialPolicyProfile {
    pub name: String,
    pub defaults: CredentialPolicyDefaults,
    pub intents: BTreeMap<String, ResolvedCredentialIntentPolicy>,
}

impl ResolvedCredentialPolicyProfile {
    fn new(name: String) -> Self {
        Self {
            name,
            defaults: CredentialPolicyDefaults::default(),
            intents: BTreeMap::new(),
        }
    }
}

/// Resolved intent policy after inheritance merge.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedCredentialIntentPolicy {
    pub intent: String,
    pub provider: Option<CloudProviderKind>,
    pub runtime: Option<CloudRuntimeKind>,
    pub secret: Option<SecretBinding>,
    pub required_scopes: Vec<String>,
    pub impersonation: Option<ImpersonationPolicy>,
    pub version_selector: Option<VersionSelector>,
}

impl ResolvedCredentialIntentPolicy {
    fn new(intent: String) -> Self {
        Self {
            intent,
            provider: None,
            runtime: None,
            secret: None,
            required_scopes: Vec::new(),
            impersonation: None,
            version_selector: None,
        }
    }
}

fn merge_intent_policy(
    parent: &ResolvedCredentialIntentPolicy,
    child: &CredentialIntentPolicy,
    default_scope_mode: &Option<ScopeMergeMode>,
) -> ResolvedCredentialIntentPolicy {
    let scope_mode = child
        .scope_merge
        .or(*default_scope_mode)
        .unwrap_or(ScopeMergeMode::Replace);
    let required_scopes = match (&child.required_scopes, scope_mode) {
        (Some(scopes), ScopeMergeMode::Replace) => scopes.clone(),
        (Some(scopes), ScopeMergeMode::Union) => {
            let mut set = BTreeSet::new();
            for s in &parent.required_scopes {
                set.insert(s.clone());
            }
            for s in scopes {
                set.insert(s.clone());
            }
            set.into_iter().collect()
        }
        (None, _) => parent.required_scopes.clone(),
    };

    ResolvedCredentialIntentPolicy {
        intent: child.intent.clone(),
        provider: child.provider.or(parent.provider),
        runtime: child.runtime.or(parent.runtime),
        secret: child.secret.clone().or_else(|| parent.secret.clone()),
        required_scopes,
        impersonation: child
            .impersonation
            .clone()
            .or_else(|| parent.impersonation.clone()),
        version_selector: child
            .version_selector
            .clone()
            .or_else(|| parent.version_selector.clone()),
    }
}

/// Policy resolution/validation errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CredentialPolicyError {
    UnknownProfile(String),
    UnknownIntent { profile: String, intent: String },
    DuplicateProfile(String),
    DuplicateIntent { profile: String, intent: String },
    InheritanceCycle(Vec<String>),
    MissingSecretBinding { profile: String, intent: String },
}

impl std::fmt::Display for CredentialPolicyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CredentialPolicyError::UnknownProfile(profile) => {
                write!(f, "unknown credential policy profile '{profile}'")
            }
            CredentialPolicyError::UnknownIntent { profile, intent } => {
                write!(f, "unknown intent policy '{intent}' in profile '{profile}'")
            }
            CredentialPolicyError::DuplicateProfile(profile) => {
                write!(f, "duplicate credential policy profile '{profile}'")
            }
            CredentialPolicyError::DuplicateIntent { profile, intent } => {
                write!(
                    f,
                    "duplicate intent policy '{intent}' in profile '{profile}'"
                )
            }
            CredentialPolicyError::InheritanceCycle(cycle) => {
                write!(
                    f,
                    "credential policy inheritance cycle: {}",
                    cycle.join(" -> ")
                )
            }
            CredentialPolicyError::MissingSecretBinding { profile, intent } => {
                write!(
                    f,
                    "missing secret binding for intent '{intent}' in profile '{profile}'"
                )
            }
        }
    }
}

impl std::error::Error for CredentialPolicyError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inherited_profile_merges_defaults_and_intents() {
        let spec = CredentialPolicySpec {
            version: 0,
            default_profile: Some("dev".to_string()),
            profiles: vec![
                CredentialPolicyProfile {
                    name: "base".to_string(),
                    inherits_from: None,
                    defaults: CredentialPolicyDefaults {
                        provider: Some(CloudProviderKind::Gcp),
                        runtime: Some(CloudRuntimeKind::LocalDev),
                        impersonation: Some(ImpersonationPolicy::IfConfigured {
                            service_account: Some(
                                "base-sa@example.iam.gserviceaccount.com".to_string(),
                            ),
                        }),
                        version_selector: Some(VersionSelector::Alias {
                            name: "active".to_string(),
                        }),
                        scope_merge: Some(ScopeMergeMode::Replace),
                    },
                    intents: vec![CredentialIntentPolicy {
                        intent: "github.gist.create".to_string(),
                        provider: None,
                        runtime: None,
                        secret: Some(SecretBinding {
                            name: "github-token".to_string(),
                            prefix: Some("dev-".to_string()),
                            delimiter: Some(String::new()),
                        }),
                        required_scopes: Some(vec!["gist:write".to_string()]),
                        scope_merge: None,
                        impersonation: None,
                        version_selector: None,
                    }],
                },
                CredentialPolicyProfile {
                    name: "dev".to_string(),
                    inherits_from: Some("base".to_string()),
                    defaults: CredentialPolicyDefaults {
                        provider: None,
                        runtime: Some(CloudRuntimeKind::GitHubActions),
                        impersonation: None,
                        version_selector: Some(VersionSelector::Alias {
                            name: "dev-active".to_string(),
                        }),
                        scope_merge: None,
                    },
                    intents: vec![CredentialIntentPolicy {
                        intent: "github.gist.create".to_string(),
                        provider: None,
                        runtime: None,
                        secret: None,
                        required_scopes: Some(vec!["gist:write".to_string(), "repo".to_string()]),
                        scope_merge: Some(ScopeMergeMode::Union),
                        impersonation: Some(ImpersonationPolicy::Always {
                            service_account: "dev-secrets@example.iam.gserviceaccount.com"
                                .to_string(),
                        }),
                        version_selector: None,
                    }],
                },
            ],
        };

        let resolved = spec
            .resolve_intent_policy("dev", "github.gist.create")
            .expect("intent policy should resolve");

        assert_eq!(resolved.provider, Some(CloudProviderKind::Gcp));
        assert_eq!(resolved.runtime, Some(CloudRuntimeKind::GitHubActions));
        assert_eq!(
            resolved.version_selector,
            Some(VersionSelector::Alias {
                name: "dev-active".to_string()
            })
        );
        assert!(resolved.required_scopes.contains(&"gist:write".to_string()));
        assert!(resolved.required_scopes.contains(&"repo".to_string()));
        assert_eq!(
            resolved.impersonation,
            Some(ImpersonationPolicy::Always {
                service_account: "dev-secrets@example.iam.gserviceaccount.com".to_string(),
            })
        );
    }

    #[test]
    fn detects_inheritance_cycle() {
        let spec = CredentialPolicySpec {
            version: 0,
            default_profile: None,
            profiles: vec![
                CredentialPolicyProfile {
                    name: "a".to_string(),
                    inherits_from: Some("b".to_string()),
                    defaults: CredentialPolicyDefaults::default(),
                    intents: vec![],
                },
                CredentialPolicyProfile {
                    name: "b".to_string(),
                    inherits_from: Some("a".to_string()),
                    defaults: CredentialPolicyDefaults::default(),
                    intents: vec![],
                },
            ],
        };

        let err = spec.resolve_profile("a").expect_err("should detect cycle");
        assert!(matches!(err, CredentialPolicyError::InheritanceCycle(_)));
    }

    #[test]
    fn resolve_intent_requires_secret_binding() {
        let spec = CredentialPolicySpec {
            version: 0,
            default_profile: None,
            profiles: vec![CredentialPolicyProfile {
                name: "dev".to_string(),
                inherits_from: None,
                defaults: CredentialPolicyDefaults::default(),
                intents: vec![CredentialIntentPolicy {
                    intent: "llm.chat.openai".to_string(),
                    provider: Some(CloudProviderKind::Gcp),
                    runtime: Some(CloudRuntimeKind::LocalDev),
                    secret: None,
                    required_scopes: Some(vec!["llm:chat_completion".to_string()]),
                    scope_merge: None,
                    impersonation: None,
                    version_selector: Some(VersionSelector::Alias {
                        name: "active".to_string(),
                    }),
                }],
            }],
        };

        let err = spec
            .resolve_intent_policy("dev", "llm.chat.openai")
            .expect_err("missing secret must fail");

        assert!(matches!(
            err,
            CredentialPolicyError::MissingSecretBinding { .. }
        ));
    }

    #[test]
    fn child_explicit_replace_overrides_parent_union() {
        // Parent sets scope_merge = Union at defaults level.
        // Child explicitly sets scope_merge = Replace.
        // The child's Replace must win, not be ignored.
        let spec = CredentialPolicySpec {
            version: 0,
            default_profile: None,
            profiles: vec![
                CredentialPolicyProfile {
                    name: "base".to_string(),
                    inherits_from: None,
                    defaults: CredentialPolicyDefaults {
                        provider: Some(CloudProviderKind::Gcp),
                        runtime: Some(CloudRuntimeKind::LocalDev),
                        impersonation: None,
                        version_selector: None,
                        scope_merge: Some(ScopeMergeMode::Union),
                    },
                    intents: vec![CredentialIntentPolicy {
                        intent: "github.gist.create".to_string(),
                        provider: None,
                        runtime: None,
                        secret: Some(SecretBinding {
                            name: "github-token".to_string(),
                            prefix: Some("dev-".to_string()),
                            delimiter: None,
                        }),
                        required_scopes: Some(vec!["gist:read".to_string()]),
                        scope_merge: None,
                        impersonation: None,
                        version_selector: None,
                    }],
                },
                CredentialPolicyProfile {
                    name: "strict".to_string(),
                    inherits_from: Some("base".to_string()),
                    defaults: CredentialPolicyDefaults {
                        provider: None,
                        runtime: None,
                        impersonation: None,
                        version_selector: None,
                        scope_merge: Some(ScopeMergeMode::Replace),
                    },
                    intents: vec![CredentialIntentPolicy {
                        intent: "github.gist.create".to_string(),
                        provider: None,
                        runtime: None,
                        secret: None,
                        required_scopes: Some(vec!["gist:write".to_string()]),
                        scope_merge: None, // inherits defaults scope_merge
                        impersonation: None,
                        version_selector: None,
                    }],
                },
            ],
        };

        let resolved = spec
            .resolve_profile("strict")
            .expect("profile should resolve");

        // The resolved defaults should have Replace (child's explicit value).
        assert_eq!(
            resolved.defaults.scope_merge,
            Some(ScopeMergeMode::Replace),
            "child explicit Replace must override parent Union at defaults level"
        );

        // The intent scopes should be replaced, not unioned.
        let intent = resolved
            .intents
            .get("github.gist.create")
            .expect("intent should exist");
        assert_eq!(
            intent.required_scopes,
            vec!["gist:write".to_string()],
            "scope_merge=Replace means child scopes replace parent scopes"
        );
    }

    #[test]
    fn child_omitted_scope_merge_inherits_parent_union() {
        // Parent sets scope_merge = Union.
        // Child omits scope_merge (None).
        // The child should inherit Union from the parent.
        let spec = CredentialPolicySpec {
            version: 0,
            default_profile: None,
            profiles: vec![
                CredentialPolicyProfile {
                    name: "base".to_string(),
                    inherits_from: None,
                    defaults: CredentialPolicyDefaults {
                        provider: Some(CloudProviderKind::Gcp),
                        runtime: Some(CloudRuntimeKind::LocalDev),
                        impersonation: None,
                        version_selector: None,
                        scope_merge: Some(ScopeMergeMode::Union),
                    },
                    intents: vec![CredentialIntentPolicy {
                        intent: "github.gist.create".to_string(),
                        provider: None,
                        runtime: None,
                        secret: Some(SecretBinding {
                            name: "github-token".to_string(),
                            prefix: Some("dev-".to_string()),
                            delimiter: None,
                        }),
                        required_scopes: Some(vec!["gist:read".to_string()]),
                        scope_merge: None,
                        impersonation: None,
                        version_selector: None,
                    }],
                },
                CredentialPolicyProfile {
                    name: "dev".to_string(),
                    inherits_from: Some("base".to_string()),
                    defaults: CredentialPolicyDefaults {
                        provider: None,
                        runtime: None,
                        impersonation: None,
                        version_selector: None,
                        scope_merge: None, // not set -- should inherit Union
                    },
                    intents: vec![CredentialIntentPolicy {
                        intent: "github.gist.create".to_string(),
                        provider: None,
                        runtime: None,
                        secret: None,
                        required_scopes: Some(vec!["gist:write".to_string()]),
                        scope_merge: None,
                        impersonation: None,
                        version_selector: None,
                    }],
                },
            ],
        };

        let resolved = spec.resolve_profile("dev").expect("profile should resolve");

        // Should inherit parent's Union.
        assert_eq!(
            resolved.defaults.scope_merge,
            Some(ScopeMergeMode::Union),
            "omitted scope_merge should inherit parent Union"
        );

        // Scopes should be unioned (parent read + child write).
        let intent = resolved
            .intents
            .get("github.gist.create")
            .expect("intent should exist");
        assert!(
            intent.required_scopes.contains(&"gist:read".to_string()),
            "parent scope should be included via union"
        );
        assert!(
            intent.required_scopes.contains(&"gist:write".to_string()),
            "child scope should be included via union"
        );
    }
}
