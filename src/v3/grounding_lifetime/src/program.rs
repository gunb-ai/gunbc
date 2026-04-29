//! Structural program slice the analyzer folds over.
//!
//! Populated from `Dag` via [`crate::extract::extract_lifetime_program`] (stub
//! today: empty program when lowering does not yet surface R2 bind graphs).
//! Worked examples 3–4 are encoded as explicit `LifetimeProgram` values in
//! unit tests until extraction is complete.

use std::collections::BTreeMap;

/// Stable key for a analyzed binding within one `LifetimeProgram` snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BindingId(pub u32);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BindingRole {
    /// (a) Top-level `data` binding at module scope — Example 3 `name`.
    TopLevelData,
    /// (b) Function parameter — Example 4 `n`.
    FunctionParameter { function: String },
    /// (c) Value leaving a function via return position.
    FunctionReturn { function: String },
}

/// Structural classification of a single use site (`t-ground-lifetime-analyzer.md` §D).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UseKind {
    /// Read / pass-through only; does not store past callee, escape, or force ownership.
    Transient,
    /// Stores in a binding that outlives the parameter’s call frame, or equivalent escape.
    StoreOrEscape,
    /// Growth / mutating container ops (`.push`, `.append`, …) — forces `Growability::Yes`.
    GrowthMutation,
    /// Forces an exclusive borrow discipline incompatible with `StoreOrEscape` on the same binding.
    ///
    /// Used only to model contradictory-use diagnostics (test plan item 5).
    BorrowExclusive,
    /// Use is visible but does not witness either growth or definite non-growth
    /// (dynamic dispatch / opaque callee — `design-emission-model.md` ~558).
    IndeterminateGrowability,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UseSite {
    pub kind: UseKind,
    /// Diagnostic label (file span wiring is Coercion-Fold / diagnostic renderer).
    pub site_label: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BindingDef {
    pub name: String,
    pub role: BindingRole,
    pub uses: Vec<UseSite>,
}

/// R3 constructs rejected at the analyzer boundary (`design-emission-model.md:635`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum R3Construct {
    Closure,
    Async,
    Pin,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LifetimeProgram {
    pub bindings: BTreeMap<BindingId, BindingDef>,
    pub r3_markers: Vec<R3Construct>,
}

impl LifetimeProgram {
    pub fn empty() -> Self {
        Self {
            bindings: BTreeMap::new(),
            r3_markers: Vec::new(),
        }
    }

    /// Example 3: `data name: String = "Alice"` with no growth / no escape uses.
    pub fn example3_top_level_string_name() -> Self {
        let mut bindings = BTreeMap::new();
        bindings.insert(
            BindingId(0),
            BindingDef {
                name: "name".to_string(),
                role: BindingRole::TopLevelData,
                uses: vec![],
            },
        );
        Self {
            bindings,
            r3_markers: vec![],
        }
    }

    /// Example 4 Case A: transient uses of parameter `n` in `greet`.
    pub fn example4_case_a_param_n_transient() -> Self {
        let mut bindings = BTreeMap::new();
        bindings.insert(
            BindingId(0),
            BindingDef {
                name: "n".to_string(),
                role: BindingRole::FunctionParameter {
                    function: "greet".to_string(),
                },
                uses: vec![UseSite {
                    kind: UseKind::Transient,
                    site_label: "greet.body.transient".to_string(),
                }],
            },
        );
        Self {
            bindings,
            r3_markers: vec![],
        }
    }

    /// Example 4 Case B: parameter stored / escaped → owned.
    pub fn example4_case_b_param_n_stored() -> Self {
        let mut bindings = BTreeMap::new();
        bindings.insert(
            BindingId(0),
            BindingDef {
                name: "n".to_string(),
                role: BindingRole::FunctionParameter {
                    function: "greet".to_string(),
                },
                uses: vec![UseSite {
                    kind: UseKind::StoreOrEscape,
                    site_label: "greet.body.store".to_string(),
                }],
            },
        );
        Self {
            bindings,
            r3_markers: vec![],
        }
    }

    pub fn example_function_return_owned() -> Self {
        let mut bindings = BTreeMap::new();
        bindings.insert(
            BindingId(0),
            BindingDef {
                name: "ret".to_string(),
                role: BindingRole::FunctionReturn {
                    function: "greet".to_string(),
                },
                uses: vec![],
            },
        );
        Self {
            bindings,
            r3_markers: vec![],
        }
    }

    pub fn contradictory_borrow_and_escape() -> Self {
        let mut bindings = BTreeMap::new();
        bindings.insert(
            BindingId(0),
            BindingDef {
                name: "x".to_string(),
                role: BindingRole::FunctionParameter {
                    function: "f".to_string(),
                },
                uses: vec![
                    UseSite {
                        kind: UseKind::BorrowExclusive,
                        site_label: "f.use.borrow".to_string(),
                    },
                    UseSite {
                        kind: UseKind::StoreOrEscape,
                        site_label: "f.use.escape".to_string(),
                    },
                ],
            },
        );
        Self {
            bindings,
            r3_markers: vec![],
        }
    }

    /// Under-determination: growability axis is load-bearing but a use site is indeterminate.
    pub fn underrefined_growability_indeterminate() -> Self {
        let mut bindings = BTreeMap::new();
        bindings.insert(
            BindingId(0),
            BindingDef {
                name: "mystery".to_string(),
                role: BindingRole::TopLevelData,
                uses: vec![UseSite {
                    kind: UseKind::IndeterminateGrowability,
                    site_label: "opaque.call".to_string(),
                }],
            },
        );
        Self {
            bindings,
            r3_markers: vec![],
        }
    }

    pub fn with_r3_construct(c: R3Construct) -> Self {
        Self {
            bindings: BTreeMap::new(),
            r3_markers: vec![c],
        }
    }
}
