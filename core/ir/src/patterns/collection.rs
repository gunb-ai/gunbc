//! Collection operation kinds for DAG-level collection processing.
//!
//! Defines the set of collection operations that can appear as
//! `PatternOp::CollectionAggregate` nodes in the IR.

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
}

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
            _ => return None,
        })
    }
}
