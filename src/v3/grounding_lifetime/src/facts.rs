//! `LifetimeFacts` carrier and per-axis sums.
//!
//! # P1 substrate-fact introduction (INVARIANTS.md §P1)
//!
//! **Step 1 (DAG-ancestor):** These types are new lane-local carriers for
//! program-side intent that mirror the worked-example substrate rows in
//! `docs/design-emission-model.md:534-546` (`ownership`, `growable`,
//! `lifetime` on target inhabitants). They do not duplicate an existing
//! single `LifetimeFacts` declaration in `dsl/std/`; the fold consumes facts
//! keyed per **program binding** (parallel authority to target-side
//! `inhabits` rows, not a sibling of `Declaration`). Attachment target for
//! a future reflected substrate is **per binding id** alongside lowering
//! output (default position in the brief: structural fact on the binding).
//!
//! **Step 2 (Coproduct-vs-coordinate):** `LifetimeFacts` is a **record**
//! (four axes are coordinates, all required per binding). Each axis field
//! is a **sum** (`Ownership`, `LifetimeScope`, `Growability`, `Encoding`):
//! one alternative at a time per axis. The **target** substrate rows at
//! `design-emission-model.md:534-546` name additional alternatives (`Conditional`,
//! `Source`, …) on the inhabitance side; this **program** carrier only carries
//! variants the R2 fold can emit today (`Owned` / `Borrowed`; `Self_` / `Caller`)
//! so the sum stays aligned with production paths (CODING.md — no inert arms).
//!
//! **Step 3 (Primitive-vs-lens-extensible):** `Ownership`, `LifetimeScope`,
//! and `Growability` are substrate-primitive axes (every Shape-A target
//! carries them computationally). `Encoding` is **lens-extensible** via
//! `LanguageSpec` vocabulary once lane 6 lands; until then the analyzer
//! carries a closed stub (`Encoding`) sufficient for `FreeMonoid<Char>` /
//! UTF-8 parity in Examples 3–4.

/// Ownership intent derived from use sites (R2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ownership {
    Owned,
    Borrowed,
}

/// Lexical / call structural lifetime scope for the value (R2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LifetimeScope {
    /// Module / static-like duration for top-level data and self-contained returns.
    Self_,
    /// Function parameter bounded by the caller’s frame.
    Caller,
}

/// Whether a growable container is required (R2 structural use scan).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Growability {
    Yes,
    No,
    NotApplicable,
}

/// Encoding axis — stub until LanguageSpec declares the full vocabulary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Encoding {
    /// `.dag` `String` / `FreeMonoid<Char>` UTF-8 sequence (Examples 3–4).
    Utf8FreeMonoidChar,
}

/// Per-binding facts the Coercion-Fold consumes (`t-ground-lifetime-analyzer.md` §C).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LifetimeFacts {
    pub ownership: Ownership,
    pub lifetime: LifetimeScope,
    pub growable: Growability,
    pub encoding: Encoding,
}
