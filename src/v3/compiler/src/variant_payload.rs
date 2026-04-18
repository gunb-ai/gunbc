use std::collections::HashMap;

use crate::dag::{Dag, DeclarationId, TypeConnective};

/// Shared emitter-side classification for a resolved variant payload.
/// Distinguishes the two payload forms that affect field projection
/// lowering:
///
/// - positional single-field payloads (`Variant(T)`) bind directly to
///   the carried value
/// - named payload fields (`Variant { x: T, ... }`) require either a
///   whole-payload carrier expression or per-field overrides,
///   depending on the target's spec rule
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum VariantPayloadShape {
    Empty,
    PositionalSingle,
    NamedFields(Vec<String>),
}

/// Shared emitter-side mirror of
/// `std.clean_emission.VariantPayloadFieldAccessRule`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum VariantPayloadFieldAccessRuleBinding {
    AccessFromPayloadBinding,
    OverrideNamedFieldsAtBindingSite,
}

/// Per-payload-port rendering authority used by the emitters.
/// `Direct` means the payload port itself renders to one expression;
/// `Fields` means the whole payload value is not renderable directly,
/// so downstream field projections must be answered by the provided
/// per-field bindings.
#[derive(Debug, Clone)]
pub(crate) enum VariantPayloadBinding<T> {
    Direct(T),
    Fields(HashMap<String, T>),
}

impl<T> VariantPayloadBinding<T> {
    pub(crate) fn direct(&self) -> Option<&T> {
        match self {
            Self::Direct(value) => Some(value),
            Self::Fields(_) => None,
        }
    }

    pub(crate) fn field(&self, label: &str) -> Option<&T> {
        match self {
            Self::Direct(_) => None,
            Self::Fields(fields) => fields.get(label),
        }
    }
}

pub(crate) fn variant_payload_shape(dag: &Dag, variant_id: DeclarationId) -> Option<VariantPayloadShape> {
    let TypeConnective::Conj { children } = &dag.declaration(variant_id).connective else {
        return None;
    };
    match children.as_slice() {
        [] => Some(VariantPayloadShape::Empty),
        [field] if field.label == "_0" => Some(VariantPayloadShape::PositionalSingle),
        fields => Some(VariantPayloadShape::NamedFields(
            fields.iter().map(|field| field.label.clone()).collect(),
        )),
    }
}
