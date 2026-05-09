//! Cross-product cell coordinates: (form × behavior × target).
//!
//! `FormAxis` enumerates the six [`v3_compiler::dag::TypeConnective`]
//! discriminants (substrate-anchored per #1229), `BehaviorAxis` the
//! five [`v3_compiler::dag::Behavior`] variants, and `ShapeATarget`
//! the three Shape A targets (Rust / Python / Go). Each `Cell` is one
//! of the 90 `(form × behavior × target)` triples.

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

/// Shape A target identity. Three concrete targets per
/// `r2-grounding-manager.md`'s portability requirements
/// (target-side primitive declarations for Rust / Python / Go).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShapeATarget {
    Rust,
    Python,
    Go,
}

impl ShapeATarget {
    /// All three Shape A targets.
    pub const ALL: [ShapeATarget; 3] = [ShapeATarget::Rust, ShapeATarget::Python, ShapeATarget::Go];

    pub fn label(self) -> &'static str {
        match self {
            ShapeATarget::Rust => "Rust",
            ShapeATarget::Python => "Python",
            ShapeATarget::Go => "Go",
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
    /// Iterator over all 90 cells, in nested-loop order
    /// (form outer, behavior middle, target inner).
    pub fn all() -> impl Iterator<Item = Cell> {
        FormAxis::ALL.iter().copied().flat_map(|connective| {
            BehaviorAxis::ALL.iter().copied().flat_map(move |behavior| {
                ShapeATarget::ALL.iter().copied().map(move |target| Cell {
                    connective,
                    behavior,
                    target,
                })
            })
        })
    }

    /// Stable key for closure-ledger gap rows (`docs/r2-closure-ledger.md`):
    /// `Form_Behavior_Target` using substrate axis labels.
    pub fn ledger_key(self) -> String {
        format!(
            "{}_{}_{}",
            self.connective.label(),
            self.behavior.label(),
            self.target.label()
        )
    }
}
