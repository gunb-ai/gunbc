//! Host transport rows for `v4.compiler.emit_host` (`run_emit_host_rust` / `run_emit_host_python` / `run_emit_host_go`).
//!
//! **Modeled authority:** `src/v4/compiler/emit_host.dag` — executable host-process boundary is
//! `tools/emit_host_runner`; substrate `.dag` assembles `EmitHostRunReceipt` from typed host facts.
//! This module is the W3 bridge exercised by integration tests until T-22 substrate eval dispatches
//! host transport directly (dissolves `emit_host_transport_not_wired`).

use emit_host_runner::{
    host_logical_run_from_exit, run_emit_host_go, run_emit_host_python, run_emit_host_rust,
    EmitHostFixtureInputs, EmitHostRunReceipt, HostExit, HostSetupFailure,
    RuntimeValueParseFailure,
};

/// MVP-2 / `eval_runtime_mvp` alignment: five stdout bytes denote runtime value `5`.
pub const MVP2_RUNTIME_VALUE_FIVE_BYTES: [u8; 5] = [0, 0, 0, 0, 0];

/// Rung-4 emit-vs-eval verdict at the Rust transport row (mirrors `run_test_claim_emit_vs_eval`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EmitHostEmitVsEvalVerdict {
    Pass,
    FailValueMismatch {
        host_receipt: EmitHostRunReceipt,
        host_stdout: Vec<u8>,
        expected_bytes: [u8; 5],
    },
    FailParse {
        host_receipt: EmitHostRunReceipt,
        parse: RuntimeValueParseFailure,
    },
    FailHostExit {
        host_receipt: EmitHostRunReceipt,
    },
}

/// Release-minimum L2 parity verdict for Rust/Python/Go over one runtime-value fixture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EmitHostCrossTargetParityVerdict {
    Pass,
    FailSetup {
        target: &'static str,
        setup: HostSetupFailure,
    },
    FailHostExit {
        target: &'static str,
        host_receipt: EmitHostRunReceipt,
    },
    FailParse {
        target: &'static str,
        host_receipt: EmitHostRunReceipt,
        parse: RuntimeValueParseFailure,
    },
    FailBehaviorMismatch {
        rust_stdout: Vec<u8>,
        python_stdout: Vec<u8>,
        go_stdout: Vec<u8>,
    },
}

/// Emit-vs-eval / harness boundary: both fixture pins must be present (fail-closed).
fn validate_emit_vs_eval_fixture_inputs(
    inputs: &EmitHostFixtureInputs,
) -> Result<(), emit_host_runner::HostSetupFailure> {
    emit_host_runner::validate_emit_host_fixture_inputs(inputs)
}

/// Host-process transport: compile + run emitted Rust, returning the runner receipt.
pub fn run_emit_host_rust_transport(
    source: &str,
    inputs: &EmitHostFixtureInputs,
    work_dir: &std::path::Path,
) -> Result<EmitHostRunReceipt, emit_host_runner::HostSetupFailure> {
    validate_emit_vs_eval_fixture_inputs(inputs)?;
    run_emit_host_rust(source, &inputs.transport(), work_dir)
}

/// Host-process transport: run emitted Python via `python3`, returning the runner receipt.
pub fn run_emit_host_python_transport(
    source: &str,
    inputs: &EmitHostFixtureInputs,
    work_dir: &std::path::Path,
) -> Result<EmitHostRunReceipt, emit_host_runner::HostSetupFailure> {
    validate_emit_vs_eval_fixture_inputs(inputs)?;
    run_emit_host_python(source, &inputs.transport(), work_dir)
}

/// Host-process transport: run emitted Go via `go run`, returning the runner receipt.
pub fn run_emit_host_go_transport(
    source: &str,
    inputs: &EmitHostFixtureInputs,
    work_dir: &std::path::Path,
) -> Result<EmitHostRunReceipt, emit_host_runner::HostSetupFailure> {
    validate_emit_vs_eval_fixture_inputs(inputs)?;
    run_emit_host_go(source, &inputs.transport(), work_dir)
}

/// True when the host exit witness is `Holds` (logical child succeeded).
pub fn host_exit_holds(exit: &HostExit) -> bool {
    exit.exit_holds()
}

/// Project logical stdout bytes when exit `Holds`; fail-closed otherwise.
pub fn host_stdout_bytes(exit: &HostExit, stdout_bytes: Vec<u8>) -> Option<Vec<u8>> {
    host_logical_run_from_exit(exit, stdout_bytes).map(|run| run.stdout_bytes)
}

fn emit_vs_eval_mvp2_verdict_from_receipt(
    receipt: EmitHostRunReceipt,
    expected_bytes: [u8; 5],
    parse_stdout: fn(&[u8]) -> Result<(), RuntimeValueParseFailure>,
) -> EmitHostEmitVsEvalVerdict {
    if !host_exit_holds(&receipt.exit) {
        return EmitHostEmitVsEvalVerdict::FailHostExit {
            host_receipt: receipt,
        };
    }
    let stdout = match host_stdout_bytes(&receipt.exit, receipt.stdout_bytes.clone()) {
        Some(bytes) => bytes,
        None => {
            return EmitHostEmitVsEvalVerdict::FailHostExit {
                host_receipt: receipt,
            };
        }
    };
    if let Err(parse) = parse_stdout(&stdout) {
        return EmitHostEmitVsEvalVerdict::FailParse {
            host_receipt: receipt,
            parse,
        };
    }
    if stdout == expected_bytes {
        EmitHostEmitVsEvalVerdict::Pass
    } else {
        EmitHostEmitVsEvalVerdict::FailValueMismatch {
            host_receipt: receipt,
            host_stdout: stdout,
            expected_bytes,
        }
    }
}

/// W3 rung-4 / W3.4 tranche-1 rung-6 rows: real `run_emit_host_rust` transport + MVP-2 five-byte check.
///
/// Matches `run_test_claim_emit_vs_eval_for_claim` / `run_test_claim_emit_vs_eval_verdict` in
/// `emit_host.dag` for the rust authority pin (host stdout parsed then compared to eval literal `5`).
pub fn run_emit_vs_eval_mvp2_transport(
    emitted_source: &str,
    inputs: &EmitHostFixtureInputs,
    work_dir: &std::path::Path,
    expected_bytes: [u8; 5],
) -> Result<EmitHostEmitVsEvalVerdict, emit_host_runner::HostSetupFailure> {
    let receipt = run_emit_host_rust_transport(emitted_source, inputs, work_dir)?;
    Ok(emit_vs_eval_mvp2_verdict_from_receipt(
        receipt,
        expected_bytes,
        emit_host_runner::runtime_value_parse_rust,
    ))
}

/// W3.4 tranche-1: real `run_emit_host_python` transport + MVP-2 five-byte stdout (same contract
/// as rung-4 rust row). Executable bridge until per-law emit + T-22 substrate dispatch land.
pub fn run_emit_vs_eval_mvp2_python_transport(
    emitted_source: &str,
    inputs: &EmitHostFixtureInputs,
    work_dir: &std::path::Path,
    expected_bytes: [u8; 5],
) -> Result<EmitHostEmitVsEvalVerdict, emit_host_runner::HostSetupFailure> {
    let receipt = run_emit_host_python_transport(emitted_source, inputs, work_dir)?;
    Ok(emit_vs_eval_mvp2_verdict_from_receipt(
        receipt,
        expected_bytes,
        emit_host_runner::runtime_value_parse_python,
    ))
}

/// W3.3: real `run_emit_host_go` transport + MVP-2 five-byte stdout (cross-target parity row).
pub fn run_emit_vs_eval_mvp2_go_transport(
    emitted_source: &str,
    inputs: &EmitHostFixtureInputs,
    work_dir: &std::path::Path,
    expected_bytes: [u8; 5],
) -> Result<EmitHostEmitVsEvalVerdict, emit_host_runner::HostSetupFailure> {
    let receipt = run_emit_host_go_transport(emitted_source, inputs, work_dir)?;
    Ok(emit_vs_eval_mvp2_verdict_from_receipt(
        receipt,
        expected_bytes,
        emit_host_runner::runtime_value_parse_go,
    ))
}

fn cross_target_stdout_bytes(
    target: &'static str,
    receipt: EmitHostRunReceipt,
    parse_stdout: fn(&[u8]) -> Result<(), RuntimeValueParseFailure>,
) -> Result<Vec<u8>, EmitHostCrossTargetParityVerdict> {
    if !host_exit_holds(&receipt.exit) {
        return Err(EmitHostCrossTargetParityVerdict::FailHostExit {
            target,
            host_receipt: receipt,
        });
    }
    let stdout = match host_stdout_bytes(&receipt.exit, receipt.stdout_bytes.clone()) {
        Some(bytes) => bytes,
        None => {
            return Err(EmitHostCrossTargetParityVerdict::FailHostExit {
                target,
                host_receipt: receipt,
            });
        }
    };
    if let Err(parse) = parse_stdout(&stdout) {
        return Err(EmitHostCrossTargetParityVerdict::FailParse {
            target,
            host_receipt: receipt,
            parse,
        });
    }
    Ok(stdout)
}

fn cross_target_parity_verdict_from_stdout(
    rust_stdout: Vec<u8>,
    python_stdout: Vec<u8>,
    go_stdout: Vec<u8>,
) -> EmitHostCrossTargetParityVerdict {
    if rust_stdout == python_stdout && rust_stdout == go_stdout {
        EmitHostCrossTargetParityVerdict::Pass
    } else {
        EmitHostCrossTargetParityVerdict::FailBehaviorMismatch {
            rust_stdout,
            python_stdout,
            go_stdout,
        }
    }
}

type EmitHostRunner = fn(
    &str,
    &EmitHostFixtureInputs,
    &std::path::Path,
) -> Result<EmitHostRunReceipt, HostSetupFailure>;

struct CrossTargetMvp2Runners {
    rust: EmitHostRunner,
    python: EmitHostRunner,
    go: EmitHostRunner,
}

fn run_cross_target_mvp2_python_parity_with_runners(
    rust_source: &str,
    python_source: &str,
    go_source: &str,
    inputs: &EmitHostFixtureInputs,
    rust_work_dir: &std::path::Path,
    python_work_dir: &std::path::Path,
    go_work_dir: &std::path::Path,
    runners: CrossTargetMvp2Runners,
) -> EmitHostCrossTargetParityVerdict {
    let rust_receipt = match (runners.rust)(rust_source, inputs, rust_work_dir) {
        Ok(receipt) => receipt,
        Err(setup) => {
            return EmitHostCrossTargetParityVerdict::FailSetup {
                target: "rust",
                setup,
            };
        }
    };
    let python_receipt = match (runners.python)(python_source, inputs, python_work_dir) {
        Ok(receipt) => receipt,
        Err(setup) => {
            return EmitHostCrossTargetParityVerdict::FailSetup {
                target: "python",
                setup,
            };
        }
    };
    let go_receipt = match (runners.go)(go_source, inputs, go_work_dir) {
        Ok(receipt) => receipt,
        Err(setup) => {
            return EmitHostCrossTargetParityVerdict::FailSetup {
                target: "go",
                setup,
            };
        }
    };

    let rust_stdout = match cross_target_stdout_bytes(
        "rust",
        rust_receipt,
        emit_host_runner::runtime_value_parse_rust,
    ) {
        Ok(stdout) => stdout,
        Err(verdict) => return verdict,
    };
    let python_stdout = match cross_target_stdout_bytes(
        "python",
        python_receipt,
        emit_host_runner::runtime_value_parse_python,
    ) {
        Ok(stdout) => stdout,
        Err(verdict) => return verdict,
    };
    let go_stdout =
        match cross_target_stdout_bytes("go", go_receipt, emit_host_runner::runtime_value_parse_go)
        {
            Ok(stdout) => stdout,
            Err(verdict) => return verdict,
        };

    cross_target_parity_verdict_from_stdout(rust_stdout, python_stdout, go_stdout)
}

/// L2 cross-target behavioral parity: Rust, Python, and Go must all execute, parse into the
/// same MVP-2 runtime-value carrier, and agree on observed stdout for the same fixture subject.
pub fn run_cross_target_mvp2_python_parity_transport(
    rust_source: &str,
    python_source: &str,
    go_source: &str,
    inputs: &EmitHostFixtureInputs,
    rust_work_dir: &std::path::Path,
    python_work_dir: &std::path::Path,
    go_work_dir: &std::path::Path,
) -> EmitHostCrossTargetParityVerdict {
    if let Err(setup) = validate_emit_vs_eval_fixture_inputs(inputs) {
        return EmitHostCrossTargetParityVerdict::FailSetup {
            target: "rust",
            setup,
        };
    }
    run_cross_target_mvp2_python_parity_with_runners(
        rust_source,
        python_source,
        go_source,
        inputs,
        rust_work_dir,
        python_work_dir,
        go_work_dir,
        CrossTargetMvp2Runners {
            rust: run_emit_host_rust_transport,
            python: run_emit_host_python_transport,
            go: run_emit_host_go_transport,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use emit_host_runner::{
        default_work_dir, BuildLog, HostLogicalFailure, HostPhase, HostSetupFailure, HostStream,
    };

    const FIXTURE_SOURCE_PASS: &str =
        "fn main() { let _ = std::io::Write::write_all(&mut std::io::stdout(), &[0u8; 5]); }";
    const FIXTURE_SOURCE_MISMATCH: &str =
        "fn main() { let _ = std::io::Write::write_all(&mut std::io::stdout(), &[1,2,3,4,5]); }";
    const FIXTURE_SOURCE_PARSE_FAIL: &str =
        "fn main() { let _ = std::io::Write::write_all(&mut std::io::stdout(), &[0u8; 3]); }";

    fn mvp2_inputs() -> EmitHostFixtureInputs {
        EmitHostFixtureInputs {
            claim_input_root: "bridge_claim_input".into(),
            expected_eval_root: "bridge_expected_eval".into(),
        }
    }

    fn receipt_with_stdout(stdout_bytes: &[u8]) -> EmitHostRunReceipt {
        EmitHostRunReceipt {
            source_text: "fixture".to_string(),
            exit: HostExit::holds(0),
            stdout_bytes: stdout_bytes.to_vec(),
            stderr_bytes: Vec::new(),
            build_log: BuildLog { lines: Vec::new() },
        }
    }

    fn receipt_with_exit_failure() -> EmitHostRunReceipt {
        EmitHostRunReceipt {
            source_text: "fixture".to_string(),
            exit: HostExit::logical_violation(HostLogicalFailure::ExitedNonzero {
                phase: HostPhase::FixtureRun,
                code: Some(1),
            }),
            stdout_bytes: Vec::new(),
            stderr_bytes: b"runtime failure".to_vec(),
            build_log: BuildLog { lines: Vec::new() },
        }
    }

    fn fake_transport_pass(
        _source: &str,
        _inputs: &EmitHostFixtureInputs,
        _work_dir: &std::path::Path,
    ) -> Result<EmitHostRunReceipt, HostSetupFailure> {
        Ok(receipt_with_stdout(&MVP2_RUNTIME_VALUE_FIVE_BYTES))
    }

    fn fake_transport_python_mismatch(
        _source: &str,
        _inputs: &EmitHostFixtureInputs,
        _work_dir: &std::path::Path,
    ) -> Result<EmitHostRunReceipt, HostSetupFailure> {
        Ok(receipt_with_stdout(&[1, 2, 3, 4, 5]))
    }

    fn fake_transport_python_parse_fail(
        _source: &str,
        _inputs: &EmitHostFixtureInputs,
        _work_dir: &std::path::Path,
    ) -> Result<EmitHostRunReceipt, HostSetupFailure> {
        Ok(receipt_with_stdout(&[0, 0, 0]))
    }

    fn fake_transport_python_exit_fail(
        _source: &str,
        _inputs: &EmitHostFixtureInputs,
        _work_dir: &std::path::Path,
    ) -> Result<EmitHostRunReceipt, HostSetupFailure> {
        Ok(receipt_with_exit_failure())
    }

    fn fake_transport_python_setup_fail(
        _source: &str,
        _inputs: &EmitHostFixtureInputs,
        _work_dir: &std::path::Path,
    ) -> Result<EmitHostRunReceipt, HostSetupFailure> {
        Err(HostSetupFailure::StreamReadFailed {
            phase: HostPhase::FixtureRun,
            stream: HostStream::Stdout,
            source: "setup failed".to_string(),
        })
    }

    fn run_fake_cross_target(runners: CrossTargetMvp2Runners) -> EmitHostCrossTargetParityVerdict {
        run_cross_target_mvp2_python_parity_with_runners(
            "rust",
            "python",
            "go",
            &mvp2_inputs(),
            std::path::Path::new("rust"),
            std::path::Path::new("python"),
            std::path::Path::new("go"),
            runners,
        )
    }

    #[test]
    fn bridge_runs_fixture_and_holds_exit() {
        let work_dir = default_work_dir(&format!("gunbc_emit_host_bridge_{}", std::process::id()));
        let receipt = run_emit_host_rust_transport(FIXTURE_SOURCE_PASS, &mvp2_inputs(), &work_dir)
            .expect("transport");
        assert!(host_exit_holds(&receipt.exit));
        let stdout =
            host_stdout_bytes(&receipt.exit, receipt.stdout_bytes.clone()).expect("logical stdout");
        assert_eq!(stdout.len(), 5);
        emit_host_runner::runtime_value_parse_rust(&stdout).expect("parse");
    }

    #[test]
    fn emit_vs_eval_mvp2_transport_passes_for_five_zero_bytes() {
        let work_dir = default_work_dir(&format!("gunbc_emit_vs_eval_pass_{}", std::process::id()));
        let verdict = run_emit_vs_eval_mvp2_transport(
            FIXTURE_SOURCE_PASS,
            &mvp2_inputs(),
            &work_dir,
            MVP2_RUNTIME_VALUE_FIVE_BYTES,
        )
        .expect("transport setup");
        assert_eq!(verdict, EmitHostEmitVsEvalVerdict::Pass);
    }

    #[test]
    fn emit_vs_eval_mvp2_transport_fails_with_host_receipt_on_value_mismatch() {
        let work_dir = default_work_dir(&format!("gunbc_emit_vs_eval_fail_{}", std::process::id()));
        let verdict = run_emit_vs_eval_mvp2_transport(
            FIXTURE_SOURCE_MISMATCH,
            &mvp2_inputs(),
            &work_dir,
            MVP2_RUNTIME_VALUE_FIVE_BYTES,
        )
        .expect("transport setup");
        match verdict {
            EmitHostEmitVsEvalVerdict::FailValueMismatch {
                host_receipt,
                host_stdout,
                expected_bytes,
            } => {
                assert!(host_exit_holds(&host_receipt.exit));
                assert_eq!(host_stdout, [1, 2, 3, 4, 5]);
                assert_eq!(expected_bytes, MVP2_RUNTIME_VALUE_FIVE_BYTES);
            }
            other => panic!("expected FailValueMismatch with Host evidence, got {other:?}"),
        }
    }

    #[test]
    fn emit_vs_eval_mvp2_transport_fails_on_unparsable_stdout() {
        let work_dir =
            default_work_dir(&format!("gunbc_emit_vs_eval_parse_{}", std::process::id()));
        let verdict = run_emit_vs_eval_mvp2_transport(
            FIXTURE_SOURCE_PARSE_FAIL,
            &mvp2_inputs(),
            &work_dir,
            MVP2_RUNTIME_VALUE_FIVE_BYTES,
        )
        .expect("transport setup");
        assert!(matches!(
            verdict,
            EmitHostEmitVsEvalVerdict::FailParse { .. }
        ));
    }

    const PYTHON_FIXTURE_SOURCE_PASS: &str = "import sys\nsys.stdout.buffer.write(b'\\x00' * 5)\n";

    #[test]
    fn bridge_python_transport_builds_runs_and_parses_stdout() {
        let work_dir =
            default_work_dir(&format!("gunbc_emit_host_py_bridge_{}", std::process::id()));
        let receipt =
            run_emit_host_python_transport(PYTHON_FIXTURE_SOURCE_PASS, &mvp2_inputs(), &work_dir)
                .expect("transport");
        assert!(host_exit_holds(&receipt.exit));
        let stdout =
            host_stdout_bytes(&receipt.exit, receipt.stdout_bytes.clone()).expect("logical stdout");
        emit_host_runner::runtime_value_parse_python(&stdout).expect("parse");
    }

    #[test]
    fn emit_vs_eval_mvp2_python_transport_passes_for_five_zero_bytes() {
        let work_dir = default_work_dir(&format!(
            "gunbc_emit_vs_eval_py_pass_{}",
            std::process::id()
        ));
        let verdict = run_emit_vs_eval_mvp2_python_transport(
            PYTHON_FIXTURE_SOURCE_PASS,
            &mvp2_inputs(),
            &work_dir,
            MVP2_RUNTIME_VALUE_FIVE_BYTES,
        )
        .expect("transport setup");
        assert_eq!(verdict, EmitHostEmitVsEvalVerdict::Pass);
    }

    const GO_FIXTURE_SOURCE_PASS: &str =
        "package main\nimport \"os\"\nfunc main() { _, _ = os.Stdout.Write(make([]byte, 5)) }\n";

    #[test]
    fn bridge_go_transport_builds_runs_and_parses_stdout() {
        let work_dir =
            default_work_dir(&format!("gunbc_emit_host_go_bridge_{}", std::process::id()));
        let receipt = run_emit_host_go_transport(GO_FIXTURE_SOURCE_PASS, &mvp2_inputs(), &work_dir)
            .expect("transport");
        assert!(host_exit_holds(&receipt.exit));
        let stdout =
            host_stdout_bytes(&receipt.exit, receipt.stdout_bytes.clone()).expect("logical stdout");
        emit_host_runner::runtime_value_parse_go(&stdout).expect("parse");
    }

    #[test]
    fn emit_vs_eval_mvp2_go_transport_passes_for_five_zero_bytes() {
        let work_dir = default_work_dir(&format!(
            "gunbc_emit_vs_eval_go_pass_{}",
            std::process::id()
        ));
        let verdict = run_emit_vs_eval_mvp2_go_transport(
            GO_FIXTURE_SOURCE_PASS,
            &mvp2_inputs(),
            &work_dir,
            MVP2_RUNTIME_VALUE_FIVE_BYTES,
        )
        .expect("transport setup");
        assert_eq!(verdict, EmitHostEmitVsEvalVerdict::Pass);
    }

    #[test]
    fn cross_target_mvp2_python_parity_classifies_matching_three_target_stdout() {
        let verdict = cross_target_parity_verdict_from_stdout(
            MVP2_RUNTIME_VALUE_FIVE_BYTES.to_vec(),
            MVP2_RUNTIME_VALUE_FIVE_BYTES.to_vec(),
            MVP2_RUNTIME_VALUE_FIVE_BYTES.to_vec(),
        );
        assert_eq!(verdict, EmitHostCrossTargetParityVerdict::Pass);
    }

    #[test]
    fn cross_target_mvp2_python_parity_classifies_python_behavior_mismatch() {
        let verdict = cross_target_parity_verdict_from_stdout(
            MVP2_RUNTIME_VALUE_FIVE_BYTES.to_vec(),
            vec![1, 2, 3, 4, 5],
            MVP2_RUNTIME_VALUE_FIVE_BYTES.to_vec(),
        );
        match verdict {
            EmitHostCrossTargetParityVerdict::FailBehaviorMismatch {
                rust_stdout,
                python_stdout,
                go_stdout,
            } => {
                assert_eq!(rust_stdout, MVP2_RUNTIME_VALUE_FIVE_BYTES);
                assert_eq!(python_stdout, [1, 2, 3, 4, 5]);
                assert_eq!(go_stdout, MVP2_RUNTIME_VALUE_FIVE_BYTES);
            }
            other => panic!("expected Python behavior mismatch, got {other:?}"),
        }
    }

    #[test]
    fn cross_target_mvp2_python_parity_transport_passes_when_three_targets_match() {
        let verdict = run_fake_cross_target(CrossTargetMvp2Runners {
            rust: fake_transport_pass,
            python: fake_transport_pass,
            go: fake_transport_pass,
        });
        assert_eq!(verdict, EmitHostCrossTargetParityVerdict::Pass);
    }

    #[test]
    fn cross_target_mvp2_python_parity_transport_reports_python_setup_failure() {
        let verdict = run_fake_cross_target(CrossTargetMvp2Runners {
            rust: fake_transport_pass,
            python: fake_transport_python_setup_fail,
            go: fake_transport_pass,
        });
        assert!(matches!(
            verdict,
            EmitHostCrossTargetParityVerdict::FailSetup {
                target: "python",
                ..
            }
        ));
    }

    #[test]
    fn cross_target_mvp2_python_parity_transport_reports_python_host_exit_failure() {
        let verdict = run_fake_cross_target(CrossTargetMvp2Runners {
            rust: fake_transport_pass,
            python: fake_transport_python_exit_fail,
            go: fake_transport_pass,
        });
        assert!(matches!(
            verdict,
            EmitHostCrossTargetParityVerdict::FailHostExit {
                target: "python",
                ..
            }
        ));
    }

    #[test]
    fn cross_target_mvp2_python_parity_transport_reports_python_parse_failure() {
        let verdict = run_fake_cross_target(CrossTargetMvp2Runners {
            rust: fake_transport_pass,
            python: fake_transport_python_parse_fail,
            go: fake_transport_pass,
        });
        assert!(matches!(
            verdict,
            EmitHostCrossTargetParityVerdict::FailParse {
                target: "python",
                ..
            }
        ));
    }

    #[test]
    fn cross_target_mvp2_python_parity_transport_reports_python_behavior_mismatch() {
        let verdict = run_fake_cross_target(CrossTargetMvp2Runners {
            rust: fake_transport_pass,
            python: fake_transport_python_mismatch,
            go: fake_transport_pass,
        });
        assert!(matches!(
            verdict,
            EmitHostCrossTargetParityVerdict::FailBehaviorMismatch { .. }
        ));
    }
}
