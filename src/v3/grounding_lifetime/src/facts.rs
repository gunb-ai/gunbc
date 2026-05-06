//! `LifetimeFacts` carrier and per-axis sums.
//!
//! # P1 substrate-fact introduction (INVARIANTS.md#p1-modeling-faithfulness)
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
//!
//! ## Practice 4 (`docs/modeling-discipline.md` §4, coproduct checkpoint)
//!
//! Each **multi-variant** `pub enum` below carries a 🟢/🟡 classification in its
//! doc comment (ledger or named dissolution trigger). `Encoding` is currently a
//! single-variant stub (N \< 2); when LanguageSpec adds axis variants, add a
//! checkpoint there too.

/// **Practice 4 — 🟢 GREEN (R2 program-intent slice).** `Owned` / `Borrowed`
/// are the irreducible outcomes this fold emits today; wider ownership lattice
/// (`Conditional`, …) stays on target inhabitance + R3 scope per
/// `design-emission-model.md:635`, not as inert arms here.
///
/// Ownership intent derived from use sites (R2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ownership {
    Owned,
    Borrowed,
}

/// **Practice 4 — 🟢 GREEN (R2 program-intent slice).** `Self_` / `Caller`
/// exhaust lifetime roles derived for (a)–(c) in the brief; `Source` /
/// `Conditional` remain target-side / R3 (`design-emission-model.md:635`).
///
/// Lexical / call structural lifetime scope for the value (R2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LifetimeScope {
    /// Module / static-like duration for top-level data and self-contained returns.
    Self_,
    /// Function parameter bounded by the caller’s frame.
    Caller,
}

/// **Practice 4 — 🟢 GREEN (R2 growability slice).** Yes / No / NotApplicable
/// exhausts growability facts this analyzer derives for the string-family R2
/// tests; container families beyond that wait extraction + LanguageSpec axes.
///
/// Whether a growable container is required (R2 structural use scan).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Growability {
    Yes,
    No,
    NotApplicable,
}

/// Encoding axis — stub until LanguageSpec declares the full vocabulary.
///
/// **Practice 4 — single arm today (fewer than two variants).** When the axis
/// becomes a real multi-variant sum in this crate, add a 🟢/🟡 checkpoint per
/// `modeling-discipline.md` §4.
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
