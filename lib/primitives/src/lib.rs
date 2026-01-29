//! Fundamental DAG primitives - the leaf operations for all compositions.
//!
//! This crate contains the minimal set of primitive operations that all
//! higher-level DAG compositions build upon. These are the "atoms" of
//! the DAG universe.
//!
//! # Categories
//!
//! - **Data**: Parse, Extract, Format, Concat, Split - pure data transformations
//! - **Collection**: Map, Filter, Fold, Sort, First/Last - list operations with cardinality
//! - **IO**: ReadFile, WriteFile, Execute, HttpRequest - world interactions (boundaries)
//! - **Control**: Loop, Branch - control flow patterns
//!
//! # Design Principles
//!
//! 1. Each primitive is tiny (< 20 lines of execute logic)
//! 2. Primitives are composable via DAG edges
//! 3. Cardinality is used for automatic test generation
//! 4. Boundaries are clearly marked for dry-run interception

pub mod collection;
pub mod control;
pub mod data;
pub mod io;

pub use collection::{CollectionOp, FilterOp, FirstOp, FoldOp, LastOp, MapOp, SortOp};
pub use control::{BranchOp, LoopOp};
pub use data::{ConcatOp, ExtractOp, FormatOp, ParseOp, SplitOp};
pub use io::{ExecuteOp, HttpRequestOp, ListFilesOp, PrepareFileWriteOp, ReadFileOp, ReadFilesOp, WriteFileOp};

use gunbc_exec::{ExecError, Executable};
use gunbc_ir::Value;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// All primitive operations in a single enum for use in DAGs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PrimitiveOp {
    // Data primitives
    Parse(ParseOp),
    Extract(ExtractOp),
    Format(FormatOp),
    Concat(ConcatOp),
    Split(SplitOp),

    // Collection primitives
    Map(MapOp),
    Filter(FilterOp),
    Fold(FoldOp),
    Sort(SortOp),
    First(FirstOp),
    Last(LastOp),

    // I/O primitives (boundaries)
    ReadFile(ReadFileOp),
    ReadFiles(ReadFilesOp),
    ListFiles(ListFilesOp),
    WriteFile(WriteFileOp),
    PrepareFileWrite(PrepareFileWriteOp),
    Execute(ExecuteOp),
    HttpRequest(HttpRequestOp),

    // Control primitives
    Loop(LoopOp),
    Branch(BranchOp),
}

impl Executable for PrimitiveOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            // Data
            PrimitiveOp::Parse(op) => op.execute(inputs),
            PrimitiveOp::Extract(op) => op.execute(inputs),
            PrimitiveOp::Format(op) => op.execute(inputs),
            PrimitiveOp::Concat(op) => op.execute(inputs),
            PrimitiveOp::Split(op) => op.execute(inputs),

            // Collection
            PrimitiveOp::Map(op) => op.execute(inputs),
            PrimitiveOp::Filter(op) => op.execute(inputs),
            PrimitiveOp::Fold(op) => op.execute(inputs),
            PrimitiveOp::Sort(op) => op.execute(inputs),
            PrimitiveOp::First(op) => op.execute(inputs),
            PrimitiveOp::Last(op) => op.execute(inputs),

            // I/O
            PrimitiveOp::ReadFile(op) => op.execute(inputs),
            PrimitiveOp::ReadFiles(op) => op.execute(inputs),
            PrimitiveOp::ListFiles(op) => op.execute(inputs),
            PrimitiveOp::WriteFile(op) => op.execute(inputs),
            PrimitiveOp::PrepareFileWrite(op) => op.execute(inputs),
            PrimitiveOp::Execute(op) => op.execute(inputs),
            PrimitiveOp::HttpRequest(op) => op.execute(inputs),

            // Control
            PrimitiveOp::Loop(op) => op.execute(inputs),
            PrimitiveOp::Branch(op) => op.execute(inputs),
        }
    }
}
