//! Structural program slice the analyzer folds over.
//!
//! Populated from `Dag` via [`crate::extract::extract_lifetime_program`] (stub
//! today: empty program when lowering does not yet surface R2 bind graphs).
//! Worked examples 3–4 are encoded as explicit `LifetimeProgram` values in
//! unit tests until extraction is complete.
//!
//! ## Practice 4 (`docs/modeling-discipline.md` §4)
//!
//! Multi-variant `pub enum` types in this module carry 🟢/🟡 checkpoints on the
//! type (ledger or named dissolution trigger).

use std::collections::BTreeMap;

/// Stable key for a analyzed binding within one `LifetimeProgram` snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BindingId(pub u32);

/// **Practice 4 — 🟡 YELLOW.** Named trigger: `extract_lifetime_program` + typed
/// binding resolution classify `Unclassified` away; LanguageSpec lane 6 grows
/// the encoding vocabulary this family keys into.
///
/// Structural classification of the binding’s type for encoding / algebra facts.
///
/// Extraction must set this from reflection; **no silent default** to UTF-8 when
/// the algebra is unknown — the fold fails closed on the encoding axis instead.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProgramTypeFamily {
    /// R2 Examples 3–4: `.dag` `String` / UTF-8 `FreeMonoid<Char>`.
    FreeMonoidCharUtf8,
    /// Lowering has not yet classified this binding’s algebra for encoding.
    Unclassified,
}

/// **Practice 4 — 🟡 YELLOW.** Named trigger: lowered `Dag` / declaration surface
/// supplies structural roles (ids + spans), replacing string `function` labels and
/// collapsing this sum toward reflected substrate facts.
///
/// (a) Top-level `data`, (b) function parameter, (c) return position — brief R2 scope.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BindingRole {
    /// (a) Top-level `data` binding at module scope — Example 3 `name`.
    TopLevelData,
    /// (b) Function parameter — Example 4 `n`.
    FunctionParameter { function: String },
    /// (c) Value leaving a function via return position.
    FunctionReturn { function: String },
}

/// **Practice 4 — 🟡 YELLOW.** Named trigger: lowering walks real store/call/return
/// sites into reflected use witnesses; test-only discriminators (`BorrowExclusive`,
/// `IndeterminateGrowability`) dissolve per modeling-discipline §4 patterns once
/// extract is authoritative (not `// scaffold:` — tracked in lane PR / brief).
///
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
    ///
    /// **Not** a substitute for a definite `Transient` use on function parameters:
    /// Case-A `Borrowed` still requires at least one transient witness; opaque
    /// growability alone must not bypass that proof.
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
    pub type_family: ProgramTypeFamily,
}

impl BindingDef {
    /// Well-formed R2 string examples (`FreeMonoid<Char>` UTF-8 is structurally known).
    pub fn r2_string_binding(
        name: impl Into<String>,
        role: BindingRole,
        uses: Vec<UseSite>,
    ) -> Self {
        Self {
            name: name.into(),
            role,
            uses,
            type_family: ProgramTypeFamily::FreeMonoidCharUtf8,
        }
    }
}

/// **Practice 4 — 🟢 GREEN (R2 boundary list).** Labels mirror the director-locked
/// R3 deferral at `design-emission-model.md:635`; each variant names one refused
/// construct — no richer field decomposition at this analyzer boundary.
///
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
            BindingDef::r2_string_binding("name", BindingRole::TopLevelData, vec![]),
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
            BindingDef::r2_string_binding(
                "n",
                BindingRole::FunctionParameter {
                    function: "greet".to_string(),
                },
                vec![UseSite {
                    kind: UseKind::Transient,
                    site_label: "greet.body.transient".to_string(),
                }],
            ),
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
            BindingDef::r2_string_binding(
                "n",
                BindingRole::FunctionParameter {
                    function: "greet".to_string(),
                },
                vec![UseSite {
                    kind: UseKind::StoreOrEscape,
                    site_label: "greet.body.store".to_string(),
                }],
            ),
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
            BindingDef::r2_string_binding(
                "ret",
                BindingRole::FunctionReturn {
                    function: "greet".to_string(),
                },
                vec![],
            ),
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
            BindingDef::r2_string_binding(
                "x",
                BindingRole::FunctionParameter {
                    function: "f".to_string(),
                },
                vec![
                    UseSite {
                        kind: UseKind::BorrowExclusive,
                        site_label: "f.use.borrow".to_string(),
                    },
                    UseSite {
                        kind: UseKind::StoreOrEscape,
                        site_label: "f.use.escape".to_string(),
                    },
                ],
            ),
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
            BindingDef::r2_string_binding(
                "mystery",
                BindingRole::TopLevelData,
                vec![UseSite {
                    kind: UseKind::IndeterminateGrowability,
                    site_label: "opaque.call".to_string(),
                }],
            ),
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
