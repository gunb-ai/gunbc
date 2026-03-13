//! Collection operation kinds for DAG-level collection processing.
//!
//! Defines the set of collection operations that can appear as
//! `PatternOp::CollectionAggregate` nodes in the IR.
//!
//! # Single-registry design (S11)
//!
//! All collection operation metadata is centralized here. Adding a new
//! collection op requires editing *only this file*. Downstream consumers
//! (lowerer, typechecker, evaluator, emitter) read from these methods
//! instead of maintaining their own parallel match arms.

use serde::{Deserialize, Serialize};

/// The kind of collection operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CollectionKind {
    Map,
    Filter,
    Fold,
    Join,
    FlatMap,
    Sort,
    Dedup,
    Any,
    All,
    Len,
    Contains,
    Split,
    Zip,
    Skip,
    Enumerate,
}

/// Emit-level collection family for code generation classification.
///
/// Collapses the fine-grained `CollectionKind` into four families that
/// correspond to distinct codegen strategies. Moved here from
/// `daglang-emit/src/computation.rs` so that the mapping is defined
/// next to the enum it classifies (S11).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmitCollectionFamily {
    Map,
    Filter,
    Fold,
    Sort,
}

/// Typecheck contract metadata for a collection/builtin operation.
///
/// Captures arity, parameter names, and output type — the information
/// the typechecker needs to validate call sites.
#[derive(Debug, Clone)]
pub struct BuiltinContract {
    /// Number of arguments (including the collection/receiver).
    pub arity: usize,
    /// Parameter names in positional order.
    pub params: &'static [&'static str],
    /// Output type name.
    pub output_type: &'static str,
}

/// All canonical collection operation names, in enum order.
///
/// Used by exhaustiveness checks and registry iteration.
pub const ALL_COLLECTION_OPS: &[CollectionKind] = &[
    CollectionKind::Map,
    CollectionKind::Filter,
    CollectionKind::Fold,
    CollectionKind::Join,
    CollectionKind::FlatMap,
    CollectionKind::Sort,
    CollectionKind::Dedup,
    CollectionKind::Any,
    CollectionKind::All,
    CollectionKind::Len,
    CollectionKind::Contains,
    CollectionKind::Split,
    CollectionKind::Zip,
    CollectionKind::Skip,
    CollectionKind::Enumerate,
];

impl CollectionKind {
    /// Node label used in DAG visualization and naming.
    pub fn node_label(&self) -> &'static str {
        match self {
            Self::Map => "MapNode",
            Self::Filter => "FilterNode",
            Self::Fold => "FoldNode",
            Self::Join => "JoinNode",
            Self::FlatMap => "FlatMapNode",
            Self::Sort => "SortNode",
            Self::Dedup => "DedupNode",
            Self::Any => "AnyNode",
            Self::All => "AllNode",
            Self::Len => "LenNode",
            Self::Contains => "ContainsNode",
            Self::Split => "SplitNode",
            Self::Zip => "ZipNode",
            Self::Skip => "SkipNode",
            Self::Enumerate => "EnumerateNode",
        }
    }

    /// The canonical name string for this variant.
    ///
    /// Inverse of [`Self::from_name`].
    pub fn from_name_reverse(&self) -> &'static str {
        match self {
            Self::Map => "map",
            Self::Filter => "filter",
            Self::Fold => "fold",
            Self::Join => "join",
            Self::FlatMap => "flat_map",
            Self::Sort => "sort",
            Self::Dedup => "dedup",
            Self::Any => "any",
            Self::All => "all",
            Self::Len => "len",
            Self::Contains => "contains",
            Self::Split => "split",
            Self::Zip => "zip",
            Self::Skip => "skip",
            Self::Enumerate => "enumerate",
        }
    }

    /// Parse a collection op name string into the corresponding variant.
    pub fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "map" => Self::Map,
            "filter" => Self::Filter,
            "fold" => Self::Fold,
            "join" => Self::Join,
            "flat_map" => Self::FlatMap,
            "sort" => Self::Sort,
            "dedup" => Self::Dedup,
            "any" => Self::Any,
            "all" => Self::All,
            "len" => Self::Len,
            "contains" => Self::Contains,
            "split" => Self::Split,
            "zip" => Self::Zip,
            "skip" => Self::Skip,
            "enumerate" => Self::Enumerate,
            _ => return None,
        })
    }

    /// Parse a name including DSL aliases into the corresponding variant.
    ///
    /// Handles both canonical names (via [`Self::from_name`]) and common
    /// aliases used in DSL source: `count`→Len, `sum`→Fold,
    /// `filter_map`→Filter, `sort_by`→Sort, `append`→Map.
    pub fn from_name_or_alias(name: &str) -> Option<Self> {
        match name {
            "count" => Some(Self::Len),
            "sum" => Some(Self::Fold),
            "filter_map" => Some(Self::Filter),
            "sort_by" => Some(Self::Sort),
            "append" => Some(Self::Map),
            _ => Self::from_name(name),
        }
    }

    /// Emit-level family for code generation classification (S11).
    ///
    /// Single source of truth for the `CollectionKind` → `EmitCollectionFamily`
    /// mapping, previously duplicated in `daglang-emit/src/computation.rs`.
    pub fn emit_family(&self) -> EmitCollectionFamily {
        match self {
            Self::Map | Self::FlatMap | Self::Join | Self::Split | Self::Zip | Self::Enumerate => {
                EmitCollectionFamily::Map
            }
            Self::Filter | Self::Contains | Self::Skip => EmitCollectionFamily::Filter,
            Self::Fold | Self::Any | Self::All | Self::Len => EmitCollectionFamily::Fold,
            Self::Sort | Self::Dedup => EmitCollectionFamily::Sort,
        }
    }

    /// Typecheck contract for this collection operation (S11).
    ///
    /// Returns the arity, parameter names, and output type that the
    /// typechecker uses to validate call sites. Single source of truth
    /// — previously duplicated in `builtin_callable_contracts()`.
    pub fn typecheck_contract(&self) -> BuiltinContract {
        match self {
            Self::Map => BuiltinContract {
                arity: 2,
                params: &["collection", "f"],
                output_type: "List",
            },
            Self::Filter => BuiltinContract {
                arity: 2,
                params: &["collection", "predicate"],
                output_type: "List",
            },
            Self::Fold => BuiltinContract {
                arity: 3,
                params: &["collection", "init", "f"],
                output_type: "Any",
            },
            Self::Join => BuiltinContract {
                arity: 2,
                params: &["collection", "separator"],
                output_type: "String",
            },
            Self::FlatMap => BuiltinContract {
                arity: 2,
                params: &["collection", "f"],
                output_type: "List",
            },
            Self::Sort => BuiltinContract {
                arity: 2,
                params: &["collection", "key_fn"],
                output_type: "List",
            },
            Self::Dedup => BuiltinContract {
                arity: 1,
                params: &["collection"],
                output_type: "List",
            },
            Self::Any => BuiltinContract {
                arity: 2,
                params: &["collection", "predicate"],
                output_type: "Bool",
            },
            Self::All => BuiltinContract {
                arity: 2,
                params: &["collection", "predicate"],
                output_type: "Bool",
            },
            Self::Len => BuiltinContract {
                arity: 1,
                params: &["collection"],
                output_type: "Int",
            },
            Self::Contains => BuiltinContract {
                arity: 2,
                params: &["collection", "item"],
                output_type: "Bool",
            },
            Self::Split => BuiltinContract {
                arity: 2,
                params: &["value", "delimiter"],
                output_type: "List",
            },
            Self::Zip => BuiltinContract {
                arity: 2,
                params: &["collection", "other"],
                output_type: "List",
            },
            Self::Skip => BuiltinContract {
                arity: 2,
                params: &["collection", "n"],
                output_type: "List",
            },
            Self::Enumerate => BuiltinContract {
                arity: 1,
                params: &["collection"],
                output_type: "List",
            },
        }
    }

    /// Whether this is an evaluator-handled intrinsic (S11).
    ///
    /// Returns `true` for operations handled by `evaluate_collection`.
    /// All collection ops are intrinsics.
    pub fn is_eval_intrinsic(&self) -> bool {
        // All CollectionKind variants are handled by the evaluator.
        true
    }
}

/// Builtin contract metadata for alias operations that map to collection ops
/// but have different names in the DSL (e.g., `filter_map`, `sort_by`).
///
/// These are operations that the typechecker needs to know about but that
/// are not direct `CollectionKind` variants. They are aliases resolved
/// to the canonical variant by `from_name_or_alias`.
pub fn alias_contracts() -> Vec<(&'static str, BuiltinContract)> {
    vec![
        (
            "filter_map",
            BuiltinContract {
                arity: 2,
                params: &["collection", "f"],
                output_type: "List",
            },
        ),
        (
            "sort_by",
            BuiltinContract {
                arity: 2,
                params: &["collection", "key_fn"],
                output_type: "List",
            },
        ),
        (
            "append",
            BuiltinContract {
                arity: 2,
                params: &["collection", "items"],
                output_type: "List",
            },
        ),
        (
            "count",
            BuiltinContract {
                arity: 1,
                params: &["collection"],
                output_type: "Int",
            },
        ),
        (
            "sum",
            BuiltinContract {
                arity: 1,
                params: &["collection"],
                output_type: "Int",
            },
        ),
    ]
}

/// Builtin contract metadata for non-collection builtins that the
/// typechecker and evaluator need to know about.
///
/// These are standalone functions (not collection ops) like `first`,
/// `last`, `starts_with`, etc.
pub fn non_collection_builtin_contracts() -> Vec<(&'static str, BuiltinContract)> {
    vec![
        (
            "first",
            BuiltinContract {
                arity: 1,
                params: &["collection"],
                output_type: "Any",
            },
        ),
        (
            "last",
            BuiltinContract {
                arity: 1,
                params: &["collection"],
                output_type: "Any",
            },
        ),
        (
            "max_by",
            BuiltinContract {
                arity: 2,
                params: &["collection", "f"],
                output_type: "Any",
            },
        ),
        (
            "starts_with",
            BuiltinContract {
                arity: 2,
                params: &["value", "prefix"],
                output_type: "Bool",
            },
        ),
        (
            "ends_with",
            BuiltinContract {
                arity: 2,
                params: &["value", "suffix"],
                output_type: "Bool",
            },
        ),
        (
            "repeat",
            BuiltinContract {
                arity: 2,
                params: &["value", "n"],
                output_type: "String",
            },
        ),
        (
            "replace_section",
            BuiltinContract {
                arity: 3,
                params: &["value", "section", "replacement"],
                output_type: "String",
            },
        ),
        (
            "chars",
            BuiltinContract {
                arity: 1,
                params: &["value"],
                output_type: "List",
            },
        ),
        (
            "to_bytes",
            BuiltinContract {
                arity: 1,
                params: &["value"],
                output_type: "Bytes",
            },
        ),
        (
            "to_json",
            BuiltinContract {
                arity: 1,
                params: &["value"],
                output_type: "Json",
            },
        ),
        (
            "hash",
            BuiltinContract {
                arity: 1,
                params: &["value"],
                output_type: "String",
            },
        ),
    ]
}

/// Check if a function name is an evaluator-handled intrinsic.
///
/// Centralizes the intrinsic name check (S11). Previously duplicated in
/// `daglang-eval/src/eval.rs`.
pub fn is_eval_intrinsic(name: &str) -> bool {
    // Check collection ops (canonical names)
    if CollectionKind::from_name(name).is_some() {
        return true;
    }
    // Check aliases
    matches!(
        name,
        "filter_map" | "flat_map" | "count" | "sum" | "sort_by" | "append"
    ) ||
    // Non-collection intrinsics
    matches!(
        name,
        "first" | "last" | "any" | "all" | "contains"
            | "starts_with" | "ends_with" | "repeat" | "chars"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_ops_have_contracts() {
        for kind in ALL_COLLECTION_OPS {
            let contract = kind.typecheck_contract();
            assert!(contract.arity > 0, "{kind:?} has zero arity");
            assert!(
                !contract.params.is_empty(),
                "{kind:?} has no params"
            );
            assert!(
                !contract.output_type.is_empty(),
                "{kind:?} has no output type"
            );
        }
    }

    #[test]
    fn all_ops_have_emit_family() {
        for kind in ALL_COLLECTION_OPS {
            // Just verifying it doesn't panic — exhaustiveness is checked
            // at compile time by the match.
            let _ = kind.emit_family();
        }
    }

    #[test]
    fn aliases_resolve_to_canonical() {
        assert_eq!(
            CollectionKind::from_name_or_alias("count"),
            Some(CollectionKind::Len)
        );
        assert_eq!(
            CollectionKind::from_name_or_alias("sum"),
            Some(CollectionKind::Fold)
        );
        assert_eq!(
            CollectionKind::from_name_or_alias("filter_map"),
            Some(CollectionKind::Filter)
        );
        assert_eq!(
            CollectionKind::from_name_or_alias("sort_by"),
            Some(CollectionKind::Sort)
        );
        assert_eq!(
            CollectionKind::from_name_or_alias("append"),
            Some(CollectionKind::Map)
        );
        // Canonical names still work
        assert_eq!(
            CollectionKind::from_name_or_alias("map"),
            Some(CollectionKind::Map)
        );
        // Unknown names return None
        assert_eq!(CollectionKind::from_name_or_alias("unknown_op"), None);
    }

    #[test]
    fn eval_intrinsic_recognizes_all_ops() {
        for kind in ALL_COLLECTION_OPS {
            // Use from_name's inverse: node_label strips "Node" suffix
            let label = kind.node_label();
            let name = &label[..label.len() - 4]; // "MapNode" -> "Map"
            let lower_name = name.to_lowercase();
            // The canonical names should be intrinsics
            if CollectionKind::from_name(&lower_name).is_some() {
                assert!(
                    is_eval_intrinsic(&lower_name),
                    "{lower_name} should be intrinsic"
                );
            }
        }
    }

    #[test]
    fn eval_intrinsic_recognizes_aliases() {
        for alias in ["filter_map", "flat_map", "count", "sum", "sort_by", "append"] {
            assert!(is_eval_intrinsic(alias), "{alias} should be intrinsic");
        }
    }

    #[test]
    fn eval_intrinsic_recognizes_non_collection() {
        for name in ["first", "last", "starts_with", "ends_with", "repeat", "chars"] {
            assert!(is_eval_intrinsic(name), "{name} should be intrinsic");
        }
    }
}
