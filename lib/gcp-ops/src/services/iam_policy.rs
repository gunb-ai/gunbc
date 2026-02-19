use serde::{Deserialize, Serialize};

/// IAM Binding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IamBinding {
    pub role: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub members: Vec<String>,
}

/// IAM Policy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IamPolicy {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bindings: Vec<IamBinding>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub etag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<i32>,
}

impl IamPolicy {
    /// Ensures that a role has the specified member.
    /// Returns true if the policy was modified, false if the member was already present.
    pub fn ensure_member(&mut self, role: &str, member: &str) -> bool {
        for binding in &mut self.bindings {
            if binding.role == role {
                if !binding.members.contains(&member.to_string()) {
                    binding.members.push(member.to_string());
                    return true;
                }
                return false;
            }
        }

        // Role not found, add a new binding
        self.bindings.push(IamBinding {
            role: role.to_string(),
            members: vec![member.to_string()],
        });
        true
    }

    /// Attempts to extract an IamPolicy from a generic JSON.
    /// Handles both direct policies and envelope policies (where the policy is nested under a `policy` key).
    pub fn extract(value: &serde_json::Value) -> Option<Self> {
        // Try direct deserialization
        if let Ok(policy) = serde_json::from_value::<IamPolicy>(value.clone()) {
            return Some(policy);
        }

        // Try extracting from an envelope
        if let Some(inner) = value.get("policy") {
            if let Ok(policy) = serde_json::from_value::<IamPolicy>(inner.clone()) {
                return Some(policy);
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ensure_member_adds_to_existing_role() {
        let mut policy = IamPolicy {
            bindings: vec![IamBinding {
                role: "roles/viewer".to_string(),
                members: vec!["user:alice@example.com".to_string()],
            }],
            etag: None,
            version: None,
        };

        assert!(policy.ensure_member("roles/viewer", "user:bob@example.com"));
        assert_eq!(policy.bindings[0].members.len(), 2);
    }

    #[test]
    fn ensure_member_adds_new_role() {
        let mut policy = IamPolicy {
            bindings: vec![],
            etag: None,
            version: None,
        };

        assert!(policy.ensure_member("roles/viewer", "user:alice@example.com"));
        assert_eq!(policy.bindings.len(), 1);
        assert_eq!(policy.bindings[0].role, "roles/viewer");
    }

    #[test]
    fn ensure_member_does_not_duplicate() {
        let mut policy = IamPolicy {
            bindings: vec![IamBinding {
                role: "roles/viewer".to_string(),
                members: vec!["user:alice@example.com".to_string()],
            }],
            etag: None,
            version: None,
        };

        assert!(!policy.ensure_member("roles/viewer", "user:alice@example.com"));
        assert_eq!(policy.bindings[0].members.len(), 1);
    }
}
