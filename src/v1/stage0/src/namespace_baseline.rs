//! BASE-SIDE RECONSTRUCTION OVER A DIFF WINDOW: the base revision's declaration index, derived
//! from the head index with the diff applied in reverse at file grain, plus the
//! dependents-direction selector the required floor plans from.
//!
//! WHO CONSUMES THIS. The required floor's planning row (`run_required_floor`) reconstructs the
//! base side of its own comparison window through `reconstruct_base_index` and selects the
//! untouched consumers of declarations whose interface changed through
//! `interface_changed_consumers` (gunbc#11194, #12120). `joint_claim_join` re-reads base-side sources
//! through `base_records` and splits a diff through `diff_sides`. `behavioral_receipt_host` runs
//! git through `git_stdout`.
//!
//! WHERE THIS MODULE CAME FROM. It is the non-gate residue of `namespace_wave_admission.rs`,
//! deleted with the wave-admission wall by operator ruling 2026-09-19 (the drop is declared at
//! `gunbc.rung_drop` `namespace_wave_admission_wall_removed`). The wall adjudicated closure,
//! membership and binding deltas on the merge path; what survives here is the acquisition and
//! baseline-reconstruction machinery the floor and the claim join used beside it, renamed so no
//! live identifier names the deleted gate. The parse-environment-of-a-revision loader at the
//! bottom survives for the same reason: the floor's base side must be read under the BASE's
//! grammar, and that requirement does not leave with the wall.

// CLIPPY ROSTER -- the finding(s) this module trips today, listed one lint per line with
// its count. Until the generated crate root stopped allowing `clippy::all` on behalf of every
// module under it, `cargo clippy --all-targets -- -D warnings` decided nothing here; the root
// now excuses only the generated modules it speaks for (v1.compiler.emit_rust
// generated_rust_lint_relaxations), and this is what that leaves visible. The list is MONOTONE
// NON-INCREASING: a name leaves when its last site is repaired, and a lint not named below reds
// the build, which is the whole point.

#![allow(
    clippy::manual_contains,  // 1
)]

use std::collections::BTreeSet;
use std::path::Path;
use std::process::Command;

use crate::cli_run::declaration_index::{
    import_surface_has, index_get, index_insert, index_records, record_from_module,
    DeclarationIndex, ModuleDeclarationRecord,
};
use crate::cli_run::DAG_PARSE_SWEEP_ROOTS;
use crate::v1_std_core::qualified_last_segment;

/// The declaration identity of an ambient kernel type in binding rows.
///
/// Kernel types have no declaring module, but that does not make them unresolved:
/// `v1.compiler.resolve` appends `std.types.kernel_type_set` to every visible-name set. Keeping
/// this identity distinct from every module path prevents an empty candidate set from conflating
/// "resolved by the kernel" with "denotes nothing".
const KERNEL_DECLARATION_IDENTITY: &str = "<kernel>";

/// The longest dotted prefix of `spelling` that is a module in the index, with the segment
/// that follows it. `None` when no prefix names a module — a host name, a kernel name, or an
/// ordinary field access on a value.
fn module_prefix_of(index: &DeclarationIndex, spelling: &str) -> Option<(String, String)> {
    let segments: Vec<&str> = spelling.split('.').collect();
    if segments.len() < 2 {
        return None;
    }
    // Longest first: `a.b.c` prefers module `a.b` over module `a`.
    for split in (1..segments.len()).rev() {
        let candidate = segments[..split].join(".");
        if index_get(index, &candidate).is_some() {
            return Some((candidate, segments[split].to_string()));
        }
    }
    None
}

/// The declaring identities a spelling admits inside one module — module paths for authored
/// declarations and `<kernel>` for an ambient kernel type. An empty set is
/// unresolved-at-this-grain; two or more is ambiguity.
fn declaring_candidates(
    index: &DeclarationIndex,
    record: &ModuleDeclarationRecord,
    spelling: &str,
) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    if let Some((module, leaf)) = module_prefix_of(index, spelling) {
        if let Some(target) = index_get(index, &module) {
            if import_surface_has(target, &leaf) {
                out.insert(declarer_of(index, &module, &leaf));
            }
        }
        return out;
    }
    if spelling.contains('.') {
        return out;
    }
    let locally_declared = record.declared.contains(spelling) || record.variants.contains(spelling);
    if locally_declared {
        out.insert(record.module_path.clone());
    }
    // Locals precede kernel names, and kernel names precede imports in the compiler's one
    // precedence authority. A structural `String` import therefore never makes bare `String`
    // denote that module: both with and without the import it denotes the ambient kernel type.
    // The early return is the discriminator the previous module-only set lacked.
    if !locally_declared && crate::std_types::kernel_type_set().contains_key(spelling) {
        out.insert(KERNEL_DECLARATION_IDENTITY.to_string());
        return out;
    }
    for claim in &record.imports {
        let Some(target) = index_get(index, &claim.target) else {
            continue;
        };
        // An `import m` with no member list exposes the target's whole surface; a member
        // list exposes exactly what it names.
        let claimed = if claim.members.is_empty() {
            import_surface_has(target, spelling)
        } else {
            claim.members.iter().any(|(m, _)| m == spelling)
        };
        if claimed && import_surface_has(target, spelling) {
            out.insert(declarer_of(index, &claim.target, spelling));
        }
    }
    out
}

/// Where a name reached through `module` is actually DECLARED. A re-export names the wrong
/// authority (DESIGN §3 — a fact's home is its declaring module), so the chain is followed to
/// the declarer, bounded by a visited set so a re-export cycle terminates at the last module
/// reached.
fn declarer_of(index: &DeclarationIndex, module: &str, name: &str) -> String {
    let mut current = module.to_string();
    let mut seen: BTreeSet<String> = BTreeSet::new();
    loop {
        if !seen.insert(current.clone()) {
            return current;
        }
        let Some(record) = index_get(index, &current) else {
            return current;
        };
        if record.declared.contains(name) || record.variants.contains(name) {
            return current;
        }
        let next = record.imports.iter().find_map(|claim| {
            let target = index_get(index, &claim.target)?;
            let claimed = if claim.members.is_empty() {
                import_surface_has(target, name)
            } else {
                claim.members.iter().any(|(m, _)| m == name)
            };
            if claimed {
                Some(claim.target.clone())
            } else {
                None
            }
        });
        match next {
            Some(n) => current = n,
            None => return current,
        }
    }
}

// ---------------------------------------------------------------------------
// THE DEPENDENTS DIRECTION — untouched consumers of a declaration whose INTERFACE changed
// ---------------------------------------------------------------------------

/// WHY A DECLARATION'S INTERFACE CHANGED between the base and head indexes. One change type
/// with its grounds as arms, not one selector per kind of change: a coproduct growing an arm is
/// one way a declaration's interface moves, a product field retyped or a function parameter
/// retyped is another, and each strands the SAME population -- the untouched modules that
/// reference the declaration (gunbc#11194 found the first; #11751 -> #12120 the second).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum InterfaceChangeGround {
    /// A coproduct GREW arms and nothing else moved: no arm removed, every surviving arm's payload
    /// unchanged, and the residual interface outside the arms (name, generic parameters) unchanged. A constructor or a projection cannot be stranded by growth; only an exhaustive
    /// `match` can, so this ground plans MATCH READERS alone (gunbc#11194's original population).
    ArmSetGrown { arms_added: Vec<String> },
    /// A coproduct's arm set differs AND growth alone does not describe it -- an arm removed, or
    /// a surviving arm's payload changed beside the growth -- so every reader is planned. Carried
    /// apart from `SignatureChanged` because the receipt names the arms, and because a read that
    /// names an arm (and not the type) is a consumer.
    ArmSetChanged {
        arms_added: Vec<String>,
        arms_removed: Vec<String>,
    },
    /// The declaration's interface text (`ModuleDeclarationRecord::declaration_interfaces`,
    /// the declaration with its body elided) differs: a field type, a parameter, a result, a
    /// constructor payload, an alias target, a brand.
    SignatureChanged,
    /// A function's `admit_callers:` roster lost entries while its interface text stayed put.
    /// Growing a roster strands nobody; narrowing it strands exactly the callers it dropped, so
    /// this ground plans THOSE modules and no other reader -- the difference between planning one
    /// module and every caller of a widely called constructor.
    AdmittedCallersNarrowed {
        removed_callers: Vec<(String, String)>,
    },
    /// A function that admitted ANY caller now carries an `admit_callers:` roster. Every reader
    /// the roster does not name is stranded; the ones it names are not.
    AdmissionIntroduced {
        admitted_callers: Vec<(String, String)>,
    },
    /// The declaration exists at base in this module and not at head. A reader that still
    /// names it is exactly as stale as one reading a retyped field.
    DeclarationRemoved,
    /// The declaration's own text is unchanged, but its interface REFERENCES a declaration whose
    /// interface changed (a product whose field is an alias that was re-branded). Its consumers
    /// are planned because its interface changed through that reference -- and ONLY then: a
    /// module that merely references a changed declaration from a body is planned itself and
    /// propagates no further.
    PropagatedThrough {
        module_path: String,
        declaration: String,
    },
}

/// One declaration whose interface differs between the base and head indexes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DeclarationInterfaceChange {
    pub module_path: String,
    pub declaration: String,
    pub ground: InterfaceChangeGround,
}

/// How a consumer's read was bound to the changed declaration, carried so the receipt can
/// name the two populations apart: a read whose candidate set names the declaring module, and a
/// bare read whose candidate set is EMPTY at this grain -- the flat last-writer-wins channel the
/// namespace cut is retiring. The second is planned too (it is a consumer in the compiler's
/// eyes, and a missed one is exactly the silent class this selector closes), but it is counted
/// under its own name so the deficit stays visible instead of being absorbed into the answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum InterfaceConsumerBinding {
    BoundToDeclaringModule,
    BoundThroughFlatBareChannel,
}

/// One untouched module that names a changed declaration (or, for a coproduct, one of its
/// arms), with the declaring module that name resolved to and the declarations in the consumer
/// that carry the read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct InterfaceChangedConsumer {
    pub changed_module_path: String,
    pub changed_declaration: String,
    pub consumer_module_path: String,
    pub consumer_rel_path: String,
    pub in_declarations: Vec<String>,
    pub binding: InterfaceConsumerBinding,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct InterfaceConsumerSelection {
    pub changes: Vec<DeclarationInterfaceChange>,
    pub consumers: Vec<InterfaceChangedConsumer>,
}

/// The declarations whose interface differs between one module's base and head records -- the
/// DIRECT changes, before propagation. A module absent at head has every base declaration
/// removed; a module absent at base is new and has no consumer that predates it.
fn direct_interface_changes(
    base_record: &ModuleDeclarationRecord,
    head_record: Option<&ModuleDeclarationRecord>,
) -> Vec<DeclarationInterfaceChange> {
    let mut out = Vec::new();
    for (declaration, base_text) in &base_record.declaration_interfaces {
        let change = |ground| DeclarationInterfaceChange {
            module_path: base_record.module_path.clone(),
            declaration: declaration.clone(),
            ground,
        };
        let Some(head_record) = head_record else {
            out.push(change(InterfaceChangeGround::DeclarationRemoved));
            continue;
        };
        let Some(head_text) = head_record.declaration_interfaces.get(declaration) else {
            out.push(change(InterfaceChangeGround::DeclarationRemoved));
            continue;
        };
        let base_arms = base_record.coproduct_arms.get(declaration);
        let head_arms = head_record.coproduct_arms.get(declaration);
        if let (Some(base_arms), Some(head_arms)) = (base_arms, head_arms) {
            if base_arms != head_arms {
                let arms_added: Vec<String> = head_arms.difference(base_arms).cloned().collect();
                let arms_removed: Vec<String> = base_arms.difference(head_arms).cloned().collect();
                let survivors_unchanged = base_arms.intersection(head_arms).all(|arm| {
                    let key = (declaration.clone(), arm.clone());
                    base_record.arm_interfaces.get(&key) == head_record.arm_interfaces.get(&key)
                });
                let residual_unchanged = base_record.coproduct_residuals.get(declaration)
                    == head_record.coproduct_residuals.get(declaration);
                out.push(change(
                    if arms_removed.is_empty() && survivors_unchanged && residual_unchanged {
                        InterfaceChangeGround::ArmSetGrown { arms_added }
                    } else {
                        InterfaceChangeGround::ArmSetChanged {
                            arms_added,
                            arms_removed,
                        }
                    },
                ));
                continue;
            }
        }
        if base_text != head_text {
            out.push(change(InterfaceChangeGround::SignatureChanged));
            continue;
        }
        // THE FOUR ROSTER TRANSITIONS, decided on presence first and difference second:
        //   absent  -> present  narrows "anyone" to the roster: every NON-admitted reader;
        //   present -> absent   widens to anyone: nobody;
        //   present -> larger   widens: nobody;
        //   present -> smaller  narrows: exactly the dropped callers.
        // Taking `base - head` before asking whether head HAS a roster would read a deleted
        // roster as narrowing every entry away -- the inverse of what it does.
        match (
            base_record.admitted_callers.get(declaration),
            head_record.admitted_callers.get(declaration),
        ) {
            (None, Some(admitted)) => {
                out.push(change(InterfaceChangeGround::AdmissionIntroduced {
                    admitted_callers: admitted.iter().cloned().collect(),
                }));
            }
            (Some(base_admitted), Some(head_admitted)) => {
                let removed_callers: Vec<(String, String)> =
                    base_admitted.difference(head_admitted).cloned().collect();
                if !removed_callers.is_empty() {
                    out.push(change(InterfaceChangeGround::AdmittedCallersNarrowed {
                        removed_callers,
                    }));
                }
            }
            (Some(_), None) | (None, None) => {}
        }
    }
    out
}

/// Whether `spelling`, read inside `record`, binds to `(module_path, declaration)` on either
/// side, and how. `None` is "not a read of this declaration": another declarer owns the
/// spelling, or the leaf is some other name.
fn read_binding(
    base: &DeclarationIndex,
    head: &DeclarationIndex,
    record: &ModuleDeclarationRecord,
    spelling: &str,
    module_path: &str,
    universe: &BTreeSet<String>,
) -> Option<InterfaceConsumerBinding> {
    let leaf = qualified_last_segment(spelling.to_string());
    if !universe.contains(&leaf) {
        return None;
    }
    let mut candidates = declaring_candidates(head, record, spelling);
    candidates.extend(declaring_candidates(base, record, spelling));
    if candidates.contains(module_path) {
        Some(InterfaceConsumerBinding::BoundToDeclaringModule)
    } else if candidates.is_empty() {
        Some(InterfaceConsumerBinding::BoundThroughFlatBareChannel)
    } else {
        // Bound to another declarer of a same-spelled name: not this declaration's consumer.
        None
    }
}

/// The names a read of the changed declaration can spell: the declaration itself and, for a
/// coproduct, every arm on either side -- a constructor call and a match arm both name an arm
/// without naming the type.
fn change_universe(
    base: &DeclarationIndex,
    head: &DeclarationIndex,
    change: &DeclarationInterfaceChange,
) -> BTreeSet<String> {
    let mut universe: BTreeSet<String> = BTreeSet::new();
    universe.insert(change.declaration.clone());
    for index in [base, head] {
        if let Some(record) = index_get(index, &change.module_path) {
            if let Some(arms) = record.coproduct_arms.get(&change.declaration) {
                universe.extend(arms.iter().cloned());
            }
        }
    }
    universe
}

/// THE SELECTOR THE REQUIRED FLOOR'S PLANNING ROW CONSUMES, derived from declarations and
/// never from paths or names (DESIGN §3c: a declaration's consumers are a fact the namespace
/// tree carries; the planned set is producer-derived, never a path filter).
///
/// THE CLASS. A declaration's interface changes in one module -- a coproduct grows an arm
/// (gunbc#11194), a product field is retyped, an alias is re-branded, a parameter or result is
/// retyped (#11751, whose stranded witness #12120 repaired) -- and a module that reads it has an
/// empty diff, so no diff-keyed selector sees it and the required floor never Strict-prepares
/// it. This is the DEPENDENTS direction; `touched_entry_files` seeding is the DEPENDENCY
/// direction, and neither closes the class alone. `gunbc.recurring_failure_mode`
/// `changed_declaration_signature_consumer_unplanned` is the row.
///
/// THE CHANGED SET. Direct changes are read per module (`direct_interface_changes`), then
/// closed under ONE propagation rule: a declaration whose INTERFACE references a changed
/// declaration (`interface_references`, the parser's type occurrences outside any body) has
/// itself changed, through that reference. Body references never propagate: a function whose
/// body alone reads a changed type is a consumer and is planned, and its callers are not --
/// that is what keeps a body-only change from planning every reverse importer.
///
/// THE CONSUMERS. A module is a consumer of a changed declaration when one of its reads
/// (`referenced`, `authored_type_references`, `called_occurrences`, `value_occurrences`,
/// `matched_arms`) spells the declaration or one of
/// its arms and `declaring_candidates` for that spelling includes the declaring module on
/// EITHER side (a read of a REMOVED name has no head-side candidate; its base-side one names
/// the declarer exactly). No second consumer relation is minted: this is `declaring_candidates`
/// asked one more question. NOT every importer: an import that is never read is not a consumer.
///
/// WHAT IS NOT SELECTED, deliberately: the declaring module itself (its own file is in the diff,
/// or -- for a propagated change -- it was planned as a consumer of the change it propagates);
/// a declaration that is NEW at head; and a read whose candidate set names a DIFFERENT declarer.
pub(crate) fn interface_changed_consumers(
    base: &DeclarationIndex,
    head: &DeclarationIndex,
) -> InterfaceConsumerSelection {
    let mut changes: Vec<DeclarationInterfaceChange> = Vec::new();
    for base_record in index_records(base) {
        changes.extend(direct_interface_changes(
            base_record,
            index_get(head, &base_record.module_path),
        ));
    }
    // PROPAGATION, a fixpoint over head interfaces. Bounded: each round adds at least one
    // (module, declaration) pair not yet in `seen`, and the pairs are finite.
    let mut seen: BTreeSet<(String, String)> = changes
        .iter()
        .map(|c| (c.module_path.clone(), c.declaration.clone()))
        .collect();
    let mut frontier: Vec<DeclarationInterfaceChange> = changes.clone();
    while !frontier.is_empty() {
        let mut next: Vec<DeclarationInterfaceChange> = Vec::new();
        for change in &frontier {
            // A narrowed admission changes who may call, not what the declaration's type is:
            // nothing whose interface spells it has changed.
            // Pure growth reaches only matches, and a match names the grown coproduct's arms
            // directly wherever it sits, so nothing propagates through a carrier of it either.
            if matches!(
                change.ground,
                InterfaceChangeGround::AdmittedCallersNarrowed { .. }
                    | InterfaceChangeGround::AdmissionIntroduced { .. }
                    | InterfaceChangeGround::ArmSetGrown { .. }
            ) {
                continue;
            }
            let universe = change_universe(base, head, change);
            for record in index_records(head) {
                for (in_declaration, spelling) in &record.interface_references {
                    if seen.contains(&(record.module_path.clone(), in_declaration.clone())) {
                        continue;
                    }
                    // A FLAT-CHANNEL interface read propagates too. `interface_references` are the
                    // parser's own type occurrences, so an empty candidate set there is a genuine
                    // read the compiler resolves last-writer-wins -- the same read that plans its
                    // module. Refusing to propagate it would plan B and leave B's readers C
                    // unplanned while the population read as closed.
                    if read_binding(base, head, record, spelling, &change.module_path, &universe)
                        .is_none()
                    {
                        continue;
                    }
                    seen.insert((record.module_path.clone(), in_declaration.clone()));
                    next.push(DeclarationInterfaceChange {
                        module_path: record.module_path.clone(),
                        declaration: in_declaration.clone(),
                        ground: InterfaceChangeGround::PropagatedThrough {
                            module_path: change.module_path.clone(),
                            declaration: change.declaration.clone(),
                        },
                    });
                }
            }
        }
        changes.extend(next.iter().cloned());
        frontier = next;
    }

    let mut consumers: Vec<InterfaceChangedConsumer> = Vec::new();
    for change in &changes {
        let universe = change_universe(base, head, change);
        let admitted_only: Option<BTreeSet<&str>> = match &change.ground {
            InterfaceChangeGround::AdmittedCallersNarrowed { removed_callers } => Some(
                removed_callers
                    .iter()
                    .map(|(module, _)| module.as_str())
                    .collect(),
            ),
            _ => None,
        };
        for consumer in index_records(head) {
            if consumer.module_path == change.module_path {
                continue;
            }
            if admitted_only
                .as_ref()
                .is_some_and(|modules| !modules.contains(consumer.module_path.as_str()))
            {
                continue;
            }
            let mut in_declarations: BTreeSet<String> = BTreeSet::new();
            let mut binding: Option<InterfaceConsumerBinding> = None;
            // `referenced` over-collects binders and labels (see its field note), so a read
            // there binds only when it RESOLVES to the declarer; the flat bare channel is
            // admitted only from the parser's own type occurrences and match-arm heads, where a
            // spelling is a genuine read. Otherwise every module with a local named like a
            // changed function would be planned -- the widening this selector must not do.
            let matches_only = matches!(change.ground, InterfaceChangeGround::ArmSetGrown { .. });
            let reads = consumer
                .referenced
                .iter()
                .map(|r| (r, false))
                .chain(consumer.authored_type_references.iter().map(|r| (r, true)))
                .chain(consumer.called_occurrences.iter().map(|r| (r, true)))
                .chain(consumer.value_occurrences.iter().map(|r| (r, true)))
                .filter(|_| !matches_only)
                .chain(consumer.matched_arms.iter().map(|r| (r, true)));
            for ((in_declaration, spelling), flat_admitted) in reads {
                let Some(bound) = read_binding(
                    base,
                    head,
                    consumer,
                    spelling,
                    &change.module_path,
                    &universe,
                ) else {
                    continue;
                };
                if bound == InterfaceConsumerBinding::BoundThroughFlatBareChannel && !flat_admitted
                {
                    continue;
                }
                in_declarations.insert(in_declaration.clone());
                // A declarer-bound read wins over a flat one for the module's disposition: the
                // module IS a resolved consumer if any read resolves, and the flat count is for
                // modules that reach the declaration by no other route.
                binding = Some(match (binding, bound) {
                    (Some(InterfaceConsumerBinding::BoundToDeclaringModule), _)
                    | (_, InterfaceConsumerBinding::BoundToDeclaringModule) => {
                        InterfaceConsumerBinding::BoundToDeclaringModule
                    }
                    _ => InterfaceConsumerBinding::BoundThroughFlatBareChannel,
                });
            }
            // An introduced roster strands only readers it does not name: a module whose every
            // reading declaration is admitted stays valid.
            if let InterfaceChangeGround::AdmissionIntroduced { admitted_callers } = &change.ground
            {
                let all_admitted = in_declarations.iter().all(|in_declaration| {
                    admitted_callers.iter().any(|(module, decl)| {
                        module == &consumer.module_path && decl == in_declaration
                    })
                });
                if all_admitted {
                    continue;
                }
            }
            if let Some(binding) = binding {
                consumers.push(InterfaceChangedConsumer {
                    changed_module_path: change.module_path.clone(),
                    changed_declaration: change.declaration.clone(),
                    consumer_module_path: consumer.module_path.clone(),
                    consumer_rel_path: consumer.rel_path.clone(),
                    in_declarations: in_declarations.into_iter().collect(),
                    binding,
                });
            }
        }
    }
    InterfaceConsumerSelection { changes, consumers }
}

// ---------------------------------------------------------------------------
// ACQUISITION — git over the workspace, and the two sides of one diff
// ---------------------------------------------------------------------------

/// Run one `git` invocation in `workspace` to completion and return its stdout, or refuse with
/// what it said. The one shell-out shape every baseline question here shares.
pub fn git_stdout(workspace: &Path, args: &[&str]) -> Result<String, String> {
    let out = Command::new("git")
        .args(args)
        .current_dir(workspace)
        .env("GIT_PAGER", "cat")
        .output()
        .map_err(|e| format!("spawn git {}: {e}", args.join(" ")))?;
    if !out.status.success() {
        return Err(format!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim_end().to_string())
}

/// Whether a repository path is one the head sweep would have parsed. The two exclusions mirror
/// `run_dag_parse_sweep`'s: build output, and the parser's deliberately malformed fixtures. A
/// base file the head sweep would not read must not enter the base index, or the two sides are
/// measured by different instruments.
///
/// THIS ANSWERS THE PARSER'S QUESTION AND NOTHING ELSE, AND IT IS APPLIED AT THE POINT OF USE.
/// It used to be applied inside `diff_sides`, so the only available answer to "what did this
/// change touch" was already narrowed to `.dag` — and a second consumer asking a DIFFERENT
/// question read that narrowed list as if it were the diff. That consumer asked about a `.rs`
/// path, a `.rs` path cannot survive a `.dag` filter, and the arm it fed was therefore false on
/// every production run: an upstream filter written for one question silently deciding another,
/// with nothing joining them (`gunbc.recurring_failure_mode` `incidental_denominator_as_wall`).
/// The repair is not a second path list — that would be two representations of one fact, the same
/// class one step later. `diff_sides` reports what the diff touched, once; every consumer applies
/// the scope its own question needs, here.
pub fn in_sweep_scope(rel: &str) -> bool {
    rel.ends_with(".dag")
        && DAG_PARSE_SWEEP_ROOTS
            .iter()
            .any(|root| rel.starts_with(&format!("{root}/")))
        && !rel.contains("/target/")
        && !rel.contains("/tests/fixtures/")
}

/// The two sides of a diff, at file grain: which head paths the change touched, and which base
/// paths must be re-read to reconstruct the baseline.
///
/// THEY ARE NOT THE SAME SET, AND A RENAME IS WHERE THEY COME APART. `git diff --name-only`
/// reports a detected rename as ONE path -- the destination (rename detection is on by default).
/// An earlier revision read that list as both sides, so a renamed module lost its base side: the
/// source was never re-read and the destination is absent from the base tree, so every
/// declaration read as newly added. Review 56471 was right to reject it.
///
/// So the diff is read rename-aware, `--name-status -z -M`, each entry contributing to the sides
/// SEPARATELY: a rename gives its destination to head and its source to base, an addition only a
/// head path, a deletion only a base path, a modification the same path to both.
///
/// THE SIDES ARE UNFILTERED, WHICH IS WHAT MAKES THIS ONE AUTHORITY FOR WHAT THE DIFF TOUCHED.
/// Scope is not applied here: it belongs to the QUESTION being asked, not to the diff — the
/// baseline reconstruction wants the parser's `in_sweep_scope`, and a rename may still cross a
/// scope boundary either way, so the scope is applied PER SIDE at the call site.
pub fn diff_sides(name_status_z: &str) -> (Vec<String>, Vec<String>) {
    let mut head_touched = Vec::new();
    let mut base_side = Vec::new();
    let mut fields = name_status_z.split('\0').filter(|f| !f.is_empty());
    while let Some(status) = fields.next() {
        // A rename or copy carries a similarity score after the letter and TWO paths after that.
        let renamed = status.starts_with('R') || status.starts_with('C');
        let Some(first) = fields.next() else { break };
        if renamed {
            let Some(second) = fields.next() else { break };
            base_side.push(first.to_string());
            head_touched.push(second.to_string());
        } else if status.starts_with('A') {
            head_touched.push(first.to_string());
        } else if status.starts_with('D') {
            base_side.push(first.to_string());
        } else {
            base_side.push(first.to_string());
            head_touched.push(first.to_string());
        }
    }
    (head_touched, base_side)
}

/// One base-side source parsed into records, or a refusal naming what could not be read.
///
/// A BASE FILE THAT DOES NOT PARSE IS AN UNOBSERVABLE BASELINE, NOT AN EMPTY ONE, and the
/// difference decides the verdict of whatever compares against it. An earlier revision returned
/// an empty vec as "the conservative direction" because it only made the caller quieter.
/// Backwards, and review 56449 was right to reject it: rows with no base side are not deltas,
/// so every row that file carried silently STOPS BEING COMPARED while the caller answers
/// success — the empty-observation narrow, ⊥-as-answer conflated with ⊥-as-ignorance, strictly
/// worse than the widen §5 forbids: a widen is expensive, a narrow is silently uncovered.
///
/// The head sweep refuses on diagnostics, so refusing here keeps both sides on ONE instrument.
/// History is not this module's to repair — but "I cannot see the baseline" is a refusal to
/// state, not a fact to assume.
///
/// Annotation-grain refusals are the exception named by
/// `fail_closed_gate_refuses_its_own_repair`: the parser still produced the module (annotation
/// bind runs after parse), so the baseline IS observable. Treating those diagnostics as
/// unreadable base sealed the transition that only moves `//` onto the declaration the grain
/// admits. Any other diagnostic, or a file that produced no module, stays unobservable.
/// THE ENVIRONMENT IS REQUIRED, NOT DEFAULTED. A default would be the HEAD grammar, and this
/// function's entire job is reading the BASE revision -- so a forgetful caller would read base text
/// under head rules and answer confidently, which is
/// `gunbc.recurring_failure_mode.base_readability_gate_refuses_a_grammar_change` itself. A caller
/// that genuinely means the head environment says so at the call site.
pub fn base_records(
    rel: &str,
    content: &str,
    environment: std::rc::Rc<crate::std_syntax::ParseEnvironment>,
) -> Result<Vec<ModuleDeclarationRecord>, String> {
    let fill = crate::v1_compiler_compile::parse_census_fill_sources_with_environment(
        std::rc::Rc::new(
            vec![std::rc::Rc::new(crate::v1_compiler_compile::SourceFile {
                path: rel.to_string(),
                content: content.to_string(),
            })]
            .into(),
        ),
        environment,
    );
    let annotation_erased_readable = !fill.modules.is_empty()
        && !fill.diagnostics.is_empty()
        && fill.diagnostics.iter().all(|d| {
            matches!(
                *d.diagnostic,
                crate::v1_std_core::CompilerDiagnostic::SourceAnnotationRefused { .. }
            )
        });
    if !fill.diagnostics.is_empty() && !annotation_erased_readable {
        return Err(format!(
            "{rel} does not parse at the base revision ({} diagnostic(s)), so its base-side \
             declarations cannot be read",
            fill.diagnostics.len()
        ));
    }
    let source_indices: std::rc::Rc<
        im::HashMap<String, std::rc::Rc<crate::v1_std_core::NewlineIndex>>,
    > = std::rc::Rc::new(
        fill.newline_indices
            .iter()
            .fold(im::HashMap::new(), |acc, i| {
                acc.update(i.file.clone(), i.clone())
            }),
    );
    Ok(fill
        .modules
        .iter()
        .map(|module| record_from_module(module, &source_indices, rel, &fill.occurrence_transport))
        .collect())
}

// ---------------------------------------------------------------------------
// THE BASE-INDEX RECONSTRUCTION — the head index with the diff applied in reverse
// ---------------------------------------------------------------------------

/// The base side of one change, reconstructed from the head index, at file grain.
///
/// LIFTED OUT OF THE DELETED WAVE-ADMISSION WALL'S RUNNER so the required floor's planning row
/// can ask the same question over ITS OWN comparison window. The two resolved different windows
/// on purpose -- the wall compared against the merge base with `origin/main`, the floor against
/// `v2.workflow.floor_diff_observe`'s resolved baseline -- so the refs are parameters and the
/// reconstruction is one function. The wall is gone (operator ruling 2026-09-19); the floor is
/// the remaining caller. It is `pub(crate)`: its only callers are in this crate, and a public
/// export would be seed surface growth under the freeze.
pub(crate) enum BaselineReconstruction {
    /// The window's base IS its head: nothing to reconstruct, and not a refusal.
    NoSubject { head: String },
    /// The base could not be observed. NOT an empty base: the two are different states with
    /// different remedies, and conflating them is the empty-observation narrow.
    NotEvaluated { reason: String },
    Reconstructed {
        base: String,
        head: String,
        base_index: DeclarationIndex,
    },
}

/// THE BASE INDEX IS THE HEAD INDEX WITH THE DIFF APPLIED IN REVERSE, at file grain -- the
/// construction, not an optimisation -- unless the two revisions speak different grammars, in
/// which case the whole base side is read under the base's own environment (see below). Only
/// changed files are re-parsed from their base blobs and substituted on the ordinary route.
pub(crate) fn reconstruct_base_index(
    workspace: &std::path::Path,
    base: &str,
    head: &str,
    head_index: &DeclarationIndex,
) -> Result<BaselineReconstruction, String> {
    let base = base.to_string();
    let head = head.to_string();
    let workspace = workspace.to_path_buf();
    if base == head {
        return Ok(BaselineReconstruction::NoSubject { head });
    }

    // WHICH GRAMMAR DOES THE BASE SPEAK? Everything below reads base-side declarations, and reading
    // them under the head's grammar is the defect the deleted wall kept tripping over: a change
    // that edits the language refuses in proportion to how thoroughly it succeeded
    // (gunbc.recurring_failure_mode.base_readability_gate_refuses_a_grammar_change). The floor's
    // reconstruction answers the same question, so the check stays.
    //
    // The cheap answer comes first. Object identity over the files declaring the environment settles
    // "same grammar?" in a few `rev-parse` calls, so the ordinary pull request -- which changes no
    // grammar -- pays nothing, and only a real grammar change pays to materialize and evaluate the
    // base corpus.
    let agreement = match environment_agreement(&workspace, &base, &head) {
        Ok(a) => a,
        // A REFUSAL HERE IS NOT A LICENCE TO USE THE HEAD'S. Not knowing which grammar the base
        // speaks makes every base-side declaration unreadable, which is ignorance, and ignorance is
        // NotEvaluated rather than a confident answer under the wrong rules.
        Err(e) => {
            return Ok(BaselineReconstruction::NotEvaluated {
                reason: format!(
                    "the base revision's parse environment could not be established ({}), so its declarations cannot be read under any grammar this run can justify", environment_load_refusal_text(&e)
                ),
            })
        }
    };

    // THE KERNEL HALF. `declaring_candidates` consults this binary's own `kernel_type_set`, a head
    // fact, so one map serves exactly when the base declares the same kernel NAMES. That is decided
    // at the grain of the name set, not the declaring file's bytes; distinct sets refuse.
    match kernel_set_serves_both(&workspace, &base, &head) {
        Ok(true) => {}
        Ok(false) => {
            return Ok(BaselineReconstruction::NotEvaluated {
                reason: format!(
                    "the kernel-name set {KERNEL_TYPES_PATH} declares at {base} differs from the one this binary carries for {head}, so one kernel map cannot speak for the base side"
                ),
            })
        }
        Err(e) => {
            return Ok(BaselineReconstruction::NotEvaluated {
                reason: format!("the kernel declaring file could not be compared ({})", environment_load_refusal_text(&e)),
            })
        }
    }

    let name_status = git_stdout(
        &workspace,
        &["diff", "--name-status", "-z", "-M", &base, &head],
    )?;
    let (head_touched, base_side) = diff_sides(&name_status);

    // THE PARSER'S SCOPE IS APPLIED HERE, WHERE THE PARSER'S QUESTION IS ASKED, AND NOWHERE ELSE.
    // `head_touched` and `base_side` are what the diff touched; these two are what the baseline
    // reconstruction may read. Filtering per side rather than once is not redundancy: a rename may
    // cross the sweep boundary in either direction, which is why `diff_sides` splits the sides in
    // the first place.
    let head_parsed: Vec<&String> = head_touched.iter().filter(|p| in_sweep_scope(p)).collect();
    let base_parsed: Vec<&String> = base_side.iter().filter(|p| in_sweep_scope(p)).collect();

    // A DERIVED ARTIFACT IS NEVER IN THE DIFF, SO IT MUST NOT BE INHERITED FROM THE HEAD.
    // The baseline is reconstructed by carrying every head record the diff did not touch and
    // re-reading the rest from the base tree. `roster.dag` is gitignored and written on the read
    // path, so the diff can never name it — and carrying it made the HEAD's roster stand as the
    // BASE's. Its base side is not read from git either (the tree does not carry it); it is
    // DERIVED from the base tree's row membership, below, by the same renderer the writer uses.
    //
    // CARRYING UNTOUCHED HEAD RECORDS IS VALID ONLY WHILE THE GRAMMARS AGREE. The reconstruction
    // below keeps every head record the diff did not touch, which assumes an untouched FILE has an
    // untouched PARSE. That holds when both revisions speak one grammar and fails exactly when they
    // do not: a keyword, literal, operator or item-form change gives an untouched file a different
    // parse, so its head records are not its base records. When the environments differ, nothing is
    // carried and every base-side file in sweep scope is read under the base's own environment.
    let (base_environment, full_base_parse, differing) = match &agreement {
        EnvironmentAgreement::Identical => (
            crate::extdeps_languages_dag_syntax::dag_parse_environment(),
            false,
            Vec::new(),
        ),
        EnvironmentAgreement::Differs {
            base_environment,
            differing_paths,
        } => (base_environment.clone(), true, differing_paths.clone()),
    };
    if full_base_parse {
        eprintln!(
            "namespace-baseline: the base and head parse environments differ ({}), so the \
             baseline is read in full under the base's own grammar rather than reconstructed from \
             untouched head records",
            differing.join(", ")
        );
    }

    let mut base_index = DeclarationIndex::default();
    if !full_base_parse {
        for record in index_records(head_index) {
            if crate::cli_run::derived_row_roster::is_derived_roster_path(&record.rel_path) {
                continue;
            }
            if !head_parsed.iter().any(|c| *c == &record.rel_path) {
                index_insert(&mut base_index, record.clone());
            }
        }
    }
    // ABSENCE AT THE BASE IS ESTABLISHED FROM AN AUTHORITATIVE LISTING, NEVER INFERRED FROM A
    // FAILURE. An earlier revision treated ANY `git show <base>:<path>` error as proof the path
    // was ADDED — a read fault, corrupt object or permission problem all read as "new file", and
    // the module's base side vanished while the caller answered as though the comparison ran.
    // Review 56449 was right to reject it: only ONE cause means added; the rest are ignorance
    // wearing its clothes.
    //
    // `ls-tree` answers what the base tree CONTAINS: a path missing from its output is absent,
    // and a failure to obtain the listing is a refusal, not an empty answer.
    let base_paths = git_stdout(&workspace, &["ls-tree", "-r", "--name-only", &base])?;
    let base_paths: BTreeSet<String> = base_paths.lines().map(|l| l.trim().to_string()).collect();
    // WHEN THE GRAMMARS DIFFER THE READ SET IS THE WHOLE BASE SIDE, not the diff's. The diff is a
    // statement about bytes; a grammar change is a statement about every file's parse.
    let owned_full: Vec<String> = if full_base_parse {
        base_paths
            .iter()
            .filter(|p| in_sweep_scope(p))
            .cloned()
            .collect()
    } else {
        Vec::new()
    };
    let read_set: Vec<&String> = if full_base_parse {
        owned_full.iter().collect()
    } else {
        base_parsed.clone()
    };
    // ONE ACQUISITION FOR THE WHOLE READ SET, not one `git show` per path. On the grammar-differs
    // route the read set is every base-side file in sweep scope -- thousands -- and the process-
    // per-file shape this prerequisite removed from the loader must not survive one layer down in
    // its consumer. The paths the base actually carries are archived once and read locally.
    let present: Vec<&str> = read_set
        .iter()
        .filter(|rel| base_paths.contains(**rel))
        .map(|rel| rel.as_str())
        .collect();
    let base_tree = workspace.join("target").join(format!(
        "gunbc-ns-base-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    if !present.is_empty() {
        if let Err(e) = materialize_revision_paths(&workspace, &base, &base_tree, &present) {
            let _ = std::fs::remove_dir_all(&base_tree);
            return Ok(BaselineReconstruction::NotEvaluated {
                reason: format!(
                    "the base revision's files could not be materialized ({}), so the baseline is \
                     unobservable and no verdict is available",
                    environment_load_refusal_text(&e)
                ),
            });
        }
    }
    for rel in &read_set {
        if !base_paths.contains(*rel) {
            // Genuinely added by this change: no base side to read, established by the listing.
            continue;
        }
        // The listing says the base carries this path, so a read failure here is UNOBSERVABLE
        // BASELINE, not news about the file.
        let content = match std::fs::read_to_string(base_tree.join(rel)) {
            Ok(c) => c,
            Err(e) => {
                let _ = std::fs::remove_dir_all(&base_tree);
                return Ok(BaselineReconstruction::NotEvaluated {
                    reason: format!(
                        "cannot read {rel} at the base revision {base} ({e}), so the baseline is \
                         partially unobservable and no verdict is available"
                    ),
                });
            }
        };
        match base_records(rel, &content, base_environment.clone()) {
            Ok(records) => {
                for record in records {
                    index_insert(&mut base_index, record);
                }
            }
            // A BASE THIS PARSER CANNOT READ IS UNEVALUATED, FULL STOP. The annotation case that
            // used to be repaired here is now answered truthfully upstream: `base_records` reads
            // the real base declarations out of an annotation-refused parse via
            // `annotation_erased_readable`, so the only reason left to reach this arm is a base
            // carrying NON-annotation diagnostics. Substituting head records there is exactly the
            // fabricated parse the readable path exists to avoid -- the remainders would still
            // compare equal whenever the head touched only comments, so the old discriminator
            // would happily certify a baseline for a file that failed to parse for an unrelated
            // reason. The residual case argues for deletion, not for retention.
            Err(reason) => {
                let _ = std::fs::remove_dir_all(&base_tree);
                return Ok(BaselineReconstruction::NotEvaluated { reason });
            }
        }
    }
    let _ = std::fs::remove_dir_all(&base_tree);

    // THE DERIVED ROSTER'S BASE SIDE, from the base tree's row membership. `base_paths` is the
    // authoritative listing already in hand, so this asks the same question the writer asks of a
    // directory. A base tree carrying no row files under that root has no roster module at all,
    // and `roster_from_path_listing` answers `None` rather than fabricating a present empty list.
    let base_path_refs: Vec<&str> = base_paths.iter().map(|p| p.as_str()).collect();
    for record in index_records(head_index) {
        let Some(root) = crate::cli_run::derived_row_roster::roster_root_prefix(&record.rel_path)
        else {
            continue;
        };
        let Some(content) = crate::cli_run::derived_row_roster::roster_from_path_listing(
            base_path_refs.iter().copied(),
            root,
        ) else {
            continue;
        };
        // SYNTHESIZED BY THE CURRENT RENDERER, SO PARSED UNDER THE CURRENT GRAMMAR. This content is
        // not bytes read from the base tree; it is new source the head's roster writer produced from
        // the base tree's path membership. The base environment is reserved for bytes that actually
        // came out of the base revision -- sending renderer output through an older grammar could
        // refuse text no historical source ever carried.
        match base_records(
            &record.rel_path,
            &content,
            crate::extdeps_languages_dag_syntax::dag_parse_environment(),
        ) {
            Ok(records) => {
                for record in records {
                    index_insert(&mut base_index, record);
                }
            }
            Err(reason) => return Ok(BaselineReconstruction::NotEvaluated { reason }),
        }
    }

    Ok(BaselineReconstruction::Reconstructed {
        base,
        head,
        base_index,
    })
}

// THE PARSE ENVIRONMENT OF A REVISION THAT IS NOT THE RUNNING BINARY'S.
//
// `ParseEnvironment` (`std.syntax`) is threaded through the tokenizer and parser so that reading a
// revision's source does not mean reading it under whatever grammar this binary was built with.
// That thread is inert until something can PRODUCE an environment other than the compiled-in
// `dag_parse_environment`. This is that producer, and the revision SELECTS THE SOURCE: the bytes
// evaluated are the bytes git holds at that revision, not the worktree's.
//
// WHY: `gunbc.recurring_failure_mode.base_readability_gate_refuses_a_grammar_change`. A gate that
// compares base-side declarations against head-side ones parses both with one compiler, and has no
// representable arm for "the base is well formed under its OWN grammar and unreadable only under
// the head's" -- so a change that edits the grammar is refused in proportion to how thoroughly it
// succeeded. The failure mode was found by the deleted wave-admission wall; the base-side
// reconstruction above keeps the same requirement, so the loader stays.
//
// ONE MATERIALIZATION, THEN THE REPOSITORY'S OWN INDEX. An earlier revision of this code listed the
// whole `.dag` tree and ran one `git show` per file -- 5,306 subprocesses on every required run --
// and recognized `module` and `import` with its own line-prefix scanner, which is a second grammar
// for declarations the module index already recognizes (section 3). Both are gone: one
// `git archive` writes the revision's `dag/` tree into a caller-owned directory, and the real
// module index and entry resolver read it from there. The loader therefore cannot disagree with the
// compiler about what a module is, because it does not decide.
//
// DECODE IS NOT HAND-WRITTEN. `Value` -> `value_to_wire_json` -> `serde_json::from_value`: the wire
// encoder resolves its tag policy from the same emitter that wrote the `#[serde(...)]` attributes
// on the mirror struct, so encoder and decoder cannot disagree about shape unless the emitter
// disagrees with itself. A hand-written decoder would fork the type's shape across nine types and
// drift the first time a field was added to `SyntaxSpec`.
//
// WHAT IT DOES NOT COVER. This reproduces the DECLARATIVE environment: which words are keywords,
// which item forms exist, which operators bind how. It does NOT reproduce the revision's PARSER --
// body parsers are dispatched on `body_kind` to code compiled into this binary, so a revision whose
// body parser behaved differently is not reproduced by supplying its environment and must not be
// claimed to be. That population stays outside the covered set.

/// The module whose declarations ARE the dag realization's parse environment.
const ENVIRONMENT_MODULE: &str = "extdeps.languages.dag.syntax";
/// The data item within it that carries the environment value.
const ENVIRONMENT_ITEM: &str = "dag_parse_environment";
/// The path, relative to the repository root, of the file declaring `ENVIRONMENT_MODULE`.
const ENVIRONMENT_MODULE_PATH: &str = "dag/extdeps/languages/dag/syntax.dag";
/// Where the corpus of `.dag` declarations lives, relative to the repository root.
const DAG_SOURCE_ROOT: &str = "dag";

/// Why an environment could not be produced for a revision.
///
/// EVERY ARM IS A REFUSAL, NEVER A SUBSTITUTION. The tempting arm when a base environment cannot be
/// read is to fall back to the head's -- precisely the assumption this code exists to remove, and it
/// would fail open on exactly the changes that alter the grammar. Section 5's absorbing fallback in
/// its purest form: nothing is missed, so the arm reads as safe, while the only signal that the base
/// was unreadable is destroyed. So the failure is typed and located and the caller decides what an
/// unreadable base means for its own verdict.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnvironmentLoadRefusal {
    /// `git` could not be run, or answered non-zero, while reading this revision.
    RevisionUnreadable {
        revision: String,
        step: String,
        cause: String,
    },
    /// The revision's materialized tree has no file at the environment module's path.
    EnvironmentModuleMissing { revision: String, path: String },
    /// The materialized corpus did not resolve, or the item did not evaluate.
    ClosureNotEvaluable { revision: String, cause: String },
    /// `ENVIRONMENT_ITEM` is not declared by `ENVIRONMENT_MODULE` at this revision.
    ///
    /// Separate from `ClosureNotEvaluable` because it is the HOMONYM refusal: the interpreter
    /// resolves a data item by bare name across the whole closure, so without this check a
    /// `dag_parse_environment` declared anywhere else could silently supply the grammar. The
    /// environment must come from the declaration that owns it.
    EnvironmentItemNotOwned {
        revision: String,
        item: String,
        module: String,
    },
    /// The item evaluated, but its value did not decode into the typed environment.
    ///
    /// The arm that fires on emitted-schema drift -- a field the wire encoder omits that the mirror
    /// struct requires. Separate from `ClosureNotEvaluable` because the owners differ: that one is a
    /// defect in the revision being read, this one is THIS binary disagreeing with its own emitter.
    ValueNotDecodable { revision: String, cause: String },
    /// The kernel-name set declared at this revision could not be read as a set of names.
    ///
    /// Separate from the environment arms because the subject differs: this is `std.types`
    /// `kernel_type_set`, not the grammar, and an operator reading the refusal must be told which
    /// fact was unreadable.
    KernelSetNotReadable { revision: String, cause: String },
}

/// The operator-facing text of a refusal.
///
/// A FREE FUNCTION, NOT A `Display` IMPL, because this module's seed-growth roster enumerates every
/// declaration it carries by `DeclarationRef`, and an `impl` block is the one item that roster
/// structurally cannot cite -- the reason an earlier lane converted the module's methods to free
/// functions. An earlier revision of this change added `impl Display` here and left the roster's
/// "carries no impl block" sentence standing over it; this keeps the sentence true.
pub fn environment_load_refusal_text(refusal: &EnvironmentLoadRefusal) -> String {
    match refusal {
        EnvironmentLoadRefusal::RevisionUnreadable {
            revision,
            step,
            cause,
        } => format!("reading revision {revision} failed at {step}: {cause}"),
        EnvironmentLoadRefusal::EnvironmentModuleMissing { revision, path } => format!(
            "{path} does not exist at revision {revision}, so the value it declares cannot be \
                 read"
        ),
        EnvironmentLoadRefusal::ClosureNotEvaluable { revision, cause } => {
            format!("the declaring closure at revision {revision} did not evaluate: {cause}")
        }
        EnvironmentLoadRefusal::EnvironmentItemNotOwned {
            revision,
            item,
            module,
        } => format!(
            "`{item}` is not declared by `{module}` at revision {revision}, so the value a \
                 bare-name lookup would return is not that declaration's"
        ),
        EnvironmentLoadRefusal::ValueNotDecodable { revision, cause } => format!(
            "the parse environment at revision {revision} evaluated but did not decode into \
                 this binary's `ParseEnvironment`: {cause}"
        ),
        EnvironmentLoadRefusal::KernelSetNotReadable { revision, cause } => format!(
            "the kernel-name set declared at revision {revision} could not be read: {cause}"
        ),
    }
}

/// Run one `git` invocation to completion, or refuse with what it said.
fn git_capture(
    repo: &std::path::Path,
    revision: &str,
    step: &str,
    args: &[&str],
) -> Result<Vec<u8>, EnvironmentLoadRefusal> {
    let out = std::process::Command::new("git")
        .args(args)
        .current_dir(repo)
        .output()
        .map_err(|e| EnvironmentLoadRefusal::RevisionUnreadable {
            revision: revision.to_string(),
            step: step.to_string(),
            cause: format!("git failed to start: {e}"),
        })?;
    if !out.status.success() {
        return Err(EnvironmentLoadRefusal::RevisionUnreadable {
            revision: revision.to_string(),
            step: step.to_string(),
            cause: String::from_utf8_lossy(&out.stderr).trim().to_string(),
        });
    }
    Ok(out.stdout)
}

/// The object id git holds for one path at one revision, or `None` if the path is absent.
///
/// Used to decide whether two revisions share a parse environment WITHOUT materializing either:
/// object identity is content identity, so equal ids over the environment's declaring files mean the
/// environments are equal by construction rather than by comparison.
pub fn blob_id_at(
    repo: &std::path::Path,
    revision: &str,
    path: &str,
) -> Result<Option<String>, EnvironmentLoadRefusal> {
    let spec = format!("{revision}:{path}");
    let out = std::process::Command::new("git")
        .args(["rev-parse", "--verify", "--quiet", &spec])
        .current_dir(repo)
        .output()
        .map_err(|e| EnvironmentLoadRefusal::RevisionUnreadable {
            revision: revision.to_string(),
            step: format!("rev-parse {spec}"),
            cause: format!("git failed to start: {e}"),
        })?;
    if !out.status.success() {
        return Ok(None);
    }
    let id = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if id.is_empty() {
        Ok(None)
    } else {
        Ok(Some(id))
    }
}

/// Materialize one revision's ENVIRONMENT CLOSURE under `dest`, in a single git invocation.
///
/// ONLY THE CLOSURE, NEVER THE WHOLE TREE, and the reason is the class this loader exists to
/// repair, met one level down. The module index that resolves the environment parses every `.dag`
/// file it is shown with THIS binary's grammar. Materializing the whole `dag/` tree therefore
/// parsed every base-side file under the head's grammar in order to learn the base's grammar --
/// and a base file written in the base's grammar refused inside the loader before the environment
/// was ever evaluated. The grammar-differs witness caught exactly that. So the index is shown only
/// the files the environment's declaring closure consists of.
///
/// THE HONEST BOUNDARY THIS DRAWS: the loader can read a base whose grammar differs from the head's
/// so long as the base's ENVIRONMENT CLOSURE is itself readable under the head's grammar. A change
/// that alters the grammar AND uses the altered grammar inside `std.syntax`'s own closure is outside
/// any head-built loader's reach -- the same bootstrap boundary the compiler itself has -- and it
/// refuses as `ClosureNotEvaluable`, never answering under the wrong rules.
///
/// `git archive | tar -x` over the named paths rather than a read per file: acquisition cost must
/// not scale with the corpus (section 6, bare minimum cost). `dest` is the caller's to remove.
fn materialize_environment_closure_at(
    repo: &std::path::Path,
    revision: &str,
    dest: &std::path::Path,
    closure_paths: &BTreeSet<String>,
) -> Result<(), EnvironmentLoadRefusal> {
    let paths: Vec<&str> = closure_paths.iter().map(String::as_str).collect();
    materialize_revision_paths(repo, revision, dest, &paths)?;
    if !dest.join(ENVIRONMENT_MODULE_PATH).exists() {
        return Err(EnvironmentLoadRefusal::EnvironmentModuleMissing {
            revision: revision.to_string(),
            path: ENVIRONMENT_MODULE_PATH.to_string(),
        });
    }
    Ok(())
}

/// Materialize the named paths of one revision under `dest`: one `git archive`, one `tar -x`.
///
/// THE ONE ACQUISITION ROUTE. Every consumer that needs a revision's bytes on disk -- the
/// environment loader above for its closure, the parse-environment witnesses for a whole `dag/`
/// tree -- comes through here, so acquisition is checked once and a defect in it is found once. An
/// earlier shape had the witnesses carrying their own `sh -c "git archive | tar"` string beside
/// this function: the same pipeline twice over, one of them unchecked hand-shell. A pathspec may
/// name a directory (`dag`) or a file; git archive accepts both.
pub fn materialize_revision_paths(
    repo: &std::path::Path,
    revision: &str,
    dest: &std::path::Path,
    paths: &[&str],
) -> Result<(), EnvironmentLoadRefusal> {
    std::fs::create_dir_all(dest).map_err(|e| EnvironmentLoadRefusal::RevisionUnreadable {
        revision: revision.to_string(),
        step: "create materialization directory".to_string(),
        cause: e.to_string(),
    })?;
    let mut args: Vec<&str> = vec!["archive", "--format=tar", revision];
    args.extend_from_slice(paths);
    let archive = git_capture(repo, revision, "archive", &args)?;
    let tar_path = dest.join("dag-tree.tar");
    std::fs::write(&tar_path, &archive).map_err(|e| {
        EnvironmentLoadRefusal::RevisionUnreadable {
            revision: revision.to_string(),
            step: "write archive".to_string(),
            cause: e.to_string(),
        }
    })?;
    let extract = std::process::Command::new("tar")
        .arg("-xf")
        .arg(&tar_path)
        .arg("-C")
        .arg(dest)
        .output()
        .map_err(|e| EnvironmentLoadRefusal::RevisionUnreadable {
            revision: revision.to_string(),
            step: "tar -xf".to_string(),
            cause: format!("tar failed to start: {e}"),
        })?;
    if !extract.status.success() {
        return Err(EnvironmentLoadRefusal::RevisionUnreadable {
            revision: revision.to_string(),
            step: "tar -xf".to_string(),
            cause: String::from_utf8_lossy(&extract.stderr).trim().to_string(),
        });
    }
    let _ = std::fs::remove_file(&tar_path);
    Ok(())
}

/// Decode an evaluated environment value into this binary's `ParseEnvironment`.
pub fn decode_environment_value(
    value: &crate::v1_interpreter::Value,
    ctx: &crate::v1_interpreter::InterpContext,
    revision: &str,
) -> Result<std::rc::Rc<crate::std_syntax::ParseEnvironment>, EnvironmentLoadRefusal> {
    let wire = super::value_to_wire_json(value, ctx).map_err(|e| {
        EnvironmentLoadRefusal::ValueNotDecodable {
            revision: revision.to_string(),
            cause: format!("wire-encode: {e}"),
        }
    })?;
    serde_json::from_value::<crate::std_syntax::ParseEnvironment>(wire)
        .map(std::rc::Rc::new)
        .map_err(|e| EnvironmentLoadRefusal::ValueNotDecodable {
            revision: revision.to_string(),
            cause: e.to_string(),
        })
}

/// Evaluate the parse environment out of an already-materialized corpus rooted at `root`.
///
/// Split from acquisition so the decode seam can be exercised against an independent oracle -- the
/// compiled-in `dag_parse_environment()` over the live tree -- without a revision in the way. The
/// revision string here is diagnostic ONLY; callers that mean "the environment AT a revision" must
/// use `load_parse_environment_at`, which selects the source.
pub fn evaluate_environment_in(
    root: &std::path::Path,
    revision: &str,
) -> Result<std::rc::Rc<crate::std_syntax::ParseEnvironment>, EnvironmentLoadRefusal> {
    let (value, ctx) =
        evaluate_owned_item_in(root, ENVIRONMENT_MODULE_PATH, ENVIRONMENT_ITEM, revision)?;
    decode_environment_value(&value, &ctx, revision)
}

/// Evaluate the data item `item`, as declared BY the file `entry_rel`, out of a materialized corpus.
///
/// The one evaluation route for a revision's declared value: the parse environment and the kernel
/// name set both come through here, so ownership and hermeticity are checked once.
fn evaluate_owned_item_in(
    root: &std::path::Path,
    entry_rel: &str,
    item: &str,
    revision: &str,
) -> Result<
    (
        crate::v1_interpreter::Value,
        crate::v1_interpreter::InterpContext,
    ),
    EnvironmentLoadRefusal,
> {
    let entry = root.join(entry_rel);
    if !entry.exists() {
        return Err(EnvironmentLoadRefusal::EnvironmentModuleMissing {
            revision: revision.to_string(),
            path: entry.display().to_string(),
        });
    }
    let dag_root = root.join(DAG_SOURCE_ROOT);
    let index = super::build_multi_entry_index(&[dag_root.display().to_string()]);
    let entry_display = entry.display().to_string();
    let (graph, indices) =
        super::resolve_entry_with_index_for_discovery_corpus(&index, &entry_display).map_err(
            |e| EnvironmentLoadRefusal::ClosureNotEvaluable {
                revision: revision.to_string(),
                cause: e,
            },
        )?;
    // HERMETIC, NOT WET. A static declaration has no business acquiring permission to perform host
    // effects while it is being decoded; `Wet` here would let a corpus under examination act during
    // examination.
    let ctx = super::make_eval_context(
        &graph,
        indices,
        crate::v1_interpreter::ExecutionMode::Hermetic,
    );
    // EXACT OWNERSHIP, NOT A BARE NAME. `eval_data_item_value` resolves by bare name across the
    // closure, so a homonymous item elsewhere in the corpus would silently supply the value. It must
    // come from the declaration that owns it.
    if !super::data_item_declared_in_file(&ctx, item, &entry_display) {
        return Err(EnvironmentLoadRefusal::EnvironmentItemNotOwned {
            revision: revision.to_string(),
            item: item.to_string(),
            module: entry_rel.to_string(),
        });
    }
    let value = crate::v1_interpreter::with_active_context(&ctx, || {
        crate::v1_interpreter::eval_data_item_value(&ctx, item)
    })
    .map_err(|e| EnvironmentLoadRefusal::ClosureNotEvaluable {
        revision: revision.to_string(),
        cause: format!("eval {item}: {e}"),
    })?
    .ok_or_else(|| EnvironmentLoadRefusal::ClosureNotEvaluable {
        revision: revision.to_string(),
        cause: format!("{item} is not a data item in `{entry_rel}`"),
    })?;
    Ok((value, ctx))
}

/// THE LOADER: the parse environment git holds at `revision`.
///
/// The revision selects the bytes. Materialization happens into a temporary directory this function
/// owns and removes, so nothing about the caller's worktree is read or written.
pub fn load_parse_environment_at(
    repo: &std::path::Path,
    revision: &str,
) -> Result<std::rc::Rc<crate::std_syntax::ParseEnvironment>, EnvironmentLoadRefusal> {
    let closure = environment_closure_paths()?;
    load_parse_environment_with_closure(repo, revision, &closure)
}

/// The loader over an ALREADY-RESOLVED closure.
///
/// `environment_agreement` resolves the closure to decide whether the grammars differ and then, on
/// the differing path, loads the base's environment -- the same closure, same inputs, with the
/// caller already holding the answer. Section 2: carry the first value rather than recompute it at
/// the least common ancestor. This is that carried value; `load_parse_environment_at` resolves once
/// for callers that hold nothing.
pub fn load_parse_environment_with_closure(
    repo: &std::path::Path,
    revision: &str,
    closure: &BTreeSet<String>,
) -> Result<std::rc::Rc<crate::std_syntax::ParseEnvironment>, EnvironmentLoadRefusal> {
    let dest = revision_scratch_root("parse-env");
    // If the base's closure has a member the head's does not, the materialized set is incomplete
    // and resolution refuses as ClosureNotEvaluable -- a located refusal, not a fabricated read.
    let outcome = materialize_environment_closure_at(repo, revision, &dest, closure)
        .and_then(|()| evaluate_environment_in(&dest, revision));
    let _ = std::fs::remove_dir_all(&dest);
    outcome
}

/// A fresh directory for one revision's materialized tree, owned and removed by the caller.
///
/// UNDER THE WORKSPACE, NOT /tmp. The module index and entry resolver refuse any path outside the
/// workspace root (`repo_relative_path_normalized`), so a corpus materialized into the system temp
/// directory cannot be read by the repository's own machinery. `target/` is where generated and
/// scratch trees already live (`target/stage0-regen-candidate` is the precedent).
fn revision_scratch_root(purpose: &str) -> std::path::PathBuf {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    super::workspace_root().join("target").join(format!(
        "gunbc-{purpose}-{}-{}",
        std::process::id(),
        stamp
    ))
}

/// The repo-relative files that declare the parse environment's closure, per the real resolver.
///
/// ASKED OF THE RESOLVER, NOT LISTED. The closure is 13 modules today and that number appears
/// nowhere: a hardcoded roster would be a second authority for what the grammar depends on and would
/// go stale, silently, the first time `std.syntax` gained an import -- a stale roster still resolves.
/// The resolved graph's own span files ARE the closure.
pub fn environment_closure_paths() -> Result<BTreeSet<String>, EnvironmentLoadRefusal> {
    closure_paths_of(ENVIRONMENT_MODULE_PATH)
}

/// The repository-relative files of the live tree's resolved closure rooted at `entry_rel`.
fn closure_paths_of(entry_rel: &str) -> Result<BTreeSet<String>, EnvironmentLoadRefusal> {
    let root = super::workspace_root();
    let entry = root.join(entry_rel);
    let dag_root = root.join(DAG_SOURCE_ROOT);
    let index = super::build_multi_entry_index(&[dag_root.display().to_string()]);
    let (graph, _indices) =
        super::resolve_entry_with_index_for_discovery_corpus(&index, &entry.display().to_string())
            .map_err(|e| EnvironmentLoadRefusal::ClosureNotEvaluable {
                revision: "live-tree".to_string(),
                cause: e,
            })?;
    let mut paths = BTreeSet::new();
    for module in graph.modules.iter() {
        for item in module.items.iter() {
            let file = item.span.file.clone();
            if let Some(idx) = file.find(&format!("{DAG_SOURCE_ROOT}/")) {
                paths.insert(file[idx..].to_string());
            }
        }
    }
    if paths.is_empty() {
        return Err(EnvironmentLoadRefusal::ClosureNotEvaluable {
            revision: "live-tree".to_string(),
            cause: format!("the resolved closure of {entry_rel} named no files"),
        });
    }
    Ok(paths)
}

/// Whether two revisions share a parse environment, and the base's environment when they do not.
#[derive(Debug, Clone)]
pub enum EnvironmentAgreement {
    /// Every file declaring the environment is byte-identical across the two revisions.
    ///
    /// Equal object ids are equal content, so the environments are identical BY CONSTRUCTION rather
    /// than by a comparison that could be wrong. Nothing needs loading, and the changed-file
    /// baseline reconstruction stays valid.
    Identical,
    /// The declaring files differ, so the base must be read under its own environment.
    Differs {
        base_environment: std::rc::Rc<crate::std_syntax::ParseEnvironment>,
        differing_paths: Vec<String>,
    },
}

/// Decide whether the base and head grammars agree, loading the base's environment only if not.
///
/// THE CHEAP CHECK COMES FIRST because the expensive one must not be paid on every run: object
/// identity over the closure's files answers "same grammar?" with a handful of `rev-parse` calls,
/// and only a real difference pays for materializing and evaluating the base corpus. The ordinary
/// pull request changes no grammar and therefore costs nothing here.
pub fn environment_agreement(
    repo: &std::path::Path,
    base: &str,
    head: &str,
) -> Result<EnvironmentAgreement, EnvironmentLoadRefusal> {
    let closure = environment_closure_paths()?;
    let mut differing = Vec::new();
    for path in &closure {
        if blob_id_at(repo, base, path)? != blob_id_at(repo, head, path)? {
            differing.push(path.clone());
        }
    }
    if differing.is_empty() {
        return Ok(EnvironmentAgreement::Identical);
    }
    Ok(EnvironmentAgreement::Differs {
        base_environment: load_parse_environment_with_closure(repo, base, &closure)?,
        differing_paths: differing,
    })
}

/// The path whose declarations the kernel-name set is derived from.
const KERNEL_TYPES_PATH: &str = "dag/std/types.dag";

/// The data item in `KERNEL_TYPES_PATH` that declares the kernel-name set.
const KERNEL_TYPES_ITEM: &str = "kernel_type_set";

/// Whether the kernel type set this binary carries can speak for both revisions.
///
/// THE FACT IS THE NAME SET, NOT THE FILE. `declaring_candidates` consults the RUNNING compiler's
/// `kernel_type_set` -- a head fact -- so the question is whether the base revision declares the same
/// set of kernel names. An earlier shape answered it by comparing the whole declaring file's blob,
/// a byte-level proxy for a name-set fact: every edit to `dag/std/types.dag`, a function body or a
/// comment included, refused the reconstruction although no such edit can change the set.
///
/// Equal blobs still settle it for free, since equal content declares equal sets. Only when the file
/// differs is the base's `kernel_type_set` evaluated out of that revision's own tree and its names
/// compared to this binary's. Distinct sets remain `false` -- threading separate base and head kernel
/// maps is the general repair and not this change's subject -- and an unreadable base set is a
/// refusal, never a substitution of the head's.
pub fn kernel_set_serves_both(
    repo: &std::path::Path,
    base: &str,
    head: &str,
) -> Result<bool, EnvironmentLoadRefusal> {
    let base_blob =
        blob_id_at(repo, base, KERNEL_TYPES_PATH).map_err(|e| as_kernel_set_refusal(base, e))?;
    // THE HEAD'S DECLARATION MUST EXIST. This binary's set speaks for the head only because the
    // head declares it; a head with no declaring file has no authority for the binary to stand in
    // for, so it refuses rather than letting the compiled-in set substitute.
    let head_blob = blob_id_at(repo, head, KERNEL_TYPES_PATH)
        .map_err(|e| as_kernel_set_refusal(head, e))?
        .ok_or_else(|| EnvironmentLoadRefusal::KernelSetNotReadable {
            revision: head.to_string(),
            cause: format!("{KERNEL_TYPES_PATH} does not exist at this revision"),
        })?;
    // Equal CONTENT is the free answer; an absent base is not equal to anything and goes on to the
    // base read, which refuses it.
    if base_blob.as_deref() == Some(head_blob.as_str()) {
        return Ok(true);
    }
    let head_names: BTreeSet<String> = crate::std_types::kernel_type_set()
        .keys()
        .cloned()
        .collect();
    Ok(kernel_names_at(repo, base)? == head_names)
}

/// The kernel names `std.types` declares at `revision`, read from that revision's own tree.
///
/// Same acquisition and evaluation route as the parse environment: materialize the declaring file's
/// closure at the revision, evaluate the item it OWNS (not a bare-name homonym), and remove the tree.
pub fn kernel_names_at(
    repo: &std::path::Path,
    revision: &str,
) -> Result<BTreeSet<String>, EnvironmentLoadRefusal> {
    let closure = closure_paths_of(KERNEL_TYPES_PATH)?;
    let dest = revision_scratch_root("kernel-set");
    let outcome = materialize_revision_paths(
        repo,
        revision,
        &dest,
        &closure.iter().map(String::as_str).collect::<Vec<_>>(),
    )
    .and_then(|()| {
        let (value, ctx) =
            evaluate_owned_item_in(&dest, KERNEL_TYPES_PATH, KERNEL_TYPES_ITEM, revision)?;
        let wire = super::value_to_wire_json(&value, &ctx).map_err(|e| {
            EnvironmentLoadRefusal::KernelSetNotReadable {
                revision: revision.to_string(),
                cause: format!("wire-encode {KERNEL_TYPES_ITEM}: {e}"),
            }
        })?;
        serde_json::from_value::<std::collections::BTreeMap<String, bool>>(wire)
            .map(|m| m.into_keys().collect())
            .map_err(|e| EnvironmentLoadRefusal::KernelSetNotReadable {
                revision: revision.to_string(),
                cause: format!("{KERNEL_TYPES_ITEM} is not a Map<String, Bool>: {e}"),
            })
    });
    let _ = std::fs::remove_dir_all(&dest);
    // ONE SUBJECT, ONE REFUSAL ARM. The shared route reports its failures in the parse
    // environment's vocabulary; left as-is, an unreadable kernel set would reach the operator
    // labelled as an unreadable grammar. Every failure here is about the kernel set.
    outcome.map_err(|e| as_kernel_set_refusal(revision, e))
}

/// Relabel a refusal from the shared acquisition route as the kernel-set refusal it is here.
///
/// ONE SUBJECT, ONE REFUSAL ARM. The shared route is subject-neutral; the kernel guard's caller must
/// still be told that it was the kernel-name set that could not be read. The inner refusal's text is
/// kept as the cause, so the specific failure stays located.
fn as_kernel_set_refusal(
    revision: &str,
    refusal: EnvironmentLoadRefusal,
) -> EnvironmentLoadRefusal {
    match refusal {
        EnvironmentLoadRefusal::KernelSetNotReadable { .. } => refusal,
        other => EnvironmentLoadRefusal::KernelSetNotReadable {
            revision: revision.to_string(),
            cause: environment_load_refusal_text(&other),
        },
    }
}
