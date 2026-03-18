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
    // First-class aliases (formerly mapped to canonical variants via
    // from_name_or_alias, which caused arity mismatches — e.g., sum
    // has arity 1 but was mapped to Fold with arity 3).
    Count,
    Sum,
    FilterMap,
    SortBy,
    Append,
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
    CollectionKind::Count,
    CollectionKind::Sum,
    CollectionKind::FilterMap,
    CollectionKind::SortBy,
    CollectionKind::Append,
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
            Self::Count => "CountNode",
            Self::Sum => "SumNode",
            Self::FilterMap => "FilterMapNode",
            Self::SortBy => "SortByNode",
            Self::Append => "AppendNode",
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
            Self::Count => "count",
            Self::Sum => "sum",
            Self::FilterMap => "filter_map",
            Self::SortBy => "sort_by",
            Self::Append => "append",
        }
    }

    /// Parse a collection op name string into the corresponding variant.
    ///
    /// Handles both canonical names and DSL aliases (count, sum,
    /// filter_map, sort_by, append) — each is a first-class variant.
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
            "count" => Self::Count,
            "sum" => Self::Sum,
            "filter_map" => Self::FilterMap,
            "sort_by" => Self::SortBy,
            "append" => Self::Append,
            _ => return None,
        })
    }

    /// Emit-level family for code generation classification (S11).
    ///
    /// Single source of truth for the `CollectionKind` → `EmitCollectionFamily`
    /// mapping, previously duplicated in `daglang-emit/src/computation.rs`.
    pub fn emit_family(&self) -> EmitCollectionFamily {
        match self {
            Self::Map
            | Self::FlatMap
            | Self::Join
            | Self::Split
            | Self::Zip
            | Self::Enumerate
            | Self::Append => EmitCollectionFamily::Map,
            Self::Filter | Self::Contains | Self::Skip | Self::FilterMap => {
                EmitCollectionFamily::Filter
            }
            Self::Fold | Self::Any | Self::All | Self::Len | Self::Count | Self::Sum => {
                EmitCollectionFamily::Fold
            }
            Self::Sort | Self::Dedup | Self::SortBy => EmitCollectionFamily::Sort,
        }
    }

    /// Typecheck contract for this collection operation (S11).
    ///
    /// Returns the arity, parameter names, and output type that the
    /// typechecker uses to validate call sites. Single source of truth.
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
            Self::Count => BuiltinContract {
                arity: 1,
                params: &["collection"],
                output_type: "Int",
            },
            Self::Sum => BuiltinContract {
                arity: 1,
                params: &["collection"],
                output_type: "Int",
            },
            Self::FilterMap => BuiltinContract {
                arity: 2,
                params: &["collection", "f"],
                output_type: "List",
            },
            Self::SortBy => BuiltinContract {
                arity: 2,
                params: &["collection", "key_fn"],
                output_type: "List",
            },
            Self::Append => BuiltinContract {
                arity: 2,
                params: &["collection", "items"],
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
            "get",
            BuiltinContract {
                arity: 2,
                params: &["collection", "index"],
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
            "string_contains",
            BuiltinContract {
                arity: 2,
                params: &["value", "substring"],
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
        // v2 compiler string builtins
        (
            "char_at",
            BuiltinContract {
                arity: 2,
                params: &["s", "pos"],
                output_type: "String",
            },
        ),
        (
            "string_length",
            BuiltinContract {
                arity: 1,
                params: &["s"],
                output_type: "Int",
            },
        ),
        (
            "substring",
            BuiltinContract {
                arity: 3,
                params: &["s", "start", "end"],
                output_type: "String",
            },
        ),
        (
            "lookup",
            BuiltinContract {
                arity: 2,
                params: &["table", "key"],
                output_type: "Any",
            },
        ),
        (
            "with",
            BuiltinContract {
                arity: 2,
                params: &["record", "updates"],
                output_type: "Any",
            },
        ),
        (
            "concat",
            BuiltinContract {
                arity: 2,
                params: &["a", "b"],
                output_type: "Any",
            },
        ),
    ]
}

/// Look up the typecheck contract for a name.
///
/// Resolution order: CollectionKind::from_name → non_collection_builtin_contracts.
pub fn contract_for_name(name: &str) -> Option<BuiltinContract> {
    // 1. Collection ops (all names are first-class variants now)
    if let Some(kind) = CollectionKind::from_name(name) {
        return Some(kind.typecheck_contract());
    }
    // 2. Non-collection builtins
    for (builtin_name, contract) in non_collection_builtin_contracts() {
        if builtin_name == name {
            return Some(contract);
        }
    }
    None
}

/// Check if a function name is an evaluator-handled intrinsic.
///
/// Derived from the registries (S11) — no hand-maintained string list.
/// A name is intrinsic if it appears in any of:
///   1. `CollectionKind::from_name` (all collection ops including former aliases)
///   2. `non_collection_builtin_contracts()` (standalone builtins)
pub fn is_eval_intrinsic(name: &str) -> bool {
    if CollectionKind::from_name(name).is_some() {
        return true;
    }
    non_collection_builtin_contracts()
        .iter()
        .any(|(n, _)| *n == name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_ops_have_contracts() {
        for kind in ALL_COLLECTION_OPS {
            let contract = kind.typecheck_contract();
            assert!(contract.arity > 0, "{kind:?} has zero arity");
            assert!(!contract.params.is_empty(), "{kind:?} has no params");
            assert!(
                !contract.output_type.is_empty(),
                "{kind:?} has no output type"
            );
        }
    }

    #[test]
    fn all_ops_have_emit_family() {
        for kind in ALL_COLLECTION_OPS {
            let _ = kind.emit_family();
        }
    }

    #[test]
    fn former_aliases_are_first_class() {
        assert_eq!(
            CollectionKind::from_name("count"),
            Some(CollectionKind::Count)
        );
        assert_eq!(CollectionKind::from_name("sum"), Some(CollectionKind::Sum));
        assert_eq!(
            CollectionKind::from_name("filter_map"),
            Some(CollectionKind::FilterMap)
        );
        assert_eq!(
            CollectionKind::from_name("sort_by"),
            Some(CollectionKind::SortBy)
        );
        assert_eq!(
            CollectionKind::from_name("append"),
            Some(CollectionKind::Append)
        );
        // Canonical names still work
        assert_eq!(CollectionKind::from_name("map"), Some(CollectionKind::Map));
        // Unknown names return None
        assert_eq!(CollectionKind::from_name("unknown_op"), None);
    }

    #[test]
    fn sum_has_correct_arity() {
        // This was the original BUG-6: sum mapped to Fold (arity 3)
        // but sum only takes 1 argument.
        let sum = contract_for_name("sum").unwrap();
        assert_eq!(sum.arity, 1, "sum should have arity 1");
        assert_eq!(sum.output_type, "Int");

        let count = contract_for_name("count").unwrap();
        assert_eq!(count.arity, 1);

        // canonical fold should still have arity 3
        let fold = contract_for_name("fold").unwrap();
        assert_eq!(fold.arity, 3);
    }

    #[test]
    fn eval_intrinsic_recognizes_all_ops() {
        for kind in ALL_COLLECTION_OPS {
            let name = kind.from_name_reverse();
            assert!(is_eval_intrinsic(name), "{name} should be intrinsic");
        }
    }

    #[test]
    fn eval_intrinsic_recognizes_non_collection() {
        for name in [
            "first",
            "last",
            "get",
            "starts_with",
            "ends_with",
            "string_contains",
            "repeat",
            "chars",
        ] {
            assert!(is_eval_intrinsic(name), "{name} should be intrinsic");
        }
    }
}
