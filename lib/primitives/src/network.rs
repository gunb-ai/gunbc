//! Network resource handle.
//!
//! A capability token representing permission to perform network I/O.

use gunbc_ir::resource::{
    capability_marker, ensure_capability_marker, AccessMode, Resource, ResourceId, ResourceKind,
};
use gunbc_ir::Value;
use std::collections::BTreeMap;

/// Network access capability.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NetworkHandle;

impl Resource for NetworkHandle {
    fn resource_id(&self) -> ResourceId {
        ResourceId::new("net")
    }

    fn access_mode(&self) -> AccessMode {
        AccessMode::Read
    }

    fn kind(&self) -> ResourceKind {
        ResourceKind::Capability
    }
}

/// Encode a NetworkHandle for DAG edges.
impl From<NetworkHandle> for Value {
    fn from(_handle: NetworkHandle) -> Self {
        let mut map = BTreeMap::new();
        map.insert("type".to_string(), Value::Str("network_handle".to_string()));
        map.insert("cap".to_string(), Value::Secret(capability_marker()));
        Value::Map(map)
    }
}

/// Decode a NetworkHandle from a Value.
impl TryFrom<&Value> for NetworkHandle {
    type Error = String;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        let map = value
            .as_map()
            .ok_or_else(|| "NetworkHandle parse error: expected map value".to_string())?;

        if let Err(e) = ensure_capability_marker(map, "NetworkHandle") {
            return Err(format!("NetworkHandle parse error: {}", e));
        }

        let type_field = map.get("type").and_then(Value::as_str).unwrap_or("");
        if type_field != "network_handle" {
            return Err(format!(
                "NetworkHandle parse error: unexpected type: {}",
                type_field
            ));
        }

        Ok(NetworkHandle)
    }
}

impl TryFrom<Value> for NetworkHandle {
    type Error = String;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        NetworkHandle::try_from(&value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn network_handle_try_from_rejects_missing_capability_marker() {
        let mut map = BTreeMap::new();
        map.insert("type".to_string(), Value::Str("network_handle".to_string()));
        let err = NetworkHandle::try_from(Value::Map(map)).expect_err("missing cap should fail");
        assert!(
            err.contains("missing capability marker"),
            "error should mention missing capability marker: {err}"
        );
    }

    #[test]
    fn network_handle_try_from_accepts_framework_encoded_value() {
        let encoded = Value::from(NetworkHandle);
        let parsed = NetworkHandle::try_from(encoded).expect("framework value should parse");
        assert_eq!(parsed, NetworkHandle);
    }
}
