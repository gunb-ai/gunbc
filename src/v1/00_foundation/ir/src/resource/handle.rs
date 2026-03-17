//! Unified resource handle type.
//!
//! `ResourceHandle<R>` is proof of resource acquisition that flows through DAG edges.
//! This unifies `ToolHandle` and build resource handles under a single abstraction.
//!
//! The handle carries:
//! - The resource it refers to
//! - The freshness key at time of acquisition (proof it was fresh)
//! - A capability marker preventing forgery

use super::super::{ResourceId, SecretString, Value};
use super::ContentHash;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::sync::LazyLock;

/// Type marker used in the map's `type` field (not a secret).
const RESOURCE_HANDLE_MARKER: &str = "resource_handle";

/// Per-process random secret for capability validation.
///
/// This prevents forgery: even though `SecretString::new()` is public,
/// constructing a `Value::Secret` with the right contents requires
/// knowing this random value, which changes every process start.
static PROCESS_SECRET: LazyLock<String> = LazyLock::new(|| {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hasher};
    let s = RandomState::new();
    let mut h = s.build_hasher();
    h.write_u64(std::process::id() as u64);
    format!("rh_{}", h.finish())
});

/// A handle proving a resource has been acquired and is fresh.
///
/// This is the **only way** to depend on a resource in a DAG. You cannot
/// construct this directly — it only comes from successful resource acquisition
/// via the framework.
///
/// # Design
///
/// The capability pattern enforces that:
/// 1. You cannot use a resource without acquiring it first
/// 2. Acquisition happens via the resource framework
/// 3. The handle carries the freshness key as proof
///
/// # Unification
///
/// Both tools and build resources use the same handle type:
/// - `ResourceHandle<ToolResource>` — carries resolved path
/// - `ResourceHandle<BuildResource>` — carries proof of freshness
///
/// The "payload" differs, but the handle pattern is identical.
#[derive(Debug, Clone)]
pub struct ResourceHandle<R> {
    /// The resource this handle refers to.
    resource_id: ResourceId,
    /// The freshness key at time of acquisition.
    key: ContentHash,
    /// Marker for the resource type.
    _marker: PhantomData<R>,
}

impl<R> ResourceHandle<R> {
    /// Create a new handle after successful acquisition.
    ///
    /// **Framework use only.** This should only be called by the resource
    /// acquisition framework after successfully verifying or creating a resource.
    pub(crate) fn acquire(resource_id: ResourceId, key: ContentHash) -> Self {
        Self {
            resource_id,
            key,
            _marker: PhantomData,
        }
    }

    /// Get the resource ID this handle refers to.
    pub fn resource_id(&self) -> &ResourceId {
        &self.resource_id
    }

    /// Get the freshness key at time of acquisition.
    ///
    /// This is proof that the resource was fresh when acquired.
    pub fn key(&self) -> &ContentHash {
        &self.key
    }

    /// Create a mock handle for testing/DryRun.
    #[cfg(test)]
    pub(crate) fn mock(resource_id: ResourceId) -> Self {
        Self {
            resource_id,
            key: ContentHash::empty(),
            _marker: PhantomData,
        }
    }
}

impl<R> PartialEq for ResourceHandle<R> {
    fn eq(&self, other: &Self) -> bool {
        self.resource_id == other.resource_id && self.key == other.key
    }
}

impl<R> Eq for ResourceHandle<R> {}

/// Create a valid handle-shaped [`Value`] for DryRun and test wiring.
///
/// This is the single authority for the runtime encoding of `ResourceHandle`
/// values outside this module.
pub fn mock_resource_handle_value(resource_id: ResourceId) -> Value {
    Value::from(ResourceHandle::<()> {
        resource_id,
        key: ContentHash::empty(),
        _marker: PhantomData,
    })
}

/// Convert a ResourceHandle to a Value for passing through DAG edges.
impl<R> From<ResourceHandle<R>> for Value {
    fn from(handle: ResourceHandle<R>) -> Self {
        let mut map = BTreeMap::new();
        map.insert(
            "type".to_string(),
            Value::Str(RESOURCE_HANDLE_MARKER.to_string()),
        );
        map.insert(
            "resource_id".to_string(),
            Value::Str(handle.resource_id.0.clone()),
        );
        map.insert(
            "key".to_string(),
            Value::Str(handle.key.as_str().to_string()),
        );
        map.insert(
            "cap".to_string(),
            Value::Secret(super::super::SecretString::new(&*PROCESS_SECRET)),
        );
        Value::Map(map)
    }
}

/// Error when parsing a ResourceHandle from a Value.
#[derive(Debug, Clone)]
pub struct HandleParseError {
    pub message: String,
}

impl std::fmt::Display for HandleParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ResourceHandle parse error: {}", self.message)
    }
}

impl std::error::Error for HandleParseError {}

/// Try to reconstruct a ResourceHandle from a Value.
///
/// The Value must be a map with:
/// - type = "resource_handle"
/// - resource_id = string
/// - key = string (hex hash)
/// - cap = secret marker
impl<R> TryFrom<&Value> for ResourceHandle<R> {
    type Error = HandleParseError;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        let map = match value {
            Value::Map(m) => m,
            _ => {
                return Err(HandleParseError {
                    message: "Expected map value".to_string(),
                })
            }
        };

        // Check capability marker (per-process secret prevents forgery)
        match map.get("cap") {
            #[allow(clippy::disallowed_methods)] // Approved: capability-marker validation
            Some(Value::Secret(s)) if s.expose_plaintext_for_transport() == *PROCESS_SECRET => {}
            _ => {
                return Err(HandleParseError {
                    message: "Missing or invalid capability marker".to_string(),
                })
            }
        }

        // Check type field
        let type_field = map.get("type").and_then(Value::as_str).unwrap_or("");
        if type_field != RESOURCE_HANDLE_MARKER {
            return Err(HandleParseError {
                message: format!(
                    "Invalid type: expected '{}', got '{}'",
                    RESOURCE_HANDLE_MARKER, type_field
                ),
            });
        }

        // Extract resource_id
        let resource_id_str = map
            .get("resource_id")
            .and_then(Value::as_str)
            .ok_or_else(|| HandleParseError {
                message: "Missing 'resource_id'".to_string(),
            })?;

        // Extract key
        let key_str = map
            .get("key")
            .and_then(Value::as_str)
            .ok_or_else(|| HandleParseError {
                message: "Missing 'key'".to_string(),
            })?;

        Ok(ResourceHandle {
            resource_id: ResourceId::new(resource_id_str),
            key: ContentHash::new(key_str),
            _marker: PhantomData,
        })
    }
}

impl<R> TryFrom<Value> for ResourceHandle<R> {
    type Error = HandleParseError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        ResourceHandle::try_from(&value)
    }
}

/// Serialization support for ResourceHandle.
///
/// This uses the same structural fields as the runtime `Value` encoding so
/// serde does not become a second, forgeable construction path.
impl<R> Serialize for ResourceHandle<R> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;

        let mut state = serializer.serialize_struct("ResourceHandle", 4)?;
        state.serialize_field("type", RESOURCE_HANDLE_MARKER)?;
        state.serialize_field("resource_id", &self.resource_id)?;
        state.serialize_field("key", &self.key)?;
        state.serialize_field("cap", &SecretString::new(&*PROCESS_SECRET))?;
        state.end()
    }
}

/// Deserialization support for ResourceHandle.
impl<'de, R> Deserialize<'de> for ResourceHandle<R> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct HandleData {
            #[serde(rename = "type")]
            type_field: String,
            resource_id: ResourceId,
            key: ContentHash,
            cap: SecretString,
        }

        let data = HandleData::deserialize(deserializer)?;

        if data.type_field != RESOURCE_HANDLE_MARKER {
            return Err(serde::de::Error::custom(format!(
                "Invalid type: expected '{}', got '{}'",
                RESOURCE_HANDLE_MARKER, data.type_field
            )));
        }

        #[allow(clippy::disallowed_methods)] // Approved: capability-marker validation
        if data.cap.expose_plaintext_for_transport() != *PROCESS_SECRET {
            return Err(serde::de::Error::custom(
                "Missing or invalid capability marker",
            ));
        }

        Ok(ResourceHandle {
            resource_id: data.resource_id,
            key: data.key,
            _marker: PhantomData,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Marker type for testing
    #[derive(Debug, Clone)]
    struct TestResource;

    #[test]
    fn test_handle_acquire() {
        let id = ResourceId::new("test:resource");
        let key = ContentHash::from_bytes(b"test");
        let handle: ResourceHandle<TestResource> = ResourceHandle::acquire(id.clone(), key.clone());

        assert_eq!(handle.resource_id(), &id);
        assert_eq!(handle.key(), &key);
    }

    #[test]
    fn test_handle_mock() {
        let handle: ResourceHandle<TestResource> =
            ResourceHandle::mock(ResourceId::new("test:mock"));

        assert_eq!(handle.resource_id().0, "test:mock");
        assert_eq!(handle.key(), &ContentHash::empty());
    }

    #[test]
    fn test_handle_to_value() {
        let handle: ResourceHandle<TestResource> = ResourceHandle::acquire(
            ResourceId::new("test:value"),
            ContentHash::from_bytes(b"data"),
        );

        let value: Value = handle.into();
        assert!(matches!(value, Value::Map(_)));
    }

    #[test]
    fn test_handle_roundtrip() {
        let original: ResourceHandle<TestResource> = ResourceHandle::acquire(
            ResourceId::new("test:roundtrip"),
            ContentHash::from_bytes(b"roundtrip"),
        );

        let value: Value = original.clone().into();
        let restored: ResourceHandle<TestResource> = value.try_into().expect("parse failed");

        assert_eq!(original.resource_id(), restored.resource_id());
        assert_eq!(original.key(), restored.key());
    }

    #[test]
    fn test_handle_parse_invalid() {
        let value = Value::Str("not a map".to_string());
        let result: Result<ResourceHandle<TestResource>, _> = value.try_into();
        assert!(result.is_err());
    }

    #[test]
    fn test_handle_forgery_rejected() {
        // Construct a Value::Map that looks like a handle but has a fake cap
        let mut map = BTreeMap::new();
        map.insert(
            "type".to_string(),
            Value::Str("resource_handle".to_string()),
        );
        map.insert(
            "resource_id".to_string(),
            Value::Str("forged:resource".to_string()),
        );
        map.insert("key".to_string(), Value::Str("deadbeef".to_string()));
        map.insert(
            "cap".to_string(),
            Value::Secret(crate::SecretString::new("resource_handle")),
        );

        let value = Value::Map(map);
        let result: Result<ResourceHandle<TestResource>, _> = ResourceHandle::try_from(&value);
        assert!(result.is_err(), "forged handle should be rejected");
        assert!(
            result.unwrap_err().message.contains("capability marker"),
            "error should mention capability marker"
        );
    }

    #[test]
    fn test_handle_equality() {
        let h1: ResourceHandle<TestResource> =
            ResourceHandle::acquire(ResourceId::new("test:eq"), ContentHash::from_bytes(b"same"));
        let h2: ResourceHandle<TestResource> =
            ResourceHandle::acquire(ResourceId::new("test:eq"), ContentHash::from_bytes(b"same"));
        let h3: ResourceHandle<TestResource> = ResourceHandle::acquire(
            ResourceId::new("test:eq"),
            ContentHash::from_bytes(b"different"),
        );

        assert_eq!(h1, h2);
        assert_ne!(h1, h3);
    }

    #[test]
    fn test_handle_serde_roundtrip() {
        let original: ResourceHandle<TestResource> = ResourceHandle::acquire(
            ResourceId::new("test:serde"),
            ContentHash::from_bytes(b"serde"),
        );

        let json = serde_json::to_string(&original).expect("serialize should succeed");
        let restored: ResourceHandle<TestResource> =
            serde_json::from_str(&json).expect("deserialize should succeed");

        assert_eq!(original, restored);
    }

    #[test]
    fn test_handle_serde_rejects_missing_capability_marker() {
        let json = serde_json::json!({
            "type": RESOURCE_HANDLE_MARKER,
            "resource_id": "test:serde-forged",
            "key": ContentHash::from_bytes(b"serde-forged"),
        });

        let result: Result<ResourceHandle<TestResource>, _> = serde_json::from_value(json);
        assert!(
            result.is_err(),
            "serde input without cap should be rejected"
        );
    }
}
