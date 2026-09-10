//! THE (PHASE, LANE) PAIR JOIN — the lane-ownership half of the phase-roster wall.
//!
//! The variant-set join (`phase_roster_findings` in the claim_executor bin) reads variant names
//! off the declaration index, so it can join phase EXISTENCE but not lane OWNERSHIP: a match arm
//! is not a declaration. Lane ownership leaves the authority only by evaluation, through
//! `gunbc.required_ci_phase_roster` `required_ci_lane_phase_rows`.
//!
//! Why this half exists: the required workflow is three independently selected lanes, and the
//! v2-native lane's isolation is load-bearing. A host edit that mapped `V2NativePhase` to another
//! lane while the authority still rosters it in `v2-native` would leave the native job selecting
//! zero phases — `phases_run=0`, no phase failure, a green required job that executed no native
//! test. Joining (phase, lane) pairs in both directions makes that edit red at run start in
//! every lane; the empty-expectation refusal and the exact ran-set check (both in the bin, over
//! the rows this module decodes) make a zero-phase selected lane refuse rather than report
//! success over nothing.

use crate::v1_interpreter::{self, Value};

/// The roster authority's entry file — the same module the variant-set join reads off the
/// declaration index, here consumed by evaluation instead.
const LANE_ROSTER_AUTHORITY_ENTRY: super::WorkspaceRootRelativeEntry =
    super::WorkspaceRootRelativeEntry("dag/gunbc/required_ci_phase_roster.dag");

/// One authority-rostered (lane, phase) pair, both names in the CLI spelling the host matches on.
pub struct LanePhaseRow {
    pub lane: String,
    pub phase: String,
}

fn lane_phase_row_from_value(
    ctx: &v1_interpreter::InterpContext,
    val: &Value,
) -> Result<LanePhaseRow, String> {
    let Value::Record { fields, .. } = val else {
        return Err(format!(
            "expected RequiredCiLanePhaseRow record, got `{}`",
            ctx.format_value(val)
        ));
    };
    let lane = match ctx.field(fields, "lane") {
        Some(Value::Str(s)) => s.to_string(),
        _ => return Err("RequiredCiLanePhaseRow missing `lane`".to_string()),
    };
    let phase = match ctx.field(fields, "phase") {
        Some(Value::Str(s)) => s.to_string(),
        _ => return Err("RequiredCiLanePhaseRow missing `phase`".to_string()),
    };
    Ok(LanePhaseRow { lane, phase })
}

/// Evaluate the authority's `required_ci_lane_phase_rows` and decode every row. Any failure —
/// the module absent, the function unevaluable, a row of the wrong shape — is a refusal, never
/// an empty roster: an unreadable authority is the state in which nothing is checking lane
/// ownership, not permission to proceed.
pub fn authority_lane_phase_rows(source_roots: &[String]) -> Result<Vec<LanePhaseRow>, String> {
    let (graph, indices) =
        super::resolve_workspace_entry(source_roots, LANE_ROSTER_AUTHORITY_ENTRY)?;
    let ctx = super::make_eval_context(&graph, indices, v1_interpreter::ExecutionMode::Hermetic);
    let value = v1_interpreter::run_in_context_with_args(
        &ctx,
        "gunbc.required_ci_phase_roster.required_ci_lane_phase_rows",
        &[],
        false,
    )
    .map_err(|e| format!("lane-roster authority evaluation refused: {e}"))?;
    match &value {
        Value::List(items) => {
            let mut out = Vec::with_capacity(items.len());
            for item in items.iter() {
                out.push(lane_phase_row_from_value(&ctx, item)?);
            }
            Ok(out)
        }
        other => Err(format!(
            "required_ci_lane_phase_rows returned `{}`, expected a List",
            ctx.format_value(other)
        )),
    }
}
