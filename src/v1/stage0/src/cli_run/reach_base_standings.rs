//! THE BASE SIDE OF THE PER-PR v2 CLAIM DIFFERENTIAL, run in a worktree at the diff base.
//!
//! `v2.workflow.required_floor` `claim_differential` judges a reached claim over a PAIR of
//! standings. The head standing is the floor's own run. This file produces the base standing for
//! exactly the reached identities, and it decides no verdict. Its caller is the required floor,
//! which runs this binary as a separate process with its working directory at a detached worktree
//! of the base. That keeps the base corpus out of the head process, whose prepared authority,
//! shared fills and judged-identity stores are per-process.
//!
//! WHAT A STANDING IS, and what refuses instead. `passed` and `failed` are verdicts: the claim
//! returned true, or returned false, threw or did not return a Bool. `not_declared` means the
//! base's resolved graph has no item under the identity, which `item_registry` answers at
//! `owner.decl` grain. A module the base cannot resolve is `failed` for every reached identity in
//! it, because the base was red there and a red base cannot hide a regression. Everything else
//! refuses the WHOLE arm: a budget interruption or completion over budget, a panic, an unresolved
//! host tool, a refused host effect, or a claim not attempted. None of those is a verdict, and
//! reading one as `failed` would turn a head failure into `still_red` or a head pass into
//! `repaired`, both non-blocking. An unmeasured base would then hide the regression this
//! differential exists to catch.
//!
//! THE BOUND, STATED. This runs HEAD's binary over the BASE tree. A change that also edits the
//! checker's Rust is judged here only for its .dag effect, which is the claim-verdict frontier
//! `gunbc.recurring_failure_mode.checker_change_landing_subject_keyed_to_changed_files` declares.

use super::*;
use std::collections::BTreeMap;

/// Whether this CI event plans the reach differential, as `v2.workflow.floor_subject_seed`
/// `reach_planned_for_event` decides. The host supplies the event and decides nothing.
pub fn reach_planned_for_event(source_roots: &[String], event_name: &str) -> Result<bool, String> {
    let root = process_workspace_root();
    let roots: Vec<String> = source_roots
        .iter()
        .map(|r| {
            let p = std::path::Path::new(r);
            if p.is_absolute() {
                r.clone()
            } else {
                root.join(p).to_string_lossy().into_owned()
            }
        })
        .collect();
    let entry = root.join("src/v2/workflow/floor_subject_seed.dag");
    let (graph, indices) = resolve_entry_graph_shared(&roots, &entry.to_string_lossy())
        .map_err(|e| format!("reach event scope authority resolve: {e}"))?;
    let ctx = make_eval_context(&graph, indices, v1_interpreter::ExecutionMode::Hermetic);
    match v1_interpreter::run_in_context_with_args(
        &ctx,
        "reach_planned_for_event",
        &[(
            Some("event_name".to_string()),
            v1_interpreter::str_value(event_name),
        )],
        false,
    ) {
        Ok(v1_interpreter::Value::Bool(b)) => Ok(b),
        other => Err(format!(
            "reach_planned_for_event returned {}",
            match other {
                Ok(v) => ctx.format_value(&v),
                Err(e) => e.to_string(),
            }
        )),
    }
}

/// The line a run that does not plan the reach differential prints, so its absence is announced.
pub fn reach_deferred_line(event_name: &str, reached_declarations: usize) -> String {
    format!(
        "[floor-phase] phase=reach-differential state=deferred_to_merge_group event={event_name:?} \
         reached_declarations={reached_declarations} -- the claims this change's body edits reach \
         are judged, head against base, by the merge group's run of the composed revision; this \
         run plans none of them"
    )
}

/// One reached identity's base standing, by the names `claim_standing_named` parses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BaseStanding {
    Passed,
    Failed,
    NotDeclared,
    /// The base outcome was not a verdict. Reported for this identity alone; the model decides
    /// what it means (`reach_claim_verdict`), and it is never read as `failed`.
    NotMeasured,
}

impl BaseStanding {
    pub fn name(&self) -> &'static str {
        match self {
            BaseStanding::Passed => "passed",
            BaseStanding::Failed => "failed",
            BaseStanding::NotDeclared => "not_declared",
            BaseStanding::NotMeasured => "not_measured",
        }
    }
}

/// Either every reached identity has a standing, or the arm refuses with the first identity
/// whose outcome was not a verdict.
pub enum ReachBaseArm {
    Completed(Vec<(String, BaseStanding)>),
    Refused { identity: String, cause: String },
}

/// The standing an outcome establishes, or why it establishes none.
pub fn base_standing_of(outcome: &ClaimOutcome) -> Result<BaseStanding, String> {
    match outcome {
        ClaimOutcome::Pass => Ok(BaseStanding::Passed),
        ClaimOutcome::Fail
        | ClaimOutcome::NotBool { .. }
        | ClaimOutcome::RuntimeError { .. }
        | ClaimOutcome::ExitFailure { .. } => Ok(BaseStanding::Failed),
        ClaimOutcome::BudgetInterrupted { .. } => Err("BudgetInterrupted".to_string()),
        ClaimOutcome::CompletedOverBudget { .. } => Err("CompletedOverBudget".to_string()),
        ClaimOutcome::HostToolUnresolved { .. } => Err("HostToolUnresolved".to_string()),
        ClaimOutcome::HostEffectRefused { .. } => Err("HostEffectRefused".to_string()),
        ClaimOutcome::Panicked { .. } => Err("Panicked".to_string()),
        ClaimOutcome::NotAttempted { .. } => Err("NotAttempted".to_string()),
    }
}

/// Evaluate exactly `identities` over `source_roots` (the base worktree's roots), one resolve per
/// module, under the floor's per-claim wall limit.
pub fn reach_base_standings(
    source_roots: &[String],
    identities: &[String],
    claim_wall_limit_ms: u64,
) -> Result<ReachBaseArm, String> {
    let module_index = try_build_module_index(source_roots)?;
    let mut by_module: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for identity in identities {
        let Some((module, _)) = identity.rsplit_once('.') else {
            return Err(format!(
                "reach identity {identity:?} has no module qualifier"
            ));
        };
        by_module
            .entry(module.to_string())
            .or_default()
            .push(identity.clone());
    }
    let index = super::entry_resolve::try_process_shared_index(source_roots)?;
    let mut standings: Vec<(String, BaseStanding)> = Vec::new();
    for (module, members) in by_module {
        let Some(source) = module_index.get(&module) else {
            for identity in members {
                standings.push((identity, BaseStanding::NotDeclared));
            }
            continue;
        };
        // The module index keys paths relative to the workspace root; anchor them there rather
        // than to the process cwd, which a caller may have set anywhere.
        let entry = {
            let p = std::path::Path::new(&source.path);
            if p.is_absolute() {
                source.path.clone()
            } else {
                process_workspace_root()
                    .join(p)
                    .to_string_lossy()
                    .into_owned()
            }
        };
        let (graph, source_indices) = match resolve_entry_with_index(&index, &entry) {
            Ok(resolved) => resolved,
            // ONLY A LOCATED COMPILE REFUSAL IS A RED BASE. A module that refuses with located
            // `file:line:col: error:` diagnostics was red at base, and a red base cannot hide a
            // regression, so its reached claims are `failed`. Any other resolve failure (a missing
            // file, an unreadable root, an index refusal) is the instrument failing, and reading it
            // as `failed` would turn a head failure into `still_red` and hide the regression. So
            // it refuses the whole arm. Found by this module's own control: a mis-anchored path
            // read as a red base.
            Err(cause) if is_located_compile_refusal(&cause) => {
                eprintln!(
                    "[reach-base] module={module} resolve=refused -- the base is red here, so \
                     every reached claim in it is failed at base: {}",
                    cause.lines().next().unwrap_or("")
                );
                for identity in members {
                    standings.push((identity, BaseStanding::Failed));
                }
                continue;
            }
            Err(cause) => {
                return Ok(ReachBaseArm::Refused {
                    identity: members.first().cloned().unwrap_or(module),
                    cause: format!(
                        "BaseResolveNotACompileRefusal {}",
                        cause.lines().next().unwrap_or("")
                    ),
                })
            }
        };
        let closure_subject = closure_subject_for_entry(&index, &entry)?;
        let ctx = make_eval_context_with_runtime_options(
            &graph,
            source_indices,
            v1_interpreter::ExecutionMode::Hermetic,
            None,
            None,
        );
        ctx.set_witness_wall_budget(Some(claim_wall_limit_ms));
        for identity in members {
            if !graph.item_registry.contains_key(&identity) {
                standings.push((identity, BaseStanding::NotDeclared));
                continue;
            }
            let function = identity
                .rsplit_once('.')
                .map(|(_, f)| f)
                .unwrap_or(&identity);
            let (outcome, _receipt) = run_claim_measured(&ctx, &closure_subject, function);
            v1_interpreter::eval_call_memo_frame_exit(&ctx);
            // A NON-VERDICT IS PER IDENTITY: one budget interruption used to void every other
            // claim's base (srv1, 2026-10-01). It is reported for this identity as not_measured,
            // and the arm refuses only when the instrument itself fails.
            let standing = match base_standing_of(&outcome) {
                Ok(standing) => standing,
                Err(cause) => {
                    eprintln!(
                        "[reach-base] identity={identity} standing=not_measured cause={cause}"
                    );
                    BaseStanding::NotMeasured
                }
            };
            if standing != BaseStanding::Passed {
                eprintln!("[reach-base] identity={identity} outcome={outcome:?}");
            }
            standings.push((identity, standing));
        }
    }
    standings.sort();
    Ok(ReachBaseArm::Completed(standings))
}

/// Whether a resolve failure is the compiler refusing located source: at least one line of the
/// `path:line:col: error: message` form the Strict gate renders for every blocking diagnostic.
pub fn is_located_compile_refusal(cause: &str) -> bool {
    cause.lines().any(|line| {
        let Some((location, _)) = line.split_once(": error: ") else {
            return false;
        };
        let mut parts = location.rsplitn(3, ':');
        let col = parts.next().unwrap_or("");
        let row = parts.next().unwrap_or("");
        let path = parts.next().unwrap_or("");
        path.ends_with(".dag")
            && !row.is_empty()
            && row.bytes().all(|b| b.is_ascii_digit())
            && !col.is_empty()
            && col.bytes().all(|b| b.is_ascii_digit())
    })
}

impl PartialOrd for BaseStanding {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for BaseStanding {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name().cmp(other.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ONE POOL FOR THE WHOLE MODULE. The process-global shared index holds a single resident
    /// pool (SharedIndexSecondResidentPool refuses a second), so every test here resolves against
    /// the same roots: the base fixture plus the absolute dag and src/v2 roots. Built once; the
    /// fixture module is inert to every other test.
    fn test_roots() -> Vec<String> {
        static ROOTS: std::sync::OnceLock<Vec<String>> = std::sync::OnceLock::new();
        ROOTS
            .get_or_init(|| {
                let root = process_workspace_root();
                let fx = root.join(format!("target/reach_base_fixture_{}", std::process::id()));
                let _ = std::fs::remove_dir_all(&fx);
                std::fs::create_dir_all(&fx).expect("fixture dir");
                std::fs::write(
                    fx.join("m.dag"),
                    "module rbase.m\n\nimport v2.std.logic { Bool }\n\n\
                     test fn holds() -> Bool {\n  true\n}\n\n\
                     test fn fails() -> Bool {\n  false\n}\n",
                )
                .expect("fixture source");
                std::iter::once(fx.to_string_lossy().into_owned())
                    .chain(
                        default_source_roots()
                            .iter()
                            .map(|r| root.join(r).to_string_lossy().into_owned()),
                    )
                    .collect()
            })
            .clone()
    }

    // A NON-VERDICT AT BASE REFUSES THE ARM. Reading any of these as `failed` would make a head
    // failure `still_red` and hide the regression, so each must be an Err, and only verdicts map.
    #[test]
    fn only_verdicts_are_base_standings() {
        assert_eq!(
            base_standing_of(&ClaimOutcome::Pass),
            Ok(BaseStanding::Passed)
        );
        assert_eq!(
            base_standing_of(&ClaimOutcome::Fail),
            Ok(BaseStanding::Failed)
        );
        assert!(is_located_compile_refusal(
            "dag/a/b.dag:12:3: error: type mismatch: expected 'X', got 'Y'"
        ));
        assert!(!is_located_compile_refusal(
            "entry file does not exist or is not a file: target/x/m.dag"
        ));
        assert!(base_standing_of(&ClaimOutcome::NotAttempted {
            halted_by: String::new()
        })
        .is_err());
    }

    /// One reached claim's verdict through the REAL model, as the floor decides it.
    fn verdict(base: &str, head: &str) -> (String, bool) {
        let root = process_workspace_root();
        let roots = test_roots();
        let entry = root.join("src/v2/workflow/required_floor.dag");
        let (graph, indices) =
            resolve_entry_graph_shared(&roots, &entry.to_string_lossy()).expect("authority");
        let ctx = make_eval_context(&graph, indices, v1_interpreter::ExecutionMode::Hermetic);
        let v = v1_interpreter::run_in_context_with_args(
            &ctx,
            "v2.workflow.required_floor.reach_claim_verdict",
            &[
                (
                    Some("identity".to_string()),
                    v1_interpreter::str_value("t.unrostered.claim"),
                ),
                (
                    Some("entry".to_string()),
                    v1_interpreter::str_value("t/unrostered.dag"),
                ),
                (
                    Some("function".to_string()),
                    v1_interpreter::str_value("claim"),
                ),
                (Some("base".to_string()), v1_interpreter::str_value(base)),
                (Some("head".to_string()), v1_interpreter::str_value(head)),
            ],
            false,
        )
        .expect("verdict");
        match &v {
            v1_interpreter::Value::Variant { fields, .. } => match (
                ctx.field(fields, "differential"),
                ctx.field(fields, "blocks"),
            ) {
                (Some(v1_interpreter::Value::Str(d)), Some(v1_interpreter::Value::Bool(b))) => {
                    (d.to_string(), *b)
                }
                _ => panic!("not a ReachVerdict: {}", ctx.format_value(&v)),
            },
            other => panic!("not a variant: {}", ctx.format_value(other)),
        }
    }

    // A HEAD-PASSED CLAIM IS NEVER RUN AT BASE AND CAN NEVER BLOCK (deep-ferret-305). The partition
    // is the model's: a passing head is exempt, a failing head goes to the base arm. And for the
    // exempt one, no base standing at all makes the real verdict block.
    #[test]
    fn a_head_passed_claim_is_never_run_at_base_and_never_blocks() {
        let root = process_workspace_root();
        let roots = test_roots();
        let entry = root.join("src/v2/workflow/required_floor.dag");
        let (graph, indices) =
            resolve_entry_graph_shared(&roots, &entry.to_string_lossy()).expect("authority");
        let ctx = make_eval_context(&graph, indices, v1_interpreter::ExecutionMode::Hermetic);
        let (needs_base, exempt) = super::super::required_floor_runner::reach_base_identities(
            &ctx,
            &[
                ("m.passes".to_string(), "passed".to_string()),
                ("m.fails".to_string(), "failed".to_string()),
                ("m.interrupted".to_string(), "not_measured".to_string()),
            ],
        )
        .expect("partition");
        assert_eq!(needs_base, vec!["m.fails".to_string()]);
        assert!(exempt.contains("m.passes") && !exempt.contains("m.fails"));
        // A head that was not measured is never sent to base: no base can make it a regression.
        assert!(exempt.contains("m.interrupted"));
        for base in ["passed", "failed", "not_declared"] {
            assert!(
                !verdict(base, "passed").1,
                "a passing head blocked under base {base}"
            );
        }
    }

    // THE PR-EVENT CONTROL (operator ruling, 2026-10-01): a pull request plans no reach and says so
    // with the count it would have reached; a merge group plans it. Through the real rule.
    #[test]
    fn a_pull_request_defers_the_reach_differential_and_a_merge_group_plans_it() {
        let roots = test_roots();
        assert!(!reach_planned_for_event(&roots, "pull_request").expect("rule"));
        assert!(reach_planned_for_event(&roots, "merge_group").expect("rule"));
        let line = reach_deferred_line("pull_request", 493);
        assert!(
            line.contains("phase=reach-differential state=deferred_to_merge_group"),
            "{line}"
        );
        assert!(line.contains("reached_declarations=493"), "{line}");
    }

    // REPORT-ONLY NEVER CLAIMS A BASE IT DID NOT RUN (review 73484). The line the floor prints
    // under a zero blocking budget is the not-measured arm and never says ran-at-merge-base; under
    // a positive budget it is the ran-at-merge-base arm. Both through the real model.
    #[test]
    fn report_only_output_never_says_the_base_ran() {
        let root = process_workspace_root();
        let roots = test_roots();
        let entry = root.join("src/v2/workflow/required_floor.dag");
        let (graph, indices) =
            resolve_entry_graph_shared(&roots, &entry.to_string_lossy()).expect("authority");
        let ctx = make_eval_context(&graph, indices, v1_interpreter::ExecutionMode::Hermetic);
        let report_only =
            super::super::required_floor_runner::reach_baseline_line(&ctx, 0, "abc").expect("line");
        assert!(
            report_only.starts_with("baseline=not-measured"),
            "{report_only}"
        );
        assert!(!report_only.contains("ran-at-merge-base"), "{report_only}");
        let blocking =
            super::super::required_floor_runner::reach_baseline_line(&ctx, 60_000, "abc")
                .expect("line");
        assert!(
            blocking.starts_with("baseline=missing-ran-at-merge-base main_sha=abc"),
            "{blocking}"
        );
    }

    // THE CONTROL REVIEW 72143 ASKED FOR, through the real base producer and the real model: a
    // reached claim NEW at base that fails at head BLOCKS, a pass-to-fail regression BLOCKS, and a
    // claim red on both sides does not. The base side is a fixture tree; the head standings are the
    // ones the floor's own fold would observe.
    #[test]
    fn a_new_failing_reach_claim_blocks_through_the_base_producer_and_the_model() {
        let roots = test_roots();
        let identities: Vec<String> = [
            "rbase.m.holds",
            "rbase.m.fails",
            "rbase.m.added",
            "rbase.gone.x",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
        let arm = reach_base_standings(&roots, &identities, 60_000);
        let standings = match arm.expect("base arm runs") {
            ReachBaseArm::Completed(s) => s,
            ReachBaseArm::Refused { identity, cause } => panic!("refused {identity}: {cause}"),
        };
        let standing = |id: &str| {
            standings
                .iter()
                .find(|(i, _)| i == id)
                .map(|(_, s)| s.name())
                .expect("every identity has a standing")
        };
        assert_eq!(standing("rbase.m.holds"), "passed");
        assert_eq!(standing("rbase.m.fails"), "failed");
        assert_eq!(standing("rbase.m.added"), "not_declared");
        assert_eq!(standing("rbase.gone.x"), "not_declared");
        // The head side: `added` is new and fails; `holds` regressed; `fails` is still red.
        assert_eq!(
            verdict(standing("rbase.m.added"), "failed"),
            ("new_claim".to_string(), true)
        );
        assert_eq!(
            verdict(standing("rbase.m.holds"), "failed"),
            ("regressed".to_string(), true)
        );
        assert_eq!(
            verdict(standing("rbase.m.fails"), "failed"),
            ("still_red".to_string(), false)
        );
        assert_eq!(
            verdict(standing("rbase.m.added"), "passed"),
            ("new_claim".to_string(), false)
        );
    }
}
