//! Collection primitives - list operations with cardinality awareness.
//!
//! These operations work on collections (List, Json arrays) and
//! respect cardinality constraints for automatic test generation.

use gunbc_exec::{require_str_list, ExecError, Executable, OutputMap};
use gunbc_ir::Value;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;


/// Wrapper enum for all collection operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CollectionOp {
    Map(MapOp),
    Filter(FilterOp),
    Fold(FoldOp),
    Sort(SortOp),
    First(FirstOp),
    Last(LastOp),
}

impl Executable for CollectionOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            CollectionOp::Map(op) => op.execute(inputs),
            CollectionOp::Filter(op) => op.execute(inputs),
            CollectionOp::Fold(op) => op.execute(inputs),
            CollectionOp::Sort(op) => op.execute(inputs),
            CollectionOp::First(op) => op.execute(inputs),
            CollectionOp::Last(op) => op.execute(inputs),
        }
    }
}

/// Map operation - apply a transformation to each element.
///
/// Note: In DAG execution, Map is implemented as a higher-order pattern
/// that expands into parallel branches. This primitive handles simple
/// string transformations directly.
///
/// Inputs:
/// - `input`: List to map over
/// - `transform`: String transformation type
///
/// Outputs:
/// - `output`: Transformed List
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum MapOp {
    /// Convert each string to uppercase
    ToUppercase,
    /// Convert each string to lowercase
    ToLowercase,
    /// Trim whitespace from each string
    Trim,
    /// Apply a prefix to each string
    Prefix(String),
    /// Apply a suffix to each string
    Suffix(String),
    /// Identity - pass through unchanged (for SubDag composition)
    #[default]
    Identity,
}

impl Executable for MapOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let list = require_str_list(&inputs, "input")?;

        let result: Vec<String> = match self {
            MapOp::ToUppercase => list.iter().map(|s| s.to_uppercase()).collect(),
            MapOp::ToLowercase => list.iter().map(|s| s.to_lowercase()).collect(),
            MapOp::Trim => list.iter().map(|s| s.trim().to_string()).collect(),
            MapOp::Prefix(prefix) => list.iter().map(|s| format!("{}{}", prefix, s)).collect(),
            MapOp::Suffix(suffix) => list.iter().map(|s| format!("{}{}", s, suffix)).collect(),
            MapOp::Identity => list.clone(),
        };

        OutputMap::new().str_list("output", result).ok()
    }
}

/// Filter operation - keep elements matching a predicate.
///
/// Cardinality: ZeroOrMore → ZeroOrMore (may reduce size)
///
/// Inputs:
/// - `input`: List to filter
/// - `pattern`: Optional pattern to match (for Contains, StartsWith, EndsWith)
///
/// Outputs:
/// - `output`: Filtered List
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum FilterOp {
    /// Keep strings containing the pattern
    Contains(String),
    /// Keep strings starting with the pattern
    StartsWith(String),
    /// Keep strings ending with the pattern
    EndsWith(String),
    /// Keep non-empty strings
    NonEmpty,
    /// Keep strings matching exact value
    Equals(String),
    /// Keep all (identity filter)
    #[default]
    All,
}

impl Executable for FilterOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let list = require_str_list(&inputs, "input")?;

        let result: Vec<String> = match self {
            FilterOp::Contains(pattern) => {
                list.iter().filter(|s| s.contains(pattern)).cloned().collect()
            }
            FilterOp::StartsWith(pattern) => list
                .iter()
                .filter(|s| s.starts_with(pattern))
                .cloned()
                .collect(),
            FilterOp::EndsWith(pattern) => {
                list.iter().filter(|s| s.ends_with(pattern)).cloned().collect()
            }
            FilterOp::NonEmpty => list.iter().filter(|s| !s.is_empty()).cloned().collect(),
            FilterOp::Equals(value) => list.iter().filter(|s| *s == value).cloned().collect(),
            FilterOp::All => list.clone(),
        };

        let count = result.len() as i64;
        OutputMap::new()
            .str_list("output", result)
            .int("count", count)
            .ok()
    }
}

/// Fold operation - reduce a list to a single value.
///
/// Cardinality: ZeroOrMore → One
///
/// Inputs:
/// - `input`: List to fold
/// - `initial`: Optional initial value
///
/// Outputs:
/// - `output`: Folded result
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum FoldOp {
    /// Concatenate all strings with separator
    Join(String),
    /// Count elements
    #[default]
    Count,
    /// Sum (for numeric strings)
    Sum,
    /// Find minimum (lexicographic)
    Min,
    /// Find maximum (lexicographic)
    Max,
}

impl Executable for FoldOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let list = require_str_list(&inputs, "input")?;

        let output = match self {
            FoldOp::Join(sep) => Value::Str(list.join(sep)),
            FoldOp::Count => Value::Int(list.len() as i64),
            FoldOp::Sum => {
                let sum: i64 = list
                    .iter()
                    .filter_map(|s| s.parse::<i64>().ok())
                    .sum();
                Value::Int(sum)
            }
            FoldOp::Min => {
                let min = list.iter().min().cloned().unwrap_or_default();
                Value::Str(min)
            }
            FoldOp::Max => {
                let max = list.iter().max().cloned().unwrap_or_default();
                Value::Str(max)
            }
        };

        OutputMap::new().value("output", output).ok()
    }
}

/// Sort operation - order elements.
///
/// Cardinality: ZeroOrMore → ZeroOrMore (preserves count)
///
/// Inputs:
/// - `input`: List to sort
///
/// Outputs:
/// - `output`: Sorted List
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum SortOp {
    /// Sort ascending (lexicographic)
    #[default]
    Ascending,
    /// Sort descending (lexicographic)
    Descending,
    /// Sort by length
    ByLength,
    /// Reverse order (not really sorting, but useful)
    Reverse,
}

impl Executable for SortOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let mut result = require_str_list(&inputs, "input")?;

        match self {
            SortOp::Ascending => result.sort(),
            SortOp::Descending => {
                result.sort();
                result.reverse();
            }
            SortOp::ByLength => result.sort_by_key(|s| s.len()),
            SortOp::Reverse => result.reverse(),
        }

        OutputMap::new().str_list("output", result).ok()
    }
}

/// First operation - extract the first element.
///
/// Cardinality: OneOrMore → One (requires non-empty)
///
/// Inputs:
/// - `input`: List with at least one element
///
/// Outputs:
/// - `output`: First element
/// - `exists`: Bool (true if list was non-empty)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FirstOp;

impl Executable for FirstOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let list = require_str_list(&inputs, "input")?;

        if let Some(first) = list.first() {
            OutputMap::new()
                .str("output", first.clone())
                .bool("exists", true)
                .ok()
        } else {
            OutputMap::new()
                .str("output", String::new())
                .bool("exists", false)
                .ok()
        }
    }
}

/// Last operation - extract the last element.
///
/// Cardinality: OneOrMore → One (requires non-empty)
///
/// Inputs:
/// - `input`: List with at least one element
///
/// Outputs:
/// - `output`: Last element
/// - `exists`: Bool (true if list was non-empty)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LastOp;

impl Executable for LastOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let list = require_str_list(&inputs, "input")?;

        if let Some(last) = list.last() {
            OutputMap::new()
                .str("output", last.clone())
                .bool("exists", true)
                .ok()
        } else {
            OutputMap::new()
                .str("output", String::new())
                .bool("exists", false)
                .ok()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_uppercase() {
        let op = MapOp::ToUppercase;
        let mut inputs = HashMap::new();
        inputs.insert(
            "input".to_string(),
            Value::str_list(vec!["hello".to_string(), "world".to_string()]),
        );

        let result = op.execute(inputs).unwrap();
        assert_eq!(
            result.get("output"),
            Some(&Value::str_list(vec!["HELLO".to_string(), "WORLD".to_string()]))
        );
    }

    #[test]
    fn test_filter_ends_with() {
        let op = FilterOp::EndsWith(".rs".to_string());
        let mut inputs = HashMap::new();
        inputs.insert(
            "input".to_string(),
            Value::str_list(vec![
                "main.rs".to_string(),
                "lib.rs".to_string(),
                "Cargo.toml".to_string(),
            ]),
        );

        let result = op.execute(inputs).unwrap();
        assert_eq!(
            result.get("output"),
            Some(&Value::str_list(vec!["main.rs".to_string(), "lib.rs".to_string()]))
        );
    }

    #[test]
    fn test_fold_count() {
        let op = FoldOp::Count;
        let mut inputs = HashMap::new();
        inputs.insert(
            "input".to_string(),
            Value::str_list(vec!["a".to_string(), "b".to_string(), "c".to_string()]),
        );

        let result = op.execute(inputs).unwrap();
        assert_eq!(result.get("output"), Some(&Value::Int(3)));
    }

    #[test]
    fn test_sort_ascending() {
        let op = SortOp::Ascending;
        let mut inputs = HashMap::new();
        inputs.insert(
            "input".to_string(),
            Value::str_list(vec!["c".to_string(), "a".to_string(), "b".to_string()]),
        );

        let result = op.execute(inputs).unwrap();
        assert_eq!(
            result.get("output"),
            Some(&Value::str_list(vec!["a".to_string(), "b".to_string(), "c".to_string()]))
        );
    }

    #[test]
    fn test_first() {
        let op = FirstOp;
        let mut inputs = HashMap::new();
        inputs.insert(
            "input".to_string(),
            Value::str_list(vec!["first".to_string(), "second".to_string()]),
        );

        let result = op.execute(inputs).unwrap();
        assert_eq!(result.get("output"), Some(&Value::Str("first".to_string())));
        assert_eq!(result.get("exists"), Some(&Value::Bool(true)));
    }

    #[test]
    fn test_first_empty() {
        let op = FirstOp;
        let mut inputs = HashMap::new();
        inputs.insert("input".to_string(), Value::str_list(vec![]));

        let result = op.execute(inputs).unwrap();
        assert_eq!(result.get("exists"), Some(&Value::Bool(false)));
    }
}
