//! Derived realization schedule: fixed concurrency from `std.realize_pack` over a
//! host budget and a derived space bound — replacing the retired AIMD memory governor.

use std::sync::{Arc, Mutex};

use crate::memory_governor::read_host_budget_bytes;
use crate::v1_compiler_compile;
use crate::v1_interpreter::{self, ExecutionMode, Value};

/// Kept so observation census can anchor the schedule receipt shape.
pub const REALIZATION_SCHEDULE_CENSUS_MARKER: &str = "[realization-schedule]";

/// Run 31916550287 receipt: first-slot worker RSS over pool baseline (~2.1 GiB).
pub const DECLARED_MEASURED_WORKER_SHARE_BYTES: u64 = 2_254_803_968;

const SCHEDULE_HIGH_WATER_NUM: u64 = 4;
const SCHEDULE_HIGH_WATER_DEN: u64 = 5;

fn schedule_emoji() -> bool {
    std::env::var("GITHUB_ACTIONS").as_deref() == Ok("true")
}

fn render_schedule_info_line(text: &str, emoji: bool) -> String {
    let glyph = if emoji { "🕐" } else { "◷" };
    format!("{glyph} {text}")
}

fn render_schedule_done_line(text: &str, emoji: bool) -> String {
    render_schedule_info_line(text, emoji)
}

fn width_from_declared_worker_share(budget_bytes: u64, independence_width: i64) -> usize {
    let usable = budget_bytes.saturating_mul(SCHEDULE_HIGH_WATER_NUM) / SCHEDULE_HIGH_WATER_DEN;
    let by_share = (usable / DECLARED_MEASURED_WORKER_SHARE_BYTES.max(1)) as i64;
    by_share.max(1).min(independence_width.max(1)) as usize
}

fn apply_budget_share_fallback(
    mut derived: DerivedScheduleWidth,
    budget_bytes: Option<i64>,
    independence_width: i64,
) -> DerivedScheduleWidth {
    if derived.verdict == "MaturationReserve" {
        if let Some(b) = budget_bytes.filter(|b| *b > 0) {
            derived.width = width_from_declared_worker_share(b as u64, independence_width);
            derived.verdict = "DeclaredWorkerShare".to_string();
        }
    }
    derived
}

/// The modeled packing verdict projected to a fixed worker count.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DerivedScheduleWidth {
    pub width: usize,
    pub verdict: String,
    pub max_derived_bound_bytes: Option<i64>,
}

impl DerivedScheduleWidth {
    pub fn refuse_if_budget_unreadable(&self) -> Option<String> {
        if self.verdict == "BudgetRefused" || self.width == 0 {
            Some(format!(
                "RealizationScheduleBudgetRefused: host memory budget unreadable — \
                 std.realize_pack refuses to fabricate a width (witness-realization P4)"
            ))
        } else {
            None
        }
    }
}

/// Marshal `Option<i64>` into the `.dag` `Int?` carrier `realize_advisory` expects.
fn optional_int_value(ctx: &v1_interpreter::InterpContext, v: Option<i64>) -> Value {
    use std::rc::Rc;
    match v {
        Some(n) => Value::Variant {
            type_name: ctx.sym("Optional"),
            variant_name: ctx.sym("Present"),
            fields: Rc::new(vec![(ctx.sym("value"), Value::Int(n))]),
        },
        None => Value::Variant {
            type_name: ctx.sym("Optional"),
            variant_name: ctx.sym("Absent"),
            fields: Rc::new(vec![]),
        },
    }
}

/// Call `std.realize_pack.realize_advisory` through the interpreter bridge (§2 — law stays modeled).
pub fn realize_pack_width_from_scalars(
    source_roots: &[String],
    derived_bytes: Option<i64>,
    budget_bytes: Option<i64>,
    independence_width: i64,
) -> Result<DerivedScheduleWidth, String> {
    let realize_ctx = crate::cli_run::resolve_entry_graph(source_roots, "dag/std/realize_pack.dag")
        .map_err(|e| format!("cannot load std.realize_pack: {e}"))?;
    let realize_ctx =
        crate::cli_run::make_eval_context(&realize_ctx.0, realize_ctx.1, ExecutionMode::Hermetic);
    let args = vec![
        (
            Some("derived_bytes".to_string()),
            optional_int_value(&realize_ctx, derived_bytes),
        ),
        (
            Some("budget_bytes".to_string()),
            optional_int_value(&realize_ctx, budget_bytes),
        ),
        (
            Some("independence_width".to_string()),
            Value::Int(independence_width),
        ),
    ];
    let result =
        v1_interpreter::run_in_context_with_args(&realize_ctx, "realize_advisory", &args, false)
            .map_err(|e| format!("realize_advisory: {e}"))?;
    match result {
        Value::Record { fields, .. } => {
            let width = match realize_ctx.field(&fields, "width") {
                Some(Value::Int(w)) => *w,
                _ => -1,
            };
            let verdict = match realize_ctx.field(&fields, "verdict") {
                Some(Value::Str(s)) => s.clone(),
                _ => "unknown".to_string(),
            };
            Ok(apply_budget_share_fallback(
                DerivedScheduleWidth {
                    width: width.max(0) as usize,
                    verdict,
                    max_derived_bound_bytes: derived_bytes,
                },
                budget_bytes,
                independence_width,
            ))
        }
        other => Err(format!(
            "realize_advisory returned {}, expected RealizeAdvisory record",
            realize_ctx.format_value(&other)
        )),
    }
}

/// Discovery corpus pool width stays serial until the shared-index crossover lands.
/// Width>1 spawns per-worker whole-tree indices (cli_run `FixedWidth` pool dissolve-on).
pub const DISCOVERY_POOL_WIDTH_UNTIL_SHARED_INDEX: usize = 1;

/// Derive the discovery batch width after roster assembly.
///
/// Per-entry derived-space scans are intentionally NOT run here: resolving every roster
/// entry to read `function_space_bytes` is O(entries) whole-tree work (tens of minutes on
/// the floor corpus). Opt-in per-row advisory logging uses `GUNBC_REALIZE_ADVISORY`.
pub fn derive_discovery_schedule_width(
    source_roots: &[String],
    _entry_function_pairs: &[(String, String)],
) -> Result<DerivedScheduleWidth, String> {
    let (budget_bytes, _source) = read_host_budget_bytes();
    let budget_i64 = budget_bytes.map(|b| b as i64);
    let independence = std::thread::available_parallelism()
        .map(|n| n.get() as i64)
        .unwrap_or(1);
    let mut derived =
        realize_pack_width_from_scalars(source_roots, None, budget_i64, independence)?;
    if derived.width > DISCOVERY_POOL_WIDTH_UNTIL_SHARED_INDEX {
        derived.width = DISCOVERY_POOL_WIDTH_UNTIL_SHARED_INDEX;
        if derived.verdict != "BudgetRefused" {
            derived.verdict = "DiscoverySerialUntilSharedIndex".to_string();
        }
    }
    Ok(derived)
}

/// Fixed-width concurrency plan for a whole floor walk.
#[derive(Debug)]
pub struct RealizationConcurrency {
    max_width: usize,
    budget_bytes: Option<u64>,
    budget_source: String,
    verdict: String,
    max_derived_bound_bytes: Option<i64>,
    active: Mutex<usize>,
    admissions: Mutex<u64>,
    max_active: Mutex<usize>,
}

impl RealizationConcurrency {
    /// Arm the walk schedule from host budget and hardware independence only.
    /// Per-discovery-batch width may be tighter and is computed separately.
    pub fn for_walk(hardware_max: usize) -> Result<Arc<Self>, String> {
        let roots = crate::cli_run::default_source_roots();
        let (budget_bytes, budget_source) = read_host_budget_bytes();
        let derived = realize_pack_width_from_scalars(
            &roots,
            None,
            budget_bytes.map(|b| b as i64),
            hardware_max as i64,
        )?;
        if let Some(msg) = derived.refuse_if_budget_unreadable() {
            return Err(msg);
        }
        let width = derived.width.max(1).min(hardware_max.max(1));
        let _ = REALIZATION_SCHEDULE_CENSUS_MARKER;
        eprintln!(
            "{}",
            render_schedule_info_line(
                &format!(
                    "realization schedule — budget={} source={} derived_width={} verdict={} max_hardware={}",
                    budget_bytes
                        .map(|b| b.to_string())
                        .unwrap_or_else(|| "unknown".into()),
                    budget_source,
                    width,
                    derived.verdict,
                    hardware_max,
                ),
                schedule_emoji(),
            )
        );
        Ok(Arc::new(RealizationConcurrency {
            max_width: width,
            budget_bytes,
            budget_source,
            verdict: derived.verdict.clone(),
            max_derived_bound_bytes: derived.max_derived_bound_bytes,
            active: Mutex::new(0),
            admissions: Mutex::new(0),
            max_active: Mutex::new(0),
        }))
    }

    pub fn budget_bytes(&self) -> Option<u64> {
        self.budget_bytes
    }

    pub fn current_target_width(&self) -> usize {
        self.max_width
    }

    pub fn try_admit(&self) -> AdmitDecision {
        let mut active = self.active.lock().unwrap();
        if *active >= self.max_width {
            return AdmitDecision::Hold;
        }
        *active += 1;
        *self.admissions.lock().unwrap() += 1;
        let mut max_active = self.max_active.lock().unwrap();
        if *active > *max_active {
            *max_active = *active;
        }
        AdmitDecision::Admit
    }

    pub fn admit_blocking(&self, _label: &str) {
        loop {
            if matches!(self.try_admit(), AdmitDecision::Admit) {
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
    }

    pub fn release_slot(&self) {
        let mut active = self.active.lock().unwrap();
        *active = active.saturating_sub(1);
    }

    pub fn receipt_line(&self) -> String {
        let _ = REALIZATION_SCHEDULE_CENSUS_MARKER;
        let admissions = *self.admissions.lock().unwrap();
        let max_active = *self.max_active.lock().unwrap();
        render_schedule_done_line(
            &format!(
                "realization schedule receipt — budget={} source={} scheduled_width={} \
                 verdict={} max_derived_bound={} admissions={} max_active={}",
                self.budget_bytes
                    .map(|b| b.to_string())
                    .unwrap_or_else(|| "unknown".into()),
                self.budget_source,
                self.max_width,
                self.verdict,
                self.max_derived_bound_bytes
                    .map(|b| b.to_string())
                    .unwrap_or_else(|| "unknown".into()),
                admissions,
                max_active,
            ),
            schedule_emoji(),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdmitDecision {
    Admit,
    Hold,
}

/// RAII slot guard for a fixed derived schedule.
pub struct RealizationSlot {
    schedule: Arc<RealizationConcurrency>,
}

impl RealizationSlot {
    pub fn acquire_blocking(schedule: &Arc<RealizationConcurrency>, label: &str) -> Self {
        schedule.admit_blocking(label);
        RealizationSlot {
            schedule: schedule.clone(),
        }
    }

    pub fn note_unit_complete(&self) {
        // Fixed schedule: no AIMD observe pass; release happens on drop only.
    }
}

impl Drop for RealizationSlot {
    fn drop(&mut self) {
        self.schedule.release_slot();
    }
}
