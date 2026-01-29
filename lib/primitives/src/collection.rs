//! Collection primitives - list operations with cardinality awareness.
//!
//! These operations work on collections (StrList, Json arrays) and
//! respect cardinality constraints for automatic test generation.

use gunbc_exec::{ExecError, Executable};
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
/// - `input`: StrList to map over
/// - `transform`: String transformation type
///
/// Outputs:
/// - `output`: Transformed StrList
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    Identity,
}

impl Default for MapOp {
    fn default() -> Self {
        MapOp::Identity
    }
}

impl Executable for MapOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let list = inputs
            .get("input")
            .and_then(|v| v.as_str_list())
            .ok_or_else(|| ExecError::new("missing or invalid 'input' string list"))?;

        let result: Vec<String> = match self {
            MapOp::ToUppercase => list.iter().map(|s| s.to_uppercase()).collect(),
            MapOp::ToLowercase => list.iter().map(|s| s.to_lowercase()).collect(),
            MapOp::Trim => list.iter().map(|s| s.trim().to_string()).collect(),
            MapOp::Prefix(prefix) => list.iter().map(|s| format!("{}{}", prefix, s)).collect(),
            MapOp::Suffix(suffix) => list.iter().map(|s| format!("{}{}", s, suffix)).collect(),
            MapOp::Identity => list.clone(),
        };

        let mut out = HashMap::new();
        out.insert("output".to_string(), Value::StrList(result));
        Ok(out)
    }
}

/// Filter operation - keep elements matching a predicate.
///
/// Cardinality: ZeroOrMore → ZeroOrMore (may reduce size)
///
/// Inputs:
/// - `input`: StrList to filter
/// - `pattern`: Optional pattern to match (for Contains, StartsWith, EndsWith)
///
/// Outputs:
/// - `output`: Filtered StrList
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    All,
}

impl Default for FilterOp {
    fn default() -> Self {
        FilterOp::All
    }
}

impl Executable for FilterOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let list = inputs
            .get("input")
            .and_then(|v| v.as_str_list())
            .ok_or_else(|| ExecError::new("missing or invalid 'input' string list"))?;

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
        let mut out = HashMap::new();
        out.insert("output".to_string(), Value::StrList(result));
        out.insert("count".to_string(), Value::Int(count));
        Ok(out)
    }
}

/// Fold operation - reduce a list to a single value.
///
/// Cardinality: ZeroOrMore → One
///
/// Inputs:
/// - `input`: StrList to fold
/// - `initial`: Optional initial value
///
/// Outputs:
/// - `output`: Folded result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FoldOp {
    /// Concatenate all strings with separator
    Join(String),
    /// Count elements
    Count,
    /// Sum (for numeric strings)
    Sum,
    /// Find minimum (lexicographic)
    Min,
    /// Find maximum (lexicographic)
    Max,
}

impl Default for FoldOp {
    fn default() -> Self {
        FoldOp::Count
    }
}

impl Executable for FoldOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let list = inputs
            .get("input")
            .and_then(|v| v.as_str_list())
            .ok_or_else(|| ExecError::new("missing or invalid 'input' string list"))?;

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

        let mut out = HashMap::new();
        out.insert("output".to_string(), output);
        Ok(out)
    }
}

/// Sort operation - order elements.
///
/// Cardinality: ZeroOrMore → ZeroOrMore (preserves count)
///
/// Inputs:
/// - `input`: StrList to sort
///
/// Outputs:
/// - `output`: Sorted StrList
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortOp {
    /// Sort ascending (lexicographic)
    Ascending,
    /// Sort descending (lexicographic)
    Descending,
    /// Sort by length
    ByLength,
    /// Reverse order (not really sorting, but useful)
    Reverse,
}

impl Default for SortOp {
    fn default() -> Self {
        SortOp::Ascending
    }
}

impl Executable for SortOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let list = inputs
            .get("input")
            .and_then(|v| v.as_str_list())
            .ok_or_else(|| ExecError::new("missing or invalid 'input' string list"))?;

        let mut result = list.clone();
        match self {
            SortOp::Ascending => result.sort(),
            SortOp::Descending => {
                result.sort();
                result.reverse();
            }
            SortOp::ByLength => result.sort_by_key(|s| s.len()),
            SortOp::Reverse => result.reverse(),
        }

        let mut out = HashMap::new();
        out.insert("output".to_string(), Value::StrList(result));
        Ok(out)
    }
}

/// First operation - extract the first element.
///
/// Cardinality: OneOrMore → One (requires non-empty)
///
/// Inputs:
/// - `input`: StrList with at least one element
///
/// Outputs:
/// - `output`: First element
/// - `exists`: Bool (true if list was non-empty)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FirstOp;

impl Executable for FirstOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let list = inputs
            .get("input")
            .and_then(|v| v.as_str_list())
            .ok_or_else(|| ExecError::new("missing or invalid 'input' string list"))?;

        let mut out = HashMap::new();
        if let Some(first) = list.first() {
            out.insert("output".to_string(), Value::Str(first.clone()));
            out.insert("exists".to_string(), Value::Bool(true));
        } else {
            out.insert("output".to_string(), Value::Str(String::new()));
            out.insert("exists".to_string(), Value::Bool(false));
        }
        Ok(out)
    }
}

/// Last operation - extract the last element.
///
/// Cardinality: OneOrMore → One (requires non-empty)
///
/// Inputs:
/// - `input`: StrList with at least one element
///
/// Outputs:
/// - `output`: Last element
/// - `exists`: Bool (true if list was non-empty)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LastOp;

impl Executable for LastOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let list = inputs
            .get("input")
            .and_then(|v| v.as_str_list())
            .ok_or_else(|| ExecError::new("missing or invalid 'input' string list"))?;

        let mut out = HashMap::new();
        if let Some(last) = list.last() {
            out.insert("output".to_string(), Value::Str(last.clone()));
            out.insert("exists".to_string(), Value::Bool(true));
        } else {
            out.insert("output".to_string(), Value::Str(String::new()));
            out.insert("exists".to_string(), Value::Bool(false));
        }
        Ok(out)
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
            Value::StrList(vec!["hello".to_string(), "world".to_string()]),
        );

        let result = op.execute(inputs).unwrap();
        assert_eq!(
            result.get("output"),
            Some(&Value::StrList(vec!["HELLO".to_string(), "WORLD".to_string()]))
        );
    }

    #[test]
    fn test_filter_ends_with() {
        let op = FilterOp::EndsWith(".rs".to_string());
        let mut inputs = HashMap::new();
        inputs.insert(
            "input".to_string(),
            Value::StrList(vec![
                "main.rs".to_string(),
                "lib.rs".to_string(),
                "Cargo.toml".to_string(),
            ]),
        );

        let result = op.execute(inputs).unwrap();
        assert_eq!(
            result.get("output"),
            Some(&Value::StrList(vec!["main.rs".to_string(), "lib.rs".to_string()]))
        );
    }

    #[test]
    fn test_fold_count() {
        let op = FoldOp::Count;
        let mut inputs = HashMap::new();
        inputs.insert(
            "input".to_string(),
            Value::StrList(vec!["a".to_string(), "b".to_string(), "c".to_string()]),
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
            Value::StrList(vec!["c".to_string(), "a".to_string(), "b".to_string()]),
        );

        let result = op.execute(inputs).unwrap();
        assert_eq!(
            result.get("output"),
            Some(&Value::StrList(vec!["a".to_string(), "b".to_string(), "c".to_string()]))
        );
    }

    #[test]
    fn test_first() {
        let op = FirstOp;
        let mut inputs = HashMap::new();
        inputs.insert(
            "input".to_string(),
            Value::StrList(vec!["first".to_string(), "second".to_string()]),
        );

        let result = op.execute(inputs).unwrap();
        assert_eq!(result.get("output"), Some(&Value::Str("first".to_string())));
        assert_eq!(result.get("exists"), Some(&Value::Bool(true)));
    }

    #[test]
    fn test_first_empty() {
        let op = FirstOp;
        let mut inputs = HashMap::new();
        inputs.insert("input".to_string(), Value::StrList(vec![]));

        let result = op.execute(inputs).unwrap();
        assert_eq!(result.get("exists"), Some(&Value::Bool(false)));
    }
}
