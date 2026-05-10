//! Cross-product cell coordinates: (form × behavior × target).
//!
//! `FormAxis` enumerates the six [`v3_compiler::dag::TypeConnective`]
//! discriminants (substrate-anchored per #1229), `BehaviorAxis` the
//! five [`v3_compiler::dag::Behavior`] variants, and `ShapeATarget`
//! is a typed reference to a target `LanguageSpec` declaration. Each
//! `Cell` is one `(form × behavior × LanguageSpec target)` triple.

use v3_compiler::dag::{Dag, DeclarationId};

/// Form-axis discriminant — mirrors [`v3_compiler::dag::TypeConnective`]
/// variants without carrying their payloads (the cross-product walker
/// only needs the discriminant identity for L6 lookup).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FormAxis {
    Atom,
    Conj,
    Disj,
    Arrow,
    Cardinality,
    Instantiation,
}

impl FormAxis {
    /// All six form variants, in substrate-declaration order
    /// (`src/v3/std/substrate.dag:164`).
    pub const ALL: [FormAxis; 6] = [
        FormAxis::Atom,
        FormAxis::Conj,
        FormAxis::Disj,
        FormAxis::Arrow,
        FormAxis::Cardinality,
        FormAxis::Instantiation,
    ];

    pub fn label(self) -> &'static str {
        match self {
            FormAxis::Atom => "Atom",
            FormAxis::Conj => "Conj",
            FormAxis::Disj => "Disj",
            FormAxis::Arrow => "Arrow",
            FormAxis::Cardinality => "Cardinality",
            FormAxis::Instantiation => "Instantiation",
        }
    }
}

/// Behavior-axis discriminant — mirrors [`v3_compiler::dag::Behavior`]
/// variants without carrying their payloads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BehaviorAxis {
    Value,
    Transform,
    Branch,
    Loop,
    Bind,
}

impl BehaviorAxis {
    /// All five behavior variants, in L1 model order.
    pub const ALL: [BehaviorAxis; 5] = [
        BehaviorAxis::Value,
        BehaviorAxis::Transform,
        BehaviorAxis::Branch,
        BehaviorAxis::Loop,
        BehaviorAxis::Bind,
    ];

    pub fn label(self) -> &'static str {
        match self {
            BehaviorAxis::Value => "Value",
            BehaviorAxis::Transform => "Transform",
            BehaviorAxis::Branch => "Branch",
            BehaviorAxis::Loop => "Loop",
            BehaviorAxis::Bind => "Bind",
        }
    }
}

/// Shape A target identity, backed by the `LanguageSpec` data declaration
/// that owns the target's substrate facts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ShapeATarget {
    pub spec: DeclarationId,
}

impl ShapeATarget {
    pub fn new(spec: DeclarationId) -> ShapeATarget {
        ShapeATarget { spec }
    }

    pub fn label(self, dag: &Dag) -> String {
        let name = dag
            .declaration(self.spec)
            .name
            .as_deref()
            .unwrap_or("<anonymous_language_spec>");
        match name {
            "rust_language" => "Rust".to_string(),
            "python_language" => "Python".to_string(),
            "go_language" => "Go".to_string(),
            other => other.to_string(),
        }
    }
}

/// One cell in the (form × behavior × target) cross product. The
/// L6 walker enumerates all 90 of these and reports per-cell coverage
/// against the LanguageSpec emission-path table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Cell {
    pub connective: FormAxis,
    pub behavior: BehaviorAxis,
    pub target: ShapeATarget,
}

impl Cell {
    /// Iterator over all cells, in nested-loop order
    /// (form outer, behavior middle, target inner).
    pub fn all(targets: &[ShapeATarget]) -> impl Iterator<Item = Cell> + '_ {
        FormAxis::ALL.iter().copied().flat_map(move |connective| {
            BehaviorAxis::ALL.iter().copied().flat_map(move |behavior| {
                targets.iter().copied().map(move |target| Cell {
                    connective,
                    behavior,
                    target,
                })
            })
        })
    }

    /// Stable key for closure-ledger gap rows (`docs/r2-closure-ledger.md`):
    /// `Form_Behavior_Target` using substrate axis labels and LanguageSpec name.
    pub fn ledger_key(self, dag: &Dag) -> String {
        format!(
            "{}_{}_{}",
            self.connective.label(),
            self.behavior.label(),
            self.target.label(dag)
        )
    }
}
