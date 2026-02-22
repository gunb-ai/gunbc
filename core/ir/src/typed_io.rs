//! Typed node I/O wrappers for gradual DAG typing hardening.
//!
//! These wrappers provide compile-time intent for port type + cardinality while
//! still lowering to existing `Port` values. Legacy stringly APIs remain
//! available during migration.

use crate::dag::Port;
use crate::types::{Cardinality, PortName, TypeId};
use std::marker::PhantomData;

/// Compile-time mapping from a Rust marker type to IR port type metadata.
pub trait PortTypeTag {
    /// IR type identifier for this marker.
    fn type_id() -> TypeId;

    /// Cardinality for this marker.
    fn cardinality() -> Cardinality {
        Cardinality::ONE
    }
}

/// Zero-or-one cardinality wrapper marker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OptionalTag<T>(PhantomData<T>);

/// Zero-or-more cardinality wrapper marker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ListTag<T>(PhantomData<T>);

/// One-or-more cardinality wrapper marker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NonEmptyListTag<T>(PhantomData<T>);

impl<T: PortTypeTag> PortTypeTag for OptionalTag<T> {
    fn type_id() -> TypeId {
        T::type_id()
    }

    fn cardinality() -> Cardinality {
        Cardinality::ZERO_OR_ONE
    }
}

impl<T: PortTypeTag> PortTypeTag for ListTag<T> {
    fn type_id() -> TypeId {
        T::type_id()
    }

    fn cardinality() -> Cardinality {
        Cardinality::ZERO_OR_MORE
    }
}

impl<T: PortTypeTag> PortTypeTag for NonEmptyListTag<T> {
    fn type_id() -> TypeId {
        T::type_id()
    }

    fn cardinality() -> Cardinality {
        Cardinality::ONE_OR_MORE
    }
}

impl PortTypeTag for String {
    fn type_id() -> TypeId {
        TypeId::from("String")
    }
}

impl PortTypeTag for bool {
    fn type_id() -> TypeId {
        TypeId::from("Bool")
    }
}

impl PortTypeTag for i64 {
    fn type_id() -> TypeId {
        TypeId::from("Int")
    }
}

impl PortTypeTag for i32 {
    fn type_id() -> TypeId {
        TypeId::from("Int")
    }
}

impl PortTypeTag for u64 {
    fn type_id() -> TypeId {
        TypeId::from("Int")
    }
}

impl PortTypeTag for serde_json::Value {
    fn type_id() -> TypeId {
        TypeId::from("Json")
    }
}

impl PortTypeTag for () {
    fn type_id() -> TypeId {
        TypeId::from("Unit")
    }
}

/// Marker for `Any`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnyTag;
impl PortTypeTag for AnyTag {
    fn type_id() -> TypeId {
        TypeId::from("Any")
    }
}

/// Marker for `TransportRequest`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransportRequestTag;
impl PortTypeTag for TransportRequestTag {
    fn type_id() -> TypeId {
        TypeId::from("TransportRequest")
    }
}

/// Marker for `TransportResponse`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransportResponseTag;
impl PortTypeTag for TransportResponseTag {
    fn type_id() -> TypeId {
        TypeId::from("TransportResponse")
    }
}

/// Marker for `Credential`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CredentialTag;
impl PortTypeTag for CredentialTag {
    fn type_id() -> TypeId {
        TypeId::from("Credential")
    }
}

/// Marker for `Secret`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SecretTag;
impl PortTypeTag for SecretTag {
    fn type_id() -> TypeId {
        TypeId::from("Secret")
    }
}

/// Marker for `FilesystemHandle`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FilesystemHandleTag;
impl PortTypeTag for FilesystemHandleTag {
    fn type_id() -> TypeId {
        TypeId::from("FilesystemHandle")
    }
}

/// Marker for `NetworkHandle`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NetworkHandleTag;
impl PortTypeTag for NetworkHandleTag {
    fn type_id() -> TypeId {
        TypeId::from("NetworkHandle")
    }
}

/// Marker for `ToolHandle`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ToolHandleTag;
impl PortTypeTag for ToolHandleTag {
    fn type_id() -> TypeId {
        TypeId::from("ToolHandle")
    }
}

/// Marker for `Platform`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlatformTag;
impl PortTypeTag for PlatformTag {
    fn type_id() -> TypeId {
        TypeId::from("Platform")
    }
}

/// Marker for `Timestamp`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimestampTag;
impl PortTypeTag for TimestampTag {
    fn type_id() -> TypeId {
        TypeId::from("Timestamp")
    }
}

/// Marker for `FilePath`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FilePathTag;
impl PortTypeTag for FilePathTag {
    fn type_id() -> TypeId {
        TypeId::from("FilePath")
    }
}

/// Marker for `Url`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UrlTag;
impl PortTypeTag for UrlTag {
    fn type_id() -> TypeId {
        TypeId::from("Url")
    }
}

macro_rules! define_named_type_tag {
    ($name:ident, $type_id:literal) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct $name;
        impl PortTypeTag for $name {
            fn type_id() -> TypeId {
                TypeId::from($type_id)
            }
        }
    };
}

define_named_type_tag!(MapTag, "Map");
define_named_type_tag!(NonEmptyStringTag, "NonEmptyString");
define_named_type_tag!(SecretNameTag, "SecretName");
define_named_type_tag!(CloudSecretConfigTag, "CloudSecretConfig");
define_named_type_tag!(OidcSubjectTokenTag, "OidcSubjectToken");
define_named_type_tag!(GcpProjectIdTag, "GcpProjectId");
define_named_type_tag!(GcpSecretIdTag, "GcpSecretId");
define_named_type_tag!(GcpSecretVersionTag, "GcpSecretVersion");
define_named_type_tag!(GcpServiceAccountEmailTag, "GcpServiceAccountEmail");

/// Generic typed port wrapper.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedPort<T: PortTypeTag> {
    name: PortName,
    _marker: PhantomData<T>,
}

impl<T: PortTypeTag> TypedPort<T> {
    /// Construct a typed port by name.
    pub fn new(name: impl Into<PortName>) -> Self {
        Self {
            name: name.into(),
            _marker: PhantomData,
        }
    }

    /// Port name.
    pub fn name(&self) -> &PortName {
        &self.name
    }

    /// IR type identifier for this port.
    pub fn type_id(&self) -> TypeId {
        T::type_id()
    }

    /// Cardinality for this port.
    pub fn cardinality(&self) -> Cardinality {
        T::cardinality()
    }

    /// Convert into legacy untyped `Port`.
    pub fn into_port(self) -> Port {
        Port::with_cardinality(self.name, T::type_id(), T::cardinality())
    }
}

impl<T: PortTypeTag> From<TypedPort<T>> for Port {
    fn from(value: TypedPort<T>) -> Self {
        value.into_port()
    }
}

/// Typed input port wrapper.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedInput<T: PortTypeTag>(TypedPort<T>);

impl<T: PortTypeTag> TypedInput<T> {
    /// Construct a typed input port by name.
    pub fn new(name: impl Into<PortName>) -> Self {
        Self(TypedPort::new(name))
    }

    /// Access underlying typed port.
    pub fn as_port(&self) -> &TypedPort<T> {
        &self.0
    }

    /// Convert into legacy untyped `Port`.
    pub fn into_port(self) -> Port {
        self.0.into_port()
    }
}

impl<T: PortTypeTag> From<TypedInput<T>> for Port {
    fn from(value: TypedInput<T>) -> Self {
        value.into_port()
    }
}

/// Typed output port wrapper.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedOutput<T: PortTypeTag>(TypedPort<T>);

impl<T: PortTypeTag> TypedOutput<T> {
    /// Construct a typed output port by name.
    pub fn new(name: impl Into<PortName>) -> Self {
        Self(TypedPort::new(name))
    }

    /// Access underlying typed port.
    pub fn as_port(&self) -> &TypedPort<T> {
        &self.0
    }

    /// Convert into legacy untyped `Port`.
    pub fn into_port(self) -> Port {
        self.0.into_port()
    }
}

impl<T: PortTypeTag> From<TypedOutput<T>> for Port {
    fn from(value: TypedOutput<T>) -> Self {
        value.into_port()
    }
}

/// Helper constructor for typed ports.
pub fn typed_port<T: PortTypeTag>(name: impl Into<PortName>) -> TypedPort<T> {
    TypedPort::new(name)
}

/// Helper constructor for typed input ports.
pub fn typed_input<T: PortTypeTag>(name: impl Into<PortName>) -> TypedInput<T> {
    TypedInput::new(name)
}

/// Helper constructor for typed output ports.
pub fn typed_output<T: PortTypeTag>(name: impl Into<PortName>) -> TypedOutput<T> {
    TypedOutput::new(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typed_string_port_maps_to_scalar_string() {
        let p: Port = typed_port::<String>("name").into();
        assert_eq!(p.name.0, "name");
        assert_eq!(p.type_id.0, "String");
        assert_eq!(p.cardinality, Cardinality::ONE);
    }

    #[test]
    fn optional_list_wrappers_map_to_cardinality_only() {
        let optional: Port = typed_input::<OptionalTag<String>>("maybe_name").into();
        assert_eq!(optional.type_id.0, "String");
        assert_eq!(optional.cardinality, Cardinality::ZERO_OR_ONE);

        let many: Port = typed_output::<ListTag<i64>>("values").into();
        assert_eq!(many.type_id.0, "Int");
        assert_eq!(many.cardinality, Cardinality::ZERO_OR_MORE);

        let non_empty: Port = typed_output::<NonEmptyListTag<UrlTag>>("urls").into();
        assert_eq!(non_empty.type_id.0, "Url");
        assert_eq!(non_empty.cardinality, Cardinality::ONE_OR_MORE);
    }

    #[test]
    fn semantic_tags_use_expected_type_ids() {
        let request: Port = typed_input::<TransportRequestTag>("request").into();
        let credential: Port = typed_output::<CredentialTag>("credential").into();
        assert_eq!(request.type_id.0, "TransportRequest");
        assert_eq!(credential.type_id.0, "Credential");
    }
}
