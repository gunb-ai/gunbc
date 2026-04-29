//! Structural fold: meet use-site constraints into `LifetimeFacts`.

use std::collections::BTreeMap;

use crate::axes::LanguageSpecAxes;
use crate::diagnostic::{EmissionDiagnostic, SiteRef};
use crate::facts::{Encoding, Growability, LifetimeFacts, LifetimeScope, Ownership};
use crate::program::{BindingDef, BindingId, BindingRole, LifetimeProgram, R3Construct, UseKind};

fn encoding_for_binding(_binding: &BindingDef) -> Encoding {
    // R2 Examples 3–4: `.dag` String is UTF-8 `FreeMonoid<Char>`.
    Encoding::Utf8FreeMonoidChar
}

fn ownership_lifetime_growable_for(
    binding: &BindingDef,
    axes: &LanguageSpecAxes,
) -> Result<(Ownership, LifetimeScope, Growability), EmissionDiagnostic> {
    match &binding.role {
        BindingRole::FunctionReturn { .. } => Ok((
            Ownership::Owned,
            LifetimeScope::Self_,
            Growability::NotApplicable,
        )),
        BindingRole::TopLevelData => {
            let mut forces_owned = false;
            let mut forces_borrow_exclusive = false;
            let mut growth = false;
            let mut indeterminate = false;
            for u in &binding.uses {
                match u.kind {
                    UseKind::Transient => {}
                    UseKind::StoreOrEscape | UseKind::GrowthMutation => forces_owned = true,
                    UseKind::BorrowExclusive => forces_borrow_exclusive = true,
                    UseKind::IndeterminateGrowability => indeterminate = true,
                }
                if u.kind == UseKind::GrowthMutation {
                    growth = true;
                }
            }
            if forces_owned && forces_borrow_exclusive {
                return Err(EmissionDiagnostic::ContradictoryUse {
                    binding: binding.name.clone(),
                    sites: binding
                        .uses
                        .iter()
                        .map(|u| SiteRef {
                            label: u.site_label.clone(),
                        })
                        .collect(),
                });
            }
            let growable = if axes.string_growability_axis_load_bearing {
                if indeterminate {
                    return Err(EmissionDiagnostic::UnderRefined {
                        axis: "growability".to_string(),
                    });
                }
                if growth {
                    Growability::Yes
                } else {
                    // No growth witnesses (including the empty-use case) ⇒ non-growable
                    // (`design-emission-model.md:553` — absence is structural).
                    Growability::No
                }
            } else {
                Growability::No
            };
            Ok((Ownership::Owned, LifetimeScope::Self_, growable))
        }
        BindingRole::FunctionParameter { .. } => {
            let mut owned_force = false;
            let mut borrow_excl = false;
            let mut indeterminate = false;
            for u in &binding.uses {
                match u.kind {
                    UseKind::Transient => {}
                    UseKind::BorrowExclusive => {}
                    UseKind::StoreOrEscape | UseKind::GrowthMutation => owned_force = true,
                    UseKind::IndeterminateGrowability => indeterminate = true,
                }
                if u.kind == UseKind::BorrowExclusive {
                    borrow_excl = true;
                }
            }
            if owned_force && borrow_excl {
                return Err(EmissionDiagnostic::ContradictoryUse {
                    binding: binding.name.clone(),
                    sites: binding
                        .uses
                        .iter()
                        .map(|u| SiteRef {
                            label: u.site_label.clone(),
                        })
                        .collect(),
                });
            }
            let ownership = if owned_force {
                Ownership::Owned
            } else {
                Ownership::Borrowed
            };
            let growable = if ownership == Ownership::Borrowed {
                Growability::NotApplicable
            } else if axes.string_growability_axis_load_bearing {
                if indeterminate {
                    return Err(EmissionDiagnostic::UnderRefined {
                        axis: "growability".to_string(),
                    });
                }
                let growth = binding
                    .uses
                    .iter()
                    .any(|u| u.kind == UseKind::GrowthMutation);
                if growth {
                    Growability::Yes
                } else {
                    Growability::No
                }
            } else {
                Growability::No
            };
            Ok((ownership, LifetimeScope::Caller, growable))
        }
    }
}

/// Run the forward borrow-checker-style meet over one program snapshot.
pub fn analyze_lifetime_program(
    program: &LifetimeProgram,
    axes: &LanguageSpecAxes,
) -> Result<BTreeMap<BindingId, LifetimeFacts>, EmissionDiagnostic> {
    if let Some(c) = program.r3_markers.first() {
        return Err(EmissionDiagnostic::OutOfR2Scope {
            construct: r3_construct_name(*c),
        });
    }

    let mut out = BTreeMap::new();
    for (&id, binding) in &program.bindings {
        let encoding = encoding_for_binding(binding);
        let (ownership, lifetime, growable) = ownership_lifetime_growable_for(binding, axes)?;
        out.insert(
            id,
            LifetimeFacts {
                ownership,
                lifetime,
                growable,
                encoding,
            },
        );
    }
    Ok(out)
}

fn r3_construct_name(c: R3Construct) -> String {
    match c {
        R3Construct::Closure => "closure".to_string(),
        R3Construct::Async => "async".to_string(),
        R3Construct::Pin => "pin".to_string(),
    }
}
