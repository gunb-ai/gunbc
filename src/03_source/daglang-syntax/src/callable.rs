//! Shared trait for callable item types (FnDef, FuncDef, PatternDef).
//!
//! Eliminates per-variant match arms in lowerer helpers where all three
//! callable kinds are treated identically.

use crate::ast::{FnDef, FuncDef, Item, Param, PatternDef, ProvidesClause, Stmt, UsesClause};

/// Common accessors for callable item definitions.
pub trait CallableItemExt {
    fn name(&self) -> &str;
    fn params(&self) -> &[Param];
    fn body_stmts(&self) -> &[Stmt];
    fn uses_clauses(&self) -> &[UsesClause];
    fn provides_clauses(&self) -> &[ProvidesClause];
}

impl CallableItemExt for FnDef {
    fn name(&self) -> &str {
        &self.name
    }
    fn params(&self) -> &[Param] {
        &self.params
    }
    fn body_stmts(&self) -> &[Stmt] {
        &self.body.stmts
    }
    fn uses_clauses(&self) -> &[UsesClause] {
        &[]
    }
    fn provides_clauses(&self) -> &[ProvidesClause] {
        &[]
    }
}

impl CallableItemExt for FuncDef {
    fn name(&self) -> &str {
        &self.name
    }
    fn params(&self) -> &[Param] {
        &self.params
    }
    fn body_stmts(&self) -> &[Stmt] {
        &self.body.stmts
    }
    fn uses_clauses(&self) -> &[UsesClause] {
        &self.uses
    }
    fn provides_clauses(&self) -> &[ProvidesClause] {
        &self.provides
    }
}

impl CallableItemExt for PatternDef {
    fn name(&self) -> &str {
        &self.name
    }
    fn params(&self) -> &[Param] {
        &self.params
    }
    fn body_stmts(&self) -> &[Stmt] {
        &self.body.stmts
    }
    fn uses_clauses(&self) -> &[UsesClause] {
        &self.uses
    }
    fn provides_clauses(&self) -> &[ProvidesClause] {
        &self.provides
    }
}

impl Item {
    /// Returns the callable trait object if this item is a callable (FnDef, FuncDef, PatternDef).
    pub fn as_callable(&self) -> Option<&dyn CallableItemExt> {
        match self {
            Item::FnDef(def) => Some(def),
            Item::FuncDef(def) => Some(def),
            Item::PatternDef(def) => Some(def),
            _ => None,
        }
    }
}
