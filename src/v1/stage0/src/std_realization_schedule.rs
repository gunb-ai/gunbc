use self::CostBasis::*;
use self::Runnable::*;
use self::ScheduleLensViolation::*;
use crate::std_lens_verdict::LensVerdict::{Holds, Violation};
use crate::std_lens_verdict::LensVerdictLocus::ModuleWholeFile;
pub use crate::std_lens_verdict::{LensVerdict, LensVerdictDiagnostic, LensVerdictLocus};
use crate::std_measure::Quantity::Time;
pub use crate::std_measure::{byte_size, measure_count, time_measure, watt};
pub use crate::std_measure::{ByteSize, Measure, Quantity, Watt};
pub use crate::std_nat::Nat;
pub use crate::std_pareto::AxisGoal;
use crate::std_pareto::AxisGoal::*;
use crate::std_types::Bool::*;
pub use crate::std_types::{Bool, ContentHash, List};
use crate::v1_rt;
use crate::v1_rt::Witness;
use crate::v1_rt::Witness::{Holds, Violates};
use crate::NonEmptyBTreeSet;
use crate::NonEmptyVec;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[serde(tag = "_variant")]
pub enum CostBasis {
    Predicted,
    Measured,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CostAccount<S> {
    pub time: Rc<Measure<Time, S, Nat>>,
    pub space: Box<ByteSize>,
    pub power: Box<Watt>,
    pub basis: CostBasis,
}

pub fn cost_account_predicted_zero<S>() -> Rc<CostAccount<S>> {
    Rc::new(CostAccount {
        time: time_measure(0),
        space: Box::new(byte_size(0)),
        power: Box::new(watt(0)),
        basis: CostBasis::Predicted,
    })
}

pub fn cost_account_measured<S>(time: Rc<Measure<Time, S, Nat>>) -> Rc<CostAccount<S>> {
    Rc::new(CostAccount {
        time: time,
        space: Box::new(byte_size(0)),
        power: Box::new(watt(0)),
        basis: CostBasis::Measured,
    })
}

pub fn cost_account_time_count<S>(account: Rc<CostAccount<S>>) -> Nat {
    measure_count(account.time.clone())
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RealizationObjective {
    pub goals: Rc<Vec<AxisGoal>>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ScheduleWitnessEntry {
    pub entry: Rc<FreeMonoid<Nat>>,
    pub function: Rc<FreeMonoid<Nat>>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "_variant")]
pub enum Runnable {
    RunnableSingleClaim {
        entry: Rc<FreeMonoid<Nat>>,
        function: Rc<FreeMonoid<Nat>>,
    },
    RunnableDiscoveryBatch {
        source_roots: Rc<Vec<String>>,
        scan_dirs: Rc<Vec<String>>,
        explicit_entries: Rc<Vec<Rc<ScheduleWitnessEntry>>>,
        skip_unaffected_node_frontier: bool,
    },
}

pub type Schedule = Rc<crate::std_types::List<Rc<List>>>;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RealizationPlan<S> {
    pub target: ContentHash,
    pub objective: Rc<RealizationObjective>,
    pub schedule: Box<Schedule>,
    pub total: Rc<CostAccount<S>>,
}

pub fn runnable_step_label(r: Rc<Runnable>) -> Rc<FreeMonoid<Nat>> {
    match (*r).clone() {
        Runnable::RunnableSingleClaim { function: f, .. } => f.clone(),
        Runnable::RunnableDiscoveryBatch { .. } => "__discovery_corpus__".to_string(),
    }
}

pub fn schedule_batch_contains_label(
    batch: Rc<Vec<Rc<Runnable>>>,
    target: Rc<FreeMonoid<Nat>>,
) -> bool {
    batch
        .iter()
        .cloned()
        .fold(false, |acc: bool, r: Rc<Runnable>| {
            (acc || (crate::v2_std_text::host_string_text_to_rust_host(runnable_step_label(
                r.clone(),
            )) == crate::v2_std_text::host_string_text_to_rust_host(target.clone())))
        })
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "_variant")]
pub enum ScheduleLensViolation {
    EmptySchedule,
    CompileGateNotFirst { expected: Rc<FreeMonoid<Nat>> },
    CorpusBeforeCompile,
    SingleBatchOnly,
}
impl ScheduleLensViolation {
    pub fn expected(&self) -> Rc<FreeMonoid<Nat>> {
        match self {
            ScheduleLensViolation::EmptySchedule => panic!("no expected on unit variant"),
            ScheduleLensViolation::CompileGateNotFirst {
                expected: __val, ..
            } => __val.clone(),
            ScheduleLensViolation::CorpusBeforeCompile => panic!("no expected on unit variant"),
            ScheduleLensViolation::SingleBatchOnly => panic!("no expected on unit variant"),
        }
    }
}

pub fn schedule_lens_module() -> Rc<FreeMonoid<Nat>> {
    thread_local! {
        static CACHED: Rc<FreeMonoid<Nat>> = {
            "std.realization_schedule".to_string()
        };
    }
    CACHED.with(|c: &Rc<FreeMonoid<Nat>>| c.clone())
}

pub fn schedule_lens_violation_diagnostic(
    kind: Rc<ScheduleLensViolation>,
    compile_gate_fn: Rc<FreeMonoid<Nat>>,
) -> Rc<LensVerdictDiagnostic> {
    {
        let at = Rc::new(LensVerdictLocus::ModuleWholeFile {
            module_name: schedule_lens_module(),
        });
        match (*kind).clone() {
            ScheduleLensViolation::EmptySchedule => Rc::new(LensVerdictDiagnostic {
                reason: "schedule_lens_empty_schedule".to_string(),
                at: at,
            }),
            ScheduleLensViolation::CompileGateNotFirst {
                expected: expected, ..
            } => Rc::new(LensVerdictDiagnostic {
                reason: ("schedule_lens_compile_gate_not_first:".to_string() + expected.clone()),
                at: at,
            }),
            ScheduleLensViolation::CorpusBeforeCompile => Rc::new(LensVerdictDiagnostic {
                reason: "schedule_lens_corpus_before_compile".to_string(),
                at: at,
            }),
            ScheduleLensViolation::SingleBatchOnly => Rc::new(LensVerdictDiagnostic {
                reason: "schedule_lens_single_batch_only".to_string(),
                at: at,
            }),
        }
    }
}

pub fn schedule_lens_verdict_for_ci_floor<S>(
    plan: Rc<RealizationPlan<S>>,
    compile_gate_fn: Rc<FreeMonoid<Nat>>,
) -> Rc<LensVerdict> {
    if (plan.schedule.clone().length() == 0) {
        Rc::new(LensVerdict::Violation {
            diagnostic: schedule_lens_violation_diagnostic(
                Rc::new(ScheduleLensViolation::EmptySchedule),
                compile_gate_fn.clone(),
            ),
        })
    } else {
        if (plan.schedule.clone().length() < 2) {
            Rc::new(LensVerdict::Violation {
                diagnostic: schedule_lens_violation_diagnostic(
                    Rc::new(ScheduleLensViolation::SingleBatchOnly),
                    compile_gate_fn.clone(),
                ),
            })
        } else {
            {
                let batch0 = plan.schedule.clone().first();
                if (batch0.clone().length() != 1) {
                    Rc::new(LensVerdict::Violation {
                        diagnostic: schedule_lens_violation_diagnostic(
                            Rc::new(ScheduleLensViolation::CompileGateNotFirst {
                                expected: compile_gate_fn.clone(),
                            }),
                            compile_gate_fn.clone(),
                        ),
                    })
                } else {
                    if !schedule_batch_contains_label(batch0.clone(), compile_gate_fn.clone()) {
                        Rc::new(LensVerdict::Violation {
                            diagnostic: schedule_lens_violation_diagnostic(
                                Rc::new(ScheduleLensViolation::CompileGateNotFirst {
                                    expected: compile_gate_fn.clone(),
                                }),
                                compile_gate_fn.clone(),
                            ),
                        })
                    } else {
                        if schedule_batch_contains_label(
                            batch0.clone(),
                            "__discovery_corpus__".to_string(),
                        ) {
                            Rc::new(LensVerdict::Violation {
                                diagnostic: schedule_lens_violation_diagnostic(
                                    Rc::new(ScheduleLensViolation::CorpusBeforeCompile),
                                    compile_gate_fn.clone(),
                                ),
                            })
                        } else {
                            {
                                let batch1 = plan.schedule.clone().skip(1).first();
                                if (batch1.length() < 2) {
                                    Rc::new(LensVerdict::Violation {
                                        diagnostic: schedule_lens_violation_diagnostic(
                                            Rc::new(ScheduleLensViolation::SingleBatchOnly),
                                            compile_gate_fn.clone(),
                                        ),
                                    })
                                } else {
                                    Rc::new(LensVerdict::Holds)
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn schedule_generates_same_batch_count<S>(
    left: Rc<RealizationPlan<S>>,
    right: Rc<RealizationPlan<S>>,
) -> bool {
    (left.schedule.clone().length() == right.schedule.clone().length())
}

pub fn schedule_witness_entry_eq(a: Rc<ScheduleWitnessEntry>, b: Rc<ScheduleWitnessEntry>) -> bool {
    ((crate::v2_std_text::host_string_text_to_rust_host(a.entry.clone())
        == crate::v2_std_text::host_string_text_to_rust_host(b.entry.clone()))
        && (crate::v2_std_text::host_string_text_to_rust_host(a.function.clone())
            == crate::v2_std_text::host_string_text_to_rust_host(b.function.clone())))
}

pub fn string_list_eq(mut left: Rc<Vec<String>>, mut right: Rc<Vec<String>>) -> bool {
    loop {
        if (v1_rt::length(left.clone()) != v1_rt::length(right.clone())) {
            break false;
        } else {
            if (v1_rt::length(left.clone()) == 0) {
                break true;
            } else {
                if (left.clone().first().cloned().as_deref()
                    != right.clone().first().cloned().as_deref())
                {
                    break false;
                } else {
                    {
                        let __tco_0 =
                            Rc::new(left.iter().cloned().skip(1 as usize).collect::<Vec<_>>());
                        let __tco_1 =
                            Rc::new(right.iter().cloned().skip(1 as usize).collect::<Vec<_>>());
                        left = __tco_0;
                        right = __tco_1;
                        continue;
                    }
                }
            }
        }
    }
}

pub fn schedule_witness_entry_list_eq(
    mut left: Rc<Vec<Rc<ScheduleWitnessEntry>>>,
    mut right: Rc<Vec<Rc<ScheduleWitnessEntry>>>,
) -> bool {
    loop {
        if (v1_rt::length(left.clone()) != v1_rt::length(right.clone())) {
            break false;
        } else {
            if (v1_rt::length(left.clone()) == 0) {
                break true;
            } else {
                if !schedule_witness_entry_eq(
                    left.clone().first().cloned(),
                    right.clone().first().cloned(),
                ) {
                    break false;
                } else {
                    {
                        let __tco_0 =
                            Rc::new(left.iter().cloned().skip(1 as usize).collect::<Vec<_>>());
                        let __tco_1 =
                            Rc::new(right.iter().cloned().skip(1 as usize).collect::<Vec<_>>());
                        left = __tco_0;
                        right = __tco_1;
                        continue;
                    }
                }
            }
        }
    }
}

pub fn runnable_eq(left: Rc<Runnable>, right: Rc<Runnable>) -> bool {
    match (*left).clone() {
        Runnable::RunnableSingleClaim {
            entry: le,
            function: lf,
            ..
        } => match (*right).clone() {
            Runnable::RunnableSingleClaim {
                entry: re,
                function: rf,
                ..
            } => {
                ((crate::v2_std_text::host_string_text_to_rust_host(le.clone())
                    == crate::v2_std_text::host_string_text_to_rust_host(re.clone()))
                    && (crate::v2_std_text::host_string_text_to_rust_host(lf.clone())
                        == crate::v2_std_text::host_string_text_to_rust_host(rf.clone())))
            }
            Runnable::RunnableDiscoveryBatch { .. } => false,
        },
        Runnable::RunnableDiscoveryBatch {
            source_roots: lsr,
            scan_dirs: lsd,
            explicit_entries: lex,
            skip_unaffected_node_frontier: lskip,
            ..
        } => match (*right).clone() {
            Runnable::RunnableSingleClaim { .. } => false,
            Runnable::RunnableDiscoveryBatch {
                source_roots: rsr,
                scan_dirs: rsd,
                explicit_entries: rex,
                skip_unaffected_node_frontier: rskip,
                ..
            } => {
                (((string_list_eq(lsr.clone(), rsr.clone())
                    && string_list_eq(lsd.clone(), rsd.clone()))
                    && schedule_witness_entry_list_eq(lex.clone(), rex.clone()))
                    && (lskip.clone() == rskip.clone()))
            }
        },
    }
}

pub fn runnable_batch_eq(
    mut left: Rc<Vec<Rc<Runnable>>>,
    mut right: Rc<Vec<Rc<Runnable>>>,
) -> bool {
    loop {
        if (v1_rt::length(left.clone()) != v1_rt::length(right.clone())) {
            break false;
        } else {
            if (v1_rt::length(left.clone()) == 0) {
                break true;
            } else {
                if !runnable_eq(
                    left.clone().first().cloned(),
                    right.clone().first().cloned(),
                ) {
                    break false;
                } else {
                    {
                        let __tco_0 =
                            Rc::new(left.iter().cloned().skip(1 as usize).collect::<Vec<_>>());
                        let __tco_1 =
                            Rc::new(right.iter().cloned().skip(1 as usize).collect::<Vec<_>>());
                        left = __tco_0;
                        right = __tco_1;
                        continue;
                    }
                }
            }
        }
    }
}

pub fn schedule_eq(mut left: Schedule, mut right: Schedule) -> bool {
    loop {
        if (left.clone().length() != right.clone().length()) {
            break false;
        } else {
            if (left.clone().length() == 0) {
                break true;
            } else {
                if !runnable_batch_eq(left.clone().first(), right.clone().first()) {
                    break false;
                } else {
                    {
                        let __tco_0 = left.skip(1);
                        let __tco_1 = right.skip(1);
                        left = __tco_0;
                        right = __tco_1;
                        continue;
                    }
                }
            }
        }
    }
}

pub fn schedule_generates_identical_schedule<S>(
    plan: Rc<RealizationPlan<S>>,
    schedule: Schedule,
) -> bool {
    schedule_eq(plan.schedule.clone(), schedule)
}

pub struct Predicted;
pub struct Measured;
